use super::*;
use crate::execution::claude_wire::{Metrics, RunState};
use crate::execution::native_result::{AcquisitionStatus, Collector};
use std::path::Path;

fn decode(value: Value) -> AdapterResult<ValidatedClaudeFrame> {
    ValidatedClaudeFrame::decode(value, &decoder_selfcheck()?)
}
fn result(error: bool) -> Value {
    if error {
        json!({"type":"result","session_id":"n","subtype":"error_during_execution","is_error":true,"errors":["ordinary failure"]})
    } else {
        json!({"type":"result","session_id":"n","subtype":"success","is_error":false,"result":"answer"})
    }
}
fn initialized() -> RunState {
    let mut state = RunState::default();
    let init = decode(json!({"type":"system","subtype":"init","session_id":"n","cwd":"/owned","tools":[],"mcp_servers":[]})).unwrap();
    state
        .observe(&init, "n", Path::new("/owned"), false)
        .unwrap();
    state
}

// CL-10: both branches share exactly the same telemetry contract. Absent,
// null and zero survive decoding distinctly; optional usage is not invented.
#[test]
fn cl10_presence_null_zero_and_domain_controls_cover_success_and_error() {
    for error in [false, true] {
        let absent = decode(result(error)).unwrap();
        let telemetry = absent.result().unwrap().telemetry();
        assert_eq!(telemetry.usage, Presence::Absent);
        assert_eq!(telemetry.input, Presence::Absent);
        assert_eq!(telemetry.cost, Presence::Absent);
        assert_eq!(telemetry.api_ms, Presence::Absent);
        for usage in [
            Value::Null,
            json!({}),
            json!({"input_tokens":null,"output_tokens":null,"cache_read_input_tokens":null,"cache_creation_input_tokens":null}),
        ] {
            let mut value = result(error);
            value["usage"] = usage.clone();
            value["total_cost_usd"] = Value::Null;
            value["duration_api_ms"] = Value::Null;
            let frame = decode(value).unwrap();
            let telemetry = frame.result().unwrap().telemetry();
            assert_eq!(
                telemetry.usage,
                if usage.is_null() {
                    Presence::Null
                } else {
                    Presence::Value(())
                }
            );
            assert_eq!(
                telemetry.input,
                if usage.get("input_tokens").is_some() {
                    Presence::Null
                } else {
                    Presence::Absent
                }
            );
            assert_eq!(telemetry.cost, Presence::Null);
            assert_eq!(telemetry.api_ms, Presence::Null);
            let metrics = Metrics::from_result(frame.result().unwrap(), false);
            assert_eq!(
                (metrics.input(), metrics.cost(), metrics.api_ms()),
                (None, None, None)
            );
        }
        for n in [0, u64::MAX] {
            let mut value = result(error);
            value["usage"] = json!({"input_tokens":n,"output_tokens":n,"cache_read_input_tokens":n,"cache_creation_input_tokens":n});
            value["duration_api_ms"] = json!(n);
            value["total_cost_usd"] = json!(0.0);
            let frame = decode(value).unwrap();
            let metrics = Metrics::from_result(frame.result().unwrap(), false);
            assert_eq!(
                (
                    metrics.input(),
                    metrics.output(),
                    metrics.cache_read(),
                    metrics.cache_write(),
                    metrics.api_ms()
                ),
                (Some(n), Some(n), Some(n), Some(n), Some(n))
            );
            assert_eq!(metrics.cost(), Some(0.0));
        }
        let mut value = result(error);
        value["usage"] = json!({"input_tokens":1.0});
        value["duration_api_ms"] = json!(2.0);
        value["total_cost_usd"] = json!(f64::MAX);
        let frame = decode(value).unwrap();
        let metrics = Metrics::from_result(frame.result().unwrap(), false);
        assert_eq!(
            (metrics.input(), metrics.api_ms(), metrics.cost()),
            (Some(1), Some(2), Some(f64::MAX))
        );
    }
}

