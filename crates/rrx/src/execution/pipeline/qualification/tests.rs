use super::*;
use crate::config::PipelineClis;
use crate::execution::pipeline::test_support::FakeCli;
use crate::execution::process::capture_probe;

struct Fixture {
    _dir: tempfile::TempDir,
    codex: FakeCli,
    claude: FakeCli,
    config: PipelineConfig,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(dir.path()).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        let codex = FakeCli::new(&root, true);
        let claude = FakeCli::new(&root, false);
        let config = PipelineConfig {
            work_root: root,
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
        ];
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
    use crate::execution::pipeline::{PipelineRole, step_arguments};
    for (role, fake, expected) in [
        (
            PipelineRole::Implement,
            &f.codex,
            vec![
                "exec",
                "--sandbox",
                "workspace-write",
                "-c",
                "sandbox_workspace_write.network_access=false",
                "-c",
                "projects./work.trust_level=\"untrusted\"",
                "-m",
                "fixture-model",
                "-c",
                "model_reasoning_effort=high",
                "-C",
                "/work",
                "-o",
                "/attempt/last.md",
                "--json",
                "--color",
                "never",
                "-",
            ],
        ),
        (
            PipelineRole::Fix,
            &f.codex,
            vec![
                "exec",
                "--sandbox",
                "workspace-write",
                "-c",
                "sandbox_workspace_write.network_access=false",
                "-c",
                "projects./work.trust_level=\"untrusted\"",
                "-m",
                "fixture-model",
                "-c",
                "model_reasoning_effort=high",
                "-C",
                "/work",
                "-o",
                "/attempt/last.md",
                "--json",
                "--color",
                "never",
                "-",
            ],
        ),
        (
            PipelineRole::Confirm,
            &f.codex,
            vec![
                "exec",
                "--sandbox",
                "read-only",
                "-c",
                "projects./work.trust_level=\"untrusted\"",
                "-m",
                "fixture-model",
                "-c",
                "model_reasoning_effort=high",
                "-C",
                "/work",
                "--output-schema",
                "/attempt/confirm.schema.json",
                "-o",
                "/attempt/last.json",
                "--json",
                "--color",
                "never",
                "-",
            ],
        ),
        (
            PipelineRole::Review,
            &f.claude,
            vec![
                "-p",
                "--permission-mode",
                "plan",
                "--tools",
                "Read,Grep,Glob",
                "--disallowedTools",
                "Bash,Edit,Write,NotebookEdit,WebFetch,WebSearch,Task",
                "--strict-mcp-config",
                "--mcp-config",
                "/attempt/mcp.json",
                "--setting-sources",
                "",
                "--disable-slash-commands",
                "--model",
                "fixture-model",
                "--output-format",
                "json",
                "--json-schema",
                "{}",
            ],
        ),
    ] {
        fake.expect_arguments(&expected);
        fake.control("exec-status", "7\n");
        fake.control("exec-output", "exec fixture\n");
        let mut command = tokio::process::Command::new(&fake.cli.executable);
        command
            .args(
                step_arguments(
                    role,
                    &fake.cli,
                    Path::new("/work"),
                    Path::new("/attempt"),
                    "{}",
                )
                .unwrap(),
            )
            .env_clear();
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
    let path = f._dir.path().join("state/state.db");
    let owner = RuntimeOwner::open_mode(&path, crate::config::ExecutionMode::CliPipeline).unwrap();
    let mut config = f.config.clone();
    config.work_root = std::fs::canonicalize(owner.ipc_path().parent().unwrap()).unwrap();
    std::fs::set_permissions(&config.work_root, std::fs::Permissions::from_mode(0o700)).unwrap();
    CliQualifier::qualify(&owner, &config).await.unwrap();
    CliQualifier::current(&owner, &config).unwrap();
    let rows = owner
        .store
        .lock()
        .unwrap()
        .pipeline_qualifications(&owner)
        .unwrap();
    for subject in ["platform", "mode_quiescence", "codex", "claude"] {
        assert_eq!(
            rows.iter().find(|r| r.subject == subject).unwrap().state,
            "qualified"
        );
    }
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
    assert!(CliQualifier::current(&owner, &config).is_err());
}

/// Help that bypasses value validation cannot prove empty settings are accepted.
#[tokio::test]
async fn b3_q_settings_help_short_circuit() {
    let f = Fixture::new();
    assert_eq!(f.q(false).await.state, "qualified");
    f.claude.control("help-before-validation", "");
    let mut help = std::fs::read_to_string(f.claude.directory.join("help")).unwrap();
    help.push_str("--verbose <LEVEL>  Include error messages\n");
    f.claude.control("help", &help);
    let row = f.q(false).await;
    assert_eq!(row.state, "refused");
    assert_eq!(row.body["reason"], "setting_sources_validation_unproven");
}

/// An empty value reporting a parser error cannot qualify even with a successful exit.
#[tokio::test]
async fn b3_q_empty_settings_error() {
    let f = Fixture::new();
    assert_eq!(f.q(false).await.state, "qualified");
    f.claude.control("settings-error", "");
    let row = f.q(false).await;
    assert_eq!(row.state, "refused");
    assert_eq!(row.body["stage"], "settings_check");
}

/// Interpreter scripts cannot stand in for the executable image pinned by Q.
#[test]
fn b3_q_native_image_required() {
    let f = Fixture::new();
    executable(&f.codex.cli.executable).unwrap();
    std::fs::write(&f.codex.cli.executable, "#!/bin/sh\nexit 0\n").unwrap();
    assert!(
        executable(&f.codex.cli.executable)
            .unwrap_err()
            .to_string()
            .contains("native Mach-O")
    );
}

/// FIFO executable and config inputs are refused within the qualification deadline.
#[test]
fn b3_q_special_files() {
    let f = Fixture::new();
    executable(&f.codex.cli.executable).unwrap();
    let home = f.codex.cli.codex_home.as_ref().unwrap();
    codex_config(home, Instant::now() + PROBE_BOUND).unwrap();
    let config = f.codex.cli.codex_home.as_ref().unwrap().join("config.toml");
    for path in [&config, &f.codex.cli.executable] {
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
        let mut mkfifo = std::process::Command::new("/usr/bin/mkfifo");
        mkfifo.arg(path);
        assert!(rrx_spawn_hygiene::status_std(mkfifo).unwrap().success());
        let started = std::time::Instant::now();
        let result = if path == &config {
            codex_config(home, Instant::now() + PROBE_BOUND)
        } else {
            executable(path).map(|_| ())
        };
        assert!(started.elapsed() < std::time::Duration::from_secs(6));
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not a regular file")
        );
        if path == &f.codex.cli.executable {
            assert!(executable(&f.codex.cli.executable).is_err());
            assert!(started.elapsed() < std::time::Duration::from_secs(6));
        }
        std::fs::remove_file(path).unwrap();
    }
}

