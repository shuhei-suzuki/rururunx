use super::super::*;
use uuid::Uuid;

// Actual schema8 objects and an already-open version8 connection, not a relabelled9 Store.
fn old8(path: &std::path::Path) -> Connection {
    let old = Connection::open(path).unwrap();
    old.create_scalar_function(
        "rrx_writer_contract_version",
        0,
        rusqlite::functions::FunctionFlags::SQLITE_UTF8
            | rusqlite::functions::FunctionFlags::SQLITE_INNOCUOUS,
        |_| Ok(8_i64),
    )
    .unwrap();
    for sql in [
        include_str!("../schema.sql"),
        include_str!("../execution.sql"),
        include_str!("../execution/native_results.sql"),
        include_str!("../execution/source_recovery.sql"),
        include_str!("../execution/verification.sql"),
    ] {
        old.execute_batch(sql).unwrap();
    }
    old.execute(
        "INSERT INTO runtime_epoch(singleton,instance_id,epoch) VALUES(1,?1,0)",
        [Uuid::new_v4().to_string()],
    )
    .unwrap();
    for table in execution::MUTABLE_TABLES
        .iter()
        .filter(|t| !super::TABLES.contains(t))
    {
        for action in ["INSERT", "UPDATE", "DELETE"] {
            old.execute_batch(&format!("CREATE TRIGGER writer_{table}_{action} BEFORE {action} ON {table} WHEN rrx_writer_contract_version()<>8 BEGIN SELECT RAISE(ABORT,'incompatible rrx writer contract'); END;")).unwrap();
        }
    }
    old.pragma_update(None, "application_id", APPLICATION_ID)
        .unwrap();
    old.pragma_update(None, "user_version", 8).unwrap();
    old
}
#[test]
fn schema9_orders_actual8_and_fences_preopened_cached_writer_all_tables() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("old8.db");
    let old = old8(&path);
    let mut cached = old
        .prepare("UPDATE runtime_epoch SET epoch=epoch+1 WHERE singleton=1")
        .unwrap();
    cached.execute([]).unwrap();
    let current = Store::open(&path).unwrap();
    assert_eq!(current.schema_version().unwrap(), 9);
    assert!(
        cached
            .execute([])
            .unwrap_err()
            .to_string()
            .contains("incompatible rrx writer contract")
    );
    assert!(
        old.execute("DELETE FROM scheduler_clock", [])
            .unwrap_err()
            .to_string()
            .contains("incompatible rrx writer contract")
    );
    for table in execution::MUTABLE_TABLES {
        for action in ["INSERT", "UPDATE", "DELETE"] {
            let sql: String = current
                .connection
                .query_row(
                    "SELECT sql FROM sqlite_schema WHERE name=?1",
                    [format!("writer_{table}_{action}")],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(sql.contains("<>9"), "{table}:{action}");
        }
    }
    assert_eq!(
        current
            .connection
            .query_row("SELECT epoch FROM runtime_epoch", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        current
            .connection
            .query_row("SELECT count(*) FROM goal_authority", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        0
    );
    drop(cached);
    drop(old);
    drop(current);
    assert_eq!(Store::open(&path).unwrap().schema_version().unwrap(), 9);
}
#[test]
fn schema9_collision_refuses_without_changing_actual8_bytes() {
    for collision in [
        "CREATE TABLE task_drivers(unrelated TEXT)",
        "CREATE TRIGGER runtime_goal_definition AFTER UPDATE ON goals BEGIN SELECT 1; END;",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("old8.db");
        let old = old8(&path);
        old.execute_batch(collision).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(Store::open(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(
            old.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            8
        );
    }
}
