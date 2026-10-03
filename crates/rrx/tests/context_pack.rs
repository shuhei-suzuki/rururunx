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
    let head = f
        .store
        .lock()
        .unwrap()
        .context(&f.task.scope(), None)
        .unwrap()
        .and_then(|c| c.data["checkpoint"]["digest"].as_str().map(str::to_owned))
        .unwrap_or_else(|| "none".into());
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
        recovery: serde_json::json!({"source_versions":{"checkpoint:head":head}}),
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
        .historical_consultation
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
    let stale_task = f.store.lock().unwrap().task(f.task.id).unwrap().unwrap();
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
    {
        let mut stale = stale_task;
        stale.next_action = Some("concurrent metadata".into());
        let error = f.store.lock().unwrap().put_task(&mut stale).unwrap_err();
        assert!(matches!(
            error.downcast_ref::<rrx::state::StateGuardError>(),
            Some(rrx::state::StateGuardError::SnapshotChanged { .. })
        ));
    }
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
    assert!(
        packs
            .publish_goal(
                &Scope::goal(f.project.id, f.task.goal_id),
                vec![],
                Default::default()
            )
            .await
            .is_err()
    );
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
    let stale_goal = f
        .store
        .lock()
        .unwrap()
        .goal(f.task.goal_id)
        .unwrap()
        .unwrap();
    let newer = packs
        .publish_goal(
            &Scope::goal(f.project.id, f.task.goal_id),
            vec!["new decision".into()],
            Default::default(),
        )
        .await
        .unwrap();
    {
        let mut stale = stale_goal.clone();
        stale.blockers.push("concurrent metadata".into());
        let error = f.store.lock().unwrap().put_goal(&mut stale).unwrap_err();
        assert!(error.to_string().contains("typed pack pointer"));
        let mut append = f
            .store
            .lock()
            .unwrap()
            .context(&goal.scope, None)
            .unwrap()
            .unwrap();
        append.version += 1;
        append.data = serde_json::json!({"opaque":"cannot replace Goal authority"});
        assert!(f.store.lock().unwrap().put_context(&append).is_err());
    }
    let original = f
        .store
        .lock()
        .unwrap()
        .goal(f.task.goal_id)
        .unwrap()
        .unwrap();
    let mut rollback = original.clone();
    rollback.context_version = goal.version;
    raw.execute(
        "UPDATE goals SET body=?1 WHERE id=?2",
        rusqlite::params![
            serde_json::to_string(&rollback).unwrap(),
            rollback.id.to_string()
        ],
    )
    .unwrap();
    assert!(packs.validate_goal(&goal).await.is_err());
    raw.execute(
        "UPDATE goals SET body=?1 WHERE id=?2",
        rusqlite::params![
            serde_json::to_string(&original).unwrap(),
            original.id.to_string()
        ],
    )
    .unwrap();
    packs.validate_goal(&newer).await.unwrap();
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
    assert_eq!(source_versions.get("checkpoint:head").unwrap(), "none");
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
                            checkpoint: Some(cp.clone()),
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
        .historical_consultation
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
    let native = Session {
        id: SessionId::new(),
        scope: target.scope(),
        agent: "fake".into(),
        provider: "fixture".into(),
        role: SessionRole::Executor,
        native_ref: None,
        pid: None,
        worktree: target.worktree.clone().unwrap(),
        state: SessionState::Exited,
        model: None,
        effort: None,
        recovery: Default::default(),
        started_at: 0,
    };
    f.store.lock().unwrap().put_session(&native, 0).unwrap();
    let own = packs
        .checkpoint(
            &target.scope(),
            native.id,
            None,
            vec![event(1, EventKind::Failure, "target unresolved failure")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    let both = packs
        .draft_task(
            &target.scope(),
            TaskInputs {
                promoted_consultation: Some(cp),
                ..input()
            },
        )
        .await
        .unwrap();
    assert_eq!(both.pack().checkpoint.as_ref(), Some(&own));
    assert!(
        both.pack()
            .historical_checkpoint
            .as_ref()
            .unwrap()
            .to_string()
            .contains("target unresolved failure")
    );
    assert!(
        both.pack()
            .historical_consultation
            .as_ref()
            .unwrap()
            .to_string()
            .contains("promoted snapshot")
    );
}

#[tokio::test]
async fn own_checkpoint_is_mandatory_even_when_the_caller_omits_its_reference() {
    let f = Fixture::new();
    let packs = f.packs();
    let draft = packs.draft_task(&f.task.scope(), input()).await.unwrap();
    assert_eq!(draft.source_versions()["checkpoint:head"], "none");
    let reference = packs.publish_task(&draft).await.unwrap();
    let DraftPreparation::Ready(before) = packs
        .prepare_draft(
            &packs.draft_task(&f.task.scope(), input()).await.unwrap(),
            SelectionRequest::default(),
            budget(),
        )
        .await
        .unwrap()
    else {
        panic!("budget")
    };
    let (native, _) = session(&f, SessionRole::Executor, SessionState::Exited);
    let cp = packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            None,
            vec![
                event(1, EventKind::Constraint, "never remove safety constraints"),
                event(2, EventKind::Failure, "failure is unresolved"),
            ],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    assert!(packs.validate_task(&reference).await.is_err());
    assert!(
        packs
            .prepare_task(&reference, SelectionRequest::default(), budget())
            .await
            .is_err()
    );
    assert!(packs.publish_task(&draft).await.is_err());
    let fresh = packs.draft_task(&f.task.scope(), input()).await.unwrap();
    assert_eq!(fresh.pack().checkpoint.as_ref(), Some(&cp));
    assert_eq!(fresh.source_versions()["checkpoint:head"], cp.digest);
    assert_ne!(
        before.source_versions["checkpoint:head"],
        fresh.source_versions()["checkpoint:head"]
    );
    let latest = packs.publish_task(&fresh).await.unwrap();
    assert!(latest.version > reference.version);
    let PreparedPack::Ready(prepared) = packs
        .prepare_task(&latest, SelectionRequest::default(), budget())
        .await
        .unwrap()
    else {
        panic!("budget")
    };
    assert!(prepared.payload.contains("never remove safety constraints"));
    assert!(prepared.payload.contains("failure is unresolved"));
    assert_eq!(prepared.source_versions["checkpoint:head"], cp.digest);
    let sources = std::collections::BTreeMap::from([("checkpoint:head".into(), cp.digest.clone())]);
    f.store
        .lock()
        .unwrap()
        .validate_checkpoint_source(&f.task.scope(), &sources)
        .unwrap();
    let mut launching = native.clone();
    launching.id = SessionId::new();
    launching.state = SessionState::Starting;
    launching.recovery = serde_json::json!({"source_versions":prepared.source_versions});
    let version = f.store.lock().unwrap().put_session(&launching, 0).unwrap();
    packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            Some(cp),
            vec![event(3, EventKind::Failure, "later failure")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    assert!(
        f.store
            .lock()
            .unwrap()
            .validate_checkpoint_source(&f.task.scope(), &sources)
            .is_err()
    );
    launching.state = SessionState::Running;
    assert!(
        f.store
            .lock()
            .unwrap()
            .put_session(&launching, version)
            .is_err()
    );
    launching.state = SessionState::Stopped;
    f.store
        .lock()
        .unwrap()
        .put_session(&launching, version)
        .unwrap();
}

#[tokio::test]
async fn checkpoint_v4_index_migrates_atomically_and_is_independent_of_record_rowids() {
    let f = Fixture::new();
    let packs = f.packs();
    let (native, _) = session(&f, SessionRole::Consultant, SessionState::Exited);
    let first = packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            None,
            vec![event(1, EventKind::Decision, "first")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    let last = packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            Some(first),
            vec![event(2, EventKind::Failure, "unresolved")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    let db = f._temp.path().join("state.db");
    let raw = rusqlite::Connection::open(&db).unwrap();
    raw.execute_batch("DROP TABLE checkpoint_heads; PRAGMA user_version=3;")
        .unwrap();
    let restored = Store::open(&db).unwrap();
    assert_eq!(restored.schema_version().unwrap(), 4);
    let scope = f.task.scope();
    let sources =
        std::collections::BTreeMap::from([("checkpoint:head".into(), last.digest.clone())]);
    restored
        .validate_checkpoint_source(&scope, &sources)
        .unwrap();
    raw.execute_batch("VACUUM;").unwrap();
    restored
        .validate_checkpoint_source(&scope, &sources)
        .unwrap();
    let artifact = packs.draft_task(&scope, input()).await.unwrap();
    assert_eq!(artifact.pack().checkpoint, Some(last));
    // Invalid migration history leaves neither the v4 marker nor its index behind.
    raw.execute_batch("DROP TABLE checkpoint_heads; PRAGMA user_version=3;")
        .unwrap();
    let mut row = f
        .store
        .lock()
        .unwrap()
        .record(artifact.pack().checkpoint.as_ref().unwrap().id)
        .unwrap()
        .unwrap();
    row.data["previous"]["digest"] = serde_json::json!("sha256:bad");
    raw.execute(
        "UPDATE records SET body=?1 WHERE id=?2",
        rusqlite::params![serde_json::to_string(&row).unwrap(), row.id.to_string()],
    )
    .unwrap();
    assert!(Store::open(&db).is_err());
    let marker: i64 = raw
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(marker, 3);
    let tables: i64 = raw
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema WHERE name='checkpoint_heads'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(tables, 0);
}

// Inference is deliberately synthetic; repository capture, Engine publication,
// native Session persistence and Git disposal use the actual implementations.
struct PackFixtureAgent {
    store: SharedStore,
    sessions: Mutex<std::collections::BTreeMap<SessionId, Session>>,
    name: String,
}
impl rrx::adapter::AgentAdapter for PackFixtureAgent {
    fn capabilities(&self) -> std::collections::BTreeSet<rrx::adapter::Capability> {
        use rrx::adapter::Capability::*;
        std::collections::BTreeSet::from([Execute, Review, NonInteractive])
    }
    fn probe(&self) -> rrx::adapter::AdapterResult<rrx::adapter::AgentInfo> {
        Ok(rrx::adapter::AgentInfo {
            agent: self.name.clone(),
            provider: "fixture".into(),
            adapter_version: "fixture".into(),
            executable: "fixture".into(),
            authenticated: None,
            model_configuration: false,
            effort_configuration: false,
            capabilities: self.capabilities(),
        })
    }
    fn start(
        &self,
        request: rrx::adapter::LaunchRequest,
    ) -> rrx::adapter::AdapterFuture<'_, Session> {
        Box::pin(async move {
            let mut s = Session {
                id: SessionId::new(),
                scope: request.scope,
                agent: self.name.clone(),
                provider: "fixture".into(),
                role: request.role,
                native_ref: None,
                pid: None,
                worktree: request.worktree,
                state: SessionState::Starting,
                model: None,
                effort: None,
                recovery: serde_json::json!({"source_versions":request.input.source_versions,"input_version":request.input.version}),
                started_at: now_ms(),
            };
            let mut store = self.store.lock().unwrap();
            let v = store.put_session(&s, 0).map_err(pack_fixture_error)?;
            s.state = SessionState::Running;
            store.put_session(&s, v).map_err(pack_fixture_error)?;
            self.sessions.lock().unwrap().insert(s.id, s.clone());
            Ok(s)
        })
    }
    fn status(
        &self,
        reference: rrx::adapter::SessionRef,
    ) -> rrx::adapter::AdapterFuture<'_, rrx::adapter::SessionStatus> {
        Box::pin(async move {
            let mut store = self.store.lock().unwrap();
            let (mut s, v) = store.session(reference.id).unwrap().unwrap();
            assert_eq!(s.scope, reference.scope);
            if s.state == SessionState::Running {
                s.state = SessionState::Exited;
                store.put_session(&s, v).map_err(pack_fixture_error)?;
            }
            Ok(rrx::adapter::SessionStatus {
                session: s,
                exit_code: Some(0),
                stdout: vec![],
                stderr: vec![],
                stdout_truncated: false,
                stderr_truncated: false,
                failure: None,
            })
        })
    }
    fn stop(
        &self,
        reference: rrx::adapter::SessionRef,
    ) -> rrx::adapter::AdapterFuture<'_, rrx::adapter::SessionStatus> {
        self.status(reference)
    }
    fn attach(&self, _: rrx::adapter::SessionRef) -> rrx::adapter::AdapterFuture<'_, ()> {
        Box::pin(async { Err(pack_fixture_error(anyhow::anyhow!("fixture unsupported"))) })
    }
    fn resume(&self, _: rrx::adapter::SessionRef) -> rrx::adapter::AdapterFuture<'_, Session> {
        Box::pin(async { Err(pack_fixture_error(anyhow::anyhow!("fixture unsupported"))) })
    }
    fn release(&self, _: rrx::adapter::SessionRef) -> rrx::adapter::AdapterResult<()> {
        Ok(())
    }
    fn subscribe(
        &self,
        _: rrx::adapter::SessionRef,
    ) -> rrx::adapter::AdapterResult<tokio::sync::watch::Receiver<rrx::adapter::SessionStatus>>
    {
        Err(pack_fixture_error(anyhow::anyhow!("fixture unsupported")))
    }
    fn usage(
        &self,
        reference: rrx::adapter::SessionRef,
        phase: String,
        round: Option<u32>,
    ) -> rrx::adapter::AdapterFuture<'_, Usage> {
        Box::pin(async move {
            Ok(Usage {
                scope: reference.scope,
                session_id: reference.id,
                agent: self.name.clone(),
                phase,
                review_round: round,
                input_tokens: None,
                cached_input_tokens: None,
                output_tokens: None,
                estimated_cost: None,
                context_pack_version: None,
                context_pack_size: None,
                repo_map_size: None,
                cache_metadata: serde_json::Value::Null,
                missing_reason: Some("synthetic fixture has no provider telemetry".into()),
            })
        })
    }
}
fn pack_fixture_error(error: anyhow::Error) -> rrx::adapter::AdapterError {
    rrx::adapter::AdapterError {
        kind: rrx::adapter::ErrorKind::StateFailure,
        message: error.to_string(),
    }
}
struct PackFixtureGates {
    cleanup: Mutex<Option<serde_json::Value>>,
    cleanup_wait_once: Mutex<bool>,
}
impl rrx::workflow::PhaseGates for PackFixtureGates {
    fn complete(
        &self,
        i: rrx::workflow::PhaseInvocation,
        status: Option<rrx::adapter::SessionStatus>,
    ) -> rrx::workflow::WorkflowFuture<'_, rrx::workflow::GateOutcome> {
        Box::pin(async move {
            if i.phase == rrx::workflow::Phase::Cleanup {
                let mut wait = self.cleanup_wait_once.lock().unwrap();
                if *wait {
                    *wait = false;
                    return Ok(rrx::workflow::GateOutcome::Waiting(
                        "fixture checkpoint during Cleanup wait".into(),
                    ));
                }
                drop(wait);
                *self.cleanup.lock().unwrap() = Some(i.context.data["task_pack"].clone());
                git(
                    &i.project.root,
                    &[
                        "worktree",
                        "remove",
                        i.task.worktree.as_ref().unwrap().to_str().unwrap(),
                    ],
                );
            }
            Ok(rrx::workflow::GateOutcome::Passed(
                rrx::workflow::Evidence {
                    scope: i.task.scope(),
                    phase: i.phase,
                    revision: i.sources.revision,
                    dependencies: i.sources.source_versions.clone(),
                    source_versions: i.sources.source_versions,
                    artifacts: vec![format!("fixture://{}", i.phase.key())],
                    review_approved: (i.phase.actor() == rrx::workflow::Actor::Reviewer)
                        .then_some(true),
                    session_id: status.map(|s| s.session.id),
                    context_version: i.context.version,
                },
            ))
        })
    }
}
fn phase_config() -> rrx::config::Config {
    let mut config = rrx::config::Config::default();
    config.minimum_workflow = rrx::config::WorkflowClass::Quick;
    config.workflow.default = rrx::config::WorkflowClass::Quick;
    config.context.repo_map_tokens = 64_000;
    config.context.review_context_tokens = 64_000;
    config
}
#[tokio::test]
async fn actual_workflow_publishes_typed_phase_packs_and_freezes_cleanup_provenance() {
    use rrx::{
        adapter::AgentRegistry,
        context_pack::workflow::{PhasePackArtifact, WorkflowPackSources},
        workflow::{StepResult, WorkflowEngine},
    };
    let f = Fixture::new();
    let mut task = Task::new(
        f.project.id,
        f.task.goal_id,
        "Phase target".into(),
        "fake".into(),
    );
    task.workflow = rrx::config::WorkflowClass::Quick;
    task.risk = RiskClass::R0;
    task.reviewers = vec!["fixture-reviewer".into()];
    {
        let mut store = f.store.lock().unwrap();
        store.put_task(&mut task).unwrap();
        WorktreeManager::create(&mut store, task.id).unwrap();
        task = store.task(task.id).unwrap().unwrap();
    }
    let worktree = task.worktree.clone().unwrap();
    let packs = f.packs();
    let sources = Arc::new(WorkflowPackSources::new(packs.clone()));
    sources.set_inputs(&task.scope(), input()).unwrap();
    let mut registry = AgentRegistry::default();
    for name in ["fake", "fixture-reviewer"] {
        registry
            .register(
                name.into(),
                Arc::new(PackFixtureAgent {
                    store: f.store.clone(),
                    sessions: Mutex::new(Default::default()),
                    name: name.into(),
                }),
            )
            .unwrap();
    }
    let gates = Arc::new(PackFixtureGates {
        cleanup: Mutex::new(None),
        cleanup_wait_once: Mutex::new(true),
    });
    let engine = WorkflowEngine::new(
        f.store.clone(),
        Arc::new(registry),
        phase_config(),
        sources,
        gates.clone(),
    )
    .unwrap();
    let first = engine.initialize(task.id, None).await.unwrap();
    let c = f
        .store
        .lock()
        .unwrap()
        .context(&task.scope(), Some(first.context_version))
        .unwrap()
        .unwrap();
    let artifact: PhasePackArtifact = serde_json::from_value(c.data["task_pack"].clone()).unwrap();
    assert_eq!(artifact.pack.scope, task.scope());
    assert_eq!(
        c.data["payload"]
            .as_str()
            .unwrap()
            .matches("MANDATORY: preserve Project boundaries.")
            .count(),
        1
    );
    assert_eq!(
        c.data["rendered_estimate"]["estimated_bytes"]
            .as_u64()
            .unwrap() as usize,
        c.data["payload"].as_str().unwrap().len()
    );
    let mut finalized = false;
    for _ in 0..40 {
        let snapshot = engine.snapshot(task.id).unwrap();
        if snapshot.finished {
            if !finalized {
                engine
                    .request_finalization(task.id, "fixture disposal proof".into())
                    .await
                    .unwrap();
                finalized = true;
                continue;
            }
            break;
        }
        let result = engine.step(task.id, Default::default()).await.unwrap();
        if let StepResult::Waiting {
            phase: rrx::workflow::Phase::Cleanup,
            ..
        } = result
        {
            assert!(worktree.exists());
            let native = {
                let store = f.store.lock().unwrap();
                store
                    .records(&task.scope(), RecordKind::Session)
                    .unwrap()
                    .into_iter()
                    .map(|r| serde_json::from_value::<Session>(r.data).unwrap())
                    .find(|s| s.role == SessionRole::Executor && s.state == SessionState::Exited)
                    .unwrap()
            };
            let late = packs
                .checkpoint(
                    &task.scope(),
                    native.id,
                    None,
                    vec![event(
                        1,
                        EventKind::Failure,
                        "late Cleanup checkpoint remains durable",
                    )],
                    HistoryPolicy {
                        recent_history_bytes: 0,
                    },
                )
                .await
                .unwrap();
            assert!(
                packs
                    .load_checkpoint(&late)
                    .unwrap()
                    .retained
                    .iter()
                    .any(|e| e.event.text == "late Cleanup checkpoint remains durable")
            );
            engine.resume_gate(task.id).await.unwrap();
        } else {
            assert!(
                !matches!(
                    result,
                    StepResult::Waiting { .. } | StepResult::Failed { .. }
                ),
                "{result:?}"
            );
        }
    }
    let final_state = engine.snapshot(task.id).unwrap();
    assert!(final_state.finished);
    assert!(!worktree.exists());
    let c = f
        .store
        .lock()
        .unwrap()
        .context(&task.scope(), None)
        .unwrap()
        .unwrap();
    assert_eq!(c.data["frozen_task_pack"], true);
    assert_eq!(
        c.data["task_pack"],
        gates.cleanup.lock().unwrap().clone().unwrap()
    );
    let proof = f.store.lock().unwrap().task(task.id).unwrap().unwrap();
    assert_eq!(proof.state, TaskState::Completed);
    let goal = packs
        .publish_goal(
            &Scope::goal(task.project_id, task.goal_id),
            vec![],
            Default::default(),
        )
        .await
        .unwrap();
    let summary = packs.goal_pack(&goal).unwrap();
    assert!(
        summary
            .tasks
            .iter()
            .find(|t| t.id == task.id)
            .unwrap()
            .historical
    );
}
#[tokio::test]
async fn workflow_capture_cache_binds_complete_scope_payload_phase_and_budget() {
    use rrx::{
        context_pack::workflow::WorkflowPackSources,
        workflow::{BudgetClass, ContextBudget, Phase, WorkflowSources},
    };
    let f = Fixture::new();
    let source = WorkflowPackSources::new(f.packs());
    let p = f
        .store
        .lock()
        .unwrap()
        .project(f.project.id)
        .unwrap()
        .unwrap();
    let t = f.store.lock().unwrap().task(f.task.id).unwrap().unwrap();
    let original = source
        .capture(
            p.clone(),
            t.clone(),
            Phase::Implement,
            ContextBudget {
                class: BudgetClass::Normal,
                discretionary_tokens: 64_000,
            },
        )
        .await
        .unwrap();
    assert!(source.pack_artifact(&original).unwrap().is_some());
    for kind in 0..4 {
        let mut changed = original.clone();
        match kind {
            0 => changed.scope.project_id = ProjectId::new(),
            1 => changed.payload.push('x'),
            2 => changed.revision.push('x'),
            _ => {
                changed
                    .source_versions
                    .insert("foreign".into(), "hash".into());
            }
        };
        assert!(source.pack_artifact(&changed).is_err());
    }
    let other = source
        .capture(
            p,
            t,
            Phase::ImplementationReview,
            ContextBudget {
                class: BudgetClass::Broad,
                discretionary_tokens: 63_000,
            },
        )
        .await
        .unwrap();
    assert_eq!(original.source_versions, other.source_versions);
    assert_ne!(original.payload, other.payload);
    assert!(source.pack_artifact(&other).unwrap().is_some());
}

