//! Thin native process contracts. Context selection and workflow decisions belong upstream.
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    process::Stdio,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use rustix::process::{Pid, Signal, WaitId, WaitIdOptions, kill_process_group, waitid};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::os::unix::process::CommandExt;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::{Child, Command},
    sync::{mpsc, watch},
    task::JoinHandle,
};

use crate::{
    config::Config,
    domain::{
        Project, ProjectState, RecordKind, Scope, Session, SessionId, SessionRole, SessionState,
        Usage, now_ms,
    },
    state::Store,
};

pub type SharedStore = Arc<Mutex<Store>>;

pub type AdapterResult<T> = Result<T, AdapterError>;
pub type AdapterFuture<'a, T> = Pin<Box<dyn Future<Output = AdapterResult<T>> + Send + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Execute,
    Consult,
    Review,
    InspectDiff,
    InspectCommand,
    Interactive,
    NonInteractive,
    Attach,
    Resume,
    PermissionInterception,
    StructuredOutput,
    UsageTelemetry,
    PromptCacheTelemetry,
    ContextCheckpoint,
    NativeGoal,
    NativeGoalStatus,
    NativeGoalResume,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub agent: String,
    pub provider: String,
    pub adapter_version: String,
    pub executable: PathBuf,
    /// None means authentication was not observable; no authentication guess is made.
    pub authenticated: Option<bool>,
    pub model_configuration: bool,
    pub effort_configuration: bool,
    pub capabilities: BTreeSet<Capability>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    UnsupportedCapability,
    ExecutableMissing,
    InvalidConfiguration,
    InvalidInput,
    OwnershipMismatch,
    StateFailure,
    StateConflict,
    Locked,
    AuthenticationUnavailable,
    ParseFailure,
    LaunchFailure,
    SessionLost,
    ProcessFailure,
    Timeout,
}

#[derive(Debug)]
pub struct AdapterError {
    pub kind: ErrorKind,
    pub message: String,
}
impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}
impl std::error::Error for AdapterError {}
fn error(kind: ErrorKind, message: impl Into<String>) -> AdapterError {
    AdapterError {
        kind,
        message: message.into(),
    }
}
fn unsupported(capability: Capability) -> AdapterError {
    error(
        ErrorKind::UnsupportedCapability,
        format!("unsupported capability {capability:?}"),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    Interactive,
    NonInteractive,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    ContextPack,
    ReviewBundle,
}

/// An already-prepared factual payload. Adapters neither retrieve nor condense context.
#[derive(Debug, Clone)]
pub struct PreparedInput {
    pub scope: Scope,
    pub kind: InputKind,
    pub revision: String,
    pub version: u64,
    pub source_versions: BTreeMap<String, String>,
    pub payload: String,
}

#[derive(Debug, Clone)]
pub struct LaunchRequest {
    pub project: Project,
    pub scope: Scope,
    pub worktree: PathBuf,
    pub role: SessionRole,
    pub mode: LaunchMode,
    pub input: PreparedInput,
    /// Explicit environment only, including HOME/PATH/config/credential variables when needed.
    /// Values stay in memory; Session/Usage/audit snapshots do not contain them.
    pub environment: BTreeMap<String, String>,
    pub model: Option<String>,
    pub effort: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SessionRef {
    pub id: SessionId,
    pub scope: Scope,
}
impl From<&Session> for SessionRef {
    fn from(session: &Session) -> Self {
        Self {
            id: session.id,
            scope: session.scope.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NativeGoalRef {
    pub scope: Scope,
    pub session_id: Option<SessionId>,
    pub native_ref: String,
}

#[derive(Debug, Clone)]
pub struct SessionStatus {
    pub session: Session,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub failure: Option<String>,
}
impl SessionStatus {
    pub fn terminal(&self) -> bool {
        matches!(
            self.session.state,
            SessionState::Exited
                | SessionState::Failed
                | SessionState::Stopped
                | SessionState::Lost
        )
    }
}

/// Native session handles belong to the provider; attach and resume must preserve its semantics.
pub trait AgentAdapter: Send + Sync {
    fn capabilities(&self) -> BTreeSet<Capability>;
    fn probe(&self) -> AdapterResult<AgentInfo>;
    fn start(&self, request: LaunchRequest) -> AdapterFuture<'_, Session>;
    fn status(&self, session: SessionRef) -> AdapterFuture<'_, SessionStatus>;
    fn stop(&self, session: SessionRef) -> AdapterFuture<'_, SessionStatus>;
    fn attach(&self, session: SessionRef) -> AdapterFuture<'_, ()>;
    fn resume(&self, session: SessionRef) -> AdapterFuture<'_, Session>;
    /// Release collected terminal output; running sessions must remain supervised.
    fn release(&self, session: SessionRef) -> AdapterResult<()>;
    fn subscribe(&self, session: SessionRef) -> AdapterResult<watch::Receiver<SessionStatus>>;
    fn usage(
        &self,
        session: SessionRef,
        phase: String,
        review_round: Option<u32>,
    ) -> AdapterFuture<'_, Usage>;
    fn submit_approval(&self, _session: SessionRef, _decision: Value) -> AdapterFuture<'_, ()> {
        Box::pin(async { Err(unsupported(Capability::PermissionInterception)) })
    }
    fn checkpoint(&self, _session: SessionRef, _input: PreparedInput) -> AdapterFuture<'_, ()> {
        Box::pin(async { Err(unsupported(Capability::ContextCheckpoint)) })
    }
    fn start_native_goal(&self, _input: PreparedInput) -> AdapterFuture<'_, NativeGoalRef> {
        Box::pin(async { Err(unsupported(Capability::NativeGoal)) })
    }
    fn native_goal_status(&self, _native_ref: NativeGoalRef) -> AdapterFuture<'_, Value> {
        Box::pin(async { Err(unsupported(Capability::NativeGoalStatus)) })
    }
    fn resume_native_goal(
        &self,
        _native_ref: NativeGoalRef,
        _input: PreparedInput,
    ) -> AdapterFuture<'_, NativeGoalRef> {
        Box::pin(async { Err(unsupported(Capability::NativeGoalResume)) })
    }
}

