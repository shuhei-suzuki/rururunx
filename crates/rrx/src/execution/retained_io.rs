//! Current-Runtime inspection of historical artifacts, without reopening Tasks.
use super::*;
use anyhow::{Result, ensure};
use std::{os::unix::ffi::OsStrExt, path::PathBuf, process::Stdio, sync::Arc, time::Duration};

pub(crate) struct RetainedGit {
    owner: Arc<RuntimeOwner>,
    artifact: ResultArtifact,
    program: PathBuf,
}
impl RetainedGit {
    pub(crate) fn new(owner: Arc<RuntimeOwner>, artifact: &ResultArtifact) -> Result<Self> {
        // Reject a manufactured/stale artifact before executable resolution or
        // any native effects. This does not depend on the old unit's paths.
        owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .validate_retained_inspection(owner.epoch, artifact)?;
        Ok(Self {
            program: resources::resolve_program("git")?,
            owner,
            artifact: artifact.clone(),
        })
    }
    #[cfg(test)]
    pub(super) fn with_program(mut self, program: PathBuf) -> Self {
        self.program = program;
        self
    }
    pub(crate) fn validate(&self) -> Result<()> {
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .validate_retained_inspection(self.owner.epoch, &self.artifact)
    }
    pub(crate) async fn run<const N: usize>(&self, args: [&str; N]) -> Result<Vec<u8>> {
        // No generic historical command capability. Inputs are exact refs/OIDs
        // from the pinned artifact and a finite, read-only command set.
        let commit_ref = format!("refs/rrx/{}/commit", self.artifact.id);
        let base_ref = format!("refs/rrx/{}/base", self.artifact.id);
        let action = match args.as_slice() {
            ["rev-parse", "--verify", reference] if *reference == commit_ref => "commit_ref",
            ["rev-parse", "--verify", reference] if *reference == base_ref => "base_ref",
            ["rev-list", "--objects", "--missing=error", oid] if *oid == self.artifact.revision => {
                "commit_graph"
            }
            ["rev-list", "--objects", "--missing=error", oid] if *oid == self.artifact.base_sha => {
                "base_graph"
            }
            ["fsck", "--full", "--strict", "--no-dangling"] => "fsck",
            _ => anyhow::bail!("unsupported retained Git command"),
        };
        let operation = OperationId::new();
        let mut command = results::git_command_for(&self.artifact.repository, &self.program)?;
        command
            .args(args)
            .current_dir(&self.artifact.repository)
            .env("RRX_PROCESS_COOKIE", operation.to_string())
            .env("GIT_NO_LAZY_FETCH", "1")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .env_remove("RRX_UNIT_ID")
            .env_remove("RRX_PROFILE")
            .env_remove("RRX_RUNTIME_SOCKET")
            .env_remove("RRX_GIT_GATE_TOKEN")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let child = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            store.reserve_retained_inspection(
                self.owner.epoch,
                &self.artifact,
                operation,
                action,
            )?;
            // There is no await/unlocked gap between durable intent and spawn.
            match process::OwnedProcess::spawn(&mut command) {
                Ok(child) => child,
                Err(error) => {
                    store.reconcile_managed_effect(
                        operation,
                        1,
                        EffectState::Unknown,
                        std::collections::BTreeMap::new(),
                    )?;
                    return Err(error);
                }
            }
        };
        let mut guard = owner::HelperGuard::new(self.owner.clone(), operation);
        let observed = {
            let capture = process::capture_child(child);
            tokio::pin!(capture);
            let mut fence = tokio::time::interval(Duration::from_millis(50));
            loop {
                tokio::select! {
                    result = &mut capture => break result?,
                    _ = fence.tick() => self.validate()?,
                }
            }
        };
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .finish_retained_inspection(
                self.owner.epoch,
                &self.artifact,
                operation,
                observed.receipt.status.code(),
                observed.receipt.group_error.is_some(),
                &results::hex(self.program.as_os_str().as_bytes()),
            )?;
        guard.disarm();
        ensure!(
            observed.receipt.status.success(),
            "retained Git failed (exit {:?})",
            observed.receipt.status.code()
        );
        Ok(observed.stdout)
    }
}
