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
        cursor: PipelineCursor,
        input: StepInput,
    ) -> Result<LaunchResult> {
        self.launch_inner(boundary, cursor, input, None).await
    }

    pub(crate) async fn launch_test(
        self,
        boundary: OwnedMutexGuard<()>,
        cursor: PipelineCursor,
        test: super::PreparedTest,
    ) -> Result<LaunchResult> {
        let input = StepInput {
            work: test.checkout.clone(),
            prompt: vec![],
            schema: String::new(),
        };
        self.launch_inner(boundary, cursor, input, Some(test)).await
    }

    async fn launch_inner(
        self,
        boundary: OwnedMutexGuard<()>,
        mut cursor: PipelineCursor,
        input: StepInput,
        test: Option<super::PreparedTest>,
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
        if step.kind == "review" {
            let (project, task, _) = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_review_context(&self.owner, &cursor)?;
            let checked = async {
                ensure!(
                    step.body["snapshot_tree"].is_string(),
                    "review_snapshot_missing"
                );
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_review_nonce_check(&self.owner, &cursor)?;
                let nonce = step.body["review_nonce"]
                    .as_str()
                    .context("review_nonce_invalid")?;
                super::review::validate_nonce(nonce)?;
                ensure!(
                    step.body["review_nonce_verified"].as_str() == Some(nonce),
                    "review_nonce_unverified"
                );
                ensure!(
                    input.work
                        == super::review_snapshot::snapshot_path(
                            &self.config.work_root,
                            &run.id,
                            step.seq,
                            step.attempt
                        )?,
                    "review_snapshot_path"
                );
                ensure!(
                    input.schema == super::review::REVIEW_SCHEMA,
                    "review_schema_mismatch"
                );
                super::SecretScanner::check(&input.prompt)?;
                let git = super::PipelineGit::new(&project.root)?;
                let diff = git.diff(&run.base_commit, &step.step_base).await?;
                let prompt = super::review::review_prompt(nonce, &task, &diff)?;
                ensure!(
                    input.prompt == prompt && hash(&prompt) == step.input_sha256,
                    "review_input_changed"
                );
                let tree = super::review_snapshot::verify(
                    &git,
                    &self.config.work_root,
                    &input.work,
                    &step.step_base,
                )
                .await
                .context("review_snapshot_changed")?;
                ensure!(
                    step.body["snapshot_tree"] == tree,
                    "review_snapshot_changed"
                );
                Ok::<_, anyhow::Error>(())
            }
            .await;
            if let Err(error) = checked {
                let mut store = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?;
                store.pipeline_launch_snapshot(&self.owner, &cursor)?;
                if let Some(hit) = error.downcast_ref::<super::SecretHit>() {
                    store.pipeline_hold_secret(
                        &self.owner,
                        &cursor,
                        &json!({"pattern":hit.pattern,"offset_sha256":hit.offset_sha256}),
                    )?;
                } else {
                    let reasons = [
                        "review_snapshot_missing",
                        "review_nonce_invalid",
                        "review_nonce_reused",
                        "review_nonce_unverified",
                        "review_snapshot_path",
                        "review_schema_mismatch",
                        "review_input_changed",
                        "review_nonce_collision",
                        "review_snapshot_changed",
                        "diff_too_large",
                    ];
                    let reason = error
                        .chain()
                        .find_map(|e| reasons.iter().copied().find(|r| e.to_string() == *r))
                        .unwrap_or("review_validation_failed");
                    store.pipeline_hold(&self.owner, &cursor, reason, None)?;
                }
                return Ok(LaunchResult::Held);
            }
        }
        // Tests require the admitted/Seatbelt producer; internal steps never spawn here.
        let (role, codex) = match step.kind.as_str() {
            "test" if test.is_some() => (PipelineRole::Implement, false),
            "implement" => (PipelineRole::Implement, true),
            "fix" => (PipelineRole::Fix, true),
            "confirm" => (PipelineRole::Confirm, true),
            "review" => (PipelineRole::Review, false),
            _ => anyhow::bail!("pipeline step has no Agent launch template"),
        };
        ensure!(
            step.provider.as_deref()
                == if test.is_some() {
                    None
                } else {
                    Some(if codex { "codex" } else { "claude" })
                },
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
        let args = if let Some(test) = &test {
            let mut args = vec![
                "-f".into(),
                attempt
                    .join("test.sb")
                    .to_str()
                    .context("profile encoding")?
                    .into(),
                "--".into(),
                test.command
                    .program
                    .to_str()
                    .context("program encoding")?
                    .into(),
            ];
            args.extend(test.command.args.clone());
            args
        } else {
            step_arguments(role, cli, &input.work, &attempt, &input.schema)?
        };
        let env = qualification::environment(
            &self.config,
            &tmp,
            &cookie,
            cli,
            codex,
            step.kind == "review",
        )?;
        let env_bytes: Vec<_> = env.iter().map(|(k, v)| (k, v.as_encoded_bytes())).collect();
        let intent = json!({"attempt_dir":attempt,"tmpdir":tmp,
            "argv_sha256":hash(&serde_json::to_vec(&args)?),
            "env_sha256":hash(&serde_json::to_vec(&env_bytes)?)});
        let launch = rrx_process_tracker::launch_mark()?;
        let started = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let result = store.pipeline_begin_launch(
                &self.owner,
                &cursor,
                Some(&cookie),
                Some(&launch),
                &intent,
            )?;
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
                test.is_some()
                    || (input.prompt.len() <= 512 * 1024
                        && hash(&input.prompt) == step.input_sha256),
                "pipeline prompt differs from plan"
            );
            ensure!(
                input.work.canonicalize()? == input.work
                    && input.work.starts_with(&self.config.work_root)
                    && input.work != self.config.work_root,
                "pipeline workspace escaped private root"
            );
            if test.is_none() && matches!(role, PipelineRole::Implement | PipelineRole::Fix) {
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
            if test.is_none() && !codex {
                write_input(&attempt.join("mcp.json"), b"{\"mcpServers\":{}}")?;
            }
            if let Some(test) = &test {
                let policy = super::tests_runner::seatbelt(
                    &test.checkout,
                    &tmp,
                    &self.config.test_cache_dirs,
                    &self.config.work_root,
                    &test.forbidden,
                )?;
                write_input(&attempt.join("test.sb"), policy.as_bytes())?;
            }
            let cwd = if let Some(test) = &test {
                let cwd = test.checkout.join(&test.command.cwd).canonicalize()?;
                ensure!(
                    cwd.starts_with(&test.checkout) && cwd.is_dir(),
                    "test cwd escaped checkout"
                );
                cwd
            } else {
                input.work.clone()
            };
            // Recheck last, after materialization and immediately before spawn.
            Ok((CliQualifier::before_launch(&self.owner, &self.config)?, cwd))
        })();
        let (images, cwd) = match prepared {
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
        let program = if test.is_some() {
            PathBuf::from("/usr/bin/sandbox-exec")
        } else {
            image.realpath.clone()
        };
        let mut command = tokio::process::Command::new(program);
        command
            .args(args)
            .current_dir(cwd)
            .env_clear()
            .envs(env)
            .process_group(0)
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(test) = &test {
            // No file materialization or qualification may intervene between recheck and spawn.
            let checked = async {
                test.profile
                    .recheck_async(&test.command, self.config.test_writable_paths()?)
                    .await
            }
            .await;
            if checked.is_err() {
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_reject(&self.owner, &cursor, "tests_program_changed")?;
                self.wake.notify_one();
                return Ok(LaunchResult::Held);
            }
        }
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
        #[cfg(all(test, target_os = "macos"))]
        super::test_support::spawned_checkpoint(pid).await;
        let leader_cookie = rrx_process_tracker::Cookie::new(&cookie)?;
        let test_leader = test.is_some();
        let birth = tokio::task::spawn_blocking(move || {
            if test_leader {
                rrx_process_tracker::owned_test_leader(pid, &leader_cookie)
            } else {
                rrx_process_tracker::owned_leader(pid, &leader_cookie)
            }
        })
        .await?;
        let recorded_birth = birth.as_ref().ok().map(|leader| leader.birth);
        let recorded = birth.map_err(anyhow::Error::from).and_then(|leader| {
            let birth = format!("{}:{}", leader.birth.0, leader.birth.1);
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_record_running(&self.owner, &cursor, pid, &birth)
        });
        if recorded.is_err() {
            // Failure after spawn can never become a pre-spawn rejection.
            if let Ok(dead) =
                custody::confirm_death(process, &cookie, &launch, recorded_birth, true).await
            {
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
                            outcome: if test.is_some() { None } else { Some("other") },
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
                launch,
                birth: recorded_birth.context("recorded leader birth missing")?,
                attempt,
                prompt: input.prompt,
                kind: if let Some(test) = test {
                    custody::MonitorKind::Test(test.command)
                } else {
                    custody::MonitorKind::Agent { pin, role }
                },
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