/// The runtime selects adapters. This baseline never infers provider capabilities from a name.
#[derive(Default)]
pub struct AgentRegistry {
    adapters: BTreeMap<String, Arc<dyn AgentAdapter>>,
}
impl AgentRegistry {
    pub fn register(&mut self, name: String, adapter: Arc<dyn AgentAdapter>) -> AdapterResult<()> {
        if name.trim().is_empty() || self.adapters.contains_key(&name) {
            return Err(error(
                ErrorKind::InvalidConfiguration,
                "agent name is blank or already registered",
            ));
        }
        self.adapters.insert(name, adapter);
        Ok(())
    }
    pub fn from_config(config: &Config, store: SharedStore) -> AdapterResult<Self> {
        config
            .validate()
            .map_err(|e| error(ErrorKind::InvalidConfiguration, e.to_string()))?;
        let mut registry = Self::default();
        for (name, agent) in &config.agents {
            if agent.model.is_some() || agent.effort.is_some() {
                return Err(error(
                    ErrorKind::InvalidConfiguration,
                    "generic adapters cannot apply model/effort settings; use a native adapter",
                ));
            }
            registry.register(
                name.clone(),
                Arc::new(GenericCliAdapter::new(
                    name.clone(),
                    agent.command.clone(),
                    store.clone(),
                )?),
            )?;
        }
        Ok(registry)
    }
    pub fn get(&self, name: &str) -> AdapterResult<Arc<dyn AgentAdapter>> {
        self.adapters.get(name).cloned().ok_or_else(|| {
            error(
                ErrorKind::InvalidConfiguration,
                format!("unregistered agent {name}"),
            )
        })
    }
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.adapters.keys().map(String::as_str)
    }
}

struct Entry {
    scope: Scope,
    status: watch::Receiver<SessionStatus>,
    stop: mpsc::Sender<()>,
}
struct Reservation {
    store: SharedStore,
    session: Option<(Session, u64)>,
    process_uncertain: Arc<AtomicBool>,
}

/// Owns the unreaped leader so PGID cannot be recycled before group cleanup.
struct ProcessGroup {
    child: Child,
    pid: Pid,
    group_owned: bool,
    process_uncertain: Arc<AtomicBool>,
}
impl ProcessGroup {
    fn new(child: Child, process_uncertain: Arc<AtomicBool>) -> AdapterResult<Self> {
        let raw = child
            .id()
            .filter(|pid| *pid > 1)
            .ok_or_else(|| error(ErrorKind::LaunchFailure, "child has no valid owned PID"))?;
        let pid = Pid::from_raw(raw as i32)
            .ok_or_else(|| error(ErrorKind::LaunchFailure, "invalid native PID"))?;
        process_uncertain.store(true, Ordering::SeqCst);
        Ok(Self {
            child,
            pid,
            group_owned: true,
            process_uncertain,
        })
    }
    async fn observe_exit(&self) -> std::io::Result<()> {
        let mut signals = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::child())?;
        loop {
            match waitid(
                WaitId::Pid(self.pid),
                WaitIdOptions::EXITED | WaitIdOptions::NOWAIT | WaitIdOptions::NOHANG,
            ) {
                Ok(Some(_)) => return Ok(()),
                Ok(None) => {}
                Err(rustix::io::Errno::INTR) => continue,
                Err(e) => return Err(e.into()),
            }
            if signals.recv().await.is_none() {
                return Err(std::io::Error::other("SIGCHLD observer closed"));
            }
        }
    }
    fn kill_group(&mut self) -> std::io::Result<()> {
        let result = kill_process_group(self.pid, Signal::KILL);
        #[cfg(target_os = "macos")]
        let result = resolve_macos_signal_result(result, || macos_group_is_dead(self.pid));
        #[cfg(not(target_os = "macos"))]
        let result = match result {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            Err(e) => Err(e.into()),
        };
        if result.is_ok() {
            self.group_owned = false;
            self.process_uncertain.store(false, Ordering::SeqCst);
        }
        result
    }
    async fn reap(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.child.wait().await
    }
}

#[cfg(target_os = "macos")]
fn macos_group_is_dead(pid: Pid) -> std::io::Result<bool> {
    let output = std::process::Command::new("/bin/ps")
        .args(["-axo", "pgid=,stat="])
        .env_clear()
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(
            "cannot verify owned process group death",
        ));
    }
    process_group_is_dead(&output.stdout, pid.as_raw_nonzero().get())
}
#[cfg(target_os = "macos")]
fn resolve_macos_signal_result(
    result: Result<(), rustix::io::Errno>,
    inspection: impl FnOnce() -> std::io::Result<bool>,
) -> std::io::Result<()> {
    match result {
        Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
        Err(rustix::io::Errno::PERM) if inspection()? => Ok(()),
        Err(e) => Err(e.into()),
    }
}
#[cfg(target_os = "macos")]
fn process_group_is_dead(output: &[u8], pid: i32) -> std::io::Result<bool> {
    let text = std::str::from_utf8(output).map_err(std::io::Error::other)?;
    for row in text.lines().filter(|row| !row.trim().is_empty()) {
        let mut fields = row.split_whitespace();
        let group = fields
            .next()
            .and_then(|v| v.parse::<i32>().ok())
            .ok_or_else(|| std::io::Error::other("invalid process-group inspection"))?;
        let state = fields
            .next()
            .ok_or_else(|| std::io::Error::other("missing process-group state"))?;
        if group == pid && !state.starts_with('Z') {
            return Ok(false);
        }
    }
    Ok(true)
}

