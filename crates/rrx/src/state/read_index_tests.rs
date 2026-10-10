//! S4 D6-R2/D6-R3 controls: the schema-11 read index, its migration entries
//! and the production seek plans. Fixtures change only schema objects and
//! `user_version`; no row, grant or authority is seeded.
use super::runtime::read::{PROJECT_GOAL_SEEK_SQL, PROJECT_GOALS_PAGE_SQL, RUNTIME_GOAL_SEEK_SQL};
use super::*;

fn plan(c: &Connection, sql: &str) -> String {
    let mut s = c.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
    let n = s.parameter_count();
    s.query_map(
        rusqlite::params_from_iter(std::iter::repeat_n(rusqlite::types::Null, n)),
        |r| r.get::<_, String>(3),
    )
    .unwrap()
    .map(Result::unwrap)
    .collect::<Vec<_>>()
    .join("\n")
}

/// R5'(2): each exact production statement seeks `goals_by_project`; no
/// `SCAN` of goals and no temporary B-tree.
fn assert_seeks(c: &Connection) {
    for sql in [
        PROJECT_GOALS_PAGE_SQL,
        PROJECT_GOAL_SEEK_SQL,
        RUNTIME_GOAL_SEEK_SQL,
    ] {
        let plan = plan(c, sql);
        assert!(
            plan.lines()
                .next()
                .is_some_and(|l| l.starts_with("SEARCH") && l.contains("goals_by_project")),
            "{plan}"
        );
        assert!(
            !plan.contains("SCAN") && !plan.contains("TEMP B-TREE"),
            "{plan}"
        );
    }
}
fn index_present(c: &Connection) -> bool {
    c.query_row(
        "SELECT count(*) FROM sqlite_schema WHERE type='index' AND name='goals_by_project' AND tbl_name='goals'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap()
        == 1
}
/// The selected database's catalog, version and every row, read only.
fn fingerprint(path: &Path) -> Vec<String> {
    let c = Connection::open(path).unwrap();
    let mut out = vec![format!(
        "user_version={}",
        c.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap()
    )];
    let mut q = c
        .prepare("SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name")
        .unwrap();
    let objects = q
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    drop(q);
    for (kind, name, sql) in &objects {
        out.push(format!("{kind}|{name}|{sql:?}"));
        if kind == "table" && !name.starts_with("sqlite_") {
            let mut rows = c.prepare(&format!("SELECT * FROM \"{name}\"")).unwrap();
            let n = rows.column_count();
            let mut all = rows
                .query_map([], |r| {
                    Ok((0..n)
                        .map(|i| format!("{:?}", r.get_ref(i).unwrap()))
                        .collect::<Vec<_>>()
                        .join(","))
                })
                .unwrap()
                .map(Result::unwrap)
                .collect::<Vec<_>>();
            all.sort();
            out.push(format!("{name}:{}", all.join(";")));
        }
    }
    out
}
/// A connection with only the given writer contract registered, as an
/// older binary would have it. It holds no permit or Runtime authority.
fn contract_writer(path: &Path, version: i64) -> Connection {
    let c = Connection::open(path).unwrap();
    c.create_scalar_function(
        "rrx_writer_contract_version",
        0,
        rusqlite::functions::FunctionFlags::SQLITE_UTF8
            | rusqlite::functions::FunctionFlags::SQLITE_INNOCUOUS,
        move |_| Ok(version),
    )
    .unwrap();
    c
}
fn writer_guards(c: &Connection) -> Vec<(String, String)> {
    let mut q = c
        .prepare("SELECT name,sql FROM sqlite_schema WHERE type='trigger' AND name GLOB 'writer_*' ORDER BY name")
        .unwrap();
    q.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}
/// R11(3): a genuine protected v10: this binary's own files and installers
/// with real rows, then the v10 shape exactly (no read index, v10 writer
/// guards, `user_version` 10). Only schema objects and the version change.
fn genuine_v10(path: &Path) {
    {
        let mut store = Store::open(path).unwrap();
        let root = path.parent().unwrap().join("v10-source");
        std::fs::create_dir_all(&root).unwrap();
        let mut project = Project::new(
            "v10".into(),
            root.canonicalize().unwrap(),
            "account-free-no-Git".into(),
            "main".into(),
        );
        store.put_project(&mut project).unwrap();
    }
    let c = contract_writer(path, SCHEMA_VERSION);
    c.execute_batch("DROP INDEX goals_by_project").unwrap();
    for (name, sql) in writer_guards(&c) {
        assert!(sql.contains(&format!("<>{SCHEMA_VERSION}")), "{name}");
        c.execute_batch(&format!(
            "DROP TRIGGER {name}; {};",
            sql.replace(&format!("<>{SCHEMA_VERSION}"), "<>10")
        ))
        .unwrap();
    }
    c.pragma_update(None, "user_version", 10).unwrap();
}

/// R11(1) / R5'(2): a fresh database carries the index, its layout is the
/// current one, and the production statements seek it.
#[test]
fn fresh_schema_11_has_the_read_index_and_seek_plans() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let store = Store::open(&path).unwrap();
    assert_eq!(store.schema_version().unwrap(), 11);
    assert!(index_present(&store.connection));
    managed_binding::validate_current_layout(&store.connection).unwrap();
    assert_seeks(&store.connection);
}

