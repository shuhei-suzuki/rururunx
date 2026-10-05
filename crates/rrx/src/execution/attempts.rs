use super::{
    git_io::UnitGit,
    resources::{ResourceManager, ResourceProfile},
    *,
};
use anyhow::{Context, Result, ensure};
use std::{collections::BTreeMap, sync::Arc};

pub struct AttemptManager {
    owner: Arc<RuntimeOwner>,
    resources: ResourceManager,
}

/// Live preparation provenance. It cannot be reconstructed from a ledger row.
/// Dropping it retires the owned preparation, without claiming process recovery.
pub struct PreparedExecutor {
    owner: Arc<RuntimeOwner>,
    unit: ExecutionUnit,
    guard: owner::PreparationGuard,
}
/// Constructed only after the owning preparation validates its physical namespace.
pub(crate) struct PreparedAdoption {
    unit: ExecutionUnit,
    sources: BTreeMap<String, String>,
    payload_digest: String,
}
impl PreparedAdoption {
    pub(crate) fn unit(&self) -> &ExecutionUnit {
        &self.unit
    }
    pub(crate) fn sources(&self) -> &BTreeMap<String, String> {
        &self.sources
    }
    pub(crate) fn payload_digest(&self) -> &str {
        &self.payload_digest
    }
}
impl PreparedExecutor {
    pub fn unit(&self) -> &ExecutionUnit {
        &self.unit
    }
    pub(crate) async fn verify_namespace(&self) -> Result<()> {
        let (task, project) = {
            let store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let current = store.validate_execution(&self.unit.authority(), true, true)?;
            ensure!(
                current.kind == UnitKind::Executor
                    && current.phase == WORKFLOW_SOURCE_BOOTSTRAP
                    && current.state == UnitState::Preparing
                    && current.work.is_none()
                    && current.session_id.is_none()
                    && current.artifact_id.is_none(),
                "invalid live preparation"
            );
            let task = store
                .task(self.unit.scope.task_id.context("Task required")?)?
                .context("Task missing")?;
            let project = store.project(task.project_id)?.context("Project missing")?;
            (task, project)
        };
        let lease = self.owner.git_lease(self.unit.id, None).await?;
        let io = UnitGit::new(self.owner.clone(), &self.unit, true)?.with_git_lease(lease);
        io.ownership(&project, &task).await?;
        ensure!(
            io.text(&self.unit.worktree, ["rev-parse", "HEAD"]).await? == self.unit.base_sha,
            "prepared worktree HEAD changed"
        );
        ensure!(
            io.run(
                &self.unit.worktree,
                ["status", "--porcelain", "--untracked-files=all"]
            )
            .await?
            .is_empty(),
            "prepared worktree changed before initial gate"
        );
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .validate_execution(&self.unit.authority(), true, true)?;
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn adopt(
        mut self,
        expected_task: u64,
        reservation: &WorkflowReservation,
        phase: &str,
        provider: &str,
        sources: &BTreeMap<String, String>,
        payload: &str,
    ) -> Result<ExecutionUnit> {
        let (task, project) = {
            let store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            store.validate_execution(&self.unit.authority(), true, true)?;
            let task = store
                .task(self.unit.scope.task_id.context("Task required")?)?
                .context("Task missing")?;
            let project = store.project(task.project_id)?.context("Project missing")?;
            (task, project)
        };
        let lease = self.owner.git_lease(self.unit.id, None).await?;
        let io = UnitGit::new(self.owner.clone(), &self.unit, true)?.with_git_lease(lease);
        io.ownership(&project, &task).await?;
        ensure!(
            io.text(&self.unit.worktree, ["rev-parse", "HEAD"]).await? == self.unit.base_sha,
            "prepared worktree HEAD changed"
        );
        ensure!(
            io.run(
                &self.unit.worktree,
                ["status", "--porcelain", "--untracked-files=all"]
            )
            .await?
            .is_empty(),
            "prepared worktree changed before adoption"
        );
        let proof = PreparedAdoption {
            unit: self.unit.clone(),
            sources: sources.clone(),
            payload_digest: super::workflow_source::digest(payload.as_bytes()),
        };
        let adopted = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .adopt_prepared_workflow_execution(
                &proof,
                expected_task,
                reservation,
                phase,
                provider,
            )?;
        self.guard.disarm();
        Ok(adopted)
    }

    /// Read an ordinary tracked file from the exact prepared commit, never HEAD
    /// or the worktree bytes. Symlinks and submodules are not file inputs.
    pub async fn read_committed_file(&self, path: &str) -> Result<Vec<u8>> {
        ensure!(
            !path.is_empty()
                && path.len() <= 4096
                && !path.contains('\0')
                && std::path::Path::new(path)
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_))),
            "committed source path must be relative"
        );
        let io = UnitGit::new(self.owner.clone(), &self.unit, true)?;
        let entries = io
            .run(
                &self.unit.worktree,
                [
                    "--literal-pathspecs",
                    "ls-tree",
                    "-z",
                    "--full-tree",
                    &self.unit.base_sha,
                    "--",
                    path,
                ],
            )
            .await?;
        let mut entries = entries.split(|byte| *byte == 0).filter(|e| !e.is_empty());
        let entry = entries.next().context("committed source file missing")?;
        ensure!(entries.next().is_none(), "ambiguous committed source path");
        let tab = entry
            .iter()
            .position(|byte| *byte == b'\t')
            .context("invalid committed source entry")?;
        let (header, name) = (&entry[..tab], &entry[tab + 1..]);
        let header = std::str::from_utf8(header)?;
        let fields = header.split(' ').collect::<Vec<_>>();
        ensure!(
            fields.len() == 3
                && matches!(fields[0], "100644" | "100755")
                && fields[1] == "blob"
                && valid_oid(fields[2])
                && name == path.as_bytes(),
            "committed source is not an exact ordinary file"
        );
        let size: usize = io
            .text(&self.unit.worktree, ["cat-file", "-s", fields[2]])
            .await?
            .parse()?;
        ensure!(size <= 256 * 1024, "committed source exceeds file bound");
        let bytes = io
            .run(&self.unit.worktree, ["cat-file", "blob", fields[2]])
            .await?;
        ensure!(bytes.len() == size, "committed blob size mismatch");
        Ok(bytes)
    }
}
impl AttemptManager {
    pub fn new(owner: Arc<RuntimeOwner>) -> Self {
        Self {
            resources: ResourceManager::new(owner.clone()),
            owner,
        }
    }
    pub async fn prepare(
        &self,
        task: crate::domain::TaskId,
        provider: &str,
        phase: &str,
        base: Option<&str>,
    ) -> Result<(ExecutionUnit, ResourceProfile)> {
        ensure!(
            phase != WORKFLOW_SOURCE_BOOTSTRAP,
            "reserved preparation phase"
        );
        self.prepare_inner(task, provider, phase, base, None).await
    }
    /// Pre-register the first Workflow's Git source namespace. This is not a
    /// native launch capability or a successful work/result artifact.
    pub async fn prepare_workflow_source(
        &self,
        task: crate::domain::TaskId,
        provider: &str,
    ) -> Result<PreparedExecutor> {
        let (unit, _) = self
            .prepare_inner(task, provider, WORKFLOW_SOURCE_BOOTSTRAP, None, None)
            .await?;
        let guard = owner::PreparationGuard::new(self.owner.clone(), &unit);
        Ok(PreparedExecutor {
            owner: self.owner.clone(),
            unit,
            guard,
        })
    }
    pub(crate) async fn prepare_workflow(
        &self,
        task: crate::domain::TaskId,
        provider: &str,
        phase: &str,
        base: &str,
        reservation: &WorkflowReservation,
    ) -> Result<(ExecutionUnit, ResourceProfile)> {
        self.prepare_inner(task, provider, phase, Some(base), Some(reservation))
            .await
    }
    async fn prepare_inner(
        &self,
        task: crate::domain::TaskId,
        provider: &str,
        phase: &str,
        base: Option<&str>,
        reservation: Option<&WorkflowReservation>,
    ) -> Result<(ExecutionUnit, ResourceProfile)> {
        ensure!(
            matches!(provider, "codex" | "claude") && !phase.is_empty() && phase.len() <= 64,
            "unsupported native provider/phase"
        );
        let (task, project) = {
            let store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let task = store.task(task)?.context("Task missing")?;
            let project = store.project(task.project_id)?.context("Project missing")?;
            (task, project)
        };
        ensure!(
            project.root.canonicalize()? == project.root
                && !self.owner.root.starts_with(&project.worktree_root),
            "state must be independent of worktree namespace"
        );
        let base = base
            .map(|oid| {
                ensure!(valid_oid(oid), "base must be exact OID");
                Ok::<_, anyhow::Error>(oid.to_owned())
            })
            .transpose()?;
        let id = UnitId::new();
        let path = project.worktree_root.join(format!("{}-{id}", task.id));
        ensure!(
            !path.exists() && path.symlink_metadata().is_err(),
            "fresh worktree path already exists"
        );
        let profile = self.resources.draft(id, &task.scope(), &path)?;
        let branch = format!("rrx/{}/{id}", task.id);
        let at = crate::domain::now_ms();
        let unit = ExecutionUnit {
            id,
            scope: task.scope(),
            kind: UnitKind::Executor,
            generation: 0,
            owner_epoch: self.owner.epoch,
            version: 0,
            phase: phase.into(),
            provider: provider.into(),
            state: UnitState::Reserved,
            native_effects_open: true,
            result_finalization_open: true,
            work: None,
            cleanup: CleanupOutcome::Unknown,
            disposition: Disposition::Active,
            worktree: path.clone(),
            branch: Some(branch.clone()),
            base_sha: base.clone().unwrap_or_default(),
            profile_digest: profile.digest.clone(),
            cookie: uuid::Uuid::new_v4().to_string(),
            session_id: None,
            artifact_id: None,
            wait_reason: None,
            capacity_retry_at: None,
            created_at: at,
            updated_at: at,
        };
        let unit = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            match reservation {
                Some(r) => store.reserve_workflow_execution(unit, task.version, r)?,
                None => store.reserve_execution(unit, task.version)?,
            }
        };
        let mut preparation_guard = owner::PreparationGuard::new(self.owner.clone(), &unit);
        let prepared = async {
            self.resources.reserve(&unit, &profile)?;
            self.resources.materialize(&profile)?;
            let mut preparing = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .transition_execution(&unit.authority(), UnitState::Preparing)?;
            preparation_guard.update(&preparing);
            let common = self.owner.git_lease(preparing.id, None).await?;
            let io =
                UnitGit::new(self.owner.clone(), &preparing, true)?.with_git_lease(common.clone());
            let base = match base {
                Some(base) => base,
                None => {
                    let base = io
                        .text(
                            &project.root,
                            [
                                "rev-parse",
                                "--verify",
                                &format!("refs/heads/{}^{{commit}}", project.base_branch),
                            ],
                        )
                        .await?;
                    preparing = self
                        .owner
                        .store
                        .lock()
                        .map_err(|_| anyhow::anyhow!("state poisoned"))?
                        .bind_execution_base(&preparing.authority(), &base)?;
                    preparation_guard.update(&preparing);
                    base
                }
            };
            let io = UnitGit::new(self.owner.clone(), &preparing, true)?.with_git_lease(common);
            results::qualified_content_scoped(&project.root, &base, &io).await?;
            std::fs::create_dir_all(&project.worktree_root)?;
            ensure!(
                project.worktree_root.canonicalize()? == project.worktree_root,
                "worktree namespace must be canonical"
            );
            io.run(
                &project.root,
                [
                    "worktree",
                    "add",
                    "-b",
                    &branch,
                    path.to_str().context("worktree UTF-8")?,
                    &base,
                ],
            )
            .await?;
            let current_task = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .task(task.id)?
                .context("Task missing")?;
            io.ownership(&project, &current_task).await?;
            let current = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .execution_unit(id)?;
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .validate_execution(&current.authority(), true, false)?;
            Ok::<_, anyhow::Error>(current)
        }
        .await;
        match prepared {
            Ok(unit) => {
                preparation_guard.disarm();
                Ok((unit, profile))
            }
            Err(e) => {
                drop(preparation_guard);
                Err(e)
            }
        }
    }
    pub async fn prepare_snapshot(
        &self,
        task: crate::domain::TaskId,
        artifact: ArtifactId,
        kind: UnitKind,
        provider: &str,
        phase: &str,
    ) -> Result<(ExecutionUnit, ResourceProfile)> {
        ensure!(
            phase != WORKFLOW_SOURCE_BOOTSTRAP,
            "reserved preparation phase"
        );
        self.prepare_snapshot_inner(task, artifact, kind, provider, phase, None)
            .await
    }
    pub(crate) async fn prepare_workflow_snapshot(
        &self,
        task: crate::domain::TaskId,
        artifact: ArtifactId,
        provider: &str,
        phase: &str,
        reservation: &WorkflowReservation,
    ) -> Result<(ExecutionUnit, ResourceProfile)> {
        self.prepare_snapshot_inner(
            task,
            artifact,
            UnitKind::Reviewer,
            provider,
            phase,
            Some(reservation),
        )
        .await
    }
    async fn prepare_snapshot_inner(
        &self,
        task: crate::domain::TaskId,
        artifact: ArtifactId,
        kind: UnitKind,
        provider: &str,
        phase: &str,
        reservation: Option<&WorkflowReservation>,
    ) -> Result<(ExecutionUnit, ResourceProfile)> {
        ensure!(
            matches!(kind, UnitKind::Reviewer | UnitKind::Verifier),
            "snapshot unit kind invalid"
        );
        let (task, a) = {
            let store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            (
                store.task(task)?.context("Task missing")?,
                store.result_artifact(artifact)?,
            )
        };
        ensure!(
            a.scope == task.scope() && a.state == ArtifactState::Published,
            "snapshot artifact not published for Task"
        );
        let id = UnitId::new();
        let path = self
            .owner
            .root
            .join("units")
            .join(id.to_string())
            .join("source");
        let profile = self.resources.draft(id, &task.scope(), &path)?;
        let at = crate::domain::now_ms();
        let unit = ExecutionUnit {
            id,
            scope: task.scope(),
            kind,
            generation: 0,
            owner_epoch: self.owner.epoch,
            version: 0,
            phase: phase.into(),
            provider: provider.into(),
            state: UnitState::Reserved,
            native_effects_open: true,
            result_finalization_open: true,
            work: None,
            cleanup: CleanupOutcome::Unknown,
            disposition: Disposition::Active,
            worktree: path,
            branch: None,
            base_sha: a.revision,
            profile_digest: profile.digest.clone(),
            cookie: uuid::Uuid::new_v4().to_string(),
            session_id: None,
            artifact_id: Some(artifact),
            wait_reason: None,
            capacity_retry_at: None,
            created_at: at,
            updated_at: at,
        };
        let unit = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            match reservation {
                Some(r) => store.reserve_workflow_execution(unit, task.version, r)?,
                None => store.reserve_execution(unit, task.version)?,
            }
        };
        let prepared = (|| {
            self.resources.reserve(&unit, &profile)?;
            self.resources.materialize(&profile)?;
            Ok::<_, anyhow::Error>(())
        })();
        match prepared {
            Ok(()) => Ok((unit, profile)),
            Err(e) => {
                self.close_failed_preparation(id)?;
                Err(e)
            }
        }
    }
    pub fn retire(
        &self,
        authority: &ExecutionAuthority,
        cancel_task: bool,
    ) -> Result<ExecutionUnit> {
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .retire_execution(authority, cancel_task)
    }
    fn close_failed_preparation(&self, id: UnitId) -> Result<()> {
        let mut store = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        let current = store.execution_unit(id)?;
        if current.native_effects_open || current.result_finalization_open {
            // Retain paths/leases and unknown issued effects. Logical closure is not disposal.
            store.retire_execution(&current.authority(), false)?;
        }
        Ok(())
    }
}