impl Drop for ProcessGroup {
    fn drop(&mut self) {
        if self.group_owned {
            let _ = self.kill_group();
        }
        // Child's kill_on_drop/reaper follows group termination, including Tokio shutdown.
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if let Some((mut session, version)) = self.session.take() {
            session.state = if self.process_uncertain.load(Ordering::SeqCst) {
                SessionState::Lost
            } else {
                SessionState::Failed
            };
            let _ = save_session(&self.store, &session, version);
        }
    }
}
pub struct GenericCliAdapter {
    agent: String,
    command: Vec<String>,
    store: SharedStore,
    git_executable: Option<PathBuf>,
    sessions: Mutex<HashMap<SessionId, Entry>>,
}
const OUTPUT_LIMIT: usize = 64 * 1024;
const TERMINAL_RETENTION_LIMIT: usize = 32;
impl GenericCliAdapter {
    pub fn new(agent: String, command: Vec<String>, store: SharedStore) -> AdapterResult<Self> {
        if agent.trim().is_empty()
            || command.first().is_none_or(|s| s.trim().is_empty())
            || command.iter().any(|s| s.contains('\0'))
        {
            return Err(error(
                ErrorKind::InvalidConfiguration,
                "generic command requires an executable and valid argv",
            ));
        }
        Ok(Self {
            agent,
            command,
            store,
            git_executable: None,
            sessions: Mutex::new(HashMap::new()),
        })
    }
    fn entry(
        &self,
        reference: &SessionRef,
    ) -> AdapterResult<(watch::Receiver<SessionStatus>, mpsc::Sender<()>)> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| error(ErrorKind::ProcessFailure, "session registry poisoned"))?;
        let entry = sessions.get(&reference.id).ok_or_else(|| {
            error(
                ErrorKind::SessionLost,
                "session has no owned live supervisor; PID hints cannot reconnect it",
            )
        })?;
        if entry.scope != reference.scope {
            return Err(error(
                ErrorKind::OwnershipMismatch,
                "session project/goal/task mismatch",
            ));
        }
        Ok((entry.status.clone(), entry.stop.clone()))
    }
}

