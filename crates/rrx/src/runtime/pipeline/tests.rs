//! Test-step entry points; scheduling remains the caller's responsibility.
use super::*;
use crate::execution::{
    pipeline::{PipelineTestRunner, SecretScanner, tests_runner::selected},
    verification::StreamEvidence,
};
use crate::state::pipeline::test_pass;
use serde_json::json;

impl Runtime {
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
        if self
            .finish_pipeline_test_cleanup(&cursor.run_id, Some(&cursor))
            .await?
        {
            return Ok(true);
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
                SecretScanner::check(&bytes)?;
            }
            test_pass(&step)
        })();
        let pass = match observed {
            Ok(pass) => pass,
            Err(_) => {
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_hold(&self.owner, &cursor, "test_evidence_incomplete", None)?;
                return Ok(false);
            }
        };
        let adopted = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let final_goal = run.kind == "goal"
                && step.body["command_index"].as_u64()
                    == Some(profile.proposal.commands.len() as u64 - 1);
            if final_goal {
                let view = store.pipeline_goal_view(run.goal_id)?;
                let prior_failed = view
                    .steps
                    .iter()
                    .filter(|s| s.run_id == run.id && s.kind == "test" && s.state == "adopted")
                    .any(|s| s.body["adoption"]["result"] != "pass");
                if !pass || prior_failed {
                    store.pipeline_hold(&self.owner, &cursor, "goal_tests_failed", None)?;
                    return Ok(false);
                }
                let goal = store.goal(run.goal_id)?.context("test Goal missing")?;
                store.pipeline_complete_goal(&self.owner, &cursor, goal.version, &command.id)?;
            } else {
                store.pipeline_adopt(
                    &self.owner,
                    &cursor,
                    &json!({"command_id":command.id,"result":if pass {"pass"} else {"fail"}}),
                )?;
            }
            store
                .pipeline_goal_view(run.goal_id)?
                .steps
                .iter()
                .any(|s| {
                    s.run_id == run.id
                        && s.seq == step.seq
                        && s.attempt == step.attempt
                        && s.state == "adopted"
                })
        };
        if !adopted {
            return Ok(false);
        }
        // Keep this commit's checkouts until all catalog commands have finished.
        if step.body["command_index"].as_u64() != Some(profile.proposal.commands.len() as u64 - 1) {
            self.wake.notify_one();
            return Ok(true);
        }
        self.finish_pipeline_test_cleanup(&run.id, None).await?;
        self.wake.notify_one();
        Ok(true)
    }

    async fn finish_pipeline_test_cleanup(
        &self,
        id: &str,
        cursor: Option<&PipelineCursor>,
    ) -> Result<bool> {
        let pending = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_test_cleanup(&self.owner, id, cursor)?;
        let Some((run, steps, project)) = pending else {
            return Ok(false);
        };
        let config = self
            .config
            .pipeline
            .as_ref()
            .context("pipeline config missing")?;
        for step in steps {
            let checkout = PipelineTestRunner::checkout(config, &step)?;
            PipelineTestRunner::remove(&project, &checkout).await?;
        }
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_test_cleanup_done(&self.owner, &run)?;
        self.wake.notify_one();
        Ok(true)
    }
}
