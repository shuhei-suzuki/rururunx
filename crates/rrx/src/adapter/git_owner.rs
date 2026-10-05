//! Selected Git calls have an independent, finite process-local owner.
//! Four slots account for the supervisor, two readers and the native worker.
//! This is not a durable operation capability or complete native containment.
use super::*;
use std::{
    ffi::OsString,
    io,
    panic::{AssertUnwindSafe, catch_unwind},
    process::{Child as StdChild, Command as StdCommand, ExitStatus},
    sync::{OnceLock, mpsc as std_mpsc},
    thread,
};
use tokio::{runtime::Runtime, sync::{Notify, oneshot}};

const JOBS: usize = 4;
const CAPACITY: usize = 64;
const WINDOW: Duration = Duration::from_millis(250);
#[cfg(test)]
static TEST_POOLS: OnceLock<Mutex<Vec<Arc<GitPool>>>> = OnceLock::new();

#[cfg(test)]
mod tests;

/// Retention is explicit. No Drop implementation releases capacity or native assets.
struct PoolState { records: Vec<Arc<OpRecord>> }
pub(super) struct GitPool { state: Mutex<PoolState>, available: Notify }
impl GitPool {
    fn new() -> Arc<Self> {
        Arc::new(Self { state: Mutex::new(PoolState { records: Vec::with_capacity(CAPACITY / JOBS) }), available: Notify::new() })
    }
    async fn run(self: &Arc<Self>, request: Request, flag: Arc<AtomicBool>, context: Context) -> AdapterResult<Vec<u8>> {
        let mut guard = CallGuard { record: None, returned: false };
        let record = loop {
            // Expiry precedes capacity and setup; no new uncertainty on refusal.
            if tokio::time::Instant::now() >= request.deadline {
                return Err(error(ErrorKind::Timeout, "Git ownership preflight timed out"));
            }
            let waiter = self.available.notified();
            tokio::pin!(waiter);
            waiter.as_mut().enable();
            let admitted = {
                let mut pool = self.state.lock().map_err(|_| error(ErrorKind::LaunchFailure, "Git owner admission unavailable"))?;
                if pool.records.len() < CAPACITY / JOBS {
                    let record = Arc::new(OpRecord::new(flag.clone(), context.clone()));
                    pool.records.push(record.clone());
                    Some(record)
                } else { None }
            };
            if let Some(record) = admitted {
                // No await between allocation and start: cancellation cannot strand
                // an unstarted reservation. FrameReady follows storing the actual handle.
                guard.record = Some(record.clone());
                self.start(record.clone(), request)?;
                break record;
            }
            tokio::time::timeout_at(request.deadline, waiter).await
                .map_err(|_| error(ErrorKind::Timeout, "Git ownership preflight timed out"))?;
        };
        loop {
            let waiter = record.ticket.result_wake.notified();
            tokio::pin!(waiter);
            waiter.as_mut().enable();
            let result = {
                let mut publication = record.ticket.publication.lock().unwrap_or_else(|poison| poison.into_inner());
                if let Some(outcome) = publication.result.take() {
                    // The live return is the ONLY settled flag clear. Terminal loss,
                    // cancellation or late settlement cannot clear a frozen result.
                    if outcome.settled && publication.state != State::FrozenUnknown {
                        record.ticket.flag.store(false, Ordering::SeqCst);
                        publication.state = State::ReturnedSettled;
                    } else if publication.state != State::CancelledBeforeSpawn {
                        publication.state = State::FrozenUnknown;
                    }
                    Some(outcome.primary)
                } else { None }
            };
            if let Some(result) = result {
                guard.returned = true;
                return result;
            }
            waiter.await;
        }
    }
    fn start(self: &Arc<Self>, record: Arc<OpRecord>, request: Request) -> AdapterResult<()> {
        let (ready, began) = std_mpsc::sync_channel(1);
        let owner = self.clone();
        let thread_record = record.clone();
        let handle = thread::Builder::new().name("rrx-git-owner".into()).spawn(move || {
            let mut liveness = FrameGuard { ticket: thread_record.ticket.clone(), armed: true };
            if began.recv().is_err() {
                thread_record.ticket.owner_loss();
                return;
            }
            let completed = catch_unwind(AssertUnwindSafe(|| supervise(&thread_record, request)));
            match completed {
                Ok(Completion::Settled(outcome)) => {
                    // Inner future is gone, native worker joined and vaults emptied
                    // before publishing/releasing. No native Drop follows release.
                    thread_record.ticket.publish(outcome);
                    owner.release(&thread_record);
                    liveness.armed = false;
                }
                Ok(Completion::Retained(outcome)) => {
                    thread_record.ticket.publish(outcome);
                    liveness.armed = false;
                }
                Err(_) => thread_record.ticket.owner_loss(),
            }
        });
        match handle {
            Ok(handle) => {
                *record.supervisor.lock().unwrap_or_else(|p| p.into_inner()) = Some(handle);
                if ready.send(()).is_err() {
                    record.ticket.owner_loss();
                }
                Ok(())
            }
            Err(_) => {
                // Explicit no-job release; no Git attempt or supervisor exists.
                self.release(&record);
                Err(error(ErrorKind::LaunchFailure, "Git owner thread unavailable"))
            }
        }
    }
    fn release(&self, record: &Arc<OpRecord>) {
        // Bookkeeping poison after actual settlement holds slots without fabricating
        // native Unknown. Caller flag/result remain governed by own settlement.
        if let Ok(mut pool) = self.state.lock() {
            pool.records.retain(|entry| !Arc::ptr_eq(entry, record));
            self.available.notify_waiters();
        }
    }
}
fn production_pool() -> Arc<GitPool> {
    static POOL: OnceLock<Arc<GitPool>> = OnceLock::new();
    POOL.get_or_init(GitPool::new).clone()
}

