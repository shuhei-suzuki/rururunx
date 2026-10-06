use std::collections::BTreeMap;

use rrx::{domain::*, state::Store};
use serde_json::json;
#[path = "support/current_writer.rs"]
mod current_writer;

fn project(store: &mut Store, name: &str, root: &std::path::Path) -> Project {
    let mut project = Project::new(
        name.into(),
        root.to_path_buf(),
        format!("repo:{name}"),
        "main".into(),
    );
    store.put_project(&mut project).unwrap();
    project
}
fn goal(store: &mut Store, project: &Project) -> Goal {
    let mut goal = Goal::new(
        project.id,
        "Complete MVP".into(),
        vec![CompletionCriterion {
            evaluator: Default::default(),
            id: "tests".into(),
            description: "tests pass".into(),
            evidence: None,
            satisfied: false,
        }],
    );
    store.put_goal(&mut goal).unwrap();
    goal
}
fn task(store: &mut Store, project: &Project, goal: &Goal) -> Task {
    let mut task = Task::new(project.id, goal.id, "Implement #42".into(), "fake".into());
    task.issue = Some(42);
    store.put_task(&mut task).unwrap();
    task
}
fn session(task: &Task, worktree: std::path::PathBuf) -> Session {
    Session {
        id: SessionId::new(),
        scope: task.scope(),
        agent: "fake".into(),
        provider: "test".into(),
        role: SessionRole::Executor,
        native_ref: Some("native-stable-id".into()),
        pid: Some(123456),
        worktree,
        state: SessionState::Running,
        model: None,
        effort: None,
        recovery: json!({"pty":"session.sock"}),
        started_at: now_ms(),
    }
}

#[test]
fn current_scope_audit_rechecks_versions_and_activity_across_connections() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("state.db");
    let mut first = Store::open(&db).unwrap();
    let mut p = project(&mut first, "one", temp.path());
    let g = goal(&mut first, &p);
    let mut t = task(&mut first, &p, &g);
    let scope = t.scope();
    let old = [p.version, g.version, t.version];
    let mut second = Store::open(&db).unwrap();
    t.title = "independent connection update".into();
    second.put_task(&mut t).unwrap();
    let before = first.events(&scope, 0, 100).unwrap().len();
    let error = first
        .audit_if_current(&scope, old, "context.selection", json!({"ready":true}))
        .unwrap_err();
    assert!(matches!(
        error.downcast_ref::<rrx::state::StateGuardError>(),
        Some(rrx::state::StateGuardError::SnapshotChanged { .. })
    ));
    assert_eq!(first.events(&scope, 0, 100).unwrap().len(), before);
    first
        .audit_if_current(
            &scope,
            [p.version, g.version, t.version],
            "context.selection",
            json!({"ready":true}),
        )
        .unwrap();
    p.state = ProjectState::Blocked;
    p.blocked_reason = Some("source changed".into());
    second.put_project(&mut p).unwrap();
    assert!(
        first
            .audit_if_current(
                &scope,
                [p.version, g.version, t.version],
                "context.selection",
                json!({"ready":true})
            )
            .is_err()
    );
    assert!(
        first
            .audit_if_current(
                &scope,
                [p.version, g.version, t.version],
                "context.created",
                json!({})
            )
            .is_err()
    );
}

