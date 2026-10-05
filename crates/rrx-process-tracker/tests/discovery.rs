//! Owned same-user child controls; no foreign process is signaled.
use rrx_process_tracker::{Cookie, Limits, Termination, discover};
use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn child(root: &std::path::Path, name: &str, cookie: &str) -> OwnedChild {
    OwnedChild(
        Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", "owned_cookie_worker", "--nocapture"])
            .env("RRX_PROCESS_COOKIE", cookie)
            .env("RRX_TRACKER_TEST_READY", root.join(name))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    )
}
fn ready(root: &std::path::Path, name: &str) {
    let at = Instant::now();
    while !root.join(name).exists() {
        assert!(
            at.elapsed() < Duration::from_secs(10),
            "owned child did not initialize"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
#[ignore = "child-only fixture entry"]
fn owned_cookie_worker() {
    let Some(marker) = std::env::var_os("RRX_TRACKER_TEST_READY") else {
        return;
    };
    std::fs::write(marker, "ready").unwrap();
    std::thread::sleep(Duration::from_secs(30));
}
#[test]
fn exact_cookie_discovers_only_the_owned_match_and_never_signals_its_sibling() {
    let root = tempfile::tempdir().unwrap();
    let cookie_value = format!(
        "tracker-fixture-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let sibling_cookie = format!("{cookie_value}-other");
    let mut matched = child(root.path(), "match", &cookie_value);
    let mut sibling = child(root.path(), "sibling", &sibling_cookie);
    ready(root.path(), "match");
    ready(root.path(), "sibling");
    let selection = discover(
        &Cookie::new(&cookie_value).unwrap(),
        Limits {
            time: Duration::from_secs(10),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        selection.processes.len(),
        1,
        "coverage: {:?}",
        selection.coverage
    );
    let process = &selection.processes[0];
    assert_eq!(process.identity().pid, matched.0.id());
    assert_ne!(process.identity().pid, sibling.0.id());
    #[cfg(target_os = "macos")]
    {
        assert!(!process.atomic_signal_available());
        assert_eq!(process.terminate(), Termination::Unsupported);
        assert_eq!(process.exited().unwrap(), None);
        assert!(matched.0.try_wait().unwrap().is_none());
    }
    #[cfg(target_os = "linux")]
    {
        assert!(
            process.atomic_signal_available(),
            "runner requires pidfd for this conformance control"
        );
        assert_eq!(process.exited().unwrap(), Some(false));
        assert_eq!(process.terminate(), Termination::Sent);
        let at = Instant::now();
        // Observe kernel exit readiness BEFORE the parent reaps the zombie.
        while process.exited().unwrap() != Some(true) {
            assert!(at.elapsed() < Duration::from_secs(10));
            std::thread::sleep(Duration::from_millis(10));
        }
        let stat = std::fs::read_to_string(format!("/proc/{}/stat", matched.0.id())).unwrap();
        assert_eq!(
            stat.rsplit_once(')').unwrap().1.split_whitespace().next(),
            Some("Z")
        );
        matched.0.wait().unwrap();
        assert_eq!(process.exited().unwrap(), Some(true));
        assert_eq!(process.terminate(), Termination::AlreadyExited);
    }
    assert!(sibling.0.try_wait().unwrap().is_none());
}
