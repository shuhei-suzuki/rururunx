//! `rrx daemon start/status/stop`: the same service, detached. The owner flock
//! is the sole arbiter; PIDs, descriptors and exits are never authority.
use super::{
    endpoint::{self, Discovery},
    service::{self, Readiness},
    transport::{self, ServiceIdentity},
};
use crate::runtime::{
    control::{ControlAction, ControlRequest, ControlResponse},
    stop::ShutdownSite,
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    ffi::OsString,
    fs::OpenOptions,
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

const READINESS_DEADLINE: Duration = Duration::from_secs(30);
const READINESS_BYTES: u64 = 4096;
const POLL: Duration = Duration::from_millis(20);

/// The owner-lock input L of the decision table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnerLock {
    /// Acquired shared and released at once, or the lock file is absent.
    Free,
    /// `EWOULDBLOCK`: an owner holds it.
    Busy,
    /// Any other failure.
    Error,
}

/// A validated live identity, as reported by `running` and `already running`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LiveIdentity {
    pub state: PathBuf,
    pub instance: String,
    pub epoch: u64,
    pub protocol: u32,
}
impl From<ServiceIdentity> for LiveIdentity {
    fn from(identity: ServiceIdentity) -> Self {
        Self {
            state: identity.state,
            instance: identity.instance,
            epoch: identity.epoch,
            protocol: transport::PROTOCOL_VERSION,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DaemonStatus {
    Running {
        identity: LiveIdentity,
        service_running: bool,
        /// Native readiness, reported separately from service readiness.
        operational: bool,
    },
    /// The owner lock is held but no endpoint validates.
    OwnerBusy,
    /// Stale, malformed or mismatched descriptor; refused or timed-out
    /// connection; or an identity that does not validate. Never "stopped".
    DiscoveryUnavailable { reason: String },
    /// Only an absent descriptor together with a free owner lock.
    NotRunning,
}
impl DaemonStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Running { .. } => "running",
            Self::OwnerBusy => "owner busy, identity unconfirmed",
            Self::DiscoveryUnavailable { .. } => "discovery unavailable",
            Self::NotRunning => "not running",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "start", rename_all = "snake_case")]
pub enum StartOutcome {
    /// This invocation's own child announced the identity that now validates.
    Running {
        identity: LiveIdentity,
    },
    AlreadyRunning {
        identity: LiveIdentity,
    },
    OwnerBusy,
    DiscoveryUnavailable {
        reason: String,
    },
    StartFailed {
        log: PathBuf,
        reason: String,
    },
    /// No conclusion. The child is not killed: exit is never cleanup.
    StartUnconfirmed {
        log: PathBuf,
    },
}
impl StartOutcome {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Running { .. } => "running",
            Self::AlreadyRunning { .. } => "already running",
            Self::OwnerBusy => "owner busy, identity unconfirmed",
            Self::DiscoveryUnavailable { .. } => "discovery unavailable",
            Self::StartFailed { .. } => "start failed",
            Self::StartUnconfirmed { .. } => "start unconfirmed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "stop", rename_all = "snake_case")]
pub enum StopOutcome {
    /// The only outcome that counts as stopped; still not cleanup success.
    Completed {
        instance: String,
        epoch: u64,
    },
    /// Owned work is held at `site`; the service is not stopped.
    Pending {
        instance: String,
        epoch: u64,
        site: ShutdownSite,
    },
    Failed,
    /// The request may have been delivered; no response arrived.
    Unconfirmed,
    /// Nothing was sent: no endpoint validated.
    NotSent {
        status: DaemonStatus,
    },
}
impl StopOutcome {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Completed { .. } => "stop completed",
            Self::Pending { .. } => "stop pending",
            Self::Failed => "stop failed",
            Self::Unconfirmed => "stop unconfirmed",
            Self::NotSent { status } => status.label(),
        }
    }
}

/// Absolute, and canonical when it exists; nothing is created.
fn absolute_state(state: &Path) -> Result<PathBuf> {
    let state = if state.is_absolute() {
        state.to_path_buf()
    } else {
        std::env::current_dir()?.join(state)
    };
    match state.canonicalize() {
        Ok(state) => Ok(state),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(state),
        Err(e) => Err(e.into()),
    }
}

