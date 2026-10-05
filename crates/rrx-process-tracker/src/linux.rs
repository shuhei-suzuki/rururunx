use super::*;
use rustix::{
    fd::{AsRawFd, OwnedFd},
    process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal},
};
use std::{
    fs::{self, File},
    io::Read,
    os::unix::fs::MetadataExt,
};

pub(super) struct Handle(Option<OwnedFd>);
impl Handle {
    pub fn signal_available(&self) -> bool {
        self.0.is_some()
    }
    pub fn terminate(&self) -> Termination {
        let Some(fd) = &self.0 else {
            return Termination::Unsupported;
        };
        match pidfd_send_signal(fd, Signal::Kill) {
            Ok(()) => Termination::Sent,
            Err(rustix::io::Errno::SRCH) => Termination::AlreadyExited,
            Err(rustix::io::Errno::PERM | rustix::io::Errno::ACCESS) => Termination::Denied,
            Err(_) => Termination::Unknown,
        }
    }
    pub fn exited(&self) -> io::Result<Option<bool>> {
        self.0
            .as_ref()
            .map(|fd| fd_pid(fd).map(|pid| Some(pid.is_none())))
            .unwrap_or(Ok(None))
    }
}
fn fd_pid(fd: &OwnedFd) -> io::Result<Option<u32>> {
    let info = fs::read_to_string(format!("/proc/self/fdinfo/{}", fd.as_raw_fd()))?;
    let value = info
        .lines()
        .find_map(|s| s.strip_prefix("Pid:"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "pidfd identity unavailable"))?
        .trim();
    if value == "-1" {
        return Ok(None);
    }
    value
        .parse()
        .map(Some)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid pidfd identity"))
}
fn identity(pid: u32) -> io::Result<Identity> {
    let root = format!("/proc/{pid}");
    let uid = fs::metadata(&root)?.uid();
    let stat = fs::read_to_string(format!("{root}/stat"))?;
    let suffix = stat
        .rsplit_once(')')
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "process stat malformed"))?
        .1;
    let birth = suffix
        .split_whitespace()
        .nth(19)
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "process birth unavailable"))?;
    Ok(Identity {
        pid,
        uid,
        birth: (birth, 0),
    })
}
pub(super) fn discover(cookie: &Cookie, limits: Limits, started: Instant) -> io::Result<Discovery> {
    let uid = rustix::process::geteuid().as_raw();
    let mut result = Discovery {
        processes: Vec::new(),
        coverage: Coverage::default(),
    };
    for entry in fs::read_dir("/proc")? {
        if started.elapsed() >= limits.time {
            result.coverage.time_exceeded = true;
            break;
        }
        let entry = match entry {
            Ok(e) => e,
            Err(_) => {
                result.coverage.identity_unavailable += 1;
                continue;
            }
        };
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        if result.coverage.inspected >= limits.processes {
            result.coverage.truncated = true;
            break;
        }
        result.coverage.inspected += 1;
        let before = match identity(pid) {
            Ok(i) => i,
            Err(_) => {
                result.coverage.identity_unavailable += 1;
                continue;
            }
        };
        if before.uid != uid {
            continue;
        }
        result.coverage.same_user += 1;
        // Bind the kernel handle BEFORE reading /proc/<pid>/environ. Verify it
        // afterwards: an old exited fd cannot authorize a reused PID's cookie.
        let fd =
            Pid::from_raw(pid as i32).and_then(|pid| pidfd_open(pid, PidfdFlags::empty()).ok());
        if fd.is_none() {
            result.coverage.handle_unavailable += 1;
        }
        let read = || -> io::Result<Vec<u8>> {
            let mut bytes = Vec::new();
            File::open(format!("/proc/{pid}/environ"))?
                .take((limits.environment_bytes + 1) as u64)
                .read_to_end(&mut bytes)?;
            if bytes.len() > limits.environment_bytes {
                discard(&mut bytes);
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "process environment exceeds bound",
                ));
            }
            Ok(bytes)
        };
        let mut bytes = match read() {
            Ok(b) => b,
            Err(_) => {
                result.coverage.environment_unavailable += 1;
                continue;
            }
        };
        let matched = matches_environment(&bytes, cookie);
        discard(&mut bytes);
        if !matched {
            continue;
        }
        let identity_matches = identity(pid).is_ok_and(|after| after == before);
        let handle_matches = fd
            .as_ref()
            .is_none_or(|fd| fd_pid(fd).is_ok_and(|p| p == Some(pid)));
        if !identity_matches || !handle_matches {
            result.coverage.identity_changed += 1;
            continue;
        }
        result.processes.push(TrackedProcess {
            identity: before,
            handle: Handle(fd),
        });
    }
    Ok(result)
}
