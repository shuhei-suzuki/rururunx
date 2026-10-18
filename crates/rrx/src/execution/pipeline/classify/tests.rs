use super::*;

const CODEX_SUCCESS: &[u8] =
    include_bytes!("../../../../tests/fixtures/pipeline/codex-success.jsonl");
const CODEX_LIMIT: &[u8] = include_bytes!("../../../../tests/fixtures/pipeline/codex-limit.jsonl");
const CLAUDE_SUCCESS: &[u8] =
    include_bytes!("../../../../tests/fixtures/pipeline/claude-success.json");
const CLAUDE_LIMIT: &[u8] = include_bytes!("../../../../tests/fixtures/pipeline/claude-limit.json");

/// Only a zero exit and an error-free final turn completion are known Codex success.
#[test]
fn b3_codex_success() {
    assert_eq!(
        classify_codex(CODEX_FIXTURE_PIN, Some(0), CODEX_SUCCESS, false),
        ProviderOutcome::Success
    );
    for (exit, output, overflow) in [
        (Some(1), CODEX_SUCCESS, false),
        (None, CODEX_SUCCESS, false),
        (Some(0), CODEX_SUCCESS, true),
        (
            Some(0),
            b"{\"type\":\"error\"}\n{\"type\":\"turn.completed\"}".as_slice(),
            false,
        ),
        (
            Some(0),
            b"{\"type\":\"turn.completed\"}\n{\"type\":\"item.completed\"}",
            false,
        ),
    ] {
        assert_eq!(
            classify_codex(CODEX_FIXTURE_PIN, exit, output, overflow),
            ProviderOutcome::Other
        );
    }
}
/// Only the complete successful Claude envelope and zero exit are known success.
#[test]
fn b3_claude_success() {
    assert_eq!(
        classify_claude(CLAUDE_FIXTURE_PIN, Some(0), CLAUDE_SUCCESS, false),
        ProviderOutcome::Success
    );
    for (exit, output, overflow) in [
        (Some(1), CLAUDE_SUCCESS, false),
        (None, CLAUDE_SUCCESS, false),
        (Some(0), CLAUDE_SUCCESS, true),
        (
            Some(0),
            b"{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":true}".as_slice(),
            false,
        ),
        (
            Some(0),
            b"{\"type\":\"result\",\"subtype\":\"success\"}",
            false,
        ),
    ] {
        assert_eq!(
            classify_claude(CLAUDE_FIXTURE_PIN, exit, output, overflow),
            ProviderOutcome::Other
        );
    }
}
/// Codex limits require the final error event's fixed code and exact message.
#[test]
fn b3_codex_limit() {
    assert_eq!(
        classify_codex(CODEX_FIXTURE_PIN, Some(1), CODEX_LIMIT, false),
        ProviderOutcome::Limit
    );
    for (from, to) in [
        ("rate_limit_exceeded", "authentication_error"),
        ("Rate limit reached", "Some Rate limit reached text"),
        ("turn.failed", "item.completed"),
    ] {
        let changed = String::from_utf8(CODEX_LIMIT.to_vec())
            .unwrap()
            .replace(from, to);
        assert_eq!(
            classify_codex(CODEX_FIXTURE_PIN, Some(1), changed.as_bytes(), false),
            ProviderOutcome::Other
        );
    }
}
/// Claude limits require an error envelope, API status and exact limit text.
#[test]
fn b3_claude_limit() {
    assert_eq!(
        classify_claude(CLAUDE_FIXTURE_PIN, Some(1), CLAUDE_LIMIT, false),
        ProviderOutcome::Limit
    );
    for (from, to) in [
        ("true", "false"),
        ("429", "401"),
        ("Rate limit exceeded", "Authentication failed"),
        ("result\"", "other\""),
    ] {
        let changed = String::from_utf8(CLAUDE_LIMIT.to_vec())
            .unwrap()
            .replacen(from, to, 1);
        assert_eq!(
            classify_claude(CLAUDE_FIXTURE_PIN, Some(1), changed.as_bytes(), false),
            ProviderOutcome::Other
        );
    }
}
/// Missing, malformed, duplicated and unrecognized wire output never clears a wait.
#[test]
fn b3_classifier_unknown_output() {
    for output in [
        b"".as_slice(),
        b"not json",
        b"{\"type\":\"turn.completed\",\"type\":\"turn.failed\"}",
        b"{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":true,\"is_error\":false}",
    ] {
        assert_eq!(
            classify_codex(CODEX_FIXTURE_PIN, Some(0), output, false),
            ProviderOutcome::Other
        );
        assert_eq!(
            classify_claude(CLAUDE_FIXTURE_PIN, Some(0), output, false),
            ProviderOutcome::Other
        );
    }
}
/// Unqualified wire versions cannot inherit fixture success or limit rules.
#[test]
fn b3_classifier_unknown_pin() {
    for bytes in [CODEX_SUCCESS, CODEX_LIMIT] {
        assert_eq!(
            classify_codex("unknown", Some(0), bytes, false),
            ProviderOutcome::Other
        );
    }
    for bytes in [CLAUDE_SUCCESS, CLAUDE_LIMIT] {
        assert_eq!(
            classify_claude("unknown", Some(0), bytes, false),
            ProviderOutcome::Other
        );
    }
}
