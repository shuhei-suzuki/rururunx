//! Real owned-child completion; no process census or reusable PID authority.
use super::plan::{STREAM_BYTES, TestCommand, hash};
use crate::execution::process::OwnedProcess;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    os::unix::{fs::OpenOptionsExt, process::ExitStatusExt},
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    io::AsyncReadExt,
    time::{Instant, interval},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureIssue {
    Timeout,
    Cancelled,
    OutputOverflow,
    OutputIo,
    DrainIncomplete,
    ChildObservation,
    ReapUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StreamEvidence {
    pub bytes: u64,
    pub sha256: String,
    pub complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandObservation {
    pub command: String,
    pub operation: crate::execution::OperationId,
    pub started_at: i64,
    pub finished_at: i64,
    pub timeout_seconds: u64,
    pub drain_seconds: u64,
    pub exit: Option<i32>,
    pub signal: Option<i32>,
    pub work_known: bool,
    pub issue: Option<CaptureIssue>,
    pub group_cleanup_unknown: bool,
    pub stdout: StreamEvidence,
    pub stderr: StreamEvidence,
}
impl CommandObservation {
    pub fn certifying(&self) -> bool {
        self.work_known
            && self.exit == Some(0)
            && self.signal.is_none()
            && self.issue.is_none()
            && self.stdout.complete
            && self.stderr.complete
    }
}
pub(crate) struct CollectedCommand {
    observation: CommandObservation,
    pub(super) stdout: Vec<u8>,
    pub(super) stderr: Vec<u8>,
}

impl CollectedCommand {
    pub(crate) fn observation(&self) -> &CommandObservation {
        &self.observation
    }
    pub(crate) fn validate(&self, command: &TestCommand) -> Result<()> {
        self.observation.validate(command)?;
        ensure!(
            self.observation.stdout.bytes == self.stdout.len() as u64
                && self.observation.stderr.bytes == self.stderr.len() as u64
                && self.observation.stdout.sha256 == hash(&self.stdout)
                && self.observation.stderr.sha256 == hash(&self.stderr),
            "verification collector bytes differ from owned observation"
        );
        Ok(())
    }
}

impl CommandObservation {
    pub(crate) fn validate(&self, command: &TestCommand) -> Result<()> {
        ensure!(
            self.command == command.id
                && self.timeout_seconds == command.timeout_seconds
                && self.drain_seconds == command.drain_seconds
                && self.started_at >= 0
                && self.finished_at >= self.started_at
                && self.stdout.bytes <= command.stdout_bytes as u64
                && self.stderr.bytes <= command.stderr_bytes as u64
                && self.exit.is_none_or(|code| code >= 0)
                && self.signal.is_none_or(|signal| signal > 0)
                && !(self.exit.is_some() && self.signal.is_some())
                && (!self.work_known || self.exit.is_some() || self.signal.is_some()),
            "verification command receipt differs from admitted command"
        );
        for stream in [&self.stdout, &self.stderr] {
            ensure!(
                stream.sha256.len() == 64 && stream.sha256.bytes().all(|b| b.is_ascii_hexdigit()),
                "verification stream digest is invalid"
            );
        }
        Ok(())
    }
}

pub(super) async fn collect(
    mut child: OwnedProcess,
    command: &TestCommand,
    operation: crate::execution::OperationId,
    fence: impl Fn() -> Result<()>,
) -> Result<CollectedCommand> {
    let started_at = crate::domain::now_ms();
    let mut out = child
        .child
        .stdout
        .take()
        .context("verification stdout missing")?;
    let mut err = child
        .child
        .stderr
        .take()
        .context("verification stderr missing")?;
    let mut out_bytes = Vec::new();
    let mut err_bytes = Vec::new();
    let mut out_done = false;
    let mut err_done = false;
    let mut known = false;
    let mut issue = None;
    let mut stdout_buffer = [0u8; 8192];
    let mut stderr_buffer = [0u8; 8192];
    let mut deadline = Instant::now() + Duration::from_secs(command.timeout_seconds);
    let mut tick = interval(Duration::from_millis(50));
    loop {
        if known && out_done && err_done {
            break;
        }
        tokio::select! {
            observed=child.exited(), if !known => {
                if observed.is_err() {issue=Some(CaptureIssue::ChildObservation);break}
                known=true;
                let _=child.signal_group();
                deadline=Instant::now()+Duration::from_secs(command.drain_seconds);
            },
            read=out.read(&mut stdout_buffer), if !out_done => {
                match read {
                    Ok(0)=>out_done=true,
                    Ok(n)=> {let left=command.stdout_bytes.saturating_sub(out_bytes.len());out_bytes.extend_from_slice(&stdout_buffer[..n.min(left)]);if n>left {issue=Some(CaptureIssue::OutputOverflow);break}},
                    Err(_)=>{issue=Some(CaptureIssue::OutputIo);break}
                }
            },
            read=err.read(&mut stderr_buffer), if !err_done => {
                match read {
                    Ok(0)=>err_done=true,
                    Ok(n)=> {let left=command.stderr_bytes.saturating_sub(err_bytes.len());err_bytes.extend_from_slice(&stderr_buffer[..n.min(left)]);if n>left {issue=Some(CaptureIssue::OutputOverflow);break}},
                    Err(_)=>{issue=Some(CaptureIssue::OutputIo);break}
                }
            },
            _=tokio::time::sleep_until(deadline)=> {issue=Some(if known {CaptureIssue::DrainIncomplete}else{CaptureIssue::Timeout});break},
            _=tick.tick()=>{if fence().is_err() {issue=Some(CaptureIssue::Cancelled);break}}
        }
    }
    // The leader remains unreaped until owned group signalling; detached pipe
    // holders cannot extend the qualified drain indefinitely.
    drop(out);
    drop(err);
    let reaped = child.stop_and_reap().await;
    let (exit, signal, group_cleanup_unknown) = match reaped {
        Ok(r) => {
            known |= r.status.code().is_some();
            (r.status.code(), r.status.signal(), r.group_error.is_some())
        }
        Err(_) => {
            issue.get_or_insert(CaptureIssue::ReapUnknown);
            known = false;
            (None, None, true)
        }
    };
    let stdout = StreamEvidence {
        bytes: out_bytes.len() as u64,
        sha256: hash(&out_bytes),
        complete: out_done,
    };
    let stderr = StreamEvidence {
        bytes: err_bytes.len() as u64,
        sha256: hash(&err_bytes),
        complete: err_done,
    };
    Ok(CollectedCommand {
        observation: CommandObservation {
            command: command.id.clone(),
            operation,
            started_at,
            finished_at: crate::domain::now_ms(),
            timeout_seconds: command.timeout_seconds,
            drain_seconds: command.drain_seconds,
            exit,
            signal,
            work_known: known,
            issue,
            group_cleanup_unknown,
            stdout,
            stderr,
        },
        stdout: out_bytes,
        stderr: err_bytes,
    })
}

/// Independent exclusive retained files; children never receive these handles.
pub(super) fn retain(root: &Path, name: &str, bytes: &[u8]) -> Result<()> {
    ensure!(
        matches!(name, "stdout" | "stderr") && bytes.len() <= STREAM_BYTES,
        "invalid verification evidence object"
    );
    std::fs::create_dir_all(root)?;
    ensure!(
        std::fs::canonicalize(root)? == root,
        "verification evidence root aliases another namespace"
    );
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join(name))?;
    f.write_all(bytes)?;
    f.sync_all()?;
    use std::os::unix::fs::PermissionsExt;
    f.set_permissions(std::fs::Permissions::from_mode(0o400))?;
    std::fs::File::open(root)?.sync_all()?;
    Ok(())
}
pub(super) fn inspect(
    root: &Path,
    name: &str,
    reference: &StreamEvidence,
    maximum: usize,
) -> Result<Vec<u8>> {
    ensure!(
        matches!(name, "stdout" | "stderr")
            && reference.bytes <= STREAM_BYTES as u64
            && reference.bytes <= maximum as u64,
        "verification evidence retrieval limit"
    );
    let path = root.join(name);
    ensure!(
        std::fs::canonicalize(&path)? == path && std::fs::symlink_metadata(&path)?.is_file(),
        "verification evidence path changed"
    );
    let mut bytes = Vec::new();
    std::fs::File::open(&path)?
        .take(reference.bytes + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 == reference.bytes && hash(&bytes) == reference.sha256,
        "verification evidence digest mismatch"
    );
    Ok(bytes)
}
pub(super) fn evidence_root(root: &Path, run: crate::execution::UnitId, index: usize) -> PathBuf {
    root.join("verification-evidence")
        .join(run.to_string())
        .join(index.to_string())
}
