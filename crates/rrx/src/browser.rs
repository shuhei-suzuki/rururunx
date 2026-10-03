//! Scoped, provider-neutral browser verification. SDKs live behind a bounded private bridge.
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{domain::*, git::WorktreeManager, state::Store};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    #[default]
    Auto,
    Playwright,
    Stagehand,
}

/// Runtime-owned configuration; Project inputs cannot introduce executables or credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BrowserConfig {
    pub backend: Backend,
    pub bridge_command: Vec<String>,
    pub browser_executable: Option<PathBuf>,
    pub artifact_root: PathBuf,
    pub headed: bool,
    pub allow_loopback_actions: bool,
    pub safe_fallback: bool,
    pub timeout_ms: u64,
    pub step_timeout_ms: u64,
    pub max_output_bytes: usize,
    pub model: Option<ModelConfig>,
}
impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            backend: Backend::Auto,
            bridge_command: vec![],
            browser_executable: None,
            artifact_root: PathBuf::from(".rrx/browser"),
            headed: false,
            allow_loopback_actions: false,
            safe_fallback: true,
            timeout_ms: 120_000,
            step_timeout_ms: 30_000,
            max_output_bytes: 262_144,
            model: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelConfig {
    pub model_name: Option<String>,
    /// Name of a Project-authorized environment reference. The value never enters records.
    pub api_key_env: Option<String>,
    /// Explicit opt-in callback executable, inheriting this bridge's owned process group.
    pub custom_command: Option<Vec<String>>,
}
/// Project policy can select routing/display and tighten budgets, never add a command,
/// credential, profile, artifact root, or mutation permission.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BrowserOverlay {
    pub backend: Option<Backend>,
    pub headed: Option<bool>,
    pub safe_fallback: Option<bool>,
    pub timeout_ms: Option<u64>,
    pub step_timeout_ms: Option<u64>,
}
impl BrowserConfig {
    pub fn apply_project(&mut self, overlay: BrowserOverlay) {
        if let Some(backend) = overlay.backend {
            self.backend = backend;
        }
        self.headed |= overlay.headed.unwrap_or(false);
        self.safe_fallback &= overlay.safe_fallback.unwrap_or(true);
        self.timeout_ms = self
            .timeout_ms
            .min(overlay.timeout_ms.unwrap_or(self.timeout_ms));
        self.step_timeout_ms = self
            .step_timeout_ms
            .min(self.timeout_ms)
            .min(overlay.step_timeout_ms.unwrap_or(self.step_timeout_ms));
    }
}
impl BrowserConfig {
    pub fn validate(&self) -> Result<()> {
        for executable in self
            .bridge_command
            .first()
            .into_iter()
            .chain(
                self.model
                    .as_ref()
                    .and_then(|m| m.custom_command.as_ref())
                    .and_then(|c| c.first()),
            )
            .map(String::as_str)
            .chain(self.browser_executable.as_ref().and_then(|p| p.to_str()))
        {
            ensure!(
                Path::new(executable).is_absolute() || !executable.contains('/'),
                "browser executable paths must be absolute or resolved from runtime PATH"
            );
        }
        ensure!(
            (100..=600_000).contains(&self.timeout_ms),
            "browser timeout out of bounds"
        );
        ensure!(
            (100..=self.timeout_ms).contains(&self.step_timeout_ms),
            "step timeout out of bounds"
        );
        ensure!(
            (1024..=1_048_576).contains(&self.max_output_bytes),
            "browser output bound invalid"
        );
        ensure!(
            !self.artifact_root.as_os_str().is_empty(),
            "artifact root missing"
        );
        if let Some(model) = &self.model {
            ensure!(
                model
                    .custom_command
                    .as_ref()
                    .is_none_or(|c| !c.is_empty() && !c[0].is_empty()),
                "model command missing"
            );
            ensure!(
                model.api_key_env.is_none() || model.custom_command.is_none(),
                "choose one model credential strategy"
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Step {
    Navigate {
        url: String,
    },
    Click {
        selector: String,
    },
    Fill {
        selector: String,
        value: String,
    },
    AssertVisible {
        selector: String,
    },
    AssertText {
        selector: String,
        expected: String,
    },
    ReadText {
        selector: String,
    },
    Observe {
        instruction: String,
        scope_selector: String,
    },
    Act {
        instruction: String,
        scope_selector: String,
    },
    Extract {
        instruction: String,
        scope_selector: String,
        schema: Value,
    },
    Screenshot {
        name: String,
    },
}
impl Step {
    pub fn adaptive(&self) -> bool {
        matches!(
            self,
            Self::Observe { .. } | Self::Act { .. } | Self::Extract { .. }
        )
    }
    pub fn mutating(&self) -> bool {
        matches!(
            self,
            Self::Click { .. } | Self::Fill { .. } | Self::Act { .. }
        )
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationRequest {
    pub scope: Scope,
    pub allowed_origins: Vec<String>,
    pub steps: Vec<Step>,
    /// Explicit caller-supplied equivalent checks; never synthesized by an LLM.
    pub deterministic_fallback: Option<Vec<Step>>,
}
pub fn route(config: &BrowserConfig, steps: &[Step]) -> Result<Backend> {
    let adaptive = steps.iter().any(Step::adaptive);
    match (config.backend, adaptive) {
        (Backend::Auto, true) => Ok(Backend::Stagehand),
        (Backend::Auto, false) => Ok(Backend::Playwright),
        (Backend::Playwright, true) => anyhow::bail!("adaptive operations require Stagehand"),
        // A known deterministic flow never opens a model client, even with Stagehand preferred.
        (Backend::Stagehand, false) => Ok(Backend::Playwright),
        (backend, _) => Ok(backend),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub protocol_version: u32,
    pub playwright_version: String,
    pub stagehand_version: Option<String>,
    pub deterministic: bool,
    pub adaptive: bool,
    pub owned_connect: bool,
    #[serde(default)]
    pub external_connect: bool,
    pub headed: bool,
    pub jev: bool,
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Usage {
    pub native_sessions: Vec<SessionId>,
    /// IDs explicitly supplied to launched native children; terminal confirmation may be absent.
    pub native_session_attempts: Vec<SessionId>,
    pub llm_calls: Option<u64>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cached_input_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
    pub inference_ms: Option<u64>,
    pub source: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Failure {
    Unavailable,
    Unsupported,
    PolicyHold,
    Timeout,
    Assertion,
    Operation,
    Protocol,
    OutputLimit,
    Cleanup,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationResult {
    pub scope: Scope,
    pub session_id: SessionId,
    pub backend: Backend,
    pub success: bool,
    pub failure: Option<Failure>,
    /// True once any mutating operation has started, including an uncertain failed action.
    pub effect_possible: bool,
    pub evidence: Vec<Value>,
    pub artifacts: Vec<String>,
    pub usage: Usage,
    pub fallback_used: bool,
    #[serde(default)]
    pub artifact_directory: Option<PathBuf>,
    pub verification_file: Option<PathBuf>,
    #[serde(default)]
    pub worktree: Option<PathBuf>,
    #[serde(default)]
    pub revision: Option<String>,
}
pub fn may_fallback(
    config: &BrowserConfig,
    request: &VerificationRequest,
    result: &VerificationResult,
) -> bool {
    config.safe_fallback
        && result.backend == Backend::Stagehand
        && !result.success
        && !result.effect_possible
        && matches!(
            result.failure,
            Some(Failure::Unavailable | Failure::Unsupported)
        )
        && request
            .deterministic_fallback
            .as_ref()
            .is_some_and(|steps| {
                !steps.is_empty() && steps.iter().all(|s| !s.adaptive() && !s.mutating())
            })
}

/// Snapshot only under Store access. Validation and all external I/O follow after releasing it.
#[derive(Debug, Clone)]
pub struct BrowserBinding {
    project: Project,
    task: Task,
    other_project_environment: BTreeSet<String>,
}
impl BrowserBinding {
    pub fn project(&self) -> &Project {
        &self.project
    }
    pub fn task(&self) -> &Task {
        &self.task
    }
    pub fn capture(store: &Store, scope: &Scope) -> Result<Self> {
        let task = store
            .task(scope.task_id.context("browser requires Task scope")?)?
            .context("Task missing")?;
        let project = store
            .project(scope.project_id)?
            .context("Project missing")?;
        ensure!(
            task.scope() == *scope && project.id == task.project_id,
            "browser ownership mismatch"
        );
        ensure!(
            project.state == ProjectState::Registered,
            "Project is not registered"
        );
        let other_project_environment = store
            .projects()?
            .into_iter()
            .filter(|other| other.id != project.id && other.state != ProjectState::Removed)
            .flat_map(|other| other.environment_refs)
            .collect();
        Ok(Self {
            project,
            task,
            other_project_environment,
        })
    }
    pub fn validate(&self, scope: &Scope) -> Result<PathBuf> {
        Ok(self.validate_status(scope)?.worktree)
    }
    fn validate_status(&self, scope: &Scope) -> Result<crate::git::WorktreeStatus> {
        ensure!(
            self.task.scope() == *scope && self.project.id == self.task.project_id,
            "browser ownership mismatch"
        );
        ensure!(
            self.project.state == ProjectState::Registered,
            "Project is not registered"
        );
        WorktreeManager::validate_binding(&self.project, &self.task)
    }
}

/// Consumer #12 can persist this envelope after rechecking the snapshot under Store access.
pub fn verification_record(result: &VerificationResult) -> Result<Record> {
    Ok(Record::new(
        result.scope.clone(),
        RecordKind::Verification,
        serde_json::to_value(result)?,
    ))
}
pub trait BrowserVerifier {
    fn capabilities(&self, binding: &BrowserBinding) -> Result<Capabilities>;
    fn verify(
        &self,
        binding: &BrowserBinding,
        request: &VerificationRequest,
    ) -> Result<VerificationResult>;
    /// Arbitrary endpoints carry no trustworthy ownership or profile isolation proof.
    fn connect_external(
        &self,
        binding: &BrowserBinding,
        request: &VerificationRequest,
        _endpoint: &str,
    ) -> Result<VerificationResult> {
        binding.validate(&request.scope)?;
        Ok(empty_result(
            request,
            SessionId::new(),
            route(&BrowserConfig::default(), &request.steps)?,
            Failure::Unsupported,
        ))
    }
}
pub struct BridgeVerifier {
    pub config: BrowserConfig,
}

#[derive(Serialize)]
struct BridgeRequest<'a> {
    protocol_version: u32,
    request: &'a VerificationRequest,
    session_id: SessionId,
    backend: Backend,
    config: &'a BrowserConfig,
    artifact_dir: &'a Path,
}
struct AttemptContext<'a> {
    binding: &'a BrowserBinding,
    cwd: &'a Path,
    session_id: SessionId,
    artifacts: &'a Path,
    deadline: Instant,
}
impl BrowserVerifier for BridgeVerifier {
    fn capabilities(&self, binding: &BrowserBinding) -> Result<Capabilities> {
        self.config.validate()?;
        let cwd = binding.validate(&binding.task.scope())?;
        let output = self.run(
            binding,
            &cwd,
            b"",
            true,
            Instant::now() + Duration::from_millis(self.config.timeout_ms),
        )?;
        let caps: Capabilities =
            serde_json::from_slice(&output).context("invalid browser capabilities")?;
        ensure!(caps.protocol_version == 1, "unsupported browser protocol");
        ensure!(
            caps.stagehand_version
                .as_ref()
                .is_none_or(|v| v.starts_with("4.")),
            "Stagehand v4 required"
        );
        Ok(caps)
    }
    fn verify(
        &self,
        binding: &BrowserBinding,
        request: &VerificationRequest,
    ) -> Result<VerificationResult> {
        self.config.validate()?;
        let status = binding.validate_status(&request.scope)?;
        let cwd = status.worktree;
        let initial_revision = status.revision;
        let session_id = SessionId::new();
        let backend = route(&self.config, &request.steps)?;
        validate_request(request)?;
        if request.steps.iter().any(Step::mutating)
            && (binding.task.risk == RiskClass::R3
                || !self.config.allow_loopback_actions
                || request.allowed_origins.iter().any(|s| !loopback_origin(s)))
        {
            return Ok(empty_result(
                request,
                session_id,
                backend,
                Failure::PolicyHold,
            ));
        }
        if let Some(key) = self
            .config
            .model
            .as_ref()
            .and_then(|m| m.api_key_env.as_ref())
        {
            ensure!(
                binding.project.environment_refs.contains(key),
                "model environment reference is not authorized by Project"
            );
        }
        let artifacts = artifact_dir(&self.config.artifact_root, &request.scope, session_id)?;
        let mut profiles = vec![PrivateProfile(artifacts.join("profile"))];
        let mut result_directory = artifacts.clone();
        // One deadline covers all browser/helper attempts. Ownership validation
        // precedes this external browser phase and never consumes a short SDK budget.
        let phase_started = Instant::now();
        let deadline = phase_started + Duration::from_millis(self.config.timeout_ms);
        let mut result = self.attempt(
            &AttemptContext {
                binding,
                cwd: &cwd,
                session_id,
                artifacts: &artifacts,
                deadline,
            },
            request,
            backend,
        );
        validate_artifacts(&mut result, &artifacts, request);
        if may_fallback(&self.config, request, &result) {
            let fallback = VerificationRequest {
                scope: request.scope.clone(),
                allowed_origins: request.allowed_origins.clone(),
                steps: request.deterministic_fallback.clone().unwrap_or_default(),
                deterministic_fallback: None,
            };
            let fallback_directory = artifacts.join("fallback");
            if profiles[0].cleanup().is_err() || private_directory(&fallback_directory).is_err() {
                fail_result(&mut result, Failure::Cleanup, "fallback_profile_cleanup");
            } else {
                let primary = result;
                profiles.push(PrivateProfile(fallback_directory.join("profile")));
                result_directory = fallback_directory;
                result = self.attempt(
                    &AttemptContext {
                        binding,
                        cwd: &cwd,
                        session_id,
                        artifacts: &result_directory,
                        deadline,
                    },
                    &fallback,
                    Backend::Playwright,
                );
                result.fallback_used = true;
                result.evidence.insert(
                    0,
                    serde_json::json!({
                        "attempt": "stagehand", "failure": primary.failure,
                        "evidence": primary.evidence, "usage": primary.usage,
                        "artifact_directory": artifacts, "artifacts": primary.artifacts,
                    }),
                );
                merge_usage(&mut result.usage, &primary.usage);
            }
        }
        // Scope and paths are facts checked by Rust, never inferred from helper text.
        validate_artifacts(&mut result, &result_directory, request);
        if result.usage.native_sessions.len() > 64
            || result.usage.native_session_attempts.len() > 64
        {
            fail_result(
                &mut result,
                Failure::Protocol,
                "native_session_budget_exceeded",
            );
        }
        result.artifact_directory = Some(result_directory);
        result.worktree = Some(cwd.clone());
        result.revision = Some(initial_revision.clone());
        // Cookies/auth profiles are ephemeral even after an abnormal child exit.
        if profiles.iter().any(|profile| profile.cleanup().is_err()) {
            fail_result(&mut result, Failure::Cleanup, "profile_cleanup");
        }
        result
            .evidence
            .push(serde_json::json!({"browser_phase_ms": phase_started.elapsed().as_millis()}));
        match WorktreeManager::validate_binding(&binding.project, &binding.task) {
            Ok(current) if current.revision == initial_revision => {}
            _ => fail_result(&mut result, Failure::Protocol, "binding_changed"),
        }
        let record = artifacts.join("verification.json");
        result.verification_file = Some(record.clone());
        if File::create_new(record)
            .and_then(|mut file| {
                serde_json::to_writer(&mut file, &result).map_err(std::io::Error::other)
            })
            .is_err()
        {
            fail_result(&mut result, Failure::Cleanup, "record_write");
            result.verification_file = None;
        }
        Ok(result)
    }
}
struct PrivateProfile(PathBuf);
impl PrivateProfile {
    fn cleanup(&self) -> Result<()> {
        match fs::symlink_metadata(&self.0) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                fs::remove_file(&self.0)?;
            }
            Ok(_) => {
                fs::remove_dir_all(&self.0)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        Ok(())
    }
}
impl Drop for PrivateProfile {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
impl BridgeVerifier {
    fn attempt(
        &self,
        context: &AttemptContext<'_>,
        request: &VerificationRequest,
        backend: Backend,
    ) -> VerificationResult {
        let session_id = context.session_id;
        // Resolve runtime-owned programs before adopting a Project-controlled cwd.
        let mut config = self.config.clone();
        if let Some(executable) = &config.browser_executable {
            let Some(name) = executable.to_str() else {
                return empty_result(request, session_id, backend, Failure::Unavailable);
            };
            match crate::adapter::resolve_executable(name) {
                Ok(path) => config.browser_executable = Some(path),
                Err(_) => return empty_result(request, session_id, backend, Failure::Unavailable),
            }
        }
        if backend == Backend::Stagehand
            && let Some(command) = config
                .model
                .as_mut()
                .and_then(|m| m.custom_command.as_mut())
        {
            match crate::adapter::resolve_executable(&command[0]) {
                Ok(path) => command[0] = path.to_string_lossy().into_owned(),
                Err(_) => return empty_result(request, session_id, backend, Failure::Unavailable),
            }
        }
        let payload = match serde_json::to_vec(&BridgeRequest {
            protocol_version: 1,
            request,
            session_id,
            backend,
            config: &config,
            artifact_dir: context.artifacts,
        }) {
            Ok(payload) if payload.len() <= 65_536 => payload,
            _ => return empty_result(request, session_id, backend, Failure::Protocol),
        };
        match self.run(
            context.binding,
            context.cwd,
            &payload,
            false,
            context.deadline,
        ) {
            Ok(bytes) => match serde_json::from_slice::<VerificationResult>(&bytes) {
                Ok(mut result)
                    if result.backend == backend
                        && result.scope == request.scope
                        && result.session_id == session_id
                        && result.success == result.failure.is_none() =>
                {
                    result.fallback_used = false;
                    result
                }
                _ => {
                    let mut result = empty_result(request, session_id, backend, Failure::Protocol);
                    result.effect_possible = request.steps.iter().any(Step::mutating);
                    result
                }
            },
            Err(error) => {
                let partial = error.downcast_ref::<BridgePartial>();
                let failure = partial.map(|p| p.failure).unwrap_or_else(|| {
                    error
                        .downcast_ref::<BridgeFailure>()
                        .map_or(Failure::Operation, |e| e.0)
                });
                let mut result = partial
                    .and_then(|p| p.output.as_ref())
                    .and_then(|bytes| serde_json::from_slice::<VerificationResult>(bytes).ok())
                    .filter(|r| {
                        r.backend == backend
                            && r.scope == request.scope
                            && r.session_id == session_id
                            && r.success == r.failure.is_none()
                    })
                    .unwrap_or_else(|| empty_result(request, session_id, backend, failure));
                result.fallback_used = false;
                fail_result(&mut result, failure, "bridge_supervision");
                result.effect_possible |= request.steps.iter().any(Step::mutating);
                result
            }
        }
    }
    fn run(
        &self,
        binding: &BrowserBinding,
        cwd: &Path,
        payload: &[u8],
        capabilities: bool,
        deadline: Instant,
    ) -> Result<Vec<u8>> {
        reap_deferred();
        let command = &self.config.bridge_command;
        ensure!(
            !command.is_empty() && !command[0].is_empty(),
            "browser bridge command is not configured"
        );
        if Instant::now() >= deadline {
            return Err(BridgeFailure(Failure::Timeout).into());
        }
        let executable = crate::adapter::resolve_executable(&command[0])
            .map_err(|_| BridgeFailure(Failure::Unavailable))?;
        let mut process = Command::new(executable);
        process
            .args(&command[1..])
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        // Preserve native user config/auth/rules while making API credentials Project-explicit.
        process.env_clear();
        for key in [
            "PATH",
            "HOME",
            "USER",
            "LOGNAME",
            "TMPDIR",
            "LANG",
            "LC_ALL",
            "XDG_CONFIG_HOME",
        ]
        .into_iter()
        .chain(binding.project.environment_refs.iter().map(String::as_str))
        {
            if let Some(value) = std::env::var_os(key) {
                process.env(key, value);
            }
        }
        if self.config.headed {
            for key in [
                "DISPLAY",
                "WAYLAND_DISPLAY",
                "XAUTHORITY",
                "XDG_RUNTIME_DIR",
            ] {
                if (!binding.other_project_environment.contains(key)
                    || binding
                        .project
                        .environment_refs
                        .iter()
                        .any(|reference| reference == key))
                    && let Some(value) = std::env::var_os(key)
                {
                    process.env(key, value);
                }
            }
        }
        for (key, value) in std::env::vars_os() {
            if key.to_str().is_some_and(|key| {
                native_environment(key)
                    && (!binding.other_project_environment.contains(key)
                        || binding
                            .project
                            .environment_refs
                            .iter()
                            .any(|reference| reference == key))
            }) {
                process.env(key, value);
            }
        }
        if capabilities {
            process.arg("--capabilities");
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            process.process_group(0);
        }
        #[cfg(not(unix))]
        {
            anyhow::bail!("owned browser process groups currently require Unix");
        }
        let child = process
            .spawn()
            .map_err(|_| BridgeFailure(Failure::Unavailable))?;
        let id = child.id();
        let mut child = OwnedChild {
            child: Some(child),
            id,
            stopped: false,
        };
        let mut stdin = child
            .child
            .as_mut()
            .unwrap()
            .stdin
            .take()
            .context("bridge stdin missing")?;
        let input = payload.to_vec();
        let (tx, rx) = mpsc::channel();
        let sender = tx.clone();
        let writer = thread::Builder::new()
            .spawn(move || {
                let outcome = stdin.write_all(&input);
                drop(stdin);
                let _ = sender.send(outcome.map(|_| None));
            })
            .map_err(|_| BridgeFailure(Failure::Cleanup))?;
        let stdout = child
            .child
            .as_mut()
            .unwrap()
            .stdout
            .take()
            .context("bridge stdout missing")?;
        let bound = self.config.max_output_bytes;
        let reader = thread::Builder::new()
            .spawn(move || {
                let mut output = vec![];
                let outcome = stdout.take((bound + 1) as u64).read_to_end(&mut output);
                let _ = tx.send(outcome.map(|_| Some(output)));
            })
            .map_err(|_| BridgeFailure(Failure::Cleanup))?;
        let mut output = None;
        let mut failed = None;
        loop {
            while let Ok(value) = rx.try_recv() {
                match value {
                    Ok(Some(bytes)) if bytes.len() > bound => failed = Some(Failure::OutputLimit),
                    Ok(Some(bytes)) => output = Some(bytes),
                    Ok(None) => {}
                    Err(_) => failed = Some(Failure::Protocol),
                }
            }
            if failed.is_some() || Instant::now() >= deadline {
                if failed.is_none() {
                    failed = Some(Failure::Timeout);
                }
                break;
            }
            match child.exited() {
                Ok(true) => break,
                Ok(false) => {}
                Err(_) => {
                    failed = Some(Failure::Operation);
                    break;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        // Kill descendants even when the bridge exits successfully; no shared/global sessions.
        // Keep the leader unreaped until its group is stopped. This reserves its
        // PID/group identity and prevents signalling an unrelated reused PID.
        let status = child.stop();
        let io_deadline = Instant::now() + Duration::from_millis(500);
        while (!writer.is_finished() || !reader.is_finished()) && Instant::now() < io_deadline {
            thread::sleep(Duration::from_millis(5));
        }
        if writer.is_finished() && reader.is_finished() {
            let _ = writer.join();
            let _ = reader.join();
        } else {
            // Trusted callbacks must inherit the owned group. An escaped pipe
            // holder is uncertain cleanup, never an unbounded supervisor wait.
            failed = Some(Failure::Cleanup);
        }
        while let Ok(value) = rx.try_recv() {
            match value {
                Ok(Some(bytes)) if bytes.len() > bound => {
                    failed.get_or_insert(Failure::OutputLimit);
                }
                Ok(Some(bytes)) => output = Some(bytes),
                Ok(None) => {}
                Err(_) => {
                    failed.get_or_insert(Failure::Protocol);
                }
            }
        }
        if status.is_err() {
            failed = Some(Failure::Cleanup);
        }
        if let Some(failure) = failed {
            return Err(BridgePartial { failure, output }.into());
        }
        if !status
            .map_err(|_| BridgeFailure(Failure::Cleanup))?
            .success()
        {
            return Err(BridgePartial {
                failure: Failure::Operation,
                output,
            }
            .into());
        }
        output.context("bridge output missing")
    }
}
#[derive(Debug)]
struct BridgeFailure(Failure);
#[derive(Debug)]
struct BridgePartial {
    failure: Failure,
    output: Option<Vec<u8>>,
}
impl std::fmt::Display for BridgePartial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "browser bridge {:?}", self.failure)
    }
}
impl std::error::Error for BridgePartial {}
fn native_environment(key: &str) -> bool {
    key.starts_with("CLAUDE_")
        || key.starts_with("ANTHROPIC_")
        || matches!(
            key,
            "HTTP_PROXY"
                | "HTTPS_PROXY"
                | "ALL_PROXY"
                | "NO_PROXY"
                | "http_proxy"
                | "https_proxy"
                | "all_proxy"
                | "no_proxy"
                | "NODE_EXTRA_CA_CERTS"
                | "NODE_OPTIONS"
                | "SSL_CERT_FILE"
                | "SSL_CERT_DIR"
        )
}
impl std::fmt::Display for BridgeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "browser bridge {:?}", self.0)
    }
}
impl std::error::Error for BridgeFailure {}
struct OwnedChild {
    child: Option<Child>,
    id: u32,
    stopped: bool,
}
impl OwnedChild {
    fn exited(&self) -> std::io::Result<bool> {
        #[cfg(unix)]
        {
            use rustix::process::{Pid, WaitId, WaitIdOptions, waitid};
            let pid = Pid::from_raw(self.id as i32)
                .ok_or_else(|| std::io::Error::other("invalid owned PID"))?;
            match waitid(
                WaitId::Pid(pid),
                WaitIdOptions::EXITED | WaitIdOptions::NOWAIT | WaitIdOptions::NOHANG,
            ) {
                Ok(status) => Ok(status.is_some()),
                Err(rustix::io::Errno::INTR) => Ok(false),
                Err(error) => Err(error.into()),
            }
        }
        #[cfg(not(unix))]
        {
            Err(std::io::Error::other(
                "owned process observation requires Unix",
            ))
        }
    }
    fn stop(&mut self) -> std::io::Result<ExitStatus> {
        self.stopped = true;
        #[cfg(unix)]
        {
            let grace = if self.exited().unwrap_or(false) {
                100
            } else {
                1000
            };
            let _ = signal_owned_group(self.id, rustix::process::Signal::TERM);
            thread::sleep(Duration::from_millis(grace));
            let cleanup = signal_owned_group(self.id, rustix::process::Signal::KILL);
            if cleanup.is_err() {
                let _ = self.child.as_mut().unwrap().kill();
            }
            let deadline = Instant::now() + Duration::from_millis(500);
            while !self.exited().unwrap_or(false) && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(5));
            }
            let observed_exit = self.exited().unwrap_or(false);
            let mut child = self.child.take().unwrap();
            if !observed_exit {
                // Retain the direct child in a reaper; never signal a recycled
                // group after reaping. Caller receives uncertain Cleanup.
                defer_reap(child);
                return Err(std::io::Error::other("owned child cleanup unconfirmed"));
            }
            let status = child.wait();
            cleanup?;
            status
        }
        #[cfg(not(unix))]
        {
            Err(std::io::Error::other("owned process groups require Unix"))
        }
    }
}
// A resource-exhausted host must not panic inside Drop. If even a reaper thread
// cannot start, retain the unreaped Child/PID until a later browser call polls it.
static DEFERRED_CHILDREN: Mutex<Vec<Child>> = Mutex::new(Vec::new());
fn defer_reap(child: Child) {
    let slot = Arc::new(Mutex::new(Some(child)));
    let worker = Arc::clone(&slot);
    if thread::Builder::new()
        .spawn(move || {
            if let Some(mut child) = worker.lock().unwrap_or_else(|p| p.into_inner()).take() {
                let _ = child.wait();
            }
        })
        .is_err()
        && let Some(child) = slot.lock().unwrap_or_else(|p| p.into_inner()).take()
    {
        DEFERRED_CHILDREN
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(child);
    }
}
fn reap_deferred() {
    DEFERRED_CHILDREN
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .retain_mut(|child| !matches!(child.try_wait(), Ok(Some(_))));
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.stopped {
            let _ = self.stop();
        }
    }
}
#[cfg(unix)]
fn signal_owned_group(id: u32, signal: rustix::process::Signal) -> std::io::Result<()> {
    use rustix::process::{Pid, kill_process_group};
    let pid = Pid::from_raw(id as i32)
        .filter(|p| p.as_raw_nonzero().get() > 1)
        .ok_or_else(|| std::io::Error::other("invalid owned process group"))?;
    match kill_process_group(pid, signal) {
        Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
        #[cfg(target_os = "macos")]
        Err(rustix::io::Errno::PERM) if crate::adapter::macos_group_is_dead(pid)? => Ok(()),
        Err(error) => Err(error.into()),
    }
}
fn fail_result(result: &mut VerificationResult, failure: Failure, code: &str) {
    result.success = false;
    result.failure = Some(failure);
    result
        .evidence
        .push(serde_json::json!({"failure_code": code}));
}
fn validate_artifacts(
    result: &mut VerificationResult,
    directory: &Path,
    request: &VerificationRequest,
) {
    if result.artifacts.iter().any(|name| {
        !safe_name(name)
            || !name.ends_with(".png")
            || !fs::symlink_metadata(directory.join(name)).is_ok_and(|m| m.is_file())
    }) {
        result.artifacts.clear();
        fail_result(result, Failure::Protocol, "invalid_artifact");
        result.effect_possible |= request.steps.iter().any(Step::mutating);
    }
}
fn private_directory(path: &Path) -> std::io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)
}
fn merge_usage(final_usage: &mut Usage, primary: &Usage) {
    for (current, earlier) in [
        (&mut final_usage.llm_calls, primary.llm_calls),
        (&mut final_usage.input_tokens, primary.input_tokens),
        (&mut final_usage.output_tokens, primary.output_tokens),
        (
            &mut final_usage.cached_input_tokens,
            primary.cached_input_tokens,
        ),
        (
            &mut final_usage.cache_write_tokens,
            primary.cache_write_tokens,
        ),
        (&mut final_usage.inference_ms, primary.inference_ms),
    ] {
        *current = current.zip(earlier).and_then(|(a, b)| a.checked_add(b));
    }
    final_usage.cost_usd = final_usage
        .cost_usd
        .zip(primary.cost_usd)
        .map(|(a, b)| a + b)
        .filter(|sum| sum.is_finite());
    final_usage
        .native_sessions
        .extend_from_slice(&primary.native_sessions);
    final_usage
        .native_session_attempts
        .extend_from_slice(&primary.native_session_attempts);
    final_usage.source = Some("browser-attempts".into());
}
fn empty_result(
    request: &VerificationRequest,
    session_id: SessionId,
    backend: Backend,
    failure: Failure,
) -> VerificationResult {
    VerificationResult {
        scope: request.scope.clone(),
        session_id,
        backend,
        success: false,
        failure: Some(failure),
        effect_possible: false,
        evidence: vec![],
        artifacts: vec![],
        usage: Usage::default(),
        fallback_used: false,
        artifact_directory: None,
        verification_file: None,
        worktree: None,
        revision: None,
    }
}
fn safe_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        && !value.starts_with('.')
}
fn loopback_origin(origin: &str) -> bool {
    let Some(authority) = origin.strip_prefix("http://") else {
        return false;
    };
    let authority = authority.trim_end_matches('/');
    ["localhost", "127.0.0.1", "[::1]"].iter().any(|host| {
        authority == *host
            || authority
                .strip_prefix(&format!("{host}:"))
                .is_some_and(|port| {
                    !port.is_empty()
                        && port.bytes().all(|b| b.is_ascii_digit())
                        && port.parse::<u16>().is_ok_and(|p| p != 0)
                })
    })
}
fn validate_request(request: &VerificationRequest) -> Result<()> {
    ensure!(
        request.scope.task_id.is_some() && request.scope.goal_id.is_some(),
        "browser requires full Task scope"
    );
    ensure!(
        !request.steps.is_empty() && request.steps.len() <= 64,
        "browser step budget exceeded"
    );
    ensure!(
        !request.allowed_origins.is_empty() && request.allowed_origins.len() <= 8,
        "origin policy missing or too large"
    );
    ensure!(
        serde_json::to_vec(request)?.len() <= 48_000,
        "browser context input budget exceeded"
    );
    if let Some(steps) = &request.deterministic_fallback {
        ensure!(
            steps.len() <= 64 && steps.iter().all(|s| !s.adaptive() && !s.mutating()),
            "fallback must contain deterministic read-only checks"
        );
    }
    for step in request
        .steps
        .iter()
        .chain(request.deterministic_fallback.iter().flatten())
    {
        if let Step::Screenshot { name } = step {
            ensure!(
                safe_name(name) && name.ends_with(".png"),
                "invalid screenshot name"
            );
        }
    }
    Ok(())
}
fn artifact_dir(root: &Path, scope: &Scope, session: SessionId) -> Result<PathBuf> {
    fs::create_dir_all(root)?;
    let root = fs::canonicalize(root)?;
    let mut path = root;
    for component in [
        scope.project_id.to_string(),
        scope.task_id.context("Task missing")?.to_string(),
        session.to_string(),
    ] {
        path.push(&component);
        match fs::create_dir(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                ensure!(
                    fs::symlink_metadata(&path)?.is_dir(),
                    "browser artifact ancestor is not a directory"
                );
                ensure!(
                    !fs::symlink_metadata(&path)?.file_type().is_symlink(),
                    "browser artifact ancestor is a symlink"
                );
                ensure!(
                    component != session.to_string(),
                    "browser session directory already exists"
                );
            }
            Err(e) => return Err(e.into()),
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
        }
    }
    Ok(path)
}
