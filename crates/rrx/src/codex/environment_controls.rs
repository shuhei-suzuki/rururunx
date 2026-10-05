// Synthetic-only controls at the actual constructor/launch/CAS consumers.
use super::super::super::environment::{ExecSite, HookPoint};
use super::*;
use crate::{domain::Project, state::Store};
use std::os::unix::ffi::OsStrExt;

fn synthetic_adapter(owned: &Fixture, executable: PathBuf) -> Arc<CodexAdapter> {
    Arc::new(
        CodexAdapter::with_environment(
            "codex".into(),
            executable,
            owned.store.clone(),
            [("OPENAI_API_KEY".into(), "synthetic-codex-marker".into())],
        )
        .unwrap()
        .component_fixture(),
    )
}
fn writer(owned: &Fixture) -> Store {
    Store::open(
        &owned
            .request
            .project
            .root
            .parent()
            .unwrap()
            .join("state.sqlite3"),
    )
    .unwrap()
}
fn foreign(writer: &mut Store, reference: &str) {
    let mut project = Project::new(
        "foreign synthetic".into(),
        "/tmp/foreign-synthetic".into(),
        "foreign-synthetic".into(),
        "main".into(),
    );
    project.environment_refs = vec![reference.into()];
    writer.put_project(&mut project).unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn initial_and_each_selected_exec_refuse_new_foreign_conflict_before_that_spawn() {
    for site in [
        ExecSite::Initial,
        ExecSite::Version,
        ExecSite::Discovery,
        ExecSite::Main,
    ] {
        let owned = Fixture::new(false);
        let (executable, directory) = wire_fixture(&owned, "complete");
        let adapter = synthetic_adapter(&owned, executable);
        let mut writer = writer(&owned);
        let seen = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let hook_seen = seen.clone();
        adapter.environment_hooks.install(
            site,
            if site == ExecSite::Initial {
                HookPoint::Initial
            } else {
                HookPoint::PreCas
            },
            move |attempt| {
                hook_seen.store(attempt, Ordering::SeqCst);
                foreign(&mut writer, "OPENAI_API_KEY")
            },
        );
        let before = no_effect_snapshot(&adapter, &owned.request.scope);
        let result = bounded(adapter.start(owned.request.clone())).await;
        // Complete any mutant-created actual supervisor before assertions.
        if let Ok(session) = &result {
            terminal_status(&adapter, &SessionRef::from(session)).await;
        }
        if directory.join("leader").exists() {
            assert_leader_reaped(&directory.join("leader"));
        }
        assert!(adapter.environment_hooks.total(site) == (0, 0));
        assert!(
            adapter
                .environment_hooks
                .counts(seen.load(Ordering::SeqCst), site)
                == (0, 0)
        );
        assert!(
            !journal_values(&directory)
                .iter()
                .any(|v| v["method"] == "turn/start")
        );
        if site == ExecSite::Initial {
            assert!(adapter.availability.sites()[Site::GitExecution as usize] == 0);
            let after = no_effect_snapshot(&adapter, &owned.request.scope);
            assert!(after["sessions"] == before["sessions"] && after["events"] == before["events"]);
        }
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("new foreign conflict reached native inference"),
        };
        assert!(
            error.kind == ErrorKind::InvalidConfiguration
                && error.message == "native environment authority unavailable"
        );
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn irrelevant_foreign_change_does_not_stale_own_start_or_restore_native_values() {
    let owned = Fixture::new(false);
    let (executable, directory) = wire_fixture(&owned, "complete");
    let adapter = synthetic_adapter(&owned, executable);
    let mut writer = writer(&owned);
    adapter
        .environment_hooks
        .install(ExecSite::Initial, HookPoint::Initial, move |_| {
            foreign(&mut writer, "IRRELEVANT_SYNTHETIC")
        });
    let observer = adapter.clone();
    adapter
        .environment_hooks
        .install(ExecSite::Main, HookPoint::PostCas, move |_| {
            let registry = observer.registry().unwrap();
            let entry = registry.values().next().unwrap();
            let watched = entry.status.borrow().session.clone();
            let controlled = entry.control.published().unwrap().session;
            let persisted = observer
                .store
                .lock()
                .unwrap()
                .session(watched.id)
                .unwrap()
                .unwrap()
                .0;
            assert!(persisted.pid.is_none() && watched.pid.is_none() && controlled.pid.is_none());
            assert!(
                serde_json::to_value(&persisted).unwrap()
                    == serde_json::to_value(&watched).unwrap()
                    && serde_json::to_value(&watched).unwrap()
                        == serde_json::to_value(&controlled).unwrap()
            );
        });
    let session = bounded(adapter.start(owned.request.clone())).await.unwrap();
    let status = terminal_status(&adapter, &SessionRef::from(&session)).await;
    assert!(status.session.state == SessionState::Exited && status.session.pid.is_none());
    for site in [ExecSite::Version, ExecSite::Discovery, ExecSite::Main] {
        assert!(adapter.environment_hooks.total(site) == (1, 1));
    }
    assert!(std::fs::read_to_string(directory.join("version-canary")).unwrap() == "present");
    assert!(std::fs::read_to_string(directory.join("selected-canary")).unwrap() == "present");
    assert_leader_reaped(&directory.join("leader"));
}
#[test]
fn preparing_only_pre_exec_preserves_first_cause_and_never_calls_closed_admission() {
    for state in [0, 1, 2, 3] {
        let preparation = super::super::super::preparation::Preparation::new();
        match state {
            0 => {
                preparation.cancel();
            }
            1 => {
                preparation.consume(|| Ok(())).unwrap();
            }
            2 => {
                preparation.checkpoint(2, || Ok(())).unwrap();
            }
            _ => {
                preparation.failed(failure(ErrorKind::Timeout, "synthetic first cause"));
            }
        }
        let mut called = false;
        let result = preparation.before_exec(|| {
            called = true;
            Ok(())
        });
        assert!(!called && result.is_err());
        let error = result.err().unwrap();
        assert!(
            error.kind
                == if state == 3 {
                    ErrorKind::Timeout
                } else {
                    ErrorKind::StateConflict
                }
        );
    }
    let preparation = super::super::super::preparation::Preparation::new();
    preparation.before_exec(|| Ok(())).unwrap();
    assert!(matches!(preparation.state().unwrap(), Admission::Preparing));
    let result = preparation
        .before_exec(|| Err::<(), _>(failure(ErrorKind::StateConflict, "synthetic CAS")));
    assert!(result.is_err() && matches!(preparation.state().unwrap(), Admission::Failing(_)));
}
#[tokio::test]
#[ignore = "env-cleared real public constructor entry; invoked by retained own process"]
async fn public_constructor_raw_child() {
    let directory = PathBuf::from(std::env::var_os("RRX_RAW_CTOR_FIXTURE").unwrap());
    let store = Arc::new(Mutex::new(
        Store::open(&directory.join("state.sqlite3")).unwrap(),
    ));
    let mut project = Project::new(
        "raw synthetic".into(),
        "/tmp/raw-synthetic".into(),
        "raw-synthetic".into(),
        "main".into(),
    );
    project.environment_refs = vec!["OPENAI_API_KEY".into()];
    store.lock().unwrap().put_project(&mut project).unwrap();
    let constructed = CodexAdapter::new("codex".into(), "/definitely-not-native".into(), store);
    if std::env::var_os("RRX_RAW_CTOR_BOUNDS").is_some() {
        let error = match constructed {
            Err(error) => error,
            Ok(_) => panic!("real public constructor accepted oversized baseline"),
        };
        assert!(error.kind == ErrorKind::InvalidConfiguration);
        std::fs::write(directory.join("complete"), "raw constructor checked").unwrap();
        return;
    }
    let adapter = constructed.unwrap();
    assert!(adapter.baseline.value_matches(
        "OPENAI_API_KEY",
        std::ffi::OsStr::from_bytes(b"synthetic-\xff")
    ));
    assert!(
        adapter.environment_candidates(project.id).unwrap()
            == BTreeSet::from(["OPENAI_API_KEY".into()])
    );
    assert!(adapter.capabilities().is_empty());
    let long_name = format!("LC_{}", "x".repeat(254));
    // A separately constructed common iterator proves the bound, without ambient override.
    assert!(
        CodexAdapter::with_environment(
            "codex".into(),
            "/definitely-not-native".into(),
            adapter.store.clone(),
            [(long_name.into(), "synthetic".into())]
        )
        .is_err()
    );
    std::fs::write(directory.join("complete"), "raw constructor checked").unwrap();
}
#[tokio::test]
async fn real_public_constructor_captures_raw_os_once_in_scoped_child() {
    use crate::adapter::{ProcessGroup, cleanup_group};
    use std::process::Stdio;
    let directory = tempfile::tempdir().unwrap();
    for bounds in [false, true] {
        let _ = std::fs::remove_file(directory.path().join("complete"));
        let mut command = tokio::process::Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "codex::session::tests::environment_controls::public_constructor_raw_child",
                "--ignored",
                "--nocapture",
            ])
            .env_clear()
            .env("HOME", directory.path())
            .env("PATH", "/usr/bin:/bin")
            .env("RRX_RAW_CTOR_FIXTURE", directory.path())
            .env(
                "OPENAI_API_KEY",
                std::ffi::OsStr::from_bytes(b"synthetic-\xff"),
            )
            .env(
                std::ffi::OsStr::from_bytes(b"LC_\xff"),
                "synthetic excluded name",
            )
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .process_group(0);
        if bounds {
            command
                .env("RRX_RAW_CTOR_BOUNDS", "1")
                .env(format!("LC_{}", "x".repeat(254)), "synthetic");
        }
        let mut child =
            ProcessGroup::new(command.spawn().unwrap(), Arc::new(AtomicBool::new(false))).unwrap();
        let observed =
            tokio::time::timeout(std::time::Duration::from_secs(30), child.observe_exit()).await;
        child = cleanup_group(child).await.unwrap();
        let status = child.reap().await.unwrap();
        assert!(observed.is_ok() && status.success() && directory.path().join("complete").exists());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_stop_at_pre_and_post_cas_cannot_spawn_selected_child() {
    for site in [ExecSite::Version, ExecSite::Discovery, ExecSite::Main] {
        for point in [HookPoint::PreCas, HookPoint::PostCas] {
            let owned = Fixture::new(false);
            let (executable, directory) = wire_fixture(&owned, "complete");
            let adapter = synthetic_adapter(&owned, executable);
            let (reached, receiver) = std::sync::mpsc::sync_channel(1);
            let release = Arc::new((Mutex::new(false), std::sync::Condvar::new()));
            let hook_release = release.clone();
            adapter
                .environment_hooks
                .install(site, point, move |attempt| {
                    reached.send(attempt).unwrap();
                    tokio::task::block_in_place(|| {
                        let (lock, changed) = &*hook_release;
                        let (released, timeout) = changed
                            .wait_timeout_while(
                                lock.lock().unwrap(),
                                std::time::Duration::from_secs(10),
                                |value| !*value,
                            )
                            .unwrap();
                        assert!(
                            *released && !timeout.timed_out(),
                            "finite synchronous hook was not released"
                        );
                    });
                });
            let actor = adapter.clone();
            let request = owned.request.clone();
            let caller = tokio::spawn(async move { actor.start(request).await });
            // This is the test block_on driver, not a task woken into the hook's LIFO slot.
            let attempt_id = receiver
                .recv_timeout(std::time::Duration::from_secs(10))
                .unwrap();
            let (reference, control) = {
                let registry = adapter.registry().unwrap();
                let entry = registry.values().next().unwrap();
                (
                    SessionRef::from(&entry.status.borrow().session),
                    entry.control.clone(),
                )
            };
            let before = no_effect_snapshot(&adapter, &owned.request.scope);
            let before_version = adapter
                .store
                .lock()
                .unwrap()
                .session(reference.id)
                .unwrap()
                .unwrap()
                .1;
            let mut stop = Box::pin(adapter.stop(reference.clone()));
            tokio::select! {
                result=&mut stop => panic!("stop unexpectedly finished while selected spawn hook retained: {result:?}"),
                _=control.preparation.wait_cancelled()=>{},
                _=tokio::time::sleep(std::time::Duration::from_secs(10))=>panic!("real stop did not latch cancellation"),
            }
            assert!(no_effect_snapshot(&adapter, &owned.request.scope) == before);
            {
                let (lock, changed) = &*release;
                *lock.lock().unwrap() = true;
                changed.notify_all();
            }
            let stopped = bounded(stop).await;
            let started = bounded(caller).await.unwrap();
            if let Ok(session) = &started {
                terminal_status(&adapter, &SessionRef::from(session)).await;
            }
            if directory.join("leader").exists() {
                assert_leader_reaped(&directory.join("leader"));
            }
            assert!(adapter.environment_hooks.counts(attempt_id, site) == (0, 0));
            assert!(started.is_err() && stopped.is_ok());
            let final_version = adapter
                .store
                .lock()
                .unwrap()
                .session(reference.id)
                .unwrap()
                .unwrap()
                .1;
            assert!(
                final_version == before_version + 1,
                "cancelled pre-exec performed an extra Session write"
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn checkpoint_roster_changes_are_own_only_and_resume_refuses_initially() {
    for relevant in [false, true] {
        let owned = Fixture::new(false);
        let (executable, directory) = wire_fixture(&owned, "complete");
        let adapter = synthetic_adapter(&owned, executable);
        let session = bounded(adapter.start(owned.request.clone())).await.unwrap();
        let reference = SessionRef::from(&session);
        let original = terminal_status(&adapter, &reference).await;
        let gate = adapter.gates.install(TestPoint::BeforeInitialPersist);
        let mut input = owned.request.input.clone();
        input.version += 1;
        input.payload = "fresh synthetic continuation".into();
        let actor = adapter.clone();
        let target = reference.clone();
        let checkpoint = tokio::spawn(async move { actor.checkpoint(target, input).await });
        bounded(gate.reached()).await;
        let mut writer = writer(&owned);
        foreign(
            &mut writer,
            if relevant {
                "OPENAI_API_KEY"
            } else {
                "IRRELEVANT_SYNTHETIC"
            },
        );
        gate.release();
        bounded(checkpoint).await.unwrap().unwrap();
        let current = adapter.current(&reference).unwrap();
        assert!(
            serde_json::to_value(&current.session).unwrap()
                == serde_json::to_value(&original.session).unwrap()
        );
        let before = no_effect_snapshot(&adapter, &owned.request.scope);
        let journal_before = journal_values(&directory);
        let resumed = bounded(adapter.resume(reference.clone())).await;
        if relevant {
            let error = match resumed {
                Err(error) => error,
                Ok(session) => {
                    terminal_status(&adapter, &SessionRef::from(&session)).await;
                    panic!("relevant foreign conflict permitted resume")
                }
            };
            assert!(error.kind == ErrorKind::InvalidConfiguration);
            let after = no_effect_snapshot(&adapter, &owned.request.scope);
            assert!(before["sessions"] == after["sessions"] && before["events"] == after["events"]);
            for site in [
                Site::GitResolve,
                Site::GitExecution,
                Site::Filesystem,
                Site::ExecutableMetadata,
                Site::Version,
                Site::Transport,
                Site::Connection,
                Site::Frame,
                Site::Grant,
            ] {
                assert!(before["sites"][site as usize] == after["sites"][site as usize]);
            }
            assert!(journal_values(&directory) == journal_before);
            let control = adapter
                .registry()
                .unwrap()
                .get(&reference.id)
                .unwrap()
                .control
                .clone();
            assert!(matches!(&*control.subscribe().borrow(), Phase::Finished(_)));
        } else {
            let resumed = resumed.unwrap();
            assert!(resumed.native_ref == original.session.native_ref);
            let status = terminal_status(&adapter, &reference).await;
            assert!(status.session.state == SessionState::Exited);
            assert!(
                journal_values(&directory)
                    .iter()
                    .filter(|v| v["method"] == "turn/start")
                    .count()
                    == 2
            );
        }
        assert_leader_reaped(&directory.join("leader"));
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn both_config_consumers_refuse_references_before_policy_and_account() {
    for (filename, main) in [("discovery-config", false), ("main-config", true)] {
        for malformed in [false, true] {
            let owned = Fixture::new(false);
            let (executable, directory) = wire_fixture(&owned, "complete");
            let adapter = synthetic_adapter(&owned, executable);
            let overrides = if malformed {
                json!({"web_search":"invalid","mcp_servers":17,"model_provider":17})
            } else {
                json!({"web_search":"invalid","model_provider":"custom","model_providers":{"custom":{"env_key":"AMBIENT_ONLY_SYNTHETIC"}}})
            };
            std::fs::write(
                directory.join(filename),
                serde_json::to_vec(&overrides).unwrap(),
            )
            .unwrap();
            let started = bounded(adapter.start(owned.request.clone())).await;
            if let Ok(session) = &started {
                terminal_status(&adapter, &SessionRef::from(session)).await;
            }
            let error = match started {
                Err(error) => error,
                Ok(_) => panic!("unpermitted provider reference reached a model frame"),
            };
            assert!(
                error.kind
                    == if malformed {
                        ErrorKind::ParseFailure
                    } else {
                        ErrorKind::UnsupportedCapability
                    }
            );
            assert!(
                error.message
                    == if malformed {
                        "invalid native provider environment references"
                    } else {
                        "unsupported native provider environment reference"
                    }
            );
            let journal = journal_values(&directory);
            assert!(!journal.iter().any(|v| matches!(
                v["method"].as_str(),
                Some("account/read" | "thread/start" | "turn/start")
            )));
            assert!(
                adapter.environment_hooks.total(ExecSite::Main)
                    == if main { (1, 1) } else { (0, 0) }
            );
            assert_leader_reaped(&directory.join("leader"));
        }
    }
}
