//! Coherent, query-only plans outside the writer mutex. Plans grant no effects.
use super::canonical::{BODY_BYTES, Body, WORKFLOW_DOMAIN};
use crate::{
    domain::*,
    execution::RuntimeOwner,
    state::{APPLICATION_ID, SCHEMA_VERSION},
    workflow::WorkflowSnapshot,
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, params};
use std::time::Duration;

const LOCK_BYTES: usize = 16 * 1024;
const LOCK_ROWS: usize = 256;

/// Nongrant current read plan. The original marker producer must validate this
/// exact plan inside its own transaction; current reads cannot replace its pins.
pub(crate) struct ScopePlan {
    instance: String,
    epoch: u64,
    pub(super) project: Body<Project>,
    pub(super) goal: Body<Goal>,
    pub(super) task: Body<Task>,
    pub(super) workflow: Option<Body<Record>>,
    pub(super) context: Option<Body<ContextVersion>>,
    pub(super) locks: Vec<Body<Record>>,
}

pub(in crate::state) fn snapshot<R>(
    owner: &RuntimeOwner,
    read: impl FnOnce(&Transaction<'_>) -> Result<R>,
) -> Result<R> {
    let mut connection = Connection::open_with_flags(
        owner.state_path(),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.busy_timeout(Duration::from_millis(250))?;
    connection.pragma_update(None, "query_only", true)?;
    let tx = connection.transaction()?;
    ensure!(
        tx.pragma_query_value(None, "application_id", |r| r.get::<_, i64>(0))? == APPLICATION_ID
            && tx.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))?
                == SCHEMA_VERSION,
        "managed snapshot state contract unavailable"
    );
    let (instance, epoch): (String, u64) = tx.query_row(
        "SELECT instance_id,epoch FROM runtime_epoch WHERE singleton=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    ensure!(
        instance == owner.instance_id() && epoch == owner.epoch() && epoch > 0,
        "managed snapshot owner retired"
    );
    let result = read(&tx)?;
    tx.commit()?;
    Ok(result)
}

fn bounded(c: &Connection, table: &str, id: &str, limit: usize) -> Result<String> {
    ensure!(
        matches!(table, "projects" | "goals" | "tasks" | "records"),
        "invalid managed snapshot table"
    );
    c.query_row(
        &format!(
            "SELECT CASE WHEN length(CAST(body AS BLOB))<=?2 THEN body END FROM {table} WHERE id=?1"
        ),
        params![id, limit],
        |r| r.get::<_, Option<String>>(0),
    )?
    .context("managed snapshot body over bound")
}

fn owners(c: &Connection, scope: &Scope) -> Result<(Body<Project>, Body<Goal>, Body<Task>)> {
    let goal_id = scope.goal_id.context("managed snapshot needs Goal")?;
    let task_id = scope.task_id.context("managed snapshot needs Task")?;
    let project = Body::<Project>::decode(
        bounded(c, "projects", &scope.project_id.to_string(), BODY_BYTES)?,
        BODY_BYTES,
    )?;
    let goal = Body::<Goal>::decode(
        bounded(c, "goals", &goal_id.to_string(), BODY_BYTES)?,
        BODY_BYTES,
    )?;
    let task = Body::<Task>::decode(
        bounded(c, "tasks", &task_id.to_string(), BODY_BYTES)?,
        BODY_BYTES,
    )?;
    let (p, g, t) = (project.parsed(), goal.parsed(), task.parsed());
    ensure!(
        p.id == scope.project_id
            && g.id == goal_id
            && g.project_id == p.id
            && t.id == task_id
            && t.scope() == *scope,
        "managed snapshot foreign body identity"
    );
    let indexed: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM projects p JOIN goals g ON g.project_id=p.id JOIN tasks t ON t.goal_id=g.id AND t.project_id=p.id WHERE p.id=?1 AND g.id=?2 AND t.id=?3 AND p.root=?4 AND p.version=?5 AND g.version=?6 AND t.version=?7 AND t.issue IS ?8 AND p.body=?9 AND g.body=?10 AND t.body=?11)",
        params![p.id.to_string(),g.id.to_string(),t.id.to_string(),p.root.to_str().context("Project path not UTF-8")?,p.version,g.version,t.version,t.issue,project.raw(),goal.raw(),task.raw()],|r|r.get(0))?;
    ensure!(
        indexed,
        "managed snapshot owner index/body identity differs"
    );
    Ok((project, goal, task))
}

