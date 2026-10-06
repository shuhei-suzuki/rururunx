//! Custody of the actual Source envelope; observations grant no Native authority.
use super::phase_supervisor::{PendingPhaseCapacity, PendingReservationError, PhaseSupervisor};
use crate::{
    domain::TaskId,
    execution::{
        OperationId, RuntimeOwner,
        phase::NativeAllocation,
        workflow_source::{SourceNativeCustody, SourceNativeHandoff},
    },
    state::{DriverReadTicket, managed_binding::OriginalMarker},
};
use anyhow::{Result, ensure};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{sync::watch, task::JoinHandle};

const MAX_HANDOFFS: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceHandoffState {
    Retained,
    Transferring,
    Reserved,
    Refused,
    Held,
}

struct Assets {
    capacity: Option<PendingPhaseCapacity>,
    origin: Option<Arc<PhasePreparationOrigin>>,
    malformed_return: Option<Box<PendingReservationError>>,
    error: Option<anyhow::Error>,
}

/// Created only below, AFTER the concrete original Source take/reserve/accept.
/// Retains original immutable input via its allocation, not a copied row proof.
/// No strong ticket, marker, capacity, Runtime or registry return edge.
pub(super) struct PhasePreparationOrigin {
    source: Arc<SourceNativeCustody>,
    allocation: Arc<NativeAllocation>,
    ticket: Weak<DriverReadTicket>,
}
impl PhasePreparationOrigin {
    pub(super) fn matches_allocation(&self, allocation: &Arc<NativeAllocation>) -> bool {
        Arc::ptr_eq(&self.allocation, allocation)
            && self.source.operation() == allocation.facts().operation_id
    }
    /// Immutable original-object linkage only, safe under SharedStore. Native
    /// must ALSO conjoin real admission/current/Driver/Unit/pair/permissions.
    pub(super) fn validate_marker(&self, marker: &OriginalMarker) -> Result<()> {
        let ticket = self
            .ticket
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("original Source ticket ended"))?;
        ensure!(
            self.matches_allocation(marker.allocation()) && marker.matches_source_ticket(&ticket),
            "Native preparation lacks same accepted Source/marker/ticket"
        );
        Ok(())
    }
}
struct Handoff {
    // Original non-Clone envelope and SAME Arc ticket survive planning refusal.
    source: SourceNativeHandoff,
    assets: Mutex<Assets>,
    changed: watch::Sender<SourceHandoffState>,
}
struct Entry {
    handoff: Arc<Handoff>,
    // The independently owned handle is never captured by its own future.
    handle: Option<JoinHandle<()>>,
}

#[derive(Default)]
pub(super) struct PhaseHandoffs {
    entries: Mutex<BTreeMap<OperationId, Entry>>,
}

/// Dropping an observer neither aborts the invocation nor releases its assets.
pub(crate) struct SourceHandoffObservation {
    changed: watch::Receiver<SourceHandoffState>,
}
impl SourceHandoffObservation {
    pub(crate) fn state(&self) -> SourceHandoffState {
        *self.changed.borrow()
    }
    pub(crate) async fn wait(&mut self) -> SourceHandoffState {
        loop {
            let state = *self.changed.borrow_and_update();
            if !matches!(
                state,
                SourceHandoffState::Retained | SourceHandoffState::Transferring
            ) {
                return state;
            }
            if self.changed.changed().await.is_err() {
                return SourceHandoffState::Held;
            }
        }
    }
}

/// Concrete original Runtime objects, with no strong Runtime/registry backlink.
struct Consumer {
    owner: Arc<RuntimeOwner>,
    phases: Arc<PhaseSupervisor>,
    control: Arc<tokio::sync::Mutex<()>>,
    running: Arc<AtomicBool>,
    stopping: Arc<AtomicBool>,
}
impl Consumer {
    fn accepting(&self) -> bool {
        self.running.load(Ordering::SeqCst) && !self.stopping.load(Ordering::SeqCst)
    }
    async fn transfer(&self, handoff: &Handoff) -> Result<SourceHandoffState> {
        let _admission = self.control.clone().lock_owned().await;
        ensure!(self.accepting(), "Runtime stopped before Source transfer");
        let source = &handoff.source;
        // Check the SAME ticket before acquiring any Source locks. This is not
        // recapture, marker publication, prepared-input proof or Native permission.
        source.ticket().validate_current_read()?;
        let origin = source.custody().original_origin()?;
        let mut transfer = origin.lock_transfer(source.custody(), &self.owner, source.ticket())?;
        let (allocation, preparation) = transfer.take_original()?;
        // No await or SharedStore acquisition while borrowing Source. The queue
        // uses its existing independent read-only allocation snapshot.
        match self.phases.reserve(allocation, preparation) {
            Ok(capacity) => {
                let allocation = capacity.allocation_arc().clone();
                // Save ownership BEFORE the Source's infallible acceptance.
                // Poison recovery here retains objects only; it grants no effect.
                handoff
                    .assets
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .capacity = Some(capacity);
                transfer.finish_accepted();
                // This constructor is private to the real successful producer.
                // It cannot run before original capacity custody/Source acceptance.
                let origin = Arc::new(PhasePreparationOrigin {
                    source: source.custody().clone(),
                    allocation: allocation.clone(),
                    ticket: Arc::downgrade(source.ticket()),
                });
                handoff
                    .assets
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .origin = Some(origin.clone());
                // Source locks have ended. Retain the SAME sealed origin beside
                // the queue's armed preparation before marker planning is allowed.
                // On failure the independent Handoff keeps both actual objects.
                self.phases.retain_preparation_origin(&allocation, origin)?;
                Ok(SourceHandoffState::Reserved)
            }
            Err(error) => {
                let reason = error.reason;
                let state = match transfer.restore_refused(*error) {
                    Ok(crate::execution::workflow_source::SourceRefusalRestoration::Restored) => {
                        SourceHandoffState::Refused
                    }
                    Ok(crate::execution::workflow_source::SourceRefusalRestoration::Held) => {
                        SourceHandoffState::Held
                    }
                    Err(original) => {
                        // Store the SAME returned allocation/armed guard before
                        // releasing Source locks. Never drop it under those locks.
                        handoff
                            .assets
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .malformed_return = Some(original);
                        SourceHandoffState::Held
                    }
                };
                // Drop the transfer/origin before any later continuation. The
                // error label is observation only, never a retry/rollback grant.
                drop(transfer);
                handoff
                    .assets
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .error = Some(anyhow::anyhow!(reason));
                Ok(state)
            }
        }
    }
}