#[test]
fn restart_preserves_hierarchy_decisions_dag_sessions_context_and_nullable_usage() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("state.db");
    let mut store = Store::open(&db).unwrap();
    assert_eq!(store.schema_version().unwrap(), rrx::state::SCHEMA_VERSION);
    let project = project(&mut store, "one", dir.path());
    let mut goal = goal(&mut store, &project);
    let mut task = task(&mut store, &project, &goal);
    task.worktree = Some(dir.path().join("worktree/issue-42"));
    task.branch = Some("feature/issue-42".into());
    task.state = TaskState::WaitingApproval;
    task.phase = Some("implement".into());
    task.context_version = 1;
    store.put_task(&mut task).unwrap();
    goal.state = GoalState::Running;
    goal.dag.nodes.push(task.id);
    goal.context_version = 1;
    store.put_goal(&mut goal).unwrap();
    let session = session(&task, task.worktree.clone().unwrap());
    store.put_session(&session, 0).unwrap();
    let mut review = Record::new(
        task.scope(),
        RecordKind::Review,
        json!({"round":1,"reviewers":["claude","codex"],"completion":"all","findings":[{"severity":"HIGH","verified":false}]}),
    );
    store.put_record(&mut review).unwrap();
    let mut approval = Record::new(
        task.scope(),
        RecordKind::Approval,
        json!({"request":"install dependency","decision":"ESCALATE","reason":"reviewer disagreement"}),
    );
    store.put_record(&mut approval).unwrap();
    let context = ContextVersion {
        scope: task.scope(),
        version: 1,
        revision: "immutable-head".into(),
        source_hashes: BTreeMap::from([("requirements".into(), "abc".into())]),
        data: json!({"goal":"Implement #42","next":"human decision","constraints":["keep permissions"]}),
    };
    store.put_context(&context).unwrap();
    let usage = Usage {
        scope: task.scope(),
        session_id: session.id,
        agent: session.agent.clone(),
        phase: "implement".into(),
        review_round: None,
        input_tokens: Some(120),
        cached_input_tokens: Some(80),
        output_tokens: Some(20),
        estimated_cost: None,
        context_pack_version: Some(1),
        context_pack_size: Some(200),
        repo_map_size: Some(100),
        cache_metadata: json!({"native":true}),
        missing_reason: Some("cost unavailable".into()),
    };
    store.put_usage(&usage).unwrap();
    drop(store);

    let mut restored = Store::open(&db).unwrap();
    assert_eq!(
        restored
            .project(project.id)
            .unwrap()
            .unwrap()
            .repository_identity,
        "repo:one"
    );
    let restored_goal = restored.goal(goal.id).unwrap().unwrap();
    assert_eq!(restored_goal.state, GoalState::Running);
    assert_eq!(restored_goal.dag.nodes, [task.id]);
    assert!(!restored_goal.completion_criteria[0].satisfied);
    assert_eq!(
        restored.task(task.id).unwrap().unwrap().state,
        TaskState::WaitingApproval
    );
    let (native, version) = restored.session(session.id).unwrap().unwrap();
    assert_eq!(native.id, session.id);
    assert_eq!(native.native_ref.as_deref(), Some("native-stable-id"));
    assert_eq!(version, 1);
    assert_eq!(
        restored.records(&task.scope(), RecordKind::Review).unwrap()[0].data["round"],
        1
    );
    assert_eq!(
        restored
            .records(&task.scope(), RecordKind::Approval)
            .unwrap()[0]
            .data["decision"],
        "ESCALATE"
    );
    assert_eq!(
        restored
            .context(&task.scope(), None)
            .unwrap()
            .unwrap()
            .revision,
        "immutable-head"
    );
    let metrics = restored.usage(&task.scope()).unwrap();
    assert_eq!(metrics[0].cached_input_tokens, Some(80));
    assert_eq!(metrics[0].estimated_cost, None);
    let events = restored
        .events(&Scope::project(project.id), 0, 100)
        .unwrap();
    assert!(events.iter().any(|event| event.kind == "approval.saved"));
    assert!(
        events
            .windows(2)
            .all(|pair| pair[0].sequence < pair[1].sequence)
    );
    restored
        .audit(&task.scope(), "human.decision", json!({"decision":"DENY"}))
        .unwrap();
    assert_eq!(
        restored
            .events(&task.scope(), events.last().unwrap().sequence, 100)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn project_identity_prevents_issue_collisions_and_rejects_cross_project_children() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::memory().unwrap();
    let one = project(&mut store, "one", &dir.path().join("one"));
    let two = project(&mut store, "two", &dir.path().join("two"));
    let goal_one = goal(&mut store, &one);
    let goal_two = goal(&mut store, &two);
    let task_one = task(&mut store, &one, &goal_one);
    let task_two = task(&mut store, &two, &goal_two);
    assert_ne!(task_one.id, task_two.id);
    assert_eq!(store.tasks(one.id, None).unwrap().len(), 1);
    assert_eq!(store.tasks(two.id, None).unwrap().len(), 1);
    let count = store.events(&Scope::project(two.id), 0, 100).unwrap().len();
    let mut invalid = Task::new(two.id, goal_one.id, "wrong project".into(), "fake".into());
    assert!(store.put_task(&mut invalid).is_err());
    assert_eq!(invalid.version, 0);
    assert_eq!(store.tasks(two.id, None).unwrap().len(), 1);
    assert_eq!(
        store.events(&Scope::project(two.id), 0, 100).unwrap().len(),
        count
    );
    let mut cross = Record::new(
        Scope::task(two.id, goal_two.id, task_one.id),
        RecordKind::Approval,
        json!({}),
    );
    assert!(store.put_record(&mut cross).is_err());
    let mut rebound = store.goal(goal_one.id).unwrap().unwrap();
    rebound.project_id = two.id;
    assert!(store.put_goal(&mut rebound).is_err());
    let mut moved = store.project(one.id).unwrap().unwrap();
    moved.root = two.root;
    assert!(store.put_project(&mut moved).is_err());
}

#[test]
fn stale_writers_do_not_overwrite_state_or_append_events() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("state.db");
    let mut writer = Store::open(&db).unwrap();
    let project = project(&mut writer, "one", dir.path());
    let goal = goal(&mut writer, &project);
    let mut task = task(&mut writer, &project, &goal);
    let mut other = Store::open(&db).unwrap();
    let mut stale = other.task(task.id).unwrap().unwrap();
    task.state = TaskState::Testing;
    writer.put_task(&mut task).unwrap();
    let count = writer.events(&task.scope(), 0, 100).unwrap().len();
    stale.state = TaskState::Completed;
    assert!(
        other
            .put_task(&mut stale)
            .unwrap_err()
            .to_string()
            .contains("stale snapshot")
    );
    assert_eq!(
        writer.task(task.id).unwrap().unwrap().state,
        TaskState::Testing
    );
    assert_eq!(writer.events(&task.scope(), 0, 100).unwrap().len(), count);
}

