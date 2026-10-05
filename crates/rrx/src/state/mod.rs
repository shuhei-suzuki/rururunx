//! Transactional SQLite snapshots + append-only logical events, scoped by Project.
mod environment;
#[cfg(test)]
mod native_dispatch_tests;
pub(crate) use environment::EnvironmentAdmission;
use std::{path::Path, time::Duration};

use anyhow::{Context, Result, bail, ensure};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::domain::*;

pub const SCHEMA_VERSION: i64 = 4;
mod execution;
pub(crate) use execution::QuotaAdmission;
pub(crate) use execution::cleanup::CleanupClaim;
pub const APPLICATION_ID: i64 = 0x52525831; // ASCII RRX1.

/// Typed transactional guards let callers distinguish contention from storage failure.
#[derive(Debug)]
pub enum StateGuardError {
    WorktreeLocked,
    ProjectInactive,
    ExecutorReserved,
    EnvironmentAuthority,
    SnapshotChanged {
        table: String,
        id: String,
        expected: u64,
    },
}
impl std::fmt::Display for StateGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProjectInactive => f.write_str("project is not registered/active"),
            Self::WorktreeLocked => {
                f.write_str("worktree has an active immutable/maintenance lock")
            }
            Self::ExecutorReserved => f.write_str("executor is reserved/live"),
            Self::EnvironmentAuthority => f.write_str("native environment authority unavailable"),
            Self::SnapshotChanged {
                table,
                id,
                expected,
            } => write!(
                f,
                "stale snapshot {table}/{id}, expected version {expected}"
            ),
        }
    }
}
impl std::error::Error for StateGuardError {}

/// Launch-time access check; native integrations must still use their owning
/// Git/Session transactional guards at the actual side effect boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkflowAccess {
    StateOnly,
    TerminalDecision,
    TerminalRecovery,
    ReadOnly,
    Mutating,
}

pub struct Store {
    connection: Connection,
}

fn register_writer_contract(connection: &Connection) -> Result<()> {
    connection.create_scalar_function(
        "rrx_writer_contract_version",
        0,
        rusqlite::functions::FunctionFlags::SQLITE_UTF8
            | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC
            | rusqlite::functions::FunctionFlags::SQLITE_INNOCUOUS,
        |_| Ok(SCHEMA_VERSION),
    )?;
    Ok(())
}

/// Corruption fixtures model a current writer, rather than an incompatible
/// legacy connection. Production callers must use Store's transactional API.
#[cfg(test)]
pub(crate) fn current_test_writer(path: &Path) -> Result<Connection> {
    let connection = Connection::open(path)?;
    register_writer_contract(&connection)?;
    Ok(connection)
}

