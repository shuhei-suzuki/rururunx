//! Actual start-future/result custody; observations grant no Native authority.
use super::phase_supervisor::{PhaseLaunch, PhaseLaunchParts};
use crate::execution::{
    OperationId,
    native::{NativePhaseStart, NativePhaseStartError},
    phase::NativeAllocation,
};
use anyhow::{Result, ensure};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use tokio::{sync::watch, task::JoinHandle};

const MAX_JOBS: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InvocationObservation {
    Reserved,
    Starting,
    Launched,
    Waiting,
    Failed,
    Uncertain,
}

struct JobState {
    observation: InvocationObservation,
    launch: Option<Arc<PhaseLaunchParts>>,
    outcome: Option<std::result::Result<NativePhaseStart, NativePhaseStartError>>,
}
struct Job {
    allocation: Arc<NativeAllocation>,
    state: Mutex<JobState>,
    changed: watch::Sender<InvocationObservation>,
}
struct Entry {
    job: Arc<Job>,
    // Independent registry ownership, never a strong return edge from Job.
    handle: Option<JoinHandle<()>>,
}

/// Runtime-owned sibling of PhaseSupervisor. Neither slots nor jobs own it.
#[derive(Default)]
pub(super) struct PhaseJobs {
    entries: Mutex<BTreeMap<OperationId, Entry>>,
}

/// Observation only. Drop does not abort, release, retry or remove anything.
pub(crate) struct PhaseInvocation {
    changed: watch::Receiver<InvocationObservation>,
}
impl PhaseInvocation {
    pub(crate) fn observation(&self) -> InvocationObservation {
        *self.changed.borrow()
    }
    pub(crate) async fn wait(&mut self) -> InvocationObservation {
        loop {
            let observation = *self.changed.borrow_and_update();
            if !matches!(
                observation,
                InvocationObservation::Reserved | InvocationObservation::Starting
            ) {
                return observation;
            }
            if self.changed.changed().await.is_err() {
                return InvocationObservation::Uncertain;
            }
        }
    }
}

impl PhaseJobs {
    /// Caller already holds actual Runtime admission and publishing capacity.
    /// Called before SQL effects; duplicate IDs cannot substitute an allocation.
    pub(super) fn reserve(&self, allocation: &Arc<NativeAllocation>) -> Result<()> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
        let operation = allocation.facts().operation_id;
        if let Some(entry) = entries.get(&operation) {
            ensure!(
                Arc::ptr_eq(&entry.job.allocation, allocation),
                "foreign phase job allocation"
            );
            ensure!(
                entry.handle.is_none()
                    && entry
                        .job
                        .changed
                        .borrow()
                        .eq(&InvocationObservation::Reserved),
                "phase job already started"
            );
            return Ok(());
        }
        ensure!(entries.len() < MAX_JOBS, "phase job capacity unavailable");
        let (changed, _) = watch::channel(InvocationObservation::Reserved);
        entries.insert(
            operation,
            Entry {
                job: Arc::new(Job {
                    allocation: allocation.clone(),
                    state: Mutex::new(JobState {
                        observation: InvocationObservation::Reserved,
                        launch: None,
                        outcome: None,
                    }),
                    changed,
                }),
                handle: None,
            },
        );
        Ok(())
    }

    /// No start or launch flag mutation occurs unless the original reservation
    /// is still available. Its actual allocation, not the key, establishes origin.
    pub(super) fn ready(&self, allocation: &Arc<NativeAllocation>) -> Result<()> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
        let entry = entries
            .get(&allocation.facts().operation_id)
            .ok_or_else(|| anyhow::anyhow!("phase job not reserved before marker"))?;
        ensure!(
            Arc::ptr_eq(&entry.job.allocation, allocation)
                && entry.handle.is_none()
                && entry
                    .job
                    .changed
                    .borrow()
                    .eq(&InvocationObservation::Reserved),
            "original phase job changed"
        );
        Ok(())
    }

    /// Runtime admission serializes this with ready/rollback. After a known
    /// handoff, preserve custody even if a mutex was poisoned: recovery here
    /// only stores actual objects and never authorizes a protected Store write.
    pub(super) fn start(&self, launch: PhaseLaunch) -> PhaseInvocation {
        let parts = launch.parts().clone();
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        let entry = entries
            .get_mut(&parts.allocation().facts().operation_id)
            .expect("actual phase job reserved before marker handoff");
        let job = entry.job.clone();
        {
            let mut state = job.state.lock().unwrap_or_else(|e| e.into_inner());
            state.launch = Some(parts);
            state.observation = InvocationObservation::Starting;
        }
        job.changed.send_replace(InvocationObservation::Starting);
        let changed = job.changed.subscribe();
        entry.handle = Some(tokio::spawn(async move {
            let _running = RunningJob(job.clone());
            let outcome = job.allocation.selected_port().start_phase(launch).await;
            let observation = match &outcome {
                Ok(NativePhaseStart::Launched { .. }) => InvocationObservation::Launched,
                Ok(NativePhaseStart::Waiting { .. }) => InvocationObservation::Waiting,
                Err(_) => InvocationObservation::Failed,
            };
            {
                let mut state = job.state.lock().unwrap_or_else(|e| e.into_inner());
                state.outcome = Some(outcome);
                state.observation = observation;
            }
            job.changed.send_replace(observation);
        }));
        PhaseInvocation { changed }
    }

    /// Only a caller holding the genuine unpublished rollback uses this port.
    pub(super) fn remove_unstarted(&self, allocation: &Arc<NativeAllocation>) -> Result<()> {
        let removed = {
            let mut entries = self
                .entries
                .lock()
                .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
            let operation = allocation.facts().operation_id;
            if let Some(entry) = entries.get(&operation) {
                ensure!(
                    Arc::ptr_eq(&entry.job.allocation, allocation)
                        && entry.handle.is_none()
                        && entry
                            .job
                            .changed
                            .borrow()
                            .eq(&InvocationObservation::Reserved),
                    "started job cannot roll back"
                );
            }
            entries.remove(&operation)
        };
        drop(removed);
        Ok(())
    }

    pub(super) fn ensure_shutdown_complete(&self) -> Result<()> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
        ensure!(
            entries.is_empty(),
            "Native phase shutdown remains pending with retained jobs"
        );
        Ok(())
    }
}

struct RunningJob(Arc<Job>);
impl Drop for RunningJob {
    fn drop(&mut self) {
        let mut state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.outcome.is_none() {
            state.observation = InvocationObservation::Uncertain;
            self.0
                .changed
                .send_replace(InvocationObservation::Uncertain);
        }
    }
}
