//! Scoped native session supervision. Workflow verdicts remain caller-owned.
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::{mpsc, oneshot, watch};

use super::{
    ownership::{ProcessOwnership, ScopeSnapshot, filesystem},
    policy::{DecisionPolicy, native_environment, provider_environment, verify_auth_readiness},
    protocol::{
        ApprovalLedger, Event, NativeRpc, OperationDecision, RpcId, TokenCounters, UsageTracker,
        failure, operation_paths,
    },
    transport::NativeServer,
};
use crate::{
    adapter::{
        AdapterFuture, AdapterResult, AgentAdapter, AgentInfo, Capability, ErrorKind, LaunchMode,
        LaunchRequest, PreparedInput, SessionRef, SessionStatus, SharedStore,
    },
    domain::{Session, SessionId, SessionRole, SessionState, Usage, now_ms},
};

const OUTPUT_LIMIT: usize = 64 * 1024;
const RETAINED_TERMINALS: usize = 32;

fn evict_one_terminal(sessions: &mut HashMap<SessionId, Entry>) {
    if let Some(id) = sessions
        .iter()
        .filter(|(_, entry)| {
            entry.status.borrow().terminal() && !entry.transition.load(Ordering::SeqCst)
        })
        .min_by_key(|(_, entry)| entry.status.borrow().session.started_at)
        .map(|(id, _)| *id)
    {
        sessions.remove(&id);
    }
}

pub struct CodexAdapter {
    agent: String,
    executable: PathBuf,
    store: SharedStore,
    sessions: Mutex<HashMap<SessionId, Entry>>,
    runtime_broker: bool,
}
struct Entry {
    status: watch::Receiver<SessionStatus>,
    publisher: watch::Sender<SessionStatus>,
    transition: Arc<AtomicBool>,
    stop: mpsc::Sender<()>,
    reply: mpsc::Sender<Reply>,
    evidence: Arc<Mutex<Evidence>>,
    request: LaunchRequest,
    schema: Option<Value>,
}
struct TransitionClaim(Arc<AtomicBool>);
impl Drop for TransitionClaim {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}
struct ResumePublication {
    previous: SessionStatus,
    sender: watch::Sender<SessionStatus>,
    evidence: Arc<Mutex<Evidence>>,
}
type OwnedReference = (
    watch::Receiver<SessionStatus>,
    mpsc::Sender<()>,
    Arc<Mutex<Evidence>>,
    mpsc::Sender<Reply>,
);
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DecisionRequest {
    native_turn: String,
    request_id: RpcId,
    decision: OperationDecision,
    operation_hash: String,
}
struct Reply {
    request: DecisionRequest,
    result: oneshot::Sender<AdapterResult<()>>,
}
#[derive(Default)]
struct Evidence {
    completed: bool,
    turn: Option<String>,
    counters: Option<TokenCounters>,
    cumulative: Option<TokenCounters>,
    pending: Option<ApprovalLedger>,
}
struct NativeTurn {
    thread: String,
    turn: String,
    previous_turn: Option<String>,
    previous_cumulative: Option<TokenCounters>,
    authority: ScopeSnapshot,
    request: LaunchRequest,
    binding: Value,
    runtime_broker: bool,
}
struct Reservation {
    store: SharedStore,
    session: Session,
    version: u64,
    ownership: ProcessOwnership,
    armed: bool,
    resume_publication: Option<ResumePublication>,
    inference_started: bool,
}
impl Reservation {
    fn persist(&mut self) -> AdapterResult<()> {
        let mut store = self
            .store
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?;
        self.version = store
            .put_session(&self.session, self.version)
            .map_err(|e| failure(ErrorKind::StateConflict, e.to_string()))?;
        if let Some(publication) = &self.resume_publication {
            let mut status = publication.sender.borrow().clone();
            status.session = self.session.clone();
            publication.sender.send_replace(status);
        }
        Ok(())
    }
    fn publish(
        &mut self,
        sender: &watch::Sender<SessionStatus>,
        status: &mut SessionStatus,
    ) -> AdapterResult<()> {
        let mut store = self
            .store
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?;
        self.version = store
            .put_session(&self.session, self.version)
            .map_err(|error| failure(ErrorKind::StateConflict, error.to_string()))?;
        status.session = self.session.clone();
        sender.send_replace(status.clone());
        Ok(())
    }
    fn terminal(
        &mut self,
        status: &mut SessionStatus,
        result: &AdapterResult<bool>,
        cleanup: &AdapterResult<std::process::ExitStatus>,
    ) -> bool {
        let uncertain = cleanup.is_err() || self.ownership.uncertain();
        self.session.state = if uncertain {
            SessionState::Lost
        } else {
            match result {
                Ok(true) => SessionState::Exited,
                Ok(false) => SessionState::Stopped,
                Err(_) => SessionState::Failed,
            }
        };
        if !uncertain {
            self.session.pid = None;
        }
        // The server process's exit status is not a native model-turn outcome.
        status.exit_code = None;
        status.failure = cleanup
            .as_ref()
            .err()
            .or(result.as_ref().err())
            .map(|error| error.to_string());
        if uncertain {
            status
                .failure
                .get_or_insert_with(|| "owned process cleanup is unverified".into());
            let diagnostic = json!({
                "session_id":self.session.id,"native_uuid":self.session.native_ref,
                "native_pid":self.session.pid,"native_group_cleanup_confirmed":cleanup.is_ok(),
                "other_owned_group_uncertain":self.ownership.uncertain(),
                "state_intent":"Lost","failure":status.failure,
            });
            let audit = self.store.lock().map_err(|_| ()).and_then(|mut store| {
                store
                    .audit(&self.session.scope, "codex.cleanup.unverified", diagnostic)
                    .map_err(|_| ())
            });
            if audit.is_err() {
                status.failure =
                    Some("owned cleanup and diagnostic publication are unverified".into());
            }
        }
        matches!(result, Ok(true)) && !uncertain
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if self.armed {
            let uncertain = self.ownership.uncertain();
            if !uncertain
                && !self.inference_started
                && let Some(publication) = &self.resume_publication
            {
                self.session = publication.previous.session.clone();
            } else {
                self.session.state = if uncertain {
                    SessionState::Lost
                } else {
                    SessionState::Failed
                };
                if !uncertain {
                    self.session.pid = None;
                }
            }
            if let Some(publication) = &self.resume_publication {
                let mut status = publication.previous.clone();
                if uncertain || self.inference_started {
                    if let Ok(mut evidence) = publication.evidence.lock() {
                        *evidence = Evidence::default();
                    }
                    status.stdout.clear();
                    status.stderr.clear();
                    status.exit_code = None;
                    status.failure =
                        Some("native resume did not establish an owned running supervisor".into());
                }
                let sender = publication.sender.clone();
                let _ = self.publish(&sender, &mut status);
            } else {
                let _ = self.persist();
            }
        }
    }
}

