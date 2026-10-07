//! Actual start-future/result custody; observations grant no Native authority.
use super::phase_supervisor::{PhaseLaunch, PhaseLaunchParts};
use crate::execution::{
    OperationId,
    native::{
        ManagedSessionRef, NativePhaseBinding, NativePhaseStart, NativePhaseStartError,
        NativePreparationCustody, PreparationConfirm,
    },
    phase::NativeAllocation,
};
use crate::state::managed_binding::{
    BindingAcknowledgment, ManagedBindingPlan, ManagedBindingWrite, plan_managed_binding,
};

mod success;
use anyhow::{Result, ensure};
use std::{
    cell::Cell,
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex, Weak},
    time::{Duration, Instant},
};
pub(in crate::runtime) use success::SuccessSweep;
#[cfg(test)]
pub(crate) use success::{
    BINDING_CONFIRM, CLOSURE_CONFIRM, HelperHold, NORMAL_WRITE, OWNER_IMMEDIATE, OWNER_PLANNING,
    ROOT_ADMISSION, SETTLED_CLOSURE, SETTLED_EVALUATION, SETTLEMENT_GAP, SUCCESS_ADMISSION,
    WritePause, count, counted, hold_helper, pause_at,
};
pub(crate) use success::{
    ClosedPhaseAck, ClosureStage, ClosureState, Retained, SettledLookup, SuccessContinuation,
    SuccessStage,
};
use tokio::{sync::watch, task::JoinHandle};

const MAX_JOBS: usize = 128;
const CLOSURE_TURNS: usize = 8;
const PREPARATION_TURNS: usize = 8;
fn rotate_closure_snapshot<K: Ord, T>(snapshot: &mut [(K, T, bool)], cursor: Option<K>) {
    if let Some(cursor) = cursor {
        let position = snapshot.partition_point(|(id, _, _)| *id < cursor);
        let len = snapshot.len();
        if len > 0 {
            snapshot.rotate_left(position % len);
        }
    }
}
fn take_closure_turn(turns: &mut usize) -> bool {
    if *turns == CLOSURE_TURNS {
        return false;
    }
    *turns += 1;
    true
}
/// Material destruction belongs to the caller, after both normal and poisoned
/// Store guards. This private helper grants no authority to its callback.
fn borrowed_store_turn<T, M, R>(
    store: &Mutex<T>,
    material: M,
    call: impl FnOnce(&mut T, &M) -> Result<R>,
) -> Result<R> {
    assert_nonsuccess_unlocked();
    let result = match store.lock() {
        Ok(mut store) => call(&mut store, &material),
        Err(poisoned) => {
            drop(poisoned);
            Err(anyhow::anyhow!("non-success Store poisoned"))
        }
    };
    drop(material);
    result
}
thread_local! {
    static ROOT_LOCK_DEPTH: Cell<usize> = const { Cell::new(0) };
    static TURN_TRANSACTIONS: Cell<Option<u8>> = const { Cell::new(None) };
}
pub(super) struct RootLockDepth;
impl RootLockDepth {
    pub(super) fn enter() -> Self {
        ROOT_LOCK_DEPTH.with(|d| d.set(d.get() + 1));
        Self
    }
}
impl Drop for RootLockDepth {
    fn drop(&mut self) {
        ROOT_LOCK_DEPTH.with(|d| d.set(d.get() - 1));
    }
}
pub(crate) fn assert_nonsuccess_unlocked() {
    ROOT_LOCK_DEPTH.with(|d| debug_assert_eq!(d.get(), 0, "RN-1 Root lock spans external work"));
}
pub(crate) fn record_nonsuccess_store_attempt() {
    assert_nonsuccess_unlocked();
    TURN_TRANSACTIONS.with(|v| {
        if let Some(count) = v.get() {
            let count = count.saturating_add(1);
            v.set(Some(count));
            debug_assert!(count <= 1, "RN-1 turn runs more than one Store transaction");
        }
    });
}
struct ClosureTurn {
    #[cfg(test)]
    observations: Arc<Mutex<Vec<(OperationId, u8)>>>,
    #[cfg(test)]
    operation: OperationId,
}
impl ClosureTurn {
    fn enter(
        #[cfg(test)] observations: Arc<Mutex<Vec<(OperationId, u8)>>>,
        #[cfg(test)] operation: OperationId,
    ) -> Self {
        TURN_TRANSACTIONS.with(|v| {
            debug_assert!(v.get().is_none());
            v.set(Some(0));
        });
        Self {
            #[cfg(test)]
            observations,
            #[cfg(test)]
            operation,
        }
    }
}
impl Drop for ClosureTurn {
    fn drop(&mut self) {
        TURN_TRANSACTIONS.with(|v| {
            let count = v.replace(None);
            #[cfg(test)]
            if let Some(count) = count {
                let mut observations = self.observations.lock().unwrap();
                assert!(observations.len() < 4096, "test turn observation bound");
                observations.push((self.operation, count));
            }
            if !std::thread::panicking() {
                debug_assert!(count.is_some_and(|n| n <= 1));
            }
        });
    }
}

