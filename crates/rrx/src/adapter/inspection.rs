//! macOS-only observational evidence for an already-owned unreaped process group.
//! This never creates signal authority from numeric hints or claims an atomic sample.
use std::{
    collections::BTreeSet,
    fs::File,
    io::{self, Read},
    os::fd::OwnedFd,
    path::Path,
    process::{Child, ExitStatus, Stdio},
    time::{Duration, Instant},
};

const LIMIT: usize = 1024 * 1024;
const BUDGET: Duration = Duration::from_millis(250);
const IDLE: Duration = Duration::from_millis(5);

fn framing(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn timed_out() -> io::Error {
    io::Error::new(
        io::ErrorKind::TimedOut,
        "native process inspection timed out",
    )
}
fn current(deadline: Instant) -> io::Result<()> {
    if Instant::now() >= deadline {
        Err(timed_out())
    } else {
        Ok(())
    }
}

/// std Child has no kill-on-drop. Once reaped or ownership is uncertain, neither
/// another signal nor another wait by numeric PID is permitted.
struct Inspector {
    child: Child,
    unreaped: bool,
    exit: Option<ExitStatus>,
}
impl Inspector {
    fn observe(&mut self) -> io::Result<()> {
        if self.exit.is_some() {
            return Ok(());
        }
        match self.child.try_wait() {
            Ok(Some(exit)) => {
                self.unreaped = false;
                self.exit = Some(exit);
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => Ok(()),
            Err(_) => {
                self.unreaped = false;
                Err(io::Error::other(
                    "native inspector reap ownership uncertain",
                ))
            }
        }
    }
    fn cleanup(&mut self) -> io::Result<()> {
        if !self.unreaped {
            return Ok(());
        }
        // The leader remains our unreaped Child even if it exited between polls.
        // A kill error must not skip mandatory wait; wait can resolve that race.
        let _kill = self.child.kill();
        let result = loop {
            match self.child.wait() {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                result => break result,
            }
        };
        self.unreaped = false;
        result
            .map(|exit| self.exit = Some(exit))
            .map_err(|_| io::Error::other("native inspector cleanup/reap uncertain"))
    }
}
impl Drop for Inspector {
    fn drop(&mut self) {
        // Only the still-owned exact direct child; no recursive group inspection.
        let _ = self.cleanup();
    }
}

struct Stream {
    file: File,
    bytes: Vec<u8>,
    eof: bool,
    overflow: &'static str,
}
impl Stream {
    fn new(file: File, overflow: &'static str) -> io::Result<Self> {
        let flags = rustix::fs::fcntl_getfl(&file)?;
        rustix::fs::fcntl_setfl(&file, flags | rustix::fs::OFlags::NONBLOCK)?;
        Ok(Self {
            file,
            bytes: vec![],
            eof: false,
            overflow,
        })
    }
    /// One bounded chunk lets the other stream, status and deadline progress.
    fn drain(&mut self, deadline: Instant) -> io::Result<bool> {
        current(deadline)?;
        if self.eof {
            return Ok(false);
        }
        let mut chunk = [0; 8192];
        let maximum = chunk.len().min(LIMIT + 1 - self.bytes.len());
        match self.file.read(&mut chunk[..maximum]) {
            Ok(0) => {
                self.eof = true;
                Ok(false)
            }
            Ok(count) => {
                self.bytes
                    .try_reserve(count)
                    .map_err(|_| io::Error::other("native inspection allocation failed"))?;
                self.bytes.extend_from_slice(&chunk[..count]);
                current(deadline)?;
                if self.bytes.len() > LIMIT {
                    Err(framing(self.overflow))
                } else {
                    Ok(true)
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(false),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => Ok(true),
            Err(error) => Err(error),
        }
    }
}

struct Frame {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit: ExitStatus,
}
fn complete(
    inspector: &mut Inspector,
    stdout: File,
    stderr: File,
    deadline: Instant,
) -> io::Result<Frame> {
    let mut stdout = Stream::new(stdout, "native inspection stdout exceeds byte budget")?;
    let mut stderr = Stream::new(stderr, "native inspection stderr exceeds byte budget")?;
    loop {
        current(deadline)?;
        let stdout_progress = stdout.drain(deadline)?;
        let stderr_progress = stderr.drain(deadline)?;
        inspector.observe()?;
        current(deadline)?;
        if stdout.eof
            && stderr.eof
            && let Some(exit) = inspector.exit
        {
            return Ok(Frame {
                stdout: stdout.bytes,
                stderr: stderr.bytes,
                exit,
            });
        }
        // No sleep while data flows: a small pipe otherwise throttles a 1MiB
        // writer past the unchanged deadline. Idle EOF waits remain bounded.
        if !stdout_progress && !stderr_progress {
            std::thread::sleep(IDLE.min(deadline.saturating_duration_since(Instant::now())));
        }
    }
}

pub(super) fn inspect(
    executable: &Path,
    leader: i32,
    observed: impl FnOnce(u32),
) -> io::Result<bool> {
    inspect_command(std::process::Command::new(executable), leader, observed)
}

#[cfg(test)]
pub(super) fn inspect_with_prefix(
    executable: &Path,
    prefix: &[String],
    leader: i32,
    observed: impl FnOnce(u32),
) -> io::Result<bool> {
    let mut command = std::process::Command::new(executable);
    command.args(prefix);
    inspect_command(command, leader, observed)
}

// Private command seam lets tests start a trusted interpreter directly. Production
// constructs only /bin/ps with no prefix; exact query/env/stdio remain shared here.
fn inspect_command(
    mut command: std::process::Command,
    leader: i32,
    observed: impl FnOnce(u32),
) -> io::Result<bool> {
    if leader <= 1 {
        return Err(framing("invalid owned inspection leader"));
    }
    let child = command
        // Order matters in the legacy negative control; production env_clear
        // leaves UNIX2003 enabled. -G is a real group, not a process group.
        .args(["-g", &leader.to_string(), "-o", "pid=,pgid=,stat="])
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let deadline = Instant::now() + BUDGET;
    let mut inspector = Inspector {
        child,
        unreaped: true,
        exit: None,
    };
    observed(inspector.child.id());
    let result = (|| {
        let stdout = inspector
            .child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("missing inspection stdout"))?;
        let stderr = inspector
            .child
            .stderr
            .take()
            .ok_or_else(|| io::Error::other("missing inspection stderr"))?;
        let frame = complete(
            &mut inspector,
            File::from(OwnedFd::from(stdout)),
            File::from(OwnedFd::from(stderr)),
            deadline,
        )?;
        if !frame.exit.success() {
            return Err(framing("native process inspection exit failed"));
        }
        if !frame.stderr.is_empty() {
            return Err(framing("native process inspection emitted diagnostics"));
        }
        let result = validate(&frame.stdout, leader);
        current(deadline)?;
        result
    })();
    if result.is_err() {
        // complete drops its nonblocking read endpoints before cleanup. This
        // mandatory owned wait is outside the observation budget, not hard bounded.
        inspector.cleanup()?;
    }
    result
}

fn positive(field: &str) -> io::Result<i32> {
    if field.is_empty() || !field.bytes().all(|b| b.is_ascii_digit()) {
        return Err(framing("invalid native inspection PID/group"));
    }
    field
        .parse::<i32>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| framing("invalid native inspection PID/group"))
}
fn zombie(state: &str) -> io::Result<bool> {
    let mut bytes = state.bytes();
    let primary = bytes
        .next()
        .ok_or_else(|| framing("missing native process state"))?;
    if !matches!(primary, b'I' | b'R' | b'S' | b'T' | b'U' | b'Z') {
        return Err(framing("unknown native process state"));
    }
    let suffix: Vec<_> = bytes.collect();
    let mut cursor = 0;
    if suffix.first().is_some_and(|v| matches!(v, b'<' | b'N')) {
        cursor += 1;
    }
    for flag in [b'X', b'E', b'V', b'L', b's', b'+'] {
        if suffix.get(cursor) == Some(&flag) {
            if flag == b'E' && primary == b'Z' {
                return Err(framing("invalid zombie process state suffix"));
            }
            cursor += 1;
        }
    }
    if cursor != suffix.len() {
        return Err(framing("invalid native process state suffix"));
    }
    Ok(primary == b'Z')
}

pub(super) fn validate(output: &[u8], leader: i32) -> io::Result<bool> {
    if leader <= 1 || output.is_empty() || output.last() != Some(&b'\n') {
        return Err(framing("incomplete native process inspection frame"));
    }
    let text =
        std::str::from_utf8(output).map_err(|_| framing("invalid native inspection UTF-8"))?;
    let mut pids = BTreeSet::new();
    let mut has_leader = false;
    let mut all_zombies = true;
    for row in text.lines().filter(|row| !row.trim().is_empty()) {
        let mut fields = row.split_whitespace();
        let pid = positive(fields.next().ok_or_else(|| framing("missing native PID"))?)?;
        let group = positive(
            fields
                .next()
                .ok_or_else(|| framing("missing native process group"))?,
        )?;
        let dead = zombie(
            fields
                .next()
                .ok_or_else(|| framing("missing native process state"))?,
        )?;
        if fields.next().is_some() || group != leader || !pids.insert(pid) {
            return Err(framing("unexpected/duplicate native inspection row"));
        }
        has_leader |= pid == leader;
        all_zombies &= dead;
    }
    if !has_leader {
        return Err(framing("native inspection expected leader missing"));
    }
    Ok(all_zombies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;

    fn builtin(body: &str) -> io::Result<bool> {
        inspect_with_prefix(
            Path::new("/bin/sh"),
            &["-c".into(), body.into()],
            42,
            |_| {},
        )
    }
    #[test]
    fn complete_frame_requires_exact_leader_all_members_and_canonical_states() {
        assert!(validate(b"42 42 Z\n43 42 ZNs+\n", 42).unwrap());
        assert!(!validate(b"42 42 Z\n43 42 S<XEVLs+\n", 42).unwrap());
        for frame in [
            &b""[..],
            b"\n",
            b"42 42 Z",
            b"43 42 Z\n",
            b"42 99 Z\n",
            b"42 42 Z\n42 42 Z\n",
            b"0 42 Z\n",
            b"+42 42 Z\n",
            b"42 42 H\n",
            b"42 42 ZN<\n",
            b"42 42 ZE\n",
            b"42 42 Zss\n",
            b"42 42 Z+X\n",
            b"42 42 Z extra\n",
            b"42 42 R\nmalformed\n",
            b"42 42 Z\n\xff\n",
        ] {
            assert!(
                validate(frame, 42).is_err(),
                "accepted malformed frame {frame:?}"
            );
        }
    }
    #[test]
    fn actual_inspector_rejects_exit_zero_stderr_empty_and_incomplete_frames() {
        for (name, body, category) in [
            (
                "diagnostic",
                "printf '42 42 Z\\n'; printf 'sysctl diagnostic' >&2",
                "diagnostics",
            ),
            ("empty", ":", "incomplete"),
            ("partial", "printf '42 42 Z'", "incomplete"),
            ("missing", "printf '43 42 Z\\n'", "leader missing"),
            ("exit", "printf '42 42 Z\\n'; exit 1", "exit failed"),
        ] {
            let error = builtin(body).unwrap_err();
            assert!(error.to_string().contains(category), "{name}: {error}");
        }
        assert!(builtin("printf '42 42 Z\\n'").unwrap());
    }
    #[test]
    fn each_output_stream_has_a_causal_size_failure() {
        let padding = " ".repeat(8192);
        let loop_body =
            format!("i=0; while [ \"$i\" -lt 130 ]; do printf '{padding}'; i=$((i+1)); done");
        for (name, body) in [
            (
                "stdout",
                format!("printf '42 42 Z'; {loop_body}; printf '\\n'"),
            ),
            (
                "stderr",
                format!("printf '42 42 Z\\n'; {{ {loop_body}; }} >&2"),
            ),
        ] {
            let started = Instant::now();
            let error = builtin(&body).unwrap_err();
            eprintln!(
                "isolated {name} overflow observation elapsed={:?} category={error}",
                started.elapsed()
            );
            assert!(
                error
                    .to_string()
                    .contains(&format!("{name} exceeds byte budget")),
                "{error}"
            );
        }
    }
    #[test]
    fn exited_inspector_retained_output_writer_is_unknown_and_watchdog_releases_mutants() {
        let (reader, retained_writer) = UnixStream::pair().unwrap();
        let output = retained_writer.try_clone().unwrap();
        let child = std::process::Command::new("/bin/sh")
            .args(["-c", "printf '42 42 Z\\n'"])
            .env_clear()
            .stdout(Stdio::from(OwnedFd::from(output)))
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let pid = rustix::process::Pid::from_raw(child.id() as i32).unwrap();
        loop {
            if rustix::process::waitid(
                rustix::process::WaitId::Pid(pid),
                rustix::process::WaitIdOptions::EXITED
                    | rustix::process::WaitIdOptions::NOWAIT
                    | rustix::process::WaitIdOptions::NOHANG,
            )
            .unwrap()
            .is_some()
            {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let (send, receive) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let mut inspector = Inspector {
                child,
                unreaped: true,
                exit: None,
            };
            let stderr = inspector.child.stderr.take().unwrap();
            let started = Instant::now();
            let result = complete(
                &mut inspector,
                File::from(OwnedFd::from(reader)),
                File::from(OwnedFd::from(stderr)),
                started + BUDGET,
            )
            .and_then(|frame| validate(&frame.stdout, 42));
            inspector.cleanup().unwrap();
            send.send((result, started.elapsed())).unwrap();
        });
        let before_release = receive.recv_timeout(Duration::from_secs(5));
        drop(retained_writer);
        thread.join().unwrap();
        // Cleanup and join precede the causal assertion, including hung mutations.
        let (result, elapsed) =
            before_release.expect("completion did not return before fixture release watchdog");
        eprintln!("retained output endpoint observation elapsed={elapsed:?}");
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
    }
    struct OwnedBoundary {
        leader: Option<super::super::ProcessGroup>,
        member: std::process::Child,
    }
    impl OwnedBoundary {
        async fn new() -> Self {
            use std::os::unix::process::CommandExt;
            let child = tokio::process::Command::new("/bin/cat")
                .env_clear()
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .process_group(0)
                .spawn()
                .unwrap();
            let mut leader = super::super::ProcessGroup::new(
                child,
                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            )
            .unwrap();
            let member = std::process::Command::new("/bin/sleep")
                .arg("30")
                .env_clear()
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .process_group(leader.pid.as_raw_nonzero().get())
                .spawn()
                .unwrap();
            drop(leader.child.stdin.take());
            let fixture = Self {
                leader: Some(leader),
                member,
            };
            tokio::time::timeout(
                Duration::from_secs(5),
                fixture.leader.as_ref().unwrap().observe_exit(),
            )
            .await
            .unwrap()
            .unwrap();
            fixture
        }
        async fn cleanup(&mut self) {
            let leader = self.leader.as_mut().unwrap();
            // Independent signal path: mutation of the tested resolver cannot
            // erase fixture ownership or make us reap the anchor first.
            leader.inspection_plan = None;
            let _ = rustix::process::kill_process_group(leader.pid, rustix::process::Signal::KILL);
            let _ = self.member.kill();
            self.member.wait().unwrap();
            leader.group_owned = false;
            leader.reap().await.unwrap();
            self.leader.take();
        }
    }
    impl Drop for OwnedBoundary {
        fn drop(&mut self) {
            if let Some(leader) = &mut self.leader {
                leader.inspection_plan = None;
                let _ =
                    rustix::process::kill_process_group(leader.pid, rustix::process::Signal::KILL);
                let _ = self.member.kill();
                let _ = self.member.wait();
                // ProcessGroup's ordinary final cleanup retains the unreaped
                // exact leader until Tokio's child reaper runs.
            }
        }
    }
    #[tokio::test]
    async fn actual_owned_zombie_leader_and_live_member_cannot_be_declared_dead() {
        let mut fixture = OwnedBoundary::new().await;
        let leader = fixture.leader.as_mut().unwrap();
        let pid = leader.pid.as_raw_nonzero().get();
        let selected = inspect(Path::new("/bin/ps"), pid, |_| {});
        leader.inspection_plan = Some(TestPlan::observe_only(false));
        let result = leader.kill_group();
        let retained = leader.group_owned;
        fixture.cleanup().await;
        assert!(
            retained,
            "a zombie leader alone cannot release group ownership"
        );
        assert_eq!(
            result.unwrap_err().raw_os_error(),
            Some(rustix::io::Errno::PERM.raw_os_error())
        );
        assert!(
            !selected.unwrap(),
            "live selected group member must be visible"
        );
    }
    #[tokio::test]
    async fn actual_legacy_mode_failure_keeps_owned_live_group_unknown() {
        let mut fixture = OwnedBoundary::new().await;
        let leader = fixture.leader.as_mut().unwrap();
        leader.inspection_plan = Some(TestPlan::observe_only(true));
        let result = leader.kill_group();
        let retained = leader.group_owned;
        fixture.cleanup().await;
        assert!(
            retained,
            "legacy selection cannot clear live group ownership"
        );
        assert!(result.unwrap_err().to_string().contains("exit failed"));
    }
    #[tokio::test]
    async fn actual_selected_group_excludes_other_owned_group_and_accepts_only_all_zombies() {
        let mut fixture = OwnedBoundary::new().await;
        let mut other = OwnedBoundary::new().await;
        let leader = fixture.leader.as_mut().unwrap();
        let pid = leader.pid.as_raw_nonzero().get();
        fixture.member.kill().unwrap();
        let member_pid = rustix::process::Pid::from_raw(fixture.member.id() as i32).unwrap();
        loop {
            if rustix::process::waitid(
                rustix::process::WaitId::Pid(member_pid),
                rustix::process::WaitIdOptions::EXITED
                    | rustix::process::WaitIdOptions::NOWAIT
                    | rustix::process::WaitIdOptions::NOHANG,
            )
            .unwrap()
            .is_some()
            {
                break;
            }
            tokio::task::yield_now().await;
        }
        let all_zombies = inspect(Path::new("/bin/ps"), pid, |_| {});
        let other_live = inspect(
            Path::new("/bin/ps"),
            other.leader.as_ref().unwrap().pid.as_raw_nonzero().get(),
            |_| {},
        );
        fixture.cleanup().await;
        other.cleanup().await;
        assert!(
            all_zombies.unwrap(),
            "other owned live group must be excluded"
        );
        assert!(!other_live.unwrap());
    }
}

/// Per-invocation macOS test evidence. No production/public configuration path.
#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) enum UnknownObservation {
    Diagnostics,
    Empty,
    MissingLeader,
    Malformed,
    Timeout,
    ExitFailure,
}
#[cfg(test)]
#[derive(Clone)]
pub(crate) struct TestPlan {
    mode: TestMode,
}
#[cfg(test)]
#[derive(Clone)]
enum TestMode {
    KillAndUnknown(UnknownObservation),
    ObserveOnly { legacy: bool },
}
#[cfg(test)]
impl TestPlan {
    pub(crate) fn unknown(observation: UnknownObservation) -> Self {
        Self {
            mode: TestMode::KillAndUnknown(observation),
        }
    }
    // Only this module's owned-boundary fixtures may construct a no-KILL plan.
    fn observe_only(legacy: bool) -> Self {
        Self {
            mode: TestMode::ObserveOnly { legacy },
        }
    }
    pub(super) fn signal(&self, pid: rustix::process::Pid) -> Result<(), rustix::io::Errno> {
        match self.mode {
            TestMode::ObserveOnly { .. } => Err(rustix::io::Errno::PERM),
            TestMode::KillAndUnknown(_) => {
                match rustix::process::kill_process_group(pid, rustix::process::Signal::KILL) {
                    Ok(()) | Err(rustix::io::Errno::SRCH | rustix::io::Errno::PERM) => {
                        Err(rustix::io::Errno::PERM)
                    }
                    error => error,
                }
            }
        }
    }
    pub(super) fn inspect(&self, leader: i32) -> io::Result<bool> {
        let body = match self.mode {
            TestMode::ObserveOnly { legacy: false } => "exec /bin/ps \"$0\" \"$@\"",
            TestMode::ObserveOnly { legacy: true } => {
                "COMMAND_MODE=legacy exec /bin/ps \"$0\" \"$@\""
            }
            TestMode::KillAndUnknown(UnknownObservation::Diagnostics) => {
                "printf '%s %s Z\\n' \"$1\" \"$1\"; printf 'fixture diagnostic' >&2"
            }
            TestMode::KillAndUnknown(UnknownObservation::Empty) => ":",
            TestMode::KillAndUnknown(UnknownObservation::MissingLeader) => {
                "printf '%s %s Z\\n' \"$(($1+1))\" \"$1\""
            }
            TestMode::KillAndUnknown(UnknownObservation::Malformed) => "printf 'malformed\\n'",
            TestMode::KillAndUnknown(UnknownObservation::Timeout) => "exec /bin/sleep 2",
            TestMode::KillAndUnknown(UnknownObservation::ExitFailure) => "exit 1",
        };
        inspect_with_prefix(
            Path::new("/bin/sh"),
            &["-c".into(), body.into()],
            leader,
            |_| {},
        )
    }
}