impl CodexAdapter {
    pub fn new(agent: String, executable: PathBuf, store: SharedStore) -> AdapterResult<Self> {
        if agent.trim().is_empty() || agent.len() > 128 || !executable.is_absolute() {
            return Err(failure(
                ErrorKind::InvalidConfiguration,
                "native Codex needs a named adapter and absolute executable",
            ));
        }
        Ok(Self {
            agent,
            executable,
            store,
            sessions: Mutex::new(HashMap::new()),
            runtime_broker: false,
        })
    }
    /// Opt-in integration for a trusted runtime Approval Broker. Native policy
    /// must already select its user/client route; automatic review is retained
    /// otherwise and launch fails explicitly before inference.
    pub fn with_runtime_broker(mut self) -> Self {
        self.runtime_broker = true;
        self
    }
    fn registry(&self) -> AdapterResult<std::sync::MutexGuard<'_, HashMap<SessionId, Entry>>> {
        self.sessions
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "native Session registry poisoned"))
    }
    fn reference(&self, reference: &SessionRef) -> AdapterResult<OwnedReference> {
        let sessions = self.registry()?;
        let entry = sessions.get(&reference.id).ok_or_else(|| {
            failure(
                ErrorKind::SessionLost,
                "native Session has no owned supervisor; PID/UUID hints cannot reconnect",
            )
        })?;
        if entry.status.borrow().session.scope != reference.scope {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "foreign native Session reference",
            ));
        }
        Ok((
            entry.status.clone(),
            entry.stop.clone(),
            entry.evidence.clone(),
            entry.reply.clone(),
        ))
    }
    fn claim(
        &self,
        reference: &SessionRef,
    ) -> AdapterResult<(TransitionClaim, LaunchRequest, Option<Value>)> {
        let sessions = self.registry()?;
        let entry = sessions.get(&reference.id).ok_or_else(|| {
            failure(
                ErrorKind::SessionLost,
                "native transition owner disappeared",
            )
        })?;
        if entry.status.borrow().session.scope != reference.scope {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "foreign native transition scope",
            ));
        }
        entry
            .transition
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| {
                failure(
                    ErrorKind::StateConflict,
                    "native Session transition already claimed",
                )
            })?;
        Ok((
            TransitionClaim(entry.transition.clone()),
            entry.request.clone(),
            entry.schema.clone(),
        ))
    }
    fn current(&self, reference: &SessionRef) -> AdapterResult<SessionStatus> {
        let (receiver, _, _, _) = self.reference(reference)?;
        let store = self
            .store
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?;
        let status = receiver.borrow().clone();
        let persisted = store
            .session(reference.id)
            .map_err(|e| failure(ErrorKind::StateFailure, e.to_string()))?
            .ok_or_else(|| failure(ErrorKind::SessionLost, "native Session record disappeared"))?
            .0;
        if serde_json::to_value(&persisted).ok() != serde_json::to_value(&status.session).ok() {
            return Err(failure(
                ErrorKind::StateConflict,
                "native Session changed outside its owned supervisor",
            ));
        }
        Ok(status)
    }
    async fn launch(
        &self,
        request: LaunchRequest,
        schema: Option<Value>,
        resume: Option<Session>,
    ) -> AdapterResult<Session> {
        if request.mode == LaunchMode::Interactive {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "native interactive policy requires an owned TUI lifecycle",
            ));
        }
        if schema.as_ref().is_some_and(|schema| {
            !schema.is_object()
                || serde_json::to_vec(schema).map_or(true, |bytes| bytes.len() > 64 * 1024)
        }) {
            return Err(failure(
                ErrorKind::InvalidInput,
                "native output schema must be a bounded JSON object",
            ));
        }
        let snapshot = ScopeSnapshot::capture(&self.store, &request, &self.agent)?;
        // The baseline comes only from the owned supervisor, never caller JSON
        // or a native UUID hint. Missing history remains unknown on resume.
        let previous_cumulative = if let Some(previous) = &resume {
            let (_, _, evidence, _) = self.reference(&SessionRef::from(previous))?;
            let evidence = evidence
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "native telemetry poisoned"))?;
            if previous.state == SessionState::Exited && evidence.completed {
                evidence.cumulative.clone()
            } else {
                None
            }
        } else {
            None
        };
        let mut session = resume.clone().unwrap_or_else(|| Session {
            id: SessionId::new(), scope: request.scope.clone(), agent: self.agent.clone(), provider: "codex".into(), role: request.role,
            native_ref: None, pid: None, worktree: request.worktree.clone(), state: SessionState::Starting,
            model: request.model.clone(), effort: request.effort.clone(),
            recovery: json!({"project_root":request.project.root,"input_revision":request.input.revision,"input_version":request.input.version,"source_versions":request.input.source_versions,"input_bytes":request.input.payload.len(),"reconnect_supported":false}), started_at: now_ms(),
        });
        let expected_version = if let Some(previous) = &resume {
            if previous.scope != request.scope
                || previous.worktree != request.worktree
                || previous.agent != self.agent
                || previous.provider != "codex"
                || previous.role != request.role
                || previous.native_ref.is_none()
                || !matches!(previous.state, SessionState::Exited | SessionState::Stopped)
            {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "native resume requires an exact owned completed/stopped Session",
                ));
            }
            let (persisted, version) = self
                .store
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?
                .session(previous.id)
                .map_err(|e| failure(ErrorKind::StateFailure, e.to_string()))?
                .ok_or_else(|| failure(ErrorKind::SessionLost, "native resume record missing"))?;
            if serde_json::to_value(persisted).ok() != serde_json::to_value(previous).ok() {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "native resume owner changed before reservation",
                ));
            }
            version
        } else {
            0
        };
        session.state = SessionState::Starting;
        session.pid = None;
        let resume_publication = if let Some(previous) = &resume {
            let sessions = self.registry()?;
            let entry = sessions.get(&previous.id).ok_or_else(|| {
                failure(ErrorKind::SessionLost, "native resume owner disappeared")
            })?;
            Some(ResumePublication {
                previous: entry.status.borrow().clone(),
                sender: entry.publisher.clone(),
                evidence: entry.evidence.clone(),
            })
        } else {
            None
        };
        let mut reservation = Reservation {
            store: self.store.clone(),
            session,
            version: expected_version,
            ownership: ProcessOwnership::default(),
            armed: false,
            resume_publication,
            inference_started: false,
        };
        reservation.persist()?;
        reservation.armed = true;
        let binding = snapshot
            .verify_git(&request, &mut reservation.ownership)
            .await?;
        snapshot.recheck(&self.store, &request, &self.agent)?;
        let executable = self.executable.clone();
        let executable = filesystem(move || {
            use std::os::unix::fs::PermissionsExt;
            let metadata = executable.metadata().map_err(|_| {
                failure(
                    ErrorKind::ExecutableMissing,
                    "native Codex executable unavailable",
                )
            })?;
            if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
                return Err(failure(
                    ErrorKind::ExecutableMissing,
                    "native Codex executable is not executable",
                ));
            }
            Ok(executable)
        })
        .await?;
        let baseline: Vec<_> = std::env::vars_os().collect();
        let environment = native_environment(
            baseline.iter().cloned(),
            &snapshot.projects,
            &snapshot.project,
            &request.environment,
        )?;
        let mut discovery = NativeServer::launch(
            &executable,
            &request.worktree,
            None,
            environment.clone(),
            reservation.ownership.group(),
        )
        .await?;
        reservation.session.pid = Some(discovery.pid());
        reservation.persist()?;
        let discovered = async {
            let config = discovery
                .rpc
                .call(
                    "config/read",
                    json!({"cwd":request.worktree,"includeLayers":false}),
                )
                .await?;
            let policy = if request.role == SessionRole::Executor {
                DecisionPolicy::for_executor(&config["config"])
            } else {
                DecisionPolicy::from_native(&config["config"])
            }?;
            let environment = provider_environment(
                &config["config"],
                &baseline,
                &snapshot.projects,
                &snapshot.project,
                &request.environment,
            )?;
            Ok::<_, crate::adapter::AdapterError>((policy, environment))
        }
        .await;
        discovery.shutdown().await?;
        let (policy, environment) = discovered?;
        if self.runtime_broker
            && request.role == SessionRole::Executor
            && !policy.permits_runtime_broker()
        {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "native approval policy does not expose the runtime-broker client route; native reviewer retained",
            ));
        }
        let mut native = NativeServer::launch(
            &executable,
            &request.worktree,
            Some(&policy),
            environment,
            reservation.ownership.group(),
        )
        .await?;
        reservation.session.pid = Some(native.pid());
        reservation.persist()?;
        let setup = async {
            let config = native
                .rpc
                .call(
                    "config/read",
                    json!({"cwd":request.worktree,"includeLayers":false}),
                )
                .await?;
            policy.verify_configuration(&config["config"])?;
            let account = native
                .rpc
                .call("account/read", json!({"refreshToken":false}))
                .await?;
            verify_auth_readiness(&account)?;
            if request.role == SessionRole::Executor {
                let environment = native
                    .rpc
                    .call("environment/status", json!({"environmentId":"local"}))
                    .await?;
                policy.verify_local_environment(&environment)?;
            }
            let mut parameters = policy.thread_parameters(&request.worktree);
            if let Some(model) = &request.model {
                parameters["model"] = json!(model);
            }
            // Native effort is a per-turn field; no native default is fabricated.
            let response = if let Some(previous) = &resume {
                parameters["threadId"] = json!(previous.native_ref);
                native.rpc.call("thread/resume", parameters).await?
            } else {
                native.rpc.call("thread/start", parameters).await?
            };
            let thread = policy.verify_thread(&response, &request.worktree)?;
            if resume
                .as_ref()
                .and_then(|session| session.native_ref.as_deref())
                .is_some_and(|expected| expected != thread)
            {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "native resume returned a foreign UUID",
                ));
            }
            let mut cursor: Option<String> = None;
            let mut finished = false;
            for _ in 0..32 {
                let page = native
                    .rpc
                    .call(
                        "mcpServerStatus/list",
                        json!({"threadId":thread,"limit":100,"cursor":cursor}),
                    )
                    .await?;
                cursor = policy.verify_inventory_page(&page)?;
                if cursor.is_none() {
                    finished = true;
                    break;
                }
            }
            if !finished {
                return Err(failure(
                    ErrorKind::ParseFailure,
                    "native inventory pagination exceeded bound",
                ));
            }
            reservation.session.native_ref = Some(thread.clone());
            reservation.session.model = response["model"].as_str().map(str::to_owned);
            reservation.session.effort = request
                .effort
                .clone()
                .or_else(|| response["reasoningEffort"].as_str().map(str::to_owned));
            reservation.persist()?;
            // No model receives stale context after native startup, hooks or discovery.
            snapshot
                .verify_binding(&request, &mut reservation.ownership, &binding)
                .await?;
            snapshot.recheck(&self.store, &request, &self.agent)?;
            let mut parameters =
                json!({"threadId":thread,"input":[{"type":"text","text":request.input.payload}]});
            if let Some(schema) = &schema {
                parameters["outputSchema"] = schema.clone();
            }
            if let Some(effort) = &request.effort {
                parameters["effort"] = json!(effort);
            }
            // A cancelled/failed call can already have started inference.
            reservation.inference_started = true;
            let turn = native.rpc.call("turn/start", parameters).await?;
            let turn = turn["turn"]["id"]
                .as_str()
                .filter(|id| !id.is_empty() && id.len() <= 256)
                .ok_or_else(|| failure(ErrorKind::ParseFailure, "native turn ID unavailable"))?
                .to_owned();
            Ok::<_, crate::adapter::AdapterError>((thread, turn))
        }
        .await;
        let (thread, turn) = match setup {
            Ok(value) => value,
            Err(error) => {
                native.shutdown().await?;
                return Err(error);
            }
        };
        reservation.session.state = SessionState::Running;
        reservation.session.recovery["native_turn"] = json!(turn);
        reservation.session.recovery["input_revision"] = json!(request.input.revision);
        reservation.session.recovery["input_version"] = json!(request.input.version);
        reservation.session.recovery["input_bytes"] = json!(request.input.payload.len());
        reservation.session.recovery["source_versions"] = json!(request.input.source_versions);
        if let Err(error) = reservation.persist() {
            native.shutdown().await?;
            return Err(error);
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
        let (sender, receiver) = if let Some(publication) = &reservation.resume_publication {
            let sender = publication.sender.clone();
            sender.send_replace(status);
            let receiver = sender.subscribe();
            (sender, receiver)
        } else {
            watch::channel(status)
        };
        let (stop, stopped) = mpsc::channel(1);
        let (reply, replies) = mpsc::channel(16);
        let evidence = Arc::new(Mutex::new(Evidence {
            turn: Some(turn.clone()),
            pending: Some(
                ApprovalLedger::new(thread.clone(), turn.clone())
                    .for_workspace(request.worktree.clone()),
            ),
            ..Evidence::default()
        }));
        {
            let mut sessions = self.registry()?;
            if sessions.len() >= RETAINED_TERMINALS {
                evict_one_terminal(&mut sessions);
            }
            sessions.insert(
                session.id,
                Entry {
                    status: receiver,
                    publisher: sender.clone(),
                    transition: Arc::new(AtomicBool::new(false)),
                    stop,
                    reply,
                    evidence: evidence.clone(),
                    request: request.clone(),
                    schema,
                },
            );
        }
        tokio::spawn(supervise(
            native,
            reservation,
            sender,
            stopped,
            evidence,
            replies,
            NativeTurn {
                thread,
                turn,
                previous_turn: resume.and_then(|session| {
                    session.recovery["native_turn"].as_str().map(str::to_owned)
                }),
                previous_cumulative,
                authority: snapshot,
                request,
                binding,
                runtime_broker: self.runtime_broker,
            },
        ));
        Ok(session)
    }
}

