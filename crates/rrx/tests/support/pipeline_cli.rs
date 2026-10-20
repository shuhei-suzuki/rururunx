//! Native file-controlled fake executables, exercising the real Mach-O Q path.
use crate::config::PipelineCli;
use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

pub struct FakeCli {
    pub directory: PathBuf,
    pub cli: PipelineCli,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LaunchPhase {
    Intent,
    Verified,
    Spawned,
    Recorded,
    Cleanup,
}

struct LaunchCheckpoint {
    phase: LaunchPhase,
    arrived: tokio::sync::oneshot::Sender<()>,
    wait: tokio::sync::oneshot::Receiver<()>,
}

// These checkpoint tests run on Tokio's current-thread runtime.
thread_local! {
    static LAUNCH_CHECKPOINT: std::cell::RefCell<Option<LaunchCheckpoint>> = const { std::cell::RefCell::new(None) };
    static SPAWNED_LEADER: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
}

/// A checkpoint delays execution only; it supplies no launch or Store authority.
pub fn pause_launch(
    phase: LaunchPhase,
) -> (
    tokio::sync::oneshot::Sender<()>,
    tokio::sync::oneshot::Receiver<()>,
) {
    let (release, wait) = tokio::sync::oneshot::channel();
    let (arrived, arrival) = tokio::sync::oneshot::channel();
    LAUNCH_CHECKPOINT.with(|slot| {
        assert!(slot.borrow().is_none());
        *slot.borrow_mut() = Some(LaunchCheckpoint {
            phase,
            arrived,
            wait,
        });
    });
    (release, arrival)
}

pub async fn launch_checkpoint(phase: LaunchPhase) {
    let wait = LAUNCH_CHECKPOINT.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot
            .as_ref()
            .is_some_and(|checkpoint| checkpoint.phase == phase)
        {
            slot.take()
        } else {
            None
        }
    });
    if let Some(checkpoint) = wait {
        checkpoint
            .arrived
            .send(())
            .expect("checkpoint arrival abandoned");
        checkpoint.wait.await.expect("launch checkpoint abandoned");
    }
}

/// Records the spawned leader PID for observation only; it supplies no signal authority.
pub async fn spawned_checkpoint(pid: u32) {
    SPAWNED_LEADER.set(Some(pid));
    launch_checkpoint(LaunchPhase::Spawned).await;
}

pub fn spawned_leader() -> Option<u32> {
    SPAWNED_LEADER.take()
}

pub async fn qualify_fixture(
    owner: &crate::execution::RuntimeOwner,
    config: &crate::config::PipelineConfig,
) -> anyhow::Result<()> {
    // Bound harness process contention during fixture setup; this is not a product lock.
    static SETUP_PERMITS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(4);
    let _permit = SETUP_PERMITS.acquire().await.unwrap();
    crate::execution::pipeline::CliQualifier::qualify(owner, config).await
}

pub fn config(
    owner: &crate::execution::RuntimeOwner,
    root: &Path,
) -> crate::config::PipelineConfig {
    let codex = FakeCli::new(root, true);
    let claude = FakeCli::new(root, false);
    let work_root = std::fs::canonicalize(owner.ipc_path().parent().unwrap()).unwrap();
    std::fs::set_permissions(&work_root, std::fs::Permissions::from_mode(0o700)).unwrap();
    crate::config::PipelineConfig {
        test_cache_dirs: vec![],
        // Reuse the existing owner/IPC fixture's short, private directory.
        work_root,
        path_prefix: vec![],
        clis: crate::config::PipelineClis {
            codex: codex.cli,
            claude: claude.cli,
        },
    }
}
impl FakeCli {
    pub fn new(root: &Path, codex: bool) -> Self {
        // Host TMPDIR may contain a symlink (e.g. /var on macOS).
        let root = std::fs::canonicalize(root).unwrap();
        let name = if codex { "codex" } else { "claude" };
        let directory = root.join(name);
        std::fs::create_dir_all(&directory).unwrap();
        let executable = directory.join(name);
        static NATIVE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
        let native = NATIVE.get_or_init(|| {
            let dir = tempfile::Builder::new()
                .prefix("b3-native-")
                .tempdir()
                .unwrap();
            let source = dir.path().join("fake.c");
            std::fs::write(&source, include_str!("pipeline_cli.c")).unwrap();
            let mut compiler = std::process::Command::new("/usr/bin/cc");
            compiler
                .args(["-Wall", "-Wextra", "-Werror"])
                .arg(&source)
                .arg("-o")
                .arg(dir.path().join("fake"));
            assert!(rrx_spawn_hygiene::status_std(compiler).unwrap().success());
            std::fs::read(dir.path().join("fake")).unwrap()
        });
        std::fs::write(&executable, native).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        let version = format!("rrx-fixture-{name}-v1");
        let flags = if codex {
            "--sandbox <MODE>\n-c, --config <VALUE>\n-m, --model <MODEL>\n-C, --cd <PATH>\n-o, --output-last-message <FILE>\n--json\n--color <WHEN>\n--output-schema <FILE>\n"
        } else {
            "-p, --print\n--permission-mode <MODE>\n--tools <TOOLS>\n--disallowedTools <TOOLS>\n--strict-mcp-config\n--mcp-config <FILE>\n--setting-sources <SOURCES>\n--disable-slash-commands\n--no-session-persistence\n--model <MODEL>\n--output-format <FORMAT>\n--json-schema <SCHEMA>\n"
        };
        for (file, content) in [
            ("version", version.as_str()),
            ("help", flags),
            ("login", "0\n"),
            ("settings-status", "0\n"),
            ("exec-status", "0\n"),
            ("exec-output", "fixture output\n"),
        ] {
            std::fs::write(directory.join(file), content).unwrap();
        }
        let home = directory.join("home");
        std::fs::create_dir(&home).unwrap();
        std::fs::set_permissions(&home, std::fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            directory,
            cli: PipelineCli {
                executable,
                version,
                model: "fixture-model".into(),
                effort: codex.then_some(crate::config::ReasoningEffort::High),
                codex_home: codex.then_some(home),
            },
        }
    }
    pub fn control(&self, name: &str, content: &str) {
        std::fs::write(self.directory.join(name), content).unwrap();
    }
    pub fn expect_arguments(&self, args: &[&str]) {
        let mut bytes = Vec::new();
        for arg in args {
            bytes.extend_from_slice(arg.as_bytes());
            bytes.push(0);
        }
        std::fs::write(self.directory.join("expected-argv"), bytes).unwrap();
    }
}
