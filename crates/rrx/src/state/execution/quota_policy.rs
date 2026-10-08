//! Pure quota decisions shared by legacy and retained pre-Session preparation.
use super::*;

pub(super) struct QuotaSnapshot {
    pub exhausted: bool,
    pub next_probe_at: i64,
    pub foreign_probe: bool,
    pub capacity_due: i64,
    pub capacity: CapacitySnapshot,
    pub fair_head_is_self: bool,
}
pub(super) struct CapacitySnapshot {
    pub global_live: usize,
    pub provider_live: usize,
    pub executor_live: usize,
    pub global_executor_live: usize,
    pub global_max: usize,
    pub provider_max: usize,
    pub executor_max: usize,
    pub high_utilization: bool,
    pub own_executor: bool,
    pub project_blocked: bool,
}
impl CapacitySnapshot {
    pub(super) fn executor_blocked(&self) -> bool {
        let maximum = if self.high_utilization {
            self.executor_max.min(1)
        } else {
            self.executor_max
        };
        self.executor_live >= maximum
            || self.global_executor_live >= self.global_max.saturating_sub(2).max(1)
    }
    fn blocked(&self) -> bool {
        self.project_blocked
            || self.global_live >= self.global_max
            || self.provider_live >= self.provider_max
            || (self.own_executor && self.executor_blocked())
    }
}
pub(super) fn high_utilization(observations: &[QuotaObservation], at: i64) -> bool {
    observations.iter().any(|o| {
        o.status == QuotaStatus::Available
            && at.saturating_sub(o.observed_at) <= 300_000
            && o.used_percent.is_some_and(|p| p >= 95.0)
    })
}
pub(super) fn fair_position<'a>(
    role: &str,
    sequence: i64,
    id: &'a str,
    last_role: &str,
) -> (bool, i64, &'a str) {
    (
        (role == "executor") == (last_role == "executor"),
        sequence,
        id,
    )
}
pub(super) enum Decision {
    Admit { probe: bool },
    Wait { reason: WaitReason, due: i64 },
}
pub(super) fn decide(s: &QuotaSnapshot, at: i64) -> Decision {
    if s.exhausted && (at < s.next_probe_at || s.foreign_probe) {
        Decision::Wait {
            reason: WaitReason::Quota,
            due: s.next_probe_at.max(at.saturating_add(1000)),
        }
    } else if s.capacity_due > at {
        Decision::Wait {
            reason: WaitReason::Capacity,
            due: s.capacity_due,
        }
    } else if s.capacity.blocked() || !s.fair_head_is_self {
        Decision::Wait {
            reason: WaitReason::Capacity,
            due: at.saturating_add(1000),
        }
    } else {
        Decision::Admit { probe: s.exhausted }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CandidateClass {
    Legacy,
    MarkedParked,
    MarkedStalled,
}
pub(super) struct CandidateFacts {
    pub marked: bool,
    pub parked: bool,
    pub unit_preparing: bool,
    pub reason_equal: bool,
    pub resume_preparing: bool,
    pub pool_equal: bool,
    pub next_due: i64,
}
pub(super) fn classify(facts: CandidateFacts, at: i64) -> CandidateClass {
    if !facts.marked {
        CandidateClass::Legacy
    } else if facts.parked
        && facts.unit_preparing
        && facts.reason_equal
        && facts.resume_preparing
        && facts.pool_equal
        && facts.next_due >= at.saturating_sub(30_000)
    {
        CandidateClass::MarkedParked
    } else {
        CandidateClass::MarkedStalled
    }
}
pub(super) fn candidate_class(tx: &Connection, id: UnitId, at: i64) -> Result<CandidateClass> {
    // Bounded scalar observation, never a marked owner reconstructed from rows.
    let (marked, parked, preparing, reason, resume, pool, due): (bool,bool,bool,bool,bool,bool,i64) = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM managed_phase_operations o WHERE o.unit_id=u.id AND o.phase_open=1), EXISTS(SELECT 1 FROM managed_phase_operations o JOIN managed_phase_readiness r ON r.operation_id=o.operation_id WHERE o.unit_id=u.id AND o.phase_open=1 AND r.state='parked' AND r.parking_version=r.version), COALESCE(json_extract(u.body,'$.state')='preparing',0), COALESCE(json_extract(u.body,'$.wait_reason')=w.reason,0),w.resume_state='preparing',COALESCE(json_extract(u.body,'$.provider')=w.provider AND w.account_key='unknown',0),w.next_due FROM quota_waiters w JOIN execution_units u ON u.id=w.unit_id WHERE w.unit_id=?1",
        [id.to_string()], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?)))?;
    Ok(classify(
        CandidateFacts {
            marked,
            parked,
            unit_preparing: preparing,
            reason_equal: reason,
            resume_preparing: resume,
            pool_equal: pool,
            next_due: due,
        },
        at,
    ))
}

