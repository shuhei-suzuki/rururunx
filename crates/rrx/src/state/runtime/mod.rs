//! Runtime integration; routing observations carry no executable authority.
pub(crate) mod driver;
mod goals;
mod proposals;
pub(crate) mod read;
mod recorded;
mod routing;
mod service;
mod waiting;

pub(super) const TABLES: &[&str] = &[
    "goal_authority",
    "runtime_control_acks",
    "scheduler_clock",
    "scheduler_projects",
    "scheduler_goals",
    "scheduler_tasks",
    "task_drivers",
    "goal_observations",
];
pub(super) fn install_schema(tx: &rusqlite::Transaction<'_>) -> anyhow::Result<()> {
    tx.execute_batch(include_str!("schema.sql"))?;
    Ok(())
}
pub(super) fn validate_legacy_namespace(tx: &rusqlite::Transaction<'_>) -> anyhow::Result<()> {
    let reserved:u64=tx.query_row("SELECT count(*) FROM sqlite_schema WHERE name IN ('runtime_goal_no_replace','runtime_goal_definition','runtime_goal_no_delete')",[],|r|r.get(0))?;
    anyhow::ensure!(
        reserved == 0,
        "legacy Runtime trigger namespace is not empty"
    );
    for name in TABLES {
        let count: u64 = tx.query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name=?1 OR (type='trigger' AND tbl_name=?1)",
            [name],
            |r| r.get(0),
        )?;
        anyhow::ensure!(count == 0, "legacy Runtime namespace is not empty");
    }
    Ok(())
}

#[cfg(test)]
mod tests;
