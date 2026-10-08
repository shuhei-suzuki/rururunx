//! The legacy Engine with Grok at its first reachable Native phase: FM §8.3
//! F2 (R). Grok is not an installed provider (S2), so there is no D2 target.
use super::{environment_tests::isolated, fixture_support::Fixture, *};
use crate::{
    config::{Config, WorkflowClass},
    domain::RiskClass,
    workflow::*,
};

/// FM §8.6.3: one shared test-only count of the Grok adapter, Sources
/// (whose capture runs the fixture's direct Git) and gate callbacks.
type Calls = Arc<std::sync::atomic::AtomicUsize>;
fn called(calls: &Calls) {
    calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}
struct Sources(Calls);
impl WorkflowSources for Sources {
    fn capture(
        &self,
        _: Project,
        task: Task,
        _: Phase,
        _: ContextBudget,
    ) -> WorkflowFuture<'_, SourceSnapshot> {
        called(&self.0);
        Box::pin(async move {
            let worktree = task.worktree.as_ref().unwrap();
            let revision = super::fixture_support::git(worktree, &["rev-parse", "HEAD"]);
            Ok(SourceSnapshot {
                artifact: None,
                scope: task.scope(),
                source_versions: BTreeMap::from([("fixture-head".into(), revision.clone())]),
                revision,
                payload: "synthetic owned Workflow forwarding context".into(),
            })
        })
    }
}
struct WorktreeEvidence(PathBuf, Calls);
impl PhaseGates for WorktreeEvidence {
    fn complete(
        &self,
        invocation: PhaseInvocation,
        status: Option<SessionStatus>,
    ) -> WorkflowFuture<'_, GateOutcome> {
        called(&self.1);
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
async fn engine(fixture: &Fixture, adapter: Arc<GrokAdapter>) -> (WorkflowEngine, Calls) {
    let task_id = fixture.request.scope.task_id.unwrap();
    let mut registry = AgentRegistry::default();
    registry.register("grok".into(), adapter).unwrap();
    let calls = registry.count_callbacks();
    let mut config = Config {
        minimum_workflow: WorkflowClass::Quick,
        ..Config::default()
    };
    config.workflow.default = WorkflowClass::Quick;
    let engine = WorkflowEngine::new(
        fixture.store.clone(),
        Arc::new(registry),
        config,
        Arc::new(Sources(calls.clone())),
        Arc::new(WorktreeEvidence(
            fixture.directory.path().join("worktree-proof.json"),
            calls.clone(),
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
    (engine, calls)
}

/// FM §8.3 F2 (R). Former subject: at the first reachable Native phase the
/// legacy Engine refused a reused generic environment map and bound Grok's
/// own `environment_refs` map to a started Session. The legacy Engine has no
/// production caller (S1) and Grok is not an installed provider (S2), so the
/// step into Implement is now refused with the typed
/// `ManagedBindingUnavailable` before source capture, whichever map is
/// passed. The Task, Workflow record, audit, Units, Sessions, completed
/// Git outputs and the Grok adapter, Sources (its capture runs the fixture's
/// direct Git) and gate callback count are unchanged; no Grok process is
/// spawned (its `grok.process_spawned` audit and Session record are in the
/// preimage).
/// Its managed equivalent is SC-N continuation work (D3/D4); the rest of the
/// former scenario is retired.
#[tokio::test]
async fn reachable_workflow_native_phase_rejects_generic_map_and_binds_own_map() {
    isolated("adapter::grok::environment_workflow_tests::forwarding_child").await;
}
#[tokio::test]
#[ignore = "only entered by owned env-cleared canary parent"]
async fn forwarding_child() {
    assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
    let mut fixture = Fixture::with_workflow(WorkflowClass::Quick);
    {
        let mut store = fixture.store.lock().unwrap();
        let task = store
            .task(fixture.request.scope.task_id.unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(task.risk, RiskClass::R0, "SETUP: accepted Quick plan");
        let mut project = fixture.request.project.clone();
        project.environment_refs = vec!["LANG".into()];
        store.put_project(&mut project).unwrap();
        fixture.request.project = project;
    }
    let (workflow, calls) = engine(&fixture, Arc::new(fixture.adapter())).await;
    let task_id = fixture.request.scope.task_id.unwrap();
    let preimage = || {
        let store = fixture.store.lock().unwrap();
        let task = store.task(task_id).unwrap().unwrap();
        let scope = task.scope();
        json!({
            "task": task,
            "workflow": store.records(&scope, RecordKind::Workflow).unwrap(),
            "events": store.events(&scope, 0, 1024).unwrap(),
            "units": store.execution_units(Some(&scope)).unwrap().len(),
            "sessions": store.records(&scope, RecordKind::Session).unwrap(),
            "git": crate::git::observed_git_outputs(),
            "callbacks": calls.load(std::sync::atomic::Ordering::SeqCst),
        })
    };
    for map in [
        // The reused generic map the former test refused.
        BTreeMap::from([
            ("HOME".into(), std::env::var("HOME").unwrap()),
            ("PATH".into(), "/usr/bin:/bin".into()),
        ]),
        // The own `environment_refs` map the former test bound.
        BTreeMap::from([("LANG".into(), "synthetic-own-locale".into())]),
    ] {
        let before = preimage();
        let error = workflow
            .step(task_id, map)
            .await
            .expect_err("FM F2: the legacy Native step must be refused");
        assert!(
            matches!(
                error.downcast_ref::<NativePreflightRefusal>(),
                Some(NativePreflightRefusal::ManagedBindingUnavailable)
            ),
            "FM F2: typed refusal: {error:#}"
        );
        assert_eq!(preimage(), before, "FM F2: preimage unchanged");
    }
    let store = fixture.store.lock().unwrap();
    let scope = store.task(task_id).unwrap().unwrap().scope();
    assert!(
        !store
            .events(&scope, 0, 1024)
            .unwrap()
            .iter()
            .any(|e| e.kind == "grok.process_spawned")
    );
    assert!(
        store
            .records(&scope, RecordKind::Session)
            .unwrap()
            .is_empty()
    );
    drop(store);
    child_completed("adapter::grok::environment_workflow_tests::forwarding_child");
}