fn scoped_records(
    c: &Connection,
    scope: &Scope,
    kind: RecordKind,
    limit: usize,
    bytes: usize,
) -> Result<Vec<Body<Record>>> {
    let mut statement = c.prepare("SELECT id,kind,project_id,goal_id,task_id,version,CASE WHEN length(CAST(body AS BLOB))<=?5 THEN body END FROM records WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind=?4 ORDER BY id LIMIT ?6")?;
    let rows = statement
        .query_map(
            params![
                scope.project_id.to_string(),
                scope.goal_id.map(|v| v.to_string()),
                scope.task_id.map(|v| v.to_string()),
                kind.key(),
                bytes,
                limit + 1
            ],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, u64>(5)?,
                    r.get::<_, Option<String>>(6)?,
                ))
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    ensure!(
        rows.len() <= limit,
        "complete scoped record inventory exceeds bound"
    );
    rows.into_iter()
        .map(|(id, key, p, g, t, version, raw)| {
            let body = Body::<Record>::decode(
                raw.context("managed scoped record exceeds byte bound")?,
                bytes,
            )?;
            let record = body.parsed();
            ensure!(
                record.id.to_string() == id
                    && record.kind == kind
                    && record.kind.key() == key
                    && record.scope == *scope
                    && record.scope.project_id.to_string() == p
                    && record.scope.goal_id.map(|v| v.to_string()) == g
                    && record.scope.task_id.map(|v| v.to_string()) == t
                    && record.version == version,
                "managed scoped record index/body identity differs"
            );
            Ok(body)
        })
        .collect()
}

/// Complete owners, exactly-zero-or-one Workflow, latest Context and all locks
/// are read in ONE transaction. Hashes include complete bodies and are produced
/// here, before any writer mutex is acquired. Zero Workflow is bootstrap data,
/// not evidence permitting native launch; the marker producer requires one.
pub(crate) fn plan_scope(owner: &RuntimeOwner, scope: &Scope) -> Result<ScopePlan> {
    snapshot(owner, |tx| read_scope(tx, owner, scope))
}

/// Shared only with the marker planner so Unit and scope pins are read in the
/// same transaction. This remains nongrant content, including zero Workflow.
pub(in crate::state) fn read_scope(
    tx: &Transaction<'_>,
    owner: &RuntimeOwner,
    scope: &Scope,
) -> Result<ScopePlan> {
    let (project, goal, task) = owners(tx, scope)?;
    let mut workflows = scoped_records(tx, scope, RecordKind::Workflow, 1, BODY_BYTES)?;
    let workflow = workflows.pop();
    if let Some(body) = &workflow {
        // Record.data is a generic Value; validate the ENTIRE inner typed
        // Workflow as well, so unknown fields cannot disappear in projection.
        Body::<WorkflowSnapshot>::decode(serde_json::to_string(&body.parsed().data)?, BODY_BYTES)?;
        body.digest(WORKFLOW_DOMAIN);
    }
    let owner_key = super::super::context_owner(scope)?;
    let row=tx.query_row("SELECT project_id,goal_id,task_id,owner,version,CASE WHEN length(CAST(body AS BLOB))<=?3 THEN body END FROM context_versions WHERE project_id=?1 AND owner=?2 ORDER BY version DESC LIMIT 1",params![scope.project_id.to_string(),owner_key,BODY_BYTES],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<String>>(2)?,r.get::<_,String>(3)?,r.get::<_,u64>(4)?,r.get::<_,Option<String>>(5)?))).optional()?;
    let context = row
        .map(|(p, g, t, key, version, raw)| -> Result<_> {
            let body = Body::<ContextVersion>::decode(
                raw.context("managed Context over bound")?,
                BODY_BYTES,
            )?;
            let context = body.parsed();
            ensure!(
                context.scope == *scope
                    && p == scope.project_id.to_string()
                    && Some(g) == scope.goal_id.map(|v| v.to_string())
                    && t == scope.task_id.map(|v| v.to_string())
                    && key == owner_key
                    && context.version == version,
                "managed Context index/body identity differs"
            );
            Ok(body)
        })
        .transpose()?;
    let locks = scoped_records(tx, scope, RecordKind::WorktreeLock, LOCK_ROWS, LOCK_BYTES)?;
    for body in &locks {
        Body::<crate::git::WorktreeLock>::decode(
            serde_json::to_string(&body.parsed().data)?,
            LOCK_BYTES,
        )?;
    }
    Ok(ScopePlan {
        instance: owner.instance_id().into(),
        epoch: owner.epoch(),
        project,
        goal,
        task,
        workflow,
        context,
        locks,
    })
}

