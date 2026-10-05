//! Owned-child hygiene, without process census or a descendant-death claim.
use anyhow::{Context, Result, ensure};
use rustix::process::{Pid, Signal, WaitId, WaitIdOptions, kill_process_group, waitid};
use std::{
    process::{ExitStatus, Stdio},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
};

pub(crate) struct OwnedProcess {
    pub(crate) child: Child,
    pid: Pid,
    unreaped: bool,
}
pub(crate) struct ProcessReceipt {
    pub(crate) status: ExitStatus,
    pub(crate) group_error: Option<String>,
}
pub(crate) struct CommandCapture {
    pub stdout: Vec<u8>,
    pub receipt: ProcessReceipt,
}
impl OwnedProcess {
    pub(crate) fn spawn(command: &mut Command) -> Result<Self> {
        command.process_group(0).kill_on_drop(true);
        let child = command.spawn().context("spawn owned command")?;
        let pid = Pid::from_raw(child.id().context("missing owned child identity")? as i32)
            .context("invalid owned child identity")?;
        Ok(Self {
            child,
            pid,
            unreaped: true,
        })
    }
    pub(crate) async fn exited(&self) -> Result<()> {
        loop {
            match waitid(
                WaitId::Pid(self.pid),
                WaitIdOptions::EXITED | WaitIdOptions::NOWAIT | WaitIdOptions::NOHANG,
            ) {
                Ok(Some(_)) => return Ok(()),
                Ok(None) | Err(rustix::io::Errno::INTR) => {
                    tokio::time::sleep(Duration::from_millis(20)).await
                }
                Err(e) => {
                    return Err(
                        anyhow::Error::from(e).context("observe owned child without reaping")
                    );
                }
            }
        }
    }
    pub(crate) fn signal_group(&self) -> Result<()> {
        ensure!(self.unreaped, "owned child has already been reaped");
        match kill_process_group(self.pid, Signal::KILL) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            Err(e) => Err(anyhow::Error::from(e).context("signal owned unreaped process group")),
        }
    }
    pub(crate) async fn stop_and_reap(&mut self) -> Result<ProcessReceipt> {
        let group_error = self.signal_group().err().map(|e| format!("{e:#}"));
        // Group hygiene failure must not discard known direct-child work.
        // The child API retains its unreaped identity for this direct stop.
        if group_error.is_some() {
            let _ = self.child.start_kill();
        }
        let status = tokio::time::timeout(Duration::from_secs(10), self.child.wait()).await??;
        self.unreaped = false;
        Ok(ProcessReceipt {
            status,
            group_error,
        })
    }
}
impl Drop for OwnedProcess {
    fn drop(&mut self) {
        if self.unreaped {
            let _ = self.signal_group();
        }
    }
}

pub(crate) async fn bounded_read(reader: impl AsyncRead + Unpin, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .await?;
    ensure!(bytes.len() <= limit, "command output exceeded its bound");
    Ok(bytes)
}

#[cfg(test)]
pub(crate) async fn capture(command: &mut Command) -> Result<Vec<u8>> {
    let observed = capture_observed(command).await?;
    ensure!(
        observed.receipt.status.success(),
        "managed command failed (exit {:?})",
        observed.receipt.status.code()
    );
    Ok(observed.stdout)
}
#[cfg(test)]
pub(crate) async fn capture_observed(command: &mut Command) -> Result<CommandCapture> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    capture_child(OwnedProcess::spawn(command)?).await
}
pub(crate) async fn capture_child(mut child: OwnedProcess) -> Result<CommandCapture> {
    let stdout = child.child.stdout.take().context("stdout missing")?;
    let stderr = child.child.stderr.take().context("stderr missing")?;
    let readers = async {
        tokio::try_join!(
            bounded_read(stdout, 4 * 1024 * 1024),
            bounded_read(stderr, 256 * 1024)
        )
    };
    tokio::pin!(readers);
    let collected=tokio::time::timeout(Duration::from_secs(60),async {
        tokio::select! {
            read=&mut readers=>{let bytes=read?;child.exited().await?;Ok::<_,anyhow::Error>(bytes)},
            exited=child.exited()=>{
                exited?;let _=child.signal_group();
                // Detached descendants may hold pipes; bounded drain is not death proof.
                tokio::time::timeout(Duration::from_secs(2),&mut readers).await.context("output drain incomplete")?
            }
        }
    }).await;
    // Keep the leader unreaped until group signaling, even after a normal exit.
    let receipt = child.stop_and_reap().await?;
    let (out, _) = collected.context("command timed out")??;
    // Do not copy arbitrary tool stderr into an audit receipt (it may be sensitive).
    Ok(CommandCapture {
        stdout: out,
        receipt,
    })
}

pub(crate) async fn capture_scoped(
    child: OwnedProcess,
    owner: &super::RuntimeOwner,
    pinned: &super::ExecutionUnit,
    native: bool,
) -> Result<CommandCapture> {
    let capture = capture_child(child);
    tokio::pin!(capture);
    let mut fence = tokio::time::interval(Duration::from_millis(50));
    loop {
        tokio::select! {
            observed = &mut capture => return observed,
            _ = fence.tick() => {
                let store = owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?;
                let current = store.execution_unit(pinned.id)?;
                ensure!(current.scope == pinned.scope && current.generation == pinned.generation && current.owner_epoch == pinned.owner_epoch && current.session_id == pinned.session_id, "helper execution identity changed");
                store.validate_execution(&current.authority(), native, !native)?;
            }
        }
    }
}
