//! Epoch/job CAS for historical cleanup; never reopens native/result authority.
use super::*;

pub(crate) struct CleanupClaim {
    pub unit: ExecutionUnit,
    pub epoch: u64,
    pub version: u64,
}
impl Store {
    pub(crate) fn due_execution_cleanup(&self, at: i64, limit: usize) -> Result<Vec<UnitId>> {
        ensure!((1..=32).contains(&limit), "invalid cleanup batch bound");
        let mut query = self.connection.prepare("SELECT j.unit_id FROM cleanup_jobs j JOIN execution_units u ON u.id=j.unit_id WHERE j.next_due<=?1 AND u.native_effects_open=0 AND u.result_finalization_open=0 ORDER BY j.next_due,j.unit_id LIMIT ?2")?;
        query
            .query_map(params![at, limit as i64], |r| r.get::<_, String>(0))?
            .map(|r| Ok(r?.parse()?))
            .collect()
    }
    pub(crate) fn claim_execution_cleanup(
        &mut self,
        id: UnitId,
        epoch: u64,
        at: i64,
    ) -> Result<Option<CleanupClaim>> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current: u64 = tx.query_row(
            "SELECT epoch FROM runtime_epoch WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        ensure!(current == epoch, "cleanup owner epoch retired");
        let unit = unit_tx(&tx, id)?;
        // Finalization helpers carry the same cookie. A broad scan must never
        // stop capture/verification helpers or remove their required inputs.
        if unit.native_effects_open || unit.result_finalization_open {
            return Ok(None);
        }
        let job: Option<(u64, i64)> = tx
            .query_row(
                "SELECT version,next_due FROM cleanup_jobs WHERE unit_id=?1",
                [id.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((version, _)) = job.filter(|(_, due)| *due <= at) else {
            return Ok(None);
        };
        let next = version.checked_add(1).context("cleanup version overflow")?;
        ensure!(tx.execute("UPDATE cleanup_jobs SET version=?1,attempts=attempts+1,next_due=?2 WHERE unit_id=?3 AND version=?4 AND next_due<=?5",
            params![next,at.saturating_add(60_000),id.to_string(),version,at])?==1,"cleanup claim CAS mismatch");
        tx.commit()?;
        Ok(Some(CleanupClaim {
            unit,
            epoch,
            version: next,
        }))
    }
    pub(crate) fn finish_execution_cleanup(
        &mut self,
        claim: &CleanupClaim,
        observation: &CleanupObservation,
    ) -> Result<()> {
        ensure!(
            observation.unit_id == claim.unit.id
                && observation.coverage.len() <= 32
                && observation.remaining.len() <= 1024
                && observation.errors.len() <= 32,
            "invalid claimed cleanup observation"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        validate_cleanup_claim(&tx, claim)?;
        tx.execute(
            "INSERT INTO cleanup_observations(unit_id,at,body) VALUES(?1,?2,?3)",
            params![
                claim.unit.id.to_string(),
                observation.at,
                serde_json::to_string(observation)?
            ],
        )?;
        let mut query = tx.prepare("SELECT id FROM resource_leases WHERE unit_id=?1 AND state NOT IN ('released','quarantined')")?;
        let ids = query
            .query_map([claim.unit.id.to_string()], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(query);
        for id in ids {
            let mut lease = lease_tx(&tx, id.parse()?)?;
            ensure!(
                lease.unit_id == claim.unit.id,
                "cleanup lease scope changed"
            );
            lease.state = LeaseState::Quarantined;
            let expected = lease.version;
            bump(&mut lease.version)?;
            ensure!(tx.execute("UPDATE resource_leases SET state='quarantined',version=?1,body=?2 WHERE id=?3 AND unit_id=?4 AND version=?5",
                params![lease.version,serde_json::to_string(&lease)?,lease.id.to_string(),claim.unit.id.to_string(),expected])?==1,"cleanup lease CAS mismatch");
        }
        // Keep a historical backlog instead of blocking Task progress or
        // assuming that an empty cookie observation frees ports/worktrees.
        tx.execute(
            "UPDATE cleanup_jobs SET next_due=?1,version=version+1 WHERE unit_id=?2 AND version=?3",
            params![
                observation.at.saturating_add(60_000),
                claim.unit.id.to_string(),
                claim.version
            ],
        )?;
        append_event(
            &tx,
            &claim.unit.scope,
            "execution.cleanup_observed",
            json!({"unit":claim.unit.id,"work":claim.unit.work,"cleanup":observation.outcome,"coverage":observation.coverage}),
        )?;
        tx.commit()?;
        Ok(())
    }
}
fn validate_cleanup_claim(tx: &Transaction<'_>, claim: &CleanupClaim) -> Result<()> {
    let epoch: u64 = tx.query_row(
        "SELECT epoch FROM runtime_epoch WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    let version: u64 = tx.query_row(
        "SELECT version FROM cleanup_jobs WHERE unit_id=?1",
        [claim.unit.id.to_string()],
        |r| r.get(0),
    )?;
    let unit = unit_tx(tx, claim.unit.id)?;
    ensure!(
        epoch == claim.epoch
            && version == claim.version
            && unit.scope == claim.unit.scope
            && unit.cookie == claim.unit.cookie
            && unit.generation == claim.unit.generation
            && unit.owner_epoch == claim.unit.owner_epoch
            && unit.version == claim.unit.version
            && !unit.native_effects_open
            && !unit.result_finalization_open,
        "cleanup claim authority changed"
    );
    Ok(())
}
