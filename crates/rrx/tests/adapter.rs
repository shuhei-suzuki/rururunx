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
        git(
            &root,
            &[
                "worktree",
                "add",
                "-b",
                "feature/task",
                worktree.to_str().unwrap(),
            ],
        );
        let mut project = Project::new(
            "fixture".into(),
            root.canonicalize().unwrap(),
            "fixture".into(),
            "main".into(),
        );
        let mut goal = Goal::new(project.id, "test".into(), vec![]);
        let mut task = Task::new(project.id, goal.id, "test".into(), "fake".into());
        task.worktree = Some(worktree.canonicalize().unwrap());
        task.branch = Some("feature/task".into());
        let scope = task.scope();
        let mut store = Store::memory().unwrap();
        store.put_project(&mut project).unwrap();
        store.put_goal(&mut goal).unwrap();
        store.put_task(&mut task).unwrap();
        let request = LaunchRequest {
            project,
            scope: scope.clone(),
            worktree,
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
        status.borrow().clone()
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
            .native_goal_status("unknown".into())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::UnsupportedCapability
    );
    assert_eq!(
        adapter
            .resume_native_goal("unknown".into(), fixture.request.input.clone())
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
        "missing".into(),
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
        ErrorKind::OwnershipMismatch
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
