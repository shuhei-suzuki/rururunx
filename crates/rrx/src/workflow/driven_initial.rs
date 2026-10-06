//! Actual initial Evidence phases; missing later typed lanes never fall back.
use super::*;
use crate::{
    execution::workflow_gates::ManagedWorkflowGates,
    execution::workflow_source::ManagedWorkflowSources,
    runtime::driver::WorkerLifetime,
    state::{DriverReadTicket, InitialGateEdge},
};
use anyhow::bail;
impl WorkflowEngine {
    pub(crate) async fn step_driven_initial(
        &self,
        task_id: TaskId,
        sources: &Arc<ManagedWorkflowSources>,
        lifetime: &WorkerLifetime,
    ) -> Result<StepResult> {
        let installed: Arc<dyn WorkflowSources> = sources.clone();
        ensure!(
            Arc::ptr_eq(&self.sources, &installed),
            "Driver Engine has another Sources producer"
        );
        let owner = self
            .registry
            .managed_owner()
            .context("Driver Runtime owner missing")?;
        let snapshot = self.read(task_id)?;
        active(&snapshot.project, &snapshot.goal, &snapshot.task)?;
        let phase = snapshot
            .workflow
            .active
            .map(|i| snapshot.workflow.history[i].phase)
            .or_else(|| next_phase(&snapshot.workflow))
            .context("driven phase missing")?;
        if !matches!(phase, Phase::Issue | Phase::Worktree) {
            // In particular, never reach public capabilities/probe/start or the
            // old Task-writing binder while the private composition is absent.
            if phase.actor() != Actor::EvidencePort {
                self.preflight_native_adapter(&snapshot.task, phase)?;
            }
            bail!("typed Driver continuation after initial Evidence is unavailable");
        }
        let context = self.context(&snapshot)?;
        let ticket = crate::state::read_driver_ticket(owner.clone(), lifetime.association()?)?
            .with_gate_namespace()?;
        ticket.matches_input_view(&snapshot.task, &snapshot.record, &context)?;
        let (frame, _) = sources
            .initial_gate_frame(&snapshot.task, &snapshot.record, &context)
            .await?;
        if let Some(index) = snapshot.workflow.active {
            match snapshot.workflow.history[index].state {
                AttemptState::Running => {
                    let plan = ticket.plan_initial_gate(frame, InitialGateEdge::Claim, None)?;
                    self.store
                        .lock()
                        .map_err(|_| anyhow::anyhow!("state poisoned"))?
                        .apply_driven_initial_gate(&plan)?;
                    self.finish_driven_initial(task_id, sources, lifetime).await
                }
                // An evaluation interrupted before its private completion cannot
                // be guessed from a receipt ID or automatically invoked again.
                AttemptState::Evaluating => Ok(StepResult::Waiting {
                    phase,
                    reason: "initial gate outcome requires its retained completion/recovery".into(),
                }),
                AttemptState::Waiting => Ok(StepResult::Waiting {
                    phase,
                    reason: snapshot.workflow.history[index]
                        .detail
                        .clone()
                        .unwrap_or_else(|| "initial gate waiting".into()),
                }),
                _ => bail!("initial gate continuation needs its typed lifecycle producer"),
            }
        } else {
            let (_, source, _) = self
                .inputs(
                    &snapshot.project,
                    &snapshot.task,
                    phase,
                    snapshot.workflow.workflow,
                )
                .await?;
            ensure!(
                same_sources(&source, &snapshot.workflow.sources),
                "Driver initial sources changed"
            );
            let next = self
                .prepare_pack(
                    &snapshot.project,
                    &snapshot.task,
                    &source,
                    phase,
                    snapshot.workflow.workflow,
                    snapshot.workflow.generation,
                )
                .await?;
            let plan = ticket.plan_initial_gate(
                frame,
                InitialGateEdge::Reserve {
                    at: self.attempt_started_at(),
                },
                Some(next),
            )?;
            self.store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .apply_driven_initial_gate(&plan)?;
            Ok(StepResult::Started {
                phase,
                session: None,
            })
        }
    }
    async fn finish_driven_initial(
        &self,
        task_id: TaskId,
        sources: &Arc<ManagedWorkflowSources>,
        lifetime: &WorkerLifetime,
    ) -> Result<StepResult> {
        let owner = self
            .registry
            .managed_owner()
            .context("Runtime owner missing")?;
        let snapshot = self.read(task_id)?;
        let index = snapshot
            .workflow
            .active
            .context("initial gate claim missing")?;
        let attempt = &snapshot.workflow.history[index];
        let phase = attempt.phase;
        let context = self.context(&snapshot)?;
        // Capture BEFORE actual gate/helper awaits, never recapture after them
        // to accept newly drifted Task/governing/input authority.
        let ticket: DriverReadTicket =
            crate::state::read_driver_ticket(owner.clone(), lifetime.association()?)?
                .with_gate_namespace()?;
        ticket.matches_input_view(&snapshot.task, &snapshot.record, &context)?;
        let (frame, _) = sources
            .initial_gate_frame(&snapshot.task, &snapshot.record, &context)
            .await?;
        let invocation = PhaseInvocation {
            project: snapshot.project.clone(),
            task: snapshot.task.clone(),
            phase,
            context,
            sources: snapshot.workflow.sources.clone(),
            budget: attempt.budget.clone(),
            prerequisites: snapshot.workflow.completed.values().cloned().collect(),
            prior_observations: attempt.observations.clone(),
        };
        let completion = ManagedWorkflowGates::new(owner, sources.clone())?
            .complete_initial_driven(invocation)
            .await?;
        let (next, result) = match completion.outcome() {
            GateOutcome::Passed(_) => {
                let next_phase = snapshot
                    .workflow
                    .configured_phases
                    .iter()
                    .copied()
                    .find(|p| *p != phase && !snapshot.workflow.completed.contains_key(p))
                    .context("initial completion requires next phase")?;
                let (_, source, _) = self
                    .inputs(
                        &snapshot.project,
                        &snapshot.task,
                        next_phase,
                        snapshot.workflow.workflow,
                    )
                    .await?;
                ensure!(
                    same_sources(&source, &snapshot.workflow.sources),
                    "initial completion sources changed"
                );
                let next = self
                    .prepare_pack(
                        &snapshot.project,
                        &snapshot.task,
                        &source,
                        next_phase,
                        snapshot.workflow.workflow,
                        snapshot.workflow.generation,
                    )
                    .await?;
                (Some(next), StepResult::Completed { phase })
            }
            GateOutcome::Waiting(reason) => (
                None,
                StepResult::Waiting {
                    phase,
                    reason: reason.clone(),
                },
            ),
            GateOutcome::Failed(_) => bail!("initial Failed lifecycle producer unavailable"),
        };
        let plan = ticket.plan_initial_gate(
            frame,
            InitialGateEdge::Complete {
                completion: Box::new(completion),
                at: now_ms(),
            },
            next,
        )?;
        self.store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .apply_driven_initial_gate(&plan)?;
        Ok(result)
    }
}
