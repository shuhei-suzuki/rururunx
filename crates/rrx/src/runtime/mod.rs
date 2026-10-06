//! Runtime integration milestones; no accepted Goal/Driver until schema9 is wired.
pub mod control;
pub mod goal;
use crate::{config::Config, execution::RuntimeOwner};
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
}
#[cfg(test)]
mod tests;
