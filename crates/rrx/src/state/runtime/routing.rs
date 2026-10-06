use super::super::*;
use anyhow::{Context, Result, ensure};
use std::path::{Path, PathBuf};

const BODY_BYTES: usize = 1024 * 1024;
const ROUTES: usize = 4096;
const TOTAL_BYTES: usize = 32 * 1024 * 1024;

/// Exact bounded read observation, never a Source/Driver capability.
#[derive(Debug)]
pub(crate) struct RoutingSnapshot {
    pub(crate) projects: Vec<Project>,
    pub(crate) tasks: Vec<Task>,
    fingerprint: String,
}
fn descendant_sql(column: &str) -> String {
    format!("(?1={column} OR substr(?1,1,length({column})+1)={column}||'/')")
}
impl Store {
    pub(crate) fn runtime_project_routes(
        &mut self,
        selector: Option<&str>,
        cwd: &Path,
        instance: &str,
        epoch: u64,
    ) -> Result<RoutingSnapshot> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Deferred)?;
        let current: (String, u64) = tx.query_row(
            "SELECT instance_id,epoch FROM runtime_epoch WHERE singleton=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        ensure!(
            epoch > 0 && current == (instance.to_owned(), epoch),
            "routing Runtime epoch changed"
        );
        let mut projects = Vec::new();
        let mut tasks = Vec::new();
        let mut consumed = 0usize;
        let (query, argument) = if let Some(selector) = selector {
            ensure!(
                !selector.trim().is_empty() && selector.len() <= 16384,
                "invalid Project selector"
            );
            if let Ok(id) = selector.parse::<ProjectId>() {
                (
                    "SELECT id,root,version,length(CAST(body AS BLOB)) FROM projects WHERE id=?1"
                        .to_owned(),
                    id.to_string(),
                )
            } else {
                ("SELECT id,root,version,length(CAST(body AS BLOB)) FROM projects WHERE json_extract(body,'$.name')=?1 AND coalesce(json_extract(body,'$.state'),'')<>'REMOVED' ORDER BY id LIMIT 3".to_owned(), selector.to_owned())
            }
        } else {
            ensure!(
                cwd.is_absolute(),
                "routing CWD must be canonical and absolute"
            );
            let query = format!(
                "SELECT id,root,version,length(CAST(body AS BLOB)) FROM projects WHERE {} AND coalesce(json_extract(body,'$.state'),'')<>'REMOVED' ORDER BY id LIMIT {}",
                descendant_sql("root"),
                ROUTES + 1
            );
            (
                query,
                cwd.to_str().context("non-UTF8 routing CWD")?.to_owned(),
            )
        };
        let mut stmt = tx.prepare(&query)?;
        let rows = stmt
            .query_map([&argument], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, u64>(2)?,
                    r.get::<_, usize>(3)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(stmt);
        ensure!(rows.len() <= ROUTES, "routing inventory exceeds bound");
        for (id, root, version, bytes) in rows {
            consumed = consumed
                .checked_add(bytes)
                .context("routing byte overflow")?;
            ensure!(
                bytes <= BODY_BYTES && consumed <= TOTAL_BYTES,
                "routing inventory bytes exceed bound"
            );
            let project: Project = read_tx(&tx, "projects", &id)?.context("Project disappeared")?;
            ensure!(
                project.id.to_string() == id
                    && project.root == PathBuf::from(root)
                    && project.version == version,
                "Project routing body/index mismatch"
            );
            projects.push(project);
        }
        if selector.is_none() {
            // Task worktrees may be outside their registered primary repository.
            let path = "json_extract(body,'$.worktree')";
            let query = format!(
                "SELECT id,project_id,goal_id,version,length(CAST(body AS BLOB)) FROM tasks WHERE {} ORDER BY id LIMIT 3",
                descendant_sql(path)
            );
            let mut stmt = tx.prepare(&query)?;
            let rows = stmt
                .query_map([&argument], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, u64>(3)?,
                        r.get::<_, usize>(4)?,
                    ))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            drop(stmt);
            ensure!(
                rows.len() <= 2,
                "ambiguous registered Task worktree routing"
            );
            for (id, project_id, goal_id, version, bytes) in rows {
                consumed = consumed
                    .checked_add(bytes)
                    .context("routing byte overflow")?;
                ensure!(
                    bytes <= BODY_BYTES && consumed <= TOTAL_BYTES,
                    "routing inventory bytes exceed bound"
                );
                let task: Task = read_tx(&tx, "tasks", &id)?.context("Task disappeared")?;
                ensure!(
                    task.id.to_string() == id
                        && task.project_id.to_string() == project_id
                        && task.goal_id.to_string() == goal_id
                        && task.version == version,
                    "Task routing body/index mismatch"
                );
                if !projects.iter().any(|p| p.id == task.project_id) {
                    let bytes: usize = tx.query_row(
                        "SELECT length(CAST(body AS BLOB)) FROM projects WHERE id=?1",
                        [&project_id],
                        |r| r.get(0),
                    )?;
                    consumed = consumed
                        .checked_add(bytes)
                        .context("routing byte overflow")?;
                    ensure!(
                        bytes <= BODY_BYTES && consumed <= TOTAL_BYTES,
                        "routing inventory bytes exceed bound"
                    );
                    let project: Project = read_tx(&tx, "projects", &project_id)?
                        .context("Task Project disappeared")?;
                    let (root, version): (String, u64) = tx.query_row(
                        "SELECT root,version FROM projects WHERE id=?1",
                        [&project_id],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )?;
                    ensure!(
                        project.id == task.project_id
                            && project.root == PathBuf::from(root)
                            && project.version == version,
                        "Task Project routing body/index mismatch"
                    );
                    projects.push(project);
                }
                tasks.push(task);
            }
        }
        projects.sort_by_key(|p| p.id);
        tasks.sort_by_key(|t| t.id);
        let fingerprint =
            crate::execution::workflow_source::digest(&serde_json::to_vec(&(&projects, &tasks))?);
        tx.commit()?;
        Ok(RoutingSnapshot {
            projects,
            tasks,
            fingerprint,
        })
    }
    pub(crate) fn recheck_runtime_project_routes(
        &mut self,
        selector: Option<&str>,
        cwd: &Path,
        instance: &str,
        epoch: u64,
        expected: &RoutingSnapshot,
    ) -> Result<()> {
        ensure!(
            self.runtime_project_routes(selector, cwd, instance, epoch)?
                .fingerprint
                == expected.fingerprint,
            "Project routing observation changed"
        );
        Ok(())
    }
}
