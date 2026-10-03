//! Bounded native JSON-RPC over the app-server's local Unix WebSocket transport.
use std::collections::{BTreeMap, VecDeque};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::net::UnixStream;
use tokio_tungstenite::{
    WebSocketStream,
    tungstenite::{Message, protocol::WebSocketConfig},
};

use crate::adapter::{AdapterError, AdapterResult, ErrorKind};

const MAX_MESSAGE: usize = 4 * 1024 * 1024;
const MAX_OUTGOING: usize = 1024 * 1024;
const MAX_QUEUED: usize = 128;
const MAX_QUEUED_BYTES: usize = 8 * 1024 * 1024;
const RPC_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) fn failure(kind: ErrorKind, message: impl Into<String>) -> AdapterError {
    AdapterError {
        kind,
        message: message.into(),
    }
}

/// Native request IDs are opaque and must be returned without conversion.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RpcId {
    Number(i64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Notification {
        method: String,
        params: Value,
    },
    Request {
        id: RpcId,
        method: String,
        params: Value,
    },
    Response {
        id: RpcId,
        result: Value,
    },
    Error {
        id: RpcId,
        code: i64,
    },
}

impl Event {
    fn parse(bytes: &[u8]) -> AdapterResult<Self> {
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|_| failure(ErrorKind::ParseFailure, "invalid native JSON-RPC"))?;
        let object = value
            .as_object()
            .ok_or_else(|| failure(ErrorKind::ParseFailure, "native JSON-RPC must be an object"))?;
        if object.get("jsonrpc").is_some_and(|v| v != "2.0") {
            return Err(failure(
                ErrorKind::ParseFailure,
                "unsupported JSON-RPC version",
            ));
        }
        let id = object
            .get("id")
            .map(|v| serde_json::from_value::<RpcId>(v.clone()))
            .transpose()
            .map_err(|_| failure(ErrorKind::ParseFailure, "invalid native RPC ID"))?;
        if matches!(&id, Some(RpcId::Text(s)) if s.len() > 256) {
            return Err(failure(ErrorKind::ParseFailure, "oversized native RPC ID"));
        }
        match (
            object.get("method"),
            object.get("result"),
            object.get("error"),
        ) {
            (Some(method), None, None) => {
                let method = method
                    .as_str()
                    .filter(|s| !s.is_empty() && s.len() <= 256)
                    .ok_or_else(|| failure(ErrorKind::ParseFailure, "invalid native method"))?
                    .to_owned();
                let params = object.get("params").cloned().unwrap_or_else(|| json!({}));
                if !params.is_object() {
                    return Err(failure(
                        ErrorKind::ParseFailure,
                        "native params must be an object",
                    ));
                }
                Ok(match id {
                    Some(id) => Self::Request { id, method, params },
                    None => Self::Notification { method, params },
                })
            }
            (None, Some(result), None) => Ok(Self::Response {
                id: id.ok_or_else(|| failure(ErrorKind::ParseFailure, "response has no RPC ID"))?,
                result: result.clone(),
            }),
            (None, None, Some(error)) => Ok(Self::Error {
                id: id.ok_or_else(|| failure(ErrorKind::ParseFailure, "error has no RPC ID"))?,
                code: error
                    .get("code")
                    .and_then(Value::as_i64)
                    .ok_or_else(|| failure(ErrorKind::ParseFailure, "invalid native error"))?,
            }),
            _ => Err(failure(
                ErrorKind::ParseFailure,
                "ambiguous native RPC envelope",
            )),
        }
    }
}

