//! One retained version-only helper. Neither a receipt nor exit zero completes
//! protected Native preparation, registration, input or Session authority.
use super::*;
use crate::state::{NativeHelperIntentCommit, NativeHelperSettlementPlan, NativeVersionHelperPlan};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::AsyncReadExt;

const OUTPUT_BYTES: usize = 64 * 1024;
const PROFILE_BYTES: usize = 64 * 1024;

pub(super) struct NativeVersionHelperCustody {
    plan: Arc<NativeVersionHelperPlan>,
    raw: Mutex<process::RetainedRawProcess>,
    state: Mutex<HelperState>,
    cancelled: AtomicBool,
    done: tokio::sync::Notify,
}
#[derive(Default)]
struct HelperState {
    intent: Option<Arc<NativeHelperIntentCommit>>,
    attempted: bool,
    observation: Option<Arc<NativeVersionObservation>>,
    settlement: Option<Arc<NativeHelperSettlementPlan>>,
    ended: bool,
}
/// Only actual capture constructs these fields. Output is never a receipt.
pub(crate) struct NativeVersionObservation {
    plan: Arc<NativeVersionHelperPlan>,
    stdout: Vec<u8>,
    bytes: usize,
    exit: Option<std::process::ExitStatus>,
    complete: bool,
    group_hygiene: bool,
}
impl NativeVersionObservation {
    pub(crate) fn matches_plan(&self, plan: &Arc<NativeVersionHelperPlan>) -> bool {
        Arc::ptr_eq(&self.plan, plan)
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    fn qualified_profile(&self) -> Option<&'static str> {
        if !self.complete || !self.exit.is_some_and(|exit| exit.success()) {
            return None;
        }
        let text = std::str::from_utf8(&self.stdout).ok()?;
        match self.plan.actor().launch().allocation().facts().provider {
            "codex" if crate::codex::managed::version(text).is_ok() => Some("codex-cli-0.160.0"),
            "claude" if claude_wire::verify_version(text).is_ok() => Some("claude-2.1.283"),
            _ => None,
        }
    }
    pub(crate) fn safe_receipt(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                "capture".into(),
                if self.complete { "complete" } else { "unknown" }.into(),
            ),
            (
                "exit".into(),
                self.exit
                    .and_then(|status| status.code())
                    .map_or_else(|| "unknown_or_signal".into(), |n| n.to_string()),
            ),
            ("bytes".into(), self.bytes.to_string()),
            ("stdout_sha256".into(), native_result::digest(&self.stdout)),
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
        ])
    }
}
impl NativeVersionHelperCustody {
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
}
/// Constructed before task spawn/first poll. A task dropped without polling
/// still revokes and attempts hygiene on the independently retained raw cell.
struct CaptureOwner {
    custody: Arc<NativeVersionHelperCustody>,
}
impl Drop for CaptureOwner {
    fn drop(&mut self) {
        self.custody.abandon();
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

fn physical_command(
    owner: &Arc<RuntimeOwner>,
    actor: &Arc<NativePreparationActor>,
) -> Result<Command> {
    actor.validate_open()?;
    let allocation = actor.launch().allocation();
    let unit = allocation.unit_snapshot();
    let facts = allocation.facts();
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
    let overlay = profile.environment(&unit.cookie, &owner.socket)?;
    let mut command = Command::new(facts.program);
    command
        .arg("--version")
        .current_dir(facts.path)
        .envs(overlay)
        .stdin(Stdio::null())
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
    ) -> Result<()> {
        let ready = original.version_original()?;
        // Full profile/path/environment qualification and encoding are outside
        // Store, custody, source, queue and effect-admission locks.
        let actor = original.state_actor()?;
        let mut command = physical_command(&self.owner, &actor)?;
        let plan = crate::state::Store::plan_phase_version_intent(&self.owner, ready)?;
        ensure!(
            Arc::ptr_eq(plan.actor(), &actor),
            "version original actor differs"
        );
        let helper = Arc::new(NativeVersionHelperCustody {
            plan: plan.clone(),
            raw: Mutex::new(process::RetainedRawProcess::default()),
            state: Mutex::new(HelperState::default()),
            cancelled: AtomicBool::new(false),
            done: tokio::sync::Notify::new(),
        });
        original.retain_helper(helper.clone())?;
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
            store.reserve_phase_version_intent(plan.clone(), &admission)?
        };
        helper.state().intent = Some(Arc::new(intent));
        actor.validate_open()?;
        admission.validate_for(&launch)?;
        // One-shot state records attempted creation even when spawn gives no
        // handle. The command is fixed and no API can retry this producer.
        helper.state().attempted = true;
        let spawned = command.spawn();
        if let Ok(child) = spawned {
            // FIRST action after spawn success is the infallible custody move.
            helper.raw().adopt(child);
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
        let observation = helper
            .state()
            .observation
            .clone()
            .context("version helper actual capture unavailable; held")?;
        // Bounded owned observation was saved before any parser/hash/Store.
        let settlement = crate::state::Store::plan_phase_version_settlement(plan, observation)?;
        helper.state().settlement = Some(settlement.clone());
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .record_phase_version_observation(&settlement)?;
        Ok(())
    }
}

async fn capture_version(guard: CaptureOwner, owner: Weak<RuntimeOwner>) {
    let helper = &guard.custody;
    let mut stdout = Vec::new();
    let mut bytes = 0usize;
    let mut complete = false;
    let mut exit = None;
    let qualified = helper.raw().qualify().is_ok();
    let pipes = if qualified {
        helper.raw().pipes().ok()
    } else {
        None
    };
    let intent = helper.state().intent.clone();
    if let (Some((mut out, mut err)), Some(intent)) = (pipes, intent) {
        let mut out_open = true;
        let mut err_open = true;
        let mut out_buffer = [0u8; 4096];
        let mut err_buffer = [0u8; 4096];
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        let mut drain_deadline = None;
        let mut exited = false;
        // Admission has been released before this task is kicked.
        let mut fence = tokio::time::interval(Duration::from_millis(50));
        fence.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            if helper.cancelled.load(Ordering::Acquire) {
                break;
            }
            if exited && !out_open && !err_open {
                complete = true;
                break;
            }
            let end = drain_deadline.unwrap_or(deadline).min(deadline);
            let read_len = (OUTPUT_BYTES.saturating_sub(bytes) + 1).min(4096);
            tokio::select! {
                read = out.read(&mut out_buffer[..read_len]), if out_open => {
                    match read {
                        Ok(0) => out_open = false,
                        Ok(n) => {
                            bytes = bytes.saturating_add(n);
                            let available = OUTPUT_BYTES.saturating_sub(stdout.len());
                            stdout.extend_from_slice(&out_buffer[..n.min(available)]);
                            if bytes > OUTPUT_BYTES { break; }
                        }, Err(_) => break,
                    }
                },
                read = err.read(&mut err_buffer[..read_len]), if err_open => {
                    match read { Ok(0) => err_open = false, Ok(n) => {
                        bytes = bytes.saturating_add(n);
                        if bytes > OUTPUT_BYTES { break; }
                    }, Err(_) => break }
                },
                _ = tokio::time::sleep_until(end) => break,
                _ = fence.tick() => {
                    let valid = owner.upgrade().is_some_and(|owner| {
                        owner.store.lock().ok().is_some_and(|mut store|
                            store.validate_phase_version_fence(&helper.plan,&intent).is_ok())
                    });
                    if !valid { break; }
                    if !exited {
                        match helper.raw().exited_unreaped() {
                            Ok(true) => {
                                exited = true;
                                let _ = helper.raw().hygiene();
                                drain_deadline = Some(tokio::time::Instant::now()+Duration::from_secs(2));
                            }, Ok(false) => {}, Err(_) => break,
                        }
                    }
                },
            }
        }
    }
    let group_hygiene = helper.raw().hygiene();
    let stop_deadline = tokio::time::Instant::now() + Duration::from_secs(10);
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
    complete &= exit.is_some();
    let observation = Arc::new(NativeVersionObservation {
        plan: helper.plan.clone(),
        stdout,
        bytes,
        exit,
        complete,
        group_hygiene,
    });
    // This original observation precedes optional parsing, hashing and SQL.
    helper.state().observation = Some(observation);
    drop(guard);
}
