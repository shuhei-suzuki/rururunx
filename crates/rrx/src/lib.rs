//! Provider-independent local orchestration. CLI parsing never starts an agent.
pub mod adapter;
pub mod cli;
pub mod codex;
pub mod config;
pub mod context;
pub mod domain;
pub mod execution;
pub mod git;
mod goal;
pub mod project;
pub mod state;
pub mod workflow;
