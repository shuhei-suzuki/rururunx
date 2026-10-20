use super::*;
use std::{
    io::{BufRead, Read, Write},
    os::unix::process::CommandExt,
    process::{Child, Command, Stdio},
};

struct Owned(Child);
impl Drop for Owned {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn spawn(mut command: Command) -> Owned {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    Owned(rrx_spawn_hygiene::spawn_std(command).unwrap())
}
fn cookie() -> Cookie {
    Cookie::new(&format!("d20-host-cookie-{}", std::process::id())).unwrap()
}
fn wait_for(mut condition: impl FnMut() -> bool) {
    let start = Instant::now();
    while !condition() {
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "host premise deadline"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn query_key(pid: u32) -> (u64, u64) {
    query_birth(pid, &query_pid(pid), true).unwrap().birth
}
fn kill_recorded(pid: u32, birth: (u64, u64)) {
    assert_eq!(query_key(pid), birth, "fixture identity changed");
    rustix::process::kill_process(
        rustix::process::Pid::from_raw(pid as i32).unwrap(),
        rustix::process::Signal::KILL,
    )
    .unwrap();
}

/// Platform sleep is absent while an executable built from this crate observes its cookie.
#[test]
fn he01_environment_premise() {
    if std::env::var_os("RRX_D20_HE1_CHILD").is_some() {
        let cookie = Cookie::new(&std::env::var("RRX_PROCESS_COOKIE").unwrap()).unwrap();
        assert_eq!(
            cookie_matches(
                std::process::id(),
                &cookie,
                Limits::default().environment_bytes
            )
            .unwrap(),
            Environment::Match
        );
        return;
    }
    let cookie = cookie();
    let mut command = Command::new("/bin/sleep");
    command.arg("30").env("RRX_PROCESS_COOKIE", &cookie.0);
    let child = spawn(command);
    wait_for(|| process_path(child.0.id()).is_ok_and(|p| p == std::path::Path::new("/bin/sleep")));
    assert_eq!(
        cookie_matches(child.0.id(), &cookie, Limits::default().environment_bytes).unwrap(),
        Environment::Absent
    );
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "macos::host::he01_environment_premise"])
        .env("RRX_D20_HE1_CHILD", "1")
        .env("RRX_PROCESS_COOKIE", &cookie.0);
    assert!(spawn(command).0.wait().unwrap().success());
}

/// Public SDK layout and cross-UID queries expose the expected parent field.
#[test]
fn he02_parent_premise() {
    assert_eq!(std::mem::offset_of!(KinfoProc, ppid), 560);
    assert_eq!(std::mem::offset_of!(KinfoProc, flag), 32);
    assert_eq!(size_of::<KinfoProc>(), 648);
    let root = query_pid(1);
    assert_eq!(root.status, 0);
    assert_eq!(root.len, size_of::<KinfoProc>());
    assert_eq!(root.info.pid, 1);
    let own = query_pid(std::process::id());
    assert_eq!(own.status, 0);
    assert_eq!(own.len, size_of::<KinfoProc>());
    assert_eq!(
        own.info.ppid,
        rustix::process::getppid().unwrap().as_raw_nonzero().get()
    );
}

/// Boot identity is readable and stable within the current boot.
#[test]
fn he03_boot_premise() {
    let a = boot_id().expect("boot identity unavailable");
    assert_eq!(a.len(), 36);
    assert_eq!(boot_id().as_ref(), Some(&a));
}

/// An unreaped owned leader retains its birth across Seatbelt exec and zombie states.
#[test]
fn he04_exec_premise() {
    let cookie = cookie();
    let mut command = Command::new("/usr/bin/sandbox-exec");
    command
        .args([
            "-p",
            "(version 1)(allow default)",
            "/usr/bin/perl",
            "-e",
            "$|=1; print \"r\"; sysread(STDIN, $b, 1); exec \"/bin/sleep\", \"1\"",
        ])
        .env("RRX_PROCESS_COOKIE", &cookie.0);
    let mut child = spawn(command);
    let pid = child.0.id();
    let mut ready = [0];
    child
        .0
        .stdout
        .as_mut()
        .unwrap()
        .read_exact(&mut ready)
        .unwrap();
    assert_eq!(&ready, b"r");
    assert_eq!(
        process_path(pid).unwrap(),
        std::path::Path::new("/usr/bin/perl")
    );
    let b0 = owned_leader(pid, &cookie).unwrap().birth;
    child.0.stdin.as_mut().unwrap().write_all(b"x").unwrap();
    wait_for(|| process_path(pid).is_ok_and(|p| p == std::path::Path::new("/bin/sleep")));
    let b1 = owned_leader(pid, &cookie).unwrap().birth;
    wait_for(|| query_birth(pid, &query_pid(pid), true).is_some_and(|p| p.zombie));
    let b2 = owned_leader(pid, &cookie).unwrap();
    assert!(b2.zombie);
    assert_eq!(b0, b1);
    assert_eq!(b1, b2.birth);
}

/// A recent platform orphan is invisible to legacy discovery but limits launched discovery.
#[test]
fn he05_platform_escape() {
    let cookie = cookie();
    let launch = launch_mark().unwrap();
    let mut command = Command::new("/usr/bin/perl");
    command.args(["-e","$|=1; use POSIX; if (fork) { exit } POSIX::setsid(); print \"$$\\n\"; close STDOUT; sleep 30"]).env("RRX_PROCESS_COOKIE",&cookie.0).process_group(0);
    let mut leader = spawn(command);
    let group = leader.0.id();
    let mut line = String::new();
    std::io::BufReader::new(leader.0.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    let pid = line.trim().parse::<u32>().unwrap();
    let birth = query_key(pid);
    wait_for(|| query_birth(group, &query_pid(group), true).is_some_and(|p| p.zombie));
    match rustix::process::kill_process_group(
        rustix::process::Pid::from_raw(group as i32).unwrap(),
        rustix::process::Signal::KILL,
    ) {
        Err(rustix::io::Errno::PERM)
            if query_birth(group, &query_pid(group), true).is_some_and(|p| p.zombie) => {}
        result => result.unwrap(),
    }
    leader.0.wait().unwrap();
    wait_for(|| {
        rustix::process::test_kill_process_group(
            rustix::process::Pid::from_raw(group as i32).unwrap(),
        ) == Err(rustix::io::Errno::SRCH)
    });
    let assertions = std::panic::catch_unwind(|| {
        assert_eq!(identity(pid).unwrap().ppid, 1);
        let old = discover(&cookie, Limits::default(), Instant::now()).unwrap();
        assert!(!old.coverage.limited());
        assert!(old.processes.is_empty());
        let new = discover_launched(&cookie, Limits::default(), Instant::now(), &launch).unwrap();
        assert!(new.coverage.limited());
        assert!(new.coverage.ancestry_unconfirmed >= 1);
        assert!(new.unconfirmed.iter().any(|i| i.pid == pid));
    });
    kill_recorded(pid, birth);
    assertions.unwrap();
}

/// Observable nonplatform escaped descendants remain cookie matches.
#[test]
fn he06_observable_escape() {
    let mode = std::env::var("RRX_D20_HE6_MODE").ok();
    if mode.as_deref() == Some("child") {
        // SAFETY: setsid only changes this explicitly spawned fixture process.
        assert!(unsafe { libc::setsid() } > 0);
        println!("READY:{}", std::process::id());
        std::io::stdout().flush().unwrap();
        let mut byte = [0];
        assert_eq!(std::io::stdin().read(&mut byte).unwrap(), 1);
        return;
    }
    if mode.as_deref() == Some("leader") {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "macos::host::he06_observable_escape",
                "--nocapture",
            ])
            .env("RRX_D20_HE6_MODE", "child");
        // Keep the parent's inherited stdin/stdout pipes in the escaped child.
        let _child = rrx_spawn_hygiene::spawn_std(command).unwrap();
        std::process::exit(0);
    }
    let cookie = cookie();
    let launch = launch_mark().unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "macos::host::he06_observable_escape",
            "--nocapture",
        ])
        .env("RRX_D20_HE6_MODE", "leader")
        .env("RRX_PROCESS_COOKIE", &cookie.0)
        .process_group(0);
    let mut leader = spawn(command);
    let mut reader = std::io::BufReader::new(leader.0.stdout.take().unwrap());
    let mut line = String::new();
    let pid = loop {
        line.clear();
        assert!(reader.read_line(&mut line).unwrap() > 0);
        if let Some((_, pid)) = line.trim().split_once("READY:") {
            break pid.parse::<u32>().unwrap();
        }
    };
    let birth = query_key(pid);
    let group = rustix::process::Pid::from_raw(leader.0.id() as i32).unwrap();
    wait_for(|| {
        query_birth(leader.0.id(), &query_pid(leader.0.id()), true).is_some_and(|p| p.zombie)
    });
    match rustix::process::kill_process_group(group, rustix::process::Signal::KILL) {
        Err(rustix::io::Errno::PERM)
            if query_birth(leader.0.id(), &query_pid(leader.0.id()), true)
                .is_some_and(|p| p.zombie) => {}
        result => result.unwrap(),
    }
    let stdin = leader.0.stdin.take().unwrap();
    leader.0.wait().unwrap();
    wait_for(|| rustix::process::test_kill_process_group(group) == Err(rustix::io::Errno::SRCH));
    wait_for(|| identity(pid).is_ok_and(|i| i.ppid == 1));
    let found = discover_launched(&cookie, Limits::default(), Instant::now(), &launch).unwrap();
    let matched = found.processes.iter().any(|p| p.identity().pid == pid);
    kill_recorded(pid, birth);
    drop(stdin);
    assert!(matched);
}

