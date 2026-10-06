//! Initial admission is separate from validation of an already live Driver.
//! Plans and durable rows remain nongrant until the actual worker acknowledges.
use super::*;
use crate::{
    execution::RuntimeOwner,
    state::managed_binding::{InstalledDriverComposition, ScopePlan},
};
use rusqlite::types::Value as SqlValue;

pub(crate) struct InitialDriverPlan {
    owner: Arc<RuntimeOwner>,
    scope: ScopePlan,
    row: Row,
    encoded: String,
    prerequisites: Vec<(String, u64, String, String, u64, String)>,
    queue: u64,
    project_rotation: u64,
    goal_rotation: u64,
    global_limit: usize,
    project_limit: usize,
}
/// Exact committed content only; this is not a reconstructed live association.
pub(crate) struct PendingDriverClaim {
    plan: InitialDriverPlan,
    composition: InstalledDriverComposition,
}
impl InitialDriverPlan {
    pub(crate) fn task(&self) -> TaskId {
        self.scope.task().id
    }
    pub(crate) fn owner_identity(&self) -> (&std::path::Path, &str) {
        (self.owner.state_path(), self.owner.instance_id())
    }
    pub(crate) fn identity(&self) -> (Uuid, u64, u64, &str) {
        (self.row.id, self.row.epoch, self.row.version, &self.encoded)
    }
}
impl PendingDriverClaim {
    pub(crate) fn task(&self) -> &Task {
        self.plan.scope.task()
    }
    pub(crate) fn identity(&self) -> (Uuid, u64, u64, &str) {
        self.plan.identity()
    }
    pub(crate) fn composition(&self) -> &InstalledDriverComposition {
        &self.composition
    }
    pub(crate) fn owner(&self) -> &Arc<RuntimeOwner> {
        &self.plan.owner
    }
    pub(crate) fn validate_current(&self, c: &Connection) -> Result<()> {
        self.plan.scope.validate_current(c)?;
        validate_prerequisites(c, &self.plan)?;
        let (id, epoch, version, body) = self.identity();
        let exact:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM task_drivers WHERE task_id=?1 AND goal_id=?2 AND project_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='driving' AND body=?7)",params![self.task().id.to_string(),self.task().goal_id.to_string(),self.task().project_id.to_string(),id.to_string(),epoch,version,body],|r|r.get(0))?;
        ensure!(exact, "pending Driver claim changed");
        Ok(())
    }
}
pub(super) type PrerequisiteRows = Vec<(String, u64, String, String, u64, String)>;
pub(super) fn read_prerequisites(c: &Connection, task: &Task) -> Result<PrerequisiteRows> {
    let goal: Goal = bounded(c, "goals", &task.goal_id.to_string(), 4 * 1024 * 1024)?;
    let mut result = Vec::new();
    for edge in goal
        .dag
        .edges
        .iter()
        .filter(|e| e.hard && e.dependent == task.id)
    {
        let (version, body): (u64, String) = c.query_row(
            "SELECT version,CASE WHEN length(CAST(body AS BLOB))<=1048576 THEN body END FROM tasks WHERE id=?1 AND goal_id=?2 AND project_id=?3",
            params![edge.prerequisite.to_string(), task.goal_id.to_string(), task.project_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let (id, wversion, wbody): (String, u64, String) = c.query_row(
            "SELECT id,version,CASE WHEN length(CAST(body AS BLOB))<=8388608 THEN body END FROM records WHERE kind='workflow' AND task_id=?1 AND goal_id=?2 AND project_id=?3",
            params![edge.prerequisite.to_string(), task.goal_id.to_string(), task.project_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let predecessor: Task = decode(body.clone())?;
        let record: Record = decode(wbody.clone())?;
        ensure!(
            predecessor.id == edge.prerequisite
                && predecessor.scope() == record.scope
                && predecessor.project_id == task.project_id
                && predecessor.goal_id == task.goal_id
                && predecessor.version == version
                && record.version == wversion
                && record.id.to_string() == id
                && record.kind == RecordKind::Workflow,
            "Driver prerequisite body/index identity differs"
        );
        result.push((
            edge.prerequisite.to_string(),
            version,
            body,
            id,
            wversion,
            wbody,
        ));
    }
    Ok(result)
}
pub(super) fn validate_prerequisite_rows(
    c: &Connection,
    task: &Task,
    rows: &PrerequisiteRows,
) -> Result<()> {
    for (id, version, body, wid, wversion, wbody) in rows {
        let count: u64 = c.query_row(
            "SELECT count(*) FROM records WHERE task_id=?1 AND kind='workflow'",
            [id],
            |r| r.get(0),
        )?;
        ensure!(
            count == 1,
            "Driver prerequisite Workflow inventory changed or malformed"
        );
        let exact: bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM tasks t JOIN records w ON w.task_id=t.id AND w.goal_id=t.goal_id AND w.project_id=t.project_id WHERE t.id=?1 AND t.goal_id=?2 AND t.project_id=?3 AND t.version=?4 AND t.body=?5 AND w.id=?6 AND w.kind='workflow' AND w.version=?7 AND w.body=?8)",
            params![id, task.goal_id.to_string(), task.project_id.to_string(), version, body, wid, wversion, wbody],
            |r| r.get(0),
        )?;
        ensure!(exact, "Driver prerequisite changed before activation/claim");
    }
    Ok(())
}
fn validate_prerequisites(c: &Connection, plan: &InitialDriverPlan) -> Result<()> {
    validate_prerequisite_rows(c, plan.scope.task(), &plan.prerequisites)
}
fn fresh(c: &Connection, task: &Task) -> Result<()> {
    let history:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM execution_units WHERE task_id=?1) OR EXISTS(SELECT 1 FROM task_execution WHERE task_id=?1) OR EXISTS(SELECT 1 FROM source_recoveries WHERE task_id=?1) OR EXISTS(SELECT 1 FROM managed_phase_operations WHERE task_id=?1) OR EXISTS(SELECT 1 FROM task_drivers WHERE task_id=?1) OR EXISTS(SELECT 1 FROM records WHERE task_id=?1 AND kind='workflow') OR EXISTS(SELECT 1 FROM context_versions WHERE task_id=?1)",[task.id.to_string()],|r|r.get(0))?;
    ensure!(
        !history && task.worktree.is_none() && task.branch.is_none() && task.revision.is_none(),
        "existing Task preparation requires genuine retained-source/recovery admission"
    );
    Ok(())
}
/// Pure/read-only coherent first-bootstrap plan. No helper, Unit or live Driver
/// is necessary to read it. Composition is checked by its actual caller first.
pub(crate) fn plan_initial_driver(
    owner: Arc<RuntimeOwner>,
    task: TaskId,
    global_limit: usize,
    project_limit: usize,
) -> Result<InitialDriverPlan> {
    ensure!(
        global_limit > 0 && project_limit > 0 && global_limit <= 4096 && project_limit <= 4096,
        "Driver capacity bound invalid"
    );
    crate::state::managed_binding::snapshot(&owner, |tx| {
        let t: Task = bounded(tx, "tasks", &task.to_string(), 1024 * 1024)?;
        let scope = crate::state::managed_binding::read_scope(tx, &owner, &t.scope())?;
        ensure!(
            scope.task().id == task && !scope.has_input_history(),
            "existing Workflow input requires retained-source Driver admission"
        );
        let pins = snapshot(tx, task)?;
        fresh(tx, scope.task())?;
        let prerequisites = read_prerequisites(tx, scope.task())?;

        let (queue,project_rotation,goal_rotation):(u64,u64,u64)=tx.query_row("SELECT s.queue_sequence,p.rotation,g.rotation FROM scheduler_tasks s JOIN scheduler_projects p ON p.project_id=s.project_id JOIN scheduler_goals g ON g.goal_id=s.goal_id AND g.project_id=s.project_id WHERE s.task_id=?1 AND s.goal_id=?2 AND s.project_id=?3",params![task.to_string(),t.goal_id.to_string(),t.project_id.to_string()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
        let row = Row {
            id: Uuid::new_v4(),
            epoch: owner.epoch(),
            version: 1,
            state: "driving".into(),
            pins,
            marker: None,
        };
        let encoded = serde_json::to_string(&row)?;
        ensure!(encoded.len() <= 128 * 1024, "Driver metadata exceeds bound");
        Ok(InitialDriverPlan {
            owner: owner.clone(),
            scope,
            row,
            encoded,
            prerequisites,
            queue,
            project_rotation,
            goal_rotation,
            global_limit,
            project_limit,
        })
    })
}
fn image(plan: &InitialDriverPlan) -> Result<Vec<SqlValue>> {
    let task = plan.scope.task();
    Ok(vec![
        task.id.to_string().into(),
        task.goal_id.to_string().into(),
        task.project_id.to_string().into(),
        plan.row.id.to_string().into(),
        i64::try_from(plan.row.epoch)?.into(),
        1i64.into(),
        "driving".to_string().into(),
        plan.encoded.clone().into(),
    ])
}
impl Store {
    /// Requires the actual Root-owned installed composition. No public DTO can
    /// make this constructor callable or convert the committed row into liveness.
    pub(crate) fn claim_initial_driver(
        &mut self,
        plan: InitialDriverPlan,
        composition: InstalledDriverComposition,
    ) -> Result<PendingDriverClaim> {
        composition.validate_for(&plan.owner, plan.scope.task())?;
        let mutation = crate::state::managed_binding::ExactRowMutation::new(
            "task_drivers",
            "INSERT",
            None,
            Some(image(&plan)?),
        )?;
        let permits = self.binding_permits.clone();
        permits.with_exact_permit(vec![mutation], || {
            let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            plan.scope.validate_current(&tx)?;
            let task=plan.scope.task();
            fresh(&tx,task)?;
            validate_prerequisites(&tx, &plan)?;

            let ranks:(u64,u64,u64)=tx.query_row("SELECT s.queue_sequence,p.rotation,g.rotation FROM scheduler_tasks s JOIN scheduler_projects p ON p.project_id=s.project_id JOIN scheduler_goals g ON g.goal_id=s.goal_id AND g.project_id=s.project_id WHERE s.task_id=?1 AND s.goal_id=?2 AND s.project_id=?3",params![task.id.to_string(),task.goal_id.to_string(),task.project_id.to_string()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
            ensure!(ranks==(plan.queue,plan.project_rotation,plan.goal_rotation),"Driver scheduler rank changed");
            // Union distinct Tasks, including held/unbound operations and claims.
            // Pending phase slots are separately retained by Root's supervisor.
            let (all,project):(usize,usize)=tx.query_row("WITH occupied AS (SELECT task_id,project_id FROM task_drivers WHERE state='driving' UNION SELECT task_id,project_id FROM execution_units WHERE native_effects_open=1 OR result_finalization_open=1 UNION SELECT task_id,project_id FROM managed_phase_operations WHERE phase_open=1) SELECT count(*),COALESCE(sum(project_id=?1),0) FROM occupied",[task.project_id.to_string()],|r|Ok((r.get(0)?,r.get(1)?)))?;
            ensure!(all<plan.global_limit && project<plan.project_limit,"distinct Task capacity unavailable");
            tx.execute("INSERT INTO task_drivers(task_id,goal_id,project_id,id,owner_epoch,version,state,body) VALUES(?1,?2,?3,?4,?5,1,'driving',?6)",params![task.id.to_string(),task.goal_id.to_string(),task.project_id.to_string(),plan.row.id.to_string(),plan.row.epoch,plan.encoded])?;
            let next_project=plan.project_rotation.checked_add(1).filter(|v|*v<=i64::MAX as u64).context("Project fairness exhausted")?;
            let next_goal=plan.goal_rotation.checked_add(1).filter(|v|*v<=i64::MAX as u64).context("Goal fairness exhausted")?;
            ensure!(tx.execute("UPDATE scheduler_projects SET rotation=?1 WHERE project_id=?2 AND rotation=?3",params![next_project,task.project_id.to_string(),plan.project_rotation])?==1,"Project fairness CAS changed");
            ensure!(tx.execute("UPDATE scheduler_goals SET rotation=?1 WHERE goal_id=?2 AND project_id=?3 AND rotation=?4",params![next_goal,task.goal_id.to_string(),task.project_id.to_string(),plan.goal_rotation])?==1,"Goal fairness CAS changed");
            append_event(&tx,&task.scope(),"rrx.private.runtime.driver_claimed",json!({"driver":plan.row.id,"epoch":plan.row.epoch,"queue_sequence":plan.queue}))?;
            permits.ensure_consumed()?;
            tx.commit()?;
            Ok(())
        })?;
        Ok(PendingDriverClaim { plan, composition })
    }
    pub(crate) fn validate_pending_driver(&self, claim: &PendingDriverClaim) -> Result<()> {
        claim.validate_current(&self.connection)
    }
}
