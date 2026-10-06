//! Private allocation from the installed Native vtable, before start effects.
//! Read facts are not an owner, input-consumption or successful-work proof.
use super::{
    ArtifactId, ExecutionAuthority, ExecutionUnit, NativeInvocationId, OperationId, RuntimeOwner,
    UnitId, UnitState, WORKFLOW_SOURCE_BOOTSTRAP,
};
use crate::{
    adapter::{PreparedInput, native::NativeAllocationSeed},
    domain::{Scope, SessionId, SessionRole},
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::Serialize;
use std::{io::Write, path::Path, time::Duration};
use uuid::Uuid;

/// A provisional, effect-free allocation. The marker transaction must consume
/// it before preparation; allocating alone neither registers nor launches.
/// Only an actual installed NativePhasePort can construct its private seed.
pub(crate) struct NativeAllocation {
    seed: NativeAllocationSeed,
    session_id: SessionId,
    invocation_id: NativeInvocationId,
    operation_id: OperationId,
    pair_id: Uuid,
}

/// Borrowed, nongrant facts for coherent original-frame planning. Copying any
/// of these identifiers cannot construct NativeAllocation or its private seed.
pub(crate) struct AllocationFacts<'a> {
    pub scope: &'a Scope,
    pub unit_id: UnitId,
    pub generation: u64,
    pub epoch: u64,
    pub unit_version: u64,
    pub origin_id: Uuid,
    pub provider: &'a str,
    pub alias: &'a str,
    pub role: SessionRole,
    pub path: &'a Path,
    pub profile_digest: &'a str,
    pub model: Option<&'a str>,
    pub effort: Option<&'a str>,
    pub session_id: SessionId,
    pub invocation_id: NativeInvocationId,
    pub operation_id: OperationId,
    pub pair_id: Uuid,
    pub artifact: Option<ArtifactId>,
    pub input: &'a PreparedInput,
    pub input_bytes: &'a [u8],
}

impl NativeAllocation {
    pub(crate) fn from_selected(seed: NativeAllocationSeed) -> Self {
        Self {
            seed,
            session_id: SessionId::new(),
            invocation_id: NativeInvocationId::new(),
            operation_id: OperationId::new(),
            pair_id: Uuid::new_v4(),
        }
    }

    pub(crate) fn facts(&self) -> AllocationFacts<'_> {
        let unit = self.seed.unit();
        let port = self.seed.port();
        AllocationFacts {
            scope: &unit.scope,
            unit_id: unit.id,
            generation: unit.generation,
            epoch: unit.owner_epoch,
            unit_version: unit.version,
            origin_id: port.origin_id(),
            provider: port.provider(),
            alias: port.alias(),
            role: self.seed.role(),
            path: &unit.worktree,
            profile_digest: &unit.profile_digest,
            model: self.seed.model(),
            effort: self.seed.effort(),
            session_id: self.session_id,
            invocation_id: self.invocation_id,
            operation_id: self.operation_id,
            pair_id: self.pair_id,
            artifact: self.seed.input().artifact,
            input: &self.seed.input().input,
            input_bytes: self.seed.input_bytes(),
        }
    }
}

const UNIT_BYTES: usize = 16 * 1024;
const INPUT_BYTES: usize = 2 * 1024 * 1024;

