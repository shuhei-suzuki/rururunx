//! Independently authored finite app-server wire contract (design §7).
//! The catalog is evidence for the separate schema gate, never a wire validator.
use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;
mod types;

pub(crate) const ITEM_TYPES: &[&str] = &[
    "userMessage",
    "hookPrompt",
    "agentMessage",
    "functionCallOutput",
    "plan",
    "reasoning",
    "commandExecution",
    "fileChange",
    "mcpToolCall",
    "dynamicToolCall",
    "collabAgentToolCall",
    "subAgentActivity",
    "webSearch",
    "imageView",
    "sleep",
    "imageGeneration",
    "enteredReviewMode",
    "exitedReviewMode",
    "contextCompaction",
];
pub(crate) const STRICT_NOTIFICATIONS: &[&str] = &[
    "error",
    "thread/started",
    "thread/status/changed",
    "thread/archived",
    "thread/deleted",
    "thread/unarchived",
    "thread/closed",
    "thread/reverted",
    "thread/settings/updated",
    "turn/started",
    "turn/completed",
    "item/started",
    "item/completed",
    "item/agentMessage/delta",
    "item/commandExecution/terminalInteraction",
    "item/fileChange/patchUpdated",
    "serverRequest/resolved",
    "mcpServer/startupStatus/updated",
    "account/updated",
    "account/rateLimits/updated",
    "account/login/completed",
];
pub(crate) fn reserved_method(method: &str) -> bool {
    notification_owner(method).is_some()
        || matches!(
            method,
            "initialized"
                | "initialize"
                | "account/read"
                | "environment/status"
                | "account/rateLimits/read"
                | "config/read"
                | "mcpServerStatus/list"
                | "thread/start"
                | "turn/start"
                | "item/commandExecution/requestApproval"
                | "item/fileChange/requestApproval"
                | "item/tool/requestUserInput"
                | "mcpServer/elicitation/request"
                | "item/permissions/requestApproval"
                | "item/tool/call"
                | "account/chatgptAuthTokens/refresh"
                | "attestation/generate"
                | "currentTime/read"
                | "applyPatchApproval"
                | "execCommandApproval"
        )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum Ownership {
    Global,
    Thread,
    ThreadTurn,
    ThreadOptionalTurn,
    OptionalThread,
    NestedThread,
    NestedTurn,
    Added,
}
pub(crate) fn notification_owner(method: &str) -> Option<Ownership> {
    use Ownership::*;
    Some(match method {
        "error"
        | "thread/tokenUsage/updated"
        | "turn/diff/updated"
        | "turn/plan/updated"
        | "item/started"
        | "item/autoApprovalReview/started"
        | "item/autoApprovalReview/completed"
        | "autoApprovalReview/strictReviewRequired"
        | "item/completed"
        | "item/agentMessage/delta"
        | "item/plan/delta"
        | "item/commandExecution/outputDelta"
        | "item/commandExecution/terminalInteraction"
        | "item/fileChange/outputDelta"
        | "item/fileChange/patchUpdated"
        | "item/mcpToolCall/progress"
        | "item/reasoning/summaryTextDelta"
        | "item/reasoning/summaryPartAdded"
        | "item/reasoning/textDelta"
        | "thread/compacted"
        | "model/rerouted"
        | "model/verification"
        | "modelProvider/authRecoveryStarted"
        | "modelProvider/authRecoveryCompleted"
        | "turn/moderationMetadata"
        | "model/safetyBuffering/updated" => ThreadTurn,
        "thread/status/changed"
        | "thread/archived"
        | "thread/deleted"
        | "thread/unarchived"
        | "thread/closed"
        | "thread/reverted"
        | "thread/name/updated"
        | "thread/attachment/updated"
        | "thread/prediction/updated"
        | "thread/goal/cleared"
        | "thread/queue/changed"
        | "thread/project/updated"
        | "thread/environment/connected"
        | "thread/environment/disconnected"
        | "thread/settings/updated"
        | "serverRequest/resolved"
        | "guardianWarning"
        | "thread/realtime/started"
        | "thread/realtime/itemAdded"
        | "thread/realtime/item/started"
        | "thread/realtime/item/transcript/delta"
        | "thread/realtime/item/completed"
        | "thread/realtime/transcript/delta"
        | "thread/realtime/transcript/done"
        | "thread/realtime/outputAudio/delta"
        | "thread/realtime/sdp"
        | "thread/realtime/error"
        | "thread/realtime/closed" => Thread,
        "thread/goal/updated" | "hook/started" | "hook/completed" => ThreadOptionalTurn,
        "thread/started" => NestedThread,
        "turn/started" | "turn/completed" => NestedTurn,
        "mcpServer/oauthLogin/completed" | "mcpServer/startupStatus/updated" | "warning" => {
            OptionalThread
        }
        "skills/changed"
        | "project/changed"
        | "command/exec/outputDelta"
        | "process/outputDelta"
        | "process/exited"
        | "mcpServer/event/stream/notification"
        | "account/updated"
        | "account/gatewayOAuth/changed"
        | "account/rateLimits/updated"
        | "app/list/updated"
        | "remoteControl/status/changed"
        | "externalAgentConfig/import/progress"
        | "externalAgentConfig/import/completed"
        | "fs/changed"
        | "deprecationNotice"
        | "configWarning"
        | "fuzzyFileSearch/sessionUpdated"
        | "fuzzyFileSearch/sessionCompleted"
        | "windows/worldWritableWarning"
        | "windowsSandbox/setupCompleted"
        | "account/login/completed" => Global,
        _ => return None,
    })
}

/// Only a successful gate can extend this finite, memory-only contract.
#[derive(Debug, Default, Clone)]
pub(crate) struct CodexWireContract {
    sending: bool,
    pub(in crate::execution) items: Vec<String>,
    pub(in crate::execution) notifications: BTreeMap<String, AddedNotification>,
}
#[derive(Debug, Clone, serde::Serialize)]
pub(in crate::execution) struct Key {
    pub required: bool,
    pub nullable: bool,
}
#[derive(Debug, Clone, serde::Serialize)]
pub(in crate::execution) struct AddedNotification {
    pub thread: Option<Key>,
    pub turn: Option<Key>,
}
impl AddedNotification {
    fn validate(&self, v: &Value) -> Result<()> {
        for (name, key) in [("threadId", &self.thread), ("turnId", &self.turn)] {
            if let Some(key) = key {
                match v.get(name) {
                    None => ensure!(!key.required, "required additional ownership key absent"),
                    Some(Value::Null) => {
                        ensure!(key.nullable, "additional ownership key not nullable")
                    }
                    Some(value) => check("Id", value, &CodexWireContract::default())?,
                }
            }
        }
        Ok(())
    }
}

fn id(v: &Value) -> Result<()> {
    ensure!(
        v.as_i64().is_some()
            || v.as_str()
                .is_some_and(|s| !s.is_empty() && s.len() <= 256 && !s.contains('\0')),
        "invalid Codex identity"
    );
    Ok(())
}
fn record(v: &Value, fields: &str, closed: bool, contract: &CodexWireContract) -> Result<()> {
    let object = v.as_object().context("Codex object required")?;
    for field in fields.split_whitespace() {
        let (key, ty) = field
            .split_once(':')
            .context("internal Codex field contract")?;
        let (key, optional) = key.strip_prefix('?').map_or((key, false), |k| (k, true));
        match object.get(key) {
            None => ensure!(optional, "required Codex field missing"),
            Some(value) => check(ty, value, contract)?,
        }
    }
    if closed || contract.sending {
        ensure!(
            object.keys().all(|k| fields.split_whitespace().any(|f| f
                .split_once(':')
                .unwrap()
                .0
                .trim_start_matches('?')
                == k)),
            "closed Codex object extended"
        );
    }
    Ok(())
}
fn check(ty: &str, value: &Value, c: &CodexWireContract) -> Result<()> {
    if let Some(t) = ty.strip_suffix('~') {
        return if value.is_null() {
            Ok(())
        } else {
            check(t, value, c)
        };
    }
    if let Some(t) = ty.strip_suffix("[]") {
        for v in value.as_array().context("Codex array required")? {
            check(t, v, c)?;
        }
        return Ok(());
    }
    if let Some(t) = ty.strip_suffix("{}") {
        for v in value.as_object().context("Codex map required")?.values() {
            check(t, v, c)?;
        }
        return Ok(());
    }
    if ty.contains('|') || ty.starts_with('=') {
        ensure!(
            value
                .as_str()
                .is_some_and(|s| ty.trim_start_matches('=').split('|').any(|t| t == s)),
            "unknown Codex enum"
        );
        return Ok(());
    }
    let valid = match ty {
        "s" => value.is_string(),
        "b" => value.is_boolean(),
        "i" => value.as_i64().is_some(),
        "u" => value.as_u64().is_some(),
        "u32" => value.as_u64().is_some_and(|n| u32::try_from(n).is_ok()),
        "u16" => value.as_u64().is_some_and(|n| u16::try_from(n).is_ok()),
        "i32" => value.as_i64().is_some_and(|n| i32::try_from(n).is_ok()),
        "n" => value.is_number(),
        "opaque" => true,
        "RpcId" => return id(value),
        "Id" => value
            .as_str()
            .is_some_and(|s| !s.is_empty() && s.len() <= 256 && !s.contains('\0')),
        "Nonempty" => value.as_str().is_some_and(|s| !s.is_empty()),
        _ => return types::check(ty, value, c),
    };
    ensure!(valid, "Codex field type or domain mismatch");
    Ok(())
}

impl CodexWireContract {
    pub(crate) fn decode(&self, bytes: &[u8]) -> Result<Frame> {
        let value = super::strict_json::decode(bytes, super::strict_json::Limits::CEILINGS)?;
        self.frame(value)
    }
    pub(crate) fn frame(&self, value: Value) -> Result<Frame> {
        let o = value.as_object().context("Codex RPC object required")?;
        ensure!(
            o.get("jsonrpc").is_none_or(|v| v == "2.0"),
            "invalid RPC version"
        );
        if let Some(v) = o.get("id") {
            id(v)?;
        }
        if let Some(method) = o.get("method") {
            ensure!(
                !o.contains_key("result") && !o.contains_key("error"),
                "ambiguous Codex RPC"
            );
            let method = method.as_str().context("invalid Codex method")?;
            let params = o.get("params").context("Codex params absent")?;
            if o.contains_key("id") {
                self.server_request(method, params)?;
                return Ok(Frame {
                    value,
                    owner: None,
                    thread: None,
                    turn: None,
                });
            }
            let owner = if let Some(owner) = notification_owner(method) {
                owner
            } else {
                self.notifications
                    .get(method)
                    .context("unknown Codex notification")?
                    .validate(params)?;
                Ownership::Added
            };
            self.notification(method, params, owner)?;
            check_optional(o.get("emittedAtMs"), "i", self)?;
            let text = |v: &Value| v.as_str().map(str::to_owned);
            let thread = match owner {
                Ownership::Global => None,
                Ownership::NestedThread => text(&params["thread"]["id"]),
                Ownership::Added if self.notifications[method].thread.is_none() => None,
                _ => text(&params["threadId"]),
            };
            let turn = match owner {
                Ownership::ThreadTurn | Ownership::ThreadOptionalTurn => text(&params["turnId"]),
                Ownership::NestedTurn => text(&params["turn"]["id"]),
                Ownership::Added if self.notifications[method].turn.is_some() => {
                    text(&params["turnId"])
                }
                _ => None,
            };
            Ok(Frame {
                value,
                owner: Some(owner),
                thread,
                turn,
            })
        } else {
            ensure!(
                o.contains_key("id")
                    && !o.contains_key("params")
                    && o.contains_key("result") != o.contains_key("error"),
                "ambiguous Codex response"
            );
            if let Some(error) = o.get("error") {
                record(error, "code:i message:s ?data:opaque", false, self)?;
            }
            Ok(Frame {
                value,
                owner: None,
                thread: None,
                turn: None,
            })
        }
    }
    pub(crate) fn sent(&self, v: &Value) -> Result<()> {
        let o = v.as_object().context("outgoing Codex object required")?;
        ensure!(
            !o.contains_key("result") && !o.contains_key("error"),
            "invalid outgoing Codex RPC"
        );
        let method = v["method"]
            .as_str()
            .context("outgoing Codex method required")?;
        ensure!(
            o.keys()
                .all(|k| matches!(k.as_str(), "id" | "method" | "params")),
            "undeclared outgoing RPC field"
        );
        if method == "initialized" {
            ensure!(
                !o.contains_key("id") && !o.contains_key("params"),
                "initialized must have no params"
            );
            return Ok(());
        }
        id(o.get("id").context("outgoing Codex id required")?)?;
        let ty = match method {
            "initialize" => "InitializeParams",
            "account/read" => "GetAccountParams",
            "environment/status" => "EnvironmentStatusParams",
            "account/rateLimits/read" => "GetAccountRateLimitsParams",
            "config/read" => "ConfigReadParams",
            "mcpServerStatus/list" => "ListMcpServerStatusParams",
            "thread/start" => "ThreadStartParams",
            "turn/start" => "TurnStartParams",
            _ => bail!("unknown outgoing Codex method"),
        };
        if method == "account/rateLimits/read" && o.get("params").is_none_or(Value::is_null) {
            return Ok(());
        }
        let mut sending = self.clone();
        sending.sending = true;
        check(
            ty,
            o.get("params").context("outgoing Codex params absent")?,
            &sending,
        )
    }
    pub(crate) fn response(&self, method: &str, v: &Value) -> Result<()> {
        check(
            match method {
                "initialize" => "InitializeResponse",
                "account/read" => "GetAccountResponse",
                "environment/status" => "EnvironmentStatusResponse",
                "account/rateLimits/read" => "GetAccountRateLimitsResponse",
                "config/read" => "ConfigReadResponse",
                "mcpServerStatus/list" => "ListMcpServerStatusResponse",
                "thread/start" => "ThreadStartResponse",
                "turn/start" => "TurnStartResponse",
                _ => bail!("unknown Codex response contract"),
            },
            v,
            self,
        )
    }
    fn server_request(&self, method: &str, v: &Value) -> Result<()> {
        check(
            match method {
                "item/commandExecution/requestApproval" => "CommandExecutionRequestApprovalParams",
                "item/fileChange/requestApproval" => "FileChangeRequestApprovalParams",
                "item/tool/requestUserInput" => "ToolRequestUserInputParams",
                "mcpServer/elicitation/request" => "McpServerElicitationRequestParams",
                "item/permissions/requestApproval" => "PermissionsRequestApprovalParams",
                "item/tool/call" => "DynamicToolCallParams",
                "account/chatgptAuthTokens/refresh" => "ChatgptAuthTokensRefreshParams",
                "attestation/generate" => "AttestationGenerateParams",
                "currentTime/read" => "CurrentTimeReadParams",
                "applyPatchApproval" => "ApplyPatchApprovalParams",
                "execCommandApproval" => "ExecCommandApprovalParams",
                _ => bail!("unknown Codex server request"),
            },
            v,
            self,
        )
    }
    fn notification(&self, method: &str, v: &Value, owner: Ownership) -> Result<()> {
        use Ownership::*;
        record(
            v,
            match owner {
                Global | Added => "",
                Thread => "threadId:Id",
                ThreadTurn => "threadId:Id turnId:Id",
                ThreadOptionalTurn => "threadId:Id ?turnId:Id~",
                OptionalThread => "?threadId:Id~",
                NestedThread => "thread:Thread",
                NestedTurn => "threadId:Id turn:Turn",
            },
            false,
            self,
        )?;
        if !STRICT_NOTIFICATIONS.contains(&method) {
            return Ok(());
        }
        let fields = match method {
            "error" => "error:TurnError willRetry:b",
            "thread/status/changed" => "status:ThreadStatus",
            "thread/settings/updated" => "threadSettings:ThreadSettings",
            "item/started" => "item:ThreadItem startedAtMs:i",
            "item/completed" => "item:ThreadItem completedAtMs:i",
            "item/agentMessage/delta" => "itemId:Id delta:s",
            "item/commandExecution/terminalInteraction" => "itemId:Id processId:s stdin:s",
            "item/fileChange/patchUpdated" => "itemId:Id changes:FileUpdateChange[]",
            "serverRequest/resolved" => "requestId:RpcId",
            "mcpServer/startupStatus/updated" => {
                "name:s status:starting|ready|failed|cancelled ?error:s~ ?failureReason:McpServerStartupFailureReason~"
            }
            "account/updated" => "?authMode:AuthMode~ ?planType:PlanType~",
            "account/rateLimits/updated" => "rateLimits:RateSnapshot",
            "account/login/completed" => {
                "success:b ?error:s~ ?loginId:s~ ?onboardingEntrypoint:DesktopOnboardingEntrypoint~"
            }
            _ => "",
        };
        record(v, fields, false, self)
    }
}
fn check_optional(v: Option<&Value>, ty: &str, c: &CodexWireContract) -> Result<()> {
    v.map_or(Ok(()), |v| check(ty, v, c))
}

/// Validated frame. Consumers can request only typed strict payloads.
pub(crate) struct Frame {
    value: Value,
    owner: Option<Ownership>,
    thread: Option<String>,
    turn: Option<String>,
}
impl Frame {
    pub(crate) fn value(&self) -> &Value {
        &self.value
    }
    pub(crate) fn handshake_notification(&self) -> Result<()> {
        let v = &self.value;
        ensure!(
            self.owner.is_some(),
            "server request or unexpected response in handshake"
        );
        ensure!(
            matches!(self.owner, Some(Ownership::Global))
                || matches!(self.owner, Some(Ownership::OptionalThread)) && self.thread.is_none()
                || matches!(self.owner, Some(Ownership::Added))
                    && self.thread.is_none()
                    && self.turn.is_none(),
            "thread notification in handshake"
        );
        let p = &v["params"];
        if v["method"] == "account/updated" {
            ensure!(
                p.get("authMode").is_none_or(|v| v == "chatgpt"),
                "contradictory account notification"
            );
        }
        if v["method"] == "account/login/completed" {
            ensure!(p["success"] == true, "contradictory login notification");
        }
        if v["method"] == "account/rateLimits/updated" {
            super::quota::CodexRate::decode(p)?;
        }
        Ok(())
    }
    pub(crate) fn foreign(&self, thread: &str, turn: &str) -> bool {
        self.thread.as_deref().is_some_and(|id| id != thread)
            || self.turn.as_deref().is_some_and(|id| id != turn)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Item<'a>(&'a Value);
impl<'a> Item<'a> {
    pub(crate) fn decode(v: &'a Value, c: &CodexWireContract) -> Result<Self> {
        check("ThreadItem", v, c)?;
        Ok(Self(v))
    }
    pub(crate) fn value(self) -> &'a Value {
        self.0
    }
    pub(crate) fn permitted(self) -> bool {
        matches!(
            self.0["type"].as_str(),
            Some(
                "userMessage"
                    | "agentMessage"
                    | "reasoning"
                    | "plan"
                    | "commandExecution"
                    | "fileChange"
                    | "contextCompaction"
            )
        )
    }
}
pub(crate) struct Turn<'a>(&'a Value);
impl<'a> Turn<'a> {
    pub(crate) fn decode(v: &'a Value, c: &CodexWireContract) -> Result<Self> {
        check("Turn", v, c)?;
        Ok(Self(v))
    }
    pub(crate) fn value(&self) -> &'a Value {
        self.0
    }
    pub(crate) fn permitted(&self) -> bool {
        self.0["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| Item(v).permitted())
    }
}
pub(crate) struct Delta<'a>(&'a Value);
impl<'a> Delta<'a> {
    pub(crate) fn decode(v: &'a Value, c: &CodexWireContract) -> Result<Self> {
        record(v, "threadId:Id turnId:Id itemId:Id delta:s", false, c)?;
        Ok(Self(v))
    }
    pub(crate) fn value(&self) -> &'a Value {
        self.0
    }
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CodexErrorKind {
    Subscription,
    Authentication,
    Capacity,
    Other,
    Unclassified,
}
pub(crate) fn error_kind(value: &Value) -> Result<CodexErrorKind> {
    types::error_kind(value)
}

