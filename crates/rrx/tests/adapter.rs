use rrx::{adapter::*, config::Config, domain::*, state::Store};
use serde_json::json;
use std::{
    collections::BTreeMap,
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    request: LaunchRequest,
    store: SharedStore,
}
fn git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
impl Fixture {
    fn new() -> Self {
        Self::configured("feature/task", false)
    }
    fn configured(branch: &str, foreign: bool) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("project");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "-b", "main"]);
        git(
            &root,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--allow-empty",
                "-m",
                "fixture",
            ],
        );
        let worktree = root.join("worktree/task");
        if foreign {
            std::fs::create_dir_all(&worktree).unwrap();
            git(&worktree, &["init", "-b", branch]);
            git(
                &worktree,
                &[
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "commit",
                    "--allow-empty",
                    "-m",
                    "foreign",
                ],
            );
        } else {
            git(
                &root,
                &["worktree", "add", "-b", branch, worktree.to_str().unwrap()],
            );
        }
        let mut project = Project::new(
            "fixture".into(),
            root.canonicalize().unwrap(),
            rrx::git::repository_identity(&root.canonicalize().unwrap(), "main").unwrap(),
            "main".into(),
        );
        let mut goal = Goal::new(
            project.id,
            "test".into(),
            vec![CompletionCriterion {
                id: "fixture".into(),
                description: "fake agent exits".into(),
                evidence: None,
                satisfied: false,
            }],
        );
        let mut task = Task::new(project.id, goal.id, "test".into(), "fake".into());
        task.worktree = Some(worktree.canonicalize().unwrap());
        task.branch = Some(branch.into());
        let scope = task.scope();
        let mut store = Store::open(&temp.path().join("state.sqlite3")).unwrap();
        store.put_project(&mut project).unwrap();
        store.put_goal(&mut goal).unwrap();
        store.put_task(&mut task).unwrap();
        let request = LaunchRequest {
            project,
            scope: scope.clone(),
            worktree: worktree.canonicalize().unwrap(),
            role: SessionRole::Executor,
            mode: LaunchMode::NonInteractive,
            input: PreparedInput {
                scope,
                kind: InputKind::ContextPack,
                revision: "fixture-revision".into(),
                version: 1,
                source_versions: BTreeMap::from([("rules".into(), "v1".into())]),
                payload: "prepared payload\n".into(),
            },
            environment: BTreeMap::from([
                ("PATH".into(), "/usr/bin:/bin".into()),
                ("TASK_MARKER".into(), "project-a".into()),
            ]),
            model: None,
            effort: None,
        };
        Self {
            _temp: temp,
            request,
            store: Arc::new(Mutex::new(store)),
        }
    }
    fn adapter(&self, script: &str) -> GenericCliAdapter {
        GenericCliAdapter::new(
            "fake".into(),
            vec![
                "/bin/sh".into(),
                "-c".into(),
                script.into(),
                "fake-agent".into(),
            ],
            self.store.clone(),
        )
        .unwrap()
    }
}
async fn finished(adapter: &dyn AgentAdapter, session: &Session) -> SessionStatus {
    let mut status = adapter.subscribe(session.into()).unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        while !status.borrow().terminal() {
            status.changed().await.unwrap();
        }
        let result = status.borrow().clone();
        if result.session.state == SessionState::Lost {
            eprintln!("native supervisor loss: {:?}", result.failure);
        }
        result
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn configured_command_receives_only_prepared_context_explicit_environment_and_task_cwd() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter("/bin/cat; /bin/pwd; /usr/bin/env; printf 'diagnostic' >&2");
    assert_eq!(
        adapter.capabilities(),
        [Capability::Execute, Capability::NonInteractive].into()
    );
    assert_eq!(adapter.probe().unwrap().authenticated, None);
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    assert_ne!(session.id.to_string(), session.pid.unwrap().to_string());
    let status = finished(&adapter, &session).await;
    assert_eq!(status.session.state, SessionState::Exited);
    let output = String::from_utf8(status.stdout).unwrap();
    assert!(output.starts_with("prepared payload\n"));
    assert!(
        output.contains(
            fixture
                .request
                .worktree
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
        )
    );
    assert!(output.contains("TASK_MARKER=project-a"));
    assert!(!output.contains("HOME="));
    assert_eq!(status.stderr, b"diagnostic");
    let saved = fixture
        .store
        .lock()
        .unwrap()
        .session(session.id)
        .unwrap()
        .unwrap()
        .0;
    assert_eq!(saved.id, session.id);
    assert_eq!(saved.scope, session.scope);
    assert_eq!(saved.state, SessionState::Exited);
    assert!(
        !serde_json::to_string(&saved)
            .unwrap()
            .contains("TASK_MARKER")
    );
    let usage = adapter
        .usage((&session).into(), "implement".into(), None)
        .await
        .unwrap();
    assert_eq!(usage.input_tokens, None);
    assert_eq!(usage.cached_input_tokens, None);
    assert_eq!(usage.output_tokens, None);
    assert!(usage.missing_reason.is_some());
    fixture.store.lock().unwrap().put_usage(&usage).unwrap();
}

