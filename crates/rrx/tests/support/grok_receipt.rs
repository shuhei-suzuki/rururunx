//! Closed test-only projection of an exact scoped attempt; never runtime authority.
use super::*;

#[derive(Clone, Copy)]
pub(super) struct Attempt {
    pub lower: i64,
    pub input_version: u64,
}
#[derive(Clone)]
pub(super) struct Observation {
    pub receipt: Result<Value, &'static str>,
    pub events: Vec<Value>,
    pub saved: Option<Session>,
    pub attempt: Option<Attempt>,
}
fn events_through(
    store: &Store,
    scope: &Scope,
    after: i64,
    upper: i64,
) -> Result<Vec<Value>, &'static str> {
    let mut after = after;
    let mut result = vec![];
    for _ in 0..1000 {
        let page = store
            .events(scope, after, 100)
            .map_err(|_| "audit_unavailable")?;
        let size = page.len();
        for event in page {
            if event.sequence <= after {
                return Err("audit_nonprogress");
            }
            after = event.sequence;
            if after > upper {
                return Ok(result);
            }
            if event.scope == *scope {
                result.push(json!({"sequence":event.sequence,"kind":event.kind,"data":event.data}));
            }
        }
        if size < 100 {
            return Ok(result);
        }
    }
    Err("audit_limit_exceeded")
}
pub(super) fn watermark(store: &SharedStore, scope: &Scope) -> Result<i64, &'static str> {
    let store = store.lock().map_err(|_| "state_unavailable")?;
    let events = events_through(&store, scope, 0, i64::MAX)?;
    Ok(events
        .last()
        .and_then(|event| event["sequence"].as_i64())
        .unwrap_or(0))
}
pub(super) fn observe(
    store: &SharedStore,
    session: &Session,
    attempt: Option<Attempt>,
) -> Observation {
    let unavailable = |reason| Observation {
        receipt: Err(reason),
        events: vec![],
        saved: None,
        attempt,
    };
    let upper = match watermark(store, &session.scope) {
        Ok(upper) => upper, // Before any next launch/resume.
        Err(reason) => return unavailable(reason),
    };
    let guard = match store.lock() {
        Ok(guard) => guard,
        Err(_) => return unavailable("state_unavailable"),
    };
    let saved = guard
        .session(session.id)
        .ok()
        .flatten()
        .map(|(session, _)| session);
    let events = attempt
        .map(|attempt| events_through(&guard, &session.scope, attempt.lower, upper))
        .transpose();
    let receipt = match (&events, attempt) {
        (Ok(Some(events)), Some(attempt)) => {
            exact_receipt(events, session, saved.as_ref(), attempt)
        }
        (Err(reason), _) => Err(*reason),
        _ => Err("attempt_unavailable"),
    };
    Observation {
        receipt,
        events: events.ok().flatten().unwrap_or_default(),
        saved,
        attempt,
    }
}
fn exact_receipt(
    events: &[Value],
    session: &Session,
    saved: Option<&Session>,
    attempt: Attempt,
) -> Result<Value, &'static str> {
    let turns: Vec<_> = events
        .iter()
        .filter(|event| {
            event["kind"] == "grok.turn_observed" && event["data"]["session"] == json!(session.id)
        })
        .collect();
    if turns.is_empty() {
        return Err("receipt_missing");
    }
    if turns.len() != 1 {
        return Err("receipt_duplicate");
    }
    let turn = &turns[0]["data"];
    if let Some(saved) = saved {
        let intent = &saved.recovery["dispatch_intent"];
        let current = intent["input_version"] == json!(attempt.input_version)
            && events.iter().any(|event| {
                event["kind"] == "session.saved"
                    && event["data"]["id"] == json!(session.id)
                    && event["data"]["evidence"]["dispatch_intent"] == *intent
            });
        if current && turn["prompt"] != saved.recovery["prompt_id"] {
            return Err("current_prompt_mismatch");
        }
    }
    let receipt = closed_receipt(&turn["cleanup_receipt"])?;
    let clean = receipt["cleanup_ok"].as_bool().unwrap()
        && !receipt["ownership_uncertain"].as_bool().unwrap()
        && receipt["output_verified"].as_bool().unwrap();
    if turn["cleanup_verified"] != json!(clean) {
        return Err("clean_equation_mismatch");
    }
    Ok(receipt)
}
fn exact_keys(value: &Value, wanted: &[&str]) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.len() == wanted.len() && wanted.iter().all(|key| object.contains_key(*key))
}
pub(super) fn closed_receipt(value: &Value) -> Result<Value, &'static str> {
    let keys = [
        "owned_process_group_created",
        "cleanup_ok",
        "cleanup_state",
        "reap_io_kind",
        "output_verified",
        "stderr_drain_state",
        "stderr_read_error",
        "ownership_uncertain",
        "uncertainty_by_stage",
        "dispatched",
        "native_outcome",
    ];
    if !exact_keys(value, &keys) {
        return Err("receipt_shape_invalid");
    }
    for key in [
        "owned_process_group_created",
        "cleanup_ok",
        "output_verified",
        "ownership_uncertain",
        "dispatched",
        "native_outcome",
    ] {
        if !value[key].is_boolean() {
            return Err("receipt_boolean_invalid");
        }
    }
    if ![
        "not_attempted",
        "group_cleanup_failed_unclassified",
        "reap_timeout",
        "reap_error",
        "succeeded",
    ]
    .contains(&value["cleanup_state"].as_str().unwrap_or(""))
    {
        return Err("cleanup_vocabulary_invalid");
    }
    if ![
        "not_started",
        "joined_returned",
        "joined_panic",
        "joined_cancelled",
        "budget_elapsed_abort_requested",
    ]
    .contains(&value["stderr_drain_state"].as_str().unwrap_or(""))
    {
        return Err("drain_vocabulary_invalid");
    }
    if value["stderr_read_error"] != "unavailable" {
        return Err("read_error_unavailable_invalid");
    }
    if value["cleanup_state"] == "reap_error" {
        if ![
            "not_found",
            "permission_denied",
            "interrupted",
            "invalid_input",
            "invalid_data",
            "timed_out",
            "would_block",
            "unexpected_eof",
            "broken_pipe",
            "out_of_memory",
            "write_zero",
            "other",
        ]
        .contains(&value["reap_io_kind"].as_str().unwrap_or(""))
        {
            return Err("reap_vocabulary_invalid");
        }
    } else if !value["reap_io_kind"].is_null() {
        return Err("reap_not_applicable_invalid");
    }
    let owned = value["owned_process_group_created"].as_bool().unwrap();
    let cleanup_success_state = matches!(
        value["cleanup_state"].as_str(),
        Some("not_attempted" | "succeeded")
    );
    if value["cleanup_ok"] != json!(cleanup_success_state)
        || owned == (value["cleanup_state"] == "not_attempted")
    {
        return Err("cleanup_lifecycle_inconsistent");
    }
    if (!owned && value["stderr_drain_state"] != "not_started")
        || value["output_verified"]
            != json!(value["stderr_drain_state"] != "budget_elapsed_abort_requested")
    {
        return Err("drain_lifecycle_inconsistent");
    }
    let stages = &value["uncertainty_by_stage"];
    let names = [
        "native_child",
        "pre_spawn",
        "in_session_binding",
        "reconciliation",
    ];
    if !exact_keys(stages, &names) || names.iter().any(|key| !stages[*key].is_boolean()) {
        return Err("stage_shape_invalid");
    }
    if value["ownership_uncertain"]
        != json!(names.iter().any(|key| stages[*key].as_bool().unwrap()))
    {
        return Err("supervise_total_projection_mismatch");
    }
    Ok(value.clone()) // Only the exact bounded allowlist passed above.
}
pub(super) fn message(receipt: &Result<Value, &'static str>) -> String {
    match receipt {
        Ok(receipt) => format!("cleanup_receipt={receipt}"),
        Err(reason) => format!("cleanup_receipt_unavailable={reason}"),
    }
}
#[track_caller]
pub(super) fn assert_state<T: std::fmt::Debug + PartialEq>(
    actual: T,
    expected: T,
    context: &str,
    projection: &str,
) {
    assert_eq!(
        actual, expected,
        "grok_terminal_state_assertion {context}; {projection}"
    );
}
pub(super) fn assert_receipt(receipt: &Result<Value, &'static str>) {
    assert!(receipt.is_ok(), "{}", message(receipt));
}

