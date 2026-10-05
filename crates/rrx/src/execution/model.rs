use crate::domain::{RecordId, Scope, SessionId};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};
use uuid::Uuid;

macro_rules! identity {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);
        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }
        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
        impl std::str::FromStr for $name {
            type Err = uuid::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(s).map(Self)
            }
        }
    };
}
identity!(UnitId);
identity!(ArtifactId);
identity!(LeaseId);
identity!(OperationId);

/// Stable semantic identity before a native Session exists. Versions are fetched
/// only for this exact unit, never by Task or diagnostic recovery metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedUnitRef {
    pub scope: Scope,
    pub unit: UnitId,
    pub generation: u64,
    pub epoch: u64,
}
impl From<&ExecutionUnit> for ManagedUnitRef {
    fn from(unit: &ExecutionUnit) -> Self {
        Self {
            scope: unit.scope.clone(),
            unit: unit.id,
            generation: unit.generation,
            epoch: unit.owner_epoch,
        }
    }
}

#[derive(Clone)]
pub(crate) struct WorkflowReservation {
    pub record: RecordId,
    pub version: u64,
    pub index: usize,
    pub workflow_generation: u64,
    pub context: u64,
    pub project_version: u64,
    pub goal_version: u64,
}

macro_rules! states {
    ($name:ident {$($variant:ident),+ $(,)?}) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),+ }
    }
}
states!(UnitKind {
    Executor,
    Reviewer,
    Verifier,
    Legacy
});
states!(UnitState {
    Reserved,
    Preparing,
    DispatchPending,
    Running,
    WaitingQuota,
    WorkKnown,
    WorkUnknown,
    Retired,
    LegacyUnreconciled
});
states!(WorkOutcome {
    Success,
    Failure,
    Unknown
});
states!(CleanupOutcome {
    Reclaimed,
    Leftovers,
    Unknown
});
states!(Disposition {
    Active,
    Completed,
    Cancelled,
    QuotaInterrupted,
    CapacityInterrupted,
    Lost,
    Refused,
    ProtocolError,
    Legacy
});
states!(NativeFailure {
    AuthenticationUnavailable,
    UnsupportedCapability,
    MetadataUnavailable,
    ProtocolFailure,
    TransportLost,
    AuthorityUnavailable
});
impl NativeFailure {
    pub fn diagnostic(self) -> &'static str {
        match self {
            Self::AuthenticationUnavailable => "native subscription authentication unavailable",
            Self::UnsupportedCapability => "native capability unavailable",
            Self::MetadataUnavailable => "native quota metadata unavailable",
            Self::ProtocolFailure => "native protocol failure",
            Self::TransportLost => "native transport lost",
            Self::AuthorityUnavailable => "native execution authority unavailable",
        }
    }
}
impl std::fmt::Display for NativeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.diagnostic())
    }
}
impl std::error::Error for NativeFailure {}
states!(ArtifactState {
    Staging,
    Ready,
    Published,
    Invalid
});
states!(LeaseState {
    Reserved,
    Creating,
    Active,
    Quarantined,
    Released
});
states!(ResourceKind {
    Worktree,
    Temp,
    Output,
    Ports,
    Docker,
    ToolSocket
});
states!(EffectState {
    Pending,
    Confirmed,
    Unknown,
    Resolved
});
states!(QuotaStatus {
    Unknown,
    Available,
    Exhausted,
    Stale
});
states!(WaitReason {
    Quota,
    Resource,
    Capacity,
    Approval,
    Evidence,
    ExternalOutcome
});

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionUnit {
    pub id: UnitId,
    pub scope: Scope,
    pub kind: UnitKind,
    pub generation: u64,
    pub owner_epoch: u64,
    pub version: u64,
    pub phase: String,
    pub provider: String,
    pub state: UnitState,
    pub native_effects_open: bool,
    pub result_finalization_open: bool,
    pub work: Option<WorkOutcome>,
    pub cleanup: CleanupOutcome,
    pub disposition: Disposition,
    pub worktree: PathBuf,
    pub branch: Option<String>,
    pub base_sha: String,
    pub profile_digest: String,
    pub cookie: String,
    pub session_id: Option<SessionId>,
    pub artifact_id: Option<ArtifactId>,
    pub wait_reason: Option<WaitReason>,
    #[serde(default)]
    pub capacity_retry_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionAuthority {
    pub scope: Scope,
    pub unit_id: UnitId,
    pub generation: u64,
    pub owner_epoch: u64,
    pub session_id: Option<SessionId>,
    pub record_version: u64,
}

impl ExecutionUnit {
    pub fn authority(&self) -> ExecutionAuthority {
        ExecutionAuthority {
            scope: self.scope.clone(),
            unit_id: self.id,
            generation: self.generation,
            owner_epoch: self.owner_epoch,
            session_id: self.session_id,
            record_version: self.version,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultArtifact {
    pub id: ArtifactId,
    pub scope: Scope,
    pub unit_id: UnitId,
    pub state: ArtifactState,
    pub revision: String,
    pub base_sha: String,
    pub object_format: String,
    pub repository: PathBuf,
    pub manifest: PathBuf,
    pub manifest_sha256: String,
    pub dependencies: BTreeMap<String, String>,
    pub version: u64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceLease {
    pub id: LeaseId,
    pub unit_id: UnitId,
    pub scope: Scope,
    pub kind: ResourceKind,
    pub namespace: String,
    pub value: String,
    pub port_start: Option<u16>,
    pub port_end: Option<u16>,
    pub state: LeaseState,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedEffect {
    pub id: OperationId,
    pub unit_id: UnitId,
    pub scope: Scope,
    pub kind: String,
    pub idempotency_key: String,
    pub expected_target: String,
    pub state: EffectState,
    /// Safe IDs only. Tool inputs, environment, credentials and raw output are never receipts.
    pub receipt: BTreeMap<String, String>,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CleanupObservation {
    pub unit_id: UnitId,
    pub at: i64,
    pub outcome: CleanupOutcome,
    pub coverage: BTreeMap<String, String>,
    pub remaining: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuotaObservation {
    pub provider: String,
    pub account_key: String,
    pub bucket: String,
    pub window_id: String,
    pub status: QuotaStatus,
    pub used_percent: Option<f64>,
    pub resets_at: Option<i64>,
    pub observed_at: i64,
    pub source_version: String,
    pub confirmed_subscription: bool,
}

/// A review gate is always attached to retained input, not a native exit code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactEvidence {
    pub artifact_id: ArtifactId,
    pub unit_id: UnitId,
    pub revision: String,
    pub source_versions: BTreeMap<String, String>,
    pub evidence_sha256: String,
    pub accepted: bool,
    pub workflow_record: Option<RecordId>,
}

pub fn valid_oid(oid: &str) -> bool {
    matches!(oid.len(), 40 | 64)
        && oid
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
pub(crate) fn key<T: Serialize>(value: T) -> String {
    serde_json::to_value(value)
        .expect("enum serialization")
        .as_str()
        .expect("enum string")
        .to_owned()
}
