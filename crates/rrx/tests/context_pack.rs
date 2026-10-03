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
// Isolated corruption fixture for reader validation; public production writers
// reject typed context creation and owned pointer movement.
fn corrupt_context(f: &Fixture, context: &ContextVersion) {
    let raw = rusqlite::Connection::open(f._temp.path().join("state.db")).unwrap();
    raw.execute("INSERT INTO context_versions(project_id,goal_id,task_id,owner,version,body) VALUES(?1,?2,?3,?4,?5,?6)",rusqlite::params![context.scope.project_id.to_string(),context.scope.goal_id.unwrap().to_string(),context.scope.task_id.unwrap().to_string(),format!("task:{}",context.scope.task_id.unwrap()),context.version,serde_json::to_string(context).unwrap()]).unwrap();
    let mut task = f.store.lock().unwrap().task(f.task.id).unwrap().unwrap();
    task.context_version = context.version;
    task.version += 1;
    raw.execute(
        "UPDATE tasks SET body=?1,version=?2 WHERE id=?3",
        rusqlite::params![
            serde_json::to_string(&task).unwrap(),
            task.version,
            task.id.to_string()
        ],
    )
    .unwrap();
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
    assert_eq!(pack.referenced_sources[0].path, "src/lib.rs");
    assert_eq!(pack.referenced_sources[0].digest, pack.artifacts[0].digest);
    assert!(
        pack.referenced_sources[0]
            .symbols
            .iter()
            .any(|s| s.name == "run")
    );
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
    let events = f
        .store
        .lock()
        .unwrap()
        .events(&f.task.scope(), 0, 1000)
        .unwrap();
    let estimate = events
        .iter()
        .rev()
        .find(|e| e.kind == "context.pack.prepared")
        .unwrap();
    assert_eq!(
        estimate.data["estimated_bytes"],
        serde_json::json!(prepared.payload.len())
    );
    assert_eq!(
        estimate.data["estimated_tokens"],
        serde_json::json!(prepared.payload.len())
    );
    assert!(estimate.data["measured_tokens"].is_null());
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
#[tokio::test]
async fn typed_reader_does_not_accept_metadata_that_disagrees_with_actual_task() {
    use sha2::{Digest, Sha256};
    let f = Fixture::new();
    let packs = f.packs();
    let original = packs
        .publish_task(&packs.draft_task(&f.task.scope(), input()).await.unwrap())
        .await
        .unwrap();
    let mut context = f
        .store
        .lock()
        .unwrap()
        .context(&f.task.scope(), Some(original.version))
        .unwrap()
        .unwrap();
    context.version += 1;
    context.data["task"]["title"] = serde_json::json!("FORGED TASK PURPOSE");
    {
        let mut store = f.store.lock().unwrap();
        assert!(store.put_context(&context).is_err());
        let mut task = store.task(f.task.id).unwrap().unwrap();
        task.context_version = context.version;
        assert!(store.put_task(&mut task).is_err());
    }
    corrupt_context(&f, &context);
    let forged = PackRef {
        scope: f.task.scope(),
        version: context.version,
        digest: format!(
            "sha256:{:x}",
            Sha256::digest(serde_json::to_vec(&context).unwrap())
        ),
    };
    assert!(
        packs
            .prepare_task(&forged, SelectionRequest::default(), budget())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn goal_before_tasks_uses_primary_sources_and_never_adopts_a_worktree() {
    let f = Fixture::new();
    let packs = f.packs();
    let mut goal = Goal::new(
        f.project.id,
        "Plan before Tasks".into(),
        vec![CompletionCriterion {
            id: "planned".into(),
            description: "Plan persisted".into(),
            satisfied: false,
            evidence: None,
        }],
    );
    f.store.lock().unwrap().put_goal(&mut goal).unwrap();
    let inputs = || GoalInputs {
        artifacts: vec![ArtifactRequest {
            kind: ArtifactKind::Requirements,
            path: "src/lib.rs".into(),
        }],
        decisions: vec!["Use authoritative source refs".into()],
        ..Default::default()
    };
    let first = packs
        .publish_goal_with_inputs(&goal.scope(), inputs())
        .await
        .unwrap();
    packs.validate_goal(&first).await.unwrap();
    let data = packs.goal_pack(&first).unwrap();
    assert!(data.tasks.is_empty());
    assert_eq!(data.repository.root, f.root);
    assert_eq!(data.artifacts[0].scope, goal.scope());
    assert_eq!(
        packs
            .publish_goal_with_inputs(&goal.scope(), inputs())
            .await
            .unwrap(),
        first
    );
    let independent = Store::open(&f._temp.path().join("state.db")).unwrap();
    let reopened = ContextPacks::new(Arc::new(Mutex::new(independent)));
    reopened.validate_goal(&first).await.unwrap();
    let head = git(&f.root, &["rev-parse", "HEAD"]);
    std::fs::write(
        f.root.join("src/lib.rs"),
        "mod codec;\npub fn run() { codec::encode2(); }\n",
    )
    .unwrap();
    assert_eq!(git(&f.root, &["rev-parse", "HEAD"]), head);
    assert!(packs.validate_goal(&first).await.is_err());
    let second = packs
        .publish_goal_with_inputs(&goal.scope(), inputs())
        .await
        .unwrap();
    assert_eq!(second.version, 2);
    assert!(data.artifacts[0].digest != packs.goal_pack(&second).unwrap().artifacts[0].digest);
    std::fs::write(
        f.root.join("RULES.md"),
        "MANDATORY: never cross Project boundaries.\n",
    )
    .unwrap();
    assert!(packs.validate_goal(&second).await.is_err());
    let third = packs
        .publish_goal_with_inputs(&goal.scope(), inputs())
        .await
        .unwrap();
    assert_eq!(third.version, 3);
    packs.validate_goal(&third).await.unwrap();
    assert!(
        packs
            .publish_goal_with_inputs(&f.task.scope(), inputs())
            .await
            .is_err()
    );
    let foreign = Scope::goal(ProjectId::new(), goal.id);
    assert!(
        packs
            .publish_goal_with_inputs(&foreign, inputs())
            .await
            .is_err()
    );
    let tasks = f.store.lock().unwrap().tasks(f.project.id, None).unwrap();
    assert_eq!(tasks.len(), 1); // only the fixture's unrelated Task exists
    assert!(!f.root.join("worktree").join(goal.id.to_string()).exists());
}

#[tokio::test]
async fn goal_retains_nonlaunchable_finalized_task_pack_after_native_cleanup() {
    let f = Fixture::new();
    let packs = f.packs();
    let task_ref = packs
        .publish_task(&packs.draft_task(&f.task.scope(), input()).await.unwrap())
        .await
        .unwrap();
    {
        let mut store = f.store.lock().unwrap();
        let mut task = store.task(f.task.id).unwrap().unwrap();
        task.state = TaskState::Merged;
        store.put_task(&mut task).unwrap();
        WorktreeManager::cleanup(&mut store, f.task.id).unwrap();
    }
    assert!(!f.worktree.exists());
    let goal = packs
        .publish_goal(
            &Scope::goal(f.project.id, f.task.goal_id),
            vec![],
            Default::default(),
        )
        .await
        .unwrap();
    packs.validate_goal(&goal).await.unwrap();
    let data = packs.goal_pack(&goal).unwrap();
    assert!(data.tasks[0].historical);
    assert_eq!(data.tasks[0].context, Some(task_ref.clone()));
    assert!(
        packs
            .prepare_task(&task_ref, SelectionRequest::default(), budget())
            .await
            .is_err()
    );
    assert!(!f.worktree.exists());
    assert_eq!(
        packs
            .publish_goal(&goal.scope, vec![], Default::default())
            .await
            .unwrap(),
        goal
    );
}

#[tokio::test]
async fn goal_primary_artifacts_reject_escaping_sources_and_replaced_repository() {
    let f = Fixture::new();
    let packs = f.packs();
    let goal = Scope::goal(f.project.id, f.task.goal_id);
    let input = |path: &str| GoalInputs {
        artifacts: vec![ArtifactRequest {
            kind: ArtifactKind::Design,
            path: path.into(),
        }],
        ..Default::default()
    };
    std::os::unix::fs::symlink(f.worktree.join("src/lib.rs"), f.root.join("foreign-link")).unwrap();
    std::fs::hard_link(f.root.join("src/lib.rs"), f.root.join("hard-link")).unwrap();
    for path in [
        "../RULES.md",
        "/etc/passwd",
        ".git/HEAD",
        "missing.txt",
        "foreign-link",
        "hard-link",
        "worktree/rrx-task/x",
    ] {
        assert!(
            packs
                .publish_goal_with_inputs(&goal, input(path))
                .await
                .is_err(),
            "{path}"
        );
    }
    // Hard-linked sources are rejected even when their apparent path is scoped.
    std::fs::remove_file(f.root.join("hard-link")).unwrap();
    let valid = packs
        .publish_goal_with_inputs(&goal, input("src/lib.rs"))
        .await
        .unwrap();
    let moved = f.root.with_file_name("moved-repo");
    std::fs::rename(&f.root, &moved).unwrap();
    assert!(packs.validate_goal(&valid).await.is_err());
    std::fs::create_dir(&f.root).unwrap();
    git(&f.root, &["init", "-b", "main"]);
    git(&f.root, &["config", "user.name", "Fixture"]);
    git(
        &f.root,
        &["config", "user.email", "fixture@example.invalid"],
    );
    git(&f.root, &["config", "commit.gpgsign", "false"]);
    git(&f.root, &["config", "core.hooksPath", ".git/hooks"]);
    std::fs::write(
        f.root.join("RULES.md"),
        "MANDATORY: preserve Project boundaries.\n",
    )
    .unwrap();
    git(&f.root, &["add", "."]);
    git(&f.root, &["commit", "-m", "replacement"]);
    assert!(
        packs
            .publish_goal_with_inputs(&goal, input("RULES.md"))
            .await
            .is_err()
    );
    std::fs::remove_dir_all(&f.root).unwrap();
    std::fs::rename(moved, &f.root).unwrap();
    packs.validate_goal(&valid).await.unwrap();
}

#[tokio::test]
async fn checkpoint_transient_count_bound_and_retained_session_provenance_survive_reload() {
    let f = Fixture::new();
    let packs = f.packs();
    let (native, _) = session(&f, SessionRole::Consultant, SessionState::Running);
    let events = (1..=4096)
        .map(|i| event(i, EventKind::Transient, "short"))
        .collect();
    let first = packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            None,
            events,
            HistoryPolicy {
                recent_history_bytes: 1024 * 1024,
            },
        )
        .await
        .unwrap();
    assert_eq!(packs.load_checkpoint(&first).unwrap().recent.len(), 4096);
    let second = packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            Some(first),
            vec![event(4097, EventKind::Transient, "latest")],
            HistoryPolicy {
                recent_history_bytes: 1024 * 1024,
            },
        )
        .await
        .unwrap();
    let cp = packs.load_checkpoint(&second).unwrap();
    assert_eq!(cp.recent.len(), 4096);
    assert_eq!(cp.omitted_transient, 1);
    assert_eq!(cp.recent[0].event.sequence, 2);
    assert_eq!(cp.recent.last().unwrap().event.sequence, 4097);
    // Generic Record persistence cannot turn a foreign/unknown actor into trusted
    // historical context merely by supplying a fresh envelope digest.
    let mut record = f.store.lock().unwrap().record(second.id).unwrap().unwrap();
    record.data["recent"][0]["session"] = serde_json::to_value(SessionId::new()).unwrap();
    assert!(f.store.lock().unwrap().put_record(&mut record).is_err());
    // Corrupt only the isolated test database to exercise reader provenance
    // independently of the production generic Record write fence.
    rusqlite::Connection::open(f._temp.path().join("state.db"))
        .unwrap()
        .execute(
            "UPDATE records SET body=?1 WHERE id=?2",
            rusqlite::params![
                serde_json::to_string(&record).unwrap(),
                record.id.to_string()
            ],
        )
        .unwrap();
    use sha2::{Digest, Sha256};
    let forged = CheckpointRef {
        scope: record.scope,
        id: record.id,
        version: record.version,
        digest: format!(
            "sha256:{:x}",
            Sha256::digest(serde_json::to_vec(&record.data).unwrap())
        ),
    };
    assert!(packs.load_checkpoint(&forged).is_err());
}

#[tokio::test]
async fn typed_checkpoints_are_immutable_and_promotion_matches_exact_current_head() {
    let f = Fixture::new();
    let packs = f.packs();
    let (native, _) = session(&f, SessionRole::Consultant, SessionState::Exited);
    let cp = packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            None,
            vec![event(
                1,
                EventKind::Constraint,
                "Never discard this constraint",
            )],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    let draft = packs
        .draft_task(
            &f.task.scope(),
            TaskInputs {
                checkpoint: Some(cp.clone()),
                ..input()
            },
        )
        .await
        .unwrap();
    let valid = packs.publish_task(&draft).await.unwrap();
    packs.validate_task(&valid).await.unwrap();
    let mut original = f.store.lock().unwrap().record(cp.id).unwrap().unwrap();
    original.data["retained"] = serde_json::json!([]);
    assert!(f.store.lock().unwrap().put_record(&mut original).is_err());
    let mut injection = Record::new(
        f.task.scope(),
        RecordKind::Checkpoint,
        original.data.clone(),
    );
    injection.data["chain_version"] = serde_json::json!(u64::MAX);
    assert!(f.store.lock().unwrap().put_record(&mut injection).is_err());
    assert_eq!(packs.load_checkpoint(&cp).unwrap().retained.len(), 1);
    // Rehashed context envelopes cannot forge a checkpoint promotion body.
    let frozen = f
        .store
        .lock()
        .unwrap()
        .context(&f.task.scope(), Some(valid.version))
        .unwrap()
        .unwrap();
    for variant in 0..2 {
        let mut forged = frozen.clone();
        forged.version = 2 + variant;
        if variant == 0 {
            forged.data["historical_checkpoint"]["retained"] = serde_json::json!([]);
        } else {
            forged.data["checkpoint"] = serde_json::Value::Null;
        }
        {
            let mut store = f.store.lock().unwrap();
            assert!(store.put_context(&forged).is_err());
            let mut t = store.task(f.task.id).unwrap().unwrap();
            t.context_version = forged.version;
            assert!(store.put_task(&mut t).is_err());
        }
        corrupt_context(&f, &forged);
        use sha2::{Digest, Sha256};
        let reference = PackRef {
            scope: forged.scope.clone(),
            version: forged.version,
            digest: format!(
                "sha256:{:x}",
                Sha256::digest(serde_json::to_vec(&forged).unwrap())
            ),
        };
        assert!(packs.validate_task(&reference).await.is_err());
    }
    let stale = packs
        .draft_task(
            &f.task.scope(),
            TaskInputs {
                checkpoint: Some(cp.clone()),
                ..input()
            },
        )
        .await
        .unwrap();
    let next = packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            Some(cp.clone()),
            vec![event(2, EventKind::Failure, "new unresolved failure")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    assert!(
        packs
            .prepare_draft(&stale, SelectionRequest::default(), budget())
            .await
            .is_err()
    );
    let error = packs.publish_task(&stale).await.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("checkpoint reference no longer current"),
        "{error:#}"
    );
    assert!(
        packs
            .draft_task(
                &f.task.scope(),
                TaskInputs {
                    checkpoint: Some(cp),
                    ..input()
                }
            )
            .await
            .is_err()
    );
    let current = packs
        .publish_task(
            &packs
                .draft_task(
                    &f.task.scope(),
                    TaskInputs {
                        checkpoint: Some(next.clone()),
                        ..input()
                    },
                )
                .await
                .unwrap(),
        )
        .await
        .unwrap();
    packs.validate_task(&current).await.unwrap();
    packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            Some(next),
            vec![event(3, EventKind::Decision, "new decision")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    assert!(packs.validate_task(&current).await.is_err());
}

#[tokio::test]
async fn mixed_role_chain_promotes_only_consultant_facts_and_current_goal_can_report_dirty_tasks() {
    let f = Fixture::new();
    let packs = f.packs();
    let (executor, _) = session(&f, SessionRole::Executor, SessionState::Exited);
    let cp = packs
        .checkpoint(
            &f.task.scope(),
            executor.id,
            None,
            vec![event(1, EventKind::Decision, "executor-only")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    let (consultant, _) = session(&f, SessionRole::Consultant, SessionState::Exited);
    let cp = packs
        .checkpoint(
            &f.task.scope(),
            consultant.id,
            Some(cp),
            vec![event(2, EventKind::Decision, "consultant-decision")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    assert_eq!(packs.load_checkpoint(&cp).unwrap().retained.len(), 2);
    let mut task = Task::new(
        f.project.id,
        f.task.goal_id,
        "Other Task".into(),
        "fake".into(),
    );
    {
        let mut store = f.store.lock().unwrap();
        store.put_task(&mut task).unwrap();
        WorktreeManager::create(&mut store, task.id).unwrap();
        task = store.task(task.id).unwrap().unwrap();
    }
    let promoted = packs
        .draft_task(
            &task.scope(),
            TaskInputs {
                checkpoint: Some(cp.clone()),
                ..input()
            },
        )
        .await
        .unwrap();
    let historical = promoted
        .pack()
        .historical_checkpoint
        .as_ref()
        .unwrap()
        .to_string();
    assert!(historical.contains("consultant-decision"));
    assert!(!historical.contains("executor-only"));
    let task_ref = packs
        .publish_task(
            &packs
                .draft_task(
                    &f.task.scope(),
                    TaskInputs {
                        checkpoint: Some(cp),
                        ..input()
                    },
                )
                .await
                .unwrap(),
        )
        .await
        .unwrap();
    let goal = packs
        .publish_goal(
            &Scope::goal(f.project.id, f.task.goal_id),
            vec![],
            Default::default(),
        )
        .await
        .unwrap();
    let (_, _) = session(&f, SessionRole::Executor, SessionState::Running);
    std::fs::write(
        f.worktree.join("src/codec.rs"),
        "pub fn encode() { panic!(\"dirty\"); }\n",
    )
    .unwrap();
    assert!(packs.validate_task(&task_ref).await.is_err());
    packs.validate_goal(&goal).await.unwrap();
    let latest = packs
        .publish_goal(&goal.scope, vec![], Default::default())
        .await
        .unwrap();
    let data = packs.goal_pack(&latest).unwrap();
    assert!(data.tasks[0].source_validation_required);
    assert!(!data.tasks[0].historical);
}

#[tokio::test]
async fn terminal_task_and_goal_never_prepare_draft_native_input() {
    let f = Fixture::new();
    let packs = f.packs();
    let draft = packs.draft_task(&f.task.scope(), input()).await.unwrap();
    let DraftPreparation::Ready(payload) = packs
        .prepare_draft(&draft, SelectionRequest::default(), budget())
        .await
        .unwrap()
    else {
        panic!("source budget");
    };
    let header: serde_json::Value =
        serde_json::from_str(payload.payload.lines().next().unwrap()).unwrap();
    assert!(header["version"].is_null());
    assert_eq!(payload.scope, f.task.scope());
    {
        let mut store = f.store.lock().unwrap();
        let mut task = store.task(f.task.id).unwrap().unwrap();
        task.state = TaskState::Cancelled;
        store.put_task(&mut task).unwrap();
    }
    assert!(f.worktree.exists());
    assert!(packs.draft_task(&f.task.scope(), input()).await.is_err());
    assert!(
        packs
            .prepare_draft(&draft, SelectionRequest::default(), budget())
            .await
            .is_err()
    );

    // Even an envelope matching current terminal metadata is historical.
    // A caller must not gain launch preparation by rehashing terminal state.
    let (p, g, t) = {
        let store = f.store.lock().unwrap();
        (
            store.project(f.project.id).unwrap().unwrap(),
            store.goal(f.task.goal_id).unwrap().unwrap(),
            store.task(f.task.id).unwrap().unwrap(),
        )
    };
    let project = |value: serde_json::Value| {
        let mut value = value;
        for k in ["version", "context_version", "created_at", "updated_at"] {
            value.as_object_mut().unwrap().remove(k);
        }
        value
    };
    use sha2::{Digest, Sha256};
    let mut envelope = draft.context_version(1).unwrap();
    let p = project(serde_json::to_value(p).unwrap());
    let g = project(serde_json::to_value(g).unwrap());
    let t = project(serde_json::to_value(t).unwrap());
    envelope.data["task"] = t.clone();
    envelope.data["goal"] = g.clone();
    envelope.data["authority_digest"] = serde_json::json!(format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(&(p, g, t)).unwrap())
    ));
    {
        let mut store = f.store.lock().unwrap();
        assert!(store.put_context(&envelope).is_err());
    }
    corrupt_context(&f, &envelope);
    let reference = PackRef {
        scope: envelope.scope.clone(),
        version: 1,
        digest: format!(
            "sha256:{:x}",
            Sha256::digest(serde_json::to_vec(&envelope).unwrap())
        ),
    };
    assert!(
        packs
            .prepare_task(&reference, SelectionRequest::default(), budget())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn genuine_foreign_project_and_goal_checkpoints_in_one_store_cannot_promote() {
    let f = Fixture::new();
    let other = Fixture::new();
    let packs = f.packs();
    // Both genuine repositories and checkpoint records share one Store so the
    // promotion scope gate, rather than missing-record/digest checks, is causal.
    let mut p = other.project.clone();
    p.version = 0;
    let mut g = other
        .store
        .lock()
        .unwrap()
        .goal(other.task.goal_id)
        .unwrap()
        .unwrap();
    g.version = 0;
    g.dag.nodes.clear();
    let mut t = other.task.clone();
    t.version = 0;
    {
        let mut store = f.store.lock().unwrap();
        store.put_project(&mut p).unwrap();
        store.put_goal(&mut g).unwrap();
        store.put_task(&mut t).unwrap();
        g.dag.nodes.push(t.id);
        store.put_goal(&mut g).unwrap();
    }
    let (mut native, _) = session(&other, SessionRole::Consultant, SessionState::Exited);
    native.id = SessionId::new();
    f.store.lock().unwrap().put_session(&native, 0).unwrap();
    let foreign = packs
        .checkpoint(
            &t.scope(),
            native.id,
            None,
            vec![event(1, EventKind::Decision, "foreign Project decision")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    packs.load_checkpoint(&foreign).unwrap();
    let error = packs
        .draft_task(
            &f.task.scope(),
            TaskInputs {
                checkpoint: Some(foreign),
                ..input()
            },
        )
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("foreign consultation/checkpoint promotion"),
        "{error:#}"
    );
    let mut goal = Goal::new(
        f.project.id,
        "Other Goal".into(),
        vec![CompletionCriterion {
            id: "done".into(),
            description: "verified".into(),
            satisfied: false,
            evidence: None,
        }],
    );
    let mut task = Task::new(
        f.project.id,
        goal.id,
        "Other Goal consultation".into(),
        "fake".into(),
    );
    {
        let mut store = f.store.lock().unwrap();
        store.put_goal(&mut goal).unwrap();
        store.put_task(&mut task).unwrap();
        WorktreeManager::create(&mut store, task.id).unwrap();
        task = store.task(task.id).unwrap().unwrap();
    }
    native.id = SessionId::new();
    native.scope = task.scope();
    native.worktree = task.worktree.clone().unwrap();
    f.store.lock().unwrap().put_session(&native, 0).unwrap();
    let foreign = packs
        .checkpoint(
            &task.scope(),
            native.id,
            None,
            vec![event(1, EventKind::Decision, "foreign Goal decision")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    packs.load_checkpoint(&foreign).unwrap();
    let error = packs
        .draft_task(
            &f.task.scope(),
            TaskInputs {
                checkpoint: Some(foreign),
                ..input()
            },
        )
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("foreign consultation/checkpoint promotion"),
        "{error:#}"
    );
}

#[tokio::test]
async fn typed_pack_generic_writers_cannot_move_pointers_or_hide_unresolved_facts() {
    let f = Fixture::new();
    let packs = f.packs();
    let first = packs
        .publish_task(&packs.draft_task(&f.task.scope(), input()).await.unwrap())
        .await
        .unwrap();
    let second = packs
        .publish_task(
            &packs
                .draft_task(
                    &f.task.scope(),
                    TaskInputs {
                        failures: vec!["X unresolved".into()],
                        ..input()
                    },
                )
                .await
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.version, 2);
    packs.validate_task(&second).await.unwrap();
    {
        let mut store = f.store.lock().unwrap();
        let mut task = store.task(f.task.id).unwrap().unwrap();
        task.context_version = first.version;
        assert!(store.put_task(&mut task).is_err());
        let mut append = store.context(&f.task.scope(), None).unwrap().unwrap();
        append.version += 1;
        append.data = serde_json::json!({"opaque":"cannot poison a typed owner"});
        assert!(store.put_context(&append).is_err());
    }
    let (native, _) = session(&f, SessionRole::Executor, SessionState::Running);
    {
        let mut store = f.store.lock().unwrap();
        let mut task = store.task(f.task.id).unwrap().unwrap();
        task.context_version = first.version;
        assert!(store.put_task(&mut task).is_err());
        let (mut native, version) = store.session(native.id).unwrap().unwrap();
        native.state = SessionState::Lost;
        store.put_session(&native, version).unwrap();
        task.context_version = 0;
        assert!(store.put_task(&mut task).is_err());
    }
    assert!(packs.validate_task(&first).await.is_err());
    // Corrupt only the stored pointer to exercise the reader independently of writers.
    let original = f.store.lock().unwrap().task(f.task.id).unwrap().unwrap();
    let mut rollback = original.clone();
    rollback.context_version = first.version;
    let raw = rusqlite::Connection::open(f._temp.path().join("state.db")).unwrap();
    raw.execute(
        "UPDATE tasks SET body=?1 WHERE id=?2",
        rusqlite::params![
            serde_json::to_string(&rollback).unwrap(),
            rollback.id.to_string()
        ],
    )
    .unwrap();
    assert!(packs.validate_task(&first).await.is_err());
    raw.execute(
        "UPDATE tasks SET body=?1 WHERE id=?2",
        rusqlite::params![
            serde_json::to_string(&original).unwrap(),
            original.id.to_string()
        ],
    )
    .unwrap();
    packs.validate_task(&second).await.unwrap();
    assert_eq!(
        packs.task_pack(&second).unwrap().failures,
        vec!["X unresolved"]
    );
    let goal = packs
        .publish_goal(
            &Scope::goal(f.project.id, f.task.goal_id),
            vec![],
            Default::default(),
        )
        .await
        .unwrap();
    {
        let mut store = f.store.lock().unwrap();
        let mut g = store.goal(f.task.goal_id).unwrap().unwrap();
        g.context_version = 0;
        assert!(store.put_goal(&mut g).is_err());
    }
    packs.validate_goal(&goal).await.unwrap();
}

#[tokio::test]
async fn cross_task_consultation_is_an_explicit_immutable_snapshot_not_a_live_head_dependency() {
    let f = Fixture::new();
    let packs = f.packs();
    let (consultant, _) = session(&f, SessionRole::Consultant, SessionState::Exited);
    let cp = packs
        .checkpoint(
            &f.task.scope(),
            consultant.id,
            None,
            vec![event(1, EventKind::Decision, "promoted snapshot")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    let mut target = Task::new(f.project.id, f.task.goal_id, "Target".into(), "fake".into());
    {
        let mut store = f.store.lock().unwrap();
        store.put_task(&mut target).unwrap();
        WorktreeManager::create(&mut store, target.id).unwrap();
        target = store.task(target.id).unwrap().unwrap();
    }
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
    let reference = packs.publish_task(&draft).await.unwrap();
    // Publication advances Task DB authority. Capture a current target draft before
    // the independent source checkpoint append so only that append is varied.
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
    let source_versions = draft.source_versions();
    assert!(!source_versions.contains_key("checkpoint:head"));
    packs
        .checkpoint(
            &f.task.scope(),
            consultant.id,
            Some(cp.clone()),
            vec![event(
                2,
                EventKind::Failure,
                "source Task continues independently",
            )],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    packs.validate_task(&reference).await.unwrap();
    let DraftPreparation::Ready(prepared) = packs
        .prepare_draft(&draft, SelectionRequest::default(), budget())
        .await
        .unwrap()
    else {
        panic!("budget");
    };
    assert_eq!(prepared.source_versions, source_versions);
    assert!(
        !prepared
            .payload
            .contains("source Task continues independently")
    );
    assert!(prepared.payload.contains("promoted snapshot"));
    assert_eq!(
        packs
            .publish_task(
                &packs
                    .draft_task(
                        &target.scope(),
                        TaskInputs {
                            checkpoint: Some(cp),
                            ..input()
                        }
                    )
                    .await
                    .unwrap()
            )
            .await
            .unwrap(),
        reference
    );
    let historical = packs
        .task_pack(&reference)
        .unwrap()
        .historical_checkpoint
        .unwrap();
    assert!(
        historical["mandatory_task_at_checkpoint"]
            .get("worktree")
            .is_none()
    );
    assert!(
        historical["mandatory_task_at_checkpoint"]
            .get("executor")
            .is_none()
    );
}
