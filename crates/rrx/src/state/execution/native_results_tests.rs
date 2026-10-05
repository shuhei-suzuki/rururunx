use super::*;
use rusqlite::functions::FunctionFlags;

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
    for table in MUTABLE_TABLES
        .iter()
        .filter(|table| !matches!(**table, "native_invocations" | "native_results"))
    {
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
    assert_eq!(current.schema_version().unwrap(), 6);
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
            assert!(sql.contains("<>6"));
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
    assert_eq!(Store::open(&path).unwrap().schema_version().unwrap(), 6);
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
