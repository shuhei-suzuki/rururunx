use super::*;
use rustix::{
    fd::{AsRawFd, OwnedFd},
    process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal},
};
use std::fs::{self, File};

pub(super) struct Handle(Option<OwnedFd>);
impl Handle {
    pub fn signal_available(&self) -> bool {
        self.0.is_some()
    }
    pub fn terminate(&self) -> Termination {
        let Some(fd) = &self.0 else {
            return Termination::Unsupported;
        };
        match pidfd_send_signal(fd, Signal::KILL) {
            Ok(()) => Termination::Sent,
            Err(rustix::io::Errno::SRCH) => Termination::AlreadyExited,
            Err(rustix::io::Errno::PERM | rustix::io::Errno::ACCESS) => Termination::Denied,
            Err(_) => Termination::Unknown,
        }
    }
    pub fn exited(&self) -> io::Result<Option<bool>> {
        use rustix::event::{PollFd, PollFlags, Timespec, poll};
        let Some(fd) = &self.0 else {
            return Ok(None);
        };
        let mut pollfd = [PollFd::new(fd, PollFlags::IN)];
        poll(&mut pollfd, Some(&Timespec::default()))?;
        let events = pollfd[0].revents();
        if events.intersects(PollFlags::NVAL | PollFlags::ERR) {
            return Err(io::Error::other("pidfd readiness unavailable"));
        }
        Ok(Some(events.intersects(PollFlags::IN | PollFlags::HUP)))
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
    // A non-dumpable same-user process has root-owned procfs inodes. Their
    // owner is not its effective UID; classify using the kernel status field.
    let uid = effective_uid(&bounded_metadata(&format!("{root}/status"))?)?;
    let stat = bounded_metadata(&format!("{root}/stat"))?;
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
fn bounded_metadata(path: &str) -> io::Result<String> {
    use std::io::Read;
    const LIMIT: usize = 64 * 1024;
    let mut bytes = Vec::new();
    File::open(path)?
        .take((LIMIT + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process metadata exceeds bound",
        ));
    }
    String::from_utf8(bytes).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "process metadata encoding unavailable",
        )
    })
}
fn effective_uid(status: &str) -> io::Result<u32> {
    status
        .lines()
        .find_map(|line| line.strip_prefix("Uid:"))
        .and_then(|uids| uids.split_whitespace().nth(1))
        .and_then(|uid| uid.parse().ok())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "process effective UID unavailable",
            )
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
        let read = File::open(format!("/proc/{pid}/environ")).and_then(|file| {
            read_environment(
                file,
                limits.environment_bytes,
                EnvironmentBuffer::new(Vec::new()),
            )
        });
        let bytes = match read {
            Ok(b) => b,
            Err(_) => {
                result.coverage.environment_unavailable += 1;
                continue;
            }
        };
        let matched = matches_environment(&bytes.bytes, cookie);
        drop(bytes);
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "child-only non-dumpable identity fixture"]
    fn nondumpable_identity_worker() {
        let Some(marker) = std::env::var_os("RRX_TRACKER_IDENTITY_READY") else {
            return;
        };
        rustix::process::set_dumpable_behavior(rustix::process::DumpableBehavior::NotDumpable)
            .unwrap();
        fs::write(marker, "ready").unwrap();
        std::thread::sleep(Duration::from_secs(30));
    }
    #[test]
    fn nondumpable_same_user_is_classified_as_inaccessible_not_foreign() {
        use std::{
            os::unix::fs::MetadataExt,
            process::{Child, Command, Stdio},
        };
        struct OwnedChild(Child);
        impl Drop for OwnedChild {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let root = tempfile::tempdir().unwrap();
        let marker = root.path().join("ready");
        let value = format!("tracker-nondumpable-{}", std::process::id());
        let mut child = OwnedChild(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--ignored",
                    "--exact",
                    "linux::tests::nondumpable_identity_worker",
                ])
                .env("RRX_TRACKER_IDENTITY_READY", &marker)
                .env("RRX_PROCESS_COOKIE", &value)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let at = Instant::now();
        while !marker.exists() {
            assert!(at.elapsed() < Duration::from_secs(10));
            assert!(child.0.try_wait().unwrap().is_none());
            std::thread::sleep(Duration::from_millis(10));
        }
        let uid = rustix::process::geteuid().as_raw();
        let observed = identity(child.0.id()).unwrap();
        assert_eq!(observed.uid, uid);
        assert_eq!(
            fs::metadata(format!("/proc/{}", child.0.id()))
                .unwrap()
                .uid(),
            0
        );
        if uid != 0 {
            assert!(File::open(format!("/proc/{}/environ", child.0.id())).is_err());
            let found = super::discover(
                &Cookie::new(&value).unwrap(),
                Limits {
                    time: Duration::from_secs(10),
                    ..Default::default()
                },
                Instant::now(),
            )
            .unwrap();
            assert!(found.processes.is_empty());
            assert!(found.coverage.environment_unavailable > 0);
        }
        // Root can read an otherwise inaccessible environment; that case does
        // not qualify the unprivileged coverage branch above.
        assert!(child.0.try_wait().unwrap().is_none());
    }
    #[test]
    fn effective_uid_uses_status_instead_of_inode_or_real_uid() {
        assert_eq!(
            effective_uid("Name:\tx\nUid:\t1000\t1001\t1002\t1003\n").unwrap(),
            1001
        );
        for malformed in ["Uid:\t1000\n", "Uid:\t1000\tbad\n", "Name:\tx\n"] {
            assert!(effective_uid(malformed).is_err());
        }
        let this = identity(std::process::id()).unwrap();
        assert_eq!(this.uid, rustix::process::geteuid().as_raw());
    }
    #[test]
    fn absent_pidfd_never_signals_or_fabricates_exit() {
        let handle = Handle(None);
        assert!(!handle.signal_available());
        assert_eq!(handle.terminate(), Termination::Unsupported);
        assert_eq!(handle.exited().unwrap(), None);
    }
}
