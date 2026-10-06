//! Retained registry boundary. Native Driver registration is unavailable until
//! the actual marker/binding producer composes; durable JSON cannot populate it.
use crate::domain::TaskId;
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use uuid::Uuid;

pub(crate) struct DriverRegistry {
    epoch: u64,
    entries: Mutex<BTreeMap<TaskId, Arc<DriverSlot>>>,
}
struct DriverSlot {
    id: Uuid,
    epoch: u64,
    binding: Mutex<(u64, String)>,
    revoked: AtomicBool,
}
impl DriverRegistry {
    pub(super) fn new(epoch: u64) -> Arc<Self> {
        Arc::new(Self {
            epoch,
            entries: Mutex::new(BTreeMap::new()),
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
        if slot.id != id || slot.epoch != epoch || slot.revoked.load(Ordering::SeqCst) {
            return false;
        }
        slot.binding
            .lock()
            .is_ok_and(|binding| binding.0 == version && binding.1 == body)
    }
}
