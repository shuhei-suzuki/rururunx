//! Managed native Session planning and schema protection.
//!
//! The marker/binding producer and Native protocol are not yet composed. These
//! bounded read primitives must not enable Workflow or replace its private proof.
mod activation;
#[cfg(test)]
pub(crate) use activation::ActivationSeams;
mod binding;
pub(crate) use activation::{
    ActivationCommit, NativeActivationPlan, member_digest, plan_native_activation, roster_digest,
};
mod closure;
mod gate;
pub(in crate::state) use closure::{
    PhaseClosedFacts, PhaseClosureImages, PhaseImage, UnlinkedPhaseClosure, plan_unlinked_closure,
    validate_original_phase_tx,
};
mod canonical;
pub(in crate::state) use canonical::Body;
pub(in crate::state) use marker_plan::unit_index_matches as unit_image_matches;
mod marker_plan;
mod marker_rows;
pub(in crate::state) use marker_rows::record_image;
mod permits;
mod protection;
mod publication;
#[cfg(test)]
pub(crate) use publication::{arm_driver_publication_fault, check_native_contract_integrity};
mod schema;
#[cfg(test)]
mod schema_tests;
mod session_identity;
mod snapshot;
mod success;
mod successor;
mod unpublished;
pub(super) use permits::{
    ExactRowMutation, PrivatePermitManager, phase_pair_columns, register_permit_function,
};
#[cfg(test)]
pub(super) use schema::TABLES;
pub(super) use schema::{
    hold_existing_workflows, install_retained_guards, install_schema, validate_current_layout,
    validate_legacy_namespace,
};

pub(crate) use crate::runtime::installation::{ActivationRoster, InstalledDriverComposition};
pub(crate) use crate::runtime::phase_supervisor::{PhaseLaunch, PhaseLaunchParts};
pub(crate) use binding::{
    BindingAcknowledgment, ManagedBindingConfirmation, ManagedBindingPlan, ManagedBindingWrite,
    plan_late_binding, plan_managed_binding,
};
pub(crate) use gate::{
    GATE_FAILED_DETAIL, GATE_UNKNOWN_DETAIL, GateClaimAcknowledgment, GateClaimPlan,
    GateObservedAcknowledgment, GateObservedPlan, SUCCESS_CLOSURE_HEADROOM, SuccessConfirmation,
    SuccessWrite,
};
pub(crate) use marker_plan::ManagedMarkerPlan;
pub(crate) use publication::{
    MarkerPublicationOutcome, MarkerPublicationPlan, MarkerTransactionObservation, OriginalMarker,
    plan_marker_publication,
};
#[cfg(test)]
pub(crate) use snapshot::plan_scope;
pub(in crate::state) use snapshot::{ScopePlan, read_scope, snapshot};
pub(in crate::state) use success::validate_settled_tx;
pub(crate) use success::{SettledCurrency, SettledPhase, plan_settled_currency};
pub(in crate::state) use successor::validate_planned_unit_tx;
pub(crate) use successor::{CurrentWorkflowSuccessor, plan_current_phase, validate_current_tx};
pub(crate) use unpublished::{UnpublishedMarkerProof, plan_unpublished_marker};

#[cfg(test)]
mod tests;
