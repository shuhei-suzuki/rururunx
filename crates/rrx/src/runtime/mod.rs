//! Trusted controls and one installed graph for the original first Executor.
//! Continuation and official Native qualification remain unavailable.
mod admission;
pub mod control;
pub(crate) mod driver;
pub mod goal;
pub(crate) mod installation;
pub(crate) mod phase_effect_admission;
mod phase_handoffs;
mod phase_jobs;
#[cfg(test)]
pub(crate) use phase_jobs::{
    OWNER_IMMEDIATE, OWNER_PLANNING, SETTLED_CLOSURE, SETTLED_EVALUATION, count, pause_at,
};
pub(crate) mod phase_supervisor;
mod service;
mod task_driver;
use crate::{cli::transport::ServiceIdentity, config::Config, execution::RuntimeOwner};
use anyhow::Result;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub struct Runtime {
    owner: Arc<RuntimeOwner>,
    config: Config,
    _drivers: Arc<driver::DriverRegistry>,
    phases: Arc<phase_supervisor::PhaseSupervisor>,
    phase_jobs: Arc<phase_jobs::PhaseJobs>,
    phase_dispatcher: Arc<phase_supervisor::PhaseDispatcher>,
    phase_handoffs: Arc<phase_handoffs::PhaseHandoffs>,
    installed:
        std::result::Result<installation::InstalledNativeGraph, installation::InstallationRefusal>,
    admission_cursor: std::sync::Mutex<Option<crate::state::CandidateKey>>,
    started: AtomicBool,
    running: Arc<AtomicBool>,
    stopping: Arc<AtomicBool>,
    wake: Arc<tokio::sync::Notify>,
    control_admission: Arc<tokio::sync::Mutex<()>>,
    #[cfg(test)]
    goal_admission_pause: std::sync::Mutex<Option<GoalAdmissionPause>>,
    #[cfg(test)]
    before_service_reconcile: std::sync::Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
    supervisor: tokio::sync::Mutex<Option<tokio::task::JoinHandle<Result<()>>>>,
}
impl Runtime {
    /// Retains the existing owner, never opens another epoch or starts an Agent.
    pub fn new(owner: Arc<RuntimeOwner>, config: Config) -> Result<Self> {
        config.validate()?;
        let drivers = driver::DriverRegistry::new(&owner);
        owner.attach_runtime_drivers(&drivers)?;
        let running = Arc::new(AtomicBool::new(false));
        let stopping = Arc::new(AtomicBool::new(false));
        let control_admission = Arc::new(tokio::sync::Mutex::new(()));
        let admission = phase_effect_admission::PhaseEffectAdmission::new(
            owner.clone(),
            control_admission.clone(),
            running.clone(),
            stopping.clone(),
        );
        let phases = phase_supervisor::PhaseSupervisor::new(
            owner.clone(),
            config.scheduler.global_max_sessions,
            config.scheduler.max_tasks_per_project,
            admission,
        );
        let phase_jobs = Arc::new(phase_jobs::PhaseJobs::default());
        let phase_dispatcher = phase_supervisor::PhaseDispatcher::new(
            owner.clone(),
            phases.clone(),
            phase_jobs.clone(),
            control_admission.clone(),
            running.clone(),
            stopping.clone(),
        );
        let installed = installation::InstalledNativeGraph::new(owner.clone(), &config);
        Ok(Self {
            installed,
            admission_cursor: std::sync::Mutex::new(None),
            owner,
            config,
            _drivers: drivers,
            phases,
            phase_jobs,
            phase_dispatcher,
            phase_handoffs: Arc::new(phase_handoffs::PhaseHandoffs::default()),
            started: AtomicBool::new(false),
            running,
            stopping,
            wake: Arc::new(tokio::sync::Notify::new()),
            control_admission,
            #[cfg(test)]
            goal_admission_pause: std::sync::Mutex::new(None),
            #[cfg(test)]
            before_service_reconcile: std::sync::Mutex::new(None),
            supervisor: tokio::sync::Mutex::new(None),
        })
    }

    /// Actual retained service identity; content only, never a Driver capability.
    pub fn control_identity(&self) -> ServiceIdentity {
        ServiceIdentity {
            state: self.owner.state_path().to_path_buf(),
            instance: self.owner.instance_id().into(),
            epoch: self.owner.epoch(),
        }
    }
    fn service_running(&self) -> bool {
        self.running.load(Ordering::SeqCst) && !self.stopping.load(Ordering::SeqCst)
    }
    pub(crate) fn is_stopping(&self) -> bool {
        self.stopping.load(Ordering::SeqCst)
    }
    /// Actual retained service/supervisor identity, not Native admission.
    pub(crate) fn composition_is_current(
        &self,
        phases: &Arc<phase_supervisor::PhaseSupervisor>,
    ) -> bool {
        self.service_running() && Arc::ptr_eq(&self.phases, phases)
    }
}
#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) use tests::accepted_goal_fixture;

#[cfg(test)]
struct GoalAdmissionPause {
    reached: tokio::sync::oneshot::Sender<()>,
    release: tokio::sync::oneshot::Receiver<()>,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::SeqCst);
        self._drivers.stop_all();
        self.wake.notify_waiters();
    }
}

pub(crate) use phase_jobs::StartEnded;
pub(crate) use phase_jobs::{
    ClosedPhaseAck, ClosureStage, ClosureState, Retained, SettledLookup, SuccessContinuation,
    SuccessStage,
};
pub(crate) use phase_jobs::{assert_nonsuccess_unlocked, record_nonsuccess_store_attempt};
