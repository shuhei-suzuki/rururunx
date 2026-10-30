use crate::execution::strict_json::{self, Limits};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Synthetic wire profiles, not a claim about an unobserved installed CLI release.
pub const CODEX_FIXTURE_PIN: &str = "rrx-fixture-codex-v1";
pub const CLAUDE_FIXTURE_PIN: &str = "rrx-fixture-claude-v1";

pub(crate) const WIRE_JSON_LIMITS: Limits = Limits {
    frame_bytes: 1024 * 1024,
    depth: 16,
    nodes: 32768,
    string_bytes: 256 * 1024,
    total_string_bytes: 1024 * 1024,
    object_entries: 4096,
    array_entries: 4096,
};
pub(crate) const CODEX_JSONL_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy)]
pub(crate) enum WireProvider {
    Codex,
    Claude,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WireProfile {
    CodexJsonlV1,
    ClaudeResultV1,
}

impl WireProfile {
    #[cfg(test)]
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::CodexJsonlV1 => "codex-jsonl-v1",
            Self::ClaudeResultV1 => "claude-result-v1",
        }
    }
}

pub(crate) fn wire_profile(provider: WireProvider, pin: &str) -> Option<WireProfile> {
    match (provider, pin) {
        (WireProvider::Codex, CODEX_FIXTURE_PIN | "codex-cli 0.164.0") => {
            Some(WireProfile::CodexJsonlV1)
        }
        (WireProvider::Claude, CLAUDE_FIXTURE_PIN | "2.1.312 (Claude Code)") => {
            Some(WireProfile::ClaudeResultV1)
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderOutcome {
    Success,
    Limit,
    Other,
}

fn decode(bytes: &[u8]) -> Option<Value> {
    strict_json::decode(bytes, WIRE_JSON_LIMITS).ok()
}

/// Classifies complete captured JSONL only; caller must separately confirm death.
pub fn classify_codex(
    pin: &str,
    exit_code: Option<i32>,
    stdout: &[u8],
    overflow: bool,
) -> ProviderOutcome {
    classify(WireProvider::Codex, pin, exit_code, stdout, overflow)
}

fn classify(
    provider: WireProvider,
    pin: &str,
    exit_code: Option<i32>,
    stdout: &[u8],
    overflow: bool,
) -> ProviderOutcome {
    let Some(profile) = wire_profile(provider, pin) else {
        return ProviderOutcome::Other;
    };
    if overflow || exit_code.is_none() {
        return ProviderOutcome::Other;
    }
    match profile {
        WireProfile::CodexJsonlV1 => codex_jsonl_v1(exit_code, stdout),
        WireProfile::ClaudeResultV1 => claude_result_v1(exit_code, stdout),
    }
}

fn codex_jsonl_v1(exit_code: Option<i32>, stdout: &[u8]) -> ProviderOutcome {
    use ProviderOutcome::*;
    if stdout.len() > CODEX_JSONL_BYTES {
        return Other;
    }
    let mut last = None;
    let mut failed = false;
    let mut started = 0;
    let mut completed = false;
    let mut success_shape = true;
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
        if completed {
            success_shape = false;
        }
        match kind {
            "turn.started" => started += 1,
            "turn.completed" => {
                success_shape &= started == 1;
                completed = true;
            }
            "thread.started" | "item.started" | "item.updated" | "item.completed" => {}
            "error" | "turn.failed" => failed = true,
            _ => success_shape = false,
        }
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
    if exit_code == Some(0)
        && last["type"] == "turn.completed"
        && started == 1
        && success_shape
        && !failed
    {
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
    classify(WireProvider::Claude, pin, exit_code, stdout, overflow)
}

fn claude_result_v1(exit_code: Option<i32>, stdout: &[u8]) -> ProviderOutcome {
    use ProviderOutcome::*;
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
    if exit_code == Some(0)
        && envelope["subtype"] == "success"
        && envelope["is_error"] == false
        && envelope["structured_output"].is_object()
        && envelope.get("api_error").is_none_or(Value::is_null)
        && envelope.get("api_error_status").is_none_or(Value::is_null)
        && envelope
            .get("errors")
            .is_none_or(|v| v.as_array().is_some_and(Vec::is_empty))
    {
        Success
    } else {
        Other
    }
}

#[cfg(test)]
mod tests;
