use rrx::{domain::*, git::WorktreeManager as Manager, state::Store};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::TempDir;

fn git(path: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{:?}: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}
struct Fixture {
    _temp: TempDir,
    root: PathBuf,
    store: Store,
    project: Project,
    goal: Goal,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap().join("repo");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "-b", "main"]);
        git(&root, &["config", "user.email", "fixture@example.invalid"]);
        git(&root, &["config", "user.name", "Fixture"]);
        // Isolate fixtures from developer global hooks/signing while production retains them.
        git(&root, &["config", "commit.gpgsign", "false"]);
        git(&root, &["config", "core.hooksPath", ".git/hooks"]);
        std::fs::write(root.join("tracked.txt"), "baseline\n").unwrap();
        git(&root, &["add", "tracked.txt"]);
        git(&root, &["commit", "-m", "fixture"]);
        let mut store = Store::open(&temp.path().join("state.db")).unwrap();
        let mut project = Project::new(
            "fixture".into(),
            root.clone(),
            rrx::git::repository_identity(&root, "main").unwrap(),
            "main".into(),
        );
        store.put_project(&mut project).unwrap();
        let mut goal = Goal::new(
            project.id,
            "fixture".into(),
            vec![CompletionCriterion {
                id: "done".into(),
                description: "fixture".into(),
                evidence: None,
                satisfied: false,
            }],
        );
        store.put_goal(&mut goal).unwrap();
        Self {
            _temp: temp,
            root,
            store,
            project,
            goal,
        }
    }
    fn task(&mut self, issue: u64) -> Task {
        let mut task = Task::new(
            self.project.id,
            self.goal.id,
            "fixture".into(),
            "fake".into(),
        );
        task.issue = Some(issue);
        self.store.put_task(&mut task).unwrap();
        task
    }
    fn session(&self, task: &Task, state: SessionState) -> Session {
        Session {
            id: SessionId::new(),
            scope: task.scope(),
            agent: "fake".into(),
            provider: "fake".into(),
            role: SessionRole::Executor,
            native_ref: None,
            pid: None,
            worktree: task.worktree.clone().unwrap(),
            state,
            model: None,
            effort: None,
            recovery: json!({}),
            started_at: now_ms(),
        }
    }
}
#[test]
fn independent_tasks_and_project_local_issue_numbers() {
    let mut a = Fixture::new();
    let mut b = Fixture::new();
    let t1 = a.task(42);
    let t2 = a.task(43);
    let t3 = b.task(42);
    let s1 = Manager::create(&mut a.store, t1.id).unwrap();
    let s2 = Manager::create(&mut a.store, t2.id).unwrap();
    let s3 = Manager::create(&mut b.store, t3.id).unwrap();
    assert_ne!(s1.worktree, s2.worktree);
    assert_ne!(s1.worktree, s3.worktree);
    assert_eq!(s1.branch, s3.branch);
    assert!(!s1.dirty);
    assert_eq!(s1.worktree, a.root.join("worktree/issue-42"));
    let mut stored = a.store.task(t1.id).unwrap().unwrap();
    stored.worktree = Some(s3.worktree);
    assert!(a.store.put_task(&mut stored).is_err());
}
#[test]
fn dirty_tracked_untracked_and_ignored_files_block_review_cleanup() {
    let mut f = Fixture::new();
    let task = f.task(1);
    let s = Manager::create(&mut f.store, task.id).unwrap();
    for path in ["tracked.txt", "extra.txt"] {
        std::fs::write(s.worktree.join(path), "dirty").unwrap();
        assert!(Manager::status(&f.store, task.id).unwrap().dirty);
        assert!(Manager::lock_review(&mut f.store, task.id, &s.revision, "review").is_err());
        assert!(Manager::cleanup(&mut f.store, task.id).is_err());
        if path == "tracked.txt" {
            git(&s.worktree, &["restore", path]);
        } else {
            std::fs::remove_file(s.worktree.join(path)).unwrap();
        }
    }
    std::fs::write(f.root.join(".git/info/exclude"), "ignored.txt\n").unwrap();
    std::fs::write(s.worktree.join("ignored.txt"), "keep").unwrap();
    assert!(Manager::status(&f.store, task.id).unwrap().dirty);
    assert!(Manager::cleanup(&mut f.store, task.id).is_err());
    assert!(s.worktree.join("ignored.txt").exists());
}
#[test]
fn review_locks_block_executor_and_detect_external_changes() {
    let mut f = Fixture::new();
    let t = f.task(1);
    let s = Manager::create(&mut f.store, t.id).unwrap();
    let task = f.store.task(t.id).unwrap().unwrap();
    assert!(Manager::lock_review(&mut f.store, t.id, "wrong", "review").is_err());
    let lock = Manager::lock_review(&mut f.store, t.id, &s.revision, "review").unwrap();
    assert!(Manager::ensure_mutation_allowed(&f.store, t.id).is_err());
    assert!(
        f.store
            .put_session(&f.session(&task, SessionState::Starting), 0)
            .is_err()
    );
    assert!(Manager::lock_review(&mut f.store, t.id, &s.revision, "second").is_err());
    assert!(Manager::cleanup(&mut f.store, t.id).is_err());
    std::fs::write(s.worktree.join("external.txt"), "dirty").unwrap();
    assert!(Manager::verify_review(&f.store, lock).is_err());
    assert!(Manager::unlock_review(&mut f.store, lock).is_err());
    std::fs::remove_file(s.worktree.join("external.txt")).unwrap();
    Manager::unlock_review(&mut f.store, lock).unwrap();
    assert!(Manager::ensure_mutation_allowed(&f.store, t.id).is_ok());
}
#[test]
fn reservations_and_locks_serialize_across_connections() {
    let mut f = Fixture::new();
    let t = f.task(1);
    let s = Manager::create(&mut f.store, t.id).unwrap();
    let task = f.store.task(t.id).unwrap().unwrap();
    let mut second = Store::open(&f._temp.path().join("state.db")).unwrap();
    let mut session = f.session(&task, SessionState::Starting);
    let version = second.put_session(&session, 0).unwrap();
    assert!(Manager::lock_review(&mut f.store, t.id, &s.revision, "review").is_err());
    assert!(Manager::cleanup(&mut f.store, t.id).is_err());
    session.state = SessionState::Lost;
    let version = second.put_session(&session, version).unwrap();
    assert!(Manager::lock_review(&mut f.store, t.id, &s.revision, "review").is_err());
    session.state = SessionState::Stopped;
    session.recovery = json!({"verified_dead":true});
    second.put_session(&session, version).unwrap();
    let lock = Manager::lock_review(&mut f.store, t.id, &s.revision, "review").unwrap();
    assert!(
        second
            .put_session(&f.session(&task, SessionState::Running), 0)
            .is_err()
    );
    Manager::unlock_review(&mut f.store, lock).unwrap();
}
#[test]
fn rejects_main_master_detached_and_mismatched_branch() {
    let mut f = Fixture::new();
    let t = f.task(1);
    let s = Manager::create(&mut f.store, t.id).unwrap();
    git(&s.worktree, &["switch", "-c", "master"]);
    assert!(Manager::ensure_mutation_allowed(&f.store, t.id).is_err());
    git(&s.worktree, &["switch", "--detach"]);
    assert!(Manager::ensure_mutation_allowed(&f.store, t.id).is_err());
    git(&s.worktree, &["switch", "-c", "other"]);
    assert!(Manager::ensure_mutation_allowed(&f.store, t.id).is_err());
    // main is already checked out at root; task may not point at that root either.
    let mut new = f.task(2);
    new.worktree = Some(f.root.clone());
    new.branch = Some("main".into());
    assert!(f.store.put_task(&mut new).is_err());
}
#[test]
fn rejects_foreign_repository_and_existing_path_or_branch() {
    let mut a = Fixture::new();
    let mut b = Fixture::new();
    let t = a.task(1);
    let bt = b.task(1);
    let bs = Manager::create(&mut b.store, bt.id).unwrap();
    std::fs::create_dir_all(a.root.join("worktree")).unwrap();
    std::fs::rename(&bs.worktree, a.root.join("worktree/issue-1")).unwrap();
    assert!(Manager::create(&mut a.store, t.id).is_err());
    let mut forged = a.task(2);
    forged.worktree = Some(a.root.join("worktree/issue-1"));
    forged.branch = Some(bs.branch);
    a.store.put_task(&mut forged).unwrap();
    assert!(Manager::status(&a.store, forged.id).is_err());
    git(&a.root, &["branch", "feature/issue-3"]);
    let duplicate = a.task(3);
    assert!(Manager::create(&mut a.store, duplicate.id).is_err());
}
#[test]
fn cleanup_rejects_unmerged_then_removes_only_owned_merged_tree() {
    let mut f = Fixture::new();
    let t = f.task(1);
    let other = f.task(2);
    let s = Manager::create(&mut f.store, t.id).unwrap();
    let other = Manager::create(&mut f.store, other.id).unwrap();
    std::fs::write(s.worktree.join("change.txt"), "change").unwrap();
    git(&s.worktree, &["add", "change.txt"]);
    git(&s.worktree, &["commit", "-m", "change"]);
    assert!(Manager::cleanup(&mut f.store, t.id).is_err());
    assert!(s.worktree.exists());
    git(&f.root, &["merge", "--ff-only", &s.branch]);
    Manager::cleanup(&mut f.store, t.id).unwrap();
    assert!(!s.worktree.exists());
    assert!(other.worktree.exists());
    assert!(f.root.join("change.txt").exists());
    let out = Command::new("git")
        .current_dir(&f.root)
        .args([
            "show-ref",
            "--quiet",
            "--verify",
            &format!("refs/heads/{}", s.branch),
        ])
        .status()
        .unwrap();
    assert!(!out.success());
    assert!(
        f.store
            .events(&t.scope(), 0, 100)
            .unwrap()
            .iter()
            .any(|e| e.kind == "worktree.cleaned")
    );
}
#[cfg(unix)]
#[test]
fn symlinks_and_native_hook_failures_are_not_bypassed() {
    use std::os::unix::{fs::PermissionsExt, fs::symlink};
    let mut f = Fixture::new();
    let t = f.task(1);
    let outside = f._temp.path().canonicalize().unwrap().join("outside");
    std::fs::create_dir(&outside).unwrap();
    symlink(&outside, f.root.join("worktree")).unwrap();
    assert!(Manager::create(&mut f.store, t.id).is_err());
    assert!(std::fs::read_dir(&outside).unwrap().next().is_none());
    std::fs::remove_file(f.root.join("worktree")).unwrap();
    let hook = f.root.join(".git/hooks/post-checkout");
    std::fs::write(
        &hook,
        "#!/bin/sh\nprintf invoked > hook-evidence.txt\nexit 7\n",
    )
    .unwrap();
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(Manager::create(&mut f.store, t.id).is_err());
    let task = f.store.task(t.id).unwrap().unwrap();
    let path = task.worktree.unwrap();
    assert!(path.join("hook-evidence.txt").exists());
    assert!(path.exists());
    assert!(Manager::ensure_mutation_allowed(&f.store, t.id).is_err());
    assert!(
        f.store
            .records(&t.scope(), RecordKind::WorktreeLock)
            .unwrap()
            .iter()
            .any(|r| r.data["active"] == true)
    );
}