/// `flock(LOCK_SH|LOCK_NB)` on an existing `owner.lock`, never created,
/// released at once. Neither an epoch nor any state is touched.
pub fn probe_lock(state: &Path) -> OwnerLock {
    let Ok(state) = absolute_state(state) else {
        return OwnerLock::Error;
    };
    let Ok(root) = endpoint::execution_root(&state) else {
        return OwnerLock::Error;
    };
    // Non-blocking, so a special file (a FIFO without a writer) cannot hold
    // the probe in `open`; only a regular file reaches `flock`.
    let file = match OpenOptions::new()
        .read(true)
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32)
        .open(root.join("owner.lock"))
    {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return OwnerLock::Free,
        Err(_) => return OwnerLock::Error,
    };
    if !file.metadata().is_ok_and(|m| m.is_file()) {
        return OwnerLock::Error;
    }
    match rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockShared) {
        Ok(()) => OwnerLock::Free,
        Err(rustix::io::Errno::WOULDBLOCK) => OwnerLock::Busy,
        Err(_) => OwnerLock::Error,
    }
}

fn reason(error: &anyhow::Error) -> String {
    format!("{error:#}")
}

/// The status decision table: D first; L only when D is not valid.
pub async fn status(state: &Path) -> DaemonStatus {
    match endpoint::discover(state).await {
        Discovery::Valid(identity, mut connection) => {
            match runtime_status(&identity, &mut connection).await {
                Ok((service_running, operational)) => DaemonStatus::Running {
                    identity: identity.into(),
                    service_running,
                    operational,
                },
                Err(error) => DaemonStatus::DiscoveryUnavailable {
                    reason: reason(&error),
                },
            }
        }
        Discovery::Absent => match probe_lock(state) {
            OwnerLock::Free => DaemonStatus::NotRunning,
            OwnerLock::Busy => DaemonStatus::OwnerBusy,
            OwnerLock::Error => DaemonStatus::DiscoveryUnavailable {
                reason: "owner lock probe failed".into(),
            },
        },
        // A stale descriptor is reported, never deleted, whatever L says.
        Discovery::Invalid(error) => DaemonStatus::DiscoveryUnavailable {
            reason: reason(&error),
        },
    }
}

async fn runtime_status(
    identity: &ServiceIdentity,
    connection: &mut BufReader<tokio::net::UnixStream>,
) -> Result<(bool, bool)> {
    match exchange(identity, connection, ControlAction::RuntimeStatus).await? {
        ControlResponse::RuntimeMetadata {
            instance,
            epoch,
            service_running,
            operational,
            ..
        } if instance == identity.instance && epoch == identity.epoch => {
            Ok((service_running, operational))
        }
        _ => anyhow::bail!("unexpected Runtime status response"),
    }
}

async fn exchange(
    identity: &ServiceIdentity,
    connection: &mut BufReader<tokio::net::UnixStream>,
    action: ControlAction,
) -> Result<ControlResponse> {
    let request = ControlRequest {
        request_id: uuid::Uuid::new_v4(),
        instance: identity.instance.clone(),
        epoch: identity.epoch,
        action,
    };
    transport::send(connection.get_mut(), &request, transport::REQUEST_BYTES).await?;
    transport::receive(connection, transport::RESPONSE_BYTES).await
}

