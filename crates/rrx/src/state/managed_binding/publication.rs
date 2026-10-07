//! Actual single-transaction marker publication from retained real producers.
//! No row/DTO/capability callback can construct the known-commit marker below.
use super::{
    canonical::Body,
    marker_plan::{ManagedMarkerPlan, plan_marker},
    marker_rows::{LINK_RESERVE_BYTES, MarkerRows, WORKFLOW_BYTES, workflow_mutation},
    snapshot::snapshot,
};
use crate::{
    domain::{RecordId, Scope},
    execution::{ExecutionUnit, RuntimeOwner, phase::NativeAllocation},
    state::{
        Store,
        runtime::driver::{DriverMarkerAdvance, DriverReadTicket},
    },
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Transaction, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Complete activation encoding, checked outside the Store mutex. Successful
/// decoding is nongrant: the real installed activation must already exist.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct NativeContract {
    workflow_id: RecordId,
    project_id: crate::domain::ProjectId,
    goal_id: crate::domain::GoalId,
    task_id: crate::domain::TaskId,
    owner_epoch: u64,
    origin: uuid::Uuid,
    profile_digest: String,
    contract_state: String,
    version: u64,
}

/// Strongly retained before the writer begins. Original captures are never
/// replaced on commit uncertainty and cannot be rebuilt from operation rows.
pub(crate) struct MarkerPublicationPlan {
    owner: Arc<RuntimeOwner>,
    allocation: Arc<NativeAllocation>,
    marker: ManagedMarkerPlan,
    driver: DriverMarkerAdvance,
    rows: MarkerRows,
    contract_body: String,
}
/// Produced ONLY after this module's actual Immediate commit returns success.
/// Field privacy, no Deserialize and no public constructor preserve provenance.
pub(crate) struct OriginalMarker {
    plan: Arc<MarkerPublicationPlan>,
}

