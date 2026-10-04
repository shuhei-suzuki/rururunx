use rrx::{
    adapter::{grok::GrokAdapter, *},
    domain::*,
    state::Store,
};
use serde_json::{Value, json};
use std::os::unix::fs::PermissionsExt;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

#[path = "support/grok_fixture.rs"]
mod fixture_support;
#[path = "support/grok_receipt.rs"]
mod receipt_support;
use fixture_support::{Fixture, git};
use rrx::git as fixture_git;

impl Fixture {
    fn review(&mut self) {
        rrx::git::WorktreeManager::lock_review(
            &mut self.store.lock().unwrap(),
            self.request.scope.task_id.unwrap(),
            &self.request.input.revision,
            "native review",
        )
        .unwrap();
        self.request.role = SessionRole::Reviewer;
        self.request.input.kind = InputKind::ReviewBundle;
    }
}
async fn finished(
    adapter: &dyn AgentAdapter,
    session: &Session,
    fixture: &Fixture,
) -> SessionStatus {
    let mut status = adapter.subscribe(session.into()).unwrap();
    let observed = tokio::time::timeout(Duration::from_secs(15), async {
        while !status.borrow().terminal() {
            status.changed().await.unwrap();
        }
        status.borrow().clone()
    })
    .await
    .unwrap();
    fixture.observe(&observed);
    observed
}

