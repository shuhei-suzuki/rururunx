use super::*;

fn fixture() -> (tempfile::TempDir, Store, Project, Goal, Task, Session) {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(&directory.path().join("state.db")).unwrap();
    let mut project = Project::new(
        "native".into(),
        directory.path().to_path_buf(),
        "fixture".into(),
        "main".into(),
    );
    store.put_project(&mut project).unwrap();
    let mut goal = Goal::new(
        project.id,
        "native".into(),
        vec![CompletionCriterion {
            id: "done".into(),
            description: "fixture".into(),
            satisfied: false,
            evidence: None,
        }],
    );
    store.put_goal(&mut goal).unwrap();
    let mut task = Task::new(project.id, goal.id, "native".into(), "native".into());
    task.worktree = Some(project.worktree_root.join("task"));
    task.branch = Some("feature/task".into());
    store.put_task(&mut task).unwrap();
    let session = Session {
        id: SessionId::new(),
        scope: task.scope(),
        agent: "native".into(),
        provider: "grok".into(),
        role: SessionRole::Executor,
        native_ref: None,
        pid: None,
        worktree: task.worktree.clone().unwrap(),
        state: SessionState::Starting,
        model: None,
        effort: None,
        recovery: json!({}),
        started_at: now_ms(),
    };
    (directory, store, project, goal, task, session)
}
#[test]
fn native_dispatch_checks_each_owner_version_across_independent_connections() {
    for changed in 0..3 {
        let (directory, mut store, mut project, mut goal, mut task, mut session) = fixture();
        let expected = [project.version, goal.version, task.version];
        let version = store
            .put_session_if_current(&session, 0, expected, &[])
            .unwrap();
        let mut other = Store::open(&directory.path().join("state.db")).unwrap();
        match changed {
            0 => {
                project.name = "changed".into();
                other.put_project(&mut project).unwrap();
            }
            1 => {
                goal.objective = "changed".into();
                other.put_goal(&mut goal).unwrap();
            }
            _ => {
                task.title = "changed".into();
                other.put_task(&mut task).unwrap();
            }
        }
        session.state = SessionState::Running;
        session.recovery = json!({"input_version":1,"prompt_id":"must-not-dispatch"});
        let before = store.events(&task.scope(), 0, 100).unwrap().len();
        assert!(matches!(
            store
                .put_session_if_current(&session, version, expected, &[])
                .unwrap_err()
                .downcast_ref::<StateGuardError>(),
            Some(StateGuardError::SnapshotChanged { .. })
        ));
        assert_eq!(
            store.session(session.id).unwrap().unwrap().0.recovery,
            json!({})
        );
        assert_eq!(store.events(&task.scope(), 0, 100).unwrap().len(), before);
    }
}
#[test]
fn native_dispatch_fences_lock_version_aba_and_session_cas() {
    let (directory, mut store, project, goal, task, mut session) = fixture();
    session.role = SessionRole::Reviewer;
    let expected = [project.version, goal.version, task.version];
    let mut lock = Record::new(
        task.scope(),
        RecordKind::WorktreeLock,
        json!({"active":true,"revision":"a".repeat(40),"worktree":task.worktree,"branch":task.branch,"reason":"review"}),
    );
    store.put_record(&mut lock).unwrap();
    let locks = [(lock.id, lock.version)];
    let version = store
        .put_session_if_current(&session, 0, expected, &locks)
        .unwrap();
    let mut other = Store::open(&directory.path().join("state.db")).unwrap();
    lock.data["active"] = json!(false);
    other.put_record(&mut lock).unwrap();
    lock.data["active"] = json!(true);
    other.put_record(&mut lock).unwrap();
    session.state = SessionState::Running;
    session.recovery = json!({"prompt_id":"must-not-dispatch"});
    assert!(
        store
            .put_session_if_current(&session, version, expected, &locks)
            .is_err()
    );
    assert_eq!(
        store.session(session.id).unwrap().unwrap().0.recovery,
        json!({})
    );
    let current = [(lock.id, lock.version)];
    let next = store
        .put_session_if_current(&session, version, expected, &current)
        .unwrap();
    assert!(next > version);
    assert!(
        store
            .put_session_if_current(&session, version, expected, &current)
            .is_err()
    );
}
#[test]
fn native_dispatch_supports_optional_consultation_and_rejects_inactive_owner() {
    let (_directory, mut store, mut project, goal, _task, mut session) = fixture();
    session.role = SessionRole::Consultant;
    session.scope = Scope::project(project.id);
    session.worktree = project.root.clone();
    assert!(
        store
            .put_session_if_current(&session, 0, [project.version, goal.version, 0], &[])
            .is_err()
    );
    let version = store
        .put_session_if_current(&session, 0, [project.version, 0, 0], &[])
        .unwrap();
    project.state = ProjectState::Blocked;
    project.blocked_reason = Some("fixture".into());
    store.put_project(&mut project).unwrap();
    session.state = SessionState::Running;
    assert!(
        store
            .put_session_if_current(&session, version, [project.version, 0, 0], &[])
            .is_err()
    );
    assert_eq!(
        store.session(session.id).unwrap().unwrap().0.state,
        SessionState::Starting
    );
}
