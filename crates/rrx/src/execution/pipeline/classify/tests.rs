use super::*;

const CODEX_SUCCESS: &[u8] =
    include_bytes!("../../../../tests/fixtures/pipeline/codex-success.jsonl");
const CODEX_LIMIT: &[u8] = include_bytes!("../../../../tests/fixtures/pipeline/codex-limit.jsonl");
const CLAUDE_SUCCESS: &[u8] =
    include_bytes!("../../../../tests/fixtures/pipeline/claude-success.json");
const CLAUDE_314_SUCCESS: &[u8] =
    include_bytes!("../../../../tests/fixtures/pipeline/claude-2.1.314-success.json");
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
    for pin in [
        CLAUDE_FIXTURE_PIN,
        "2.1.312 (Claude Code)",
        "2.1.314 (Claude Code)",
    ] {
        assert_eq!(
            classify_claude(pin, Some(1), CLAUDE_LIMIT, false),
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
                classify_claude(pin, Some(1), changed.as_bytes(), false),
                ProviderOutcome::Other
            );
        }
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

/// F-w1: Only exact provider and pin pairs can select an implemented profile.
#[test]
fn f_w1_exact_provider_pins() {
    for (provider, pin, success, limit) in [
        (
            WireProvider::Codex,
            "codex-cli 0.164.0",
            CODEX_SUCCESS,
            CODEX_LIMIT,
        ),
        (
            WireProvider::Claude,
            "2.1.312 (Claude Code)",
            CLAUDE_SUCCESS,
            CLAUDE_LIMIT,
        ),
        (
            WireProvider::Claude,
            "2.1.314 (Claude Code)",
            CLAUDE_314_SUCCESS,
            CLAUDE_LIMIT,
        ),
        (
            WireProvider::Codex,
            CODEX_FIXTURE_PIN,
            CODEX_SUCCESS,
            CODEX_LIMIT,
        ),
        (
            WireProvider::Claude,
            CLAUDE_FIXTURE_PIN,
            CLAUDE_SUCCESS,
            CLAUDE_LIMIT,
        ),
    ] {
        assert!(wire_profile(provider, pin).is_some());
        let foreign = match provider {
            WireProvider::Codex => CLAUDE_FIXTURE_PIN,
            WireProvider::Claude => CODEX_FIXTURE_PIN,
        };
        for changed in [
            "unknown".to_owned(),
            pin.to_ascii_uppercase(),
            format!("{pin} suffix"),
            format!("prefix {pin}"),
            foreign.to_owned(),
        ] {
            assert!(wire_profile(provider, &changed).is_none());
            for bytes in [success, limit] {
                assert_eq!(
                    classify(provider, &changed, Some(0), bytes, false),
                    ProviderOutcome::Other
                );
            }
        }
    }
    assert!(wire_profile(WireProvider::Codex, "2.1.312 (Claude Code)").is_none());
    assert!(wire_profile(WireProvider::Claude, "codex-cli 0.164.0").is_none());
    for (provider, pin) in [
        (WireProvider::Claude, "2.1.313 (Claude Code)"),
        (WireProvider::Claude, "2.1.314"),
        (WireProvider::Claude, "2.1.3140 (Claude Code)"),
        (WireProvider::Codex, "2.1.314 (Claude Code)"),
    ] {
        assert!(wire_profile(provider, pin).is_none());
        for output in [CLAUDE_314_SUCCESS, CLAUDE_LIMIT] {
            assert_eq!(
                classify(provider, pin, Some(0), output, false),
                ProviderOutcome::Other
            );
        }
    }
}

/// F-w2: Real and synthetic pins share success, fixed limit and authentication-failure behavior.
#[test]
fn f_w2_real_pin_fixtures() {
    for (provider, pins, success, limit) in [
        (
            WireProvider::Codex,
            [CODEX_FIXTURE_PIN, "codex-cli 0.164.0"].as_slice(),
            CODEX_SUCCESS,
            CODEX_LIMIT,
        ),
        (
            WireProvider::Claude,
            [CLAUDE_FIXTURE_PIN, "2.1.312 (Claude Code)"].as_slice(),
            CLAUDE_SUCCESS,
            CLAUDE_LIMIT,
        ),
        (
            WireProvider::Claude,
            ["2.1.314 (Claude Code)"].as_slice(),
            CLAUDE_314_SUCCESS,
            CLAUDE_LIMIT,
        ),
    ] {
        for pin in pins {
            assert_eq!(
                classify(provider, pin, Some(0), success, false),
                ProviderOutcome::Success
            );
            assert_eq!(
                classify(provider, pin, Some(1), limit, false),
                ProviderOutcome::Limit
            );
            let auth = String::from_utf8(limit.to_vec())
                .unwrap()
                .replace("rate_limit_exceeded", "authentication_error")
                .replace("429", "401")
                .replace("Rate limit", "Authentication");
            assert_eq!(
                classify(provider, pin, Some(1), auth.as_bytes(), false),
                ProviderOutcome::Other
            );
            assert_eq!(
                classify(provider, pin, None, limit, false),
                ProviderOutcome::Other
            );
            for (exit, overflow, bytes) in [
                (Some(1), false, success),
                (None, false, success),
                (Some(0), true, success),
                (Some(0), false, &success[..success.len() / 2]),
                (Some(0), false, b"unknown".as_slice()),
            ] {
                assert_eq!(
                    classify(provider, pin, exit, bytes, overflow),
                    ProviderOutcome::Other
                );
            }
        }
    }
}

