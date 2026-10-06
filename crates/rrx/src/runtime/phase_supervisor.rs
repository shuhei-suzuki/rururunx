//! Real Runtime-owned allocation retention and private marker publication.
//! Installation remains unavailable until the actual Native/binder composition
//! is complete; retention alone creates no Native or input-consumption proof.
use crate::{
    domain::ProjectId,
    execution::{OperationId, RuntimeOwner, owner::PreparationGuard, phase::NativeAllocation},
};
use anyhow::{Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

const MAX_PENDING: usize = 128;
const PAGE: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PendingObservation {
    PendingMarker,
    MarkerPublicationPending,
    HeldOwnerChanged,
    Stopping,
}
struct Slot {
    allocation: Arc<NativeAllocation>,
    // Constructed only by actual Source reserve, before finish_accepted. This
    // never becomes false, including after an original unpublished proof.
    accepted_source: bool,
    origin: Mutex<Option<Arc<super::phase_handoffs::PhasePreparationOrigin>>>,
    preparation: Mutex<Option<PreparationGuard>>,
    observation: Mutex<PendingObservation>,
    publication: Mutex<PublicationState>,
    publication_plan: Mutex<Option<Arc<crate::state::managed_binding::MarkerPublicationPlan>>>,
    marker: Mutex<Option<Arc<crate::state::managed_binding::OriginalMarker>>>,
    launch_handed_off: AtomicBool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum PublicationState {
    Unmarked,
    Publishing,
}
#[derive(Default)]
struct Queue {
    entries: BTreeMap<OperationId, Arc<Slot>>,
    projects: BTreeMap<ProjectId, VecDeque<OperationId>>,
    rotation: VecDeque<ProjectId>,
}

/// Private actual reservation, not Clone/Deserialize or constructed from IDs.
/// The Runtime retains the slot independently; caller Drop does not retire it.
pub(crate) struct PendingPhaseCapacity {
    supervisor: Weak<PhaseSupervisor>,
    slot: Arc<Slot>,
}
/// The same real slot retained across marker publication and commit uncertainty.
/// This is NOT OriginalMarker or permission to register/start a Native process.
/// Drop leaves the publishing slot charged; it cannot silently abandon it.
pub(crate) struct MarkerPublicationRetention {
    supervisor: Arc<PhaseSupervisor>,
    capacity: Arc<PendingPhaseCapacity>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublicationObservation {
    Planned,
    Reserved,
    Publishing,
    KnownMarker,
    HandedOff,
    RestoredHeld,
    Held,
    RollbackHeld,
}
struct PublicationAssets {
    reservation: Option<Arc<super::phase_jobs::PhaseJobReservation>>,
    retention: Option<Arc<MarkerPublicationRetention>>,
    observation: PublicationObservation,
    publication_error: Option<anyhow::Error>,
    rollback_error: Option<anyhow::Error>,
}
/// Original Handoff-owned publication siblings. No strong Runtime, registry,
/// dispatcher or handle; the owning slot never holds this cell strongly.
pub(super) struct OriginalPublicationCustody {
    capacity: Arc<PendingPhaseCapacity>,
    origin: Arc<super::phase_handoffs::PhasePreparationOrigin>,
    plan: Arc<crate::state::managed_binding::MarkerPublicationPlan>,
    outcome: Arc<crate::state::managed_binding::MarkerPublicationOutcome>,
    attempted: AtomicBool,
    assets: Mutex<PublicationAssets>,
}
impl OriginalPublicationCustody {
    pub(super) fn new(
        capacity: PendingPhaseCapacity,
        origin: Arc<super::phase_handoffs::PhasePreparationOrigin>,
        plan: Arc<crate::state::managed_binding::MarkerPublicationPlan>,
    ) -> Arc<Self> {
        let outcome = crate::state::managed_binding::MarkerPublicationOutcome::new(plan.clone());
        Arc::new(Self {
            capacity: Arc::new(capacity),
            origin,
            plan,
            outcome,
            attempted: AtomicBool::new(false),
            assets: Mutex::new(PublicationAssets {
                reservation: None,
                retention: None,
                observation: PublicationObservation::Planned,
                publication_error: None,
                rollback_error: None,
            }),
        })
    }
    fn observation(&self, observation: PublicationObservation) {
        self.assets
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .observation = observation;
    }
    fn retain_error(&self, error: anyhow::Error, rollback: bool) -> anyhow::Error {
        let diagnostic = anyhow::anyhow!("{error:#}");
        let mut assets = self.assets.lock().unwrap_or_else(|e| e.into_inner());
        if rollback {
            assets.observation = PublicationObservation::RollbackHeld;
            assets.rollback_error = Some(error);
        } else {
            assets.observation = PublicationObservation::Held;
            assets.publication_error = Some(error);
        }
        diagnostic
    }
    pub(super) fn may_attempt_unpublished(&self) -> Result<bool> {
        use crate::state::managed_binding::MarkerTransactionObservation as Tx;
        Ok(matches!(
            self.outcome.transaction()?,
            Tx::NotConstructedRefused | Tx::PrecommitDropped
        ) && !self.outcome.is_confirmed()?
            && self.outcome.first_marker()?.is_none())
    }
}
/// Historical no-plan guard control, isolated from actual publication/start.
#[cfg(test)]
pub(crate) struct LegacyMarkerPublicationRetention {
    retention: MarkerPublicationRetention,
    reservation: super::phase_jobs::PhaseJobReservation,
}
#[cfg(test)]
impl LegacyMarkerPublicationRetention {
    pub(crate) fn is_retained(&self) -> bool {
        self.retention.is_retained()
    }
}
/// One non-Clone handoff from the actual known-commit Runtime producer.
pub(crate) struct PhaseLaunch {
    parts: Arc<PhaseLaunchParts>,
}
/// The Core may share this SAME object internally, but cannot remint a launch
/// from a ManagedInput, IDs, SQL rows or another adapter's returned Session.
pub(crate) struct PhaseLaunchParts {
    marker: Arc<crate::state::managed_binding::OriginalMarker>,
    retention: Arc<MarkerPublicationRetention>,
    origin: Arc<super::phase_handoffs::PhasePreparationOrigin>,
    preparation: Weak<crate::execution::native::NativePreparationCustody>,
}
impl PhaseLaunch {
    pub(super) fn parts(&self) -> &Arc<PhaseLaunchParts> {
        &self.parts
    }
    pub(crate) fn into_parts(self) -> Arc<PhaseLaunchParts> {
        self.parts
    }
}
impl PhaseLaunchParts {
    /// SAME independently retained actual Root job cell, never strong here:
    /// custody owns actor, actor owns launch. A strong return edge would leak it.
    pub(crate) fn preparation_custody(
        &self,
    ) -> &Weak<crate::execution::native::NativePreparationCustody> {
        &self.preparation
    }
    /// Original Source lineage plus actual current/Driver currency. Still no
    /// Native permission: its writer must validate admission and exact stage,
    /// Unit/input/pair/lifecycle and compiled mutations in this SAME transaction.
    pub(crate) fn validate_preparation_origin_tx(
        &self,
        tx: &rusqlite::Transaction<'_>,
        current: &crate::state::managed_binding::CurrentWorkflowSuccessor,
    ) -> Result<()> {
        self.validate_preparation_original()?;
        crate::state::managed_binding::validate_current_tx(tx, &self.marker, current)?;
        self.marker.validate_driver_live_tx(tx)
    }
    /// Immutable SAME Source/ticket/allocation linkage only. Nongrant closure
    /// uses this after normal currency revocation; it issues no effect/input
    /// permission and cannot replace the retained Source ticket from rows.
    pub(crate) fn validate_preparation_original(&self) -> Result<()> {
        self.origin.validate_marker(&self.marker)
    }
    pub(crate) fn admission(&self) -> &Arc<super::phase_effect_admission::PhaseEffectAdmission> {
        &self.retention.supervisor.admission
    }
    pub(crate) fn marker(&self) -> &Arc<crate::state::managed_binding::OriginalMarker> {
        &self.marker
    }
    pub(crate) fn allocation(&self) -> &Arc<NativeAllocation> {
        self.marker.allocation()
    }
    pub(crate) fn is_retained(&self) -> bool {
        self.retention.is_retained()
    }
}
impl MarkerPublicationRetention {
    pub(crate) fn allocation(&self) -> &NativeAllocation {
        self.capacity.allocation()
    }
    pub(crate) fn is_retained(&self) -> bool {
        self.supervisor.contains(&self.capacity.slot)
    }
}
pub(crate) struct PendingReservationError {
    pub(crate) reason: &'static str,
    pub(crate) allocation: NativeAllocation,
    pub(crate) preparation: PreparationGuard,
}
impl std::fmt::Debug for PendingReservationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingReservationError")
            .field("reason", &self.reason)
            .finish_non_exhaustive()
    }
}
impl PendingPhaseCapacity {
    pub(super) fn allocation_arc(&self) -> &Arc<NativeAllocation> {
        &self.slot.allocation
    }
    fn preparation_origin(&self) -> Result<Arc<super::phase_handoffs::PhasePreparationOrigin>> {
        let origin = self
            .slot
            .origin
            .lock()
            .map_err(|_| anyhow::anyhow!("Source preparation origin poisoned"))?
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("actual accepted Source preparation origin missing"))?;
        ensure!(
            origin.matches_allocation(&self.slot.allocation),
            "Source origin allocation differs"
        );
        Ok(origin)
    }
    /// Borrowing the immutable real allocation never holds the queue mutex.
    pub(crate) fn allocation(&self) -> &NativeAllocation {
        &self.slot.allocation
    }
    pub(crate) fn observation(&self) -> Result<PendingObservation> {
        Ok(*self
            .slot
            .observation
            .lock()
            .map_err(|_| anyhow::anyhow!("pending observation poisoned"))?)
    }
    /// Checks actual retained membership only, never Driver/native currency.
    pub(crate) fn is_retained(&self) -> bool {
        self.supervisor
            .upgrade()
            .is_some_and(|s| s.contains(&self.slot))
    }
    /// Publication and marked operations cannot use this removal path.
    pub(crate) fn abandon_unmarked(self) -> Result<()> {
        let supervisor = self
            .supervisor
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("pending supervisor ended"))?;
        supervisor.remove_unmarked(&self.slot)
    }
}

