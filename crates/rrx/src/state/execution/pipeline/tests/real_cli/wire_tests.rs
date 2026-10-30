use super::*;

fn encoded(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}

/// F-w4: Only a complete bounded last receipt can pass, while absence never implies validation.
#[test]
fn f_w4_last_receipt_states() {
    let valid = json!({"complete":true,"bytes":256 * 1024,"sha256":hash(b"last")});
    assert_eq!(
        last_output(WireProvider::Codex, &json!({"last":valid})),
        "passed"
    );
    for invalid in [
        Value::Null,
        json!({"complete":false,"bytes":1,"sha256":hash(b"last")}),
        json!({"complete":true,"bytes":256 * 1024 + 1,"sha256":hash(b"last")}),
        json!({"complete":true,"bytes":1,"sha256":"invalid"}),
        json!({"complete":true,"bytes":1,"sha256":"z".repeat(64)}),
    ] {
        for outcome in ["success", "limit", "other"] {
            let exit = json!({"issue":null,"last":invalid,"provider_outcome":outcome});
            assert_eq!(last_output(WireProvider::Codex, &exit), "not_checked");
            assert_eq!(last_output(WireProvider::Claude, &exit), "not_applicable");
        }
    }
    assert_eq!(last_output(WireProvider::Codex, &json!({})), "not_checked");
    assert_eq!(
        last_output(
            WireProvider::Codex,
            &json!({"last":valid,"issue":"last_output_invalid"})
        ),
        "failed"
    );
}

/// F-w5: Unprojected text, IDs and nested values cannot change either provider's shape.
#[test]
fn f_w5_shape_discards_payloads() {
    for provider in [WireProvider::Codex, WireProvider::Claude] {
        let mut shapes = Vec::new();
        for mark in ["payload_first", "payload_second"] {
            let value = match provider {
                WireProvider::Codex => {
                    json!({"type":"turn.failed","message":format!("top_{mark}"),"thread_id":format!("id_{mark}"),
                    "error":{"code":"authentication_error","message":format!("error_{mark}"),"nested":{"path":format!("path_{mark}")}},"item":{"text":mark}})
                }
                WireProvider::Claude => {
                    json!({"type":"result","subtype":"error","is_error":true,"result":format!("result_{mark}"),
                    "message":format!("top_{mark}"),"session_id":format!("id_{mark}"),"structured_output":{"text":format!("structured_{mark}")},
                    "errors":[format!("first_{mark}"),format!("second_{mark}")],"modelUsage":{"fixture-model":{"text":format!("usage_{mark}")}},
                    "api_error":{"status":401,"message":format!("api_{mark}"),"nested":{"path":format!("path_{mark}")}}})
                }
            };
            let shape = wire_shape(provider, &encoded(&value));
            assert_eq!(shape["state"], "recorded");
            assert!(!shape.to_string().contains(mark));
            shapes.push(shape);
        }
        assert_eq!(shapes[0], shapes[1]);
    }
}