enum WorkflowCompletion<'a> {
    Executor(&'a crate::execution::WorkflowPublication),
    Readonly(&'a crate::execution::ReadonlyCompletion),
}
impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let connection = Connection::open(path)
            .with_context(|| format!("cannot open local state {}", path.display()))?;
        Self::initialize(connection)
    }

    pub fn memory() -> Result<Self> {
        Self::initialize(Connection::open_in_memory()?)
    }

    fn initialize(mut connection: Connection) -> Result<Self> {
        register_writer_contract(&connection)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
        ensure!(
            (0..=SCHEMA_VERSION).contains(&version),
            "unsupported state schema {version}, supported {SCHEMA_VERSION}"
        );
        let application: i64 =
            connection.pragma_query_value(None, "application_id", |row| row.get(0))?;
        ensure!(
            application == APPLICATION_ID || (version == 0 && application == 0),
            "not an rrx state database (application_id={application})"
        );
        connection.pragma_update(None, "foreign_keys", true)?;
        if version < SCHEMA_VERSION {
            let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            // Recheck under the write lock: another runtime may have initialized it.
            let locked_version: i64 =
                tx.pragma_query_value(None, "user_version", |row| row.get(0))?;
            if locked_version == 0 {
                let application: i64 =
                    tx.pragma_query_value(None, "application_id", |row| row.get(0))?;
                let objects: i64 = tx.query_row(
                    "SELECT COUNT(*) FROM sqlite_schema WHERE substr(name,1,7) <> 'sqlite_'",
                    [],
                    |row| row.get(0),
                )?;
                ensure!(
                    application == 0 && objects == 0,
                    "refusing to initialize a nonempty or foreign database"
                );
                tx.execute_batch(include_str!("schema.sql"))?;
                execution::install_schema(&tx)?;
                tx.pragma_update(None, "application_id", APPLICATION_ID)?;
                tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            } else {
                ensure!(
                    (1..=SCHEMA_VERSION).contains(&locked_version),
                    "unsupported state schema {locked_version}"
                );
                let application: i64 =
                    tx.pragma_query_value(None, "application_id", |row| row.get(0))?;
                ensure!(application == APPLICATION_ID, "not an rrx state database");
                // Ordered JSON-format migrations; SQL layout and ownership/audit stay intact.
                // v2 adds Project blocked_reason; v3 adds authoritative Workflow records.
                for next in (locked_version + 1)..=SCHEMA_VERSION {
                    if next == 4 {
                        execution::install_schema(&tx)?;
                    }
                    tx.pragma_update(None, "user_version", next)?;
                }
            }
            tx.commit()?;
        }
        connection.pragma_update(None, "journal_mode", "WAL")?;
        Ok(Self { connection })
    }

    pub fn schema_version(&self) -> Result<i64> {
        Ok(self
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))?)
    }

    pub fn project(&self, id: ProjectId) -> Result<Option<Project>> {
        self.read("projects", &id.to_string())
    }
    pub fn goal(&self, id: GoalId) -> Result<Option<Goal>> {
        self.read("goals", &id.to_string())
    }
    pub fn task(&self, id: TaskId) -> Result<Option<Task>> {
        self.read("tasks", &id.to_string())
    }
    pub fn record(&self, id: RecordId) -> Result<Option<Record>> {
        self.read("records", &id.to_string())
    }

    pub fn projects(&self) -> Result<Vec<Project>> {
        self.list("projects", None, None, None)
    }
    pub fn goals(&self, project: ProjectId) -> Result<Vec<Goal>> {
        self.list("goals", Some(project), None, None)
    }
    pub fn tasks(&self, project: ProjectId, goal: Option<GoalId>) -> Result<Vec<Task>> {
        self.list("tasks", Some(project), goal, None)
    }
    pub fn records(&self, scope: &Scope, kind: RecordKind) -> Result<Vec<Record>> {
        let mut statement = self.connection.prepare(
            "SELECT body FROM records WHERE project_id=?1 AND (?2 IS NULL OR goal_id=?2) AND (?3 IS NULL OR task_id=?3) AND kind=?4 ORDER BY rowid")?;
        let rows = statement.query_map(
            params![
                scope.project_id.to_string(),
                str_id(scope.goal_id),
                str_id(scope.task_id),
                kind.key()
            ],
            |row| row.get::<_, String>(0),
        )?;
        rows.map(|row| decode(row?)).collect()
    }

    pub fn put_project(&mut self, project: &mut Project) -> Result<()> {
        ensure!(
            !project.name.trim().is_empty() && !project.base_branch.trim().is_empty(),
            "project name/base branch must be nonempty"
        );
        ensure!(
            project.root.is_absolute()
                && project.max_tasks > 0
                && project.root.components().all(|c| !matches!(
                    c,
                    std::path::Component::ParentDir | std::path::Component::CurDir
                )),
            "project root must be absolute and task limit positive"
        );
        let root = project.root.to_str().context("project root is not UTF-8")?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if project.state == ProjectState::Removed {
            ensure_project_idle(&tx, project.id)?;
        }
        if let Some(previous) = read_tx::<Project>(&tx, "projects", &project.id.to_string())? {
            ensure!(
                previous.root == project.root
                    && previous.repository_identity == project.repository_identity,
                "project identity/root cannot silently change"
            );
        }
        if let Some(previous) = read_tx::<Project>(&tx, "projects", &project.id.to_string())?
            && previous.worktree_root != project.worktree_root
        {
            let bindings: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM tasks WHERE project_id=?1 AND json_extract(body,'$.worktree') IS NOT NULL)", [project.id.to_string()], |row| row.get(0))?;
            ensure!(
                !bindings,
                "cannot change Project namespace after task worktree binding"
            );
        }
        {
            let mut statement = tx.prepare("SELECT body FROM projects WHERE id<>?1")?;
            for body in
                statement.query_map([project.id.to_string()], |row| row.get::<_, String>(0))?
            {
                let other: Project = decode(body?)?;
                ensure!(
                    other.repository_identity != project.repository_identity,
                    "repository identity already registered"
                );
                ensure!(
                    !project.root.starts_with(&other.root)
                        && !other.root.starts_with(&project.root)
                        && !project.worktree_root.starts_with(&other.worktree_root)
                        && !other.worktree_root.starts_with(&project.worktree_root)
                        && !project.root.starts_with(&other.worktree_root)
                        && !other.worktree_root.starts_with(&project.root)
                        && !other.root.starts_with(&project.worktree_root)
                        && !project.worktree_root.starts_with(&other.root),
                    "Project roots/worktree namespaces overlap"
                );
            }
        }
        let mut next = project.clone();
        bump(&mut next.version)?;
        next.updated_at = now_ms();
        let body = serde_json::to_string(&next)?;
        write_snapshot(
            &tx,
            "projects",
            &next.id.to_string(),
            project.version,
            "INSERT INTO projects(id,root,version,body) VALUES(?1,?2,?3,?4)",
            params![next.id.to_string(), root, next.version, body],
            &body,
            next.version,
        )?;
        append_event(
            &tx,
            &Scope::project(next.id),
            "project.saved",
            json!({"version":next.version,"state":next.state}),
        )?;
        tx.commit()?;
        *project = next;
        Ok(())
    }

    pub fn put_goal(&mut self, goal: &mut Goal) -> Result<()> {
        ensure!(
            !goal.objective.trim().is_empty() && !goal.completion_criteria.is_empty(),
            "goal needs objective and explicit completion criteria"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if !goal_terminal(goal.state) {
            let previous = read_tx::<Goal>(&tx, "goals", &goal.id.to_string())?;
            let safe_update = if let Some(old) = &previous {
                let mut metadata = goal.clone();
                metadata.state = old.state;
                metadata.blockers = old.blockers.clone();
                !goal_terminal(old.state)
                    && (old.state == goal.state
                        || matches!(
                            goal.state,
                            GoalState::Blocked | GoalState::WaitingHuman | GoalState::Paused
                        ))
                    && serde_json::to_value(metadata)? == serde_json::to_value(old)?
            } else {
                false
            };
            ensure_activity_write(&tx, goal.project_id, safe_update)?;
        }
        if let Some(previous) = read_tx::<Goal>(&tx, "goals", &goal.id.to_string())? {
            ensure!(
                previous.project_id == goal.project_id,
                "goal project binding is immutable"
            );
        }
        validate_goal_references(&tx, goal)?;
        let mut next = goal.clone();
        bump(&mut next.version)?;
        next.updated_at = now_ms();
        let body = serde_json::to_string(&next)?;
        write_snapshot(
            &tx,
            "goals",
            &next.id.to_string(),
            goal.version,
            "INSERT INTO goals(id,project_id,version,body) VALUES(?1,?2,?3,?4)",
            params![
                next.id.to_string(),
                next.project_id.to_string(),
                next.version,
                body
            ],
            &body,
            next.version,
        )?;
        append_event(
            &tx,
            &next.scope(),
            "goal.saved",
            json!({"version":next.version,"state":next.state}),
        )?;
        tx.commit()?;
        *goal = next;
        Ok(())
    }

    pub fn put_task(&mut self, task: &mut Task) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if owns_workflow(&tx, &task.scope())? {
            let old: Task = read_tx(&tx, "tasks", &task.id.to_string())?.context("unknown Task")?;
            ensure!(
                old.workflow == task.workflow
                    && old.risk == task.risk
                    && old.context_version == task.context_version
                    && old.revision == task.revision
                    && old.phase == task.phase
                    && old.artifacts == task.artifacts
                    && (old.state == task.state
                        || (!task_terminal(old.state) && task.state == TaskState::WaitingHuman)),
                "workflow-owned Task fields require atomic transition"
            );
        }
        let next = put_task_tx(&tx, task)?;
        tx.commit()?;
        *task = next;
        Ok(())
    }
    pub fn put_record(&mut self, record: &mut Record) -> Result<()> {
        ensure!(
            record.kind != RecordKind::Workflow,
            "Workflow authority requires atomic Task/context transition"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let next = put_record_tx(&tx, record)?;
        tx.commit()?;
        *record = next;
        Ok(())
    }
    /// Atomic Task workflow transition. Every ownership snapshot is checked under
    /// the same immediate transaction; failed context/history writes roll back Task.
    pub(crate) fn put_workflow_transition(
        &mut self,
        task: &mut Task,
        workflow: &mut Record,
        context: Option<&ContextVersion>,
        project_version: u64,
        goal_version: u64,
        access: WorkflowAccess,
    ) -> Result<()> {
        self.put_workflow_transition_inner(
            task,
            workflow,
            context,
            project_version,
            goal_version,
            access,
            None,
        )
    }
    pub(crate) fn put_workflow_result_transition(
        &mut self,
        task: &mut Task,
        workflow: &mut Record,
        context: &ContextVersion,
        project_version: u64,
        goal_version: u64,
        publication: &crate::execution::WorkflowPublication,
    ) -> Result<()> {
        self.put_workflow_transition_inner(
            task,
            workflow,
            Some(context),
            project_version,
            goal_version,
            WorkflowAccess::StateOnly,
            Some(WorkflowCompletion::Executor(publication)),
        )
    }
    pub(crate) fn put_workflow_readonly_transition(
        &mut self,
        task: &mut Task,
        workflow: &mut Record,
        context: &ContextVersion,
        project_version: u64,
        goal_version: u64,
        completion: &crate::execution::ReadonlyCompletion,
    ) -> Result<()> {
        self.put_workflow_transition_inner(
            task,
            workflow,
            Some(context),
            project_version,
            goal_version,
            WorkflowAccess::StateOnly,
            Some(WorkflowCompletion::Readonly(completion)),
        )
    }
    // Exact owner CAS and optional completion proof are independent inputs.
    #[allow(clippy::too_many_arguments)]
    fn put_workflow_transition_inner(
        &mut self,
        task: &mut Task,
        workflow: &mut Record,
        context: Option<&ContextVersion>,
        project_version: u64,
        goal_version: u64,
        access: WorkflowAccess,
        publication: Option<WorkflowCompletion<'_>>,
    ) -> Result<()> {
        ensure!(
            workflow.kind == RecordKind::Workflow && workflow.scope == task.scope(),
            "workflow requires exact owning Task scope"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let conservative = matches!(
            access,
            WorkflowAccess::TerminalDecision | WorkflowAccess::TerminalRecovery
        );
        if !conservative {
            ensure_project_registered(&tx, task.project_id)?;
        }
        let project: Project =
            read_tx(&tx, "projects", &task.project_id.to_string())?.context("unknown project")?;
        let goal: Goal =
            read_tx(&tx, "goals", &task.goal_id.to_string())?.context("unknown goal")?;
        for (table, id, actual, expected) in [
            (
                "projects",
                project.id.to_string(),
                project.version,
                project_version,
            ),
            ("goals", goal.id.to_string(), goal.version, goal_version),
        ] {
            if actual != expected {
                bail!(StateGuardError::SnapshotChanged {
                    table: table.into(),
                    id,
                    expected
                });
            }
        }
        ensure!(project.state != ProjectState::Removed, "Project removed");
        ensure!(
            goal.project_id == task.project_id
                && (conservative
                    || matches!(
                        goal.state,
                        GoalState::Created | GoalState::Analyzing | GoalState::Running
                    )),
            "goal is inactive for workflow progression"
        );
        if matches!(access, WorkflowAccess::ReadOnly | WorkflowAccess::Mutating) {
            let mut statement = tx.prepare(
                "SELECT body FROM records WHERE project_id=?1 AND goal_id=?2 AND task_id=?3",
            )?;
            for body in statement.query_map(
                params![
                    task.project_id.to_string(),
                    task.goal_id.to_string(),
                    task.id.to_string()
                ],
                |r| r.get::<_, String>(0),
            )? {
                let record: Record = decode(body?)?;
                if record.kind == RecordKind::Session {
                    let session: Session = serde_json::from_value(record.data)?;
                    if execution::logically_retired_session(&tx, &session)? {
                        continue;
                    }
                    if crate::git::executor_reserved(&session)
                        || session.state == SessionState::Lost
                        || (access == WorkflowAccess::Mutating && !session_terminal(session.state))
                    {
                        bail!(StateGuardError::ExecutorReserved);
                    }
                } else if access == WorkflowAccess::Mutating
                    && record.kind == RecordKind::WorktreeLock
                    && serde_json::from_value::<crate::git::WorktreeLock>(record.data)?.active
                {
                    bail!(StateGuardError::WorktreeLocked);
                }
            }
        }
        let previous_task: Task = read_tx(&tx, "tasks", &task.id.to_string())?
            .context("workflow requires existing Task")?;
        ensure!(
            !task_terminal(previous_task.state) || access == WorkflowAccess::TerminalRecovery,
            "terminal Task cannot resume workflow progression"
        );
        let count: i64 = tx.query_row("SELECT COUNT(*) FROM records WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind='workflow' AND id<>?4",
            params![task.project_id.to_string(),task.goal_id.to_string(),task.id.to_string(),workflow.id.to_string()], |r| r.get(0))?;
        ensure!(count == 0, "Task already owns a workflow");
        let previous_workflow: Option<Record> = read_tx(&tx, "records", &workflow.id.to_string())?;
        if conservative {
            ensure!(
                context.is_none(),
                "terminal decision/recovery cannot publish context"
            );
            let previous = previous_workflow
                .as_ref()
                .context("terminal operation requires existing workflow")?;
            let before: crate::workflow::WorkflowSnapshot =
                serde_json::from_value(previous.data.clone())?;
            let after: crate::workflow::WorkflowSnapshot =
                serde_json::from_value(workflow.data.clone())?;
            let mut expected = before.clone();
            if access == WorkflowAccess::TerminalDecision {
                ensure!(
                    !task_terminal(previous_task.state)
                        && before.terminal_decision.is_none()
                        && matches!(task.state, TaskState::Cancelled | TaskState::Failed),
                    "invalid terminal decision"
                );
                expected.terminal_decision = after.terminal_decision.clone();
                ensure!(
                    expected.terminal_decision.is_some(),
                    "terminal decision required"
                );
            } else {
                ensure!(
                    task.state == previous_task.state && before.terminal_decision.is_some(),
                    "terminal recovery cannot change Task decision"
                );
                let index = before.active.context("terminal reservation missing")?;
                let attempt = &before.history[index];
                ensure!(
                    attempt.state != crate::workflow::AttemptState::Evaluating
                        || crate::workflow::known_gate_observation(attempt).is_some(),
                    "unknown external outcome requires explicit recovery"
                );
                ensure!(
                    attempt.session_id.is_some() || !attempt.dispatch_started,
                    "unbound dispatch requires explicit recovery"
                );
                expected.active = None;
                expected.history[index].state = crate::workflow::AttemptState::Interrupted;
                expected.history[index].completed_at = after.history[index].completed_at;
                expected.history[index].detail = after.history[index].detail.clone();
                ensure!(
                    expected.history[index].completed_at.is_some()
                        && expected.history[index]
                            .detail
                            .as_ref()
                            .is_some_and(|s| !s.trim().is_empty()),
                    "recovery requires timestamp/reason"
                );
            }
            ensure!(
                serde_json::to_value(expected)? == serde_json::to_value(&after)?,
                "terminal operation may only record decision or close its reservation"
            );
            let mut expected_task = previous_task.clone();
            expected_task.state = task.state;
            ensure!(
                serde_json::to_value(expected_task)? == serde_json::to_value(&*task)?,
                "terminal operation may only change Task decision"
            );
        }
        crate::workflow::validate_transition(task, workflow, previous_workflow.as_ref())?;
        let typed_workflow: crate::workflow::WorkflowSnapshot =
            serde_json::from_value(workflow.data.clone())?;
        for attempt in &typed_workflow.history {
            if let Some(identity) = &attempt.unit {
                let unit = execution::unit_tx(&tx, identity.unit)?;
                ensure!(
                    identity == &crate::execution::ManagedUnitRef::from(&unit)
                        && unit.scope == task.scope()
                        && unit.phase == attempt.phase.key(),
                    "Workflow preparation unit identity mismatch"
                );
            }
            if let Some(identity) = &attempt.execution {
                let unit = execution::unit_tx(&tx, identity.unit)?;
                ensure!(
                    unit.scope == task.scope()
                        && identity.scope == unit.scope
                        && identity.generation == unit.generation
                        && identity.epoch == unit.owner_epoch
                        && attempt.session_id == Some(identity.session)
                        && unit.session_id == Some(identity.session),
                    "Workflow managed identity mismatch"
                );
                if let Some(preparation) = &attempt.unit {
                    ensure!(
                        preparation == &crate::execution::ManagedUnitRef::from(&unit),
                        "Workflow Session differs from preparation unit"
                    );
                }
            }
        }
        if let Some(previous) = &previous_workflow {
            let before: crate::workflow::WorkflowSnapshot =
                serde_json::from_value(previous.data.clone())?;
            let after: crate::workflow::WorkflowSnapshot =
                serde_json::from_value(workflow.data.clone())?;
            if before.active.is_some() && after.active != before.active {
                let attempt = &before.history[before.active.unwrap()];
                if attempt.unit.is_some()
                    && after.history[before.active.unwrap()].state
                        == crate::workflow::AttemptState::Interrupted
                {
                    ensure!(
                        execution::managed_attempt_retired(&tx, &task.scope(), attempt)?,
                        "managed reservation retirement requires closed exact authority"
                    );
                }
                ensure!(
                    !(access != WorkflowAccess::TerminalRecovery
                        && attempt.phase.actor() != crate::workflow::Actor::EvidencePort
                        && attempt.dispatch_started
                        && attempt.session_id.is_none()
                        && !(after.history[before.active.unwrap()].state
                            == crate::workflow::AttemptState::Interrupted
                            && execution::managed_attempt_retired(&tx, &task.scope(), attempt)?)),
                    crate::workflow::UNBOUND_NATIVE_RECOVERY_REQUIRED
                );
                let mut statement = tx.prepare("SELECT body FROM records WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind='session'")?;
                let own_id = before
                    .active
                    .and_then(|index| before.history[index].session_id);
                let mut own_found = own_id.is_none();
                for body in statement.query_map(
                    params![
                        task.project_id.to_string(),
                        task.goal_id.to_string(),
                        task.id.to_string()
                    ],
                    |row| row.get::<_, String>(0),
                )? {
                    let record: Record = decode(body?)?;
                    let session: Session = serde_json::from_value(record.data)?;
                    let own = before
                        .active
                        .and_then(|index| before.history[index].session_id)
                        == Some(session.id);
                    let retired = execution::logically_retired_session(&tx, &session)?;
                    if retired && !own {
                        continue;
                    }
                    if retired
                        && own
                        && after.history[before.active.unwrap()].state
                            == crate::workflow::AttemptState::Interrupted
                    {
                        own_found = true;
                        continue;
                    }
                    ensure!(
                        !crate::git::executor_reserved(&session)
                            && session.state != SessionState::Lost
                            && (!own
                                || matches!(
                                    session.state,
                                    SessionState::Exited
                                        | SessionState::Stopped
                                        | SessionState::Failed
                                )),
                        "closing workflow reservation requires verified native termination"
                    );
                    own_found |= own;
                    if own
                        && before.active.is_some_and(|index| {
                            after.history[index].state == crate::workflow::AttemptState::Succeeded
                        })
                    {
                        ensure!(
                            session.state == SessionState::Exited,
                            "native completion requires persisted Exited Session"
                        );
                    }
                }
                ensure!(
                    own_found,
                    "closing workflow reservation requires persisted owned Session"
                );
            }
        }
        if let Some(context) = context {
            ensure!(
                context.scope == task.scope() && context.version == task.context_version,
                "context pointer/scope differs from workflow Task"
            );
            crate::workflow::validate_context(task, workflow, context)?;
            put_context_tx(&tx, context)?;
        } else {
            let owner = context_owner(&task.scope())?;
            let latest: u64 = tx.query_row("SELECT COALESCE(MAX(version),0) FROM context_versions WHERE project_id=?1 AND owner=?2",
                params![task.project_id.to_string(),owner], |r| r.get(0))?;
            ensure!(
                latest == task.context_version,
                "workflow context pointer is stale"
            );
            let body: String = tx.query_row(
                "SELECT body FROM context_versions WHERE project_id=?1 AND owner=?2 AND version=?3",
                params![task.project_id.to_string(), owner, latest],
                |row| row.get(0),
            )?;
            crate::workflow::validate_context(task, workflow, &decode(body)?)?;
        }
        if let Some(completion) = publication {
            let previous = previous_workflow
                .as_ref()
                .context("completion requires a reserved Workflow")?;
            let context = context.context("completion requires a new ContextVersion")?;
            match completion {
                WorkflowCompletion::Executor(publication) => execution::publish_workflow_result_tx(
                    &tx,
                    publication,
                    task,
                    workflow,
                    previous,
                    context,
                )?,
                WorkflowCompletion::Readonly(completion) => {
                    execution::complete_workflow_readonly_tx(
                        &tx, completion, task, workflow, previous, context,
                    )?
                }
            }
        }
        let next_task = put_task_tx(&tx, task)?;
        let next_workflow = put_record_tx(&tx, workflow)?;
        tx.commit()?;
        *task = next_task;
        *workflow = next_workflow;
        Ok(())
    }

    /// Conservative factual journal independent of owner activity. This never
    /// changes Task, Session identity/state, context pointers or gate approval.
    pub(crate) fn observe_workflow_gate(
        &mut self,
        record: &mut Record,
        index: usize,
        expected: &crate::workflow::PhaseAttempt,
        observation: crate::workflow::GateObservation,
    ) -> Result<()> {
        ensure!(
            observation.outcome.is_some() != observation.error.is_some()
                && observation.sources.scope == record.scope,
            "observation needs exactly one actual outcome/error"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut latest: Record = read_tx(&tx, "records", &record.id.to_string())?
            .context("workflow missing for observation")?;
        ensure!(
            latest.kind == RecordKind::Workflow && latest.scope == record.scope,
            "foreign workflow observation"
        );
        let mut workflow: crate::workflow::WorkflowSnapshot =
            serde_json::from_value(latest.data.clone())?;
        ensure!(
            workflow.active == Some(index),
            "observation attempt no longer active"
        );
        let attempt = workflow
            .history
            .get_mut(index)
            .context("observation attempt missing")?;
        ensure!(
            attempt.state == crate::workflow::AttemptState::Evaluating
                && attempt.phase == expected.phase
                && attempt.generation == expected.generation
                && attempt.context_version == expected.context_version
                && attempt.session_id == expected.session_id
                && attempt.agent == expected.agent
                && attempt.started_at == expected.started_at
                && attempt.claimed_observations == expected.claimed_observations
                && attempt.observations.len() == attempt.claimed_observations,
            "observation identity differs"
        );
        ensure!(
            observation.sources.payload.is_empty(),
            "observation cannot duplicate Context Pack payload"
        );
        attempt.detail = Some(if let Some(error) = &observation.error {
            error.clone()
        } else {
            "gate outcome observed".into()
        });
        attempt.observations.push(observation.clone());
        latest.data = serde_json::to_value(workflow)?;
        let next = put_record_tx(&tx, &latest)?;
        append_event(
            &tx,
            &next.scope,
            "workflow.gate_observed",
            json!({"workflow":next.id,"attempt":index,"observation":observation}),
        )?;
        tx.commit()?;
        *record = next;
        Ok(())
    }
    pub fn put_session(&mut self, session: &Session, expected_version: u64) -> Result<u64> {
        let mut record = Record::new(
            session.scope.clone(),
            RecordKind::Session,
            serde_json::to_value(session)?,
        );
        record.id = RecordId(session.id.0);
        record.version = expected_version;
        if let Some(previous) = self.record(record.id)? {
            record.created_at = previous.created_at;
        }
        self.put_record(&mut record)?;
        Ok(record.version)
    }

    /// Provider dispatch write-ahead boundary. Scope and lock authority are checked
    /// in the same Immediate transaction as the Session CAS, including other writers.
    pub(crate) fn put_session_if_current(
        &mut self,
        session: &Session,
        expected_session: u64,
        expected: [u64; 3],
        expected_locks: &[(RecordId, u64)],
    ) -> Result<u64> {
        self.put_session_current(session, expected_session, expected, expected_locks, None)
    }

    pub(crate) fn put_session_with_environment_if_current(
        &mut self,
        session: &Session,
        expected_session: u64,
        expected: [u64; 3],
        expected_locks: &[(RecordId, u64)],
        admission: &EnvironmentAdmission,
    ) -> Result<u64> {
        self.put_session_current(
            session,
            expected_session,
            expected,
            expected_locks,
            Some(admission),
        )
    }

    fn put_session_current(
        &mut self,
        session: &Session,
        expected_session: u64,
        expected: [u64; 3],
        expected_locks: &[(RecordId, u64)],
        admission: Option<&EnvironmentAdmission>,
    ) -> Result<u64> {
        let scope = &session.scope;
        validate_scope(scope)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        for ((table, id), version) in [
            ("projects", Some(scope.project_id.to_string())),
            ("goals", scope.goal_id.map(|id| id.to_string())),
            ("tasks", scope.task_id.map(|id| id.to_string())),
        ]
        .into_iter()
        .zip(expected)
        {
            let Some(id) = id else {
                ensure!(version == 0, "absent parent version must be zero");
                continue;
            };
            let actual: Option<u64> = tx
                .query_row(
                    &format!("SELECT version FROM {table} WHERE id=?1"),
                    [&id],
                    |row| row.get(0),
                )
                .optional()?;
            if actual != Some(version) {
                bail!(StateGuardError::SnapshotChanged {
                    table: table.into(),
                    id,
                    expected: version
                });
            }
        }
        if let Some(id) = scope.goal_id {
            let goal: Goal = read_tx(&tx, "goals", &id.to_string())?.context("native Goal lost")?;
            ensure!(
                goal.project_id == scope.project_id,
                "native Goal ownership mismatch"
            );
            ensure!(
                !matches!(
                    goal.state,
                    GoalState::Paused
                        | GoalState::Completed
                        | GoalState::Cancelled
                        | GoalState::Failed
                ),
                "native Goal inactive"
            );
        }
        if let Some(id) = scope.task_id {
            let task: Task = read_tx(&tx, "tasks", &id.to_string())?.context("native Task lost")?;
            ensure!(task.scope() == *scope, "native Task ownership mismatch");
            ensure!(
                !task_terminal(task.state)
                    && (session.role != SessionRole::Executor
                        || !matches!(task.state, TaskState::ReadyForPr | TaskState::PrCreated)),
                "native Task inactive"
            );
        }
        let mut statement=tx.prepare("SELECT body FROM records WHERE project_id=?1 AND goal_id IS ?2 AND task_id IS ?3 AND kind='worktree_lock'")?;
        let mut actual = vec![];
        for body in statement.query_map(
            params![
                scope.project_id.to_string(),
                scope.goal_id.map(|id| id.to_string()),
                scope.task_id.map(|id| id.to_string())
            ],
            |row| row.get::<_, String>(0),
        )? {
            let record: Record = decode(body?)?;
            actual.push((record.id, record.version));
        }
        drop(statement);
        actual.sort();
        let mut expected_locks = expected_locks.to_vec();
        expected_locks.sort();
        if actual != expected_locks {
            bail!(StateGuardError::SnapshotChanged {
                table: "worktree_lock".into(),
                id: session.id.to_string(),
                expected: 0
            });
        }
        let mut record = Record::new(
            scope.clone(),
            RecordKind::Session,
            serde_json::to_value(session)?,
        );
        record.id = RecordId(session.id.0);
        record.version = expected_session;
        if let Some(previous) = read_tx::<Record>(&tx, "records", &record.id.to_string())? {
            record.created_at = previous.created_at;
        }
        guard_record_tx(&tx, &record)?;
        if let Some(admission) = admission {
            // Preserve the existing guard/CAS order before any foreign decision.
            let actual: Option<u64> = tx
                .query_row(
                    "SELECT version FROM records WHERE id=?1",
                    [record.id.to_string()],
                    |row| row.get(0),
                )
                .optional()?;
            if record.version == 0 {
                ensure!(actual.is_none(), "snapshot insert failed");
            } else if actual != Some(record.version) {
                bail!(StateGuardError::SnapshotChanged {
                    table: "records".into(),
                    id: record.id.to_string(),
                    expected: record.version
                });
            }
            environment::evaluate(&tx, scope.project_id, admission)?;
        }
        let next = write_record_tx(&tx, &record)?;
        tx.commit()?;
        Ok(next.version)
    }

    pub fn session(&self, id: SessionId) -> Result<Option<(Session, u64)>> {
        match self.record(RecordId(id.0))? {
            Some(record) => {
                ensure!(
                    record.kind == RecordKind::Session,
                    "identity is not a session"
                );
                Ok(Some((serde_json::from_value(record.data)?, record.version)))
            }
            None => Ok(None),
        }
    }

    pub fn put_context(&mut self, context: &ContextVersion) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        ensure!(
            !owns_workflow(&tx, &context.scope)?,
            "workflow-owned ContextVersion requires atomic transition"
        );
        put_context_tx(&tx, context)?;
        tx.commit()?;
        Ok(())
    }
    pub fn context(&self, scope: &Scope, version: Option<u64>) -> Result<Option<ContextVersion>> {
        let owner = context_owner(scope)?;
        let body: Option<String> = self.connection.query_row(
            "SELECT body FROM context_versions WHERE project_id=?1 AND owner=?2 AND (?3 IS NULL OR version=?3) ORDER BY version DESC LIMIT 1",
            params![scope.project_id.to_string(), owner, version], |row| row.get(0)).optional()?;
        let context: Option<ContextVersion> = body.map(decode).transpose()?;
        if let Some(context) = &context {
            ensure!(context.scope == *scope, "context scope mismatch");
        }
        Ok(context)
    }

    pub fn put_usage(&mut self, usage: &Usage) -> Result<()> {
        validate_scope(&usage.scope)?;
        // Legacy observations remain unqualified. Enforce the same integer
        // storage range required by the composed telemetry design without
        // casting, clamping or turning an invalid observation into zero.
        ensure!(
            [
                usage.input_tokens,
                usage.cached_input_tokens,
                usage.output_tokens,
                usage.context_pack_version,
                usage.context_pack_size,
                usage.repo_map_size,
            ]
            .into_iter()
            .flatten()
            .all(|value| i64::try_from(value).is_ok()),
            "usage integer exceeds the supported storage range"
        );
        ensure!(
            usage
                .estimated_cost
                .is_none_or(|value| value.is_finite() && value >= 0.0),
            "usage cost must be finite/nonnegative"
        );
        if usage.input_tokens.is_none()
            && usage.output_tokens.is_none()
            && usage.cached_input_tokens.is_none()
            && usage.estimated_cost.is_none()
        {
            ensure!(
                usage
                    .missing_reason
                    .as_ref()
                    .is_some_and(|s| !s.trim().is_empty()),
                "unavailable usage needs an explicit reason"
            );
        }
        let (session, _) = self
            .session(usage.session_id)?
            .context("usage session not found")?;
        ensure!(
            session.scope == usage.scope && session.agent == usage.agent,
            "usage/session project/goal/task/agent mismatch"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO usage(project_id,goal_id,task_id,session_id,body) VALUES(?1,?2,?3,?4,?5)",
            params![
                usage.scope.project_id.to_string(),
                str_id(usage.scope.goal_id),
                str_id(usage.scope.task_id),
                usage.session_id.to_string(),
                serde_json::to_string(usage)?
            ],
        )?;
        append_event(
            &tx,
            &usage.scope,
            "usage.recorded",
            json!({"session":usage.session_id,"phase":usage.phase,"review_round":usage.review_round}),
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn usage(&self, scope: &Scope) -> Result<Vec<Usage>> {
        validate_scope(scope)?;
        let mut statement = self.connection.prepare("SELECT project_id,goal_id,task_id,session_id,body FROM usage WHERE project_id=?1 AND (?2 IS NULL OR goal_id=?2) AND (?3 IS NULL OR task_id=?3) ORDER BY sequence")?;
        let rows = statement.query_map(
            params![
                scope.project_id.to_string(),
                str_id(scope.goal_id),
                str_id(scope.task_id)
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )?;
        rows.map(|row| {
            let (project, goal, task, session, body) = row?;
            // Serde errors can quote arbitrary persisted body values. Keep
            // this legacy read refusal static, including the anyhow chain.
            let usage: Usage =
                decode(body).map_err(|_| anyhow::anyhow!("invalid persisted usage snapshot"))?;
            ensure!(
                usage.scope.project_id.to_string() == project
                    && str_id(usage.scope.goal_id) == goal
                    && str_id(usage.scope.task_id) == task
                    && usage.session_id.to_string() == session,
                "usage row/body identity mismatch"
            );
            Ok(usage)
        })
        .collect()
    }

    pub fn audit(&mut self, scope: &Scope, kind: &str, data: Value) -> Result<()> {
        validate_scope(scope)?;
        ensure!(!kind.trim().is_empty(), "audit kind must be nonempty");
        ensure!(
            !reserved_audit_kind(kind),
            "audit kind is reserved for Store mutations"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        append_event(&tx, scope, kind, data)?;
        tx.commit()?;
        Ok(())
    }

    /// Publish a Task-scoped observation only while its authority versions are
    /// current. One Immediate transaction excludes independent SQLite writers.
    pub fn audit_if_current(
        &mut self,
        scope: &Scope,
        expected: [u64; 3],
        kind: &str,
        data: Value,
    ) -> Result<()> {
        validate_scope(scope)?;
        ensure!(
            !kind.trim().is_empty() && !reserved_audit_kind(kind),
            "invalid/reserved audit kind"
        );
        let goal_id = scope.goal_id.context("current audit requires Goal scope")?;
        let task_id = scope.task_id.context("current audit requires Task scope")?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        for ((table, id), version) in [
            ("projects", scope.project_id.to_string()),
            ("goals", goal_id.to_string()),
            ("tasks", task_id.to_string()),
        ]
        .into_iter()
        .zip(expected)
        {
            let actual: Option<u64> = tx
                .query_row(
                    &format!("SELECT version FROM {table} WHERE id=?1"),
                    [&id],
                    |row| row.get(0),
                )
                .optional()?;
            if actual != Some(version) {
                bail!(StateGuardError::SnapshotChanged {
                    table: table.into(),
                    id,
                    expected: version
                });
            }
        }
        let project: Project = decode(tx.query_row(
            "SELECT body FROM projects WHERE id=?1",
            [scope.project_id.to_string()],
            |row| row.get(0),
        )?)?;
        if project.state != ProjectState::Registered {
            bail!(StateGuardError::ProjectInactive);
        }
        let goal: Goal = decode(tx.query_row(
            "SELECT body FROM goals WHERE id=?1",
            [goal_id.to_string()],
            |row| row.get(0),
        )?)?;
        let task: Task = decode(tx.query_row(
            "SELECT body FROM tasks WHERE id=?1",
            [task_id.to_string()],
            |row| row.get(0),
        )?)?;
        ensure!(
            goal.project_id == project.id && task.scope() == *scope,
            "foreign audit Project/Goal/Task scope"
        );
        append_event(&tx, scope, kind, data)?;
        tx.commit()?;
        Ok(())
    }

    pub fn events(&self, scope: &Scope, after: i64, limit: usize) -> Result<Vec<AuditEvent>> {
        ensure!(
            (1..=10000).contains(&limit),
            "event query limit must be 1..10000"
        );
        let mut statement = self.connection.prepare("SELECT sequence,goal_id,task_id,kind,at,data FROM audit WHERE project_id=?1 AND (?2 IS NULL OR goal_id=?2) AND (?3 IS NULL OR task_id=?3) AND sequence>?4 ORDER BY sequence LIMIT ?5")?;
        let rows = statement.query_map(
            params![
                scope.project_id.to_string(),
                str_id(scope.goal_id),
                str_id(scope.task_id),
                after,
                limit as i64
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )?;
        rows.map(|row| {
            let (sequence, goal, task, kind, at, data) = row?;
            Ok(AuditEvent {
                sequence,
                scope: Scope {
                    project_id: scope.project_id,
                    goal_id: goal.map(|s| s.parse()).transpose()?,
                    task_id: task.map(|s| s.parse()).transpose()?,
                },
                kind,
                at,
                data: serde_json::from_str(&data)?,
            })
        })
        .collect()
    }

    fn read<T: DeserializeOwned>(&self, table: &str, id: &str) -> Result<Option<T>> {
        let sql = format!("SELECT body FROM {table} WHERE id=?1");
        let body: Option<String> = self
            .connection
            .query_row(&sql, [id], |row| row.get(0))
            .optional()?;
        body.map(decode).transpose()
    }

    fn list<T: DeserializeOwned>(
        &self,
        table: &str,
        project: Option<ProjectId>,
        goal: Option<GoalId>,
        _task: Option<TaskId>,
    ) -> Result<Vec<T>> {
        let (sql, args) = if table == "projects" {
            (
                "SELECT body FROM projects ORDER BY rowid".to_string(),
                vec![],
            )
        } else if table == "goals" {
            (
                "SELECT body FROM goals WHERE project_id=?1 ORDER BY rowid".to_string(),
                vec![str_id(project)],
            )
        } else {
            ("SELECT body FROM tasks WHERE project_id=?1 AND (?2 IS NULL OR goal_id=?2) ORDER BY rowid".to_string(), vec![str_id(project), str_id(goal)])
        };
        let mut statement = self.connection.prepare(&sql)?;
        let rows = statement.query_map(rusqlite::params_from_iter(args), |row| {
            row.get::<_, String>(0)
        })?;
        rows.map(|row| decode(row?)).collect()
    }
}

fn owns_workflow(tx: &Transaction<'_>, scope: &Scope) -> Result<bool> {
    if scope.task_id.is_none() {
        return Ok(false);
    }
    Ok(tx.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind='workflow')", params![scope.project_id.to_string(), scope.goal_id.map(|id| id.to_string()), scope.task_id.map(|id| id.to_string())], |row| row.get(0))?)
}
fn put_task_tx(tx: &Transaction<'_>, task: &Task) -> Result<Task> {
    ensure!(
        !task.title.trim().is_empty() && !task.executor.trim().is_empty(),
        "task title/executor must be nonempty"
    );
    if !task_terminal(task.state) {
        let previous = read_tx::<Task>(tx, "tasks", &task.id.to_string())?;
        let safe_update = if let Some(old) = &previous {
            let mut metadata = task.clone();
            metadata.state = old.state;
            metadata.blockers = old.blockers.clone();
            metadata.next_action = old.next_action.clone();
            !task_terminal(old.state)
                && (old.state == task.state || task.state == TaskState::WaitingHuman)
                && serde_json::to_value(metadata)? == serde_json::to_value(old)?
        } else {
            false
        };
        ensure_activity_write(tx, task.project_id, safe_update)?;
    }
    if let Some(previous) = read_tx::<Task>(tx, "tasks", &task.id.to_string())? {
        ensure!(
            previous.project_id == task.project_id && previous.goal_id == task.goal_id,
            "task ownership is immutable"
        );
        let admitted_rebind: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM task_execution a JOIN execution_units u ON u.id=a.active_unit WHERE a.task_id=?1 AND u.kind='executor' AND u.generation=a.generation AND u.worktree=?2 AND u.branch=?3 AND u.native_effects_open=1)",
            params![task.id.to_string(),task.worktree.as_ref().map(|p|p.to_string_lossy().into_owned()),task.branch], |r|r.get(0),
        )?;
        ensure!(
            admitted_rebind
                || (previous
                    .worktree
                    .as_ref()
                    .is_none_or(|path| task.worktree.as_ref() == Some(path))
                    && previous
                        .branch
                        .as_ref()
                        .is_none_or(|branch| task.branch.as_ref() == Some(branch))),
            "assigned task worktree/branch binding is immutable"
        );
        ensure!(
            task.workflow >= previous.workflow,
            "workflow downgrade is forbidden"
        );
    }
    ensure!(
        task.worktree.is_some() == task.branch.is_some(),
        "task path/branch must bind together"
    );
    if let (Some(path), Some(branch)) = (&task.worktree, &task.branch) {
        let project: Project =
            read_tx(tx, "projects", &task.project_id.to_string())?.context("unknown project")?;
        ensure!(
            path.is_absolute()
                && path.parent() == Some(project.worktree_root.as_path())
                && path.components().all(|c| !matches!(
                    c,
                    std::path::Component::ParentDir | std::path::Component::CurDir
                )),
            "task path must be normal direct child of Project namespace"
        );
        ensure!(!branch.trim().is_empty(), "task branch must be nonempty");
        let mut statement = tx.prepare("SELECT body FROM tasks WHERE project_id=?1 AND id<>?2")?;
        for body in statement.query_map(
            params![task.project_id.to_string(), task.id.to_string()],
            |row| row.get::<_, String>(0),
        )? {
            let other: Task = decode(body?)?;
            ensure!(
                other.worktree.as_ref() != Some(path) && other.branch.as_ref() != Some(branch),
                "task worktree/branch already owned"
            );
        }
    }
    let mut next = task.clone();
    bump(&mut next.version)?;
    next.updated_at = now_ms();
    let body = serde_json::to_string(&next)?;
    write_snapshot(
        tx,
        "tasks",
        &next.id.to_string(),
        task.version,
        "INSERT INTO tasks(id,project_id,goal_id,issue,version,body) VALUES(?1,?2,?3,?4,?5,?6)",
        params![
            next.id.to_string(),
            next.project_id.to_string(),
            next.goal_id.to_string(),
            next.issue,
            next.version,
            body
        ],
        &body,
        next.version,
    )?;
    // Issue is query metadata and may be linked after Task creation.
    if task_terminal(next.state) {
        execution::fence_task_tx(tx, &next.scope())?;
    }
    tx.execute(
        "UPDATE tasks SET issue=?1 WHERE id=?2",
        params![next.issue, next.id.to_string()],
    )?;
    append_event(
        tx,
        &next.scope(),
        "task.saved",
        json!({"version":next.version,"state":next.state,"phase":next.phase,"workflow":next.workflow}),
    )?;
    Ok(next)
}

fn guard_record_tx(tx: &Transaction<'_>, record: &Record) -> Result<()> {
    validate_scope(&record.scope)?;
    if record.kind == RecordKind::Session {
        let managed: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM session_units WHERE session_id=?1)",
            [record.id.to_string()],
            |r| r.get(0),
        )?;
        ensure!(
            !managed,
            "managed Session writes require execution-unit authority"
        );
        let session: Session =
            serde_json::from_value(record.data.clone()).context("invalid session payload")?;
        ensure!(
            session.scope == record.scope && session.id.0 == record.id.0,
            "session identity/scope mismatch"
        );
    }
    if let Some(previous) = read_tx::<Record>(tx, "records", &record.id.to_string())? {
        ensure!(
            previous.scope == record.scope && previous.kind == record.kind,
            "record scope/kind is immutable"
        );
    }
    if record.kind == RecordKind::Session
        && let Some(previous) = read_tx::<Record>(tx, "records", &record.id.to_string())?
    {
        let old: Session = serde_json::from_value(previous.data)?;
        let new: Session = serde_json::from_value(record.data.clone())?;
        ensure!(
            old.agent == new.agent
                && old.provider == new.provider
                && old.role == new.role
                && old.worktree == new.worktree,
            "session actor/worktree identity is immutable"
        );
    }
    validate_worktree_exclusion(tx, record)?;
    Ok(())
}
fn put_record_tx(tx: &Transaction<'_>, record: &Record) -> Result<Record> {
    guard_record_tx(tx, record)?;
    write_record_tx(tx, record)
}
/// Private caller must have checked the original Record guards in this transaction.
fn write_record_tx(tx: &Transaction<'_>, record: &Record) -> Result<Record> {
    let mut next = record.clone();
    bump(&mut next.version)?;
    next.updated_at = now_ms();
    let body = serde_json::to_string(&next)?;
    write_snapshot(
        tx,
        "records",
        &next.id.to_string(),
        record.version,
        "INSERT INTO records(id,kind,project_id,goal_id,task_id,version,body) VALUES(?1,?2,?3,?4,?5,?6,?7)",
        params![
            next.id.to_string(),
            next.kind.key(),
            next.scope.project_id.to_string(),
            str_id(next.scope.goal_id),
            str_id(next.scope.task_id),
            next.version,
            body
        ],
        &body,
        next.version,
    )?;
    append_event(
        tx,
        &next.scope,
        &format!("{}.saved", next.kind.key()),
        json!({"id":next.id,"version":next.version,"evidence": match next.kind {
            RecordKind::Review | RecordKind::Approval | RecordKind::WorktreeLock => next.data.clone(),
            RecordKind::Workflow => json!({"generation":next.data["generation"],"context_version":next.data["context_version"],"active":next.data["active"],"finished":next.data["finished"],"attempt":next.data["history"].as_array().and_then(|a|a.last()),"escalation":next.data["escalations"].as_array().and_then(|a|a.last()),"retry":next.data["retries"].as_array().and_then(|a|a.last()),"invalidation":next.data["invalidations"].as_array().and_then(|a|a.last())}),
            RecordKind::Session => json!({"state":next.data["state"],"agent":next.data["agent"],"provider":next.data["provider"],"role":next.data["role"],"native_ref":next.data["native_ref"],"dispatch_intent":next.data["recovery"]["dispatch_intent"]}),
            _ => Value::Null,
        }}),
    )?;
    Ok(next)
}

fn put_context_tx(tx: &Transaction<'_>, context: &ContextVersion) -> Result<()> {
    validate_scope(&context.scope)?;
    ensure!(
        context.scope.goal_id.is_some(),
        "context must belong to a goal/task"
    );
    ensure!(
        !context.revision.is_empty(),
        "context revision must be explicit"
    );
    let owner = context_owner(&context.scope)?;
    let latest: u64 = tx.query_row(
        "SELECT COALESCE(MAX(version),0) FROM context_versions WHERE project_id=?1 AND owner=?2",
        params![context.scope.project_id.to_string(), owner],
        |row| row.get(0),
    )?;
    ensure!(
        context.version == latest.checked_add(1).context("context version overflow")?,
        "context versions must be consecutive, expected {}",
        latest + 1
    );
    tx.execute("INSERT INTO context_versions(project_id,goal_id,task_id,owner,version,body) VALUES(?1,?2,?3,?4,?5,?6)",
            params![context.scope.project_id.to_string(), str_id(context.scope.goal_id), str_id(context.scope.task_id), owner, context.version, serde_json::to_string(context)?])?;
    append_event(
        tx,
        &context.scope,
        "context.created",
        json!({"version":context.version,"revision":context.revision}),
    )?;
    Ok(())
}

pub(crate) fn goal_terminal(state: GoalState) -> bool {
    matches!(
        state,
        GoalState::Completed | GoalState::Cancelled | GoalState::Failed
    )
}
pub(crate) fn task_terminal(state: TaskState) -> bool {
    matches!(
        state,
        TaskState::Completed | TaskState::Cancelled | TaskState::Failed | TaskState::Merged
    )
}
fn session_terminal(state: SessionState) -> bool {
    matches!(
        state,
        SessionState::Exited | SessionState::Stopped | SessionState::Failed
    )
}
fn ensure_project_registered(tx: &Transaction<'_>, id: ProjectId) -> Result<()> {
    let project: Project = read_tx(tx, "projects", &id.to_string())?.context("unknown project")?;
    if project.state != ProjectState::Registered {
        bail!(StateGuardError::ProjectInactive);
    }
    Ok(())
}
/// Blocked Projects may describe existing work conservatively, never start/resume it.
fn ensure_activity_write(tx: &Transaction<'_>, id: ProjectId, safe_update: bool) -> Result<()> {
    let project: Project = read_tx(tx, "projects", &id.to_string())?.context("unknown project")?;
    if project.state != ProjectState::Registered
        && !(project.state == ProjectState::Blocked && safe_update)
    {
        bail!(StateGuardError::ProjectInactive);
    }
    Ok(())
}
/// Checked in the same write transaction as removal, including Lost sessions.
fn ensure_project_idle(tx: &Transaction<'_>, id: ProjectId) -> Result<()> {
    for table in ["goals", "tasks", "records"] {
        let mut statement = tx.prepare(&format!("SELECT body FROM {table} WHERE project_id=?1"))?;
        for row in statement.query_map([id.to_string()], |row| row.get::<_, String>(0))? {
            let body = row?;
            let idle = match table {
                "goals" => goal_terminal(decode::<Goal>(body)?.state),
                "tasks" => task_terminal(decode::<Task>(body)?.state),
                _ => {
                    let record: Record = decode(body)?;
                    match record.kind {
                        RecordKind::Session => {
                            session_terminal(serde_json::from_value::<Session>(record.data)?.state)
                        }
                        RecordKind::WorktreeLock => {
                            !serde_json::from_value::<crate::git::WorktreeLock>(record.data)?.active
                        }
                        RecordKind::Workflow => serde_json::from_value::<
                            crate::workflow::WorkflowSnapshot,
                        >(record.data)?
                        .active
                        .is_none(),
                        _ => true,
                    }
                }
            };
            ensure!(
                idle,
                "cannot remove project with active {table}; complete/cancel or reconcile them first"
            );
        }
    }
    Ok(())
}

fn str_id<T: std::fmt::Display>(id: Option<T>) -> Option<String> {
    id.map(|id| id.to_string())
}
fn validate_goal_references(tx: &Transaction<'_>, goal: &Goal) -> Result<()> {
    use std::collections::BTreeSet;
    let mut references: BTreeSet<_> = goal.dag.hard_order()?.into_iter().collect();
    for proposal in &goal.followups {
        references.extend(proposal.dependencies.iter().copied());
    }
    for id in references {
        let owned: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM tasks WHERE id=?1 AND project_id=?2 AND goal_id=?3)",
            params![
                id.to_string(),
                goal.project_id.to_string(),
                goal.id.to_string()
            ],
            |row| row.get(0),
        )?;
        ensure!(owned, "goal references missing/foreign task {id}");
    }
    Ok(())
}
fn decode<T: DeserializeOwned>(body: String) -> Result<T> {
    serde_json::from_str(&body).context("invalid persisted snapshot")
}
/// Reservations and immutable locks serialize across independent SQLite connections.
fn validate_worktree_exclusion(tx: &Transaction<'_>, record: &Record) -> Result<()> {
    use crate::git::{WorktreeLock, executor_reserved};
    if record.kind == RecordKind::Session {
        let session: Session = serde_json::from_value(record.data.clone())?;
        if !session_terminal(session.state) {
            let safe_update =
                if let Some(previous) = read_tx::<Record>(tx, "records", &record.id.to_string())? {
                    let old: Session = serde_json::from_value(previous.data)?;
                    let mut metadata = session.clone();
                    metadata.state = old.state;
                    if session.pid.is_none() {
                        metadata.pid = old.pid;
                    }
                    !session_terminal(old.state)
                        && (session.state == SessionState::Lost
                            || (old.state != SessionState::Lost
                                && matches!(
                                    session.state,
                                    SessionState::WaitingHuman | SessionState::WaitingApproval
                                )))
                        && serde_json::to_value(metadata)? == serde_json::to_value(&old)?
                } else {
                    false
                };
            ensure_activity_write(tx, record.scope.project_id, safe_update)?;
        }
    }
    if record.kind == RecordKind::WorktreeLock
        && serde_json::from_value::<WorktreeLock>(record.data.clone())?.active
    {
        ensure_project_registered(tx, record.scope.project_id)?;
    }

    if record.kind == RecordKind::Session {
        let session: Session = serde_json::from_value(record.data.clone())?;
        if session.role == SessionRole::Executor {
            let id = session
                .scope
                .task_id
                .context("executor requires task scope")?;
            let task: Task =
                read_tx(tx, "tasks", &id.to_string())?.context("unknown executor task")?;
            ensure!(
                task.scope() == session.scope && task.worktree.as_ref() == Some(&session.worktree),
                "executor worktree/scope differs from task binding"
            );
        }
    }
    let acquiring = if record.kind == RecordKind::WorktreeLock {
        ensure!(
            record.scope.task_id.is_some(),
            "worktree lock needs exact task scope"
        );
        let lock: WorktreeLock = serde_json::from_value(record.data.clone())?;
        ensure!(
            !lock.reason.trim().is_empty() && !lock.revision.trim().is_empty(),
            "lock needs reason/revision"
        );
        let task = read_tx::<Task>(tx, "tasks", &record.scope.task_id.unwrap().to_string())?
            .context("unknown lock task")?;
        ensure!(
            task.scope() == record.scope
                && task.worktree.as_ref() == Some(&lock.worktree)
                && task.branch.as_ref() == Some(&lock.branch),
            "lock binding mismatch"
        );
        if let Some(previous) = read_tx::<Record>(tx, "records", &record.id.to_string())? {
            let old: WorktreeLock = serde_json::from_value(previous.data)?;
            ensure!(
                old.revision == lock.revision
                    && old.worktree == lock.worktree
                    && old.branch == lock.branch
                    && old.reason == lock.reason,
                "lock identity is immutable"
            );
        }
        lock.active
    } else {
        false
    };
    let executor = if record.kind == RecordKind::Session {
        executor_reserved(&serde_json::from_value::<Session>(record.data.clone())?)
    } else {
        false
    };
    if !acquiring && !executor {
        return Ok(());
    }
    let mut statement = tx.prepare("SELECT body FROM records WHERE project_id=?1 AND goal_id IS ?2 AND task_id IS ?3 AND (kind='worktree_lock' OR kind='session')")?;
    let rows = statement.query_map(
        params![
            record.scope.project_id.to_string(),
            str_id(record.scope.goal_id),
            str_id(record.scope.task_id)
        ],
        |row| row.get::<_, String>(0),
    )?;
    for row in rows {
        let other: Record = decode(row?)?;
        if other.id == record.id {
            continue;
        }
        match other.kind {
            RecordKind::WorktreeLock => {
                let lock: WorktreeLock = serde_json::from_value(other.data)?;
                if lock.active {
                    bail!(StateGuardError::WorktreeLocked);
                }
            }
            RecordKind::Session if acquiring || executor => {
                let session: Session = serde_json::from_value(other.data)?;
                if execution::logically_retired_session(tx, &session)? {
                    continue;
                }
                if executor_reserved(&session) {
                    bail!(StateGuardError::ExecutorReserved);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_scope(scope: &Scope) -> Result<()> {
    ensure!(
        scope.task_id.is_none() || scope.goal_id.is_some(),
        "task scope requires goal identity"
    );
    Ok(())
}
fn context_owner(scope: &Scope) -> Result<String> {
    validate_scope(scope)?;
    if let Some(task) = scope.task_id {
        Ok(format!("task:{task}"))
    } else {
        Ok(format!(
            "goal:{}",
            scope.goal_id.context("context scope needs goal/task")?
        ))
    }
}
fn bump(version: &mut u64) -> Result<()> {
    *version = version
        .checked_add(1)
        .context("snapshot version overflow")?;
    Ok(())
}
fn read_tx<T: DeserializeOwned>(tx: &Connection, table: &str, id: &str) -> Result<Option<T>> {
    let body: Option<String> = tx
        .query_row(
            &format!("SELECT body FROM {table} WHERE id=?1"),
            [id],
            |row| row.get(0),
        )
        .optional()?;
    body.map(decode).transpose()
}

#[allow(clippy::too_many_arguments)]
fn write_snapshot(
    tx: &Transaction<'_>,
    table: &str,
    id: &str,
    expected: u64,
    insert: &str,
    insert_params: impl rusqlite::Params,
    body: &str,
    version: u64,
) -> Result<()> {
    if expected == 0 {
        tx.execute(insert, insert_params)
            .context("snapshot insert failed")?;
    } else {
        let changed = tx.execute(
            &format!("UPDATE {table} SET version=?1,body=?2 WHERE id=?3 AND version=?4"),
            params![version, body, id, expected],
        )?;
        if changed != 1 {
            bail!(StateGuardError::SnapshotChanged {
                table: table.into(),
                id: id.into(),
                expected
            });
        }
    }
    Ok(())
}
fn append_event(tx: &Transaction<'_>, scope: &Scope, kind: &str, data: Value) -> Result<()> {
    tx.execute(
        "INSERT INTO audit(project_id,goal_id,task_id,kind,at,data) VALUES(?1,?2,?3,?4,?5,?6)",
        params![
            scope.project_id.to_string(),
            str_id(scope.goal_id),
            str_id(scope.task_id),
            kind,
            now_ms(),
            serde_json::to_string(&data)?
        ],
    )?;
    Ok(())
}

fn reserved_audit_kind(kind: &str) -> bool {
    kind.ends_with(".saved")
        || matches!(
            kind,
            "context.created" | "usage.recorded" | "workflow.gate_observed"
        )
}
