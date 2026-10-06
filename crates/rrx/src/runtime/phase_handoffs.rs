//! Custody of the actual Source envelope; observations grant no Native authority.
use super::phase_supervisor::{PendingPhaseCapacity, PendingReservationError, PhaseSupervisor};
use crate::{
    adapter::native::NativePhasePort,
    domain::{ContextVersion, Record, Task, TaskId},
    execution::{
        RuntimeOwner,
        phase::NativeAllocation,
        workflow_source::{ManagedWorkflowSources, SourceNativeCustody, SourceNativeHandoff},
    },
    state::{DriverReadTicket, managed_binding::OriginalMarker},
};
use anyhow::{Result, ensure};
use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};
use tokio::{sync::watch, task::JoinHandle};

const MAX_HANDOFFS: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceHandoffState {
    AwaitingOffer,
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
    /// Reject a different genuine but row-equal ticket BEFORE marker planning
    /// or SQL. Only the Source producer's SAME original Arc can continue.
    pub(super) fn validate_ticket(&self, ticket: &Arc<DriverReadTicket>) -> Result<()> {
        let original = self
            .ticket
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("original Source ticket ended"))?;
        ensure!(
            Arc::ptr_eq(&original, ticket),
            "different original Source ticket"
        );
        Ok(())
    }
    /// Validate a SAME saved original plan before uncertain-commit confirmation.
    /// No current row, recaptured ticket or post-commit cache is substituted.
    pub(super) fn validate_publication(
        &self,
        plan: &crate::state::managed_binding::MarkerPublicationPlan,
    ) -> Result<()> {
        let ticket = self
            .ticket
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("original Source ticket ended"))?;
        ensure!(
            self.matches_allocation(plan.allocation()) && plan.matches_source_ticket(&ticket),
            "publication plan lacks same original Source ticket/allocation"
        );
        Ok(())
    }
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
    slot: Arc<IngressSlot>,
    // The independently owned handle is never captured by its own future.
    handle: Option<JoinHandle<()>>,
}

#[derive(Default)]
pub(super) struct PhaseHandoffs {
    entries: Mutex<BTreeMap<TaskId, Entry>>,
}

/// Independent EMPTY slot. It owns no ticket, allocation, plan or Consumer.
struct IngressSlot {
    task: TaskId,
    handoff: Mutex<Option<Arc<Handoff>>>,
    changed: watch::Sender<SourceHandoffState>,
}

/// Held by the actual inline Source producer, never a spawned offer. The queue
/// reference is Weak: another Task's marker can retain the whole Driver registry.
struct PreOfferConsumer {
    owner: Arc<RuntimeOwner>,
    phases: Weak<PhaseSupervisor>,
    selected: Weak<NativePhasePort>,
    control: Arc<tokio::sync::Mutex<()>>,
    running: Arc<AtomicBool>,
    stopping: Arc<AtomicBool>,
}

/// Nongrant finite ingress reservation. Consumed once in the SAME poll as the
/// actual inline offer returns; Drop can remove only this SAME original EMPTY.
pub(crate) struct SourceHandoffReservation {
    registry: Weak<PhaseHandoffs>,
    slot: Arc<IngressSlot>,
    consumer: PreOfferConsumer,
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
                SourceHandoffState::AwaitingOffer
                    | SourceHandoffState::Retained
                    | SourceHandoffState::Transferring
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

impl PreOfferConsumer {
    fn accepting(&self) -> bool {
        self.running.load(Ordering::SeqCst) && !self.stopping.load(Ordering::SeqCst)
    }
    /// Used only AFTER the original envelope is saved in the independent slot.
    fn upgrade(&self, source: &SourceNativeHandoff) -> Result<Consumer> {
        ensure!(
            self.accepting(),
            "Runtime stopped before Source installation"
        );
        let phases = self
            .phases
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("original phase supervisor ended"))?;
        let selected = self
            .selected
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("original selected port ended"))?;
        ensure!(
            phases.belongs_to(&self.owner)
                && source.ticket().belongs_to_owner(&self.owner)
                && Arc::ptr_eq(source.custody().selected_port(), &selected),
            "Source installation has different original components"
        );
        Ok(Consumer {
            owner: self.owner.clone(),
            phases,
            control: self.control.clone(),
            running: self.running.clone(),
            stopping: self.stopping.clone(),
        })
    }
}