/// Reparenting is complete while the original parent is still an unreaped zombie.
#[test]
fn he10_reparent_premise() {
    let mut windows = 0;
    for _ in 0..1000 {
        let mut command = Command::new("/usr/bin/perl");
        command.args([
            "-e",
            "$|=1; if (my $g = fork) { print \"$g\\n\"; exit } close STDOUT; sleep 30",
        ]);
        let mut parent = spawn(command);
        let p = parent.0.id();
        let mut line = String::new();
        std::io::BufReader::new(parent.0.stdout.take().unwrap())
            .read_line(&mut line)
            .unwrap();
        let g = line.trim().parse::<u32>().unwrap();
        let birth = identity(g).unwrap().birth;
        let assertions = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let start = Instant::now();
            loop {
                let parent = query_pid(p);
                let child = identity(g).unwrap();
                assert_eq!(child.birth, birth);
                if query_birth(p, &parent, true).is_some_and(|b| b.zombie) {
                    assert_eq!(child.ppid, 1, "child not reparented before parent zombie");
                    assert!(query_birth(p, &query_pid(p), true).unwrap().zombie);
                    break;
                }
                if child.ppid == p {
                    windows += 1;
                }
                assert!(start.elapsed() < Duration::from_secs(5));
            }
        }));
        parent.0.wait().unwrap();
        kill_recorded(g, birth);
        assertions.unwrap();
    }
    eprintln!("HE10 iterations=1000 observed_parent_windows={windows}");
}
