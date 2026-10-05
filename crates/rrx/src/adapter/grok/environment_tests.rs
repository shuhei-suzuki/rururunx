//! Owned env-cleared synthetic consumers. Canary values are invented, never native credentials.
use super::fixture_support::Fixture;
use super::*;
use crate::domain::ProjectState;

pub(super) async fn isolated(name: &str) {
    isolated_with(name, &[], false).await;
}
async fn isolated_with(name: &str, extras: &[(String, String)], omit_ssl: bool) {
    let home = tempfile::tempdir().unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", name, "--ignored", "--nocapture"])
        .env_clear()
        .env("HOME", home.path())
        .env("PATH", "/usr/bin:/bin")
        .env("RRX_INSPECTION_FIXTURE_CHILD", "1")
        .env("GROK_SYNTHETIC_AUTH", "synthetic-native-global")
        .env("XAI_API_KEY", "synthetic-xai-global")
        .env("SSLKEYLOGFILE", "synthetic-keylog-locator")
        .env("NODE_TLS_REJECT_UNAUTHORIZED", "synthetic-tls-control")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .process_group(0);
    if omit_ssl {
        command.env_remove("SSLKEYLOGFILE");
    }
    command.envs(extras.iter().cloned());
    let mut child =
        ProcessGroup::new(command.spawn().unwrap(), Arc::new(AtomicBool::new(false))).unwrap();
    let stdout = tokio::spawn(read_git_output(child.child.stdout.take().unwrap()));
    let stderr = tokio::spawn(read_git_output(child.child.stderr.take().unwrap()));
    let observation_started = std::time::Instant::now();
    let observed = tokio::time::timeout(Duration::from_secs(60), child.observe_exit()).await;
    let observation_elapsed_ms = observation_started.elapsed().as_millis();
    let observation = match &observed {
        Ok(Ok(())) => "exit_observed",
        Ok(Err(_)) => "observation_io_error",
        Err(_) => "watchdog_expired",
    };
    child = cleanup_group(child).await.unwrap();
    let exit = child.reap().await.unwrap();
    let stdout = String::from_utf8(stdout.await.unwrap().unwrap()).unwrap();
    let stderr = String::from_utf8(stderr.await.unwrap().unwrap()).unwrap();
    assert!(
        observed.is_ok() && exit.success(),
        "synthetic environment child failed; observation={observation}; observation_elapsed_ms={observation_elapsed_ms}; reaped_exit={exit}; {stdout}; {stderr}"
    );
    assert_child_completed(&stdout, &stderr, name);
}
async fn terminal(adapter: &GrokAdapter, fixture: &Fixture, session: &Session) -> SessionStatus {
    let mut watch = adapter.subscribe(session.into()).unwrap();
    let result = tokio::time::timeout(Duration::from_secs(15), async {
        while !watch.borrow().terminal() {
            watch.changed().await.unwrap();
        }
        watch.borrow().clone()
    })
    .await
    .unwrap();
    fixture.observe(&result);
    result
}
fn own_refs(fixture: &mut Fixture, names: &[&str]) {
    let mut project = fixture.request.project.clone();
    project.environment_refs = names.iter().map(|name| (*name).to_owned()).collect();
    fixture
        .store
        .lock()
        .unwrap()
        .put_project(&mut project)
        .unwrap();
    fixture.request.project = project;
}
fn foreign(fixture: &Fixture, names: &[&str]) -> (tempfile::TempDir, crate::domain::Project) {
    let directory = tempfile::tempdir().unwrap();
    let mut project = crate::domain::Project::new(
        "synthetic-foreign".into(),
        directory.path().to_path_buf(),
        "synthetic-foreign-identity".into(),
        "main".into(),
    );
    project.environment_refs = names.iter().map(|name| (*name).to_owned()).collect();
    fixture
        .store
        .lock()
        .unwrap()
        .put_project(&mut project)
        .unwrap();
    (directory, project)
}
fn expect_environment(fixture: &Fixture, expected: Value) -> PathBuf {
    let observed = fixture.directory.path().join("environment.json");
    fixture.synthetic(
        "RRX_ENVIRONMENT_OBSERVED",
        observed.to_str().unwrap().into(),
    );
    fixture.synthetic("RRX_ENVIRONMENT_EXPECTED", expected.to_string());
    observed
}
fn assert_environment(observed: &Path) {
    assert!(
        observed.is_file(),
        "synthetic child canary was not written for current attempt"
    );
    let matches: Value = serde_json::from_slice(&std::fs::read(observed).unwrap()).unwrap();
    assert!(!matches.as_object().unwrap().is_empty());
    assert!(
        matches
            .as_object()
            .unwrap()
            .values()
            .all(|value| value == &json!(true)),
        "synthetic environment canary did not reach child"
    );
}
fn assert_not_spawned(fixture: &Fixture, status: &SessionStatus) {
    let observation = fixture.observation(status);
    receipt_support::assert_receipt(&observation.receipt);
    assert_eq!(
        serde_json::to_value(observation.saved.as_ref().unwrap()).unwrap(),
        serde_json::to_value(&status.session).unwrap(),
        "persisted and watched rejection differ"
    );
    assert_eq!(
        observation.attempt.unwrap().input_version,
        fixture.request.input.version
    );
    assert!(
        !observation
            .events
            .iter()
            .any(|event| event["kind"] == "grok.process_spawned"
                && event["data"]["session"] == json!(status.session.id)),
        "environment rejection started native child"
    );
    assert!(
        !observation
            .events
            .iter()
            .any(|event| event["kind"] == "session.saved"
                && event["data"]["id"] == json!(status.session.id)
                && event["data"]["evidence"]["dispatch_intent"]["input_version"].as_u64()
                    == Some(fixture.request.input.version)),
        "environment rejection consumed new prompt"
    );
}
async fn rejected_start(adapter: &GrokAdapter, fixture: &Fixture) -> AdapterError {
    match fixture.start(adapter).await {
        Err(error) => error,
        Ok(session) => {
            let status = terminal(adapter, fixture, &session).await;
            if status.session.pid.is_none() {
                adapter.release((&session).into()).unwrap();
            }
            assert_not_spawned(fixture, &status);
            panic!("environment admission unexpectedly reserved a Session");
        }
    }
}

