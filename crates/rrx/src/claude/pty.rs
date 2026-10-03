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
    io::unix::AsyncFd,
    process::Command,
    sync::{Mutex, mpsc},
    task::JoinHandle,
};

#[derive(Clone)]
pub(super) struct Terminal {
    fd: Arc<AsyncFd<OwnedFd>>,
    alive: Arc<AtomicBool>,
    writer: Arc<Mutex<()>>,
    #[cfg(test)]
    reads: Arc<std::sync::atomic::AtomicUsize>,
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
            let mut ready = tokio::time::timeout_at(deadline, self.fd.writable())
                .await
                .map_err(|_| failure(ErrorKind::Timeout, "terminal input deadline exceeded"))?
                .map_err(|_| {
                    failure(ErrorKind::ProcessFailure, "terminal input readiness failed")
                })?;
            if !self.live() {
                return Err(failure(
                    ErrorKind::SessionLost,
                    "owned terminal is no longer live",
                ));
            }
            match ready.try_io(|fd| {
                rustix::io::write(fd.get_ref(), remaining).map_err(std::io::Error::from)
            }) {
                Ok(Ok(0)) => {
                    return Err(failure(ErrorKind::ProcessFailure, "terminal input closed"));
                }
                Ok(Ok(n)) => remaining = &remaining[n..],
                Ok(Err(error)) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Ok(Err(_)) => {
                    return Err(failure(ErrorKind::ProcessFailure, "terminal input failed"));
                }
                Err(_) => {}
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
            self.fd.get_ref(),
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
            fd: Arc::new(AsyncFd::new(master).map_err(|_| {
                failure(
                    ErrorKind::LaunchFailure,
                    "terminal readiness registration failed",
                )
            })?),
            alive: Arc::new(AtomicBool::new(true)),
            writer: Arc::new(Mutex::new(())),
            #[cfg(test)]
            reads: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        };
        let (tx, output) = mpsc::channel(16);
        let fd = terminal.fd.clone();
        let alive = terminal.alive.clone();
        #[cfg(test)]
        let reads = terminal.reads.clone();
        let reader = tokio::spawn(async move {
            let mut buffer = [0; 4096];
            while alive.load(Ordering::SeqCst) {
                let mut ready = match fd.readable().await {
                    Ok(ready) => ready,
                    Err(_) => {
                        let _ = tx
                            .send(Err(failure(
                                ErrorKind::ProcessFailure,
                                "terminal output readiness failed",
                            )))
                            .await;
                        break;
                    }
                };
                if !alive.load(Ordering::SeqCst) {
                    break;
                }
                #[cfg(test)]
                reads.fetch_add(1, Ordering::SeqCst);
                match ready.try_io(|fd| {
                    rustix::io::read(fd.get_ref(), &mut buffer).map_err(std::io::Error::from)
                }) {
                    Ok(Ok(0)) => break,
                    Ok(Ok(n)) => {
                        if tx.send(Ok(buffer[..n].to_vec())).await.is_err() {
                            break;
                        }
                    }
                    Ok(Err(error)) if error.kind() == std::io::ErrorKind::Interrupted => {}
                    // Both platforms can report EIO after the final slave closes.
                    Ok(Err(error))
                        if error.raw_os_error() == Some(rustix::io::Errno::IO.raw_os_error()) =>
                    {
                        break;
                    }
                    Ok(Err(_)) => {
                        let _ = tx
                            .send(Err(failure(
                                ErrorKind::ProcessFailure,
                                "terminal output read failed",
                            )))
                            .await;
                        break;
                    }
                    Err(_) => {}
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
    pub fn resize_owned(&mut self, rows: u16, columns: u16) -> AdapterResult<()> {
        self.terminal.resize(rows, columns)?;
        // Only this supervisor owns cleanup; no reap or group replacement can
        // occur while this mutable transport call holds the unreaped leader.
        let group = self
            .group
            .as_ref()
            .ok_or_else(|| failure(ErrorKind::SessionLost, "terminal group owner ended"))?;
        let pid = group
            .child
            .id()
            .and_then(|pid| rustix::process::Pid::from_raw(pid as i32))
            .ok_or_else(|| failure(ErrorKind::SessionLost, "terminal leader is no longer owned"))?;
        rustix::process::kill_process_group(pid, rustix::process::Signal::WINCH).map_err(|_| {
            failure(
                ErrorKind::ProcessFailure,
                "owned terminal resize signal failed",
            )
        })
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
#[cfg(test)]
mod tests {
    use super::*;
    async fn contains(transport: &mut PtyTransport, needle: &str) {
        let mut output = Vec::new();
        tokio::time::timeout(Duration::from_secs(3), async {
            while !String::from_utf8_lossy(&output).contains(needle) {
                output.extend(transport.output.recv().await.unwrap().unwrap());
                assert!(output.len() < 128 * 1024);
            }
        })
        .await
        .unwrap();
    }
    fn fixture(script: &str) -> (tempfile::TempDir, PtyTransport, Arc<AtomicBool>) {
        let temp = tempfile::tempdir().unwrap();
        let flag = Arc::new(AtomicBool::new(false));
        let args = vec!["-u".into(), "-c".into(), script.into()];
        let transport = PtyTransport::launch(
            Path::new("/usr/bin/python3"),
            temp.path(),
            &args,
            std::env::vars_os().collect(),
            flag.clone(),
        )
        .unwrap();
        (temp, transport, flag)
    }
    #[tokio::test]
    async fn readiness_idle_input_resize_and_stop_have_bounded_owned_lifecycle() {
        let (_temp, mut transport, flag) = fixture(
            "import os,sys,signal;assert all(os.isatty(n) for n in (0,1,2));signal.signal(signal.SIGWINCH,lambda *_:print('RESIZED='+str(os.get_terminal_size(1)),flush=True));print('READY');line=sys.stdin.readline();print('ACK:'+line.strip());sys.stdin.readline()",
        );
        contains(&mut transport, "READY\r\n").await;
        // Let the initial readable edge drain; subsequent idle waiting must
        // issue no periodic read syscall and publish no fabricated output.
        tokio::time::sleep(Duration::from_millis(100)).await;
        let reads = transport.terminal.reads.load(Ordering::SeqCst);
        assert!(
            tokio::time::timeout(Duration::from_millis(150), transport.output.recv())
                .await
                .is_err()
        );
        assert_eq!(transport.terminal.reads.load(Ordering::SeqCst), reads);
        transport.resize_owned(40, 100).unwrap();
        contains(&mut transport, "columns=100, lines=40").await;
        transport.terminal.write(b"hello\n").await.unwrap();
        contains(&mut transport, "ACK:hello").await;
        let terminal = transport.terminal.clone();
        tokio::time::timeout(Duration::from_secs(5), transport.cleanup())
            .await
            .unwrap()
            .unwrap();
        assert!(!flag.load(Ordering::SeqCst));
        assert!(!terminal.live());
        assert_eq!(
            terminal.write(b"again\n").await.unwrap_err().kind,
            ErrorKind::SessionLost
        );
    }
    #[tokio::test]
    async fn output_backpressure_and_input_cancellation_do_not_delay_group_cleanup() {
        let (_temp, mut transport, flag) = fixture(
            "import os,termios,time;mode=termios.tcgetattr(0);mode[3]&=~(termios.ICANON|termios.ECHO);termios.tcsetattr(0,termios.TCSANOW,mode);os.write(1,b'x'*5000000);time.sleep(60)",
        );
        let first = tokio::time::timeout(Duration::from_secs(3), transport.output.recv())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(
            !first.is_empty() && first.iter().all(|b| *b == b'x'),
            "fixture must enter output flood before measuring backpressure"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(transport.output.len() <= 16);
        assert!(!transport.output.is_empty());
        let terminal = transport.terminal.clone();
        let writer = tokio::spawn(async move {
            loop {
                if terminal.write(&[b'a'; 4096]).await.is_err() {
                    return;
                }
            }
        });
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert!(
            !writer.is_finished(),
            "backpressured input must await readiness"
        );
        writer.abort();
        let _ = writer.await;
        tokio::time::timeout(Duration::from_secs(5), transport.cleanup())
            .await
            .unwrap()
            .unwrap();
        assert!(!flag.load(Ordering::SeqCst));
    }
    #[test]
    fn ansi_metadata_does_not_hide_a_trust_screen_or_echo_response_marker() {
        assert_eq!(plain_terminal(b"Yes, I \x1b[1mtrust\x1b[0m this folder\x1b]8;;https://example.invalid/secret\x07label\x1b]8;;\x07"),"Yes, I trust this folderlabel");
    }
}
