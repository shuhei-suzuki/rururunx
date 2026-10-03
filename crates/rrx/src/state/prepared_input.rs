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
    ensure!(
        context.scope == *scope
            && task.scope() == *scope
            && frame.version == context.version
            && task.context_version == context.version
            && frame.revision == context.revision,
        "native input differs from latest typed Task authority"
    );
    if context.data.get("task_pack").is_some() {
        crate::context_pack::workflow::context_artifact(context)?;
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
    validate(connection, &session.scope, context, &frame)?;
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
            ensure!(
                input.kind == InputKind::ContextPack,
                "typed Context requires ContextPack input"
            );
            validate(&self.connection, scope, &context, &Frame::input(input))
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
