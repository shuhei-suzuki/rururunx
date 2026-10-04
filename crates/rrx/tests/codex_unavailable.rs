//! Executes the ordinary library; no private fixture availability is linked here.
use rrx::{adapter::*, codex::CodexAdapter, domain::*, state::Store};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
};

const REASON: &str = "native workload ownership and dispatch producer unavailable";

fn unavailable<T: std::fmt::Debug>(result: AdapterResult<T>) {
    let error = result.unwrap_err();
    assert_eq!(error.kind, ErrorKind::UnsupportedCapability);
    assert_eq!(error.message, REASON);
}
fn snapshot(store: &SharedStore, scope: &Scope) -> Value {
    let store = store.lock().unwrap();
    json!({"sessions":store.records(scope,RecordKind::Session).unwrap(),"events":store.events(scope,0,1000).unwrap()})
}

#[tokio::test]
#[ignore = "reexecuted by the scoped ordinary-library harness"]
async fn ordinary_empty_library_child() {
    let directory = std::path::PathBuf::from(std::env::var_os("RRX_EMPTY_FIXTURE").unwrap());
    let root = directory.join("project").canonicalize().unwrap();
    let mut store = Store::open(&directory.join("state.sqlite3")).unwrap();
    let mut project = Project::new(
        "ordinary fixture".into(),
        root.clone(),
        std::fs::read_to_string(directory.join("identity")).unwrap(),
        "main".into(),
    );
    store.put_project(&mut project).unwrap();
    let scope = Scope::project(project.id);
    let request = LaunchRequest {
        project,
        scope: scope.clone(),
        worktree: root,
        role: SessionRole::Consultant,
        mode: LaunchMode::NonInteractive,
        input: PreparedInput {
            scope: scope.clone(),
            kind: InputKind::ContextPack,
            revision: std::fs::read_to_string(directory.join("revision")).unwrap(),
            version: 1,
            source_versions: BTreeMap::from([("rules".into(), "v1".into())]),
            payload: "synthetic bounded current input".into(),
        },
        environment: BTreeMap::new(),
        model: None,
        effort: None,
    };
    let store = Arc::new(Mutex::new(store));
    let before = snapshot(&store, &scope);
    let adapter = Arc::new(
        CodexAdapter::new(
            "codex".into(),
            directory.join("codex-sentinel"),
            store.clone(),
        )
        .unwrap()
        .with_runtime_broker(),
    );
    let unknown = SessionRef {
        id: SessionId::new(),
        scope: scope.clone(),
    };
    let schema = json!({"type":"object","properties":{"ok":{"type":"boolean"}},"required":["ok"],"additionalProperties":false});
    assert!(adapter.capabilities().is_empty());
    unavailable(adapter.probe());
    let started = adapter.start(request.clone()).await;
    assert_eq!(
        snapshot(&store, &scope),
        before,
        "ordinary start reached a durable effect"
    );
    assert!(!directory.join("git-effect").exists());
    assert!(!directory.join("codex-effect").exists());
    unavailable(started);
    let started = adapter
        .start_structured(request.clone(), schema.clone())
        .await;
    assert_eq!(
        snapshot(&store, &scope),
        before,
        "ordinary structured start reached a durable effect"
    );
    assert!(!directory.join("git-effect").exists());
    assert!(!directory.join("codex-effect").exists());
    unavailable(started);
    unavailable(adapter.resume(unknown.clone()).await);
    unavailable(
        adapter
            .checkpoint(unknown.clone(), request.input.clone())
            .await,
    );
    unavailable(adapter.attach(unknown.clone()).await);
    for decision in ["Approve", "Deny", "Cancel"] {
        unavailable(
            adapter
                .submit_approval(unknown.clone(), json!({"decision":decision}))
                .await,
        );
    }
    unavailable(adapter.release(unknown.clone()));
    let dynamic: Arc<dyn AgentAdapter> = adapter;
    assert!(dynamic.capabilities().is_empty());
    unavailable(dynamic.probe());
    unavailable(dynamic.start(request.clone()).await);
    unavailable(dynamic.start_structured(request.clone(), schema).await);
    unavailable(dynamic.resume(unknown.clone()).await);
    unavailable(
        dynamic
            .checkpoint(unknown.clone(), request.input.clone())
            .await,
    );
    unavailable(dynamic.attach(unknown.clone()).await);
    unavailable(
        dynamic
            .submit_approval(unknown.clone(), json!({"decision":"Deny"}))
            .await,
    );
    unavailable(dynamic.release(unknown.clone()));
    assert_eq!(
        dynamic.status(unknown.clone()).await.unwrap_err().kind,
        ErrorKind::SessionLost
    );
    assert_eq!(
        dynamic.stop(unknown.clone()).await.unwrap_err().kind,
        ErrorKind::SessionLost
    );
    assert_eq!(
        dynamic.subscribe(unknown.clone()).unwrap_err().kind,
        ErrorKind::SessionLost
    );
    assert_eq!(
        dynamic
            .usage(unknown.clone(), "review".into(), None)
            .await
            .unwrap_err()
            .kind,
        ErrorKind::SessionLost
    );
    assert_eq!(
        dynamic
            .pending_approvals(unknown.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::SessionLost
    );
    let status = SessionStatus {
        session: Session {
            id: unknown.id,
            scope: scope.clone(),
            agent: "codex".into(),
            provider: "codex".into(),
            role: SessionRole::Consultant,
            native_ref: Some("caller hint".into()),
            pid: None,
            worktree: request.worktree.clone(),
            state: SessionState::Exited,
            model: None,
            effort: None,
            recovery: json!({"completed":true}),
            started_at: now_ms(),
        },
        exit_code: Some(0),
        stdout: vec![],
        stderr: vec![],
        stdout_truncated: false,
        stderr_truncated: false,
        failure: None,
    };
    assert!(!dynamic.transport_succeeded(&status));
    let native = NativeGoalRef {
        scope: scope.clone(),
        session_id: None,
        native_ref: "synthetic".into(),
    };
    for error in [
        dynamic
            .start_native_goal(request.input.clone())
            .await
            .unwrap_err(),
        dynamic
            .native_goal_status(native.clone())
            .await
            .unwrap_err(),
        dynamic
            .resume_native_goal(native, request.input)
            .await
            .unwrap_err(),
    ] {
        assert_eq!(error.kind, ErrorKind::UnsupportedCapability);
    }
    assert_eq!(snapshot(&store, &scope), before);
    assert!(!directory.join("codex-effect").exists());
    assert!(!directory.join("git-effect").exists());
}

fn fixture_git(root: &Path, home: &Path, args: &[&str]) -> String {
    let output = Command::new("/usr/bin/git")
        .args(args)
        .current_dir(root)
        .env_clear()
        .env("HOME", home)
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert!(output.status.success(), "synthetic Git setup failed");
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

#[test]
fn ordinary_library_refuses_in_scoped_child_with_effect_reaching_input() {
    use std::os::unix::fs::PermissionsExt;
    let temporary = tempfile::Builder::new()
        .prefix("rrx-empty-")
        .tempdir_in("/tmp")
        .unwrap();
    let directory = temporary.path().canonicalize().unwrap();
    let root = directory.join("project");
    let home = directory.join("home");
    let bin = directory.join("bin");
    for path in [&root, &home, &bin] {
        std::fs::create_dir(path).unwrap();
    }
    fixture_git(&root, &home, &["init", "-b", "main"]);
    std::fs::write(root.join("fixture"), "synthetic\n").unwrap();
    fixture_git(&root, &home, &["add", "fixture"]);
    fixture_git(
        &root,
        &home,
        &[
            "-c",
            "user.name=Synthetic",
            "-c",
            "user.email=synthetic@example.invalid",
            "commit",
            "-m",
            "fixture",
        ],
    );
    let revision = fixture_git(&root, &home, &["rev-parse", "HEAD"]);
    std::fs::write(directory.join("revision"), &revision).unwrap();
    let identity =
        serde_json::to_string(&(root.join(".git").canonicalize().unwrap(), vec![revision]))
            .unwrap();
    std::fs::write(directory.join("identity"), identity).unwrap();
    for (path, marker) in [
        (bin.join("git"), directory.join("git-effect")),
        (
            directory.join("codex-sentinel"),
            directory.join("codex-effect"),
        ),
    ] {
        // Finite builtin-only leaf: a gate mutant cannot start real native/auth/model work.
        std::fs::write(
            &path,
            format!(
                "#!/bin/sh\nprintf invoked > '{}'\nexit 1\n",
                marker.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    // No parent environment mutation. This finite test child is the actual owner;
    // any gate mutant's Git sentinel is awaited/cleaned by the real library helper.
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "ordinary_empty_library_child",
            "--ignored",
            "--nocapture",
        ])
        .env_clear()
        .env("HOME", &home)
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .env("RRX_EMPTY_FIXTURE", &directory)
        .spawn()
        .unwrap();
    let pid = rustix::process::Pid::from_raw(child.id() as i32).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    let mut timed_out = false;
    let mut observation_uncertain = false;
    loop {
        match rustix::process::waitid(
            rustix::process::WaitId::Pid(pid),
            rustix::process::WaitIdOptions::EXITED
                | rustix::process::WaitIdOptions::NOHANG
                | rustix::process::WaitIdOptions::NOWAIT,
        ) {
            Ok(Some(_)) => break,
            Ok(None) => {}
            Err(_) => observation_uncertain = true,
        }
        if std::time::Instant::now() >= deadline && !timed_out {
            timed_out = true;
            // Actual Child handle only, never a saved/telemetry PID. A failed
            // kill/observation retains this Child until a real exit is observed.
            if child.kill().is_err() {
                observation_uncertain = true;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let status = child.wait().unwrap();
    assert!(
        !timed_out && !observation_uncertain,
        "ordinary harness timeout/observation failure; no acceptance"
    );
    assert!(
        status.success(),
        "ordinary library child failed; preserve its effect evidence"
    );
    assert!(!directory.join("codex-effect").exists());
    assert!(!directory.join("git-effect").exists());
}
