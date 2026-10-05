//! Actual built-binary/IPC configuration binding with a synthetic Docker CLI.
//! No Docker daemon, provider, authentication file or unrelated repository is used.
use rrx::{
    config::Config,
    domain::{CompletionCriterion, Goal, Task},
    execution::{RuntimeOwner, attempts::AttemptManager, ipc::ToolServer},
    project::{AddProject, ProjectRegistry},
};
use serde_json::json;
use std::{
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
    time::Duration,
};
use tokio::process::Command;

#[test]
fn installed_docker_entry_binds_runtime_and_shim_configuration() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let tools = root.join("tools");
    std::fs::create_dir(&tools).unwrap();
    let docker = tools.join("docker");
    std::fs::write(&docker, include_str!("../src/execution/docker/fixture.py")).unwrap();
    std::fs::set_permissions(&docker, std::fs::Permissions::from_mode(0o700)).unwrap();
    for name in ["config-a", "config-b"] {
        std::fs::create_dir(root.join(name)).unwrap();
    }
    let mut paths = vec![tools];
    paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "installed_docker_routing_worker",
            "--nocapture",
        ])
        .env("RRX_DOCKER_FIXTURE_ROOT", &root)
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env("DOCKER_CONFIG", root.join("config-a"))
        .env_remove("DOCKER_HOST")
        .env_remove("DOCKER_CONTEXT")
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "fixture worker failed: {}\n{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
}

async fn git(root: &Path, args: &[&str]) {
    let output = Command::new("/usr/bin/git")
        .current_dir(root)
        .args(args)
        .output()
        .await
        .unwrap();
    assert!(output.status.success(), "fixture Git failed");
}

#[tokio::test]
#[ignore = "child-only fixture with isolated command environment"]
async fn installed_docker_routing_worker() {
    let root = std::path::PathBuf::from(std::env::var_os("RRX_DOCKER_FIXTURE_ROOT").unwrap());
    let source = root.join("repo");
    std::fs::create_dir(&source).unwrap();
    git(&source, &["init", "-b", "main"]).await;
    std::fs::write(source.join("answer.txt"), "base\n").unwrap();
    git(&source, &["add", "answer.txt"]).await;
    git(
        &source,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            "base",
        ],
    )
    .await;
    let state = root.join("state.db");
    let owner = RuntimeOwner::open(&state).unwrap();
    let task = {
        let shared = owner.store();
        let mut store = shared.lock().unwrap();
        let project = ProjectRegistry::new(&mut store)
            .add(&source, AddProject::default(), &Config::default())
            .unwrap();
        let mut goal = Goal::new(
            project.id,
            "fixture".into(),
            vec![CompletionCriterion {
                id: "answer".into(),
                description: "configuration binding".into(),
                evidence: None,
                satisfied: false,
            }],
        );
        store.put_goal(&mut goal).unwrap();
        let mut task = Task::new(project.id, goal.id, "managed Docker".into(), "codex".into());
        store.put_task(&mut task).unwrap();
        task
    };
    let (unit, profile) = AttemptManager::new(owner.clone())
        .prepare(task.id, "codex", "Implement", None)
        .await
        .unwrap();
    let fake_state = root.join("tools/docker.json");
    let a = root.join("config-a");
    let b = root.join("config-b");
    std::fs::write(
        &fake_state,
        serde_json::to_vec(&json!({
            "database":state,"unit":unit.id,"cookie":unit.cookie,"calls":[],
            "containers":{},"networks":{},"volumes":{},
            "engine_by_config":{a.to_str().unwrap():"engine-a",b.to_str().unwrap():"engine-b"}
        }))
        .unwrap(),
    )
    .unwrap();
    let shim = profile.tool_bin.join("docker");
    std::fs::remove_file(&shim).unwrap();
    symlink(env!("CARGO_BIN_EXE_rrx"), &shim).unwrap();
    let _server = ToolServer::start(owner.clone()).unwrap();
    let read = || {
        serde_json::from_slice::<serde_json::Value>(&std::fs::read(&fake_state).unwrap()).unwrap()
    };
    // Both synthetic config roots contain a context named default. The managed
    // caller must be refused before even Runtime qualification probes occur.
    let mut different = Command::new(&shim);
    different
        .current_dir(&unit.worktree)
        .args(["run", "-d", "fixture-image"])
        .envs(profile.environment(&unit.cookie, owner.ipc_path()).unwrap())
        .env("DOCKER_CONFIG", &b);
    let refused = tokio::time::timeout(Duration::from_secs(15), different.output())
        .await
        .unwrap()
        .unwrap();
    assert!(!refused.status.success());
    assert!(read()["calls"].as_array().unwrap().is_empty());
    assert!(read()["containers"].as_object().unwrap().is_empty());
    let mut host = Command::new(&shim);
    host.current_dir(&unit.worktree)
        .args(["run", "-d", "fixture-image"])
        .envs(profile.environment(&unit.cookie, owner.ipc_path()).unwrap())
        .env("DOCKER_HOST", "unix:///fixture-different.sock");
    assert!(
        !tokio::time::timeout(Duration::from_secs(15), host.output())
            .await
            .unwrap()
            .unwrap()
            .status
            .success()
    );
    assert!(read()["calls"].as_array().unwrap().is_empty());
    let mut matching = Command::new(&shim);
    matching
        .current_dir(&unit.worktree)
        .args(["run", "-d", "fixture-image"])
        .envs(profile.environment(&unit.cookie, owner.ipc_path()).unwrap());
    let accepted = tokio::time::timeout(Duration::from_secs(15), matching.output())
        .await
        .unwrap()
        .unwrap();
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    let value = read();
    let containers = value["containers"].as_object().unwrap();
    assert_eq!(containers.len(), 1);
    let container = containers.values().next().unwrap();
    assert_eq!(container["engine"], "engine-a");
    assert_eq!(
        container["labels"]["org.rururunx.unit"],
        unit.id.to_string()
    );
    let connection = rusqlite::Connection::open(&state).unwrap();
    let body: String = connection.query_row("SELECT body FROM managed_effects WHERE unit_id=?1 AND json_extract(body,'$.kind')='docker_target'",
        [unit.id.to_string()], |row|row.get(0)).unwrap();
    let target: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(target["state"], "confirmed");
    assert_eq!(target["receipt"]["context"], "default");
    let body: String = connection.query_row("SELECT body FROM managed_effects WHERE unit_id=?1 AND json_extract(body,'$.kind')='docker_create'",
        [unit.id.to_string()], |row|row.get(0)).unwrap();
    let creation: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(creation["id"], container["operation"]);
    assert_eq!(creation["expected_target"], container["name"]);
}
