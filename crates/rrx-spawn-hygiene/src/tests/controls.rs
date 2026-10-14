use super::{
    eof_observation::{
        DeliveryAttempt, Fault, JoinedReader, LiveObservation, LivePoint, ReadPurpose, ReaderExit,
        WorkerObservation, classify_inheritance, poll_read, receive_reader, worker_input,
    },
    owner::{Classification, WorkerGroupOwner},
};
use crate::{
    output_std, output_tokio,
    platform::{self, Options},
    spawn_std, spawn_tokio, status_std, status_tokio,
};
use std::{
    io::{self, Read, Write},
    os::fd::{AsRawFd, OwnedFd},
    os::unix::process::ExitStatusExt,
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

struct InnerOwner {
    child: Child,
    readers: Vec<JoinHandle<ReaderExit>>,
    joined: Vec<JoinedReader>,
    cancel: Arc<AtomicBool>,
    reaped: bool,
    observation_lost: bool,
    closed: bool,
}
impl InnerOwner {
    fn new(child: Child) -> Self {
        Self {
            child,
            readers: vec![],
            joined: vec![],
            cancel: Arc::new(AtomicBool::new(false)),
            reaped: false,
            observation_lost: false,
            closed: false,
        }
    }

    fn observe_live(&mut self, point: LivePoint, fault: Fault) -> LiveObservation {
        match self.child.try_wait() {
            Ok(Some(_)) => {
                self.reaped = true;
                LiveObservation::Exited
            }
            Ok(None) => fault.live_observation(point),
            Err(error) => {
                // A synthetic classification fault never changes custody.
                // Actual loss of the Child observation cannot authorize cleanup.
                self.observation_lost = true;
                LiveObservation::Error(error.into())
            }
        }
    }

    fn wait(&mut self) -> io::Result<std::process::ExitStatus> {
        match self.child.wait() {
            Ok(status) => {
                self.reaped = true;
                Ok(status)
            }
            Err(error) => {
                self.observation_lost = true;
                Err(error)
            }
        }
    }

    fn hold(&self) -> ! {
        // The existing outer worker owner remains responsible for containment.
        // This owner neither returns nor signals/reaps after observation loss.
        loop {
            let _ = io::stderr().write_all(b"P4C HELD inner observation\n");
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    fn close(&mut self) -> &[JoinedReader] {
        if self.closed {
            return &self.joined;
        }
        if self.observation_lost {
            self.hold();
        }
        self.cancel.store(true, Ordering::Release);
        self.child.stdin.take();
        if !self.reaped {
            if let Err(error) = self.child.kill()
                && error.kind() == io::ErrorKind::PermissionDenied
            {
                self.observation_lost = true;
                self.hold();
            }
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                match self.child.try_wait() {
                    Ok(Some(_)) => {
                        self.reaped = true;
                        break;
                    }
                    Err(_) => {
                        self.observation_lost = true;
                        self.hold();
                    }
                    Ok(None) if Instant::now() >= deadline => {
                        let _ = io::stderr().write_all(b"P4C STALL inner reap\n");
                        std::process::abort();
                    }
                    Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                }
            }
        }
        for reader in self.readers.drain(..) {
            let deadline = Instant::now() + Duration::from_secs(2);
            while !reader.is_finished() {
                if Instant::now() >= deadline {
                    let _ = io::stderr().write_all(b"P4C STALL reader join\n");
                    std::process::abort();
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            self.joined.push(match reader.join() {
                Ok(exit) => JoinedReader::Completed(exit),
                Err(_) => JoinedReader::Panicked,
            });
        }
        self.closed = true;
        &self.joined
    }

    fn close_without_readers(&mut self) -> Result<(), String> {
        let joined = self.close();
        if joined.is_empty() {
            Ok(())
        } else {
            Err(format!("SETUP unexpected reader joins: {joined:?}"))
        }
    }
}
impl Drop for InnerOwner {
    fn drop(&mut self) {
        // close is idempotent and retains every successful/panicked join.
        // Inheritance inspects the retained results before emitting its marker.
        self.close();
    }
}

fn inheritance(cloexec: bool, high: bool, fault: Fault) -> WorkerObservation {
    match inheritance_observation(cloexec, high, fault) {
        Ok(observation) => observation,
        Err(reason) => WorkerObservation::Setup(reason),
    }
}

fn inheritance_observation(
    cloexec: bool,
    high: bool,
    fault: Fault,
) -> Result<WorkerObservation, String> {
    let (read, mut write) = rustix::pipe::pipe().map_err(|e| format!("SETUP pipe {e}"))?;
    if high {
        // This dedicated worker normalizes its own soft limit to a realizable
        // fixture size. Some macOS launchers advertise an enormous soft limit
        // beyond the kernel's actual usable descriptor table. Other tests and
        // the harness keep their existing limits and concurrency unchanged.
        use rustix::process::{Resource, getrlimit, setrlimit};
        let mut limit = getrlimit(Resource::Nofile);
        println!("P4C LIMIT before={limit:?}");
        limit.current = Some(limit.current.unwrap_or(1024).min(1024));
        setrlimit(Resource::Nofile, limit).map_err(|e| format!("SETUP setrlimit {e}"))?;
        println!("P4C LIMIT after={:?}", getrlimit(Resource::Nofile));
        let target = i32::try_from(
            getrlimit(Resource::Nofile)
                .current
                .ok_or("SETUP unlimited fd")?
                .saturating_sub(1),
        )
        .map_err(|_| "SETUP fd limit")?;
        write = rustix::io::fcntl_dupfd_cloexec(&write, target)
            .map_err(|e| format!("SETUP high dup fd={target}: {e}"))?;
        if write.as_raw_fd() != target {
            return Err(format!(
                "SETUP high fd differs: expected {target}, got {}",
                write.as_raw_fd()
            ));
        }
        println!("P4C FD high={target}");
    }
    rustix::io::fcntl_setfd(
        &write,
        if cloexec {
            rustix::io::FdFlags::CLOEXEC
        } else {
            rustix::io::FdFlags::empty()
        },
    )
    .map_err(|e| format!("SETUP fd flags {e}"))?;
    let mut command = Command::new("/bin/sh");
    command
        .args([
            "-c",
            match fault {
                Fault::ReadyMissing => "printf 'wrong\\n'; read go",
                Fault::ReadyDeadline => "read go",
                Fault::ReadyClosed => "exec 1>&-; read go",
                _ => "printf 'ready\\n'; read go",
            },
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    // Inner children deliberately remain in the raw worker group. The outer
    // owner contains spawn itself, including a stalled pre_exec callback.
    let mut owner = InnerOwner::new(spawn_std(command).map_err(|e| format!("SETUP spawn {e}"))?);
    let pid = rustix::process::Pid::from_raw(owner.child.id() as i32).ok_or("SETUP pid")?;
    let in_group = rustix::process::getpgid(Some(pid)) == Ok(rustix::process::getpgrp());
    let stdout = owner.child.stdout.take().ok_or("SETUP stdout")?;
    let ready = poll_read(
        std::fs::File::from(OwnedFd::from(stdout)),
        &owner.cancel,
        ReadPurpose::Ready,
        fault,
    );
    let before = owner.observe_live(LivePoint::Before, fault);
    // Keep the existing ready/before -> parent writer close -> EOF/after order.
    drop(write);
    let read = std::fs::File::from(read);
    let cancel = owner.cancel.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    // A finite transport-timeout control retains a sender while the actual
    // reader completes without publishing. No timer or authority is added.
    let keepalive = (fault == Fault::TransportTimeout).then(|| tx.clone());
    owner.readers.push(std::thread::spawn(move || {
        if fault == Fault::ReaderPanic {
            panic!("synthetic EOF reader panic");
        }
        if fault == Fault::Cancel {
            cancel.store(true, Ordering::Release);
        }
        let read = poll_read(read, &cancel, ReadPurpose::Eof, fault);
        let delivery = if matches!(fault, Fault::Disconnect | Fault::TransportTimeout) {
            drop(tx);
            DeliveryAttempt::NotSent
        } else {
            match tx.send(read) {
                Ok(()) => DeliveryAttempt::Sent,
                Err(_) => DeliveryAttempt::Failed,
            }
        };
        ReaderExit { read, delivery }
    }));
    let transport = receive_reader(&rx, &owner.readers[0]);
    drop(keepalive);
    drop(rx);
    let after = owner.observe_live(LivePoint::After, fault);
    let joined = owner.close();
    // Written only after inner recovery and all reader joins. These values are
    // classification evidence, never process/group authority.
    println!(
        "P4C EOF observations ready={ready:?} before={before:?} after={after:?} transport={transport:?} joined={joined:?}"
    );
    Ok(classify_inheritance(
        in_group, ready, before, after, transport, joined,
    ))
}

#[allow(
    clippy::disallowed_methods,
    reason = "test-only error injection uses the same installed callback and original std spawn"
)]
fn expect_spawn_err(
    mut command: Command,
    options: Options,
    errno: i32,
    marker: &std::path::Path,
) -> Result<(), String> {
    platform::install_with_options(&mut command, options)
        .map_err(|e| format!("SETUP prepare {e}"))?;
    let result = command.spawn();
    let error = match result {
        Err(error) => error,
        Ok(child) => {
            let mut owner = InnerOwner::new(child);
            let _ = io::stderr()
                .write_all(format!("P4C UNEXPECTED_CHILD pid={}\n", owner.child.id()).as_bytes());
            owner.close_without_readers()?;
            return Err("unexpected Child".into());
        }
    };
    if error.raw_os_error() != Some(errno) {
        return Err(format!(
            "expected errno {errno}, got {:?}",
            error.raw_os_error()
        ));
    }
    if marker.exists() {
        return Err("helper marker created".into());
    }
    // This worker has no other child. Observation only; no wait-any reap or
    // SIGCHLD ignore can steal another owner's leader.
    match rustix::process::waitid(
        rustix::process::WaitId::Pgid(Some(rustix::process::getpgrp())),
        rustix::process::WaitIdOptions::EXITED
            | rustix::process::WaitIdOptions::NOWAIT
            | rustix::process::WaitIdOptions::NOHANG,
    ) {
        Err(rustix::io::Errno::CHILD) => Ok(()),
        other => Err(format!("child/zombie remains after Err: {other:?}")),
    }
}

fn stdio() -> Result<(), String> {
    let mut command = Command::new("/bin/sh");
    command
        .args([
            "-c",
            "read input; printf 'out:%s' \"$input\"; printf err >&2",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut owner = InnerOwner::new(spawn_std(command).map_err(|e| format!("stdio spawn {e}"))?);
    owner
        .child
        .stdin
        .as_mut()
        .ok_or("stdin absent")?
        .write_all(b"hello\n")
        .map_err(|e| e.to_string())?;
    owner.child.stdin.take();
    let mut stdout = String::new();
    let mut stderr = String::new();
    owner
        .child
        .stdout
        .as_mut()
        .ok_or("stdout absent")?
        .read_to_string(&mut stdout)
        .map_err(|e| e.to_string())?;
    owner
        .child
        .stderr
        .as_mut()
        .ok_or("stderr absent")?
        .read_to_string(&mut stderr)
        .map_err(|e| e.to_string())?;
    let status = owner.wait().map_err(|e| format!("SETUP stdio wait {e}"))?;
    owner.close_without_readers()?;
    if !status.success() || stdout != "out:hello" || stderr != "err" {
        return Err("stdio changed".into());
    }
    let mut default = Command::new("/bin/sh");
    default.args(["-c", "cat; printf out; printf err >&2"]);
    let out = output_std(default).map_err(|e| e.to_string())?;
    if out.stdout != b"out" || out.stderr != b"err" {
        return Err("std output defaults".into());
    }
    let mut explicit = Command::new("/bin/sh");
    explicit
        .args(["-c", "printf ignored"])
        .stdout(Stdio::null());
    if !output_std(explicit)
        .map_err(|e| e.to_string())?
        .stdout
        .is_empty()
    {
        return Err("std explicit stdout overridden".into());
    }
    if !status_std(Command::new("/usr/bin/true"))
        .map_err(|e| e.to_string())?
        .success()
    {
        return Err("std status".into());
    }
    let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    runtime.block_on(async {
        let mut command = tokio::process::Command::new("/bin/sh");
        command
            .args(["-c", "printf out; printf err >&2"])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let out = output_tokio(command).await.map_err(|e| e.to_string())?;
        if out.stdout != b"out" || out.stderr != b"err" {
            return Err("Tokio output override changed".into());
        }
        let mut command = tokio::process::Command::new("/usr/bin/true");
        command.kill_on_drop(true);
        let mut child = spawn_tokio(command).map_err(|e| e.to_string())?;
        if !child.wait().await.map_err(|e| e.to_string())?.success() {
            return Err("Tokio spawn".into());
        }
        if !status_tokio(tokio::process::Command::new("/usr/bin/true"))
            .await
            .map_err(|e| e.to_string())?
            .success()
        {
            return Err("Tokio status".into());
        }
        Ok(())
    })
}

#[test]
fn worker() {
    let Ok(case) = std::env::var("RRX_P4_CASE") else {
        return;
    };
    let directory = std::path::PathBuf::from(std::env::var_os("RRX_P4_DIR").unwrap());
    if let Some(api) = case.strip_prefix("API_") {
        super::api_wiring::run_worker(api, &directory);
        return;
    }
    if let Some(input) = worker_input(&case) {
        inheritance(input.cloexec, input.high, input.fault).emit();
        return;
    }
    let marker = directory.join("marker");
    let mut command = Command::new("/bin/sh");
    command.args(["-c", "touch \"$1\"", "p4"]).arg(&marker);
    let result = match case.as_str() {
        "C2b" => expect_spawn_err(
            Command::new(directory.join("absent-program")),
            Options::default(),
            libc::ENOENT,
            &marker,
        ),
        "C2c" => expect_spawn_err(
            command,
            Options {
                errno: Some(libc::EIO),
                ..Default::default()
            },
            libc::EIO,
            &marker,
        ),
        #[cfg(target_os = "linux")]
        "C2d" => expect_spawn_err(
            command,
            Options {
                errno: Some(libc::ENOSYS),
            },
            libc::ENOSYS,
            &marker,
        ),
        #[cfg(target_os = "macos")]
        "C2e" => expect_spawn_err(
            command,
            Options {
                capacity: Some(1),
                ..Default::default()
            },
            libc::ENOBUFS,
            &marker,
        ),
        "C2g" => stdio(),
        "LISTFDS" => crate::tests::check_listfds(),
        "OW2" => leave_member(),
        _ => Err("SETUP unknown case".into()),
    };
    match result {
        Ok(()) => println!("P4C RESULT pass"),
        Err(reason) if reason.starts_with("SETUP") => println!("P4C SETUP fail:{reason}"),
        Err(reason) => println!("P4C RESULT fail:{reason}"),
    }
}

#[allow(
    clippy::disallowed_methods,
    reason = "OW2 deliberately leaves a raw synthetic member for the outer group owner; regular-file stdio only"
)]
fn leave_member() -> Result<(), String> {
    let mut command = Command::new("/bin/sleep");
    command
        .arg("60")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // Deliberately retained by the outer group, never turned into an authority
    // PID from worker output. Leader remains owned until the final group signal.
    let _child = command.spawn().map_err(|e| format!("SETUP member {e}"))?;
    Ok(())
}

pub(super) fn run(case: &str) {
    let owner = WorkerGroupOwner::start(case)
        .unwrap_or_else(|error| panic!("P4C SETUP before child: {error}"));
    let report = owner.finish();
    println!(
        "P4C WORKER stdout={} stderr={}",
        report.stdout, report.stderr
    );
    println!(
        "P4C HARNESS case={case} classification={:?} complete={} signal={:?} stopped={} trace={:?}",
        report.classification,
        report.complete,
        report.status.signal(),
        report.stopped,
        report.trace
    );
    assert!(report.complete, "delayed recovery is not a pass");
    assert_eq!(report.classification, Classification::Pass);
    assert_eq!(
        &report.trace[report.trace.len() - 3..],
        &["S4:accepted", "S6:reaped", "S7:ESRCH"]
    );
}

#[test]
fn c1() {
    run("C1");
}
#[test]
fn c2a() {
    run("C2a");
}
#[test]
fn c2b() {
    run("C2b");
}
#[test]
fn c2c() {
    run("C2c");
}
#[cfg(target_os = "linux")]
#[test]
fn c2d() {
    run("C2d");
}
#[cfg(target_os = "macos")]
#[test]
fn c2e() {
    run("C2e");
}
#[test]
fn c2f() {
    run("C2f");
}
#[test]
fn c2g() {
    run("C2g");
}
#[test]
fn ow2() {
    run("OW2");
}