#[test]
fn append_only_audit_and_failed_journal_write_roll_back_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("state.db");
    let mut store = Store::open(&db).unwrap();
    let project = project(&mut store, "one", dir.path());
    let goal = goal(&mut store, &project);
    let mut task = task(&mut store, &project, &goal);
    let raw = current_writer::open(&db).unwrap();
    assert!(raw.execute("UPDATE audit SET kind='forged'", []).is_err());
    assert!(raw.execute("DELETE FROM audit", []).is_err());
    raw.execute_batch("CREATE TRIGGER fail_task_audit BEFORE INSERT ON audit WHEN NEW.kind='task.saved' BEGIN SELECT RAISE(ABORT,'injected journal failure'); END;").unwrap();
    let original = task.version;
    task.state = TaskState::Completed;
    assert!(store.put_task(&mut task).is_err());
    assert_eq!(task.version, original);
    let restored = store.task(task.id).unwrap().unwrap();
    assert_eq!(restored.state, TaskState::Created);
    assert_eq!(restored.version, original);
}

#[test]
fn context_versions_are_immutable_consecutive_and_scoped() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::memory().unwrap();
    let project = project(&mut store, "one", dir.path());
    let goal = goal(&mut store, &project);
    let task = task(&mut store, &project, &goal);
    let mut context = ContextVersion {
        scope: task.scope(),
        version: 1,
        revision: "a".into(),
        source_hashes: BTreeMap::new(),
        data: json!({"next":"review"}),
    };
    store.put_context(&context).unwrap();
    assert!(store.put_context(&context).is_err());
    context.version = 3;
    assert!(store.put_context(&context).is_err());
    context.version = 2;
    context.revision = "b".into();
    store.put_context(&context).unwrap();
    assert_eq!(
        store
            .context(&task.scope(), Some(1))
            .unwrap()
            .unwrap()
            .revision,
        "a"
    );
    assert_eq!(
        store
            .context(&task.scope(), None)
            .unwrap()
            .unwrap()
            .revision,
        "b"
    );
    assert!(
        store
            .context(&Scope::task(project.id, GoalId::new(), task.id), None)
            .is_err()
    );
}

#[test]
fn usage_missing_is_explicit_and_session_ownership_is_enforced() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::memory().unwrap();
    let project = project(&mut store, "one", dir.path());
    let goal = goal(&mut store, &project);
    let task = task(&mut store, &project, &goal);
    let mut task = task;
    task.worktree = Some(dir.path().join("worktree/issue-42"));
    task.branch = Some("feature/issue-42".into());
    store.put_task(&mut task).unwrap();
    let session = session(&task, task.worktree.clone().unwrap());
    store.put_session(&session, 0).unwrap();
    let mut usage = Usage {
        scope: task.scope(),
        session_id: session.id,
        agent: "fake".into(),
        phase: "execute".into(),
        review_round: None,
        input_tokens: None,
        cached_input_tokens: None,
        output_tokens: None,
        estimated_cost: None,
        context_pack_version: None,
        context_pack_size: None,
        repo_map_size: None,
        cache_metadata: json!({}),
        missing_reason: None,
    };
    assert!(store.put_usage(&usage).is_err());
    usage.missing_reason = Some("native CLI exposes no usage".into());
    store.put_usage(&usage).unwrap();
    usage.scope.task_id = Some(TaskId::new());
    assert!(store.put_usage(&usage).is_err());
    assert_eq!(store.usage(&task.scope()).unwrap().len(), 1);
}

#[test]
fn future_schema_is_rejected_without_rewriting_database() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("future.db");
    let raw = current_writer::open(&db).unwrap();
    raw.pragma_update(None, "application_id", rrx::state::APPLICATION_ID)
        .unwrap();
    raw.pragma_update(None, "user_version", 999).unwrap();
    drop(raw);
    let before = std::fs::read(&db).unwrap();
    let error = Store::open(&db).err().unwrap();
    assert!(error.to_string().contains("unsupported state schema 999"));
    assert_eq!(std::fs::read(&db).unwrap(), before);
    let raw = current_writer::open(db).unwrap();
    let version: i64 = raw
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 999);
}

#[test]
fn changing_decisions_preserves_previous_round_evidence_in_journal() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::memory().unwrap();
    let project = project(&mut store, "one", dir.path());
    let goal = goal(&mut store, &project);
    let task = task(&mut store, &project, &goal);
    let mut approval = Record::new(
        task.scope(),
        RecordKind::Approval,
        json!({"decision":"ESCALATE","reason":"conflict"}),
    );
    store.put_record(&mut approval).unwrap();
    approval.data = json!({"decision":"DENY","reason":"human decision"});
    store.put_record(&mut approval).unwrap();
    let decisions: Vec<_> = store
        .events(&task.scope(), 0, 100)
        .unwrap()
        .into_iter()
        .filter(|event| event.kind == "approval.saved")
        .collect();
    assert_eq!(decisions.len(), 2);
    assert_eq!(decisions[0].data["evidence"]["decision"], "ESCALATE");
    assert_eq!(decisions[1].data["evidence"]["decision"], "DENY");
    assert_eq!(store.record(approval.id).unwrap().unwrap().version, 2);
}

