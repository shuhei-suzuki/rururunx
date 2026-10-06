//! SAME Prepared registration, exact intent and known-commit activation.
use super::*;
use crate::runtime::phase_effect_admission::PhaseEffectAdmissionGuard;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegistrationAckSource { Committed, Confirmed }
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct RegistrationAck {
    pub(crate) readiness: u64,
    pub(crate) unit_version: u64,
    pub(crate) source: RegistrationAckSource,
}
pub(crate) struct KnownTransportRegistration {
    plan: Arc<NativeTransportStartPlan>, ack:RegistrationAck,
}
impl KnownTransportRegistration {
    pub(crate) fn activation(self, original:&Arc<NativeTransportStartPlan>) -> Option<RegistrationAck> {
        Arc::ptr_eq(&self.plan,original).then_some(self.ack)
    }
}
pub(crate) enum RegistrationProbe { Committed(KnownTransportRegistration), Absent, Held }

impl NativeReadyLineage {
    pub(super) fn readiness_version(&self) -> Result<u64> {
        let SqlValue::Integer(version)=self.readiness().column("version")? else { anyhow::bail!("prepared readiness version absent") };
        Ok(u64::try_from(*version)?)
    }
}
fn transport_effect(launch:&PhaseLaunchParts, command_digest:&str) -> Result<ManagedEffect> {
    let f=launch.allocation().facts();
    let target=crate::execution::native_result::digest(format!("{}:{}:{}:{}:{}:{}",f.operation_id,f.pair_id,f.epoch,f.session_id,f.invocation_id,command_digest).as_bytes());
    Ok(ManagedEffect { id:OperationId::new(),unit_id:f.unit_id,scope:f.scope.clone(),
        kind:"native_phase_transport".into(), idempotency_key:format!("native-transport-{}",f.operation_id),
        expected_target:format!("transport:{target}"),state:EffectState::Pending,receipt:BTreeMap::new(),version:1 })
}

