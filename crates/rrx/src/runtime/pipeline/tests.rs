//! Test-step entry points; scheduling remains the caller's responsibility.
use super::*;
use crate::execution::{
    pipeline::{PipelineTestRunner, SecretHit, SecretScanner, tests_runner::selected},
    verification::StreamEvidence,
};
use crate::runtime::PipelineCleanupOutcome;
use crate::state::pipeline::test_pass;
use serde_json::json;

impl Runtime {
    #[cfg(all(test, target_os = "macos"))]
    pub(crate) async fn wait_pipeline_test_boundary(&self) {
        assert!(
            self.service_running(),
            "pipeline test helper cannot retry: service stopped"
        );
        drop(self.control_admission.lock().await);
    }
    #[cfg(all(test, target_os = "macos"))]
    pub(crate) fn pipeline_test_service_running(&self) -> bool {
        self.service_running()
    }
    #[cfg(all(test, target_os = "macos"))]
    pub(crate) async fn hold_pipeline_test_boundary(&self) -> tokio::sync::OwnedMutexGuard<()> {
        self.control_admission.clone().lock_owned().await
    }
    /// Prepares only a planned step's detached checkout and launches with the shared custody.
    pub async fn launch_pipeline_test(&self, cursor: PipelineCursor) -> Result<LaunchResult> {
        ensure!(
            self.config.execution.mode == ExecutionMode::CliPipeline,
            "ExecutionModeMismatch"
        );
        let Ok(boundary) = self.control_admission.clone().try_lock_owned() else {
            return Ok(LaunchResult::Waiting);
        };
        if !self.service_running() || self.stopping.load(Ordering::SeqCst) {
            return Ok(LaunchResult::Waiting);
        }
        let profile = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_test_context(&self.owner, &cursor)?;
        let Some(profile) = profile else {
            return Ok(LaunchResult::Held);
        };
        let (run, step) = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_launch_snapshot(&self.owner, &cursor)?;
        let config = self
            .config
            .pipeline
            .as_ref()
            .context("pipeline config missing")?;
        let prepared = PipelineTestRunner::prepare(&self.owner, config, &run, &step, profile).await;
        let test = match prepared {
            Ok(test) => test,
            Err(_) => {
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_hold(&self.owner, &cursor, "test_checkout_failed", None)?;
                return Ok(LaunchResult::Held);
            }
        };
        StepLauncher::new(self.owner.clone(), config.clone(), self.wake.clone())
            .launch_test(boundary, cursor, test)
            .await
    }

    /// Adopts both passing and failing observations after re-reading retained streams.
    pub async fn adopt_pipeline_test(&self, cursor: PipelineCursor) -> Result<bool> {
        self.adopt_test_result(cursor, false).await
    }

    /// Stage the last round result so a hold can still use exited-to-held.
    pub async fn adopt_pipeline_round_test(&self, cursor: PipelineCursor) -> Result<bool> {
        self.adopt_test_result(cursor, true).await
    }

