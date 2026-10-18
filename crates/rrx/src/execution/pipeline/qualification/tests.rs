use super::*;
use crate::config::PipelineClis;
#[path = "../../../../tests/support/pipeline_cli.rs"]
mod harness;
use harness::FakeCli;

struct Fixture {
    _dir: tempfile::TempDir,
    codex: FakeCli,
    claude: FakeCli,
    config: PipelineConfig,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let codex = FakeCli::new(dir.path(), true);
        let claude = FakeCli::new(dir.path(), false);
        let config = PipelineConfig {
            work_root: dir.path().into(),
            path_prefix: vec![],
            clis: PipelineClis {
                codex: codex.cli.clone(),
                claude: claude.cli.clone(),
            },
        };
        Self {
            _dir: dir,
            codex,
            claude,
            config,
        }
    }
    async fn q(&self, codex: bool) -> PipelineQualification {
        let (name, fake) = if codex {
            ("codex", &self.codex)
        } else {
            ("claude", &self.claude)
        };
        qualify_cli(&self.config, name, &fake.cli, self._dir.path()).await
    }
}

/// Exact pins, required flags and authenticated fake executables qualify.
#[tokio::test]
async fn b3_q_accepts_pinned_clis() {
    let f = Fixture::new();
    for codex in [true, false] {
        let row = f.q(codex).await;
        assert_eq!(row.state, "qualified", "{row:?}");
        assert_eq!(row.executable_sha256.as_ref().unwrap().len(), 64);
    }
}
/// A version differing from the complete pin is refused.
#[tokio::test]
async fn b3_q_wrong_version() {
    let f = Fixture::new();
    for fake in [&f.codex, &f.claude] {
        fake.control("version", "wrong-version\n");
    }
    for codex in [true, false] {
        let row = f.q(codex).await;
        assert_eq!(row.state, "refused");
        assert_eq!(row.body["stage"], "version_check", "{row:?}");
    }
}
/// Every template flag is required as an exact option token.
#[tokio::test]
async fn b3_q_missing_flag() {
    let f = Fixture::new();
    for (codex, fake, flags) in [
        (true, &f.codex, CODEX_FLAGS),
        (false, &f.claude, CLAUDE_FLAGS),
    ] {
        let original = std::fs::read_to_string(fake.directory.join("help")).unwrap();
        for flag in flags {
            fake.control("help", &original.replace(flag, &format!("{flag}-impostor")));
            let row = f.q(codex).await;
            assert_eq!(row.state, "refused", "{flag}");
            assert_eq!(row.body["stage"], "help_check", "{row:?}");
        }
    }
}
/// Logged-out status cannot qualify even when version and help match.
#[tokio::test]
async fn b3_q_logged_out() {
    let f = Fixture::new();
    for fake in [&f.codex, &f.claude] {
        fake.control("login", "1\n");
    }
    for codex in [true, false] {
        let row = f.q(codex).await;
        assert_eq!(row.state, "refused");
        assert_eq!(row.body["stage"], "login_check", "{row:?}");
    }
}