/// A finite unconsumed registration plan. No public ID/DTO can construct this.
/// The retained launch is rechecked at commit; this value is not a Native owner.
pub(crate) struct NativeTransportStartPlan {
    prepared: Arc<crate::execution::native::PreparedNativePhase>,
    launch: Arc<PhaseLaunchParts>,
    governing_digest: String,
    effect: ManagedEffect,
    effect_raw: String,
    current: CurrentWorkflowSuccessor,
    owner_before: PairRow,
    owner_after: PairRow,
    readiness_before: PairRow,
    readiness_after: PairRow,
    session: Session,
    record: Record,
    record_raw: String,
    unit_after: ExecutionUnit,
    unit_raw: String,
    invocation: crate::execution::native_result::NativeInvocation,
    invocation_raw: String,
}
impl NativeTransportStartPlan {
    pub(crate) fn launch(&self) -> &Arc<PhaseLaunchParts> { &self.launch }
    pub(crate) fn session(&self) -> &Session { &self.session }
    pub(crate) fn unit(&self) -> &ExecutionUnit { &self.unit_after }
    pub(crate) fn prepared(&self) -> &Arc<crate::execution::native::PreparedNativePhase> { &self.prepared }
    pub(crate) fn transport_intent(&self) -> &ManagedEffect { &self.effect }
    pub(crate) fn transport_intent_raw(&self) -> &str { &self.effect_raw }
    pub(crate) fn registered_readiness(&self) -> Result<u64> { Ok(self.prepared.lineage().readiness_version()?.checked_add(1).context("readiness exhausted")?) }
    pub(super) fn governing_digest(&self) -> &str { &self.governing_digest }
    fn validate_origin_tx(&self,tx:&Transaction<'_>) -> Result<()> {
        self.prepared.validate_original()?;
        self.launch.validate_preparation_origin_tx(tx,&self.current)?;
        validate_unit_authority_facts(tx,&self.current.unit().authority(),self.current.unit())?;
        validate_native_effect_open(self.current.unit())?;
        let original=self.launch.marker().original_plan();
        validate_parent_activity_facts(self.current.unit(),original.project().0,original.goal().0,original.task_after().0)?;
        validate_governing_context_facts(tx,self.current.unit(),&self.governing_digest)
    }
    fn validate_pre_tx(&self,tx:&Transaction<'_>) -> Result<()> {
        self.prepared.validate_open()?;
        self.validate_origin_tx(tx)?;
        self.prepared.lineage().validate_registration_tx(tx)?;
        self.prepared.quota().validate_registration_tx(tx)?;
        self.prepared.history().validate_inventory(tx)?;
        registration_unit(self.current.unit(),&self.launch)?;
        ensure!(!verification::is_command_unit(tx,self.current.unit().id)?,"command-only verifier cannot register Native");
        no_registration(tx,&self.launch)?;
        self.owner_before.validate_tx(tx)?;self.readiness_before.validate_tx(tx)?;
        let absent:bool=tx.query_row("SELECT NOT EXISTS(SELECT 1 FROM managed_effects WHERE id=?1 OR idempotency_key=?2)",params![self.effect.id.to_string(),self.effect.idempotency_key],|r|r.get(0))?;
        ensure!(absent,"transport intent identity exists");Ok(())
    }
    fn validate_post_tx(&self,tx:&Transaction<'_>) -> Result<()> {
        // Confirmation validates the SAME original Source/frame and known own
        // postimages. It does not reuse the pre-registration Unit CAS.
        self.prepared.validate_original()?;
        let post=self.current.with_known_unit(Arc::new(crate::state::managed_binding::Body::decode(self.unit_raw.clone(),16*1024)?))?;
        self.launch.validate_preparation_origin_tx(tx,&post)?;
        validate_unit_authority_facts(tx,&self.unit_after.authority(),&self.unit_after)?;
        validate_native_effect_open(&self.unit_after)?;
        let original=self.launch.marker().original_plan();
        validate_parent_activity_facts(&self.unit_after,original.project().0,original.goal().0,original.task_after().0)?;
        validate_governing_context_facts(tx,&self.unit_after,&self.governing_digest)?;
        self.owner_after.validate_tx(tx)?;self.readiness_after.validate_tx(tx)?;
        self.prepared.quota().validate_registration_tx(tx)?;
        self.prepared.history().validate_registration_inventory(tx,&self.effect)?;
        let (p,g,t)=scope_keys(&self.session.scope)?;
        let exact:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE id=?1 AND kind='session' AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND version=1 AND body=?5) AND EXISTS(SELECT 1 FROM session_units WHERE session_id=?1 AND unit_id=?6 AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND dispatch_state='pending') AND NOT EXISTS(SELECT 1 FROM session_units WHERE (session_id=?1 OR unit_id=?6) AND NOT(session_id=?1 AND unit_id=?6))",params![self.session.id.to_string(),p,g,t,self.record_raw,self.unit_after.id.to_string()],|r|r.get(0))?;
        ensure!(exact,"registration Session complete postimage changed");
        let i=&self.invocation;
        let exact:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM native_invocations WHERE id=?1 AND unit_id=?2 AND session_id=?3 AND project_id=?4 AND goal_id=?5 AND task_id=?6 AND generation=?7 AND owner_epoch=?8 AND provider=?9 AND state='not_dispatched' AND version=1 AND body=?10) AND NOT EXISTS(SELECT 1 FROM native_invocations WHERE (unit_id=?2 OR session_id=?3) AND id<>?1) AND NOT EXISTS(SELECT 1 FROM managed_phase_admissions WHERE pair_id=?11 OR operation_id=?12)",params![i.id.to_string(),i.unit_id.to_string(),i.session_id.to_string(),p,g,t,i.generation,i.owner_epoch,i.provider,self.invocation_raw,self.launch.allocation().facts().pair_id.to_string(),self.launch.allocation().facts().operation_id.to_string()],|r|r.get(0))?;
        ensure!(exact,"registration invocation complete postimage changed");Ok(())
    }
    fn known(self:&Arc<Self>,source:RegistrationAckSource) -> Result<KnownTransportRegistration> {
        Ok(KnownTransportRegistration { plan:self.clone(),ack:RegistrationAck {readiness:self.registered_readiness()?,unit_version:self.unit_after.version,source} })
    }
}