/// Issued only from this registry's retained outcome and finished start task.
/// The allocation pointer is a witness, never reconstructed from an ID or row.
pub(crate) struct StartEnded {
    allocation: Arc<NativeAllocation>,
}
impl StartEnded {
    pub(crate) fn matches_allocation(&self, allocation: &Arc<NativeAllocation>) -> bool {
        Arc::ptr_eq(&self.allocation, allocation)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InvocationObservation {
    Reserved,
    Starting,
    Binding,
    Bound,
    BindingHeld,
    Waiting,
    Failed,
    Uncertain,
    ClosedNonSuccess,
    ClosedSuccess,
}

/// What the retained binding plan's last write or confirmation established.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BindingState {
    /// No write outcome yet (no plan, or the start task is writing).
    Unwritten,
    /// The write returned Err: only the SAME plan may be confirmed.
    Uncertain,
    /// Confirmation proved the SAME plan rolled back.
    RolledBack,
    /// The write was definitively refused before any write.
    Conflict,
}

struct JobState {
    // EMPTY/nongrant until the SAME actual selected Native start installs its
    // own actor and original plan. Retained BEFORE marker/start/future effects.
    preparation: Arc<NativePreparationCustody>,
    observation: InvocationObservation,
    launch: Option<Arc<PhaseLaunchParts>>,
    outcome: Option<std::result::Result<RetainedStart, NativePhaseStartError>>,
    binding_plan: Option<Arc<ManagedBindingPlan>>,
    binding_error: Option<anyhow::Error>,
    // Outcome of the retained binding plan; Known is `binding_ack`.
    binding_state: BindingState,
    binding_ack: Option<Arc<BindingAcknowledgment>>,
    success: Option<Arc<SuccessContinuation>>,
    success_ack: Option<Arc<crate::state::managed_binding::SuccessClosureAcknowledgment>>,
    success_due: Instant,
    success_backoff: u64,
    success_attention: Option<&'static str>,
    nonsuccess: Option<Arc<crate::state::NativeNonSuccessClosurePlan>>,
    closed_ack: Option<Arc<crate::state::PhaseClosedAcknowledgment>>,
    closure_due: Instant,
    closure_backoff: u64,
    preparation_due: Instant,
    preparation_backoff: u64,
    #[cfg(test)]
    preparation_busy: u64,
    #[cfg(test)]
    preparation_attempts: Vec<(Instant, u64)>,
    uncertain: bool,
    slot_released: bool,
    attention: Option<&'static str>,
}
/// Actual returned objects, never reconstructed from DTOs or registry IDs.
enum RetainedStart {
    Launched {
        _handle: ManagedSessionRef,
        binding: Arc<NativePhaseBinding>,
    },
}
struct Job {
    allocation: Arc<NativeAllocation>,
    state: Mutex<JobState>,
    changed: watch::Sender<InvocationObservation>,
}
struct Entry {
    job: Arc<Job>,
    // Independent registry ownership, never a strong return edge from Job.
    handle: Option<JoinHandle<()>>,
}

/// Produced by the actual reservation, retaining SAME entry identity without
/// a job -> launch -> reservation -> job ownership cycle. Reuse never grants
/// this call permission to remove somebody else's existing reservation.
pub(super) struct PhaseJobReservation {
    job: Weak<Job>,
    allocation: Arc<NativeAllocation>,
    fresh: bool,
}

/// Runtime-owned sibling of PhaseSupervisor. Neither slots nor jobs own it.
#[derive(Default)]
pub(super) struct PhaseJobs {
    entries: Mutex<BTreeMap<OperationId, Entry>>,
    closure_cursor: Mutex<Option<OperationId>>,
    preparation_cursor: Mutex<Option<OperationId>>,
    success_cursor: Mutex<Option<OperationId>>,
    #[cfg(test)]
    success_turns: Arc<Mutex<Vec<(OperationId, &'static str)>>>,
    #[cfg(test)]
    closure_turns: Arc<Mutex<Vec<(OperationId, u8)>>>,
}
#[cfg(test)]
#[derive(Debug)]
pub(super) struct ObservedJob {
    pub unit: crate::execution::UnitId,
    pub finished: bool,
    pub refusal: Option<String>,
    pub attention: Option<&'static str>,
    pub preparation: crate::execution::native::PreparationFacts,
    pub due: Instant,
    pub preparation_due: Instant,
    pub observation: InvocationObservation,
    pub bound: bool,
    pub success: bool,
    pub success_closed: bool,
    pub success_attention: Option<&'static str>,
    pub owner_live: Option<bool>,
    pub settled: Option<bool>,
    pub binding_error: Option<String>,
    pub binding_state: &'static str,
    pub preparation_busy: u64,
    /// (attempt time, applied backoff in ms) per Held confirmation.
    pub preparation_attempts: Vec<(Instant, u64)>,
}

/// Test-only, Task-scoped synchronization point between the preparation
/// sweep's selection and the admission try. It parks and resumes only; it
/// grants, records and constructs nothing.
#[cfg(test)]
type PreparationPark = (
    crate::domain::TaskId,
    tokio::sync::oneshot::Sender<()>,
    tokio::sync::oneshot::Receiver<()>,
);
#[cfg(test)]
static PREPARATION_PARKS: Mutex<Vec<PreparationPark>> = Mutex::new(Vec::new());
#[cfg(test)]
pub(super) fn arm_preparation_park(
    task: crate::domain::TaskId,
) -> (
    tokio::sync::oneshot::Receiver<()>,
    tokio::sync::oneshot::Sender<()>,
) {
    let (reached, reached_rx) = tokio::sync::oneshot::channel();
    let (release, release_rx) = tokio::sync::oneshot::channel();
    PREPARATION_PARKS
        .lock()
        .unwrap()
        .push((task, reached, release_rx));
    (reached_rx, release)
}
#[cfg(test)]
pub(super) async fn park_selected_preparations(items: &[PreparationItem]) {
    for item in items {
        let Some(task) = item.task() else { continue };
        let park = {
            let mut parks = PREPARATION_PARKS.lock().unwrap();
            parks
                .iter()
                .position(|(armed, _, _)| *armed == task)
                .map(|index| parks.remove(index))
        };
        if let Some((_, reached, release)) = park {
            let _ = reached.send(());
            let _ = release.await;
        }
    }
}

/// One selected uncertain preparation commit. Holds no strong Job, Runtime
/// or Store; the outcome is applied only to the SAME job's custody.
pub(super) struct PreparationItem {
    job: Weak<Job>,
    custody: Arc<NativePreparationCustody>,
}
impl PreparationItem {
    #[cfg(test)]
    pub(super) fn task(&self) -> Option<crate::domain::TaskId> {
        self.job.upgrade()?.allocation.facts().scope.task_id
    }
    /// At most one admission try and one confirmation transaction.
    pub(super) fn confirm(self) -> Result<()> {
        let outcome = self.custody.confirm_preparation_nonblocking();
        let Some(job) = self.job.upgrade() else {
            return Ok(());
        };
        let _depth = RootLockDepth::enter();
        let mut state = job
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
        if !Arc::ptr_eq(&state.preparation, &self.custody) {
            return Ok(());
        }
        match outcome {
            // Retained in the custody; the predicate no longer selects it.
            PreparationConfirm::Known => {}
            PreparationConfirm::Held => {
                let backoff = state.preparation_backoff;
                let now = Instant::now();
                state.preparation_due = now + Duration::from_millis(backoff);
                state.preparation_backoff = backoff.saturating_mul(2).min(5000);
                #[cfg(test)]
                state.preparation_attempts.push((now, backoff));
            }
            PreparationConfirm::NotAttempted => {
                #[cfg(test)]
                {
                    state.preparation_busy += 1;
                }
            }
        }
        Ok(())
    }
}

impl PhaseJobs {
    #[cfg(test)]
    pub(super) fn observed_turns(&self) -> Vec<(OperationId, u8)> {
        self.closure_turns.lock().unwrap().clone()
    }
    /// Nongrant observations from existing entries, never constructors.
    #[cfg(test)]
    pub(super) fn observed_jobs(&self) -> Vec<ObservedJob> {
        let entries = self.entries.lock().unwrap();
        entries
            .values()
            .map(|entry| {
                let state = entry.job.state.lock().unwrap();
                ObservedJob {
                    unit: entry.job.allocation.facts().unit_id,
                    finished: entry.handle.as_ref().is_some_and(JoinHandle::is_finished),
                    refusal: state
                        .outcome
                        .as_ref()
                        .and_then(|r| r.as_ref().err())
                        .map(|e| e.error.to_string()),
                    attention: state.attention,
                    preparation: state.preparation.observed_facts(),
                    due: state.closure_due,
                    preparation_due: state.preparation_due,
                    observation: state.observation,
                    bound: state.binding_ack.is_some(),
                    binding_error: state.binding_error.as_ref().map(|e| format!("{e:#}")),
                    binding_state: match state.binding_state {
                        BindingState::Unwritten => "unwritten",
                        BindingState::Uncertain => "uncertain",
                        BindingState::RolledBack => "rolled back",
                        BindingState::Conflict => "conflict",
                    },
                    success: state.success.is_some(),
                    success_closed: state.success_ack.is_some(),
                    success_attention: state.success_attention,
                    owner_live: match &state.outcome {
                        Some(Ok(RetainedStart::Launched { binding, .. })) => {
                            Some(binding.owner().is_live())
                        }
                        _ => None,
                    },
                    settled: match &state.outcome {
                        Some(Ok(RetainedStart::Launched { binding, .. })) => binding
                            .owner_arc()
                            .binding_snapshot()
                            .ok()
                            .map(|s| s.settlement().is_some()),
                        _ => None,
                    },
                    preparation_busy: state.preparation_busy,
                    preparation_attempts: state.preparation_attempts.clone(),
                }
            })
            .collect()
    }
    /// Test-only borrowing of an existing original retained object. A copied
    /// Unit ID can select a reader result, never construct an allocation.
    #[cfg(test)]
    pub(super) fn original_allocation(
        &self,
        unit: crate::execution::UnitId,
    ) -> Result<Arc<NativeAllocation>> {
        self.entries
            .lock()
            .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?
            .values()
            .find(|entry| entry.job.allocation.facts().unit_id == unit)
            .map(|entry| entry.job.allocation.clone())
            .ok_or_else(|| anyhow::anyhow!("original retained allocation absent"))
    }
    fn closure_snapshot(&self) -> Result<Vec<(OperationId, Arc<Job>, bool)>> {
        self.rotated_snapshot(&self.closure_cursor)
    }
    fn rotated_snapshot(
        &self,
        cursor: &Mutex<Option<OperationId>>,
    ) -> Result<Vec<(OperationId, Arc<Job>, bool)>> {
        let mut snapshot = {
            let _depth = RootLockDepth::enter();
            let entries = self
                .entries
                .lock()
                .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
            ensure!(
                entries.len() <= MAX_JOBS,
                "phase closure registry exceeds bound"
            );
            entries
                .iter()
                .filter_map(|(id, entry)| {
                    entry
                        .handle
                        .as_ref()
                        .map(|handle| (*id, entry.job.clone(), handle.is_finished()))
                })
                .collect::<Vec<_>>()
        };
        let cursor = {
            let _depth = RootLockDepth::enter();
            *cursor
                .lock()
                .map_err(|_| anyhow::anyhow!("phase closure cursor poisoned"))?
        };
        rotate_closure_snapshot(&mut snapshot, cursor);
        Ok(snapshot)
    }
    fn set_closure_cursor(&self, cursor: Option<OperationId>) -> Result<()> {
        let _depth = RootLockDepth::enter();
        *self
            .closure_cursor
            .lock()
            .map_err(|_| anyhow::anyhow!("phase closure cursor poisoned"))? = cursor;
        Ok(())
    }
    fn set_preparation_cursor(&self, cursor: Option<OperationId>) -> Result<()> {
        let _depth = RootLockDepth::enter();
        *self
            .preparation_cursor
            .lock()
            .map_err(|_| anyhow::anyhow!("phase preparation cursor poisoned"))? = cursor;
        Ok(())
    }
    /// Selects at most eight ended jobs whose original preparation commit is
    /// uncertain and due. Returns only the job Weak and the SAME custody Arc;
    /// no Root lock, Runtime or Store access outlives this call.
    pub(super) fn reconcile_preparations(
        &self,
        stopping: &AtomicBool,
    ) -> Result<Vec<PreparationItem>> {
        let snapshot = self.rotated_snapshot(&self.preparation_cursor)?;
        let mut items = Vec::new();
        for (operation, job, finished) in snapshot {
            if stopping.load(Ordering::SeqCst) {
                return Ok(Vec::new());
            }
            let (eligible, custody) = {
                let _depth = RootLockDepth::enter();
                let state = job
                    .state
                    .lock()
                    .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
                (
                    state.closed_ack.is_none()
                        && state.preparation_due <= Instant::now()
                        && (state.outcome.as_ref().is_some_and(Result::is_err)
                            || (state.outcome.is_none()
                                && state.observation == InvocationObservation::Uncertain
                                && finished)),
                    state.preparation.clone(),
                )
            };
            if !eligible || !custody.known_commit_due() {
                continue;
            }
            if items.len() == PREPARATION_TURNS {
                self.set_preparation_cursor(Some(operation))?;
                return Ok(items);
            }
            items.push(PreparationItem {
                job: Arc::downgrade(&job),
                custody,
            });
        }
        self.set_preparation_cursor(None)?;
        Ok(items)
    }
    fn retire_closed(&self, job: &Arc<Job>) -> Result<()> {
        {
            let _depth = RootLockDepth::enter();
            let state = job
                .state
                .lock()
                .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
            ensure!(
                (state.closed_ack.is_some() || state.success_ack.is_some()) && state.slot_released,
                "phase closure not acknowledged/released"
            );
        }
        let removed = {
            let _depth = RootLockDepth::enter();
            let mut entries = self
                .entries
                .lock()
                .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
            let id = job.allocation.facts().operation_id;
            let entry = entries
                .get(&id)
                .ok_or_else(|| anyhow::anyhow!("closed phase job absent"))?;
            ensure!(
                Arc::ptr_eq(&entry.job, job)
                    && entry.handle.as_ref().is_some_and(JoinHandle::is_finished),
                "closed phase start not finished or original job differs"
            );
            entries.remove(&id)
        };
        drop(removed);
        Ok(())
    }
    fn release_acknowledged(
        &self,
        phases: &super::phase_supervisor::PhaseSupervisor,
        job: &Arc<Job>,
        ack: &Arc<crate::state::PhaseClosedAcknowledgment>,
    ) -> Result<()> {
        let released = {
            let _depth = RootLockDepth::enter();
            job.state
                .lock()
                .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?
                .slot_released
        };
        if !released {
            phases.retire_closed_marked(&ClosedPhaseAck::NonSuccess(ack.clone()))?;
            let _depth = RootLockDepth::enter();
            job.state
                .lock()
                .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?
                .slot_released = true;
        }
        self.retire_closed(job)
    }
    pub(super) fn reconcile_nonsuccess(
        &self,
        phases: &super::phase_supervisor::PhaseSupervisor,
        stopping: &AtomicBool,
    ) -> Result<bool> {
        use crate::execution::native::{NativeClosureStep, PreparationYield};
        use crate::state::{NativeNonSuccessConfirmation, NativeNonSuccessWrite, Store};
        let snapshot = self.closure_snapshot()?;
        let mut turns = 0usize;
        let mut pending = false;
        for (operation, job, finished) in snapshot {
            if stopping.load(Ordering::SeqCst) {
                return Ok(false);
            }
            let (ack, ended, due, custody, saved, uncertain) = {
                let _depth = RootLockDepth::enter();
                let state = job
                    .state
                    .lock()
                    .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
                (
                    state.closed_ack.clone(),
                    state.outcome.as_ref().is_some_and(Result::is_err)
                        || (state.outcome.is_none()
                            && state.observation == InvocationObservation::Uncertain
                            && finished),
                    state.closure_due <= Instant::now(),
                    state.preparation.clone(),
                    state.nonsuccess.clone(),
                    state.uncertain,
                )
            };
            if let Some(ack) = ack {
                if self.release_acknowledged(phases, &job, &ack).is_err() {
                    pending = true;
                }
                continue;
            }
            if !ended || !due {
                continue;
            }
            if !take_closure_turn(&mut turns) {
                self.set_closure_cursor(Some(operation))?;
                return Ok(true);
            }
            let _turn = ClosureTurn::enter(
                #[cfg(test)]
                self.closure_turns.clone(),
                #[cfg(test)]
                operation,
            );
            let owner = job
                .allocation
                .selected_port()
                .selected_adapter()?
                .owner
                .clone();
            let plan = match saved {
                Some(plan) => plan,
                None => {
                    let witness = StartEnded {
                        allocation: job.allocation.clone(),
                    };
                    let proof = match custody.nonsuccess_step(&witness) {
                        NativeClosureStep::NotEligible => {
                            job.closure_held("not eligible")?;
                            continue;
                        }
                        NativeClosureStep::Held(_cause) => {
                            job.closure_held("original preparation Held")?;
                            continue;
                        }
                        NativeClosureStep::Preparation(step) => {
                            match step {
                                PreparationYield::Closed | PreparationYield::RolledBack => {
                                    job.closure_ready()?;
                                    pending = true;
                                }
                                PreparationYield::Conflict | PreparationYield::Uncertain => {
                                    job.closure_retry()?
                                }
                            }
                            // Even Closed ends the turn: never run RN-1 here.
                            continue;
                        }
                        NativeClosureStep::Proof(proof) => proof,
                    };
                    let plan = match Store::plan_phase_nonsuccess_closure(&owner, proof) {
                        Ok(plan) => plan,
                        Err(_cause) => {
                            job.closure_held("original non-success plan Held")?;
                            continue;
                        }
                    };
                    {
                        let _depth = RootLockDepth::enter();
                        let mut state = job
                            .state
                            .lock()
                            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
                        ensure!(
                            state.nonsuccess.is_none() && state.closed_ack.is_none(),
                            "non-success original plan already retained"
                        );
                        state.nonsuccess = Some(plan.clone());
                    }
                    plan
                }
            };
            let material: crate::state::NativeNonSuccessMaterial =
                match Store::materialize_phase_nonsuccess(&plan) {
                    Ok(material) => material,
                    Err(_cause) => {
                        job.closure_held("SAME non-success material Held")?;
                        continue;
                    }
                };
            // Borrowed turn material outlives the Store guard on every path,
            // including poisoning. Errors never transfer its ownership.
            let result = borrowed_store_turn(&owner.store, material, |store, material| {
                if uncertain {
                    store
                        .confirm_phase_nonsuccess(material)
                        .map(|value| match value {
                            NativeNonSuccessConfirmation::Known(ack) => TurnResult::Known(ack),
                            NativeNonSuccessConfirmation::RolledBack => TurnResult::RolledBack,
                        })
                } else {
                    store
                        .close_phase_nonsuccess(material)
                        .map(|value| match value {
                            NativeNonSuccessWrite::Known(ack) => TurnResult::Known(ack),
                            NativeNonSuccessWrite::Conflict(cause) => TurnResult::Conflict(cause),
                        })
                }
            });
            match result {
                Ok(TurnResult::Known(ack)) => {
                    ensure!(
                        ack.matches_allocation(&job.allocation),
                        "non-success acknowledgment differs from job allocation"
                    );
                    let ack = Arc::new(ack);
                    {
                        let _depth = RootLockDepth::enter();
                        let mut state = job
                            .state
                            .lock()
                            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
                        ensure!(
                            state.closed_ack.is_none(),
                            "non-success acknowledgment already retained"
                        );
                        state.closed_ack = Some(ack.clone());
                        state.observation = InvocationObservation::ClosedNonSuccess;
                    }
                    job.changed
                        .send_replace(InvocationObservation::ClosedNonSuccess);
                    if self.release_acknowledged(phases, &job, &ack).is_err() {
                        pending = true;
                    }
                }
                Ok(TurnResult::Conflict(_cause)) => {
                    let removed = {
                        let _depth = RootLockDepth::enter();
                        let mut state = job
                            .state
                            .lock()
                            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
                        ensure!(
                            state
                                .nonsuccess
                                .as_ref()
                                .is_some_and(|p| Arc::ptr_eq(p, &plan)),
                            "non-success conflict differs from retained plan"
                        );
                        state.uncertain = false;
                        state.nonsuccess.take()
                    };
                    drop(removed);
                    job.closure_held("original currency conflict; Held")?;
                }
                Ok(TurnResult::RolledBack) => {
                    {
                        let _depth = RootLockDepth::enter();
                        let mut state = job
                            .state
                            .lock()
                            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
                        state.uncertain = false;
                    }
                    job.closure_ready()?;
                    pending = true;
                }
                Err(_cause) if uncertain => job.closure_held("SAME uncertain confirmation Held")?,
                Err(_cause) => {
                    {
                        let _depth = RootLockDepth::enter();
                        job.state
                            .lock()
                            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?
                            .uncertain = true;
                    }
                    job.closure_retry()?;
                }
            }
        }
        self.set_closure_cursor(None)?;
        Ok(pending)
    }
    /// Caller already holds actual Runtime admission and publishing capacity.
    /// Called before SQL effects; duplicate IDs cannot substitute an allocation.
    pub(super) fn reserve(
        &self,
        allocation: &Arc<NativeAllocation>,
    ) -> Result<PhaseJobReservation> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
        let operation = allocation.facts().operation_id;
        if let Some(entry) = entries.get(&operation) {
            ensure!(
                Arc::ptr_eq(&entry.job.allocation, allocation),
                "foreign phase job allocation"
            );
            ensure!(
                entry.handle.is_none()
                    && entry
                        .job
                        .changed
                        .borrow()
                        .eq(&InvocationObservation::Reserved),
                "phase job already started"
            );
            return Ok(PhaseJobReservation {
                job: Arc::downgrade(&entry.job),
                allocation: allocation.clone(),
                fresh: false,
            });
        }
        ensure!(entries.len() < MAX_JOBS, "phase job capacity unavailable");
        let (changed, _) = watch::channel(InvocationObservation::Reserved);
        let job = Arc::new(Job {
            allocation: allocation.clone(),
            state: Mutex::new(JobState {
                preparation: NativePreparationCustody::new(allocation.clone()),
                observation: InvocationObservation::Reserved,
                launch: None,
                outcome: None,
                binding_plan: None,
                binding_error: None,
                binding_state: BindingState::Unwritten,
                binding_ack: None,
                success: None,
                success_ack: None,
                success_due: Instant::now(),
                success_backoff: 100,
                success_attention: None,
                nonsuccess: None,
                closed_ack: None,
                closure_due: Instant::now(),
                closure_backoff: 100,
                preparation_due: Instant::now(),
                preparation_backoff: 100,
                #[cfg(test)]
                preparation_busy: 0,
                #[cfg(test)]
                preparation_attempts: Vec::new(),
                uncertain: false,
                slot_released: false,
                attention: None,
            }),
            changed,
        });
        entries.insert(
            operation,
            Entry {
                job: job.clone(),
                handle: None,
            },
        );
        Ok(PhaseJobReservation {
            job: Arc::downgrade(&job),
            allocation: allocation.clone(),
            fresh: true,
        })
    }

