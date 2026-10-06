//! Actual start-future/result custody; observations grant no Native authority.
use super::phase_supervisor::{PhaseLaunch, PhaseLaunchParts};
use crate::execution::{
    OperationId,
    native::{
        ManagedSessionRef, NativePhaseBinding, NativePhaseStart, NativePhaseStartError,
        NativePreparationCustody,
    },
    phase::NativeAllocation,
};
use crate::state::managed_binding::{ManagedBindingPlan, plan_managed_binding};
use anyhow::{Result, ensure};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, Weak},
};
use tokio::{sync::watch, task::JoinHandle};

const MAX_JOBS: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InvocationObservation {
    Reserved,
    Starting,
    Binding,
    Bound,
    BindingHeld,
    Waiting,
    Failed,
    Uncertain,
}

struct JobState {
    // EMPTY/nongrant until the SAME actual selected Native start installs its
    // own actor and original plan. Retained BEFORE marker/start/future effects.
    preparation: Arc<NativePreparationCustody>,
    observation: InvocationObservation,
    launch: Option<Arc<PhaseLaunchParts>>,
    outcome: Option<std::result::Result<RetainedStart, NativePhaseStartError>>,
    binding_plan: Option<Arc<ManagedBindingPlan>>,
    binding_error: Option<anyhow::Error>,
}
/// Actual returned objects, never reconstructed from DTOs or registry IDs.
enum RetainedStart {
    Launched {
        _handle: ManagedSessionRef,
        binding: Arc<NativePhaseBinding>,
    },
    Waiting(Box<NativePhaseStart>),
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

/// Produced by the actual reservation, retaining SAME entry identity without
/// a job -> launch -> reservation -> job ownership cycle. Reuse never grants
/// this call permission to remove somebody else's existing reservation.
pub(super) struct PhaseJobReservation {
    job: Weak<Job>,
    allocation: Arc<NativeAllocation>,
    fresh: bool,
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
                InvocationObservation::Reserved
                    | InvocationObservation::Starting
                    | InvocationObservation::Binding
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
    pub(super) fn reserve(
        &self,
        allocation: &Arc<NativeAllocation>,
    ) -> Result<PhaseJobReservation> {
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
            return Ok(PhaseJobReservation {
                job: Arc::downgrade(&entry.job),
                allocation: allocation.clone(),
                fresh: false,
            });
        }
        ensure!(entries.len() < MAX_JOBS, "phase job capacity unavailable");
        let (changed, _) = watch::channel(InvocationObservation::Reserved);
        let job = Arc::new(Job {
            allocation: allocation.clone(),
            state: Mutex::new(JobState {
                preparation: NativePreparationCustody::new(allocation.clone()),
                observation: InvocationObservation::Reserved,
                launch: None,
                outcome: None,
                binding_plan: None,
                binding_error: None,
            }),
            changed,
        });
        entries.insert(
            operation,
            Entry {
                job: job.clone(),
                handle: None,
            },
        );
        Ok(PhaseJobReservation {
            job: Arc::downgrade(&job),
            allocation: allocation.clone(),
            fresh: true,
        })
    }

