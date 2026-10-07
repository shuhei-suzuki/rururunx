//! Original-image RN-1 planning. Heavy turn material is never retained in Jobs.
use super::{
    canonical::{BODY_BYTES, WORKFLOW_DOMAIN, decode_value, digest, encode},
    marker_rows::{WORKFLOW_BYTES, record_image},
    permits::{ExactRowMutation, PrivatePermitManager, columns},
    publication::{OriginalMarker, charged_scope_bytes},
    successor::{ClosureLedger, validate_closure_ledger_tx},
};
use crate::{
    domain::Record,
    state::NonSuccessReader,
    workflow::{AttemptState, WorkflowSnapshot},
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Transaction, params, params_from_iter, types::Value as SqlValue};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub(crate) const REASON_DETAIL: &str =
    "native start ended before dispatch; explicit retry required";
const OP_BYTES: usize = 4 * 1024 * 1024;
const KIND: &str = "rrx.private.workflow.phase_closed";

pub(in crate::state) struct UnlinkedPhaseClosure {
    attempt: usize,
    workflow_version_after: u64,
    workflow_growth: u64,
    at: i64,
    workflow_after_sha256: String,
    operation_body_after_sha256: String,
}
pub(in crate::state) struct PhaseClosureImages {
    workflow_after: Record,
    workflow_after_raw: String,
    operation_after: Vec<SqlValue>,
}
pub(in crate::state) struct PhaseClosedFacts {
    pub(in crate::state) allocated_session_id: String,
    pub(in crate::state) unit_id: String,
    pub(in crate::state) unit_versions: (u64, u64),
    pub(in crate::state) readiness_version: i64,
    pub(in crate::state) lease_released: bool,
    pub(in crate::state) probe_released: bool,
}
pub(in crate::state) enum PhaseImage<'a> {
    Open,
    Closed {
        images: &'a PhaseClosureImages,
        at: i64,
        data: &'a str,
    },
}