#[test]
fn immutable_review_detects_clean_head_change() {
    let mut f = Fixture::new();
    let t = f.task(1);
    let s = Manager::create(&mut f.store, t.id).unwrap();
    let lock = Manager::lock_review(&mut f.store, t.id, &s.revision, "review").unwrap();
    git(
        &s.worktree,
        &["commit", "--allow-empty", "-m", "external head change"],
    );
    assert!(!Manager::status(&f.store, t.id).unwrap().dirty);
    assert!(Manager::verify_review(&f.store, lock).is_err());
    assert!(Manager::unlock_review(&mut f.store, lock).is_err());
}
#[test]
fn protected_base_and_git_metadata_namespace_are_rejected() {
    let mut f = Fixture::new();
    let t = f.task(1);
    let s = Manager::create(&mut f.store, t.id).unwrap();
    f.project.base_branch = s.branch;
    f.store.put_project(&mut f.project).unwrap();
    assert!(Manager::ensure_mutation_allowed(&f.store, t.id).is_err());
    f.project.base_branch = "main".into();
    f.project.worktree_root = f.root.join(".git/nested");
    assert!(f.store.put_project(&mut f.project).is_err());
    let mut task = f.store.task(t.id).unwrap().unwrap();
    task.state = TaskState::Completed;
    f.store.put_task(&mut task).unwrap();
    let mut fresh = Fixture::new();
    fresh.project.worktree_root = fresh.root.join(".git/nested");
    fresh.store.put_project(&mut fresh.project).unwrap();
    let new = fresh.task(2);
    assert!(Manager::create(&mut fresh.store, new.id).is_err());
    assert!(!fresh.root.join(".git/nested").exists());
}
#[test]
fn duplicate_executors_and_malformed_locks_fail_closed() {
    let mut f = Fixture::new();
    let t = f.task(1);
    Manager::create(&mut f.store, t.id).unwrap();
    let task = f.store.task(t.id).unwrap().unwrap();
    let mut first = f.session(&task, SessionState::Starting);
    let version = f.store.put_session(&first, 0).unwrap();
    assert!(
        f.store
            .put_session(&f.session(&task, SessionState::Starting), 0)
            .is_err()
    );
    first.state = SessionState::Stopped;
    f.store.put_session(&first, version).unwrap();
    let mut malformed = Record::new(t.scope(), RecordKind::WorktreeLock, json!({"active":false}));
    assert!(f.store.put_record(&mut malformed).is_err());
}

