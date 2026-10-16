//! Bounded S Version/Claude initialize capture; no Unit, quota or model input.
use super::*;
use anyhow::Context;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Read,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};
// Allows large native CLI distributions while bounding each identity observation.
const PROGRAM_BYTES: u64 = 512 * 1024 * 1024;
fn interruption(cancelled: &AtomicBool, deadline: Instant) -> Option<StartupRefusal> {
    if cancelled.load(Ordering::SeqCst) {
        Some(StartupRefusal::Stopped)
    } else if Instant::now() >= deadline {
        Some(StartupRefusal::Deadline)
    } else {
        None
    }
}
#[derive(Debug, Serialize, PartialEq, Eq)]
pub(super) struct Program {
    path: PathBuf,
    dev: u64,
    ino: u64,
    size: u64,
    mtime: (i64, i64),
    ctime: (i64, i64),
    sha256: String,
}
impl Program {
    pub(super) async fn read(
        path: &Path,
        cancelled: Arc<AtomicBool>,
        deadline: Instant,
    ) -> std::result::Result<Self, StartupRefusal> {
        if let Some(reason) = interruption(&cancelled, deadline) {
            return Err(reason);
        }
        let path = path.to_path_buf();
        let stop = cancelled.clone();
        let mut task = tokio::task::spawn_blocking(move || Self::read_file(&path, &stop, deadline));
        let mut tick = tokio::time::interval(Duration::from_millis(10));
        loop {
            if let Some(reason) = interruption(&cancelled, deadline) {
                task.abort();
                return Err(reason);
            }
            tokio::select! { biased;
                result = &mut task => {
                    if let Some(reason) = interruption(&cancelled, deadline) {
                        return Err(reason);
                    }
                    return result.ok().and_then(Result::ok).ok_or(StartupRefusal::ProgramIdentity);
                },
                _ = tick.tick() => {},
            }
        }
    }
    fn read_file(path: &Path, cancelled: &AtomicBool, deadline: Instant) -> Result<Self> {
        let path = path.canonicalize()?;
        // NONBLOCK prevents opening a FIFO from waiting for a writer; fstat is
        // checked before reading any bytes. NOFOLLOW closes the final symlink swap.
        let mut file = File::from(rustix::fs::open(
            &path,
            rustix::fs::OFlags::RDONLY
                | rustix::fs::OFlags::NONBLOCK
                | rustix::fs::OFlags::NOCTTY
                | rustix::fs::OFlags::NOFOLLOW
                | rustix::fs::OFlags::CLOEXEC,
            rustix::fs::Mode::empty(),
        )?);
        let m = file.metadata()?;
        ensure!(m.is_file(), "startup program is not a file");
        ensure!(m.len() <= PROGRAM_BYTES, "startup program over size bound");
        let mut h = Sha256::new();
        let mut bytes = [0u8; 65536];
        let mut total = 0u64;
        loop {
            ensure!(
                interruption(cancelled, deadline).is_none(),
                "startup identity interrupted"
            );
            let n = file.read(&mut bytes)?;
            if n == 0 {
                break;
            }
            total += n as u64;
            ensure!(total <= m.len(), "startup program grew during hashing");
            h.update(&bytes[..n]);
        }
        ensure!(total == m.len(), "startup program shrank during hashing");
        let identity = Self {
            path,
            dev: m.dev(),
            ino: m.ino(),
            size: m.len(),
            mtime: (m.mtime(), m.mtime_nsec()),
            ctime: (m.ctime(), m.ctime_nsec()),
            sha256: format!("{:x}", h.finalize()),
        };
        identity.stat()?;
        Ok(identity)
    }
    fn stat(&self) -> Result<()> {
        let m = self.path.metadata()?;
        ensure!(
            m.is_file()
                && (
                    m.dev(),
                    m.ino(),
                    m.len(),
                    m.mtime(),
                    m.mtime_nsec(),
                    m.ctime(),
                    m.ctime_nsec()
                ) == (
                    self.dev,
                    self.ino,
                    self.size,
                    self.mtime.0,
                    self.mtime.1,
                    self.ctime.0,
                    self.ctime.1
                ),
            "startup executable stat changed"
        );
        Ok(())
    }
    async fn verify(
        &self,
        cancelled: Arc<AtomicBool>,
        deadline: Instant,
    ) -> std::result::Result<(), StartupRefusal> {
        if Self::read(&self.path, cancelled, deadline).await? == *self {
            Ok(())
        } else {
            Err(StartupRefusal::ProgramIdentity)
        }
    }
}
#[derive(Serialize)]
pub(super) struct Observation {
    pub(super) known: bool,
    pub(super) pass: bool,
    pub(super) reason: Option<StartupRefusal>,
    protocol: bool,
    stdout_eof: bool,
    stderr_eof: bool,
    leader_reaped: bool,
    signal: bool,
    group_hygiene: bool,
    attempted: bool,
    exit: Option<i32>,
    bytes: usize,
    version: Option<String>,
}
impl Observation {
    pub(super) fn validate_body(body: &Value) -> Result<()> {
        // Every serialized field is at its largest possible representation:
        // false, longest refusal, signed exit, usize bytes, and legal version.
        let largest = Self {
            known: false,
            pass: false,
            reason: Some(StartupRefusal::CodexHandshakeUnavailable),
            protocol: false,
            stdout_eof: false,
            stderr_eof: false,
            leader_reaped: false,
            signal: false,
            group_hygiene: false,
            attempted: false,
            exit: Some(i32::MIN),
            bytes: usize::MAX,
            version: Some("v".repeat(96)),
        };
        let mut settled = body.clone();
        settled["observation"] = serde_json::to_value(largest)?;
        ensure!(
            serde_json::to_vec(body)?.len() <= 4096 && serde_json::to_vec(&settled)?.len() <= 4096,
            "startup intent lacks settlement headroom"
        );
        Ok(())
    }
    pub(super) fn not_spawned(reason: StartupRefusal) -> Self {
        Self {
            known: true,
            pass: false,
            reason: Some(reason),
            protocol: false,
            stdout_eof: false,
            stderr_eof: false,
            leader_reaped: false,
            signal: false,
            group_hygiene: false,
            attempted: false,
            exit: None,
            bytes: 0,
            version: None,
        }
    }
}
fn version(bytes: &[u8]) -> Result<String> {
    ensure!(bytes.len() <= 256, "startup version over bound");
    let text = std::str::from_utf8(bytes)?
        .strip_suffix('\n')
        .unwrap_or(std::str::from_utf8(bytes)?);
    let text = text.strip_suffix('\r').unwrap_or(text);
    ensure!(
        !text.is_empty() && !text.chars().any(char::is_control),
        "startup version is not a single line"
    );
    if text.len() <= 96
        && text
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b" ._()+/:-".contains(&c))
    {
        Ok(text.into())
    } else {
        Ok(format!(
            "nonrecordable:{}",
            crate::execution::native_result::digest(text.as_bytes())
        ))
    }
}

