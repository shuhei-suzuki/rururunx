//! Live association comes from the actual retained worker, never its durable row.
use crate::{
    domain::TaskId,
    state::{DriverPreparationAdvance, InitialDriverPlan, PendingDriverClaim},
};
use anyhow::{Context as _, Result, ensure};
use std::future::Future;
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Waker},
};
use uuid::Uuid;

pub(crate) struct DriverRegistry {
    epoch: u64,
    state_path: std::path::PathBuf,
    instance: String,
    entries: Mutex<BTreeMap<TaskId, Arc<DriverSlot>>>,
    preparation_cursor: Mutex<Option<TaskId>>,
}
struct DriverSlot {
    task: TaskId,
    id: Uuid,
    epoch: u64,
    binding: Mutex<(u64, String)>,
    active: AtomicBool,
    worker_entered: AtomicBool,
    activated: AtomicBool,
    initial_claim: Mutex<Option<Arc<InitialDriverPlan>>>,
    preparation: Mutex<Option<Arc<DriverPreparationAdvance>>>,
    revoked: AtomicBool,
    cancel: tokio::sync::Notify,
    // Installed synchronously immediately after spawn; never detached on caller Drop.
    job: Mutex<Option<tokio::task::JoinHandle<Result<()>>>>,
    exit: Mutex<Option<DriverExitKind>>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum DriverExitKind {
    Returned,
    Failed,
    Panicked,
    Aborted,
}
/// Produced only by observing the actual retained JoinHandle. It conveys a
/// factual dispatcher outcome, not Native success, no-effect or settlement.
pub(crate) struct DriverExit {
    task: TaskId,
    id: Uuid,
    epoch: u64,
    kind: DriverExitKind,
    never_activated: Option<Arc<InitialDriverPlan>>,
}
impl DriverExit {
    pub(crate) fn identity(&self) -> (TaskId, Uuid, u64) {
        (self.task, self.id, self.epoch)
    }
    pub(crate) fn never_activated(&self) -> Option<&InitialDriverPlan> {
        self.never_activated.as_deref()
    }
    pub(crate) fn label(&self) -> &'static str {
        match self.kind {
            DriverExitKind::Returned => "returned",
            DriverExitKind::Failed => "failed",
            DriverExitKind::Panicked => "panicked",
            DriverExitKind::Aborted => "aborted",
        }
    }
}
/// A reserved, non-live registry entry. No serialized IDs can construct it.
pub(super) struct PendingRegistration {
    registry: Arc<DriverRegistry>,
    slot: Arc<DriverSlot>,
}
pub(crate) struct WorkerLifetime {
    registry: Arc<DriverRegistry>,
    slot: Arc<DriverSlot>,
}
/// Created only after the actual Task dispatcher enters its retained future.
pub(super) struct StartupAcknowledgement {
    slot: Arc<DriverSlot>,
}
impl Drop for WorkerLifetime {
    fn drop(&mut self) {
        self.slot.active.store(false, Ordering::SeqCst);
        self.slot.revoked.store(true, Ordering::SeqCst);
        self.slot.cancel.notify_waiters();
        // Actual worker unwind/cancellation makes an unstarted retained plan
        // observable. No plan is reminted and no resources are released here.
        if let Some(plan) = self
            .slot
            .preparation
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
        {
            plan.allow_reconciliation();
        }
    }
}
impl Drop for PendingRegistration {
    fn drop(&mut self) {
        let Ok(job) = self.slot.job.lock() else {
            self.slot.revoked.store(true, Ordering::SeqCst);
            return;
        };
        if job.is_some() || self.slot.worker_entered.load(Ordering::SeqCst) {
            return;
        }
        self.slot.revoked.store(true, Ordering::SeqCst);
        if let Ok(mut entries) = self.registry.entries.lock()
            && entries
                .get(&self.slot.task)
                .is_some_and(|s| Arc::ptr_eq(s, &self.slot))
        {
            entries.remove(&self.slot.task);
        }
        // No DB/history release is asserted. Uncertain claim commits stay held.
    }
}
impl PendingRegistration {
    pub(super) fn bind_claim(&self, claim: &PendingDriverClaim) -> Result<()> {
        let (id, epoch, version, body) = claim.identity();
        ensure!(
            id == self.slot.id
                && epoch == self.slot.epoch
                && version == 1
                && *self
                    .slot
                    .binding
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Driver binding poisoned"))?
                    == (version, body.to_owned()),
            "pending actual claim differs"
        );
        let mut current = self
            .slot
            .initial_claim
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver initial claim custody poisoned"))?;
        ensure!(current.is_none(), "Driver initial claim already retained");
        *current = Some(claim.initial_plan());
        Ok(())
    }
    pub(super) fn spawn(
        self,
        work: impl FnOnce(
            WorkerLifetime,
            StartupAcknowledgement,
        ) -> std::pin::Pin<Box<dyn Future<Output = Result<()>> + Send>>
        + Send
        + 'static,
    ) -> Result<()> {
        // Custody storage is acquired BEFORE spawn. Poison cannot turn a created
        // JoinHandle into an unobserved/detached job.
        let mut job = self
            .slot
            .job
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver job custody poisoned"))?;
        ensure!(
            job.is_none() && !self.slot.revoked.load(Ordering::SeqCst),
            "Driver job already installed or revoked"
        );
        let registry = self.registry.clone();
        let slot = self.slot.clone();
        *job = Some(tokio::spawn(async move {
            ensure!(
                !slot.revoked.load(Ordering::SeqCst),
                "Driver revoked before worker startup"
            );
            slot.worker_entered.store(true, Ordering::SeqCst);
            let lifetime = WorkerLifetime {
                registry,
                slot: slot.clone(),
            };
            let ack = StartupAcknowledgement { slot };
            work(lifetime, ack).await
        }));
        Ok(())
    }
}
/// Authentic association to the retained Task dispatcher; not deserializable.
/// Scalar copies of its binding are content, never constructors for this type.
pub(crate) struct DriverAssociation {
    registry: Arc<DriverRegistry>,
    slot: Arc<DriverSlot>,
}
impl DriverAssociation {
    pub(crate) fn task(&self) -> TaskId {
        self.slot.task
    }
    pub(crate) fn owner_matches(&self, owner: &crate::execution::RuntimeOwner) -> bool {
        self.registry.epoch == owner.epoch()
            && self.registry.state_path == owner.state_path()
            && self.registry.instance == owner.instance_id()
    }
    pub(crate) fn binding(&self) -> Result<(Uuid, u64, u64, String)> {
        let b = self
            .slot
            .binding
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver binding poisoned"))?;
        Ok((self.slot.id, self.slot.epoch, b.0, b.1.clone()))
    }
    pub(crate) fn validates(&self, id: Uuid, epoch: u64, version: u64, body: &str) -> bool {
        self.registry
            .is_current(self.slot.task, id, epoch, version, body)
    }
    /// The same advance enters actual slot custody BEFORE any SQL or caller await.
    pub(crate) fn retain_preparation(&self, plan: &Arc<DriverPreparationAdvance>) -> Result<()> {
        ensure!(plan.belongs_to(self), "preparation association differs");
        let mut pending = self
            .slot
            .preparation
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver preparation custody poisoned"))?;
        ensure!(
            pending.is_none(),
            "Driver preparation reconciliation already pending"
        );
        let (id, epoch, version, body) = plan.original_binding();
        let binding = self
            .slot
            .binding
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver binding poisoned"))?;
        ensure!(
            self.slot.id == id
                && self.slot.epoch == epoch
                && binding.0 == version
                && binding.1 == body
                && self.slot.active.load(Ordering::SeqCst)
                && self.slot.worker_entered.load(Ordering::SeqCst)
                && !self.slot.revoked.load(Ordering::SeqCst),
            "Driver preparation no longer live"
        );
        *pending = Some(plan.clone());
        Ok(())
    }
    pub(crate) fn same_association(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.registry, &other.registry) && Arc::ptr_eq(&self.slot, &other.slot)
    }
    pub(crate) fn preparation_retained(
        &self,
        plan: &Arc<DriverPreparationAdvance>,
    ) -> Result<bool> {
        let pending = self
            .slot
            .preparation
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver preparation custody poisoned"))?;
        Ok(pending.as_ref().is_some_and(|p| Arc::ptr_eq(p, plan)))
    }
    pub(crate) fn retire_preparation(&self, plan: &Arc<DriverPreparationAdvance>) -> Result<()> {
        let mut pending = self
            .slot
            .preparation
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver preparation custody poisoned"))?;
        ensure!(
            pending.as_ref().is_some_and(|p| Arc::ptr_eq(p, plan)),
            "Driver preparation custody changed"
        );
        pending.take();
        Ok(())
    }
    pub(crate) fn publish_exact(
        &self,
        publication: &crate::state::DriverPublication,
    ) -> Result<()> {
        let (task, id, epoch, version, body) = publication.before();
        let (next_version, next_body) = publication.after();
        ensure!(
            task == self.slot.task
                && id == self.slot.id
                && epoch == self.slot.epoch
                && self.slot.active.load(Ordering::SeqCst)
                && self.slot.worker_entered.load(Ordering::SeqCst)
                && !self.slot.revoked.load(Ordering::SeqCst),
            "Driver publication no longer owns the actual worker"
        );
        ensure!(
            next_version
                == version
                    .checked_add(1)
                    .context("Driver publication version exhausted")?
                && next_body.len() <= 128 * 1024,
            "Driver publication image invalid"
        );
        let mut binding = self
            .slot
            .binding
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver binding poisoned"))?;
        // Only this same sealed publication's exact planned post-image may
        // reconcile an already published cache. The Store checked all captured
        // post-rows before creating it; no SQL/current-row recapture constructs
        // a publication or activates a stopped/replaced worker.
        if binding.0 == next_version && binding.1 == next_body {
            return Ok(());
        }
        ensure!(
            binding.0 == version && binding.1 == body,
            "Driver publication original cache differs"
        );
        // Store exclusion/control admission is held by the real publisher.
        // The new cached bytes were constructed and checked by its sealed plan.
        *binding = (next_version, next_body.to_owned());
        Ok(())
    }
}
impl WorkerLifetime {
    pub(crate) fn association(&self) -> Result<DriverAssociation> {
        ensure!(
            self.slot.active.load(Ordering::SeqCst) && !self.revoked(),
            "actual Driver association is not live"
        );
        Ok(DriverAssociation {
            registry: self.registry.clone(),
            slot: self.slot.clone(),
        })
    }
    pub(super) fn activate(
        &self,
        ack: StartupAcknowledgement,
        claim: &PendingDriverClaim,
    ) -> Result<()> {
        let (id, epoch, version, body) = claim.identity();
        ensure!(
            Arc::ptr_eq(&ack.slot, &self.slot)
                && self.slot.task == claim.task().id
                && self.slot.id == id
                && self.slot.epoch == epoch,
            "Driver startup acknowledgement differs"
        );
        ensure!(
            self.slot.worker_entered.load(Ordering::SeqCst)
                && !self.slot.revoked.load(Ordering::SeqCst),
            "Driver startup no longer live"
        );
        let binding = self
            .slot
            .binding
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver binding poisoned"))?;
        ensure!(
            binding.0 == version && binding.1 == body,
            "Driver initial binding differs"
        );
        // Caller holds service admission and SharedStore exclusion; SQL's exact
        // committed row/full original plan was checked immediately beforehand.
        self.slot.activated.store(true, Ordering::SeqCst);
        self.slot.active.store(true, Ordering::SeqCst);
        Ok(())
    }
    pub(super) fn revoked(&self) -> bool {
        self.slot.revoked.load(Ordering::SeqCst)
    }
    pub(super) async fn cancelled(&self) {
        loop {
            let notified = self.slot.cancel.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.revoked() {
                return;
            }
            notified.await;
        }
    }
}
impl DriverRegistry {
    pub(super) fn new(owner: &crate::execution::RuntimeOwner) -> Arc<Self> {
        Arc::new(Self {
            epoch: owner.epoch(),
            state_path: owner.state_path().to_path_buf(),
            instance: owner.instance_id().to_owned(),
            entries: Mutex::new(BTreeMap::new()),
            preparation_cursor: Mutex::new(None),
        })
    }
    pub(super) fn reserve_pending(
        self: &Arc<Self>,
        plan: &InitialDriverPlan,
    ) -> Result<PendingRegistration> {
        let (id, epoch, version, body) = plan.identity();
        ensure!(
            epoch == self.epoch
                && plan.owner_identity() == (self.state_path.as_path(), self.instance.as_str())
                && version == 1
                && body.len() <= 128 * 1024,
            "Driver pending identity invalid"
        );
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver registry poisoned"))?;
        ensure!(
            entries.len() < 4096 && !entries.contains_key(&plan.task()),
            "Driver slot held or capacity unavailable"
        );
        let slot = Arc::new(DriverSlot {
            task: plan.task(),
            id,
            epoch,
            binding: Mutex::new((version, body.to_owned())),
            active: AtomicBool::new(false),
            worker_entered: AtomicBool::new(false),
            activated: AtomicBool::new(false),
            initial_claim: Mutex::new(None),
            preparation: Mutex::new(None),
            revoked: AtomicBool::new(false),
            cancel: tokio::sync::Notify::new(),
            job: Mutex::new(None),
            exit: Mutex::new(None),
        });
        entries.insert(plan.task(), slot.clone());
        Ok(PendingRegistration {
            registry: self.clone(),
            slot,
        })
    }
    pub(crate) fn is_current(
        &self,
        task: TaskId,
        id: Uuid,
        epoch: u64,
        version: u64,
        body: &str,
    ) -> bool {
        if epoch != self.epoch {
            return false;
        }
        let Ok(entries) = self.entries.lock() else {
            return false;
        };
        let Some(slot) = entries.get(&task) else {
            return false;
        };
        if slot.id != id
            || slot.epoch != epoch
            || !slot.active.load(Ordering::SeqCst)
            || !slot.worker_entered.load(Ordering::SeqCst)
            || slot.revoked.load(Ordering::SeqCst)
        {
            return false;
        }
        slot.binding
            .lock()
            .is_ok_and(|b| b.0 == version && b.1 == body)
    }
    pub(super) fn stop_all(&self) {
        // Poison recovery is exclusively conservative revocation/custody.
        let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        for slot in entries.values() {
            slot.active.store(false, Ordering::SeqCst);
            slot.revoked.store(true, Ordering::SeqCst);
            slot.cancel.notify_waiters();
        }
    }
    /// Complete joins are observed, including panic/Err. Pending jobs keep their
    /// actual handles; a result label never retires job custody.
    pub(super) fn observe_finished(&self) -> Result<usize> {
        let slots: Vec<_> = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver registry poisoned"))?
            .values()
            .cloned()
            .collect();
        let mut pending = 0;
        for slot in slots {
            let mut job = slot
                .job
                .lock()
                .map_err(|_| anyhow::anyhow!("Driver job custody poisoned"))?;
            if let Some(handle) = job.as_mut() {
                let mut context = Context::from_waker(Waker::noop());
                match std::pin::Pin::new(handle).poll(&mut context) {
                    Poll::Pending => pending += 1,
                    Poll::Ready(actual_outcome) => {
                        let kind = match actual_outcome {
                            Ok(Ok(())) => DriverExitKind::Returned,
                            Ok(Err(_)) => DriverExitKind::Failed,
                            Err(error) if error.is_panic() => DriverExitKind::Panicked,
                            Err(_) => DriverExitKind::Aborted,
                        };
                        // Poison recovery preserves factual evidence/custody only.
                        *slot.exit.lock().unwrap_or_else(|e| e.into_inner()) = Some(kind);
                        job.take();
                        slot.active.store(false, Ordering::SeqCst);
                        slot.revoked.store(true, Ordering::SeqCst);
                    }
                }
            }
        }
        Ok(pending)
    }
    /// Clone only the actual retained advances; no registry lock crosses Store.
    pub(super) fn pending_preparations(&self) -> Result<Vec<Arc<DriverPreparationAdvance>>> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver registry poisoned"))?;
        let mut cursor = self
            .preparation_cursor
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver preparation cursor poisoned"))?;
        let mut plans = Vec::new();
        let mut last = None;
        // Bounded in-memory registry (4096) and rotating 64-plan page. A held
        // earliest plan cannot permanently starve a later actual producer.
        for (task, slot) in entries
            .iter()
            .filter(|(task, _)| cursor.is_none_or(|c| **task > c))
            .chain(
                entries
                    .iter()
                    .filter(|(task, _)| cursor.is_some_and(|c| **task <= c)),
            )
        {
            if let Some(plan) = slot
                .preparation
                .lock()
                .map_err(|_| anyhow::anyhow!("Driver preparation custody poisoned"))?
                .as_ref()
            {
                plans.push(plan.clone());
                last = Some(*task);
                if plans.len() == 64 {
                    break;
                }
            }
        }
        if let Some(last) = last {
            *cursor = Some(last);
        }
        Ok(plans)
    }
    /// Whole-registry observation after Store reconciliation, never a grant.
    pub(super) fn retained_preparations(&self) -> Result<usize> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver registry poisoned"))?;
        ensure!(entries.len() <= 4096, "Driver registry count exceeds bound");
        let mut count = 0;
        for slot in entries.values() {
            if slot
                .preparation
                .lock()
                .map_err(|_| anyhow::anyhow!("Driver preparation custody poisoned"))?
                .is_some()
            {
                count += 1;
            }
        }
        Ok(count)
    }
    pub(super) fn pending_exits(&self) -> Result<Vec<DriverExit>> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver registry poisoned"))?;
        let mut exits = Vec::new();
        for slot in entries.values() {
            if let Some(kind) = *slot.exit.lock().unwrap_or_else(|e| e.into_inner()) {
                exits.push(DriverExit {
                    task: slot.task,
                    id: slot.id,
                    epoch: slot.epoch,
                    kind,
                    never_activated: if !slot.activated.load(Ordering::SeqCst) {
                        slot.initial_claim
                            .lock()
                            .map_err(|_| anyhow::anyhow!("Driver initial claim custody poisoned"))?
                            .clone()
                    } else {
                        None
                    },
                });
                if exits.len() == 64 {
                    break;
                }
            }
        }
        Ok(exits)
    }
    pub(super) fn acknowledge_exit(
        &self,
        exit: &DriverExit,
        publication: &crate::state::DriverExitPublication,
    ) -> Result<()> {
        ensure!(
            publication.identity() == exit.identity(),
            "Driver exit publication differs"
        );
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("Driver registry poisoned"))?;
        let slot = entries
            .get(&exit.task)
            .ok_or_else(|| anyhow::anyhow!("Driver exit no longer retained"))?;
        ensure!(
            slot.id == exit.id && slot.epoch == exit.epoch,
            "Driver exit identity changed"
        );
        let mut pending = slot.exit.lock().unwrap_or_else(|e| e.into_inner());
        ensure!(
            *pending == Some(exit.kind),
            "Driver exit observation changed"
        );
        *pending = None;
        let remove = publication.initial_closed()
            && exit.never_activated.is_some()
            && slot
                .preparation
                .lock()
                .map_err(|_| anyhow::anyhow!("Driver preparation custody poisoned"))?
                .is_none();
        drop(pending);
        if remove {
            entries.remove(&exit.task);
        }
        Ok(())
    }
}
