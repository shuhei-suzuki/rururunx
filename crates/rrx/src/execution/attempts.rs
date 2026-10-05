use super::{
    git_io::UnitGit,
    resources::{ResourceManager, ResourceProfile},
    results::{git, text},
    *,
};
use anyhow::{Context, Result, ensure};
use std::sync::Arc;

pub struct AttemptManager {
    owner: Arc<RuntimeOwner>,
    resources: ResourceManager,
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
        let base = match base {
            Some(oid) => {
                ensure!(valid_oid(oid), "base must be exact OID");
                oid.to_owned()
            }
            None => text(
                &git(
                    &project.root,
                    [
                        "rev-parse",
                        "--verify",
                        &format!("refs/heads/{}^{{commit}}", project.base_branch),
                    ],
                )
                .await?,
            )?,
        };
        ensure!(valid_oid(&base), "base identity missing");
        results::qualified_content(&project.root, &base).await?;
        let id = UnitId::new();
        let path = project.worktree_root.join(format!("{}-{id}", task.id));
        ensure!(
            !path.exists() && !path.symlink_metadata().is_ok(),
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
            base_sha: base.clone(),
            profile_digest: profile.digest.clone(),
            cookie: uuid::Uuid::new_v4().to_string(),
            session_id: None,
            artifact_id: None,
            wait_reason: None,
            created_at: at,
            updated_at: at,
        };
        let unit = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .reserve_execution(unit, task.version)?;
        let prepared = async {
            self.resources.reserve(&unit, &profile)?;
            self.resources.materialize(&profile)?;
            let preparing = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .transition_execution(&unit.authority(), UnitState::Preparing)?;
            std::fs::create_dir_all(&project.worktree_root)?;
            ensure!(
                project.worktree_root.canonicalize()? == project.worktree_root,
                "worktree namespace must be canonical"
            );
            let common = self.owner.git_lease(preparing.id, None).await?;
            let io = UnitGit::new(self.owner.clone(), &preparing, true)?.with_git_lease(common);
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
            Ok(unit) => Ok((unit, profile)),
            Err(e) => {
                self.close_failed_preparation(id)?;
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
            created_at: at,
            updated_at: at,
        };
        let unit = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .reserve_execution(unit, task.version)?;
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
