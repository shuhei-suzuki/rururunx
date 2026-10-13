//! Bounded Claude stream-json values; provider text is never an ownership authority.
use crate::adapter::{AdapterError, AdapterResult, ErrorKind};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
mod decoded;
pub(super) use decoded::{ValidatedClaudeFrame, ValidatedClaudeResult, decoder_selfcheck};

pub(super) fn failure(kind: ErrorKind, message: impl Into<String>) -> AdapterError {
    AdapterError {
        kind,
        message: message.into(),
    }
}
pub(super) const LINE_LIMIT: usize = 2 * 1024 * 1024;
pub(super) fn bounded_id(value: &Value) -> AdapterResult<String> {
    value
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control))
        .map(str::to_owned)
        .ok_or_else(|| {
            failure(
                ErrorKind::ParseFailure,
                "invalid native message/request identity",
            )
        })
}
pub(super) fn verify_version(version: &str) -> AdapterResult<()> {
    // A pinned local version/schema baseline, not native Agent conformance.
    // Real account/settings/hook qualification belongs to Phase 3. Future
    // versions require a capability probe, rather than inheriting permissions.
    if version.split_whitespace().next() != Some("2.1.294") {
        return Err(failure(
            ErrorKind::UnsupportedCapability,
            "unverified native Claude wire version",
        ));
    }
    Ok(())
}
pub(super) fn control(subtype: &str, id: &str) -> Value {
    let request = if subtype == "initialize" {
        json!({"subtype":subtype,"hooks":null})
    } else {
        json!({"subtype":subtype})
    };
    json!({"type":"control_request","request_id":id,"request":request})
}
pub(super) fn control_result(frame: &ValidatedClaudeFrame, id: &str) -> AdapterResult<Value> {
    let message = frame.raw();
    if message["type"] != "control_response"
        || message["response"]["request_id"] != id
        || message["response"]["subtype"] != "success"
        || !message["response"]["response"].is_object()
    {
        return Err(failure(
            ErrorKind::ParseFailure,
            "uncorrelated/error native control response",
        ));
    }
    Ok(message["response"]["response"].clone())
}
#[derive(Clone, Debug)]
pub(super) struct Pending {
    pub request_id: String,
    pub tool_use_id: String,
    pub tool_name: String,
    pub input: Value,
    pub hash: String,
}
impl Pending {
    pub fn parse(frame: &ValidatedClaudeFrame, native: &str) -> AdapterResult<Self> {
        let message = frame.raw();
        let request_id = bounded_id(&message["request_id"])?;
        let request = &message["request"];
        if request["subtype"] != "can_use_tool" || !request["input"].is_object() {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "unsupported native control request",
            ));
        }
        let tool_use_id = bounded_id(&request["tool_use_id"])?;
        let tool_name = bounded_id(&request["tool_name"])?;
        let input = request["input"].clone();
        let bytes = serde_json::to_vec(&json!([native, request_id, tool_use_id, tool_name, input]))
            .map_err(|_| {
                failure(
                    ErrorKind::ParseFailure,
                    "native operation serialization failed",
                )
            })?;
        if bytes.len() > 64 * 1024 {
            return Err(failure(
                ErrorKind::ParseFailure,
                "native permission operation too large",
            ));
        }
        let hash = format!("{:x}", Sha256::digest(bytes));
        Ok(Self {
            request_id,
            tool_use_id,
            tool_name,
            input,
            hash,
        })
    }
    pub fn reply(&self, allow: bool) -> Value {
        let response = if allow {
            json!({"behavior":"allow","updatedInput":self.input})
        } else {
            json!({"behavior":"deny","message":"Scoped runtime did not grant this operation"})
        };
        json!({"type":"control_response","response":{"subtype":"success","request_id":self.request_id,"response":response}})
    }
    pub fn public(&self, native: &str) -> Value {
        // Tool input may contain secrets; only the bounded operation digest is
        // persisted/public. The original input stays in this process's memory.
        json!({"native_uuid":native,"request_id":self.request_id,"tool_use_id":self.tool_use_id,"tool_name":self.tool_name,"operation_hash":self.hash})
    }
}
/// Validated telemetry only; neither metrics nor its absence grants execution.
#[derive(Clone, Debug)]
pub struct Metrics {
    input: Option<u64>,
    output: Option<u64>,
    cache_read: Option<u64>,
    cache_write: Option<u64>,
    cost: Option<f64>,
    api_ms: Option<u64>,
    cumulative_cost: Option<f64>,
    cumulative_api_ms: Option<u64>,
}
impl Metrics {
    pub(super) fn from_result(result: &ValidatedClaudeResult, resumed: bool) -> Self {
        let telemetry = result.telemetry();
        let cost = telemetry.cost.value();
        let api_ms = telemetry.api_ms.value();
        let usage = matches!(telemetry.usage, decoded::Presence::Value(()));
        // This receive contract does not attribute resumed gauges to a phase.
        // Monotonicity cannot establish whether gauges are cumulative or reset.
        // Retain actual raw gauges, but never invent a resumed phase delta.
        let delta_cost = (!resumed).then_some(cost).flatten();
        let delta_ms = (!resumed).then_some(api_ms).flatten();
        Self {
            input: usage.then(|| telemetry.input.value()).flatten(),
            output: usage.then(|| telemetry.output.value()).flatten(),
            cache_read: usage.then(|| telemetry.cache_read.value()).flatten(),
            cache_write: usage.then(|| telemetry.cache_write.value()).flatten(),
            cost: delta_cost,
            api_ms: delta_ms,
            cumulative_cost: cost,
            cumulative_api_ms: api_ms,
        }
    }
    pub fn input(&self) -> Option<u64> {
        self.input
    }
    pub fn output(&self) -> Option<u64> {
        self.output
    }
    pub fn cache_read(&self) -> Option<u64> {
        self.cache_read
    }
    pub fn cache_write(&self) -> Option<u64> {
        self.cache_write
    }
    pub fn cost(&self) -> Option<f64> {
        self.cost
    }
    pub fn api_ms(&self) -> Option<u64> {
        self.api_ms
    }
    pub fn cumulative_cost(&self) -> Option<f64> {
        self.cumulative_cost
    }
    pub fn cumulative_api_ms(&self) -> Option<u64> {
        self.cumulative_api_ms
    }
}
#[derive(Default)]
pub(super) struct RunState {
    ids: BTreeMap<String, String>,
    tasks: BTreeSet<String>,
    pub initialized: bool,
    pub result: Option<ValidatedClaudeResult>,
    state: Option<String>,
    injected_turn: bool,
    await_final_idle: bool,
    pub failed_tasks: bool,
    pub had_background: bool,
    pub terminal_capacity: bool,
    assistant_error: Option<String>,
}
impl RunState {
    fn remember(&mut self, frame: &ValidatedClaudeFrame) -> AdapterResult<bool> {
        if let Some(id) = frame.raw().get("uuid") {
            let id = bounded_id(id)?;
            let signature = frame.digest();
            if let Some(old) = self.ids.get(&id) {
                if old != &signature {
                    return Err(failure(
                        ErrorKind::ParseFailure,
                        "native UUID content changed",
                    ));
                }
                return Ok(false);
            }
            if self.ids.len() >= 8192 {
                return Err(failure(
                    ErrorKind::ParseFailure,
                    "native message count bound exceeded",
                ));
            }
            self.ids.insert(id, signature);
        }
        Ok(true)
    }
    /// Only the bootstrap receive loop has this request correlation. Absence
    /// of optional session_id remains valid; presence must name this Session.
    pub fn initialize_response(
        &mut self,
        frame: &ValidatedClaudeFrame,
        native: &str,
        request: &str,
    ) -> AdapterResult<Value> {
        let response = control_result(frame, request)?;
        if frame.raw().get("session_id").is_some_and(|id| id != native) {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "foreign native bootstrap response",
            ));
        }
        self.remember(frame)?;
        Ok(response)
    }
    pub fn observe(
        &mut self,
        frame: &ValidatedClaudeFrame,
        native: &str,
        cwd: &std::path::Path,
        decision: bool,
    ) -> AdapterResult<bool> {
        let message = frame.raw();
        // Preserve O03 even when this is an identical redelivery of a UUID
        // remembered by bootstrap: the normal loop has no response correlator.
        if message["type"] == "control_response" {
            return Err(failure(
                ErrorKind::ParseFailure,
                "unsupported native output frame",
            ));
        }
        if !self.remember(frame)? {
            return Ok(false);
        }
        // A legal foreign rate event is ignored, but only after full decode and
        // UUID-content validation. It never reaches the subscription pool.
        if message["type"] == "rate_limit_event" && message["session_id"] != native {
            return Ok(false);
        }
        if message.get("session_id").is_some_and(|id| id != native) {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "foreign native message UUID",
            ));
        }
        match message["type"].as_str() {
            Some("system") => match message["subtype"].as_str() {
                Some("init") => {
                    if self.initialized
                        || message["session_id"] != native
                        || message["cwd"].as_str() != cwd.to_str()
                    {
                        return Err(failure(
                            ErrorKind::OwnershipMismatch,
                            "native init identity/workspace mismatch",
                        ));
                    }
                    let tools = message["tools"].as_array().ok_or_else(|| {
                        failure(ErrorKind::ParseFailure, "native tool inventory unavailable")
                    })?;
                    let mcp = message["mcp_servers"].as_array().ok_or_else(|| {
                        failure(ErrorKind::ParseFailure, "native MCP inventory unavailable")
                    })?;
                    if decision && (!tools.is_empty() || !mcp.is_empty()) {
                        return Err(failure(
                            ErrorKind::UnsupportedCapability,
                            "native decision tools were not removed",
                        ));
                    }
                    self.initialized = true;
                }
                Some("session_state_changed") => {
                    let state = message["state"]
                        .as_str()
                        .filter(|s| matches!(*s, "idle" | "running" | "requires_action"))
                        .ok_or_else(|| {
                            failure(ErrorKind::ParseFailure, "unknown native session state")
                        })?;
                    self.state = Some(state.into());
                    if state == "idle" && self.tasks.is_empty() && !self.injected_turn {
                        self.await_final_idle = false;
                    }
                }
                Some("task_started")
                    if matches!(
                        message["task_type"].as_str(),
                        Some("local_agent" | "local_workflow")
                    ) =>
                {
                    self.had_background = true;
                    self.tasks.insert(bounded_id(&message["task_id"])?);
                    if self.tasks.len() > 64 {
                        return Err(failure(
                            ErrorKind::ParseFailure,
                            "native agent task bound exceeded",
                        ));
                    }
                }
                Some("task_notification")
                    if matches!(
                        message["status"].as_str(),
                        Some("completed" | "failed" | "stopped")
                    ) =>
                {
                    self.finish_task(
                        message["task_id"].as_str().unwrap_or(""),
                        message["status"].as_str().unwrap_or(""),
                    );
                }
                Some("task_updated")
                    if matches!(
                        message["patch"]["status"].as_str(),
                        Some("completed" | "failed" | "stopped" | "killed")
                    ) =>
                {
                    self.finish_task(
                        message["task_id"].as_str().unwrap_or(""),
                        message["patch"]["status"].as_str().unwrap_or(""),
                    );
                }
                _ => {}
            },
            Some("assistant") => {
                if !self.initialized {
                    return Err(failure(
                        ErrorKind::ParseFailure,
                        "native assistant before init",
                    ));
                }
                if decision
                    && message["message"]["content"]
                        .as_array()
                        .is_some_and(|v| v.iter().any(|c| c["type"] == "tool_use"))
                {
                    return Err(failure(
                        ErrorKind::UnsupportedCapability,
                        "decision native tool attempt",
                    ));
                }
                if self.result.is_some() {
                    self.had_background = true;
                    self.injected_turn |= message["parent_tool_use_id"].is_null();
                    return Ok(false);
                }
                if self.injected_turn {
                    return Ok(false);
                }
                if message["session_id"] == native
                    && message["parent_tool_use_id"].is_null()
                    && frame.human_origin()
                {
                    // This is a hint for a later owned terminal, not a terminal
                    // itself: the native CLI may retry this assistant error.
                    self.assistant_error = message["error"].as_str().and_then(|s| {
                        matches!(
                            s,
                            "authentication_failed"
                                | "billing_error"
                                | "rate_limit"
                                | "server_error"
                        )
                        .then(|| s.to_owned())
                    });
                }
            }
            Some("result") => {
                if !self.initialized || message["session_id"] != native {
                    return Err(failure(
                        ErrorKind::OwnershipMismatch,
                        "native terminal identity missing",
                    ));
                }
                // Native streaming can inject separate background/peer/channel
                // turns. Their results are not proof for the input we sent.
                let subtype = message["subtype"].as_str().filter(|s| !s.is_empty());
                let is_error = message["is_error"].as_bool();
                if subtype.is_none()
                    || is_error.is_none()
                    || (subtype != Some("success") && is_error != Some(true))
                {
                    return Err(failure(
                        ErrorKind::ParseFailure,
                        "malformed native terminal",
                    ));
                }
                if !frame.human_origin() {
                    self.had_background |= self.result.is_some();
                    self.injected_turn = false;
                    self.failed_tasks |= subtype != Some("success") || is_error != Some(false);
                    return Ok(false);
                }
                if !matches!(
                    subtype,
                    Some(
                        "success"
                            | "error_during_execution"
                            | "error_max_turns"
                            | "error_max_budget_usd"
                            | "error_max_structured_output_retries"
                    )
                ) {
                    return Err(failure(
                        ErrorKind::UnsupportedCapability,
                        "unsupported native terminal subtype",
                    ));
                }
                if message["subtype"] != "success" || message["is_error"] != false {
                    let authentication =
                        message
                            .get("result")
                            .and_then(Value::as_str)
                            .is_some_and(|text| {
                                let lower = text.to_ascii_lowercase();
                                lower.contains("not logged in")
                                    || lower.contains("authentication failed")
                                    || lower.contains("invalid api key")
                            });
                    let api_failure = matches!(subtype, Some("success" | "error_during_execution"));
                    let api_status = frame
                        .result()
                        .ok_or_else(|| {
                            failure(ErrorKind::ParseFailure, "validated native terminal missing")
                        })?
                        .api_error_status();
                    self.terminal_capacity = api_failure
                        && (matches!(
                            self.assistant_error.as_deref(),
                            Some("billing_error" | "rate_limit" | "server_error")
                        ) || matches!(api_status, decoded::Presence::Value(429 | 503))
                            || message
                                .get("result")
                                .and_then(Value::as_str)
                                .is_some_and(capacity_text)
                            || message
                                .get("errors")
                                .and_then(Value::as_array)
                                .is_some_and(|a| {
                                    a.iter()
                                        .take(64)
                                        .any(|s| s.as_str().is_some_and(capacity_text))
                                }));
                    return Err(failure(
                        if authentication
                            || (api_failure
                                && (self.assistant_error.as_deref()
                                    == Some("authentication_failed")
                                    || matches!(api_status, decoded::Presence::Value(401))))
                        {
                            ErrorKind::AuthenticationUnavailable
                        } else {
                            ErrorKind::ProcessFailure
                        },
                        "native turn reported an unsuccessful terminal",
                    ));
                }
                self.result = frame.result().cloned();
            }
            Some("user") => {
                if self.result.is_some()
                    && message["parent_tool_use_id"].is_null()
                    && !frame.human_origin()
                {
                    self.had_background = true;
                    self.injected_turn = true;
                }
            }
            Some(
                "stream_event" | "tool_progress" | "tool_use_summary" | "auth_status"
                | "rate_limit_event" | "control_request",
            ) => {}
            _ => {
                return Err(failure(
                    ErrorKind::ParseFailure,
                    "unsupported native output frame",
                ));
            }
        }
        Ok(true)
    }
    fn finish_task(&mut self, id: &str, status: &str) {
        if self.tasks.remove(id) {
            self.await_final_idle |= self.result.is_some();
            self.failed_tasks |= status != "completed";
        }
    }
    pub fn complete(&self) -> bool {
        self.result.is_some()
            && self.tasks.is_empty()
            && !self.injected_turn
            && !self.await_final_idle
            && self.state.as_deref().is_none_or(|s| s == "idle")
    }
}

fn capacity_text(text: &str) -> bool {
    // Text never proves subscription exhaustion. These finite diagnostics only
    // choose Unknown + a bounded recheck, and are never retained in the ledger.
    let lower = text
        .chars()
        .take(4096)
        .collect::<String>()
        .to_ascii_lowercase();
    [
        "rate limit",
        "rate_limit",
        "usage limit",
        "too many requests",
        "overloaded",
        "capacity unavailable",
    ]
    .iter()
    .any(|s| lower.contains(s))
}
#[cfg(test)]
mod tests;
