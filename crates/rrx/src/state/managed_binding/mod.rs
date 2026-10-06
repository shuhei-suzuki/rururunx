//! Managed phase planning. Read snapshots and digests never grant dispatch.
//!
//! The marker/binding producer and Native protocol are not yet composed. These
//! bounded read primitives must not enable Workflow or replace its private proof.
mod canonical;
mod marker_plan;
mod permits;
mod protection;
mod schema;
#[cfg(test)]
mod schema_tests;
mod snapshot;
pub(super) use permits::{PrivatePermitManager, register_permit_function};
#[cfg(test)]
pub(super) use schema::TABLES;
pub(super) use schema::{
    hold_existing_workflows, install_schema, validate_current_layout, validate_legacy_namespace,
};

pub(crate) use marker_plan::plan_marker;

pub(crate) use snapshot::plan_scope;

#[cfg(test)]
mod tests;
