use crate::{adapter::SharedStore,state::Store};
use anyhow::{Context,Result,ensure};
use std::{fs::{File,OpenOptions},path::{Path,PathBuf},sync::{Arc,Mutex}};
use std::os::unix::fs::OpenOptionsExt;

/// The kernel lock fences schedulers, never claims their descendants are dead.
pub struct RuntimeOwner {
    _lock:File,
    pub(crate) store:SharedStore,
    pub(crate) root:PathBuf,
    pub(crate) instance:String,
    pub(crate) epoch:u64,
}
impl RuntimeOwner {
    pub fn open(state:&Path) -> Result<Arc<Self>> {
        let path=if state.is_absolute(){state.to_path_buf()}else{std::env::current_dir()?.join(state)};
        let parent=path.parent().context("state needs parent")?;
        std::fs::create_dir_all(parent)?;
        let root=parent.join(format!("{}.execution",path.file_name().context("state needs file name")?.to_string_lossy()));
        std::fs::create_dir_all(&root)?;
        let root=root.canonicalize()?;
        let lock_path=root.join("owner.lock");
        ensure!(!std::fs::symlink_metadata(&lock_path).is_ok_and(|m|m.file_type().is_symlink()),"owner lock cannot be a symlink");
        let lock=OpenOptions::new().read(true).write(true).create(true).truncate(false).mode(0o600)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32).open(&lock_path)?;
        rustix::fs::flock(&lock,rustix::fs::FlockOperation::NonBlockingLockExclusive)
            .context("another Runtime owns this state root")?;
        // Rust File::open uses close-on-exec; verify rather than inheriting authority into a CLI.
        ensure!(rustix::io::fcntl_getfd(&lock)?.contains(rustix::io::FdFlags::CLOEXEC),"owner lock must be close-on-exec");
        let mut store=Store::open(&path)?;
        let (instance,epoch)=store.begin_execution_epoch()?;
        Ok(Arc::new(Self {_lock:lock,store:Arc::new(Mutex::new(store)),root,instance,epoch}))
    }
    pub fn store(&self) -> SharedStore {self.store.clone()}
    pub fn state_root(&self) -> &Path {&self.root}
    pub fn instance_id(&self) -> &str {&self.instance}
    pub fn epoch(&self) -> u64 {self.epoch}
}
