//! Native Codex app-server transport; native inference/authentication stay in Codex.
mod attempt;
mod availability;
mod custody;
mod environment;
mod ownership;
mod preparation;
mod session;
pub use session::CodexAdapter;
pub mod policy;
mod protocol;
mod transport;