/// R11(3)/(5) / R5'(2): a genuine v10 migrates to 11 with every row
/// unchanged, gains exactly the index and the v11 guards, then refuses a
/// contract-10 writer; the migrated database seeks the index too.
#[test]
fn genuine_v10_migrates_to_11_adding_only_the_index_and_guards() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    genuine_v10(&path);
    let before = fingerprint(&path);
    let store = Store::open(&path).unwrap();
    assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    assert!(index_present(&store.connection));
    for (name, sql) in writer_guards(&store.connection) {
        assert!(sql.contains(&format!("<>{SCHEMA_VERSION}")), "{name}");
    }
    assert_seeks(&store.connection);
    drop(store);
    let after = fingerprint(&path);
    // Rows are identical; the catalog differs only by the index, the
    // re-created writer guards and the version.
    let rows = |f: &[String]| -> Vec<String> {
        f.iter()
            .filter(|l| !l.starts_with("trigger|writer_") && !l.starts_with("user_version"))
            .filter(|l| !l.starts_with("index|goals_by_project"))
            .cloned()
            .collect()
    };
    assert_eq!(rows(&after), rows(&before));
    assert!(
        after
            .iter()
            .any(|l| l.starts_with("index|goals_by_project"))
    );
    // R11(5): an old (contract-10) writer is refused on the migrated file.
    let old = contract_writer(&path, 10);
    assert!(
        old.execute("UPDATE runtime_epoch SET epoch=epoch WHERE singleton=1", [])
            .is_err()
    );
    let tables: usize = writer_guards(&old).len();
    assert_eq!(tables, 3 * execution::MUTABLE_TABLES.len());
}

/// One labelled change to a genuine v10 copy.
type Tamper = (&'static str, fn(&Connection));

/// R11(4)/(4'): an invalid v10 is refused with nothing changed. (4') are
/// the copies only the exact-v10 check can refuse: the v11 guard reinstall
/// would otherwise repair them.
#[test]
fn invalid_v10_copies_are_refused_unchanged() {
    let tamper: [Tamper; 5] = [
        ("binding trigger dropped", |c| {
            let name: String = c
                .query_row(
                    "SELECT name FROM sqlite_schema WHERE type='trigger' AND name NOT GLOB 'writer_*' ORDER BY name LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            c.execute_batch(&format!("DROP TRIGGER {name}")).unwrap();
        }),
        ("extra object", |c| {
            c.execute_batch("CREATE TABLE s4_extra(x)").unwrap()
        }),
        ("index already present", |c| {
            c.execute_batch("CREATE INDEX goals_by_project ON goals(project_id, id)")
                .unwrap()
        }),
        ("writer guard dropped", |c| {
            let (name, _) = writer_guards(c).remove(0);
            c.execute_batch(&format!("DROP TRIGGER {name}")).unwrap();
        }),
        ("writer guard altered", |c| {
            let (name, sql) = writer_guards(c).remove(0);
            c.execute_batch(&format!(
                "DROP TRIGGER {name}; {};",
                sql.replace("<>10", "<>9")
            ))
            .unwrap();
        }),
    ];
    for (label, apply) in tamper {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        genuine_v10(&path);
        apply(&contract_writer(&path, 10));
        let before = fingerprint(&path);
        assert!(Store::open(&path).is_err(), "{label}: accepted");
        assert_eq!(fingerprint(&path), before, "{label}: changed");
    }
}
