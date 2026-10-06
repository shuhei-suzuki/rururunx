use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuotaAdmission {
    Admitted,
    Waiting { reason: WaitReason, next_due: i64 },
}

impl Store {
    pub fn quota_observations(
        &self,
        provider: &str,
        account: &str,
    ) -> Result<Vec<QuotaObservation>> {
        windows(&self.connection, provider, account)
    }
    pub(crate) fn observe_quota(&mut self, observation: &QuotaObservation) -> Result<()> {
        self.observe_quota_inner(observation, None)
    }
    /// A correlated fresh native response from the one admitted recovery probe.
    /// Unsolicited observations cannot clear an exhaustion with no known reset.
    pub(crate) fn observe_quota_from_probe(
        &mut self,
        observation: &QuotaObservation,
        authority: &ExecutionAuthority,
    ) -> Result<()> {
        self.observe_quota_inner(observation, Some(authority))
    }
    fn observe_quota_inner(
        &mut self,
        observation: &QuotaObservation,
        probe_authority: Option<&ExecutionAuthority>,
    ) -> Result<()> {
        ensure!(
            [
                &observation.provider,
                &observation.account_key,
                &observation.bucket,
                &observation.window_id,
                &observation.source_version
            ]
            .iter()
            .all(|s| !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)),
            "invalid quota identity"
        );
        ensure!(
            observation
                .used_percent
                .is_none_or(|p| p.is_finite() && (0.0..=100.0).contains(&p)),
            "invalid quota percentage"
        );
        ensure!(
            observation.status != QuotaStatus::Exhausted || observation.confirmed_subscription,
            "unconfirmed subscription exhaustion"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO quota_pools(provider,account_key) VALUES(?1,?2) ON CONFLICT DO NOTHING",
            params![observation.provider, observation.account_key],
        )?;
        let qualified_probe = if let Some(authority) = probe_authority {
            let unit = validate_authority(&tx, authority, true, false)?;
            ensure!(
                unit.provider == observation.provider,
                "probe provider mismatch"
            );
            let own:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM quota_pools p JOIN quota_leases l ON l.unit_id=p.probe_unit WHERE p.provider=?1 AND p.account_key=?2 AND p.probe_unit=?3 AND l.active=1 AND l.provider=p.provider AND l.account_key=p.account_key AND l.epoch=?4)",
                params![observation.provider,observation.account_key,unit.id.to_string(),unit.owner_epoch],|r|r.get(0))?;
            ensure!(own, "quota recovery requires the admitted pool probe");
            true
        } else {
            false
        };
        let prior = windows(&tx, &observation.provider, &observation.account_key)?
            .into_iter()
            .find(|o| o.bucket == observation.bucket);
        if !super::quota_observation::applies(prior.as_ref(), observation, qualified_probe) {
            tx.commit()?;
            return Ok(());
        }
        tx.execute("INSERT INTO quota_windows(provider,account_key,bucket,observed_at,body) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(provider,account_key,bucket) DO UPDATE SET observed_at=excluded.observed_at,body=excluded.body",
            params![observation.provider,observation.account_key,observation.bucket,observation.observed_at,serde_json::to_string(observation)?])?;
        if observation.status == QuotaStatus::Exhausted {
            let next = observation
                .resets_at
                .unwrap_or_else(|| observation.observed_at.saturating_add(60_000));
            tx.execute("UPDATE quota_pools SET next_probe_at=MAX(next_probe_at,?1) WHERE provider=?2 AND account_key=?3",params![next,observation.provider,observation.account_key])?;
        } else if observation.status == QuotaStatus::Available {
            // Bucket recovery never retires a live pool probe. A refresh may contain
            // several windows, all requiring the same correlated lease. Only release
            // or terminal retirement relinquishes that lease.
            if !windows(&tx, &observation.provider, &observation.account_key)?
                .iter()
                .any(|o| o.status == QuotaStatus::Exhausted)
            {
                tx.execute(
                    "UPDATE quota_pools SET backoff=60000 WHERE provider=?1 AND account_key=?2",
                    params![observation.provider, observation.account_key],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }
    /// Unknown balances have explicit finite concurrency; they never manufacture token capacity.
    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn reserve_execution_quota(
        &mut self,
        authority: &ExecutionAuthority,
        provider: &str,
        account: &str,
        global_max: usize,
        executor_max: usize,
        provider_max: usize,
        at: i64,
    ) -> Result<QuotaAdmission> {
        self.reserve_execution_quota_with_project_limit(
            authority,
            provider,
            account,
            global_max,
            executor_max,
            provider_max,
            usize::MAX,
            at,
        )
    }
    // Keep pool identity, exact authority, clock and each independent cap explicit.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn reserve_execution_quota_with_project_limit(
        &mut self,
        authority: &ExecutionAuthority,
        provider: &str,
        account: &str,
        global_max: usize,
        executor_max: usize,
        provider_max: usize,
        project_max: usize,
        at: i64,
    ) -> Result<QuotaAdmission> {
        ensure!(
            project_max > 0
                && global_max > 0
                && executor_max > 0
                && provider_max > 0
                && global_max <= 1024
                && provider_max <= 1024,
            "invalid quota concurrency"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = validate_authority(&tx, authority, true, false)?;
        ensure!(
            unit.phase != WORKFLOW_SOURCE_BOOTSTRAP,
            "source preparation cannot consume native capacity"
        );
        ensure!(unit.provider == provider, "foreign quota provider");
        tx.execute(
            "INSERT INTO quota_pools(provider,account_key) VALUES(?1,?2) ON CONFLICT DO NOTHING",
            params![provider, account],
        )?;
        let already: Option<(String, String)> = tx
            .query_row(
                "SELECT provider,account_key FROM quota_leases WHERE unit_id=?1 AND active=1",
                [unit.id.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        if let Some((p, a)) = already {
            ensure!(
                p == provider && a == account,
                "quota lease account mismatch"
            );
            tx.commit()?;
            return Ok(QuotaAdmission::Admitted);
        }
        let (next, probe): (i64, Option<String>) = tx.query_row(
            "SELECT next_probe_at,probe_unit FROM quota_pools WHERE provider=?1 AND account_key=?2",
            params![provider, account],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let observations = windows(&tx, provider, account)?;
        let exhausted = observations
            .iter()
            .any(|o| o.status == QuotaStatus::Exhausted);
        let global: usize = tx.query_row(
            "SELECT COUNT(*) FROM quota_leases WHERE active=1",
            [],
            |r| r.get(0),
        )?;
        let (provider_live,executor_live):(usize,usize)=tx.query_row("SELECT COUNT(*),COALESCE(SUM(role='executor'),0) FROM quota_leases WHERE provider=?1 AND account_key=?2 AND active=1",params![provider,account],|r|Ok((r.get(0)?,r.get(1)?)))?;
        let global_executor: usize = tx.query_row(
            "SELECT COUNT(*) FROM quota_leases WHERE active=1 AND role='executor'",
            [],
            |r| r.get(0),
        )?;
        // A recent high-utilization observation lowers concurrency; it is not a token promise.
        let project_blocked = project_capacity_blocked(&tx, &unit, project_max)?;
        let capacity = super::quota_policy::CapacitySnapshot {
            global_live: global,
            provider_live,
            executor_live,
            global_executor_live: global_executor,
            global_max,
            provider_max,
            executor_max,
            high_utilization: super::quota_policy::high_utilization(&observations, at),
            own_executor: unit.kind == UnitKind::Executor,
            project_blocked,
        };
        // A terminal unclassified native capacity error closes that attempt.
        // Its fresh successor waits for a bounded local recheck; no subscription
        // observation or provider-wide exhaustion is fabricated. Read indexed
        // identities through unit_tx so a JSON body cannot redirect authority.
        let mut q = tx.prepare("SELECT id FROM execution_units WHERE task_id=?1 AND id!=?2")?;
        let history = q
            .query_map(
                params![unit.scope.task_id.unwrap().to_string(), unit.id.to_string()],
                |r| r.get::<_, String>(0),
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(q);
        let mut capacity_due = at;
        for id in history {
            let previous = unit_tx(&tx, id.parse()?)?;
            ensure!(
                previous.scope == unit.scope,
                "capacity history scope mismatch"
            );
            if previous.provider == provider
                && previous.disposition == Disposition::CapacityInterrupted
            {
                ensure!(
                    !previous.native_effects_open
                        && !previous.result_finalization_open
                        && previous.work == Some(WorkOutcome::Unknown),
                    "invalid capacity terminal"
                );
                let due = previous
                    .capacity_retry_at
                    .context("capacity terminal recheck missing")?;
                ensure!(due >= previous.created_at, "invalid capacity recheck time");
                capacity_due = capacity_due.max(due);
            }
        }
        let resume = unit.state;
        tx.execute("INSERT INTO quota_waiters(unit_id,provider,account_key,reason,next_due,fairness_sequence,resume_state) VALUES(?1,?2,?3,'capacity',?4,?4,?5) ON CONFLICT(unit_id) DO NOTHING",
            params![unit.id.to_string(),provider,account,at,key(resume)])?;
        let last: String = tx.query_row(
            "SELECT last_role FROM quota_pools WHERE provider=?1 AND account_key=?2",
            params![provider, account],
            |r| r.get(0),
        )?;
        let mut q=tx.prepare("SELECT q.unit_id FROM quota_waiters q JOIN execution_units u ON u.id=q.unit_id WHERE q.provider=?1 AND q.account_key=?2 AND q.next_due<=?3 AND u.native_effects_open=1 ORDER BY CASE WHEN (u.kind='executor')=(?4='executor') THEN 1 ELSE 0 END,q.fairness_sequence,q.unit_id")?;
        let candidates = q
            .query_map(params![provider, account, at, last], |r| {
                r.get::<_, String>(0)
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(q);
        let mut first = None;
        for id in candidates {
            let candidate = unit_tx(&tx, id.parse()?)?;
            if project_capacity_blocked(&tx, &candidate, project_max)? {
                continue;
            }
            if candidate.kind == UnitKind::Executor && capacity.executor_blocked() {
                continue;
            }
            let eligible = match super::quota_policy::candidate_class(&tx, candidate.id, at)? {
                super::quota_policy::CandidateClass::Legacy => {
                    validate_authority(&tx, &candidate.authority(), true, false).is_ok()
                }
                super::quota_policy::CandidateClass::MarkedParked => true,
                super::quota_policy::CandidateClass::MarkedStalled => false,
            };
            if eligible {
                first = Some(candidate.id);
                break;
            }
        }
        let decision = super::quota_policy::decide(
            &super::quota_policy::QuotaSnapshot {
                exhausted,
                next_probe_at: next,
                foreign_probe: probe.as_ref().is_some_and(|id| id != &unit.id.to_string()),
                capacity_due,
                capacity,
                fair_head_is_self: first == Some(unit.id),
            },
            at,
        );
        let wait = match decision {
            super::quota_policy::Decision::Wait { reason, due } => Some((reason, due)),
            super::quota_policy::Decision::Admit { .. } => None,
        };
        if let Some((reason, due)) = wait {
            unit.wait_reason = Some(reason);
            if reason == WaitReason::Quota {
                unit.state = UnitState::WaitingQuota;
            }
            write_unit(&tx, &mut unit)?;
            tx.execute("INSERT INTO quota_waiters(unit_id,provider,account_key,reason,next_due,fairness_sequence,resume_state) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(unit_id) DO UPDATE SET reason=excluded.reason,next_due=excluded.next_due",
                params![unit.id.to_string(),provider,account,key(reason),due,at,key(resume)])?;
            tx.commit()?;
            return Ok(QuotaAdmission::Waiting {
                reason,
                next_due: due,
            });
        }
        // One native recovery probe, not one probe per Task. Lease survives until terminal.
        if exhausted {
            tx.execute("UPDATE quota_pools SET probe_unit=?1,next_probe_at=?2+backoff,backoff=MIN(backoff*2,1800000) WHERE provider=?3 AND account_key=?4",
                params![unit.id.to_string(),at,provider,account])?;
        }
        tx.execute("INSERT INTO quota_leases(unit_id,provider,account_key,role,epoch,active) VALUES(?1,?2,?3,?4,?5,1) ON CONFLICT(unit_id) DO UPDATE SET provider=excluded.provider,account_key=excluded.account_key,role=excluded.role,epoch=excluded.epoch,active=1",
            params![unit.id.to_string(),provider,account,key(unit.kind),unit.owner_epoch])?;
        tx.execute(
            "UPDATE quota_pools SET last_role=?1 WHERE provider=?2 AND account_key=?3",
            params![
                if unit.kind == UnitKind::Executor {
                    "executor"
                } else {
                    "reviewer"
                },
                provider,
                account
            ],
        )?;
        let resume: Option<String> = tx
            .query_row(
                "SELECT resume_state FROM quota_waiters WHERE unit_id=?1",
                [unit.id.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        tx.execute(
            "DELETE FROM quota_waiters WHERE unit_id=?1",
            [unit.id.to_string()],
        )?;
        unit.wait_reason = None;
        if unit.state == UnitState::WaitingQuota {
            unit.state =
                serde_json::from_value(json!(resume.context("quota resume state missing")?))?;
        }
        write_unit(&tx, &mut unit)?;
        append_event(
            &tx,
            &unit.scope,
            "execution.quota_admitted",
            json!({"unit":unit.id,"provider":provider,"balance_known":!observations.is_empty(),"probe":exhausted}),
        )?;
        tx.commit()?;
        Ok(QuotaAdmission::Admitted)
    }
    pub(crate) fn release_execution_quota(&mut self, id: UnitId) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        release_quota_tx(&tx, id)?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn execution_is_quota_probe(
        &self,
        unit: UnitId,
        provider: &str,
        account: &str,
    ) -> Result<bool> {
        Ok(self.connection.query_row("SELECT EXISTS(SELECT 1 FROM quota_pools p JOIN quota_leases l ON l.unit_id=p.probe_unit WHERE p.provider=?1 AND p.account_key=?2 AND p.probe_unit=?3 AND l.active=1 AND l.provider=p.provider AND l.account_key=p.account_key)",params![provider,account,unit.to_string()],|r|r.get(0))?)
    }
    pub(crate) fn mark_execution_quota_retry(
        &mut self,
        authority: &ExecutionAuthority,
    ) -> Result<ExecutionUnit> {
        self.mark_execution_quota_wait_inner(authority, true)
    }
    /// Plan-window telemetry retains the live turn without proving a recovery retry.
    pub(crate) fn mark_execution_quota_wait(
        &mut self,
        authority: &ExecutionAuthority,
    ) -> Result<ExecutionUnit> {
        self.mark_execution_quota_wait_inner(authority, false)
    }
    fn mark_execution_quota_wait_inner(
        &mut self,
        authority: &ExecutionAuthority,
        native_retry: bool,
    ) -> Result<ExecutionUnit> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = validate_authority(&tx, authority, true, false)?;
        ensure!(
            matches!(unit.state, UnitState::Running | UnitState::WaitingQuota),
            "quota retry requires the same live native turn"
        );
        ensure!(
            !native_retry || unit.provider == "codex",
            "native recovery retry is not established for this provider"
        );
        let account: String = tx.query_row(
            "SELECT account_key FROM quota_leases WHERE unit_id=?1 AND active=1 AND provider=?2",
            params![unit.id.to_string(), unit.provider],
            |r| r.get(0),
        )?;
        unit.state = UnitState::WaitingQuota;
        unit.wait_reason = Some(WaitReason::Quota);
        write_unit(&tx, &mut unit)?;
        if native_retry {
            tx.execute("UPDATE quota_pools SET probe_unit=?1 WHERE provider=?2 AND account_key=?3 AND probe_unit IS NULL",params![unit.id.to_string(),unit.provider,account])?;
        }
        append_event(
            &tx,
            &unit.scope,
            if native_retry {
                "execution.native_quota_retry"
            } else {
                "execution.native_quota_wait"
            },
            json!({"unit":unit.id,"session":unit.session_id}),
        )?;
        tx.commit()?;
        Ok(unit)
    }
    /// Only accepted recovery of every participating plan window resumes this live turn.
    pub(crate) fn resume_execution_quota_wait(
        &mut self,
        authority: &ExecutionAuthority,
        buckets: &std::collections::BTreeSet<String>,
    ) -> Result<ExecutionUnit> {
        ensure!(
            !buckets.is_empty() && buckets.len() <= 5,
            "invalid live quota buckets"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = validate_authority(&tx, authority, true, false)?;
        ensure!(
            unit.provider == "claude",
            "plan-window recovery requires the owned Claude turn"
        );
        let account: String = tx.query_row(
            "SELECT account_key FROM quota_leases WHERE unit_id=?1 AND active=1 AND provider=?2 AND epoch=?3",
            params![unit.id.to_string(), unit.provider, unit.owner_epoch], |r| r.get(0))?;
        let observations = windows(&tx, &unit.provider, &account)?;
        let available = buckets.iter().all(|bucket| {
            observations
                .iter()
                .any(|o| &o.bucket == bucket && o.status == QuotaStatus::Available)
        });
        if unit.state == UnitState::WaitingQuota
            && unit.wait_reason == Some(WaitReason::Quota)
            && available
        {
            unit.state = UnitState::Running;
            unit.wait_reason = None;
            write_unit(&tx, &mut unit)?;
            append_event(
                &tx,
                &unit.scope,
                "execution.native_quota_recovered",
                json!({"unit":unit.id,"session":unit.session_id}),
            )?;
        }
        // Existing input, worktree, lease and any admitted probe are unchanged.
        tx.commit()?;
        Ok(unit)
    }
}
fn windows(
    connection: &Connection,
    provider: &str,
    account: &str,
) -> Result<Vec<QuotaObservation>> {
    let mut s=connection.prepare("SELECT bucket,observed_at,body FROM quota_windows WHERE provider=?1 AND account_key=?2 ORDER BY bucket")?;
    s.query_map(params![provider, account], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?
    .map(|r| {
        let (bucket, at, body) = r?;
        let o: QuotaObservation = decode(body)?;
        ensure!(
            o.provider == provider
                && o.account_key == account
                && o.bucket == bucket
                && o.observed_at == at,
            "quota window indexed/body mismatch"
        );
        Ok(o)
    })
    .collect()
}
pub(super) fn release_quota_tx(tx: &Transaction<'_>, id: UnitId) -> Result<()> {
    tx.execute(
        "UPDATE quota_leases SET active=0 WHERE unit_id=?1",
        [id.to_string()],
    )?;
    tx.execute("UPDATE quota_pools SET probe_unit=NULL,next_probe_at=MAX(next_probe_at,?1+backoff) WHERE probe_unit=?2",params![now_ms(),id.to_string()])?;
    Ok(())
}

pub(super) fn project_capacity_blocked(
    tx: &Transaction<'_>,
    unit: &ExecutionUnit,
    configured: usize,
) -> Result<bool> {
    let project: Project =
        read_tx(tx, "projects", &unit.scope.project_id.to_string())?.context("unknown project")?;
    let limit = configured.min(project.max_tasks);
    ensure!(limit > 0, "invalid project concurrency");
    let (tasks, own): (usize, bool) = tx.query_row(
        "SELECT COUNT(DISTINCT u.task_id),COALESCE(MAX(u.task_id=?2),0) FROM quota_leases q JOIN execution_units u ON u.id=q.unit_id WHERE q.active=1 AND u.project_id=?1",
        params![unit.scope.project_id.to_string(),unit.scope.task_id.unwrap().to_string()],
        |r| Ok((r.get(0)?, r.get(1)?)))?;
    Ok(!own && tasks >= limit)
}