pub(crate) struct PhaseSupervisor {
    owner: Arc<RuntimeOwner>,
    admission: Arc<super::phase_effect_admission::PhaseEffectAdmission>,
    global: usize,
    per_project: usize,
    closed: AtomicBool,
    queue: Mutex<Queue>,
}
impl PhaseSupervisor {
    pub(crate) fn belongs_to(&self, owner: &Arc<RuntimeOwner>) -> bool {
        Arc::ptr_eq(&self.owner, owner)
    }
    pub(super) fn new(
        owner: Arc<RuntimeOwner>,
        global: usize,
        per_project: usize,
        admission: Arc<super::phase_effect_admission::PhaseEffectAdmission>,
    ) -> Arc<Self> {
        Arc::new(Self {
            owner,
            admission,
            global: global.min(MAX_PENDING),
            per_project: per_project.min(MAX_PENDING),
            closed: AtomicBool::new(false),
            queue: Mutex::new(Queue::default()),
        })
    }
    pub(super) fn reserve(
        self: &Arc<Self>,
        allocation: NativeAllocation,
        preparation: PreparationGuard,
    ) -> std::result::Result<PendingPhaseCapacity, Box<PendingReservationError>> {
        self.reserve_with_policy(allocation, preparation, false)
    }
    /// Only the concrete Source transfer calls this path. Set SAME guard and
    /// slot policy while constructing the successful reservation, so later
    /// origin installation failure cannot leave accepted custody drainable.
    pub(super) fn reserve_source(
        self: &Arc<Self>,
        allocation: NativeAllocation,
        preparation: PreparationGuard,
    ) -> std::result::Result<PendingPhaseCapacity, Box<PendingReservationError>> {
        self.reserve_with_policy(allocation, preparation, true)
    }
    fn reserve_with_policy(
        self: &Arc<Self>,
        allocation: NativeAllocation,
        mut preparation: PreparationGuard,
        accepted_source: bool,
    ) -> std::result::Result<PendingPhaseCapacity, Box<PendingReservationError>> {
        let checked = (|| -> Result<()> {
            let f = allocation.facts();
            ensure!(
                f.state_path == self.owner.state_path()
                    && f.state_root == self.owner.state_root()
                    && f.instance_id == self.owner.instance_id()
                    && f.epoch == self.owner.epoch(),
                "foreign pending owner"
            );
            ensure!(
                preparation.matches(&self.owner, allocation.unit_snapshot())?,
                "foreign pending preparation"
            );
            let current = crate::execution::phase::allocation_snapshot(
                &self.owner,
                &allocation.unit_snapshot().authority(),
            )?;
            ensure!(
                serde_json::to_value(current)? == serde_json::to_value(allocation.unit_snapshot())?,
                "pending original Unit changed"
            );
            Ok(())
        })();
        if checked.is_err() {
            return Err(Box::new(PendingReservationError {
                reason: "pending allocation/preparation identity unavailable",
                allocation,
                preparation,
            }));
        }
        // All full reads/encoding above finish before the queue mutex is acquired.
        let mut queue = match self.queue.lock() {
            Ok(q) => q,
            Err(_) => {
                return Err(Box::new(PendingReservationError {
                    reason: "pending queue poisoned",
                    allocation,
                    preparation,
                }));
            }
        };
        let facts = allocation.facts();
        let project = facts.scope.project_id;
        let id = facts.operation_id;
        let unavailable = self.closed.load(Ordering::SeqCst)
            || queue.entries.len() >= self.global
            || queue
                .projects
                .get(&project)
                .is_some_and(|v| v.len() >= self.per_project)
            || queue.entries.contains_key(&id)
            || queue
                .entries
                .values()
                .any(|v| v.allocation.facts().unit_id == facts.unit_id);
        if unavailable {
            drop(queue);
            return Err(Box::new(PendingReservationError {
                reason: "pending capacity unavailable",
                allocation,
                preparation,
            }));
        }
        if accepted_source {
            preparation.hold_accepted_source();
        }
        let slot = Arc::new(Slot {
            allocation: Arc::new(allocation),
            accepted_source,
            origin: Mutex::new(None),
            preparation: Mutex::new(Some(preparation)),
            observation: Mutex::new(PendingObservation::PendingMarker),
            publication: Mutex::new(PublicationState::Unmarked),
            publication_plan: Mutex::new(None),
            marker: Mutex::new(None),
            launch_handed_off: AtomicBool::new(false),
        });
        if !queue.projects.contains_key(&project) {
            queue.rotation.push_back(project);
        }
        queue.projects.entry(project).or_default().push_back(id);
        queue.entries.insert(id, slot.clone());
        Ok(PendingPhaseCapacity {
            supervisor: Arc::downgrade(self),
            slot,
        })
    }
    /// Only the real Source transfer can create this private sealed origin.
    /// Exact allocation Arc and retained unmarked slot, never scalar IDs, are
    /// required. No encoding, Source/Store mutex, guard Drop or await here.
    pub(super) fn retain_preparation_origin(
        &self,
        allocation: &Arc<NativeAllocation>,
        origin: Arc<super::phase_handoffs::PhasePreparationOrigin>,
    ) -> Result<()> {
        ensure!(
            origin.matches_allocation(allocation),
            "foreign Source origin"
        );
        let queue = self
            .queue
            .lock()
            .map_err(|_| anyhow::anyhow!("pending queue poisoned"))?;
        let slot = queue
            .entries
            .get(&allocation.facts().operation_id)
            .ok_or_else(|| anyhow::anyhow!("original Source capacity no longer retained"))?;
        ensure!(
            !self.closed.load(Ordering::SeqCst) && Arc::ptr_eq(&slot.allocation, allocation),
            "Source origin capacity changed"
        );
        let publication = slot
            .publication
            .lock()
            .map_err(|_| anyhow::anyhow!("publication state poisoned"))?;
        ensure!(
            *publication == PublicationState::Unmarked,
            "Source origin cannot replace publishing provenance"
        );
        let mut saved = slot
            .origin
            .lock()
            .map_err(|_| anyhow::anyhow!("Source preparation origin poisoned"))?;
        ensure!(
            saved.is_none(),
            "Source preparation origin already installed"
        );
        *saved = Some(origin);
        Ok(())
    }
    fn contains(&self, slot: &Arc<Slot>) -> bool {
        self.queue.lock().is_ok_and(|q| {
            q.entries
                .get(&slot.allocation.facts().operation_id)
                .is_some_and(|current| Arc::ptr_eq(current, slot))
        })
    }
    fn begin_publication(
        self: &Arc<Self>,
        capacity: Arc<PendingPhaseCapacity>,
    ) -> Result<MarkerPublicationRetention> {
        ensure!(
            capacity
                .supervisor
                .upgrade()
                .is_some_and(|s| Arc::ptr_eq(&s, self)),
            "foreign publication supervisor"
        );
        let q = self
            .queue
            .lock()
            .map_err(|_| anyhow::anyhow!("pending queue poisoned"))?;
        ensure!(
            !self.closed.load(Ordering::SeqCst)
                && q.entries
                    .get(&capacity.allocation().facts().operation_id)
                    .is_some_and(|s| Arc::ptr_eq(s, &capacity.slot)),
            "pending publication capacity unavailable"
        );
        // Queue -> publication -> preparation is the only mutation lock order.
        // These locks never span SQL, hashing, guard Drop or an async boundary.
        let mut state = capacity
            .slot
            .publication
            .lock()
            .map_err(|_| anyhow::anyhow!("publication state poisoned"))?;
        ensure!(
            *state == PublicationState::Unmarked,
            "marker publication already begun"
        );
        let mut preparation = capacity
            .slot
            .preparation
            .lock()
            .map_err(|_| anyhow::anyhow!("pending preparation poisoned"))?;
        preparation
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("pending preparation absent"))?
            .hold_marker_publication();
        *state = PublicationState::Publishing;
        drop(preparation);
        drop(state);
        drop(q);
        Ok(MarkerPublicationRetention {
            supervisor: self.clone(),
            capacity,
        })
    }
    fn restore_unpublished(
        &self,
        slot: &Arc<Slot>,
        proof: crate::state::managed_binding::UnpublishedMarkerProof,
    ) -> Result<()> {
        ensure!(
            proof.matches(&self.owner, &slot.allocation),
            "foreign rollback proof"
        );
        let q = self
            .queue
            .lock()
            .map_err(|_| anyhow::anyhow!("pending queue poisoned"))?;
        ensure!(
            q.entries
                .get(&slot.allocation.facts().operation_id)
                .is_some_and(|s| Arc::ptr_eq(s, slot)),
            "publication slot no longer retained"
        );
        let mut state = slot
            .publication
            .lock()
            .map_err(|_| anyhow::anyhow!("publication state poisoned"))?;
        ensure!(
            *state == PublicationState::Publishing,
            "slot is not publishing"
        );
        let mut preparation = slot
            .preparation
            .lock()
            .map_err(|_| anyhow::anyhow!("pending preparation poisoned"))?;
        // The genuine absence proof above excludes a committed marker. Never
        // clear a saved OriginalMarker merely because an error/drop occurred.
        let saved_marker = slot
            .marker
            .lock()
            .map_err(|_| anyhow::anyhow!("saved marker poisoned"))?;
        ensure!(
            saved_marker.is_none(),
            "known marker cannot be restored as unpublished"
        );
        let mut saved_plan = slot
            .publication_plan
            .lock()
            .map_err(|_| anyhow::anyhow!("publication plan poisoned"))?;
        preparation
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("pending preparation absent"))?
            .restore_unmarked_retirement();
        let removed_plan = saved_plan.take();
        *state = PublicationState::Unmarked;
        drop(saved_plan);
        drop(saved_marker);
        drop(preparation);
        drop(state);
        drop(q);
        drop(removed_plan);
        Ok(())
    }
    fn remove_unmarked(&self, slot: &Arc<Slot>) -> Result<()> {
        let removed = {
            let mut q = self
                .queue
                .lock()
                .map_err(|_| anyhow::anyhow!("pending queue poisoned"))?;
            let f = slot.allocation.facts();
            ensure!(
                q.entries
                    .get(&f.operation_id)
                    .is_some_and(|current| Arc::ptr_eq(current, slot)),
                "pending slot no longer retained"
            );
            ensure!(
                *slot
                    .publication
                    .lock()
                    .map_err(|_| anyhow::anyhow!("publication state poisoned"))?
                    == PublicationState::Unmarked
                    && !slot.accepted_source,
                "publishing operation needs proven rollback or genuine closure"
            );
            let removed = q.entries.remove(&f.operation_id).expect("checked slot");
            let ids = q
                .projects
                .get_mut(&f.scope.project_id)
                .expect("retained project");
            ids.retain(|id| *id != f.operation_id);
            if ids.is_empty() {
                q.projects.remove(&f.scope.project_id);
                q.rotation.retain(|p| *p != f.scope.project_id);
            }
            removed
        };
        Self::release(removed);
        Ok(())
    }
    fn release(slot: Arc<Slot>) {
        // Taking from the shared Arc slot also revokes its guard when a caller
        // retains nongrant allocation reads. No queue/observation lock spans Drop.
        let guard = slot
            .preparation
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        drop(guard);
    }
    fn fair_page(&self) -> Result<Vec<Arc<Slot>>> {
        let mut q = self
            .queue
            .lock()
            .map_err(|_| anyhow::anyhow!("pending queue poisoned"))?;
        let target = q.entries.len().min(PAGE);
        let mut page = Vec::with_capacity(target);
        let mut seen = BTreeSet::new();
        // Each nonempty Project contributes one slot per round; all its own
        // slots rotate too. This finite bound covers unequal Project sizes.
        let rounds = target.saturating_mul(q.projects.len());
        for _ in 0..rounds {
            if page.len() == target {
                break;
            }
            let Some(project) = q.rotation.pop_front() else {
                break;
            };
            q.rotation.push_back(project);
            let ids = q.projects.get_mut(&project).expect("rotated project");
            let id = ids.pop_front().expect("nonempty project");
            ids.push_back(id);
            if seen.insert(id) {
                page.push(q.entries.get(&id).expect("retained operation").clone());
            }
        }
        Ok(page)
    }
    pub(super) fn reconcile_pending(&self) -> Result<bool> {
        let page = self.fair_page()?;
        let pending = !page.is_empty();
        for slot in page {
            let publishing = *slot
                .publication
                .lock()
                .map_err(|_| anyhow::anyhow!("publication state poisoned"))?
                == PublicationState::Publishing;
            let a = &slot.allocation;
            let current = (!publishing).then(|| {
                crate::execution::phase::allocation_snapshot(
                    &self.owner,
                    &a.unit_snapshot().authority(),
                )
            });
            let valid = current
                .unwrap_or_else(|| {
                    Err(anyhow::anyhow!(
                        "publication requires original marker currency"
                    ))
                })
                .and_then(|u| {
                    Ok(serde_json::to_value(u)? == serde_json::to_value(a.unit_snapshot())?)
                })
                .unwrap_or(false);
            // Reconciliation is a level observation, not capacity/binding proof.
            // No queue mutex spans readonly SQL, encoding or Store guard Drop.
            let mut observation = slot
                .observation
                .lock()
                .map_err(|_| anyhow::anyhow!("pending observation poisoned"))?;
            if self.closed.load(Ordering::SeqCst) {
                *observation = PendingObservation::Stopping;
            } else if self.contains(&slot) {
                *observation = if publishing {
                    PendingObservation::MarkerPublicationPending
                } else if valid {
                    PendingObservation::PendingMarker
                } else {
                    PendingObservation::HeldOwnerChanged
                };
            }
        }
        Ok(pending)
    }
    pub(super) fn close_unmarked(&self) {
        self.closed.store(true, Ordering::SeqCst);
        let slots = {
            let mut q = self.queue.lock().unwrap_or_else(|e| e.into_inner());
            // Poison is not evidence that publication never began. Retain such
            // slots and their guards instead of treating them as unmarked.
            let unmarked = q
                .entries
                .iter()
                .filter_map(|(id, slot)| {
                    slot.publication.lock().ok().and_then(|state| {
                        (*state == PublicationState::Unmarked && !slot.accepted_source)
                            .then_some(*id)
                    })
                })
                .collect::<BTreeSet<_>>();
            let slots = unmarked
                .iter()
                .filter_map(|id| q.entries.remove(id))
                .collect::<Vec<_>>();
            for ids in q.projects.values_mut() {
                ids.retain(|id| !unmarked.contains(id));
            }
            q.projects.retain(|_, ids| !ids.is_empty());
            let remaining = q.projects.keys().copied().collect::<BTreeSet<_>>();
            q.rotation.retain(|project| remaining.contains(project));
            slots
        };
        for slot in slots {
            *slot.observation.lock().unwrap_or_else(|e| e.into_inner()) =
                PendingObservation::Stopping;
            Self::release(slot);
        }
    }
    pub(super) fn delay(pending: bool, backoff: &mut u32) -> Duration {
        if !pending {
            *backoff = 0;
            return Duration::from_secs(5);
        }
        let millis = 100_u64.saturating_mul(1 << (*backoff).min(6)).min(5000);
        *backoff = backoff.saturating_add(1).min(6);
        Duration::from_millis(millis)
    }
    pub(super) fn ensure_accepted_shutdown_complete(&self) -> Result<()> {
        let queue = self
            .queue
            .lock()
            .map_err(|_| anyhow::anyhow!("pending queue poisoned"))?;
        ensure!(
            !queue.entries.values().any(|slot| slot.accepted_source),
            "Source phase shutdown remains pending with accepted originals"
        );
        Ok(())
    }
}
impl Drop for PhaseSupervisor {
    fn drop(&mut self) {
        self.close_unmarked();
    }
}

