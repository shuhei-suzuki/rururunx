//! Managed native Session planning and schema protection.
//!
//! The marker/binding producer and Native protocol are not yet composed. These
//! bounded read primitives must not enable Workflow or replace its private proof.
mod canonical;
mod composition;
mod marker_plan;
mod permits;
mod protection;
mod schema;
#[cfg(test)]
mod schema_tests;
mod snapshot;
mod unpublished;
pub(super) use permits::{ExactRowMutation, PrivatePermitManager, register_permit_function};
#[cfg(test)]
pub(super) use schema::TABLES;
pub(super) use schema::{
    hold_existing_workflows, install_retained_guards, install_schema, validate_current_layout,
    validate_legacy_namespace,
};

pub(crate) use composition::InstalledDriverComposition;
pub(crate) use marker_plan::{ManagedMarkerPlan, plan_marker};
pub(crate) use snapshot::plan_scope;
pub(in crate::state) use snapshot::{ScopePlan, read_scope, snapshot};
pub(crate) use unpublished::{UnpublishedMarkerProof, plan_unpublished_marker};

#[cfg(test)]
mod tests;
