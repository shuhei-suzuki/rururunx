//! Context publication CAS and checkpoint append share one authoritative SQLite transaction.
use super::*;
use crate::context_pack::{Checkpoint, CheckpointRef};
use sha2::{Digest, Sha256};

pub(super) fn migrate_v4(tx: &Transaction<'_>) -> Result<()> {
    tx.execute_batch(include_str!("checkpoint-heads.sql"))?;
    let mut query=tx.prepare("SELECT body FROM records WHERE kind='checkpoint' AND json_extract(body,'$.data.format')='rrx.checkpoint.v1' ORDER BY project_id,goal_id,task_id,json_extract(body,'$.data.chain_version')")?;
    let mut previous: Option<(Record, Checkpoint)> = None;
    for body in query.query_map([], |r| r.get::<_, String>(0))? {
        let record: Record = decode(body?)?;
        let cp: Checkpoint = serde_json::from_value(record.data.clone())?;
        ensure!(
            record.version == 1 && record.scope == cp.scope && cp.scope.task_id.is_some(),
            "invalid typed checkpoint migration authority"
        );
        if let Some((record, old)) = &previous
            && old.scope == cp.scope
        {
            let reference = CheckpointRef {
                scope: record.scope.clone(),
                id: record.id,
                version: record.version,
                digest: checkpoint_digest(&record.data)?,
            };
            ensure!(
                cp.chain_version
                    == old
                        .chain_version
                        .checked_add(1)
                        .context("checkpoint chain overflow")?
                    && cp.previous.as_ref() == Some(&reference),
                "checkpoint migration chain invalid"
            );
        } else {
            ensure!(
                cp.chain_version == 1 && cp.previous.is_none(),
                "checkpoint migration missing origin"
            );
        }
        write_checkpoint_head(tx, &record)?;
        previous = Some((record, cp));
    }
    Ok(())
}
fn write_checkpoint_head(tx: &Transaction<'_>, record: &Record) -> Result<()> {
    tx.execute("INSERT INTO checkpoint_heads(project_id,goal_id,task_id,record_id) VALUES(?1,?2,?3,?4) ON CONFLICT(project_id,goal_id,task_id) DO UPDATE SET record_id=excluded.record_id",params![record.scope.project_id.to_string(),str_id(record.scope.goal_id),str_id(record.scope.task_id),record.id.to_string()])?;
    Ok(())
}
pub(super) fn validate_checkpoint_source(
    connection: &Connection,
    scope: &Scope,
    key: Option<&str>,
) -> Result<()> {
    if let Some(key) = key {
        ensure!(
            scope.task_id.is_some() && scope.goal_id.is_some(),
            "checkpoint source requires Task scope"
        );
        let current = checkpoint_head_record(connection, scope)?
            .map(|r| checkpoint_digest(&r.data))
            .transpose()?
            .unwrap_or_else(|| "none".into());
        ensure!(
            key == current,
            "checkpoint source changed before native reservation"
        );
    }
    Ok(())
}
pub(super) fn guard_context_checkpoint(
    tx: &Transaction<'_>,
    task: &Task,
    context: &ContextVersion,
) -> Result<()> {
    if context.data.get("task_pack").is_some() {
        let artifact = crate::context_pack::workflow::context_artifact(context)?;
        if context.data["frozen_task_pack"] == true {
            ensure!(
                task.state == TaskState::Completed
                    && context.data["phase"]
                        == serde_json::to_value(crate::workflow::Phase::Cleanup)?,
                "only finalized Cleanup may preserve historical capture guards"
            );
            ensure!(
                context.source_hashes.get("checkpoint:head")
                    == artifact.source_versions.get("checkpoint:head"),
                "frozen Cleanup checkpoint provenance changed"
            );
            // Terminal history preserves the admitted attempt, not a new launch.
            // Later checkpoint facts remain in their immutable journal chain.
            return Ok(());
        } else {
            pack_guard(tx, &context.scope, artifact.authority_versions)?;
        }
    }
    validate_checkpoint_source(
        tx,
        &context.scope,
        context
            .source_hashes
            .get("checkpoint:head")
            .map(String::as_str),
    )
}
/// A new irreversible Cleanup claim uses live checkpoint authority. Once the
/// operation is admitted, its observation and terminal frozen pack are historical.
pub(super) fn guard_irreversible_checkpoint(tx: &Transaction<'_>, record: &Record) -> Result<()> {
    let Some(index) = record.data["active"]
        .as_u64()
        .and_then(|i| usize::try_from(i).ok())
    else {
        return Ok(());
    };
    let next = &record.data["history"][index];
    let evaluating = serde_json::to_value(crate::workflow::AttemptState::Evaluating)?;
    let phase: crate::workflow::Phase = serde_json::from_value(next["phase"].clone())?;
    if !matches!(
        phase,
        crate::workflow::Phase::Pr
            | crate::workflow::Phase::MergeGate
            | crate::workflow::Phase::Cleanup
    ) || next["state"] != evaluating
    {
        return Ok(());
    }
    let previous = read_tx::<Record>(tx, "records", &record.id.to_string())?
        .context("irreversible claim requires existing workflow")?;
    if previous.data["active"].as_u64() == Some(index as u64)
        && previous.data["history"][index]["state"] == evaluating
    {
        return Ok(());
    }
    validate_checkpoint_source(
        tx,
        &record.scope,
        record.data["sources"]["source_versions"]["checkpoint:head"].as_str(),
    )
}
fn session_restore_sha256(session: &Session) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(session)?)
    ))
}
fn input_metadata_equal(old: &Session, new: &Session) -> bool {
    [
        "input_version",
        "input_revision",
        "input_bytes",
        "input_sha256",
        "source_versions",
    ]
    .iter()
    .all(|key| old.recovery[*key] == new.recovery[*key])
}
fn dispatch_consumed(session: &Session) -> bool {
    session.recovery["dispatch_intent"]["consumed"] == true
        || session.recovery["native_dispatch_unobserved"] == true
}
pub(super) fn guard_launch_checkpoint(tx: &Transaction<'_>, record: &Record) -> Result<()> {
    if record.kind != RecordKind::Session {
        return Ok(());
    }
    let session: Session = serde_json::from_value(record.data.clone())?;
    if session.scope.task_id.is_none() {
        return Ok(());
    }
    let previous = read_tx::<Record>(tx, "records", &record.id.to_string())?
        .map(|r| serde_json::from_value::<Session>(r.data))
        .transpose()?;
    let latest = latest_context(tx, &session.scope)?;
    let typed = latest.as_ref().is_some_and(|c| typed_pack(&c.data));
    let protected = typed
        || session.recovery["source_versions"]["checkpoint:head"].is_string()
        || previous
            .as_ref()
            .is_some_and(|old| old.recovery["source_versions"]["checkpoint:head"].is_string());
    let mut fresh = false;
    let mut restoring = false;
    if let Some(old) = &previous {
        fresh = session_terminal(old.state) && session.state == SessionState::Starting;
        if protected {
            ensure!(
                !session_terminal(old.state) || session_terminal(session.state) || fresh,
                "terminal native Session requires explicit fresh Starting continuation"
            );
            if fresh {
                let input_version = session.recovery["input_version"]
                    .as_u64()
                    .context("fresh continuation requires input version")?;
                ensure!(
                    input_version > old.recovery["input_version"].as_u64().unwrap_or(0),
                    "fresh continuation requires higher input version"
                );
                ensure!(
                    session.recovery["pre_dispatch_restore_sha256"].as_str()
                        == Some(session_restore_sha256(old)?.as_str())
                        && !dispatch_consumed(&session),
                    "fresh continuation requires exact terminal restore proof before dispatch"
                );
            } else {
                restoring = old.state == SessionState::Starting
                    && session_terminal(session.state)
                    && !dispatch_consumed(old)
                    && old.recovery["pre_dispatch_restore_sha256"].as_str()
                        == Some(session_restore_sha256(&session)?.as_str());
                ensure!(
                    input_metadata_equal(old, &session) || restoring,
                    "admitted native input metadata is immutable"
                );
                ensure!(
                    old.recovery["pre_dispatch_restore_sha256"]
                        == session.recovery["pre_dispatch_restore_sha256"]
                        || restoring,
                    "native terminal restore proof is immutable outside fresh continuation"
                );
            }
        }
    } else if protected {
        ensure!(
            session_terminal(session.state)
                || matches!(
                    session.state,
                    SessionState::Starting | SessionState::Running
                ),
            "new native Session requires initial Starting or Running admission"
        );
        ensure!(
            session.recovery["pre_dispatch_restore_sha256"].is_null(),
            "terminal restore proof requires persisted prior terminal Session"
        );
    }
    // The dispatch CAS precedes the native wire and Running acknowledgement.
    // A newly consumed intent must use live authority even if the Session state
    // remains Starting or a caller publishes it from a later observation.
    let new_dispatch = !restoring
        && dispatch_consumed(&session)
        && previous.as_ref().is_none_or(|old| {
            !dispatch_consumed(old)
                || old.recovery["dispatch_intent"] != session.recovery["dispatch_intent"]
        });
    if !matches!(
        session.state,
        SessionState::Starting | SessionState::Running
    ) && !new_dispatch
    {
        return Ok(());
    }
    if previous
        .as_ref()
        .is_some_and(|old| old.state != SessionState::Starting)
        && !fresh
        && !new_dispatch
    {
        return Ok(());
    }
    let key = session.recovery["source_versions"]["checkpoint:head"].as_str();
    if typed {
        super::prepared_input::validate_session(
            tx,
            &session,
            latest.as_ref().expect("typed context exists"),
        )?;
    }
    validate_checkpoint_source(tx, &session.scope, key)
}
pub(super) fn guard_checkpoint_write(tx: &Transaction<'_>, record: &Record) -> Result<()> {
    let previous = read_tx::<Record>(tx, "records", &record.id.to_string())?;
    ensure!(
        !(record.kind == RecordKind::Checkpoint
            && (record.data["format"] == "rrx.checkpoint.v1"
                || previous
                    .as_ref()
                    .is_some_and(|r| r.data["format"] == "rrx.checkpoint.v1"))),
        "typed checkpoints are immutable and require the dedicated append transaction"
    );
    Ok(())
}
fn checkpoint_head_record(connection: &Connection, scope: &Scope) -> Result<Option<Record>> {
    let body:Option<String>=connection.query_row("SELECT r.body FROM checkpoint_heads h JOIN records r ON r.id=h.record_id AND r.project_id=h.project_id AND r.goal_id=h.goal_id AND r.task_id=h.task_id WHERE h.project_id=?1 AND h.goal_id=?2 AND h.task_id=?3",params![scope.project_id.to_string(),str_id(scope.goal_id),str_id(scope.task_id)],|r|r.get(0)).optional()?;
    body.map(decode).transpose()
}
fn own_checkpoint_tx(
    tx: &Transaction<'_>,
    scope: &Scope,
    reference: Option<&CheckpointRef>,
) -> Result<()> {
    ensure!(
        reference.is_none_or(|r| r.scope == *scope),
        "foreign own checkpoint reference"
    );
    let current = checkpoint_head_record(tx, scope)?
        .map(|r| {
            Ok::<_, anyhow::Error>(CheckpointRef {
                scope: r.scope,
                id: r.id,
                version: r.version,
                digest: checkpoint_digest(&r.data)?,
            })
        })
        .transpose()?;
    ensure!(
        current.as_ref() == reference,
        "checkpoint reference no longer current"
    );
    Ok(())
}
pub(super) fn typed_pack(data: &Value) -> bool {
    matches!(
        data["format"].as_str(),
        Some("rrx.task-pack.v1" | "rrx.goal-pack.v1")
    ) || data["task_pack"]["format"] == "rrx.phase-pack.v1"
}
pub(super) fn latest_context(
    connection: &Connection,
    scope: &Scope,
) -> Result<Option<ContextVersion>> {
    let owner = context_owner(scope)?;
    let body:Option<String>=connection.query_row("SELECT body FROM context_versions WHERE project_id=?1 AND owner=?2 ORDER BY version DESC LIMIT 1",params![scope.project_id.to_string(),owner],|r|r.get(0)).optional()?;
    body.map(decode).transpose()
}
pub(super) fn guard_context_write(tx: &Transaction<'_>, context: &ContextVersion) -> Result<()> {
    ensure!(
        !typed_pack(&context.data)
            && !latest_context(tx, &context.scope)?.is_some_and(|c| typed_pack(&c.data)),
        "typed pack context requires the owned publication transaction"
    );
    Ok(())
}
pub(super) fn guard_pack_pointer(
    tx: &Transaction<'_>,
    scope: &Scope,
    old: u64,
    next: u64,
) -> Result<()> {
    ensure!(
        old == next || !latest_context(tx, scope)?.is_some_and(|c| typed_pack(&c.data)),
        "typed pack pointer requires the owned publication transaction"
    );
    Ok(())
}
impl Store {
    /// Stable lowerhex SHA256 of the exact prior terminal Session, used only for
    /// bounded pre-dispatch restoration of a fresh continuation.
    pub fn session_restore_sha256(session: &Session) -> Result<String> {
        session_restore_sha256(session)
    }
    /// Pure scoped check; callers hold SharedStore only for this bounded SQL lookup.
    /// Legacy inputs without Issue 19's key retain their original contract.
    pub fn validate_checkpoint_source(
        &self,
        scope: &Scope,
        sources: &std::collections::BTreeMap<String, String>,
    ) -> Result<()> {
        validate_checkpoint_source(
            &self.connection,
            scope,
            sources.get("checkpoint:head").map(String::as_str),
        )
    }
    pub(crate) fn pack_checkpoint_head(&self, scope: &Scope) -> Result<Option<CheckpointRef>> {
        checkpoint_head_record(&self.connection, scope)?
            .map(|r| {
                Ok(CheckpointRef {
                    scope: r.scope,
                    id: r.id,
                    version: r.version,
                    digest: checkpoint_digest(&r.data)?,
                })
            })
            .transpose()
    }
    pub(crate) fn audit_pack_preparation(
        &mut self,
        scope: &Scope,
        expected: [u64; 3],
        checkpoint: Option<&CheckpointRef>,
        data: Value,
        prepared: Option<&crate::adapter::PreparedInput>,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        pack_guard(&tx, scope, expected)?;
        own_checkpoint_tx(&tx, scope, checkpoint)?;
        if let Some(input) = prepared {
            ensure!(*scope == input.scope, "foreign prepared frame");
            super::prepared_input::publish(&tx, input)?;
        }
        append_event(&tx, scope, "context.pack.prepared", data)?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn publish_context_pack(
        &mut self,
        binding: &Scope,
        expected: [u64; 3],
        context: &ContextVersion,
        task_versions: &[(TaskId, u64)],
    ) -> Result<()> {
        ensure!(
            context.scope == *binding && binding.task_id.is_some(),
            "foreign pack publication"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (_, mut goal, mut task) = pack_guard(&tx, binding, expected)?;
        if context.scope.task_id.is_some() {
            ensure!(
                !task_terminal(task.state),
                "terminal Task cannot publish new pack"
            );
        }
        ensure!(
            !goal_terminal(goal.state),
            "terminal Goal cannot publish new pack"
        );
        let checkpoint = context
            .data
            .get("checkpoint")
            .filter(|v| !v.is_null())
            .cloned()
            .map(serde_json::from_value::<CheckpointRef>)
            .transpose()?;
        own_checkpoint_tx(&tx, binding, checkpoint.as_ref())?;
        pack_idle(&tx, &context.scope)?;
        for (id, version) in task_versions {
            let t: Task =
                read_tx(&tx, "tasks", &id.to_string())?.context("missing referenced Task")?;
            ensure!(
                t.project_id == binding.project_id
                    && Some(t.goal_id) == binding.goal_id
                    && t.version == *version,
                "Goal Task context changed during publication"
            );
        }
        let owner = context_owner(&context.scope)?;
        let latest:Option<String>=tx.query_row("SELECT body FROM context_versions WHERE project_id=?1 AND owner=?2 ORDER BY version DESC LIMIT 1",
            params![context.scope.project_id.to_string(),owner],|r|r.get(0)).optional()?;
        let latest: Option<ContextVersion> = latest.map(decode).transpose()?;
        let pointer = if context.scope.task_id.is_some() {
            task.context_version
        } else {
            goal.context_version
        };
        if let Some(old) = &latest
            && old.version == context.version
        {
            ensure!(
                serde_json::to_value(old)? == serde_json::to_value(context)?
                    && pointer == context.version,
                "context reuse must match exact published version/pointer"
            );
            tx.commit()?;
            return Ok(());
        }
        let next = latest.as_ref().map_or(Ok(1), |c| {
            c.version.checked_add(1).context("pack version overflow")
        })?;
        ensure!(
            context.version == next && !context.revision.is_empty(),
            "pack version must be consecutive with explicit revision"
        );
        tx.execute("INSERT INTO context_versions(project_id,goal_id,task_id,owner,version,body) VALUES(?1,?2,?3,?4,?5,?6)",
            params![context.scope.project_id.to_string(),str_id(context.scope.goal_id),str_id(context.scope.task_id),owner,context.version,serde_json::to_string(context)?])?;
        if context.scope.task_id.is_some() {
            let prior = task.version;
            task.context_version = context.version;
            bump(&mut task.version)?;
            task.updated_at = now_ms();
            let body = serde_json::to_string(&task)?;
            write_snapshot(
                &tx,
                "tasks",
                &task.id.to_string(),
                prior,
                "",
                params![],
                &body,
                task.version,
            )?;
            append_event(
                &tx,
                &task.scope(),
                "task.saved",
                json!({"version":task.version,"context_version":task.context_version}),
            )?;
        } else {
            let prior = goal.version;
            goal.context_version = context.version;
            bump(&mut goal.version)?;
            goal.updated_at = now_ms();
            let body = serde_json::to_string(&goal)?;
            write_snapshot(
                &tx,
                "goals",
                &goal.id.to_string(),
                prior,
                "",
                params![],
                &body,
                goal.version,
            )?;
            append_event(
                &tx,
                &goal.scope(),
                "goal.saved",
                json!({"version":goal.version,"context_version":goal.context_version}),
            )?;
        }
        append_event(
            &tx,
            &context.scope,
            "context.created",
            json!({"version":context.version,"revision":context.revision,"format":context.data["format"]}),
        )?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn append_pack_checkpoint(
        &mut self,
        scope: &Scope,
        expected: [u64; 3],
        session: SessionId,
        session_version: u64,
        previous: Option<&CheckpointRef>,
        record: &mut Record,
    ) -> Result<()> {
        ensure!(
            record.kind == RecordKind::Checkpoint && record.scope == *scope && record.version == 0,
            "checkpoint must be a new exact-scoped record"
        );
        let checkpoint: Checkpoint = serde_json::from_value(record.data.clone())?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (_, goal, task) = pack_guard(&tx, scope, expected)?;
        ensure!(
            !goal_terminal(goal.state) && !task_terminal(task.state),
            "terminal work cannot append checkpoint"
        );
        let native: Record =
            read_tx(&tx, "records", &session.to_string())?.context("missing checkpoint Session")?;
        let session: Session = serde_json::from_value(native.data)?;
        ensure!(
            native.kind == RecordKind::Session
                && native.version == session_version
                && native.scope == *scope
                && session.scope == *scope
                && matches!(
                    session.role,
                    SessionRole::Executor | SessionRole::Consultant
                )
                && task.worktree.as_ref() == Some(&session.worktree),
            "checkpoint Session changed/foreign"
        );
        let latest = checkpoint_head_record(&tx, scope)?;
        match (previous, latest) {
            (None, None) => ensure!(
                checkpoint.chain_version == 1 && checkpoint.first_sequence == 1,
                "invalid first checkpoint"
            ),
            (Some(reference), Some(old)) => {
                ensure!(
                    reference.scope == *scope
                        && reference.id == old.id
                        && reference.version == old.version
                        && checkpoint_digest(&old.data)? == reference.digest,
                    "incremental checkpoint no longer current"
                );
                let old: Checkpoint = serde_json::from_value(old.data)?;
                ensure!(
                    checkpoint.chain_version
                        == old
                            .chain_version
                            .checked_add(1)
                            .context("checkpoint overflow")?
                        && checkpoint.first_sequence
                            == old
                                .last_sequence
                                .checked_add(1)
                                .context("sequence overflow")?,
                    "checkpoint chain not consecutive"
                );
            }
            _ => bail!("checkpoint predecessor no longer current"),
        }
        ensure!(
            read_tx::<Record>(&tx, "records", &record.id.to_string())?.is_none(),
            "checkpoint cannot overwrite existing record"
        );
        let mut next = record.clone();
        next.version = 1;
        next.updated_at = now_ms();
        let body = serde_json::to_string(&next)?;
        tx.execute("INSERT INTO records(id,kind,project_id,goal_id,task_id,version,body) VALUES(?1,'checkpoint',?2,?3,?4,1,?5)",
            params![next.id.to_string(),scope.project_id.to_string(),str_id(scope.goal_id),str_id(scope.task_id),body])?;
        write_checkpoint_head(&tx, &next)?;
        append_event(
            &tx,
            scope,
            "checkpoint.saved",
            json!({"id":next.id,"version":1,"session":session.id,"chain_version":checkpoint.chain_version,
            "first_sequence":checkpoint.first_sequence,"last_sequence":checkpoint.last_sequence,"input_digest":checkpoint.input_digest,
            "retained":checkpoint.retained.len(),"recent":checkpoint.recent.len(),"recent_bytes":checkpoint.recent_bytes,
            "omitted_transient":checkpoint.omitted_transient,"omitted_first_sequence":checkpoint.omitted_first_sequence,"omitted_last_sequence":checkpoint.omitted_last_sequence,"omitted_digest":checkpoint.omitted_digest,"measured_tokens":Value::Null}),
        )?;
        tx.commit()?;
        *record = next;
        Ok(())
    }
}
fn checkpoint_digest(value: &Value) -> Result<String> {
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(value)?)
    ))
}
fn pack_guard(
    tx: &Transaction<'_>,
    scope: &Scope,
    expected: [u64; 3],
) -> Result<(Project, Goal, Task)> {
    validate_scope(scope)?;
    let gid = scope.goal_id.context("pack requires Goal")?;
    let tid = scope.task_id.context("pack requires Task")?;
    let project: Project =
        read_tx(tx, "projects", &scope.project_id.to_string())?.context("unknown Project")?;
    let goal: Goal = read_tx(tx, "goals", &gid.to_string())?.context("unknown Goal")?;
    let task: Task = read_tx(tx, "tasks", &tid.to_string())?.context("unknown Task")?;
    ensure!(
        project.state == ProjectState::Registered,
        "inactive Project cannot publish context/checkpoint"
    );
    ensure!(
        goal.project_id == project.id && task.scope() == *scope,
        "foreign pack transaction scope"
    );
    for ((table, id), version, actual) in [
        (
            ("projects", project.id.to_string()),
            expected[0],
            project.version,
        ),
        (("goals", goal.id.to_string()), expected[1], goal.version),
        (("tasks", task.id.to_string()), expected[2], task.version),
    ] {
        if version != actual {
            bail!(StateGuardError::SnapshotChanged {
                table: table.into(),
                id,
                expected: version
            });
        }
    }
    Ok((project, goal, task))
}
fn pack_idle(tx: &Transaction<'_>, scope: &Scope) -> Result<()> {
    if scope.task_id.is_some() {
        let owned:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind='workflow')",
        params![scope.project_id.to_string(),str_id(scope.goal_id),str_id(scope.task_id)],|r|r.get(0))?;
        ensure!(
            !owned,
            "workflow owns Task context publication; use PhaseContext port"
        );
    }
    let mut statement=tx.prepare("SELECT body FROM records WHERE project_id=?1 AND goal_id=?2 AND task_id IS ?3 AND (kind='session' OR kind='worktree_lock')")?;
    for row in statement.query_map(
        params![
            scope.project_id.to_string(),
            str_id(scope.goal_id),
            str_id(scope.task_id)
        ],
        |r| r.get::<_, String>(0),
    )? {
        let record: Record = decode(row?)?;
        match record.kind {
            RecordKind::Session => {
                let s: Session = serde_json::from_value(record.data)?;
                ensure!(
                    session_terminal(s.state),
                    "live/Lost Session launch context cannot be rewritten"
                );
            }
            RecordKind::WorktreeLock => {
                let lock: crate::git::WorktreeLock = serde_json::from_value(record.data)?;
                ensure!(!lock.active, "locked context cannot publish");
            }
            _ => {}
        }
    }
    Ok(())
}

