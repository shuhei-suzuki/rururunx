//! Binding10 installation, negative Session projection and exact writer guards.
//! No owner/input/marker/Driver is reconstructed by migration or an indexed DTO.
use super::{
    canonical::{BODY_BYTES, Body, decode_value},
    permits::{self, ExactRowMutation, PrivatePermitManager},
};
use crate::{
    domain::{Record, RecordKind, Session},
    state::{APPLICATION_ID, SCHEMA_VERSION},
};
use anyhow::{Context, Result, ensure};
use rusqlite::{
    Connection, Transaction, functions::FunctionFlags, params, types::Value as SqlValue,
};
use serde_json::json;

pub(in crate::state) const TABLES: &[&str] = &[
    "workflow_native_contracts",
    "managed_phase_operations",
    "managed_marker_bodies",
    "managed_phase_owners",
    "managed_phase_inputs",
    "managed_phase_admissions",
    "managed_phase_readiness",
    "scoped_session_identities",
];
pub(super) const MIGRATION_ROWS: usize = 65_536;
pub(super) const MIGRATION_BYTES: usize = 512 * 1024 * 1024;
pub(super) const SESSION_BYTES: usize = 4 * 1024 * 1024;
const KINDS: &[&str] = &[
    "rrx.private.workflow.session_bound",
    "rrx.private.workflow.native_diagnostic",
    "rrx.private.workflow.gate_claim",
    "rrx.private.workflow.gate_observed",
    "rrx.private.workflow.gate_hold",
    "rrx.private.workflow.terminal_decision",
    "rrx.private.workflow.phase_closed",
];
fn kinds() -> String {
    KINDS
        .iter()
        .map(|k| format!("'{k}'"))
        .collect::<Vec<_>>()
        .join(",")
}

/// A pure projection of original bounded bytes, not a token and not positive evidence.
/// Invalid/duplicate/unknown/mismatched content is retained as explicit malformed.
fn projection(
    id: &str,
    p: &str,
    g: Option<&str>,
    t: Option<&str>,
    version: i64,
    raw: Option<&str>,
) -> String {
    let valid = (|| -> Result<Session> {
        let raw = raw.context("Session body over bound")?;
        let value = decode_value(raw, SESSION_BYTES)?;
        let record: Record = serde_json::from_value(value.clone())?;
        ensure!(
            serde_json::to_value(&record)? == value,
            "Record drops unknown members"
        );
        ensure!(
            record.kind == RecordKind::Session
                && record.id.to_string() == id
                && record.scope.project_id.to_string() == p
                && record.scope.goal_id.map(|v| v.to_string()).as_deref() == g
                && record.scope.task_id.map(|v| v.to_string()).as_deref() == t
                && i64::try_from(record.version).ok() == Some(version),
            "Session envelope/index mismatch"
        );
        let session: Session = serde_json::from_value(record.data.clone())?;
        ensure!(
            serde_json::to_value(&session)? == record.data
                && session.id.to_string() == id
                && session.scope == record.scope,
            "Session payload/index mismatch"
        );
        ensure!(
            !session.provider.is_empty()
                && session.provider.len() <= 512
                && session
                    .native_ref
                    .as_ref()
                    .is_none_or(|v| !v.is_empty() && v.len() <= 512),
            "Session identity text over bound"
        );
        Ok(session)
    })();
    let (provider, native_ref, malformed) = match valid {
        Ok(s) => (Some(s.provider), s.native_ref, 0),
        Err(_) => (None, None, 1),
    };
    // Scope comes from the indexed source, even when its payload claims a foreign scope.
    json!({"session_id":id,"project_id":p,"goal_id":g,"task_id":t,"record_version":version,"provider":provider,"native_ref":native_ref,"malformed":malformed}).to_string()
}
pub(super) fn register_projection(c: &Connection) -> Result<()> {
    c.create_scalar_function(
        "rrx_session_identity",
        6,
        FunctionFlags::SQLITE_UTF8
            | FunctionFlags::SQLITE_DETERMINISTIC
            | FunctionFlags::SQLITE_INNOCUOUS,
        |ctx| {
            let id = ctx.get::<String>(0)?;
            let p = ctx.get::<String>(1)?;
            let g = ctx.get::<Option<String>>(2)?;
            let t = ctx.get::<Option<String>>(3)?;
            let version = ctx.get::<i64>(4)?;
            let raw = match ctx.get_raw(5) {
                rusqlite::types::ValueRef::Text(b) if b.len() <= SESSION_BYTES => {
                    std::str::from_utf8(b).ok()
                }
                _ => None,
            };
            Ok(projection(
                &id,
                &p,
                g.as_deref(),
                t.as_deref(),
                version,
                raw,
            ))
        },
    )?;
    Ok(())
}

