//! Native Grok private ACP supervisor. Grok owns inference, authentication and hooks.
mod files;
mod ownership;
mod protocol;
mod schema;

use super::*;
use files::{Inventory, ScopedFiles, digest};
use ownership::{ProcessOwnership, ScopeSnapshot, filesystem};
use protocol::{Rpc, TurnEvidence};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use tokio::sync::Notify;

const NATIVE_VERSION: &str = "1.0.46";
const SESSIONS_LIMIT: usize = 128;
const TURN_TIMEOUT: Duration = Duration::from_secs(300);
pub(super) fn failure(kind: ErrorKind, message: impl Into<String>) -> AdapterError {
    error(kind, message)
}

struct OwnedEntry {
    #[cfg(all(test, target_os = "macos"))]
    process_inspection: Option<ProcessInspectionPlan>,
    scope: Scope,
    transition: Mutex<()>,
    status: watch::Receiver<SessionStatus>,
    events: watch::Sender<SessionStatus>,
    request: Mutex<LaunchRequest>,
    schema: Option<Value>,
    busy: AtomicBool,
    stopping: AtomicBool,
    stop: Notify,
    completed: AtomicBool,
    cleaned: AtomicBool,
    usage: Mutex<Value>,
}
struct Busy(Option<Arc<OwnedEntry>>);
impl Busy {
    /// Publish idle once while the terminal transition is held. Disarming before
    /// opening the flag prevents a later guard drop from clearing a new claim.
    fn publish_idle(&mut self) {
        if let Some(entry) = self.0.take() {
            entry.busy.store(false, Ordering::SeqCst);
        }
    }
}
impl Drop for Busy {
    fn drop(&mut self) {
        if let Some(entry) = &self.0 {
            entry.busy.store(false, Ordering::SeqCst);
        }
    }
}

