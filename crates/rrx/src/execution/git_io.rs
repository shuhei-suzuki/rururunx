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
    git_lease: Option<Arc<owner::GitLease>>,
    driver: Option<crate::state::DriverReadTicket>,
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
            git_lease: None,
            driver: None,
        })
    }
    pub(crate) fn with_driver_ticket(
        mut self,
        ticket: crate::state::DriverReadTicket,
    ) -> Result<Self> {
        ticket.preparation_matches(&self.unit)?;
        self.driver = Some(ticket);
        Ok(self)
    }
    pub(crate) fn validate_driver_current(&self) -> Result<()> {
        if let Some(ticket) = &self.driver {
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .validate_driver_read(ticket)?;
        }
        Ok(())
    }
    pub(crate) fn with_git_lease(mut self, lease: Arc<owner::GitLease>) -> Self {
        self.git_lease = Some(lease);
        self
    }
    pub(crate) async fn run<I, S>(&self, root: &Path, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let observed = self.run_observed(root, args).await?;
        ensure!(
            observed.receipt.status.success(),
            "unit Git failed (exit {:?})",
            observed.receipt.status.code()
        );
        Ok(observed.stdout)
    }
    pub(crate) async fn run_observed<I, S>(
        &self,
        root: &Path,
        args: I,
    ) -> Result<process::CommandCapture>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let mut command = results::git_command_for(
            root,
            self.profile
                .real_tools
                .get("git")
                .context("qualified Git executable missing")?,
        )?;
        command.args(args);
        self.run_command(root, command, "git_helper").await
    }
    /// Read-only Docker qualification; exact scoped registration still precedes
    /// every probe. This never grants creation or historical cleanup authority.
    pub(crate) async fn probe_docker(&self, args: &[String]) -> Result<Vec<u8>> {
        self.probe_docker_for(
            args,
            self.profile
                .real_tools
                .get("docker")
                .context("Docker is absent from profile")?,
        )
        .await
    }
    pub(crate) async fn probe_docker_for(
        &self,
        args: &[String],
        program: &Path,
    ) -> Result<Vec<u8>> {
        ensure!(self.native, "Docker probe requires open native authority");
        ensure!(
            program.is_absolute(),
            "Docker probe program must be absolute"
        );
        let mut command = tokio::process::Command::new(program);
        command.env("DOCKER_API_VERSION", "1.48");
        command.args(args).current_dir(&self.unit.worktree);
        let observed = self
            .run_command(&self.unit.worktree, command, "docker_probe")
            .await?;
        ensure!(
            observed.receipt.status.success(),
            "Docker probe unavailable"
        );
        ensure!(
            observed.stdout.len() <= 8192,
            "Docker probe response exceeds bound"
        );
        Ok(observed.stdout)
    }
    async fn run_command(
        &self,
        root: &Path,
        mut command: tokio::process::Command,
        kind: &str,
    ) -> Result<process::CommandCapture> {
        command
            .envs(
                self.profile
                    .environment(&self.unit.cookie, self.owner.ipc_path())?,
            )
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let operation = OperationId::new();
        if let Some(lease) = &self.git_lease {
            command.env("RRX_GIT_GATE_TOKEN", lease.id.to_string());
        } else {
            command.env_remove("RRX_GIT_GATE_TOKEN");
        }
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
            store.reserve_execution_helper_pinned(
                &current.authority(),
                operation,
                self.native,
                root,
                kind,
                self.driver.as_ref(),
            )?;
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
        let mut helper_guard = owner::HelperGuard::new(self.owner.clone(), operation);
        let observed = process::capture_scoped_pinned(
            child,
            &self.owner,
            &self.unit,
            self.native,
            self.driver.as_ref(),
        )
        .await;
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
            .reconcile_managed_effect_pinned(
                operation,
                1,
                if observed.is_ok() {
                    EffectState::Confirmed
                } else {
                    EffectState::Unknown
                },
                receipt,
                if observed.is_ok() {
                    self.driver.as_ref()
                } else {
                    None
                },
            )?;
        helper_guard.disarm();
        observed
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