async fn rejected_resume(
    adapter: &GrokAdapter,
    fixture: &Fixture,
    session: &Session,
) -> AdapterError {
    let prior = adapter.status(session.into()).await.unwrap();
    let lower = receipt_support::watermark(&fixture.store, &fixture.request.scope).unwrap();
    match fixture
        .resume(adapter, session.into(), fixture.request.input.version)
        .await
    {
        Err(error) => {
            let after = adapter.status(session.into()).await.unwrap();
            assert_eq!(
                serde_json::to_value(&prior.session).unwrap(),
                serde_json::to_value(&after.session).unwrap()
            );
            assert!(
                fixture
                    .store
                    .lock()
                    .unwrap()
                    .events(&fixture.request.scope, lower, 100)
                    .unwrap()
                    .iter()
                    .all(|event| event.scope != fixture.request.scope),
                "selection rejection altered owning audit"
            );
            error
        }
        Ok(resumed) => {
            let status = terminal(adapter, fixture, &resumed).await;
            if status.session.pid.is_none() {
                adapter.release((&resumed).into()).unwrap();
            }
            assert_not_spawned(fixture, &status);
            panic!("selection rejection unexpectedly reserved a new native attempt");
        }
    }
}
async fn fresh_checkpoint(
    adapter: &GrokAdapter,
    fixture: &mut Fixture,
    session: &Session,
    version: u64,
) {
    let mut input = fixture.request.input.clone();
    input.version = version;
    input.payload = "explicit fresh selection continuation".into();
    adapter
        .checkpoint(session.into(), input.clone())
        .await
        .unwrap();
    fixture.request.input = input;
}
#[tokio::test]
async fn initial_and_resume_selection_keep_invalid_and_live_ownership_guards() {
    isolated("adapter::grok::environment_tests::selection_child").await;
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn selection_child() {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    for names in [
        vec!["INVALID-NAME"],
        vec!["LANG", "LANG"],
        vec!["HOME"],
        vec!["GIT_DIR"],
        vec!["LANG", "NODE_TLS_REJECT_UNAUTHORIZED"],
    ] {
        let mut fixture = Fixture::new();
        own_refs(&mut fixture, &names);
        let adapter = fixture.adapter();
        let error = rejected_start(&adapter, &fixture).await;
        assert_eq!(error.kind, ErrorKind::InvalidConfiguration);
        assert_eq!(
            error.message,
            if names.contains(&"NODE_TLS_REJECT_UNAUTHORIZED") {
                "native control references are not scoped environment values"
            } else {
                "owning environment references invalid"
            }
        );
        assert!(
            fixture
                .store
                .lock()
                .unwrap()
                .records(&fixture.request.scope, RecordKind::Session)
                .unwrap()
                .is_empty()
        );
    }
    let mut fixture = Fixture::new();
    fixture
        .request
        .environment
        .insert("GIT_CONFIG_GLOBAL".into(), "synthetic".into());
    let adapter = fixture.adapter();
    let error = rejected_start(&adapter, &fixture).await;
    assert_eq!(error.kind, ErrorKind::InvalidInput);
    assert_eq!(error.message, "invalid/leaking Git environment");
    assert!(
        fixture
            .store
            .lock()
            .unwrap()
            .records(&fixture.request.scope, RecordKind::Session)
            .unwrap()
            .is_empty()
    );
    for kind in 0..5 {
        let mut fixture = Fixture::new();
        let (_foreign_dir, mut other) = foreign(&fixture, &[]);
        let adapter = fixture.adapter();
        let first = fixture.start(&adapter).await.unwrap();
        let status = terminal(&adapter, &fixture, &first).await;
        assert!(
            adapter.transport_succeeded(&status),
            "selection control failed; {:?}; {}",
            status.failure,
            fixture.receipt_message(&status)
        );
        if kind == 4 {
            own_refs(&mut fixture, &["NODE_TLS_REJECT_UNAUTHORIZED"]);
        } else {
            other.environment_refs = if kind == 3 {
                vec!["INVALID-NAME".into(), "GROK_SYNTHETIC_AUTH".into()]
            } else {
                vec!["GROK_SYNTHETIC_AUTH".into()]
            };
            if kind == 1 {
                other.state = ProjectState::Blocked;
                other.blocked_reason = Some("synthetic".into());
            }
            if kind == 2 {
                other.state = ProjectState::Removed;
            }
            fixture
                .store
                .lock()
                .unwrap()
                .put_project(&mut other)
                .unwrap();
        }
        fresh_checkpoint(&adapter, &mut fixture, &first, 2).await;
        let error = rejected_resume(&adapter, &fixture, &first).await;
        assert_eq!(error.kind, ErrorKind::InvalidConfiguration);
        assert_eq!(
            error.message,
            if kind == 4 {
                "native control references are not scoped environment values"
            } else {
                "native environment authority unavailable"
            }
        );
        adapter.release((&first).into()).unwrap();
    }
    child_completed("adapter::grok::environment_tests::selection_child");
}
#[tokio::test]
async fn native_constructor_enforces_actual_baseline_count_and_name_bounds() {
    for (kind, count) in [
        ("count_ok", 509),
        ("count_bad", 510),
        ("name_ok", 0),
        ("name_bad", 0),
    ] {
        let mut extras = (0..count)
            .map(|i| (format!("GROK_BOUND_{i}"), "synthetic-bound".into()))
            .collect::<Vec<_>>();
        extras.push(("RRX_CONSTRUCTOR_BOUNDARY".into(), kind.into()));
        if kind.starts_with("name_") {
            extras.push((
                format!(
                    "GROK_{}",
                    "X".repeat(if kind == "name_ok" { 251 } else { 252 })
                ),
                "synthetic-bound".into(),
            ));
        }
        isolated_with(
            "adapter::grok::environment_tests::constructor_child",
            &extras,
            false,
        )
        .await;
    }
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn constructor_child() {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    let fixture = Fixture::new();
    let result = GrokAdapter::new(
        "grok".into(),
        fixture.executable.clone(),
        fixture.store.clone(),
    );
    match std::env::var("RRX_CONSTRUCTOR_BOUNDARY").unwrap().as_str() {
        "count_ok" | "name_ok" => assert!(result.is_ok(), "supported constructor bound rejected"),
        "count_bad" | "name_bad" => {
            let error = result
                .err()
                .expect("unsupported constructor bound accepted");
            assert_eq!(error.kind, ErrorKind::InvalidConfiguration);
            assert_eq!(error.message, "native environment authority unavailable");
        }
        _ => unreachable!(),
    }
    assert!(
        fixture
            .store
            .lock()
            .unwrap()
            .records(&fixture.request.scope, RecordKind::Session)
            .unwrap()
            .is_empty()
    );
    child_completed("adapter::grok::environment_tests::constructor_child");
}
#[tokio::test]
async fn absent_native_baseline_cannot_be_introduced_by_declared_caller() {
    isolated_with(
        "adapter::grok::environment_tests::absent_baseline_child",
        &[],
        true,
    )
    .await;
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn absent_baseline_child() {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    let mut fixture = Fixture::new();
    own_refs(&mut fixture, &["SSLKEYLOGFILE"]);
    let observed = expect_environment(
        &fixture,
        json!({"SSLKEYLOGFILE":null,"GROK_SYNTHETIC_AUTH":"synthetic-native-global"}),
    );
    let adapter = fixture.adapter();
    assert!(
        adapter
            .environment_candidates(fixture.request.project.id)
            .unwrap()
            .is_empty()
    );
    let first = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &fixture, &first).await;
    assert!(adapter.transport_succeeded(&status));
    assert_environment(&observed);
    adapter.release((&first).into()).unwrap();
    fixture
        .request
        .environment
        .insert("SSLKEYLOGFILE".into(), "synthetic-introduction".into());
    assert_eq!(
        rejected_start(&adapter, &fixture).await.kind,
        ErrorKind::InvalidConfiguration
    );
    child_completed("adapter::grok::environment_tests::absent_baseline_child");
}

#[tokio::test]
async fn environment_initial_selection_drives_real_canary_and_rejection_consumers() {
    isolated("adapter::grok::environment_tests::initial_child").await;
}
#[tokio::test]
async fn live_reference_change_rejects_actual_start_before_spawn() {
    isolated("adapter::grok::environment_tests::start_admission_child").await;
}
#[tokio::test]
async fn live_reference_change_rejects_actual_resume_before_spawn() {
    isolated("adapter::grok::environment_tests::resume_admission_child").await;
}
#[tokio::test]
async fn actual_stop_after_admission_never_spawns_and_keeps_snapshots_coherent() {
    isolated("adapter::grok::environment_tests::stop_admission_child").await;
}
#[tokio::test]
async fn owning_reference_change_after_snapshot_rejects_start() {
    isolated("adapter::grok::environment_tests::own_start_change_child").await;
}
#[tokio::test]
async fn owning_reference_change_after_snapshot_rejects_resume() {
    isolated("adapter::grok::environment_tests::own_resume_change_child").await;
}

async fn owning_change(resume: bool, name: &str) {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    let mut fixture = Fixture::new();
    own_refs(&mut fixture, &["LANG"]);
    fixture
        .request
        .environment
        .insert("LANG".into(), "synthetic-own-locale".into());
    let database = fixture.directory.path().join("state.db");
    let project_id = fixture.request.scope.project_id;
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counted = calls.clone();
    let mut adapter = fixture.adapter();
    adapter.before_environment_admission = Some(Arc::new(move |_entry| {
        let change = counted.fetch_add(1, Ordering::SeqCst) == usize::from(resume);
        let database = database.clone();
        Box::pin(async move {
            if change {
                let mut store = Store::open(&database).unwrap();
                let mut project = store.project(project_id).unwrap().unwrap();
                project.environment_refs.clear();
                store.put_project(&mut project).unwrap();
            }
            Ok(())
        })
    }));
    let first = fixture.start(&adapter).await.unwrap();
    let mut status = terminal(&adapter, &fixture, &first).await;
    let previous = status.session.recovery.clone();
    if resume {
        assert!(
            adapter.transport_succeeded(&status),
            "{:?}; {}",
            status.failure,
            fixture.receipt_message(&status)
        );
        let mut input = fixture.request.input.clone();
        input.version = 2;
        input.payload = "explicit fresh owning continuation".into();
        adapter
            .checkpoint((&first).into(), input.clone())
            .await
            .unwrap();
        fixture.request.input = input;
        let second = fixture.resume(&adapter, (&first).into(), 2).await.unwrap();
        status = terminal(&adapter, &fixture, &second).await;
        for key in [
            "input_version",
            "dispatch_intent",
            "prompt_id",
            "dispatch_state",
        ] {
            assert_eq!(status.session.recovery[key], previous[key]);
        }
    }
    assert_not_spawned(&fixture, &status);
    assert!(
        status
            .failure
            .as_deref()
            .unwrap()
            .starts_with("StateConflict:")
    );
    assert_eq!(status.session.state, SessionState::Failed);
    assert!(status.session.pid.is_none());
    assert_eq!(calls.load(Ordering::SeqCst), 1 + usize::from(resume));
    adapter.release((&first).into()).unwrap();
    child_completed(name);
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn own_start_change_child() {
    owning_change(
        false,
        "adapter::grok::environment_tests::own_start_change_child",
    )
    .await;
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn own_resume_change_child() {
    owning_change(
        true,
        "adapter::grok::environment_tests::own_resume_change_child",
    )
    .await;
}

#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn initial_child() {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    let mut positive = Fixture::new();
    own_refs(&mut positive, &["LANG"]);
    let (_control_dir, _) = foreign(&positive, &["NODE_TLS_REJECT_UNAUTHORIZED"]);
    positive
        .request
        .environment
        .insert("LANG".into(), "synthetic-own-locale".into());
    positive.mode("good");
    assert_eq!(positive.synthetic_value("RRX_MODE"), "good");
    assert_eq!(
        PathBuf::from(positive.synthetic_value("RRX_DATABASE")),
        positive.directory.path().join("state.db")
    );
    let observed = expect_environment(
        &positive,
        json!({
            "LANG":"synthetic-own-locale", "GROK_SYNTHETIC_AUTH":"synthetic-native-global",
            "RRX_CALLER_FORBIDDEN":null, "NODE_TLS_REJECT_UNAUTHORIZED":"synthetic-tls-control"
        }),
    );
    let adapter = positive.adapter();
    let launched = positive.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &positive, &launched).await;
    assert!(
        adapter.transport_succeeded(&status),
        "{:?}; {}",
        status.failure,
        positive.receipt_message(&status)
    );
    assert_environment(&observed);
    assert_eq!(
        std::fs::read_to_string(positive.request.worktree.join("result.txt")).unwrap(),
        "owned edit\n"
    );
    adapter.release((&launched).into()).unwrap();
    own_refs(&mut positive, &["LANG", "NODE_TLS_REJECT_UNAUTHORIZED"]);
    assert_eq!(
        rejected_start(&adapter, &positive).await.kind,
        ErrorKind::InvalidConfiguration
    );

    let mut denied = Fixture::new();
    assert!(
        denied.request.environment.is_empty(),
        "native request fixture metadata was not migrated"
    );
    let (_foreign_dir, mut other) = foreign(&denied, &["LANG"]);
    denied
        .request
        .environment
        .insert("LANG".into(), "synthetic-foreign-locale".into());
    let adapter = denied.adapter();
    assert_eq!(
        rejected_start(&adapter, &denied).await.kind,
        ErrorKind::InvalidConfiguration
    );
    denied.request.environment.clear();
    other.environment_refs = vec!["GROK_SYNTHETIC_AUTH".into()];
    denied
        .store
        .lock()
        .unwrap()
        .put_project(&mut other)
        .unwrap();
    assert_eq!(
        rejected_start(&adapter, &denied).await.kind,
        ErrorKind::InvalidConfiguration
    );
    other.state = ProjectState::Blocked;
    other.blocked_reason = Some("synthetic".into());
    denied
        .store
        .lock()
        .unwrap()
        .put_project(&mut other)
        .unwrap();
    assert_eq!(
        rejected_start(&adapter, &denied).await.kind,
        ErrorKind::InvalidConfiguration
    );
    assert_eq!(
        adapter.environment_candidates(other.id).unwrap(),
        BTreeSet::from(["GROK_SYNTHETIC_AUTH".into()])
    );
    other.state = ProjectState::Removed;
    denied
        .store
        .lock()
        .unwrap()
        .put_project(&mut other)
        .unwrap();
    assert_eq!(
        rejected_start(&adapter, &denied).await.kind,
        ErrorKind::InvalidConfiguration
    );
    assert!(
        denied
            .store
            .lock()
            .unwrap()
            .records(&denied.request.scope, RecordKind::Session)
            .unwrap()
            .is_empty()
    );
    assert!(
        adapter
            .environment_candidates(denied.request.project.id)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        adapter.environment_candidates(other.id).unwrap(),
        BTreeSet::from(["GROK_SYNTHETIC_AUTH".into()])
    );
    own_refs(&mut denied, &["GROK_SYNTHETIC_AUTH"]);
    let observed = expect_environment(
        &denied,
        json!({"GROK_SYNTHETIC_AUTH":"synthetic-native-global","XAI_API_KEY":"synthetic-xai-global","SSLKEYLOGFILE":"synthetic-keylog-locator"}),
    );
    let session = denied.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &denied, &session).await;
    assert!(
        adapter.transport_succeeded(&status),
        "{:?}; {}",
        status.failure,
        denied.receipt_message(&status)
    );
    assert_environment(&observed);
    adapter.release((&session).into()).unwrap();
    for key in [
        "RRX_CALLER_FORBIDDEN",
        "HOME",
        "NODE_TLS_REJECT_UNAUTHORIZED",
        "UNSUPPORTED_REFERENCE",
    ] {
        denied
            .request
            .environment
            .insert(key.into(), "synthetic".into());
        let error = rejected_start(&adapter, &denied).await;
        assert_eq!(error.kind, ErrorKind::InvalidConfiguration);
        if key == "UNSUPPORTED_REFERENCE" {
            assert_eq!(
                error.message,
                "native environment value cannot replace intentional runtime authority"
            );
        }
        denied.request.environment.clear();
    }
    let mut decision = Fixture::new();
    decision.request.role = SessionRole::Consultant;
    own_refs(&mut decision, &["LANG"]);
    decision
        .request
        .environment
        .insert("LANG".into(), "synthetic-own-locale".into());
    let observed = expect_environment(
        &decision,
        json!({"LANG":"synthetic-own-locale","GROK_SYNTHETIC_AUTH":"synthetic-native-global"}),
    );
    let adapter = decision.adapter();
    let session = decision.start_structured(&adapter, json!({"type":"object","properties":{"verdict":{"type":"string","enum":["DENY"]},"reason":{"type":"string"}},"required":["verdict","reason"],"additionalProperties":false})).await.unwrap();
    let status = terminal(&adapter, &decision, &session).await;
    receipt_support::assert_state(
        status.session.state,
        SessionState::Exited,
        "scoped decision environment",
        &decision.receipt_message(&status),
    );
    assert!(adapter.transport_succeeded(&status));
    assert_eq!(
        serde_json::from_slice::<Value>(&status.stdout).unwrap()["verdict"],
        "DENY"
    );
    assert_environment(&observed);
    adapter.release((&session).into()).unwrap();
    child_completed("adapter::grok::environment_tests::initial_child");
}

#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn start_admission_child() {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    // Relevant reference is created through a second SQLite connection only AFTER
    // initial selection and all Git/filesystem/profile preflight, before admission.
    let fixture = Fixture::new();
    let (_foreign_dir, mut other) = foreign(&fixture, &[]);
    let database = fixture.directory.path().join("state.db");
    let mut adapter = fixture.adapter();
    other.environment_refs = vec!["GROK_SYNTHETIC_AUTH".into()];
    adapter.before_environment_admission = Some(Arc::new(move |_entry| {
        let database = database.clone();
        let mut other = other.clone();
        Box::pin(async move {
            Store::open(&database)
                .unwrap()
                .put_project(&mut other)
                .unwrap();
            Ok(())
        })
    }));
    let session = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &fixture, &session).await;
    assert_not_spawned(&fixture, &status);
    assert_eq!(
        status.failure.as_deref(),
        Some("InvalidConfiguration: native environment authority unavailable")
    );
    assert_eq!(status.session.state, SessionState::Failed);
    assert!(status.session.pid.is_none());
    assert_not_spawned(&fixture, &status);
    adapter.release((&session).into()).unwrap();

    child_completed("adapter::grok::environment_tests::start_admission_child");
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn resume_admission_child() {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    let mut resumed = Fixture::new();
    let (_foreign_dir, mut other) = foreign(&resumed, &[]);
    let mut adapter = resumed.adapter();
    let database = resumed.directory.path().join("state.db");
    other.environment_refs = vec!["GROK_SYNTHETIC_AUTH".into()];
    let invocations = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = invocations.clone();
    adapter.before_environment_admission = Some(Arc::new(move |_entry| {
        let database = database.clone();
        let mut other = other.clone();
        let change = count.fetch_add(1, Ordering::SeqCst) == 1;
        Box::pin(async move {
            if change {
                Store::open(&database)
                    .unwrap()
                    .put_project(&mut other)
                    .unwrap();
            }
            Ok(())
        })
    }));
    let first = resumed.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &resumed, &first).await;
    assert!(
        adapter.transport_succeeded(&status),
        "{:?}; {}",
        status.failure,
        resumed.receipt_message(&status)
    );
    let previous = status.session.recovery.clone();
    let mut input = resumed.request.input.clone();
    input.version = 2;
    input.payload = "explicit fresh continuation".into();
    adapter
        .checkpoint((&first).into(), input.clone())
        .await
        .unwrap();
    resumed.request.input = input;
    let second = resumed.resume(&adapter, (&first).into(), 2).await.unwrap();
    let status = terminal(&adapter, &resumed, &second).await;
    assert_not_spawned(&resumed, &status);
    assert_eq!(
        status.failure.as_deref(),
        Some("InvalidConfiguration: native environment authority unavailable")
    );
    assert_eq!(status.session.state, SessionState::Failed);
    assert_eq!(
        status.session.recovery["input_version"],
        previous["input_version"]
    );
    for key in ["dispatch_intent", "prompt_id", "dispatch_state"] {
        assert_eq!(status.session.recovery[key], previous[key]);
    }
    assert_eq!(invocations.load(Ordering::SeqCst), 2);
    adapter.release((&second).into()).unwrap();

    child_completed("adapter::grok::environment_tests::resume_admission_child");
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn stop_admission_child() {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    let fixture = Fixture::new();
    let mut adapter = fixture.adapter();
    let adapter_slot: Arc<Mutex<Option<std::sync::Weak<GrokAdapter>>>> = Arc::new(Mutex::new(None));
    let stop_result = Arc::new(Mutex::new(None));
    let slot = adapter_slot.clone();
    let result = stop_result.clone();
    adapter.after_environment_admission = Some(Arc::new(move |entry| {
        let adapter = slot.lock().unwrap().as_ref().unwrap().upgrade().unwrap();
        let result = result.clone();
        Box::pin(async move {
            let reference = SessionRef::from(&entry.status.borrow().session);
            let stopped = tokio::spawn(async move { adapter.stop(reference).await });
            *result.lock().unwrap() = Some(stopped);
            while !entry.stopping.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }
            Ok(())
        })
    }));
    let adapter = Arc::new(adapter);
    *adapter_slot.lock().unwrap() = Some(Arc::downgrade(&adapter));
    let session = fixture.start(adapter.as_ref()).await.unwrap();
    let status = terminal(&adapter, &fixture, &session).await;
    assert_eq!(status.session.state, SessionState::Stopped);
    let stopped = stop_result.lock().unwrap().take().unwrap();
    let stopped = stopped.await.unwrap().unwrap();
    assert_eq!(stopped.session.state, SessionState::Stopped);
    assert_not_spawned(&fixture, &status);
    let (saved, version) = fixture
        .store
        .lock()
        .unwrap()
        .session(session.id)
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::to_value(saved).unwrap(),
        serde_json::to_value(&status.session).unwrap()
    );
    assert!(
        version >= 3,
        "reservation, admission and terminal publications did not remain coherent"
    );
    adapter.release((&session).into()).unwrap();
    child_completed("adapter::grok::environment_tests::stop_admission_child");
}

