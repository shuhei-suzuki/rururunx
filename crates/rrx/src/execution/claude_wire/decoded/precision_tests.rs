use super::*;
use crate::execution::claude_wire::RunState;
use std::path::Path;

fn bytes(text: &str) -> AdapterResult<ValidatedClaudeFrame> {
    ValidatedClaudeFrame::decode_bytes(text.as_bytes(), &decoder_selfcheck()?)
}
fn initialized() -> RunState {
    let mut state = RunState::default();
    let frame = bytes(r#"{"type":"system","subtype":"init","session_id":"n","cwd":"/owned","tools":[],"mcp_servers":[]}"#).unwrap();
    state
        .observe(&frame, "n", Path::new("/owned"), false)
        .unwrap();
    state
}
fn terminal(number: &str) -> String {
    format!(
        r#"{{"type":"result","session_id":"n","subtype":"error_during_execution","is_error":true,"errors":[],"api_error_status":{number}}}"#
    )
}

#[test]
fn checked_api_status_presence_and_equivalent_numeric_consumers() {
    let absent = bytes(r#"{"type":"result","session_id":"n","subtype":"error_during_execution","is_error":true,"errors":[]}"#).unwrap();
    assert_eq!(
        absent.result().unwrap().api_error_status(),
        Presence::Absent
    );
    let null = bytes(&terminal("null")).unwrap();
    assert_eq!(null.result().unwrap().api_error_status(), Presence::Null);
    for (forms, status, kind, capacity) in [
        (
            ["401", "401.0", "4.01e2"],
            401,
            ErrorKind::AuthenticationUnavailable,
            false,
        ),
        (
            ["429", "429.0", "4.29e2"],
            429,
            ErrorKind::ProcessFailure,
            true,
        ),
        (
            ["503", "503.0", "5.03e2"],
            503,
            ErrorKind::ProcessFailure,
            true,
        ),
        (
            ["200", "200.0", "2e2"],
            200,
            ErrorKind::ProcessFailure,
            false,
        ),
        (["0", "0.0", "0e2"], 0, ErrorKind::ProcessFailure, false),
    ] {
        for form in forms {
            let frame = bytes(&terminal(form)).unwrap();
            assert_eq!(
                frame.result().unwrap().api_error_status(),
                Presence::Value(status)
            );
            let mut state = initialized();
            assert_eq!(
                state
                    .observe(&frame, "n", Path::new("/owned"), false)
                    .unwrap_err()
                    .kind,
                kind
            );
            assert_eq!(state.terminal_capacity, capacity);
            assert!(state.result.is_none() && !state.complete());
        }
    }
    {
        let form = "null";
        let mut state = initialized();
        assert_eq!(
            state
                .observe(
                    &bytes(&terminal(form)).unwrap(),
                    "n",
                    Path::new("/owned"),
                    false
                )
                .unwrap_err()
                .kind,
            ErrorKind::ProcessFailure
        );
        assert!(!state.terminal_capacity);
    }
    for bad in ["401.5", "-1", "65536", "9007199254740992.5", "1e-400"] {
        assert!(bytes(&terminal(bad)).is_err());
    }
}

fn counter_frames(n: &str) -> Vec<String> {
    vec![
        format!(
            r#"{{"type":"result","session_id":"n","subtype":"success","is_error":false,"result":"answer","usage":{{"input_tokens":{n},"output_tokens":{n},"cache_read_input_tokens":{n},"cache_creation_input_tokens":{n}}},"duration_api_ms":{n}}}"#
        ),
        format!(
            r#"{{"type":"result","session_id":"n","subtype":"success","is_error":false,"result":"answer","origin":{{"kind":"peer","from":"p","verifiedPeerPid":{n}}}}}"#
        ),
        format!(
            r#"{{"type":"system","subtype":"api_retry","attempt":{n},"max_retries":{n},"retry_delay_ms":{n},"error_status":null}}"#
        ),
        format!(
            r#"{{"type":"stream_event","event":{{"type":"content_block_start","index":{n},"content_block":{{"type":"text","text":"text"}}}}}}"#
        ),
        format!(
            r#"{{"type":"stream_event","event":{{"type":"content_block_delta","index":{n},"delta":{{"type":"text_delta","text":"text"}}}}}}"#
        ),
        format!(r#"{{"type":"stream_event","event":{{"type":"content_block_stop","index":{n}}}}}"#),
        format!(
            r#"{{"type":"stream_event","event":{{"type":"message_start","message":{{"id":"m","type":"message","role":"assistant","model":"model","content":[],"stop_reason":null,"stop_sequence":null,"usage":{{"input_tokens":{n},"output_tokens":{n},"cache_read_input_tokens":{n},"cache_creation_input_tokens":{n}}}}}}}}}"#
        ),
        format!(
            r#"{{"type":"stream_event","event":{{"type":"message_delta","delta":{{"stop_reason":null,"stop_sequence":null}},"usage":{{"output_tokens":{n},"cache_read_input_tokens":{n},"cache_creation_input_tokens":{n}}}}}}}"#
        ),
    ]
}

#[test]
fn every_uint_consumer_uses_original_decimal_semantics() {
    for n in [
        "0",
        "-0.0",
        "1.0",
        "1e0",
        "9007199254740993.0",
        "18446744073709551615",
        "18446744073709551615.0",
        "184467440737095516150e-1",
        "1.8446744073709551615e19",
    ] {
        for text in counter_frames(n) {
            assert!(bytes(&text).is_ok(), "normal uint form {n}");
        }
    }
    for n in [
        "-1",
        "0.5",
        "9007199254740992.5",
        "18446744073709551615.5",
        "18446744073709551616",
        "18446744073709551616.0",
        "1e-400",
    ] {
        for text in counter_frames(n) {
            assert!(bytes(&text).is_err(), "invalid original uint form {n}");
        }
    }
    let text = &counter_frames("18446744073709551615.0")[0];
    let frame = bytes(text).unwrap();
    let telemetry = frame.result().unwrap().telemetry();
    assert_eq!(telemetry.input, Presence::Value(u64::MAX));
    assert_eq!(telemetry.output, Presence::Value(u64::MAX));
    assert_eq!(telemetry.cache_read, Presence::Value(u64::MAX));
    assert_eq!(telemetry.cache_write, Presence::Value(u64::MAX));
    assert_eq!(telemetry.api_ms, Presence::Value(u64::MAX));
    for n in ["401.0", "4.01e2", "65535.0"] {
        let text = format!(
            r#"{{"type":"system","subtype":"api_retry","attempt":0,"max_retries":1,"retry_delay_ms":0,"error_status":{n}}}"#
        );
        assert!(bytes(&text).is_ok());
    }
    for n in ["65536.0", "9007199254740992.5"] {
        let text = format!(
            r#"{{"type":"system","subtype":"api_retry","attempt":0,"max_retries":1,"retry_delay_ms":0,"error_status":{n}}}"#
        );
        assert!(bytes(&text).is_err());
    }
    // The negative producer is original bytes, not json!(an already-rounded f64).
    let foreign = counter_frames("9007199254740992.5")[0]
        .replace("\"session_id\":\"n\"", "\"session_id\":\"foreign\"");
    assert!(bytes(&foreign).is_err());
}

fn uuid_result(n: &str, upper: bool, reordered: bool) -> String {
    let uuid = if upper {
        "12345678-1234-1234-1234-123456789ABC"
    } else {
        "12345678-1234-1234-1234-123456789abc"
    };
    if reordered {
        format!(
            r#"{{"extra":{{"b":[null,2],"a":{n}}},"uuid":"{uuid}","result":"answer","is_error":false,"subtype":"success","session_id":"n","type":"result"}}"#
        )
    } else {
        format!(
            r#"{{"type":"result","session_id":"n","subtype":"success","is_error":false,"result":"answer","uuid":"{uuid}","extra":{{"a":{n},"b":[null,2]}}}}"#
        )
    }
}

#[test]
fn uuid_content_uses_original_unknown_numbers_and_semantic_controls() {
    for (a, b) in [
        ("9007199254740992.0", "9007199254740993.0"),
        ("9007199254740992.1", "9007199254740992.2"),
        ("1e-400", "2e-400"),
    ] {
        let first = bytes(&uuid_result(a, true, false)).unwrap();
        let changed = bytes(&uuid_result(b, false, true)).unwrap();
        assert_ne!(first.digest(), changed.digest());
        let mut state = initialized();
        assert!(
            state
                .observe(&first, "n", Path::new("/owned"), false)
                .unwrap()
        );
        assert_eq!(
            state
                .observe(&changed, "n", Path::new("/owned"), false)
                .unwrap_err()
                .kind,
            ErrorKind::ParseFailure
        );
    }
    for (a, b) in [
        ("1.0", "1e0"),
        ("9007199254740993.0", "90071992547409930e-1"),
    ] {
        let first = bytes(&uuid_result(a, true, false)).unwrap();
        let same = bytes(&uuid_result(b, false, true)).unwrap();
        assert_eq!(first.digest(), same.digest());
        let mut state = initialized();
        assert!(
            state
                .observe(&first, "n", Path::new("/owned"), false)
                .unwrap()
        );
        assert!(
            !state
                .observe(&same, "n", Path::new("/owned"), false)
                .unwrap()
        );
    }
    assert_ne!(
        bytes(&uuid_result("1", false, false)).unwrap().digest(),
        bytes(&uuid_result("1.0", false, false)).unwrap().digest()
    );
    assert_ne!(
        bytes(&uuid_result("0.0", false, false)).unwrap().digest(),
        bytes(&uuid_result("-0.0", false, false)).unwrap().digest()
    );
    assert!(
        bytes(r#"{"type":"auth_status","isAuthenticating":false,"output":[],"extra":1e400}"#)
            .is_err()
    );
    assert!(
        bytes(
            r#"{"type":"auth_status","type":"auth_status","isAuthenticating":false,"output":[]}"#
        )
        .is_err()
    );
}

#[test]
fn bootstrap_ownership_history_and_normal_response_refusal_share_the_actual_state() {
    let uuid = "12345678-1234-1234-1234-123456789abc";
    for session in ["", ",\"session_id\":\"n\""] {
        let text = format!(
            r#"{{"type":"control_response","uuid":"{uuid}"{session},"response":{{"request_id":"rrx-initialize","subtype":"success","response":{{}}}}}}"#
        );
        let response = bytes(&text).unwrap();
        let mut state = RunState::default();
        assert_eq!(
            state
                .initialize_response(&response, "n", "rrx-initialize")
                .unwrap(),
            json!({})
        );
        assert!(!state.initialized && state.result.is_none());
        // An exact bootstrap UUID redelivery still cannot bypass O03.
        assert_eq!(
            state
                .observe(&response, "n", Path::new("/owned"), false)
                .unwrap_err()
                .kind,
            ErrorKind::ParseFailure
        );
        let changed_init = bytes(&format!(r#"{{"type":"system","subtype":"init","uuid":"{uuid}","session_id":"n","cwd":"/owned","tools":[],"mcp_servers":[]}}"#)).unwrap();
        assert_eq!(
            state
                .observe(&changed_init, "n", Path::new("/owned"), false)
                .unwrap_err()
                .kind,
            ErrorKind::ParseFailure
        );
        assert!(!state.initialized && state.result.is_none());
        let init = bytes(r#"{"type":"system","subtype":"init","session_id":"n","cwd":"/owned","tools":[],"mcp_servers":[]}"#).unwrap();
        state
            .observe(&init, "n", Path::new("/owned"), false)
            .unwrap();
        let changed_result = bytes(&uuid_result("1.0", false, false)).unwrap();
        assert_eq!(
            state
                .observe(&changed_result, "n", Path::new("/owned"), false)
                .unwrap_err()
                .kind,
            ErrorKind::ParseFailure
        );
        assert!(state.result.is_none() && !state.complete());
    }
    let foreign = bytes(r#"{"type":"control_response","session_id":"foreign","response":{"request_id":"rrx-initialize","subtype":"success","response":{}}}"#).unwrap();
    assert_eq!(
        RunState::default()
            .initialize_response(&foreign, "n", "rrx-initialize")
            .unwrap_err()
            .kind,
        ErrorKind::OwnershipMismatch
    );
    let wrong = bytes(r#"{"type":"control_response","response":{"request_id":"wrong","subtype":"success","response":{}}}"#).unwrap();
    assert_eq!(
        RunState::default()
            .initialize_response(&wrong, "n", "rrx-initialize")
            .unwrap_err()
            .kind,
        ErrorKind::ParseFailure
    );
}
