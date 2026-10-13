use rustix::process::{self, Pid, Signal, WaitId, WaitIdOptions};
use std::{
    fs::File,
    io::{self, Write},
    os::unix::process::{CommandExt, ExitStatusExt},
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Stage {
    Observe,
    FinalSignal,
    Reap,
    Gate,
    Lost,
    Finished,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Observation {
    Running,
    Exited,
    Interrupted,
    Error,
    Accepted,
    Reaped,
    Empty,
    Remaining,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Poll,
    StopGroup,
    StopLeader,
    FinalGroup,
    Reap,
    ReadGroup,
    Hold,
    Finish,
}

// Pure decisions are separate from the owner executing them. A failed authority
fn signal_accepted(macos: bool, final_signal: bool, result: Result<(), i32>) -> bool {
    result.is_ok()
        || (final_signal && (result == Err(libc::ESRCH) || (macos && result == Err(libc::EPERM))))
}

// Pure decisions are separate from the owner executing them. A failed authority
// observation is permanent; neither later empty reads nor a deadline restores it.
pub(super) fn decide(stage: Stage, observation: Observation, group: bool, expired: bool) -> Action {
    match stage {
        Stage::Observe => match observation {
            Observation::Error => Action::Hold,
            Observation::Exited if group => Action::FinalGroup,
            Observation::Exited => Action::Reap,
            Observation::Running if expired && group => Action::StopGroup,
            Observation::Running if expired => Action::StopLeader,
            _ => Action::Poll,
        },
        Stage::FinalSignal if observation == Observation::Accepted => Action::Reap,
        Stage::FinalSignal => Action::FinalGroup,
        Stage::Reap if observation == Observation::Reaped && group => Action::ReadGroup,
        Stage::Reap => Action::Hold,
        Stage::Gate if observation == Observation::Empty && group => Action::Finish,
        Stage::Gate => Action::ReadGroup,
        Stage::Lost => Action::Hold,
        Stage::Finished => Action::Finish,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Classification {
    Setup,
    NoResult,
    Pass,
    Fail(String),
    Timeout,
}
#[derive(Debug)]
pub(super) struct Report {
    pub classification: Classification,
    pub complete: bool,
    pub trace: Vec<&'static str>,
    pub status: ExitStatus,
    pub stopped: bool,
    pub stdout: String,
    pub stderr: String,
}
impl Report {
    pub fn killed(&self, registered_stop: bool) -> bool {
        self.complete
            && (matches!(self.classification, Classification::Fail(_))
                || (registered_stop
                    && self.classification == Classification::Timeout
                    && self.stopped
                    && self.status.signal() == Some(libc::SIGKILL)))
    }
}

pub(super) struct WorkerGroupOwner {
    child: Child,
    pid: Option<Pid>,
    directory: tempfile::TempDir,
    stage: Stage,
    group: bool,
    deadline: Instant,
    recovery: Option<Instant>,
    held: bool,
    timeout: bool,
    stopped: bool,
    trace: Vec<&'static str>,
    status: Option<ExitStatus>,
}

impl WorkerGroupOwner {
    #[allow(
        clippy::disallowed_methods,
        reason = "raw harness must contain even a broken or stalled hygiene callback; only regular-file stdio"
    )]
    pub fn start(case: &str) -> io::Result<Self> {
        let directory = tempfile::tempdir()?;
        let stdout = File::create(directory.path().join("stdout"))?;
        let stderr = File::create(directory.path().join("stderr"))?;
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut command = Command::new(std::env::current_exe()?);
        command
            .args(["--exact", "tests::controls::worker", "--nocapture"])
            .env("RRX_P4_CASE", case)
            .env("RRX_P4_DIR", directory.path())
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr);
        // The successful Child is stored immediately in its sole owner. No
        // fallible operation or assertion separates spawn from this custody.
        let mut owner = Self {
            child: command.spawn()?,
            pid: None,
            directory,
            stage: Stage::Observe,
            group: false,
            deadline,
            recovery: None,
            held: false,
            timeout: false,
            stopped: false,
            trace: vec![],
            status: None,
        };
        owner.pid = Pid::from_raw(owner.child.id() as i32).filter(|p| p.as_raw_pid() > 1);
        owner.check_group();
        Ok(owner)
    }

    fn check_group(&mut self) {
        if !self.group
            && let Some(pid) = self.pid
            && process::getpgid(Some(pid)) == Ok(pid)
        {
            self.group = true;
            self.trace.push("S1:group");
        }
    }

    fn observe(&mut self) -> Observation {
        let Some(pid) = self.pid else {
            self.stage = Stage::Lost;
            return Observation::Error;
        };
        match process::waitid(
            WaitId::Pid(pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOWAIT | WaitIdOptions::NOHANG,
        ) {
            Ok(Some(_)) => Observation::Exited,
            Ok(None) => Observation::Running,
            Err(rustix::io::Errno::INTR) => Observation::Interrupted,
            Err(_) => {
                self.stage = Stage::Lost;
                Observation::Error
            }
        }
    }

    // Every real signal is preceded by a fresh same-thread WNOWAIT observation.
    fn signal(&mut self, final_signal: bool, group: bool) -> Observation {
        if !matches!(self.observe(), Observation::Exited | Observation::Running) {
            return Observation::Error;
        }
        let result = if group {
            match self.pid {
                Some(pid) => {
                    process::kill_process_group(pid, Signal::KILL).map_err(io::Error::from)
                }
                None => return Observation::Error,
            }
        } else {
            self.child.kill()
        };
        let accepted = signal_accepted(
            cfg!(target_os = "macos"),
            final_signal,
            result.map_err(|error| error.raw_os_error().unwrap_or(libc::EIO)),
        );
        if accepted {
            if final_signal || !self.stopped {
                self.trace
                    .push(if final_signal { "S4:accepted" } else { "S3:ok" });
            }
            if !final_signal {
                self.stopped = true;
            }
            Observation::Accepted
        } else {
            Observation::Error
        }
    }

    fn diagnostic(&self) {
        // Write to the real stderr fd so libtest capture cannot hide Held.
        let _ = io::stderr().lock().write_all(
            format!(
                "P4C HELD stage={:?} pid={:?} group={} trace={:?}\n",
                self.stage, self.pid, self.group, self.trace
            )
            .as_bytes(),
        );
    }

    fn collect(&mut self) {
        let mut last_diagnostic = Instant::now();
        let mut observation = Observation::Running;
        loop {
            if self.stage == Stage::Finished {
                return;
            }
            let now = Instant::now();
            if self.recovery.is_some_and(|deadline| now >= deadline) {
                self.held = true;
            }
            if self.held && last_diagnostic.elapsed() >= Duration::from_secs(1) {
                self.diagnostic();
                last_diagnostic = Instant::now();
            }
            if self.stage == Stage::Observe {
                self.check_group();
                observation = self.observe();
                if observation == Observation::Exited {
                    self.trace.push("S2:exited");
                    self.recovery.get_or_insert(now + Duration::from_secs(10));
                } else if now >= self.deadline && observation == Observation::Running {
                    self.timeout = true;
                    self.recovery.get_or_insert(now + Duration::from_secs(10));
                }
            }
            match decide(self.stage, observation, self.group, now >= self.deadline) {
                Action::Poll => {}
                Action::StopGroup | Action::StopLeader => {
                    observation = self.signal(false, self.group);
                }
                Action::FinalGroup => {
                    if self.group {
                        self.stage = Stage::FinalSignal;
                        observation = self.signal(true, true);
                    } else {
                        self.stage = Stage::Lost;
                    }
                }
                Action::Reap => {
                    // S4 was accepted, or the unestablished leader exited. This
                    // wait happens once; any error permanently loses authority.
                    match self.child.wait() {
                        Ok(status) => {
                            self.status = Some(status);
                            self.trace.push("S6:reaped");
                            self.stage = Stage::Reap;
                            observation = Observation::Reaped;
                        }
                        Err(_) => {
                            self.stage = Stage::Lost;
                            observation = Observation::Error;
                        }
                    }
                }
                Action::ReadGroup => {
                    self.stage = Stage::Gate;
                    observation = match self.pid.map(process::test_kill_process_group) {
                        Some(Err(rustix::io::Errno::SRCH)) => {
                            self.trace.push("S7:ESRCH");
                            Observation::Empty
                        }
                        _ => Observation::Remaining,
                    };
                }
                Action::Hold => {
                    self.stage = Stage::Lost;
                    self.held = true;
                    // Diagnostic read only. No wait or real signal after loss.
                    if let Some(pid) = self.pid {
                        let _ = process::test_kill_process_group(pid);
                    }
                }
                Action::Finish => {
                    self.stage = Stage::Finished;
                    return;
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn finish(mut self) -> Report {
        self.collect();
        // Only reachable after the five custody/recovery gates. Reading and
        // classifying output never confers signal authority.
        let stdout = std::fs::read_to_string(self.directory.path().join("stdout"));
        let status = match self.status {
            Some(status) => status,
            None => unreachable!("Finished requires successful reap"),
        };
        let classification = if self.timeout {
            Classification::Timeout
        } else {
            match &stdout {
                Ok(text) => classify(text, status.success()),
                Err(_) => Classification::NoResult,
            }
        };
        Report {
            classification,
            complete: !self.held,
            trace: self.trace.clone(),
            status,
            stopped: self.stopped,
            stdout: stdout.unwrap_or_else(|error| format!("unreadable stdout: {error}")),
            stderr: std::fs::read_to_string(self.directory.path().join("stderr"))
                .unwrap_or_else(|error| format!("unreadable stderr: {error}")),
        }
    }
}

impl Drop for WorkerGroupOwner {
    fn drop(&mut self) {
        self.collect();
    }
}

fn classify(stdout: &str, success: bool) -> Classification {
    let lines: Vec<_> = stdout
        .lines()
        .filter(|line| line.starts_with("P4C RESULT ") || line.starts_with("P4C SETUP "))
        .collect();
    match lines.as_slice() {
        [line] if line.starts_with("P4C SETUP fail") => Classification::Setup,
        ["P4C RESULT pass"] if success => Classification::Pass,
        [line] if success && line.starts_with("P4C RESULT fail:") => {
            Classification::Fail(line.trim_start_matches("P4C RESULT fail:").to_owned())
        }
        _ => Classification::NoResult,
    }
}

#[test]
fn ow1_custody_table() {
    use {Action::*, Observation::*, Stage::*};
    assert_eq!(decide(Observe, Exited, true, false), FinalGroup);
    assert_eq!(decide(FinalSignal, Accepted, true, false), Action::Reap);
    assert_eq!(decide(Stage::Reap, Reaped, true, false), ReadGroup);
    for obs in [Remaining, Error, Accepted] {
        for expired in [false, true] {
            assert_eq!(decide(Gate, obs, true, expired), ReadGroup);
        }
    }
    assert_eq!(decide(Gate, Empty, true, true), Finish);
    for obs in [Running, Exited, Empty, Error, Reaped] {
        assert_eq!(decide(Lost, obs, true, true), Hold);
    }
    assert_eq!(decide(Observe, Error, true, true), Hold);
    assert_eq!(decide(Stage::Reap, Error, true, true), Hold);
    assert_eq!(decide(Observe, Running, false, true), StopLeader);
    assert_eq!(decide(Observe, Exited, false, false), Action::Reap);
    assert_eq!(decide(Stage::Reap, Reaped, false, true), Hold);
    assert_ne!(decide(Gate, Empty, false, true), Finish);
    // macOS S4 EPERM can authorize reap but cannot bypass S7. S3 EPERM
    // is never stop success; non-macOS S4 EPERM never authorizes reap.
    assert!(signal_accepted(true, true, Err(libc::EPERM)));
    assert!(!signal_accepted(true, false, Err(libc::EPERM)));
    assert!(!signal_accepted(false, true, Err(libc::EPERM)));
    let accepted = Observation::Accepted;
    assert_eq!(decide(FinalSignal, accepted, true, false), Action::Reap);
    assert_eq!(decide(Gate, Remaining, true, true), ReadGroup);
    assert_eq!(decide(FinalSignal, Error, true, true), FinalGroup);
    assert_eq!(
        classify("P4C RESULT fail:broken\n", false),
        Classification::NoResult
    );
    assert_eq!(
        classify("P4C RESULT pass\n", false),
        Classification::NoResult
    );
    assert_eq!(
        classify("P4C RESULT fail:broken\n", true),
        Classification::Fail("broken".into())
    );
    assert_eq!(
        classify("P4C RESULT pass\nP4C RESULT pass\n", true),
        Classification::NoResult
    );
    assert_eq!(classify("P4C STALL\n", true), Classification::NoResult);
    for class in [
        Classification::Setup,
        Classification::NoResult,
        Classification::Timeout,
    ] {
        let report = Report {
            classification: class,
            complete: false,
            trace: vec![],
            status: ExitStatus::from_raw(libc::SIGKILL),
            stopped: true,
            stdout: String::new(),
            stderr: String::new(),
        };
        assert!(!report.killed(true));
    }
}
