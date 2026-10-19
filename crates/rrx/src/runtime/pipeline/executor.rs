//! Rule-G entry points; the scheduler selects steps, these methods perform one fenced action.
use crate::execution::pipeline::{
    AdoptionChecker, LaunchResult, PipelineGit, SecretHit, SecretScanner, StepInput, StepLauncher,
    TaskWorkspace, adoption, workspace::ControlImage,
};
use crate::{
    config::ExecutionMode,
    runtime::Runtime,
    state::pipeline::{PipelineCursor, PipelineLaunch, PipelineRun, PipelineStep},
};
use anyhow::{Context, Result, ensure};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

impl Runtime {
    fn executor_ready(&self) -> Result<bool> {
        ensure!(
            self.config.execution.mode == ExecutionMode::CliPipeline,
            "ExecutionModeMismatch"
        );
        Ok(self.service_running() && !self.stopping.load(Ordering::SeqCst))
    }
    fn executor_snapshot(&self, cursor: &PipelineCursor) -> Result<(PipelineRun, PipelineStep)> {
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_launch_snapshot(&self.owner, cursor)
    }
    fn task_workspace(
        &self,
        run: &PipelineRun,
        step: &PipelineStep,
        project: &Path,
    ) -> Result<TaskWorkspace> {
        TaskWorkspace::bound(
            project,
            &self
                .config
                .pipeline
                .as_ref()
                .context("pipeline config missing")?
                .work_root,
            Path::new(run.worktree.as_deref().context("Task worktree missing")?),
            run.branch.as_deref().context("Task branch missing")?,
            &step.step_base,
        )
    }
    fn executor_attempt(&self, step: &PipelineStep) -> Result<PathBuf> {
        ensure!(
            !step.run_id.is_empty()
                && step
                    .run_id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b)),
            "invalid run component"
        );
        let expected = self
            .config
            .pipeline
            .as_ref()
            .context("pipeline config missing")?
            .work_root
            .join("runs")
            .join(&step.run_id)
            .join(format!("{}-{}", step.seq, step.attempt));
        ensure!(
            step.body["intent"]["attempt_dir"].as_str() == expected.to_str(),
            "output_evidence_changed"
        );
        Ok(expected)
    }
    /// The Executor consumes operator text and policy scopes; no arbitrary executable or argv.
    pub async fn launch_pipeline_executor(
        &self,
        mut cursor: PipelineCursor,
        prompt: Vec<u8>,
    ) -> Result<LaunchResult> {
        let Ok(boundary) = self.control_admission.clone().try_lock_owned() else {
            return Ok(LaunchResult::Waiting);
        };
        if !self.executor_ready()? {
            return Ok(LaunchResult::Waiting);
        }
        let (run, step) = self.executor_snapshot(&cursor)?;
        ensure!(
            step.state == "planned" && matches!(step.kind.as_str(), "implement" | "fix"),
            "Executor launch state changed"
        );
        let preflight = async {
            SecretScanner::check(&prompt).context("prompt.md")?;
            let (project, _, controls) = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_executor_context(&self.owner, &cursor)?;
            let work = self.task_workspace(&run, &step, &project.root)?;
            work.git.reject_drivers(&step.step_base).await?;
            work.clean_at(&step.step_base).await?;
            let actual = work.controls().await?;
            let expected: ControlImage =
                serde_json::from_value(controls.context("git_control_changed")?)?;
            ensure!(actual == expected, "git_control_changed");
            Ok::<_, anyhow::Error>(serde_json::to_value(actual)?)
        }
        .await;
        let controls = match preflight {
            Ok(controls) => controls,
            Err(error) => {
                let mut store = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?;
                if let Some(evidence) = secret_evidence(&error) {
                    store.pipeline_hold_secret(&self.owner, &cursor, &evidence)?;
                } else {
                    store.pipeline_hold(&self.owner, &cursor, preflight_reason(&error), None)?;
                }
                return Ok(LaunchResult::Held);
            }
        };
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_executor_preflight(&self.owner, &cursor, &controls)?;
        cursor.step_version += 1;
        StepLauncher::new(
            self.owner.clone(),
            self.config
                .pipeline
                .clone()
                .context("pipeline config missing")?,
            self.wake.clone(),
        )
        .launch(
            boundary,
            cursor,
            StepInput {
                work: run.worktree.context("Task worktree missing")?.into(),
                prompt,
                schema: String::new(),
            },
        )
        .await
    }

    /// Records SHA/tree before ref-CAS; resumed committed steps use only that recorded identity.
    pub async fn adopt_pipeline_executor(&self, mut cursor: PipelineCursor) -> Result<bool> {
        let Ok(_boundary) = self.control_admission.try_lock() else {
            return Ok(false);
        };
        if !self.executor_ready()? {
            return Ok(false);
        }
        let (run, mut step) = self.executor_snapshot(&cursor)?;
        ensure!(
            matches!(step.kind.as_str(), "implement" | "fix")
                && matches!(step.state.as_str(), "exited" | "committed"),
            "Executor adoption state changed"
        );
        if step.state == "exited" && step.body["provider_outcome"] == "limit" {
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_record_limited(&self.owner, &cursor)?;
            self.wake.notify_one();
            return Ok(false);
        }
        let result = async {
            let (project, scope, _) = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_executor_context(&self.owner, &cursor)?;
            let work = self.task_workspace(&run, &step, &project.root)?;
            let attempt = self.executor_attempt(&step)?;
            adoption::evidence(&step, &attempt)?;
            if step.state == "exited" {
                let candidate = AdoptionChecker::build(&work, &step, &scope, &attempt).await?;
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_record_commit(
                        &self.owner,
                        &cursor,
                        &candidate.commit,
                        &candidate.tree,
                        &candidate.evidence,
                    )
                    .context("pipeline_progress_write_failed")?;
                cursor.step_version += 1;
                step = self
                    .executor_snapshot(&cursor)
                    .context("pipeline_progress_write_failed")?
                    .1;
            }
            adoption::diff_evidence(&step, &attempt)?;
            AdoptionChecker::finish(&work, &step).await?;
            // Re-read retained output even when resuming after the ref update.
            adoption::evidence(&step, &attempt)?;
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_adopt(
                    &self.owner,
                    &cursor,
                    &json!({"commit":step.result_commit,"tree":step.result_tree}),
                )
                .context("pipeline_progress_write_failed")?;
            Ok::<_, anyhow::Error>(())
        }
        .await;
        match result {
            Ok(()) => {
                self.wake.notify_one();
                Ok(true)
            }
            Err(error) => {
                if error.to_string() == "pipeline_progress_write_failed" {
                    return Err(error);
                }
                if let Some(evidence) = secret_evidence(&error) {
                    self.owner
                        .store
                        .lock()
                        .map_err(|_| anyhow::anyhow!("state poisoned"))?
                        .pipeline_hold_secret(&self.owner, &cursor, &evidence)?;
                    return Ok(false);
                }
                // Never copy tool output or a matching secret into the database.
                let reason = error
                    .chain()
                    .map(ToString::to_string)
                    .find(|s| {
                        [
                            "agent_commit",
                            "scope_violation",
                            "protected_path",
                            "no_change",
                            "agent_failed",
                            "secret_detected",
                            "git_control_changed",
                            "git_driver_declared",
                            "ref_moved",
                            "output_evidence_changed",
                            "diff_too_large",
                        ]
                        .contains(&s.as_str())
                    })
                    .unwrap_or_else(|| "executor_validation_failed".into());
                let mut store = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?;
                if step.state == "exited"
                    && [
                        "agent_commit",
                        "scope_violation",
                        "protected_path",
                        "no_change",
                        "agent_failed",
                    ]
                    .contains(&reason.as_str())
                {
                    store.pipeline_reject(&self.owner, &cursor, &reason)?;
                } else {
                    store.pipeline_hold(&self.owner, &cursor, &reason, None)?;
                }
                Ok(false)
            }
        }
    }

    /// Resets only the bound worktree after certified death, then verifies the saved controls.
    pub async fn initialize_pipeline_executor(&self, cursor: &PipelineCursor) -> Result<bool> {
        let Ok(_boundary) = self.control_admission.try_lock() else {
            return Ok(false);
        };
        if !self.executor_ready()? {
            return Ok(false);
        }
        let (run, step) = self.executor_snapshot(cursor)?;
        ensure!(
            matches!(step.kind.as_str(), "implement" | "fix")
                && matches!(step.state.as_str(), "interrupted" | "limited" | "abandoned")
                && step.body["death_confirmed"] == true,
            "workspace death unconfirmed"
        );
        let (project, _, _) = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_executor_context(&self.owner, cursor)?;
        let work = self.task_workspace(&run, &step, &project.root)?;
        let controls = serde_json::from_value(step.body["executor_controls"].clone())?;
        work.initialize(&controls).await?;
        Ok(true)
    }

    /// Materializes a new prepare once; recovery accepts complete state and holds partial state.
    pub async fn prepare_pipeline_task(&self, mut cursor: PipelineCursor) -> Result<bool> {
        let Ok(_boundary) = self.control_admission.try_lock() else {
            return Ok(false);
        };
        if !self.executor_ready()? {
            return Ok(false);
        }
        let (run, step) = self.executor_snapshot(&cursor)?;
        ensure!(
            step.kind == "prepare" && matches!(step.state.as_str(), "planned" | "running"),
            "prepare state changed"
        );
        let project = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .project(run.project_id)?
            .context("Project missing")?;
        let git = PipelineGit::new(&project.root)?;
        let work_path = Path::new(run.worktree.as_deref().context("Task worktree missing")?);
        let branch = run.branch.as_deref().context("Task branch missing")?;
        let root = &self
            .config
            .pipeline
            .as_ref()
            .context("pipeline config missing")?
            .work_root;
        let fresh = step.state == "planned";
        if fresh {
            let started = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_begin_launch(
                    &self.owner,
                    &cursor,
                    None,
                    None,
                    &json!({"worktree":work_path,"branch":branch}),
                )?;
            if started != PipelineLaunch::Started {
                return Ok(false);
            }
            cursor.step_version += 1;
        }
        let result = async {
            ensure!(
                work_path.starts_with(root)
                    && work_path != root
                    && work_path
                        .parent()
                        .context("work parent missing")?
                        .canonicalize()?
                        == work_path.parent().context("work parent missing")?,
                "materialization_partial"
            );
            ensure!(
                branch.starts_with("rrx/")
                    && branch.split('/').all(|part| !part.is_empty()
                        && part
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))),
                "materialization_partial"
            );
            git.reject_tree_drivers(&run.base_commit).await?;
            let journals = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_goal_view(run.goal_id)?
                .journals;
            let create = journals
                .iter()
                .find(|j| j.kind == "goal_create")
                .context("Goal journal missing")?;
            let goal_ref = format!("refs/heads/rrx/goal/{}", run.goal_id);
            ensure!(
                create.ref_name == goal_ref && create.project_id == run.project_id,
                "Goal journal mismatch"
            );
            let refs = git
                .text(&[
                    "for-each-ref",
                    "--format=%(refname) %(objectname)",
                    "refs/heads/",
                ])
                .await?;
            let find = |name: &str| {
                refs.lines().find_map(|l| {
                    l.split_once(' ')
                        .filter(|(r, _)| *r == name)
                        .map(|(_, sha)| sha.to_owned())
                })
            };
            let observed = find(&goal_ref);
            let task_ref = format!("refs/heads/{branch}");
            git.require_direct_ref(&goal_ref).await?;
            git.require_direct_ref(&task_ref).await?;
            let registered = git.run(&["worktree", "list", "--porcelain", "-z"]).await?;
            let present = registered
                .split(|b| *b == 0)
                .any(|field| field == format!("worktree {}", work_path.display()).as_bytes());
            let absent = !work_path.try_exists()? && find(&task_ref).is_none() && !present;
            ensure!(!fresh || absent, "materialization_partial");
            if !fresh && absent {
                anyhow::bail!("prepare_absent");
            }
            if create.state == "pending" {
                ensure!(
                    create.body["creator_run_id"] == run.id && create.new_commit == run.base_commit,
                    "Goal journal mismatch"
                );
                if observed.is_none() {
                    ensure!(fresh, "materialization_partial");
                    let listing = git.run(&["worktree", "list", "--porcelain", "-z"]).await?;
                    ensure!(
                        !listing
                            .split(|b| *b == 0)
                            .any(|field| field == format!("branch {goal_ref}").as_bytes()),
                        "Goal branch checked out"
                    );
                    git.run(&[
                        "update-ref",
                        "--no-deref",
                        &goal_ref,
                        &create.new_commit,
                        &create.old_commit,
                    ])
                    .await?;
                } else {
                    ensure!(observed.as_deref() == Some(&create.new_commit), "ref_moved");
                }
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_release_settle(
                        &self.owner,
                        &cursor,
                        &create.id,
                        create.version,
                        &create.new_commit,
                    )
                    .context("pipeline_progress_write_failed")?;
            } else {
                ensure!(
                    create.state == "done" && observed.as_deref() == Some(&run.base_commit),
                    "ref_moved"
                );
            }
            if absent {
                ensure!(fresh, "prepare_absent");
                git.run(&[
                    "worktree",
                    "add",
                    "--no-checkout",
                    "-b",
                    branch,
                    work_path.to_str().context("work path encoding")?,
                    &run.base_commit,
                ])
                .await?;
            }
            let work = self.task_workspace(&run, &step, &project.root)?;
            work.git.reject_drivers(&step.step_base).await?;
            if absent {
                work.git.run(&["reset", "--hard", &run.base_commit]).await?;
            }
            work.clean_at(&run.base_commit).await?;
            let controls = work.controls().await?;
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_adopt(&self.owner, &cursor, &json!({"controls":controls}))
                .context("pipeline_progress_write_failed")?;
            Ok::<_, anyhow::Error>(())
        }
        .await;
        if let Err(error) = result {
            if error.to_string() == "pipeline_progress_write_failed" {
                return Err(error);
            }
            if error.to_string() == "prepare_absent" {
                self.owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .pipeline_prepare_absent(&self.owner, &cursor)?;
                return Ok(false);
            }
            let reason = if error.to_string() == "git_driver_declared" {
                "git_driver_declared"
            } else {
                "materialization_partial"
            };
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .pipeline_hold(&self.owner, &cursor, reason, None)?;
            return Ok(false);
        }
        self.wake.notify_one();
        Ok(true)
    }
}
fn preflight_reason(error: &anyhow::Error) -> &'static str {
    const REASONS: &[&str] = &[
        "task_scope_unresolved",
        "git_control_changed",
        "git_driver_declared",
        "symbolic_pipeline_ref",
        "agent_commit",
        "worktree_drift",
        "Git timeout",
    ];
    error
        .chain()
        .find_map(|cause| {
            REASONS
                .iter()
                .copied()
                .find(|reason| cause.to_string() == *reason)
        })
        .unwrap_or("executor_preflight_failed")
}

fn secret_evidence(error: &anyhow::Error) -> Option<serde_json::Value> {
    let hit = error.downcast_ref::<SecretHit>()?;
    let file = error
        .chain()
        .map(ToString::to_string)
        .find(|s| {
            [
                "prompt.md",
                "diff.patch",
                "stdout.jsonl",
                "stderr.log",
                "last.md",
            ]
            .contains(&s.as_str())
        })
        .unwrap_or_else(|| "output".into());
    Some(json!({"pattern":hit.pattern,"file":file,"offset_sha256":hit.offset_sha256}))
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Preflight retains fixed actionable reasons and never persists arbitrary error text.
    #[test]
    fn b5_f1_preflight_reasons() {
        for reason in [
            "task_scope_unresolved",
            "git_control_changed",
            "git_driver_declared",
            "symbolic_pipeline_ref",
            "agent_commit",
            "worktree_drift",
            "Git timeout",
        ] {
            let error = anyhow::anyhow!(reason).context("untrusted wrapper");
            assert_eq!(preflight_reason(&error), reason);
        }
        assert_eq!(
            preflight_reason(&anyhow::anyhow!("untrusted details")),
            "executor_preflight_failed"
        );
    }
}
