//! Bounded native Git outside the ledger lock; admission and spawn have no await gap.
use super::*;
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
};

pub(crate) struct UnitGit {
    owner: Arc<RuntimeOwner>,
    unit: ExecutionUnit,
    profile: resources::ResourceProfile,
    native: bool,
}
impl UnitGit {
    pub(crate) fn new(
        owner: Arc<RuntimeOwner>,
        unit: &ExecutionUnit,
        native: bool,
    ) -> Result<Self> {
        let profile = resources::ResourceManager::new(owner.clone()).profile(unit)?;
        Ok(Self {
            owner,
            unit: unit.clone(),
            profile,
            native,
        })
    }
    pub(crate) async fn run<I, S>(&self, root: &Path, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let mut command = results::git_command(root)?;
        command
            .args(args)
            .envs(
                self.profile
                    .environment(&self.unit.cookie, self.owner.ipc_path())?,
            )
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let operation = OperationId::new();
        let child = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let current = store.execution_unit(self.unit.id)?;
            ensure!(
                current.scope == self.unit.scope
                    && current.generation == self.unit.generation
                    && current.owner_epoch == self.unit.owner_epoch
                    && current.session_id == self.unit.session_id,
                "Git preparation authority retired"
            );
            store.validate_execution(&current.authority(), self.native, !self.native)?;
            // Capture/inspection has Runtime-only finalization authority; never grant native tools.
            store.reserve_execution_helper(&current.authority(), operation, self.native, root)?;
            match process::OwnedProcess::spawn(&mut command) {
                Ok(child) => child,
                Err(e) => {
                    store.reconcile_managed_effect(
                        operation,
                        1,
                        EffectState::Unknown,
                        BTreeMap::new(),
                    )?;
                    return Err(e);
                }
            }
        };
        let observed = process::capture_child(child).await;
        let mut receipt = BTreeMap::new();
        if let Ok(o) = &observed {
            receipt.insert(
                "exit".into(),
                o.receipt
                    .status
                    .code()
                    .map_or_else(|| "signal".into(), |c| c.to_string()),
            );
            receipt.insert(
                "group_cleanup".into(),
                if o.receipt.group_error.is_some() {
                    "unknown"
                } else {
                    "requested"
                }
                .into(),
            );
        }
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .reconcile_managed_effect(
                operation,
                1,
                if observed.is_ok() {
                    EffectState::Confirmed
                } else {
                    EffectState::Unknown
                },
                receipt,
            )?;
        let observed = observed?;
        ensure!(
            observed.receipt.status.success(),
            "unit Git failed (exit {:?})",
            observed.receipt.status.code()
        );
        Ok(observed.stdout)
    }
    pub(crate) async fn text(
        &self,
        root: &Path,
        args: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
    ) -> Result<String> {
        results::text(&self.run(root, args).await?)
    }
    pub(crate) async fn ownership(
        &self,
        project: &crate::domain::Project,
        task: &crate::domain::Task,
    ) -> Result<()> {
        let root = &project.root;
        let path = task.worktree.as_deref().context("Task worktree missing")?;
        let source_top = PathBuf::from(self.text(root, ["rev-parse", "--show-toplevel"]).await?);
        let source_git_dir = PathBuf::from(
            self.text(root, ["rev-parse", "--path-format=absolute", "--git-dir"])
                .await?,
        );
        let source_common = PathBuf::from(
            self.text(
                root,
                ["rev-parse", "--path-format=absolute", "--git-common-dir"],
            )
            .await?,
        );
        let source_roots = self
            .text(
                root,
                [
                    "rev-list",
                    "--max-parents=0",
                    &format!("refs/heads/{}", project.base_branch),
                ],
            )
            .await?
            .lines()
            .map(str::to_owned)
            .collect();
        let task_top = PathBuf::from(self.text(path, ["rev-parse", "--show-toplevel"]).await?);
        let task_common = PathBuf::from(
            self.text(
                path,
                ["rev-parse", "--path-format=absolute", "--git-common-dir"],
            )
            .await?,
        );
        let branch = self
            .text(path, ["symbolic-ref", "--quiet", "--short", "HEAD"])
            .await?;
        let revision = self
            .text(path, ["rev-parse", "--verify", "HEAD^{commit}"])
            .await?;
        crate::git::validate_worktree_ownership(
            project,
            task,
            crate::git::WorktreeOwnershipFacts {
                source_top,
                source_git_dir,
                source_common,
                source_roots,
                task_top,
                task_common,
                branch,
                revision,
            },
        )?;
        Ok(())
    }
}
