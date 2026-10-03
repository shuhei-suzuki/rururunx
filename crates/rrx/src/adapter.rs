//! Thin native process contracts. Context selection and workflow decisions belong upstream.
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    process::Stdio,
    sync::{Arc, Mutex},
    time::Duration,
};

use nix::{
    sys::signal::{Signal, killpg},
    unistd::Pid,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
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
    fn start_native_goal(&self, _input: PreparedInput) -> AdapterFuture<'_, String> {
        Box::pin(async { Err(unsupported(Capability::NativeGoal)) })
    }
    fn native_goal_status(&self, _native_ref: String) -> AdapterFuture<'_, Value> {
        Box::pin(async { Err(unsupported(Capability::NativeGoalStatus)) })
    }
    fn resume_native_goal(
        &self,
        _native_ref: String,
        _input: PreparedInput,
    ) -> AdapterFuture<'_, String> {
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
    session: Option<Session>,
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if let Some(mut session) = self.session.take() {
            session.state = SessionState::Failed;
            let _ = save_session(&self.store, &session);
        }
    }
}
pub struct GenericCliAdapter {
    agent: String,
    command: Vec<String>,
    store: SharedStore,
    sessions: Mutex<HashMap<SessionId, Entry>>,
}
const OUTPUT_LIMIT: usize = 64 * 1024;
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
            let worktree = validate_request(&request)?;
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
            validate_persisted(&self.store, &request, &worktree)?;
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
                recovery: json!({"reconnect_supported":false,"project_root":request.project.root,"input_revision":request.input.revision,"input_version":request.input.version,"source_versions":request.input.source_versions}),
                started_at: now_ms(),
            };
            // Reserve before asynchronous preflight. Store's task lock/session exclusion is
            // authoritative; recovery must explicitly resolve a stale reservation.
            save_session(&self.store, &session)?;
            let mut reservation = Reservation {
                store: self.store.clone(),
                session: Some(session.clone()),
            };
            let launch = async {
                ensure_unlocked(&self.store, &request.scope)?;
                validate_git(&self.store, &request, &worktree).await?;
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
                command
                    .spawn()
                    .map_err(|e| error(ErrorKind::LaunchFailure, e.to_string()))
            }
            .await;
            let mut child = match launch {
                Ok(child) => child,
                Err(e) => {
                    session.state = SessionState::Failed;
                    save_session(&self.store, &session)?;
                    return Err(e);
                }
            };
            session.pid = child.id();
            session.state = SessionState::Running;
            if let Err(e) = save_session(&self.store, &session) {
                if let Some(pid) = child.id() {
                    let _ = killpg(Pid::from_raw(pid as i32), Signal::SIGKILL);
                }
                let _ = child.wait().await;
                return Err(e);
            }
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
            let mut stdin = child.stdin.take().expect("piped stdin");
            let stdout = child.stdout.take().expect("piped stdout");
            let stderr = child.stderr.take().expect("piped stderr");
            let input = request.input.payload.into_bytes();
            let writer = tokio::spawn(async move {
                stdin.write_all(&input).await?;
                stdin.shutdown().await
            });
            let stdout = tokio::spawn(drain(stdout, events.clone(), true));
            let stderr = tokio::spawn(drain(stderr, events.clone(), false));
            self.sessions
                .lock()
                .map_err(|_| error(ErrorKind::ProcessFailure, "session registry poisoned"))?
                .insert(
                    session.id,
                    Entry {
                        scope: request.scope,
                        status,
                        stop,
                    },
                );
            tokio::spawn(supervise(
                child,
                controls,
                events,
                writer,
                stdout,
                stderr,
                self.store.clone(),
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
    fn attach(&self, reference: SessionRef) -> AdapterFuture<'_, ()> {
        Box::pin(async move {
            self.entry(&reference)?;
            Err(unsupported(Capability::Attach))
        })
    }
    fn resume(&self, reference: SessionRef) -> AdapterFuture<'_, Session> {
        Box::pin(async move {
            self.entry(&reference)?;
            Err(unsupported(Capability::Resume))
        })
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
                context_pack_size: None,
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

fn with_store<T>(
    store: &SharedStore,
    f: impl FnOnce(&mut Store) -> anyhow::Result<T>,
) -> AdapterResult<T> {
    let mut store = store
        .lock()
        .map_err(|_| error(ErrorKind::ProcessFailure, "state store poisoned"))?;
    f(&mut store).map_err(|e| error(ErrorKind::InvalidInput, e.to_string()))
}

fn save_session(store: &SharedStore, session: &Session) -> AdapterResult<()> {
    with_store(store, |store| {
        let version = store.session(session.id)?.map_or(0, |(_, version)| version);
        store.put_session(session, version)?;
        Ok(())
    })
}

fn validate_persisted(
    store: &SharedStore,
    request: &LaunchRequest,
    worktree: &Path,
) -> AdapterResult<()> {
    with_store(store, |store| {
        let project = store
            .project(request.scope.project_id)?
            .ok_or_else(|| anyhow::anyhow!("Project not registered"))?;
        anyhow::ensure!(
            project.state == ProjectState::Registered
                && project.root == request.project.root
                && project.worktree_root == request.project.worktree_root
                && project.base_branch == request.project.base_branch,
            "Project snapshot does not match registry"
        );
        let goal = store
            .goal(
                request
                    .scope
                    .goal_id
                    .ok_or_else(|| anyhow::anyhow!("Goal required"))?,
            )?
            .ok_or_else(|| anyhow::anyhow!("Goal not found"))?;
        let task = store
            .task(
                request
                    .scope
                    .task_id
                    .ok_or_else(|| anyhow::anyhow!("Task required"))?,
            )?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        anyhow::ensure!(
            goal.project_id == project.id && task.scope() == request.scope,
            "persisted Project/Goal/Task ownership mismatch"
        );
        anyhow::ensure!(
            task.worktree
                .as_deref()
                .map(Path::canonicalize)
                .transpose()?
                .as_deref()
                == Some(worktree),
            "Task worktree does not match launch CWD"
        );
        Ok(())
    })
}

fn ensure_unlocked(store: &SharedStore, scope: &Scope) -> AdapterResult<()> {
    with_store(store, |store| {
        for lock in store.records(scope, RecordKind::WorktreeLock)? {
            anyhow::ensure!(
                lock.data["active"].as_bool() == Some(false),
                "worktree is review locked or lock state is malformed"
            );
        }
        Ok(())
    })
}

async fn git_value(
    cwd: &Path,
    args: &[&str],
    environment: &BTreeMap<String, String>,
) -> AdapterResult<String> {
    let git = resolve_executable("git")?;
    let mut command = Command::new(git);
    command
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .envs(environment)
        .kill_on_drop(true);
    let output = tokio::time::timeout(Duration::from_secs(5), command.output())
        .await
        .map_err(|_| error(ErrorKind::Timeout, "Git ownership preflight timed out"))?
        .map_err(|e| {
            error(
                ErrorKind::InvalidInput,
                format!("Git ownership preflight failed: {e}"),
            )
        })?;
    if !output.status.success() {
        return Err(error(
            ErrorKind::OwnershipMismatch,
            "Git ownership preflight failed",
        ));
    }
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|_| error(ErrorKind::InvalidInput, "invalid Git path/branch encoding"))
}

