//! Native Codex app-server transport; native inference/authentication stay in Codex.
mod attempt;
mod availability;
mod custody;
mod ownership;
mod preparation;
mod session;
pub use session::CodexAdapter;
pub(crate) mod managed;
pub mod policy;
mod protocol;
mod transport;

/// AC-11: independent of the retained #19 component's availability inputs.
fn require_legacy_entry() -> crate::adapter::AdapterResult<()> {
    crate::execution::capability::require_legacy_entry().map_err(|refusal| {
        protocol::failure(
            crate::adapter::ErrorKind::CapabilityUnavailable(refusal),
            refusal.to_string(),
        )
    })
}