/// F-w5: Raw, escaped and decoded secrets erase the entire shape, including earlier events.
#[test]
fn f_w5_secret_projection_is_atomic() {
    let secret = "EXAMPLE_SECRET=fixture";
    for provider in [WireProvider::Codex, WireProvider::Claude] {
        for field in [
            "message",
            "error",
            "structured_output",
            "modelUsage",
            "api_error",
            "errors",
        ] {
            for payload in [
                json!(secret),
                json!({"a=b\tEXAMPLE_SECRET=fixture":"value"}),
                json!([secret]),
            ] {
                let mut value = json!({"type":"result","subtype":"success","is_error":false});
                if matches!(provider, WireProvider::Codex) {
                    value["type"] = json!("turn.failed");
                }
                value[field] = payload;
                let bytes = encoded(&value);
                let mut variants = vec![
                    bytes.clone(),
                    String::from_utf8(bytes)
                        .unwrap()
                        .replace("EXAMPLE_SECRET", "\\u0045XAMPLE_SECRET")
                        .into_bytes(),
                ];
                if matches!(provider, WireProvider::Codex) {
                    for bytes in &mut variants {
                        let mut first = b"{\"type\":\"turn.started\"}\n".to_vec();
                        first.append(bytes);
                        *bytes = first;
                    }
                }
                for bytes in variants {
                    assert_eq!(wire_shape(provider, &bytes), marker("secret_detected"));
                }
            }
        }
        let key = br#"{"type":"result","a=b\tEXAMPLE_SECRET=fixture":{}}"#;
        review::scan_raw(key, 0).unwrap();
        assert_eq!(wire_shape(provider, key), marker("secret_detected"));
    }
    assert_eq!(
        finish_shape(json!({"state":"recorded","value":secret})),
        marker("secret_detected")
    );
    let mut split = vec![b' '; 8192 - 8];
    split.extend_from_slice(b"EXAMPLE_SECRET=fixture");
    assert_eq!(
        wire_shape(WireProvider::Codex, &split),
        marker("secret_detected")
    );
    for prefix in [
        b"{broken\n".to_vec(),
        b"{\"type\":null}\n".to_vec(),
        b"{\"type\":\"item.completed\"}\n".repeat(SHAPE_EVENTS + 1),
    ] {
        let mut bytes = prefix;
        bytes.extend_from_slice(
            br#"{"type":"item.completed","item":{"a=b\tEXAMPLE_SECRET=fixture":"value"}}"#,
        );
        review::scan_all_raw(&bytes).unwrap();
        assert_eq!(
            wire_shape(WireProvider::Codex, &bytes),
            marker("secret_detected")
        );
    }
}

