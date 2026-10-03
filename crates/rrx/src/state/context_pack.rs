//! Context publication CAS and checkpoint append share one authoritative SQLite transaction.
use super::*;
use crate::context_pack::{Checkpoint, CheckpointRef};
use sha2::{Digest, Sha256};

impl Store {
    pub(crate) fn publish_context_pack(
        &mut self,
        binding: &Scope,
        expected: [u64; 3],
        context: &ContextVersion,
        task_versions: &[(TaskId, u64)],
    ) -> Result<()> {
        ensure!(
            context.scope == *binding
                || context.scope
                    == Scope::goal(binding.project_id, binding.goal_id.context("missing Goal")?),
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
        let latest:Option<String>=tx.query_row("SELECT body FROM records WHERE kind='checkpoint' AND project_id=?1 AND goal_id=?2 AND task_id=?3 AND json_extract(body,'$.data.format')='rrx.checkpoint.v1' ORDER BY json_extract(body,'$.data.chain_version') DESC LIMIT 1",
            params![scope.project_id.to_string(),str_id(scope.goal_id),str_id(scope.task_id)],|r|r.get(0)).optional()?;
        let latest: Option<Record> = latest.map(decode).transpose()?;
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
        append_event(
            &tx,
            scope,
            "checkpoint.saved",
            json!({"id":next.id,"version":1,"session":session.id,"chain_version":checkpoint.chain_version,
            "first_sequence":checkpoint.first_sequence,"last_sequence":checkpoint.last_sequence,"input_digest":checkpoint.input_digest,
            "retained":checkpoint.retained.len(),"recent":checkpoint.recent.len(),"recent_bytes":checkpoint.recent_bytes,
            "omitted_transient":checkpoint.omitted_transient,"measured_tokens":Value::Null}),
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

impl Store {
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
        bump(&mut g.version)?;
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
            "goal.saved",
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
