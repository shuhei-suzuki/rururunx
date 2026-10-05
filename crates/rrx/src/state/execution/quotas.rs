use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuotaAdmission { Admitted, Waiting { reason:WaitReason, next_due:i64 } }

impl Store {
    pub fn quota_observations(&self,provider:&str,account:&str) -> Result<Vec<QuotaObservation>> {
        let mut s=self.connection.prepare("SELECT body FROM quota_windows WHERE provider=?1 AND account_key=?2 ORDER BY bucket")?;
        s.query_map(params![provider,account],|r|r.get::<_,String>(0))?.map(|r|r.map_err(anyhow::Error::from).and_then(decode)).collect()
    }
    pub(crate) fn observe_quota(&mut self,observation:&QuotaObservation) -> Result<()> {
        ensure!([&observation.provider,&observation.account_key,&observation.bucket,&observation.window_id,&observation.source_version]
            .iter().all(|s|!s.is_empty() && s.len()<=256 && !s.chars().any(char::is_control)),"invalid quota identity");
        ensure!(observation.used_percent.is_none_or(|p|p.is_finite() && (0.0..=100.0).contains(&p)),"invalid quota percentage");
        ensure!(observation.status!=QuotaStatus::Exhausted || observation.confirmed_subscription,"unconfirmed subscription exhaustion");
        let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("INSERT INTO quota_pools(provider,account_key) VALUES(?1,?2) ON CONFLICT DO NOTHING",params![observation.provider,observation.account_key])?;
        let prior:Option<String>=tx.query_row("SELECT body FROM quota_windows WHERE provider=?1 AND account_key=?2 AND bucket=?3",params![observation.provider,observation.account_key,observation.bucket],|r|r.get(0)).optional()?;
        if let Some(prior)=prior {
            let old:QuotaObservation=decode(prior)?;
            if observation.observed_at<old.observed_at { tx.commit()?;return Ok(()); }
            if old.status==QuotaStatus::Exhausted && observation.status!=QuotaStatus::Exhausted {
                // A different connection/window cannot reopen exhaustion before the old reset.
                let fresh_window=observation.status==QuotaStatus::Available && old.resets_at.is_some_and(|r|observation.observed_at>=r)
                    && observation.window_id!=old.window_id;
                if !fresh_window {tx.commit()?;return Ok(());}
            }
        }
        tx.execute("INSERT INTO quota_windows(provider,account_key,bucket,observed_at,body) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(provider,account_key,bucket) DO UPDATE SET observed_at=excluded.observed_at,body=excluded.body",
            params![observation.provider,observation.account_key,observation.bucket,observation.observed_at,serde_json::to_string(observation)?])?;
        if observation.status==QuotaStatus::Exhausted {
            let next=observation.resets_at.unwrap_or_else(||observation.observed_at.saturating_add(60_000));
            tx.execute("UPDATE quota_pools SET next_probe_at=MAX(next_probe_at,?1) WHERE provider=?2 AND account_key=?3",params![next,observation.provider,observation.account_key])?;
        } else if observation.status==QuotaStatus::Available {
            tx.execute("UPDATE quota_pools SET backoff=60000,probe_unit=NULL WHERE provider=?1 AND account_key=?2",params![observation.provider,observation.account_key])?;
        }
        tx.commit()?;Ok(())
    }
    /// Unknown balances have explicit finite concurrency; they never manufacture token capacity.
    pub(crate) fn reserve_execution_quota(&mut self,authority:&ExecutionAuthority,provider:&str,account:&str,
        global_max:usize,executor_max:usize,provider_max:usize,at:i64) -> Result<QuotaAdmission> {
        ensure!(global_max>0 && executor_max>0 && provider_max>0 && global_max<=1024 && provider_max<=1024,"invalid quota concurrency");
        let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit=validate_authority(&tx,authority,true,false)?;
        ensure!(unit.provider==provider,"foreign quota provider");
        tx.execute("INSERT INTO quota_pools(provider,account_key) VALUES(?1,?2) ON CONFLICT DO NOTHING",params![provider,account])?;
        let already:Option<(String,String)>=tx.query_row("SELECT provider,account_key FROM quota_leases WHERE unit_id=?1 AND active=1",[unit.id.to_string()],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        if let Some((p,a))=already {ensure!(p==provider && a==account,"quota lease account mismatch");tx.commit()?;return Ok(QuotaAdmission::Admitted);}
        let (next,probe):(i64,Option<String>)=tx.query_row("SELECT next_probe_at,probe_unit FROM quota_pools WHERE provider=?1 AND account_key=?2",params![provider,account],|r|Ok((r.get(0)?,r.get(1)?)))?;
        let mut windows=tx.prepare("SELECT body FROM quota_windows WHERE provider=?1 AND account_key=?2")?;
        let observations=windows.query_map(params![provider,account],|r|r.get::<_,String>(0))?.map(|r|r.map_err(anyhow::Error::from).and_then(decode::<QuotaObservation>)).collect::<Result<Vec<_>>>()?;
        drop(windows);
        let exhausted=observations.iter().any(|o|o.status==QuotaStatus::Exhausted);
        let global:usize=tx.query_row("SELECT COUNT(*) FROM quota_leases WHERE active=1",[],|r|r.get(0))?;
        let (provider_live,executor_live):(usize,usize)=tx.query_row("SELECT COUNT(*),COALESCE(SUM(role='executor'),0) FROM quota_leases WHERE provider=?1 AND account_key=?2 AND active=1",params![provider,account],|r|Ok((r.get(0)?,r.get(1)?)))?;
        let capacity=global>=global_max || provider_live>=provider_max || (unit.kind==UnitKind::Executor && executor_live>=executor_max);
        let wait=if exhausted && (at<next || probe.as_ref().is_some_and(|id|id!=&unit.id.to_string())) {
            Some((WaitReason::Quota,next.max(at.saturating_add(1_000))))
        } else if capacity {Some((WaitReason::Capacity,at.saturating_add(1_000)))} else {None};
        if let Some((reason,due))=wait {
            unit.wait_reason=Some(reason);if reason==WaitReason::Quota{unit.state=UnitState::WaitingQuota;}
            write_unit(&tx,&mut unit)?;
            tx.execute("INSERT INTO quota_waiters(unit_id,reason,next_due,fairness_sequence) VALUES(?1,?2,?3,?4) ON CONFLICT(unit_id) DO UPDATE SET reason=excluded.reason,next_due=excluded.next_due",
                params![unit.id.to_string(),key(reason),due,at])?;
            tx.commit()?;return Ok(QuotaAdmission::Waiting{reason,next_due:due});
        }
        // One native recovery probe, not one probe per Task. Lease survives until terminal.
        if exhausted {
            tx.execute("UPDATE quota_pools SET probe_unit=?1,next_probe_at=?2+backoff,backoff=MIN(backoff*2,1800000) WHERE provider=?3 AND account_key=?4",
                params![unit.id.to_string(),at,provider,account])?;
        }
        tx.execute("INSERT INTO quota_leases(unit_id,provider,account_key,role,epoch,active) VALUES(?1,?2,?3,?4,?5,1) ON CONFLICT(unit_id) DO UPDATE SET provider=excluded.provider,account_key=excluded.account_key,role=excluded.role,epoch=excluded.epoch,active=1",
            params![unit.id.to_string(),provider,account,key(unit.kind),unit.owner_epoch])?;
        tx.execute("DELETE FROM quota_waiters WHERE unit_id=?1",[unit.id.to_string()])?;
        unit.wait_reason=None;if unit.state==UnitState::WaitingQuota{unit.state=UnitState::Reserved;}
        write_unit(&tx,&mut unit)?;
        append_event(&tx,&unit.scope,"execution.quota_admitted",json!({"unit":unit.id,"provider":provider,"balance_known":!observations.is_empty(),"probe":exhausted}))?;
        tx.commit()?;Ok(QuotaAdmission::Admitted)
    }
    pub(crate) fn release_execution_quota(&mut self,id:UnitId) -> Result<()> {
        let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        release_quota_tx(&tx,id)?;
        tx.commit()?;Ok(())
    }
}
pub(super) fn release_quota_tx(tx:&Transaction<'_>,id:UnitId)->Result<()> {
    tx.execute("UPDATE quota_leases SET active=0 WHERE unit_id=?1",[id.to_string()])?;
    tx.execute("UPDATE quota_pools SET probe_unit=NULL,next_probe_at=MAX(next_probe_at,?1+backoff) WHERE probe_unit=?2",params![now_ms(),id.to_string()])?;
    Ok(())
}
