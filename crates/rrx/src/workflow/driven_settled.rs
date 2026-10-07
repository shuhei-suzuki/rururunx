//! SC Driver continuation of the ORIGINAL settled first-Executor phase. At
//! most one Workflow/ledger-advancing write per call, only through the
//! continuation's single-flight stage token; every await precedes the Store.
use super::*;
use crate::{
    runtime::{
        ClosureStage, ClosureState, Retained, SettledLookup, SuccessContinuation, SuccessStage,
        driver::WorkerLifetime,
    },
    state::managed_binding::{
        InstalledDriverComposition, SettledPhase, SuccessWrite, plan_settled_currency,
    },
};

const HELD_UNCERTAIN: &str = "settled write outcome uncertain; Held";

/// The SAME worker's borrowed inputs for one settled step.
struct SettledStep<'a> {
    snapshot: &'a Snapshot,
    composition: &'a InstalledDriverComposition,
    lifetime: &'a WorkerLifetime,
    owner: &'a Arc<crate::execution::RuntimeOwner>,
    settled: &'a Arc<SettledPhase>,
}
const HELD_UNKNOWN: &str = "gate outcome unknown; explicit recovery required";

impl WorkflowEngine {
    /// Read-only Waiting after a successful closure, replacing only the
    /// missing-lane error for this durable state. `None` when not that state.
    pub(super) fn post_success_closure(&self, snapshot: &Snapshot) -> Option<StepResult> {
        let workflow = &snapshot.workflow;
        let last = workflow.history.last()?;
        let next = next_phase(workflow)?;
        (workflow.active.is_none()
            && !workflow.finished
            && last.phase.actor() == Actor::Executor
            && last.state == AttemptState::Succeeded
            && workflow.completed.contains_key(&last.phase)
            && workflow
                .configured_phases
                .iter()
                .find(|p| p.actor() == Actor::Executor)
                == Some(&last.phase))
        .then(|| StepResult::Waiting {
            phase: next,
            reason: format!(
                "{} typed Driver continuation unavailable (SC-N)",
                next.key()
            ),
        })
    }
    /// Once each after a Known, published closure: release the SAME Sources
    /// slot handoff custody. Nothing else is written.
    pub(super) async fn retire_settled_sources(
        &self,
        task: TaskId,
        composition: &InstalledDriverComposition,
        lifetime: &WorkerLifetime,
    ) -> Result<()> {
        let lookup = {
            let runtime = composition
                .runtime()
                .upgrade()
                .context("original Runtime ended")?;
            runtime.settled_phase(task, &lifetime.association()?)?
        };
        let SettledLookup::Settled(success) = lookup else {
            return Ok(());
        };
        let mut stage = success.lock_stage().await;
        if let Some(ClosureStage {
            state:
                ClosureState::Known {
                    ack,
                    published: true,
                },
            ..
        }) = &stage.closure
            && !stage.sources_retired
        {
            let ack = ack.clone();
            composition
                .sources()
                .retire_closed_handoff(task, &ack)
                .await?;
            stage.sources_retired = true;
        }
        Ok(())
    }

    pub(super) async fn continue_settled_executor(
        &self,
        snapshot: Snapshot,
        composition: &InstalledDriverComposition,
        lifetime: &WorkerLifetime,
        success: Arc<SuccessContinuation>,
        phase: Phase,
    ) -> Result<StepResult> {
        let owner = self
            .registry
            .managed_owner()
            .context("Driver Runtime owner missing")?;
        let settled = success.settled().clone();
        let index = snapshot
            .workflow
            .active
            .context("settled Driver active attempt absent")?;
        let attempt = snapshot.workflow.history[index].clone();
        ensure!(
            attempt.phase == settled.phase() && phase == settled.phase(),
            "settled Driver phase differs"
        );
        let mut stage = success.lock_stage().await;
        let waiting = |reason: &str| {
            Ok(StepResult::Waiting {
                phase,
                reason: reason.into(),
            })
        };
        let step = SettledStep {
            snapshot: &snapshot,
            composition,
            lifetime,
            owner: &owner,
            settled: &settled,
        };
        match attempt.state {
            AttemptState::Running => self.settled_claim(&mut stage, &step, phase).await,
            AttemptState::Evaluating => self.settled_evaluation(&mut stage, &step, index).await,
            // The gate's fixed Waiting/held detail; no automatic re-claim.
            AttemptState::Waiting => waiting(attempt.detail.as_deref().unwrap_or("workflow held")),
            _ => waiting("settled phase requires explicit recovery"),
        }
    }

