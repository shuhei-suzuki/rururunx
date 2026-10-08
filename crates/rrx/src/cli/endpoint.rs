//! Dedicated local service discovery. Reading never opens a Store or owner epoch.
use super::transport::{self, Hello, ServiceIdentity};
use crate::{execution::RuntimeOwner, execution::strict_json};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::BufReader,
    net::{UnixListener, UnixStream},
};

const DESCRIPTOR_BYTES: usize = 8192;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    protocol: u32,
    identity: ServiceIdentity,
    socket: PathBuf,
}
impl Descriptor {
    fn validate(&self, state: &Path) -> Result<()> {
        self.identity.validate()?;
        ensure!(
            self.protocol == transport::PROTOCOL_VERSION && self.identity.state == state,
            "control descriptor identity mismatch"
        );
        ensure!(
            self.socket.is_absolute() && self.socket.as_os_str().len() < 100,
            "invalid control socket path"
        );
        Ok(())
    }
}

fn owned_directory(path: &Path, private: bool) -> Result<()> {
    let m = path.symlink_metadata()?;
    ensure!(
        m.is_dir()
            && m.uid() == rustix::process::getuid().as_raw()
            && m.mode() & (if private { 0o077 } else { 0o022 }) == 0,
        "control directory ownership or permissions invalid"
    );
    Ok(())
}

fn control_directory(state: &Path) -> Result<PathBuf> {
    let root = state
        .parent()
        .context("state parent missing")?
        .join(format!(
            "{}.execution",
            state
                .file_name()
                .context("state name missing")?
                .to_string_lossy()
        ));
    owned_directory(&root, false)?;
    Ok(root.join("control"))
}

fn private_file(path: &Path, create: bool) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(create)
        .create(create)
        .truncate(false)
        .mode(0o600)
        // Reject special files from the opened descriptor without first
        // blocking on FIFO open. Regular files retain their normal semantics.
        .custom_flags((rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32)
        .open(path)?;
    let m = file.metadata()?;
    ensure!(
        m.is_file()
            && m.nlink() == 1
            && m.uid() == rustix::process::getuid().as_raw()
            && m.mode() & 0o077 == 0,
        "control file ownership or permissions invalid"
    );
    Ok(file)
}

