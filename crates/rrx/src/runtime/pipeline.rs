//! Pipeline entry points share the control service's actual admission boundary.
use super::Runtime;
use crate::{
    config::ExecutionMode,
    execution::pipeline::{LaunchResult, StepInput, StepLauncher},
    state::pipeline::PipelineCursor,
};
use anyhow::{Context, Result, ensure};
use std::sync::atomic::Ordering;

impl Runtime {
    /// One launch attempt; scheduling and process cancellation policy live upstream.
    pub async fn launch_pipeline_step(
        &self,
        cursor: PipelineCursor,
        input: StepInput,
    ) -> Result<LaunchResult> {
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

    /// Rule G: progress is serialized with controls and revalidated in the Store tx.
    pub async fn record_pipeline_limit(&self, cursor: &PipelineCursor) -> Result<bool> {
        let Ok(_boundary) = self.control_admission.try_lock() else {
            return Ok(false);
        };
        if !self.service_running() || self.stopping.load(Ordering::SeqCst) {
            return Ok(false);
        }
        ensure!(
            self.config.execution.mode == ExecutionMode::CliPipeline,
            "ExecutionModeMismatch"
        );
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .pipeline_record_limited(&self.owner, cursor)?;
        Ok(true)
    }
}
