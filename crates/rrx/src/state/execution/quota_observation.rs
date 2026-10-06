//! Shared factual window ordering and exhaustion policy. This pure decision
//! creates no lease, actor, probe or authority.
use crate::execution::{QuotaObservation, QuotaStatus};
pub(super) fn applies(
    prior: Option<&QuotaObservation>,
    o: &QuotaObservation,
    qualified_probe: bool,
) -> bool {
    let Some(old) = prior else { return true };
    if o.observed_at < old.observed_at {
        return false;
    }
    if old.status == QuotaStatus::Exhausted && o.status != QuotaStatus::Exhausted {
        let fresh_window = o.status == QuotaStatus::Available
            && old.resets_at.is_some_and(|r| o.observed_at >= r)
            && o.window_id != old.window_id;
        let fresh_probe = qualified_probe
            && o.status == QuotaStatus::Available
            && o.observed_at > old.observed_at;
        return fresh_window || fresh_probe;
    }
    true
}
#[cfg(test)]
mod registration_quota_primitive_tests {
    use super::*;
    fn old() -> QuotaObservation {
        QuotaObservation {
            provider: "codex".into(),
            account_key: "unknown".into(),
            bucket: "primary".into(),
            window_id: "one".into(),
            status: QuotaStatus::Exhausted,
            used_percent: Some(100.0),
            resets_at: None,
            observed_at: 10,
            source_version: "closed-profile".into(),
            confirmed_subscription: true,
        }
    }
    #[test]
    fn exhaustion_recovery_keeps_window_and_own_probe_conditions() {
        let mut old = old();
        let mut observation = old.clone();
        observation.status = QuotaStatus::Available;
        for probe in [false, true] {
            observation.observed_at = 9;
            assert!(!applies(Some(&old), &observation, probe));
            observation.observed_at = 10;
            assert!(!applies(Some(&old), &observation, probe));
            observation.observed_at = 11;
            assert_eq!(applies(Some(&old), &observation, probe), probe);
        }
        old.resets_at = Some(20);
        observation.observed_at = 20;
        assert!(!applies(Some(&old), &observation, false));
        observation.window_id = "two".into();
        assert!(applies(Some(&old), &observation, false));
        observation.status = QuotaStatus::Unknown;
        assert!(!applies(Some(&old), &observation, true));
        observation.status = QuotaStatus::Exhausted;
        assert!(applies(Some(&old), &observation, false));
        assert!(applies(None, &observation, false));
    }
}
