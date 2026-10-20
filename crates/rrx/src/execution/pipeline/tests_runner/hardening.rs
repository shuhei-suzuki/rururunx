use super::*;
use std::process::Stdio;

const UNRESTRICTED: &str = "(version 1)\n(allow default)\n";
struct Sandbox {
    _dir: tempfile::TempDir,
    root: PathBuf,
    checkout: PathBuf,
    tmp: PathBuf,
    policy: String,
}
impl Sandbox {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let work = root.join("work");
        let checkout = work.join("co");
        let tmp = work.join("tmp");
        for p in [&checkout, &tmp] {
            std::fs::create_dir_all(p).unwrap();
        }
        let policy = seatbelt(&checkout, &tmp, &[], &work, &[]).unwrap();
        Self {
            _dir: dir,
            root,
            checkout,
            tmp,
            policy,
        }
    }
    fn status(&self, policy: &str, args: &[&str]) -> bool {
        let path = self.root.join("profile.sb");
        std::fs::write(&path, policy).unwrap();
        let mut command = std::process::Command::new("/usr/bin/sandbox-exec");
        command
            .arg("-f")
            .arg(path)
            .arg("--")
            .args(args)
            .current_dir(&self.checkout)
            .env_clear()
            .env("HOME", std::env::var_os("HOME").unwrap())
            .env(
                "PATH",
                "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin",
            )
            .env("TMPDIR", &self.tmp)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        rrx_spawn_hygiene::status_std(command).unwrap().success()
    }
    fn denied(&self, args: &[&str]) {
        assert!(
            !self.status(&self.policy, args),
            "unexpected success: {args:?}"
        );
    }
}
fn python() -> PathBuf {
    ["/opt/homebrew/bin/python3", "/usr/local/bin/python3"]
        .iter()
        .map(PathBuf::from)
        .find(|p| p.is_file())
        .expect("Homebrew python3 required")
        .canonicalize()
        .unwrap()
}

/// Ordinary toolchains, subprocesses and null redirection work within the grants.
#[test]
fn b6_f1_ordinary_commands() {
    let s = Sandbox::new();
    for args in [
        vec!["/usr/bin/true"],
        vec![
            "/bin/sh",
            "-c",
            "echo hi > out.txt && cat out.txt >/dev/null",
        ],
        vec!["/usr/bin/perl", "-e", "open F,\">/dev/null\" or exit 1"],
        vec![
            "/usr/bin/perl",
            "-e",
            "open F,\">$ENV{TMPDIR}/a\" or exit 1",
        ],
        vec!["/usr/bin/git", "--version"],
        vec!["/usr/bin/whoami"],
        vec!["/bin/date"],
    ] {
        assert!(s.status(&s.policy, &args), "{args:?}");
    }
    assert!(s.checkout.join("out.txt").is_file());
    assert!(s.tmp.join("a").is_file());
    let python = python();
    assert!(s.status(
        &s.policy,
        &[
            python.to_str().unwrap(),
            "-I",
            "-c",
            "import os,tempfile; f,p=tempfile.mkstemp(); os.close(f); os.unlink(p)"
        ]
    ));
    let cargo = PathBuf::from(std::env::var_os("HOME").unwrap()).join(".cargo/bin/cargo");
    assert!(s.status(&s.policy, &[cargo.to_str().unwrap(), "--version"]));
}

