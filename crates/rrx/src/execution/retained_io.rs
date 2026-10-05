//! Current-Runtime inspection of historical artifacts, without reopening Tasks.
use super::*;
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeSet,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    process::Stdio,
    sync::{Arc, Mutex},
    time::Duration,
};

pub(crate) struct RetainedGit {
    owner: Arc<RuntimeOwner>,
    artifact: ResultArtifact,
    program: PathBuf,
    source_blobs: Mutex<BTreeSet<String>>,
    recovery: Option<crate::state::SourceReadBinding>,
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
            source_blobs: Mutex::new(BTreeSet::new()),
            recovery: None,
        })
    }
    pub(crate) fn for_recovery(
        owner: Arc<RuntimeOwner>,
        artifact: &ResultArtifact,
        binding: crate::state::SourceReadBinding,
    ) -> Result<Self> {
        ensure!(
            binding.owns_artifact(artifact.id),
            "source-bound reader artifact differs"
        );
        owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .validate_source_read(&binding)?;
        let mut io = Self::new(owner, artifact)?;
        io.recovery = Some(binding);
        Ok(io)
    }
    #[cfg(test)]
    pub(super) fn with_program(mut self, program: PathBuf) -> Self {
        self.program = program;
        self
    }
    pub(crate) fn validate(&self) -> Result<()> {
        let store = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        store.validate_retained_inspection(self.owner.epoch, &self.artifact)?;
        if let Some(binding) = &self.recovery {
            store.validate_source_read(binding)?;
        }
        Ok(())
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
            ["ls-tree", "-r", "-z", "-l", "--full-tree", oid] if *oid == self.artifact.revision => {
                "source_tree"
            }
            ["cat-file", "blob", oid]
                if self
                    .source_blobs
                    .lock()
                    .map_err(|_| anyhow::anyhow!("source index poisoned"))?
                    .contains(*oid) =>
            {
                "source_blob"
            }
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
            if self.recovery.is_some() {
                store.reserve_retained_inspection_bound(
                    self.owner.epoch,
                    &self.artifact,
                    operation,
                    action,
                    self.recovery.as_ref(),
                )?;
            } else {
                store.reserve_retained_inspection(
                    self.owner.epoch,
                    &self.artifact,
                    operation,
                    action,
                )?;
            }
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
        {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let program = results::hex(self.program.as_os_str().as_bytes());
            if self.recovery.is_some() {
                store.finish_retained_inspection_bound(
                    self.owner.epoch,
                    &self.artifact,
                    operation,
                    (
                        observed.receipt.status.code(),
                        observed.receipt.group_error.is_some(),
                        &program,
                    ),
                    self.recovery.as_ref(),
                )?;
            } else {
                store.finish_retained_inspection(
                    self.owner.epoch,
                    &self.artifact,
                    operation,
                    observed.receipt.status.code(),
                    observed.receipt.group_error.is_some(),
                    &program,
                )?;
            }
        }
        guard.disarm();
        ensure!(
            observed.receipt.status.success(),
            "retained Git failed (exit {:?})",
            observed.receipt.status.code()
        );
        if action == "source_tree" {
            let mut blobs = BTreeSet::new();
            let mut count = 0;
            for entry in observed.stdout.split(|b| *b == 0).filter(|e| !e.is_empty()) {
                count += 1;
                ensure!(count <= 4096, "retained source inventory exceeds bound");
                let header = entry
                    .split(|b| *b == b'\t')
                    .next()
                    .context("source tree header missing")?;
                let fields = std::str::from_utf8(header)?
                    .split_whitespace()
                    .collect::<Vec<_>>();
                ensure!(
                    fields.len() == 4 && valid_oid(fields[2]),
                    "invalid retained source tree entry"
                );
                if matches!(fields[0], "100644" | "100755") && fields[1] == "blob" {
                    let size: usize = fields[3].parse()?;
                    if size <= 256 * 1024 {
                        blobs.insert(fields[2].to_owned());
                    }
                }
            }
            *self
                .source_blobs
                .lock()
                .map_err(|_| anyhow::anyhow!("source index poisoned"))? = blobs;
        }
        Ok(observed.stdout)
    }
}