impl PhaseHandoffs {
    /// Actual private Runtime calls this only after its complete original ticket
    /// read. Task is a capacity key; no Native/Driver proof is constructed here.
    fn reserve_empty(
        self: &Arc<Self>,
        task: TaskId,
        consumer: PreOfferConsumer,
    ) -> Result<SourceHandoffReservation> {
        ensure!(
            consumer.accepting(),
            "Runtime is not accepting Source handoffs"
        );
        tokio::runtime::Handle::try_current()?;
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("Source handoffs poisoned"))?;
        ensure!(
            entries.len() < MAX_HANDOFFS && !entries.contains_key(&task),
            "Source handoff ingress full or Task already retained"
        );
        let (changed, _) = watch::channel(SourceHandoffState::AwaitingOffer);
        let slot = Arc::new(IngressSlot {
            task,
            handoff: Mutex::new(None),
            changed,
        });
        entries.insert(
            task,
            Entry {
                slot: slot.clone(),
                handle: None,
            },
        );
        Ok(SourceHandoffReservation {
            registry: Arc::downgrade(self),
            slot,
            consumer,
        })
    }

    pub(super) fn observe(&self, task: TaskId) -> Result<Option<SourceHandoffObservation>> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("Source handoffs poisoned"))?;
        Ok(entries.get(&task).map(|entry| SourceHandoffObservation {
            changed: entry.slot.changed.subscribe(),
        }))
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

impl SourceHandoffReservation {
    /// No caller-supplied future or validator. This wrapper owns ONLY the actual
    /// Sources producer and establishes its cancellation/destructor ordering.
    pub(crate) fn offer_first_executor<'a>(
        self,
        sources: &'a Arc<ManagedWorkflowSources>,
        task: &'a Task,
        record: &'a Record,
        context: &'a ContextVersion,
        ticket: DriverReadTicket,
        selected: Arc<NativePhasePort>,
    ) -> impl Future<Output = Result<SourceHandoffObservation>> + Send + 'a {
        InlineSourceOffer {
            offer: Some(Box::pin(
                sources.offer_first_executor(task, record, context, ticket, selected),
            )),
            reservation: Some(self),
        }
    }
    /// Infallible ownership move, NOT an acceptance/proof API. No await, status
    /// projection or origin check can precede saving the SAME actual envelope.
    fn install(self, source: SourceNativeHandoff) -> SourceHandoffObservation {
        let handoff = Arc::new(Handoff {
            source,
            assets: Mutex::new(Assets {
                capacity: None,
                origin: None,
                malformed_return: None,
                error: None,
            }),
            changed: self.slot.changed.clone(),
        });
        let poisoned = match self.slot.handoff.lock() {
            Ok(mut saved) => {
                *saved = Some(handoff.clone());
                false
            }
            Err(error) => {
                *error.into_inner() = Some(handoff.clone());
                true
            }
        };
        // This slot can be filled only by this non-Clone consuming reservation.
        // It is independently owned before any fallible check or future exists.
        handoff.changed.send_replace(SourceHandoffState::Retained);
        let observation = SourceHandoffObservation {
            changed: handoff.changed.subscribe(),
        };
        let installed = if poisoned {
            Err(anyhow::anyhow!("original ingress slot poisoned"))
        } else {
            self.install_retained(&handoff)
        };
        if let Err(error) = installed {
            handoff
                .assets
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .error = Some(error);
            handoff.changed.send_replace(SourceHandoffState::Held);
        }
        observation
    }

    fn install_retained(&self, handoff: &Arc<Handoff>) -> Result<()> {
        ensure!(
            handoff.source.ticket().task().id == self.slot.task,
            "Source envelope belongs to a different original Task"
        );
        // Strong queue ownership is constructed here only AFTER saving envelope,
        // then moved into the independently retained invocation, not the Driver.
        let consumer = self.consumer.upgrade(&handoff.source)?;
        let executor = tokio::runtime::Handle::try_current()?;
        let registry = self
            .registry
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("original handoff registry ended"))?;
        let mut entries = registry
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("Source handoffs poisoned"))?;
        let entry = entries
            .get_mut(&self.slot.task)
            .ok_or_else(|| anyhow::anyhow!("original ingress slot ended"))?;
        ensure!(
            Arc::ptr_eq(&entry.slot, &self.slot) && entry.handle.is_none(),
            "original ingress slot replaced or already invoked"
        );
        let running = RunningHandoff(handoff.clone());
        let handoff = handoff.clone();
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
        // No await follows spawn before the actual handle reaches its owner.
        entry.handle = Some(handle);
        Ok(())
    }
}

