//! Private pre-Session quota plans. Rows are exact images, never grant issuers.
use super::*;
use super::preparation::NativeReadyLineage;
use super::version::InventoryBudget;
use crate::execution::native::{NativePreparationActor, prepared::PreparedPhaseNoCurrentDispatch};
use crate::runtime::phase_effect_admission::PhaseEffectAdmissionGuard;
use crate::state::managed_binding::Body;
use super::super::quota_policy::{self, CandidateClass, Decision, QuotaSnapshot};

const LEASES: &str = "SELECT l.unit_id,l.provider,l.account_key,l.role,l.epoch,l.active,u.project_id,u.task_id,u.kind FROM quota_leases l JOIN execution_units u ON u.id=l.unit_id WHERE l.active=1 ORDER BY l.unit_id LIMIT 4097";
const WAITERS: &str = "SELECT w.unit_id,w.provider,w.account_key,w.reason,w.next_due,w.fairness_sequence,w.resume_state,u.kind,u.project_id,u.task_id,u.native_effects_open,u.version,COALESCE(json_extract(u.body,'$.state'),''),COALESCE(json_extract(u.body,'$.wait_reason'),''),EXISTS(SELECT 1 FROM managed_phase_operations o WHERE o.unit_id=u.id AND o.phase_open=1),EXISTS(SELECT 1 FROM managed_phase_operations o JOIN managed_phase_readiness r ON r.operation_id=o.operation_id WHERE o.unit_id=u.id AND o.phase_open=1 AND r.state='parked' AND r.parking_version=r.version),json_extract(p.body,'$.max_tasks') FROM quota_waiters w JOIN execution_units u ON u.id=w.unit_id JOIN projects p ON p.id=u.project_id WHERE w.provider=?1 AND w.account_key='unknown' AND w.next_due<=?2 AND u.native_effects_open=1 ORDER BY CASE WHEN (u.kind='executor')=(?3='executor') THEN 1 ELSE 0 END,w.fairness_sequence,w.unit_id LIMIT 4097";
const POOL: &str = "SELECT provider,account_key,next_probe_at,probe_unit,backoff,last_role FROM quota_pools WHERE provider=?1 AND account_key='unknown'";
const OWN_WAITER: &str = "SELECT unit_id,provider,account_key,reason,next_due,fairness_sequence,resume_state FROM quota_waiters WHERE unit_id=?1";
const OWN_LEASE: &str = "SELECT unit_id,provider,account_key,role,epoch,active FROM quota_leases WHERE unit_id=?1";
const WINDOWS: &str = "SELECT provider,account_key,bucket,observed_at,body FROM quota_windows WHERE provider=?1 AND account_key='unknown' ORDER BY bucket LIMIT 65";
const HISTORY: &str = "SELECT id,body FROM execution_units WHERE task_id=?1 AND id<>?2 ORDER BY id LIMIT 257";
type Row = Vec<SqlValue>;