#[tokio::test]
async fn native_execute_edits_only_owned_files_and_preserves_actual_exit() {
    let mut fixture = Fixture::new();
    fixture.request.input.payload = "  /always-approve".into();
    fixture.request.environment.insert(
        "RRX_EXPECT_INPUT".into(),
        fixture.request.input.payload.clone(),
    );
    let adapter = fixture.adapter();
    let session = fixture.start(&adapter).await.unwrap();
    let status = finished(&adapter, &session, &fixture).await;
    receipt_support::assert_state(
        status.session.state,
        SessionState::Exited,
        &format!("{:?}", status.failure),
        &fixture.receipt_message(&status),
    );
    assert!(
        adapter.transport_succeeded(&status),
        "{}",
        fixture.receipt_message(&status)
    );
    assert!(
        status.session.pid.is_none(),
        "{}",
        fixture.receipt_message(&status)
    );
    assert_ne!(
        status.exit_code,
        Some(0),
        "{}",
        fixture.receipt_message(&status)
    );
    let observation = fixture.observation(&status);
    receipt_support::assert_receipt(&observation.receipt);
    assert_eq!(observation.saved.as_ref().unwrap().id, session.id);
    assert_eq!(observation.attempt.unwrap().input_version, 1);
    assert!(!observation.events.is_empty());

    assert_eq!(
        std::fs::read_to_string(fixture.request.worktree.join("result.txt")).unwrap(),
        "owned edit\n"
    );
    assert!(!fixture.directory.path().join("foreign.txt").exists());
    let usage = adapter
        .usage((&session).into(), "execute".into(), None)
        .await
        .unwrap();
    assert_eq!(usage.input_tokens, Some(101));
    assert_eq!(usage.cached_input_tokens, Some(0));
    assert_eq!(usage.estimated_cost, None);
    fixture.mode("replace_existing");
    let replacement = fixture.start(&adapter).await.unwrap();
    let replacement = finished(&adapter, &replacement, &fixture).await;
    assert!(
        adapter.transport_succeeded(&replacement),
        "{:?}; {}",
        replacement.failure,
        fixture.receipt_message(&replacement)
    );
    assert_eq!(
        std::fs::read_to_string(fixture.request.worktree.join("own.txt")).unwrap(),
        "owned edit\n"
    );
    fixture.mode("supplemental_read");
    let supplemental = fixture.start(&adapter).await.unwrap();
    let supplemental = finished(&adapter, &supplemental, &fixture).await;
    assert!(
        adapter.transport_succeeded(&supplemental),
        "{:?}; {}",
        supplemental.failure,
        fixture.receipt_message(&supplemental)
    );
    let mut forged = status.clone();
    fixture.mode("failed_called_read");
    let failed_called = fixture.start(&adapter).await.unwrap();
    let failed_called = finished(&adapter, &failed_called, &fixture).await;
    assert!(
        adapter.transport_succeeded(&failed_called),
        "{:?}; {}",
        failed_called.failure,
        fixture.receipt_message(&failed_called)
    );
    forged.session.recovery["prompt_id"] = json!("forged");
    assert!(
        !adapter.transport_succeeded(&forged),
        "{}",
        fixture.receipt_message(&forged)
    );
    for status in [&replacement, &supplemental, &failed_called] {
        receipt_support::assert_receipt(&fixture.observation(status).receipt);
    }
    adapter.release((&session).into()).unwrap();
}
#[tokio::test]
async fn concurrent_native_reviewers_share_exact_lock_and_validate_structured_verdict() {
    let mut fixture = Fixture::new();
    fixture.review();
    let adapter = Arc::new(fixture.adapter());
    let schema = json!({"type":"object","properties":{"verdict":{"type":"string","enum":["DENY"]},"reason":{"type":"string"}},"required":["verdict","reason"],"additionalProperties":false});
    let (one, two) = tokio::join!(
        fixture.start_structured(&*adapter, schema.clone()),
        fixture.start_structured(&*adapter, schema.clone())
    );
    let one = one.unwrap();
    let two = two.unwrap();
    assert_ne!(one.id, two.id);
    let (a, b) = tokio::join!(
        finished(&*adapter, &one, &fixture),
        finished(&*adapter, &two, &fixture)
    );
    assert!(
        adapter.transport_succeeded(&a),
        "{:?}; {}",
        a.failure,
        fixture.receipt_message(&a)
    );
    assert!(
        adapter.transport_succeeded(&b),
        "{:?}; {}",
        b.failure,
        fixture.receipt_message(&b)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&a.stdout).unwrap()["verdict"],
        "DENY"
    );
    // Issue 9 owns Review Set policy. Prove this provider's registered object-safe
    // launch/status/completion lifecycle independently of its inherent schema API.
    let mut registry = AgentRegistry::default();
    registry.register("grok".into(), adapter).unwrap();
    let registered = registry.get("grok").unwrap();
    assert!(registered.capabilities().contains(&Capability::Review));
    let (three, four) = tokio::join!(fixture.start(&*registered), fixture.start(&*registered));
    let three = three.unwrap();
    let four = four.unwrap();
    assert_ne!(three.id, four.id);
    let (c, d) = tokio::join!(
        finished(&*registered, &three, &fixture),
        finished(&*registered, &four, &fixture)
    );
    assert!(
        registered.transport_succeeded(&c),
        "{:?}; {}",
        c.failure,
        fixture.receipt_message(&c)
    );
    assert!(
        registered.transport_succeeded(&d),
        "{:?}; {}",
        d.failure,
        fixture.receipt_message(&d)
    );
    assert_eq!(c.session.scope, fixture.request.scope);
    assert_eq!(d.session.scope, fixture.request.scope);
    assert_eq!(
        serde_json::to_value(registered.status((&three).into()).await.unwrap().session).unwrap(),
        serde_json::to_value(&c.session).unwrap()
    );
    // The workflow receives dyn AgentAdapter from the registry. Caller constraints
    // must reach the same native prompt and local validator through that path.
    let (five, six) = tokio::join!(
        fixture.start_structured(&*registered, schema.clone()),
        fixture.start_structured(&*registered, schema)
    );
    let five = five.unwrap();
    let six = six.unwrap();
    assert_ne!(five.id, six.id);
    let (e, f) = tokio::join!(
        finished(&*registered, &five, &fixture),
        finished(&*registered, &six, &fixture)
    );
    for status in [&e, &f] {
        assert!(
            registered.transport_succeeded(status),
            "{:?}",
            status.failure
        );
        let output: Value = serde_json::from_slice(&status.stdout).unwrap();
        assert_eq!(output["verdict"], "DENY");
        assert!(output["reason"].is_string());
        assert_eq!(output.as_object().unwrap().len(), 2);
    }
    for status in [&a, &b, &c, &d, &e, &f] {
        receipt_support::assert_receipt(&fixture.observation(status).receipt);
    }
    for session in [&one, &two, &three, &four, &five, &six] {
        registered.release(session.into()).unwrap();
    }
}
#[tokio::test]
async fn native_auth_inventory_config_parser_and_tool_evidence_fail_closed() {
    for mode in [
        "version",
        "no_auth_method",
        "auth_error",
        "inventory",
        "config",
        "malformed",
        "oversize",
        "bypass",
        "hook",
        "unfinished",
        "denied_completed",
        "late_write",
        "late_tool",
        "invalid_callback_id",
        "config_update",
        "callback_budget",
        "path_budget",
        "wrong_method",
        "ambiguous",
        "unnotified",
        "hook_failure",
        "unowned_write",
        "unowned_read",
        "unknown_fs_method",
    ] {
        let mut fixture = Fixture::new();
        fixture.mode(mode);
        let adapter = fixture.adapter();
        let session = fixture.start(&adapter).await.unwrap();
        let status = finished(&adapter, &session, &fixture).await;
        receipt_support::assert_state(
            status.session.state,
            if [
                "malformed",
                "oversize",
                "bypass",
                "denied_completed",
                "invalid_callback_id",
                "config_update",
                "callback_budget",
                "path_budget",
                "wrong_method",
                "ambiguous",
                "unowned_write",
                "unowned_read",
                "unknown_fs_method",
            ]
            .contains(&mode)
            {
                SessionState::Lost
            } else {
                SessionState::Failed
            },
            &format!("{mode}: {:?}", status.failure),
            &fixture.receipt_message(&status),
        );
        assert!(
            !adapter.transport_succeeded(&status),
            "{}",
            fixture.receipt_message(&status)
        );
        assert!(
            status.failure.is_some(),
            "{mode}; {}",
            fixture.receipt_message(&status)
        );
        if mode == "hook_failure" {
            let window = fixture.observation(&status);
            let turns: Vec<_> = window
                .events
                .iter()
                .filter(|event| {
                    event["kind"] == "grok.turn_observed"
                        && event["data"]["session"] == json!(session.id)
                })
                .collect();
            assert_eq!(turns.len(), 1, "{}", fixture.receipt_message(&status));
            let observation = &turns[0]["data"];
            assert_eq!(
                observation["reconciliation_attempted"],
                true,
                "{}",
                fixture.receipt_message(&status)
            );
            assert!(
                observation["reconciliation_error"]
                    .as_str()
                    .unwrap()
                    .contains("unexplained native/concurrent worktree effect"),
                "{}",
                fixture.receipt_message(&status)
            );
        }
        if mode == "unowned_write" {
            assert!(
                !fixture.request.worktree.join("unowned.txt").exists(),
                "unowned write must be rejected before effects"
            );
        }
        if mode == "unknown_fs_method" {
            assert!(
                !fixture
                    .observation(&status)
                    .events
                    .iter()
                    .any(|event| event["kind"] == "grok.fs_observed"
                        && event["data"]["session"] == json!(session.id)),
                "unsupported FS method must never read or write; {}",
                fixture.receipt_message(&status)
            );
        }
        assert!(
            status.session.pid.is_none(),
            "{mode}; {}",
            fixture.receipt_message(&status)
        );
        assert!(
            !fixture.request.worktree.join("late.txt").exists(),
            "{mode}: late callback mutated Task"
        );
        assert!(
            !fixture.request.worktree.join("invalid-id.txt").exists(),
            "{mode}: malformed request mutated Task"
        );
        receipt_support::assert_receipt(&fixture.observation(&status).receipt);
    }
}
#[tokio::test]
async fn native_resume_requires_fresh_checkpoint_preserves_uuid_and_discards_replay() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter();
    let session = fixture.start(&adapter).await.unwrap();
    let first = finished(&adapter, &session, &fixture).await;
    assert!(
        adapter.transport_succeeded(&first),
        "{:?}; {}",
        first.failure,
        fixture.receipt_message(&first)
    );
    receipt_support::assert_receipt(&fixture.observation(&first).receipt);
    assert_eq!(
        adapter.resume((&session).into()).await.unwrap_err().kind,
        ErrorKind::InvalidInput
    );
    let mut input = fixture.request.input.clone();
    input.version = 2;
    input.payload = "explicit fresh continuation".into();
    adapter.checkpoint((&session).into(), input).await.unwrap();
    let resumed = fixture
        .resume(&adapter, (&session).into(), 2)
        .await
        .unwrap();
    assert_eq!(resumed.id, session.id);
    let second = finished(&adapter, &resumed, &fixture).await;
    assert_eq!(
        second.session.native_ref,
        first.session.native_ref,
        "{}",
        fixture.receipt_message(&second)
    );
    assert!(
        adapter.transport_succeeded(&second),
        "{:?}; {}",
        second.failure,
        fixture.receipt_message(&second)
    );
    assert!(!String::from_utf8_lossy(&second.stdout).contains("REPLAY"));
    assert_eq!(
        second.session.recovery["input_version"],
        2,
        "{}",
        fixture.receipt_message(&second)
    );
    let usage = adapter
        .usage((&resumed).into(), "resume".into(), None)
        .await
        .unwrap();
    assert_eq!(usage.input_tokens, Some(202));
    assert_eq!(usage.output_tokens, Some(22));
    receipt_support::assert_receipt(&fixture.observation(&second).receipt);
}
#[tokio::test]
async fn native_stop_permissions_foreign_refs_and_environment_guards_are_explicit() {
    let mut fixture = Fixture::new();
    fixture.mode("permission");
    let adapter = fixture.adapter();
    let session = fixture.start(&adapter).await.unwrap();
    let initial = finished(&adapter, &session, &fixture).await;
    assert!(
        adapter.transport_succeeded(&initial),
        "{}",
        fixture.receipt_message(&initial)
    );
    let mut foreign = SessionRef::from(&session);
    foreign.scope.task_id = Some(TaskId::new());
    assert_eq!(
        adapter.status(foreign).await.unwrap_err().kind,
        ErrorKind::OwnershipMismatch
    );
    fixture.mode("hang");
    let spawn_observed = fixture.directory.path().join("spawn-observed");
    fixture.request.environment.insert(
        "RRX_SPAWN_OBSERVED".into(),
        spawn_observed.to_str().unwrap().into(),
    );
    let session = fixture.start(&adapter).await.unwrap();
    let stopped = adapter.stop((&session).into()).await.unwrap();
    fixture.observe(&stopped);
    receipt_support::assert_state(
        stopped.session.state,
        SessionState::Stopped,
        "terminal",
        &fixture.receipt_message(&stopped),
    );
    assert!(
        !spawn_observed.exists(),
        "stop during preflight still spawned native process"
    );
    assert!(
        !fixture
            .observation(&stopped)
            .events
            .iter()
            .any(|event| event["kind"] == "grok.process_spawned"
                && event["data"]["session"] == json!(session.id)),
        "stopped native process was spawned but killed before fixture startup; {}",
        fixture.receipt_message(&stopped)
    );
    assert!(
        stopped.session.pid.is_none(),
        "{}",
        fixture.receipt_message(&stopped)
    );
    assert!(
        !adapter.transport_succeeded(&stopped),
        "{}",
        fixture.receipt_message(&stopped)
    );
    let observation = fixture.observation(&stopped);
    receipt_support::assert_receipt(&observation.receipt);
    let receipt = observation.receipt.unwrap();
    assert_eq!(receipt["owned_process_group_created"], false);
    assert_eq!(receipt["cleanup_state"], "not_attempted");
    assert_eq!(receipt["cleanup_ok"], true);
    assert_eq!(receipt["output_verified"], true);
    assert_eq!(receipt["stderr_drain_state"], "not_started");
    assert_eq!(receipt["ownership_uncertain"], false);
    assert_eq!(
        receipt["uncertainty_by_stage"],
        json!({"native_child":false,"pre_spawn":false,"in_session_binding":false,"reconciliation":false})
    );

    for key in [
        "HOME",
        "GROK_HOME",
        "XAI_API_KEY",
        "LD_PRELOAD",
        "DYLD_INSERT_LIBRARIES",
        "NODE_OPTIONS",
        "NODE_TLS_REJECT_UNAUTHORIZED",
        "NODE_EXTRA_CA_CERTS",
        "NODE_PATH",
        "BUN_OPTIONS",
        "OPENSSL_CONF",
        "SSLKEYLOGFILE",
        "BASH_ENV",
        "ENV",
        "SHELL",
        "ZDOTDIR",
        "UNKNOWN_NATIVE_OVERRIDE",
        "HTTPS_PROXY",
    ] {
        let mut request = fixture.request.clone();
        request
            .environment
            .insert(key.into(), "foreign override".into());
        assert_eq!(
            adapter.start(request).await.unwrap_err().kind,
            ErrorKind::InvalidConfiguration
        );
    }
    assert_eq!(
        adapter.attach((&session).into()).await.unwrap_err().kind,
        ErrorKind::UnsupportedCapability
    );
    assert!(
        !adapter
            .capabilities()
            .contains(&Capability::PermissionInterception)
    );
}

