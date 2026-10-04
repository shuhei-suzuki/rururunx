//! Private process group with bounded wire queues and kill-before-reap ownership.
use super::protocol::{LINE_LIMIT, failure};
use crate::adapter::{AdapterResult, ErrorKind, ProcessGroup, cleanup_group};
use serde_json::Value;
use std::{
    collections::VecDeque,
    ffi::OsString,
    path::Path,
    process::Stdio,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::{ChildStdin, Command},
    sync::mpsc,
    task::JoinHandle,
};

pub(super) struct Transport {
    group: Option<ProcessGroup>,
    stdin: Option<ChildStdin>,
    pub frames: mpsc::Receiver<AdapterResult<Value>>,
    pub diagnostics: mpsc::Receiver<Vec<u8>>,
    pub pending: VecDeque<AdapterResult<Value>>,
    pub startup_diagnostics: Vec<u8>,
    pub startup_diagnostics_truncated: bool,
    readers: Vec<JoinHandle<()>>,
}
impl Transport {
    pub fn launch(
        executable: &Path,
        cwd: &Path,
        args: &[String],
        environment: Vec<(OsString, OsString)>,
        flag: Arc<AtomicBool>,
    ) -> AdapterResult<Self> {
        let mut command = Command::new(executable);
        command
            .args(args)
            .current_dir(cwd)
            .env_clear()
            .envs(environment)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        let child = command
            .spawn()
            .map_err(|_| failure(ErrorKind::LaunchFailure, "native Claude launch failed"))?;
        let mut group = ProcessGroup::new(child, flag)?;
        let stdin = group.child.stdin.take().expect("piped native stdin");
        let stdout = group.child.stdout.take().expect("piped native stdout");
        let stderr = group.child.stderr.take().expect("piped native stderr");
        let (tx, frames) = mpsc::channel(32);
        let (diag, diagnostics) = mpsc::channel(8);
        let readers = vec![
            tokio::spawn(read_frames(stdout, tx)),
            tokio::spawn(read_diagnostics(stderr, diag)),
        ];
        Ok(Self {
            group: Some(group),
            stdin: Some(stdin),
            frames,
            diagnostics,
            pending: VecDeque::new(),
            startup_diagnostics: Vec::new(),
            startup_diagnostics_truncated: false,
            readers,
        })
    }
    pub fn pid(&self) -> u32 {
        self.group
            .as_ref()
            .expect("owned group")
            .child
            .id()
            .expect("unreaped leader")
    }
    pub async fn write(&mut self, value: &Value) -> AdapterResult<()> {
        let bytes = Self::encode(value)?;
        self.write_encoded(&bytes).await
    }
    pub fn encode(value: &Value) -> AdapterResult<Vec<u8>> {
        let mut bytes = serde_json::to_vec(value)
            .map_err(|_| failure(ErrorKind::InvalidInput, "native input serialization failed"))?;
        if bytes.len() > LINE_LIMIT {
            return Err(failure(
                ErrorKind::InvalidInput,
                "native wire input exceeds bound",
            ));
        }
        bytes.push(b'\n');
        Ok(bytes)
    }
    pub async fn write_encoded(&mut self, bytes: &[u8]) -> AdapterResult<()> {
        tokio::time::timeout(
            Duration::from_secs(5),
            self.stdin
                .as_mut()
                .expect("open native stdin")
                .write_all(bytes),
        )
        .await
        .map_err(|_| failure(ErrorKind::Timeout, "native stdin deadline exceeded"))?
        .map_err(|_| failure(ErrorKind::ProcessFailure, "native stdin closed"))
    }
    pub async fn request(&mut self, subtype: &str, id: &str) -> AdapterResult<Value> {
        self.write(&super::protocol::control(subtype, id)).await?;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
        let mut diagnostics_open = true;
        loop {
            tokio::select! {
                _ = tokio::time::sleep_until(deadline) => return Err(failure(ErrorKind::Timeout, "native initialization deadline exceeded")),
                bytes = self.diagnostics.recv(), if diagnostics_open => match bytes {
                    Some(bytes) => {
                        self.startup_diagnostics.extend_from_slice(&bytes);
                        if self.startup_diagnostics.len() > 64 * 1024 {
                            self.startup_diagnostics.drain(..self.startup_diagnostics.len() - 64 * 1024);
                            self.startup_diagnostics_truncated = true;
                        }
                    }
                    None => diagnostics_open = false,
                },
                frame = self.frames.recv() => {
                    let frame = frame.ok_or_else(|| failure(ErrorKind::ProcessFailure, "native closed before control response"))??;
                    if frame["type"] == "control_response" { return super::protocol::control_result(&frame, id); }
                    if !matches!(frame["type"].as_str(), Some("system" | "auth_status" | "rate_limit_event")) || self.pending.len() >= 32 {
                        return Err(failure(ErrorKind::ParseFailure, "unexpected native initialization frame or buffer bound"));
                    }
                    self.pending.push_back(Ok(frame));
                }
            }
        }
    }

