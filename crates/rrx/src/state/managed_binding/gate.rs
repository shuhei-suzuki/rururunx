//! SC `gate_claim` and fused `gate_observed`: one Workflow advance plus one
//! typed link each, on a settled marker-bound open phase. Writes return
//! Known, typed Conflict (no write possible) or Err (uncertain); confirmation
//! is the two-branch postimage/preimage classification, never the open-phase
//! validator alone.
use super::{
    canonical::{BODY_BYTES, Body, WORKFLOW_DOMAIN, encode},
    marker_rows::{WORKFLOW_BYTES, record_image},
    permits::ExactRowMutation,
    publication::charged_scope_bytes,
    success::{SettledCurrency, SettledPhase, plan_settled_currency, validate_settled_tx},
    successor::LINK_KINDS,
};
use crate::{
    domain::{Record, Task},
    execution::{
        RuntimeOwner,
        workflow_gates::{SettledGateCompletion, SettledGateOutcome},
    },
    state::Store,
    workflow::{AttemptState, Evidence, GateObservation, GateOutcome, WorkflowSnapshot},
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, Transaction, TransactionBehavior, params, types::Value as SqlValue};
use serde_json::{Map, Value, json};
use std::sync::Arc;

const CLAIM_KIND: &str = "rrx.private.workflow.gate_claim";
const OBSERVED_KIND: &str = "rrx.private.workflow.gate_observed";
const OBSERVATION_BYTES: usize = 64 * 1024;
/// Mandatory closure reservation (Context + Workflow + Task + small rows).
pub(crate) const SUCCESS_CLOSURE_HEADROOM: u64 = (8 + 8 + 1) * 1024 * 1024 + 512 * 1024;
pub(crate) const GATE_FAILED_DETAIL: &str = "gate failed; bound non-success closure unavailable";
pub(crate) const GATE_UNKNOWN_DETAIL: &str = "gate outcome unknown; explicit recovery required";

/// `Err` from a write port is uncertain; the SAME plan goes to confirmation.
pub(crate) enum SuccessWrite<A> {
    Known(Arc<A>),
    Conflict(anyhow::Error),
}
/// `Err` from confirmation is Held.
pub(crate) enum SuccessConfirmation<A> {
    Known(Arc<A>),
    RolledBack,
}

/// One Workflow + link advance of a settled open phase at a planned endpoint.
struct LinkAdvance {
    currency: SettledCurrency,
    after: Body<Record>,
    kind: &'static str,
    data: String,
    at: i64,
    headroom: u64,
}

pub(crate) struct GateClaimPlan {
    advance: LinkAdvance,
}
pub(crate) struct GateClaimAcknowledgment {
    plan: Arc<GateClaimPlan>,
}
impl GateClaimAcknowledgment {
    pub(crate) fn settled(&self) -> &Arc<SettledPhase> {
        self.plan.advance.currency.settled()
    }
}

/// What a fused observation records; built only from a sealed completion.
enum Observed {
    Passed(Box<(Evidence, Record)>),
    Waiting(String),
    Failed,
    Unknown,
}
pub(crate) struct GateObservedPlan {
    observed: Observed,
    advance: LinkAdvance,
}
pub(crate) struct GateObservedAcknowledgment {
    plan: Arc<GateObservedPlan>,
}
impl GateObservedAcknowledgment {
    pub(crate) fn settled(&self) -> &Arc<SettledPhase> {
        self.plan.advance.currency.settled()
    }
    /// The Passed evidence and its gate receipt; None for any other outcome.
    pub(crate) fn passed(&self) -> Option<(&Evidence, &Record)> {
        match &self.plan.observed {
            Observed::Passed(passed) => Some((&passed.0, &passed.1)),
            _ => None,
        }
    }
    /// The Workflow postimage this acknowledgment committed.
    pub(in crate::state) fn workflow_after(&self) -> &Body<Record> {
        &self.plan.advance.after
    }
    /// The ledger count after this acknowledgment's link.
    pub(in crate::state) fn link_count(&self) -> usize {
        self.plan.advance.currency.current().link_count() + 1
    }
}