impl AgentAdapter for GenericCliAdapter {
    fn capabilities(&self) -> BTreeSet<Capability> {
        [Capability::Execute, Capability::NonInteractive].into()
    }
    fn probe(&self) -> AdapterResult<AgentInfo> {
        let executable = resolve_executable(&self.command[0])?;
        Ok(AgentInfo {
            agent: self.agent.clone(),
            provider: "generic-cli".into(),
            adapter_version: env!("CARGO_PKG_VERSION").into(),
            executable,
            authenticated: None,
            model_configuration: false,
            effort_configuration: false,
            capabilities: self.capabilities(),
        })
    }
    fn start(&self, request: LaunchRequest) -> AdapterFuture<'_, Session> {
        Box::pin(async move {
            if request.mode == LaunchMode::Interactive {
                return Err(unsupported(Capability::Interactive));
            }
            if request.model.is_some() || request.effort.is_some() {
                return Err(error(
                    ErrorKind::InvalidConfiguration,
                    "generic adapter does not implement model/effort configuration",
                ));
            }
            if request.role != SessionRole::Executor {
                return Err(unsupported(if request.role == SessionRole::Consultant {
                    Capability::Consult
                } else {
                    Capability::Review
                }));
            }
            let worktree = validate_request(&request)?;
            validate_persisted(&self.store, &request, &worktree, &self.agent)?;
            ensure_unlocked(&self.store, &request.scope)?;
            let executable = self.probe()?.executable;
            let mut session = Session {
                id: SessionId::new(),
                scope: request.scope.clone(),
                agent: self.agent.clone(),
                provider: "generic-cli".into(),
                role: request.role,
                native_ref: None,
                pid: None,
                worktree: worktree.clone(),
                state: SessionState::Starting,
                model: None,
                effort: None,
                recovery: json!({"reconnect_supported":false,"project_root":request.project.root,"input_revision":request.input.revision,"input_version":request.input.version,"source_versions":request.input.source_versions,"input_bytes":request.input.payload.len()}),
                started_at: now_ms(),
            };
            // Reserve before asynchronous preflight. Store's task lock/session exclusion is
            // authoritative; recovery must explicitly resolve a stale reservation.
            let mut version = save_session(&self.store, &session, 0)?;
            let mut reservation = Reservation {
                store: self.store.clone(),
                session: Some((session.clone(), version)),
                process_uncertain: Arc::new(AtomicBool::new(false)),
            };
            let launch = async {
                ensure_unlocked(&self.store, &request.scope)?;
                validate_git(
                    &self.store,
                    &request,
                    &worktree,
                    self.git_executable.as_deref(),
                    reservation.process_uncertain.clone(),
                )
                .await?;
                let mut command = Command::new(executable);
                command
                    .args(&self.command[1..])
                    .current_dir(&worktree)
                    .env_clear()
                    .envs(&request.environment)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .kill_on_drop(true);
                // PTY handling is a separate native capability, never simulated with pipes.
                command.process_group(0);
                command.as_std_mut().arg0(&self.command[0]);
                command
                    .spawn()
                    .map_err(|e| error(ErrorKind::LaunchFailure, e.to_string()))
            }
            .await;
            let mut child = match launch {
                Ok(child) => ProcessGroup::new(child, reservation.process_uncertain.clone())?,
                Err(e) => {
                    session.state = if e.kind == ErrorKind::SessionLost {
                        SessionState::Lost
                    } else {
                        SessionState::Failed
                    };
                    session.recovery["launch_failure"] = json!(e.message);
                    save_session(&self.store, &session, version)?;
                    reservation.session = None;
                    return Err(e);
                }
            };
            session.pid = child.child.id();
            session.state = SessionState::Running;
            version = match save_session(&self.store, &session, version) {
                Ok(version) => version,
                Err(e) => {
                    if let Err(cleanup) = child.kill_group() {
                        // Unknown native group death must not release the reservation.
                        reservation.session = None;
                        return Err(error(
                            ErrorKind::SessionLost,
                            format!(
                                "{e}; native cleanup failed: {cleanup}; Starting reservation retained"
                            ),
                        ));
                    }
                    let _ = child.reap().await;
                    return Err(e);
                }
            };
            reservation.session = Some((session.clone(), version));
            let initial = SessionStatus {
                session: session.clone(),
                exit_code: None,
                stdout: vec![],
                stderr: vec![],
                stdout_truncated: false,
                stderr_truncated: false,
                failure: None,
            };
            let (events, status) = watch::channel(initial);
            let (stop, controls) = mpsc::channel(1);
            let mut stdin = child.child.stdin.take().expect("piped stdin");
            let stdout = child.child.stdout.take().expect("piped stdout");
            let stderr = child.child.stderr.take().expect("piped stderr");
            let input = request.input.payload.into_bytes();
            let writer = tokio::spawn(async move {
                stdin.write_all(&input).await?;
                stdin.shutdown().await
            });
            let stdout = tokio::spawn(drain(stdout, events.clone(), true));
            let stderr = tokio::spawn(drain(stderr, events.clone(), false));
            {
                let mut sessions = self
                    .sessions
                    .lock()
                    .map_err(|_| error(ErrorKind::ProcessFailure, "session registry poisoned"))?;
                let mut terminal: Vec<_> = sessions
                    .iter()
                    .filter(|(_, e)| e.status.borrow().terminal())
                    .map(|(id, e)| (*id, e.status.borrow().session.started_at))
                    .collect();
                terminal.sort_by_key(|(_, time)| *time);
                let excess = terminal.len().saturating_sub(TERMINAL_RETENTION_LIMIT - 1);
                for (id, _) in terminal.into_iter().take(excess) {
                    sessions.remove(&id);
                }
                sessions.insert(
                    session.id,
                    Entry {
                        scope: request.scope,
                        status,
                        stop,
                    },
                );
            }
            tokio::spawn(supervise(
                child,
                controls,
                events,
                ProcessIo {
                    writer,
                    stdout,
                    stderr,
                },
                self.store.clone(),
                version,
            ));
            reservation.session = None;
            Ok(session)
        })
    }
    fn status(&self, reference: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        Box::pin(async move {
            let (status, _) = self.entry(&reference)?;
            Ok(status.borrow().clone())
        })
    }
    fn stop(&self, reference: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        Box::pin(async move {
            let (mut status, stop) = self.entry(&reference)?;
            if !status.borrow().terminal() {
                let _ = stop.send(()).await;
                tokio::time::timeout(Duration::from_secs(5), async {
                    while !status.borrow().terminal() {
                        status.changed().await.map_err(|_| {
                            error(
                                ErrorKind::SessionLost,
                                "supervisor ended without terminal status",
                            )
                        })?;
                    }
                    Ok::<_, AdapterError>(())
                })
                .await
                .map_err(|_| error(ErrorKind::Timeout, "native stop did not complete"))??;
            }
            Ok(status.borrow().clone())
        })
    }
    fn attach(&self, _reference: SessionRef) -> AdapterFuture<'_, ()> {
        Box::pin(async { Err(unsupported(Capability::Attach)) })
    }
    fn resume(&self, _reference: SessionRef) -> AdapterFuture<'_, Session> {
        Box::pin(async { Err(unsupported(Capability::Resume)) })
    }
    fn release(&self, reference: SessionRef) -> AdapterResult<()> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| error(ErrorKind::ProcessFailure, "session registry poisoned"))?;
        let entry = sessions.get(&reference.id).ok_or_else(|| {
            error(
                ErrorKind::SessionLost,
                "session already released or unavailable",
            )
        })?;
        if entry.scope != reference.scope {
            return Err(error(
                ErrorKind::OwnershipMismatch,
                "session ownership mismatch",
            ));
        }
        if !entry.status.borrow().terminal() {
            return Err(error(
                ErrorKind::InvalidInput,
                "running session cannot be released",
            ));
        }
        sessions.remove(&reference.id);
        Ok(())
    }
    fn subscribe(&self, reference: SessionRef) -> AdapterResult<watch::Receiver<SessionStatus>> {
        Ok(self.entry(&reference)?.0)
    }
    fn usage(
        &self,
        reference: SessionRef,
        phase: String,
        review_round: Option<u32>,
    ) -> AdapterFuture<'_, Usage> {
        Box::pin(async move {
            let (status, _) = self.entry(&reference)?;
            let status = status.borrow();
            Ok(Usage {
                scope: status.session.scope.clone(),
                session_id: status.session.id,
                agent: status.session.agent.clone(),
                phase,
                review_round,
                input_tokens: None,
                cached_input_tokens: None,
                output_tokens: None,
                estimated_cost: None,
                context_pack_version: status.session.recovery["input_version"].as_u64(),
                context_pack_size: status.session.recovery["input_bytes"].as_u64(),
                repo_map_size: None,
                cache_metadata: Value::Null,
                missing_reason: Some("generic CLI exposes no usage or cache telemetry".into()),
            })
        })
    }
}

fn resolve_executable(command: &str) -> AdapterResult<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let executable = |path: &Path| {
        path.is_file()
            && path
                .metadata()
                .is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
    };
    let path = Path::new(command);
    if path.is_absolute() {
        if executable(path) {
            return path
                .canonicalize()
                .map_err(|e| error(ErrorKind::ExecutableMissing, e.to_string()));
        }
    } else if !command.contains('/') {
        if let Some(paths) = std::env::var_os("PATH") {
            for directory in std::env::split_paths(&paths).filter(|p| p.is_absolute()) {
                let path = directory.join(command);
                if executable(&path) {
                    return path
                        .canonicalize()
                        .map_err(|e| error(ErrorKind::ExecutableMissing, e.to_string()));
                }
            }
        }
    } else {
        return Err(error(
            ErrorKind::InvalidConfiguration,
            "executable paths must be absolute or resolved from runtime PATH",
        ));
    }
    Err(error(
        ErrorKind::ExecutableMissing,
        format!("executable unavailable: {command}"),
    ))
}

