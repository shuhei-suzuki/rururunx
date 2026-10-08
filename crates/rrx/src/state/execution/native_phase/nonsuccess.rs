//! Sealed access to retained RN-1 images; no row-derived authority.
use super::version::{InventoryBudget, RetiredUnitImage};
use super::*;
use crate::execution::native::NativeNoDispatchClosureProof;
use crate::state::managed_binding::{
    PhaseClosedFacts, PhaseClosureImages, PhaseImage, UnlinkedPhaseClosure, plan_unlinked_closure,
    validate_original_phase_tx,
};

pub(crate) struct NonSuccessReader(());
pub(crate) struct NativeNonSuccessClosurePlan {
    proof: Arc<NativeNoDispatchClosureProof>,
    images: UnlinkedPhaseClosure,
    unit_after: RetiredUnitImage,
    audit_data: String,
}
pub(crate) struct NativeNonSuccessMaterial {
    plan: Arc<NativeNonSuccessClosurePlan>,
    images: PhaseClosureImages,
}
pub(crate) struct PhaseClosedAcknowledgment {
    plan: Arc<NativeNonSuccessClosurePlan>,
}
pub(crate) enum NativeNonSuccessWrite {
    Known(PhaseClosedAcknowledgment),
    Conflict(anyhow::Error),
}
pub(crate) enum NativeNonSuccessConfirmation {
    Known(PhaseClosedAcknowledgment),
    RolledBack,
}
impl PhaseClosedAcknowledgment {
    pub(crate) fn matches_allocation(
        &self,
        allocation: &Arc<crate::execution::phase::NativeAllocation>,
    ) -> bool {
        Arc::ptr_eq(self.plan.proof.launch().allocation(), allocation)
    }
    pub(crate) fn matches_marker(
        &self,
        marker: &Arc<crate::state::managed_binding::OriginalMarker>,
    ) -> bool {
        Arc::ptr_eq(self.plan.proof.launch().marker(), marker)
    }
}
fn validate_common(tx: &Transaction<'_>, proof: &NativeNoDispatchClosureProof) -> Result<()> {
    let launch = proof.launch();
    selected_database(tx, launch)?;
    proof.validate_original()?;
    launch.validate_preparation_original()?;
    let closure = proof.closure(&NonSuccessReader(()));
    closure.validate_original(tx)?;
    closure.validate_after_tx(tx)
}
fn validate_open(tx: &Transaction<'_>, plan: &NativeNonSuccessClosurePlan) -> Result<()> {
    validate_common(tx, &plan.proof)?;
    validate_original_phase_tx(tx, plan.proof.launch().marker(), PhaseImage::Open)?;
    plan.proof
        .closure(&NonSuccessReader(()))
        .unit()
        .validate_indexed_tx(tx)?;
    plan.proof
        .closure(&NonSuccessReader(()))
        .unit()
        .validate_tx(tx)?;
    plan.images.validate_budget_tx(
        tx,
        plan.proof.launch().marker(),
        plan.audit_data.len() as u64,
    )
}
fn classify_images(
    closed: Result<()>,
    unit_after: Result<()>,
    open: impl FnOnce() -> Result<()>,
) -> Result<bool> {
    if closed.is_ok() && unit_after.is_ok() {
        return Ok(true);
    }
    open().context("uncertain non-success mixed images or currency changed; Held")?;
    Ok(false)
}
impl Store {
    pub(crate) fn plan_phase_nonsuccess_closure(
        owner: &Arc<RuntimeOwner>,
        proof: Arc<NativeNoDispatchClosureProof>,
    ) -> Result<Arc<NativeNonSuccessClosurePlan>> {
        proof.validate_original()?;
        let launch = proof.launch();
        let adapter = launch.allocation().selected_port().selected_adapter()?;
        let f = launch.allocation().facts();
        ensure!(
            Arc::ptr_eq(owner, &adapter.owner)
                && f.state_path == owner.state_path()
                && f.instance_id == owner.instance_id()
                && f.epoch == owner.epoch(),
            "non-success planner uses another owner"
        );
        let at = now_ms();
        let reader = NonSuccessReader(());
        let (images, unit_after, audit_data) = snapshot(owner, |tx| {
            let budget = InventoryBudget::new(tx)?;
            budget.finish((|| {
                validate_common(tx, &proof)?;
                let closure = proof.closure(&reader);
                closure.unit().validate_indexed_tx(tx)?;
                closure.unit().validate_tx(tx)?;
                let images = plan_unlinked_closure(&reader, tx, launch.marker(), at)?;
                let unit_after = closure.unit().plan_retired(launch, at)?;
                let (readiness_version, lease_released, probe_released) =
                    closure.payload_facts()?;
                let facts = PhaseClosedFacts {
                    allocated_session_id: f.session_id.to_string(),
                    unit_id: unit_after.unit_id().into(),
                    unit_versions: unit_after.versions(),
                    readiness_version,
                    lease_released,
                    probe_released,
                };
                let audit_data = images.audit_data(launch.marker(), &facts)?;
                images.validate_budget_tx(tx, launch.marker(), audit_data.len() as u64)?;
                Ok((images, unit_after, audit_data))
            })())
        })?;
        let plan = Arc::new(NativeNonSuccessClosurePlan {
            proof,
            images,
            unit_after,
            audit_data,
        });
        Ok(plan)
    }
    pub(crate) fn materialize_phase_nonsuccess(
        plan: &Arc<NativeNonSuccessClosurePlan>,
    ) -> Result<NativeNonSuccessMaterial> {
        crate::runtime::assert_nonsuccess_unlocked();
        plan.proof.validate_original()?;
        Ok(NativeNonSuccessMaterial {
            plan: plan.clone(),
            images: plan.images.materialize(plan.proof.launch().marker())?,
        })
    }
    pub(crate) fn close_phase_nonsuccess(
        &mut self,
        material: &NativeNonSuccessMaterial,
    ) -> Result<NativeNonSuccessWrite> {
        crate::runtime::record_nonsuccess_store_attempt();
        let plan = &material.plan;
        if let Err(cause) = selected_database(&self.connection, plan.proof.launch()) {
            return Ok(NativeNonSuccessWrite::Conflict(cause));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        {
            let budget = InventoryBudget::new(&tx)?;
            if let Err(cause) = budget.finish(validate_open(&tx, plan)) {
                return Ok(NativeNonSuccessWrite::Conflict(cause));
            }
        }
        let reader = NonSuccessReader(());
        material.images.write_tx(
            &reader,
            &tx,
            &self.binding_permits,
            plan.proof.launch().marker(),
            plan.images.link(&plan.audit_data),
            |tx| {
                plan.proof
                    .closure(&reader)
                    .unit()
                    .write_retired_tx(tx, &plan.unit_after)
            },
        )?;
        tx.commit()?;
        Ok(NativeNonSuccessWrite::Known(PhaseClosedAcknowledgment {
            plan: plan.clone(),
        }))
    }
    pub(crate) fn confirm_phase_nonsuccess(
        &mut self,
        material: &NativeNonSuccessMaterial,
    ) -> Result<NativeNonSuccessConfirmation> {
        crate::runtime::record_nonsuccess_store_attempt();
        let plan = &material.plan;
        selected_database(&self.connection, plan.proof.launch())?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let known = {
            let budget = InventoryBudget::new(&tx)?;
            budget.finish((|| {
                validate_common(&tx, &plan.proof)?;
                let closed = validate_original_phase_tx(
                    &tx,
                    plan.proof.launch().marker(),
                    PhaseImage::Closed {
                        images: &material.images,
                        at: plan.images.at(),
                        data: &plan.audit_data,
                    },
                );
                classify_images(closed, plan.unit_after.validate_indexed_tx(&tx), || {
                    validate_open(&tx, plan)
                })
            })())?
        };
        tx.commit()?;
        Ok(if known {
            NativeNonSuccessConfirmation::Known(PhaseClosedAcknowledgment { plan: plan.clone() })
        } else {
            NativeNonSuccessConfirmation::RolledBack
        })
    }
}

#[cfg(test)]
mod primitives {
    use super::*;
    #[test]
    fn nongrant_rn1_confirmation_requires_complete_post_or_complete_pre_images() {
        // Truth-table control of the actual port classifier. No proof or SQL
        // currency is constructed; genuine per-parent cases remain SETUP.
        for mask in 0u8..16 {
            let predicate = |value| {
                if value {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("nongrant image differs"))
                }
            };
            let unit_post = mask & 1 != 0;
            let operation_post = mask & 2 != 0;
            let workflow_post = mask & 4 != 0;
            let link_post = mask & 8 != 0;
            let result = classify_images(
                predicate(operation_post && workflow_post && link_post),
                predicate(unit_post),
                || predicate(mask == 0),
            );
            match mask {
                15 => assert!(result.unwrap()),
                0 => assert!(!result.unwrap()),
                _ => assert!(result.is_err(), "accepted mixed image mask {mask}"),
            }
        }
    }
}
