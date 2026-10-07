//! Atomic Driver snapshots and claims. Rows are content; only the retained registry supplies liveness.
use super::super::*;
use crate::{runtime::driver::DriverRegistry, workflow::WorkflowSnapshot};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Weak};
use uuid::Uuid;
mod candidates;
pub(crate) use candidates::{CandidateKey, CandidatePage};
mod claim;
pub(crate) use claim::{InitialDriverPlan, PendingDriverClaim, plan_initial_driver};
mod ticket;
pub(crate) use ticket::{DriverReadTicket, read_driver_ticket};
mod marker;
mod namespace;
mod preparation;
pub(in crate::state) use namespace::NamespaceSnapshot;
pub(crate) use preparation::{DriverPreparationAdvance, InitialGateEdge};
mod observation;
pub(crate) use marker::{DriverClosureAdvance, DriverMarkerAdvance, DriverPublication};
pub(crate) use observation::DriverExitPublication;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pin {
    version: u64,
    digest: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pins {
    scope: Scope,
    project: Pin,
    goal: Pin,
    task: Pin,
    workflow: Option<(RecordId, Pin)>,
    context: Option<Pin>,
    generation: u64,
    prerequisites: String,
    #[serde(default)]
    preparation: Option<PreparationPin>,
    #[serde(default)]
    source: Option<SourcePin>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PreparationPin {
    unit: crate::execution::UnitId,
    version: u64,
    digest: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourcePin {
    id: Uuid,
    version: u64,
    digest: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    id: Uuid,
    epoch: u64,
    version: u64,
    state: String,
    pins: Pins,
    #[serde(default)]
    marker: Option<MarkerAnchor>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MarkerAnchor {
    operation: crate::execution::OperationId,
    digest: String,
}
fn hash<T: Serialize>(body: &T) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(body)?)))
}
fn pin<T: Serialize>(version: u64, body: &T) -> Result<Pin> {
    Ok(Pin {
        version,
        digest: hash(body)?,
    })
}
fn bounded<T: DeserializeOwned>(c: &Connection, table: &str, id: &str, max: usize) -> Result<T> {
    ensure!(
        matches!(table, "projects" | "goals" | "tasks" | "records"),
        "Driver table invalid"
    );
    let size: usize = c.query_row(
        &format!("SELECT length(CAST(body AS BLOB)) FROM {table} WHERE id=?1"),
        [id],
        |r| r.get(0),
    )?;
    ensure!(size <= max, "Driver snapshot row exceeds bound");
    read_tx(c, table, id)?.context("Driver snapshot missing")
}
fn snapshot(c: &Connection, task_id: TaskId) -> Result<Pins> {
    let task: Task = bounded(c, "tasks", &task_id.to_string(), 1024 * 1024)?;
    let project: Project = bounded(c, "projects", &task.project_id.to_string(), 1024 * 1024)?;
    let goal: Goal = bounded(c, "goals", &task.goal_id.to_string(), 4 * 1024 * 1024)?;
    for (table, id, v) in [
        ("projects", project.id.to_string(), project.version),
        ("goals", goal.id.to_string(), goal.version),
        ("tasks", task.id.to_string(), task.version),
    ] {
        let actual: u64 = c.query_row(
            &format!("SELECT version FROM {table} WHERE id=?1"),
            [id],
            |r| r.get(0),
        )?;
        ensure!(actual == v, "Driver body/index version differs");
    }
    ensure!(
        task.id == task_id
            && task.project_id == project.id
            && task.goal_id == goal.id
            && goal.project_id == project.id,
        "Driver scope differs"
    );
    let accepted: String = c.query_row(
        "SELECT definition_sha256 FROM goal_authority WHERE goal_id=?1 AND project_id=?2",
        params![goal.id.to_string(), project.id.to_string()],
        |r| r.get(0),
    )?;
    let definition = crate::runtime::goal::GoalDefinition {
        title: goal.title.clone(),
        objective: goal.objective.clone(),
        criteria: goal
            .completion_criteria
            .iter()
            .map(|v| crate::runtime::goal::CriterionDefinition {
                id: v.id.clone(),
                description: v.description.clone(),
                evaluator: v.evaluator.clone(),
            })
            .collect(),
        constraints: goal.constraints.clone(),
        non_goals: goal.non_goals.clone(),
        source_refs: goal.source_refs.clone(),
    };
    ensure!(
        definition.canonical_digest()? == accepted,
        "accepted definition changed"
    );
    ensure!(
        project.state == ProjectState::Registered
            && goal.state == GoalState::Running
            && !task_terminal(task.state),
        "Driver lifecycle inactive"
    );
    goal.dag.hard_order()?;
    ensure!(
        goal.dag.nodes.contains(&task.id),
        "Task not accepted in Goal DAG"
    );
    let mut query=c.prepare("SELECT id,length(CAST(body AS BLOB)) FROM records WHERE task_id=?1 AND goal_id=?2 AND project_id=?3 AND kind='workflow' LIMIT 2")?;
    let ids = query
        .query_map(
            params![
                task.id.to_string(),
                goal.id.to_string(),
                project.id.to_string()
            ],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, usize>(1)?)),
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    ensure!(
        ids.len() <= 1 && ids.iter().all(|v| v.1 <= 8 * 1024 * 1024),
        "Driver Workflow ambiguous/overbound"
    );
    let workflow = if let Some((id, _)) = ids.first() {
        let record: Record = bounded(c, "records", id, 8 * 1024 * 1024)?;
        ensure!(
            record.scope == task.scope() && record.id.to_string() == *id,
            "Workflow indexed scope differs"
        );
        Some((record.id, pin(record.version, &record)?))
    } else {
        None
    };
    let latest: u64 = c.query_row(
        "SELECT COALESCE(MAX(version),0) FROM context_versions WHERE project_id=?1 AND owner=?2",
        params![project.id.to_string(), context_owner(&task.scope())?],
        |r| r.get(0),
    )?;
    ensure!(
        latest == task.context_version,
        "Driver Context head differs"
    );
    let context = if task.context_version > 0 {
        let body:String=c.query_row("SELECT body FROM context_versions WHERE project_id=?1 AND owner=?2 AND version=?3 AND length(CAST(body AS BLOB))<=8388608",params![project.id.to_string(),context_owner(&task.scope())?,task.context_version],|r|r.get(0))?;
        let cv: ContextVersion = decode(body)?;
        ensure!(
            cv.scope == task.scope() && cv.version == task.context_version,
            "Driver Context differs"
        );
        Some(pin(cv.version, &cv)?)
    } else {
        None
    };
    let generation: u64 = c
        .query_row(
            "SELECT generation FROM task_execution WHERE task_id=?1",
            [task.id.to_string()],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);
    let mut prerequisites = Vec::new();
    for (_, _, body, _, _, wbody) in claim::read_prerequisites(c, &task)? {
        let predecessor: Task = decode(body)?;
        let record: Record = decode(wbody)?;
        let w: WorkflowSnapshot = serde_json::from_value(record.data.clone())?;
        ensure!(
            w.finished
                && w.active.is_none()
                && w.configured_phases
                    .iter()
                    .all(|p| w.completed.contains_key(p)),
            "prerequisite requires actual complete Workflow evidence"
        );
        prerequisites.push((
            predecessor.id,
            pin(predecessor.version, &predecessor)?,
            pin(record.version, &record)?,
        ));
    }
    Ok(Pins {
        scope: task.scope(),
        project: pin(project.version, &project)?,
        goal: pin(goal.version, &goal)?,
        task: pin(task.version, &task)?,
        workflow,
        context,
        generation,
        prerequisites: hash(&prerequisites)?,
        preparation: preparation_anchor(c, &task, generation)?
            .map(|(unit, _)| -> Result<_> {
                Ok(PreparationPin {
                    unit: unit.id,
                    version: unit.version,
                    digest: hash(&unit)?,
                })
            })
            .transpose()?,
        source: crate::state::execution::source_recovery::driver_anchor(c, task.id)?.map(
            |(id, version, body)| SourcePin {
                id,
                version,
                digest: format!("{:x}", Sha256::digest(body.as_bytes())),
            },
        ),
    })
}

