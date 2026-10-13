//! All post-fork unsafe operations are confined to this module.
#![allow(unsafe_code)]

use std::io;
use std::process::Command;

#[cfg(target_os = "macos")]
#[link(name = "proc")]
unsafe extern "C" {}

#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[derive(Default)]
pub(super) struct Options {
    #[cfg(test)]
    pub errno: Option<i32>,
    #[cfg(all(test, target_os = "macos"))]
    pub capacity: Option<usize>,
}

pub(super) fn install(command: &mut Command) -> io::Result<()> {
    install_with_options(command, Options::default())
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub(super) fn install_with_options(command: &mut Command, options: Options) -> io::Result<()> {
    // SAFETY: the callback only invokes a syscall and constructs an errno-only
    // io::Error. It allocates no memory, opens no descriptors, and takes no locks.
    unsafe {
        command.pre_exec(move || {
            #[cfg(test)]
            if let Some(errno) = options.errno {
                return Err(io::Error::from_raw_os_error(errno));
            }
            #[cfg(not(test))]
            let _ = &options;
            if libc::syscall(
                libc::SYS_close_range,
                3u32,
                u32::MAX,
                libc::CLOSE_RANGE_CLOEXEC,
            ) == 0
            {
                Ok(())
            } else {
                // No scanning fallback: failure must prevent exec.
                Err(io::Error::last_os_error())
            }
        });
    }
    Ok(())
}

#[cfg(any(target_os = "macos", test))]
pub(super) fn classify_listfds(ret: i32, bytes: i32, errno: i32) -> Result<usize, i32> {
    if ret <= 0 {
        return Err(if errno == 0 { libc::EIO } else { errno });
    }
    let record = 8;
    if ret % record != 0 || ret > bytes {
        return Err(libc::EIO);
    }
    if ret == bytes {
        return Err(libc::ENOBUFS);
    }
    Ok((ret / record) as usize)
}

#[cfg(target_os = "macos")]
pub(super) fn install_with_options(command: &mut Command, options: Options) -> io::Result<()> {
    use std::mem::size_of;
    // SAFETY: null with zero bytes is libproc's documented size probe. No buffer
    // is accessed. errno is saved immediately when the probe fails.
    let probe = unsafe {
        libc::proc_pidinfo(
            libc::getpid(),
            libc::PROC_PIDLISTFDS,
            0,
            std::ptr::null_mut(),
            0,
        )
    };
    if probe <= 0 {
        let errno = io::Error::last_os_error().raw_os_error().unwrap_or(0);
        return Err(io::Error::from_raw_os_error(if errno == 0 {
            libc::EIO
        } else {
            errno
        }));
    }
    let record = size_of::<libc::proc_fdinfo>();
    if record != 8 || probe as usize % record != 0 {
        return Err(io::Error::from_raw_os_error(libc::EIO));
    }
    let cap = (probe as usize / record)
        .checked_mul(2)
        .and_then(|count| count.checked_add(64))
        .map(|count| count.max(1024))
        .ok_or_else(|| io::Error::from_raw_os_error(libc::ENOBUFS))?;
    #[cfg(test)]
    let cap = options.capacity.unwrap_or(cap);
    let bytes = cap
        .checked_mul(record)
        .and_then(|bytes| i32::try_from(bytes).ok())
        .ok_or_else(|| io::Error::from_raw_os_error(libc::ENOBUFS))?;
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(cap)
        .map_err(|_| io::Error::from_raw_os_error(libc::ENOBUFS))?;
    buffer.resize_with(cap, || libc::proc_fdinfo {
        proc_fd: 0,
        proc_fdtype: 0,
    });
    // SAFETY: the aligned, fully initialized buffer is moved into the callback.
    // Its byte capacity is checked for overflow and the FFI c_int range. Child
    // code only uses libproc, getpid, fcntl, and errno access, with no allocation,
    // descriptor creation, locks, or buffer resizing after fork.
    unsafe {
        command.pre_exec(move || {
            let ret = libc::proc_pidinfo(
                libc::getpid(),
                libc::PROC_PIDLISTFDS,
                0,
                buffer.as_mut_ptr().cast(),
                bytes,
            );
            let errno = if ret <= 0 {
                io::Error::last_os_error().raw_os_error().unwrap_or(0)
            } else {
                0
            };
            let count =
                classify_listfds(ret, bytes, errno).map_err(io::Error::from_raw_os_error)?;
            for entry in &buffer[..count] {
                let fd = entry.proc_fd;
                if fd < 3 {
                    continue;
                }
                #[cfg(test)]
                if let Some(errno) = options.errno {
                    return Err(io::Error::from_raw_os_error(errno));
                }
                let flags = libc::fcntl(fd, libc::F_GETFD);
                if flags == -1 {
                    let error = io::Error::last_os_error();
                    if error.raw_os_error() == Some(libc::EBADF) {
                        continue;
                    }
                    return Err(error);
                }
                if flags & libc::FD_CLOEXEC == 0
                    && libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) == -1
                {
                    return Err(io::Error::last_os_error());
                }
            }
            #[cfg(not(test))]
            let _ = &options;
            Ok(())
        });
    }
    Ok(())
}

#[cfg(not(any(target_os = "macos", all(target_os = "linux", target_env = "gnu"))))]
pub(super) fn install_with_options(_command: &mut Command, _options: Options) -> io::Result<()> {
    Err(io::Error::from_raw_os_error(libc::ENOSYS))
}
