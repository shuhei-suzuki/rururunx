//! Owned-child hygiene, without process census or a descendant-death claim.
use anyhow::{Context, Result, ensure};
use rustix::process::{Pid, Signal, WaitId, WaitIdOptions, kill_process_group, waitid};
#[cfg(test)]
use std::process::Stdio;
use std::{process::ExitStatus, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
};

pub(crate) struct OwnedProcess {
    pub(crate) child: Child,
    pid: Pid,
    unreaped: bool,
}
/// Protected transport custody never exposes a process before final Core handoff.
#[derive(Default)]
pub(crate) enum NativeChildCell {
    #[default]
    Empty,
    Raw(RetainedRawProcess),
    Owned(OwnedProcess),
    Transferred,
}
pub(crate) struct NativePipes {
    pub(crate) stdin: tokio::process::ChildStdin,
    pub(crate) stdout: tokio::process::ChildStdout,
    pub(crate) stderr: tokio::process::ChildStderr,
}
pub(crate) struct Refused<S> {
    pub(crate) shell: S,
    pub(crate) reason: &'static str,
}
impl NativeChildCell {
    pub(crate) fn adopt(&mut self, child: Child) {
        // The one-shot spawn checks Empty before creating a Child.
        *self = Self::Raw(RetainedRawProcess {
            child: Some(child),
            pid: None,
            reaped: false,
        });
    }
    pub(crate) fn qualify(&mut self) -> Result<()> {
        let Self::Raw(raw) = self else {
            anyhow::bail!("transport raw custody absent")
        };
        raw.qualify()
    }
    pub(crate) fn upgrade_in_place(&mut self) -> Result<()> {
        let Self::Raw(raw) = self else {
            anyhow::bail!("transport raw custody absent")
        };
        ensure!(
            !raw.reaped && raw.child.is_some(),
            "transport child no longer unreaped"
        );
        let pid = raw.pid.context("transport child unqualified")?;
        // All checks precede the only move; the emptied raw Drop has no child.
        let child = raw.child.take().unwrap();
        *self = Self::Owned(OwnedProcess {
            child,
            pid,
            unreaped: true,
        });
        Ok(())
    }
    pub(crate) fn take_native_pipes(&mut self) -> Result<NativePipes> {
        let Self::Owned(process) = self else {
            anyhow::bail!("transport owned custody absent")
        };
        let child = &mut process.child;
        ensure!(
            child.stdin.is_some() && child.stdout.is_some() && child.stderr.is_some(),
            "transport pipes incomplete"
        );
        Ok(NativePipes {
            stdin: child.stdin.take().unwrap(),
            stdout: child.stdout.take().unwrap(),
            stderr: child.stderr.take().unwrap(),
        })
    }
    pub(crate) fn transfer_with<S, T>(
        &mut self,
        open: bool,
        shell: S,
        build: fn(S, OwnedProcess) -> T,
    ) -> std::result::Result<T, Refused<S>> {
        let ready = matches!(self, Self::Owned(p) if p.unreaped && p.child.stdin.is_none() && p.child.stdout.is_none() && p.child.stderr.is_none());
        if !open || !ready {
            return Err(Refused {
                shell,
                reason: "transport transfer refused",
            });
        }
        let Self::Owned(process) = std::mem::replace(self, Self::Transferred) else {
            unreachable!()
        };
        Ok(build(shell, process))
    }
    pub(crate) fn qualified(&self) -> bool {
        matches!(self, Self::Owned(_)) || matches!(self, Self::Raw(raw) if raw.pid.is_some())
    }
    pub(crate) fn hygiene(&mut self) -> &'static str {
        match self {
            Self::Raw(raw) => {
                let qualified = raw.pid.is_some();
                raw.hygiene();
                if qualified {
                    "group_signal_attempted"
                } else {
                    "direct_kill_attempted"
                }
            }
            Self::Owned(p) => {
                if p.unreaped {
                    if p.signal_group().is_err() {
                        let _ = p.child.start_kill();
                    }
                }
                "group_signal_attempted"
            }
            _ => "n/a",
        }
    }
    pub(crate) fn try_reap(&mut self) -> Result<Option<ExitStatus>> {
        match self {
            Self::Raw(raw) => raw.reap(),
            Self::Owned(p) => {
                let status = p.child.try_wait()?;
                if status.is_some() {
                    p.unreaped = false;
                }
                Ok(status)
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod native_child_cell_tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    async fn spawned() -> NativeChildCell {
        let mut command = Command::new("/bin/sleep");
        command
            .arg("30")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .kill_on_drop(true);
        let mut cell = NativeChildCell::Empty;
        let child = command.spawn().unwrap();
        cell.adopt(child);
        cell
    }
    async fn closed(cell: &mut NativeChildCell) {
        cell.hygiene();
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                if cell.try_reap().unwrap().is_some() {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
    }
    #[tokio::test]
    async fn raw_upgrade_refuses_without_releasing_original_child() {
        let mut cell = spawned().await;
        let refused = cell.upgrade_in_place().is_err();
        let retained =
            matches!(&cell,NativeChildCell::Raw(raw) if raw.has_child() && raw.pid.is_none());
        closed(&mut cell).await;
        assert!(refused && retained);
    }
    #[tokio::test]
    async fn missing_pipe_refuses_before_taking_any_other_pipe() {
        let mut cell = spawned().await;
        cell.qualify().unwrap();
        cell.upgrade_in_place().unwrap();
        let NativeChildCell::Owned(process) = &mut cell else {
            panic!()
        };
        let stderr = process.child.stderr.take();
        let refused = cell.take_native_pipes().is_err();
        let intact = matches!(&cell,NativeChildCell::Owned(p) if p.child.stdin.is_some() && p.child.stdout.is_some());
        drop(stderr);
        closed(&mut cell).await;
        assert!(refused && intact);
    }
    struct Shell(Arc<AtomicUsize>);
    impl Drop for Shell {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn build(shell: Shell, process: OwnedProcess) -> (Shell, OwnedProcess) {
        (shell, process)
    }
    #[tokio::test]
    async fn refused_transfer_returns_same_shell_and_keeps_child_in_cell() {
        let mut cell = spawned().await;
        cell.qualify().unwrap();
        cell.upgrade_in_place().unwrap();
        let pipes = cell.take_native_pipes().unwrap();
        let drops = Arc::new(AtomicUsize::new(0));
        let shell = Shell(drops.clone());
        let refused = match cell.transfer_with(false, shell, build) {
            Err(refused) => refused,
            Ok((shell, mut process)) => {
                process.stop_and_reap().await.unwrap();
                drop(pipes);
                drop(shell);
                panic!("closed handoff transferred child");
            }
        };
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert!(matches!(&cell,NativeChildCell::Owned(p) if p.unreaped));
        drop(refused.shell);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        drop(pipes);
        closed(&mut cell).await;
    }
    #[tokio::test]
    async fn transfer_requires_all_pipes_taken_and_moves_original_once() {
        let mut cell = spawned().await;
        cell.qualify().unwrap();
        cell.upgrade_in_place().unwrap();
        let original = match &cell {
            NativeChildCell::Owned(p) => p.child.id(),
            _ => None,
        };
        let drops = Arc::new(AtomicUsize::new(0));
        let shell = Shell(drops.clone());
        let refused = cell.transfer_with(true, shell, build).err().unwrap();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        let pipes = cell.take_native_pipes().unwrap();
        let (shell, mut process) = match cell.transfer_with(true, refused.shell, build) {
            Ok(value) => value,
            Err(_) => panic!("qualified transfer refused"),
        };
        assert_eq!(process.child.id(), original);
        assert!(matches!(cell, NativeChildCell::Transferred));
        assert!(
            cell.transfer_with(true, Shell(drops.clone()), build)
                .is_err()
        );
        process.stop_and_reap().await.unwrap();
        drop(pipes);
        drop(shell);
    }
}
/// Empty nongrant custody allocated before spawn. Adoption is infallible and
/// precedes every identity/pipe check; unknown identity permits direct hygiene.
#[derive(Default)]
pub(crate) struct RetainedRawProcess {
    child: Option<Child>,
    pid: Option<Pid>,
    reaped: bool,
}
impl RetainedRawProcess {
    pub(crate) fn has_child(&self) -> bool {
        self.child.is_some()
    }
    /// Known leader reap only; never group/descendant death or workload absence.
    pub(crate) fn leader_reaped(&self) -> bool {
        self.reaped
    }
    pub(crate) fn adopt(&mut self, child: Child) {
        // The private one-shot producer ensures this cell is empty.
        self.child = Some(child);
    }
    pub(crate) fn qualify(&mut self) -> Result<()> {
        let raw = self
            .child
            .as_ref()
            .context("retained raw child absent")?
            .id()
            .context("retained raw identity absent")?;
        self.pid =
            Some(Pid::from_raw(i32::try_from(raw)?).context("invalid retained raw identity")?);
        Ok(())
    }
    pub(crate) fn pipes(
        &mut self,
    ) -> Result<(tokio::process::ChildStdout, tokio::process::ChildStderr)> {
        let child = self.child.as_mut().context("retained child absent")?;
        let out = child.stdout.take().context("retained stdout absent")?;
        let err = child.stderr.take().context("retained stderr absent")?;
        Ok((out, err))
    }
    pub(crate) fn exited_unreaped(&self) -> Result<bool> {
        let pid = self
            .pid
            .context("unqualified child cannot use numeric identity")?;
        ensure!(!self.reaped, "retained child already reaped");
        match waitid(
            WaitId::Pid(pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOWAIT | WaitIdOptions::NOHANG,
        ) {
            Ok(Some(_)) => Ok(true),
            Ok(None) | Err(rustix::io::Errno::INTR) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
    pub(crate) fn hygiene(&mut self) -> bool {
        if self.reaped {
            return true;
        }
        let Some(child) = self.child.as_mut() else {
            return false;
        };
        if let Some(pid) = self.pid {
            match kill_process_group(pid, Signal::KILL) {
                Ok(()) | Err(rustix::io::Errno::SRCH) => return true,
                Err(_) => {}
            }
        }
        // Unqualified identity or failed group signal never adopts another PID.
        let _ = child.start_kill();
        false
    }
    pub(crate) fn reap(&mut self) -> Result<Option<ExitStatus>> {
        let Some(child) = self.child.as_mut() else {
            return Ok(None);
        };
        let status = child.try_wait()?;
        if status.is_some() {
            self.reaped = true;
        }
        Ok(status)
    }
}
impl Drop for RetainedRawProcess {
    fn drop(&mut self) {
        // Run before Child destruction/reaping; this never proves group death.
        let _ = self.hygiene();
    }
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
    capture_scoped_pinned(child, owner, pinned, native, None).await
}
pub(crate) async fn capture_scoped_pinned(
    child: OwnedProcess,
    owner: &super::RuntimeOwner,
    pinned: &super::ExecutionUnit,
    native: bool,
    driver: Option<&crate::state::DriverReadTicket>,
) -> Result<CommandCapture> {
    let capture = capture_child(child);
    tokio::pin!(capture);
    let mut fence = tokio::time::interval(Duration::from_millis(50));
    loop {
        tokio::select! {
            observed = &mut capture => return observed,
            _ = fence.tick() => {
                let mut store = owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?;
                if let Some(ticket) = driver { store.validate_driver_read(ticket)?; }
                let current = store.execution_unit(pinned.id)?;
                ensure!(current.scope == pinned.scope && current.generation == pinned.generation && current.owner_epoch == pinned.owner_epoch && current.session_id == pinned.session_id, "helper execution identity changed");
                store.validate_execution(&current.authority(), native, !native)?;
            }
        }
    }
}

#[cfg(test)]
mod retained_raw_tests {
    use super::*;

    async fn reaped(raw: &mut RetainedRawProcess) -> ExitStatus {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                if let Some(status) = raw.reap().unwrap() {
                    return status;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap()
    }
    #[tokio::test]
    async fn nongrant_raw_adoption_precedes_qualification_and_direct_hygiene() {
        let mut command = Command::new("/bin/sleep");
        command.arg("30").process_group(0).kill_on_drop(true);
        let mut custody = RetainedRawProcess::default();
        let child = command.spawn().unwrap();
        custody.adopt(child);
        assert!(custody.has_child());
        assert!(custody.pid.is_none());
        assert!(custody.exited_unreaped().is_err());
        assert!(!custody.hygiene());
        assert!(!reaped(&mut custody).await.success());
        assert!(custody.reaped);
    }
    #[tokio::test]
    async fn nongrant_qualified_raw_leader_is_signalled_before_reaping() {
        let mut command = Command::new("/bin/sleep");
        command.arg("30").process_group(0).kill_on_drop(true);
        let mut custody = RetainedRawProcess::default();
        let child = command.spawn().unwrap();
        custody.adopt(child);
        custody.qualify().unwrap();
        assert!(custody.pid.is_some());
        assert!(!custody.reaped);
        assert!(custody.hygiene());
        assert!(!reaped(&mut custody).await.success());
        assert!(custody.exited_unreaped().is_err());
    }
}