#[tokio::test]
async fn unknown_native_dispatch_keeps_clean_dead_executor_reserved() {
    let mut fixture = Fixture::new();
    fixture.mode("oversize");
    let adapter = fixture.adapter();
    let session = fixture.start(&adapter).await.unwrap();
    let status = finished(&adapter, &session, &fixture).await;
    receipt_support::assert_state(
        status.session.state,
        SessionState::Lost,
        "terminal",
        &fixture.receipt_message(&status),
    );
    assert!(
        status.session.pid.is_none(),
        "{}",
        fixture.receipt_message(&status)
    );
    assert!(
        !adapter.transport_succeeded(&status),
        "{}",
        fixture.receipt_message(&status)
    );
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .session(session.id)
            .unwrap()
            .unwrap()
            .0
            .state,
        SessionState::Lost,
        "{}",
        fixture.receipt_message(&status)
    );
    assert_eq!(
        git(&fixture.request.worktree, &["status", "--porcelain"]),
        ""
    );
    assert!(
        rrx::git::WorktreeManager::lock_review(
            &mut fixture.store.lock().unwrap(),
            fixture.request.scope.task_id.unwrap(),
            &fixture.request.input.revision,
            "cannot lock Lost executor"
        )
        .is_err(),
        "{}",
        fixture.receipt_message(&status)
    );
    assert_eq!(
        adapter
            .start(fixture.request.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::StateConflict,
        "{}",
        fixture.receipt_message(&status)
    );
    assert_eq!(
        adapter.release((&session).into()).unwrap_err().kind,
        ErrorKind::SessionLost,
        "{}",
        fixture.receipt_message(&status)
    );
    let mut input = fixture.request.input.clone();
    input.version = 2;
    assert_eq!(
        adapter
            .checkpoint((&session).into(), input)
            .await
            .unwrap_err()
            .kind,
        ErrorKind::InvalidInput,
        "{}",
        fixture.receipt_message(&status)
    );
}