pub(super) fn typed(before: &Record) -> Result<WorkflowSnapshot> {
    let workflow: WorkflowSnapshot = serde_json::from_value(before.data.clone())?;
    ensure!(
        encode(&serde_json::to_value(&workflow)?, BODY_BYTES)? == encode(&before.data, BODY_BYTES)?,
        "settled Workflow typed roundtrip drops fields"
    );
    Ok(workflow)
}
fn advanced(
    before: &Record,
    workflow: WorkflowSnapshot,
    task: &Task,
    at: i64,
) -> Result<Body<Record>> {
    advanced_from(before, before, workflow, task, at)
}
fn advanced_from(
    before: &Record,
    previous: &Record,
    workflow: WorkflowSnapshot,
    task: &Task,
    at: i64,
) -> Result<Body<Record>> {
    let after = bumped(before, workflow, at)?;
    crate::workflow::validate_transition(task, &after, Some(previous))?;
    Body::decode(serde_json::to_string(&after)?, BODY_BYTES)
}
fn bumped(before: &Record, workflow: WorkflowSnapshot, at: i64) -> Result<Record> {
    let mut after = before.clone();
    after.version = before
        .version
        .checked_add(1)
        .filter(|v| *v <= i64::MAX as u64)
        .context("settled Workflow version exhausted")?;
    after.updated_at = at;
    after.data = serde_json::to_value(workflow)?;
    Ok(after)
}

/// Running (bound) -> Evaluating; claimed_observations = observations.len().
fn claim_delta(before: &Record, task: &Task, at: i64) -> Result<(usize, Body<Record>)> {
    let original = typed(before)?;
    let mut workflow = original.clone();
    let index = workflow
        .active
        .context("settled claim active attempt absent")?;
    let attempt = workflow
        .history
        .get_mut(index)
        .context("settled claim attempt absent")?;
    ensure!(
        attempt.state == AttemptState::Running
            && attempt.dispatch_started
            && attempt.session_id.is_some()
            && attempt.execution.is_some()
            && attempt.completed_at.is_none()
            && attempt.claimed_observations == 0
            && attempt.observations.is_empty(),
        "settled claim attempt is not the bound Running attempt"
    );
    attempt.state = AttemptState::Evaluating;
    attempt.claimed_observations = attempt.observations.len();
    let mut neutral = workflow.clone();
    neutral.history[index].state = original.history[index].state.clone();
    neutral.history[index].claimed_observations = original.history[index].claimed_observations;
    ensure!(
        encode(&serde_json::to_value(neutral)?, BODY_BYTES)? == encode(&before.data, BODY_BYTES)?,
        "settled claim changes unrelated facts"
    );
    Ok((index, advanced(before, workflow, task, at)?))
}

