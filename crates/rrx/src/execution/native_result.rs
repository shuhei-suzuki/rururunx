//! Bounded native content DTOs. A serialized receipt is never approval authority.
use super::{
    ArtifactId, Disposition, NativeInvocationId, NativeResultId, OperationId, UnitId, WorkOutcome,
};
use crate::domain::{Scope, SessionId};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const INVOCATION_BYTES: usize = 64 * 1024;
pub const RECEIPT_BYTES: usize = 2 * 1024 * 1024;
pub const ANSWER_BYTES: usize = 1024 * 1024;
pub const PREFIX_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvocationState {
    NotDispatched,
    InputPending,
    Acknowledged,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeInvocation {
    pub id: NativeInvocationId,
    pub unit_id: UnitId,
    pub session_id: SessionId,
    pub scope: Scope,
    pub generation: u64,
    pub owner_epoch: u64,
    pub provider: String,
    pub profile: String,
    pub native_version: String,
    pub unit_version: u64,
    pub session_version: u64,
    pub context_version: Option<u64>,
    pub context_sha256: Option<String>,
    pub source_versions: BTreeMap<String, String>,
    pub source_sha256: String,
    pub revision: String,
    pub artifact_id: Option<ArtifactId>,
    pub artifact_version: Option<u64>,
    pub payload_sha256: String,
    pub input_operation: Option<OperationId>,
    pub frame_sha256: Option<String>,
    pub native_thread: Option<String>,
    pub native_turn: Option<String>,
    pub state: InvocationState,
    pub version: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionStatus {
    Complete,
    Missing,
    Partial,
    Ambiguous,
    Unsupported,
    Overflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptAuthority {
    OwnedTerminal,
    HistoricalDraft,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrefixEvidence {
    pub prefix: String,
    pub prefix_sha256: String,
    pub observed_bytes: u64,
    pub exact_length: bool,
    pub unseen_suffix: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireEvidence {
    pub observed_sha256: String,
    pub observed_bytes: u64,
    /// false means only a bounded observed prefix was hashed; never the full frame.
    pub exact_length: bool,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeResultReceipt {
    pub id: NativeResultId,
    pub invocation_id: NativeInvocationId,
    pub unit_id: UnitId,
    pub session_id: SessionId,
    pub scope: Scope,
    pub generation: u64,
    pub owner_epoch: u64,
    pub provider: String,
    pub native_thread: Option<String>,
    pub native_turn: Option<String>,
    pub acquisition: AcquisitionStatus,
    pub authority: ReceiptAuthority,
    pub text: Option<String>,
    pub structured_output: Option<serde_json::Value>,
    pub prefix: Option<PrefixEvidence>,
    pub answer_sha256: Option<String>,
    pub terminal_sha256: Option<String>,
    #[serde(default)]
    pub wire: Option<WireEvidence>,
    pub observed_work: WorkOutcome,
    pub disposition: Disposition,
    pub diagnostics: Vec<String>,
    pub captured_at: i64,
    pub version: u64,
}

pub(crate) fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}
fn sha(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn checked_version(value: u64) -> bool {
    value > 0 && value <= i64::MAX as u64
}
fn native_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && !value.contains('\0')
}

impl NativeInvocation {
    pub(crate) fn validate(&self) -> Result<()> {
        ensure!(
            self.scope.goal_id.is_some() && self.scope.task_id.is_some(),
            "native invocation requires Task scope"
        );
        ensure!(
            checked_version(self.generation)
                && checked_version(self.owner_epoch)
                && checked_version(self.version)
                && checked_version(self.unit_version)
                && checked_version(self.session_version),
            "invalid native invocation version"
        );
        ensure!(
            matches!(self.provider.as_str(), "claude" | "codex")
                && native_id(&self.profile)
                && native_id(&self.native_version),
            "invalid native invocation profile"
        );
        ensure!(
            self.context_version.is_some() == self.context_sha256.is_some()
                && self.context_version.is_none_or(checked_version)
                && self.context_sha256.as_deref().is_none_or(sha)
                && self.artifact_id.is_some() == self.artifact_version.is_some()
                && self.artifact_version.is_none_or(checked_version),
            "invalid native invocation input binding"
        );
        ensure!(
            sha(&self.payload_sha256)
                && sha(&self.source_sha256)
                && self.source_sha256 == digest(&serde_json::to_vec(&self.source_versions)?)
                && self.source_versions.len() <= 128
                && self.source_versions.iter().all(|(k, v)| !k.is_empty()
                    && k.len() <= 128
                    && !k.contains('\0')
                    && v.len() <= 256
                    && !v.contains('\0'))
                && super::valid_oid(&self.revision),
            "invalid native invocation source frame"
        );
        ensure!(
            self.input_operation.is_some() == self.frame_sha256.is_some()
                && self.frame_sha256.as_deref().is_none_or(sha)
                && self.native_thread.as_deref().is_none_or(native_id)
                && self.native_turn.as_deref().is_none_or(native_id),
            "invalid native input identity"
        );
        ensure!(
            self.state != InvocationState::NotDispatched || self.input_operation.is_none(),
            "undispatched invocation has an input effect"
        );
        ensure!(
            self.state != InvocationState::InputPending || self.input_operation.is_some(),
            "pending native input effect missing"
        );
        ensure!(
            self.state != InvocationState::Acknowledged
                || (self.input_operation.is_some()
                    && self.native_thread.is_some()
                    && (self.provider != "codex" || self.native_turn.is_some())),
            "unacknowledged native input"
        );
        ensure!(
            serde_json::to_vec(self)?.len() <= INVOCATION_BYTES,
            "native invocation encoding limit"
        );
        Ok(())
    }
}
impl NativeResultReceipt {
    pub(crate) fn projection(&self) -> serde_json::Value {
        serde_json::json!({"schema":"native_answer_v1","receipt_id":self.id,"acquisition":self.acquisition,
            "authority_class":self.authority,"work":self.observed_work,"disposition":self.disposition,
            "text":self.text,"prefix_evidence":self.prefix})
    }
    pub(crate) fn validate(&self) -> Result<()> {
        ensure!(
            self.scope.goal_id.is_some()
                && self.scope.task_id.is_some()
                && checked_version(self.generation)
                && checked_version(self.owner_epoch)
                && self.version == 1
                && self.captured_at >= 0,
            "invalid native receipt identity"
        );
        ensure!(
            matches!(self.provider.as_str(), "claude" | "codex")
                && self.native_thread.as_deref().is_none_or(native_id)
                && self.native_turn.as_deref().is_none_or(native_id),
            "invalid native receipt provider identity"
        );
        ensure!(
            self.diagnostics.len() <= 16
                && self.diagnostics.iter().all(|s| !s.is_empty()
                    && s.len() <= 64
                    && s.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')),
            "invalid native receipt diagnostic"
        );
        ensure!(
            self.answer_sha256.as_deref().is_none_or(sha)
                && self.terminal_sha256.as_deref().is_none_or(sha),
            "invalid native receipt digest"
        );
        // Initial acquisition profile is text-only. Retaining an arbitrary Value here
        // would bypass decoded-content/depth bounds for non-complete receipts.
        ensure!(
            self.structured_output.is_none(),
            "structured native content profile is unsupported"
        );
        if let Some(wire) = &self.wire {
            ensure!(
                sha(&wire.observed_sha256)
                    && wire.observed_bytes <= (4 * 1024 * 1024 + 8192)
                    && matches!(
                        wire.category.as_str(),
                        "frame_bytes"
                            | "invalid_json"
                            | "depth"
                            | "nodes"
                            | "string_bytes"
                            | "total_string_bytes"
                            | "object_entries"
                            | "array_entries"
                            | "duplicate_key"
                            | "eof"
                    ),
                "invalid native wire evidence"
            );
        }
        if let Some(prefix) = &self.prefix {
            ensure!(
                prefix.prefix.len() <= PREFIX_BYTES
                    && prefix.prefix_sha256 == digest(prefix.prefix.as_bytes())
                    && prefix.observed_bytes >= prefix.prefix.len() as u64
                    && prefix.unseen_suffix
                        == (!prefix.exact_length
                            || prefix.observed_bytes > prefix.prefix.len() as u64)
                    && self.acquisition != AcquisitionStatus::Missing,
                "invalid native receipt prefix"
            );
        }
        ensure!(
            self.text.as_ref().is_none_or(|s| s.len() <= ANSWER_BYTES),
            "native answer byte limit"
        );
        if self.acquisition == AcquisitionStatus::Complete {
            ensure!(
                self.prefix.is_none()
                    && self.structured_output.is_none()
                    && self.text.is_some()
                    && self.answer_sha256 == self.text.as_ref().map(|s| digest(s.as_bytes()))
                    && self.terminal_sha256.is_some(),
                "incomplete native answer receipt"
            );
        } else {
            ensure!(
                self.answer_sha256.is_none() && self.text.is_none(),
                "partial native content cannot have a complete answer"
            );
        }
        ensure!(
            serde_json::to_vec(self)?.len() <= RECEIPT_BYTES,
            "native receipt encoding limit"
        );
        Ok(())
    }
}

/// Pure bounded content collector. Only the native supervisor supplies owned frames;
/// neither this content nor its projection constitutes review authority.
#[derive(Default)]
pub(super) struct Collector {
    items: BTreeMap<String, (String, String)>,
    final_text: Option<String>,
    final_id: Option<String>,
    draft: String,
    observed: u64,
    observed_complete: bool,
    events: usize,
    exhausted: bool,
    completed: bool,
    claude_signature: Option<String>,
    claude_ids: BTreeMap<String, String>,
    status: Option<AcquisitionStatus>,
    terminal: Option<String>,
    diagnostics: Vec<String>,
}
pub(super) struct Content {
    pub acquisition: AcquisitionStatus,
    pub text: Option<String>,
    pub prefix: Option<PrefixEvidence>,
    pub answer_sha256: Option<String>,
    pub terminal_sha256: Option<String>,
    pub diagnostics: Vec<String>,
}
impl Collector {
    fn poison(&mut self, status: AcquisitionStatus, diagnostic: &str) {
        if status == AcquisitionStatus::Overflow {
            self.exhausted = true;
        }
        // Overflow and ambiguity are sticky; no subsequent successful frame repairs them.
        if !matches!(
            self.status,
            Some(AcquisitionStatus::Overflow | AcquisitionStatus::Ambiguous)
        ) {
            self.status = Some(status);
        }
        if self.diagnostics.len() < 16 && !self.diagnostics.iter().any(|d| d == diagnostic) {
            self.diagnostics.push(diagnostic.into());
        }
    }
    fn event(&mut self) -> bool {
        self.events = self.events.saturating_add(1);
        if self.events > 1024 {
            self.poison(AcquisitionStatus::Overflow, "answer_event_limit");
            false
        } else {
            true
        }
    }
    fn prefix(&mut self, text: &str, append: bool) {
        self.observed_complete = !append;
        if !append {
            self.draft.clear();
            self.observed = 0;
        }
        self.observed = self.observed.saturating_add(text.len() as u64);
        let available = PREFIX_BYTES.saturating_sub(self.draft.len());
        let mut n = text.len().min(available);
        while !text.is_char_boundary(n) {
            n -= 1;
        }
        self.draft.push_str(&text[..n]);
    }
    pub(super) fn overflowed(&self) -> bool {
        self.exhausted
    }
    pub(super) fn confirm_complete(&mut self) {
        self.completed = true;
    }
    pub(super) fn claude_redelivery(&mut self, frame: &serde_json::Value) {
        if frame["uuid"]
            .as_str()
            .is_some_and(|id| self.claude_ids.contains_key(id))
        {
            self.claude_terminal(frame);
        }
    }
    pub(super) fn wire_failed(&mut self, wire: &WireEvidence) {
        let status = if matches!(
            wire.category.as_str(),
            "frame_bytes"
                | "string_bytes"
                | "total_string_bytes"
                | "nodes"
                | "object_entries"
                | "array_entries"
        ) {
            AcquisitionStatus::Overflow
        } else if wire.category == "duplicate_key" {
            AcquisitionStatus::Ambiguous
        } else {
            AcquisitionStatus::Partial
        };
        self.poison(status, "native_wire_rejected");
    }
    pub(super) fn protocol_lost(&mut self) {
        self.poison(AcquisitionStatus::Partial, "native_protocol_incomplete");
    }
    pub(super) fn discard_sensitive(&mut self) {
        self.final_text = None;
        self.draft.clear();
        self.observed = 0;
        self.status = Some(AcquisitionStatus::Missing);
        self.terminal = None;
        self.diagnostics = vec!["native_sensitive_failure".into()];
    }
    pub(super) fn codex_item(&mut self, item: &serde_json::Value, completed: bool) {
        if !self.event() {
            return;
        }
        let Some(id) = item["id"].as_str().filter(|id| native_id(id)) else {
            self.poison(AcquisitionStatus::Ambiguous, "answer_item_identity_missing");
            return;
        };
        let Some(kind) = item["type"].as_str() else {
            self.poison(AcquisitionStatus::Ambiguous, "answer_item_type_missing");
            return;
        };
        if self.items.get(id).is_some_and(|(old, _)| old != kind) {
            self.poison(AcquisitionStatus::Ambiguous, "answer_item_type_changed");
            return;
        }
        if !self.items.contains_key(id) && self.items.len() >= 256 {
            self.poison(AcquisitionStatus::Overflow, "answer_item_limit");
            return;
        }
        if kind != "agentMessage" || !completed {
            self.items
                .entry(id.into())
                .or_insert((kind.into(), String::new()));
            return;
        }
        let signature = digest(&serde_json::to_vec(item).expect("Value encoding"));
        if let Some((_, old)) = self.items.get(id)
            && !old.is_empty()
        {
            if old != &signature {
                self.poison(AcquisitionStatus::Ambiguous, "answer_item_changed");
            }
            return;
        }
        self.items.insert(id.into(), (kind.into(), signature));
        let Some(text) = item["text"].as_str() else {
            self.poison(AcquisitionStatus::Ambiguous, "answer_text_missing");
            return;
        };
        if text.len() > ANSWER_BYTES {
            self.prefix(text, false);
            self.poison(AcquisitionStatus::Overflow, "answer_byte_limit");
            return;
        }
        if item.get("delivery").is_some_and(|v| !v.is_null())
            || item
                .get("questions")
                .is_some_and(|v| !v.is_null() && v.as_array().is_none_or(|a| !a.is_empty()))
        {
            self.poison(
                AcquisitionStatus::Unsupported,
                "answer_delivery_unsupported",
            );
            return;
        }
        match item["phase"].as_str() {
            Some("commentary") => {}
            Some("final_answer") => {
                if self.final_id.as_deref().is_some_and(|old| old != id) {
                    self.poison(AcquisitionStatus::Ambiguous, "answer_multiple_finals");
                    return;
                }
                self.prefix(text, false);
                self.final_id = Some(id.into());
                self.final_text = Some(text.into());
            }
            _ => {
                self.prefix(text, false);
                self.poison(AcquisitionStatus::Unsupported, "answer_phase_unsupported");
            }
        }
    }
    pub(super) fn codex_delta(&mut self, params: &serde_json::Value) {
        if !self.event() {
            return;
        }
        let Some(delta) = params["delta"].as_str() else {
            self.poison(AcquisitionStatus::Ambiguous, "answer_delta_invalid");
            return;
        };
        // Drafts never become complete answers, even if a terminal later succeeds.
        if self.final_text.is_none() {
            self.prefix(delta, true);
        }
        if self.observed > ANSWER_BYTES as u64 {
            self.poison(AcquisitionStatus::Overflow, "answer_byte_limit");
        }
    }
    pub(super) fn codex_terminal(&mut self, turn: &serde_json::Value) {
        self.terminal = Some(digest(&serde_json::to_vec(turn).expect("Value encoding")));
        let Some(items) = turn["items"].as_array() else {
            self.poison(
                AcquisitionStatus::Ambiguous,
                "answer_terminal_items_missing",
            );
            return;
        };
        if turn.get("itemsView").is_none_or(|v| v == "full") {
            for item in items {
                if item["type"] != "agentMessage" {
                    continue;
                }
                let Some(id) = item["id"].as_str() else {
                    self.poison(
                        AcquisitionStatus::Ambiguous,
                        "answer_terminal_identity_missing",
                    );
                    continue;
                };
                if self.items.get(id).is_none_or(|(_, signature)| {
                    signature.is_empty()
                        || signature != &digest(&serde_json::to_vec(item).expect("Value encoding"))
                }) {
                    self.poison(
                        AcquisitionStatus::Ambiguous,
                        "answer_terminal_unobserved_or_changed",
                    );
                }
            }
            if let Some(id) = &self.final_id
                && !items.iter().any(|item| item["id"] == *id)
            {
                self.poison(
                    AcquisitionStatus::Ambiguous,
                    "answer_terminal_final_missing",
                );
            }
        } else if !matches!(turn["itemsView"].as_str(), Some("summary" | "notLoaded")) {
            self.poison(
                AcquisitionStatus::Unsupported,
                "answer_terminal_view_unsupported",
            );
        }
    }
    pub(super) fn claude_terminal(&mut self, frame: &serde_json::Value) {
        if !self.event() {
            return;
        }
        let signature=digest(&serde_json::to_vec(&serde_json::json!({"subtype":frame["subtype"],"is_error":frame["is_error"],"result":frame["result"],"structured_output":frame["structured_output"]})).expect("Value encoding"));
        if let Some(id) = frame["uuid"].as_str() {
            if !self.claude_ids.contains_key(id) && self.claude_ids.len() >= 256 {
                self.poison(AcquisitionStatus::Overflow, "answer_item_limit");
                return;
            }
            if self.claude_ids.get(id).is_some_and(|old| old != &signature) {
                self.poison(AcquisitionStatus::Ambiguous, "answer_result_changed");
                return;
            }
            self.claude_ids.insert(id.into(), signature.clone());
        }
        if let Some(old) = &self.claude_signature {
            if old != &signature {
                self.poison(AcquisitionStatus::Ambiguous, "answer_multiple_results");
            }
            return;
        }
        self.claude_signature = Some(signature);
        self.terminal = Some(digest(&serde_json::to_vec(frame).expect("Value encoding")));
        if frame["subtype"] != "success" {
            self.poison(
                AcquisitionStatus::Unsupported,
                "answer_error_representation_unsupported",
            );
            return;
        }
        if frame.get("structured_output").is_some_and(|v| !v.is_null()) {
            if let Some(text) = frame["result"].as_str() {
                self.prefix(text, false);
            }
            self.poison(
                AcquisitionStatus::Unsupported,
                "answer_structured_profile_unsupported",
            );
            return;
        }
        match frame["result"].as_str() {
            Some(text) if text.len() <= ANSWER_BYTES => {
                self.prefix(text, false);
                self.final_text = Some(text.into());
            }
            Some(text) => {
                self.prefix(text, false);
                self.poison(AcquisitionStatus::Overflow, "answer_byte_limit");
            }
            None => self.poison(AcquisitionStatus::Missing, "answer_text_missing"),
        }
    }
    pub(super) fn content(&self) -> Content {
        let status = self.status.unwrap_or(
            if self.completed && self.final_text.is_some() && self.terminal.is_some() {
                AcquisitionStatus::Complete
            } else if self.observed > 0 {
                AcquisitionStatus::Partial
            } else {
                AcquisitionStatus::Missing
            },
        );
        let text = (status == AcquisitionStatus::Complete)
            .then(|| self.final_text.clone())
            .flatten();
        let prefix = (status != AcquisitionStatus::Complete
            && status != AcquisitionStatus::Missing
            && self.observed > 0)
            .then(|| PrefixEvidence {
                prefix: self.draft.clone(),
                prefix_sha256: digest(self.draft.as_bytes()),
                observed_bytes: self.observed,
                exact_length: self.observed_complete,
                unseen_suffix: !self.observed_complete || self.observed > self.draft.len() as u64,
            });
        let hash = text.as_ref().map(|t| digest(t.as_bytes()));
        Content {
            acquisition: status,
            text,
            prefix,
            answer_sha256: hash,
            terminal_sha256: self.terminal.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }
}
