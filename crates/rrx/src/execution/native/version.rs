//! Retained finite P helpers. Neither a receipt nor exit zero completes
//! protected Native preparation, registration, input or Session authority.
use super::*;
use crate::state::{
    NativeHelperIntentCommit, NativeHelperSettlementCommit, NativeHelperSettlementPlan,
    NativeVersionClosurePlan, NativeVersionHelperPlan,
};
use std::io::Read;
use std::os::unix::process::ExitStatusExt;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::AsyncReadExt;

pub(super) const OUTPUT_BYTES: usize = 64 * 1024;
const PROFILE_BYTES: usize = 64 * 1024;

/// Nongrant acknowledgement of this original actor's readonly helper history.
/// This is not full Native preparation: hooks, quota admission, allocation proof
/// and Reviewer artifact lease remain separate required producers/consumers.
/// It grants neither registration nor transport/input or static admission.
pub(crate) struct NativeReadonlyHelperCompletion {
    actor: Arc<NativePreparationActor>,
    _correspondence: readonly::NativeGitCorrespondence,
    pub(crate) commit: crate::state::NativeHelperHistoryCommit,
    pub(super) contract: Option<super::super::codex_wire::CodexWireContract>,
}
impl NativeReadonlyHelperCompletion {
    pub(crate) fn matches_actor(&self, actor: &Arc<NativePreparationActor>) -> bool {
        Arc::ptr_eq(&self.actor, actor) && self.commit.matches_actor(actor)
    }
}

#[derive(Default)]
struct BoundedOutput {
    stdout: Vec<u8>,
    bytes: usize,
    limit: usize,
}
impl BoundedOutput {
    fn read_window(&self) -> usize {
        (self.limit.saturating_sub(self.bytes) + 1).min(4096)
    }
    fn observe(&mut self, bytes: &[u8], stdout: bool) -> bool {
        let Some(total) = self.bytes.checked_add(bytes.len()) else {
            return false;
        };
        self.bytes = total;
        if stdout {
            let available = self.limit.saturating_sub(self.stdout.len());
            self.stdout
                .extend_from_slice(&bytes[..bytes.len().min(available)]);
        }
        self.bytes <= self.limit
    }
}