/// Appends exactly one observation and applies its disposition.
fn observed_delta(
    before: &Record,
    task: &Task,
    observed: &Observed,
    sources: &crate::workflow::SourceSnapshot,
    at: i64,
) -> Result<(usize, Body<Record>)> {
    let original = typed(before)?;
    let mut workflow = original.clone();
    let index = workflow
        .active
        .context("settled observation active attempt absent")?;
    let attempt = workflow
        .history
        .get_mut(index)
        .context("settled observation attempt absent")?;
    ensure!(
        attempt.state == AttemptState::Evaluating
            && attempt.claimed_observations == attempt.observations.len(),
        "settled observation requires the claimed Evaluating attempt"
    );
    let (outcome, error) = match observed {
        Observed::Passed(passed) => (Some(GateOutcome::Passed(passed.0.clone())), None),
        Observed::Waiting(detail) => {
            attempt.state = AttemptState::Waiting;
            attempt.detail = Some(detail.clone());
            (Some(GateOutcome::Waiting(detail.clone())), None)
        }
        Observed::Failed => {
            attempt.state = AttemptState::Waiting;
            attempt.detail = Some(GATE_FAILED_DETAIL.into());
            (Some(GateOutcome::Failed(GATE_FAILED_DETAIL.into())), None)
        }
        Observed::Unknown => {
            attempt.state = AttemptState::Waiting;
            attempt.detail = Some(GATE_UNKNOWN_DETAIL.into());
            (None, Some(GATE_UNKNOWN_DETAIL.to_owned()))
        }
    };
    let observation = GateObservation {
        sources: crate::workflow::authority_only(sources),
        outcome,
        error,
        at,
    };
    ensure!(
        serde_json::to_vec(&observation)?.len() <= OBSERVATION_BYTES,
        "settled observation exceeds its bound"
    );
    attempt.observations.push(observation.clone());
    match observed {
        Observed::Failed => workflow.held_reason = Some(GATE_FAILED_DETAIL.into()),
        Observed::Unknown => workflow.held_reason = Some(GATE_UNKNOWN_DETAIL.into()),
        _ => {}
    }
    if matches!(observed, Observed::Unknown) {
        // A sealed Unknown has no known outcome, so the generic validator's
        // known-observation exit cannot admit it. Its SC-only audited
        // disposition is checked as an exact delta from the persisted before:
        // one observation plus the fixed Waiting/detail/held_reason, nothing else.
        let mut neutral = workflow.clone();
        let attempt = &mut neutral.history[index];
        let appended = attempt
            .observations
            .pop()
            .context("settled Unknown observation absent")?;
        ensure!(
            appended.outcome.is_none()
                && appended.error.as_deref() == Some(GATE_UNKNOWN_DETAIL)
                && attempt.state == AttemptState::Waiting
                && attempt.detail.as_deref() == Some(GATE_UNKNOWN_DETAIL)
                && neutral.held_reason.as_deref() == Some(GATE_UNKNOWN_DETAIL),
            "settled Unknown disposition differs"
        );
        attempt.state = original.history[index].state.clone();
        attempt.detail = original.history[index].detail.clone();
        neutral.held_reason = original.held_reason.clone();
        ensure!(
            encode(&serde_json::to_value(neutral)?, BODY_BYTES)?
                == encode(&before.data, BODY_BYTES)?,
            "settled Unknown changes unrelated facts"
        );
        let after = bumped(before, workflow, at)?;
        return Ok((
            index,
            Body::decode(serde_json::to_string(&after)?, BODY_BYTES)?,
        ));
    }
    // The audited link is the observer. As in the Driver gate writer, the
    // exact virtual observer image (only this observation appended) is the
    // transition predecessor; every other transition rule stays unchanged.
    let mut image = original;
    image.history[index].observations.push(observation);
    let mut observer = before.clone();
    observer.data = serde_json::to_value(&image)?;
    let after = advanced_from(before, &observer, workflow, task, at)?;
    Ok((index, after))
}

/// The trigger-required header shared by every typed link.
pub(super) fn header(
    currency: &SettledCurrency,
    after: &Body<Record>,
    kind: &str,
    index: usize,
    at: i64,
) -> Result<Map<String, Value>> {
    let marker = currency.settled().marker();
    let original = marker.original_plan();
    let current = currency.current();
    let before = current.workflow_body();
    let workflow = typed(before.parsed())?;
    let f = marker.allocation().facts();
    let data = json!({"kind":kind.strip_prefix("rrx.private.workflow.").context("link kind")?,
        "project_id":f.scope.project_id,"goal_id":f.scope.goal_id,"task_id":f.scope.task_id,
        "workflow_id":before.parsed().id,"generation":workflow.generation,"attempt_index":index,
        "phase":workflow.history[index].phase.key(),
        "workflow_version_before":before.parsed().version,"workflow_version_after":after.parsed().version,
        "context_version":original.context().0.version,"private_operation_ref":marker.operation(),
        "original_marker_frame_sha256":marker.frame_digest(),
        "workflow_body_sha256_before":before.digest(WORKFLOW_DOMAIN),
        "workflow_body_sha256_after":after.digest(WORKFLOW_DOMAIN),
        "prior_ledger_digest":current.prior_ledger_digest(marker)?,
        "canonical_body_recipe":"rrx.workflow-body-sha256/v1","at":at});
    data.as_object().cloned().context("link header object")
}