#[tokio::test]
async fn stop_after_dispatch_preserves_unknown_outcome_until_explicit_recovery() {
    let mut fixture = Fixture::new();
    fixture.mode("hang");
    let adapter = fixture.adapter();
    let session = fixture.start(&adapter).await.unwrap();
    let mut watch = adapter.subscribe((&session).into()).unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        while watch.borrow().session.state != SessionState::Running {
            watch.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    let stopped = adapter.stop((&session).into()).await.unwrap();
    fixture.observe(&stopped);
    receipt_support::assert_state(
        stopped.session.state,
        SessionState::Lost,
        "terminal",
        &fixture.receipt_message(&stopped),
    );
    assert!(
        stopped.session.pid.is_none(),
        "{}",
        fixture.receipt_message(&stopped)
    );
    assert!(
        !adapter.transport_succeeded(&stopped),
        "{}",
        fixture.receipt_message(&stopped)
    );
    assert_eq!(
        adapter
            .start(fixture.request.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::StateConflict,
        "{}",
        fixture.receipt_message(&stopped)
    );
    receipt_support::assert_receipt(&fixture.observation(&stopped).receipt);
}

#[tokio::test]
#[ignore = "requires installed Grok and existing native auth; explicit isolated acceptance"]
async fn installed_native_edit_fresh_continuation_and_structured_decision() {
    let mut fixture = Fixture::new();
    let native = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|path| path.join("grok"))
        .find(|path| path.is_file())
        .expect("installed native Grok");
    let adapter = GrokAdapter::new(
        "grok".into(),
        native.canonicalize().unwrap(),
        fixture.store.clone(),
    )
    .unwrap();
    fixture.request.model = Some("grok-4.7".into());
    fixture.request.effort = Some("low".into());
    let foreign = fixture.directory.path().join("foreign.txt");
    fixture.request.input.payload = format!(
        "Authorized isolated native adapter acceptance. Use read_file to read own.txt. Use search_replace to create result.txt containing exactly NATIVE_EDIT_PROVED plus a newline. Then explicitly attempt search_replace on {} with content FOREIGN_MUST_BE_DENIED, and report the supervisor rejection. Use only those named tools and fixture paths. Finish when the own file is correct; do not use alternate methods.",
        foreign.display()
    );
    async fn native_finished(
        adapter: &GrokAdapter,
        session: &Session,
        fixture: &Fixture,
    ) -> SessionStatus {
        let mut status = adapter.subscribe(session.into()).unwrap();
        let observed = tokio::time::timeout(Duration::from_secs(330), async {
            while !status.borrow().terminal() {
                status.changed().await.unwrap();
            }
            status.borrow().clone()
        })
        .await
        .unwrap();
        fixture.observe(&observed);
        observed
    }
    let session = fixture.start(&adapter).await.unwrap();
    let first = native_finished(&adapter, &session, &fixture).await;
    assert!(
        adapter.transport_succeeded(&first),
        "{:?}; {}",
        first.failure,
        fixture.receipt_message(&first)
    );
    assert_eq!(
        std::fs::read_to_string(fixture.request.worktree.join("result.txt")).unwrap(),
        "NATIVE_EDIT_PROVED\n"
    );
    assert!(!foreign.exists());
    let mut input = fixture.request.input.clone();
    input.version = 2;
    input.payload="New explicit continuation input. Read result.txt and preserve it. Create continued.txt with exactly NATIVE_CONTINUATION_PROVED plus a newline using search_replace. Do not repeat or modify the prior file, and use no other paths/tools.".into();
    adapter.checkpoint((&session).into(), input).await.unwrap();
    let resumed = fixture
        .resume(&adapter, (&session).into(), 2)
        .await
        .unwrap();
    let second = native_finished(&adapter, &resumed, &fixture).await;
    assert!(
        adapter.transport_succeeded(&second),
        "{:?}; {}",
        second.failure,
        fixture.receipt_message(&second)
    );
    assert_eq!(
        second.session.native_ref,
        first.session.native_ref,
        "{}",
        fixture.receipt_message(&second)
    );
    assert_eq!(
        std::fs::read_to_string(fixture.request.worktree.join("continued.txt")).unwrap(),
        "NATIVE_CONTINUATION_PROVED\n"
    );
    git(
        &fixture.request.worktree,
        &["add", "result.txt", "continued.txt"],
    );
    git(
        &fixture.request.worktree,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-m",
            "verified native fixture",
        ],
    );
    fixture.request.input.version = 3;
    fixture.request.input.revision = git(&fixture.request.worktree, &["rev-parse", "HEAD"]);
    fixture.request.input.payload="Decision-only supplied public fixture bundle. Requirement: writes must remain in the owned Task. Proposed operation: write to a different Project. Return verdict DENY and a concise reason; no files, searches, tools or operations are authorized.".into();
    fixture.review();
    let schema = json!({"type":"object","properties":{"verdict":{"type":"string","enum":["DENY"]},"reason":{"type":"string"}},"required":["verdict","reason"],"additionalProperties":false});
    let review = fixture.start_structured(&adapter, schema).await.unwrap();
    let decision = native_finished(&adapter, &review, &fixture).await;
    assert!(
        adapter.transport_succeeded(&decision),
        "{:?}; {}",
        decision.failure,
        fixture.receipt_message(&decision)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&decision.stdout).unwrap()["verdict"],
        "DENY"
    );
    assert_eq!(
        git(&fixture.request.worktree, &["status", "--porcelain"]),
        ""
    );
    let usage = adapter
        .usage((&resumed).into(), "continuation".into(), None)
        .await
        .unwrap();
    assert!(usage.input_tokens.is_some_and(|v| v > 0));
    assert!(usage.output_tokens.is_some_and(|v| v > 0));
    assert_eq!(usage.estimated_cost, None);
}

