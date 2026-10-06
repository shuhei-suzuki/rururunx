//! Complete scoped negative identity observation. This never issues a Session,
//! Native owner or input proof; protected projection writers remain mandatory.
use crate::domain::{Scope, SessionId};
use anyhow::{Result, ensure};
use rusqlite::{Connection, params};

const IDENTITIES: usize = 4096;

/// Read BOTH scoped record and scoped projection inventories. Missing, foreign,
/// malformed or stale projections cannot disappear through an inner join.
pub(super) fn validate_negative_identities(
    c: &Connection,
    scope: &Scope,
    own: SessionId,
    provider: &str,
    native_ref: Option<&str>,
) -> Result<()> {
    ensure!(
        !provider.is_empty()
            && provider.len() <= 512
            && native_ref.is_none_or(|id| !id.is_empty() && id.len() <= 512),
        "own native Session identity exceeds projection profile"
    );
    let mut q = c.prepare(
        "WITH record_ids AS (SELECT CASE WHEN length(CAST(id AS BLOB))<=36 THEN id END id FROM records WHERE project_id=?1 AND goal_id IS ?2 AND task_id IS ?3 AND kind='session' LIMIT 4097),projection_ids AS (SELECT CASE WHEN length(CAST(session_id AS BLOB))<=36 THEN session_id END session_id FROM scoped_session_identities WHERE project_id=?1 AND goal_id IS ?2 AND task_id IS ?3 LIMIT 4097),ids AS (SELECT id FROM record_ids UNION SELECT session_id FROM projection_ids LIMIT 4097) SELECT ids.id,CASE WHEN length(CAST(r.kind AS BLOB))<=32 THEN r.kind END,CASE WHEN length(CAST(r.project_id AS BLOB))<=36 THEN r.project_id END,CASE WHEN length(CAST(r.goal_id AS BLOB))<=36 THEN r.goal_id END,CASE WHEN length(CAST(r.task_id AS BLOB))<=36 THEN r.task_id END,r.version,CASE WHEN length(CAST(i.session_id AS BLOB))<=36 THEN i.session_id END,CASE WHEN length(CAST(i.project_id AS BLOB))<=36 THEN i.project_id END,CASE WHEN length(CAST(i.goal_id AS BLOB))<=36 THEN i.goal_id END,CASE WHEN length(CAST(i.task_id AS BLOB))<=36 THEN i.task_id END,i.record_version,CASE WHEN length(CAST(i.provider AS BLOB))<=512 THEN i.provider END,CASE WHEN i.native_ref IS NULL OR length(CAST(i.native_ref AS BLOB))<=512 THEN i.native_ref END,i.malformed,length(CAST(i.native_ref AS BLOB)) FROM ids LEFT JOIN records r ON r.id=ids.id LEFT JOIN scoped_session_identities i ON i.session_id=ids.id ORDER BY ids.id"
    )?;
    let project = scope.project_id.to_string();
    let goal = scope.goal_id.map(|id| id.to_string());
    let task = scope.task_id.map(|id| id.to_string());
    let mut cursor = q.query(params![project, goal, task])?;
    let mut count = 0;
    let mut own_seen = false;
    while let Some(row) = cursor.next()? {
        ensure!(
            count <= IDENTITIES,
            "complete scoped Session identity history exceeds4096"
        );
        count += 1;
        let id: String = row.get(0)?;
        let indexed_id: Option<String> = row.get(6)?;
        let record_version: Option<u64> = row.get(5)?;
        let indexed_version: Option<u64> = row.get(10)?;
        let selected_provider: Option<String> = row.get(11)?;
        let selected_ref: Option<String> = row.get(12)?;
        let native_bytes: Option<usize> = row.get(14)?;
        ensure!(
            id.parse::<SessionId>()
                .is_ok_and(|parsed| parsed.to_string() == id)
                && indexed_id.as_deref() == Some(id.as_str())
                && row.get::<_, Option<String>>(1)?.as_deref() == Some("session")
                && row.get::<_, Option<String>>(2)?.as_ref() == Some(&project)
                && row.get::<_, Option<String>>(3)? == goal
                && row.get::<_, Option<String>>(4)? == task
                && row.get::<_, Option<String>>(7)?.as_ref() == Some(&project)
                && row.get::<_, Option<String>>(8)? == goal
                && row.get::<_, Option<String>>(9)? == task
                && record_version.is_some_and(|v| v > 0 && v <= i64::MAX as u64)
                && indexed_version == record_version
                && row.get::<_, Option<i64>>(13)? == Some(0)
                && selected_provider.as_ref().is_some_and(|p| !p.is_empty())
                && native_bytes.is_none_or(|n| n > 0 && n <= 512)
                && native_bytes.is_none() == selected_ref.is_none(),
            "complete Session identity projection is missing/foreign/malformed"
        );
        if id == own.to_string() {
            ensure!(
                !own_seen
                    && selected_provider.as_deref() == Some(provider)
                    && selected_ref.as_deref() == native_ref,
                "own latest Session identity differs from protected projection"
            );
            own_seen = true;
        } else if let Some(native_ref) = native_ref {
            ensure!(
                selected_provider.as_deref() != Some(provider)
                    || selected_ref.as_deref() != Some(native_ref),
                "distinct historical Session owns the same native identity"
            );
        }
    }
    ensure!(own_seen, "own latest Session identity is absent");
    Ok(())
}
