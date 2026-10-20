//! Rule-G review entry points. Neither a Task checkout nor caller-provided argv is trusted.
use super::super::Runtime;
use crate::{
    config::ExecutionMode,
    execution::pipeline::{
        LaunchResult, PipelineGit, SecretHit, StepInput, StepLauncher, VerdictDecoder,
        review::{self, REVIEW_SCHEMA},
        review_snapshot,
    },
    state::pipeline::{PipelineCursor, PipelineRun, PipelineStep},
};
use anyhow::{Context, Result, ensure};
use serde_json::json;
use std::sync::atomic::Ordering;

impl Runtime {
    fn reviewer_ready(&self) -> Result<bool> {
        ensure!(
            self.config.execution.mode == ExecutionMode::CliPipeline,
            "ExecutionModeMismatch"
        );
        Ok(self.service_running() && !self.stopping.load(Ordering::SeqCst))
    }
    fn review_snapshot(&self, cursor: &PipelineCursor) -> Result<(PipelineRun, PipelineStep)> {
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_launch_snapshot(&self.owner, cursor)
    }
    async fn prepare_review(&self, cursor: &PipelineCursor) -> Result<(StepInput, String)> {
        let (run, step) = self.review_snapshot(cursor)?;
        ensure!(
            step.kind == "review" && step.state == "planned",
            "review_prepare_state"
        );
        let (project, task, _) = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_review_context(&self.owner, cursor)?;
        let config = self
            .config
            .pipeline
            .as_ref()
            .context("pipeline config missing")?;
        let git = PipelineGit::new(&project.root)?;
        let diff = git.diff(&run.base_commit, &step.step_base).await?;
        let nonce = step.body["review_nonce"]
            .as_str()
            .context("review_nonce_invalid")?;
        let prompt = review::review_prompt(nonce, &task, &diff)?;
        ensure!(
            crate::execution::verification::hash(&prompt) == step.input_sha256,
            "review_input_changed"
        );
        let path =
            review_snapshot::snapshot_path(&config.work_root, &run.id, step.seq, step.attempt)?;
        if let Some(work) = &run.worktree {
            let work = std::path::Path::new(work);
            ensure!(
                !path.starts_with(work) && !work.starts_with(&path),
                "review_snapshot_changed"
            );
        }
        let tree =
            review_snapshot::rebuild(&git, &config.work_root, &path, &step.step_base).await?;
        Ok((
            StepInput {
                work: path,
                prompt,
                schema: REVIEW_SCHEMA.into(),
            },
            tree,
        ))
    }

    /// Materialize from the selected commit; retries delete and rebuild this private snapshot.
    pub async fn prepare_pipeline_review(
        &self,
        cursor: PipelineCursor,
    ) -> Result<Option<StepInput>> {
        let Ok(_boundary) = self.control_admission.try_lock() else {
            return Ok(None);
        };
        if !self.reviewer_ready()? {
            return Ok(None);
        }
        match self.prepare_review(&cursor).await {
            Ok((input, _)) => Ok(Some(input)),
            Err(e) => {
                self.review_failure(&cursor, &e, false)?;
                Ok(None)
            }
        }
    }
    /// Launch with the fixed Reviewer schema/argv and the Agent's matching-cookie custody.
    pub async fn launch_pipeline_review(&self, mut cursor: PipelineCursor) -> Result<LaunchResult> {
        let Ok(boundary) = self.control_admission.clone().try_lock_owned() else {
            return Ok(LaunchResult::Waiting);
        };
        if !self.reviewer_ready()? {
            return Ok(LaunchResult::Waiting);
        }
        let (input, tree) = match self.prepare_review(&cursor).await {
            Ok(prepared) => prepared,
            Err(e) => {
                self.review_failure(&cursor, &e, false)?;
                return Ok(LaunchResult::Held);
            }
        };
        let recorded = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_review_preflight(&self.owner, &cursor, &tree);
        if let Err(e) = recorded {
            self.review_failure(&cursor, &e, false)?;
            return Ok(LaunchResult::Held);
        }
        cursor.step_version += 1;
        StepLauncher::new(
            self.owner.clone(),
            self.config
                .pipeline
                .clone()
                .context("pipeline config missing")?,
            self.wake.clone(),
        )
        .launch(boundary, cursor, input)
        .await
    }