/// Factual observations only. An empty cell or commit error never proves that
/// the marker rolled back; only the original unpublished protocol may do so.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MarkerTransactionObservation {
    Unobserved,
    NotConstructedRefused,
    PrecommitDropped,
    CommitAttempted,
    CommitError,
    Committed,
}
struct PublicationFacts {
    transaction: MarkerTransactionObservation,
    confirmed: bool,
    first_marker: Option<Arc<OriginalMarker>>,
}
/// Independently retained beside the original plan before admission. Runtime
/// can read these facts; only this actual Store producer can advance them.
pub(crate) struct MarkerPublicationOutcome {
    plan: Arc<MarkerPublicationPlan>,
    facts: Mutex<PublicationFacts>,
}
impl MarkerPublicationOutcome {
    pub(crate) fn new(plan: Arc<MarkerPublicationPlan>) -> Arc<Self> {
        Arc::new(Self {
            plan,
            facts: Mutex::new(PublicationFacts {
                transaction: MarkerTransactionObservation::Unobserved,
                confirmed: false,
                first_marker: None,
            }),
        })
    }
    pub(crate) fn transaction(&self) -> Result<MarkerTransactionObservation> {
        Ok(self
            .facts
            .lock()
            .map_err(|_| anyhow::anyhow!("marker outcome poisoned"))?
            .transaction)
    }
    pub(crate) fn first_marker(&self) -> Result<Option<Arc<OriginalMarker>>> {
        Ok(self
            .facts
            .lock()
            .map_err(|_| anyhow::anyhow!("marker outcome poisoned"))?
            .first_marker
            .clone())
    }
    pub(crate) fn is_confirmed(&self) -> Result<bool> {
        Ok(self
            .facts
            .lock()
            .map_err(|_| anyhow::anyhow!("marker outcome poisoned"))?
            .confirmed)
    }
    fn validate_plan(&self, plan: &Arc<MarkerPublicationPlan>) -> Result<()> {
        ensure!(
            Arc::ptr_eq(&self.plan, plan),
            "different original outcome plan"
        );
        Ok(())
    }
    fn observe(&self, plan: &Arc<MarkerPublicationPlan>) -> Result<TransactionObserver<'_>> {
        self.validate_plan(plan)?;
        ensure!(
            self.transaction()? == MarkerTransactionObservation::Unobserved,
            "original marker transaction already observed"
        );
        Ok(TransactionObserver {
            outcome: self,
            constructed: false,
            commit_called: false,
        })
    }
    fn retain_transaction(&self, observation: MarkerTransactionObservation) {
        // Poison recovery preserves an actual observation; it grants no write,
        // handoff or permission to repeat the transaction.
        self.facts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .transaction = observation;
    }
    fn retain_confirmation(&self) {
        self.facts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .confirmed = true;
    }
    fn retain_first_marker(&self) -> Arc<OriginalMarker> {
        let mut facts = self.facts.lock().unwrap_or_else(|e| e.into_inner());
        facts
            .first_marker
            .get_or_insert_with(|| {
                Arc::new(OriginalMarker {
                    plan: self.plan.clone(),
                })
            })
            .clone()
    }
}
/// Constructed before the actual transaction, so precommit error/unwind drops
/// the transaction before this observer records that control-flow boundary.
struct TransactionObserver<'a> {
    outcome: &'a MarkerPublicationOutcome,
    constructed: bool,
    commit_called: bool,
}
impl TransactionObserver<'_> {
    fn commit_attempted(&mut self) {
        self.commit_called = true;
        self.outcome
            .retain_transaction(MarkerTransactionObservation::CommitAttempted);
    }
    fn commit_returned(&self, success: bool) {
        self.outcome.retain_transaction(if success {
            MarkerTransactionObservation::Committed
        } else {
            MarkerTransactionObservation::CommitError
        });
    }
}
impl Drop for TransactionObserver<'_> {
    fn drop(&mut self) {
        if !self.commit_called {
            self.outcome.retain_transaction(if self.constructed {
                MarkerTransactionObservation::PrecommitDropped
            } else {
                MarkerTransactionObservation::NotConstructedRefused
            });
        }
    }
}
impl OriginalMarker {
    pub(super) fn validate_unadvanced_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        self.plan.rows.validate_unadvanced_tx(tx)
    }
    pub(super) fn original_operation_image(&self) -> Result<&[rusqlite::types::Value]> {
        self.plan.rows.original_operation_image()
    }
    pub(super) fn validate_closed_tx(
        &self,
        tx: &Transaction<'_>,
        operation: &[rusqlite::types::Value],
    ) -> Result<()> {
        self.plan.rows.validate_closed_tx(tx, operation)
    }
    /// Immutable original lineage; copied rows or a fresh ticket cannot match.
    /// Current successor, live Driver and actual Native eligibility are separate.
    pub(crate) fn matches_source_ticket(&self, ticket: &Arc<DriverReadTicket>) -> bool {
        self.plan.driver.matches_source_ticket(ticket)
    }
    pub(super) fn publication_plan(&self) -> Arc<MarkerPublicationPlan> {
        self.plan.clone()
    }
    pub(super) fn validate_open_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        self.plan.rows.validate_open_tx(tx)
    }
    /// Recheck the SAME retained Driver/Source post anchor, never a row-derived
    /// replacement ticket. Callers must also validate current phase facts and
    /// their actual Native owner in this transaction before a new effect.
    pub(crate) fn validate_driver_live_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        ensure!(
            self.plan
                .owner
                .state_path()
                .to_str()
                .is_some_and(|path| tx.path() == Some(path)),
            "original marker Driver validation uses another database"
        );
        self.plan.driver.validate_live_tx(tx)
    }
    pub(crate) fn matches_original_plan(&self, plan: &Arc<MarkerPublicationPlan>) -> bool {
        Arc::ptr_eq(&self.plan, plan)
    }
    pub(crate) fn allocation(&self) -> &Arc<NativeAllocation> {
        &self.plan.allocation
    }
    pub(crate) fn scope(&self) -> Scope {
        self.plan.marker.scope()
    }
    pub(crate) fn operation(&self) -> crate::execution::OperationId {
        self.plan.marker.operation
    }
    pub(crate) fn frame_digest(&self) -> &str {
        &self.plan.marker.frame_digest
    }
    pub(crate) fn unit(&self) -> &ExecutionUnit {
        self.plan.marker.unit.parsed()
    }
    pub(crate) fn original_plan(&self) -> &ManagedMarkerPlan {
        &self.plan.marker
    }
}