/// A `RuntimeStop` request through the endpoint, never a signal.
pub async fn stop(state: &Path) -> StopOutcome {
    let (identity, mut connection) = match endpoint::discover(state).await {
        Discovery::Valid(identity, connection) => (identity, connection),
        Discovery::Absent => {
            return StopOutcome::NotSent {
                status: status(state).await,
            };
        }
        Discovery::Invalid(error) => {
            return StopOutcome::NotSent {
                status: DaemonStatus::DiscoveryUnavailable {
                    reason: reason(&error),
                },
            };
        }
    };
    let request = ControlRequest {
        request_id: uuid::Uuid::new_v4(),
        instance: identity.instance.clone(),
        epoch: identity.epoch,
        action: ControlAction::RuntimeStop,
    };
    if transport::send(connection.get_mut(), &request, transport::REQUEST_BYTES)
        .await
        .is_err()
    {
        return StopOutcome::Unconfirmed;
    }
    match transport::receive::<ControlResponse>(&mut connection, transport::RESPONSE_BYTES).await {
        Ok(ControlResponse::RuntimeStopped { instance, epoch })
            if instance == identity.instance && epoch == identity.epoch =>
        {
            StopOutcome::Completed { instance, epoch }
        }
        Ok(ControlResponse::RuntimeStopPending {
            instance,
            epoch,
            site,
        }) if instance == identity.instance && epoch == identity.epoch => StopOutcome::Pending {
            instance,
            epoch,
            site,
        },
        Ok(_) => StopOutcome::Failed,
        // I/O timeout or EOF after the request was sent.
        Err(_) => StopOutcome::Unconfirmed,
    }
}

/// The program that runs the service: this executable, `serve --detached`.
pub struct Launch {
    program: PathBuf,
    leading: Vec<OsString>,
}
impl Launch {
    pub fn current() -> Result<Self> {
        Ok(Self {
            program: std::env::current_exe().context("locate rrx executable")?,
            leading: Vec::new(),
        })
    }
    /// `program leading... --state S [--config C] serve --detached`.
    pub fn new(program: PathBuf, leading: Vec<OsString>) -> Self {
        Self { program, leading }
    }
}

pub async fn start(state: &Path, config: Option<&Path>) -> StartOutcome {
    match Launch::current() {
        Ok(launch) => start_with(&launch, state, config).await,
        Err(error) => StartOutcome::StartFailed {
            log: PathBuf::new(),
            reason: reason(&error),
        },
    }
}

/// The start table acts on D and L, not on the `status` label: an invalid
/// descriptor with a free lock is spawned over, so the exclusive owner's
/// `bind` can replace a known-shape leftover for the same state.
pub async fn start_with(launch: &Launch, state: &Path, config: Option<&Path>) -> StartOutcome {
    match endpoint::discover(state).await {
        Discovery::Valid(identity, _) => StartOutcome::AlreadyRunning {
            identity: identity.into(),
        },
        Discovery::Absent => match probe_lock(state) {
            OwnerLock::Free => spawn(launch, state, config).await,
            OwnerLock::Busy => StartOutcome::OwnerBusy,
            OwnerLock::Error => StartOutcome::DiscoveryUnavailable {
                reason: "owner lock probe failed".into(),
            },
        },
        Discovery::Invalid(error) => match probe_lock(state) {
            OwnerLock::Free => spawn(launch, state, config).await,
            OwnerLock::Busy | OwnerLock::Error => StartOutcome::DiscoveryUnavailable {
                reason: reason(&error),
            },
        },
    }
}

/// `<state>.execution/daemon/serve.log`: 0700 directory, 0600 regular file
/// owned by this UID with a single link, opened without following links.
fn open_log(state: &Path) -> Result<(PathBuf, std::fs::File)> {
    let parent = state.parent().context("state needs parent")?;
    std::fs::create_dir_all(parent)?;
    let root = endpoint::execution_root(state)?;
    std::fs::create_dir_all(&root)?;
    endpoint::owned_execution_root(&root)?;
    let directory = root.join("daemon");
    match std::fs::DirBuilder::new().mode(0o700).create(&directory) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e.into()),
    }
    endpoint::owned_private_directory(&directory)?;
    let path = directory.join("serve.log");
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .mode(0o600)
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32)
        .open(&path)
        .with_context(|| format!("open daemon log {}", path.display()))?;
    let m = file.metadata()?;
    anyhow::ensure!(
        m.is_file()
            && m.nlink() == 1
            && m.uid() == rustix::process::getuid().as_raw()
            && m.mode() & 0o077 == 0,
        "daemon log ownership or permissions invalid"
    );
    Ok((path, file))
}