fn validate_request(request: &LaunchRequest) -> AdapterResult<PathBuf> {
    if request.scope.project_id != request.project.id
        || request.input.scope != request.scope
        || (request.scope.task_id.is_some() && request.scope.goal_id.is_none())
    {
        return Err(error(
            ErrorKind::OwnershipMismatch,
            "launch/context project/goal/task mismatch",
        ));
    }
    if request.project.state != ProjectState::Registered
        || request.input.revision.trim().is_empty()
        || request.input.version == 0
        || request.input.payload.is_empty()
    {
        return Err(error(
            ErrorKind::InvalidInput,
            "launch requires a registered Project and versioned nonempty input/revision",
        ));
    }
    let review = matches!(
        request.role,
        SessionRole::Reviewer | SessionRole::ApprovalReviewer
    );
    if review != (request.input.kind == InputKind::ReviewBundle)
        || (request.role == SessionRole::Executor && request.scope.task_id.is_none())
    {
        return Err(error(
            ErrorKind::InvalidInput,
            "executors require a Task; reviewers require Review Bundle inputs",
        ));
    }
    if request
        .environment
        .iter()
        .any(|(key, value)| key.is_empty() || key.contains(['=', '\0']) || value.contains('\0'))
    {
        return Err(error(ErrorKind::InvalidInput, "invalid environment entry"));
    }
    if request
        .environment
        .keys()
        .any(|key| key.starts_with("GIT_"))
    {
        return Err(error(
            ErrorKind::InvalidInput,
            "Git environment overrides are not accepted in owned agent launches",
        ));
    }
    let canonical = |p: &Path| {
        p.canonicalize()
            .map_err(|e| error(ErrorKind::InvalidInput, format!("invalid launch path: {e}")))
    };
    let worktree = canonical(&request.worktree)?;
    let root = canonical(&request.project.root)?;
    let namespace = canonical(&request.project.worktree_root)?;
    if !worktree.is_dir()
        || !(worktree.starts_with(&namespace)
            || (worktree == root && request.role == SessionRole::Consultant))
    {
        return Err(error(
            ErrorKind::OwnershipMismatch,
            "launch worktree is outside Project namespace",
        ));
    }
    Ok(worktree)
}

fn save_session(
    store: &SharedStore,
    session: &Session,
    expected_version: u64,
) -> AdapterResult<u64> {
    let mut store = store
        .lock()
        .map_err(|_| error(ErrorKind::StateFailure, "state store poisoned"))?;
    let current = store
        .session(session.id)
        .map_err(|e| error(ErrorKind::StateFailure, e.to_string()))?
        .map_or(0, |(_, v)| v);
    if current != expected_version {
        return Err(error(
            ErrorKind::StateConflict,
            "session snapshot changed concurrently",
        ));
    }
    store
        .put_session(session, expected_version)
        .map_err(state_error)
}

fn state_error(error_value: anyhow::Error) -> AdapterError {
    use crate::state::StateGuardError;
    let kind = match error_value.downcast_ref::<StateGuardError>() {
        Some(StateGuardError::WorktreeLocked) => ErrorKind::Locked,
        Some(StateGuardError::ExecutorReserved | StateGuardError::SnapshotChanged { .. }) => {
            ErrorKind::StateConflict
        }
        None => ErrorKind::StateFailure,
    };
    error(kind, error_value.to_string())
}
fn validate_persisted(
    store: &SharedStore,
    request: &LaunchRequest,
    worktree: &Path,
    agent: &str,
) -> AdapterResult<()> {
    use crate::domain::{GoalState, TaskState};
    let store = store
        .lock()
        .map_err(|_| error(ErrorKind::StateFailure, "state store poisoned"))?;
    let owned = || {
        error(
            ErrorKind::OwnershipMismatch,
            "persisted Project/Goal/Task/worktree ownership mismatch",
        )
    };
    let project = store
        .project(request.scope.project_id)
        .map_err(state_error)?
        .ok_or_else(owned)?;
    if project.state != ProjectState::Registered
        || project.root != request.project.root
        || project.worktree_root != request.project.worktree_root
        || project.base_branch != request.project.base_branch
    {
        return Err(owned());
    }
    let goal = store
        .goal(request.scope.goal_id.ok_or_else(owned)?)
        .map_err(state_error)?
        .ok_or_else(owned)?;
    let task = store
        .task(request.scope.task_id.ok_or_else(owned)?)
        .map_err(state_error)?
        .ok_or_else(owned)?;
    if goal.project_id != project.id
        || task.scope() != request.scope
        || task.executor != agent
        || task
            .worktree
            .as_deref()
            .map(Path::canonicalize)
            .transpose()
            .map_err(|e| error(ErrorKind::OwnershipMismatch, e.to_string()))?
            .as_deref()
            != Some(worktree)
    {
        return Err(owned());
    }
    if matches!(
        goal.state,
        GoalState::Paused | GoalState::Completed | GoalState::Cancelled | GoalState::Failed
    ) || matches!(
        task.state,
        TaskState::Completed
            | TaskState::Cancelled
            | TaskState::Merged
            | TaskState::Failed
            | TaskState::ReadyForPr
            | TaskState::PrCreated
    ) {
        return Err(error(
            ErrorKind::InvalidInput,
            "Task/Goal lifecycle does not allow executor launch",
        ));
    }
    Ok(())
}
fn ensure_unlocked(store: &SharedStore, scope: &Scope) -> AdapterResult<()> {
    let store = store
        .lock()
        .map_err(|_| error(ErrorKind::StateFailure, "state store poisoned"))?;
    for lock in store
        .records(scope, RecordKind::WorktreeLock)
        .map_err(state_error)?
    {
        if lock.data["active"].as_bool() != Some(false) {
            return Err(error(
                ErrorKind::Locked,
                "worktree is review locked or lock state is malformed",
            ));
        }
    }
    Ok(())
}

