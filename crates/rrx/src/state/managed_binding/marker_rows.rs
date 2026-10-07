//! Exact allocation rows, materialized before the marker writer lock.
//! This plan is not OriginalMarker, registration, input or launch authority.
use super::{
    canonical::{Body, WORKFLOW_DOMAIN, encode},
    marker_plan::ManagedMarkerPlan,
    permits::{ExactRowMutation, columns},
};
use crate::{domain::*, execution::phase::NativeAllocation};
use anyhow::{Context, Result, ensure};
use rusqlite::{Transaction, params_from_iter, types::Value as SqlValue};
use serde_json::json;

pub(super) const LINK_RESERVE_BYTES: u64 = 256 * 4096;
pub(super) const WORKFLOW_BYTES: u64 = 128 * 1024 * 1024;

/// Only this module can construct a row. Table/column names are compiled;
/// ordinary values always go through SQLite parameters, never SQL interpolation.
struct Insert {
    table: &'static str,
    values: Vec<SqlValue>,
}
impl Insert {
    fn new(table: &'static str, values: Vec<SqlValue>) -> Result<Self> {
        ensure!(
            matches!(
                table,
                "managed_phase_operations"
                    | "managed_marker_bodies"
                    | "managed_phase_owners"
                    | "managed_phase_inputs"
                    | "managed_phase_readiness"
            ) && columns(table).is_some_and(|c| c.len() == values.len()),
            "incomplete marker allocation image"
        );
        Ok(Self { table, values })
    }
    fn mutation(&self) -> Result<ExactRowMutation> {
        ExactRowMutation::new(self.table, "INSERT", None, Some(self.values.clone()))
    }
    fn write(&self, tx: &Transaction<'_>) -> Result<()> {
        let names = columns(self.table).context("marker table contract absent")?;
        let arguments = (1..=names.len())
            .map(|i| format!("?{i}"))
            .collect::<Vec<_>>();
        ensure!(
            tx.execute(
                &format!(
                    "INSERT INTO {} ({}) VALUES ({})",
                    self.table,
                    names.join(","),
                    arguments.join(",")
                ),
                params_from_iter(&self.values),
            )? == 1,
            "marker allocation was not inserted"
        );
        Ok(())
    }
    fn validate_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        let names = columns(self.table).context("marker table contract absent")?;
        let predicates = names
            .iter()
            .enumerate()
            .map(|(index, name)| format!("{name} IS ?{}", index + 1))
            .collect::<Vec<_>>();
        let exact: bool = tx.query_row(
            &format!(
                "SELECT EXISTS(SELECT 1 FROM {} WHERE {})",
                self.table,
                predicates.join(" AND ")
            ),
            params_from_iter(&self.values),
            |r| r.get(0),
        )?;
        ensure!(exact, "original marker allocation image differs");
        Ok(())
    }
}

/// Owned exact full column images. There is no public table/action selector,
/// caller-supplied permit, positive callback or row-to-Native-proof conversion.
pub(super) struct MarkerRows {
    rows: Vec<Insert>,
    bytes: u64,
}
fn text(value: impl ToString) -> SqlValue {
    SqlValue::Text(value.to_string())
}
fn number(value: u64) -> Result<SqlValue> {
    Ok(SqlValue::Integer(i64::try_from(value)?))
}
fn body(value: &serde_json::Value, bound: usize) -> Result<String> {
    String::from_utf8(encode(value, bound)?).context("marker body is not UTF-8")
}