async fn spawn(launch: &Launch, state: &Path, config: Option<&Path>) -> StartOutcome {
    let failed = |log: &Path, error: anyhow::Error| StartOutcome::StartFailed {
        log: log.to_path_buf(),
        reason: reason(&error),
    };
    let state = match absolute_state(state) {
        Ok(state) => state,
        Err(error) => return failed(Path::new(""), error),
    };
    let (log, file) = match open_log(&state) {
        Ok(opened) => opened,
        Err(error) => return failed(Path::new(""), error),
    };
    let config = match config.map(absolute_state).transpose() {
        Ok(config) => config,
        Err(error) => return failed(&log, error),
    };
    let mut command = tokio::process::Command::new(&launch.program);
    command.args(&launch.leading).arg("--state").arg(&state);
    if let Some(config) = &config {
        command.arg("--config").arg(config);
    }
    command
        .args(["serve", "--detached"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(file))
        .current_dir(state.parent().unwrap_or(Path::new("/")))
        .kill_on_drop(false);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => return failed(&log, error.into()),
    };
    let Some(stdout) = child.stdout.take() else {
        return failed(&log, anyhow::anyhow!("readiness channel missing"));
    };
    let mut readiness = BufReader::new(stdout.take(READINESS_BYTES));
    let mut line = Vec::new();
    let deadline = tokio::time::Instant::now() + READINESS_DEADLINE;
    let mut ended = false;
    let announced = loop {
        match child.try_wait() {
            Ok(Some(exit)) => {
                // Anything the child wrote before exiting is already in the pipe.
                if !ended {
                    let _ = tokio::time::timeout(
                        Duration::from_millis(200),
                        readiness.read_until(b'\n', &mut line),
                    )
                    .await;
                }
                if line.ends_with(b"\n") {
                    break line;
                }
                return exited(&state, &log, exit.code()).await;
            }
            Ok(None) => {}
            Err(error) => return failed(&log, error.into()),
        }
        if tokio::time::Instant::now() >= deadline {
            return StartOutcome::StartUnconfirmed { log };
        }
        if ended {
            tokio::time::sleep(POLL).await;
            continue;
        }
        tokio::select! {
            read = readiness.read_until(b'\n', &mut line) => match read {
                Ok(0) => ended = true,
                Ok(_) if line.ends_with(b"\n") => break line,
                // The bounded channel ended without a delimiter.
                Ok(_) => ended = true,
                Err(error) => return failed(&log, error.into()),
            },
            () = tokio::time::sleep(POLL) => {}
        }
    };
    let announced: Readiness = match serde_json::from_slice(&announced[..announced.len() - 1]) {
        Ok(announced) => announced,
        Err(_) => return failed(&log, anyhow::anyhow!("malformed readiness announcement")),
    };
    #[cfg(test)]
    tests::pause(tests::BEFORE_IDENTITY_MATCH, &state);
    // Own-child line, then a separately checked live identity match.
    match endpoint::discover(&state).await {
        Discovery::Valid(identity, _)
            if identity.instance == announced.instance && identity.epoch == announced.epoch =>
        {
            StartOutcome::Running {
                identity: identity.into(),
            }
        }
        Discovery::Valid(identity, _) => StartOutcome::AlreadyRunning {
            identity: identity.into(),
        },
        _ => StartOutcome::StartUnconfirmed { log },
    }
}

/// A child that exited without the line. A winner's endpoint that this
/// child never announced is never success.
async fn exited(state: &Path, log: &Path, code: Option<i32>) -> StartOutcome {
    if code != Some(i32::from(service::EXIT_OWNER_BUSY)) {
        return StartOutcome::StartFailed {
            log: log.to_path_buf(),
            reason: match code {
                Some(code) => format!("service exited with code {code} before readiness"),
                None => "service ended by a signal before readiness".into(),
            },
        };
    }
    match endpoint::discover(state).await {
        Discovery::Valid(identity, _) => StartOutcome::AlreadyRunning {
            identity: identity.into(),
        },
        _ => StartOutcome::OwnerBusy,
    }
}

#[cfg(test)]
mod tests;
