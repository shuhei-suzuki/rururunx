//! Original-frame planning from the actual selected allocation. A plan is NOT
//! OriginalMarker: only its future genuine capacity/Driver-checked Immediate
//! publication may produce that authority. There is deliberately no launch port.
use super::{
    canonical::{BODY_BYTES, Body, WORKFLOW_DOMAIN, encode},
    snapshot::{ScopePlan, read_scope, snapshot},
};
use crate::{
    adapter::InputKind,
    domain::*,
    execution::{
        Disposition, ExecutionUnit, RuntimeOwner, UnitState, native_result,
        phase::{AllocationFacts, NativeAllocation},
    },
    workflow::{Actor, AttemptState, WorkflowSnapshot},
};
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use std::collections::BTreeMap;

const UNIT_BYTES: usize = 16 * 1024;

/// Complete immutable pre/post content; not Clone/Deserialize and not usable by
/// any Native entry. Native allocation remains owned by its retained supervisor.
pub(crate) struct ManagedMarkerPlan {
    pub(super) before: ScopePlan,
    pub(super) unit: Body<ExecutionUnit>,
    pub(super) governing: (u64, u64, String),
    pub(super) task_after: Body<Task>,
    pub(super) workflow_after: Body<Record>,
    pub(super) context_digest: String,
    pub(super) input_digest: String,
    pub(super) frame: Vec<u8>,
    pub(super) frame_digest: String,
    pub(super) operation: crate::execution::OperationId,
    pub(super) pair: uuid::Uuid,
    pub(super) allocated_session: SessionId,
    pub(super) origin: uuid::Uuid,
}

#[derive(Serialize)]
struct BodyPin {
    version: u64,
    sha256: String,
}
#[derive(Serialize)]
struct LockPin {
    id: RecordId,
    version: u64,
    sha256: String,
}
#[derive(Serialize)]
struct Frame<'a> {
    canonical_body_recipe: &'static str,
    instance: &'a str,
    epoch: u64,
    scope: &'a Scope,
    project: BodyPin,
    goal: BodyPin,
    task: BodyPin,
    workflow_id: RecordId,
    workflow: BodyPin,
    workflow_generation: u64,
    attempt_index: usize,
    phase: &'a str,
    actor: &'a str,
    provider: &'a str,
    alias: &'a str,
    role: SessionRole,
    program: &'a std::path::Path,
    unit_id: crate::execution::UnitId,
    execution_generation: u64,
    unit: BodyPin,
    worktree: &'a std::path::Path,
    profile_digest: &'a str,
    governing_digest: &'a str,
    model: Option<&'a str>,
    effort: Option<&'a str>,
    context: BodyPin,
    revision: &'a str,
    source_versions: &'a BTreeMap<String, String>,
    artifact: Option<crate::execution::ArtifactId>,
    prepared_input_sha256: &'a str,
    origin: uuid::Uuid,
    operation: crate::execution::OperationId,
    pair: uuid::Uuid,
    allocated_session: SessionId,
    native_invocation: crate::execution::NativeInvocationId,
    locks: Vec<LockPin>,
}

fn next(version: u64) -> Result<u64> {
    version
        .checked_add(1)
        .filter(|v| *v <= i64::MAX as u64)
        .context("managed marker version exhausted")
}