fn preparation_anchor(
    c: &Connection,
    task: &Task,
    generation: u64,
) -> Result<Option<(crate::execution::ExecutionUnit, String)>> {
    let indexed: Option<(u64, String)> = c.query_row(
        "SELECT generation,active_unit FROM task_execution WHERE task_id=?1 AND project_id=?2 AND goal_id=?3",
        params![task.id.to_string(), task.project_id.to_string(), task.goal_id.to_string()],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).optional()?;
    let Some((actual_generation, id)) = indexed else {
        ensure!(
            generation == 0,
            "Driver preparation generation index changed"
        );
        return Ok(None);
    };
    ensure!(
        actual_generation == generation && generation > 0,
        "Driver preparation generation differs"
    );
    let body: String = c.query_row(
        "SELECT CASE WHEN length(CAST(body AS BLOB))<=16384 THEN body END FROM execution_units WHERE id=?1",
        [&id], |r| r.get::<_, Option<String>>(0),
    )?.context("Driver preparation Unit exceeds bound")?;
    let unit = crate::state::execution::unit_tx(c, id.parse()?)?;
    ensure!(
        unit.scope == task.scope()
            && unit.kind == crate::execution::UnitKind::Executor
            && unit.generation == generation,
        "Driver preparation scope/generation/identity differs"
    );
    Ok(Some((unit, body)))
}
pub(in crate::state) fn register_liveness(
    connection: &Connection,
    registry: Weak<DriverRegistry>,
) -> Result<()> {
    connection.create_scalar_function(
        "rrx_live_task_driver",
        5,
        rusqlite::functions::FunctionFlags::SQLITE_UTF8
            | rusqlite::functions::FunctionFlags::SQLITE_INNOCUOUS,
        move |ctx| {
            let task: String = ctx.get(0)?;
            let id: String = ctx.get(1)?;
            let epoch: u64 = ctx.get(2)?;
            let version: u64 = ctx.get(3)?;
            let body: String = ctx.get(4)?;
            if body.len() > 128 * 1024 {
                return Ok(false);
            }
            let Some(registry) = registry.upgrade() else {
                return Ok(false);
            };
            Ok(match (task.parse(), id.parse()) {
                (Ok(t), Ok(id)) => registry.is_current(t, id, epoch, version, &body),
                _ => false,
            })
        },
    )?;
    Ok(())
}
pub(in crate::state) fn validate(c: &Connection, task_id: TaskId) -> Result<()> {
    let managed:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM tasks t JOIN goal_authority a ON a.goal_id=t.goal_id AND a.project_id=t.project_id WHERE t.id=?1)",[task_id.to_string()],|r|r.get(0))?;
    if !managed {
        return Ok(());
    }
    let(id,project,goal,epoch,version,body):(String,String,String,u64,u64,String)=c.query_row("SELECT id,project_id,goal_id,owner_epoch,version,body FROM task_drivers WHERE task_id=?1 AND state='driving' AND length(CAST(body AS BLOB))<=131072",[task_id.to_string()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?)))?;
    let current_epoch: u64 = c.query_row(
        "SELECT epoch FROM runtime_epoch WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    ensure!(epoch > 0 && epoch == current_epoch, "Driver epoch retired");
    let live: bool = c.query_row(
        "SELECT rrx_live_task_driver(?1,?2,?3,?4,?5)",
        params![task_id.to_string(), id, epoch, version, body],
        |r| r.get(0),
    )?;
    ensure!(live, "actual retained Task Driver unavailable");
    let row: Row = decode(body)?;
    ensure!(
        row.marker.is_none(),
        "marker-bound Driver requires the genuine managed successor/lifecycle reader"
    );
    let current = snapshot(c, task_id)?;
    ensure!(
        row.id.to_string() == id
            && row.pins.scope.project_id.to_string() == project
            && row.pins.scope.goal_id.map(|id| id.to_string()).as_deref() == Some(goal.as_str())
            && row.epoch == epoch
            && row.version == version
            && row.state == "driving"
            && serde_json::to_value(&row.pins)? == serde_json::to_value(current)?,
        "Driver authority snapshot changed"
    );
    Ok(())
}
impl Store {
    pub(crate) fn attach_runtime_drivers(&self, registry: &Arc<DriverRegistry>) -> Result<()> {
        register_liveness(&self.connection, Arc::downgrade(registry))
    }
}

