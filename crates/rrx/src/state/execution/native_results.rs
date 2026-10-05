//! Typed bounded reads; DTOs returned here do not carry native producer authority.
use super::*;
use crate::execution::native_result::{
    INVOCATION_BYTES, NativeInvocation, NativeResultReceipt, RECEIPT_BYTES,
};

pub(in crate::state) fn install_schema(tx: &Transaction<'_>) -> Result<()> {
    tx.execute_batch(include_str!("native_results.sql"))?;
    Ok(())
}
pub(in crate::state) fn validate_legacy_namespace(tx: &Transaction<'_>) -> Result<()> {
    let count: usize = tx.query_row(
        "SELECT COUNT(*) FROM sqlite_schema WHERE name IN ('native_invocations','native_results','native_invocation_identity','native_invocation_no_delete','native_result_no_update','native_result_no_delete')",
        [], |r| r.get(0))?;
    ensure!(count == 0, "legacy native receipt namespace is not empty");
    Ok(())
}
fn bounded_body(
    connection: &Connection,
    table: &str,
    id: &str,
    maximum: usize,
) -> Result<Option<String>> {
    let n: Option<usize> = connection
        .query_row(
            &format!("SELECT length(CAST(body AS BLOB)) FROM {table} WHERE id=?1"),
            [id],
            |r| r.get(0),
        )
        .optional()?;
    let Some(n) = n else { return Ok(None) };
    ensure!(n <= maximum, "native content body exceeds bound");
    Ok(Some(connection.query_row(
        &format!("SELECT body FROM {table} WHERE id=?1"),
        [id],
        |r| r.get(0),
    )?))
}
pub(super) fn invocation_tx(
    connection: &Connection,
    id: NativeInvocationId,
) -> Result<NativeInvocation> {
    let invocation: NativeInvocation = decode(
        bounded_body(
            connection,
            "native_invocations",
            &id.to_string(),
            INVOCATION_BYTES,
        )?
        .context("native invocation missing")?,
    )?;
    invocation.validate()?;
    ensure!(invocation.id == id, "native invocation identity mismatch");
    let mut columns = scoped_columns(&invocation.scope);
    columns.extend([
        ("id", json!(id)),
        ("unit_id", json!(invocation.unit_id)),
        ("session_id", json!(invocation.session_id)),
        ("generation", json!(invocation.generation)),
        ("owner_epoch", json!(invocation.owner_epoch)),
        ("provider", json!(invocation.provider)),
        ("state", json!(invocation.state)),
        ("version", json!(invocation.version)),
        ("input_operation", json!(invocation.input_operation)),
        ("native_thread", json!(invocation.native_thread)),
        ("native_turn", json!(invocation.native_turn)),
    ]);
    check_indexed(connection, "native_invocations", &columns)?;
    let unit = unit_tx(connection, invocation.unit_id)?;
    ensure!(
        unit.scope == invocation.scope
            && unit.generation == invocation.generation
            && unit.owner_epoch == invocation.owner_epoch
            && unit.session_id == Some(invocation.session_id)
            && unit.provider == invocation.provider,
        "native invocation Unit mismatch"
    );
    let matched: bool=connection.query_row("SELECT EXISTS(SELECT 1 FROM session_units WHERE session_id=?1 AND unit_id=?2 AND project_id=?3 AND goal_id=?4 AND task_id=?5)",
        params![invocation.session_id.to_string(),invocation.unit_id.to_string(),invocation.scope.project_id.to_string(),invocation.scope.goal_id.unwrap().to_string(),invocation.scope.task_id.unwrap().to_string()],|r|r.get(0))?;
    ensure!(matched, "native invocation Session mismatch");
    Ok(invocation)
}
impl Store {
    pub fn native_invocation(&self, id: NativeInvocationId) -> Result<NativeInvocation> {
        invocation_tx(&self.connection, id)
    }
    pub fn native_result(&self, id: NativeResultId) -> Result<NativeResultReceipt> {
        let receipt: NativeResultReceipt = decode(
            bounded_body(
                &self.connection,
                "native_results",
                &id.to_string(),
                RECEIPT_BYTES,
            )?
            .context("native result missing")?,
        )?;
        receipt.validate()?;
        ensure!(receipt.id == id, "native receipt identity mismatch");
        let mut columns = scoped_columns(&receipt.scope);
        columns.extend([
            ("id", json!(id)),
            ("invocation_id", json!(receipt.invocation_id)),
            ("unit_id", json!(receipt.unit_id)),
            ("session_id", json!(receipt.session_id)),
            ("generation", json!(receipt.generation)),
            ("owner_epoch", json!(receipt.owner_epoch)),
            ("provider", json!(receipt.provider)),
            ("acquisition", json!(receipt.acquisition)),
            ("authority", json!(receipt.authority)),
            ("version", json!(receipt.version)),
        ]);
        check_indexed(&self.connection, "native_results", &columns)?;
        let invocation = invocation_tx(&self.connection, receipt.invocation_id)?;
        ensure!(
            receipt.unit_id == invocation.unit_id
                && receipt.session_id == invocation.session_id
                && receipt.scope == invocation.scope
                && receipt.generation == invocation.generation
                && receipt.owner_epoch == invocation.owner_epoch
                && receipt.provider == invocation.provider
                && receipt.native_thread == invocation.native_thread
                && receipt.native_turn == invocation.native_turn,
            "native result binding mismatch"
        );
        Ok(receipt)
    }
}
