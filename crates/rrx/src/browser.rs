//! Scoped, provider-neutral browser verification. SDKs live behind a bounded private bridge.
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::mpsc,
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
        ensure!(
            self.task.scope() == *scope && self.project.id == self.task.project_id,
            "browser ownership mismatch"
        );
        ensure!(
            self.project.state == ProjectState::Registered,
            "Project is not registered"
        );
        Ok(WorktreeManager::validate_binding(&self.project, &self.task)?.worktree)
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
        let deadline = Instant::now() + Duration::from_millis(self.config.timeout_ms);
        let cwd = binding.validate(&request.scope)?;
        let initial_revision =
            WorktreeManager::validate_binding(&binding.project, &binding.task)?.revision;
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
        let profile = PrivateProfile(artifacts.join("profile"));
        let mut result =
            self.attempt(binding, request, session_id, backend, &artifacts, deadline)?;
        if may_fallback(&self.config, request, &result) {
            let fallback = VerificationRequest {
                scope: request.scope.clone(),
                allowed_origins: request.allowed_origins.clone(),
                steps: request.deterministic_fallback.clone().unwrap_or_default(),
                deterministic_fallback: None,
            };
            result = self.attempt(
                binding,
                &fallback,
                session_id,
                Backend::Playwright,
                &artifacts,
                deadline,
            )?;
            result.fallback_used = true;
        }
        // Scope and paths are facts checked by Rust, never inferred from helper text.
        ensure!(
            result.scope == request.scope && result.session_id == session_id,
            "browser response ownership mismatch"
        );
        for path in &result.artifacts {
            ensure!(
                safe_name(path) && path.ends_with(".png"),
                "invalid browser artifact reference"
            );
            let file = artifacts.join(path);
            ensure!(
                fs::symlink_metadata(&file)?.is_file(),
                "artifact must be a regular owned file"
            );
        }
        ensure!(
            result.usage.native_sessions.len() <= 64,
            "native callback session budget exceeded"
        );
        result.artifact_directory = Some(artifacts.clone());
        result.worktree = Some(cwd.clone());
        result.revision = Some(initial_revision.clone());
        if WorktreeManager::validate_binding(&binding.project, &binding.task)?.revision
            != initial_revision
        {
            result.success = false;
            result.failure = Some(Failure::Protocol);
            result
                .evidence
                .push(serde_json::json!({"binding": "revision_changed"}));
        }
        // Cookies/auth profiles are ephemeral even after an abnormal child exit.
        if profile.cleanup().is_err() {
            result.success = false;
            result.failure = Some(Failure::Cleanup);
        }
        let record = artifacts.join("verification.json");
        let mut file = File::create_new(record)?;
        serde_json::to_writer(&mut file, &result)?;
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
        binding: &BrowserBinding,
        request: &VerificationRequest,
        session_id: SessionId,
        backend: Backend,
        artifacts: &Path,
        deadline: Instant,
    ) -> Result<VerificationResult> {
        let cwd = binding
            .task
            .worktree
            .as_deref()
            .context("Task worktree missing")?;
        let payload = serde_json::to_vec(&BridgeRequest {
            protocol_version: 1,
            request,
            session_id,
            backend,
            config: &self.config,
            artifact_dir: artifacts,
        })?;
        ensure!(
            payload.len() <= 65_536,
            "browser request exceeds input budget"
        );
        match self.run(binding, cwd, &payload, false, deadline) {
            Ok(bytes) => match serde_json::from_slice::<VerificationResult>(&bytes) {
                Ok(result)
                    if result.backend == backend
                        && result.scope == request.scope
                        && result.session_id == session_id =>
                {
                    Ok(result)
                }
                _ => {
                    let mut result = empty_result(request, session_id, backend, Failure::Protocol);
                    result.effect_possible = request.steps.iter().any(Step::mutating);
                    Ok(result)
                }
            },
            Err(error) => {
                let failure = error
                    .downcast_ref::<BridgeFailure>()
                    .map_or(Failure::Operation, |e| e.0);
                let mut result = empty_result(request, session_id, backend, failure);
                result.effect_possible = request.steps.iter().any(Step::mutating);
                Ok(result)
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
        let command = &self.config.bridge_command;
        ensure!(
            !command.is_empty() && !command[0].is_empty(),
            "browser bridge command is not configured"
        );
        if Instant::now() >= deadline {
            return Err(BridgeFailure(Failure::Timeout).into());
        }
        let mut process = Command::new(&command[0]);
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
            "CLAUDE_CONFIG_DIR",
        ]
        .into_iter()
        .chain(binding.project.environment_refs.iter().map(String::as_str))
        {
            if let Some(value) = std::env::var_os(key) {
                process.env(key, value);
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
            child,
            id,
            stopped: false,
        };
        let mut stdin = child.child.stdin.take().context("bridge stdin missing")?;
        let input = payload.to_vec();
        let (tx, rx) = mpsc::channel();
        let sender = tx.clone();
        let writer = thread::spawn(move || {
            let outcome = stdin.write_all(&input);
            drop(stdin);
            let _ = sender.send(outcome.map(|_| None));
        });
        let stdout = child.child.stdout.take().context("bridge stdout missing")?;
        let bound = self.config.max_output_bytes;
        let reader = thread::spawn(move || {
            let mut output = vec![];
            let outcome = stdout.take((bound + 1) as u64).read_to_end(&mut output);
            let _ = tx.send(outcome.map(|_| Some(output)));
        });
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
            if output.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        // Kill descendants even when the bridge exits successfully; no shared/global sessions.
        // Keep the leader unreaped until its group is stopped. This reserves its
        // PID/group identity and prevents signalling an unrelated reused PID.
        let status = child.stop();
        let _ = writer.join();
        let _ = reader.join();
        if let Some(failure) = failed {
            return Err(BridgeFailure(failure).into());
        }
        if !status
            .map_err(|_| BridgeFailure(Failure::Operation))?
            .success()
        {
            return Err(BridgeFailure(Failure::Operation).into());
        }
        output.context("bridge output missing")
    }
}
#[derive(Debug)]
struct BridgeFailure(Failure);
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
    child: Child,
    id: u32,
    stopped: bool,
}
impl OwnedChild {
    fn stop(&mut self) -> std::io::Result<ExitStatus> {
        self.stopped = true;
        terminate_group(&mut self.child, self.id)
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.stopped {
            let _ = self.stop();
        }
    }
}
fn terminate_group(child: &mut Child, id: u32) -> std::io::Result<ExitStatus> {
    #[cfg(unix)]
    {
        let group = format!("-{id}");
        let _ = Command::new("/bin/kill")
            .args(["-TERM", "--", &group])
            .stderr(Stdio::null())
            .status();
        thread::sleep(Duration::from_millis(100));
        let _ = Command::new("/bin/kill")
            .args(["-KILL", "--", &group])
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
    child.wait()
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
