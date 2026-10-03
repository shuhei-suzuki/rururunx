use rrx::{
    config::Config,
    domain::*,
    git::WorktreeManager,
    project::{
        AddProject, ProjectRegistry as Registry, effective_config, environment_names, scoped_file,
    },
    state::Store,
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};
use tempfile::TempDir;
fn git(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
struct Fixture {
    _tmp: TempDir,
    root: PathBuf,
    a: PathBuf,
    b: PathBuf,
    db: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let a = root.join("a");
        let b = root.join("b");
        for repo in [&a, &b] {
            std::fs::create_dir(repo).unwrap();
            git(repo, &["init", "-b", "main"]);
            git(repo, &["config", "user.email", "fixture@example.invalid"]);
            git(repo, &["config", "user.name", "Fixture"]);
            git(repo, &["config", "commit.gpgsign", "false"]);
            git(repo, &["config", "core.hooksPath", ".git/hooks"]);
            std::fs::write(
                repo.join("rules.md"),
                repo.file_name().unwrap().to_str().unwrap(),
            )
            .unwrap();
            git(repo, &["add", "rules.md"]);
            git(repo, &["commit", "-m", "fixture"]);
        }
        let db = root.join("state.db");
        Self {
            _tmp: tmp,
            root,
            a,
            b,
            db,
        }
    }
    fn cli(&self, cwd: &Path, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_rrx"))
            .current_dir(cwd)
            .arg("--state")
            .arg(&self.db)
            .args(args)
            .output()
            .unwrap()
    }
    fn good(&self, args: &[&str]) -> String {
        let out = self.cli(&self.root, args);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }
    fn add(&self, store: &mut Store, root: &Path) -> Project {
        Registry::new(store)
            .add(root, AddProject::default(), &Config::default())
            .unwrap()
    }
}
fn goal(store: &mut Store, p: &Project) -> Goal {
    let mut g = Goal::new(
        p.id,
        "fixture".into(),
        vec![CompletionCriterion {
            id: "done".into(),
            description: "pass".into(),
            evidence: None,
            satisfied: false,
        }],
    );
    store.put_goal(&mut g).unwrap();
    g
}
fn task(store: &mut Store, p: &Project, g: &Goal) -> Task {
    let mut t = Task::new(p.id, g.id, "Issue 42".into(), "fake".into());
    t.issue = Some(42);
    store.put_task(&mut t).unwrap();
    t
}
#[test]
fn actual_cli_registry_restarts_recovers_and_soft_removes_without_deleting_files() {
    let f = Fixture::new();
    let aid = f
        .good(&["project", "add", f.a.to_str().unwrap()])
        .split('\t')
        .next()
        .unwrap()
        .to_owned();
    let bid = f
        .good(&["project", "add", f.b.to_str().unwrap()])
        .split('\t')
        .next()
        .unwrap()
        .to_owned();
    assert_ne!(aid, bid);
    assert!(
        f.good(&["project", "add", f.a.to_str().unwrap()])
            .starts_with(&aid)
    );
    let list: Value = serde_json::from_str(&f.good(&["project", "list", "--json"])).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 2);
    let out = f.cli(&f.a, &["project", "status", "--json"]);
    assert!(out.status.success());
    let status: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(status["project"]["id"], aid);
    std::fs::rename(&f.a, f.root.join("moved")).unwrap();
    let status: Value =
        serde_json::from_str(&f.good(&["project", "status", &aid, "--json"])).unwrap();
    assert_eq!(status["project"]["state"], "BLOCKED");
    assert!(
        status["project"]["blocked_reason"]
            .as_str()
            .unwrap()
            .contains("missing")
    );
    // A different repository at the old path must not acquire the existing ID.
    std::fs::rename(&f.b, &f.a).unwrap();
    assert!(
        !f.cli(&f.root, &["project", "add", f.a.to_str().unwrap()])
            .status
            .success()
    );
    std::fs::rename(&f.a, &f.b).unwrap();
    std::fs::rename(f.root.join("moved"), &f.a).unwrap();
    assert!(f.good(&["project", "status", &aid]).contains("Blocked"));
    assert!(
        f.good(&["project", "add", f.a.to_str().unwrap()])
            .starts_with(&aid)
    );
    let p: Value = serde_json::from_str(&f.good(&["project", "status", &aid, "--json"])).unwrap();
    assert_eq!(p["project"]["state"], "REGISTERED");
    f.good(&["project", "remove", &aid]);
    assert!(f.a.join("rules.md").exists());
    assert_eq!(
        serde_json::from_str::<Value>(&f.good(&["project", "list", "--json"]))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        serde_json::from_str::<Value>(&f.good(&["project", "list", "--all", "--json"]))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(
        f.good(&["project", "add", f.a.to_str().unwrap()])
            .starts_with(&aid)
    );
    let store = Store::open(&f.db).unwrap();
    assert!(
        store
            .events(&Scope::project(aid.parse().unwrap()), 0, 100)
            .unwrap()
            .len()
            >= 4
    );
}
#[test]
fn two_project_issue42_worktrees_context_and_goals_are_isolated() {
    let f = Fixture::new();
    let mut s = Store::open(&f.db).unwrap();
    let a = f.add(&mut s, &f.a);
    let b = f.add(&mut s, &f.b);
    let ga = goal(&mut s, &a);
    let gb = goal(&mut s, &b);
    let ta = task(&mut s, &a, &ga);
    let tb = task(&mut s, &b, &gb);
    let wa = WorktreeManager::create(&mut s, ta.id).unwrap();
    let wb = WorktreeManager::create(&mut s, tb.id).unwrap();
    assert_eq!(wa.branch, wb.branch);
    assert_ne!(wa.worktree, wb.worktree);
    let mut t = s.task(ta.id).unwrap().unwrap();
    t.worktree = Some(wb.worktree.clone());
    assert!(s.put_task(&mut t).is_err());
    let mut foreign = Task::new(a.id, gb.id, "foreign".into(), "fake".into());
    assert!(s.put_task(&mut foreign).is_err());
    let ctx = ContextVersion {
        scope: ta.scope(),
        version: 1,
        revision: wa.revision,
        source_hashes: Default::default(),
        data: json!({"rules":"A"}),
    };
    s.put_context(&ctx).unwrap();
    assert!(s.context(&tb.scope(), None).unwrap().is_none());
    let mut foreign_scope = ctx.clone();
    foreign_scope.scope = Scope::task(b.id, ga.id, ta.id);
    assert!(s.put_context(&foreign_scope).is_err());
    assert_eq!(
        Registry::new(&mut s)
            .resolve(None, &wa.worktree)
            .unwrap()
            .id,
        a.id
    );
    let unbound = a.worktree_root.join("unbound");
    std::fs::create_dir(&unbound).unwrap();
    assert!(Registry::new(&mut s).resolve(None, &unbound).is_err());
}
#[test]
fn selector_ambiguity_linked_roots_subdirectories_and_foreign_cwd_are_rejected() {
    let f = Fixture::new();
    let mut s = Store::open(&f.db).unwrap();
    let a = Registry::new(&mut s)
        .add(
            &f.a,
            AddProject {
                name: Some("same".into()),
                ..Default::default()
            },
            &Config::default(),
        )
        .unwrap();
    Registry::new(&mut s)
        .add(
            &f.b,
            AddProject {
                name: Some("same".into()),
                ..Default::default()
            },
            &Config::default(),
        )
        .unwrap();
    assert!(
        Registry::new(&mut s)
            .resolve(Some("same"), &f.root)
            .is_err()
    );
    assert_eq!(
        Registry::new(&mut s)
            .resolve(Some(&a.id.to_string()), &f.root)
            .unwrap()
            .id,
        a.id
    );
    std::fs::create_dir(f.a.join("sub")).unwrap();
    assert!(
        Registry::new(&mut s)
            .add(&f.a.join("sub"), AddProject::default(), &Config::default())
            .is_err()
    );
    git(
        &f.a,
        &[
            "worktree",
            "add",
            "-b",
            "linked",
            f.root.join("linked").to_str().unwrap(),
        ],
    );
    assert!(
        Registry::new(&mut s)
            .add(
                &f.root.join("linked"),
                AddProject::default(),
                &Config::default()
            )
            .is_err()
    );
    let nested = f.a.join("nested");
    std::fs::create_dir(&nested).unwrap();
    git(&nested, &["init", "-b", "main"]);
    assert!(Registry::new(&mut s).resolve(None, &nested).is_err());
    let bare = f.root.join("bare");
    git(&f.root, &["init", "--bare", bare.to_str().unwrap()]);
    assert!(
        Registry::new(&mut s)
            .add(&bare, AddProject::default(), &Config::default())
            .is_err()
    );
}
#[test]
fn config_rules_environment_and_symlinks_never_cross_project() {
    let f = Fixture::new();
    let mut s = Store::open(&f.db).unwrap();
    std::fs::write(
        f.a.join("project.toml"),
        "[scheduler]\nmax_tasks_per_project = 2\n[context]\nenabled = false",
    )
    .unwrap();
    let runtime = Config::default();
    let a = Registry::new(&mut s)
        .add(
            &f.a,
            AddProject {
                config_ref: Some("project.toml".into()),
                rule_refs: vec!["rules.md".into()],
                environment_refs: vec!["PROJECT_A_TOKEN".into()],
                ..Default::default()
            },
            &runtime,
        )
        .unwrap();
    let b = f.add(&mut s, &f.b);
    let ac = effective_config(&a, &runtime).unwrap();
    let bc = effective_config(&b, &runtime).unwrap();
    assert_eq!(ac.scheduler.max_tasks_per_project, 2);
    assert_eq!(ac.scheduler.global_max_sessions, 12);
    assert!(!ac.context.enabled);
    assert!(bc.context.enabled);
    assert_eq!(environment_names(&a).unwrap(), ["PROJECT_A_TOKEN"]);
    assert!(environment_names(&b).unwrap().is_empty());
    assert_eq!(a.rule_refs, [f.a.join("rules.md")]);
    assert!(b.rule_refs.is_empty());
    assert!(scoped_file(&a, &f.b.join("rules.md")).is_err());
    assert!(scoped_file(&a, Path::new("../b/rules.md")).is_err());
    std::os::unix::fs::symlink(f.b.join("rules.md"), f.a.join("foreign.md")).unwrap();
    assert!(
        Registry::new(&mut s)
            .add(
                &f.a,
                AddProject {
                    rule_refs: vec!["foreign.md".into()],
                    ..Default::default()
                },
                &runtime
            )
            .is_err()
    );
    for name in [
        "TOKEN=secret",
        "GIT_DIR",
        "PATH",
        "CODEX_HOME",
        "LD_PRELOAD",
        "RRX_STATE_PATH",
        "1TOKEN",
    ] {
        assert!(
            Registry::new(&mut s)
                .add(
                    &f.b,
                    AddProject {
                        environment_refs: vec![name.into()],
                        ..Default::default()
                    },
                    &runtime
                )
                .is_err(),
            "{name}"
        );
    }
    std::fs::remove_file(f.a.join("project.toml")).unwrap();
    std::os::unix::fs::symlink(f.b.join("rules.md"), f.a.join("project.toml")).unwrap();
    assert!(effective_config(&a, &runtime).is_err());
    Registry::new(&mut s).reconcile().unwrap();
    let blocked = s.project(a.id).unwrap().unwrap();
    assert_eq!(blocked.state, ProjectState::Blocked);
    assert!(effective_config(&blocked, &runtime).is_err());
    let out = f.cli(
        &f.root,
        &[
            "--project-config",
            f.b.join("rules.md").to_str().unwrap(),
            "project",
            "status",
            &a.id.to_string(),
        ],
    );
    assert!(!out.status.success());
}
#[test]
fn removal_checks_live_goals_tasks_sessions_and_locks_under_store_transaction() {
    let f = Fixture::new();
    let mut s = Store::open(&f.db).unwrap();
    let mut p = f.add(&mut s, &f.a);
    let mut g = goal(&mut s, &p);
    let mut t = task(&mut s, &p, &g);
    assert!(
        Registry::new(&mut s)
            .remove(&p.id.to_string(), &f.root)
            .is_err()
    );
    g.state = GoalState::Completed;
    s.put_goal(&mut g).unwrap();
    assert!(
        Registry::new(&mut s)
            .remove(&p.id.to_string(), &f.root)
            .is_err()
    );
    let wt = WorktreeManager::create(&mut s, t.id).unwrap();
    t = s.task(t.id).unwrap().unwrap();
    t.state = TaskState::Completed;
    s.put_task(&mut t).unwrap();
    let mut session = Session {
        id: SessionId::new(),
        scope: t.scope(),
        agent: "fake".into(),
        provider: "fixture".into(),
        role: SessionRole::Reviewer,
        native_ref: None,
        pid: None,
        worktree: wt.worktree.clone(),
        state: SessionState::Lost,
        model: None,
        effort: None,
        recovery: json!({}),
        started_at: now_ms(),
    };
    let version = s.put_session(&session, 0).unwrap();
    assert!(
        Registry::new(&mut s)
            .remove(&p.id.to_string(), &f.root)
            .is_err()
    );
    session.state = SessionState::Stopped;
    s.put_session(&session, version).unwrap();
    let lock = WorktreeManager::lock_review(&mut s, t.id, &wt.revision, "fixture").unwrap();
    assert!(
        Registry::new(&mut s)
            .remove(&p.id.to_string(), &f.root)
            .is_err()
    );
    WorktreeManager::unlock_review(&mut s, lock).unwrap();
    Registry::new(&mut s)
        .remove(&p.id.to_string(), &f.root)
        .unwrap();
    assert!(wt.worktree.exists());
    assert!(f.a.exists());
    // Independent stale Store cannot create active work after removal commits.
    let mut reopened = Store::open(&f.db).unwrap();
    let mut next = Goal::new(p.id, "late".into(), g.completion_criteria.clone());
    assert!(reopened.put_goal(&mut next).is_err());
    let mut next = Task::new(p.id, g.id, "late".into(), "fake".into());
    assert!(reopened.put_task(&mut next).is_err());
    session.state = SessionState::Starting;
    assert!(reopened.put_session(&session, 2).is_err());
    p = s.project(p.id).unwrap().unwrap();
    p.state = ProjectState::Registered;
    s.put_project(&mut p).unwrap();
}
#[test]
fn git_routing_env_does_not_redirect_registry_to_other_project() {
    let f = Fixture::new();
    let out = Command::new(env!("CARGO_BIN_EXE_rrx"))
        .arg("--state")
        .arg(&f.db)
        .args(["project", "add", f.a.to_str().unwrap()])
        .env("GIT_DIR", f.b.join(".git"))
        .env("GIT_WORK_TREE", &f.b)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let store = Store::open(&f.db).unwrap();
    let p = store.projects().unwrap().pop().unwrap();
    assert_eq!(p.root, f.a);
    assert_eq!(
        p.repository_identity,
        rrx::git::repository_identity(&f.a, "main").unwrap()
    );
}
#[test]
fn global_default_state_and_help_config_check_are_independent_of_cwd() {
    let f = Fixture::new();
    let home = f.root.join("home");
    std::fs::create_dir(&home).unwrap();
    for cwd in [&f.a, &f.b] {
        let out = Command::new(env!("CARGO_BIN_EXE_rrx"))
            .current_dir(cwd)
            .env_remove("RRX_STATE_PATH")
            .env("XDG_STATE_HOME", &home)
            .args(["project", "add", cwd.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(out.status.success());
    }
    let store = Store::open(&home.join("rururunx/state.sqlite3")).unwrap();
    assert_eq!(store.projects().unwrap().len(), 2);
    let untouched = f.root.join("must-not-exist/state.db");
    for command in ["--help", "--version", "config-check"] {
        let out = Command::new(env!("CARGO_BIN_EXE_rrx"))
            .arg("--state")
            .arg(&untouched)
            .arg(command)
            .output()
            .unwrap();
        assert!(out.status.success());
        assert!(!untouched.parent().unwrap().exists());
    }
}