/// Exact current planning, hashing and complete encoded output preparation are
/// outside SharedStore. The same original actual launch is mandatory at commit.
fn plan_prepared_transport(
    owner: &crate::execution::RuntimeOwner,
    prepared: Arc<crate::execution::native::PreparedNativePhase>,
) -> Result<Arc<NativeTransportStartPlan>> {
    prepared.validate_open()?;
    let launch = prepared.launch().clone();
    let (session, seed, command_digest) = prepared.registration_material()?;
    let governing_digest = prepared.governing_digest()?;
    ensure!(
        launch.is_retained() && Arc::ptr_eq(launch.marker().allocation(), launch.allocation()),
        "Native registration lost original retained launch"
    );
    let f = launch.allocation().facts();
    ensure!(
        session.id == f.session_id
            && session.scope == *f.scope
            && session.agent == f.alias
            && session.provider == f.provider
            && session.role == f.role
            && session.worktree == f.path
            && session.model.as_deref() == f.model
            && session.effort.as_deref() == f.effort
            && session.pid.is_none()
            && session.native_ref.is_none()
            && session.state == SessionState::Starting
            && seed.id() == f.invocation_id
            && crate::execution::phase::encode_input(seed.input())? == f.input_bytes,
        "Native registration seed or Session differs from original selected allocation"
    );
    let current = prepared.lineage().current().copy_original();
    registration_attempt(&current, &launch)?;
    let (owner_before, readiness_before, artifact_version) = snapshot(owner, |tx| {
        prepared.lineage().validate_registration_tx(tx)?;
        prepared.history().validate_inventory(tx)?;
        prepared.quota().validate_registration_tx(tx)?;
        registration_unit(current.unit(), &launch)?;
        no_registration(tx, &launch)?;
        let artifact_version = current
            .unit()
            .artifact_id
            .map(|id| self_artifact_tx(tx, id).map(|a| a.version))
            .transpose()?;
        Ok((
            original_owner(tx, &launch)?,
            prepared.lineage().readiness().copy_image(),
            artifact_version,
        ))
    })?;
    let mut owner_after = PairRow {
        table: owner_before.table,
        values: owner_before.values.clone(),
    };
    let mut body = owner_before.body()?;
    body["native_invocation_id"] = json!(f.invocation_id);
    body["validated"] = json!(true);
    body["version"] = json!(2);
    owner_after.replace(
        "native_invocation_id",
        SqlValue::Text(f.invocation_id.to_string()),
    )?;
    owner_after.replace("validated", SqlValue::Integer(1))?;
    owner_after.replace("version", SqlValue::Integer(2))?;
    owner_after.set_body(&body)?;
    let mut readiness_after = PairRow {
        table: readiness_before.table,
        values: readiness_before.values.clone(),
    };
    let mut body = readiness_before.body()?;
    body["state"] = json!("registered");
    body["version"] = json!(prepared.lineage().readiness_version()?.checked_add(1).context("readiness exhausted")?);
    readiness_after.replace("state", SqlValue::Text("registered".into()))?;
    readiness_after.replace("version", SqlValue::Integer(i64::try_from(prepared.lineage().readiness_version()?.checked_add(1).context("readiness exhausted")?)?))?;
    readiness_after.set_body(&body)?;
    let at = now_ms();
    let record = Record {
        id: RecordId(session.id.0),
        scope: session.scope.clone(),
        kind: RecordKind::Session,
        version: 1,
        data: serde_json::to_value(&session)?,
        created_at: at,
        updated_at: at,
    };
    let record_raw = serde_json::to_string(&record)?;
    ensure!(
        record_raw.len() <= PAIR_BODY_BYTES,
        "Native Session registration exceeds profile"
    );
    let mut unit_after = current.unit().clone();
    unit_after.version = unit_after
        .version
        .checked_add(1)
        .context("Native Unit version exhausted")?;
    unit_after.updated_at = at;
    unit_after.session_id = Some(session.id);
    unit_after.state = UnitState::DispatchPending;
    let unit_raw = serde_json::to_string(&unit_after)?;
    ensure!(
        unit_raw.len() <= 16 * 1024,
        "Native Unit registration exceeds profile"
    );
    let context = launch.marker().original_plan().context().0;
    let context_hash = crate::execution::native_result::digest(&serde_json::to_vec(context)?);
    let invocation = seed.invocation(&unit_after, &session, Some(context_hash), artifact_version);
    invocation.validate()?;
    let invocation_raw = serde_json::to_string(&invocation)?;
    ensure!(
        invocation_raw.len() <= crate::execution::native_result::INVOCATION_BYTES,
        "Native registration invocation exceeds profile"
    );
    let effect = transport_effect(&launch, &command_digest)?;
    let effect_raw = serde_json::to_string(&effect)?;
    ensure!(prepared.history().len() <= 252, "prepared effect reserve exhausted");
    Ok(Arc::new(NativeTransportStartPlan {
        prepared, governing_digest, effect, effect_raw, launch,
        current,
        owner_before,
        owner_after,
        readiness_before,
        readiness_after,
        session,
        record,
        record_raw,
        unit_after,
        unit_raw,
        invocation,
        invocation_raw,
    }))
}