/// The owner verifies socket directory/inode/permissions before passing a stream.
/// This client cannot connect to TCP or to a shared native daemon.
pub struct NativeRpc {
    socket: WebSocketStream<UnixStream>,
    next_id: i64,
    queued: VecDeque<(Event, usize)>,
    queued_bytes: usize,
}
impl NativeRpc {
    pub async fn connect(stream: UnixStream) -> AdapterResult<Self> {
        let config = WebSocketConfig::default()
            .max_message_size(Some(MAX_MESSAGE))
            .max_frame_size(Some(MAX_MESSAGE))
            .write_buffer_size(0)
            .max_write_buffer_size(MAX_MESSAGE + MAX_OUTGOING);
        let (socket, _) = tokio::time::timeout(
            RPC_TIMEOUT,
            tokio_tungstenite::client_async_with_config("ws://localhost/", stream, Some(config)),
        )
        .await
        .map_err(|_| failure(ErrorKind::Timeout, "native WebSocket handshake timed out"))?
        .map_err(|_| failure(ErrorKind::ParseFailure, "native WebSocket handshake failed"))?;
        Ok(Self {
            socket,
            next_id: 1,
            queued: VecDeque::new(),
            queued_bytes: 0,
        })
    }
    pub async fn send(&mut self, value: Value) -> AdapterResult<()> {
        let text = serde_json::to_string(&value)
            .map_err(|_| failure(ErrorKind::InvalidInput, "invalid RPC payload"))?;
        if text.len() > MAX_OUTGOING {
            return Err(failure(
                ErrorKind::InvalidInput,
                "native RPC payload exceeds limit",
            ));
        }
        tokio::time::timeout(RPC_TIMEOUT, self.socket.send(Message::Text(text.into())))
            .await
            .map_err(|_| failure(ErrorKind::Timeout, "native RPC write timed out"))?
            .map_err(|_| failure(ErrorKind::ProcessFailure, "native RPC write failed"))
    }
    async fn read(&mut self) -> AdapterResult<(Event, usize)> {
        loop {
            let message = self
                .socket
                .next()
                .await
                .ok_or_else(|| failure(ErrorKind::ProcessFailure, "native RPC transport closed"))?
                .map_err(|_| {
                    failure(
                        ErrorKind::ParseFailure,
                        "invalid or oversized native WebSocket frame",
                    )
                })?;
            match message {
                Message::Text(text) => return Ok((Event::parse(text.as_bytes())?, text.len())),
                // Native protocol messages are UTF-8 text, never binary JSON.
                Message::Binary(_) => {
                    return Err(failure(
                        ErrorKind::ParseFailure,
                        "unexpected native binary frame",
                    ));
                }
                Message::Close(_) => {
                    return Err(failure(
                        ErrorKind::ProcessFailure,
                        "native RPC transport closed",
                    ));
                }
                Message::Ping(_) => self.socket.flush().await.map_err(|_| {
                    failure(ErrorKind::ProcessFailure, "native control frame failed")
                })?,
                Message::Pong(_) => {}
                Message::Frame(_) => {
                    return Err(failure(
                        ErrorKind::ParseFailure,
                        "unexpected raw native frame",
                    ));
                }
            }
        }
    }
    pub async fn receive(&mut self) -> AdapterResult<Event> {
        if let Some((event, bytes)) = self.queued.pop_front() {
            self.queued_bytes -= bytes;
            return Ok(event);
        }
        Ok(self.read().await?.0)
    }
    fn queue(&mut self, event: Event, bytes: usize) -> AdapterResult<()> {
        if self.queued.len() >= MAX_QUEUED
            || bytes > MAX_QUEUED_BYTES.saturating_sub(self.queued_bytes)
        {
            return Err(failure(
                ErrorKind::ParseFailure,
                "native preflight event queue exceeded limit",
            ));
        }
        self.queued.push_back((event, bytes));
        self.queued_bytes += bytes;
        Ok(())
    }
    /// For bootstrap/lifecycle RPCs. No target-operation approval is supplied here.
    /// Live callbacks are delivered by receive after the native turn is established.
    pub async fn call(&mut self, method: &str, params: Value) -> AdapterResult<Value> {
        let id = RpcId::Number(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| failure(ErrorKind::ProcessFailure, "native RPC ID space exhausted"))?;
        self.send(json!({"id":id,"method":method,"params":params}))
            .await?;
        let deadline = tokio::time::Instant::now() + RPC_TIMEOUT;
        loop {
            let (event, bytes) = tokio::time::timeout_at(deadline, self.read())
                .await
                .map_err(|_| failure(ErrorKind::Timeout, "native RPC response timed out"))??;
            match event {
                Event::Response { id: got, result } if got == id => return Ok(result),
                Event::Error { id: got, code } if got == id => {
                    return Err(failure(
                        ErrorKind::ProcessFailure,
                        format!("native RPC {method} rejected (code {code})"),
                    ));
                }
                Event::Response { .. } | Event::Error { .. } => {
                    return Err(failure(
                        ErrorKind::ParseFailure,
                        "foreign or replayed native response ID",
                    ));
                }
                Event::Request { id, .. } => {
                    self.send(json!({"id":id,"error":{"code":-32601,"message":"Operation approval unavailable during runtime preflight"}})).await?;
                    return Err(failure(
                        ErrorKind::UnsupportedCapability,
                        "native callback requires an established scoped turn",
                    ));
                }
                event => self.queue(event, bytes)?,
            }
        }
    }
    pub async fn initialize(&mut self) -> AdapterResult<()> {
        self.call("initialize", json!({"clientInfo":{"name":"rururunx","title":"rururunx scoped native session","version":env!("CARGO_PKG_VERSION")},"capabilities":{"experimentalApi":true}})).await?;
        self.send(json!({"method":"initialized","params":{}})).await
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PendingRequest {
    pub id: RpcId,
    pub method: String,
    pub params: Value,
}

/// A reply can grant only the exact already pending operation once.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum OperationDecision {
    Approve,
    Deny,
    Cancel,
}

pub struct ApprovalLedger {
    thread: String,
    turn: String,
    pending: BTreeMap<RpcId, PendingRequest>,
    seen: BTreeMap<RpcId, ()>,
}
impl ApprovalLedger {
    pub fn new(thread: String, turn: String) -> Self {
        Self {
            thread,
            turn,
            pending: BTreeMap::new(),
            seen: BTreeMap::new(),
        }
    }
    pub fn insert(
        &mut self,
        id: RpcId,
        method: String,
        params: Value,
    ) -> AdapterResult<PendingRequest> {
        if matches!(&id, RpcId::Text(value) if value.is_empty() || value.len() > 256)
            || serde_json::to_vec(&params).map_or(true, |value| value.len() > 64 * 1024)
        {
            return Err(failure(
                ErrorKind::ParseFailure,
                "native approval exceeds correlation/argument bounds",
            ));
        }
        if params["threadId"].as_str() != Some(self.thread.as_str())
            || params["turnId"].as_str() != Some(self.turn.as_str())
            || params["itemId"]
                .as_str()
                .is_none_or(|value| value.is_empty() || value.len() > 256)
        {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "foreign native approval thread/turn/item",
            ));
        }
        if !matches!(
            method.as_str(),
            "item/commandExecution/requestApproval" | "item/fileChange/requestApproval"
        ) {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "unsupported native approval kind; no grant supplied",
            ));
        }
        let keys = if method == "item/commandExecution/requestApproval" {
            &[
                "threadId",
                "turnId",
                "itemId",
                "startedAtMs",
                "reason",
                "command",
                "cwd",
                "commandActions",
                "approvalId",
                "availableDecisions",
                "kind",
                "environmentId",
                "additionalPermissions",
                "networkApprovalContext",
                "proposedExecpolicyAmendment",
                "proposedNetworkPolicyAmendments",
            ][..]
        } else {
            &[
                "threadId",
                "turnId",
                "itemId",
                "startedAtMs",
                "reason",
                "grantRoot",
            ][..]
        };
        if params
            .as_object()
            .is_none_or(|object| object.keys().any(|key| !keys.contains(&key.as_str())))
            || [
                "grantRoot",
                "additionalPermissions",
                "networkApprovalContext",
            ]
            .iter()
            .any(|key| params.get(*key).is_some_and(|value| !value.is_null()))
            || params.get("kind").is_some_and(|value| value != "command")
            || params
                .get("environmentId")
                .is_some_and(|value| !value.is_null() && value != "local")
        {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "native approval requires an unsupported permission, persistent root, terminal input or remote environment; no grant supplied",
            ));
        }
        if self.seen.contains_key(&id) || self.pending.len() >= 16 || self.seen.len() >= 4096 {
            return Err(failure(
                ErrorKind::ParseFailure,
                "replayed or excessive native approval request",
            ));
        }
        let request = PendingRequest {
            id: id.clone(),
            method,
            params,
        };
        self.seen.insert(id.clone(), ());
        self.pending.insert(id, request.clone());
        Ok(request)
    }
    pub fn reply(&mut self, id: &RpcId, decision: OperationDecision) -> AdapterResult<Value> {
        let request = self.pending.get(id).ok_or_else(|| {
            failure(
                ErrorKind::OwnershipMismatch,
                "unknown or already answered native approval ID",
            )
        })?;
        if matches!(decision, OperationDecision::Approve)
            && let Some(decisions) = request
                .params
                .get("availableDecisions")
                .filter(|value| !value.is_null())
            && decisions
                .as_array()
                .is_none_or(|values| !values.iter().any(|value| value == "accept"))
        {
            return Err(failure(
                ErrorKind::UnsupportedCapability,
                "native request does not offer one-operation approval",
            ));
        }
        let request = self
            .pending
            .remove(id)
            .expect("pending native request checked");
        let decision = match decision {
            OperationDecision::Approve => "accept",
            OperationDecision::Deny => "decline",
            OperationDecision::Cancel => "cancel",
        };
        Ok(json!({"id":request.id,"result":{"decision":decision}}))
    }
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
    pub fn pending(&self) -> Vec<PendingRequest> {
        self.pending.values().cloned().collect()
    }
    pub fn preview_reply(&self, id: &RpcId, decision: OperationDecision) -> AdapterResult<Value> {
        let mut preview = Self {
            thread: self.thread.clone(),
            turn: self.turn.clone(),
            pending: self
                .pending
                .get(id)
                .map(|request| BTreeMap::from([(id.clone(), request.clone())]))
                .unwrap_or_default(),
            seen: BTreeMap::new(),
        };
        preview.reply(id, decision)
    }
}