fn unit(
    c: &Connection,
    facts: &AllocationFacts<'_>,
    original: &ExecutionUnit,
) -> Result<Body<ExecutionUnit>> {
    let raw = c.query_row(
        "SELECT CASE WHEN length(CAST(body AS BLOB))<=?2 THEN body END FROM execution_units WHERE id=?1",
        params![facts.unit_id.to_string(), UNIT_BYTES],
        |r| r.get::<_, Option<String>>(0),
    ).optional()?.flatten().context("managed marker Unit unavailable/over bound")?;
    let body = Body::<ExecutionUnit>::decode(raw, UNIT_BYTES)?;
    let u = body.parsed();
    // Compare the COMPLETE original capture, not just the readable scalar DTO.
    ensure!(
        encode(&serde_json::to_value(original)?, UNIT_BYTES)?
            == encode(&serde_json::to_value(u)?, UNIT_BYTES)?,
        "allocated Unit content changed"
    );
    let indexed = unit_index_matches(c, u, body.raw())?;
    ensure!(
        indexed
            && u.scope == *facts.scope
            && u.id == facts.unit_id
            && u.generation == facts.generation
            && u.owner_epoch == facts.epoch
            && u.version == facts.unit_version
            && u.provider == facts.provider
            && u.worktree == facts.path
            && u.profile_digest == facts.profile_digest
            && u.artifact_id == facts.artifact
            && u.state == UnitState::Preparing
            && u.native_effects_open
            && u.result_finalization_open
            && u.session_id.is_none()
            && u.work.is_none()
            && u.disposition == Disposition::Active,
        "managed marker Unit index/selection/lifecycle changed"
    );
    Ok(body)
}

fn unit_index_matches(c: &Connection, u: &ExecutionUnit, raw: &str) -> Result<bool> {
    Ok(c.query_row(
        "SELECT EXISTS(SELECT 1 FROM execution_units u JOIN task_execution t ON t.task_id=u.task_id WHERE u.id=?1 AND u.project_id=?2 AND u.goal_id=?3 AND u.task_id=?4 AND u.kind=?5 AND u.generation=?6 AND u.owner_epoch=?7 AND u.version=?8 AND u.native_effects_open=?9 AND u.result_finalization_open=?10 AND u.worktree=?11 AND u.branch IS ?12 AND u.body=?13 AND t.project_id=u.project_id AND t.goal_id=u.goal_id AND t.generation=u.generation)",
        params![u.id.to_string(), u.scope.project_id.to_string(),
            u.scope.goal_id.context("Unit Goal missing")?.to_string(),
            u.scope.task_id.context("Unit Task missing")?.to_string(),
            serde_json::to_value(u.kind)?.as_str().context("Unit kind invalid")?,
            u.generation,u.owner_epoch,u.version,u.native_effects_open,u.result_finalization_open,
            u.worktree.to_str().context("Unit path not UTF-8")?,u.branch,raw], |r|r.get(0),
    )?)
}