impl AgentAdapter for CodexAdapter {
    fn capabilities(&self) -> BTreeSet<Capability> {
        [
            Capability::Execute,
            Capability::Consult,
            Capability::Review,
            Capability::NonInteractive,
            Capability::StructuredOutput,
            Capability::UsageTelemetry,
            Capability::PromptCacheTelemetry,
            Capability::Resume,
            Capability::ContextCheckpoint,
            Capability::PermissionInterception,
        ]
        .into()
    }
    fn probe(&self) -> AdapterResult<AgentInfo> {
        Ok(AgentInfo {
            agent: self.agent.clone(),
            provider: "codex".into(),
            adapter_version: env!("CARGO_PKG_VERSION").into(),
            executable: self.executable.clone(),
            authenticated: None,
            model_configuration: true,
            effort_configuration: true,
            capabilities: self.capabilities(),
        })
    }
    fn start(&self, request: LaunchRequest) -> AdapterFuture<'_, Session> {
        Box::pin(self.launch(request, None, None))
    }
    fn start_structured(
        &self,
        request: LaunchRequest,
        schema: Value,
    ) -> AdapterFuture<'_, Session> {
        Box::pin(self.launch(request, Some(schema), None))
    }
    fn status(&self, session: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        Box::pin(async move { self.current(&session) })
    }
    fn stop(&self, session: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        Box::pin(async move {
            let (mut receiver, stop, _, _) = self.reference(&session)?;
            if !receiver.borrow().terminal() {
                match stop.try_send(()) {
                    Ok(()) | Err(mpsc::error::TrySendError::Full(())) => {}
                    Err(mpsc::error::TrySendError::Closed(())) => {
                        return Err(failure(
                            ErrorKind::SessionLost,
                            "native stop supervisor unavailable",
                        ));
                    }
                }
                while !receiver.borrow().terminal() {
                    receiver.changed().await.map_err(|_| {
                        failure(
                            ErrorKind::SessionLost,
                            "native supervisor disappeared during stop",
                        )
                    })?;
                }
            }
            self.current(&session)
        })
    }
    fn attach(&self, _session: SessionRef) -> AdapterFuture<'_, ()> {
        Box::pin(async {
            Err(failure(
                ErrorKind::UnsupportedCapability,
                "native interactive connection requires a verified owned TUI lifecycle",
            ))
        })
    }
    fn resume(&self, session: SessionRef) -> AdapterFuture<'_, Session> {
        Box::pin(async move {
            let (_claim, request, schema) = self.claim(&session)?;
            let previous = self.current(&session)?.session;
            self.launch(request, schema, Some(previous)).await
        })
    }
    fn release(&self, session: SessionRef) -> AdapterResult<()> {
        let (_claim, _, _) = self.claim(&session)?;
        let status = self.current(&session)?;
        if !status.terminal() {
            return Err(failure(
                ErrorKind::StateConflict,
                "running native Session cannot be released",
            ));
        }
        self.registry()?.remove(&session.id);
        Ok(())
    }
    fn subscribe(&self, session: SessionRef) -> AdapterResult<watch::Receiver<SessionStatus>> {
        self.reference(&session).map(|(receiver, _, _, _)| receiver)
    }
    fn usage(
        &self,
        session: SessionRef,
        phase: String,
        review_round: Option<u32>,
    ) -> AdapterFuture<'_, Usage> {
        Box::pin(async move {
            if phase.trim().is_empty() {
                return Err(failure(ErrorKind::InvalidInput, "usage phase is blank"));
            }
            let status = self.current(&session)?;
            let (_, _, evidence, _) = self.reference(&session)?;
            let evidence = evidence
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "native telemetry poisoned"))?;
            let counters = evidence.counters.clone().unwrap_or_default();
            Ok(Usage {
                scope: status.session.scope,
                session_id: session.id,
                agent: self.agent.clone(),
                phase,
                review_round,
                input_tokens: counters.input,
                cached_input_tokens: counters.cached_input,
                output_tokens: counters.output,
                estimated_cost: None,
                context_pack_version: status.session.recovery["input_version"].as_u64(),
                context_pack_size: status.session.recovery["input_bytes"].as_u64(),
                repo_map_size: None,
                cache_metadata: json!({"native_uuid":status.session.native_ref,"native_turn":evidence.turn,"counter_scope":"current_turn","cache_write_input_tokens":counters.cache_write_input,"reasoning_output_tokens":counters.reasoning_output,"total_tokens":counters.total}),
                missing_reason: Some(
                    if evidence.counters.is_some() {
                        "native Codex does not expose monetary cost; counters missing a resume baseline or reset remain unavailable"
                    } else {
                        "native token notification not observed; monetary cost unavailable"
                    }
                    .into(),
                ),
            })
        })
    }
    fn pending_approvals(&self, session: SessionRef) -> AdapterFuture<'_, Value> {
        Box::pin(async move {
            let status = self.current(&session)?;
            let (_, _, evidence, _) = self.reference(&session)?;
            let evidence = evidence
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "native approval state poisoned"))?;
            Ok(
                json!({"scope":status.session.scope,"session_id":session.id,"native_uuid":status.session.native_ref,"native_turn":evidence.turn,"requires_human":!self.runtime_broker,"requests":evidence.pending.as_ref().map(ApprovalLedger::pending).unwrap_or_default()}),
            )
        })
    }
    fn submit_approval(&self, session: SessionRef, decision: Value) -> AdapterFuture<'_, ()> {
        Box::pin(async move {
            if serde_json::to_vec(&decision).map_or(true, |value| value.len() > 4096) {
                return Err(failure(
                    ErrorKind::InvalidInput,
                    "native approval reply exceeds limit",
                ));
            }
            let request: DecisionRequest = serde_json::from_value(decision).map_err(|_| {
                failure(
                    ErrorKind::InvalidInput,
                    "native approval reply needs exact turn/request ID, reviewed operation digest and one operation decision",
                )
            })?;
            if self.current(&session)?.terminal() {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "terminal native Session has no pending operations",
                ));
            }
            if matches!(request.decision, OperationDecision::Approve) && !self.runtime_broker {
                return Err(failure(
                    ErrorKind::UnsupportedCapability,
                    "native approval requires its Human/client route; automatic review cannot be replaced by a runtime grant",
                ));
            }
            let (_, _, _, replies) = self.reference(&session)?;
            let (result, receiver) = oneshot::channel();
            replies.try_send(Reply { request, result }).map_err(|_| {
                failure(
                    ErrorKind::StateConflict,
                    "native approval channel unavailable or full",
                )
            })?;
            tokio::time::timeout(std::time::Duration::from_secs(40), receiver)
                .await
                .map_err(|_| {
                    failure(
                        ErrorKind::Timeout,
                        "native approval acknowledgement timed out; inspect its durable intent",
                    )
                })?
                .map_err(|_| {
                    failure(
                        ErrorKind::SessionLost,
                        "native approval supervisor disappeared",
                    )
                })?
        })
    }
    fn checkpoint(&self, session: SessionRef, input: PreparedInput) -> AdapterFuture<'_, ()> {
        Box::pin(async move {
            let (_claim, mut request, _) = self.claim(&session)?;
            let status = self.current(&session)?;
            if input.scope != status.session.scope {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "foreign native checkpoint scope",
                ));
            }
            if !status.terminal() || status.session.state == SessionState::Lost {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "native checkpoint requires a confirmed terminal Session",
                ));
            }
            if input.version <= request.input.version {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "native checkpoint must advance its prepared input version",
                ));
            }
            request.input = input;
            let snapshot = ScopeSnapshot::capture(&self.store, &request, &self.agent)?;
            let (persisted, version) = self
                .store
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?
                .session(session.id)
                .map_err(|error| failure(ErrorKind::StateFailure, error.to_string()))?
                .ok_or_else(|| {
                    failure(
                        ErrorKind::SessionLost,
                        "native checkpoint record disappeared",
                    )
                })?;
            if serde_json::to_value(persisted).ok() != serde_json::to_value(&status.session).ok() {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "native checkpoint owner changed before reservation",
                ));
            }
            let publication = {
                let sessions = self.registry()?;
                let entry = sessions.get(&session.id).ok_or_else(|| {
                    failure(
                        ErrorKind::SessionLost,
                        "native checkpoint owner disappeared",
                    )
                })?;
                ResumePublication {
                    previous: status.clone(),
                    sender: entry.publisher.clone(),
                    evidence: entry.evidence.clone(),
                }
            };
            let mut reservation = Reservation {
                store: self.store.clone(),
                session: status.session.clone(),
                version,
                ownership: ProcessOwnership::default(),
                armed: false,
                resume_publication: Some(publication),
                inference_started: false,
            };
            reservation.session.state = SessionState::Starting;
            reservation.persist()?;
            reservation.armed = true;
            // Checkpoints are validated before replacing resumable prepared input.
            snapshot
                .verify_git(&request, &mut reservation.ownership)
                .await?;
            snapshot.recheck(&self.store, &request, &self.agent)?;
            reservation.session = status.session.clone();
            reservation.persist()?;
            reservation.armed = false;
            self.current(&session)?;
            let mut sessions = self.registry()?;
            let entry = sessions.get_mut(&session.id).ok_or_else(|| {
                failure(
                    ErrorKind::SessionLost,
                    "native checkpoint owner disappeared",
                )
            })?;
            entry.request = request;
            Ok(())
        })
    }
}

struct ApprovalAuthority {
    snapshot: ScopeSnapshot,
    request: LaunchRequest,
    binding: Value,
    runtime_broker: bool,
}

