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

/// Borrowed W4 inputs of the SAME retained plan for one writer call.
/// Fields and construction stay in this module; owns no images or payload.
pub(in crate::state) struct PhaseClosedLink<'a> {
    closure: &'a UnlinkedPhaseClosure,
    audit_data: &'a str,
}

fn workflow_delta(
    before: &Record,
    task: &crate::domain::Task,
    original_unit: &crate::execution::ExecutionUnit,
    at: i64,
) -> Result<(usize, Record)> {
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
            && *unit == crate::execution::ManagedUnitRef::from(original_unit),
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
    crate::workflow::validate_transition(task, &after, Some(before))?;
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
    let (index, workflow_after) = workflow_delta(
        marker.original_plan().workflow_after().0,
        marker.original_plan().task_after().0,
        marker.unit(),
        at,
    )?;
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
    pub(in crate::state) fn link<'a>(&'a self, audit_data: &'a str) -> PhaseClosedLink<'a> {
        PhaseClosedLink {
            closure: self,
            audit_data,
        }
    }
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
            attempt == self.attempt()
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
        link: PhaseClosedLink<'_>,
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
            SqlValue::Integer(link.closure.at),
            SqlValue::Text(link.audit_data.into()),
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
            ensure!(tx.execute("INSERT INTO audit(sequence,project_id,goal_id,task_id,kind,at,data) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![sequence,scope.project_id.to_string(),scope.goal_id.map(|v|v.to_string()),scope.task_id.map(|v|v.to_string()),KIND,link.closure.at,link.audit_data])? == 1, "non-success link missing");
            permits.ensure_consumed()
        })
    }
}

#[cfg(test)]
mod primitives {
    use super::*;
    use crate::{
        domain::*,
        execution::*,
        workflow::{Phase, PhaseAttempt, SourceSnapshot},
    };
    use std::collections::BTreeMap;

