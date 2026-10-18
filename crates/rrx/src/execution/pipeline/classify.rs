use crate::execution::strict_json::{self, Limits};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Synthetic wire profiles, not a claim about an unobserved installed CLI release.
pub const CODEX_FIXTURE_PIN: &str = "rrx-fixture-codex-v1";
pub const CLAUDE_FIXTURE_PIN: &str = "rrx-fixture-claude-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderOutcome {
    Success,
    Limit,
    Other,
}

fn decode(bytes: &[u8]) -> Option<Value> {
    strict_json::decode(
        bytes,
        Limits {
            frame_bytes: 1024 * 1024,
            depth: 16,
            nodes: 32768,
            string_bytes: 256 * 1024,
            total_string_bytes: 1024 * 1024,
            object_entries: 4096,
            array_entries: 4096,
        },
    )
    .ok()
}

/// Classifies complete captured JSONL only; caller must separately confirm death.
pub fn classify_codex(
    pin: &str,
    exit_code: Option<i32>,
    stdout: &[u8],
    overflow: bool,
) -> ProviderOutcome {
    use ProviderOutcome::*;
    if pin != CODEX_FIXTURE_PIN || overflow || stdout.len() > 16 * 1024 * 1024 {
        return Other;
    }
    let mut last = None;
    let mut failed = false;
    for line in stdout
        .split(|b| *b == b'\n')
        .filter(|line| !line.iter().all(u8::is_ascii_whitespace))
    {
        let Some(event) = decode(line) else {
            return Other;
        };
        let Some(kind) = event["type"].as_str() else {
            return Other;
        };
        if matches!(kind, "error" | "turn.failed") {
            failed = true;
        }
        last = Some(event);
    }
    let Some(last) = last else {
        return Other;
    };
    if matches!(last["type"].as_str(), Some("error" | "turn.failed"))
        && last["error"]["code"] == "rate_limit_exceeded"
        && last["error"]["message"] == "Rate limit reached"
    {
        return Limit;
    }
    if exit_code == Some(0) && last["type"] == "turn.completed" && !failed {
        Success
    } else {
        Other
    }
}

/// Classifies a complete result envelope, never free text or stderr substrings.
pub fn classify_claude(
    pin: &str,
    exit_code: Option<i32>,
    stdout: &[u8],
    overflow: bool,
) -> ProviderOutcome {
    use ProviderOutcome::*;
    if pin != CLAUDE_FIXTURE_PIN || overflow {
        return Other;
    }
    let Some(envelope) = decode(stdout) else {
        return Other;
    };
    if envelope["type"] != "result" {
        return Other;
    }
    if envelope["is_error"] == true
        && envelope["api_error"]["status"] == 429
        && envelope["result"] == "Rate limit exceeded"
    {
        return Limit;
    }
    if exit_code == Some(0) && envelope["subtype"] == "success" && envelope["is_error"] == false {
        Success
    } else {
        Other
    }
}

#[cfg(test)]
mod tests;
