//! SC Root sweep: binding convergence (D1), settlement discovery (D2) and
//! continuation housekeeping (D3). Classification runs under short job locks;
//! each turn takes at most one snapshot plus one Store transaction.
use super::*;
use crate::runtime::installation::SuccessAdmission;
use crate::state::managed_binding::{
    GateClaimAcknowledgment, GateClaimPlan, GateObservedAcknowledgment, GateObservedPlan,
    ManagedBindingConfirmation, SettledPhase, SuccessClosureAcknowledgment, SuccessClosurePlan,
    SuccessConfirmation, SuccessWrite, plan_late_binding,
};

const SUCCESS_TURNS: usize = 8;
const SETTLEMENT_REPOLL_MS: u64 = 5000;

/// Retained once per job after a bound owned success is observed. The stage
/// mutex is a leaf and the single-flight token for Driver and Root writers.
pub(crate) struct SuccessContinuation {
    settled: Arc<SettledPhase>,
    stage: tokio::sync::Mutex<SuccessStage>,
}
/// Retained compact plans and acknowledgments of the continuation. None of
/// them points back to a Job, PhaseJobs or Runtime.
#[derive(Default)]
pub(crate) struct SuccessStage {
    pub(crate) claim: Option<Retained<GateClaimPlan, GateClaimAcknowledgment>>,
    /// Set before evaluation starts; never re-evaluated afterwards.
    pub(crate) evaluation_started: bool,
    pub(crate) completion: Option<crate::execution::workflow_gates::SettledGateCompletion>,
    pub(crate) observed: Option<Retained<GateObservedPlan, GateObservedAcknowledgment>>,
    pub(crate) closure: Option<ClosureStage>,
    /// Set once the SAME Driver retired its Sources slot handoff custody.
    pub(crate) sources_retired: bool,
}
/// The retained closure: its SAME plan and what is known about it.
pub(crate) struct ClosureStage {
    pub(crate) plan: Arc<SuccessClosurePlan>,
    pub(crate) state: ClosureState,
}
pub(crate) enum ClosureState {
    /// The write returned Err: only the SAME plan may be confirmed.
    Uncertain,
    /// Confirmation proved the SAME plan rolled back.
    RolledBack,
    /// Known; `published` once the SAME association published the Driver.
    Known {
        ack: Arc<SuccessClosureAcknowledgment>,
        published: bool,
    },
}
/// One retained write: its SAME plan and what its write or confirmation
/// established. A typed Conflict drops the plan.
pub(crate) enum Retained<P, A> {
    /// The write returned Err: only the SAME plan may be confirmed.
    Uncertain(Arc<P>),
    /// Confirmation proved the SAME plan rolled back; retry it unchanged.
    RolledBack(Arc<P>),
    Known(Arc<A>),
}
impl<P, A> Retained<P, A> {
    pub(crate) fn known(&self) -> Option<&Arc<A>> {
        match self {
            Self::Known(ack) => Some(ack),
            _ => None,
        }
    }
    pub(crate) fn is_uncertain(&self) -> bool {
        matches!(self, Self::Uncertain(_))
    }
    /// Records a write outcome of the SAME plan: Known, uncertain, or None
    /// for a typed Conflict (the plan is dropped).
    pub(crate) fn from_write(plan: Arc<P>, write: Result<SuccessWrite<A>>) -> Option<Self> {
        match write {
            Ok(SuccessWrite::Known(ack)) => Some(Self::Known(ack)),
            Ok(SuccessWrite::Conflict(_cause)) => None,
            Err(_cause) => Some(Self::Uncertain(plan)),
        }
    }
    /// Records a confirmation of the SAME plan; Err stays uncertain (Held).
    pub(crate) fn from_confirmation(
        plan: Arc<P>,
        confirmation: Result<SuccessConfirmation<A>>,
    ) -> (Self, bool) {
        match confirmation {
            Ok(SuccessConfirmation::Known(ack)) => (Self::Known(ack), true),
            Ok(SuccessConfirmation::RolledBack) => (Self::RolledBack(plan), true),
            Err(_cause) => (Self::Uncertain(plan), false),
        }
    }
}
impl SuccessStage {
    /// Root D3: an uncertain claim or observed write awaiting confirmation.
    fn has_uncertain(&self) -> bool {
        self.claim.as_ref().is_some_and(Retained::is_uncertain)
            || self.observed.as_ref().is_some_and(Retained::is_uncertain)
    }
    /// Root D3: an admitted action (closure confirmation, publication retry).
    fn admitted_action(&self) -> Option<ActionKind> {
        match &self.closure.as_ref()?.state {
            ClosureState::Uncertain => Some(ActionKind::ConfirmClosure),
            ClosureState::Known {
                published: false, ..
            } => Some(ActionKind::RetryPublication),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActionKind {
    ConfirmClosure,
    RetryPublication,
}
/// One admitted Root turn, run by the service loop under its own
/// non-blocking admission after the synchronous sweep returns.
pub(in crate::runtime) struct SuccessAction {
    job: Weak<Job>,
    success: Arc<SuccessContinuation>,
    kind: ActionKind,
}
pub(in crate::runtime) struct SuccessSweep {
    actions: Vec<SuccessAction>,
    releases: Vec<SuccessRelease>,
    pending: bool,
}
impl SuccessSweep {
    pub(in crate::runtime) fn into_actions(
        self,
    ) -> (Vec<SuccessAction>, Vec<SuccessRelease>, bool) {
        (self.actions, self.releases, self.pending)
    }
}
/// A closed, published and Sources-retired success whose Root custody can
/// be released; pointer-checked, retried without any DB write.
pub(in crate::runtime) struct SuccessRelease {
    job: Weak<Job>,
    ack: Arc<SuccessClosureAcknowledgment>,
}
impl crate::runtime::Runtime {
    /// Root release order: the PhaseSupervisor slot, the Source handoff, then
    /// the PhaseJobs entry. Nothing here writes the database.
    pub(in crate::runtime) fn release_success(&self, release: SuccessRelease) -> Result<()> {
        let Some(job) = release.job.upgrade() else {
            return Ok(());
        };
        let closed = ClosedPhaseAck::Success(release.ack.clone());
        let released = {
            let _depth = RootLockDepth::enter();
            job.state
                .lock()
                .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?
                .slot_released
        };
        if !released {
            self.phases.retire_closed_marked(&closed)?;
            let _depth = RootLockDepth::enter();
            job.state
                .lock()
                .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?
                .slot_released = true;
        }
        let task = release
            .ack
            .settled()
            .marker()
            .scope()
            .task_id
            .ok_or_else(|| anyhow::anyhow!("success release Task absent"))?;
        self.phase_handoffs.retire_closed(task, &closed)?;
        self.phase_jobs.retire_closed(&job)
    }
}
impl SuccessAction {
    #[cfg(test)]
    pub(in crate::runtime) fn task(&self) -> Option<crate::domain::TaskId> {
        self.success.settled().marker().scope().task_id
    }
    /// ONE synchronous Store turn under the caller's admission. The Root
    /// confirms or publishes only; it never commits a new closure.
    pub(in crate::runtime) fn run(self, admitted: &SuccessAdmission) -> Result<()> {
        let owner = admitted.runtime().owner.clone();
        let Some(mut stage) = self.success.try_stage()? else {
            return Ok(());
        };
        let Some(closure) = stage.closure.as_mut() else {
            return Ok(());
        };
        match (self.kind, &closure.state) {
            (ActionKind::ConfirmClosure, ClosureState::Uncertain) => {
                #[cfg(test)]
                pause_at(
                    self.success.settled().marker().scope().task_id,
                    CLOSURE_CONFIRM,
                );
                let material = crate::state::Store::materialize_phase_success(&closure.plan)?;
                let mut store = owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("success Store poisoned"))?;
                match store.confirm_phase_success(&material) {
                    Ok(SuccessConfirmation::Known(ack)) => {
                        let published = store.publish_success_driver(&ack).is_ok();
                        drop(store);
                        closure.state = ClosureState::Known {
                            ack: ack.clone(),
                            published,
                        };
                        drop(stage);
                        if let Some(job) = self.job.upgrade() {
                            job.install_success_ack(&self.success, ack)?;
                        }
                    }
                    Ok(SuccessConfirmation::RolledBack) => {
                        closure.state = ClosureState::RolledBack;
                    }
                    Err(_cause) => {
                        drop(store);
                        drop(stage);
                        if let Some(job) = self.job.upgrade() {
                            job.success_held("success closure outcome uncertain; Held", 5000)?;
                        }
                    }
                }
            }
            (ActionKind::RetryPublication, ClosureState::Known { ack, .. }) => {
                let ack = ack.clone();
                let published = owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("success Store poisoned"))?
                    .publish_success_driver(&ack)
                    .is_ok();
                closure.state = ClosureState::Known { ack, published };
            }
            _ => {}
        }
        Ok(())
    }
}
impl SuccessContinuation {
    pub(crate) fn settled(&self) -> &Arc<SettledPhase> {
        &self.settled
    }
    /// Single-flight: Some only if no other caller holds the stage.
    pub(crate) fn try_stage(&self) -> Result<Option<tokio::sync::MutexGuard<'_, SuccessStage>>> {
        Ok(self.stage.try_lock().ok())
    }
    /// The SAME live Driver worker's single-flight token; it may be held
    /// across the worker's own capture and gate awaits, never under Store.
    pub(crate) async fn lock_stage(&self) -> tokio::sync::MutexGuard<'_, SuccessStage> {
        self.stage.lock().await
    }
}

