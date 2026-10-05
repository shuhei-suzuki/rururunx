//! Transactional execution authority. No Git/native/process I/O occurs here.
use super::*;
use crate::execution::{model::key, *};
use std::collections::BTreeMap;
use uuid::Uuid;

const MUTABLE_TABLES: &[&str] = &[
    "projects",
    "goals",
    "tasks",
    "records",
    "context_versions",
    "usage",
    "audit",
    "runtime_epoch",
    "execution_units",
    "execution_context",
    "task_execution",
    "session_units",
    "result_artifacts",
    "artifact_dependencies",
    "resource_leases",
    "managed_effects",
    "cleanup_jobs",
    "cleanup_observations",
    "quota_pools",
    "quota_windows",
    "quota_leases",
    "quota_waiters",
];

pub(super) fn install_schema(tx: &Transaction<'_>) -> Result<()> {
    migration::validate_legacy(tx)?;
    tx.execute_batch(include_str!("execution.sql"))?;
    tx.execute(
        "INSERT INTO runtime_epoch(singleton,instance_id,epoch) VALUES(1,?1,0)",
        [Uuid::new_v4().to_string()],
    )?;
    // This guard is connection-local, not a persisted flag a pre-open old client inherits.
    for table in MUTABLE_TABLES {
        for action in ["INSERT", "UPDATE", "DELETE"] {
            tx.execute_batch(&format!(
                "CREATE TRIGGER writer_{table}_{action} BEFORE {action} ON {table} \
                 WHEN rrx_writer_contract_version()<>4 BEGIN \
                 SELECT RAISE(ABORT,'incompatible rrx writer contract'); END;"
            ))?;
        }
    }
    // Register historical paths without promoting native liveness or work success.
    let mut tasks = tx.prepare("SELECT body FROM tasks")?;
    let old = tasks
        .query_map([], |r| r.get::<_, String>(0))?
        .map(|r| r.map_err(anyhow::Error::from).and_then(decode::<Task>))
        .collect::<Result<Vec<_>>>()?;
    drop(tasks);
    for task in old {
        if let Some(path) = &task.worktree {
            let at = now_ms();
            let unit = ExecutionUnit {
                id: UnitId::new(),
                scope: task.scope(),
                kind: UnitKind::Legacy,
                generation: 0,
                owner_epoch: 0,
                version: 1,
                phase: task.phase.clone().unwrap_or_default(),
                provider: task.executor.clone(),
                state: UnitState::LegacyUnreconciled,
                native_effects_open: false,
                result_finalization_open: false,
                work: Some(WorkOutcome::Unknown),
                cleanup: CleanupOutcome::Unknown,
                disposition: Disposition::Legacy,
                worktree: path.clone(),
                branch: task.branch.clone(),
                base_sha: task.revision.clone().unwrap_or_default(),
                profile_digest: String::new(),
                cookie: String::new(),
                session_id: None,
                artifact_id: None,
                wait_reason: Some(WaitReason::ExternalOutcome),
                created_at: at,
                updated_at: at,
            };
            insert_unit(tx, &unit)?;
            tx.execute("INSERT INTO task_execution(task_id,project_id,goal_id,generation,active_unit) VALUES(?1,?2,?3,0,NULL)",
                params![task.id.to_string(),task.project_id.to_string(),task.goal_id.to_string()])?;
            let lease = ResourceLease {
                id: LeaseId::new(),
                unit_id: unit.id,
                scope: unit.scope.clone(),
                kind: ResourceKind::Worktree,
                namespace: task.project_id.to_string(),
                value: path.to_string_lossy().into_owned(),
                port_start: None,
                port_end: None,
                state: LeaseState::Quarantined,
                version: 1,
            };
            insert_lease(tx, &lease)?;
        }
    }
    Ok(())
}

