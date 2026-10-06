//! Prescribed initial Evidence edges of the SAME actual pre-marker worker.
use super::*;
use crate::execution::workflow_gates::InitialGateCompletion;
use crate::workflow::{
    AttemptState, GateObservation, GateOutcome, Phase, PhaseAttempt, WorkflowSnapshot,
};

pub(crate) enum InitialGateEdge {
    Reserve {
        at: i64,
    },
    Claim,
    Complete {
        completion: Box<InitialGateCompletion>,
        at: i64,
    },
}
pub(super) struct GateInput {
    pub(super) index: usize,
    pub(super) observed_before: Option<Record>,
    pub(super) observation: Option<GateObservation>,
    receipt: Option<Record>,
    _completion: Option<Box<InitialGateCompletion>>,
}
impl GateInput {
    pub(super) fn validate_before_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        if let Some(receipt) = &self.receipt {
            let body = serde_json::to_string(receipt)?;
            let exact:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE id=?1 AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND kind='verification' AND version=?5 AND body=?6)",params![receipt.id.to_string(),receipt.scope.project_id.to_string(),receipt.scope.goal_id.map(|v|v.to_string()),receipt.scope.task_id.map(|v|v.to_string()),receipt.version,body],|r|r.get(0))?;
            ensure!(exact, "actual initial gate receipt changed");
        }
        Ok(())
    }
}
pub(super) fn initial_only(w: &WorkflowSnapshot) -> Result<()> {
    ensure!(
        w.generation == 1
            && !w.finished
            && w.terminal_decision.is_none()
            && w.sources.artifact.is_none()
            && w.escalations.is_empty()
            && w.invalidations.is_empty()
            && w.retries.is_empty()
            && w.finalizations.is_empty()
            && w.history.len() <= 2
            && w.history
                .iter()
                .all(|a| matches!(a.phase, Phase::Issue | Phase::Worktree)
                    && a.unit.is_none()
                    && a.execution.is_none()
                    && a.session_id.is_none()
                    && !a.dispatch_started
                    && a.agent.is_none()
                    && a.native_wait.is_none()
                    && a.next_due.is_none())
            && w.completed
                .keys()
                .all(|p| matches!(p, Phase::Issue | Phase::Worktree)),
        "driven continuation is not an initial Evidence phase"
    );
    Ok(())
}
impl DriverReadTicket {
    /// Only the real worker's actual Sources frame can prescribe these edges.
    /// No caller-provided Task/Workflow post image is accepted.
    pub(crate) fn plan_initial_gate(
        self,
        frame: crate::execution::workflow_source::InitialInputFrame,
        edge: InitialGateEdge,
        new_context: Option<ContextVersion>,
    ) -> Result<Arc<DriverPreparationAdvance>> {
        ensure!(
            self.row.marker.is_none() && self.source.is_none() && self.namespace.is_some(),
            "initial Evidence requires original pre-marker lane"
        );
        frame.validate(&self.owner)?;
        self.preparation_matches(frame.unit())?;
        let unit = frame.unit().clone();
        ensure!(
            unit.phase == WORKFLOW_SOURCE_BOOTSTRAP && unit.state == UnitState::Preparing,
            "initial Evidence Unit changed"
        );
        let (old_record, old_context) = self.scope.workflow_input()?;
        let old_record = old_record.clone();
        let mut context = old_context.clone();
        let mut record = old_record.clone();
        let mut task = self.task().clone();
        let mut w: WorkflowSnapshot = serde_json::from_value(record.data.clone())?;
        initial_only(&w)?;
        frame.validate_context(&task, &record, &context)?;
        let fresh_context = new_context.is_some();
        let mut gate = GateInput {
            index: 0,
            observed_before: None,
            observation: None,
            receipt: None,
            _completion: None,
        };
        let at = match edge {
            InitialGateEdge::Reserve { at } => {
                ensure!(
                    w.active.is_none()
                        && w.history.iter().all(|a| a.state == AttemptState::Succeeded),
                    "initial gate has active or unclosed history"
                );
                let phase = crate::workflow::next_phase(&w).context("next phase missing")?;
                ensure!(
                    matches!(phase, Phase::Issue | Phase::Worktree),
                    "native continuation unavailable"
                );
                context = new_context.context("reservation needs new Context")?;
                w.context_version = context.version;
                w.context_fresh = true;
                w.sources.payload = context.data["payload"]
                    .as_str()
                    .context("payload missing")?
                    .into();
                task.context_version = context.version;
                task.revision = Some(context.revision.clone());
                task.phase = Some(phase.key().into());
                task.state = phase.task_state();
                let index = w.history.len();
                let budget = serde_json::from_value(context.data["budget"].clone())?;
                w.history.push(PhaseAttempt {
                    phase,
                    generation: w.generation,
                    context_version: context.version,
                    budget,
                    state: AttemptState::Running,
                    session_id: None,
                    execution: None,
                    unit: None,
                    native_wait: None,
                    next_due: None,
                    dispatch_started: false,
                    observations: vec![],
                    claimed_observations: 0,
                    agent: None,
                    started_at: at,
                    completed_at: None,
                    detail: None,
                });
                w.active = Some(index);
                gate.index = index;
                at
            }
            InitialGateEdge::Claim => {
                ensure!(!fresh_context, "gate claim cannot replace input");
                let index = w.active.context("gate claim requires active attempt")?;
                let attempt = &mut w.history[index];
                ensure!(
                    attempt.state == AttemptState::Running
                        && attempt.observations.is_empty()
                        && attempt.claimed_observations == 0,
                    "initial gate cannot replay prior outcomes"
                );
                attempt.state = AttemptState::Evaluating;
                gate.index = index;
                now_ms()
            }
            InitialGateEdge::Complete { completion, at } => {
                let index = w.active.context("completion requires active gate")?;
                let attempt = &w.history[index];
                ensure!(
                    attempt.state == AttemptState::Evaluating
                        && attempt.observations.is_empty()
                        && attempt.claimed_observations == 0,
                    "initial outcome already recorded or unclaimed"
                );
                let invocation = completion.invocation();
                let (project, _) = self.scope.governing_owners();
                ensure!(
                    serde_json::to_value(&invocation.task)? == serde_json::to_value(&task)?
                        && serde_json::to_value(&invocation.project)?
                            == serde_json::to_value(project)?
                        && invocation.phase == attempt.phase
                        && invocation.sources == w.sources
                        && serde_json::to_value(&invocation.context)?
                            == serde_json::to_value(&context)?
                        && invocation.budget == attempt.budget
                        && serde_json::to_value(&invocation.prerequisites)?
                            == serde_json::to_value(w.completed.values().collect::<Vec<_>>())?
                        && invocation.prior_observations.is_empty(),
                    "actual gate completion belongs to another claim"
                );
                let mut sources = w.sources.clone();
                sources.payload.clear();
                let observation = GateObservation {
                    sources,
                    outcome: Some(completion.outcome().clone()),
                    error: None,
                    at,
                };
                // This exact virtual observer image is derived ONLY from the
                // concrete completion; ordinary transition/observer rules stay unchanged.
                let mut observed = w.clone();
                observed.history[index]
                    .observations
                    .push(observation.clone());
                observed.history[index].detail = Some("gate outcome observed".into());
                let mut before = old_record.clone();
                before.data = serde_json::to_value(&observed)?;
                gate.index = index;
                gate.observed_before = Some(before);
                gate.observation = Some(observation);
                gate.receipt = completion.receipt().cloned();
                w = observed;
                match completion.outcome() {
                    GateOutcome::Passed(evidence) => {
                        crate::workflow::validate_evidence(
                            evidence,
                            &task.scope(),
                            &w.sources,
                            &w.history[index],
                        )?;
                        ensure!(gate.receipt.is_some(), "genuine Passed receipt absent");
                        task.artifacts.extend(evidence.artifacts.iter().cloned());
                        w.completed.insert(invocation.phase, evidence.clone());
                        w.history[index].state = AttemptState::Succeeded;
                        w.history[index].completed_at = Some(at);
                        w.active = None;
                        context = new_context.context("Passed needs next Context")?;
                        task.context_version = context.version;
                        task.revision = Some(context.revision.clone());
                        w.context_version = context.version;
                        w.context_fresh = true;
                        w.sources.payload = context.data["payload"]
                            .as_str()
                            .context("payload missing")?
                            .into();
                        task.phase = crate::workflow::next_phase(&w).map(|p| p.key().into());
                        task.state = invocation.phase.task_state();
                    }
                    GateOutcome::Waiting(reason) => {
                        ensure!(
                            !fresh_context && !reason.trim().is_empty() && reason.len() <= 4096,
                            "Waiting cannot replace input or have unbounded detail"
                        );
                        w.history[index].state = AttemptState::Waiting;
                        w.history[index].detail = Some(reason.clone());
                        task.state = TaskState::WaitingHuman;
                        task.blockers.push(reason.clone());
                    }
                    GateOutcome::Failed(_) => {
                        bail!("initial Failed lifecycle producer unavailable")
                    }
                }
                gate._completion = Some(completion);
                at
            }
        };
        record.data = serde_json::to_value(w)?;
        frame.validate_context(&task, &record, &context)?;
        crate::workflow::validate_transition(
            &task,
            &record,
            gate.observed_before.as_ref().or(Some(&old_record)),
        )?;
        let before = record.clone();
        bump(&mut task.version)?;
        task.updated_at = at;
        bump(&mut record.version)?;
        record.updated_at = at;
        let task_body = serde_json::to_string(&task)?;
        let record_body = serde_json::to_string(&record)?;
        let context_body = serde_json::to_string(&context)?;
        ensure!(
            task_body.len() <= 1024 * 1024
                && record_body.len() <= 8 * 1024 * 1024
                && context_body.len() <= 8 * 1024 * 1024,
            "initial Evidence image exceeds bounds"
        );
        let mut next = self.row.clone();
        bump(&mut next.version)?;
        next.pins.task = pin(task.version, &task)?;
        next.pins.workflow = Some((record.id, pin(record.version, &record)?));
        next.pins.context = Some(pin(context.version, &context)?);
        let body = serde_json::to_string(&next)?;
        ensure!(
            body.len() <= 128 * 1024,
            "initial Evidence Driver exceeds bound"
        );
        let (p, g) = self.scope.governing_owners();
        let governing = crate::state::execution::governing_digest(p, g)?;
        ensure!(
            frame.governing() == governing,
            "initial Evidence governing changed"
        );
        let unit_body = serde_json::to_string(&unit)?;
        let plan = Arc::new(DriverPreparationAdvance {
            ticket: self,
            unit,
            unit_body,
            task,
            task_body,
            next,
            body,
            initial: false,
            governing,
            input: Some(InitialInput {
                gate: Some(gate),
                executor: None,
                fresh_context,
                frame,
                record_before: before,
                record,
                record_body,
                context,
                context_body,
            }),
            reconcile_ready: std::sync::atomic::AtomicBool::new(false),
        });
        plan.ticket.association.retain_preparation(&plan)?;
        Ok(plan)
    }
}
impl DriverPreparationAdvance {
    pub(in crate::state) fn gate_write(
        &self,
    ) -> Result<(Task, Record, Option<&ContextVersion>, u64, u64)> {
        let input = self.input.as_ref().context("initial gate input missing")?;
        ensure!(input.gate.is_some(), "not a gate plan");
        let mut task = self.task.clone();
        task.version = self.ticket.task().version;
        task.updated_at = self.ticket.task().updated_at;
        Ok((
            task,
            input.record_before.clone(),
            input.fresh_context.then_some(&input.context),
            self.next.pins.project.version,
            self.next.pins.goal.version,
        ))
    }
}