#[tokio::test]
#[ignore = "requires installed Grok and existing native auth; isolated decision correlation acceptance"]
async fn installed_native_structured_decision_has_exact_response_correlation() {
    let mut fixture = Fixture::new();
    let native = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|path| path.join("grok"))
        .find(|path| path.is_file())
        .expect("installed native Grok");
    let adapter = GrokAdapter::new(
        "grok".into(),
        native.canonicalize().unwrap(),
        fixture.store.clone(),
    )
    .unwrap();
    fixture.review();
    fixture.request.model = Some("grok-4.7".into());
    fixture.request.effort = Some("low".into());
    fixture.request.input.payload = "Decision-only supplied fixture bundle. Requirement: writes must remain in the owned Task. Proposed operation: write to a different Project. Return verdict DENY and a concise reason; no files, searches, tools or operations are authorized.".into();
    let schema = json!({"type":"object","properties":{"verdict":{"type":"string","enum":["DENY"]},"reason":{"type":"string"}},"required":["verdict","reason"],"additionalProperties":false});
    let mut registry = AgentRegistry::default();
    registry.register("grok".into(), Arc::new(adapter)).unwrap();
    let adapter = registry.get("grok").unwrap();
    let session = fixture.start_structured(&*adapter, schema).await.unwrap();
    let mut status = adapter.subscribe((&session).into()).unwrap();
    let decision = tokio::time::timeout(Duration::from_secs(330), async {
        while !status.borrow().terminal() {
            status.changed().await.unwrap();
        }
        status.borrow().clone()
    })
    .await
    .unwrap();
    fixture.observe(&decision);
    assert!(
        adapter.transport_succeeded(&decision),
        "{:?}; {}",
        decision.failure,
        fixture.receipt_message(&decision)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&decision.stdout).unwrap()["verdict"],
        "DENY"
    );
    assert_eq!(
        git(&fixture.request.worktree, &["status", "--porcelain"]),
        ""
    );
}

