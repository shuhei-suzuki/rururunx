//! Owned-child hygiene, without process census or a descendant-death claim.
use anyhow::{Context, Result, ensure};
use rustix::process::{Pid, Signal, WaitId, WaitIdOptions, kill_process_group, waitid};
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
                if p.unreaped && p.signal_group().is_err() {
                    let _ = p.child.start_kill();
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
    pub(crate) fn spawn(mut command: Command) -> Result<Self> {
        command.process_group(0).kill_on_drop(true);
        let child = rrx_spawn_hygiene::spawn_tokio(command).context("spawn owned command")?;
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
pub(crate) async fn capture(command: Command) -> Result<Vec<u8>> {
    let observed = capture_observed(command).await?;
    ensure!(
        observed.receipt.status.success(),
        "managed command failed (exit {:?})",
        observed.receipt.status.code()
    );
    Ok(observed.stdout)
}
#[cfg(test)]
pub(crate) async fn capture_observed(mut command: Command) -> Result<CommandCapture> {
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

/// Signal only an unreaped owned group; EPERM requires confirmed-dead inspection.
fn resolve_probe_signal(
    owned: &OwnedProcess,
    signal: std::result::Result<(), rustix::io::Errno>,
    inspect: impl FnOnce() -> std::io::Result<bool>,
) -> Result<()> {
    ensure!(owned.unreaped, "probe child has already been reaped");
    #[cfg(any(target_os = "macos", test))]
    let result = crate::adapter::resolve_macos_signal_result(signal, inspect);
    #[cfg(not(any(target_os = "macos", test)))]
    let result: std::io::Result<()> = {
        let _ = inspect;
        match signal {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            Err(e) => Err(e.into()),
        }
    };
    result.context("signal probe unreaped process group")
}
fn signal_probe_group(
    owned: &OwnedProcess,
    kill: impl FnOnce(Pid, Signal) -> std::result::Result<(), rustix::io::Errno>,
    inspect: impl FnOnce(Pid) -> std::io::Result<bool>,
) -> Result<()> {
    ensure!(owned.unreaped, "probe child has already been reaped");
    let result = kill(owned.pid, Signal::KILL);
    resolve_probe_signal(owned, result, || inspect(owned.pid))
}
fn inspect_probe_group(pid: Pid) -> std::io::Result<bool> {
    #[cfg(target_os = "macos")]
    {
        crate::adapter::macos_group_is_dead(pid)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = pid;
        Ok(false)
    }
}

/// Startup probes retain unreaped custody until cleanup and direct wait finish.
#[cfg(all(test, target_os = "macos"))]
pub(crate) async fn capture_probe(command: Command) -> Result<(ProcessReceipt, Vec<u8>, Vec<u8>)> {
    capture_probe_until(
        command,
        Duration::from_secs(5),
        tokio::time::Instant::now() + Duration::from_secs(5),
    )
    .await
}
pub(crate) async fn capture_probe_until(
    command: Command,
    bound: Duration,
    overall: tokio::time::Instant,
) -> Result<(ProcessReceipt, Vec<u8>, Vec<u8>)> {
    capture_probe_until_with_kill(command, bound, overall, |pid, signal, kill| {
        kill(pid, signal)
    })
    .await
}
type ProbeKill = fn(Pid, Signal) -> std::result::Result<(), rustix::io::Errno>;

async fn capture_probe_until_with_kill(
    command: Command,
    bound: Duration,
    overall: tokio::time::Instant,
    kill: impl FnOnce(Pid, Signal, ProbeKill) -> std::result::Result<(), rustix::io::Errno>
    + Send
    + 'static,
) -> Result<(ProcessReceipt, Vec<u8>, Vec<u8>)> {
    // Keep the platform defaults in one place; EPERM tests replace only the
    // signal outcome while exercising this same kill/inspection wiring.
    capture_probe_with_cleanup(command, bound, overall, move |owned| {
        signal_probe_group(
            owned,
            |pid, signal| kill(pid, signal, kill_process_group),
            inspect_probe_group,
        )
    })
    .await
}
async fn capture_probe_with_cleanup(
    command: Command,
    bound: Duration,
    overall: tokio::time::Instant,
    cleanup: impl FnOnce(&OwnedProcess) -> Result<()> + Send + 'static,
) -> Result<(ProcessReceipt, Vec<u8>, Vec<u8>)> {
    capture_probe_with_spawn(command, bound, overall, OwnedProcess::spawn, cleanup).await
}
async fn capture_probe_with_spawn(
    mut command: Command,
    bound: Duration,
    overall: tokio::time::Instant,
    spawn: impl FnOnce(Command) -> Result<OwnedProcess> + Send + 'static,
    cleanup: impl FnOnce(&OwnedProcess) -> Result<()> + Send + 'static,
) -> Result<(ProcessReceipt, Vec<u8>, Vec<u8>)> {
    ensure!(
        tokio::time::Instant::now() < overall,
        "probe timed out before spawn"
    );
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Spawn includes synchronous platform hygiene and fork/exec contention. Do
    // not block an async worker or charge that wait to the child's observation
    // budget. Stamp the deadline in the worker, not after its join is scheduled:
    // the child is already running while that join waits to be polled.
    let (mut child, deadline) = tokio::task::spawn_blocking(move || {
        ensure!(
            tokio::time::Instant::now() < overall,
            "probe timed out before spawn"
        );
        let child = spawn(command)?;
        let deadline = (tokio::time::Instant::now() + bound).min(overall);
        Ok::<_, anyhow::Error>((child, deadline))
    })
    .await
    .context("probe spawn worker failed")??;
    let stdout = child.child.stdout.take().context("probe stdout missing")?;
    let stderr = child.child.stderr.take().context("probe stderr missing")?;
    let readers =
        async { tokio::try_join!(bounded_read(stdout, 4096), bounded_read(stderr, 4096)) };
    tokio::pin!(readers);
    let mut bytes = None;
    let observed = tokio::time::timeout_at(deadline, async {
        tokio::select! {
            read = &mut readers => { bytes = Some(read?); child.exited().await?; },
            exit = child.exited() => { exit?; }
        }
        Ok::<_, anyhow::Error>(())
    })
    .await;
    // No cleanup await is inside the observation timeout. A timeout while the
    // inspector runs retains this handle and collects its custody/result first.
    let mut worker = tokio::task::spawn_blocking(move || {
        let result = cleanup(&child);
        (child, result)
    });
    let (joined, cleanup_timed_out) = match tokio::time::timeout_at(deadline, &mut worker).await {
        Ok(result) => (result, false),
        Err(_) => (worker.await, true),
    };
    let (mut child, cleanup) = joined.context("probe cleanup worker failed")?;
    let group_error = cleanup.err().map(|e| format!("{e:#}"));
    if group_error.is_some() {
        let _ = child.child.start_kill();
    }
    let status = tokio::time::timeout(Duration::from_secs(10), child.child.wait()).await??;
    child.unreaped = false;
    let receipt = ProcessReceipt {
        status,
        group_error,
    };
    ensure!(
        receipt.group_error.is_none(),
        "probe group cleanup failed: {:?}",
        receipt.group_error
    );
    observed.context("probe timed out")??;
    ensure!(!cleanup_timed_out, "probe timed out during cleanup");
    let (out, err) = match bytes {
        Some(bytes) => bytes,
        None => tokio::time::timeout_at(
            deadline.min(tokio::time::Instant::now() + Duration::from_secs(2)),
            &mut readers,
        )
        .await
        .context("probe drain incomplete")??,
    };
    ensure!(
        out.len() + err.len() <= 4096,
        "probe output exceeded its bound"
    );
    Ok((receipt, out, err))
}

#[cfg(test)]
mod probe_tests {
    use super::*;

    /// The default probe inspector refuses live-group EPERM and accepts confirmed-dead EPERM.
    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn b3_probe_default_signal_wiring() {
        for live in [false, true] {
            let pid = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
            let observed_pid = pid.clone();
            let mut command = if live {
                let mut command = Command::new("/bin/sleep");
                command.arg("10");
                command
            } else {
                Command::new("/usr/bin/true")
            };
            command.env_clear();
            let result = capture_probe_until_with_kill(
                command,
                Duration::from_millis(500),
                tokio::time::Instant::now() + Duration::from_secs(5),
                move |pid, signal, kill| {
                    assert_eq!(signal, Signal::KILL);
                    observed_pid.store(
                        pid.as_raw_nonzero().get(),
                        std::sync::atomic::Ordering::SeqCst,
                    );
                    if !live {
                        match kill(pid, signal) {
                            Ok(()) | Err(rustix::io::Errno::SRCH) => {}
                            Err(error) => return Err(error),
                        }
                    }
                    Err(rustix::io::Errno::PERM)
                },
            )
            .await;
            if live {
                assert!(
                    result
                        .err()
                        .expect("live-group EPERM must refuse")
                        .to_string()
                        .contains("probe group cleanup failed")
                );
            } else {
                assert!(result.unwrap().0.status.success());
            }
            let pid = Pid::from_raw(pid.load(std::sync::atomic::Ordering::SeqCst)).unwrap();
            assert_eq!(
                waitid(
                    WaitId::Pid(pid),
                    WaitIdOptions::EXITED | WaitIdOptions::NOHANG
                )
                .unwrap_err(),
                rustix::io::Errno::CHILD
            );
        }
    }

    /// Blocking launch preparation neither spends child observation time nor stalls async work.
    #[tokio::test]
    async fn b3_q_probe_deadlines() {
        let (entered, ready) = tokio::sync::oneshot::channel();
        let (release, released) = std::sync::mpsc::channel();
        let probe = capture_probe_with_spawn(
            Command::new("/usr/bin/true"),
            Duration::from_millis(500),
            tokio::time::Instant::now() + Duration::from_secs(5),
            move |command| {
                entered.send(()).unwrap();
                released.recv_timeout(Duration::from_secs(3)).unwrap();
                OwnedProcess::spawn(command)
            },
            |child| signal_probe_group(child, kill_process_group, |_| Ok(true)),
        );
        let observer = async {
            ready.await.unwrap();
            tokio::time::sleep(Duration::from_millis(700)).await;
            release.send(()).unwrap();
        };
        let (result, ()) = tokio::join!(probe, observer);
        let (receipt, _, _) = result.unwrap();
        assert!(receipt.status.success());
    }

    /// Both the child's own bound and the overall CLI deadline stop observation and reap.
    #[tokio::test]
    async fn b3_probe_observation_bounds() {
        for (bound, overall) in [
            (Duration::from_millis(500), Duration::from_secs(5)),
            (Duration::from_secs(5), Duration::from_millis(500)),
        ] {
            let started = std::time::Instant::now();
            let pid = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
            let observed_pid = pid.clone();
            let mut command = Command::new("/bin/sleep");
            command.arg("10");
            let result = capture_probe_with_spawn(
                command,
                bound,
                tokio::time::Instant::now() + overall,
                OwnedProcess::spawn,
                move |child| {
                    observed_pid.store(
                        child.pid.as_raw_nonzero().get(),
                        std::sync::atomic::Ordering::SeqCst,
                    );
                    signal_probe_group(child, kill_process_group, |_| Ok(true))
                },
            )
            .await;
            assert!(
                result
                    .err()
                    .expect("sleep must time out")
                    .to_string()
                    .contains("probe timed out")
            );
            assert!(started.elapsed() < Duration::from_secs(3));
            let pid = Pid::from_raw(pid.load(std::sync::atomic::Ordering::SeqCst)).unwrap();
            assert_eq!(
                waitid(
                    WaitId::Pid(pid),
                    WaitIdOptions::EXITED | WaitIdOptions::NOHANG
                )
                .unwrap_err(),
                rustix::io::Errno::CHILD
            );
        }
    }

    /// A deadline during blocking cleanup cannot release custody before its result and reap.
    #[tokio::test]
    async fn b3_probe_cleanup_deadline() {
        let completed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let done = completed.clone();
        let pid = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
        let observed_pid = pid.clone();
        let started = std::time::Instant::now();
        let result = capture_probe_with_cleanup(
            Command::new("/usr/bin/true"),
            Duration::from_millis(500),
            tokio::time::Instant::now() + Duration::from_millis(500),
            move |owned| {
                observed_pid.store(
                    owned.pid.as_raw_nonzero().get(),
                    std::sync::atomic::Ordering::SeqCst,
                );
                std::thread::sleep(Duration::from_millis(700));
                let result = resolve_probe_signal(owned, Err(rustix::io::Errno::PERM), || Ok(true));
                done.store(true, std::sync::atomic::Ordering::SeqCst);
                result
            },
        )
        .await;
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("probe timed out during cleanup")
        );
        assert!(started.elapsed() >= Duration::from_millis(700));
        assert!(completed.load(std::sync::atomic::Ordering::SeqCst));
        let pid =
            rustix::process::Pid::from_raw(pid.load(std::sync::atomic::Ordering::SeqCst)).unwrap();
        assert_eq!(
            rustix::process::waitid(
                rustix::process::WaitId::Pid(pid),
                rustix::process::WaitIdOptions::EXITED | rustix::process::WaitIdOptions::NOHANG
            )
            .unwrap_err(),
            rustix::io::Errno::CHILD
        );
    }

    /// A live-group EPERM on the actual probe cleanup path refuses its result after reaping.
    #[tokio::test]
    async fn b3_probe_live_group_refused() {
        let error = capture_probe_with_cleanup(
            Command::new("/usr/bin/true"),
            Duration::from_secs(5),
            tokio::time::Instant::now() + Duration::from_secs(5),
            |owned| resolve_probe_signal(owned, Err(rustix::io::Errno::PERM), || Ok(false)),
        )
        .await
        .err()
        .unwrap();
        assert!(
            error.to_string().contains("probe group cleanup failed"),
            "{error:#}"
        );
    }
    /// The production signal wiring preserves the kill result and the supplied inspector.
    #[tokio::test]
    async fn b3_probe_signal_wiring() {
        for (signal, dead, accepted) in [
            (Err(rustix::io::Errno::PERM), true, true),
            (Err(rustix::io::Errno::PERM), false, false),
            (Err(rustix::io::Errno::ACCESS), true, false),
        ] {
            let error = capture_probe_with_cleanup(
                Command::new("/usr/bin/true"),
                Duration::from_secs(5),
                tokio::time::Instant::now() + Duration::from_secs(5),
                move |owned| {
                    signal_probe_group(
                        owned,
                        |pid, kind| {
                            assert_eq!(pid, owned.pid);
                            assert_eq!(kind, Signal::KILL);
                            signal
                        },
                        |pid| {
                            assert_eq!(pid, owned.pid);
                            Ok(dead)
                        },
                    )
                },
            )
            .await;
            assert_eq!(error.is_ok(), accepted, "{:?}", error.err());
        }
    }

    /// Only confirmed dead groups resolve EPERM; every uncertain observation refuses.
    #[test]
    fn b3_probe_macos_signal_resolution() {
        use crate::adapter::resolve_macos_signal_result as resolve;
        use rustix::io::Errno;
        assert!(resolve(Err(Errno::PERM), || Ok(true)).is_ok());
        assert!(resolve(Err(Errno::PERM), || Ok(false)).is_err());
        for kind in [
            std::io::ErrorKind::InvalidData,
            std::io::ErrorKind::TimedOut,
        ] {
            assert!(resolve(Err(Errno::PERM), || Err(std::io::Error::from(kind))).is_err());
        }
        assert!(resolve(Err(Errno::ACCESS), || Ok(true)).is_err());
    }
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
    capture_scoped_with(
        child,
        owner,
        pinned,
        HelperCurrency::Generic { native, driver },
    )
    .await
}
/// Which currency the running-helper fence rechecks on every tick.
pub(crate) enum HelperCurrency<'a> {
    /// The existing generic authority, unchanged.
    Generic {
        native: bool,
        driver: Option<&'a crate::state::DriverReadTicket>,
    },
    /// The protected reader of a settled, marker-bound phase.
    Settled(&'a crate::state::managed_binding::SettledCurrency),
}
/// Both arms keep the 50 ms interval (first tick immediate), the pinned
/// identity comparison and the kill-on-Err drop of the capture future.
pub(crate) async fn capture_scoped_with(
    child: OwnedProcess,
    owner: &super::RuntimeOwner,
    pinned: &super::ExecutionUnit,
    currency: HelperCurrency<'_>,
) -> Result<CommandCapture> {
    #[cfg(test)]
    let held = match &currency {
        HelperCurrency::Settled(settled) => settled.settled().marker().scope().task_id,
        HelperCurrency::Generic { .. } => None,
    };
    let capture = async move {
        #[cfg(test)]
        crate::runtime::hold_helper(held).await;
        capture_child(child).await
    };
    tokio::pin!(capture);
    let mut fence = tokio::time::interval(Duration::from_millis(50));
    loop {
        tokio::select! {
            observed = &mut capture => return observed,
            _ = fence.tick() => {
                let mut store = owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?;
                match &currency {
                    HelperCurrency::Generic { native, driver } => {
                        if let Some(ticket) = driver { store.validate_driver_read(ticket)?; }
                        let current = store.execution_unit(pinned.id)?;
                        ensure!(current.scope == pinned.scope && current.generation == pinned.generation && current.owner_epoch == pinned.owner_epoch && current.session_id == pinned.session_id, "helper execution identity changed");
                        store.validate_execution(&current.authority(), *native, !*native)?;
                    }
                    HelperCurrency::Settled(settled) => {
                        let current = store.execution_unit(pinned.id)?;
                        ensure!(current.scope == pinned.scope && current.generation == pinned.generation && current.owner_epoch == pinned.owner_epoch && current.session_id == pinned.session_id, "helper execution identity changed");
                        store.validate_settled_helper(settled, pinned)?;
                        #[cfg(test)]
                        crate::runtime::count(settled.settled().marker().scope().task_id, "settled fence tick");
                    }
                }
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
        let child = rrx_spawn_hygiene::spawn_tokio(command).unwrap();
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
        let child = rrx_spawn_hygiene::spawn_tokio(command).unwrap();
        custody.adopt(child);
        custody.qualify().unwrap();
        assert!(custody.pid.is_some());
        assert!(!custody.reaped);
        assert!(custody.hygiene());
        assert!(!reaped(&mut custody).await.success());
        assert!(custody.exited_unreaped().is_err());
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
        let child = rrx_spawn_hygiene::spawn_tokio(command).unwrap();
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