#[tokio::test]
async fn explicit_unsupported_operations_and_registry_preserve_provider_independence() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter("/bin/cat");
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    finished(&adapter, &session).await;
    assert_eq!(
        adapter.attach((&session).into()).await.unwrap_err().kind,
        ErrorKind::UnsupportedCapability
    );
    assert_eq!(
        adapter.resume((&session).into()).await.unwrap_err().kind,
        ErrorKind::UnsupportedCapability
    );
    assert_eq!(
        adapter
            .start_native_goal(fixture.request.input.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::UnsupportedCapability
    );
    assert_eq!(
        adapter
            .native_goal_status(NativeGoalRef {
                scope: fixture.request.scope.clone(),
                session_id: Some(session.id),
                native_ref: "unknown".into()
            })
            .await
            .unwrap_err()
            .kind,
        ErrorKind::UnsupportedCapability
    );
    assert_eq!(
        adapter
            .resume_native_goal(
                NativeGoalRef {
                    scope: fixture.request.scope.clone(),
                    session_id: Some(session.id),
                    native_ref: "unknown".into()
                },
                fixture.request.input.clone()
            )
            .await
            .unwrap_err()
            .kind,
        ErrorKind::UnsupportedCapability
    );
    assert_eq!(
        adapter
            .submit_approval((&session).into(), json!("APPROVE"))
            .await
            .unwrap_err()
            .kind,
        ErrorKind::UnsupportedCapability
    );
    assert_eq!(
        adapter
            .checkpoint((&session).into(), fixture.request.input.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::UnsupportedCapability
    );
    let mut interactive = fixture.request.clone();
    interactive.mode = LaunchMode::Interactive;
    assert_eq!(
        adapter.start(interactive).await.unwrap_err().kind,
        ErrorKind::UnsupportedCapability
    );
    let mut review = fixture.request.clone();
    review.role = SessionRole::Reviewer;
    review.input.kind = InputKind::ReviewBundle;
    assert_eq!(
        adapter.start(review).await.unwrap_err().kind,
        ErrorKind::UnsupportedCapability
    );
    let config: Config = toml::from_str("[agents.fake]\ncommand=['/bin/cat']").unwrap();
    let registry = AgentRegistry::from_config(&config, fixture.store.clone()).unwrap();
    assert_eq!(registry.names().collect::<Vec<_>>(), ["fake"]);
    assert!(registry.get("missing").is_err());
    assert!(
        !registry
            .get("fake")
            .unwrap()
            .capabilities()
            .contains(&Capability::NativeGoal)
    );
}

#[tokio::test]
async fn stop_reaps_child_and_descendants_and_is_idempotent() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter("/bin/cat; sleep 60 & printf 'descendant=%s\n' \"$!\"; wait");
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    let mut events = adapter.subscribe((&session).into()).unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while !String::from_utf8_lossy(&events.borrow().stdout).contains("descendant=") {
            events.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    let descendant: i32 = String::from_utf8_lossy(&events.borrow().stdout)
        .lines()
        .find_map(|line| line.strip_prefix("descendant="))
        .unwrap()
        .parse()
        .unwrap();
    let stopped = adapter.stop((&session).into()).await.unwrap();
    assert_eq!(stopped.session.state, SessionState::Stopped);
    assert_eq!(
        adapter.stop((&session).into()).await.unwrap().session.state,
        SessionState::Stopped
    );
    // A killed descendant may briefly be a zombie pending the host's init reaper.
    let output = Command::new("ps")
        .args(["-o", "stat=", "-p", &descendant.to_string()])
        .output()
        .unwrap();
    let state = String::from_utf8_lossy(&output.stdout);
    assert!(
        state.trim().is_empty() || state.trim().starts_with('Z'),
        "descendant still alive: {state}"
    );
}

#[tokio::test]
async fn failed_launch_nonzero_exit_and_bounded_output_are_observable() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter("/bin/cat; printf 'failure' >&2; exit 17");
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    let status = finished(&adapter, &session).await;
    assert_eq!(status.session.state, SessionState::Failed);
    assert_eq!(status.exit_code, Some(17));
    let missing = GenericCliAdapter::new(
        "fake".into(),
        vec!["/no/such/agent".into()],
        fixture.store.clone(),
    )
    .unwrap();
    assert_eq!(
        missing
            .start(fixture.request.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::ExecutableMissing
    );
    let adapter = fixture.adapter("/bin/cat; /usr/bin/yes x | /usr/bin/head -c 200000; /usr/bin/yes y | /usr/bin/head -c 200000 >&2");
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    let status = finished(&adapter, &session).await;
    assert_eq!(status.session.state, SessionState::Exited);
    assert_eq!(status.stdout.len(), 65536);
    assert_eq!(status.stderr.len(), 65536);
    assert!(status.stdout_truncated && status.stderr_truncated);
}

#[tokio::test]
async fn project_goal_task_context_cwd_branch_and_lock_boundaries_fail_closed() {
    let fixture = Fixture::new();
    let other = Fixture::new();
    let adapter = fixture.adapter("/bin/cat");
    let mut foreign = fixture.request.clone();
    foreign.input.scope = other.request.scope.clone();
    assert_eq!(
        adapter.start(foreign).await.unwrap_err().kind,
        ErrorKind::OwnershipMismatch
    );
    let mut foreign = fixture.request.clone();
    foreign.worktree = other.request.worktree;
    assert_eq!(
        adapter.start(foreign).await.unwrap_err().kind,
        ErrorKind::OwnershipMismatch
    );
    let mut foreign = fixture.request.clone();
    foreign.scope.task_id = Some(TaskId::new());
    foreign.input.scope = foreign.scope.clone();
    assert!(adapter.start(foreign).await.is_err());
    let mut forged = fixture.request.clone();
    forged
        .environment
        .insert("GIT_DIR".into(), "/foreign".into());
    assert_eq!(
        adapter.start(forged).await.unwrap_err().kind,
        ErrorKind::InvalidInput
    );
    let mut lock = Record::new(
        fixture.request.scope.clone(),
        RecordKind::WorktreeLock,
        json!({"active":true,"revision":"fixture","worktree":fixture.request.worktree,"branch":"feature/task","reason":"review"}),
    );
    fixture.store.lock().unwrap().put_record(&mut lock).unwrap();
    assert!(adapter.start(fixture.request.clone()).await.is_err());
    lock.data["active"] = json!(false);
    fixture.store.lock().unwrap().put_record(&mut lock).unwrap();
    git(&fixture.request.worktree, &["checkout", "-b", "unexpected"]);
    assert_eq!(
        adapter
            .start(fixture.request.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::OwnershipMismatch
    );
    let session = fixture
        .store
        .lock()
        .unwrap()
        .records(&fixture.request.scope, RecordKind::Session)
        .unwrap()
        .last()
        .unwrap()
        .data
        .clone();
    assert_eq!(session["state"], "FAILED");
    git(&fixture.request.worktree, &["checkout", "feature/task"]);
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    finished(&adapter, &session).await;
    let reference = SessionRef {
        id: session.id,
        scope: other.request.scope,
    };
    assert_eq!(
        adapter.status(reference.clone()).await.unwrap_err().kind,
        ErrorKind::OwnershipMismatch
    );
    assert_eq!(
        adapter.stop(reference.clone()).await.unwrap_err().kind,
        ErrorKind::OwnershipMismatch
    );
    assert_eq!(
        adapter.attach(reference).await.unwrap_err().kind,
        ErrorKind::UnsupportedCapability
    );
    let fresh = fixture.adapter("/bin/cat");
    assert_eq!(
        fresh.status((&session).into()).await.unwrap_err().kind,
        ErrorKind::SessionLost
    );
}

#[tokio::test]
async fn separate_projects_run_concurrently_without_environment_or_context_reuse() {
    let a = Fixture::new();
    let b = Fixture::new();
    let aa = a.adapter("/bin/cat; printf '%s' \"$TASK_MARKER\"");
    let ba = b.adapter("/bin/cat; printf '%s' \"$TASK_MARKER\"");
    let mut rb = b.request.clone();
    rb.environment
        .insert("TASK_MARKER".into(), "project-b".into());
    rb.input.payload = "only-b\n".into();
    let (sa, sb) = tokio::join!(aa.start(a.request.clone()), ba.start(rb));
    let sa = sa.unwrap();
    let sb = sb.unwrap();
    let (aout, bout) = tokio::join!(finished(&aa, &sa), finished(&ba, &sb));
    assert_eq!(aout.stdout, b"prepared payload\nproject-a");
    assert_eq!(bout.stdout, b"only-b\nproject-b");
    assert_ne!(sa.scope.project_id, sb.scope.project_id);
    assert_ne!(sa.id, sb.id);
}

#[tokio::test]
async fn foreign_git_repository_inside_namespace_detached_and_protected_branches_are_rejected() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter("/bin/cat");
    let mut forged = fixture.request.clone();
    forged.project.base_branch = "forged".into();
    assert!(adapter.start(forged).await.is_err());
    git(&fixture.request.worktree, &["checkout", "--detach"]);
    assert_eq!(
        adapter
            .start(fixture.request.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::OwnershipMismatch
    );
    git(&fixture.request.worktree, &["checkout", "feature/task"]);
    let protected = Fixture::configured("master", false);
    assert_eq!(
        protected
            .adapter("/bin/cat")
            .start(protected.request.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::OwnershipMismatch
    );
    let foreign = Fixture::configured("feature/task", true);
    assert_eq!(
        foreign
            .adapter("/bin/cat")
            .start(foreign.request.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::OwnershipMismatch
    );
}

#[tokio::test]
async fn immediate_stop_and_adapter_drop_do_not_leave_running_children() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter("sleep 60");
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    assert_eq!(
        adapter.stop((&session).into()).await.unwrap().session.state,
        SessionState::Stopped
    );
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    let mut events = adapter.subscribe((&session).into()).unwrap();
    drop(adapter);
    tokio::time::timeout(Duration::from_secs(5), async {
        while !events.borrow().terminal() {
            events.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    assert_eq!(events.borrow().session.state, SessionState::Stopped);
}

fn assert_process_dead(pid: i32) {
    let output = Command::new("ps")
        .args(["-o", "stat=", "-p", &pid.to_string()])
        .output()
        .unwrap();
    let state = String::from_utf8_lossy(&output.stdout);
    let alive = !state.trim().is_empty() && !state.trim().starts_with('Z');
    // Clean a failed regression's known fixture child before asserting, so mutations
    // cannot leave an unrelated long-lived process behind.
    if alive {
        let _ = rustix::process::kill_process(
            rustix::process::Pid::from_raw(pid).unwrap(),
            rustix::process::Signal::KILL,
        );
    }
    assert!(!alive, "fixture descendant still running: {state}");
}

#[tokio::test]
async fn natural_exit_cleans_redirected_background_descendants_before_terminal_persistence() {
    let fixture = Fixture::new();
    let adapter = fixture
        .adapter("/bin/cat; sleep 60 >/dev/null 2>&1 & printf 'descendant=%s\n' \"$!\"; exit 0");
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    let status = finished(&adapter, &session).await;
    let pid = String::from_utf8_lossy(&status.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("descendant="))
        .unwrap()
        .parse()
        .unwrap();
    assert_process_dead(pid);
    assert_eq!(status.session.state, SessionState::Exited);
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
        SessionState::Exited
    );
}

#[tokio::test]
async fn concurrent_snapshot_updates_are_not_overwritten_and_identity_survives_reopen() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter("/bin/cat; sleep 60");
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    {
        let mut store = fixture.store.lock().unwrap();
        let (mut newer, version) = store.session(session.id).unwrap().unwrap();
        newer.state = SessionState::Lost;
        newer.recovery["operator_note"] = json!("verify process death");
        store.put_session(&newer, version).unwrap();
    }
    let status = adapter.stop((&session).into()).await.unwrap();
    assert!(status.failure.unwrap().contains("changed concurrently"));
    let reopened = Arc::new(Mutex::new(
        Store::open(&fixture._temp.path().join("state.sqlite3")).unwrap(),
    ));
    let saved = reopened
        .lock()
        .unwrap()
        .session(session.id)
        .unwrap()
        .unwrap()
        .0;
    assert_eq!(saved.id, session.id);
    assert_eq!(saved.scope, session.scope);
    assert_eq!(saved.state, SessionState::Lost);
    assert_eq!(saved.recovery["operator_note"], "verify process death");
    let restarted =
        GenericCliAdapter::new("fake".into(), vec!["/bin/cat".into()], reopened).unwrap();
    assert_eq!(
        restarted.status((&session).into()).await.unwrap_err().kind,
        ErrorKind::SessionLost
    );
}

#[tokio::test]
async fn cancelled_preflight_marks_reserved_session_failed_and_large_unread_stdin_is_failed() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter("/bin/cat");
    let mut launch = adapter.start(fixture.request.clone());
    std::future::poll_fn(|context| {
        assert!(
            launch.as_mut().poll(context).is_pending(),
            "preflight should yield for native Git"
        );
        std::task::Poll::Ready(())
    })
    .await;
    drop(launch);
    let records = fixture
        .store
        .lock()
        .unwrap()
        .records(&fixture.request.scope, RecordKind::Session)
        .unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].data["state"], "FAILED");
    let adapter = fixture.adapter("exit 0");
    let mut request = fixture.request.clone();
    request.input.payload = "x".repeat(2 * 1024 * 1024);
    let session = adapter.start(request).await.unwrap();
    let status = finished(&adapter, &session).await;
    assert_eq!(status.session.state, SessionState::Failed);
    assert!(status.failure.unwrap().contains("stdin"));
}

#[tokio::test]
async fn terminal_retention_is_bounded_and_output_can_be_released() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter("/bin/cat");
    let first = adapter.start(fixture.request.clone()).await.unwrap();
    finished(&adapter, &first).await;
    let mut last = first.clone();
    for _ in 0..33 {
        last = adapter.start(fixture.request.clone()).await.unwrap();
        finished(&adapter, &last).await;
    }
    assert_eq!(
        adapter.status((&first).into()).await.unwrap_err().kind,
        ErrorKind::SessionLost
    );
    adapter.release((&last).into()).unwrap();
    assert_eq!(
        adapter.status((&last).into()).await.unwrap_err().kind,
        ErrorKind::SessionLost
    );
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .session(first.id)
            .unwrap()
            .unwrap()
            .0
            .state,
        SessionState::Exited
    );
}

