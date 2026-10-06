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
mod phase_fence_tests;
mod phase_protocol;
pub(crate) use phase_protocol::{
    ConsumedPhaseInput, NativePhaseBinding, NativePhaseSession, OwnedPhaseSettlement,
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
    pub receipt: Option<NativeResultId>,
    pub observed_work: Option<WorkOutcome>,
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
/// The private selected-vtable result preserves the same original handoff for
/// a typed wait/reconcile. Returning it is not Runtime-owned future retention.
pub(crate) enum NativePhaseStart {
    Launched {
        handle: ManagedSessionRef,
        binding: NativePhaseBinding,
    },
    Waiting {
        launch: Arc<crate::state::managed_binding::PhaseLaunchParts>,
        unit: Box<ExecutionUnit>,
        reason: WaitReason,
        next_due: i64,
    },
}
pub(crate) struct NativePhaseStartError {
    pub(crate) launch: Arc<crate::state::managed_binding::PhaseLaunchParts>,
    pub(crate) error: anyhow::Error,
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
    terminal: Arc<Mutex<Option<Arc<NativeTerminal>>>>,
    update: watch::Sender<NativeStatus>,
    phase: Option<Arc<phase_protocol::PhaseActor>>,
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
        self.start_with_launch(input, model, effort, executable, None)
            .await
    }
    /// The concrete installed vtable is the only caller supplying a genuine
    /// launch. Allocated DTOs and old public adapters never reach this entry.
    pub(crate) async fn start_phase(
        &self,
        launch: Arc<crate::state::managed_binding::PhaseLaunchParts>,
    ) -> std::result::Result<NativePhaseStart, NativePhaseStartError> {
        let result = self.start_phase_inner(launch.clone()).await;
        result.map_err(|error| NativePhaseStartError { launch, error })
    }
    async fn start_phase_inner(
        &self,
        launch: Arc<crate::state::managed_binding::PhaseLaunchParts>,
    ) -> Result<NativePhaseStart> {
        let allocation = launch.allocation();
        let selected = allocation.selected_port().selected_adapter()?;
        ensure!(
            std::ptr::eq(selected.sessions.as_ref(), self)
                && Arc::ptr_eq(&selected.owner, &self.owner)
                && launch.is_retained(),
            "Native launch lost actual selected sessions/retention"
        );
        let input = allocation.prepared_input().clone();
        let facts = allocation.facts();
        let model = facts.model.map(str::to_owned);
        let effort = facts.effort.map(str::to_owned);
        let executable = facts.program.to_owned();
        match self
            .start_with_launch(input, model, effort, Some(executable), Some(launch.clone()))
            .await?
        {
            NativeStart::Launched(handle) => {
                let binding = self.phase_binding(&handle)?;
                Ok(NativePhaseStart::Launched { handle, binding })
            }
            NativeStart::Waiting {
                unit,
                reason,
                next_due,
            } => Ok(NativePhaseStart::Waiting {
                launch,
                unit,
                reason,
                next_due,
            }),
        }
    }
    async fn start_with_launch(
        &self,
        input: ManagedInput,
        model: Option<String>,
        effort: Option<String>,
        executable: Option<std::path::PathBuf>,
        launch: Option<Arc<crate::state::managed_binding::PhaseLaunchParts>>,
    ) -> Result<NativeStart> {
        // Public ManagedInput/Unit identity is not a managed phase owner. This
        // standalone entry cannot qualify or prepare a protected Workflow.
        let protected = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .managed_phase_required(&input.authority.scope)?;
        ensure!(
            protected == launch.is_some(),
            NativeFailure::AuthorityUnavailable
        );
        // The current frame and exact allocated owner are mandatory before the
        // first helper. This is a private actual launch, not a legacy bypass.
        if let Some(parts) = &launch {
            let current =
                crate::state::managed_binding::plan_current_phase(&self.owner, parts.marker())?;
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .validate_phase_preparation(parts, &current)?;
        }
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
                && unit.phase != WORKFLOW_SOURCE_BOOTSTRAP
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
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .validate_native_input(&unit.authority(), &input.input)?;
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
            id: launch.as_ref().map_or_else(SessionId::new, |parts| {
                parts.allocation().facts().session_id
            }),
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
        let seed = NativeSeed {
            input: input.input.clone(),
            id: launch
                .as_ref()
                .map_or_else(NativeInvocationId::new, |parts| {
                    parts.allocation().facts().invocation_id
                }),
            profile: format!("text_v1/{}", unit.profile_digest),
            native_version: version.trim().into(),
        };
        let mut registration_guard = None;
        let registration = launch
            .as_ref()
            .map(|parts| {
                crate::state::Store::plan_native_phase_registration(
                    &self.owner,
                    parts.clone(),
                    session.clone(),
                    &seed,
                )
            })
            .transpose()?;
        let mut phase = None;
        let mut child = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            unit = if let Some(plan) = registration {
                let (registered, projection, version) = store.register_phase_session(plan)?;
                // Registration is already a known commit. Retain its exact Unit
                // before a later private-actor construction can fail, rather
                // than leaving the older preparation version as the only guard.
                registration_guard = Some(RegistrationGuard {
                    owner: self.owner.clone(),
                    unit_id: registered.id,
                    invocation: seed.id,
                    armed: true,
                });
                preparation_guard.update(&registered);
                let parts = launch
                    .as_ref()
                    .context("actual Native launch disappeared")?
                    .clone();
                phase = Some(phase_protocol::PhaseActor::registered(
                    parts, projection, version,
                )?);
                registered
            } else {
                store.register_native_session(&unit.authority(), &session, &seed)?
            };
            if registration_guard.is_none() {
                registration_guard = Some(RegistrationGuard {
                    owner: self.owner.clone(),
                    unit_id: unit.id,
                    invocation: seed.id,
                    armed: true,
                });
            }
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
            receipt: None,
            observed_work: None,
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
        let frozen_terminal = Arc::new(Mutex::new(None));
        let core = Core {
            owner: self.owner.clone(),
            unit,
            session,
            record_version: 1,
            invocation: seed.id,
            collector: native_result::Collector::default(),
            receipt_saved: false,
            captured_terminal: None,
            observed_input: None,
            observed_terminal: None,
            frozen_terminal: frozen_terminal.clone(),
            wire: Lines::new(stdin, stdout, limit),
            child,
            update: update.clone(),
            controls: receiver,
            native,
            drain,
            phase: phase.clone(),
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
                    terminal: frozen_terminal,
                    update,
                    phase,
                },
            );
        preparation_guard.disarm();
        if let Some(guard) = registration_guard.as_mut() {
            guard.armed = false;
        }
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
    /// Read a proof issued by the retained actual Native actor. Session IDs and
    /// persisted rows cannot manufacture this private owner observation.
    pub(crate) fn phase_binding(&self, handle: &ManagedSessionRef) -> Result<NativePhaseBinding> {
        let phase = {
            let entries = self
                .entries
                .lock()
                .map_err(|_| anyhow::anyhow!("native registry unavailable"))?;
            let entry = entries
                .get(&handle.session)
                .context("native Session missing")?;
            ensure!(entry.handle == *handle, "foreign native phase Session");
            entry
                .phase
                .clone()
                .context("actual native phase owner unavailable")?
        };
        // No registry lock spans projection copying or later Store validation.
        phase.owner.binding_snapshot()
    }
    pub fn status(&self, handle: &ManagedSessionRef) -> Result<NativeStatus> {
        let mut status = self.entry(handle)?.0.borrow().clone();
        let (pending, updater, phase) = {
            let entries = self
                .entries
                .lock()
                .map_err(|_| anyhow::anyhow!("native registry poisoned"))?;
            let entry = entries
                .get(&handle.session)
                .context("native Session missing")?;
            (
                entry.terminal.clone(),
                entry.update.clone(),
                entry.phase.clone(),
            )
        };
        let mut reconciled = false;
        let frozen = pending
            .lock()
            .map_err(|_| anyhow::anyhow!("native terminal poisoned"))?
            .clone();
        if let Some(proof) = frozen.as_ref() {
            status.observed_work = Some(proof.receipt.observed_work);
            if persist_saved_terminal(&self.owner, phase.as_ref(), proof).is_ok() {
                clear_saved_terminal(&pending, proof)?;
                reconciled = true;
                status.diagnostic = None;
            }
        }
        let store = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        let unit = store.execution_unit(handle.unit)?;
        if let Some((session, _)) = store.session(handle.session)? {
            status.session = session;
        }
        if let Some(receipt) = store.native_session_result(handle.session)? {
            status.receipt = Some(receipt.id);
            status.result = Some(receipt.projection());
        }
        status.authority = unit.authority();
        status.work = unit.work;
        status.disposition = unit.disposition;
        status.cleanup = unit.cleanup;
        status.wait_reason = unit.wait_reason;
        if reconciled {
            updater.send_modify(|current| current.clone_from(&status));
        }
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
            entry.phase.is_none(),
            "actual managed phase proofs require a private confirmed handoff before release"
        );
        // Logical terminal state is durable before hygiene/watch publication. A
        // caller may observe that committed result and release the registry entry;
        // the supervisor retains its OwnedProcess until stop_and_reap completes.
        let store = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        let unit = store.execution_unit(handle.unit)?;
        let (session, _) = store
            .session(handle.session)?
            .context("native Session missing")?;
        ensure!(
            unit.scope == handle.scope
                && unit.generation == handle.generation
                && unit.owner_epoch == handle.epoch
                && unit.session_id == Some(handle.session)
                && session.scope == handle.scope
                && !unit.native_effects_open
                && matches!(
                    session.state,
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
    evidence: Option<native_result::WireEvidence>,
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
            evidence: None,
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
            if chunk.is_empty() {
                self.evidence = Some(native_result::WireEvidence {
                    observed_sha256: native_result::digest(&self.partial),
                    observed_bytes: self.partial.len() as u64,
                    exact_length: false,
                    category: "eof".into(),
                });
                anyhow::bail!(NativeFailure::TransportLost);
            }
            let newline = chunk.iter().position(|b| *b == b'\n');
            let n = newline.map_or(chunk.len(), |i| i + 1);
            if self.partial.len() + n > self.limit {
                use sha2::{Digest, Sha256};
                let mut digest = Sha256::new();
                digest.update(&self.partial);
                digest.update(&chunk[..n]);
                self.evidence = Some(native_result::WireEvidence {
                    observed_sha256: format!("{:x}", digest.finalize()),
                    observed_bytes: (self.partial.len() + n) as u64,
                    exact_length: false,
                    category: "frame_bytes".into(),
                });
                anyhow::bail!("native frame too large");
            }
            self.partial.extend_from_slice(&chunk[..n]);
            self.stdout.consume(n);
            if newline.is_some() {
                let bytes = std::mem::take(&mut self.partial);
                let decoded = super::strict_json::decode(
                    &bytes,
                    super::strict_json::Limits {
                        frame_bytes: self.limit.min(4 * 1024 * 1024),
                        depth: 32,
                        nodes: 65_536,
                        string_bytes: 1024 * 1024,
                        total_string_bytes: 4 * 1024 * 1024,
                        object_entries: 4096,
                        array_entries: 4096,
                    },
                );
                if let Err(error) = &decoded {
                    use super::strict_json::Error;
                    let category = match error {
                        Error::InvalidLimits | Error::InvalidJson => "invalid_json",
                        Error::FrameBytes => "frame_bytes",
                        Error::Depth => "depth",
                        Error::Nodes => "nodes",
                        Error::StringBytes => "string_bytes",
                        Error::TotalStringBytes => "total_string_bytes",
                        Error::ObjectEntries => "object_entries",
                        Error::ArrayEntries => "array_entries",
                        Error::DuplicateKey => "duplicate_key",
                    };
                    self.evidence = Some(native_result::WireEvidence {
                        observed_sha256: native_result::digest(&bytes),
                        observed_bytes: bytes.len() as u64,
                        exact_length: true,
                        category: category.into(),
                    });
                }
                return decoded.map_err(anyhow::Error::from);
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
    invocation: NativeInvocationId,
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
    ensure!(
        !store.managed_phase_required(&pinned.scope)?,
        NativeFailure::AuthorityUnavailable
    );
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
    let effect = ManagedEffect {
        id: operation,
        unit_id: current.id,
        scope: current.scope,
        kind: kind.into(),
        idempotency_key: format!("native-{operation}"),
        expected_target: format!("session/{session}/sha256/{digest}"),
        state: EffectState::Pending,
        receipt: BTreeMap::new(),
        version: 1,
    };
    if kind == "native_input" {
        store.reserve_native_input(&authority, invocation, &effect, &digest)?;
    } else {
        store.reserve_managed_effect(&authority, &effect)?;
    }
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
    ensure!(
        !store.managed_phase_required(&pinned.scope)?,
        NativeFailure::AuthorityUnavailable
    );
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
fn actual_native_authority(
    owner: &RuntimeOwner,
    pinned: &ExecutionUnit,
    session: SessionId,
    phase: Option<&Arc<NativePhaseSession>>,
) -> Result<ExecutionAuthority> {
    let Some(phase) = phase else {
        return native_authority(owner, pinned, session);
    };
    let facts = phase.allocation().facts();
    ensure!(
        facts.unit_id == pinned.id
            && facts.scope == &pinned.scope
            && facts.generation == pinned.generation
            && facts.epoch == pinned.owner_epoch
            && facts.session_id == session,
        "actual Native actor semantic identity changed"
    );
    let plan = crate::state::Store::plan_native_phase_owner(owner, phase)?;
    owner
        .store
        .lock()
        .map_err(|_| anyhow::anyhow!("state poisoned"))?
        .validate_phase_owner(plan)
}
// These non-Clone private-field values originate only at the actual Native actor.
// DTO/GenericRecord/tool JSON cannot construct a dispatch or terminal producer.
pub(crate) struct NativeSeed {
    input: PreparedInput,
    id: NativeInvocationId,
    profile: String,
    native_version: String,
}
impl NativeSeed {
    pub(crate) fn id(&self) -> NativeInvocationId {
        self.id
    }
    pub(crate) fn input(&self) -> &PreparedInput {
        &self.input
    }
    pub(crate) fn invocation(
        &self,
        unit: &ExecutionUnit,
        session: &Session,
        context_hash: Option<String>,
        artifact_version: Option<u64>,
    ) -> native_result::NativeInvocation {
        native_result::NativeInvocation {
            id: self.id,
            unit_id: unit.id,
            session_id: session.id,
            scope: unit.scope.clone(),
            generation: unit.generation,
            owner_epoch: unit.owner_epoch,
            provider: unit.provider.clone(),
            profile: self.profile.clone(),
            native_version: self.native_version.clone(),
            unit_version: unit.version,
            session_version: 1,
            context_version: context_hash.as_ref().map(|_| self.input.version),
            context_sha256: context_hash,
            source_versions: self.input.source_versions.clone(),
            source_sha256: native_result::digest(
                &serde_json::to_vec(&self.input.source_versions).expect("source encoding"),
            ),
            revision: self.input.revision.clone(),
            artifact_id: unit.artifact_id,
            artifact_version,
            payload_sha256: native_result::digest(self.input.payload.as_bytes()),
            input_operation: None,
            frame_sha256: None,
            native_thread: None,
            native_turn: None,
            state: native_result::InvocationState::NotDispatched,
            version: 1,
        }
    }
}
struct RegistrationGuard {
    owner: Arc<RuntimeOwner>,
    unit_id: UnitId,
    invocation: NativeInvocationId,
    armed: bool,
}
impl Drop for RegistrationGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let Ok(mut store) = self.owner.store.lock() else {
            return;
        };
        let Ok(invocation) = store.native_invocation(self.invocation) else {
            return;
        };
        if invocation.state == native_result::InvocationState::Closed {
            return;
        }
        let Ok(mut unit) = store.execution_unit(self.unit_id) else {
            return;
        };
        if unit.native_effects_open {
            let Ok(retired) =
                store.retire_execution_as(&unit.authority(), false, Disposition::Lost)
            else {
                return;
            };
            unit = retired;
        }
        let Ok(Some((session, version))) = store.session(invocation.session_id) else {
            return;
        };
        let receipt = native_result::NativeResultReceipt {
            id: NativeResultId::new(),
            invocation_id: invocation.id,
            unit_id: unit.id,
            session_id: session.id,
            scope: unit.scope.clone(),
            generation: unit.generation,
            owner_epoch: unit.owner_epoch,
            provider: unit.provider.clone(),
            native_thread: invocation.native_thread,
            native_turn: invocation.native_turn,
            acquisition: native_result::AcquisitionStatus::Missing,
            authority: native_result::ReceiptAuthority::HistoricalDraft,
            text: None,
            structured_output: None,
            prefix: None,
            answer_sha256: None,
            terminal_sha256: None,
            wire: None,
            observed_work: WorkOutcome::Unknown,
            disposition: Disposition::Lost,
            diagnostics: vec!["native_supervisor_unavailable".into()],
            captured_at: now_ms(),
            version: 1,
        };
        let _ = store.finish_native_result(&NativeTerminal {
            receipt,
            session,
            failure: None,
            session_version: version,
        });
    }
}
pub(crate) struct NativeTerminal {
    receipt: native_result::NativeResultReceipt,
    session: Session,
    failure: Option<NativeFailure>,
    session_version: u64,
}
impl NativeTerminal {
    fn persist(
        &self,
        store: &mut crate::state::Store,
    ) -> Result<(ExecutionUnit, native_result::NativeResultReceipt, Session)> {
        let current = store.execution_unit(self.receipt.unit_id)?;
        // Retrying a private observed terminal never reopens permissions. If the
        // governing input changed during a storage outage, retain it as history.
        if current.native_effects_open
            && store
                .validate_execution(&current.authority(), false, true)
                .is_err()
        {
            store.retire_execution_as(&current.authority(), false, Disposition::Lost)?;
        }
        store.finish_native_result(self)
    }
    pub(crate) fn session_version(&self) -> u64 {
        self.session_version
    }
    pub(crate) fn receipt(&self) -> &native_result::NativeResultReceipt {
        &self.receipt
    }
    pub(crate) fn session(&self) -> &Session {
        &self.session
    }
    pub(crate) fn failure(&self) -> Option<NativeFailure> {
        self.failure
    }
}
/// Retry only the actual saved terminal and its original private phase plan.
/// Full planning/projection runs outside SharedStore; no persisted DTO issues a
/// phase owner or a new input/settlement proof.
fn retain_saved_terminal(
    pending: &Mutex<Option<Arc<NativeTerminal>>>,
    proof: &Arc<NativeTerminal>,
) -> Result<()> {
    let mut frozen = pending
        .lock()
        .map_err(|_| anyhow::anyhow!("native terminal poisoned"))?;
    if let Some(original) = frozen.as_ref() {
        ensure!(
            Arc::ptr_eq(original, proof),
            "different actual native terminal already captured"
        );
    } else {
        *frozen = Some(proof.clone());
    }
    Ok(())
}
fn clear_saved_terminal(
    pending: &Mutex<Option<Arc<NativeTerminal>>>,
    proof: &Arc<NativeTerminal>,
) -> Result<()> {
    let mut frozen = pending
        .lock()
        .map_err(|_| anyhow::anyhow!("native terminal poisoned"))?;
    if let Some(original) = frozen.as_ref() {
        ensure!(
            Arc::ptr_eq(original, proof),
            "native terminal compare-clear origin changed"
        );
        *frozen = None;
    }
    Ok(())
}
fn persist_saved_terminal(
    owner: &RuntimeOwner,
    phase: Option<&Arc<phase_protocol::PhaseActor>>,
    terminal: &Arc<NativeTerminal>,
) -> Result<(
    ExecutionUnit,
    native_result::NativeResultReceipt,
    Session,
    u64,
)> {
    if let Some(phase) = phase {
        let original = phase.terminal_plan(owner, terminal)?;
        let first = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .finish_phase_terminal(&original);
        let commit = match first {
            Ok(commit) => commit,
            Err(error) => {
                // A failed commit might actually have committed. Only actual
                // receipt absence permits replacing its original plan. At most
                // one confirmed-rollback replan is attempted per observation.
                if !original.confirmed_absent(owner)? {
                    return Err(error);
                }
                let next = phase.replan_terminal_after_absence(owner, &original, terminal)?;
                owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .finish_phase_terminal(&next)?
            }
        };
        let (saved, actual_owner, unit, receipt, session, version, owned_success) =
            commit.into_parts();
        ensure!(
            Arc::ptr_eq(&saved, terminal) && Arc::ptr_eq(&actual_owner, &phase.owner),
            "native terminal transaction changed private origin"
        );
        if owned_success {
            phase.settled(
                saved,
                unit.clone(),
                receipt.clone(),
                session.clone(),
                version,
            )?;
        } else {
            phase.owner.project(&session, version)?;
            phase.owner.revoke();
        }
        return Ok((unit, receipt, session, version));
    }
    let mut store = owner
        .store
        .lock()
        .map_err(|_| anyhow::anyhow!("state poisoned"))?;
    let (unit, receipt, session) = terminal.persist(&mut store)?;
    let version = store
        .session(session.id)?
        .context("native terminal Session missing")?
        .1;
    Ok((unit, receipt, session, version))
}
struct Core {
    owner: Arc<RuntimeOwner>,
    unit: ExecutionUnit,
    session: Session,
    record_version: u64,
    invocation: NativeInvocationId,
    collector: native_result::Collector,
    receipt_saved: bool,
    captured_terminal: Option<Arc<NativeTerminal>>,
    observed_input: Option<(String, Option<String>)>,
    observed_terminal: Option<(WorkOutcome, Disposition, Option<NativeFailure>)>,
    frozen_terminal: Arc<Mutex<Option<Arc<NativeTerminal>>>>,
    wire: Lines,
    child: process::OwnedProcess,
    update: watch::Sender<NativeStatus>,
    controls: mpsc::Receiver<Control>,
    native: String,
    drain: tokio::task::JoinHandle<()>,
    phase: Option<Arc<phase_protocol::PhaseActor>>,
}
impl Core {
    fn retain_input_ack(&mut self, thread: &str, turn: Option<&str>) -> Result<()> {
        claude_wire::bounded_id(&json!(thread))?;
        if let Some(turn) = turn {
            claude_wire::bounded_id(&json!(turn))?;
        }
        if let Some((old_thread, old_turn)) = &self.observed_input {
            ensure!(
                old_thread == thread && old_turn.as_deref() == turn,
                "actual owned input acknowledgement changed"
            );
        } else {
            self.observed_input = Some((thread.into(), turn.map(str::to_owned)));
        }
        if let Some(phase) = &self.phase {
            phase.consumed()?.acknowledge(thread, turn)?;
        }
        Ok(())
    }
    fn authority(&self) -> Result<ExecutionAuthority> {
        actual_native_authority(
            &self.owner,
            &self.unit,
            self.session.id,
            self.phase.as_ref().map(|phase| &phase.owner),
        )
        .context(NativeFailure::AuthorityUnavailable)
    }
    fn admit(
        &self,
        value: &Value,
        authority: Option<&ExecutionAuthority>,
        kind: &str,
    ) -> Result<OperationId> {
        let Some(phase) = &self.phase else {
            return admit_native_frame(
                &self.owner,
                &self.unit,
                self.session.id,
                self.invocation,
                authority,
                kind,
                value,
            );
        };
        let plan = crate::state::Store::plan_native_phase_dispatch(
            &self.owner,
            &phase.owner,
            authority,
            kind,
            value,
        )?;
        let commit = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .admit_phase_dispatch(plan)?;
        let (operation, digest, expected_thread, input) = commit.into_parts();
        if input {
            // Only the successful actual same-TX input intent reaches this
            // private producer; retain it BEFORE the first wire await.
            phase.admitted(operation, digest, expected_thread)?;
        }
        Ok(operation)
    }
    fn acknowledge_input(&self, thread: &str, turn: Option<&str>) -> Result<()> {
        if let Some(phase) = &self.phase {
            let consumed = phase.consumed()?;
            // This is an observed response from the owned protocol, not a DB
            // lookup. Retain it across a later persistence error or lost return.
            consumed.acknowledge(thread, turn)?;
            let plan = crate::state::Store::plan_native_phase_input_ack(
                &self.owner,
                &phase.owner,
                consumed,
                thread,
                turn,
            )?;
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .acknowledge_phase_input(plan)
        } else {
            let authority = self.authority()?;
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .ack_native_invocation(&authority, self.invocation, thread, turn)
        }
    }
    async fn send_effect(
        &mut self,
        value: &Value,
        authority: Option<&ExecutionAuthority>,
        kind: &str,
    ) -> Result<()> {
        let operation = self.admit(value, authority, kind)?;
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
        // Every actual phase request crosses the same before-wire journal;
        // metadata/setup has no input-consumption authority of its own.
        let kind = if self.phase.is_some() {
            Some(kind.unwrap_or("native_setup"))
        } else {
            kind
        };
        let operation = kind
            .map(|kind| {
                self.admit(
                    &json!({"id":self.wire.next,"method":method,"params":params}),
                    None,
                    kind,
                )
            })
            .transpose()?;
        let owner = self.owner.clone();
        let pinned = self.unit.clone();
        let session = self.session.id;
        let phase = self.phase.as_ref().map(|phase| phase.owner.clone());
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
                    _=fence.tick()=>{if let Err(error)=actual_native_authority(&owner,&pinned,session,phase.as_ref()){break Err(error);}}
                }
            }
        };
        if method == "turn/start"
            && let Ok(response) = &result
        {
            let turn = claude_wire::bounded_id(&response["turn"]["id"])?;
            let thread = self
                .session
                .native_ref
                .clone()
                .context("actual turn response lacks established thread")?;
            // Save the owned response BEFORE any transport-receipt write. A
            // Store fault below must not erase an ACK that was already read.
            self.retain_input_ack(&thread, Some(&turn))?;
        }
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
        if self.unit.provider == "claude" {
            // RunState accepted this exact owned initialize observation. Retain
            // input consumption before Session/Unit projection can fail.
            self.retain_input_ack(&native, None)?;
        }
        let mut session = self.session.clone();
        session.native_ref = Some(native);
        session.state = SessionState::Running;
        let (unit, version) = if let Some(phase) = &self.phase {
            let plan = crate::state::Store::plan_native_phase_projection(
                &self.owner,
                &phase.owner,
                session,
            )?;
            let (unit, committed, version) = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .project_phase_session(plan)?;
            phase.owner.project(&committed, version)?;
            self.session = committed;
            (unit, version)
        } else {
            let authority = self.authority()?;
            let result = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .update_execution_session(&authority, &session, self.record_version)?;
            self.session = session;
            result
        };
        self.unit = unit;
        self.record_version = version;
        if self.unit.provider == "claude" {
            self.acknowledge_input(&self.native, None)?;
        }
        self.update.send_modify(|s| {
            s.session = self.session.clone();
            s.authority = self.unit.authority();
            s.wait_reason = self.unit.wait_reason;
        });
        Ok(())
    }
    fn capture(
        &mut self,
        work: WorkOutcome,
        disposition: Disposition,
        failure: Option<NativeFailure>,
    ) -> Result<NativeTerminal> {
        let native_result::Content {
            acquisition,
            text,
            prefix,
            answer_sha256,
            terminal_sha256,
            diagnostics,
        } = self.collector.content();
        let (actual_ack, projected_session, projected_version) = if let Some(phase) = &self.phase {
            let binding = phase.owner.binding_snapshot()?;
            (
                binding
                    .consumed()
                    .map(|input| input.acknowledgement())
                    .transpose()?
                    .flatten(),
                binding.session().clone(),
                binding.record_version(),
            )
        } else {
            (
                self.observed_input.clone(),
                self.session.clone(),
                self.record_version,
            )
        };
        let mut receipt = native_result::NativeResultReceipt {
            id: NativeResultId::new(),
            invocation_id: self.invocation,
            unit_id: self.unit.id,
            session_id: self.session.id,
            scope: self.unit.scope.clone(),
            generation: self.unit.generation,
            owner_epoch: self.unit.owner_epoch,
            provider: self.unit.provider.clone(),
            native_thread: actual_ack.as_ref().map(|ack| ack.0.clone()),
            native_turn: actual_ack.as_ref().and_then(|ack| ack.1.clone()),
            acquisition,
            authority: native_result::ReceiptAuthority::HistoricalDraft,
            text,
            structured_output: None,
            prefix,
            answer_sha256,
            terminal_sha256,
            wire: self.wire.evidence.clone(),
            observed_work: work,
            disposition,
            diagnostics,
            captured_at: now_ms(),
            version: 1,
        };
        if receipt.validate().is_err() {
            // JSON escaping can exceed the encoded receipt budget before decoded text does.
            // Preserve only a bounded UTF-8 evidence prefix, never a misleading full hash.
            let text = receipt.text.take().unwrap_or_default();
            let mut n = text.len().min(native_result::PREFIX_BYTES);
            while !text.is_char_boundary(n) {
                n -= 1;
            }
            let prefix = text[..n].to_owned();
            receipt.acquisition = native_result::AcquisitionStatus::Overflow;
            receipt.answer_sha256 = None;
            receipt.prefix = Some(native_result::PrefixEvidence {
                prefix_sha256: native_result::digest(prefix.as_bytes()),
                prefix,
                observed_bytes: text.len() as u64,
                exact_length: true,
                unseen_suffix: text.len() > n,
            });
            receipt.diagnostics = vec!["answer_encoded_limit".into()];
            receipt.validate()?;
        }
        Ok(NativeTerminal {
            receipt,
            session: projected_session,
            failure,
            session_version: projected_version,
        })
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
        let (mut work, mut disposition, _output, mut failure) = match result {
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
        if let Some((observed_work, observed_disposition, observed_failure)) =
            self.observed_terminal
        {
            // This came from the owned, validated terminal frame. Subsequent
            // optional quota/storage bookkeeping is not a Native work failure.
            work = observed_work;
            disposition = observed_disposition;
            failure = observed_failure;
        }
        if disposition == Disposition::Completed && work != WorkOutcome::Unknown {
            self.collector.confirm_complete();
        }
        if failure == Some(NativeFailure::AuthenticationUnavailable) {
            self.collector.discard_sensitive();
        } else if failure.is_some() {
            self.collector.protocol_lost();
            if let Some(wire) = &self.wire.evidence {
                self.collector.wire_failed(wire);
            }
        }
        // Content, native work, Session closure and quota release commit together.
        // Hygiene runs afterward and cannot replace the known work axis.
        let persisted = (|| {
            // Capture actual owned protocol facts BEFORE the first Store read.
            // No Store outage can force a known terminal to be recaptured Unknown.
            let proof = Arc::new(self.capture(work, disposition, failure)?);
            self.captured_terminal = Some(proof.clone());
            let pending = self.frozen_terminal.clone();
            retain_saved_terminal(&pending, &proof)?;
            let (unit, receipt, session, version) =
                persist_saved_terminal(&self.owner, self.phase.as_ref(), &proof)?;
            clear_saved_terminal(&pending, &proof)?;
            self.unit = unit;
            self.session = session;
            self.record_version = version;
            self.receipt_saved = true;
            Ok::<_, anyhow::Error>(receipt)
        })();
        let stopped = self.child.stop_and_reap().await;
        self.drain.abort();
        #[cfg(test)]
        if profile.output.join("fixture-delay-publication").exists() {
            // An account-free fixture may schedule a status reconciliation after
            // native terminal capture but before the supervisor publishes watch.
            std::fs::write(profile.output.join("fixture-publication-ready"), "ready").unwrap();
            tokio::time::timeout(Duration::from_secs(5), async {
                while !profile.output.join("fixture-publication-release").exists() {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await
            .unwrap();
        }
        let mut store = match self.owner.store.lock() {
            Ok(store) => store,
            Err(_) => return,
        };
        let current = match store.execution_unit(self.unit.id) {
            Ok(unit) => unit,
            Err(_) => return,
        };
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
        let saved_receipt = store.native_session_result(self.session.id).ok().flatten();
        let saved_session = store
            .session(self.session.id)
            .ok()
            .flatten()
            .map(|(session, _)| session);
        self.update.send_modify(|s| {
            s.authority = final_authority;
            if let Some(session) = saved_session {
                s.session = session;
            }
            s.work = current.work;
            s.wait_reason = current.wait_reason;
            s.disposition = current.disposition;
            s.cleanup = CleanupOutcome::Unknown;
            s.pending.clear();
            s.observed_work = Some(work);
            // A concurrent status retry may have committed the sealed terminal
            // while hygiene awaited. The durable result outranks that stale Err.
            s.receipt = saved_receipt.as_ref().map(|r| r.id);
            s.result = saved_receipt.as_ref().map(|r| r.projection());
            s.failure = if current.disposition == Disposition::Cancelled {
                None
            } else {
                failure
            };
            s.diagnostic = if let Some(failure) = s.failure {
                Some(failure.diagnostic())
            } else if persisted.is_err() && saved_receipt.is_none() {
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
    fn drop_phase(&mut self) {
        self.drain.abort();
        let pending = self.frozen_terminal.clone();
        let Ok(mut proof) = pending.lock().map(|f| f.clone()) else {
            return;
        };
        if proof.is_none() {
            proof = self.captured_terminal.clone();
        }
        let saved_receipt = self
            .owner
            .store
            .lock()
            .ok()
            .and_then(|store| store.native_session_result(self.session.id).ok().flatten());
        if saved_receipt.is_some() {
            self.receipt_saved = true;
        }
        if proof.is_none() && !self.receipt_saved {
            self.collector.protocol_lost();
            if let Ok(captured) = self.capture(WorkOutcome::Unknown, Disposition::Lost, None) {
                let captured = Arc::new(captured);
                self.captured_terminal = Some(captured.clone());
                if retain_saved_terminal(&pending, &captured).is_ok() {
                    proof = Some(captured);
                }
            }
        }
        let mut terminal_pending = proof.is_some();
        let observed = proof.as_ref().map(|proof| proof.receipt.observed_work);
        if let Some(proof) = proof.as_ref()
            && let Ok((unit, _, session, version)) =
                persist_saved_terminal(&self.owner, self.phase.as_ref(), proof)
        {
            self.unit = unit;
            self.session = session;
            self.record_version = version;
            self.receipt_saved = true;
            if clear_saved_terminal(&pending, proof).is_ok() {
                terminal_pending = false;
            }
        }
        let Ok(store) = self.owner.store.lock() else {
            return;
        };
        let Ok(unit) = store.execution_unit(self.unit.id) else {
            return;
        };
        let session = store.session(self.session.id).ok().flatten().map(|s| s.0);
        let receipt = store.native_session_result(self.session.id).ok().flatten();
        self.update.send_modify(|s| {
            if let Some(session) = session {
                s.session = session;
            }
            s.authority = unit.authority();
            s.work = unit.work;
            s.disposition = unit.disposition;
            s.wait_reason = unit.wait_reason;
            s.cleanup = unit.cleanup;
            s.pending.clear();
            if let Some(receipt) = receipt {
                s.receipt = Some(receipt.id);
                s.result = Some(receipt.projection());
                s.observed_work = Some(receipt.observed_work);
            }
            if unit.disposition == Disposition::Cancelled {
                s.failure = None;
            }
            if terminal_pending {
                s.observed_work = observed;
                s.diagnostic = Some("native terminal pending persistence");
            } else if s.diagnostic == Some("native terminal pending persistence") {
                s.diagnostic = None;
            }
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
        let initialized = json!({"method":"initialized","params":{}});
        if self.phase.is_some() {
            self.send_effect(&initialized, None, "native_setup").await?;
        } else {
            self.wire.send(&initialized).await?;
        }
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
        self.acknowledge_input(&thread_id, Some(&turn_id))?;
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
                        Some("item/started")=>{approvals.item(&p["item"])?; if p["threadId"]==thread_id && p["turnId"]==turn_id {self.collector.codex_item(&p["item"],false);}},
                        Some("item/completed") if p["threadId"]==thread_id && p["turnId"]==turn_id=>self.collector.codex_item(&p["item"],true),
                        Some("item/agentMessage/delta") if p["threadId"]==thread_id && p["turnId"]==turn_id=>self.collector.codex_delta(p),
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
                            if matches!(status,"completed"|"failed") {
                                self.collector.codex_terminal(&p["turn"]);
                                self.observed_terminal=Some(if status=="completed" {(WorkOutcome::Success,Disposition::Completed,None)}
                                    else if quota_ended || quota::codex_subscription_error(&p["turn"]["error"]) {(WorkOutcome::Unknown,Disposition::QuotaInterrupted,None)}
                                    else if p["turn"]["error"]["codexErrorInfo"]=="unauthorized" {(WorkOutcome::Unknown,Disposition::Refused,Some(NativeFailure::AuthenticationUnavailable))}
                                    else if quota::codex_capacity_error(&p["turn"]["error"]) {(WorkOutcome::Unknown,Disposition::CapacityInterrupted,None)}
                                    else {(WorkOutcome::Failure,Disposition::Completed,None)});
                            }
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
                    if self.collector.overflowed() {return Ok((WorkOutcome::Unknown,Disposition::ProtocolError,None));}
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
        let initialize = claude_wire::control("initialize", "rrx-initialize");
        if self.phase.is_some() {
            self.send_effect(&initialize, None, "native_setup").await?;
        } else {
            self.wire.send(&initialize).await?;
        }
        let owner = self.owner.clone();
        let pinned = self.unit.clone();
        let session = self.session.id;
        let phase = self.phase.as_ref().map(|phase| phase.owner.clone());
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
                _=fence.tick()=>{actual_native_authority(&owner,&pinned,session,phase.as_ref())?;}
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
                    if frame["type"]=="result" && frame["session_id"]==self.native {
                        if seen.as_ref().is_ok_and(|accepted|*accepted) {self.collector.claude_terminal(&frame);}
                        else if seen.as_ref().is_ok_and(|accepted|!*accepted) {self.collector.claude_redelivery(&frame);}
                    }
                    if self.collector.overflowed() {return Ok((WorkOutcome::Unknown,Disposition::ProtocolError,None));}

                    if seen.as_ref().is_err_and(|e|e.kind==ErrorKind::ProcessFailure) {
                        // Capture this accepted owned terminal BEFORE optional Store
                        // bookkeeping. A quota-sensitive execution error remains
                        // Unknown if accepted bucket state cannot be read; an outage
                        // cannot invent work failure or subscription exhaustion.
                        self.collector.claude_terminal(&frame);
                        let quota_sensitive=matches!(frame["subtype"].as_str(),Some("success"|"error_during_execution"));
                        let provisional=if quota_sensitive && !quota_buckets.is_empty() {
                            (WorkOutcome::Unknown,Disposition::Lost,None)
                        } else if state.terminal_capacity || (unclassified_limit && quota_sensitive) {
                            (WorkOutcome::Unknown,Disposition::CapacityInterrupted,None)
                        } else {
                            (WorkOutcome::Failure,Disposition::Completed,None)
                        };
                        self.observed_terminal=Some(provisional);
                        // Consult accepted bucket state, not the last telemetry frame.
                        // Budget/turn/output caps remain work failures even during quota exhaustion.
                        let quota_exhausted=quota_sensitive && !quota_buckets.is_empty() && self.owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?
                            .quota_observations("claude","unknown")?.iter().any(|o|o.status==QuotaStatus::Exhausted && quota_buckets.contains(&o.bucket));
                        let observed=if quota_exhausted {(WorkOutcome::Unknown,Disposition::QuotaInterrupted,None)}
                            else if state.terminal_capacity || (unclassified_limit && quota_sensitive) {(WorkOutcome::Unknown,Disposition::CapacityInterrupted,None)}
                            else {(WorkOutcome::Failure,Disposition::Completed,None)};
                        self.observed_terminal=Some(observed);
                        return Ok((observed.0,observed.1,if observed.0==WorkOutcome::Failure {Some(frame)} else {None}));
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
        if let Some(phase) = &self.phase {
            // Revoke normal/live delivery before every early Drop return. A
            // previously retained known settlement has its separate late path.
            phase.owner.revoke();
            self.drop_phase();
            return;
        }
        self.drain.abort();
        let owner = self.owner.clone();
        let Ok(mut store) = owner.store.lock() else {
            return;
        };
        let Ok(mut unit) = store.execution_unit(self.unit.id) else {
            return;
        };
        let pending = self.frozen_terminal.clone();
        let Ok(mut proof) = pending.lock().map(|f| f.clone()) else {
            return;
        };
        if proof.is_none() {
            proof = self.captured_terminal.clone();
        }
        let mut terminal_pending = proof.is_some();
        if let Some(proof) = proof.as_ref()
            && let Ok((saved, _, session)) = proof.persist(&mut store)
        {
            unit = saved;
            self.session = session;
            self.receipt_saved = true;
            if clear_saved_terminal(&pending, proof).is_ok() {
                terminal_pending = false;
            }
        }
        let abandoned = unit.native_effects_open && !terminal_pending;
        if abandoned {
            let Ok(retired) =
                store.retire_execution_as(&unit.authority(), false, Disposition::Lost)
            else {
                return;
            };
            unit = retired;
        }
        if !self.receipt_saved && !terminal_pending {
            self.collector.protocol_lost();
            if let Ok(proof) = self.capture(WorkOutcome::Unknown, Disposition::Lost, None)
                && let Ok((_, _, session)) = store.finish_native_result(&proof)
            {
                self.session = session;
                self.receipt_saved = true;
            }
        }
        if !unit.native_effects_open
            && let Ok(Some((mut session, version))) = store.session(self.session.id)
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
        let saved_receipt = store.native_session_result(self.session.id).ok().flatten();
        self.update.send_modify(|s| {
            if let Some(session) = session {
                s.session = session;
            }
            s.authority = unit.authority();
            s.work = unit.work;
            s.disposition = unit.disposition;
            s.wait_reason = unit.wait_reason;
            s.cleanup = unit.cleanup;
            if let Some(receipt) = saved_receipt {
                s.receipt = Some(receipt.id);
                s.result = Some(receipt.projection());
                s.observed_work = Some(receipt.observed_work);
                if s.diagnostic == Some("native terminal pending persistence") {
                    s.diagnostic = None;
                }
            }
            if unit.disposition == Disposition::Cancelled {
                s.failure = None;
            }
            s.pending.clear();
            if terminal_pending {
                s.observed_work = proof.as_ref().map(|proof| proof.receipt.observed_work);
                s.diagnostic = Some("native terminal pending persistence");
            }
            if abandoned {
                s.diagnostic = Some("native supervisor ended without terminal");
            }
        });
    }
}