/// What one turn did, for the cfg(test) turn observer only.
enum Turn {
    Skipped,
    Taken(&'static str),
    /// Returned for the service loop's admitted segment.
    Action(Arc<SuccessContinuation>, ActionKind),
    /// Returned for the service loop's Root release.
    Release(Arc<SuccessClosureAcknowledgment>),
}

impl PhaseJobs {
    #[cfg(test)]
    pub(in crate::runtime) fn observed_success_turns(&self) -> Vec<(OperationId, &'static str)> {
        self.success_turns.lock().unwrap().clone()
    }
    fn set_success_cursor(&self, cursor: Option<OperationId>) -> Result<()> {
        let _depth = RootLockDepth::enter();
        *self
            .success_cursor
            .lock()
            .map_err(|_| anyhow::anyhow!("phase success cursor poisoned"))? = cursor;
        Ok(())
    }
    /// Synchronous classification and unadmitted Store turns. Returns whether
    /// a retained action remains pending.
    pub(in crate::runtime) fn reconcile_success(
        &self,
        stopping: &AtomicBool,
    ) -> Result<SuccessSweep> {
        let snapshot = self.rotated_snapshot(&self.success_cursor)?;
        let mut turns = 0usize;
        let mut sweep = SuccessSweep {
            actions: Vec::new(),
            releases: Vec::new(),
            pending: false,
        };
        for (operation, job, _finished) in snapshot {
            if stopping.load(Ordering::SeqCst) {
                return Ok(SuccessSweep {
                    actions: Vec::new(),
                    releases: Vec::new(),
                    pending: false,
                });
            }
            if turns == SUCCESS_TURNS {
                self.set_success_cursor(Some(operation))?;
                sweep.pending = true;
                return Ok(sweep);
            }
            let label = match job.success_turn()? {
                Turn::Skipped => continue,
                Turn::Taken(label) => label,
                Turn::Action(success, kind) => {
                    sweep.actions.push(SuccessAction {
                        job: Arc::downgrade(&job),
                        success,
                        kind,
                    });
                    "admitted action"
                }
                Turn::Release(ack) => {
                    sweep.releases.push(SuccessRelease {
                        job: Arc::downgrade(&job),
                        ack,
                    });
                    "release"
                }
            };
            turns += 1;
            sweep.pending = true;
            #[cfg(test)]
            self.success_turns.lock().unwrap().push((operation, label));
            #[cfg(not(test))]
            let _ = label;
        }
        self.set_success_cursor(None)?;
        Ok(sweep)
    }
}

/// Classification read under one short job lock.
struct Classified {
    binding: Arc<NativePhaseBinding>,
    plan: Option<Arc<ManagedBindingPlan>>,
    state: BindingState,
    ack: Option<Arc<BindingAcknowledgment>>,
    success: Option<Arc<SuccessContinuation>>,
}

impl Job {
    fn classify(&self) -> Result<Option<Classified>> {
        let _depth = RootLockDepth::enter();
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
        let Some(Ok(RetainedStart::Launched { binding, .. })) = &state.outcome else {
            return Ok(None);
        };
        // The start task's one-shot binding is still in flight.
        if state.observation == InvocationObservation::Binding
            || state.closed_ack.is_some()
            || state.success_due > Instant::now()
        {
            return Ok(None);
        }
        Ok(Some(Classified {
            binding: binding.clone(),
            plan: state.binding_plan.clone(),
            state: state.binding_state,
            ack: state.binding_ack.clone(),
            success: state.success.clone(),
        }))
    }
    fn success_turn(&self) -> Result<Turn> {
        let Some(c) = self.classify()? else {
            return Ok(Turn::Skipped);
        };
        if let Some(success) = c.success {
            return self.housekeeping_turn(&success);
        }
        match c.ack {
            Some(ack) => self.settlement_turn(&c.binding, ack),
            None => self.binding_turn(c),
        }
    }
    /// D1: converge the binding of the SAME owner.
    fn binding_turn(&self, c: Classified) -> Result<Turn> {
        let owner = self.allocation.selected_port().owner();
        if let (BindingState::Uncertain, Some(plan)) = (c.state, &c.plan) {
            #[cfg(test)]
            pause_at(self.allocation.facts().scope.task_id, BINDING_CONFIRM);
            let confirmation = owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("binding Store poisoned"))?
                .confirm_managed_binding(plan);
            match confirmation {
                Ok(ManagedBindingConfirmation::Known(ack)) => self.install_ack(plan, ack)?,
                Ok(ManagedBindingConfirmation::RolledBack) => {
                    self.set_binding_state(plan, BindingState::RolledBack)?;
                    self.success_ready()?;
                }
                Err(_cause) => self.success_held("binding outcome uncertain; Held", 5000)?,
            }
            return Ok(Turn::Taken("binding confirm"));
        }
        let snap = Arc::new(c.binding.owner_arc().binding_snapshot()?);
        let retained_late = c.plan.as_ref().is_some_and(|p| p.is_late());
        let plan = if snap.settlement().is_some() {
            match (c.state, retained_late) {
                // Every late image derives from the SAME settlement: a late
                // plan's own refusal cannot be repaired by replanning.
                (BindingState::Conflict, true) => {
                    self.success_held("late binding refused; Held", 5000)?;
                    return Ok(Turn::Taken("late refused"));
                }
                (BindingState::RolledBack, true) => c.plan.clone().expect("retained late plan"),
                _ => match plan_late_binding(owner, snap) {
                    Ok(plan) => {
                        let plan = Arc::new(plan);
                        self.retain_binding_plan(&plan)?;
                        plan
                    }
                    Err(_cause) => {
                        self.success_held("late binding plan refused; Held", 5000)?;
                        return Ok(Turn::Taken("late plan refused"));
                    }
                },
            }
        } else if snap.is_live() {
            match (c.state, &c.plan) {
                (BindingState::Conflict, _) => match plan_managed_binding(owner, snap) {
                    Ok(plan) => {
                        let plan = Arc::new(plan);
                        self.retain_binding_plan(&plan)?;
                        plan
                    }
                    Err(_cause) => {
                        self.success_retry()?;
                        return Ok(Turn::Taken("normal plan refused"));
                    }
                },
                (BindingState::RolledBack, Some(plan)) => plan.clone(),
                _ => return Ok(Turn::Skipped),
            }
        } else {
            // Revoked and unbound without a settlement: no conclusion.
            self.success_held(
                "owned success settlement not observed",
                SETTLEMENT_REPOLL_MS,
            )?;
            return Ok(Turn::Taken("unbound settlement poll"));
        };
        let write = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("binding Store poisoned"))?
            .bind_managed_phase(&plan);
        match self.record_binding_write(&plan, write) {
            Ok(()) => {
                self.set_observation(InvocationObservation::Bound)?;
            }
            Err(_cause) => self.success_retry()?,
        }
        Ok(Turn::Taken(if plan.is_late() {
            "late bind"
        } else {
            "normal bind"
        }))
    }
    /// D2: a bound owner that is no longer live may hold its settlement.
    fn settlement_turn(
        &self,
        binding: &Arc<NativePhaseBinding>,
        ack: Arc<BindingAcknowledgment>,
    ) -> Result<Turn> {
        // One atomic load; a live owner costs no turn.
        if binding.owner().is_live() {
            return Ok(Turn::Skipped);
        }
        let snap = Arc::new(binding.owner_arc().binding_snapshot()?);
        if snap.settlement().is_none() {
            // The revoke->settle gap, a Core drop before its saved terminal
            // settles, or a non-success terminal: never concluded here.
            self.success_held(
                "owned success settlement not observed",
                SETTLEMENT_REPOLL_MS,
            )?;
            return Ok(Turn::Taken("settlement poll"));
        }
        let settled = SettledPhase::issue(snap, ack)?;
        {
            let _depth = RootLockDepth::enter();
            let mut state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
            ensure!(state.success.is_none(), "success continuation installed");
            state.success = Some(Arc::new(SuccessContinuation {
                settled,
                stage: tokio::sync::Mutex::new(SuccessStage::default()),
            }));
            state.success_attention = None;
            state.success_backoff = 100;
        }
        // Hint only; the Driver re-reads retained custody.
        self.changed.send_replace(InvocationObservation::Bound);
        Ok(Turn::Taken("continuation installed"))
    }
    /// D3: confirm a retained uncertain claim or observed write (one Store
    /// turn, no admission). A busy stage is skipped without a turn.
    fn housekeeping_turn(&self, success: &Arc<SuccessContinuation>) -> Result<Turn> {
        let Some(mut stage) = success.try_stage()? else {
            return Ok(Turn::Skipped);
        };
        if let Some(kind) = stage.admitted_action() {
            drop(stage);
            return Ok(Turn::Action(success.clone(), kind));
        }
        if let Some(ClosureState::Known {
            ack,
            published: true,
        }) = stage.closure.as_ref().map(|c| &c.state)
        {
            let ack = ack.clone();
            let retired = stage.sources_retired;
            drop(stage);
            // Install the Driver's Known closure once (pointer-checked).
            let installed = {
                let _depth = RootLockDepth::enter();
                self.state
                    .lock()
                    .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?
                    .success_ack
                    .is_some()
            };
            if !installed {
                self.install_success_ack(success, ack)?;
                return Ok(Turn::Taken("success acknowledged"));
            }
            return Ok(if retired {
                Turn::Release(ack)
            } else {
                Turn::Skipped
            });
        }
        if !stage.has_uncertain() {
            return Ok(Turn::Skipped);
        }
        let owner = self.allocation.selected_port().owner();
        let mut store = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("success Store poisoned"))?;
        let confirmed = if let Some(Retained::Uncertain(plan)) = &stage.claim {
            let plan = plan.clone();
            let (next, confirmed) =
                Retained::from_confirmation(plan.clone(), store.confirm_settled_gate_claim(&plan));
            stage.claim = Some(next);
            confirmed
        } else if let Some(Retained::Uncertain(plan)) = &stage.observed {
            let plan = plan.clone();
            let (next, confirmed) = Retained::from_confirmation(
                plan.clone(),
                store.confirm_settled_gate_observed(&plan),
            );
            if matches!(next, Retained::Known(_)) {
                // Compact retention: the claim and completion are spent.
                stage.claim = None;
                stage.completion = None;
            }
            stage.observed = Some(next);
            confirmed
        } else {
            return Ok(Turn::Skipped);
        };
        drop(store);
        drop(stage);
        if confirmed {
            self.success_ready()?;
            self.changed.send_replace(InvocationObservation::Bound);
        } else {
            self.success_held("settled write outcome uncertain; Held", 5000)?;
        }
        Ok(Turn::Taken("success confirm"))
    }
    /// Installs the Known closure acknowledgment once (pointer-checked) and
    /// sends `ClosedSuccess`.
    pub(crate) fn install_success_ack(
        &self,
        success: &Arc<SuccessContinuation>,
        ack: Arc<SuccessClosureAcknowledgment>,
    ) -> Result<()> {
        {
            let _depth = RootLockDepth::enter();
            let mut state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
            ensure!(
                state
                    .success
                    .as_ref()
                    .is_some_and(|retained| Arc::ptr_eq(retained, success))
                    && Arc::ptr_eq(ack.settled(), success.settled()),
                "success acknowledgment differs from the retained continuation"
            );
            if state.success_ack.is_none() {
                state.success_ack = Some(ack);
            }
            state.observation = InvocationObservation::ClosedSuccess;
        }
        self.changed
            .send_replace(InvocationObservation::ClosedSuccess);
        Ok(())
    }
    fn install_ack(
        &self,
        plan: &Arc<ManagedBindingPlan>,
        ack: Arc<BindingAcknowledgment>,
    ) -> Result<()> {
        {
            let _depth = RootLockDepth::enter();
            let mut state = self
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
            ensure!(
                ack.matches_plan(plan)
                    && state
                        .binding_plan
                        .as_ref()
                        .is_some_and(|retained| Arc::ptr_eq(retained, plan))
                    && state.binding_ack.is_none(),
                "binding confirmation differs from the retained plan"
            );
            state.binding_ack = Some(ack);
            state.binding_state = BindingState::Unwritten;
            state.observation = InvocationObservation::Bound;
        }
        self.changed.send_replace(InvocationObservation::Bound);
        self.success_ready()
    }
    fn set_binding_state(&self, plan: &Arc<ManagedBindingPlan>, next: BindingState) -> Result<()> {
        let _depth = RootLockDepth::enter();
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
        ensure!(
            state
                .binding_plan
                .as_ref()
                .is_some_and(|retained| Arc::ptr_eq(retained, plan)),
            "binding state differs from the retained plan"
        );
        state.binding_state = next;
        Ok(())
    }
    fn set_observation(&self, observation: InvocationObservation) -> Result<()> {
        {
            let _depth = RootLockDepth::enter();
            self.state
                .lock()
                .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?
                .observation = observation;
        }
        self.changed.send_replace(observation);
        Ok(())
    }
    fn success_held(&self, reason: &'static str, delay_ms: u64) -> Result<()> {
        let _depth = RootLockDepth::enter();
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
        state.success_attention = Some(reason);
        state.success_backoff = delay_ms;
        state.success_due = Instant::now() + Duration::from_millis(delay_ms);
        Ok(())
    }
    fn success_retry(&self) -> Result<()> {
        let _depth = RootLockDepth::enter();
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
        state.success_due = Instant::now() + Duration::from_millis(state.success_backoff);
        state.success_backoff = state.success_backoff.saturating_mul(2).min(5000);
        Ok(())
    }
    fn success_ready(&self) -> Result<()> {
        let _depth = RootLockDepth::enter();
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
        state.success_due = Instant::now();
        state.success_backoff = 100;
        state.success_attention = None;
        Ok(())
    }
}