#[tokio::test]
async fn native_structured_consumer_rejects_each_schema_violation_and_live_decision_tools() {
    for registered_path in [false, true] {
        for mode in [
            "schema_enum",
            "schema_required",
            "schema_extra",
            "decision_tool",
        ] {
            let mut fixture = Fixture::new();
            fixture.review();
            fixture.mode(mode);
            let adapter = Arc::new(fixture.adapter());
            let mut registry = AgentRegistry::default();
            registry.register("grok".into(), adapter.clone()).unwrap();
            let registered = registry.get("grok").unwrap();
            let selected: &dyn AgentAdapter = if registered_path {
                &*registered
            } else {
                &*adapter
            };
            let schema = json!({"type":"object","properties":{"verdict":{"type":"string","enum":["DENY"]},"reason":{"type":"string"}},"required":["verdict","reason"],"additionalProperties":false});
            let session = fixture.start_structured(selected, schema).await.unwrap();
            let status = finished(selected, &session, &fixture).await;
            receipt_support::assert_state(
                status.session.state,
                if mode == "decision_tool" {
                    SessionState::Lost
                } else {
                    SessionState::Failed
                },
                &format!("{mode}: {:?}", status.failure),
                &fixture.receipt_message(&status),
            );
            assert!(
                !adapter.transport_succeeded(&status),
                "{}",
                fixture.receipt_message(&status)
            );
            assert!(
                status.failure.is_some(),
                "{}",
                fixture.receipt_message(&status)
            );
            receipt_support::assert_receipt(&fixture.observation(&status).receipt);
            if status.session.state == SessionState::Lost {
                assert_eq!(
                    selected.release((&session).into()).unwrap_err().kind,
                    ErrorKind::SessionLost,
                    "uncertain decision/cleanup retains reservation; {mode}, registered={registered_path}"
                );
            } else {
                selected.release((&session).into()).unwrap();
            }
        }
    }
}

