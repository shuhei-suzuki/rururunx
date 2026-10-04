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
fn assert_not_spawned(fixture: &Fixture, session: &Session, lower: u64) {
    let store = fixture.store.lock().unwrap();
    let events = store.events(&session.scope, lower, 500).unwrap();
    assert!(
        !events
            .iter()
            .any(|event| event.kind == "grok.process_spawned"),
        "environment rejection started native child"
    );
    assert!(
        !events.iter().any(|event| event.kind == "session.saved"
            && event.data["evidence"]["dispatch_intent"]["input_version"].as_u64()
                == Some(fixture.request.input.version)),
        "environment rejection consumed new prompt"
    );
}
#[tokio::test]
async fn environment_initial_selection_drives_real_canary_and_rejection_consumers() {
    isolated("adapter::grok::environment_tests::initial_child").await;
}
#[tokio::test]
async fn environment_live_admission_drives_actual_start_resume_and_stop_consumers() {
    isolated("adapter::grok::environment_tests::admission_child").await;
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
    adapter.release(launched.into()).await.unwrap();

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
        denied.start(&adapter).await.unwrap_err().kind,
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
        denied.start(&adapter).await.unwrap_err().kind,
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
        denied.start(&adapter).await.unwrap_err().kind,
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
    adapter.release(session.into()).await.unwrap();
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
            denied.start(&adapter).await.unwrap_err().kind,
            ErrorKind::InvalidConfiguration
        );
        denied.request.environment.clear();
    }
    child_completed("adapter::grok::environment_tests::initial_child");
}

#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn admission_child() {
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
    let lower = receipt_support::watermark(&fixture.store, &fixture.request.scope).unwrap();
    let session = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &fixture, &session).await;
    assert_eq!(
        status.failure.as_ref().unwrap().kind,
        ErrorKind::InvalidConfiguration
    );
    assert_eq!(status.session.state, SessionState::Failed);
    assert!(status.session.pid.is_none());
    assert_not_spawned(&fixture, &session, lower);
    adapter.release(session.into()).await.unwrap();

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
        .checkpoint(first.clone().into(), input.clone())
        .await
        .unwrap();
    resumed.request.input = input;
    let lower = receipt_support::watermark(&resumed.store, &resumed.request.scope).unwrap();
    let second = resumed
        .resume(&adapter, first.clone().into(), 2)
        .await
        .unwrap();
    let status = terminal(&adapter, &resumed, &second).await;
    assert_eq!(
        status.failure.as_ref().unwrap().kind,
        ErrorKind::InvalidConfiguration
    );
    assert_eq!(status.session.state, SessionState::Failed);
    assert_eq!(
        status.session.recovery["input_version"],
        previous["input_version"]
    );
    for key in ["dispatch_intent", "prompt_id", "dispatch_state"] {
        assert_eq!(status.session.recovery[key], previous[key]);
    }
    assert_not_spawned(&resumed, &second, lower);
    assert_eq!(invocations.load(Ordering::SeqCst), 2);
    adapter.release(second.into()).await.unwrap();

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
    let lower = receipt_support::watermark(&fixture.store, &fixture.request.scope).unwrap();
    let session = fixture.start(adapter.as_ref()).await.unwrap();
    let status = terminal(&adapter, &fixture, &session).await;
    assert_eq!(status.session.state, SessionState::Stopped);
    let stopped = stop_result.lock().unwrap().take().unwrap();
    let stopped = stopped.await.unwrap().unwrap();
    assert_eq!(stopped.session.state, SessionState::Stopped);
    assert_not_spawned(&fixture, &session, lower);
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
    adapter.release(session.into()).await.unwrap();
    child_completed("adapter::grok::environment_tests::admission_child");
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
