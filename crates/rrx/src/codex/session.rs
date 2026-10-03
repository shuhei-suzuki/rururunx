//! Scoped native session supervision. Workflow verdicts remain caller-owned.
use serde_json::{Value, json};
use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, watch};

use super::{
    ownership::{ProcessOwnership, ScopeSnapshot, filesystem},
    policy::{DecisionPolicy, native_environment},
    protocol::{Event, TokenCounters, UsageTracker, failure},
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

pub struct CodexAdapter {
    agent: String,
    executable: PathBuf,
    store: SharedStore,
    sessions: Mutex<HashMap<SessionId, Entry>>,
}
struct Entry {
    status: watch::Receiver<SessionStatus>,
    stop: mpsc::Sender<()>,
    evidence: Arc<Mutex<Evidence>>,
    request: LaunchRequest,
    schema: Option<Value>,
}
type OwnedReference = (
    watch::Receiver<SessionStatus>,
    mpsc::Sender<()>,
    Arc<Mutex<Evidence>>,
);
#[derive(Default)]
struct Evidence {
    completed: bool,
    turn: Option<String>,
    counters: Option<TokenCounters>,
}
struct NativeTurn {
    thread: String,
    turn: String,
    previous_turn: Option<String>,
}
struct Reservation {
    store: SharedStore,
    session: Session,
    version: u64,
    ownership: ProcessOwnership,
    armed: bool,
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
        Ok(())
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if self.armed {
            self.session.state = if self.ownership.uncertain() {
                SessionState::Lost
            } else {
                SessionState::Failed
            };
            let _ = self.persist();
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
        })
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
        ))
    }
    fn current(&self, reference: &SessionRef) -> AdapterResult<SessionStatus> {
        let (receiver, _, _) = self.reference(reference)?;
        let status = receiver.borrow().clone();
        let store = self
            .store
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?;
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
        let mut reservation = Reservation {
            store: self.store.clone(),
            session,
            version: expected_version,
            ownership: ProcessOwnership::default(),
            armed: false,
        };
        reservation.persist()?;
        reservation.armed = true;
        snapshot
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
        let environment = native_environment(
            std::env::vars_os(),
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
            if request.role == SessionRole::Executor {
                DecisionPolicy::for_executor(&config["config"])
            } else {
                DecisionPolicy::from_native(&config["config"])
            }
        }
        .await;
        discovery.shutdown().await?;
        let policy = discovered?;
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
                .verify_git(&request, &mut reservation.ownership)
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
        let (sender, receiver) = watch::channel(status);
        let (stop, stopped) = mpsc::channel(1);
        let evidence = Arc::new(Mutex::new(Evidence {
            turn: Some(turn.clone()),
            ..Evidence::default()
        }));
        {
            let mut sessions = self.registry()?;
            if sessions.len() >= RETAINED_TERMINALS
                && let Some(id) = sessions
                    .iter()
                    .filter(|(_, entry)| entry.status.borrow().terminal())
                    .min_by_key(|(_, entry)| entry.status.borrow().session.started_at)
                    .map(|(id, _)| *id)
            {
                sessions.remove(&id);
            }
            sessions.insert(
                session.id,
                Entry {
                    status: receiver,
                    stop,
                    evidence: evidence.clone(),
                    request,
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
            NativeTurn {
                thread,
                turn,
                previous_turn: resume.and_then(|session| {
                    session.recovery["native_turn"].as_str().map(str::to_owned)
                }),
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
            let (mut receiver, stop, _) = self.reference(&session)?;
            if !receiver.borrow().terminal() {
                stop.try_send(()).map_err(|_| {
                    failure(
                        ErrorKind::StateConflict,
                        "native stop already requested or supervisor unavailable",
                    )
                })?;
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
            let previous = self.current(&session)?.session;
            let (request, schema) = {
                let sessions = self.registry()?;
                let entry = sessions
                    .get(&session.id)
                    .expect("owned reference validated");
                (entry.request.clone(), entry.schema.clone())
            };
            self.launch(request, schema, Some(previous)).await
        })
    }
    fn release(&self, session: SessionRef) -> AdapterResult<()> {
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
        self.reference(&session).map(|(receiver, _, _)| receiver)
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
            let (_, _, evidence) = self.reference(&session)?;
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
                cache_metadata: json!({"native_uuid":status.session.native_ref,"native_turn":evidence.turn,"cache_write_input_tokens":counters.cache_write_input,"reasoning_output_tokens":counters.reasoning_output,"total_tokens":counters.total}),
                missing_reason: Some(
                    if evidence.counters.is_some() {
                        "native Codex does not expose monetary cost"
                    } else {
                        "native token notification not observed; monetary cost unavailable"
                    }
                    .into(),
                ),
            })
        })
    }
    fn checkpoint(&self, session: SessionRef, input: PreparedInput) -> AdapterFuture<'_, ()> {
        Box::pin(async move {
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
            let mut sessions = self.registry()?;
            let entry = sessions.get_mut(&session.id).ok_or_else(|| {
                failure(
                    ErrorKind::SessionLost,
                    "native checkpoint owner disappeared",
                )
            })?;
            let mut request = entry.request.clone();
            request.input = input;
            ScopeSnapshot::capture(&self.store, &request, &self.agent)?;
            entry.request = request;
            Ok(())
        })
    }
}

