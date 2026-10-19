use super::*;
use crate::execution::verification::{Applicability, Category, TestsProfile};
use std::{collections::BTreeMap, os::unix::fs::PermissionsExt};

pub(crate) fn proposal(program: PathBuf) -> TestsProfile {
    TestsProfile {
        commands: vec![TestCommand {
            id: "test".into(),
            category: Category::Tests,
            program,
            args: vec![],
            cwd: ".".into(),
            timeout_seconds: 2,
            drain_seconds: 1,
            stdout_bytes: 1024,
            stderr_bytes: 1024,
        }],
        applicability: BTreeMap::from([
            (Category::Tests, Applicability::Required),
            (
                Category::Lint,
                Applicability::NotApplicable {
                    reason: "unused".into(),
                },
            ),
            (
                Category::Build,
                Applicability::NotApplicable {
                    reason: "unused".into(),
                },
            ),
            (
                Category::Typecheck,
                Applicability::NotApplicable {
                    reason: "unused".into(),
                },
            ),
        ]),
    }
}

/// Rechecking rejects executable bytes changed after explicit admission.
#[test]
fn b6_recheck_program() {
    let dir = tempfile::tempdir().unwrap();
    let program = dir.path().join("test-program");
    std::fs::write(&program, b"first").unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    let profile = AdmittedProfile::admit(proposal(program)).unwrap();
    let command = &profile.proposal.commands[0];
    profile.recheck(command).unwrap();
    std::fs::write(&command.program, b"other").unwrap();
    assert!(profile.recheck(command).is_err());
}

/// Profile validation rejects limits beyond the qualified envelope.
#[test]
fn b6_profile_limits() {
    let valid = proposal("/usr/bin/true".into());
    valid.validate().unwrap();
    for (field, value) in [
        ("timeout_seconds", 7201),
        ("drain_seconds", 11),
        ("stdout_bytes", 33554433),
        ("stderr_bytes", 0),
    ] {
        let mut body = serde_json::to_value(&valid).unwrap();
        body["commands"][0][field] = serde_json::json!(value);
        assert!(
            serde_json::from_value::<TestsProfile>(body)
                .unwrap()
                .validate()
                .is_err(),
            "{field}"
        );
    }
}

/// A test command must match both its bounded profile index and admitted identity.
#[test]
fn b6_command_selection() {
    let profile = AdmittedProfile::admit(proposal("/usr/bin/true".into())).unwrap();
    let mut step: PipelineStep = serde_json::from_value(serde_json::json!({"run_id":"fixture","seq":1,"attempt":1,"kind":"test","provider":null,"owner_epoch":1,"round":0,"state":"planned","version":1,"cookie":null,"pid":null,"birth":null,"step_base":"a".repeat(40),"input_sha256":"b".repeat(64),"result_commit":null,"result_tree":null,"body":{"command_id":"test","command_index":0}})).unwrap();
    selected(&profile, &step).unwrap();
    for index in [
        serde_json::Value::Null,
        serde_json::json!(32),
        serde_json::json!(-1),
    ] {
        step.body["command_index"] = index;
        assert!(selected(&profile, &step).is_err());
    }
    step.body["command_index"] = serde_json::json!(0);
    step.body["command_id"] = serde_json::json!("other");
    assert!(selected(&profile, &step).is_err());
}

/// Seatbelt literals reject profile syntax and noncanonical aliases.
#[test]
fn b6_seatbelt_paths() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    assert!(absolute_literal(&root).is_ok());
    for name in ["quote\"", "paren)", "line\n", "slash\\", "space name"] {
        let path = root.join(name);
        std::fs::create_dir(&path).unwrap();
        assert!(absolute_literal(&path).is_err());
    }
    let alias = root.join("alias");
    std::os::unix::fs::symlink(&root, &alias).unwrap();
    assert!(absolute_literal(&alias).is_err());
    assert!(absolute_literal(Path::new("/")).is_err());
}

/// Cache grants cannot include managed directories or the operator home itself.
#[test]
fn b6_seatbelt_protected() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let work = root.join("work");
    let checkout = work.join("co");
    let tmp = work.join("tmp");
    let project = root.join("project");
    let cache = root.join("cache");
    for path in [&checkout, &tmp, &project, &cache] {
        std::fs::create_dir_all(path).unwrap();
    }
    assert!(
        seatbelt(
            &checkout,
            &tmp,
            std::slice::from_ref(&cache),
            &work,
            std::slice::from_ref(&project)
        )
        .is_ok()
    );
    for path in [&project, &root, &work, &checkout] {
        assert!(
            seatbelt(
                &checkout,
                &tmp,
                std::slice::from_ref(path),
                &work,
                std::slice::from_ref(&project)
            )
            .is_err()
        );
    }
    let home = PathBuf::from(std::env::var_os("HOME").unwrap())
        .canonicalize()
        .unwrap();
    assert!(seatbelt(&checkout, &tmp, &[home], &work, &[]).is_err());
}

#[cfg(target_os = "macos")]
mod host {
    use super::*;
    fn run(policy: &str, args: &[&str], root: &Path) -> std::process::ExitStatus {
        let path = root.join("test.sb");
        std::fs::write(&path, policy).unwrap();
        rrx_spawn_hygiene::status_std({
            let mut command = std::process::Command::new("/usr/bin/sandbox-exec");
            command
                .arg("-f")
                .arg(path)
                .arg("--")
                .args(args)
                .env_clear()
                .env("PATH", "/usr/bin:/bin");
            command
        })
        .unwrap()
    }
    fn dirs() -> (
        tempfile::TempDir,
        PathBuf,
        PathBuf,
        PathBuf,
        PathBuf,
        String,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let work = root.join("work");
        let checkout = work.join("co");
        let tmp = work.join("tmp");
        let cache = root.join("cache");
        for path in [&checkout, &tmp, &cache] {
            std::fs::create_dir_all(path).unwrap();
        }
        let policy = seatbelt(&checkout, &tmp, std::slice::from_ref(&cache), &work, &[]).unwrap();
        (dir, root, checkout, tmp, cache, policy)
    }

    /// Seatbelt prevents opening even a local listening network endpoint.
    #[test]
    fn b6_seatbelt_network() {
        let (_dir, root, _, _, _, policy) = dirs();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port().to_string();
        assert!(
            run(
                "(version 1)\n(allow default)\n",
                &["/usr/bin/nc", "-z", "-w", "1", "127.0.0.1", &port],
                &root
            )
            .success()
        );
        assert!(
            !run(
                &policy,
                &["/usr/bin/nc", "-z", "-w", "1", "127.0.0.1", &port],
                &root
            )
            .success()
        );
    }

    /// Seatbelt permits writes only within checkout, TMPDIR and declared caches.
    #[test]
    fn b6_seatbelt_writes() {
        let (_dir, root, checkout, tmp, cache, policy) = dirs();
        for dir in [checkout, tmp, cache] {
            let target = dir.join("allowed");
            assert!(
                run(
                    &policy,
                    &["/usr/bin/touch", target.to_str().unwrap()],
                    &root
                )
                .success()
            );
            assert!(target.is_file());
        }
        let target = root.join("outside");
        assert!(
            !run(
                &policy,
                &["/usr/bin/touch", target.to_str().unwrap()],
                &root
            )
            .success()
        );
        assert!(!target.exists());
    }
}