pub(super) fn invalidate_tx(
    permits: &crate::state::managed_binding::PrivatePermitManager,
    tx: &Transaction<'_>,
    task: TaskId,
    id: &str,
    epoch: u64,
    version: u64,
    body: &str,
) -> Result<()> {
    ensure!(body.len() <= 128 * 1024, "Driver metadata exceeds bound");
    let mut row: Row = decode(body.to_owned())?;
    ensure!(
        row.id.to_string() == id
            && row.epoch == epoch
            && row.version == version
            && row.state == "driving"
            && row.pins.scope.task_id == Some(task),
        "Driver invalidation identity differs"
    );
    row.state = "invalid".into();
    row.version = version
        .checked_add(1)
        .filter(|v| *v <= i64::MAX as u64)
        .context("Driver version exhausted")?;
    let next_body = serde_json::to_string(&row)?;
    ensure!(
        next_body.len() <= 128 * 1024,
        "Driver metadata exceeds bound"
    );
    let indexed: (String, String) = tx.query_row(
        "SELECT goal_id,project_id FROM task_drivers WHERE task_id=?1 AND id=?2 AND owner_epoch=?3 AND version=?4 AND state='driving' AND body=?5",
        params![task.to_string(), id, epoch, version, body],
        |r| Ok((r.get(0)?,r.get(1)?)),
    )?;
    ensure!(
        row.pins.scope.goal_id.map(|v| v.to_string()).as_deref() == Some(indexed.0.as_str())
            && row.pins.scope.project_id.to_string() == indexed.1,
        "Driver invalidation indexed scope differs"
    );
    use rusqlite::types::Value as SqlValue;
    let columns = |version: u64, state: &str, body: &str| -> Result<Vec<SqlValue>> {
        Ok(vec![
            task.to_string().into(),
            indexed.0.clone().into(),
            indexed.1.clone().into(),
            id.to_owned().into(),
            i64::try_from(epoch)?.into(),
            i64::try_from(version)?.into(),
            state.to_owned().into(),
            body.to_owned().into(),
        ])
    };
    let mutation = crate::state::managed_binding::ExactRowMutation::new(
        "task_drivers",
        "UPDATE",
        Some(columns(version, "driving", body)?),
        Some(columns(row.version, "invalid", &next_body)?),
    )?;
    permits.with_exact_permit(vec![mutation], || {
    ensure!(tx.execute("UPDATE task_drivers SET state='invalid',version=?1,body=?2 WHERE task_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='driving'",params![row.version,next_body,task.to_string(),id,epoch,version])?==1,"Driver invalidation CAS differs");
    permits.ensure_consumed()
    })?;
    append_event(
        tx,
        &row.pins.scope,
        "rrx.private.runtime.driver_invalidated",
        json!({"driver":id,"epoch":epoch,"version":row.version}),
    )?;
    Ok(())
}