impl MarkerRows {
    pub(super) fn validate_open_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        // Native owner/readiness facts may advance separately. The original
        // open operation and preparation template stay exact and immutable.
        for table in ["managed_phase_operations", "managed_phase_inputs"] {
            self.rows
                .iter()
                .find(|row| row.table == table)
                .context("original open marker image absent")?
                .validate_tx(tx)?;
        }
        Ok(())
    }
    pub(super) fn plan(marker: &ManagedMarkerPlan, allocation: &NativeAllocation) -> Result<Self> {
        let f = allocation.facts();
        // The owner key is the genuine factory's immutable allocated pair key.
        // Owner/input identities live in distinct keyed tables; neither key is
        // constructed by copying a Session/SQL body or a provider name.
        let owner_id = f.pair_id;
        ensure!(
            marker.operation == f.operation_id
                && marker.pair == f.pair_id
                && marker.allocated_session == f.session_id
                && marker.origin == f.origin_id
                && marker.unit.parsed().id == f.unit_id
                && marker.scope() == *f.scope,
            "marker rows differ from original selected allocation"
        );
        let project = f.scope.project_id.to_string();
        let goal = f.scope.goal_id.context("marker Goal absent")?.to_string();
        let task = f.scope.task_id.context("marker Task absent")?.to_string();
        let workflow = marker.workflow_after.parsed();
        let snapshot: crate::workflow::WorkflowSnapshot =
            serde_json::from_value(workflow.data.clone())?;
        let index = snapshot.active.context("marker attempt absent")?;
        let attempt = snapshot
            .history
            .get(index)
            .context("marker attempt outside history")?;
        let actor = match attempt.phase.actor() {
            crate::workflow::Actor::Executor => "executor",
            crate::workflow::Actor::Reviewer => "reviewer",
            crate::workflow::Actor::EvidencePort => anyhow::bail!("evidence has no native marker"),
        };
        let role = serde_json::to_value(f.role)?;
        let role = role.as_str().context("marker role is not text")?;
        let path = f.path.to_str().context("marker worktree is not UTF-8")?;
        let context = marker.context().0;
        let p_digest = marker.before.project.digest(WORKFLOW_DOMAIN);
        let g_digest = marker.before.goal.digest(WORKFLOW_DOMAIN);
        let t_digest = marker.task_after.digest(WORKFLOW_DOMAIN);
        let w_digest = marker.workflow_after.digest(WORKFLOW_DOMAIN);
        let original_frame = serde_json::from_slice::<serde_json::Value>(&marker.frame)?;
        let operation_body = body(
            &json!({
                "original_frame": original_frame,
                "owner_id": owner_id,
                "audit_reserve_bytes": LINK_RESERVE_BYTES,
                "workflow_budget_bytes": WORKFLOW_BYTES,
                "phase_open": true,
                "version": 1,
            }),
            4 * 1024 * 1024,
        )?;
        let owner_body = body(
            &json!({
                "owner_id": owner_id, "operation_id": f.operation_id,
                "scope": f.scope, "unit_id": f.unit_id, "owner_epoch": f.epoch,
                "execution_generation": f.generation, "allocated_session_id": f.session_id,
                "provider": f.provider, "alias": f.alias, "role": f.role,
                "worktree": f.path, "origin": f.origin_id,
                "native_invocation_id": null, "validated": false, "version": 1,
            }),
            2 * 1024 * 1024,
        )?;
        let template: serde_json::Value = serde_json::from_slice(f.input_bytes)?;
        let input_body = body(
            &json!({
                "pair_id": f.pair_id, "owner_id": owner_id, "operation_id": f.operation_id,
                "scope": f.scope, "context_version": context.version,
                "context_digest": marker.context_digest, "profile_digest": f.profile_digest,
                "prepared_input_sha256": marker.input_digest, "template": template,
                "payload_bytes": f.input.payload.len(), "version": 1,
            }),
            2 * 1024 * 1024,
        )?;
        let readiness_body = body(
            &json!({
                "operation_id": f.operation_id, "origin": f.origin_id, "owner_epoch": f.epoch,
                "state": "allocated", "start_ended": false, "known_terminal": false,
                "parking_version": null, "version": 1,
            }),
            4096,
        )?;
        let rows = vec![
            Insert::new(
                "managed_phase_operations",
                vec![
                    text(f.operation_id),
                    text(workflow.id),
                    text(&project),
                    text(&goal),
                    text(&task),
                    number(snapshot.generation)?,
                    number(u64::try_from(index)?)?,
                    number(f.epoch)?,
                    text(f.unit_id),
                    number(f.generation)?,
                    text(f.session_id),
                    text(owner_id),
                    text(f.pair_id),
                    number(marker.before.project.parsed().version)?,
                    number(marker.before.goal.parsed().version)?,
                    number(marker.task_after.parsed().version)?,
                    number(workflow.version)?,
                    text(&p_digest),
                    text(&g_digest),
                    text(&t_digest),
                    text(&w_digest),
                    text(&marker.frame_digest),
                    text(f.origin_id),
                    text(attempt.phase.key()),
                    text(actor),
                    text(f.provider),
                    text(f.alias),
                    text(role),
                    number(context.version)?,
                    number(1)?,
                    number(1)?,
                    text(operation_body),
                ],
            )?,
            Insert::new(
                "managed_marker_bodies",
                vec![
                    text(f.operation_id),
                    text(workflow.id),
                    number(workflow.version)?,
                    text(w_digest),
                    text(marker.workflow_after.raw()),
                ],
            )?,
            Insert::new(
                "managed_phase_owners",
                vec![
                    text(owner_id),
                    text(f.operation_id),
                    text(&project),
                    text(&goal),
                    text(&task),
                    text(f.unit_id),
                    number(f.epoch)?,
                    number(f.generation)?,
                    text(f.session_id),
                    text(f.provider),
                    text(f.alias),
                    text(role),
                    text(path),
                    text(f.origin_id),
                    SqlValue::Null,
                    number(0)?,
                    number(1)?,
                    text(owner_body),
                ],
            )?,
            Insert::new(
                "managed_phase_inputs",
                vec![
                    text(f.pair_id),
                    text(owner_id),
                    text(f.operation_id),
                    text(&project),
                    text(&goal),
                    text(&task),
                    number(context.version)?,
                    text(&marker.context_digest),
                    text(&f.input.revision),
                    text(&marker.input_digest),
                    number(u64::try_from(f.input.payload.len())?)?,
                    text(f.profile_digest),
                    number(1)?,
                    text(input_body),
                ],
            )?,
            Insert::new(
                "managed_phase_readiness",
                vec![
                    text(f.operation_id),
                    text(f.origin_id),
                    number(f.epoch)?,
                    text("allocated"),
                    number(0)?,
                    number(0)?,
                    SqlValue::Null,
                    number(1)?,
                    text(readiness_body),
                ],
            )?,
        ];
        let mut bytes = 0u64;
        for row in &rows {
            for value in &row.values {
                let cost = match value {
                    SqlValue::Text(s) => s.len(),
                    SqlValue::Integer(_) => 8,
                    SqlValue::Null => 0,
                    _ => anyhow::bail!("marker image has unsupported value"),
                };
                bytes = bytes
                    .checked_add(u64::try_from(cost)?)
                    .context("marker bytes overflow")?;
            }
        }
        Ok(Self { rows, bytes })
    }
    pub(super) fn exact_mutations(&self) -> Result<Vec<ExactRowMutation>> {
        self.rows.iter().map(Insert::mutation).collect()
    }
    pub(super) fn durable_bytes(&self) -> u64 {
        self.bytes
    }
    pub(super) fn write_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        // Operation first; allocated owner/input FKs are deferred, whereas
        // their identity triggers validate the already inserted operation.
        for row in &self.rows {
            row.write(tx)?;
        }
        Ok(())
    }
    /// Exact planned rows, including every metadata column and encoded body.
    /// Neither a successful lookup nor these images reconstructs authority.
    pub(super) fn validate_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        for row in &self.rows {
            row.validate_tx(tx)?;
        }
        Ok(())
    }
}

/// Complete encoded Record column image for the same private permit batch.
pub(in crate::state) fn record_image(record: &Record, raw: &str) -> Result<Vec<SqlValue>> {
    Ok(vec![
        text(record.id),
        text(record.kind.key()),
        text(record.scope.project_id),
        record.scope.goal_id.map(text).unwrap_or(SqlValue::Null),
        record.scope.task_id.map(text).unwrap_or(SqlValue::Null),
        number(record.version)?,
        text(raw),
    ])
}

/// Marker-specific Record mutation, never a general binding/access-mode selector.
pub(super) fn workflow_mutation(plan: &ManagedMarkerPlan) -> Result<ExactRowMutation> {
    let before: &Body<Record> = plan
        .before
        .workflow
        .as_ref()
        .context("marker Workflow absent")?;
    ExactRowMutation::new(
        "records",
        "UPDATE",
        Some(record_image(before.parsed(), before.raw())?),
        Some(record_image(
            plan.workflow_after.parsed(),
            plan.workflow_after.raw(),
        )?),
    )
}