#[tokio::test]
async fn goal_summary_includes_unplanned_tasks_and_rejects_membership_staleness() {
    let f = Fixture::new();
    let packs = f.packs();
    let scope = Scope {
        project_id: f.project.id,
        goal_id: Some(f.task.goal_id),
        task_id: None,
    };
    let old = packs
        .publish_goal(&scope, vec![], Default::default())
        .await
        .unwrap();
    packs.validate_goal(&old).await.unwrap();
    let mut added = Task::new(
        f.project.id,
        f.task.goal_id,
        "Unplanned owned Task".into(),
        "fake".into(),
    );
    f.store.lock().unwrap().put_task(&mut added).unwrap();
    assert!(
        !f.store
            .lock()
            .unwrap()
            .goal(f.task.goal_id)
            .unwrap()
            .unwrap()
            .dag
            .nodes
            .contains(&added.id)
    );
    assert!(packs.validate_goal(&old).await.is_err());
    let current = packs
        .publish_goal(&scope, vec![], Default::default())
        .await
        .unwrap();
    let pack = packs.goal_pack(&current).unwrap();
    assert_eq!(pack.tasks.len(), 2);
    assert!(
        pack.tasks
            .iter()
            .any(|t| t.id == added.id && t.context.is_none())
    );
    packs.validate_goal(&current).await.unwrap();
}