#[test]
fn audit_replacement_upsert_and_backdated_sequence_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("state.db");
    let mut store = Store::open(&db).unwrap();
    let project = project(&mut store, "one", dir.path());
    let raw = current_writer::open(&db).unwrap();
    let audit_image = || {
        raw.query_row(
        "SELECT json_group_array(json_object('sequence',sequence,'project_id',project_id,'goal_id',goal_id,'task_id',task_id,'kind',kind,'at',at,'data',data)) FROM (SELECT * FROM audit ORDER BY sequence)",
        [], |row| row.get::<_, String>(0),
    ).unwrap()
    };
    let project_image: String = raw
        .query_row(
            "SELECT body FROM projects WHERE id=?1",
            [project.id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    let original = audit_image();
    for sql in [
        "REPLACE INTO audit(sequence,project_id,kind,at,data) VALUES(1,?1,'forged',0,'{}')",
        "INSERT OR REPLACE INTO audit(sequence,project_id,kind,at,data) VALUES(1,?1,'forged',0,'{}')",
        "INSERT INTO audit(sequence,project_id,kind,at,data) VALUES(1,?1,'forged',0,'{}') ON CONFLICT(sequence) DO UPDATE SET kind='forged'",
    ] {
        assert!(
            raw.execute(sql, [project.id.to_string()]).is_err(),
            "accepted {sql}"
        );
        assert_eq!(audit_image(), original);
    }
    raw.execute(
        "INSERT INTO audit(sequence,project_id,kind,at,data) VALUES(50,?1,'future',0,'{}')",
        [project.id.to_string()],
    )
    .unwrap();
    let with_future = audit_image();
    assert!(
        raw.execute(
            "INSERT INTO audit(sequence,project_id,kind,at,data) VALUES(2,?1,'backdated',0,'{}')",
            [project.id.to_string()]
        )
        .is_err()
    );
    assert_eq!(audit_image(), with_future);
    assert_eq!(
        raw.query_row(
            "SELECT body FROM projects WHERE id=?1",
            [project.id.to_string()],
            |row| row.get::<_, String>(0)
        )
        .unwrap(),
        project_image
    );
    assert_eq!(
        store.events(&Scope::project(project.id), 0, 100).unwrap()[0].kind,
        "project.saved"
    );
}

#[test]
fn nongrant_current_writer_denies_protected_insert_and_session_projection() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("nongrant.db");
    let mut store = Store::open(&db).unwrap();
    let project = project(&mut store, "one", dir.path());
    let raw = current_writer::open(&db).unwrap();
    let changes = raw.total_changes();
    let schema: String = raw.query_row("SELECT group_concat(sql,';') FROM (SELECT sql FROM sqlite_schema WHERE sql IS NOT NULL ORDER BY name)",[],|row|row.get(0)).unwrap();
    // Deliberately invalid corruption attempt, never a retained Driver fixture.
    let denied = raw.execute("INSERT INTO task_drivers(task_id,goal_id,project_id,id,owner_epoch,version,state,body) VALUES('canary','canary',?1,'canary',0,0,'invalid','{}')",[project.id.to_string()]);
    assert!(denied.is_err(), "nongrant protected insertion was allowed");
    assert!(
        denied
            .unwrap_err()
            .to_string()
            .contains("private retained Driver writer required")
    );
    assert_eq!(raw.total_changes(), changes);
    assert_eq!(
        raw.query_row("SELECT count(*) FROM task_drivers", [], |row| row
            .get::<_, usize>(0))
            .unwrap(),
        0
    );
    let mut session = Record::new(Scope::project(project.id), RecordKind::Session, json!({}));
    session.version = 1;
    let projection = raw.execute(
        "INSERT INTO records(id,kind,project_id,goal_id,task_id,version,body) VALUES(?1,'session',?2,NULL,NULL,1,?3)",
        rusqlite::params![session.id.to_string(),project.id.to_string(),serde_json::to_string(&session).unwrap()],
    );
    assert!(
        projection.is_err(),
        "nongrant connection projected Session identity"
    );
    assert!(
        projection
            .unwrap_err()
            .to_string()
            .contains("nongrant corruption canary cannot project Session identity")
    );
    for table in ["records", "scoped_session_identities"] {
        assert_eq!(
            raw.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                row.get::<_, usize>(0)
            })
            .unwrap(),
            0,
            "rejected Session insertion changed {table}"
        );
    }
    assert_eq!(raw.query_row("SELECT group_concat(sql,';') FROM (SELECT sql FROM sqlite_schema WHERE sql IS NOT NULL ORDER BY name)",[],|row|row.get::<_,String>(0)).unwrap(),schema);
    assert_eq!(
        serde_json::to_value(store.project(project.id).unwrap().unwrap()).unwrap(),
        serde_json::to_value(project).unwrap()
    );
}

