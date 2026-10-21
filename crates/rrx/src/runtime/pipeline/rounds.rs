//! Explicit round entry points; the automatic tick and release execution are separate.
use crate::{
    config::ExecutionMode,
    execution::pipeline::{
        self, LaunchResult, PipelineGit, RoundDecision, StepInput, StepLauncher, confirm, review,
        review_snapshot, rounds,
    },
    runtime::Runtime,
    state::pipeline::{
        PipelineCursor, PipelineStep, RoundContext, round_action, round_fix_input,
        round_source_review, step_body_fits,
    },
};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeSet, sync::atomic::Ordering};

impl Runtime {
    fn round_ready(&self) -> Result<bool> {
        ensure!(
            self.config.execution.mode == ExecutionMode::CliPipeline,
            "ExecutionModeMismatch"
        );
        Ok(self.service_running() && !self.stopping.load(Ordering::SeqCst))
    }
    fn round_context(&self, cursor: &PipelineCursor) -> Result<RoundContext> {
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_round_context(&self.owner, cursor)
    }

    /// Begin the re-test using the existing admitted test producer after an rrx commit.
    pub async fn plan_pipeline_round_test(&self, cursor: PipelineCursor) -> Result<bool> {
        let Ok(_boundary) = self.control_admission.try_lock() else {
            return Ok(false);
        };
        if !self.round_ready()? {
            return Ok(false);
        }
        let ctx = match self.round_context(&cursor) {
            // The Store held the planned re-test; nothing runs (§7.2).
            Err(error) if error.to_string() == "tests_profile_changed" => {
                self.wake.notify_one();
                return Ok(false);
            }
            ctx => ctx?,
        };
        ensure!(
            matches!(ctx.step.kind.as_str(), "implement" | "fix") && ctx.step.state == "adopted",
            "round_test_source"
        );
        let next = next_step(&ctx, "test", ctx.run.round, test_body(&ctx, 0)?);
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_plan_next(&self.owner, &cursor, &next, true)?;
        Ok(true)
    }

