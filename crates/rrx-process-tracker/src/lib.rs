//! Cooperative discovery, not custody, complete process ownership or a sandbox.
//!
//! Only exact cookies in accessible same-user process environments are selected.
//! Public results never contain argv or environment values. Enumeration races,
//! inaccessible processes and stripped cookies prevent a complete-death claim.
//! Discovered-PID signaling is disabled on macOS; Linux uses a retained pidfd.
use std::{
    io,
    time::{Duration, Instant},
};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod macos;
#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "macos")]
use macos as platform;

const KEY: &[u8] = b"RRX_PROCESS_COOKIE=";

/// Opaque matching input. Its Debug implementation deliberately omits the value.
pub struct Cookie(String);
impl Cookie {
    pub fn new(value: &str) -> io::Result<Self> {
        if !(16..=128).contains(&value.len())
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid process cookie",
            ));
        }
        Ok(Self(value.into()))
    }
}
impl std::fmt::Debug for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Cookie(<redacted>)")
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub processes: usize,
    pub environment_bytes: usize,
    /// Checked between OS calls; it cannot interrupt a blocking kernel call.
    pub time: Duration,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            processes: 4096,
            environment_bytes: 256 * 1024,
            time: Duration::from_millis(500),
        }
    }
}
impl Limits {
    fn validate(&self) -> io::Result<()> {
        if self.processes == 0
            || self.processes > 65536
            || self.environment_bytes == 0
            || self.environment_bytes > 1024 * 1024
            || self.time.is_zero()
            || self.time > Duration::from_secs(30)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid discovery bounds",
            ));
        }
        Ok(())
    }
}
/// Birth fields are OS-specific: Linux start ticks; macOS seconds/microseconds.
/// This value is observation metadata, never an authority to signal a raw PID.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity {
    pub pid: u32,
    pub uid: u32,
    pub birth: (u64, u64),
}
#[derive(Default, Debug)]
pub struct Coverage {
    pub inspected: usize,
    pub same_user: usize,
    pub environment_unavailable: usize,
    pub identity_unavailable: usize,
    pub identity_changed: usize,
    pub handle_unavailable: usize,
    pub truncated: bool,
    pub time_exceeded: bool,
}
impl Coverage {
    /// Completeness of this bounded observation only; never full Task membership.
    pub fn limited(&self) -> bool {
        self.environment_unavailable != 0
            || self.identity_unavailable != 0
            || self.identity_changed != 0
            || self.handle_unavailable != 0
            || self.truncated
            || self.time_exceeded
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Termination {
    Sent,
    AlreadyExited,
    /// An atomic process handle is unavailable; no raw-PID fallback occurs.
    Unsupported,
    Denied,
    Unknown,
}
/// Cannot be manufactured from caller-provided PID/birth metadata.
pub struct TrackedProcess {
    identity: Identity,
    handle: platform::Handle,
}
impl TrackedProcess {
    pub fn identity(&self) -> Identity {
        self.identity
    }
    pub fn atomic_signal_available(&self) -> bool {
        self.handle.signal_available()
    }
    /// A successful send is not disappearance or complete workload reclamation.
    pub fn terminate(&self) -> Termination {
        self.handle.terminate()
    }
    /// A process-handle observation, not a scope-empty check.
    pub fn exited(&self) -> io::Result<Option<bool>> {
        self.handle.exited()
    }
}
impl std::fmt::Debug for TrackedProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackedProcess")
            .field("identity", &self.identity)
            .field("atomic_signal_available", &self.atomic_signal_available())
            .finish()
    }
}
#[derive(Debug)]
pub struct Discovery {
    pub processes: Vec<TrackedProcess>,
    pub coverage: Coverage,
}
pub fn discover(cookie: &Cookie, limits: Limits) -> io::Result<Discovery> {
    limits.validate()?;
    platform::discover(cookie, limits, Instant::now())
}

// Interpret only the exact cookie field. Everything else stays unparsed and is
// overwritten before its temporary buffer is dropped. Repeated cookies refuse
// matching rather than choosing an arbitrary environment entry.
fn matches_environment(bytes: &[u8], cookie: &Cookie) -> bool {
    let mut values = bytes
        .split(|b| *b == 0)
        .filter_map(|entry| entry.strip_prefix(KEY));
    let selected = values.next() == Some(cookie.0.as_bytes());
    selected && values.next().is_none()
}
fn discard(bytes: &mut [u8]) {
    bytes.fill(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_environment_field_and_duplicate_refusal() {
        let cookie = Cookie::new("test-cookie-123456").unwrap();
        assert!(matches_environment(
            b"OTHER=discarded\0RRX_PROCESS_COOKIE=test-cookie-123456\0",
            &cookie
        ));
        for value in [
            &b"RRX_PROCESS_COOKIE=test-cookie-123456-extra\0"[..],
            &b"NOT_RRX_PROCESS_COOKIE=test-cookie-123456\0"[..],
            &b"RRX_PROCESS_COOKIE=test-cookie-123456\0RRX_PROCESS_COOKIE=test-cookie-123456\0"[..],
        ] {
            assert!(!matches_environment(value, &cookie));
        }
        assert!(!format!("{cookie:?}").contains("123456"));
    }
    #[test]
    fn invalid_cookie_and_scan_bounds_refuse() {
        assert!(Cookie::new("short").is_err());
        assert!(Cookie::new("test-cookie-123456\n").is_err());
        assert!(
            discover(
                &Cookie::new("test-cookie-123456").unwrap(),
                Limits {
                    processes: 0,
                    ..Default::default()
                }
            )
            .is_err()
        );
    }
}