fn read_descriptor(path: &Path, state: &Path) -> Result<Descriptor> {
    let file = private_file(path, false)?;
    ensure!(
        file.metadata()?.len() <= DESCRIPTOR_BYTES as u64,
        "control descriptor limit"
    );
    let mut bytes = Vec::new();
    file.take(DESCRIPTOR_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    let value = strict_json::decode(
        &bytes,
        strict_json::Limits {
            frame_bytes: DESCRIPTOR_BYTES,
            total_string_bytes: DESCRIPTOR_BYTES,
            ..strict_json::Limits::CEILINGS
        },
    )?;
    let descriptor: Descriptor = serde_json::from_value(value)
        .map_err(|_| anyhow::anyhow!("invalid control descriptor shape"))?;
    descriptor.validate(state)?;
    Ok(descriptor)
}

/// Retains the actual owner and a distinct control-publication lock. Neither the
/// descriptor nor a client connection is native/Goal/Driver authority.
pub struct ControlEndpoint {
    listener: UnixListener,
    descriptor: Descriptor,
    descriptor_path: PathBuf,
    _lock: File,
    _socket_directory: tempfile::TempDir,
    owner: Arc<RuntimeOwner>,
}
/// Only the dedicated listener can construct this accepted peer connection.
pub struct AcceptedControlConnection(UnixStream);
impl AcceptedControlConnection {
    pub(crate) fn stream(&self) -> &UnixStream {
        &self.0
    }
}
impl tokio::io::AsyncRead for AcceptedControlConnection {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().0).poll_read(cx, buf)
    }
}
impl tokio::io::AsyncWrite for AcceptedControlConnection {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.get_mut().0).poll_write(cx, buf)
    }
    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().0).poll_flush(cx)
    }
    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().0).poll_shutdown(cx)
    }
}
impl ControlEndpoint {
    pub fn bind(owner: Arc<RuntimeOwner>) -> Result<Self> {
        let directory = control_directory(owner.state_path())?;
        match std::fs::DirBuilder::new().mode(0o700).create(&directory) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.into()),
        }
        owned_directory(&directory, true)?;
        let lock = private_file(&directory.join("endpoint.lock"), true)?;
        rustix::fs::flock(&lock, rustix::fs::FlockOperation::NonBlockingLockExclusive)
            .context("control endpoint already published")?;
        let descriptor_path = directory.join("endpoint.json");
        if descriptor_path.symlink_metadata().is_ok() {
            // A stale descriptor is replaced only by the actual owner, and only
            // after checking that it belongs to this canonical state.
            read_descriptor(&descriptor_path, owner.state_path())?;
        }
        let socket_directory = tempfile::Builder::new()
            .prefix("rrx-control-")
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir_in("/tmp")?;
        let socket = socket_directory.path().canonicalize()?.join("control.sock");
        owned_directory(socket.parent().context("socket parent missing")?, true)?;
        let descriptor = Descriptor {
            protocol: transport::PROTOCOL_VERSION,
            identity: ServiceIdentity {
                state: owner.state_path().into(),
                instance: owner.instance_id().into(),
                epoch: owner.epoch(),
            },
            socket,
        };
        descriptor.validate(owner.state_path())?;
        let listener = UnixListener::bind(&descriptor.socket)?;
        std::fs::set_permissions(&descriptor.socket, std::fs::Permissions::from_mode(0o600))?;
        let bytes = serde_json::to_vec(&descriptor)?;
        ensure!(bytes.len() <= DESCRIPTOR_BYTES, "control descriptor limit");
        let mut temporary = tempfile::NamedTempFile::new_in(&directory)?;
        temporary.write_all(&bytes)?;
        temporary.as_file().sync_all()?;
        temporary.persist(&descriptor_path)?;
        File::open(&directory)?.sync_all()?;
        Ok(Self {
            owner,
            listener,
            descriptor,
            descriptor_path,
            _lock: lock,
            _socket_directory: socket_directory,
        })
    }

    pub async fn accept(&self) -> Result<AcceptedControlConnection> {
        let mut accepted = self.accept_peer().await?;
        transport::send(
            &mut accepted,
            &Hello {
                protocol: self.descriptor.protocol,
                identity: self.descriptor.identity.clone(),
            },
            transport::RESPONSE_BYTES,
        )
        .await?;
        Ok(accepted)
    }
    pub(crate) async fn accept_peer(&self) -> Result<AcceptedControlConnection> {
        let (stream, _) = self.listener.accept().await?;
        ensure!(
            stream.peer_cred()?.uid() == rustix::process::getuid().as_raw(),
            "foreign control peer UID"
        );
        Ok(AcceptedControlConnection(stream))
    }
}
impl Drop for ControlEndpoint {
    fn drop(&mut self) {
        if read_descriptor(&self.descriptor_path, self.owner.state_path())
            .is_ok_and(|d| d == self.descriptor)
        {
            let _ = std::fs::remove_file(&self.descriptor_path);
        }
    }
}

