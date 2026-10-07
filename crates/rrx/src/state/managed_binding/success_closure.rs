//! SC successful `phase_closed`: ONE Immediate that publishes the artifact,
//! closes the Unit's finalization, advances Task, Workflow and Context,
//! closes the operation and re-anchors the Driver marker-free. Plans retain
//! inputs and one digest; images are re-materialized per turn from the SAME
//! inputs and must match, otherwise Held.
use super::super::{append_event, context_owner, put_context_tx};
use super::{
    canonical::{BODY_BYTES, Body},
    closure::operation_delta,
    gate::{
        GateObservedAcknowledgment, SuccessConfirmation, SuccessWrite, header, link_data,
        selected_database, typed, validate_link_head_tx,
    },
    marker_rows::{WORKFLOW_BYTES, record_image},
    permits::ExactRowMutation,
    publication::charged_scope_bytes,
    success::{SettledCurrency, SettledPhase, plan_settled_currency, validate_settled_tx},
};
use crate::{
    domain::{ContextVersion, Record, Task},
    execution::{
        ArtifactState, ExecutionUnit, ResultArtifact, RuntimeOwner, results::WorkflowPublication,
    },
    state::{Store, runtime::driver::DriverClosureAdvance},
    workflow::{succeed_attempt, succeed_context, validate_context, validate_evidence},
};
use anyhow::{Context, Result, ensure};
use rusqlite::{
    Connection, OptionalExtension, Transaction, TransactionBehavior, params, params_from_iter,
    types::Value as SqlValue,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Arc;

const KIND: &str = "rrx.private.workflow.phase_closed";

/// Query-only plan of the SAME Passed observation; retains inputs only.
pub(crate) struct SuccessClosurePlan {
    currency: SettledCurrency,
    observed: Arc<GateObservedAcknowledgment>,
    publication: WorkflowPublication,
    context: ContextVersion,
    context_raw: String,
    at: i64,
    /// The latest cleanup observation (rowid, body) when planned: the read
    /// overlay the generic Driver snapshot applies to the decoded Unit.
    cleanup: Option<(i64, String)>,
    digest: String,
}
/// One turn's images; dropped after the Store guard.
pub(crate) struct SuccessClosureMaterial {
    plan: Arc<SuccessClosurePlan>,
    task: Task,
    task_raw: String,
    workflow: Body<Record>,
    unit: ExecutionUnit,
    unit_raw: String,
    artifact: ResultArtifact,
    artifact_raw: String,
    operation: Vec<SqlValue>,
    driver: Arc<DriverClosureAdvance>,
    data: String,
}
/// Known closure; retains the Driver re-anchor for publication retries.
pub(crate) struct SuccessClosureAcknowledgment {
    plan: Arc<SuccessClosurePlan>,
    driver: Arc<DriverClosureAdvance>,
}
impl SuccessClosureAcknowledgment {
    pub(crate) fn settled(&self) -> &Arc<SettledPhase> {
        self.plan.currency.settled()
    }
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_cleanup(c: &Connection, unit: crate::execution::UnitId) -> Result<Option<(i64, String)>> {
    Ok(c.query_row(
        "SELECT rowid,body FROM cleanup_observations WHERE unit_id=?1 ORDER BY rowid DESC LIMIT 1",
        [unit.to_string()],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()?)
}

/// Deterministic images from the SAME retained inputs.
fn images(plan: &Arc<SuccessClosurePlan>) -> Result<SuccessClosureMaterial> {
    let settled = plan.currency.settled();
    let marker = settled.marker();
    let original = marker.original_plan();
    let (evidence, receipt) = plan
        .observed
        .passed()
        .context("success closure requires a Passed observation")?;
    let before = plan.observed.workflow_after();
    let mut workflow = typed(before.parsed())?;
    let index = workflow
        .active
        .context("success closure active attempt absent")?;
    let attempt_before = workflow.history[index].clone();
    let phase = attempt_before.phase;
    let source = attempt_before
        .observations
        .last()
        .context("success closure observation absent")?
        .sources
        .clone();
    let (task_before, task_before_raw) = original.task_after();
    let mut task = task_before.clone();
    validate_evidence(evidence, &task_before.scope(), &source, &attempt_before)?;
    let at = plan.at;
    let next = succeed_attempt(
        &mut task,
        &mut workflow,
        index,
        evidence.clone(),
        source,
        || at,
    );
    ensure!(
        plan.context
            .source_hashes
            .get("workflow:phase")
            .map(String::as_str)
            == Some(next.unwrap_or(phase).key()),
        "success Context is not for the next phase"
    );
    succeed_context(&mut task, &mut workflow, phase, next, &plan.context);
    task.version = task
        .version
        .checked_add(1)
        .context("success Task version exhausted")?;
    task.updated_at = at;
    let task_raw = serde_json::to_string(&task)?;
    ensure!(task_raw.len() <= 1024 * 1024, "success Task exceeds bound");
    let mut record = before.parsed().clone();
    record.version = record
        .version
        .checked_add(1)
        .filter(|v| *v <= i64::MAX as u64)
        .context("success Workflow version exhausted")?;
    record.updated_at = at;
    record.data = serde_json::to_value(&workflow)?;
    crate::workflow::validate_transition(&task, &record, Some(before.parsed()))?;
    validate_context(&task, &record, &plan.context)?;
    let workflow_body = Body::decode(serde_json::to_string(&record)?, BODY_BYTES)?;
    let artifact = plan.publication.artifact();
    ensure!(
        artifact.state == ArtifactState::Ready,
        "success closure artifact is not Ready"
    );
    let terminal = settled.settlement().unit();
    let mut unit = terminal.clone();
    unit.result_finalization_open = false;
    unit.artifact_id = Some(artifact.id);
    unit.version = unit
        .version
        .checked_add(1)
        .context("success Unit version exhausted")?;
    unit.updated_at = at;
    let unit_raw = serde_json::to_string(&unit)?;
    ensure!(unit_raw.len() <= 16 * 1024, "success Unit exceeds bound");
    crate::state::execution::publish_result_core(
        &unit,
        artifact,
        &task,
        &record,
        before.parsed(),
        &plan.context,
    )?;
    let mut published = artifact.clone();
    published.state = ArtifactState::Published;
    published.version = published
        .version
        .checked_add(1)
        .context("success artifact version exhausted")?;
    let artifact_raw = serde_json::to_string(&published)?;
    let operation = operation_delta(marker.original_operation_image()?)?;
    // The generic Driver snapshot decodes the Unit with the cleanup overlay.
    let mut decoded = unit.clone();
    if let Some((_, body)) = &plan.cleanup {
        let observation: crate::execution::CleanupObservation = serde_json::from_str(body)?;
        decoded.cleanup = observation.outcome;
    }
    let driver = Arc::new(marker.driver_advance().plan_success_closure(
        &task,
        &record,
        &plan.context,
        &decoded,
    )?);
    let header = header(&plan.currency, &workflow_body, KIND, index, at)?;
    let data = link_data(
        header,
        json!({"closure":"success","proof_source":"owned_settlement_gate_passed",
            "native_receipt_ref":settled.settlement().receipt().id,"gate_receipt_ref":receipt.id,
            "artifact_id":artifact.id,"artifact_revision":artifact.revision,
            "manifest_sha256":artifact.manifest_sha256,
            "unit_version_before":terminal.version,"unit_version_after":unit.version,
            "task_version_before":task_before.version,"task_version_after":task.version,
            "next_context_version":plan.context.version,"next_phase":next.map(|p| p.key()),
            "operation_version_after":2,"driver_version_after":driver.version_after()}),
    )?;
    let _ = task_before_raw;
    Ok(SuccessClosureMaterial {
        plan: plan.clone(),
        task,
        task_raw,
        workflow: workflow_body,
        unit,
        unit_raw,
        artifact: published,
        artifact_raw,
        operation,
        driver,
        data,
    })
}

fn material_digest(m: &SuccessClosureMaterial) -> Result<String> {
    let SqlValue::Text(operation) = &m.operation[31] else {
        anyhow::bail!("success operation postimage absent")
    };
    Ok(sha(serde_json::to_string(&json!([
        m.task_raw,
        m.workflow.raw(),
        m.unit_raw,
        m.artifact_raw,
        operation,
        m.plan.context_raw,
        m.data,
    ]))?
    .as_bytes()))
}

fn unit_row_tx(c: &Connection, unit: &ExecutionUnit, raw: &str) -> Result<bool> {
    let kind = serde_json::to_value(unit.kind)?;
    Ok(c.query_row(
        "SELECT EXISTS(SELECT 1 FROM execution_units WHERE id=?1 AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND kind=?5 AND generation=?6 AND owner_epoch=?7 AND version=?8 AND native_effects_open=?9 AND result_finalization_open=?10 AND worktree=?11 AND branch IS ?12 AND body=?13)",
        params![unit.id.to_string(),unit.scope.project_id.to_string(),unit.scope.goal_id.map(|v|v.to_string()),unit.scope.task_id.map(|v|v.to_string()),kind.as_str().context("Unit kind")?,unit.generation,unit.owner_epoch,unit.version,unit.native_effects_open,unit.result_finalization_open,unit.worktree.to_str().context("Unit path")?,unit.branch,raw],
        |r| r.get(0),
    )?)
}
fn artifact_row_tx(c: &Connection, artifact: &ResultArtifact, raw: &str) -> Result<bool> {
    let state = serde_json::to_value(artifact.state)?;
    Ok(c.query_row(
        "SELECT EXISTS(SELECT 1 FROM result_artifacts WHERE id=?1 AND unit_id=?2 AND state=?3 AND version=?4 AND body=?5)",
        params![artifact.id.to_string(),artifact.unit_id.to_string(),state.as_str().context("artifact state")?,artifact.version,raw],
        |r| r.get(0),
    )?)
}
fn task_row_tx(c: &Connection, task: &Task, raw: &str) -> Result<bool> {
    Ok(c.query_row(
        "SELECT EXISTS(SELECT 1 FROM tasks WHERE id=?1 AND version=?2 AND body=?3)",
        params![task.id.to_string(), task.version, raw],
        |r| r.get(0),
    )?)
}
fn context_row_tx(c: &Connection, context: &ContextVersion) -> Result<Option<String>> {
    Ok(c.query_row(
        "SELECT body FROM context_versions WHERE project_id=?1 AND owner=?2 AND version=?3",
        params![
            context.scope.project_id.to_string(),
            context_owner(&context.scope)?,
            context.version
        ],
        |r| r.get(0),
    )
    .optional()?)
}

/// Every closure conjunct before the first write; a refusal is a Conflict.
fn checked_close_tx(tx: &Transaction<'_>, m: &SuccessClosureMaterial) -> Result<i64> {
    let plan = &m.plan;
    let settled = plan.currency.settled();
    let marker = settled.marker();
    validate_settled_tx(tx, &plan.currency)?;
    ensure!(
        plan.currency.current().workflow_raw() == plan.observed.workflow_after().raw()
            && plan.currency.current().link_count() == plan.observed.link_count()
            && plan.currency.current().link_count() < 256,
        "success closure endpoint is not the SAME Passed observation"
    );
    let artifact = plan.publication.artifact();
    ensure!(
        artifact_row_tx(tx, artifact, &serde_json::to_string(artifact)?)?,
        "success closure Ready artifact changed"
    );
    let (task_before, task_before_raw) = marker.original_plan().task_after();
    ensure!(
        task_row_tx(tx, task_before, task_before_raw)?
            && context_row_tx(tx, &plan.context)?.is_none(),
        "success closure Task or next Context changed"
    );
    ensure!(
        read_cleanup(tx, m.unit.id)? == plan.cleanup,
        "success closure cleanup overlay changed"
    );
    let unsettled: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM managed_effects WHERE unit_id=?1 AND state NOT IN ('confirmed','resolved') LIMIT 1)",
        [m.unit.id.to_string()],
        |r| r.get(0),
    )?;
    ensure!(
        !unsettled,
        "success closure requires every Unit effect settled"
    );
    m.driver.validate_current_tx(tx)?;
    ensure!(
        m.driver.association_live(),
        "success closure Driver no longer owns a live worker"
    );
    let growth = (m.workflow.raw().len() as u64)
        .saturating_sub(plan.observed.workflow_after().raw().len() as u64)
        .saturating_add((m.task_raw.len() as u64).saturating_sub(task_before_raw.len() as u64))
        .saturating_add(plan.context_raw.len() as u64)
        .saturating_add(m.data.len() as u64);
    ensure!(
        charged_scope_bytes(tx, &marker.scope())?
            .checked_add(growth)
            .is_some_and(|v| v <= WORKFLOW_BYTES),
        "success closure scope budget exhausted"
    );
    next_audit_sequence(tx)
}
fn next_audit_sequence(tx: &Transaction<'_>) -> Result<i64> {
    let sequence: i64 = tx.query_row(
        "SELECT COALESCE((SELECT seq FROM sqlite_sequence WHERE name='audit'),0)",
        [],
        |r| r.get(0),
    )?;
    sequence.checked_add(1).context("audit identity exhausted")
}

/// Two-branch confirmation of the SAME material.
fn confirm_close_tx(tx: &Transaction<'_>, m: &SuccessClosureMaterial) -> Result<Option<bool>> {
    let plan = &m.plan;
    let settled = plan.currency.settled();
    let marker = settled.marker();
    let original = marker.original_plan();
    settled.settlement().validate_terminal_images_tx(tx)?;
    let workflow_raw: Option<String> = tx.query_row(
        "SELECT CASE WHEN length(CAST(body AS BLOB))<=?2 THEN body END FROM records WHERE id=?1 AND kind='workflow'",
        params![m.workflow.parsed().id.to_string(), BODY_BYTES],
        |r| r.get(0),
    )?;
    let workflow_raw = workflow_raw.context("success confirmation Workflow over bound")?;
    if workflow_raw == m.workflow.raw() {
        // Postimage branch: every W1-W8 row and the head relations.
        let count = plan.observed.link_count() + 1;
        let closed = unit_row_tx(tx, &m.unit, &m.unit_raw)?
            && artifact_row_tx(tx, &m.artifact, &m.artifact_raw)?
            && task_row_tx(tx, &m.task, &m.task_raw)?
            && context_row_tx(tx, &plan.context)?.as_deref() == Some(plan.context_raw.as_str())
            && validate_link_head_tx(tx, settled, count, KIND, plan.at, &m.data)?;
        if !closed {
            return Ok(None);
        }
        marker.validate_closed_tx(tx, &m.operation)?;
        m.driver.validate_closed_tx(tx)?;
        original.before.validate_success_projection(
            tx,
            &m.task,
            &m.task_raw,
            (m.workflow.parsed(), m.workflow.raw()),
            (&plan.context, &plan.context_raw),
        )?;
        return Ok(Some(true));
    }
    if workflow_raw == plan.observed.workflow_after().raw() {
        // Preimage branch: the gate_observed endpoint, link absent.
        validate_settled_tx(tx, &plan.currency)?;
        let artifact = plan.publication.artifact();
        let (task_before, task_before_raw) = original.task_after();
        ensure!(
            artifact_row_tx(tx, artifact, &serde_json::to_string(artifact)?)?
                && task_row_tx(tx, task_before, task_before_raw)?
                && context_row_tx(tx, &plan.context)?.is_none(),
            "success closure preimage rows changed"
        );
        m.driver.validate_current_tx(tx)?;
        return Ok(Some(false));
    }
    Ok(None)
}

impl Store {
    /// Query-only, outside every lock. `context` is the genuine Context Pack
    /// for the next phase (or the phase) at version latest + 1.
    pub(crate) fn plan_phase_success(
        owner: &RuntimeOwner,
        observed: &Arc<GateObservedAcknowledgment>,
        publication: WorkflowPublication,
        context: ContextVersion,
        at: i64,
    ) -> Result<Arc<SuccessClosurePlan>> {
        let settled = observed.settled();
        let currency = plan_settled_currency(owner, settled)?;
        ensure!(
            currency.current().workflow_raw() == observed.workflow_after().raw()
                && currency.current().link_count() == observed.link_count(),
            "success closure is not at the SAME Passed observation endpoint"
        );
        let unit = settled.settlement().unit().id;
        let cleanup = super::snapshot::snapshot(owner, |tx| read_cleanup(tx, unit))?;
        let context_raw = serde_json::to_string(&context)?;
        ensure!(
            context_raw.len() <= BODY_BYTES,
            "success Context exceeds bound"
        );
        let mut plan = SuccessClosurePlan {
            currency,
            observed: observed.clone(),
            publication,
            context,
            context_raw,
            at,
            cleanup,
            digest: String::new(),
        };
        // The digest of the first materialization binds every later turn.
        let staged = Arc::new(plan);
        let digest = material_digest(&images(&staged)?)?;
        plan = Arc::into_inner(staged).context("success closure plan shared while planned")?;
        plan.digest = digest;
        Ok(Arc::new(plan))
    }
    pub(crate) fn materialize_phase_success(
        plan: &Arc<SuccessClosurePlan>,
    ) -> Result<SuccessClosureMaterial> {
        let material = images(plan)?;
        ensure!(
            material_digest(&material)? == plan.digest,
            "SAME success closure material digest differs; Held"
        );
        Ok(material)
    }
    /// Synchronous. Caller holds `control_admission` (a SuccessAdmission).
    pub(crate) fn close_phase_success(
        &mut self,
        m: &SuccessClosureMaterial,
    ) -> Result<SuccessWrite<SuccessClosureAcknowledgment>> {
        #[cfg(test)]
        let fault = super::fault::take(
            m.plan.currency.settled().marker().scope().task_id,
            super::fault::CLOSURE,
        );
        #[cfg(test)]
        super::fault::before(fault)?;
        let result = self.close_phase_success_inner(m)?;
        #[cfg(test)]
        let result = super::fault::after(fault, matches!(result, SuccessWrite::Known(_)), result)?;
        Ok(result)
    }
    fn close_phase_success_inner(
        &mut self,
        m: &SuccessClosureMaterial,
    ) -> Result<SuccessWrite<SuccessClosureAcknowledgment>> {
        let settled = m.plan.currency.settled();
        if let Err(cause) = selected_database(&self.connection, settled) {
            return Ok(SuccessWrite::Conflict(cause));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let _planned = match checked_close_tx(&tx, m) {
            Ok(sequence) => sequence,
            Err(cause) => {
                drop(tx);
                return Ok(SuccessWrite::Conflict(cause));
            }
        };
        let marker = settled.marker();
        let terminal = settled.settlement().unit();
        let terminal_raw = settled.settlement().terminal_unit_raw();
        // W1: exact 13-column Unit CAS from the sealed terminal postimage.
        let kind = serde_json::to_value(terminal.kind)?;
        ensure!(tx.execute("UPDATE execution_units SET version=?1,result_finalization_open=?2,body=?3 WHERE id=?4 AND project_id=?5 AND goal_id=?6 AND task_id=?7 AND kind=?8 AND generation=?9 AND owner_epoch=?10 AND version=?11 AND native_effects_open=?12 AND result_finalization_open=?13 AND worktree=?14 AND branch IS ?15 AND body=?16",
            params![m.unit.version,m.unit.result_finalization_open,m.unit_raw,terminal.id.to_string(),terminal.scope.project_id.to_string(),terminal.scope.goal_id.map(|v|v.to_string()),terminal.scope.task_id.map(|v|v.to_string()),kind.as_str().context("Unit kind")?,terminal.generation,terminal.owner_epoch,terminal.version,terminal.native_effects_open,terminal.result_finalization_open,terminal.worktree.to_str().context("Unit path")?,terminal.branch,terminal_raw])? == 1, "success Unit CAS changed");
        // W2: Ready -> Published.
        let artifact = m.plan.publication.artifact();
        ensure!(tx.execute("UPDATE result_artifacts SET state='published',version=?1,body=?2 WHERE id=?3 AND version=?4 AND state='ready' AND body=?5",
            params![m.artifact.version,m.artifact_raw,artifact.id.to_string(),artifact.version,serde_json::to_string(artifact)?])? == 1, "success artifact CAS changed");
        // W3: exact Task CAS from the marker Task.
        let (task_before, task_before_raw) = marker.original_plan().task_after();
        ensure!(
            tx.execute(
                "UPDATE tasks SET version=?1,body=?2 WHERE id=?3 AND version=?4 AND body=?5",
                params![
                    m.task.version,
                    m.task_raw,
                    task_before.id.to_string(),
                    task_before.version,
                    task_before_raw
                ]
            )? == 1,
            "success Task CAS changed"
        );
        // W4: the next Context.
        put_context_tx(&tx, &m.plan.context)?;
        append_event(
            &tx,
            &terminal.scope,
            "execution.result_published",
            json!({"unit":terminal.id,"artifact":artifact.id,"sha":artifact.revision,"workflow":m.workflow.parsed().id,"context_version":m.plan.context.version}),
        )?;
        // W1-W4 appended their own audit rows; the typed link takes the next.
        let sequence = next_audit_sequence(&tx)?;
        let scope = marker.scope();
        let audit = vec![
            SqlValue::Integer(sequence),
            SqlValue::Text(scope.project_id.to_string()),
            scope
                .goal_id
                .map_or(SqlValue::Null, |v| SqlValue::Text(v.to_string())),
            scope
                .task_id
                .map_or(SqlValue::Null, |v| SqlValue::Text(v.to_string())),
            SqlValue::Text(KIND.into()),
            SqlValue::Integer(m.plan.at),
            SqlValue::Text(m.data.clone()),
        ];
        let before = m.plan.observed.workflow_after();
        let mut writes = vec![
            ExactRowMutation::new(
                "managed_phase_operations",
                "UPDATE",
                Some(marker.original_operation_image()?.to_vec()),
                Some(m.operation.clone()),
            )?,
            ExactRowMutation::new(
                "records",
                "UPDATE",
                Some(record_image(before.parsed(), before.raw())?),
                Some(record_image(m.workflow.parsed(), m.workflow.raw())?),
            )?,
            ExactRowMutation::new("audit", "INSERT", None, Some(audit))?,
        ];
        writes.extend(m.driver.exact_mutations()?);
        self.binding_permits.with_exact_permit(writes, || {
            // W5: operation phase_open 1 -> 0, version 2.
            let names = super::permits::columns("managed_phase_operations").context("operation columns absent")?;
            let set = names.iter().enumerate().map(|(i,n)| format!("{n}=?{}",i+1)).collect::<Vec<_>>().join(",");
            let predicate = names.iter().enumerate().map(|(i,n)| format!("{n} IS ?{}",i+33)).collect::<Vec<_>>().join(" AND ");
            ensure!(tx.execute(&format!("UPDATE managed_phase_operations SET {set} WHERE {predicate}"), params_from_iter(m.operation.iter().chain(marker.original_operation_image()?)))? == 1, "success operation complete CAS changed");
            // W6: Workflow.
            let b = before.parsed();
            ensure!(tx.execute("UPDATE records SET version=?1,body=?2 WHERE id=?3 AND kind='workflow' AND project_id=?4 AND goal_id IS ?5 AND task_id IS ?6 AND version=?7 AND body=?8", params![m.workflow.parsed().version,m.workflow.raw(),b.id.to_string(),scope.project_id.to_string(),scope.goal_id.map(|v|v.to_string()),scope.task_id.map(|v|v.to_string()),b.version,before.raw()])? == 1, "success Workflow CAS changed");
            // W7: the typed success link.
            ensure!(tx.execute("INSERT INTO audit(sequence,project_id,goal_id,task_id,kind,at,data) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![sequence,scope.project_id.to_string(),scope.goal_id.map(|v|v.to_string()),scope.task_id.map(|v|v.to_string()),KIND,m.plan.at,m.data])? == 1, "success link missing");
            // W8: the marker-free Driver.
            m.driver.write_tx(&tx)?;
            self.binding_permits.ensure_consumed()
        })?;
        tx.commit()?;
        Ok(SuccessWrite::Known(Arc::new(
            SuccessClosureAcknowledgment {
                plan: m.plan.clone(),
                driver: m.driver.clone(),
            },
        )))
    }
    /// One read-only Immediate after an uncertain closure write.
    pub(crate) fn confirm_phase_success(
        &mut self,
        m: &SuccessClosureMaterial,
    ) -> Result<SuccessConfirmation<SuccessClosureAcknowledgment>> {
        let settled = m.plan.currency.settled();
        selected_database(&self.connection, settled)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let outcome = confirm_close_tx(&tx, m)?;
        tx.commit()?;
        match outcome {
            Some(true) => Ok(SuccessConfirmation::Known(Arc::new(
                SuccessClosureAcknowledgment {
                    plan: m.plan.clone(),
                    driver: m.driver.clone(),
                },
            ))),
            Some(false) => Ok(SuccessConfirmation::RolledBack),
            None => anyhow::bail!("success closure is neither committed nor rolled back"),
        }
    }
    /// Synchronous; caller holds `control_admission`. Exact committed-row
    /// check plus the SAME association's `publish_exact`.
    pub(crate) fn publish_success_driver(
        &mut self,
        ack: &Arc<SuccessClosureAcknowledgment>,
    ) -> Result<()> {
        self.publish_driver_closure(&ack.driver)
    }
}
