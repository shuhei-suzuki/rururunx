//! First normal-return factual binding, using only an actual Native actor proof.
//! Closed-settlement and typed successor reconciliation remain separate work;
//! no availability is enabled by this private writer.
use super::{
    canonical::{BODY_BYTES, Body, WORKFLOW_DOMAIN, encode},
    marker_rows::{WORKFLOW_BYTES, record_image},
    permits::ExactRowMutation,
    publication::charged_scope_bytes,
    session_identity::validate_negative_identities,
    snapshot::snapshot,
    successor::{CurrentWorkflowSuccessor, plan_current_phase, validate_current_tx},
};
use crate::{
    domain::*,
    execution::{
        Disposition, RuntimeOwner, UnitState,
        native::{ManagedSessionRef, NativePhaseBinding},
        native_result::{self, InvocationState, NativeInvocation},
    },
    state::Store,
    workflow::{Actor, AttemptState, WorkflowSnapshot},
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, Transaction, TransactionBehavior, params, types::Value as SqlValue};
use serde_json::{Value, json};

const SESSION_BYTES: usize = 4 * 1024 * 1024;
const OWNER_BYTES: usize = 32 * 1024;
const KIND: &str = "rrx.private.workflow.session_bound";

/// Retains the original actual actor across planning/commit errors. Neither a
/// Session DTO nor a public ManagedSessionRef can construct this argument.
pub(crate) struct ManagedBindingPlan {
    proof: NativePhaseBinding,
    current: CurrentWorkflowSuccessor,
    session: Body<Record>,
    owner_raw: String,
    invocation: Body<NativeInvocation>,
    after: Body<Record>,
    audit_data: String,
    at: i64,
    record_mutation: ExactRowMutation,
    already_bound: bool,
}
pub(crate) enum ManagedBindingOutcome {
    Bound,
    AlreadyBound,
}

fn text(value: impl ToString) -> SqlValue {
    SqlValue::Text(value.to_string())
}

fn latest_session(c: &Connection, proof: &NativePhaseBinding) -> Result<Body<Record>> {
    let f = proof.allocation().facts();
    let raw: Option<String> = c.query_row(
        "SELECT CASE WHEN length(CAST(body AS BLOB))<=?2 THEN body END FROM records WHERE id=?1",
        params![f.session_id.to_string(), SESSION_BYTES],
        |r| r.get(0),
    )?;
    let body = Body::<Record>::decode(
        raw.context("own latest Session body over bound")?,
        SESSION_BYTES,
    )?;
    let record = body.parsed();
    let session: Session = serde_json::from_value(record.data.clone())?;
    ensure!(
        encode(&serde_json::to_value(&session)?, SESSION_BYTES)?
            == encode(&record.data, SESSION_BYTES)?
            && record.id == RecordId(f.session_id.0)
            && record.kind == RecordKind::Session
            && record.scope == *f.scope
            && record.version >= proof.record_version()
            && record.version > 0
            && record.version <= i64::MAX as u64
            && session.id == f.session_id
            && session.scope == *f.scope
            && session.agent == f.alias
            && session.provider == f.provider
            && session.role == f.role
            && session.worktree == f.path
            && session.model.as_deref() == f.model
            && session.effort.as_deref() == f.effort
            && session.started_at == proof.session().started_at
            && proof
                .session()
                .native_ref
                .as_ref()
                .is_none_or(|id| session.native_ref.as_ref() == Some(id))
            && matches!(
                session.state,
                SessionState::Starting
                    | SessionState::Running
                    | SessionState::WaitingApproval
                    | SessionState::WaitingHuman
            ),
        "own latest Session immutable identity or normal eligibility changed"
    );
    // This current snapshot is not a stale returned Session-version CAS. PID,
    // initial native UUID and lifecycle can have advanced before this read.
    let exact: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM records WHERE id=?1 AND kind='session' AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND version=?5 AND body=?6)",
        params![record.id.to_string(),f.scope.project_id.to_string(),f.scope.goal_id.context("Session Goal absent")?.to_string(),f.scope.task_id.context("Session Task absent")?.to_string(),record.version,body.raw()], |r|r.get(0),
    )?;
    ensure!(exact, "own latest Session body/index disagreement");
    validate_negative_identities(
        c,
        f.scope,
        f.session_id,
        f.provider,
        session.native_ref.as_deref(),
    )?;
    Ok(body)
}

