//! macOS actual supervise receipts; synthetic scoped ACP, no model/auth/config.
use super::*;

use super::fixture_support::Fixture;

async fn terminal(adapter: &GrokAdapter, session: &Session, fixture: &Fixture) -> SessionStatus {
    let mut watch = adapter.subscribe(session.into()).unwrap();
    let status = tokio::time::timeout(Duration::from_secs(15), async {
        while !watch.borrow().terminal() {
            watch.changed().await.unwrap();
        }
        watch.borrow().clone()
    })
    .await
    .unwrap();
    fixture.observe(&status);
    status
}

async fn sanitized(name: &str, expected_failure: bool) {
    let home = tempfile::tempdir().unwrap();
    let mut command = tokio::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", name, "--ignored", "--nocapture"])
        .env_clear()
        .env("HOME", home.path())
        .env("PATH", "/usr/bin:/bin")
        .env("RRX_INSPECTION_FIXTURE_CHILD", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .process_group(0);
    let mut child =
        ProcessGroup::new(command.spawn().unwrap(), Arc::new(AtomicBool::new(false))).unwrap();
    let stdout = tokio::spawn(read_git_output(child.child.stdout.take().unwrap()));
    let stderr = tokio::spawn(read_git_output(child.child.stderr.take().unwrap()));
    let observed = tokio::time::timeout(Duration::from_secs(60), child.observe_exit()).await;
    child = cleanup_group(child).await.unwrap();
    let exit = child.reap().await.unwrap();
    let stdout = String::from_utf8(stdout.await.unwrap().unwrap()).unwrap();
    let stderr = String::from_utf8(stderr.await.unwrap().unwrap()).unwrap();
    assert!(
        observed.is_ok(),
        "owned sanitized fixture deadline; {stdout}; {stderr}"
    );
    if !expected_failure {
        assert!(
            exit.success(),
            "owned sanitized fixture failed; {stdout}; {stderr}"
        );
        assert_child_completed(&stdout, &stderr, name);
    } else {
        assert!(!exit.success(), "expected state assertion did not execute");
        assert!(
            stdout.contains("running 1 test") && stdout.contains("0 passed; 1 failed"),
            "wrong failure count; {stdout}"
        );
        assert!(
            stderr.contains("grok_terminal_state_assertion expected_clean_protocol"),
            "state assertion label missing; {stderr}"
        );
        assert!(
            stderr.contains("receipt_tests.rs:")
                && stderr.contains("left: Lost")
                && stderr.contains("right: Failed"),
            "wrong assertion location/state; {stderr}"
        );
        let message = stderr
            .lines()
            .find(|line| line.contains("grok_terminal_state_assertion expected_clean_protocol"))
            .unwrap();
        assert!(
            message.contains("group_cleanup_failed_unclassified")
                && message.contains("\"ownership_uncertain\":true")
                && message.contains("\"native_child\":true"),
            "state assertion lost bounded cleanup facts: {message}"
        );
    }
}
#[tokio::test]
async fn dispatched_clean_receipt_reaches_actual_supervise_stages() {
    sanitized(
        "adapter::grok::receipt_tests::dispatched_clean_child",
        false,
    )
    .await;
}
#[tokio::test]
async fn dispatched_unknown_receipt_retains_result_error_and_reservation() {
    sanitized(
        "adapter::grok::receipt_tests::dispatched_unknown_child",
        false,
    )
    .await;
}
#[tokio::test]
async fn completed_resume_receipts_keep_exact_attempt_windows_and_trace() {
    sanitized(
        "adapter::grok::receipt_tests::completed_resume_child",
        false,
    )
    .await;
}
#[tokio::test]
async fn before_spawn_stop_receipt_has_only_preflight_creation_trace() {
    sanitized(
        "adapter::grok::receipt_tests::before_spawn_stop_child",
        false,
    )
    .await;
}
#[tokio::test]
async fn earlier_state_assertion_exposes_bounded_unclean_receipt() {
    sanitized("adapter::grok::receipt_tests::state_assertion_child", true).await;
}
fn synthetic_fixture(mode: &str) -> Fixture {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    let mut fixture = Fixture::new();
    fixture.mode(mode);
    for (name, file) in [
        ("RRX_PROMPT_OBSERVED", "prompt-observed"),
        ("RRX_PYTHON_OBSERVED", "python-observed"),
    ] {
        fixture.request.environment.insert(
            name.into(),
            fixture.directory.path().join(file).to_str().unwrap().into(),
        );
    }
    for name in [
        "RRX_DATABASE",
        "RRX_FOREIGN",
        "RRX_PROMPT_OBSERVED",
        "RRX_PYTHON_OBSERVED",
    ] {
        let path = PathBuf::from(&fixture.request.environment[name]);
        let path = path
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .join(path.file_name().unwrap());
        let directory = fixture.directory.path().canonicalize().unwrap();
        assert!(
            path.starts_with(directory)
                && !path.starts_with(&fixture.request.project.root)
                && !path.starts_with(&fixture.request.worktree),
            "synthetic sidecar layout"
        );
    }
    fixture
}
fn assert_full_trace(
    adapter: &GrokAdapter,
    status: &SessionStatus,
    fixture: &Fixture,
    version: u64,
) {
    let observation = fixture.observation(status);
    let turn = observation
        .events
        .iter()
        .find(|event| {
            event["kind"] == "grok.turn_observed"
                && event["data"]["session"] == json!(status.session.id)
        })
        .unwrap();
    assert_eq!(
        turn["data"]["reconciliation_attempted"],
        true,
        "{}",
        fixture.receipt_message(status)
    );
    assert!(
        turn["data"]["reconciliation_error"].is_null(),
        "{}",
        fixture.receipt_message(status)
    );
    let entry = adapter.entry(&(&status.session).into()).unwrap();
    let trace = entry
        .ownership_trace
        .snapshot(version)
        .expect("trace incomplete; no stage mutation credit");
    use OwnershipStage::*;
    let phases = [PreSpawn, NativeChild, InSessionBinding, Reconciliation];
    let mut offset = 0;
    let counts: Vec<_> = phases
        .iter()
        .map(|stage| {
            let before = offset;
            while trace.get(offset) == Some(stage) {
                offset += 1;
            }
            offset - before
        })
        .collect();
    assert_eq!(
        offset,
        trace.len(),
        "actual stage order has extra/out-of-order entry"
    );
    assert!(
        counts.iter().all(|count| *count > 0),
        "actual stage block absent"
    );
    assert_eq!(counts[1], 1);
    assert_eq!(
        counts[0], counts[3],
        "reconcile-binding actual stage size relation"
    );
    assert_eq!(
        counts[0],
        counts[2] + 1,
        "verify+index actual stage size relation"
    );
}
fn assert_dispatch_prerequisites(fixture: &Fixture, status: &SessionStatus) {
    // Independent of the turn event and the cleanup receipt under mutation.
    assert!(
        fixture.directory.path().join("prompt-observed").exists(),
        "prompt wire marker missing"
    );
    let observation = fixture.observation(status);
    let saved = observation.saved.as_ref().unwrap();
    let version = observation.attempt.unwrap().input_version;
    assert_eq!(saved.recovery["dispatch_state"], "dispatching");
    assert_eq!(saved.recovery["dispatch_intent"]["input_version"], version);
    let prompt = saved.recovery["prompt_id"].as_str().unwrap();
    assert!(
        observation
            .events
            .iter()
            .any(|event| event["kind"] == "session.saved"
                && event["data"]["id"] == json!(saved.id)
                && event["data"]["evidence"]["dispatch_intent"]
                    == saved.recovery["dispatch_intent"])
    );
    let callbacks: Vec<_> = observation
        .events
        .iter()
        .filter(|event| {
            event["kind"] == "grok.fs_observed"
                && event["data"]["session"] == json!(saved.id)
                && event["data"]["prompt"] == prompt
                && event["data"]["path"] == "unseen.txt"
        })
        .collect();
    assert_eq!(
        callbacks.len(),
        1,
        "exact independent unseen-read callback missing"
    );
    let callback = &callbacks[0]["data"];
    assert_eq!(callback["method"], "fs/read_text_file");
    assert_eq!(callback["succeeded"], true);
    assert_eq!(callback["effect_may_have_occurred"], false);
    let identity: Value = serde_json::from_slice(
        &std::fs::read(fixture.directory.path().join("python-observed")).unwrap(),
    )
    .unwrap();
    assert!(
        identity["executable"]
            .as_str()
            .is_some_and(|s| !s.is_empty())
    );
    assert_eq!(identity["version_info"][0], 3);
    eprintln!("independent dispatched prerequisites passed");
}
async fn dispatched(unknown: bool) {
    let fixture = synthetic_fixture("unowned_read");
    let mut adapter = fixture.adapter();
    if unknown {
        adapter.process_inspection = Some(ProcessInspectionPlan::unknown(
            UnknownObservation::Diagnostics,
        ));
    }
    let session = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &session, &fixture).await;
    let projection = fixture.receipt_message(&status);
    receipt_support::assert_state(
        status.session.state,
        SessionState::Lost,
        "dispatched_unowned_read",
        &projection,
    );
    assert!(!adapter.transport_succeeded(&status), "{projection}");

    assert_dispatch_prerequisites(&fixture, &status);
    let observation = fixture.observation(&status);
    receipt_support::assert_receipt(&observation.receipt);
    let receipt = observation.receipt.as_ref().unwrap();
    assert_eq!(receipt["dispatched"], true);
    assert_eq!(receipt["native_outcome"], false);
    assert_eq!(receipt["owned_process_group_created"], true);
    assert_eq!(receipt["cleanup_ok"], !unknown, "{projection}");
    assert_eq!(
        receipt["cleanup_state"],
        if unknown {
            "group_cleanup_failed_unclassified"
        } else {
            "succeeded"
        },
        "{projection}"
    );
    assert_eq!(receipt["ownership_uncertain"], unknown, "{projection}");
    assert_eq!(
        receipt["uncertainty_by_stage"]["native_child"], unknown,
        "{projection}"
    );
    let turn = observation
        .events
        .iter()
        .find(|event| {
            event["kind"] == "grok.turn_observed" && event["data"]["session"] == json!(session.id)
        })
        .unwrap();
    assert_eq!(turn["data"]["completed"], false);
    assert_eq!(turn["data"]["cleanup_verified"], !unknown, "{projection}");
    if !unknown {
        assert_full_trace(&adapter, &status, &fixture, 1);
    }
    assert_eq!(status.session.pid.is_some(), unknown, "{projection}");
    assert!(crate::git::executor_reserved(&observation.saved.unwrap()));
    assert_eq!(
        adapter
            .start(fixture.request.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::StateConflict
    );
    assert_eq!(
        adapter.release((&session).into()).unwrap_err().kind,
        ErrorKind::SessionLost,
        "{projection}"
    );
    assert!(
        status
            .failure
            .as_ref()
            .is_some_and(|s| s.contains("unowned native file read callback")),
        "{projection}"
    );
}
#[tokio::test]
#[ignore = "only its env-cleared owning parent enters this synthetic case"]
async fn dispatched_clean_child() {
    dispatched(false).await;
    child_completed("adapter::grok::receipt_tests::dispatched_clean_child");
}
#[tokio::test]
#[ignore = "only its env-cleared owning parent enters this synthetic case"]
async fn dispatched_unknown_child() {
    dispatched(true).await;
    child_completed("adapter::grok::receipt_tests::dispatched_unknown_child");
}
#[tokio::test]
#[ignore = "only its env-cleared owning parent enters this synthetic case"]
async fn completed_resume_child() {
    let fixture = synthetic_fixture("good");
    let adapter = fixture.adapter();
    let schema = json!({"type":"object","properties":{"verdict":{"type":"string","enum":["DENY"]},"reason":{"type":"string"}},"required":["verdict","reason"],"additionalProperties":false});
    let session = fixture.start_structured(&adapter, schema).await.unwrap();
    let first = terminal(&adapter, &session, &fixture).await;
    let projection = fixture.receipt_message(&first);
    receipt_support::assert_state(
        first.session.state,
        SessionState::Exited,
        "completed",
        &projection,
    );
    assert!(adapter.transport_succeeded(&first), "{projection}");
    assert!(first.session.pid.is_none(), "{projection}");
    receipt_support::assert_receipt(&fixture.observation(&first).receipt);
    assert_full_trace(&adapter, &first, &fixture, 1);
    let mut input = fixture.request.input.clone();
    input.version = 2;
    input.payload = "explicit new fixture continuation".into();
    adapter.checkpoint((&session).into(), input).await.unwrap();
    let resumed = fixture
        .resume(&adapter, (&session).into(), 2)
        .await
        .unwrap();
    let second = terminal(&adapter, &resumed, &fixture).await;
    let projection = fixture.receipt_message(&second);
    receipt_support::assert_state(
        second.session.state,
        SessionState::Exited,
        "resumed",
        &projection,
    );
    assert!(adapter.transport_succeeded(&second), "{projection}");
    assert!(second.session.pid.is_none(), "{projection}");
    receipt_support::assert_receipt(&fixture.observation(&second).receipt);
    assert_full_trace(&adapter, &second, &fixture, 2);
    assert_eq!(second.session.native_ref, first.session.native_ref);
    assert!(!fixture.directory.path().join("foreign.txt").exists());
    child_completed("adapter::grok::receipt_tests::completed_resume_child");
}
#[tokio::test]
#[ignore = "only its env-cleared owning parent enters this synthetic case"]
async fn before_spawn_stop_child() {
    let fixture = synthetic_fixture("hang");
    let adapter = fixture.adapter();
    let session = fixture.start(&adapter).await.unwrap();
    let status = adapter.stop((&session).into()).await.unwrap();
    fixture.observe(&status);
    let projection = fixture.receipt_message(&status);
    receipt_support::assert_state(
        status.session.state,
        SessionState::Stopped,
        "before_spawn",
        &projection,
    );
    assert!(status.session.pid.is_none(), "{projection}");
    assert!(!adapter.transport_succeeded(&status), "{projection}");
    let observation = fixture.observation(&status);
    receipt_support::assert_receipt(&observation.receipt);
    let receipt = observation.receipt.unwrap();
    assert_eq!(receipt["owned_process_group_created"], false);
    assert_eq!(receipt["cleanup_state"], "not_attempted");
    assert_eq!(receipt["stderr_drain_state"], "not_started");
    assert_eq!(receipt["cleanup_ok"], true);
    assert_eq!(receipt["output_verified"], true);
    assert_eq!(receipt["ownership_uncertain"], false);
    assert!(
        !observation
            .events
            .iter()
            .any(|event| event["kind"] == "grok.process_spawned"
                && event["data"]["session"] == json!(session.id))
    );
    let entry = adapter.entry(&(&status.session).into()).unwrap();
    let trace = entry.ownership_trace.snapshot(1).expect("trace incomplete");
    assert!(!trace.is_empty());
    assert!(trace.iter().all(|stage| *stage == OwnershipStage::PreSpawn));
    child_completed("adapter::grok::receipt_tests::before_spawn_stop_child");
}
#[tokio::test]
#[ignore = "expected strict-state failure observed only by its env-cleared owning parent"]
async fn state_assertion_child() {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    let fixture = Fixture::new();
    let mut adapter = GrokAdapter::new(
        "grok".into(),
        PathBuf::from("/bin/cat"),
        fixture.store.clone(),
    )
    .unwrap();
    adapter.process_inspection = Some(ProcessInspectionPlan::unknown(
        UnknownObservation::Diagnostics,
    ));
    let session = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &session, &fixture).await;
    let projection = fixture.receipt_message(&status);
    receipt_support::assert_state(
        status.session.state,
        SessionState::Failed,
        "expected_clean_protocol",
        &projection,
    );
}

#[tokio::test]
#[should_panic(expected = "exact positive child did not run one passing test")]
async fn positive_parent_rejects_zero_matched_child_after_owned_cleanup() {
    sanitized("adapter::grok::receipt_tests::nonexistent_child", false).await;
}