/// No helpers, effects, writes, owner opening or authority refresh. All original
/// Unit/scope/Context/Workflow/lock bodies share ONE selected-owner read snapshot.
pub(crate) fn plan_marker(
    owner: &RuntimeOwner,
    workflow_id: RecordId,
    allocation: &NativeAllocation,
) -> Result<ManagedMarkerPlan> {
    let facts = allocation.facts();
    ensure!(
        facts.state_path == owner.state_path()
            && facts.state_root == owner.state_root()
            && facts.instance_id == owner.instance_id()
            && facts.epoch == owner.epoch(),
        "selected Native allocation belongs to another actual owner"
    );
    let (before, unit, governing) = snapshot(owner, |tx| {
        let governing = tx.query_row(
            "SELECT project_version,goal_version,governing_digest FROM execution_context WHERE unit_id=?1",
            [facts.unit_id.to_string()],
            |r| Ok((r.get::<_,u64>(0)?,r.get::<_,u64>(1)?,r.get::<_,String>(2)?)),
        )?;
        Ok((
            read_scope(tx, owner, facts.scope)?,
            unit(tx, &facts, allocation.unit_snapshot())?,
            governing,
        ))
    })?;
    ensure!(
        governing.0 == before.project.parsed().version
            && governing.1 == before.goal.parsed().version
            && governing.2
                == crate::state::execution_governing_digest(
                    before.project.parsed(),
                    before.goal.parsed()
                )?,
        "managed marker original governing admission changed"
    );
    let saved = before
        .workflow
        .as_ref()
        .context("managed marker needs exactly one Workflow")?;
    ensure!(
        saved.parsed().id == workflow_id,
        "managed marker Workflow differs"
    );
    let context = before
        .context
        .as_ref()
        .context("managed marker Context missing")?;
    let cv = context.parsed();
    let task = before.task.parsed();
    let mut workflow: WorkflowSnapshot = serde_json::from_value(saved.parsed().data.clone())?;
    let index = workflow
        .active
        .context("managed marker active attempt missing")?;
    let attempt = workflow
        .history
        .get(index)
        .context("managed marker active attempt out of range")?;
    let phase = attempt.phase;
    let actor = phase.actor();
    let (role, kind, actor_name) = match actor {
        Actor::Executor => (SessionRole::Executor, InputKind::ContextPack, "executor"),
        Actor::Reviewer => (SessionRole::Reviewer, InputKind::ReviewBundle, "reviewer"),
        Actor::EvidencePort => anyhow::bail!("evidence phase cannot allocate a Native marker"),
    };
    ensure!(
        before.project.parsed().state == ProjectState::Registered
            && before.goal.parsed().state == GoalState::Running
            && !crate::state::task_terminal(task.state)
            && task.blockers.is_empty()
            && !workflow.finished
            && workflow.terminal_decision.is_none()
            && workflow.held_reason.is_none()
            && workflow.context_fresh
            && workflow.generation > 0
            && attempt.generation == workflow.generation
            && attempt.state == AttemptState::Running
            && !attempt.dispatch_started
            && attempt.session_id.is_none()
            && attempt.execution.is_none()
            && attempt.native_wait.is_none()
            && attempt.next_due.is_none()
            && attempt.completed_at.is_none()
            && attempt.unit.as_ref()
                == Some(&crate::execution::ManagedUnitRef::from(unit.parsed()))
            && unit.parsed().phase == phase.key()
            && facts.role == role
            && attempt.agent.as_deref() == Some(facts.alias)
            && (actor != Actor::Executor || task.executor == facts.alias)
            && (actor != Actor::Reviewer || task.reviewers.iter().any(|a| a == facts.alias)),
        "managed marker phase/actor/owner differs"
    );
    ensure!(
        facts.input.scope == *facts.scope
            && facts.input.kind == kind
            && cv.scope == *facts.scope
            && cv.version > 0
            && cv.version == task.context_version
            && cv.version == workflow.context_version
            && cv.version == attempt.context_version
            && cv.version == facts.input.version
            && cv.revision == facts.input.revision
            && task.revision.as_deref() == Some(cv.revision.as_str())
            && cv.revision == workflow.sources.revision
            && cv.revision == unit.parsed().base_sha
            && cv.source_hashes == facts.input.source_versions
            && serde_json::to_string(&cv.data)? == facts.input.payload
            && cv.data["phase"] == serde_json::to_value(phase)?
            && cv.data["workflow"] == serde_json::to_value(workflow.workflow)?
            && cv.data["generation"].as_u64() == Some(workflow.generation)
            && cv.data["budget"] == serde_json::to_value(&attempt.budget)?
            && cv.data["payload"].as_str() == Some(workflow.sources.payload.as_str())
            && workflow.sources.scope == *facts.scope
            && workflow.sources.artifact == facts.artifact,
        "managed marker exact Context/input differs"
    );
    let source_governing = cv
        .source_hashes
        .iter()
        .filter(|(k, _)| !k.starts_with("workflow:"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<BTreeMap<_, _>>();
    ensure!(
        source_governing == workflow.sources.source_versions,
        "managed marker governing/instruction source set differs"
    );
    ensure!(
        actor != Actor::Executor
            || (task.worktree.as_deref() == Some(facts.path)
                && task.branch == unit.parsed().branch),
        "executor prepared namespace differs"
    );
    ensure!(
        actor != Actor::Reviewer || facts.artifact.is_some(),
        "reviewer requires actual retained input artifact"
    );

    // The only planned initial marker projection: fixed timestamp/version and
    // state/phase, plus this attempt's dispatch flag. Binding never repeats it.
    let at = now_ms();
    let mut task_after = task.clone();
    task_after.state = phase.task_state();
    task_after.phase = Some(phase.key().into());
    task_after.version = next(task.version)?;
    task_after.updated_at = at;
    workflow.history[index].dispatch_started = true;
    let mut workflow_after = saved.parsed().clone();
    workflow_after.data = serde_json::to_value(&workflow)?;
    workflow_after.version = next(workflow_after.version)?;
    workflow_after.updated_at = at;
    let task_after = Body::<Task>::decode(serde_json::to_string(&task_after)?, BODY_BYTES)?;
    let workflow_after =
        Body::<Record>::decode(serde_json::to_string(&workflow_after)?, BODY_BYTES)?;
    let context_digest = context.digest(WORKFLOW_DOMAIN);
    let input_digest = native_result::digest(facts.input_bytes);
    let pin = |version, sha256| BodyPin { version, sha256 };
    let frame = Frame {
        canonical_body_recipe: "rrx.workflow-body-sha256/v1",
        instance: owner.instance_id(),
        epoch: owner.epoch(),
        scope: facts.scope,
        project: pin(
            before.project.parsed().version,
            before.project.digest(WORKFLOW_DOMAIN),
        ),
        goal: pin(
            before.goal.parsed().version,
            before.goal.digest(WORKFLOW_DOMAIN),
        ),
        task: pin(
            task_after.parsed().version,
            task_after.digest(WORKFLOW_DOMAIN),
        ),
        workflow_id,
        workflow: pin(
            workflow_after.parsed().version,
            workflow_after.digest(WORKFLOW_DOMAIN),
        ),
        workflow_generation: workflow.generation,
        attempt_index: index,
        phase: phase.key(),
        actor: actor_name,
        provider: facts.provider,
        alias: facts.alias,
        role,
        program: facts.program,
        unit_id: facts.unit_id,
        execution_generation: facts.generation,
        unit: pin(unit.parsed().version, unit.digest(WORKFLOW_DOMAIN)),
        worktree: facts.path,
        profile_digest: facts.profile_digest,
        governing_digest: &governing.2,
        model: facts.model,
        effort: facts.effort,
        context: pin(cv.version, context_digest.clone()),
        revision: &cv.revision,
        source_versions: &cv.source_hashes,
        artifact: facts.artifact,
        prepared_input_sha256: &input_digest,
        origin: facts.origin_id,
        operation: facts.operation_id,
        pair: facts.pair_id,
        allocated_session: facts.session_id,
        native_invocation: facts.invocation_id,
        locks: before
            .locks
            .iter()
            .map(|l| LockPin {
                id: l.parsed().id,
                version: l.parsed().version,
                sha256: l.digest(WORKFLOW_DOMAIN),
            })
            .collect(),
    };
    let frame = encode(&serde_json::to_value(frame)?, 4 * 1024 * 1024)?;
    let frame_digest = super::canonical::digest(WORKFLOW_DOMAIN, &frame);
    Ok(ManagedMarkerPlan {
        before,
        unit,
        governing,
        task_after,
        workflow_after,
        context_digest,
        input_digest,
        frame,
        frame_digest,
        operation: facts.operation_id,
        pair: facts.pair_id,
        allocated_session: facts.session_id,
        origin: facts.origin_id,
    })
}

impl ManagedMarkerPlan {
    /// Nongrant transaction recheck, to be composed only with actual retained
    /// capacity/Driver/allocation consumers. This cannot publish or start.
    pub(super) fn validate_current(&self, c: &Connection) -> Result<()> {
        self.before.validate_current(c)?;
        let u = self.unit.parsed();
        let current = unit_index_matches(c, u, self.unit.raw())?;
        let governing:bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM execution_context WHERE unit_id=?1 AND project_version=?2 AND goal_version=?3 AND governing_digest=?4)",
            params![u.id.to_string(),self.governing.0,self.governing.1,self.governing.2],|r|r.get(0),
        )?;
        ensure!(governing, "managed marker governing admission changed");
        ensure!(current, "managed marker original Unit changed");
        Ok(())
    }
}