/// Provider selection is explicit; neither agent names nor arbitrary argv select Grok.
pub struct GrokAdapter {
    #[cfg(all(test, target_os = "macos"))]
    process_inspection: Option<ProcessInspectionPlan>,
    agent: String,
    executable: PathBuf,
    store: SharedStore,
    baseline: BTreeMap<String, String>,
    sessions: Mutex<HashMap<SessionId, Arc<OwnedEntry>>>,
}
impl GrokAdapter {
    pub fn new(agent: String, executable: PathBuf, store: SharedStore) -> AdapterResult<Self> {
        if agent.trim().is_empty() || !executable.is_absolute() {
            return Err(failure(
                ErrorKind::InvalidConfiguration,
                "Grok requires named agent and absolute executable",
            ));
        }
        let executable = resolve_executable(executable.to_str().ok_or_else(|| {
            failure(
                ErrorKind::InvalidConfiguration,
                "invalid executable encoding",
            )
        })?)?;
        // Values remain private memory, never audit/recovery metadata.
        let mut baseline = BTreeMap::new();
        for (key, value) in std::env::vars_os() {
            if let Some(key) = key.to_str().filter(|key| baseline_key(key)) {
                let value = value.into_string().map_err(|_| {
                    failure(
                        ErrorKind::InvalidConfiguration,
                        "native baseline environment must be UTF-8",
                    )
                })?;
                baseline.insert(key.to_owned(), value);
            }
        }
        Ok(Self {
            #[cfg(all(test, target_os = "macos"))]
            process_inspection: None,
            agent,
            executable,
            store,
            baseline,
            sessions: Mutex::new(HashMap::new()),
        })
    }
    pub fn start_structured(
        &self,
        request: LaunchRequest,
        output_schema: Value,
    ) -> AdapterFuture<'_, Session> {
        Box::pin(async move {
            schema::check(&output_schema)?;
            self.launch(request, Some(output_schema)).await
        })
    }
    fn entry(&self, reference: &SessionRef) -> AdapterResult<Arc<OwnedEntry>> {
        let entry = self
            .sessions
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "native registry poisoned"))?
            .get(&reference.id)
            .cloned()
            .ok_or_else(|| {
                failure(
                    ErrorKind::SessionLost,
                    "no private owned Grok supervisor; persisted PID is not reconnect authority",
                )
            })?;
        if entry.scope != reference.scope {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "foreign Grok Session scope",
            ));
        }
        let transition = entry
            .transition
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "native transition poisoned"))?;
        let status = entry.status.borrow().clone();
        assert_saved(&self.store, &status.session)?;
        drop(transition);
        Ok(entry)
    }
    fn environment(&self, request: &LaunchRequest) -> AdapterResult<BTreeMap<String, String>> {
        let mut environment = self.baseline.clone();
        for (key, value) in &request.environment {
            if key.starts_with("GIT_")
                || key.is_empty()
                || key.contains(['=', '\0'])
                || value.contains('\0')
            {
                return Err(failure(
                    ErrorKind::InvalidInput,
                    "invalid/leaking Git environment",
                ));
            }
            if !ordinary_key(key) && (!baseline_key(key) || self.baseline.get(key) != Some(value)) {
                return Err(failure(
                    ErrorKind::InvalidConfiguration,
                    "native auth/config/safety environment cannot replace intentional runtime baseline",
                ));
            }
            environment.insert(key.clone(), value.clone());
        }
        Ok(environment)
    }
    async fn launch(
        &self,
        request: LaunchRequest,
        output_schema: Option<Value>,
    ) -> AdapterResult<Session> {
        if request.mode != LaunchMode::NonInteractive {
            return Err(unsupported(Capability::Interactive));
        }
        if request.role == SessionRole::ApprovalReviewer {
            return Err(unsupported(Capability::Review));
        }
        if request.scope.task_id.is_none() {
            return Err(failure(
                ErrorKind::InvalidInput,
                "Grok baseline requires exact Task scope",
            ));
        }
        let snapshot = ScopeSnapshot::capture(&self.store, &request, &self.agent)?;
        let environment = self.environment(&request)?;
        let session = Session {
            id: SessionId::new(),
            scope: request.scope.clone(),
            agent: self.agent.clone(),
            provider: "grok".into(),
            role: request.role,
            native_ref: None,
            pid: None,
            worktree: request.worktree.clone(),
            state: SessionState::Starting,
            model: request.model.clone(),
            effort: request.effort.clone(),
            recovery: json!({"reconnect_supported":false,"input_version":request.input.version,"dispatch_state":"reserved"}),
            started_at: now_ms(),
        };
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
        let entry = Arc::new(OwnedEntry {
            #[cfg(all(test, target_os = "macos"))]
            process_inspection: self.process_inspection.clone(),
            scope: session.scope.clone(),
            transition: Mutex::new(()),
            status,
            events: events.clone(),
            request: Mutex::new(request.clone()),
            schema: output_schema,
            busy: AtomicBool::new(true),
            stopping: AtomicBool::new(false),
            stop: Notify::new(),
            completed: AtomicBool::new(false),
            cleaned: AtomicBool::new(false),
            usage: Mutex::new(Value::Null),
        });
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "native registry poisoned"))?;
        if sessions.len() >= SESSIONS_LIMIT {
            return Err(failure(
                ErrorKind::StateConflict,
                "release collected terminal native Sessions before further launch",
            ));
        }
        let version = save_current(&self.store, &session, 0, &snapshot)?;
        sessions.insert(session.id, entry.clone());
        drop(sessions);
        tokio::spawn(supervise(
            Actor {
                store: self.store.clone(),
                agent: self.agent.clone(),
                executable: self.executable.clone(),
                request,
                environment,
                snapshot,
                session: session.clone(),
                version,
                entry,
                events,
                rpc: None,
                files: None,
                evidence: TurnEvidence::default(),
                prompt: String::new(),
                active: false,
                native_before_calls: 0,
                dispatched: false,
                native_outcome: false,
                callbacks: 0,
            },
            None,
        ));
        Ok(session)
    }
}
fn baseline_key(key: &str) -> bool {
    ["GROK_", "XAI_", "DYLD_", "LD_", "NODE_", "BUN_", "OPENSSL_"]
        .iter()
        .any(|prefix| key.starts_with(prefix))
        || [
            "SSLKEYLOGFILE",
            "BASH_ENV",
            "ENV",
            "SHELL",
            "ZDOTDIR",
            "HOME",
            "PATH",
            "TMPDIR",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_STATE_HOME",
            "XDG_CACHE_HOME",
            "NODE_OPTIONS",
            "SSL_CERT_FILE",
            "SSL_CERT_DIR",
            "REQUESTS_CA_BUNDLE",
            "CURL_CA_BUNDLE",
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "NO_PROXY",
            "http_proxy",
            "https_proxy",
            "all_proxy",
            "no_proxy",
        ]
        .contains(&key)
}
// Additional scoped values have no native loader/auth/permission selector meaning.
fn ordinary_key(key: &str) -> bool {
    key.starts_with("RRX_")
        || ["LANG", "LC_ALL", "LC_CTYPE", "TERM", "COLORTERM", "TZ"].contains(&key)
}
fn assert_saved(store: &SharedStore, session: &Session) -> AdapterResult<u64> {
    let store = store
        .lock()
        .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?;
    let (current, version) = store
        .session(session.id)
        .map_err(state_error)?
        .ok_or_else(|| failure(ErrorKind::SessionLost, "native persisted Session missing"))?;
    if serde_json::to_value(current)
        .map_err(|_| failure(ErrorKind::StateFailure, "Session encoding failed"))?
        != serde_json::to_value(session)
            .map_err(|_| failure(ErrorKind::StateFailure, "Session encoding failed"))?
    {
        return Err(failure(
            ErrorKind::StateConflict,
            "private native Session snapshot changed",
        ));
    }
    Ok(version)
}
fn save_current(
    store: &SharedStore,
    session: &Session,
    version: u64,
    snapshot: &ScopeSnapshot,
) -> AdapterResult<u64> {
    store
        .lock()
        .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?
        .put_session_if_current(
            session,
            version,
            [
                snapshot.project.version,
                snapshot.goal.as_ref().expect("Task Goal").version,
                snapshot.task.as_ref().expect("Task").version,
            ],
            &snapshot
                .locks
                .iter()
                .map(|lock| (lock.id, lock.version))
                .collect::<Vec<_>>(),
        )
        .map_err(state_error)
}
impl AgentAdapter for GrokAdapter {
    fn capabilities(&self) -> BTreeSet<Capability> {
        [
            Capability::Execute,
            Capability::Consult,
            Capability::Review,
            Capability::NonInteractive,
            Capability::Resume,
            Capability::StructuredOutput,
            Capability::UsageTelemetry,
            Capability::PromptCacheTelemetry,
            Capability::ContextCheckpoint,
        ]
        .into()
    }
    fn probe(&self) -> AdapterResult<AgentInfo> {
        Ok(AgentInfo {
            agent: self.agent.clone(),
            provider: "grok".into(),
            adapter_version: env!("CARGO_PKG_VERSION").into(),
            executable: resolve_executable(
                self.executable.to_str().expect("validated executable"),
            )?,
            authenticated: None,
            model_configuration: true,
            effort_configuration: true,
            capabilities: self.capabilities(),
        })
    }
    fn start(&self, request: LaunchRequest) -> AdapterFuture<'_, Session> {
        Box::pin(async move {
            let schema = if request.role == SessionRole::Reviewer {
                Some(json!({"type":"object"}))
            } else {
                None
            };
            self.launch(request, schema).await
        })
    }
    fn status(&self, reference: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        Box::pin(async move { Ok(self.entry(&reference)?.status.borrow().clone()) })
    }
    fn subscribe(&self, reference: SessionRef) -> AdapterResult<watch::Receiver<SessionStatus>> {
        Ok(self.entry(&reference)?.status.clone())
    }
    fn transport_succeeded(&self, status: &SessionStatus) -> bool {
        let Ok(entry) = self.entry(&(&status.session).into()) else {
            return false;
        };
        let cached = entry.status.borrow();
        status.session.state == SessionState::Exited
            && status.session.pid.is_none()
            && status.failure.is_none()
            && entry.completed.load(Ordering::SeqCst)
            && entry.cleaned.load(Ordering::SeqCst)
            && serde_json::to_value(&cached.session).ok()
                == serde_json::to_value(&status.session).ok()
            && cached.exit_code == status.exit_code
            && cached.stdout == status.stdout
            && cached.failure == status.failure
    }
    fn stop(&self, reference: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        Box::pin(async move {
            let entry = self.entry(&reference)?;
            let mut status = entry.status.clone();
            if !status.borrow().terminal() {
                entry.stopping.store(true, Ordering::SeqCst);
                entry.stop.notify_one();
                tokio::time::timeout(Duration::from_secs(5), async {
                    while !status.borrow().terminal() {
                        status.changed().await.map_err(|_| {
                            failure(ErrorKind::SessionLost, "native actor disappeared")
                        })?;
                    }
                    Ok::<_, AdapterError>(())
                })
                .await
                .map_err(|_| failure(ErrorKind::Timeout, "owned native stop not yet verified"))??;
            }
            Ok(status.borrow().clone())
        })
    }
    fn attach(&self, _: SessionRef) -> AdapterFuture<'_, ()> {
        Box::pin(async { Err(unsupported(Capability::Attach)) })
    }
    fn release(&self, reference: SessionRef) -> AdapterResult<()> {
        let entry = self.entry(&reference)?;
        if entry
            .busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(failure(ErrorKind::StateConflict, "native Session is busy"));
        }
        let mut retirement = Busy(Some(entry.clone()));
        if !entry.cleaned.load(Ordering::SeqCst)
            || !entry.status.borrow().terminal()
            || entry.status.borrow().session.state == SessionState::Lost
        {
            return Err(failure(
                ErrorKind::SessionLost,
                "native Session is live or uncertain; reservation retained",
            ));
        }
        self.sessions
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "registry poisoned"))?
            .remove(&reference.id);
        // A caller may already hold this Arc from before registry removal. Keep
        // its begin fence closed forever; failed retirement restores it via Drop.
        retirement.0 = None;
        Ok(())
    }
    fn checkpoint(&self, reference: SessionRef, input: PreparedInput) -> AdapterFuture<'_, ()> {
        Box::pin(async move {
            let entry = self.entry(&reference)?;
            if entry
                .busy
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                return Err(failure(ErrorKind::StateConflict, "native Session is busy"));
            }
            let _busy = Busy(Some(entry.clone()));
            let current = entry.status.borrow().session.clone();
            if !entry.cleaned.load(Ordering::SeqCst)
                || current.state == SessionState::Lost
                || input.version <= current.recovery["input_version"].as_u64().unwrap_or(0)
            {
                return Err(failure(
                    ErrorKind::InvalidInput,
                    "resume checkpoint must be fresh and prior termination verified",
                ));
            }
            let mut request = entry
                .request
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "request poisoned"))?
                .clone();
            request.input = input;
            let current_project = self
                .store
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "state poisoned"))?
                .project(reference.scope.project_id)
                .map_err(state_error)?
                .ok_or_else(|| failure(ErrorKind::SessionLost, "Project missing"))?;
            if request.project.id != current_project.id
                || request.project.root != current_project.root
                || request.project.repository_identity != current_project.repository_identity
                || request.project.base_branch != current_project.base_branch
                || request.project.worktree_root != current_project.worktree_root
            {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "checkpoint cannot rebind native Project identity",
                ));
            }
            request.project = current_project;
            if request.scope != reference.scope {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "checkpoint scope mismatch",
                ));
            }
            let snapshot = ScopeSnapshot::capture(&self.store, &request, &self.agent)?;
            let mut ownership = ProcessOwnership::default();
            snapshot.verify_git(&request, &mut ownership).await?;
            snapshot.recheck(&self.store, &request, &self.agent)?;
            let version = assert_saved(&self.store, &current)?;
            let mut next = current;
            next.recovery["checkpoint_version"] = json!(request.input.version);
            let _transition = entry
                .transition
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "transition poisoned"))?;
            save_current(&self.store, &next, version, &snapshot)?;
            // Sender remains owned by actor after terminal; use a retained sender clone below.
            *entry
                .request
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "request poisoned"))? = request;
            // Checkpoint metadata is persisted without rewriting the private terminal snapshot.
            // The exact next snapshot is published through the entry's retained sender.
            entry_sender(&entry).send_modify(|status| status.session = next);
            Ok(())
        })
    }
    fn resume(&self, reference: SessionRef) -> AdapterFuture<'_, Session> {
        Box::pin(async move {
            let entry = self.entry(&reference)?;
            if entry
                .busy
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "native resume already active",
                ));
            }
            let mut busy = Busy(Some(entry.clone()));
            let mut session = entry.status.borrow().session.clone();
            let request = entry
                .request
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "request poisoned"))?
                .clone();
            if !entry.cleaned.load(Ordering::SeqCst)
                || session.state == SessionState::Lost
                || !matches!(
                    session.state,
                    SessionState::Exited | SessionState::Stopped | SessionState::Failed
                )
                || request.input.version <= session.recovery["input_version"].as_u64().unwrap_or(0)
            {
                return Err(failure(
                    ErrorKind::InvalidInput,
                    "native resume requires verified death and explicit higher checkpoint input version",
                ));
            }
            let native = session
                .native_ref
                .clone()
                .ok_or_else(|| failure(ErrorKind::SessionLost, "native UUID unavailable"))?;
            let snapshot = ScopeSnapshot::capture(&self.store, &request, &self.agent)?;
            let environment = self.environment(&request)?;
            let version = assert_saved(&self.store, &session)?;
            session.state = SessionState::Starting;
            let transition = entry
                .transition
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "transition poisoned"))?;
            let version = save_current(&self.store, &session, version, &snapshot)?;
            entry.completed.store(false, Ordering::SeqCst);
            entry.cleaned.store(false, Ordering::SeqCst);
            *entry
                .usage
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "usage poisoned"))? = Value::Null;
            entry.stopping.store(false, Ordering::SeqCst);
            let events = entry_sender(&entry);
            events.send_modify(|status| {
                status.session = session.clone();
                status.failure = None;
                status.exit_code = None;
                status.stdout.clear();
                status.stderr.clear();
            });
            drop(transition);
            busy.0.take();
            tokio::spawn(supervise(
                Actor {
                    store: self.store.clone(),
                    agent: self.agent.clone(),
                    executable: self.executable.clone(),
                    request,
                    environment,
                    snapshot,
                    session: session.clone(),
                    version,
                    entry,
                    events,
                    rpc: None,
                    files: None,
                    evidence: TurnEvidence::default(),
                    prompt: String::new(),
                    active: false,
                    native_before_calls: 0,
                    dispatched: false,
                    native_outcome: false,
                    callbacks: 0,
                },
                Some(native),
            ));
            Ok(session)
        })
    }
    fn usage(
        &self,
        reference: SessionRef,
        phase: String,
        review_round: Option<u32>,
    ) -> AdapterFuture<'_, Usage> {
        Box::pin(async move {
            let entry = self.entry(&reference)?;
            let session = entry.status.borrow().session.clone();
            let usage = entry
                .usage
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "usage poisoned"))?
                .clone();
            Ok(Usage {
                scope: session.scope,
                session_id: session.id,
                agent: session.agent,
                phase,
                review_round,
                input_tokens: usage["inputTokens"].as_u64(),
                cached_input_tokens: usage["cachedReadTokens"].as_u64(),
                output_tokens: usage["outputTokens"].as_u64(),
                estimated_cost: None,
                context_pack_version: session.recovery["input_version"].as_u64(),
                context_pack_size: session.recovery["input_bytes"].as_u64(),
                repo_map_size: None,
                cache_metadata: json!({"cache_creation_tokens":usage["cacheCreationTokens"].as_u64()}),
                missing_reason: Some(
                    "cost conversion uncontracted; absent native aggregate counters remain null"
                        .into(),
                ),
            })
        })
    }
}

