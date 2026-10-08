use super::*;
use crate::config::WorkflowClass;
use rusqlite::functions::FunctionFlags;
use std::path::PathBuf;

fn legacy5(path: &Path) -> Connection {
    let old = Connection::open(path).unwrap();
    old.create_scalar_function(
        "rrx_writer_contract_version",
        0,
        FunctionFlags::SQLITE_UTF8
            | FunctionFlags::SQLITE_DETERMINISTIC
            | FunctionFlags::SQLITE_INNOCUOUS,
        |_| Ok(5_i64),
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
            old.execute_batch(&format!("CREATE TRIGGER writer_{table}_{action} BEFORE {action} ON {table} WHEN rrx_writer_contract_version()<>5 BEGIN SELECT RAISE(ABORT,'incompatible rrx writer contract'); END;")).unwrap();
        }
    }
    old.pragma_update(None, "application_id", APPLICATION_ID)
        .unwrap();
    old.pragma_update(None, "user_version", 5).unwrap();
    old
}

#[test]
fn schema6_migrates_real5_layout_and_fences_preopen_cached_writer() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let old = legacy5(&path);
    let mut cached = old
        .prepare("UPDATE runtime_epoch SET epoch=epoch+1 WHERE singleton=1")
        .unwrap();
    cached.execute([]).unwrap();
    let mut current = Store::open(&path).unwrap();
    assert_eq!(current.schema_version().unwrap(), SCHEMA_VERSION);
    assert!(cached.execute([]).is_err());
    for sql in [
        "UPDATE runtime_epoch SET epoch=epoch+1 WHERE singleton=1",
        "DELETE FROM runtime_epoch WHERE singleton=1",
        "INSERT INTO runtime_epoch(singleton,instance_id,epoch) VALUES(1,'forged',100) ON CONFLICT(singleton) DO UPDATE SET epoch=100",
        "INSERT INTO native_results(id,invocation_id,unit_id,session_id,project_id,goal_id,task_id,generation,owner_epoch,provider,acquisition,authority,version,body) VALUES('r','i','u','s','p','g','t',1,1,'codex','complete','owned_terminal',1,'{}')",
    ] {
        let error = old.execute(sql, []).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("incompatible rrx writer contract"),
            "{error}"
        );
    }
    assert_eq!(
        current
            .connection
            .query_row("SELECT epoch FROM runtime_epoch", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        1
    );
    assert_eq!(current.begin_execution_epoch().unwrap().1, 2);
    for table in MUTABLE_TABLES {
        for action in ["INSERT", "UPDATE", "DELETE"] {
            let sql: String = current
                .connection
                .query_row(
                    "SELECT sql FROM sqlite_schema WHERE type='trigger' AND name=?1",
                    [format!("writer_{table}_{action}")],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(sql.contains(&format!("<>{SCHEMA_VERSION}")));
        }
    }
    assert_eq!(
        current
            .connection
            .query_row("SELECT COUNT(*) FROM native_results", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        0
    );
    drop(cached);
    drop(old);
    drop(current);
    assert_eq!(
        Store::open(&path).unwrap().schema_version().unwrap(),
        SCHEMA_VERSION
    );
}

#[test]
fn schema6_failed_table_install_rolls_back_guards_and_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let old = legacy5(&path);
    // A conflicting reserved object is corruption, not an ignorable partial migration.
    old.execute_batch("CREATE VIEW native_invocations AS SELECT 'unsupported' AS body")
        .unwrap();
    assert!(Store::open(&path).is_err());
    assert_eq!(
        old.pragma_query_value::<i64, _>(None, "user_version", |r| r.get(0))
            .unwrap(),
        5
    );
    assert_eq!(
        old.query_row(
            "SELECT COUNT(*) FROM sqlite_schema WHERE type='table' AND name='native_results'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        old.execute(
            "UPDATE runtime_epoch SET epoch=epoch+1 WHERE singleton=1",
            []
        )
        .unwrap(),
        1
    );
}

// Raw test-only ledger rows exercise typed readers; they mint no producer grant.
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
        reviewers: &[],
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

fn populated_receipt(
    mut modify: impl FnMut(&mut crate::execution::native_result::NativeResultReceipt),
) -> (Store, NativeResultId) {
    use crate::execution::native_result::*;
    let (_fixture, mut store, task, epoch) = fixture();
    let unit = store
        .reserve_execution(draft(&task, epoch), task.version)
        .unwrap();
    let unit = store
        .transition_execution(&unit.authority(), UnitState::Preparing)
        .unwrap();
    let session = session(&unit);
    let unit = store
        .register_execution_session(&unit.authority(), &session)
        .unwrap();
    let invocation = NativeInvocation {
        id: NativeInvocationId::new(),
        unit_id: unit.id,
        session_id: session.id,
        scope: unit.scope.clone(),
        generation: unit.generation,
        owner_epoch: unit.owner_epoch,
        provider: unit.provider.clone(),
        profile: "text_fixture".into(),
        native_version: "fixture".into(),
        unit_version: unit.version,
        session_version: 1,
        context_version: None,
        context_sha256: None,
        source_versions: BTreeMap::new(),
        source_sha256: digest(b"{}"),
        revision: unit.base_sha.clone(),
        artifact_id: None,
        artifact_version: None,
        payload_sha256: digest(b"input"),
        input_operation: None,
        frame_sha256: None,
        native_thread: None,
        native_turn: None,
        state: InvocationState::Closed,
        version: 1,
    };
    invocation.validate().unwrap();
    let (p, g, t) = scope_keys(&unit.scope).unwrap();
    store.connection.execute("INSERT INTO native_invocations(id,unit_id,session_id,project_id,goal_id,task_id,generation,owner_epoch,provider,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,'closed',1,?10)", params![invocation.id.to_string(),unit.id.to_string(),session.id.to_string(),p,g,t,unit.generation,unit.owner_epoch,unit.provider,serde_json::to_string(&invocation).unwrap()]).unwrap();
    let mut receipt = NativeResultReceipt {
        id: NativeResultId::new(),
        invocation_id: invocation.id,
        unit_id: unit.id,
        session_id: session.id,
        scope: unit.scope.clone(),
        generation: unit.generation,
        owner_epoch: unit.owner_epoch,
        provider: unit.provider.clone(),
        native_thread: None,
        native_turn: None,
        acquisition: AcquisitionStatus::Complete,
        authority: ReceiptAuthority::OwnedTerminal,
        text: Some("answer".into()),
        structured_output: None,
        prefix: None,
        answer_sha256: Some(digest(b"answer")),
        terminal_sha256: Some(digest(b"terminal")),
        wire: None,
        observed_work: WorkOutcome::Success,
        disposition: Disposition::Completed,
        diagnostics: vec![],
        captured_at: now_ms(),
        version: 1,
    };
    modify(&mut receipt);
    let body = serde_json::to_string(&receipt).unwrap();
    assert!(
        body.len() <= RECEIPT_BYTES,
        "negative controls reach typed validation below SQL ceiling"
    );
    store.connection.execute("INSERT INTO native_results(id,invocation_id,unit_id,session_id,project_id,goal_id,task_id,generation,owner_epoch,provider,acquisition,authority,version,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,'owned_terminal',1,?12)",params![receipt.id.to_string(),invocation.id.to_string(),unit.id.to_string(),session.id.to_string(),p,g,t,unit.generation,unit.owner_epoch,unit.provider,key(receipt.acquisition),body]).unwrap();
    (store, receipt.id)
}

#[test]
fn populated_native_reader_accepts_text_and_bounded_partial_prefix() {
    use crate::execution::native_result::*;
    let (store, id) = populated_receipt(|_| {});
    assert_eq!(
        store.native_result(id).unwrap().text.as_deref(),
        Some("answer")
    );
    let (store, id) = populated_receipt(|r| {
        r.acquisition = AcquisitionStatus::Partial;
        r.text = None;
        r.answer_sha256 = None;
        r.prefix = Some(PrefixEvidence {
            prefix: "前".into(),
            prefix_sha256: digest("前".as_bytes()),
            observed_bytes: 7,
            exact_length: true,
            unseen_suffix: true,
        });
    });
    let result = store.native_result(id).unwrap();
    assert_eq!(result.prefix.unwrap().prefix, "前");
}

#[test]
fn populated_native_reader_refuses_unqualified_structured_and_inconsistent_prefix() {
    use crate::execution::native_result::*;
    for structured in [
        json!({"answer":"structured"}),
        json!("x".repeat(ANSWER_BYTES + 1)),
        (0..33).fold(json!("deep"), |v, _| json!([v])),
    ] {
        let (store, id) = populated_receipt(|r| {
            r.acquisition = AcquisitionStatus::Partial;
            r.text = None;
            r.answer_sha256 = None;
            r.structured_output = Some(structured.clone());
        });
        assert!(
            store
                .native_result(id)
                .unwrap_err()
                .to_string()
                .contains("structured native content profile is unsupported")
        );
    }
    for (status, observed, exact, unseen) in [
        (AcquisitionStatus::Missing, 3, true, false),
        (AcquisitionStatus::Partial, 3, false, false),
        (AcquisitionStatus::Partial, 4, true, false),
        (AcquisitionStatus::Partial, 3, true, true),
    ] {
        let (store, id) = populated_receipt(|r| {
            r.acquisition = status;
            r.text = None;
            r.answer_sha256 = None;
            r.prefix = Some(PrefixEvidence {
                prefix: "前".into(),
                prefix_sha256: digest("前".as_bytes()),
                observed_bytes: observed,
                exact_length: exact,
                unseen_suffix: unseen,
            });
        });
        assert!(
            store
                .native_result(id)
                .unwrap_err()
                .to_string()
                .contains("invalid native receipt prefix")
        );
    }
}
