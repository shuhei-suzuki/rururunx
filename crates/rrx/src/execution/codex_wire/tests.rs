use super::*;
use serde_json::json;
fn turn() -> Value {
    json!({"id":"u","items":[],"status":"completed"})
}
fn thread() -> Value {
    json!({"id":"t","cwd":"/own","modelProvider":"openai","status":{"type":"idle"},"turns":[]})
}
fn settings() -> Value {
    json!({"approvalPolicy":"never","approvalsReviewer":"user","collaborationMode":{"mode":"default","settings":{"model":"m"}},"cwd":"/own","model":"m","modelProvider":"openai","sandboxPolicy":{"type":"workspaceWrite"}})
}
fn init() -> Value {
    json!({"codexHome":"/home","platformFamily":"unix","platformOs":if cfg!(target_os="macos") {"macos"}else{"linux"},"userAgent":"fixture"})
}
/// CX-3/4 all eight outgoing methods and all eight response decoders share the finite contract.
#[test]
fn u4_methods_and_response_types() {
    let c = CodexWireContract::default();
    for (method, params, response) in [
        (
            "initialize",
            json!({"clientInfo":{"name":"rrx","version":"test"},"capabilities":{"experimentalApi":true}}),
            init(),
        ),
        (
            "account/read",
            json!({"refreshToken":false}),
            json!({"requiresOpenaiAuth":true,"account":{"type":"chatgpt","email":null,"planType":"pro"}}),
        ),
        (
            "environment/status",
            json!({"environmentId":"local"}),
            json!({"status":"ready"}),
        ),
        (
            "account/rateLimits/read",
            json!({}),
            json!({"rateLimits":{}}),
        ),
        (
            "config/read",
            json!({"cwd":"/own","includeLayers":false}),
            json!({"config":{},"origins":{}}),
        ),
        (
            "mcpServerStatus/list",
            json!({"threadId":"t","limit":100,"cursor":null}),
            json!({"data":[]}),
        ),
        (
            "thread/start",
            json!({"cwd":"/own","approvalPolicy":"never","sandbox":"workspace-write"}),
            json!({"thread":thread(),"cwd":"/own","modelProvider":"openai","model":"m","approvalPolicy":"never","approvalsReviewer":"user","sandbox":{"type":"workspaceWrite"}}),
        ),
        (
            "turn/start",
            json!({"threadId":"t","input":[{"type":"text","text":"bounded"}]}),
            json!({"turn":turn()}),
        ),
    ] {
        let mut request = json!({"id":1,"method":method,"params":params});
        c.sent(&request).unwrap();
        c.response(method, &response).unwrap();
        request["params"]["undeclared"] = json!(true);
        assert!(c.sent(&request).is_err(), "{method}");
        assert!(c.response(method, &Value::Null).is_err(), "{method}");
    }
    c.sent(&json!({"method":"initialized"})).unwrap();
    assert!(
        c.sent(&json!({"method":"initialized","params":{}}))
            .is_err()
    );
}
/// Rate responses and notifications require a non-null snapshot, even when an optional bucket map exists.
#[test]
fn u4_rate_snapshot_required_at_both_inbound_boundaries() {
    let c = CodexWireContract::default();
    for value in [
        json!({}),
        json!({"ordinaryUsageAllowed":true}),
        json!({"rateLimitsByLimitId":{"x":{"primary":{"usedPercent":20}}}}),
        json!({"rateLimits":null}),
    ] {
        assert!(
            c.response("account/rateLimits/read", &value).is_err(),
            "{value}"
        );
        assert!(
            c.frame(json!({"method":"account/rateLimits/updated","params":value}))
                .is_err(),
            "missing or null snapshot notification accepted"
        );
    }
    let empty = json!({"rateLimits":{}});
    c.response("account/rateLimits/read", &empty).unwrap();
    c.frame(json!({"method":"account/rateLimits/updated","params":empty}))
        .unwrap();
}
/// CX-4 key-tags, nested domains, refinements and four RPC forms are checked before consumers.
#[test]
fn u4_nested_types_and_rpc_forms() {
    let c = CodexWireContract::default();
    for value in [
        json!({"method":"skills/changed","params":{}}),
        json!({"id":"r","method":"currentTime/read","params":{"threadId":"t"}}),
        json!({"id":1,"result":null}),
        json!({"id":1,"error":{"code":-32601,"message":"unavailable"}}),
    ] {
        c.decode(&serde_json::to_vec(&value).unwrap()).unwrap();
    }
    for value in [
        json!({"method":"skills/changed"}),
        json!({"method":"skills/changed","params":{},"result":{}}),
        json!({"id":null,"result":{}}),
        json!({"id":1,"result":{},"error":{"code":1,"message":"x"}}),
        json!({"id":1,"result":{},"params":{}}),
    ] {
        assert!(c.frame(value).is_err());
    }
    assert!(
        c.decode(br#"{"method":"skills/changed","params":{"duplicate":1,"duplicate":2}}"#)
            .is_err()
    );
    for value in [
        json!({"httpConnectionFailed":{"httpStatusCode":65535}}),
        json!({"activeTurnNotSteerable":{"turnKind":"compact"}}),
    ] {
        error_kind(&value).unwrap();
    }
    for value in [
        json!({"httpConnectionFailed":{"httpStatusCode":65536}}),
        json!({"httpConnectionFailed":false}),
        json!({"activeTurnNotSteerable":{"turnKind":"future"}}),
        json!({"httpConnectionFailed":{},"activeTurnNotSteerable":{"turnKind":"review"}}),
    ] {
        assert!(error_kind(&value).is_err());
    }
    assert_eq!(
        error_kind(&json!({})).unwrap(),
        CodexErrorKind::Unclassified
    );
    assert_eq!(
        error_kind(&json!("future")).unwrap(),
        CodexErrorKind::Unclassified
    );
    for value in [
        json!({}),
        json!({"granular":{"rules":true,"sandbox_approval":false,"mcp_elicitations":false},"extra":true}),
    ] {
        assert!(check("AskForApproval", &value, &c).is_err());
    }
    for value in [
        json!({"features":{"apps":"false"}}),
        json!({"mcp_servers":{"server":{"enabled":null}}}),
        json!({"web_search":"future"}),
    ] {
        assert!(check("Config", &value, &c).is_err());
    }
    assert!(
        c.response(
            "config/read",
            &json!({"config":{},"origins":{},"layers":[]})
        )
        .is_err()
    );
    assert!(
        check(
            "McpElicitationSchema",
            &json!({"type":"object","properties":{},"extra":true}),
            &c
        )
        .is_err()
    );
}
/// CX-5/6 all basis notification ownership classes are validated, including malformed foreign frames.
#[test]
fn u4_notifications_and_ownership() {
    let c = CodexWireContract::default();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/codex-contract/basis-pair.json"
    ))
    .unwrap();
    let methods = fixture["files"]["ServerNotification"]["oneOf"]
        .as_array()
        .unwrap();
    assert_eq!(methods.len(), 84);
    for method in methods {
        let method = method["properties"]["method"]["enum"][0].as_str().unwrap();
        let owner = notification_owner(method).unwrap();
        let mut p = match owner {
            Ownership::Global | Ownership::OptionalThread => json!({}),
            Ownership::Thread => json!({"threadId":"t"}),
            Ownership::ThreadTurn => json!({"threadId":"t","turnId":"u"}),
            Ownership::ThreadOptionalTurn => json!({"threadId":"t","turnId":null}),
            Ownership::NestedThread => json!({"thread":thread()}),
            Ownership::NestedTurn => json!({"threadId":"t","turn":turn()}),
            Ownership::Added => unreachable!(),
        };
        let extra = match method {
            "error" => json!({"error":{"message":"bounded"},"willRetry":false}),
            "thread/status/changed" => json!({"status":{"type":"active","activeFlags":[]}}),
            "thread/settings/updated" => json!({"threadSettings":settings()}),
            "item/started" | "item/completed" => {
                json!({"item":{"id":"i","type":"reasoning"},"startedAtMs":0,"completedAtMs":0})
            }
            "item/agentMessage/delta" => json!({"itemId":"i","delta":"bounded"}),
            "item/commandExecution/terminalInteraction" => {
                json!({"itemId":"i","processId":"p","stdin":""})
            }
            "item/fileChange/patchUpdated" => json!({"itemId":"i","changes":[]}),
            "serverRequest/resolved" => json!({"requestId":1}),
            "mcpServer/startupStatus/updated" => json!({"name":"s","status":"ready"}),
            "account/rateLimits/updated" => json!({"rateLimits":{}}),
            "account/login/completed" => json!({"success":true}),
            _ => json!({}),
        };
        p.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let v = json!({"method":method,"params":p});
        let frame = c.frame(v.clone()).unwrap();
        assert!(!frame.foreign("t", "u"), "{method}");
        if owner == Ownership::Global {
            let mut extended = v.clone();
            extended["params"]["threadId"] = json!({"tolerated":"foreign"});
            extended["params"]["turnId"] = json!("foreign");
            let extended = c.frame(extended).unwrap();
            assert!(!extended.foreign("t", "u"));
            extended.handshake_notification().unwrap();
        }
        let handshake = matches!(owner, Ownership::Global | Ownership::OptionalThread);
        assert_eq!(
            frame.handshake_notification().is_ok(),
            handshake,
            "{method}"
        );
        if !matches!(owner, Ownership::Global) {
            let mut invalid = v.clone();
            if owner == Ownership::NestedThread {
                invalid["params"]["thread"]["id"] = json!(9);
            } else {
                invalid["params"]["threadId"] = json!(9);
            }
            assert!(c.frame(invalid).is_err(), "malformed foreign {method}");
        }
    }
}
/// CX-4 all eleven server request payload closures are checked before handshake refusal.
#[test]
fn u4_server_requests() {
    let c = CodexWireContract::default();
    for (m, p) in [
        (
            "item/commandExecution/requestApproval",
            json!({"itemId":"i","startedAtMs":0,"threadId":"t","turnId":"u"}),
        ),
        (
            "item/fileChange/requestApproval",
            json!({"itemId":"i","startedAtMs":0,"threadId":"t","turnId":"u"}),
        ),
        (
            "item/tool/requestUserInput",
            json!({"isBlocking":true,"itemId":"i","questions":[],"threadId":"t","turnId":"u"}),
        ),
        (
            "mcpServer/elicitation/request",
            json!({"serverName":"s","threadId":"t","mode":"form","message":"x","requestedSchema":{"type":"object","properties":{"choice":{"type":"string","enum":["a"]}}}}),
        ),
        (
            "item/permissions/requestApproval",
            json!({"cwd":"/own","itemId":"i","permissions":{},"startedAtMs":0,"threadId":"t","turnId":"u"}),
        ),
        (
            "item/tool/call",
            json!({"arguments":null,"callId":"c","threadId":"t","tool":"x","turnId":"u"}),
        ),
        (
            "account/chatgptAuthTokens/refresh",
            json!({"reason":"unauthorized"}),
        ),
        ("attestation/generate", json!({})),
        ("currentTime/read", json!({"threadId":"t"})),
        (
            "applyPatchApproval",
            json!({"callId":"c","conversationId":"t","fileChanges":{}}),
        ),
        (
            "execCommandApproval",
            json!({"callId":"c","command":[],"conversationId":"t","cwd":"/own","parsedCmd":[]}),
        ),
    ] {
        if m != "attestation/generate" {
            assert!(
                c.frame(json!({"id":"r","method":m,"params":{}})).is_err(),
                "missing required request payload {m}"
            );
        }
        let f = c.frame(json!({"id":"r","method":m,"params":p})).unwrap();
        assert!(f.handshake_notification().is_err(), "{m}");
        assert!(
            c.frame(json!({"id":"r","method":m,"params":[]})).is_err(),
            "{m}"
        );
    }
}
/// Only the seven approved owned item kinds can contribute to a successful M0 turn.
#[test]
fn u4_item_scope() {
    let c = CodexWireContract::default();
    for kind in ITEM_TYPES {
        let mut v = json!({"id":"i","type":kind});
        match *kind {
            "agentMessage" => v["text"] = json!("bounded"),
            "commandExecution" => {
                v["command"] = json!("fixture");
                v["commandActions"] = json!([]);
                v["cwd"] = json!("/own");
                v["status"] = json!("completed");
            }
            "fileChange" => {
                v["changes"] = json!([]);
                v["status"] = json!("completed");
            }
            _ => {
                v["tolerated"] = json!({"codexErrorInfo":"usageLimitExceeded","status":"completed"})
            }
        }
        let expected = [
            "userMessage",
            "agentMessage",
            "reasoning",
            "plan",
            "commandExecution",
            "fileChange",
            "contextCompaction",
        ]
        .contains(kind);
        assert_eq!(
            Item::decode(&v, &c).unwrap().permitted(),
            expected,
            "{kind}"
        );
        let t = json!({"id":"u","status":"completed","items":[v]});
        assert_eq!(Turn::decode(&t, &c).unwrap().permitted(), expected);
    }
    assert!(Item::decode(&json!({"id":"i","type":"future"}), &c).is_err());
}
/// Config Opaque/tolerated values cannot affect the shared restrictive judgement predicate.
#[test]
fn u4_config_judgement_reads_only_strict_paths() {
    let mut config = json!({"web_search":"disabled","features":{"apps":false},"mcp_servers":{"server":{"enabled":false}}});
    for value in [
        json!({"filesystem":"opaque","network":{"enabled":true}}),
        json!(false),
        json!(null),
    ] {
        config["permissions"] = value.clone();
        config["approval_policy"] = value;
        assert!(
            ConfigJudgement::decode(&config)
                .unwrap()
                .restricted(["apps"].into_iter(), ["server"].into_iter())
        );
    }
    config["features"]["apps"] = json!(true);
    assert!(
        !ConfigJudgement::decode(&config)
            .unwrap()
            .restricted(["apps"].into_iter(), ["server"].into_iter())
    );
    config["features"]["apps"] = json!(false);
    config["mcp_servers"]["server"]["enabled"] = json!(true);
    assert!(
        !ConfigJudgement::decode(&config)
            .unwrap()
            .restricted(["apps"].into_iter(), ["server"].into_iter())
    );
    config["mcp_servers"]["server"]["enabled"] = json!(false);
    config["web_search"] = json!("live");
    assert!(
        !ConfigJudgement::decode(&config)
            .unwrap()
            .restricted(["apps"].into_iter(), ["server"].into_iter())
    );
}
/// CX-6 handshake requires every correlated successful RPC, ChatGPT, ready and valid rate, with strict tail handling.
#[test]
fn u4_handshake_correlation_and_rate() {
    for scenario in [
        "empty_valid",
        "usable",
        "id",
        "api",
        "pending",
        "disconnected",
        "unknown",
        "environment-malformed",
        "rate-absent",
        "rate-null",
        "rate-map-only",
        "rate",
        "error",
        "missing",
        "tail",
        "contradictory",
    ] {
        let mut h = Handshake::new(CodexWireContract::default());
        h.initial().unwrap();
        let responses = [
            init(),
            json!({"requiresOpenaiAuth":true,"account":{"type":if scenario=="api" {"apiKey"}else{"chatgpt"},"email":null,"planType":"pro"}}),
            json!({"status":match scenario {
                "pending" | "disconnected" | "unknown" => scenario,
                "environment-malformed" => "unsupported",
                _ => "ready",
            }}),
            if scenario == "usable" {
                json!({"rateLimits":{"primary":{"usedPercent":20}}})
            } else if scenario == "rate" {
                json!({"rateLimits":{},"rateLimitsByLimitId":{"unselected":{"primary":{"usedPercent":"20"}}}})
            } else if scenario == "rate-absent" {
                json!({})
            } else if scenario == "rate-null" {
                json!({"rateLimits":null})
            } else if scenario == "rate-map-only" {
                json!({"rateLimitsByLimitId":{"x":{"primary":{"usedPercent":20}}}})
            } else {
                json!({"rateLimits":{}})
            },
        ];
        assert_eq!(
            CodexWireContract::default()
                .response("environment/status", &responses[2])
                .is_ok(),
            scenario != "environment-malformed",
            "{scenario}"
        );
        let mut valid = true;
        for (i, r) in responses.into_iter().enumerate() {
            if scenario == "missing" && i == 3 {
                break;
            }
            let v = if scenario == "error" {
                json!({"id":i+1,"error":{"code":-32601,"message":"unsupported"}})
            } else {
                json!({"id":if scenario=="id" {99}else{i+1},"result":r})
            };
            match h.observe(&serde_json::to_vec(&v).unwrap()) {
                Ok(sent) => {
                    if i == 0 {
                        assert_eq!(sent[0], json!({"method":"initialized"}));
                    }
                }
                Err(_) => {
                    valid = false;
                    break;
                }
            }
        }
        if scenario == "tail" {
            valid &= h.observe(b"{").is_ok();
        }
        if scenario == "contradictory" {
            valid &= h
                .observe(br#"{"method":"account/updated","params":{"authMode":null}}"#)
                .is_ok();
        }
        assert_eq!(
            valid && h.complete(),
            matches!(scenario, "empty_valid" | "usable"),
            "{scenario}"
        );
        if matches!(scenario, "empty_valid" | "usable") {
            assert_eq!(h.rate, Some(scenario == "usable"));
        } else if matches!(
            scenario,
            "pending"
                | "disconnected"
                | "unknown"
                | "environment-malformed"
                | "rate-absent"
                | "rate-null"
                | "rate-map-only"
                | "rate"
        ) {
            assert_eq!(h.rate, None, "handshake refusal observed quota: {scenario}");
        }
    }
}