#[tokio::test]
async fn actual_engine_tiny_discretionary_budget_preserves_facts_and_provider_restart() {
    use rrx::{
        adapter::AgentRegistry,
        context_pack::workflow::{PhasePackArtifact, WorkflowPackSources},
        workflow::{BudgetClass, ContextBudget, Phase, WorkflowEngine, WorkflowSources},
    };
    let f = Fixture::new();
    let packs = f.packs();
    let (native, _) = session(&f, SessionRole::Executor, SessionState::Exited);
    let checkpoint = packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            None,
            vec![event(1, EventKind::Failure, "mandatory unresolved failure")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    let sources = Arc::new(WorkflowPackSources::new(packs.clone()));
    let mut inputs = input();
    inputs.verification = vec!["durable verification fact".into()];
    sources.set_inputs(&f.task.scope(), inputs).unwrap();
    let mut config = phase_config();
    config.context.repo_map_tokens = 1;
    let engine = WorkflowEngine::new(
        f.store.clone(),
        Arc::new(AgentRegistry::default()),
        config,
        sources,
        Arc::new(PackFixtureGates {
            cleanup: Mutex::new(None),
            cleanup_wait_once: Mutex::new(false),
        }),
    )
    .unwrap();
    let initialized = engine.initialize(f.task.id, None).await.unwrap();
    let (p, t, c) = {
        let store = f.store.lock().unwrap();
        (
            store.project(f.project.id).unwrap().unwrap(),
            store.task(f.task.id).unwrap().unwrap(),
            store
                .context(&f.task.scope(), Some(initialized.context_version))
                .unwrap()
                .unwrap(),
        )
    };
    let artifact: PhasePackArtifact = serde_json::from_value(c.data["task_pack"].clone()).unwrap();
    assert_eq!(artifact.budget.discretionary_tokens, 1);
    assert!(artifact.optional_bytes <= 1);
    assert!(artifact.mandatory_bytes > 1);
    assert_eq!(artifact.pack.checkpoint, Some(checkpoint));
    let payload = c.data["payload"].as_str().unwrap();
    for fact in [
        "Never remove Project safety constraints",
        "mandatory unresolved failure",
        "durable verification fact",
        "Use scoped authoritative references",
    ] {
        assert!(payload.contains(fact), "missing {fact}");
    }
    assert_eq!(
        payload
            .matches("MANDATORY: preserve Project boundaries.")
            .count(),
        1
    );
    assert_eq!(
        c.data["rendered_estimate"]["mandatory_bytes"]
            .as_u64()
            .unwrap()
            + c.data["rendered_estimate"]["optional_bytes"]
                .as_u64()
                .unwrap(),
        payload.len() as u64
    );
    assert!(c.data["rendered_estimate"]["measured_tokens"].is_null());
    let restarted = WorkflowPackSources::new(packs);
    let source = restarted
        .capture(
            p,
            t,
            Phase::Implement,
            ContextBudget {
                class: BudgetClass::Normal,
                discretionary_tokens: 1,
            },
        )
        .await
        .unwrap();
    let recovered: PhasePackArtifact =
        serde_json::from_value(restarted.pack_artifact(&source).unwrap().unwrap()).unwrap();
    assert_eq!(recovered.pack.decisions, artifact.pack.decisions);
    assert_eq!(recovered.pack.verification, artifact.pack.verification);
    assert_eq!(recovered.pack.artifacts, artifact.pack.artifacts);
    assert_eq!(recovered.source_versions, artifact.source_versions);
}

#[tokio::test]
async fn actual_engine_mandatory_rules_exceeding_absolute_cap_never_publish() {
    use rrx::{
        adapter::AgentRegistry, context_pack::workflow::WorkflowPackSources,
        workflow::WorkflowEngine,
    };
    let f = Fixture::new();
    let mut p = f
        .store
        .lock()
        .unwrap()
        .project(f.project.id)
        .unwrap()
        .unwrap();
    for n in 0..5 {
        let path = f.root.join(format!("mandatory-{n}.md"));
        std::fs::write(&path, "X".repeat(220_000)).unwrap();
        p.rule_refs.push(path);
    }
    f.store.lock().unwrap().put_project(&mut p).unwrap();
    let engine = WorkflowEngine::new(
        f.store.clone(),
        Arc::new(AgentRegistry::default()),
        phase_config(),
        Arc::new(WorkflowPackSources::new(f.packs())),
        Arc::new(PackFixtureGates {
            cleanup: Mutex::new(None),
            cleanup_wait_once: Mutex::new(false),
        }),
    )
    .unwrap();
    let error = engine.initialize(f.task.id, None).await.unwrap_err();
    assert!(
        format!("{error:#}").contains("absolute 1 MiB cap"),
        "{error:#}"
    );
    let store = f.store.lock().unwrap();
    assert_eq!(store.task(f.task.id).unwrap().unwrap().context_version, 0);
    assert!(store.context(&f.task.scope(), None).unwrap().is_none());
    assert!(
        store
            .records(&f.task.scope(), RecordKind::Workflow)
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn admitted_session_reentry_preserves_launch_head_after_incremental_checkpoint() {
    let f = Fixture::new();
    let packs = f.packs();
    let draft = packs.draft_task(&f.task.scope(), input()).await.unwrap();
    let reference = packs.publish_task(&draft).await.unwrap();
    let PreparedPack::Ready(prepared) = packs
        .prepare_task(&reference, SelectionRequest::default(), budget())
        .await
        .unwrap()
    else {
        panic!("budget")
    };
    let (mut native, mut version) = session(&f, SessionRole::Executor, SessionState::Starting);
    native.recovery = serde_json::json!({"source_versions":prepared.source_versions});
    native.state = SessionState::Running;
    version = f
        .store
        .lock()
        .unwrap()
        .put_session(&native, version)
        .unwrap();
    packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            None,
            vec![event(1, EventKind::Constraint, "new mandatory constraint")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    let initial = native.recovery.clone();
    for waiting in [
        SessionState::WaitingApproval,
        SessionState::WaitingHuman,
        SessionState::Lost,
    ] {
        native.state = waiting;
        version = f
            .store
            .lock()
            .unwrap()
            .put_session(&native, version)
            .unwrap();
        native.state = SessionState::Running;
        version = f
            .store
            .lock()
            .unwrap()
            .put_session(&native, version)
            .unwrap();
        assert_eq!(native.recovery, initial);
    }
    let mut stale = native.clone();
    stale.id = SessionId::new();
    stale.state = SessionState::Starting;
    assert!(f.store.lock().unwrap().put_session(&stale, 0).is_err());
    native.state = SessionState::Exited;
    f.store
        .lock()
        .unwrap()
        .put_session(&native, version)
        .unwrap();
    assert!(f.store.lock().unwrap().put_session(&stale, 0).is_err());
    let mut missing = stale;
    missing.recovery = serde_json::json!({});
    assert!(f.store.lock().unwrap().put_session(&missing, 0).is_err());
}

#[tokio::test]
async fn goal_pointer_publication_preserves_admitted_task_authority_and_rejects_stale_goal_refs() {
    let f = Fixture::new();
    let packs = f.packs();
    let task_ref = packs
        .publish_task(&packs.draft_task(&f.task.scope(), input()).await.unwrap())
        .await
        .unwrap();
    let PreparedPack::Ready(prepared) = packs
        .prepare_task(&task_ref, SelectionRequest::default(), budget())
        .await
        .unwrap()
    else {
        panic!("budget")
    };
    let (mut native, mut version) = session(&f, SessionRole::Executor, SessionState::Running);
    native.recovery = serde_json::json!({"source_versions":prepared.source_versions});
    let before = f
        .store
        .lock()
        .unwrap()
        .goal(f.task.goal_id)
        .unwrap()
        .unwrap();
    let goal = packs
        .publish_goal(&before.scope(), vec![], Default::default())
        .await
        .unwrap();
    let current = packs
        .publish_goal(
            &before.scope(),
            vec!["new Goal summary observation".into()],
            Default::default(),
        )
        .await
        .unwrap();
    assert_eq!(
        f.store
            .lock()
            .unwrap()
            .goal(f.task.goal_id)
            .unwrap()
            .unwrap()
            .version,
        before.version
    );
    assert!(current.version > goal.version);
    assert!(packs.validate_goal(&goal).await.is_err());
    packs.validate_goal(&current).await.unwrap();
    packs.validate_task(&task_ref).await.unwrap();
    native.state = SessionState::WaitingApproval;
    version = f
        .store
        .lock()
        .unwrap()
        .put_session(&native, version)
        .unwrap();
    native.state = SessionState::Running;
    version = f
        .store
        .lock()
        .unwrap()
        .put_session(&native, version)
        .unwrap();
    native.state = SessionState::Exited;
    f.store
        .lock()
        .unwrap()
        .put_session(&native, version)
        .unwrap();
    let reopened = Store::open(&f._temp.path().join("state.db")).unwrap();
    let g = reopened.goal(f.task.goal_id).unwrap().unwrap();
    assert_eq!(g.version, before.version);
    assert_eq!(g.context_version, current.version);
    let mut stale = before;
    stale.blockers.push("stale writer".into());
    assert!(f.store.lock().unwrap().put_goal(&mut stale).is_err());
    let PreparedPack::Ready(new_launch) = packs
        .prepare_task(&task_ref, SelectionRequest::default(), budget())
        .await
        .unwrap()
    else {
        panic!("budget")
    };
    assert_eq!(new_launch.version, prepared.version);
}

#[tokio::test]
async fn goal_summary_preserves_opaque_legacy_context_without_typed_claims() {
    let f = Fixture::new();
    let packs = f.packs();
    let context = ContextVersion {
        scope: f.task.scope(),
        version: 1,
        revision: git(&f.worktree, &["rev-parse", "HEAD"]),
        source_hashes: Default::default(),
        data: serde_json::json!({"legacy":"source-only Workflow provider"}),
    };
    {
        let mut store = f.store.lock().unwrap();
        store.put_context(&context).unwrap();
        let mut t = store.task(f.task.id).unwrap().unwrap();
        t.context_version = 1;
        store.put_task(&mut t).unwrap();
    }
    let reference = packs
        .publish_goal(
            &Scope::goal(f.project.id, f.task.goal_id),
            vec![],
            Default::default(),
        )
        .await
        .unwrap();
    let pack = packs.goal_pack(&reference).unwrap();
    let descriptor = pack.tasks.iter().find(|t| t.id == f.task.id).unwrap();
    assert!(!descriptor.typed_context);
    assert!(descriptor.context.is_some());
    assert!(
        packs
            .task_pack(descriptor.context.as_ref().unwrap())
            .is_err()
    );
    packs.validate_goal(&reference).await.unwrap();
}

#[tokio::test]
async fn checkpoint_admission_preserves_renderable_mandatory_headroom() {
    let f = Fixture::new();
    let packs = f.packs();
    let (native, _) = session(&f, SessionRole::Executor, SessionState::Exited);
    std::fs::write(f.root.join("RULES.md"), "R".repeat(220_000)).unwrap();
    let events = (1..=110)
        .map(|n| event(n, EventKind::Constraint, &"C".repeat(8192)))
        .collect();
    let before = f
        .store
        .lock()
        .unwrap()
        .events(&f.task.scope(), 0, 10000)
        .unwrap()
        .len();
    let error = packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            None,
            events,
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap_err();
    assert!(
        format!("{error:#}").contains("checkpoint admission"),
        "{error:#}"
    );
    assert!(
        f.store
            .lock()
            .unwrap()
            .records(&f.task.scope(), RecordKind::Checkpoint)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        f.store
            .lock()
            .unwrap()
            .events(&f.task.scope(), 0, 10000)
            .unwrap()
            .len(),
        before
    );
    let current = packs
        .checkpoint(
            &f.task.scope(),
            native.id,
            None,
            vec![event(1, EventKind::Constraint, "admitted mandatory fact")],
            HistoryPolicy {
                recent_history_bytes: 0,
            },
        )
        .await
        .unwrap();
    assert_eq!(packs.load_checkpoint(&current).unwrap().chain_version, 1);
    packs.draft_task(&f.task.scope(), input()).await.unwrap();
}
