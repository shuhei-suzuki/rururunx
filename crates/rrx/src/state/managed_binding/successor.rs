//! Nongrant coherent Workflow ledger currency. Actual Native/pair eligibility
//! and every permitted typed writer are separate mandatory consumers.
use super::{
    canonical::{Body, LEDGER_DOMAIN, WORKFLOW_DOMAIN},
    marker_plan::unit_index_matches,
    publication::{MarkerPublicationPlan, OriginalMarker},
    snapshot::{read_scope, snapshot},
};
use crate::{
    domain::{AuditEvent, Record},
    execution::{ExecutionUnit, RuntimeOwner, UnitKind},
    workflow::WorkflowSnapshot,
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, Transaction, params};
use serde_json::Value;
use std::sync::Arc;

const KINDS: &str = "'rrx.private.workflow.session_bound','rrx.private.workflow.native_diagnostic','rrx.private.workflow.gate_claim','rrx.private.workflow.gate_observed','rrx.private.workflow.gate_hold','rrx.private.workflow.terminal_decision','rrx.private.workflow.phase_closed'";
const RECIPE: &str = "rrx.workflow-body-sha256/v1";
const UNIT_BYTES: usize = 16 * 1024;

/// Private coherent read product, not Clone/Deserialize or Native authority.
/// Keeping the original actual plan prevents SQL/current-row origin replacement.
pub(crate) struct CurrentWorkflowSuccessor {
    original: Arc<MarkerPublicationPlan>,
    workflow: Arc<Body<Record>>,
    unit: Arc<Body<ExecutionUnit>>,
    count: usize,
    head: Option<Arc<Link>>,
}
struct Link {
    event: AuditEvent,
    raw: String,
}
impl CurrentWorkflowSuccessor {
    pub(in crate::state) fn copy_original(&self) -> Self {
        Self { original:self.original.clone(), workflow:self.workflow.clone(), unit:self.unit.clone(),
            count:self.count, head:self.head.clone() }
    }
    /// Only an own planned, known committed transition can reach this port.
    /// It copies the original parent/ledger facts and performs no row lookup.
    pub(in crate::state) fn with_known_unit(&self, body: Arc<Body<ExecutionUnit>>) -> Result<Self> {
        validate_unit_identity(body.parsed(), self.unit(), true)?;
        Ok(Self {
            original: self.original.clone(),
            workflow: self.workflow.clone(),
            unit: body, count: self.count,
            head: self.head.clone(),
        })
    }
    pub(crate) fn workflow(&self) -> &Record {
        self.workflow.parsed()
    }
    pub(super) fn workflow_raw(&self) -> &str {
        self.workflow.raw()
    }
    pub(super) fn sole_binding_link(&self) -> Option<&AuditEvent> {
        self.head
            .as_ref()
            .filter(|link| {
                self.count == 1 && link.event.kind == "rrx.private.workflow.session_bound"
            })
            .map(|link| &link.event)
    }
    /// Current factual projection, never a live owner or input admission proof.
    pub(crate) fn unit(&self) -> &ExecutionUnit {
        self.unit.parsed()
    }
    pub(crate) fn unit_raw(&self) -> &str {
        self.unit.raw()
    }
    pub(crate) fn has_links(&self) -> bool {
        self.count != 0
    }
}

fn validate_unit_identity(unit: &ExecutionUnit, original: &ExecutionUnit, increasing: bool) -> Result<()> {
    ensure!(unit.id == original.id && unit.scope == original.scope && unit.kind == original.kind
        && unit.generation == original.generation && unit.owner_epoch == original.owner_epoch
        && unit.phase == original.phase && unit.provider == original.provider && unit.worktree == original.worktree
        && unit.branch == original.branch && unit.base_sha == original.base_sha && unit.profile_digest == original.profile_digest
        && unit.cookie == original.cookie && unit.created_at == original.created_at
        && unit.version >= original.version && (!increasing || unit.version > original.version)
        && unit.version <= i64::MAX as u64 && (unit.kind != UnitKind::Reviewer || unit.artifact_id == original.artifact_id),
        "current allocated Unit immutable identity or version changed");
    Ok(())
}

