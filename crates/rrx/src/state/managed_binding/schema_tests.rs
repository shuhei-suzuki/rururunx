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

// These are actual migrated factual history controls for the negative reader.
// They do not construct a Native actor, Driver, admission or positive binder.
fn check_negative_history(store: &Store, scope: &Scope, own: &Record) -> anyhow::Result<()> {
    let s: Session = serde_json::from_value(own.data.clone()).unwrap();
    super::session_identity::validate_negative_identities(
        &store.connection,
        scope,
        s.id,
        &s.provider,
        s.native_ref.as_deref(),
    )
}

#[test]
fn negative_history_unique_identity_is_read_only_and_lost_duplicate_refuses() {
    for duplicate in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let old = old9(&path);
        let scope = history(&old);
        let own = session(&scope, SessionState::Running);
        insert_record(&old, &own, None);
        let mut lost = session(&scope, SessionState::Lost);
        if !duplicate {
            lost.data["native_ref"] = serde_json::json!("different-native");
        }
        insert_record(&old, &lost, None);
        let store = Store::open(&path).unwrap();
        let before: i64 = store
            .connection
            .query_row("SELECT total_changes()", [], |r| r.get(0))
            .unwrap();
        let result = check_negative_history(&store, &scope, &own);
        if duplicate {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("distinct historical Session owns the same native identity")
            );
        } else {
            result.unwrap();
        }
        assert_eq!(
            store
                .connection
                .query_row("SELECT total_changes()", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            before
        );
    }
}

#[test]
fn negative_history_null_identity_and_different_provider_do_not_invent_conflicts() {
    for null_own in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let old = old9(&path);
        let scope = history(&old);
        let mut own = session(&scope, SessionState::Running);
        if null_own {
            own.data["native_ref"] = serde_json::Value::Null;
        }
        insert_record(&old, &own, None);
        let mut other = session(&scope, SessionState::Lost);
        if !null_own {
            other.data["provider"] = serde_json::json!("codex");
        }
        insert_record(&old, &other, None);
        let store = Store::open(&path).unwrap();
        check_negative_history(&store, &scope, &own).unwrap();
    }
}

#[test]
fn negative_history_malformed_unrelated_history_cannot_disappear() {
    for malformed in ["extra-field", "long-native"] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let old = old9(&path);
        let scope = history(&old);
        let own = session(&scope, SessionState::Running);
        insert_record(&old, &own, None);
        let mut other = session(&scope, SessionState::Lost);
        other.data["native_ref"] = serde_json::json!("unrelated-native");
        if malformed == "extra-field" {
            other.data["unexpected"] = serde_json::json!(true);
        } else {
            other.data["native_ref"] = serde_json::json!("x".repeat(513));
        }
        insert_record(&old, &other, None);
        let store = Store::open(&path).unwrap();
        assert!(
            check_negative_history(&store, &scope, &own)
                .unwrap_err()
                .to_string()
                .contains("complete Session identity projection is missing/foreign/malformed")
        );
    }
}

#[test]
fn negative_history_own_projection_and_presence_are_required() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let old = old9(&path);
    let scope = history(&old);
    let mut own = session(&scope, SessionState::Running);
    insert_record(&old, &own, None);
    let store = Store::open(&path).unwrap();
    own.data["native_ref"] = serde_json::json!("caller-drift");
    assert!(
        check_negative_history(&store, &scope, &own)
            .unwrap_err()
            .to_string()
            .contains("own latest Session identity differs from protected projection")
    );
    let absent = session(&scope, SessionState::Running);
    // Use a null candidate to avoid mistaking a duplicate-UUID refusal for absence.
    let mut absent = absent;
    absent.data["native_ref"] = serde_json::Value::Null;
    assert!(
        check_negative_history(&store, &scope, &absent)
            .unwrap_err()
            .to_string()
            .contains("own latest Session identity is absent")
    );
}

