//! Owned env-cleared synthetic consumers. Canary values are invented, never native credentials.
use super::fixture_support::Fixture;
use super::*;
use crate::domain::ProjectState;

async fn isolated(name: &str) {
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
        observed.is_ok() && exit.success(),
        "synthetic environment child failed; {stdout}; {stderr}"
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
fn assert_not_spawned(fixture: &Fixture, status: &SessionStatus) {
    let observation = fixture.observation(status);
    receipt_support::assert_receipt(&observation.receipt);
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
#[ignore = "only entered by owned env-cleared canary parent"]
async fn initial_child() {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    let mut positive = Fixture::new();
    own_refs(&mut positive, &["LANG"]);
    positive
        .request
        .environment
        .insert("LANG".into(), "synthetic-own-locale".into());
    let observed = positive.directory.path().join("environment.json");
    positive.synthetic(
        "RRX_ENVIRONMENT_OBSERVED",
        observed.to_str().unwrap().into(),
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
    let canary: Value = serde_json::from_slice(&std::fs::read(&observed).unwrap()).unwrap();
    assert!(
        canary["LANG"] == "synthetic-own-locale",
        "synthetic own locale mismatch"
    );
    assert!(
        canary["GROK_SYNTHETIC_AUTH"] == "synthetic-native-global",
        "synthetic native baseline mismatch"
    );
    assert!(canary["RRX_CALLER_FORBIDDEN"].is_null());
    adapter.release((&launched).into()).unwrap();

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
    let session = denied.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &denied, &session).await;
    assert!(
        adapter.transport_succeeded(&status),
        "{:?}; {}",
        status.failure,
        denied.receipt_message(&status)
    );
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
        assert_eq!(
            rejected_start(&adapter, &denied).await.kind,
            ErrorKind::InvalidConfiguration
        );
        denied.request.environment.clear();
    }
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
    assert_not_spawned(&resumed, &status);
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
    let mut control = None;
    for kind in 0..9 {
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
        let first = fixture.start(&adapter).await.unwrap();
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
            adapter
                .checkpoint((&first).into(), input.clone())
                .await
                .unwrap();
            active.store(boundary == 2, Ordering::SeqCst);
            fixture.request.input = input;
            let second = fixture.resume(&adapter, (&first).into(), 2).await.unwrap();
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
        adapter.release((&first).into()).unwrap();
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
        let adapter = fixture.adapter();
        let first = fixture.start(&adapter).await.unwrap();
        let status = terminal(&adapter, &fixture, &first).await;
        assert!(
            adapter.transport_succeeded(&status),
            "own/shared native reference rejected; {:?}; {}",
            status.failure,
            fixture.receipt_message(&status)
        );
        adapter.release((&first).into()).unwrap();
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
    let first = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &fixture, &first).await;
    assert!(
        adapter.transport_succeeded(&status),
        "own ordinary alternative rejected; {:?}; {}",
        status.failure,
        fixture.receipt_message(&status)
    );
    adapter.release((&first).into()).unwrap();
    // A declared unsupported provider key is harmless until a caller tries passing it.
    fixture.request.environment.clear();
    own_refs(&mut fixture, &["OTHER_PROVIDER_REFERENCE"]);
    let first = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &fixture, &first).await;
    assert!(
        adapter.transport_succeeded(&status),
        "unused owning reference changed provider authority"
    );
    adapter.release((&first).into()).unwrap();
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
    let adapter = fixture.adapter();
    let first = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &fixture, &first).await;
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
    let denied = adapter.resume((&first).into()).await.unwrap_err();
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
    let second = fixture.resume(&adapter, (&first).into(), 3).await.unwrap();
    let status = terminal(&adapter, &fixture, &second).await;
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
