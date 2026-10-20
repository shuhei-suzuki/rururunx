//! Foreground product control service. This composes controls, not native drivers.
use super::{endpoint::ControlEndpoint, transport};
use crate::{
    config::Config,
    execution::RuntimeOwner,
    runtime::{
        Runtime,
        control::{ControlAction, ControlRequest, ControlResponse},
    },
};
use anyhow::{Context, Result, ensure};
use std::{
    os::unix::fs::OpenOptionsExt,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::BufReader,
    sync::{Notify, Semaphore},
    task::JoinSet,
};

pub const MAX_CONNECTIONS: usize = 64;
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

/// Exit code of a detached service whose owner lock is held by another owner.
pub const EXIT_OWNER_BUSY: u8 = 75;
/// Exit code of a detached service whose `setsid` failed; nothing was opened.
pub const EXIT_DETACH_FAILED: u8 = 71;

/// The owner lock was held by another process; no epoch was begun.
#[derive(Debug)]
struct OwnerBusy;
impl std::fmt::Display for OwnerBusy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("owner busy: another Runtime owns this state root")
    }
}
impl std::error::Error for OwnerBusy {}

/// The one-line readiness announcement written by a detached service to the
/// parent-owned stdout pipe after its endpoint is bound.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Readiness {
    pub instance: String,
    pub epoch: u64,
}

/// Detach from the invoker's session. This is the detached service's first
/// action, before configuration, owner or endpoint. A process group leader
/// cannot `setsid`; the spawner never makes the child one.
pub fn detach() -> Result<()> {
    rustix::process::setsid().context("detach Runtime service session")?;
    Ok(())
}

/// Process exit code for a failed `serve`.
pub fn exit_code(error: &anyhow::Error) -> u8 {
    if error.downcast_ref::<OwnerBusy>().is_some() {
        EXIT_OWNER_BUSY
    } else {
        1
    }
}

fn announce(owner: &RuntimeOwner) -> Result<()> {
    use std::io::Write;
    let line = serde_json::to_string(&Readiness {
        instance: owner.instance_id().into(),
        epoch: owner.epoch(),
    })?;
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{line}").context("readiness announcement")?;
    stdout.flush().context("readiness announcement")?;
    drop(stdout);
    // A later write must never reach the parent's closed pipe.
    let null = std::fs::OpenOptions::new()
        .write(true)
        .custom_flags(rustix::fs::OFlags::CLOEXEC.bits() as i32)
        .open("/dev/null")?;
    rustix::stdio::dup2_stdout(&null).context("retire readiness channel")?;
    Ok(())
}

fn open_owner(state: &Path, config: &Config) -> Result<Arc<RuntimeOwner>> {
    config.validate()?;
    RuntimeOwner::open_mode(state, config.execution.mode)
        .map_err(|error| {
            if error.downcast_ref::<rustix::io::Errno>() == Some(&rustix::io::Errno::WOULDBLOCK) {
                error.context(OwnerBusy)
            } else {
                error
            }
        })
        .context("Open explicit Runtime service owner")
}

pub async fn serve(state: &Path, config: Config, detached: bool) -> Result<()> {
    let environment = std::env::vars_os().collect::<Vec<_>>();
    serve_with_environment(state, config, detached, environment).await
}

// Auxiliary startup refusal only: names cannot establish that an environment is secret-free,
// and changing this process's environment would not erase its initial KERN_PROCARGS2 bytes.
fn check_startup_environment(
    environment: impl IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
) -> Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let mut names = Vec::new();
    for (name, _) in environment {
        let bytes = name.as_bytes();
        let upper = bytes.to_ascii_uppercase();
        if matches!(
            bytes,
            b"GITHUB_TOKEN" | b"GH_TOKEN" | b"GH_ENTERPRISE_TOKEN" | b"GITHUB_ENTERPRISE_TOKEN"
        ) || upper.starts_with(b"AWS_")
            || [b"TOKEN".as_slice(), b"SECRET", b"PASSWORD", b"API_KEY"]
                .iter()
                .any(|part| upper.windows(part.len()).any(|window| window == *part))
        {
            names.push(name);
        }
    }
    names.sort();
    ensure!(
        names.is_empty(),
        "rrxd auxiliary startup refusal: token-like environment variable names: {names:?}"
    );
    Ok(())
}