fn current_unit(c: &Connection, marker: &OriginalMarker) -> Result<Body<ExecutionUnit>> {
    let original = marker.unit();
    let raw: Option<String> = c.query_row(
        "SELECT CASE WHEN length(CAST(body AS BLOB))<=?2 THEN body END FROM execution_units WHERE id=?1",
        params![original.id.to_string(), UNIT_BYTES],
        |r| r.get(0),
    )?;
    let body = Body::<ExecutionUnit>::decode(
        raw.context("current allocated Unit exceeds complete body bound")?,
        UNIT_BYTES,
    )?;
    let unit = body.parsed();
    validate_unit_identity(unit, original, false)?;
    ensure!(
        unit.id == original.id
            && unit.scope == original.scope
            && unit.kind == original.kind
            && unit.generation == original.generation
            && unit.owner_epoch == original.owner_epoch
            && unit.phase == original.phase
            && unit.provider == original.provider
            && unit.worktree == original.worktree
            && unit.branch == original.branch
            && unit.base_sha == original.base_sha
            && unit.profile_digest == original.profile_digest
            && unit.cookie == original.cookie
            && unit.created_at == original.created_at
            && unit.version >= original.version
            && unit.version <= i64::MAX as u64
            && (unit.kind != UnitKind::Reviewer || unit.artifact_id == original.artifact_id)
            && unit_index_matches(c, unit, body.raw())?,
        "current allocated Unit immutable identity or complete index changed"
    );
    // Executor artifact publication may advance artifact_id legitimately. All
    // mutable lifecycle/result fields are factual only; each actual consumer
    // must prove its own stage eligibility and original private input/owner.
    Ok(body)
}

fn rows(c: &Connection, marker: &OriginalMarker) -> Result<Vec<Link>> {
    let sql = format!(
        "SELECT sequence,project_id,goal_id,task_id,kind,at,CASE WHEN length(CAST(data AS BLOB))<=4096 THEN data END FROM audit WHERE kind IN ({KINDS}) AND json_extract(data,'$.private_operation_ref')=?1 ORDER BY sequence LIMIT 257"
    );
    let mut statement = c.prepare(&sql)?;
    let mut cursor = statement.query([marker.operation().to_string()])?;
    let mut events = Vec::new();
    while let Some(row) = cursor.next()? {
        ensure!(
            events.len() < 256,
            "complete Workflow ledger exceeds256 links"
        );
        let raw: Option<String> = row.get(6)?;
        let data = Body::<Value>::decode(
            raw.context("complete Workflow link exceeds4096bytes")?,
            4096,
        )?;
        events.push(Link {
            event: AuditEvent {
                sequence: row.get(0)?,
                scope: crate::domain::Scope {
                    project_id: row.get::<_, String>(1)?.parse()?,
                    goal_id: row
                        .get::<_, Option<String>>(2)?
                        .map(|s| s.parse())
                        .transpose()?,
                    task_id: row
                        .get::<_, Option<String>>(3)?
                        .map(|s| s.parse())
                        .transpose()?,
                },
                kind: row.get(4)?,
                at: row.get(5)?,
                data: data.parsed().clone(),
            },
            raw: data.raw().to_owned(),
        });
    }
    Ok(events)
}
fn hash_text(value: &Value) -> Result<&str> {
    let s = value
        .as_str()
        .context("Workflow ledger digest is not text")?;
    ensure!(
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "Workflow ledger digest encoding differs"
    );
    Ok(s)
}