fn registered_owner(c: &Connection, proof: &NativePhaseBinding) -> Result<String> {
    let f = proof.allocation().facts();
    let raw: Option<String> = c.query_row(
        "SELECT CASE WHEN length(CAST(body AS BLOB))<=32768 THEN body END FROM managed_phase_owners WHERE owner_id=?1",
        [f.pair_id.to_string()], |r|r.get(0),
    )?;
    let body = Body::<Value>::decode(
        raw.context("own registered pair body over bound")?,
        OWNER_BYTES,
    )?;
    let expected = json!({"owner_id":f.pair_id,"operation_id":f.operation_id,"scope":f.scope,
        "unit_id":f.unit_id,"owner_epoch":f.epoch,"execution_generation":f.generation,
        "allocated_session_id":f.session_id,"provider":f.provider,"alias":f.alias,
        "role":f.role,"worktree":f.path,"origin":f.origin_id,
        "native_invocation_id":f.invocation_id,"validated":true,"version":2});
    ensure!(
        body.parsed() == &expected,
        "actual registered pair complete body differs"
    );
    validate_registered_owner_tx(c, proof, body.raw())?;
    Ok(body.raw().to_owned())
}

fn validate_registered_owner_tx(
    c: &Connection,
    proof: &NativePhaseBinding,
    raw: &str,
) -> Result<()> {
    let f = proof.allocation().facts();
    let role = serde_json::to_value(f.role)?;
    let exact: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM managed_phase_owners o JOIN session_units s ON s.session_id=o.allocated_session_id AND s.unit_id=o.unit_id AND s.project_id=o.project_id AND s.goal_id=o.goal_id AND s.task_id=o.task_id JOIN native_invocations n ON n.id=o.native_invocation_id AND n.session_id=s.session_id AND n.unit_id=s.unit_id AND n.project_id=o.project_id AND n.goal_id=o.goal_id AND n.task_id=o.task_id AND n.generation=o.execution_generation AND n.owner_epoch=o.owner_epoch AND n.provider=o.provider WHERE o.owner_id=?1 AND o.operation_id=?2 AND o.project_id=?3 AND o.goal_id=?4 AND o.task_id=?5 AND o.unit_id=?6 AND o.owner_epoch=?7 AND o.execution_generation=?8 AND o.allocated_session_id=?9 AND o.provider=?10 AND o.alias=?11 AND o.role=?12 AND o.worktree=?13 AND o.origin=?14 AND o.native_invocation_id=?15 AND o.validated=1 AND o.version=2 AND o.body=?16 AND n.state<>'closed')",
        params![f.pair_id.to_string(),f.operation_id.to_string(),f.scope.project_id.to_string(),f.scope.goal_id.context("pair Goal absent")?.to_string(),f.scope.task_id.context("pair Task absent")?.to_string(),f.unit_id.to_string(),f.epoch,f.generation,f.session_id.to_string(),f.provider,f.alias,role.as_str().context("pair role invalid")?,f.path.to_str().context("pair path invalid")?,f.origin_id.to_string(),f.invocation_id.to_string(),raw],|r|r.get(0),
    )?;
    ensure!(
        exact,
        "same actual registered owner/Unit/Session/invocation differs"
    );
    Ok(())
}

fn normal_eligibility(
    proof: &NativePhaseBinding,
    current: &CurrentWorkflowSuccessor,
) -> Result<()> {
    let f = proof.allocation().facts();
    let unit = current.unit();
    ensure!(
        proof.is_live()
            && proof.owner().launch_parts().is_retained()
            && std::ptr::eq(proof.marker().allocation().as_ref(), proof.allocation())
            && proof.settlement().is_none()
            && unit.session_id == Some(f.session_id)
            && unit.native_effects_open
            && unit.result_finalization_open
            && unit.work.is_none()
            && unit.disposition == Disposition::Active
            && matches!(
                unit.state,
                UnitState::DispatchPending | UnitState::Running | UnitState::WaitingQuota
            ),
        "normal binding lacks genuine live registration; terminal settlement requires its own predicate"
    );
    Ok(())
}

