//! A direct native child with real tty descriptors, kept in its private owned PGID.
use super::protocol::failure;
use crate::adapter::{AdapterResult, ErrorKind, ProcessGroup, cleanup_group};
use rustix::{
    fs::{Mode, OFlags, open},
    io::{FdFlags, dup, fcntl_setfd},
    pty::{OpenptFlags, grantpt, openpt, ptsname, unlockpt},
    termios::{Winsize, tcsetwinsize},
};
use std::{
    ffi::OsString,
    os::fd::OwnedFd,
    path::Path,
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    process::Command,
    sync::{Mutex, mpsc},
    task::JoinHandle,
};

#[derive(Clone)]
pub(super) struct Terminal {
    fd: Arc<OwnedFd>,
    alive: Arc<AtomicBool>,
    writer: Arc<Mutex<()>>,
}
impl Terminal {
    pub fn live(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }
    pub async fn write(&self, bytes: &[u8]) -> AdapterResult<()> {
        if bytes.is_empty() || bytes.len() > 4096 {
            return Err(failure(
                ErrorKind::InvalidInput,
                "terminal input must contain 1..4096 bytes",
            ));
        }
        let _writer = tokio::time::timeout(Duration::from_secs(2), self.writer.lock())
            .await
            .map_err(|_| failure(ErrorKind::Timeout, "terminal input queue deadline exceeded"))?;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        let mut remaining = bytes;
        while !remaining.is_empty() {
            if !self.live() {
                return Err(failure(
                    ErrorKind::SessionLost,
                    "owned terminal is no longer live",
                ));
            }
            match rustix::io::write(&*self.fd, remaining) {
                Ok(0) => return Err(failure(ErrorKind::ProcessFailure, "terminal input closed")),
                Ok(n) => remaining = &remaining[n..],
                Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {
                    if tokio::time::Instant::now() >= deadline {
                        return Err(failure(
                            ErrorKind::Timeout,
                            "terminal input deadline exceeded",
                        ));
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
                Err(_) => return Err(failure(ErrorKind::ProcessFailure, "terminal input failed")),
            }
        }
        Ok(())
    }
    pub fn resize(&self, rows: u16, columns: u16) -> AdapterResult<()> {
        if !self.live() {
            return Err(failure(
                ErrorKind::SessionLost,
                "owned terminal is no longer live",
            ));
        }
        if !(10..=200).contains(&rows) || !(20..=400).contains(&columns) {
            return Err(failure(
                ErrorKind::InvalidInput,
                "terminal size outside bounds",
            ));
        }
        tcsetwinsize(
            &*self.fd,
            Winsize {
                ws_row: rows,
                ws_col: columns,
                ws_xpixel: 0,
                ws_ypixel: 0,
            },
        )
        .map_err(|_| failure(ErrorKind::ProcessFailure, "terminal resize failed"))
    }
}
pub(super) struct PtyTransport {
    group: Option<ProcessGroup>,
    pub terminal: Terminal,
    pub output: mpsc::Receiver<AdapterResult<Vec<u8>>>,
    reader: JoinHandle<()>,
}
impl PtyTransport {
    pub fn launch(
        executable: &Path,
        cwd: &Path,
        args: &[String],
        env: Vec<(OsString, OsString)>,
        flag: Arc<AtomicBool>,
    ) -> AdapterResult<Self> {
        let setup = || -> rustix::io::Result<_> {
            let master = openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY)?;
            fcntl_setfd(&master, FdFlags::CLOEXEC)?;
            grantpt(&master)?;
            unlockpt(&master)?;
            let name = ptsname(&master, Vec::new())?;
            let slave = open(
                name.as_c_str(),
                OFlags::RDWR | OFlags::NOCTTY | OFlags::CLOEXEC,
                Mode::empty(),
            )?;
            tcsetwinsize(
                &slave,
                Winsize {
                    ws_row: 30,
                    ws_col: 120,
                    ws_xpixel: 0,
                    ws_ypixel: 0,
                },
            )?;
            rustix::fs::fcntl_setfl(
                &master,
                rustix::fs::fcntl_getfl(&master)? | OFlags::NONBLOCK,
            )?;
            Ok((master, slave))
        };
        let (master, slave) = setup()
            .map_err(|_| failure(ErrorKind::LaunchFailure, "private tty allocation failed"))?;
        let stdout = dup(&slave)
            .map_err(|_| failure(ErrorKind::LaunchFailure, "tty descriptor duplicate failed"))?;
        let stderr = dup(&slave)
            .map_err(|_| failure(ErrorKind::LaunchFailure, "tty descriptor duplicate failed"))?;
        let mut command = Command::new(executable);
        command
            .args(args)
            .current_dir(cwd)
            .env_clear()
            .envs(env)
            .stdin(Stdio::from(slave))
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .process_group(0);
        let child = command
            .spawn()
            .map_err(|_| failure(ErrorKind::LaunchFailure, "native tty launch failed"))?;
        let group = ProcessGroup::new(child, flag)?;
        let terminal = Terminal {
            fd: Arc::new(master),
            alive: Arc::new(AtomicBool::new(true)),
            writer: Arc::new(Mutex::new(())),
        };
        let (tx, output) = mpsc::channel(16);
        let fd = terminal.fd.clone();
        let alive = terminal.alive.clone();
        let reader = tokio::spawn(async move {
            let mut buffer = [0; 4096];
            while alive.load(Ordering::SeqCst) {
                match rustix::io::read(&*fd, &mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(Ok(buffer[..n].to_vec())).await.is_err() {
                            break;
                        }
                    }
                    Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {
                        tokio::time::sleep(Duration::from_millis(10)).await
                    }
                    // macOS/Linux report EIO once the last slave closes.
                    Err(rustix::io::Errno::IO) => break,
                    Err(_) => {
                        let _ = tx
                            .send(Err(failure(
                                ErrorKind::ProcessFailure,
                                "terminal output read failed",
                            )))
                            .await;
                        break;
                    }
                }
            }
        });
        Ok(Self {
            group: Some(group),
            terminal,
            output,
            reader,
        })
    }
    pub fn pid(&self) -> u32 {
        self.group
            .as_ref()
            .expect("owned tty group")
            .child
            .id()
            .expect("unreaped tty leader")
    }
    pub async fn cleanup(mut self) -> AdapterResult<std::process::ExitStatus> {
        self.terminal.alive.store(false, Ordering::SeqCst);
        self.reader.abort();
        let _ = (&mut self.reader).await;
        let mut group = cleanup_group(self.group.take().expect("owned tty group")).await?;
        tokio::time::timeout(Duration::from_secs(5), group.reap())
            .await
            .map_err(|_| failure(ErrorKind::SessionLost, "tty reap deadline exceeded"))?
            .map_err(|_| failure(ErrorKind::SessionLost, "tty reap failed"))
    }
}
impl Drop for PtyTransport {
    fn drop(&mut self) {
        self.terminal.alive.store(false, Ordering::SeqCst);
        self.reader.abort();
    }
}
#[cfg(test)]
pub(super) fn plain_terminal(bytes: &[u8]) -> String {
    let mut text = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != 0x1b {
            text.push(bytes[index]);
            index += 1;
            continue;
        }
        index += 1;
        match bytes.get(index) {
            Some(b'[') => {
                index += 1;
                while index < bytes.len() {
                    let b = bytes[index];
                    index += 1;
                    if (0x40..=0x7e).contains(&b) {
                        break;
                    }
                }
            }
            Some(b']') => {
                index += 1;
                while index < bytes.len() {
                    if bytes[index] == 7 {
                        index += 1;
                        break;
                    }
                    if bytes[index] == 0x1b && bytes.get(index + 1) == Some(&b'\\') {
                        index += 2;
                        break;
                    }
                    index += 1;
                }
            }
            Some(_) => index += 1,
            None => {}
        }
    }
    String::from_utf8_lossy(&text).into_owned()
}