/// F-w6: Observation records unknown event names in order and sorted failure key sets without classifying them.
#[test]
fn f_w6_order_and_keys() {
    let shape = wire_shape(WireProvider::Codex, br#"{"type":"unknown.event"}
{"type":"turn.failed","z":1,"error":{"message":"omitted","code":"authentication_error","a":"omitted"}}"#);
    assert_eq!(
        shape,
        json!({"state":"recorded","events":[{"type":"unknown.event"},
        {"type":"turn.failed","top_level_keys":["error","type","z"],"error_keys":["a","code","message"],"error_code":"authentication_error"}]})
    );
    assert_eq!(
        wire_shape(WireProvider::Codex, br#"{"type":"error"}"#)["events"][0]["error_keys"],
        "absent"
    );
    assert_eq!(
        wire_shape(WireProvider::Codex, br#"{"type":"error","error":{}}"#)["events"][0]["error_code"],
        "absent"
    );
}

/// F-w6: Explicit nulls stay distinct from missing fields and do not make the shape unrecordable.
#[test]
fn f_w6_claude_nulls() {
    for api in [
        None,
        Some(Value::Null),
        Some(json!({})),
        Some(json!({"status":Value::Null})),
        Some(json!({"status":429})),
    ] {
        for top in [None, Some(Value::Null), Some(json!(100)), Some(json!(599))] {
            let mut value =
                json!({"type":"result","subtype":"success","is_error":false,"errors":[]});
            if let Some(api) = &api {
                value["api_error"] = api.clone();
            }
            if let Some(top) = &top {
                value["api_error_status"] = top.clone();
            }
            let shape = envelope_keys(&encoded(&value));
            assert_eq!(shape["state"], "recorded");
            assert_eq!(
                shape["api_error_status"],
                top.clone().unwrap_or(json!("absent"))
            );
            assert_eq!(
                shape["api_error_status_nested"],
                api.as_ref()
                    .and_then(|a| a.get("status"))
                    .cloned()
                    .unwrap_or(json!("absent"))
            );
            if api == Some(Value::Null) {
                assert_eq!(shape["api_error_keys"], Value::Null);
            }
            if api.is_none() {
                assert_eq!(shape["api_error_keys"], "absent");
            }
        }
    }
}

/// F-w6: Invalid types, identifiers and status ranges redact every projected field.
#[test]
fn f_w6_invalid_fields_redact_atomically() {
    for (field, values) in [
        (
            "type",
            vec![
                json!(null),
                json!(false),
                json!("Bad"),
                json!("bad..type"),
                json!("bad/path"),
                json!("é"),
            ],
        ),
        ("subtype", vec![json!(null), json!(1), json!("bad-type")]),
        ("is_error", vec![json!(null), json!("false"), json!(0)]),
        ("api_error", vec![json!("error"), json!([]), json!(false)]),
        (
            "api_error_status",
            vec![
                json!(99),
                json!(600),
                json!(429.0),
                json!("429"),
                json!(false),
            ],
        ),
        (
            "errors",
            vec![
                json!(null),
                json!("error"),
                json!([1]),
                json!(["ok", false]),
            ],
        ),
    ] {
        for value in values {
            let mut input = json!({"type":"result","subtype":"success","is_error":false});
            input[field] = value;
            assert_eq!(
                envelope_keys(&encoded(&input)),
                marker("redacted"),
                "{field}"
            );
        }
    }
    for value in [json!(-1), json!(99), json!(600), json!("429"), json!(true)] {
        assert_eq!(
            envelope_keys(&encoded(&json!({"api_error":{"status":value}}))),
            marker("redacted")
        );
    }
    for value in [
        json!(null),
        json!([]),
        json!("error"),
        json!({"code":true}),
        json!({"code":"Bad"}),
    ] {
        assert_eq!(
            wire_shape(
                WireProvider::Codex,
                &encoded(&json!({"type":"error","error":value}))
            ),
            marker("redacted")
        );
    }
    assert_eq!(envelope_keys(br#"{"bad-key":1}"#), marker("redacted"));
}

/// F-w6: Shape event, key, identifier and serialized-size budgets never leave partial projections.
#[test]
fn f_w6_shape_bounds() {
    let frame = b"{\"type\":\"item.completed\"}\n";
    assert_eq!(
        wire_shape(WireProvider::Codex, &frame.repeat(SHAPE_EVENTS))["state"],
        "recorded"
    );
    assert_eq!(
        wire_shape(WireProvider::Codex, &frame.repeat(SHAPE_EVENTS + 1)),
        marker("shape_overflow")
    );
    for nested in [false, true] {
        let mut map = serde_json::Map::new();
        for n in 0..SHAPE_KEYS {
            map.insert(format!("key{n}"), Value::Null);
        }
        let value = if nested {
            json!({"api_error":map.clone()})
        } else {
            Value::Object(map.clone())
        };
        assert_eq!(envelope_keys(&encoded(&value))["state"], "recorded");
        map.insert("extra".into(), Value::Null);
        let value = if nested {
            json!({"api_error":map})
        } else {
            Value::Object(map)
        };
        assert_eq!(envelope_keys(&encoded(&value)), marker("shape_overflow"));
    }
    for n in [64, 65] {
        let input = json!({"type":"a".repeat(n)});
        assert_eq!(
            wire_shape(WireProvider::Codex, &encoded(&input))["state"],
            if n == 64 {
                "recorded"
            } else {
                "shape_overflow"
            }
        );
        let mut map = serde_json::Map::new();
        map.insert("A".repeat(n), Value::Null);
        assert_eq!(
            envelope_keys(&encoded(&Value::Object(map)))["state"],
            if n == 64 {
                "recorded"
            } else {
                "shape_overflow"
            }
        );
    }
    let shape = json!({"padding":"x".repeat(SHAPE_BYTES - 14)});
    assert_eq!(encoded(&shape).len(), SHAPE_BYTES);
    assert_eq!(finish_shape(shape.clone()), shape);
    assert_eq!(
        finish_shape(json!({"padding":"x".repeat(SHAPE_BYTES - 13)})),
        marker("shape_overflow")
    );
    // The overall bound must also apply while producing a valid many-key projection.
    let mut error = serde_json::Map::new();
    for n in 0..SHAPE_KEYS {
        error.insert(format!("key{n:060}"), Value::Null);
    }
    let mut event = encoded(&json!({"type":"error","error":error}));
    event.push(b'\n');
    assert_eq!(
        wire_shape(WireProvider::Codex, &event.repeat(100)),
        marker("shape_overflow")
    );
}

/// F-w6: Shared strict JSON budgets and diagnostics map to fixed observation states.
#[test]
fn f_w6_strict_decode_bounds() {
    use strict_json::Error::*;
    assert!(strict_json::decode(b"{}", WIRE_JSON_LIMITS).is_ok());
    let mut cases = vec![
        (vec![b' '; WIRE_JSON_LIMITS.frame_bytes + 1], FrameBytes),
        (
            encoded(&json!({"value":"x".repeat(WIRE_JSON_LIMITS.string_bytes+1)})),
            StringBytes,
        ),
        (("[".repeat(17) + "0" + &"]".repeat(17)).into_bytes(), Depth),
        (
            encoded(&json!(vec![0; WIRE_JSON_LIMITS.array_entries + 1])),
            ArrayEntries,
        ),
        (b"{broken".to_vec(), InvalidJson),
        (
            br#"{"type":"result","type":"error"}"#.to_vec(),
            DuplicateKey,
        ),
    ];
    let map: serde_json::Map<_, _> = (0..=WIRE_JSON_LIMITS.object_entries)
        .map(|i| (format!("k{i}"), Value::Null))
        .collect();
    cases.push((encoded(&Value::Object(map)), ObjectEntries));
    let nodes = json!(vec![vec![0; 4096]; 8]);
    cases.push((encoded(&nodes), Nodes));
    // Escapes keep raw bytes within the frame budget while decoded key/value bytes hit the total budget only in a custom budget.
    for (bytes, error) in cases {
        assert_eq!(
            strict_json::decode(&bytes, WIRE_JSON_LIMITS).unwrap_err(),
            error
        );
        assert_eq!(envelope_keys(&bytes), marker(decode_state(error)));
    }
    assert_eq!(decode_state(TotalStringBytes), "strict_json_limit");
    assert_eq!(decode_state(InvalidLimits), "absent_or_invalid");
    let errors = json!({"errors":vec!["omitted";4096]});
    assert_eq!(envelope_keys(&encoded(&errors))["errors_count"], 4096);
    assert_eq!(
        envelope_keys(&encoded(&json!({"errors":vec!["omitted";4097]}))),
        marker("strict_json_limit")
    );
}

fn step_receipt(bytes: &[u8]) -> PipelineStep {
    let mut step = super::super::b5::step();
    step.body["exit"] = json!({"stdout_truncated":false,"stdout":{"complete":true,"bytes":bytes.len(),"sha256":hash(bytes)}});
    step
}

/// F-w6: Receipt integrity, regular files and complete capture are prerequisites to observation.
#[test]
fn f_w6_receipts_and_capture_bound() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let path = root.join("stdout.jsonl");
    let bytes = b"{\"type\":\"turn.started\"}\n";
    let step = step_receipt(bytes);
    assert_eq!(
        observed_shape(&root, &step, WireProvider::Codex),
        marker("absent_or_invalid")
    );
    fs::write(&path, bytes).unwrap();
    assert_eq!(
        observed_shape(&root, &step, WireProvider::Codex)["state"],
        "recorded"
    );
    for pointer in [
        "/exit/stdout/complete",
        "/exit/stdout/bytes",
        "/exit/stdout/sha256",
        "/exit/stdout_truncated",
    ] {
        let mut invalid = step.clone();
        *invalid.body.pointer_mut(pointer).unwrap() = match pointer {
            "/exit/stdout/complete" => json!(false),
            "/exit/stdout/bytes" => json!(1),
            "/exit/stdout/sha256" => json!("0".repeat(64)),
            _ => json!(true),
        };
        assert_eq!(
            observed_shape(&root, &invalid, WireProvider::Codex),
            marker("absent_or_invalid")
        );
    }
    fs::remove_file(&path).unwrap();
    let target = root.join("target");
    fs::write(&target, bytes).unwrap();
    std::os::unix::fs::symlink(&target, &path).unwrap();
    assert_eq!(
        observed_shape(&root, &step, WireProvider::Codex),
        marker("absent_or_invalid")
    );
    fs::remove_file(&path).unwrap();
    fs::hard_link(&target, &path).unwrap();
    assert_eq!(
        observed_shape(&root, &step, WireProvider::Codex),
        marker("absent_or_invalid")
    );
    fs::remove_file(&path).unwrap();
    let mut boundary = bytes.to_vec();
    while boundary.len() < CODEX_JSONL_BYTES {
        let count = (CODEX_JSONL_BYTES - boundary.len()).min(WIRE_JSON_LIMITS.frame_bytes);
        boundary.extend(std::iter::repeat_n(b' ', count - 1));
        boundary.push(b'\n');
    }
    fs::write(&path, &boundary).unwrap();
    let step = step_receipt(&boundary);
    assert_eq!(
        observed_shape(&root, &step, WireProvider::Codex)["state"],
        "recorded"
    );
    boundary.push(b' ');
    fs::write(&path, &boundary).unwrap();
    let mut truncated = step_receipt(&boundary);
    truncated.body["exit"]["stdout_truncated"] = json!(true);
    assert_eq!(
        observed_shape(&root, &truncated, WireProvider::Codex),
        marker("absent_or_invalid")
    );
    assert_eq!(
        wire_shape(WireProvider::Codex, &boundary),
        marker("absent_or_invalid")
    );
}

/// F-w6: Recordability uses the product outcome and last receipt without reclassifying recorded wire shapes.
#[test]
fn f_w6_pass_uses_product_outcome() {
    let mut wire = json!({"shape":{"state":"recorded","events":[{"type":"new.event"}]},"provider_outcome":"success","last_output":"passed"});
    assert!(wire_recordable(&wire));
    for outcome in ["limit", "other"] {
        wire["provider_outcome"] = json!(outcome);
        assert!(!wire_recordable(&wire));
    }
    wire["provider_outcome"] = json!("success");
    for state in [
        "secret_detected",
        "redacted",
        "shape_overflow",
        "strict_json_limit",
        "absent_or_invalid",
        "not_run",
    ] {
        wire["shape"] = marker(state);
        assert!(!wire_recordable(&wire));
    }
    wire["shape"] = marker("recorded");
    for last in ["failed", "not_checked"] {
        wire["last_output"] = json!(last);
        assert!(!wire_recordable(&wire));
    }
}

/// F-w6: Native fake CLIs record real pin shapes and stop subsequent Agents on product or observation failure.
#[tokio::test]
async fn f_w6_fake_wire_passes() {
    for scenario in [
        FakeScenario {
            pins: Some(("codex-cli 0.164.0", "2.1.312 (Claude Code)")),
            ..FAKE_PASS
        },
        FakeScenario {
            pins: Some(("codex-cli unknown", "2.1.312 (Claude Code)")),
            ..FAKE_PASS
        },
        FakeScenario {
            codex_output: "{\"type\":\"new.event\"}\n",
            ..FAKE_PASS
        },
        FakeScenario {
            codex_last: None,
            ..FAKE_PASS
        },
        FakeScenario {
            claude_output: Some("{\"type\":\"result\",\"is_error\":true}"),
            ..FAKE_PASS
        },
        FakeScenario {
            overflow_shape: true,
            ..FAKE_PASS
        },
        FakeScenario {
            claude_output: Some(
                "{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"structured_output\":{},\"bad-key\":1}",
            ),
            ..FAKE_PASS
        },
    ] {
        let out = tempfile::tempdir().unwrap();
        let summary = run_pass(PassMode::Fake(scenario), out.path()).await;
        let implement = &summary["steps"][1];
        assert_eq!(
            implement["wire"]["shape"]["state"],
            if scenario.overflow_shape {
                "shape_overflow"
            } else {
                "recorded"
            }
        );
        if scenario.pins == Some(("codex-cli 0.164.0", "2.1.312 (Claude Code)")) {
            assert_eq!(summary["state"], "adopted");
            assert_eq!(implement["wire"]["profile"], "codex-jsonl-v1");
            assert_eq!(summary["steps"][3]["wire"]["profile"], "claude-result-v1");
        } else {
            assert_eq!(summary["state"], "failed");
            if scenario.claude_output.is_none() {
                assert_eq!(summary["steps"][3]["wire"], marker("not_run"));
                assert!(summary["prompts"].get("review").is_none());
            }
        }
        if scenario.pins.is_some_and(|p| p.0 == "codex-cli unknown") {
            assert_eq!(implement["wire"]["profile"], "unmapped");
            assert_eq!(implement["wire"]["provider_outcome"], "other");
            assert_eq!(implement["reason"], "agent_failed");
        }
        if scenario.codex_last.is_none() {
            assert_eq!(implement["wire"]["last_output"], "failed");
        }
        if scenario.overflow_shape {
            assert_eq!(implement["wire"]["provider_outcome"], "success");
        }
    }
}