/// Actual marker dispatcher: original objects only, no Runtime or Driver backlink.
pub(super) struct PhaseDispatcher {
    owner: Arc<RuntimeOwner>,
    phases: Arc<PhaseSupervisor>,
    phase_jobs: Arc<super::phase_jobs::PhaseJobs>,
    control_admission: Arc<tokio::sync::Mutex<()>>,
    running: Arc<AtomicBool>,
    stopping: Arc<AtomicBool>,
}
impl PhaseDispatcher {
    pub(super) fn new(
        owner: Arc<RuntimeOwner>,
        phases: Arc<PhaseSupervisor>,
        phase_jobs: Arc<super::phase_jobs::PhaseJobs>,
        control_admission: Arc<tokio::sync::Mutex<()>>,
        running: Arc<AtomicBool>,
        stopping: Arc<AtomicBool>,
    ) -> Arc<Self> {
        Arc::new(Self {
            owner,
            phases,
            phase_jobs,
            control_admission,
            running,
            stopping,
        })
    }
    pub(super) fn supervisor(&self) -> &Arc<PhaseSupervisor> {
        &self.phases
    }
    pub(super) fn belongs_to(&self, owner: &Arc<RuntimeOwner>) -> bool {
        Arc::ptr_eq(&self.owner, owner) && self.phases.belongs_to(owner)
    }
    fn service_running(&self) -> bool {
        self.running.load(Ordering::SeqCst) && !self.stopping.load(Ordering::SeqCst)
    }
    fn validate_original_custody(&self, publication: &OriginalPublicationCustody) -> Result<()> {
        let capacity = &publication.capacity;
        ensure!(
            capacity.slot.accepted_source
                && capacity
                    .supervisor
                    .upgrade()
                    .is_some_and(|s| Arc::ptr_eq(&s, &self.phases))
                && capacity.is_retained()
                && Arc::ptr_eq(capacity.allocation_arc(), publication.plan.allocation()),
            "different original accepted publication capacity"
        );
        let origin = capacity.preparation_origin()?;
        ensure!(
            Arc::ptr_eq(&origin, &publication.origin),
            "different original Source origin"
        );
        origin.validate_publication(&publication.plan)
    }
    fn original_retention(
        &self,
        publication: &OriginalPublicationCustody,
    ) -> Result<Arc<MarkerPublicationRetention>> {
        let assets = publication
            .assets
            .lock()
            .map_err(|_| anyhow::anyhow!("publication custody poisoned"))?;
        let retention = assets
            .retention
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("original publication retention absent"))?;
        ensure!(
            Arc::ptr_eq(&retention.capacity, &publication.capacity)
                && Arc::ptr_eq(&retention.supervisor, &self.phases),
            "different original publication retention"
        );
        Ok(retention.clone())
    }
    /// The first genuine Arc already belongs to the independent outcome cell.
    /// Preserve it in the SAME slot before any fallible origin/job lookup.
    fn handoff_phase_marker(
        &self,
        publication: &OriginalPublicationCustody,
        marker: Arc<crate::state::managed_binding::OriginalMarker>,
    ) -> Result<()> {
        ensure!(
            marker.matches_original_plan(&publication.plan),
            "different returned marker plan"
        );
        let slot = &publication.capacity.slot;
        let marker = {
            let first = marker.clone();
            let mut saved = slot.marker.lock().unwrap_or_else(|e| e.into_inner());
            let original = saved.get_or_insert(marker).clone();
            ensure!(
                original.matches_original_plan(&publication.plan) && Arc::ptr_eq(&original, &first),
                "different first marker retained"
            );
            original
        };
        publication.observation(PublicationObservation::KnownMarker);
        self.validate_original_custody(publication)?;
        let retention = self.original_retention(publication)?;
        let reservation = publication
            .assets
            .lock()
            .map_err(|_| anyhow::anyhow!("publication custody poisoned"))?
            .reservation
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("original job reservation absent"))?;
        self.phase_jobs.ready(&reservation)?;
        let preparation = self.phase_jobs.preparation_custody(&reservation)?;
        publication.origin.validate_marker(&marker)?;
        let q = self
            .phases
            .queue
            .lock()
            .map_err(|_| anyhow::anyhow!("pending queue poisoned"))?;
        ensure!(
            !self.phases.closed.load(Ordering::SeqCst)
                && q.entries
                    .get(&slot.allocation.facts().operation_id)
                    .is_some_and(|current| Arc::ptr_eq(current, slot))
                && Arc::ptr_eq(marker.allocation(), &slot.allocation),
            "marker handoff capacity ended or changed"
        );
        let state = slot
            .publication
            .lock()
            .map_err(|_| anyhow::anyhow!("publication state poisoned"))?;
        ensure!(
            *state == PublicationState::Publishing,
            "marker is not publishing"
        );
        let plan = slot
            .publication_plan
            .lock()
            .map_err(|_| anyhow::anyhow!("publication plan poisoned"))?;
        ensure!(
            plan.as_ref()
                .is_some_and(|p| Arc::ptr_eq(p, &publication.plan)),
            "marker handoff is not the same retained original plan"
        );
        ensure!(
            slot.launch_handed_off
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok(),
            "original Native launch already handed off"
        );
        drop(plan);
        drop(state);
        drop(q);
        // PhaseJobs retains its actual launch, eager guard, handle and results.
        // Handoff does not keep an unused observation receiver.
        drop(self.phase_jobs.start(PhaseLaunch {
            parts: Arc::new(PhaseLaunchParts {
                marker,
                retention,
                origin: publication.origin.clone(),
                preparation: Arc::downgrade(&preparation),
            }),
        }));
        publication.observation(PublicationObservation::HandedOff);
        Ok(())
    }
    /// A wake ID cannot replace these independently saved original objects.
    /// Exact confirmation never republishes the marker or replaces its first Arc.
    pub(super) async fn reconcile_phase_marker(
        &self,
        publication: &Arc<OriginalPublicationCustody>,
    ) -> Result<()> {
        let result = async {
            let _admission = self.control_admission.lock().await;
            ensure!(
                self.service_running(),
                "Runtime stopped before marker reconciliation"
            );
            self.validate_original_custody(publication)?;
            ensure!(
                publication.attempted.load(Ordering::SeqCst)
                    && !publication
                        .capacity
                        .slot
                        .launch_handed_off
                        .load(Ordering::SeqCst),
                "original publication absent or already handed off"
            );
            self.original_retention(publication)?;
            {
                let slot = &publication.capacity.slot;
                let state = slot
                    .publication
                    .lock()
                    .map_err(|_| anyhow::anyhow!("publication state poisoned"))?;
                let plan = slot
                    .publication_plan
                    .lock()
                    .map_err(|_| anyhow::anyhow!("publication plan poisoned"))?;
                ensure!(
                    *state == PublicationState::Publishing
                        && plan
                            .as_ref()
                            .is_some_and(|p| Arc::ptr_eq(p, &publication.plan)),
                    "original publishing plan no longer retained"
                );
            }
            let marker = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .confirm_retained_marker(&publication.plan, &publication.outcome)?;
            self.handoff_phase_marker(publication, marker)
        }
        .await;
        result.map_err(|error| publication.retain_error(error, false))
    }
    /// Plan from SAME accepted Source originals, without any queue/Store borrow.
    pub(super) fn plan_original_marker(
        &self,
        allocation: Arc<NativeAllocation>,
        origin: Arc<super::phase_handoffs::PhasePreparationOrigin>,
        ticket: Arc<crate::state::DriverReadTicket>,
        workflow: crate::domain::RecordId,
    ) -> Result<Arc<crate::state::managed_binding::MarkerPublicationPlan>> {
        ensure!(
            origin.matches_allocation(&allocation),
            "foreign Source allocation"
        );
        origin.validate_ticket(&ticket)?;
        let plan = crate::state::managed_binding::plan_marker_publication(
            self.owner.clone(),
            allocation,
            ticket,
            workflow,
        )?;
        origin.validate_publication(&plan)?;
        Ok(plan)
    }
    /// The Handoff saves this SAME custody before admission. All pre-latch
    /// partial assets are retained; only one original publication may enter SQL.
    pub(super) async fn publish_planned_marker(
        &self,
        publication: &Arc<OriginalPublicationCustody>,
    ) -> Result<()> {
        let result = async {
            let _admission = self.control_admission.lock().await;
            ensure!(
                self.service_running(),
                "Runtime stopped before marker admission"
            );
            self.validate_original_custody(publication)?;
            ensure!(
                !publication.attempted.load(Ordering::SeqCst),
                "original publication already attempted"
            );
            {
                let assets = publication
                    .assets
                    .lock()
                    .map_err(|_| anyhow::anyhow!("publication custody poisoned"))?;
                ensure!(
                    assets.observation == PublicationObservation::Planned,
                    "original publication already entered or held"
                );
            }
            let slot = &publication.capacity.slot;
            {
                let q = self
                    .phases
                    .queue
                    .lock()
                    .map_err(|_| anyhow::anyhow!("pending queue poisoned"))?;
                ensure!(
                    !self.phases.closed.load(Ordering::SeqCst)
                        && q.entries
                            .get(&slot.allocation.facts().operation_id)
                            .is_some_and(|current| Arc::ptr_eq(current, slot)),
                    "original publication slot ended"
                );
                let state = slot
                    .publication
                    .lock()
                    .map_err(|_| anyhow::anyhow!("publication state poisoned"))?;
                ensure!(
                    *state == PublicationState::Unmarked,
                    "marker publication already begun"
                );
                let mut saved = slot
                    .publication_plan
                    .lock()
                    .map_err(|_| anyhow::anyhow!("publication plan poisoned"))?;
                ensure!(saved.is_none(), "publication already has an original plan");
                *saved = Some(publication.plan.clone());
            }
            let reservation = Arc::new(self.phase_jobs.reserve(&slot.allocation)?);
            {
                let mut assets = publication.assets.lock().unwrap_or_else(|e| e.into_inner());
                assets.reservation = Some(reservation);
                assets.observation = PublicationObservation::Reserved;
            }
            let retention = Arc::new(
                self.phases
                    .begin_publication(publication.capacity.clone())?,
            );
            {
                let mut assets = publication.assets.lock().unwrap_or_else(|e| e.into_inner());
                assets.retention = Some(retention);
                assets.observation = PublicationObservation::Publishing;
            }
            ensure!(
                publication
                    .attempted
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok(),
                "original publication already attempted"
            );
            let marker = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .publish_managed_marker(&publication.plan, &publication.outcome)?;
            self.handoff_phase_marker(publication, marker)
        }
        .await;
        result.map_err(|error| publication.retain_error(error, false))
    }
    /// One inline eligible attempt, after the publisher released admission.
    /// Observed precommit is eligibility only; the unchanged proof is mandatory.
    pub(super) async fn rollback_marker_publication(
        &self,
        publication: &Arc<OriginalPublicationCustody>,
    ) -> Result<()> {
        let result = async {
            let _admission = self.control_admission.lock().await;
            ensure!(
                publication.may_attempt_unpublished()?,
                "publication outcome cannot qualify unpublished proof"
            );
            self.validate_original_custody(publication)?;
            ensure!(
                publication.attempted.load(Ordering::SeqCst)
                    && !publication
                        .capacity
                        .slot
                        .launch_handed_off
                        .load(Ordering::SeqCst),
                "publication has no original rollback responsibility"
            );
            self.original_retention(publication)?;
            let reservation = publication
                .assets
                .lock()
                .map_err(|_| anyhow::anyhow!("publication custody poisoned"))?
                .reservation
                .as_ref()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("original job reservation absent"))?;
            let plan = crate::state::managed_binding::plan_unpublished_marker(
                &self.owner,
                publication.capacity.allocation(),
            )?;
            let proof = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .confirm_unpublished_marker(plan)?;
            self.phases
                .restore_unpublished(&publication.capacity.slot, proof)?;
            // SAME accepted slot remains retained, even on closed rollback.
            // Reuse is not this call's removable reservation.
            self.phase_jobs.remove_unstarted(&reservation)?;
            publication.observation(PublicationObservation::RestoredHeld);
            Ok(())
        }
        .await;
        result.map_err(|error| publication.retain_error(error, true))
    }
    #[cfg(test)]
    async fn retain_marker_publication(
        &self,
        capacity: PendingPhaseCapacity,
    ) -> Result<LegacyMarkerPublicationRetention> {
        let _admission = self.control_admission.lock().await;
        ensure!(
            self.service_running(),
            "Runtime is not accepting marker publication"
        );
        ensure!(
            !capacity.slot.accepted_source
                && capacity
                    .slot
                    .origin
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Source origin poisoned"))?
                    .is_none()
                && capacity
                    .slot
                    .publication_plan
                    .lock()
                    .map_err(|_| anyhow::anyhow!("publication plan poisoned"))?
                    .is_none(),
            "legacy no-plan control cannot adopt accepted Source publication"
        );
        let reservation = self.phase_jobs.reserve(capacity.allocation_arc())?;
        let retention = self.phases.begin_publication(Arc::new(capacity))?;
        Ok(LegacyMarkerPublicationRetention {
            retention,
            reservation,
        })
    }
    #[cfg(test)]
    async fn rollback_legacy_marker_publication(
        &self,
        publication: LegacyMarkerPublicationRetention,
    ) -> Result<Arc<PendingPhaseCapacity>> {
        let _admission = self.control_admission.lock().await;
        let retention = publication.retention;
        ensure!(
            !retention.capacity.slot.accepted_source
                && Arc::ptr_eq(&retention.supervisor, &self.phases)
                && retention.is_retained(),
            "legacy rollback requires same never-accepted slot"
        );
        let plan = crate::state::managed_binding::plan_unpublished_marker(
            &self.owner,
            retention.allocation(),
        )?;
        let proof = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .confirm_unpublished_marker(plan)?;
        self.phases
            .restore_unpublished(&retention.capacity.slot, proof)?;
        self.phase_jobs.remove_unstarted(&publication.reservation)?;
        if self.phases.closed.load(Ordering::SeqCst) {
            self.phases.remove_unmarked(&retention.capacity.slot)?;
        }
        Ok(retention.capacity)
    }
}