    /// Apply the ordered table to stored decoded observations; no LLM chooses the next kind.
    pub async fn decide_pipeline_round(
        &self,
        mut cursor: PipelineCursor,
    ) -> Result<Option<RoundDecision>> {
        let Ok(_boundary) = self.control_admission.try_lock() else {
            return Ok(None);
        };
        if !self.round_ready()? {
            return Ok(None);
        }
        let mut ctx = match self.round_context(&cursor) {
            Err(error) if error.to_string() == "tests_profile_changed" => {
                self.wake.notify_one();
                return Ok(Some(RoundDecision::Hold("tests_profile_changed")));
            }
            ctx => ctx?,
        };
        let result = async {
            if ctx.step.kind == "review" && ctx.step.body["filtering"].is_null() {
                ensure!(ctx.step.state == "exited" && ctx.step.body["round_result_validated"] == true, "round_filter_requires_exited");
                let config = self.config.pipeline.as_ref().context("pipeline config missing")?;
                let git = PipelineGit::new(&ctx.project.root)?;
                let path = review_snapshot::snapshot_path(&config.work_root, &ctx.run.id, ctx.step.seq, ctx.step.attempt)?;
                let tree = review_snapshot::verify(&git, &config.work_root, &path, &ctx.step.step_base).await?;
                ensure!(ctx.step.body["adoption"]["snapshot_tree"] == tree, "review_snapshot_changed");
                let verdict = serde_json::from_value(ctx.step.body["adoption"]["verdict"].clone())?;
                let listing = git.run(&["diff-tree", "--no-commit-id", "-r", "--no-renames", "--name-only", "-z", &ctx.run.base_commit, &ctx.run.head_commit, "--"]).await?;
                let changed = listing.split(|b| *b == 0).filter(|s| !s.is_empty()).map(|s| Ok(std::str::from_utf8(s)?.to_owned())).collect::<Result<BTreeSet<_>>>()?;
                let listing = git.run(&["ls-tree", "-r", "--name-only", "-z", &ctx.step.step_base, "--"]).await?;
                let reviewed = listing.split(|b| *b == 0).filter(|s| !s.is_empty()).map(|s| Ok(std::str::from_utf8(s)?.to_owned())).collect::<Result<BTreeSet<_>>>()?;
                let filtered = confirm::filter(&verdict, ctx.threshold, &path, &reviewed, &changed, &ctx.scopes)?;
                // Detect external snapshot drift during filtering as well as at adoption.
                review_snapshot::verify(&git, &config.work_root, &path, &ctx.step.step_base).await?;
                self.owner.store.lock().map_err(|_| anyhow::anyhow!("state poisoned"))?.pipeline_record_filter(&self.owner, &cursor, &filtered)?;
                cursor.step_version += 1;
                ctx = self.round_context(&cursor)?;
            }
            let action = round_action(&ctx)?;
            let next = match action {
                RoundDecision::Hold(_) => None,
                RoundDecision::Fix => {
                    let input = round_fix_input(&ctx)?;
                    let nonce = uuid::Uuid::new_v4().simple().to_string();
                    let prompt = rounds::fix_prompt(&nonce, &input)?;
                    let input_digest = crate::execution::verification::hash(&serde_json::to_vec(&input)?);
                    let mut step = next_step(&ctx, "fix", ctx.run.round + 1, json!({"fix_nonce":nonce,"fix_input":input,"fix_input_sha256":input_digest,"confirmed_ids":input.confirmed_ids()}));
                    step.input_sha256 = crate::execution::verification::hash(&prompt);
                    Some(step)
                }
                RoundDecision::Confirm => {
                    let (_, _, filtered) = round_source_review(&ctx)?;
                    let nonce = uuid::Uuid::new_v4().simple().to_string();
                    let diff = PipelineGit::new(&ctx.project.root)?.diff(&ctx.run.base_commit, &ctx.run.head_commit).await?;
                    let prompt = confirm::prompt(&nonce, &filtered.blocking, &diff)?;
                    let mut step = next_step(&ctx, "confirm", ctx.run.round, json!({"confirm_nonce":nonce,"review_seq":ctx.step.seq}));
                    step.input_sha256 = crate::execution::verification::hash(&prompt);
                    Some(step)
                }
                RoundDecision::Review => {
                    let nonce = uuid::Uuid::new_v4().simple().to_string();
                    let diff = PipelineGit::new(&ctx.project.root)?.diff(&ctx.run.base_commit, &ctx.run.head_commit).await?;
                    let prompt = review::review_prompt(&nonce, &ctx.task, &diff)?;
                    let mut step = next_step(&ctx, "review", ctx.run.round, json!({"review_nonce":nonce}));
                    step.input_sha256 = crate::execution::verification::hash(&prompt);
                    Some(step)
                }
                RoundDecision::Test(index) => Some(next_step(&ctx, "test", ctx.run.round, test_body(&ctx, index)?)),
                RoundDecision::Release => Some(next_step(&ctx, "release", ctx.run.round, json!({}))),
            };
            let action = self.owner.store.lock().map_err(|_| anyhow::anyhow!("state poisoned"))?.pipeline_apply_round(&self.owner, &cursor, next.as_ref())?;
            Ok::<_, anyhow::Error>(action)
        }.await;
        match result {
            Ok(action) => {
                self.wake.notify_one();
                Ok(Some(action))
            }
            Err(error) if error.downcast_ref::<pipeline::SecretHit>().is_some() => {
                self.round_failure(&cursor, &error, false)?;
                Ok(Some(RoundDecision::Hold("secret_detected")))
            }
            Err(error) if confirm::failure_reason(&error) != "confirm_validation_failed" => {
                let reason = confirm::failure_reason(&error);
                self.round_failure(&cursor, &error, false)?;
                Ok(Some(RoundDecision::Hold(reason)))
            }
            Err(error) => Err(error),
        }
    }

