//! Exact own-row factual transport bookkeeping. No input or permission grant.
use super::*;
use crate::execution::native::transport::NativeTransportObservation;

fn observation_readiness_matches(
    registered: u64,
    version: u64,
    ended: bool,
    known: bool,
    state: &str,
) -> bool {
    (state == "registered" && !known && registered_readiness_matches(registered, version, ended))
        || (state == "closed"
            && ended
            && known
            && (registered.checked_add(1) == Some(version)
                || registered.checked_add(2) == Some(version)))
}

pub(crate) struct NativeTransportSettlementPlan {
    observation: Arc<NativeTransportObservation>,
    owner: Option<NativeOwnerPlan>,
    unit: version::LatestUnitImage,
    session: PairRow,
    owner_row: PairRow,
    readiness: PairRow,
    before_values: Vec<SqlValue>,
    after_values: Vec<SqlValue>,
    update_values: Vec<SqlValue>,
}
impl NativeTransportSettlementPlan {
    fn validate_original(&self) -> Result<()> {
        self.observation.validate_original()?;
        self.observation.phase().validate_known_registration()?;
        self.observation
            .plan()
            .launch()
            .validate_preparation_original()
    }
    fn exact_effect(&self, tx: &Transaction<'_>, after: bool) -> Result<bool> {
        let values = if after {
            &self.after_values
        } else {
            &self.before_values
        };
        Ok(tx.query_row("SELECT EXISTS(SELECT 1 FROM managed_effects WHERE id IS ?1 AND unit_id IS ?2 AND project_id IS ?3 AND goal_id IS ?4 AND task_id IS ?5 AND idempotency_key IS ?6 AND state IS ?7 AND body IS ?8 AND version IS ?9)",params_from_iter(values),|r|r.get(0))?)
    }
}

