//! Exact first readiness transition from the same actual Native actor. No
//! Session/input/helper authority is produced by this factual write.
use super::*;
use crate::{
    execution::native::NativePreparationActor,
    runtime::phase_effect_admission::PhaseEffectAdmissionGuard,
};

pub(crate) struct NativePreparationPlan {
    actor: Arc<NativePreparationActor>,
    current: CurrentWorkflowSuccessor,
    owner_before: PairRow,
    readiness_before: PairRow,
    readiness_after: PairRow,
}
pub(crate) struct NativePreparationCommit {
    original: Arc<NativePreparationPlan>,
}
impl NativePreparationCommit {
    pub(crate) fn matches_plan(&self, plan: &Arc<NativePreparationPlan>) -> bool {
        Arc::ptr_eq(&self.original, plan)
    }
}
impl NativePreparationPlan {
    pub(crate) fn actor(&self) -> &Arc<NativePreparationActor> {
        &self.actor
    }
    pub(crate) fn matches_actor(&self, actor: &Arc<NativePreparationActor>) -> bool {
        Arc::ptr_eq(&self.actor, actor)
    }
    fn validate_common(&self, tx: &Transaction<'_>) -> Result<()> {
        self.actor.validate_original()?;
        let launch = self.actor.launch();
        selected_database(tx, launch)?;
        launch.validate_preparation_origin_tx(tx, &self.current)?;
        registration_unit(self.current.unit(), launch)?;
        no_registration(tx, launch)?;
        self.owner_before.validate_tx(tx)?;
        Ok(())
    }
}

impl Store {
    /// A separate coherent readonly snapshot is bounded by the original Root
    /// frame and finite Native pair profile; never encode under SharedStore.
    pub(crate) fn plan_native_preparation(
        runtime: &crate::execution::RuntimeOwner,
        actor: Arc<NativePreparationActor>,
    ) -> Result<Arc<NativePreparationPlan>> {
        actor.validate_open()?;
        let launch = actor.launch();
        ensure!(launch.is_retained(), "Native preparation retention ended");
        let current = plan_current_phase(runtime, launch.marker())?;
        registration_attempt(&current, launch)?;
        let (owner_before, readiness_before) = snapshot(runtime, |tx| {
            launch.validate_preparation_origin_tx(tx, &current)?;
            registration_unit(current.unit(), launch)?;
            no_registration(tx, launch)?;
            let readiness = initial_readiness(tx, launch)?;
            let SqlValue::Text(raw) = readiness.column("body")? else {
                anyhow::bail!("Native readiness body absent")
            };
            ensure!(
                raw.len() <= 4096,
                "Native readiness exceeds complete 4-KiB profile"
            );
            Ok((original_owner(tx, launch)?, readiness))
        })?;
        let mut readiness_after = PairRow {
            table: readiness_before.table,
            values: readiness_before.values.clone(),
        };
        let mut body = readiness_before.body()?;
        body["state"] = json!("preparing");
        body["version"] = json!(2);
        readiness_after.replace("state", SqlValue::Text("preparing".into()))?;
        readiness_after.replace("version", SqlValue::Integer(2))?;
        readiness_after.set_body(&body)?;
        let SqlValue::Text(raw) = readiness_after.column("body")? else {
            anyhow::bail!("Native planned readiness body absent")
        };
        ensure!(
            raw.len() <= 4096,
            "Native planned readiness exceeds complete profile"
        );
        Ok(Arc::new(NativePreparationPlan {
            actor,
            current,
            owner_before,
            readiness_before,
            readiness_after,
        }))
    }

    /// Only the allocated→preparing readiness image is permitted. The known
    /// commit retains the same actor/plan, not a row-minted preparation proof.
    pub(crate) fn begin_native_preparation(
        &mut self,
        plan: Arc<NativePreparationPlan>,
        admission: &PhaseEffectAdmissionGuard,
    ) -> Result<NativePreparationCommit> {
        selected_database(&self.connection, plan.actor.launch())?;
        let mutation = plan
            .readiness_before
            .update_permission(&plan.readiness_after)?;
        self.binding_permits.with_exact_permit(vec![mutation], || {
            let tx = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)?;
            admission.validate_for(plan.actor.launch())?;
            plan.actor.validate_open()?;
            plan.validate_common(&tx)?;
            plan.readiness_before.validate_tx(&tx)?;
            plan.readiness_before
                .update_tx(&tx, &plan.readiness_after)?;
            self.binding_permits.ensure_consumed()?;
            tx.commit()?;
            Ok(())
        })?;
        Ok(NativePreparationCommit { original: plan })
    }

    /// Factual confirmation requires the same independently retained original
    /// plan and complete postimage. Mixed/stale/pre images remain held; this
    /// method neither retries effects nor builds a new plan from current rows.
    pub(crate) fn confirm_native_preparation(
        &mut self,
        plan: Arc<NativePreparationPlan>,
        admission: &PhaseEffectAdmissionGuard,
    ) -> Result<NativePreparationCommit> {
        selected_database(&self.connection, plan.actor.launch())?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        admission.validate_for(plan.actor.launch())?;
        plan.validate_common(&tx)?;
        plan.readiness_after.validate_tx(&tx)?;
        tx.commit()?;
        Ok(NativePreparationCommit { original: plan })
    }
}
