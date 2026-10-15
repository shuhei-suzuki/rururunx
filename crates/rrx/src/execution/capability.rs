//! Finite nongrant refusals. These values cannot authorize a helper or a Unit.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityRefusal {
    ClaudeExecutorUnavailable,
    ExecutorProviderUnavailable,
}
impl std::fmt::Display for CapabilityRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CapabilityRefusal {}

/// EX-1/B1: provider identity, independent of the operator's alias spelling.
pub(crate) fn require_executor(provider: Option<&str>) -> Result<(), CapabilityRefusal> {
    match provider {
        Some("codex") => Ok(()),
        Some("claude") => Err(CapabilityRefusal::ClaudeExecutorUnavailable),
        _ => Err(CapabilityRefusal::ExecutorProviderUnavailable),
    }
}