#[test]
fn aliased_worktree_bindings_and_goal_scoped_executors_are_rejected() {
    let mut f = Fixture::new();
    let t = f.task(1);
    let status = Manager::create(&mut f.store, t.id).unwrap();
    let mut duplicate = f.task(1);
    duplicate.worktree = Some(status.worktree.clone());
    duplicate.branch = Some(status.branch.clone());
    assert!(
        f.store.put_task(&mut duplicate).is_err(),
        "duplicate worktree ownership accepted"
    );
    let task = f.store.task(t.id).unwrap().unwrap();
    let mut session = f.session(&task, SessionState::Starting);
    session.scope = Scope::goal(task.project_id, task.goal_id);
    assert!(
        f.store.put_session(&session, 0).is_err(),
        "goal-scoped executor accepted"
    );
    session.scope = task.scope();
    session.worktree = f.root.clone();
    assert!(
        f.store.put_session(&session, 0).is_err(),
        "executor cwd differs from owned worktree"
    );
}
#[test]
fn cleanup_checks_native_branch_delete_predicate_before_removal() {
    let mut f = Fixture::new();
    let t = f.task(1);
    let s = Manager::create(&mut f.store, t.id).unwrap();
    git(&f.root, &["branch", "old"]);
    git(&s.worktree, &["commit", "--allow-empty", "-m", "change"]);
    git(&f.root, &["merge", "--ff-only", &s.branch]);
    git(&f.root, &["switch", "old"]);
    assert!(Manager::cleanup(&mut f.store, t.id).is_err());
    assert!(
        s.worktree.exists(),
        "worktree deleted before native branch predicate checked"
    );
    git(&f.root, &["switch", "main"]);
    git(&s.worktree, &["branch", "--set-upstream-to=old"]);
    assert!(Manager::cleanup(&mut f.store, t.id).is_err());
    assert!(s.worktree.exists());
}
#[test]
fn linked_and_wrong_identity_project_roots_are_rejected() {
    let mut f = Fixture::new();
    let t = f.task(1);
    let s = Manager::create(&mut f.store, t.id).unwrap();
    let mut nested = Project::new("nested".into(), s.worktree, "nested".into(), "main".into());
    assert!(
        f.store.put_project(&mut nested).is_err(),
        "overlapping Project root accepted"
    );
    // A separate Store must also reject treating a linked worktree as a source repository.
    let mut separate = Store::memory().unwrap();
    separate.put_project(&mut nested).unwrap();
    let mut goal = Goal::new(
        nested.id,
        "fixture".into(),
        f.goal.completion_criteria.clone(),
    );
    separate.put_goal(&mut goal).unwrap();
    let mut task = Task::new(nested.id, goal.id, "fixture".into(), "fake".into());
    separate.put_task(&mut task).unwrap();
    assert!(Manager::create(&mut separate, task.id).is_err());
}