    // Factual DTOs only. No database, Runtime, allocation, marker or proof.
    fn facts() -> (Task, ExecutionUnit, Record) {
        let mut task = Task::new(
            ProjectId::new(),
            GoalId::new(),
            "nongrant".into(),
            "claude".into(),
        );
        task.revision = Some("a".repeat(40));
        task.context_version = 1;
        let unit = ExecutionUnit {
            id: UnitId::new(),
            scope: task.scope(),
            kind: UnitKind::Executor,
            generation: 1,
            owner_epoch: 1,
            version: 1,
            phase: "Implement".into(),
            provider: "claude".into(),
            state: UnitState::Preparing,
            native_effects_open: true,
            result_finalization_open: true,
            work: None,
            cleanup: CleanupOutcome::Unknown,
            disposition: Disposition::Active,
            worktree: "/nongrant".into(),
            branch: Some("nongrant".into()),
            base_sha: task.revision.clone().unwrap(),
            profile_digest: "b".repeat(64),
            cookie: "nongrant".into(),
            session_id: None,
            artifact_id: None,
            wait_reason: None,
            capacity_retry_at: None,
            created_at: 1,
            updated_at: 1,
        };
        let config = crate::config::Config::default();
        let workflow = WorkflowSnapshot {
            workflow: task.workflow,
            risk: task.risk,
            generation: 1,
            context_version: 1,
            context_fresh: true,
            active: Some(0),
            completed: BTreeMap::new(),
            history: vec![PhaseAttempt {
                phase: Phase::Implement,
                generation: 1,
                context_version: 1,
                budget: crate::workflow::budget(task.workflow, Phase::Implement, &config),
                state: AttemptState::Running,
                session_id: None,
                execution: None,
                unit: Some(ManagedUnitRef::from(&unit)),
                native_wait: None,
                next_due: None,
                dispatch_started: true,
                observations: vec![],
                claimed_observations: 0,
                agent: Some("claude".into()),
                started_at: 1,
                completed_at: None,
                detail: None,
            }],
            escalations: vec![],
            retries: vec![],
            invalidations: vec![],
            terminal_decision: None,
            finalizations: vec![],
            sources: SourceSnapshot {
                scope: task.scope(),
                revision: task.revision.clone().unwrap(),
                artifact: None,
                source_versions: BTreeMap::new(),
                payload: "nongrant".into(),
            },
            configured_phases: crate::workflow::phases(task.workflow, &config),
            finished: false,
            held_reason: None,
        };
        let record = Record::new(
            task.scope(),
            RecordKind::Workflow,
            serde_json::to_value(workflow).unwrap(),
        );
        (task, unit, record)
    }
    #[test]
    fn nongrant_rn1_workflow_delta_changes_only_failed_completion_detail_and_record_clock() {
        let (task, unit, before) = facts();
        let (index, after) = workflow_delta(&before, &task, &unit, 44).unwrap();
        assert_eq!(index, 0);
        assert_eq!(after.version, before.version + 1);
        assert_eq!(after.updated_at, 44);
        let workflow: WorkflowSnapshot = serde_json::from_value(after.data.clone()).unwrap();
        assert_eq!(workflow.history[0].state, AttemptState::Failed);
        assert_eq!(workflow.history[0].completed_at, Some(44));
        assert_eq!(workflow.history[0].detail.as_deref(), Some(REASON_DETAIL));
        assert!(
            workflow.history[0].session_id.is_none() && workflow.history[0].execution.is_none()
        );
        let mut neutral = after.clone();
        neutral.version = before.version;
        neutral.updated_at = before.updated_at;
        let mut value = workflow;
        value.history[0].state = AttemptState::Running;
        value.history[0].completed_at = None;
        value.history[0].detail = None;
        neutral.data = serde_json::to_value(value).unwrap();
        assert_eq!(
            serde_json::to_value(neutral).unwrap(),
            serde_json::to_value(before).unwrap()
        );
    }
    #[test]
    fn nongrant_rn1_workflow_refuses_nonoriginal_states_fields_and_typed_loss() {
        let (task, unit, before) = facts();
        for (field, value) in [
            ("state", json!("failed")),
            ("dispatch_started", json!(false)),
            ("session_id", json!(SessionId::new())),
            ("completed_at", json!(9)),
            ("native_wait", json!("quota")),
            ("next_due", json!(9)),
            ("claimed_observations", json!(1)),
            ("unit", Value::Null),
        ] {
            let mut changed = before.clone();
            changed.data["history"][0][field] = value;
            assert!(
                workflow_delta(&changed, &task, &unit, 44).is_err(),
                "accepted {field}"
            );
        }
        let mut changed = before.clone();
        changed.data["unrecognized"] = json!("lost");
        assert!(workflow_delta(&changed, &task, &unit, 44).is_err());
        let mut changed_unit = unit.clone();
        changed_unit.owner_epoch += 1;
        assert!(workflow_delta(&before, &task, &changed_unit, 44).is_err());
    }
    fn operation(raw: String) -> Vec<SqlValue> {
        let mut values = (0..32)
            .map(|i| SqlValue::Text(format!("original-{i}")))
            .collect::<Vec<_>>();
        values[29] = SqlValue::Integer(1);
        values[30] = SqlValue::Integer(1);
        values[31] = SqlValue::Text(raw);
        values
    }
    #[test]
    fn nongrant_rn1_operation_delta_preserves_every_other_column_and_body_key() {
        let body = json!({"phase_open":true,"version":1,"original_frame":{"a":[1,"keep"]},"owner_id":"original","extra":true});
        let before = operation(body.to_string());
        let after = operation_delta(&before).unwrap();
        assert_eq!(&after[..29], &before[..29]);
        assert_eq!(after[29], SqlValue::Integer(0));
        assert_eq!(after[30], SqlValue::Integer(2));
        let SqlValue::Text(raw) = &after[31] else {
            panic!("body missing")
        };
        let mut changed: Value = serde_json::from_str(raw).unwrap();
        assert_eq!(changed["phase_open"], false);
        assert_eq!(changed["version"], 2);
        changed["phase_open"] = json!(true);
        changed["version"] = json!(1);
        assert_eq!(changed, body);
    }
    #[test]
    fn nongrant_rn1_operation_refuses_ambiguous_oversize_or_closed_preimages() {
        for raw in [
            "{\"phase_open\":true,\"phase_open\":false,\"version\":1}".into(),
            "x".repeat(OP_BYTES + 1),
            "{\"phase_open\":false,\"version\":1}".into(),
        ] {
            assert!(operation_delta(&operation(raw)).is_err());
        }
        let mut before = operation("{\"phase_open\":true,\"version\":1}".into());
        before[29] = SqlValue::Integer(0);
        assert!(operation_delta(&before).is_err());
        before[29] = SqlValue::Integer(1);
        before[30] = SqlValue::Integer(2);
        assert!(operation_delta(&before).is_err());
    }
}