/// Idle sweeps do not hash images; each launch rechecks both pins exactly once.
#[tokio::test]
async fn b3_q_launch_hash_frequency() {
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open_mode(
        &dir.path().join("state/state.db"),
        crate::config::ExecutionMode::CliPipeline,
    )
    .unwrap();
    let config = crate::execution::pipeline::test_support::config(&owner, dir.path());
    CliQualifier::qualify(&owner, &config).await.unwrap();
    let before = HASH_COUNT.with(|n| n.get());
    for _ in 0..20 {
        CliQualifier::current(&owner, &config).unwrap();
    }
    assert_eq!(HASH_COUNT.with(|n| n.get()), before);
    for launches in 1..=2 {
        CliQualifier::before_launch(&owner, &config).unwrap();
        assert_eq!(HASH_COUNT.with(|n| n.get()), before + launches * 2);
    }
    std::fs::write(&config.clis.codex.executable, "changed").unwrap();
    CliQualifier::current(&owner, &config).unwrap();
    assert!(CliQualifier::before_launch(&owner, &config).is_err());
}

/// Reviewer login uses the same empty settings condition as its step template.
#[tokio::test]
async fn b3_q_login_settings() {
    let f = Fixture::new();
    assert_eq!(f.q(false).await.state, "qualified");
    let calls = std::fs::read_to_string(f.claude.directory.join("calls")).unwrap();
    assert!(calls.lines().any(|s| s == "--setting-sources  auth status"));
    assert!(!calls.lines().any(|s| s == "auth status"));
}

