use super::*;
use crate::config::WorkflowClass;
use std::path::PathBuf;

thread_local! {
    /// The fixture Project's worktree root (`draft` places Units under it);
    /// each test runs on its own thread.
    static WORKTREE_ROOT: std::cell::RefCell<PathBuf> =
        std::cell::RefCell::new(PathBuf::from("/tmp/rrx-source/worktree"));
}
/// FM §8.1 L: a legacy Quick codex Task (file-backed, migrated rows).
fn fixture() -> (crate::runtime::LegacyFixture, Store, Task, u64) {
    let (legacy, mut store) = crate::runtime::legacy_store(vec![crate::runtime::LegacyTask {
        key: "task",
        executor: "codex",
        workflow: WorkflowClass::Quick,
        risk: crate::domain::RiskClass::R0,
    }]);
    let t = legacy.task();
    let root = store.project(t.project_id).unwrap().unwrap().worktree_root;
    WORKTREE_ROOT.with(|r| *r.borrow_mut() = root);
    let (_, epoch) = store.begin_execution_epoch().unwrap();
    (legacy, store, t, epoch)
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
        worktree: WORKTREE_ROOT.with(|r| r.borrow().join(format!("{}-{id}", task.id))),
        branch: Some(format!("rrx/{}/{id}", task.id)),
        base_sha: "a".repeat(40),
        profile_digest: "b".repeat(64),
        cookie: Uuid::new_v4().to_string(),
        session_id: None,
        artifact_id: None,
        wait_reason: None,
        capacity_retry_at: None,
        created_at: at,
        updated_at: at,
    }
}

fn session(unit: &ExecutionUnit) -> Session {
    Session {
        id: SessionId::new(),
        scope: unit.scope.clone(),
        agent: unit.provider.clone(),
        provider: unit.provider.clone(),
        role: SessionRole::Executor,
        native_ref: None,
        pid: None,
        worktree: unit.worktree.clone(),
        state: SessionState::Starting,
        model: None,
        effort: None,
        recovery: json!({}),
        started_at: now_ms(),
    }
}

// Scalar/pure predicate harness only: no execution Unit, owner, Driver or
// accepted Goal row is installed, and no marked Native proof is issued.
fn nongrant_factoring_fixture() -> (Connection, Project, Goal, Task, ExecutionUnit) {
    let project = Project::new(
        "project".into(),
        PathBuf::from("/tmp/rrx-source"),
        "git-local".into(),
        "main".into(),
    );
    let goal = Goal::new(project.id, "goal".into(), vec![]);
    let mut task = Task::new(project.id, goal.id, "task".into(), "codex".into());
    let mut unit = draft(&task, 1);
    unit.generation = 1;
    unit.version = 1;
    task.worktree = Some(unit.worktree.clone());
    task.branch = unit.branch.clone();
    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(
            "CREATE TABLE runtime_epoch(singleton INTEGER, epoch INTEGER);
        INSERT INTO runtime_epoch VALUES(1,1);
        CREATE TABLE task_execution(task_id TEXT, generation INTEGER);
        CREATE TABLE execution_context(unit_id TEXT, governing_digest TEXT);",
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO task_execution VALUES(?1,1)",
            [task.id.to_string()],
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO execution_context VALUES(?1,?2)",
            params![
                unit.id.to_string(),
                governing_digest(&project, &goal).unwrap()
            ],
        )
        .unwrap();
    (connection, project, goal, task, unit)
}