// The entry retains its sender so explicit checkpoint publication is available after actor exit.
fn entry_sender(entry: &OwnedEntry) -> watch::Sender<SessionStatus> {
    entry.events.clone()
}

struct Actor {
    store: SharedStore,
    agent: String,
    executable: PathBuf,
    request: LaunchRequest,
    environment: BTreeMap<String, String>,
    snapshot: ScopeSnapshot,
    session: Session,
    version: u64,
    entry: Arc<OwnedEntry>,
    events: watch::Sender<SessionStatus>,
    rpc: Option<Rpc>,
    files: Option<Arc<Mutex<ScopedFiles>>>,
    evidence: TurnEvidence,
    prompt: String,
    active: bool,
    native_before_calls: u64,
    dispatched: bool,
    native_outcome: bool,
    callbacks: usize,
}
impl Actor {
    fn owner(&self) -> AdapterResult<()> {
        self.snapshot
            .recheck_scope(&self.store, &self.request, &self.agent)?;
        let version = assert_saved(&self.store, &self.session)?;
        if version != self.version {
            return Err(failure(
                ErrorKind::StateConflict,
                "native Session version changed",
            ));
        }
        Ok(())
    }
    fn publish(&mut self) -> AdapterResult<()> {
        let _transition = self
            .entry
            .transition
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "transition poisoned"))?;
        self.version = save_current(&self.store, &self.session, self.version, &self.snapshot)?;
        self.events
            .send_modify(|status| status.session = self.session.clone());
        Ok(())
    }
    fn stopped(&self) -> bool {
        self.entry.stopping.load(Ordering::SeqCst)
    }
    async fn request(
        &mut self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> AdapterResult<Value> {
        self.owner()?;
        if self.stopped() {
            return Err(failure(ErrorKind::ProcessFailure, "native stop requested"));
        }
        let (id, frame) = self
            .rpc
            .as_mut()
            .expect("owned RPC")
            .frame(method, params)?;
        self.rpc.as_mut().expect("owned RPC").send(&frame).await?;
        self.response(id, timeout).await
    }
    async fn response(&mut self, id: u64, timeout: Duration) -> AdapterResult<Value> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if self.stopped() {
                return Err(failure(ErrorKind::ProcessFailure, "native stop requested"));
            }
            let value = tokio::select! {
                _=self.entry.stop.notified()=>{if self.stopped(){return Err(failure(ErrorKind::ProcessFailure,"native stop requested"));}continue;},
                value=tokio::time::timeout_at(deadline,self.rpc.as_mut().expect("owned RPC").receive())=>value.map_err(|_|failure(ErrorKind::Timeout,"native ACP request timed out"))??,
            };
            if value.get("method").is_some() {
                if value.get("id").is_some() {
                    self.callback(value).await?;
                } else if (self.active || self.dispatched) && value["method"] == "session/update" {
                    if !self.active
                        && value["params"]["update"]["sessionUpdate"] != "session_info_update"
                    {
                        return Err(failure(
                            ErrorKind::OwnershipMismatch,
                            "native turn notification after terminal response",
                        ));
                    }
                    if let Some(text) = self.evidence.update(
                        &self.request.worktree,
                        &value["params"],
                        self.session
                            .native_ref
                            .as_deref()
                            .expect("active native UUID"),
                        &self.prompt,
                        self.request.role != SessionRole::Executor,
                    )? {
                        self.events
                            .send_modify(|status| append_output(status, text.as_bytes()));
                    }
                }
                continue;
            }
            if value["id"].as_u64() != Some(id)
                || (value.get("result").is_some() == value.get("error").is_some())
            {
                return Err(failure(
                    ErrorKind::ParseFailure,
                    format!(
                        "ACP response mismatch actual_numeric_present={} actual_numeric_equals_expected={} actual_string_id={} expected_decimal_string={} result_end_turn={} result_native_matches={} result_prompt_matches={} has_result={} has_error={}",
                        value["id"].as_u64().is_some(),
                        value["id"].as_u64() == Some(id),
                        value["id"].is_string(),
                        value["id"].as_str() == Some(id.to_string().as_str()),
                        value["result"]["stopReason"] == "end_turn",
                        self.session
                            .native_ref
                            .as_deref()
                            .is_some_and(|native| value["result"]["_meta"]["sessionId"].as_str()
                                == Some(native)),
                        value["result"]["_meta"]["promptId"] == self.prompt,
                        value.get("result").is_some(),
                        value.get("error").is_some()
                    ),
                ));
            }
            if value.get("error").is_some() {
                if self.active {
                    self.native_outcome = true;
                }
                return Err(failure(
                    ErrorKind::ProcessFailure,
                    "native ACP returned error",
                ));
            }
            return Ok(value["result"].clone());
        }
    }
    async fn callback(&mut self, value: Value) -> AdapterResult<()> {
        if !(value["id"].as_u64().is_some() || value["id"].as_str().is_some_and(|s| s.len() <= 128))
        {
            return Err(failure(
                ErrorKind::ParseFailure,
                "invalid reverse request ID before side effects",
            ));
        }
        self.callbacks += 1;
        if self.callbacks > 256
            || value["params"]["path"]
                .as_str()
                .is_some_and(|s| s.len() > 4096)
        {
            return Err(failure(
                ErrorKind::InvalidInput,
                "native callback/path budget exceeded",
            ));
        }
        let method = value["method"].as_str().unwrap_or("");
        let params = &value["params"];
        let owner = self.owner();
        let valid = self.active
            && !self.stopped()
            && params["sessionId"].as_str() == self.session.native_ref.as_deref()
            && owner.is_ok();
        let mut fatal = !valid
            || ![
                "fs/read_text_file",
                "fs/write_text_file",
                "session/request_permission",
            ]
            .contains(&method)
            || (self.request.role != SessionRole::Executor
                && (method.starts_with("fs/") || method == "session/request_permission"));
        let result = if !valid {
            Err(failure(
                ErrorKind::OwnershipMismatch,
                "inactive/foreign callback",
            ))
        } else if method == "session/request_permission" {
            let option = params["options"]
                .as_array()
                .and_then(|a| a.iter().find(|v| v["kind"] == "reject_once"))
                .and_then(|v| v["optionId"].as_str());
            if let Some(option) = option {
                Ok(json!({"outcome":{"outcome":"selected","optionId":option}}))
            } else {
                fatal = true;
                Ok(json!({"outcome":{"outcome":"cancelled"}}))
            }
        } else if self.request.role == SessionRole::Executor
            && ["fs/read_text_file", "fs/write_text_file"].contains(&method)
        {
            let path = params["path"].as_str().unwrap_or("");
            let files = self.files.as_ref().expect("executor FS").clone();
            let params = params.clone();
            let write = method == "fs/write_text_file";
            if write {
                self.evidence
                    .authorize_write(&self.request.worktree, path)?;
            }
            // Mutating workers are never detached on timeout/cancel. Until join returns,
            // stop cannot imply their death or free the executor reservation.
            let started = tokio::time::Instant::now();
            let (result, effect_may_have_occurred) = tokio::task::spawn_blocking(move || {
                let mut files = files
                    .lock()
                    .map_err(|_| failure(ErrorKind::StateFailure, "scoped FS poisoned"))?;
                if write {
                    Ok(files.write_observed(&params))
                } else {
                    Ok((files.read(&params), false))
                }
            })
            .await
            .map_err(|_| failure(ErrorKind::ProcessFailure, "scoped FS worker failed"))??;
            self.store.lock().map_err(|_|failure(ErrorKind::StateFailure,"state poisoned"))?.audit(&self.session.scope,"grok.fs_observed",json!({"session":self.session.id,"native":self.session.native_ref,"prompt":self.prompt,"method":method,"path":path,"succeeded":result.is_ok(),"effect_may_have_occurred":effect_may_have_occurred})).map_err(state_error)?;
            self.evidence.callback(
                &self.request.worktree,
                path,
                method,
                result.is_ok(),
                effect_may_have_occurred,
            )?;
            if started.elapsed() > Duration::from_secs(5) {
                fatal = true;
            }
            let current = self.owner();
            if current.is_err() {
                fatal = true;
            }
            if self.stopped() {
                fatal = true;
                Err(failure(
                    ErrorKind::ProcessFailure,
                    "stop requested during FS work",
                ))
            } else {
                current.and(result)
            }
        } else {
            Err(failure(
                ErrorKind::UnsupportedCapability,
                "reverse capability unsupported",
            ))
        };
        self.rpc
            .as_mut()
            .expect("owned RPC")
            .reply(value["id"].clone(), result)
            .await?;
        if fatal {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "unexpected or stale native callback; no grant",
            ));
        }
        Ok(())
    }
    fn inventory_gate(&mut self, info: &Value) -> AdapterResult<()> {
        let data = &info["result"];
        let expected = if self.request.role == SessionRole::Executor {
            "rururunx-executor"
        } else {
            "rururunx-decision"
        };
        if data["sessionId"].as_str() != self.session.native_ref.as_deref()
            || data["cwd"].as_str() != self.request.worktree.to_str()
            || data["agentName"] != expected
            || data["context"]["toolDefinitionsCount"].as_u64()
                != Some(if self.request.role == SessionRole::Executor {
                    2
                } else {
                    0
                })
        {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "native curated identity/workspace/tool contract mismatch",
            ));
        }
        let calls = data["context"]["toolCallCount"]
            .as_u64()
            .ok_or_else(|| failure(ErrorKind::ParseFailure, "native tool-call evidence missing"))?;
        if self.request.role != SessionRole::Executor && calls != 0 {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "decision native tool calls observed",
            ));
        }
        if !self.dispatched {
            self.native_before_calls = calls;
        } else if self
            .native_before_calls
            .checked_add(self.evidence.tool_count() as u64)
            != Some(calls)
        {
            return Err(failure(
                ErrorKind::ParseFailure,
                "native tool count differs from owned live notifications",
            ));
        }
        Ok(())
    }
}
fn append_output(status: &mut SessionStatus, bytes: &[u8]) {
    const LIMIT: usize = 65_536;
    let excess = status
        .stdout
        .len()
        .saturating_add(bytes.len())
        .saturating_sub(LIMIT);
    if excess > 0 {
        status.stdout_truncated = true;
        status.stdout.drain(..excess.min(status.stdout.len()));
    }
    status
        .stdout
        .extend_from_slice(&bytes[bytes.len().saturating_sub(LIMIT)..]);
}