pub(super) struct NativeVersionHelperCustody {
    plan: Arc<NativeVersionHelperPlan>,
    ipc: Arc<super::super::ipc::PreparationIpcSink>,
    raw: Mutex<process::RetainedRawProcess>,
    state: Mutex<HelperState>,
    cancelled: AtomicBool,
    done: tokio::sync::Notify,
    output_limit: usize,
    git_lease: Mutex<Option<Arc<owner::GitLease>>>,
}
#[derive(Default)]
struct HelperState {
    intent: Option<Arc<NativeHelperIntentCommit>>,
    attempted: bool,
    started: Option<tokio::time::Instant>,
    observation: Option<Arc<NativeVersionObservation>>,
    settlement: Option<Arc<NativeHelperSettlementPlan>>,
    closure: Option<Arc<NativeVersionClosurePlan>>,
    closed: Option<Arc<NativeHelperSettlementCommit>>,
    ended: bool,
}
/// Only actual capture constructs these fields. Output is never a receipt.
pub(crate) struct NativeVersionObservation {
    plan: Arc<NativeVersionHelperPlan>,
    stdout: Vec<u8>,
    bytes: usize,
    exit: Option<std::process::ExitStatus>,
    complete: bool,
    stdout_eof: bool,
    stderr_eof: bool,
    valid: bool,
    group_hygiene: bool,
    stop_method: StopMethod,
    attempted: bool,
    returned_child: bool,
    qualification: std::sync::OnceLock<bool>,
    schema: std::sync::OnceLock<schema::Gate>,
    handshake: bool,
    ipc: super::super::ipc::PreparationIpcObservation,
}
#[derive(Clone, Copy)]
enum StopMethod {
    Unknown,
    SelfExit,
    GraceSignal,
    ForcedSignal,
}
impl StopMethod {
    fn label(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::SelfExit => "self",
            Self::GraceSignal => "grace_signal",
            Self::ForcedSignal => "forced_signal",
        }
    }
}
impl NativeVersionObservation {
    pub(crate) fn qualified(&self) -> bool {
        *self.qualification.get_or_init(|| self.qualify_once())
    }
    fn qualify_once(&self) -> bool {
        if !self.complete || !self.valid || self.ipc.invalid {
            return false;
        }
        match self.plan.action() {
            readonly::NativePhaseHelperAction::Version => self.qualified_profile().is_some(),
            readonly::NativePhaseHelperAction::Schema => {
                self.exit.is_some_and(|e| e.success())
                    && self
                        .schema_gate()
                        .is_some_and(|g| g.receipt.pass && g.contract.is_some())
            }
            readonly::NativePhaseHelperAction::Handshake => {
                self.handshake && self.plan.contract().is_some()
            }
            readonly::NativePhaseHelperAction::Git(action) => {
                self.plan.git_seal().is_some_and(|seal| {
                    seal.qualify(*action, &self.stdout, self.exit.and_then(|e| e.code()))
                        .is_ok()
                        && readonly::require_conversion(*action, seal, self.plan.conversion())
                            .is_ok()
                })
            }
        }
    }
    pub(crate) fn settle_confirmed(&self) -> bool {
        self.complete
            && match self.plan.action() {
                // Preserve version's complete-capture factual bookkeeping;
                // its successful-exit/profile qualification stays separate.
                readonly::NativePhaseHelperAction::Version
                | readonly::NativePhaseHelperAction::Schema
                | readonly::NativePhaseHelperAction::Handshake => true,
                readonly::NativePhaseHelperAction::Git(a) => {
                    a.accepts_exit(self.exit.and_then(|e| e.code()))
                }
            }
    }
    pub(super) fn stdout(&self) -> &[u8] {
        &self.stdout
    }
    pub(super) fn qualify_git_for(
        &self,
        seal: &Arc<readonly::NativeGitSourceSeal>,
        action: readonly::PhaseGitAction,
    ) -> Result<()> {
        ensure!(
            matches!(self.plan.action(), readonly::NativePhaseHelperAction::Git(a) if *a == action)
                && self.plan.git_seal().is_some_and(|s| Arc::ptr_eq(s, seal))
                && self.qualified(),
            "actual Git observation action/seal differs"
        );
        Ok(())
    }
    pub(crate) fn matches_plan(&self, plan: &Arc<NativeVersionHelperPlan>) -> bool {
        Arc::ptr_eq(&self.plan, plan)
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn qualified_profile(&self) -> Option<&'static str> {
        if !matches!(
            self.plan.action(),
            readonly::NativePhaseHelperAction::Version
        ) || !self.complete
            || !self.exit.is_some_and(|exit| exit.success())
        {
            return None;
        }
        let text = std::str::from_utf8(&self.stdout).ok()?;
        match self.plan.actor().launch().allocation().facts().provider {
            "codex" if qualified_codex_phase_version(text) => Some("codex-cli-0.160.0"),
            "claude" if claude_wire::verify_version(text).is_ok() => Some("claude-2.1.294"),
            _ => None,
        }
    }
    pub(super) fn version_text(&self) -> Result<&str> {
        ensure!(
            self.qualified_profile().is_some(),
            "SAME version observation unqualified"
        );
        Ok(std::str::from_utf8(&self.stdout)?)
    }
    fn schema_gate(&self) -> Option<&schema::Gate> {
        self.plan.directory().map(|directory| {
            self.schema.get_or_init(|| {
                directory.gate().unwrap_or_else(|e| schema::Gate {
                    receipt: schema::file_refused(&e),
                    contract: None,
                })
            })
        })
    }
    pub(crate) fn schema_contract(&self) -> Result<super::super::codex_wire::CodexWireContract> {
        ensure!(self.qualified(), "fresh Schema unqualified");
        self.schema_gate()
            .and_then(|g| g.contract.clone())
            .context("fresh Schema additions absent")
    }
    pub(crate) fn safe_receipt(&self) -> BTreeMap<String, String> {
        let mut receipt = BTreeMap::from([
            ("action".into(), self.plan.action().label().into()),
            ("stdout_sha256".into(), native_result::digest(&self.stdout)),
            (
                "expectation".into(),
                if self.qualified() {
                    "match"
                } else if self.complete {
                    "mismatch"
                } else {
                    "unparsed"
                }
                .into(),
            ),
            (
                "creation".into(),
                if self.returned_child {
                    "returned_child"
                } else if self.attempted {
                    "attempt_without_returned_handle"
                } else {
                    "not_attempted"
                }
                .into(),
            ),
            (
                "capture".into(),
                format!(
                    "stdout_eof={};stderr_eof={};leader_reaped={};stop={}",
                    self.stdout_eof,
                    self.stderr_eof,
                    self.exit.is_some(),
                    self.stop_method.label()
                ),
            ),
            (
                "exit".into(),
                self.exit.map_or_else(
                    || "unknown".into(),
                    |status| {
                        status.code().map_or_else(
                            || {
                                status
                                    .signal()
                                    .map_or_else(|| "unknown".into(), |n| format!("signal:{n}"))
                            },
                            |n| n.to_string(),
                        )
                    },
                ),
            ),
            ("bytes".into(), self.bytes.to_string()),
            ("stdout_sha256".into(), native_result::digest(&self.stdout)),
            ("action".into(), self.plan.action().label().into()),
            (
                "profile".into(),
                self.qualified_profile()
                    .unwrap_or("unsupported_or_unknown")
                    .into(),
            ),
            (
                "hygiene".into(),
                if self.group_hygiene {
                    "group_signal_attempted"
                } else {
                    "unknown"
                }
                .into(),
            ),
        ]);
        receipt.insert(
            "ipc_marker_refused".into(),
            self.ipc
                .refusals
                .iter()
                .filter(|v| {
                    **v == super::super::ipc::PreparationIpcRefusal::CookieAcceptedMarkerRefused
                })
                .count()
                .to_string(),
        );
        receipt.insert(
            "ipc_other_refused".into(),
            self.ipc
                .refusals
                .iter()
                .filter(|v| **v == super::super::ipc::PreparationIpcRefusal::OtherRefused)
                .count()
                .to_string(),
        );
        receipt.insert("ipc_invalid".into(), self.ipc.invalid.to_string());
        if let Some(gate) = self.schema_gate() {
            receipt.insert(
                "schema_result".into(),
                gate.receipt.reason.map_or_else(
                    || "pass".into(),
                    |r| serde_json::to_value(r).unwrap().as_str().unwrap().into(),
                ),
            );
            if let Some(position) = gate.receipt.positions.first() {
                receipt.insert("schema_position".into(), position.clone());
            }
            receipt.insert(
                "schema_counts".into(),
                format!(
                    "{}:{}:{}",
                    gate.receipt.tolerated_changes,
                    gate.receipt.accepted_item_types,
                    gate.receipt.accepted_notifications
                ),
            );
            receipt.insert(
                "schema_additions_sha256".into(),
                gate.receipt.additions_sha256.clone(),
            );
        }
        receipt
    }
}
pub(super) fn qualified_codex_phase_version(text: &str) -> bool {
    matches!(
        text,
        "codex-cli 0.160.0" | "codex-cli 0.160.0\n" | "codex-cli 0.160.0\r\n"
    )
}
impl NativeVersionHelperCustody {
    pub(super) fn observation(&self) -> Result<Arc<NativeVersionObservation>> {
        self.state()
            .observation
            .clone()
            .context("actual helper observation absent")
    }

