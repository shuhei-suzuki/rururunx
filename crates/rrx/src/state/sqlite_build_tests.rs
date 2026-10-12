//! The bundled SQLite build that every Store connection uses.
use rusqlite::Connection;

/// Issue #87 D4: `.cargo/config.toml` compiles the bundled SQLite without
/// allocator statistics. With them on, every allocation and free takes one
/// process-global mutex, and under parallel load that mutex stalls Tokio
/// workers inside Store calls.
#[test]
fn bundled_sqlite_keeps_no_allocator_statistics() {
    let connection = Connection::open_in_memory().unwrap();
    let options: Vec<String> = connection
        .prepare("PRAGMA compile_options")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert!(
        options.iter().any(|option| option == "DEFAULT_MEMSTATUS=0"),
        "bundled SQLite compile options: {options:?}"
    );
}
