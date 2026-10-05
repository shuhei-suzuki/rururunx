use super::*;
use crate::config::WorkflowClass;
use std::path::PathBuf;

fn fixture() -> (Store, Task, u64) {
    let mut store = Store::memory().unwrap();
    let mut p = Project::new(
        "project".into(),
        PathBuf::from("/tmp/rrx-source"),
        "git-local".into(),
        "main".into(),
    );
    store.put_project(&mut p).unwrap();
    let mut g = Goal::new(
        p.id,
        "goal".into(),
        vec![CompletionCriterion {
            id: "result".into(),
            description: "accepted commit".into(),
            evidence: None,
            satisfied: false,
        }],
    );
    store.put_goal(&mut g).unwrap();
    let mut t = Task::new(p.id, g.id, "task".into(), "codex".into());
    t.workflow = WorkflowClass::Quick;
    store.put_task(&mut t).unwrap();
    let (_, epoch) = store.begin_execution_epoch().unwrap();
    (store, t, epoch)
}
fn draft(task: &Task, epoch: u64) -> ExecutionUnit {
    let id = UnitId::new();
    let at = now_ms();
    ExecutionUnit {
        id,
        scope: task.scope(),
        kind: UnitKind::Executor,
        generation: 0,
        owner_epoch: epoch,
        version: 0,
        phase: "Implement".into(),
        provider: "codex".into(),
        state: UnitState::Reserved,
        native_effects_open: true,
        result_finalization_open: true,
        work: None,
        cleanup: CleanupOutcome::Unknown,
        disposition: Disposition::Active,
        worktree: PathBuf::from(format!("/tmp/rrx-source/worktree/{}-{id}", task.id)),
        branch: Some(format!("rrx/{}/{id}", task.id)),
        base_sha: "a".repeat(40),
        profile_digest: "b".repeat(64),
        cookie: Uuid::new_v4().to_string(),
        session_id: None,
        artifact_id: None,
        wait_reason: None,
        created_at: at,
        updated_at: at,
    }
}

#[test]
fn normal_terminal_cleanup_keeps_finalization_and_result_publication_races_cancel() {
    let (mut store, t, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&t, epoch), t.version)
        .unwrap();
    let a = unit.authority();
    let known = store
        .finish_execution(&a, WorkOutcome::Success, Disposition::Completed)
        .unwrap();
    assert!(!known.native_effects_open && known.result_finalization_open);
    store
        .record_execution_cleanup(&CleanupObservation {
            unit_id: unit.id,
            at: now_ms(),
            outcome: CleanupOutcome::Unknown,
            coverage: BTreeMap::from([("process".into(), "inspection unavailable".into())]),
            remaining: vec![],
            errors: vec![],
        })
        .unwrap();
    assert_eq!(
        store.execution_unit(unit.id).unwrap().work,
        Some(WorkOutcome::Success)
    );
    let current_authority = store.execution_unit(unit.id).unwrap().authority();
    assert!(
        store
            .validate_execution(&current_authority, false, true)
            .is_ok()
    );
    store.retire_execution(&current_authority, false).unwrap();
    assert!(store.validate_execution(&a, false, true).is_err());
    let current = store.task(t.id).unwrap().unwrap();
    let retry = store
        .reserve_execution(draft(&current, epoch), current.version)
        .unwrap();
    assert_ne!(retry.worktree, unit.worktree);
    assert!(retry.generation > unit.generation);
    assert_eq!(
        store.execution_unit(unit.id).unwrap().work,
        Some(WorkOutcome::Success)
    );
}

