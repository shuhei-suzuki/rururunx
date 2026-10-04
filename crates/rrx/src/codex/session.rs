//! Scoped native session supervision. Workflow verdicts remain caller-owned.
#[cfg(test)]
use super::attempt::{TestGates, TestPoint};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
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
    attempt::{CallerGuard, Control, Outcome, Phase, TaskGuard},
    ownership::{ProcessOwnership, ScopeSnapshot, filesystem},
    policy::{DecisionPolicy, native_environment, provider_environment, verify_auth_readiness},
    preparation::{Admission, Cause},
    protocol::{
        ApprovalLedger, Event, NativeRpc, OperationDecision, RpcId, TokenCounters, UsageTracker,
        failure, operation_paths,
    },
    transport::{NativeServer, failure_after_cleanup, result_after_cleanup},
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
const INTERRUPT_DRAIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

fn empty_status(session: Session) -> SessionStatus {
    SessionStatus {
        session,
        exit_code: None,
        stdout: vec![],
        stderr: vec![],
        stdout_truncated: false,
        stderr_truncated: false,
        failure: None,
    }
}

fn fresh_session(request: &LaunchRequest, agent: &str) -> Session {
    Session {
        id: SessionId::new(),
        scope: request.scope.clone(),
        agent: agent.into(),
        provider: "codex".into(),
        role: request.role,
        native_ref: None,
        pid: None,
        worktree: request.worktree.clone(),
        state: SessionState::Starting,
        model: request.model.clone(),
        effort: request.effort.clone(),
        recovery: json!({"project_root":request.project.root,"input_revision":request.input.revision,"input_version":request.input.version,"source_versions":request.input.source_versions,"input_bytes":request.input.payload.len(),"reconnect_supported":false}),
        started_at: now_ms(),
    }
}