async fn answer_approval(
    native: &mut NativeRpc,
    reservation: &mut Reservation,
    evidence: &Arc<Mutex<Evidence>>,
    authority: &ApprovalAuthority,
    sender: &watch::Sender<SessionStatus>,
    status: &mut SessionStatus,
    reply: Reply,
) -> AdapterResult<bool> {
    if reply.result.is_closed() {
        return Ok(false);
    }
    let preview = (|| {
        let evidence = evidence
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "native approval state poisoned"))?;
        if evidence.turn.as_deref() != Some(reply.request.native_turn.as_str()) {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "approval belongs to another native turn",
            ));
        }
        if matches!(reply.request.decision, OperationDecision::Approve) && !authority.runtime_broker
        {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "native Human approval cannot be granted through the runtime-broker route",
            ));
        }
        let pending = evidence
            .pending
            .as_ref()
            .ok_or_else(|| failure(ErrorKind::SessionLost, "native approval ledger unavailable"))?;
        let request = pending.request(&reply.request.request_id)?;
        if request.operation_hash != reply.request.operation_hash {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "native reply does not match the reviewed operation digest",
            ));
        }
        Ok((
            pending.preview_reply(&reply.request.request_id, reply.request.decision)?,
            request,
        ))
    })();
    let (preview, operation) = match preview {
        Ok(preview) => preview,
        Err(error) => {
            let _ = reply.result.send(Err(error));
            return Ok(false);
        }
    };
    if matches!(reply.request.decision, OperationDecision::Approve) {
        let preflight = async {
            let paths = operation_paths(&operation, &authority.request.worktree)?;
            let workspace = authority.request.worktree.clone();
            filesystem(move || {
                use std::os::unix::fs::MetadataExt;
                let canonical = workspace.canonicalize().map_err(|_| {
                    failure(
                        ErrorKind::OwnershipMismatch,
                        "native operation workspace unavailable",
                    )
                })?;
                for path in paths {
                    let mut ancestor = path.as_path();
                    loop {
                        match std::fs::symlink_metadata(ancestor) {
                            Ok(metadata) => {
                                if metadata.is_file() && metadata.nlink() > 1 {
                                    return Err(failure(
                                        ErrorKind::OwnershipMismatch,
                                        "native operation target has unscoped hard-link aliases",
                                    ));
                                }
                                if !ancestor
                                    .canonicalize()
                                    .map_err(|_| {
                                        failure(
                                            ErrorKind::OwnershipMismatch,
                                            "native operation target resolution unavailable",
                                        )
                                    })?
                                    .starts_with(&canonical)
                                {
                                    return Err(failure(
                                        ErrorKind::OwnershipMismatch,
                                        "native operation target resolves outside its workspace",
                                    ));
                                }
                                break;
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                                ancestor = ancestor.parent().ok_or_else(|| {
                                    failure(
                                        ErrorKind::OwnershipMismatch,
                                        "native operation target has no scoped ancestor",
                                    )
                                })?;
                            }
                            Err(_) => {
                                return Err(failure(
                                    ErrorKind::OwnershipMismatch,
                                    "native operation target metadata unavailable",
                                ));
                            }
                        }
                    }
                }
                Ok(())
            })
            .await?;
            authority
                .snapshot
                .verify_binding(
                    &authority.request,
                    &mut reservation.ownership,
                    &authority.binding,
                )
                .await?;
            authority.snapshot.recheck_scope(
                &reservation.store,
                &authority.request,
                &reservation.session.agent,
            )
        }
        .await;
        if let Err(error) = preflight {
            let fatal = reservation.ownership.uncertain();
            let kind = error.kind;
            let message = error.to_string();
            let _ = reply.result.send(Err(error));
            // No effect intent or native reply exists yet. Leave the request
            // pending so a transient/stale grant can be denied, cancelled or
            // reviewed again; unverified child cleanup still stops as Lost.
            return if fatal {
                Err(failure(kind, message))
            } else {
                Ok(false)
            };
        }
    }
    let validated = async {
        if reply.result.is_closed() { return Ok(false); }
        {
            let mut store = reservation.store.lock().map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?;
            let (persisted, version) = store.session(reservation.session.id).map_err(|error| failure(ErrorKind::StateFailure, error.to_string()))?
                .ok_or_else(|| failure(ErrorKind::SessionLost, "native approval Session disappeared"))?;
            if version != reservation.version || serde_json::to_value(persisted).ok() != serde_json::to_value(&reservation.session).ok() {
                return Err(failure(ErrorKind::StateConflict, "native approval owner changed before reply"));
            }
            let data = json!({"session_id":reservation.session.id,"native_uuid":reservation.session.native_ref,"native_turn":reply.request.native_turn,"request_id":reply.request.request_id,"operation_hash":operation.operation_hash,"reply":preview,"effect":"intent_only"});
            if matches!(reply.request.decision, OperationDecision::Approve) {
                store.audit_if_current(&reservation.session.scope, [authority.snapshot.project.version, authority.snapshot.goal.as_ref().expect("Executor Goal validated").version, authority.snapshot.task.as_ref().expect("Executor Task validated").version], "codex.approval.reply_intent", data)
            } else {
                store.audit(&reservation.session.scope, "codex.approval.reply_intent", data)
            }.map_err(|error| failure(ErrorKind::StateConflict, error.to_string()))?;
            // Fence the still-owned Session immediately before the native reply.
            reservation.version = store.put_session(&reservation.session, reservation.version).map_err(|error| failure(ErrorKind::StateConflict, error.to_string()))?;
        }
        native.send(preview).await?;
        let empty = {
            let mut evidence = evidence.lock().map_err(|_| failure(ErrorKind::StateFailure, "native approval state poisoned"))?;
            let pending = evidence.pending.as_mut().ok_or_else(|| failure(ErrorKind::SessionLost, "native approval ledger unavailable"))?;
            pending.reply(&reply.request.request_id, reply.request.decision)?;
            pending.is_empty()
        };
        if matches!(reply.request.decision, OperationDecision::Cancel) { return Ok(true); }
        if empty {
            reservation.session.state = SessionState::Running;
            reservation.publish(sender, status)?;
        }
        Ok(false)
    }.await;
    match validated {
        Ok(cancelled) => {
            let _ = reply.result.send(Ok(()));
            Ok(cancelled)
        }
        Err(error) => {
            let kind = error.kind;
            let message = error.to_string();
            let _ = reply.result.send(Err(failure(kind, message.clone())));
            Err(failure(kind, message))
        }
    }
}

async fn supervise(
    mut native: NativeServer,
    mut reservation: Reservation,
    sender: watch::Sender<SessionStatus>,
    mut stop: mpsc::Receiver<()>,
    evidence: Arc<Mutex<Evidence>>,
    mut replies: mpsc::Receiver<Reply>,
    identity: NativeTurn,
) {
    let NativeTurn {
        thread,
        turn,
        previous_turn,
        previous_cumulative,
        authority,
        request,
        binding,
        runtime_broker,
    } = identity;
    let mut status = sender.borrow().clone();
    let mut tracker = if previous_turn.is_some() {
        UsageTracker::resumed(thread.clone(), turn.clone(), previous_cumulative)
    } else {
        UsageTracker::new(thread.clone(), turn.clone())
    };
    let approval_authority = ApprovalAuthority {
        snapshot: authority,
        request,
        binding,
        runtime_broker,
    };
    let result = loop {
        let event = tokio::select! {
            biased;
            _ = stop.recv() => {
                let _ = native.rpc.call("turn/interrupt", json!({"threadId":thread,"turnId":turn})).await;
                break Ok(false);
            },
            Some(reply) = replies.recv() => {
                match answer_approval(&mut native.rpc, &mut reservation, &evidence, &approval_authority, &sender, &mut status, reply).await {
                    Ok(true) => break Ok(false),
                    Ok(false) => continue,
                    Err(error) => break Err(error),
                }
            },
            event = native.receive() => event,
        };
        if let Ok(Event::Request { id, method, params }) = &event {
            let pending = (|| {
                if reservation.session.role != SessionRole::Executor {
                    return Err(failure(
                        ErrorKind::UnsupportedCapability,
                        "decision-only native Session cannot request an operation",
                    ));
                }
                let observed = evidence
                    .lock()
                    .map_err(|_| {
                        failure(ErrorKind::StateFailure, "native approval state poisoned")
                    })?
                    .pending
                    .as_mut()
                    .ok_or_else(|| {
                        failure(ErrorKind::SessionLost, "native approval ledger unavailable")
                    })?
                    .insert(id.clone(), method.clone(), params.clone())?;
                reservation.store.lock().map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?
                    .audit(&reservation.session.scope, "codex.approval.requested", json!({"session_id":reservation.session.id,"native_uuid":reservation.session.native_ref,"native_turn":turn,"request":observed,"requires_human":!runtime_broker}))
                    .map_err(|error| failure(ErrorKind::StateFailure, error.to_string()))?;
                reservation.session.state = if runtime_broker {
                    SessionState::WaitingApproval
                } else {
                    SessionState::WaitingHuman
                };
                reservation.publish(&sender, &mut status)
            })();
            if let Err(error) = pending {
                // Explicit unsupported native requests receive a bounded decline,
                // rather than waiting indefinitely for a human-input API we lack.
                if error.kind == ErrorKind::UnsupportedCapability {
                    let _=reservation.store.lock().map_err(|_|failure(ErrorKind::StateFailure,"state store poisoned"))
                        .and_then(|mut store|store.audit(&reservation.session.scope,"codex.request.unsupported",json!({"session_id":reservation.session.id,"native_uuid":reservation.session.native_ref,"native_turn":turn,"request_id":id,"native_method":method,"params":params,"effect":"no_grant"})).map_err(|audit|failure(ErrorKind::StateFailure,audit.to_string())));
                    let _=native.rpc.send(json!({"id":id,"error":{"code":-32601,"message":"Native request unsupported by this runtime route"}})).await;
                }
                break Err(error);
            }
            continue;
        }
        match event.and_then(|event| {
            if resumed_usage(&event, &thread, previous_turn.as_deref(), &mut tracker)? {
                return Ok(None);
            }
            let outcome = session_event(
                event.clone(),
                &thread,
                &turn,
                &mut tracker,
                &mut status,
                reservation.session.role == SessionRole::Executor,
            )?;
            observe_approval_notification(
                &event,
                &evidence,
                &mut reservation,
                &sender,
                &mut status,
            )?;
            Ok(outcome)
        }) {
            Ok(Some(completed)) => break Ok(completed),
            Ok(None) => {
                if let Ok(mut evidence) = evidence.lock() {
                    evidence.counters = tracker.turn_counters();
                    evidence.cumulative = tracker.total.clone();
                }
                sender.send_replace(status.clone());
            }
            Err(error) => break Err(error),
        }
    };
    let stderr = native.stderr();
    status.stderr = stderr.bytes;
    status.stderr_truncated = stderr.truncated;
    let cleanup = native.shutdown().await;
    let completed = reservation.terminal(&mut status, &result, &cleanup);
    if let Ok(mut evidence) = evidence.lock() {
        evidence.completed = completed;
        evidence.counters = tracker.turn_counters();
        evidence.cumulative = tracker.total;
        evidence.pending = None;
    }
    match reservation.publish(&sender, &mut status) {
        Ok(()) => {
            reservation.armed = false;
        }
        Err(error) => {
            if let Ok(mut evidence) = evidence.lock() {
                evidence.completed = false;
            }
            status.session.state = SessionState::Lost;
            status.failure = Some(format!("native terminal persistence failed: {error}"));
        }
    }
    sender.send_replace(status);
}