    /// No start or launch flag mutation occurs unless the original reservation
    /// is still available. Its actual allocation, not the key, establishes origin.
    pub(super) fn ready(&self, reservation: &PhaseJobReservation) -> Result<()> {
        let allocation = &reservation.allocation;
        let job = reservation
            .job
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("original reserved job ended"))?;
        let entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
        let entry = entries
            .get(&allocation.facts().operation_id)
            .ok_or_else(|| anyhow::anyhow!("phase job not reserved before marker"))?;
        ensure!(
            Arc::ptr_eq(&entry.job, &job)
                && Arc::ptr_eq(&entry.job.allocation, allocation)
                && entry.handle.is_none()
                && entry
                    .job
                    .changed
                    .borrow()
                    .eq(&InvocationObservation::Reserved),
            "original phase job changed"
        );
        Ok(())
    }

    /// Root's SAME original reserved job, never a new cell from readable rows.
    /// Clone the actual retained object before queue/Store admission. Launch
    /// stores only its Weak identity, avoiding custody -> actor -> launch cycles.
    pub(super) fn preparation_custody(
        &self,
        reservation: &PhaseJobReservation,
    ) -> Result<Arc<NativePreparationCustody>> {
        let allocation = &reservation.allocation;
        let job = reservation
            .job
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("original reserved job ended"))?;
        let entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
        let entry = entries
            .get(&allocation.facts().operation_id)
            .ok_or_else(|| anyhow::anyhow!("actual preparation job not reserved"))?;
        ensure!(
            Arc::ptr_eq(&entry.job, &job)
                && Arc::ptr_eq(&entry.job.allocation, allocation)
                && entry.handle.is_none(),
            "original preparation job changed"
        );
        let state = entry
            .job
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
        ensure!(
            state.observation == InvocationObservation::Reserved
                && state.preparation.matches_allocation(allocation),
            "original preparation custody differs"
        );
        Ok(state.preparation.clone())
    }

    /// Runtime admission serializes this with ready/rollback. After a known
    /// handoff, preserve custody even if a mutex was poisoned: recovery here
    /// only stores actual objects and never authorizes a protected Store write.
    pub(super) fn start(&self, launch: PhaseLaunch) {
        let parts = launch.parts().clone();
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        let entry = entries
            .get_mut(&parts.allocation().facts().operation_id)
            .expect("actual phase job reserved before marker handoff");
        let job = entry.job.clone();
        let preparation = {
            let mut state = job.state.lock().unwrap_or_else(|e| e.into_inner());
            state.launch = Some(parts);
            state.observation = InvocationObservation::Starting;
            state.preparation.clone()
        };
        job.changed.send_replace(InvocationObservation::Starting);
        // Capture an already-constructed guard: an unpolled future can be
        // destroyed when its Tokio executor stops, before its body ever runs.
        let running = RunningJob(job.clone());
        entry.handle = Some(tokio::spawn(async move {
            let _running = running;
            let mut parked = preparation.parked_updates();
            let start = job
                .allocation
                .selected_port()
                .start_phase(launch, preparation.clone());
            tokio::pin!(start);
            let result = loop {
                tokio::select! {
                    result=&mut start=>break result,
                    changed=parked.changed()=>{
                        if changed.is_err() { continue; }
                        let is_parked=*parked.borrow_and_update();
                        let observation=if preparation.quota_attention() {InvocationObservation::BindingHeld} else if is_parked {InvocationObservation::Waiting} else {InvocationObservation::Starting};
                        { let mut state=job.state.lock().unwrap_or_else(|e|e.into_inner()); if state.outcome.is_none() { state.observation=observation; } }
                        job.changed.send_replace(observation);
                    },
                }
            };
            let outcome = result.map(|start| match start {
                NativePhaseStart::Launched { handle, binding } => RetainedStart::Launched {
                    _handle: handle,
                    binding: Arc::from(binding),
                },
            });
            let refused = outcome.is_err();
            let (observation, binding) = match &outcome {
                Ok(RetainedStart::Launched { binding, .. }) => {
                    (InvocationObservation::Binding, Some(binding.clone()))
                }
                Err(_) => (InvocationObservation::Failed, None),
            };
            {
                let mut state = job.state.lock().unwrap_or_else(|e| e.into_inner());
                state.outcome = Some(outcome);
                state.observation = observation;
            }
            if refused {
                // Actual outcome is already retained. Nongrant abandonment must
                // not hold the Root job mutex or retire Unit/Task from an error.
                preparation.abandon();
            }
            job.changed.send_replace(observation);
            if let Some(binding) = binding {
                // Both the actual proof and handle are already retained. No
                // fallible planning or Store access can consume their sole owner.
                let result = job.bind_returned(binding);
                let observation = if result.is_ok() {
                    InvocationObservation::Bound
                } else {
                    InvocationObservation::BindingHeld
                };
                {
                    let mut state = job.state.lock().unwrap_or_else(|e| e.into_inner());
                    state.binding_error = result.err();
                    state.observation = observation;
                }
                job.changed.send_replace(observation);
            }
        }));
    }

    /// Only a caller holding the genuine unpublished rollback uses this port.
    pub(super) fn remove_unstarted(&self, reservation: &PhaseJobReservation) -> Result<bool> {
        if !reservation.fresh {
            return Ok(false);
        }
        let allocation = &reservation.allocation;
        let removed = {
            let mut entries = self
                .entries
                .lock()
                .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
            let operation = allocation.facts().operation_id;
            let job = reservation
                .job
                .upgrade()
                .ok_or_else(|| anyhow::anyhow!("original reserved job ended"))?;
            let entry = entries
                .get(&operation)
                .ok_or_else(|| anyhow::anyhow!("original reserved job removed"))?;
            ensure!(
                Arc::ptr_eq(&entry.job, &job)
                    && Arc::ptr_eq(&entry.job.allocation, allocation)
                    && entry.handle.is_none()
                    && entry
                        .job
                        .changed
                        .borrow()
                        .eq(&InvocationObservation::Reserved),
                "started job cannot roll back"
            );
            entries.remove(&operation)
        };
        drop(removed);
        Ok(true)
    }

    pub(super) fn ensure_shutdown_complete(&self) -> Result<()> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
        ensure!(
            entries.is_empty(),
            "Native phase shutdown remains pending with retained jobs"
        );
        Ok(())
    }
}

