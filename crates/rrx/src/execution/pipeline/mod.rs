//! Qualified CLI launch and evidence-based process custody. No scheduler or adoption.
mod launcher;
pub use launcher::{LaunchResult, StepInput, StepLauncher};
mod custody;
pub use custody::StepCustody;
mod classify;
pub use classify::{ProviderOutcome, classify_claude, classify_codex};
mod qualification;
pub(crate) use qualification::QualificationRecord;
pub use qualification::{CliQualifier, QualifiedCli};
mod argv;
pub use argv::{PipelineRole, step_arguments};

#[cfg(all(test, target_os = "macos"))]
#[path = "../../../tests/support/pipeline_cli.rs"]
pub(crate) mod test_support;
