//! Mechanical custody only. No native workload, admission or settlement authority.
use super::{attempt::Control, protocol::failure};
use crate::adapter::{AdapterResult, ErrorKind};
use futures_util::FutureExt;
use serde::Serialize;
use std::{
    io::{self, Write},
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};
use tokio::sync::mpsc;
#[cfg(test)]
use tokio::sync::oneshot;
use uuid::Uuid;

const JOBS: usize = 3;
const LIMIT: usize = 64;
const REQUESTS: usize = 64;
const METADATA: usize = 4096;

pub(super) struct Pool(Mutex<PoolState>);
#[derive(Default)]
struct PoolState {
    used: usize,
    entries: Vec<Entry>,
}
struct Entry {
    _owner: Arc<Control>,
    inventory: Arc<Inventory>,
    creator: Option<JoinHandle<CreatorResult>>,
}
impl Pool {
    pub fn global() -> Arc<Self> {
        static POOL: OnceLock<Arc<Pool>> = OnceLock::new();
        POOL.get_or_init(|| Arc::new(Self(Mutex::new(PoolState::default()))))
            .clone()
    }
    #[cfg(test)]
    pub fn isolated() -> Arc<Self> {
        Arc::new(Self(Mutex::new(PoolState::default())))
    }
    pub fn reserve(self: &Arc<Self>, owner: &Arc<Control>) -> AdapterResult<Arc<Inventory>> {
        self.drain();
        let (sender, receiver) = mpsc::channel(REQUESTS);
        let inventory = Arc::new(Inventory {
            owner: Arc::downgrade(owner),
            generation: Uuid::new_v4(),
            sender,
            assets: Mutex::new(Assets::default()),
            accepted: AtomicUsize::new(0),
            inflight: AtomicUsize::new(0),
            effects: AtomicUsize::new(0),
            unknown: AtomicBool::new(false),
            worker_uncertain: AtomicBool::new(false),
            joined: AtomicBool::new(false),
            created: AtomicBool::new(false),
            endpoint_taken: AtomicBool::new(false),
            open: Mutex::new(true),
            #[cfg(test)]
            paused: AtomicBool::new(false),
        });
        {
            let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
            if state.used.checked_add(JOBS).is_none_or(|n| n > LIMIT) {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "preparation job capacity exhausted",
                ));
            }
            state.used += JOBS;
            state.entries.push(Entry {
                _owner: owner.clone(),
                inventory: inventory.clone(),
                creator: None,
            });
        }
        let (begin, begun) = std::sync::mpsc::channel();
        let observed = inventory.clone();
        let retained_owner = owner.clone();
        let spawned = std::thread::Builder::new()
            .name("rrx-preparation-custody".into())
            .spawn(move || {
                let _owner = retained_owner;
                if begun.recv().is_err() {
                    observed.unknown.store(true, Ordering::SeqCst);
                }
                run(&observed, receiver)
            });
        match spawned {
            Ok(handle) => {
                {
                    let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
                    let entry = state
                        .entries
                        .iter_mut()
                        .find(|e| Arc::ptr_eq(&e.inventory, &inventory))
                        .expect("reserved inventory retained");
                    entry.creator = Some(handle);
                }
                let _ = begin.send(());
                Ok(inventory)
            }
            Err(_) => {
                // Opaque creation failure never proves absence. The complete
                // declaration remains charged; no actor/worker is attempted.
                inventory.unknown.store(true, Ordering::SeqCst);
                Err(failure(
                    ErrorKind::SessionLost,
                    "preparation custodian creation unverified",
                ))
            }
        }
    }
    pub fn drain(&self) {
        let finished: Vec<_> = {
            let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
            state
                .entries
                .iter_mut()
                .filter_map(|entry| {
                    if entry.creator.as_ref().is_some_and(JoinHandle::is_finished) {
                        Some((
                            entry.inventory.clone(),
                            entry.creator.take().expect("finished handle"),
                        ))
                    } else {
                        None
                    }
                })
                .collect()
        };
        for (inventory, creator) in finished {
            let result = creator.join(); // never under pool/inventory/registry locks
            let released = match result {
                Ok(result) => {
                    let n = 1
                        + usize::from(result.actor_joined)
                        + usize::from(matches!(
                            result.worker,
                            WorkerDisposition::Joined | WorkerDisposition::NotCreated
                        ));
                    let complete = n == JOBS
                        && !inventory.unknown.load(Ordering::SeqCst)
                        && inventory.accepted.load(Ordering::SeqCst) == 0
                        && inventory.inflight.load(Ordering::SeqCst) == 0;
                    inventory.joined.store(complete, Ordering::SeqCst);
                    n
                }
                Err(_) => {
                    inventory.unknown.store(true, Ordering::SeqCst);
                    0
                }
            };
            let retired = {
                let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
                state.used = state
                    .used
                    .checked_sub(released)
                    .expect("exact joined reservations");
                if inventory.joined.load(Ordering::SeqCst) {
                    let index = state
                        .entries
                        .iter()
                        .position(|e| Arc::ptr_eq(&e.inventory, &inventory))
                        .expect("creator has one reservation");
                    Some(state.entries.remove(index))
                } else {
                    None
                }
            };
            drop(retired); // Control/future destructors cannot reenter a held lock
        }
    }
    #[cfg(test)]
    pub fn used(&self) -> usize {
        self.0.lock().unwrap().used
    }
}