async fn validate_git(
    store: &SharedStore,
    request: &LaunchRequest,
    worktree: &Path,
    executable: Option<&Path>,
    process_uncertain: Arc<AtomicBool>,
) -> AdapterResult<()> {
    let (project, task, goal) = {
        let store = store
            .lock()
            .map_err(|_| error(ErrorKind::StateFailure, "state store poisoned"))?;
        (
            store
                .project(request.scope.project_id)
                .map_err(state_error)?
                .ok_or_else(|| error(ErrorKind::OwnershipMismatch, "Project lost"))?,
            store
                .task(request.scope.task_id.expect("validated Task"))
                .map_err(state_error)?
                .ok_or_else(|| error(ErrorKind::OwnershipMismatch, "Task lost"))?,
            store
                .goal(request.scope.goal_id.expect("validated Goal"))
                .map_err(state_error)?
                .ok_or_else(|| error(ErrorKind::OwnershipMismatch, "Goal lost"))?,
        )
    };
    // Never hold the shared Store across external Git or project hooks.
    let executable = match executable {
        Some(path) => path.to_path_buf(),
        None => resolve_executable("git")?,
    };
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let text = |cwd: PathBuf, args: Vec<String>| {
        let executable = executable.clone();
        let environment = request.environment.clone();
        let process_uncertain = process_uncertain.clone();
        async move {
            bounded_git(
                &executable,
                &cwd,
                &args,
                environment,
                deadline,
                process_uncertain,
            )
            .await
        }
    };
    let args = |args: &[&str]| args.iter().map(|s| (*s).to_string()).collect();
    text(
        project.root.clone(),
        args(&["check-ref-format", "--branch", &project.base_branch]),
    )
    .await?;
    let source_top = text(
        project.root.clone(),
        args(&["rev-parse", "--show-toplevel"]),
    )
    .await?;
    let source_git_dir = text(
        project.root.clone(),
        args(&["rev-parse", "--path-format=absolute", "--git-dir"]),
    )
    .await?;
    let source_common = text(
        project.root.clone(),
        args(&["rev-parse", "--path-format=absolute", "--git-common-dir"]),
    )
    .await?;
    let roots = text(
        project.root.clone(),
        vec![
            "rev-list".into(),
            "--max-parents=0".into(),
            format!("refs/heads/{}", project.base_branch),
        ],
    )
    .await?;
    let task_top = text(
        worktree.to_path_buf(),
        args(&["rev-parse", "--show-toplevel"]),
    )
    .await?;
    let task_common = text(
        worktree.to_path_buf(),
        args(&["rev-parse", "--path-format=absolute", "--git-common-dir"]),
    )
    .await?;
    let branch = text(
        worktree.to_path_buf(),
        args(&["symbolic-ref", "--quiet", "--short", "HEAD"]),
    )
    .await?;
    let revision = text(
        worktree.to_path_buf(),
        args(&["rev-parse", "--verify", "HEAD^{commit}"]),
    )
    .await?;
    crate::git::validate_worktree_ownership(
        &project,
        &task,
        crate::git::WorktreeOwnershipFacts {
            source_top: source_top.into(),
            source_git_dir: source_git_dir.into(),
            source_common: source_common.into(),
            source_roots: roots.lines().map(str::to_owned).collect(),
            task_top: task_top.into(),
            task_common: task_common.into(),
            branch,
            revision,
        },
    )
    .map_err(|e| error(ErrorKind::OwnershipMismatch, e.to_string()))?;
    let store = store
        .lock()
        .map_err(|_| error(ErrorKind::StateFailure, "state store poisoned"))?;
    if store
        .project(project.id)
        .map_err(state_error)?
        .is_none_or(|p| p.version != project.version)
        || store
            .task(task.id)
            .map_err(state_error)?
            .is_none_or(|t| t.version != task.version)
        || store
            .goal(goal.id)
            .map_err(state_error)?
            .is_none_or(|g| g.version != goal.version)
    {
        return Err(error(
            ErrorKind::StateConflict,
            "Project/Goal/Task changed during Git preflight",
        ));
    }
    Ok(())
}