async fn supervise(mut actor: Actor, load: Option<String>) {
    let mut busy_guard = Busy(Some(actor.entry.clone()));
    let mut ownership = ProcessOwnership::default();
    let mut process = None;
    let mut stderr = None;
    let mut profile = None;
    let mut baseline = None;
    let mut binding = None;
    let mut index = None;
    let result=async {
        actor.owner()?;
        let observed_binding=actor.snapshot.verify_git(&actor.request,&mut ownership).await?;binding=Some(observed_binding);actor.owner()?;
        index=Some(index_digest(&actor.request.worktree,&mut ownership).await?);
        let root=actor.request.worktree.clone();baseline=Some(filesystem(move||Inventory::capture(&root)).await?);
        if actor.request.role==SessionRole::Executor {
            let root=actor.request.worktree.clone();let project=actor.request.project.clone();actor.files=Some(Arc::new(Mutex::new(filesystem(move||ScopedFiles::new(root,&project)).await?)));
        }
        let decision=actor.request.role!=SessionRole::Executor;
        let owned=filesystem(move||Profile::new(decision)).await?;
        let expected_profile=owned.hash.clone();if load.is_some() && actor.session.recovery["profile_digest"].as_str()!=Some(&expected_profile){return Err(failure(ErrorKind::OwnershipMismatch,"loaded Session profile contract changed"));}owned.verify()?;
        let mut command=Command::new(&actor.executable);
        command.args(["--disable-web-search","--sandbox",if decision{"read-only"}else{"strict"},"agent","--no-leader","--agent-profile"]).arg(&owned.path).arg("stdio")
            .current_dir(&actor.request.worktree).env_clear().envs(&actor.environment).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true).process_group(0);
        actor.owner()?;if actor.stopped(){return Err(failure(ErrorKind::ProcessFailure,"native stop before spawn"));}
        let child=ProcessGroup::new(command.spawn().map_err(|e|failure(if e.kind()==std::io::ErrorKind::NotFound {ErrorKind::ExecutableMissing}else{ErrorKind::LaunchFailure},"native Grok could not spawn"))?,ownership.group())?;
        #[cfg(all(test,target_os="macos"))]
        let child = { let mut child = child; child.inspection_plan = actor.entry.process_inspection.clone(); child };
        actor.session.pid=child.child.id();process=Some(child);
        actor.store.lock().map_err(|_|failure(ErrorKind::StateFailure,"state poisoned"))?.audit(&actor.session.scope,"grok.process_spawned",json!({"session":actor.session.id,"pid":actor.session.pid})).map_err(state_error)?;
        actor.publish()?;
        let child=process.as_mut().expect("owned child");
        actor.rpc=Some(Rpc::new(child.child.stdin.take().expect("piped stdin"),child.child.stdout.take().expect("piped stdout")));
        stderr=Some(tokio::spawn(drain(child.child.stderr.take().expect("piped stderr"),actor.events.clone(),false)));profile=Some(owned);
        let initialize=actor.request("initialize",json!({"protocolVersion":1,"clientCapabilities":{"fs":{"readTextFile":!decision,"writeTextFile":!decision},"terminal":false},"_meta":{"startupHints":{"nonInteractive":true}}}),Duration::from_secs(30)).await?;
        if initialize["protocolVersion"]!=1 || initialize["_meta"]["agentVersion"]!=NATIVE_VERSION || initialize["agentCapabilities"]["loadSession"]!=true {return Err(failure(ErrorKind::UnsupportedCapability,"untested native Grok protocol/version/load contract"));}
        if !initialize["authMethods"].as_array().is_some_and(|a|a.iter().any(|v|v["id"]=="cached_token")){return Err(failure(ErrorKind::AuthenticationUnavailable,"native cached-token method unavailable"));}
        actor.request("authenticate",json!({"methodId":"cached_token","_meta":{"headless":true}}),Duration::from_secs(30)).await.map_err(|e|if e.kind==ErrorKind::ProcessFailure && e.message=="native ACP returned error" {failure(ErrorKind::AuthenticationUnavailable,"native cached authentication unavailable; no interactive fallback")}else{e})?;
        let mut params=json!({"cwd":actor.request.worktree,"mcpServers":[]});if let Some(id)=&load{params["sessionId"]=json!(id);}
        let session=actor.request(if load.is_some(){"session/load"}else{"session/new"},params,Duration::from_secs(30)).await?;
        let native=if let Some(id)=load {id}else{session["sessionId"].as_str().ok_or_else(||failure(ErrorKind::ParseFailure,"native Session ID missing"))?.to_owned()};
        uuid::Uuid::parse_str(&native).map_err(|_|failure(ErrorKind::ParseFailure,"invalid native Session UUID"))?;
        actor.session.native_ref=Some(native.clone());
        actor.session.recovery=json!({"reconnect_supported":false,"native_version":NATIVE_VERSION,"native_capabilities":initialize["agentCapabilities"],"profile_digest":expected_profile,"input_version":actor.session.recovery["input_version"],"input_revision":actor.request.input.revision,"source_versions":actor.request.input.source_versions,"input_bytes":actor.request.input.payload.len()});
        actor.publish()?;
        let model=actor.request.model.clone();let effort=actor.request.effort.clone();let mut options=session["configOptions"].clone();
        for (key,value) in [("model",model.as_ref()),("reasoning_effort",effort.as_ref())]{if let Some(value)=value {
            options=actor.request("session/set_config_option",json!({"sessionId":native,"configId":key,"value":value}),Duration::from_secs(30)).await?["configOptions"].clone();
        }}
        for (key,value) in [("model",model.as_ref()),("reasoning_effort",effort.as_ref())]{if let Some(value)=value
            && !options.as_array().is_some_and(|a|a.iter().any(|v|v["id"]==key && v["currentValue"]==*value)){return Err(failure(ErrorKind::InvalidConfiguration,"native model/effort configuration did not match"));}}
        let info=actor.request("_x.ai/session/info",json!({"sessionId":native}),Duration::from_secs(15)).await?;actor.inventory_gate(&info)?;
        actor.snapshot.verify_binding(&actor.request,&mut ownership,binding.as_ref().expect("preflight binding")).await?;actor.owner()?;profile.as_ref().expect("owned profile").verify()?;
        actor.prompt=uuid::Uuid::new_v4().to_string();
        let mut params=json!({"sessionId":native,"prompt":[{"type":"text","text":format!("Prepared Task input follows:\n\n{}",actor.request.input.payload)}],"_meta":{"promptId":actor.prompt,"screenMode":"headless"}});
        if let Some(schema)=&actor.entry.schema{params["_meta"]["outputSchema"]=schema.clone();}
        let (id,frame)=actor.rpc.as_mut().expect("RPC").frame("session/prompt",params)?;
        actor.session.state=SessionState::Running;actor.session.recovery["input_version"]=json!(actor.request.input.version);actor.session.recovery["prompt_id"]=json!(actor.prompt);actor.session.recovery["dispatch_state"]=json!("dispatching");
        actor.session.recovery["dispatch_intent"]=json!({"input_version":actor.request.input.version,"prompt_id":actor.prompt});
        actor.publish()?; // Durable consumption before the only dispatch write.
        actor.dispatched=true;
        actor.rpc.as_mut().expect("RPC").send(&frame).await?;
        actor.active=true;
        let response=actor.response(id,TURN_TIMEOUT).await?;actor.active=false;
        if response["_meta"]["sessionId"]==native && response["_meta"]["promptId"]==actor.prompt && ["end_turn","max_tokens","max_turn_requests","refusal","cancelled"].contains(&response["stopReason"].as_str().unwrap_or("")){actor.native_outcome=true;}
        if actor.native_outcome { *actor.entry.usage.lock().map_err(|_|failure(ErrorKind::StateFailure,"usage poisoned"))?=response["_meta"]["usage"].clone(); }
        if !actor.native_outcome || response["stopReason"]!="end_turn" {return Err(failure(ErrorKind::ProcessFailure,"native turn did not complete with owned end_turn evidence"));}
        actor.store.lock().map_err(|_|failure(ErrorKind::StateFailure,"state poisoned"))?.audit(&actor.session.scope,"grok.native_turn_observed",json!({"session":actor.session.id,"native":native,"prompt":actor.prompt,"stop_reason":"end_turn","structured_output_digest":response["_meta"].get("structuredOutput").and_then(|v|serde_json::to_vec(v).ok()).map(|v|digest(&v))})).map_err(state_error)?;
        for field in ["serverSideToolCalls","serverToolCalls","webSearchCalls","xSearchCalls","searchSources"] {
            if response["_meta"].get(field).is_some_and(|v| v!=&Value::Null && v!=&json!(0) && v!=&json!([])) {return Err(failure(ErrorKind::OwnershipMismatch,"native hosted tool/search evidence outside contract"));}
        }
        let info=actor.request("_x.ai/session/info",json!({"sessionId":native}),Duration::from_secs(15)).await?;actor.inventory_gate(&info)?;
        actor.evidence.finished()?;
        if response["_meta"].get("structuredOutputError").is_some_and(|v|!v.is_null()){return Err(failure(ErrorKind::ParseFailure,"native structured output failed"));}
        if let Some(schema)=&actor.entry.schema {
            let output=response["_meta"].get("structuredOutput").ok_or_else(||failure(ErrorKind::ParseFailure,"native structured output missing"))?;
            schema::validate(schema,output)?;
            let bytes=serde_json::to_vec(output).map_err(|_|failure(ErrorKind::ParseFailure,"structured output encoding failed"))?;
            actor.events.send_modify(|status|{status.stdout=bytes;status.stdout_truncated=false;});
        }

        actor.owner()?;
        actor.request("session/close",json!({"sessionId":native}),Duration::from_secs(15)).await?;
        Ok::<_,AdapterError>(())
    }.await;
    actor.active = false;
    // Native completion remains provisional until all owned process/host work is done.
    let cleanup = if let Some(child) = process {
        match cleanup_group(child).await {
            Ok(mut child) => tokio::time::timeout(Duration::from_millis(250), child.reap())
                .await
                .map_err(|_| failure(ErrorKind::SessionLost, "native reap uncertain"))
                .and_then(|v| v.map_err(|_| failure(ErrorKind::SessionLost, "native reap failed")))
                .map(|status| status.code()),
            Err(error) => Err(error),
        }
    } else {
        Ok(None)
    };
    actor.rpc.take();
    let mut output_verified = true;
    if let Some(mut reader) = stderr
        && tokio::time::timeout(Duration::from_millis(250), &mut reader)
            .await
            .is_err()
    {
        reader.abort();
        output_verified = false;
    }
    drop(profile);
    let reconciliation_attempted = cleanup.is_ok() && (result.is_ok() || actor.dispatched);
    let reconciled = if reconciliation_attempted {
        async {
            actor
                .snapshot
                .verify_binding(
                    &actor.request,
                    &mut ownership,
                    binding.as_ref().expect("owned binding"),
                )
                .await?;
            let current_index = index_digest(&actor.request.worktree, &mut ownership).await?;
            if index.as_ref() != Some(&current_index) {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "native index changed outside ACP writes",
                ));
            }
            let root = actor.request.worktree.clone();
            let after = filesystem(move || Inventory::capture(&root)).await;
            after.and_then(|after| {
                let writes = actor
                    .files
                    .as_ref()
                    .map(|files| {
                        files
                            .lock()
                            .map(|f| f.writes.clone())
                            .map_err(|_| failure(ErrorKind::StateFailure, "FS poisoned"))
                    })
                    .transpose()?
                    .unwrap_or_default();
                baseline
                    .as_ref()
                    .ok_or_else(|| failure(ErrorKind::StateFailure, "baseline missing"))?
                    .reconcile(&after, &writes)
            })
        }
        .await
    } else {
        Ok(())
    };
    let clean = cleanup.is_ok() && !ownership.uncertain() && output_verified;
    let completed = result.is_ok() && reconciled.is_ok() && clean && !actor.stopped();
    let reconciliation_error = reconciled.as_ref().err().map(ToString::to_string);
    let mut diagnostic = result
        .err()
        .or_else(|| {
            cleanup
                .as_ref()
                .err()
                .map(|e| failure(e.kind, e.message.clone()))
        })
        .or_else(|| reconciled.err());
    if actor.dispatched && !actor.native_outcome {
        diagnostic = Some(failure(
            ErrorKind::SessionLost,
            format!(
                "native dispatch outcome unknown; reservation retained: {}",
                diagnostic
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "stop requested".into())
            ),
        ));
    }
    let _=actor.store.lock().map(|mut store|store.audit(&actor.session.scope,"grok.turn_observed",json!({"session":actor.session.id,"native":actor.session.native_ref,"prompt":actor.prompt,"completed":completed,"cleanup_verified":clean,"reconciliation_attempted":reconciliation_attempted,"reconciliation_error":reconciliation_error,"exit_code":cleanup.as_ref().ok().copied().flatten(),"diagnostic":diagnostic.as_ref().map(ToString::to_string)})));
    let state = if !clean || (actor.dispatched && !actor.native_outcome) {
        SessionState::Lost
    } else if actor.stopped() {
        SessionState::Stopped
    } else if completed {
        SessionState::Exited
    } else {
        SessionState::Failed
    };
    if let Ok(store) = actor.store.lock()
        && let Ok(Some((saved, version))) = store.session(actor.session.id)
        && version == actor.version
    {
        actor.session = saved;
    }
    actor.session.state = state;
    if clean {
        actor.session.pid = None;
    }
    let transition = actor.entry.transition.lock();
    let saved = save_session(&actor.store, &actor.session, actor.version);
    actor.entry.cleaned.store(clean, Ordering::SeqCst);
    actor
        .entry
        .completed
        .store(completed && saved.is_ok(), Ordering::SeqCst);
    busy_guard.publish_idle();
    actor.events.send_modify(|status| {
        status.session = actor.session.clone();
        status.exit_code = cleanup.ok().flatten();
        status.failure = diagnostic.map(|e| e.to_string());
        if let Err(error) = &saved {
            status.session.state = SessionState::Lost;
            status.failure = Some(format!("native terminal persistence uncertain: {error}"));
        }
    });
    drop(transition);
}