#[test]
fn dag_and_followup_references_cannot_cross_project_or_goal_boundaries() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::memory().unwrap();
    let one = project(&mut store, "one", &dir.path().join("one"));
    let two = project(&mut store, "two", &dir.path().join("two"));
    let mut goal_one = goal(&mut store, &one);
    let goal_two = goal(&mut store, &two);
    let foreign_task = task(&mut store, &two, &goal_two);
    goal_one.dag.nodes.push(foreign_task.id);
    assert!(store.put_goal(&mut goal_one).is_err());
    goal_one.dag.nodes.clear();
    goal_one.followups.push(FollowupProposal {
        title: "follow-up".into(),
        scope: "same goal".into(),
        rationale: "needed".into(),
        acceptance_criteria: vec!["done".into()],
        dependencies: vec![foreign_task.id],
        risk: RiskClass::R1,
        material_scope_expansion: false,
        disposition: "proposed".into(),
    });
    assert!(store.put_goal(&mut goal_one).is_err());
    goal_one.followups.clear();
    let local_task = task(&mut store, &one, &goal_one);
    goal_one.dag.edges.push(Dependency {
        prerequisite: local_task.id,
        dependent: foreign_task.id,
        hard: true,
    });
    assert!(store.put_goal(&mut goal_one).is_err());
    goal_one.dag.edges.clear();
    goal_one.dag.nodes.push(local_task.id);
    let undeclared = task(&mut store, &one, &goal_one);
    goal_one.dag.edges.push(Dependency {
        prerequisite: local_task.id,
        dependent: undeclared.id,
        hard: true,
    });
    assert!(store.put_goal(&mut goal_one).is_err());
    goal_one.dag.edges.clear();
    store.put_goal(&mut goal_one).unwrap();
}

#[test]
fn foreign_databases_and_unknown_snapshots_are_rejected_without_data_loss() {
    let dir = tempfile::tempdir().unwrap();
    for (table, version) in [("other_app", 0), ("other_app", 1), ("sqlite3_data", 0)] {
        let db = dir.path().join(format!("foreign-{table}-{version}.db"));
        let raw = current_writer::open(&db).unwrap();
        raw.execute_batch(&format!(
            "CREATE TABLE {table}(value TEXT); INSERT INTO {table} VALUES('keep');"
        ))
        .unwrap();
        raw.pragma_update(None, "user_version", version).unwrap();
        drop(raw);
        let before = std::fs::read(&db).unwrap();
        assert!(Store::open(&db).is_err());
        assert_eq!(std::fs::read(&db).unwrap(), before);
    }
    let db = dir.path().join("state.db");
    let mut store = Store::open(&db).unwrap();
    let project = project(&mut store, "one", dir.path());
    let raw = current_writer::open(&db).unwrap();
    raw.execute(
        "UPDATE projects SET body=json_set(body,'$.future_secret_field','preserve') WHERE id=?1",
        [project.id.to_string()],
    )
    .unwrap();
    assert!(store.project(project.id).is_err());
    assert!(
        store
            .audit(
                &Scope::project(project.id),
                "approval.saved",
                json!({"decision":"ALLOW"})
            )
            .is_err()
    );
}

#[test]
fn context_versions_reject_sql_update_delete_and_replace() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("state.db");
    let mut store = Store::open(&db).unwrap();
    let project = project(&mut store, "one", dir.path());
    let goal = goal(&mut store, &project);
    let context = ContextVersion {
        scope: goal.scope(),
        version: 1,
        revision: "a".into(),
        source_hashes: BTreeMap::new(),
        data: json!({}),
    };
    store.put_context(&context).unwrap();
    let raw = current_writer::open(&db).unwrap();
    assert!(
        raw.execute("UPDATE context_versions SET body='{}'", [])
            .is_err()
    );
    assert!(raw.execute("DELETE FROM context_versions", []).is_err());
    assert!(
        raw.execute(
            "INSERT OR REPLACE INTO context_versions SELECT * FROM context_versions",
            []
        )
        .is_err()
    );
    assert_eq!(
        store
            .context(&goal.scope(), Some(1))
            .unwrap()
            .unwrap()
            .revision,
        "a"
    );
}

