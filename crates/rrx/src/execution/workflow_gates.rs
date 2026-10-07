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
/// Actual initial-gate result. Only the concrete producer can construct this;
/// a receipt ID or serialized GateOutcome does not create completion authority.
pub(crate) struct InitialGateCompletion {
    invocation: PhaseInvocation,
    outcome: GateOutcome,
    receipt: Option<Record>,
}
impl InitialGateCompletion {
    pub(crate) fn invocation(&self) -> &PhaseInvocation {
        &self.invocation
    }
    pub(crate) fn outcome(&self) -> &GateOutcome {
        &self.outcome
    }
    pub(crate) fn receipt(&self) -> Option<&Record> {
        self.receipt.as_ref()
    }
}
/// Sealed settled-gate disposition. `GateOutcome` has no Unknown; the
/// producer seals one for any evaluation failure after its claim check.
pub(crate) enum SettledGateOutcome {
    Known(GateOutcome),
    Unknown,
}
/// Built only inside `evaluate_settled`; no other constructor.
pub(crate) struct SettledGateCompletion {
    outcome: SettledGateOutcome,
    receipt: Option<Record>,
    sources: SourceSnapshot,
    claim: Arc<crate::state::managed_binding::GateClaimAcknowledgment>,
}
impl SettledGateCompletion {
    pub(crate) fn belongs_to(
        &self,
        claim: &Arc<crate::state::managed_binding::GateClaimAcknowledgment>,
    ) -> bool {
        Arc::ptr_eq(&self.claim, claim)
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        SettledGateOutcome,
        Option<Record>,
        SourceSnapshot,
        Arc<crate::state::managed_binding::GateClaimAcknowledgment>,
    ) {
        (self.outcome, self.receipt, self.sources, self.claim)
    }
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
    pub(crate) async fn complete_initial_driven(
        &self,
        invocation: PhaseInvocation,
    ) -> Result<InitialGateCompletion> {
        ensure!(
            matches!(invocation.phase, Phase::Issue | Phase::Worktree),
            "driven initial gate only"
        );
        let outcome = self.evaluate(invocation.clone(), None).await?;
        let receipt = if let GateOutcome::Passed(evidence) = &outcome {
            let ids = evidence
                .artifacts
                .iter()
                .filter_map(|a| a.strip_prefix("rrx-gate:"))
                .collect::<Vec<_>>();
            ensure!(ids.len() == 1, "initial gate receipt missing");
            let store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let receipt = store
                .record(ids[0].parse()?)?
                .context("initial receipt missing")?;
            ensure!(
                receipt.kind == RecordKind::Verification
                    && receipt.scope == invocation.task.scope()
                    && receipt.data["schema"] == "managed_workflow_gate_v1"
                    && receipt.data["claim"]
                        == serde_json::to_value(Self::claim(&store, &invocation)?.0)?
                    && receipt.data["phase"] == serde_json::to_value(invocation.phase)?
                    && receipt.data["revision"] == invocation.sources.revision
                    && receipt.data["sources"]
                        == serde_json::to_value(&invocation.sources.source_versions)?,
                "initial receipt identity changed"
            );
            Some(receipt)
        } else {
            None
        };
        Ok(InitialGateCompletion {
            invocation,
            outcome,
            receipt,
        })
    }
    fn claim(store: &Store, invocation: &PhaseInvocation) -> Result<(Claim, WorkflowSnapshot)> {
        let task = store.task(invocation.task.id)?.context("Task missing")?;
        let project = store.project(task.project_id)?.context("Project missing")?;
        let goal = store.goal(task.goal_id)?.context("Goal missing")?;
        ensure!(
            serde_json::to_value(&task)? == serde_json::to_value(&invocation.task)?
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
impl ManagedWorkflowGates {
    /// `evaluate` for a settled, bound, marker-bound phase. The claim check
    /// failing returns Err and no completion; every later failure returns a
    /// sealed `Unknown` completion. No generic `validate_execution`,
    /// `validate_authority` or Driver `validate` is reached.
    pub(crate) async fn evaluate_settled(
        &self,
        claim: &Arc<crate::state::managed_binding::GateClaimAcknowledgment>,
        invocation: PhaseInvocation,
    ) -> Result<SettledGateCompletion> {
        let (checked, workflow) = {
            let store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            Self::claim(&store, &invocation)?
        };
        let sources = invocation.sources.clone();
        let (outcome, receipt) = match self
            .evaluate_settled_checked(claim, &invocation, checked, &workflow)
            .await
        {
            Ok((outcome, receipt)) => (SettledGateOutcome::Known(outcome), receipt),
            // The raw error never reaches durable rows or payloads.
            Err(_cause) => (SettledGateOutcome::Unknown, None),
        };
        Ok(SettledGateCompletion {
            outcome,
            receipt,
            sources,
            claim: claim.clone(),
        })
    }
    async fn evaluate_settled_checked(
        &self,
        claim: &Arc<crate::state::managed_binding::GateClaimAcknowledgment>,
        invocation: &PhaseInvocation,
        checked: Claim,
        workflow: &WorkflowSnapshot,
    ) -> Result<(GateOutcome, Option<Record>)> {
        let settled = claim.settled();
        let phase = invocation.phase;
        ensure!(phase == settled.phase(), "settled gate phase differs");
        if phase != Phase::Implement {
            return Ok((
                GateOutcome::Waiting(format!(
                    "{} requires its qualified production evidence integration",
                    phase.key()
                )),
                None,
            ));
        }
        // Planned query-only at the post-claim endpoint, reused below.
        let currency = Arc::new(crate::state::managed_binding::plan_settled_currency(
            &self.owner,
            settled,
        )?);
        let (unit, artifact) = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let unit = Self::settled_terminal(&mut store, invocation, workflow, &currency)?;
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
        ensure!(
            invocation.sources.artifact == Some(artifact.id)
                && artifact.scope == invocation.task.scope()
                && artifact.revision == invocation.sources.revision
                && artifact.dependencies == invocation.sources.source_versions,
            "gate retained artifact differs from observed input"
        );
        ResultStore::new(self.owner.clone())
            .verify(&artifact)
            .await?;
        let current = self
            .sources
            .capture_settled(
                &currency,
                &invocation.project,
                &invocation.task,
                phase,
                &invocation.budget,
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
            Self::claim(&store, invocation)?.0 == checked,
            "gate claim changed during checks"
        );
        // G7: the stored Unit row equals the sealed terminal raw exactly.
        store.validate_settled_helper(&currency, &unit)?;
        ensure!(
            serde_json::to_value(store.result_artifact(artifact.id)?)?
                == serde_json::to_value(&artifact)?,
            "gate artifact changed during checks"
        );
        let mut receipt = Record::new(
            invocation.task.scope(),
            RecordKind::Verification,
            json!({"schema":"managed_workflow_gate_v1","claim":checked,"phase":phase,
                "revision":invocation.sources.revision,"sources":invocation.sources.source_versions,
                "launch_revision":invocation.context.revision,
                "context_data_sha256":digest(&serde_json::to_vec(&invocation.context.data)?),
                "unit":ManagedUnitRef::from(&unit),
                "profile_digest":&unit.profile_digest,
                "artifact":artifact.id,
                "manifest_sha256":&artifact.manifest_sha256}),
        );
        ensure!(
            serde_json::to_vec(&receipt.data)?.len() <= 64 * 1024,
            "gate receipt exceeds bound"
        );
        store.put_record(&mut receipt)?;
        let evidence = Evidence {
            scope: invocation.task.scope(),
            phase,
            revision: invocation.sources.revision.clone(),
            source_versions: invocation.sources.source_versions.clone(),
            dependencies: invocation.sources.source_versions.clone(),
            artifacts: vec![
                format!("rrx-gate:{}", receipt.id),
                format!("rrx-artifact:{}", artifact.id),
            ],
            review_approved: None,
            session_id: Some(settled.settlement().session().id),
            context_version: invocation.context.version,
        };
        Ok((GateOutcome::Passed(evidence), Some(receipt)))
    }
    /// `terminal`'s predicates sourced from the SAME settlement: decoded
    /// values with the `cleanup` overlay excluded, and the stored Unit row
    /// compared exactly through the protected reader.
    fn settled_terminal(
        store: &mut Store,
        invocation: &PhaseInvocation,
        workflow: &WorkflowSnapshot,
        currency: &crate::state::managed_binding::SettledCurrency,
    ) -> Result<ExecutionUnit> {
        let attempt = &workflow.history[workflow.active.context("active claim missing")?];
        let settlement = currency.settled().settlement();
        let unit = settlement.unit();
        let session = settlement.session();
        let handle = crate::execution::native::ManagedSessionRef {
            scope: unit.scope.clone(),
            unit: unit.id,
            generation: unit.generation,
            epoch: unit.owner_epoch,
            session: session.id,
        };
        ensure!(
            session.state == SessionState::Exited
                && settlement.receipt().observed_work == WorkOutcome::Success
                && unit.work == Some(WorkOutcome::Success)
                && unit.state == UnitState::WorkKnown
                && unit.kind == UnitKind::Executor
                && unit.phase == invocation.phase.key()
                && !unit.native_effects_open
                && unit.result_finalization_open
                && attempt.execution.as_ref() == Some(&handle)
                && attempt.unit.as_ref() == Some(&ManagedUnitRef::from(unit))
                && attempt.session_id == Some(session.id)
                && unit.session_id == Some(session.id)
                && unit.scope == invocation.task.scope()
                && session.role == SessionRole::Executor
                && session.provider == unit.provider
                && session.worktree == unit.worktree
                && attempt.agent.as_ref() == Some(&session.agent),
            "gate requires exact owned successful settlement"
        );
        let mut current = store.execution_unit(unit.id)?;
        current.cleanup = unit.cleanup;
        ensure!(
            serde_json::to_value(&current)? == serde_json::to_value(unit)?,
            "gate Unit differs from the settled terminal postimage"
        );
        store.validate_settled_helper(currency, unit)?;
        Ok(unit.clone())
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