    async fn settled_claim(
        &self,
        stage: &mut SuccessStage,
        step: &SettledStep<'_>,
        phase: Phase,
    ) -> Result<StepResult> {
        let (snapshot, composition, owner, settled) =
            (step.snapshot, step.composition, step.owner, step.settled);
        let plan = match stage.claim.take() {
            Some(Retained::RolledBack(plan)) => plan,
            Some(other) => {
                stage.claim = Some(other);
                return Ok(StepResult::Waiting {
                    phase,
                    reason: HELD_UNCERTAIN.into(),
                });
            }
            None => {
                let index = snapshot.workflow.active.context("settled claim attempt")?;
                let budget = snapshot.workflow.history[index].budget.clone();
                let currency = Arc::new(plan_settled_currency(owner, settled)?);
                let source = composition
                    .sources()
                    .capture_settled(&currency, &snapshot.project, &snapshot.task, phase, &budget)
                    .await?;
                // Drift policy: no write for an invalidating change of a bound
                // open phase (that needs its own typed port).
                let (config, current, _) = self
                    .inputs(
                        &snapshot.project,
                        &snapshot.task,
                        phase,
                        snapshot.workflow.workflow,
                    )
                    .await?;
                let class = snapshot.workflow.workflow;
                if class < config.minimum_workflow
                    || class < risk_workflow(&config, snapshot.task.risk)
                    || rules_changed(&source, &current)
                    || (!target_producing(phase) && !same_sources(&source, &current))
                {
                    return Ok(StepResult::Waiting {
                        phase,
                        reason: "settled sources or policy drifted; Held".into(),
                    });
                }
                crate::state::Store::plan_settled_gate_claim(
                    owner,
                    plan_settled_currency(owner, settled)?,
                    now_ms(),
                )?
            }
        };
        let write = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .claim_settled_gate(&plan);
        stage.claim = Retained::from_write(plan, write);
        Ok(StepResult::Started {
            phase,
            session: None,
        })
    }

    async fn settled_evaluation(
        &self,
        stage: &mut SuccessStage,
        step: &SettledStep<'_>,
        index: usize,
    ) -> Result<StepResult> {
        let (snapshot, composition, owner, settled) =
            (step.snapshot, step.composition, step.owner, step.settled);
        let attempt = &snapshot.workflow.history[index];
        let phase = attempt.phase;
        let waiting = |reason: &str| {
            Ok(StepResult::Waiting {
                phase,
                reason: reason.into(),
            })
        };
        // A Known observation no longer needs the claim or the completion
        // (released below when it became Known; SC §12 compact retention).
        if let Some(Retained::Known(ack)) = stage.observed.as_ref() {
            let ack = ack.clone();
            if ack.passed().is_none() {
                return waiting("settled gate observed; Workflow re-read");
            }
            return self.settled_closure(stage, step, &ack).await;
        }
        let Some(claim) = stage.claim.as_ref().and_then(Retained::known).cloned() else {
            return waiting(HELD_UNKNOWN);
        };
        let observed_plan = match stage.observed.take() {
            Some(Retained::RolledBack(plan)) => plan,
            Some(other) => {
                stage.observed = Some(other);
                return waiting(HELD_UNCERTAIN);
            }
            None => {
                if stage.completion.is_none() {
                    if stage.evaluation_started {
                        return waiting(HELD_UNKNOWN);
                    }
                    stage.evaluation_started = true;
                    let currency = Arc::new(plan_settled_currency(owner, settled)?);
                    let sources = composition
                        .sources()
                        .capture_settled(
                            &currency,
                            &snapshot.project,
                            &snapshot.task,
                            phase,
                            &attempt.budget,
                        )
                        .await?;
                    let invocation = PhaseInvocation {
                        project: snapshot.project.clone(),
                        task: snapshot.task.clone(),
                        phase,
                        context: self.context(snapshot)?,
                        sources,
                        budget: attempt.budget.clone(),
                        prerequisites: snapshot.workflow.completed.values().cloned().collect(),
                        prior_observations: attempt.observations.clone(),
                    };
                    match composition
                        .gates()
                        .evaluate_settled(&claim, invocation)
                        .await
                    {
                        Ok(completion) => stage.completion = Some(completion),
                        Err(_cause) => return waiting(HELD_UNKNOWN),
                    }
                }
                // The sealed completion stays in the stage until its
                // observation is Known; a refused plan is this turn's Held.
                let completion = stage
                    .completion
                    .as_ref()
                    .context("settled gate completion absent")?;
                match crate::state::Store::plan_settled_gate_observed(
                    owner,
                    &claim,
                    completion,
                    now_ms(),
                ) {
                    Ok(plan) => plan,
                    Err(_cause) => return waiting(HELD_UNKNOWN),
                }
            }
        };
        let write = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .observe_settled_gate(&observed_plan);
        stage.observed = Retained::from_write(observed_plan, write);
        if matches!(stage.observed, Some(Retained::Known(_))) {
            stage.completion = None;
            stage.claim = None;
        }
        Ok(StepResult::Started {
            phase,
            session: None,
        })
    }