#[test]
fn nongrant_factoring_rejects_identity_epoch_and_generation_drift() {
    let (connection, _, _, task, unit) = nongrant_factoring_fixture();
    validate_unit_authority_facts(&connection, &unit.authority(), &unit).unwrap();
    let mut foreign = unit.authority();
    foreign.record_version += 1;
    assert!(validate_unit_authority_facts(&connection, &foreign, &unit).is_err());
    connection
        .execute(
            "UPDATE task_execution SET generation=2 WHERE task_id=?1",
            [task.id.to_string()],
        )
        .unwrap();
    assert!(validate_unit_authority_facts(&connection, &unit.authority(), &unit).is_err());
    connection
        .execute(
            "UPDATE task_execution SET generation=1 WHERE task_id=?1",
            [task.id.to_string()],
        )
        .unwrap();
    connection
        .execute("UPDATE runtime_epoch SET epoch=2 WHERE singleton=1", [])
        .unwrap();
    assert!(validate_unit_authority_facts(&connection, &unit.authority(), &unit).is_err());
    assert_eq!(
        connection
            .query_row("SELECT epoch FROM runtime_epoch", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        connection
            .query_row("SELECT generation FROM task_execution", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn nongrant_factoring_parent_activity_and_executor_projection_are_conjunctive() {
    let (_, project, goal, task, unit) = nongrant_factoring_fixture();
    let original = serde_json::to_value(&task).unwrap();
    validate_parent_activity_facts(&unit, &project, &goal, &task).unwrap();
    for state in [ProjectState::Blocked, ProjectState::Removed] {
        let mut changed = project.clone();
        changed.state = state;
        assert!(validate_parent_activity_facts(&unit, &changed, &goal, &task).is_err());
    }
    for state in [
        GoalState::Paused,
        GoalState::Completed,
        GoalState::Cancelled,
        GoalState::Failed,
    ] {
        let mut changed = goal.clone();
        changed.state = state;
        assert!(validate_parent_activity_facts(&unit, &project, &changed, &task).is_err());
    }
    for state in [
        TaskState::Completed,
        TaskState::Failed,
        TaskState::Cancelled,
    ] {
        let mut changed = task.clone();
        changed.state = state;
        assert!(validate_parent_activity_facts(&unit, &project, &goal, &changed).is_err());
    }
    let mut foreign = task.clone();
    foreign.id = TaskId::new();
    assert!(validate_parent_activity_facts(&unit, &project, &goal, &foreign).is_err());
    let mut changed = task.clone();
    changed.worktree = None;
    assert!(validate_parent_activity_facts(&unit, &project, &goal, &changed).is_err());
    let mut changed = task.clone();
    changed.branch = Some("foreign".into());
    assert!(validate_parent_activity_facts(&unit, &project, &goal, &changed).is_err());
    let mut reviewer = unit.clone();
    reviewer.kind = UnitKind::Reviewer;
    // The ordinary readonly-role predicate does not adopt executor projection.
    validate_parent_activity_facts(&reviewer, &project, &goal, &changed).unwrap();
    assert_eq!(original, serde_json::to_value(&task).unwrap());
}

#[test]
fn nongrant_factoring_governing_digest_is_original_and_read_only() {
    let (connection, project, mut goal, _, unit) = nongrant_factoring_fixture();
    let original = governing_digest(&project, &goal).unwrap();
    validate_governing_context_facts(&connection, &unit, &original).unwrap();
    assert!(validate_governing_context_facts(&connection, &unit, "foreign").is_err());
    goal.constraints.push("new accepted constraint".into());
    let changed = governing_digest(&project, &goal).unwrap();
    assert_ne!(original, changed);
    assert!(validate_governing_context_facts(&connection, &unit, &changed).is_err());
    validate_governing_context_facts(&connection, &unit, &original).unwrap();
    assert_eq!(
        original,
        connection
            .query_row(
                "SELECT governing_digest FROM execution_context WHERE unit_id=?1",
                [unit.id.to_string()],
                |r| r.get::<_, String>(0),
            )
            .unwrap()
    );
}

#[test]
fn nongrant_factoring_native_open_predicate_never_accepts_closed_flag() {
    let (_, _, _, _, unit) = nongrant_factoring_fixture();
    validate_native_effect_open(&unit).unwrap();
    let mut closed = unit.clone();
    closed.native_effects_open = false;
    assert!(validate_native_effect_open(&closed).is_err());
    assert_eq!(
        serde_json::to_value(unit).unwrap()["native_effects_open"],
        true
    );
}

#[test]
fn workflow_source_bootstrap_refuses_session_capacity_and_non_git_intents() {
    let (_fixture, mut store, task, epoch) = fixture();
    let mut spec = draft(&task, epoch);
    spec.phase = WORKFLOW_SOURCE_BOOTSTRAP.into();
    let unit = store.reserve_execution(spec, task.version).unwrap();
    let unit = store
        .transition_execution(&unit.authority(), UnitState::Preparing)
        .unwrap();
    assert!(
        store
            .register_execution_session(&unit.authority(), &session(&unit))
            .is_err()
    );
    assert!(
        store
            .reserve_execution_quota(&unit.authority(), "codex", "unknown", 6, 2, 3, now_ms())
            .is_err()
    );
    for kind in ["native_version", "docker_probe"] {
        assert!(
            store
                .reserve_execution_helper(
                    &unit.authority(),
                    OperationId::new(),
                    true,
                    &unit.worktree,
                    kind
                )
                .is_err()
        );
    }
    let effect = ManagedEffect {
        id: OperationId::new(),
        unit_id: unit.id,
        scope: unit.scope.clone(),
        kind: "native_input".into(),
        idempotency_key: "input".into(),
        expected_target: "turn".into(),
        state: EffectState::Pending,
        receipt: BTreeMap::new(),
        version: 1,
    };
    assert!(
        store
            .reserve_managed_effect(&unit.authority(), &effect)
            .is_err()
    );
    assert!(store.managed_effects(unit.id).unwrap().is_empty());
    assert_eq!(store.execution_unit(unit.id).unwrap().version, unit.version);
    assert_eq!(
        store
            .records(&task.scope(), RecordKind::Session)
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        store
            .connection
            .query_row("SELECT COUNT(*) FROM quota_pools", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        0
    );
    // The allowed helper still gets a real, journaled intent.
    let helper = OperationId::new();
    store
        .reserve_execution_helper(
            &unit.authority(),
            helper,
            true,
            &unit.worktree,
            "git_helper",
        )
        .unwrap();
    assert_eq!(store.managed_effect(helper).unwrap().kind, "git_helper");
}

#[test]
fn schema5_replaces_contract4_guards_and_fences_already_open_writer() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    // Genuine schema-4 layout and connection-local contract-4 function.
    let old = Connection::open(&path).unwrap();
    old.create_scalar_function(
        "rrx_writer_contract_version",
        0,
        rusqlite::functions::FunctionFlags::SQLITE_UTF8
            | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC
            | rusqlite::functions::FunctionFlags::SQLITE_INNOCUOUS,
        |_| Ok(4_i64),
    )
    .unwrap();
    old.execute_batch(include_str!("../schema.sql")).unwrap();
    old.execute_batch(include_str!("../execution.sql")).unwrap();
    old.execute(
        "INSERT INTO runtime_epoch(singleton,instance_id,epoch) VALUES(1,?1,0)",
        [Uuid::new_v4().to_string()],
    )
    .unwrap();
    for table in MUTABLE_TABLES.iter().filter(|table| {
        !crate::state::runtime::TABLES.contains(table)
            && !crate::state::managed_binding::TABLES.contains(table)
            && !matches!(
                **table,
                "native_invocations"
                    | "native_results"
                    | "source_recoveries"
                    | "verification_profiles"
                    | "workflow_verification_contracts"
                    | "verification_runs"
                    | "verification_commands"
            )
    }) {
        for action in ["INSERT", "UPDATE", "DELETE"] {
            old.execute_batch(&format!("CREATE TRIGGER writer_{table}_{action} BEFORE {action} ON {table} WHEN rrx_writer_contract_version()<>4 BEGIN SELECT RAISE(ABORT,'incompatible rrx writer contract'); END;")).unwrap();
        }
    }
    old.pragma_update(None, "application_id", APPLICATION_ID)
        .unwrap();
    old.pragma_update(None, "user_version", 4).unwrap();
    let mut cached = old
        .prepare("UPDATE runtime_epoch SET epoch=epoch+1 WHERE singleton=1")
        .unwrap();
    cached.execute([]).unwrap();
    let mut current = Store::open(&path).unwrap();
    assert_eq!(current.schema_version().unwrap(), SCHEMA_VERSION);
    assert!(cached.execute([]).is_err());
    assert!(
        old.execute(
            "UPDATE runtime_epoch SET epoch=epoch+1 WHERE singleton=1",
            []
        )
        .is_err()
    );
    assert_eq!(
        old.query_row("SELECT epoch FROM runtime_epoch", [], |r| r
            .get::<_, u64>(0))
            .unwrap(),
        1
    );
    assert_eq!(current.begin_execution_epoch().unwrap().1, 2);
    let guards: u64 = current.connection.query_row("SELECT COUNT(*) FROM sqlite_schema WHERE type='trigger' AND name LIKE 'writer_%' AND sql LIKE ?1", [format!("%<>{SCHEMA_VERSION}%")], |r| r.get(0)).unwrap();
    assert_eq!(guards, (MUTABLE_TABLES.len() * 3) as u64);
    // The unrelated append-only guard remains installed across the upgrade.
    assert!(current.connection.query_row("SELECT 1 FROM sqlite_schema WHERE type='trigger' AND name='cleanup_observation_no_update'", [], |r| r.get::<_, i64>(0)).is_ok());
}

#[test]
fn new_generation_closes_old_reviewers_without_erasing_known_work_or_artifacts() {
    let (_fixture, mut store, task, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&task, epoch), task.version)
        .unwrap();
    let unit = store
        .finish_execution(
            &unit.authority(),
            WorkOutcome::Success,
            Disposition::Completed,
        )
        .unwrap();
    // Ledger-only artifact control; this does not attest any on-disk graph.
    let mut artifact = ResultArtifact {
        id: ArtifactId::new(),
        scope: unit.scope.clone(),
        unit_id: unit.id,
        state: ArtifactState::Staging,
        revision: "c".repeat(40),
        base_sha: unit.base_sha.clone(),
        object_format: "sha1".into(),
        repository: PathBuf::from("/tmp/fixture-retained.git"),
        manifest: PathBuf::from("/tmp/fixture-manifest.json"),
        manifest_sha256: String::new(),
        dependencies: std::collections::BTreeMap::new(),
        version: 1,
        created_at: now_ms(),
    };
    store.stage_result(&unit.authority(), &artifact).unwrap();
    artifact.state = ArtifactState::Ready;
    artifact.version = 2;
    artifact.manifest_sha256 = "d".repeat(64);
    store.ready_result(&artifact, 1).unwrap();
    let current_task = store.task(task.id).unwrap().unwrap();
    let mut stale_verified = artifact.clone();
    stale_verified
        .dependencies
        .insert("rules".into(), "changed".into());
    assert!(
        store
            .publish_execution_result(&unit.authority(), &stale_verified, current_task.version)
            .is_err()
    );
    assert_eq!(
        store.result_artifact(artifact.id).unwrap().state,
        ArtifactState::Ready
    );
    let published = store
        .publish_execution_result(&unit.authority(), &artifact, current_task.version)
        .unwrap();
    let current_task = store.task(task.id).unwrap().unwrap();
    let mut old = Vec::new();
    for kind in [UnitKind::Reviewer, UnitKind::Verifier] {
        let mut spec = draft(&current_task, epoch);
        spec.kind = kind;
        spec.artifact_id = Some(artifact.id);
        spec.base_sha = artifact.revision.clone();
        spec.branch = None;
        let review = store.reserve_execution(spec, current_task.version).unwrap();
        assert_eq!(
            store
                .reserve_execution_quota(&review.authority(), "codex", "unknown", 6, 2, 3, now_ms())
                .unwrap(),
            QuotaAdmission::Admitted
        );
        let mut review = store.execution_unit(review.id).unwrap();
        if kind == UnitKind::Verifier {
            review = store
                .finish_execution(
                    &review.authority(),
                    WorkOutcome::Success,
                    Disposition::Completed,
                )
                .unwrap();
        }
        old.push(review);
    }
    let fresh = store
        .reserve_execution(draft(&current_task, epoch), current_task.version)
        .unwrap();
    assert!(fresh.generation > unit.generation);
    for previous in &old {
        let closed = store.execution_unit(previous.id).unwrap();
        assert!(!closed.native_effects_open && !closed.result_finalization_open);
        assert_eq!(
            closed.work,
            Some(if previous.kind == UnitKind::Verifier {
                WorkOutcome::Success
            } else {
                WorkOutcome::Unknown
            })
        );
        assert_eq!(
            closed.disposition,
            if previous.kind == UnitKind::Verifier {
                Disposition::Completed
            } else {
                Disposition::Cancelled
            }
        );
        assert!(
            store
                .validate_execution(&previous.authority(), true, false)
                .is_err()
        );
        assert_eq!(
            store
                .connection
                .query_row(
                    "SELECT COUNT(*) FROM cleanup_jobs WHERE unit_id=?1",
                    [previous.id.to_string()],
                    |r| r.get::<_, u64>(0)
                )
                .unwrap(),
            1
        );
    }
    assert_eq!(
        store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM quota_leases WHERE active=1",
                [],
                |r| r.get::<_, u64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        serde_json::to_value(store.result_artifact(artifact.id).unwrap()).unwrap(),
        serde_json::to_value(published).unwrap()
    );
    // A new readonly unit may still review that retained prior SHA concurrently
    // with the fresh executor; it belongs to the freshly admitted generation.
    let current_task = store.task(task.id).unwrap().unwrap();
    let mut spec = draft(&current_task, epoch);
    spec.kind = UnitKind::Reviewer;
    spec.artifact_id = Some(artifact.id);
    spec.base_sha = artifact.revision;
    spec.branch = None;
    let review = store.reserve_execution(spec, current_task.version).unwrap();
    assert_eq!(review.generation, fresh.generation);
    assert!(
        store
            .validate_execution(&review.authority(), true, false)
            .is_ok()
    );
    assert!(
        store
            .validate_execution(&fresh.authority(), true, false)
            .is_ok()
    );
    assert_eq!(
        store
            .reserve_execution_quota(&review.authority(), "codex", "unknown", 6, 2, 3, now_ms())
            .unwrap(),
        QuotaAdmission::Admitted
    );
    let review = store.execution_unit(review.id).unwrap();
    store.retire_execution(&fresh.authority(), false).unwrap();
    let stopped_review = store.execution_unit(review.id).unwrap();
    assert!(!stopped_review.native_effects_open && !stopped_review.result_finalization_open);
    assert_eq!(stopped_review.work, Some(WorkOutcome::Unknown));
    assert_eq!(
        store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM quota_leases WHERE active=1",
                [],
                |r| r.get::<_, u64>(0)
            )
            .unwrap(),
        0
    );
    let current_task = store.task(task.id).unwrap().unwrap();
    let retry = store
        .reserve_execution(draft(&current_task, epoch), current_task.version)
        .unwrap();
    assert!(retry.generation > fresh.generation);
    assert_ne!(retry.worktree, fresh.worktree);
    assert_eq!(
        store.result_artifact(artifact.id).unwrap().state,
        ArtifactState::Published
    );
}

#[test]
fn a_live_executor_cannot_be_replaced_or_leak_its_capacity_through_failed_reservation() {
    let (_fixture, mut store, task, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&task, epoch), task.version)
        .unwrap();
    assert_eq!(
        store
            .reserve_execution_quota(&unit.authority(), "codex", "unknown", 6, 2, 3, now_ms())
            .unwrap(),
        QuotaAdmission::Admitted
    );
    let current = store.execution_unit(unit.id).unwrap();
    let current_task = store.task(task.id).unwrap().unwrap();
    let next = draft(&current_task, epoch);
    assert!(
        store
            .reserve_execution(next.clone(), current_task.version)
            .is_err()
    );
    assert!(store.execution_unit(next.id).is_err());
    assert_eq!(
        store.execution_unit(current.id).unwrap().authority(),
        current.authority()
    );
    assert_eq!(
        store.task(task.id).unwrap().unwrap().version,
        current_task.version
    );
    assert_eq!(
        store
            .connection
            .query_row(
                "SELECT generation FROM task_execution WHERE task_id=?1",
                [task.id.to_string()],
                |r| r.get::<_, u64>(0)
            )
            .unwrap(),
        current.generation
    );
    assert_eq!(
        store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM quota_leases WHERE active=1",
                [],
                |r| r.get::<_, u64>(0)
            )
            .unwrap(),
        1
    );
    store.retire_execution(&current.authority(), false).unwrap();
    assert!(store.reserve_execution(next, current_task.version).is_ok());
    assert_eq!(
        store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM quota_leases WHERE active=1",
                [],
                |r| r.get::<_, u64>(0)
            )
            .unwrap(),
        0
    );
}