    pub(super) fn captured_bytes(&self) -> Result<usize> {
        self.state()
            .observation
            .as_ref()
            .map(|o| o.bytes)
            .context("authentic helper observation remains held")
    }
    pub(super) fn closed(&self) -> Result<Arc<NativeHelperSettlementCommit>> {
        let (observation, closed) = {
            let state = self.state();
            (state.observation.clone(), state.closed.clone())
        };
        ensure!(
            observation.as_ref().is_some_and(|o| o.qualified()),
            "original helper qualification failed or remains unknown"
        );
        let closed = closed.context("known original helper settlement remains held")?;
        ensure!(
            closed.matches_plan(&self.plan),
            "closed original helper plan differs"
        );
        Ok(closed)
    }
    pub(super) fn matches_actor(&self, actor: &Arc<NativePreparationActor>) -> bool {
        Arc::ptr_eq(self.plan.actor(), actor)
    }
    pub(super) fn abandon(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
    fn raw(&self) -> std::sync::MutexGuard<'_, process::RetainedRawProcess> {
        // Poison recovery retains actual child ownership, never effect grant.
        self.raw
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn state(&self) -> std::sync::MutexGuard<'_, HelperState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    pub(super) fn reconcile(&self, owner: &Arc<RuntimeOwner>) -> Result<()> {
        if self.state().closed.is_some() {
            // SAME actual committed acknowledgement is durable history, not
            // current permission. Later Unit/inventory evolution cannot erase
            // it or require replaying an already-known nongrant settlement.
            return Ok(());
        }
        let (observation, settlement, closure) = {
            let state = self.state();
            (
                state.observation.clone(),
                state.settlement.clone(),
                state.closure.clone(),
            )
        };
        let settlement = match settlement {
            Some(settlement) => settlement,
            None => {
                let observation =
                    observation.context("same owned version observation pending; held")?;
                let planned = crate::state::Store::plan_phase_version_settlement(
                    self.plan.clone(),
                    observation,
                )?;
                self.state().settlement.get_or_insert(planned).clone()
            }
        };
        let closure = match closure {
            Some(closure) => closure,
            None => {
                // Receipt encoding and latest full Unit snapshot precede the
                // writer mutex. Retain once; CAS drift never refreshes history.
                let planned = crate::state::Store::plan_phase_version_closure(owner, settlement)?;
                self.state().closure.get_or_insert(planned).clone()
            }
        };
        let closed = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .close_phase_version_observation(&closure)?;
        self.state().closed.get_or_insert(Arc::new(closed));
        Ok(())
    }
}
/// Constructed before task spawn/first poll. A task dropped without polling
/// still revokes and attempts hygiene on the independently retained raw cell.
struct CaptureOwner {
    custody: Arc<NativeVersionHelperCustody>,
}
impl Drop for CaptureOwner {
    fn drop(&mut self) {
        self.custody.abandon();
        self.custody.ipc.finish();
        let _ = self.custody.raw().hygiene();
        let mut state = self.custody.state();
        state.ended = true;
        drop(state);
        self.custody.done.notify_one();
    }
}
/// Cancellation before/after creation releases the independently eager task.
struct CaptureKick {
    sender: Option<tokio::sync::oneshot::Sender<()>>,
    custody: Arc<NativeVersionHelperCustody>,
    completed: bool,
}
impl CaptureKick {
    fn start(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(());
        }
    }
}
impl Drop for CaptureKick {
    fn drop(&mut self) {
        if !self.completed {
            self.custody.abandon();
        }
        self.start();
    }
}

pub(super) fn physical_command(
    owner: &Arc<RuntimeOwner>,
    actor: &Arc<NativePreparationActor>,
    action: &readonly::NativePhaseHelperAction,
    seal: Option<&Arc<readonly::NativeGitSourceSeal>>,
    lease: Option<&Arc<owner::GitLease>>,
) -> Result<Command> {
    let profile = qualified_physical_profile(owner, actor)?;
    let facts = actor.launch().allocation().facts();
    let unit = actor.launch().allocation().unit_snapshot();
    let overlay = profile.environment(&unit.cookie, &owner.socket)?;
    build_helper_command(actor, action, seal, lease, &profile, overlay, facts)
}