    async fn settled_closure(
        &self,
        stage: &mut SuccessStage,
        step: &SettledStep<'_>,
        observed: &Arc<crate::state::managed_binding::GateObservedAcknowledgment>,
    ) -> Result<StepResult> {
        let (snapshot, composition, lifetime, owner, settled) = (
            step.snapshot,
            step.composition,
            step.lifetime,
            step.owner,
            step.settled,
        );
        let index = snapshot
            .workflow
            .active
            .context("settled closure attempt")?;
        let phase = snapshot.workflow.history[index].phase;
        let plan = match stage.closure.take() {
            Some(ClosureStage {
                plan,
                state: ClosureState::RolledBack,
            }) => plan,
            Some(other) => {
                stage.closure = Some(other);
                return Ok(StepResult::Waiting {
                    phase,
                    reason: HELD_UNCERTAIN.into(),
                });
            }
            None => {
                let (evidence, _) = observed
                    .passed()
                    .context("settled closure requires a Passed observation")?;
                let markers = evidence
                    .artifacts
                    .iter()
                    .filter_map(|s| s.strip_prefix("rrx-artifact:"))
                    .collect::<Vec<_>>();
                ensure!(
                    markers.len() == 1,
                    "settled closure requires one retained artifact"
                );
                #[cfg(test)]
                crate::runtime::pause_at(Some(snapshot.task.id), crate::runtime::SETTLED_CLOSURE);
                let currency = Arc::new(plan_settled_currency(owner, settled)?);
                let publication = crate::execution::results::ResultStore::new(owner.clone())
                    .settled_publication(&currency, markers[0].parse()?)
                    .await?;
                let source = snapshot.workflow.history[index]
                    .observations
                    .last()
                    .context("settled closure observation absent")?
                    .sources
                    .clone();
                let mut task = snapshot.task.clone();
                let mut workflow = snapshot.workflow.clone();
                let next = succeed_attempt(
                    &mut task,
                    &mut workflow,
                    index,
                    evidence.clone(),
                    source.clone(),
                    now_ms,
                );
                let context = self
                    .prepare_pack(
                        &snapshot.project,
                        &task,
                        &source,
                        next.unwrap_or(phase),
                        snapshot.workflow.workflow,
                        snapshot.workflow.generation,
                    )
                    .await?;
                crate::state::Store::plan_phase_success(
                    owner,
                    observed,
                    publication,
                    context,
                    now_ms(),
                )?
            }
        };
        let material = crate::state::Store::materialize_phase_success(&plan)?;
        #[cfg(test)]
        crate::runtime::pause_at(Some(snapshot.task.id), crate::runtime::SUCCESS_ADMISSION);
        let admission = composition.admit_success(lifetime).await?;
        let (write, published) = {
            let mut store = self
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let write = store.close_phase_success(&material);
            let published = match &write {
                Ok(SuccessWrite::Known(ack)) => store.publish_success_driver(ack).is_ok(),
                _ => false,
            };
            (write, published)
        };
        drop(material);
        drop(admission);
        match write {
            Ok(SuccessWrite::Known(ack)) => {
                stage.closure = Some(ClosureStage {
                    plan,
                    state: ClosureState::Known {
                        ack: ack.clone(),
                        published,
                    },
                });
                if published {
                    composition
                        .sources()
                        .retire_closed_handoff(snapshot.task.id, &ack)
                        .await?;
                    stage.sources_retired = true;
                }
            }
            Ok(SuccessWrite::Conflict(_cause)) => stage.closure = None,
            Err(_cause) => {
                stage.closure = Some(ClosureStage {
                    plan,
                    state: ClosureState::Uncertain,
                })
            }
        }
        Ok(StepResult::Started {
            phase,
            session: None,
        })
    }
}