fn observe_approval_notification(
    event: &Event,
    evidence: &Arc<Mutex<Evidence>>,
    reservation: &mut Reservation,
    sender: &watch::Sender<SessionStatus>,
    status: &mut SessionStatus,
) -> AdapterResult<()> {
    let Event::Notification { method, params } = event else {
        return Ok(());
    };
    let (retired, empty) = {
        let mut evidence = evidence
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "native approval state poisoned"))?;
        let Some(ledger) = evidence.pending.as_mut() else {
            return Ok(());
        };
        let retired = match method.as_str() {
            "item/started" => {
                ledger.observe_item(&params["item"])?;
                vec![]
            }
            "item/completed" => {
                ledger.retire_item(params["item"]["id"].as_str().ok_or_else(|| {
                    failure(
                        ErrorKind::ParseFailure,
                        "completed native item identity unavailable",
                    )
                })?)
            }
            "serverRequest/resolved" => {
                if params["threadId"] != reservation.session.native_ref.as_deref().unwrap_or("") {
                    return Err(failure(
                        ErrorKind::OwnershipMismatch,
                        "foreign native request resolution",
                    ));
                }
                let id: RpcId =
                    serde_json::from_value(params["requestId"].clone()).map_err(|_| {
                        failure(
                            ErrorKind::ParseFailure,
                            "native resolution request identity unavailable",
                        )
                    })?;
                ledger.retire(&id).into_iter().collect()
            }
            "turn/completed" => ledger.retire_turn(),
            _ => vec![],
        };
        (retired, ledger.is_empty())
    };
    if !retired.is_empty() {
        reservation.store.lock().map_err(|_|failure(ErrorKind::StateFailure,"state store poisoned"))?
            .audit(&reservation.session.scope,"codex.approval.retired",json!({"session_id":reservation.session.id,"native_uuid":reservation.session.native_ref,"native_method":method,"requests":retired.iter().map(|request|json!({"request_id":request.id,"operation_hash":request.operation_hash})).collect::<Vec<_>>(),"effect":"no_runtime_reply"}))
            .map_err(|error|failure(ErrorKind::StateFailure,error.to_string()))?;
        if empty
            && matches!(
                reservation.session.state,
                SessionState::WaitingHuman | SessionState::WaitingApproval
            )
        {
            reservation.session.state = SessionState::Running;
            reservation.publish(sender, status)?;
        }
    }
    Ok(())
}

/// Native resume can report the already completed, owned prior turn's counters.
/// Validate the baseline and discard that gauge; it is not new turn usage.
fn resumed_usage(
    event: &Event,
    thread: &str,
    previous_turn: Option<&str>,
    tracker: &mut UsageTracker,
) -> AdapterResult<bool> {
    let Event::Notification { method, params } = event else {
        return Ok(false);
    };
    if method != "thread/tokenUsage/updated"
        || previous_turn.is_none()
        || params["turnId"].as_str() != previous_turn
    {
        return Ok(false);
    }
    if params["threadId"] != thread {
        return Err(failure(
            ErrorKind::OwnershipMismatch,
            "foreign historical native usage thread",
        ));
    }
    let total = TokenCounters::from_native(&params["tokenUsage"]["total"])?;
    TokenCounters::from_native(&params["tokenUsage"]["last"])?;
    tracker.verify_previous_total(&total);
    Ok(true)
}

fn session_event(
    event: Event,
    thread: &str,
    turn: &str,
    tracker: &mut UsageTracker,
    status: &mut SessionStatus,
    execute: bool,
) -> AdapterResult<Option<bool>> {
    let Event::Notification { method, params } = event else {
        return Err(failure(
            ErrorKind::UnsupportedCapability,
            "decision-only native Session cannot authorize an operation or unsolicited response",
        ));
    };
    if params.get("threadId").is_some_and(|id| id != thread)
        || params.get("turnId").is_some_and(|id| id != turn)
    {
        return Err(failure(
            ErrorKind::OwnershipMismatch,
            format!("foreign native notification thread/turn ({method})"),
        ));
    }
    match method.as_str() {
        "thread/tokenUsage/updated" => {
            tracker.update(&params)?;
        }
        "item/started" | "item/completed" => {
            if params["threadId"] != thread || params["turnId"] != turn {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "native item lacks exact thread/turn identity",
                ));
            }
            match params["item"]["type"].as_str() {
                Some(
                    "agentMessage" | "userMessage" | "reasoning" | "plan" | "contextCompaction",
                ) => {}
                Some("commandExecution" | "fileChange") if execute => {}
                _ => {
                    return Err(failure(
                        ErrorKind::ProcessFailure,
                        "unexpected target operation in decision-only native Session",
                    ));
                }
            }
            if method == "item/completed" && params["item"]["type"] == "agentMessage" {
                let text = params["item"]["text"].as_str().ok_or_else(|| {
                    failure(ErrorKind::ParseFailure, "native assistant text unavailable")
                })?;
                status.stdout = text.as_bytes()[text.len().saturating_sub(OUTPUT_LIMIT)..].to_vec();
                status.stdout_truncated = text.len() > OUTPUT_LIMIT;
            }
        }
        "turn/completed" => {
            if params["threadId"] != thread || params["turn"]["id"] != turn {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "native completion lacks exact thread/turn identity",
                ));
            }
            if params["turn"]["status"] != "completed" {
                return Err(native_turn_error(&params["turn"]["error"]));
            }
            return Ok(Some(true));
        }
        "error" => {
            if params["threadId"] != thread || params["turnId"] != turn {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "native error lacks exact thread/turn identity",
                ));
            }
            if params["willRetry"] == true {
                return Ok(None);
            }
            return Err(native_turn_error(&params["error"]));
        }
        "thread/started" => {
            if params["thread"]["id"] != thread
                || params["thread"]["cwd"].as_str() != status.session.worktree.to_str()
            {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "native thread announcement is not its owning workspace",
                ));
            }
        }
        "turn/started" => {
            if params["threadId"] != thread || params["turn"]["id"] != turn {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "native turn announcement has foreign identity",
                ));
            }
        }
        _ => {}
    }
    Ok(None)
}

fn native_turn_error(error: &Value) -> crate::adapter::AdapterError {
    let info = &error["codexErrorInfo"];
    let unauthorized = info == "unauthorized"
        || [
            "httpConnectionFailed",
            "responseStreamConnectionFailed",
            "responseStreamDisconnected",
            "responseTooManyFailedAttempts",
        ]
        .iter()
        .any(|kind| matches!(info[*kind]["httpStatusCode"].as_u64(), Some(401 | 403)));
    failure(
        if unauthorized {
            ErrorKind::AuthenticationUnavailable
        } else {
            ErrorKind::ProcessFailure
        },
        if unauthorized {
            "native Codex authentication rejected"
        } else {
            "native turn failed, interrupted or reported an error"
        },
    )
}

#[cfg(test)]
fn decision_event(
    event: Event,
    thread: &str,
    turn: &str,
    tracker: &mut UsageTracker,
    status: &mut SessionStatus,
) -> AdapterResult<Option<bool>> {
    session_event(event, thread, turn, tracker, status, false)
}

#[cfg(test)]
mod tests {
    use super::super::ownership::tests::Fixture;
    use super::*;
    use crate::domain::{ProjectId, Scope};
    use futures_util::StreamExt;

    #[tokio::test]
    async fn any_uncertain_owned_group_retains_lost_reservation_and_scoped_diagnostic() {
        use std::{os::unix::process::ExitStatusExt, sync::atomic::Ordering};
        for uncertain_git in [false, true] {
            let mut fixture = ApprovalFixture::new(true).await;
            fixture.reservation.session.pid = Some(42);
            let cleanup = if uncertain_git {
                fixture
                    .reservation
                    .ownership
                    .group()
                    .store(true, Ordering::SeqCst);
                Ok(std::process::ExitStatus::from_raw(0))
            } else {
                Err(failure(
                    ErrorKind::ProcessFailure,
                    "injected unverified shutdown",
                ))
            };
            assert!(
                !fixture
                    .reservation
                    .terminal(&mut fixture.status, &Ok(true), &cleanup)
            );
            fixture
                .reservation
                .publish(&fixture.sender, &mut fixture.status)
                .unwrap();
            assert_eq!(fixture.status.session.state, SessionState::Lost);
            assert_eq!(fixture.status.session.pid, Some(42));
            assert!(crate::git::executor_reserved(&fixture.status.session));
            let store = fixture.reservation.store.lock().unwrap();
            assert_eq!(
                store
                    .session(fixture.status.session.id)
                    .unwrap()
                    .unwrap()
                    .0
                    .state,
                SessionState::Lost
            );
            let diagnostic = store
                .events(&fixture.status.session.scope, 0, 1000)
                .unwrap()
                .into_iter()
                .find(|event| event.kind == "codex.cleanup.unverified")
                .unwrap();
            assert_eq!(diagnostic.scope, fixture.status.session.scope);
            assert_eq!(diagnostic.data["native_pid"], 42);
            assert_eq!(
                diagnostic.data["other_owned_group_uncertain"],
                uncertain_git
            );
            assert_eq!(fixture.status.exit_code, None);
        }
    }
    #[tokio::test]
    async fn server_exit_zero_does_not_fabricate_a_stopped_or_failed_turn_success() {
        use std::os::unix::process::ExitStatusExt;
        for outcome in [
            Ok(false),
            Err(failure(ErrorKind::ProcessFailure, "native failed")),
        ] {
            let mut fixture = ApprovalFixture::new(true).await;
            fixture.reservation.session.pid = Some(42);
            assert!(!fixture.reservation.terminal(
                &mut fixture.status,
                &outcome,
                &Ok(std::process::ExitStatus::from_raw(0))
            ));
            fixture
                .reservation
                .publish(&fixture.sender, &mut fixture.status)
                .unwrap();
            assert!(matches!(
                fixture.status.session.state,
                SessionState::Stopped | SessionState::Failed
            ));
            assert_eq!(fixture.status.session.pid, None);
            assert_eq!(fixture.status.exit_code, None);
        }
    }