fn goal_pack_tasks(connection: &Connection, scope: &Scope) -> Result<Vec<Task>> {
    ensure!(
        scope.goal_id.is_some() && scope.task_id.is_none(),
        "exact Goal scope required"
    );
    let mut query = connection.prepare(
        "SELECT body FROM tasks WHERE project_id=?1 AND goal_id=?2 ORDER BY id LIMIT 129",
    )?;
    let tasks: Vec<Task> = query
        .query_map(
            params![scope.project_id.to_string(), str_id(scope.goal_id)],
            |r| r.get::<_, String>(0),
        )?
        .map(|r| decode(r?))
        .collect::<Result<_>>()?;
    ensure!(tasks.len() <= 128, "Goal Task set exceeds pack limit");
    ensure!(
        tasks
            .iter()
            .all(|t| t.project_id == scope.project_id && Some(t.goal_id) == scope.goal_id),
        "foreign Goal Task membership"
    );
    Ok(tasks)
}
impl Store {
    pub(crate) fn goal_pack_tasks(&self, scope: &Scope) -> Result<Vec<Task>> {
        goal_pack_tasks(&self.connection, scope)
    }
    pub(crate) fn publish_goal_context_pack(
        &mut self,
        expected: [u64; 2],
        context: &ContextVersion,
        task_versions: &[(TaskId, u64)],
    ) -> Result<()> {
        ensure!(
            context.scope.task_id.is_none() && context.scope.goal_id.is_some(),
            "Goal publication requires exact Goal scope"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let p: Project = read_tx(&tx, "projects", &context.scope.project_id.to_string())?
            .context("unknown Project")?;
        let mut g: Goal = read_tx(&tx, "goals", &context.scope.goal_id.unwrap().to_string())?
            .context("unknown Goal")?;
        ensure!(
            p.version == expected[0]
                && g.version == expected[1]
                && p.state == ProjectState::Registered
                && g.scope() == context.scope
                && !goal_terminal(g.state),
            "Goal publication authority changed/inactive"
        );
        pack_idle(&tx, &context.scope)?;
        let current = goal_pack_tasks(&tx, &context.scope)?;
        ensure!(
            current
                .iter()
                .map(|t| (t.id, t.version))
                .collect::<Vec<_>>()
                == task_versions,
            "Goal Task membership/version changed during publication"
        );
        for (id, version) in task_versions {
            let t: Task =
                read_tx(&tx, "tasks", &id.to_string())?.context("missing referenced Task")?;
            ensure!(
                t.project_id == p.id && t.goal_id == g.id && t.version == *version,
                "Goal Task summary changed during publication"
            );
        }
        let owner = context_owner(&context.scope)?;
        let latest:Option<String>=tx.query_row("SELECT body FROM context_versions WHERE project_id=?1 AND owner=?2 ORDER BY version DESC LIMIT 1",params![p.id.to_string(),owner],|r|r.get(0)).optional()?;
        let latest: Option<ContextVersion> = latest.map(decode).transpose()?;
        if let Some(old) = &latest
            && old.version == context.version
        {
            ensure!(
                serde_json::to_value(old)? == serde_json::to_value(context)?
                    && g.context_version == context.version,
                "Goal context reuse mismatch"
            );
            tx.commit()?;
            return Ok(());
        }
        ensure!(
            context.version
                == latest.as_ref().map_or(Ok(1), |c| c
                    .version
                    .checked_add(1)
                    .context("Goal context overflow"))?
                && !context.revision.is_empty(),
            "Goal context must be consecutive"
        );
        tx.execute("INSERT INTO context_versions(project_id,goal_id,task_id,owner,version,body) VALUES(?1,?2,NULL,?3,?4,?5)",params![p.id.to_string(),g.id.to_string(),owner,context.version,serde_json::to_string(context)?])?;
        let expected_version = g.version;
        g.context_version = context.version;
        // Pointer-only publication changes its independent ContextVersion head;
        // it must not invalidate admitted Tasks' Goal lifecycle authority.
        g.updated_at = now_ms();
        write_snapshot(
            &tx,
            "goals",
            &g.id.to_string(),
            expected_version,
            "",
            params![],
            &serde_json::to_string(&g)?,
            g.version,
        )?;
        append_event(
            &tx,
            &g.scope(),
            "goal.context_updated",
            json!({"version":g.version,"context_version":g.context_version}),
        )?;
        append_event(
            &tx,
            &context.scope,
            "context.created",
            json!({"version":context.version,"revision":context.revision,"format":context.data["format"]}),
        )?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture {
        _temp: tempfile::TempDir,
        store: Store,
        db: std::path::PathBuf,
        p: Project,
        g: Goal,
        t: Task,
    }
    impl Fixture {
        fn new() -> Self {
            let temp = tempfile::tempdir().unwrap();
            let db = temp.path().join("state.db");
            let mut store = Store::open(&db).unwrap();
            let mut p = Project::new(
                "one".into(),
                temp.path().join("repo"),
                "repo:one".into(),
                "main".into(),
            );
            store.put_project(&mut p).unwrap();
            let mut g = Goal::new(
                p.id,
                "Goal".into(),
                vec![CompletionCriterion {
                    id: "done".into(),
                    description: "verified".into(),
                    evidence: None,
                    satisfied: false,
                }],
            );
            store.put_goal(&mut g).unwrap();
            let mut t = Task::new(p.id, g.id, "Task".into(), "fake".into());
            store.put_task(&mut t).unwrap();
            Self {
                _temp: temp,
                store,
                db,
                p,
                g,
                t,
            }
        }
        fn context(&self, scope: Scope) -> ContextVersion {
            ContextVersion {
                scope,
                version: 1,
                revision: "exact-head".into(),
                source_hashes: Default::default(),
                data: json!({"format":"fixture"}),
            }
        }
    }
    #[test]
    fn task_pack_atomic_cas_and_audit_failure_leave_no_partial_publication() {
        let mut f = Fixture::new();
        let scope = f.t.scope();
        let c = f.context(scope.clone());
        let expected = [f.p.version, f.g.version, f.t.version];
        let mut other = Store::open(&f.db).unwrap();
        f.t.next_action = Some("new instruction".into());
        other.put_task(&mut f.t).unwrap();
        let before = f.store.events(&scope, 0, 100).unwrap().len();
        assert!(
            f.store
                .publish_context_pack(&scope, expected, &c, &[])
                .is_err()
        );
        assert!(f.store.context(&scope, None).unwrap().is_none());
        assert_eq!(f.store.task(f.t.id).unwrap().unwrap().context_version, 0);
        assert_eq!(f.store.events(&scope, 0, 100).unwrap().len(), before);
        let expected = [f.p.version, f.g.version, f.t.version];
        f.store.connection.execute_batch("CREATE TRIGGER fail_context_audit BEFORE INSERT ON audit WHEN NEW.kind='context.created' BEGIN SELECT RAISE(ABORT,'injected context journal failure'); END;").unwrap();
        assert!(
            f.store
                .publish_context_pack(&scope, expected, &c, &[])
                .is_err()
        );
        assert!(f.store.context(&scope, None).unwrap().is_none());
        assert_eq!(f.store.task(f.t.id).unwrap().unwrap().version, f.t.version);
        assert_eq!(f.store.task(f.t.id).unwrap().unwrap().context_version, 0);
        assert_eq!(f.store.events(&scope, 0, 100).unwrap().len(), before);
        f.store
            .connection
            .execute_batch("DROP TRIGGER fail_context_audit;")
            .unwrap();
        f.store
            .publish_context_pack(&scope, expected, &c, &[])
            .unwrap();
        let saved = f.store.task(f.t.id).unwrap().unwrap();
        assert_eq!(saved.context_version, 1);
        assert_eq!(saved.version, f.t.version + 1);
        f.store
            .publish_context_pack(&scope, [f.p.version, f.g.version, saved.version], &c, &[])
            .unwrap();
        assert_eq!(
            f.store.task(f.t.id).unwrap().unwrap().version,
            saved.version
        );
    }
    #[test]
    fn preparation_audit_rechecks_checkpoint_head_without_task_version_change() {
        let mut f = Fixture::new();
        let scope = f.t.scope();
        let expected = [f.p.version, f.g.version, f.t.version];
        let insert = |store: &Store, chain: u64| {
            let mut r = Record::new(
                scope.clone(),
                RecordKind::Checkpoint,
                json!({"format":"rrx.checkpoint.v1","chain_version":chain}),
            );
            r.version = 1;
            store.connection.execute("INSERT INTO records(id,project_id,goal_id,task_id,kind,version,body) VALUES(?1,?2,?3,?4,'checkpoint',1,?5)",params![r.id.to_string(),scope.project_id.to_string(),str_id(scope.goal_id),str_id(scope.task_id),serde_json::to_string(&r).unwrap()]).unwrap();
            let tx = store.connection.unchecked_transaction().unwrap();
            write_checkpoint_head(&tx, &r).unwrap();
            tx.commit().unwrap();
        };
        // Raw immutable rows isolate the final transaction from earlier service gates.
        insert(&f.store, 1);
        let old = f.store.pack_checkpoint_head(&scope).unwrap().unwrap();
        f.store
            .audit_pack_preparation(&scope, expected, Some(&old), json!({"control":true}), None)
            .unwrap();
        insert(&f.store, 2);
        let before = f.store.events(&scope, 0, 100).unwrap().len();
        assert!(
            f.store
                .audit_pack_preparation(&scope, expected, Some(&old), json!({"stale":true}), None)
                .is_err()
        );
        assert_eq!(f.store.events(&scope, 0, 100).unwrap().len(), before);
        assert_eq!(f.store.task(f.t.id).unwrap().unwrap().version, f.t.version);
        let current = f.store.pack_checkpoint_head(&scope).unwrap().unwrap();
        f.store
            .audit_pack_preparation(
                &scope,
                expected,
                Some(&current),
                json!({"current":true}),
                None,
            )
            .unwrap();
        assert_eq!(f.store.events(&scope, 0, 100).unwrap().len(), before + 1);
    }
    #[test]
    fn goal_publication_rechecks_new_task_membership_without_goal_version_change() {
        let mut f = Fixture::new();
        let scope = f.g.scope();
        let context = f.context(scope.clone());
        let expected = [f.p.version, f.g.version];
        let captured = vec![(f.t.id, f.t.version)];
        let mut other = Store::open(&f.db).unwrap();
        let mut added = Task::new(f.p.id, f.g.id, "New unplanned Task".into(), "fake".into());
        other.put_task(&mut added).unwrap();
        assert_eq!(other.goal(f.g.id).unwrap().unwrap().version, f.g.version);
        let before = f.store.events(&scope, 0, 100).unwrap().len();
        assert!(
            f.store
                .publish_goal_context_pack(expected, &context, &captured)
                .is_err()
        );
        assert!(f.store.context(&scope, None).unwrap().is_none());
        assert_eq!(f.store.events(&scope, 0, 100).unwrap().len(), before);
        let current = f
            .store
            .goal_pack_tasks(&scope)
            .unwrap()
            .iter()
            .map(|t| (t.id, t.version))
            .collect::<Vec<_>>();
        f.store
            .publish_goal_context_pack(expected, &context, &current)
            .unwrap();
        assert_eq!(f.store.goal(f.g.id).unwrap().unwrap().context_version, 1);
    }
    #[test]
    fn goal_context_head_is_independent_from_semantic_version_and_has_one_winner() {
        let mut f = Fixture::new();
        let scope = f.g.scope();
        let mut context = f.context(scope.clone());
        context.data = json!({"format":"rrx.goal-pack.v1","observation":"first"});
        let expected = [f.p.version, f.g.version];
        let captured = vec![(f.t.id, f.t.version)];
        f.store
            .publish_goal_context_pack(expected, &context, &captured)
            .unwrap();
        let mut other = Store::open(&f.db).unwrap();
        let mut contender = context.clone();
        contender.data["observation"] = json!("different concurrent content");
        assert!(
            other
                .publish_goal_context_pack(expected, &contender, &captured)
                .is_err()
        );
        assert_eq!(
            other.context(&scope, None).unwrap().unwrap().data,
            context.data
        );
        let stored = other.goal(f.g.id).unwrap().unwrap();
        assert_eq!(stored.version, f.g.version);
        assert_eq!(stored.context_version, 1);
        f.g.blockers.push("stale pointer rollback".into());
        assert!(other.put_goal(&mut f.g).is_err());
        let mut semantic = stored;
        semantic.blockers.push("semantic authority mutation".into());
        other.put_goal(&mut semantic).unwrap();
        assert_eq!(semantic.version, expected[1] + 1);
        context.version = 2;
        assert!(
            f.store
                .publish_goal_context_pack(expected, &context, &captured)
                .is_err()
        );
    }
    #[test]
    fn goal_pack_transaction_rechecks_summary_activity_and_goal_session() {
        let mut f = Fixture::new();
        let scope = f.g.scope();
        let c = f.context(scope.clone());
        let expected = [f.p.version, f.g.version];
        let mut other = Store::open(&f.db).unwrap();
        let old_task_version = f.t.version;
        f.t.next_action = Some("new Task summary".into());
        other.put_task(&mut f.t).unwrap();
        assert!(
            f.store
                .publish_goal_context_pack(expected, &c, &[(f.t.id, old_task_version)])
                .is_err()
        );
        assert!(f.store.context(&scope, None).unwrap().is_none());
        assert_eq!(f.store.goal(f.g.id).unwrap().unwrap().context_version, 0);
        let mut session = Session {
            id: SessionId::new(),
            scope: scope.clone(),
            agent: "fixture".into(),
            provider: "fixture".into(),
            role: SessionRole::Consultant,
            native_ref: None,
            pid: None,
            worktree: f.p.root.clone(),
            state: SessionState::Lost,
            model: None,
            effort: None,
            recovery: json!({}),
            started_at: now_ms(),
        };
        let sv = other.put_session(&session, 0).unwrap();
        assert!(
            f.store
                .publish_goal_context_pack(expected, &c, &[(f.t.id, f.t.version)])
                .is_err()
        );
        session.state = SessionState::Stopped;
        other.put_session(&session, sv).unwrap();
        f.p.state = ProjectState::Blocked;
        f.p.blocked_reason = Some("source missing".into());
        other.put_project(&mut f.p).unwrap();
        assert!(
            f.store
                .publish_goal_context_pack([f.p.version, f.g.version], &c, &[(f.t.id, f.t.version)])
                .is_err()
        );
        assert!(f.store.context(&scope, None).unwrap().is_none());
        f.p.state = ProjectState::Registered;
        f.p.blocked_reason = None;
        other.put_project(&mut f.p).unwrap();
        f.store.connection.execute_batch("CREATE TRIGGER fail_goal_context_audit BEFORE INSERT ON audit WHEN NEW.kind='context.created' BEGIN SELECT RAISE(ABORT,'injected context journal failure'); END;").unwrap();
        assert!(
            f.store
                .publish_goal_context_pack([f.p.version, f.g.version], &c, &[(f.t.id, f.t.version)])
                .is_err()
        );
        assert!(f.store.context(&scope, None).unwrap().is_none());
        assert_eq!(f.store.goal(f.g.id).unwrap().unwrap().context_version, 0);
        f.store
            .connection
            .execute_batch("DROP TRIGGER fail_goal_context_audit;")
            .unwrap();
        f.store
            .publish_goal_context_pack([f.p.version, f.g.version], &c, &[(f.t.id, f.t.version)])
            .unwrap();
        assert_eq!(f.store.goal(f.g.id).unwrap().unwrap().context_version, 1);
    }
}