pub(super) fn link_data(mut header: Map<String, Value>, extra: Value) -> Result<String> {
    for (key, value) in extra.as_object().context("link extra object")? {
        ensure!(
            header.insert(key.clone(), value.clone()).is_none(),
            "link field collides with its header"
        );
    }
    String::from_utf8(encode(&Value::Object(header), 4096)?).context("link encoding invalid")
}

fn advance(
    currency: SettledCurrency,
    after: Body<Record>,
    kind: &'static str,
    data: String,
    at: i64,
    headroom: u64,
) -> Result<LinkAdvance> {
    Ok(LinkAdvance {
        currency,
        after,
        kind,
        data,
        at,
        headroom,
    })
}

pub(super) fn selected_database(c: &Connection, settled: &SettledPhase) -> Result<()> {
    ensure!(
        settled
            .allocation()
            .facts()
            .state_path
            .to_str()
            .is_some_and(|path| c.path() == Some(path)),
        "settled writer is not the selected owner's database"
    );
    Ok(())
}

/// Checks before the first write; a refusal here is a definitive Conflict.
fn checked_advance_tx(tx: &Transaction<'_>, a: &LinkAdvance) -> Result<i64> {
    validate_settled_tx(tx, &a.currency)?;
    let scope = a.currency.settled().marker().scope();
    let growth = u64::try_from(
        a.after
            .raw()
            .len()
            .saturating_sub(a.currency.current().workflow_raw().len()),
    )?;
    ensure!(
        charged_scope_bytes(tx, &scope)?
            .checked_add(growth)
            .and_then(|v| v.checked_add(a.data.len() as u64))
            .and_then(|v| v.checked_add(a.headroom))
            .is_some_and(|v| v <= WORKFLOW_BYTES),
        "settled link scope budget or closure headroom exhausted"
    );
    let sequence: i64 = tx.query_row(
        "SELECT COALESCE((SELECT seq FROM sqlite_sequence WHERE name='audit'),0)",
        [],
        |r| r.get(0),
    )?;
    sequence.checked_add(1).context("audit identity exhausted")
}

