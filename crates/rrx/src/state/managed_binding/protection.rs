//! Conservative legacy-entry classification, never an affirmative phase grant.
use super::canonical::{BODY_BYTES, decode_value};
use crate::{
    domain::*,
    state::{SCHEMA_VERSION, Store},
};
use anyhow::{Context, Result, ensure};
use rusqlite::{OptionalExtension, params};

impl Store {
    /// Legacy Native6 must reject protected Workflow or accepted-Goal scopes
    /// before helpers. Absence/malformed/oversize/layout drift also refuses.
    /// This is a bounded body/index check, not full canonical planning/hashing
    /// under SharedStore, and it cannot mint a private phase protocol.
    pub(crate) fn managed_phase_required(&self, scope: &Scope) -> Result<bool> {
        let version = self.schema_version()?;
        ensure!(
            version == SCHEMA_VERSION,
            "managed legacy classification schema changed"
        );
        let goal = scope
            .goal_id
            .context("native classification requires Goal")?;
        let task = scope
            .task_id
            .context("native classification requires Task")?;
        let row=self.connection.query_row(
            "SELECT p.id,p.root,p.version,CASE WHEN length(CAST(p.body AS BLOB))<=?4 THEN p.body END,g.id,g.project_id,g.version,CASE WHEN length(CAST(g.body AS BLOB))<=?4 THEN g.body END,t.id,t.project_id,t.goal_id,t.issue,t.version,CASE WHEN length(CAST(t.body AS BLOB))<=?4 THEN t.body END,EXISTS(SELECT 1 FROM goal_authority a WHERE a.project_id=p.id AND a.goal_id=g.id),EXISTS(SELECT 1 FROM records r WHERE r.task_id=t.id AND r.kind='workflow') FROM tasks t JOIN goals g ON g.id=t.goal_id AND g.project_id=t.project_id JOIN projects p ON p.id=t.project_id WHERE t.id=?1 AND t.goal_id=?2 AND t.project_id=?3",
            params![task.to_string(),goal.to_string(),scope.project_id.to_string(),BODY_BYTES],
            |r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,u64>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,u64>(6)?,r.get::<_,Option<String>>(7)?,r.get::<_,String>(8)?,r.get::<_,String>(9)?,r.get::<_,String>(10)?,r.get::<_,Option<u64>>(11)?,r.get::<_,u64>(12)?,r.get::<_,Option<String>>(13)?,r.get::<_,bool>(14)?,r.get::<_,bool>(15)?)),
        ).optional()?.context("native classification owner unavailable")?;
        let p: Project = serde_json::from_value(decode_value(
            &row.3.context("native Project body over bound")?,
            BODY_BYTES,
        )?)
        .map_err(|_| anyhow::anyhow!("native Project body invalid"))?;
        let g: Goal = serde_json::from_value(decode_value(
            &row.7.context("native Goal body over bound")?,
            BODY_BYTES,
        )?)
        .map_err(|_| anyhow::anyhow!("native Goal body invalid"))?;
        let t: Task = serde_json::from_value(decode_value(
            &row.13.context("native Task body over bound")?,
            BODY_BYTES,
        )?)
        .map_err(|_| anyhow::anyhow!("native Task body invalid"))?;
        ensure!(
            p.id == scope.project_id
                && p.id.to_string() == row.0
                && p.root.to_str() == Some(row.1.as_str())
                && p.version == row.2
                && g.id == goal
                && g.id.to_string() == row.4
                && g.project_id == p.id
                && g.project_id.to_string() == row.5
                && g.version == row.6
                && t.id == task
                && t.id.to_string() == row.8
                && t.scope() == *scope
                && t.project_id.to_string() == row.9
                && t.goal_id.to_string() == row.10
                && t.issue == row.11
                && t.version == row.12,
            "native classification body/index identity differs"
        );
        // Supported current10 must have its actual contract layout. A missing
        // private table is an error, never absence enabling the legacy route.
        let contract = if version >= 10 {
            self.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM workflow_native_contracts WHERE task_id=?1)",
                [task.to_string()],
                |r| r.get::<_, bool>(0),
            )?
        } else {
            false
        };
        if row.14 || row.15 || contract {
            return Ok(true);
        }
        ensure!(
            p.state == ProjectState::Registered
                && matches!(
                    g.state,
                    GoalState::Created | GoalState::Analyzing | GoalState::Running
                )
                && !matches!(
                    t.state,
                    TaskState::Completed | TaskState::Failed | TaskState::Canceled
                ),
            "legacy native owner inactive"
        );
        Ok(false)
    }
}
