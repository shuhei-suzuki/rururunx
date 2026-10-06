//! Initial admission is separate from validation of an already live Driver.
//! Plans and durable rows remain nongrant until the actual worker acknowledges.
use super::*;
use crate::{
    execution::RuntimeOwner,
    state::managed_binding::{InstalledDriverComposition, ScopePlan},
};
use rusqlite::types::Value as SqlValue;
use std::collections::BTreeSet;

pub(crate) struct InitialDriverPlan {
    owner: Arc<RuntimeOwner>,
    scope: ScopePlan,
    row: Row,
    encoded: String,
    closed_encoded: String,
    prerequisites: Vec<(String, u64, String, String, u64, String)>,
    queue: u64,
    project_rotation: u64,
    goal_rotation: u64,
    global_limit: usize,
    project_limit: usize,
}
/// Exact committed content only; this is not a reconstructed live association.
pub(crate) struct PendingDriverClaim {
    plan: Arc<InitialDriverPlan>,
    composition: InstalledDriverComposition,
}
impl InitialDriverPlan {
    pub(super) fn close_unactivated_tx(
        &self,
        tx: &Transaction<'_>,
        permits: &crate::state::managed_binding::PrivatePermitManager,
    ) -> Result<()> {
        self.scope.validate_current(tx)?;
        validate_prerequisites(tx, self)?;
        let task = self.scope.task();
        let history: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM execution_units WHERE task_id=?1) OR EXISTS(SELECT 1 FROM task_execution WHERE task_id=?1) OR EXISTS(SELECT 1 FROM source_recoveries WHERE task_id=?1) OR EXISTS(SELECT 1 FROM managed_phase_operations WHERE task_id=?1) OR EXISTS(SELECT 1 FROM records WHERE task_id=?1 AND kind IN ('workflow','session')) OR EXISTS(SELECT 1 FROM context_versions WHERE task_id=?1)",[task.id.to_string()], |r|r.get(0))?;
        ensure!(
            !history,
            "unactivated claim has preparation/native history; retained recovery required"
        );
        let closed: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM task_drivers WHERE task_id=?1 AND goal_id=?2 AND project_id=?3 AND id=?4 AND owner_epoch=?5 AND version=2 AND state='invalid' AND body=?6) AND EXISTS(SELECT 1 FROM audit WHERE task_id=?1 AND goal_id=?2 AND project_id=?3 AND kind='rrx.private.runtime.driver_unactivated_closed' AND json_extract(data,'$.driver')=?4 AND json_extract(data,'$.epoch')=?5 AND json_extract(data,'$.version')=2)",params![task.id.to_string(),task.goal_id.to_string(),task.project_id.to_string(),self.row.id.to_string(),self.row.epoch,self.closed_encoded],|r|r.get(0))?;
        if closed {
            return Ok(());
        }
        let original: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM task_drivers WHERE task_id=?1 AND goal_id=?2 AND project_id=?3 AND id=?4 AND owner_epoch=?5 AND version=1 AND state='driving' AND body=?6)",params![task.id.to_string(),task.goal_id.to_string(),task.project_id.to_string(),self.row.id.to_string(),self.row.epoch,self.encoded],|r|r.get(0))?;
        ensure!(original, "unactivated claim original row changed");
        let mut after = image(self)?;
        after[5] = 2i64.into();
        after[6] = "invalid".to_owned().into();
        after[7] = self.closed_encoded.clone().into();
        let mutation = crate::state::managed_binding::ExactRowMutation::new(
            "task_drivers",
            "UPDATE",
            Some(image(self)?),
            Some(after),
        )?;
        permits.with_exact_permit(vec![mutation], || {
            ensure!(tx.execute("UPDATE task_drivers SET state='invalid',version=2,body=?1 WHERE task_id=?2 AND id=?3 AND owner_epoch=?4 AND version=1 AND state='driving' AND body=?5",params![self.closed_encoded,task.id.to_string(),self.row.id.to_string(),self.row.epoch,self.encoded])?==1,"unactivated claim closure CAS changed");
            permits.ensure_consumed()
        })?;
        append_event(
            tx,
            &task.scope(),
            "rrx.private.runtime.driver_unactivated_closed",
            json!({"driver":self.row.id,"epoch":self.row.epoch,"version":2,"native_settlement":false}),
        )?;
        Ok(())
    }
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
    pub(crate) fn initial_plan(&self) -> Arc<InitialDriverPlan> {
        self.plan.clone()
    }
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
pub(super) const PREREQUISITE_ROWS: usize = 128;
pub(super) const PREREQUISITE_BYTES: usize = 16 * 1024 * 1024;
pub(super) fn read_prerequisites(c: &Connection, task: &Task) -> Result<PrerequisiteRows> {
    read_prerequisites_bounded(c, task, usize::MAX, usize::MAX)
}
fn read_prerequisites_bounded(
    c: &Connection,
    task: &Task,
    max_rows: usize,
    max_bytes: usize,
) -> Result<PrerequisiteRows> {
    let goal: Goal = bounded(c, "goals", &task.goal_id.to_string(), 4 * 1024 * 1024)?;
    let mut identities = Vec::new();
    let mut seen = BTreeSet::new();
    let mut rows = 0usize;
    let mut bytes = 0usize;
    // This first pass copies no prerequisite body. Every full original encoded
    // length is charged in the same coherent read before any copy or decode.
    for edge in goal
        .dag
        .edges
        .iter()
        .filter(|e| e.hard && e.dependent == task.id)
    {
        ensure!(
            seen.insert(edge.prerequisite),
            "duplicate hard prerequisite"
        );
        rows = rows
            .checked_add(2)
            .context("Driver prerequisite row overflow")?;
        ensure!(
            rows <= max_rows,
            "Driver prerequisite corpus row budget exceeded"
        );
        let (version, task_len): (u64, usize) = c.query_row(
            "SELECT version,length(CAST(body AS BLOB)) FROM tasks WHERE id=?1 AND goal_id=?2 AND project_id=?3",
            params![edge.prerequisite.to_string(), task.goal_id.to_string(), task.project_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let count: u64 = c.query_row(
            "SELECT count(*) FROM records WHERE task_id=?1 AND kind='workflow'",
            [edge.prerequisite.to_string()],
            |r| r.get(0),
        )?;
        ensure!(count == 1, "Driver prerequisite Workflow inventory differs");
        let (wid, wversion, workflow_len): (String, u64, usize) = c.query_row(
            "SELECT id,version,length(CAST(body AS BLOB)) FROM records WHERE kind='workflow' AND task_id=?1 AND goal_id=?2 AND project_id=?3",
            params![edge.prerequisite.to_string(), task.goal_id.to_string(), task.project_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        ensure!(
            task_len <= 1024 * 1024 && workflow_len <= 8 * 1024 * 1024,
            "Driver prerequisite body exceeds bound"
        );
        bytes = bytes
            .checked_add(task_len)
            .and_then(|n| n.checked_add(workflow_len))
            .context("Driver prerequisite byte overflow")?;
        ensure!(
            bytes <= max_bytes,
            "Driver prerequisite corpus byte budget exceeded"
        );
        identities.push((
            edge.prerequisite,
            version,
            wid,
            wversion,
            task_len,
            workflow_len,
        ));
    }
    let mut result = Vec::with_capacity(identities.len());
    for (id, version, wid, wversion, task_len, workflow_len) in identities {
        let body: String = c.query_row(
            "SELECT body FROM tasks WHERE id=?1 AND goal_id=?2 AND project_id=?3 AND version=?4 AND length(CAST(body AS BLOB))=?5",
            params![id.to_string(), task.goal_id.to_string(), task.project_id.to_string(), version, task_len], |r|r.get(0),
        )?;
        let wbody: String = c.query_row(
            "SELECT body FROM records WHERE id=?1 AND kind='workflow' AND task_id=?2 AND goal_id=?3 AND project_id=?4 AND version=?5 AND length(CAST(body AS BLOB))=?6",
            params![wid, id.to_string(), task.goal_id.to_string(), task.project_id.to_string(), wversion, workflow_len], |r|r.get(0),
        )?;
        let predecessor: Task = decode(body.clone())?;
        let record: Record = decode(wbody.clone())?;
        ensure!(
            predecessor.id == id
                && predecessor.scope() == record.scope
                && predecessor.project_id == task.project_id
                && predecessor.goal_id == task.goal_id
                && predecessor.version == version
                && record.version == wversion
                && record.id.to_string() == wid
                && record.kind == RecordKind::Workflow,
            "Driver prerequisite body/index identity differs"
        );
        result.push((id.to_string(), version, body, wid, wversion, wbody));
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
        let mut closed = row.clone();
        closed.state = "invalid".into();
        closed.version = 2;
        let closed_encoded = serde_json::to_string(&closed)?;
        ensure!(
            encoded.len() <= 128 * 1024 && closed_encoded.len() <= 128 * 1024,
            "Driver metadata exceeds bound"
        );
        Ok(InitialDriverPlan {
            owner: owner.clone(),
            scope,
            row,
            encoded,
            closed_encoded,
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
        Ok(PendingDriverClaim {
            plan: Arc::new(plan),
            composition,
        })
    }
    pub(crate) fn validate_pending_driver(&self, claim: &PendingDriverClaim) -> Result<()> {
        claim.validate_current(&self.connection)
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;

    // Only historical read data in a minimal database. These rows construct no
    // accepted Goal, Driver, Unit, input, composition or completion authority.
    fn history(count: usize) -> (Connection, Task, usize, TaskId) {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE TABLE goals(id TEXT PRIMARY KEY,body TEXT); CREATE TABLE tasks(id TEXT PRIMARY KEY,project_id TEXT,goal_id TEXT,version INTEGER,body TEXT); CREATE TABLE records(id TEXT PRIMARY KEY,project_id TEXT,goal_id TEXT,task_id TEXT,kind TEXT,version INTEGER,body TEXT);").unwrap();
        let p = ProjectId::new();
        let mut goal = Goal::new(p, "historical read fixture".into(), vec![]);
        let task = Task::new(p, goal.id, "dependent".into(), "codex".into());
        let mut bytes = 0;
        let mut first = None;
        goal.dag.nodes.push(task.id);
        for i in 0..count {
            let predecessor = Task::new(
                p,
                goal.id,
                format!("predecessor-{i}-日本語"),
                "codex".into(),
            );
            first.get_or_insert(predecessor.id);
            goal.dag.nodes.push(predecessor.id);
            goal.dag.edges.push(crate::domain::Dependency {
                prerequisite: predecessor.id,
                dependent: task.id,
                hard: true,
            });
            let record = Record::new(
                predecessor.scope(),
                RecordKind::Workflow,
                json!({"historical":i}),
            );
            let body = serde_json::to_string(&predecessor).unwrap();
            let wbody = serde_json::to_string(&record).unwrap();
            bytes += body.len() + wbody.len();
            c.execute(
                "INSERT INTO tasks VALUES(?1,?2,?3,?4,?5)",
                params![
                    predecessor.id.to_string(),
                    p.to_string(),
                    goal.id.to_string(),
                    predecessor.version,
                    body
                ],
            )
            .unwrap();
            c.execute(
                "INSERT INTO records VALUES(?1,?2,?3,?4,'workflow',?5,?6)",
                params![
                    record.id.to_string(),
                    p.to_string(),
                    goal.id.to_string(),
                    predecessor.id.to_string(),
                    record.version,
                    wbody
                ],
            )
            .unwrap();
        }
        c.execute(
            "INSERT INTO goals VALUES(?1,?2)",
            params![goal.id.to_string(), serde_json::to_string(&goal).unwrap()],
        )
        .unwrap();
        (c, task, bytes, first.unwrap())
    }
    #[test]
    fn production_prerequisite_row_profile_refuses_complete_surplus() {
        let (mut c, task, _, _) = history(PREREQUISITE_ROWS / 2);
        assert_eq!(
            read_prerequisites(&c, &task).unwrap().len(),
            PREREQUISITE_ROWS / 2
        );
        let tx = c.transaction().unwrap();
        assert_eq!(
            read_prerequisites_bounded(&tx, &task, PREREQUISITE_ROWS, PREREQUISITE_BYTES)
                .unwrap()
                .len(),
            PREREQUISITE_ROWS / 2
        );
        drop(tx);
        let (c, task, bytes, _) = history(PREREQUISITE_ROWS / 2 + 1);
        assert!(
            read_prerequisites(&c, &task)
                .unwrap_err()
                .to_string()
                .contains("row budget")
        );
        assert_eq!(
            read_prerequisites_bounded(&c, &task, PREREQUISITE_ROWS + 2, bytes)
                .unwrap()
                .len(),
            PREREQUISITE_ROWS / 2 + 1
        );
    }
    #[test]
    fn production_prerequisite_byte_profile_is_exact_and_complete() {
        let (mut c, task, _, _) = history(2);
        let task_bytes: usize = c
            .query_row(
                "SELECT sum(length(CAST(body AS BLOB))) FROM tasks",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let mut q = c.prepare("SELECT body FROM records ORDER BY id").unwrap();
        let records: Vec<String> = q
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        drop(q);
        let mut wanted = PREREQUISITE_BYTES - task_bytes;
        let mut final_id = None;
        for (i, body) in records.into_iter().enumerate() {
            let mut record: Record = decode(body).unwrap();
            record.data["padding"] = json!("");
            let empty = serde_json::to_string(&record).unwrap();
            let size = if i == 0 { wanted / 2 } else { wanted };
            wanted -= size;
            record.data["padding"] = json!("x".repeat(size - empty.len()));
            let full = serde_json::to_string(&record).unwrap();
            assert_eq!(full.len(), size);
            assert!(full.len() < 8 * 1024 * 1024);
            c.execute(
                "UPDATE records SET body=?1 WHERE id=?2",
                params![full, record.id.to_string()],
            )
            .unwrap();
            final_id = Some(record.id);
        }
        let tx = c.transaction().unwrap();
        let bodies = read_prerequisites(&tx, &task).unwrap();
        assert_eq!(
            bodies.iter().map(|r| r.2.len() + r.5.len()).sum::<usize>(),
            PREREQUISITE_BYTES
        );
        drop(bodies);
        drop(tx);
        // JSON whitespace is encoded material too; each row remains valid and
        // below its individual bound, while the complete corpus is one byte over.
        c.execute(
            "UPDATE records SET body=body||' ' WHERE id=?1",
            [final_id.unwrap().to_string()],
        )
        .unwrap();
        let tx = c.transaction().unwrap();
        assert!(
            read_prerequisites(&tx, &task)
                .unwrap_err()
                .to_string()
                .contains("byte budget")
        );
        assert_eq!(
            read_prerequisites_bounded(&tx, &task, PREREQUISITE_ROWS, PREREQUISITE_BYTES + 1)
                .unwrap()
                .len(),
            2
        );
    }
    #[test]
    fn prerequisite_reader_encoded_byte_and_row_limits_are_inclusive() {
        let (mut c, task, bytes, _) = history(2);
        let tx = c.transaction().unwrap();
        assert_eq!(
            read_prerequisites_bounded(&tx, &task, 4, bytes)
                .unwrap()
                .len(),
            2
        );
        assert!(
            read_prerequisites_bounded(&tx, &task, 4, bytes - 1)
                .unwrap_err()
                .to_string()
                .contains("byte budget")
        );
        assert!(
            read_prerequisites_bounded(&tx, &task, 3, bytes)
                .unwrap_err()
                .to_string()
                .contains("row budget")
        );
        assert_eq!(read_prerequisites(&tx, &task).unwrap().len(), 2);
    }
    #[test]
    fn prerequisite_corpus_refuses_before_decoding_any_body() {
        let (mut c, task, bytes, first) = history(2);
        // A first-body parse canary must remain unread when complete inventory
        // exceeds budget at a later prerequisite. Preserve its original length.
        c.execute(
            "UPDATE tasks SET body=replace(body,'{','!') WHERE id=?1",
            [first.to_string()],
        )
        .unwrap();
        let tx = c.transaction().unwrap();
        assert!(
            read_prerequisites_bounded(&tx, &task, 4, bytes - 1)
                .unwrap_err()
                .to_string()
                .contains("byte budget")
        );
        assert!(
            read_prerequisites_bounded(&tx, &task, 3, bytes)
                .unwrap_err()
                .to_string()
                .contains("row budget")
        );
        assert!(
            !read_prerequisites_bounded(&tx, &task, 4, bytes)
                .unwrap_err()
                .to_string()
                .contains("budget")
        );
    }
}
