//! Actual forwarding at the first reachable native phase; no native gate completion.
use super::{environment_tests::isolated, fixture_support::Fixture, *};
use crate::{
    config::{Config, WorkflowClass},
    domain::{RiskClass, TaskState},
    workflow::*,
};

struct Sources;
impl WorkflowSources for Sources {
    fn capture(
        &self,
        _: Project,
        task: Task,
        _: Phase,
        _: ContextBudget,
    ) -> WorkflowFuture<'_, SourceSnapshot> {
        Box::pin(async move {
            let worktree = task.worktree.as_ref().unwrap();
            let revision = super::fixture_support::git(worktree, &["rev-parse", "HEAD"]);
            Ok(SourceSnapshot {
                scope: task.scope(),
                source_versions: BTreeMap::from([("fixture-head".into(), revision.clone())]),
                revision,
                payload: "synthetic owned Workflow forwarding context".into(),
            })
        })
    }
}
struct WorktreeEvidence(PathBuf);
impl PhaseGates for WorktreeEvidence {
    fn complete(
        &self,
        invocation: PhaseInvocation,
        status: Option<SessionStatus>,
    ) -> WorkflowFuture<'_, GateOutcome> {
        Box::pin(async move {
            assert_eq!(invocation.phase, Phase::Worktree);
            assert!(status.is_none());
            let worktree = invocation.task.worktree.as_ref().unwrap();
            assert!(worktree.is_dir());
            let head = super::fixture_support::git(worktree, &["rev-parse", "HEAD"]);
            assert_eq!(head, invocation.sources.revision);
            std::fs::write(
                &self.0,
                serde_json::to_vec(&json!({
                    "scope": invocation.task.scope(), "worktree": worktree, "head": head
                }))?,
            )?;
            Ok(GateOutcome::Passed(Evidence {
                scope: invocation.task.scope(),
                phase: Phase::Worktree,
                revision: head,
                source_versions: invocation.sources.source_versions.clone(),
                dependencies: invocation.sources.source_versions,
                artifacts: vec![self.0.to_str().unwrap().into()],
                review_approved: None,
                session_id: None,
                context_version: invocation.context.version,
            }))
        })
    }
}
async fn engine(fixture: &Fixture, adapter: Arc<GrokAdapter>) -> WorkflowEngine {
    let task_id = fixture.request.scope.task_id.unwrap();
    {
        let mut store = fixture.store.lock().unwrap();
        let mut task = store.task(task_id).unwrap().unwrap();
        task.risk = RiskClass::R0;
        store.put_task(&mut task).unwrap();
    }
    let mut registry = AgentRegistry::default();
    registry.register("grok".into(), adapter).unwrap();
    let mut config = Config {
        minimum_workflow: WorkflowClass::Quick,
        ..Config::default()
    };
    config.workflow.default = WorkflowClass::Quick;
    let engine = WorkflowEngine::new(
        fixture.store.clone(),
        Arc::new(registry),
        config,
        Arc::new(Sources),
        Arc::new(WorktreeEvidence(
            fixture.directory.path().join("worktree-proof.json"),
        )),
    )
    .unwrap();
    engine.initialize(task_id, None).await.unwrap();
    assert!(matches!(
        engine.step(task_id, BTreeMap::new()).await.unwrap(),
        StepResult::Completed {
            phase: Phase::Worktree
        }
    ));
    engine
}

#[tokio::test]
async fn reachable_workflow_native_phase_rejects_generic_map_and_binds_own_map() {
    isolated("adapter::grok::environment_workflow_tests::forwarding_child").await;
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn forwarding_child() {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    let denied = Fixture::with_workflow(WorkflowClass::Quick);
    let adapter = Arc::new(denied.adapter());
    let workflow = engine(&denied, adapter).await;
    let task_id = denied.request.scope.task_id.unwrap();
    let reused_generic_map = BTreeMap::from([
        ("HOME".into(), std::env::var("HOME").unwrap()),
        ("PATH".into(), "/usr/bin:/bin".into()),
    ]);
    let result = workflow.step(task_id, reused_generic_map).await.unwrap();
    let reason = match result {
        StepResult::Failed {
            phase: Phase::Implement,
            reason,
        } => reason,
        other => panic!("native control map forwarding unexpectedly succeeded: {other:?}"),
    };
    assert_eq!(
        reason,
        "InvalidConfiguration: native environment value cannot replace intentional runtime authority"
    );
    let snapshot = workflow.snapshot(task_id).unwrap();
    let attempt = snapshot.history.last().unwrap();
    assert_eq!(attempt.phase, Phase::Implement);
    assert_eq!(attempt.state, AttemptState::Failed);
    assert!(attempt.dispatch_started && attempt.session_id.is_none());
    assert_eq!(attempt.detail.as_deref(), Some(reason.as_str()));
    {
        let store = denied.store.lock().unwrap();
        let task = store.task(task_id).unwrap().unwrap();
        assert_eq!(task.state, TaskState::WaitingHuman);
        assert!(task.blockers.contains(&reason));
        assert!(
            store
                .records(&task.scope(), RecordKind::Session)
                .unwrap()
                .is_empty()
        );
        let events = store.events(&task.scope(), 0, 1024).unwrap();
        assert!(!events.iter().any(|e| e.kind == "grok.process_spawned"));
        assert!(events.iter().any(
            |e| e.kind == "workflow.saved" && e.data["evidence"]["attempt"]["detail"] == reason
        ));
        let records = store.records(&task.scope(), RecordKind::Workflow).unwrap();
        assert_eq!(records[0].data["history"][1]["detail"], reason);
    }

    // Separate actual Task at the same reachable phase; no implicit failed retry.
    let mut allowed = Fixture::with_workflow(WorkflowClass::Quick);
    {
        let mut store = allowed.store.lock().unwrap();
        let mut project = allowed.request.project.clone();
        project.environment_refs = vec!["LANG".into()];
        store.put_project(&mut project).unwrap();
        allowed.request.project = project;
    }
    let adapter = Arc::new(allowed.adapter());
    let workflow = engine(&allowed, adapter.clone()).await;
    let task_id = allowed.request.scope.task_id.unwrap();
    let result = workflow
        .step(
            task_id,
            BTreeMap::from([("LANG".into(), "synthetic-own-locale".into())]),
        )
        .await
        .unwrap();
    let id = match result {
        StepResult::Started {
            phase: Phase::Implement,
            session: Some(id),
        } => id,
        other => panic!("own native map did not bind at intended phase: {other:?}"),
    };
    let bound = allowed
        .store
        .lock()
        .unwrap()
        .session(id)
        .unwrap()
        .unwrap()
        .0;
    assert_eq!(bound.agent, "grok");
    assert_eq!(bound.scope, allowed.request.scope);
    assert_eq!(bound.role, SessionRole::Executor);
    let snapshot = workflow.snapshot(task_id).unwrap();
    let attempt = snapshot.history.last().unwrap();
    assert_eq!(attempt.phase, Phase::Implement);
    assert_eq!(attempt.agent.as_deref(), Some("grok"));
    assert_eq!(attempt.session_id, Some(id));
    let mut watch = adapter.subscribe((&bound).into()).unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        while !watch.borrow().terminal() {
            watch.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    let terminal = watch.borrow().clone();
    assert!(
        terminal.session.pid.is_none(),
        "owned native cleanup remains uncertain"
    );
    assert_ne!(terminal.session.state, SessionState::Lost);
    adapter.release((&bound).into()).unwrap();
    // Started+binding alone is the positive; #43 still owns native phase completion.
    child_completed("adapter::grok::environment_workflow_tests::forwarding_child");
}