/// A reviewer that rejects empty setting sources cannot qualify from help presence alone.
#[tokio::test]
async fn b3_q_reviewer_empty_settings() {
    let f = Fixture::new();
    f.claude.control("settings-status", "1\n");
    let row = f.q(false).await;
    assert_eq!(row.state, "refused");
    assert_eq!(row.body["stage"], "settings_check", "{row:?}");
}
/// A CLI image changed during its probes is refused.
#[tokio::test]
async fn b3_q_changed_sha() {
    let f = Fixture::new();
    for fake in [&f.codex, &f.claude] {
        fake.control("mutate", "");
    }
    for codex in [true, false] {
        let row = f.q(codex).await;
        assert_eq!(row.state, "refused");
        assert_eq!(row.body["stage"], "image_recheck", "{row:?}");
    }
}
/// An executable changed after qualification fails the launch-time recheck.
#[tokio::test]
async fn b3_q_recheck_changed_sha() {
    let f = Fixture::new();
    let row = f.q(true).await;
    let qualified = QualifiedCli {
        realpath: serde_json::from_value(row.body["realpath"].clone()).unwrap(),
        sha256: row.executable_sha256.unwrap(),
    };
    qualified.recheck(&f.codex.cli).unwrap();
    std::fs::write(&f.codex.cli.executable, "#!/bin/sh\nexit 0\n").unwrap();
    assert!(qualified.recheck(&f.codex.cli).is_err());
}
/// Probe environments are subsets of the approved environment variables.
#[tokio::test]
async fn b3_q_environment() {
    let f = Fixture::new();
    for (codex, fake) in [(true, &f.codex), (false, &f.claude)] {
        assert_eq!(f.q(codex).await.state, "qualified");
        let environment = std::fs::read_to_string(fake.directory.join("environment")).unwrap();
        let allowed = [
            "HOME",
            "USER",
            "LOGNAME",
            "PATH",
            "LANG",
            "TMPDIR",
            "TERM",
            "NO_COLOR",
            "RRX_PROCESS_COOKIE",
            "CODEX_HOME",
            "PWD",
            "SHLVL",
            "_",
        ];
        // Shells add PWD, SHLVL and _ themselves; verify the actual Command input separately.
        assert!(
            environment
                .lines()
                .all(|line| allowed.contains(&line.split('=').next().unwrap()))
        );
        let env = super::environment(
            &f.config,
            f._dir.path(),
            "0123456789abcdef",
            &fake.cli,
            codex,
        )
        .unwrap();
        assert!(
            env.iter()
                .all(|(key, _)| allowed[..10].contains(&key.as_str()))
        );
        assert_eq!(env.iter().any(|(key, _)| key == "CODEX_HOME"), codex);
    }
}
/// Unapproved Codex configuration cannot activate hooks, MCP or profiles.
#[tokio::test]
async fn b3_q_codex_config() {
    let f = Fixture::new();
    let path = f.codex.cli.codex_home.as_ref().unwrap().join("config.toml");
    for config in [
        "notify=['x']",
        "[mcp_servers.bad]\ncommand='x'",
        "[hooks]",
        "[profiles]",
        "unknown=true",
        "unknown='value'",
    ] {
        std::fs::write(&path, config).unwrap();
        let row = f.q(true).await;
        assert_eq!(row.state, "refused");
        assert_eq!(row.body["stage"], "config");
    }
}
/// Probe output over four KiB cannot produce a qualification.
#[tokio::test]
async fn b3_q_output_bound() {
    let f = Fixture::new();
    f.codex.control("help", &"x".repeat(4097));
    let row = f.q(true).await;
    assert_eq!(row.state, "refused");
    assert_eq!(row.body["stage"], "help_probe");
}
/// The reusable fake exec interface preserves captured output and exit status.
#[tokio::test]
async fn b3_fake_exec_interface() {
    let f = Fixture::new();
    for fake in [&f.codex, &f.claude] {
        fake.control("exec-status", "7\n");
        fake.control("exec-output", "exec fixture\n");
        let mut command = tokio::process::Command::new(&fake.cli.executable);
        command.arg("exec").env_clear();
        let (receipt, out, _) = capture_probe(command).await.unwrap();
        assert_eq!(receipt.status.code(), Some(7));
        assert_eq!(out, b"exec fixture\n");
    }
}

/// Failed startup probes are retained as refused observations in the actual owner epoch.
#[tokio::test]
async fn b3_q_records_refusals() {
    let f = Fixture::new();
    let owner = RuntimeOwner::open_mode(
        &f._dir.path().join("state.db"),
        crate::config::ExecutionMode::CliPipeline,
    )
    .unwrap();
    for fake in [&f.codex, &f.claude] {
        fake.control("version", "wrong\n");
    }
    assert!(
        CliQualifier::record_clis(&owner, &f.config, f._dir.path())
            .await
            .is_err()
    );
    let rows = owner
        .store
        .lock()
        .unwrap()
        .pipeline_qualifications(&owner)
        .unwrap();
    for subject in ["codex", "claude"] {
        assert_eq!(
            rows.iter().find(|r| r.subject == subject).unwrap().state,
            "refused"
        );
    }
    assert!(CliQualifier::current(&owner, &f.config).is_err());
}
/// Startup qualification probes hold neither the Store mutex nor a SQLite writer transaction.
#[tokio::test]
async fn b3_q_outside_store_locks() {
    let f = Fixture::new();
    let owner = RuntimeOwner::open_mode(
        &f._dir.path().join("state.db"),
        crate::config::ExecutionMode::CliPipeline,
    )
    .unwrap();
    f.codex.control("pause", "");
    let qualifier = CliQualifier::record_clis(&owner, &f.config, f._dir.path());
    let observer = async {
        tokio::time::timeout(std::time::Duration::from_secs(4), async {
            while !f.codex.directory.join("entered").exists() {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        let unlocked = owner.store.try_lock().is_ok();
        let connection = rusqlite::Connection::open(owner.state_path()).unwrap();
        let transaction_available = connection.execute_batch("BEGIN IMMEDIATE;ROLLBACK").is_ok();
        f.codex.control("release", "");
        assert!(unlocked && transaction_available);
    };
    let (_, ()) = tokio::join!(qualifier, observer);
}
/// A new epoch cannot reuse previous CLI observations as current Q.
#[tokio::test]
async fn b3_q_requires_current_epoch() {
    let f = Fixture::new();
    let path = f._dir.path().join("state.db");
    let owner = RuntimeOwner::open_mode(&path, crate::config::ExecutionMode::CliPipeline).unwrap();
    for fake in [&f.codex, &f.claude] {
        fake.control("version", "wrong\n");
    }
    assert!(
        CliQualifier::record_clis(&owner, &f.config, f._dir.path())
            .await
            .is_err()
    );
    drop(owner);
    let owner = RuntimeOwner::open_mode(&path, crate::config::ExecutionMode::CliPipeline).unwrap();
    assert_eq!(
        owner
            .store
            .lock()
            .unwrap()
            .pipeline_qualifications(&owner)
            .unwrap()
            .len(),
        1
    );
    assert!(CliQualifier::current(&owner, &f.config).is_err());
}
