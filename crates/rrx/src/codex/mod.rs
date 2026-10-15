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