#[test]
fn cl10_each_telemetry_mutation_is_rejected_before_owned_or_nonhuman_consumption() {
    for error in [false, true] {
        for origin in [
            Value::Null,
            json!({"kind":"human"}),
            json!({"kind":"peer","from":"p"}),
        ] {
            for foreign in [false, true] {
                let mut control = result(error);
                control["origin"] = origin.clone();
                if foreign {
                    control["session_id"] = json!("foreign");
                }
                assert!(decode(control.clone()).is_ok());
                for bad in [
                    json!(false),
                    json!("0"),
                    json!([]),
                    json!(-1),
                    json!(0.5),
                    json!(18_446_744_073_709_551_616.0),
                ] {
                    for key in [
                        "input_tokens",
                        "output_tokens",
                        "cache_read_input_tokens",
                        "cache_creation_input_tokens",
                    ] {
                        let mut mutant = control.clone();
                        mutant["usage"] = json!({key:bad.clone()});
                        assert!(
                            decode(mutant).is_err(),
                            "{error}/{origin}/{foreign}/{key}/{bad}"
                        );
                    }
                    let mut mutant = control.clone();
                    mutant["duration_api_ms"] = bad.clone();
                    assert!(decode(mutant).is_err());
                }
                for bad in [json!(false), json!("0"), json!([]), json!(0)] {
                    let mut mutant = control.clone();
                    mutant["usage"] = bad;
                    assert!(decode(mutant).is_err());
                }
                for bad in [json!(false), json!("0"), json!([]), json!(-0.01)] {
                    let mut mutant = control.clone();
                    mutant["total_cost_usd"] = bad;
                    assert!(decode(mutant).is_err());
                }
            }
        }
    }
    // Original-byte overflow cannot be rounded into a finite valid value.
    assert!(strict_json::decode(b"{\"total_cost_usd\":1e400}", limits()).is_err());
}

#[test]
fn cl10_invalid_result_never_enters_runstate_or_collector_even_after_valid_unfinished_result() {
    for prior_unfinished in [false, true] {
        let mut state = initialized();
        let mut collector = Collector::default();
        if prior_unfinished {
            let running = decode(
                json!({"type":"system","subtype":"session_state_changed","state":"running"}),
            )
            .unwrap();
            state
                .observe(&running, "n", Path::new("/owned"), false)
                .unwrap();
            let prior = decode(result(false)).unwrap();
            state
                .observe(&prior, "n", Path::new("/owned"), false)
                .unwrap();
            collector.claude_terminal(prior.result().unwrap());
            assert!(!state.complete());
        }
        let mut mutant = result(false);
        mutant["usage"] = json!({"input_tokens":"wrongtype"});
        let error = decode(mutant).err().unwrap();
        assert_eq!(error.kind, ErrorKind::ParseFailure);
        // This is the same gate as Core::claude: no validated result means no
        // observe/collector/metrics/confirm call. A pre-complete candidate is
        // not a captured NativeTerminal. Core terminal/CAS controls live in
        // installation tests; this pure control does not prove settlement.
        collector.protocol_lost();
        assert!(!state.complete());
        assert_ne!(collector.content().acquisition, AcquisitionStatus::Complete);
        assert!(collector.content().text.is_none());
        if !prior_unfinished {
            assert!(state.result.is_none());
            assert!(collector.content().terminal_sha256.is_none());
        }
    }
}

#[test]
fn cl10_structured_output_is_bounded_but_never_answer_authority() {
    for structured in [None, Some(Value::Null), Some(json!({"decision":"APPROVE"}))] {
        let mut value = result(false);
        if let Some(structured) = &structured {
            value["structured_output"] = structured.clone();
        }
        let frame = decode(value).unwrap();
        let mut collector = Collector::default();
        collector.claude_terminal(frame.result().unwrap());
        collector.confirm_complete();
        let content = collector.content();
        if structured.as_ref().is_some_and(|v| !v.is_null()) {
            assert_eq!(content.acquisition, AcquisitionStatus::Unsupported);
            assert!(content.text.is_none());
        } else {
            assert_eq!(content.acquisition, AcquisitionStatus::Complete);
            assert_eq!(content.text.as_deref(), Some("answer"));
        }
    }
    let mut value = result(false);
    value["structured_output"] = json!("s".repeat(1024 * 1024 + 1));
    assert!(decode(value).is_err());
}

