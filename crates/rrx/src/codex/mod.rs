//! Native Codex app-server transport; native inference/authentication stay in Codex.
mod ownership;
mod preparation;
mod session;
pub use session::CodexAdapter;
pub mod policy;
pub mod protocol;
pub mod transport;