#[test]
fn pre_open_legacy_writer_and_cached_statement_cannot_write_after_upgrade() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    // Build the genuine old schema with a connection that has no new writer function.
    let old = Connection::open(&path).unwrap();
    old.execute_batch(include_str!("../schema.sql")).unwrap();
    old.pragma_update(None, "application_id", APPLICATION_ID)
        .unwrap();
    old.pragma_update(None, "user_version", 3).unwrap();
    let mut project = Project::new(
        "old".into(),
        PathBuf::from("/p"),
        "git-local".into(),
        "main".into(),
    );
    project.version = 1;
    old.execute(
        "INSERT INTO projects(id,root,version,body) VALUES(?1,'/p',1,?2)",
        params![
            project.id.to_string(),
            serde_json::to_string(&project).unwrap()
        ],
    )
    .unwrap();
    let mut cached=old.prepare("UPDATE projects SET version=version+1,body=json_set(body,'$.version',version+1) WHERE root='/p'").unwrap();
    cached.execute([]).unwrap();
    let upgraded = Store::open(&path).unwrap();
    assert_eq!(upgraded.schema_version().unwrap(), 4);
    assert!(cached.execute([]).is_err());
    for sql in [
        "UPDATE projects SET version=version+1 WHERE root='/p'",
        "DELETE FROM projects WHERE root='/p'",
        "INSERT INTO projects(id,root,version,body) VALUES('q','/q',1,'{}')",
        "INSERT OR REPLACE INTO projects(id,root,version,body) VALUES('p','/p',1,'{}')",
    ] {
        assert!(
            old.execute(sql, []).is_err(),
            "legacy write succeeded: {sql}"
        );
    }
    assert_eq!(
        old.query_row("SELECT version FROM projects WHERE root='/p'", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
}

#[test]
fn two_tasks_keep_independent_authority_and_orphan_port_leases() {
    let (mut store, t, epoch) = fixture();
    let first = store
        .reserve_execution(draft(&t, epoch), t.version)
        .unwrap();
    let mut other = Task::new(t.project_id, t.goal_id, "sibling".into(), "codex".into());
    store.put_task(&mut other).unwrap();
    let second = store
        .reserve_execution(draft(&other, epoch), other.version)
        .unwrap();
    let lease = ResourceLease {
        id: LeaseId::new(),
        unit_id: first.id,
        scope: first.scope.clone(),
        kind: ResourceKind::Ports,
        namespace: "host".into(),
        value: "30000-30031".into(),
        port_start: Some(30000),
        port_end: Some(30031),
        state: LeaseState::Reserved,
        version: 1,
    };
    store
        .reserve_execution_leases(&first.authority(), std::slice::from_ref(&lease))
        .unwrap();
    store.retire_execution(&first.authority(), false).unwrap();
    assert!(
        store
            .validate_execution(&second.authority(), true, false)
            .is_ok()
    );
    let collision = ResourceLease {
        unit_id: second.id,
        scope: second.scope.clone(),
        id: LeaseId::new(),
        ..lease.clone()
    };
    assert!(
        store
            .reserve_execution_leases(&second.authority(), &[collision])
            .is_err()
    );
    assert!(
        store
            .release_execution_lease(lease.id, second.id, 1, true)
            .is_err()
    );
    assert_eq!(
        store.execution_leases(first.id).unwrap()[0].state,
        LeaseState::Reserved
    );
}

#[test]
fn stale_callbacks_and_generic_task_cancellation_cannot_reopen_authority() {
    let (mut store, t, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&t, epoch), t.version)
        .unwrap();
    let preparing = store
        .transition_execution(&unit.authority(), UnitState::Preparing)
        .unwrap();
    assert!(
        store
            .transition_execution(&unit.authority(), UnitState::DispatchPending)
            .is_err()
    );
    let pending = store
        .transition_execution(&preparing.authority(), UnitState::DispatchPending)
        .unwrap();
    let running = store
        .transition_execution(&pending.authority(), UnitState::Running)
        .unwrap();
    assert!(
        store
            .transition_execution(&running.authority(), UnitState::Preparing)
            .is_err()
    );
    let mut task = store.task(t.id).unwrap().unwrap();
    task.state = TaskState::Cancelled;
    store.put_task(&mut task).unwrap();
    assert!(
        store
            .validate_execution(&running.authority(), true, false)
            .is_err()
    );
    let old = store.execution_unit(unit.id).unwrap();
    assert!(!old.native_effects_open && !old.result_finalization_open);
}

#[test]
fn retirement_is_allowed_after_goal_pause_but_resume_cannot_revive_unit() {
    let (mut store, t, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&t, epoch), t.version)
        .unwrap();
    let mut goal = store.goal(t.goal_id).unwrap().unwrap();
    goal.state = GoalState::Paused;
    store.put_goal(&mut goal).unwrap();
    assert!(
        store
            .validate_execution(&unit.authority(), true, false)
            .is_err()
    );
    store.retire_execution(&unit.authority(), false).unwrap();
    goal.state = GoalState::Running;
    store.put_goal(&mut goal).unwrap();
    assert!(
        store
            .validate_execution(&unit.authority(), true, false)
            .is_err()
    );
}