#[derive(Clone, Default)]
struct Context {
    #[cfg(all(test, target_os = "macos"))]
    plan: Option<ProcessInspectionPlan>,
    #[cfg(test)]
    hooks: TestHooks,
}
#[cfg(test)]
#[derive(Clone, Default)]
pub(crate) struct TestGitContext { pool: Option<Arc<GitPool>>, context: Context }
#[cfg(test)]
impl TestGitContext {
    pub(crate) fn isolated() -> Self {
        let pool = GitPool::new();
        let mut inventory = TEST_POOLS.get_or_init(|| Mutex::new(Vec::with_capacity(256))).lock().unwrap();
        assert!(inventory.len() < 256, "finite test owner inventory exhausted");
        inventory.push(pool.clone());
        Self { pool: Some(pool), context: Context::default() }
    }
    #[cfg(target_os = "macos")]
    pub(crate) fn with_plan(plan: ProcessInspectionPlan) -> Self {
        let mut context = Self::isolated(); context.context.plan = Some(plan); context
    }
}
#[cfg(test)]
#[derive(Clone, Default)]
struct TestHooks {
    initialized_error: bool,
    worker_panic: bool,
    supervisor_panic: bool,
}
struct Request { executable: PathBuf, cwd: PathBuf, args: Vec<String>, environment: Vec<(OsString, OsString)>, deadline: tokio::time::Instant }
pub(super) async fn run(
    executable: &Path, cwd: &Path, args: &[String], environment: Vec<(OsString, OsString)>,
    deadline: tokio::time::Instant, flag: Arc<AtomicBool>,
    #[cfg(test)] test: Option<TestGitContext>,
) -> AdapterResult<Vec<u8>> {
    #[cfg(test)]
    let (pool, context) = match test { Some(test) => (test.pool.unwrap_or_else(production_pool), test.context), None => (production_pool(), Context::default()) };
    #[cfg(not(test))]
    let (pool, context) = (production_pool(), Context::default());
    pool.run(Request { executable: executable.to_owned(), cwd: cwd.to_owned(), args: args.to_vec(), environment, deadline }, flag, context).await
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State { Admitted, CancelledBeforeSpawn, Spawning, Live, FrozenUnknown, ReturnedSettled }
struct Publication { state: State, result: Option<Outcome>, primary: Option<AdapterError> }
struct Outcome { primary: AdapterResult<Vec<u8>>, settled: bool }
struct Ticket {
    publication: Mutex<Publication>, flag: Arc<AtomicBool>, cancel: AtomicBool, lost: AtomicBool,
    result_wake: Notify, cancel_wake: Notify, commands: OnceLock<std_mpsc::SyncSender<NativeCommand>>,
}
impl Ticket {
    fn command(&self, command: NativeCommand) {
        if let Some(sender) = self.commands.get() { let _ = sender.try_send(command); }
    }
    fn cancel(&self) {
        if !self.cancel.swap(true, Ordering::SeqCst) {
            let mut publication = self.publication.lock().unwrap_or_else(|p| p.into_inner());
            match publication.state {
                State::Admitted => publication.state = State::CancelledBeforeSpawn,
                State::Spawning | State::Live => publication.state = State::FrozenUnknown,
                _ => {},
            }
            self.command(NativeCommand::Cancel);
            self.cancel_wake.notify_one();
        }
    }
    fn owner_loss(&self) {
        if !self.lost.swap(true, Ordering::SeqCst) {
            let poisoned = self.publication.is_poisoned();
            let mut publication = self.publication.lock().unwrap_or_else(|p| p.into_inner());
            if publication.result.is_none() && !matches!(publication.state, State::FrozenUnknown | State::ReturnedSettled | State::CancelledBeforeSpawn) {
                let before = publication.state == State::Admitted && !poisoned;
                publication.state = if before { State::CancelledBeforeSpawn } else { State::FrozenUnknown };
                if poisoned { self.flag.store(true, Ordering::SeqCst); }
                let primary = publication.primary.take().unwrap_or_else(|| error(if before { ErrorKind::LaunchFailure } else { ErrorKind::SessionLost }, "Git owner execution lost"));
                publication.result = Some(Outcome { primary: Err(primary), settled: false });
            }
            self.command(NativeCommand::Lost);
            self.cancel_wake.notify_one();
        }
        self.result_wake.notify_one();
    }
    fn establish(&self, primary: &AdapterError) {
        let mut publication = self.publication.lock().unwrap_or_else(|p| p.into_inner());
        if publication.primary.is_none() { publication.primary = Some(error(primary.kind, primary.message.clone())); }
    }
    fn publish(&self, outcome: Outcome) {
        let mut publication = self.publication.lock().unwrap_or_else(|p| p.into_inner());
        if publication.result.is_none() && publication.state != State::ReturnedSettled {
            if !outcome.settled && publication.state != State::CancelledBeforeSpawn { publication.state = State::FrozenUnknown; }
            publication.result = Some(outcome);
        }
        self.result_wake.notify_one();
    }
}
struct CallGuard { record: Option<Arc<OpRecord>>, returned: bool }
impl Drop for CallGuard { fn drop(&mut self) { if !self.returned && let Some(record) = &self.record { record.ticket.cancel(); } } }
struct FrameGuard { ticket: Arc<Ticket>, armed: bool }
impl Drop for FrameGuard { fn drop(&mut self) { if self.armed { self.ticket.owner_loss(); } } }

/// Vaults own resources before effects; frames borrow them and cannot drop them on unwind.
struct OpRecord {
    ticket: Arc<Ticket>, context: Context,
    supervisor: Mutex<Option<thread::JoinHandle<()>>>, worker: Mutex<Option<thread::JoinHandle<()>>>,
    runtime: Mutex<Option<Runtime>>, readers: Mutex<Readers>, native: Mutex<NativeAssets>,
    native_settled: AtomicBool, native_terminal_error: AtomicBool,
}
impl OpRecord {
    fn new(flag: Arc<AtomicBool>, context: Context) -> Self {
        Self { ticket: Arc::new(Ticket { publication: Mutex::new(Publication { state: State::Admitted, result: None, primary: None }), flag, cancel: AtomicBool::new(false), lost: AtomicBool::new(false), result_wake: Notify::new(), cancel_wake: Notify::new(), commands: OnceLock::new() }), context,
            supervisor: Mutex::new(None), worker: Mutex::new(None), runtime: Mutex::new(None), readers: Mutex::new(Readers::default()), native: Mutex::new(NativeAssets::default()), native_settled: AtomicBool::new(false), native_terminal_error: AtomicBool::new(false) }
    }
}
#[derive(Default)]
struct NativeAssets { child: Option<StdChild>, group: Option<Pid>, signal_issued: bool, cleanup: Option<io::Result<()>>, reaped: bool }
#[derive(Default)]
struct Readers { stdout: Option<JoinHandle<AdapterResult<Vec<u8>>>>, stderr: Option<JoinHandle<AdapterResult<Vec<u8>>>>, stdout_joined: bool, stderr_joined: bool }
impl Readers {
    fn abort(&self) { if let Some(task) = &self.stdout { task.abort(); } if let Some(task) = &self.stderr { task.abort(); } }
    fn settled(&self) -> bool { (self.stdout.is_none() || self.stdout_joined) && (self.stderr.is_none() || self.stderr_joined) }
    async fn join_stdout(&mut self) -> AdapterResult<Vec<u8>> {
        let Some(handle) = self.stdout.as_mut() else { return Ok(Vec::new()); };
        let result = handle.await; self.stdout_joined = true;
        result.map_err(|e| error(ErrorKind::ProcessFailure, e.to_string()))?
    }
    async fn join_stderr(&mut self) -> AdapterResult<()> {
        let Some(handle) = self.stderr.as_mut() else { return Ok(()); };
        let result = handle.await; self.stderr_joined = true;
        result.map_err(|e| error(ErrorKind::ProcessFailure, e.to_string()))?.map(|_| ())
    }
    async fn finish_joins(&mut self) {
        if !self.stdout_joined { let _ = self.join_stdout().await; }
        if !self.stderr_joined { let _ = self.join_stderr().await; }
    }
}
enum NativeCommand { AuthorizeSpawn, CleanupAndReap, Cancel, Lost }
struct Stages { spawned: oneshot::Sender<AdapterResult<Option<Pid>>>, cleanup: oneshot::Sender<io::Result<()>>, reaped: oneshot::Sender<io::Result<ExitStatus>> }
enum Completion { Settled(Outcome), Retained(Outcome) }
fn supervisor_lost() -> AdapterError { error(ErrorKind::SessionLost, "Git owner execution lost") }

fn supervise(record: &Arc<OpRecord>, request: Request) -> Completion {
    let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(_) => return Completion::Settled(Outcome { primary: Err(error(ErrorKind::LaunchFailure, "Git owner runtime unavailable")), settled: true }),
    };
    *record.runtime.lock().unwrap() = Some(runtime);
    // The actual Runtime stays inside the preallocated vault across inner unwind.
    let mut runtime_vault = record.runtime.lock().unwrap();
    let runtime = runtime_vault.as_ref().unwrap();
    let signals = {
        let _entered = runtime.enter();
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::child())
    };
    let mut signals = match signals {
        Ok(signals) => signals,
        Err(_) => {
            drop(runtime_vault.take());
            return Completion::Settled(Outcome { primary: Err(error(ErrorKind::LaunchFailure, "Git owner signal setup unavailable")), settled: true });
        }
    };
    let (commands, receive) = std_mpsc::sync_channel(4);
    let _ = record.ticket.commands.set(commands);
    let (spawn_sender, spawn_receiver) = oneshot::channel();
    let (cleanup_sender, cleanup_receiver) = oneshot::channel();
    let (reap_sender, reap_receiver) = oneshot::channel();
    let worker_record = record.clone();
    let deadline = request.deadline;
    let worker = thread::Builder::new().name("rrx-git-native".into()).spawn(move || {
        // These Senders belong to this frame, never the retained vault. Unwind
        // disconnects pending stages and wakes the independent supervisor.
        native_worker(&worker_record, request, receive, Stages { spawned: spawn_sender, cleanup: cleanup_sender, reaped: reap_sender });
    });
    let worker = match worker {
        Ok(handle) => handle,
        Err(_) => {
            drop(signals); drop(runtime_vault.take());
            return Completion::Settled(Outcome { primary: Err(error(ErrorKind::LaunchFailure, "Git native worker unavailable")), settled: true });
        }
    };
    *record.worker.lock().unwrap() = Some(worker);
    {
        let mut publication = record.ticket.publication.lock().unwrap();
        if publication.state == State::Admitted && !record.ticket.cancel.load(Ordering::SeqCst) {
            publication.state = State::Spawning;
            record.ticket.flag.store(true, Ordering::SeqCst);
            record.ticket.command(NativeCommand::AuthorizeSpawn);
        } else { record.ticket.command(NativeCommand::Cancel); }
    }
    let mut readers = record.readers.lock().unwrap();
    let outcome = runtime.block_on(supervisor_work(record, deadline, &mut signals, &mut readers, spawn_receiver, cleanup_receiver, reap_receiver));
    if !outcome.settled {
        // Publish the frozen caller outcome before late observations. Same frame,
        // same handles/wait; no replacement observer or flag clear is permitted.
        record.ticket.publish(Outcome { primary: outcome.primary.as_ref().map(Clone::clone).map_err(|e| error(e.kind, e.message.clone())), settled: false });
    }
    let terminal_native = record.native_settled.load(Ordering::SeqCst);
    if !terminal_native {
        return Completion::Retained(outcome);
    }
    if !readers.settled() {
        runtime.block_on(readers.finish_joins());
    }
    if !readers.settled() { return Completion::Retained(outcome); }
    // Joining the actual worker, not receiving its result, precedes success.
    let worker = record.worker.lock().unwrap().take().unwrap();
    if worker.join().is_err() { record.ticket.owner_loss(); return Completion::Retained(Outcome { primary: Err(supervisor_lost()), settled: false }); }
    // Opaque spawn Err remains terminal Unknown even with measured no handle.
    if record.ticket.lost.load(Ordering::SeqCst) || record.native_terminal_error.load(Ordering::SeqCst) {
        return Completion::Retained(outcome);
    }
    let ambiguous_spawn = record.native.lock().unwrap().child.is_none() && record.ticket.flag.load(Ordering::SeqCst) && matches!(outcome.primary.as_ref().err().map(|e| e.kind), Some(ErrorKind::ProcessFailure));
    if ambiguous_spawn { return Completion::Retained(outcome); }
    *readers = Readers::default();
    drop(readers); drop(signals);
    record.native.lock().unwrap().child.take();
    drop(runtime_vault.take());
    // Dropping the stored supervisor handle detaches only its nonblocking epilogue,
    // after all work/assets are destroyed; no resource settlement is inferred.
    record.supervisor.lock().unwrap().take();
    Completion::Settled(outcome)
}

