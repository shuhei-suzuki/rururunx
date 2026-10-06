//! Nongrant SQL head-walk controls. These construct no Runtime, actor, Source,
//! permit, managed launch, Prepared value or Legacy-success authority.
use super::*;
use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Facts {
    db: Connection,
    own: ExecutionUnit,
    project: Project,
    goal: Goal,
    task: Task,
    calls: Arc<AtomicUsize>,
}
impl Facts {
    fn new() -> Self {
        let project = Project::new(
            "head facts".into(),
            "/head-facts".into(),
            "facts".into(),
            "main".into(),
        );
        let goal = Goal::new(project.id, "head facts".into(), vec![]);
        let task = Task::new(project.id, goal.id, "head facts".into(), "native".into());
        let own = ExecutionUnit {
            id: UnitId::new(),
            scope: task.scope(),
            kind: UnitKind::Executor,
            generation: 1,
            owner_epoch: 1,
            version: 1,
            phase: "Implement".into(),
            provider: "claude".into(),
            state: UnitState::Preparing,
            native_effects_open: true,
            result_finalization_open: true,
            work: None,
            cleanup: CleanupOutcome::Unknown,
            disposition: Disposition::Active,
            worktree: "/head-facts".into(),
            branch: None,
            base_sha: "a".repeat(40),
            profile_digest: "b".repeat(64),
            cookie: uuid::Uuid::new_v4().to_string(),
            session_id: None,
            artifact_id: None,
            wait_reason: Some(WaitReason::Capacity),
            capacity_retry_at: None,
            created_at: 1,
            updated_at: 1,
        };
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE projects(id TEXT,version INTEGER,body TEXT);
            CREATE TABLE goals(id TEXT,project_id TEXT,version INTEGER,body TEXT);
            CREATE TABLE tasks(id TEXT,goal_id TEXT,project_id TEXT,version INTEGER,body TEXT);
            CREATE TABLE execution_units(id TEXT,project_id TEXT,goal_id TEXT,task_id TEXT,kind TEXT,generation INTEGER,owner_epoch INTEGER,version INTEGER,native_effects_open INTEGER,result_finalization_open INTEGER,worktree TEXT,branch TEXT,body TEXT);
            CREATE TABLE cleanup_observations(unit_id TEXT,at INTEGER,body TEXT);
            CREATE TABLE quota_leases(unit_id TEXT,active INTEGER);
            CREATE TABLE quota_waiters(unit_id TEXT,provider TEXT,account_key TEXT,reason TEXT,next_due INTEGER,fairness_sequence INTEGER,resume_state TEXT);
            CREATE TABLE managed_phase_operations(unit_id TEXT,operation_id TEXT,phase_open INTEGER);
            CREATE TABLE managed_phase_readiness(operation_id TEXT,state TEXT,version INTEGER,parking_version INTEGER);
            CREATE TABLE runtime_epoch(singleton INTEGER,epoch INTEGER);
            INSERT INTO runtime_epoch VALUES(1,2);").unwrap();
        db.execute(
            "INSERT INTO projects VALUES(?1,?2,?3)",
            params![
                project.id.to_string(),
                project.version,
                serde_json::to_string(&project).unwrap()
            ],
        )
        .unwrap();
        db.execute(
            "INSERT INTO goals VALUES(?1,?2,?3,?4)",
            params![
                goal.id.to_string(),
                project.id.to_string(),
                goal.version,
                serde_json::to_string(&goal).unwrap()
            ],
        )
        .unwrap();
        db.execute(
            "INSERT INTO tasks VALUES(?1,?2,?3,?4,?5)",
            params![
                task.id.to_string(),
                goal.id.to_string(),
                project.id.to_string(),
                task.version,
                serde_json::to_string(&task).unwrap()
            ],
        )
        .unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        // Only count the real validator's epoch read. The observation never
        // changes a query result or substitutes a validator verdict.
        db.authorizer(Some(move |context: AuthContext<'_>| {
            if matches!(
                context.action,
                AuthAction::Read {
                    table_name: "runtime_epoch",
                    column_name: "epoch"
                }
            ) {
                observed.fetch_add(1, Ordering::SeqCst);
            }
            Authorization::Allow
        }));
        Self {
            db,
            own,
            project,
            goal,
            task,
            calls,
        }
    }
    fn candidate(&self, sequence: i64, marked: bool) -> Row {
        let mut unit = self.own.clone();
        unit.id = UnitId::new();
        let id = unit.id.to_string();
        self.db
            .execute(
                "INSERT INTO execution_units VALUES(?1,?2,?3,?4,'executor',1,1,1,1,1,?5,NULL,?6)",
                params![
                    id,
                    self.project.id.to_string(),
                    self.goal.id.to_string(),
                    self.task.id.to_string(),
                    unit.worktree.to_string_lossy(),
                    serde_json::to_string(&unit).unwrap()
                ],
            )
            .unwrap();
        self.db.execute("INSERT INTO quota_waiters VALUES(?1,'claude','unknown','capacity',1000,?2,'preparing')",params![id,sequence]).unwrap();
        if marked {
            self.db
                .execute(
                    "INSERT INTO managed_phase_operations VALUES(?1,?1,1)",
                    [&id],
                )
                .unwrap();
            self.db
                .execute(
                    "INSERT INTO managed_phase_readiness VALUES(?1,'parked',3,3)",
                    [&id],
                )
                .unwrap();
        }
        vec![
            t(id),
            t("claude"),
            t("unknown"),
            t("capacity"),
            i(1000),
            i(sequence),
            t("preparing"),
            t("executor"),
        ]
    }
    fn walk(&mut self, rows: &[Row], own_waiter: Option<&Row>) -> Result<bool> {
        let changes = self.db.total_changes();
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = preceding_head_inventory(
            &tx,
            HeadInventory {
                own: &self.own,
                own_waiter,
                last_role: "reviewer",
                waiters: rows,
                at: 1000,
                project_max: 4,
                executor_throttled: false,
            },
        );
        assert_eq!(
            tx.total_changes(),
            changes,
            "head observation must write nothing"
        );
        result
    }
}

