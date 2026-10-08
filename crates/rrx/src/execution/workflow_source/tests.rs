//! Negative actual Workflow/Store consumers; evidence and native programs are
//! account-free fixtures, not production gate policy or provider acceptance.
use super::*;
use crate::{
    adapter::{AgentRegistry, InputKind, SessionStatus},
    workflow::{Evidence, GateOutcome, PhaseGates, PhaseInvocation, StepResult, WorkflowEngine},
};
struct Gates;
impl PhaseGates for Gates {
    fn complete(
        &self,
        invocation: PhaseInvocation,
        _: Option<SessionStatus>,
    ) -> WorkflowFuture<'_, GateOutcome> {
        Box::pin(async move {
            Ok(GateOutcome::Passed(Evidence {
                scope: invocation.task.scope(),
                phase: invocation.phase,
                revision: invocation.sources.revision,
                source_versions: invocation.sources.source_versions.clone(),
                artifacts: vec!["fixture-only-worktree-evidence".into()],
                dependencies: invocation.sources.source_versions,
                review_approved: None,
                session_id: None,
                context_version: invocation.context.version,
            }))
        })
    }
}
#[derive(Clone, Copy)]
enum Fault {
    Envelope,
    TaskCas,
    Helper,
}
struct FaultSources {
    inner: Arc<ManagedWorkflowSources>,
    fault: Fault,
    /// Calls that reached the first-adoption consumer.
    reached: std::sync::atomic::AtomicUsize,
}
impl WorkflowSources for FaultSources {
    fn capture(
        &self,
        project: Project,
        task: Task,
        phase: Phase,
        budget: ContextBudget,
    ) -> WorkflowFuture<'_, SourceSnapshot> {
        self.inner.capture(project, task, phase, budget)
    }
    fn committed_input(
        &self,
        project: Project,
        task: Task,
        phase: Phase,
        class: WorkflowClass,
    ) -> WorkflowFuture<'_, Option<CommittedWorkflowInput>> {
        self.inner.committed_input(project, task, phase, class)
    }
    fn take_initial_executor(
        &self,
        project: &Project,
        task: &Task,
        phase: Phase,
        budget: &ContextBudget,
    ) -> WorkflowFuture<'_, Option<InitialWorkflowExecutor>> {
        let project = project.clone();
        let task = task.clone();
        let budget = budget.clone();
        self.reached
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(async move {
            let token = self
                .inner
                .take_initial_executor(&project, &task, phase, &budget)
                .await?
                .context("initial proof missing")?;
            match self.fault {
                Fault::Envelope => {
                    let (context, reservation) = {
                        let store = self.inner.owner.store.lock().unwrap();
                        let record = store.records(&task.scope(), RecordKind::Workflow)?[0].clone();
                        let workflow: WorkflowSnapshot = serde_json::from_value(record.data)?;
                        let index = workflow.active.context("active phase required")?;
                        let context = store
                            .context(&task.scope(), Some(task.context_version))?
                            .unwrap();
                        let goal = store.goal(task.goal_id)?.unwrap();
                        let reservation = WorkflowReservation {
                            record: record.id,
                            version: record.version,
                            index,
                            workflow_generation: workflow.generation,
                            context: context.version,
                            project_version: project.version,
                            goal_version: goal.version,
                        };
                        (context, reservation)
                    };
                    let mut envelope = context.data.clone();
                    envelope["generation"] = serde_json::json!(999_999);
                    let input = PreparedInput {
                        scope: task.scope(),
                        kind: InputKind::ContextPack,
                        revision: context.revision,
                        version: context.version,
                        source_versions: context.source_hashes,
                        payload: serde_json::to_string(&envelope)?,
                    };
                    let result = token
                        .adopt(task.version, &reservation, phase.key(), "codex", &input)
                        .await;
                    ensure!(
                        result.is_err(),
                        "altered complete Context envelope was adopted"
                    );
                    let error = result.err().unwrap();
                    ensure!(
                        error
                            .to_string()
                            .contains("prepared source/rule frame differs from Context"),
                        "unexpected envelope refusal: {error}"
                    );
                    Err(error)
                }
                Fault::TaskCas => {
                    let mut changed = task.clone();
                    changed.next_action = Some("fixture changed after producer read".into());
                    self.inner
                        .owner
                        .store
                        .lock()
                        .unwrap()
                        .put_task(&mut changed)?;
                    Ok(Some(token))
                }
                Fault::Helper => {
                    let unit = token.prepared.unit();
                    self.inner
                        .owner
                        .store
                        .lock()
                        .unwrap()
                        .reserve_execution_helper(
                            &unit.authority(),
                            OperationId::new(),
                            true,
                            &unit.worktree,
                            "git_helper",
                        )?;
                    Ok(Some(token))
                }
            }
        })
    }
}
/// FM §8.3 F2 (R). Former subject: first adoption of the prepared bootstrap
/// Unit (`adopt_prepared_workflow_execution`) refused a changed Context
/// envelope, a stale Task CAS and an unsettled helper atomically before
/// Native. That consumer is reachable only through legacy `prepare_agent`
/// behind F2 (`workflow.rs:1815`), so it is a retired path ("none: retired
/// path"); the D2 lane uses `first_executor_frame` instead. Each fault's
/// legacy-Engine step into the Native phase is now refused with the typed
/// `ManagedBindingUnavailable` before source capture: the fault hook is never
/// reached, and the Task, Workflow record, audit, the prepared Unit, its
/// effects, Sessions, result artifacts and completed Git outputs are
/// unchanged.
#[tokio::test]
async fn first_adoption_refuses_changed_envelope_task_cas_and_unsettled_helper_atomically() {
    for fault in [Fault::Envelope, Fault::TaskCas, Fault::Helper] {
        let (dir, owner) = crate::runtime::legacy_fixture(
            results::tests::seed,
            vec![crate::runtime::LegacyTask {
                workflow: WorkflowClass::Quick,
                risk: crate::domain::RiskClass::R0,
                ..crate::runtime::LegacyTask::standard("adoption", "codex")
            }],
        )
        .await;
        let task = dir.task();
        assert_eq!(task.workflow, WorkflowClass::Quick, "SETUP: accepted class");
        let program = native::tests::program(dir.path(), "codex");
        let mut config = Config {
            minimum_workflow: WorkflowClass::Quick,
            ..Default::default()
        };
        config.workflow.risk_mapping = [WorkflowClass::Quick; 4];
        config.agents.insert(
            "codex".into(),
            crate::config::AgentConfig {
                provider: Some("codex".into()),
                command: vec![program.to_string_lossy().into()],
                ..Default::default()
            },
        );
        let sources = Arc::new(ManagedWorkflowSources::new(owner.clone(), config.clone()).unwrap());
        let prepared = sources.prepare(task.id, "codex").await.unwrap();
        let registry =
            Arc::new(AgentRegistry::from_managed_config(&config, owner.clone()).unwrap());
        let faults = Arc::new(FaultSources {
            inner: sources,
            fault,
            reached: Default::default(),
        });
        let engine = WorkflowEngine::new(
            owner.store(),
            registry,
            config,
            faults.clone(),
            Arc::new(Gates),
        )
        .unwrap();
        engine.initialize(task.id, None).await.unwrap();
        assert!(matches!(
            engine.step(task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Completed { .. }
        ));
        let preimage = || {
            let store = owner.store.lock().unwrap();
            let scope = task.scope();
            serde_json::json!({
                "task": store.task(task.id).unwrap(),
                "workflow": store.records(&scope, RecordKind::Workflow).unwrap(),
                "events": store.events(&scope, 0, 1000).unwrap().len(),
                "units": store.execution_units(Some(&scope)).unwrap(),
                "effects": format!("{:?}", store.managed_effects(prepared.id).unwrap()),
                "sessions": store.records(&scope, RecordKind::Session).unwrap().len(),
                "artifacts": store.result_artifacts(&scope).unwrap().len(),
                "git": crate::git::observed_git_outputs(),
            })
        };
        let before = preimage();
        let error = engine
            .step(task.id, BTreeMap::new())
            .await
            .expect_err("FM F2: the legacy Native step must be refused");
        assert!(
            matches!(
                error.downcast_ref::<crate::workflow::NativePreflightRefusal>(),
                Some(crate::workflow::NativePreflightRefusal::ManagedBindingUnavailable)
            ),
            "FM F2: typed refusal: {error:#}"
        );
        assert_eq!(preimage(), before, "FM F2: preimage unchanged");
        assert_eq!(
            faults.reached.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "FM F2: the first-adoption consumer is not reached"
        );
        let store = owner.store.lock().unwrap();
        let unit = store.execution_unit(prepared.id).unwrap();
        assert_eq!(unit.phase, WORKFLOW_SOURCE_BOOTSTRAP);
        assert!(unit.session_id.is_none());
        assert_eq!(store.execution_units(Some(&task.scope())).unwrap().len(), 1);
    }
}
#[tokio::test]
async fn retained_corpus_reads_exact_tree_blobs_after_retirement_and_refuses_unrelated_objects() {
    let (_dir, owner, task) = results::tests::fixture().await;
    let attempts = attempts::AttemptManager::new(owner.clone());
    let (unit, _) = attempts
        .prepare(task.id, "codex", "implement", None)
        .await
        .unwrap();
    let old_blob = results::text(
        &results::git(&unit.worktree, ["rev-parse", "HEAD:answer.txt"])
            .await
            .unwrap(),
    )
    .unwrap();
    std::fs::write(
        unit.worktree.join("answer.txt"),
        "retained changed source\n",
    )
    .unwrap();
    results::git(&unit.worktree, ["add", "answer.txt"])
        .await
        .unwrap();
    results::git(
        &unit.worktree,
        [
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            "retained source",
        ],
    )
    .await
    .unwrap();
    let sha = results::text(
        &results::git(&unit.worktree, ["rev-parse", "HEAD"])
            .await
            .unwrap(),
    )
    .unwrap();
    let done = owner
        .store
        .lock()
        .unwrap()
        .finish_execution(
            &unit.authority(),
            WorkOutcome::Success,
            Disposition::Completed,
        )
        .unwrap();
    let result = results::ResultStore::new(owner.clone());
    let artifact = result
        .capture(&done.authority(), &sha, BTreeMap::new())
        .await
        .unwrap();
    let current = owner.store.lock().unwrap().execution_unit(unit.id).unwrap();
    attempts.retire(&current.authority(), false).unwrap();
    std::fs::remove_dir_all(owner.root.join("units").join(unit.id.to_string())).unwrap();
    std::fs::write(
        unit.worktree.join("answer.txt"),
        "survivor bytes must not enter source\n",
    )
    .unwrap();
    let io = RetainedGit::new(owner.clone(), &artifact).unwrap();
    assert!(io.run(["cat-file", "blob", &old_blob]).await.is_err());
    let files = read_corpus(CorpusReader::Retained(&io), &artifact.repository, &sha)
        .await
        .unwrap();
    assert_eq!(files.files.len(), 1);
    assert_eq!(
        files.files[0].bytes.as_deref(),
        Some(b"retained changed source\n".as_slice())
    );
    assert!(io.run(["cat-file", "blob", &old_blob]).await.is_err());
    let (effects, after) = {
        let store = owner.store.lock().unwrap();
        (
            store.managed_effects(unit.id).unwrap(),
            store.execution_unit(unit.id).unwrap(),
        )
    };
    let retained = effects
        .iter()
        .filter(|e| e.kind == "retained_git")
        .collect::<Vec<_>>();
    assert_eq!(retained.len(), 2);
    assert!(retained.iter().all(|e| {
        e.state == EffectState::Confirmed
            && e.expected_target
                .starts_with(&format!("artifact:{}:", artifact.id))
    }));
    assert!(!after.native_effects_open && !after.result_finalization_open);
    assert_eq!(after.work, Some(WorkOutcome::Success));
}