    async fn prepare_confirm_input(&self, cursor: &PipelineCursor) -> Result<(StepInput, String)> {
        let ctx = self.round_context(cursor)?;
        ensure!(
            ctx.step.kind == "confirm" && ctx.step.state == "planned",
            "confirm_prepare_state"
        );
        let (_, _, filtered) = round_source_review(&ctx)?;
        let config = self
            .config
            .pipeline
            .as_ref()
            .context("pipeline config missing")?;
        let git = PipelineGit::new(&ctx.project.root)?;
        let diff = git.diff(&ctx.run.base_commit, &ctx.step.step_base).await?;
        let nonce = ctx.step.body["confirm_nonce"]
            .as_str()
            .context("review_nonce_invalid")?;
        let prompt = confirm::prompt(nonce, &filtered.blocking, &diff)?;
        ensure!(
            crate::execution::verification::hash(&prompt) == ctx.step.input_sha256,
            "confirm_input_changed"
        );
        let path = review_snapshot::snapshot_path(
            &config.work_root,
            &ctx.run.id,
            ctx.step.seq,
            ctx.step.attempt,
        )?;
        if let Some(work) = &ctx.run.worktree {
            ensure!(
                !path.starts_with(work) && !std::path::Path::new(work).starts_with(&path),
                "review_snapshot_changed"
            );
        }
        let tree =
            review_snapshot::rebuild(&git, &config.work_root, &path, &ctx.step.step_base).await?;
        Ok((
            StepInput {
                work: path,
                prompt,
                schema: confirm::CONFIRM_SCHEMA.into(),
            },
            tree,
        ))
    }

    /// Confirmation always rebuilds its own sealed snapshot of the reviewed commit.
    pub async fn prepare_pipeline_confirm(
        &self,
        cursor: PipelineCursor,
    ) -> Result<Option<StepInput>> {
        let Ok(_boundary) = self.control_admission.try_lock() else {
            return Ok(None);
        };
        if !self.round_ready()? {
            return Ok(None);
        }
        self.round_context(&cursor)?;
        match self.prepare_confirm_input(&cursor).await {
            Ok((input, tree)) => {
                let recorded = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_confirm_preflight(&self.owner, &cursor, &tree);
                if let Err(error) = recorded {
                    self.round_failure(&cursor, &error, false)?;
                    return Ok(None);
                }
                Ok(Some(input))
            }
            Err(error) => {
                self.round_failure(&cursor, &error, false)?;
                Ok(None)
            }
        }
    }

