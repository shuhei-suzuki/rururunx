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
    governing_digest: String,
}
pub(crate) struct NativePreparationCommit {
    original: Arc<NativePreparationPlan>,
}
/// Images advance only from this actor's own known quota transition.
pub(crate) enum NativeReadyLineage {
    Initial(Arc<NativePreparationCommit>),
    Quota { commit: Arc<NativePreparationCommit>, current: CurrentWorkflowSuccessor, readiness: PairRow },
}
impl NativeReadyLineage {
    pub(super) fn validate_closure_tx(&self,tx:&Transaction<'_>) -> Result<()> {
        let original=&self.commit().original;
        original.actor.validate_original()?;
        selected_database(tx,original.actor.launch())?;
        original.owner_before.validate_tx(tx)?;
        no_registration(tx,original.actor.launch())?;
        self.readiness().validate_tx(tx)
    }
    pub(super) fn commit(&self) -> &Arc<NativePreparationCommit> { match self { Self::Initial(c) | Self::Quota { commit:c, .. } => c } }
    pub(super) fn current(&self) -> &CurrentWorkflowSuccessor { match self { Self::Initial(c) => &c.original.current, Self::Quota { current, .. } => current } }
    pub(super) fn readiness(&self) -> &PairRow { match self { Self::Initial(c) => &c.original.readiness_after, Self::Quota { readiness, .. } => readiness } }
    pub(crate) fn unit(&self) -> &ExecutionUnit { self.current().unit() }
    pub(crate) fn validate_prepared_shape(&self) -> Result<()> {
        let row = self.readiness();
        ensure!(row.column("state")? == &SqlValue::Text("preparing".into()) && matches!(row.column("version")?, SqlValue::Integer(2 | 4)) && row.column("parking_version")? == &SqlValue::Null && self.unit().state == UnitState::Preparing && self.unit().wait_reason.is_none(), "known admitted lineage is not prepared");
        Ok(())
    }
    pub(super) fn validate_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        match self { Self::Initial(c) => c.validate_version_ready(tx), Self::Quota { commit, current, readiness } => { commit.original.validate_common_with(tx,current)?; readiness.validate_tx(tx) } }
    }
    pub(super) fn known_successor(&self, unit: Arc<crate::state::managed_binding::Body<ExecutionUnit>>, readiness: PairRow) -> Result<Arc<Self>> {
        Ok(Arc::new(Self::Quota { commit: self.commit().clone(), current: self.current().with_known_unit(unit)?, readiness }))
    }
}
impl NativePreparationCommit {
    pub(super) fn project_limit(&self) -> usize { self.original.actor.launch().marker().original_plan().project().0.max_tasks }
    pub(super) fn actor(&self) -> &Arc<NativePreparationActor> {
        self.original.actor()
    }
    pub(super) fn validate_version_ready(&self, tx: &Transaction<'_>) -> Result<()> {
        self.original.validate_common(tx)?;
        self.original.readiness_after.validate_tx(tx)
    }
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
        self.validate_common_with(tx, &self.current)
    }
    fn validate_common_with(&self, tx: &Transaction<'_>, current: &CurrentWorkflowSuccessor) -> Result<()> {
        self.actor.validate_original()?;
        let launch = self.actor.launch();
        selected_database(tx, launch)?;
        launch.validate_preparation_origin_tx(tx, current)?;
        self.validate_facts(tx, current.unit())?;
        registration_unit(current.unit(), launch)?;
        no_registration(tx, launch)?;
        self.owner_before.validate_tx(tx)?;
        Ok(())
    }
    fn validate_facts(&self, tx: &Transaction<'_>, unit: &ExecutionUnit) -> Result<()> {
        let marker = self.actor.launch().marker().original_plan();
        // Root's SAME current/origin check above has already matched these
        // original full parent and Unit images. No row decoding or hashing is
        // needed here, and these factual checks cannot grant a Native stage.
        validate_unit_authority_facts(tx, &unit.authority(), unit)?;
        validate_native_effect_open(unit)?;
        validate_parent_activity_facts(
            unit,
            marker.project().0,
            marker.goal().0,
            marker.task_after().0,
        )?;
        validate_governing_context_facts(tx, unit, &self.governing_digest)
    }
}

impl Store {
    pub(crate) fn check_phase_no_current_dispatch(&mut self, actor: &Arc<NativePreparationActor>, lineage: &NativeReadyLineage, completion: &Arc<crate::execution::native::version::NativeReadonlyHelperCompletion>, admission: &PhaseEffectAdmissionGuard) -> Result<()> {
        let tx = self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        {
            let budget = super::version::InventoryBudget::new(&tx)?;
            budget.finish((|| {
                admission.validate_for(actor.launch())?; actor.validate_open()?;
                lineage.validate_tx(&tx)?;
                ensure!(matches!(lineage, NativeReadyLineage::Initial(_)) && completion.matches_actor(actor) && lineage.unit().wait_reason.is_none(), "no-dispatch Initial facts differ");
                no_registration(&tx,actor.launch())?;
                completion.commit.validate_inventory(&tx)
            })())?;
        }
        tx.commit()?; Ok(())
    }
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
        let marker = launch.marker().original_plan();
        // Original marker parent bodies are immutable and checked in the
        // snapshot and every later transaction. Hash only before Store entry.
        let governing_digest = governing_digest(marker.project().0, marker.goal().0)?;
        let (owner_before, readiness_before) = snapshot(runtime, |tx| {
            launch.validate_preparation_origin_tx(tx, &current)?;
            let unit = current.unit();
            validate_unit_authority_facts(tx, &unit.authority(), unit)?;
            validate_native_effect_open(unit)?;
            validate_parent_activity_facts(
                unit,
                marker.project().0,
                marker.goal().0,
                marker.task_after().0,
            )?;
            validate_governing_context_facts(tx, unit, &governing_digest)?;
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
            governing_digest,
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
