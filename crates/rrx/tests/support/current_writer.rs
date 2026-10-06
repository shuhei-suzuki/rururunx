//! Direct corruption canary connection, explicitly nongrant. This supplies the
//! current SQL function surfaces without a private permit or live Driver registry.
//! It is never a public/production writer or a genuine execution fixture.
//!
//! Surfaces mirror state/mod.rs::register_writer_contract, managed_binding/
//! permits.rs::register_permit_function, schema.rs::register_projection and
//! runtime/driver.rs::register_liveness: version(0), permit(-1), identity(6),
//! liveness(5), with the same UTF8/INNOCUOUS and deterministic flags. SQLite must
//! resolve functions even for a trigger branch that does not apply to this row.
//! Ordinary audit/context/usage, legacy DAG and proposed-Goal observation canaries
//! need no grant. Every permit/liveness query returns false for every argument;
//! identity projection raises an error rather than creating Session metadata.
//! No trigger, schema, PRAGMA, production function or permission is changed.
pub fn open(path: impl AsRef<std::path::Path>) -> rusqlite::Result<rusqlite::Connection> {
    let connection = rusqlite::Connection::open(path)?;
    let nongrant_flags = rusqlite::functions::FunctionFlags::SQLITE_UTF8
        | rusqlite::functions::FunctionFlags::SQLITE_INNOCUOUS;
    connection.create_scalar_function("rrx_binding_permit", -1, nongrant_flags, |_| Ok(false))?;
    connection.create_scalar_function("rrx_live_task_driver", 5, nongrant_flags, |_| Ok(false))?;
    connection.create_scalar_function(
        "rrx_session_identity",
        6,
        nongrant_flags | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC,
        |_| {
            Err::<String, _>(rusqlite::Error::UserFunctionError(
                "nongrant corruption canary cannot project Session identity".into(),
            ))
        },
    )?;
    connection.create_scalar_function(
        "rrx_writer_contract_version",
        0,
        rusqlite::functions::FunctionFlags::SQLITE_UTF8
            | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC
            | rusqlite::functions::FunctionFlags::SQLITE_INNOCUOUS,
        |_| Ok(rrx::state::SCHEMA_VERSION),
    )?;
    Ok(connection)
}