    struct ApprovalFixture {
        _owned: Fixture,
        reservation: Reservation,
        evidence: Arc<Mutex<Evidence>>,
        authority: ApprovalAuthority,
        sender: watch::Sender<SessionStatus>,
        status: SessionStatus,
    }
    impl ApprovalFixture {
        async fn new(runtime_broker: bool) -> Self {
            let owned = Fixture::new(true);
            let snapshot = ScopeSnapshot::capture(&owned.store, &owned.request, "codex").unwrap();
            let binding = snapshot
                .verify_git(&owned.request, &mut ProcessOwnership::default())
                .await
                .unwrap();
            let mut status = status();
            status.session.scope = owned.request.scope.clone();
            status.session.worktree = owned.request.worktree.clone();
            status.session.role = SessionRole::Executor;
            status.session.native_ref = Some("thread".into());
            status.session.state = SessionState::Starting;
            let mut reservation = Reservation {
                store: owned.store.clone(),
                session: status.session.clone(),
                version: 0,
                ownership: ProcessOwnership::default(),
                armed: false,
                resume_publication: None,
                inference_started: false,
            };
            reservation.persist().unwrap();
            reservation.session.state = if runtime_broker {
                SessionState::WaitingApproval
            } else {
                SessionState::WaitingHuman
            };
            reservation.persist().unwrap();
            status.session = reservation.session.clone();
            let (sender, _) = watch::channel(status.clone());
            let mut pending = ApprovalLedger::new("thread".into(), "turn".into())
                .for_workspace(owned.request.worktree.clone());
            pending.insert(RpcId::Number(1), "item/commandExecution/requestApproval".into(), json!({"threadId":"thread","turnId":"turn","itemId":"item","command":"exact native operation","cwd":owned.request.worktree})).unwrap();
            let evidence = Arc::new(Mutex::new(Evidence {
                turn: Some("turn".into()),
                pending: Some(pending),
                ..Evidence::default()
            }));
            let authority = ApprovalAuthority {
                snapshot,
                request: owned.request.clone(),
                binding,
                runtime_broker,
            };
            Self {
                _owned: owned,
                reservation,
                evidence,
                authority,
                sender,
                status,
            }
        }
        async fn answer(&mut self, rpc: &mut NativeRpc, mut reply: Reply) -> AdapterResult<bool> {
            if reply.request.operation_hash == "fixture-current-operation"
                && let Ok(request) = self
                    .evidence
                    .lock()
                    .unwrap()
                    .pending
                    .as_ref()
                    .unwrap()
                    .request(&reply.request.request_id)
            {
                reply.request.operation_hash = request.operation_hash;
            }
            answer_approval(
                rpc,
                &mut self.reservation,
                &self.evidence,
                &self.authority,
                &self.sender,
                &mut self.status,
                reply,
            )
            .await
        }
        fn intents(&self) -> usize {
            self.reservation
                .store
                .lock()
                .unwrap()
                .events(&self.reservation.session.scope, 0, 1000)
                .unwrap()
                .into_iter()
                .filter(|event| event.kind == "codex.approval.reply_intent")
                .count()
        }
        fn terminal_adapter(&mut self) -> (Arc<CodexAdapter>, SessionRef) {
            self.reservation.session.state = SessionState::Exited;
            self.reservation.session.pid = None;
            self.reservation
                .publish(&self.sender, &mut self.status)
                .unwrap();
            self.evidence.lock().unwrap().completed = true;
            let adapter = Arc::new(
                CodexAdapter::new(
                    "codex".into(),
                    "/definitely-missing-codex".into(),
                    self.reservation.store.clone(),
                )
                .unwrap(),
            );
            let (stop, _) = mpsc::channel(1);
            let (reply, _) = mpsc::channel(16);
            adapter.registry().unwrap().insert(
                self.status.session.id,
                Entry {
                    status: self.sender.subscribe(),
                    publisher: self.sender.clone(),
                    transition: Arc::new(AtomicBool::new(false)),
                    stop,
                    reply,
                    evidence: self.evidence.clone(),
                    request: self.authority.request.clone(),
                    schema: None,
                },
            );
            (adapter, SessionRef::from(&self.status.session))
        }
    }
    #[tokio::test]
    async fn claimed_transition_excludes_release_resume_and_eviction_without_poisoning_registry() {
        let mut fixture = ApprovalFixture::new(true).await;
        let (adapter, reference) = fixture.terminal_adapter();
        let (claim, _, _) = adapter.claim(&reference).unwrap();
        let release_adapter = adapter.clone();
        let release_ref = reference.clone();
        let release = tokio::spawn(async move { release_adapter.release(release_ref) });
        assert_eq!(
            release.await.unwrap().unwrap_err().kind,
            ErrorKind::StateConflict
        );
        assert_eq!(
            adapter.resume(reference.clone()).await.unwrap_err().kind,
            ErrorKind::StateConflict
        );
        assert_eq!(
            adapter.current(&reference).unwrap().session.state,
            SessionState::Exited
        );
        evict_one_terminal(&mut adapter.registry().unwrap());
        assert!(
            adapter
                .registry()
                .unwrap()
                .get(&reference.id)
                .unwrap()
                .transition
                .load(Ordering::SeqCst)
        );
        drop(claim);
        adapter.release(reference.clone()).unwrap();
        assert_eq!(
            adapter.resume(reference).await.unwrap_err().kind,
            ErrorKind::SessionLost
        );
        assert!(adapter.registry().unwrap().is_empty());
    }
    #[tokio::test]
    async fn failed_pre_inference_resume_restores_the_owned_terminal_record_and_watch() {
        let mut fixture = ApprovalFixture::new(true).await;
        let (adapter, reference) = fixture.terminal_adapter();
        let original = adapter.current(&reference).unwrap();
        for _ in 0..2 {
            assert_eq!(
                adapter.resume(reference.clone()).await.unwrap_err().kind,
                ErrorKind::ExecutableMissing
            );
            let restored = adapter.current(&reference).unwrap();
            assert_eq!(
                serde_json::to_value(restored.session).unwrap(),
                serde_json::to_value(&original.session).unwrap()
            );
            assert!(fixture.evidence.lock().unwrap().completed);
        }
        adapter.release(reference).unwrap();
    }
    #[tokio::test]
    async fn checkpoint_rejects_stale_or_regressed_input_without_destroying_resume_state() {
        let mut fixture = ApprovalFixture::new(true).await;
        let (adapter, reference) = fixture.terminal_adapter();
        let original = adapter.current(&reference).unwrap();
        let mut input = fixture.authority.request.input.clone();
        assert_eq!(
            adapter
                .checkpoint(reference.clone(), input.clone())
                .await
                .unwrap_err()
                .kind,
            ErrorKind::StateConflict
        );
        input.version += 1;
        input.revision = "0".repeat(40);
        assert_eq!(
            adapter
                .checkpoint(reference.clone(), input)
                .await
                .unwrap_err()
                .kind,
            ErrorKind::StateConflict
        );
        assert_eq!(
            serde_json::to_value(adapter.current(&reference).unwrap().session).unwrap(),
            serde_json::to_value(&original.session).unwrap()
        );
        let mut input = fixture.authority.request.input.clone();
        input.version += 1;
        adapter
            .checkpoint(reference.clone(), input.clone())
            .await
            .unwrap();
        assert_eq!(
            adapter
                .registry()
                .unwrap()
                .get(&reference.id)
                .unwrap()
                .request
                .input
                .version,
            input.version
        );
        assert_eq!(
            adapter.resume(reference.clone()).await.unwrap_err().kind,
            ErrorKind::ExecutableMissing
        );
        assert_eq!(
            adapter.current(&reference).unwrap().session.state,
            SessionState::Exited
        );
        adapter.release(reference).unwrap();
    }
    #[tokio::test]
    async fn abandoned_resume_after_inference_or_uncertain_cleanup_publishes_its_real_outcome() {
        for uncertain in [false, true] {
            let mut fixture = ApprovalFixture::new(true).await;
            let (adapter, reference) = fixture.terminal_adapter();
            let previous = adapter.current(&reference).unwrap();
            let version = fixture.reservation.version;
            let mut reservation = Reservation {
                store: fixture.reservation.store.clone(),
                session: previous.session.clone(),
                version,
                ownership: ProcessOwnership::default(),
                armed: false,
                resume_publication: Some(ResumePublication {
                    previous: previous.clone(),
                    sender: fixture.sender.clone(),
                    evidence: fixture.evidence.clone(),
                }),
                inference_started: !uncertain,
            };
            reservation.session.state = SessionState::Starting;
            reservation.session.pid = Some(42);
            reservation.persist().unwrap();
            reservation.armed = true;
            if uncertain {
                reservation.ownership.group().store(true, Ordering::SeqCst);
            }
            drop(reservation);
            let current = adapter.current(&reference).unwrap();
            assert_eq!(
                current.session.state,
                if uncertain {
                    SessionState::Lost
                } else {
                    SessionState::Failed
                }
            );
            assert_eq!(current.session.pid, if uncertain { Some(42) } else { None });
            assert!(!fixture.evidence.lock().unwrap().completed);
            adapter.release(reference).unwrap();
        }
    }
    fn reply(
        decision: OperationDecision,
        turn: &str,
    ) -> (Reply, oneshot::Receiver<AdapterResult<()>>) {
        let (result, receiver) = oneshot::channel();
        (
            Reply {
                request: DecisionRequest {
                    native_turn: turn.into(),
                    request_id: RpcId::Number(1),
                    decision,
                    operation_hash: "fixture-current-operation".into(),
                },
                result,
            },
            receiver,
        )
    }
    async fn rpc_peer() -> (
        NativeRpc,
        mpsc::Receiver<Value>,
        tokio::task::JoinHandle<()>,
    ) {
        let (client, server) = tokio::net::UnixStream::pair().unwrap();
        let (sender, receiver) = mpsc::channel(16);
        let peer = tokio::spawn(async move {
            let mut socket = tokio_tungstenite::accept_async(server).await.unwrap();
            while let Some(Ok(message)) = socket.next().await {
                if let Ok(text) = message.to_text() {
                    let value = serde_json::from_str(text).unwrap();
                    if sender.send(value).await.is_err() {
                        break;
                    }
                }
            }
        });
        (NativeRpc::connect(client).await.unwrap(), receiver, peer)
    }

