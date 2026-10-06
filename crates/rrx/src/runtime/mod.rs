//! Trusted Goal controls and durable attention. Native Driver admission is unavailable.
pub mod control;
pub(crate) mod driver;
pub mod goal;
mod service;
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
    started: AtomicBool,
    running: Arc<AtomicBool>,
    stopping: AtomicBool,
    wake: Arc<tokio::sync::Notify>,
    control_admission: tokio::sync::Mutex<()>,
    #[cfg(test)]
    goal_admission_pause: std::sync::Mutex<Option<GoalAdmissionPause>>,
    supervisor: tokio::sync::Mutex<Option<tokio::task::JoinHandle<Result<()>>>>,
}
impl Runtime {
    /// Retains the existing owner, never opens another epoch or starts an Agent.
    pub fn new(owner: Arc<RuntimeOwner>, config: Config) -> Result<Self> {
        config.validate()?;
        let drivers = driver::DriverRegistry::new(owner.epoch());
        owner.attach_runtime_drivers(&drivers)?;
        Ok(Self {
            owner,
            config,
            _drivers: drivers,
            started: AtomicBool::new(false),
            running: Arc::new(AtomicBool::new(false)),
            stopping: AtomicBool::new(false),
            wake: Arc::new(tokio::sync::Notify::new()),
            control_admission: tokio::sync::Mutex::new(()),
            #[cfg(test)]
            goal_admission_pause: std::sync::Mutex::new(None),
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
}
#[cfg(test)]
mod tests;

#[cfg(test)]
struct GoalAdmissionPause {
    reached: tokio::sync::oneshot::Sender<()>,
    release: tokio::sync::oneshot::Receiver<()>,
}