/// Selected-state discovery/handshake is read-only, including failure. A missing
/// service reports an explicit instruction instead of auto-starting an epoch.
pub async fn connect(state: &Path) -> Result<(ServiceIdentity, BufReader<UnixStream>)> {
    connect_inner(state)
        .await
        .context("Runtime service unavailable or discovery refused; start rrx serve explicitly")
}
async fn connect_inner(state: &Path) -> Result<(ServiceIdentity, BufReader<UnixStream>)> {
    let state = state
        .canonicalize()
        .context("state unavailable; start rrx serve explicitly")?;
    let directory = control_directory(&state)?;
    owned_directory(&directory, true)?;
    let descriptor = read_descriptor(&directory.join("endpoint.json"), &state)?;
    owned_directory(
        descriptor
            .socket
            .parent()
            .context("socket parent missing")?,
        true,
    )?;
    let socket_metadata = descriptor.socket.symlink_metadata()?;
    ensure!(
        socket_metadata.file_type().is_socket()
            && socket_metadata.uid() == rustix::process::getuid().as_raw()
            && socket_metadata.mode() & 0o077 == 0,
        "control socket ownership or permissions invalid"
    );
    let stream = tokio::time::timeout(
        Duration::from_secs(30),
        UnixStream::connect(&descriptor.socket),
    )
    .await
    .context("control connection deadline")??;
    ensure!(
        stream.peer_cred()?.uid() == rustix::process::getuid().as_raw(),
        "foreign service UID"
    );
    let mut reader = BufReader::new(stream);
    let hello: Hello = transport::receive(&mut reader, transport::RESPONSE_BYTES).await?;
    hello.validate_for(&descriptor.identity)?;
    Ok((descriptor.identity, reader))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn actual_endpoint_connects_without_epoch_effect_and_refuses_second_publisher() {
        let dir = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        let before = owner.store().lock().unwrap().connection_epoch_for_test();
        let endpoint = ControlEndpoint::bind(owner.clone()).unwrap();
        assert!(ControlEndpoint::bind(owner.clone()).is_err());
        let (accepted, connected) = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::join!(endpoint.accept(), connect(owner.state_path()))
        })
        .await
        .unwrap();
        let _server = accepted.unwrap();
        let (identity, _reader) = connected.unwrap();
        assert_eq!(identity.state, owner.state_path());
        assert_eq!(identity.instance, owner.instance_id());
        assert_eq!(identity.epoch, before);
        assert_eq!(
            owner.store().lock().unwrap().connection_epoch_for_test(),
            before
        );
        let descriptor_path = endpoint.descriptor_path.clone();
        drop(endpoint);
        assert!(!descriptor_path.exists());
        // The same retained owner may publish again only after the first endpoint
        // has gone; no owner reopen or implicit epoch transition is involved.
        let _replacement = ControlEndpoint::bind(owner.clone()).unwrap();
        assert_eq!(
            owner.store().lock().unwrap().connection_epoch_for_test(),
            before
        );
    }

    #[tokio::test]
    async fn discovery_refuses_changed_descriptor_and_actual_hello_without_deleting_it() {
        let dir = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        let endpoint = ControlEndpoint::bind(owner.clone()).unwrap();
        let mut changed = endpoint.descriptor.clone();
        changed.identity.epoch += 1;
        std::fs::write(
            &endpoint.descriptor_path,
            serde_json::to_vec(&changed).unwrap(),
        )
        .unwrap();
        let (accepted, connected) = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::join!(endpoint.accept(), connect(owner.state_path()))
        })
        .await
        .unwrap();
        assert!(accepted.is_ok());
        assert!(
            connected.is_err(),
            "actual service Hello must match discovery pins"
        );
        let descriptor_path = endpoint.descriptor_path.clone();
        drop(endpoint);
        assert!(
            descriptor_path.exists(),
            "Drop must preserve another descriptor identity"
        );
        let mut foreign = changed;
        foreign.identity.state = dir.path().join("foreign.db");
        std::fs::write(&descriptor_path, serde_json::to_vec(&foreign).unwrap()).unwrap();
        assert!(connect(owner.state_path()).await.is_err());
        assert!(ControlEndpoint::bind(owner.clone()).is_err());
        assert_eq!(
            owner.store().lock().unwrap().connection_epoch_for_test(),
            owner.epoch()
        );
    }

    #[tokio::test]
    async fn missing_or_symlinked_discovery_refuses_without_creating_state() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.db");
        assert!(connect(&missing).await.is_err());
        assert!(!missing.exists());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);

        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        let endpoint = ControlEndpoint::bind(owner.clone()).unwrap();
        let path = endpoint.descriptor_path.clone();
        let bytes = std::fs::read(&path).unwrap();
        drop(endpoint);
        let target = dir.path().join("descriptor-target");
        std::fs::write(&target, &bytes).unwrap();
        std::os::unix::fs::symlink(&target, &path).unwrap();
        assert!(connect(owner.state_path()).await.is_err());
        assert!(ControlEndpoint::bind(owner.clone()).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), bytes);
        assert!(path.symlink_metadata().unwrap().file_type().is_symlink());
    }

    #[test]
    fn actual_fifo_descriptor_refuses_connect_drop_and_stale_bind_without_blocking() {
        let dir = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let endpoint = runtime.block_on(async { ControlEndpoint::bind(owner.clone()).unwrap() });
        let path = endpoint.descriptor_path.clone();
        std::fs::remove_file(&path).unwrap();
        // rustix::mkfifoat is not exposed on Apple targets. The standard OS
        // fixture utility creates only this test-owned FIFO, without shell eval.
        assert!(
            std::process::Command::new("/usr/bin/mkfifo")
                .args(["-m", "600"])
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        let (tx, rx) = std::sync::mpsc::channel();
        // The watchdog belongs to the actual caller. A wrong open may block this
        // detached thread, but cannot prevent the test's assertion from failing.
        // Its lifetime does not certify interruption of a filesystem syscall.
        std::thread::spawn(move || {
            let refused = runtime.block_on(connect(owner.state_path())).is_err();
            drop(endpoint);
            let preserved = path.symlink_metadata().unwrap().file_type().is_fifo();
            let stale_refused = runtime.block_on(async { ControlEndpoint::bind(owner).is_err() });
            let _ = tx.send((refused, preserved, stale_refused));
        });
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(2))
                .expect("actual FIFO discovery/Drop/stale bind blocked"),
            (true, true, true)
        );
    }
}
