use super::*;

impl Store {
    pub(crate) fn reserve_managed_effect(&mut self,authority:&ExecutionAuthority,effect:&ManagedEffect) -> Result<()> {
        ensure!(effect.unit_id==authority.unit_id && effect.scope==authority.scope && effect.state==EffectState::Pending
            && effect.version==1 && effect.receipt.is_empty() && !effect.expected_target.is_empty()
            && effect.expected_target.len()<=4096 && effect.idempotency_key.len()<=256 && !effect.idempotency_key.is_empty(),"invalid effect intent");
        let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        validate_authority(&tx,authority,true,false)?;
        let (p,g,t)=scope_keys(&effect.scope)?;
        tx.execute("INSERT INTO managed_effects(id,unit_id,project_id,goal_id,task_id,idempotency_key,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,'pending',1,?7)",
            params![effect.id.to_string(),effect.unit_id.to_string(),p,g,t,effect.idempotency_key,serde_json::to_string(effect)?])?;
        append_event(&tx,&effect.scope,"execution.effect_intent",json!({"unit":effect.unit_id,"operation":effect.id,"kind":effect.kind}))?;
        tx.commit()?;Ok(())
    }
    pub fn managed_effect(&self,id:OperationId) -> Result<ManagedEffect> {
        let body:String=self.connection.query_row("SELECT body FROM managed_effects WHERE id=?1",[id.to_string()],|r|r.get(0))?;
        let effect:ManagedEffect=decode(body)?;ensure!(effect.id==id,"effect indexed identity mismatch");Ok(effect)
    }
    pub fn managed_effects(&self,unit:UnitId) -> Result<Vec<ManagedEffect>> {
        let mut s=self.connection.prepare("SELECT body FROM managed_effects WHERE unit_id=?1 ORDER BY rowid")?;
        s.query_map([unit.to_string()],|r|r.get::<_,String>(0))?.map(|r|r.map_err(anyhow::Error::from).and_then(decode)).collect()
    }
    pub(crate) fn reconcile_managed_effect(&mut self,id:OperationId,expected:u64,state:EffectState,receipt:BTreeMap<String,String>) -> Result<()> {
        ensure!(receipt.len()<=16 && receipt.iter().all(|(k,v)|k.len()<=64 && v.len()<=256 && !v.chars().any(char::is_control)),"invalid effect safe receipt");
        let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let body:String=tx.query_row("SELECT body FROM managed_effects WHERE id=?1",[id.to_string()],|r|r.get(0))?;
        let mut effect:ManagedEffect=decode(body)?;
        ensure!(effect.version==expected && effect.state!=EffectState::Resolved,"effect CAS/terminal conflict");
        ensure!(state!=EffectState::Pending,"effect intent cannot be replayed");
        effect.state=state;effect.receipt=receipt;effect.version+=1;
        let changed=tx.execute("UPDATE managed_effects SET state=?1,version=?2,body=?3 WHERE id=?4 AND version=?5",params![key(state),effect.version,serde_json::to_string(&effect)?,id.to_string(),expected])?;
        ensure!(changed==1,"effect CAS conflict");
        append_event(&tx,&effect.scope,"execution.effect_reconciled",json!({"unit":effect.unit_id,"operation":id,"state":state}))?;
        tx.commit()?;Ok(())
    }
    pub(crate) fn release_execution_lease(&mut self,id:LeaseId,unit_id:UnitId,expected:u64,confirmed:bool) -> Result<()> {
        let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let body:String=tx.query_row("SELECT body FROM resource_leases WHERE id=?1",[id.to_string()],|r|r.get(0))?;
        let mut lease:ResourceLease=decode(body)?;
        ensure!(lease.id==id && lease.unit_id==unit_id && lease.version==expected,"lease identity/CAS conflict");
        let unit=unit_tx(&tx,unit_id)?;
        ensure!(!unit.native_effects_open,"cannot release active execution resources");
        if matches!(lease.kind,ResourceKind::Worktree|ResourceKind::Temp|ResourceKind::Output) {
            ensure!(!unit.result_finalization_open,"result capture still depends on paths");
        }
        lease.state=if confirmed {LeaseState::Released}else{LeaseState::Quarantined};lease.version+=1;
        tx.execute("UPDATE resource_leases SET state=?1,version=?2,body=?3 WHERE id=?4 AND version=?5",params![key(lease.state),lease.version,serde_json::to_string(&lease)?,id.to_string(),expected])?;
        append_event(&tx,&unit.scope,"execution.resource_disposition",json!({"unit":unit_id,"lease":id,"state":lease.state}))?;
        tx.commit()?;Ok(())
    }
}