/// F-w3: A single start before a single terminal completion and only known intervening events can succeed.
#[test]
fn f_w3_codex_sequence() {
    for types in [
        vec!["turn.completed"],
        vec!["turn.started", "turn.started", "turn.completed"],
        vec!["turn.completed", "turn.started"],
        vec!["turn.started", "turn.completed", "turn.completed"],
        vec!["turn.started", "turn.completed", "item.completed"],
        vec!["turn.started", "new.event", "turn.completed"],
        vec!["turn.started", "error", "turn.completed"],
        vec!["turn.started", "turn.failed", "turn.completed"],
    ] {
        let output = types
            .iter()
            .map(|kind| serde_json::json!({"type":kind}).to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            classify_codex("codex-cli 0.164.0", Some(0), output.as_bytes(), false),
            ProviderOutcome::Other,
            "{types:?}"
        );
    }
    let output = [
        "thread.started",
        "turn.started",
        "item.started",
        "item.updated",
        "item.completed",
        "turn.completed",
    ]
    .map(|kind| serde_json::json!({"type":kind}).to_string())
    .join("\n");
    assert_eq!(
        classify_codex("codex-cli 0.164.0", Some(0), output.as_bytes(), false),
        ProviderOutcome::Success
    );
}

/// F-w3: Contradictory error fields or nonobject structured output cannot be Claude success.
#[test]
fn f_w3_claude_contradictions() {
    for (pin, fixture) in [
        ("2.1.312 (Claude Code)", CLAUDE_SUCCESS),
        ("2.1.314 (Claude Code)", CLAUDE_314_SUCCESS),
    ] {
        use serde_json::json;
        let success: Value = serde_json::from_slice(fixture).unwrap();
        for (field, values) in [
            ("type", vec![Value::Null, json!(1), json!("other")]),
            ("subtype", vec![Value::Null, json!(true), json!("error")]),
            ("is_error", vec![Value::Null, json!("false"), json!(true)]),
            (
                "api_error",
                vec![
                    json!({}),
                    json!({"status":429}),
                    json!("error"),
                    json!(false),
                ],
            ),
            (
                "api_error_status",
                vec![json!(429), json!("429"), json!({}), json!(false)],
            ),
            (
                "errors",
                vec![
                    json!(["failed"]),
                    json!([1]),
                    json!({}),
                    Value::Null,
                    json!("failed"),
                ],
            ),
            (
                "structured_output",
                vec![Value::Null, json!("{}"), json!([]), json!(false)],
            ),
        ] {
            for value in values {
                let mut changed = success.clone();
                changed[field] = value;
                assert_eq!(
                    classify_claude(pin, Some(0), &serde_json::to_vec(&changed).unwrap(), false),
                    ProviderOutcome::Other,
                    "{field}"
                );
            }
        }
        let mut changed = success.clone();
        changed.as_object_mut().unwrap().remove("structured_output");
        assert_eq!(
            classify_claude(pin, Some(0), &serde_json::to_vec(&changed).unwrap(), false),
            ProviderOutcome::Other
        );
        let mut allowed = success;
        allowed["api_error"] = Value::Null;
        allowed["api_error_status"] = Value::Null;
        allowed["errors"] = json!([]);
        assert_eq!(
            classify_claude(pin, Some(0), &serde_json::to_vec(&allowed).unwrap(), false),
            ProviderOutcome::Success
        );
    }
}

/// F-w3: Raw duplicate keys and decoder budgets apply to both real profiles.
#[test]
fn f_w3_invalid_and_bounded_json() {
    for (provider, pin, valid) in [
        (WireProvider::Codex, "codex-cli 0.164.0", CODEX_SUCCESS),
        (
            WireProvider::Claude,
            "2.1.312 (Claude Code)",
            CLAUDE_SUCCESS,
        ),
        (
            WireProvider::Claude,
            "2.1.314 (Claude Code)",
            CLAUDE_314_SUCCESS,
        ),
    ] {
        let duplicate = String::from_utf8(valid.to_vec()).unwrap().replacen(
            "\"type\":",
            "\"type\":null,\"type\":",
            1,
        );
        assert_eq!(
            classify(provider, pin, Some(0), duplicate.as_bytes(), false),
            ProviderOutcome::Other
        );
        for (bytes, outcome) in [
            (WIRE_JSON_LIMITS.string_bytes, ProviderOutcome::Success),
            (WIRE_JSON_LIMITS.string_bytes + 1, ProviderOutcome::Other),
        ] {
            let padded = match provider {
                WireProvider::Codex => std::str::from_utf8(valid).unwrap().replacen(
                    "\"thread_id\":\"fixture\"",
                    &format!("\"thread_id\":\"{}\"", "x".repeat(bytes)),
                    1,
                ),
                WireProvider::Claude => {
                    let mut envelope: Value = serde_json::from_slice(valid).unwrap();
                    envelope["padding"] = serde_json::json!("x".repeat(bytes));
                    envelope.to_string()
                }
            };
            assert_eq!(
                classify(provider, pin, Some(0), padded.as_bytes(), false),
                outcome
            );
        }
        assert_eq!(
            classify(
                provider,
                pin,
                Some(0),
                &vec![b' '; WIRE_JSON_LIMITS.frame_bytes + 1],
                false
            ),
            ProviderOutcome::Other
        );
    }
}
