//! Scoped native turn supervision and private completed-turn evidence.
use super::{
    ownership::{ProcessOwnership, ScopeSnapshot, filesystem},
    policy,
    protocol::{Metrics, Pending, RunState, bounded_id, failure, verify_version},
    pty::{PtyTransport, Terminal},
    transport::Transport,
};
use crate::{
    adapter::{
        AdapterFuture, AdapterResult, AgentAdapter, AgentInfo, Capability, ErrorKind, LaunchMode,
        LaunchRequest, SessionRef, SessionStatus, SharedStore,
    },
    domain::{Session, SessionId, SessionState, Usage, now_ms},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::sync::{mpsc, oneshot, watch};

const OUTPUT_LIMIT: usize = 64 * 1024;
const RETAINED: usize = 32;

pub struct ClaudeAdapter {
    agent: String,
    executable: PathBuf,
    store: SharedStore,
    sessions: Mutex<HashMap<SessionId, Entry>>,
    runtime_broker: bool,
    turn_timeout: Duration,
    #[cfg(test)]
    before_input_fence: Option<(Arc<tokio::sync::Barrier>, Arc<tokio::sync::Barrier>)>,
}
struct Entry {
    status: watch::Receiver<SessionStatus>,
    stop: mpsc::Sender<()>,
    replies: mpsc::Sender<Reply>,
    evidence: Arc<Mutex<Evidence>>,
    request: LaunchRequest,
    transition: Arc<AtomicBool>,
    terminal: Option<(Terminal, ScopeSnapshot, Value)>,
    resize: Option<mpsc::Sender<Resize>>,
    input: Option<mpsc::Sender<TerminalInput>>,
}
#[derive(Default)]
struct Evidence {
    completed: bool,
    native: Option<String>,
    attempt: Option<String>,
    metrics: Option<Metrics>,
    pending: Option<Pending>,
    terminal_observed: bool,
}
type OwnedReference = (
    watch::Receiver<SessionStatus>,
    mpsc::Sender<()>,
    mpsc::Sender<Reply>,
    Arc<Mutex<Evidence>>,
);
struct Transition(Arc<AtomicBool>);
impl Drop for Transition {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Decision {
    native_uuid: String,
    request_id: String,
    tool_use_id: String,
    operation_hash: String,
    decision: String,
}
struct Resize {
    rows: u16,
    columns: u16,
    result: oneshot::Sender<AdapterResult<()>>,
}
struct TerminalInput {
    bytes: Vec<u8>,
    result: oneshot::Sender<AdapterResult<()>>,
}
struct Reply {
    decision: Decision,
    result: oneshot::Sender<AdapterResult<()>>,
}
struct Reservation {
    store: SharedStore,
    session: Session,
    version: u64,
    ownership: ProcessOwnership,
    armed: bool,
    input_may_have_been_sent: bool,
}
impl Reservation {
    fn persist_current(&mut self, snapshot: &ScopeSnapshot) -> AdapterResult<()> {
        self.commit_current(self.session.clone(), snapshot)
    }
    fn commit_current(
        &mut self,
        candidate: Session,
        snapshot: &ScopeSnapshot,
    ) -> AdapterResult<()> {
        self.version = self
            .store
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?
            .put_session_if_current(
                &candidate,
                self.version,
                snapshot.versions(),
                &snapshot.lock_versions(),
            )
            .map_err(|_| {
                failure(
                    ErrorKind::StateConflict,
                    "native dispatch authority changed",
                )
            })?;
        self.session = candidate;
        Ok(())
    }
    fn persist(&mut self) -> AdapterResult<()> {
        self.version = self
            .store
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?
            .put_session(&self.session, self.version)
            .map_err(|_| {
                failure(
                    ErrorKind::StateConflict,
                    "native Session publication conflict",
                )
            })?;
        Ok(())
    }
    fn publish(
        &mut self,
        status: &mut SessionStatus,
        sender: &watch::Sender<SessionStatus>,
    ) -> AdapterResult<()> {
        self.persist()?;
        status.session = self.session.clone();
        sender.send_replace(status.clone());
        Ok(())
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if self.armed {
            let uncertain = self.ownership.uncertain();
            self.session.state = if uncertain || self.input_may_have_been_sent {
                SessionState::Lost
            } else {
                SessionState::Failed
            };
            if !uncertain {
                self.session.pid = None;
            }
            let _ = self.persist();
        }
    }
}
impl ClaudeAdapter {
    pub fn new(agent: String, executable: PathBuf, store: SharedStore) -> AdapterResult<Self> {
        if agent.trim().is_empty() || !executable.is_absolute() || executable.to_str().is_none() {
            return Err(failure(
                ErrorKind::InvalidConfiguration,
                "native Claude requires a named agent and absolute executable",
            ));
        }
        Ok(Self {
            agent,
            executable,
            store,
            sessions: Mutex::new(HashMap::new()),
            runtime_broker: false,
            turn_timeout: Duration::from_secs(600),
            #[cfg(test)]
            before_input_fence: None,
        })
    }
    /// Enable exact one-shot replies from the trusted runtime broker. Native
    /// policy can decide before callbacks; this does not grant blanket authority.
    pub fn with_runtime_broker(mut self) -> Self {
        self.runtime_broker = true;
        self
    }
    pub fn with_turn_timeout(mut self, timeout: Duration) -> AdapterResult<Self> {
        if timeout.is_zero() || timeout > Duration::from_secs(3600) {
            return Err(failure(
                ErrorKind::InvalidConfiguration,
                "native turn timeout outside bounds",
            ));
        }
        self.turn_timeout = timeout;
        Ok(self)
    }
    async fn owned_terminal(&self, reference: &SessionRef) -> AdapterResult<Terminal> {
        let (terminal, snapshot, binding, request) = {
            let registry = self.registry()?;
            let entry = registry.get(&reference.id).ok_or_else(|| {
                failure(ErrorKind::SessionLost, "owned terminal transport missing")
            })?;
            if entry.status.borrow().session.scope != reference.scope {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "foreign terminal attachment",
                ));
            }
            if entry.transition.load(Ordering::SeqCst) || entry.status.borrow().terminal() {
                return Err(failure(ErrorKind::SessionLost, "terminal is not live"));
            }
            let (terminal, snapshot, binding) = entry.terminal.clone().ok_or_else(|| {
                failure(
                    ErrorKind::UnsupportedCapability,
                    "native print session has no interactive terminal",
                )
            })?;
            (terminal, snapshot, binding, entry.request.clone())
        };
        self.current(reference)?;
        snapshot.recheck_scope(&self.store, &request, &self.agent)?;
        snapshot.verify_path_binding(&request, &binding).await?;
        snapshot.recheck_scope(&self.store, &request, &self.agent)?;
        if !terminal.live() {
            return Err(failure(ErrorKind::SessionLost, "terminal owner has ended"));
        }
        Ok(terminal)
    }
    /// Bytes go to the existing native UI, including its ordinary trust/permission
    /// dialogs. This is explicit user input, never an automatic grant source.
    pub async fn terminal_input(&self, reference: SessionRef, bytes: Vec<u8>) -> AdapterResult<()> {
        if bytes.is_empty() || bytes.len() > 4096 {
            return Err(failure(
                ErrorKind::InvalidInput,
                "terminal input must contain 1..4096 bytes",
            ));
        }
        self.owned_terminal(&reference).await?;
        let input = self
            .registry()?
            .get(&reference.id)
            .and_then(|entry| entry.input.clone())
            .ok_or_else(|| failure(ErrorKind::SessionLost, "terminal input owner missing"))?;
        let (tx, rx) = oneshot::channel();
        tokio::time::timeout(
            Duration::from_secs(2),
            input.send(TerminalInput { bytes, result: tx }),
        )
        .await
        .map_err(|_| failure(ErrorKind::Timeout, "terminal input queue deadline exceeded"))?
        .map_err(|_| failure(ErrorKind::SessionLost, "terminal input owner ended"))?;
        tokio::time::timeout(Duration::from_secs(5), rx)
            .await
            .map_err(|_| failure(ErrorKind::Timeout, "terminal input reply deadline exceeded"))?
            .map_err(|_| failure(ErrorKind::SessionLost, "terminal input owner ended"))?
    }
    pub async fn terminal_resize(
        &self,
        reference: SessionRef,
        rows: u16,
        columns: u16,
    ) -> AdapterResult<()> {
        self.owned_terminal(&reference).await?;
        let resize = {
            let registry = self.registry()?;
            registry
                .get(&reference.id)
                .and_then(|entry| entry.resize.clone())
                .ok_or_else(|| {
                    failure(
                        ErrorKind::SessionLost,
                        "owned terminal resize channel missing",
                    )
                })?
        };
        let (tx, rx) = oneshot::channel();
        tokio::time::timeout(
            Duration::from_secs(2),
            resize.send(Resize {
                rows,
                columns,
                result: tx,
            }),
        )
        .await
        .map_err(|_| {
            failure(
                ErrorKind::Timeout,
                "terminal resize queue deadline exceeded",
            )
        })?
        .map_err(|_| failure(ErrorKind::SessionLost, "terminal resize owner ended"))?;
        tokio::time::timeout(Duration::from_secs(2), rx)
            .await
            .map_err(|_| failure(ErrorKind::Timeout, "terminal resize deadline exceeded"))?
            .map_err(|_| failure(ErrorKind::SessionLost, "terminal resize owner ended"))?
    }
    #[allow(clippy::too_many_arguments)]
    async fn launch_terminal(
        &self,
        request: LaunchRequest,
        mut reservation: Reservation,
        snapshot: ScopeSnapshot,
        binding: Value,
        native: String,
        args: Vec<String>,
        environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    ) -> AdapterResult<Session> {
        if request.input.payload.len() > 64 * 1024 || request.input.payload.contains('\0') {
            return Err(failure(
                ErrorKind::InvalidInput,
                "interactive prepared context must fit bounded native argv",
            ));
        }
        let mut args = args.into_iter().skip(6).collect::<Vec<_>>();
        args.push("--".into());
        args.push(request.input.payload.clone());
        snapshot
            .verify_binding(&request, &mut reservation.ownership, &binding)
            .await?;
        snapshot.recheck(&self.store, &request, &self.agent)?;
        let mut candidate = reservation.session.clone();
        candidate.state = SessionState::WaitingHuman;
        candidate.recovery["requested_native_uuid"] = json!(native);
        candidate.recovery["dispatch_intent"] = json!({"kind":"terminal_start","attempt":candidate.recovery["attempt"],"requested_native_uuid":native,"input_version":request.input.version,"input_revision":request.input.revision,"input_bytes":request.input.payload.len()});
        reservation.commit_current(candidate, &snapshot)?;
        let transport = PtyTransport::launch(
            &self.executable,
            &request.worktree,
            &args,
            environment,
            reservation.ownership.group(),
        )?;
        reservation.input_may_have_been_sent = true;
        reservation.session.pid = Some(transport.pid());
        reservation.session.state = SessionState::WaitingHuman;
        // UI output is not authoritative structured UUID/turn confirmation.
        reservation.session.native_ref = None;
        reservation.session.recovery["requested_native_uuid"] = native.into();
        reservation.persist()?;
        let session = reservation.session.clone();
        let status = SessionStatus {
            session: session.clone(),
            exit_code: None,
            stdout: vec![],
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
            failure: None,
        };
        let (sender, receiver) = watch::channel(status.clone());
        let (stop_tx, stop) = mpsc::channel(1);
        let (resize_tx, resize) = mpsc::channel(4);
        let (input_tx, input) = mpsc::channel(4);
        let (replies_tx, replies) = mpsc::channel(1);
        drop(replies);
        let evidence = Arc::new(Mutex::new(Evidence::default()));
        {
            let mut registry = self.registry()?;
            if registry.len() >= RETAINED {
                return Err(failure(
                    ErrorKind::Locked,
                    "native retained session limit reached",
                ));
            }
            registry.insert(
                session.id,
                Entry {
                    status: receiver,
                    stop: stop_tx,
                    replies: replies_tx,
                    evidence,
                    request: request.clone(),
                    transition: Arc::new(AtomicBool::new(false)),
                    terminal: Some((
                        transport.terminal.clone(),
                        snapshot.clone(),
                        binding.clone(),
                    )),
                    resize: Some(resize_tx),
                    input: Some(input_tx),
                },
            );
        }
        let timeout = self.turn_timeout;
        tokio::spawn(supervise_terminal(
            transport,
            reservation,
            status,
            sender,
            stop,
            resize,
            input,
            self.agent.clone(),
            request,
            snapshot,
            binding,
            timeout,
        ));
        Ok(session)
    }
    /// Exact bounded original operation for a trusted runtime broker. Values
    /// stay in memory: neither audit nor recovery records persist tool inputs.
    pub fn pending_operation(&self, reference: SessionRef) -> AdapterResult<Value> {
        let status = self.current(&reference)?;
        let (_, _, _, evidence) = self.reference(&reference)?;
        let evidence = evidence.lock().map_err(|_| {
            failure(
                ErrorKind::StateFailure,
                "native permission journal poisoned",
            )
        })?;
        let pending = evidence
            .pending
            .as_ref()
            .ok_or_else(|| failure(ErrorKind::StateConflict, "no native permission pending"))?;
        let native = evidence.native.as_deref().ok_or_else(|| {
            failure(
                ErrorKind::SessionLost,
                "native permission owner unconfirmed",
            )
        })?;
        if status.session.state != SessionState::WaitingApproval
            || status.session.native_ref.as_deref() != Some(native)
        {
            return Err(failure(
                ErrorKind::StateConflict,
                "native permission owner changed",
            ));
        }
        let mut operation = pending.public(native);
        operation["input"] = pending.input.clone();
        Ok(operation)
    }
    fn registry(&self) -> AdapterResult<std::sync::MutexGuard<'_, HashMap<SessionId, Entry>>> {
        self.sessions
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "native registry poisoned"))
    }
    fn reference(&self, reference: &SessionRef) -> AdapterResult<OwnedReference> {
        let registry = self.registry()?;
        let entry = registry.get(&reference.id).ok_or_else(|| {
            failure(
                ErrorKind::SessionLost,
                "no owned native transport; UUID/PID hints cannot reconnect",
            )
        })?;
        if entry.status.borrow().session.scope != reference.scope {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "foreign native Session",
            ));
        }
        Ok((
            entry.status.clone(),
            entry.stop.clone(),
            entry.replies.clone(),
            entry.evidence.clone(),
        ))
    }
    fn current(&self, reference: &SessionRef) -> AdapterResult<SessionStatus> {
        let (receiver, _, _, _) = self.reference(reference)?;
        let status = receiver.borrow().clone();
        self.check_persisted(&status)?;
        Ok(status)
    }
    fn check_persisted(&self, status: &SessionStatus) -> AdapterResult<()> {
        let store = self
            .store
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?;
        let persisted = store
            .session(status.session.id)
            .map_err(|_| failure(ErrorKind::StateFailure, "native Session unavailable"))?
            .ok_or_else(|| failure(ErrorKind::SessionLost, "native Session record missing"))?
            .0;
        if serde_json::to_value(persisted).ok() != serde_json::to_value(&status.session).ok() {
            return Err(failure(
                ErrorKind::StateConflict,
                "native Session changed outside its supervisor",
            ));
        }
        Ok(())
    }
    async fn launch(
        &self,
        request: LaunchRequest,
        resume: Option<Session>,
    ) -> AdapterResult<Session> {
        if request.mode == LaunchMode::Interactive
            && request.role != crate::domain::SessionRole::Consultant
        {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "interactive native mode currently requires Consultant role",
            ));
        }
        let snapshot = ScopeSnapshot::capture(&self.store, &request, &self.agent)?;
        let (previous, expected_version) = if let Some(old) = &resume {
            if old.agent != self.agent
                || old.provider != "claude"
                || old.scope != request.scope
                || old.worktree != request.worktree
                || old.role != request.role
                || old.state != SessionState::Exited
            {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "resume requires exact completed owned native Session",
                ));
            }
            let (_, _, _, evidence) = self.reference(&SessionRef::from(old))?;
            let evidence = evidence
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "native evidence poisoned"))?;
            if !evidence.completed || evidence.native != old.native_ref {
                return Err(failure(
                    ErrorKind::SessionLost,
                    "native UUID lacks completed private proof",
                ));
            }
            let (_, version) = self
                .store
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?
                .session(old.id)
                .map_err(|_| failure(ErrorKind::StateFailure, "native resume record unavailable"))?
                .ok_or_else(|| failure(ErrorKind::SessionLost, "native resume record missing"))?;
            (evidence.metrics.clone(), version)
        } else {
            (None, 0)
        };
        let native = resume
            .as_ref()
            .and_then(|s| s.native_ref.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let attempt = uuid::Uuid::new_v4().to_string();
        let session = Session {
            id: resume.as_ref().map(|s| s.id).unwrap_or_default(),
            scope: request.scope.clone(),
            agent: self.agent.clone(),
            provider: "claude".into(),
            role: request.role,
            native_ref: None,
            pid: None,
            worktree: request.worktree.clone(),
            state: SessionState::Starting,
            model: request.model.clone(),
            effort: request.effort.clone(),
            recovery: json!({"attempt":attempt,"input_revision":request.input.revision,"input_version":request.input.version,"source_versions":request.input.source_versions,"input_bytes":request.input.payload.len(),"reconnect_supported":false}),
            started_at: now_ms(),
        };
        let mut reservation = Reservation {
            store: self.store.clone(),
            session,
            version: expected_version,
            ownership: ProcessOwnership::default(),
            armed: false,
            input_may_have_been_sent: false,
        };
        reservation.persist_current(&snapshot)?;
        reservation.armed = true;
        let binding = snapshot
            .verify_git(&request, &mut reservation.ownership)
            .await?;
        snapshot.recheck(&self.store, &request, &self.agent)?;
        let executable = self.executable.clone();
        filesystem(move || {
            use std::os::unix::fs::PermissionsExt;
            let metadata = executable.metadata().map_err(|_| {
                failure(
                    ErrorKind::ExecutableMissing,
                    "native Claude executable unavailable",
                )
            })?;
            if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
                return Err(failure(
                    ErrorKind::ExecutableMissing,
                    "native Claude executable is not executable",
                ));
            }
            Ok(())
        })
        .await?;
        let environment = policy::environment(
            std::env::vars_os(),
            &snapshot.projects,
            &snapshot.project,
            &request.environment,
        )?;
        let version = crate::adapter::bounded_git(
            &self.executable,
            &request.worktree,
            &["--version".into()],
            environment.clone(),
            tokio::time::Instant::now() + Duration::from_secs(5),
            reservation.ownership.group(),
        )
        .await?;
        verify_version(&version)?;
        let args = policy::arguments(&request, &native, resume.is_some())?;
        if request.mode == LaunchMode::Interactive {
            return self
                .launch_terminal(
                    request,
                    reservation,
                    snapshot,
                    binding,
                    native,
                    args,
                    environment,
                )
                .await;
        }
        let mut transport = Transport::launch(
            &self.executable,
            &request.worktree,
            &args,
            environment,
            reservation.ownership.group(),
        )?;
        reservation.session.pid = Some(transport.pid());
        reservation.persist()?;
        let initialize=async {
            transport.request("initialize",&format!("init-{attempt}")).await?;
            if policy::decision(request.role) {
                let inventory=transport.request("mcp_status",&format!("mcp-{attempt}")).await?;
                if !inventory["mcpServers"].as_array().is_some_and(Vec::is_empty) {
                    return Err(failure(ErrorKind::UnsupportedCapability,"native decision inherited MCP inventory is not empty"));
                }
            }
            snapshot.recheck(&self.store,&request,&self.agent)?;
            snapshot.verify_binding(&request,&mut reservation.ownership,&binding).await?;
            snapshot.recheck(&self.store,&request,&self.agent)?;
            let input=Transport::encode(&json!({"type":"user","message":{"role":"user","content":request.input.payload},"origin":{"kind":"human"},"parent_tool_use_id":null,"session_id":native}))?;
            #[cfg(test)]
            if let Some((ready,release))=&self.before_input_fence {ready.wait().await;release.wait().await;}
            let mut candidate=reservation.session.clone();
            candidate.native_ref=Some(native.clone());
            candidate.state=SessionState::Running;
            candidate.recovery["dispatch_intent"]=json!({"kind":"input","attempt":attempt,"native_uuid":native,"input_version":request.input.version,"input_revision":request.input.revision,"input_bytes":request.input.payload.len()});
            reservation.commit_current(candidate,&snapshot)?;
            reservation.input_may_have_been_sent=true;
            transport.write_encoded(&input).await?;
            Ok::<_,crate::adapter::AdapterError>(())
        }.await;
        if let Err(primary) = initialize {
            let cleanup = transport.cleanup().await;
            return if cleanup.is_err() {
                Err(failure(
                    ErrorKind::SessionLost,
                    "native initialization and owned cleanup failed",
                ))
            } else {
                Err(primary)
            };
        }
        let session = reservation.session.clone();
        let status = SessionStatus {
            session: session.clone(),
            exit_code: None,
            stdout: vec![],
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
            failure: None,
        };
        let (sender, receiver) = watch::channel(status.clone());
        let (stop_tx, stop) = mpsc::channel(1);
        let (replies_tx, replies) = mpsc::channel(8);
        let evidence = Arc::new(Mutex::new(Evidence {
            native: Some(native.clone()),
            attempt: Some(attempt),
            ..Evidence::default()
        }));
        {
            let mut registry = self.registry()?;
            if registry.len() >= RETAINED && resume.is_none() {
                if let Some(id) = registry
                    .iter()
                    .filter(|(_, e)| {
                        e.status.borrow().terminal() && !e.transition.load(Ordering::SeqCst)
                    })
                    .min_by_key(|(_, e)| e.status.borrow().session.started_at)
                    .map(|(id, _)| *id)
                {
                    registry.remove(&id);
                }
                if registry.len() >= RETAINED {
                    return Err(failure(
                        ErrorKind::Locked,
                        "native retained session limit reached",
                    ));
                }
            }
            registry.insert(
                session.id,
                Entry {
                    status: receiver,
                    stop: stop_tx,
                    replies: replies_tx,
                    evidence: evidence.clone(),
                    request: request.clone(),
                    transition: Arc::new(AtomicBool::new(false)),
                    terminal: None,
                    resize: None,
                    input: None,
                },
            );
        }
        let timeout = self.turn_timeout;
        let broker = self.runtime_broker;
        let agent = self.agent.clone();
        tokio::spawn(async move {
            supervise(
                transport,
                reservation,
                status,
                sender,
                evidence,
                request,
                snapshot,
                binding,
                native,
                previous,
                resume.is_some(),
                stop,
                replies,
                broker,
                agent,
                timeout,
            )
            .await;
        });
        Ok(session)
    }
}
impl AgentAdapter for ClaudeAdapter {
    fn capabilities(&self) -> BTreeSet<Capability> {
        [
            Capability::Execute,
            Capability::Consult,
            Capability::Review,
            Capability::NonInteractive,
            Capability::Resume,
            Capability::PermissionInterception,
            Capability::StructuredOutput,
            Capability::UsageTelemetry,
            Capability::PromptCacheTelemetry,
        ]
        .into_iter()
        .collect()
    }
    fn probe(&self) -> AdapterResult<AgentInfo> {
        Ok(AgentInfo {
            agent: self.agent.clone(),
            provider: "claude".into(),
            adapter_version: env!("CARGO_PKG_VERSION").into(),
            executable: self.executable.clone(),
            authenticated: None,
            model_configuration: true,
            effort_configuration: true,
            capabilities: self.capabilities(),
        })
    }
    fn start(&self, request: LaunchRequest) -> AdapterFuture<'_, Session> {
        Box::pin(self.launch(request, None))
    }
    fn transport_succeeded(&self, status: &SessionStatus) -> bool {
        let Ok(registry) = self.registry() else {
            return false;
        };
        let Some(entry) = registry.get(&status.session.id) else {
            return false;
        };
        if entry.transition.load(Ordering::SeqCst)
            || status.session.state != SessionState::Exited
            || status.failure.is_some()
            || serde_json::to_value(&entry.status.borrow().session).ok()
                != serde_json::to_value(&status.session).ok()
            || self.check_persisted(status).is_err()
        {
            return false;
        }
        let Ok(evidence) = entry.evidence.lock() else {
            return false;
        };
        evidence.completed
            && evidence.native == status.session.native_ref
            && status.session.recovery["attempt"].as_str() == evidence.attempt.as_deref()
    }
    fn status(&self, reference: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        Box::pin(async move { self.current(&reference) })
    }
    fn stop(&self, reference: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        Box::pin(async move {
            let (mut receiver, stop, _, _) = self.reference(&reference)?;
            if !receiver.borrow().terminal() {
                let _ = stop.send(()).await;
            }
            tokio::time::timeout(Duration::from_secs(10), async {
                while !receiver.borrow().terminal() {
                    receiver.changed().await.map_err(|_| {
                        failure(ErrorKind::SessionLost, "native supervisor disappeared")
                    })?;
                }
                Ok::<_, crate::adapter::AdapterError>(())
            })
            .await
            .map_err(|_| failure(ErrorKind::Timeout, "native stop deadline exceeded"))??;
            self.current(&reference)
        })
    }
    fn attach(&self, reference: SessionRef) -> AdapterFuture<'_, ()> {
        Box::pin(async move {
            let _ = self.owned_terminal(&reference).await?;
            Ok(())
        })
    }
    fn resume(&self, reference: SessionRef) -> AdapterFuture<'_, Session> {
        Box::pin(async move {
            let (claim, request) = {
                let registry = self.registry()?;
                let entry = registry.get(&reference.id).ok_or_else(|| {
                    failure(ErrorKind::SessionLost, "native resume owner missing")
                })?;
                if entry.status.borrow().session.scope != reference.scope {
                    return Err(failure(
                        ErrorKind::OwnershipMismatch,
                        "foreign native resume",
                    ));
                }
                entry
                    .transition
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                    .map_err(|_| {
                        failure(
                            ErrorKind::StateConflict,
                            "native transition already claimed",
                        )
                    })?;
                (Transition(entry.transition.clone()), entry.request.clone())
            };
            let previous = self.current(&reference)?;
            let result = self.launch(request, Some(previous.session)).await;
            drop(claim);
            result
        })
    }
    fn release(&self, reference: SessionRef) -> AdapterResult<()> {
        let mut registry = self.registry()?;
        let entry = registry
            .get(&reference.id)
            .ok_or_else(|| failure(ErrorKind::SessionLost, "native release owner missing"))?;
        if entry.status.borrow().session.scope != reference.scope {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "foreign native release",
            ));
        }
        if !entry.status.borrow().terminal() || entry.transition.load(Ordering::SeqCst) {
            return Err(failure(
                ErrorKind::StateConflict,
                "cannot release active native transport",
            ));
        }
        registry.remove(&reference.id);
        Ok(())
    }
    fn subscribe(&self, reference: SessionRef) -> AdapterResult<watch::Receiver<SessionStatus>> {
        Ok(self.reference(&reference)?.0)
    }
    fn usage(
        &self,
        reference: SessionRef,
        phase: String,
        review_round: Option<u32>,
    ) -> AdapterFuture<'_, Usage> {
        Box::pin(async move {
            if phase.is_empty() || phase.len() > 128 {
                return Err(failure(ErrorKind::InvalidInput, "invalid usage phase"));
            }
            let registry = self.registry()?;
            let entry = registry
                .get(&reference.id)
                .ok_or_else(|| failure(ErrorKind::SessionLost, "native usage owner missing"))?;
            let status = entry.status.borrow().clone();
            if status.session.scope != reference.scope {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "foreign native usage",
                ));
            }
            if entry.transition.load(Ordering::SeqCst) {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "usage overlaps native transition",
                ));
            }
            self.check_persisted(&status)?;
            let evidence = entry
                .evidence
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "native usage journal poisoned"))?;
            let metrics = evidence.metrics.as_ref();
            Ok(Usage {
                scope: reference.scope,
                session_id: reference.id,
                agent: self.agent.clone(),
                phase,
                review_round,
                input_tokens: metrics.and_then(|m| m.input),
                cached_input_tokens: metrics.and_then(|m| m.cache_read),
                output_tokens: metrics.and_then(|m| m.output),
                estimated_cost: metrics.and_then(|m| m.cost),
                context_pack_version: Some(entry.request.input.version),
                context_pack_size: Some(entry.request.input.payload.len() as u64),
                repo_map_size: None,
                cache_metadata: json!({"provider":"claude","native_uuid":evidence.native,"cache_creation_input_tokens":metrics.and_then(|m|m.cache_write),"duration_api_ms":metrics.and_then(|m|m.api_ms),"token_counters":"per_invocation","cost_duration_gauges":"owned_resume_delta"}),
                missing_reason: if metrics
                    .is_none_or(|m| m.input.is_none() || m.output.is_none() || m.cost.is_none())
                {
                    Some("native metrics unavailable, incomplete or resumed gauge baseline/reset unknown".into())
                } else {
                    None
                },
            })
        })
    }
    fn submit_approval(&self, reference: SessionRef, value: Value) -> AdapterFuture<'_, ()> {
        Box::pin(async move {
            let decision: Decision = serde_json::from_value(value).map_err(|_| {
                failure(
                    ErrorKind::InvalidInput,
                    "invalid exact native approval reply",
                )
            })?;
            if uuid::Uuid::parse_str(&decision.native_uuid).is_err()
                || decision.request_id.is_empty()
                || decision.request_id.len() > 256
                || decision.tool_use_id.is_empty()
                || decision.tool_use_id.len() > 256
                || decision.operation_hash.len() != 64
                || !decision
                    .operation_hash
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit())
                || !matches!(decision.decision.as_str(), "ALLOW" | "DENY" | "ESCALATE")
            {
                return Err(failure(
                    ErrorKind::InvalidInput,
                    "native approval identities/decision outside bounds",
                ));
            }
            let (_, _, replies, _) = self.reference(&reference)?;
            let (tx, rx) = oneshot::channel();
            replies
                .send(Reply {
                    decision,
                    result: tx,
                })
                .await
                .map_err(|_| {
                    failure(
                        ErrorKind::SessionLost,
                        "native approval transport unavailable",
                    )
                })?;
            tokio::time::timeout(Duration::from_secs(15), rx)
                .await
                .map_err(|_| failure(ErrorKind::Timeout, "native approval deadline exceeded"))?
                .map_err(|_| {
                    failure(
                        ErrorKind::SessionLost,
                        "native approval supervisor unavailable",
                    )
                })?
        })
    }
}
fn tail(target: &mut Vec<u8>, truncated: &mut bool, bytes: &[u8]) {
    target.extend_from_slice(bytes);
    if target.len() > OUTPUT_LIMIT {
        target.drain(..target.len() - OUTPUT_LIMIT);
        *truncated = true;
    }
}
#[allow(clippy::too_many_arguments)]
async fn supervise(
    mut transport: Transport,
    mut reservation: Reservation,
    mut status: SessionStatus,
    sender: watch::Sender<SessionStatus>,
    evidence: Arc<Mutex<Evidence>>,
    request: LaunchRequest,
    snapshot: ScopeSnapshot,
    binding: Value,
    native: String,
    previous: Option<Metrics>,
    resumed: bool,
    mut stop: mpsc::Receiver<()>,
    mut replies: mpsc::Receiver<Reply>,
    broker: bool,
    agent: String,
    timeout: Duration,
) {
    let result = run(
        &mut transport,
        &mut reservation,
        &mut status,
        &sender,
        &evidence,
        &request,
        &snapshot,
        &binding,
        &native,
        previous.as_ref(),
        resumed,
        &mut stop,
        &mut replies,
        broker,
        &agent,
        timeout,
    )
    .await;
    let cleanup = if matches!(result, Ok(true)) {
        transport.finish().await
    } else {
        transport.cleanup().await
    };
    let uncertain = cleanup.is_err() || reservation.ownership.uncertain();
    let observed = evidence
        .lock()
        .map(|e| e.terminal_observed)
        .unwrap_or(false);
    let unknown = reservation.input_may_have_been_sent && !observed;
    reservation.session.state = if uncertain || (unknown && !matches!(result, Ok(true))) {
        SessionState::Lost
    } else {
        match &result {
            Ok(true) => SessionState::Exited,
            Ok(false) => SessionState::Stopped,
            Err(_) => SessionState::Failed,
        }
    };
    if !uncertain {
        reservation.session.pid = None;
    }
    status.exit_code = None;
    status.failure = match (result.as_ref().err(), cleanup.as_ref().err()) {
        (Some(p), Some(c)) => Some(format!("{p}; owned cleanup unverified: {c}")),
        (Some(e), None) | (None, Some(e)) => Some(e.to_string()),
        _ => None,
    };
    if uncertain {
        status
            .failure
            .get_or_insert_with(|| "owned native process cleanup unverified".into());
    }
    reservation
        .session
        .recovery
        .as_object_mut()
        .expect("runtime recovery object")
        .remove("pending_permission");
    let persisted = reservation.persist().is_ok();
    status.session = reservation.session.clone();
    if !persisted {
        status.failure = Some("native terminal CAS failed; completion is unverified".into());
        status.session.state = SessionState::Lost;
        sender.send_replace(status.clone());
    }
    if let Ok(mut evidence) = evidence.lock() {
        evidence.pending = None;
        evidence.completed = matches!(result, Ok(true)) && !uncertain && persisted;
    }
    sender.send_replace(status);
    reservation.armed = !persisted;
}
#[allow(clippy::too_many_arguments)]
async fn run(
    transport: &mut Transport,
    reservation: &mut Reservation,
    status: &mut SessionStatus,
    sender: &watch::Sender<SessionStatus>,
    evidence: &Arc<Mutex<Evidence>>,
    request: &LaunchRequest,
    snapshot: &ScopeSnapshot,
    binding: &Value,
    native: &str,
    previous: Option<&Metrics>,
    resumed: bool,
    stop: &mut mpsc::Receiver<()>,
    replies: &mut mpsc::Receiver<Reply>,
    broker: bool,
    agent: &str,
    timeout: Duration,
) -> AdapterResult<bool> {
    let deadline = tokio::time::Instant::now() + timeout;
    let mut state = RunState::default();
    let mut request_ids = BTreeSet::new();
    let mut diag_open = true;
    loop {
        tokio::select! {
            biased;
            _=stop.recv()=>return Ok(false),
            _=tokio::time::sleep_until(deadline)=>return Err(failure(ErrorKind::Timeout,"native turn deadline exceeded")),
            reply=replies.recv()=>if let Some(reply)=reply {
                let outcome=respond(transport,reservation,status,sender,evidence,request,snapshot,binding,native,broker,agent,&reply.decision).await;
                let fatal=outcome.as_ref().err().is_some_and(|e|!matches!(e.kind,ErrorKind::InvalidInput|ErrorKind::OwnershipMismatch|ErrorKind::StateConflict));
                let _=reply.result.send(outcome);
                if fatal {return Err(failure(ErrorKind::ProcessFailure,"native permission response failed"));}
            },
            bytes=transport.diagnostics.recv(),if diag_open=>match bytes {Some(bytes)=>{tail(&mut status.stderr,&mut status.stderr_truncated,&bytes);sender.send_replace(status.clone());},None=>diag_open=false},
            frame=transport.frames.recv()=>{
                let message=frame.ok_or_else(||failure(ErrorKind::ProcessFailure,"native output closed before verified terminal"))??;
                if message["type"]=="control_request" {
                    if policy::decision(request.role) {return Err(failure(ErrorKind::UnsupportedCapability,"native decision permission request"));}
                    if !state.initialized || message.get("session_id").is_some_and(|id|id!=native) {return Err(failure(ErrorKind::OwnershipMismatch,"native permission before confirmed init or from foreign UUID"));}
                    let pending=Pending::parse(&message,native)?;
                    if !request_ids.insert(pending.request_id.clone())||request_ids.len()>128 {return Err(failure(ErrorKind::ParseFailure,"native permission request reused or limit exceeded"));}
                    {
                        let mut evidence=evidence.lock().map_err(|_|failure(ErrorKind::StateFailure,"native pending journal poisoned"))?;
                        if evidence.pending.is_some(){return Err(failure(ErrorKind::ParseFailure,"concurrent native permission requests unsupported"));}
                        evidence.pending=Some(pending.clone());
                    }
                    reservation.session.state=SessionState::WaitingApproval;
                    reservation.session.recovery["pending_permission"]=pending.public(native);
                    reservation.publish(status,sender)?;
                    if !broker {
                        transport.write(&pending.reply(false)).await?;
                        evidence.lock().map_err(|_|failure(ErrorKind::StateFailure,"native pending journal poisoned"))?.pending=None;
                        reservation.session.state=SessionState::Running;reservation.session.recovery.as_object_mut().expect("recovery object").remove("pending_permission");reservation.publish(status,sender)?;
                    }
                    continue;
                }
                if message["type"]=="control_cancel_request" {
                    let id=bounded_id(&message["request_id"])?;
                    let mut journal=evidence.lock().map_err(|_|failure(ErrorKind::StateFailure,"native pending journal poisoned"))?;
                    if journal.pending.as_ref().is_none_or(|p|p.request_id!=id) {
                        if request_ids.contains(&id) {continue;}
                        return Err(failure(ErrorKind::ParseFailure,"foreign native permission cancellation"));
                    }
                    journal.pending=None;drop(journal);
                    reservation.session.state=SessionState::Running;reservation.session.recovery.as_object_mut().expect("recovery object").remove("pending_permission");reservation.publish(status,sender)?;continue;
                }
                let observed=state.observe(&message,native,&request.worktree,policy::decision(request.role));
                // Capture only a correlated accepted native terminal. A failed
                // native turn can still carry real usage; duplicates/foreign
                // records never replace the private journal.
                let accepted_terminal=matches!(observed,Ok(true)) || observed.as_ref().err().is_some_and(|e|matches!(e.kind,ErrorKind::ProcessFailure|ErrorKind::AuthenticationUnavailable));
                if message["type"]=="result" && message["session_id"]==native && state.initialized && accepted_terminal {
                    let metrics=Metrics::parse(&message,previous,resumed)?;
                    let mut journal=evidence.lock().map_err(|_|failure(ErrorKind::StateFailure,"native metrics journal poisoned"))?;
                    journal.metrics=Some(metrics);
                    if observed.is_err() {journal.terminal_observed=true;}
                }
                if !observed? {continue;}
                if message["type"]=="assistant" && let Some(content)=message["message"]["content"].as_array() {
                    for part in content {if part["type"]=="text" && let Some(text)=part["text"].as_str(){tail(&mut status.stdout,&mut status.stdout_truncated,text.as_bytes());}}
                }
                if let Some(text)=message.get("result").and_then(Value::as_str) {status.stdout.clear();tail(&mut status.stdout,&mut status.stdout_truncated,text.as_bytes());}
                sender.send_replace(status.clone());
                if state.complete() {
                    if evidence.lock().map_err(|_|failure(ErrorKind::StateFailure,"native journal poisoned"))?.pending.is_some(){return Err(failure(ErrorKind::ParseFailure,"native terminal with pending permission"));}
                    evidence.lock().map_err(|_|failure(ErrorKind::StateFailure,"native journal poisoned"))?.terminal_observed=true;
                    snapshot.recheck_scope(&reservation.store,request,agent)?;
                    return Ok(true);
                }
            }
        }
    }
}
#[allow(clippy::too_many_arguments)]
async fn respond(
    transport: &mut Transport,
    reservation: &mut Reservation,
    status: &mut SessionStatus,
    sender: &watch::Sender<SessionStatus>,
    evidence: &Arc<Mutex<Evidence>>,
    request: &LaunchRequest,
    snapshot: &ScopeSnapshot,
    binding: &Value,
    native: &str,
    broker: bool,
    agent: &str,
    decision: &Decision,
) -> AdapterResult<()> {
    let pending = evidence
        .lock()
        .map_err(|_| failure(ErrorKind::StateFailure, "native pending journal poisoned"))?
        .pending
        .clone()
        .ok_or_else(|| {
            failure(
                ErrorKind::StateConflict,
                "native permission is no longer pending",
            )
        })?;
    if decision.native_uuid != native
        || decision.request_id != pending.request_id
        || decision.tool_use_id != pending.tool_use_id
        || decision.operation_hash != pending.hash
    {
        return Err(failure(
            ErrorKind::OwnershipMismatch,
            "foreign/stale native permission reply",
        ));
    }
    if decision.decision == "ESCALATE" {
        return Ok(());
    }
    let allow = match decision.decision.as_str() {
        "ALLOW" if broker => true,
        "DENY" => false,
        _ => {
            return Err(failure(
                ErrorKind::InvalidInput,
                "unsupported/ungranted native permission decision",
            ));
        }
    };
    let response = Transport::encode(&pending.reply(allow))?;
    if allow {
        snapshot.recheck_scope(&reservation.store, request, agent)?;
        snapshot
            .verify_binding(request, &mut reservation.ownership, binding)
            .await?;
        snapshot.recheck_scope(&reservation.store, request, agent)?;
    }
    {
        let mut candidate = reservation.session.clone();
        candidate.state = SessionState::Running;
        candidate
            .recovery
            .as_object_mut()
            .expect("recovery object")
            .remove("pending_permission");
        candidate.recovery["dispatch_intent"] = json!({"kind":"permission","operation":pending.public(native),"decision":decision.decision});
        reservation.commit_current(candidate, snapshot)?;
    }
    // Remove one-shot authority before the write, including a cancelled caller.
    evidence
        .lock()
        .map_err(|_| failure(ErrorKind::StateFailure, "native pending journal poisoned"))?
        .pending = None;
    transport.write_encoded(&response).await?;
    status.session = reservation.session.clone();
    sender.send_replace(status.clone());
    Ok(())
}
#[allow(clippy::too_many_arguments)]
async fn supervise_terminal(
    mut transport: PtyTransport,
    mut reservation: Reservation,
    mut status: SessionStatus,
    sender: watch::Sender<SessionStatus>,
    mut stop: mpsc::Receiver<()>,
    mut resize: mpsc::Receiver<Resize>,
    mut input: mpsc::Receiver<TerminalInput>,
    agent: String,
    request: LaunchRequest,
    snapshot: ScopeSnapshot,
    binding: Value,
    timeout: Duration,
) {
    let deadline = tokio::time::Instant::now() + timeout;
    let failure = loop {
        tokio::select! {
            biased;
            _=stop.recv()=>break None,
            command=resize.recv()=>if let Some(command)=command {let _=command.result.send(transport.resize_owned(command.rows,command.columns));},
            command=input.recv()=>if let Some(command)=command {
                let result=async {
                    if command.result.is_closed(){return Err(failure(ErrorKind::StateConflict,"terminal input caller cancelled before dispatch"));}
                    snapshot.recheck_scope(&reservation.store,&request,&agent)?;
                    snapshot.verify_path_binding(&request,&binding).await?;
                    if command.result.is_closed(){return Err(failure(ErrorKind::StateConflict,"terminal input caller cancelled before dispatch"));}
                    let mut candidate=reservation.session.clone();
                    candidate.recovery["terminal_input_version"]=json!(candidate.recovery["terminal_input_version"].as_u64().unwrap_or(0).checked_add(1).ok_or_else(||failure(ErrorKind::StateFailure,"terminal input version exhausted"))?);
                    candidate.recovery["dispatch_intent"]=json!({"kind":"terminal_input","input_version":candidate.recovery["terminal_input_version"],"input_bytes":command.bytes.len()});
                    reservation.commit_current(candidate,&snapshot)?;
                    status.session=reservation.session.clone();sender.send_replace(status.clone());
                    transport.terminal.write(&command.bytes).await
                }.await;
                let fatal=result.as_ref().err().is_some_and(|e|matches!(e.kind,ErrorKind::Timeout|ErrorKind::ProcessFailure|ErrorKind::SessionLost));
                let _=command.result.send(result);
                if fatal {break Some("terminal input dispatch unverified".into());}
            },
            _=tokio::time::sleep_until(deadline)=>break Some("native terminal deadline exceeded".to_owned()),
            output=transport.output.recv()=>match output {
                Some(Ok(bytes))=>{tail(&mut status.stdout,&mut status.stdout_truncated,&bytes);sender.send_replace(status.clone());},
                Some(Err(error))=>break Some(error.to_string()),
                None=>break None,
            }
        }
    };
    let cleanup = transport.cleanup().await;
    let uncertain = cleanup.is_err() || reservation.ownership.uncertain();
    reservation.session.state = if uncertain {
        SessionState::Lost
    } else if failure.is_some() {
        SessionState::Failed
    } else {
        SessionState::Stopped
    };
    if !uncertain {
        reservation.session.pid = None;
    }
    status.failure = failure.or_else(|| cleanup.err().map(|e| e.to_string()));
    status.exit_code = None;
    let persisted = reservation.publish(&mut status, &sender).is_ok();
    if !persisted {
        status.session.state = SessionState::Lost;
        status.failure = Some("native terminal CAS failed".into());
        sender.send_replace(status);
    }
    reservation.armed = !persisted;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claude::ownership::tests::Fixture;
    use std::os::unix::fs::PermissionsExt;
    fn executable(temp: &tempfile::TempDir, behavior: &str) -> PathBuf {
        let path = temp.path().join("native-fixture");
        let script = format!(
            r#"#!/usr/bin/env python3
import json,os,sys
if '--version' in sys.argv:
 print('2.1.283 (Claude Code)');sys.exit(0)
native=next(a.split('=',1)[1] for a in sys.argv if a.startswith(('--session-id=','--resume=')))
behavior={behavior:?}
if '-p' not in sys.argv:
 print('REAL_TTY='+str(sys.stdin.isatty() and sys.stdout.isatty() and sys.stderr.isatty()),flush=True)
 for line in sys.stdin:
  if line.strip()=='QUESTION':print('NATIVE_TERMINAL_ANSWER',flush=True)
 sys.exit(0)
def emit(v):print(json.dumps(v),flush=True)
for line in sys.stdin:
 m=json.loads(line)
 if m['type']=='control_request':
  answer={{'mcpServers':[]}} if m['request']['subtype']=='mcp_status' else {{}}
  if behavior=='mcp' and m['request']['subtype']=='mcp_status':answer={{'mcpServers':[{{'name':'unexpected'}}]}}
  emit({{'type':'control_response','response':{{'subtype':'success','request_id':m['request_id'],'response':answer}}}})
 elif m['type']=='user':
  if behavior=='fence':open(os.path.join(os.path.dirname(__file__),'native-user-dispatched'),'w').write('dispatched')
  if behavior=='preinit':
   emit({{'type':'control_request','request_id':'early','request':{{'subtype':'can_use_tool','tool_name':'Read','tool_use_id':'early-op','input':{{'file_path':'proof.txt'}}}}}});continue
  emit({{'type':'system','subtype':'init','session_id':native,'cwd':os.getcwd(),'tools':[],'mcp_servers':[]}})
  if behavior=='pending':
   emit({{'type':'control_request','request_id':'permission-1','request':{{'subtype':'can_use_tool','tool_name':'Bash','tool_use_id':'operation-1','input':{{'command':'pwd'}}}}}})
   continue
  if behavior=='hang':continue
  if behavior=='background':emit({{'type':'system','subtype':'task_started','task_type':'local_agent','task_id':'still-running'}})
  emit({{'type':'result','uuid':'result-1','subtype':'success' if behavior!='native-error' else 'error_during_execution','is_error':behavior=='native-error','session_id':native,'result':'{{"fixture":true}}','usage':{{'input_tokens':2,'output_tokens':13,'cache_creation_input_tokens':5,'cache_read_input_tokens':7}},'total_cost_usd':0.2 if '--resume='+native in sys.argv else 0.1,'duration_api_ms':200 if '--resume='+native in sys.argv else 100}})
 elif m['type']=='control_response':
  assert m['response']['request_id']=='permission-1'
  assert 'updatedPermissions' not in m['response']['response']
  if m['response']['response']['behavior']=='allow':assert m['response']['response']['updatedInput']=={{'command':'pwd'}}
  emit({{'type':'result','subtype':'success','is_error':False,'session_id':native,'result':'done'}})
"#
        );
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        path
    }
    async fn terminal(adapter: &ClaudeAdapter, reference: SessionRef) -> SessionStatus {
        let mut receiver = adapter.subscribe(reference.clone()).unwrap();
        tokio::time::timeout(Duration::from_secs(20), async {
            while !receiver.borrow().terminal() {
                receiver.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        adapter.status(reference).await.unwrap()
    }
    #[tokio::test]
    async fn owned_terminal_resume_telemetry_and_forged_completion() {
        let fixture = Fixture::new(false);
        let temp = tempfile::tempdir().unwrap();
        let adapter = ClaudeAdapter::new(
            "claude".into(),
            executable(&temp, "success"),
            fixture.store.clone(),
        )
        .unwrap();
        let session = adapter.start(fixture.request.clone()).await.unwrap();
        let status = terminal(&adapter, (&session).into()).await;
        assert!(adapter.transport_succeeded(&status));
        assert_eq!(status.exit_code, None);
        let usage = adapter
            .usage((&session).into(), "consult".into(), None)
            .await
            .unwrap();
        assert_eq!(
            (
                usage.input_tokens,
                usage.output_tokens,
                usage.cached_input_tokens,
                usage.estimated_cost
            ),
            (Some(2), Some(13), Some(7), Some(0.1))
        );
        let mut forged = status.clone();
        forged.session.recovery["attempt"] = "foreign".into();
        assert!(!adapter.transport_succeeded(&forged));
        let resumed = adapter.resume((&session).into()).await.unwrap();
        let status = terminal(&adapter, (&resumed).into()).await;
        assert!(adapter.transport_succeeded(&status));
        assert_eq!(resumed.native_ref, session.native_ref);
        let usage = adapter
            .usage((&session).into(), "resume".into(), None)
            .await
            .unwrap();
        assert_eq!(usage.input_tokens, Some(2));
        assert_eq!(usage.estimated_cost, Some(0.1));
        adapter.release((&session).into()).unwrap();
        assert!(!adapter.transport_succeeded(&status));
        assert_eq!(
            adapter.resume((&session).into()).await.unwrap_err().kind,
            ErrorKind::SessionLost
        );
    }
    #[tokio::test]
    async fn decision_inventory_native_error_timeout_and_restart_do_not_succeed() {
        for behavior in ["mcp", "native-error", "hang"] {
            let fixture = Fixture::new(false);
            let temp = tempfile::tempdir().unwrap();
            let adapter = ClaudeAdapter::new(
                "claude".into(),
                executable(&temp, behavior),
                fixture.store.clone(),
            )
            .unwrap()
            .with_turn_timeout(Duration::from_millis(250))
            .unwrap();
            let result = adapter.start(fixture.request.clone()).await;
            if behavior == "mcp" {
                assert_eq!(result.unwrap_err().kind, ErrorKind::UnsupportedCapability);
                continue;
            }
            let session = result.unwrap();
            let status = terminal(&adapter, (&session).into()).await;
            assert_eq!(
                status.session.state,
                if behavior == "hang" {
                    SessionState::Lost
                } else {
                    SessionState::Failed
                }
            );
            assert!(!adapter.transport_succeeded(&status));
            assert!(status.failure.is_some());
            let other = ClaudeAdapter::new(
                "claude".into(),
                adapter.executable.clone(),
                fixture.store.clone(),
            )
            .unwrap();
            assert_eq!(
                other.status((&session).into()).await.unwrap_err().kind,
                ErrorKind::SessionLost
            );
        }
    }
    #[tokio::test]
    async fn concurrent_owner_change_after_preflight_never_dispatches_or_consumes_input() {
        let fixture = Fixture::new(true);
        let temp = tempfile::tempdir().unwrap();
        let ready = Arc::new(tokio::sync::Barrier::new(2));
        let release = Arc::new(tokio::sync::Barrier::new(2));
        let mut adapter = ClaudeAdapter::new(
            "claude".into(),
            executable(&temp, "fence"),
            fixture.store.clone(),
        )
        .unwrap();
        adapter.before_input_fence = Some((ready.clone(), release.clone()));
        let adapter = Arc::new(adapter);
        let request = fixture.request.clone();
        let owner = adapter.clone();
        let launch = tokio::spawn(async move { owner.start(request).await });
        tokio::time::timeout(Duration::from_secs(10), ready.wait())
            .await
            .unwrap();
        let mut other = crate::state::Store::open(
            &fixture
                .request
                .project
                .root
                .parent()
                .unwrap()
                .join("state.sqlite3"),
        )
        .unwrap();
        let mut task = other
            .task(fixture.request.scope.task_id.unwrap())
            .unwrap()
            .unwrap();
        task.title = "concurrent replacement after final preflight".into();
        other.put_task(&mut task).unwrap();
        release.wait().await;
        assert_eq!(
            launch.await.unwrap().unwrap_err().kind,
            ErrorKind::StateConflict
        );
        assert!(!temp.path().join("native-user-dispatched").exists());
        let store = fixture.store.lock().unwrap();
        let sessions = store
            .records(&fixture.request.scope, crate::domain::RecordKind::Session)
            .unwrap();
        assert_eq!(sessions.len(), 1);
        let session: Session = serde_json::from_value(sessions[0].data.clone()).unwrap();
        assert_eq!(session.state, SessionState::Failed);
        assert!(session.pid.is_none());
        assert!(session.recovery.get("dispatch_intent").is_none());
    }
    #[tokio::test]
    async fn stopping_unfinished_native_input_or_background_aggregate_keeps_lost() {
        for behavior in ["hang", "background"] {
            let fixture = Fixture::new(true);
            let temp = tempfile::tempdir().unwrap();
            let adapter = ClaudeAdapter::new(
                "claude".into(),
                executable(&temp, behavior),
                fixture.store.clone(),
            )
            .unwrap();
            let session = adapter.start(fixture.request.clone()).await.unwrap();
            if behavior == "background" {
                tokio::time::timeout(Duration::from_secs(3), async {
                    loop {
                        if adapter
                            .usage((&session).into(), "execute".into(), None)
                            .await
                            .unwrap()
                            .input_tokens
                            .is_some()
                        {
                            break;
                        }
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .unwrap();
            }
            let status = adapter.stop((&session).into()).await.unwrap();
            assert_eq!(status.session.state, SessionState::Lost);
            assert!(status.session.pid.is_none());
            assert!(crate::git::executor_reserved(&status.session));
            assert!(!adapter.transport_succeeded(&status));
            if behavior == "background" {
                assert_eq!(
                    adapter
                        .usage((&session).into(), "execute".into(), None)
                        .await
                        .unwrap()
                        .input_tokens,
                    Some(2)
                );
            }
            assert_eq!(
                adapter
                    .start(fixture.request.clone())
                    .await
                    .unwrap_err()
                    .kind,
                ErrorKind::StateConflict
            );
        }
    }
    #[tokio::test]
    async fn consumed_executor_input_without_terminal_keeps_lost_reservation_after_group_death() {
        let fixture = Fixture::new(true);
        let temp = tempfile::tempdir().unwrap();
        let adapter = ClaudeAdapter::new(
            "claude".into(),
            executable(&temp, "hang"),
            fixture.store.clone(),
        )
        .unwrap()
        .with_turn_timeout(Duration::from_millis(200))
        .unwrap();
        let session = adapter.start(fixture.request.clone()).await.unwrap();
        let status = terminal(&adapter, (&session).into()).await;
        assert_eq!(status.session.state, SessionState::Lost);
        assert!(status.session.pid.is_none());
        assert!(crate::git::executor_reserved(&status.session));
        assert!(!adapter.transport_succeeded(&status));
        assert_eq!(
            adapter
                .start(fixture.request.clone())
                .await
                .unwrap_err()
                .kind,
            ErrorKind::StateConflict
        );
    }
    #[tokio::test]
    async fn permission_before_native_init_never_establishes_grant_authority() {
        let fixture = Fixture::new(true);
        let temp = tempfile::tempdir().unwrap();
        let adapter = ClaudeAdapter::new(
            "claude".into(),
            executable(&temp, "preinit"),
            fixture.store.clone(),
        )
        .unwrap()
        .with_runtime_broker();
        let session = adapter.start(fixture.request.clone()).await.unwrap();
        let status = terminal(&adapter, (&session).into()).await;
        assert_eq!(status.session.state, SessionState::Lost);
        assert!(!adapter.transport_succeeded(&status));
        assert!(status.session.recovery.get("pending_permission").is_none());
        assert!(adapter.pending_operation((&session).into()).is_err());
    }
    #[tokio::test]
    async fn exact_pending_broker_escalation_reply_replay_and_stop() {
        let fixture = Fixture::new(true);
        let temp = tempfile::tempdir().unwrap();
        let adapter = ClaudeAdapter::new(
            "claude".into(),
            executable(&temp, "pending"),
            fixture.store.clone(),
        )
        .unwrap()
        .with_runtime_broker();
        let session = adapter.start(fixture.request.clone()).await.unwrap();
        let mut receiver = adapter.subscribe((&session).into()).unwrap();
        tokio::time::timeout(Duration::from_secs(10), async {
            while receiver.borrow().session.state != SessionState::WaitingApproval {
                receiver.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        let pending = receiver.borrow().session.recovery["pending_permission"].clone();
        assert!(pending.get("input").is_none());
        assert_eq!(
            adapter.pending_operation((&session).into()).unwrap()["input"],
            json!({"command":"pwd"})
        );
        let mut reply = pending.as_object().unwrap().clone();
        reply.remove("tool_name");
        reply.insert("decision".into(), "ESCALATE".into());
        adapter
            .submit_approval((&session).into(), Value::Object(reply.clone()))
            .await
            .unwrap();
        assert_eq!(
            adapter
                .status((&session).into())
                .await
                .unwrap()
                .session
                .state,
            SessionState::WaitingApproval
        );
        let mut wrong = reply.clone();
        wrong.insert("tool_use_id".into(), "foreign".into());
        assert_eq!(
            adapter
                .submit_approval((&session).into(), Value::Object(wrong))
                .await
                .unwrap_err()
                .kind,
            ErrorKind::OwnershipMismatch
        );
        reply.insert("decision".into(), "ALLOW".into());
        adapter
            .submit_approval((&session).into(), Value::Object(reply.clone()))
            .await
            .unwrap();
        let status = terminal(&adapter, (&session).into()).await;
        assert!(adapter.transport_succeeded(&status));
        assert!(
            adapter
                .submit_approval((&session).into(), Value::Object(reply))
                .await
                .is_err()
        );
        assert!(
            fixture
                .store
                .lock()
                .unwrap()
                .events(&fixture.request.scope, 0, 100)
                .unwrap()
                .iter()
                .any(|e| e.kind == "session.saved"
                    && e.data["evidence"]["dispatch_intent"]["kind"] == "permission"
                    && e.data["evidence"]["dispatch_intent"]["decision"] == "ALLOW"
                    && e.data["evidence"]["dispatch_intent"]["operation"]
                        .get("input")
                        .is_none())
        );
    }
    #[tokio::test]
    async fn terminal_is_real_scoped_live_owned_and_never_fabricates_turn_completion() {
        let fixture = Fixture::new(false);
        let temp = tempfile::tempdir().unwrap();
        let adapter = ClaudeAdapter::new(
            "claude".into(),
            executable(&temp, "success"),
            fixture.store.clone(),
        )
        .unwrap();
        let mut request = fixture.request.clone();
        request.mode = LaunchMode::Interactive;
        let session = adapter.start(request).await.unwrap();
        let mut receiver = adapter.subscribe((&session).into()).unwrap();
        tokio::time::timeout(Duration::from_secs(5), async {
            while !String::from_utf8_lossy(&receiver.borrow().stdout).contains("REAL_TTY=True") {
                receiver.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        adapter.attach((&session).into()).await.unwrap();
        adapter
            .terminal_resize((&session).into(), 40, 100)
            .await
            .unwrap();
        adapter
            .terminal_input((&session).into(), b"QUESTION\n".to_vec())
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(5), async {
            while !String::from_utf8_lossy(&receiver.borrow().stdout)
                .contains("NATIVE_TERMINAL_ANSWER")
            {
                receiver.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        let mut foreign = SessionRef::from(&session);
        foreign.scope.project_id = crate::domain::ProjectId::new();
        assert_eq!(
            adapter
                .terminal_input(foreign, b"QUESTION\n".to_vec())
                .await
                .unwrap_err()
                .kind,
            ErrorKind::OwnershipMismatch
        );
        let status = adapter.stop((&session).into()).await.unwrap();
        assert_eq!(status.session.state, SessionState::Stopped);
        assert!(!adapter.transport_succeeded(&status));
        assert!(adapter.attach((&session).into()).await.is_err());
        let usage = adapter
            .usage((&session).into(), "interactive".into(), None)
            .await
            .unwrap();
        assert!(usage.input_tokens.is_none() && usage.estimated_cost.is_none());
    }
    #[tokio::test]
    #[ignore = "actual native UI fixture on authorized public primary repository; no automatic trust selection"]
    async fn actual_native_terminal_response_or_preserved_trust_prompt() {
        use crate::{
            adapter::{InputKind, PreparedInput},
            domain::{Project, Scope},
            state::Store,
        };
        use std::collections::BTreeMap;
        let root = PathBuf::from("/Users/shuheisuzuki/Documents/dev/rururunx")
            .canonicalize()
            .unwrap();
        let mut project = Project::new(
            "public-native-ui-proof".into(),
            root.clone(),
            crate::git::repository_identity(&root, "main").unwrap(),
            "main".into(),
        );
        let mut store = Store::memory().unwrap();
        store.put_project(&mut project).unwrap();
        let scope = Scope::project(project.id);
        let revision = std::process::Command::new("/usr/bin/git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&root)
            .output()
            .unwrap();
        let revision = String::from_utf8(revision.stdout)
            .unwrap()
            .trim()
            .to_owned();
        let request=LaunchRequest{project,scope:scope.clone(),worktree:root,role:crate::domain::SessionRole::Consultant,mode:LaunchMode::Interactive,input:PreparedInput{scope,kind:InputKind::ContextPack,revision,version:1,source_versions:BTreeMap::new(),payload:"Independent MANUAL READ-ONLY terminal protocol fixture. This is the read-only-session exception: no implementation, tools, repository operations or delegation. Preserve native rules. Reply with the concatenation of NATIVE_PTY_RRX5_ and RESPONSE, without spaces or any other text.".into()},environment:BTreeMap::new(),model:None,effort:None};
        let adapter = ClaudeAdapter::new(
            "claude".into(),
            "/Users/shuheisuzuki/.nvm/versions/node/v22.19.0/bin/claude".into(),
            Arc::new(Mutex::new(store)),
        )
        .unwrap()
        .with_turn_timeout(Duration::from_secs(60))
        .unwrap();
        let session = adapter.start(request).await.unwrap();
        let mut receiver = adapter.subscribe((&session).into()).unwrap();
        adapter.attach((&session).into()).await.unwrap();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(12);
        let mut cursor_replies = 0;
        let (response, trust, setup) = loop {
            let status = receiver.borrow().clone();
            let text = super::super::pty::plain_terminal(&status.stdout);
            let lower = text.to_ascii_lowercase();
            let response = text.contains("NATIVE_PTY_RRX5_RESPONSE");
            let trust = lower.contains("do you trust")
                || lower.contains("trust this folder")
                || lower.contains("yes, i trust")
                || (lower.contains("trust") && lower.contains("safety"));
            let setup = lower.contains("text style")
                || lower.contains("choose the text")
                || lower.contains("let’s get started")
                || lower.contains("let's get started")
                || lower.contains("select your preferred")
                || lower.contains("welcome to claude");
            let cursor_queries = text.matches("\x1b[6n").count();
            if cursor_queries > cursor_replies {
                adapter
                    .terminal_input((&session).into(), b"\x1b[1;1R".to_vec())
                    .await
                    .unwrap();
                cursor_replies = cursor_queries;
            }
            if response || trust || status.terminal() {
                break (response, trust, setup);
            }
            if tokio::time::timeout_at(deadline, receiver.changed())
                .await
                .is_err()
            {
                break (response, trust, setup);
            }
        };
        let before_resize = receiver.borrow().stdout.len();
        adapter
            .terminal_resize((&session).into(), 40, 100)
            .await
            .unwrap();
        let resize_redrawn = tokio::time::timeout(Duration::from_secs(3), async {
            while receiver.borrow().stdout.len() <= before_resize {
                receiver.changed().await.unwrap();
            }
        })
        .await
        .is_ok();
        eprintln!(
            "actual native terminal resize: requested_rows=40, columns=100, native_redraw_observed={}",
            resize_redrawn
        );
        let status = adapter.stop((&session).into()).await.unwrap();
        let text = String::from_utf8_lossy(&status.stdout).to_ascii_lowercase();
        let labels = [
            "error",
            "terminal",
            "tty",
            "interactive",
            "permission",
            "claude",
            "please",
            "auth",
            "login",
            "trust",
            "theme",
            "dark",
            "light",
            "oauth",
            "thinking",
            "welcome",
            "color",
            "unknown",
            "stdout",
            "stream",
            "mcp",
            "format",
            "settings",
            "disabled",
            "working",
            "directory",
            "supported",
            "continue",
            "json",
            "prompt",
            "invalid",
            "usage",
            "command",
            "stdio",
            "session",
            "model",
            "help",
            "input",
            "required",
            "wizard",
            "safety",
            "press",
            "enter",
            "remote",
            "setup",
            "first",
            "restricted",
            "denied",
            "default",
            "argument",
            "display",
            "inspector",
            "background",
            "splash",
            "onboarding",
            "keyboard",
            "wait",
            "ctrl",
            "exit",
        ];
        eprintln!(
            "native UI fixed-category labels: {:?}; ascii_bytes={}; unicode_chars={}",
            labels
                .into_iter()
                .filter(|label| text.contains(label))
                .collect::<Vec<_>>(),
            status.stdout.iter().filter(|b| b.is_ascii()).count(),
            text.chars().filter(|c| !c.is_ascii()).count()
        );
        eprintln!(
            "actual native terminal evidence: response={}, trust_prompt_preserved={}, output_bytes={}, state={:?}, owned_cleanup_verified={}, setup_ui={}, cursor_replies={}",
            response,
            trust,
            status.stdout.len(),
            status.session.state,
            status.session.pid.is_none(),
            setup,
            cursor_replies
        );
        assert!(
            response || trust || setup,
            "native UI did not produce bounded consultation/trust evidence"
        );
        assert!(!adapter.transport_succeeded(&status));
    }
    #[tokio::test]
    #[ignore = "real native executor/reviewer fixture; explicit authorized host run only"]
    async fn actual_native_executor_reads_private_nonce_then_fresh_immutable_review() {
        let mut fixture = Fixture::new(true);
        let nonce = uuid::Uuid::new_v4().to_string();
        let file = fixture.request.worktree.join("proof.txt");
        std::fs::write(&file, &nonce).unwrap();
        let adapter = ClaudeAdapter::new(
            "claude".into(),
            "/Users/shuheisuzuki/.nvm/versions/node/v22.19.0/bin/claude".into(),
            fixture.store.clone(),
        )
        .unwrap()
        .with_turn_timeout(Duration::from_secs(120))
        .unwrap()
        .with_runtime_broker();
        let mut request = fixture.request.clone();
        request.input.payload="Independent MANUAL READ-ONLY execution fixture. This is the read-only-session exception: no implementation, edits, delegation, or operations except reading the one owned proof.txt file. Preserve all native rules. Use Read to read proof.txt in the current task directory and return only its text content, without decoration.".into();
        let session = adapter.start(request).await.unwrap();
        let mut receiver = adapter.subscribe((&session).into()).unwrap();
        tokio::time::timeout(Duration::from_secs(150), async {
            loop {
                let status = receiver.borrow().clone();
                if status.terminal() {
                    break;
                }
                if status.session.state == SessionState::WaitingApproval {
                    let mut operation = adapter.pending_operation((&session).into()).unwrap();
                    let expected = operation["tool_name"] == "Read"
                        && operation["input"]["file_path"]
                            .as_str()
                            .is_some_and(|name| std::path::Path::new(name) == file.as_path());
                    operation.as_object_mut().unwrap().remove("tool_name");
                    operation.as_object_mut().unwrap().remove("input");
                    operation["decision"] = if expected { "ALLOW" } else { "DENY" }.into();
                    adapter
                        .submit_approval((&session).into(), operation)
                        .await
                        .unwrap();
                }
                receiver.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        let status = adapter.status((&session).into()).await.unwrap();
        assert!(adapter.transport_succeeded(&status), "{:?}", status.failure);
        assert_eq!(String::from_utf8_lossy(&status.stdout).trim(), nonce);
        eprintln!(
            "actual executor read nonce matched; usage={}",
            serde_json::to_string(
                &adapter
                    .usage((&session).into(), "execute".into(), None)
                    .await
                    .unwrap()
            )
            .unwrap()
        );
        std::fs::remove_file(file).unwrap();
        fixture.review();
        fixture.request.input.payload="Independent MANUAL READ-ONLY immutable review fixture. This is the read-only-session exception: no implementation, execution, delegation, repository operations or tools. Preserve native rules. Factual target is a clean synthetic fixture with one committed .gitignore. Return only JSON {\"review_fixture\":true}.".into();
        let review = adapter.start(fixture.request.clone()).await.unwrap();
        assert_ne!(review.native_ref, session.native_ref);
        let mut receiver = adapter.subscribe((&review).into()).unwrap();
        tokio::time::timeout(Duration::from_secs(150), async {
            while !receiver.borrow().terminal() {
                receiver.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        let status = adapter.status((&review).into()).await.unwrap();
        assert!(adapter.transport_succeeded(&status), "{:?}", status.failure);
        assert_eq!(
            serde_json::from_slice::<Value>(&status.stdout).unwrap(),
            json!({"review_fixture":true})
        );
        eprintln!(
            "actual fresh immutable reviewer usage={}",
            serde_json::to_string(
                &adapter
                    .usage((&review).into(), "review".into(), Some(1))
                    .await
                    .unwrap()
            )
            .unwrap()
        );
    }
    #[tokio::test]
    #[ignore = "real installed native CLI/account probe; explicit authorized host run only"]
    async fn actual_native_zero_tools_and_exact_owned_resume() {
        let fixture = Fixture::new(false);
        let executable =
            PathBuf::from("/Users/shuheisuzuki/.nvm/versions/node/v22.19.0/bin/claude");
        let adapter =
            ClaudeAdapter::new("claude".into(), executable, fixture.store.clone()).unwrap();
        let mut request = fixture.request.clone();
        request.input.payload="Independent MANUAL READ-ONLY protocol fixture. This is the read-only-session exception: no implementation, tools, repository operations or delegation are requested. Retain ordinary native rules. Return only JSON {\"fixture\":true}.".into();
        let session = adapter.start(request).await.unwrap();
        let mut receiver = adapter.subscribe((&session).into()).unwrap();
        tokio::time::timeout(Duration::from_secs(180), async {
            while !receiver.borrow().terminal() {
                receiver.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        let status = adapter.status((&session).into()).await.unwrap();
        assert!(adapter.transport_succeeded(&status), "{:?}", status.failure);
        assert_eq!(
            serde_json::from_slice::<Value>(&status.stdout).unwrap(),
            json!({"fixture":true})
        );
        let usage = adapter
            .usage((&session).into(), "consult".into(), None)
            .await
            .unwrap();
        assert!(usage.input_tokens.is_some());
        eprintln!(
            "native fresh telemetry: {}",
            serde_json::to_string(&usage).unwrap()
        );
        let resumed = adapter.resume((&session).into()).await.unwrap();
        assert_eq!(session.native_ref, resumed.native_ref);
        let mut receiver = adapter.subscribe((&resumed).into()).unwrap();
        tokio::time::timeout(Duration::from_secs(180), async {
            while !receiver.borrow().terminal() {
                receiver.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        let status = adapter.status((&resumed).into()).await.unwrap();
        assert!(adapter.transport_succeeded(&status), "{:?}", status.failure);
        eprintln!(
            "native resume telemetry: {}",
            serde_json::to_string(
                &adapter
                    .usage((&resumed).into(), "resume".into(), None)
                    .await
                    .unwrap()
            )
            .unwrap()
        );
    }
}