    async fn adopt_test_result(&self, cursor: PipelineCursor, defer: bool) -> Result<bool> {
        ensure!(
            self.config.execution.mode == ExecutionMode::CliPipeline,
            "ExecutionModeMismatch"
        );
        let Ok(_boundary) = self.control_admission.try_lock() else {
            return Ok(false);
        };
        if !self.service_running() || self.stopping.load(Ordering::SeqCst) {
            return Ok(false);
        }
        let profile = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_test_context(&self.owner, &cursor)?;
        let Some(profile) = profile else {
            return Ok(false);
        };
        let (run, step) = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_launch_snapshot(&self.owner, &cursor)?;
        // Task results are staged for their round decision (§8.5); Goal checks adopt directly.
        let reason = if defer {
            "round_result_kind"
        } else {
            "round_adoption_required"
        };
        ensure!(defer == (run.kind == "task"), reason);
        let command = selected(&profile, &step)?;
        let config = self
            .config
            .pipeline
            .as_ref()
            .context("pipeline config missing")?;
        let attempt = config
            .work_root
            .join("runs")
            .join(&run.id)
            .join(format!("{}-{}", step.seq, step.attempt));
        let observed = (|| -> Result<bool> {
            ensure!(
                step.body["intent"]["attempt_dir"].as_str() == attempt.to_str(),
                "test attempt changed"
            );
            for (file, key, limit) in [
                ("stdout.log", "stdout", command.stdout_bytes),
                ("stderr.log", "stderr", command.stderr_bytes),
            ] {
                let evidence: StreamEvidence =
                    serde_json::from_value(step.body["exit"][key].clone())?;
                let bytes =
                    crate::execution::pipeline::adoption::read_regular(&attempt.join(file), limit)?;
                // inspect uses the collector's fixed leaf-name contract; compare directly here.
                ensure!(
                    bytes.len() as u64 == evidence.bytes
                        && crate::execution::verification::hash(&bytes) == evidence.sha256,
                    "test output evidence changed"
                );
                SecretScanner::check(&bytes).with_context(|| file)?;
            }
            test_pass(&step)
        })();
        let pass = match observed {
            Ok(pass) => pass,
            Err(error) => {
                let mut store = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?;
                if let Some(hit) = error.downcast_ref::<SecretHit>() {
                    let file = error
                        .chain()
                        .map(ToString::to_string)
                        .find(|text| matches!(text.as_str(), "stdout.log" | "stderr.log"))
                        .context("test secret evidence file missing")?;
                    store.pipeline_hold_secret(&self.owner, &cursor, &json!({
                        "pattern": hit.pattern, "file": file, "offset_sha256": hit.offset_sha256,
                    }))?;
                } else {
                    store.pipeline_hold(&self.owner, &cursor, "test_evidence_incomplete", None)?;
                }
                return Ok(false);
            }
        };
        let last =
            step.body["command_index"].as_u64() == Some(profile.proposal.commands.len() as u64 - 1);
        let adopted = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            if run.kind == "goal" && !pass {
                // Recovery acts on the last step, so hold the failing command's own step.
                store.pipeline_hold(&self.owner, &cursor, "goal_check_failed", None)?;
                return Ok(false);
            }
            if run.kind == "goal" && last {
                let goal = store.goal(run.goal_id)?.context("test Goal missing")?;
                store.pipeline_complete_goal(&self.owner, &cursor, goal.version, &command.id)?;
            } else {
                let evidence =
                    json!({"command_id":command.id,"result":if pass {"pass"} else {"fail"}});
                if defer {
                    store.pipeline_stage_round_result(&self.owner, &cursor, &evidence)?;
                } else {
                    store.pipeline_adopt(&self.owner, &cursor, &evidence)?;
                }
            }
            store
                .pipeline_goal_view(run.goal_id)?
                .steps
                .iter()
                .any(|s| {
                    s.run_id == run.id
                        && s.seq == step.seq
                        && s.attempt == step.attempt
                        && (s.state == "adopted"
                            || (defer
                                && s.state == "exited"
                                && s.body["round_result_validated"] == true))
                })
        };
        if !adopted {
            return Ok(false);
        }
        self.wake.notify_one();
        Ok(true)
    }

    pub(crate) async fn sweep_pipeline_test_cleanup(&self) -> Result<PipelineCleanupOutcome> {
        let Ok(_sweep) = self.pipeline_cleanup_sweep.try_lock() else {
            return Ok(PipelineCleanupOutcome::Complete);
        };
        let runs = {
            let Ok(_boundary) = self.control_admission.try_lock() else {
                return Ok(PipelineCleanupOutcome::BoundaryBusy);
            };
            if !self.service_running() {
                return Ok(PipelineCleanupOutcome::Complete);
            }
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_test_cleanup_runs(&self.owner)?
        };
        for id in runs {
            match self.finish_pipeline_test_cleanup(&id).await {
                Ok(PipelineCleanupOutcome::BoundaryBusy) => {
                    return Ok(PipelineCleanupOutcome::BoundaryBusy);
                }
                Ok(PipelineCleanupOutcome::Complete) => {}
                Err(_) => {
                    eprintln!("test cleanup sweep failed: run={id} class=cleanup_sweep");
                    if self.record_pipeline_cleanup_sweep_error(&id).is_err() {
                        eprintln!(
                            "test cleanup error could not be recorded: run={id} class=cleanup_sweep"
                        );
                    }
                }
            }
        }
        Ok(PipelineCleanupOutcome::Complete)
    }

    fn record_pipeline_cleanup_sweep_error(&self, id: &str) -> Result<()> {
        let _boundary = self.control_admission.try_lock()?;
        let pending = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_test_cleanup_pending(&self.owner, id)?;
        let config = self
            .config
            .pipeline
            .as_ref()
            .context("pipeline config missing")?;
        for obligation in pending {
            let path =
                PipelineTestRunner::checkout_path(config, id, obligation.seq, obligation.attempt)?;
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_test_cleanup_result(
                    &self.owner,
                    &obligation,
                    Some(("cleanup_sweep", &path)),
                )?;
        }
        Ok(())
    }

    async fn finish_pipeline_test_cleanup(&self, id: &str) -> Result<PipelineCleanupOutcome> {
        let pending = {
            let Ok(_boundary) = self.control_admission.try_lock() else {
                return Ok(PipelineCleanupOutcome::BoundaryBusy);
            };
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_test_cleanup(&self.owner, id)?
        };
        let config = self
            .config
            .pipeline
            .as_ref()
            .context("pipeline config missing")?;
        for (obligation, step, project) in pending {
            let checkout = PipelineTestRunner::checkout(config, &step)?;
            let removed = PipelineTestRunner::remove(&project, config, &step).await;
            let tmp_reclaimed = if removed.is_ok() {
                let config = config.clone();
                let step = step.clone();
                tokio::task::spawn_blocking(move || {
                    PipelineTestRunner::reclaim_tmpdir(&config, &step)
                })
                .await?
            } else {
                Ok(())
            };
            let failure = if removed.is_err() {
                Some(("checkout_remove", checkout))
            } else if tmp_reclaimed.is_err() {
                // Only the rrx-derived path is retained; never the test's recorded value.
                let cookie = step.cookie.as_deref().context("cleanup cookie missing")?;
                let prefix = cookie.get(..8).context("cleanup cookie short")?;
                Some(("tmpdir_remove", config.work_root.join("t").join(prefix)))
            } else {
                None
            };
            // Attempts/cookies have disjoint paths. A launch may proceed during removal;
            // the result transaction must still revalidate the current owner/epoch.
            // Shutdown holds this boundary while joining the supervisor: never wait here.
            let Ok(_boundary) = self.control_admission.try_lock() else {
                return Ok(PipelineCleanupOutcome::BoundaryBusy);
            };
            if let Some((class, path)) = &failure {
                eprintln!(
                    "test cleanup pending: run={id} class={class} path={}",
                    path.display()
                );
            }
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_test_cleanup_result(
                    &self.owner,
                    &obligation,
                    failure
                        .as_ref()
                        .map(|(class, path)| (*class, path.as_path())),
                )?;
        }
        Ok(PipelineCleanupOutcome::Complete)
    }
}
