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
    pub project_tasks: usize,
}
impl QuotaScheduler {
    pub fn new(owner: Arc<RuntimeOwner>) -> Self {
        Self {
            owner,
            global_total: 6,
            provider_executor: 2,
            provider_total: 3,
            project_tasks: 4,
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
            .reserve_execution_quota_with_project_limit(
                authority,
                &provider,
                "unknown",
                self.global_total,
                self.provider_executor,
                self.provider_total,
                self.project_tasks,
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

/// Typed R-1 rate evidence, shared by selfcheck and the existing quota consumer.
/// Metadata-only/empty snapshots never invent a usable window.
#[derive(Debug)]
pub(crate) struct CodexRate {
    snapshots: Vec<(String, RateSnapshot)>,
    pub(crate) ordinary: RatePresence<bool>,
}
#[derive(Debug, PartialEq)]
pub(crate) enum RatePresence<T> {
    Absent,
    Null,
    Value(T),
}
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RateSnapshot {
    limit_id: Option<String>,
    primary: Option<RateWindow>,
    secondary: Option<RateWindow>,
    rate_limit_reached_type: Option<ReachedKind>,
    spend_control_reached: Option<bool>,
}
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReachedKind {
    RateLimitReached,
    WorkspaceOwnerCreditsDepleted,
    WorkspaceMemberCreditsDepleted,
    WorkspaceOwnerUsageLimitReached,
    WorkspaceMemberUsageLimitReached,
}
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RateWindow {
    used_percent: i32,
    resets_at: Option<i64>,
    window_duration_mins: Option<i64>,
}
impl CodexRate {
    /// `account/rateLimits/updated` declares only `rateLimits`; response-only
    /// keys on the notification are retained by the frame and never read here.
    /// The declared snapshot alone reaches the shared snapshot rules.
    pub(crate) fn decode_notification(value: &Value) -> Result<Self> {
        super::codex_wire::rate_notification(value)?;
        Self::decode(&serde_json::json!({"rateLimits": value["rateLimits"]}))
    }
    pub(crate) fn decode(value: &Value) -> Result<Self> {
        super::codex_wire::CodexWireContract::default()
            .response("account/rateLimits/read", value)?;
        let envelope = value.as_object().context("invalid rate envelope")?;
        let decode = |v: &Value| -> Result<RateSnapshot> {
            // Derived visitors also accept sequences and single-key enum maps;
            // native evidence is a snapshot object with object windows only.
            let object = v.as_object().context("invalid rate snapshot")?;
            for key in ["primary", "secondary"] {
                ensure!(
                    matches!(object.get(key), None | Some(Value::Null | Value::Object(_))),
                    "invalid rate window"
                );
            }
            ensure!(
                matches!(
                    object.get("rateLimitReachedType"),
                    None | Some(Value::Null | Value::String(_))
                ),
                "invalid rate reached type"
            );
            let snapshot: RateSnapshot = serde_json::from_value(v.clone())?;
            ensure!(
                snapshot
                    .limit_id
                    .as_ref()
                    .is_none_or(|id| !id.is_empty() && id.len() <= 128),
                "invalid rate limit ID"
            );
            for w in [&snapshot.primary, &snapshot.secondary]
                .into_iter()
                .flatten()
            {
                if let Some(reset) = w.resets_at {
                    reset.checked_mul(1000).context("quota reset overflow")?;
                }
                // Parsed as int64 even though duration is not used for admission.
                let _ = w.window_duration_mins;
            }
            Ok(snapshot)
        };
        let fallback = decode(
            envelope
                .get("rateLimits")
                .context("missing native quota snapshot")?,
        )?;
        let ordinary = match envelope.get("ordinaryUsageAllowed") {
            None => RatePresence::Absent,
            Some(Value::Null) => RatePresence::Null,
            Some(v) => RatePresence::Value(v.as_bool().context("invalid ordinary usage flag")?),
        };
        let snapshots = match envelope.get("rateLimitsByLimitId") {
            None | Some(Value::Null) => vec![(
                fallback.limit_id.clone().unwrap_or_else(|| "all".into()),
                fallback,
            )],
            Some(v) => {
                let map = v.as_object().context("invalid native quota bucket map")?;
                ensure!(map.len() <= 32, "too many quota buckets");
                map.iter()
                    .map(|(key, v)| {
                        ensure!(
                            !key.is_empty() && key.len() <= 128,
                            "invalid quota bucket key"
                        );
                        Ok((key.clone(), decode(v)?))
                    })
                    .collect::<Result<Vec<_>>>()?
            }
        };
        Ok(Self {
            snapshots,
            ordinary,
        })
    }
    pub(crate) fn observations(&self, at: i64) -> Vec<QuotaObservation> {
        // Presence is metadata only, never an availability/admission override.
        let _ = &self.ordinary;
        let mut observations = Vec::new();
        for (bucket, snapshot) in &self.snapshots {
            for (window, w) in [
                ("primary", &snapshot.primary),
                ("secondary", &snapshot.secondary),
            ] {
                let Some(w) = w else { continue };
                let percent = w.used_percent;
                let reset = w.resets_at.map(|r| r * 1000);
                let exhausted = percent >= 100
                    || matches!(
                        snapshot.rate_limit_reached_type,
                        Some(ReachedKind::RateLimitReached)
                    );
                let credit = snapshot.spend_control_reached == Some(true)
                    || matches!(
                        snapshot.rate_limit_reached_type,
                        Some(
                            ReachedKind::WorkspaceOwnerCreditsDepleted
                                | ReachedKind::WorkspaceMemberCreditsDepleted
                                | ReachedKind::WorkspaceOwnerUsageLimitReached
                                | ReachedKind::WorkspaceMemberUsageLimitReached
                        )
                    );
                observations.push(QuotaObservation {
                    provider: "codex".into(),
                    account_key: "unknown".into(),
                    bucket: format!("{bucket}/{window}"),
                    window_id: reset
                        .map_or_else(|| format!("unknown-{bucket}-{window}"), |r| r.to_string()),
                    status: if exhausted {
                        QuotaStatus::Exhausted
                    } else if credit || percent < 0 {
                        QuotaStatus::Unknown
                    } else {
                        QuotaStatus::Available
                    },
                    used_percent: (0..=100).contains(&percent).then_some(f64::from(percent)),
                    resets_at: reset,
                    observed_at: at,
                    source_version: "codex/account.rateLimits".into(),
                    confirmed_subscription: exhausted,
                });
            }
        }
        observations
    }
}
pub fn codex_windows(value: &Value, at: i64) -> Result<Vec<QuotaObservation>> {
    Ok(CodexRate::decode(value)
        .context(NativeFailure::ProtocolFailure)?
        .observations(at))
}
pub fn codex_notification_windows(value: &Value, at: i64) -> Result<Vec<QuotaObservation>> {
    CodexRate::decode_notification(value)
        .map(|rate| rate.observations(at))
        .context(NativeFailure::ProtocolFailure)
}
pub fn codex_subscription_error(error: &Value) -> bool {
    super::codex_wire::error_kind(&error["codexErrorInfo"])
        .is_ok_and(|k| k == super::codex_wire::CodexErrorKind::Subscription)
}

pub(super) fn codex_capacity_error(error: &Value) -> bool {
    super::codex_wire::error_kind(&error["codexErrorInfo"])
        .is_ok_and(|k| k == super::codex_wire::CodexErrorKind::Capacity)
}

/// Compatibility projection of the shared strict Claude decoder. Malformed or
/// unknown rate fields are errors, distinct from a legal non-rate frame.
pub fn claude_window(value: &Value, at: i64) -> Result<Option<QuotaObservation>> {
    let checked = super::claude_wire::decoder_selfcheck()?;
    let frame = super::claude_wire::ValidatedClaudeFrame::decode(value.clone(), &checked)?;
    Ok(frame.rate().map(|rate| rate.observation(at)))
}

pub(crate) fn rate_selfcheck() -> Result<()> {
    use serde_json::json;
    for value in [
        json!({"rateLimits":{}}),
        json!({"rateLimits":{"primary":null,"secondary":null}}),
        json!({"rateLimits":{},"rateLimitsByLimitId":{}}),
    ] {
        ensure!(
            CodexRate::decode(&value)?.observations(0).is_empty(),
            "empty rate invented windows"
        );
    }
    for reached in [
        "workspace_owner_credits_depleted",
        "workspace_member_credits_depleted",
        "workspace_owner_usage_limit_reached",
        "workspace_member_usage_limit_reached",
    ] {
        let r = CodexRate::decode(
            &json!({"rateLimits":{"primary":{"usedPercent":2},"rateLimitReachedType":reached}}),
        )?;
        ensure!(
            r.observations(0)[0].status == QuotaStatus::Unknown,
            "credits became available"
        );
    }
    for invalid in [
        json!({}),
        json!({"rateLimits":null}),
        json!({"rateLimits":{"limitId":1}}),
        json!({"rateLimits":{"rateLimitReachedType":"future"}}),
        json!({"rateLimits":{"spendControlReached":1}}),
        json!({"rateLimits":{"primary":{"usedPercent":2147483648_i64}}}),
        json!({"rateLimits":{"primary":{"usedPercent":2,"resetsAt":i64::MAX}}}),
        json!({"rateLimits":{},"ordinaryUsageAllowed":1}),
        json!({"rateLimits":{},"rateLimitsByLimitId":{"x":null}}),
        json!({"rateLimits":{"primary":false},"rateLimitsByLimitId":{}}),
        json!({"rateLimits":[null,[20,null,null],null,null,null]}),
        json!({"rateLimits":{},"rateLimitsByLimitId":{"x":{"secondary":[20,null,null]}}}),
    ] {
        ensure!(
            CodexRate::decode(&invalid).is_err(),
            "malformed rate accepted"
        );
    }
    // The flags are structurally distinct but never authorize ordinary usage.
    for (v, p) in [
        (json!({"rateLimits":{}}), RatePresence::Absent),
        (
            json!({"rateLimits":{},"ordinaryUsageAllowed":null}),
            RatePresence::Null,
        ),
        (
            json!({"rateLimits":{},"ordinaryUsageAllowed":false}),
            RatePresence::Value(false),
        ),
        (
            json!({"rateLimits":{},"ordinaryUsageAllowed":true}),
            RatePresence::Value(true),
        ),
    ] {
        ensure!(CodexRate::decode(&v)?.ordinary == p, "rate presence lost");
    }
    Ok(())
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
                &json!({"type":"system","subtype":"api_retry","attempt":1,"max_retries":2,"retry_delay_ms":0,"error_status":429,"status_code":429}),
                1
            )
            .unwrap()
            .is_none()
        );
        assert!(claude_window(&json!({"type":"rate_limit_event","session_id":"native","rate_limit_info":{"status":"rejected","rateLimitType":"new_unknown"}}),1).is_err());
        assert!(codex_subscription_error(
            &json!({"codexErrorInfo":"usageLimitExceeded"})
        ));
    }
    #[test]
    fn native_optional_windows_and_buckets_keep_unknown_fields_unknown() {
        let values=codex_windows(&json!({"rateLimits":{},"rateLimitsByLimitId":{"codex":{"primary":{"usedPercent":100,"resetsAt":123},"secondary":null},"review":{"primary":{"usedPercent":20},"secondary":null}}}),1).unwrap();
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
        let c=claude_window(&json!({"type":"rate_limit_event","session_id":"native","rate_limit_info":{"status":"rejected","rateLimitType":"five_hour","resetsAt":123}}),1).unwrap().unwrap();
        assert_eq!(c.used_percent, None);
        assert_eq!(c.status, QuotaStatus::Exhausted);
    }
}

#[cfg(test)]
mod r1_tests {
    use super::*;
    use serde_json::json;