impl PhaseHandoffs {
    fn retain(
        &self,
        source: SourceNativeHandoff,
        consumer: Consumer,
    ) -> std::result::Result<SourceHandoffObservation, SourceNativeHandoff> {
        let executor = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => return Err(source),
        };
        let mut entries = match self.entries.lock() {
            Ok(entries) => entries,
            Err(_) => return Err(source),
        };
        // The finite first-Executor lane is stricter than per-Unit uniqueness:
        // only one original handoff per Task may be held here. IDs are keys only;
        // the actual private envelope/ticket remains the sole continuation.
        let operation = source.custody().operation();
        let task = source.ticket().task().id;
        if !consumer.accepting()
            || entries.len() >= MAX_HANDOFFS
            || entries.contains_key(&operation)
            || entries
                .values()
                .any(|e| e.handoff.source.ticket().task().id == task)
        {
            return Err(source);
        }
        let (changed, _) = watch::channel(SourceHandoffState::Retained);
        let handoff = Arc::new(Handoff {
            source,
            assets: Mutex::new(Assets {
                capacity: None,
                origin: None,
                malformed_return: None,
                error: None,
            }),
            changed,
        });
        let observation = SourceHandoffObservation {
            changed: handoff.changed.subscribe(),
        };
        entries.insert(
            operation,
            Entry {
                handoff: handoff.clone(),
                handle: None,
            },
        );
        // The envelope is independently retained before constructing this future.
        // An eager guard also covers destruction BEFORE its first poll.
        let running = RunningHandoff(handoff.clone());
        let handle = executor.spawn(async move {
            let _running = running;
            handoff
                .changed
                .send_replace(SourceHandoffState::Transferring);
            match consumer.transfer(&handoff).await {
                Ok(state) => {
                    handoff.changed.send_replace(state);
                }
                Err(error) => {
                    handoff
                        .assets
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .error = Some(error);
                    handoff.changed.send_replace(SourceHandoffState::Held);
                }
            }
        });
        // No await occurs before the actual handle reaches its independent owner.
        entries
            .get_mut(&operation)
            .expect("retained actual Source envelope")
            .handle = Some(handle);
        Ok(observation)
    }

    pub(super) fn observe(&self, task: TaskId) -> Option<SourceHandoffObservation> {
        let entries = self.entries.lock().ok()?;
        entries
            .values()
            .find(|e| e.handoff.source.ticket().task().id == task)
            .map(|e| SourceHandoffObservation {
                changed: e.handoff.changed.subscribe(),
            })
    }

    pub(super) fn ensure_shutdown_complete(&self) -> Result<()> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("Source handoffs poisoned"))?;
        // Future completion alone cannot settle Source originals/capacity. This
        // increment has no genuine closure/retirement producer, so none is evicted.
        ensure!(
            entries.is_empty(),
            "Source handoff shutdown remains pending with retained originals"
        );
        Ok(())
    }
}

struct RunningHandoff(Arc<Handoff>);
impl Drop for RunningHandoff {
    fn drop(&mut self) {
        self.0.changed.send_if_modified(|state| {
            if matches!(
                state,
                SourceHandoffState::Retained | SourceHandoffState::Transferring
            ) {
                *state = SourceHandoffState::Held;
                true
            } else {
                false
            }
        });
    }
}

impl super::Runtime {
    /// Concrete synchronous entry. On ordinary refusal caller MUST retain the
    /// SAME returned envelope; no copied IDs can later reconstruct it.
    pub(crate) fn retain_source_handoff(
        &self,
        source: SourceNativeHandoff,
    ) -> std::result::Result<SourceHandoffObservation, SourceNativeHandoff> {
        self.phase_handoffs.retain(
            source,
            Consumer {
                owner: self.owner.clone(),
                phases: self.phases.clone(),
                control: self.control_admission.clone(),
                running: self.running.clone(),
                stopping: self.stopping.clone(),
            },
        )
    }
}