fn workflow_delta(before: &Record, marker: &OriginalMarker, at: i64) -> Result<(usize, Record)> {
    let mut workflow: WorkflowSnapshot = serde_json::from_value(before.data.clone())?;
    ensure!(
        encode(&serde_json::to_value(&workflow)?, BODY_BYTES)? == encode(&before.data, BODY_BYTES)?,
        "non-success Workflow typed roundtrip drops fields"
    );
    let index = workflow
        .active
        .context("non-success original active attempt absent")?;
    let attempt = workflow
        .history
        .get_mut(index)
        .context("non-success original attempt absent")?;
    let f = marker.allocation().facts();
    let unit = attempt
        .unit
        .as_ref()
        .context("non-success original attempt Unit absent")?;
    ensure!(
        attempt.state == AttemptState::Running
            && attempt.dispatch_started
            && attempt.session_id.is_none()
            && attempt.execution.is_none()
            && attempt.completed_at.is_none()
            && attempt.native_wait.is_none()
            && attempt.next_due.is_none()
            && attempt.observations.is_empty()
            && attempt.claimed_observations == 0
            && unit.scope == *f.scope
            && unit.unit == f.unit_id
            && unit.generation == f.generation
            && unit.epoch == f.epoch,
        "non-success attempt is not original undispatched Running"
    );
    attempt.state = AttemptState::Failed;
    attempt.completed_at = Some(at);
    attempt.detail = Some(REASON_DETAIL.into());
    let original_workflow: WorkflowSnapshot = serde_json::from_value(before.data.clone())?;
    // Check every field, including unrecognized data already covered by roundtrip.
    let mut neutral = workflow.clone();
    neutral.history[index].state = original_workflow.history[index].state.clone();
    neutral.history[index].completed_at = original_workflow.history[index].completed_at;
    neutral.history[index]
        .detail
        .clone_from(&original_workflow.history[index].detail);
    ensure!(
        encode(&serde_json::to_value(neutral)?, BODY_BYTES)? == encode(&before.data, BODY_BYTES)?,
        "non-success Workflow changes unrelated facts"
    );
    let mut after = before.clone();
    after.version = before
        .version
        .checked_add(1)
        .filter(|v| *v <= i64::MAX as u64)
        .context("non-success Workflow version exhausted")?;
    after.updated_at = at;
    after.data = serde_json::to_value(workflow)?;
    crate::workflow::validate_transition(
        marker.original_plan().task_after().0,
        &after,
        Some(before),
    )?;
    Ok((index, after))
}
fn operation_delta(before: &[SqlValue]) -> Result<Vec<SqlValue>> {
    ensure!(
        before.len() == 32
            && before[29] == SqlValue::Integer(1)
            && before[30] == SqlValue::Integer(1),
        "non-success original operation is not open version1"
    );
    let SqlValue::Text(raw) = &before[31] else {
        anyhow::bail!("non-success operation body absent")
    };
    let original = decode_value(raw, OP_BYTES)?;
    ensure!(
        original["phase_open"] == true && original["version"] == 1,
        "non-success operation body/index differs"
    );
    let mut body = original.clone();
    let object = body
        .as_object_mut()
        .context("non-success operation body not object")?;
    object.insert("phase_open".into(), Value::Bool(false));
    object.insert("version".into(), json!(2));
    let mut after = before.to_vec();
    after[29] = SqlValue::Integer(0);
    after[30] = SqlValue::Integer(2);
    after[31] = SqlValue::Text(String::from_utf8(encode(&body, OP_BYTES)?)?);
    Ok(after)
}
fn material(marker: &OriginalMarker, at: i64) -> Result<(usize, PhaseClosureImages)> {
    let (index, workflow_after) =
        workflow_delta(marker.original_plan().workflow_after().0, marker, at)?;
    let workflow_after_raw = serde_json::to_string(&workflow_after)?;
    ensure!(
        workflow_after_raw.len() <= BODY_BYTES,
        "non-success Workflow postimage exceeds bound"
    );
    Ok((
        index,
        PhaseClosureImages {
            workflow_after,
            workflow_after_raw,
            operation_after: operation_delta(marker.original_operation_image()?)?,
        },
    ))
}
fn operation_digest(images: &PhaseClosureImages) -> Result<String> {
    let SqlValue::Text(raw) = &images.operation_after[31] else {
        anyhow::bail!("operation postimage absent")
    };
    Ok(format!("{:x}", Sha256::digest(raw.as_bytes())))
}
fn workflow_digest(images: &PhaseClosureImages) -> Result<String> {
    Ok(digest(
        WORKFLOW_DOMAIN,
        &encode(&serde_json::to_value(&images.workflow_after)?, BODY_BYTES)?,
    ))
}
pub(in crate::state) fn validate_original_phase_tx(
    tx: &Transaction<'_>,
    marker: &OriginalMarker,
    image: PhaseImage<'_>,
) -> Result<()> {
    let original = marker.original_plan();
    let workflow = match image {
        PhaseImage::Open => {
            marker.validate_unadvanced_tx(tx)?;
            validate_closure_ledger_tx(tx, marker, ClosureLedger::Empty)?;
            original.workflow_after()
        }
        PhaseImage::Closed { images, at, data } => {
            marker.validate_closed_tx(tx, &images.operation_after)?;
            validate_closure_ledger_tx(tx, marker, ClosureLedger::SolePhaseClosed { at, data })?;
            (&images.workflow_after, images.workflow_after_raw.as_str())
        }
    };
    original.before.validate_projection(
        tx,
        original.task_after.parsed(),
        original.task_after.raw(),
        Some(workflow),
    )?;
    marker.validate_driver_live_tx(tx)
}
pub(in crate::state) fn plan_unlinked_closure(
    _: &NonSuccessReader,
    tx: &Transaction<'_>,
    marker: &OriginalMarker,
    at: i64,
) -> Result<UnlinkedPhaseClosure> {
    validate_original_phase_tx(tx, marker, PhaseImage::Open)?;
    let (attempt, images) = material(marker, at)?;
    let before = marker.original_plan().workflow_after().1.len() as u64;
    Ok(UnlinkedPhaseClosure {
        attempt,
        workflow_version_after: images.workflow_after.version,
        workflow_growth: (images.workflow_after_raw.len() as u64).saturating_sub(before),
        at,
        workflow_after_sha256: workflow_digest(&images)?,
        operation_body_after_sha256: operation_digest(&images)?,
    })
}
impl UnlinkedPhaseClosure {
    pub(in crate::state) fn attempt(&self) -> usize {
        self.attempt
    }
    pub(in crate::state) fn at(&self) -> i64 {
        self.at
    }
    pub(in crate::state) fn materialize(
        &self,
        marker: &OriginalMarker,
    ) -> Result<PhaseClosureImages> {
        let (attempt, images) = material(marker, self.at)?;
        ensure!(
            attempt == self.attempt
                && images.workflow_after.version == self.workflow_version_after
                && workflow_digest(&images)? == self.workflow_after_sha256
                && operation_digest(&images)? == self.operation_body_after_sha256,
            "SAME non-success material digest differs; Held"
        );
        Ok(images)
    }
    pub(in crate::state) fn validate_budget_tx(
        &self,
        tx: &Transaction<'_>,
        marker: &OriginalMarker,
        audit_bytes: u64,
    ) -> Result<()> {
        ensure!(
            charged_scope_bytes(tx, &marker.scope())?
                .checked_add(self.workflow_growth)
                .and_then(|v| v.checked_add(audit_bytes))
                .is_some_and(|v| v <= WORKFLOW_BYTES),
            "non-success complete scoped budget exceeded"
        );
        Ok(())
    }
    pub(in crate::state) fn audit_data(
        &self,
        marker: &OriginalMarker,
        facts: &PhaseClosedFacts,
    ) -> Result<String> {
        let original = marker.original_plan();
        let before = original.workflow_after();
        let workflow: WorkflowSnapshot = serde_json::from_value(before.0.data.clone())?;
        let f = marker.allocation().facts();
        let data = json!({"kind":"phase_closed","project_id":f.scope.project_id,"goal_id":f.scope.goal_id,"task_id":f.scope.task_id,
            "workflow_id":before.0.id,"generation":workflow.generation,"attempt_index":self.attempt,"phase":workflow.history[self.attempt].phase.key(),
            "workflow_version_before":before.0.version,"workflow_version_after":self.workflow_version_after,"context_version":original.context().0.version,
            "private_operation_ref":marker.operation(),"original_marker_frame_sha256":marker.frame_digest(),
            "workflow_body_sha256_before":original.workflow_after.digest(WORKFLOW_DOMAIN),"workflow_body_sha256_after":self.workflow_after_sha256,
            "prior_ledger_digest":marker.frame_digest(),"canonical_body_recipe":"rrx.workflow-body-sha256/v1","at":self.at,
            "closure":"non_success","proof_source":"no_current_dispatch","reason":"start_ended_before_dispatch",
            "attempt_state_after":"failed","awaiting":"explicit_retry","task_version_preserved":original.task_after().0.version,"session_bound":false,
            "allocated_session_id":facts.allocated_session_id,"unit_id":facts.unit_id,"unit_version_before":facts.unit_versions.0,"unit_version_after":facts.unit_versions.1,
            "operation_version_after":2,"readiness_version":facts.readiness_version,"lease_released":facts.lease_released,"probe_released":facts.probe_released});
        Ok(String::from_utf8(encode(&data, 4096)?)?)
    }
}
impl PhaseClosureImages {
    pub(in crate::state) fn write_tx(
        &self,
        _: &NonSuccessReader,
        tx: &Transaction<'_>,
        permits: &PrivatePermitManager,
        marker: &OriginalMarker,
        closure: &UnlinkedPhaseClosure,
        audit_data: &str,
        w1: impl FnOnce(&Transaction<'_>) -> Result<()>,
    ) -> Result<()> {
        let before = marker.original_plan().workflow_after();
        let scope = marker.scope();
        let sequence: i64 = tx.query_row(
            "SELECT COALESCE((SELECT seq FROM sqlite_sequence WHERE name='audit'),0)",
            [],
            |r| r.get(0),
        )?;
        let sequence = sequence
            .checked_add(1)
            .context("audit sequence exhausted")?;
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
            SqlValue::Integer(closure.at),
            SqlValue::Text(audit_data.into()),
        ];
        let writes = vec![
            ExactRowMutation::new(
                "managed_phase_operations",
                "UPDATE",
                Some(marker.original_operation_image()?.to_vec()),
                Some(self.operation_after.clone()),
            )?,
            ExactRowMutation::new(
                "records",
                "UPDATE",
                Some(record_image(before.0, before.1)?),
                Some(record_image(
                    &self.workflow_after,
                    &self.workflow_after_raw,
                )?),
            )?,
            ExactRowMutation::new("audit", "INSERT", None, Some(audit))?,
        ];
        permits.with_exact_permit(writes, || {
            w1(tx)?;
            let names = columns("managed_phase_operations").context("operation columns absent")?;
            let set = names.iter().enumerate().map(|(i,n)| format!("{n}=?{}",i+1)).collect::<Vec<_>>().join(",");
            let predicate = names.iter().enumerate().map(|(i,n)| format!("{n} IS ?{}",i+33)).collect::<Vec<_>>().join(" AND ");
            ensure!(tx.execute(&format!("UPDATE managed_phase_operations SET {set} WHERE {predicate}"), params_from_iter(self.operation_after.iter().chain(marker.original_operation_image()?)))? == 1, "non-success operation complete CAS changed");
            ensure!(tx.execute("UPDATE records SET version=?1,body=?2 WHERE id=?3 AND kind='workflow' AND project_id=?4 AND goal_id IS ?5 AND task_id IS ?6 AND version=?7 AND body=?8", params![self.workflow_after.version,self.workflow_after_raw,before.0.id.to_string(),scope.project_id.to_string(),scope.goal_id.map(|v|v.to_string()),scope.task_id.map(|v|v.to_string()),before.0.version,before.1])? == 1, "non-success Workflow CAS changed");
            ensure!(tx.execute("INSERT INTO audit(sequence,project_id,goal_id,task_id,kind,at,data) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![sequence,scope.project_id.to_string(),scope.goal_id.map(|v|v.to_string()),scope.task_id.map(|v|v.to_string()),KIND,closure.at,audit_data])? == 1, "non-success link missing");
            permits.ensure_consumed()
        })
    }
}
