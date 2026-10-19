use super::{CliQualifier, PipelineRole, StepCustody, custody, qualification, step_arguments};
use crate::{
    adapter::ProcessGroup,
    config::PipelineConfig,
    execution::RuntimeOwner,
    state::pipeline::{PipelineCursor, PipelineExit, PipelineLaunch},
};
use anyhow::{Context, Result, ensure};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs::OpenOptions,
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, atomic::AtomicBool},
};
use tokio::sync::{Notify, OwnedMutexGuard};

/// Upstream supplies a verified workspace and a scanned prompt; never executable argv.
pub struct StepInput {
    pub work: PathBuf,
    pub prompt: Vec<u8>,
    pub schema: String,
}

pub enum LaunchResult {
    Waiting,
    Held,
    Unknown,
    RecordFailed,
    Started(StepCustody),
}

/// Constructed only with the Runtime's real execution boundary.
pub struct StepLauncher {
    owner: Arc<RuntimeOwner>,
    config: PipelineConfig,
    wake: Arc<Notify>,
}
impl StepLauncher {
    pub(crate) fn new(owner: Arc<RuntimeOwner>, config: PipelineConfig, wake: Arc<Notify>) -> Self {
        Self {
            owner,
            config,
            wake,
        }
    }

    pub(crate) async fn launch(
        self,
        boundary: OwnedMutexGuard<()>,
        mut cursor: PipelineCursor,
        input: StepInput,
    ) -> Result<LaunchResult> {
        let (run, step) = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_launch_snapshot(&self.owner, &cursor)?;
        ensure!(
            run.state == "running" && step.state == "planned",
            "pipeline launch state changed"
        );
        // Tests require the admitted/Seatbelt producer; internal steps never spawn here.
        let (role, codex) = match step.kind.as_str() {
            "implement" => (PipelineRole::Implement, true),
            "fix" => (PipelineRole::Fix, true),
            "confirm" => (PipelineRole::Confirm, true),
            "review" => (PipelineRole::Review, false),
            _ => anyhow::bail!("pipeline step has no Agent launch template"),
        };
        ensure!(
            step.provider.as_deref() == Some(if codex { "codex" } else { "claude" }),
            "pipeline provider differs from kind"
        );
        ensure!(
            !run.id.is_empty()
                && run
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b)),
            "invalid run directory component"
        );
        let cookie = uuid::Uuid::new_v4().simple().to_string();
        let attempt = self
            .config
            .work_root
            .join("runs")
            .join(&run.id)
            .join(format!("{}-{}", step.seq, step.attempt));
        let tmp = self.config.work_root.join("t").join(&cookie[..8]);
        let cli = if codex {
            &self.config.clis.codex
        } else {
            &self.config.clis.claude
        };
        let args = step_arguments(role, cli, &input.work, &attempt, &input.schema)?;
        let env = qualification::environment(&self.config, &tmp, &cookie, cli, codex)?;
        let env_bytes: Vec<_> = env.iter().map(|(k, v)| (k, v.as_encoded_bytes())).collect();
        let intent = json!({"attempt_dir":attempt,"tmpdir":tmp,
            "argv_sha256":hash(&serde_json::to_vec(&args)?),
            "env_sha256":hash(&serde_json::to_vec(&env_bytes)?)});
        let started = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let result =
                store.pipeline_begin_launch(&self.owner, &cursor, Some(&cookie), &intent)?;
            if result == PipelineLaunch::Started {
                cursor.step_version += 1;
                cursor.wait_version = store
                    .pipeline_takeover_view()?
                    .limit_wait
                    .map(|w| w.version);
            }
            result
        }; // tx1 and the Store mutex end before filesystem I/O or spawn.
        match started {
            PipelineLaunch::Waiting => return Ok(LaunchResult::Waiting),
            PipelineLaunch::Held => return Ok(LaunchResult::Held),
            PipelineLaunch::Started => {}
        }
        #[cfg(all(test, target_os = "macos"))]
        super::test_support::launch_checkpoint(super::test_support::LaunchPhase::Intent).await;
        let prepared = (|| -> Result<_> {
            CliQualifier::before_launch(&self.owner, &self.config)?;
            ensure!(
                input.prompt.len() <= 512 * 1024 && hash(&input.prompt) == step.input_sha256,
                "pipeline prompt differs from plan"
            );
            ensure!(
                input.work.canonicalize()? == input.work
                    && input.work.starts_with(&self.config.work_root)
                    && input.work != self.config.work_root,
                "pipeline workspace escaped private root"
            );
            if matches!(role, PipelineRole::Implement | PipelineRole::Fix) {
                ensure!(
                    run.worktree.as_deref() == input.work.to_str(),
                    "pipeline workspace differs from run"
                );
            }
            private_parents(
                &self.config.work_root,
                attempt.parent().context("attempt parent missing")?,
            )?;
            std::fs::DirBuilder::new().mode(0o700).create(&attempt)?;
            private_parents(
                &self.config.work_root,
                tmp.parent().context("temp parent missing")?,
            )?;
            std::fs::DirBuilder::new().mode(0o700).create(&tmp)?;
            ensure!(tmp.as_os_str().len() <= 60, "pipeline TMPDIR exceeds bound");
            write_input(&attempt.join("prompt.md"), &input.prompt)?;
            if matches!(role, PipelineRole::Confirm) {
                write_input(
                    &attempt.join("confirm.schema.json"),
                    input.schema.as_bytes(),
                )?;
            }
            if !codex {
                write_input(&attempt.join("mcp.json"), b"{\"mcpServers\":{}}")?;
            }
            // Recheck last, after materialization and immediately before spawn.
            CliQualifier::before_launch(&self.owner, &self.config)
        })();
        let images = match prepared {
            Ok(images) => images,
            Err(_) => {
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_reject(&self.owner, &cursor, "launch_preflight_failed")?;
                self.wake.notify_one();
                return Ok(LaunchResult::Held);
            }
        };
        let image = &images[usize::from(!codex)];
        #[cfg(all(test, target_os = "macos"))]
        super::test_support::launch_checkpoint(super::test_support::LaunchPhase::Verified).await;
        let mut command = tokio::process::Command::new(&image.realpath);
        command
            .args(args)
            .current_dir(&input.work)
            .env_clear()
            .envs(env)
            .process_group(0)
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let child = match rrx_spawn_hygiene::spawn_tokio(command) {
            Ok(child) => child,
            Err(_) => {
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_record_launch_unknown(&self.owner, &cursor, "launch_unknown")?;
                self.wake.notify_one();
                return Ok(LaunchResult::Unknown);
            }
        };
        let process = ProcessGroup::new(child, Arc::new(AtomicBool::new(true)))?;
        let pid = process.child.id().context("owned child PID missing")?;
        let birth = custody::birth(&cookie, pid).await;
        let recorded = birth.and_then(|birth| {
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_record_running(&self.owner, &cursor, pid, &birth)
        });
        if recorded.is_err() {
            // Failure after spawn can never become a pre-spawn rejection.
            if let Ok(dead) = custody::confirm_death(process, &cookie, true).await {
                let _ = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_finish_step(
                        &self.owner,
                        &cursor,
                        &dead.evidence,
                        PipelineExit {
                            outcome: Some("other"),
                            evidence: &json!({"issue":"record_failed"}),
                            issue: Some("record_failed"),
                        },
                    );
            }
            self.wake.notify_one();
            return Ok(LaunchResult::RecordFailed);
        }
        cursor.step_version += 1;
        #[cfg(all(test, target_os = "macos"))]
        super::test_support::launch_checkpoint(super::test_support::LaunchPhase::Recorded).await;
        let pin = cli.version.clone();
        drop(boundary); // No stdin/output wait or monitoring under control_admission.
        Ok(LaunchResult::Started(StepCustody::start(
            custody::Monitor {
                owner: self.owner,
                wake: self.wake,
                cursor,
                process,
                cookie,
                attempt,
                prompt: input.prompt,
                pin,
                role,
            },
        )))
    }
}

pub(super) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn private_parents(root: &Path, path: &Path) -> Result<()> {
    let mut current = root.to_path_buf();
    for component in path.strip_prefix(root)?.components() {
        ensure!(
            matches!(component, std::path::Component::Normal(_)),
            "invalid private path"
        );
        current.push(component);
        super::ensure_private_directory(&current)?;
    }
    Ok(())
}
fn write_input(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    file.set_permissions(std::fs::Permissions::from_mode(0o400))?;
    Ok(())
}