    #[tokio::test]
    async fn native_reply_is_correlated_one_time_and_audited_before_real_wire_delivery() {
        let mut fixture = ApprovalFixture::new(true).await;
        let (mut rpc, mut wire, peer) = rpc_peer().await;
        let (answer, result) = reply(OperationDecision::Approve, "turn");
        assert!(!fixture.answer(&mut rpc, answer).await.unwrap());
        result.await.unwrap().unwrap();
        assert_eq!(
            wire.recv().await.unwrap(),
            json!({"id":1,"result":{"decision":"accept"}})
        );
        assert_eq!(fixture.intents(), 1);
        assert!(
            fixture
                .evidence
                .lock()
                .unwrap()
                .pending
                .as_ref()
                .unwrap()
                .is_empty()
        );
        assert_eq!(fixture.status.session.state, SessionState::Running);
        let (duplicate, result) = reply(OperationDecision::Approve, "turn");
        assert!(!fixture.answer(&mut rpc, duplicate).await.unwrap());
        assert_eq!(
            result.await.unwrap().unwrap_err().kind,
            ErrorKind::OwnershipMismatch
        );
        assert_eq!(fixture.intents(), 1);
        assert!(wire.try_recv().is_err());
        drop(rpc);
        peer.abort();
    }

    #[tokio::test]
    async fn native_human_route_can_be_denied_but_cannot_be_silently_granted_by_broker() {
        let mut fixture = ApprovalFixture::new(false).await;
        let (mut rpc, mut wire, peer) = rpc_peer().await;
        let (answer, result) = reply(OperationDecision::Approve, "turn");
        assert!(!fixture.answer(&mut rpc, answer).await.unwrap());
        assert_eq!(
            result.await.unwrap().unwrap_err().kind,
            ErrorKind::UnsupportedCapability
        );
        assert_eq!(fixture.intents(), 0);
        assert_eq!(fixture.status.session.state, SessionState::WaitingHuman);
        assert!(wire.try_recv().is_err());
        let (denial, result) = reply(OperationDecision::Deny, "turn");
        assert!(!fixture.answer(&mut rpc, denial).await.unwrap());
        result.await.unwrap().unwrap();
        assert_eq!(
            wire.recv().await.unwrap(),
            json!({"id":1,"result":{"decision":"decline"}})
        );
        assert_eq!(fixture.intents(), 1);
        drop(rpc);
        peer.abort();
    }

    #[tokio::test]
    async fn cancelled_or_previous_turn_replies_cannot_claim_a_pending_native_request() {
        let mut fixture = ApprovalFixture::new(true).await;
        let (mut rpc, mut wire, peer) = rpc_peer().await;
        let (cancelled, result) = reply(OperationDecision::Approve, "turn");
        drop(result);
        assert!(!fixture.answer(&mut rpc, cancelled).await.unwrap());
        let (foreign, result) = reply(OperationDecision::Approve, "previous-turn");
        assert!(!fixture.answer(&mut rpc, foreign).await.unwrap());
        assert_eq!(
            result.await.unwrap().unwrap_err().kind,
            ErrorKind::OwnershipMismatch
        );
        assert_eq!(fixture.intents(), 0);
        assert!(
            !fixture
                .evidence
                .lock()
                .unwrap()
                .pending
                .as_ref()
                .unwrap()
                .is_empty()
        );
        assert!(wire.try_recv().is_err());
        let (cancel, result) = reply(OperationDecision::Cancel, "turn");
        assert!(fixture.answer(&mut rpc, cancel).await.unwrap());
        result.await.unwrap().unwrap();
        assert_eq!(
            wire.recv().await.unwrap(),
            json!({"id":1,"result":{"decision":"cancel"}})
        );
        drop(rpc);
        peer.abort();
    }