#[tokio::test]
async fn executable_symlink_keeps_configured_argv_zero_and_terminal_goal_cannot_launch() {
    let fixture = Fixture::new();
    let alias = fixture._temp.path().join("fake-shell");
    std::os::unix::fs::symlink("/bin/sh", &alias).unwrap();
    let adapter = GenericCliAdapter::new(
        "fake".into(),
        vec![
            alias.to_str().unwrap().into(),
            "-c".into(),
            "/bin/cat; printf '%s' \"$0\"".into(),
        ],
        fixture.store.clone(),
    )
    .unwrap();
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    let status = finished(&adapter, &session).await;
    assert!(String::from_utf8_lossy(&status.stdout).ends_with(alias.to_str().unwrap()));
    {
        let mut store = fixture.store.lock().unwrap();
        let mut goal = store
            .goal(fixture.request.scope.goal_id.unwrap())
            .unwrap()
            .unwrap();
        goal.state = GoalState::Paused;
        store.put_goal(&mut goal).unwrap();
    }
    assert_eq!(
        adapter
            .start(fixture.request.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::InvalidInput
    );
}

#[test]
fn runtime_shutdown_terminates_native_group_and_preserves_uncertain_reservation() {
    let fixture = Fixture::new();
    let adapter = fixture
        .adapter("/bin/cat; sleep 60 >/dev/null 2>&1 & printf 'descendant=%s\n' \"$!\"; wait");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let (session, descendant) = runtime.block_on(async {
        let session = adapter.start(fixture.request.clone()).await.unwrap();
        let mut events = adapter.subscribe((&session).into()).unwrap();
        tokio::time::timeout(Duration::from_secs(5), async {
            while !String::from_utf8_lossy(&events.borrow().stdout).contains("descendant=") {
                events.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        let pid: i32 = String::from_utf8_lossy(&events.borrow().stdout)
            .lines()
            .find_map(|line| line.strip_prefix("descendant="))
            .unwrap()
            .parse()
            .unwrap();
        (session, pid)
    });
    drop(runtime);
    assert_process_dead(descendant);
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
        SessionState::Running
    );
}

#[tokio::test]
async fn atomic_starting_reservation_excludes_duplicate_executor_and_review_acquisition() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter("/bin/cat; sleep 60");
    let (left, right) = tokio::join!(
        adapter.start(fixture.request.clone()),
        adapter.start(fixture.request.clone())
    );
    assert_eq!(usize::from(left.is_ok()) + usize::from(right.is_ok()), 1);
    let (session, rejected) = match (left, right) {
        (Ok(session), Err(error)) | (Err(error), Ok(session)) => (session, error),
        _ => unreachable!("one executor is reserved"),
    };
    assert_eq!(rejected.kind, ErrorKind::StateConflict);
    let mut lock = Record::new(
        fixture.request.scope.clone(),
        RecordKind::WorktreeLock,
        json!({"active":true,"revision":"fixture","worktree":fixture.request.worktree.canonicalize().unwrap(),"branch":"feature/task","reason":"review race"}),
    );
    assert!(fixture.store.lock().unwrap().put_record(&mut lock).is_err());
    adapter.stop((&session).into()).await.unwrap();
    fixture.store.lock().unwrap().put_record(&mut lock).unwrap();
    assert_eq!(
        adapter
            .start(fixture.request.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::Locked
    );
}

#[tokio::test]
async fn lost_executor_keeps_worktree_reserved_until_explicit_verified_dead_resolution() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter("/bin/cat; sleep 60");
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    adapter.stop((&session).into()).await.unwrap();
    let mut store = fixture.store.lock().unwrap();
    let (mut snapshot, version) = store.session(session.id).unwrap().unwrap();
    snapshot.state = SessionState::Lost;
    let version = store.put_session(&snapshot, version).unwrap();
    let mut lock = Record::new(
        fixture.request.scope.clone(),
        RecordKind::WorktreeLock,
        json!({"active":true,"revision":"fixture","worktree":fixture.request.worktree.canonicalize().unwrap(),"branch":"feature/task","reason":"uncertain process"}),
    );
    assert!(store.put_record(&mut lock).is_err());
    snapshot.state = SessionState::Stopped;
    snapshot.recovery["verified_dead"] = json!(true);
    store.put_session(&snapshot, version).unwrap();
    store.put_record(&mut lock).unwrap();
}

#[tokio::test]
async fn blocked_project_rejects_new_launch_and_allows_owned_native_stop() {
    let fixture = Fixture::new();
    let adapter = fixture.adapter("/bin/cat; sleep 60");
    let session = adapter.start(fixture.request.clone()).await.unwrap();
    {
        let mut store = fixture.store.lock().unwrap();
        let mut project = store.project(session.scope.project_id).unwrap().unwrap();
        project.state = ProjectState::Blocked;
        project.blocked_reason = Some("source unavailable fixture".into());
        store.put_project(&mut project).unwrap();
    }
    assert_eq!(
        adapter
            .start(fixture.request.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::OwnershipMismatch
    );
    let stopped = adapter.stop((&session).into()).await.unwrap();
    assert_eq!(stopped.session.state, SessionState::Stopped);
    let store = fixture.store.lock().unwrap();
    let (persisted, _) = store.session(session.id).unwrap().unwrap();
    assert_eq!(persisted.state, SessionState::Stopped);
    assert_eq!(persisted.recovery, session.recovery);
    assert_eq!(persisted.native_ref, session.native_ref);
    assert_eq!(
        store
            .records(&session.scope, RecordKind::Session)
            .unwrap()
            .len(),
        1
    );
}