/// Private concrete producer owner; no public arbitrary-future constructor.
struct InlineSourceOffer<'a> {
    offer: Option<Pin<Box<dyn Future<Output = Result<SourceNativeHandoff>> + Send + 'a>>>,
    reservation: Option<SourceHandoffReservation>,
}
impl Future for InlineSourceOffer<'_> {
    type Output = Result<SourceHandoffObservation>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self
            .offer
            .as_mut()
            .expect("actual inline Source producer")
            .as_mut()
            .poll(cx)
        {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(source)) => {
                // First save real ownership; no fallible action or await between
                // actual producer success and independent retention.
                let observation = self
                    .reservation
                    .take()
                    .expect("original ingress reservation")
                    .install(source);
                drop(self.offer.take());
                Poll::Ready(Ok(observation))
            }
            Poll::Ready(Err(error)) => {
                drop(self.offer.take());
                drop(self.reservation.take());
                Poll::Ready(Err(error))
            }
        }
    }
}
impl Drop for InlineSourceOffer<'_> {
    fn drop(&mut self) {
        // Never infer producer termination from a watch or timeout. End the real
        // sole inline future before Drop can remove this original EMPTY slot.
        drop(self.offer.take());
        drop(self.reservation.take());
    }
}

impl Drop for SourceHandoffReservation {
    fn drop(&mut self) {
        // In the concrete inline callsite this Drop means the sole offer future
        // ended or was dropped. No detached offer may still produce an envelope.
        let Some(registry) = self.registry.upgrade() else {
            return;
        };
        let Ok(mut entries) = registry.entries.lock() else {
            self.slot.changed.send_replace(SourceHandoffState::Held);
            return;
        };
        let Some(entry) = entries.get(&self.slot.task) else {
            return;
        };
        if !Arc::ptr_eq(&entry.slot, &self.slot) || entry.handle.is_some() {
            return;
        }
        let Ok(saved) = self.slot.handoff.lock() else {
            self.slot.changed.send_replace(SourceHandoffState::Held);
            return;
        };
        if saved.is_some() {
            return;
        }
        // Detach only this SAME actual EMPTY; no ID/status/timer settlement.
        let removed = entries.remove(&self.slot.task);
        drop(saved);
        drop(entries);
        drop(removed);
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
    fn source_consumer(&self, selected: &Arc<NativePhasePort>) -> PreOfferConsumer {
        PreOfferConsumer {
            owner: self.owner.clone(),
            phases: Arc::downgrade(&self.phases),
            selected: Arc::downgrade(selected),
            control: self.control_admission.clone(),
            running: self.running.clone(),
            stopping: self.stopping.clone(),
        }
    }
    pub(crate) fn observe_source_handoff(
        &self,
        task: TaskId,
    ) -> Result<Option<SourceHandoffObservation>> {
        self.phase_handoffs.observe(task)
    }
    /// Reserve before the actual inline offer. Strong Runtime ends before await.
    pub(crate) fn reserve_source_handoff(
        &self,
        ticket: &DriverReadTicket,
        selected: &Arc<NativePhasePort>,
    ) -> Result<SourceHandoffReservation> {
        ensure!(
            self.service_running()
                && ticket.belongs_to_owner(&self.owner)
                && std::ptr::eq(selected.owner(), self.owner.as_ref())
                && selected.alias() == ticket.task().executor,
            "Source ingress has different original Runtime/selection"
        );
        selected.selected_adapter()?;
        ticket.validate_current_read()?;
        self.phase_handoffs
            .reserve_empty(ticket.task().id, self.source_consumer(selected))
    }
}

#[cfg(test)]
mod tests;