async fn supervisor_work(
    record: &OpRecord, deadline: tokio::time::Instant, signals: &mut tokio::signal::unix::Signal,
    readers: &mut Readers, spawned: oneshot::Receiver<AdapterResult<Option<Pid>>>,
    cleanup: oneshot::Receiver<io::Result<()>>, mut reaped: oneshot::Receiver<io::Result<ExitStatus>>,
) -> Outcome {
    let pid = match spawned.await {
        Ok(Ok(Some(pid))) => pid,
        Ok(Ok(None)) => return Outcome { primary: Err(error(ErrorKind::LaunchFailure, "Git launch cancelled before spawn")), settled: true },
        Ok(Err(primary)) => return Outcome { primary: Err(primary), settled: false },
        Err(_) => { record.ticket.owner_loss(); return Outcome { primary: Err(supervisor_lost()), settled: false }; }
    };
    { let mut publication = record.ticket.publication.lock().unwrap(); if publication.state == State::Spawning { publication.state = State::Live; } }
    let initialized = {
        let _entered = tokio::runtime::Handle::current().enter();
        let mut native = record.native.lock().unwrap();
        let child = native.child.as_mut().unwrap();
        let stdout = child.stdout.take(); let stderr = child.stderr.take();
        // Child remains anchored in NativeAssets during fallible registrations.
        (|| -> AdapterResult<()> {
            #[cfg(test)]
            if record.context.hooks.initialized_error { return Err(error(ErrorKind::LaunchFailure, "Git pipe initialization failed")); }
            let stdout = stdout.ok_or_else(|| error(ErrorKind::LaunchFailure, "Git stdout unavailable"))?;
            let stderr = stderr.ok_or_else(|| error(ErrorKind::LaunchFailure, "Git stderr unavailable"))?;
            let stdout = tokio::process::ChildStdout::from_std(stdout).map_err(|_| error(ErrorKind::LaunchFailure, "Git stdout registration failed"))?;
            let stderr = tokio::process::ChildStderr::from_std(stderr).map_err(|_| error(ErrorKind::LaunchFailure, "Git stderr registration failed"))?;
            readers.stdout = Some(tokio::spawn(read_git_output(stdout)));
            readers.stderr = Some(tokio::spawn(read_git_output(stderr)));
            Ok(())
        })()
    };
    #[cfg(test)]
    if record.context.hooks.supervisor_panic { panic!("synthetic Git supervisor inner panic"); }
    let observed = if initialized.is_ok() {
        tokio::select! {
            result = tokio::time::timeout_at(deadline, observe(pid, signals)) => match result {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => Err(error(ErrorKind::SessionLost, format!("Git child observation failed: {e}"))),
                Err(_) => Err(error(ErrorKind::Timeout, "Git ownership preflight timed out")),
            },
            _ = cancelled(&record.ticket) => Err(error(ErrorKind::Timeout, "Git ownership preflight timed out")),
        }
    } else { initialized };
    // Observe future is destroyed before authorizing actual owning Child::wait.
    record.ticket.command(NativeCommand::CleanupAndReap);
    let cleanup = match cleanup.await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(error(ErrorKind::SessionLost, format!("native process group cleanup failed: {e}"))),
        Err(_) => { record.ticket.owner_loss(); Err(supervisor_lost()) },
    };
    let mut reap_pending = false;
    let mut primary = match cleanup { Err(e) => Err(e), Ok(()) => match tokio::time::timeout(WINDOW, &mut reaped).await {
        Ok(Ok(Ok(exit))) => Ok(exit),
        Ok(Ok(Err(e))) => Err(error(ErrorKind::SessionLost, format!("Git child reap failed: {e}"))),
        Ok(Err(_)) => { record.ticket.owner_loss(); Err(supervisor_lost()) },
        Err(_) => { reap_pending = true; Err(error(ErrorKind::SessionLost, "Git child death not confirmed after cleanup")) },
    }};
    if primary.is_ok() && let Err(e) = observed { primary = Err(e); }
    if let Err(e) = &primary { record.ticket.establish(e); readers.abort(); }
    let output_deadline = tokio::time::Instant::now() + WINDOW;
    let output = tokio::time::timeout_at(output_deadline, async {
        let stdout = readers.join_stdout().await;
        if stdout.is_err() { readers.abort(); }
        let stderr = readers.join_stderr().await;
        stdout.and_then(|out| stderr.map(|_| out))
    }).await;
    if output.is_err() { readers.abort(); }
    let output = output.unwrap_or_else(|_| Err(error(ErrorKind::ProcessFailure, "Git output remained open after cleanup")));
    let primary = match primary {
        Err(e) => Err(e),
        Ok(exit) => match output { Err(e) => Err(e), Ok(output) if exit.success() => Ok(output), Ok(_) => Err(error(ErrorKind::OwnershipMismatch, "Git ownership preflight failed")) },
    };
    if let Err(e) = &primary { record.ticket.establish(e); }
    // A deadline return is frozen before awaiting the same outstanding wait late.
    let settled = record.native_settled.load(Ordering::SeqCst) && readers.settled();
    if !settled {
        record.ticket.publish(Outcome { primary: primary.as_ref().map(Clone::clone).map_err(|e| error(e.kind, e.message.clone())), settled: false });
        // Only an outstanding original wait can be observed late. Returned errors
        // and cleanup loss never authorize a repeated wait or replacement worker.
        if reap_pending { let _ = reaped.await; }
    }
    Outcome { primary, settled }
}
async fn cancelled(ticket: &Ticket) {
    loop {
        let wake = ticket.cancel_wake.notified(); tokio::pin!(wake); wake.as_mut().enable();
        if ticket.cancel.load(Ordering::SeqCst) || ticket.lost.load(Ordering::SeqCst) { return; }
        wake.await;
    }
}
async fn observe(pid: Pid, signals: &mut tokio::signal::unix::Signal) -> io::Result<()> {
    loop {
        match waitid(WaitId::Pid(pid), WaitIdOptions::EXITED | WaitIdOptions::NOWAIT | WaitIdOptions::NOHANG) {
            Ok(Some(_)) => return Ok(()), Ok(None) => {}, Err(rustix::io::Errno::INTR) => continue, Err(e) => return Err(e.into()),
        }
        if signals.recv().await.is_none() { return Err(io::Error::other("SIGCHLD observer closed")); }
    }
}

