//! Actual built-binary mediation with account-free Git hooks. This is neither
//! an authenticated Agent test nor evidence of native sandbox compatibility.
use rrx::{
    config::Config,
    domain::{CompletionCriterion, Goal, Task},
    execution::{
        RuntimeOwner, attempts::AttemptManager, ipc::ToolServer, resources::ResourceProfile,
    },
    project::{AddProject, ProjectRegistry},
};
use std::{
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
    time::Duration,
};
use tokio::process::Command;

async fn git(path: &Path, args: &[&str]) -> String {
    let output = Command::new("/usr/bin/git")
        .current_dir(path)
        .args(args)
        .kill_on_drop(true)
        .output()
        .await
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
async fn managed(
    owner: &RuntimeOwner,
    unit: &rrx::execution::ExecutionUnit,
    profile: &ResourceProfile,
    foreign: &ResourceProfile,
    args: &[&str],
) -> std::process::Output {
    // Repoint only this owned fixture's shim link to the real Cargo-built rrx.
    // Production materialization uses the installed current executable.
    let link = profile.tool_bin.join("git");
    if std::fs::read_link(&link).unwrap() != Path::new(env!("CARGO_BIN_EXE_rrx")) {
        std::fs::remove_file(&link).unwrap();
        symlink(env!("CARGO_BIN_EXE_rrx"), &link).unwrap();
    }
    let mut command = Command::new(&link);
    command
        .current_dir(&unit.worktree)
        .args(args)
        .envs(profile.environment(&unit.cookie, owner.ipc_path()).unwrap())
        .env("TMPDIR", &foreign.temp)
        .env("CARGO_TARGET_DIR", foreign.output.join("cargo-target"))
        .env_remove("GIT_INDEX_FILE")
        .env_remove("RRX_GIT_GATE_TOKEN")
        .kill_on_drop(true);
    tokio::time::timeout(Duration::from_secs(20), command.output())
        .await
        .expect("managed Git must terminate")
        .unwrap()
}

#[tokio::test]
async fn installed_entry_preserves_native_candidate_index_and_owner_resources() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("repo");
    std::fs::create_dir(&source).unwrap();
    git(&source, &["init", "-b", "main"]).await;
    for (key, value) in [
        ("user.name", "Fixture"),
        ("user.email", "fixture@example.invalid"),
        ("commit.gpgsign", "false"),
    ] {
        git(&source, &["config", key, value]).await;
    }
    std::fs::write(source.join("answer.txt"), "SAFE\n").unwrap();
    git(&source, &["add", "answer.txt"]).await;
    git(&source, &["commit", "-m", "base"]).await;
    let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
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
                evaluator: Default::default(),
                id: "answer".into(),
                description: "exact candidate".into(),
                evidence: None,
                satisfied: false,
            }],
        );
        store.put_goal(&mut goal).unwrap();
        let mut task = Task::new(project.id, goal.id, "candidate".into(), "codex".into());
        store.put_task(&mut task).unwrap();
        task
    };
    let attempts = AttemptManager::new(owner.clone());
    let (unit, profile) = attempts
        .prepare(task.id, "codex", "Implement", None)
        .await
        .unwrap();
    let mut sibling = Task::new(
        task.project_id,
        task.goal_id,
        "sibling".into(),
        "codex".into(),
    );
    owner
        .store()
        .lock()
        .unwrap()
        .put_task(&mut sibling)
        .unwrap();
    let (sibling_unit, sibling_profile) = attempts
        .prepare(sibling.id, "codex", "Implement", None)
        .await
        .unwrap();
    let _server = ToolServer::start(owner.clone()).unwrap();
    let hook = source.join(".git/hooks/pre-commit");
    std::fs::write(
        &hook,
        r#"#!/usr/bin/python3
import json, os, subprocess, sys, tempfile
with tempfile.NamedTemporaryFile(dir=os.environ['TMPDIR']) as temporary:
    marker = {'tmp':os.environ['TMPDIR'], 'target':os.environ['CARGO_TARGET_DIR'],
              'index':os.environ.get('GIT_INDEX_FILE'), 'temporary':temporary.name}
    with open(os.path.join(os.environ['RRX_OUTPUT_DIR'],'hook-context.json'),'w') as result:
        json.dump(marker,result)
    diff = subprocess.run(['git','diff','--cached'],capture_output=True)
    if diff.returncode: sys.exit(2)
    if b'+FORBIDDEN' in diff.stdout: sys.exit(1)
"#,
    )
    .unwrap();
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(unit.worktree.join("answer.txt"), "FORBIDDEN\n").unwrap();
    let rejected = managed(
        &owner,
        &unit,
        &profile,
        &sibling_profile,
        &["commit", "-a", "-m", "must reject"],
    )
    .await;
    assert!(
        !rejected.status.success(),
        "the hook must inspect the candidate index"
    );
    assert_eq!(
        git(&unit.worktree, &["rev-parse", "HEAD"]).await,
        unit.base_sha
    );
    let marker: serde_json::Value =
        serde_json::from_slice(&std::fs::read(profile.output.join("hook-context.json")).unwrap())
            .unwrap();
    assert_eq!(marker["tmp"], profile.temp.to_str().unwrap());
    assert_eq!(
        marker["target"],
        profile.output.join("cargo-target").to_str().unwrap()
    );
    assert!(marker["index"].as_str().unwrap().ends_with("index.lock"));
    assert!(Path::new(marker["temporary"].as_str().unwrap()).starts_with(&profile.temp));
    std::fs::write(unit.worktree.join("answer.txt"), "SAFE accepted\n").unwrap();
    let accepted = managed(
        &owner,
        &unit,
        &profile,
        &sibling_profile,
        &["commit", "-a", "-m", "accepted candidate"],
    )
    .await;
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    assert_eq!(
        git(&unit.worktree, &["show", "HEAD:answer.txt"]).await,
        "SAFE accepted"
    );
    // Partial commits use a different native candidate-index form.
    std::fs::write(unit.worktree.join("answer.txt"), "FORBIDDEN\n").unwrap();
    let partial = managed(
        &owner,
        &unit,
        &profile,
        &sibling_profile,
        &["commit", "-m", "partial must reject", "--", "answer.txt"],
    )
    .await;
    assert!(!partial.status.success());
    let marker: serde_json::Value =
        serde_json::from_slice(&std::fs::read(profile.output.join("hook-context.json")).unwrap())
            .unwrap();
    assert!(
        Path::new(marker["index"].as_str().unwrap())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("next-index-")
    );
    std::fs::write(unit.worktree.join("answer.txt"), "SAFE partial\n").unwrap();
    let partial = managed(
        &owner,
        &unit,
        &profile,
        &sibling_profile,
        &["commit", "-m", "accepted partial", "--", "answer.txt"],
    )
    .await;
    assert!(
        partial.status.success(),
        "{}",
        String::from_utf8_lossy(&partial.stderr)
    );
    assert_eq!(
        git(&unit.worktree, &["show", "HEAD:answer.txt"]).await,
        "SAFE partial"
    );
    assert_eq!(
        git(&sibling_unit.worktree, &["rev-parse", "HEAD"]).await,
        sibling_unit.base_sha
    );
    assert_eq!(std::fs::read_dir(&sibling_profile.temp).unwrap().count(), 0);
    assert_eq!(
        std::fs::read_dir(&sibling_profile.output).unwrap().count(),
        0
    );
    // The executor's branch binding stays usable: detached transitions refuse
    // before effects; ordinary status and the sibling continue afterwards.
    let detached = managed(
        &owner,
        &unit,
        &profile,
        &sibling_profile,
        &["checkout", "--detach", &unit.base_sha],
    )
    .await;
    assert!(!detached.status.success());
    assert_eq!(
        git(&unit.worktree, &["branch", "--show-current"]).await,
        unit.branch.as_deref().unwrap()
    );
    assert!(
        managed(
            &owner,
            &unit,
            &profile,
            &sibling_profile,
            &["status", "--porcelain"]
        )
        .await
        .status
        .success()
    );
}
