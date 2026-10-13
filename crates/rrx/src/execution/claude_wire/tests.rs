use super::*;
use std::path::Path;

fn decode(value: Value) -> AdapterResult<ValidatedClaudeFrame> {
    ValidatedClaudeFrame::decode(value, &decoder_selfcheck()?)
}
fn observe(state: &mut RunState, value: Value, decision: bool) -> AdapterResult<bool> {
    state.observe(&decode(value)?, "n", Path::new("/owned"), decision)
}
fn init(state: &mut RunState) {
    observe(state, json!({"type":"system","subtype":"init","session_id":"n","cwd":"/owned","tools":[],"mcp_servers":[]}), false).unwrap();
}
fn success(text: &str) -> Value {
    json!({"type":"result","session_id":"n","subtype":"success","is_error":false,"result":text})
}
fn user(origin: Value) -> Value {
    json!({"type":"user","origin":origin,"parent_tool_use_id":null,"message":{"role":"user","content":"feedback"}})
}
fn assistant(parent: Value) -> Value {
    json!({"type":"assistant","session_id":"n","parent_tool_use_id":parent,"message":{"role":"assistant","content":[{"type":"text","text":"injected"}]}})
}
fn notification(task: &str, status: &str) -> Value {
    json!({"type":"system","subtype":"task_notification","task_id":task,"status":status,"output_file":"","summary":""})
}
fn metrics(fields: Value, resumed: bool) -> AdapterResult<Metrics> {
    let mut value = success("answer");
    value
        .as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    let frame = decode(value)?;
    Ok(Metrics::from_result(frame.result().unwrap(), resumed))
}

// The old pin stays until the actual S/P gates replace it. This test is not
// capability qualification, and must retire with that later implementation.
#[test]
fn version_gate_accepts_only_the_exact_pinned_release() {
    for ok in ["2.1.294 (Claude Code)", "2.1.294", "2.1.294\n"] {
        assert!(verify_version(ok).is_ok());
    }
    for refused in [
        "2.1.283 (Claude Code)",
        "2.1.293 (Claude Code)",
        "2.1.2940 (Claude Code)",
        "2.1.29",
        "v2.1.294",
        "",
    ] {
        assert!(verify_version(refused).is_err());
    }
}

#[test]
fn injected_failures_do_not_replace_human_proof_and_midturn_user_does_not_hang() {
    for malformed in [false, true] {
        let mut state = RunState::default();
        init(&mut state);
        observe(
            &mut state,
            user(json!({"kind":"channel","server":"hook-feedback"})),
            false,
        )
        .unwrap();
        observe(&mut state, success("human"), false).unwrap();
        assert!(
            state.complete(),
            "midturn feedback cannot strand owned input"
        );
        assert!(!observe(&mut state, assistant(json!("subagent")), false).unwrap());
        assert!(state.complete());
        observe(&mut state, user(json!({"kind":"task-notification"})), false).unwrap();
        assert!(!state.complete());
        let injected = if malformed {
            json!({"type":"result","session_id":"n","origin":{"kind":"task-notification"}})
        } else {
            json!({"type":"result","session_id":"n","subtype":"error_during_execution","is_error":true,"errors":[],"origin":{"kind":"task-notification"}})
        };
        if malformed {
            assert_eq!(
                observe(&mut state, injected, false).unwrap_err().kind,
                ErrorKind::ParseFailure
            );
            assert!(!state.complete());
        } else {
            assert!(!observe(&mut state, injected, false).unwrap());
            assert!(state.failed_tasks && state.complete());
        }
        assert_eq!(state.result.as_ref().unwrap().raw()["result"], "human");
        assert!(state.had_background);
    }
}

#[test]
fn injected_or_malformed_results_cannot_complete_the_owned_input() {
    let mut state = RunState::default();
    init(&mut state);
    for origin in [
        json!({"kind":"task-notification"}),
        json!({"kind":"channel","server":"mcp"}),
        json!({"kind":"peer","from":"peer-1"}),
    ] {
        let mut value = success("foreign");
        value["origin"] = origin;
        assert!(!observe(&mut state, value, true).unwrap());
        assert!(!state.complete());
    }
    for result in [
        json!({"type":"result","session_id":"n"}),
        json!({"type":"result","session_id":"n","subtype":"error_during_execution","is_error":false,"errors":[]}),
    ] {
        assert_eq!(
            observe(&mut state, result, true).unwrap_err().kind,
            ErrorKind::ParseFailure
        );
        assert!(!state.complete());
    }
    assert!(observe(&mut state, success("human"), true).unwrap());
    assert!(state.complete());
}

#[test]
fn completed_background_injection_preserves_human_result_and_requires_final_idle() {
    let mut state = RunState::default();
    init(&mut state);
    observe(&mut state, json!({"type":"system","subtype":"task_started","task_id":"t","description":"background","task_type":"local_agent"}), false).unwrap();
    observe(&mut state, success("human"), false).unwrap();
    observe(
        &mut state,
        json!({"type":"system","subtype":"session_state_changed","state":"idle"}),
        false,
    )
    .unwrap();
    assert!(!state.complete());
    observe(&mut state, notification("t", "completed"), false).unwrap();
    assert!(!state.complete());
    assert!(!observe(&mut state, assistant(Value::Null), false).unwrap());
    let mut injected = success("injected");
    injected["origin"] = json!({"kind":"task-notification"});
    assert!(!observe(&mut state, injected, false).unwrap());
    assert_eq!(state.result.as_ref().unwrap().raw()["result"], "human");
    observe(&mut state, notification("foreign-peer", "failed"), false).unwrap();
    assert!(!state.failed_tasks && !state.complete());
    observe(
        &mut state,
        json!({"type":"system","subtype":"session_state_changed","state":"idle"}),
        false,
    )
    .unwrap();
    assert!(state.complete());
}