async fn supervise(
    mut native: NativeServer,
    mut reservation: Reservation,
    sender: watch::Sender<SessionStatus>,
    mut stop: mpsc::Receiver<()>,
    evidence: Arc<Mutex<Evidence>>,
    identity: NativeTurn,
) {
    let NativeTurn {
        thread,
        turn,
        previous_turn,
    } = identity;
    let mut status = sender.borrow().clone();
    let mut tracker = UsageTracker::new(thread.clone(), turn.clone());
    let result = loop {
        let event = tokio::select! {
            _ = stop.recv() => {
                let _ = native.rpc.call("turn/interrupt", json!({"threadId":thread,"turnId":turn})).await;
                break Ok(false);
            },
            event = native.receive() => event,
        };
        match event.and_then(|event| {
            if resumed_usage(&event, &thread, previous_turn.as_deref())? {
                return Ok(None);
            }
            session_event(
                event,
                &thread,
                &turn,
                &mut tracker,
                &mut status,
                reservation.session.role == SessionRole::Executor,
            )
        }) {
            Ok(Some(completed)) => break Ok(completed),
            Ok(None) => {
                if let Ok(mut evidence) = evidence.lock() {
                    evidence.counters = tracker.last.clone();
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
    let completed = matches!(result, Ok(true)) && cleanup.is_ok();
    reservation.session.state = match (&result, &cleanup) {
        (_, Err(_)) => SessionState::Lost,
        (Ok(true), _) => SessionState::Exited,
        (Ok(false), _) => SessionState::Stopped,
        (Err(_), _) => SessionState::Failed,
    };
    reservation.session.pid = None;
    status.exit_code = cleanup.as_ref().ok().and_then(|exit| exit.code());
    status.failure = cleanup
        .as_ref()
        .err()
        .or(result.as_ref().err())
        .map(|error| error.to_string());
    match reservation.persist() {
        Ok(()) => {
            reservation.armed = false;
            status.session = reservation.session.clone();
            if let Ok(mut evidence) = evidence.lock() {
                evidence.completed = completed;
                evidence.counters = tracker.last;
            }
        }
        Err(error) => {
            status.session.state = SessionState::Lost;
            status.failure = Some(format!("native terminal persistence failed: {error}"));
        }
    }
    sender.send_replace(status);
}

/// Native resume can report the already completed, owned prior turn's counters.
/// Validate and discard that gauge; it is neither current input nor new usage.
fn resumed_usage(event: &Event, thread: &str, previous_turn: Option<&str>) -> AdapterResult<bool> {
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
    TokenCounters::from_native(&params["tokenUsage"]["total"])?;
    TokenCounters::from_native(&params["tokenUsage"]["last"])?;
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
                Some("agentMessage" | "userMessage" | "reasoning" | "plan") => {}
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
            if params["threadId"] != thread
                || params["turn"]["id"] != turn
                || params["turn"]["status"] != "completed"
            {
                return Err(failure(
                    ErrorKind::ProcessFailure,
                    "native turn failed, interrupted or changed identity",
                ));
            }
            return Ok(Some(true));
        }
        "error" => {
            return Err(failure(
                ErrorKind::ProcessFailure,
                "native turn reported an error",
            ));
        }
        _ => {}
    }
    Ok(None)
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
    use super::*;
    use crate::domain::{ProjectId, Scope};
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
    fn resume_accepts_only_validated_owned_previous_usage_without_recounting_it() {
        let event = |thread: &str, turn: &str| Event::Notification {
            method: "thread/tokenUsage/updated".into(),
            params: json!({"threadId":thread,"turnId":turn,"tokenUsage":{"total":{"inputTokens":10},"last":{"inputTokens":10}}}),
        };
        assert!(resumed_usage(&event("own", "previous"), "own", Some("previous")).unwrap());
        assert!(!resumed_usage(&event("own", "current"), "own", Some("previous")).unwrap());
        assert!(!resumed_usage(&event("own", "previous"), "own", None).unwrap());
        assert_eq!(
            resumed_usage(&event("foreign", "previous"), "own", Some("previous"))
                .unwrap_err()
                .kind,
            ErrorKind::OwnershipMismatch
        );
        let mut forged = event("own", "previous");
        if let Event::Notification { params, .. } = &mut forged {
            params["tokenUsage"]["last"]["inputTokens"] = json!(-1);
        }
        assert_eq!(
            resumed_usage(&forged, "own", Some("previous"))
                .unwrap_err()
                .kind,
            ErrorKind::ParseFailure
        );
    }
}