fn unit_tx(tx: &Connection, id: UnitId) -> Result<ExecutionUnit> {
    let (body,version,scope,kind,generation,epoch,native,finalize):(String,u64,(String,String,String),String,u64,u64,bool,bool)=tx.query_row(
        "SELECT body,version,project_id,goal_id,task_id,kind,generation,owner_epoch,native_effects_open,result_finalization_open FROM execution_units WHERE id=?1",
        [id.to_string()],|r| Ok((r.get(0)?,r.get(1)?,(r.get(2)?,r.get(3)?,r.get(4)?),r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?)))?;
    let mut unit: ExecutionUnit = decode(body)?;
    ensure!(
        unit.id == id
            && unit.version == version
            && scope == scope_keys(&unit.scope)?
            && kind == key(unit.kind)
            && generation == unit.generation
            && epoch == unit.owner_epoch
            && native == unit.native_effects_open
            && finalize == unit.result_finalization_open,
        "execution unit indexed/body identity mismatch"
    );
    check_indexed(
        tx,
        "execution_units",
        &[
            ("id", json!(id)),
            ("worktree", json!(unit.worktree)),
            ("branch", json!(unit.branch)),
        ],
    )?;
    // Cleanup is an independent factual projection. Appending a janitor receipt
    // cannot supersede native/finalization authority or invalidate a capture CAS.
    if let Some((at, body)) = tx
        .query_row(
            "SELECT at,body FROM cleanup_observations WHERE unit_id=?1 ORDER BY rowid DESC LIMIT 1",
            [id.to_string()],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?
    {
        let observation: CleanupObservation = decode(body)?;
        ensure!(
            observation.unit_id == id && observation.at == at,
            "cleanup observation indexed/body mismatch"
        );
        unit.cleanup = observation.outcome;
    }
    Ok(unit)
}
fn check_indexed(connection: &Connection, table: &str, columns: &[(&str, Value)]) -> Result<()> {
    let mut values = Vec::new();
    let mut clauses = Vec::new();
    for (name, value) in columns {
        clauses.push(format!("{name} IS ?"));
        values.push(match value {
            Value::Null => rusqlite::types::Value::Null,
            Value::String(s) => rusqlite::types::Value::Text(s.clone()),
            Value::Bool(v) => rusqlite::types::Value::Integer(i64::from(*v)),
            Value::Number(n) => {
                rusqlite::types::Value::Integer(n.as_i64().context("invalid indexed integer")?)
            }
            _ => anyhow::bail!("invalid indexed value"),
        });
    }
    let matched: bool = connection.query_row(
        &format!(
            "SELECT EXISTS(SELECT 1 FROM {table} WHERE {})",
            clauses.join(" AND ")
        ),
        rusqlite::params_from_iter(values),
        |r| r.get(0),
    )?;
    ensure!(matched, "{table} indexed/body mismatch");
    Ok(())
}
fn scoped_columns(scope: &Scope) -> Vec<(&'static str, Value)> {
    vec![
        ("project_id", json!(scope.project_id)),
        ("goal_id", json!(scope.goal_id)),
        ("task_id", json!(scope.task_id)),
    ]
}
fn scope_keys(scope: &Scope) -> Result<(String, String, String)> {
    Ok((
        scope.project_id.to_string(),
        scope.goal_id.context("unit requires Goal")?.to_string(),
        scope.task_id.context("unit requires Task")?.to_string(),
    ))
}
fn insert_unit(tx: &Transaction<'_>, unit: &ExecutionUnit) -> Result<()> {
    let (p, g, t) = scope_keys(&unit.scope)?;
    tx.execute("INSERT INTO execution_units(id,project_id,goal_id,task_id,kind,generation,owner_epoch,version,native_effects_open,result_finalization_open,worktree,branch,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
        params![unit.id.to_string(),p,g,t,key(unit.kind),unit.generation,unit.owner_epoch,unit.version,unit.native_effects_open,unit.result_finalization_open,unit.worktree.to_string_lossy(),unit.branch,serde_json::to_string(unit)?])?;
    Ok(())
}
fn write_unit(tx: &Transaction<'_>, unit: &mut ExecutionUnit) -> Result<()> {
    let old = unit.version;
    bump(&mut unit.version)?;
    unit.updated_at = now_ms();
    let changed=tx.execute("UPDATE execution_units SET version=?1,native_effects_open=?2,result_finalization_open=?3,body=?4 WHERE id=?5 AND version=?6",
        params![unit.version,unit.native_effects_open,unit.result_finalization_open,serde_json::to_string(unit)?,unit.id.to_string(),old])?;
    ensure!(changed == 1, "execution unit CAS conflict");
    Ok(())
}
pub(super) fn fence_task_tx(tx: &Transaction<'_>, scope: &Scope) -> Result<()> {
    let (_, _, task) = scope_keys(scope)?;
    let mut s=tx.prepare("SELECT id FROM execution_units WHERE task_id=?1 AND (native_effects_open=1 OR result_finalization_open=1)")?;
    let ids = s
        .query_map([&task], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(s);
    for id in ids {
        let mut unit = unit_tx(tx, id.parse()?)?;
        ensure!(unit.scope == *scope, "Task fence scope mismatch");
        unit.native_effects_open = false;
        unit.result_finalization_open = false;
        unit.state = UnitState::Retired;
        unit.disposition = Disposition::Cancelled;
        if unit.work.is_none() {
            unit.work = Some(WorkOutcome::Unknown);
        }
        write_unit(tx, &mut unit)?;
        quotas::release_quota_tx(tx, unit.id)?;
        tx.execute(
            "DELETE FROM quota_waiters WHERE unit_id=?1",
            [unit.id.to_string()],
        )?;
        tx.execute("INSERT INTO cleanup_jobs(unit_id,next_due,attempts,version) VALUES(?1,?2,0,1) ON CONFLICT(unit_id) DO NOTHING",params![unit.id.to_string(),now_ms()])?;
        append_event(
            tx,
            scope,
            "execution.task_fenced",
            json!({"unit":unit.id,"work":unit.work}),
        )?;
    }
    tx.execute("UPDATE task_execution SET generation=generation+1,active_unit=NULL WHERE task_id=?1 AND active_unit IS NOT NULL",[task])?;
    Ok(())
}
fn validate_authority(
    tx: &Connection,
    authority: &ExecutionAuthority,
    native: bool,
    finalize: bool,
) -> Result<ExecutionUnit> {
    let unit = unit_tx(tx, authority.unit_id)?;
    ensure!(
        unit.authority() == *authority,
        "execution authority identity mismatch"
    );
    let epoch: u64 = tx.query_row(
        "SELECT epoch FROM runtime_epoch WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    ensure!(
        epoch == authority.owner_epoch && epoch > 0,
        "execution owner epoch retired"
    );
    let generation: u64 = tx.query_row(
        "SELECT generation FROM task_execution WHERE task_id=?1",
        [authority
            .scope
            .task_id
            .context("Task required")?
            .to_string()],
        |r| r.get(0),
    )?;
    ensure!(
        generation == authority.generation,
        "execution generation retired"
    );
    ensure!(
        !native || unit.native_effects_open,
        "native effect permission closed"
    );
    ensure!(
        !finalize || unit.result_finalization_open,
        "result finalization permission closed"
    );
    if native || finalize {
        let project: Project = read_tx(tx, "projects", &unit.scope.project_id.to_string())?
            .context("unknown Project")?;
        let goal: Goal = read_tx(
            tx,
            "goals",
            &unit.scope.goal_id.context("Goal required")?.to_string(),
        )?
        .context("unknown Goal")?;
        let task: Task = read_tx(
            tx,
            "tasks",
            &unit.scope.task_id.context("Task required")?.to_string(),
        )?
        .context("unknown Task")?;
        ensure!(
            task.scope() == unit.scope && !task_terminal(task.state),
            "inactive/foreign Task"
        );
        ensure!(
            project.state == ProjectState::Registered
                && !matches!(
                    goal.state,
                    GoalState::Paused
                        | GoalState::Completed
                        | GoalState::Cancelled
                        | GoalState::Failed
                ),
            "inactive Project/Goal"
        );
        let saved: String = tx.query_row(
            "SELECT governing_digest FROM execution_context WHERE unit_id=?1",
            [unit.id.to_string()],
            |r| r.get(0),
        )?;
        ensure!(
            saved == governing_digest(&project, &goal)?,
            "governing instructions changed; fresh admission required"
        );
        if unit.kind == UnitKind::Executor {
            ensure!(
                task.worktree.as_ref() == Some(&unit.worktree) && task.branch == unit.branch,
                "executor projection changed"
            );
        }
    }
    Ok(unit)
}
fn governing_digest(project: &Project, goal: &Goal) -> Result<String> {
    use sha2::{Digest, Sha256};
    // Scheduling/status/evidence bookkeeping does not rewrite the accepted instruction frame.
    let criteria = goal
        .completion_criteria
        .iter()
        .map(|c| (&c.id, &c.description))
        .collect::<Vec<_>>();
    let frame = json!({"project":[project.id,project.root,project.repository_identity,project.base_branch,
        project.config_ref,project.rule_refs,project.environment_refs],
        "goal":[goal.id,goal.project_id,goal.title,goal.objective,criteria,goal.constraints,
        goal.non_goals,goal.source_refs,goal.context_version,goal.dag]});
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(&frame)?)))
}
fn insert_lease(tx: &Transaction<'_>, lease: &ResourceLease) -> Result<()> {
    let (p, g, t) = scope_keys(&lease.scope)?;
    tx.execute("INSERT INTO resource_leases(id,unit_id,project_id,goal_id,task_id,kind,namespace,value,port_start,port_end,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
        params![lease.id.to_string(),lease.unit_id.to_string(),p,g,t,key(lease.kind),lease.namespace,lease.value,lease.port_start,lease.port_end,key(lease.state),lease.version,serde_json::to_string(lease)?])?;
    Ok(())
}

