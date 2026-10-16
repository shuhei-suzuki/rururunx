//! Bounded S Version/Claude initialize capture; no Unit, quota or model input.
use super::*;
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
    pub(super) fn read(path: &Path) -> Result<Self> {
        let path = path.canonicalize()?;
        let mut file = File::open(&path)?;
        let m = file.metadata()?;
        ensure!(m.is_file(), "startup program is not a file");
        let mut h = Sha256::new();
        let mut bytes = [0u8; 65536];
        loop {
            let n = file.read(&mut bytes)?;
            if n == 0 {
                break;
            }
            h.update(&bytes[..n]);
        }
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
    fn verify(&self) -> Result<()> {
        ensure!(
            Self::read(&self.path)? == *self,
            "startup executable hash changed"
        );
        Ok(())
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

pub(super) async fn run(
    startup: &Arc<Startup>,
    plan: &AliasPlan,
    step: &str,
    checked: &DecoderChecked,
    parent: Instant,
    aggregate: &mut usize,
) -> Observation {
    if *aggregate >= 8 * 1024 * 1024 || Instant::now() >= parent {
        return Observation::not_spawned(StartupRefusal::Budget);
    }
    if plan.directory.verify_empty().is_err() {
        return Observation::not_spawned(StartupRefusal::PrivatePath);
    }
    if plan.program.verify().is_err() {
        return Observation::not_spawned(StartupRefusal::ProgramIdentity);
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
    let spawned = (|| -> Result<_> {
        plan.directory.verify_empty()?;
        plan.program.stat()?;
        ensure!(
            !startup.cancelled.load(Ordering::SeqCst) && Instant::now() < deadline,
            "startup stopped before spawn"
        );
        // Check the live owner, never a saved Task or receipt.
        startup
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .old_startup_unknown(&startup.owner)?;
        let process = rrx_spawn_hygiene::spawn_tokio(command)?;
        let mut raw = child.raw.lock().unwrap();
        raw.adopt(process);
        raw.qualify()?;
        raw.pipes()
    })();
    let pipes = match spawned {
        Ok(p) => p,
        Err(_) => {
            child.revoke();
            let mut o = Observation::not_spawned(StartupRefusal::Protocol);
            o.known = !child.raw.lock().unwrap().has_child();
            return o;
        }
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
        let window = (limit
            .saturating_sub(observation.bytes)
            .min((8usize * 1024 * 1024).saturating_sub(*aggregate))
            + 1)
        .min(4096);
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
            Ok(n) => {
                observation.bytes = observation.bytes.saturating_add(n);
                *aggregate = aggregate.saturating_add(n);
                if observation.bytes > limit || *aggregate > 8 * 1024 * 1024 {
                    valid = false;
                    observation.reason = Some(StartupRefusal::Budget);
                    child.revoke();
                    observation.signal = true;
                    break;
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
    #[test]
    fn program_hash_and_stat_must_both_match_the_observed_original() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("program");
        std::fs::write(&path, b"first").unwrap();
        let mut program = Program::read(&path).unwrap();
        program.verify().unwrap();
        // Nongrant observation data only: a changed expected digest with an
        // unchanged real descriptor stat must not be accepted as identity.
        program.sha256 = "0".repeat(64);
        assert!(program.verify().is_err(), "AC-09: content hash ignored");
        let program = Program::read(&path).unwrap();
        std::fs::write(&path, b"second").unwrap();
        assert!(
            program.stat().is_err(),
            "AC-09: changed program stat accepted"
        );
    }
}