    /// Use the fixed Confirmer argv through the same matching-cookie Agent custody.
    pub async fn launch_pipeline_confirm(
        &self,
        mut cursor: PipelineCursor,
    ) -> Result<LaunchResult> {
        let Ok(boundary) = self.control_admission.clone().try_lock_owned() else {
            return Ok(LaunchResult::Waiting);
        };
        if !self.round_ready()? {
            return Ok(LaunchResult::Waiting);
        }
        self.round_context(&cursor)?;
        let prepared = self.prepare_confirm_input(&cursor).await;
        let (input, tree) = match prepared {
            Ok(input) => input,
            Err(error) => {
                self.round_failure(&cursor, &error, false)?;
                return Ok(LaunchResult::Held);
            }
        };
        let recorded = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_confirm_preflight(&self.owner, &cursor, &tree);
        if let Err(error) = recorded {
            self.round_failure(&cursor, &error, false)?;
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

    /// Accept only strict labels and source-matching evidence, keeping exited until decision.
    pub async fn adopt_pipeline_confirm(&self, cursor: PipelineCursor) -> Result<bool> {
        let Ok(_boundary) = self.control_admission.try_lock() else {
            return Ok(false);
        };
        if !self.round_ready()? {
            return Ok(false);
        }
        let ctx = self.round_context(&cursor)?;
        ensure!(
            ctx.step.kind == "confirm" && ctx.step.state == "exited",
            "confirm_adoption_state"
        );
        let checked = async {
            let config = self
                .config
                .pipeline
                .as_ref()
                .context("pipeline config missing")?;
            let path = review_snapshot::snapshot_path(
                &config.work_root,
                &ctx.run.id,
                ctx.step.seq,
                ctx.step.attempt,
            )?;
            let git = PipelineGit::new(&ctx.project.root)?;
            let tree = review_snapshot::verify(&git, &config.work_root, &path, &ctx.step.step_base)
                .await?;
            ensure!(
                ctx.step.body["snapshot_tree"] == tree,
                "review_snapshot_changed"
            );
            let attempt = config
                .work_root
                .join("runs")
                .join(&ctx.run.id)
                .join(format!("{}-{}", ctx.step.seq, ctx.step.attempt));
            ensure!(
                ctx.step.body["intent"]["attempt_dir"].as_str() == attempt.to_str(),
                "confirm_output_changed"
            );
            let bytes = confirm::output(&ctx.step, &attempt)?;
            if ctx.step.body["provider_outcome"] == "limit" {
                return Ok(None);
            }
            ensure!(
                ctx.step.body["exit"]["code"] == 0
                    && ctx.step.body["provider_outcome"] == "success",
                "confirm_agent_failed"
            );
            let (_, _, filtered) = round_source_review(&ctx)?;
            let mut evidence = confirm::decode(&bytes, &filtered.blocking, &path)?;
            review_snapshot::verify(&git, &config.work_root, &path, &ctx.step.step_base).await?;
            evidence["snapshot_tree"] = json!(tree);
            let mut stored = ctx.step.body.clone();
            stored["adoption"] = evidence.clone();
            ensure!(step_body_fits(&stored)?, "confirm_storage_limit");
            Ok::<_, anyhow::Error>(Some(evidence))
        }
        .await;
        match checked {
            Ok(Some(evidence)) => {
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_stage_round_result(&self.owner, &cursor, &evidence)?;
                Ok(true)
            }
            Ok(None) => {
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_record_limited(&self.owner, &cursor)?;
                Ok(false)
            }
            Err(error) => {
                self.round_failure(&cursor, &error, true)?;
                Ok(false)
            }
        }
    }

    /// Fix callers cannot substitute text for the stored confirmed-only or test-summary input.
    pub async fn launch_pipeline_fix(&self, cursor: PipelineCursor) -> Result<LaunchResult> {
        // launch_pipeline_executor acquires the boundary and the common launcher rechecks this input.
        let ctx = self.round_context(&cursor)?;
        let input = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_fix_input(&self.owner, &cursor)?;
        let prompt = rounds::fix_prompt(
            ctx.step.body["fix_nonce"]
                .as_str()
                .context("review_nonce_invalid")?,
            &input,
        )?;
        self.launch_pipeline_executor(cursor, prompt).await
    }

    fn round_failure(
        &self,
        cursor: &PipelineCursor,
        error: &anyhow::Error,
        reject: bool,
    ) -> Result<()> {
        let mut store = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        store.pipeline_launch_snapshot(&self.owner, cursor)?;
        if let Some(hit) = error.downcast_ref::<pipeline::SecretHit>() {
            store.pipeline_hold_secret(
                &self.owner,
                cursor,
                &json!({"pattern":hit.pattern,"offset_sha256":hit.offset_sha256}),
            )?;
        } else {
            let reason = confirm::failure_reason(error);
            if reject
                && matches!(
                    reason,
                    "confirm_schema" | "confirm_label_ids" | "confirm_agent_failed"
                )
            {
                store.pipeline_reject(&self.owner, cursor, reason)?;
            } else {
                store.pipeline_hold(&self.owner, cursor, reason, None)?;
            }
        }
        self.wake.notify_one();
        Ok(())
    }
}

fn next_step(ctx: &RoundContext, kind: &str, round: u32, body: Value) -> PipelineStep {
    PipelineStep {
        run_id: ctx.run.id.clone(),
        seq: ctx.step.seq + 1,
        attempt: 1,
        kind: kind.into(),
        provider: match kind {
            "review" => Some("claude".into()),
            "confirm" | "fix" => Some("codex".into()),
            _ => None,
        },
        owner_epoch: ctx.run.owner_epoch,
        round,
        state: "planned".into(),
        version: 0,
        cookie: None,
        pid: None,
        birth: None,
        step_base: ctx.run.head_commit.clone(),
        input_sha256: crate::execution::verification::hash(b""),
        result_commit: None,
        result_tree: None,
        body,
    }
}
fn test_body(ctx: &RoundContext, index: usize) -> Result<Value> {
    let command = ctx
        .profile
        .proposal
        .commands
        .get(index)
        .context("round_tests_incomplete")?;
    Ok(json!({"command_index":index,"command_id":command.id}))
}