/// Actual9 SQL objects are compared with the pinned compiled9 shape before any
/// installation writes. No label-only upgrade of a wrong or partial layout.
pub(in crate::state) fn validate_legacy_namespace(tx: &Transaction<'_>) -> Result<()> {
    let reserved:i64=tx.query_row("SELECT count(*) FROM sqlite_schema WHERE name GLOB 'binding_*' OR name IN ('workflow_native_contracts','managed_phase_operations','managed_marker_bodies','managed_phase_owners','managed_phase_inputs','managed_phase_admissions','managed_phase_readiness','scoped_session_identities')",[],|r|r.get(0))?;
    ensure!(reserved == 0, "Binding10 reserved namespace collision");
    let version: i64 = tx.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version == 9 {
        validate_actual9(tx)?;
    }
    Ok(())
}
fn validate_actual9(c: &Connection) -> Result<()> {
    let reference = Connection::open_in_memory()?;
    for sql in [
        include_str!("../schema.sql"),
        include_str!("../execution.sql"),
        include_str!("../execution/native_results.sql"),
        include_str!("../execution/source_recovery.sql"),
        include_str!("../execution/verification.sql"),
        include_str!("../runtime/schema.sql"),
    ] {
        reference.execute_batch(sql)?;
    }
    let mut q=reference.prepare("SELECT name,type,sql FROM sqlite_schema WHERE sql IS NOT NULL AND name NOT GLOB 'sqlite_*'")?;
    let expected_objects: i64 = reference.query_row(
        "SELECT count(*) FROM sqlite_schema WHERE sql IS NOT NULL AND name NOT GLOB 'sqlite_*'",
        [],
        |r| r.get(0),
    )?;
    let expected_writers = 3 * crate::state::execution::MUTABLE_TABLES
        .iter()
        .filter(|t| !TABLES.contains(t))
        .count() as i64;
    let actual_objects: i64 = c.query_row(
        "SELECT count(*) FROM sqlite_schema WHERE sql IS NOT NULL AND name NOT GLOB 'sqlite_*'",
        [],
        |r| r.get(0),
    )?;
    ensure!(
        actual_objects == expected_objects + expected_writers,
        "actual9 contains unknown or missing SQL objects"
    );
    for row in q.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })? {
        let (name, kind, sql) = row?;
        let actual: (String, String) = c
            .query_row(
                "SELECT type,sql FROM sqlite_schema WHERE name=?1",
                [&name],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .with_context(|| format!("actual9 object unavailable: {name}"))?;
        ensure!(actual == (kind, sql), "actual9 SQL layout differs: {name}");
    }
    for table in crate::state::execution::MUTABLE_TABLES
        .iter()
        .filter(|t| !TABLES.contains(t))
    {
        for action in ["INSERT", "UPDATE", "DELETE"] {
            let name = format!("writer_{table}_{action}");
            let expected = format!(
                "CREATE TRIGGER {name} BEFORE {action} ON {table} WHEN rrx_writer_contract_version()<>9 BEGIN SELECT RAISE(ABORT,'incompatible rrx writer contract'); END"
            );
            let actual: String = c.query_row(
                "SELECT sql FROM sqlite_schema WHERE name=?1",
                [&name],
                |r| r.get(0),
            )?;
            // Existing generated guards include optional spacing before END/semicolon.
            ensure!(
                normalize(&actual) == normalize(&expected),
                "actual9 writer layout differs: {table}/{action}"
            );
        }
    }
    ensure!(
        c.pragma_query_value::<i64, _>(None, "application_id", |r| r.get(0))? == APPLICATION_ID,
        "foreign actual9 database"
    );
    Ok(())
}
fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace() && *c != ';')
        .collect()
}

