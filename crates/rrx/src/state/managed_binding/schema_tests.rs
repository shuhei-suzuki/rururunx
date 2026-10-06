//! Schema/permission mechanics only. No fixture constructs a PhaseSessionOwner,
//! PreparedPhaseInput, native admission or Driver capability.
use super::{permits::*, schema::*};
use crate::{domain::*, state::*};
use rusqlite::{functions::FunctionFlags, types::Value as SqlValue};
use std::{path::Path, sync::Arc};

fn old9(path: &Path) -> Connection {
    let c = Connection::open(path).unwrap();
    c.create_scalar_function(
        "rrx_writer_contract_version",
        0,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_INNOCUOUS,
        |_| Ok(9_i64),
    )
    .unwrap();
    for sql in [
        include_str!("../schema.sql"),
        include_str!("../execution.sql"),
        include_str!("../execution/native_results.sql"),
        include_str!("../execution/source_recovery.sql"),
        include_str!("../execution/verification.sql"),
        include_str!("../runtime/schema.sql"),
    ] {
        c.execute_batch(sql).unwrap();
    }
    c.execute(
        "INSERT INTO runtime_epoch(singleton,instance_id,epoch) VALUES(1,?1,0)",
        [uuid::Uuid::new_v4().to_string()],
    )
    .unwrap();
    for table in execution::MUTABLE_TABLES
        .iter()
        .filter(|t| !TABLES.contains(t))
    {
        for action in ["INSERT", "UPDATE", "DELETE"] {
            c.execute_batch(&format!("CREATE TRIGGER writer_{table}_{action} BEFORE {action} ON {table} WHEN rrx_writer_contract_version()<>9 BEGIN SELECT RAISE(ABORT,'incompatible rrx writer contract'); END;")).unwrap();
        }
    }
    c.pragma_update(None, "application_id", APPLICATION_ID)
        .unwrap();
    c.pragma_update(None, "user_version", 9).unwrap();
    c
}
// Exact old9 factual history, not current accepted Goal/driver authority.
fn history(c: &Connection) -> Scope {
    let mut p = Project::new(
        "historical".into(),
        "/tmp/historical".into(),
        "no-execution".into(),
        "main".into(),
    );
    p.version = 1;
    let mut g = Goal::new(p.id, "historical".into(), vec![]);
    g.version = 1;
    let mut t = Task::new(p.id, g.id, "historical".into(), "worker".into());
    t.version = 1;
    c.execute(
        "INSERT INTO projects(id,root,version,body) VALUES(?1,?2,1,?3)",
        params![
            p.id.to_string(),
            p.root.to_str(),
            serde_json::to_string(&p).unwrap()
        ],
    )
    .unwrap();
    c.execute(
        "INSERT INTO goals(id,project_id,version,body) VALUES(?1,?2,1,?3)",
        params![
            g.id.to_string(),
            p.id.to_string(),
            serde_json::to_string(&g).unwrap()
        ],
    )
    .unwrap();
    c.execute(
        "INSERT INTO tasks(id,project_id,goal_id,version,body) VALUES(?1,?2,?3,1,?4)",
        params![
            t.id.to_string(),
            p.id.to_string(),
            g.id.to_string(),
            serde_json::to_string(&t).unwrap()
        ],
    )
    .unwrap();
    t.scope()
}
fn session(scope: &Scope, state: SessionState) -> Record {
    let id = SessionId::new();
    let s = Session {
        id,
        scope: scope.clone(),
        agent: "historical-agent".into(),
        provider: "claude".into(),
        role: SessionRole::Executor,
        native_ref: Some("same-native".into()),
        pid: None,
        worktree: "/tmp/historical/work".into(),
        state,
        model: None,
        effort: None,
        recovery: serde_json::json!({}),
        started_at: 1,
    };
    let mut r = Record::new(
        scope.clone(),
        RecordKind::Session,
        serde_json::to_value(s).unwrap(),
    );
    r.id = RecordId(id.0);
    r.version = 1;
    r
}
fn insert_record(c: &Connection, r: &Record, raw: Option<&str>) -> String {
    let body = raw
        .map(str::to_owned)
        .unwrap_or_else(|| serde_json::to_string(r).unwrap());
    c.execute("INSERT INTO records(id,kind,project_id,goal_id,task_id,version,body) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![r.id.to_string(),r.kind.key(),r.scope.project_id.to_string(),r.scope.goal_id.map(|v|v.to_string()),r.scope.task_id.map(|v|v.to_string()),r.version,body]).unwrap();
    body
}
#[test]
fn actual9_to10_fences_preopened_cached_all_table_writers_and_reopens() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let old = old9(&path);
    let mut cached = old
        .prepare("UPDATE runtime_epoch SET epoch=epoch+1 WHERE singleton=1")
        .unwrap();
    cached.execute([]).unwrap();
    let s = Store::open(&path).unwrap();
    assert_eq!(s.schema_version().unwrap(), 10);
    assert!(cached.execute([]).is_err());
    for table in execution::MUTABLE_TABLES {
        for action in ["INSERT", "UPDATE", "DELETE"] {
            let sql: String = s
                .connection
                .query_row(
                    "SELECT sql FROM sqlite_schema WHERE name=?1",
                    [format!("writer_{table}_{action}")],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(sql.contains("<>10"), "{table}/{action}");
        }
    }
    assert!(
        old.execute("UPDATE scheduler_clock SET sequence=sequence+1", [])
            .is_err()
    );
    // An actual old9 initializer refuses current10; cached old9 SQL lacks its permit functions too.
    let version: i64 = old
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert!(version > 9);
    drop(cached);
    drop(s);
    drop(old);
    assert_eq!(Store::open(&path).unwrap().schema_version().unwrap(), 10);
}
#[test]
fn namespace_and_wrong_actual9_layout_refuse_atomically_without_file_changes() {
    for sql in [
        "CREATE TABLE managed_phase_inputs(unrelated TEXT)",
        "CREATE INDEX binding_collision ON records(id)",
        "ALTER TABLE tasks ADD COLUMN wrong TEXT",
        "DROP TRIGGER audit_no_replace",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let old = old9(&path);
        old.execute_batch(sql).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(Store::open(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(
            old.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            9
        );
    }
}
#[test]
fn session_projection_retains_duplicate_lost_and_malformed_original_history() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let old = old9(&path);
    let scope = history(&old);
    let a = session(&scope, SessionState::Exited);
    let b = session(&scope, SessionState::Lost);
    insert_record(&old, &a, None);
    insert_record(&old, &b, None);
    let bad = session(&scope, SessionState::Lost);
    let raw = serde_json::to_string(&bad).unwrap();
    let raw = raw.replacen("{", "{\"unknown\":0,", 1);
    insert_record(&old, &bad, Some(&raw));
    let s = Store::open(&path).unwrap();
    assert_eq!(s.connection.query_row("SELECT count(*) FROM scoped_session_identities WHERE provider='claude' AND native_ref='same-native'",[],|r|r.get::<_,u64>(0)).unwrap(),2);
    assert_eq!(
        s.connection
            .query_row(
                "SELECT malformed FROM scoped_session_identities WHERE session_id=?1",
                [bad.id.to_string()],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert!(
        s.connection
            .execute(
                "DELETE FROM scoped_session_identities WHERE session_id=?1",
                [bad.id.to_string()]
            )
            .is_err()
    );
    assert!(s.connection.execute("UPDATE scoped_session_identities SET malformed=0,provider='claude',native_ref='hidden' WHERE session_id=?1",[bad.id.to_string()]).is_err());
    assert!(s.connection.execute("INSERT OR REPLACE INTO scoped_session_identities SELECT * FROM scoped_session_identities WHERE session_id=?1",[bad.id.to_string()]).is_err());
    assert!(
        s.connection
            .execute("DELETE FROM records WHERE id=?1", [b.id.to_string()])
            .is_err()
    );
    let mut a2 = a.clone();
    a2.version = 2;
    a2.data["native_ref"] = serde_json::json!("new-native");
    s.connection
        .execute(
            "UPDATE records SET version=2,body=?1 WHERE id=?2",
            params![serde_json::to_string(&a2).unwrap(), a.id.to_string()],
        )
        .unwrap();
    let actual: (i64, String) = s
        .connection
        .query_row(
            "SELECT record_version,native_ref FROM scoped_session_identities WHERE session_id=?1",
            [a.id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(actual, (2, "new-native".into()));
    // Original duplicate member cannot be erased by Value decoding in the projection.
    let mut a3 = a2.clone();
    a3.version = 3;
    let raw = serde_json::to_string(&a3)
        .unwrap()
        .replacen("{", "{\"version\":999,", 1);
    s.connection
        .execute(
            "UPDATE records SET version=3,body=?1 WHERE id=?2",
            params![raw, a.id.to_string()],
        )
        .unwrap();
    assert_eq!(
        s.connection
            .query_row(
                "SELECT malformed FROM scoped_session_identities WHERE session_id=?1",
                [a.id.to_string()],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
}
#[test]
fn migration_inventory_counts_complete_lengths_even_when_not_copied_and_rolls_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let mut c = old9(&path);
    let scope = history(&c);
    for _ in 0..129 {
        insert_record(&c, &session(&scope, SessionState::Exited), None);
    }
    // Current functions are registered for installation; old9 table guards are replaced within the atomic upgrade.
    let manager = Arc::new(PrivatePermitManager::default());
    register_permit_function(&c, manager.clone()).unwrap();
    let tx = c
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    install_schema(&tx).unwrap();
    let stats = migrate_inventory(&tx, &manager, 130, 1024 * 1024).unwrap();
    assert_eq!(stats.rows, 129);
    assert_eq!(stats.pages, 2);
    assert!(stats.source_bytes > 129);
    tx.rollback().unwrap();
    let huge = session(&scope, SessionState::Lost);
    let raw = format!("{{\"large\":\"{}\"}}", "x".repeat(SESSION_BYTES + 1));
    insert_record(&c, &huge, Some(&raw));
    let tx = c
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    install_schema(&tx).unwrap();
    assert!(migrate_inventory(&tx, &manager, 131, SESSION_BYTES).is_err());
    tx.rollback().unwrap();
    assert_eq!(
        c.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        9
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name='scoped_session_identities'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
}
fn contract_plan(scope: &Scope, wf: &Record, body: String) -> ExactRowMutation {
    ExactRowMutation::new(
        "workflow_native_contracts",
        "INSERT",
        None,
        Some(vec![
            SqlValue::Text(wf.id.to_string()),
            SqlValue::Text(scope.project_id.to_string()),
            SqlValue::Text(scope.goal_id.unwrap().to_string()),
            SqlValue::Text(scope.task_id.unwrap().to_string()),
            SqlValue::Integer(0),
            SqlValue::Text("schema-mechanics".into()),
            SqlValue::Null,
            SqlValue::Text("legacy_held".into()),
            SqlValue::Integer(1),
            SqlValue::Text(body),
        ]),
    )
    .unwrap()
}
#[test]
fn exact_permit_all_columns_is_one_use_and_revoked_on_error_and_unwind() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let old = old9(&path);
    let scope = history(&old);
    let s = Store::open(&path).unwrap();
    let mut wf = Record::new(
        scope.clone(),
        RecordKind::Workflow,
        serde_json::json!({"schema_fixture_only":true}),
    );
    wf.version = 1;
    insert_record(&s.connection, &wf, None);
    let body = serde_json::json!({"schema_fixture_only":true}).to_string();
    let insert = |body: &str| -> Result<()> {
        s.connection.execute("INSERT INTO workflow_native_contracts(workflow_id,project_id,goal_id,task_id,owner_epoch,origin,profile_digest,contract_state,version,body) VALUES(?1,?2,?3,?4,0,'schema-mechanics',NULL,'legacy_held',1,?5)",params![wf.id.to_string(),scope.project_id.to_string(),scope.goal_id.unwrap().to_string(),scope.task_id.unwrap().to_string(),body])?;
        Ok(())
    };
    assert!(insert(&body).is_err());
    assert!(
        s.binding_permits
            .with_exact_permit(vec![contract_plan(&scope, &wf, body.clone())], || insert(
                "{\"wrong\":true}"
            ))
            .is_err()
    );
    assert!(insert(&body).is_err());
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: Result<()> = s
            .binding_permits
            .with_exact_permit(vec![contract_plan(&scope, &wf, body.clone())], || {
                panic!("owned fixture unwind")
            });
    }));
    assert!(panic.is_err());
    assert!(insert(&body).is_err());
    s.binding_permits
        .with_exact_permit(vec![contract_plan(&scope, &wf, body.clone())], || {
            insert(&body)?;
            s.binding_permits.ensure_consumed()
        })
        .unwrap();
    assert!(
        s.connection
            .execute(
                "UPDATE records SET version=version+1 WHERE id=?1",
                [wf.id.to_string()]
            )
            .is_err()
    );
    s.connection
        .pragma_update(None, "recursive_triggers", false)
        .unwrap();
    assert!(
        s.connection
            .execute(
                "INSERT OR REPLACE INTO records SELECT * FROM records WHERE id=?1",
                [wf.id.to_string()]
            )
            .is_err()
    );
    assert!(s.connection.execute("INSERT OR REPLACE INTO workflow_native_contracts SELECT * FROM workflow_native_contracts WHERE workflow_id=?1",[wf.id.to_string()]).is_err());
    assert!(
        s.connection
            .execute(
                "DELETE FROM workflow_native_contracts WHERE workflow_id=?1",
                [wf.id.to_string()]
            )
            .is_err()
    );
    // The permission is not a wildcard/nested or reusable mode.
    let manager = PrivatePermitManager::default();
    let plan = || {
        ExactRowMutation::new(
            "records",
            "INSERT",
            None,
            Some(vec![
                SqlValue::Text("id".into()),
                SqlValue::Text("session".into()),
                SqlValue::Text("p".into()),
                SqlValue::Null,
                SqlValue::Null,
                SqlValue::Integer(1),
                SqlValue::Text("{}".into()),
            ]),
        )
        .unwrap()
    };
    let c = Connection::open_in_memory().unwrap();
    let manager = Arc::new(manager);
    register_permit_function(&c, manager.clone()).unwrap();
    let call = || {
        c.query_row("SELECT rrx_binding_permit('records','INSERT',NULL,NULL,NULL,NULL,NULL,NULL,NULL,'id','session','p',NULL,NULL,1,'{}')",[],|r|r.get::<_,bool>(0)).unwrap()
    };
    assert!(!call());
    manager
        .with_exact_permit(vec![plan()], || {
            assert!(manager.with_exact_permit(vec![plan()], || Ok(())).is_err());
            assert!(call());
            assert!(!call());
            manager.ensure_consumed()
        })
        .unwrap();
    assert!(!call());
    assert!(ExactRowMutation::new("records", "INSERT", None, Some(vec![SqlValue::Null])).is_err());
}
#[test]
fn future_schema_refuses_open_before_wal_and_cached_current_guards_refuse_future_writer_contract() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let s = Store::open(&path).unwrap();
    let mut cached = s
        .connection
        .prepare("UPDATE runtime_epoch SET epoch=epoch+1 WHERE singleton=1")
        .unwrap();
    cached.execute([]).unwrap();
    // Isolated future-contract fixture, NOT a production Review11 migration.
    let future = Connection::open(&path).unwrap();
    future.execute_batch("DROP TRIGGER writer_runtime_epoch_UPDATE; CREATE TRIGGER writer_runtime_epoch_UPDATE BEFORE UPDATE ON runtime_epoch WHEN rrx_writer_contract_version()<>11 BEGIN SELECT RAISE(ABORT,'incompatible rrx writer contract'); END;").unwrap();
    future.pragma_update(None, "user_version", 11).unwrap();
    assert!(cached.execute([]).is_err());
    assert!(Store::open(&path).is_err());
}
#[test]
fn fresh_private_tables_have_complete_column_images_and_domain_guards() {
    let mut s = Store::memory().unwrap();
    for table in TABLES.iter().filter(|t| **t != "scoped_session_identities") {
        let mut q = s
            .connection
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap();
        let actual = q
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(actual, columns(table).unwrap(), "{table}");
        for action in ["INSERT", "UPDATE", "DELETE"] {
            assert!(
                s.connection
                    .query_row(
                        "SELECT 1 FROM sqlite_schema WHERE name=?1",
                        [format!("binding_{table}_{action}")],
                        |r| r.get::<_, i64>(0)
                    )
                    .is_ok()
            );
        }
    }
    assert!(
        s.audit(
            &Scope::project(ProjectId::new()),
            "rrx.private.workflow.session_bound",
            serde_json::json!({})
        )
        .is_err()
    );
}

// Export an actual original00df Store-created/migrated schema10. This test-only
// harness does not change any original production byte or issue native proofs.
#[test]
#[ignore = "owned historical fixture exporter, explicit target path required"]
fn historical_store_harness_initializes_actual00df() {
    let path = std::path::PathBuf::from(std::env::var_os("RRX_BINDING10_OLD_STORE_PATH")
        .expect("explicit owned fixture path required"));
    assert!(path.is_absolute());
    let store = Store::open(&path).unwrap();
    assert_eq!(store.schema_version().unwrap(), 10);
    assert_eq!(store.connection.query_row(
        "SELECT count(*) FROM sqlite_schema WHERE name='binding_record_native_ref'",
        [], |r|r.get::<_,i64>(0)).unwrap(), 0);
    assert_eq!(store.connection.query_row(
        "SELECT count(*) FROM managed_phase_operations", [], |r|r.get::<_,i64>(0)).unwrap(),0);
    drop(store);
}