    /// Missing required snapshots and malformed evidence fail the wire contract without inventing quota.
    #[test]
    fn missing_and_malformed_rate_snapshots_are_protocol_failures() {
        for value in [
            json!({}),
            json!({"ordinaryUsageAllowed":true}),
            json!({"rateLimitsByLimitId":{"x":{"primary":{"usedPercent":20}}}}),
            Value::Null,
            json!({"rateLimits":null}),
            json!({"rateLimits":false}),
            json!({"rateLimits":{"primary":true}}),
            json!({"rateLimits":{},"ordinaryUsageAllowed":1}),
            json!({"rateLimits":{},"rateLimitsByLimitId":{"x":null}}),
            // Sequence and enum-map forms are never typed rate evidence.
            json!({"rateLimits":[null,[20,null,null],null,null,null]}),
            json!({"rateLimits":{"primary":[20,null,null]}}),
            json!({"rateLimits":{},"rateLimitsByLimitId":{"x":[null,[20,null,null],null,null,null]}}),
            json!({"rateLimits":{"primary":{"usedPercent":20},"rateLimitReachedType":{"rate_limit_reached":null}}}),
        ] {
            let error = codex_windows(&value, 0).unwrap_err();
            assert_eq!(
                error.downcast_ref::<NativeFailure>(),
                Some(&NativeFailure::ProtocolFailure),
                "{value}"
            );
        }
        assert!(
            codex_windows(&json!({"rateLimits":{}}), 0)
                .unwrap()
                .is_empty()
        );
        let observations =
            codex_windows(&json!({"rateLimits":{"primary":{"usedPercent":20}}}), 0).unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].status, QuotaStatus::Available);
    }

    /// R-1 validates both snapshots before classifying windows and credit flags.
    #[test]
    fn typed_rate_selfcheck_and_live_consumer_share_classification() {
        rate_selfcheck().unwrap();
        let checked = super::super::claude_wire::decoder_selfcheck().unwrap();
        let _ = checked;
        for (reached, spend, percent, status) in [
            (Value::Null, false, 20, QuotaStatus::Available),
            (
                json!("rate_limit_reached"),
                false,
                20,
                QuotaStatus::Exhausted,
            ),
            (
                json!("workspace_owner_credits_depleted"),
                false,
                20,
                QuotaStatus::Unknown,
            ),
            (Value::Null, true, 20, QuotaStatus::Unknown),
            (
                json!("workspace_member_usage_limit_reached"),
                true,
                100,
                QuotaStatus::Exhausted,
            ),
            (Value::Null, false, -1, QuotaStatus::Unknown),
            (Value::Null, false, 101, QuotaStatus::Exhausted),
        ] {
            let value = json!({"rateLimits":{"primary":{"usedPercent":percent},"rateLimitReachedType":reached,"spendControlReached":spend}});
            let observations = codex_windows(&value, 1).unwrap();
            assert_eq!(
                observations[0].status, status,
                "R-1: quota classification differs"
            );
            assert_eq!(
                observations[0].confirmed_subscription,
                status == QuotaStatus::Exhausted
            );
        }
        let mut map = serde_json::Map::new();
        for i in 0..32 {
            map.insert(format!("bucket-{i}"), json!({}));
        }
        let mut value = json!({"rateLimits":{},"rateLimitsByLimitId":map});
        assert!(codex_windows(&value, 1).unwrap().is_empty());
        value["rateLimitsByLimitId"]["extra"] = json!({});
        assert!(codex_windows(&value, 1).is_err(), "R-1: map budget widened");
        assert!(
            codex_windows(
                &json!({"rateLimits":{"primary":true},"rateLimitsByLimitId":{}}),
                1
            )
            .is_err(),
            "R-1: unselected malformed fallback hidden"
        );
    }

    /// A rate notification is observed only through its declared `rateLimits` snapshot.
    #[test]
    fn u4r1b_rate_notification_reads_only_the_declared_snapshot() {
        let declared =
            json!({"rateLimits":{"limitId":"codex","primary":{"usedPercent":100,"resetsAt":123}}});
        let baseline =
            serde_json::to_value(codex_notification_windows(&declared, 1).unwrap()).unwrap();
        assert_eq!(baseline.as_array().unwrap().len(), 1);
        assert_eq!(baseline[0]["status"], json!(QuotaStatus::Exhausted));
        for (key, undeclared) in [
            ("rateLimitsByLimitId", json!({})),
            (
                "rateLimitsByLimitId",
                json!({"other":{"primary":{"usedPercent":20}}}),
            ),
            ("rateLimitsByLimitId", json!({"x":null})),
            ("ordinaryUsageAllowed", json!(1)),
            ("ordinaryUsageAllowed", json!(true)),
        ] {
            let mut value = declared.clone();
            value[key] = undeclared;
            assert_eq!(
                serde_json::to_value(codex_notification_windows(&value, 1).unwrap()).unwrap(),
                baseline,
                "{value}"
            );
        }
        for malformed in [
            json!({}),
            json!({"rateLimits":null}),
            json!({"rateLimits":{"primary":true}}),
            json!({"rateLimits":{"primary":{"usedPercent":"100"}}}),
            json!({"rateLimits":[null,[20,null,null],null,null,null]}),
        ] {
            assert_eq!(
                codex_notification_windows(&malformed, 1)
                    .unwrap_err()
                    .downcast_ref::<NativeFailure>(),
                Some(&NativeFailure::ProtocolFailure),
                "{malformed}"
            );
        }
    }
}