fn binding_invocation(
    c: &Connection,
    proof: &NativePhaseBinding,
    current: &CurrentWorkflowSuccessor,
) -> Result<Body<NativeInvocation>> {
    let f = proof.allocation().facts();
    let raw: Option<String> = c.query_row(
        "SELECT CASE WHEN length(CAST(body AS BLOB))<=?2 THEN body END FROM native_invocations WHERE id=?1",
        params![f.invocation_id.to_string(),native_result::INVOCATION_BYTES],|r|r.get(0),
    )?;
    let body = Body::<NativeInvocation>::decode(
        raw.context("registered invocation body over bound")?,
        native_result::INVOCATION_BYTES,
    )?;
    let invocation = body.parsed();
    invocation.validate()?;
    ensure!(
        invocation.id == f.invocation_id
            && invocation.unit_id == f.unit_id
            && invocation.session_id == f.session_id
            && invocation.scope == *f.scope
            && invocation.generation == f.generation
            && invocation.owner_epoch == f.epoch
            && invocation.provider == f.provider
            && invocation.state != InvocationState::Closed
            && invocation.unit_version >= f.unit_version
            && invocation.unit_version <= current.unit().version
            && invocation.context_version == Some(f.input.version)
            && invocation.context_sha256.as_deref()
                == Some(
                    native_result::digest(&serde_json::to_vec(
                        proof.marker().original_plan().context().0
                    )?)
                    .as_str()
                )
            && invocation.source_versions == f.input.source_versions
            && invocation.source_sha256
                == native_result::digest(&serde_json::to_vec(&f.input.source_versions)?)
            && invocation.revision == f.input.revision
            && invocation.artifact_id == f.artifact
            && invocation.payload_sha256 == native_result::digest(f.input.payload.as_bytes()),
        "registered invocation differs from original actual input allocation"
    );
    validate_invocation_tx(c, &body)?;
    Ok(body)
}

fn validate_invocation_tx(c: &Connection, body: &Body<NativeInvocation>) -> Result<()> {
    let n = body.parsed();
    let state = serde_json::to_value(n.state)?;
    let exact: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM native_invocations WHERE id=?1 AND unit_id=?2 AND session_id=?3 AND project_id=?4 AND goal_id=?5 AND task_id=?6 AND generation=?7 AND owner_epoch=?8 AND provider=?9 AND state=?10 AND version=?11 AND input_operation IS ?12 AND native_thread IS ?13 AND native_turn IS ?14 AND body=?15)",
        params![n.id.to_string(),n.unit_id.to_string(),n.session_id.to_string(),n.scope.project_id.to_string(),n.scope.goal_id.context("invocation Goal absent")?.to_string(),n.scope.task_id.context("invocation Task absent")?.to_string(),n.generation,n.owner_epoch,n.provider,state.as_str().context("invocation state invalid")?,n.version,n.input_operation.map(|v|v.to_string()),n.native_thread,n.native_turn,body.raw()],|r|r.get(0),
    )?;
    ensure!(exact, "complete current invocation body/index changed");
    Ok(())
}

fn projection(proof: &NativePhaseBinding, at: i64) -> Result<Body<Record>> {
    let marker = proof.marker().original_plan();
    let original = marker.workflow_after().0;
    let mut workflow: WorkflowSnapshot = serde_json::from_value(original.data.clone())?;
    // Complete typed roundtrip prevents a binder projection dropping any field.
    ensure!(
        encode(&serde_json::to_value(&workflow)?, BODY_BYTES)?
            == encode(&original.data, BODY_BYTES)?,
        "original Workflow projection drops fields"
    );
    let index = workflow.active.context("original active attempt absent")?;
    let attempt = workflow
        .history
        .get_mut(index)
        .context("original active attempt invalid")?;
    ensure!(
        attempt.session_id.is_none()
            && attempt.execution.is_none()
            && attempt.dispatch_started
            && attempt.state == AttemptState::Running
            && attempt.completed_at.is_none(),
        "original marked attempt is not unbound"
    );
    let f = proof.allocation().facts();
    ensure!(
        matches!(
            (attempt.phase.actor(), f.role),
            (Actor::Executor, SessionRole::Executor) | (Actor::Reviewer, SessionRole::Reviewer)
        ),
        "allocated phase actor differs"
    );
    attempt.session_id = Some(f.session_id);
    attempt.execution = Some(ManagedSessionRef {
        scope: f.scope.clone(),
        unit: f.unit_id,
        generation: f.generation,
        epoch: f.epoch,
        session: f.session_id,
    });
    let mut after = original.clone();
    after.data = serde_json::to_value(workflow)?;
    after.version = after
        .version
        .checked_add(1)
        .filter(|v| *v <= i64::MAX as u64)
        .context("binding Record version exhausted")?;
    after.updated_at = at;
    Body::decode(serde_json::to_string(&after)?, BODY_BYTES)
}