pub(crate) fn plan_marker_publication(
    owner: Arc<RuntimeOwner>,
    allocation: Arc<NativeAllocation>,
    ticket: Arc<DriverReadTicket>,
    workflow: RecordId,
) -> Result<Arc<MarkerPublicationPlan>> {
    let marker = plan_marker(&owner, workflow, &allocation)?;
    let driver = ticket.plan_marker_advance(&marker)?;
    let rows = MarkerRows::plan(&marker, &allocation)?;
    // This is the actual existing activation contract. No configured alias,
    // selected CLI or successful read synthesizes a composed activation.
    let contract_body = snapshot(&owner, |tx| {
        let f = allocation.facts();
        let raw: Option<String> = tx.query_row(
            "SELECT CASE WHEN length(CAST(body AS BLOB))<=4096 THEN body END FROM workflow_native_contracts WHERE workflow_id=?1 AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND owner_epoch=?5 AND origin=?6 AND profile_digest=?7 AND contract_state='composed' AND version=1",
            params![workflow.to_string(), f.scope.project_id.to_string(), f.scope.goal_id.context("marker Goal absent")?.to_string(), f.scope.task_id.context("marker Task absent")?.to_string(), f.epoch, f.origin_id.to_string(), f.profile_digest],
            |r| r.get(0),
        )?;
        let body = Body::<NativeContract>::decode(
            raw.context("actual composed Workflow activation unavailable")?,
            4096,
        )?;
        let contract = body.parsed();
        ensure!(
            contract.workflow_id == workflow
                && contract.project_id == f.scope.project_id
                && Some(contract.goal_id) == f.scope.goal_id
                && Some(contract.task_id) == f.scope.task_id
                && contract.owner_epoch == f.epoch
                && contract.origin == f.origin_id
                && contract.profile_digest == f.profile_digest
                && contract.contract_state == "composed"
                && contract.version == 1,
            "actual Workflow activation body/index differs"
        );
        Ok(body.raw().to_owned())
    })?;
    Ok(Arc::new(MarkerPublicationPlan {
        owner,
        allocation,
        marker,
        driver,
        rows,
        contract_body,
    }))
}
impl MarkerPublicationPlan {
    /// Immutable original-object linkage, never a currency or effect grant.
    pub(crate) fn matches_source_ticket(&self, ticket: &Arc<DriverReadTicket>) -> bool {
        self.driver.matches_source_ticket(ticket)
    }
    pub(crate) fn allocation(&self) -> &Arc<NativeAllocation> {
        &self.allocation
    }
    fn validate_contract(&self, tx: &Transaction<'_>) -> Result<()> {
        let f = self.allocation.facts();
        let exact: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM workflow_native_contracts WHERE workflow_id=?1 AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND owner_epoch=?5 AND origin=?6 AND profile_digest=?7 AND contract_state='composed' AND version=1 AND body=?8)",
            params![self.marker.workflow_after.parsed().id.to_string(), f.scope.project_id.to_string(), f.scope.goal_id.context("marker Goal absent")?.to_string(), f.scope.task_id.context("marker Task absent")?.to_string(), f.epoch, f.origin_id.to_string(), f.profile_digest, self.contract_body],
            |r| r.get(0),
        )?;
        ensure!(exact, "original Workflow activation changed");
        Ok(())
    }
}