#[derive(Default)]
struct Assets {
    actor: Option<tokio::task::JoinHandle<()>>,
    actor_installed: bool,
    worker: Option<JoinHandle<WorkerResult>>,
    worker_attempted: bool,
}
pub(super) struct Inventory {
    owner: Weak<Control>,
    generation: Uuid,
    sender: mpsc::Sender<Request>,
    assets: Mutex<Assets>,
    accepted: AtomicUsize,
    inflight: AtomicUsize,
    effects: AtomicUsize,
    unknown: AtomicBool,
    worker_uncertain: AtomicBool,
    joined: AtomicBool,
    created: AtomicBool,
    endpoint_taken: AtomicBool,
    open: Mutex<bool>,
    #[cfg(test)]
    paused: AtomicBool,
}
impl Inventory {
    pub fn install_actor(&self, actor: tokio::task::JoinHandle<()>) {
        let mut assets = self.assets.lock().unwrap_or_else(|e| e.into_inner());
        assert!(!assets.actor_installed);
        assets.actor = Some(actor);
        assets.actor_installed = true;
    }
    pub fn outstanding(&self) -> bool {
        self.effects.load(Ordering::SeqCst) != 0
            || self.accepted.load(Ordering::SeqCst) != 0
            || self.inflight.load(Ordering::SeqCst) != 0
    }
    pub fn joined(&self) -> bool {
        self.joined.load(Ordering::SeqCst)
    }
    pub fn created(&self) -> bool {
        self.created.load(Ordering::SeqCst)
    }
    pub fn unknown(&self) -> bool {
        self.unknown.load(Ordering::SeqCst)
    }
    pub fn endpoint(self: &Arc<Self>) -> AdapterResult<Endpoint> {
        self.endpoint_taken
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| {
                failure(
                    ErrorKind::StateConflict,
                    "preparation endpoint already owned",
                )
            })?;
        Ok(Endpoint {
            inventory: self.clone(),
            generation: self.generation,
        })
    }
    #[cfg(test)]
    pub async fn create(&self, spec: FileSpec) -> AdapterResult<()> {
        #[derive(Serialize)]
        struct CreateMeta<'a> {
            kind: &'static str,
            path: &'a std::path::Path,
        }
        let _metadata = encoded_metadata(&CreateMeta {
            kind: "create",
            path: &spec.path,
        })?;
        let (reply, response) = oneshot::channel();
        let owner = self
            .owner
            .upgrade()
            .ok_or_else(|| failure(ErrorKind::SessionLost, "preparation owner unavailable"))?;
        {
            let open = self.open.lock().unwrap_or_else(|e| e.into_inner());
            if !*open || owner.jobs_revoked() {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "preparation effects revoked",
                ));
            }
            self.accepted.fetch_add(1, Ordering::SeqCst);
            if self.sender.try_send(Request::Create(spec, reply)).is_err() {
                self.accepted.fetch_sub(1, Ordering::SeqCst);
                return Err(failure(ErrorKind::StateConflict, "preparation inbox full"));
            }
        }
        response
            .await
            .map_err(|_| failure(ErrorKind::SessionLost, "preparation creator lost"))?
    }
    #[cfg(test)]
    pub fn pause(&self, paused: bool) {
        self.paused.store(paused, Ordering::SeqCst);
    }
    #[cfg(test)]
    pub fn accepted(&self) -> usize {
        self.accepted.load(Ordering::SeqCst)
    }
    #[cfg(test)]
    pub fn worker_joined(&self) -> bool {
        self.created()
            && self.effects.load(Ordering::SeqCst) == 0
            && self.assets.lock().unwrap().worker.is_none()
    }
}

