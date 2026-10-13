//! The finite M0 Claude receive contract. Extra fields remain bounded and hashed;
//! they are never variant tags, classifications, or execution authority.
use super::{LINE_LIMIT, failure};
use crate::adapter::{AdapterResult, ErrorKind};
use crate::execution::{native_result::digest, strict_json};
use serde_json::{Map, Value, json};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Presence<T> {
    Absent,
    Null,
    Value(T),
}
impl<T: Copy> Presence<T> {
    pub(super) fn value(&self) -> Option<T> {
        match self {
            Self::Value(value) => Some(*value),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Telemetry {
    pub usage: Presence<()>,
    pub input: Presence<u64>,
    pub output: Presence<u64>,
    pub cache_read: Presence<u64>,
    pub cache_write: Presence<u64>,
    pub cost: Presence<f64>,
    pub api_ms: Presence<u64>,
}

/// Only the decoder can construct this value. All of the terminal, including
/// foreign/nonhuman/error branches and optional telemetry, has been checked.
#[derive(Clone, Debug)]
pub(in crate::execution) struct ValidatedClaudeResult {
    raw: Value,
    telemetry: Telemetry,
}
impl ValidatedClaudeResult {
    pub(in crate::execution) fn raw(&self) -> &Value {
        &self.raw
    }
    pub(super) fn telemetry(&self) -> &Telemetry {
        &self.telemetry
    }
}

pub(in crate::execution) struct ValidatedClaudeFrame {
    raw: Value,
    result: Option<ValidatedClaudeResult>,
    human_origin: bool,
    rate: Option<ValidatedRate>,
}
impl ValidatedClaudeFrame {
    /// `Lines` already checks original bytes for duplicate keys and budgets.
    /// Recheck bounds here so all other producers (including future S/P) have
    /// the same bounded contract. This cannot detect duplicates lost upstream;
    /// byte producers must use strict_json::decode on the original bytes first.
    pub(in crate::execution) fn decode(value: Value, _: &DecoderChecked) -> AdapterResult<Self> {
        Self::decode_contract(value)
    }
    fn decode_contract(value: Value) -> AdapterResult<Self> {
        let bytes = serde_json::to_vec(&value).map_err(|_| rejected())?;
        let mut raw = strict_json::decode(&bytes, limits()).map_err(|_| rejected())?;
        let object = object_value(&raw)?;
        let kind = string(required(object, "type")?)?.to_owned();
        optional(object, "session_id", false, id)?;
        let human_origin = origin(object.get("origin"))?;
        if let Some(uuid) = object.get("uuid") {
            let uuid = uuid::Uuid::parse_str(string(uuid)?).map_err(|_| rejected())?;
            raw["uuid"] = Value::String(uuid.hyphenated().to_string());
        }
        // UUID normalization may have changed the object; validate the retained
        // representation and keep every unknown field in its canonical digest.
        let object = object_value(&raw)?;
        let mut result = None;
        let mut rate = None;
        match kind.as_str() {
            "system" => system(object)?,
            "assistant" => {
                id(required(object, "session_id")?)?;
                nullable(required(object, "parent_tool_use_id")?, id)?;
                let message = object_value(required(object, "message")?)?;
                enumeration(required(message, "role")?, &["assistant"])?;
                blocks(required(message, "content")?, BlockSet::Assistant)?;
                optional(object, "error", false, |value| {
                    enumeration(
                        value,
                        &[
                            "authentication_failed",
                            "oauth_org_not_allowed",
                            "account_on_hold",
                            "billing_error",
                            "rate_limit",
                            "overloaded",
                            "invalid_request",
                            "model_not_found",
                            "server_error",
                            "max_output_tokens",
                            "cloud_credential_error",
                            "unknown",
                        ],
                    )
                    .map(|_| ())
                })?;
            }
            "user" => {
                optional(object, "parent_tool_use_id", true, id)?;
                let message = object_value(required(object, "message")?)?;
                enumeration(required(message, "role")?, &["user"])?;
                string_or_blocks(required(message, "content")?, BlockSet::User)?;
            }
            "result" => {
                id(required(object, "session_id")?)?;
                let subtype = enumeration(
                    required(object, "subtype")?,
                    &[
                        "success",
                        "error_during_execution",
                        "error_max_turns",
                        "error_max_budget_usd",
                        "error_max_structured_output_retries",
                    ],
                )?;
                let is_error = boolean(required(object, "is_error")?)?;
                if subtype == "success" {
                    string(required(object, "result")?)?;
                    // RunState also consults errors when a success-subtype
                    // result reports is_error=true. Never let this consumed
                    // field escape through the additional-field rule.
                    optional(object, "errors", false, strings)?;
                } else {
                    if !is_error {
                        return Err(rejected());
                    }
                    strings(required(object, "errors")?)?;
                    optional(object, "result", false, |value| string(value).map(|_| ()))?;
                }
                optional(object, "api_error_status", true, |value| {
                    u16::try_from(uint(value)?)
                        .map_err(|_| rejected())
                        .map(|_| ())
                })?;
                // structured_output is bounded opaque JSON, including null.
                // The whole-value strict budget check above covers this sink.
                result = Some(ValidatedClaudeResult {
                    raw: raw.clone(),
                    telemetry: telemetry(object)?,
                });
            }
            "control_request" => {
                id(required(object, "request_id")?)?;
                let request = object_value(required(object, "request")?)?;
                enumeration(required(request, "subtype")?, &["can_use_tool"])?;
                id(required(request, "tool_use_id")?)?;
                id(required(request, "tool_name")?)?;
                object_value(required(request, "input")?)?;
            }
            "control_response" => {
                let response = object_value(required(object, "response")?)?;
                id(required(response, "request_id")?)?;
                match enumeration(required(response, "subtype")?, &["success", "error"])? {
                    "success" => {
                        object_value(required(response, "response")?)?;
                        optional(response, "error", false, |value| string(value).map(|_| ()))?;
                    }
                    _ => {
                        string(required(response, "error")?)?;
                        optional(response, "response", false, |value| {
                            object_value(value).map(|_| ())
                        })?;
                    }
                }
            }
            "stream_event" => {
                optional(object, "parent_tool_use_id", true, id)?;
                stream(object_value(required(object, "event")?)?)?;
            }
            "tool_progress" => {
                for key in ["tool_use_id", "tool_name"] {
                    id(required(object, key)?)?;
                }
                nullable(required(object, "parent_tool_use_id")?, id)?;
                nonnegative(required(object, "elapsed_time_seconds")?)?;
            }
            "tool_use_summary" => {
                string(required(object, "summary")?)?;
                for value in array(required(object, "preceding_tool_use_ids")?)? {
                    id(value)?;
                }
            }
            "auth_status" => {
                boolean(required(object, "isAuthenticating")?)?;
                strings(required(object, "output")?)?;
                optional(object, "error", false, |value| string(value).map(|_| ()))?;
            }
            "rate_limit_event" => {
                id(required(object, "session_id")?)?;
                rate = Some(decode_rate(required(object, "rate_limit_info")?)?);
            }
            _ => return Err(rejected()),
        }
        Ok(Self {
            raw,
            result,
            human_origin,
            rate,
        })
    }
    pub(in crate::execution) fn raw(&self) -> &Value {
        &self.raw
    }
    pub(in crate::execution) fn result(&self) -> Option<&ValidatedClaudeResult> {
        self.result.as_ref()
    }
    pub(in crate::execution) fn human_origin(&self) -> bool {
        self.human_origin
    }
    pub(in crate::execution) fn rate(&self) -> Option<&ValidatedRate> {
        self.rate.as_ref()
    }
    pub(in crate::execution) fn digest(&self) -> String {
        digest(&serde_json::to_vec(&canonical(&self.raw)).expect("bounded Value encoding"))
    }
}

fn canonical(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut keys = object.keys().collect::<Vec<_>>();
            keys.sort();
            Value::Object(
                keys.into_iter()
                    .map(|key| (key.clone(), canonical(&object[key])))
                    .collect(),
            )
        }
        Value::Array(values) => Value::Array(values.iter().map(canonical).collect()),
        _ => value.clone(),
    }
}
fn limits() -> strict_json::Limits {
    strict_json::Limits {
        frame_bytes: LINE_LIMIT,
        ..strict_json::Limits::CEILINGS
    }
}
fn rejected() -> crate::adapter::AdapterError {
    failure(
        ErrorKind::ParseFailure,
        "Claude finite wire contract rejected",
    )
}
fn object_value(value: &Value) -> AdapterResult<&Map<String, Value>> {
    value.as_object().ok_or_else(rejected)
}
fn required<'a>(object: &'a Map<String, Value>, key: &str) -> AdapterResult<&'a Value> {
    object.get(key).ok_or_else(rejected)
}
fn string(value: &Value) -> AdapterResult<&str> {
    value.as_str().ok_or_else(rejected)
}
fn id(value: &Value) -> AdapterResult<()> {
    super::bounded_id(value).map(|_| ())
}
fn array(value: &Value) -> AdapterResult<&Vec<Value>> {
    value.as_array().ok_or_else(rejected)
}
fn strings(value: &Value) -> AdapterResult<()> {
    for value in array(value)? {
        string(value)?;
    }
    Ok(())
}
fn boolean(value: &Value) -> AdapterResult<bool> {
    value.as_bool().ok_or_else(rejected)
}
fn enumeration<'a>(value: &'a Value, allowed: &[&str]) -> AdapterResult<&'a str> {
    let text = string(value)?;
    if allowed.contains(&text) {
        Ok(text)
    } else {
        Err(rejected())
    }
}
fn uint(value: &Value) -> AdapterResult<u64> {
    if let Some(n) = value.as_u64() {
        return Ok(n);
    }
    // JSON number includes integral floats. Casting is checked before Rust's
    // saturating float conversion; 2^64 is never accepted as u64::MAX.
    let n = value.as_f64().ok_or_else(rejected)?;
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 || n >= 18_446_744_073_709_551_616.0 {
        return Err(rejected());
    }
    Ok(n as u64)
}
fn nonnegative(value: &Value) -> AdapterResult<f64> {
    value
        .as_f64()
        .filter(|n| n.is_finite() && *n >= 0.0)
        .ok_or_else(rejected)
}
fn nullable(value: &Value, check: impl FnOnce(&Value) -> AdapterResult<()>) -> AdapterResult<()> {
    if value.is_null() {
        Ok(())
    } else {
        check(value)
    }
}
fn optional(
    object: &Map<String, Value>,
    key: &str,
    null: bool,
    check: impl FnOnce(&Value) -> AdapterResult<()>,
) -> AdapterResult<()> {
    match object.get(key) {
        None => Ok(()),
        Some(Value::Null) if null => Ok(()),
        Some(value) => check(value),
    }
}
fn presence<T>(
    value: Option<&Value>,
    parse: impl FnOnce(&Value) -> AdapterResult<T>,
) -> AdapterResult<Presence<T>> {
    match value {
        None => Ok(Presence::Absent),
        Some(Value::Null) => Ok(Presence::Null),
        Some(value) => Ok(Presence::Value(parse(value)?)),
    }
}
fn telemetry(object: &Map<String, Value>) -> AdapterResult<Telemetry> {
    let usage = presence(object.get("usage"), |value| object_value(value).map(|_| ()))?;
    let counters = match usage {
        Presence::Value(()) => object.get("usage").and_then(Value::as_object),
        _ => None,
    };
    let counter = |key| presence(counters.and_then(|object| object.get(key)), uint);
    Ok(Telemetry {
        usage,
        input: counter("input_tokens")?,
        output: counter("output_tokens")?,
        cache_read: counter("cache_read_input_tokens")?,
        cache_write: counter("cache_creation_input_tokens")?,
        cost: presence(object.get("total_cost_usd"), nonnegative)?,
        api_ms: presence(object.get("duration_api_ms"), uint)?,
    })
}
fn origin(value: Option<&Value>) -> AdapterResult<bool> {
    let object = match value {
        None | Some(Value::Null) => return Ok(true),
        Some(value) => object_value(value)?,
    };
    let kind = enumeration(
        required(object, "kind")?,
        &[
            "human",
            "channel",
            "peer",
            "task-notification",
            "coordinator",
            "auto-continuation",
            "unclassified",
        ],
    )?;
    match kind {
        "channel" => {
            string(required(object, "server")?)?;
        }
        "peer" => {
            id(required(object, "from")?)?;
            optional(object, "fromMode", false, |value| {
                enumeration(value, &["bypass", "prompting"]).map(|_| ())
            })?;
            for key in ["name", "fromSession", "senderTaskId", "body"] {
                optional(object, key, false, |value| string(value).map(|_| ()))?;
            }
            optional(object, "verifiedPeerPid", false, |value| {
                uint(value).map(|_| ())
            })?;
        }
        "task-notification" => {
            optional(object, "subkind", false, |value| {
                enumeration(value, &["scheduled-trigger", "peer-send-message"]).map(|_| ())
            })?;
            for key in ["fireReason", "runId"] {
                optional(object, key, false, |value| string(value).map(|_| ()))?;
            }
        }
        _ => {}
    }
    Ok(kind == "human")
}