/// Complete scoped encoded body costs and open audit reservations. Aggregate
/// queries copy no bodies; every surface has an explicit row/byte refusal bound.
/// This is accounting, not a Native/Driver permission or a latency guarantee.
pub(super) fn charged_scope_bytes(tx: &Transaction<'_>, scope: &Scope) -> Result<u64> {
    let p = scope.project_id.to_string();
    let g = scope.goal_id.context("budget Goal missing")?.to_string();
    let t = scope.task_id.context("budget Task missing")?.to_string();
    let mut used = 0u64;
    // Compiled table names only, all values parameterized. This includes the
    // entire Task history rather than hiding older operation/Session costs.
    for table in [
        "records",
        "context_versions",
        "execution_units",
        "managed_phase_operations",
        "managed_phase_owners",
        "managed_phase_inputs",
        "workflow_native_contracts",
        "task_drivers",
        "source_recoveries",
        "native_invocations",
        "native_results",
        "scoped_session_identities",
        "result_artifacts",
        "resource_leases",
        "managed_effects",
        "verification_runs",
    ] {
        let (count, bytes): (u64, u64) = tx.query_row(
            &format!("SELECT count(*),COALESCE(sum(bytes),0) FROM (SELECT length(CAST(body AS BLOB)) bytes FROM {table} WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 LIMIT 4097)"),
            params![p, g, t], |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        ensure!(
            count <= 4096 && bytes <= WORKFLOW_BYTES,
            "complete marker budget surface exceeds bound"
        );
        used = used.checked_add(bytes).context("marker budget overflow")?;
    }
    for table in [
        "managed_marker_bodies",
        "managed_phase_admissions",
        "managed_phase_readiness",
    ] {
        let (count, bytes): (u64, u64) = tx.query_row(
            &format!("SELECT count(*),COALESCE(sum(bytes),0) FROM (SELECT length(CAST(s.body AS BLOB)) bytes FROM {table} s JOIN managed_phase_operations o ON o.operation_id=s.operation_id WHERE o.project_id=?1 AND o.goal_id=?2 AND o.task_id=?3 LIMIT 4097)"),
            params![p, g, t], |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        ensure!(
            count <= 4096 && bytes <= WORKFLOW_BYTES,
            "complete marker budget relation exceeds bound"
        );
        used = used.checked_add(bytes).context("marker budget overflow")?;
    }
    let (count, audit_bytes): (u64, u64) = tx.query_row(
        "SELECT count(*),COALESCE(sum(bytes),0) FROM (SELECT length(CAST(data AS BLOB)) bytes FROM audit WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind GLOB 'rrx.private.workflow.*' LIMIT 65537)",
        params![p, g, t], |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    ensure!(
        count <= 65536 && audit_bytes <= WORKFLOW_BYTES,
        "complete marker audit budget exceeds bound"
    );
    used = used
        .checked_add(audit_bytes)
        .context("marker budget overflow")?;
    let (open, reserved): (u64, u64) = tx.query_row(
        "WITH spent AS (SELECT json_extract(data,'$.private_operation_ref') operation_id,sum(length(CAST(data AS BLOB))) bytes FROM audit WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind GLOB 'rrx.private.workflow.*' GROUP BY json_extract(data,'$.private_operation_ref')) SELECT count(*),COALESCE(sum(max(0,1048576-COALESCE(s.bytes,0))),0) FROM managed_phase_operations o LEFT JOIN spent s ON s.operation_id=o.operation_id WHERE o.project_id=?1 AND o.goal_id=?2 AND o.task_id=?3 AND o.phase_open=1",
        params![p, g, t], |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    ensure!(
        open <= 128 && reserved <= WORKFLOW_BYTES,
        "open marker audit reservation exceeds bound"
    );
    used.checked_add(reserved)
        .context("complete scope budget overflow")
}

fn reserve_budget(tx: &Transaction<'_>, plan: &MarkerPublicationPlan) -> Result<()> {
    let required = charged_scope_bytes(tx, &plan.marker.scope())?
        .checked_add(plan.rows.durable_bytes())
        .and_then(|v| v.checked_add(LINK_RESERVE_BYTES))
        // Conservative full post-image charge, never an optimistic subtraction
        // of a previous body or a reservation belonging to another operation.
        .and_then(|v| v.checked_add(plan.marker.task_after.raw().len() as u64))
        .and_then(|v| v.checked_add(plan.marker.workflow_after.raw().len() as u64))
        // Both private Driver and optional Source freeze encode at most128KiB.
        // Charge both maxima even when no Source row is written, so the marker
        // cannot hide those new durable bodies behind its audit reservation.
        .and_then(|v| v.checked_add(2 * 128 * 1024))
        .context("marker reservation overflow")?;
    ensure!(
        required <= WORKFLOW_BYTES,
        "mandatory Workflow audit reserve unavailable"
    );
    Ok(())
}

impl Store {
    /// Only the Runtime's SAME retained pre-transaction plan may reconcile an
    /// uncertain marker. No current-row capture, new marker or Native replay.
    /// This initial reconciliation is deliberately unavailable after handoff.
    pub(crate) fn confirm_retained_marker(
        &mut self,
        plan: &Arc<MarkerPublicationPlan>,
        outcome: &MarkerPublicationOutcome,
    ) -> Result<Arc<OriginalMarker>> {
        outcome.validate_plan(plan)?;
        ensure!(
            plan.owner
                .state_path()
                .to_str()
                .is_some_and(|path| self.connection.path() == Some(path)),
            "marker reconciliation uses another database"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        plan.marker.before.validate_projection(
            &tx,
            plan.marker.task_after.parsed(),
            plan.marker.task_after.raw(),
            Some((
                plan.marker.workflow_after.parsed(),
                plan.marker.workflow_after.raw(),
            )),
        )?;
        plan.validate_contract(&tx)?;
        plan.rows.validate_tx(&tx)?;
        let facts = plan.allocation.facts();
        let original_unit: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM execution_units WHERE id=?1 AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND generation=?5 AND owner_epoch=?6 AND version=?7 AND body=?8)",
            params![facts.unit_id.to_string(),facts.scope.project_id.to_string(),facts.scope.goal_id.context("marker Goal missing")?.to_string(),facts.scope.task_id.context("marker Task missing")?.to_string(),facts.generation,facts.epoch,facts.unit_version,plan.marker.unit.raw()],
            |r| r.get(0),
        )?;
        ensure!(original_unit, "original pre-handoff Unit changed");
        let dispatched: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM records WHERE id=?1) OR EXISTS(SELECT 1 FROM session_units WHERE session_id=?1 OR unit_id=?2) OR EXISTS(SELECT 1 FROM native_invocations WHERE id=?3 OR unit_id=?2) OR EXISTS(SELECT 1 FROM managed_phase_admissions WHERE operation_id=?4 OR pair_id=?5)",
            params![facts.session_id.to_string(),facts.unit_id.to_string(),facts.invocation_id.to_string(),facts.operation_id.to_string(),facts.pair_id.to_string()],
            |r| r.get(0),
        )?;
        ensure!(
            !dispatched,
            "retained marker already has Native admission/history"
        );
        tx.commit()?;
        outcome.retain_confirmation();
        // Still under SAME SharedStore exclusion: this private advance checks
        // actual selection/Unit/Source and exact committed Driver bytes before
        // accepting its original cache or its own identical planned post-cache.
        self.publish_driver_marker(&plan.driver)?;
        Ok(outcome.retain_first_marker())
    }
    /// Caller is the real Runtime under control admission and retains this exact
    /// plan in its charged slot BEFORE entry. No async/IO/public callbacks here.
    pub(crate) fn publish_managed_marker(
        &mut self,
        plan: &Arc<MarkerPublicationPlan>,
        outcome: &MarkerPublicationOutcome,
    ) -> Result<Arc<OriginalMarker>> {
        let mut observed = outcome.observe(plan)?;
        ensure!(
            plan.owner
                .state_path()
                .to_str()
                .is_some_and(|path| self.connection.path() == Some(path)),
            "marker writer is not the selected owner's database"
        );
        let mut writes = plan.rows.exact_mutations()?;
        writes.push(workflow_mutation(&plan.marker)?);
        writes.extend(plan.driver.exact_mutations()?);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        observed.constructed = true;
        // Original old-frame checks MUST precede initial Task/W writes.
        plan.marker.validate_current(&tx)?;
        plan.driver.validate_current_tx(&tx)?;
        plan.validate_contract(&tx)?;
        reserve_budget(&tx, plan)?;
        let before_task = plan.marker.before.task.parsed();
        let after_task = plan.marker.task_after.parsed();
        let before_w = plan
            .marker
            .before
            .workflow
            .as_ref()
            .context("marker Workflow missing")?;
        let after_w = &plan.marker.workflow_after;
        self.binding_permits.with_exact_permit(writes, || {
            ensure!(tx.execute(
                "UPDATE tasks SET version=?1,body=?2 WHERE id=?3 AND project_id=?4 AND goal_id=?5 AND issue IS ?6 AND version=?7 AND body=?8",
                params![after_task.version, plan.marker.task_after.raw(), before_task.id.to_string(), before_task.project_id.to_string(), before_task.goal_id.to_string(), before_task.issue, before_task.version, plan.marker.before.task.raw()],
            )? == 1, "initial marker Task CAS changed");
            ensure!(tx.execute(
                "UPDATE records SET version=?1,body=?2 WHERE id=?3 AND kind='workflow' AND project_id=?4 AND goal_id=?5 AND task_id=?6 AND version=?7 AND body=?8",
                params![after_w.parsed().version, after_w.raw(), before_w.parsed().id.to_string(), before_task.project_id.to_string(), before_task.goal_id.to_string(), before_task.id.to_string(), before_w.parsed().version, before_w.raw()],
            )? == 1, "initial marker Workflow CAS changed");
            plan.rows.write_tx(&tx)?;
            plan.driver.freeze_marker_tx(&tx)?;
            self.binding_permits.ensure_consumed()?;
            observed.commit_attempted();
            let committed = tx.commit();
            observed.commit_returned(committed.is_ok());
            committed?;
            Ok(())
        })?;
        // A publication failure after commit never creates a replacement plan
        // or rolls the marker back. Runtime keeps this original pending slot.
        self.publish_driver_marker(&plan.driver)?;
        Ok(outcome.retain_first_marker())
    }
}