fn native_worker(record: &OpRecord, request: Request, receiver: std_mpsc::Receiver<NativeCommand>, stages: Stages) {
    let mut spawned = Some(stages.spawned); let mut cleanup = Some(stages.cleanup); let mut reaped = Some(stages.reaped);
    let result = catch_unwind(AssertUnwindSafe(|| {
        loop {
            let Ok(command) = receiver.recv() else { return; }; // Not a relied-on loss mechanism.
            match command {
                NativeCommand::AuthorizeSpawn => {
                    if record.ticket.cancel.load(Ordering::SeqCst) || record.ticket.lost.load(Ordering::SeqCst) {
                        record.native_settled.store(true, Ordering::SeqCst);
                        if let Some(sender) = spawned.take() { let _ = sender.send(Ok(None)); } return;
                    }
                    let mut command = StdCommand::new(&request.executable);
                    command.args(&request.args).current_dir(&request.cwd).env_clear().envs(&request.environment)
                        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).process_group(0);
                    let mut native = record.native.lock().unwrap();
                    let child = command.spawn();
                    let result = match child {
                        Ok(child) => {
                            native.child = Some(child); // Anchor before binding/stdio/wrapping.
                            let raw = native.child.as_ref().unwrap().id();
                            let pid = if raw > 1 && raw <= i32::MAX as u32 { Pid::from_raw(raw as i32) } else { None };
                            native.group = pid;
                            pid.map(Some).ok_or_else(|| error(ErrorKind::LaunchFailure, "invalid native PID"))
                        }
                        Err(e) => Err(error(ErrorKind::ProcessFailure, e.to_string())),
                    };
                    let failed = result.is_err();
                    if let Some(sender) = spawned.take() { let _ = sender.send(result); }
                    if failed { return; }
                }
                NativeCommand::Cancel | NativeCommand::Lost => {
                    let mut native = record.native.lock().unwrap();
                    if native.child.is_none() {
                        record.native_settled.store(true, Ordering::SeqCst);
                        if let Some(sender) = spawned.take() { let _ = sender.send(Ok(None)); } return;
                    }
                    first_cleanup(record, &mut native);
                    if record.ticket.lost.load(Ordering::SeqCst) { return; }
                }
                NativeCommand::CleanupAndReap => {
                    let mut native = record.native.lock().unwrap();
                    first_cleanup(record, &mut native);
                    let result = native.cleanup.as_ref().unwrap();
                    let succeeded = result.is_ok();
                    if let Some(sender) = cleanup.take() { let _ = sender.send(match result { Ok(()) => Ok(()), Err(e) => Err(io::Error::new(e.kind(), e.to_string())) }); }
                    if !succeeded { record.native_terminal_error.store(true, Ordering::SeqCst); return; }
                    let result = native.child.as_mut().unwrap().wait();
                    if result.is_ok() { native.reaped = true; record.native_settled.store(true, Ordering::SeqCst); } else { record.native_terminal_error.store(true, Ordering::SeqCst); }
                    if let Some(sender) = reaped.take() { let _ = sender.send(result); }
                    return;
                }
            }
        }
    }));
    if result.is_err() {
        // Retain actual anchor after panic. First cleanup only if never attempted;
        // the bit is stored BEFORE KILL/inspection and prevents implicit retry.
        let mut native = record.native.lock().unwrap_or_else(|p| p.into_inner());
        if native.child.is_some() && !native.signal_issued { first_cleanup(record, &mut native); }
        record.ticket.owner_loss();
    }
}
fn first_cleanup(_record: &OpRecord, native: &mut NativeAssets) {
    if native.signal_issued { return; }
    native.signal_issued = true;
    #[cfg(test)]
    if _record.context.hooks.worker_panic { panic!("synthetic Git native worker panic"); }
    let result = match native.group {
        None => Err(io::Error::other("Git child group binding unavailable")),
        Some(pid) => {
            #[cfg(all(test, target_os = "macos"))]
            let signal = match &_record.context.plan { Some(plan) => plan.signal(pid), None => kill_process_group(pid, Signal::KILL) };
            #[cfg(not(all(test, target_os = "macos")))]
            let signal = kill_process_group(pid, Signal::KILL);
            #[cfg(target_os = "macos")]
            let result = resolve_macos_signal_result(signal, || {
                #[cfg(test)]
                if let Some(plan) = &_record.context.plan && let Some(observation) = plan.inspect(pid.as_raw_nonzero().get()) { return observation; }
                macos_group_is_dead(pid)
            });
            #[cfg(not(target_os = "macos"))]
            let result = match signal { Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()), Err(e) => Err(e.into()) };
            result
        }
    };
    if result.is_ok() { native.group = None; }
    native.cleanup = Some(result);
}