struct Profile {
    _directory: tempfile::TempDir,
    path: PathBuf,
    hash: String,
    identity: (u64, u64),
}
impl Profile {
    fn new(decision: bool) -> AdapterResult<Self> {
        let directory = tempfile::Builder::new()
            .prefix("rrx-grok-")
            .tempdir()
            .map_err(|_| {
                failure(
                    ErrorKind::LaunchFailure,
                    "native profile directory unavailable",
                )
            })?;
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
            .map_err(|_| {
                failure(
                    ErrorKind::LaunchFailure,
                    "profile directory protection failed",
                )
            })?;
        let path = directory.path().join("agent.md");
        let text = format!(
            "---\nname: {}\ndescription: Scoped native Grok runtime profile preserving native instructions\ninjectDefaultTools: false\ntoolConfig:\n  tools:\n    - id: GrokBuild:read_file\n    - id: GrokBuild:search_replace\ntools: [read_file, search_replace]\ndisallowedTools: [{}search_tool, use_tool, web_search, x_search, web_fetch]\npermissionMode: {}\n---\n",
            if decision {
                "rururunx-decision"
            } else {
                "rururunx-executor"
            },
            if decision {
                "read_file, search_replace, "
            } else {
                ""
            },
            if decision { "dontAsk" } else { "default" }
        );
        std::fs::write(&path, &text)
            .map_err(|_| failure(ErrorKind::LaunchFailure, "profile write failed"))?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o400))
            .map_err(|_| failure(ErrorKind::LaunchFailure, "profile protection failed"))?;
        let metadata = path
            .symlink_metadata()
            .map_err(|_| failure(ErrorKind::LaunchFailure, "profile metadata unavailable"))?;
        Ok(Self {
            _directory: directory,
            path,
            hash: digest(text.as_bytes()),
            identity: (metadata.dev(), metadata.ino()),
        })
    }
    fn verify(&self) -> AdapterResult<()> {
        let metadata = self
            .path
            .symlink_metadata()
            .map_err(|_| failure(ErrorKind::OwnershipMismatch, "profile unavailable"))?;
        if !metadata.is_file()
            || metadata.mode() & 0o777 != 0o400
            || (metadata.dev(), metadata.ino()) != self.identity
            || std::fs::read(&self.path).map(|v| digest(&v)).ok().as_ref() != Some(&self.hash)
        {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "protected native profile changed",
            ));
        }
        Ok(())
    }
}

