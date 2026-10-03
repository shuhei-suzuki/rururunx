//! Native Claude Code sessions. No model API, context selection or workflow engine.
mod ownership;
mod policy;
mod protocol;
mod pty;
mod session;
mod transport;
pub use session::ClaudeAdapter;
