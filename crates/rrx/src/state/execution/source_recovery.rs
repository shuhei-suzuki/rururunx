//! Private source-only recovery claims. Stored metadata never recreates a grant.
use super::*;
use crate::execution::workflow_source::{ReconstructedFrame, digest, task_digest};
use serde::{Deserialize, Serialize};

const META_BYTES: usize = 128 * 1024;
const OWNER_BYTES: usize = 1024 * 1024;
const CONTEXT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Pin {
    version: u64,
    digest: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pins {
    scope: Scope,
    project: Pin,
    goal: Pin,
    task: Pin,
    workflow: Pin,
    workflow_id: RecordId,
    generation: u64,
    context: Pin,
    artifact: ResultArtifact,
    governing: String,
    instruction: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    id: Uuid,
    epoch: u64,
    version: u64,
    state: String,
    pins: Pins,
    frame: Option<String>,
    #[serde(default)]
    marker: Option<MarkerSourceAnchor>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MarkerSourceAnchor {
    operation: crate::execution::OperationId,
    marker_digest: String,
    prior_source_version: u64,
    prior_source_body_sha256: String,
}
struct Snapshot {
    project: Project,
    goal: Goal,
    task: Task,
    pins: Pins,
}
/// Created only by begin's atomic current-owner claim; deliberately non-Clone.
pub(crate) struct SourceRecovery {
    pub(crate) project: Project,
    pub(crate) goal: Goal,
    pub(crate) task: Task,
    pub(crate) artifact: ResultArtifact,
    binding: SourceReadBinding,
}
#[derive(Clone)]
pub(crate) struct SourceReadBinding {
    task: TaskId,
    id: Uuid,
    epoch: u64,
    preparing_version: Option<u64>,
    frame: Option<String>,
    artifact: ArtifactId,
}
impl SourceReadBinding {
    pub(crate) fn owns_artifact(&self, id: ArtifactId) -> bool {
        self.artifact == id
    }
}
impl SourceRecovery {
    pub(crate) fn binding(&self) -> SourceReadBinding {
        self.binding.clone()
    }
}
/// Only a validated pre-write snapshot may advance resulting typed bookkeeping.
pub(in crate::state) struct SourceAdvance(Option<Row>);

pub(in crate::state) fn install_schema(tx: &Transaction<'_>) -> Result<()> {
    tx.execute_batch(include_str!("source_recovery.sql"))?;
    Ok(())
}
pub(in crate::state) fn validate_legacy_namespace(tx: &Transaction<'_>) -> Result<()> {
    let count: u64 = tx.query_row("SELECT COUNT(*) FROM sqlite_schema WHERE name IN ('source_recoveries','source_recovery_identity','source_recovery_no_replace','source_recovery_no_delete')", [], |r| r.get(0))?;
    ensure!(count == 0, "legacy source recovery namespace is not empty");
    Ok(())
}
fn bounded<T: DeserializeOwned>(c: &Connection, table: &str, id: &str, max: usize) -> Result<T> {
    let bytes: usize = c.query_row(
        &format!("SELECT length(CAST(body AS BLOB)) FROM {table} WHERE id=?1"),
        [id],
        |r| r.get(0),
    )?;
    ensure!(bytes <= max, "source recovery {table} body exceeds bound");
    decode(c.query_row(
        &format!("SELECT body FROM {table} WHERE id=?1"),
        [id],
        |r| r.get(0),
    )?)
}
fn pin<T: Serialize>(version: u64, body: &T) -> Result<Pin> {
    Ok(Pin {
        version,
        digest: digest(&serde_json::to_vec(body)?),
    })
}
fn epoch(c: &Connection, expected: u64) -> Result<()> {
    let current: u64 = c.query_row(
        "SELECT epoch FROM runtime_epoch WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    ensure!(
        expected > 0 && current == expected,
        "source recovery epoch retired"
    );
    Ok(())
}
fn snapshot(c: &Connection, task_id: TaskId) -> Result<Snapshot> {
    let task: Task = bounded(c, "tasks", &task_id.to_string(), OWNER_BYTES)?;
    ensure!(
        task.id == task_id && !task_terminal(task.state),
        "source recovery Task inactive/foreign"
    );
    let project: Project = bounded(c, "projects", &task.project_id.to_string(), OWNER_BYTES)?;
    let goal: Goal = bounded(c, "goals", &task.goal_id.to_string(), 4 * 1024 * 1024)?;
    ensure!(
        project.id == task.project_id
            && goal.id == task.goal_id
            && goal.project_id == project.id
            && project.state == ProjectState::Registered
            && goal.state == GoalState::Running,
        "source recovery owners inactive/foreign"
    );
    check_indexed(
        c,
        "projects",
        &[
            ("id", json!(project.id)),
            ("version", json!(project.version)),
            ("root", json!(project.root)),
        ],
    )?;
    check_indexed(
        c,
        "goals",
        &[
            ("id", json!(goal.id)),
            ("version", json!(goal.version)),
            ("project_id", json!(project.id)),
        ],
    )?;
    check_indexed(
        c,
        "tasks",
        &[
            ("id", json!(task.id)),
            ("version", json!(task.version)),
            ("project_id", json!(project.id)),
            ("goal_id", json!(goal.id)),
            ("issue", json!(task.issue)),
        ],
    )?;
    let (p, g, t) = scope_keys(&task.scope())?;
    let mut stmt=c.prepare("SELECT id FROM records WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind='workflow' LIMIT 2")?;
    let ids = stmt
        .query_map(params![p, g, t], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    ensure!(ids.len() == 1, "source recovery requires sole Workflow");
    let record: Record = bounded(c, "records", &ids[0], CONTEXT_BYTES)?;
    ensure!(
        record.id.to_string() == ids[0]
            && record.scope == task.scope()
            && record.kind == RecordKind::Workflow,
        "source recovery Workflow scope/body mismatch"
    );
    let mut columns = scoped_columns(&record.scope);
    columns.extend([
        ("id", json!(record.id)),
        ("version", json!(record.version)),
        ("kind", json!("workflow")),
    ]);
    check_indexed(c, "records", &columns)?;
    let workflow: crate::workflow::WorkflowSnapshot = serde_json::from_value(record.data.clone())?;
    let owner = context_owner(&task.scope())?;
    let (latest,bytes):(u64,usize)=c.query_row("SELECT version,length(CAST(body AS BLOB)) FROM context_versions WHERE project_id=?1 AND owner=?2 ORDER BY version DESC LIMIT 1",params![p,owner],|r|Ok((r.get(0)?,r.get(1)?)))?;
    ensure!(
        latest == task.context_version && bytes <= CONTEXT_BYTES,
        "source recovery current Context exceeds bound or changed"
    );
    let context: ContextVersion = decode(c.query_row(
        "SELECT body FROM context_versions WHERE project_id=?1 AND owner=?2 AND version=?3",
        params![p, owner, latest],
        |r| r.get(0),
    )?)?;
    let mut columns = scoped_columns(&context.scope);
    columns.extend([("owner", json!(owner)), ("version", json!(context.version))]);
    check_indexed(c, "context_versions", &columns)?;
    // Validate an existing historical Workflow, not an invented initial row.
    crate::workflow::validate_transition(&task, &record, Some(&record))?;
    crate::workflow::validate_context(&task, &record, &context)?;
    let id = workflow
        .sources
        .artifact
        .context("Published source artifact required; pre-artifact recovery unsupported")?;
    let _: ResultArtifact = bounded(c, "result_artifacts", &id.to_string(), META_BYTES)?;
    let artifact = self_artifact_tx(c, id)?;
    ensure!(
        artifact.scope == task.scope()
            && artifact.state == ArtifactState::Published
            && artifact.revision == context.revision
            && artifact.revision == workflow.sources.revision
            && artifact.dependencies == workflow.sources.source_versions,
        "source recovery requires exact Published frame"
    );
    ensure!(
        artifact.dependencies.len() <= 128
            && artifact
                .dependencies
                .iter()
                .all(|(k, v)| k.len() <= 128 && v.len() <= 256),
        "source map exceeds bound"
    );
    let mut stmt =
        c.prepare("SELECT name,digest FROM artifact_dependencies WHERE artifact_id=?1 LIMIT 129")?;
    let deps = stmt
        .query_map([id.to_string()], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<BTreeMap<_, _>>>()?;
    ensure!(
        deps == artifact.dependencies,
        "source recovery indexed dependencies changed"
    );
    let unit = unit_tx(c, artifact.unit_id)?;
    ensure!(
        unit.scope == artifact.scope && unit.kind == UnitKind::Executor,
        "source recovery producing Executor mismatch"
    );
    let pins = Pins {
        scope: task.scope(),
        project: pin(project.version, &project)?,
        goal: pin(goal.version, &goal)?,
        task: pin(task.version, &task)?,
        workflow: pin(record.version, &record)?,
        workflow_id: record.id,
        generation: workflow.generation,
        context: pin(context.version, &context)?,
        artifact,
        governing: governing_digest(&project, &goal)?,
        instruction: task_digest(&task)?,
    };
    Ok(Snapshot {
        project,
        goal,
        task,
        pins,
    })
}
fn row(c: &Connection, task: TaskId) -> Result<Option<Row>> {
    let n: Option<usize> = c
        .query_row(
            "SELECT length(CAST(body AS BLOB)) FROM source_recoveries WHERE task_id=?1",
            [task.to_string()],
            |r| r.get(0),
        )
        .optional()?;
    let Some(n) = n else { return Ok(None) };
    ensure!(n <= META_BYTES, "source recovery metadata exceeds bound");
    let row: Row = decode(c.query_row(
        "SELECT body FROM source_recoveries WHERE task_id=?1",
        [task.to_string()],
        |r| r.get(0),
    )?)?;
    ensure!(
        row.pins.scope.task_id == Some(task),
        "source recovery Task index mismatch"
    );
    let mut columns = scoped_columns(&row.pins.scope);
    columns.extend([
        ("id", json!(row.id)),
        ("owner_epoch", json!(row.epoch)),
        ("version", json!(row.version)),
        ("state", json!(row.state)),
    ]);
    check_indexed(c, "source_recoveries", &columns)?;
    Ok(Some(row))
}
fn validate_row(c: &Connection, row: &Row) -> Result<()> {
    epoch(c, row.epoch)?;
    ensure!(
        row.marker.is_none(),
        "marker-bound Source7 requires the genuine managed successor reader, not generic source recapture"
    );
    ensure!(
        serde_json::to_vec(&snapshot(c, row.pins.scope.task_id.context("Task missing")?)?.pins)?
            == serde_json::to_vec(&row.pins)?,
        "source recovery full authority snapshot changed"
    );
    Ok(())
}
/// Sealed projection from Root's genuine allocation-derived marker plan. This
/// contains no source/native grant; its SQL write must be part of the SAME
/// original marker transaction/exact permit batch, never ordinary after_write.
pub(in crate::state) struct SourceMarkerAdvance {
    old: Row,
    old_body: String,
    next: Row,
    next_body: String,
    next_digest: String,
}
pub(in crate::state) fn plan_source_marker_advance(
    original: &(Uuid, u64, String),
    marker: &crate::state::managed_binding::ManagedMarkerPlan,
) -> Result<SourceMarkerAdvance> {
    ensure!(
        original.2.len() <= META_BYTES,
        "original Source7 exceeds bound"
    );
    let old: Row = decode(original.2.clone())?;
    let (project, _) = marker.project();
    let (goal, _) = marker.goal();
    let (task, _) = marker.task_before();
    let (workflow, _) = marker.workflow_before();
    let (context, _) = marker.context();
    ensure!(
        old.id == original.0
            && old.version == original.1
            && old.state == "installed"
            && old.marker.is_none()
            && old.frame.is_some()
            && old.epoch == marker.unit().0.owner_epoch
            && old.pins.scope == marker.scope()
            && old.pins.project == pin(project.version, project)?
            && old.pins.goal == pin(goal.version, goal)?
            && old.pins.task == pin(task.version, task)?
            && old.pins.workflow == pin(workflow.version, workflow)?
            && old.pins.workflow_id == workflow.id
            && old.pins.context == pin(context.version, context)?
            && old.pins.instruction == task_digest(task)?
            && old.pins.governing == governing_digest(project, goal)?,
        "Source7 original marker frame differs"
    );
    let (task_after, _) = marker.task_after();
    let (workflow_after, _) = marker.workflow_after();
    let mut next = old.clone();
    bump(&mut next.version)?;
    next.pins.task = pin(task_after.version, task_after)?;
    next.pins.workflow = pin(workflow_after.version, workflow_after)?;
    next.marker = Some(MarkerSourceAnchor {
        operation: marker.operation(),
        marker_digest: marker.marker_digest().into(),
        prior_source_version: old.version,
        prior_source_body_sha256: digest(original.2.as_bytes()),
    });
    let next_body = serde_json::to_string(&next)?;
    ensure!(
        next_body.len() <= META_BYTES,
        "marker Source7 metadata exceeds bound"
    );
    let next_digest = digest(next_body.as_bytes());
    Ok(SourceMarkerAdvance {
        old,
        old_body: original.2.clone(),
        next,
        next_body,
        next_digest,
    })
}
impl SourceMarkerAdvance {
    pub(in crate::state) fn mutation(
        &self,
    ) -> Result<crate::state::managed_binding::ExactRowMutation> {
        crate::state::managed_binding::ExactRowMutation::new(
            "source_recoveries",
            "UPDATE",
            Some(image(&self.old, &self.old_body)?),
            Some(image(&self.next, &self.next_body)?),
        )
    }
    pub(in crate::state) fn resulting_pin(&self) -> (Uuid, u64, &str) {
        (self.next.id, self.next.version, &self.next_digest)
    }
    pub(in crate::state) fn validate_original_tx(&self, c: &Connection) -> Result<()> {
        let (p, g, t) = scope_keys(&self.old.pins.scope)?;
        let exact: bool = c.query_row("SELECT EXISTS(SELECT 1 FROM source_recoveries WHERE task_id=?1 AND project_id=?2 AND goal_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='installed' AND body=?7)", params![t,p,g,self.old.id.to_string(),self.old.epoch,self.old.version,self.old_body], |r| r.get(0))?;
        ensure!(exact, "original Source7 marker row changed");
        Ok(())
    }
    /// Root's marker consumer validates the Driver/original frame BEFORE any
    /// writes. This asserts only the preplanned Source row mutation; it neither
    /// opens a helper nor manufactures post-marker successor authority.
    pub(in crate::state) fn write_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        self.validate_original_tx(tx)?;
        ensure!(tx.execute("UPDATE source_recoveries SET version=?1,body=?2 WHERE task_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='installed' AND body=?7", params![self.next.version,self.next_body,self.old.pins.scope.task_id.context("source Task missing")?.to_string(),self.old.id.to_string(),self.old.epoch,self.old.version,self.old_body])? == 1, "Source7 marker CAS changed");
        Ok(())
    }
    pub(in crate::state) fn validate_result_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        let (p, g, t) = scope_keys(&self.next.pins.scope)?;
        let exact:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM source_recoveries WHERE task_id=?1 AND project_id=?2 AND goal_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='installed' AND body=?7)",params![t,p,g,self.next.id.to_string(),self.next.epoch,self.next.version,self.next_body],|r|r.get(0))?;
        ensure!(exact, "planned marker Source7 result changed");
        Ok(())
    }
}
/// Exact nongrant Source7 content for the genuine retained Driver's coherent
/// read ticket. Public metadata cannot reconstruct a source-read capability.
pub(in crate::state) fn driver_anchor(
    c: &Connection,
    task: TaskId,
) -> Result<Option<(Uuid, u64, String)>> {
    let Some(row) = row(c, task)? else {
        return Ok(None);
    };
    ensure!(
        row.state == "installed",
        "Driver source recovery is not installed"
    );
    validate_row(c, &row)?;
    let body: String = c.query_row(
        "SELECT body FROM source_recoveries WHERE task_id=?1 AND id=?2 AND owner_epoch=?3 AND version=?4 AND state='installed' AND length(CAST(body AS BLOB))<=131072",
        params![task.to_string(), row.id.to_string(), row.epoch, row.version],
        |r| r.get(0),
    )?;
    Ok(Some((row.id, row.version, body)))
}
fn image(row: &Row, body: &str) -> Result<Vec<rusqlite::types::Value>> {
    let (p, g, t) = scope_keys(&row.pins.scope)?;
    Ok(vec![
        t.into(),
        p.into(),
        g.into(),
        row.id.to_string().into(),
        i64::try_from(row.epoch)?.into(),
        i64::try_from(row.version)?.into(),
        row.state.clone().into(),
        body.to_owned().into(),
    ])
}
fn original_image(c: &Connection, task: TaskId) -> Result<(Row, Vec<rusqlite::types::Value>)> {
    let original = row(c, task)?.context("source recovery mutation original missing")?;
    let body: String = c.query_row("SELECT body FROM source_recoveries WHERE task_id=?1 AND length(CAST(body AS BLOB))<=131072", [task.to_string()], |r| r.get(0))?;
    let columns = image(&original, &body)?;
    Ok((original, columns))
}
fn write(
    c: &Connection,
    permits: &crate::state::managed_binding::PrivatePermitManager,
    row: &mut Row,
) -> Result<()> {
    let old = row.version;
    let task = row
        .pins
        .scope
        .task_id
        .context("source recovery Task missing")?;
    let (original, old_image) = original_image(c, task)?;
    ensure!(
        original.version == old && original.pins.scope == row.pins.scope,
        "source recovery mutation original changed"
    );
    bump(&mut row.version)?;
    let body = serde_json::to_string(row)?;
    ensure!(
        body.len() <= META_BYTES,
        "source recovery metadata exceeds bound"
    );
    let mutation = crate::state::managed_binding::ExactRowMutation::new(
        "source_recoveries",
        "UPDATE",
        Some(old_image),
        Some(image(row, &body)?),
    )?;
    permits.with_exact_permit(vec![mutation], || {
        ensure!(c.execute("UPDATE source_recoveries SET id=?1,owner_epoch=?2,version=?3,state=?4,body=?5 WHERE task_id=?6 AND version=?7",params![row.id.to_string(),row.epoch,row.version,row.state,body,task.to_string(),old])?==1,"source recovery CAS changed");
        permits.ensure_consumed()
    })?;
    Ok(())
}
pub(crate) fn validate_binding(c: &Connection, binding: &SourceReadBinding) -> Result<()> {
    let row = row(c, binding.task)?.context("source recovery claim missing")?;
    ensure!(
        row.id == binding.id
            && row.epoch == binding.epoch
            && row.pins.artifact.id == binding.artifact
            && match binding.preparing_version {
                Some(v) => row.version == v && row.state == "preparing",
                None => row.state == "installed" && row.frame == binding.frame,
            },
        "source recovery claim replaced/closed"
    );
    validate_row(c, &row)
}
pub(in crate::state) fn validate_task(c: &Connection, task: TaskId) -> Result<()> {
    if let Some(row) = row(c, task)? {
        ensure!(
            row.state == "installed",
            "recovered source not installed/current"
        );
        validate_row(c, &row)?;
    }
    Ok(())
}
pub(in crate::state) fn before_write(
    c: &Connection,
    task: TaskId,
    conservative: bool,
) -> Result<SourceAdvance> {
    let row = row(c, task)?;
    if !conservative && let Some(row) = &row {
        ensure!(row.state == "installed", "source recovery is not installed");
        validate_row(c, row)?;
    }
    Ok(SourceAdvance(row))
}
pub(in crate::state) fn after_write(
    c: &Connection,
    permits: &crate::state::managed_binding::PrivatePermitManager,
    old: SourceAdvance,
    conservative: bool,
) -> Result<()> {
    let Some(mut row) = old.0 else { return Ok(()) };
    let next = if conservative {
        None
    } else {
        snapshot(c, row.pins.scope.task_id.unwrap()).ok()
    };
    if let Some(next) = next
        && next.pins.project == row.pins.project
        && next.pins.goal == row.pins.goal
        && next.pins.generation == row.pins.generation
        && next.pins.workflow_id == row.pins.workflow_id
        && serde_json::to_vec(&next.pins.artifact)? == serde_json::to_vec(&row.pins.artifact)?
        && next.pins.governing == row.pins.governing
        && next.pins.instruction == row.pins.instruction
    {
        row.pins = next.pins;
    } else {
        row.state = "invalid".into();
    }
    write(c, permits, &mut row)?;
    Ok(())
}
pub(in crate::state) fn invalidate_epoch(
    c: &Connection,
    permits: &crate::state::managed_binding::PrivatePermitManager,
) -> Result<()> {
    // Bodies remain auditable; old rows never constitute a new current capability.
    let mut stmt = c.prepare("SELECT task_id FROM source_recoveries WHERE state<>'invalid'")?;
    let ids = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(stmt);
    for id in ids {
        let mut r = row(c, id.parse()?)?.unwrap();
        r.state = "invalid".into();
        write(c, permits, &mut r)?;
    }
    Ok(())
}
impl Store {
    /// Coherent current source-owned Task for Context rendering. A projected
    /// Workflow Task is not yet a durable authority body. No recovery row leaves
    /// existing opaque/legacy source semantics unchanged.
    pub(crate) fn recovered_source_task(&mut self, task: TaskId) -> Result<Option<Task>> {
        let tx = self.connection.transaction()?;
        let current = if let Some(row) = row(&tx, task)? {
            ensure!(
                row.state == "installed",
                "source recovery is not installed/current"
            );
            validate_row(&tx, &row)?;
            Some(snapshot(&tx, task)?.task)
        } else {
            None
        };
        tx.commit()?;
        Ok(current)
    }
    pub(crate) fn begin_retained_source_recovery(
        &mut self,
        task: TaskId,
        owner_epoch: u64,
    ) -> Result<SourceRecovery> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        epoch(&tx, owner_epoch)?;
        let snapshot = snapshot(&tx, task)?;
        let old = row(&tx, task)?;
        ensure!(
            old.as_ref()
                .is_none_or(|r| r.state != "preparing" && r.marker.is_none()),
            "source recovery already Preparing or marker-bound; genuine closure/epoch recovery required"
        );
        let mut row = Row {
            id: Uuid::new_v4(),
            epoch: owner_epoch,
            version: old.as_ref().map_or(1, |r| r.version),
            state: "preparing".into(),
            pins: snapshot.pins,
            frame: None,
            marker: None,
        };
        if old.is_some() {
            write(&tx, &self.binding_permits, &mut row)?;
        } else {
            let (p, g, t) = scope_keys(&row.pins.scope)?;
            let body = serde_json::to_string(&row)?;
            ensure!(
                body.len() <= META_BYTES,
                "source recovery metadata exceeds bound"
            );
            let mutation = crate::state::managed_binding::ExactRowMutation::new(
                "source_recoveries",
                "INSERT",
                None,
                Some(image(&row, &body)?),
            )?;
            self.binding_permits.with_exact_permit(vec![mutation], || {
                tx.execute("INSERT INTO source_recoveries(task_id,project_id,goal_id,id,owner_epoch,version,state,body) VALUES(?1,?2,?3,?4,?5,1,'preparing',?6)",params![t,p,g,row.id.to_string(),row.epoch,body])?;
                self.binding_permits.ensure_consumed()
            })?;
        }
        append_event(
            &tx,
            &row.pins.scope,
            "execution.source_recovery_claimed",
            json!({"recovery":row.id,"version":row.version,"epoch":row.epoch,"artifact":row.pins.artifact.id}),
        )?;
        tx.commit()?;
        let binding = SourceReadBinding {
            task,
            id: row.id,
            epoch: row.epoch,
            preparing_version: Some(row.version),
            frame: None,
            artifact: row.pins.artifact.id,
        };
        Ok(SourceRecovery {
            project: snapshot.project,
            goal: snapshot.goal,
            task: snapshot.task,
            artifact: row.pins.artifact,
            binding,
        })
    }
    pub(crate) fn validate_source_read(&self, binding: &SourceReadBinding) -> Result<()> {
        validate_binding(&self.connection, binding)
    }
    pub(crate) fn accept_retained_source_recovery(
        &mut self,
        claim: &SourceRecovery,
        frame: &ReconstructedFrame,
    ) -> Result<SourceReadBinding> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        validate_binding(&tx, &claim.binding)?;
        let mut row = row(&tx, claim.task.id)?.unwrap();
        ensure!(
            frame.scope() == &row.pins.scope
                && frame.revision() == row.pins.artifact.revision
                && frame.artifact() == row.pins.artifact.id
                && frame.versions() == &row.pins.artifact.dependencies
                && frame.governing() == row.pins.governing,
            "reconstructed frame differs from claimed source"
        );
        row.state = "installed".into();
        row.frame = Some(frame.digest().into());
        write(&tx, &self.binding_permits, &mut row)?;
        append_event(
            &tx,
            &row.pins.scope,
            "execution.source_recovery_installed",
            json!({"recovery":row.id,"version":row.version,"epoch":row.epoch,"artifact":row.pins.artifact.id,"frame":row.frame}),
        )?;
        tx.commit()?;
        Ok(SourceReadBinding {
            task: claim.task.id,
            id: row.id,
            epoch: row.epoch,
            preparing_version: None,
            frame: row.frame,
            artifact: row.pins.artifact.id,
        })
    }
    pub(crate) fn abandon_retained_source_recovery(
        &mut self,
        binding: &SourceReadBinding,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(mut row) = row(&tx, binding.task)?
            && row.id == binding.id
            && row.epoch == binding.epoch
            && row.state == "preparing"
            && Some(row.version) == binding.preparing_version
        {
            row.state = "invalid".into();
            write(&tx, &self.binding_permits, &mut row)?;
            append_event(
                &tx,
                &row.pins.scope,
                "execution.source_recovery_abandoned",
                json!({"recovery":row.id,"epoch":row.epoch}),
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn old6(path: &Path) -> Connection {
        let old = Connection::open(path).unwrap();
        old.create_scalar_function(
            "rrx_writer_contract_version",
            0,
            rusqlite::functions::FunctionFlags::SQLITE_UTF8
                | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC
                | rusqlite::functions::FunctionFlags::SQLITE_INNOCUOUS,
            |_| Ok(6_i64),
        )
        .unwrap();
        old.execute_batch(include_str!("../schema.sql")).unwrap();
        old.execute_batch(include_str!("../execution.sql")).unwrap();
        old.execute_batch(include_str!("native_results.sql"))
            .unwrap();
        old.execute(
            "INSERT INTO runtime_epoch(singleton,instance_id,epoch) VALUES(1,?1,0)",
            [Uuid::new_v4().to_string()],
        )
        .unwrap();
        for table in MUTABLE_TABLES.iter().filter(|t| {
            !crate::state::runtime::TABLES.contains(t)
                && !crate::state::managed_binding::TABLES.contains(t)
                && !matches!(
                    **t,
                    "source_recoveries"
                        | "verification_profiles"
                        | "workflow_verification_contracts"
                        | "verification_runs"
                        | "verification_commands"
                )
        }) {
            for action in ["INSERT", "UPDATE", "DELETE"] {
                old.execute_batch(&format!("CREATE TRIGGER writer_{table}_{action} BEFORE {action} ON {table} WHEN rrx_writer_contract_version()<>6 BEGIN SELECT RAISE(ABORT,'incompatible rrx writer contract'); END;")).unwrap();
            }
        }
        old.pragma_update(None, "application_id", APPLICATION_ID)
            .unwrap();
        old.pragma_update(None, "user_version", 6).unwrap();
        old
    }
    #[test]
    fn schema7_orders_migration_and_fences_already_open6_writer() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let old = old6(&path);
        let mut cached = old
            .prepare("UPDATE runtime_epoch SET epoch=epoch+1 WHERE singleton=1")
            .unwrap();
        cached.execute([]).unwrap();
        let current = Store::open(&path).unwrap();
        assert_eq!(current.schema_version().unwrap(), SCHEMA_VERSION);
        assert!(
            cached
                .execute([])
                .unwrap_err()
                .to_string()
                .contains("incompatible rrx writer contract")
        );
        for table in MUTABLE_TABLES {
            for action in ["INSERT", "UPDATE", "DELETE"] {
                let sql: String = current
                    .connection
                    .query_row(
                        "SELECT sql FROM sqlite_schema WHERE type='trigger' AND name=?1",
                        [format!("writer_{table}_{action}")],
                        |r| r.get(0),
                    )
                    .unwrap();
                assert!(sql.contains(&format!("<>{SCHEMA_VERSION}")));
            }
        }
        assert_eq!(
            current
                .connection
                .query_row("SELECT COUNT(*) FROM source_recoveries", [], |r| r
                    .get::<_, u64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            current
                .connection
                .query_row("SELECT epoch FROM runtime_epoch", [], |r| r
                    .get::<_, u64>(0))
                .unwrap(),
            1
        );
        drop(cached);
        drop(old);
        drop(current);
        assert_eq!(
            Store::open(&path).unwrap().schema_version().unwrap(),
            SCHEMA_VERSION
        );
    }
    #[test]
    fn schema7_namespace_collision_rolls_back_exact6_bytes_and_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let old = old6(&path);
        old.execute("CREATE TABLE source_recoveries(unrelated TEXT)", [])
            .unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(
            Store::open(&path)
                .err()
                .unwrap()
                .to_string()
                .contains("legacy source recovery namespace")
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(
            old.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            6
        );
        assert_eq!(
            old.query_row("SELECT epoch FROM runtime_epoch", [], |r| r
                .get::<_, u64>(0))
                .unwrap(),
            0
        );
    }
}
