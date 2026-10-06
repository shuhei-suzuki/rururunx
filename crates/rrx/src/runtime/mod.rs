//! Runtime integration milestones; no accepted Goal/Driver until schema9 is wired.
pub mod control;
pub mod goal;
use crate::{cli::transport::ServiceIdentity, config::Config, execution::RuntimeOwner};
use anyhow::Result;
use std::sync::Arc;

pub struct Runtime {
    owner: Arc<RuntimeOwner>,
    config: Config,
}
impl Runtime {
    /// Retains the existing owner, never opens another epoch or starts an Agent.
    pub fn new(owner: Arc<RuntimeOwner>, config: Config) -> Result<Self> {
        config.validate()?;
        Ok(Self { owner, config })
    }

    /// Actual retained service identity; content only, never a Driver capability.
    pub fn control_identity(&self) -> ServiceIdentity {
        ServiceIdentity {
            state: self.owner.state_path().to_path_buf(),
            instance: self.owner.instance_id().into(),
            epoch: self.owner.epoch(),
        }
    }
}
#[cfg(test)]
mod tests;
