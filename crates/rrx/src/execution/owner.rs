use super::{OperationId, UnitId};
use crate::{adapter::SharedStore, state::Store};
use anyhow::{Context, Result, ensure};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Weak},
};

/// The kernel lock fences schedulers, never claims their descendants are dead.
pub struct RuntimeOwner {
    _lock: File,
    _database_file: File,
    state_path: PathBuf,
    pub(crate) store: SharedStore,
    pub(crate) root: PathBuf,
    pub(crate) instance: String,
    pub(crate) epoch: u64,
    pub(crate) git_gate: Arc<tokio::sync::Mutex<()>>,
    /// Serialize cooperative namespace selection through durable lease admission;
    /// never held while materializing files, awaiting Git or running native work.
    pub(crate) resource_admission: Arc<tokio::sync::Mutex<()>>,
    git_leases: Mutex<BTreeMap<OperationId, Weak<GitLease>>>,
    runtime_drivers: Mutex<Weak<crate::runtime::driver::DriverRegistry>>,
    _ipc: tempfile::TempDir,
    pub(crate) socket: PathBuf,
}

/// Reentry belongs to one live command chain and one unit. Children retain the
/// root lock even if the original command returns before their receipts arrive.
pub(crate) struct GitLease {
    pub(crate) id: OperationId,
    unit: UnitId,
    _guard: tokio::sync::OwnedMutexGuard<()>,
}

/// Covers async preparation abandonment; it never adopts another Session or
/// disposes inputs. Child handles own their independent best-effort stop.
pub(crate) struct PreparationGuard {
    owner: Arc<RuntimeOwner>,
    unit: super::ExecutionUnit,
    armed: bool,
    unmarked_retirement: bool,
}
impl PreparationGuard {
    /// Nongrant identity inspection for transfer to the actual Runtime queue.
    pub(crate) fn matches(
        &self,
        owner: &Arc<RuntimeOwner>,
        unit: &super::ExecutionUnit,
    ) -> Result<bool> {
        Ok(self.armed
            && Arc::ptr_eq(&self.owner, owner)
            && serde_json::to_value(&self.unit)? == serde_json::to_value(unit)?)
    }

    pub(crate) fn new(owner: Arc<RuntimeOwner>, unit: &super::ExecutionUnit) -> Self {
        Self {
            owner,
            unit: unit.clone(),
            armed: true,
            unmarked_retirement: true,
        }
    }
    /// Retention policy only: this does not authorize a helper, input or close.
    /// A publishing slot may have committed even when its caller disappeared.
    pub(crate) fn hold_marker_publication(&mut self) {
        self.unmarked_retirement = false;
    }
    /// Only the actual supervisor's unpublished-marker transaction may restore
    /// this policy. It must exclude a concurrent publisher and check the ledger.
    pub(crate) fn restore_unmarked_retirement(&mut self) {
        self.unmarked_retirement = true;
    }
    pub(crate) fn update(&mut self, unit: &super::ExecutionUnit) {
        self.unit = unit.clone();
    }
    pub(crate) fn disarm(&mut self) {
        self.armed = false;
    }
}
impl Drop for PreparationGuard {
    fn drop(&mut self) {
        if !self.armed || !self.unmarked_retirement {
            return;
        }
        if let Ok(mut store) = self.owner.store.lock()
            && let Ok(current) = store.execution_unit(self.unit.id)
            && current.scope == self.unit.scope
            && current.generation == self.unit.generation
            && current.owner_epoch == self.unit.owner_epoch
            && current.session_id == self.unit.session_id
            && current.work.is_none()
            && current.native_effects_open
        {
            let _ =
                store.retire_execution_as(&current.authority(), false, super::Disposition::Lost);
            if let Some(id) = current.session_id
                && let Ok(Some((mut session, version))) = store.session(id)
            {
                session.state = crate::domain::SessionState::Lost;
                let _ = store.close_execution_session(current.id, &session, version);
            }
        }
    }
}

/// A dropped helper wait has an uncertain outcome, never a replayable intent.
pub(crate) struct HelperGuard {
    owner: Arc<RuntimeOwner>,
    operation: OperationId,
    armed: bool,
}
impl HelperGuard {
    pub(crate) fn new(owner: Arc<RuntimeOwner>, operation: OperationId) -> Self {
        Self {
            owner,
            operation,
            armed: true,
        }
    }
    pub(crate) fn disarm(&mut self) {
        self.armed = false;
    }
}
impl Drop for HelperGuard {
    fn drop(&mut self) {
        if self.armed
            && let Ok(mut store) = self.owner.store.lock()
        {
            let _ = store.reconcile_managed_effect(
                self.operation,
                1,
                super::EffectState::Unknown,
                BTreeMap::from([("transport".into(), "helper_wait_abandoned".into())]),
            );
        }
    }
}