impl Store {
    pub(crate) fn plan_transport_settlement(
        runtime: &crate::execution::RuntimeOwner,
        observation: Arc<NativeTransportObservation>,
    ) -> Result<Arc<NativeTransportSettlementPlan>> {
        Self::plan_transport_observation(runtime, observation, false)
    }
    pub(crate) fn plan_transport_closure(
        runtime: &crate::execution::RuntimeOwner,
        observation: Arc<NativeTransportObservation>,
    ) -> Result<Arc<NativeTransportSettlementPlan>> {
        Self::plan_transport_observation(runtime, observation, true)
    }
    fn plan_transport_observation(
        runtime: &crate::execution::RuntimeOwner,
        observation: Arc<NativeTransportObservation>,
        closure: bool,
    ) -> Result<Arc<NativeTransportSettlementPlan>> {
        observation.validate_original()?;
        observation
            .plan()
            .launch()
            .validate_preparation_original()?;
        let phase = observation.phase();
        phase.validate_known_registration()?;
        let normal = if closure {
            None
        } else {
            Some(plan_native_owner(runtime, phase)?)
        };
        let origin = observation.plan();
        let f = origin.launch.allocation().facts();
        let (unit, session, owner_row, readiness) = snapshot(runtime, |tx| {
            let budget = version::InventoryBudget::new(tx)?;
            budget.finish((|| {
                let unit=version::LatestUnitImage::read(tx,origin.unit())?;
                let session=PairRow::read(tx,"records",&f.session_id.to_string())?;
                let record:Record=serde_json::from_value(session.body()?)?;
                let stored:Session=serde_json::from_value(record.data.clone())?;
                let original=origin.session();
                ensure!(record.id==RecordId(f.session_id.0) && record.kind==RecordKind::Session && record.scope==*f.scope && record.version>0 && serde_json::to_value(&stored)?==record.data && stored.id==original.id && stored.scope==original.scope && stored.agent==original.agent && stored.provider==original.provider && stored.role==original.role && stored.worktree==original.worktree && stored.model==original.model && stored.effort==original.effort && stored.recovery==original.recovery && stored.started_at==original.started_at,"transport latest Session original changed");
                let (p,g,t)=scope_keys(f.scope)?;
                for (name,value) in [("id",SqlValue::Text(record.id.to_string())),("kind",SqlValue::Text("session".into())),("project_id",SqlValue::Text(p)),("goal_id",SqlValue::Text(g)),("task_id",SqlValue::Text(t)),("version",SqlValue::Integer(i64::try_from(record.version)?))] { ensure!(session.column(name)?==&value,"transport Session indexed/body mismatch"); }
                let owner_row=PairRow::read(tx,"managed_phase_owners",&f.pair_id.to_string())?;
                ensure!(owner_row.values==origin.owner_after.values,"transport registered owner changed");
                let readiness=PairRow::read(tx,"managed_phase_readiness",&f.operation_id.to_string())?;
                let body=readiness.body()?;
                let version=body["version"].as_u64().context("transport readiness version absent")?;
                let ended=body["start_ended"].as_bool().context("transport readiness ended absent")?;
                let known=body["known_terminal"].as_bool().context("transport readiness terminal absent")?;
                let registered=origin.registered_readiness()?;
                ensure!(observation_readiness_matches(registered,version,ended,known,body["state"].as_str().context("transport readiness state absent")?),"transport readiness lineage changed");
                let state=if known {"closed"} else {"registered"};
                ensure!(body==json!({"operation_id":f.operation_id,"origin":f.origin_id,"owner_epoch":f.epoch,"state":state,"start_ended":ended,"known_terminal":known,"parking_version":null,"version":version}),"transport readiness original body changed");
                for (name,value) in [("operation_id",SqlValue::Text(f.operation_id.to_string())),("origin",SqlValue::Text(f.origin_id.to_string())),("owner_epoch",SqlValue::Integer(i64::try_from(f.epoch)?)),("state",SqlValue::Text(state.into())),("start_ended",SqlValue::Integer(i64::from(ended))),("known_terminal",SqlValue::Integer(i64::from(known))),("parking_version",SqlValue::Null),("version",SqlValue::Integer(i64::try_from(version)?))] { ensure!(readiness.column(name)?==&value,"transport readiness indexed/body mismatch"); }
                Ok((unit,session,owner_row,readiness))
            })())
        })?;
        let mut after = origin.transport_intent().clone();
        after.state = observation.state();
        after.receipt = observation.receipt().clone();
        after.version = 2;
        let after_raw = serde_json::to_string(&after)?;
        ensure!(
            after_raw.len() <= 8192,
            "transport observation exceeds bound"
        );
        let values = |effect: &ManagedEffect, raw: &str| -> Result<Vec<SqlValue>> {
            let (p, g, t) = scope_keys(&effect.scope)?;
            Ok(vec![
                SqlValue::Text(effect.id.to_string()),
                SqlValue::Text(effect.unit_id.to_string()),
                SqlValue::Text(p),
                SqlValue::Text(g),
                SqlValue::Text(t),
                SqlValue::Text(effect.idempotency_key.clone()),
                SqlValue::Text(key(effect.state)),
                SqlValue::Text(raw.into()),
                SqlValue::Integer(i64::try_from(effect.version)?),
            ])
        };
        let before_values = values(origin.transport_intent(), origin.transport_intent_raw())?;
        let after_values = values(&after, &after_raw)?;
        let update_values = vec![
            SqlValue::Text(key(after.state)),
            SqlValue::Text(after_raw.clone()),
            before_values[0].clone(),
            before_values[7].clone(),
        ];
        Ok(Arc::new(NativeTransportSettlementPlan {
            observation,
            owner: normal,
            unit,
            session,
            owner_row,
            readiness,
            before_values,
            after_values,
            update_values,
        }))
    }
    pub(crate) fn record_transport_settlement(
        &mut self,
        plan: &Arc<NativeTransportSettlementPlan>,
    ) -> Result<()> {
        plan.validate_original()?;
        selected_database(&self.connection, plan.observation.plan().launch())?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        {
            let budget = version::InventoryBudget::new(&tx)?;
            budget.finish((|| {
                plan.validate_original()?;
                if plan.exact_effect(&tx,true)? { return Ok(()); }
                if let Some(owner)=&plan.owner { owner.validate_tx(&tx)?; }
                else { plan.unit.validate_tx(&tx)?; plan.session.validate_tx(&tx)?; plan.owner_row.validate_tx(&tx)?; plan.readiness.validate_tx(&tx)?; }
                ensure!(plan.exact_effect(&tx,false)?,"transport own intent changed; SAME observation held");
                ensure!(tx.execute("UPDATE managed_effects SET state=?1,version=2,body=?2 WHERE id=?3 AND version=1 AND body=?4",params_from_iter(&plan.update_values))?==1,"transport own-row CAS changed");
                Ok(())
            })())?;
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod transport_readiness_primitive_tests {
    use super::observation_readiness_matches;
    // Scalar lifecycle controls only: no actor, acknowledgement or SQL grant.
    #[test]
    fn factual_closure_tracks_registration_start_end_and_known_terminal() {
        for registered in [3, 5] {
            assert!(observation_readiness_matches(
                registered,
                registered,
                false,
                false,
                "registered"
            ));
            assert!(observation_readiness_matches(
                registered,
                registered + 1,
                true,
                false,
                "registered"
            ));
            assert!(observation_readiness_matches(
                registered,
                registered + 1,
                true,
                true,
                "closed"
            ));
            assert!(observation_readiness_matches(
                registered,
                registered + 2,
                true,
                true,
                "closed"
            ));
            assert!(!observation_readiness_matches(
                registered,
                registered + 3,
                true,
                true,
                "closed"
            ));
            assert!(!observation_readiness_matches(
                registered,
                registered + 1,
                true,
                false,
                "closed"
            ));
            assert!(!observation_readiness_matches(
                registered,
                registered,
                false,
                true,
                "registered"
            ));
            assert!(!observation_readiness_matches(
                registered,
                registered + 2,
                true,
                false,
                "registered"
            ));
        }
    }
}
