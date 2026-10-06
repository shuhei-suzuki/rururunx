//! Pure quota decisions shared by legacy and retained pre-Session preparation.
use super::*;

pub(super) struct QuotaSnapshot {
    pub exhausted: bool,
    pub next_probe_at: i64,
    pub foreign_probe: bool,
    pub capacity_due: i64,
    pub capacity_blocked: bool,
    pub fair_head_is_self: bool,
}
pub(super) enum Decision { Admit { probe: bool }, Wait { reason: WaitReason, due: i64 } }
pub(super) fn decide(s: &QuotaSnapshot, at: i64) -> Decision {
    if s.exhausted && (at < s.next_probe_at || s.foreign_probe) {
        Decision::Wait { reason: WaitReason::Quota, due: s.next_probe_at.max(at.saturating_add(1000)) }
    } else if s.capacity_due > at {
        Decision::Wait { reason: WaitReason::Capacity, due: s.capacity_due }
    } else if s.capacity_blocked || !s.fair_head_is_self {
        Decision::Wait { reason: WaitReason::Capacity, due: at.saturating_add(1000) }
    } else { Decision::Admit { probe: s.exhausted } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CandidateClass { Legacy, MarkedParked, MarkedStalled }
pub(super) fn classify(marked: bool, parked: bool, unit_preparing: bool, reason_equal: bool, resume_preparing: bool, pool_equal: bool, next_due: i64, at: i64) -> CandidateClass {
    if !marked { CandidateClass::Legacy }
    else if parked && unit_preparing && reason_equal && resume_preparing && pool_equal && next_due >= at.saturating_sub(30_000) { CandidateClass::MarkedParked }
    else { CandidateClass::MarkedStalled }
}
pub(super) fn candidate_class(tx: &Connection, id: UnitId, at: i64) -> Result<CandidateClass> {
    // Bounded scalar observation, never a marked owner reconstructed from rows.
    let (marked, parked, preparing, reason, resume, pool, due): (bool,bool,bool,bool,bool,bool,i64) = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM managed_phase_operations o WHERE o.unit_id=u.id AND o.phase_open=1), EXISTS(SELECT 1 FROM managed_phase_operations o JOIN managed_phase_readiness r ON r.operation_id=o.operation_id WHERE o.unit_id=u.id AND o.phase_open=1 AND r.state='parked' AND r.parking_version=r.version), json_extract(u.body,'$.state')='preparing', json_extract(u.body,'$.wait_reason')=w.reason,w.resume_state='preparing',json_extract(u.body,'$.provider')=w.provider AND w.account_key='unknown',w.next_due FROM quota_waiters w JOIN execution_units u ON u.id=w.unit_id WHERE w.unit_id=?1",
        [id.to_string()], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?)))?;
    Ok(classify(marked,parked,preparing,reason,resume,pool,due,at))
}