#[test]
fn unclassified_capacity_waits_for_a_fresh_attempt_without_closing_a_sibling_pool() {
    let (_fixture, mut store, task, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&task, epoch), task.version)
        .unwrap();
    let closed = store
        .finish_execution(
            &unit.authority(),
            WorkOutcome::Unknown,
            Disposition::CapacityInterrupted,
        )
        .unwrap();
    let due = closed.capacity_retry_at.unwrap();
    assert_eq!(closed.wait_reason, Some(WaitReason::Capacity));
    assert!(
        store
            .reserve_execution_quota(&closed.authority(), "codex", "unknown", 6, 2, 3, due)
            .is_err()
    );
    let current_task = store.task(task.id).unwrap().unwrap();
    let retry = store
        .reserve_execution(draft(&current_task, epoch), current_task.version)
        .unwrap();
    assert_ne!(closed.id, retry.id);
    assert_ne!(closed.worktree, retry.worktree);
    assert_eq!(
        store
            .reserve_execution_quota(&retry.authority(), "codex", "unknown", 6, 2, 3, due - 1)
            .unwrap(),
        QuotaAdmission::Waiting {
            reason: WaitReason::Capacity,
            next_due: due
        }
    );
    let waiting = store.execution_unit(retry.id).unwrap();
    let mut sibling = Task::new(
        task.project_id,
        task.goal_id,
        "sibling".into(),
        "codex".into(),
    );
    store.put_task(&mut sibling).unwrap();
    let sibling_unit = store
        .reserve_execution(draft(&sibling, epoch), sibling.version)
        .unwrap();
    assert_eq!(
        store
            .reserve_execution_quota(
                &sibling_unit.authority(),
                "codex",
                "unknown",
                6,
                2,
                3,
                due - 1
            )
            .unwrap(),
        QuotaAdmission::Admitted
    );
    assert_eq!(
        store
            .reserve_execution_quota(&waiting.authority(), "codex", "unknown", 6, 2, 3, due)
            .unwrap(),
        QuotaAdmission::Admitted
    );
    assert!(
        store
            .quota_observations("codex", "unknown")
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        store.execution_unit(closed.id).unwrap().capacity_retry_at,
        Some(due)
    );
}