fn spawn(
    startup: &Startup,
    plan: &AliasPlan,
    child: &ChildCustody,
    command: Command,
    deadline: Instant,
) -> std::result::Result<crate::execution::process::RetainedPipes, Observation> {
    let spawned = (|| -> std::result::Result<_, StartupRefusal> {
        plan.directory
            .verify_empty()
            .map_err(|_| StartupRefusal::PrivatePath)?;
        plan.program
            .stat()
            .map_err(|_| StartupRefusal::ProgramIdentity)?;
        if let Some(reason) = interruption(&startup.cancelled, deadline) {
            return Err(reason);
        }
        #[cfg(test)]
        tests::STORE_WAIT_OBSERVER.with(|observer| {
            if let Some(waiting) = observer.borrow_mut().take() {
                assert!(matches!(
                    startup.owner.store.try_lock(),
                    Err(std::sync::TryLockError::WouldBlock)
                ));
                waiting.send(deadline).unwrap();
            }
        });
        // Check the live owner, never a saved Task or receipt.
        startup
            .owner
            .store
            .lock()
            .map_err(|_| StartupRefusal::Journal)?
            .old_startup_unknown(&startup.owner)
            .map_err(|_| StartupRefusal::Journal)?;
        // Revoke stores the flag before taking this slot lock, so a stop or
        // deadline during the Store wait is seen here, and a later revoke
        // waits for adoption and then signals the adopted group.
        let mut raw = child.raw.lock().unwrap();
        if let Some(reason) = interruption(&startup.cancelled, deadline) {
            return Err(reason);
        }
        let process =
            rrx_spawn_hygiene::spawn_tokio(command).map_err(|_| StartupRefusal::Protocol)?;
        raw.adopt(process);
        raw.qualify().map_err(|_| StartupRefusal::Protocol)?;
        raw.pipes().map_err(|_| StartupRefusal::Protocol)
    })();
    spawned.map_err(|reason| {
        child.revoke();
        let mut observation = Observation::not_spawned(reason);
        observation.known = !child.raw.lock().unwrap().has_child();
        observation
    })
}

