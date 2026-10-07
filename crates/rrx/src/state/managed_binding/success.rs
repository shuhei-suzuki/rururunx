//! Successful original-phase continuation (SC). Every product here derives
//! from the SAME owner's sealed settlement and the binder's acknowledgment;
//! no row, receipt ID, DTO or serialized value constructs one.
use super::{binding::BindingAcknowledgment, publication::OriginalMarker};
use crate::{
    execution::{
        native::{NativePhaseBinding, OwnedPhaseSettlement},
        phase::NativeAllocation,
    },
    workflow::{Phase, WorkflowSnapshot},
};
use anyhow::{Context, Result, ensure};
use std::sync::Arc;

/// A bound owned success: the snapshot whose settlement is present and the
/// acknowledgment of the SAME owner's committed `session_bound` link.
pub(crate) struct SettledPhase {
    binding: Arc<NativePhaseBinding>,
    bound: Arc<BindingAcknowledgment>,
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
        Ok(Arc::new(Self {
            binding,
            bound,
            phase,
        }))
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
    pub(crate) fn binding(&self) -> &Arc<NativePhaseBinding> {
        &self.binding
    }
    pub(crate) fn bound(&self) -> &Arc<BindingAcknowledgment> {
        &self.bound
    }
    /// The marker's active attempt.
    pub(crate) fn phase(&self) -> Phase {
        self.phase
    }
}