#[tokio::test]
async fn parent_replacement_after_native_preflight_never_reaches_prompt_wire() {
    let mut fixture = Fixture::new();
    fixture.mode("pause_info");
    let pause = fixture.directory.path().join("pause");
    let prompt_observed = fixture.directory.path().join("prompt-observed");
    fixture
        .request
        .environment
        .insert("RRX_PAUSE".into(), pause.to_str().unwrap().into());
    fixture.request.environment.insert(
        "RRX_PROMPT_OBSERVED".into(),
        prompt_observed.to_str().unwrap().into(),
    );
    let adapter = fixture.adapter();
    let session = fixture.start(&adapter).await.unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        while !pause.exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let mut other = Store::open(&fixture.directory.path().join("state.db")).unwrap();
    let mut task = other
        .task(fixture.request.scope.task_id.unwrap())
        .unwrap()
        .unwrap();
    task.title = "concurrent replacement after native admission".into();
    other.put_task(&mut task).unwrap();
    std::fs::write(pause.with_extension("continue"), "resume native response").unwrap();
    let status = finished(&adapter, &session, &fixture).await;
    receipt_support::assert_state(
        status.session.state,
        SessionState::Failed,
        &format!("{:?}", status.failure),
        &fixture.receipt_message(&status),
    );
    assert!(
        !adapter.transport_succeeded(&status),
        "{}",
        fixture.receipt_message(&status)
    );
    assert!(
        status.session.pid.is_none(),
        "{}",
        fixture.receipt_message(&status)
    );
    assert!(
        !prompt_observed.exists(),
        "native prompt was sent under replaced parent authority"
    );
    let saved = other.session(session.id).unwrap().unwrap().0;
    assert!(saved.recovery.get("prompt_id").is_none());
    assert_ne!(saved.recovery["dispatch_state"], "dispatching");
    assert_eq!(other.task(task.id).unwrap().unwrap().title, task.title);
    receipt_support::assert_receipt(&fixture.observation(&status).receipt);
}
