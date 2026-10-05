//! A private native server process. Runtime scope/prepared-input checks belong to
//! the AgentAdapter before launch; this transport never adopts an existing daemon.
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use std::time::Duration;

use tokio::{
    io::{AsyncRead, AsyncReadExt},
    net::UnixStream,
    process::Command,
    task::JoinHandle,
};

use super::{
    availability::{Availability, Site},
    policy::DecisionPolicy,
    preparation::Preparation,
    protocol::{NativeRpc, failure},
};
use crate::adapter::{AdapterResult, ErrorKind, ProcessGroup, cleanup_group};

const START_TIMEOUT: Duration = Duration::from_secs(15);
const CLEANUP_TIMEOUT: Duration = Duration::from_millis(250);
const TAIL_LIMIT: usize = 64 * 1024;

#[derive(Debug, Default, Clone)]
pub(super) struct OutputTail {
    pub(super) bytes: Vec<u8>,
    pub(super) truncated: bool,
}
struct Reader {
    task: JoinHandle<()>,
    tail: Arc<Mutex<OutputTail>>,
}
impl Reader {
    fn spawn(mut stream: impl AsyncRead + Unpin + Send + 'static) -> Self {
        let tail = Arc::new(Mutex::new(OutputTail::default()));
        let target = tail.clone();
        let task = tokio::spawn(async move {
            let mut buffer = [0; 8192];
            loop {
                let length = match stream.read(&mut buffer).await {
                    Ok(0) | Err(_) => break,
                    Ok(length) => length,
                };
                let Ok(mut tail) = target.lock() else {
                    break;
                };
                let excess = tail
                    .bytes
                    .len()
                    .saturating_add(length)
                    .saturating_sub(TAIL_LIMIT);
                if excess > 0 {
                    tail.truncated = true;
                    let removed = excess.min(tail.bytes.len());
                    tail.bytes.drain(..removed);
                }
                tail.bytes.extend_from_slice(&buffer[..length]);
            }
        });
        Self { task, tail }
    }
    fn snapshot(&self) -> OutputTail {
        self.tail
            .lock()
            .map(|tail| tail.clone())
            .unwrap_or_default()
    }
}
impl Drop for Reader {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub(super) struct NativeServer {
    pub(super) rpc: NativeRpc,
    process: ProcessGroup,
    directory: tempfile::TempDir,
    socket: PathBuf,
    socket_binding: SocketBinding,
    stdout: Reader,
    stderr: Reader,
    pid: u32,
}
impl NativeServer {
    /// Test transport for the actual supervisor: real owned child/group cleanup,
    /// private socket identity and reader lifecycle, with a synthetic RPC peer.
    /// This is not a native launch/peer-authentication conformance fixture.
    #[cfg(test)]
    pub(super) fn supervisor_fixture(
        rpc: NativeRpc,
        uncertain: Arc<AtomicBool>,
    ) -> AdapterResult<Self> {
        rpc.require_dispatch()?;
        let directory = tempfile::Builder::new()
            .prefix("rrx-test-")
            .tempdir_in("/tmp")
            .map_err(|error| failure(ErrorKind::LaunchFailure, error.to_string()))?;
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
            .map_err(|error| failure(ErrorKind::LaunchFailure, error.to_string()))?;
        let socket = directory.path().canonicalize().unwrap().join("s");
        let listener = std::os::unix::net::UnixListener::bind(&socket)
            .map_err(|error| failure(ErrorKind::LaunchFailure, error.to_string()))?;
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600))
            .map_err(|error| failure(ErrorKind::LaunchFailure, error.to_string()))?;
        let socket_binding = SocketBinding::capture(&socket)?;
        drop(listener);
        let child = Command::new("/bin/sleep")
            .arg("30")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .kill_on_drop(true)
            .spawn()
            .map_err(|error| failure(ErrorKind::LaunchFailure, error.to_string()))?;
        let mut process = ProcessGroup::new(child, uncertain)?;
        let pid = process.child.id().expect("owned fixture child");
        let stdout = Reader::spawn(process.child.stdout.take().expect("piped stdout"));
        let stderr = Reader::spawn(process.child.stderr.take().expect("piped stderr"));
        Ok(Self {
            rpc,
            process,
            directory,
            socket,
            socket_binding,
            stdout,
            stderr,
            pid,
        })
    }
    // Keep existing ownership/preparation inputs explicit at this private effect boundary.
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn launch_preparing(
        availability: &Availability,
        executable: &Path,
        workspace: &Path,
        policy: Option<&DecisionPolicy>,
        environment: super::environment::ExecEnvironment<'_>,
        mut before_spawn: Option<super::environment::SpawnBoundary<'_>>,
        uncertain: Arc<AtomicBool>,
        preparation: &Preparation,
    ) -> AdapterResult<Self> {
        availability.require()?;
        availability.record(Site::Transport);
        preparation.check()?;
        if !executable.is_absolute() || !workspace.is_absolute() || workspace.to_str().is_none() {
            return Err(failure(
                ErrorKind::InvalidConfiguration,
                "native executable/CWD must be absolute; CWD must be UTF-8",
            ));
        }
        // macOS's default TMPDIR can exceed Unix address limits; /tmp is the
        // OS temporary namespace, and tempfile atomically creates an owned dir.
        let directory = tempfile::Builder::new()
            .prefix("rrx-")
            .tempdir_in("/tmp")
            .map_err(|_| {
                failure(
                    ErrorKind::LaunchFailure,
                    "private native IPC directory unavailable",
                )
            })?;
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
            .map_err(|_| {
                failure(
                    ErrorKind::LaunchFailure,
                    "cannot protect native IPC directory",
                )
            })?;
        let socket = directory
            .path()
            .canonicalize()
            .map_err(|_| failure(ErrorKind::LaunchFailure, "invalid native IPC directory"))?
            .join("s");
        if socket.as_os_str().len() >= 100 {
            return Err(failure(
                ErrorKind::LaunchFailure,
                "private native socket path exceeds limit",
            ));
        }
        let mut command = Command::new(executable);
        if let Some(policy) = policy {
            command.args(policy.arguments());
        }
        command
            .args(["app-server", "--listen"])
            .arg(format!("unix://{}", socket.display()))
            .current_dir(workspace)
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .process_group(0);
        environment.apply(&mut command);
        if !environment.selected() {
            before_spawn = None;
        }
        if let Some(boundary) = before_spawn.as_mut() {
            boundary.admit()?;
        }
        preparation.check()?;
        if let Some(boundary) = &before_spawn {
            boundary.spawn_attempt();
        }
        let child = command.spawn().map_err(|_| {
            preparation.failed(failure(
                ErrorKind::LaunchFailure,
                "native Codex app-server could not start",
            ))
        })?;
        if let Some(boundary) = &before_spawn {
            boundary.spawned();
        }
        let mut process =
            ProcessGroup::new(child, uncertain).map_err(|error| preparation.failed(error))?;
        let pid = process.child.id().expect("validated owned child PID");
        let stdout = Reader::spawn(process.child.stdout.take().expect("piped stdout"));
        let stderr = Reader::spawn(process.child.stderr.take().expect("piped stderr"));
        let connected = preparation.wait(async { tokio::select! {
            result = tokio::time::timeout(START_TIMEOUT, connect_socket(&socket, pid)) => match result {
                Ok(result) => result,
                Err(_) => Err(failure(ErrorKind::Timeout, "private native listener startup timed out")),
            },
            result = process.observe_exit() => {
                let _ = result;
                Err(failure(ErrorKind::LaunchFailure, "native Codex exited before private IPC initialization"))
            },
        }}).await;
        let (stream, socket_binding) = match connected {
            Ok(connection) => connection,
            Err(error) => {
                return Err(failure_after_cleanup(
                    error,
                    terminate(process).await.map(|_| ()),
                ));
            }
        };
        let initialized = preparation
            .wait(async {
                let mut rpc = NativeRpc::connect(stream, availability).await?;
                rpc.initialize().await?;
                Ok::<_, crate::adapter::AdapterError>(rpc)
            })
            .await;
        let rpc = match initialized {
            Ok(rpc) => rpc,
            Err(error) => {
                let cleanup = terminate(process)
                    .await
                    .and_then(|_| socket_binding.cleanup(&socket));
                return Err(failure_after_cleanup(error, cleanup));
            }
        };
        Ok(Self {
            rpc,
            process,
            directory,
            socket,
            socket_binding,
            stdout,
            stderr,
            pid,
        })
    }
    pub(super) fn pid(&self) -> u32 {
        self.pid
    }
    pub(super) fn stderr(&self) -> OutputTail {
        self.stderr.snapshot()
    }
    pub(super) async fn receive(&mut self) -> AdapterResult<super::protocol::Event> {
        tokio::select! {
            event = self.rpc.receive() => event,
            _ = self.process.observe_exit() => Err(failure(ErrorKind::ProcessFailure, "native server exited during its turn")),
        }
    }
    pub(super) async fn shutdown(self) -> AdapterResult<ExitStatus> {
        // Drop IPC only after group cleanup has run on a blocking worker and the
        // still-owned leader has been reaped. Uncertainty remains armed on error.
        let Self {
            rpc,
            process,
            directory,
            socket,
            socket_binding,
            stdout,
            stderr,
            ..
        } = self;
        drop(rpc);
        let result = terminate(process).await;
        let cleanup = if result.is_ok() {
            socket_binding.cleanup(&socket)
        } else {
            Ok(())
        };
        drop(stdout);
        drop(stderr);
        drop(directory);
        let result = result?;
        cleanup?;
        Ok(result)
    }
}
pub(super) fn failure_after_cleanup(
    primary: crate::adapter::AdapterError,
    cleanup: AdapterResult<()>,
) -> crate::adapter::AdapterError {
    match cleanup {
        Ok(()) => primary,
        Err(cleanup) => failure(
            ErrorKind::SessionLost,
            format!("owned cleanup unverified: {cleanup}; original failure: {primary}"),
        ),
    }
}
pub(super) fn result_after_cleanup<T>(
    result: AdapterResult<T>,
    cleanup: AdapterResult<()>,
) -> AdapterResult<T> {
    match result {
        Ok(value) => cleanup.map(|()| value),
        Err(primary) => Err(failure_after_cleanup(primary, cleanup)),
    }
}
async fn terminate(process: ProcessGroup) -> AdapterResult<ExitStatus> {
    let mut process = cleanup_group(process).await?;
    tokio::time::timeout(CLEANUP_TIMEOUT, process.reap())
        .await
        .map_err(|_| {
            failure(
                ErrorKind::SessionLost,
                "native group cleanup completed but owned child death is unconfirmed",
            )
        })?
        .map_err(|_| failure(ErrorKind::SessionLost, "native child could not be reaped"))
}
fn verify_directory(path: &Path) -> AdapterResult<()> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| {
        failure(
            ErrorKind::OwnershipMismatch,
            "owned native IPC directory disappeared",
        )
    })?;
    if !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o777 != 0o700
    {
        return Err(failure(
            ErrorKind::OwnershipMismatch,
            "native IPC directory is not private and owned",
        ));
    }
    Ok(())
}
fn verify_socket(path: &Path) -> AdapterResult<(u64, u64)> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| failure(ErrorKind::OwnershipMismatch, "native socket disappeared"))?;
    if !metadata.file_type().is_socket()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o777 != 0o600
    {
        return Err(failure(
            ErrorKind::OwnershipMismatch,
            "native socket is not private and owned",
        ));
    }
    Ok((metadata.dev(), metadata.ino()))
}
#[derive(Debug)]
struct SocketBinding {
    alias_identity: (u64, u64),
    target: PathBuf,
    target_identity: (u64, u64),
}
impl SocketBinding {
    fn ready(path: &Path) -> AdapterResult<Option<Self>> {
        let metadata = std::fs::symlink_metadata(path).map_err(|_| {
            failure(
                ErrorKind::OwnershipMismatch,
                "native alias metadata unavailable",
            )
        })?;
        if metadata.uid() != rustix::process::geteuid().as_raw() {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "native alias owner mismatch",
            ));
        }
        if metadata.file_type().is_symlink() {
            match path.canonicalize() {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(_) => {
                    return Err(failure(
                        ErrorKind::OwnershipMismatch,
                        "native alias target metadata unavailable",
                    ));
                }
                Ok(_) => {}
            }
        }
        Self::capture(path).map(Some)
    }
    fn capture(path: &Path) -> AdapterResult<Self> {
        let metadata = std::fs::symlink_metadata(path).map_err(|_| {
            failure(
                ErrorKind::OwnershipMismatch,
                "native socket alias disappeared",
            )
        })?;
        if metadata.uid() != rustix::process::geteuid().as_raw() {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "native socket alias owner changed",
            ));
        }
        let target = if metadata.file_type().is_symlink() {
            // Codex 0.160 creates a private alias to its short hashed native socket
            // in an owner-only codex-daemon directory. Do not chmod that target or
            // assume its ownership from its name; authenticate the connected peer.
            let target = path.canonicalize().map_err(|_| {
                failure(
                    ErrorKind::OwnershipMismatch,
                    "native socket alias target is unavailable",
                )
            })?;
            let temporary = Path::new("/tmp").canonicalize().map_err(|_| {
                failure(
                    ErrorKind::OwnershipMismatch,
                    "OS temporary namespace unavailable",
                )
            })?;
            if !target.starts_with(temporary) {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "native socket alias target is outside OS temporary namespace",
                ));
            }
            verify_directory(target.parent().ok_or_else(|| {
                failure(
                    ErrorKind::OwnershipMismatch,
                    "native socket target has no private parent",
                )
            })?)?;
            target
        } else if metadata.file_type().is_socket() {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(
                |_| {
                    failure(
                        ErrorKind::OwnershipMismatch,
                        "cannot protect owned direct native socket",
                    )
                },
            )?;
            path.to_path_buf()
        } else {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "unexpected object at owned native socket",
            ));
        };
        let target_identity = verify_socket(&target)?;
        Ok(Self {
            alias_identity: (metadata.dev(), metadata.ino()),
            target,
            target_identity,
        })
    }
    fn verify(&self, path: &Path) -> AdapterResult<()> {
        let metadata = std::fs::symlink_metadata(path).map_err(|_| {
            failure(
                ErrorKind::OwnershipMismatch,
                "owned native socket alias disappeared",
            )
        })?;
        if (metadata.dev(), metadata.ino()) != self.alias_identity
            || path.canonicalize().ok().as_ref() != Some(&self.target)
            || verify_socket(&self.target)? != self.target_identity
        {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "owned native socket alias or target was replaced",
            ));
        }
        verify_directory(self.target.parent().expect("validated target parent"))
    }
    fn cleanup(&self, path: &Path) -> AdapterResult<()> {
        // Only a binding whose peer was authenticated to the owned group reaches
        // this method, and only after confirmed group termination. Leave the
        // shared native parent and every unrelated socket untouched.
        if self.target != path {
            match std::fs::symlink_metadata(&self.target) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                Err(_) => {
                    return Err(failure(
                        ErrorKind::ProcessFailure,
                        "owned native IPC cleanup metadata unavailable",
                    ));
                }
                Ok(_) => {}
            }
            verify_directory(self.target.parent().expect("validated target parent"))?;
            if verify_socket(&self.target)? != self.target_identity {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "native socket target changed before cleanup",
                ));
            }
            std::fs::remove_file(&self.target).map_err(|_| {
                failure(
                    ErrorKind::ProcessFailure,
                    "verified owned native socket cleanup failed",
                )
            })?;
        }
        Ok(())
    }
}
fn verify_peer(stream: &UnixStream, leader: u32) -> AdapterResult<()> {
    let peer = stream.peer_cred().map_err(|_| {
        failure(
            ErrorKind::UnsupportedCapability,
            "native Unix peer credentials unavailable",
        )
    })?;
    let pid = peer
        .pid()
        .and_then(rustix::process::Pid::from_raw)
        .ok_or_else(|| {
            failure(
                ErrorKind::UnsupportedCapability,
                "native Unix peer PID unavailable",
            )
        })?;
    let group = rustix::process::getpgid(Some(pid)).map_err(|_| {
        failure(
            ErrorKind::OwnershipMismatch,
            "native peer process group unavailable",
        )
    })?;
    if peer.uid() != rustix::process::geteuid().as_raw() || group.as_raw_pid() != leader as i32 {
        return Err(failure(
            ErrorKind::OwnershipMismatch,
            "native Unix peer does not belong to the unreaped owned process group",
        ));
    }
    Ok(())
}
async fn connect_socket(path: &Path, leader: u32) -> AdapterResult<(UnixStream, SocketBinding)> {
    verify_directory(path.parent().expect("owned socket parent"))?;
    loop {
        match std::fs::symlink_metadata(path) {
            Ok(_) => {
                let Some(binding) = SocketBinding::ready(path)? else {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    continue;
                };
                match UnixStream::connect(&binding.target).await {
                    Ok(stream) => {
                        verify_peer(&stream, leader)?;
                        binding.verify(path)?;
                        return Ok((stream, binding));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {}
                    Err(_) => {
                        return Err(failure(
                            ErrorKind::ProcessFailure,
                            "cannot connect to private native socket",
                        ));
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "native socket metadata unavailable",
                ));
            }
        }
        // Bounded startup readiness only; ordinary sessions wait on native events.
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dangling_owned_alias_waits_only_until_target_exists_and_keeps_privacy_checks() {
        let directory = tempfile::Builder::new().tempdir_in("/tmp").unwrap();
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let target = directory.path().canonicalize().unwrap().join("target");
        let alias = directory.path().join("alias");
        std::os::unix::fs::symlink(&target, &alias).unwrap();
        assert!(SocketBinding::ready(&alias).unwrap().is_none());
        let listener = std::os::unix::net::UnixListener::bind(&target).unwrap();
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o666)).unwrap();
        assert_eq!(
            SocketBinding::ready(&alias).unwrap_err().kind,
            ErrorKind::OwnershipMismatch
        );
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o600)).unwrap();
        let binding = SocketBinding::ready(&alias).unwrap().unwrap();
        binding.verify(&alias).unwrap();
        drop(listener);
    }
    #[test]
    fn cleanup_uncertainty_retains_original_sanitized_failure_without_changing_confirmed_errors() {
        let failed = failure_after_cleanup(
            failure(ErrorKind::Timeout, "native startup timed out"),
            Err(failure(
                ErrorKind::SessionLost,
                "native group death unverified",
            )),
        );
        assert_eq!(failed.kind, ErrorKind::SessionLost);
        assert!(
            failed.message.contains("native startup timed out")
                && failed.message.contains("native group death unverified")
        );
        let confirmed = failure_after_cleanup(
            failure(ErrorKind::Timeout, "native startup timed out"),
            Ok(()),
        );
        assert_eq!(confirmed.kind, ErrorKind::Timeout);
        assert_eq!(confirmed.message, "native startup timed out");
    }
    #[test]
    fn successful_discovery_cannot_survive_uncertain_shutdown_and_failed_setup_keeps_both_causes() {
        let successful = result_after_cleanup(
            Ok("discovered native profile"),
            Err(failure(
                ErrorKind::SessionLost,
                "discovery child still unverified",
            )),
        );
        assert_eq!(successful.unwrap_err().kind, ErrorKind::SessionLost);
        let failed: AdapterResult<()> = result_after_cleanup(
            Err(failure(
                ErrorKind::AuthenticationUnavailable,
                "native login required",
            )),
            Err(failure(
                ErrorKind::SessionLost,
                "native group death unverified",
            )),
        );
        let failed = failed.unwrap_err();
        assert_eq!(failed.kind, ErrorKind::SessionLost);
        assert!(failed.message.contains("native login required"));
        assert!(failed.message.contains("native group death unverified"));
        let confirmed: AdapterResult<()> = result_after_cleanup(
            Err(failure(
                ErrorKind::AuthenticationUnavailable,
                "native login required",
            )),
            Ok(()),
        );
        assert_eq!(
            confirmed.unwrap_err().kind,
            ErrorKind::AuthenticationUnavailable
        );
        assert_eq!(
            result_after_cleanup(Ok("native profile"), Ok(())).unwrap(),
            "native profile"
        );
    }
    #[test]
    fn ipc_rejects_symlinks_non_sockets_and_permissive_objects() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        verify_directory(directory.path()).unwrap();
        let socket = directory.path().join("socket");
        let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        verify_socket(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o666)).unwrap();
        assert!(verify_socket(&socket).is_err());
        std::fs::remove_file(&socket).unwrap();
        std::fs::write(&socket, b"not a socket").unwrap();
        assert!(verify_socket(&socket).is_err());
        let link = directory.path().join("link");
        std::os::unix::fs::symlink(&socket, &link).unwrap();
        assert!(verify_socket(&link).is_err());
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(verify_directory(directory.path()).is_err());
        drop(listener);
    }
    #[tokio::test]
    async fn cancelled_bootstrap_cleans_and_reaps_owned_process_before_returning() {
        let directory = tempfile::tempdir().unwrap();
        let executable = directory.path().join("native-fixture");
        let ready = directory.path().join("ready");
        std::fs::write(
            &executable,
            "#!/bin/sh\nprintf '%s' \"$$\" > \"$RRX_FIXTURE_READY\"\nexec /bin/sleep 30\n",
        )
        .unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        let preparation = Arc::new(Preparation::new());
        let uncertain = Arc::new(AtomicBool::new(false));
        let mut action = {
            let preparation = preparation.clone();
            let uncertain = uncertain.clone();
            let cwd = directory.path().to_path_buf();
            let ready = ready.clone();
            tokio::spawn(async move {
                NativeServer::launch_preparing(
                    &crate::codex::availability::component_availability(),
                    &executable,
                    &cwd,
                    None,
                    super::super::environment::ExecEnvironment::Ambient(vec![(
                        "RRX_FIXTURE_READY".into(),
                        ready.into_os_string(),
                    )]),
                    None,
                    uncertain,
                    &preparation,
                )
                .await
            })
        };
        tokio::time::timeout(Duration::from_secs(3), async {
            while !ready.exists() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        assert!(uncertain.load(std::sync::atomic::Ordering::SeqCst));
        let cancelled_at = tokio::time::Instant::now();
        preparation.cancel();
        let result = tokio::time::timeout(Duration::from_secs(17), &mut action)
            .await
            .unwrap()
            .unwrap();
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("cancelled listener startup returned a server"),
        };
        assert_eq!(error.kind, ErrorKind::StateConflict);
        assert_eq!(
            error.message,
            "native preparation cancelled before admission"
        );
        assert!(!uncertain.load(std::sync::atomic::Ordering::SeqCst));
        let pid = std::fs::read_to_string(ready).unwrap();
        let observed = std::process::Command::new("/bin/ps")
            .args(["-p", &pid, "-o", "pid="])
            .output()
            .unwrap();
        assert!(
            observed.stdout.is_empty(),
            "bootstrap must reap its owned leader"
        );
        assert!(
            cancelled_at.elapsed() < Duration::from_secs(2),
            "cancellation must wake bootstrap before its original listener deadline"
        );
    }

    #[tokio::test]
    async fn output_is_drained_without_unbounded_retention() {
        use tokio::io::AsyncWriteExt;
        let (reader, mut writer) = tokio::io::duplex(8192);
        let stream = Reader::spawn(reader);
        writer.write_all(&vec![b'a'; TAIL_LIMIT * 2]).await.unwrap();
        writer.write_all(b"END").await.unwrap();
        writer.shutdown().await.unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            while !stream.task.is_finished() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let tail = stream.snapshot();
        assert_eq!(tail.bytes.len(), TAIL_LIMIT);
        assert!(tail.truncated && tail.bytes.ends_with(b"END"));
    }
    #[tokio::test]
    async fn private_alias_does_not_authorize_a_foreign_daemon_peer() {
        let directory = tempfile::Builder::new()
            .prefix("rrx-peer-test-")
            .tempdir_in("/tmp")
            .unwrap();
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let directory_path = directory.path().canonicalize().unwrap();
        let target = directory_path.join("actual");
        let listener = tokio::net::UnixListener::bind(&target).unwrap();
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o600)).unwrap();
        let alias = directory_path.join("alias");
        std::os::unix::fs::symlink(&target, &alias).unwrap();
        let binding = SocketBinding::capture(&alias).unwrap();
        binding.verify(&alias).unwrap();
        let client = UnixStream::connect(&target).await.unwrap();
        let (peer, _) = listener.accept().await.unwrap();
        let own_group = rustix::process::getpgid(None).unwrap().as_raw_pid() as u32;
        verify_peer(&client, own_group).unwrap();
        assert_eq!(
            verify_peer(&client, own_group.wrapping_add(10000))
                .unwrap_err()
                .kind,
            ErrorKind::OwnershipMismatch
        );
        std::fs::remove_file(&alias).unwrap();
        std::os::unix::fs::symlink(directory_path.join("foreign"), &alias).unwrap();
        assert!(binding.verify(&alias).is_err());
        drop(peer);
    }
    #[test]
    fn cleanup_preserves_other_native_sockets_and_rejects_replaced_targets() {
        for replace in [false, true] {
            let directory = tempfile::Builder::new()
                .prefix("rrx-ipc-cleanup-")
                .tempdir_in("/tmp")
                .unwrap();
            std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
                .unwrap();
            let parent = directory.path().canonicalize().unwrap();
            let target = parent.join("owned");
            let foreign = parent.join("foreign");
            let alias = parent.join("alias");
            let owned_listener = std::os::unix::net::UnixListener::bind(&target).unwrap();
            let foreign_listener = std::os::unix::net::UnixListener::bind(&foreign).unwrap();
            for path in [&target, &foreign] {
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
            }
            std::os::unix::fs::symlink(&target, &alias).unwrap();
            let binding = SocketBinding::capture(&alias).unwrap();
            if replace {
                std::fs::remove_file(&target).unwrap();
                std::fs::rename(&foreign, &target).unwrap();
                assert_eq!(
                    binding.cleanup(&alias).unwrap_err().kind,
                    ErrorKind::OwnershipMismatch
                );
                assert!(target.exists());
            } else {
                binding.cleanup(&alias).unwrap();
                assert!(!target.exists() && foreign.exists() && parent.is_dir());
            }
            drop(owned_listener);
            drop(foreign_listener);
        }
    }
}