/// Complete chain extraction and hashes occur on one query-only snapshot,
/// before SharedStore. This validates currency, never permission or success.
pub(crate) fn plan_current_phase(
    owner: &RuntimeOwner,
    marker: &OriginalMarker,
) -> Result<CurrentWorkflowSuccessor> {
    let original = marker.original_plan();
    let f = marker.allocation().facts();
    ensure!(
        f.state_path == owner.state_path()
            && f.instance_id == owner.instance_id()
            && f.epoch == owner.epoch(),
        "current plan uses another actual owner"
    );
    snapshot(owner, |tx| {
        marker.validate_open_tx(tx)?;
        let unit = current_unit(tx, marker)?;
        let current = read_scope(tx, owner, &marker.scope())?;
        let workflow = current.workflow.context("current Workflow absent")?;
        original.before.validate_projection(
            tx,
            original.task_after.parsed(),
            original.task_after.raw(),
            Some((workflow.parsed(), workflow.raw())),
        )?;
        let template: WorkflowSnapshot =
            serde_json::from_value(original.workflow_after.parsed().data.clone())?;
        let index = template
            .active
            .context("original marker active attempt absent")?;
        let phase = template
            .history
            .get(index)
            .context("original marker attempt absent")?
            .phase
            .key();
        let links = rows(tx, marker)?;
        if links.is_empty() {
            ensure!(
                workflow.raw() == original.workflow_after.raw(),
                "unlinked original Workflow encoded bytes changed"
            );
        }
        let mut version = original.workflow_after.parsed().version;
        let mut body_digest = original.workflow_after.digest(WORKFLOW_DOMAIN);
        let mut ledger_digest = marker.frame_digest().to_owned();
        let mut sequence = 0;
        for (ordinal, link) in links.iter().enumerate() {
            let event = &link.event;
            let data = &event.data;
            ensure!(
                event.sequence > sequence
                    && event.scope == marker.scope()
                    && data["kind"].as_str() == event.kind.strip_prefix("rrx.private.workflow.")
                    && data["canonical_body_recipe"] == RECIPE
                    && data["project_id"] == f.scope.project_id.to_string()
                    && data["goal_id"]
                        == f.scope.goal_id.context("marker Goal absent")?.to_string()
                    && data["task_id"]
                        == f.scope.task_id.context("marker Task absent")?.to_string()
                    && data["workflow_id"] == workflow.parsed().id.to_string()
                    && data["private_operation_ref"] == marker.operation().to_string()
                    && data["original_marker_frame_sha256"] == marker.frame_digest()
                    && data["generation"].as_u64() == Some(template.generation)
                    && data["attempt_index"].as_u64() == Some(u64::try_from(index)?)
                    && data["phase"] == phase
                    && data["context_version"].as_u64() == Some(original.context().0.version)
                    && data["workflow_version_before"].as_u64() == Some(version)
                    && data["workflow_version_after"].as_u64() == version.checked_add(1)
                    && data["workflow_body_sha256_before"] == body_digest
                    && data["prior_ledger_digest"] == ledger_digest,
                "original complete Workflow ledger predecessor differs"
            );
            if event.kind == "rrx.private.workflow.session_bound" {
                ensure!(ordinal == 0, "binding is not the first factual link");
            }
            // Own digest is excluded; immutable payload is hashed in full.
            ensure!(
                data.get("ledger_digest").is_none(),
                "self-referential Workflow digest"
            );
            body_digest = hash_text(&data["workflow_body_sha256_after"])?.to_owned();
            ledger_digest = Body::<Value>::decode(link.raw.clone(), 4096)?.digest(LEDGER_DOMAIN);
            version = version
                .checked_add(1)
                .context("Workflow version exhausted")?;
            sequence = event.sequence;
        }
        ensure!(
            workflow.parsed().id == original.workflow_after.parsed().id
                && workflow.parsed().version == version
                && workflow.digest(WORKFLOW_DOMAIN) == body_digest,
            "current Workflow is not the complete ledger endpoint"
        );
        Ok(CurrentWorkflowSuccessor {
            original: marker.publication_plan(),
            workflow: Arc::new(workflow),
            unit: Arc::new(unit),
            count: links.len(),
            head: links.into_iter().last().map(Arc::new),
        })
    })
}

/// Rechecks exact current Workflow and immutable ledger endpoint in the effect's
/// SAME Immediate transaction. No hashing, IO or Native calls occur here.
/// Native eligibility must additionally validate its real original owner/pair.
pub(crate) fn validate_current_tx(
    tx: &Transaction<'_>,
    marker: &OriginalMarker,
    current: &CurrentWorkflowSuccessor,
) -> Result<()> {
    ensure!(
        marker.matches_original_plan(&current.original),
        "different original marker current plan"
    );
    let original = marker.original_plan();
    marker.validate_open_tx(tx)?;
    original.before.validate_projection(
        tx,
        original.task_after.parsed(),
        original.task_after.raw(),
        Some((current.workflow.parsed(), current.workflow.raw())),
    )?;
    ensure!(
        unit_index_matches(tx, current.unit.parsed(), current.unit.raw())?,
        "complete current allocated Unit changed before effect admission"
    );
    let sql = format!(
        "SELECT count(*),COALESCE(max(sequence),0) FROM (SELECT sequence FROM audit WHERE kind IN ({KINDS}) AND json_extract(data,'$.private_operation_ref')=?1 ORDER BY sequence LIMIT 257)"
    );
    let actual: (usize, i64) = tx.query_row(&sql, [marker.operation().to_string()], |r| {
        Ok((r.get(0)?, r.get(1)?))
    })?;
    ensure!(
        actual
            == (
                current.count,
                current.head.as_ref().map_or(0, |h| h.event.sequence)
            ),
        "complete immutable Workflow ledger head changed"
    );
    if let Some(head) = &current.head {
        let event = &head.event;
        let exact: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM audit WHERE sequence=?1 AND project_id=?2 AND goal_id IS ?3 AND task_id IS ?4 AND kind=?5 AND at=?6 AND data=?7)",
            params![event.sequence,event.scope.project_id.to_string(),event.scope.goal_id.map(|v|v.to_string()),event.scope.task_id.map(|v|v.to_string()),event.kind,event.at,head.raw],|r|r.get(0),
        )?;
        ensure!(exact, "exact Workflow ledger endpoint changed");
    }
    Ok(())
}
