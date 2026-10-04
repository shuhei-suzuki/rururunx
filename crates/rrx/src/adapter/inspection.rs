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
        let result = self.child.wait();
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
    if leader <= 1 {
        return Err(framing("invalid owned inspection leader"));
    }
    let child = std::process::Command::new(executable)
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
