//! Bounded Claude stream-json values; provider text is never an ownership authority.
use crate::adapter::{AdapterError, AdapterResult, ErrorKind};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

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
    // This is the installed wire/control baseline actually exercised. Future
    // versions require a capability probe, rather than inheriting permissions.
    if version.split_whitespace().next() != Some("2.1.283") {
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
pub(super) fn control_result(message: &Value, id: &str) -> AdapterResult<Value> {
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
    pub fn parse(message: &Value, native: &str) -> AdapterResult<Self> {
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
#[derive(Clone, Default, Debug)]
pub(super) struct Metrics {
    pub input: Option<u64>,
    pub output: Option<u64>,
    pub cache_read: Option<u64>,
    pub cache_write: Option<u64>,
    pub cost: Option<f64>,
    pub api_ms: Option<u64>,
    pub cumulative_cost: Option<f64>,
    pub cumulative_api_ms: Option<u64>,
}
impl Metrics {
    pub fn parse(result: &Value, previous: Option<&Self>, resumed: bool) -> AdapterResult<Self> {
        let counter = |key: &str| -> AdapterResult<Option<u64>> {
            match result.get("usage").and_then(|u| u.get(key)) {
                None | Some(Value::Null) => Ok(None),
                Some(v) => v.as_u64().map(Some).ok_or_else(|| {
                    failure(ErrorKind::ParseFailure, "invalid native token counter")
                }),
            }
        };
        let cost = match result.get("total_cost_usd") {
            None | Some(Value::Null) => None,
            Some(v) => Some(
                v.as_f64()
                    .filter(|n| n.is_finite() && *n >= 0.0)
                    .ok_or_else(|| failure(ErrorKind::ParseFailure, "invalid native cost"))?,
            ),
        };
        let api_ms =
            match result.get("duration_api_ms") {
                None | Some(Value::Null) => None,
                Some(v) => Some(v.as_u64().ok_or_else(|| {
                    failure(ErrorKind::ParseFailure, "invalid native API duration")
                })?),
            };
        // Actual native result.usage is per invocation; native cost/duration
        // gauges span the resumed UUID. Never subtract invocation token counts.
        let delta_cost = if resumed {
            cost.zip(previous.and_then(|p| p.cumulative_cost))
                .and_then(|(a, b)| (a >= b).then_some(a - b))
        } else {
            cost
        };
        let delta_ms = if resumed {
            api_ms
                .zip(previous.and_then(|p| p.cumulative_api_ms))
                .and_then(|(a, b)| a.checked_sub(b))
        } else {
            api_ms
        };
        Ok(Self {
            input: counter("input_tokens")?,
            output: counter("output_tokens")?,
            cache_read: counter("cache_read_input_tokens")?,
            cache_write: counter("cache_creation_input_tokens")?,
            cost: delta_cost,
            api_ms: delta_ms,
            cumulative_cost: cost,
            cumulative_api_ms: api_ms,
        })
    }
}
#[derive(Default)]
pub(super) struct RunState {
    ids: BTreeSet<String>,
    tasks: BTreeSet<String>,
    pub initialized: bool,
    pub result: Option<Value>,
    state: Option<String>,
}
impl RunState {
    pub fn observe(
        &mut self,
        message: &Value,
        native: &str,
        cwd: &std::path::Path,
        decision: bool,
    ) -> AdapterResult<bool> {
        if let Some(id) = message.get("uuid") {
            let id = bounded_id(id)?;
            if !self.ids.insert(id) {
                return Ok(false);
            }
            if self.ids.len() > 8192 {
                return Err(failure(
                    ErrorKind::ParseFailure,
                    "native message count bound exceeded",
                ));
            }
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
                }
                Some("task_started")
                    if matches!(
                        message["task_type"].as_str(),
                        Some("local_agent" | "local_workflow")
                    ) =>
                {
                    self.tasks.insert(bounded_id(&message["task_id"])?);
                    if self.tasks.len() > 64 {
                        return Err(failure(
                            ErrorKind::ParseFailure,
                            "native agent task bound exceeded",
                        ));
                    }
                }
                Some("task_notification") => {
                    self.tasks.remove(message["task_id"].as_str().unwrap_or(""));
                }
                Some("task_updated")
                    if matches!(
                        message["patch"]["status"].as_str(),
                        Some("completed" | "failed" | "stopped")
                    ) =>
                {
                    self.tasks.remove(message["task_id"].as_str().unwrap_or(""));
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
                if message["parent_tool_use_id"].is_null() {
                    self.result = None;
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
            }
            Some("result") => {
                if !self.initialized || message["session_id"] != native {
                    return Err(failure(
                        ErrorKind::OwnershipMismatch,
                        "native terminal identity missing",
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
                    return Err(failure(
                        if authentication {
                            ErrorKind::AuthenticationUnavailable
                        } else {
                            ErrorKind::ProcessFailure
                        },
                        "native turn reported an unsuccessful terminal",
                    ));
                }
                self.result = Some(message.clone());
            }
            Some(
                "user" | "stream_event" | "tool_progress" | "tool_use_summary" | "auth_status"
                | "rate_limit_event",
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
    pub fn complete(&self) -> bool {
        self.result.is_some()
            && self.tasks.is_empty()
            && self.state.as_deref().is_none_or(|s| s == "idle")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invocation_tokens_and_resumed_gauges_have_different_authority() {
        let a = Metrics::parse(&json!({"usage":{"input_tokens":2,"output_tokens":13,"cache_creation_input_tokens":11924,"cache_read_input_tokens":531},"total_cost_usd":0.0957662,"duration_api_ms":1585}),None,false).unwrap();
        let b = Metrics::parse(&json!({"usage":{"input_tokens":2,"output_tokens":12,"cache_creation_input_tokens":351,"cache_read_input_tokens":12228},"total_cost_usd":0.1012678,"duration_api_ms":3094}),Some(&a),true).unwrap();
        assert_eq!(
            (b.input, b.output, b.cache_write, b.api_ms),
            (Some(2), Some(12), Some(351), Some(1509))
        );
        assert!((b.cost.unwrap() - 0.0055016).abs() < 1e-10);
        let reset = Metrics::parse(
            &json!({"total_cost_usd":0,"duration_api_ms":1}),
            Some(&b),
            true,
        )
        .unwrap();
        assert!(reset.input.is_none() && reset.cost.is_none() && reset.api_ms.is_none());
    }
    #[test]
    fn exact_native_operation_reply_never_persists_grants() {
        let p = Pending::parse(&json!({"request_id":"req","request":{"subtype":"can_use_tool","tool_use_id":"use","tool_name":"Bash","input":{"command":"pwd"}}}),"native").unwrap();
        assert_eq!(
            p.reply(true)["response"]["response"]["updatedInput"],
            p.input
        );
        assert!(
            p.reply(true)["response"]["response"]
                .get("updatedPermissions")
                .is_none()
        );
        assert!(p.public("native").get("input").is_none());
        let mut other = p.clone();
        other.tool_use_id = "different".into();
        let q = Pending::parse(&json!({"request_id":"req","request":{"subtype":"can_use_tool","tool_use_id":other.tool_use_id,"tool_name":"Bash","input":p.input}}),"native").unwrap();
        assert_ne!(p.hash, q.hash);
    }
    #[test]
    fn background_agent_and_running_state_prevent_early_success() {
        let mut s = RunState::default();
        let path = std::path::Path::new("/owned");
        s.observe(&json!({"type":"system","subtype":"init","session_id":"n","cwd":"/owned","tools":[],"mcp_servers":[]}),"n",path,true).unwrap();
        s.observe(&json!({"type":"system","subtype":"task_started","task_id":"a","task_type":"local_agent"}),"n",path,false).unwrap();
        s.observe(
            &json!({"type":"result","session_id":"n","subtype":"success","is_error":false}),
            "n",
            path,
            false,
        )
        .unwrap();
        assert!(!s.complete());
        s.observe(
            &json!({"type":"system","subtype":"session_state_changed","state":"running"}),
            "n",
            path,
            false,
        )
        .unwrap();
        s.observe(
            &json!({"type":"system","subtype":"task_notification","task_id":"a"}),
            "n",
            path,
            false,
        )
        .unwrap();
        assert!(!s.complete());
        s.observe(
            &json!({"type":"system","subtype":"session_state_changed","state":"idle"}),
            "n",
            path,
            false,
        )
        .unwrap();
        assert!(s.complete());
    }
}