async fn index_digest(root: &Path, ownership: &mut ProcessOwnership) -> AdapterResult<String> {
    let executable = resolve_executable("git")?;
    let bytes = bounded_git_raw(
        &executable,
        root,
        &["ls-files".into(), "--stage".into(), "-z".into()],
        crate::git::native_environment(),
        tokio::time::Instant::now() + Duration::from_secs(5),
        ownership.group(),
    )
    .await?;
    Ok(digest(&bytes))
}

#[cfg(test)]
mod registry_tests {
    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn sanitized_native_process_unknown_cleanup_reaches_grok_durable_reservation() {
        let home = tempfile::tempdir().unwrap();
        let mut command = tokio::process::Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "adapter::grok::registry_tests::sanitized_grok_cleanup_child",
                "--ignored",
                "--nocapture",
            ])
            .env_clear()
            .env("HOME", home.path())
            .env("PATH", "/usr/bin:/bin")
            .env("RRX_INSPECTION_FIXTURE_CHILD", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .process_group(0);
        let mut child =
            ProcessGroup::new(command.spawn().unwrap(), Arc::new(AtomicBool::new(false))).unwrap();
        let stdout = tokio::spawn(read_git_output(child.child.stdout.take().unwrap()));
        let stderr = tokio::spawn(read_git_output(child.child.stderr.take().unwrap()));
        let observed = tokio::time::timeout(Duration::from_secs(60), child.observe_exit()).await;
        child = cleanup_group(child).await.unwrap();
        let exit = child.reap().await.unwrap();
        let output = stdout.await.unwrap().unwrap();
        let diagnostic = stderr.await.unwrap().unwrap();
        assert!(
            observed.is_ok(),
            "isolated child fixture timed out: {} {}",
            String::from_utf8_lossy(&output),
            String::from_utf8_lossy(&diagnostic)
        );
        assert!(
            exit.success(),
            "sanitized fixture failed: {} {}",
            String::from_utf8_lossy(&output),
            String::from_utf8_lossy(&diagnostic)
        );
    }
    #[cfg(target_os = "macos")]
    #[tokio::test]
    #[ignore = "entered only by the env-cleared parent fixture; no ambient native baseline"]
    async fn sanitized_grok_cleanup_child() {
        assert_eq!(std::env::var("RRX_INSPECTION_FIXTURE_CHILD").unwrap(), "1");
        for unknown in [false, true] {
            let (_temp, store, project, task, worktree) = super::super::tests::preflight_fixture();
            let revision = std::process::Command::new("/usr/bin/git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&worktree)
                .env_clear()
                .output()
                .unwrap();
            assert!(revision.status.success());
            let mut request = super::super::tests::fixture_request(project, &task, worktree);
            request.input.revision = String::from_utf8(revision.stdout)
                .unwrap()
                .trim()
                .to_owned();
            request.environment.clear();
            let mut adapter =
                GrokAdapter::new("fake".into(), PathBuf::from("/bin/cat"), store.clone()).unwrap();
            if unknown {
                adapter.process_inspection = Some(ProcessInspectionPlan::unknown(
                    UnknownObservation::Diagnostics,
                ));
            }
            let launched = adapter.start(request).await.unwrap();
            let status = tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    let status = adapter.status(SessionRef::from(&launched)).await.unwrap();
                    if status.terminal() {
                        break status;
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap();
            let state = store.lock().unwrap();
            let saved = state.session(launched.id).unwrap().unwrap().0;
            let events = state.events(&task.scope(), 0, 100).unwrap();
            assert!(
                events
                    .iter()
                    .any(|event| event.kind == "grok.process_spawned")
            );
            assert_eq!(
                saved.state,
                if unknown {
                    SessionState::Lost
                } else {
                    SessionState::Failed
                }
            );
            assert_eq!(status.session.state, saved.state);
            drop(state);
            assert!(!adapter.transport_succeeded(&status));
        }
    }
    use super::*;
    #[tokio::test]
    async fn actor_journals_failed_write_effect_and_denies_completion() {
        use crate::domain::{CompletionCriterion, Goal, Task};
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let worktree = root.join("worktree/task");
        std::fs::create_dir_all(&worktree).unwrap();
        let mut store = crate::state::Store::open(&root.join("state.db")).unwrap();
        let mut project = Project::new("fixture".into(), root, "fixture".into(), "main".into());
        store.put_project(&mut project).unwrap();
        let mut goal = Goal::new(
            project.id,
            "fixture".into(),
            vec![CompletionCriterion {
                id: "effect".into(),
                description: "observe actual failed write".into(),
                satisfied: false,
                evidence: None,
            }],
        );
        store.put_goal(&mut goal).unwrap();
        let mut task = Task::new(project.id, goal.id, "fixture".into(), "grok".into());
        task.worktree = Some(worktree.clone());
        task.branch = Some("feature/task".into());
        store.put_task(&mut task).unwrap();
        let request = LaunchRequest {
            project,
            scope: task.scope(),
            worktree: worktree.clone(),
            role: SessionRole::Executor,
            mode: LaunchMode::NonInteractive,
            input: PreparedInput {
                scope: task.scope(),
                kind: InputKind::ContextPack,
                revision: "a".repeat(40),
                version: 1,
                source_versions: BTreeMap::new(),
                payload: "fixture".into(),
            },
            environment: BTreeMap::new(),
            model: None,
            effort: None,
        };
        let store = Arc::new(Mutex::new(store));
        let snapshot = ScopeSnapshot::capture(&store, &request, "grok").unwrap();
        let session = Session {
            id: SessionId::new(),
            scope: task.scope(),
            agent: "grok".into(),
            provider: "grok".into(),
            role: SessionRole::Executor,
            native_ref: Some("native".into()),
            pid: None,
            worktree: worktree.clone(),
            state: SessionState::Running,
            model: None,
            effort: None,
            recovery: json!({}),
            started_at: now_ms(),
        };
        let version = store.lock().unwrap().put_session(&session, 0).unwrap();
        let (events, status) = watch::channel(SessionStatus {
            session: session.clone(),
            exit_code: None,
            stdout: vec![],
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
            failure: None,
        });
        let entry = Arc::new(OwnedEntry {
            #[cfg(all(test, target_os = "macos"))]
            process_inspection: None,
            scope: session.scope.clone(),
            transition: Mutex::new(()),
            status,
            events: events.clone(),
            request: Mutex::new(request.clone()),
            schema: None,
            busy: AtomicBool::new(true),
            stopping: AtomicBool::new(false),
            stop: Notify::new(),
            completed: AtomicBool::new(false),
            cleaned: AtomicBool::new(false),
            usage: Mutex::new(Value::Null),
        });
        let mut files = ScopedFiles::new(worktree.clone(), &request.project).unwrap();
        files.fail_sync = true;
        let mut child = tokio::process::Command::new("/bin/cat")
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let rpc = Rpc::new(child.stdin.take().unwrap(), child.stdout.take().unwrap());
        let mut actor = Actor {
            store: store.clone(),
            agent: "grok".into(),
            executable: PathBuf::from("/bin/cat"),
            request,
            environment: BTreeMap::new(),
            snapshot,
            session: session.clone(),
            version,
            entry,
            events,
            rpc: Some(rpc),
            files: Some(Arc::new(Mutex::new(files))),
            evidence: TurnEvidence::default(),
            prompt: "prompt".into(),
            active: true,
            native_before_calls: 0,
            dispatched: true,
            native_outcome: false,
            callbacks: 0,
        };
        actor.evidence.update(&worktree, &json!({"sessionId":"native","_meta":{"promptId":"prompt"},"update":{"sessionUpdate":"tool_call","toolCallId":"write","_meta":{"x.ai/tool":{"name":"search_replace"}},"status":"pending","rawInput":{"file_path":"effect.txt"}}}), "native", "prompt", false).unwrap();
        let callback = actor.callback(json!({"jsonrpc":"2.0","id":1,"method":"fs/write_text_file","params":{"sessionId":"native","path":"effect.txt","content":"applied before sync error"}})).await;
        let response = tokio::time::timeout(
            Duration::from_secs(5),
            actor.rpc.as_mut().unwrap().receive(),
        )
        .await;
        // Close the only stdin writer and reap the owned echo process before assertions.
        actor.rpc.take();
        let exited = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
        assert!(exited.unwrap().unwrap().success());
        callback.unwrap();
        assert!(response.unwrap().unwrap().get("error").is_some());
        assert_eq!(
            std::fs::read_to_string(worktree.join("effect.txt")).unwrap(),
            "applied before sync error"
        );
        let event = store
            .lock()
            .unwrap()
            .events(&session.scope, 0, 100)
            .unwrap()
            .into_iter()
            .find(|event| event.kind == "grok.fs_observed")
            .unwrap();
        assert_eq!(event.data["session"], json!(session.id));
        assert_eq!(event.data["native"], "native");
        assert_eq!(event.data["prompt"], "prompt");
        assert_eq!(event.data["succeeded"], false);
        assert_eq!(event.data["effect_may_have_occurred"], true);
        actor.evidence.update(&worktree, &json!({"sessionId":"native","_meta":{"promptId":"prompt"},"update":{"sessionUpdate":"tool_call_update","toolCallId":"write","status":"failed"}}), "native", "prompt", false).unwrap();
        assert_eq!(
            actor.evidence.finished().unwrap_err().kind,
            ErrorKind::ProcessFailure
        );
    }
    #[test]
    fn successful_retirement_permanently_fences_previously_handed_entry() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let store = Arc::new(Mutex::new(
            crate::state::Store::open(&root.join("state.db")).unwrap(),
        ));
        let mut project = Project::new(
            "fixture".into(),
            root.clone(),
            "fixture".into(),
            "main".into(),
        );
        store.lock().unwrap().put_project(&mut project).unwrap();
        let scope = Scope::project(project.id);
        let session = Session {
            id: SessionId::new(),
            scope: scope.clone(),
            agent: "grok".into(),
            provider: "grok".into(),
            role: SessionRole::Consultant,
            native_ref: None,
            pid: None,
            worktree: root.clone(),
            state: SessionState::Exited,
            model: None,
            effort: None,
            recovery: json!({}),
            started_at: now_ms(),
        };
        store.lock().unwrap().put_session(&session, 0).unwrap();
        let (events, status) = watch::channel(SessionStatus {
            session: session.clone(),
            exit_code: None,
            stdout: vec![],
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
            failure: None,
        });
        let entry = Arc::new(OwnedEntry {
            #[cfg(all(test, target_os = "macos"))]
            process_inspection: None,
            scope: scope.clone(),
            transition: Mutex::new(()),
            status,
            events,
            request: Mutex::new(LaunchRequest {
                project,
                scope: scope.clone(),
                worktree: root,
                role: SessionRole::Consultant,
                mode: LaunchMode::NonInteractive,
                input: PreparedInput {
                    scope,
                    kind: InputKind::ContextPack,
                    revision: "a".repeat(40),
                    version: 1,
                    source_versions: BTreeMap::new(),
                    payload: "fixture".into(),
                },
                environment: BTreeMap::new(),
                model: None,
                effort: None,
            }),
            schema: None,
            busy: AtomicBool::new(false),
            stopping: AtomicBool::new(false),
            stop: Notify::new(),
            completed: AtomicBool::new(false),
            cleaned: AtomicBool::new(false),
            usage: Mutex::new(Value::Null),
        });
        let adapter = GrokAdapter::new("grok".into(), PathBuf::from("/bin/sh"), store).unwrap();
        adapter
            .sessions
            .lock()
            .unwrap()
            .insert(session.id, entry.clone());
        // Models a concurrent begin that obtained the private Arc immediately
        // before release. Cleanup failure must keep it registered and usable.
        let previously_handed = adapter.entry(&(&session).into()).unwrap();
        assert_eq!(
            adapter.release((&session).into()).unwrap_err().kind,
            ErrorKind::SessionLost
        );
        assert!(!previously_handed.busy.load(Ordering::SeqCst));
        assert!(adapter.entry(&(&session).into()).is_ok());
        entry.cleaned.store(true, Ordering::SeqCst);
        entry.busy.store(true, Ordering::SeqCst);
        let mut completing_actor = Busy(Some(entry.clone()));
        completing_actor.publish_idle();
        adapter.release((&session).into()).unwrap();
        // Terminal publication can wake a releaser before the old actor scope
        // ends. Its still-live guard must never reopen the retired entry.
        drop(completing_actor);
        assert!(adapter.entry(&(&session).into()).is_err());
        assert!(
            previously_handed
                .busy
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
        );
    }
}
