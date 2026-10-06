//! Synchronous finite-pass admission. Keys/reasons are never grants.
use super::{
    Runtime,
    task_driver::{AdmitOutcome, SkipReason},
};
use crate::state::CandidatePage;
use anyhow::Result;
use std::sync::Arc;
impl Runtime {
    /// SAME production evaluation used by the finite sweep. Its result is a
    /// nongrant stage observation; it never makes row content into authority.
    pub(super) fn evaluate_driver_candidate(
        self: &Arc<Self>,
        admission: &tokio::sync::MutexGuard<'_, ()>,
        key: &crate::state::CandidateKey,
    ) -> Result<AdmitOutcome> {
        let current = match self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .current_task_bounded(key)
        {
            Ok(task) => task,
            Err(_) => return Ok(AdmitOutcome::Skipped(SkipReason::CurrentTaskBounded)),
        };
        let composition = match self.installed_driver_composition(&current) {
            Ok(composition) => composition,
            Err(_) => return Ok(AdmitOutcome::Skipped(SkipReason::Composition)),
        };
        self.admit_task_driver(admission, key, current.id, composition)
    }
    pub(super) fn admit_ready_tasks(self: &Arc<Self>) -> Result<usize> {
        if self.installed.is_err() {
            return Ok(0);
        }
        let Ok(admission) = self.control_admission.try_lock() else {
            return Ok(0);
        };
        if !self.service_running() {
            return Ok(0);
        }
        let mut stored = self
            .admission_cursor
            .lock()
            .map_err(|_| anyhow::anyhow!("admission cursor poisoned"))?;
        let mut cursor = stored.clone();
        let result = (|| {
            let mut evaluations = 0;
            let mut claimed = 0;
            loop {
                let page = self
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .ready_driver_candidates(
                        self.owner.instance_id(),
                        self.owner.epoch(),
                        cursor.as_ref(),
                        self.config.scheduler.global_max_sessions,
                        self.config.scheduler.max_tasks_per_project,
                    )?;
                let CandidatePage::Rows { keys, more } = page else {
                    return Ok(claimed);
                };
                if keys.is_empty() {
                    cursor = None;
                    return Ok(claimed);
                }
                for key in keys {
                    evaluations += 1;
                    cursor = Some(key.clone());
                    let outcome = self.evaluate_driver_candidate(&admission, &key);
                    match outcome {
                        Ok(AdmitOutcome::Claimed) => {
                            claimed += 1;
                            break;
                        } // discard pre-claim page
                        Ok(AdmitOutcome::Skipped(_reason)) => {}
                        Err(error) => return Err(error),
                    }
                    if evaluations >= 32 {
                        return Ok(claimed);
                    }
                }
                if evaluations >= 32 {
                    return Ok(claimed);
                }
                // Even the end of a short page is followed by a current read;
                // only an empty read resets the finite pass, never an in-pass wrap.
                let _ = more;
            }
        })();
        *stored = cursor;
        result
    }
}