/// Native cumulative counters are gauges, not deltas to add on each notification.
#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct TokenCounters {
    pub input: Option<u64>,
    pub cached_input: Option<u64>,
    pub cache_write_input: Option<u64>,
    pub output: Option<u64>,
    pub reasoning_output: Option<u64>,
    pub total: Option<u64>,
}
impl TokenCounters {
    fn fields(&self) -> [Option<u64>; 6] {
        [
            self.input,
            self.cached_input,
            self.cache_write_input,
            self.output,
            self.reasoning_output,
            self.total,
        ]
    }
    fn from_fields(fields: [Option<u64>; 6]) -> Self {
        let [
            input,
            cached_input,
            cache_write_input,
            output,
            reasoning_output,
            total,
        ] = fields;
        Self {
            input,
            cached_input,
            cache_write_input,
            output,
            reasoning_output,
            total,
        }
    }
    pub fn from_native(value: &Value) -> AdapterResult<Self> {
        let field = |name: &str| match value.get(name) {
            None | Some(Value::Null) => Ok(None),
            Some(v) => v
                .as_u64()
                .map(Some)
                .ok_or_else(|| failure(ErrorKind::ParseFailure, "invalid native token counter")),
        };
        if !value.is_object() {
            return Err(failure(
                ErrorKind::ParseFailure,
                "invalid native token usage",
            ));
        }
        Ok(Self {
            input: field("inputTokens")?,
            cached_input: field("cachedInputTokens")?,
            cache_write_input: field("cacheWriteInputTokens")?,
            output: field("outputTokens")?,
            reasoning_output: field("reasoningOutputTokens")?,
            total: field("totalTokens")?,
        })
    }
}

