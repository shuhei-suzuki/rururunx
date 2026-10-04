//! Provider-independent local orchestration. CLI parsing never starts an agent.
pub mod adapter;
pub mod codex;
pub mod config;
pub mod context;
pub mod context_pack;
pub mod domain;
pub mod git;
pub mod project;
pub mod state;
pub mod workflow;