#[test]
fn ambiguous_quota_updates_cannot_clear_exhaustion_and_terminal_probe_is_released() {
    let (mut store, t, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&t, epoch), t.version)
        .unwrap();
    let at = now_ms();
    let reset = at + 60000;
    let observation = QuotaObservation {
        provider: "codex".into(),
        account_key: "unknown".into(),
        bucket: "all".into(),
        window_id: "old".into(),
        status: QuotaStatus::Exhausted,
        used_percent: Some(100.0),
        resets_at: Some(reset),
        observed_at: at,
        source_version: "fixture".into(),
        confirmed_subscription: true,
    };
    store.observe_quota(&observation).unwrap();
    for state in [QuotaStatus::Stale, QuotaStatus::Unknown] {
        store
            .observe_quota(&QuotaObservation {
                status: state,
                observed_at: at + 1,
                ..observation.clone()
            })
            .unwrap();
        assert_eq!(
            store.quota_observations("codex", "unknown").unwrap()[0].status,
            QuotaStatus::Exhausted
        );
    }
    assert!(matches!(
        store
            .reserve_execution_quota(&unit.authority(), "codex", "unknown", 6, 2, 3, at + 2)
            .unwrap(),
        quotas::QuotaAdmission::Waiting {
            reason: WaitReason::Quota,
            ..
        }
    ));
    let current = store.execution_unit(unit.id).unwrap();
    assert_eq!(
        store
            .reserve_execution_quota(&current.authority(), "codex", "unknown", 6, 2, 3, reset + 1)
            .unwrap(),
        quotas::QuotaAdmission::Admitted
    );
    let current = store.execution_unit(unit.id).unwrap();
    store.retire_execution(&current.authority(), false).unwrap();
    let task = store.task(t.id).unwrap().unwrap();
    let next = store
        .reserve_execution(draft(&task, epoch), task.version)
        .unwrap();
    assert_eq!(
        store
            .reserve_execution_quota(
                &next.authority(),
                "codex",
                "unknown",
                6,
                2,
                3,
                reset + 1800001
            )
            .unwrap(),
        quotas::QuotaAdmission::Admitted
    );
}

#[test]
fn malformed_legacy_authority_rolls_back_schema_upgrade() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let old = Connection::open(&path).unwrap();
    old.execute_batch(include_str!("../schema.sql")).unwrap();
    old.pragma_update(None, "application_id", APPLICATION_ID)
        .unwrap();
    old.pragma_update(None, "user_version", 3).unwrap();
    old.execute(
        "INSERT INTO projects(id,root,version,body) VALUES('invalid','/p',1,'{}')",
        [],
    )
    .unwrap();
    assert!(Store::open(&path).is_err());
    assert_eq!(
        old.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        old.query_row(
            "SELECT COUNT(*) FROM sqlite_schema WHERE name='execution_units'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
}

#[test]
fn body_corruption_cannot_redirect_unit_or_lease_authority() {
    let (mut store, t, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&t, epoch), t.version)
        .unwrap();
    let mut corrupt = unit.clone();
    corrupt.worktree = PathBuf::from("/foreign");
    store
        .connection
        .execute(
            "UPDATE execution_units SET body=?1 WHERE id=?2",
            params![
                serde_json::to_string(&corrupt).unwrap(),
                unit.id.to_string()
            ],
        )
        .unwrap();
    assert!(store.execution_unit(unit.id).is_err());
    assert!(store.retire_execution(&unit.authority(), false).is_err());
}

#[test]
fn quota_bucket_body_cannot_impersonate_another_indexed_row() {
    let (mut store, t, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&t, epoch), t.version)
        .unwrap();
    let a = QuotaObservation {
        provider: "codex".into(),
        account_key: "unknown".into(),
        bucket: "a".into(),
        window_id: "old".into(),
        status: QuotaStatus::Exhausted,
        used_percent: Some(100.0),
        resets_at: None,
        observed_at: now_ms(),
        source_version: "fixture".into(),
        confirmed_subscription: true,
    };
    let b = QuotaObservation {
        bucket: "b".into(),
        status: QuotaStatus::Available,
        used_percent: Some(1.0),
        ..a.clone()
    };
    store.observe_quota(&a).unwrap();
    store.observe_quota(&b).unwrap();
    assert_eq!(
        store.quota_observations("codex", "unknown").unwrap().len(),
        2
    );
    store
        .connection
        .execute(
            "UPDATE quota_windows SET body=?1 WHERE bucket='a'",
            [serde_json::to_string(&b).unwrap()],
        )
        .unwrap();
    assert!(store.quota_observations("codex", "unknown").is_err());
    assert!(
        store
            .reserve_execution_quota(
                &unit.authority(),
                "codex",
                "unknown",
                6,
                2,
                3,
                now_ms() + 60001
            )
            .is_err()
    );
    assert!(store.observe_quota(&a).is_err());
}