/// CODEX_HOME stays separate from work, state and every registered Project.
#[test]
fn b3_q_home_separation() {
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open_mode(
        &dir.path().join("state/state.db"),
        crate::config::ExecutionMode::CliPipeline,
    )
    .unwrap();
    let mut config = crate::execution::pipeline::test_support::config(&owner, dir.path());
    CliQualifier::check_layout(&owner, &config).unwrap();
    std::fs::set_permissions(owner.state_root(), std::fs::Permissions::from_mode(0o700)).unwrap();
    for home in [&config.work_root, &owner.state_root().to_path_buf()] {
        let mut invalid = config.clone();
        invalid.clis.codex.codex_home = Some(home.clone());
        assert!(
            CliQualifier::check_layout(&owner, &invalid)
                .unwrap_err()
                .to_string()
                .contains("overlap")
        );
    }
    let source = dir.path().join("source");
    std::fs::create_dir(&source).unwrap();
    std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut project = crate::domain::Project::new(
        "source".into(),
        source.canonicalize().unwrap(),
        "fixture".into(),
        "main".into(),
    );
    owner
        .store
        .lock()
        .unwrap()
        .put_project(&mut project)
        .unwrap();
    config.clis.codex.codex_home = Some(source.canonicalize().unwrap());
    assert!(
        CliQualifier::check_layout(&owner, &config)
            .unwrap_err()
            .to_string()
            .contains("Project overlaps")
    );
}

/// Home permissions are rechecked before launch even when config and Q are unchanged.
#[tokio::test]
async fn b3_q_home_permissions() {
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open_mode(
        &dir.path().join("state/state.db"),
        crate::config::ExecutionMode::CliPipeline,
    )
    .unwrap();
    let config = crate::execution::pipeline::test_support::config(&owner, dir.path());
    CliQualifier::qualify(&owner, &config).await.unwrap();
    CliQualifier::before_launch(&owner, &config).unwrap();
    let home = config.clis.codex.codex_home.as_ref().unwrap();
    std::fs::set_permissions(home, std::fs::Permissions::from_mode(0o750)).unwrap();
    let error = CliQualifier::before_launch(&owner, &config).unwrap_err();
    assert!(format!("{error:#}").contains("mode 0700"));
}

/// PATH prefixes and their parents cannot become group or world writable before a launch.
#[tokio::test]
async fn b3_q_path_parents() {
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open_mode(
        &dir.path().join("state/state.db"),
        crate::config::ExecutionMode::CliPipeline,
    )
    .unwrap();
    let mut config = crate::execution::pipeline::test_support::config(&owner, dir.path());
    let root = std::fs::canonicalize(dir.path()).unwrap();
    let parent = root.join("prefix");
    let leaf = parent.join("bin");
    std::fs::create_dir_all(&leaf).unwrap();
    config.path_prefix = vec![leaf.clone()];
    CliQualifier::qualify(&owner, &config).await.unwrap();
    CliQualifier::before_launch(&owner, &config).unwrap();
    for path in [&leaf, &parent] {
        for mode in [0o775, 0o757] {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
            assert!(
                CliQualifier::before_launch(&owner, &config)
                    .unwrap_err()
                    .to_string()
                    .contains("untrusted pipeline PATH")
            );
        }
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

/// A PATH symlink cannot hide a writable parent behind a trusted canonical target.
#[test]
fn b3_q_path_alias() {
    let f = Fixture::new();
    let target = f.config.work_root.join("target");
    let parent = f.config.work_root.join("writable");
    std::fs::create_dir(&target).unwrap();
    std::fs::create_dir(&parent).unwrap();
    let mut config = f.config.clone();
    config.path_prefix = vec![target.clone()];
    trusted_path_prefix(&config).unwrap();
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o777)).unwrap();
    let alias = parent.join("bin");
    std::os::unix::fs::symlink(&target, &alias).unwrap();
    config.path_prefix = vec![alias];
    assert!(
        trusted_path_prefix(&config)
            .unwrap_err()
            .to_string()
            .contains("canonical")
    );
}

