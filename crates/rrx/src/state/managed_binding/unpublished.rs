//! A known-unpublished rollback observation, never a Native admission proof.
//! Canonical encoding is outside SharedStore; the actual Immediate checks exact
//! current Unit bytes and absence of this allocated operation/session/invocation.
use crate::{
    domain::Scope,
    execution::{RuntimeOwner, UnitState, phase::NativeAllocation},
    state::Store,
};
use anyhow::{Result, ensure};
use rusqlite::{TransactionBehavior, params};
use std::sync::Arc;

pub(crate) struct UnpublishedMarkerPlan {
    owner: Arc<RuntimeOwner>,
    scope: Scope,
    operation: String,
    pair: String,
    session: String,
    invocation: String,
    unit: String,
    generation: u64,
    version: u64,
    body: String,
}
/// Only the Store's successfully ended Immediate can produce this observation.
pub(crate) struct UnpublishedMarkerProof(UnpublishedMarkerPlan);

pub(crate) fn plan_unpublished_marker(
    owner: &Arc<RuntimeOwner>,
    allocation: &NativeAllocation,
) -> Result<UnpublishedMarkerPlan> {
    let f = allocation.facts();
    let u = allocation.unit_snapshot();
    ensure!(
        f.state_path == owner.state_path()
            && f.state_root == owner.state_root()
            && f.instance_id == owner.instance_id()
            && f.epoch == owner.epoch()
            && u.state == UnitState::Preparing
            && u.session_id.is_none()
            && u.work.is_none()
            && u.native_effects_open
            && u.result_finalization_open,
        "unpublished marker original identity unavailable"
    );
    let body = serde_json::to_string(u)?;
    ensure!(body.len() <= 16 * 1024, "unpublished Unit exceeds bound");
    Ok(UnpublishedMarkerPlan {
        owner: owner.clone(),
        scope: f.scope.clone(),
        operation: f.operation_id.to_string(),
        pair: f.pair_id.to_string(),
        session: f.session_id.to_string(),
        invocation: f.invocation_id.to_string(),
        unit: u.id.to_string(),
        generation: u.generation,
        version: u.version,
        body,
    })
}
impl UnpublishedMarkerProof {
    pub(crate) fn matches(&self, owner: &Arc<RuntimeOwner>, allocation: &NativeAllocation) -> bool {
        let f = allocation.facts();
        Arc::ptr_eq(&self.0.owner, owner)
            && self.0.scope == *f.scope
            && self.0.operation == f.operation_id.to_string()
            && self.0.pair == f.pair_id.to_string()
            && self.0.session == f.session_id.to_string()
            && self.0.invocation == f.invocation_id.to_string()
            && self.0.unit == f.unit_id.to_string()
            && self.0.generation == f.generation
            && self.0.version == f.unit_version
    }
}
impl Store {
    /// The caller holds the SAME Runtime's control admission until restoring its
    /// slot. No concurrent actual marker publisher can enter that boundary.
    pub(crate) fn confirm_unpublished_marker(
        &mut self,
        plan: UnpublishedMarkerPlan,
    ) -> Result<UnpublishedMarkerProof> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let goal = plan
            .scope
            .goal_id
            .ok_or_else(|| anyhow::anyhow!("rollback Goal missing"))?;
        let task = plan
            .scope
            .task_id
            .ok_or_else(|| anyhow::anyhow!("rollback Task missing"))?;
        let current: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM runtime_epoch e JOIN task_execution t ON t.task_id=?1 JOIN execution_units u ON u.id=?2 WHERE e.singleton=1 AND e.instance_id=?3 AND e.epoch=?4 AND t.project_id=?5 AND t.goal_id=?6 AND t.generation=?7 AND u.project_id=t.project_id AND u.goal_id=t.goal_id AND u.task_id=t.task_id AND u.generation=t.generation AND u.owner_epoch=e.epoch AND u.version=?8 AND u.native_effects_open=1 AND u.result_finalization_open=1 AND u.body=?9)",
            params![task.to_string(),plan.unit,plan.owner.instance_id(),plan.owner.epoch(),
                plan.scope.project_id.to_string(),goal.to_string(),plan.generation,plan.version,plan.body],
            |r| r.get(0),
        )?;
        ensure!(current, "unpublished marker original Unit/epoch changed");
        let operation: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM managed_phase_operations WHERE operation_id=?1 OR pair_id=?2 OR allocated_session_id=?3)",
            params![plan.operation,plan.pair,plan.session], |r| r.get(0),
        )?;
        let invocation: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM native_invocations WHERE id=?1 OR session_id=?2)",
            params![plan.invocation, plan.session],
            |r| r.get(0),
        )?;
        let session: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM records WHERE id=?1)",
            [&plan.session],
            |r| r.get(0),
        )?;
        ensure!(
            !operation && !invocation && !session,
            "marker may be published; retain original operation"
        );
        // This transaction writes nothing. A failed end is not a rollback proof.
        tx.rollback()?;
        Ok(UnpublishedMarkerProof(plan))
    }
}
