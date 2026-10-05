//! Private source-CAS publication of exact standalone frame hashes; no prompt copy.
use super::context_pack::{latest_context, typed_pack};
use super::*;
use crate::adapter::{InputKind, PreparedInput};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
struct Frame {
    version: u64,
    revision: String,
    bytes: usize,
    sha256: String,
    sources: BTreeMap<String, String>,
}
fn payload_sha(payload: &str) -> String {
    format!("{:x}", Sha256::digest(payload.as_bytes()))
}
impl Frame {
    fn input(input: &PreparedInput) -> Self {
        Self {
            version: input.version,
            revision: input.revision.clone(),
            bytes: input.payload.len(),
            sha256: payload_sha(&input.payload),
            sources: input.source_versions.clone(),
        }
    }
    fn session(session: &Session) -> Result<Self> {
        let r = &session.recovery;
        Ok(Self {
            version: r["input_version"]
                .as_u64()
                .context("native input version missing")?,
            revision: r["input_revision"]
                .as_str()
                .context("native input revision missing")?
                .into(),
            bytes: usize::try_from(
                r["input_bytes"]
                    .as_u64()
                    .context("native input bytes missing")?,
            )?,
            sha256: r["input_sha256"]
                .as_str()
                .context("native input SHA256 missing")?
                .into(),
            sources: serde_json::from_value(r["source_versions"].clone())?,
        })
    }
}
pub(super) fn migrate_v5(tx: &Transaction<'_>) -> Result<()> {
    tx.execute_batch(include_str!("prepared-inputs.sql"))?;
    Ok(())
}
fn row(connection: &Connection, scope: &Scope, frame: &Frame) -> Result<Option<Frame>> {
    let result: Option<(String, u64, String)> = connection.query_row(
        "SELECT revision,input_bytes,source_versions FROM prepared_pack_inputs WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND context_version=?4 AND payload_sha256=?5",
        params![scope.project_id.to_string(), str_id(scope.goal_id), str_id(scope.task_id), frame.version, frame.sha256],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).optional()?;
    result
        .map(|(revision, bytes, sources)| {
            Ok(Frame {
                version: frame.version,
                revision,
                bytes: usize::try_from(bytes)?,
                sha256: frame.sha256.clone(),
                sources: serde_json::from_str(&sources)?,
            })
        })
        .transpose()
}
pub(super) fn publish(tx: &Transaction<'_>, input: &PreparedInput) -> Result<()> {
    let context =
        latest_context(tx, &input.scope)?.context("prepared input requires published context")?;
    ensure!(
        context.data["format"] == "rrx.task-pack.v1" && context.version == input.version,
        "standalone prepared input requires current Task pack"
    );
    let task: Task = read_tx(
        tx,
        "tasks",
        &input
            .scope
            .task_id
            .context("prepared input requires Task")?
            .to_string(),
    )?
    .context("unknown prepared Task")?;
    ensure!(
        task.context_version == input.version,
        "prepared input pointer changed"
    );
    let frame = Frame::input(input);
    if let Some(old) = row(tx, &input.scope, &frame)? {
        ensure!(old == frame, "prepared input hash metadata mismatch");
        return Ok(());
    }
    let variants: u64 = tx.query_row("SELECT COUNT(*) FROM prepared_pack_inputs WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND context_version=?4", params![input.scope.project_id.to_string(),str_id(input.scope.goal_id),str_id(input.scope.task_id),input.version],|r|r.get(0))?;
    ensure!(
        variants < 128,
        "prepared input variants exceed 128 for this context version"
    );
    tx.execute("INSERT INTO prepared_pack_inputs(project_id,goal_id,task_id,context_version,payload_sha256,revision,input_bytes,source_versions) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)", params![input.scope.project_id.to_string(),str_id(input.scope.goal_id),str_id(input.scope.task_id),input.version,frame.sha256,frame.revision,frame.bytes,serde_json::to_string(&frame.sources)?])?;
    Ok(())
}
fn validate(
    connection: &Connection,
    scope: &Scope,
    context: &ContextVersion,
    frame: &Frame,
    session: Option<&Session>,
) -> Result<()> {
    let body: String = connection.query_row(
        "SELECT body FROM tasks WHERE id=?1",
        [scope
            .task_id
            .context("typed input requires Task")?
            .to_string()],
        |r| r.get(0),
    )?;
    let task: Task = decode(body)?;
    let goal_body: String = connection.query_row(
        "SELECT body FROM goals WHERE id=?1",
        [task.goal_id.to_string()],
        |r| r.get(0),
    )?;
    let goal: Goal = decode(goal_body)?;
    let project_body: String = connection.query_row(
        "SELECT body FROM projects WHERE id=?1",
        [task.project_id.to_string()],
        |r| r.get(0),
    )?;
    let project: Project = decode(project_body)?;
    ensure!(
        project.state == ProjectState::Registered
            && matches!(
                goal.state,
                GoalState::Created | GoalState::Analyzing | GoalState::Running
            )
            && !task_terminal(task.state)
            && context.data["frozen_task_pack"] != true,
        "typed input owner is inactive or historical"
    );
    ensure!(
        context.scope == *scope
            && task.scope() == *scope
            && frame.version == context.version
            && task.context_version == context.version
            && frame.revision == context.revision,
        "native input differs from latest typed Task authority"
    );
    if crate::context_pack::workflow::is_phase_context(&context.data) {
        let artifact = crate::context_pack::workflow::context_artifact(context)?;
        ensure!(
            artifact.phase.actor() != crate::workflow::Actor::EvidencePort,
            "evidence phase cannot launch a native actor"
        );
        let body: Option<String> = connection.query_row(
            "SELECT body FROM records WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind='workflow' LIMIT 1",
            params![scope.project_id.to_string(),str_id(scope.goal_id),str_id(scope.task_id)],
            |r| r.get(0),
        ).optional()?;
        let record: Record = decode(body.context("typed phase has no owned Workflow")?)?;
        let workflow: crate::workflow::WorkflowSnapshot = serde_json::from_value(record.data)?;
        let attempt = workflow
            .active
            .and_then(|index| workflow.history.get(index))
            .context("typed phase has no active native attempt")?;
        ensure!(
            !workflow.finished
                && workflow.context_version == context.version
                && attempt.context_version == context.version
                && attempt.generation == workflow.generation
                && attempt.phase == artifact.phase
                && attempt.state == crate::workflow::AttemptState::Running
                && attempt.dispatch_started
                && session.is_none_or(|native| {
                    attempt.session_id.is_none_or(|owned| owned == native.id)
                        && attempt.agent.as_deref() == Some(native.agent.as_str())
                        && native.role
                            == match artifact.phase.actor() {
                                crate::workflow::Actor::Executor => SessionRole::Executor,
                                crate::workflow::Actor::Reviewer => SessionRole::Reviewer,
                                crate::workflow::Actor::EvidencePort => unreachable!(),
                            }
                }),
            "typed input differs from active native attempt"
        );
        ensure!(
            crate::context_pack::instruction_versions(&project, &goal, &task)?
                .iter()
                .all(|(key, value)| frame.sources.get(key) == Some(value)),
            "native phase input semantic authority changed"
        );
        let payload = context.data["payload"]
            .as_str()
            .context("typed phase payload missing")?;
        ensure!(
            frame.sha256 == payload_sha(payload)
                && frame.bytes == payload.len()
                && frame.sources == context.source_hashes,
            "native phase input differs from immutable complete rendered frame"
        );
    } else {
        ensure!(
            row(connection, scope, frame)?.as_ref() == Some(frame),
            "native input has no exact privately prepared frame authority"
        );
        let pack: crate::context_pack::TaskPack = serde_json::from_value(context.data.clone())?;
        ensure!(
            pack.authority_digest == crate::context_pack::authority(&project, &goal, &task)?
                && pack.task == crate::context_pack::projection(&task)?
                && pack.goal == crate::context_pack::projection(&goal)?,
            "native standalone pack semantic authority changed"
        );
        ensure!(
            frame.sources.get("checkpoint:head")
                == Some(&crate::context_pack::head_digest(pack.checkpoint.as_ref())),
            "native input checkpoint differs from pinned Task pack"
        );
    }
    super::context_pack::validate_checkpoint_source(
        connection,
        scope,
        frame.sources.get("checkpoint:head").map(String::as_str),
    )
}
pub(super) fn validate_session(
    connection: &Connection,
    session: &Session,
    context: &ContextVersion,
) -> Result<()> {
    let frame = Frame::session(session)?;
    validate(connection, &session.scope, context, &frame, Some(session))?;
    if session.recovery["dispatch_intent"]["consumed"] == true {
        ensure!(
            session.recovery["dispatch_intent"]["input_sha256"].as_str()
                == Some(frame.sha256.as_str()),
            "consumed dispatch differs from pinned native input SHA256"
        );
    }
    Ok(())
}
impl Store {
    /// Pure bounded check of actual request bytes against private published authority.
    /// Native adapters also preserve their private request-to-wire correspondence.
    pub fn validate_context_input(&self, scope: &Scope, input: &PreparedInput) -> Result<()> {
        ensure!(*scope == input.scope, "foreign prepared input scope");
        if let Some(context) = latest_context(&self.connection, scope)?
            && typed_pack(&context.data)
        {
            let expected = if crate::context_pack::workflow::is_phase_context(&context.data) {
                let phase: crate::workflow::Phase =
                    serde_json::from_value(context.data["phase"].clone())?;
                if phase.actor() == crate::workflow::Actor::Reviewer {
                    InputKind::ReviewBundle
                } else {
                    InputKind::ContextPack
                }
            } else {
                InputKind::ContextPack
            };
            ensure!(
                input.kind == expected,
                "typed input kind differs from owned phase"
            );
            validate(
                &self.connection,
                scope,
                &context,
                &Frame::input(input),
                None,
            )
        } else {
            super::context_pack::validate_checkpoint_source(
                &self.connection,
                scope,
                input
                    .source_versions
                    .get("checkpoint:head")
                    .map(String::as_str),
            )
        }
    }
}