impl Store {
    pub(crate) fn plan_prepared_transport(runtime:&crate::execution::RuntimeOwner,prepared:Arc<crate::execution::native::PreparedNativePhase>) -> Result<Arc<NativeTransportStartPlan>> {
        plan_prepared_transport(runtime,prepared)
    }
    pub(crate) fn register_prepared_transport(&mut self,plan:&Arc<NativeTransportStartPlan>,admission:&PhaseEffectAdmissionGuard) -> Result<KnownTransportRegistration> {
        selected_database(&self.connection,&plan.launch)?;
        let (p,g,t)=scope_keys(&plan.session.scope)?;
        let mutations=vec![plan.owner_before.update_permission(&plan.owner_after)?,plan.readiness_before.update_permission(&plan.readiness_after)?,
            ExactRowMutation::new("records","INSERT",None,Some(vec![SqlValue::Text(plan.record.id.to_string()),SqlValue::Text("session".into()),SqlValue::Text(p.clone()),SqlValue::Text(g.clone()),SqlValue::Text(t.clone()),SqlValue::Integer(1),SqlValue::Text(plan.record_raw.clone())]))?];
        self.binding_permits.with_exact_permit(mutations,|| {
            let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            {
                let budget=super::version::InventoryBudget::new(&tx)?;
                budget.finish((|| {
                    admission.validate_for(&plan.launch)?;
                    plan.validate_pre_tx(&tx)?;
                    tx.execute("INSERT INTO records(id,kind,project_id,goal_id,task_id,version,body) VALUES(?1,'session',?2,?3,?4,1,?5)",params![plan.session.id.to_string(),p,g,t,plan.record_raw])?;
                    tx.execute("INSERT INTO session_units(session_id,unit_id,project_id,goal_id,task_id,dispatch_state) VALUES(?1,?2,?3,?4,?5,'pending')",params![plan.session.id.to_string(),plan.unit_after.id.to_string(),p,g,t])?;
                    ensure!(tx.execute("UPDATE execution_units SET version=?1,body=?2 WHERE id=?3 AND version=?4 AND body=?5",params![plan.unit_after.version,plan.unit_raw,plan.unit_after.id.to_string(),plan.current.unit().version,plan.current.unit_raw()])?==1,"prepared registration Unit CAS changed");
                    let i=&plan.invocation;
                    tx.execute("INSERT INTO native_invocations(id,unit_id,session_id,project_id,goal_id,task_id,generation,owner_epoch,provider,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,'not_dispatched',1,?10)",params![i.id.to_string(),i.unit_id.to_string(),i.session_id.to_string(),p,g,t,i.generation,i.owner_epoch,i.provider,plan.invocation_raw])?;
                    plan.owner_before.update_tx(&tx,&plan.owner_after)?;
                    plan.readiness_before.update_tx(&tx,&plan.readiness_after)?;
                    // Exact Native-only row writer. Generic effect authority is
                    // deliberately unavailable for this protected kind.
                    tx.execute("INSERT INTO managed_effects(id,unit_id,project_id,goal_id,task_id,idempotency_key,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,'pending',1,?7)",params![plan.effect.id.to_string(),plan.effect.unit_id.to_string(),p,g,t,plan.effect.idempotency_key,plan.effect_raw])?;
                    append_event(&tx,&plan.session.scope,"execution.session_intent",json!({"unit":plan.unit_after.id,"session":plan.session.id}))?;
                    append_event(&tx,&plan.session.scope,"execution.native_invocation",json!({"unit":plan.unit_after.id,"session":plan.session.id,"invocation":i.id}))?;
                    self.binding_permits.ensure_consumed()?;Ok(())
                })())?;
            }
            tx.commit()?;Ok(())
        })?;
        plan.known(RegistrationAckSource::Committed)
    }
    pub(crate) fn confirm_prepared_transport(&mut self,plan:&Arc<NativeTransportStartPlan>,admission:&PhaseEffectAdmissionGuard) -> Result<RegistrationProbe> {
        selected_database(&self.connection,&plan.launch)?;
        let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let probe={
            let budget=super::version::InventoryBudget::new(&tx)?;
            budget.finish((|| {
                admission.validate_for(&plan.launch)?;
                let readiness=PairRow::read(&tx,"managed_phase_readiness",&plan.launch.allocation().facts().operation_id.to_string())?;
                if readiness.values==plan.readiness_after.values {
                    plan.validate_post_tx(&tx)?;
                    return Ok(RegistrationProbe::Committed(plan.known(RegistrationAckSource::Confirmed)?));
                }
                if readiness.values==plan.readiness_before.values {
                    plan.validate_pre_tx(&tx)?;
                    return Ok(RegistrationProbe::Absent);
                }
                Ok(RegistrationProbe::Held)
            })())?
        };
        tx.commit()?;Ok(probe)
    }
}
