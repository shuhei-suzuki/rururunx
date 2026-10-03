use rrx::{
    adapter::SharedStore,
    context::{Budget, SelectionRequest},
    context_pack::*,
    domain::*,
    git::WorktreeManager,
    state::Store,
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};
use tempfile::TempDir;

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
struct Fixture {
    _temp: TempDir,
    root: PathBuf,
    worktree: PathBuf,
    store: SharedStore,
    project: Project,
    task: Task,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap().join("repo");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "-b", "main"]);
        git(&root, &["config", "user.name", "Fixture"]);
        git(&root, &["config", "user.email", "fixture@example.invalid"]);
        git(&root, &["config", "commit.gpgsign", "false"]);
        git(&root, &["config", "core.hooksPath", ".git/hooks"]);
        std::fs::write(root.join(".gitignore"), "worktree/\nignored/\n").unwrap();
        std::fs::write(
            root.join("RULES.md"),
            "MANDATORY: preserve Project boundaries.\n",
        )
        .unwrap();
        std::fs::create_dir(root.join("src")).unwrap();
        std::fs::write(
            root.join("src/lib.rs"),
            "mod codec;\npub fn run() { codec::encode(); }\n",
        )
        .unwrap();
        std::fs::write(
            root.join("src/codec.rs"),
            "pub fn encode() -> String { String::new() }\n",
        )
        .unwrap();
        std::fs::write(root.join("unrelated.txt"), "weather sunshine\n").unwrap();
        git(&root, &["add", "."]);
        git(&root, &["commit", "-m", "fixture"]);
        let mut store = Store::open(&temp.path().join("state.db")).unwrap();
        let mut project = Project::new(
            "fixture".into(),
            root.clone(),
            rrx::git::repository_identity(&root, "main").unwrap(),
            "main".into(),
        );
        project.rule_refs = vec![root.join("RULES.md")];
        store.put_project(&mut project).unwrap();
        let mut goal = Goal::new(
            project.id,
            "Retain exact scope".into(),
            vec![CompletionCriterion {
                id: "done".into(),
                description: "verified independently".into(),
                evidence: None,
                satisfied: false,
            }],
        );
        goal.state = GoalState::Running;
        goal.constraints = vec!["Never remove Project safety constraints".into()];
        store.put_goal(&mut goal).unwrap();
        let mut task = Task::new(project.id, goal.id, "Implement codec".into(), "fake".into());
        task.issue = Some(42);
        task.acceptance_criteria = vec!["never omit required evidence".into()];
        store.put_task(&mut task).unwrap();
        let worktree = WorktreeManager::create(&mut store, task.id)
            .unwrap()
            .worktree;
        task = store.task(task.id).unwrap().unwrap();
        goal.dag.nodes = vec![task.id];
        store.put_goal(&mut goal).unwrap();
        Self {
            _temp: temp,
            root,
            worktree,
            store: Arc::new(Mutex::new(store)),
            project,
            task,
        }
    }
    fn packs(&self) -> ContextPacks {
        ContextPacks::new(self.store.clone())
    }
}
fn budget() -> Budget {
    Budget {
        estimated_tokens: 100_000,
        bytes: 100_000,
    }
}
fn input() -> TaskInputs {
    TaskInputs {
        artifacts: vec![ArtifactRequest {
            kind: ArtifactKind::Source,
            path: "src/lib.rs".into(),
        }],
        decisions: vec!["Use scoped authoritative references".into()],
        ..Default::default()
    }
}
fn event(sequence: u64, kind: EventKind, text: &str) -> HistoryEvent {
    HistoryEvent {
        sequence,
        kind,
        text: text.into(),
    }
}
fn session(f: &Fixture, role: SessionRole, state: SessionState) -> (Session, u64) {
    let s = Session {
        id: SessionId::new(),
        scope: f.task.scope(),
        agent: "fixture".into(),
        provider: "fixture".into(),
        role,
        native_ref: None,
        pid: None,
        worktree: f.worktree.clone(),
        state,
        model: None,
        effort: None,
        recovery: serde_json::json!({}),
        started_at: now_ms(),
    };
    let version = f.store.lock().unwrap().put_session(&s, 0).unwrap();
    (s, version)
}
#[tokio::test]
async fn publication_reopens_is_idempotent_and_native_payload_uses_owned_authority() {
    let f = Fixture::new();
    let packs = f.packs();
    let draft = packs.draft_task(&f.task.scope(), input()).await.unwrap();
    let reference = packs.publish_task(&draft).await.unwrap();
    assert_eq!(reference.version, 1);
    packs.validate_task(&reference).await.unwrap();
    let second = packs.draft_task(&f.task.scope(), input()).await.unwrap();
    assert_eq!(packs.publish_task(&second).await.unwrap(), reference);
    let reopened = ContextPacks::new(Arc::new(Mutex::new(
        Store::open(&f._temp.path().join("state.db")).unwrap(),
    )));
    reopened.validate_task(&reference).await.unwrap();
    let pack = reopened.task_pack(&reference).unwrap();
    assert_eq!(pack.artifacts[0].scope, f.task.scope());
    assert_eq!(pack.repository_identity, f.project.repository_identity);
    assert!(pack.artifacts[0].digest.starts_with("sha256:"));
    assert_eq!(pack.artifacts[0].path, "src/lib.rs");
    let PreparedPack::Ready(prepared) = reopened
        .prepare_task(
            &reference,
            SelectionRequest {
                task_text: "codec".into(),
                ..Default::default()
            },
            budget(),
        )
        .await
        .unwrap()
    else {
        panic!("expected ready")
    };
    assert!(
        prepared
            .payload
            .contains("Never remove Project safety constraints")
    );
    assert!(
        prepared
            .payload
            .contains("MANDATORY: preserve Project boundaries")
    );
    assert!(prepared.payload.contains("task_context_pack"));
    assert_eq!(prepared.scope, f.task.scope());
    assert_eq!(
        f.store
            .lock()
            .unwrap()
            .task(f.task.id)
            .unwrap()
            .unwrap()
            .context_version,
        1
    );
    assert_eq!(
        f.store
            .lock()
            .unwrap()
            .context(&f.task.scope(), None)
            .unwrap()
            .unwrap()
            .version,
        1
    );
}
#[tokio::test]
async fn dirty_rule_and_semantic_changes_stale_packs_and_create_versions() {
    let f = Fixture::new();
    let packs = f.packs();
    let r = packs
        .publish_task(&packs.draft_task(&f.task.scope(), input()).await.unwrap())
        .await
        .unwrap();
    std::fs::write(
        f.worktree.join("src/codec.rs"),
        "pub fn encode() -> String { panic!(\"dirty\") }\n",
    )
    .unwrap();
    assert!(packs.validate_task(&r).await.is_err());
    let r2 = packs
        .publish_task(&packs.draft_task(&f.task.scope(), input()).await.unwrap())
        .await
        .unwrap();
    assert_eq!(r2.version, 2);
    std::fs::write(
        f.root.join("RULES.md"),
        "MANDATORY: stronger safety policy\n",
    )
    .unwrap();
    assert!(packs.validate_task(&r2).await.is_err());
    let r3 = packs
        .publish_task(&packs.draft_task(&f.task.scope(), input()).await.unwrap())
        .await
        .unwrap();
    assert_eq!(r3.version, 3);
    {
        let mut store = f.store.lock().unwrap();
        let mut goal = store.goal(f.task.goal_id).unwrap().unwrap();
        goal.constraints
            .push("New mandatory Goal constraint".into());
        store.put_goal(&mut goal).unwrap();
    }
    assert!(packs.validate_task(&r3).await.is_err());
    let r4 = packs
        .publish_task(&packs.draft_task(&f.task.scope(), input()).await.unwrap())
        .await
        .unwrap();
    assert_eq!(r4.version, 4);
    packs.validate_task(&r4).await.unwrap();
}
#[tokio::test]
async fn phase_bookkeeping_is_not_source_authority_but_goal_instructions_are() {
    let f = Fixture::new();
    let packs = f.packs();
    let before = packs
        .draft_task(&f.task.scope(), input())
        .await
        .unwrap()
        .source_versions();
    {
        let mut store = f.store.lock().unwrap();
        let mut task = store.task(f.task.id).unwrap().unwrap();
        task.state = TaskState::Planning;
        task.phase = Some("Design".into());
        task.workflow = rrx::config::WorkflowClass::Strict;
        task.risk = RiskClass::R3;
        store.put_task(&mut task).unwrap();
        let mut goal = store.goal(task.goal_id).unwrap().unwrap();
        goal.completion_criteria[0].satisfied = true;
        store.put_goal(&mut goal).unwrap();
    }
    let after = packs
        .draft_task(&f.task.scope(), input())
        .await
        .unwrap()
        .source_versions();
    assert_eq!(before, after);
    {
        let mut store = f.store.lock().unwrap();
        let mut task = store.task(f.task.id).unwrap().unwrap();
        task.acceptance_criteria.push("new criterion".into());
        store.put_task(&mut task).unwrap();
    }
    let changed = packs
        .draft_task(&f.task.scope(), input())
        .await
        .unwrap()
        .source_versions();
    assert_ne!(after["instruction:task"], changed["instruction:task"]);
    {
        let mut store = f.store.lock().unwrap();
        let mut goal = store.goal(f.task.goal_id).unwrap().unwrap();
        goal.constraints.push("new safety constraint".into());
        store.put_goal(&mut goal).unwrap();
    }
    let changed_goal = packs
        .draft_task(&f.task.scope(), input())
        .await
        .unwrap()
        .source_versions();
    assert_ne!(
        changed["instruction:goal"],
        changed_goal["instruction:goal"]
    );
}
#[tokio::test]
async fn checkpoint_preserves_semantics_incrementally_and_never_rewrites_live_launch_pack() {
    let f = Fixture::new();
    let packs = f.packs();
    let reference = packs
        .publish_task(&packs.draft_task(&f.task.scope(), input()).await.unwrap())
        .await
        .unwrap();
    let (mut native, version) = session(&f, SessionRole::Executor, SessionState::Running);
    let events = vec![
        event(1, EventKind::Goal, "durable objective"),
        event(2, EventKind::Decision, "confirmed decision"),
        event(3, EventKind::Completed, "completed work"),
        event(4, EventKind::Failure, "failure is unresolved"),
        event(5, EventKind::Finding, "independent finding"),
        event(6, EventKind::NextAction, "next safe action"),
        event(7, EventKind::Constraint, "never remove safety constraints"),
        event(8, EventKind::CriticalReference, "src/codec.rs::encode"),
        event(9, EventKind::Verification, "cargo test passed"),
        event(10, EventKind::Transient, "old transient line"),
        event(11, EventKind::Transient, "newest transient line"),
    ];
    let cp = packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            None,
            events,
            HistoryPolicy {
                recent_history_bytes: 170,
            },
        )
        .await
        .unwrap();
    let data = packs.load_checkpoint(&cp).unwrap();
    assert_eq!(data.retained.len(), 9);
    assert!(data.recent_bytes <= 170);
    assert!(data.omitted_transient > 0);
    assert_eq!(
        f.store
            .lock()
            .unwrap()
            .task(f.task.id)
            .unwrap()
            .unwrap()
            .context_version,
        reference.version
    );
    assert!(
        packs
            .publish_task(
                &packs
                    .draft_task(
                        &f.task.scope(),
                        TaskInputs {
                            checkpoint: Some(cp.clone()),
                            ..input()
                        }
                    )
                    .await
                    .unwrap()
            )
            .await
            .is_err()
    );
    let cp2 = packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            Some(cp.clone()),
            vec![event(12, EventKind::Transient, "future transient line")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    let second = packs.load_checkpoint(&cp2).unwrap();
    assert_eq!(second.retained, data.retained);
    assert!(second.recent.is_empty());
    assert_eq!(second.omitted_transient, 3);
    assert!(
        packs
            .checkpoint(
                &f.task.scope(),
                native.id,
                Some(cp),
                vec![event(12, EventKind::Decision, "forked chain")],
                HistoryPolicy {
                    recent_history_bytes: 0
                }
            )
            .await
            .is_err()
    );
    native.state = SessionState::Exited;
    f.store
        .lock()
        .unwrap()
        .put_session(&native, version)
        .unwrap();
    let promoted = packs
        .publish_task(
            &packs
                .draft_task(
                    &f.task.scope(),
                    TaskInputs {
                        checkpoint: Some(cp2),
                        ..input()
                    },
                )
                .await
                .unwrap(),
        )
        .await
        .unwrap();
    let data = packs.task_pack(&promoted).unwrap();
    let historical = data.historical_checkpoint.unwrap().to_string();
    assert!(historical.contains("confirmed decision"));
    assert!(historical.contains("never remove safety constraints"));
    assert!(!historical.contains("future transient line"));
    let audit = f
        .store
        .lock()
        .unwrap()
        .events(&f.task.scope(), 0, 1000)
        .unwrap();
    let audit = serde_json::to_string(&audit).unwrap();
    assert!(!audit.contains("confirmed decision"));
    assert!(audit.contains("input_digest"));
}
#[tokio::test]
async fn consultation_promotes_historical_facts_without_transcript_and_rejects_foreign_refs() {
    let f = Fixture::new();
    let packs = f.packs();
    let (native, _) = session(&f, SessionRole::Consultant, SessionState::Exited);
    let cp = packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            None,
            vec![
                event(1, EventKind::Decision, "consultation conclusion"),
                event(2, EventKind::Transient, "FULL TRANSCRIPT SENTINEL"),
            ],
            HistoryPolicy {
                recent_history_bytes: 1000,
            },
        )
        .await
        .unwrap();
    let target = {
        let mut store = f.store.lock().unwrap();
        let mut task = Task::new(
            f.project.id,
            f.task.goal_id,
            "New target".into(),
            "fake".into(),
        );
        task.issue = Some(43);
        store.put_task(&mut task).unwrap();
        WorktreeManager::create(&mut store, task.id).unwrap();
        store.task(task.id).unwrap().unwrap()
    };
    let draft = packs
        .draft_task(
            &target.scope(),
            TaskInputs {
                checkpoint: Some(cp.clone()),
                ..input()
            },
        )
        .await
        .unwrap();
    let value = serde_json::to_string(draft.pack()).unwrap();
    assert!(value.contains("consultation conclusion"));
    assert!(!value.contains("FULL TRANSCRIPT SENTINEL"));
    assert!(value.contains(&f.task.id.to_string()));
    assert_eq!(draft.pack().artifacts[0].scope, target.scope());
    assert_ne!(draft.pack().repository.worktree, f.worktree);
    let r = packs.publish_task(&draft).await.unwrap();
    packs.validate_task(&r).await.unwrap();
    let foreign = Fixture::new();
    assert!(
        foreign
            .packs()
            .draft_task(
                &foreign.task.scope(),
                TaskInputs {
                    checkpoint: Some(cp.clone()),
                    ..input()
                }
            )
            .await
            .is_err()
    );
    let mut forged = cp;
    forged.scope = target.scope();
    assert!(packs.load_checkpoint(&forged).is_err());
    let mut bad = r;
    bad.digest = "sha256:forged".into();
    assert!(packs.task_pack(&bad).is_err());
}
#[tokio::test]
async fn complete_rendered_budget_and_authoritative_path_guards_fail_closed() {
    let f = Fixture::new();
    let packs = f.packs();
    let reference = packs
        .publish_task(&packs.draft_task(&f.task.scope(), input()).await.unwrap())
        .await
        .unwrap();
    let PreparedPack::NeedsBudget { required_bytes, .. } = packs
        .prepare_task(
            &reference,
            SelectionRequest::default(),
            Budget {
                bytes: 1,
                estimated_tokens: 1,
            },
        )
        .await
        .unwrap()
    else {
        panic!("budget bypass")
    };
    assert!(required_bytes > 1000);
    let PreparedPack::Ready(prepared) = packs
        .prepare_task(&reference, SelectionRequest::default(), budget())
        .await
        .unwrap()
    else {
        panic!("budget error")
    };
    let cap = prepared.payload.len() - 1;
    let small = packs
        .prepare_task(
            &reference,
            SelectionRequest::default(),
            Budget {
                bytes: cap,
                estimated_tokens: cap,
            },
        )
        .await
        .unwrap();
    if let PreparedPack::Ready(input) = small {
        assert!(input.payload.len() <= cap);
        assert!(
            input
                .payload
                .contains("MANDATORY: preserve Project boundaries")
        );
    }
    for path in ["../RULES.md", "/etc/passwd", ".git/HEAD", "missing.txt"] {
        assert!(
            packs
                .draft_task(
                    &f.task.scope(),
                    TaskInputs {
                        artifacts: vec![ArtifactRequest {
                            kind: ArtifactKind::Design,
                            path: path.into()
                        }],
                        ..Default::default()
                    }
                )
                .await
                .is_err(),
            "{path}"
        );
    }
    let link = f.worktree.join("foreign-link");
    std::os::unix::fs::symlink(f.root.join("RULES.md"), &link).unwrap();
    assert!(
        packs
            .draft_task(
                &f.task.scope(),
                TaskInputs {
                    artifacts: vec![ArtifactRequest {
                        kind: ArtifactKind::Source,
                        path: "foreign-link".into()
                    }],
                    ..Default::default()
                }
            )
            .await
            .is_err()
    );
}
#[tokio::test]
async fn goal_pack_references_tasks_without_history_and_detects_changed_goal_authority() {
    let f = Fixture::new();
    let packs = f.packs();
    let task_ref = packs
        .publish_task(&packs.draft_task(&f.task.scope(), input()).await.unwrap())
        .await
        .unwrap();
    let goal = packs
        .publish_goal(
            &f.task.scope(),
            vec!["cross-task decision".into()],
            std::collections::BTreeMap::from([("provider_input_tokens".into(), None)]),
        )
        .await
        .unwrap();
    packs.validate_goal(&goal).await.unwrap();
    packs.validate_task(&task_ref).await.unwrap();
    assert_eq!(
        packs
            .publish_goal(
                &f.task.scope(),
                vec!["cross-task decision".into()],
                std::collections::BTreeMap::from([("provider_input_tokens".into(), None)])
            )
            .await
            .unwrap(),
        goal
    );
    let stored = f
        .store
        .lock()
        .unwrap()
        .context(&goal.scope, None)
        .unwrap()
        .unwrap();
    let data: GoalPack = serde_json::from_value(stored.data).unwrap();
    assert_eq!(data.tasks[0].context.as_ref().unwrap(), &task_ref);
    let text = serde_json::to_string(&data).unwrap();
    assert!(!text.contains("Use scoped authoritative references"));
    assert!(!text.contains("historical_checkpoint"));
    assert_eq!(data.next_work_candidates, vec![f.task.id]);
    {
        let mut store = f.store.lock().unwrap();
        let mut p = store.project(f.project.id).unwrap().unwrap();
        p.name = "new display name".into();
        store.put_project(&mut p).unwrap();
    }
    assert!(packs.validate_goal(&goal).await.is_err());
}
#[tokio::test]
async fn stale_drafts_lost_sessions_and_immutable_locks_cannot_publish_context() {
    let f = Fixture::new();
    let packs = f.packs();
    let stale = packs.draft_task(&f.task.scope(), input()).await.unwrap();
    {
        let mut independent = Store::open(&f._temp.path().join("state.db")).unwrap();
        let mut task = independent.task(f.task.id).unwrap().unwrap();
        task.next_action = Some("changed externally".into());
        independent.put_task(&mut task).unwrap();
    }
    assert!(packs.publish_task(&stale).await.is_err());
    let (native, _) = session(&f, SessionRole::Consultant, SessionState::Lost);
    let draft = packs.draft_task(&f.task.scope(), input()).await.unwrap();
    assert!(packs.publish_task(&draft).await.is_err());
    assert_eq!(
        f.store
            .lock()
            .unwrap()
            .task(f.task.id)
            .unwrap()
            .unwrap()
            .context_version,
        0
    );
    let (mut native, version) = f.store.lock().unwrap().session(native.id).unwrap().unwrap();
    native.state = SessionState::Stopped;
    f.store
        .lock()
        .unwrap()
        .put_session(&native, version)
        .unwrap();
    let revision = git(&f.worktree, &["rev-parse", "HEAD"]);
    let lock = WorktreeManager::lock_review(
        &mut f.store.lock().unwrap(),
        f.task.id,
        &revision,
        "fixture",
    )
    .unwrap();
    assert!(
        packs
            .publish_task(&packs.draft_task(&f.task.scope(), input()).await.unwrap())
            .await
            .is_err()
    );
    WorktreeManager::unlock_review(&mut f.store.lock().unwrap(), lock).unwrap();
    packs
        .publish_task(&packs.draft_task(&f.task.scope(), input()).await.unwrap())
        .await
        .unwrap();
}