fn system(object: &Map<String, Value>) -> AdapterResult<()> {
    match string(required(object, "subtype")?)? {
        "init" => {
            id(required(object, "session_id")?)?;
            string(required(object, "cwd")?)?;
            strings(required(object, "tools")?)?;
            for value in array(required(object, "mcp_servers")?)? {
                let server = object_value(value)?;
                string(required(server, "name")?)?;
                string(required(server, "status")?)?;
            }
        }
        "session_state_changed" => {
            enumeration(
                required(object, "state")?,
                &["idle", "running", "requires_action"],
            )?;
        }
        "task_started" => {
            id(required(object, "task_id")?)?;
            string(required(object, "description")?)?;
            optional(object, "task_type", false, |value| {
                string(value).map(|_| ())
            })?;
        }
        "task_notification" => {
            id(required(object, "task_id")?)?;
            enumeration(
                required(object, "status")?,
                &["completed", "failed", "stopped"],
            )?;
            for key in ["output_file", "summary"] {
                string(required(object, key)?)?;
            }
            optional(object, "usage", false, |value| {
                let usage = object_value(value)?;
                for key in ["total_tokens", "tool_uses", "duration_ms"] {
                    nonnegative(required(usage, key)?)?;
                }
                Ok(())
            })?;
        }
        "task_updated" => {
            id(required(object, "task_id")?)?;
            let patch = object_value(required(object, "patch")?)?;
            optional(patch, "status", false, |value| {
                enumeration(
                    value,
                    &["pending", "running", "completed", "failed", "killed"],
                )
                .map(|_| ())
            })?;
            for key in ["description", "error"] {
                optional(patch, key, false, |value| string(value).map(|_| ()))?;
            }
            for key in ["end_time", "total_paused_ms"] {
                optional(patch, key, false, |value| nonnegative(value).map(|_| ()))?;
            }
            optional(patch, "is_backgrounded", false, |value| {
                boolean(value).map(|_| ())
            })?;
        }
        "api_retry" => {
            for key in ["attempt", "max_retries", "retry_delay_ms"] {
                uint(required(object, key)?)?;
            }
            nullable(required(object, "error_status")?, |value| {
                u16::try_from(uint(value)?)
                    .map(|_| ())
                    .map_err(|_| rejected())
            })?;
        }
        _ => return Err(rejected()),
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum BlockSet {
    Assistant,
    User,
    Text,
}
fn string_or_blocks(value: &Value, set: BlockSet) -> AdapterResult<()> {
    if value.is_string() {
        Ok(())
    } else {
        blocks(value, set)
    }
}
fn blocks(value: &Value, set: BlockSet) -> AdapterResult<()> {
    for value in array(value)? {
        block(value, set)?;
    }
    Ok(())
}
fn block(value: &Value, set: BlockSet) -> AdapterResult<()> {
    let object = object_value(value)?;
    match string(required(object, "type")?)? {
        "text" => {
            string(required(object, "text")?)?;
        }
        "thinking" if matches!(set, BlockSet::Assistant) => {
            string(required(object, "thinking")?)?;
            optional(object, "signature", false, |value| {
                string(value).map(|_| ())
            })?;
        }
        "tool_use" if matches!(set, BlockSet::Assistant) => {
            for key in ["id", "name"] {
                id(required(object, key)?)?;
            }
            object_value(required(object, "input")?)?;
        }
        "tool_result" if matches!(set, BlockSet::User) => {
            id(required(object, "tool_use_id")?)?;
            string_or_blocks(required(object, "content")?, BlockSet::Text)?;
            optional(object, "is_error", false, |value| {
                boolean(value).map(|_| ())
            })?;
        }
        _ => return Err(rejected()),
    }
    Ok(())
}
fn stream(object: &Map<String, Value>) -> AdapterResult<()> {
    match string(required(object, "type")?)? {
        "message_start" => {
            let message = object_value(required(object, "message")?)?;
            id(required(message, "id")?)?;
            enumeration(required(message, "type")?, &["message"])?;
            enumeration(required(message, "role")?, &["assistant"])?;
            string(required(message, "model")?)?;
            blocks(required(message, "content")?, BlockSet::Assistant)?;
            for key in ["stop_reason", "stop_sequence"] {
                nullable(required(message, key)?, |value| string(value).map(|_| ()))?;
            }
            stream_usage(required(message, "usage")?, true)?;
        }
        "message_delta" => {
            let delta = object_value(required(object, "delta")?)?;
            for key in ["stop_reason", "stop_sequence"] {
                nullable(required(delta, key)?, |value| string(value).map(|_| ()))?;
            }
            stream_usage(required(object, "usage")?, false)?;
        }
        "content_block_start" => {
            uint(required(object, "index")?)?;
            block(required(object, "content_block")?, BlockSet::Assistant)?;
        }
        "content_block_delta" => {
            uint(required(object, "index")?)?;
            let delta = object_value(required(object, "delta")?)?;
            let key = match string(required(delta, "type")?)? {
                "text_delta" => "text",
                "thinking_delta" => "thinking",
                "input_json_delta" => "partial_json",
                "signature_delta" => "signature",
                _ => return Err(rejected()),
            };
            string(required(delta, key)?)?;
        }
        "content_block_stop" => {
            uint(required(object, "index")?)?;
        }
        "message_stop" | "ping" => {}
        _ => return Err(rejected()),
    }
    Ok(())
}
fn stream_usage(value: &Value, input: bool) -> AdapterResult<()> {
    let object = object_value(value)?;
    uint(required(object, "output_tokens")?)?;
    if input {
        uint(required(object, "input_tokens")?)?;
    }
    for key in ["cache_read_input_tokens", "cache_creation_input_tokens"] {
        optional(object, key, false, |value| uint(value).map(|_| ()))?;
    }
    Ok(())
}

/// A classified event may have no usable timestamp/utilization. That is a
/// valid empty observation, distinct from malformed and never-observed.
pub(in crate::execution) struct ValidatedRate {
    bucket: String,
    rejected: bool,
    reset_ms: Presence<i64>,
    utilization: Presence<f64>,
}
impl ValidatedRate {
    pub(in crate::execution) fn observation(
        &self,
        at: i64,
    ) -> crate::execution::quota::QuotaObservation {
        use crate::execution::quota::{QuotaObservation, QuotaStatus};
        let reset = self.reset_ms.value();
        QuotaObservation {
            provider: "claude".into(),
            account_key: "unknown".into(),
            bucket: self.bucket.clone(),
            window_id: reset.map_or_else(
                || format!("unknown-{}", self.bucket),
                |value| value.to_string(),
            ),
            status: if self.rejected {
                QuotaStatus::Exhausted
            } else {
                QuotaStatus::Available
            },
            used_percent: self.utilization.value().map(|n| n * 100.0),
            resets_at: reset,
            observed_at: at,
            source_version: "claude/rate_limit_event".into(),
            confirmed_subscription: self.rejected,
        }
    }
}
fn decode_rate(value: &Value) -> AdapterResult<ValidatedRate> {
    let object = object_value(value)?;
    let bucket = enumeration(
        required(object, "rateLimitType")?,
        &[
            "five_hour",
            "seven_day",
            "seven_day_opus",
            "seven_day_sonnet",
            "seven_day_overage_included",
        ],
    )?
    .to_owned();
    let rejected = enumeration(
        required(object, "status")?,
        &["allowed", "allowed_warning", "rejected"],
    )? == "rejected";
    let reset_ms = presence(object.get("resetsAt"), |value| {
        value
            .as_i64()
            .and_then(|n| n.checked_mul(1000))
            .ok_or_else(rejected_rate)
    })?;
    let utilization = presence(object.get("utilization"), |value| {
        value
            .as_f64()
            .filter(|n| n.is_finite() && (0.0..=1.0).contains(n))
            .ok_or_else(rejected_rate)
    })?;
    Ok(ValidatedRate {
        bucket,
        rejected,
        reset_ms,
        utilization,
    })
}
fn rejected_rate() -> crate::adapter::AdapterError {
    rejected()
}

/// Nongrant evidence from the pure decoder, not evidence of a live event or
/// installation/startup qualification. Future S must require this same value.
pub(in crate::execution) struct DecoderChecked {
    _private: (),
}
pub(in crate::execution) fn decoder_selfcheck() -> AdapterResult<DecoderChecked> {
    // These are literal contract vectors, independent of observed CLI output.
    // Check both acceptance and rejection; silently dropping malformed to None
    // cannot produce the checked value.
    let empty_frame = ValidatedClaudeFrame::decode_contract(
        json!({"type":"rate_limit_event","session_id":"selfcheck","rate_limit_info":{"rateLimitType":"five_hour", "status":"allowed"}}),
    )?;
    let usable_frame = ValidatedClaudeFrame::decode_contract(
        json!({"type":"rate_limit_event","session_id":"selfcheck","rate_limit_info":{"rateLimitType":"seven_day", "status":"rejected", "resetsAt":123, "utilization":0.5}}),
    )?;
    let empty = empty_frame.rate().ok_or_else(rejected)?;
    let usable = usable_frame.rate().ok_or_else(rejected)?;
    if empty.reset_ms != Presence::Absent
        || empty.utilization != Presence::Absent
        || empty.rejected
        || !usable.rejected
        || usable.reset_ms != Presence::Value(123000)
        || usable.utilization != Presence::Value(0.5)
    {
        return Err(rejected());
    }
    for invalid in [
        json!({}),
        json!({"rateLimitType":"future_window","status":"allowed"}),
        json!({"rateLimitType":"five_hour","status":"future_status"}),
        json!({"rateLimitType":"five_hour","status":"allowed","resetsAt":"123"}),
        json!({"rateLimitType":"five_hour","status":"allowed","resetsAt":i64::MAX}),
        json!({"rateLimitType":"five_hour","status":"allowed","utilization":1.01}),
    ] {
        let frame =
            json!({"type":"rate_limit_event","session_id":"selfcheck","rate_limit_info":invalid});
        if ValidatedClaudeFrame::decode_contract(frame).is_ok() {
            return Err(rejected());
        }
    }
    for invalid in [
        json!({"type":"rate_limit_event","rate_limit_info":{"rateLimitType":"five_hour","status":"allowed"}}),
        json!({"type":"rate_limit_event","session_id":null,"rate_limit_info":{"rateLimitType":"five_hour","status":"allowed"}}),
        json!({"type":"rate_limit_event","session_id":"selfcheck","rate_limit_info":null}),
    ] {
        if ValidatedClaudeFrame::decode_contract(invalid).is_ok() {
            return Err(rejected());
        }
    }
    let nonrate = ValidatedClaudeFrame::decode_contract(
        json!({"type":"auth_status","isAuthenticating":false,"output":[]}),
    )?;
    if nonrate.rate().is_some() {
        return Err(rejected());
    }
    Ok(DecoderChecked { _private: () })
}

#[cfg(test)]
mod tests;