impl super::Runtime {
    #[cfg(test)]
    pub(crate) async fn retain_marker_publication(
        &self,
        capacity: PendingPhaseCapacity,
    ) -> Result<LegacyMarkerPublicationRetention> {
        self.phase_dispatcher
            .retain_marker_publication(capacity)
            .await
    }
    #[cfg(test)]
    pub(crate) async fn rollback_marker_publication(
        &self,
        publication: LegacyMarkerPublicationRetention,
    ) -> Result<Arc<PendingPhaseCapacity>> {
        self.phase_dispatcher
            .rollback_legacy_marker_publication(publication)
            .await
    }
    /// Genuine retained queue admission only. Native-ready Driver/marker is absent.
    pub(crate) async fn reserve_pending_phase(
        &self,
        allocation: NativeAllocation,
        preparation: PreparationGuard,
    ) -> std::result::Result<PendingPhaseCapacity, Box<PendingReservationError>> {
        let _admission = self.control_admission.lock().await;
        if !self.service_running() {
            return Err(Box::new(PendingReservationError {
                reason: "Runtime service is not accepting pending phases",
                allocation,
                preparation,
            }));
        }
        let result = self.phases.reserve(allocation, preparation);
        self.wake.notify_one();
        result
    }
}

#[cfg(test)]
mod tests;