pub(in crate::state) fn install_schema(tx: &Transaction<'_>) -> Result<()> {
    tx.execute_batch(include_str!("schema.sql"))?;
    install_record_guards(tx)?;
    install_identity_guards(tx)?;
    install_ledger_guards(tx)?;
    Ok(())
}
fn permit(table: &str, action: &str) -> String {
    let columns = permits::columns(table).expect("compiled private table");
    let mut args = vec![format!("'{table}'"), format!("'{action}'")];
    args.extend(columns.iter().map(|col| {
        if action == "INSERT" {
            "NULL".into()
        } else {
            format!("OLD.{col}")
        }
    }));
    args.extend(columns.iter().map(|col| {
        if action == "DELETE" {
            "NULL".into()
        } else {
            format!("NEW.{col}")
        }
    }));
    format!("rrx_binding_permit({})", args.join(","))
}
fn install_record_guards(tx: &Transaction<'_>) -> Result<()> {
    for action in ["INSERT", "UPDATE", "DELETE"] {
        let row = if action == "DELETE" { "OLD" } else { "NEW" };
        let protected = format!(
            "(({row}.kind='workflow' AND EXISTS(SELECT 1 FROM workflow_native_contracts c WHERE c.workflow_id={row}.id)) OR ({row}.kind='session' AND EXISTS(SELECT 1 FROM managed_phase_owners o WHERE o.allocated_session_id={row}.id)))"
        );
        let old_protected = if action == "UPDATE" {
            " OR (OLD.kind='workflow' AND EXISTS(SELECT 1 FROM workflow_native_contracts c WHERE c.workflow_id=OLD.id)) OR (OLD.kind='session' AND EXISTS(SELECT 1 FROM managed_phase_owners o WHERE o.allocated_session_id=OLD.id))"
        } else {
            ""
        };
        tx.execute_batch(&format!("CREATE TRIGGER binding_record_{action} BEFORE {action} ON records WHEN ({protected}{old_protected}) AND NOT {} BEGIN SELECT RAISE(ABORT,'private managed Record writer required'); END;",permit("records",action)))?;
    }
    tx.execute_batch("CREATE TRIGGER binding_record_no_replace BEFORE INSERT ON records WHEN EXISTS(SELECT 1 FROM records r WHERE r.id=NEW.id AND (r.kind='session' OR EXISTS(SELECT 1 FROM workflow_native_contracts c WHERE c.workflow_id=r.id))) BEGIN SELECT RAISE(ABORT,'managed/history Record replacement refused'); END;
CREATE TRIGGER binding_record_session_identity BEFORE UPDATE ON records WHEN OLD.kind='session' AND (NEW.id IS NOT OLD.id OR NEW.kind IS NOT OLD.kind OR NEW.project_id IS NOT OLD.project_id OR NEW.goal_id IS NOT OLD.goal_id OR NEW.task_id IS NOT OLD.task_id OR NEW.version<>OLD.version+1) BEGIN SELECT RAISE(ABORT,'Session history identity/version retained'); END;
CREATE TRIGGER binding_record_native_ref BEFORE UPDATE ON records WHEN OLD.kind='session' AND json_type(OLD.body,'$.data.native_ref')='text' AND (json_type(NEW.body,'$.data.native_ref') IS NOT 'text' OR json_extract(NEW.body,'$.data.native_ref') IS NOT json_extract(OLD.body,'$.data.native_ref')) BEGIN SELECT RAISE(ABORT,'established native Session identity is immutable'); END;
CREATE TRIGGER binding_record_session_delete BEFORE DELETE ON records WHEN OLD.kind='session' BEGIN SELECT RAISE(ABORT,'Session negative history retained'); END;
CREATE TRIGGER binding_record_workflow_identity BEFORE UPDATE ON records WHEN EXISTS(SELECT 1 FROM workflow_native_contracts c WHERE c.workflow_id=OLD.id) AND (NEW.id IS NOT OLD.id OR NEW.kind IS NOT OLD.kind OR NEW.project_id IS NOT OLD.project_id OR NEW.goal_id IS NOT OLD.goal_id OR NEW.task_id IS NOT OLD.task_id OR NEW.version<>OLD.version+1) BEGIN SELECT RAISE(ABORT,'managed Workflow identity/version retained'); END;
CREATE TRIGGER binding_record_workflow_delete BEFORE DELETE ON records WHEN EXISTS(SELECT 1 FROM workflow_native_contracts c WHERE c.workflow_id=OLD.id) BEGIN SELECT RAISE(ABORT,'managed Workflow history retained'); END;")?;
    Ok(())
}
fn install_identity_guards(tx: &Transaction<'_>) -> Result<()> {
    let source = "rrx_session_identity(r.id,r.project_id,r.goal_id,r.task_id,r.version,CASE WHEN length(CAST(r.body AS BLOB))<=4194304 THEN r.body END)";
    let agreement = format!(
        "EXISTS(SELECT 1 FROM records r WHERE r.id=NEW.session_id AND r.kind='session' AND r.project_id IS NEW.project_id AND r.goal_id IS NEW.goal_id AND r.task_id IS NEW.task_id AND r.version IS NEW.record_version AND NEW.body={source} AND json_extract(NEW.body,'$.provider') IS NEW.provider AND json_extract(NEW.body,'$.native_ref') IS NEW.native_ref AND json_extract(NEW.body,'$.malformed') IS NEW.malformed)"
    );
    for action in ["INSERT", "UPDATE"] {
        tx.execute_batch(&format!("CREATE TRIGGER binding_identity_{action} BEFORE {action} ON scoped_session_identities WHEN NOT {agreement} BEGIN SELECT RAISE(ABORT,'Session projection differs from indexed source'); END;"))?;
    }
    tx.execute_batch("CREATE TRIGGER binding_identity_delete BEFORE DELETE ON scoped_session_identities BEGIN SELECT RAISE(ABORT,'negative Session identity retained'); END;
CREATE TRIGGER binding_identity_scope BEFORE UPDATE ON scoped_session_identities WHEN NEW.session_id IS NOT OLD.session_id OR NEW.project_id IS NOT OLD.project_id OR NEW.goal_id IS NOT OLD.goal_id OR NEW.task_id IS NOT OLD.task_id BEGIN SELECT RAISE(ABORT,'negative Session scope retained'); END;
CREATE TRIGGER binding_identity_no_replace BEFORE INSERT ON scoped_session_identities WHEN EXISTS(SELECT 1 FROM scoped_session_identities WHERE session_id=NEW.session_id) BEGIN SELECT RAISE(ABORT,'negative Session identity replacement refused'); END;")?;
    for action in ["INSERT", "UPDATE"] {
        let proj = "rrx_session_identity(NEW.id,NEW.project_id,NEW.goal_id,NEW.task_id,NEW.version,CASE WHEN length(CAST(NEW.body AS BLOB))<=4194304 THEN NEW.body END)";
        // UPDATE then INSERT-if-missing uses no UPSERT/REPLACE escape hatch.
        tx.execute_batch(&format!("CREATE TRIGGER binding_session_projection_{action} AFTER {action} ON records WHEN NEW.kind='session' BEGIN
UPDATE scoped_session_identities SET record_version=NEW.version,provider=json_extract({proj},'$.provider'),native_ref=json_extract({proj},'$.native_ref'),malformed=json_extract({proj},'$.malformed'),body={proj} WHERE session_id=NEW.id;
INSERT INTO scoped_session_identities(session_id,project_id,goal_id,task_id,record_version,provider,native_ref,malformed,body) SELECT NEW.id,NEW.project_id,NEW.goal_id,NEW.task_id,NEW.version,json_extract({proj},'$.provider'),json_extract({proj},'$.native_ref'),json_extract({proj},'$.malformed'),{proj} WHERE NOT EXISTS(SELECT 1 FROM scoped_session_identities WHERE session_id=NEW.id); END;"))?;
    }
    Ok(())
}
fn install_ledger_guards(tx: &Transaction<'_>) -> Result<()> {
    let ks = kinds();
    let op = "json_extract(data,'$.private_operation_ref')";
    let after = "json_extract(data,'$.workflow_version_after')";
    let prior = "json_extract(data,'$.prior_ledger_digest')";
    tx.execute_batch(&format!("CREATE UNIQUE INDEX binding_audit_successor ON audit({op},{after}) WHERE kind IN ({ks});
CREATE UNIQUE INDEX binding_audit_predecessor ON audit({op},{prior}) WHERE kind IN ({ks});
CREATE INDEX binding_audit_operation ON audit({op},sequence) WHERE kind IN ({ks});
CREATE TRIGGER binding_audit_private BEFORE INSERT ON audit WHEN NEW.kind GLOB 'rrx.private.workflow.*' AND (NEW.kind NOT IN ({ks}) OR NOT {}) BEGIN SELECT RAISE(ABORT,'private Workflow audit writer required'); END;
CREATE TRIGGER binding_audit_no_replace BEFORE INSERT ON audit WHEN NEW.kind IN ({ks}) AND EXISTS(SELECT 1 FROM audit a WHERE a.kind IN ({ks}) AND json_extract(a.data,'$.private_operation_ref')=json_extract(NEW.data,'$.private_operation_ref') AND (json_extract(a.data,'$.workflow_version_after')=json_extract(NEW.data,'$.workflow_version_after') OR json_extract(a.data,'$.prior_ledger_digest')=json_extract(NEW.data,'$.prior_ledger_digest'))) BEGIN SELECT RAISE(ABORT,'reserved ledger identity retained'); END;",permit("audit","INSERT")))?;
    let required = [
        "private_operation_ref",
        "original_marker_frame_sha256",
        "workflow_body_sha256_before",
        "workflow_body_sha256_after",
        "prior_ledger_digest",
        "phase",
        "project_id",
        "goal_id",
        "task_id",
        "workflow_id",
        "canonical_body_recipe",
    ];
    let header = required
        .iter()
        .map(|k| format!("json_type(NEW.data,'$.{k}') IS NOT 'text'"))
        .chain(
            [
                "workflow_version_before",
                "workflow_version_after",
                "generation",
                "attempt_index",
                "context_version",
            ]
            .into_iter()
            .map(|k| format!("json_type(NEW.data,'$.{k}') IS NOT 'integer'")),
        )
        .collect::<Vec<_>>()
        .join(" OR ");
    let unique_counts=[("session_bound",1),("native_diagnostic",47),("gate_claim",99),("gate_observed",99),("gate_hold",8),("terminal_decision",1),("phase_closed",1)].into_iter().map(|(name,max)|format!("(NEW.kind='rrx.private.workflow.{name}' AND (SELECT count(*) FROM audit a WHERE a.kind=NEW.kind AND json_extract(a.data,'$.private_operation_ref')=json_extract(NEW.data,'$.private_operation_ref'))>={max})")).collect::<Vec<_>>().join(" OR ");
    tx.execute_batch(&format!("CREATE TRIGGER binding_audit_chain BEFORE INSERT ON audit WHEN NEW.kind IN ({ks}) AND (
 typeof(NEW.data)<>'text' OR length(CAST(NEW.data AS BLOB))>4096 OR {header}
 OR {unique_counts}
 OR NOT EXISTS(SELECT 1 FROM managed_phase_operations o WHERE o.operation_id=json_extract(NEW.data,'$.private_operation_ref')
 AND o.workflow_id=json_extract(NEW.data,'$.workflow_id') AND o.project_id IS NEW.project_id AND o.goal_id IS NEW.goal_id AND o.task_id IS NEW.task_id
 AND o.project_id=json_extract(NEW.data,'$.project_id') AND o.goal_id=json_extract(NEW.data,'$.goal_id') AND o.task_id=json_extract(NEW.data,'$.task_id')
 AND o.marker_digest=json_extract(NEW.data,'$.original_marker_frame_sha256') AND o.workflow_generation=json_extract(NEW.data,'$.generation') AND o.attempt_index=json_extract(NEW.data,'$.attempt_index') AND o.phase=json_extract(NEW.data,'$.phase') AND o.context_version=json_extract(NEW.data,'$.context_version')
 AND json_extract(NEW.data,'$.workflow_version_after')=json_extract(NEW.data,'$.workflow_version_before')+1
 AND json_extract(NEW.data,'$.workflow_version_after')-o.marker_workflow_version BETWEEN 1 AND 256
 AND json_extract(NEW.data,'$.workflow_version_after')-o.marker_workflow_version=1+(SELECT count(*) FROM audit a WHERE a.kind IN ({ks}) AND json_extract(a.data,'$.private_operation_ref')=o.operation_id)
 AND ((o.phase_open=1 AND NEW.kind<>'rrx.private.workflow.phase_closed') OR (o.phase_open=0 AND NEW.kind='rrx.private.workflow.phase_closed'))
 AND ((json_extract(NEW.data,'$.workflow_version_before')=o.marker_workflow_version AND json_extract(NEW.data,'$.prior_ledger_digest')=o.marker_digest AND json_extract(NEW.data,'$.workflow_body_sha256_before')=o.marker_workflow_sha256)
 OR EXISTS(SELECT 1 FROM audit a WHERE a.kind IN ({ks}) AND json_extract(a.data,'$.private_operation_ref')=o.operation_id AND json_extract(a.data,'$.workflow_version_after')=json_extract(NEW.data,'$.workflow_version_before') AND json_extract(a.data,'$.workflow_body_sha256_after')=json_extract(NEW.data,'$.workflow_body_sha256_before')))
 AND EXISTS(SELECT 1 FROM records r WHERE r.id=o.workflow_id AND r.kind='workflow' AND r.version=json_extract(NEW.data,'$.workflow_version_after')))
 OR (NEW.kind='rrx.private.workflow.session_bound' AND EXISTS(SELECT 1 FROM audit a WHERE a.kind IN ({ks}) AND json_extract(a.data,'$.private_operation_ref')=json_extract(NEW.data,'$.private_operation_ref')))
 OR (NEW.kind='rrx.private.workflow.gate_observed' AND NOT EXISTS(SELECT 1 FROM audit a WHERE a.kind='rrx.private.workflow.gate_claim' AND json_extract(a.data,'$.private_operation_ref')=json_extract(NEW.data,'$.private_operation_ref') AND json_extract(a.data,'$.workflow_version_after')=json_extract(NEW.data,'$.workflow_version_before')))
 ) BEGIN SELECT RAISE(ABORT,'reserved Workflow ledger shape/allowance differs'); END;"))?;
    Ok(())
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(in crate::state) struct MigrationInventory {
    pub rows: usize,
    pub source_bytes: usize,
    pub pages: usize,
    pub malformed_sessions: usize,
    pub held_workflows: usize,
}

/// Complete indexed inventory, bounded pages. The full length is charged BEFORE
/// deciding whether to copy/decode a body. A refused transaction publishes nothing.
pub(in crate::state) fn hold_existing_workflows(
    tx: &Transaction<'_>,
    manager: &PrivatePermitManager,
) -> Result<MigrationInventory> {
    migrate_inventory(tx, manager, MIGRATION_ROWS, MIGRATION_BYTES)
}
pub(super) fn migrate_inventory(
    tx: &Transaction<'_>,
    manager: &PrivatePermitManager,
    max_rows: usize,
    max_bytes: usize,
) -> Result<MigrationInventory> {
    let mut inventory = MigrationInventory::default();
    let mut cursor = None::<i64>;
    loop {
        let mut q=tx.prepare("SELECT rowid,id,kind,project_id,goal_id,task_id,version,length(CAST(body AS BLOB)) FROM records WHERE (?1 IS NULL OR rowid>?1) AND kind IN ('workflow','session') ORDER BY rowid LIMIT 128")?;
        let rows = q
            .query_map([cursor], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, i64>(6)?,
                    r.get::<_, usize>(7)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(q);
        if rows.is_empty() {
            break;
        }
        inventory.pages += 1;
        for (rowid, id, kind, p, g, t, version, length) in rows {
            inventory.rows = inventory
                .rows
                .checked_add(1)
                .context("Binding10 inventory rows overflow")?;
            inventory.source_bytes = inventory
                .source_bytes
                .checked_add(length)
                .context("Binding10 inventory bytes overflow")?;
            ensure!(
                inventory.rows <= max_rows && inventory.source_bytes <= max_bytes,
                "Binding10 complete legacy inventory bound exceeded"
            );
            cursor = Some(rowid);
            let cap = if kind == "session" {
                SESSION_BYTES
            } else {
                BODY_BYTES
            };
            let raw: Option<String> = if length <= cap {
                Some(
                    tx.query_row("SELECT body FROM records WHERE rowid=?1", [rowid], |r| {
                        r.get(0)
                    })?,
                )
            } else {
                None
            };
            if kind == "session" {
                let projected =
                    projection(&id, &p, g.as_deref(), t.as_deref(), version, raw.as_deref());
                let v: serde_json::Value = serde_json::from_str(&projected)?;
                inventory.malformed_sessions += usize::from(v["malformed"] == 1);
                tx.execute("INSERT INTO scoped_session_identities(session_id,project_id,goal_id,task_id,record_version,provider,native_ref,malformed,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![id,p,g,t,version,v["provider"].as_str(),v["native_ref"].as_str(),v["malformed"].as_i64(),projected])?;
            } else {
                let raw = raw.context("legacy Workflow complete body over bound")?;
                let envelope = Body::<Record>::decode(raw, BODY_BYTES)?;
                let record = envelope.parsed();
                ensure!(
                    record.kind == RecordKind::Workflow
                        && record.id.to_string() == id
                        && record.scope.project_id.to_string() == p
                        && record.scope.goal_id.map(|v| v.to_string()) == g
                        && record.scope.task_id.map(|v| v.to_string()) == t
                        && i64::try_from(record.version).ok() == Some(version),
                    "legacy Workflow body/index mismatch"
                );
                let workflow = Body::<crate::workflow::WorkflowSnapshot>::decode(
                    serde_json::to_string(&record.data)?,
                    BODY_BYTES,
                )?;
                if workflow.parsed().active.is_some() || !workflow.parsed().finished {
                    let g = g.context("legacy Workflow Goal missing")?;
                    let t = t.context("legacy Workflow Task missing")?;
                    let origin = "legacy-schema9";
                    let body=json!({"workflow_id":id,"project_id":p,"goal_id":g,"task_id":t,"owner_epoch":0,"origin":origin,"profile_digest":null,"contract_state":"legacy_held","version":1}).to_string();
                    let row = vec![
                        SqlValue::Text(id.clone()),
                        SqlValue::Text(p.clone()),
                        SqlValue::Text(g.clone()),
                        SqlValue::Text(t.clone()),
                        SqlValue::Integer(0),
                        SqlValue::Text(origin.into()),
                        SqlValue::Null,
                        SqlValue::Text("legacy_held".into()),
                        SqlValue::Integer(1),
                        SqlValue::Text(body.clone()),
                    ];
                    manager.with_exact_permit(vec![ExactRowMutation::new("workflow_native_contracts","INSERT",None,Some(row))?],||{
                        tx.execute("INSERT INTO workflow_native_contracts(workflow_id,project_id,goal_id,task_id,owner_epoch,origin,profile_digest,contract_state,version,body) VALUES(?1,?2,?3,?4,0,?5,NULL,'legacy_held',1,?6)",params![id,p,g,t,origin,body])?;
                        manager.ensure_consumed()
                    })?;
                    inventory.held_workflows += 1;
                }
            }
        }
    }
    // Instrumentation is returned to the caller/tests, not a new authority row.
    ensure!(SCHEMA_VERSION == 10, "Binding10 installer contract differs");
    Ok(inventory)
}
