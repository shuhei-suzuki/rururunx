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