/// Only judgement paths survive this projection; Opaque config is never exposed.
pub(crate) struct ConfigJudgement {
    web_search: Option<String>,
    features: BTreeMap<String, bool>,
    servers: BTreeMap<String, Option<bool>>,
}
#[derive(Debug)]
pub(crate) struct ExecutorCapabilityUnverified;
impl std::fmt::Display for ExecutorCapabilityUnverified {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ExecutorCapabilityUnverified")
    }
}
impl std::error::Error for ExecutorCapabilityUnverified {}
impl ConfigJudgement {
    pub(crate) fn decode(v: &Value) -> Result<Self> {
        check("Config", v, &CodexWireContract::default())?;
        Ok(Self {
            web_search: v["web_search"].as_str().map(str::to_owned),
            features: v["features"]
                .as_object()
                .map(|m| {
                    m.iter()
                        .map(|(k, v)| (k.clone(), v.as_bool().unwrap()))
                        .collect()
                })
                .unwrap_or_default(),
            servers: v["mcp_servers"]
                .as_object()
                .map(|m| {
                    m.iter()
                        .map(|(k, v)| (k.clone(), v["enabled"].as_bool()))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
    pub(crate) fn restricted<'a, 'b>(
        &self,
        mut features: impl Iterator<Item = &'a str>,
        mut servers: impl Iterator<Item = &'b str>,
    ) -> bool {
        self.web_search.as_deref() == Some("disabled")
            && features.all(|name| self.features.get(name) == Some(&false))
            && servers.all(|name| self.servers.get(name) == Some(&Some(false)))
    }
}

/// S/P handshake state; no thread, model input, authority, or quota pool writes.
pub(crate) struct Handshake {
    contract: CodexWireContract,
    step: usize,
    notifications: usize,
    pub(crate) rate: Option<bool>, // None=not observed, false=empty_valid, true=usable
}
impl Handshake {
    pub(crate) fn new(contract: CodexWireContract) -> Self {
        Self {
            contract,
            step: 0,
            notifications: 0,
            rate: None,
        }
    }
    fn request(&self) -> Value {
        let (method, params) = match self.step {
            0 => (
                "initialize",
                serde_json::json!({"clientInfo":{"name":"rrx","version":env!("CARGO_PKG_VERSION")},"capabilities":{"experimentalApi":true}}),
            ),
            1 => ("account/read", serde_json::json!({"refreshToken":false})),
            2 => (
                "environment/status",
                serde_json::json!({"environmentId":"local"}),
            ),
            _ => ("account/rateLimits/read", serde_json::json!({})),
        };
        serde_json::json!({"id":self.step+1,"method":method,"params":params})
    }
    pub(crate) fn initial(&self) -> Result<Value> {
        let v = self.request();
        self.contract.sent(&v)?;
        Ok(v)
    }
    pub(crate) fn observe(&mut self, bytes: &[u8]) -> Result<Vec<Value>> {
        let frame = self.contract.decode(bytes)?;
        if frame.value.get("method").is_some() {
            frame.handshake_notification()?;
            self.notifications += 1;
            ensure!(
                self.notifications <= 1024,
                "Codex handshake notification bound"
            );
            return Ok(vec![]);
        }
        ensure!(
            self.step < 4
                && frame.value["id"] == self.step + 1
                && frame.value.get("error").is_none(),
            "Codex handshake RPC correlation or error"
        );
        let method = self.request()["method"].as_str().unwrap().to_owned();
        let response = &frame.value["result"];
        self.contract.response(&method, response)?;
        match self.step {
            0 => crate::codex::managed::initialization(response)?,
            1 => ensure!(
                response["account"]["type"] == "chatgpt",
                "Codex ChatGPT account required"
            ),
            2 => ensure!(
                response["status"] == "ready",
                "Codex local environment not ready"
            ),
            3 => {
                self.rate = Some(
                    !super::quota::CodexRate::decode(response)?
                        .observations(0)
                        .is_empty(),
                )
            }
            _ => unreachable!(),
        }
        let mut outgoing = vec![];
        if self.step == 0 {
            outgoing.push(serde_json::json!({"method":"initialized"}));
        }
        self.step += 1;
        if self.step < 4 {
            outgoing.push(self.request());
        }
        for v in &outgoing {
            self.contract.sent(v)?;
        }
        Ok(outgoing)
    }
    pub(crate) fn complete(&self) -> bool {
        self.step == 4
    }
}