async fn bounded_git(
    executable: &Path,
    cwd: &Path,
    args: &[String],
    environment: BTreeMap<String, String>,
    deadline: tokio::time::Instant,
    process_uncertain: Arc<AtomicBool>,
) -> AdapterResult<String> {
    if tokio::time::Instant::now() >= deadline {
        return Err(error(
            ErrorKind::Timeout,
            "Git ownership preflight timed out",
        ));
    }
    let mut command = Command::new(executable);
    command
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .envs(environment)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .process_group(0);
    let mut child = ProcessGroup::new(
        command
            .spawn()
            .map_err(|e| error(ErrorKind::ProcessFailure, e.to_string()))?,
        process_uncertain,
    )?;
    let stdout = child.child.stdout.take().expect("piped Git stdout");
    let stderr = child.child.stderr.take().expect("piped Git stderr");
    let mut stdout = tokio::spawn(read_git_output(stdout));
    let mut stderr = tokio::spawn(read_git_output(stderr));
    let observed = tokio::time::timeout_at(deadline, child.observe_exit()).await;
    child = cleanup_group(child).await?;
    let exit = child.reap().await.map_err(|e| {
        error(
            ErrorKind::SessionLost,
            format!("Git child reap failed: {e}"),
        )
    })?;
    if observed.is_err() {
        stdout.abort();
        stderr.abort();
        return Err(error(
            ErrorKind::Timeout,
            "Git ownership preflight timed out",
        ));
    }
    observed.expect("checked deadline").map_err(|e| {
        error(
            ErrorKind::SessionLost,
            format!("Git child observation failed: {e}"),
        )
    })?;
    let output = tokio::time::timeout(Duration::from_millis(250), async {
        let out = (&mut stdout)
            .await
            .map_err(|e| error(ErrorKind::ProcessFailure, e.to_string()))??;
        let _ = (&mut stderr)
            .await
            .map_err(|e| error(ErrorKind::ProcessFailure, e.to_string()))??;
        Ok::<_, AdapterError>(out)
    })
    .await;
    let output = match output {
        Ok(output) => output?,
        Err(_) => {
            stdout.abort();
            stderr.abort();
            return Err(error(
                ErrorKind::ProcessFailure,
                "Git output remained open after cleanup",
            ));
        }
    };
    if !exit.success() {
        return Err(error(
            ErrorKind::OwnershipMismatch,
            "Git ownership preflight failed",
        ));
    }
    String::from_utf8(output)
        .map(|value| value.trim().to_string())
        .map_err(|_| error(ErrorKind::ParseFailure, "invalid Git metadata encoding"))
}
async fn read_git_output(reader: impl AsyncRead + Unpin) -> AdapterResult<Vec<u8>> {
    let mut bytes = vec![];
    reader
        .take((OUTPUT_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(|e| error(ErrorKind::ProcessFailure, e.to_string()))?;
    if bytes.len() > OUTPUT_LIMIT {
        return Err(error(
            ErrorKind::InvalidInput,
            "Git metadata exceeds output budget",
        ));
    }
    Ok(bytes)
}
async fn cleanup_group(mut child: ProcessGroup) -> AdapterResult<ProcessGroup> {
    let (child, result) = tokio::task::spawn_blocking(move || {
        let result = child.kill_group();
        (child, result)
    })
    .await
    .map_err(|e| {
        error(
            ErrorKind::SessionLost,
            format!("native cleanup worker failed: {e}"),
        )
    })?;
    result.map_err(|e| {
        error(
            ErrorKind::SessionLost,
            format!("native process group cleanup failed: {e}"),
        )
    })?;
    Ok(child)
}

async fn drain<R: AsyncRead + Unpin>(
    mut reader: R,
    events: watch::Sender<SessionStatus>,
    stdout: bool,
) {
    let mut buffer = [0u8; 8192];
    loop {
        match reader.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => events.send_modify(|status| {
                let (tail, truncated) = if stdout {
                    (&mut status.stdout, &mut status.stdout_truncated)
                } else {
                    (&mut status.stderr, &mut status.stderr_truncated)
                };
                tail.extend_from_slice(&buffer[..n]);
                if tail.len() > OUTPUT_LIMIT {
                    tail.drain(..tail.len() - OUTPUT_LIMIT);
                    *truncated = true;
                }
            }),
            Err(e) => {
                events.send_modify(|s| s.failure = Some(format!("native output read failed: {e}")));
                break;
            }
        }
    }
}

struct ProcessIo {
    writer: JoinHandle<std::io::Result<()>>,
    stdout: JoinHandle<()>,
    stderr: JoinHandle<()>,
}

async fn supervise(
    child: ProcessGroup,
    mut controls: mpsc::Receiver<()>,
    events: watch::Sender<SessionStatus>,
    io: ProcessIo,
    store: SharedStore,
    version: u64,
) {
    let ProcessIo {
        writer,
        mut stdout,
        mut stderr,
    } = io;
    let mut stopped = false;
    let mut stop_failure = None;
    let exited = tokio::select! {
        result = child.observe_exit() => result,
        _ = controls.recv() => { stopped = true; Ok(()) }
    };
    let result = match exited {
        Ok(()) => match cleanup_group(child).await {
            Ok(mut child) => child.reap().await,
            Err(e) => {
                stop_failure = Some(e.to_string());
                Err(std::io::Error::other("owned process cleanup not confirmed"))
            }
        },
        Err(e) => {
            stop_failure = Some(format!(
                "native exit observation failed; recovery must verify process death: {e}"
            ));
            drop(child);
            Err(e)
        }
    };
    let delivery_cancelled = !writer.is_finished() && !stopped;
    if !writer.is_finished() {
        writer.abort();
    }
    let input_failure = match writer.await {
        Ok(Err(e)) if !stopped => Some(format!("native stdin delivery failed: {e}")),
        _ if delivery_cancelled => {
            Some("native process exited before stdin delivery completed".into())
        }
        _ => None,
    };
    let outputs = async {
        let _ = (&mut stdout).await;
        let _ = (&mut stderr).await;
    };
    if tokio::time::timeout(Duration::from_millis(250), outputs)
        .await
        .is_err()
    {
        stdout.abort();
        stderr.abort();
        events.send_modify(|s| {
            s.failure =
                Some("native output remained open after process exit; capture stopped".into())
        });
    }
    let mut terminal = events.borrow().clone();
    {
        let status = &mut terminal;
        status.failure = stop_failure.or(input_failure).or(status.failure.take());
        match result {
            Ok(exit) => {
                status.exit_code = exit.code();
                status.session.state = if stopped && status.failure.is_none() {
                    SessionState::Stopped
                } else if exit.success() && status.failure.is_none() {
                    SessionState::Exited
                } else {
                    SessionState::Failed
                };
            }
            Err(e) => {
                status
                    .failure
                    .get_or_insert_with(|| format!("native process wait failed: {e}"));
                status.session.state = SessionState::Lost;
            }
        }
    }
    if let Err(e) = save_session(&store, &terminal.session, version) {
        terminal.failure = Some(format!("terminal state persistence failed: {e}"));
        terminal.session.state = SessionState::Lost;
    }
    events.send_replace(terminal);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CompletionCriterion, Goal, Task};
    use std::os::unix::fs::PermissionsExt;

    #[cfg(target_os = "macos")]
    #[test]
    fn signal_permission_failure_requires_verified_dead_group() {
        let denied = || Err(rustix::io::Errno::PERM);
        assert!(
            resolve_macos_signal_result(denied(), || process_group_is_dead(b"42 R\n", 42)).is_err()
        );
        assert!(
            resolve_macos_signal_result(denied(), || process_group_is_dead(b"42 Z\n99 R\n", 42))
                .is_ok()
        );
        assert!(
            resolve_macos_signal_result(denied(), || process_group_is_dead(b"malformed\n", 42))
                .is_err()
        );
        assert!(
            resolve_macos_signal_result(denied(), || Err(std::io::Error::other("ps failed")))
                .is_err()
        );
        assert!(resolve_macos_signal_result(Err(rustix::io::Errno::ACCESS), || Ok(true)).is_err());
    }

    fn preflight_fixture() -> (tempfile::TempDir, SharedStore, Project, Task, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("repo");
        std::fs::create_dir(&root).unwrap();
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        git(&["init", "-b", "main"]);
        git(&[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--allow-empty",
            "-m",
            "fixture",
        ]);
        let root = root.canonicalize().unwrap();
        let mut project = Project::new(
            "timeout fixture".into(),
            root.clone(),
            crate::git::repository_identity(&root, "main").unwrap(),
            "main".into(),
        );
        let mut goal = Goal::new(
            project.id,
            "bounded preflight".into(),
            vec![CompletionCriterion {
                id: "bounded".into(),
                description: "Git must terminate".into(),
                evidence: None,
                satisfied: false,
            }],
        );
        let mut task = Task::new(project.id, goal.id, "timeout".into(), "fake".into());
        let mut state = Store::memory().unwrap();
        state.put_project(&mut project).unwrap();
        state.put_goal(&mut goal).unwrap();
        state.put_task(&mut task).unwrap();
        let worktree = crate::git::WorktreeManager::create(&mut state, task.id)
            .unwrap()
            .worktree;
        let store = Arc::new(Mutex::new(state));
        (temp, store, project, task, worktree)
    }

    #[tokio::test]
    async fn hanging_git_preflight_times_out_without_holding_shared_store() {
        let (temp, store, project, task, worktree) = preflight_fixture();
        let marker = temp.path().join("started");
        let shim = temp.path().join("slow-git");
        let quoted = marker.to_str().unwrap().replace('\'', "'\\''");
        std::fs::write(
            &shim,
            format!("#!/bin/sh\nprintf '%s' \"$$\" > '{quoted}'\nsleep 60\n"),
        )
        .unwrap();
        std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut adapter =
            GenericCliAdapter::new("fake".into(), vec!["/bin/cat".into()], store.clone()).unwrap();
        adapter.git_executable = Some(shim);
        let scope = task.scope();
        let request = LaunchRequest {
            project,
            scope: scope.clone(),
            worktree,
            role: SessionRole::Executor,
            mode: LaunchMode::NonInteractive,
            input: PreparedInput {
                scope,
                kind: InputKind::ContextPack,
                revision: "fixture".into(),
                version: 1,
                source_versions: BTreeMap::new(),
                payload: "fixture\n".into(),
            },
            environment: BTreeMap::from([("PATH".into(), "/usr/bin:/bin".into())]),
            model: None,
            effort: None,
        };
        let scope = request.scope.clone();
        let launch = tokio::spawn(async move { adapter.start(request).await });
        tokio::time::timeout(Duration::from_secs(3), async {
            while !marker.exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        assert!(
            store.try_lock().is_ok(),
            "external Git must not own the shared Store mutex"
        );
        let result = tokio::time::timeout(Duration::from_secs(7), launch)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.unwrap_err().kind, ErrorKind::Timeout);
        let pid = std::fs::read_to_string(marker).unwrap();
        let output = std::process::Command::new("ps")
            .args(["-o", "stat=", "-p", &pid])
            .output()
            .unwrap();
        assert!(
            output.stdout.is_empty(),
            "timed-out Git leader must be reaped"
        );
        assert_eq!(
            store
                .lock()
                .unwrap()
                .records(&scope, RecordKind::Session)
                .unwrap()[0]
                .data["state"],
            "FAILED"
        );
    }

    #[tokio::test]
    async fn ownership_change_during_native_preflight_prevents_executor_launch() {
        let (temp, store, project, task, worktree) = preflight_fixture();
        let marker = temp.path().join("started");
        let release = temp.path().join("release");
        let shim = temp.path().join("gated-git");
        let quote = |path: &Path| path.to_str().unwrap().replace('\'', "'\\''");
        std::fs::write(&shim, format!(
            "#!/bin/sh\ntouch '{}'\nwhile [ ! -f '{}' ]; do sleep 0.02; done\nexec /usr/bin/git \"$@\"\n",
            quote(&marker), quote(&release),
        )).unwrap();
        std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut adapter =
            GenericCliAdapter::new("fake".into(), vec!["/bin/cat".into()], store.clone()).unwrap();
        adapter.git_executable = Some(shim);
        let scope = task.scope();
        let request = LaunchRequest {
            project,
            scope: scope.clone(),
            worktree,
            role: SessionRole::Executor,
            mode: LaunchMode::NonInteractive,
            input: PreparedInput {
                scope: scope.clone(),
                kind: InputKind::ContextPack,
                revision: "fixture".into(),
                version: 1,
                source_versions: BTreeMap::new(),
                payload: "fixture".into(),
            },
            environment: BTreeMap::from([("PATH".into(), "/usr/bin:/bin".into())]),
            model: None,
            effort: None,
        };
        let launch = tokio::spawn(async move { adapter.start(request).await });
        tokio::time::timeout(Duration::from_secs(3), async {
            while !marker.exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        {
            let mut state = store.lock().unwrap();
            let mut goal = state.goal(scope.goal_id.unwrap()).unwrap().unwrap();
            goal.objective = "changed during preflight".into();
            state.put_goal(&mut goal).unwrap();
        }
        std::fs::write(release, "ready").unwrap();
        let result = tokio::time::timeout(Duration::from_secs(5), launch)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.unwrap_err().kind, ErrorKind::StateConflict);
        let records = store
            .lock()
            .unwrap()
            .records(&scope, RecordKind::Session)
            .unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].data["state"], "FAILED");
        assert!(
            records[0].data["pid"].is_null(),
            "executor must never spawn"
        );
    }
}