    #[tokio::test]
    async fn failed_intent_audit_prevents_delivery_of_native_acceptance() {
        let mut fixture = ApprovalFixture::new(true).await;
        let database = fixture
            .authority
            .snapshot
            .project
            .root
            .parent()
            .unwrap()
            .join("state.sqlite3");
        rusqlite::Connection::open(database).unwrap().execute_batch("CREATE TRIGGER reject_native_intent BEFORE INSERT ON audit WHEN NEW.kind='codex.approval.reply_intent' BEGIN SELECT RAISE(ABORT,'injected audit failure'); END;").unwrap();
        let (mut rpc, mut wire, peer) = rpc_peer().await;
        let (answer, result) = reply(OperationDecision::Approve, "turn");
        assert!(fixture.answer(&mut rpc, answer).await.is_err());
        assert!(result.await.unwrap().is_err());
        assert_eq!(fixture.intents(), 0);
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(100), wire.recv())
                .await
                .is_err()
        );
        drop(rpc);
        peer.abort();
    }

    #[tokio::test]
    async fn changed_durable_owner_or_paused_goal_prevents_native_operation_grant() {
        for paused in [false, true] {
            let mut fixture = ApprovalFixture::new(true).await;
            {
                let mut store = fixture.reservation.store.lock().unwrap();
                if paused {
                    let mut goal = fixture.authority.snapshot.goal.clone().unwrap();
                    goal.state = crate::domain::GoalState::Paused;
                    store.put_goal(&mut goal).unwrap();
                } else {
                    let mut session = fixture.reservation.session.clone();
                    session.state = SessionState::Stopped;
                    store
                        .put_session(&session, fixture.reservation.version)
                        .unwrap();
                }
            }
            let (mut rpc, mut wire, peer) = rpc_peer().await;
            let (answer, result) = reply(OperationDecision::Approve, "turn");
            let outcome = fixture.answer(&mut rpc, answer).await;
            if paused {
                assert!(!outcome.unwrap());
            } else {
                assert!(outcome.is_err());
            }
            assert!(result.await.unwrap().is_err());
            assert_eq!(fixture.intents(), 0);
            assert!(
                !fixture
                    .evidence
                    .lock()
                    .unwrap()
                    .pending
                    .as_ref()
                    .unwrap()
                    .is_empty()
            );
            assert!(wire.try_recv().is_err());
            drop(rpc);
            peer.abort();
        }
    }
    #[tokio::test]
    async fn unrelated_project_registration_does_not_invalidate_an_owned_native_approval() {
        let mut fixture = ApprovalFixture::new(true).await;
        let foreign = Fixture::new(false);
        let mut project = foreign.request.project.clone();
        project.version = 0;
        fixture
            .reservation
            .store
            .lock()
            .unwrap()
            .put_project(&mut project)
            .unwrap();
        let (mut rpc, mut wire, peer) = rpc_peer().await;
        let (answer, result) = reply(OperationDecision::Approve, "turn");
        assert!(!fixture.answer(&mut rpc, answer).await.unwrap());
        result.await.unwrap().unwrap();
        assert_eq!(
            wire.recv().await.unwrap(),
            json!({"id":1,"result":{"decision":"accept"}})
        );
        assert_eq!(fixture.intents(), 1);
        drop(rpc);
        peer.abort();
    }
    #[tokio::test]
    async fn failed_grant_preflight_keeps_the_native_request_pending_and_deniable() {
        let mut fixture = ApprovalFixture::new(true).await;
        fixture.authority.request.input.revision = "0".repeat(40);
        let (mut rpc, mut wire, peer) = rpc_peer().await;
        let (answer, result) = reply(OperationDecision::Approve, "turn");
        assert!(!fixture.answer(&mut rpc, answer).await.unwrap());
        assert_eq!(
            result.await.unwrap().unwrap_err().kind,
            ErrorKind::StateConflict
        );
        assert_eq!(fixture.intents(), 0);
        assert!(wire.try_recv().is_err());
        assert_eq!(fixture.status.session.state, SessionState::WaitingApproval);
        assert!(
            !fixture
                .evidence
                .lock()
                .unwrap()
                .pending
                .as_ref()
                .unwrap()
                .is_empty()
        );
        let (denial, result) = reply(OperationDecision::Deny, "turn");
        assert!(!fixture.answer(&mut rpc, denial).await.unwrap());
        result.await.unwrap().unwrap();
        assert_eq!(
            wire.recv().await.unwrap(),
            json!({"id":1,"result":{"decision":"decline"}})
        );
        assert_eq!(fixture.intents(), 1);
        drop(rpc);
        peer.abort();
    }
    #[tokio::test]
    async fn native_resolution_retires_the_pending_operation_without_sending_a_runtime_reply() {
        let mut fixture = ApprovalFixture::new(true).await;
        let resolved = Event::Notification {
            method: "serverRequest/resolved".into(),
            params: json!({"threadId":"thread","requestId":1}),
        };
        observe_approval_notification(
            &resolved,
            &fixture.evidence,
            &mut fixture.reservation,
            &fixture.sender,
            &mut fixture.status,
        )
        .unwrap();
        assert_eq!(fixture.status.session.state, SessionState::Running);
        let audits = fixture
            .reservation
            .store
            .lock()
            .unwrap()
            .events(&fixture.status.session.scope, 0, 1000)
            .unwrap();
        assert!(
            audits
                .iter()
                .any(|audit| audit.kind == "codex.approval.retired"
                    && audit.data["effect"] == "no_runtime_reply")
        );
        let (mut rpc, mut wire, peer) = rpc_peer().await;
        let (answer, result) = reply(OperationDecision::Approve, "turn");
        assert!(!fixture.answer(&mut rpc, answer).await.unwrap());
        assert_eq!(
            result.await.unwrap().unwrap_err().kind,
            ErrorKind::OwnershipMismatch
        );
        assert_eq!(fixture.intents(), 0);
        assert!(wire.try_recv().is_err());
        drop(rpc);
        peer.abort();
    }
    #[tokio::test]
    async fn wrong_operation_digest_cannot_claim_the_native_request() {
        let mut fixture = ApprovalFixture::new(true).await;
        let (mut rpc, mut wire, peer) = rpc_peer().await;
        let (mut answer, result) = reply(OperationDecision::Approve, "turn");
        answer.request.operation_hash = "another-operation-digest".into();
        assert!(!fixture.answer(&mut rpc, answer).await.unwrap());
        assert_eq!(
            result.await.unwrap().unwrap_err().kind,
            ErrorKind::OwnershipMismatch
        );
        assert_eq!(fixture.intents(), 0);
        assert!(wire.try_recv().is_err());
        assert!(
            !fixture
                .evidence
                .lock()
                .unwrap()
                .pending
                .as_ref()
                .unwrap()
                .is_empty()
        );
        drop(rpc);
        peer.abort();
    }
    #[tokio::test]
    async fn native_patch_grant_rejects_symlink_and_hard_link_targets_outside_the_task() {
        use std::os::unix::fs::symlink;
        for hard_link in [false, true] {
            let mut fixture = ApprovalFixture::new(true).await;
            let outside = fixture
                .authority
                .snapshot
                .project
                .root
                .join("outside-task-file");
            std::fs::write(&outside, "isolated fixture sentinel").unwrap();
            let alias = fixture.authority.request.worktree.join("alias");
            if hard_link {
                std::fs::hard_link(&outside, &alias).unwrap();
            } else {
                symlink(&outside, &alias).unwrap();
            }
            let mut ledger = ApprovalLedger::new("thread".into(), "turn".into())
                .for_workspace(fixture.authority.request.worktree.clone());
            ledger.observe_item(&json!({"id":"item","type":"fileChange","changes":[{"path":"alias","diff":"proposed patch","kind":{"type":"update","move_path":null}}]})).unwrap();
            ledger
                .insert(
                    RpcId::Number(1),
                    "item/fileChange/requestApproval".into(),
                    json!({"threadId":"thread","turnId":"turn","itemId":"item"}),
                )
                .unwrap();
            fixture.evidence.lock().unwrap().pending = Some(ledger);
            let (mut rpc, mut wire, peer) = rpc_peer().await;
            let (answer, result) = reply(OperationDecision::Approve, "turn");
            assert!(!fixture.answer(&mut rpc, answer).await.unwrap());
            assert_eq!(
                result.await.unwrap().unwrap_err().kind,
                ErrorKind::OwnershipMismatch
            );
            assert_eq!(fixture.intents(), 0);
            assert!(wire.try_recv().is_err());
            assert_eq!(
                std::fs::read_to_string(&outside).unwrap(),
                "isolated fixture sentinel"
            );
            let (denial, result) = reply(OperationDecision::Deny, "turn");
            assert!(!fixture.answer(&mut rpc, denial).await.unwrap());
            result.await.unwrap().unwrap();
            assert_eq!(
                wire.recv().await.unwrap(),
                json!({"id":1,"result":{"decision":"decline"}})
            );
            drop(rpc);
            peer.abort();
        }
    }
    fn status() -> SessionStatus {
        SessionStatus {
            session: Session {
                id: SessionId::new(),
                scope: Scope::project(ProjectId::new()),
                agent: "codex".into(),
                provider: "codex".into(),
                role: SessionRole::Consultant,
                native_ref: None,
                pid: None,
                worktree: PathBuf::from("/fixture"),
                state: SessionState::Running,
                model: None,
                effort: None,
                recovery: json!({}),
                started_at: now_ms(),
            },
            exit_code: None,
            stdout: vec![],
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
            failure: None,
        }
    }
    #[test]
    fn native_completion_is_not_process_exit_and_foreign_or_operation_events_fail_closed() {
        let mut status = status();
        let mut tracker = UsageTracker::new("thread".into(), "turn".into());
        let event = |thread: &str, turn: &str, kind: &str| Event::Notification {
            method: "item/completed".into(),
            params: json!({"threadId":thread,"turnId":turn,"item":{"type":kind,"text":"exact final answer"}}),
        };
        assert_eq!(
            decision_event(
                event("foreign", "turn", "agentMessage"),
                "thread",
                "turn",
                &mut tracker,
                &mut status
            )
            .unwrap_err()
            .kind,
            ErrorKind::OwnershipMismatch
        );
        assert_eq!(
            decision_event(
                event("thread", "foreign", "agentMessage"),
                "thread",
                "turn",
                &mut tracker,
                &mut status
            )
            .unwrap_err()
            .kind,
            ErrorKind::OwnershipMismatch
        );
        assert_eq!(
            decision_event(
                event("thread", "turn", "commandExecution"),
                "thread",
                "turn",
                &mut tracker,
                &mut status
            )
            .unwrap_err()
            .kind,
            ErrorKind::ProcessFailure
        );
        assert_eq!(
            decision_event(
                event("thread", "turn", "agentMessage"),
                "thread",
                "turn",
                &mut tracker,
                &mut status
            )
            .unwrap(),
            None
        );
        assert_eq!(status.stdout, b"exact final answer");
        let completed = Event::Notification {
            method: "turn/completed".into(),
            params: json!({"threadId":"thread","turn":{"id":"turn","status":"completed"}}),
        };
        assert_eq!(
            decision_event(completed, "thread", "turn", &mut tracker, &mut status).unwrap(),
            Some(true)
        );
        assert_eq!(status.session.state, SessionState::Running);
        assert_eq!(status.exit_code, None);
    }
    #[test]
    fn assistant_retention_is_bounded_and_failed_native_turns_are_explicit() {
        let mut status = status();
        let mut tracker = UsageTracker::new("thread".into(), "turn".into());
        decision_event(Event::Notification { method:"item/completed".into(),params:json!({"threadId":"thread","turnId":"turn","item":{"type":"agentMessage","text":"x".repeat(OUTPUT_LIMIT+10)}}) },"thread","turn",&mut tracker,&mut status).unwrap();
        assert_eq!(status.stdout.len(), OUTPUT_LIMIT);
        assert!(status.stdout_truncated);
        assert_eq!(
            decision_event(
                Event::Notification {
                    method: "turn/completed".into(),
                    params: json!({"threadId":"thread","turn":{"id":"turn","status":"failed"}})
                },
                "thread",
                "turn",
                &mut tracker,
                &mut status
            )
            .unwrap_err()
            .kind,
            ErrorKind::ProcessFailure
        );
    }
    #[test]
    fn authentication_failure_is_typed_without_exposing_native_error_payloads() {
        for info in [
            json!("unauthorized"),
            json!({"httpConnectionFailed":{"httpStatusCode":401}}),
            json!({"responseStreamConnectionFailed":{"httpStatusCode":403}}),
            json!({"responseStreamDisconnected":{"httpStatusCode":401}}),
            json!({"responseTooManyFailedAttempts":{"httpStatusCode":403}}),
        ] {
            let error = native_turn_error(
                &json!({"message":"secret-native-payload","codexErrorInfo":info}),
            );
            assert_eq!(error.kind, ErrorKind::AuthenticationUnavailable);
            assert!(!error.to_string().contains("secret-native-payload"));
        }
        assert_eq!(
            native_turn_error(
                &json!({"codexErrorInfo":{"httpConnectionFailed":{"httpStatusCode":500}}})
            )
            .kind,
            ErrorKind::ProcessFailure
        );
        let mut status = status();
        let mut tracker = UsageTracker::new("thread".into(), "turn".into());
        let event = Event::Notification {
            method: "turn/completed".into(),
            params: json!({"threadId":"foreign","turn":{"id":"turn","status":"failed","error":{"codexErrorInfo":"unauthorized"}}}),
        };
        assert_eq!(
            decision_event(event, "thread", "turn", &mut tracker, &mut status)
                .unwrap_err()
                .kind,
            ErrorKind::OwnershipMismatch
        );
    }
    #[test]
    fn retrying_native_errors_do_not_end_the_turn_and_embedded_start_ids_are_checked() {
        let mut status = status();
        let mut tracker = UsageTracker::new("thread".into(), "turn".into());
        let event = Event::Notification {
            method: "error".into(),
            params: json!({"threadId":"thread","turnId":"turn","willRetry":true,"error":{"codexErrorInfo":"serverOverloaded"}}),
        };
        assert_eq!(
            decision_event(event, "thread", "turn", &mut tracker, &mut status).unwrap(),
            None
        );
        for event in [
            Event::Notification {
                method: "thread/started".into(),
                params: json!({"thread":{"id":"foreign","cwd":status.session.worktree}}),
            },
            Event::Notification {
                method: "turn/started".into(),
                params: json!({"threadId":"thread","turn":{"id":"foreign"}}),
            },
            Event::Notification {
                method: "error".into(),
                params: json!({"willRetry":false,"error":{"codexErrorInfo":"unauthorized"}}),
            },
        ] {
            assert_eq!(
                decision_event(event, "thread", "turn", &mut tracker, &mut status)
                    .unwrap_err()
                    .kind,
                ErrorKind::OwnershipMismatch
            );
        }
    }
    #[test]
    fn resume_accepts_only_validated_owned_previous_usage_without_recounting_it() {
        let mut tracker = UsageTracker::resumed("own".into(), "current".into(), None);
        let event = |thread: &str, turn: &str| Event::Notification {
            method: "thread/tokenUsage/updated".into(),
            params: json!({"threadId":thread,"turnId":turn,"tokenUsage":{"total":{"inputTokens":10},"last":{"inputTokens":10}}}),
        };
        assert!(
            resumed_usage(
                &event("own", "previous"),
                "own",
                Some("previous"),
                &mut tracker
            )
            .unwrap()
        );
        assert!(
            !resumed_usage(
                &event("own", "current"),
                "own",
                Some("previous"),
                &mut tracker
            )
            .unwrap()
        );
        assert!(!resumed_usage(&event("own", "previous"), "own", None, &mut tracker).unwrap());
        assert_eq!(
            resumed_usage(
                &event("foreign", "previous"),
                "own",
                Some("previous"),
                &mut tracker
            )
            .unwrap_err()
            .kind,
            ErrorKind::OwnershipMismatch
        );
        let mut forged = event("own", "previous");
        if let Event::Notification { params, .. } = &mut forged {
            params["tokenUsage"]["last"]["inputTokens"] = json!(-1);
        }
        assert_eq!(
            resumed_usage(&forged, "own", Some("previous"), &mut tracker)
                .unwrap_err()
                .kind,
            ErrorKind::ParseFailure
        );
    }
    #[test]
    fn historical_replay_that_exceeds_shutdown_gauge_cannot_inflate_resumed_usage() {
        let baseline = TokenCounters::from_native(&json!({"inputTokens":10})).unwrap();
        let mut tracker = UsageTracker::resumed("own".into(), "current".into(), Some(baseline));
        tracker.update(&json!({"threadId":"own","turnId":"current","tokenUsage":{"total":{"inputTokens":30},"last":{"inputTokens":5}}})).unwrap();
        assert_eq!(tracker.turn_counters().unwrap().input, Some(20));
        let prior = Event::Notification {
            method: "thread/tokenUsage/updated".into(),
            params: json!({"threadId":"own","turnId":"previous","tokenUsage":{"total":{"inputTokens":15},"last":{"inputTokens":5}}}),
        };
        assert!(resumed_usage(&prior, "own", Some("previous"), &mut tracker).unwrap());
        assert_eq!(tracker.turn_counters().unwrap().input, None);
        tracker.update(&json!({"threadId":"own","turnId":"current","tokenUsage":{"total":{"inputTokens":35},"last":{"inputTokens":5}}})).unwrap();
        assert_eq!(tracker.turn_counters().unwrap().input, None);
    }
}
