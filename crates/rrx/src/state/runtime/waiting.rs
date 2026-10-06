//! Bounded status facts only. These observations cannot issue or resume a launch.
use super::super::*;
use crate::{
    execution::{ExecutionUnit, UnitState, WaitReason},
    workflow::PhaseWaitingObservation,
};

const HELD: PhaseWaitingObservation = PhaseWaitingObservation::Held;

fn eligible(task: &Task, goal: &Goal) -> bool {
    matches!(goal.state, GoalState::Analyzing | GoalState::Running)
        && !matches!(
            task.state,
            TaskState::Completed | TaskState::Cancelled | TaskState::Failed | TaskState::Merged
        )
}

/// Joins at most two open operations. A second row, a missing owner or malformed
/// image is attention for this Task only; no authority is inferred from it.
pub(super) fn observe(
    connection: &Connection,
    task: &Task,
    goal: &Goal,
) -> Option<PhaseWaitingObservation> {
    if !eligible(task, goal) {
        return None;
    }
    observe_bounded(connection, task).unwrap_or(Some(HELD))
}

fn observe_bounded(
    connection: &Connection,
    task: &Task,
) -> Result<Option<PhaseWaitingObservation>> {
    let mut query = connection.prepare_cached(
        "SELECT CASE WHEN typeof(u.body)='text' AND length(CAST(u.body AS BLOB))<=16384 THEN u.body END,
         CASE WHEN typeof(r.body)='text' AND length(CAST(r.body AS BLOB))<=4096 THEN r.body END,
         r.state,r.version,r.parking_version,
         CASE WHEN length(w.reason)<=16 THEN w.reason END,w.next_due,
         COALESCE(u.project_id=o.project_id AND u.goal_id=o.goal_id AND u.task_id=o.task_id
          AND u.generation=o.execution_generation AND u.owner_epoch=o.owner_epoch
          AND u.owner_epoch=e.epoch AND u.native_effects_open=1
          AND o.marker_task_version=?4
          AND ow.operation_id=o.operation_id AND ow.unit_id=o.unit_id
          AND ow.project_id=o.project_id AND ow.goal_id=o.goal_id AND ow.task_id=o.task_id
          AND ow.owner_epoch=o.owner_epoch AND ow.execution_generation=o.execution_generation
          AND ow.origin=o.origin AND ow.provider=o.provider
          AND d.owner_epoch=o.owner_epoch AND d.state='driving'
          AND r.owner_epoch=o.owner_epoch AND r.origin=o.origin
          AND r.start_ended=0 AND r.known_terminal=0,0),
         COALESCE(w.provider=o.provider AND w.account_key='unknown' AND w.resume_state='preparing',0),
         o.unit_id,o.provider,o.phase,o.execution_generation,o.owner_epoch,u.version,o.operation_id,o.origin
         FROM managed_phase_operations o
         LEFT JOIN execution_units u ON u.id=o.unit_id
         LEFT JOIN managed_phase_owners ow ON ow.owner_id=o.owner_id
         LEFT JOIN managed_phase_readiness r ON r.operation_id=o.operation_id
         LEFT JOIN quota_waiters w ON w.unit_id=o.unit_id
         LEFT JOIN task_drivers d ON d.task_id=o.task_id AND d.goal_id=o.goal_id AND d.project_id=o.project_id
         LEFT JOIN runtime_epoch e ON e.singleton=1
         WHERE o.project_id=?1 AND o.goal_id=?2 AND o.task_id=?3 AND o.phase_open=1 LIMIT 2"
    )?;
    let mut rows = query.query(params![
        task.project_id.to_string(),
        task.goal_id.to_string(),
        task.id.to_string(),
        task.version
    ])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    let observation = (|| -> Result<Option<PhaseWaitingObservation>> {
        if !row.get::<_, bool>(7)? {
            return Ok(Some(HELD));
        }
        let state: String = row.get(2)?;
        if matches!(state.as_str(), "held" | "deferred") {
            return Ok(Some(HELD));
        }
        if state != "parked" {
            return Ok(None);
        }
        let raw: String = row.get(0)?;
        let unit: ExecutionUnit = serde_json::from_str(&raw)?;
        let ready: Value = serde_json::from_str(&row.get::<_, String>(1)?)?;
        let version: u64 = row.get(3)?;
        let parking: Option<u64> = row.get(4)?;
        let reason: String = row.get(5)?;
        let due: i64 = row.get(6)?;
        let expected = match reason.as_str() {
            "quota" => WaitReason::Quota,
            "capacity" => WaitReason::Capacity,
            _ => return Ok(Some(HELD)),
        };
        let matches = row.get::<_, bool>(8)?
            && parking == Some(version)
            && ready["state"] == "parked"
            && ready["version"] == version
            && ready["parking_version"] == version
            && ready["start_ended"] == false
            && ready["known_terminal"] == false
            && unit.id.to_string() == row.get::<_, String>(9)?
            && unit.scope == task.scope()
            && unit.state == UnitState::Preparing
            && unit.native_effects_open
            && unit.wait_reason == Some(expected)
            && unit.session_id.is_none()
            && unit.provider == row.get::<_, String>(10)?
            && unit.phase == row.get::<_, String>(11)?
            && unit.generation == row.get::<_, u64>(12)?
            && unit.owner_epoch == row.get::<_, u64>(13)?
            && unit.version == row.get::<_, u64>(14)?
            && ready["operation_id"] == row.get::<_, String>(15)?
            && ready["origin"] == row.get::<_, String>(16)?
            && ready["owner_epoch"] == unit.owner_epoch;
        Ok(Some(if matches {
            PhaseWaitingObservation::Waiting {
                reason: expected,
                next_due: due,
            }
        } else {
            HELD
        }))
    })()
    .unwrap_or(Some(HELD));
    Ok(if rows.next()?.is_some() {
        Some(HELD)
    } else {
        observation
    })
}