/// Launch checks detect image changes that preserve the native header.
#[tokio::test]
async fn b3_q_before_launch_sha() {
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open_mode(
        &dir.path().join("state/state.db"),
        crate::config::ExecutionMode::CliPipeline,
    )
    .unwrap();
    let config = crate::execution::pipeline::test_support::config(&owner, dir.path());
    CliQualifier::qualify(&owner, &config).await.unwrap();
    CliQualifier::before_launch(&owner, &config).unwrap();
    use std::io::Write;
    OpenOptions::new()
        .append(true)
        .open(&config.clis.codex.executable)
        .unwrap()
        .write_all(b"changed image")
        .unwrap();
    let error = CliQualifier::before_launch(&owner, &config).unwrap_err();
    assert!(
        error.to_string().contains("qualified executable changed"),
        "{error:#}"
    );
}

/// Equal image bytes at a different realpath cannot reuse a launch qualification.
#[tokio::test]
async fn b3_q_before_launch_realpath() {
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open_mode(
        &dir.path().join("state/state.db"),
        crate::config::ExecutionMode::CliPipeline,
    )
    .unwrap();
    let mut config = crate::execution::pipeline::test_support::config(&owner, dir.path());
    let original = config.clis.codex.executable.clone();
    let alias = original.with_file_name("alias");
    let copy = original.with_file_name("copy");
    std::fs::copy(&original, &copy).unwrap();
    std::os::unix::fs::symlink(&original, &alias).unwrap();
    config.clis.codex.executable = alias.clone();
    CliQualifier::qualify(&owner, &config).await.unwrap();
    CliQualifier::before_launch(&owner, &config).unwrap();
    std::fs::remove_file(&alias).unwrap();
    std::os::unix::fs::symlink(&copy, &alias).unwrap();
    let error = CliQualifier::before_launch(&owner, &config).unwrap_err();
    assert!(
        error.to_string().contains("qualified executable changed"),
        "{error:#}"
    );
}

/// The entire state directory excludes sibling work and home directories.
#[test]
fn b3_q_state_directory_separation() {
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open_mode(
        &dir.path().join("state/state.db"),
        crate::config::ExecutionMode::CliPipeline,
    )
    .unwrap();
    let config = crate::execution::pipeline::test_support::config(&owner, dir.path());
    CliQualifier::check_layout(&owner, &config).unwrap();
    let sibling = owner.state_path().parent().unwrap().join("sibling");
    std::fs::create_dir(&sibling).unwrap();
    std::fs::set_permissions(&sibling, std::fs::Permissions::from_mode(0o700)).unwrap();
    for home in [false, true] {
        let mut invalid = config.clone();
        if home {
            invalid.clis.codex.codex_home = Some(sibling.clone());
        } else {
            invalid.work_root = sibling.clone();
        }
        assert!(
            CliQualifier::check_layout(&owner, &invalid)
                .unwrap_err()
                .to_string()
                .contains("overlap")
        );
    }
}

/// Removed Projects neither block Q nor require an extant root at launch.
#[tokio::test]
async fn b3_q_removed_project() {
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open_mode(
        &dir.path().join("state/state.db"),
        crate::config::ExecutionMode::CliPipeline,
    )
    .unwrap();
    let config = crate::execution::pipeline::test_support::config(&owner, dir.path());
    for root in [dir.path().join("deleted"), config.work_root.clone()] {
        let mut project =
            crate::domain::Project::new("removed".into(), root, "fixture".into(), "main".into());
        project.state = crate::domain::ProjectState::Removed;
        owner
            .store
            .lock()
            .unwrap()
            .put_project(&mut project)
            .unwrap();
    }
    CliQualifier::qualify(&owner, &config).await.unwrap();
    CliQualifier::before_launch(&owner, &config).unwrap();
}

