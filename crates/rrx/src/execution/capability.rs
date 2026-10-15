//! Finite nongrant refusals. These values cannot authorize a helper or a Unit.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityRefusal {
    ClaudeExecutorUnavailable,
    LegacyEntryUnavailable,
    M0CapabilityChainUnavailable,
}
impl std::fmt::Display for CapabilityRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CapabilityRefusal {}

/// EX-1/B1: provider identity, independent of the operator's alias spelling.
pub(crate) fn require_executor(provider: Option<&str>) -> Result<(), CapabilityRefusal> {
    if provider == Some("claude") {
        return Err(CapabilityRefusal::ClaudeExecutorUnavailable);
    }
    Ok(())
}

/// AC-11/L-1: obsolete entry points never acquire preparation or process custody.
pub(crate) fn require_legacy_entry() -> Result<(), CapabilityRefusal> {
    Err(CapabilityRefusal::LegacyEntryUnavailable)
}

/// U2 branch boundary, replaced only together with the real S/P/Executor chain.
pub(crate) fn require_m0_capabilities() -> Result<(), CapabilityRefusal> {
    Err(CapabilityRefusal::M0CapabilityChainUnavailable)
}