impl Job {
    fn closure_held(&self, reason: &'static str) -> Result<()> {
        debug_assert!(reason.len() <= 128);
        let _depth = RootLockDepth::enter();
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
        state.attention = Some(reason);
        state.closure_due = Instant::now() + Duration::from_secs(5);
        Ok(())
    }
    fn closure_retry(&self) -> Result<()> {
        let _depth = RootLockDepth::enter();
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
        state.closure_due = Instant::now() + Duration::from_millis(state.closure_backoff);
        state.closure_backoff = state.closure_backoff.saturating_mul(2).min(5000);
        Ok(())
    }
    fn closure_ready(&self) -> Result<()> {
        let _depth = RootLockDepth::enter();
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
        state.closure_due = Instant::now();
        state.closure_backoff = 100;
        state.attention = None;
        Ok(())
    }
    fn bind_returned(&self, proof: Arc<NativePhaseBinding>) -> Result<()> {
        let owner = self.allocation.selected_port().owner();
        let plan = Arc::new(plan_managed_binding(owner, proof)?);
        self.retain_binding_plan(&plan)?;
        #[cfg(test)]
        success::pause_before_normal_write(self.allocation.facts().scope.task_id);
        // Short job locks above never overlap the selected owner's Store lock.
        let write = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("binding Store poisoned"))?
            .bind_managed_phase(&plan);
        self.record_binding_write(&plan, write)
    }
    /// A retained plan is replaced only after it was RolledBack or
    /// definitively refused; never two plans for one job.
    fn retain_binding_plan(&self, plan: &Arc<ManagedBindingPlan>) -> Result<()> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        ensure!(
            state.binding_ack.is_none()
                && (state.binding_plan.is_none()
                    || matches!(
                        state.binding_state,
                        BindingState::RolledBack | BindingState::Conflict
                    )),
            "binding plan already retained"
        );
        state.binding_plan = Some(plan.clone());
        state.binding_state = BindingState::Unwritten;
        Ok(())
    }
    /// Records Known, typed Conflict or uncertain for the SAME retained plan.
    fn record_binding_write(
        &self,
        plan: &Arc<ManagedBindingPlan>,
        write: Result<ManagedBindingWrite>,
    ) -> Result<()> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        ensure!(
            state
                .binding_plan
                .as_ref()
                .is_some_and(|retained| Arc::ptr_eq(retained, plan)),
            "binding outcome differs from the retained plan"
        );
        match write {
            Ok(ManagedBindingWrite::Known(ack)) => {
                ensure!(ack.matches_plan(plan), "binding acknowledgment differs");
                state.binding_ack = Some(ack);
                state.binding_state = BindingState::Unwritten;
                Ok(())
            }
            Ok(ManagedBindingWrite::Conflict(cause)) => {
                state.binding_state = BindingState::Conflict;
                Err(cause.context("binding definitively refused"))
            }
            Err(cause) => {
                state.binding_state = BindingState::Uncertain;
                Err(cause.context("binding outcome uncertain"))
            }
        }
    }
}