fn pin_starting_input(
    session: &mut Session,
    request: &LaunchRequest,
    previous: Option<&Session>,
) -> AdapterResult<()> {
    session.state = SessionState::Starting;
    session.pid = None;
    // A fresh continuation is a new attempt: pin its source metadata while
    // Starting, before admission. Later observations must not rebind it.
    session.recovery["input_revision"] = json!(request.input.revision);
    session.recovery["input_version"] = json!(request.input.version);
    session.recovery["input_bytes"] = json!(request.input.payload.len());
    session.recovery["input_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(request.input.payload.as_bytes())
    ));
    session.recovery["source_versions"] = json!(request.input.source_versions);
    if let Some(previous) = previous {
        // Durable proof for exact pre-dispatch rollback; never permission to
        // change source metadata after the consumed intent was published.
        session.recovery["pre_dispatch_restore_sha256"] = json!(format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(previous)
                    .map_err(|error| { failure(ErrorKind::StateFailure, error.to_string()) })?
            )
        ));
    }
    if let Some(recovery) = session.recovery.as_object_mut() {
        // Historical identity cannot describe the newly pinned input. Keep it
        // explicitly historical until the new native turn is acknowledged.
        if let Some(previous_turn) = recovery.remove("native_turn") {
            recovery.insert("previous_native_turn".into(), previous_turn);
        }
        recovery.remove("dispatch_intent");
        recovery.remove("native_dispatch_unobserved");
    }
    Ok(())
}

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
    sessions: Arc<Mutex<HashMap<SessionId, Entry>>>,
    runtime_broker: bool,
    #[cfg(test)]
    gates: Arc<TestGates>,
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
    control: Arc<Control>,
}
struct TransitionClaim(Arc<AtomicBool>);
impl Drop for TransitionClaim {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}
struct RegisteredTransition {
    sessions: Arc<Mutex<HashMap<SessionId, Entry>>>,
    id: SessionId,
    control: Arc<Control>,
    previous_control: Option<Arc<Control>>,
    active: bool,
}
impl RegisteredTransition {
    fn transfer(&mut self, entry: &mut Entry) -> AdapterResult<()> {
        if !Arc::ptr_eq(&entry.control, &self.control) {
            return Err(failure(
                ErrorKind::StateConflict,
                "native attempt owner advanced",
            ));
        }
        self.control.supervised();
        entry.transition.store(false, Ordering::SeqCst);
        self.active = false;
        Ok(())
    }
}
impl Drop for RegisteredTransition {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        // Nested native/Reservation owners finish first; this last transition
        // owner supplies uncertainty if a panic/drop skipped normal completion.
        let phase = self.control.subscribe().borrow().clone();
        if !matches!(phase, Phase::Finished(_)) {
            let error = failure(
                ErrorKind::SessionLost,
                if std::thread::panicking() {
                    "owned native preparation task panicked"
                } else {
                    "owned native preparation task abandoned its final publication"
                },
            );
            let cause = self.control.preparation.cause(&error);
            let publication_result = self
                .control
                .published()
                .filter(|status| status.session.state == SessionState::Lost)
                .ok_or(Cause::Failed(error.kind, error.message));
            // Only explicit normal cleanup/publication can establish restore.
            // An abandoned frame never classifies its old snapshot as restored.
            self.control.finished(Outcome::Lost {
                cause,
                publication_result,
            });
        }
        if let Ok(mut sessions) = self.sessions.lock()
            && let Some(entry) = sessions.get_mut(&self.id)
            && Arc::ptr_eq(&entry.control, &self.control)
        {
            let restored = matches!(
                &*self.control.subscribe().borrow(),
                Phase::Finished(outcome) if matches!(outcome.as_ref(),Outcome::RestoredBeforeAdmission { .. } | Outcome::CheckpointCommitted { .. })
            );
            if let Some(previous) = &self.previous_control
                && restored
            {
                entry.control = previous.clone();
                entry.stop = previous.stop.clone();
                entry.transition.store(false, Ordering::SeqCst);
            } else if self.control.published().is_none() {
                sessions.remove(&self.id);
            } else {
                entry.transition.store(false, Ordering::SeqCst);
            }
        }
    }
}
struct Registered {
    transition: RegisteredTransition,
    request: LaunchRequest,
    schema: Option<Value>,
    previous: Option<SessionStatus>,
    publisher: watch::Sender<SessionStatus>,
    stopped: mpsc::Receiver<()>,
}
struct Supervision {
    native: NativeServer,
    reservation: Reservation,
    sender: watch::Sender<SessionStatus>,
    stopped: mpsc::Receiver<()>,
    evidence: Arc<Mutex<Evidence>>,
    replies: mpsc::Receiver<Reply>,
    identity: NativeTurn,
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
    published_attempt: bool,
    ownership: ProcessOwnership,
    armed: bool,
    resume_publication: Option<ResumePublication>,
    inference_started: bool,
    attempt: Arc<Control>,
    #[cfg(test)]
    gates: Arc<TestGates>,
}
impl Reservation {
    fn finish_preparation_error(
        &mut self,
        error: crate::adapter::AdapterError,
    ) -> crate::adapter::AdapterError {
        let selected = self.attempt.preparation.failed(error);
        let cleanup_failure = self.attempt.preparation.cleanup_failure();
        let cause = self.attempt.preparation.cause(&selected);
        let previous = self
            .resume_publication
            .as_ref()
            .filter(|publication| publication.previous.terminal())
            .map(|publication| publication.previous.clone());
        let unstarted = !self.published_attempt;
        let uncertain = self.ownership.uncertain() || self.inference_started;
        let uncertainty_reason = match (self.ownership.uncertain(), self.inference_started) {
            (true, true) => Some(
                "owned process cleanup is unverified; consumed native input outcome is unobserved",
            ),
            (true, false) => Some("owned process cleanup is unverified"),
            (false, true) => Some("consumed native input outcome is unobserved"),
            (false, false) => None,
        };
        let uncertainty_detail = uncertainty_reason.map(|reason| match &cleanup_failure {
            Some(detail) => format!("{reason}; cleanup: {detail}"),
            None => reason.to_owned(),
        });
        let mut status = if !uncertain {
            previous.clone().unwrap_or_else(|| {
                let mut status = empty_status(self.session.clone());
                status.session.state = SessionState::Failed;
                status.session.pid = None;
                status.failure = Some(cause.error().to_string());
                status
            })
        } else {
            let mut status = self
                .attempt
                .published()
                .unwrap_or_else(|| empty_status(self.session.clone()));
            status.session = self.session.clone();
            status.session.state = SessionState::Lost;
            if !self.ownership.uncertain() {
                status.session.pid = None;
            }
            if self.inference_started {
                status.session.recovery["native_dispatch_unobserved"] = json!(true);
            }
            status.failure = Some(format!(
                "{}; first cause: {selected}",
                uncertainty_detail.as_deref().unwrap()
            ));
            if let Some(publication) = &self.resume_publication
                && let Ok(mut evidence) = publication.evidence.lock()
            {
                *evidence = Evidence::default();
            }
            status
        };
        self.session = status.session.clone();
        let publication = if unstarted && previous.is_some() && !uncertain {
            // No Starting was committed by this attempt. Its exact historical
            // watch/Session needs no rollback write or additional CAS failure.
            Ok(())
        } else if unstarted {
            Err(failure(
                selected.kind,
                "initial native preparation was not published",
            ))
        } else if let Some(sender) = self
            .resume_publication
            .as_ref()
            .map(|publication| publication.sender.clone())
        {
            self.publish(&sender, &mut status)
        } else {
            self.persist_unchecked()
        };
        // A failed exact restore/publication remains explicitly unpublished. Do
        // not let Drop attempt another write or overwrite a second writer.
        self.armed = false;
        let outcome = match publication {
            Ok(()) if uncertain => Outcome::Lost {
                cause: cause.clone(),
                publication_result: Ok(status),
            },
            Ok(()) if previous.is_some() => Outcome::RestoredBeforeAdmission {
                cause: cause.clone(),
                snapshot: status,
            },
            Ok(()) => Outcome::FailedBeforeAdmission {
                cause: cause.clone(),
                snapshot: status,
            },
            Err(error) if uncertain => Outcome::Lost {
                cause: cause.clone(),
                publication_result: Err(Cause::Failed(error.kind, error.message)),
            },
            Err(error) if previous.is_some() => Outcome::RestoreUnpublished {
                cause: cause.clone(),
                error: Cause::Failed(error.kind, error.message),
            },
            Err(error) => Outcome::FreshUnpublished {
                cause: cause.clone(),
                error: Cause::Failed(error.kind, error.message),
            },
        };
        let secondary: Vec<_> = self
            .attempt
            .preparation
            .later_failures()
            .iter()
            .map(|kind| format!("{kind:?}"))
            .collect();
        if self.published_attempt || uncertain {
            let _ = self.store.lock().map_err(|_|()).and_then(|mut store| {
            store.audit(&self.session.scope,"codex.preparation.finished", json!({
                "session_id":self.session.id,
                "cause":match &cause {Cause::Cancelled=>"cancelled",Cause::Failed(_,_)=>"failed"},
                "failure_kind":match &cause {Cause::Failed(kind,_)=>Some(format!("{kind:?}")),_=>None},
                "later_failure_kinds":secondary,
                "uncertainty_reason":uncertainty_reason,
                "cleanup_detail_retained":cleanup_failure.is_some(),
                "input_consumed":self.inference_started,
                "snapshot_published":outcome.snapshot().is_some(),
            })).map_err(|_|())
        });
        }
        let returned = if uncertain {
            let publication = outcome
                .error()
                .map(|error| format!("; publication: {error}"))
                .unwrap_or_default();
            failure(
                ErrorKind::SessionLost,
                format!(
                    "{}; first cause: {selected}{publication}",
                    uncertainty_detail.as_deref().unwrap()
                ),
            )
        } else {
            outcome.error().unwrap_or(selected)
        };
        self.attempt.finished(outcome);
        returned
    }
    async fn dispatch(
        &mut self,
        rpc: &mut NativeRpc,
        authority: &ScopeSnapshot,
        request: &LaunchRequest,
        thread: &str,
        schema: Option<&Value>,
    ) -> AdapterResult<Value> {
        let mut parameters =
            json!({"threadId":thread,"input":[{"type":"text","text":request.input.payload}]});
        if let Some(schema) = schema {
            parameters["outputSchema"] = schema.clone();
        }
        if let Some(effort) = &request.effort {
            parameters["effort"] = json!(effort);
        }
        let prepared = rpc.prepare_call("turn/start", parameters)?;
        self.admit_dispatch(authority, request)?;
        // Any cancelled/failed write can already have started native inference.
        self.inference_started = true;
        self.attempt.awaiting_ack();
        rpc.dispatch_call(prepared).await
    }
    fn admit_dispatch(
        &mut self,
        authority: &ScopeSnapshot,
        request: &LaunchRequest,
    ) -> AdapterResult<()> {
        let previous_intent = self.session.recovery.get("dispatch_intent").cloned();
        self.session.recovery["dispatch_intent"] = json!({
            "id":uuid::Uuid::new_v4(),"origin":"runtime","consumed":true,
            "input_version":request.input.version,"input_bytes":request.input.payload.len(),
            "input_sha256":format!("{:x}",Sha256::digest(request.input.payload.as_bytes())),
            "authority_versions":authority.versions(),
        });
        let attempt = self.attempt.clone();
        let admitted = attempt.preparation.consume(|| {
            #[cfg(test)]
            attempt.cas_entered.store(true, Ordering::SeqCst);
            self.store
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))
                .and_then(|mut store| {
                    store
                        .put_session_if_current(
                            &self.session,
                            self.version,
                            authority.versions(),
                            &authority.lock_versions(),
                        )
                        .map_err(super::ownership::state_error)
                })
        });
        match admitted {
            Ok(version) => {
                self.version = version;
                Ok(())
            }
            Err(error) => {
                if let Some(previous) = previous_intent {
                    self.session.recovery["dispatch_intent"] = previous;
                } else if let Some(recovery) = self.session.recovery.as_object_mut() {
                    recovery.remove("dispatch_intent");
                }
                Err(error)
            }
        }
    }
    fn persist(&mut self) -> AdapterResult<()> {
        let attempt = self.attempt.clone();
        attempt.preparation.publish(|| self.persist_unchecked())
    }
    fn persist_unchecked(&mut self) -> AdapterResult<()> {
        let mut store = self
            .store
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?;
        self.version = store
            .put_session(&self.session, self.version)
            .map_err(super::ownership::state_error)?;
        self.published_attempt = true;
        // Preparation carries no new turn output. Its exact prior snapshot is
        // kept privately for rollback, rather than relabelled as current output.
        let status = empty_status(self.session.clone());
        if let Some(publication) = &self.resume_publication {
            publication.sender.send_replace(status.clone());
        }
        self.attempt.publish(status);
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
            .map_err(super::ownership::state_error)?;
        status.session = self.session.clone();
        sender.send_replace(status.clone());
        self.attempt.publish(status.clone());
        Ok(())
    }
    fn terminal(
        &mut self,
        status: &mut SessionStatus,
        result: &AdapterResult<bool>,
        cleanup: &AdapterResult<std::process::ExitStatus>,
        native_terminal_observed: bool,
    ) -> bool {
        let group_uncertain = cleanup.is_err() || self.ownership.uncertain();
        // Interrupt acknowledgement and owned process death do not establish
        // the outcome of a consumed native input or its target operations.
        let dispatch_uncertain = self.inference_started && !native_terminal_observed;
        let uncertain = group_uncertain || dispatch_uncertain;
        self.session.state = if uncertain {
            SessionState::Lost
        } else {
            match result {
                Ok(true) => SessionState::Exited,
                Ok(false) => SessionState::Stopped,
                Err(_) => SessionState::Failed,
            }
        };
        if !group_uncertain {
            self.session.pid = None;
        }
        if dispatch_uncertain {
            self.session.recovery["native_dispatch_unobserved"] = json!(true);
        }
        // The server process's exit status is not a native model-turn outcome.
        status.exit_code = None;
        status.failure = match (result.as_ref().err(), cleanup.as_ref().err()) {
            (Some(primary), Some(cleanup)) => Some(format!(
                "owned cleanup unverified: {cleanup}; original failure: {primary}"
            )),
            (Some(error), None) | (None, Some(error)) => Some(error.to_string()),
            (None, None) => None,
        };
        if uncertain {
            status.failure.get_or_insert_with(|| {
                if group_uncertain {
                    "owned process cleanup is unverified"
                } else {
                    "consumed native input outcome is unobserved"
                }
                .into()
            });
            let diagnostic = json!({
                "session_id":self.session.id,"native_uuid":self.session.native_ref,
                "native_pid":self.session.pid,"native_group_cleanup_confirmed":cleanup.is_ok(),
                "other_owned_group_uncertain":self.ownership.uncertain(),
                "native_terminal_observed":native_terminal_observed,
                "native_dispatch_unobserved":dispatch_uncertain,
                "dispatch_intent":self.session.recovery["dispatch_intent"],
                "state_intent":"Lost","failure":status.failure,
            });
            let audit = self.store.lock().map_err(|_| ()).and_then(|mut store| {
                store
                    .audit(
                        &self.session.scope,
                        if group_uncertain {
                            "codex.cleanup.unverified"
                        } else {
                            "codex.dispatch.unobserved"
                        },
                        diagnostic,
                    )
                    .map_err(|_| ())
            });
            if audit.is_err() {
                status.failure = Some(format!(
                    "owned cleanup diagnostic publication is unverified; original failure: {}",
                    status
                        .failure
                        .as_deref()
                        .unwrap_or("owned process cleanup is unverified")
                ));
            }
        }
        matches!(result, Ok(true)) && !uncertain
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if self.armed {
            let uncertain = self.ownership.uncertain();
            {
                // Only an explicit normal cleanup/restore path can prove a
                // pre-admission restore. Panic/runtime-drop remains uncertain.
                self.session.state = SessionState::Lost;
                if !uncertain {
                    self.session.pid = None;
                }
                if self.inference_started {
                    self.session.recovery["native_dispatch_unobserved"] = json!(true);
                    let diagnostic = json!({"session_id":self.session.id,"native_uuid":self.session.native_ref,
                        "native_pid":self.session.pid,"owned_groups_cleanup_confirmed":!uncertain,
                        "dispatch_intent":self.session.recovery["dispatch_intent"],
                        "reason":"native turn acknowledgement or owned supervisor was not established","state_intent":"Lost"});
                    let audit = self.store.lock().map_err(|_| ()).and_then(|mut store| {
                        store
                            .audit(&self.session.scope, "codex.dispatch.unobserved", diagnostic)
                            .map_err(|_| ())
                    });
                    if audit.is_err() {
                        self.session.recovery["diagnostic_publication_unverified"] = json!(true);
                    }
                }
            }
            if let Some(publication) = &self.resume_publication {
                let mut status = publication.previous.clone();
                if self.session.state == SessionState::Lost {
                    if let Ok(mut evidence) = publication.evidence.lock() {
                        *evidence = Evidence::default();
                    }
                    status.stdout.clear();
                    status.stderr.clear();
                    status.exit_code = None;
                    status.failure =
                        Some("owned native attempt ended without verified completion".into());
                }
                let sender = publication.sender.clone();
                let _ = self.publish(&sender, &mut status);
            } else {
                let _ = self.persist_unchecked();
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
            sessions: Arc::new(Mutex::new(HashMap::new())),
            runtime_broker: false,
            #[cfg(test)]
            gates: Arc::new(TestGates::default()),
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
    fn owned_handles(&self) -> Self {
        Self {
            agent: self.agent.clone(),
            executable: self.executable.clone(),
            store: self.store.clone(),
            sessions: self.sessions.clone(),
            runtime_broker: self.runtime_broker,
            #[cfg(test)]
            gates: self.gates.clone(),
        }
    }
    fn register_fresh(
        &self,
        request: LaunchRequest,
        schema: Option<Value>,
    ) -> AdapterResult<Registered> {
        let session = fresh_session(&request, &self.agent);
        let (publisher, status) = watch::channel(empty_status(session.clone()));
        let (control, stopped) = Control::new(None);
        let (reply, _) = mpsc::channel(16);
        let mut sessions = self.registry()?;
        if sessions.len() >= RETAINED_TERMINALS {
            evict_one_terminal(&mut sessions);
        }
        sessions.insert(
            session.id,
            Entry {
                status,
                publisher: publisher.clone(),
                transition: Arc::new(AtomicBool::new(true)),
                stop: control.stop.clone(),
                reply,
                evidence: Arc::new(Mutex::new(Evidence::default())),
                request: request.clone(),
                schema: schema.clone(),
                control: control.clone(),
            },
        );
        Ok(Registered {
            transition: RegisteredTransition {
                sessions: self.sessions.clone(),
                id: session.id,
                control,
                previous_control: None,
                active: true,
            },
            request,
            schema,
            previous: None,
            publisher,
            stopped,
        })
    }
    fn register_existing(&self, reference: &SessionRef) -> AdapterResult<Registered> {
        let mut sessions = self.registry()?;
        let entry = sessions.get_mut(&reference.id).ok_or_else(|| {
            failure(
                ErrorKind::SessionLost,
                "native preparation owner unavailable",
            )
        })?;
        let previous = entry.status.borrow().clone();
        if previous.session.scope != reference.scope {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "foreign native preparation scope",
            ));
        }
        if !previous.terminal() || previous.session.state == SessionState::Lost {
            return Err(failure(
                ErrorKind::StateConflict,
                "native preparation requires a confirmed terminal Session",
            ));
        }
        if !matches!(*entry.control.subscribe().borrow(), Phase::Finished(_)) {
            return Err(failure(
                ErrorKind::StateConflict,
                "native predecessor has not finished its final publication",
            ));
        }
        // Registry -> Store, without current/reference reentry. No write or await.
        let persisted = self
            .store
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?
            .session(reference.id)
            .map_err(super::ownership::state_error)?
            .ok_or_else(|| {
                failure(
                    ErrorKind::SessionLost,
                    "native preparation record unavailable",
                )
            })?
            .0;
        if serde_json::to_value(&persisted).ok() != serde_json::to_value(&previous.session).ok() {
            return Err(failure(
                ErrorKind::StateConflict,
                "native preparation owner changed",
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
        let (control, stopped) = Control::new(Some(previous.clone()));
        let previous_control = std::mem::replace(&mut entry.control, control.clone());
        entry.stop = control.stop.clone();
        Ok(Registered {
            transition: RegisteredTransition {
                sessions: self.sessions.clone(),
                id: reference.id,
                control,
                previous_control: Some(previous_control),
                active: true,
            },
            request: entry.request.clone(),
            schema: entry.schema.clone(),
            previous: Some(previous),
            publisher: entry.publisher.clone(),
            stopped,
        })
    }
    async fn launch(
        &self,
        request: LaunchRequest,
        schema: Option<Value>,
        resume: Option<Session>,
    ) -> AdapterResult<Session> {
        let registered = if let Some(previous) = resume {
            self.register_existing(&SessionRef::from(&previous))?
        } else {
            self.register_fresh(request, schema)?
        };
        self.spawn_launch(registered).await
    }
    async fn spawn_launch(&self, registered: Registered) -> AdapterResult<Session> {
        let control = registered.transition.control.clone();
        let task_guard = TaskGuard(control.clone());
        let handles = self.owned_handles();
        let (result, receiver) = oneshot::channel();
        let mut guard = CallerGuard::new(control.clone());
        control.spawn(async move {
            let _task = task_guard;
            match handles.prepare_launch(registered).await {
                Ok(supervision) => {
                    let session = supervision.reservation.session.clone();
                    let _ = result.send(Ok(session));
                    supervise(
                        supervision.native,
                        supervision.reservation,
                        supervision.sender,
                        supervision.stopped,
                        supervision.evidence,
                        supervision.replies,
                        supervision.identity,
                    )
                    .await;
                }
                Err(error) => {
                    let _ = result.send(Err(error));
                }
            }
        })?;
        let result = receiver.await.map_err(|_| {
            failure(
                ErrorKind::SessionLost,
                "owned native preparation task ended without caller result",
            )
        });
        guard.disarm();
        result?
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
    /// Bind telemetry/completion to one registry owner and persisted turn while
    /// excluding replacement by resume/release. No provider or caller JSON is
    /// sufficient without this supervisor's private journal.
    fn observe_owned<T>(
        &self,
        reference: &SessionRef,
        observe: impl FnOnce(&SessionStatus, &Evidence) -> AdapterResult<T>,
    ) -> AdapterResult<T> {
        let sessions = self.registry()?;
        let entry = sessions.get(&reference.id).ok_or_else(|| {
            failure(
                ErrorKind::SessionLost,
                "native observation owner unavailable",
            )
        })?;
        if entry.transition.load(Ordering::SeqCst) {
            return Err(failure(
                ErrorKind::StateConflict,
                "native observation overlaps a transition",
            ));
        }
        let store = self
            .store
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?;
        let status = entry.status.borrow().clone();
        if status.session.scope != reference.scope {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "foreign native observation",
            ));
        }
        let persisted = store
            .session(reference.id)
            .map_err(|error| failure(ErrorKind::StateFailure, error.to_string()))?
            .ok_or_else(|| {
                failure(
                    ErrorKind::SessionLost,
                    "native observation record unavailable",
                )
            })?
            .0;
        let evidence = entry
            .evidence
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "native journal poisoned"))?;
        if serde_json::to_value(&persisted).ok() != serde_json::to_value(&status.session).ok()
            || status.session.recovery["native_turn"].as_str() != evidence.turn.as_deref()
        {
            return Err(failure(
                ErrorKind::StateConflict,
                "native observation belongs to a different persisted attempt",
            ));
        }
        observe(&status, &evidence)
    }
    async fn prepare_checkpoint(
        &self,
        registered: Registered,
        input: PreparedInput,
    ) -> AdapterResult<()> {
        let session = SessionRef {
            id: registered.transition.id,
            scope: registered.request.scope.clone(),
        };
        let control = registered.transition.control.clone();
        #[cfg(test)]
        self.gates.wait(TestPoint::BeforeStarting).await;
        let mut request = registered.request.clone();
        let context = (|| -> AdapterResult<_> {
            control.preparation.check()?;
            let status = registered.previous.clone().ok_or_else(|| {
                failure(
                    ErrorKind::StateConflict,
                    "checkpoint has no prior terminal snapshot",
                )
            })?;
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
            // An explicit fresh checkpoint may refresh mutable Project metadata,
            // but may never rebind the owned repository/worktree identity.
            let project = self
                .store
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?
                .project(request.project.id)
                .map_err(|error| failure(ErrorKind::StateFailure, error.to_string()))?
                .ok_or_else(|| failure(ErrorKind::SessionLost, "checkpoint Project disappeared"))?;
            if project.root != request.project.root
                || project.repository_identity != request.project.repository_identity
                || project.base_branch != request.project.base_branch
                || project.worktree_root != request.project.worktree_root
            {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "native checkpoint cannot rebind its Project repository",
                ));
            }
            request.project = project;
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
            if serde_json::to_value(&persisted).ok() != serde_json::to_value(&status.session).ok() {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "native checkpoint owner changed before reservation",
                ));
            }

            let evidence = self
                .registry()?
                .get(&session.id)
                .ok_or_else(|| failure(ErrorKind::SessionLost, "checkpoint owner disappeared"))?
                .evidence
                .clone();
            let publication = ResumePublication {
                previous: status.clone(),
                sender: registered.publisher.clone(),
                evidence,
            };
            let mut reservation = Reservation {
                store: self.store.clone(),
                session: status.session.clone(),
                version,
                published_attempt: false,
                ownership: ProcessOwnership::default(),
                armed: false,
                resume_publication: Some(publication),
                inference_started: false,
                attempt: control.clone(),
                #[cfg(test)]
                gates: self.gates.clone(),
            };
            pin_starting_input(&mut reservation.session, &request, Some(&status.session))?;
            Ok((status, snapshot, reservation))
        })();
        let (previous, snapshot, mut reservation) = match context {
            Ok(context) => context,
            Err(error) => {
                let error = control.preparation.failed(error);
                let cause = control.preparation.cause(&error);
                if let Some(snapshot) = registered.previous.clone() {
                    control.finished(Outcome::RestoredBeforeAdmission { cause, snapshot });
                } else {
                    control.finished(Outcome::FreshUnpublished {
                        cause,
                        error: Cause::Failed(error.kind, error.message.clone()),
                    });
                }
                return Err(error);
            }
        };
        let validated = async {
            #[cfg(test)]
            self.gates.wait(TestPoint::BeforeInitialPersist).await;
            reservation.persist()?;
            reservation.armed = true;
            snapshot
                .verify_git_preparing(&request, &mut reservation.ownership, &control.preparation)
                .await?;
            snapshot.recheck(&self.store, &request, &self.agent)?;
            #[cfg(test)]
            self.gates.wait(TestPoint::BeforeCheckpointCommit).await;
            // Registry -> admission -> Store, atomic request replacement and
            // exact restore. No current()/reference() reentry and no await.
            let mut sessions = self.registry()?;
            let entry = sessions.get_mut(&session.id).ok_or_else(|| {
                failure(
                    ErrorKind::SessionLost,
                    "checkpoint owner disappeared during commit",
                )
            })?;
            if !Arc::ptr_eq(&entry.control, &control) {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "checkpoint attempt advanced",
                ));
            }
            control.preparation.checkpoint(request.input.version, || {
                let mut store = self
                    .store
                    .lock()
                    .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?;
                let version = store
                    .put_session_if_current(
                        &previous.session,
                        reservation.version,
                        snapshot.versions(),
                        &snapshot.lock_versions(),
                    )
                    .map_err(super::ownership::state_error)?;
                reservation.version = version;
                reservation.session = previous.session.clone();
                entry.request = request.clone();
                registered.publisher.send_replace(previous.clone());
                control.publish(previous.clone());
                reservation.armed = false;
                Ok(())
            })
        }
        .await
        .map_err(|error| control.preparation.failed(error));
        if let Err(error) = validated {
            return Err(reservation.finish_preparation_error(error));
        }
        #[cfg(test)]
        self.gates.wait(TestPoint::AfterCheckpointCommit).await;
        control.finished(Outcome::CheckpointCommitted {
            input_version: request.input.version,
            snapshot: previous,
        });
        #[cfg(test)]
        self.gates.wait(TestPoint::AfterCheckpointFinished).await;
        // Drop restores the prior terminal control only by exact Arc identity.
        Ok(())
    }
    async fn spawn_checkpoint(
        &self,
        registered: Registered,
        input: PreparedInput,
    ) -> AdapterResult<()> {
        let control = registered.transition.control.clone();
        let task_guard = TaskGuard(control.clone());
        let handles = self.owned_handles();
        let (result, receiver) = oneshot::channel();
        let mut guard = CallerGuard::new(control.clone());
        control.spawn(async move {
            let _task = task_guard;
            let value = handles.prepare_checkpoint(registered, input).await;
            let _ = result.send(value);
        })?;
        let value = receiver.await.map_err(|_| {
            failure(
                ErrorKind::SessionLost,
                "owned native checkpoint task ended without caller result",
            )
        });
        guard.disarm();
        value?
    }
    async fn prepare_launch(&self, mut registered: Registered) -> AdapterResult<Supervision> {
        let request = registered.request.clone();
        let schema = registered.schema.clone();
        let resume = registered
            .previous
            .as_ref()
            .map(|status| status.session.clone());
        let control = registered.transition.control.clone();
        #[cfg(test)]
        self.gates.wait(TestPoint::BeforeStarting).await;
        let context = (|| -> AdapterResult<_> {
            control.preparation.check()?;
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
            let mut session = resume
                .clone()
                .unwrap_or_else(|| registered.publisher.borrow().session.clone());
            if resume.as_ref().is_some_and(|previous| {
                previous.recovery["input_version"]
                    .as_u64()
                    .is_none_or(|version| request.input.version <= version)
            }) {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "native resume needs a newly checkpointed continuation; previous input cannot be replayed",
                ));
            }
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
                    .ok_or_else(|| {
                        failure(ErrorKind::SessionLost, "native resume record missing")
                    })?;
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
            pin_starting_input(&mut session, &request, resume.as_ref())?;
            let evidence = self
                .registry()?
                .get(&registered.transition.id)
                .ok_or_else(|| {
                    failure(
                        ErrorKind::SessionLost,
                        "native preparation owner disappeared",
                    )
                })?
                .evidence
                .clone();
            let resume_publication = Some(ResumePublication {
                previous: registered
                    .previous
                    .clone()
                    .unwrap_or_else(|| registered.publisher.borrow().clone()),
                sender: registered.publisher.clone(),
                evidence,
            });
            let reservation = Reservation {
                store: self.store.clone(),
                session,
                version: expected_version,
                published_attempt: false,
                ownership: ProcessOwnership::default(),
                armed: false,
                resume_publication,
                inference_started: false,
                attempt: control.clone(),
                #[cfg(test)]
                gates: self.gates.clone(),
            };
            Ok((snapshot, previous_cumulative, reservation))
        })();
        let (snapshot, previous_cumulative, mut reservation) = match context {
            Ok(context) => context,
            Err(error) => {
                let error = control.preparation.failed(error);
                let cause = control.preparation.cause(&error);
                control.finished(if let Some(snapshot) = registered.previous.clone() {
                    Outcome::RestoredBeforeAdmission { cause, snapshot }
                } else {
                    Outcome::FreshUnpublished {
                        cause,
                        error: Cause::Failed(error.kind, error.message.clone()),
                    }
                });
                return Err(error);
            }
        };
        let prepared=async {
            control.preparation.check()?;
        #[cfg(test)]
        self.gates.wait(TestPoint::BeforeInitialPersist).await;
        reservation.persist()?;
        reservation.armed = true;
        let binding = snapshot
            .verify_git_preparing(&request, &mut reservation.ownership, &control.preparation)
            .await?;
        snapshot.recheck(&self.store, &request, &self.agent)?;
        let executable = self.executable.clone();
        let executable = control.preparation.wait(filesystem(move || {
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
        }))
        .await?;
        let baseline: Vec<_> = std::env::vars_os().collect();
        let environment = native_environment(
            baseline.iter().cloned(),
            &snapshot.projects,
            &snapshot.project,
            &request.environment,
        )?;
        let version = super::preparation::bounded_git(
            &executable,
            &request.worktree,
            &["--version".into()],
            environment.clone(),
            tokio::time::Instant::now() + std::time::Duration::from_secs(5),
            reservation.ownership.group(),
            &control.preparation,
        )
        .await?;
        super::protocol::verify_native_version(&version)?;
        #[cfg(test)]
        self.gates.wait(TestPoint::BeforeBootstrap).await;
        let mut discovery = NativeServer::launch_preparing(
            &executable,
            &request.worktree,
            None,
            environment.clone(),
            reservation.ownership.group(),
            &control.preparation,
        )
        .await?;
        let discovered = async {
            control.preparation.check()?;
            reservation.session.pid = Some(discovery.pid());
            reservation.persist()?;
            let config = control.preparation.wait(discovery
                .rpc
                .call(
                    "config/read",
                    json!({"cwd":request.worktree,"includeLayers":false}),
                ))
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
        .await.map_err(|error|control.preparation.failed(error));
        let (policy, environment) =
            result_after_cleanup(discovered, discovery.shutdown().await.map(|_| ()))?;
        if self.runtime_broker
            && request.role == SessionRole::Executor
            && !policy.permits_runtime_broker()
        {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "native approval policy does not expose the runtime-broker client route; native reviewer retained",
            ));
        }
        let mut native = NativeServer::launch_preparing(
            &executable,
            &request.worktree,
            Some(&policy),
            environment,
            reservation.ownership.group(),
            &control.preparation,
        )
        .await?;
        let setup = async {
            control.preparation.check()?;
            reservation.session.pid = Some(native.pid());
            reservation.persist()?;
            let config = control.preparation.wait(native
                .rpc
                .call(
                    "config/read",
                    json!({"cwd":request.worktree,"includeLayers":false}),
                ))
                .await?;
            policy.verify_configuration(&config["config"])?;
            let account = control.preparation.wait(native
                .rpc
                .call("account/read", json!({"refreshToken":false})))
                .await?;
            verify_auth_readiness(&account)?;
            if request.role == SessionRole::Executor {
                let environment = control.preparation.wait(native
                    .rpc
                    .call("environment/status", json!({"environmentId":"local"})))
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
                control.preparation.wait(native.rpc.call("thread/resume", parameters)).await?
            } else {
                control.preparation.wait(native.rpc.call("thread/start", parameters)).await?
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
                let page = control.preparation.wait(native
                    .rpc
                    .call(
                        "mcpServerStatus/list",
                        json!({"threadId":thread,"limit":100,"cursor":cursor}),
                    ))
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
            if snapshot.verify_git_preparing(&request,&mut reservation.ownership,&control.preparation).await? != binding {
                return Err(failure(ErrorKind::OwnershipMismatch,"native workspace replaced during session initialization"));
            }
            snapshot.recheck(&self.store, &request, &self.agent)?;
            #[cfg(test)]
            self.gates.wait(TestPoint::BeforeDispatch).await;
            let turn = reservation
                .dispatch(
                    &mut native.rpc,
                    &snapshot,
                    &request,
                    &thread,
                    schema.as_ref(),
                )
                .await?;
            let turn = turn["turn"]["id"]
                .as_str()
                .filter(|id| !id.is_empty() && id.len() <= 256)
                .ok_or_else(|| failure(ErrorKind::ParseFailure, "native turn ID unavailable"))?
                .to_owned();
            Ok::<_, crate::adapter::AdapterError>((thread, turn))
        }
        .await.map_err(|error|control.preparation.failed(error));
        let (thread, turn) = match setup {
            Ok(value) => value,
            Err(error) => {
                #[cfg(test)]
                self.gates.wait(TestPoint::BeforeNativeCleanup).await;
                return Err(failure_after_cleanup(
                    error,
                    native.shutdown().await.map(|_| ()),
                ));
            }
        };
        reservation.session.state = SessionState::Running;
        reservation.session.recovery["native_turn"] = json!(turn);
        if let Err(error) = reservation.persist().map_err(|error|control.preparation.failed(error)) {
            return Err(failure_after_cleanup(
                error,
                native.shutdown().await.map(|_| ()),
            ));
        }
        Ok::<_,crate::adapter::AdapterError>((native,thread,turn,binding))
        }.await.map_err(|error|control.preparation.failed(error));
        let (native, thread, turn, binding) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => return Err(reservation.finish_preparation_error(error)),
        };
        let evidence = Arc::new(Mutex::new(Evidence {
            turn: Some(turn.clone()),
            pending: Some(
                ApprovalLedger::new(thread.clone(), turn.clone())
                    .for_workspace(request.worktree.clone()),
            ),
            ..Evidence::default()
        }));
        let (reply, replies) = mpsc::channel(16);
        let installed = (|| {
            let mut sessions = self.registry()?;
            let entry = sessions.get_mut(&registered.transition.id).ok_or_else(|| {
                failure(
                    ErrorKind::SessionLost,
                    "native preparation owner disappeared during transfer",
                )
            })?;
            if !Arc::ptr_eq(&entry.control, &control) {
                return Err(failure(
                    ErrorKind::StateConflict,
                    "native attempt owner advanced before transfer",
                ));
            }
            entry.evidence = evidence.clone();
            if let Some(publication) = &mut reservation.resume_publication {
                publication.evidence = evidence.clone();
            }
            entry.reply = reply;
            entry.request = request.clone();
            entry.schema = schema;
            registered.transition.transfer(entry)
        })()
        .map_err(|error| control.preparation.failed(error));
        if let Err(error) = installed {
            let error = failure_after_cleanup(error, native.shutdown().await.map(|_| ()));
            return Err(reservation.finish_preparation_error(error));
        }
        Ok(Supervision {
            native,
            reservation,
            sender: registered.publisher,
            stopped: registered.stopped,
            evidence,
            replies,
            identity: NativeTurn {
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
        })
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
    fn transport_succeeded(&self, status: &SessionStatus) -> bool {
        self.observe_owned(&SessionRef::from(&status.session), |owned, evidence| {
            Ok(evidence.completed
                && evidence.turn.is_some()
                && owned.session.native_ref.is_some()
                && owned.session.state == SessionState::Exited
                && owned.session.pid.is_none()
                && owned.failure.is_none()
                && status.failure.is_none()
                && status.exit_code == owned.exit_code
                && serde_json::to_value(&status.session).ok()
                    == serde_json::to_value(&owned.session).ok())
        })
        .unwrap_or(false)
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
            let control = {
                let sessions = self.registry()?;
                let entry = sessions.get(&session.id).ok_or_else(|| {
                    failure(ErrorKind::SessionLost, "native stop owner unavailable")
                })?;
                if entry.status.borrow().session.scope != session.scope {
                    return Err(failure(
                        ErrorKind::OwnershipMismatch,
                        "foreign native stop scope",
                    ));
                }
                entry.control.clone()
            };
            // Phase precedes admission: an idle stop must not reinterpret a
            // completed cancellation/failed restore as its own cancellation.
            let initial = control.subscribe().borrow().clone();
            let idle = matches!(initial, Phase::Finished(_));
            if !idle && !control.preparation.cancel() && control.consumed() {
                // The same channel was installed before preparation and remains
                // queued through native turn acknowledgement/supervision.
                let _ = control.stop.try_send(());
            }
            #[cfg(test)]
            self.gates.wait(TestPoint::StopWaiting).await;
            let outcome = control.wait_finished().await?;
            let snapshot = outcome.snapshot().cloned().ok_or_else(|| {
                outcome.error().unwrap_or_else(|| {
                    failure(
                        ErrorKind::SessionLost,
                        "captured native attempt has no published final snapshot",
                    )
                })
            })?;
            let persisted = self
                .store
                .lock()
                .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?
                .session(session.id)
                .map_err(super::ownership::state_error)?
                .ok_or_else(|| {
                    failure(
                        ErrorKind::SessionLost,
                        "captured native attempt record disappeared",
                    )
                })?
                .0;
            if serde_json::to_value(&persisted).ok() != serde_json::to_value(&snapshot.session).ok()
            {
                return Err(failure(
                    ErrorKind::StateConflict,
                    format!(
                        "captured native attempt advanced; factual outcome: {}",
                        outcome
                            .cause()
                            .map(|cause| cause.error().to_string())
                            .unwrap_or_else(|| "completed".into())
                    ),
                ));
            }
            if let Some(error) = outcome.error() {
                return Err(error);
            }
            if matches!(outcome, Outcome::Lost { .. }) {
                return Err(failure(
                    ErrorKind::SessionLost,
                    "captured native attempt remains Lost; cleanup or operation outcome is unverified",
                ));
            }
            if !idle {
                match &outcome {
                    Outcome::RestoredBeforeAdmission { cause, .. } => {
                        let error = cause.error();
                        return Err(failure(
                            error.kind,
                            format!(
                                "{}; exact prior Session restored before input admission",
                                error.message
                            ),
                        ));
                    }
                    Outcome::FailedBeforeAdmission {
                        cause: Cause::Failed(kind, message),
                        ..
                    } => return Err(failure(*kind, message.clone())),
                    Outcome::CheckpointCommitted { input_version, .. } => {
                        if !matches!(control.preparation.state(), Ok(Admission::CheckpointCommitted(version)) if version == *input_version)
                        {
                            return Err(failure(
                                ErrorKind::StateFailure,
                                "native checkpoint completion lacks its committed input version",
                            ));
                        }
                    }
                    _ => {}
                }
            }
            Ok(snapshot)
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
            let registered = self.register_existing(&session)?;
            self.spawn_launch(registered).await
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
            self.observe_owned(&session, |status, evidence| {
            let counters = evidence.counters.clone().unwrap_or_default();
            Ok(Usage {
                scope: status.session.scope.clone(),
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
        })
    }
    fn pending_approvals(&self, session: SessionRef) -> AdapterFuture<'_, Value> {
        Box::pin(async move {
            self.observe_owned(&session, |status, evidence| {
            Ok(
                json!({"scope":status.session.scope,"session_id":session.id,"native_uuid":status.session.native_ref,"native_turn":evidence.turn,"requires_human":!self.runtime_broker,"requests":evidence.pending.as_ref().map(ApprovalLedger::pending).unwrap_or_default()}),
            )
            })
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
            let registered = self.register_existing(&session)?;
            self.spawn_checkpoint(registered, input).await
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
        let preflight_ownership = reservation.ownership.checkpoint();
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
            let fatal = reservation.ownership.uncertain_since(&preflight_ownership);
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
            // Approvals fence parent versions, lock ABA and Session CAS together,
            // excluding a writer on a second SQLite connection after preflight.
            reservation.version = if matches!(reply.request.decision, OperationDecision::Approve) {
                store.put_session_if_current(&reservation.session, reservation.version, authority.snapshot.versions(), &authority.snapshot.lock_versions())
            } else {
                store.put_session(&reservation.session, reservation.version)
            }.map_err(|error| failure(ErrorKind::StateConflict, error.to_string()))?;
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
    let attempt = reservation.attempt.clone();
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
    let mut native_terminal_observed = false;
    let mut interrupt = None;
    let mut drain_deadline = None;
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
            _ = stop.recv(), if drain_deadline.is_none() => {
                let deadline = tokio::time::Instant::now() + INTERRUPT_DRAIN_TIMEOUT;
                drain_deadline = Some(deadline);
                interrupt = tokio::time::timeout_at(deadline, native.rpc.interrupt_turn(&thread, &turn))
                    .await.ok().and_then(Result::ok);
                continue;
            },
            Some(reply) = replies.recv() => {
                if drain_deadline.is_some() {
                    let _ = reply.result.send(Err(failure(ErrorKind::StateConflict, "native turn is draining after stop")));
                    continue;
                }
                match answer_approval(&mut native.rpc, &mut reservation, &evidence, &approval_authority, &sender, &mut status, reply).await {
                    Ok(true) => {
                        let deadline = tokio::time::Instant::now() + INTERRUPT_DRAIN_TIMEOUT;
                        drain_deadline = Some(deadline);
                        interrupt = tokio::time::timeout_at(deadline, native.rpc.interrupt_turn(&thread, &turn))
                            .await.ok().and_then(Result::ok);
                        continue;
                    },
                    Ok(false) => continue,
                    Err(error) => break Err(error),
                }
            },
            event = async {
                if let Some(deadline) = drain_deadline {
                    tokio::time::timeout_at(deadline, native.receive()).await
                        .map_err(|_| failure(ErrorKind::Timeout, "native interrupt terminal observation timed out"))?
                } else {
                    native.receive().await
                }
            } => event,
        };
        if let Ok(Event::Response { id, .. } | Event::Error { id, .. }) = &event
            && interrupt.as_ref() == Some(id)
        {
            interrupt = None;
            continue;
        }
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
                if drain_deadline.is_some() {
                    return Ok(());
                }
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
            if let Some(deadline) = drain_deadline {
                let declined = async {
                    let response = evidence
                        .lock()
                        .map_err(|_| {
                            failure(ErrorKind::StateFailure, "native approval state poisoned")
                        })?
                        .pending
                        .as_ref()
                        .ok_or_else(|| {
                            failure(ErrorKind::SessionLost, "native approval ledger unavailable")
                        })?
                        .preview_reply(id, OperationDecision::Deny)?;
                    reservation.store.lock()
                        .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?
                        .audit(&reservation.session.scope, "codex.approval.interrupt_declined",
                            json!({"session_id":reservation.session.id,"native_turn":turn,"request_id":id,"effect":"no_grant"}))
                        .map_err(|error| failure(ErrorKind::StateFailure, error.to_string()))?;
                    native.rpc.send(response).await?;
                    evidence
                        .lock()
                        .map_err(|_| {
                            failure(ErrorKind::StateFailure, "native approval state poisoned")
                        })?
                        .pending
                        .as_mut()
                        .ok_or_else(|| {
                            failure(ErrorKind::SessionLost, "native approval ledger unavailable")
                        })?
                        .reply(id, OperationDecision::Deny)?;
                    Ok::<_, crate::adapter::AdapterError>(())
                };
                match tokio::time::timeout_at(deadline, declined).await {
                    Ok(Ok(())) => continue,
                    Ok(Err(error)) => break Err(error),
                    Err(_) => {
                        break Err(failure(
                            ErrorKind::Timeout,
                            "native interrupt decline timed out",
                        ));
                    }
                }
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
                &mut native_terminal_observed,
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
    if let Err(error) = &result {
        let _ = attempt
            .preparation
            .failed(failure(error.kind, error.message.clone()));
    }
    let cleanup = native.shutdown().await;
    let completed = reservation.terminal(&mut status, &result, &cleanup, native_terminal_observed);
    if let Ok(mut evidence) = evidence.lock() {
        evidence.completed = completed;
        evidence.counters = tracker.turn_counters();
        evidence.cumulative = tracker.total;
        evidence.pending = None;
    }
    let publication_error = match reservation.publish(&sender, &mut status) {
        Ok(()) => {
            reservation.armed = false;
            #[cfg(test)]
            reservation
                .gates
                .wait(TestPoint::AfterTerminalPublication)
                .await;
            None
        }
        Err(error) => {
            if let Ok(mut evidence) = evidence.lock() {
                evidence.completed = false;
            }
            status.session.state = SessionState::Lost;
            status.failure = Some(format!("native terminal persistence failed: {error}"));
            sender.send_replace(status.clone());
            Some(Cause::Failed(error.kind, error.message))
        }
    };
    sender.send_replace(status.clone());
    // Drop's conservative publication, if required, is part of this attempt's
    // last act and precedes its own level-triggered Finished signal.
    drop(reservation);
    let published = attempt.published();
    let outcome = match published {
        Some(snapshot)
            if snapshot.session.state != SessionState::Lost && publication_error.is_none() =>
        {
            Outcome::Terminal { snapshot }
        }
        snapshot => {
            let cause = publication_error.unwrap_or_else(|| {
                result
                    .as_ref()
                    .err()
                    .or_else(|| cleanup.as_ref().err())
                    .map(|error| Cause::Failed(error.kind, error.message.clone()))
                    .unwrap_or_else(|| {
                        Cause::Failed(
                            ErrorKind::SessionLost,
                            "native attempt outcome or cleanup is unverified".into(),
                        )
                    })
            });
            let publication_result = snapshot
                .filter(|status| status.session.state == SessionState::Lost)
                .ok_or_else(|| {
                    Cause::Failed(
                        ErrorKind::SessionLost,
                        "native terminal publication is unavailable".into(),
                    )
                });
            Outcome::Lost {
                cause,
                publication_result,
            }
        }
    };
    attempt.finished(outcome);
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
    native_terminal_observed: &mut bool,
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
            if !matches!(
                params["turn"]["status"].as_str(),
                Some("completed" | "failed" | "interrupted")
            ) {
                return Err(failure(
                    ErrorKind::ParseFailure,
                    "native terminal status is invalid",
                ));
            }
            *native_terminal_observed = true;
            if params["turn"]["status"] == "interrupted" {
                return Ok(Some(false));
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
            match params["willRetry"].as_bool() {
                Some(true) => return Ok(None),
                Some(false) => *native_terminal_observed = true,
                None => {
                    return Err(failure(
                        ErrorKind::ParseFailure,
                        "native error retry disposition is unavailable",
                    ));
                }
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
    session_event(event, thread, turn, tracker, status, false, &mut false)
}

#[cfg(test)]
mod tests {
    use super::super::ownership::tests::Fixture;
    use super::*;
    use crate::domain::{ProjectId, Scope};
    use futures_util::{SinkExt, StreamExt};

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
                    .terminal(&mut fixture.status, &Ok(true), &cleanup, true)
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
                &Ok(std::process::ExitStatus::from_raw(0)),
                true
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
            status.session.recovery["native_turn"] = json!("turn");
            status.session.recovery["input_version"] = json!(owned.request.input.version);
            status.session.state = SessionState::Starting;
            let mut reservation = Reservation {
                store: owned.store.clone(),
                session: status.session.clone(),
                version: 0,
                published_attempt: false,
                ownership: ProcessOwnership::default(),
                armed: false,
                resume_publication: None,
                inference_started: false,
                attempt: Control::new(None).0,
                gates: Arc::new(TestGates::default()),
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
                    control: {
                        let (control, _) = Control::new(Some(self.status.clone()));
                        control.finished(Outcome::Terminal {
                            snapshot: self.status.clone(),
                        });
                        control
                    },
                },
            );
            (adapter, SessionRef::from(&self.status.session))
        }
    }

    #[tokio::test]
    async fn native_input_admission_is_atomic_with_scope_versions_and_durable_consumption() {
        let mut fixture = ApprovalFixture::new(true).await;
        let request = fixture.authority.request.clone();
        fixture
            .reservation
            .admit_dispatch(&fixture.authority.snapshot, &request)
            .unwrap();
        let committed = fixture.reservation.session.recovery["dispatch_intent"].clone();
        let scope = fixture.reservation.session.scope.clone();
        {
            let store = fixture.reservation.store.lock().unwrap();
            let stored = store
                .session(fixture.reservation.session.id)
                .unwrap()
                .unwrap()
                .0;
            assert_eq!(stored.recovery["dispatch_intent"], committed);
            assert_eq!(committed["input_bytes"], request.input.payload.len());
            assert_eq!(
                committed["input_sha256"],
                format!("{:x}", Sha256::digest(request.input.payload.as_bytes()))
            );
            assert_eq!(committed["consumed"], true);
            let event = store
                .events(&scope, 0, 1000)
                .unwrap()
                .into_iter()
                .rev()
                .find(|event| event.kind == "session.saved")
                .unwrap();
            assert_eq!(event.data["evidence"]["dispatch_intent"], committed);
        }
        // A second writer changes the owner after the old snapshot was captured;
        // admission must neither overwrite it nor journal another consumed input.
        let db = fixture
            .authority
            .request
            .project
            .root
            .parent()
            .unwrap()
            .join("state.sqlite3");
        let mut other = crate::state::Store::open(&db).unwrap();
        let mut task = other.task(scope.task_id.unwrap()).unwrap().unwrap();
        task.title = "changed after preflight".into();
        other.put_task(&mut task).unwrap();
        fixture.reservation.attempt = Control::new(None).0;
        let before = fixture
            .reservation
            .store
            .lock()
            .unwrap()
            .events(&scope, 0, 1000)
            .unwrap()
            .len();
        assert_eq!(
            fixture
                .reservation
                .admit_dispatch(&fixture.authority.snapshot, &request)
                .unwrap_err()
                .kind,
            ErrorKind::StateConflict
        );
        assert_eq!(
            fixture.reservation.session.recovery["dispatch_intent"],
            committed
        );
        let store = fixture.reservation.store.lock().unwrap();
        assert_eq!(store.events(&scope, 0, 1000).unwrap().len(), before);
        assert_eq!(
            store
                .session(fixture.reservation.session.id)
                .unwrap()
                .unwrap()
                .0
                .recovery["dispatch_intent"],
            committed
        );
    }

    #[tokio::test]
    async fn unobserved_native_input_stays_reserved_after_transport_loss_or_stop_with_dead_processes()
     {
        use std::os::unix::process::ExitStatusExt;
        for (observed, stopped) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut fixture = ApprovalFixture::new(true).await;
            fixture.reservation.inference_started = true;
            fixture.reservation.session.pid = Some(42);
            let result = if stopped {
                Ok(false)
            } else {
                Err(failure(ErrorKind::ProcessFailure, "native response lost"))
            };
            assert!(!fixture.reservation.terminal(
                &mut fixture.status,
                &result,
                &Ok(std::process::ExitStatus::from_raw(0)),
                observed
            ));
            fixture
                .reservation
                .publish(&fixture.sender, &mut fixture.status)
                .unwrap();
            assert_eq!(
                fixture.status.session.state,
                if observed && stopped {
                    SessionState::Stopped
                } else if observed {
                    SessionState::Failed
                } else {
                    SessionState::Lost
                }
            );
            assert_eq!(fixture.status.session.pid, None);
            assert_eq!(
                crate::git::executor_reserved(&fixture.status.session),
                !observed
            );
            if !observed {
                assert_eq!(
                    fixture.status.session.recovery["native_dispatch_unobserved"],
                    true
                );
                let store = fixture.reservation.store.lock().unwrap();
                let event = store
                    .events(&fixture.status.session.scope, 0, 1000)
                    .unwrap()
                    .into_iter()
                    .find(|event| event.kind == "codex.dispatch.unobserved")
                    .unwrap();
                assert_eq!(event.data["native_group_cleanup_confirmed"], true);
            }
        }
    }
    #[tokio::test]
    async fn workflow_completion_requires_private_owned_turn_and_verified_persisted_terminal() {
        let mut fixture = ApprovalFixture::new(true).await;
        let (adapter, reference) = fixture.terminal_adapter();
        let status = adapter.current(&reference).unwrap();
        assert!(adapter.transport_succeeded(&status));
        assert_eq!(status.exit_code, None);
        let mut fabricated = status.clone();
        fabricated.exit_code = Some(0);
        assert!(!adapter.transport_succeeded(&fabricated));
        fabricated = status.clone();
        fabricated.session.scope.project_id = crate::domain::ProjectId::new();
        assert!(!adapter.transport_succeeded(&fabricated));
        fixture.evidence.lock().unwrap().completed = false;
        fixture.reservation.session.recovery["completed"] = json!(true);
        fixture
            .reservation
            .publish(&fixture.sender, &mut fixture.status)
            .unwrap();
        assert!(!adapter.transport_succeeded(&fixture.status));
        fixture.evidence.lock().unwrap().completed = true;
        fixture.evidence.lock().unwrap().turn = Some("foreign-turn".into());
        assert!(!adapter.transport_succeeded(&fixture.status));
        fixture.evidence.lock().unwrap().turn = Some("turn".into());
        fixture.reservation.session.state = SessionState::Lost;
        fixture.reservation.session.pid = Some(42);
        fixture
            .reservation
            .publish(&fixture.sender, &mut fixture.status)
            .unwrap();
        assert!(!adapter.transport_succeeded(&fixture.status));
    }
    #[tokio::test]
    async fn observations_reject_transition_and_stale_turn_instead_of_mixing_telemetry() {
        let mut fixture = ApprovalFixture::new(true).await;
        let (adapter, reference) = fixture.terminal_adapter();
        fixture.evidence.lock().unwrap().counters = Some(TokenCounters {
            input: Some(42),
            ..TokenCounters::default()
        });
        assert_eq!(
            adapter
                .usage(reference.clone(), "review".into(), Some(2))
                .await
                .unwrap()
                .input_tokens,
            Some(42)
        );
        let (claim, _, _) = adapter.claim(&reference).unwrap();
        assert_eq!(
            adapter
                .usage(reference.clone(), "review".into(), Some(2))
                .await
                .unwrap_err()
                .kind,
            ErrorKind::StateConflict
        );
        assert!(!adapter.transport_succeeded(&fixture.status));
        drop(claim);
        fixture.evidence.lock().unwrap().turn = Some("different-resumed-turn".into());
        assert_eq!(
            adapter
                .usage(reference.clone(), "review".into(), Some(2))
                .await
                .unwrap_err()
                .kind,
            ErrorKind::StateConflict
        );
        assert_eq!(
            adapter.pending_approvals(reference).await.unwrap_err().kind,
            ErrorKind::StateConflict
        );
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
    async fn checkpoint_and_resume_publish_fresh_input_before_reserving_and_restore_consumed_history()
     {
        let mut fixture = ApprovalFixture::new(true).await;
        fixture.status.session.recovery["dispatch_intent"] = json!({
            "id":"historical-runtime-input","origin":"runtime","consumed":true,
        });
        let (adapter, reference) = fixture.terminal_adapter();
        let original = adapter.current(&reference).unwrap().session;
        let mut input = fixture.authority.request.input.clone();
        input.version += 1;
        input.payload = "explicit fresh continuation with owned context".into();
        let db = fixture
            .authority
            .request
            .project
            .root
            .parent()
            .unwrap()
            .join("state.sqlite3");
        let connection = rusqlite::Connection::open(db).unwrap();
        connection.execute_batch("CREATE TABLE checkpoint_expectation(version INTEGER,revision TEXT,bytes INTEGER,hash TEXT,restore TEXT,sources TEXT);
            CREATE TRIGGER verify_reserved_checkpoint BEFORE UPDATE ON records
            WHEN NEW.kind='session' AND json_extract(NEW.body,'$.data.state')='STARTING'
            AND EXISTS(SELECT 1 FROM checkpoint_expectation WHERE
                version IS NOT json_extract(NEW.body,'$.data.recovery.input_version') OR
                revision IS NOT json_extract(NEW.body,'$.data.recovery.input_revision') OR
                bytes IS NOT json_extract(NEW.body,'$.data.recovery.input_bytes') OR
                hash IS NOT json_extract(NEW.body,'$.data.recovery.input_sha256') OR
                restore IS NOT json_extract(NEW.body,'$.data.recovery.pre_dispatch_restore_sha256') OR
                sources IS NOT json_extract(NEW.body,'$.data.recovery.source_versions') OR
                json_extract(NEW.body,'$.data.recovery.native_turn') IS NOT NULL OR
                json_extract(NEW.body,'$.data.recovery.previous_native_turn') IS NOT 'turn' OR
                json_extract(NEW.body,'$.data.recovery.dispatch_intent') IS NOT NULL)
            BEGIN SELECT RAISE(ABORT,'reserved input differs from fresh checkpoint'); END;").unwrap();
        connection
            .execute(
                "INSERT INTO checkpoint_expectation VALUES(?1,?2,?3,?4,?5,?6)",
                rusqlite::params![
                    input.version,
                    input.revision,
                    input.payload.len(),
                    format!("{:x}", Sha256::digest(input.payload.as_bytes())),
                    format!(
                        "{:x}",
                        Sha256::digest(serde_json::to_vec(&original).unwrap())
                    ),
                    serde_json::to_string(&input.source_versions).unwrap(),
                ],
            )
            .unwrap();
        adapter
            .checkpoint(reference.clone(), input.clone())
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(adapter.current(&reference).unwrap().session).unwrap(),
            serde_json::to_value(&original).unwrap()
        );
        assert_eq!(
            adapter
                .registry()
                .unwrap()
                .get(&reference.id)
                .unwrap()
                .request
                .input
                .payload,
            input.payload
        );
        assert_eq!(
            adapter.resume(reference.clone()).await.unwrap_err().kind,
            ErrorKind::ExecutableMissing
        );
        assert_eq!(
            serde_json::to_value(adapter.current(&reference).unwrap().session).unwrap(),
            serde_json::to_value(original).unwrap()
        );
        adapter.release(reference).unwrap();
    }
    #[tokio::test]
    async fn failed_pre_inference_resume_restores_the_owned_terminal_record_and_watch() {
        let mut fixture = ApprovalFixture::new(true).await;
        let (adapter, reference) = fixture.terminal_adapter();
        let mut input = fixture.authority.request.input.clone();
        input.version += 1;
        adapter.checkpoint(reference.clone(), input).await.unwrap();
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
    async fn resume_never_replays_cached_prompt_and_checkpoint_refreshes_only_own_metadata() {
        let mut fixture = ApprovalFixture::new(true).await;
        let (adapter, reference) = fixture.terminal_adapter();
        let original = adapter.current(&reference).unwrap();
        assert_eq!(
            adapter.resume(reference.clone()).await.unwrap_err().kind,
            ErrorKind::StateConflict
        );
        assert_eq!(
            serde_json::to_value(adapter.current(&reference).unwrap().session).unwrap(),
            serde_json::to_value(original.session).unwrap()
        );
        let mut project = fixture.authority.request.project.clone();
        project.name = "updated own display metadata".into();
        fixture
            .reservation
            .store
            .lock()
            .unwrap()
            .put_project(&mut project)
            .unwrap();
        let mut input = fixture.authority.request.input.clone();
        input.version += 1;
        input.payload = "explicit safe continuation".into();
        adapter.checkpoint(reference.clone(), input).await.unwrap();
        assert_eq!(
            adapter
                .registry()
                .unwrap()
                .get(&reference.id)
                .unwrap()
                .request
                .project
                .version,
            project.version
        );
        assert_eq!(
            adapter.resume(reference.clone()).await.unwrap_err().kind,
            ErrorKind::ExecutableMissing
        );
        // The failed pre-inference attempt may retry this explicit continuation.
        assert_eq!(
            adapter.current(&reference).unwrap().session.state,
            SessionState::Exited
        );
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
                published_attempt: false,
                ownership: ProcessOwnership::default(),
                armed: false,
                resume_publication: Some(ResumePublication {
                    previous: previous.clone(),
                    sender: fixture.sender.clone(),
                    evidence: fixture.evidence.clone(),
                }),
                inference_started: !uncertain,
                attempt: Control::new(None).0,
                gates: Arc::new(TestGates::default()),
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
            assert_eq!(current.session.state, SessionState::Lost);
            assert_eq!(current.session.pid, if uncertain { Some(42) } else { None });
            assert_eq!(
                current.session.recovery["native_dispatch_unobserved"],
                if uncertain { Value::Null } else { json!(true) }
            );
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
    async fn actual_supervisor_drains_stop_cancel_and_declines_without_inventing_terminal_proof() {
        for case in ["stop", "cancel", "decline", "ack_only", "foreign"] {
            let mut fixture = ApprovalFixture::new(true).await;
            fixture
                .reservation
                .admit_dispatch(&fixture.authority.snapshot, &fixture.authority.request)
                .unwrap();
            fixture.reservation.inference_started = true;
            fixture.status.session = fixture.reservation.session.clone();
            fixture.sender.send_replace(fixture.status.clone());
            let (client, server) = tokio::net::UnixStream::pair().unwrap();
            let cwd = fixture.authority.request.worktree.clone();
            let declined = Arc::new(AtomicBool::new(false));
            let observed_decline = declined.clone();
            let peer = tokio::spawn(async move {
                let mut socket = tokio_tungstenite::accept_async(server).await.unwrap();
                while let Some(Ok(message)) = socket.next().await {
                    let Ok(text) = message.to_text() else {
                        continue;
                    };
                    let value: Value = serde_json::from_str(text).unwrap();
                    if value["method"] != "turn/interrupt" {
                        continue;
                    }
                    assert_eq!(
                        value["params"],
                        json!({"threadId":"thread","turnId":"turn"})
                    );
                    if case == "decline" {
                        socket.send(tokio_tungstenite::tungstenite::Message::Text(json!({"id":2,"method":"item/commandExecution/requestApproval","params":{"threadId":"thread","turnId":"turn","itemId":"after-stop","command":"synthetic forbidden grant","cwd":cwd}}).to_string().into())).await.unwrap();
                        let response = socket.next().await.unwrap().unwrap();
                        let response: Value =
                            serde_json::from_str(response.to_text().unwrap()).unwrap();
                        assert_eq!(response, json!({"id":2,"result":{"decision":"decline"}}));
                        observed_decline.store(true, Ordering::SeqCst);
                    }
                    if case != "ack_only" {
                        let turn = if case == "foreign" {
                            "other-turn"
                        } else {
                            "turn"
                        };
                        socket.send(tokio_tungstenite::tungstenite::Message::Text(json!({"method":"turn/completed","params":{"threadId":"thread","turn":{"id":turn,"status":"interrupted"}}}).to_string().into())).await.unwrap();
                    }
                    // Terminal precedes acknowledgement: the previous synchronous
                    // call queued it and then broke before the supervisor read it.
                    let _ = socket
                        .send(tokio_tungstenite::tungstenite::Message::Text(
                            json!({"id":value["id"],"result":{}}).to_string().into(),
                        ))
                        .await;
                }
            });
            let rpc = NativeRpc::connect(client).await.unwrap();
            let native =
                NativeServer::supervisor_fixture(rpc, fixture.reservation.ownership.group())
                    .unwrap();
            fixture.reservation.session.pid = Some(native.pid());
            fixture
                .reservation
                .publish(&fixture.sender, &mut fixture.status)
                .unwrap();
            let mut receiver = fixture.sender.subscribe();
            let (stop_sender, stop) = mpsc::channel(1);
            let (reply_sender, replies) = mpsc::channel(1);
            let mut cancel_receipt = None;
            if case == "cancel" {
                let (mut cancel, receipt) = reply(OperationDecision::Cancel, "turn");
                cancel_receipt = Some(receipt);
                cancel.request.operation_hash = fixture
                    .evidence
                    .lock()
                    .unwrap()
                    .pending
                    .as_ref()
                    .unwrap()
                    .request(&RpcId::Number(1))
                    .unwrap()
                    .operation_hash;
                reply_sender.send(cancel).await.unwrap();
            } else {
                stop_sender.send(()).await.unwrap();
            }
            let store = fixture.reservation.store.clone();
            let id = fixture.reservation.session.id;
            let evidence = fixture.evidence.clone();
            let authority = fixture.authority;
            let task = tokio::spawn(supervise(
                native,
                fixture.reservation,
                fixture.sender,
                stop,
                evidence.clone(),
                replies,
                NativeTurn {
                    thread: "thread".into(),
                    turn: "turn".into(),
                    previous_turn: None,
                    previous_cumulative: None,
                    authority: authority.snapshot,
                    request: authority.request,
                    binding: authority.binding,
                    runtime_broker: true,
                },
            ));
            tokio::time::timeout(std::time::Duration::from_secs(15), async {
                while !receiver.borrow().terminal() {
                    receiver.changed().await.unwrap();
                }
                task.await.unwrap();
            })
            .await
            .expect("bounded actual supervisor termination");
            let status = receiver.borrow().clone();
            let expected = if matches!(case, "ack_only" | "foreign") {
                SessionState::Lost
            } else {
                SessionState::Stopped
            };
            assert_eq!(status.session.state, expected, "{case}");
            assert_eq!(status.session.pid, None, "owned group verified dead {case}");
            assert_eq!(status.exit_code, None);
            assert!(!evidence.lock().unwrap().completed);
            if let Some(receipt) = cancel_receipt {
                receipt.await.unwrap().unwrap();
            }
            assert_eq!(
                serde_json::to_value(&status.session).unwrap(),
                serde_json::to_value(store.lock().unwrap().session(id).unwrap().unwrap().0)
                    .unwrap()
            );
            if case == "decline" {
                assert!(declined.load(Ordering::SeqCst));
                assert!(
                    store
                        .lock()
                        .unwrap()
                        .events(&status.session.scope, 0, 1000)
                        .unwrap()
                        .iter()
                        .any(|e| e.kind == "codex.approval.interrupt_declined"
                            && e.data["effect"] == "no_grant")
                );
            }
            if expected == SessionState::Lost {
                assert_eq!(status.session.recovery["native_dispatch_unobserved"], true);
            }
            drop(stop_sender);
            drop(reply_sender);
            peer.await.unwrap();
        }
    }

    #[tokio::test]
    async fn source_consultation_dispatch_ignores_descendant_task_review_locks() {
        for goal_scoped in [false, true] {
            let mut owned = Fixture::new(true);
            let task_scope = owned.request.scope.clone();
            let lock_id = crate::git::WorktreeManager::lock_review(
                &mut owned.store.lock().unwrap(),
                task_scope.task_id.unwrap(),
                &owned.request.input.revision,
                "separate descendant review",
            )
            .unwrap();
            for active in [true, false] {
                if !active {
                    crate::git::WorktreeManager::unlock_review(
                        &mut owned.store.lock().unwrap(),
                        lock_id,
                    )
                    .unwrap();
                }
                owned.request.scope = Scope {
                    project_id: task_scope.project_id,
                    goal_id: if goal_scoped {
                        task_scope.goal_id
                    } else {
                        None
                    },
                    task_id: None,
                };
                owned.request.input.scope = owned.request.scope.clone();
                owned.request.worktree = owned.request.project.root.clone();
                owned.request.role = SessionRole::Consultant;
                let authority =
                    ScopeSnapshot::capture(&owned.store, &owned.request, "codex").unwrap();
                let mut session = status().session;
                session.id = SessionId::new();
                session.scope = owned.request.scope.clone();
                session.worktree = owned.request.worktree.clone();
                session.role = SessionRole::Consultant;
                pin_starting_input(&mut session, &owned.request, None).unwrap();
                let mut reservation = Reservation {
                    store: owned.store.clone(),
                    session,
                    version: 0,
                    published_attempt: false,
                    ownership: ProcessOwnership::default(),
                    armed: false,
                    resume_publication: None,
                    inference_started: false,
                    attempt: Control::new(None).0,
                    gates: Arc::new(TestGates::default()),
                };
                reservation.persist().unwrap();
                let (mut rpc, mut wire, peer) = rpc_peer().await;
                {
                    let mut dispatch = Box::pin(reservation.dispatch(
                        &mut rpc,
                        &authority,
                        &owned.request,
                        "own-source-thread",
                        None,
                    ));
                    let message = tokio::select! {
                        outcome = &mut dispatch => panic!("source consultation rejected by descendant lock: {outcome:?}"),
                        message = wire.recv() => message.unwrap(),
                    };
                    assert_eq!(message["method"], "turn/start");
                    assert_eq!(message["params"]["threadId"], "own-source-thread");
                    assert_eq!(
                        message["params"]["input"][0]["text"],
                        owned.request.input.payload
                    );
                }
                let store = owned.store.lock().unwrap();
                let persisted = store.session(reservation.session.id).unwrap().unwrap().0;
                assert_eq!(persisted.scope, owned.request.scope);
                assert_eq!(persisted.recovery["dispatch_intent"]["consumed"], true);
                let descendant = store.record(lock_id).unwrap().unwrap();
                assert_eq!(descendant.scope, task_scope);
                assert_eq!(descendant.data["active"], active);
                drop(store);
                drop(rpc);
                peer.abort();
            }
        }
    }

    #[tokio::test]
    async fn actual_turn_wire_is_fenced_after_preflight_and_consumption_precedes_delivery() {
        for stale_owner in [false, true] {
            let mut fixture = ApprovalFixture::new(true).await;
            let request = fixture.authority.request.clone();
            let store = fixture.reservation.store.clone();
            let session_id = fixture.reservation.session.id;
            if stale_owner {
                let db = request.project.root.parent().unwrap().join("state.sqlite3");
                let mut writer = crate::state::Store::open(&db).unwrap();
                let mut task = writer
                    .task(request.scope.task_id.unwrap())
                    .unwrap()
                    .unwrap();
                task.title = "second writer after final preflight".into();
                writer.put_task(&mut task).unwrap();
            }
            let (mut rpc, mut wire, peer) = rpc_peer().await;
            {
                let mut dispatch = Box::pin(fixture.reservation.dispatch(
                    &mut rpc,
                    &fixture.authority.snapshot,
                    &request,
                    "thread",
                    None,
                ));
                if stale_owner {
                    let outcome =
                        tokio::time::timeout(std::time::Duration::from_millis(200), &mut dispatch)
                            .await;
                    assert!(
                        matches!(outcome, Ok(Err(error)) if error.kind == ErrorKind::StateConflict)
                    );
                    assert!(wire.try_recv().is_err());
                    assert!(
                        store
                            .lock()
                            .unwrap()
                            .session(session_id)
                            .unwrap()
                            .unwrap()
                            .0
                            .recovery["dispatch_intent"]
                            .is_null()
                    );
                } else {
                    let message = tokio::select! {
                        outcome = &mut dispatch => panic!("dispatch finished before fixture response: {outcome:?}"),
                        message = wire.recv() => message.unwrap(),
                    };
                    assert_eq!(message["method"], "turn/start");
                    assert_eq!(message["params"]["input"][0]["text"], request.input.payload);
                    let stored = store
                        .lock()
                        .unwrap()
                        .session(session_id)
                        .unwrap()
                        .unwrap()
                        .0;
                    assert_eq!(stored.recovery["dispatch_intent"]["consumed"], true);
                    assert_eq!(
                        stored.recovery["dispatch_intent"]["input_bytes"],
                        request.input.payload.len()
                    );
                }
            }
            assert_eq!(fixture.reservation.inference_started, !stale_owner);
            drop(rpc);
            peer.abort();
        }
    }

    #[tokio::test]
    async fn oversized_encoded_input_is_rejected_before_consumption_or_native_wire() {
        let mut fixture = ApprovalFixture::new(true).await;
        let mut request = fixture.authority.request.clone();
        // The prepared text is under the plain-byte limit; JSON escaping makes
        // the actual frame too large. Test the complete serialized wire bound.
        request.input.payload = "\"".repeat(600_000);
        let before = fixture.reservation.version;
        let (mut rpc, mut wire, peer) = rpc_peer().await;
        assert_eq!(
            fixture
                .reservation
                .dispatch(
                    &mut rpc,
                    &fixture.authority.snapshot,
                    &request,
                    "thread",
                    None
                )
                .await
                .unwrap_err()
                .kind,
            ErrorKind::InvalidInput
        );
        assert_eq!(fixture.reservation.version, before);
        assert!(!fixture.reservation.inference_started);
        assert!(fixture.reservation.session.recovery["dispatch_intent"].is_null());
        assert!(wire.try_recv().is_err());
        drop(rpc);
        peer.abort();
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
        // Production always owns a live app-server group during approval.
        // That preexisting flag must not turn this rejected grant into Lost.
        let server = fixture.reservation.ownership.group();
        server.store(true, std::sync::atomic::Ordering::SeqCst);
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
        assert!(server.load(std::sync::atomic::Ordering::SeqCst));
        assert!(fixture.reservation.ownership.uncertain());
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
            fixture
                .reservation
                .ownership
                .group()
                .store(true, std::sync::atomic::Ordering::SeqCst);
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
    fn only_exact_authoritative_native_terminal_events_resolve_consumed_input() {
        for (method, params, expected, success) in [
            (
                "turn/completed",
                json!({"threadId":"foreign","turn":{"id":"turn","status":"failed"}}),
                false,
                false,
            ),
            (
                "turn/completed",
                json!({"threadId":"thread","turn":{"id":"other","status":"completed"}}),
                false,
                false,
            ),
            (
                "turn/completed",
                json!({"threadId":"thread","turn":{"id":"turn","status":"unknown"}}),
                false,
                false,
            ),
            (
                "error",
                json!({"threadId":"thread","turnId":"turn","willRetry":true}),
                false,
                true,
            ),
            (
                "error",
                json!({"threadId":"thread","turnId":"turn"}),
                false,
                false,
            ),
            (
                "error",
                json!({"threadId":"thread","turnId":"turn","willRetry":false}),
                true,
                false,
            ),
            (
                "turn/completed",
                json!({"threadId":"thread","turn":{"id":"turn","status":"failed"}}),
                true,
                false,
            ),
            (
                "turn/completed",
                json!({"threadId":"thread","turn":{"id":"turn","status":"completed"}}),
                true,
                true,
            ),
        ] {
            let mut observed = false;
            let mut status = status();
            let mut tracker = UsageTracker::new("thread".into(), "turn".into());
            let result = session_event(
                Event::Notification {
                    method: method.into(),
                    params,
                },
                "thread",
                "turn",
                &mut tracker,
                &mut status,
                false,
                &mut observed,
            );
            assert_eq!(result.is_ok(), success, "{method}");
            assert_eq!(observed, expected, "{method}");
        }
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
    async fn bounded<T>(future: impl std::future::Future<Output = T>) -> T {
        tokio::time::timeout(std::time::Duration::from_secs(10), future)
            .await
            .expect("owned fixture must complete within its unchanged deadline")
    }
    fn preparing_control(adapter: &CodexAdapter, id: SessionId) -> Arc<Control> {
        adapter
            .registry()
            .unwrap()
            .get(&id)
            .unwrap()
            .control
            .clone()
    }
    fn bootstrap_fixture(owned: &Fixture) -> (PathBuf, PathBuf) {
        use std::os::unix::fs::PermissionsExt;
        let parent = owned.request.project.root.parent().unwrap();
        let executable = parent.join("synthetic-codex");
        let marker = parent.join("native-bootstrap-ready");
        let quoted = format!("'{}'", marker.to_str().unwrap().replace('\'', "'\\''"));
        std::fs::write(&executable,format!("#!/bin/sh\nif [ \"$1\" = '--version' ]; then printf '%s\\n' 'codex-cli 0.160.0'; exit 0; fi\nprintf '%s' \"$$\" > {quoted}\nexec /bin/sleep 30\n")).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        (executable, marker)
    }
    async fn ready(marker: &std::path::Path) {
        bounded(async {
            while !marker.exists() {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        })
        .await;
    }
    fn assert_leader_reaped(marker: &std::path::Path) {
        let pid = std::fs::read_to_string(marker).unwrap();
        let output = std::process::Command::new("/bin/ps")
            .args(["-p", &pid, "-o", "pid="])
            .output()
            .unwrap();
        assert!(
            output.stdout.is_empty(),
            "owned bootstrap leader must be reaped before Finished"
        );
    }

    #[tokio::test]
    async fn registered_start_caller_drop_before_starting_does_not_write_or_spawn() {
        let owned = Fixture::new(true);
        let (executable, marker) = bootstrap_fixture(&owned);
        let adapter =
            Arc::new(CodexAdapter::new("codex".into(), executable, owned.store.clone()).unwrap());
        let gate = adapter.gates.install(TestPoint::BeforeStarting);
        let request = owned.request.clone();
        let actor = adapter.clone();
        let caller = tokio::spawn(async move { actor.start(request).await });
        bounded(gate.reached()).await;
        let (id, control) = {
            let registry = adapter.registry().unwrap();
            let (id, entry) = registry.iter().next().unwrap();
            (*id, entry.control.clone())
        };
        let before = owned.store.lock().unwrap().session(id).unwrap();
        caller.abort();
        let _ = caller.await;
        assert!(matches!(
            control.preparation.state().unwrap(),
            Admission::Cancelled
        ));
        gate.release();
        let outcome = bounded(control.wait_finished()).await.unwrap();
        assert!(before.is_none());
        assert!(matches!(
            outcome,
            Outcome::FreshUnpublished {
                cause: Cause::Cancelled,
                ..
            }
        ));
        assert!(owned.store.lock().unwrap().session(id).unwrap().is_none());
        assert!(!adapter.registry().unwrap().contains_key(&id));
        assert!(!marker.exists());
    }

    #[tokio::test]
    async fn actual_registered_fresh_bootstrap_stop_or_caller_drop_verifies_cleanup_before_failed()
    {
        for drop_caller in [false, true] {
            let owned = Fixture::new(true);
            let (executable, marker) = bootstrap_fixture(&owned);
            let adapter = Arc::new(
                CodexAdapter::new("codex".into(), executable, owned.store.clone()).unwrap(),
            );
            let actor = adapter.clone();
            let request = owned.request.clone();
            let mut caller = tokio::spawn(async move { actor.start(request).await });
            tokio::select! {
                _=ready(&marker)=>{},
                early=&mut caller=>panic!("bootstrap fixture ended before readiness: {early:?}"),
            }
            let (id, reference) = {
                let registry = adapter.registry().unwrap();
                let (id, entry) = registry.iter().next().unwrap();
                (*id, SessionRef::from(&entry.status.borrow().session))
            };
            let control = preparing_control(&adapter, id);
            let starting = adapter.current(&reference).unwrap();
            assert_eq!(starting.session.state, SessionState::Starting);
            if drop_caller {
                caller.abort();
                let _ = caller.await;
                let outcome = bounded(control.wait_finished()).await.unwrap();
                assert!(matches!(
                    outcome,
                    Outcome::FailedBeforeAdmission {
                        cause: Cause::Cancelled,
                        ..
                    }
                ));
            } else {
                let stopped = bounded(adapter.stop(reference.clone())).await.unwrap();
                assert_eq!(stopped.session.state, SessionState::Failed);
                assert_eq!(
                    bounded(caller).await.unwrap().unwrap_err().kind,
                    ErrorKind::StateConflict
                );
            }
            let snapshot = adapter.current(&reference).unwrap();
            assert_eq!(snapshot.session.state, SessionState::Failed);
            assert_eq!(snapshot.session.pid, None);
            assert!(snapshot.session.recovery.get("dispatch_intent").is_none());
            assert!(
                snapshot
                    .failure
                    .as_deref()
                    .unwrap()
                    .contains("cancelled before admission")
            );
            assert_eq!(snapshot.exit_code, None);
            assert_leader_reaped(&marker);
            assert_eq!(
                bounded(adapter.stop(reference.clone()))
                    .await
                    .unwrap()
                    .session
                    .state,
                SessionState::Failed
            );
            adapter.release(reference).unwrap();
        }
    }

    #[tokio::test]
    async fn actual_registered_resume_cancel_before_starting_restores_exact_snapshot_and_idle_stop()
    {
        let mut fixture = ApprovalFixture::new(true).await;
        let (adapter, reference) = fixture.terminal_adapter();
        let original = adapter.current(&reference).unwrap();
        let mut input = fixture.authority.request.input.clone();
        input.version += 1;
        adapter
            .checkpoint(reference.clone(), input.clone())
            .await
            .unwrap();
        let gate = adapter.gates.install(TestPoint::BeforeStarting);
        let actor = adapter.clone();
        let target = reference.clone();
        let caller = tokio::spawn(async move { actor.resume(target).await });
        bounded(gate.reached()).await;
        let control = preparing_control(&adapter, reference.id);
        let actor = adapter.clone();
        let target = reference.clone();
        let stopper = tokio::spawn(async move { actor.stop(target).await });
        bounded(async {
            while !matches!(control.preparation.state().unwrap(), Admission::Cancelled) {
                tokio::task::yield_now().await;
            }
        })
        .await;
        gate.release();
        let launch_error = bounded(caller).await.unwrap().unwrap_err();
        let stop_error = bounded(stopper).await.unwrap().unwrap_err();
        assert_eq!(launch_error.kind, ErrorKind::StateConflict);
        assert!(stop_error.message.contains("exact prior Session restored"));
        assert!(matches!(
            bounded(control.wait_finished()).await.unwrap(),
            Outcome::RestoredBeforeAdmission {
                cause: Cause::Cancelled,
                ..
            }
        ));
        assert_eq!(
            serde_json::to_value(adapter.current(&reference).unwrap().session).unwrap(),
            serde_json::to_value(&original.session).unwrap()
        );
        assert_eq!(
            bounded(adapter.stop(reference.clone()))
                .await
                .unwrap()
                .session
                .state,
            SessionState::Exited
        );
        assert!(fixture.evidence.lock().unwrap().completed);
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
        adapter.release(reference).unwrap();
    }

    #[tokio::test]
    async fn actual_checkpoint_stop_has_both_linear_orders_and_finished_window_is_idle() {
        for commit_first in [false, true] {
            let mut fixture = ApprovalFixture::new(true).await;
            let (adapter, reference) = fixture.terminal_adapter();
            let original = adapter.current(&reference).unwrap();
            let original_version = fixture
                .reservation
                .store
                .lock()
                .unwrap()
                .session(reference.id)
                .unwrap()
                .unwrap()
                .1;
            let mut input = fixture.authority.request.input.clone();
            input.version += 1;
            let point = if commit_first {
                TestPoint::AfterCheckpointCommit
            } else {
                TestPoint::BeforeCheckpointCommit
            };
            let gate = adapter.gates.install(point);
            let actor = adapter.clone();
            let target = reference.clone();
            let updated = input.clone();
            let caller = tokio::spawn(async move { actor.checkpoint(target, updated).await });
            bounded(gate.reached()).await;
            let control = preparing_control(&adapter, reference.id);
            let stop_gate = adapter.gates.install(TestPoint::StopWaiting);
            let actor = adapter.clone();
            let target = reference.clone();
            let stopper = tokio::spawn(async move { actor.stop(target).await });
            bounded(stop_gate.reached()).await;
            let admission = control.preparation.state().unwrap();
            gate.release();
            stop_gate.release();
            let checkpoint = bounded(caller).await.unwrap();
            let stop = bounded(stopper).await.unwrap();
            if commit_first {
                assert!(
                    matches!(admission,Admission::CheckpointCommitted(version) if version==input.version)
                );
                checkpoint.unwrap();
                assert_eq!(stop.unwrap().session.state, SessionState::Exited);
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
            } else {
                assert!(matches!(admission, Admission::Cancelled));
                assert_eq!(checkpoint.unwrap_err().kind, ErrorKind::StateConflict);
                assert!(
                    stop.unwrap_err()
                        .message
                        .contains("exact prior Session restored")
                );
                assert_eq!(
                    adapter
                        .registry()
                        .unwrap()
                        .get(&reference.id)
                        .unwrap()
                        .request
                        .input
                        .version,
                    fixture.authority.request.input.version
                );
            }
            let current = adapter.current(&reference).unwrap();
            assert_eq!(
                serde_json::to_value(current.session).unwrap(),
                serde_json::to_value(&original.session).unwrap()
            );
            assert!(
                fixture
                    .reservation
                    .store
                    .lock()
                    .unwrap()
                    .session(reference.id)
                    .unwrap()
                    .unwrap()
                    .1
                    > original_version
            );
            assert_eq!(
                bounded(adapter.stop(reference.clone()))
                    .await
                    .unwrap()
                    .session
                    .state,
                SessionState::Exited,
                "Store version increments do not change exact snapshot equality"
            );
            adapter.release(reference).unwrap();
        }
        let mut fixture = ApprovalFixture::new(true).await;
        let (adapter, reference) = fixture.terminal_adapter();
        let mut input = fixture.authority.request.input.clone();
        input.version += 1;
        let gate = adapter.gates.install(TestPoint::AfterCheckpointFinished);
        let actor = adapter.clone();
        let target = reference.clone();
        let caller = tokio::spawn(async move { actor.checkpoint(target, input).await });
        bounded(gate.reached()).await;
        let idle = bounded(adapter.stop(reference.clone())).await;
        gate.release();
        bounded(caller).await.unwrap().unwrap();
        assert_eq!(idle.unwrap().session.state, SessionState::Exited);
        adapter.release(reference).unwrap();
    }
    #[tokio::test]
    async fn captured_stop_a_cannot_cancel_later_resume_b_or_adopt_its_status() {
        let mut fixture = ApprovalFixture::new(true).await;
        let (mut adapter, reference) = fixture.terminal_adapter();
        let (executable, marker) = bootstrap_fixture(&fixture._owned);
        Arc::get_mut(&mut adapter).unwrap().executable = executable;
        let mut input = fixture.authority.request.input.clone();
        input.version += 1;
        adapter
            .checkpoint(reference.clone(), input.clone())
            .await
            .unwrap();
        let preparation_gate = adapter.gates.install(TestPoint::BeforeStarting);
        let actor = adapter.clone();
        let target = reference.clone();
        let caller_a = tokio::spawn(async move { actor.resume(target).await });
        bounded(preparation_gate.reached()).await;
        let control_a = preparing_control(&adapter, reference.id);
        let stop_gate = adapter.gates.install(TestPoint::StopWaiting);
        let actor = adapter.clone();
        let target = reference.clone();
        let mut stopper_a = tokio::spawn(async move { actor.stop(target).await });
        bounded(stop_gate.reached()).await;
        preparation_gate.release();
        let error_a = bounded(caller_a).await.unwrap().unwrap_err();
        let outcome_a = bounded(control_a.wait_finished()).await.unwrap();
        input.version += 1;
        adapter.checkpoint(reference.clone(), input).await.unwrap();
        let actor = adapter.clone();
        let target = reference.clone();
        let mut caller_b = tokio::spawn(async move { actor.resume(target).await });
        tokio::select! {_=ready(&marker)=>{},early=&mut caller_b=>panic!("B ended before actual bootstrap: {early:?}")}
        let control_b = preparing_control(&adapter, reference.id);
        let own_a = Arc::ptr_eq(&control_a, &control_b);
        stop_gate.release();
        let before_b_stop =
            tokio::time::timeout(std::time::Duration::from_secs(1), &mut stopper_a).await;
        let completed_a_before_b_stop = before_b_stop.is_ok();
        let unchanged_b = matches!(control_b.preparation.state().unwrap(), Admission::Preparing);
        let snapshot_b = adapter.current(&reference).unwrap();
        let stopper_b = bounded(adapter.stop(reference.clone())).await;
        let error_b = bounded(caller_b).await.unwrap().unwrap_err();
        let stopped_a = match before_b_stop {
            Ok(result) => result.unwrap(),
            Err(_) => bounded(stopper_a).await.unwrap(),
        };
        assert!(
            completed_a_before_b_stop,
            "A must finish without waiting for B's outcome"
        );
        assert!(!own_a);
        assert_eq!(error_a.kind, ErrorKind::StateConflict);
        assert!(matches!(
            outcome_a,
            Outcome::RestoredBeforeAdmission {
                cause: Cause::Cancelled,
                ..
            }
        ));
        let stopped_a = stopped_a.unwrap_err();
        assert_eq!(stopped_a.kind, ErrorKind::StateConflict);
        assert!(stopped_a.message.contains("advanced; factual outcome"));
        assert!(stopped_a.message.contains("cancelled before admission"));
        assert!(
            unchanged_b,
            "captured A never cancels the registry's replacement B"
        );
        assert_eq!(snapshot_b.session.state, SessionState::Starting);
        assert!(
            stopper_b
                .unwrap_err()
                .message
                .contains("exact prior Session restored")
        );
        assert_eq!(error_b.kind, ErrorKind::StateConflict);
        assert_leader_reaped(&marker);
        adapter.release(reference).unwrap();
    }

    #[tokio::test]
    async fn checkpoint_restore_cas_failure_is_unpublished_and_cannot_adopt_another_writer() {
        let mut fixture = ApprovalFixture::new(true).await;
        let (adapter, reference) = fixture.terminal_adapter();
        let original = adapter.current(&reference).unwrap();
        let mut input = fixture.authority.request.input.clone();
        input.version += 1;
        let gate = adapter.gates.install(TestPoint::BeforeCheckpointCommit);
        let actor = adapter.clone();
        let target = reference.clone();
        let caller = tokio::spawn(async move { actor.checkpoint(target, input).await });
        bounded(gate.reached()).await;
        let control = preparing_control(&adapter, reference.id);
        let db = fixture
            .authority
            .request
            .project
            .root
            .parent()
            .unwrap()
            .join("state.sqlite3");
        let mut writer = crate::state::Store::open(&db).unwrap();
        let (mut changed, version) = writer.session(reference.id).unwrap().unwrap();
        changed.recovery["synthetic_second_writer"] = json!(true);
        writer.put_session(&changed, version).unwrap();
        let stop_gate = adapter.gates.install(TestPoint::StopWaiting);
        let actor = adapter.clone();
        let target = reference.clone();
        let stopper = tokio::spawn(async move { actor.stop(target).await });
        bounded(stop_gate.reached()).await;
        gate.release();
        stop_gate.release();
        let launch = bounded(caller).await.unwrap().unwrap_err();
        let stopped = bounded(stopper).await.unwrap().unwrap_err();
        let outcome = bounded(control.wait_finished()).await.unwrap();
        assert_eq!(launch.kind, ErrorKind::StateConflict);
        assert_eq!(stopped.kind, ErrorKind::StateConflict);
        assert!(matches!(
            outcome,
            Outcome::RestoreUnpublished {
                cause: Cause::Cancelled,
                ..
            }
        ));
        assert_eq!(
            serde_json::to_value(writer.session(reference.id).unwrap().unwrap().0).unwrap(),
            serde_json::to_value(changed).unwrap()
        );
        assert_eq!(
            adapter
                .registry()
                .unwrap()
                .get(&reference.id)
                .unwrap()
                .request
                .input
                .version,
            fixture.authority.request.input.version
        );
        assert_ne!(
            serde_json::to_value(writer.session(reference.id).unwrap().unwrap().0).unwrap(),
            serde_json::to_value(original.session).unwrap()
        );
        assert_eq!(
            bounded(adapter.stop(reference.clone()))
                .await
                .unwrap_err()
                .kind,
            ErrorKind::StateConflict
        );
        // The foreign Session was never adopted. Drop only the fixture's private
        // registry bookkeeping; no live native process exists in this checkpoint.
        adapter.registry().unwrap().remove(&reference.id);
    }
    const SYNTHETIC_THREAD: &str = "01a10085-deba-7831-8b1b-f302ca6c6da8";
    fn quoted_path(path: &std::path::Path) -> String {
        format!("'{}'", path.to_str().unwrap().replace('\'', "'\\''"))
    }
    fn wire_fixture(owned: &Fixture, mode: &str) -> (PathBuf, PathBuf) {
        use std::os::unix::fs::PermissionsExt;
        let directory = owned
            .request
            .project
            .root
            .parent()
            .unwrap()
            .join("native-fixture");
        std::fs::create_dir(&directory).unwrap();
        std::fs::write(directory.join("mode"), mode).unwrap();
        let executable = directory.join("synthetic-codex");
        let binary = std::env::current_exe().unwrap();
        let script = format!(
            "#!/bin/sh\nif [ \"$1\" = '--version' ]; then printf '%s\\n' 'codex-cli 0.160.0'; exit 0; fi\nprintf '%s\\n' \"$@\" > {args}\nexport RRX_SYNTHETIC_NATIVE_DIRECTORY={directory}\nprintf '%s' \"$$\" > {leader}\nexec {binary} --exact codex::session::tests::synthetic_owned_native_rpc_child --ignored --nocapture\n",
            args = quoted_path(&directory.join("arguments")),
            directory = quoted_path(&directory),
            leader = quoted_path(&directory.join("leader")),
            binary = quoted_path(&binary)
        );
        std::fs::write(&executable, script).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        (executable, directory)
    }
    fn synthetic_merge(target: &mut Value, source: Value) {
        match (target, source) {
            (Value::Object(target), Value::Object(source)) => {
                for (key, value) in source {
                    synthetic_merge(target.entry(key).or_insert(Value::Null), value);
                }
            }
            (target, source) => *target = source,
        }
    }
    fn journal(directory: &std::path::Path, value: &Value) {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(directory.join("journal"))
            .unwrap();
        writeln!(file, "{value}").unwrap();
    }
    fn journal_values(directory: &std::path::Path) -> Vec<Value> {
        std::fs::read_to_string(directory.join("journal"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
    async fn socket_send(
        socket: &mut tokio_tungstenite::WebSocketStream<tokio::net::UnixStream>,
        value: Value,
    ) {
        socket
            .send(tokio_tungstenite::tungstenite::Message::Text(
                value.to_string().into(),
            ))
            .await
            .unwrap();
    }
    async fn synthetic_terminal(
        socket: &mut tokio_tungstenite::WebSocketStream<tokio::net::UnixStream>,
        turn: &str,
        status: &str,
    ) {
        socket_send(socket,json!({"method":"turn/completed","params":{"threadId":SYNTHETIC_THREAD,"turn":{"id":turn,"status":status}}})).await;
    }

    /// Explicitly synthetic, with no model/authentication or native containment
    /// claim. The real launch path must authenticate this owned child's kernel
    /// peer and preserve its process group throughout protocol/cleanup tests.
    #[tokio::test]
    #[ignore = "owned synthetic subprocess entry; invoked only by private fixture wrapper"]
    async fn synthetic_owned_native_rpc_child() {
        use std::os::unix::fs::PermissionsExt;
        let directory = PathBuf::from(
            std::env::var_os("RRX_SYNTHETIC_NATIVE_DIRECTORY").expect("private fixture directory"),
        );
        let arguments = std::fs::read_to_string(directory.join("arguments")).unwrap();
        let args: Vec<_> = arguments.lines().collect();
        let socket = args
            .iter()
            .find_map(|arg| arg.strip_prefix("unix://"))
            .unwrap();
        let mut config = json!({"sandbox_mode":"workspace-write","approval_policy":"on-request","approvals_reviewer":"user","mcp_servers":{},"web_search":"disabled","features":{},"permissions":{}});
        for pair in args.windows(2) {
            if pair[0] == "--disable" {
                config["features"][pair[1]] = json!(false);
            }
            if pair[0] == "-c" {
                let override_config: toml::Value = toml::from_str(pair[1]).unwrap();
                synthetic_merge(&mut config, serde_json::to_value(override_config).unwrap());
            }
        }
        let listener = tokio::net::UnixListener::bind(socket).unwrap();
        std::fs::set_permissions(socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
        let mut current_turn = String::new();
        let cwd = std::env::current_dir().unwrap();
        loop {
            let frame = if !current_turn.is_empty()
                && std::fs::read_to_string(directory.join("mode")).unwrap() == "ack_hold"
            {
                match tokio::time::timeout(std::time::Duration::from_secs(3), socket.next()).await {
                    Ok(frame) => frame,
                    Err(_) => {
                        // Finite fixture fallback permits mutation assertions only
                        // after the supervisor's actual owned cleanup has finished.
                        synthetic_terminal(&mut socket, &current_turn, "interrupted").await;
                        current_turn.clear();
                        continue;
                    }
                }
            } else {
                socket.next().await
            };
            let Some(Ok(frame)) = frame else {
                break;
            };
            let Ok(text) = frame.to_text() else {
                continue;
            };
            let value: Value = serde_json::from_str(text).unwrap();
            journal(&directory, &value);
            let method = value["method"].as_str().unwrap_or("");
            let id = value["id"].clone();
            let response = match method {
                "initialize" => {
                    json!({"userAgent":"synthetic-codex","codexHome":directory,"platformFamily":"unix","platformOs":std::env::consts::OS})
                }
                "initialized" => continue,
                "config/read" => json!({"config":config}),
                "account/read" => json!({"requiresOpenaiAuth":false,"account":null}),
                "environment/status" => json!({"status":"ready"}),
                "thread/start" | "thread/resume" => {
                    assert_eq!(value["params"]["cwd"], json!(cwd));
                    if method == "thread/resume" {
                        assert_eq!(value["params"]["threadId"], SYNTHETIC_THREAD);
                    }
                    json!({"cwd":cwd,"thread":{"id":SYNTHETIC_THREAD,"cwd":cwd},"activePermissionProfile":{"id":value["params"]["permissions"]},"approvalPolicy":value["params"]["approvalPolicy"],"approvalsReviewer":"user","model":"synthetic-model"})
                }
                "mcpServerStatus/list" => json!({"data":[],"nextCursor":null}),
                "turn/start" => {
                    let number = std::fs::read_to_string(directory.join("turn-counter"))
                        .unwrap_or_else(|_| "0".into())
                        .parse::<u64>()
                        .unwrap()
                        + 1;
                    std::fs::write(directory.join("turn-counter"), number.to_string()).unwrap();
                    current_turn = format!("synthetic-turn-{number}");
                    let mode = std::fs::read_to_string(directory.join("mode")).unwrap();
                    if mode == "ack_lost" {
                        break;
                    }
                    if mode == "ack_timeout" {
                        tokio::time::sleep(std::time::Duration::from_secs(35)).await;
                        continue;
                    }
                    if mode == "ack_hold" {
                        ready(&directory.join("release-ack")).await;
                    }
                    socket_send(
                        &mut socket,
                        json!({"id":id,"result":{"turn":{"id":current_turn}}}),
                    )
                    .await;
                    if mode == "approval" {
                        socket_send(&mut socket,json!({"id":"synthetic-approval","method":"item/commandExecution/requestApproval","params":{"threadId":SYNTHETIC_THREAD,"turnId":current_turn,"itemId":"synthetic-operation","command":"synthetic no-effect operation","cwd":cwd}})).await;
                    } else if mode == "complete" {
                        socket_send(&mut socket,json!({"method":"item/completed","params":{"threadId":SYNTHETIC_THREAD,"turnId":current_turn,"item":{"id":"synthetic-message","type":"agentMessage","text":current_turn}}})).await;
                        synthetic_terminal(&mut socket, &current_turn, "completed").await;
                    }
                    continue;
                }
                "turn/interrupt" => {
                    assert_eq!(
                        value["params"],
                        json!({"threadId":SYNTHETIC_THREAD,"turnId":current_turn})
                    );
                    socket_send(&mut socket, json!({"id":id,"result":{}})).await;
                    synthetic_terminal(&mut socket, &current_turn, "interrupted").await;
                    continue;
                }
                "" if id == "synthetic-approval" => {
                    assert_eq!(value["result"]["decision"], "accept");
                    synthetic_terminal(&mut socket, &current_turn, "completed").await;
                    continue;
                }
                _ => panic!("unexpected synthetic RPC method: {method}"),
            };
            socket_send(&mut socket, json!({"id":id,"result":response})).await;
        }
    }

    async fn terminal_status(adapter: &CodexAdapter, reference: &SessionRef) -> SessionStatus {
        let mut status = adapter.subscribe(reference.clone()).unwrap();
        bounded(async {
            loop {
                let snapshot = status.borrow_and_update().clone();
                if snapshot.terminal() {
                    let control = adapter
                        .registry()
                        .unwrap()
                        .get(&reference.id)
                        .unwrap()
                        .control
                        .clone();
                    control.wait_finished().await.unwrap();
                    return snapshot;
                }
                status.changed().await.unwrap();
            }
        })
        .await
    }

    #[tokio::test]
    async fn terminal_publication_must_finish_before_next_attempt_and_has_no_late_watch_write() {
        let owned = Fixture::new(true);
        let (executable, directory) = wire_fixture(&owned, "complete");
        let adapter =
            Arc::new(CodexAdapter::new("codex".into(), executable, owned.store.clone()).unwrap());
        let gate = adapter.gates.install(TestPoint::AfterTerminalPublication);
        let session = bounded(adapter.start(owned.request.clone())).await.unwrap();
        let reference = SessionRef::from(&session);
        bounded(gate.reached()).await;
        let control = preparing_control(&adapter, reference.id);
        let mut watch = adapter.subscribe(reference.clone()).unwrap();
        let final_a = watch.borrow_and_update().clone();
        let version = owned
            .store
            .lock()
            .unwrap()
            .session(reference.id)
            .unwrap()
            .unwrap()
            .1;
        let mut input = owned.request.input.clone();
        input.version += 1;
        let checkpoint = adapter.checkpoint(reference.clone(), input.clone()).await;
        let resume = adapter.resume(reference.clone()).await;
        let before = owned
            .store
            .lock()
            .unwrap()
            .session(reference.id)
            .unwrap()
            .unwrap();
        gate.release();
        bounded(control.wait_finished()).await.unwrap();
        let late_watch_write = watch.has_changed().unwrap();
        assert_eq!(final_a.session.state, SessionState::Exited);
        assert_eq!(checkpoint.unwrap_err().kind, ErrorKind::StateConflict);
        assert_eq!(resume.unwrap_err().kind, ErrorKind::StateConflict);
        assert_eq!(before.1, version);
        assert_eq!(
            serde_json::to_value(before.0).unwrap(),
            serde_json::to_value(&final_a.session).unwrap()
        );
        assert!(
            !late_watch_write,
            "the final Store/watch publication is this attempt's last shared write"
        );
        assert_leader_reaped(&directory.join("leader"));
        adapter.checkpoint(reference.clone(), input).await.unwrap();
        bounded(adapter.resume(reference.clone())).await.unwrap();
        let final_b = terminal_status(&adapter, &reference).await;
        assert_eq!(final_b.session.state, SessionState::Exited);
        assert_eq!(final_b.session.recovery["input_version"], 2);
        adapter.release(reference).unwrap();
    }

    #[tokio::test]
    async fn cancel_before_initial_resume_or_checkpoint_write_needs_no_restore_cas_or_audit() {
        for resume in [false, true] {
            let mut fixture = ApprovalFixture::new(true).await;
            let (adapter, reference) = fixture.terminal_adapter();
            let mut input = fixture.authority.request.input.clone();
            input.version += 1;
            if resume {
                adapter
                    .checkpoint(reference.clone(), input.clone())
                    .await
                    .unwrap();
            }
            let original = adapter.current(&reference).unwrap();
            let version = fixture
                .reservation
                .store
                .lock()
                .unwrap()
                .session(reference.id)
                .unwrap()
                .unwrap()
                .1;
            let events = fixture
                .reservation
                .store
                .lock()
                .unwrap()
                .events(&reference.scope, 0, 1000)
                .unwrap()
                .len();
            let db = fixture
                .authority
                .request
                .project
                .root
                .parent()
                .unwrap()
                .join("state.sqlite3");
            let writer = rusqlite::Connection::open(db).unwrap();
            writer.execute_batch("CREATE TRIGGER reject_unnecessary_session_write BEFORE UPDATE ON records WHEN NEW.kind='session' BEGIN SELECT RAISE(ABORT,'unnecessary session write'); END;").unwrap();
            let gate = adapter.gates.install(TestPoint::BeforeInitialPersist);
            let actor = adapter.clone();
            let target = reference.clone();
            let caller = tokio::spawn(async move {
                if resume {
                    actor.resume(target).await.map(|_| ())
                } else {
                    actor.checkpoint(target, input).await
                }
            });
            bounded(gate.reached()).await;
            let control = preparing_control(&adapter, reference.id);
            let actor = adapter.clone();
            let target = reference.clone();
            let stopper = tokio::spawn(async move { actor.stop(target).await });
            bounded(async {
                while !matches!(control.preparation.state().unwrap(), Admission::Cancelled) {
                    tokio::task::yield_now().await;
                }
            })
            .await;
            gate.release();
            let error = bounded(caller).await.unwrap().unwrap_err();
            let stopped = bounded(stopper).await.unwrap().unwrap_err();
            let outcome = bounded(control.wait_finished()).await.unwrap();
            let (stored, after) = fixture
                .reservation
                .store
                .lock()
                .unwrap()
                .session(reference.id)
                .unwrap()
                .unwrap();
            let events_after = fixture
                .reservation
                .store
                .lock()
                .unwrap()
                .events(&reference.scope, 0, 1000)
                .unwrap()
                .len();
            assert_eq!(error.kind, ErrorKind::StateConflict);
            assert!(stopped.message.contains("exact prior Session restored"));
            assert!(matches!(
                outcome,
                Outcome::RestoredBeforeAdmission {
                    cause: Cause::Cancelled,
                    ..
                }
            ));
            assert_eq!(after, version);
            assert_eq!(events_after, events);
            assert_eq!(
                serde_json::to_value(stored).unwrap(),
                serde_json::to_value(&original.session).unwrap()
            );
            assert!(fixture.evidence.lock().unwrap().completed);
            assert_eq!(
                bounded(adapter.stop(reference.clone()))
                    .await
                    .unwrap()
                    .session
                    .state,
                SessionState::Exited
            );
            adapter.release(reference).unwrap();
        }
    }

    #[tokio::test]
    async fn abnormal_supervisor_drop_clears_the_installed_live_approval_journal() {
        let owned = Fixture::new(true);
        let (executable, directory) = wire_fixture(&owned, "approval");
        let adapter = CodexAdapter::new("codex".into(), executable, owned.store.clone())
            .unwrap()
            .with_runtime_broker();
        let session = bounded(adapter.start(owned.request.clone())).await.unwrap();
        let reference = SessionRef::from(&session);
        let mut status = adapter.subscribe(reference.clone()).unwrap();
        bounded(async {
            loop {
                if status.borrow_and_update().session.state == SessionState::WaitingApproval {
                    break;
                }
                status.changed().await.unwrap();
            }
        })
        .await;
        let pending = bounded(adapter.pending_approvals(reference.clone()))
            .await
            .unwrap();
        let (control, evidence) = {
            let registry = adapter.registry().unwrap();
            let entry = registry.get(&reference.id).unwrap();
            (entry.control.clone(), entry.evidence.clone())
        };
        control.abort_owned_task();
        bounded(control.wait_finished()).await.unwrap();
        // Abrupt task destruction is Lost, not a normal verified cleanup. Keep
        // the fixture runtime alive until its owned child's reaper completes.
        bounded(async {
            loop {
                let pid = std::fs::read_to_string(directory.join("leader")).unwrap();
                let output = std::process::Command::new("/bin/ps")
                    .args(["-p", &pid, "-o", "pid="])
                    .output()
                    .unwrap();
                if output.stdout.is_empty() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await;
        let current = adapter.current(&reference).unwrap();
        let evidence = evidence.lock().unwrap();
        assert_eq!(pending["requests"].as_array().unwrap().len(), 1);
        assert_eq!(current.session.state, SessionState::Lost);
        assert!(
            evidence.pending.is_none(),
            "Drop must clear the journal installed for this actual turn"
        );
        assert!(!evidence.completed);
        drop(evidence);
        assert!(
            bounded(adapter.pending_approvals(reference.clone()))
                .await
                .is_err()
        );
        adapter.release(reference).unwrap();
    }

    #[tokio::test]
    async fn preparation_cleanup_uncertainty_returns_lost_and_retains_first_cause_and_detail() {
        for cancelled in [false, true] {
            let mut fixture = ApprovalFixture::new(true).await;
            let (adapter, reference) = fixture.terminal_adapter();
            let previous = adapter.current(&reference).unwrap();
            let reservation = &mut fixture.reservation;
            reservation.resume_publication = Some(ResumePublication {
                previous,
                sender: fixture.sender.clone(),
                evidence: fixture.evidence.clone(),
            });
            reservation.attempt = Control::new(None).0;
            reservation.session.state = SessionState::Starting;
            reservation.session.pid = Some(42);
            reservation.persist().unwrap();
            reservation.ownership.group().store(true, Ordering::SeqCst);
            let primary = if cancelled {
                assert!(reservation.attempt.preparation.cancel());
                Cause::Cancelled.error()
            } else {
                reservation
                    .attempt
                    .preparation
                    .failed(failure(ErrorKind::Timeout, "synthetic primary timeout"))
            };
            let composite = failure_after_cleanup(
                primary,
                Err(failure(
                    ErrorKind::SessionLost,
                    "synthetic selected-group inspection unavailable",
                )),
            );
            let error = reservation.finish_preparation_error(composite);
            let status = adapter.current(&reference).unwrap();
            let outcome = reservation.attempt.wait_finished().await.unwrap();
            let events = reservation
                .store
                .lock()
                .unwrap()
                .events(&reference.scope, 0, 1000)
                .unwrap();
            let audit = events
                .iter()
                .rev()
                .find(|event| event.kind == "codex.preparation.finished")
                .unwrap();
            assert_eq!(error.kind, ErrorKind::SessionLost);
            assert!(
                error
                    .message
                    .contains("synthetic selected-group inspection unavailable")
            );
            assert!(
                status
                    .failure
                    .as_deref()
                    .unwrap()
                    .contains("synthetic selected-group inspection unavailable")
            );
            assert_eq!(status.session.state, SessionState::Lost);
            assert_eq!(status.session.pid, Some(42));
            assert!(matches!(
                outcome,
                Outcome::Lost {
                    publication_result: Ok(_),
                    ..
                }
            ));
            assert!(match outcome.cause().unwrap() {
                Cause::Cancelled => cancelled,
                Cause::Failed(kind, _) => !cancelled && *kind == ErrorKind::Timeout,
            });
            assert_eq!(
                audit.data["uncertainty_reason"],
                "owned process cleanup is unverified"
            );
            assert_eq!(audit.data["cleanup_detail_retained"], true);
            assert!(
                audit.data["later_failure_kinds"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("SessionLost"))
            );
            assert!(
                !audit
                    .data
                    .to_string()
                    .contains("synthetic selected-group inspection unavailable"),
                "audit must not copy private error payloads"
            );
            adapter.release(reference).unwrap();
        }
    }

    #[tokio::test]
    async fn registered_consumption_queues_stop_through_ack_and_native_missing_ack_stays_lost() {
        for mode in ["ack_hold", "ack_lost"] {
            let owned = Fixture::new(true);
            let (executable, directory) = wire_fixture(&owned, mode);
            let adapter = Arc::new(
                CodexAdapter::new("codex".into(), executable, owned.store.clone()).unwrap(),
            );
            let pre_admission = adapter.gates.install(TestPoint::BeforeDispatch);
            let actor = adapter.clone();
            let request = owned.request.clone();
            let mut caller = tokio::spawn(async move { actor.start(request).await });
            bounded(pre_admission.reached()).await;
            let (reference, control) = {
                let registry = adapter.registry().unwrap();
                let entry = registry.values().next().unwrap();
                (
                    SessionRef::from(&entry.status.borrow().session),
                    entry.control.clone(),
                )
            };
            let mut status = adapter.subscribe(reference.clone()).unwrap();
            assert_eq!(status.borrow().session.state, SessionState::Starting);
            pre_admission.release();
            if mode == "ack_hold" {
                bounded(async {
                    while !journal_values(&directory)
                        .iter()
                        .any(|value| value["method"] == "turn/start")
                    {
                        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                    }
                })
                .await;
                assert!(control.consumed());
                let stop_gate = adapter.gates.install(TestPoint::StopWaiting);
                let actor = adapter.clone();
                let target = reference.clone();
                let stopper = tokio::spawn(async move { actor.stop(target).await });
                bounded(stop_gate.reached()).await;
                std::fs::write(directory.join("release-ack"), "").unwrap();
                stop_gate.release();
                bounded(&mut caller).await.unwrap().unwrap();
                let stopped = bounded(stopper).await.unwrap().unwrap();
                assert_eq!(stopped.session.state, SessionState::Stopped);
                assert_eq!(stopped.session.pid, None);
                let final_snapshot = bounded(control.wait_finished())
                    .await
                    .unwrap()
                    .snapshot()
                    .unwrap()
                    .clone();
                assert_eq!(
                    serde_json::to_value(final_snapshot.session).unwrap(),
                    serde_json::to_value(stopped.session).unwrap()
                );
                assert_eq!(
                    journal_values(&directory)
                        .iter()
                        .filter(|value| value["method"] == "turn/interrupt")
                        .count(),
                    1
                );
                assert!(status.changed().await.is_ok());
                assert_eq!(
                    status.borrow().session.state,
                    SessionState::Stopped,
                    "same preparation subscription receives final supervision publication"
                );
            } else {
                let error = bounded(&mut caller).await.unwrap().unwrap_err();
                assert!(
                    matches!(
                        error.kind,
                        ErrorKind::ProcessFailure
                            | ErrorKind::SessionLost
                            | ErrorKind::ParseFailure
                    ),
                    "actual missing-ack transport error: {error:?}"
                );
                let outcome = bounded(control.wait_finished()).await.unwrap();
                assert!(matches!(
                    outcome,
                    Outcome::Lost {
                        publication_result: Ok(_),
                        ..
                    }
                ));
                let lost = adapter.current(&reference).unwrap();
                assert_eq!(lost.session.state, SessionState::Lost);
                assert_eq!(lost.session.pid, None);
                assert_eq!(lost.session.recovery["native_dispatch_unobserved"], true);
                assert_eq!(
                    bounded(adapter.stop(reference.clone()))
                        .await
                        .unwrap_err()
                        .kind,
                    ErrorKind::SessionLost
                );
            }
            assert_eq!(
                journal_values(&directory)
                    .iter()
                    .filter(|value| value["method"] == "turn/start")
                    .count(),
                1
            );
            adapter.release(reference).unwrap();
        }
    }

    #[tokio::test]
    async fn resumed_supervised_transition_allows_actual_approval_usage_and_exact_one_reply() {
        let owned = Fixture::new(true);
        let (executable, directory) = wire_fixture(&owned, "complete");
        let adapter = CodexAdapter::new("codex".into(), executable, owned.store.clone())
            .unwrap()
            .with_runtime_broker();
        let session = bounded(adapter.start(owned.request.clone())).await.unwrap();
        let reference = SessionRef::from(&session);
        let first = terminal_status(&adapter, &reference).await;
        assert_eq!(first.session.state, SessionState::Exited);
        assert!(adapter.transport_succeeded(&first));
        std::fs::write(directory.join("mode"), "approval").unwrap();
        let mut input = owned.request.input.clone();
        input.version += 1;
        input.payload = "new explicit continuation".into();
        adapter
            .checkpoint(reference.clone(), input.clone())
            .await
            .unwrap();
        let resumed = bounded(adapter.resume(reference.clone())).await.unwrap();
        let mut status = adapter.subscribe(reference.clone()).unwrap();
        bounded(async {
            loop {
                if status.borrow_and_update().session.state == SessionState::WaitingApproval {
                    break;
                }
                status.changed().await.unwrap();
            }
        })
        .await;
        assert_eq!(resumed.native_ref, first.session.native_ref);
        let transition_released = !adapter
            .registry()
            .unwrap()
            .get(&reference.id)
            .unwrap()
            .transition
            .load(Ordering::SeqCst);
        let usage = bounded(adapter.usage(reference.clone(), "execution".into(), None)).await;
        let pending = bounded(adapter.pending_approvals(reference.clone())).await;
        let (usage, pending) = match (usage, pending) {
            (Ok(usage), Ok(pending)) => (usage, pending),
            (usage, pending) => {
                let _ = bounded(adapter.stop(reference.clone())).await;
                panic!(
                    "supervised observation must be available after transfer: usage={usage:?},pending={pending:?}"
                );
            }
        };
        assert_eq!(usage.input_tokens, None);
        assert_eq!(pending["requests"].as_array().unwrap().len(), 1);
        let decision = json!({"native_turn":pending["native_turn"],"request_id":pending["requests"][0]["id"],"decision":"Approve","operation_hash":pending["requests"][0]["operation_hash"]});
        bounded(adapter.submit_approval(reference.clone(), decision.clone()))
            .await
            .unwrap();
        let second = terminal_status(&adapter, &reference).await;
        assert_eq!(second.session.state, SessionState::Exited);
        assert!(
            transition_released,
            "transition releases with supervised installation, not terminal completion"
        );
        assert_eq!(second.session.recovery["input_version"], input.version);
        assert!(adapter.transport_succeeded(&second));
        assert!(
            bounded(adapter.submit_approval(reference.clone(), decision))
                .await
                .is_err()
        );
        let values = journal_values(&directory);
        assert_eq!(values.iter().filter(|value|value["id"]=="synthetic-approval" && value.get("method").is_none()).count(),1);
        let starts: Vec<_> = values
            .iter()
            .filter(|value| value["method"] == "turn/start")
            .collect();
        assert_eq!(starts.len(), 2);
        assert_eq!(starts[1]["params"]["input"][0]["text"], input.payload);
        adapter.release(reference).unwrap();
    }

    #[tokio::test]
    async fn post_consumption_caller_drop_keeps_the_registered_stop_channel_idle() {
        let owned = Fixture::new(true);
        let (executable, directory) = wire_fixture(&owned, "ack_hold");
        let adapter =
            Arc::new(CodexAdapter::new("codex".into(), executable, owned.store.clone()).unwrap());
        let gate = adapter.gates.install(TestPoint::BeforeDispatch);
        let actor = adapter.clone();
        let request = owned.request.clone();
        let caller = tokio::spawn(async move { actor.start(request).await });
        bounded(gate.reached()).await;
        let (reference, control) = {
            let registry = adapter.registry().unwrap();
            let entry = registry.values().next().unwrap();
            (
                SessionRef::from(&entry.status.borrow().session),
                entry.control.clone(),
            )
        };
        gate.release();
        bounded(async {
            while !journal_values(&directory)
                .iter()
                .any(|value| value["method"] == "turn/start")
            {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        })
        .await;
        caller.abort();
        let _ = caller.await;
        std::fs::write(directory.join("release-ack"), "").unwrap();
        bounded(async {
            let mut phase = control.subscribe();
            loop {
                if matches!(
                    *phase.borrow_and_update(),
                    Phase::Supervised | Phase::Finished(_)
                ) {
                    break;
                }
                phase.changed().await.unwrap();
            }
        })
        .await;
        // A finite quiet window distinguishes the preserved live receiver from
        // replacing it with a closed channel that injects an unsolicited stop.
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let unsolicited = journal_values(&directory)
            .iter()
            .filter(|value| value["method"] == "turn/interrupt")
            .count();
        let stopped = bounded(adapter.stop(reference.clone())).await.unwrap();
        bounded(control.wait_finished()).await.unwrap();
        assert_leader_reaped(&directory.join("leader"));
        assert_eq!(
            unsolicited, 0,
            "caller drop cannot retire the live stop receiver"
        );
        assert_eq!(stopped.session.state, SessionState::Stopped);
        assert_eq!(
            journal_values(&directory)
                .iter()
                .filter(|value| value["method"] == "turn/interrupt")
                .count(),
            1
        );
        adapter.release(reference).unwrap();
    }
    #[tokio::test]
    async fn actual_failed_admission_cas_latches_cause_before_cleanup_and_survives_stop_caller_drop()
     {
        for database_failure in [false, true] {
            let owned = Fixture::new(true);
            let (executable, directory) = wire_fixture(&owned, "complete");
            let adapter = Arc::new(
                CodexAdapter::new("codex".into(), executable, owned.store.clone()).unwrap(),
            );
            let admission_gate = adapter.gates.install(TestPoint::BeforeDispatch);
            let actor = adapter.clone();
            let request = owned.request.clone();
            let caller = tokio::spawn(async move { actor.start(request).await });
            bounded(admission_gate.reached()).await;
            let (reference, control) = {
                let registry = adapter.registry().unwrap();
                let entry = registry.values().next().unwrap();
                (
                    SessionRef::from(&entry.status.borrow().session),
                    entry.control.clone(),
                )
            };
            let db = owned
                .request
                .project
                .root
                .parent()
                .unwrap()
                .join("state.sqlite3");
            if database_failure {
                let writer = rusqlite::Connection::open(&db).unwrap();
                writer.execute_batch("CREATE TRIGGER synthetic_admission_error BEFORE UPDATE ON records WHEN NEW.kind='session' AND json_extract(NEW.body,'$.data.recovery.dispatch_intent.consumed')=1 BEGIN SELECT RAISE(ABORT,'synthetic admission write failure'); END;").unwrap();
            } else {
                let mut writer = crate::state::Store::open(&db).unwrap();
                let mut task = writer
                    .task(owned.request.scope.task_id.unwrap())
                    .unwrap()
                    .unwrap();
                task.title = "actual owner changed after final preflight".into();
                writer.put_task(&mut task).unwrap();
            }
            let cleanup_gate = adapter.gates.install(TestPoint::BeforeNativeCleanup);
            admission_gate.release();
            bounded(cleanup_gate.reached()).await;
            let before_cancel = control.preparation.state().unwrap();
            let finished_before_cleanup =
                matches!(*control.subscribe().borrow(), Phase::Finished(_));
            caller.abort();
            let _ = caller.await;
            let stop_gate = adapter.gates.install(TestPoint::StopWaiting);
            let actor = adapter.clone();
            let target = reference.clone();
            let stopper = tokio::spawn(async move { actor.stop(target).await });
            bounded(stop_gate.reached()).await;
            let after_cancel = control.preparation.state().unwrap();
            let before_cleanup = adapter.current(&reference).unwrap();
            cleanup_gate.release();
            stop_gate.release();
            let stopped = bounded(stopper).await.unwrap();
            // Preserve fixture ownership even if a mutant completes the private
            // phase too early: the real final shared publication follows cleanup.
            terminal_status(&adapter, &reference).await;
            let stopped = stopped.unwrap_err();
            let outcome = bounded(control.wait_finished()).await.unwrap();
            assert!(
                !finished_before_cleanup,
                "Finished cannot precede owned cleanup/publication"
            );
            let expected = if database_failure {
                ErrorKind::StateFailure
            } else {
                ErrorKind::StateConflict
            };
            assert!(
                matches!(before_cancel, Admission::Failing(Cause::Failed(kind,_)) if kind==expected),
                "failed CAS must latch before held cleanup"
            );
            assert!(
                matches!(after_cancel,Admission::Failing(Cause::Failed(kind,_)) if kind==expected)
            );
            assert_eq!(before_cleanup.session.state, SessionState::Starting);
            assert_eq!(stopped.kind, expected);
            assert!(!stopped.message.contains("cancelled"));
            assert!(
                matches!(outcome,Outcome::FailedBeforeAdmission{cause:Cause::Failed(kind,_),..} if kind==expected)
            );
            let stored = adapter.current(&reference).unwrap();
            assert_eq!(stored.session.state, SessionState::Failed);
            assert_eq!(stored.session.pid, None);
            assert!(stored.session.recovery.get("dispatch_intent").is_none());
            assert!(!journal_values(&directory).iter().any(|value| matches!(
                value["method"].as_str(),
                Some("turn/start" | "turn/interrupt")
            )));
            assert_leader_reaped(&directory.join("leader"));
            let events = owned
                .store
                .lock()
                .unwrap()
                .events(&owned.request.scope, 0, 1000)
                .unwrap();
            assert!(
                events
                    .iter()
                    .any(|event| event.kind == "codex.preparation.finished"
                        && event.data["cause"] == "failed")
            );
            adapter.release(reference).unwrap();
        }
    }

    #[tokio::test]
    async fn actual_resume_pre_admission_cancel_has_zero_new_wire_and_restores_prior_turn_output() {
        let owned = Fixture::new(true);
        let (executable, directory) = wire_fixture(&owned, "complete");
        let adapter =
            Arc::new(CodexAdapter::new("codex".into(), executable, owned.store.clone()).unwrap());
        let first = bounded(adapter.start(owned.request.clone())).await.unwrap();
        let reference = SessionRef::from(&first);
        let previous = terminal_status(&adapter, &reference).await;
        let mut input = owned.request.input.clone();
        input.version += 1;
        input.payload = "only explicit new input".into();
        adapter
            .checkpoint(reference.clone(), input.clone())
            .await
            .unwrap();
        let gate = adapter.gates.install(TestPoint::BeforeDispatch);
        let actor = adapter.clone();
        let target = reference.clone();
        let caller = tokio::spawn(async move { actor.resume(target).await });
        bounded(gate.reached()).await;
        let starting = adapter.current(&reference).unwrap();
        let control = preparing_control(&adapter, reference.id);
        let stop_gate = adapter.gates.install(TestPoint::StopWaiting);
        let actor = adapter.clone();
        let target = reference.clone();
        let stopper = tokio::spawn(async move { actor.stop(target).await });
        bounded(stop_gate.reached()).await;
        gate.release();
        stop_gate.release();
        let resumed = bounded(caller).await.unwrap().unwrap_err();
        let stopped = bounded(stopper).await.unwrap().unwrap_err();
        let outcome = bounded(control.wait_finished()).await.unwrap();
        assert!(
            starting.stdout.is_empty(),
            "preparation cannot relabel historical output as this turn's output"
        );
        assert_eq!(resumed.kind, ErrorKind::StateConflict);
        assert!(stopped.message.contains("exact prior Session restored"));
        assert!(matches!(
            outcome,
            Outcome::RestoredBeforeAdmission {
                cause: Cause::Cancelled,
                ..
            }
        ));
        let current = adapter.current(&reference).unwrap();
        assert_eq!(
            serde_json::to_value(&current.session).unwrap(),
            serde_json::to_value(&previous.session).unwrap()
        );
        assert_eq!(current.stdout, previous.stdout);
        assert!(adapter.transport_succeeded(&current));
        assert_eq!(
            journal_values(&directory)
                .iter()
                .filter(|value| value["method"] == "turn/start")
                .count(),
            1
        );
        assert_leader_reaped(&directory.join("leader"));
        // Cancellation retains the explicit pending input for a later, fresh
        // owned resume; it never replays the previous mutating payload.
        bounded(adapter.resume(reference.clone())).await.unwrap();
        let actual = terminal_status(&adapter, &reference).await;
        assert_eq!(actual.session.recovery["input_version"], input.version);
        assert_eq!(actual.stdout, b"synthetic-turn-2");
        let values = journal_values(&directory);
        let starts: Vec<_> = values
            .iter()
            .filter(|value| value["method"] == "turn/start")
            .collect();
        assert_eq!(starts.len(), 2);
        assert_eq!(starts[1]["params"]["input"][0]["text"], input.payload);
        adapter.release(reference).unwrap();
    }
    #[tokio::test]
    async fn actual_registered_review_git_cancel_reaps_owned_hook_before_failed_without_native_spawn()
     {
        use std::os::unix::fs::PermissionsExt;
        let mut owned = Fixture::new(true);
        crate::git::WorktreeManager::lock_review(
            &mut owned.store.lock().unwrap(),
            owned.request.scope.task_id.unwrap(),
            &owned.request.input.revision,
            "synthetic immutable review",
        )
        .unwrap();
        owned.request.role = SessionRole::Reviewer;
        owned.request.input.kind = crate::adapter::InputKind::ReviewBundle;
        let parent = owned.request.project.root.parent().unwrap();
        let hook = parent.join("synthetic-fsmonitor");
        let marker = parent.join("git-hook-ready");
        std::fs::write(
            &hook,
            format!(
                "#!/bin/sh\nprintf '%s' \"$$\" > {}\nexec /bin/sleep 30\n",
                quoted_path(&marker)
            ),
        )
        .unwrap();
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o700)).unwrap();
        let configured = std::process::Command::new("git")
            .current_dir(&owned.request.worktree)
            .args(["config", "core.fsmonitor", hook.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(configured.status.success());
        let (executable, native_marker) = bootstrap_fixture(&owned);
        let adapter =
            Arc::new(CodexAdapter::new("codex".into(), executable, owned.store.clone()).unwrap());
        let actor = adapter.clone();
        let request = owned.request.clone();
        let mut caller = tokio::spawn(async move { actor.start(request).await });
        tokio::select! {_=ready(&marker)=>{},early=&mut caller=>panic!("Git fixture ended before hook readiness: {early:?}")}
        let reference = {
            let registry = adapter.registry().unwrap();
            SessionRef::from(&registry.values().next().unwrap().status.borrow().session)
        };
        let control = preparing_control(&adapter, reference.id);
        let stopped = bounded(adapter.stop(reference.clone())).await;
        let launch = bounded(caller).await.unwrap();
        let outcome = bounded(control.wait_finished()).await.unwrap();
        assert_eq!(stopped.unwrap().session.state, SessionState::Failed);
        assert_eq!(launch.unwrap_err().kind, ErrorKind::StateConflict);
        assert!(matches!(
            outcome,
            Outcome::FailedBeforeAdmission {
                cause: Cause::Cancelled,
                ..
            }
        ));
        assert_leader_reaped(&marker);
        assert!(
            !native_marker.exists(),
            "cancel during real Git preflight prevents later native bootstrap"
        );
        assert_eq!(adapter.current(&reference).unwrap().session.pid, None);
        adapter.release(reference).unwrap();
    }

    #[tokio::test]
    async fn actual_consumed_rpc_timeout_after_caller_drop_keeps_owned_task_and_lost_completion() {
        let owned = Fixture::new(true);
        let (executable, directory) = wire_fixture(&owned, "ack_timeout");
        let adapter =
            Arc::new(CodexAdapter::new("codex".into(), executable, owned.store.clone()).unwrap());
        let gate = adapter.gates.install(TestPoint::BeforeDispatch);
        let actor = adapter.clone();
        let request = owned.request.clone();
        let caller = tokio::spawn(async move { actor.start(request).await });
        bounded(gate.reached()).await;
        let (reference, control) = {
            let registry = adapter.registry().unwrap();
            let entry = registry.values().next().unwrap();
            (
                SessionRef::from(&entry.status.borrow().session),
                entry.control.clone(),
            )
        };
        gate.release();
        bounded(async {
            while !journal_values(&directory)
                .iter()
                .any(|value| value["method"] == "turn/start")
            {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        })
        .await;
        let started = tokio::time::Instant::now();
        caller.abort();
        let _ = caller.await;
        assert!(
            control.consumed(),
            "caller drop loses cancellation after committed consumption"
        );
        let stop_gate = adapter.gates.install(TestPoint::StopWaiting);
        let actor = adapter.clone();
        let target = reference.clone();
        let stopper = tokio::spawn(async move { actor.stop(target).await });
        bounded(stop_gate.reached()).await;
        stop_gate.release();
        // Exercise the existing native 30s RPC deadline, without replacing it
        // with a shorter test-only timeout or claiming ACK as terminal proof.
        let stopped = tokio::time::timeout(std::time::Duration::from_secs(40), stopper)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        let outcome = control.wait_finished().await.unwrap();
        assert_eq!(stopped.kind, ErrorKind::SessionLost);
        assert!(started.elapsed() >= std::time::Duration::from_secs(29));
        assert!(matches!(
            &outcome,
            Outcome::Lost {
                cause: Cause::Failed(ErrorKind::Timeout, _),
                publication_result: Ok(_)
            }
        ));
        let snapshot = adapter.current(&reference).unwrap();
        assert_eq!(snapshot.session.state, SessionState::Lost);
        assert_eq!(snapshot.session.pid, None);
        assert_eq!(
            snapshot.session.recovery["native_dispatch_unobserved"],
            true
        );
        assert!(
            snapshot
                .failure
                .unwrap()
                .contains("native RPC response timed out")
        );
        assert_eq!(
            journal_values(&directory)
                .iter()
                .filter(|value| value["method"] == "turn/start")
                .count(),
            1
        );
        assert!(
            !journal_values(&directory)
                .iter()
                .any(|value| value["method"] == "turn/interrupt"),
            "queued stop cannot send an interrupt before the unobserved ACK supplies a turn identity"
        );
        assert_leader_reaped(&directory.join("leader"));
        adapter.release(reference).unwrap();
    }
    #[tokio::test]
    async fn actual_admission_cas_holds_cancellation_order_through_a_second_sqlite_writer() {
        let mut fixture = ApprovalFixture::new(true).await;
        let db = fixture
            .authority
            .request
            .project
            .root
            .parent()
            .unwrap()
            .join("state.sqlite3");
        let writer = rusqlite::Connection::open(db).unwrap();
        writer.execute_batch("BEGIN IMMEDIATE").unwrap();
        let control = fixture.reservation.attempt.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let attempted = Arc::new(AtomicBool::new(false));
        let snapshot = &fixture.authority.snapshot;
        let request = &fixture.authority.request;
        let reservation = &mut fixture.reservation;
        let (admitted, cancellation, entered) = std::thread::scope(|threads| {
            let admission = threads.spawn(|| reservation.admit_dispatch(snapshot, request));
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
            while !control.cas_entered.load(Ordering::SeqCst)
                && std::time::Instant::now() < deadline
            {
                std::thread::yield_now();
            }
            let entered = control.cas_entered.load(Ordering::SeqCst);
            let cancellation = threads.spawn({
                let control = control.clone();
                let attempted = attempted.clone();
                let cancelled = cancelled.clone();
                move || {
                    attempted.store(true, Ordering::SeqCst);
                    let won = control.preparation.cancel();
                    cancelled.store(won, Ordering::SeqCst);
                    won
                }
            });
            while !attempted.load(Ordering::SeqCst) {
                std::thread::yield_now();
            }
            // SQL is held on the independent connection. A cancellation attempt
            // runs concurrently with the actual consumer's blocked Store CAS.
            std::thread::sleep(std::time::Duration::from_millis(50));
            writer.execute_batch("COMMIT").unwrap();
            (
                admission.join().unwrap(),
                cancellation.join().unwrap(),
                entered,
            )
        });
        assert!(
            entered,
            "actual consumer reached its Store CAS before cancellation"
        );
        admitted.unwrap();
        assert!(
            !cancellation,
            "cancel cannot win inside a consumed-input CAS critical section"
        );
        assert!(!cancelled.load(Ordering::SeqCst));
        assert!(control.consumed());
        assert_eq!(
            fixture.reservation.session.recovery["dispatch_intent"]["consumed"],
            true
        );
        let stored = fixture
            .reservation
            .store
            .lock()
            .unwrap()
            .session(fixture.reservation.session.id)
            .unwrap()
            .unwrap()
            .0;
        assert_eq!(
            stored.recovery["dispatch_intent"],
            fixture.reservation.session.recovery["dispatch_intent"]
        );
        // The opposite order rejects the actual consumer before any second write.
        let previous = fixture.reservation.version;
        let (cancelled_control, _) = Control::new(None);
        assert!(cancelled_control.preparation.cancel());
        fixture.reservation.attempt = cancelled_control;
        let error = fixture
            .reservation
            .admit_dispatch(&fixture.authority.snapshot, &fixture.authority.request)
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::StateConflict);
        assert_eq!(fixture.reservation.version, previous);
        assert_eq!(
            fixture
                .reservation
                .store
                .lock()
                .unwrap()
                .session(fixture.reservation.session.id)
                .unwrap()
                .unwrap()
                .0
                .recovery["dispatch_intent"],
            stored.recovery["dispatch_intent"]
        );
        // Observe the actual consumer's failure before any outer async launch
        // helper can redundantly latch it. Failed CAS must already be terminal
        // under the same admission mutex when admit_dispatch returns.
        writer.execute_batch("CREATE TRIGGER synthetic_dispatch_error BEFORE UPDATE ON records WHEN NEW.kind='session' AND json_extract(NEW.body,'$.data.recovery.dispatch_intent.consumed')=1 BEGIN SELECT RAISE(ABORT,'synthetic dispatch write failure'); END;").unwrap();
        let (failed_control, _) = Control::new(None);
        fixture.reservation.attempt = failed_control.clone();
        let error = fixture
            .reservation
            .admit_dispatch(&fixture.authority.snapshot, &fixture.authority.request)
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::StateFailure);
        assert!(matches!(
            failed_control.preparation.state().unwrap(),
            Admission::Failing(Cause::Failed(ErrorKind::StateFailure, _))
        ));
        assert!(!failed_control.preparation.cancel());
        assert_eq!(fixture.reservation.version, previous);
        assert_eq!(
            fixture.reservation.session.recovery["dispatch_intent"],
            stored.recovery["dispatch_intent"]
        );
    }
}