fn payload(proof: &NativePhaseBinding, after: &Body<Record>, at: i64) -> Result<String> {
    let marker = proof.marker().original_plan();
    let before = marker.workflow_after().0;
    let workflow: WorkflowSnapshot = serde_json::from_value(before.data.clone())?;
    let index = workflow.active.context("original binding attempt absent")?;
    let attempt = &workflow.history[index];
    let f = proof.allocation().facts();
    let data = json!({"kind":"session_bound","project_id":f.scope.project_id,
        "goal_id":f.scope.goal_id,"task_id":f.scope.task_id,"workflow_id":before.id,
        "generation":workflow.generation,"attempt_index":index,"phase":attempt.phase.key(),
        "session_id":f.session_id,"provider":f.provider,"actor":attempt.phase.actor(),"role":f.role,
        "workflow_version_before":before.version,"workflow_version_after":after.parsed().version,
        "task_version_preserved":marker.task_after().0.version,"dispatch_started":true,
        "marker_identity":{"scope":f.scope,"workflow_id":before.id,"workflow_version":before.version,
            "generation":workflow.generation,"attempt_index":index,"context_version":marker.context().0.version},
        "context_version":marker.context().0.version,"proof_source":"normal_return",
        "private_operation_ref":f.operation_id,"original_marker_frame_sha256":proof.marker().frame_digest(),
        "workflow_body_sha256_before":marker.workflow_after.digest(WORKFLOW_DOMAIN),
        "workflow_body_sha256_after":after.digest(WORKFLOW_DOMAIN),
        "prior_ledger_digest":proof.marker().frame_digest(),"canonical_body_recipe":"rrx.workflow-body-sha256/v1","at":at});
    String::from_utf8(encode(&data, 4096)?).context("binding audit encoding invalid")
}

/// Own actor supplies the genuine registration; a coherent read supplies only
/// current facts. No receipt ID, status poll, alias or capability mints a proof.
pub(crate) fn plan_managed_binding(
    owner: &RuntimeOwner,
    proof: NativePhaseBinding,
) -> Result<ManagedBindingPlan> {
    let current = plan_current_phase(owner, proof.marker())?;
    normal_eligibility(&proof, &current)?;
    let (session, owner_raw, invocation) = snapshot(owner, |tx| {
        validate_current_tx(tx, proof.marker(), &current)?;
        Ok((
            latest_session(tx, &proof)?,
            registered_owner(tx, &proof)?,
            binding_invocation(tx, &proof, &current)?,
        ))
    })?;
    let already_bound = current.has_links();
    let at = if already_bound {
        current
            .sole_binding_link()
            .context("typed factual successor reconciliation is not yet composed")?
            .at
    } else {
        now_ms()
    };
    let after = projection(&proof, at)?;
    let audit_data = payload(&proof, &after, at)?;
    if already_bound {
        let link = current
            .sole_binding_link()
            .context("exact first binding link absent")?;
        ensure!(
            link.scope == *proof.allocation().facts().scope
                && link.at == at
                && encode(&link.data, 4096)? == audit_data.as_bytes()
                && encode(&serde_json::to_value(current.workflow())?, BODY_BYTES)?
                    == encode(&serde_json::to_value(after.parsed())?, BODY_BYTES)?,
            "current tuple is not the exact same committed first binding"
        );
    } else {
        ensure!(
            current.workflow_raw() == proof.marker().original_plan().workflow_after().1,
            "first binding is not the complete original post-marker Workflow"
        );
    }
    let record_mutation = ExactRowMutation::new(
        "records",
        "UPDATE",
        Some(record_image(current.workflow(), current.workflow_raw())?),
        Some(record_image(after.parsed(), after.raw())?),
    )?;
    Ok(ManagedBindingPlan {
        proof,
        current,
        session,
        owner_raw,
        invocation,
        after,
        audit_data,
        at,
        record_mutation,
        already_bound,
    })
}

