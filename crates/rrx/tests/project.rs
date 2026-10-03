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
    let ac = effective_config(&s, a.id, &runtime).unwrap();
    let bc = effective_config(&s, b.id, &runtime).unwrap();
    assert_eq!(ac.scheduler.max_tasks_per_project, 2);
    assert_eq!(ac.scheduler.global_max_sessions, 12);
    assert!(!ac.context.enabled);
    assert!(bc.context.enabled);
    assert_eq!(environment_names(&s, a.id).unwrap(), ["PROJECT_A_TOKEN"]);
    assert!(environment_names(&s, b.id).unwrap().is_empty());
    assert_eq!(a.rule_refs, [f.a.join("rules.md")]);
    assert!(b.rule_refs.is_empty());
    assert!(scoped_file(&s, a.id, &f.b.join("rules.md")).is_err());
    assert!(scoped_file(&s, a.id, Path::new("../b/rules.md")).is_err());
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
        "ANTHROPIC_BASE_URL",
        "OPENAI_BASE_URL",
        "HTTPS_PROXY",
        "http_proxy",
        "SSL_CERT_FILE",
        "NODE_EXTRA_CA_CERTS",
        "CLAUDE_CODE_DEBUG",
        "CODEX_CONFIG",
        "PYTHONHOME",
        "EDITOR",
        "RUBYOPT",
        "NODE_PATH",
        "TMPDIR",
        "XDG_CACHE_HOME",
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
    assert!(effective_config(&s, a.id, &runtime).is_err());
    Registry::new(&mut s).reconcile().unwrap();
    let blocked = s.project(a.id).unwrap().unwrap();
    assert_eq!(blocked.state, ProjectState::Blocked);
    assert!(effective_config(&s, blocked.id, &runtime).is_err());
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

#[test]
fn separate_git_directory_and_symlink_namespace_are_not_source_references() {
    let f = Fixture::new();
    let mut store = Store::open(&f.db).unwrap();
    // Move primary Git metadata into a nonstandard source-local directory.
    git(
        &f.a,
        &[
            "init",
            "--separate-git-dir",
            f.a.join("metadata").to_str().unwrap(),
        ],
    );
    let p = f.add(&mut store, &f.a);
    assert!(scoped_file(&store, p.id, Path::new("metadata/config")).is_err());
    assert!(
        Registry::new(&mut store)
            .add(
                &f.a,
                AddProject {
                    rule_refs: vec!["metadata/config".into()],
                    ..Default::default()
                },
                &Config::default()
            )
            .is_err()
    );
    assert!(
        Registry::new(&mut store)
            .add(
                &f.a,
                AddProject {
                    worktree_root: Some("metadata/worktrees".into()),
                    ..Default::default()
                },
                &Config::default()
            )
            .is_err()
    );
    std::os::unix::fs::symlink(&f.b, f.a.join("escaped")).unwrap();
    assert!(
        Registry::new(&mut store)
            .add(
                &f.a,
                AddProject {
                    worktree_root: Some("escaped/tasks".into()),
                    ..Default::default()
                },
                &Config::default()
            )
            .is_err()
    );
    let nested = f.a.join("nested");
    std::fs::create_dir(&nested).unwrap();
    git(&nested, &["init", "-b", "main"]);
    std::fs::write(nested.join("rules.md"), "foreign").unwrap();
    assert!(scoped_file(&store, p.id, &nested.join("rules.md")).is_err());
    assert!(
        Registry::new(&mut store)
            .add(
                &f.a,
                AddProject {
                    worktree_root: Some("nested/tasks".into()),
                    ..Default::default()
                },
                &Config::default()
            )
            .is_err()
    );
    assert_eq!(
        store.project(p.id).unwrap().unwrap().worktree_root,
        p.worktree_root
    );
}