/// Replace cumulative gauges and emit only changed scoped observations. The
/// runtime persists these snapshots by native thread/turn rather than summing
/// repeated cumulative notifications (including notifications after resume).
#[derive(Debug, Clone)]
pub struct UsageTracker {
    thread: String,
    turn: String,
    pub total: Option<TokenCounters>,
    pub last: Option<TokenCounters>,
    // None denotes a newly created native thread, Some(None) an owned resume
    // whose previous cumulative gauge was unavailable. Never substitute zero
    // for missing native counters on resume.
    baseline: Option<Option<TokenCounters>>,
    valid: [bool; 6],
}
impl UsageTracker {
    pub fn new(thread: String, turn: String) -> Self {
        Self {
            thread,
            turn,
            total: None,
            last: None,
            baseline: None,
            valid: [true; 6],
        }
    }
    pub fn resumed(thread: String, turn: String, baseline: Option<TokenCounters>) -> Self {
        Self {
            baseline: Some(baseline),
            ..Self::new(thread, turn)
        }
    }
    /// A replayed owned prior turn can show that our shutdown-time gauge lagged.
    /// Never attribute the difference to the newly resumed turn. Once uncertain,
    /// this baseline stays unknown for the whole current turn.
    pub fn verify_previous_total(&mut self, previous: &TokenCounters) {
        if self
            .baseline
            .as_ref()
            .is_some_and(|baseline| baseline.as_ref() != Some(previous))
        {
            self.baseline = Some(None);
        }
    }
    /// Entire current turn, including every native model call and tool cycle.
    /// `last` is a single model call and cannot represent a multi-call turn.
    pub fn turn_counters(&self) -> Option<TokenCounters> {
        let current = self.total.as_ref()?.fields();
        let baseline = self.baseline.as_ref().map(|prior| {
            prior
                .as_ref()
                .map(TokenCounters::fields)
                .unwrap_or([None; 6])
        });
        Some(TokenCounters::from_fields(std::array::from_fn(|index| {
            if !self.valid[index] {
                return None;
            }
            let value = current[index]?;
            match baseline {
                None => Some(value),
                Some(prior) => value.checked_sub(prior[index]?),
            }
        })))
    }
    pub fn update(&mut self, params: &Value) -> AdapterResult<bool> {
        if params["threadId"].as_str() != Some(self.thread.as_str())
            || params["turnId"].as_str() != Some(self.turn.as_str())
        {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "foreign or stale native token usage",
            ));
        }
        let total = TokenCounters::from_native(&params["tokenUsage"]["total"])?;
        let last = TokenCounters::from_native(&params["tokenUsage"]["last"])?;
        if self.total.as_ref() == Some(&total) && self.last.as_ref() == Some(&last) {
            return Ok(false);
        }
        if let Some(previous) = &self.total {
            for (index, (before, after)) in previous
                .fields()
                .into_iter()
                .zip(total.fields())
                .enumerate()
            {
                if matches!((before, after), (Some(before), Some(after)) if after < before) {
                    self.valid[index] = false;
                }
            }
        }
        if let Some(Some(baseline)) = &self.baseline {
            for (index, (before, after)) in baseline
                .fields()
                .into_iter()
                .zip(total.fields())
                .enumerate()
            {
                if matches!((before, after), (Some(before), Some(after)) if after < before) {
                    self.valid[index] = false;
                }
            }
        }
        self.total = Some(total);
        self.last = Some(last);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::accept_async;

    fn request() -> Value {
        json!({"threadId":"own-thread","turnId":"own-turn","itemId":"item-1","command":"supplied exact operation"})
    }

    #[test]
    fn approval_arguments_cannot_expand_permissions_or_adopt_remote_terminal_operations() {
        for (method, field, value) in [
            (
                "item/fileChange/requestApproval",
                "grantRoot",
                json!("/other-project"),
            ),
            (
                "item/commandExecution/requestApproval",
                "additionalPermissions",
                json!({"fileSystem":{"write":["/other-project"]}}),
            ),
            (
                "item/commandExecution/requestApproval",
                "networkApprovalContext",
                json!({"host":"foreign.invalid","protocol":"https"}),
            ),
            (
                "item/commandExecution/requestApproval",
                "kind",
                json!("writeStdin"),
            ),
            (
                "item/commandExecution/requestApproval",
                "environmentId",
                json!("remote"),
            ),
            (
                "item/commandExecution/requestApproval",
                "newPermissionGrant",
                json!({"all":true}),
            ),
        ] {
            let mut ledger = ApprovalLedger::new("own-thread".into(), "own-turn".into());
            let mut params = json!({"threadId":"own-thread","turnId":"own-turn","itemId":"item-1"});
            params[field] = value;
            assert_eq!(
                ledger
                    .insert(RpcId::Number(1), method.into(), params)
                    .unwrap_err()
                    .kind,
                ErrorKind::UnsupportedCapability
            );
            assert!(ledger.is_empty());
            assert!(
                ledger
                    .reply(&RpcId::Number(1), OperationDecision::Approve)
                    .is_err()
            );
        }
    }

    #[test]
    fn persistent_only_prompt_can_be_denied_but_never_converted_to_operation_approval() {
        let mut ledger = ApprovalLedger::new("own-thread".into(), "own-turn".into());
        let mut params = request();
        params["availableDecisions"] = json!(["acceptForSession", "decline"]);
        ledger
            .insert(
                RpcId::Number(1),
                "item/commandExecution/requestApproval".into(),
                params,
            )
            .unwrap();
        assert_eq!(
            ledger
                .reply(&RpcId::Number(1), OperationDecision::Approve)
                .unwrap_err()
                .kind,
            ErrorKind::UnsupportedCapability
        );
        assert_eq!(
            ledger
                .reply(&RpcId::Number(1), OperationDecision::Deny)
                .unwrap(),
            json!({"id":1,"result":{"decision":"decline"}})
        );
        let mut params = request();
        params["availableDecisions"] = json!(["accept", "acceptForSession"]);
        params["proposedExecpolicyAmendment"] = json!(["native", "proposal"]);
        ledger
            .insert(
                RpcId::Number(2),
                "item/commandExecution/requestApproval".into(),
                params.clone(),
            )
            .unwrap();
        assert_eq!(
            ledger
                .reply(&RpcId::Number(2), OperationDecision::Approve)
                .unwrap(),
            json!({"id":2,"result":{"decision":"accept"}})
        );
    }

    #[test]
    fn oversized_native_callback_ids_and_arguments_never_enter_the_pending_map() {
        for (id, params) in [
            (RpcId::Text("x".repeat(257)), request()),
            (RpcId::Number(1), {
                let mut params = request();
                params["command"] = json!("x".repeat(65536));
                params
            }),
        ] {
            let mut ledger = ApprovalLedger::new("own-thread".into(), "own-turn".into());
            assert_eq!(
                ledger
                    .insert(id, "item/commandExecution/requestApproval".into(), params)
                    .unwrap_err()
                    .kind,
                ErrorKind::ParseFailure
            );
            assert!(ledger.is_empty());
        }
    }

    fn usage(total: Value, last: Value) -> Value {
        json!({"threadId":"thread","turnId":"turn","tokenUsage":{"total":total,"last":last}})
    }

    #[test]
    fn whole_turn_counts_every_model_call_without_summing_duplicate_gauges() {
        let mut tracker = UsageTracker::new("thread".into(), "turn".into());
        let first = usage(
            json!({"inputTokens":100,"outputTokens":10,"cachedInputTokens":0}),
            json!({"inputTokens":100,"outputTokens":10}),
        );
        tracker.update(&first).unwrap();
        let next = usage(
            json!({"inputTokens":250,"outputTokens":35,"cachedInputTokens":80}),
            json!({"inputTokens":150,"outputTokens":25}),
        );
        assert!(tracker.update(&next).unwrap());
        assert!(!tracker.update(&next).unwrap());
        let counters = tracker.turn_counters().unwrap();
        assert_eq!(counters.input, Some(250));
        assert_eq!(counters.output, Some(35));
        assert_eq!(counters.cached_input, Some(80));
        assert_eq!(counters.cache_write_input, None);
        assert_eq!(tracker.last.unwrap().input, Some(150));
    }

    #[test]
    fn resumed_turn_subtracts_owned_baseline_preserving_unknown_and_native_zero() {
        let baseline = TokenCounters::from_native(
            &json!({"inputTokens":250,"outputTokens":35,"cachedInputTokens":80}),
        )
        .unwrap();
        let mut tracker = UsageTracker::resumed("thread".into(), "turn".into(), Some(baseline));
        let next = usage(
            json!({"inputTokens":400,"outputTokens":60,"cachedInputTokens":80,"cacheWriteInputTokens":20}),
            json!({"inputTokens":75,"outputTokens":12}),
        );
        tracker.update(&next).unwrap();
        let counters = tracker.turn_counters().unwrap();
        assert_eq!(counters.input, Some(150));
        assert_eq!(counters.output, Some(25));
        assert_eq!(counters.cached_input, Some(0));
        assert_eq!(counters.cache_write_input, None);
        let mut unknown = UsageTracker::resumed("thread".into(), "turn".into(), None);
        unknown.update(&next).unwrap();
        assert_eq!(unknown.turn_counters().unwrap(), TokenCounters::default());
    }

    #[test]
    fn native_counter_reset_stays_unknown_even_after_the_counter_recovers() {
        let baseline =
            TokenCounters::from_native(&json!({"inputTokens":100,"outputTokens":20})).unwrap();
        let mut tracker = UsageTracker::resumed("thread".into(), "turn".into(), Some(baseline));
        tracker
            .update(&usage(
                json!({"inputTokens":90,"outputTokens":25}),
                json!({}),
            ))
            .unwrap();
        tracker
            .update(&usage(
                json!({"inputTokens":150,"outputTokens":30}),
                json!({}),
            ))
            .unwrap();
        assert_eq!(tracker.turn_counters().unwrap().input, None);
        assert_eq!(tracker.turn_counters().unwrap().output, Some(10));
        let mut fresh = UsageTracker::new("thread".into(), "turn".into());
        for input in [100, 80, 130] {
            fresh
                .update(&usage(json!({"inputTokens":input}), json!({})))
                .unwrap();
        }
        assert_eq!(fresh.turn_counters().unwrap().input, None);
    }

    #[test]
    fn approvals_are_scoped_one_time_and_cannot_create_persistent_grants() {
        let mut ledger = ApprovalLedger::new("own-thread".into(), "own-turn".into());
        let id = RpcId::Text("native-opaque".into());
        let mut foreign = request();
        foreign["threadId"] = json!("other-project-thread");
        assert_eq!(
            ledger
                .insert(
                    id.clone(),
                    "item/commandExecution/requestApproval".into(),
                    foreign
                )
                .unwrap_err()
                .kind,
            ErrorKind::OwnershipMismatch
        );
        let mut foreign = request();
        foreign["turnId"] = json!("stale-turn");
        assert!(
            ledger
                .insert(
                    id.clone(),
                    "item/commandExecution/requestApproval".into(),
                    foreign
                )
                .is_err()
        );
        assert!(
            ledger
                .insert(id.clone(), "item/tool/requestUserInput".into(), request())
                .is_err()
        );
        let pending = ledger
            .insert(
                id.clone(),
                "item/commandExecution/requestApproval".into(),
                request(),
            )
            .unwrap();
        assert_eq!(pending.params, request());
        assert_eq!(
            ledger.reply(&id, OperationDecision::Approve).unwrap(),
            json!({"id":"native-opaque","result":{"decision":"accept"}})
        );
        assert!(ledger.reply(&id, OperationDecision::Approve).is_err());
        assert!(
            ledger
                .insert(
                    id,
                    "item/commandExecution/requestApproval".into(),
                    request()
                )
                .is_err()
        );
        assert!(ledger.is_empty());
    }
    #[test]
    fn usage_preserves_absence_and_observed_zero_and_rejects_invalid_counters() {
        let a = TokenCounters::from_native(
            &json!({"inputTokens":41,"cachedInputTokens":0,"totalTokens":43}),
        )
        .unwrap();
        let b = TokenCounters::from_native(
            &json!({"inputTokens":41,"cachedInputTokens":0,"totalTokens":43}),
        )
        .unwrap();
        assert_eq!(a, b);
        assert_eq!(a.cached_input, Some(0));
        assert_eq!(a.output, None);
        assert!(TokenCounters::from_native(&json!({"inputTokens":-1})).is_err());
        assert!(TokenCounters::from_native(&json!({"cachedInputTokens":"0"})).is_err());
        let mut tracker = UsageTracker::new("own-thread".into(), "own-turn".into());
        let mut notification = json!({"threadId":"own-thread","turnId":"own-turn","tokenUsage":{"total":{"inputTokens":41,"cachedInputTokens":0,"totalTokens":43},"last":{"inputTokens":41,"totalTokens":43}}});
        assert!(tracker.update(&notification).unwrap());
        assert!(!tracker.update(&notification).unwrap());
        assert_eq!(tracker.total.as_ref().unwrap().input, Some(41));
        notification["tokenUsage"]["total"]["inputTokens"] = json!(82);
        assert!(tracker.update(&notification).unwrap());
        assert_eq!(tracker.total.as_ref().unwrap().input, Some(82));
        notification["threadId"] = json!("other-project-thread");
        assert_eq!(
            tracker.update(&notification).unwrap_err().kind,
            ErrorKind::OwnershipMismatch
        );
        assert_eq!(tracker.total.as_ref().unwrap().input, Some(82));
        notification["threadId"] = json!("own-thread");
        notification["turnId"] = json!("stale-turn");
        assert!(tracker.update(&notification).is_err());
    }
    #[test]
    fn ambiguous_envelopes_and_invalid_ids_are_rejected() {
        for value in [
            json!({"id":1,"result":{},"error":{"code":1}}),
            json!({"id":{},"result":{}}),
            json!({"method":"event","result":{}}),
            json!({"method":"event","params":[]}),
            json!({"result":{}}),
        ] {
            assert!(Event::parse(value.to_string().as_bytes()).is_err());
        }
        assert!(matches!(
            Event::parse(br#"{"id":"opaque","method":"request","params":{}}"#).unwrap(),
            Event::Request { .. }
        ));
    }
    #[tokio::test]
    async fn websocket_control_frames_and_events_before_response_are_preserved() {
        let (client, server) = UnixStream::pair().unwrap();
        let server = tokio::spawn(async move {
            let mut server = accept_async(server).await.unwrap();
            let sent: Value =
                serde_json::from_str(server.next().await.unwrap().unwrap().to_text().unwrap())
                    .unwrap();
            server.send(Message::Ping(vec![1, 2].into())).await.unwrap();
            server
                .send(Message::Text(
                    json!({"method":"thread/started","params":{"threadId":"own"}})
                        .to_string()
                        .into(),
                ))
                .await
                .unwrap();
            server
                .send(Message::Text(
                    json!({"id":sent["id"],"result":{"ok":true}})
                        .to_string()
                        .into(),
                ))
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(25)).await;
        });
        let mut rpc = NativeRpc::connect(client).await.unwrap();
        assert_eq!(
            rpc.call("fixture", json!({})).await.unwrap(),
            json!({"ok":true})
        );
        assert!(
            matches!(rpc.receive().await.unwrap(), Event::Notification { method, .. } if method == "thread/started")
        );
        server.await.unwrap();
    }
    #[tokio::test]
    async fn response_identity_queue_bounds_and_oversized_frames_fail_closed() {
        for case in 0..3 {
            let (client, server) = UnixStream::pair().unwrap();
            let task = tokio::spawn(async move {
                let mut server = accept_async(server).await.unwrap();
                server.next().await.unwrap().unwrap();
                if case == 0 {
                    server
                        .send(Message::Text(
                            json!({"id":999,"result":{}}).to_string().into(),
                        ))
                        .await
                        .unwrap();
                } else if case == 1 {
                    for _ in 0..=MAX_QUEUED {
                        server
                            .send(Message::Text(
                                json!({"method":"event","params":{}}).to_string().into(),
                            ))
                            .await
                            .unwrap();
                    }
                } else {
                    let _ = server
                        .send(Message::Text("a".repeat(MAX_MESSAGE + 1).into()))
                        .await;
                }
            });
            let mut rpc = NativeRpc::connect(client).await.unwrap();
            assert_eq!(
                rpc.call("fixture", json!({})).await.unwrap_err().kind,
                ErrorKind::ParseFailure
            );
            // A rejected oversized frame leaves its sender backpressured until
            // the owning connection is closed; never await it while keeping it open.
            drop(rpc);
            tokio::time::timeout(Duration::from_secs(2), task)
                .await
                .expect("fixture peer terminates after connection close")
                .unwrap();
        }
    }
}