impl Store {
    #[cfg(test)]
    pub(crate) fn connection_epoch_for_test(&self) -> u64 {
        self.connection
            .query_row(
                "SELECT epoch FROM runtime_epoch WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .unwrap()
    }
    /// Only the holder of the exclusive Runtime lock may call this at startup.
    /// Closing logical authority does not establish native workload death.
    pub(crate) fn begin_execution_epoch(&mut self) -> Result<(String, u64)> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (instance, old): (String, u64) = tx.query_row(
            "SELECT instance_id,epoch FROM runtime_epoch WHERE singleton=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let epoch = old.checked_add(1).context("owner epoch overflow")?;
        let mut statement=tx.prepare("SELECT id FROM execution_units WHERE native_effects_open=1 OR result_finalization_open=1")?;
        let units = statement
            .query_map([], |r| r.get::<_, String>(0))?
            .map(|r| {
                r.map_err(anyhow::Error::from)
                    .and_then(|id| unit_tx(&tx, id.parse()?))
            })
            .collect::<Result<Vec<_>>>()?;
        drop(statement);
        for mut unit in units {
            unit.native_effects_open = false;
            unit.result_finalization_open = false;
            if unit.work.is_none() {
                unit.work = Some(WorkOutcome::Unknown);
                unit.disposition = Disposition::Lost;
            }
            unit.state = if matches!(unit.work, Some(WorkOutcome::Success | WorkOutcome::Failure)) {
                UnitState::WorkKnown
            } else {
                UnitState::WorkUnknown
            };
            write_unit(&tx, &mut unit)?;
            if unit.kind == UnitKind::Executor {
                let next = unit
                    .generation
                    .checked_add(1)
                    .context("generation overflow")?;
                tx.execute("UPDATE task_execution SET generation=?1,active_unit=NULL WHERE task_id=?2 AND active_unit=?3 AND generation=?4",params![next,unit.scope.task_id.context("executor Task missing")?.to_string(),unit.id.to_string(),unit.generation])?;
            }
            tx.execute("INSERT INTO cleanup_jobs(unit_id,next_due,attempts,version) VALUES(?1,?2,0,1) ON CONFLICT(unit_id) DO UPDATE SET next_due=excluded.next_due,version=cleanup_jobs.version+1",params![unit.id.to_string(),now_ms()])?;
            append_event(
                &tx,
                &unit.scope,
                "execution.epoch_fenced",
                json!({"unit":unit.id,"work":unit.work,"cleanup":unit.cleanup}),
            )?;
        }
        // A stopped/cancelled unit can still have a native transport or an
        // unacknowledged tool effect when Runtime disappears. Reconcile all
        // managed bindings, including units whose authority was already closed.
        sessions::fence_epoch_sessions(&tx)?;
        effects::fence_epoch_effects(&tx)?;
        tx.execute(
            "UPDATE runtime_epoch SET epoch=?1 WHERE singleton=1",
            [epoch],
        )?;
        tx.execute("UPDATE quota_leases SET active=0 WHERE active=1", [])?;
        tx.execute("UPDATE quota_pools SET probe_unit=NULL", [])?;
        tx.execute("DELETE FROM quota_waiters", [])?;
        tx.commit()?;
        Ok((instance, epoch))
    }
    pub fn execution_unit(&self, id: UnitId) -> Result<ExecutionUnit> {
        unit_tx(&self.connection, id)
    }
    pub fn execution_units(&self, scope: Option<&Scope>) -> Result<Vec<ExecutionUnit>> {
        let mut statement = self
            .connection
            .prepare("SELECT id FROM execution_units ORDER BY rowid")?;
        let ids = statement
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        ids.into_iter()
            .map(|id| self.execution_unit(id.parse()?))
            .filter(|r| match r {
                Ok(u) => scope.is_none_or(|s| u.scope == *s),
                Err(_) => true,
            })
            .collect()
    }
    pub fn validate_execution(
        &self,
        authority: &ExecutionAuthority,
        native: bool,
        finalize: bool,
    ) -> Result<ExecutionUnit> {
        validate_authority(&self.connection, authority, native, finalize)
    }
    pub(crate) fn reserve_execution(
        &mut self,
        mut unit: ExecutionUnit,
        expected_task: u64,
    ) -> Result<ExecutionUnit> {
        ensure!(
            unit.kind != UnitKind::Legacy
                && unit.version == 0
                && valid_oid(&unit.base_sha)
                && unit.profile_digest.len() == 64
                && unit.cookie.len() == 36
                && unit.worktree.is_absolute()
                && unit.state == UnitState::Reserved
                && unit.session_id.is_none(),
            "invalid fresh execution unit"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let task: Task = read_tx(
            &tx,
            "tasks",
            &unit.scope.task_id.context("Task required")?.to_string(),
        )?
        .context("unknown Task")?;
        ensure!(
            task.scope() == unit.scope
                && task.version == expected_task
                && !task_terminal(task.state),
            "Task CAS/scope/lifecycle mismatch"
        );
        let project: Project = read_tx(&tx, "projects", &unit.scope.project_id.to_string())?
            .context("unknown Project")?;
        let goal: Goal =
            read_tx(&tx, "goals", &task.goal_id.to_string())?.context("unknown Goal")?;
        ensure!(
            project.state == ProjectState::Registered
                && !matches!(
                    goal.state,
                    GoalState::Paused
                        | GoalState::Completed
                        | GoalState::Cancelled
                        | GoalState::Failed
                ),
            "inactive Project/Goal"
        );
        let epoch: u64 = tx.query_row(
            "SELECT epoch FROM runtime_epoch WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        ensure!(epoch > 0 && epoch == unit.owner_epoch, "stale owner epoch");
        let prior: Option<u64> = tx
            .query_row(
                "SELECT generation FROM task_execution WHERE task_id=?1",
                [task.id.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        unit.generation = if unit.kind == UnitKind::Executor {
            prior
                .unwrap_or(0)
                .checked_add(1)
                .context("generation overflow")?
        } else {
            prior.context("review/verifier requires admitted artifact generation")?
        };
        if unit.kind == UnitKind::Executor {
            ensure!(
                unit.worktree.parent() == Some(project.worktree_root.as_path())
                    && unit
                        .branch
                        .as_ref()
                        .is_some_and(|b| b.starts_with("rrx/") && b != &project.base_branch),
                "invalid fresh executor namespace"
            );
        } else {
            let artifact =
                self_artifact_tx(&tx, unit.artifact_id.context("snapshot requires artifact")?)?;
            ensure!(
                artifact.scope == unit.scope
                    && artifact.state == ArtifactState::Published
                    && artifact.revision == unit.base_sha,
                "snapshot artifact/scope mismatch"
            );
        }
        unit.version = 1;
        insert_unit(&tx, &unit)?;
        tx.execute("INSERT INTO execution_context(unit_id,project_version,goal_version,governing_digest) VALUES(?1,?2,?3,?4)",
            params![unit.id.to_string(),project.version,goal.version,governing_digest(&project,&goal)?])?;
        if unit.kind == UnitKind::Executor {
            tx.execute("INSERT INTO task_execution(task_id,project_id,goal_id,generation,active_unit) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(task_id) DO UPDATE SET generation=excluded.generation,active_unit=excluded.active_unit",
                params![task.id.to_string(),task.project_id.to_string(),task.goal_id.to_string(),unit.generation,unit.id.to_string()])?;
            let mut next = task.clone();
            next.worktree = Some(unit.worktree.clone());
            next.branch = unit.branch.clone();
            let next = put_task_tx(&tx, &next)?;
            append_event(
                &tx,
                &unit.scope,
                "execution.attempt_reserved",
                json!({"unit":unit.id,"generation":unit.generation,"task_version":next.version}),
            )?;
        } else {
            append_event(
                &tx,
                &unit.scope,
                "execution.snapshot_reserved",
                json!({"unit":unit.id,"artifact":unit.artifact_id}),
            )?;
        }
        tx.commit()?;
        Ok(unit)
    }
    pub(crate) fn transition_execution(
        &mut self,
        authority: &ExecutionAuthority,
        state: UnitState,
    ) -> Result<ExecutionUnit> {
        ensure!(
            matches!(
                state,
                UnitState::Preparing
                    | UnitState::DispatchPending
                    | UnitState::Running
                    | UnitState::WaitingQuota
            ),
            "invalid nonterminal transition"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = validate_authority(&tx, authority, true, false)?;
        let allowed = matches!(
            (unit.state, state),
            (UnitState::Reserved, UnitState::Preparing)
                | (UnitState::Preparing, UnitState::DispatchPending)
                | (UnitState::DispatchPending, UnitState::Running)
                | (UnitState::Running, UnitState::WaitingQuota)
                | (UnitState::WaitingQuota, UnitState::Running)
        );
        ensure!(allowed, "invalid execution state edge");
        unit.state = state;
        write_unit(&tx, &mut unit)?;
        append_event(
            &tx,
            &unit.scope,
            "execution.state",
            json!({"unit":unit.id,"state":unit.state}),
        )?;
        tx.commit()?;
        Ok(unit)
    }
    pub(crate) fn finish_execution(
        &mut self,
        authority: &ExecutionAuthority,
        work: WorkOutcome,
        disposition: Disposition,
    ) -> Result<ExecutionUnit> {
        self.finish_execution_with_failure(authority, work, disposition, None)
    }
    pub(crate) fn finish_execution_with_failure(
        &mut self,
        authority: &ExecutionAuthority,
        work: WorkOutcome,
        disposition: Disposition,
        failure: Option<NativeFailure>,
    ) -> Result<ExecutionUnit> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = validate_authority(&tx, authority, false, true)?;
        ensure!(unit.work.is_none(), "work terminal already recorded");
        unit.native_effects_open = false;
        unit.work = Some(work);
        unit.disposition = disposition;
        unit.wait_reason = if disposition == Disposition::QuotaInterrupted {
            Some(WaitReason::Quota)
        } else {
            None
        };
        unit.state = if work == WorkOutcome::Unknown {
            UnitState::WorkUnknown
        } else {
            UnitState::WorkKnown
        };
        if disposition != Disposition::Completed {
            unit.result_finalization_open = false;
        }
        write_unit(&tx, &mut unit)?;
        tx.execute("INSERT INTO cleanup_jobs(unit_id,next_due,attempts,version) VALUES(?1,?2,0,1) ON CONFLICT(unit_id) DO NOTHING",params![unit.id.to_string(),now_ms()])?;
        quotas::release_quota_tx(&tx, unit.id)?;
        append_event(
            &tx,
            &unit.scope,
            "execution.work_terminal",
            json!({"unit":unit.id,"work":unit.work,"disposition":unit.disposition,"native_failure":failure}),
        )?;
        tx.commit()?;
        Ok(unit)
    }
    pub(crate) fn retire_execution(
        &mut self,
        authority: &ExecutionAuthority,
        cancel_task: bool,
    ) -> Result<ExecutionUnit> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = validate_authority(&tx, authority, false, false)?;
        unit.native_effects_open = false;
        unit.result_finalization_open = false;
        unit.state = UnitState::Retired;
        unit.disposition = Disposition::Cancelled;
        if unit.work.is_none() {
            unit.work = Some(WorkOutcome::Unknown);
        }
        write_unit(&tx, &mut unit)?;
        if unit.kind == UnitKind::Executor {
            tx.execute("UPDATE task_execution SET generation=generation+1,active_unit=NULL WHERE task_id=?1 AND generation=?2",params![unit.scope.task_id.unwrap().to_string(),unit.generation])?;
            if cancel_task {
                let mut task: Task =
                    read_tx(&tx, "tasks", &unit.scope.task_id.unwrap().to_string())?
                        .context("unknown Task")?;
                task.state = TaskState::Cancelled;
                put_task_tx(&tx, &task)?;
            }
        }
        quotas::release_quota_tx(&tx, unit.id)?;
        tx.execute(
            "DELETE FROM quota_waiters WHERE unit_id=?1",
            [unit.id.to_string()],
        )?;
        tx.execute("INSERT INTO cleanup_jobs(unit_id,next_due,attempts,version) VALUES(?1,?2,0,1) ON CONFLICT(unit_id) DO NOTHING",params![unit.id.to_string(),now_ms()])?;
        append_event(
            &tx,
            &unit.scope,
            "execution.retired",
            json!({"unit":unit.id,"generation":unit.generation,"work":unit.work}),
        )?;
        tx.commit()?;
        Ok(unit)
    }
    pub(crate) fn reserve_execution_leases(
        &mut self,
        authority: &ExecutionAuthority,
        leases: &[ResourceLease],
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        validate_authority(&tx, authority, true, false)?;
        for lease in leases {
            ensure!(
                lease.unit_id == authority.unit_id
                    && lease.scope == authority.scope
                    && lease.version == 1
                    && lease.state == LeaseState::Reserved
                    && !lease.value.is_empty()
                    && lease.value.len() <= 4096,
                "lease identity/state mismatch"
            );
            insert_lease(&tx, lease)?;
        }
        append_event(
            &tx,
            &authority.scope,
            "execution.resources_reserved",
            json!({"unit":authority.unit_id,"count":leases.len()}),
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn execution_leases(&self, id: UnitId) -> Result<Vec<ResourceLease>> {
        let mut s = self
            .connection
            .prepare("SELECT id FROM resource_leases WHERE unit_id=?1 ORDER BY rowid")?;
        let ids = s
            .query_map([id.to_string()], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        ids.into_iter()
            .map(|key| {
                let lease = lease_tx(&self.connection, key.parse()?)?;
                ensure!(lease.unit_id == id, "foreign lease");
                Ok(lease)
            })
            .collect()
    }
    pub(crate) fn record_execution_cleanup(
        &mut self,
        observation: &CleanupObservation,
    ) -> Result<()> {
        ensure!(
            observation.coverage.len() <= 32
                && observation.remaining.len() <= 1024
                && observation.errors.len() <= 32,
            "cleanup observation bound exceeded"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = unit_tx(&tx, observation.unit_id)?;
        unit.cleanup = observation.outcome;
        tx.execute(
            "INSERT INTO cleanup_observations(unit_id,at,body) VALUES(?1,?2,?3)",
            params![
                unit.id.to_string(),
                observation.at,
                serde_json::to_string(observation)?
            ],
        )?;
        append_event(
            &tx,
            &unit.scope,
            "execution.cleanup_observed",
            json!({"unit":unit.id,"work":unit.work,"cleanup":unit.cleanup,"coverage":observation.coverage}),
        )?;
        tx.commit()?;
        Ok(())
    }
}

fn self_artifact_tx(connection: &Connection, id: ArtifactId) -> Result<ResultArtifact> {
    let body: String = connection.query_row(
        "SELECT body FROM result_artifacts WHERE id=?1",
        [id.to_string()],
        |r| r.get(0),
    )?;
    let artifact: ResultArtifact = decode(body)?;
    ensure!(artifact.id == id, "artifact indexed identity mismatch");
    let mut columns = scoped_columns(&artifact.scope);
    columns.extend([
        ("id", json!(id)),
        ("unit_id", json!(artifact.unit_id)),
        ("state", json!(artifact.state)),
        ("version", json!(artifact.version)),
    ]);
    check_indexed(connection, "result_artifacts", &columns)?;
    Ok(artifact)
}
fn lease_tx(connection: &Connection, id: LeaseId) -> Result<ResourceLease> {
    let body: String = connection.query_row(
        "SELECT body FROM resource_leases WHERE id=?1",
        [id.to_string()],
        |r| r.get(0),
    )?;
    let lease: ResourceLease = decode(body)?;
    ensure!(lease.id == id, "lease identity mismatch");
    let mut cols = scoped_columns(&lease.scope);
    cols.extend([
        ("id", json!(id)),
        ("unit_id", json!(lease.unit_id)),
        ("kind", json!(lease.kind)),
        ("namespace", json!(lease.namespace)),
        ("value", json!(lease.value)),
        ("port_start", json!(lease.port_start)),
        ("port_end", json!(lease.port_end)),
        ("state", json!(lease.state)),
        ("version", json!(lease.version)),
    ]);
    check_indexed(connection, "resource_leases", &cols)?;
    Ok(lease)
}
fn effect_tx(connection: &Connection, id: OperationId) -> Result<ManagedEffect> {
    let body: String = connection.query_row(
        "SELECT body FROM managed_effects WHERE id=?1",
        [id.to_string()],
        |r| r.get(0),
    )?;
    let effect: ManagedEffect = decode(body)?;
    ensure!(effect.id == id, "effect identity mismatch");
    let mut cols = scoped_columns(&effect.scope);
    cols.extend([
        ("id", json!(id)),
        ("unit_id", json!(effect.unit_id)),
        ("idempotency_key", json!(effect.idempotency_key)),
        ("state", json!(effect.state)),
        ("version", json!(effect.version)),
    ]);
    check_indexed(connection, "managed_effects", &cols)?;
    Ok(effect)
}

// Other state operations share the same connection and transaction helpers.
mod artifacts;
mod effects;
mod quotas;
pub(crate) use quotas::QuotaAdmission;
mod migration;
mod sessions;
pub(super) use sessions::logically_retired_session;
#[cfg(test)]
mod tests;