#[test]
fn epoch_recovery_preserves_known_work_and_fences_live_and_already_retired_transports() {
    let (_fixture, mut store, task, epoch) = fixture();
    let mut units = Vec::new();
    let mut sessions = Vec::new();
    for index in 0..3 {
        let task = if index == 0 {
            task.clone()
        } else {
            let mut sibling = Task::new(
                task.project_id,
                task.goal_id,
                format!("sibling-{index}"),
                "codex".into(),
            );
            store.put_task(&mut sibling).unwrap();
            sibling
        };
        let unit = store
            .reserve_execution(draft(&task, epoch), task.version)
            .unwrap();
        let unit = store
            .transition_execution(&unit.authority(), UnitState::Preparing)
            .unwrap();
        assert_eq!(
            store
                .reserve_execution_quota(&unit.authority(), "codex", "unknown", 6, 2, 3, now_ms())
                .unwrap(),
            QuotaAdmission::Admitted
        );
        let unit = store.execution_unit(unit.id).unwrap();
        let mut session = session(&unit);
        let unit = store
            .register_execution_session(&unit.authority(), &session)
            .unwrap();
        session.state = SessionState::Running;
        session.native_ref = Some(format!("fixture-{index}"));
        session.pid = Some(424242 + index);
        let (unit, version) = store
            .update_execution_session(&unit.authority(), &session, 1)
            .unwrap();
        store
            .reserve_managed_effect(
                &unit.authority(),
                &ManagedEffect {
                    id: OperationId::new(),
                    unit_id: unit.id,
                    scope: unit.scope.clone(),
                    kind: "docker".into(),
                    idempotency_key: format!("fixture-{index}"),
                    expected_target: format!("container-{index}"),
                    state: EffectState::Pending,
                    receipt: BTreeMap::new(),
                    version: 1,
                },
            )
            .unwrap();
        let unit = match index {
            0 => {
                let unit = store
                    .finish_execution(
                        &unit.authority(),
                        WorkOutcome::Success,
                        Disposition::Completed,
                    )
                    .unwrap();
                session.state = SessionState::Exited;
                store
                    .close_execution_session(unit.id, &session, version)
                    .unwrap();
                unit
            }
            2 => store.retire_execution(&unit.authority(), false).unwrap(),
            _ => unit,
        };
        units.push(unit);
        sessions.push(session);
    }
    let mut waiter_task = Task::new(
        task.project_id,
        task.goal_id,
        "waiting-capacity".into(),
        "codex".into(),
    );
    store.put_task(&mut waiter_task).unwrap();
    let waiter = store
        .reserve_execution(draft(&waiter_task, epoch), waiter_task.version)
        .unwrap();
    assert!(matches!(
        store
            .reserve_execution_quota(&waiter.authority(), "codex", "unknown", 1, 2, 3, now_ms())
            .unwrap(),
        QuotaAdmission::Waiting {
            reason: WaitReason::Capacity,
            ..
        }
    ));
    assert_eq!(
        store
            .connection
            .query_row("SELECT COUNT(*) FROM quota_waiters", [], |row| row
                .get::<_, u64>(0))
            .unwrap(),
        1
    );
    let (_, next_epoch) = store.begin_execution_epoch().unwrap();
    assert_eq!(next_epoch, epoch + 1);
    for (index, prior) in units.iter().enumerate() {
        let recovered = store.execution_unit(prior.id).unwrap();
        assert!(!recovered.native_effects_open && !recovered.result_finalization_open);
        assert_eq!(
            recovered.work,
            Some(if index == 0 {
                WorkOutcome::Success
            } else {
                WorkOutcome::Unknown
            })
        );
        assert_eq!(
            recovered.disposition,
            [
                Disposition::Completed,
                Disposition::Lost,
                Disposition::Cancelled
            ][index]
        );
        assert_eq!(
            recovered.state,
            if index == 0 {
                UnitState::WorkKnown
            } else if index == 1 {
                UnitState::WorkUnknown
            } else {
                UnitState::Retired
            }
        );
        let record = store
            .record(RecordId(sessions[index].id.0))
            .unwrap()
            .unwrap();
        let session: Session = serde_json::from_value(record.data).unwrap();
        assert_eq!(
            session.state,
            if index == 0 {
                SessionState::Exited
            } else {
                SessionState::Lost
            }
        );
        assert_eq!(
            session.pid, sessions[index].pid,
            "historical PID is diagnostic, never a recovery signal authority"
        );
        let effect = store.managed_effects(prior.id).unwrap().remove(0);
        assert_eq!(effect.state, EffectState::Unknown);
        assert_eq!(effect.receipt["transport"], "runtime_epoch_lost");
        assert!(
            store
                .validate_execution(&prior.authority(), true, false)
                .is_err()
        );
        let task = store.task(prior.scope.task_id.unwrap()).unwrap().unwrap();
        let retry = store
            .reserve_execution(draft(&task, next_epoch), task.version)
            .unwrap();
        assert_ne!(retry.worktree, prior.worktree);
        assert!(retry.generation > prior.generation);
    }
    assert_eq!(
        store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM quota_leases WHERE active=1",
                [],
                |r| r.get::<_, u64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        store
            .connection
            .query_row("SELECT COUNT(*) FROM quota_waiters", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn epoch_recovery_rolls_back_if_a_body_redirects_an_indexed_unit() {
    let (_fixture, mut store, task, epoch) = fixture();
    let first = store
        .reserve_execution(draft(&task, epoch), task.version)
        .unwrap();
    let mut sibling = Task::new(
        task.project_id,
        task.goal_id,
        "sibling".into(),
        "codex".into(),
    );
    store.put_task(&mut sibling).unwrap();
    let second = store
        .reserve_execution(draft(&sibling, epoch), sibling.version)
        .unwrap();
    store
        .connection
        .execute(
            "UPDATE execution_units SET body=?1 WHERE id=?2",
            params![
                serde_json::to_string(&second).unwrap(),
                first.id.to_string()
            ],
        )
        .unwrap();
    assert!(store.begin_execution_epoch().is_err());
    assert_eq!(store.connection_epoch_for_test(), epoch);
    let second_after = store.execution_unit(second.id).unwrap();
    assert_eq!(second_after.version, second.version);
    assert!(second_after.native_effects_open && second_after.result_finalization_open);
    assert_eq!(
        store
            .connection
            .query_row("SELECT COUNT(*) FROM cleanup_jobs", [], |row| row
                .get::<_, u64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn normal_terminal_cleanup_keeps_finalization_and_result_publication_races_cancel() {
    let (_fixture, mut store, t, epoch) = fixture();
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
            actions: Vec::new(),
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
fn cleanup_claim_waits_for_finalization_and_preserves_known_work_and_sibling() {
    let (_fixture, mut store, task, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&task, epoch), task.version)
        .unwrap();
    let known = store
        .finish_execution(
            &unit.authority(),
            WorkOutcome::Success,
            Disposition::Completed,
        )
        .unwrap();
    let at = now_ms().saturating_add(1000);
    assert!(store.due_execution_cleanup(at, 4).unwrap().is_empty());
    assert!(
        store
            .claim_execution_cleanup(unit.id, epoch, at)
            .unwrap()
            .is_none()
    );
    store.retire_execution(&known.authority(), false).unwrap();
    let mut sibling = Task::new(
        task.project_id,
        task.goal_id,
        "cleanup sibling".into(),
        "codex".into(),
    );
    store.put_task(&mut sibling).unwrap();
    let second = store
        .reserve_execution(draft(&sibling, epoch), sibling.version)
        .unwrap();
    let lease = ResourceLease {
        id: LeaseId::new(),
        unit_id: second.id,
        scope: second.scope.clone(),
        kind: ResourceKind::Ports,
        namespace: "host".into(),
        value: "31000-31031".into(),
        port_start: Some(31000),
        port_end: Some(31031),
        state: LeaseState::Reserved,
        version: 1,
    };
    store
        .reserve_execution_leases(&second.authority(), std::slice::from_ref(&lease))
        .unwrap();
    let before_task = store.task(task.id).unwrap().unwrap();
    let before = store.execution_unit(unit.id).unwrap();
    assert_eq!(store.due_execution_cleanup(at, 4).unwrap(), vec![unit.id]);
    let claim = store
        .claim_execution_cleanup(unit.id, epoch, at)
        .unwrap()
        .unwrap();
    assert!(
        store
            .claim_execution_cleanup(unit.id, epoch, at)
            .unwrap()
            .is_none()
    );
    let observation = CleanupObservation {
        actions: Vec::new(),
        unit_id: unit.id,
        at,
        outcome: CleanupOutcome::Leftovers,
        coverage: BTreeMap::from([("cookie".into(), "bounded observation".into())]),
        remaining: vec!["fixture-last-seen".into()],
        errors: vec![],
    };
    store
        .finish_execution_cleanup(&claim, &observation)
        .unwrap();
    let after = store.execution_unit(unit.id).unwrap();
    assert_eq!(after.version, before.version);
    assert_eq!(after.work, Some(WorkOutcome::Success));
    assert_eq!(after.disposition, before.disposition);
    assert_eq!(after.cleanup, CleanupOutcome::Leftovers);
    assert_eq!(after.worktree, before.worktree);
    assert_eq!(
        serde_json::to_value(store.task(task.id).unwrap().unwrap()).unwrap(),
        serde_json::to_value(before_task).unwrap()
    );
    assert_eq!(
        store.execution_unit(second.id).unwrap().authority(),
        second.authority()
    );
    assert!(
        store
            .validate_execution(&second.authority(), true, false)
            .is_ok()
    );
    assert_eq!(
        store.execution_leases(second.id).unwrap()[0].state,
        lease.state
    );
    assert_eq!(
        store.execution_leases(second.id).unwrap()[0].version,
        lease.version
    );
    assert!(
        store
            .finish_execution_cleanup(&claim, &observation)
            .is_err()
    );
}

#[test]
fn cleanup_backlog_fences_expired_claims_and_old_runtime_epochs() {
    let (_fixture, mut store, task, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&task, epoch), task.version)
        .unwrap();
    let lease = ResourceLease {
        id: LeaseId::new(),
        unit_id: unit.id,
        scope: unit.scope.clone(),
        kind: ResourceKind::Worktree,
        namespace: task.project_id.to_string(),
        value: unit.worktree.to_string_lossy().into_owned(),
        port_start: None,
        port_end: None,
        state: LeaseState::Reserved,
        version: 1,
    };
    store
        .reserve_execution_leases(&unit.authority(), std::slice::from_ref(&lease))
        .unwrap();
    store.retire_execution(&unit.authority(), false).unwrap();
    let at = now_ms().saturating_add(1000);
    let first = store
        .claim_execution_cleanup(unit.id, epoch, at)
        .unwrap()
        .unwrap();
    let second = store
        .claim_execution_cleanup(unit.id, epoch, at + 60_000)
        .unwrap()
        .unwrap();
    let observation = CleanupObservation {
        actions: Vec::new(),
        unit_id: unit.id,
        at: at + 60_000,
        outcome: CleanupOutcome::Unknown,
        coverage: BTreeMap::new(),
        remaining: vec![],
        errors: vec!["unavailable".into()],
    };
    assert!(
        store
            .finish_execution_cleanup(&first, &observation)
            .is_err()
    );
    assert_eq!(
        store.execution_leases(unit.id).unwrap()[0].state,
        LeaseState::Reserved
    );
    store
        .finish_execution_cleanup(&second, &observation)
        .unwrap();
    assert_eq!(
        store.execution_leases(unit.id).unwrap()[0].state,
        LeaseState::Quarantined
    );
    let third = store
        .claim_execution_cleanup(unit.id, epoch, at + 120_000)
        .unwrap()
        .unwrap();
    let (_, successor) = store.begin_execution_epoch().unwrap();
    assert!(successor > epoch);
    assert!(
        store
            .finish_execution_cleanup(&third, &observation)
            .is_err()
    );
    assert!(
        store
            .claim_execution_cleanup(unit.id, epoch, at + 180_000)
            .is_err()
    );
    assert!(
        store
            .claim_execution_cleanup(unit.id, successor, at + 180_000)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM cleanup_observations WHERE unit_id=?1",
                [unit.id.to_string()],
                |row| row.get::<_, u64>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(
        store.execution_unit(unit.id).unwrap().work,
        Some(WorkOutcome::Unknown)
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
    assert_eq!(upgraded.schema_version().unwrap(), SCHEMA_VERSION);
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
    let (_fixture, mut store, t, epoch) = fixture();
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
    let (_fixture, mut store, t, epoch) = fixture();
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
    let (_fixture, mut store, t, epoch) = fixture();
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
    let (_fixture, mut store, t, epoch) = fixture();
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
    let (_fixture, mut store, t, epoch) = fixture();
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
    let (_fixture, mut store, t, epoch) = fixture();
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
    let (_fixture, mut store, t, epoch) = fixture();
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
    let (_fixture, mut store, t, epoch) = fixture();
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
    let (_fixture, mut store, t, epoch) = fixture();
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

#[test]
fn multi_window_recovery_preserves_one_probe_until_its_actual_release() {
    let (_fixture, mut store, task, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&task, epoch), task.version)
        .unwrap();
    let at = now_ms();
    let primary = QuotaObservation {
        provider: "codex".into(),
        account_key: "unknown".into(),
        bucket: "primary".into(),
        window_id: "unknown".into(),
        status: QuotaStatus::Exhausted,
        used_percent: Some(100.0),
        resets_at: None,
        observed_at: at,
        source_version: "fixture".into(),
        confirmed_subscription: true,
    };
    let secondary = QuotaObservation {
        bucket: "secondary".into(),
        ..primary.clone()
    };
    store.observe_quota(&primary).unwrap();
    store.observe_quota(&secondary).unwrap();
    assert_eq!(
        store
            .reserve_execution_quota(&unit.authority(), "codex", "unknown", 6, 2, 3, at + 60001)
            .unwrap(),
        quotas::QuotaAdmission::Admitted
    );
    let probe = store.execution_unit(unit.id).unwrap();
    let recovered_primary = QuotaObservation {
        status: QuotaStatus::Available,
        used_percent: Some(20.0),
        observed_at: at + 60001,
        ..primary
    };
    store
        .observe_quota_from_probe(&recovered_primary, &probe.authority())
        .unwrap();
    assert!(
        store
            .execution_is_quota_probe(probe.id, "codex", "unknown")
            .unwrap()
    );
    let mut sibling = Task::new(
        task.project_id,
        task.goal_id,
        "sibling".into(),
        "codex".into(),
    );
    store.put_task(&mut sibling).unwrap();
    let sibling = store
        .reserve_execution(draft(&sibling, epoch), sibling.version)
        .unwrap();
    assert!(matches!(
        store
            .reserve_execution_quota(
                &sibling.authority(),
                "codex",
                "unknown",
                6,
                2,
                3,
                at + 600001
            )
            .unwrap(),
        quotas::QuotaAdmission::Waiting {
            reason: WaitReason::Quota,
            ..
        }
    ));
    assert!(
        store
            .execution_is_quota_probe(probe.id, "codex", "unknown")
            .unwrap()
    );
    let recovered_secondary = QuotaObservation {
        status: QuotaStatus::Available,
        used_percent: Some(10.0),
        observed_at: at + 60001,
        ..secondary
    };
    store
        .observe_quota_from_probe(&recovered_secondary, &probe.authority())
        .unwrap();
    assert!(
        store
            .quota_observations("codex", "unknown")
            .unwrap()
            .iter()
            .all(|o| o.status == QuotaStatus::Available)
    );
    assert!(
        store
            .execution_is_quota_probe(probe.id, "codex", "unknown")
            .unwrap()
    );
    store.release_execution_quota(probe.id).unwrap();
    assert!(
        !store
            .execution_is_quota_probe(probe.id, "codex", "unknown")
            .unwrap()
    );
    assert!(
        store
            .observe_quota_from_probe(&recovered_secondary, &probe.authority())
            .is_err()
    );
}