pub(super) fn qualified_physical_profile(
    owner: &Arc<RuntimeOwner>,
    actor: &Arc<NativePreparationActor>,
) -> Result<resources::ResourceProfile> {
    actor.validate_open()?;
    let allocation = actor.launch().allocation();
    let unit = allocation.unit_snapshot();
    let facts = allocation.facts();
    super::metadata::validate_unit_temp(owner, unit.id)?;
    ensure!(
        facts.program.is_absolute()
            && facts.program.canonicalize()? == facts.program
            && facts.program.is_file()
            && facts.path.is_absolute()
            && facts.path.canonicalize()? == facts.path,
        "original Native program/worktree physical profile differs"
    );
    let root = owner.root.join("units").join(unit.id.to_string());
    let profile_path = root.join("profile.json");
    ensure!(
        root.canonicalize()? == root && profile_path.canonicalize()? == profile_path,
        "original profile path aliases another namespace"
    );
    let mut bytes = Vec::new();
    std::fs::File::open(profile_path)?
        .take(PROFILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= PROFILE_BYTES,
        "bounded physical resource profile exceeded"
    );
    let value = strict_json::decode(
        &bytes,
        strict_json::Limits {
            frame_bytes: PROFILE_BYTES,
            depth: 8,
            nodes: 1024,
            string_bytes: 4096,
            total_string_bytes: PROFILE_BYTES,
            object_entries: 128,
            array_entries: 32,
        },
    )?;
    let profile: resources::ResourceProfile = serde_json::from_value(value)?;
    ensure!(
        serde_json::to_vec(&profile)?.len() <= PROFILE_BYTES,
        "encoded profile exceeds bound"
    );
    profile.validate(owner, unit)?;
    ensure!(
        profile.temp == root.join("tmp")
            && profile.output == root.join("output")
            && profile.cache == root.join("cache")
            && profile.tool_bin == root.join("tool-bin"),
        "original resource child namespace differs"
    );
    Ok(profile)
}

fn build_helper_command(
    actor: &Arc<NativePreparationActor>,
    action: &readonly::NativePhaseHelperAction,
    seal: Option<&Arc<readonly::NativeGitSourceSeal>>,
    lease: Option<&Arc<owner::GitLease>>,
    profile: &resources::ResourceProfile,
    overlay: BTreeMap<String, String>,
    facts: crate::execution::phase::AllocationFacts<'_>,
) -> Result<Command> {
    let mut command = if !matches!(action, readonly::NativePhaseHelperAction::Git(_)) {
        ensure!(lease.is_none(), "version cannot consume Git lease");
        let mut command = Command::new(facts.program);
        match action {
            readonly::NativePhaseHelperAction::Version => {
                command.arg("--version");
            }
            readonly::NativePhaseHelperAction::Schema => {
                command.args(["app-server", "generate-json-schema", "--experimental"]);
            }
            readonly::NativePhaseHelperAction::Handshake => {
                command.args(prepared::command_argv(
                    facts.provider,
                    facts.role,
                    facts.model,
                    facts.effort,
                    None,
                )?);
            }
            readonly::NativePhaseHelperAction::Git(_) => unreachable!("Git handled separately"),
        }
        command
    } else {
        let lease = lease.context("original readonly Git lease absent")?;
        let program = profile
            .real_tools
            .get("git")
            .context("original real Git program absent")?;
        ensure!(
            program.is_absolute() && program.canonicalize()? == *program && program.is_file(),
            "original real Git physical profile differs"
        );
        let readonly::NativePhaseHelperAction::Git(git) = action else {
            unreachable!("version handled")
        };
        let seal = seal.context("original Git seal absent")?;
        ensure!(
            seal.matches_actor(actor),
            "Git command original seal differs"
        );
        let mut command = super::super::results::git_command_for(git.cwd(actor), program)?;
        command
            .args(git.argv(seal.revision())?)
            .env_clear()
            .envs(crate::git::native_environment())
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_NO_LAZY_FETCH", "1")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .env("RRX_GIT_GATE_TOKEN", lease.id.to_string());
        command
    };
    command
        .current_dir(action.cwd(actor))
        .envs(overlay)
        .stdin(
            if matches!(action, readonly::NativePhaseHelperAction::Handshake) {
                Stdio::piped()
            } else {
                Stdio::null()
            },
        )
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .kill_on_drop(true);
    Ok(command)
}

