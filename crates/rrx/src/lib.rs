//! Provider-independent local orchestration. CLI parsing never starts an agent.
pub mod adapter;
pub mod config;
pub mod context;
pub mod domain;
pub mod git;
pub mod project;
pub mod state;