#[test]
fn usage_integer_overflow_cannot_publish_rows_or_audit_or_change_owners() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::memory().unwrap();
    let p = project(&mut store, "usage-range", temp.path());
    let g = goal(&mut store, &p);
    let mut t = task(&mut store, &p, &g);
    t.worktree = Some(p.worktree_root.join("task-worktree"));
    t.branch = Some("feature/usage-range".into());
    store.put_task(&mut t).unwrap();
    let s = session(&t, t.worktree.clone().unwrap());
    store.put_session(&s, 0).unwrap();
    let maximum = u64::try_from(i64::MAX).unwrap();
    let valid = Usage {
        scope: t.scope(),
        session_id: s.id,
        agent: s.agent.clone(),
        phase: "legacy-unqualified".into(),
        review_round: None,
        input_tokens: Some(maximum),
        cached_input_tokens: Some(0),
        output_tokens: Some(maximum),
        estimated_cost: None,
        context_pack_version: Some(maximum),
        context_pack_size: Some(maximum),
        repo_map_size: Some(maximum),
        cache_metadata: json!({"legacy":true}),
        missing_reason: Some("legacy observations have no qualified counter contract".into()),
    };
    // The exact representable boundary and a real zero are persisted unchanged.
    store.put_usage(&valid).unwrap();
    let saved = store.usage(&t.scope()).unwrap();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].input_tokens, Some(maximum));
    assert_eq!(saved[0].cached_input_tokens, Some(0));
    assert_eq!(saved[0].output_tokens, Some(maximum));
    assert_eq!(saved[0].context_pack_version, Some(maximum));
    assert_eq!(saved[0].context_pack_size, Some(maximum));
    assert_eq!(saved[0].repo_map_size, Some(maximum));
    let before = serde_json::to_value((
        store.project(p.id).unwrap(),
        store.goal(g.id).unwrap(),
        store.task(t.id).unwrap(),
        store.session(s.id).unwrap(),
        saved,
        store.events(&t.scope(), 0, 100).unwrap(),
    ))
    .unwrap();
    // Independent fields must each refuse at the actual write consumer, even
    // when the other fields and genuine existing Session are otherwise valid.
    for field in [
        "input_tokens",
        "cached_input_tokens",
        "output_tokens",
        "context_pack_version",
        "context_pack_size",
        "repo_map_size",
    ] {
        for overflow in [maximum + 1, u64::MAX] {
            let mut body = serde_json::to_value(&valid).unwrap();
            body[field] = json!(overflow);
            let invalid: Usage = serde_json::from_value(body).unwrap();
            let error = store
                .put_usage(&invalid)
                .expect_err(&format!("overflow stored for {field}"));
            assert!(
                error.to_string().contains("supported storage range"),
                "{field}: {error}"
            );
            let after = serde_json::to_value((
                store.project(p.id).unwrap(),
                store.goal(g.id).unwrap(),
                store.task(t.id).unwrap(),
                store.session(s.id).unwrap(),
                store.usage(&t.scope()).unwrap(),
                store.events(&t.scope(), 0, 100).unwrap(),
            ))
            .unwrap();
            assert_eq!(after, before, "overflow published data for {field}");
        }
    }
}

// Snapshot all persisted usage and audit rows, including Project/foreign events.
// Refusal must neither repair history nor append a diagnostic event.
fn legacy_usage_rows_snapshot(raw: &rusqlite::Connection) -> serde_json::Value {
    let mut usage = raw.prepare("SELECT sequence,project_id,goal_id,task_id,session_id,body FROM usage ORDER BY sequence").unwrap();
    let usage = usage
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut audit = raw
        .prepare(
            "SELECT sequence,project_id,goal_id,task_id,kind,at,data FROM audit ORDER BY sequence",
        )
        .unwrap();
    let audit = audit
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    json!({"usage":usage, "audit":audit})
}

#[test]
fn legacy_usage_query_checks_row_body_identity_and_invalid_scope_without_writes() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("usage-scope.db");
    let mut store = Store::open(&db).unwrap();
    let p = project(&mut store, "usage-scope", &temp.path().join("one"));
    let g = goal(&mut store, &p);
    let mut t = task(&mut store, &p, &g);
    t.worktree = Some(p.worktree_root.join("task-worktree"));
    t.branch = Some("feature/usage-scope".into());
    store.put_task(&mut t).unwrap();
    let s = session(&t, t.worktree.clone().unwrap());
    store.put_session(&s, 0).unwrap();
    let valid = Usage {
        scope: t.scope(),
        session_id: s.id,
        agent: s.agent.clone(),
        phase: "legacy-unqualified".into(),
        review_round: None,
        input_tokens: Some(7),
        cached_input_tokens: None,
        output_tokens: Some(3),
        estimated_cost: None,
        context_pack_version: None,
        context_pack_size: None,
        repo_map_size: None,
        cache_metadata: json!({"legacy":true}),
        missing_reason: Some("legacy history is unqualified".into()),
    };
    store.put_usage(&valid).unwrap();
    let foreign_p = project(&mut store, "foreign-usage-scope", &temp.path().join("two"));
    let foreign_g = goal(&mut store, &foreign_p);
    let foreign_t = task(&mut store, &foreign_p, &foreign_g);
    // Both legitimate Tasks have Issue42, but distinct Project identities.
    assert_eq!(foreign_t.issue, t.issue);
    let views = [
        t.scope(),
        Scope {
            project_id: p.id,
            goal_id: Some(g.id),
            task_id: None,
        },
        Scope {
            project_id: p.id,
            goal_id: None,
            task_id: None,
        },
    ];
    for scope in &views {
        assert_eq!(
            serde_json::to_value(store.usage(scope).unwrap()).unwrap(),
            json!([valid])
        );
    }
    let raw = current_writer::open(&db).unwrap();
    let snapshot = || {
        json!({"persisted_rows":legacy_usage_rows_snapshot(&raw),"project":store.project(p.id).unwrap(),
            "goal":store.goal(g.id).unwrap(),"task":store.task(t.id).unwrap(),
            "session":store.session(s.id).unwrap(),
            "audit":store.events(&t.scope(),0,1000).unwrap()})
    };
    for field in [
        "project",
        "goal",
        "task",
        "session",
        "missing_goal",
        "missing_task",
    ] {
        let mut corrupt = valid.clone();
        match field {
            "project" => corrupt.scope.project_id = foreign_p.id,
            "goal" => corrupt.scope.goal_id = Some(foreign_g.id),
            "task" => corrupt.scope.task_id = Some(foreign_t.id),
            "session" => corrupt.session_id = SessionId::new(),
            "missing_goal" => corrupt.scope.goal_id = None,
            "missing_task" => corrupt.scope.task_id = None,
            _ => unreachable!(),
        }
        assert_eq!(
            raw.execute(
                "UPDATE usage SET body=?1",
                [serde_json::to_string(&corrupt).unwrap()]
            )
            .unwrap(),
            1
        );
        let before = snapshot();
        for scope in &views {
            let error = store.usage(scope).unwrap_err();
            assert_eq!(
                error.to_string(),
                "usage row/body identity mismatch",
                "field {field}"
            );
            assert_eq!(
                snapshot(),
                before,
                "query must not change corrupt history or owners: {field}"
            );
        }
    }
    raw.execute(
        "UPDATE usage SET body=?1",
        [serde_json::to_string(&valid).unwrap()],
    )
    .unwrap();
    let before = snapshot();
    let invalid = Scope {
        project_id: p.id,
        goal_id: None,
        task_id: Some(t.id),
    };
    assert_eq!(
        store.usage(&invalid).unwrap_err().to_string(),
        "task scope requires goal identity"
    );
    assert_eq!(snapshot(), before);
    for scope in &views {
        assert_eq!(
            serde_json::to_value(store.usage(scope).unwrap()).unwrap(),
            json!([valid])
        );
    }
}