pub(super) fn effective_state(
    task: &Task,
    observation: Option<&PhaseWaitingObservation>,
) -> TaskState {
    match observation {
        Some(PhaseWaitingObservation::Waiting {
            reason: WaitReason::Quota,
            ..
        }) => TaskState::WaitingQuota,
        Some(PhaseWaitingObservation::Waiting {
            reason: WaitReason::Capacity,
            ..
        }) => TaskState::WaitingCapacity,
        _ => task.state,
    }
}

impl Store {
    pub(crate) fn phase_waiting_observation(
        &self,
        task: &Task,
        goal: &Goal,
    ) -> Result<Option<PhaseWaitingObservation>> {
        let tx = self.connection.unchecked_transaction()?;
        let observation = observe(&tx, task, goal);
        tx.commit()?;
        Ok(observation)
    }
}

#[cfg(test)]
mod waiting_observation_tests {
    use super::*;
    use crate::execution::{CleanupOutcome, Disposition, UnitId, UnitKind};

    // Nongrant SQL status fixture. It creates no managed permit, Native actor,
    // retained launch, Driver claim or input authority.
    struct Facts {
        db: Connection,
        task: Task,
        goal: Goal,
    }
    impl Facts {
        fn new(reason: WaitReason) -> Self {
            let project = ProjectId::new();
            let mut goal = Goal::new(project, "status fixture".into(), vec![]);
            goal.state = GoalState::Running;
            let mut task = Task::new(project, goal.id, "status".into(), "native".into());
            task.version = 7;
            task.state = TaskState::Implementing;
            task.phase = Some("Implement".into());
            let unit = ExecutionUnit {
                id: UnitId::new(),
                scope: task.scope(),
                kind: UnitKind::Executor,
                generation: 1,
                owner_epoch: 1,
                version: 3,
                phase: "Implement".into(),
                provider: "claude".into(),
                state: UnitState::Preparing,
                native_effects_open: true,
                result_finalization_open: true,
                work: None,
                cleanup: CleanupOutcome::Unknown,
                disposition: Disposition::Active,
                worktree: "/status-facts".into(),
                branch: None,
                base_sha: "a".repeat(40),
                profile_digest: "b".repeat(64),
                cookie: uuid::Uuid::new_v4().to_string(),
                session_id: None,
                artifact_id: None,
                wait_reason: Some(reason),
                capacity_retry_at: None,
                created_at: 1,
                updated_at: 2,
            };
            let db = Connection::open_in_memory().unwrap();
            db.execute_batch("CREATE TABLE managed_phase_operations(operation_id TEXT,owner_id TEXT,unit_id TEXT,project_id TEXT,goal_id TEXT,task_id TEXT,execution_generation INTEGER,owner_epoch INTEGER,marker_task_version INTEGER,provider TEXT,phase TEXT,origin TEXT,phase_open INTEGER);
                CREATE TABLE execution_units(id TEXT,project_id TEXT,goal_id TEXT,task_id TEXT,generation INTEGER,owner_epoch INTEGER,version INTEGER,native_effects_open INTEGER,body TEXT);
                CREATE TABLE managed_phase_owners(owner_id TEXT,operation_id TEXT,unit_id TEXT,project_id TEXT,goal_id TEXT,task_id TEXT,owner_epoch INTEGER,execution_generation INTEGER,origin TEXT,provider TEXT);
                CREATE TABLE managed_phase_readiness(operation_id TEXT,origin TEXT,owner_epoch INTEGER,state TEXT,start_ended INTEGER,known_terminal INTEGER,version INTEGER,parking_version INTEGER,body TEXT);
                CREATE TABLE quota_waiters(unit_id TEXT,provider TEXT,account_key TEXT,resume_state TEXT,reason TEXT,next_due INTEGER);
                CREATE TABLE task_drivers(task_id TEXT,goal_id TEXT,project_id TEXT,owner_epoch INTEGER,state TEXT);
                CREATE TABLE runtime_epoch(singleton INTEGER,epoch INTEGER);").unwrap();
            db.execute("INSERT INTO managed_phase_operations VALUES('operation','owner',?1,?2,?3,?4,1,1,7,'claude','Implement','origin',1)", params![unit.id.to_string(), project.to_string(), goal.id.to_string(), task.id.to_string()]).unwrap();
            db.execute(
                "INSERT INTO execution_units VALUES(?1,?2,?3,?4,1,1,3,1,?5)",
                params![
                    unit.id.to_string(),
                    project.to_string(),
                    goal.id.to_string(),
                    task.id.to_string(),
                    serde_json::to_string(&unit).unwrap()
                ],
            )
            .unwrap();
            db.execute("INSERT INTO managed_phase_owners VALUES('owner','operation',?1,?2,?3,?4,1,1,'origin','claude')", params![unit.id.to_string(), project.to_string(), goal.id.to_string(), task.id.to_string()]).unwrap();
            db.execute("INSERT INTO managed_phase_readiness VALUES('operation','origin',1,'parked',0,0,3,3,?1)", [json!({"operation_id":"operation","origin":"origin","owner_epoch":1,"state":"parked","start_ended":false,"known_terminal":false,"version":3,"parking_version":3}).to_string()]).unwrap();
            db.execute(
                "INSERT INTO quota_waiters VALUES(?1,'claude','unknown','preparing',?2,1000)",
                params![
                    unit.id.to_string(),
                    serde_json::to_value(reason).unwrap().as_str().unwrap()
                ],
            )
            .unwrap();
            db.execute(
                "INSERT INTO task_drivers VALUES(?1,?2,?3,1,'driving')",
                params![
                    task.id.to_string(),
                    goal.id.to_string(),
                    project.to_string()
                ],
            )
            .unwrap();
            db.execute("INSERT INTO runtime_epoch VALUES(1,1)", [])
                .unwrap();
            Self { db, task, goal }
        }
        fn observation(&self) -> Option<PhaseWaitingObservation> {
            observe(&self.db, &self.task, &self.goal)
        }
    }