#[test]
fn cl4_origin_all_finite_branches_and_one_field_mutations() {
    let origins = [
        json!({"kind":"human"}),
        json!({"kind":"channel","server":"mcp"}),
        json!({"kind":"peer","from":"p","fromMode":"bypass","name":"peer","fromSession":"s","senderTaskId":"t","body":"body","verifiedPeerPid":0}),
        json!({"kind":"task-notification","subkind":"scheduled-trigger","fireReason":"timer","runId":"r"}),
        json!({"kind":"coordinator"}),
        json!({"kind":"auto-continuation"}),
        json!({"kind":"unclassified"}),
    ];
    for origin in origins {
        let mut value = result(false);
        value["origin"] = origin.clone();
        let frame = decode(value.clone()).unwrap();
        assert_eq!(frame.human_origin(), origin["kind"] == "human");
        let mut state = initialized();
        assert_eq!(
            state
                .observe(&frame, "n", Path::new("/owned"), false)
                .unwrap(),
            frame.human_origin()
        );
        assert_eq!(state.complete(), frame.human_origin());
        for key in origin.as_object().unwrap().keys() {
            let mut mutant = value.clone();
            mutant["origin"][key] = Value::Null;
            assert!(
                decode(mutant).is_err(),
                "optional null cannot become absence: {key}"
            );
        }
    }
    for origin in [
        json!({}),
        json!({"kind":"channel"}),
        json!({"kind":"peer"}),
        json!({"kind":"hook"}),
        json!({"kind":"mcp-channel"}),
        json!({"kind":"peer","from":"p","fromMode":"new"}),
        json!({"kind":"peer","from":"p","verifiedPeerPid":-1}),
        json!({"kind":"task-notification","subkind":"new"}),
    ] {
        let mut value = result(false);
        value["origin"] = origin;
        assert!(decode(value).is_err());
    }
}

#[test]
fn cl4_error_result_checks_all_errors_before_classification_limit() {
    for error in [false, true] {
        let mut value = result(error);
        value["errors"] = json!(vec!["ordinary"; 65]);
        assert!(decode(value.clone()).is_ok());
        value["errors"][64] = json!({"malformed":"not a string"});
        assert!(decode(value).is_err());
    }
    for key in ["subtype", "is_error", "errors", "session_id"] {
        let mut value = result(true);
        value.as_object_mut().unwrap().remove(key);
        assert!(decode(value).is_err(), "{key}");
    }
}

#[test]
fn cl10_api_status_and_result_branch_presence_are_checked_before_error_hints() {
    for error in [false, true] {
        for status in [Value::Null, json!(0), json!(65535)] {
            let mut control = result(error);
            control["api_error_status"] = status;
            assert!(decode(control).is_ok());
        }
        for status in [
            json!("429"),
            json!(false),
            json!(-1),
            json!(429.5),
            json!(65536),
        ] {
            let mut mutant = result(error);
            mutant["api_error_status"] = status;
            assert!(decode(mutant).is_err());
        }
    }
    let mut control = result(true);
    control["result"] = json!("error text");
    assert!(decode(control.clone()).is_ok());
    control["result"] = Value::Null;
    assert!(decode(control).is_err());
    let mut control = result(false);
    control.as_object_mut().unwrap().remove("result");
    assert!(decode(control).is_err());
}