    /// No start or launch flag mutation occurs unless the original reservation
    /// is still available. Its actual allocation, not the key, establishes origin.
    pub(super) fn ready(&self, reservation: &PhaseJobReservation) -> Result<()> {
        let allocation = &reservation.allocation;
        let job = reservation
            .job
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("original reserved job ended"))?;
        let entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
        let entry = entries
            .get(&allocation.facts().operation_id)
            .ok_or_else(|| anyhow::anyhow!("phase job not reserved before marker"))?;
        ensure!(
            Arc::ptr_eq(&entry.job, &job)
                && Arc::ptr_eq(&entry.job.allocation, allocation)
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

    /// Root's SAME original reserved job, never a new cell from readable rows.
    /// Clone the actual retained object before queue/Store admission. Launch
    /// stores only its Weak identity, avoiding custody -> actor -> launch cycles.
    pub(super) fn preparation_custody(
        &self,
        reservation: &PhaseJobReservation,
    ) -> Result<Arc<NativePreparationCustody>> {
        let allocation = &reservation.allocation;
        let job = reservation
            .job
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("original reserved job ended"))?;
        let entries = self
            .entries
            .lock()
            .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
        let entry = entries
            .get(&allocation.facts().operation_id)
            .ok_or_else(|| anyhow::anyhow!("actual preparation job not reserved"))?;
        ensure!(
            Arc::ptr_eq(&entry.job, &job)
                && Arc::ptr_eq(&entry.job.allocation, allocation)
                && entry.handle.is_none(),
            "original preparation job changed"
        );
        let state = entry
            .job
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("phase job state poisoned"))?;
        ensure!(
            state.observation == InvocationObservation::Reserved
                && state.preparation.matches_allocation(allocation),
            "original preparation custody differs"
        );
        Ok(state.preparation.clone())
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
        let preparation = {
            let mut state = job.state.lock().unwrap_or_else(|e| e.into_inner());
            state.launch = Some(parts);
            state.observation = InvocationObservation::Starting;
            state.preparation.clone()
        };
        job.changed.send_replace(InvocationObservation::Starting);
        let changed = job.changed.subscribe();
        // Capture an already-constructed guard: an unpolled future can be
        // destroyed when its Tokio executor stops, before its body ever runs.
        let running = RunningJob(job.clone());
        entry.handle = Some(tokio::spawn(async move {
            let _running = running;
            let outcome = job
                .allocation
                .selected_port()
                .start_phase(launch, preparation.clone())
                .await
                .map(|start| match start {
                    NativePhaseStart::Launched { handle, binding } => RetainedStart::Launched {
                        _handle: handle,
                        binding: Arc::from(binding),
                    },
                    waiting @ NativePhaseStart::Waiting { .. } => {
                        RetainedStart::Waiting(Box::new(waiting))
                    }
                });
            let refused = outcome.is_err();
            let (observation, binding) = match &outcome {
                Ok(RetainedStart::Launched { binding, .. }) => {
                    (InvocationObservation::Binding, Some(binding.clone()))
                }
                Ok(RetainedStart::Waiting(_)) => (InvocationObservation::Waiting, None),
                Err(_) => (InvocationObservation::Failed, None),
            };
            {
                let mut state = job.state.lock().unwrap_or_else(|e| e.into_inner());
                state.outcome = Some(outcome);
                state.observation = observation;
            }
            if refused {
                // Actual outcome is already retained. Nongrant abandonment must
                // not hold the Root job mutex or retire Unit/Task from an error.
                preparation.abandon();
            }
            job.changed.send_replace(observation);
            if let Some(binding) = binding {
                // Both the actual proof and handle are already retained. No
                // fallible planning or Store access can consume their sole owner.
                let result = job.bind_returned(binding);
                let observation = if result.is_ok() {
                    InvocationObservation::Bound
                } else {
                    InvocationObservation::BindingHeld
                };
                {
                    let mut state = job.state.lock().unwrap_or_else(|e| e.into_inner());
                    state.binding_error = result.err();
                    state.observation = observation;
                }
                job.changed.send_replace(observation);
            }
        }));
        PhaseInvocation { changed }
    }

    /// Only a caller holding the genuine unpublished rollback uses this port.
    pub(super) fn remove_unstarted(&self, reservation: &PhaseJobReservation) -> Result<bool> {
        if !reservation.fresh {
            return Ok(false);
        }
        let allocation = &reservation.allocation;
        let removed = {
            let mut entries = self
                .entries
                .lock()
                .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?;
            let operation = allocation.facts().operation_id;
            let job = reservation
                .job
                .upgrade()
                .ok_or_else(|| anyhow::anyhow!("original reserved job ended"))?;
            let entry = entries
                .get(&operation)
                .ok_or_else(|| anyhow::anyhow!("original reserved job removed"))?;
            ensure!(
                Arc::ptr_eq(&entry.job, &job)
                    && Arc::ptr_eq(&entry.job.allocation, allocation)
                    && entry.handle.is_none()
                    && entry
                        .job
                        .changed
                        .borrow()
                        .eq(&InvocationObservation::Reserved),
                "started job cannot roll back"
            );
            entries.remove(&operation)
        };
        drop(removed);
        Ok(true)
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

impl Job {
    fn bind_returned(&self, proof: Arc<NativePhaseBinding>) -> Result<()> {
        let owner = self.allocation.selected_port().owner();
        let plan = Arc::new(plan_managed_binding(owner, proof)?);
        {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            ensure!(
                state.binding_plan.is_none(),
                "binding plan already retained"
            );
            state.binding_plan = Some(plan.clone());
        }
        // Short job locks above never overlap the selected owner's Store lock.
        owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("binding Store poisoned"))?
            .bind_managed_phase(&plan)?;
        Ok(())
    }
}