#[test]
fn nongrant_e1_malformed_goal_task_abort_before_validator_and_leave_rows_unchanged() {
    for table in ["goals", "tasks"] {
        let mut facts = Facts::new();
        let candidate = facts.candidate(1, false);
        facts
            .db
            .execute(&format!("UPDATE {table} SET body='{{}}'"), [])
            .unwrap();
        {
            let tx = facts.db.transaction().unwrap();
            let id = text(&candidate, 0).unwrap().parse().unwrap();
            candidate_shape(&tx, id).unwrap();
            let unit = unit_tx(&tx, id).unwrap();
            assert!(
                !crate::state::execution::quotas::project_capacity_blocked(&tx, &unit, 4).unwrap()
            );
        }
        let result = facts.walk(&[candidate], None);
        assert!(
            result.is_err(),
            "malformed {table} must abort before the validator"
        );
        let error = result.unwrap_err();
        assert!(format!("{error:#}").contains("legacy head structure"));
        assert_eq!(facts.calls.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn nongrant_e1_owner_identity_and_version_mismatch_abort_before_validator() {
    for change in [
        "UPDATE goals SET project_id='foreign'",
        "UPDATE tasks SET goal_id='foreign'",
        "UPDATE projects SET version=version+1",
        "UPDATE goals SET version=version+1",
        "UPDATE tasks SET version=version+1",
    ] {
        let mut facts = Facts::new();
        let candidate = facts.candidate(1, false);
        facts.db.execute(change, []).unwrap();
        let error = facts.walk(&[candidate], None).unwrap_err();
        assert!(format!("{error:#}").contains("owner identity or version differs"));
        assert_eq!(facts.calls.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn nongrant_e1_exact_legacy_refusal_is_passed_over_and_ninth_call_aborts() {
    for count in [8, 9] {
        let mut facts = Facts::new();
        let rows = (0..count)
            .map(|n| facts.candidate(n, false))
            .collect::<Vec<_>>();
        {
            let tx = facts.db.transaction().unwrap();
            let id = text(&rows[0], 0).unwrap().parse().unwrap();
            let candidate = unit_tx(&tx, id).unwrap();
            structural_legacy_owners(&tx, &candidate).unwrap();
            let error = validate_authority(&tx, &candidate.authority(), true, false).unwrap_err();
            assert!(error.to_string().contains("execution owner epoch retired"));
        }
        facts.calls.store(0, Ordering::SeqCst);
        let result = facts.walk(&rows, None);
        if count == 8 {
            assert!(!result.unwrap());
        } else {
            assert!(
                result.is_err(),
                "ninth Legacy call must abort before invoking it"
            );
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("ninth call required")
            );
        }
        assert_eq!(facts.calls.load(Ordering::SeqCst), 8);
    }
}

#[test]
fn nongrant_e1_marked_head_does_not_decode_legacy_goal_or_validate_generic_authority() {
    let mut facts = Facts::new();
    let row = facts.candidate(1, true);
    facts.db.execute("UPDATE goals SET body='{}'", []).unwrap();
    assert!(facts.walk(&[row], None).unwrap());
    assert_eq!(facts.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn nongrant_e1_actual_and_virtual_self_stop_before_later_malformed_candidates() {
    for actual in [false, true] {
        let mut facts = Facts::new();
        let late = facts.candidate(1001, true);
        facts
            .db
            .execute("UPDATE projects SET body='{}'", [])
            .unwrap();
        let mut own_row = late.clone();
        own_row[0] = t(facts.own.id.to_string());
        own_row[5] = i(10);
        let result = facts.walk(&[late], actual.then_some(&own_row));
        assert!(
            matches!(result, Ok(false)),
            "later candidate must remain unexamined"
        );
        assert_eq!(facts.calls.load(Ordering::SeqCst), 0);
    }
    let mut facts = Facts::new();
    let early = facts.candidate(999, true);
    assert!(facts.walk(&[early], None).unwrap());
}

#[test]
fn nongrant_e1_whole_plan_charge_includes_both_branches_and_refuses_overflow() {
    let before = PairRow {
        table: "managed_phase_readiness",
        values: vec![t("before")],
    };
    let branch = QuotaBranch {
        after: Images {
            pool: Some(default_pool("claude")),
            waiter: None,
            lease: None,
        },
        readiness: PairRow {
            table: "managed_phase_readiness",
            values: vec![t("after")],
        },
        unit: None,
        decision: Decision::Admit { probe: false },
        mutation: None,
    };
    let charge = branch_bytes(&branch, &before).unwrap();
    assert_eq!(
        charge,
        row_bytes(branch.after.pool.as_ref().unwrap()).unwrap() + 5
    );
    let mut bytes = PLAN_BYTES - charge;
    charge_plan_bytes(&mut bytes, charge).unwrap();
    assert!(
        charge_plan_bytes(&mut bytes, charge).is_err(),
        "second owned branch must be charged"
    );
    let mut bytes = usize::MAX;
    assert!(charge_plan_bytes(&mut bytes, 1).is_err());
    assert_eq!(row_bytes(&[t("four"), i(1), SqlValue::Null]).unwrap(), 20);
}

#[test]
fn nongrant_e1_marked_head_stops_after_seven_exact_legacy_refusals() {
    let mut facts = Facts::new();
    let mut rows = (0..7)
        .map(|n| facts.candidate(n, false))
        .collect::<Vec<_>>();
    rows.push(facts.candidate(7, true));
    rows.push(facts.candidate(8, false));
    assert!(facts.walk(&rows, None).unwrap());
    assert_eq!(facts.calls.load(Ordering::SeqCst), 7);
}