/// Agent writable and state paths cannot supply PATH entries or executable images.
#[tokio::test]
async fn b3_q_launch_path_separation() {
    for location in 0..4 {
        for image in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let owner = RuntimeOwner::open_mode(
                &dir.path().join("state/state.db"),
                crate::config::ExecutionMode::CliPipeline,
            )
            .unwrap();
            // A separate owner epoch/DB tests startup Q without attempting to
            // overwrite immutable Q observations from the positive control.
            let checker = RuntimeOwner::open_mode(
                &dir.path().join("state/checker.db"),
                crate::config::ExecutionMode::CliPipeline,
            )
            .unwrap();
            let mut config = crate::execution::pipeline::test_support::config(&owner, dir.path());
            let source = dir.path().join("source");
            std::fs::create_dir(&source).unwrap();
            for state in [&owner, &checker] {
                let mut project = crate::domain::Project::new(
                    "source".into(),
                    source.canonicalize().unwrap(),
                    "fixture".into(),
                    "main".into(),
                );
                state
                    .store
                    .lock()
                    .unwrap()
                    .put_project(&mut project)
                    .unwrap();
            }
            let root = [
                config.work_root.clone(),
                config.clis.codex.codex_home.clone().unwrap(),
                source.canonicalize().unwrap(),
                owner.state_path().parent().unwrap().to_path_buf(),
            ][location]
                .clone();
            let alias = dir.path().canonicalize().unwrap().join("safe-bin");
            let target = root.join("image");
            if image {
                std::fs::copy(&config.clis.codex.executable, &target).unwrap();
                std::os::unix::fs::symlink(&config.clis.codex.executable, &alias).unwrap();
                config.clis.codex.executable = alias.clone();
            } else {
                std::fs::create_dir(&alias).unwrap();
                config.path_prefix = vec![alias.clone()];
            }
            CliQualifier::qualify(&owner, &config).await.unwrap();
            CliQualifier::before_launch(&owner, &config).unwrap();
            if image {
                std::fs::remove_file(&alias).unwrap();
            } else {
                std::fs::remove_dir(&alias).unwrap();
            }
            std::os::unix::fs::symlink(if image { &target } else { &root }, &alias).unwrap();
            assert!(
                CliQualifier::before_launch(&owner, &config)
                    .unwrap_err()
                    .to_string()
                    .contains("launch path overlaps")
            );
            assert!(
                CliQualifier::qualify(&checker, &config)
                    .await
                    .unwrap_err()
                    .to_string()
                    .contains("launch path overlaps")
            );
        }
    }
}

/// Each probe gets its own five seconds while the CLI has a finite overall budget.
#[test]
fn b3_q_probe_deadlines() {
    let start = Instant::now();
    let overall = start + CLI_BOUND;
    let first = probe_deadline(start, overall);
    let later = start + Duration::from_secs(4);
    assert_eq!(first, start + Duration::from_secs(5));
    assert_eq!(
        probe_deadline(later, overall),
        later + Duration::from_secs(5)
    );
    assert_eq!(
        probe_deadline(overall - Duration::from_secs(1), overall),
        overall
    );
    assert!(CLI_BOUND > PROBE_BOUND && CLI_BOUND <= Duration::from_secs(35));
}

/// Successful sequential probes may together exceed one probe's time budget.
#[tokio::test]
async fn b3_q_sequential_probe_budget() {
    let f = Fixture::new();
    f.claude.control("delay-ms", "1100");
    let start = Instant::now();
    let row = f.q(false).await;
    assert_eq!(row.state, "qualified", "{row:?}");
    assert!(start.elapsed() > PROBE_BOUND);
}