impl Store {
    /// Legacy helpers consume this exact bounded Task snapshot. Classification
    /// and subsequent effects must not use different body/index identities.
    pub(crate) fn legacy_worktree_task(&self, id: TaskId) -> Result<Task> {
        let indexed = self.connection.query_row(
            "SELECT t.id,t.project_id,t.goal_id,t.version,CASE WHEN length(CAST(t.body AS BLOB))<=1048576 THEN t.body END,EXISTS(SELECT 1 FROM goal_authority a WHERE a.goal_id=t.goal_id AND a.project_id=t.project_id) FROM tasks t WHERE t.id=?1",
            [id.to_string()],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, u64>(3)?, row.get::<_, Option<String>>(4)?, row.get::<_, bool>(5)?)),
        ).optional()?.context("unknown legacy worktree Task")?;
        let body = indexed
            .4
            .context("legacy worktree Task exceeds body bound")?;
        let task: Task = decode(body)?;
        ensure!(
            task.id == id
                && indexed.0 == id.to_string()
                && indexed.1 == task.project_id.to_string()
                && indexed.2 == task.goal_id.to_string()
                && indexed.3 == task.version,
            "legacy worktree Task body/index identity differs"
        );
        ensure!(
            !indexed.5,
            "accepted Goal worktree helpers require the unavailable managed Driver/binding producer"
        );
        Ok(task)
    }
}

#[cfg(test)]
impl Store {
    /// Read-only observer of the generic Driver reader; grants nothing.
    pub(crate) fn validate_task_driver(&self, task: TaskId) -> Result<()> {
        validate(&self.connection, task)
    }
}