impl Store {
    /// Write exactly Workflow + one reserved audit; on error the caller still
    /// retains the SAME plan/proof. No ordinary fail/retry/owner refresh is used.
    pub(crate) fn bind_managed_phase(
        &mut self,
        plan: &ManagedBindingPlan,
    ) -> Result<ManagedBindingOutcome> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        validate_current_tx(&tx, plan.proof.marker(), &plan.current)?;
        normal_eligibility(&plan.proof, &plan.current)?;
        validate_registered_owner_tx(&tx, &plan.proof, &plan.owner_raw)?;
        validate_invocation_tx(&tx, &plan.invocation)?;
        let record = plan.session.parsed();
        let exact: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE id=?1 AND kind='session' AND project_id=?2 AND goal_id IS ?3 AND task_id IS ?4 AND version=?5 AND body=?6)",params![record.id.to_string(),record.scope.project_id.to_string(),record.scope.goal_id.map(|v|v.to_string()),record.scope.task_id.map(|v|v.to_string()),record.version,plan.session.raw()],|r|r.get(0))?;
        ensure!(
            exact,
            "current latest Session advanced; plan again without refreshing original pins"
        );
        let session: Session = serde_json::from_value(record.data.clone())?;
        validate_negative_identities(
            &tx,
            &record.scope,
            session.id,
            &session.provider,
            session.native_ref.as_deref(),
        )?;
        if plan.already_bound {
            tx.commit()?;
            return Ok(ManagedBindingOutcome::AlreadyBound);
        }
        let growth = u64::try_from(
            plan.after
                .raw()
                .len()
                .saturating_sub(plan.current.workflow_raw().len()),
        )?;
        ensure!(
            charged_scope_bytes(&tx, &record.scope)?
                .checked_add(growth)
                .is_some_and(|bytes| bytes <= WORKFLOW_BYTES),
            "binding complete scope budget exhausted"
        );
        // One <=4096-byte first link spends its own already charged1-MiB
        // original reservation. No new reserve or readiness row is written.
        let sequence: i64 = tx.query_row(
            "SELECT COALESCE((SELECT seq FROM sqlite_sequence WHERE name='audit'),0)",
            [],
            |r| r.get(0),
        )?;
        let sequence = sequence
            .checked_add(1)
            .context("audit identity exhausted")?;
        let audit_image = vec![
            SqlValue::Integer(sequence),
            text(record.scope.project_id),
            record.scope.goal_id.map(text).unwrap_or(SqlValue::Null),
            record.scope.task_id.map(text).unwrap_or(SqlValue::Null),
            text(KIND),
            SqlValue::Integer(plan.at),
            text(&plan.audit_data),
        ];
        let audit_mutation = ExactRowMutation::new("audit", "INSERT", None, Some(audit_image))?;
        // Record image was prepared outsideStore. Copying it into the one-use
        // permit is bounded; audit's integer identity is allocated under TX.
        let record_mutation = plan.record_mutation.copy_for_transaction()?;
        self.binding_permits.with_exact_permit(vec![record_mutation,audit_mutation],|| {
            let before = plan.current.workflow();
            let after = plan.after.parsed();
            ensure!(tx.execute("UPDATE records SET version=?1,body=?2 WHERE id=?3 AND kind='workflow' AND project_id=?4 AND goal_id IS ?5 AND task_id IS ?6 AND version=?7 AND body=?8",params![after.version,plan.after.raw(),before.id.to_string(),before.scope.project_id.to_string(),before.scope.goal_id.map(|v|v.to_string()),before.scope.task_id.map(|v|v.to_string()),before.version,plan.current.workflow_raw()])?==1,"factual Workflow-only binding CAS changed");
            ensure!(tx.execute("INSERT INTO audit(sequence,project_id,goal_id,task_id,kind,at,data) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![sequence,record.scope.project_id.to_string(),record.scope.goal_id.map(|v|v.to_string()),record.scope.task_id.map(|v|v.to_string()),KIND,plan.at,plan.audit_data])?==1,"reserved binding link missing");
            self.binding_permits.ensure_consumed()
        })?;
        tx.commit()?;
        Ok(ManagedBindingOutcome::Bound)
    }
}
