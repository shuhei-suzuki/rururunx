use super::*;
use crate::execution::{
    pipeline::tests_runner::tests::proposal,
    verification::{AdmittedProfile, hash},
};

fn admitted_body() -> String {
    serde_json::to_string(&AdmittedProfile::admit(proposal("/usr/bin/true".into())).unwrap())
        .unwrap()
}

/// Profile reads reject changed bodies even when the stored key still exists.
#[test]
fn b6_profile_hash() {
    let body = admitted_body();
    decode_tests_profile(&hash(body.as_bytes()), &body).unwrap();
    assert!(decode_tests_profile(&hash(body.as_bytes()), &(body + " ")).is_err());
}

/// A matching digest cannot authorize undecodable profile bytes.
#[test]
fn b6_profile_decode() {
    let body = "{\"proposal\":null,\"programs\":{}}";
    assert!(decode_tests_profile(&hash(body.as_bytes()), body).is_err());
}

/// A matching digest cannot authorize a decoded catalog with invalid policy.
#[test]
fn b6_profile_validate() {
    let mut body: Value = serde_json::from_str(&admitted_body()).unwrap();
    body["proposal"]["commands"][0]["timeout_seconds"] = json!(0);
    let body = serde_json::to_string(&body).unwrap();
    assert!(decode_tests_profile(&hash(body.as_bytes()), &body).is_err());
}

fn observed() -> PipelineStep {
    let mut step = super::b5::step();
    step.kind = "test".into();
    step.provider = None;
    step.body["exit"] = json!({"exit_code":0,"signal":null,"issue":null,"timed_out":false,"overflow":false,
        "stdout":{"bytes":0,"sha256":hash(b""),"complete":true},
        "stderr":{"bytes":0,"sha256":hash(b""),"complete":true}});
    step
}

/// Only zero exit without a timeout or overflow classifies as passing.
#[test]
fn b6_result_pass_fail() {
    let step = observed();
    assert!(test_pass(&step).unwrap());
    for (key, value) in [
        ("exit_code", json!(1)),
        ("timed_out", json!(true)),
        ("overflow", json!(true)),
    ] {
        let mut failed = step.clone();
        failed.body["exit"][key] = value;
        assert!(!test_pass(&failed).unwrap());
    }
    let mut signalled = step;
    signalled.body["exit"]["exit_code"] = Value::Null;
    signalled.body["exit"]["signal"] = json!(9);
    assert!(!test_pass(&signalled).unwrap());
}

/// Missing exit, stream, death or I/O observations cannot certify a test result.
#[test]
fn b6_result_missing() {
    for pointer in [
        "/death_confirmed",
        "/exit/exit_code",
        "/exit/stdout/complete",
        "/exit/stderr/complete",
        "/exit/issue",
    ] {
        let mut step = observed();
        *step.body.pointer_mut(pointer).unwrap() = if pointer.ends_with("issue") {
            json!("output_io")
        } else if pointer.ends_with("exit_code") {
            Value::Null
        } else {
            json!(false)
        };
        assert!(test_pass(&step).is_err(), "{pointer}");
    }
}

#[cfg(target_os = "macos")]
mod host;