#[test]
fn cleared_refs_recover_and_stale_input_snapshots_are_rejected() {
    let f = Fixture::new();
    std::fs::write(f.a.join("project.toml"), "[context]\nenabled = false").unwrap();
    let text = f.good(&[
        "--project-config",
        "project.toml",
        "project",
        "add",
        f.a.to_str().unwrap(),
        "--rule",
        "rules.md",
        "--env-ref",
        "PROJECT_A_TOKEN",
    ]);
    let id: ProjectId = text.split('\t').next().unwrap().parse().unwrap();
    std::fs::remove_file(f.a.join("project.toml")).unwrap();
    std::fs::remove_file(f.a.join("rules.md")).unwrap();
    f.good(&["project", "list"]);
    let text = f.good(&[
        "project",
        "add",
        f.a.to_str().unwrap(),
        "--clear-project-config",
        "--clear-rules",
        "--clear-env-refs",
    ]);
    assert!(text.starts_with(&id.to_string()));
    let store = Store::open(&f.db).unwrap();
    let p = store.project(id).unwrap().unwrap();
    assert_eq!(p.state, ProjectState::Registered);
    assert!(p.config_ref.is_none() && p.rule_refs.is_empty() && p.environment_refs.is_empty());
    f.good(&["project", "remove", &id.to_string()]);
    // Existing connection and old snapshot both predate removal in another process.
    assert_eq!(p.state, ProjectState::Registered);
    assert!(effective_config(&store, id, &Config::default()).is_err());
    assert!(environment_names(&store, id).is_err());
    assert!(scoped_file(&store, id, Path::new("rules.md")).is_err());
}
#[test]
fn blocked_projects_accept_lost_and_blocker_updates_without_starting_new_work() {
    let f = Fixture::new();
    let mut s = Store::open(&f.db).unwrap();
    let p = f.add(&mut s, &f.a);
    let mut g = goal(&mut s, &p);
    let mut t = task(&mut s, &p, &g);
    let wt = WorktreeManager::create(&mut s, t.id).unwrap();
    t = s.task(t.id).unwrap().unwrap();
    let mut session = Session {
        id: SessionId::new(),
        scope: t.scope(),
        agent: "fake".into(),
        provider: "fixture".into(),
        role: SessionRole::Executor,
        native_ref: None,
        pid: None,
        worktree: wt.worktree,
        state: SessionState::Running,
        model: None,
        effort: None,
        recovery: json!({}),
        started_at: now_ms(),
    };
    let version = s.put_session(&session, 0).unwrap();
    // Reconciliation blocks source that has moved; existing process remains unknown/live.
    std::fs::rename(&f.a, f.root.join("moved")).unwrap();
    Registry::new(&mut s).reconcile().unwrap();
    session.state = SessionState::Lost;
    let version = s.put_session(&session, version).unwrap();
    g.state = GoalState::Blocked;
    g.blockers.push("source moved".into());
    s.put_goal(&mut g).unwrap();
    t.blockers.push("source moved".into());
    s.put_task(&mut t).unwrap();
    assert!(
        Registry::new(&mut s)
            .remove(&p.id.to_string(), &f.root)
            .is_err()
    );
    session.state = SessionState::Running;
    assert!(s.put_session(&session, version).is_err());
    session.state = SessionState::Lost;
    session.id = SessionId::new();
    assert!(s.put_session(&session, 0).is_err());
    g.state = GoalState::Running;
    assert!(s.put_goal(&mut g).is_err());
    let mut new_task = Task::new(p.id, g.id, "new".into(), "fake".into());
    assert!(s.put_task(&mut new_task).is_err());
}
#[test]
fn blocked_cwd_and_missing_git_never_infer_or_rebind_foreign_sources() {
    let f = Fixture::new();
    let mut store = Store::open(&f.db).unwrap();
    let a = f.add(&mut store, &f.a);
    let b = f.add(&mut store, &f.b);
    let out = Command::new(env!("CARGO_BIN_EXE_rrx"))
        .current_dir(&f.root)
        .arg("--state")
        .arg(&f.db)
        .args(["project", "list"])
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(
        store.project(a.id).unwrap().unwrap().state,
        ProjectState::Registered
    );
    assert_eq!(
        store.project(b.id).unwrap().unwrap().state,
        ProjectState::Registered
    );
    let mut p = a.clone();
    p.state = ProjectState::Blocked;
    p.blocked_reason = Some("fixture".into());
    store.put_project(&mut p).unwrap();
    let nested = f.a.join("nested");
    std::fs::create_dir(&nested).unwrap();
    git(&nested, &["init", "-b", "main"]);
    assert!(Registry::new(&mut store).resolve(None, &nested).is_err());
    assert!(
        Registry::new(&mut store)
            .resolve(None, &f.a.join(".git"))
            .is_err()
    );
    assert_eq!(
        Registry::new(&mut store).resolve(None, &f.a).unwrap().id,
        a.id
    );
    std::fs::rename(&f.a, f.root.join("old-a")).unwrap();
    std::fs::rename(&f.b, &f.a).unwrap();
    assert!(Registry::new(&mut store).resolve(None, &f.a).is_err());
}
#[test]
fn active_creation_and_removal_race_has_one_valid_winner() {
    use std::sync::{Arc, Barrier};
    let f = Fixture::new();
    let mut s = Store::open(&f.db).unwrap();
    let p = f.add(&mut s, &f.a);
    let barrier = Arc::new(Barrier::new(2));
    let db = f.db.clone();
    let root = f.root.clone();
    let project = p.clone();
    let gate = barrier.clone();
    let remove = std::thread::spawn(move || {
        let mut s = Store::open(&db).unwrap();
        gate.wait();
        Registry::new(&mut s)
            .remove(&project.id.to_string(), &root)
            .is_ok()
    });
    let db = f.db.clone();
    let project = p.clone();
    let create = std::thread::spawn(move || {
        let mut s = Store::open(&db).unwrap();
        barrier.wait();
        let mut g = Goal::new(
            project.id,
            "race".into(),
            vec![CompletionCriterion {
                id: "done".into(),
                description: "race".into(),
                evidence: None,
                satisfied: false,
            }],
        );
        s.put_goal(&mut g).is_ok()
    });
    assert_ne!(remove.join().unwrap(), create.join().unwrap());
    let stored = s.project(p.id).unwrap().unwrap();
    assert_eq!(
        stored.state == ProjectState::Removed,
        s.goals(p.id).unwrap().is_empty()
    );
}

