//! A private native server process. Runtime scope/prepared-input checks belong to
//! the AgentAdapter before launch; this transport never adopts an existing daemon.
use std::ffi::OsString;
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
    policy::DecisionPolicy,
    protocol::{NativeRpc, failure},
};
use crate::adapter::{AdapterResult, ErrorKind, ProcessGroup, cleanup_group};

const START_TIMEOUT: Duration = Duration::from_secs(15);
const CLEANUP_TIMEOUT: Duration = Duration::from_millis(250);
const TAIL_LIMIT: usize = 64 * 1024;

#[derive(Debug, Default, Clone)]
pub struct OutputTail {
    pub bytes: Vec<u8>,
    pub truncated: bool,
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

pub struct NativeServer {
    pub rpc: NativeRpc,
    process: ProcessGroup,
    directory: tempfile::TempDir,
    socket: PathBuf,
    socket_identity: (u64, u64),
    stdout: Reader,
    stderr: Reader,
    pid: u32,
}
impl NativeServer {
    pub async fn launch(
        executable: &Path,
        workspace: &Path,
        policy: Option<&DecisionPolicy>,
        environment: Vec<(OsString, OsString)>,
        uncertain: Arc<AtomicBool>,
    ) -> AdapterResult<Self> {
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
            .envs(environment)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .process_group(0);
        let child = command.spawn().map_err(|_| {
            failure(
                ErrorKind::LaunchFailure,
                "native Codex app-server could not start",
            )
        })?;
        let mut process = ProcessGroup::new(child, uncertain)?;
        let pid = process.child.id().expect("validated owned child PID");
        let stdout = Reader::spawn(process.child.stdout.take().expect("piped stdout"));
        let stderr = Reader::spawn(process.child.stderr.take().expect("piped stderr"));
        let connected = tokio::select! {
            result = tokio::time::timeout(START_TIMEOUT, connect_socket(&socket)) => match result {
                Ok(result) => result,
                Err(_) => Err(failure(ErrorKind::Timeout, "private native listener startup timed out")),
            },
            result = process.observe_exit() => {
                let _ = result;
                Err(failure(ErrorKind::LaunchFailure, "native Codex exited before private IPC initialization"))
            },
        };
        let (stream, socket_identity) = match connected {
            Ok(connection) => connection,
            Err(error) => {
                terminate(process).await?;
                return Err(error);
            }
        };
        let initialized = async {
            let mut rpc = NativeRpc::connect(stream).await?;
            rpc.initialize().await?;
            Ok::<_, crate::adapter::AdapterError>(rpc)
        }
        .await;
        let rpc = match initialized {
            Ok(rpc) => rpc,
            Err(error) => {
                terminate(process).await?;
                return Err(error);
            }
        };
        Ok(Self {
            rpc,
            process,
            directory,
            socket,
            socket_identity,
            stdout,
            stderr,
            pid,
        })
    }
    pub fn pid(&self) -> u32 {
        self.pid
    }
    pub fn socket(&self) -> AdapterResult<&Path> {
        verify_directory(self.directory.path())?;
        if verify_socket(&self.socket)? != self.socket_identity {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "owned native socket was replaced",
            ));
        }
        Ok(&self.socket)
    }
    pub fn stdout(&self) -> OutputTail {
        self.stdout.snapshot()
    }
    pub fn stderr(&self) -> OutputTail {
        self.stderr.snapshot()
    }
    pub async fn observe_exit(&self) -> std::io::Result<()> {
        self.process.observe_exit().await
    }
    pub async fn shutdown(self) -> AdapterResult<ExitStatus> {
        // Drop IPC only after group cleanup has run on a blocking worker and the
        // still-owned leader has been reaped. Uncertainty remains armed on error.
        let Self {
            rpc,
            process,
            directory,
            stdout,
            stderr,
            ..
        } = self;
        drop(rpc);
        let result = terminate(process).await;
        drop(stdout);
        drop(stderr);
        drop(directory);
        result
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
async fn connect_socket(path: &Path) -> AdapterResult<(UnixStream, (u64, u64))> {
    verify_directory(path.parent().expect("owned socket parent"))?;
    loop {
        match std::fs::symlink_metadata(path) {
            Ok(metadata) => {
                if !metadata.file_type().is_socket()
                    || metadata.uid() != rustix::process::geteuid().as_raw()
                {
                    return Err(failure(
                        ErrorKind::OwnershipMismatch,
                        "unexpected object at owned native socket",
                    ));
                }
                // The already owner-only parent prevents other users racing this
                // chmod. The socket itself must also become owner-only before use.
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(
                    |_| failure(ErrorKind::OwnershipMismatch, "cannot protect native socket"),
                )?;
                let identity = verify_socket(path)?;
                match UnixStream::connect(path).await {
                    Ok(stream) => {
                        if verify_socket(path)? != identity {
                            return Err(failure(
                                ErrorKind::OwnershipMismatch,
                                "native socket changed during connection",
                            ));
                        }
                        return Ok((stream, identity));
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
}