pub(super) async fn run(
    startup: &Arc<Startup>,
    plan: &AliasPlan,
    step: &str,
    checked: &DecoderChecked,
    parent: Instant,
    aggregate: &mut usize,
) -> Observation {
    if let Some(reason) = interruption(&startup.cancelled, parent) {
        return Observation::not_spawned(reason);
    }
    if *aggregate >= 8 * 1024 * 1024 {
        return Observation::not_spawned(StartupRefusal::Budget);
    }
    if plan.directory.verify_empty().is_err() {
        return Observation::not_spawned(StartupRefusal::PrivatePath);
    }
    if let Err(reason) = plan.program.verify(startup.cancelled.clone(), parent).await {
        return Observation::not_spawned(reason);
    }
    let handshake = step == "handshake";
    let native = uuid::Uuid::new_v4().to_string();
    let request = uuid::Uuid::new_v4().to_string();
    let mut decoder =
        crate::execution::claude_wire::StartupClaude::new(checked, native.clone(), request);
    let argv = if handshake {
        match crate::execution::native::prepared::command_argv(
            "claude",
            crate::domain::SessionRole::Reviewer,
            plan.model.as_deref(),
            plan.effort.as_deref(),
            uuid::Uuid::parse_str(&native).ok(),
        ) {
            Ok(argv) if argv.windows(2).any(|a| a == ["--permission-mode", "plan"]) => argv,
            _ => return Observation::not_spawned(StartupRefusal::Protocol),
        }
    } else {
        vec!["--version".into()]
    };
    let mut command = Command::new(&plan.program.path);
    command
        .args(argv)
        .current_dir(&plan.directory.path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .kill_on_drop(true);
    let child = Arc::new(ChildCustody {
        raw: Mutex::new(RetainedRawProcess::default()),
    });
    // The slot exists before spawn, and adoption precedes identity and pipes.
    startup.children.lock().unwrap().push(child.clone());
    let deadline =
        parent.min(Instant::now() + Duration::from_secs(if handshake { 90 } else { 30 }));
    let pipes = match spawn(startup, plan, &child, command, deadline) {
        Ok(p) => p,
        Err(observation) => return observation,
    };
    let (mut out, mut err) = (pipes.stdout, pipes.stderr);
    let mut observation = Observation::not_spawned(StartupRefusal::Protocol);
    observation.known = false;
    observation.attempted = true;
    let mut protocol = !handshake;
    let mut valid = true;
    let mut response_deadline = if handshake {
        Some(deadline.min(Instant::now() + Duration::from_secs(30)))
    } else {
        None
    };
    let mut close_at = None;
    if handshake {
        let mut bytes = serde_json::to_vec(&decoder.request()).unwrap();
        bytes.push(b'\n');
        let sent = tokio::time::timeout_at(response_deadline.unwrap(), async {
            let mut stdin = pipes.stdin.lock().await;
            // Revoke cannot take stdin while it is held here; recheck under it.
            ensure!(
                interruption(&startup.cancelled, deadline).is_none(),
                "startup interrupted before initialize"
            );
            stdin
                .as_mut()
                .context("startup stdin absent")?
                .write_all(&bytes)
                .await?;
            Ok::<_, anyhow::Error>(())
        })
        .await;
        if !matches!(sent, Ok(Ok(()))) {
            valid = false;
            child.revoke();
            observation.signal = true;
            close_at = Some(Instant::now());
        }
    } else {
        pipes.stdin.lock().await.take();
    }
    let limit: usize = if handshake { 1024 * 1024 } else { 64 * 1024 };
    let mut buffer = Vec::new();
    let mut ob = [0u8; 4096];
    let mut eb = [0u8; 4096];
    let mut drain = None;
    let mut reap_end = deadline;
    let mut deliberate = false;
    // After a budget overflow, bytes are drained to EOF but never retained or counted.
    let mut overflowed = false;
    let mut tick = tokio::time::interval(Duration::from_millis(20));
    while !(observation.stdout_eof && observation.stderr_eof && observation.leader_reaped) {
        let now = Instant::now();
        if now >= deadline {
            valid = false;
            observation.reason = Some(StartupRefusal::Deadline);
            child.revoke();
            observation.signal = true;
            break;
        }
        if startup.cancelled.load(Ordering::SeqCst) {
            valid = false;
            observation.reason = Some(StartupRefusal::Stopped);
            close_at = Some(now);
        }
        if response_deadline.is_some_and(|d| now >= d) {
            valid = false;
            observation.reason = Some(StartupRefusal::Deadline);
            close_at = Some(now);
            response_deadline = None;
        }
        if now >= reap_end && !observation.leader_reaped {
            break;
        }
        if startup
            .owner
            .store
            .lock()
            .map_err(|_| ())
            .and_then(|store| store.old_startup_unknown(&startup.owner).map_err(|_| ()))
            .is_err()
        {
            valid = false;
        }
        if (!valid || close_at.is_some_and(|d| now >= d)) && !observation.signal {
            deliberate = protocol && valid && close_at.is_some_and(|d| now >= d);
            observation.group_hygiene = child.raw.lock().unwrap().hygiene();
            reap_end = deadline.min(now + Duration::from_secs(10));
            observation.signal = true;
            drain = Some(deadline.min(now + Duration::from_secs(2)));
        }
        {
            let mut raw = child.raw.lock().unwrap();
            if !observation.signal && raw.exited_unreaped().unwrap_or(false) {
                // Hygiene while numeric group identity is still unreaped.
                observation.group_hygiene = raw.hygiene();
                reap_end = deadline.min(now + Duration::from_secs(10));
                observation.signal = true;
                drain = Some(deadline.min(now + Duration::from_secs(2)));
            }
            if observation.signal
                && !observation.leader_reaped
                && let Ok(Some(exit)) = raw.reap()
            {
                observation.exit = exit.code();
                observation.leader_reaped = true;
            }
        }
        if drain.is_some_and(|d| now >= d) && (!observation.stdout_eof || !observation.stderr_eof) {
            break;
        }
        let window = if overflowed {
            ob.len()
        } else {
            (limit
                .saturating_sub(observation.bytes)
                .min((8usize * 1024 * 1024).saturating_sub(*aggregate))
                + 1)
            .min(4096)
        };
        let (n, stdout) = tokio::select! {biased;
            _=tick.tick()=>{continue;},
            n=out.read(&mut ob[..window]),if !observation.stdout_eof=>(n,true),
            n=err.read(&mut eb[..window]),if !observation.stderr_eof=>(n,false),
        };
        match n {
            Ok(0) => {
                if stdout {
                    observation.stdout_eof = true;
                } else {
                    observation.stderr_eof = true;
                }
            }
            Ok(_) if overflowed => {}
            Ok(n) => {
                observation.bytes = observation.bytes.saturating_add(n);
                *aggregate = aggregate.saturating_add(n);
                if observation.bytes > limit || *aggregate > 8 * 1024 * 1024 {
                    // The group is signalled by the invalid branch on the next
                    // pass; drain and reap stay bounded by the parent deadline.
                    valid = false;
                    overflowed = true;
                    observation.reason = Some(StartupRefusal::Budget);
                    response_deadline = None;
                    continue;
                }
                if stdout && valid {
                    buffer.extend_from_slice(&ob[..n]);
                    if handshake {
                        while let Some(end) = buffer.iter().position(|b| *b == b'\n') {
                            let line = buffer.drain(..=end).collect::<Vec<_>>();
                            match decoder.observe(&line[..line.len() - 1]) {
                                Ok(initialized) => {
                                    if initialized && !protocol {
                                        protocol = true;
                                        response_deadline = None;
                                        pipes.stdin.lock().await.take();
                                        close_at = Some(Instant::now() + Duration::from_secs(5));
                                    }
                                }
                                Err(_) => {
                                    valid = false;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {
                valid = false;
                break;
            }
        }
    }
    child.raw.lock().unwrap().close_stdin();
    if !observation.leader_reaped {
        child.revoke();
        observation.signal = true;
    }
    observation.known =
        observation.stdout_eof && observation.stderr_eof && observation.leader_reaped;
    if handshake {
        valid &= buffer.is_empty();
    } else if observation.known && observation.exit == Some(0) {
        match version(&buffer) {
            Ok(v) => observation.version = Some(v),
            Err(_) => valid = false,
        }
    } else {
        valid = false;
    }
    observation.protocol = protocol;
    // A deliberate group signal after successful initialize is permitted;
    // spontaneous nonzero exit remains a known failure.
    observation.pass = observation.known
        && protocol
        && observation.group_hygiene
        && valid
        && (observation.exit == Some(0) || handshake && deliberate && observation.exit.is_none());
    if observation.pass {
        observation.reason = None;
    }
    observation
}

#[cfg(test)]
mod tests {
    use super::*;
    std::thread_local! {
        pub(super) static STORE_WAIT_OBSERVER: std::cell::RefCell<Option<std::sync::mpsc::SyncSender<Instant>>> = const { std::cell::RefCell::new(None) };
    }
    /// The blocking reader checks stop and deadline before hashing even a large regular file.
    #[test]
    fn blocking_identity_read_refuses_stop_and_expired_deadline() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large");
        File::create(&path).unwrap().set_len(PROGRAM_BYTES).unwrap();
        for stop in [true, false] {
            let started = Instant::now();
            let deadline = if stop {
                started + Duration::from_secs(10)
            } else {
                started
            };
            let error = Program::read_file(&path, &AtomicBool::new(stop), deadline).unwrap_err();
            assert_eq!(error.to_string(), "startup identity interrupted");
            assert!(
                started.elapsed() < Duration::from_secs(1),
                "blocking identity read did not stop immediately"
            );
        }
    }
    /// T-1 records any bounded legal single line without matching a CLI number.
    #[test]
    fn version_observation_is_bounded_and_not_a_number_pin() {
        for bytes in [
            b"arbitrary 99.7\n".as_slice(),
            b"CLI/0.0+dev",
            "非ASCII\n".as_bytes(),
        ] {
            assert!(version(bytes).is_ok(), "T-1: lawful version refused");
        }
        assert_eq!(
            version("非ASCII\n".as_bytes()).unwrap().split(':').next(),
            Some("nonrecordable")
        );
        for bytes in [
            vec![],
            b"one\ntwo\n".to_vec(),
            vec![b'a'; 257],
            b"x\0y".to_vec(),
        ] {
            assert!(version(&bytes).is_err(), "T-1: invalid version accepted");
        }
    }
    /// AC-09 compares the complete observed program identity, including the content hash.
    #[tokio::test]
    async fn program_hash_and_stat_must_both_match_the_observed_original() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("program");
        std::fs::write(&path, b"first").unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let end = Instant::now() + Duration::from_secs(10);
        let mut program = Program::read(&path, cancelled.clone(), end).await.unwrap();
        program.verify(cancelled.clone(), end).await.unwrap();
        // Nongrant observation data only: a changed expected digest with an
        // unchanged real descriptor stat must not be accepted as identity.
        program.sha256 = "0".repeat(64);
        assert!(
            program.verify(cancelled.clone(), end).await.is_err(),
            "AC-09: content hash ignored"
        );
        let program = Program::read(&path, cancelled, end).await.unwrap();
        std::fs::write(&path, b"second").unwrap();
        assert!(
            program.stat().is_err(),
            "AC-09: changed program stat accepted"
        );
    }
    /// Identity observation refuses nonregular and oversized files before any spawn.
    #[tokio::test]
    async fn program_identity_rejects_special_and_oversized_files() {
        let dir = tempfile::tempdir().unwrap();
        let oversized = dir.path().join("oversized");
        File::create(&oversized)
            .unwrap()
            .set_len(PROGRAM_BYTES + 1)
            .unwrap();
        for path in [dir.path(), oversized.as_path()] {
            assert_eq!(
                Program::read(
                    path,
                    Arc::new(AtomicBool::new(false)),
                    Instant::now() + Duration::from_secs(2)
                )
                .await,
                Err(StartupRefusal::ProgramIdentity),
            );
        }
    }
    /// A FIFO is refused without waiting for a writer or reading any bytes.
    #[tokio::test]
    async fn program_identity_refuses_fifo_without_waiting_for_writer() {
        let dir = tempfile::tempdir().unwrap();
        let fifo = dir.path().join("fifo");
        let mut command = std::process::Command::new("/usr/bin/mkfifo");
        command.arg(&fifo);
        assert!(rrx_spawn_hygiene::status_std(command).unwrap().success());
        assert_eq!(
            Program::read(
                &fifo,
                Arc::new(AtomicBool::new(false)),
                Instant::now() + Duration::from_secs(2)
            )
            .await,
            Err(StartupRefusal::ProgramIdentity),
        );
    }
    /// Stop and the parent deadline interrupt an in-progress identity observation on one async worker.
    #[tokio::test]
    async fn program_identity_observation_obeys_stop_and_parent_deadline() {
        use std::{future::Future, task::Poll};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large");
        File::create(&path).unwrap().set_len(PROGRAM_BYTES).unwrap();
        for stop in [true, false] {
            let owner =
                RuntimeOwner::open(
                    &dir.path()
                        .join(if stop { "stop.db" } else { "deadline.db" }),
                )
                .unwrap();
            let startup = Startup::new(owner);
            let deadline = Instant::now() + Duration::from_millis(100);
            let mut config = Config::default();
            config
                .agents
                .insert("a".into(), crate::config::AgentConfig::default());
            let checked = Arc::new(crate::execution::claude_wire::decoder_selfcheck().unwrap());
            let read = startup.run(
                config,
                Some((
                    uuid::Uuid::new_v4(),
                    checked,
                    vec![("a".into(), "claude".into(), path.clone())],
                )),
                deadline,
            );
            tokio::pin!(read);
            std::future::poll_fn(|cx| {
                assert!(
                    read.as_mut().poll(cx).is_pending(),
                    "identity did not yield the async worker"
                );
                Poll::Ready(())
            })
            .await;
            if stop {
                startup.revoke();
            }
            read.await;
            let status = startup.snapshot().unwrap();
            assert_eq!(
                status.reason,
                Some(if stop {
                    StartupRefusal::Stopped
                } else {
                    StartupRefusal::Deadline
                })
            );
            assert_eq!(
                status.state,
                if stop {
                    StartupState::Stopped
                } else {
                    StartupState::Refused
                }
            );
            assert!(
                Instant::now() < deadline + Duration::from_secs(1),
                "identity ignored stop/deadline"
            );
            assert!(startup.children.lock().unwrap().is_empty());
        }
    }
    /// Revoke at the final spawn guard and an expired parent retain their typed not-spawned reasons.
    #[tokio::test]
    async fn pre_spawn_stop_and_expired_parent_are_typed_refusals() {
        let dir = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        let startup = Startup::new(owner.clone());
        let paths = path::Paths::new(owner).unwrap();
        let program = dir.path().join("program");
        std::fs::write(&program, b"never spawned").unwrap();
        let end = Instant::now() + Duration::from_secs(10);
        let plan = AliasPlan {
            alias: "a".into(),
            provider: "claude".into(),
            program: Program::read(&program, startup.cancelled.clone(), end)
                .await
                .unwrap(),
            directory: paths.alias(uuid::Uuid::new_v4(), "a").unwrap(),
            intents: Vec::new(),
            model: None,
            effort: None,
        };
        let checked = crate::execution::claude_wire::decoder_selfcheck().unwrap();
        let mut aggregate = 0;
        let expired = run(
            &startup,
            &plan,
            "version",
            &checked,
            Instant::now(),
            &mut aggregate,
        )
        .await;
        assert_eq!(expired.reason, Some(StartupRefusal::Deadline));
        assert!(expired.known && !expired.attempted);
        assert!(startup.children.lock().unwrap().is_empty());
        startup.revoke();
        let child = ChildCustody {
            raw: Mutex::new(RetainedRawProcess::default()),
        };
        let observed = spawn(&startup, &plan, &child, Command::new(&program), end)
            .err()
            .unwrap();
        assert_eq!(observed.reason, Some(StartupRefusal::Stopped));
        assert!(observed.known && !observed.attempted);
        assert!(!child.raw.lock().unwrap().has_child());
    }
    /// Stop or deadline expiry while spawn waits for the Store lock spawns and adopts nothing.
    #[tokio::test]
    async fn stop_or_deadline_during_store_wait_spawns_nothing() {
        use std::os::unix::fs::PermissionsExt;
        for stop in [true, false] {
            let dir = tempfile::tempdir().unwrap();
            let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
            let startup = Startup::new(owner.clone());
            let paths = path::Paths::new(owner.clone()).unwrap();
            let program = dir.path().join("program");
            std::fs::write(&program, b"#!/bin/sh\nexec /bin/sleep 30\n").unwrap();
            std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
            let end = Instant::now() + Duration::from_secs(10);
            let plan = AliasPlan {
                alias: "a".into(),
                provider: "claude".into(),
                program: Program::read(&program, startup.cancelled.clone(), end)
                    .await
                    .unwrap(),
                directory: paths.alias(uuid::Uuid::new_v4(), "a").unwrap(),
                intents: Vec::new(),
                model: None,
                effort: None,
            };
            let child = ChildCustody {
                raw: Mutex::new(RetainedRawProcess::default()),
            };
            let runtime = tokio::runtime::Handle::current();
            let store = owner.store.lock().unwrap();
            let (waiting, reached_wait) = std::sync::mpsc::sync_channel(1);
            let observed = std::thread::scope(|scope| {
                let probe = scope.spawn(|| {
                    let _runtime = runtime.enter();
                    STORE_WAIT_OBSERVER.with(|observer| *observer.borrow_mut() = Some(waiting));
                    let mut command = Command::new(&program);
                    command
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .process_group(0)
                        .kill_on_drop(true);
                    let deadline = if stop {
                        end
                    } else {
                        Instant::now() + Duration::from_millis(300)
                    };
                    spawn(&startup, &plan, &child, command, deadline)
                });
                let reached = reached_wait.recv_timeout(Duration::from_secs(5));
                if let Ok(deadline) = reached {
                    if stop {
                        startup.revoke();
                    } else {
                        // Sleep only to expire the deadline after observing the real wait.
                        std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
                    }
                }
                drop(store);
                let observed = probe.join().unwrap();
                reached.expect("probe never passed its first check to wait for Store");
                observed
            });
            let Err(observed) = observed else {
                panic!("spawned after stop/deadline");
            };
            assert_eq!(
                observed.reason,
                Some(if stop {
                    StartupRefusal::Stopped
                } else {
                    StartupRefusal::Deadline
                })
            );
            assert!(observed.known && !observed.attempted);
            assert!(!child.raw.lock().unwrap().has_child());
        }
    }
    /// A budget overflow still drains and reaps the leader, so the step is a known Budget refusal.
    #[tokio::test]
    async fn budget_overflow_is_drained_reaped_and_known() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        let startup = Startup::new(owner);
        let program = dir.path().join("program");
        // The version step limit is 64 KiB; this writes one byte more and exits.
        std::fs::write(
            &program,
            b"#!/bin/sh\nexec /usr/bin/head -c 65537 /dev/zero\n",
        )
        .unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut config = Config::default();
        config
            .agents
            .insert("a".into(), crate::config::AgentConfig::default());
        let checked = Arc::new(crate::execution::claude_wire::decoder_selfcheck().unwrap());
        startup
            .run(
                config,
                Some((
                    uuid::Uuid::new_v4(),
                    checked,
                    vec![("a".into(), "claude".into(), program)],
                )),
                Instant::now() + Duration::from_secs(60),
            )
            .await;
        let step = startup.snapshot().unwrap().aliases[0].steps[0].clone();
        assert_eq!(
            (step.step.as_str(), step.state, step.reason),
            ("version", ProbeState::Refused, Some(StartupRefusal::Budget)),
            "budget overflow was not a known refusal"
        );
        // No explicit shutdown has run; the probe itself reaped the leader.
        let children = startup.children.lock().unwrap();
        assert_eq!(children.len(), 1);
        assert!(children[0].raw.lock().unwrap().leader_reaped());
    }
}