#[test]
fn concurrent_duplicate_issue_creation_has_one_durable_owner() {
    use std::sync::{Arc, Barrier};
    let mut f = Fixture::new();
    let a = f.task(7);
    let b = f.task(7);
    let barrier = Arc::new(Barrier::new(2));
    let db = f._temp.path().join("state.db");
    let handles: Vec<_> = [a.id, b.id]
        .into_iter()
        .map(|id| {
            let barrier = barrier.clone();
            let db = db.clone();
            std::thread::spawn(move || {
                let mut store = Store::open(&db).unwrap();
                barrier.wait();
                Manager::create(&mut store, id).is_ok()
            })
        })
        .collect();
    let successes = handles
        .into_iter()
        .map(|h| usize::from(h.join().unwrap()))
        .sum::<usize>();
    assert_eq!(successes, 1);
    let tasks = f.store.tasks(f.project.id, None).unwrap();
    assert_eq!(tasks.iter().filter(|t| t.worktree.is_some()).count(), 1);
}
#[test]
fn qualified_base_ref_wins_over_same_named_tag_and_replacement_is_blocked() {
    let mut f = Fixture::new();
    git(&f.root, &["tag", "main"]);
    git(&f.root, &["commit", "--allow-empty", "-m", "base changed"]);
    let t = f.task(1);
    let s = Manager::create(&mut f.store, t.id).unwrap();
    let out = Command::new("git")
        .current_dir(&f.root)
        .args(["rev-parse", "refs/heads/main"])
        .output()
        .unwrap();
    assert_eq!(s.revision, String::from_utf8(out.stdout).unwrap().trim());
    // Replacing the source checkout with unrelated history must not silently rebind Project.
    let old = f.root.with_file_name("old-repo");
    std::fs::rename(&f.root, &old).unwrap();
    std::fs::create_dir(&f.root).unwrap();
    git(&f.root, &["init", "-b", "main"]);
    git(&f.root, &["config", "user.name", "Replacement"]);
    git(
        &f.root,
        &["config", "user.email", "replacement@example.invalid"],
    );
    git(&f.root, &["config", "commit.gpgsign", "false"]);
    git(&f.root, &["commit", "--allow-empty", "-m", "unrelated"]);
    let other = f.task(2);
    assert!(Manager::create(&mut f.store, other.id).is_err());
}

#[test]
fn cleanup_after_upstream_tracking_ref_is_pruned() {
    let mut f = Fixture::new();
    let t = f.task(1);
    let s = Manager::create(&mut f.store, t.id).unwrap();
    git(
        &f.root,
        &[
            "config",
            "remote.origin.url",
            "https://example.invalid/fixture.git",
        ],
    );
    git(
        &f.root,
        &[
            "config",
            "remote.origin.fetch",
            "+refs/heads/*:refs/remotes/origin/*",
        ],
    );
    let tracking = format!("refs/remotes/origin/{}", s.branch);
    git(&f.root, &["update-ref", &tracking, &s.revision]);
    git(
        &s.worktree,
        &[
            "branch",
            "--set-upstream-to",
            &format!("origin/{}", s.branch),
        ],
    );
    git(&f.root, &["update-ref", "-d", &tracking]);
    Manager::cleanup(&mut f.store, t.id).unwrap();
    assert!(!s.worktree.exists());
}