/// Provisional current-unit snapshot, on a separate read-only connection. Full
/// owner/Context/Workflow/lock validation belongs to the marker's actual CAS.
pub(super) fn allocation_snapshot(
    owner: &RuntimeOwner,
    authority: &ExecutionAuthority,
) -> Result<ExecutionUnit> {
    let mut connection = Connection::open_with_flags(
        owner.state_path(),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.busy_timeout(Duration::from_millis(250))?;
    let tx = connection.transaction()?;
    ensure!(
        tx.query_row("PRAGMA application_id", [], |r| r.get::<_, i64>(0))?
            == crate::state::APPLICATION_ID
            && tx.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))?
                == crate::state::SCHEMA_VERSION,
        "native allocation state contract unavailable"
    );
    let body: String = tx
        .query_row(
            "SELECT body FROM execution_units WHERE id=?1 AND length(CAST(body AS BLOB))<=?2",
            params![authority.unit_id.to_string(), UNIT_BYTES],
            |r| r.get(0),
        )
        .optional()?
        .context("bounded native allocation Unit unavailable")?;
    let value = super::strict_json::decode(
        body.as_bytes(),
        super::strict_json::Limits {
            frame_bytes: UNIT_BYTES,
            depth: 8,
            nodes: 256,
            string_bytes: 4096,
            total_string_bytes: UNIT_BYTES,
            object_entries: 64,
            array_entries: 64,
        },
    )?;
    let unit: ExecutionUnit = serde_json::from_value(value)?;
    let scope = &unit.scope;
    let indexed: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM execution_units WHERE id=?1 AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND kind=?5 AND generation=?6 AND owner_epoch=?7 AND version=?8 AND native_effects_open=?9 AND result_finalization_open=?10 AND worktree=?11 AND branch IS ?12 AND body=?13)",
        params![unit.id.to_string(), scope.project_id.to_string(),
            scope.goal_id.context("native Goal missing")?.to_string(),
            scope.task_id.context("native Task missing")?.to_string(),
            serde_json::to_value(unit.kind)?.as_str().context("native kind unavailable")?,
            unit.generation, unit.owner_epoch, unit.version, unit.native_effects_open,
            unit.result_finalization_open, unit.worktree.to_str().context("native path is not UTF-8")?,
            unit.branch, body], |r| r.get(0),
    )?;
    let current: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM runtime_epoch e JOIN task_execution t ON t.task_id=?1 WHERE e.singleton=1 AND e.instance_id=?2 AND e.epoch=?3 AND t.project_id=?4 AND t.goal_id=?5 AND t.generation=?6)",
        params![scope.task_id.context("native Task missing")?.to_string(),
            owner.instance_id(), owner.epoch(), scope.project_id.to_string(),
            scope.goal_id.context("native Goal missing")?.to_string(), unit.generation],
        |r| r.get(0),
    )?;
    ensure!(
        indexed
            && current
            && unit.authority() == *authority
            && unit.owner_epoch == owner.epoch()
            && unit.native_effects_open
            && unit.result_finalization_open
            && unit.state == UnitState::Preparing
            && unit.work.is_none()
            && unit.session_id.is_none()
            && unit.phase != WORKFLOW_SOURCE_BOOTSTRAP
            && super::valid_oid(&unit.base_sha)
            && unit.worktree.is_absolute()
            && unit.worktree.as_os_str().len() <= 4096
            && unit.profile_digest.len() == 64,
        "native allocation current identity unavailable"
    );
    tx.commit()?;
    Ok(unit)
}

/// Encode all input fields into a finite original template. A bounded writer
/// enforces the complete escaped encoding cost, including keys and overhead.
pub(super) fn encode_input(input: &PreparedInput) -> Result<Vec<u8>> {
    ensure!(
        input.scope.goal_id.is_some()
            && input.scope.task_id.is_some()
            && input.version > 0
            && super::valid_oid(&input.revision)
            && !input.payload.is_empty()
            && input.payload.len() <= 1024 * 1024
            && input.source_versions.len() <= 4096
            && input.source_versions.iter().all(|(key, value)| {
                !key.is_empty()
                    && key.len() <= 4096
                    && !key.chars().any(char::is_control)
                    && !value.is_empty()
                    && value.len() <= 128
                    && !value.chars().any(char::is_control)
            }),
        "native allocation input bounds unavailable"
    );
    #[derive(Serialize)]
    struct Frame<'a> {
        scope: &'a Scope,
        kind: &'static str,
        revision: &'a str,
        version: u64,
        source_versions: &'a std::collections::BTreeMap<String, String>,
        payload: &'a str,
    }
    struct Limited(Vec<u8>);
    impl Write for Limited {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > INPUT_BYTES.saturating_sub(self.0.len()) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "native input encoding exceeds bound",
                ));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut encoded = Limited(Vec::new());
    serde_json::to_writer(
        &mut encoded,
        &Frame {
            scope: &input.scope,
            kind: match input.kind {
                crate::adapter::InputKind::ContextPack => "context_pack",
                crate::adapter::InputKind::ReviewBundle => "review_bundle",
            },
            revision: &input.revision,
            version: input.version,
            source_versions: &input.source_versions,
            payload: &input.payload,
        },
    )?;
    Ok(encoded.0)
}