    /// Decode only retained, scan-checked output after death and a second target-tree check.
    pub async fn adopt_pipeline_review(&self, cursor: PipelineCursor) -> Result<bool> {
        let Ok(_boundary) = self.control_admission.try_lock() else {
            return Ok(false);
        };
        if !self.reviewer_ready()? {
            return Ok(false);
        }
        let (run, step) = self.review_snapshot(&cursor)?;
        ensure!(
            step.kind == "review" && step.state == "exited",
            "review_adoption_state"
        );
        let checked = async {
            let (project, _, threshold) = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_review_context(&self.owner, &cursor)?;
            let config = self
                .config
                .pipeline
                .as_ref()
                .context("pipeline config missing")?;
            let path =
                review_snapshot::snapshot_path(&config.work_root, &run.id, step.seq, step.attempt)?;
            ensure!(
                step.body["death_confirmed"] == true,
                "review_exit_uncertified"
            );
            let tree = review_snapshot::verify(
                &PipelineGit::new(&project.root)?,
                &config.work_root,
                &path,
                &step.step_base,
            )
            .await
            .context("review_snapshot_changed")?;
            ensure!(
                step.body["snapshot_tree"] == tree,
                "review_snapshot_changed"
            );
            let attempt = config
                .work_root
                .join("runs")
                .join(&run.id)
                .join(format!("{}-{}", step.seq, step.attempt));
            ensure!(
                step.body["intent"]["attempt_dir"].as_str() == attempt.to_str(),
                "review_output_changed"
            );
            let bytes = review::output(&step, &attempt)?;
            if step.body["provider_outcome"] == "limit" {
                return Ok(None);
            }
            ensure!(step.body["exit"]["code"] == 0, "review_agent_failed");
            let mut decoded = VerdictDecoder::decode(&bytes, &config.clis.claude.model, threshold)?;
            ensure!(
                step.body["provider_outcome"] == "success",
                "review_agent_failed"
            );
            decoded["snapshot_tree"] = json!(tree);
            let mut stored = step.body.clone();
            stored["adoption"] = decoded.clone();
            ensure!(
                serde_json::to_vec(&stored)?.len() <= 260 * 1024,
                "review_storage_limit"
            );
            Ok::<_, anyhow::Error>(Some(decoded))
        }
        .await;
        match checked {
            Ok(Some(evidence)) => {
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_adopt(&self.owner, &cursor, &evidence)?;
                self.wake.notify_one();
                Ok(true)
            }
            Ok(None) => {
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_record_limited(&self.owner, &cursor)?;
                self.wake.notify_one();
                Ok(false)
            }
            Err(e) => {
                self.review_failure(&cursor, &e, true)?;
                Ok(false)
            }
        }
    }
    fn review_failure(
        &self,
        cursor: &PipelineCursor,
        error: &anyhow::Error,
        exited: bool,
    ) -> Result<()> {
        let mut store = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        if let Some(hit) = error.downcast_ref::<SecretHit>() {
            store.pipeline_hold_secret(
                &self.owner,
                cursor,
                &json!({"pattern":hit.pattern,"offset_sha256":hit.offset_sha256}),
            )?;
        } else {
            let reasons = [
                "verdict_schema",
                "review_envelope_mismatch",
                "review_model_mismatch",
                "review_snapshot_changed",
                "review_agent_failed",
                "review_output_changed",
                "diff_too_large",
                "review_storage_limit",
                "review_nonce_invalid",
                "review_nonce_collision",
                "review_nonce_reused",
                "review_input_changed",
                "review_snapshot_unsupported",
            ];
            let reason = error
                .chain()
                .find_map(|cause| reasons.iter().copied().find(|r| cause.to_string() == *r))
                .unwrap_or("review_validation_failed");
            if exited && matches!(reason, "verdict_schema" | "review_agent_failed") {
                store.pipeline_reject(&self.owner, cursor, reason)?;
            } else {
                store.pipeline_hold(&self.owner, cursor, reason, None)?;
            }
        }
        self.wake.notify_one();
        Ok(())
    }
}
