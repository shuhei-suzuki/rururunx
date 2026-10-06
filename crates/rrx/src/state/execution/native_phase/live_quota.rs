//! Registered quota callbacks require the SAME Live actor and exact own lease.
//! Pool/window observations cannot reconstruct private quota-probe authority.
use super::*;
use rusqlite::types::ValueRef;

type Row = Vec<SqlValue>;
const LEASE: &str =
    "SELECT unit_id,provider,account_key,role,epoch,active FROM quota_leases WHERE unit_id=?1";
const POOL: &str = "SELECT provider,account_key,next_probe_at,probe_unit,backoff,last_role FROM quota_pools WHERE provider=?1 AND account_key='unknown'";
const WINDOWS: &str = "SELECT provider,account_key,bucket,observed_at,body FROM quota_windows WHERE provider=?1 AND account_key='unknown' ORDER BY bucket LIMIT 65";
fn row(tx: &Transaction<'_>, sql: &str, key: &str, columns: usize) -> Result<Row> {
    Ok(tx.query_row(sql, [key], |r| {
        (0..columns)
            .map(|i| r.get(i))
            .collect::<rusqlite::Result<Row>>()
    })?)
}
fn windows(tx: &Transaction<'_>, provider: &str) -> Result<(Vec<Row>, Vec<QuotaObservation>)> {
    let mut statement = tx.prepare(WINDOWS)?;
    let mut cursor = statement.query([provider])?;
    let mut images = Vec::new();
    let mut observations = Vec::new();
    let mut total = 0usize;
    while let Some(r) = cursor.next()? {
        ensure!(images.len() < 64, "registered quota window bound exceeded");
        let mut image = Vec::new();
        for i in 0..5 {
            if let ValueRef::Text(bytes) = r.get_ref(i)? {
                ensure!(bytes.len() <= 16 * 1024, "quota image byte bound exceeded");
                total = total
                    .checked_add(bytes.len())
                    .context("quota image size overflow")?;
                ensure!(
                    total <= 1024 * 1024,
                    "quota window total byte bound exceeded"
                );
            }
            image.push(r.get::<_, SqlValue>(i)?);
        }
        let SqlValue::Text(raw) = &image[4] else {
            anyhow::bail!("quota window body absent")
        };
        let value = crate::execution::strict_json::decode(
            raw.as_bytes(),
            crate::execution::strict_json::Limits {
                frame_bytes: 16 * 1024,
                depth: 8,
                nodes: 128,
                string_bytes: 4096,
                total_string_bytes: 16 * 1024,
                object_entries: 32,
                array_entries: 16,
            },
        )?;
        let o: QuotaObservation = serde_json::from_value(value)?;
        valid_observation(&o)?;
        ensure!(
            image[0] == SqlValue::Text(o.provider.clone())
                && image[1] == SqlValue::Text(o.account_key.clone())
                && image[2] == SqlValue::Text(o.bucket.clone())
                && image[3] == SqlValue::Integer(o.observed_at)
                && o.provider == provider
                && o.account_key == "unknown",
            "quota window complete index/body differs"
        );
        observations.push(o);
        images.push(image);
    }
    Ok((images, observations))
}
struct Images {
    lease: Row,
    pool: Row,
    windows: Vec<Row>,
    observations: Vec<QuotaObservation>,
}
fn read(tx: &Transaction<'_>, owner: &NativeOwnerPlan) -> Result<Images> {
    let unit = owner.current.unit();
    let f = owner.binding.allocation().facts();
    let lease = row(tx, LEASE, &unit.id.to_string(), 6)?;
    let expected = vec![
        SqlValue::Text(f.unit_id.to_string()),
        SqlValue::Text(f.provider.into()),
        SqlValue::Text("unknown".into()),
        SqlValue::Text(key(unit.kind)),
        SqlValue::Integer(i64::try_from(f.epoch)?),
        SqlValue::Integer(1),
    ];
    ensure!(
        lease == expected,
        "registered quota own active lease differs"
    );
    let pool = row(tx, POOL, &unit.provider, 6)?;
    ensure!(
        pool[0] == SqlValue::Text(f.provider.into())
            && pool[1] == SqlValue::Text("unknown".into())
            && matches!(pool[2], SqlValue::Integer(_))
            && matches!(pool[4], SqlValue::Integer(60_000..=1_800_000))
            && matches!(&pool[5],SqlValue::Text(role) if matches!(role.as_str(),"executor"|"reviewer"))
            && match &pool[3] {
                SqlValue::Null => true,
                SqlValue::Text(id) => uuid::Uuid::parse_str(id).is_ok_and(|v| v.to_string() == *id),
                _ => false,
            },
        "registered quota pool bounded identity differs"
    );
    let (windows, observations) = windows(tx, &unit.provider)?;
    Ok(Images {
        lease,
        pool,
        windows,
        observations,
    })
}
enum Mutation {
    None,
    Observation {
        value: QuotaObservation,
        raw: String,
        next_probe: i64,
        reset_backoff: bool,
    },
    Unit {
        value: ExecutionUnit,
        raw: String,
        retry: bool,
        event: &'static str,
    },
}
pub(crate) struct NativeLiveQuotaPlan {
    owner: NativeOwnerPlan,
    before: Images,
    mutation: Mutation,
}
impl NativeLiveQuotaPlan {
    pub(crate) fn is_own_probe(&self) -> bool {
        self.before.pool[3] == SqlValue::Text(self.owner.current.unit().id.to_string())
    }
    pub(crate) fn observations(&self) -> &[QuotaObservation] {
        &self.before.observations
    }
    fn validate_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        self.owner.validate_tx(tx)?;
        let actual = read(tx, &self.owner)?;
        ensure!(
            actual.lease == self.before.lease
                && actual.pool == self.before.pool
                && actual.windows == self.before.windows,
            "registered quota complete plan images changed"
        );
        Ok(())
    }
}
fn plan(
    runtime: &crate::execution::RuntimeOwner,
    phase: &Arc<NativePhaseSession>,
) -> Result<NativeLiveQuotaPlan> {
    let owner = plan_native_owner(runtime, phase)?;
    let before = snapshot(runtime, |tx| {
        let budget = super::version::InventoryBudget::new(tx)?;
        budget.finish((|| {
            owner.validate_tx(tx)?;
            read(tx, &owner)
        })())
    })?;
    Ok(NativeLiveQuotaPlan {
        owner,
        before,
        mutation: Mutation::None,
    })
}
fn valid_observation(o: &QuotaObservation) -> Result<()> {
    ensure!(
        [
            &o.provider,
            &o.account_key,
            &o.bucket,
            &o.window_id,
            &o.source_version
        ]
        .iter()
        .all(|s| !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control))
            && o.used_percent
                .is_none_or(|p| p.is_finite() && (0.0..=100.0).contains(&p))
            && (o.status != QuotaStatus::Exhausted || o.confirmed_subscription),
        "invalid registered quota observation"
    );
    Ok(())
}
impl Store {
    pub(crate) fn plan_phase_quota_read(
        runtime: &crate::execution::RuntimeOwner,
        phase: &Arc<NativePhaseSession>,
    ) -> Result<NativeLiveQuotaPlan> {
        plan(runtime, phase)
    }
    pub(crate) fn plan_phase_quota_observation(
        runtime: &crate::execution::RuntimeOwner,
        phase: &Arc<NativePhaseSession>,
        o: &QuotaObservation,
        recovery: bool,
    ) -> Result<NativeLiveQuotaPlan> {
        valid_observation(o)?;
        let mut p = plan(runtime, phase)?;
        ensure!(
            o.provider == p.owner.current.unit().provider && o.account_key == "unknown",
            "registered quota observation provider/account differs"
        );
        ensure!(
            !recovery || p.is_own_probe(),
            "registered quota recovery lacks SAME active own probe"
        );
        let prior = p.observations().iter().find(|old| old.bucket == o.bucket);
        if !super::super::quota_observation::applies(prior, o, recovery) {
            return Ok(p);
        }
        if prior.is_none() {
            ensure!(
                p.observations().len() < 64,
                "registered quota new window bound exceeded"
            );
        }
        let SqlValue::Integer(next) = p.before.pool[2] else {
            anyhow::bail!("quota next probe absent")
        };
        let next_probe = if o.status == QuotaStatus::Exhausted {
            next.max(
                o.resets_at
                    .unwrap_or_else(|| o.observed_at.saturating_add(60_000)),
            )
        } else {
            next
        };
        let reset_backoff = o.status == QuotaStatus::Available
            && !p
                .observations()
                .iter()
                .any(|old| old.bucket != o.bucket && old.status == QuotaStatus::Exhausted);
        p.mutation = Mutation::Observation {
            value: o.clone(),
            raw: serde_json::to_string(o)?,
            next_probe,
            reset_backoff,
        };
        Ok(p)
    }
    pub(crate) fn plan_phase_quota_wait(
        runtime: &crate::execution::RuntimeOwner,
        phase: &Arc<NativePhaseSession>,
        retry: bool,
    ) -> Result<NativeLiveQuotaPlan> {
        let mut p = plan(runtime, phase)?;
        let mut unit = p.owner.current.unit().clone();
        ensure!(
            matches!(unit.state, UnitState::Running | UnitState::WaitingQuota)
                && (!retry || unit.provider == "codex"),
            "registered live quota turn unavailable"
        );
        unit.state = UnitState::WaitingQuota;
        unit.wait_reason = Some(WaitReason::Quota);
        unit.version = unit
            .version
            .checked_add(1)
            .context("quota Unit version exhausted")?;
        unit.updated_at = now_ms();
        p.mutation = Mutation::Unit {
            raw: serde_json::to_string(&unit)?,
            value: unit,
            retry,
            event: if retry {
                "execution.native_quota_retry"
            } else {
                "execution.native_quota_wait"
            },
        };
        Ok(p)
    }
    pub(crate) fn plan_phase_quota_resume(
        runtime: &crate::execution::RuntimeOwner,
        phase: &Arc<NativePhaseSession>,
        buckets: &std::collections::BTreeSet<String>,
    ) -> Result<NativeLiveQuotaPlan> {
        ensure!(
            !buckets.is_empty() && buckets.len() <= 5,
            "invalid registered quota buckets"
        );
        let mut p = plan(runtime, phase)?;
        let mut unit = p.owner.current.unit().clone();
        ensure!(
            unit.provider == "claude",
            "registered plan-window recovery requires Claude"
        );
        let available = buckets.iter().all(|bucket| {
            p.observations()
                .iter()
                .any(|o| &o.bucket == bucket && o.status == QuotaStatus::Available)
        });
        if unit.state == UnitState::WaitingQuota
            && unit.wait_reason == Some(WaitReason::Quota)
            && available
        {
            unit.state = UnitState::Running;
            unit.wait_reason = None;
            unit.version = unit
                .version
                .checked_add(1)
                .context("quota Unit version exhausted")?;
            unit.updated_at = now_ms();
            p.mutation = Mutation::Unit {
                raw: serde_json::to_string(&unit)?,
                value: unit,
                retry: false,
                event: "execution.native_quota_recovered",
            };
        }
        Ok(p)
    }
    pub(crate) fn apply_phase_live_quota(
        &mut self,
        plan: NativeLiveQuotaPlan,
    ) -> Result<ExecutionUnit> {
        selected_database(&self.connection, plan.owner.binding.owner().launch_parts())?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = {
            let budget = super::version::InventoryBudget::new(&tx)?;
            budget.finish((|| {
                plan.validate_tx(&tx)?;
                match &plan.mutation {
                    Mutation::None=>Ok(plan.owner.current.unit().clone()),
                    Mutation::Observation {value:o,raw,next_probe,reset_backoff}=>{
                        tx.execute("INSERT INTO quota_windows(provider,account_key,bucket,observed_at,body) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(provider,account_key,bucket) DO UPDATE SET observed_at=excluded.observed_at,body=excluded.body",params![o.provider,o.account_key,o.bucket,o.observed_at,raw])?;
                        tx.execute("UPDATE quota_pools SET next_probe_at=?1,backoff=CASE WHEN ?2 THEN 60000 ELSE backoff END WHERE provider=?3 AND account_key='unknown'",params![next_probe,reset_backoff,o.provider])?;
                        Ok(plan.owner.current.unit().clone())
                    },
                    Mutation::Unit {value:u,raw,retry,event}=>{
                        let old=plan.owner.current.unit();
                        ensure!(tx.execute("UPDATE execution_units SET version=?1,body=?2 WHERE id=?3 AND version=?4 AND body=?5",params![u.version,raw,u.id.to_string(),old.version,plan.owner.current.unit_raw()])?==1,"registered quota Unit CAS changed");
                        if *retry {tx.execute("UPDATE quota_pools SET probe_unit=?1 WHERE provider=?2 AND account_key='unknown' AND probe_unit IS NULL",params![u.id.to_string(),u.provider])?;}
                        append_event(&tx,&u.scope,event,json!({"unit":u.id,"session":u.session_id}))?;Ok(u.clone())
                    },
                }
            })())?
        };
        tx.commit()?;
        Ok(result)
    }
}