async fn validate_git(
    store: &SharedStore,
    request: &LaunchRequest,
    worktree: &Path,
) -> AdapterResult<()> {
    let args = ["rev-parse", "--path-format=absolute", "--git-common-dir"];
    let root_git = git_value(&request.project.root, &args, &request.environment).await?;
    let task_git = git_value(worktree, &args, &request.environment).await?;
    let top = git_value(
        worktree,
        &["rev-parse", "--show-toplevel"],
        &request.environment,
    )
    .await?;
    let branch = git_value(
        worktree,
        &["symbolic-ref", "--quiet", "--short", "HEAD"],
        &request.environment,
    )
    .await?;
    let expected_branch = with_store(store, |store| {
        Ok(store
            .task(request.scope.task_id.expect("validated Task"))?
            .and_then(|task| task.branch))
    })?;
    let canonical = |path: &str| {
        Path::new(path).canonicalize().map_err(|e| {
            error(
                ErrorKind::OwnershipMismatch,
                format!("Git ownership path missing: {e}"),
            )
        })
    };
    if expected_branch.as_deref() != Some(branch.as_str())
        || canonical(&root_git)? != canonical(&task_git)?
        || canonical(&top)? != worktree
        || matches!(branch.as_str(), "main" | "master")
        || branch == request.project.base_branch
    {
        return Err(error(
            ErrorKind::OwnershipMismatch,
            "executor requires an owned worktree on a non-base branch",
        ));
    }
    Ok(())
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

async fn supervise(
    mut child: Child,
    mut controls: mpsc::Receiver<()>,
    events: watch::Sender<SessionStatus>,
    writer: JoinHandle<std::io::Result<()>>,
    mut stdout: JoinHandle<()>,
    mut stderr: JoinHandle<()>,
    store: SharedStore,
) {
    let mut stopped = false;
    let mut stop_failure = None;
    let result = tokio::select! {
        result = child.wait() => result,
        _ = controls.recv() => {
            stopped = true;
            if let Some(pid) = child.id() {
                // Only a process group created and still owned by this supervisor is signalled.
                if let Err(e) = killpg(Pid::from_raw(pid as i32), Signal::SIGKILL) {
                    if e != nix::errno::Errno::ESRCH { stop_failure = Some(format!("native process group stop failed: {e}")); }
                }
            }
            child.wait().await
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
                status.failure = Some(format!("native process wait failed: {e}"));
                status.session.state = SessionState::Failed;
            }
        }
    }
    if let Err(e) = save_session(&store, &terminal.session) {
        terminal.failure = Some(format!("terminal state persistence failed: {e}"));
        terminal.session.state = SessionState::Failed;
    }
    events.send_replace(terminal);
}
