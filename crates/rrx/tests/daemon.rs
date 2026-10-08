//! #81 S1 controls through the compiled `rrx daemon` entry: the status table
//! (C-S1c), probe vs start (C-S1c2), old descriptors (C-S1e2) and no TCP
//! (C-S1g). The flock is the only arbiter; no PID is used as authority.
use serde_json::Value;
use std::{
    fs::OpenOptions,
    io::Write,
    os::unix::fs::{DirBuilderExt, FileTypeExt, OpenOptionsExt},
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{Duration, Instant},
};

struct Fixture {
    _dir: tempfile::TempDir,
    base: PathBuf,
    state: PathBuf,
    config: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("rrx-d-")
            .tempdir_in("/tmp")
            .unwrap();
        let base = dir.path().canonicalize().unwrap();
        let config = base.join("config.toml");
        std::fs::write(
            &config,
            "[agents.worker]\nprovider='claude'\ncommand=['/usr/bin/false']\n",
        )
        .unwrap();
        Self {
            state: base.join("state.db"),
            base,
            config,
            _dir: dir,
        }
    }
    fn daemon(&self, verb: &str) -> (Output, Value) {
        let output = Command::new(env!("CARGO_BIN_EXE_rrx"))
            .arg("--state")
            .arg(&self.state)
            .arg("--config")
            .arg(&self.config)
            .args(["daemon", verb, "--json"])
            .current_dir(&self.base)
            .output()
            .unwrap();
        let value = serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
            panic!(
                "daemon {verb}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (output, value)
    }
    fn status(&self) -> Value {
        self.daemon("status").1
    }
    fn start(&self) -> Value {
        self.daemon("start").1
    }
    fn start_running(&self) -> Value {
        let started = self.start();
        assert_eq!(started["start"], "running", "{started}");
        started["identity"].clone()
    }
    fn stop_and_wait(&self) {
        let stopped = self.daemon("stop").1;
        assert_eq!(stopped["stop"], "completed", "{stopped}");
        let deadline = Instant::now() + Duration::from_secs(30);
        while self.status()["status"] != "not_running" {
            assert!(Instant::now() < deadline, "SETUP: owner not released");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    fn root(&self) -> PathBuf {
        self.base.join("state.db.execution")
    }
    fn descriptor(&self) -> PathBuf {
        self.root().join("control/endpoint.json")
    }
    fn write_descriptor(&self, bytes: &[u8]) {
        for (dir, mode) in [(self.root(), 0o755), (self.root().join("control"), 0o700)] {
            match std::fs::DirBuilder::new().mode(mode).create(&dir) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => panic!("{e}"),
            }
        }
        let _ = std::fs::remove_file(self.descriptor());
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(self.descriptor())
            .unwrap()
            .write_all(bytes)
            .unwrap();
    }
    /// A strictly shaped descriptor whose socket does not exist.
    fn stale(&self, protocol: u32, state: &Path) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "protocol": protocol,
            "identity": {
                "state": state,
                "instance": uuid::Uuid::new_v4().to_string(),
                "epoch": 1,
            },
            "socket": self.base.join("absent.sock"),
        }))
        .unwrap()
    }
    /// The test holds the owner lock only; no Store, epoch or endpoint.
    fn hold_owner_lock(&self) -> std::fs::File {
        match std::fs::DirBuilder::new().mode(0o755).create(self.root()) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => panic!("{e}"),
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(self.root().join("owner.lock"))
            .unwrap();
        rustix::fs::flock(&lock, rustix::fs::FlockOperation::NonBlockingLockExclusive).unwrap();
        lock
    }
    fn epoch_row(&self) -> Option<(String, i64)> {
        if !self.state.exists() {
            return None;
        }
        let connection = rusqlite::Connection::open_with_flags(
            &self.state,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        connection
            .query_row(
                "SELECT instance_id,epoch FROM runtime_epoch WHERE singleton=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok()
    }
    /// The detached service of this state root, found only to inspect it.
    fn service_pid(&self) -> u32 {
        let output = Command::new("pgrep")
            .arg("-f")
            .arg(format!("{} --config", self.state.display()))
            .output()
            .unwrap();
        let pids: Vec<u32> = String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .map(|p| p.parse().unwrap())
            .collect();
        assert_eq!(pids.len(), 1, "SETUP: one detached service: {pids:?}");
        pids[0]
    }
}
impl Drop for Fixture {
    /// Bounded cleanup: a probe that blocks must fail the control, not hang it.
    fn drop(&mut self) {
        let Ok(mut child) = Command::new(env!("CARGO_BIN_EXE_rrx"))
            .arg("--state")
            .arg(&self.state)
            .args(["daemon", "stop"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        else {
            return;
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        while matches!(child.try_wait(), Ok(None)) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        let _ = child.kill();
        let _ = child.wait();
    }
}

/// C-S1c: one case per non-`valid` cell. Each leaves the epoch row and the
/// descriptor bytes unchanged; a discovery failure is never `not running`.
#[test]
fn c_s1c_status_table_cells_and_no_effect() {
    let f = Fixture::new();
    // absent + free
    let (output, status) = f.daemon("status");
    assert_eq!(status["status"], "not_running", "{status}");
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(
        std::fs::read_dir(&f.base).unwrap().count(),
        1,
        "status wrote"
    );
    // absent + busy: the test holds only the owner lock.
    {
        let _lock = f.hold_owner_lock();
        let (output, status) = f.daemon("status");
        assert_eq!(status["status"], "owner_busy", "{status}");
        assert_eq!(output.status.code(), Some(4));
    }
    f.start_running();
    f.stop_and_wait();
    let epoch = f.epoch_row();
    assert!(epoch.is_some());
    // stale descriptor + free
    let stale = f.stale(2, &f.state);
    f.write_descriptor(&stale);
    let (output, status) = f.daemon("status");
    assert_eq!(status["status"], "discovery_unavailable", "{status}");
    assert_eq!(output.status.code(), Some(4));
    assert_eq!(std::fs::read(f.descriptor()).unwrap(), stale);
    // mismatch + busy
    let foreign = f.stale(2, &f.base.join("foreign.db"));
    f.write_descriptor(&foreign);
    {
        let _lock = f.hold_owner_lock();
        assert_eq!(f.status()["status"], "discovery_unavailable");
    }
    assert_eq!(std::fs::read(f.descriptor()).unwrap(), foreign);
    assert_eq!(f.epoch_row(), epoch, "status changed the epoch");
}

/// Sol 6062003954 M1: a special `owner.lock` (a FIFO without a writer) is a
/// typed L=error for status, start and stop, within an outer deadline, and
/// nothing changes: no epoch, no descriptor, no spawned owner.
#[test]
fn owner_lock_fifo_is_a_typed_probe_error_for_every_command() {
    let f = Fixture::new();
    f.start_running();
    f.stop_and_wait();
    let epoch = f.epoch_row();
    let lock = f.root().join("owner.lock");
    std::fs::remove_file(&lock).unwrap();
    assert!(
        Command::new("/usr/bin/mkfifo")
            .args(["-m", "600"])
            .arg(&lock)
            .status()
            .unwrap()
            .success()
    );
    for verb in ["status", "start", "stop"] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_rrx"))
            .arg("--state")
            .arg(&f.state)
            .arg("--config")
            .arg(&f.config)
            .args(["daemon", verb, "--json"])
            .current_dir(&f.base)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while child.try_wait().unwrap().is_none() {
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("daemon {verb} blocked on a FIFO owner lock");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let output = child.wait_with_output().unwrap();
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        let typed = match verb {
            "status" => value["status"].clone(),
            "start" => value["start"].clone(),
            _ => value["status"]["status"].clone(),
        };
        assert_eq!(typed, "discovery_unavailable", "daemon {verb}: {value}");
        assert!(!output.status.success(), "daemon {verb} succeeded");
    }
    assert_eq!(f.epoch_row(), epoch, "an epoch was begun");
    assert!(!f.descriptor().exists(), "a descriptor was published");
    assert!(
        std::fs::symlink_metadata(&lock)
            .unwrap()
            .file_type()
            .is_fifo(),
        "the FIFO was replaced"
    );
}

/// C-S1c2: status and start concurrently. A failed start is typed
/// `owner busy`, writes nothing, and the next start succeeds.
#[test]
fn c_s1c2_concurrent_status_never_corrupts_start() {
    let f = Fixture::new();
    for _ in 0..10 {
        let before = f.epoch_row();
        let status = {
            let mut command = Command::new(env!("CARGO_BIN_EXE_rrx"));
            command
                .arg("--state")
                .arg(&f.state)
                .args(["daemon", "status", "--json"])
                .stdout(std::process::Stdio::null());
            command.spawn().unwrap()
        };
        let started = f.start();
        let mut status = status;
        status.wait().unwrap();
        if started["start"] != "running" {
            assert_eq!(started["start"], "owner_busy", "{started}");
            assert_eq!(f.epoch_row(), before, "a refused start began an epoch");
            assert!(!f.descriptor().exists(), "a refused start published");
            f.start_running();
        }
        f.stop_and_wait();
    }
}

/// C-S1e2 through `daemon start`: a same-state protocol-1 descriptor and a
/// crashed protocol-2 leftover are replaced by a protocol-2 service; foreign
/// and malformed descriptors are refused, typed, and nothing is published.
#[test]
fn c_s1e2_daemon_start_replaces_known_leftovers_only() {
    let f = Fixture::new();
    f.start_running();
    f.stop_and_wait();
    // A valid protocol-1 descriptor for the same state, lock free.
    f.write_descriptor(&f.stale(1, &f.state));
    let identity = f.start_running();
    assert_eq!(identity["protocol"], 2);
    let published: Value = serde_json::from_slice(&std::fs::read(f.descriptor()).unwrap()).unwrap();
    assert_eq!(published["protocol"], 2);
    assert_eq!(published["identity"]["epoch"], identity["epoch"]);
    // A crash leaves the protocol-2 descriptor behind.
    let pid = f.service_pid();
    assert!(
        Command::new("kill")
            .args(["-9", &pid.to_string()])
            .status()
            .unwrap()
            .success()
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    while f.status()["status"] != "discovery_unavailable" {
        assert!(Instant::now() < deadline, "SETUP: crash not observed");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        f.descriptor().exists(),
        "SETUP: crash removed the descriptor"
    );
    let replaced = f.start_running();
    assert_eq!(
        replaced["epoch"].as_u64().unwrap(),
        identity["epoch"].as_u64().unwrap() + 1
    );
    f.stop_and_wait();
    // Foreign state and malformed descriptors are refused.
    for bytes in [
        f.stale(2, &f.base.join("foreign.db")),
        br#"{"protocol":2}"#.to_vec(),
    ] {
        f.write_descriptor(&bytes);
        let started = f.start();
        assert_eq!(started["start"], "start_failed", "{started}");
        assert_eq!(std::fs::read(f.descriptor()).unwrap(), bytes);
        let deadline = Instant::now() + Duration::from_secs(30);
        while f.status()["status"] != "discovery_unavailable"
            || rrx_lock_busy(&f.root().join("owner.lock"))
        {
            assert!(Instant::now() < deadline, "refused start kept an owner");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
fn rrx_lock_busy(path: &Path) -> bool {
    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    rustix::fs::flock(&file, rustix::fs::FlockOperation::NonBlockingLockShared).is_err()
}

/// C-S1g: the daemon owns no AF_INET/AF_INET6 socket; its control listener
/// is AF_UNIX. Sockets are matched to the namespace tables by inode.
#[test]
fn c_s1g_daemon_owns_no_inet_socket() {
    let f = Fixture::new();
    f.start_running();
    let pid = f.service_pid();
    if cfg!(target_os = "linux") {
        let mut inodes = Vec::new();
        for entry in std::fs::read_dir(format!("/proc/{pid}/fd")).unwrap() {
            let Ok(target) = std::fs::read_link(entry.unwrap().path()) else {
                continue;
            };
            let target = target.to_string_lossy().into_owned();
            if let Some(inode) = target
                .strip_prefix("socket:[")
                .and_then(|t| t.strip_suffix(']'))
            {
                inodes.push(inode.to_string());
            }
        }
        assert!(!inodes.is_empty(), "SETUP: no socket observed");
        let table = |name: &str, column: usize| -> Vec<String> {
            std::fs::read_to_string(format!("/proc/{pid}/net/{name}"))
                .unwrap_or_default()
                .lines()
                .skip(1)
                .filter_map(|l| l.split_whitespace().nth(column).map(str::to_string))
                .collect()
        };
        for name in ["tcp", "tcp6", "udp", "udp6"] {
            let inet = table(name, 9);
            assert!(
                inodes.iter().all(|i| !inet.contains(i)),
                "daemon owns an {name} socket"
            );
        }
        let unix = table("unix", 6);
        assert!(
            inodes.iter().any(|i| unix.contains(i)),
            "control listener is not AF_UNIX"
        );
    } else {
        let inet = Command::new("lsof")
            .args(["-n", "-P", "-a", "-p", &pid.to_string(), "-i"])
            .output()
            .unwrap();
        assert!(
            String::from_utf8_lossy(&inet.stdout).trim().is_empty(),
            "daemon owns an inet socket: {}",
            String::from_utf8_lossy(&inet.stdout)
        );
        let unix = Command::new("lsof")
            .args(["-n", "-P", "-a", "-p", &pid.to_string(), "-U"])
            .output()
            .unwrap();
        assert!(
            !String::from_utf8_lossy(&unix.stdout).trim().is_empty(),
            "control listener is not AF_UNIX"
        );
    }
    f.stop_and_wait();
}