/// Test-only, Task-scoped hold at one named site (the start task between its
/// normal binding plan and write, or a settled gate evaluation after its claim
/// check). It parks and resumes only; it grants nothing.
#[cfg(test)]
pub(crate) struct WritePause {
    state: Mutex<u8>,
    changed: std::sync::Condvar,
}
#[cfg(test)]
type PauseKey = (crate::domain::TaskId, &'static str);
#[cfg(test)]
static WRITE_PAUSES: Mutex<Vec<(PauseKey, Arc<WritePause>)>> = Mutex::new(Vec::new());
#[cfg(test)]
pub(crate) const NORMAL_WRITE: &str = "normal write";
#[cfg(test)]
pub(crate) const SETTLED_EVALUATION: &str = "settled evaluation";
#[cfg(test)]
pub(crate) const SETTLED_CLOSURE: &str = "settled closure";
#[cfg(test)]
pub(crate) const SUCCESS_ADMISSION: &str = "success admission";
#[cfg(test)]
pub(crate) const SETTLEMENT_GAP: &str = "settlement gap";
#[cfg(test)]
pub(crate) const BINDING_CONFIRM: &str = "binding confirm";
#[cfg(test)]
pub(crate) const CLOSURE_CONFIRM: &str = "closure confirm";
#[cfg(test)]
pub(crate) const ROOT_ADMISSION: &str = "root admission";
#[cfg(test)]
pub(crate) const SUCCESS_ADMITTED: &str = "success admitted";
#[cfg(test)]
pub(crate) const OWNER_IMMEDIATE: &str = "owner immediate";
#[cfg(test)]
pub(crate) const OWNER_PLANNING: &str = "owner planning";
/// Test-only async hold of one settled helper capture of a Task: the helper
/// stays in flight while its fence keeps ticking. Parks and resumes only.
#[cfg(test)]
pub(crate) struct HelperHold {
    reached: std::sync::atomic::AtomicBool,
    release: tokio::sync::watch::Sender<bool>,
}
#[cfg(test)]
static HELPER_HOLDS: Mutex<Vec<(crate::domain::TaskId, Arc<HelperHold>)>> = Mutex::new(Vec::new());
#[cfg(test)]
impl HelperHold {
    pub(crate) fn arm(task: crate::domain::TaskId) -> Arc<Self> {
        let hold = Arc::new(Self {
            reached: std::sync::atomic::AtomicBool::new(false),
            release: tokio::sync::watch::channel(false).0,
        });
        HELPER_HOLDS.lock().unwrap().push((task, hold.clone()));
        hold
    }
    pub(crate) fn reached(&self) -> bool {
        self.reached.load(std::sync::atomic::Ordering::SeqCst)
    }
    pub(crate) fn release(&self) {
        self.release.send_replace(true);
    }
}
#[cfg(test)]
pub(crate) async fn hold_helper(task: Option<crate::domain::TaskId>) {
    let Some(task) = task else { return };
    let hold = {
        let mut holds = HELPER_HOLDS.lock().unwrap();
        holds
            .iter()
            .position(|(armed, _)| *armed == task)
            .map(|index| holds.remove(index).1)
    };
    let Some(hold) = hold else { return };
    let mut released = hold.release.subscribe();
    hold.reached
        .store(true, std::sync::atomic::Ordering::SeqCst);
    while !*released.borrow_and_update() {
        if released.changed().await.is_err() {
            return;
        }
    }
}
/// Test-only Task-scoped event counters (marker sites, BR retries).
#[cfg(test)]
static COUNTS: Mutex<Vec<(PauseKey, usize)>> = Mutex::new(Vec::new());
#[cfg(test)]
pub(crate) fn count(task: Option<crate::domain::TaskId>, label: &'static str) {
    let Some(task) = task else { return };
    let mut counts = COUNTS.lock().unwrap();
    match counts.iter_mut().find(|(key, _)| *key == (task, label)) {
        Some((_, n)) => *n += 1,
        None => counts.push(((task, label), 1)),
    }
}
#[cfg(test)]
pub(crate) fn counted(task: crate::domain::TaskId, label: &'static str) -> usize {
    COUNTS
        .lock()
        .unwrap()
        .iter()
        .find(|(key, _)| *key == (task, label))
        .map_or(0, |(_, n)| *n)
}
#[cfg(test)]
impl WritePause {
    pub(crate) fn arm(task: crate::domain::TaskId) -> Arc<Self> {
        Self::arm_at(task, NORMAL_WRITE)
    }
    pub(crate) fn arm_at(task: crate::domain::TaskId, site: &'static str) -> Arc<Self> {
        let pause = Arc::new(Self {
            state: Mutex::new(0),
            changed: std::sync::Condvar::new(),
        });
        WRITE_PAUSES
            .lock()
            .unwrap()
            .push(((task, site), pause.clone()));
        pause
    }
    pub(crate) fn reached(&self) -> bool {
        *self.state.lock().unwrap() == 1
    }
    pub(crate) fn release(&self) {
        *self.state.lock().unwrap() = 2;
        self.changed.notify_all();
    }
}
#[cfg(test)]
pub(super) fn pause_before_normal_write(task: Option<crate::domain::TaskId>) {
    pause_at(task, NORMAL_WRITE);
}
#[cfg(test)]
pub(crate) fn pause_at(task: Option<crate::domain::TaskId>, site: &'static str) {
    let Some(task) = task else { return };
    let pause = {
        let mut pauses = WRITE_PAUSES.lock().unwrap();
        pauses
            .iter()
            .position(|(armed, _)| *armed == (task, site))
            .map(|index| pauses.remove(index).1)
    };
    let Some(pause) = pause else { return };
    *pause.state.lock().unwrap() = 1;
    pause.changed.notify_all();
    let wait = || {
        let mut state = pause.state.lock().unwrap();
        while *state != 2 {
            state = pause.changed.wait(state).unwrap();
        }
    };
    match tokio::runtime::Handle::try_current().map(|h| h.runtime_flavor()) {
        Ok(tokio::runtime::RuntimeFlavor::MultiThread) => tokio::task::block_in_place(wait),
        _ => wait(),
    }
}

/// A closed phase's acknowledgment retained by its job.
pub(crate) enum ClosedPhaseAck {
    NonSuccess(Arc<crate::state::PhaseClosedAcknowledgment>),
    Success(Arc<SuccessClosureAcknowledgment>),
}
impl ClosedPhaseAck {
    pub(crate) fn matches_allocation(&self, allocation: &Arc<NativeAllocation>) -> bool {
        match self {
            Self::NonSuccess(ack) => ack.matches_allocation(allocation),
            Self::Success(ack) => Arc::ptr_eq(ack.settled().allocation(), allocation),
        }
    }
    pub(crate) fn matches_marker(
        &self,
        marker: &Arc<crate::state::managed_binding::OriginalMarker>,
    ) -> bool {
        match self {
            Self::NonSuccess(ack) => ack.matches_marker(marker),
            Self::Success(ack) => std::ptr::eq(ack.settled().marker(), marker.as_ref()),
        }
    }
}
/// Root custody read for the SAME Driver association; nongrant.
pub(crate) enum SettledLookup {
    NoHandoff,
    Pending,
    Held(&'static str),
    Settled(Arc<SuccessContinuation>),
    Closed(ClosedPhaseAck),
}

impl PhaseJobs {
    /// The job whose allocation is pointer-equal to the original handoff's.
    fn settled_lookup(&self, allocation: &Arc<NativeAllocation>) -> Result<SettledLookup> {
        let job = {
            let _depth = RootLockDepth::enter();
            let entries = self
                .entries
                .lock()
                .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
            match entries.get(&allocation.facts().operation_id) {
                Some(entry) if Arc::ptr_eq(&entry.job.allocation, allocation) => entry.job.clone(),
                Some(_) => return Ok(SettledLookup::Held("phase job allocation differs")),
                None => return Ok(SettledLookup::Pending),
            }
        };
        let _depth = RootLockDepth::enter();
        let state = job
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
        if let Some(ack) = &state.closed_ack {
            return Ok(SettledLookup::Closed(ClosedPhaseAck::NonSuccess(
                ack.clone(),
            )));
        }
        // A closed success keeps its continuation: the Driver still releases
        // its Sources slot through the SAME stage after the acknowledgment.
        if let Some(success) = &state.success {
            return Ok(SettledLookup::Settled(success.clone()));
        }
        Ok(match state.success_attention {
            Some(reason) if reason.ends_with("Held") => SettledLookup::Held(reason),
            _ => SettledLookup::Pending,
        })
    }
}

impl crate::runtime::Runtime {
    /// Root custody only, pointer-checked, one lock at a time; no SQL.
    /// Passive status and Goal views never call this.
    pub(crate) fn settled_phase(
        &self,
        task: crate::domain::TaskId,
        association: &crate::runtime::driver::DriverAssociation,
    ) -> Result<SettledLookup> {
        match self.phase_handoffs.settled_origin(task, association)? {
            None => Ok(SettledLookup::NoHandoff),
            Some(allocation) => self.phase_jobs.settled_lookup(&allocation),
        }
    }
}