impl NativeSessions {
    pub(super) async fn prepare_phase_version(
        &self,
        original: Arc<NativePreparationCustody>,
    ) -> Result<Arc<NativeVersionHelperCustody>> {
        let ready = original.version_original()?;
        // Full profile/path/environment qualification and encoding are outside
        // Store, custody, source, queue and effect-admission locks.
        let actor = original.state_actor()?;
        let action = readonly::NativePhaseHelperAction::Version;
        let command = physical_command(&self.owner, &actor, &action, None, None)?;
        let plan = crate::state::Store::plan_phase_version_intent(&self.owner, ready)?;
        self.run_phase_helper(original, actor, plan, command, None)
            .await
    }
    pub(super) async fn prepare_phase_git(
        &self,
        original: Arc<NativePreparationCustody>,
        mut previous: Arc<NativeVersionHelperCustody>,
    ) -> Result<Arc<NativeReadonlyHelperCompletion>> {
        use readonly::{NativePhaseHelperAction, PhaseGitAction};
        let actor = original.state_actor()?;
        // A qualified actual version closure is mandatory before even seal or
        // lease setup, never inferred from SQL or a supplied version string.
        previous.closed()?;
        let seal = readonly::NativeGitSourceSeal::new(&actor)?;
        let lease = self
            .owner
            .git_lease(actor.launch().allocation().facts().unit_id, None)
            .await?;
        let mut observations = Vec::with_capacity(readonly::GIT_ACTIONS);
        let mut conversion = None;
        for action in PhaseGitAction::ORDERED {
            seal.validate_deadline()?;
            let known = previous.closed()?;
            let command = physical_command(
                &self.owner,
                &actor,
                &NativePhaseHelperAction::Git(action),
                Some(&seal),
                Some(&lease),
            )?;
            let plan = crate::state::Store::plan_phase_git_intent(
                &self.owner,
                known,
                action,
                seal.clone(),
                conversion.clone(),
            )?;
            previous = self
                .run_phase_helper(
                    original.clone(),
                    actor.clone(),
                    plan,
                    command,
                    Some(lease.clone()),
                )
                .await?;
            previous.closed()?;
            observations.push(previous.observation()?);
            if action == PhaseGitAction::Head {
                readonly::qualify_ownership(&seal, &observations)?;
            }
            if action == PhaseGitAction::ConversionAttrs {
                conversion = Some(readonly::ConversionIdentity::qualify(
                    &seal,
                    observations[8].clone(),
                    observations[11].clone(),
                )?);
            }
        }
        let correspondence = readonly::qualify_correspondence(
            seal,
            conversion.context("same batch conversion absent")?,
            observations,
        )?;
        let history = original.qualified_history()?;
        let launch = actor.launch().clone();
        let admission = launch.admission().enter(launch.clone()).await?;
        let commit = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .confirm_phase_helper_history(history, &admission)?;
        let completion = Arc::new(NativeReadonlyHelperCompletion {
            actor,
            _correspondence: correspondence,
            commit,
            contract: original.metadata_contract()?,
        });
        original.retain_completion(completion.clone())?;
        drop(admission);
        Ok(completion)
    }
    pub(super) async fn run_phase_helper(
        &self,
        original: Arc<NativePreparationCustody>,
        actor: Arc<NativePreparationActor>,
        plan: Arc<NativeVersionHelperPlan>,
        command: Command,
        lease: Option<Arc<owner::GitLease>>,
    ) -> Result<Arc<NativeVersionHelperCustody>> {
        ensure!(
            Arc::ptr_eq(plan.actor(), &actor),
            "version original actor differs"
        );
        let identity = actor.program.get().context("fresh P identity absent")?;
        if !matches!(plan.action(), readonly::NativePhaseHelperAction::Git(_)) {
            identity
                .verify(
                    actor.identity_cancellation(),
                    tokio::time::Instant::now() + Duration::from_secs(30),
                )
                .await
                .map_err(|_| anyhow::anyhow!("P helper executable changed"))?;
        }
        actor.validate_open()?;
        let helper = Arc::new(NativeVersionHelperCustody {
            plan: plan.clone(),
            ipc: super::super::ipc::PreparationIpcSink::new(
                &self.owner,
                actor.launch().allocation().unit_snapshot(),
            ),
            raw: Mutex::new(process::RetainedRawProcess::default()),
            state: Mutex::new(HelperState::default()),
            cancelled: AtomicBool::new(false),
            done: tokio::sync::Notify::new(),
            output_limit: original.next_output_limit(plan.action())?,
            git_lease: Mutex::new(lease),
        });
        original.retain_helper(helper.clone())?;
        self.owner.register_preparation_ipc(&helper.ipc)?;
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let mut kick = CaptureKick {
            sender: Some(sender),
            custody: helper.clone(),
            completed: false,
        };
        let capture = CaptureOwner {
            custody: helper.clone(),
        };
        let owner = Arc::downgrade(&self.owner);
        tokio::spawn(async move {
            let _ = receiver.await;
            capture_version(capture, owner).await;
        });
        let launch = actor.launch().clone();
        let admission = launch.admission().enter(launch.clone()).await?;
        let intent = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            match plan.action() {
                readonly::NativePhaseHelperAction::Version => {
                    store.reserve_phase_version_intent(plan.clone(), &admission)?
                }
                readonly::NativePhaseHelperAction::Schema
                | readonly::NativePhaseHelperAction::Handshake => {
                    store.reserve_phase_metadata_intent(plan.clone(), &admission)?
                }
                readonly::NativePhaseHelperAction::Git(_) => {
                    store.reserve_phase_git_intent(plan.clone(), &admission)?
                }
            }
        };
        helper.state().intent = Some(Arc::new(intent));
        if let Some(seal) = plan.git_seal() {
            seal.start_batch();
            seal.validate_deadline()?;
        }
        actor.validate_open()?;
        admission.validate_for(&launch)?;
        // One-shot state records attempted creation even when spawn gives no
        // handle. The command is fixed and no API can retry this producer.
        if !matches!(plan.action(), readonly::NativePhaseHelperAction::Git(_)) {
            identity.stat()?;
        }
        if let Some(dir) = plan.directory() {
            dir.verify(&actor)?;
        }
        {
            let mut state = helper.state();
            state.attempted = true;
            state.started = Some(tokio::time::Instant::now());
        }
        let spawned = rrx_spawn_hygiene::spawn_tokio(command);
        if let Ok(child) = spawned {
            // FIRST action after spawn success is the infallible custody move.
            helper.raw().adopt(child);
            helper.ipc.start(helper.raw().group_id());
        }
        drop(admission);
        kick.start();
        loop {
            let notified = helper.done.notified();
            if helper.state().ended {
                break;
            }
            notified.await;
        }
        kick.completed = true;
        // Both initial completion and later reconciliation use SAME authentic
        // observation; cancellation cannot force normal permission reopening.
        helper.reconcile(&self.owner)?;
        Ok(helper)
    }
}

