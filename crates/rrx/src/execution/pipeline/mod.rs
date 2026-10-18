//! CLI startup qualification only; pipeline step execution is not installed yet.
mod classify;
pub use classify::{ProviderOutcome, classify_claude, classify_codex};
mod qualification;
pub use qualification::{CliQualifier, QualifiedCli};
mod argv;
pub use argv::{PipelineRole, step_arguments};
