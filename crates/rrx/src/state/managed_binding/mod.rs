//! Managed phase planning. Read snapshots and digests never grant dispatch.
//!
//! The marker/binding producer and Native protocol are not yet composed. These
//! bounded read primitives must not enable Workflow or replace its private proof.
mod canonical;
mod protection;
mod snapshot;

pub(crate) use snapshot::plan_scope;

#[cfg(test)]
mod tests;
