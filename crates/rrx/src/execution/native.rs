//! Host-native sessions admitted by result and resource authority, not death proof.
use super::*;
use crate::{
    adapter::{AdapterError, ErrorKind, PreparedInput},
    domain::{Session, SessionId, SessionRole, SessionState, now_ms},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, VecDeque},
    process::Stdio,
    sync::{Arc, Mutex, Weak},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout, Command},
    sync::{mpsc, watch},
};
#[cfg(test)]
pub(crate) mod tests;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedSessionRef {
    pub scope: crate::domain::Scope,
    pub unit: UnitId,
    pub generation: u64,
    pub epoch: u64,
    pub session: SessionId,
}
#[derive(Clone, Debug)]
pub struct ManagedInput {
    pub agent: String,
    pub authority: ExecutionAuthority,
    pub artifact: Option<ArtifactId>,
    pub input: PreparedInput,
}
#[derive(Clone, Debug)]
pub struct NativeStatus {
    pub handle: ManagedSessionRef,
    pub authority: ExecutionAuthority,
    pub session: Session,
    pub work: Option<WorkOutcome>,
    pub disposition: Disposition,
    pub cleanup: CleanupOutcome,
    pub wait_reason: Option<WaitReason>,
    pub pending: Vec<Value>,
    pub result: Option<Value>,
    pub metrics: Option<Value>,
    pub diagnostic: Option<&'static str>,
    pub failure: Option<NativeFailure>,
}
pub enum NativeStart {
    Launched(ManagedSessionRef),
    Waiting {
        unit: Box<ExecutionUnit>,
        reason: WaitReason,
        next_due: i64,
    },
}
enum Control {
    Cancel,
    Approval {
        authority: ExecutionAuthority,
        id: Value,
        hash: String,
        allow: bool,
        response: tokio::sync::oneshot::Sender<Result<()>>,
    },
}
struct Entry {
    handle: ManagedSessionRef,
    status: watch::Receiver<NativeStatus>,
    control: mpsc::Sender<Control>,
}
/// Conservative caps: aliases for a provider share the strictest configured cap.
/// Counting leases in SQLite includes admission before a Session exists.
#[derive(Clone)]
pub(crate) struct NativeLimits {
    global: usize,
    project: usize,
    providers: BTreeMap<String, usize>,
}
impl NativeLimits {
    fn default_policy() -> Self {
        Self {
            global: 6,
            project: 4,
            providers: BTreeMap::new(),
        }
    }
    pub(crate) fn configured(config: &crate::config::Config) -> Self {
        let mut providers = BTreeMap::<String, usize>::new();
        for agent in config.agents.values() {
            if let (Some(provider), Some(cap)) = (&agent.provider, agent.max_concurrent) {
                providers
                    .entry(provider.clone())
                    .and_modify(|c| *c = (*c).min(cap))
                    .or_insert(cap);
            }
        }
        Self {
            global: config.scheduler.global_max_sessions.min(6),
            project: config.scheduler.max_tasks_per_project,
            providers,
        }
    }
    fn scheduler(&self, owner: Arc<RuntimeOwner>, provider: &str) -> quota::QuotaScheduler {
        let mut scheduler = quota::QuotaScheduler::new(owner);
        scheduler.global_total = self.global;
        scheduler.project_tasks = self.project;
        if let Some(cap) = self.providers.get(provider) {
            scheduler.provider_executor = scheduler.provider_executor.min(*cap);
            scheduler.provider_total = scheduler.provider_total.min(*cap);
        }
        scheduler
    }
}
pub struct NativeSessions {
    limits: NativeLimits,
    owner: Arc<RuntimeOwner>,
    _tools: Arc<ipc::ToolServer>,
    _cleanup: cleanup::CleanupWorker,
    entries: Mutex<BTreeMap<SessionId, Entry>>,
    starts: Mutex<BTreeMap<UnitId, Weak<tokio::sync::Mutex<()>>>>,
}
impl NativeSessions {
    pub fn new(owner: Arc<RuntimeOwner>) -> Result<Self> {
        Self::with_limits(owner, NativeLimits::default_policy())
    }
    pub(crate) fn with_limits(owner: Arc<RuntimeOwner>, limits: NativeLimits) -> Result<Self> {
        let tools = Arc::new(ipc::ToolServer::start(owner.clone())?);
        let cleanup = cleanup::CleanupWorker::start(&owner)?;
        Ok(Self {
            limits,
            owner,
            _tools: tools,
            _cleanup: cleanup,
            entries: Mutex::new(BTreeMap::new()),
            starts: Mutex::new(BTreeMap::new()),
        })
    }
    pub async fn start(
        &self,
        input: ManagedInput,
        model: Option<String>,
        effort: Option<String>,
    ) -> Result<NativeStart> {
        self.start_inner(input, model, effort, None).await
    }
    pub(crate) async fn start_inner(
        &self,
        input: ManagedInput,
        model: Option<String>,
        effort: Option<String>,
        executable: Option<std::path::PathBuf>,
    ) -> Result<NativeStart> {
        let gate = {
            let mut starts = self
                .starts
                .lock()
                .map_err(|_| anyhow::anyhow!("start state poisoned"))?;
            starts.retain(|_, gate| gate.strong_count() > 0);
            match starts.get(&input.authority.unit_id).and_then(Weak::upgrade) {
                Some(gate) => gate,
                None => {
                    let gate = Arc::new(tokio::sync::Mutex::new(()));
                    starts.insert(input.authority.unit_id, Arc::downgrade(&gate));
                    gate
                }
            }
        };
        // Start preparation for separate Tasks remains concurrent. A competing
        // start for this unit cannot adopt or retire the successful winner.
        let _start = gate.lock().await;
        let mut unit = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .validate_execution(&input.authority, true, false)?;
        ensure!(
            matches!(unit.state, UnitState::Preparing | UnitState::WaitingQuota)
                && unit.session_id.is_none()
                && input.artifact == unit.artifact_id
                && input.input.scope == unit.scope
                && input.input.revision == unit.base_sha
                && valid_oid(&unit.base_sha)
                && !input.agent.trim().is_empty()
                && input.agent.len() <= 128
                && !input.agent.chars().any(char::is_control)
                && input.input.version > 0
                && !input.input.payload.is_empty()
                && input.input.payload.len() <= 1024 * 1024,
            "native input identity mismatch"
        );
        let mut preparation_guard = owner::PreparationGuard::new(self.owner.clone(), &unit);
        ensure!(
            matches!(unit.provider.as_str(), "codex" | "claude"),
            "unsupported native provider"
        );
        ensure!(
            [&model, &effort]
                .into_iter()
                .flatten()
                .all(|s| !s.is_empty() && s.len() <= 128 && !s.contains('\0')),
            "invalid native model/effort"
        );
        if unit.kind != UnitKind::Executor {
            results::verify_readonly_source(&unit.worktree)?;
            let io = super::git_io::UnitGit::new(self.owner.clone(), &unit, true)?;
            ensure!(
                io.text(&unit.worktree, ["rev-parse", "HEAD"]).await? == unit.base_sha,
                "review snapshot HEAD mismatch"
            );
            ensure!(
                io.run(
                    &unit.worktree,
                    ["status", "--porcelain", "--untracked-files=all"]
                )
                .await?
                .is_empty(),
                "review snapshot is dirty"
            );
        }
        let profile = resources::ResourceManager::new(self.owner.clone()).profile(&unit)?;
        let program = match executable {
            Some(program) => program,
            None => resources::resolve_program(&unit.provider)?,
        };
        let mut version = Command::new(&program);
        version
            .arg("--version")
            .current_dir(&unit.worktree)
            .envs(profile.environment(&unit.cookie, self.owner.ipc_path())?);
        version
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let operation = OperationId::new();
        let child = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let current = store.validate_execution(&unit.authority(), true, false)?;
            store.reserve_execution_helper(
                &current.authority(),
                operation,
                true,
                &unit.worktree,
                "native_version",
            )?;
            match process::OwnedProcess::spawn(&mut version) {
                Ok(child) => child,
                Err(error) => {
                    store.reconcile_managed_effect(
                        operation,
                        1,
                        EffectState::Unknown,
                        BTreeMap::new(),
                    )?;
                    return Err(error);
                }
            }
        };
        let mut helper_guard = owner::HelperGuard::new(self.owner.clone(), operation);
        let observed = process::capture_scoped(child, &self.owner, &unit, true).await;
        let receipt = observed
            .as_ref()
            .ok()
            .map(|o| {
                BTreeMap::from([(
                    "exit".into(),
                    o.receipt
                        .status
                        .code()
                        .map_or_else(|| "signal".into(), |n| n.to_string()),
                )])
            })
            .unwrap_or_default();
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .reconcile_managed_effect(
                operation,
                1,
                if observed.is_ok() {
                    EffectState::Confirmed
                } else {
                    EffectState::Unknown
                },
                receipt,
            )?;
        helper_guard.disarm();
        let observed = observed?;
        ensure!(
            observed.receipt.status.success(),
            "native version helper failed"
        );
        let version = results::text(&observed.stdout)?;
        if unit.provider == "codex" {
            crate::codex::managed::version(&version)?;
        } else {
            claude_wire::verify_version(&version)?;
        }
        if let Some((reason, next_due)) = self
            .limits
            .scheduler(self.owner.clone(), &unit.provider)
            .admission(&unit.authority(), now_ms())?
        {
            let unit = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .execution_unit(unit.id)?;
            preparation_guard.disarm();
            return Ok(NativeStart::Waiting {
                unit: Box::new(unit),
                reason,
                next_due,
            });
        }
        unit = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .execution_unit(unit.id)?;
        preparation_guard.update(&unit);
        let mut session = Session {
            id: SessionId::new(),
            scope: unit.scope.clone(),
            agent: input.agent.clone(),
            provider: unit.provider.clone(),
            role: match unit.kind {
                UnitKind::Executor => SessionRole::Executor,
                UnitKind::Verifier => SessionRole::Consultant,
                _ => SessionRole::Reviewer,
            },
            native_ref: None,
            pid: None,
            worktree: unit.worktree.clone(),
            state: SessionState::Starting,
            model: model.clone(),
            effort: effort.clone(),
            recovery: json!({"unit":unit.id,"generation":unit.generation,"epoch":unit.owner_epoch}),
            started_at: now_ms(),
        };
        let native = uuid::Uuid::new_v4().to_string();
        let mut command = Command::new(&program);
        if unit.provider == "codex" {
            command.args(["app-server", "--listen", "stdio://"]);
        } else {
            command
                .args([
                    "-p",
                    "--input-format",
                    "stream-json",
                    "--output-format",
                    "stream-json",
                    "--verbose",
                    "--session-id",
                ])
                .arg(&native)
                .args([
                    "--permission-prompt-tool",
                    "stdio",
                    "--settings",
                    r#"{"forceLoginMethod":"claudeai"}"#,
                ]);
            if let Some(model) = &model {
                command.arg("--model").arg(model);
            }
            if let Some(effort) = &effort {
                command.arg("--effort").arg(effort);
            }
            if unit.kind != UnitKind::Executor {
                // Native plan mode routes writes to the permission callback even
                // when settings allow them; this Runtime never grants reviewer writes.
                // This is a cooperative native role policy, not host containment.
                command.args(["--permission-mode", "plan"]);
            }
        }
        command
            .current_dir(&unit.worktree)
            .envs(profile.environment(&unit.cookie, self.owner.ipc_path())?)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            unit = store.register_execution_session(&unit.authority(), &session)?;
            preparation_guard.update(&unit);
            match process::OwnedProcess::spawn(&mut command) {
                Ok(child) => child,
                Err(e) => {
                    store.retire_execution(&unit.authority(), false)?;
                    session.state = SessionState::Lost;
                    store.close_execution_session(unit.id, &session, 1)?;
                    return Err(e);
                }
            }
        };
        session.pid = child.child.id();
        let stdin = child.child.stdin.take().context("native stdin missing")?;
        let stdout = child.child.stdout.take().context("native stdout missing")?;
        let stderr = child.child.stderr.take().context("native stderr missing")?;
        // Draining bounds memory and retains no provider stderr/authentication material.
        let drain = tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut stderr = stderr;
            let mut buf = [0; 8192];
            loop {
                match stderr.read(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
            }
        });
        let handle = ManagedSessionRef {
            scope: unit.scope.clone(),
            unit: unit.id,
            generation: unit.generation,
            epoch: unit.owner_epoch,
            session: session.id,
        };
        let initial = NativeStatus {
            handle: handle.clone(),
            authority: unit.authority(),
            session: session.clone(),
            work: None,
            disposition: Disposition::Active,
            cleanup: CleanupOutcome::Unknown,
            wait_reason: None,
            pending: vec![],
            result: None,
            metrics: None,
            diagnostic: None,
            failure: None,
        };
        let (update, status) = watch::channel(initial);
        let (control, receiver) = mpsc::channel(16);
        let limit = if unit.provider == "claude" {
            claude_wire::LINE_LIMIT
        } else {
            4 * 1024 * 1024
        };
        let core = Core {
            owner: self.owner.clone(),
            unit,
            session,
            record_version: 1,
            wire: Lines::new(stdin, stdout, limit),
            child,
            update,
            controls: receiver,
            native,
            drain,
        };
        self.entries
            .lock()
            .map_err(|_| anyhow::anyhow!("sessions poisoned"))?
            .insert(
                handle.session,
                Entry {
                    handle: handle.clone(),
                    status,
                    control,
                },
            );
        preparation_guard.disarm();
        tokio::spawn(core.run(input.input, model, effort, profile));
        Ok(NativeStart::Launched(handle))
    }
    fn entry(
        &self,
        handle: &ManagedSessionRef,
    ) -> Result<(watch::Receiver<NativeStatus>, mpsc::Sender<Control>)> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("sessions poisoned"))?;
        let entry = entries
            .get(&handle.session)
            .context("native Session missing")?;
        ensure!(
            entry.handle == *handle,
            "foreign native Session/unit/generation"
        );
        Ok((entry.status.clone(), entry.control.clone()))
    }
    pub fn subscribe(&self, handle: &ManagedSessionRef) -> Result<watch::Receiver<NativeStatus>> {
        Ok(self.entry(handle)?.0)
    }
    pub fn status(&self, handle: &ManagedSessionRef) -> Result<NativeStatus> {
        let mut status = self.entry(handle)?.0.borrow().clone();
        let unit = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .execution_unit(handle.unit)?;
        status.authority = unit.authority();
        status.work = unit.work;
        status.disposition = unit.disposition;
        status.cleanup = unit.cleanup;
        status.wait_reason = unit.wait_reason;
        Ok(status)
    }
    pub fn release(&self, handle: &ManagedSessionRef) -> Result<()> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("native registry poisoned"))?;
        let entry = entries
            .get(&handle.session)
            .context("native Session missing")?;
        ensure!(
            entry.handle == *handle,
            "foreign native Session/unit/generation"
        );
        ensure!(
            matches!(
                entry.status.borrow().session.state,
                SessionState::Exited
                    | SessionState::Stopped
                    | SessionState::Lost
                    | SessionState::Failed
            ),
            "live native Session cannot be released"
        );
        entries.remove(&handle.session);
        Ok(())
    }
    pub async fn cancel(&self, handle: &ManagedSessionRef) -> Result<()> {
        let (_, control) = self.entry(handle)?;
        {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let unit = store.execution_unit(handle.unit)?;
            if unit.native_effects_open || unit.result_finalization_open {
                store.retire_execution(&unit.authority(), false)?;
            }
        }
        let _ = control.send(Control::Cancel).await;
        Ok(())
    }
    pub async fn approve(
        &self,
        handle: &ManagedSessionRef,
        authority: ExecutionAuthority,
        id: Value,
        hash: String,
        allow: bool,
    ) -> Result<()> {
        let (_, control) = self.entry(handle)?;
        ensure!(
            authority.unit_id == handle.unit
                && authority.scope == handle.scope
                && authority.session_id == Some(handle.session),
            "foreign approval authority"
        );
        let (response, receive) = tokio::sync::oneshot::channel();
        control
            .send(Control::Approval {
                authority,
                id,
                hash,
                allow,
                response,
            })
            .await?;
        tokio::time::timeout(Duration::from_secs(10), receive).await??
    }
}
struct Lines {
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    partial: Vec<u8>,
    queued: VecDeque<Value>,
    queued_bytes: usize,
    next: u64,
    limit: usize,
}
impl Lines {
    fn new(stdin: ChildStdin, stdout: ChildStdout, limit: usize) -> Self {
        Self {
            stdin,
            stdout: BufReader::new(stdout),
            partial: vec![],
            queued: VecDeque::new(),
            queued_bytes: 0,
            next: 1,
            limit,
        }
    }
    async fn send(&mut self, value: &Value) -> Result<()> {
        let mut bytes = serde_json::to_vec(value)?;
        ensure!(
            bytes.len() <= 1024 * 1024,
            "native outgoing frame too large"
        );
        bytes.push(b'\n');
        tokio::time::timeout(Duration::from_secs(10), self.stdin.write_all(&bytes)).await??;
        self.stdin.flush().await?;
        Ok(())
    }
    async fn line(&mut self) -> Result<Value> {
        loop {
            let chunk = self.stdout.fill_buf().await?;
            ensure!(!chunk.is_empty(), NativeFailure::TransportLost);
            let newline = chunk.iter().position(|b| *b == b'\n');
            let n = newline.map_or(chunk.len(), |i| i + 1);
            ensure!(
                self.partial.len() + n <= self.limit,
                "native frame too large"
            );
            self.partial.extend_from_slice(&chunk[..n]);
            self.stdout.consume(n);
            if newline.is_some() {
                let bytes = std::mem::take(&mut self.partial);
                return Ok(serde_json::from_slice(&bytes)?);
            }
        }
    }
    async fn receive(&mut self) -> Result<Value> {
        if let Some(value) = self.queued.pop_front() {
            self.queued_bytes -= serde_json::to_vec(&value)?.len();
            Ok(value)
        } else {
            self.line().await
        }
    }
    fn queue(&mut self, value: Value) -> Result<()> {
        let n = serde_json::to_vec(&value)?.len();
        ensure!(
            self.queued.len() < 128 && self.queued_bytes + n <= 8 * 1024 * 1024,
            "native startup queue too large"
        );
        self.queued.push_back(value);
        self.queued_bytes += n;
        Ok(())
    }
    async fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next;
        self.next = self.next.checked_add(1).context("RPC identity exhausted")?;
        self.send(&json!({"id":id,"method":method,"params":params}))
            .await?;
        self.response(id).await
    }
    async fn response(&mut self, id: u64) -> Result<Value> {
        tokio::time::timeout(Duration::from_secs(30),async {loop {
            let value=self.line().await?;crate::codex::managed::frame(&value)?;
            if value.get("method").is_none() && value["id"]==id {
                if value.get("error").is_some() {
                    anyhow::bail!(if value["error"]["code"] == -32601 {NativeFailure::UnsupportedCapability}else{NativeFailure::ProtocolFailure});
                }
                return Ok(value["result"].clone());
            }
            if value.get("id").is_some() && value.get("method").is_some(){
                self.send(&json!({"id":value["id"],"error":{"code":-32601,"message":"No native turn established"}})).await?;
            }else{self.queue(value)?;}
        }}).await?
    }
}
/// A private scoped dispatch boundary. The ledger stores a digest, never the
/// Agent input, permission arguments, native credentials or a replayable frame.
fn admit_native_frame(
    owner: &RuntimeOwner,
    pinned: &ExecutionUnit,
    session: SessionId,
    expected: Option<&ExecutionAuthority>,
    kind: &str,
    frame: &Value,
) -> Result<OperationId> {
    use sha2::{Digest, Sha256};
    ensure!(
        matches!(kind, "native_input" | "native_permission" | "native_setup"),
        "invalid native dispatch kind"
    );
    let bytes = serde_json::to_vec(frame)?;
    ensure!(
        bytes.len() <= 1024 * 1024,
        "native outgoing frame too large"
    );
    let digest = format!("{:x}", Sha256::digest(bytes));
    let mut store = owner
        .store
        .lock()
        .map_err(|_| anyhow::anyhow!("state poisoned"))?;
    let current = store.execution_unit(pinned.id)?;
    ensure!(
        current.scope == pinned.scope
            && current.generation == pinned.generation
            && current.owner_epoch == pinned.owner_epoch
            && current.session_id == Some(session),
        "native dispatch semantic identity changed"
    );
    let authority = expected.cloned().unwrap_or_else(|| current.authority());
    ensure!(
        authority == current.authority(),
        "native permission authority changed"
    );
    let operation = OperationId::new();
    store.reserve_managed_effect(
        &authority,
        &ManagedEffect {
            id: operation,
            unit_id: current.id,
            scope: current.scope,
            kind: kind.into(),
            idempotency_key: format!("native-{operation}"),
            expected_target: format!("session/{session}/sha256/{digest}"),
            state: EffectState::Pending,
            receipt: BTreeMap::new(),
            version: 1,
        },
    )?;
    Ok(operation)
}
fn native_authority(
    owner: &RuntimeOwner,
    pinned: &ExecutionUnit,
    session: SessionId,
) -> Result<ExecutionAuthority> {
    let store = owner
        .store
        .lock()
        .map_err(|_| anyhow::anyhow!("state poisoned"))?;
    let unit = store.execution_unit(pinned.id)?;
    ensure!(
        unit.scope == pinned.scope
            && unit.generation == pinned.generation
            && unit.owner_epoch == pinned.owner_epoch
            && unit.session_id == Some(session),
        "native semantic identity changed"
    );
    store.validate_execution(&unit.authority(), true, false)?;
    Ok(unit.authority())
}
struct Core {
    owner: Arc<RuntimeOwner>,
    unit: ExecutionUnit,
    session: Session,
    record_version: u64,
    wire: Lines,
    child: process::OwnedProcess,
    update: watch::Sender<NativeStatus>,
    controls: mpsc::Receiver<Control>,
    native: String,
    drain: tokio::task::JoinHandle<()>,
}
impl Core {
    fn authority(&self) -> Result<ExecutionAuthority> {
        native_authority(&self.owner, &self.unit, self.session.id)
            .context(NativeFailure::AuthorityUnavailable)
    }
    async fn send_effect(
        &mut self,
        value: &Value,
        authority: Option<&ExecutionAuthority>,
        kind: &str,
    ) -> Result<()> {
        let operation = admit_native_frame(
            &self.owner,
            &self.unit,
            self.session.id,
            authority,
            kind,
            value,
        )?;
        // The durable intent is the dispatch winner. Later retirement cannot revoke
        // already issued bytes, and lost acknowledgements are never blindly replayed.
        let sent = self.wire.send(value).await;
        let receipt = BTreeMap::from([(
            "transport".into(),
            if sent.is_ok() { "written" } else { "unknown" }.into(),
        )]);
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .reconcile_managed_effect(
                operation,
                1,
                if sent.is_ok() {
                    EffectState::Confirmed
                } else {
                    EffectState::Unknown
                },
                receipt,
            )?;
        sent
    }
    async fn boot_call(
        &mut self,
        method: &str,
        params: Value,
        kind: Option<&str>,
    ) -> Result<Value> {
        let operation = kind
            .map(|kind| {
                admit_native_frame(
                    &self.owner,
                    &self.unit,
                    self.session.id,
                    None,
                    kind,
                    &json!({"id":self.wire.next,"method":method,"params":params}),
                )
            })
            .transpose()?;
        let owner = self.owner.clone();
        let pinned = self.unit.clone();
        let session = self.session.id;
        let mut fence = tokio::time::interval(Duration::from_millis(100));
        let result = {
            let call = self.wire.call(method, params);
            tokio::pin!(call);
            loop {
                tokio::select! {
                    result=&mut call=>break result,
                    control=self.controls.recv()=>match control {
                        Some(Control::Approval {response,..})=>{let _=response.send(Err(anyhow::anyhow!("native turn not established")));},
                        _=>break Err(anyhow::anyhow!("native bootstrap cancelled"))
                    },
                    _=fence.tick()=>{if let Err(error)=native_authority(&owner,&pinned,session){break Err(error);}}
                }
            }
        };
        if let Some(operation) = operation {
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .reconcile_managed_effect(
                    operation,
                    1,
                    if result.is_ok() {
                        EffectState::Confirmed
                    } else {
                        EffectState::Unknown
                    },
                    BTreeMap::from([(
                        "native_ack".into(),
                        if result.is_ok() {
                            "received"
                        } else {
                            "unknown"
                        }
                        .into(),
                    )]),
                )?;
        }
        result
    }
    fn ack(&mut self, native: String) -> Result<()> {
        self.session.native_ref = Some(native);
        self.session.state = SessionState::Running;
        let authority = self.authority()?;
        let (unit, version) = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .update_execution_session(&authority, &self.session, self.record_version)?;
        self.unit = unit;
        self.record_version = version;
        self.update.send_modify(|s| {
            s.session = self.session.clone();
            s.authority = self.unit.authority();
            s.wait_reason = self.unit.wait_reason;
        });
        Ok(())
    }
    async fn run(
        mut self,
        input: PreparedInput,
        model: Option<String>,
        effort: Option<String>,
        profile: resources::ResourceProfile,
    ) {
        let result = if self.unit.provider == "codex" {
            self.codex(&input, model, effort, &profile).await
        } else {
            self.claude(&input).await
        };
        let (work, disposition, output, failure) = match result {
            Ok(value) => (value.0, value.1, value.2, None),
            Err(error) => {
                // Never retain raw native errors, output or credentials. Only
                // finite nonsecret categories survive the supervisor.
                let category = if let Some(category) = error.downcast_ref::<NativeFailure>() {
                    *category
                } else if let Some(error) = error.downcast_ref::<AdapterError>() {
                    match error.kind {
                        ErrorKind::AuthenticationUnavailable => {
                            NativeFailure::AuthenticationUnavailable
                        }
                        ErrorKind::UnsupportedCapability | ErrorKind::ExecutableMissing => {
                            NativeFailure::UnsupportedCapability
                        }
                        ErrorKind::OwnershipMismatch
                        | ErrorKind::StateFailure
                        | ErrorKind::StateConflict
                        | ErrorKind::Locked => NativeFailure::AuthorityUnavailable,
                        ErrorKind::SessionLost | ErrorKind::Timeout => NativeFailure::TransportLost,
                        _ => NativeFailure::ProtocolFailure,
                    }
                } else if error.is::<std::io::Error>() || error.is::<tokio::time::error::Elapsed>()
                {
                    NativeFailure::TransportLost
                } else {
                    NativeFailure::ProtocolFailure
                };
                let disposition = match category {
                    NativeFailure::TransportLost => Disposition::Lost,
                    NativeFailure::ProtocolFailure => Disposition::ProtocolError,
                    _ => Disposition::Refused,
                };
                (WorkOutcome::Unknown, disposition, None, Some(category))
            }
        };
        // Persist work before hygiene; late completions cannot reopen a cancelled generation.
        let persisted = (|| {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let current = store.execution_unit(self.unit.id)?;
            if current.native_effects_open
                && let Err(e) = store.finish_execution_with_failure(
                    &current.authority(),
                    work,
                    disposition,
                    failure,
                )
            {
                let current = store.execution_unit(self.unit.id)?;
                if current.native_effects_open || current.result_finalization_open {
                    store.retire_execution(&current.authority(), false)?;
                }
                return Err(e);
            }
            Ok::<_, anyhow::Error>(())
        })();
        let stopped = self.child.stop_and_reap().await;
        self.drain.abort();
        let mut store = match self.owner.store.lock() {
            Ok(store) => store,
            Err(_) => return,
        };
        let current = match store.execution_unit(self.unit.id) {
            Ok(unit) => unit,
            Err(_) => return,
        };
        self.session.state = if current.disposition == Disposition::Cancelled {
            SessionState::Stopped
        } else if current.disposition == Disposition::Lost {
            SessionState::Lost
        } else {
            SessionState::Exited
        };
        let session_result =
            store.close_execution_session(self.unit.id, &self.session, self.record_version);
        let group_error = stopped.as_ref().map_or(true, |s| s.group_error.is_some());
        let _ = store.record_execution_cleanup(&CleanupObservation {
            actions: Vec::new(),
            unit_id: self.unit.id,
            at: now_ms(),
            outcome: CleanupOutcome::Unknown,
            coverage: BTreeMap::from([
                (
                    "owned_group".into(),
                    if group_error {
                        "signal observation incomplete"
                    } else {
                        "termination requested"
                    }
                    .into(),
                ),
                ("escaped_processes".into(), "not yet scanned".into()),
                ("docker".into(), "pending reconciliation".into()),
            ]),
            remaining: vec![],
            errors: if group_error {
                vec!["owned_group_cleanup_unknown".into()]
            } else {
                vec![]
            },
        });
        let final_authority = store
            .execution_unit(self.unit.id)
            .map(|u| u.authority())
            .unwrap_or_else(|_| current.authority());
        self.update.send_modify(|s| {
            s.authority = final_authority;
            s.session = self.session.clone();
            s.work = current.work;
            s.wait_reason = current.wait_reason;
            s.disposition = current.disposition;
            s.cleanup = CleanupOutcome::Unknown;
            s.pending.clear();
            s.result = output;
            s.failure = if current.disposition == Disposition::Cancelled {
                None
            } else {
                failure
            };
            s.diagnostic = if let Some(failure) = s.failure {
                Some(failure.diagnostic())
            } else if persisted.is_err() || session_result.is_err() {
                Some("terminal persistence conflict")
            } else if current.disposition == Disposition::CapacityInterrupted {
                Some("native capacity unclassified; fresh attempt waits for bounded recheck")
            } else if current.disposition == Disposition::QuotaInterrupted {
                Some("subscription quota exhausted; fresh attempt waits for recovery")
            } else if group_error {
                Some("owned group cleanup incomplete")
            } else {
                None
            };
        });
    }
    async fn codex(
        &mut self,
        input: &PreparedInput,
        model: Option<String>,
        effort: Option<String>,
        profile: &resources::ResourceProfile,
    ) -> Result<(WorkOutcome, Disposition, Option<Value>)> {
        self.authority()?;
        let init=self.boot_call("initialize",json!({"clientInfo":{"name":"rururunx","version":env!("CARGO_PKG_VERSION")},"capabilities":{"experimentalApi":true}}),None).await?;
        crate::codex::managed::initialization(&init)
            .context(NativeFailure::UnsupportedCapability)?;
        self.wire
            .send(&json!({"method":"initialized","params":{}}))
            .await?;
        let account = self
            .boot_call("account/read", json!({"refreshToken":false}), None)
            .await?;
        ensure!(
            account["account"]["type"] == "chatgpt",
            NativeFailure::AuthenticationUnavailable
        );
        let environment = self
            .boot_call("environment/status", json!({"environmentId":"local"}), None)
            .await?;
        ensure!(
            environment["status"] == "ready",
            NativeFailure::UnsupportedCapability
        );
        let quota = self
            .boot_call("account/rateLimits/read", json!({}), None)
            .await?;
        let probe = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .execution_is_quota_probe(self.unit.id, "codex", "unknown")?;
        let observations =
            quota::codex_windows(&quota, now_ms()).context(NativeFailure::MetadataUnavailable)?;
        let exhausted = observations
            .iter()
            .any(|o| o.status == QuotaStatus::Exhausted);
        for observation in observations {
            let scheduler = quota::QuotaScheduler::new(self.owner.clone());
            if probe && observation.status == QuotaStatus::Available {
                scheduler.observe_probe(&self.authority()?, &observation)?;
            } else {
                scheduler.observe(&observation)?;
            }
        }
        if exhausted && !probe {
            return Ok((WorkOutcome::Unknown, Disposition::QuotaInterrupted, None));
        }
        let mut params = json!({"cwd":self.unit.worktree,"environments":[{"environmentId":"local","cwd":self.unit.worktree,"runtimeWorkspaceRoots":[self.unit.worktree,profile.temp,profile.output,profile.cache]}]});
        if let Some(model) = model {
            params["model"] = json!(model);
        }
        if self.unit.kind != UnitKind::Executor {
            params["sandbox"] = json!("read-only");
        }
        self.authority()?;
        let thread = self
            .boot_call("thread/start", params, Some("native_setup"))
            .await?;
        ensure!(
            thread["cwd"].as_str() == self.unit.worktree.to_str()
                && thread["modelProvider"] == "openai",
            NativeFailure::UnsupportedCapability
        );
        let thread_id = claude_wire::bounded_id(&thread["thread"]["id"])?;
        self.ack(thread_id.clone())?;
        let mut turn = json!({"threadId":thread_id,"input":[{"type":"text","text":input.payload,"text_elements":[]}]});
        if let Some(effort) = effort {
            turn["effort"] = json!(effort);
        }
        self.authority()?;
        let started = self
            .boot_call("turn/start", turn, Some("native_input"))
            .await?;
        let turn_id = claude_wire::bounded_id(&started["turn"]["id"])?;
        let mut approvals =
            crate::codex::managed::Approvals::new(&thread_id, &turn_id, &self.unit.worktree);
        let mut quota_ended = false;
        let mut fence = tokio::time::interval(Duration::from_millis(100));
        loop {
            tokio::select! {
                frame=self.wire.receive()=>{
                    let frame=frame?;crate::codex::managed::frame(&frame)?;
                    let p=&frame["params"];
                    if frame.get("id").is_some() && frame.get("method").is_some(){
                        self.authority()?;let pending=approvals.insert(&frame)?;self.update.send_modify(|s|s.pending.push(pending));continue;
                    }
                    if frame["method"]=="account/rateLimits/updated" {
                        for observation in quota::codex_windows(p,now_ms())?{quota::QuotaScheduler::new(self.owner.clone()).observe(&observation)?;}continue;
                    }
                    if p.get("threadId").is_some_and(|id|id!=&json!(thread_id)){continue;}
                    if p.get("turnId").is_some_and(|id|id!=&json!(turn_id)){continue;}
                    match frame["method"].as_str() {
                        Some("item/started")=>approvals.item(&p["item"])? ,
                        Some("error") if p["threadId"]==thread_id && p["turnId"]==turn_id && quota::codex_subscription_error(&p["error"])=>{
                            quota_ended = !p["willRetry"].as_bool().context("native retry flag missing")?;
                            quota::QuotaScheduler::new(self.owner.clone()).observe(&QuotaObservation {provider:"codex".into(),account_key:"unknown".into(),bucket:"native.subscription".into(),window_id:"unknown-native".into(),
                                status:QuotaStatus::Exhausted,used_percent:None,resets_at:None,observed_at:now_ms(),source_version:"codex-cli 0.160.0/usageLimitExceeded".into(),confirmed_subscription:true})?;
                            if !quota_ended {
                                let authority=self.authority()?;
                                self.unit=self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.mark_execution_quota_retry(&authority)?;
                            }
                            self.update.send_modify(|s|{s.authority=self.unit.authority();s.wait_reason=self.unit.wait_reason;s.diagnostic=Some("subscription quota exhausted; native retry state retained");});
                        },
                        Some("turn/completed") if p["threadId"]==thread_id && p["turn"]["id"]==turn_id=>{
                            let status=p["turn"]["status"].as_str().context("native turn status missing")?;
                            // The terminal may be the only subscription error notification,
                            // or follow a willRetry=true notification for the same turn.
                            if status=="failed" && quota::codex_subscription_error(&p["turn"]["error"]) {
                                quota_ended=true;
                                quota::QuotaScheduler::new(self.owner.clone()).observe(&QuotaObservation {provider:"codex".into(),account_key:"unknown".into(),bucket:"native.subscription".into(),window_id:"unknown-native".into(),
                                    status:QuotaStatus::Exhausted,used_percent:None,resets_at:None,observed_at:now_ms(),source_version:"codex-cli 0.160.0/terminal usageLimitExceeded".into(),confirmed_subscription:true})?;
                            }
                            let current_probe=self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.execution_is_quota_probe(self.unit.id,"codex","unknown")?;
                            if status=="completed" && current_probe {
                                let windows=self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.quota_observations("codex","unknown")?;
                                if let Some(old)=windows.into_iter().find(|o|o.bucket=="native.subscription" && o.status==QuotaStatus::Exhausted) {
                                    quota::QuotaScheduler::new(self.owner.clone()).observe_probe(&self.authority()?,&QuotaObservation {status:QuotaStatus::Available,observed_at:now_ms(),source_version:"codex-cli 0.160.0/correlated recovery turn completed".into(),..old})?;
                                }
                            }
                            return Ok(if status=="completed" {(WorkOutcome::Success,Disposition::Completed,Some(p["turn"].clone()))}
                                else if quota_ended {(WorkOutcome::Unknown,Disposition::QuotaInterrupted,None)}
                                else if status=="failed" && p["turn"]["error"]["codexErrorInfo"]=="unauthorized" {anyhow::bail!(NativeFailure::AuthenticationUnavailable)}
                                else if status=="failed" && quota::codex_capacity_error(&p["turn"]["error"]) {(WorkOutcome::Unknown,Disposition::CapacityInterrupted,None)}
                                else if status=="failed" {(WorkOutcome::Failure,Disposition::Completed,Some(p["turn"].clone()))}
                                else {(WorkOutcome::Unknown,Disposition::Lost,None)});
                        },_=>{}
                    }
                },
                control=self.controls.recv()=>{
                    match control {Some(Control::Approval {authority,id,hash,allow,response})=>{
                        let reply=(||{self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.validate_execution(&authority,true,false)?;
                            ensure!(self.unit.kind==UnitKind::Executor || !allow,"read-only native reviewer cannot grant a tool effect");approvals.reply(&id,&hash,allow)})();
                        let result=match reply{Ok(value)=>self.send_effect(&value,Some(&authority),"native_permission").await,Err(e)=>Err(e)};
                        if result.is_ok(){self.update.send_modify(|s|s.pending.retain(|p|p["id"]!=id));}let _=response.send(result);
                    },_=>return Ok((WorkOutcome::Unknown,Disposition::Cancelled,None))}
                },
                _=fence.tick()=>{self.authority()?;}
            }
        }
    }
    async fn claude(
        &mut self,
        input: &PreparedInput,
    ) -> Result<(WorkOutcome, Disposition, Option<Value>)> {
        self.authority()?;
        self.wire
            .send(&claude_wire::control("initialize", "rrx-initialize"))
            .await?;
        let owner = self.owner.clone();
        let pinned = self.unit.clone();
        let session = self.session.id;
        let mut fence = tokio::time::interval(Duration::from_millis(100));
        tokio::time::timeout(Duration::from_secs(30),async {
            loop {tokio::select! {
                frame=self.wire.line()=>{
                    let frame=frame?;
                    if frame["type"]=="control_response" {
                        claude_wire::control_result(&frame,"rrx-initialize")?;break;
                    }
                    self.wire.queue(frame)?;
                },
                control=self.controls.recv()=>match control {
                    Some(Control::Approval {response,..})=>{let _=response.send(Err(anyhow::anyhow!("native input not established")));},
                    _=>anyhow::bail!("native bootstrap cancelled")
                },
                _=fence.tick()=>{native_authority(&owner,&pinned,session)?;}
            }}Ok::<_,anyhow::Error>(())
        }).await??;
        self.authority()?;
        self.send_effect(&json!({"type":"user","session_id":self.native,"parent_tool_use_id":null,"message":{"role":"user","content":input.payload}}),None,"native_input").await?;
        let mut state = claude_wire::RunState::default();
        let mut pending = BTreeMap::<String, claude_wire::Pending>::new();
        let mut quota_buckets = std::collections::BTreeSet::new();
        let mut unclassified_limit = false;
        let mut fence = tokio::time::interval(Duration::from_millis(100));
        loop {
            tokio::select! {
                frame=self.wire.receive()=>{
                    let frame=frame?;
                    // Only this native Session's event may change its subscription pool.
                    if frame["type"]=="rate_limit_event" && frame["session_id"]!=self.native {continue;}
                    if frame["type"]=="rate_limit_event" && frame["session_id"]==self.native && frame["rate_limit_info"]["status"]=="rejected" && quota::claude_window(&frame,now_ms())?.is_none() {
                        unclassified_limit=true;
                    }
                    if let Some(observation)=quota::claude_window(&frame,now_ms())? {
                        quota_buckets.insert(observation.bucket.clone());
                        let scheduler=quota::QuotaScheduler::new(self.owner.clone());
                        let probe=self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.execution_is_quota_probe(self.unit.id,"claude","unknown")?;
                        if probe && observation.status==QuotaStatus::Available{scheduler.observe_probe(&self.authority()?,&observation)?;}else{scheduler.observe(&observation)?;}
                        if observation.status==QuotaStatus::Exhausted {
                            let authority=self.authority()?;
                            let mut store=self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?;
                            // A rejected stale window may have been ignored by the pool.
                            // Only accepted exhaustion for this Session's bucket waits.
                            if store.quota_observations("claude","unknown")?.iter().any(|o|o.bucket==observation.bucket && o.status==QuotaStatus::Exhausted) {
                                self.unit=store.mark_execution_quota_wait(&authority)?;
                                self.update.send_modify(|s|{s.authority=self.unit.authority();s.wait_reason=self.unit.wait_reason;s.diagnostic=Some("subscription quota exhausted; native retry state retained");});
                            }
                        } else if observation.status==QuotaStatus::Available {
                            let authority=self.authority()?;
                            let recovered=self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.resume_execution_quota_wait(&authority,&quota_buckets)?;
                            if recovered.version!=self.unit.version {
                                self.unit=recovered;
                                self.update.send_modify(|s|{s.authority=self.unit.authority();s.wait_reason=self.unit.wait_reason;if s.wait_reason.is_none(){s.diagnostic=None;}});
                            }
                        }
                    }
                    if frame["type"]=="control_request"{
                        self.authority()?;let permission=claude_wire::Pending::parse(&frame,&self.native)?;
                        ensure!(pending.len()<64 && !pending.contains_key(&permission.request_id),"native permission duplicate/limit");
                        self.update.send_modify(|s|s.pending.push(permission.public(&self.native)));pending.insert(permission.request_id.clone(),permission);continue;
                    }
                    let seen=state.observe(&frame,&self.native,&self.unit.worktree,false);
                    if seen.as_ref().is_err_and(|e|e.kind==ErrorKind::ProcessFailure) {
                        // Consult accepted bucket state, not the last telemetry frame.
                        // Budget/turn/output caps remain work failures even during quota exhaustion.
                        let quota_exhausted=matches!(frame["subtype"].as_str(),Some("success"|"error_during_execution")) && self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?
                            .quota_observations("claude","unknown")?.iter().any(|o|o.status==QuotaStatus::Exhausted && quota_buckets.contains(&o.bucket));
                        return Ok(if quota_exhausted{(WorkOutcome::Unknown,Disposition::QuotaInterrupted,None)}
                            else if state.terminal_capacity || (unclassified_limit && matches!(frame["subtype"].as_str(),Some("success"|"error_during_execution"))){(WorkOutcome::Unknown,Disposition::CapacityInterrupted,None)}
                            else{(WorkOutcome::Failure,Disposition::Completed,Some(frame))});
                    }
                    seen?;
                    if state.initialized && self.session.native_ref.is_none(){self.ack(self.native.clone())?;}
                    if state.complete(){
                        let result=state.result.take().context("native successful result missing")?;
                        let metrics=claude_wire::Metrics::parse(&result,None,false)?;
                        self.update.send_modify(|s|s.metrics=Some(json!({"input":metrics.input,"output":metrics.output,"cache_read":metrics.cache_read,"cache_write":metrics.cache_write,
                            "cost":metrics.cost,"api_ms":metrics.api_ms,"cumulative_cost":metrics.cumulative_cost,"cumulative_api_ms":metrics.cumulative_api_ms})));
                        return Ok((WorkOutcome::Success,Disposition::Completed,Some(result)));
                    }
                },
                control=self.controls.recv()=>{match control {
                    Some(Control::Approval {authority,id,hash,allow,response})=>{
                        let permission=(||{
                            self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?.validate_execution(&authority,true,false)?;
                            ensure!(self.unit.kind==UnitKind::Executor || !allow,"read-only native reviewer cannot grant a tool effect");
                            let key=id.as_str().context("native permission id must be text")?;let p=pending.get(key).context("native permission absent/already answered")?;
                            ensure!(p.hash==hash,"native permission input changed");Ok::<_,anyhow::Error>((key.to_owned(),p.reply(allow)))
                        })();
                        let result=match permission{Ok((key,value))=>{let sent=self.send_effect(&value,Some(&authority),"native_permission").await;if sent.is_ok(){pending.remove(&key);self.update.send_modify(|s|s.pending.retain(|p|p["request_id"]!=id));}sent},Err(e)=>Err(e)};
                        let _=response.send(result);
                    },_=>return Ok((WorkOutcome::Unknown,Disposition::Cancelled,None))
                }},
                _=fence.tick()=>{self.authority()?;}
            }
        }
    }
}
impl Drop for Core {
    fn drop(&mut self) {
        self.drain.abort();
        let Ok(mut store) = self.owner.store.lock() else {
            return;
        };
        let Ok(mut unit) = store.execution_unit(self.unit.id) else {
            return;
        };
        let abandoned = unit.native_effects_open;
        if abandoned {
            let Ok(retired) =
                store.retire_execution_as(&unit.authority(), false, Disposition::Lost)
            else {
                return;
            };
            unit = retired;
        }
        if let Ok(Some((mut session, version))) = store.session(self.session.id)
            && !matches!(
                session.state,
                SessionState::Exited
                    | SessionState::Failed
                    | SessionState::Stopped
                    | SessionState::Lost
            )
        {
            session.state = if unit.disposition == Disposition::Cancelled {
                SessionState::Stopped
            } else if unit.work == Some(WorkOutcome::Failure) {
                SessionState::Failed
            } else if unit.work == Some(WorkOutcome::Unknown) {
                SessionState::Lost
            } else {
                SessionState::Exited
            };
            let _ = store.close_execution_session(unit.id, &session, version);
        }
        let Ok(unit) = store.execution_unit(self.unit.id) else {
            return;
        };
        let session = store
            .session(self.session.id)
            .ok()
            .flatten()
            .map(|(session, _)| session);
        self.update.send_modify(|s| {
            if let Some(session) = session {
                s.session = session;
            }
            s.authority = unit.authority();
            s.work = unit.work;
            s.disposition = unit.disposition;
            s.wait_reason = unit.wait_reason;
            s.cleanup = unit.cleanup;
            if unit.disposition == Disposition::Cancelled {
                s.failure = None;
            }
            s.pending.clear();
            if abandoned {
                s.diagnostic = Some("native supervisor ended without terminal");
            }
        });
    }
}