    #[test]
    fn exact_park_facts_project_waiting_without_persisting_task_or_database() {
        for (reason, state) in [
            (WaitReason::Quota, TaskState::WaitingQuota),
            (WaitReason::Capacity, TaskState::WaitingCapacity),
        ] {
            let facts = Facts::new(reason);
            let before = serde_json::to_string(&facts.task).unwrap();
            let changes = facts.db.total_changes();
            let observation = facts.observation();
            assert_eq!(
                observation,
                Some(PhaseWaitingObservation::Waiting {
                    reason,
                    next_due: 1000
                })
            );
            assert_eq!(effective_state(&facts.task, observation.as_ref()), state);
            assert_eq!(facts.task.version, 7);
            assert_eq!(serde_json::to_string(&facts.task).unwrap(), before);
            assert_eq!(facts.db.total_changes(), changes);
        }
    }
    #[test]
    fn inconsistent_stale_or_ambiguous_facts_are_held_for_only_that_task() {
        for sql in [
            "UPDATE managed_phase_readiness SET parking_version=2",
            "UPDATE managed_phase_readiness SET start_ended=1",
            "UPDATE managed_phase_readiness SET known_terminal=1",
            "UPDATE managed_phase_readiness SET body='{}'",
            "UPDATE managed_phase_readiness SET state='held'",
            "UPDATE quota_waiters SET reason='quota'",
            "UPDATE quota_waiters SET account_key='other'",
            "UPDATE quota_waiters SET resume_state='running'",
            "UPDATE runtime_epoch SET epoch=2",
            "UPDATE task_drivers SET state='invalid'",
            "DELETE FROM managed_phase_owners",
            "UPDATE managed_phase_operations SET marker_task_version=8",
            "UPDATE execution_units SET body='{}'",
            "INSERT INTO managed_phase_operations SELECT * FROM managed_phase_operations",
        ] {
            let facts = Facts::new(WaitReason::Capacity);
            facts.db.execute(sql, []).unwrap();
            assert_eq!(
                facts.observation(),
                Some(PhaseWaitingObservation::Held),
                "{sql}"
            );
            assert_eq!(
                effective_state(&facts.task, facts.observation().as_ref()),
                TaskState::Implementing
            );
            let other = Task::new(
                facts.task.project_id,
                facts.goal.id,
                "other".into(),
                "native".into(),
            );
            assert_eq!(observe(&facts.db, &other, &facts.goal), None);
        }
    }
    #[test]
    fn terminal_or_paused_parents_and_absent_operation_keep_original_status() {
        let mut facts = Facts::new(WaitReason::Quota);
        facts.goal.state = GoalState::Paused;
        assert_eq!(facts.observation(), None);
        facts.goal.state = GoalState::Running;
        facts.task.state = TaskState::Cancelled;
        assert_eq!(facts.observation(), None);
        facts.task.state = TaskState::Implementing;
        facts
            .db
            .execute("DELETE FROM managed_phase_operations", [])
            .unwrap();
        assert_eq!(facts.observation(), None);
    }
}