#[test]
fn schema_one_project_migrates_without_changing_identity_or_history() {
    let f = Fixture::new();
    let mut store = Store::open(&f.db).unwrap();
    let p = f.add(&mut store, &f.a);
    let events = store.events(&Scope::project(p.id), 0, 100).unwrap();
    drop(store);
    let old = rusqlite::Connection::open(&f.db).unwrap();
    // Version 1 has the same SQL tables, and Project JSON without blocked_reason.
    old.execute(
        "UPDATE projects SET body=json_remove(body,'$.blocked_reason')",
        [],
    )
    .unwrap();
    old.pragma_update(None, "user_version", 1).unwrap();
    drop(old);
    let reopened = Store::open(&f.db).unwrap();
    assert_eq!(reopened.schema_version().unwrap(), 2);
    let recovered = reopened.project(p.id).unwrap().unwrap();
    assert_eq!(recovered.root, p.root);
    assert_eq!(recovered.repository_identity, p.repository_identity);
    assert!(recovered.blocked_reason.is_none());
    assert_eq!(
        reopened
            .events(&Scope::project(p.id), 0, 100)
            .unwrap()
            .len(),
        events.len()
    );
}

#[test]
fn invalid_xdg_state_directory_falls_back_to_absolute_home() {
    let f = Fixture::new();
    let home = f.root.join("isolated-home");
    std::fs::create_dir(&home).unwrap();
    for xdg in ["", "relative"] {
        let out = Command::new(env!("CARGO_BIN_EXE_rrx"))
            .current_dir(&f.root)
            .env_remove("RRX_STATE_PATH")
            .env("XDG_STATE_HOME", xdg)
            .env("HOME", &home)
            .args(["project", "list"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(home.join(".local/state/rururunx/state.sqlite3").exists());
        assert!(!f.root.join("relative").exists());
    }
}