#[test]
fn negative_history_exact4096_accepts_and4097_refuses_without_cross_scope_charge() {
    for count in [4096, 4097] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let old = old9(&path);
        let scope = history(&old);
        let own = session(&scope, SessionState::Running);
        insert_record(&old, &own, None);
        for index in 1..count {
            let mut other = session(&scope, SessionState::Lost);
            other.data["native_ref"] = serde_json::json!(format!("history-{index}"));
            insert_record(&old, &other, None);
        }
        let other_scope = Scope {
            project_id: scope.project_id,
            goal_id: None,
            task_id: None,
        };
        // Same native UUID in another scope neither conflicts nor consumes this bound.
        insert_record(&old, &session(&other_scope, SessionState::Lost), None);
        let store = Store::open(&path).unwrap();
        let result = check_negative_history(&store, &scope, &own);
        if count == 4096 {
            result.unwrap();
        } else {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("complete scoped Session identity history exceeds4096")
            );
        }
    }
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
    a2.data["pid"] = serde_json::json!(12345);
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
    assert_eq!(actual, (2, "same-native".into()));
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

#[test]
fn initializer_rechecks_current_after_initial_observation_without_reinstalling() {
    for initial in [0, 9] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("concurrent.db");
        if initial == 9 {
            drop(old9(&path));
        }
        let (observed_send, observed_receive) = std::sync::mpsc::sync_channel(0);
        let (release_send, release_receive) = std::sync::mpsc::sync_channel(0);
        let worker_path = path.clone();
        let worker = std::thread::spawn(move || {
            let connection = Connection::open(worker_path).unwrap();
            Store::initialize_observed(connection, |version| {
                // This is the actual first read inside the production initializer.
                observed_send.send(version).unwrap();
                release_receive
                    .recv_timeout(std::time::Duration::from_secs(10))
                    .unwrap();
            })
        });
        assert_eq!(
            observed_receive
                .recv_timeout(std::time::Duration::from_secs(10))
                .unwrap(),
            initial
        );
        // Complete the competing initializer BEFORE releasing the first reader.
        let winner = Store::open(&path).unwrap();
        assert_eq!(winner.schema_version().unwrap(), SCHEMA_VERSION);
        let schema_before: i64 = winner
            .connection
            .query_row("SELECT count(*) FROM sqlite_schema", [], |r| r.get(0))
            .unwrap();
        release_send.send(()).unwrap();
        let late = worker.join().unwrap().expect("late initializer must accept the completed current schema without reinstalling Binding10");
        assert_eq!(late.schema_version().unwrap(), SCHEMA_VERSION);
        assert_eq!(
            late.connection
                .query_row("SELECT count(*) FROM sqlite_schema", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            schema_before
        );
        assert_eq!(
            late.connection
                .query_row("SELECT count(*) FROM managed_phase_operations", [], |r| r
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            0
        );
    }
}

fn session_facts(store: &Store, id: RecordId) -> (String, String, i64) {
    let record = store
        .connection
        .query_row(
            "SELECT body FROM records WHERE id=?1",
            [id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    let index = store
        .connection
        .query_row(
            "SELECT body FROM scoped_session_identities WHERE session_id=?1",
            [id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    let events = store
        .connection
        .query_row("SELECT count(*) FROM audit", [], |r| r.get(0))
        .unwrap();
    (record, index, events)
}
fn task_path(c: &Connection, scope: &Scope, path: &Path) {
    let raw: String = c
        .query_row(
            "SELECT body FROM tasks WHERE id=?1",
            [scope.task_id.unwrap().to_string()],
            |r| r.get(0),
        )
        .unwrap();
    let mut task: Task = serde_json::from_str(&raw).unwrap();
    task.worktree = Some(path.to_owned());
    task.branch = Some("historical".into());
    c.execute(
        "UPDATE tasks SET body=?1 WHERE id=?2",
        params![serde_json::to_string(&task).unwrap(), task.id.to_string()],
    )
    .unwrap();
}
#[test]
fn store_history_uuid_cannot_be_removed_or_replaced_and_same_identity_status_is_valid() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let old = old9(&path);
    let scope = history(&old);
    let work = dir.path().join("work");
    std::fs::create_dir(&work).unwrap();
    task_path(&old, &scope, &work);
    let histories = [SessionState::Exited, SessionState::Lost].map(|state| {
        let mut record = session(&scope, state);
        record.data["worktree"] = serde_json::json!(work);
        insert_record(&old, &record, None);
        record
    });
    let mut store = Store::open(&path).unwrap();
    for record in &histories {
        let before = session_facts(&store, record.id);
        for replacement in [
            serde_json::json!("different-native"),
            serde_json::Value::Null,
        ] {
            let mut changed = record.clone();
            changed.data["native_ref"] = replacement;
            let error = store.put_record(&mut changed).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("established native Session identity is immutable"),
                "expected SQL identity fence, got {error:#}"
            );
            assert_eq!(
                session_facts(&store, record.id),
                before,
                "failed historical update changed record/index/audit"
            );
            assert_eq!(
                changed.version, record.version,
                "failed writer changed caller version"
            );
        }
    }
    assert_eq!(store.connection.query_row("SELECT count(*) FROM scoped_session_identities WHERE provider='claude' AND native_ref='same-native'",[],|r|r.get::<_,i64>(0)).unwrap(),2);
    // Factual terminal metadata and conservative Lost remain legitimate; no cleanup authority is granted.
    for mut record in histories {
        record.data["pid"] = serde_json::json!(12345);
        store.put_record(&mut record).unwrap();
        assert_eq!(record.version, 2);
        assert_eq!(
            store
                .connection
                .query_row(
                    "SELECT native_ref FROM scoped_session_identities WHERE session_id=?1",
                    [record.id.to_string()],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            "same-native"
        );
    }
}
#[test]
fn store_initial_none_to_some_and_same_uuid_lifecycle_pid_update_remain_valid() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let old = old9(&path);
    let scope = history(&old);
    let work = dir.path().join("work");
    std::fs::create_dir(&work).unwrap();
    task_path(&old, &scope, &work);
    let mut record = session(&scope, SessionState::Starting);
    record.data["worktree"] = serde_json::json!(work);
    record.data["native_ref"] = serde_json::Value::Null;
    insert_record(&old, &record, None);
    let mut store = Store::open(&path).unwrap();
    record.data["native_ref"] = serde_json::json!("first-native");
    store.put_record(&mut record).unwrap();
    assert_eq!(record.version, 2);
    record.data["state"] = serde_json::to_value(SessionState::Running).unwrap();
    record.data["pid"] = serde_json::json!(12345);
    store.put_record(&mut record).unwrap();
    assert_eq!(record.version, 3);
    let (actual, version) = store.session(SessionId(record.id.0)).unwrap().unwrap();
    assert_eq!(actual.native_ref.as_deref(), Some("first-native"));
    assert_eq!(actual.state, SessionState::Running);
    assert_eq!(actual.pid, Some(12345));
    assert_eq!(version, 3);
    assert_eq!(
        store
            .connection
            .query_row(
                "SELECT native_ref FROM scoped_session_identities WHERE session_id=?1",
                [record.id.to_string()],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "first-native"
    );
    // Raw supported mutation is subject to the same fence, not just public API checks.
    let before = session_facts(&store, record.id);
    let mut changed = record.clone();
    changed.version = 4;
    changed.data["native_ref"] = serde_json::Value::Null;
    assert!(
        store
            .connection
            .execute(
                "UPDATE records SET version=4,body=?1 WHERE id=?2",
                params![
                    serde_json::to_string(&changed).unwrap(),
                    record.id.to_string()
                ]
            )
            .is_err()
    );
    assert_eq!(session_facts(&store, record.id), before);
}

// Requires a separately compiled ACTUAL immutable00df Store test exporter. It
// initializes this owned path itself; no current trigger deletion substitutes.
fn actual_old10(path: &Path) {
    let executable = std::env::var_os("RRX_BINDING10_OLD_STORE_HARNESS")
        .expect("actual compiled00df historical Store harness required");
    let output = std::process::Command::new(executable)
        .args([
            "state::managed_binding::schema_tests::historical_store_harness_initializes_actual00df",
            "--exact",
            "--ignored",
        ])
        .env("RRX_BINDING10_OLD_STORE_PATH", path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "actual old Store producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"),
        "old Store producer did not execute its exact control"
    );
}
fn unchanged_old10(path: &Path) -> Vec<u8> {
    let c = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    assert_eq!(
        c.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        10
    );
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name='binding_record_native_ref'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM managed_phase_operations", [], |r| r
            .get::<_, i64>(
            0
        ))
        .unwrap(),
        0
    );
    drop(c);
    std::fs::read(path).unwrap()
}
#[test]
#[ignore = "requires recorded actual immutable00df Store harness; run in historical qualification"]
fn current_layout_refuses_actual_old10_store_before_wal_and_leaves_original_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("actual-old10.db");
    actual_old10(&path);
    let before = unchanged_old10(&path);
    let error = Store::open(&path)
        .err()
        .expect("old10 protection layout accepted");
    assert!(
        error
            .to_string()
            .contains("incompatible current state protection layout"),
        "{error:#}"
    );
    assert_eq!(
        unchanged_old10(&path),
        before,
        "refusal changed actual old10 database bytes"
    );
}
#[test]
#[ignore = "requires recorded actual immutable00df Store harness; run in historical qualification"]
fn locked_current_layout_refuses_actual_old10_winner_after_initial0_and9() {
    for initial in [0, 9] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("actual-old10-winner.db");
        if initial == 9 {
            let old = old9(&path);
            let _scope = history(&old);
            drop(old);
        }
        let (observed_send, observed_receive) = std::sync::mpsc::sync_channel(0);
        let (release_send, release_receive) = std::sync::mpsc::sync_channel(0);
        let worker_path = path.clone();
        let worker = std::thread::spawn(move || {
            Store::initialize_observed(Connection::open(worker_path).unwrap(), |version| {
                observed_send.send(version).unwrap();
                release_receive
                    .recv_timeout(std::time::Duration::from_secs(10))
                    .unwrap();
            })
        });
        let observed = observed_receive
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        // Finish the old production initializer on THIS same selected path.
        actual_old10(&path);
        let before = unchanged_old10(&path);
        release_send.send(()).unwrap();
        let result = worker.join().unwrap();
        assert_eq!(observed, initial);
        let error = result.err().expect("locked-current accepted old10 winner");
        assert!(
            error
                .to_string()
                .contains("incompatible current state protection layout"),
            "{error:#}"
        );
        assert_eq!(
            unchanged_old10(&path),
            before,
            "locked-current refusal changed winner bytes"
        );
    }
}
#[test]
fn current_layout_rejects_missing_changed_literal_and_extra_sql_objects() {
    for alteration in ["missing", "literal", "extra"] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("current.db");
        let store = Store::open(&path).unwrap();
        if alteration == "extra" {
            store
                .connection
                .execute_batch("CREATE VIEW unexpected_layout AS SELECT 1")
                .unwrap();
        } else {
            let sql: String = store
                .connection
                .query_row(
                    "SELECT sql FROM sqlite_schema WHERE name='binding_record_native_ref'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            store
                .connection
                .execute_batch("DROP TRIGGER binding_record_native_ref")
                .unwrap();
            if alteration == "literal" {
                // Whitespace inside the SQL literal is meaningful; normalization
                // must not erase this change when comparing the complete contract.
                let changed = sql.replace("native Session", "native  Session");
                assert_ne!(sql, changed);
                store.connection.execute_batch(&changed).unwrap();
            }
        }
        drop(store);
        let before = std::fs::read(&path).unwrap();
        assert!(
            Store::open(&path)
                .err()
                .unwrap()
                .to_string()
                .contains("incompatible current state protection layout")
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}
#[test]
fn complete_current_catalog_bounds_before_materializing_sql_objects() {
    for cause in ["object", "total", "count"] {
        let store = Store::memory().unwrap();
        match cause {
            "object" => {
                store
                    .connection
                    .execute_batch(&format!(
                        "CREATE VIEW oversized_sql AS SELECT '{}'",
                        "x".repeat(64 * 1024)
                    ))
                    .unwrap();
            }
            "total" => {
                for i in 0..40 {
                    store
                        .connection
                        .execute_batch(&format!(
                            "CREATE VIEW extra_{i} AS SELECT '{}'",
                            "x".repeat(60 * 1024)
                        ))
                        .unwrap();
                }
            }
            "count" => {
                for i in 0..1025 {
                    store
                        .connection
                        .execute_batch(&format!("CREATE VIEW extra_{i} AS SELECT 1"))
                        .unwrap();
                }
            }
            _ => unreachable!(),
        }
        let error = validate_current_layout(&store.connection).unwrap_err();
        assert!(
            error.to_string().contains("complete bound"),
            "{cause}: {error:#}"
        );
    }
}