#[test]
fn control_and_native_whitelist_are_separate_finite_policies() {
    use crate::project::{environment_name_forbidden, environment_name_valid};
    assert_eq!(
        environment::CONTROL_ADDITIONS,
        &["NODE_TLS_REJECT_UNAUTHORIZED"]
    );
    assert_eq!(
        BASELINE_PREFIXES,
        &["GROK_", "XAI_", "DYLD_", "LD_", "NODE_", "BUN_", "OPENSSL_"]
    );
    assert_eq!(
        BASELINE_NAMES,
        &[
            "SSLKEYLOGFILE",
            "BASH_ENV",
            "ENV",
            "SHELL",
            "ZDOTDIR",
            "HOME",
            "PATH",
            "TMPDIR",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_STATE_HOME",
            "XDG_CACHE_HOME",
            "NODE_OPTIONS",
            "SSL_CERT_FILE",
            "SSL_CERT_DIR",
            "REQUESTS_CA_BUNDLE",
            "CURL_CA_BUNDLE",
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "NO_PROXY",
            "http_proxy",
            "https_proxy",
            "all_proxy",
            "no_proxy",
        ]
    );
    for key in BASELINE_NAMES {
        assert_eq!(environment_name_forbidden(key), *key != "SSLKEYLOGFILE");
    }
    let prefixed = BASELINE_PREFIXES
        .iter()
        .map(|prefix| format!("{prefix}SYNTHETIC"));
    for key in BASELINE_NAMES
        .iter()
        .map(|key| (*key).to_owned())
        .chain(prefixed)
        .chain([
            "GROK_CONFIG_DIR".into(),
            "GROK_API_KEY".into(),
            "OPENSSL_MODULES".into(),
            "NODE_TLS_REJECT_UNAUTHORIZED".into(),
            "GROK_INVALID-NAME".into(),
        ])
    {
        assert!(baseline_key(&key));
        assert_eq!(
            environment::control(&key),
            !environment_name_valid(&key)
                || environment_name_forbidden(&key)
                || environment::CONTROL_ADDITIONS.contains(&key.as_str())
        );
    }
    for name in [
        "XAI_API_KEY",
        "GROK_FUTURE_REFERENCE",
        "OPENSSL_CONF",
        "BUN_OPTIONS",
        "SSLKEYLOGFILE",
    ] {
        assert!(baseline_key(name));
        assert!(!environment::control(name));
    }
    for name in [
        "HOME",
        "PATH",
        "TMPDIR",
        "NODE_OPTIONS",
        "NODE_PATH",
        "NODE_EXTRA_CA_CERTS",
        "LD_PRELOAD",
        "DYLD_INSERT_LIBRARIES",
        "https_proxy",
    ] {
        assert!(environment::control(name));
    }
    assert!(environment::control("NODE_TLS_REJECT_UNAUTHORIZED"));
    assert!(!ordinary_key("RRX_SYNTHETIC"));
    assert!(!baseline_key("UNRELATED_SYNTHETIC"));
}