impl RuntimeOwner {
    pub fn open(state: &Path) -> Result<Arc<Self>> {
        let path = if state.is_absolute() {
            state.to_path_buf()
        } else {
            std::env::current_dir()?.join(state)
        };
        let parent = path.parent().context("state needs parent")?;
        std::fs::create_dir_all(parent)?;
        // SQLite's own locking must not be shadowed by a flock on its DB inode.
        // Canonicalize symlinks and refuse unsupported hardlink aliases instead.
        let database_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(&path)?;
        ensure!(
            database_file.metadata()?.nlink() == 1,
            "hardlinked state databases are unsupported"
        );
        ensure!(
            rustix::io::fcntl_getfd(&database_file)?.contains(rustix::io::FdFlags::CLOEXEC),
            "database descriptor must be close-on-exec"
        );
        let path = path.canonicalize()?;
        let parent = path.parent().context("canonical state needs parent")?;
        let root = parent.join(format!(
            "{}.execution",
            path.file_name()
                .context("state needs file name")?
                .to_string_lossy()
        ));
        std::fs::create_dir_all(&root)?;
        let root = root.canonicalize()?;
        let lock_path = root.join("owner.lock");
        ensure!(
            !std::fs::symlink_metadata(&lock_path).is_ok_and(|m| m.file_type().is_symlink()),
            "owner lock cannot be a symlink"
        );
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(&lock_path)?;
        rustix::fs::flock(&lock, rustix::fs::FlockOperation::NonBlockingLockExclusive)
            .context("another Runtime owns this state root")?;
        // Rust File::open uses close-on-exec; verify rather than inheriting authority into a CLI.
        ensure!(
            rustix::io::fcntl_getfd(&lock)?.contains(rustix::io::FdFlags::CLOEXEC),
            "owner lock must be close-on-exec"
        );
        ensure!(
            database_file.metadata()?.nlink() == 1,
            "state database alias changed"
        );
        let mut store = Store::open(&path)?;
        let (instance, epoch) = store.begin_execution_epoch()?;
        let ipc = tempfile::Builder::new().prefix("rrx-").tempdir_in("/tmp")?;
        let socket = ipc.path().canonicalize()?.join("runtime.sock");
        ensure!(
            socket.as_os_str().len() < 100,
            "Runtime IPC path exceeds Unix socket limit"
        );
        Ok(Arc::new(Self {
            _lock: lock,
            _database_file: database_file,
            state_path: path,
            store: Arc::new(Mutex::new(store)),
            root,
            instance,
            epoch,
            git_gate: Arc::new(tokio::sync::Mutex::new(())),
            resource_admission: Arc::new(tokio::sync::Mutex::new(())),
            git_leases: Mutex::new(BTreeMap::new()),
            runtime_drivers: Mutex::new(Weak::new()),
            _ipc: ipc,
            socket,
        }))
    }
    pub fn store(&self) -> SharedStore {
        self.store.clone()
    }
    pub(crate) fn attach_runtime_drivers(
        &self,
        registry: &Arc<crate::runtime::driver::DriverRegistry>,
    ) -> Result<()> {
        {
            let mut retained = self
                .runtime_drivers
                .lock()
                .map_err(|_| anyhow::anyhow!("Runtime association poisoned"))?;
            ensure!(
                retained.upgrade().is_none(),
                "one Runtime already owns this service"
            );
            *retained = Arc::downgrade(registry);
        }
        // Association and registry locks are released before SharedStore.
        self.store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .attach_runtime_drivers(registry)
    }
    /// Canonical database selected by the retained owner, for control identity.
    pub fn state_path(&self) -> &Path {
        &self.state_path
    }
    pub fn state_root(&self) -> &Path {
        &self.root
    }
    pub fn instance_id(&self) -> &str {
        &self.instance
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn ipc_path(&self) -> &Path {
        &self.socket
    }
    pub(crate) async fn git_lease(
        &self,
        unit: UnitId,
        parent: Option<OperationId>,
    ) -> Result<Arc<GitLease>> {
        if let Some(parent) = parent {
            let leases = self
                .git_leases
                .lock()
                .map_err(|_| anyhow::anyhow!("Git lease state poisoned"))?;
            let lease = leases
                .get(&parent)
                .and_then(Weak::upgrade)
                .context("Git reentry root retired")?;
            ensure!(lease.unit == unit, "Git reentry belongs to another unit");
            return Ok(lease);
        }
        let guard = self.git_gate.clone().lock_owned().await;
        let lease = Arc::new(GitLease {
            id: OperationId::new(),
            unit,
            _guard: guard,
        });
        let mut leases = self
            .git_leases
            .lock()
            .map_err(|_| anyhow::anyhow!("Git lease state poisoned"))?;
        leases.retain(|_, lease| lease.strong_count() > 0);
        leases.insert(lease.id, Arc::downgrade(&lease));
        Ok(lease)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn database_aliases_cannot_create_a_second_owner_or_advance_epoch() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join("state.db");
        let owner = RuntimeOwner::open(&state).unwrap();
        let symlink = dir.path().join("alias.db");
        std::os::unix::fs::symlink(&state, &symlink).unwrap();
        let hardlink = dir.path().join("hardlink.db");
        std::fs::hard_link(&state, &hardlink).unwrap();
        for path in [&state, &symlink, &hardlink] {
            assert!(RuntimeOwner::open(path).is_err());
        }
        let epoch: u64 = owner.store.lock().unwrap().connection_epoch_for_test();
        assert_eq!(epoch, owner.epoch());
        std::fs::remove_file(&hardlink).unwrap();
        drop(owner);
        let successor = RuntimeOwner::open(&state).unwrap();
        assert_eq!(successor.epoch(), epoch + 1);
    }
}