#[derive(Clone, PartialEq)]
struct Images { pool: Option<Row>, waiter: Option<Row>, lease: Option<Row> }
pub(crate) struct NativeQuotaClosurePlan {
    actor:Arc<NativePreparationActor>,no_dispatch:Arc<PreparedPhaseNoCurrentDispatch>,lineage:Arc<NativeReadyLineage>,
    // Latest factual Unit is confined to this nongrant plan. It never enters
    // quota lineage, Prepared or any registration/permission constructor.
    unit:super::version::LatestUnitImage,before:Images,after:Images,readiness:PairRow,at:i64,
}
pub(crate) struct NativePreparationClosureCommit { original:Arc<NativeQuotaClosurePlan> }
pub(crate) enum NativeQuotaClosureConfirmation { Known(NativePreparationClosureCommit), RolledBack }
impl NativePreparationClosureCommit { pub(crate) fn matches_plan(&self,plan:&Arc<NativeQuotaClosurePlan>)->bool { Arc::ptr_eq(&self.original,plan) } }
impl NativeQuotaClosurePlan {
    pub(crate) fn matches(&self,actor:&Arc<NativePreparationActor>,lineage:&Arc<NativeReadyLineage>)->bool { Arc::ptr_eq(&self.actor,actor) && Arc::ptr_eq(&self.lineage,lineage) }
    fn validate_original(&self,tx:&Transaction<'_>)->Result<()> {
        self.actor.validate_original()?;ensure!(self.actor.is_revoked(),"nongrant preparation closure requires original revocation");
        self.no_dispatch.validate_original(&self.actor)?;no_registration(tx,self.actor.launch())?;
        self.no_dispatch.completion.commit.validate_inventory(tx)?;
        self.unit.validate_original(self.actor.launch().allocation().unit_snapshot())
    }
}
struct Snapshot { images: Images, leases: Vec<Row>, waiters: Vec<Row>, windows: Vec<Row>, history: Vec<Row>, legacy: Vec<LegacyCandidate> }
struct LegacyCandidate { unit: ExecutionUnit, raw: String, project: Body<Project>, goal: Body<Goal>, task: Body<Task>, digest: String }
impl LegacyCandidate {
    fn validate(&self, tx: &Transaction<'_>) -> Result<()> {
        ensure!(crate::state::managed_binding::unit_image_matches(tx,&self.unit,&self.raw)?, "legacy fairness Unit image changed");
        for (table,id,raw) in [("projects",self.project.parsed().id.to_string(),self.project.raw()),("goals",self.goal.parsed().id.to_string(),self.goal.raw()),("tasks",self.task.parsed().id.to_string(),self.task.raw())] {
            ensure!(tx.query_row(&format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id=?1 AND body=?2)"),params![id,raw],|r|r.get::<_,bool>(0))?, "legacy fairness parent image changed");
        }
        validate_unit_authority_facts(tx,&self.unit.authority(),&self.unit)?;
        validate_native_effect_open(&self.unit)?;
        crate::state::runtime::driver::validate(tx,self.unit.scope.task_id.context("legacy candidate Task absent")?)?;
        source_recovery::validate_task(tx,self.unit.scope.task_id.context("legacy candidate Task absent")?)?;
        validate_parent_activity_facts(&self.unit,self.project.parsed(),self.goal.parsed(),self.task.parsed())?;
        validate_governing_context_facts(tx,&self.unit,&self.digest)
    }
}

/// All shapes are qualified in one pass before copying any selected values.
fn rows(tx: &Transaction<'_>, sql: &str, values: &[SqlValue], limit: usize, body_column: Option<(usize,usize)>, scalar_bytes: &mut usize, body_bytes: &mut usize) -> Result<Vec<Row>> {
    let mut statement = tx.prepare(sql)?;
    let n = statement.column_count();
    {
        let columns=(0..n).map(|i|format!("c{i}")).collect::<Vec<_>>();
        let projection=columns.iter().map(|c|format!("typeof({c}),length(CAST({c} AS BLOB))")).collect::<Vec<_>>().join(",");
        let mut shapes=tx.prepare(&format!("WITH selected({}) AS ({sql}) SELECT {projection} FROM selected",columns.join(",")))?;
        let mut cursor = shapes.query(params_from_iter(values))?;
        let mut count = 0;
        while let Some(row) = cursor.next()? {
            count += 1; ensure!(count <= limit, "complete private quota inventory exceeds row bound");
            for i in 0..n {
                let (charge,max) = if body_column.is_some_and(|(c,_)| c==i) { (true,body_column.expect("checked body column").1) } else { (false,4096) };
                let kind:String=row.get(i*2)?;
                let length:Option<usize>=row.get(i*2+1)?;
                let bytes=match kind.as_str() { "text"=>{let length=length.context("quota shape text length absent")?;ensure!(length<=max,"private quota column exceeds bound");length},"integer"|"null" if !charge=>8,_=>anyhow::bail!("private quota column type differs") };
                let total = if charge { &mut *body_bytes } else { &mut *scalar_bytes };
                *total = total.checked_add(bytes).context("private quota inventory byte overflow")?;
                ensure!(*scalar_bytes <= 4*1024*1024 && *body_bytes <= 72*1024*1024, "private quota complete inventory byte bound exceeded");
            }
        }
    }
    let mut cursor = statement.query(params_from_iter(values))?;
    let mut output = Vec::new();
    while let Some(row) = cursor.next()? { output.push((0..n).map(|i|row.get(i)).collect::<rusqlite::Result<Row>>()?); }
    Ok(output)
}
fn text(row: &Row, i: usize) -> Result<&str> { match row.get(i) { Some(SqlValue::Text(v)) => Ok(v), _ => anyhow::bail!("private quota text absent") } }
fn integer(row: &Row, i: usize) -> Result<i64> { match row.get(i) { Some(SqlValue::Integer(v)) => Ok(*v), _ => anyhow::bail!("private quota integer absent") } }
fn t(s: impl Into<String>) -> SqlValue { SqlValue::Text(s.into()) }
fn i(n:i64) -> SqlValue { SqlValue::Integer(n) }
fn default_pool(provider:&str) -> Row { vec![t(provider),t("unknown"),i(0),SqlValue::Null,i(60_000),t("reviewer")] }

pub(crate) struct NativeQuotaPlan {
    actor: Arc<NativePreparationActor>,
    no_dispatch: Arc<PreparedPhaseNoCurrentDispatch>,
    pre: Arc<NativeReadyLineage>,
    at: i64,
    before: Snapshot,
    after: Images,
    readiness: PairRow,
    unit: Option<Arc<Body<ExecutionUnit>>>,
    decision: Decision,
}
pub(crate) struct NativeQuotaAdmitted { plan: Arc<NativeQuotaPlan>, lineage: Arc<NativeReadyLineage> }
pub(crate) struct NativeParkedPhase { plan: Arc<NativeQuotaPlan>, lineage: Arc<NativeReadyLineage>, due: i64, reason: WaitReason }
pub(crate) enum NativeQuotaOutcome { Admitted(Arc<NativeQuotaAdmitted>), Parked(Arc<NativeParkedPhase>) }
pub(crate) enum NativeQuotaConfirmation { Known(NativeQuotaOutcome), RolledBack }
pub(crate) enum NativeQuotaWrite { Known(NativeQuotaOutcome), Conflict }
impl NativeQuotaPlan {
    pub(crate) fn matches_pre(&self,pre:&Arc<NativeReadyLineage>,no_dispatch:&Arc<PreparedPhaseNoCurrentDispatch>) -> bool { Arc::ptr_eq(&self.pre,pre) && Arc::ptr_eq(&self.no_dispatch,no_dispatch) }
    pub(crate) fn matches_actor(&self, actor: &Arc<NativePreparationActor>) -> bool { Arc::ptr_eq(&self.actor,actor) }
    fn validate_negative(&self,tx:&Transaction<'_>) -> Result<()> { self.no_dispatch.validate_original(&self.actor)?; no_registration(tx,self.actor.launch())?; self.no_dispatch.completion.commit.validate_inventory(tx) }
    fn outcome(self: &Arc<Self>) -> Result<NativeQuotaOutcome> {
        // Called only after this plan's known commit or exact postimage confirm.
        let lineage = if let Some(unit) = &self.unit { self.pre.known_successor(unit.clone(),self.readiness.copy_image())? } else { self.pre.clone() };
        Ok(match self.decision { Decision::Admit { .. } => NativeQuotaOutcome::Admitted(Arc::new(NativeQuotaAdmitted { plan:self.clone(),lineage })), Decision::Wait { reason,due } => NativeQuotaOutcome::Parked(Arc::new(NativeParkedPhase { plan:self.clone(),lineage,due,reason })) })
    }
}
impl NativeQuotaAdmitted {
    pub(crate) fn matches_plan(&self,plan:&Arc<NativeQuotaPlan>) -> bool { Arc::ptr_eq(&self.plan,plan) }
    pub(crate) fn lineage(&self) -> &Arc<NativeReadyLineage> { &self.lineage }
    pub(crate) fn matches(&self,actor:&Arc<NativePreparationActor>,no_dispatch:&Arc<PreparedPhaseNoCurrentDispatch>) -> bool { self.plan.matches_actor(actor) && Arc::ptr_eq(&self.plan.no_dispatch,no_dispatch) }
}
impl NativeParkedPhase {
    pub(crate) fn matches_plan(&self,plan:&Arc<NativeQuotaPlan>) -> bool { Arc::ptr_eq(&self.plan,plan) }
    pub(crate) fn due(&self) -> i64 { self.due }
    pub(crate) fn reason(&self) -> WaitReason { self.reason }
    pub(crate) fn lineage(&self) -> &Arc<NativeReadyLineage> { &self.lineage }
}

fn read_snapshot(tx: &Transaction<'_>, lineage:&NativeReadyLineage, at:i64, legacy:bool) -> Result<Snapshot> {
    let unit = lineage.unit(); let provider = &unit.provider; let id = unit.id.to_string();
    let mut scalar_bytes=0; let mut body_bytes=0;
    let mut read = |sql:&str,v:Vec<SqlValue>,limit,body| rows(tx,sql,&v,limit,body,&mut scalar_bytes,&mut body_bytes);
    let pool = read(POOL,vec![t(provider)],1,None)?.pop();
    let effective = pool.clone().unwrap_or_else(|| default_pool(provider));
    let leases = read(LEASES,vec![],4096,None)?;
    let waiters = read(WAITERS,vec![t(provider),i(at),effective[5].clone()],4096,None)?;
    let windows = read(WINDOWS,vec![t(provider)],64,Some((4,8192)))?;
    let history = read(HISTORY,vec![t(unit.scope.task_id.context("quota Task absent")?.to_string()),t(&id)],256,Some((1,16*1024)))?;
    let waiter = read(OWN_WAITER,vec![t(&id)],1,None)?.pop();
    let lease = read(OWN_LEASE,vec![t(&id)],1,None)?.pop();
    let mut candidates=Vec::new();
    if legacy {
        for row in &waiters {
            if text(row,0)? == id { break; }
            if integer(row,14)? != 0 { continue; }
            let candidate_id = text(row,0)?;
            let raw_rows = read("SELECT body FROM execution_units WHERE id=?1",vec![t(candidate_id)],1,Some((0,16*1024)))?;
            let raw = text(raw_rows.first().context("legacy candidate absent")?,0)?.to_owned();
            let candidate: ExecutionUnit = Body::<ExecutionUnit>::decode(raw.clone(),16*1024)?.parsed().clone();
            ensure!(candidate.id.to_string()==candidate_id && crate::state::managed_binding::unit_image_matches(tx,&candidate,&raw)?, "candidate complete Unit index/body differs");
            let mut parent = |table:&str,id:String| -> Result<String> {
                let sql=format!("SELECT body FROM {table} WHERE id=?1");
                let r=read(&sql,vec![t(id)],1,Some((0,8*1024*1024)))?;
                Ok(text(r.first().context("legacy candidate parent absent")?,0)?.into())
            };
            let project=Body::<Project>::decode(parent("projects",candidate.scope.project_id.to_string())?,8*1024*1024)?;
            let goal=Body::<Goal>::decode(parent("goals",candidate.scope.goal_id.context("candidate Goal absent")?.to_string())?,8*1024*1024)?;
            let task=Body::<Task>::decode(parent("tasks",candidate.scope.task_id.context("candidate Task absent")?.to_string())?,8*1024*1024)?;
            let digest=governing_digest(project.parsed(),goal.parsed())?;
            let saved=LegacyCandidate { unit:candidate,raw,project,goal,task,digest };
            // Refused Legacy heads are skipped, as in the existing policy.
            if saved.validate(tx).is_ok() { candidates.push(saved); }
        }
    }
    ensure!(scalar_bytes.checked_add(body_bytes).is_some_and(|n|n<=8*1024*1024), "private quota owned plan exceeds 8-MiB profile");
    if pool.is_none() { ensure!(windows.is_empty() && waiters.is_empty() && leases.iter().all(|l|text(l,1).ok()!=Some(provider.as_str()) || text(l,2).ok()!=Some("unknown")), "cold pool has FK-dependent inventory"); }
    Ok(Snapshot { images:Images { pool,waiter,lease },leases,waiters,windows,history,legacy:candidates })
}

fn project_blocked(leases:&[Row],project:&str,task:&str,max:usize) -> Result<bool> {
    let mut tasks=std::collections::BTreeSet::new();
    for l in leases { if text(l,6)?==project { tasks.insert(text(l,7)?); } }
    Ok(!tasks.contains(task) && tasks.len()>=max)
}
fn policy(snapshot:&Snapshot,lineage:&NativeReadyLineage,caps:&NativeQuotaCaps,at:i64) -> Result<Decision> {
    let unit=lineage.unit(); let id=unit.id.to_string(); let provider=&unit.provider;
    let pool=snapshot.images.pool.clone().unwrap_or_else(||default_pool(provider));
    let observations=snapshot.windows.iter().map(|r| {
        let o:QuotaObservation=decode(text(r,4)?.into())?;
        ensure!(o.provider==text(r,0)? && o.account_key==text(r,1)? && o.bucket==text(r,2)? && o.observed_at==integer(r,3)?, "quota window body/index differs"); Ok(o)
    }).collect::<Result<Vec<_>>>()?;
    let exhausted=observations.iter().any(|o|o.status==QuotaStatus::Exhausted);
    let high=observations.iter().any(|o|o.status==QuotaStatus::Available && at.saturating_sub(o.observed_at)<=300_000 && o.used_percent.is_some_and(|p|p>=95.0));
    let executor_max=if high { caps.executor.min(1) } else { caps.executor };
    let provider_live=snapshot.leases.iter().filter(|l|text(l,1).ok()==Some(provider.as_str()) && text(l,2).ok()==Some("unknown")).count();
    let executor_live=snapshot.leases.iter().filter(|l|text(l,1).ok()==Some(provider.as_str()) && text(l,2).ok()==Some("unknown") && text(l,3).ok()==Some("executor")).count();
    let global_executor=snapshot.leases.iter().filter(|l|text(l,3).ok()==Some("executor")).count();
    let executor_blocked=executor_live>=executor_max || global_executor>=caps.global.saturating_sub(2).max(1);
    let project_max=caps.project.min(lineage.commit().project_limit());
    ensure!(project_max>0,"quota project concurrency invalid");
    let capacity=project_blocked(&snapshot.leases,&unit.scope.project_id.to_string(),&unit.scope.task_id.context("quota Task absent")?.to_string(),project_max)? || snapshot.leases.len()>=caps.global || provider_live>=caps.provider || (unit.kind==UnitKind::Executor && executor_blocked);
    let mut capacity_due=at;
    for r in &snapshot.history {
        let old:ExecutionUnit=Body::<ExecutionUnit>::decode(text(r,1)?.into(),16*1024)?.parsed().clone();
        ensure!(old.id.to_string()==text(r,0)? && old.scope==unit.scope,"capacity history identity differs");
        if old.provider==*provider && old.disposition==Disposition::CapacityInterrupted {
            ensure!(!old.native_effects_open && !old.result_finalization_open && old.work==Some(WorkOutcome::Unknown),"invalid capacity terminal");
            let due=old.capacity_retry_at.context("capacity recheck absent")?; ensure!(due>=old.created_at,"invalid capacity recheck"); capacity_due=capacity_due.max(due);
        }
    }
    let mut first=None;
    for row in &snapshot.waiters {
        let candidate=text(row,0)?;
        let max=caps.project.min(usize::try_from(integer(row,16)?)?);
        ensure!(max>0,"candidate project concurrency invalid");
        if project_blocked(&snapshot.leases,text(row,8)?,text(row,9)?,max)? || (text(row,7)?=="executor" && executor_blocked) { continue; }
        let class=quota_policy::classify(integer(row,14)?!=0,integer(row,15)?!=0,text(row,12)?=="preparing",text(row,13)?==text(row,3)?,text(row,6)?=="preparing",text(row,1)?==provider && text(row,2)?=="unknown",integer(row,4)?,at);
        if candidate==id || class==CandidateClass::MarkedParked || (class==CandidateClass::Legacy && snapshot.legacy.iter().any(|c|c.unit.id.to_string()==candidate)) { first=Some(candidate); break; }
    }
    let self_head=first==Some(id.as_str()) || (snapshot.images.waiter.is_none() && first.is_none());
    Ok(quota_policy::decide(&QuotaSnapshot { exhausted,next_probe_at:integer(&pool,2)?,foreign_probe:pool[3]!=SqlValue::Null && pool[3]!=t(&id),capacity_due,capacity_blocked:capacity,fair_head_is_self:self_head },at))
}
pub(crate) struct NativeQuotaCaps { pub(crate) global:usize,pub(crate) executor:usize,pub(crate) provider:usize,pub(crate) project:usize }

impl Store {
    pub(crate) fn plan_phase_quota(owner:&Arc<crate::execution::RuntimeOwner>,actor:Arc<NativePreparationActor>,no_dispatch:Arc<PreparedPhaseNoCurrentDispatch>,pre:Arc<NativeReadyLineage>,caps:NativeQuotaCaps,at:i64) -> Result<Arc<NativeQuotaPlan>> {
        ensure!(caps.global>0 && caps.global<=1024 && caps.executor>0 && caps.provider>0 && caps.provider<=1024 && caps.project>0,"invalid private quota concurrency");
        actor.validate_open()?; no_dispatch.validate_original(&actor)?;
        let before=snapshot(owner,|tx| { let budget=InventoryBudget::new(tx)?; budget.finish((|| { pre.validate_tx(tx)?; no_registration(tx,actor.launch())?; no_dispatch.completion.commit.validate_inventory(tx)?; read_snapshot(tx,&pre,at,true) })()) })?;
        let decision=policy(&before,&pre,&caps,at)?;
        let unit=pre.unit(); let id=unit.id.to_string(); let provider=&unit.provider;
        if let Some(l)=&before.images.lease { ensure!(text(l,0)?==id && text(l,1)?==provider && text(l,2)?=="unknown" && integer(l,5)?==0,"own pre-Session lease is foreign or active"); }
        let parked=pre.readiness().column("state")?==&t("parked");
        ensure!(parked==before.images.waiter.is_some(),"known lineage and own waiter differ");
        if let Some(w)=&before.images.waiter { ensure!(text(w,0)?==id && text(w,1)?==provider && text(w,2)?=="unknown" && text(w,6)?=="preparing" && unit.wait_reason.map(key).as_deref()==Some(text(w,3)?),"SAME parked waiter image differs"); }
        let mut after=before.images.clone(); let mut pool=after.pool.clone().unwrap_or_else(||default_pool(provider));
        let mut readiness=pre.readiness().copy_image(); let mut next=unit.clone();
        match decision {
            Decision::Admit { probe } => {
                after.waiter=None; after.lease=Some(vec![t(&id),t(provider),t("unknown"),t(key(unit.kind)),i(i64::try_from(unit.owner_epoch)?),i(1)]);
                pool[5]=t(if unit.kind==UnitKind::Executor {"executor"} else {"reviewer"});
                if probe { pool[3]=t(&id); pool[2]=i(at.saturating_add(integer(&pool,4)?)); pool[4]=i(integer(&pool,4)?.saturating_mul(2).min(1_800_000)); }
                if parked { next.wait_reason=None; readiness.transition_readiness("preparing",4,None,false)?; }
            },
            Decision::Wait { reason,due } => {
                let fairness=before.images.waiter.as_ref().map(|w|integer(w,5)).transpose()?.unwrap_or(at);
                after.waiter=Some(vec![t(&id),t(provider),t("unknown"),t(key(reason)),i(due),i(fairness),t("preparing")]);
                next.wait_reason=Some(reason);
                if !parked { readiness.transition_readiness("parked",3,Some(3),false)?; }
            },
        }
        after.pool=Some(pool);
        let changed=next.wait_reason!=unit.wait_reason;
        let unit=if changed { next.version=next.version.checked_add(1).filter(|v|*v<=i64::MAX as u64).context("Unit version exhausted")?; next.updated_at=at; Some(Arc::new(Body::<ExecutionUnit>::decode(serde_json::to_string(&next)?,16*1024)?)) } else { None };
        Ok(Arc::new(NativeQuotaPlan { actor,no_dispatch,pre,at,before,after,readiness,unit,decision }))
    }
}

fn image_matches(tx:&Transaction<'_>,table:&str,columns:&[&str],key_value:&SqlValue,image:&Option<Row>) -> Result<bool> {
    if let Some(row)=image {
        ensure!(row.len()==columns.len(),"incomplete exact quota image");
        let predicates=columns.iter().enumerate().map(|(i,c)|format!("{c} IS ?{}",i+1)).collect::<Vec<_>>().join(" AND ");
        Ok(tx.query_row(&format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE {predicates})"),params_from_iter(row),|r|r.get(0))?)
    } else { Ok(!tx.query_row(&format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE {} IS ?1)",columns[0]),[key_value],|r|r.get::<_,bool>(0))?) }
}
fn write_image(tx:&Transaction<'_>,table:&str,columns:&[&str],before:&Option<Row>,after:&Option<Row>) -> Result<()> {
    if before==after { return Ok(()); }
    let count=if let Some(old)=before {
        let predicates=columns.iter().enumerate().map(|(i,c)|format!("{c} IS ?{}",i+1)).collect::<Vec<_>>().join(" AND ");
        if let Some(next)=after {
            let set=columns.iter().enumerate().map(|(i,c)|format!("{c}=?{}",columns.len()+i+1)).collect::<Vec<_>>().join(",");
            let values=old.iter().chain(next.iter());
            tx.execute(&format!("UPDATE {table} SET {set} WHERE {predicates}"),params_from_iter(values))?
        } else { tx.execute(&format!("DELETE FROM {table} WHERE {predicates}"),params_from_iter(old))? }
    } else {
        let next=after.as_ref().context("exact quota insert image absent")?;
        let params=(1..=columns.len()).map(|i|format!("?{i}")).collect::<Vec<_>>().join(",");
        tx.execute(&format!("INSERT INTO {table}({}) SELECT {params} WHERE NOT EXISTS(SELECT 1 FROM {table} WHERE {} IS ?1)",columns.join(","),columns[0]),params_from_iter(next))?
    };
    ensure!(count==1,"private exact quota write CAS changed"); Ok(())
}
const POOL_COLUMNS:&[&str]=&["provider","account_key","next_probe_at","probe_unit","backoff","last_role"];
const WAITER_COLUMNS:&[&str]=&["unit_id","provider","account_key","reason","next_due","fairness_sequence","resume_state"];
const LEASE_COLUMNS:&[&str]=&["unit_id","provider","account_key","role","epoch","active"];

fn images_match(tx:&Transaction<'_>,plan:&NativeQuotaPlan,images:&Images)->Result<bool> {
    let f=plan.actor.launch().allocation().facts();
    // Pool identity includes provider AND account. Other accounts are unrelated.
    let pool:Option<Row>={ let mut scalars=0;let mut bodies=0; rows(tx,POOL,&[t(f.provider)],1,None,&mut scalars,&mut bodies)?.pop() };
    Ok(pool==images.pool && image_matches(tx,"quota_waiters",WAITER_COLUMNS,&t(f.unit_id.to_string()),&images.waiter)? && image_matches(tx,"quota_leases",LEASE_COLUMNS,&t(f.unit_id.to_string()),&images.lease)?)
}
fn read_images(tx:&Transaction<'_>,actor:&Arc<NativePreparationActor>)->Result<Images> {
    let f=actor.launch().allocation().facts();let mut scalar=0;let mut body=0;
    Ok(Images {
        pool:rows(tx,POOL,&[t(f.provider)],1,None,&mut scalar,&mut body)?.pop(),
        waiter:rows(tx,OWN_WAITER,&[t(f.unit_id.to_string())],1,None,&mut scalar,&mut body)?.pop(),
        lease:rows(tx,OWN_LEASE,&[t(f.unit_id.to_string())],1,None,&mut scalar,&mut body)?.pop(),
    })
}
fn sort_waiters(waiters:&mut [Row],last:&str) {
    waiters.sort_by(|a,b| {
        let ar=(text(a,7).ok()==Some("executor"))==(last=="executor");
        let br=(text(b,7).ok()==Some("executor"))==(last=="executor");
        (ar,integer(a,5).ok(),text(a,0).ok()).cmp(&(br,integer(b,5).ok(),text(b,0).ok()))
    });
}
fn expected_inventories(plan:&NativeQuotaPlan,post:bool)->Result<(Vec<Row>,Vec<Row>)> {
    let mut leases=plan.before.leases.clone(); let mut waiters=plan.before.waiters.clone();
    if post {
        let unit=plan.pre.unit(); let id=unit.id.to_string();
        leases.retain(|l|text(l,0).ok()!=Some(id.as_str()));
        if let Some(l)=&plan.after.lease && integer(l,5)?==1 {
            let mut row=l.clone(); row.extend([t(unit.scope.project_id.to_string()),t(unit.scope.task_id.context("quota Task absent")?.to_string()),t(key(unit.kind))]); leases.push(row);
        }
        leases.sort_by(|a,b|text(a,0).ok().cmp(&text(b,0).ok()));
        waiters.retain(|w|text(w,0).ok()!=Some(id.as_str()));
        if let Some(w)=&plan.after.waiter && integer(w,4)?<=plan.at { anyhow::bail!("quota post-waiter unexpectedly due in same claim"); }
        sort_waiters(&mut waiters,text(plan.after.pool.as_ref().context("quota post-pool absent")?,5)?);
    }
    Ok((leases,waiters))
}
fn inventories_match(tx:&Transaction<'_>,plan:&NativeQuotaPlan,post:bool)->Result<bool> {
    let actual=read_snapshot(tx,&plan.pre,plan.at,false)?;
    let (leases,waiters)=expected_inventories(plan,post)?;
    Ok(actual.leases==leases && actual.waiters==waiters && actual.windows==plan.before.windows && actual.history==plan.before.history)
}
fn validate_post_unit(tx:&Transaction<'_>,plan:&NativeQuotaPlan)->Result<()> {
    let (unit,raw)=plan.unit.as_ref().map_or((plan.pre.unit(),plan.pre.current().unit_raw()),|b|(b.parsed(),b.raw()));
    ensure!(crate::state::managed_binding::unit_image_matches(tx,unit,raw)?,"quota own complete Unit image changed"); Ok(())
}
fn apply_images(tx:&Transaction<'_>,plan:&NativeQuotaPlan)->Result<()> {
    let f=plan.actor.launch().allocation().facts();
    if plan.before.images.pool.is_none() {
        // Exact absence, no ON CONFLICT fallback. Required before every FK row.
        let default=insert_cold_pool(tx,f.provider)?;
        write_image(tx,"quota_pools",POOL_COLUMNS,&Some(default),&plan.after.pool)?;
    } else { write_image(tx,"quota_pools",POOL_COLUMNS,&plan.before.images.pool,&plan.after.pool)?; }
    write_image(tx,"quota_waiters",WAITER_COLUMNS,&plan.before.images.waiter,&plan.after.waiter)?;
    write_image(tx,"quota_leases",LEASE_COLUMNS,&plan.before.images.lease,&plan.after.lease)?;
    if plan.readiness.values!=plan.pre.readiness().values { plan.pre.readiness().update_tx(tx,&plan.readiness)?; }
    if let Some(body)=&plan.unit {
        let n=tx.execute("UPDATE execution_units SET version=?1,body=?2 WHERE id=?3 AND version=?4 AND body=?5",params![body.parsed().version,body.raw(),f.unit_id.to_string(),plan.pre.unit().version,plan.pre.current().unit_raw()])?;
        ensure!(n==1,"quota complete Unit CAS changed");
    }
    Ok(())
}
fn insert_cold_pool(tx:&Transaction<'_>,provider:&str)->Result<Row> {
    let default=default_pool(provider);
    let n=tx.execute("INSERT INTO quota_pools(provider,account_key,next_probe_at,probe_unit,backoff,last_role) SELECT ?1,?2,?3,?4,?5,?6 WHERE NOT EXISTS(SELECT 1 FROM quota_pools WHERE provider=?1 AND account_key=?2)",params_from_iter(&default))?;
    ensure!(n==1,"cold private quota pool absence changed");Ok(default)
}

#[cfg(test)]
mod primitive_tests {
    use super::*;
    fn db()->Connection {let c=Connection::open_in_memory().unwrap();c.execute_batch("PRAGMA foreign_keys=ON;CREATE TABLE quota_pools(provider TEXT,account_key TEXT,next_probe_at INTEGER,probe_unit TEXT,backoff INTEGER,last_role TEXT,PRIMARY KEY(provider,account_key));CREATE TABLE quota_waiters(unit_id TEXT PRIMARY KEY,provider TEXT,account_key TEXT,reason TEXT,next_due INTEGER,fairness_sequence INTEGER,resume_state TEXT,FOREIGN KEY(provider,account_key) REFERENCES quota_pools(provider,account_key));").unwrap();c}
    #[test]
    fn nongrant_native_cold_pool_is_complete_exact_absence_and_preserves_foreign_accounts() {
        let mut c=db();let tx=c.transaction().unwrap();
        tx.execute("INSERT INTO quota_pools VALUES('claude','other',9,'foreign',120000,'executor')",[]).unwrap();
        let image=insert_cold_pool(&tx,"claude").unwrap();assert_eq!(image,vec![t("claude"),t("unknown"),i(0),SqlValue::Null,i(60000),t("reviewer")]);
        let mut s=0;let mut b=0;assert_eq!(rows(&tx,POOL,&[t("claude")],1,None,&mut s,&mut b).unwrap(),[image]);
        assert!(insert_cold_pool(&tx,"claude").is_err());
        let waiter=vec![t("unit"),t("claude"),t("unknown"),t("capacity"),i(1000),i(0),t("preparing")];
        write_image(&tx,"quota_waiters",WAITER_COLUMNS,&None,&Some(waiter)).unwrap();
        assert_eq!(tx.query_row("SELECT next_probe_at FROM quota_pools WHERE account_key='other'",[],|r|r.get::<_,i64>(0)).unwrap(),9);
        tx.commit().unwrap();
    }
    #[test]
    fn nongrant_native_pool_cas_matches_every_column_without_partial_write() {
        for column in 0..6 {
            let mut c=db();let tx=c.transaction().unwrap();let before=insert_cold_pool(&tx,"claude").unwrap();
            let replacement=match column {0=>t("codex"),1=>t("other"),2=>i(1),3=>t("foreign"),4=>i(120000),_=>t("executor")};
            let mut actual=before.clone();actual[column]=replacement.clone();
            tx.execute(&format!("UPDATE quota_pools SET {}=?1",POOL_COLUMNS[column]),[replacement]).unwrap();
            let mut after=before.clone();after[5]=t("executor");
            assert!(write_image(&tx,"quota_pools",POOL_COLUMNS,&Some(before),&Some(after)).is_err(),"column {column}");
            assert!(image_matches(&tx,"quota_pools",POOL_COLUMNS,&actual[0],&Some(actual)).unwrap());
        }
    }
    #[test]
    fn nongrant_native_waiter_delete_requires_complete_seven_column_image() {
        for column in 0..7 {
            let mut c=db();let tx=c.transaction().unwrap();insert_cold_pool(&tx,"claude").unwrap();
            let before=vec![t("unit"),t("claude"),t("unknown"),t("capacity"),i(1000),i(1),t("preparing")];
            write_image(&tx,"quota_waiters",WAITER_COLUMNS,&None,&Some(before.clone())).unwrap();
            // provider/account changes need another legitimate FK pool.
            tx.execute("INSERT INTO quota_pools VALUES('codex','other',0,NULL,60000,'reviewer')",[]).unwrap();
            tx.execute("INSERT INTO quota_pools VALUES('codex','unknown',0,NULL,60000,'reviewer')",[]).unwrap();
            tx.execute("INSERT INTO quota_pools VALUES('claude','other',0,NULL,60000,'reviewer')",[]).unwrap();
            let replacement=match column {0=>t("other-unit"),1=>t("codex"),2=>t("other"),3=>t("quota"),4=>i(2000),5=>i(2),_=>t("running")};
            tx.execute(&format!("UPDATE quota_waiters SET {}=?1",WAITER_COLUMNS[column]),[replacement]).unwrap();
            assert!(write_image(&tx,"quota_waiters",WAITER_COLUMNS,&Some(before),&None).is_err(),"column {column}");
            assert_eq!(tx.query_row("SELECT count(*) FROM quota_waiters",[],|r|r.get::<_,usize>(0)).unwrap(),1);
        }
    }
    #[test]
    fn nongrant_native_quota_inventory_limit_plus_one_and_byte_bounds_refuse() {
        let mut c=Connection::open_in_memory().unwrap();c.execute_batch("CREATE TABLE inventory(id INTEGER,body TEXT)").unwrap();
        let tx=c.transaction().unwrap();
        for limit in [64usize,256,4096] {
            tx.execute("DELETE FROM inventory",[]).unwrap();
            for n in 0..limit {tx.execute("INSERT INTO inventory VALUES(?1,'x')",[n]).unwrap();}
            let mut s=0;let mut b=0;assert_eq!(rows(&tx,"SELECT id,body FROM inventory",&[],limit,Some((1,8192)),&mut s,&mut b).unwrap().len(),limit);
            tx.execute("INSERT INTO inventory VALUES(?1,'x')",[limit]).unwrap();let mut s=0;let mut b=0;assert!(rows(&tx,"SELECT id,body FROM inventory",&[],limit,Some((1,8192)),&mut s,&mut b).is_err());
        }
        tx.execute("DELETE FROM inventory",[]).unwrap();tx.execute("INSERT INTO inventory VALUES(1,?1)",["x".repeat(8192)]).unwrap();
        let mut s=0;let mut b=0;assert!(rows(&tx,"SELECT id,body FROM inventory",&[],1,Some((1,8192)),&mut s,&mut b).is_ok());
        tx.execute("UPDATE inventory SET body=?1",["x".repeat(8193)]).unwrap();let mut s=0;let mut b=0;assert!(rows(&tx,"SELECT id,body FROM inventory",&[],1,Some((1,8192)),&mut s,&mut b).is_err());
    }
}
impl Store {
    pub(crate) fn plan_phase_quota_closure(owner:&Arc<crate::execution::RuntimeOwner>,actor:Arc<NativePreparationActor>,no_dispatch:Arc<PreparedPhaseNoCurrentDispatch>,lineage:Arc<NativeReadyLineage>,at:i64)->Result<Arc<NativeQuotaClosurePlan>> {
        actor.validate_original()?;ensure!(actor.is_revoked(),"preparation closure not revoked");
        ensure!(std::ptr::eq(actor.launch().allocation().selected_port().owner(),owner.as_ref()),"nongrant closure selected another owner");
        no_dispatch.validate_original(&actor)?;
        let (unit,before)=snapshot(owner,|tx| {
            let budget=InventoryBudget::new(tx)?;
            budget.finish((|| {
                lineage.validate_closure_tx(tx)?;no_dispatch.completion.commit.validate_inventory(tx)?;
                let unit=super::version::LatestUnitImage::read(tx,actor.launch().allocation().unit_snapshot())?;
                Ok((unit,read_images(tx,&actor)?))
            })())
        })?;
        let f=actor.launch().allocation().facts();let mut after=before.clone();
        if let Some(w)=&before.waiter { ensure!(text(w,0)?==f.unit_id.to_string() && text(w,1)?==f.provider && text(w,2)?=="unknown" && text(w,6)?=="preparing","closure own waiter identity differs"); }
        after.waiter=None;
        if let Some(l)=&mut after.lease {
            ensure!(text(l,0)?==f.unit_id.to_string() && text(l,1)?==f.provider && text(l,2)?=="unknown" && integer(l,4)?==i64::try_from(f.epoch)?,"closure own lease identity/epoch differs");
            l[5]=i(0);
        }
        if let Some(p)=&mut after.pool && p[3]==t(f.unit_id.to_string()) {
            p[3]=SqlValue::Null;p[2]=i(integer(p,2)?.max(at.saturating_add(integer(p,4)?)));
        }
        let mut readiness=lineage.readiness().copy_image();let version=match readiness.column("version")? {SqlValue::Integer(v)=>v.checked_add(1).context("readiness closure version exhausted")?,_=>anyhow::bail!("readiness closure version absent")};
        readiness.transition_readiness("closed",version,None,true)?;
        Ok(Arc::new(NativeQuotaClosurePlan {actor,no_dispatch,lineage,unit,before,after,readiness,at}))
    }
    pub(crate) fn close_phase_quota(&mut self,plan:Arc<NativeQuotaClosurePlan>)->Result<Option<NativePreparationClosureCommit>> {
        selected_database(&self.connection,plan.actor.launch())?;
        let mutation=plan.lineage.readiness().update_permission(&plan.readiness)?;
        let committed=self.binding_permits.with_exact_permit(vec![mutation],|| {
            let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            {
                let budget=InventoryBudget::new(&tx)?;
                let same=budget.finish((|| {
                    plan.validate_original(&tx)?;plan.lineage.validate_closure_tx(&tx)?;
                    ensure!(now_ms().abs_diff(plan.at)<=5000,"preparation closure plan clock stale");
                    if read_images(&tx,&plan.actor)?!=plan.before || plan.unit.validate_tx(&tx).is_err() { return Ok(false); }
                    write_image(&tx,"quota_pools",POOL_COLUMNS,&plan.before.pool,&plan.after.pool)?;
                    write_image(&tx,"quota_waiters",WAITER_COLUMNS,&plan.before.waiter,&plan.after.waiter)?;
                    write_image(&tx,"quota_leases",LEASE_COLUMNS,&plan.before.lease,&plan.after.lease)?;
                    plan.lineage.readiness().update_tx(&tx,&plan.readiness)?;
                    self.binding_permits.ensure_consumed()?;Ok(true)
                })())?;
                if !same { return Ok(false); }
            }
            tx.commit()?;Ok(true)
        })?;
        Ok(committed.then_some(NativePreparationClosureCommit {original:plan}))
    }
    pub(crate) fn confirm_phase_quota_closure(&mut self,plan:Arc<NativeQuotaClosurePlan>)->Result<NativeQuotaClosureConfirmation> {
        selected_database(&self.connection,plan.actor.launch())?;
        let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let post={
            let budget=InventoryBudget::new(&tx)?;
            budget.finish((|| {
                plan.validate_original(&tx)?;plan.unit.validate_tx(&tx)?;
                let actual=read_images(&tx,&plan.actor)?;
                if actual==plan.after && plan.readiness.validate_tx(&tx).is_ok() { return Ok(true); }
                ensure!(actual==plan.before && plan.lineage.validate_closure_tx(&tx).is_ok(),"uncertain preparation closure mixed images; Held");Ok(false)
            })())?
        };
        tx.commit()?;
        Ok(if post { NativeQuotaClosureConfirmation::Known(NativePreparationClosureCommit {original:plan}) } else { NativeQuotaClosureConfirmation::RolledBack })
    }
    pub(crate) fn commit_phase_quota(&mut self,plan:Arc<NativeQuotaPlan>,admission:&PhaseEffectAdmissionGuard)->Result<NativeQuotaWrite> {
        selected_database(&self.connection,plan.actor.launch())?;
        let mutation=(plan.readiness.values!=plan.pre.readiness().values).then(||plan.pre.readiness().update_permission(&plan.readiness)).transpose()?;
        let permitted=mutation.is_some();
        let mut write=|| -> Result<bool> {
            let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            {
                let budget=InventoryBudget::new(&tx)?;
                let same=budget.finish((|| {
                    admission.validate_for(plan.actor.launch())?;plan.actor.validate_open()?;
                    plan.pre.validate_tx(&tx)?;plan.validate_negative(&tx)?;
                    ensure!(now_ms().abs_diff(plan.at)<=5000,"private quota plan clock stale");
                    if !images_match(&tx,&plan,&plan.before.images)? || !inventories_match(&tx,&plan,false)? { return Ok(false); }
                    // Until the reviewed hashing/Legacy contract delta exists,
                    // no alternative validator silently substitutes for it.
                    ensure!(plan.before.legacy.is_empty(),"private quota Legacy-head contract unresolved");
                    apply_images(&tx,&plan)?;
                    if permitted { self.binding_permits.ensure_consumed()?; }
                    Ok(true)
                })())?;
                if !same { return Ok(false); }
            }
            tx.commit()?; Ok(true)
        };
        let known=if let Some(mutation)=mutation { self.binding_permits.with_exact_permit(vec![mutation],write)? } else { write()? };
        if known { Ok(NativeQuotaWrite::Known(plan.outcome()?)) } else { Ok(NativeQuotaWrite::Conflict) }
    }
    pub(crate) fn confirm_phase_quota(&mut self,plan:Arc<NativeQuotaPlan>,admission:&PhaseEffectAdmissionGuard)->Result<NativeQuotaConfirmation> {
        selected_database(&self.connection,plan.actor.launch())?;
        let tx=self.connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let post={
            let budget=InventoryBudget::new(&tx)?;
            budget.finish((|| {
                admission.validate_for(plan.actor.launch())?;plan.actor.validate_open()?;plan.validate_negative(&tx)?;
                let post=images_match(&tx,&plan,&plan.after)? && inventories_match(&tx,&plan,true)? && plan.readiness.validate_tx(&tx).is_ok() && validate_post_unit(&tx,&plan).is_ok();
                if post { return Ok(true); }
                ensure!(images_match(&tx,&plan,&plan.before.images)? && inventories_match(&tx,&plan,false)? && plan.pre.validate_tx(&tx).is_ok(),"quota uncertain commit has mixed images; Held");
                Ok(false)
            })())?
        };
        tx.commit()?;
        if post { Ok(NativeQuotaConfirmation::Known(plan.outcome()?)) } else { Ok(NativeQuotaConfirmation::RolledBack) }
    }
}