async fn capture_version(guard: CaptureOwner, owner: Weak<RuntimeOwner>) {
    let helper = &guard.custody;
    let mut output = BoundedOutput {
        limit: helper.output_limit,
        ..Default::default()
    };
    let mut handshake = helper
        .plan
        .contract()
        .cloned()
        .map(super::super::codex_wire::Handshake::new);
    let mut valid = true;
    let mut pending = Vec::new();
    let mut exit = None;
    let mut stdout_eof = false;
    let mut stderr_eof = false;
    let mut signalled_after_grace = false;
    let mut stop_method = StopMethod::Unknown;
    let returned_child = helper.raw().has_child();
    let (attempted, started) = {
        let state = helper.state();
        (state.attempted, state.started)
    };
    let action_deadline = started.unwrap_or_else(tokio::time::Instant::now)
        + Duration::from_secs(match helper.plan.action() {
            readonly::NativePhaseHelperAction::Schema => 60,
            readonly::NativePhaseHelperAction::Handshake => 90,
            _ => 30,
        });
    let deadline = helper
        .plan
        .git_seal()
        .and_then(|s| s.deadline())
        .map_or(action_deadline, |d| d.min(action_deadline));
    let qualified = helper.raw().qualify().is_ok();
    let pipes = if qualified {
        helper.raw().pipes().ok()
    } else {
        None
    };
    let intent = helper.state().intent.clone();
    if let (Some(pipes), Some(intent)) = (pipes, intent) {
        let mut response_deadline = handshake
            .as_ref()
            .map(|_| deadline.min(tokio::time::Instant::now() + Duration::from_secs(30)));
        if let Some(protocol) = &handshake {
            valid = send_handshake(
                &pipes.stdin,
                protocol.initial().ok(),
                response_deadline.unwrap(),
                helper.plan.actor(),
            )
            .await;
        } else {
            pipes.stdin.lock().await.take();
        }
        let (mut out, mut err) = (pipes.stdout, pipes.stderr);
        let mut out_open = true;
        let mut err_open = true;
        let mut out_buffer = [0u8; 4096];
        let mut err_buffer = [0u8; 4096];
        let mut grace = None;
        let mut drain = None;
        let mut finishing = false;
        let mut stop_requested = !valid;
        let mut protocol_refused = false;
        let mut exited = false;
        let mut fence = tokio::time::interval(Duration::from_millis(50));
        fence.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            if exited && !out_open && !err_open {
                break;
            }
            let now = tokio::time::Instant::now();
            if helper.cancelled.load(Ordering::Acquire) {
                valid = false;
                stop_requested = true;
            }
            if response_deadline.is_some_and(|d| now >= d) {
                valid = false;
                stop_requested = true;
                response_deadline = None;
            }
            if now >= deadline {
                valid = false;
                stop_method = StopMethod::ForcedSignal;
                break;
            }
            if stop_requested && drain.is_none() {
                valid = false;
                finishing = true;
                response_deadline = None;
                grace = None;
                helper.raw().hygiene();
                stop_method = StopMethod::ForcedSignal;
                drain = Some((now + Duration::from_secs(2)).min(deadline));
            }
            if !finishing && (protocol_refused || handshake.as_ref().is_some_and(|h| h.complete()))
            {
                finishing = true;
                response_deadline = None;
                pipes.stdin.lock().await.take();
                if !exited {
                    grace = Some((now + Duration::from_secs(5)).min(deadline));
                }
            }
            if grace.is_some_and(|d| now >= d) {
                helper.raw().hygiene();
                signalled_after_grace = true;
                stop_method = StopMethod::GraceSignal;
                grace = None;
                drain.get_or_insert((now + Duration::from_secs(2)).min(deadline));
            }
            if drain.is_some_and(|d| now >= d) {
                break;
            }
            let end = [Some(deadline), response_deadline, grace, drain]
                .into_iter()
                .flatten()
                .min()
                .unwrap();
            let read_len = output.read_window();
            tokio::select! {
                read=out.read(&mut out_buffer[..read_len]), if out_open => match read {
                    Ok(0) => { out_open=false; stdout_eof=true; },
                    Ok(n) => {
                        if !output.observe(&out_buffer[..n],true) { valid=false; stop_requested=true; }
                        if valid && let Some(protocol)=&mut handshake {
                            pending.extend_from_slice(&out_buffer[..n]);
                            while let Some(newline)=pending.iter().position(|b| *b==b'\n') {
                                let line: Vec<_>=pending.drain(..=newline).collect();
                                match protocol.observe(&line[..line.len()-1]) {
                                    Ok(outgoing) => {
                                        let progressed=!outgoing.is_empty();
                                        for value in outgoing {
                                            if !send_handshake(&pipes.stdin,Some(value),end,helper.plan.actor()).await { valid=false; stop_requested=true; break; }
                                        }
                                        if !valid { break; }
                                        if protocol.complete() { response_deadline=None; }
                                        else if progressed { response_deadline=Some((tokio::time::Instant::now()+Duration::from_secs(30)).min(deadline)); }
                                    },
                                    Err(_) => { valid=false; protocol_refused=true; break; },
                                }
                            }
                        }
                    },
                    Err(_) => { valid=false;stop_requested=true;out_open=false; },
                },
                read=err.read(&mut err_buffer[..read_len]), if err_open => match read {
                    Ok(0) => { err_open=false;stderr_eof=true; },
                    Ok(n) => { if !output.observe(&err_buffer[..n],false) { valid=false;stop_requested=true; } },
                    Err(_) => { valid=false;stop_requested=true;err_open=false; },
                },
                _=tokio::time::sleep_until(end)=>{},
                _=fence.tick()=>{
                    if !stop_requested {
                        let current=owner.upgrade().is_some_and(|owner| owner.store.lock().ok().is_some_and(|mut store| store.validate_phase_version_fence(&helper.plan,&intent).is_ok()));
                        if !current { valid=false;stop_requested=true; }
                    }
                    if !exited {
                        let observed={helper.raw().exited_unreaped()};
                        match observed {
                            Ok(true)=>{
                                exited=true;
                                if matches!(stop_method, StopMethod::Unknown) { stop_method=StopMethod::SelfExit; }
                                helper.raw().hygiene();
                                grace=None;
                                drain.get_or_insert((tokio::time::Instant::now()+Duration::from_secs(2)).min(deadline));
                            },
                            Ok(false)=>{},
                            Err(_)=>{ valid=false;stop_method=StopMethod::ForcedSignal;break; },
                        }
                    }
                },
            }
        }
    }
    if returned_child && matches!(stop_method, StopMethod::Unknown) {
        stop_method = StopMethod::ForcedSignal;
    }
    let group_hygiene = helper.raw().hygiene();
    let stop_deadline = (tokio::time::Instant::now() + Duration::from_secs(10)).min(deadline);
    loop {
        if !helper.raw().has_child() {
            break;
        }
        match helper.raw().reap() {
            Ok(Some(status)) => {
                exit = Some(status);
                break;
            }
            Err(_) => break,
            Ok(None) => {}
        }
        if tokio::time::Instant::now() >= stop_deadline {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let complete = stdout_eof && stderr_eof && exit.is_some();
    let observation = Arc::new(NativeVersionObservation {
        plan: helper.plan.clone(),
        stdout: output.stdout,
        bytes: output.bytes,
        exit,
        complete,
        stdout_eof,
        stderr_eof,
        valid,
        group_hygiene,
        stop_method,
        attempted,
        returned_child,
        qualification: std::sync::OnceLock::new(),
        schema: std::sync::OnceLock::new(),
        handshake: valid
            && pending.is_empty()
            && handshake.as_ref().is_some_and(|h| h.complete())
            && exit.is_some_and(|s| s.success() || (signalled_after_grace && s.code().is_none())),
        ipc: helper.ipc.finish(),
    });
    // Retain the physical original before optional Schema parsing, hashing or SQL.
    helper.state().observation = Some(observation);
    release_git_serialization(&helper.raw(), &helper.git_lease);
    drop(guard);
}

async fn send_handshake(
    stdin: &Arc<tokio::sync::Mutex<Option<ChildStdin>>>,
    value: Option<Value>,
    deadline: tokio::time::Instant,
    actor: &NativePreparationActor,
) -> bool {
    let Some(value) = value else {
        return false;
    };
    let Ok(mut bytes) = serde_json::to_vec(&value) else {
        return false;
    };
    bytes.push(b'\n');
    tokio::time::timeout_at(deadline, async {
        let mut guard = stdin.lock().await;
        actor.validate_open()?;
        let pipe = guard.as_mut().context("P handshake stdin closed")?;
        pipe.write_all(&bytes).await?;
        pipe.flush().await?;
        Ok::<_, anyhow::Error>(())
    })
    .await
    .is_ok_and(|r| r.is_ok())
}

/// Serialization only: drops THIS helper's lease clone after a known leader
/// reap, or when the completed capture never returned a child. Raw Child,
/// observation and custody stay retained; this infers no descendant death,
/// NoChild, success or grant. An unreaped/unknown leader keeps the gate.
fn release_git_serialization(
    raw: &process::RetainedRawProcess,
    lease: &Mutex<Option<Arc<owner::GitLease>>>,
) {
    if raw.leader_reaped() || !raw.has_child() {
        lease
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
}

#[cfg(test)]
mod output_tests {
    use super::*;

    #[test]
    fn nongrant_output_combines_both_streams_with_one_overflow_sentinel() {
        let mut output = BoundedOutput {
            limit: OUTPUT_BYTES,
            ..Default::default()
        };
        assert!(output.observe(&vec![b'o'; OUTPUT_BYTES / 2], true));
        assert!(output.observe(&vec![b'e'; OUTPUT_BYTES / 2], false));
        assert_eq!(output.bytes, OUTPUT_BYTES);
        assert_eq!(output.stdout.len(), OUTPUT_BYTES / 2);
        assert_eq!(output.read_window(), 1);
        assert!(!output.observe(b"e", false));
        assert_eq!(output.bytes, OUTPUT_BYTES + 1);
        assert_eq!(output.stdout.len(), OUTPUT_BYTES / 2);
        // Discarded stderr never replenishes combined quota.
        assert!(!output.observe(b"o", true));
        let mut out_only = BoundedOutput {
            limit: OUTPUT_BYTES,
            ..Default::default()
        };
        assert!(out_only.observe(&vec![b'o'; OUTPUT_BYTES], true));
        assert!(!out_only.observe(b"o", true));
        assert_eq!(out_only.stdout.len(), OUTPUT_BYTES);
    }
}

#[cfg(test)]
mod git_serialization_tests {
    // Nongrant raw/gate primitives only: real spawned leaders and the actual
    // Runtime Git gate. Not Native actor, Task or helper qualification proof.
    use super::*;

    type LeaseCell = Mutex<Option<Arc<owner::GitLease>>>;

    fn retained(script: &str) -> process::RetainedRawProcess {
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .kill_on_drop(true);
        let mut raw = process::RetainedRawProcess::default();
        raw.adopt(rrx_spawn_hygiene::spawn_tokio(command).unwrap());
        raw.qualify().unwrap();
        raw
    }
    async fn known_reap(raw: &mut process::RetainedRawProcess) -> std::process::ExitStatus {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                if let Some(status) = raw.reap().unwrap() {
                    return status;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap()
    }
    async fn held_lease(dir: &tempfile::TempDir) -> (Arc<RuntimeOwner>, LeaseCell) {
        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        let lease = owner.git_lease(UnitId::new(), None).await.unwrap();
        (owner, Mutex::new(Some(lease)))
    }
    /// A different Unit obtains (then drops) the SAME Runtime common gate.
    async fn other_unit_admitted(owner: &RuntimeOwner) -> bool {
        let admission = owner.git_lease(UnitId::new(), None);
        tokio::time::timeout(Duration::from_millis(300), admission)
            .await
            .is_ok_and(|lease| lease.is_ok())
    }
    fn held(cell: &LeaseCell) -> bool {
        cell.lock().unwrap().is_some()
    }

    #[tokio::test]
    async fn nongrant_known_reap_releases_only_serialization_for_any_exit() {
        // Success and a mismatch-shaped nonzero exit release alike.
        for (script, success) in [("exit 0", true), ("echo mismatch; exit 3", false)] {
            let dir = tempfile::tempdir().unwrap();
            let (owner, lease) = held_lease(&dir).await;
            let mut raw = retained(script);
            assert_eq!(known_reap(&mut raw).await.success(), success);
            assert!(!other_unit_admitted(&owner).await);
            release_git_serialization(&raw, &lease);
            assert!(!held(&lease));
            assert!(other_unit_admitted(&owner).await);
            // Original raw Child custody remains; reap is not workload absence.
            assert!(raw.has_child() && raw.leader_reaped());
        }
    }
    #[tokio::test]
    async fn nongrant_stopped_and_reaped_leader_releases_serialization() {
        let dir = tempfile::tempdir().unwrap();
        let (owner, lease) = held_lease(&dir).await;
        let mut raw = retained("exec /bin/sleep 30");
        assert!(raw.hygiene());
        assert!(!known_reap(&mut raw).await.success());
        release_git_serialization(&raw, &lease);
        assert!(!held(&lease));
        assert!(other_unit_admitted(&owner).await);
        assert!(raw.has_child());
    }
    #[tokio::test]
    async fn nongrant_live_unreaped_leader_retains_gate_until_stop_and_reap() {
        let dir = tempfile::tempdir().unwrap();
        let (owner, lease) = held_lease(&dir).await;
        let mut raw = retained("exec /bin/sleep 30");
        assert!(raw.reap().unwrap().is_none());
        release_git_serialization(&raw, &lease);
        assert!(held(&lease));
        assert!(!other_unit_admitted(&owner).await);
        assert!(raw.hygiene());
        assert!(!known_reap(&mut raw).await.success());
        release_git_serialization(&raw, &lease);
        assert!(!held(&lease));
        assert!(other_unit_admitted(&owner).await);
    }
    #[tokio::test]
    async fn nongrant_exited_unreaped_leader_retains_gate() {
        let dir = tempfile::tempdir().unwrap();
        let (owner, lease) = held_lease(&dir).await;
        let mut raw = retained("exit 0");
        tokio::time::timeout(Duration::from_secs(10), async {
            while !raw.exited_unreaped().unwrap() {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        // Observed exit is not a reap; serialization stays conservative.
        release_git_serialization(&raw, &lease);
        assert!(held(&lease));
        assert!(!other_unit_admitted(&owner).await);
        assert!(known_reap(&mut raw).await.success());
        release_git_serialization(&raw, &lease);
        assert!(!held(&lease));
        assert!(other_unit_admitted(&owner).await);
    }
    #[tokio::test]
    async fn nongrant_capture_without_returned_child_releases_serialization() {
        let dir = tempfile::tempdir().unwrap();
        let (owner, lease) = held_lease(&dir).await;
        release_git_serialization(&process::RetainedRawProcess::default(), &lease);
        assert!(!held(&lease));
        assert!(other_unit_admitted(&owner).await);
    }
}