/// Pasteboard reads and writes cannot cross the Mach service boundary.
#[test]
fn b6_f1_pasteboard() {
    let s = Sandbox::new();
    let mut capture = std::process::Command::new("/usr/bin/pbpaste");
    capture.args(["-p", "find"]).stderr(Stdio::null());
    let saved = rrx_spawn_hygiene::output_std(capture).unwrap();
    assert!(saved.status.success());
    struct Restore(Vec<u8>);
    impl Drop for Restore {
        fn drop(&mut self) {
            use std::io::Write;
            let mut command = std::process::Command::new("/usr/bin/pbcopy");
            command
                .args(["-p", "find"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            let mut child = rrx_spawn_hygiene::spawn_std(command).unwrap();
            child.stdin.take().unwrap().write_all(&self.0).unwrap();
            assert!(child.wait().unwrap().success());
        }
    }
    let _restore = Restore(saved.stdout);
    assert!(s.status(UNRESTRICTED, &["/usr/bin/pbpaste", "-p", "find"]));
    s.denied(&["/usr/bin/pbpaste", "-p", "find"]);
    s.denied(&["/usr/bin/pbcopy", "-p", "find"]);
}

/// A readable fixture keychain item is inaccessible through securityd in the sandbox.
#[test]
fn b6_f1_keychain() {
    let s = Sandbox::new();
    let keychain = s.root.join("fixture.keychain-db");
    let path = keychain.to_str().unwrap();
    struct Delete(PathBuf);
    impl Drop for Delete {
        fn drop(&mut self) {
            let mut cmd = std::process::Command::new("/usr/bin/security");
            cmd.args(["delete-keychain"])
                .arg(&self.0)
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            assert!(rrx_spawn_hygiene::status_std(cmd).unwrap().success());
        }
    }
    assert!(s.status(
        UNRESTRICTED,
        &[
            "/usr/bin/security",
            "create-keychain",
            "-p",
            "fixture-only",
            path
        ]
    ));
    let _delete = Delete(keychain.clone());
    assert!(s.status(
        UNRESTRICTED,
        &[
            "/usr/bin/security",
            "unlock-keychain",
            "-p",
            "fixture-only",
            path
        ]
    ));
    assert!(s.status(
        UNRESTRICTED,
        &[
            "/usr/bin/security",
            "add-generic-password",
            "-a",
            "fixture",
            "-s",
            "rrx-fixture",
            "-w",
            "fixture-value",
            path
        ]
    ));
    let args = [
        "/usr/bin/security",
        "find-generic-password",
        "-a",
        "fixture",
        "-s",
        "rrx-fixture",
        "-w",
        path,
    ];
    assert!(s.status(UNRESTRICTED, &args));
    s.denied(&args);
}

/// Host application, launchd and preference services remain unavailable to tests.
#[test]
fn b6_f1_host_services() {
    let s = Sandbox::new();
    s.denied(&["/usr/bin/open", "-g", s.checkout.to_str().unwrap()]);
    let label = format!("rrx-fixture-{}", uuid::Uuid::new_v4());
    struct RemoveJob(String);
    impl Drop for RemoveJob {
        fn drop(&mut self) {
            let mut command = std::process::Command::new("/bin/launchctl");
            command
                .args(["remove", &self.0])
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            let _ = rrx_spawn_hygiene::status_std(command).unwrap();
        }
    }
    let _remove = RemoveJob(label.clone());
    s.denied(&[
        "/bin/launchctl",
        "submit",
        "-l",
        &label,
        "--",
        "/usr/bin/true",
    ]);
    s.denied(&["/bin/launchctl", "list"]);
    let domain = format!("org.rrx.fixture.{}", uuid::Uuid::new_v4());
    struct DeletePreference(String);
    impl Drop for DeletePreference {
        fn drop(&mut self) {
            let mut command = std::process::Command::new("/usr/bin/defaults");
            command
                .args(["delete", &self.0])
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            let _ = rrx_spawn_hygiene::status_std(command).unwrap();
        }
    }
    let _delete = DeletePreference(domain.clone());
    s.denied(&[
        "/usr/bin/defaults",
        "write",
        &domain,
        "fixture",
        "-string",
        "value",
    ]);
}

/// Both control-style UNIX sockets and localhost server sockets remain denied.
#[test]
fn b6_f1_local_sockets() {
    let s = Sandbox::new();
    let owner = RuntimeOwner::open(&s.root.join("state/state.db")).unwrap();
    let socket = owner.ipc_path().with_file_name("control.sock");
    let _listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    let args = [
        "/usr/bin/perl",
        "-e",
        "use Socket; socket(my $s, AF_UNIX, SOCK_STREAM, 0) or exit 1; connect($s, pack_sockaddr_un($ARGV[0])) or exit 2",
        socket.to_str().unwrap(),
    ];
    assert!(s.status(UNRESTRICTED, &args));
    s.denied(&args);
    let code = "use Socket; socket(my $s, AF_INET, SOCK_STREAM, 0) or exit 1; bind($s, sockaddr_in(0, inet_aton('127.0.0.1'))) or exit 2; listen($s,1) or exit 3";
    assert!(s.status(UNRESTRICTED, &["/usr/bin/perl", "-e", code]));
    s.denied(&["/usr/bin/perl", "-e", code]);
}

/// A test cannot create a hardlink to an outside file inside its checkout.
#[test]
fn b6_f1_hardlink() {
    let s = Sandbox::new();
    let outside = s.root.join("outside");
    std::fs::write(&outside, b"protected").unwrap();
    let link = s.checkout.join("link");
    for args in [
        vec!["/bin/ln", outside.to_str().unwrap(), link.to_str().unwrap()],
        vec![
            "/usr/bin/perl",
            "-e",
            "link($ARGV[0], $ARGV[1]) or exit 1",
            outside.to_str().unwrap(),
            link.to_str().unwrap(),
        ],
    ] {
        assert!(s.status(UNRESTRICTED, &args));
        std::fs::remove_file(&link).unwrap();
        s.denied(&args);
        assert!(!link.exists());
    }
    assert_eq!(std::fs::read(outside).unwrap(), b"protected");
}
