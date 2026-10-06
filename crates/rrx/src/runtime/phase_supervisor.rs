//! Real Runtime-owned, bounded pre-marker allocation retention. This is nongrant:
//! no Driver, OriginalMarker, native owner/input/terminal or readiness is minted.
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
    HeldOwnerChanged,
    Stopping,
}
struct Slot {
    allocation: Arc<NativeAllocation>,
    preparation: Mutex<Option<PreparationGuard>>,
    observation: Mutex<PendingObservation>,
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
    /// This first stage has no marked transition. Future marked operations must
    /// use genuine typed closure, not expose this unmarked removal path to them.
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
    global: usize,
    per_project: usize,
    closed: AtomicBool,
    queue: Mutex<Queue>,
}
impl PhaseSupervisor {
    pub(super) fn new(owner: Arc<RuntimeOwner>, global: usize, per_project: usize) -> Arc<Self> {
        Arc::new(Self {
            owner,
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
        let slot = Arc::new(Slot {
            allocation: Arc::new(allocation),
            preparation: Mutex::new(Some(preparation)),
            observation: Mutex::new(PendingObservation::PendingMarker),
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
    fn contains(&self, slot: &Arc<Slot>) -> bool {
        !self.closed.load(Ordering::SeqCst)
            && self.queue.lock().is_ok_and(|q| {
                q.entries
                    .get(&slot.allocation.facts().operation_id)
                    .is_some_and(|current| Arc::ptr_eq(current, slot))
            })
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
            let a = &slot.allocation;
            let current = crate::execution::phase::allocation_snapshot(
                &self.owner,
                &a.unit_snapshot().authority(),
            );
            let valid = current
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
                *observation = if valid {
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
            let entries = std::mem::take(&mut q.entries);
            q.projects.clear();
            q.rotation.clear();
            entries.into_values().collect::<Vec<_>>()
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
}
impl Drop for PhaseSupervisor {
    fn drop(&mut self) {
        self.close_unmarked();
    }
}

impl super::Runtime {
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
