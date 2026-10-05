//! Public SDK proc_info.h/libproc.h and KERN_PROCARGS2; no private entitlement.
//! Safety: all pointer destinations below have their exact writable allocation;
//! initialized lengths and struct return sizes are checked before interpretation.
use super::*;
use std::mem::{MaybeUninit, size_of};

// macOS SDK sys/proc_info.h: PROC_UID_ONLY. proc_listpids returns byte counts.
const PROC_UID_ONLY: u32 = 4;
#[link(name = "proc")]
unsafe extern "C" {}

pub(super) struct Handle;
impl Handle {
    pub fn signal_available(&self) -> bool {
        false
    }
    pub fn terminate(&self) -> Termination {
        Termination::Unsupported
    }
    pub fn exited(&self) -> io::Result<Option<bool>> {
        Ok(None)
    }
}
fn identity(pid: u32) -> io::Result<Identity> {
    let mut info = MaybeUninit::<libc::proc_bsdinfo>::uninit();
    // SAFETY: the destination points to one full writable proc_bsdinfo. No field
    // is read unless the kernel reports the exact struct size.
    let bytes = unsafe {
        libc::proc_pidinfo(
            pid as i32,
            libc::PROC_PIDTBSDINFO,
            0,
            info.as_mut_ptr().cast(),
            size_of::<libc::proc_bsdinfo>() as i32,
        )
    };
    if bytes != size_of::<libc::proc_bsdinfo>() as i32 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the exact-size return above initialized the complete public struct.
    let info = unsafe { info.assume_init() };
    if info.pbi_pid != pid {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process identity changed",
        ));
    }
    Ok(Identity {
        pid,
        uid: info.pbi_uid,
        birth: (info.pbi_start_tvsec, info.pbi_start_tvusec),
    })
}
fn environment_offset(bytes: &[u8]) -> Option<usize> {
    let argc = i32::from_ne_bytes(bytes.get(..size_of::<i32>())?.try_into().ok()?);
    if !(0..=65536).contains(&argc) {
        return None;
    }
    let mut at = size_of::<i32>();
    // Executable path followed by padding, then exactly argc argument strings.
    at += bytes.get(at..)?.iter().position(|b| *b == 0)? + 1;
    while bytes.get(at) == Some(&0) {
        at += 1;
    }
    for _ in 0..argc {
        at += bytes.get(at..)?.iter().position(|b| *b == 0)? + 1;
    }
    Some(at)
}
fn cookie_matches(pid: u32, cookie: &Cookie, cap: usize) -> io::Result<bool> {
    let mut mib = [libc::CTL_KERN, libc::KERN_PROCARGS2, pid as i32];
    let mut buffer = EnvironmentBuffer::new(vec![0_u8; cap]);
    let bytes = &mut buffer.bytes;
    let mut count = bytes.len();
    // SAFETY: mib contains three initialized integers; bytes owns cap writable
    // bytes; count points to an initialized size. No new-value buffer is supplied.
    let status = unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            bytes.as_mut_ptr().cast(),
            &mut count,
            std::ptr::null_mut(),
            0,
        )
    };
    let result = if status != 0 || count > cap {
        Err(io::Error::last_os_error())
    } else {
        bytes.truncate(count);
        environment_offset(bytes)
            .map(|at| matches_environment(&bytes[at..], cookie))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "process environment unavailable",
                )
            })
    };
    drop(buffer);
    result
}
pub(super) fn discover(cookie: &Cookie, limits: Limits, started: Instant) -> io::Result<Discovery> {
    let uid = rustix::process::geteuid().as_raw();
    let mut pids = vec![0_i32; limits.processes];
    // SAFETY: pids is a fully initialized contiguous integer buffer of the
    // specified byte length. The selector returns same-effective-user PIDs.
    let count = unsafe {
        libc::proc_listpids(
            PROC_UID_ONLY,
            uid,
            pids.as_mut_ptr().cast(),
            (pids.len() * size_of::<i32>()) as i32,
        )
    };
    if count <= 0 {
        return Err(io::Error::last_os_error());
    }
    let mut result = Discovery {
        processes: Vec::new(),
        coverage: Coverage {
            truncated: count as usize >= pids.len() * size_of::<i32>(),
            ..Default::default()
        },
    };
    let returned = (count as usize / size_of::<i32>()).min(pids.len());
    for pid in pids.into_iter().take(returned).filter(|pid| *pid > 0) {
        if started.elapsed() >= limits.time {
            result.coverage.time_exceeded = true;
            break;
        }
        result.coverage.inspected += 1;
        let before = match identity(pid as u32) {
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
        let matched = match cookie_matches(pid as u32, cookie, limits.environment_bytes) {
            Ok(m) => m,
            Err(_) => {
                result.coverage.environment_unavailable += 1;
                continue;
            }
        };
        if !matched {
            continue;
        }
        if !identity(pid as u32).is_ok_and(|after| after == before) {
            result.coverage.identity_changed += 1;
            continue;
        }
        result.processes.push(TrackedProcess {
            identity: before,
            handle: Handle,
        });
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn argument_that_looks_like_cookie_is_not_an_environment_match() {
        let cookie = Cookie::new("test-cookie-123456").unwrap();
        let mut bytes = 2_i32.to_ne_bytes().to_vec();
        bytes.extend_from_slice(
            b"/fixture\0\0fixture\0RRX_PROCESS_COOKIE=test-cookie-123456\0OTHER=discarded\0",
        );
        let at = environment_offset(&bytes).unwrap();
        assert!(!matches_environment(&bytes[at..], &cookie));
        bytes.extend_from_slice(b"RRX_PROCESS_COOKIE=test-cookie-123456\0");
        assert!(matches_environment(&bytes[at..], &cookie));
    }
}
