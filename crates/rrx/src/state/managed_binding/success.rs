//! Successful original-phase continuation (SC). Every product here derives
//! from the SAME owner's sealed settlement and the binder's acknowledgment;
//! no row, receipt ID, DTO or serialized value constructs one.
use super::{
    binding::BindingAcknowledgment,
    publication::OriginalMarker,
    successor::{CurrentWorkflowSuccessor, plan_current_phase, validate_current_tx},
};
use crate::{
    execution::{
        RuntimeOwner,
        native::{NativePhaseBinding, OwnedPhaseSettlement},
        phase::NativeAllocation,
    },
    workflow::{Phase, WorkflowSnapshot},
};
use anyhow::{Context, Result, ensure};
use rusqlite::Transaction;
use std::sync::Arc;

/// A bound owned success: the snapshot whose settlement is present and the
/// acknowledgment of the SAME owner's committed `session_bound` link.
pub(crate) struct SettledPhase {
    binding: Arc<NativePhaseBinding>,
    phase: Phase,
}
impl SettledPhase {
    /// The binding comes only from `NativePhaseSession::binding_snapshot` and
    /// the acknowledgment only from the binder; this checks pointers only.
    pub(crate) fn issue(
        binding: Arc<NativePhaseBinding>,
        bound: Arc<BindingAcknowledgment>,
    ) -> Result<Arc<Self>> {
        let settlement = binding
            .settlement()
            .context("settled phase requires the owner's settlement")?;
        ensure!(
            bound.matches_owner(binding.owner())
                && settlement.belongs_to(binding.owner())
                && std::ptr::eq(bound.marker(), binding.marker())
                && std::ptr::eq(settlement.allocation(), binding.allocation()),
            "settled phase owner, marker or allocation differs"
        );
        let workflow: WorkflowSnapshot = serde_json::from_value(
            binding
                .marker()
                .original_plan()
                .workflow_after()
                .0
                .data
                .clone(),
        )?;
        let phase = workflow
            .active
            .and_then(|index| workflow.history.get(index))
            .context("settled phase marker attempt absent")?
            .phase;
        Ok(Arc::new(Self { binding, phase }))
    }
    pub(crate) fn marker(&self) -> &OriginalMarker {
        self.binding.marker()
    }
    pub(crate) fn settlement(&self) -> &OwnedPhaseSettlement {
        self.binding
            .settlement()
            .expect("issued only with a settlement")
    }
    pub(crate) fn allocation(&self) -> &Arc<NativeAllocation> {
        self.binding.marker().allocation()
    }
    /// The marker's active attempt.
    pub(crate) fn phase(&self) -> Phase {
        self.phase
    }
}

/// Query-only planned currency of a settled phase at one ledger endpoint;
/// nongrant. Planned once per stage outside the Store mutex and reused.
pub(crate) struct SettledCurrency {
    settled: Arc<SettledPhase>,
    current: CurrentWorkflowSuccessor,
}
impl SettledCurrency {
    pub(crate) fn settled(&self) -> &Arc<SettledPhase> {
        &self.settled
    }
    pub(in crate::state) fn current(&self) -> &CurrentWorkflowSuccessor {
        &self.current
    }
}
pub(crate) fn plan_settled_currency(
    owner: &RuntimeOwner,
    settled: &Arc<SettledPhase>,
) -> Result<SettledCurrency> {
    let current = plan_current_phase(owner, settled.marker())?;
    let unit = settled.settlement().unit();
    let mut decoded = current.unit().clone();
    decoded.cleanup = unit.cleanup;
    ensure!(
        serde_json::to_value(&decoded)? == serde_json::to_value(unit)?,
        "settled currency Unit differs from the terminal postimage"
    );
    Ok(SettledCurrency {
        settled: settled.clone(),
        current,
    })
}
/// The single protected replacement for generic finalize authority on a
/// marker-bound open phase. Exact rechecks only: no decode, encode or hash.
pub(in crate::state) fn validate_settled_tx(
    tx: &Transaction<'_>,
    c: &SettledCurrency,
) -> Result<()> {
    let marker = c.settled.marker();
    marker.validate_driver_live_tx(tx)?;
    validate_current_tx(tx, marker, &c.current)?;
    let settlement = c.settled.settlement();
    settlement.validate_terminal_unit_tx(tx)?;
    let unit = settlement.unit();
    ensure!(
        unit.result_finalization_open
            && !unit.native_effects_open
            && unit.session_id == Some(c.settled.allocation().facts().session_id),
        "settled Unit finalization, effects or Session differ"
    );
    Ok(())
}