pub(super) struct Endpoint {
    inventory: Arc<Inventory>,
    generation: Uuid,
}
impl Endpoint {
    pub fn try_note(&self, text: &str) -> AdapterResult<()> {
        #[derive(Serialize)]
        struct Note<'a> {
            kind: &'static str,
            text: &'a str,
        }
        let bytes = encoded_metadata(&Note { kind: "note", text })?;
        let owner = self
            .inventory
            .owner
            .upgrade()
            .ok_or_else(|| failure(ErrorKind::SessionLost, "preparation owner unavailable"))?;
        let open = self
            .inventory
            .open
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if !*open || self.generation != self.inventory.generation || owner.jobs_revoked() {
            return Err(failure(
                ErrorKind::StateConflict,
                "preparation endpoint revoked or foreign",
            ));
        }
        self.inventory.accepted.fetch_add(1, Ordering::SeqCst);
        if self
            .inventory
            .sender
            .try_send(Request::Note(self.generation, bytes))
            .is_err()
        {
            self.inventory.accepted.fetch_sub(1, Ordering::SeqCst);
            return Err(failure(ErrorKind::StateConflict, "preparation inbox full"));
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn corrupt_generation(&mut self) {
        self.generation = Uuid::new_v4();
    }
}
fn encoded_metadata(value: &impl Serialize) -> AdapterResult<Vec<u8>> {
    struct Bounded(Vec<u8>);
    impl Write for Bounded {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self
                .0
                .len()
                .checked_add(bytes.len())
                .is_none_or(|n| n > METADATA)
            {
                return Err(io::Error::other("preparation metadata exceeds4096 bytes"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut bytes = Bounded(Vec::new());
    serde_json::to_writer(&mut bytes, value).map_err(|_| {
        failure(
            ErrorKind::InvalidInput,
            "preparation metadata exceeds4096 bytes",
        )
    })?;
    Ok(bytes.0)
}
enum Request {
    Note(Uuid, Vec<u8>),
    #[cfg(test)]
    Create(FileSpec, oneshot::Sender<AdapterResult<()>>),
}
#[derive(Clone, Copy)]
enum WorkerDisposition {
    Joined,
    NotCreated,
    Unknown,
}
struct CreatorResult {
    actor_joined: bool,
    worker: WorkerDisposition,
}
type WorkerResult = ();

fn run(inventory: &Arc<Inventory>, mut receiver: mpsc::Receiver<Request>) -> CreatorResult {
    let mut actor = None;
    let mut worker = None;
    loop {
        for _ in 0..REQUESTS {
            #[cfg(test)]
            if inventory.paused.load(Ordering::SeqCst) {
                break;
            }
            let Ok(request) = receiver.try_recv() else {
                break;
            };
            inventory.inflight.fetch_add(1, Ordering::SeqCst);
            let handled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let owner = inventory.owner.upgrade();
                let permitted = owner.as_ref().is_some_and(|o| !o.jobs_revoked());
                match request {
                    Request::Note(generation, bytes) => {
                        if permitted
                            && generation == inventory.generation
                            && bytes.len() <= METADATA
                            && let Some(owner) = owner
                        {
                            owner.record_mechanical_note();
                        }
                    }
                    #[cfg(test)]
                    Request::Create(spec, reply) => {
                        let result = if permitted {
                            create_worker(inventory, spec)
                        } else {
                            Err(failure(
                                ErrorKind::StateConflict,
                                "preparation effects revoked",
                            ))
                        };
                        let _ = reply.send(result);
                    }
                }
            }));
            if handled.is_err() {
                inventory.unknown.store(true, Ordering::SeqCst);
                inventory.worker_uncertain.store(true, Ordering::SeqCst);
            }
            inventory.inflight.fetch_sub(1, Ordering::SeqCst);
            inventory.accepted.fetch_sub(1, Ordering::SeqCst);
        }
        let (actor_handle, worker_handle, attempted) = {
            let mut assets = inventory.assets.lock().unwrap_or_else(|e| e.into_inner());
            let a = if assets
                .actor
                .as_ref()
                .is_some_and(tokio::task::JoinHandle::is_finished)
            {
                assets.actor.take()
            } else {
                None
            };
            let w = if assets.worker.as_ref().is_some_and(JoinHandle::is_finished) {
                assets.worker.take()
            } else {
                None
            };
            (a, w, assets.worker_attempted)
        };
        if let Some(handle) = actor_handle {
            actor = Some(matches!(handle.now_or_never(), Some(Ok(()))));
            if actor == Some(false) {
                inventory.unknown.store(true, Ordering::SeqCst);
            }
        }
        if let Some(handle) = worker_handle {
            worker = Some(match handle.join() {
                Ok(_) => {
                    inventory.effects.store(0, Ordering::SeqCst);
                    if inventory.worker_uncertain.load(Ordering::SeqCst) {
                        WorkerDisposition::Unknown
                    } else {
                        WorkerDisposition::Joined
                    }
                }
                Err(_) => {
                    inventory.unknown.store(true, Ordering::SeqCst);
                    WorkerDisposition::Unknown
                }
            });
        }
        if let Some(actor_joined) = actor
            && inventory.accepted.load(Ordering::SeqCst) == 0
            && inventory.inflight.load(Ordering::SeqCst) == 0
            && (worker.is_some() || !attempted || inventory.unknown.load(Ordering::SeqCst))
        {
            let mut open = inventory.open.lock().unwrap_or_else(|e| e.into_inner());
            if inventory.accepted.load(Ordering::SeqCst) == 0 {
                *open = false;
                return CreatorResult {
                    actor_joined,
                    worker: worker.unwrap_or(if attempted {
                        WorkerDisposition::Unknown
                    } else {
                        WorkerDisposition::NotCreated
                    }),
                };
            }
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[cfg(test)]
pub(super) struct FileSpec {
    pub path: std::path::PathBuf,
    pub started: Arc<AtomicBool>,
    pub release: std::sync::mpsc::Receiver<()>,
    pub panic: bool,
    pub panic_actor: bool,
    pub panic_before_begin: bool,
}
#[cfg(test)]
fn create_worker(inventory: &Arc<Inventory>, spec: FileSpec) -> AdapterResult<()> {
    use std::fs::File;
    let panic_before_begin = spec.panic_before_begin;
    let (begin, begun) = std::sync::mpsc::channel();
    {
        let mut assets = inventory.assets.lock().unwrap_or_else(|e| e.into_inner());
        if assets.worker_attempted {
            return Err(failure(
                ErrorKind::StateConflict,
                "preparation worker already attempted",
            ));
        }
        assets.worker_attempted = true;
    }
    let worker = std::thread::Builder::new()
        .name("rrx-closed-file-worker".into())
        .spawn(move || {
            if begun.recv().is_err() {
                return;
            }
            let mut file = File::create(&spec.path).expect("owned fixture file");
            file.write_all(b"owned").expect("owned fixture write");
            spec.started.store(true, Ordering::SeqCst);
            // The closed fixture self expires; a failed assertion cannot orphan a
            // thread/file indefinitely. This is not a native cleanup deadline.
            let _ = spec.release.recv_timeout(Duration::from_secs(5));
            assert!(!spec.panic, "injected closed worker panic");
            drop(file);
        });
    match worker {
        Ok(handle) => {
            {
                let mut assets = inventory.assets.lock().unwrap_or_else(|e| e.into_inner());
                assets.worker = Some(handle);
            }
            inventory.created.store(true, Ordering::SeqCst);
            inventory.effects.store(1, Ordering::SeqCst);
            assert!(
                !panic_before_begin,
                "injected creator callback panic before Begin"
            );
            let _ = begin.send(());
            Ok(())
        }
        Err(_) => {
            inventory.unknown.store(true, Ordering::SeqCst);
            Err(failure(
                ErrorKind::SessionLost,
                "preparation worker creation unverified",
            ))
        }
    }
}

#[cfg(test)]
pub(super) struct Factory {
    pub pool: Arc<Pool>,
    pending: Mutex<Option<FileSpec>>,
}
#[cfg(test)]
impl Factory {
    pub fn new(pool: Arc<Pool>) -> Arc<Self> {
        Arc::new(Self {
            pool,
            pending: Mutex::new(None),
        })
    }
    pub fn enqueue(&self, spec: FileSpec) {
        assert!(self.pending.lock().unwrap().replace(spec).is_none());
    }
    pub async fn run(&self, control: &Arc<Control>) -> crate::adapter::AdapterError {
        let spec = self.pending.lock().unwrap().take();
        if let Some(spec) = spec {
            let panic_actor = spec.panic_actor;
            let result = control
                .custody()
                .expect("declared fixture custody")
                .create(spec)
                .await;
            if let Err(error) = result {
                return error;
            }
            assert!(!panic_actor, "injected actor panic after resource creation");
        }
        failure(
            ErrorKind::StateConflict,
            "injected closed fixture context refusal",
        )
    }
}
