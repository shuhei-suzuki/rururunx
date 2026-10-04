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

#[cfg(test)]
mod diagnostic_tests;
mod diagnostics;
use diagnostics::{
    Cleanup, Collector, Eof, Exit, Framing, Kill, Refusal, Site, StreamKind, Validation, framing,
};

const LIMIT: usize = 1024 * 1024;
const BUDGET: Duration = Duration::from_millis(250);
const IDLE: Duration = Duration::from_millis(5);

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
    // Private fixture safety survives facts removal/Drop during guard mutants.
    // It is set only after this exact Child actually returned Some from try_wait.
    #[cfg(test)]
    settled_fixture: Option<ExitStatus>,
}
impl Inspector {
    fn observe(&mut self, facts: &mut Collector) -> io::Result<()> {
        if self.exit.is_some() {
            return Ok(());
        }
        let status = facts.facts.status.get_or_insert_with(Default::default);
        status.calls = status.calls.saturating_add(1);
        let result = self.child.try_wait();
        #[cfg(test)]
        let mut result = result;
        #[cfg(test)]
        if facts.hooks.status_error_after_reap
            && let Ok(Some(exit)) = result
        {
            // Only an actual reaped Child can enter this synthetic error branch.
            facts.hooks.actual_reaped = Some(exit);
            self.settled_fixture = Some(exit);
            result = Err(io::Error::from(io::ErrorKind::PermissionDenied));
        }
        match result {
            Ok(Some(exit)) => {
                status.exit = Exit::observed(exit);
                self.unreaped = false;
                self.exit = Some(exit);
                Ok(())
            }
            Ok(None) => {
                status.pending = status.pending.saturating_add(1);
                Ok(())
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                status.interrupted = status.interrupted.saturating_add(1);
                Ok(())
            }
            Err(error) => {
                status.errors = status.errors.saturating_add(1);
                status.source_kind = Some(error.kind());
                self.unreaped = false;
                let error = io::Error::other("native inspector reap ownership uncertain");
                facts.fail(
                    Site::Status,
                    StreamKind::None,
                    Refusal::Io(error.kind()),
                    error.kind(),
                );
                Err(error)
            }
        }
    }
    fn cleanup(&mut self) -> io::Result<()> {
        self.cleanup_recorded(None)
    }
    fn cleanup_recorded(&mut self, mut facts: Option<&mut Collector>) -> io::Result<()> {
        if !self.unreaped {
            if let Some(facts) = &mut facts {
                facts.facts.cleanup = if self.exit.is_some() {
                    Cleanup::ReapedByStatus
                } else {
                    Cleanup::Relinquished
                };
            }
            return Ok(());
        }
        // The leader remains our unreaped Child even if it exited between polls.
        // A kill error must not skip mandatory wait; wait can resolve that race.
        #[cfg(test)]
        let already_reaped = self.settled_fixture;
        #[cfg(test)]
        let kill = if already_reaped.is_some() {
            if let Some(facts) = &mut facts {
                facts.hooks.synthetic_kills += 1;
            }
            Ok(())
        } else {
            self.child.kill()
        };
        #[cfg(not(test))]
        let kill = self.child.kill();
        if let Some(facts) = &mut facts {
            facts.facts.kill = match kill {
                Ok(()) => Kill::Ok,
                Err(ref e) => Kill::Error(e.kind()),
            };
        }
        let result = loop {
            #[cfg(test)]
            let wait = if let Some(exit) = already_reaped {
                if let Some(facts) = &mut facts {
                    facts.hooks.synthetic_waits += 1;
                }
                Ok(exit)
            } else {
                self.child.wait()
            };
            #[cfg(not(test))]
            let wait = self.child.wait();
            match wait {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                result => break result,
            }
        };
        self.unreaped = false;
        if let Some(facts) = &mut facts {
            facts.facts.cleanup = if result.is_ok() {
                Cleanup::KillThenReaped
            } else {
                Cleanup::WaitFailed
            };
        }
        result
            .map(|exit| self.exit = Some(exit))
            .map_err(|_| io::Error::other("native inspector cleanup/reap uncertain"))
    }
}
/// Both real observation failure and the settled-child synthetic unit fixture use
/// this exact helper. Cleanup timing never replaces the frozen observation facts.
fn cleanup_failure(inspector: &mut Inspector, facts: &mut Collector) -> io::Result<()> {
    let started = Instant::now();
    let result = inspector.cleanup_recorded(Some(facts));
    facts.facts.cleanup_us = Some(diagnostics::micros(started.elapsed()));
    result
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
    overflow: Framing,
}
impl Stream {
    fn new(file: File, overflow: Framing) -> io::Result<Self> {
        let flags = rustix::fs::fcntl_getfl(&file)?;
        rustix::fs::fcntl_setfl(&file, flags | rustix::fs::OFlags::NONBLOCK)?;
        Ok(Self {
            file,
            bytes: vec![],
            eof: false,
            overflow,
        })
    }
    #[cfg(test)]
    fn drain(&mut self, deadline: Instant) -> io::Result<bool> {
        self.drain_recorded(deadline, &mut Collector::new(), StreamKind::Stdout)
    }
    /// One bounded chunk lets the other stream, status and deadline progress.
    fn drain_recorded(
        &mut self,
        deadline: Instant,
        facts: &mut Collector,
        stream: StreamKind,
    ) -> io::Result<bool> {
        facts.deadline(deadline, Site::DrainDeadline, stream)?;
        if self.eof {
            return Ok(false);
        }
        let mut chunk = [0; 8192];
        let maximum = chunk.len().min(LIMIT + 1 - self.bytes.len());
        #[cfg(test)]
        let injected = facts.inject(Site::Read, stream).err();
        #[cfg(not(test))]
        let injected: Option<io::Error> = None;
        let read = injected.map_or_else(
            || {
                if let Some(s) = facts.stream(stream) {
                    s.calls = s.calls.saturating_add(1);
                }
                self.file.read(&mut chunk[..maximum])
            },
            Err,
        );
        match read {
            Ok(0) => {
                if let Some(s) = facts.stream(stream) {
                    s.eof = Eof::Observed;
                }
                self.eof = true;
                Ok(false)
            }
            Ok(count) => {
                if let Some(s) = facts.stream(stream) {
                    s.bytes = s.bytes.saturating_add(count as u64);
                    s.eof = Eof::Pending;
                }
                #[cfg(test)]
                facts.inject(Site::Allocation, stream)?;
                self.bytes.try_reserve(count).map_err(|_| {
                    let e = io::Error::other("native inspection allocation failed");
                    facts.fail(Site::Allocation, stream, Refusal::Allocation, e.kind());
                    e
                })?;
                self.bytes.extend_from_slice(&chunk[..count]);
                facts.deadline(deadline, Site::AfterReadDeadline, stream)?;
                if self.bytes.len() > LIMIT {
                    let e = framing(self.overflow);
                    facts.fail(
                        Site::Budget,
                        stream,
                        Refusal::Frame(self.overflow),
                        e.kind(),
                    );
                    Err(e)
                } else {
                    Ok(true)
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if let Some(s) = facts.stream(stream) {
                    s.would_block = s.would_block.saturating_add(1);
                    s.eof = Eof::Pending;
                }
                Ok(false)
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                if let Some(s) = facts.stream(stream) {
                    s.interrupted_read();
                }
                Ok(true)
            }
            Err(error) => {
                facts.fail(Site::Read, stream, Refusal::Io(error.kind()), error.kind());
                Err(error)
            }
        }
    }
}

struct Frame {
    #[cfg(test)]
    command: Option<(String, Vec<String>, usize)>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit: ExitStatus,
}
#[cfg(test)]
fn complete(
    inspector: &mut Inspector,
    stdout: File,
    stderr: File,
    deadline: Instant,
) -> io::Result<Frame> {
    complete_recorded(inspector, stdout, stderr, deadline, &mut Collector::new())
}
fn complete_recorded(
    inspector: &mut Inspector,
    stdout: File,
    stderr: File,
    deadline: Instant,
    facts: &mut Collector,
) -> io::Result<Frame> {
    // Both endpoints are available before either nonblocking setup is attempted.
    let _ = facts.stream(StreamKind::Stdout);
    let _ = facts.stream(StreamKind::Stderr);
    let mut setup = |file, overflow, stream| {
        #[cfg(test)]
        facts.inject(Site::Setup, stream)?;
        Stream::new(file, overflow)
            .inspect_err(|e| facts.fail(Site::Setup, stream, Refusal::Io(e.kind()), e.kind()))
    };
    let mut stdout = setup(stdout, Framing::StdoutBudget, StreamKind::Stdout)?;
    let mut stderr = setup(stderr, Framing::StderrBudget, StreamKind::Stderr)?;
    loop {
        facts.deadline(deadline, Site::LoopDeadline, StreamKind::None)?;
        let stdout_progress = stdout.drain_recorded(deadline, facts, StreamKind::Stdout)?;
        let stderr_progress = stderr.drain_recorded(deadline, facts, StreamKind::Stderr)?;
        inspector.observe(facts)?;
        facts.deadline(deadline, Site::AfterStatusDeadline, StreamKind::None)?;
        if stdout.eof
            && stderr.eof
            && let Some(exit) = inspector.exit
        {
            return Ok(Frame {
                #[cfg(test)]
                command: None,
                stdout: stdout.bytes,
                stderr: stderr.bytes,
                exit,
            });
        }
        // No sleep while data flows; keep the unchanged native polling boundary.
        if !stdout_progress && !stderr_progress {
            std::thread::sleep(facts.idle(deadline));
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
fn selected_command(mut command: std::process::Command, leader: i32) -> std::process::Command {
    // Order matters in legacy mode; production env_clear leaves UNIX2003 enabled.
    command.args(["-g", &leader.to_string(), "-o", "pid=,pgid=,stat="]);
    command
}
fn inspect_command(
    command: std::process::Command,
    leader: i32,
    observed: impl FnOnce(u32),
) -> io::Result<bool> {
    inspect_command_recorded(command, leader, observed, Collector::new())
}
fn inspect_command_recorded(
    command: std::process::Command,
    leader: i32,
    observed: impl FnOnce(u32),
    mut facts: Collector,
) -> io::Result<bool> {
    if leader <= 1 {
        let e = framing(Framing::InvalidLeader);
        facts.fail(
            Site::Input,
            StreamKind::None,
            Refusal::Frame(Framing::InvalidLeader),
            e.kind(),
        );
        return Err(facts.attach(e));
    }
    let (frame, deadline, mut facts) =
        observe_command_recorded(selected_command(command, leader), observed, facts)?;
    if !frame.exit.success() {
        let e = framing(Framing::Exit);
        facts.fail(
            Site::Exit,
            StreamKind::None,
            Refusal::Frame(Framing::Exit),
            e.kind(),
        );
        return Err(facts.attach(e));
    }
    if !frame.stderr.is_empty() {
        let e = framing(Framing::Stderr);
        facts.fail(
            Site::Stderr,
            StreamKind::Stderr,
            Refusal::Frame(Framing::Stderr),
            e.kind(),
        );
        return Err(facts.attach(e));
    }
    let result = validate(&frame.stdout, leader);
    facts.facts.validation = match &result {
        Ok(true) => Validation::Dead,
        Ok(false) => Validation::Live,
        Err(e) => Validation::Refused(diagnostics::refusal(e)),
    };
    // This deadline masks the validation result exactly as before. Freeze only the
    // actually returned error, keeping validation separately for diagnosis.
    if let Err(e) = facts.deadline(deadline, Site::AfterValidationDeadline, StreamKind::None) {
        return Err(facts.attach(e));
    }
    result.map_err(|e| {
        facts.fail(
            Site::Validation,
            StreamKind::None,
            diagnostics::refusal(&e),
            e.kind(),
        );
        facts.attach(e)
    })
}
#[cfg(test)]
fn observe_command(
    command: std::process::Command,
    observed: impl FnOnce(u32),
) -> io::Result<(Frame, Instant)> {
    observe_command_recorded(command, observed, Collector::new())
        .map(|(frame, deadline, _)| (frame, deadline))
}
// Frame collection owns the identical stdio, environment, budget and cleanup boundary.
fn observe_command_recorded(
    mut command: std::process::Command,
    observed: impl FnOnce(u32),
    mut facts: Collector,
) -> io::Result<(Frame, Instant, Collector)> {
    command.env_clear();
    #[cfg(test)]
    let command_metadata = (
        command.get_program().to_string_lossy().into_owned(),
        command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect(),
        command.get_envs().count(),
    );
    let spawn_started = Instant::now();
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    // Start the same post-spawn deadline before recording spawn timing.
    let spawned_at = Instant::now();
    let deadline = spawned_at + BUDGET;
    facts.started = Some(deadline - BUDGET);
    facts.facts.spawn_us = Some(diagnostics::micros(
        spawned_at.duration_since(spawn_started),
    ));
    let child = match child {
        Ok(child) => child,
        Err(e) => {
            facts.started = None;
            facts.fail(
                Site::Spawn,
                StreamKind::None,
                Refusal::Io(e.kind()),
                e.kind(),
            );
            return Err(facts.attach(e));
        }
    };
    let mut inspector = Inspector {
        child,
        unreaped: true,
        exit: None,
        #[cfg(test)]
        settled_fixture: None,
    };
    observed(inspector.child.id());
    let result = (|| {
        #[cfg(test)]
        facts.inject(Site::Endpoint, StreamKind::Stdout)?;
        let stdout = inspector.child.stdout.take().ok_or_else(|| {
            let e = io::Error::other("missing inspection stdout");
            facts.fail(
                Site::Endpoint,
                StreamKind::Stdout,
                Refusal::MissingEndpoint,
                e.kind(),
            );
            e
        })?;
        let _ = facts.stream(StreamKind::Stdout);
        #[cfg(test)]
        facts.inject(Site::Endpoint, StreamKind::Stderr)?;
        let stderr = inspector.child.stderr.take().ok_or_else(|| {
            let e = io::Error::other("missing inspection stderr");
            facts.fail(
                Site::Endpoint,
                StreamKind::Stderr,
                Refusal::MissingEndpoint,
                e.kind(),
            );
            e
        })?;
        let _ = facts.stream(StreamKind::Stderr);
        complete_recorded(
            &mut inspector,
            File::from(OwnedFd::from(stdout)),
            File::from(OwnedFd::from(stderr)),
            deadline,
            &mut facts,
        )
    })();
    match result {
        Ok(frame) => {
            facts.facts.cleanup = Cleanup::ReapedByStatus;
            #[cfg(test)]
            let frame = {
                let mut frame = frame;
                frame.command = Some(command_metadata);
                frame
            };
            Ok((frame, deadline, facts))
        }
        Err(original) => {
            // Endpoints drop before mandatory wait, outside the observation budget.
            // Facts freeze at the first failure and do not promote cleanup authority.
            let _ = cleanup_failure(&mut inspector, &mut facts);
            Err(facts.attach(original))
        }
    }
}

fn positive(field: &str) -> io::Result<i32> {
    if field.is_empty() || !field.bytes().all(|b| b.is_ascii_digit()) {
        return Err(framing(Framing::Identifier));
    }
    field
        .parse::<i32>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| framing(Framing::Identifier))
}
fn zombie(state: &str) -> io::Result<bool> {
    let mut bytes = state.bytes();
    let primary = bytes.next().ok_or_else(|| framing(Framing::MissingState))?;
    if !matches!(primary, b'I' | b'R' | b'S' | b'T' | b'U' | b'Z') {
        return Err(framing(Framing::UnknownState));
    }
    let suffix: Vec<_> = bytes.collect();
    let mut cursor = 0;
    if suffix.first().is_some_and(|v| matches!(v, b'<' | b'N')) {
        cursor += 1;
    }
    for flag in [b'X', b'E', b'V', b'L', b's', b'+'] {
        if suffix.get(cursor) == Some(&flag) {
            if flag == b'E' && primary == b'Z' {
                return Err(framing(Framing::ZombieSuffix));
            }
            cursor += 1;
        }
    }
    if cursor != suffix.len() {
        return Err(framing(Framing::StateSuffix));
    }
    Ok(primary == b'Z')
}

pub(super) fn validate(output: &[u8], leader: i32) -> io::Result<bool> {
    if leader <= 1 || output.is_empty() || output.last() != Some(&b'\n') {
        return Err(framing(Framing::Incomplete));
    }
    let text = std::str::from_utf8(output).map_err(|_| framing(Framing::Utf8))?;
    let mut pids = BTreeSet::new();
    let mut has_leader = false;
    let mut all_zombies = true;
    for row in text.lines().filter(|row| !row.trim().is_empty()) {
        let mut fields = row.split_whitespace();
        let pid = positive(fields.next().ok_or_else(|| framing(Framing::MissingPid))?)?;
        let group = positive(
            fields
                .next()
                .ok_or_else(|| framing(Framing::MissingGroup))?,
        )?;
        let dead = zombie(
            fields
                .next()
                .ok_or_else(|| framing(Framing::MissingRowState))?,
        )?;
        if fields.next().is_some() || group != leader || !pids.insert(pid) {
            return Err(framing(Framing::UnexpectedRow));
        }
        has_leader |= pid == leader;
        all_zombies &= dead;
    }
    if !has_leader {
        return Err(framing(Framing::MissingLeader));
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
            if name == "empty" || name == "partial" {
                assert!(
                    error.to_string().starts_with(Framing::Incomplete.message()),
                    "{name}: {error}"
                );
            }
        }
        assert!(builtin("printf '42 42 Z\\n'").unwrap());
    }
    #[test]
    fn oversized_child_streams_remain_unknown_when_size_or_deadline_wins() {
        for (name, body) in [
            (
                "stdout",
                "exec /usr/bin/awk 'BEGIN {printf \"42 42 Z\"; printf \"%1048577s\", \"\"; printf \"\\n\"; exit}'",
            ),
            (
                "stderr",
                "printf '42 42 Z\\n'; exec /usr/bin/awk 'BEGIN {printf \"%1048577s\", \"\"; exit}' >&2",
            ),
        ] {
            let started = Instant::now();
            let error = builtin(body).unwrap_err();
            eprintln!(
                "isolated {name} overflow observation elapsed={:?} category={error}",
                started.elapsed()
            );
            // Process scheduling can exhaust the unchanged observation budget
            // before the producer fills the pipe. This OS-wrapper control proves
            // no accepted frame, not a causal byte-cap kill in that case.
            assert!(
                error.kind() == io::ErrorKind::TimedOut
                    || (error.kind() == io::ErrorKind::InvalidData
                        && error
                            .to_string()
                            .contains(&format!("{name} exceeds byte budget"))),
                "{error}"
            );
        }
    }
    #[test]
    fn prepared_stream_cap_rejects_complete_zombie_prefix_before_hidden_live_row() {
        let directory = tempfile::tempdir().unwrap();
        for name in ["stdout", "stderr"] {
            let path = directory.path().join(name);
            // The next real read ends a complete all-Z row exactly at LIMIT+1.
            // Omitting the byte-cap check would accept this prefix and hide the
            // following live row on its next zero-length read. The prepared bytes
            // are a reader-boundary unit fixture, not an actual ps/pipe sample.
            std::fs::write(&path, b"\n43 42 S\n").unwrap();
            let category = if name == "stdout" {
                "native inspection stdout exceeds byte budget"
            } else {
                "native inspection stderr exceeds byte budget"
            };
            let mut stream = Stream::new(
                File::open(&path).unwrap(),
                if name == "stdout" {
                    Framing::StdoutBudget
                } else {
                    Framing::StderrBudget
                },
            )
            .unwrap();
            stream.bytes = b"42 42 Z".to_vec();
            stream.bytes.resize(LIMIT, b' ');
            let result = stream.drain(Instant::now() + BUDGET);
            // The cut is otherwise valid death evidence. It must be refused by
            // the actual drain cap, independently of framing/diagnostic barriers.
            assert!(validate(&stream.bytes, 42).unwrap());
            assert!(!stream.eof);
            let error = result.expect_err("reader accepted a truncated all-Z prefix");
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
            assert_eq!(error.to_string(), category);
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
                #[cfg(test)]
                settled_fixture: None,
            };
            let stderr = inspector.child.stderr.take().unwrap();
            let started = Instant::now();
            let mut facts = Collector::new();
            facts.started = Some(started);
            let result = complete_recorded(
                &mut inspector,
                File::from(OwnedFd::from(reader)),
                File::from(OwnedFd::from(stderr)),
                started + BUDGET,
                &mut facts,
            )
            .and_then(|frame| validate(&frame.stdout, 42));
            cleanup_failure(&mut inspector, &mut facts).unwrap();
            send.send((result, started.elapsed(), facts.facts)).unwrap();
        });
        let before_release = receive.recv_timeout(Duration::from_secs(5));
        drop(retained_writer);
        thread.join().unwrap();
        // Cleanup and join precede the causal assertion, including hung mutations.
        let (result, elapsed, facts) =
            before_release.expect("completion did not return before fixture release watchdog");
        eprintln!("retained output endpoint observation elapsed={elapsed:?}");
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
        assert!(facts.site.is_some_and(|site| matches!(
            site,
            Site::LoopDeadline
                | Site::DrainDeadline
                | Site::AfterReadDeadline
                | Site::AfterStatusDeadline
        )));
        let stdout = facts.stdout.unwrap();
        assert_ne!(stdout.eof, Eof::Observed, "fixture still owns its writer");
        if stdout.bytes > 0 {
            assert_eq!(stdout.eof, Eof::Pending);
        }
        if let Some(status) = facts.status {
            assert!(status.calls > 0);
            if status.exit == Exit::Success {
                assert_eq!(facts.cleanup, Cleanup::ReapedByStatus);
            }
        }
        // Scheduling may expire before status/stderr/bytes are reached; no exact
        // EOF, throughput or cleanup classification is invented in that case.
    }
    #[test]
    fn actual_read_error_reaps_the_owned_inspector_without_accepting_a_frame() {
        let directory = tempfile::tempdir().unwrap();
        let child = std::process::Command::new("/bin/sh")
            .args(["-c", ":"])
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut inspector = Inspector {
            child,
            unreaped: true,
            exit: None,
            #[cfg(test)]
            settled_fixture: None,
        };
        let stderr = inspector.child.stderr.take().unwrap();
        // A safe owned directory FD exercises the actual read syscall's error
        // path. It does not fake a successful fcntl/read or signal another PID.
        let result = complete(
            &mut inspector,
            File::open(directory.path()).unwrap(),
            File::from(OwnedFd::from(stderr)),
            Instant::now() + BUDGET,
        );
        inspector.cleanup().unwrap();
        assert!(!inspector.unreaped);
        assert!(inspector.exit.is_some());
        assert_eq!(
            result.err().unwrap().raw_os_error(),
            Some(rustix::io::Errno::ISDIR.raw_os_error())
        );
    }
    fn selected_sample(leader: i32, legacy: bool) -> io::Result<Frame> {
        let mut command = if legacy {
            let mut command = std::process::Command::new("/bin/sh");
            command.args([
                "-c",
                "exec /usr/bin/env -i COMMAND_MODE=legacy /bin/ps \"$0\" \"$@\"",
            ]);
            command
        } else {
            std::process::Command::new("/bin/ps")
        };
        command = selected_command(command, leader);
        observe_command(command, |_| {}).map(|(frame, _)| frame)
    }
    fn global_sample() -> io::Result<Frame> {
        let mut command = std::process::Command::new("/bin/ps");
        // Diagnostic fixed argv is independent of the mutated production selector.
        command.args(["-A", "-o", "pid=,pgid=,stat="]);
        observe_command(command, |_| {}).map(|(frame, _)| frame)
    }
    fn record_membership(
        label: &str,
        selected: io::Result<Frame>,
        global: io::Result<Frame>,
        leader: i32,
        member: u32,
    ) {
        let selected = selected.unwrap();
        let global = global.unwrap();
        assert_eq!(selected.exit.code(), Some(0));
        assert!(selected.stderr.is_empty());
        assert_eq!(global.exit.code(), Some(0));
        assert!(global.stderr.is_empty());
        let rows = |frame: &Frame| -> Vec<(i32, i32, String)> {
            String::from_utf8(frame.stdout.clone())
                .unwrap()
                .lines()
                .map(|line| {
                    let fields: Vec<_> = line.split_whitespace().collect();
                    assert_eq!(fields.len(), 3);
                    (
                        positive(fields[0]).unwrap(),
                        positive(fields[1]).unwrap(),
                        fields[2].to_owned(),
                    )
                })
                .collect()
        };
        let selected_rows = rows(&selected);
        let global_owned: Vec<_> = rows(&global)
            .into_iter()
            .filter(|row| row.1 == leader)
            .collect();
        let pids =
            |rows: &[(i32, i32, String)]| rows.iter().map(|row| row.0).collect::<BTreeSet<_>>();
        let expected = BTreeSet::from([leader, i32::try_from(member).unwrap()]);
        assert_eq!(pids(&selected_rows), expected);
        assert_eq!(pids(&global_owned), expected);
        validate(&selected.stdout, leader).unwrap();
        let command = selected.command.as_ref().unwrap();
        assert_eq!(command.0, "/bin/ps");
        assert_eq!(
            command.1,
            [
                "-g".to_owned(),
                leader.to_string(),
                "-o".to_owned(),
                "pid=,pgid=,stat=".to_owned()
            ]
        );
        assert_eq!(command.2, 0);
        // Print only the fixture-owned rows; never dump unrelated global metadata.
        eprintln!(
            "owned_ps_observation={}",
            serde_json::json!({"label":label,"environment":"env_clear; COMMAND_MODE absent","argv":command.1,"program":command.0,"explicit_environment_entries":command.2,"exit_code":selected.exit.code(),"stderr_bytes":selected.stderr.len(),"selected_rows":selected_rows,"global_filtered_pids":pids(&global_owned)})
        );
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
        let sample = selected_sample(pid, false);
        let global = global_sample();
        let member = fixture.member.id();
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
        record_membership("zombie-leader-live-member", sample, global, pid, member);
    }
    #[tokio::test]
    async fn actual_legacy_mode_failure_keeps_owned_live_group_unknown() {
        let mut fixture = OwnedBoundary::new().await;
        let leader = fixture.leader.as_mut().unwrap();
        let pid = leader.pid.as_raw_nonzero().get();
        let sample = selected_sample(pid, true);
        leader.inspection_plan = Some(TestPlan::observe_only(true));
        let result = leader.kill_group();
        let retained = leader.group_owned;
        fixture.cleanup().await;
        assert!(
            retained,
            "legacy selection cannot clear live group ownership"
        );
        assert!(result.unwrap_err().to_string().contains("exit failed"));
        let sample = sample.unwrap();
        assert_eq!(sample.exit.code(), Some(1));
        assert!(!sample.stderr.is_empty());
        eprintln!(
            "owned_ps_observation={}",
            serde_json::json!({"label":"legacy-rejected","environment":"only COMMAND_MODE=legacy","argv":["-g",pid.to_string(),"-o","pid=,pgid=,stat="],"outer_command":sample.command,"exit_code":sample.exit.code(),"stderr_bytes":sample.stderr.len()})
        );
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
        let sample = selected_sample(pid, false);
        let global = global_sample();
        let member = fixture.member.id();
        let other_pid = other.leader.as_ref().unwrap().pid.as_raw_nonzero().get();
        let other_member = other.member.id();
        let other_sample = selected_sample(other_pid, false);
        let other_global = global_sample();
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
        record_membership("all-zombie-members", sample, global, pid, member);
        record_membership(
            "excluded-other-live-group",
            other_sample,
            other_global,
            other_pid,
            other_member,
        );
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
    Partial,
}
#[cfg(test)]
#[derive(Clone)]
pub(crate) struct TestPlan {
    mode: TestMode,
    // Independent original-kind test observation, before AdapterError erases it.
    original_kind: std::sync::Arc<std::sync::Mutex<Option<io::ErrorKind>>>,
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
            original_kind: Default::default(),
        }
    }
    // Only this module's owned-boundary fixtures may construct a no-KILL plan.
    fn observe_only(legacy: bool) -> Self {
        Self {
            mode: TestMode::ObserveOnly { legacy },
            original_kind: Default::default(),
        }
    }
    pub(crate) fn original_error_kind(&self) -> Option<io::ErrorKind> {
        self.original_kind.lock().ok().and_then(|kind| *kind)
    }
    pub(crate) fn assert_diagnostics_transport(&self, diagnostic: &str) {
        assert!(diagnostic.contains("inspection_facts{"), "{diagnostic}");
        match self
            .original_error_kind()
            .expect("real inspector did not run")
        {
            io::ErrorKind::InvalidData => {
                assert!(
                    diagnostic.contains("site=stderr_guard,stream=stderr,refusal=stderr_nonempty"),
                    "{diagnostic}"
                );
                for fact in [
                    "stdout_eof=observed",
                    "stderr_eof=observed",
                    "stderr_bytes=18,",
                    "exit=success",
                    "validation=not_reached",
                    "cleanup=reaped_by_status_observation",
                    "kill=not_requested",
                ] {
                    assert!(diagnostic.contains(fact), "{fact}: {diagnostic}");
                }
            }
            io::ErrorKind::TimedOut => assert!(
                diagnostic.contains("site=deadline_") && diagnostic.contains("refusal=deadline"),
                "{diagnostic}"
            ),
            kind => panic!("unexpected original inspector kind {kind:?}: {diagnostic}"),
        }
        let count = |key: &str| -> Option<u64> {
            diagnostic
                .split_once(&format!("{key}="))?
                .1
                .split([',', '}'])
                .next()?
                .parse()
                .ok()
        };
        for stream in ["stdout", "stderr"] {
            if let Some(reads) = count(&format!("{stream}_reads")) {
                let would_block = count(&format!("{stream}_would_block")).unwrap();
                let interrupted = count(&format!("{stream}_interrupted")).unwrap();
                let eof = u64::from(diagnostic.contains(&format!("{stream}_eof=observed")));
                assert!(reads >= would_block + interrupted + eof, "{diagnostic}");
            }
        }
        if let Some(polls) = count("status_polls") {
            assert!(polls > 0, "{diagnostic}");
        }
        assert!(
            !diagnostic.contains("fixture diagnostic"),
            "raw child stderr leaked"
        );
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
    pub(super) fn inspect(&self, leader: i32) -> Option<io::Result<bool>> {
        let body = match self.mode {
            TestMode::ObserveOnly { legacy: false } => return None,
            TestMode::ObserveOnly { legacy: true } => {
                "exec /usr/bin/env -i COMMAND_MODE=legacy /bin/ps \"$0\" \"$@\""
            }
            TestMode::KillAndUnknown(UnknownObservation::Diagnostics) => {
                "printf '%s %s Z\\n' \"$1\" \"$1\"; printf 'fixture diagnostic' >&2"
            }
            TestMode::KillAndUnknown(UnknownObservation::Empty) => ":",
            TestMode::KillAndUnknown(UnknownObservation::MissingLeader) => {
                "printf '%s %s Z\\n' \"$(($1+1))\" \"$1\""
            }
            TestMode::KillAndUnknown(UnknownObservation::Malformed) => "printf 'malformed\\n'",
            TestMode::KillAndUnknown(UnknownObservation::Timeout) => {
                "printf '%s %s Z\\n' \"$1\" \"$1\"; exec /bin/sleep 2"
            }
            TestMode::KillAndUnknown(UnknownObservation::ExitFailure) => {
                "printf '%s %s Z\\n' \"$1\" \"$1\"; exit 1"
            }
            TestMode::KillAndUnknown(UnknownObservation::Partial) => {
                "printf '%s %s Z' \"$1\" \"$1\""
            }
        };
        let result = inspect_with_prefix(
            Path::new("/bin/sh"),
            &["-c".into(), body.into()],
            leader,
            |_| {},
        );
        if let Ok(mut kind) = self.original_kind.lock()
            && kind.is_none()
        {
            *kind = result.as_ref().err().map(io::Error::kind);
        }
        Some(result)
    }
}
