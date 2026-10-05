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
#[tokio::test]
async fn first_adoption_refuses_changed_envelope_task_cas_and_unsettled_helper_atomically() {
    for fault in [Fault::Envelope, Fault::TaskCas, Fault::Helper] {
        let (dir, owner, fixture_task) = results::tests::fixture().await;
        let mut task = Task::new(
            fixture_task.project_id,
            fixture_task.goal_id,
            "negative adoption".into(),
            "codex".into(),
        );
        task.workflow = WorkflowClass::Quick;
        owner.store.lock().unwrap().put_task(&mut task).unwrap();
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
        let engine = WorkflowEngine::new(
            owner.store(),
            registry,
            config,
            Arc::new(FaultSources {
                inner: sources,
                fault,
            }),
            Arc::new(Gates),
        )
        .unwrap();
        engine.initialize(task.id, None).await.unwrap();
        assert!(matches!(
            engine.step(task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Completed { .. }
        ));
        let result = engine.step(task.id, BTreeMap::new()).await;
        assert!(result.is_err(), "fault must refuse before native");
        let expected = match fault {
            Fault::Envelope => "prepared source/rule frame differs",
            Fault::TaskCas => "prepared Task CAS changed",
            Fault::Helper => "unresolved or native effects",
        };
        assert!(result.err().unwrap().to_string().contains(expected));
        let store = owner.store.lock().unwrap();
        let unit = store.execution_unit(prepared.id).unwrap();
        assert_eq!(unit.phase, WORKFLOW_SOURCE_BOOTSTRAP);
        assert!(!unit.native_effects_open && !unit.result_finalization_open);
        assert_eq!(unit.work, Some(WorkOutcome::Unknown));
        assert!(unit.session_id.is_none());
        assert_eq!(store.execution_units(Some(&task.scope())).unwrap().len(), 1);
        assert!(
            store
                .records(&task.scope(), RecordKind::Session)
                .unwrap()
                .is_empty()
        );
        assert!(store.result_artifacts(&task.scope()).unwrap().is_empty());
        assert!(
            store
                .managed_effects(unit.id)
                .unwrap()
                .iter()
                .all(|e| e.kind == "git_helper")
        );
        let workflow: WorkflowSnapshot = serde_json::from_value(
            store.records(&task.scope(), RecordKind::Workflow).unwrap()[0]
                .data
                .clone(),
        )
        .unwrap();
        assert!(workflow.history[workflow.active.unwrap()].unit.is_none());
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
    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0].bytes.as_deref(),
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