struct RunningJob(Arc<Job>);
impl Drop for RunningJob {
    fn drop(&mut self) {
        let preparation = {
            let mut state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
            if state.outcome.is_none() || state.observation == InvocationObservation::Binding {
                state.observation = InvocationObservation::Uncertain;
                self.0
                    .changed
                    .send_replace(InvocationObservation::Uncertain);
                Some(state.preparation.clone())
            } else {
                None
            }
        };
        if let Some(preparation) = preparation {
            preparation.abandon();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        adapter::{AgentRegistry, InputKind, PreparedInput},
        config::{AgentConfig, Config},
        execution::{attempts::AttemptManager, native::ManagedInput},
    };

    /// Actual legacy Unit/allocation and EMPTY jobs only. This does not create
    /// accepted Source, marker, prepared Native input or an installed issuer.
    #[tokio::test]
    async fn actual_empty_job_reservation_removes_only_its_fresh_original() {
        let (_dir, owner, task) = crate::execution::results::tests::fixture().await;
        let (unit, _) = AttemptManager::new(owner.clone())
            .prepare(task.id, "codex", "Implement", None)
            .await
            .unwrap();
        let mut config = Config::default();
        config.agents.insert(
            "codex".into(),
            AgentConfig {
                provider: Some("codex".into()),
                command: vec!["/usr/bin/false".into()],
                ..Default::default()
            },
        );
        let agents = AgentRegistry::from_managed_config(&config, owner.clone()).unwrap();
        let allocation = Arc::new(
            agents
                .native_phase_port("codex")
                .unwrap()
                .allocate(
                    ManagedInput {
                        agent: "codex".into(),
                        authority: unit.authority(),
                        artifact: None,
                        input: PreparedInput {
                            scope: unit.scope.clone(),
                            kind: InputKind::ContextPack,
                            revision: unit.base_sha.clone(),
                            version: 1,
                            source_versions: Default::default(),
                            payload: "nongrant EMPTY job".into(),
                        },
                    },
                    None,
                    None,
                )
                .unwrap(),
        );
        let jobs = PhaseJobs::default();
        let fresh = jobs.reserve(&allocation).unwrap();
        let reused = jobs.reserve(&allocation).unwrap();
        let original = fresh.job.upgrade().unwrap();
        assert!(Arc::ptr_eq(&original, &reused.job.upgrade().unwrap()));
        assert!(
            !jobs.remove_unstarted(&reused).unwrap(),
            "reuse removed original job"
        );
        jobs.ready(&fresh).unwrap();
        assert!(Arc::ptr_eq(&original, &fresh.job.upgrade().unwrap()));
        let foreign_jobs = PhaseJobs::default();
        let foreign = foreign_jobs.reserve(&allocation).unwrap();
        assert!(
            jobs.remove_unstarted(&foreign).is_err(),
            "foreign entry token removed same-ID job"
        );
        jobs.ready(&fresh).unwrap();
        assert!(jobs.remove_unstarted(&fresh).unwrap());
        assert!(jobs.ready(&reused).is_err());
        jobs.ensure_shutdown_complete().unwrap();
        assert!(foreign_jobs.remove_unstarted(&foreign).unwrap());
        let current = owner.store.lock().unwrap().execution_unit(unit.id).unwrap();
        assert_eq!(
            serde_json::to_value(current).unwrap(),
            serde_json::to_value(unit).unwrap()
        );
    }
}
