//! Native quota decoding; ordinary HTTP 429 is never subscription proof.
use super::*;
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::sync::Arc;

pub struct QuotaScheduler {
    owner: Arc<RuntimeOwner>,
    pub global_total: usize,
    pub provider_executor: usize,
    pub provider_total: usize,
}
impl QuotaScheduler {
    pub fn new(owner: Arc<RuntimeOwner>) -> Self {
        Self {
            owner,
            global_total: 6,
            provider_executor: 2,
            provider_total: 3,
        }
    }
    pub fn observe(&self, observation: &QuotaObservation) -> Result<()> {
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .observe_quota(observation)
    }
    pub(crate) fn observe_probe(
        &self,
        authority: &ExecutionAuthority,
        observation: &QuotaObservation,
    ) -> Result<()> {
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .observe_quota_from_probe(observation, authority)
    }
    pub fn admission(
        &self,
        authority: &ExecutionAuthority,
        at: i64,
    ) -> Result<Option<(WaitReason, i64)>> {
        let provider = {
            self.owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .validate_execution(authority, true, false)?
                .provider
        };
        let outcome = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .reserve_execution_quota(
                authority,
                &provider,
                "unknown",
                self.global_total,
                self.provider_executor,
                self.provider_total,
                at,
            )?;
        match outcome {
            crate::state::QuotaAdmission::Admitted => Ok(None),
            crate::state::QuotaAdmission::Waiting { reason, next_due } => {
                Ok(Some((reason, next_due)))
            }
        }
    }
    pub fn release(&self, unit: UnitId) -> Result<()> {
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .release_execution_quota(unit)
    }
}

/// Shapes pinned against `codex-cli 0.160.0 app-server generate-json-schema --experimental`.
pub fn codex_windows(value: &Value, at: i64) -> Result<Vec<QuotaObservation>> {
    let mut snapshots = Vec::new();
    if let Some(all) = value.get("rateLimitsByLimitId").filter(|v| !v.is_null()) {
        let map = all.as_object().context("invalid native quota bucket map")?;
        ensure!(map.len() <= 32, "too many quota buckets");
        for (key, snapshot) in map {
            snapshots.push((key.as_str(), snapshot));
        }
    } else {
        let snapshot = value
            .get("rateLimits")
            .context("missing native quota snapshot")?;
        snapshots.push((snapshot["limitId"].as_str().unwrap_or("all"), snapshot));
    }
    let mut observations = Vec::new();
    for (bucket, snapshot) in snapshots {
        ensure!(
            !bucket.is_empty() && bucket.len() <= 128 && snapshot.is_object(),
            "invalid quota snapshot"
        );
        for window in ["primary", "secondary"] {
            let Some(w) = snapshot.get(window).filter(|v| !v.is_null()) else {
                continue;
            };
            let percent = w["usedPercent"]
                .as_i64()
                .context("native quota percentage missing")?;
            ensure!(
                i32::try_from(percent).is_ok(),
                "native quota percentage is outside its int32 wire contract"
            );
            let reset = match w.get("resetsAt") {
                None | Some(Value::Null) => None,
                Some(v) => Some(
                    v.as_i64()
                        .context("invalid native quota reset")?
                        .checked_mul(1000)
                        .context("quota reset overflow")?,
                ),
            };
            let reached = snapshot["rateLimitReachedType"] == "rate_limit_reached";
            observations.push(QuotaObservation {
                provider: "codex".into(),
                account_key: "unknown".into(),
                bucket: format!("{bucket}/{window}"),
                window_id: reset
                    .map_or_else(|| format!("unknown-{bucket}-{window}"), |r| r.to_string()),
                status: if reached || percent >= 100 {
                    QuotaStatus::Exhausted
                } else if percent < 0 {
                    QuotaStatus::Unknown
                } else {
                    QuotaStatus::Available
                },
                // The wire schema has no 0..100 constraint. Keep an out-of-range
                // fraction unknown rather than fabricating or clamping capacity.
                used_percent: (0..=100).contains(&percent).then_some(percent as f64),
                resets_at: reset,
                observed_at: at,
                source_version: "codex-cli 0.160.0/account.rateLimits".into(),
                confirmed_subscription: reached || percent >= 100,
            });
        }
    }
    Ok(observations)
}
pub fn codex_subscription_error(error: &Value) -> bool {
    error["codexErrorInfo"] == "usageLimitExceeded"
}

