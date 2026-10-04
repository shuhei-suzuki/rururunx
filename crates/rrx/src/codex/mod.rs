//! Native Codex app-server transport; native inference/authentication stay in Codex.
mod attempt;
mod availability;
mod ownership;
mod preparation;
mod session;
pub use session::CodexAdapter;
pub mod policy;
mod protocol;
mod transport;