#[test]
fn invocation_tokens_and_resumed_gauges_have_different_authority() {
    let a = metrics(json!({"usage":{"input_tokens":2,"output_tokens":13,"cache_creation_input_tokens":11924,"cache_read_input_tokens":531},"total_cost_usd":0.0957662,"duration_api_ms":1585}), false).unwrap();
    assert_eq!((a.cost(), a.api_ms()), (Some(0.0957662), Some(1585)));
    let b = metrics(json!({"usage":{"input_tokens":2,"output_tokens":12,"cache_creation_input_tokens":351,"cache_read_input_tokens":12228},"total_cost_usd":0.1012678,"duration_api_ms":3094}), true).unwrap();
    assert_eq!(
        (b.input(), b.output(), b.cache_write(), b.api_ms()),
        (Some(2), Some(12), Some(351), None)
    );
    assert_eq!(b.cost(), None);
    assert_eq!(b.cumulative_cost(), Some(0.1012678));
    assert_eq!(b.cumulative_api_ms(), Some(3094));
    // The old wrongtype-to-None expectation is intentionally replaced by
    // frame rejection. Legal absence/null/zero remain separately covered.
    assert!(metrics(json!({"usage":{"input_tokens":-1,"output_tokens":"13"},"total_cost_usd":-2,"duration_api_ms":"100"}), false).is_err());
    let reset = metrics(json!({"total_cost_usd":0,"duration_api_ms":1}), true).unwrap();
    assert!(reset.input().is_none() && reset.cost().is_none() && reset.api_ms().is_none());
    assert_eq!(
        (reset.cumulative_cost(), reset.cumulative_api_ms()),
        (Some(0.0), Some(1))
    );
}

#[test]
fn normal_control_responses_are_rejected_while_bootstrap_and_requests_keep_their_consumers() {
    let bootstrap = decode(json!({"type":"control_response","response":{"request_id":"rrx-initialize","subtype":"success","response":{}}})).unwrap();
    assert_eq!(
        control_result(&bootstrap, "rrx-initialize").unwrap(),
        json!({})
    );
    assert_eq!(
        control_result(&bootstrap, "other-request")
            .unwrap_err()
            .kind,
        ErrorKind::ParseFailure
    );
    for response in [
        bootstrap,
        decode(json!({"type":"control_response","response":{"request_id":"unsolicited","subtype":"success","response":{}}})).unwrap(),
        decode(json!({"type":"control_response","response":{"request_id":"unsolicited","subtype":"error","error":"fixture error"}})).unwrap(),
    ] {
        let mut state = RunState::default();
        init(&mut state);
        assert_eq!(
            state.observe(&response, "n", Path::new("/owned"), false).unwrap_err().kind,
            ErrorKind::ParseFailure,
            "normal loop has no outstanding control response correlation"
        );
        assert!(state.initialized && state.result.is_none() && !state.complete());
    }
    let request = decode(json!({"type":"control_request","request_id":"req","request":{"subtype":"can_use_tool","tool_use_id":"use","tool_name":"Bash","input":{"command":"pwd"}}})).unwrap();
    let mut state = RunState::default();
    init(&mut state);
    assert!(
        state
            .observe(&request, "n", Path::new("/owned"), false)
            .unwrap()
    );
    assert_eq!(Pending::parse(&request, "n").unwrap().request_id, "req");
    assert!(observe(&mut state, success("human"), false).unwrap());
    assert!(state.complete());
}

#[test]
fn exact_native_operation_reply_never_persists_grants() {
    let frame = decode(json!({"type":"control_request","request_id":"req","request":{"subtype":"can_use_tool","tool_use_id":"use","tool_name":"Bash","input":{"command":"pwd"}}})).unwrap();
    let p = Pending::parse(&frame, "native").unwrap();
    assert_eq!(
        p.reply(true)["response"]["response"]["updatedInput"],
        p.input
    );
    assert!(
        p.reply(true)["response"]["response"]
            .get("updatedPermissions")
            .is_none()
    );
    assert!(p.public("native").get("input").is_none());
    let frame = decode(json!({"type":"control_request","request_id":"req","request":{"subtype":"can_use_tool","tool_use_id":"different","tool_name":"Bash","input":p.input}})).unwrap();
    let q = Pending::parse(&frame, "native").unwrap();
    assert_ne!(p.hash, q.hash);
}

#[test]
fn background_agent_and_running_state_prevent_early_success() {
    let mut state = RunState::default();
    init(&mut state);
    observe(&mut state, json!({"type":"system","subtype":"task_started","task_id":"a","description":"background","task_type":"local_agent"}), false).unwrap();
    observe(&mut state, success("answer"), false).unwrap();
    assert!(!state.complete());
    observe(
        &mut state,
        json!({"type":"system","subtype":"session_state_changed","state":"running"}),
        false,
    )
    .unwrap();
    observe(&mut state, notification("a", "completed"), false).unwrap();
    assert!(!state.complete());
    observe(
        &mut state,
        json!({"type":"system","subtype":"session_state_changed","state":"idle"}),
        false,
    )
    .unwrap();
    assert!(state.complete());
}