    pub async fn finish(mut self) -> AdapterResult<std::process::ExitStatus> {
        // EOF permits native transcript/usage flushing. Observe exit without
        // reaping so the group identity remains owned during final cleanup.
        self.stdin.take();
        if let Some(group) = &self.group {
            let _ = tokio::time::timeout(Duration::from_secs(5), group.observe_exit()).await;
        }
        self.cleanup().await
    }
    pub async fn cleanup(mut self) -> AdapterResult<std::process::ExitStatus> {
        let group = self.group.take().expect("owned group");
        let result = async {
            let mut group = cleanup_group(group).await?;
            tokio::time::timeout(Duration::from_secs(5), group.reap())
                .await
                .map_err(|_| failure(ErrorKind::SessionLost, "native reap timed out"))?
                .map_err(|_| failure(ErrorKind::SessionLost, "native reap failed"))
        }
        .await;
        for reader in self.readers.drain(..) {
            reader.abort();
            let _ = reader.await;
        }
        result
    }
}
impl Drop for Transport {
    fn drop(&mut self) {
        // ProcessGroup Drop owns group cleanup. Readers never own cleanup authority.
        for reader in &self.readers {
            reader.abort();
        }
    }
}
async fn read_frames<R: AsyncRead + Unpin>(mut reader: R, tx: mpsc::Sender<AdapterResult<Value>>) {
    let mut pending = Vec::new();
    let mut chunk = [0; 8192];
    loop {
        let n = match reader.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => {
                let _ = tx
                    .send(Err(failure(
                        ErrorKind::ProcessFailure,
                        "native stdout read failed",
                    )))
                    .await;
                return;
            }
        };
        for byte in &chunk[..n] {
            if *byte == b'\n' {
                if !pending.is_empty() {
                    let message = serde_json::from_slice(&pending).map_err(|_| {
                        failure(ErrorKind::ParseFailure, "malformed native JSON frame")
                    });
                    pending.clear();
                    if tx.send(message).await.is_err() {
                        return;
                    }
                }
            } else {
                pending.push(*byte);
                if pending.len() > LINE_LIMIT {
                    let _ = tx
                        .send(Err(failure(
                            ErrorKind::ParseFailure,
                            "native frame limit exceeded",
                        )))
                        .await;
                    return;
                }
            }
        }
    }
    if !pending.is_empty() {
        let _ = tx
            .send(Err(failure(
                ErrorKind::ParseFailure,
                "incomplete native JSON frame",
            )))
            .await;
    }
}
async fn read_diagnostics<R: AsyncRead + Unpin>(mut reader: R, tx: mpsc::Sender<Vec<u8>>) {
    let mut buf = [0; 4096];
    loop {
        match reader.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                if tx.send(buf[..n].to_vec()).await.is_err() {
                    break;
                }
            }
        }
    }
}