impl ScopePlan {
    pub(in crate::state) fn governing_owners(&self) -> (&Project, &Goal) {
        (self.project.parsed(), self.goal.parsed())
    }
    pub(in crate::state) fn task(&self) -> &Task {
        self.task.parsed()
    }
    pub(in crate::state) fn has_input_history(&self) -> bool {
        self.workflow.is_some() || self.context.is_some()
    }
    pub(in crate::state) fn matches_marker(
        &self,
        marker: &super::marker_plan::ManagedMarkerPlan,
    ) -> Result<()> {
        ensure!(
            marker.scope() == self.task.parsed().scope()
                && marker.project().1 == self.project.raw()
                && marker.goal().1 == self.goal.raw()
                && marker.task_before().1 == self.task.raw()
                && self
                    .workflow
                    .as_ref()
                    .is_some_and(|w| w.raw() == marker.workflow_before().1)
                && self
                    .context
                    .as_ref()
                    .is_some_and(|c| c.raw() == marker.context().1),
            "Driver/marker original scope frame differs"
        );
        Ok(())
    }

    /// Current CAS is exact encoded bytes plus metadata and COMPLETE inventories;
    /// it does not hash bodies, advance pins, mutate state or grant native work.
    pub(in crate::state) fn validate_current(&self, c: &Connection) -> Result<()> {
        self.validate_projection(
            c,
            self.task.parsed(),
            self.task.raw(),
            self.workflow.as_ref().map(|w| (w.parsed(), w.raw())),
        )
    }
    /// Nongrant exact resulting-row check. The caller's actual sealed producer
    /// validates the permitted old/new projection outside the write mutex.
    pub(in crate::state) fn validate_projection(
        &self,
        c: &Connection,
        task: &Task,
        task_raw: &str,
        workflow: Option<(&Record, &str)>,
    ) -> Result<()> {
        self.validate_projection_context(
            c,
            task,
            task_raw,
            workflow,
            self.context.as_ref().map(|b| (b.parsed(), b.raw())),
        )
    }
    /// Only a sealed initial-input producer may prescribe absence -> first
    /// Context. Original owner/lock bytes and ordinary projection remain exact.
    pub(in crate::state) fn validate_input_projection(
        &self,
        c: &Connection,
        task: &Task,
        task_raw: &str,
        workflow: (&Record, &str),
        context: (&ContextVersion, &str),
    ) -> Result<()> {
        ensure!(
            self.workflow.is_none()
                && self.context.is_none()
                && context.0.version == 1
                && task.context_version == 1,
            "initial input projection requires original absence"
        );
        self.validate_projection_context(c, task, task_raw, Some(workflow), Some(context))
    }
    fn validate_projection_context(
        &self,
        c: &Connection,
        task: &Task,
        task_raw: &str,
        workflow: Option<(&Record, &str)>,
        context: Option<(&ContextVersion, &str)>,
    ) -> Result<()> {
        let scope = self.task.parsed().scope();
        ensure!(
            task.scope() == scope
                && workflow
                    .as_ref()
                    .is_none_or(|(w, _)| w.scope == scope && w.kind == RecordKind::Workflow),
            "Driver planned row scope differs"
        );
        let (p, g, t) = (self.project.parsed(), self.goal.parsed(), task);
        let current:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM runtime_epoch e JOIN projects p ON p.id=?1 JOIN goals g ON g.id=?2 AND g.project_id=p.id JOIN tasks t ON t.id=?3 AND t.project_id=p.id AND t.goal_id=g.id WHERE e.singleton=1 AND e.instance_id=?4 AND e.epoch=?5 AND p.root=?6 AND p.version=?7 AND g.version=?8 AND t.version=?9 AND t.issue IS ?10 AND p.body=?11 AND g.body=?12 AND t.body=?13)",params![p.id.to_string(),g.id.to_string(),t.id.to_string(),self.instance,self.epoch,p.root.to_str().context("Project path not UTF-8")?,p.version,g.version,t.version,t.issue,self.project.raw(),self.goal.raw(),task_raw],|r|r.get(0))?;
        ensure!(current, "managed original owners changed");
        let workflows = exact_records(c, &scope, RecordKind::Workflow, 1, BODY_BYTES)?;
        let expected = workflow
            .iter()
            .map(|(v, raw)| (v.id.to_string(), v.version, (*raw).to_owned()))
            .collect::<Vec<_>>();
        ensure!(workflows == expected, "managed Workflow changed");
        let locks = exact_records(c, &scope, RecordKind::WorktreeLock, LOCK_ROWS, LOCK_BYTES)?;
        let expected = self
            .locks
            .iter()
            .map(|v| {
                (
                    v.parsed().id.to_string(),
                    v.parsed().version,
                    v.raw().to_owned(),
                )
            })
            .collect::<Vec<_>>();
        ensure!(locks == expected, "managed complete lock set changed");
        let owner_key = super::super::context_owner(&scope)?;
        let actual=c.query_row("SELECT version,CASE WHEN length(CAST(body AS BLOB))<=?3 THEN body END FROM context_versions WHERE project_id=?1 AND owner=?2 ORDER BY version DESC LIMIT 1",params![scope.project_id.to_string(),owner_key,BODY_BYTES],|r|Ok((r.get::<_,u64>(0)?,r.get::<_,Option<String>>(1)?))).optional()?;
        let expected = context.map(|(b, raw)| (b.version, Some(raw.to_owned())));
        let _ = (&actual, &expected); // causal mutation: omit latest Context head equality
        if let Some((context, raw)) = context {
            ensure!(
                context.scope == scope,
                "managed Context planned scope differs"
            );
            let indexed:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM context_versions WHERE project_id=?1 AND goal_id=?2 AND task_id IS ?3 AND owner=?4 AND version=?5 AND body=?6)",params![scope.project_id.to_string(),scope.goal_id.map(|v|v.to_string()),scope.task_id.map(|v|v.to_string()),owner_key,context.version,raw],|r|r.get(0))?;
            ensure!(indexed, "managed Context index changed");
        }
        Ok(())
    }
}

fn exact_records(
    c: &Connection,
    scope: &Scope,
    kind: RecordKind,
    limit: usize,
    bytes: usize,
) -> Result<Vec<(String, u64, String)>> {
    let mut statement=c.prepare("SELECT id,version,CASE WHEN length(CAST(body AS BLOB))<=?5 THEN body END FROM records WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind=?4 ORDER BY id LIMIT ?6")?;
    let rows = statement
        .query_map(
            params![
                scope.project_id.to_string(),
                scope.goal_id.map(|v| v.to_string()),
                scope.task_id.map(|v| v.to_string()),
                kind.key(),
                bytes,
                limit + 1
            ],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, u64>(1)?,
                    r.get::<_, Option<String>>(2)?,
                ))
            },
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    ensure!(rows.len() <= limit, "managed current inventory over bound");
    rows.into_iter()
        .map(|(id, version, body)| {
            Ok((
                id,
                version,
                body.context("managed current body over bound")?,
            ))
        })
        .collect()
}