#[test]
fn legacy_usage_query_preserves_null_scopes_and_refuses_null_column_wildcards() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("usage-null-scope.db");
    let mut store = Store::open(&db).unwrap();
    let p = project(&mut store, "usage-null-scope", temp.path());
    let g = goal(&mut store, &p);
    let t = task(&mut store, &p, &g);
    let project_scope = Scope {
        project_id: p.id,
        goal_id: None,
        task_id: None,
    };
    let goal_scope = Scope {
        project_id: p.id,
        goal_id: Some(g.id),
        task_id: None,
    };
    let mut sessions = Vec::new();
    let mut valid = Vec::new();
    // Public Store writers accept non-executor Project/Goal observations. These
    // remain unqualified legacy history, with no native/private ownership claim.
    for scope in [project_scope.clone(), goal_scope.clone(), t.scope()] {
        let mut s = session(&t, p.worktree_root.join("legacy-review"));
        s.scope = scope.clone();
        s.role = SessionRole::Reviewer;
        store.put_session(&s, 0).unwrap();
        let usage = Usage {
            scope,
            session_id: s.id,
            agent: s.agent.clone(),
            phase: "legacy-unqualified".into(),
            review_round: None,
            input_tokens: Some(7),
            cached_input_tokens: None,
            output_tokens: Some(3),
            estimated_cost: None,
            context_pack_version: None,
            context_pack_size: None,
            repo_map_size: None,
            cache_metadata: json!({"legacy":true}),
            missing_reason: Some("legacy history is unqualified".into()),
        };
        store.put_usage(&usage).unwrap();
        sessions.push(s);
        valid.push(usage);
    }
    let raw = current_writer::open(&db).unwrap();
    let snapshot = || {
        json!({
            "persisted_rows":legacy_usage_rows_snapshot(&raw),
            "project":store.project(p.id).unwrap(), "goal":store.goal(g.id).unwrap(),
            "task":store.task(t.id).unwrap(),
            "sessions":sessions.iter().map(|s|store.session(s.id).unwrap()).collect::<Vec<_>>()
        })
    };
    let before = snapshot();
    // Return every selected row unchanged and in insertion order, including NULL
    // row identities. Reading NULL as String must fail this positive consumer.
    assert_eq!(
        serde_json::to_value(store.usage(&project_scope).unwrap()).unwrap(),
        json!(valid)
    );
    assert_eq!(
        serde_json::to_value(store.usage(&goal_scope).unwrap()).unwrap(),
        json!([valid[1], valid[2]])
    );
    assert_eq!(
        serde_json::to_value(store.usage(&t.scope()).unwrap()).unwrap(),
        json!([valid[2]])
    );
    assert_eq!(snapshot(), before);
    for (index, field) in [(0, "goal"), (1, "task")] {
        let mut corrupt = valid[index].clone();
        if field == "goal" {
            corrupt.scope.goal_id = Some(g.id);
        } else {
            corrupt.scope.task_id = Some(t.id);
        }
        assert_eq!(
            raw.execute(
                "UPDATE usage SET body=?1 WHERE session_id=?2",
                rusqlite::params![
                    serde_json::to_string(&corrupt).unwrap(),
                    corrupt.session_id.to_string()
                ]
            )
            .unwrap(),
            1
        );
        let before = snapshot();
        for view in [&project_scope, &goal_scope, &t.scope()] {
            // Filtering uses row columns. The other rows remain readable when
            // the corrupt NULL-scoped row is outside the requested view.
            let selects_corrupt = view.goal_id.is_none() || (index == 1 && view.task_id.is_none());
            if selects_corrupt {
                assert_eq!(
                    store.usage(view).unwrap_err().to_string(),
                    "usage row/body identity mismatch",
                    "NULL {field}"
                );
            } else if view.task_id.is_some() {
                assert_eq!(
                    serde_json::to_value(store.usage(view).unwrap()).unwrap(),
                    json!([valid[2]])
                );
            } else {
                assert_eq!(
                    serde_json::to_value(store.usage(view).unwrap()).unwrap(),
                    json!([valid[1], valid[2]])
                );
            }
            assert_eq!(
                snapshot(),
                before,
                "refusal or filtered read wrote history: NULL {field}"
            );
        }
        raw.execute(
            "UPDATE usage SET body=?1 WHERE session_id=?2",
            rusqlite::params![
                serde_json::to_string(&valid[index]).unwrap(),
                valid[index].session_id.to_string()
            ],
        )
        .unwrap();
    }
    assert_eq!(snapshot(), before);
    assert_eq!(
        serde_json::to_value(store.usage(&project_scope).unwrap()).unwrap(),
        json!(valid)
    );
}

