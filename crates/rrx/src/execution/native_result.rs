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
        if let Some(prefix) = &self.prefix {
            ensure!(
                prefix.prefix.len() <= PREFIX_BYTES
                    && prefix.prefix_sha256 == digest(prefix.prefix.as_bytes())
                    && prefix.observed_bytes >= prefix.prefix.len() as u64,
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