enum TurnResult {
    Known(crate::state::PhaseClosedAcknowledgment),
    Conflict(anyhow::Error),
    RolledBack,
}

struct RunningJob(Arc<Job>);
impl Drop for RunningJob {
    fn drop(&mut self) {
        let preparation = {
            let mut state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
            if state.outcome.is_none() || state.observation == InvocationObservation::Binding {
                state.observation = InvocationObservation::Uncertain;
                self.0
                    .changed
                    .send_replace(InvocationObservation::Uncertain);
                Some(state.preparation.clone())
            } else {
                None
            }
        };
        if let Some(preparation) = preparation {
            preparation.abandon();
        }
    }
}

#[cfg(test)]
mod nonsuccess_primitives {
    use super::*;
    // Production scheduling/material helpers only: no Job, actor or proof.
    #[test]
    fn nongrant_rn1_due_rotation_delivers_every_job_within_finite_passes() {
        for n in [1usize, 5, 8, 9, 17, 128] {
            let mut delivered = vec![0usize; n];
            let mut cursor = None;
            for _ in 0..n.div_ceil(CLOSURE_TURNS) {
                let mut rows = (0..n).map(|id| (id, (), true)).collect::<Vec<_>>();
                rotate_closure_snapshot(&mut rows, cursor);
                let mut turns = 0;
                cursor = None;
                for (id, _, _) in rows {
                    if !take_closure_turn(&mut turns) {
                        cursor = Some(id);
                        break;
                    }
                    delivered[id] += 1;
                }
                assert!(turns <= 8);
            }
            assert!(
                delivered.iter().all(|count| *count > 0),
                "starved due job among {n}: {delivered:?}"
            );
        }
    }
    #[test]
    fn nongrant_rn1_material_drops_after_store_on_result_and_poison_paths() {
        struct Material {
            store: Arc<Mutex<()>>,
            dropped: Arc<AtomicBool>,
        }
        impl Drop for Material {
            fn drop(&mut self) {
                match self.store.try_lock() {
                    Ok(guard) => drop(guard),
                    Err(std::sync::TryLockError::Poisoned(error)) => drop(error),
                    Err(std::sync::TryLockError::WouldBlock) => {
                        panic!("Material dropped while Store held")
                    }
                }
                self.dropped.store(true, Ordering::SeqCst);
            }
        }
        for outcome in [0, 1, 2] {
            let store = Arc::new(Mutex::new(()));
            let dropped = Arc::new(AtomicBool::new(false));
            let material = Material {
                store: store.clone(),
                dropped: dropped.clone(),
            };
            if outcome == 2 {
                let poisoned = store.clone();
                let _ = std::thread::spawn(move || {
                    let _guard = poisoned.lock().unwrap();
                    panic!("intentional local mutex poison");
                })
                .join();
            }
            let result = borrowed_store_turn(&store, material, |_, _| {
                if outcome == 0 {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("nongrant error"))
                }
            });
            assert_eq!(result.is_ok(), outcome == 0);
            assert!(dropped.load(Ordering::SeqCst));
        }
    }
    #[test]
    fn nongrant_rn1_debug_rejects_root_lock_and_second_transaction() {
        let held = std::panic::catch_unwind(|| {
            let _depth = RootLockDepth::enter();
            assert_nonsuccess_unlocked();
        });
        assert_eq!(held.is_err(), cfg!(debug_assertions));
        let second = std::panic::catch_unwind(|| {
            let _turn = ClosureTurn::enter(Arc::default(), OperationId::new());
            record_nonsuccess_store_attempt();
            record_nonsuccess_store_attempt();
        });
        assert_eq!(second.is_err(), cfg!(debug_assertions));
    }
}