#[test]
fn legacy_usage_query_redacts_malformed_body_decode_chain_without_writes() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("usage-decode.db");
    let mut store = Store::open(&db).unwrap();
    let p = project(&mut store, "usage-decode", temp.path());
    let g = goal(&mut store, &p);
    let t = task(&mut store, &p, &g);
    let mut s = session(&t, p.worktree_root.join("legacy-review"));
    s.role = SessionRole::Reviewer;
    store.put_session(&s, 0).unwrap();
    let valid = Usage {
        scope: t.scope(),
        session_id: s.id,
        agent: s.agent.clone(),
        phase: "legacy-unqualified".into(),
        review_round: None,
        input_tokens: Some(7),
        cached_input_tokens: None,
        output_tokens: Some(3),
        estimated_cost: None,
        context_pack_version: None,
        context_pack_size: None,
        repo_map_size: None,
        cache_metadata: json!({"legacy":true}),
        missing_reason: Some("legacy history is unqualified".into()),
    };
    store.put_usage(&valid).unwrap();
    let raw = current_writer::open(&db).unwrap();
    let snapshot = || {
        json!({
            "persisted_rows":legacy_usage_rows_snapshot(&raw),
            "project":store.project(p.id).unwrap(), "goal":store.goal(g.id).unwrap(),
            "task":store.task(t.id).unwrap(), "session":store.session(s.id).unwrap()
        })
    };
    assert_eq!(
        serde_json::to_value(store.usage(&t.scope()).unwrap()).unwrap(),
        json!([valid])
    );
    // A synthetic canary, never a credential/native observation. Keep SQLite's
    // supported schema/JSON constraints enabled; only the body type is corrupt.
    let canary = "STATIC_USAGE_BODY_CANARY_DO_NOT_PROJECT";
    for field in ["input_tokens", "estimated_cost", "scope"] {
        let mut body = serde_json::to_value(&valid).unwrap();
        body[field] = json!(canary);
        assert_eq!(
            raw.execute(
                "UPDATE usage SET body=?1",
                [serde_json::to_string(&body).unwrap()]
            )
            .unwrap(),
            1
        );
        let before = snapshot();
        for view in [
            t.scope(),
            Scope {
                project_id: p.id,
                goal_id: Some(g.id),
                task_id: None,
            },
            Scope {
                project_id: p.id,
                goal_id: None,
                task_id: None,
            },
        ] {
            let error = store.usage(&view).unwrap_err();
            assert_eq!(snapshot(), before, "decode refusal wrote state: {field}");
            let rendered = [
                error.to_string(),
                format!("{error:#}"),
                format!("{error:?}"),
            ];
            // Check actual leakage before the separately required static code:
            // a different outer context alone does not demonstrate a leak.
            for text in &rendered {
                assert!(
                    !text.contains(canary),
                    "usage read leaked body via {field}: {text}"
                );
            }
            for text in &rendered[..2] {
                assert_eq!(
                    text, "invalid persisted usage snapshot",
                    "decode refusal has only its static projection"
                );
            }
            assert_eq!(error.chain().count(), 1, "raw decode cause must not escape");
        }
    }
    raw.execute(
        "UPDATE usage SET body=?1",
        [serde_json::to_string(&valid).unwrap()],
    )
    .unwrap();
    let before = snapshot();
    assert_eq!(
        serde_json::to_value(store.usage(&t.scope()).unwrap()).unwrap(),
        json!([valid])
    );
    assert_eq!(snapshot(), before);
}