impl Store {
    fn write_advance(&mut self, a: &LinkAdvance) -> Result<Option<anyhow::Error>> {
        if let Err(cause) = selected_database(&self.connection, a.currency.settled()) {
            return Ok(Some(cause));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let sequence = match checked_advance_tx(&tx, a) {
            Ok(sequence) => sequence,
            Err(cause) => {
                drop(tx);
                return Ok(Some(cause));
            }
        };
        let scope = a.currency.settled().marker().scope();
        let audit = vec![
            SqlValue::Integer(sequence),
            SqlValue::Text(scope.project_id.to_string()),
            scope
                .goal_id
                .map_or(SqlValue::Null, |v| SqlValue::Text(v.to_string())),
            scope
                .task_id
                .map_or(SqlValue::Null, |v| SqlValue::Text(v.to_string())),
            SqlValue::Text(a.kind.into()),
            SqlValue::Integer(a.at),
            SqlValue::Text(a.data.clone()),
        ];
        let writes = vec![
            // Materialized per write from the SAME retained images; plans
            // never keep a duplicate full-image mutation (SC §12).
            ExactRowMutation::new(
                "records",
                "UPDATE",
                Some(record_image(
                    a.currency.current().workflow(),
                    a.currency.current().workflow_raw(),
                )?),
                Some(record_image(a.after.parsed(), a.after.raw())?),
            )?,
            ExactRowMutation::new("audit", "INSERT", None, Some(audit))?,
        ];
        self.binding_permits.with_exact_permit(writes, || {
            let before = a.currency.current().workflow();
            let after = a.after.parsed();
            ensure!(tx.execute("UPDATE records SET version=?1,body=?2 WHERE id=?3 AND kind='workflow' AND project_id=?4 AND goal_id IS ?5 AND task_id IS ?6 AND version=?7 AND body=?8",params![after.version,a.after.raw(),before.id.to_string(),before.scope.project_id.to_string(),before.scope.goal_id.map(|v|v.to_string()),before.scope.task_id.map(|v|v.to_string()),before.version,a.currency.current().workflow_raw()])?==1,"settled Workflow CAS changed");
            ensure!(tx.execute("INSERT INTO audit(sequence,project_id,goal_id,task_id,kind,at,data) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![sequence,scope.project_id.to_string(),scope.goal_id.map(|v|v.to_string()),scope.task_id.map(|v|v.to_string()),a.kind,a.at,a.data])?==1,"settled link missing");
            self.binding_permits.ensure_consumed()
        })?;
        tx.commit()?;
        Ok(None)
    }
    /// Two-branch confirmation: Some(true) Known, Some(false) RolledBack,
    /// None mixed (Held).
    fn confirm_advance(&mut self, a: &LinkAdvance) -> Result<Option<bool>> {
        let settled = a.currency.settled();
        selected_database(&self.connection, settled)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let outcome = confirm_advance_tx(&tx, a)?;
        tx.commit()?;
        Ok(outcome)
    }
    pub(crate) fn plan_settled_gate_claim(
        _owner: &RuntimeOwner,
        currency: SettledCurrency,
        at: i64,
    ) -> Result<Arc<GateClaimPlan>> {
        let marker = currency.settled().marker();
        let task = marker.original_plan().task_after().0;
        let (index, after) = claim_delta(currency.current().workflow(), task, at)?;
        let header = header(&currency, &after, CLAIM_KIND, index, at)?;
        let phase = currency.settled().phase().key();
        let data = link_data(
            header,
            json!({"claimed_observations":0,"gate_phase":phase,
                "closure_headroom_bytes":SUCCESS_CLOSURE_HEADROOM}),
        )?;
        Ok(Arc::new(GateClaimPlan {
            advance: advance(
                currency,
                after,
                CLAIM_KIND,
                data,
                at,
                SUCCESS_CLOSURE_HEADROOM,
            )?,
        }))
    }
    pub(crate) fn claim_settled_gate(
        &mut self,
        plan: &Arc<GateClaimPlan>,
    ) -> Result<SuccessWrite<GateClaimAcknowledgment>> {
        #[cfg(test)]
        let fault = super::fault::take(
            plan.advance.currency.settled().marker().scope().task_id,
            super::fault::CLAIM,
        );
        #[cfg(test)]
        super::fault::before(fault)?;
        let result = self.claim_settled_gate_inner(plan)?;
        #[cfg(test)]
        let result = super::fault::after(fault, matches!(result, SuccessWrite::Known(_)), result)?;
        Ok(result)
    }
    fn claim_settled_gate_inner(
        &mut self,
        plan: &Arc<GateClaimPlan>,
    ) -> Result<SuccessWrite<GateClaimAcknowledgment>> {
        Ok(match self.write_advance(&plan.advance)? {
            None => SuccessWrite::Known(Arc::new(GateClaimAcknowledgment { plan: plan.clone() })),
            Some(cause) => SuccessWrite::Conflict(cause),
        })
    }
    pub(crate) fn confirm_settled_gate_claim(
        &mut self,
        plan: &Arc<GateClaimPlan>,
    ) -> Result<SuccessConfirmation<GateClaimAcknowledgment>> {
        match self.confirm_advance(&plan.advance)? {
            Some(true) => Ok(SuccessConfirmation::Known(Arc::new(
                GateClaimAcknowledgment { plan: plan.clone() },
            ))),
            Some(false) => Ok(SuccessConfirmation::RolledBack),
            None => anyhow::bail!("gate claim is neither committed nor rolled back"),
        }
    }
    /// Plans the fused observation at the post-claim endpoint from the
    /// producer's sealed completion of the SAME claim.
    pub(crate) fn plan_settled_gate_observed(
        owner: &RuntimeOwner,
        claim: &Arc<GateClaimAcknowledgment>,
        completion: &SettledGateCompletion,
        at: i64,
    ) -> Result<Arc<GateObservedPlan>> {
        ensure!(
            completion.belongs_to(claim),
            "gate completion belongs to another claim"
        );
        #[cfg(test)]
        super::fault::before(super::fault::take(
            claim.settled().marker().scope().task_id,
            super::fault::OBSERVED_PLAN,
        ))?;
        let (outcome, receipt, sources) = completion.parts();
        let settled = claim.settled();
        let phase = settled.phase();
        let observed = match (outcome, receipt) {
            (SettledGateOutcome::Known(GateOutcome::Passed(evidence)), Some(receipt)) => {
                Observed::Passed(Box::new((evidence.clone(), receipt.clone())))
            }
            (SettledGateOutcome::Known(GateOutcome::Waiting(detail)), None)
                if *detail
                    == format!(
                        "{} requires its qualified production evidence integration",
                        phase.key()
                    )
                    && detail.len() <= 128 =>
            {
                Observed::Waiting(detail.clone())
            }
            (SettledGateOutcome::Known(GateOutcome::Failed(_)), None) => Observed::Failed,
            _ => Observed::Unknown,
        };
        let currency = plan_settled_currency(owner, settled)?;
        let claimed = &claim.plan.advance;
        ensure!(
            currency.current().workflow_raw() == claimed.after.raw()
                && currency.current().link_count() == claimed.currency.current().link_count() + 1,
            "gate observation is not at the SAME claim endpoint"
        );
        let task = settled.marker().original_plan().task_after().0;
        let (index, after) =
            observed_delta(currency.current().workflow(), task, &observed, sources, at)?;
        let header = header(&currency, &after, OBSERVED_KIND, index, at)?;
        let extra = match &observed {
            Observed::Passed(passed) => json!({"outcome":"passed",
                "reason_code":"gate_passed","gate_receipt_ref":passed.1.id,
                "evidence_sha256":crate::execution::native_result::digest(&serde_json::to_vec(&passed.0)?)}),
            Observed::Waiting(_) => {
                json!({"outcome":"waiting","reason_code":"evidence_integration_unavailable"})
            }
            Observed::Failed => json!({"outcome":"held","reason_code":"gate_failed"}),
            Observed::Unknown => json!({"outcome":"held","reason_code":"gate_unknown"}),
        };
        let data = link_data(header, extra)?;
        let headroom = if matches!(observed, Observed::Passed(_)) {
            SUCCESS_CLOSURE_HEADROOM
        } else {
            0
        };
        Ok(Arc::new(GateObservedPlan {
            observed,
            advance: advance(currency, after, OBSERVED_KIND, data, at, headroom)?,
        }))
    }
    pub(crate) fn observe_settled_gate(
        &mut self,
        plan: &Arc<GateObservedPlan>,
    ) -> Result<SuccessWrite<GateObservedAcknowledgment>> {
        #[cfg(test)]
        let fault = super::fault::take(
            plan.advance.currency.settled().marker().scope().task_id,
            super::fault::OBSERVED,
        );
        #[cfg(test)]
        super::fault::before(fault)?;
        #[cfg(test)]
        if fault == Some(super::fault::CommitFault::Conflict) {
            return Ok(SuccessWrite::Conflict(anyhow::anyhow!(
                "injected typed observed conflict"
            )));
        }
        let result = self.observe_settled_gate_inner(plan)?;
        #[cfg(test)]
        let result = super::fault::after(fault, matches!(result, SuccessWrite::Known(_)), result)?;
        Ok(result)
    }
    fn observe_settled_gate_inner(
        &mut self,
        plan: &Arc<GateObservedPlan>,
    ) -> Result<SuccessWrite<GateObservedAcknowledgment>> {
        Ok(match self.write_advance(&plan.advance)? {
            None => {
                SuccessWrite::Known(Arc::new(GateObservedAcknowledgment { plan: plan.clone() }))
            }
            Some(cause) => SuccessWrite::Conflict(cause),
        })
    }
    pub(crate) fn confirm_settled_gate_observed(
        &mut self,
        plan: &Arc<GateObservedPlan>,
    ) -> Result<SuccessConfirmation<GateObservedAcknowledgment>> {
        match self.confirm_advance(&plan.advance)? {
            Some(true) => Ok(SuccessConfirmation::Known(Arc::new(
                GateObservedAcknowledgment { plan: plan.clone() },
            ))),
            Some(false) => Ok(SuccessConfirmation::RolledBack),
            None => anyhow::bail!("gate observation is neither committed nor rolled back"),
        }
    }
}

/// Exactly the planned link is the ledger head, directly after the planned
/// predecessor count.
pub(in crate::state) fn validate_link_head_tx(
    c: &Connection,
    settled: &SettledPhase,
    count: usize,
    kind: &str,
    at: i64,
    data: &str,
) -> Result<bool> {
    let operation = settled.marker().operation().to_string();
    let actual: usize = c.query_row(
        &format!("SELECT count(*) FROM (SELECT sequence FROM audit WHERE kind IN ({LINK_KINDS}) AND json_extract(data,'$.private_operation_ref')=?1 LIMIT 257)"),
        [&operation],
        |r| r.get(0),
    )?;
    if actual != count {
        return Ok(false);
    }
    let head: Option<(String, i64, Option<String>)> = c
        .query_row(
            &format!("SELECT kind,at,CASE WHEN length(CAST(data AS BLOB))<=4096 THEN data END FROM audit WHERE kind IN ({LINK_KINDS}) AND json_extract(data,'$.private_operation_ref')=?1 ORDER BY sequence DESC LIMIT 1"),
            [&operation],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            e => Err(e),
        })?;
    Ok(head.is_some_and(|(k, a, d)| k == kind && a == at && d.as_deref() == Some(data)))
}

fn confirm_advance_tx(tx: &Transaction<'_>, a: &LinkAdvance) -> Result<Option<bool>> {
    let settled = a.currency.settled();
    let marker = settled.marker();
    let original = marker.original_plan();
    // Common immutable authority: sealed terminal images and the open phase.
    settled.settlement().validate_terminal_images_tx(tx)?;
    marker.validate_open_tx(tx)?;
    let raw: Option<String> = tx.query_row(
        "SELECT CASE WHEN length(CAST(body AS BLOB))<=?2 THEN body END FROM records WHERE id=?1 AND kind='workflow'",
        params![a.after.parsed().id.to_string(), BODY_BYTES],
        |r| r.get(0),
    )?;
    let raw = raw.context("settled confirmation Workflow over bound")?;
    if raw == a.after.raw() {
        // Postimage branch: the open-phase conjuncts at the post endpoint.
        original.before.validate_projection(
            tx,
            original.task_after.parsed(),
            original.task_after.raw(),
            Some((a.after.parsed(), a.after.raw())),
        )?;
        marker.validate_driver_live_tx(tx)?;
        settled.settlement().validate_terminal_unit_tx(tx)?;
        let count = a.currency.current().link_count() + 1;
        return Ok(
            validate_link_head_tx(tx, settled, count, a.kind, a.at, &a.data)?.then_some(true),
        );
    }
    if raw == a.currency.current().workflow_raw() {
        // Preimage branch: the SAME planned pre endpoint, link absent.
        validate_settled_tx(tx, &a.currency)?;
        return Ok(Some(false));
    }
    Ok(None)
}