pub(super) fn codex_capacity_error(error: &Value) -> bool {
    // These are finite members of the installed 0.160.0 schema. They do not
    // establish a subscription balance and must not poison a sibling pool.
    let info = &error["codexErrorInfo"];
    matches!(
        info.as_str(),
        Some("rateLimitExceeded" | "flexUnavailable" | "serverOverloaded")
    ) || [
        "httpConnectionFailed",
        "responseStreamConnectionFailed",
        "responseStreamDisconnected",
        "responseTooManyFailedAttempts",
    ]
    .iter()
    .any(|key| matches!(info[key]["httpStatusCode"].as_u64(), Some(429 | 503)))
}

/// Recognize a bounded plan window only. Missing/type-unknown fields stay unclassified.
pub fn claude_window(value: &Value, at: i64) -> Result<Option<QuotaObservation>> {
    if value["type"] != "rate_limit_event" {
        return Ok(None);
    }
    let info = &value["rate_limit_info"];
    let Some(bucket) = info["rateLimitType"].as_str() else {
        return Ok(None);
    };
    if !matches!(
        bucket,
        "five_hour"
            | "seven_day"
            | "seven_day_opus"
            | "seven_day_sonnet"
            | "seven_day_overage_included"
    ) {
        return Ok(None);
    }
    let status = match info["status"].as_str() {
        Some("rejected") => QuotaStatus::Exhausted,
        Some("allowed" | "allowed_warning") => QuotaStatus::Available,
        _ => return Ok(None),
    };
    let reset = match info.get("resetsAt") {
        None | Some(Value::Null) => None,
        Some(v) => Some(
            v.as_i64()
                .context("invalid Claude quota reset")?
                .checked_mul(1000)
                .context("quota reset overflow")?,
        ),
    };
    let used = match info.get("utilization") {
        None | Some(Value::Null) => None,
        Some(v) => {
            let n = v.as_f64().context("invalid Claude quota utilization")?;
            ensure!(
                n.is_finite() && (0.0..=1.0).contains(&n),
                "invalid Claude quota utilization range"
            );
            Some(n * 100.0)
        }
    };
    Ok(Some(QuotaObservation {
        provider: "claude".into(),
        account_key: "unknown".into(),
        bucket: bucket.into(),
        window_id: reset.map_or_else(|| format!("unknown-{bucket}"), |r| r.to_string()),
        status,
        used_percent: used,
        resets_at: reset,
        observed_at: at,
        source_version: "Claude Code 2.1.283/rate_limit_event (fixture conformance)".into(),
        confirmed_subscription: status == QuotaStatus::Exhausted,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn retry_429_and_text_do_not_establish_subscription_exhaustion() {
        assert!(!codex_subscription_error(
            &json!({"message":"usageLimitExceeded","codexErrorInfo":{"httpConnectionFailed":{"httpStatusCode":429}}})
        ));
        assert!(
            claude_window(
                &json!({"type":"system","subtype":"api_retry","status_code":429}),
                1
            )
            .unwrap()
            .is_none()
        );
        assert!(claude_window(&json!({"type":"rate_limit_event","rate_limit_info":{"status":"rejected","rateLimitType":"new_unknown"}}),1).unwrap().is_none());
        assert!(codex_subscription_error(
            &json!({"codexErrorInfo":"usageLimitExceeded"})
        ));
    }
    #[test]
    fn native_optional_windows_and_buckets_keep_unknown_fields_unknown() {
        let values=codex_windows(&json!({"rateLimitsByLimitId":{"codex":{"primary":{"usedPercent":100,"resetsAt":123},"secondary":null},"review":{"primary":{"usedPercent":20},"secondary":null}}}),1).unwrap();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].status, QuotaStatus::Exhausted);
        assert_eq!(values[0].resets_at, Some(123000));
        assert_eq!(values[1].resets_at, None);
        let excess =
            codex_windows(&json!({"rateLimits":{"primary":{"usedPercent":101}}}), 1).unwrap();
        assert_eq!(excess[0].status, QuotaStatus::Exhausted);
        assert_eq!(excess[0].used_percent, None);
        assert!(excess[0].confirmed_subscription);
        let negative =
            codex_windows(&json!({"rateLimits":{"primary":{"usedPercent":-1}}}), 1).unwrap();
        assert_eq!(negative[0].status, QuotaStatus::Unknown);
        assert_eq!(negative[0].used_percent, None);
        assert!(!negative[0].confirmed_subscription);
        let c=claude_window(&json!({"type":"rate_limit_event","rate_limit_info":{"status":"rejected","rateLimitType":"five_hour","resetsAt":123}}),1).unwrap().unwrap();
        assert_eq!(c.used_percent, None);
        assert_eq!(c.status, QuotaStatus::Exhausted);
    }
}