#[test]
fn cl4_assistant_error_enum_and_user_tool_result_content_are_finite() {
    let assistant = json!({"type":"assistant","session_id":"n","parent_tool_use_id":null,"message":{"role":"assistant","content":[]}});
    for error in [
        "authentication_failed",
        "oauth_org_not_allowed",
        "account_on_hold",
        "billing_error",
        "rate_limit",
        "overloaded",
        "invalid_request",
        "model_not_found",
        "server_error",
        "max_output_tokens",
        "cloud_credential_error",
        "unknown",
    ] {
        let mut control = assistant.clone();
        control["error"] = json!(error);
        assert!(decode(control).is_ok());
    }
    for error in [Value::Null, json!(false), json!("future_error")] {
        let mut mutant = assistant.clone();
        mutant["error"] = error;
        assert!(decode(mutant).is_err());
    }
    let tool_result = json!({"type":"user","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"u","content":"output"}]}});
    assert!(decode(tool_result.clone()).is_ok());
    for content in [
        Value::Null,
        json!(false),
        json!([{"type":"text","text":false}]),
        json!([{"type":"image","source":{}}]),
    ] {
        let mut mutant = tool_result.clone();
        mutant["message"]["content"][0]["content"] = content;
        assert!(decode(mutant).is_err());
    }
}

#[test]
fn cl4_finite_frame_controls_and_required_field_mutations() {
    let frames = [
        json!({"type":"system","subtype":"init","session_id":"n","cwd":"/owned","tools":["Bash"],"mcp_servers":[{"name":"mcp","status":"new-status-string"}]}),
        json!({"type":"system","subtype":"session_state_changed","state":"idle"}),
        json!({"type":"system","subtype":"task_started","task_id":"t","description":"d","task_type":"new-task-string"}),
        json!({"type":"system","subtype":"task_notification","task_id":"t","status":"completed","output_file":"","summary":"","usage":{"total_tokens":1,"tool_uses":0,"duration_ms":0}}),
        json!({"type":"system","subtype":"task_updated","task_id":"t","patch":{}}),
        json!({"type":"system","subtype":"api_retry","attempt":0,"max_retries":1,"retry_delay_ms":0,"error_status":null}),
        json!({"type":"assistant","session_id":"n","parent_tool_use_id":null,"message":{"role":"assistant","content":[{"type":"text","text":"t"},{"type":"thinking","thinking":"t","signature":"s"},{"type":"tool_use","id":"u","name":"Bash","input":{}}]}}),
        json!({"type":"user","message":{"role":"user","content":[{"type":"text","text":"t"},{"type":"tool_result","tool_use_id":"u","content":[{"type":"text","text":"t"}],"is_error":false}]}}),
        result(false),
        result(true),
        json!({"type":"control_request","request_id":"r","request":{"subtype":"can_use_tool","tool_use_id":"u","tool_name":"Bash","input":{}}}),
        json!({"type":"control_response","response":{"request_id":"r","subtype":"success","response":{}}}),
        json!({"type":"control_response","response":{"request_id":"r","subtype":"error","error":"unavailable"}}),
        json!({"type":"stream_event","event":{"type":"ping"}}),
        json!({"type":"tool_progress","tool_use_id":"u","tool_name":"Bash","parent_tool_use_id":null,"elapsed_time_seconds":0.5}),
        json!({"type":"tool_use_summary","summary":"","preceding_tool_use_ids":["u"]}),
        json!({"type":"auth_status","isAuthenticating":false,"output":["status"]}),
        json!({"type":"rate_limit_event","session_id":"n","rate_limit_info":{"status":"allowed","rateLimitType":"five_hour"}}),
    ];
    for control in frames {
        assert!(decode(control.clone()).is_ok(), "{control}");
        for (key, value) in control.as_object().unwrap() {
            // Optional nullable/opaque fields have their own controls above.
            // Here each top-level known required field gets a wrongtype mutant.
            if matches!(key.as_str(), "task_type" | "usage") {
                continue;
            }
            if !value.is_null() {
                let mut mutant = control.clone();
                mutant[key] = Value::Null;
                assert!(decode(mutant).is_err(), "{key}/{control}");
            }
        }
    }
}

#[test]
fn cl4_stream_nested_variants_require_their_payloads_and_reject_unknown_tags() {
    let events = [
        json!({"type":"message_start","message":{"id":"m","type":"message","role":"assistant","model":"model","content":[],"stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":0,"output_tokens":0}}}),
        json!({"type":"message_delta","delta":{"stop_reason":null,"stop_sequence":null},"usage":{"output_tokens":0}}),
        json!({"type":"content_block_start","index":0,"content_block":{"type":"text","text":"t"}}),
        json!({"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"t"}}),
        json!({"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"t"}}),
        json!({"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"{"}}),
        json!({"type":"content_block_delta","index":0,"delta":{"type":"signature_delta","signature":"s"}}),
        json!({"type":"content_block_stop","index":0}),
        json!({"type":"message_stop"}),
        json!({"type":"ping"}),
    ];
    for event in events {
        let frame = json!({"type":"stream_event","event":event});
        assert!(decode(frame.clone()).is_ok());
        let mut mutant = frame.clone();
        mutant["event"]["type"] = json!("future_event");
        assert!(decode(mutant).is_err());
        for key in event.as_object().unwrap().keys() {
            let mut mutant = frame.clone();
            mutant["event"].as_object_mut().unwrap().remove(key);
            assert!(decode(mutant).is_err());
        }
    }
    let unknown_block = json!({"type":"assistant","session_id":"n","parent_tool_use_id":null,"message":{"role":"assistant","content":[{"type":"image","source":{}}]}});
    assert!(decode(unknown_block).is_err());
    for subtype in [
        "compact_boundary",
        "informational",
        "permission_denied",
        "future_system",
    ] {
        assert!(decode(json!({"type":"system","subtype":subtype})).is_err());
    }
    for kind in ["future_frame", "", "result_extra"] {
        assert!(decode(json!({"type":kind})).is_err());
    }
}

#[test]
fn uuid_dedup_hashes_all_fields_and_preserves_null_array_and_content_changes() {
    let mut control = result(false);
    control["uuid"] = json!("12345678-1234-1234-1234-123456789ABC");
    control["extra"] = json!({"a":1,"b":[1,2]});
    let frame = decode(control.clone()).unwrap();
    assert_eq!(frame.raw()["uuid"], "12345678-1234-1234-1234-123456789abc");
    let mut state = initialized();
    assert!(
        state
            .observe(&frame, "n", Path::new("/owned"), false)
            .unwrap()
    );
    assert!(
        !state
            .observe(
                &decode(control.clone()).unwrap(),
                "n",
                Path::new("/owned"),
                false
            )
            .unwrap()
    );
    for (key, changed) in [
        ("extra", json!({"a":1,"b":[2,1]})),
        ("new", Value::Null),
        ("usage", json!({"input_tokens":0})),
        ("result", json!("changed")),
    ] {
        let mut mutant = control.clone();
        mutant[key] = changed;
        let frame = decode(mutant).unwrap();
        assert_eq!(
            state
                .observe(&frame, "n", Path::new("/owned"), false)
                .unwrap_err()
                .kind,
            ErrorKind::ParseFailure
        );
    }
    let reordered: Value = serde_json::from_str("{\"type\":\"result\",\"result\":\"answer\",\"is_error\":false,\"subtype\":\"success\",\"session_id\":\"n\",\"extra\":{\"b\":[1,2],\"a\":1},\"uuid\":\"12345678-1234-1234-1234-123456789abc\"}").unwrap();
    assert_eq!(decode(reordered).unwrap().digest(), frame.digest());
    control["uuid"] = json!("answer-A");
    assert!(decode(control).is_err());
    // Do not pretend reserializing an upstream Value can catch raw duplicates.
    assert!(strict_json::decode(b"{\"type\":\"result\",\"type\":\"result\"}", limits()).is_err());
}

#[test]
fn r1_selfcheck_and_strict_rate_vectors_do_not_certify_live_observation() {
    let checked = decoder_selfcheck().unwrap();
    for bucket in [
        "five_hour",
        "seven_day",
        "seven_day_opus",
        "seven_day_sonnet",
        "seven_day_overage_included",
    ] {
        for status in ["allowed", "allowed_warning", "rejected"] {
            let mut control = json!({"type":"rate_limit_event","session_id":"n","rate_limit_info":{"rateLimitType":bucket,"status":status}});
            let empty = ValidatedClaudeFrame::decode(control.clone(), &checked).unwrap();
            let observation = empty.rate().unwrap().observation(1);
            assert_eq!(observation.resets_at, None);
            assert_eq!(observation.used_percent, None);
            assert_eq!(observation.source_version, "claude/rate_limit_event");
            control["rate_limit_info"]["resetsAt"] = json!(123);
            control["rate_limit_info"]["utilization"] = json!(0.5);
            let usable = decode(control.clone())
                .unwrap()
                .rate()
                .unwrap()
                .observation(1);
            assert_eq!(
                (usable.resets_at, usable.used_percent),
                (Some(123000), Some(50.0))
            );
            for key in ["status", "rateLimitType"] {
                let mut mutant = control.clone();
                mutant["rate_limit_info"][key] = Value::Null;
                assert!(decode(mutant).is_err());
            }
            for (key, bad) in [
                ("resetsAt", json!("123")),
                ("resetsAt", json!(i64::MAX)),
                ("utilization", json!(false)),
                ("utilization", json!(1.01)),
                ("utilization", json!(-0.01)),
            ] {
                let mut mutant = control.clone();
                mutant["rate_limit_info"][key] = bad;
                assert!(decode(mutant).is_err());
            }
            control["rate_limit_info"]["resetsAt"] = Value::Null;
            control["rate_limit_info"]["utilization"] = Value::Null;
            assert!(decode(control).is_ok());
        }
    }
    let foreign = json!({"type":"rate_limit_event","session_id":"foreign","rate_limit_info":{"rateLimitType":"five_hour","status":"allowed"}});
    let mut state = initialized();
    assert!(
        !state
            .observe(
                &decode(foreign.clone()).unwrap(),
                "n",
                Path::new("/owned"),
                false
            )
            .unwrap()
    );
    let mut malformed_foreign = foreign;
    malformed_foreign["rate_limit_info"]["status"] = json!("future_status");
    assert!(decode(malformed_foreign).is_err());
    let nonrate =
        decode(json!({"type":"auth_status","isAuthenticating":false,"output":[]})).unwrap();
    assert!(nonrate.rate().is_none());
}