#[tokio::test]
async fn unrelated_foreign_changes_do_not_revoke_start() {
    isolated("adapter::grok::environment_tests::irrelevant_start_child").await;
}
#[tokio::test]
async fn unrelated_foreign_changes_do_not_revoke_checkpoint() {
    isolated("adapter::grok::environment_tests::irrelevant_checkpoint_child").await;
}
#[tokio::test]
async fn unrelated_foreign_changes_do_not_revoke_resume() {
    isolated("adapter::grok::environment_tests::irrelevant_resume_child").await;
}
fn mutation_hook(
    database: PathBuf,
    foreign: crate::domain::Project,
    kind: usize,
    enabled: Arc<AtomicBool>,
) -> EnvironmentHook {
    Arc::new(move |_entry| {
        let database = database.clone();
        let mut foreign = foreign.clone();
        let apply = enabled.swap(false, Ordering::SeqCst);
        Box::pin(async move {
            if apply {
                let mut store = Store::open(&database).unwrap();
                match kind {
                    0 => {}
                    1 => {
                        foreign.id = crate::domain::ProjectId::new();
                        foreign.version = 0;
                        foreign.repository_identity =
                            format!("synthetic-new-identity-{}", foreign.id);
                        foreign.root = foreign
                            .root
                            .with_file_name(format!("synthetic-new-{}", foreign.id));
                        foreign.worktree_root = foreign.root.join("worktree");
                        store.put_project(&mut foreign).unwrap();
                    }
                    2 => {
                        foreign.environment_refs = vec!["FOREIGN_ABSENT_NATIVE_NAME".into()];
                        store.put_project(&mut foreign).unwrap();
                    }
                    3 => {
                        foreign.environment_refs =
                            vec!["HOME".into(), "NODE_TLS_REJECT_UNAUTHORIZED".into()];
                        store.put_project(&mut foreign).unwrap();
                    }
                    4 => {
                        foreign.state = ProjectState::Blocked;
                        foreign.blocked_reason = Some("synthetic".into());
                        store.put_project(&mut foreign).unwrap();
                    }
                    5 => {
                        foreign.state = ProjectState::Removed;
                        store.put_project(&mut foreign).unwrap();
                    }
                    6 => {
                        foreign.name = "synthetic-irrelevant-rename".into();
                        store.put_project(&mut foreign).unwrap();
                    }
                    7 => {
                        foreign.environment_refs.clear();
                        store.put_project(&mut foreign).unwrap();
                    }
                    8 => {
                        let connection = rusqlite::Connection::open(&database).unwrap();
                        connection.execute("UPDATE projects SET body=json_set(body,'$.root',json('null')) WHERE id=?1",[foreign.id.to_string()]).unwrap();
                    }
                    _ => unreachable!(),
                }
            }
            Ok(())
        })
    })
}
async fn irrelevant(boundary: usize, name: &str) {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    assert!(boundary < 3);
    let mut control = None;
    let mut checkpoint_control = None;
    for kind in 0..9 {
        eprintln!("grok_environment_progress boundary={boundary} case={kind} stage=fixture");
        let mut fixture = Fixture::new();
        let initial_refs: &[&str] = if kind == 7 {
            &["FOREIGN_ABSENT_NATIVE_NAME"]
        } else {
            &[]
        };
        let (_foreign_dir, foreign) = foreign(&fixture, initial_refs);
        let active = Arc::new(AtomicBool::new(boundary == 0));
        let hook = mutation_hook(
            fixture.directory.path().join("state.db"),
            foreign,
            kind,
            active.clone(),
        );
        let mut adapter = fixture.adapter();
        if boundary == 1 {
            adapter.checkpoint_environment = Some(hook);
        } else {
            adapter.before_environment_admission = Some(hook);
        }
        eprintln!("grok_environment_progress boundary={boundary} case={kind} stage=start");
        let first = fixture.start(&adapter).await.unwrap();
        eprintln!("grok_environment_progress boundary={boundary} case={kind} stage=terminal");
        let mut status = terminal(&adapter, &fixture, &first).await;
        assert!(
            adapter.transport_succeeded(&status),
            "irrelevant start rejected kind{kind}; {:?}; {}",
            status.failure,
            fixture.receipt_message(&status)
        );
        if boundary != 0 {
            let mut input = fixture.request.input.clone();
            input.version = 2;
            input.payload = "explicit fresh continuation".into();
            active.store(boundary == 1, Ordering::SeqCst);
            let lower = receipt_support::watermark(&fixture.store, &fixture.request.scope).unwrap();
            eprintln!("grok_environment_progress boundary={boundary} case={kind} stage=checkpoint");
            adapter
                .checkpoint((&first).into(), input.clone())
                .await
                .unwrap();
            let checkpoint_events = fixture
                .store
                .lock()
                .unwrap()
                .events(&fixture.request.scope, lower, 100)
                .unwrap();
            let checkpoint_kinds = checkpoint_events
                .iter()
                .filter(|event| event.scope == fixture.request.scope)
                .map(|event| event.kind.clone())
                .collect::<Vec<_>>();
            assert!(checkpoint_kinds.iter().any(|kind| kind == "session.saved"));
            if let Some(control) = &checkpoint_control {
                assert_eq!(
                    &checkpoint_kinds, control,
                    "unrelated metadata changed owning checkpoint event kinds/counts"
                );
            } else {
                checkpoint_control = Some(checkpoint_kinds);
            }
            active.store(boundary == 2, Ordering::SeqCst);
            fixture.request.input = input;
            eprintln!("grok_environment_progress boundary={boundary} case={kind} stage=resume");
            let second = fixture.resume(&adapter, (&first).into(), 2).await.unwrap();
            eprintln!(
                "grok_environment_progress boundary={boundary} case={kind} stage=resumed_terminal"
            );
            status = terminal(&adapter, &fixture, &second).await;
            assert!(
                adapter.transport_succeeded(&status),
                "irrelevant resume rejected kind{kind}; {:?}; {}",
                status.failure,
                fixture.receipt_message(&status)
            );
        }
        let events = fixture.observation(&status).events;
        let kinds = events
            .iter()
            .map(|event| event["kind"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        if let Some(control) = &control {
            assert_eq!(
                &kinds, control,
                "irrelevant foreign metadata correlated with owning event kinds/counts"
            );
        } else {
            control = Some(kinds);
        }
        eprintln!("grok_environment_progress boundary={boundary} case={kind} stage=release");
        adapter.release((&first).into()).unwrap();
        eprintln!("grok_environment_progress boundary={boundary} case={kind} stage=finished");
    }
    child_completed(name);
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn irrelevant_start_child() {
    irrelevant(
        0,
        "adapter::grok::environment_tests::irrelevant_start_child",
    )
    .await;
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn irrelevant_checkpoint_child() {
    irrelevant(
        1,
        "adapter::grok::environment_tests::irrelevant_checkpoint_child",
    )
    .await;
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn irrelevant_resume_child() {
    irrelevant(
        2,
        "adapter::grok::environment_tests::irrelevant_resume_child",
    )
    .await;
}

#[tokio::test]
async fn native_references_and_ordinary_callers_keep_exact_scoped_alternatives() {
    isolated("adapter::grok::environment_tests::reference_child").await;
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn reference_child() {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    for (key, value) in [
        ("XAI_API_KEY", "synthetic-xai-global"),
        ("SSLKEYLOGFILE", "synthetic-keylog-locator"),
    ] {
        let mut fixture = Fixture::new();
        let (_foreign_dir, _foreign) = foreign(&fixture, &[key]);
        own_refs(&mut fixture, &[key]);
        fixture.request.environment.insert(key.into(), value.into());
        let observed = expect_environment(&fixture, json!({key:value}));
        let adapter = fixture.adapter();
        let first = fixture.start(&adapter).await.unwrap();
        let status = terminal(&adapter, &fixture, &first).await;
        assert!(
            adapter.transport_succeeded(&status),
            "own/shared native reference rejected; {:?}; {}",
            status.failure,
            fixture.receipt_message(&status)
        );
        assert_environment(&observed);
        adapter.release((&first).into()).unwrap();
        fixture.request.environment.clear();
        std::fs::remove_file(&observed).unwrap(); // Prior attempt cannot satisfy this child's proof.
        let retained = fixture.start(&adapter).await.unwrap();
        let status = terminal(&adapter, &fixture, &retained).await;
        assert_environment(&observed);
        assert!(
            adapter.transport_succeeded(&status),
            "retained owning native baseline failed"
        );
        assert_environment(&observed);
        adapter.release((&retained).into()).unwrap();
        fixture
            .request
            .environment
            .insert(key.into(), "synthetic-replacement".into());
        assert_eq!(
            rejected_start(&adapter, &fixture).await.kind,
            ErrorKind::InvalidConfiguration
        );
        fixture.request.environment.clear();
        own_refs(&mut fixture, &[]);
        let error = rejected_start(&adapter, &fixture).await;
        assert_eq!(error.kind, ErrorKind::InvalidConfiguration);
        assert_eq!(error.message, "native environment authority unavailable");
    }
    let mut fixture = Fixture::new();
    let (_foreign_dir, mut other) = foreign(&fixture, &["TZ"]);
    fixture
        .request
        .environment
        .insert("TZ".into(), "synthetic-own-tz".into());
    let adapter = fixture.adapter();
    let foreign_error = rejected_start(&adapter, &fixture).await;
    other.environment_refs = vec![];
    fixture
        .store
        .lock()
        .unwrap()
        .put_project(&mut other)
        .unwrap();
    let undeclared_error = rejected_start(&adapter, &fixture).await;
    assert_eq!(
        (foreign_error.kind, foreign_error.message),
        (undeclared_error.kind, undeclared_error.message),
        "foreign inventory became an error side channel"
    );
    own_refs(&mut fixture, &["TZ"]);
    let observed = expect_environment(
        &fixture,
        json!({"TZ":"synthetic-own-tz", "GROK_SYNTHETIC_AUTH":"synthetic-native-global"}),
    );
    let first = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &fixture, &first).await;
    assert!(
        adapter.transport_succeeded(&status),
        "own ordinary alternative rejected; {:?}; {}",
        status.failure,
        fixture.receipt_message(&status)
    );
    adapter.release((&first).into()).unwrap();
    assert_environment(&observed);
    // A declared unsupported provider key is harmless until a caller tries passing it.
    fixture.request.environment.clear();
    own_refs(&mut fixture, &["OTHER_PROVIDER_REFERENCE"]);
    expect_environment(
        &fixture,
        json!({"GROK_SYNTHETIC_AUTH":"synthetic-native-global","TZ":null}),
    );
    let first = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &fixture, &first).await;
    assert!(
        adapter.transport_succeeded(&status),
        "unused owning reference changed provider authority"
    );
    adapter.release((&first).into()).unwrap();
    fixture.request.environment.insert(
        "OTHER_PROVIDER_REFERENCE".into(),
        "synthetic-unsupported".into(),
    );
    let denied = rejected_start(&adapter, &fixture).await;
    assert_eq!(denied.kind, ErrorKind::InvalidConfiguration);
    assert_eq!(
        denied.message,
        "native environment value cannot replace intentional runtime authority"
    );
    fixture.request.environment.clear();
    // Mixed registry-invalid strings still contribute their valid retained reference.
    other.environment_refs = vec!["INVALID-NAME".into(), "GROK_SYNTHETIC_AUTH".into()];
    fixture
        .store
        .lock()
        .unwrap()
        .put_project(&mut other)
        .unwrap();
    assert_eq!(
        rejected_start(&adapter, &fixture).await.kind,
        ErrorKind::InvalidConfiguration
    );
    let connection = rusqlite::Connection::open(fixture.directory.path().join("state.db")).unwrap();
    let mut oversized = vec!["INVALID-NAME".to_owned(); 3000];
    oversized.push("GROK_SYNTHETIC_AUTH".into());
    for projection in [json!(oversized), json!({}), json!([{}])] {
        connection
            .execute(
                "UPDATE projects SET body=json_set(body,'$.environment_refs',json(?2)) WHERE id=?1",
                [other.id.to_string(), projection.to_string()],
            )
            .unwrap();
        let error = rejected_start(&adapter, &fixture).await;
        assert_eq!(error.kind, ErrorKind::InvalidConfiguration);
        assert_eq!(error.message, "native environment authority unavailable");
    }
    child_completed("adapter::grok::environment_tests::reference_child");
}

#[tokio::test]
async fn owning_reference_refresh_and_existing_consultant_reservation_guards_remain_required() {
    isolated("adapter::grok::environment_tests::ownership_child").await;
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn ownership_child() {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    let mut fixture = Fixture::new();
    own_refs(&mut fixture, &["TZ"]);
    fixture
        .request
        .environment
        .insert("TZ".into(), "synthetic-own-tz".into());
    let observed = expect_environment(
        &fixture,
        json!({"TZ":"synthetic-own-tz","GROK_SYNTHETIC_AUTH":"synthetic-native-global"}),
    );
    let adapter = fixture.adapter();
    let first = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &fixture, &first).await;
    assert_environment(&observed);
    assert!(
        adapter.transport_succeeded(&status),
        "initial owning continuation control failed"
    );
    own_refs(&mut fixture, &[]);
    let mut input = fixture.request.input.clone();
    input.version = 2;
    input.payload = "explicit fresh continuation".into();
    adapter
        .checkpoint((&first).into(), input.clone())
        .await
        .unwrap();
    let prior = adapter.status((&first).into()).await.unwrap();
    fixture.request.input = input.clone();
    let denied = rejected_resume(&adapter, &fixture, &first).await;
    assert_eq!(denied.kind, ErrorKind::InvalidConfiguration);
    let unchanged = adapter.status((&first).into()).await.unwrap();
    assert_eq!(
        serde_json::to_value(&prior.session).unwrap(),
        serde_json::to_value(&unchanged.session).unwrap()
    );
    own_refs(&mut fixture, &["TZ"]);
    // A metadata mutation after checkpoint must remain the original stale-owner guard.
    assert_eq!(
        adapter.resume((&first).into()).await.unwrap_err().kind,
        ErrorKind::StateConflict
    );
    input.version = 3;
    adapter
        .checkpoint((&first).into(), input.clone())
        .await
        .unwrap();
    fixture.request.input = input;
    std::fs::remove_file(&observed).unwrap(); // Fresh resumed child must supply its own proof.
    let second = fixture.resume(&adapter, (&first).into(), 3).await.unwrap();
    let status = terminal(&adapter, &fixture, &second).await;
    assert_environment(&observed);
    assert!(
        adapter.transport_succeeded(&status),
        "explicit fresh owning Project refresh did not restore continuation; {:?}; {}",
        status.failure,
        fixture.receipt_message(&status)
    );
    adapter.release((&second).into()).unwrap();

    let mut fixture = Fixture::new();
    fixture.request.role = SessionRole::Consultant;
    let mut adapter = fixture.adapter();
    let database = fixture.directory.path().join("state.db");
    let scope = fixture.request.scope.clone();
    let worktree = fixture.request.worktree.clone();
    let executor = Session {
        id: SessionId::new(),
        scope,
        agent: "grok".into(),
        provider: "grok".into(),
        role: SessionRole::Executor,
        native_ref: None,
        pid: None,
        worktree,
        state: SessionState::Starting,
        model: None,
        effort: None,
        recovery: json!({}),
        started_at: now_ms(),
    };
    let executor_id = executor.id;
    adapter.before_environment_admission = Some(Arc::new(move |_entry| {
        let database = database.clone();
        let executor = executor.clone();
        Box::pin(async move {
            Store::open(&database)
                .unwrap()
                .put_session(&executor, 0)
                .unwrap();
            Ok(())
        })
    }));
    let session = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &fixture, &session).await;
    assert_not_spawned(&fixture, &status);
    assert_eq!(
        status.failure.as_deref(),
        Some("Locked: native review conflicts with an active or Lost executor")
    );
    assert_eq!(status.session.state, SessionState::Failed);
    assert!(
        fixture
            .store
            .lock()
            .unwrap()
            .session(executor_id)
            .unwrap()
            .unwrap()
            .0
            .pid
            .is_none()
    );
    // Synthetic reservation owns no OS process. Explicitly settle it; never infer
    // another executor's death from the failed Consultant or from a numeric PID.
    let mut store = fixture.store.lock().unwrap();
    let (mut executor, version) = store.session(executor_id).unwrap().unwrap();
    executor.state = SessionState::Stopped;
    store.put_session(&executor, version).unwrap();
    drop(store);
    adapter.release((&session).into()).unwrap();
    child_completed("adapter::grok::environment_tests::ownership_child");
}