pub(super) async fn serve_with_environment(
    state: &Path,
    config: Config,
    detached: bool,
    environment: impl IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
) -> Result<()> {
    check_startup_environment(environment)?;
    let pipeline = config.execution.mode == crate::config::ExecutionMode::CliPipeline;
    let owner = open_owner(state, &config)?;
    let runtime =
        Arc::new(Runtime::new(owner.clone(), config).context("Construct Runtime service")?);
    if pipeline {
        runtime
            .start()
            .await
            .context("Qualify pipeline Runtime service")?;
    }
    let endpoint =
        ControlEndpoint::bind(owner.clone()).context("Bind private Runtime control endpoint")?;
    if detached {
        // Before accepting any connection: the parent ties this line to its
        // own child, then separately to the live identity.
        announce(&owner)?;
    }
    drop(owner);
    if !pipeline {
        runtime.start().await.context("Start Runtime service")?;
    }
    let stop = Arc::new(Notify::new());
    let stop_failed = Arc::new(AtomicBool::new(false));
    let permits = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    let mut connections = JoinSet::new();
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let result = loop {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => break signal.context("Runtime signal unavailable"),
            _ = terminate.recv() => break Ok(()),
            () = stop.notified() => break Ok(()),
            Some(_) = connections.join_next(), if !connections.is_empty() => {},
            accepted = endpoint.accept_peer() => {
                let Ok(accepted) = accepted else {
                    // Individual peer/Hello failures cannot retire the service.
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                };
                while connections.try_join_next().is_some() {}
                if connections.len() >= MAX_CONNECTIONS {
                    drop(accepted);
                    continue;
                }
                let Ok(permit) = permits.clone().try_acquire_owned() else {
                    // No control, queue or detached worker is admitted on overload.
                    drop(accepted);
                    continue;
                };
                let runtime = runtime.clone();
                let stop = stop.clone();
                let stop_failed = stop_failed.clone();
                connections.spawn(async move {
                    let _permit = permit;
                    let mut reader = BufReader::new(accepted);
                    if transport::send(reader.get_mut(), &transport::Hello {
                        protocol: transport::PROTOCOL_VERSION,
                        identity: runtime.control_identity(),
                    }, transport::RESPONSE_BYTES).await.is_err() { return; }
                    let request = transport::receive::<ControlRequest>(&mut reader, transport::REQUEST_BYTES).await;
                    let (response, stopping) = match request {
                        Ok(request) => {
                            let id = request.request_id;
                            let is_stop = matches!(request.action, ControlAction::RuntimeStop);
                            let response = runtime.handle_control(reader.get_ref().stream(), request).await;
                            let stopping = is_stop && runtime.is_stopping();
                            let pending = matches!(response, Ok(ControlResponse::RuntimeStopPending { .. }));
                            if is_stop
                                && (stopping || pending)
                                && !matches!(response, Ok(ControlResponse::RuntimeStopped { .. }))
                            {
                                // A repeated shutdown may retire the original pending or
                                // failed stop; retain it so the process cannot claim success.
                                stop_failed.store(true, Ordering::SeqCst);
                            }
                            (response.unwrap_or(ControlResponse::Rejected { request_id: Some(id) }), stopping)
                        }
                        Err(_) => (ControlResponse::Rejected { request_id: None }, false),
                    };
                    // A stop acknowledgement is emitted only by successful Runtime shutdown.
                    let _ = transport::send(reader.get_mut(), &response, transport::RESPONSE_BYTES).await;
                    if stopping { stop.notify_one(); }
                });
            }
        }
    };
    // Stop accepting before waiting for other connections; do not self-join.
    // Already committed controls are not rolled back by a cancelled socket wait.
    let shutdown = runtime.shutdown().await;
    let drained = tokio::time::timeout(DRAIN_TIMEOUT, async {
        while connections.join_next().await.is_some() {}
    })
    .await;
    if drained.is_err() {
        connections.abort_all();
        tokio::time::timeout(DRAIN_TIMEOUT, async {
            while connections.join_next().await.is_some() {}
        })
        .await
        .context("Runtime cancelled connection shutdown remains pending")?;
    }
    drop(endpoint); // Removes only the descriptor belonging to this retained owner.
    result?;
    shutdown?;
    ensure!(
        !stop_failed.load(Ordering::SeqCst),
        "Runtime control shutdown failed"
    );
    Ok(())
}

#[cfg(test)]
mod startup_tests {
    use super::*;
    use crate::config::ExecutionMode;

    /// Both daemon modes refuse every token-like name before opening state, without values.
    #[tokio::test]
    async fn b6_f3_startup_refusal() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join("unopened/state.db");
        for mode in [ExecutionMode::CliPipeline, ExecutionMode::ManagedNative] {
            for name in [
                "GITHUB_TOKEN",
                "GH_TOKEN",
                "GH_ENTERPRISE_TOKEN",
                "GITHUB_ENTERPRISE_TOKEN",
                "AWS_REGION",
                "aws_access_key_id",
                "AWS_",
                "prefixToKeNsuffix",
                "prefixSeCrEtsuffix",
                "prefixPaSsWoRdsuffix",
                "prefixApI_kEysuffix",
            ] {
                let mut config = Config::default();
                config.execution.mode = mode;
                let error = serve_with_environment(
                    &state,
                    config,
                    false,
                    [(name.into(), "fixture-value-never-echo".into())],
                )
                .await
                .unwrap_err()
                .to_string();
                assert!(error.contains(name), "{error}");
                assert!(!error.contains("fixture-value-never-echo"));
                assert!(!state.parent().unwrap().exists());
            }
        }
        let error = check_startup_environment([
            ("AWS_REGION".into(), "first-value".into()),
            ("custom_token".into(), "second-value".into()),
        ])
        .unwrap_err()
        .to_string();
        assert!(error.contains("AWS_REGION") && error.contains("custom_token"));
        assert!(!error.contains("first-value") && !error.contains("second-value"));
    }

    /// A clean allowlisted daemon environment is accepted without examining its values.
    #[test]
    fn b6_f3_startup_clean() {
        check_startup_environment([
            ("HOME".into(), "/fixture/home".into()),
            ("PATH".into(), "/usr/bin:/bin".into()),
            ("TMPDIR".into(), "/fixture/tmp".into()),
            ("LANG".into(), "en_US.UTF-8".into()),
        ])
        .unwrap();
    }
}
