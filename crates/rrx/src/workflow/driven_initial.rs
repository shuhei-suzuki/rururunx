//! Actual initial Evidence phases; missing later typed lanes never fall back.
use super::*;
use crate::{
    execution::workflow_gates::ManagedWorkflowGates,
    execution::workflow_source::ManagedWorkflowSources,
    runtime::driver::WorkerLifetime,
    state::{DriverReadTicket, InitialGateEdge, managed_binding::InstalledDriverComposition},
};
use anyhow::bail;
impl WorkflowEngine {
    fn preflight_installed_native(
        &self,
        composition: &InstalledDriverComposition,
        task: &Task,
        phase: Phase,
    ) -> Result<Arc<crate::adapter::native::NativePhasePort>> {
        ensure!(
            std::ptr::eq(self, composition.engine().as_ref())
                && composition.is_current()
                && task.id == composition.original_task().id
                && task.executor == composition.selected().alias(),
            "installed driven Engine/Task identity differs"
        );
        let port = self.registry.native_phase_port(&task.executor)?;
        ensure!(
            Arc::ptr_eq(&port, composition.selected()),
            "installed driven port differs"
        );
        let snapshot = self.read(task.id)?;
        let workflow = &snapshot.workflow;
        ensure!(
            workflow.generation == 1
                && workflow
                    .configured_phases
                    .iter()
                    .find(|p| p.actor() == Actor::Executor)
                    == Some(&phase)
                && workflow
                    .history
                    .iter()
                    .enumerate()
                    .all(|(index, attempt)| attempt.phase.actor() != Actor::Executor
                        || (workflow.active == Some(index)
                            && index + 1 == workflow.history.len()
                            && attempt.phase == phase
                            && attempt.state == AttemptState::Running
                            && attempt.generation == workflow.generation)),
            "installed driven phase is not the original first Executor"
        );
        Ok(port)
    }

    pub(crate) async fn step_driven_initial(
        &self,
        task_id: TaskId,
        composition: &InstalledDriverComposition,
        lifetime: &WorkerLifetime,
    ) -> Result<StepResult> {
        let sources = composition.sources();
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
            // After a successful first-Executor closure: read-only Waiting for
            // the unavailable next lane (SC-N), after the Sources release.
            if let Some(waiting) = self.post_success_closure(&snapshot) {
                self.retire_settled_sources(task_id, composition, lifetime)
                    .await?;
                return Ok(waiting);
            }
            if phase.actor() == Actor::Executor {
                if let Some(index) = snapshot.workflow.active {
                    let attempt = &snapshot.workflow.history[index];
                    if attempt.state == AttemptState::Failed {
                        return Ok(StepResult::Failed {
                            phase,
                            reason: attempt.detail.clone().unwrap_or_default(),
                        });
                    }
                }
                if snapshot.workflow.active.is_some() {
                    let lookup = {
                        let runtime = composition
                            .runtime()
                            .upgrade()
                            .context("original Runtime ended")?;
                        runtime.settled_phase(task_id, &lifetime.association()?)?
                    };
                    match lookup {
                        crate::runtime::SettledLookup::Settled(success) => {
                            return self
                                .continue_settled_executor(
                                    snapshot,
                                    composition,
                                    lifetime,
                                    success,
                                    phase,
                                )
                                .await;
                        }
                        crate::runtime::SettledLookup::Held(reason) => {
                            return Ok(StepResult::Waiting {
                                phase,
                                reason: reason.into(),
                            });
                        }
                        _ => {}
                    }
                    return self
                        .offer_driven_first_executor(snapshot, composition, lifetime, phase)
                        .await;
                }
                // Keep this unchanged genuine-composition refusal before every
                // namespace helper and every preparatory write.
                let port = self.preflight_installed_native(composition, &snapshot.task, phase)?;
                return self
                    .reserve_driven_first_executor(snapshot, sources, lifetime, port)
                    .await;
            }
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
    async fn offer_driven_first_executor(
        &self,
        snapshot: Snapshot,
        composition: &InstalledDriverComposition,
        lifetime: &WorkerLifetime,
        phase: Phase,
    ) -> Result<StepResult> {
        let sources = composition.sources();
        let runtime = composition.runtime();
        let observation = {
            let runtime = runtime.upgrade().context("original Runtime ended")?;
            runtime.observe_source_handoff(snapshot.task.id)?
        };
        if let Some(observation) = observation {
            // Observation never authorizes a new offer, retry or Workflow write.
            let waiting = self.waiting_observation(snapshot.task.id)?;
            let reason = match waiting {
                Some(PhaseWaitingObservation::Waiting { reason, next_due }) => format!(
                    "original Source handoff {:?}; {:?} waiting until {next_due}",
                    observation.state(),
                    reason
                ),
                Some(PhaseWaitingObservation::Held) => format!(
                    "original Source handoff {:?}; Held: Native waiting facts require attention",
                    observation.state()
                ),
                None => format!(
                    "original Source handoff {:?}; marker/binding/terminal continuation required",
                    observation.state()
                ),
            };
            return Ok(StepResult::Waiting { phase, reason });
        }
        // Unchanged genuine-composition preflight precedes callbacks and offer.
        let port = self.preflight_installed_native(composition, &snapshot.task, phase)?;
        let owner = self
            .registry
            .managed_owner()
            .context("Driver Runtime owner missing")?;
        let context = self.context(&snapshot)?;
        let ticket = crate::state::read_driver_ticket(owner, lifetime.association()?)?
            .with_gate_namespace()?;
        ticket.matches_input_view(&snapshot.task, &snapshot.record, &context)?;
        let reservation = {
            let runtime = runtime.upgrade().context("original Runtime ended")?;
            runtime.reserve_source_handoff(&ticket, &port)?
        };
        // This concrete inline port retains the returned envelope in SAME poll.
        // Its eager Drop ends the offer future BEFORE removing original EMPTY.
        let observation = reservation
            .offer_first_executor(
                sources,
                &snapshot.task,
                &snapshot.record,
                &context,
                ticket,
                port,
            )
            .await?;
        Ok(StepResult::Waiting {
            phase,
            reason: format!(
                "original Source handoff {:?}; marker/binding/terminal continuation required",
                observation.state()
            ),
        })
    }
    async fn reserve_driven_first_executor(
        &self,
        snapshot: Snapshot,
        sources: &Arc<ManagedWorkflowSources>,
        lifetime: &WorkerLifetime,
        selected: Arc<crate::adapter::native::NativePhasePort>,
    ) -> Result<StepResult> {
        let owner = self
            .registry
            .managed_owner()
            .context("Driver Runtime owner missing")?;
        let context = self.context(&snapshot)?;
        let ticket = crate::state::read_driver_ticket(owner, lifetime.association()?)?
            .with_gate_namespace()?;
        ticket.matches_input_view(&snapshot.task, &snapshot.record, &context)?;
        let phase = next_phase(&snapshot.workflow).context("first Executor phase missing")?;
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
            "first Executor source changed"
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
        let (completion, ticket) = sources
            .first_executor_frame(&snapshot.task, &snapshot.record, &context, ticket)
            .await?;
        let plan =
            ticket.plan_first_executor(completion, selected, next, self.attempt_started_at())?;
        self.store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .reserve_driven_first_executor(&plan)?;
        Ok(StepResult::Started {
            phase,
            session: None,
        })
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