#[test]
fn exhaustion_without_reset_recovers_only_through_the_current_pool_probe() {
    let (mut store, t, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&t, epoch), t.version)
        .unwrap();
    let at = now_ms();
    let exhausted = QuotaObservation {
        provider: "codex".into(),
        account_key: "unknown".into(),
        bucket: "all".into(),
        window_id: "unknown-all".into(),
        status: QuotaStatus::Exhausted,
        used_percent: Some(100.0),
        resets_at: None,
        observed_at: at,
        source_version: "fixture".into(),
        confirmed_subscription: true,
    };
    store.observe_quota(&exhausted).unwrap();
    let available = QuotaObservation {
        status: QuotaStatus::Available,
        used_percent: Some(10.0),
        observed_at: at + 60001,
        ..exhausted.clone()
    };
    store.observe_quota(&available).unwrap();
    assert_eq!(
        store.quota_observations("codex", "unknown").unwrap()[0].status,
        QuotaStatus::Exhausted
    );
    assert!(
        store
            .observe_quota_from_probe(&available, &unit.authority())
            .is_err()
    );
    assert_eq!(
        store
            .reserve_execution_quota(&unit.authority(), "codex", "unknown", 6, 2, 3, at + 60001)
            .unwrap(),
        quotas::QuotaAdmission::Admitted
    );
    let probe = store.execution_unit(unit.id).unwrap();
    store
        .observe_quota_from_probe(&available, &probe.authority())
        .unwrap();
    assert_eq!(
        store.quota_observations("codex", "unknown").unwrap()[0].status,
        QuotaStatus::Available
    );
    store.retire_execution(&probe.authority(), false).unwrap();
    assert!(
        store
            .observe_quota_from_probe(&available, &probe.authority())
            .is_err()
    );
}

#[test]
fn governing_instruction_change_fences_effects_but_preserves_historical_retirement() {
    let (mut store, t, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&t, epoch), t.version)
        .unwrap();
    let mut goal = store.goal(t.goal_id).unwrap().unwrap();
    goal.blockers.push("bookkeeping".into());
    store.put_goal(&mut goal).unwrap();
    assert!(
        store
            .validate_execution(&unit.authority(), true, false)
            .is_ok()
    );
    goal.constraints.push("new accepted constraint".into());
    store.put_goal(&mut goal).unwrap();
    assert!(
        store
            .validate_execution(&unit.authority(), true, false)
            .is_err()
    );
    assert!(
        store
            .finish_execution(
                &unit.authority(),
                WorkOutcome::Success,
                Disposition::Completed
            )
            .is_err()
    );
    store.retire_execution(&unit.authority(), false).unwrap();
    let current = store.task(t.id).unwrap().unwrap();
    let retry = store
        .reserve_execution(draft(&current, epoch), current.version)
        .unwrap();
    let mut project = store.project(t.project_id).unwrap().unwrap();
    project.rule_refs.push(PathBuf::from("/new/rules"));
    store.put_project(&mut project).unwrap();
    assert!(
        store
            .validate_execution(&retry.authority(), true, false)
            .is_err()
    );
    store.retire_execution(&retry.authority(), false).unwrap();
}

#[test]
fn unknown_remote_effect_gates_its_target_phase_without_blocking_local_retry() {
    let (mut store, t, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&t, epoch), t.version)
        .unwrap();
    let effect = ManagedEffect {
        id: OperationId::new(),
        unit_id: unit.id,
        scope: unit.scope.clone(),
        kind: "publish".into(),
        idempotency_key: "first-publish".into(),
        expected_target: "origin/pr/123".into(),
        state: EffectState::Pending,
        receipt: BTreeMap::new(),
        version: 1,
    };
    store
        .reserve_managed_effect(&unit.authority(), &effect)
        .unwrap();
    store.retire_execution(&unit.authority(), false).unwrap();
    let current = store.task(t.id).unwrap().unwrap();
    let retry = store
        .reserve_execution(draft(&current, epoch), current.version)
        .unwrap();
    let later = ManagedEffect {
        id: OperationId::new(),
        unit_id: retry.id,
        idempotency_key: "later-publish".into(),
        ..effect.clone()
    };
    assert!(
        store
            .reserve_managed_effect(&retry.authority(), &later)
            .is_err()
    );
    let local = ManagedEffect {
        kind: "local-test".into(),
        idempotency_key: "local-test".into(),
        ..later
    };
    store
        .reserve_managed_effect(&retry.authority(), &local)
        .unwrap();
}