#[cfg(test)]
mod tests {
    use super::*;
    fn receipt() -> Value {
        json!({"owned_process_group_created":true,"cleanup_ok":true,"cleanup_state":"succeeded","reap_io_kind":null,"output_verified":true,"stderr_drain_state":"joined_returned","stderr_read_error":"unavailable","ownership_uncertain":false,"uncertainty_by_stage":{"native_child":false,"pre_spawn":false,"in_session_binding":false,"reconciliation":false},"dispatched":true,"native_outcome":true})
    }
    fn session() -> Session {
        Session {
            id: SessionId::new(),
            scope: Scope::task(Default::default(), Default::default(), Default::default()),
            agent: "synthetic".into(),
            provider: "synthetic".into(),
            role: SessionRole::Executor,
            native_ref: None,
            pid: None,
            worktree: "/synthetic".into(),
            state: SessionState::Exited,
            model: None,
            effort: None,
            recovery: json!({"prompt_id":7,"dispatch_intent":{"input_version":2,"state":"dispatching"}}),
            started_at: 0,
        }
    }
    #[test]
    fn projection_rejects_extra_unbounded_values_and_inconsistent_operands() {
        let mut value = receipt();
        assert!(closed_receipt(&value).is_ok());
        value["private_body"] = json!("synthetic private body");
        let error = closed_receipt(&value).unwrap_err();
        assert_eq!(error, "receipt_shape_invalid");
        assert!(!message(&Err(error)).contains("synthetic private body"));
        value.as_object_mut().unwrap().remove("private_body");
        value["reap_io_kind"] = json!("timed_out");
        assert_eq!(
            closed_receipt(&value).unwrap_err(),
            "reap_not_applicable_invalid"
        );
        value["reap_io_kind"] = Value::Null;
        value["uncertainty_by_stage"]["native_child"] = json!(true);
        assert_eq!(
            closed_receipt(&value).unwrap_err(),
            "supervise_total_projection_mismatch"
        );
    }
    #[test]
    fn projection_rejects_impossible_cleanup_and_drain_combinations() {
        for (key, changed, error) in [
            ("cleanup_ok", json!(false), "cleanup_lifecycle_inconsistent"),
            (
                "cleanup_state",
                json!("group_cleanup_failed_unclassified"),
                "cleanup_lifecycle_inconsistent",
            ),
            (
                "owned_process_group_created",
                json!(false),
                "cleanup_lifecycle_inconsistent",
            ),
            (
                "output_verified",
                json!(false),
                "drain_lifecycle_inconsistent",
            ),
            (
                "stderr_drain_state",
                json!("budget_elapsed_abort_requested"),
                "drain_lifecycle_inconsistent",
            ),
        ] {
            let mut value = receipt();
            value[key] = changed;
            assert_eq!(closed_receipt(&value).unwrap_err(), error);
        }
        let mut absent = receipt();
        absent["owned_process_group_created"] = json!(false);
        absent["cleanup_state"] = json!("not_attempted");
        assert_eq!(
            closed_receipt(&absent).unwrap_err(),
            "drain_lifecycle_inconsistent"
        );
        absent["stderr_drain_state"] = json!("not_started");
        assert!(closed_receipt(&absent).is_ok());
        let mut elapsed = receipt();
        elapsed["output_verified"] = json!(false);
        elapsed["stderr_drain_state"] = json!("budget_elapsed_abort_requested");
        assert!(closed_receipt(&elapsed).is_ok());
    }
    #[test]
    fn exact_attempt_rejects_missing_duplicate_and_current_prompt_mismatch() {
        let saved = session();
        let attempt = Attempt {
            lower: 1,
            input_version: 2,
        };
        let turn = json!({"kind":"grok.turn_observed","data":{"session":saved.id,"prompt":6,"cleanup_verified":true,"cleanup_receipt":receipt()}});
        let intent = json!({"kind":"session.saved","data":{"id":saved.id,"evidence":{"dispatch_intent":saved.recovery["dispatch_intent"]}}});
        assert_eq!(
            exact_receipt(&[], &saved, Some(&saved), attempt).unwrap_err(),
            "receipt_missing"
        );
        assert_eq!(
            exact_receipt(&[turn.clone(), turn.clone()], &saved, Some(&saved), attempt)
                .unwrap_err(),
            "receipt_duplicate"
        );
        assert_eq!(
            exact_receipt(
                &[turn.clone(), intent.clone()],
                &saved,
                Some(&saved),
                attempt
            )
            .unwrap_err(),
            "current_prompt_mismatch"
        );
        // Earlier input intent is not authority for this attempt's prompt.
        assert!(
            exact_receipt(
                &[turn.clone(), intent],
                &saved,
                Some(&saved),
                Attempt {
                    lower: 1,
                    input_version: 3
                }
            )
            .is_ok()
        );
        let mut inconsistent = turn;
        inconsistent["data"]["cleanup_verified"] = json!(false);
        assert_eq!(
            exact_receipt(&[inconsistent], &saved, None, attempt).unwrap_err(),
            "clean_equation_mismatch"
        );
    }
    #[test]
    fn complete_paging_honors_closed_upper_window() {
        let fixture = super::super::fixture_support::Fixture::new();
        let mut store = fixture.store.lock().unwrap();
        let scope = &fixture.request.scope;
        let lower = events_through(&store, scope, 0, i64::MAX)
            .unwrap()
            .last()
            .unwrap()["sequence"]
            .as_i64()
            .unwrap();
        for ordinal in 0..205 {
            store
                .audit(scope, "synthetic.page", json!({"ordinal":ordinal}))
                .unwrap();
        }
        let upper = events_through(&store, scope, lower, i64::MAX)
            .unwrap()
            .last()
            .unwrap()["sequence"]
            .as_i64()
            .unwrap();
        store.audit(scope, "synthetic.later", json!({})).unwrap();
        let events = events_through(&store, scope, lower, upper).unwrap();
        assert_eq!(events.len(), 205);
        assert_eq!(events[204]["data"]["ordinal"], 204);
        assert!(events.iter().all(|event| event["kind"] == "synthetic.page"));
    }
}
