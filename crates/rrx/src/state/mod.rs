//! Transactional SQLite snapshots + append-only logical events, scoped by Project.
use std::{path::Path, time::Duration};

use anyhow::{Context, Result, bail, ensure};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::domain::*;

pub const SCHEMA_VERSION: i64 = 1;
pub const APPLICATION_ID: i64 = 0x52525831; // ASCII RRX1.

pub struct Store {
    connection: Connection,
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
        if version == 0 {
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
                tx.pragma_update(None, "application_id", APPLICATION_ID)?;
                tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            } else {
                ensure!(
                    locked_version == SCHEMA_VERSION,
                    "unsupported state schema {locked_version}"
                );
                let application: i64 =
                    tx.pragma_query_value(None, "application_id", |row| row.get(0))?;
                ensure!(application == APPLICATION_ID, "not an rrx state database");
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
        if let Some(previous) = read_tx::<Project>(&tx, "projects", &project.id.to_string())? {
            ensure!(
                previous.root == project.root
                    && previous.repository_identity == project.repository_identity,
                "project identity/root cannot silently change"
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
                        && !other.worktree_root.starts_with(&project.worktree_root),
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
        ensure!(
            !task.title.trim().is_empty() && !task.executor.trim().is_empty(),
            "task title/executor must be nonempty"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(previous) = read_tx::<Task>(&tx, "tasks", &task.id.to_string())? {
            ensure!(
                previous.project_id == task.project_id && previous.goal_id == task.goal_id,
                "task ownership is immutable"
            );
            ensure!(
                previous
                    .worktree
                    .as_ref()
                    .is_none_or(|path| task.worktree.as_ref() == Some(path))
                    && previous
                        .branch
                        .as_ref()
                        .is_none_or(|branch| task.branch.as_ref() == Some(branch)),
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
            let project: Project = read_tx(&tx, "projects", &task.project_id.to_string())?
                .context("unknown project")?;
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
            let mut statement =
                tx.prepare("SELECT body FROM tasks WHERE project_id=?1 AND id<>?2")?;
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
            &tx,
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
        tx.execute(
            "UPDATE tasks SET issue=?1 WHERE id=?2",
            params![next.issue, next.id.to_string()],
        )?;
        append_event(
            &tx,
            &next.scope(),
            "task.saved",
            json!({"version":next.version,"state":next.state,"phase":next.phase,"workflow":next.workflow}),
        )?;
        tx.commit()?;
        *task = next;
        Ok(())
    }

    pub fn put_record(&mut self, record: &mut Record) -> Result<()> {
        validate_scope(&record.scope)?;
        if record.kind == RecordKind::Session {
            let session: Session =
                serde_json::from_value(record.data.clone()).context("invalid session payload")?;
            ensure!(
                session.scope == record.scope && session.id.0 == record.id.0,
                "session identity/scope mismatch"
            );
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(previous) = read_tx::<Record>(&tx, "records", &record.id.to_string())? {
            ensure!(
                previous.scope == record.scope && previous.kind == record.kind,
                "record scope/kind is immutable"
            );
        }
        if record.kind == RecordKind::Session
            && let Some(previous) = read_tx::<Record>(&tx, "records", &record.id.to_string())?
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
        validate_worktree_exclusion(&tx, record)?;
        let mut next = record.clone();
        bump(&mut next.version)?;
        next.updated_at = now_ms();
        let body = serde_json::to_string(&next)?;
        write_snapshot(
            &tx,
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
            &tx,
            &next.scope,
            &format!("{}.saved", next.kind.key()),
            json!({"id":next.id,"version":next.version,"evidence": match next.kind {
                RecordKind::Review | RecordKind::Approval | RecordKind::WorktreeLock => next.data.clone(),
                RecordKind::Session => json!({"state":next.data["state"],"agent":next.data["agent"],"provider":next.data["provider"],"role":next.data["role"],"native_ref":next.data["native_ref"]}),
                _ => Value::Null,
            }}),
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
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let latest: u64 = tx.query_row("SELECT COALESCE(MAX(version),0) FROM context_versions WHERE project_id=?1 AND owner=?2",
            params![context.scope.project_id.to_string(), owner], |row| row.get(0))?;
        ensure!(
            context.version == latest.checked_add(1).context("context version overflow")?,
            "context versions must be consecutive, expected {}",
            latest + 1
        );
        tx.execute("INSERT INTO context_versions(project_id,goal_id,task_id,owner,version,body) VALUES(?1,?2,?3,?4,?5,?6)",
            params![context.scope.project_id.to_string(), str_id(context.scope.goal_id), str_id(context.scope.task_id), owner, context.version, serde_json::to_string(context)?])?;
        append_event(
            &tx,
            &context.scope,
            "context.created",
            json!({"version":context.version,"revision":context.revision}),
        )?;
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
        let mut statement = self.connection.prepare("SELECT body FROM usage WHERE project_id=?1 AND (?2 IS NULL OR goal_id=?2) AND (?3 IS NULL OR task_id=?3) ORDER BY sequence")?;
        let rows = statement.query_map(
            params![
                scope.project_id.to_string(),
                str_id(scope.goal_id),
                str_id(scope.task_id)
            ],
            |row| row.get::<_, String>(0),
        )?;
        rows.map(|row| decode(row?)).collect()
    }

    pub fn audit(&mut self, scope: &Scope, kind: &str, data: Value) -> Result<()> {
        validate_scope(scope)?;
        ensure!(!kind.trim().is_empty(), "audit kind must be nonempty");
        ensure!(
            !kind.ends_with(".saved") && kind != "context.created" && kind != "usage.recorded",
            "audit kind is reserved for Store mutations"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
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

fn str_id<T: std::fmt::Display>(id: Option<T>) -> Option<String> {
    id.map(|id| id.to_string())
}
fn validate_goal_references(tx: &Transaction<'_>, goal: &Goal) -> Result<()> {
    use std::collections::BTreeSet;
    let nodes: BTreeSet<_> = goal.dag.nodes.iter().copied().collect();
    ensure!(
        nodes.len() == goal.dag.nodes.len(),
        "duplicate goal DAG nodes"
    );
    let mut references = nodes.clone();
    for edge in &goal.dag.edges {
        ensure!(
            nodes.contains(&edge.prerequisite) && nodes.contains(&edge.dependent),
            "DAG edge endpoints must be declared nodes"
        );
        references.insert(edge.prerequisite);
        references.insert(edge.dependent);
    }
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
                ensure!(
                    !lock.active,
                    "worktree has an active immutable/maintenance lock"
                );
            }
            RecordKind::Session if acquiring || executor => {
                let session: Session = serde_json::from_value(other.data)?;
                ensure!(!executor_reserved(&session), "executor is reserved/live");
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
fn read_tx<T: DeserializeOwned>(tx: &Transaction<'_>, table: &str, id: &str) -> Result<Option<T>> {
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
            bail!("stale snapshot {table}/{id}, expected version {expected}");
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
