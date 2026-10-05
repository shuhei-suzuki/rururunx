//! Real initial/result observations; absent verification/review integrations wait.
//! These receipts do not grant native effects or replace atomic result publication.
use super::{
    results::ResultStore,
    workflow_source::{ManagedWorkflowSources, digest},
    *,
};
use crate::{adapter::SessionStatus, domain::*, state::Store, workflow::*};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;

pub struct ManagedWorkflowGates {
    owner: Arc<RuntimeOwner>,
    sources: Arc<ManagedWorkflowSources>,
}
#[derive(Debug, PartialEq, Eq, Serialize)]
struct Claim {
    record: RecordId,
    version: u64,
    index: usize,
    generation: u64,
    task_version: u64,
    project_version: u64,
    goal_version: u64,
    context: u64,
    observations: usize,
}
impl ManagedWorkflowGates {
    pub fn new(owner: Arc<RuntimeOwner>, sources: Arc<ManagedWorkflowSources>) -> Result<Self> {
        ensure!(
            sources.belongs_to(&owner),
            "gate and source must share the same Runtime"
        );
        Ok(Self { owner, sources })
    }
    fn claim(store: &Store, invocation: &PhaseInvocation) -> Result<(Claim, WorkflowSnapshot)> {
        let task = store.task(invocation.task.id)?.context("Task missing")?;
        let project = store.project(task.project_id)?.context("Project missing")?;
        let goal = store.goal(task.goal_id)?.context("Goal missing")?;
        ensure!(
            true /* mutation: omit exact Task invocation frame */
                && serde_json::to_value(&project)? == serde_json::to_value(&invocation.project)?
                && goal.project_id == task.project_id
                && invocation.sources.scope == task.scope()
                && project.state == ProjectState::Registered
                && matches!(
                    goal.state,
                    GoalState::Created | GoalState::Analyzing | GoalState::Running
                )
                && !crate::state::task_terminal(task.state),
            "foreign, stale or inactive gate owners"
        );
        let mut records = store.records(&task.scope(), RecordKind::Workflow)?;
        ensure!(records.len() == 1, "gate requires sole Workflow");
        let record = records.remove(0);
        let workflow: WorkflowSnapshot = serde_json::from_value(record.data)?;
        let index = workflow.active.context("gate requires active claim")?;
        let attempt = workflow
            .history
            .get(index)
            .context("gate attempt missing")?;
        let context = store
            .context(&task.scope(), Some(attempt.context_version))?
            .context("gate Context missing")?;
        ensure!(
            attempt.phase == invocation.phase
                && attempt.state == AttemptState::Evaluating
                && attempt.generation == workflow.generation
                && attempt.claimed_observations == attempt.observations.len()
                && context.version == task.context_version
                && serde_json::to_value(context)? == serde_json::to_value(&invocation.context)?
                && serde_json::to_value(&invocation.prior_observations)?
                    == serde_json::to_value(&attempt.observations)?
                && serde_json::to_value(&invocation.prerequisites)?
                    == serde_json::to_value(workflow.completed.values().collect::<Vec<_>>())?,
            "gate requires exact Evaluating Context/claim/prerequisites"
        );
        let claim = Claim {
            record: record.id,
            version: record.version,
            index,
            generation: workflow.generation,
            task_version: task.version,
            project_version: project.version,
            goal_version: goal.version,
            context: attempt.context_version,
            observations: attempt.claimed_observations,
        };
        Ok((claim, workflow))
    }
    fn terminal(
        store: &Store,
        invocation: &PhaseInvocation,
        workflow: &WorkflowSnapshot,
        status: &SessionStatus,
    ) -> Result<ExecutionUnit> {
        let attempt = &workflow.history[workflow.active.context("active claim missing")?];
        let native = status
            .execution
            .as_ref()
            .context("managed terminal missing")?;
        let unit = store.validate_execution(&native.authority, false, true)?;
        let (session, _) = store
            .session(status.session.id)?
            .context("persisted terminal missing")?;
        ensure!(
            status.terminal()
                && status.session.state == SessionState::Exited
                && status.failure.is_none()
                && native.work == Some(WorkOutcome::Success)
                && unit.work == Some(WorkOutcome::Success)
                && unit.state == UnitState::WorkKnown
                && unit.kind == UnitKind::Executor
                && unit.phase == invocation.phase.key()
                && !unit.native_effects_open
                && unit.result_finalization_open
                && attempt.execution.as_ref() == Some(&native.handle)
                && attempt.unit.as_ref() == Some(&ManagedUnitRef::from(&unit))
                && attempt.session_id == Some(session.id)
                && unit.session_id == Some(session.id)
                && native.handle.session == session.id
                && native.handle.scope == unit.scope
                && native.handle.unit == unit.id
                && native.handle.generation == unit.generation
                && native.handle.epoch == unit.owner_epoch
                && unit.scope == invocation.task.scope()
                && session.role == SessionRole::Executor
                && session.provider == unit.provider
                && session.worktree == unit.worktree
                && attempt.agent.as_ref() == Some(&session.agent)
                && serde_json::to_value(&session)? == serde_json::to_value(&status.session)?
                && serde_json::to_value(&session)? == serde_json::to_value(&native.session)?,
            "gate requires exact owned successful terminal"
        );
        Ok(unit)
    }
    async fn evaluate(
        &self,
        invocation: PhaseInvocation,
        transport: Option<SessionStatus>,
    ) -> Result<GateOutcome> {
        let (claim, workflow) = {
            let store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            Self::claim(&store, &invocation)?
        };
        let mut checked_unit = None;
        let mut checked_artifact = None;
        let phase = invocation.phase;
        match phase {
            Phase::Issue | Phase::Worktree => {
                ensure!(
                    transport.is_none(),
                    "initial gate cannot receive a terminal"
                );
                if invocation.task.title.trim().is_empty()
                    || invocation.task.acceptance_criteria.is_empty()
                    || invocation
                        .task
                        .acceptance_criteria
                        .iter()
                        .any(|a| a.trim().is_empty())
                {
                    return Ok(GateOutcome::Waiting(
                        "registered Task specification requires title and acceptance criteria"
                            .into(),
                    ));
                }
                let goal = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .goal(invocation.task.goal_id)?
                    .context("Goal missing")?;
                if goal.objective.trim().is_empty() {
                    return Ok(GateOutcome::Waiting(
                        "registered Goal objective is empty".into(),
                    ));
                }
                checked_unit = Some(
                    self.sources
                        .verify_initial(
                            &invocation.project,
                            &invocation.task,
                            phase,
                            &invocation.budget,
                            &invocation.sources,
                        )
                        .await?,
                );
            }
            Phase::Implement => {
                let status = transport
                    .as_ref()
                    .context("Implement requires actual managed terminal")?;
                let (unit, artifact) = {
                    let store = self
                        .owner
                        .store
                        .lock()
                        .map_err(|_| anyhow::anyhow!("state poisoned"))?;
                    let unit = Self::terminal(&store, &invocation, &workflow, status)?;
                    let artifact = store.result_artifact(
                        invocation
                            .sources
                            .artifact
                            .context("retained result missing")?,
                    )?;
                    ensure!(
                        artifact.state == ArtifactState::Ready && artifact.unit_id == unit.id,
                        "Implement requires the successful unit's Ready artifact"
                    );
                    (unit, artifact)
                };
                checked_unit = Some(unit);
                checked_artifact = Some(artifact);
            }
            Phase::RequirementsCommit | Phase::DesignCommit | Phase::Commit => {
                ensure!(transport.is_none(), "commit gate has no native Session");
                let predecessor = match phase {
                    Phase::RequirementsCommit => Phase::Requirements,
                    Phase::DesignCommit => Phase::Design,
                    _ => Phase::Implement,
                };
                let evidence = workflow
                    .completed
                    .get(&predecessor)
                    .context("corresponding Executor evidence missing")?;
                let markers = evidence
                    .artifacts
                    .iter()
                    .filter_map(|s| s.strip_prefix("rrx-artifact:"))
                    .collect::<Vec<_>>();
                ensure!(
                    markers.len() == 1,
                    "commit prerequisite requires one retained artifact"
                );
                let artifact = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .result_artifact(markers[0].parse()?)?;
                ensure!(
                    artifact.state == ArtifactState::Published
                        && evidence.scope == invocation.task.scope()
                        && evidence.revision == invocation.sources.revision
                        && evidence.source_versions == invocation.sources.source_versions,
                    "commit prerequisite is foreign/stale/unpublished"
                );
                checked_artifact = Some(artifact);
            }
            _ => {
                return Ok(GateOutcome::Waiting(format!(
                    "{} requires its qualified production evidence integration",
                    phase.key()
                )));
            }
        }
        if let Some(artifact) = &checked_artifact {
            ensure!(
                invocation.sources.artifact == Some(artifact.id)
                    && artifact.scope == invocation.task.scope()
                    && artifact.revision == invocation.sources.revision
                    && artifact.dependencies == invocation.sources.source_versions,
                "gate retained artifact differs from observed input"
            );
            ResultStore::new(self.owner.clone())
                .verify(artifact)
                .await?;
        }
        let current = self
            .sources
            .capture(
                invocation.project.clone(),
                invocation.task.clone(),
                phase,
                invocation.budget.clone(),
            )
            .await?;
        ensure!(
            current == invocation.sources,
            "gate source changed during checks"
        );
        let mut store = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        ensure!(
            Self::claim(&store, &invocation)?.0 == claim,
            "gate claim changed during checks"
        );
        if let Some(unit) = &checked_unit {
            ensure!(
                serde_json::to_value(store.validate_execution(&unit.authority(), false, true)?)?
                    == serde_json::to_value(unit)?,
                "gate unit changed during checks"
            );
        }
        if let Some(artifact) = &checked_artifact {
            ensure!(
                serde_json::to_value(store.result_artifact(artifact.id)?)?
                    == serde_json::to_value(artifact)?,
                "gate artifact changed during checks"
            );
        }
        let mut receipt = Record::new(
            invocation.task.scope(),
            RecordKind::Verification,
            json!({"schema":"managed_workflow_gate_v1","claim":claim,"phase":phase,
                "revision":invocation.sources.revision,"sources":invocation.sources.source_versions,
                "launch_revision":invocation.context.revision,
                "context_data_sha256":digest(&serde_json::to_vec(&invocation.context.data)?),
                "unit":checked_unit.as_ref().map(ManagedUnitRef::from),
                "profile_digest":checked_unit.as_ref().map(|u| &u.profile_digest),
                "artifact":checked_artifact.as_ref().map(|a| a.id),
                "manifest_sha256":checked_artifact.as_ref().map(|a| &a.manifest_sha256)}),
        );
        ensure!(
            serde_json::to_vec(&receipt.data)?.len() <= 64 * 1024,
            "gate receipt exceeds bound"
        );
        store.put_record(&mut receipt)?;
        let mut artifacts = vec![format!("rrx-gate:{}", receipt.id)];
        if let Some(artifact) = checked_artifact {
            artifacts.push(format!("rrx-artifact:{}", artifact.id));
        }
        Ok(GateOutcome::Passed(Evidence {
            scope: invocation.task.scope(),
            phase,
            revision: invocation.sources.revision,
            source_versions: invocation.sources.source_versions.clone(),
            dependencies: invocation.sources.source_versions,
            artifacts,
            review_approved: None,
            session_id: transport.as_ref().map(|s| s.session.id),
            context_version: invocation.context.version,
        }))
    }
}
impl PhaseGates for ManagedWorkflowGates {
    fn complete(
        &self,
        invocation: PhaseInvocation,
        transport: Option<SessionStatus>,
    ) -> WorkflowFuture<'_, GateOutcome> {
        Box::pin(self.evaluate(invocation, transport))
    }
}