#[cfg(test)]
mod primitive_tests {
    use super::*;
    #[test]
    fn nongrant_native_capacity_policy_matches_legacy_caps_and_role_reservation() {
        for high in [false, true] {
            for own_executor in [false, true] {
                for project_blocked in [false, true] {
                    for global_live in [0, 5, 6, 7] {
                        for provider_live in [0, 2, 3, 4] {
                            for executor_live in [0, 1, 2, 3] {
                                for global_executor_live in [0, 3, 4, 5] {
                                    let snapshot = CapacitySnapshot {
                                        global_live,
                                        provider_live,
                                        executor_live,
                                        global_executor_live,
                                        global_max: 6,
                                        provider_max: 3,
                                        executor_max: 2,
                                        high_utilization: high,
                                        own_executor,
                                        project_blocked,
                                    };
                                    let executor_blocked = executor_live
                                        >= if high { 1 } else { 2 }
                                        || global_executor_live >= 4;
                                    let expected = project_blocked
                                        || global_live >= 6
                                        || provider_live >= 3
                                        || (own_executor && executor_blocked);
                                    assert_eq!(snapshot.executor_blocked(), executor_blocked);
                                    assert_eq!(snapshot.blocked(), expected);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn nongrant_native_quota_policy_preserves_legacy_decision_order() {
        for exhausted in [false, true] {
            for foreign_probe in [false, true] {
                for capacity_blocked in [false, true] {
                    for fair_head_is_self in [false, true] {
                        for next_probe_at in [500, 1000, 1500] {
                            for capacity_due in [500, 1000, 1500] {
                                let s = QuotaSnapshot {
                                    exhausted,
                                    next_probe_at,
                                    foreign_probe,
                                    capacity_due,
                                    capacity: CapacitySnapshot {
                                        global_live: usize::from(capacity_blocked),
                                        provider_live: 0,
                                        executor_live: 0,
                                        global_executor_live: 0,
                                        global_max: 1,
                                        provider_max: 1,
                                        executor_max: 1,
                                        high_utilization: false,
                                        own_executor: false,
                                        project_blocked: false,
                                    },
                                    fair_head_is_self,
                                };
                                let expected =
                                    if exhausted && (1000 < next_probe_at || foreign_probe) {
                                        Some((WaitReason::Quota, next_probe_at.max(2000)))
                                    } else if capacity_due > 1000 {
                                        Some((WaitReason::Capacity, capacity_due))
                                    } else if capacity_blocked || !fair_head_is_self {
                                        Some((WaitReason::Capacity, 2000))
                                    } else {
                                        None
                                    };
                                match (decide(&s, 1000), expected) {
                                    (Decision::Admit { probe }, None) => {
                                        assert_eq!(probe, exhausted)
                                    }
                                    (Decision::Wait { reason, due }, Some((r, d))) => {
                                        assert_eq!(reason, r);
                                        assert_eq!(due, d);
                                    }
                                    _ => panic!("shared quota decision changed"),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn nongrant_native_marked_fair_position_has_exact_stall_boundary() {
        let facts = |marked, values: [bool; 5], next_due| CandidateFacts {
            marked,
            parked: values[0],
            unit_preparing: values[1],
            reason_equal: values[2],
            resume_preparing: values[3],
            pool_equal: values[4],
            next_due,
        };
        assert_eq!(
            classify(facts(false, [false; 5], 0), 100_000),
            CandidateClass::Legacy
        );
        assert_eq!(
            classify(facts(true, [true; 5], 70_000), 100_000),
            CandidateClass::MarkedParked
        );
        assert_eq!(
            classify(facts(true, [true; 5], 69_999), 100_000),
            CandidateClass::MarkedStalled
        );
        for omitted in 0..5 {
            let mut values = [true; 5];
            values[omitted] = false;
            assert_eq!(
                classify(facts(true, values, 70_000), 100_000),
                CandidateClass::MarkedStalled
            );
        }
    }
}
