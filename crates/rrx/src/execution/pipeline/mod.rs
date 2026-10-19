//! Qualified CLI launch, process custody and checked Executor adoption.
mod git;
pub use git::PipelineGit;
pub(crate) mod adoption;
pub use adoption::AdoptionChecker;
pub(crate) mod workspace;
pub use workspace::TaskWorkspace;
mod secret;
pub(crate) use secret::SecretHit;
pub use secret::SecretScanner;
mod launcher;
pub(crate) mod tests_runner;
pub use launcher::{LaunchResult, StepInput, StepLauncher};
pub use tests_runner::PipelineTestRunner;
pub(crate) use tests_runner::PreparedTest;
mod custody;
pub use custody::StepCustody;
mod classify;
pub use classify::{ProviderOutcome, classify_claude, classify_codex};
mod qualification;
pub(crate) use qualification::QualificationRecord;
pub use qualification::{CliQualifier, QualifiedCli};
mod argv;
pub use argv::{PipelineRole, step_arguments};

fn ensure_private_directory(path: &std::path::Path) -> anyhow::Result<()> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};

    match std::fs::DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error.into()),
    }
    let meta = std::fs::symlink_metadata(path)?;
    anyhow::ensure!(
        meta.is_dir()
            && !meta.file_type().is_symlink()
            && meta.permissions().mode() & 0o777 == 0o700
            && meta.uid() == rustix::process::getuid().as_raw(),
        "private directory changed"
    );
    Ok(())
}

#[cfg(all(test, target_os = "macos"))]
#[path = "../../../tests/support/pipeline_cli.rs"]
pub(crate) mod test_support;
