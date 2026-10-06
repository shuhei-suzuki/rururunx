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
            evaluator: Default::default(),
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
    session.recovery = json!({"prompt_id":"must-not-dispatch","dispatch_intent":{"input_version":1,"prompt_id":"must-not-dispatch"},"private_payload":"must-not-journal"});
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
    let events = store.events(&task.scope(), 0, 100).unwrap();
    let saved = events
        .iter()
        .rev()
        .find(|e| e.kind == "session.saved")
        .unwrap();
    assert_eq!(
        saved.data["evidence"]["dispatch_intent"],
        session.recovery["dispatch_intent"]
    );
    assert!(!saved.data.to_string().contains("must-not-journal"));
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

fn environment_policy() -> EnvironmentAdmission {
    EnvironmentAdmission::new(["GROK_SYNTHETIC_AUTH".into()], ["LANG".into()]).unwrap()
}
fn foreign_environment(store: &mut Store) -> (tempfile::TempDir, Project) {
    let directory = tempfile::tempdir().unwrap();
    let mut project = Project::new(
        "foreign".into(),
        directory.path().to_path_buf(),
        "foreign-fixture".into(),
        "main".into(),
    );
    store.put_project(&mut project).unwrap();
    (directory, project)
}
#[test]
fn environment_admission_rechecks_live_names_without_foreign_version_tokens() {
    let (directory, mut store, mut project, goal, task, session) = fixture();
    project.environment_refs = vec!["LANG".into()];
    store.put_project(&mut project).unwrap();
    let expected = [project.version, goal.version, task.version];
    let policy = environment_policy();
    store
        .check_environment_admission(project.id, &policy)
        .unwrap();
    let (_foreign_dir, mut foreign) = foreign_environment(&mut store);
    let version = store
        .put_session_if_current(&session, 0, expected, &[])
        .unwrap();
    let mut other = Store::open(&directory.path().join("state.db")).unwrap();
    foreign.environment_refs = vec!["GROK_SYNTHETIC_AUTH".into()];
    other.put_project(&mut foreign).unwrap();
    let watermark = store.events(&task.scope(), 0, 100).unwrap().len();
    assert!(matches!(
        store
            .put_session_with_environment_if_current(&session, version, expected, &[], &policy)
            .unwrap_err()
            .downcast_ref::<StateGuardError>(),
        Some(StateGuardError::EnvironmentAuthority)
    ));
    assert_eq!(store.session(session.id).unwrap().unwrap().1, version);
    assert_eq!(
        store.events(&task.scope(), 0, 100).unwrap().len(),
        watermark
    );
    foreign.environment_refs = vec!["FOREIGN_UNRELATED".into()];
    other.put_project(&mut foreign).unwrap();
    let next = store
        .put_session_with_environment_if_current(&session, version, expected, &[], &policy)
        .unwrap();
    assert_eq!(next, version + 1);
}
#[test]
fn environment_admission_preserves_session_and_parent_guard_precedence() {
    let (_directory, mut store, mut project, goal, task, session) = fixture();
    project.environment_refs = vec!["LANG".into()];
    store.put_project(&mut project).unwrap();
    let expected = [project.version, goal.version, task.version];
    let (_foreign_dir, mut foreign) = foreign_environment(&mut store);
    foreign.environment_refs = vec!["GROK_SYNTHETIC_AUTH".into()];
    store.put_project(&mut foreign).unwrap();
    let version = store
        .put_session_if_current(&session, 0, expected, &[])
        .unwrap();
    let policy = environment_policy();
    let error = store
        .put_session_with_environment_if_current(&session, version + 1, expected, &[], &policy)
        .unwrap_err();
    assert!(
        matches!(error.downcast_ref::<StateGuardError>(), Some(StateGuardError::SnapshotChanged { table, .. }) if table=="records")
    );
    let error = store
        .put_session_with_environment_if_current(&session, 0, expected, &[], &policy)
        .unwrap_err();
    assert_eq!(error.to_string(), "snapshot insert failed");
    let mut wrong = session.clone();
    wrong.agent = "rebound".into();
    assert_eq!(
        store
            .put_session_with_environment_if_current(&wrong, version, expected, &[], &policy)
            .unwrap_err()
            .to_string(),
        "session actor/worktree identity is immutable"
    );
    let error = store
        .put_session_with_environment_if_current(
            &session,
            version,
            [project.version, goal.version, task.version + 1],
            &[],
            &policy,
        )
        .unwrap_err();
    assert!(
        matches!(error.downcast_ref::<StateGuardError>(), Some(StateGuardError::SnapshotChanged { table, .. }) if table=="tasks")
    );
}
#[test]
fn environment_projection_streams_all_names_and_ignores_unrelated_foreign_metadata() {
    let (_directory, mut store, mut project, _goal, _task, _session) = fixture();
    project.environment_refs = vec!["LANG".into()];
    store.put_project(&mut project).unwrap();
    let (_foreign_dir, foreign) = foreign_environment(&mut store);
    let policy = environment_policy();
    let mut body = serde_json::to_value(&foreign).unwrap();
    body["root"] = json!({"invalid unrelated metadata":true});
    body["environment_refs"] = json!(vec!["INVALID-NAME"; 3000]);
    store
        .connection
        .execute(
            "UPDATE projects SET body=?1 WHERE id=?2",
            params![body.to_string(), foreign.id.to_string()],
        )
        .unwrap();
    store
        .check_environment_admission(project.id, &policy)
        .unwrap();
    body["environment_refs"]
        .as_array_mut()
        .unwrap()
        .push(json!("GROK_SYNTHETIC_AUTH"));
    store
        .connection
        .execute(
            "UPDATE projects SET body=?1 WHERE id=?2",
            params![body.to_string(), foreign.id.to_string()],
        )
        .unwrap();
    assert!(matches!(
        store
            .check_environment_admission(project.id, &policy)
            .unwrap_err()
            .downcast_ref::<StateGuardError>(),
        Some(StateGuardError::EnvironmentAuthority)
    ));
    store
        .connection
        .execute(
            "UPDATE projects SET body=json_set(body,'$.environment_refs',json(?1)) WHERE id=?2",
            params![
                json!(["LANG", "GROK_SYNTHETIC_AUTH"]).to_string(),
                project.id.to_string()
            ],
        )
        .unwrap();
    store
        .check_environment_admission(project.id, &policy)
        .unwrap();
    for value in [Value::Null, json!({}), json!(["UNRELATED", null])] {
        body["environment_refs"] = value;
        store
            .connection
            .execute(
                "UPDATE projects SET body=?1 WHERE id=?2",
                params![body.to_string(), foreign.id.to_string()],
            )
            .unwrap();
        assert_eq!(
            store
                .check_environment_admission(project.id, &policy)
                .unwrap_err()
                .to_string(),
            "native environment authority unavailable"
        );
    }
    assert_eq!(
        store.environment_candidates(project.id, &policy).unwrap(),
        std::collections::BTreeSet::from(["GROK_SYNTHETIC_AUTH".into()])
    );
}

#[test]
fn environment_name_dto_rejects_bounds_without_dropping_candidates() {
    assert!(EnvironmentAdmission::new((0..512).map(|i| format!("NATIVE_{i}")), []).is_ok());
    assert!(EnvironmentAdmission::new((0..513).map(|i| format!("NATIVE_{i}")), []).is_err());
    assert!(EnvironmentAdmission::new(["A".repeat(256)], []).is_ok());
    assert!(EnvironmentAdmission::new(["A".repeat(257)], []).is_err());
    let wide = |i| format!("N{i:03}_{}", "X".repeat(251));
    assert!(EnvironmentAdmission::new((0..256).map(wide), []).is_ok());
    assert!(EnvironmentAdmission::new((0..257).map(wide), []).is_err());
    assert!(EnvironmentAdmission::new([], (0..128).map(|i| format!("CALLER_{i}"))).is_ok());
    assert!(EnvironmentAdmission::new([], (0..129).map(|i| format!("CALLER_{i}"))).is_err());
}
#[test]
fn environment_policy_keeps_blocked_lock_and_lost_guard_precedence() {
    for guard in 0..3 {
        let (_directory, mut store, mut project, goal, task, mut session) = fixture();
        let (_foreign_dir, mut foreign) = foreign_environment(&mut store);
        foreign.environment_refs = vec!["GROK_SYNTHETIC_AUTH".into()];
        store.put_project(&mut foreign).unwrap();
        let mut locks = vec![];
        if guard == 0 {
            project.state = ProjectState::Blocked;
            project.blocked_reason = Some("synthetic".into());
            store.put_project(&mut project).unwrap();
        } else if guard == 1 {
            let mut lock = Record::new(
                task.scope(),
                RecordKind::WorktreeLock,
                json!({"active":true,"revision":"a".repeat(40),"worktree":task.worktree,"branch":task.branch,"reason":"review"}),
            );
            store.put_record(&mut lock).unwrap();
            locks.push((lock.id, lock.version));
        } else {
            let version = store.put_session(&session, 0).unwrap();
            session.state = SessionState::Lost;
            store.put_session(&session, version).unwrap();
            session.id = SessionId::new();
            session.state = SessionState::Starting;
        }
        let error = store
            .put_session_with_environment_if_current(
                &session,
                0,
                [project.version, goal.version, task.version],
                &locks,
                &environment_policy(),
            )
            .unwrap_err();
        assert!(matches!(
            (guard, error.downcast_ref::<StateGuardError>()),
            (0, Some(StateGuardError::ProjectInactive))
                | (1, Some(StateGuardError::WorktreeLocked))
                | (2, Some(StateGuardError::ExecutorReserved))
        ));
    }
}

#[test]
fn own_environment_projection_rejects_invalid_authority_without_changing_native_precedence() {
    for projection in [
        json!(["LANG", "INVALID-NAME"]),
        json!(["LANG", "LANG"]),
        json!(["LANG", "HOME"]),
        json!(["LANG", "GROK_SYNTHETIC_AUTH", "INVALID-NAME"]),
        json!({}),
        Value::Null,
    ] {
        let (_directory, mut store, mut project, goal, task, session) = fixture();
        project.environment_refs = vec!["LANG".into()];
        store.put_project(&mut project).unwrap();
        let expected = [project.version, goal.version, task.version];
        let version = store
            .put_session_if_current(&session, 0, expected, &[])
            .unwrap();
        store
            .connection
            .execute(
                "UPDATE projects SET body=json_set(body,'$.environment_refs',json(?1)) WHERE id=?2",
                params![projection.to_string(), project.id.to_string()],
            )
            .unwrap();
        let policy = environment_policy();
        let watermark = store.events(&task.scope(), 0, 100).unwrap().len();
        for error in [
            store
                .check_environment_admission(project.id, &policy)
                .unwrap_err(),
            store
                .environment_candidates(project.id, &policy)
                .unwrap_err(),
        ] {
            assert!(matches!(
                error.downcast_ref::<StateGuardError>(),
                Some(StateGuardError::EnvironmentAuthority)
            ));
            assert_eq!(
                error.to_string(),
                "native environment authority unavailable"
            );
        }
        let error = store
            .put_session_with_environment_if_current(&session, version, expected, &[], &policy)
            .unwrap_err();
        if projection.is_array() {
            assert!(matches!(
                error.downcast_ref::<StateGuardError>(),
                Some(StateGuardError::EnvironmentAuthority)
            ));
        } else {
            // Native CAS must keep its earlier full owning-Project decode. A names-only
            // operator/policy error cannot override that existing guard precedence.
            assert!(!matches!(
                error.downcast_ref::<StateGuardError>(),
                Some(StateGuardError::EnvironmentAuthority)
            ));
        }
        assert_eq!(store.session(session.id).unwrap().unwrap().1, version);
        assert_eq!(
            store.events(&task.scope(), 0, 100).unwrap().len(),
            watermark
        );
    }
    let (_directory, mut store, mut project, goal, task, session) = fixture();
    let version = store
        .put_session_if_current(
            &session,
            0,
            [project.version, goal.version, task.version],
            &[],
        )
        .unwrap();
    store
        .connection
        .execute(
            "UPDATE projects SET body=json_remove(body,'$.environment_refs') WHERE id=?1",
            [project.id.to_string()],
        )
        .unwrap();
    let policy = environment_policy();
    assert_eq!(
        store
            .check_environment_admission(project.id, &policy)
            .unwrap_err()
            .to_string(),
        "native environment authority unavailable"
    );
    assert_eq!(
        store
            .environment_candidates(project.id, &policy)
            .unwrap_err()
            .to_string(),
        "native environment authority unavailable"
    );
    let error = store
        .put_session_with_environment_if_current(
            &session,
            version,
            [project.version, goal.version, task.version],
            &[],
            &policy,
        )
        .unwrap_err();
    assert!(!matches!(
        error.downcast_ref::<StateGuardError>(),
        Some(StateGuardError::EnvironmentAuthority)
    ));
    // An explicit own operator check is available while Blocked and does not load
    // malformed unrelated foreign Project metadata or inspect any foreign root.
    project.environment_refs = vec!["GROK_SYNTHETIC_AUTH".into()];
    project.state = ProjectState::Blocked;
    project.blocked_reason = Some("synthetic".into());
    // Restore body before using the normal versioned Project writer.
    store
        .connection
        .execute(
            "UPDATE projects SET body=?1 WHERE id=?2",
            params![
                serde_json::to_string(&Project {
                    environment_refs: vec![],
                    state: ProjectState::Registered,
                    blocked_reason: None,
                    ..project.clone()
                })
                .unwrap(),
                project.id.to_string()
            ],
        )
        .unwrap();
    store.put_project(&mut project).unwrap();
    let (_foreign_directory, foreign) = foreign_environment(&mut store);
    store.connection.execute("UPDATE projects SET body=json_set(body,'$.root',json('null'),'$.environment_refs',json('{}')) WHERE id=?1", [foreign.id.to_string()]).unwrap();
    assert_eq!(
        store.environment_candidates(project.id, &policy).unwrap(),
        std::collections::BTreeSet::from(["GROK_SYNTHETIC_AUTH".into()])
    );
}

#[test]
fn environment_transaction_rejects_invalid_own_names_independently_of_caller_subset() {
    let (_directory, mut store, mut project, goal, task, session) = fixture();
    project.environment_refs = vec!["LANG".into(), "INVALID-NAME".into()];
    store.put_project(&mut project).unwrap();
    let expected = [project.version, goal.version, task.version];
    let version = store
        .put_session_if_current(&session, 0, expected, &[])
        .unwrap();
    let before = store.events(&task.scope(), 0, 100).unwrap().len();
    // Caller LANG remains owned: only own-name validation rejects this authority.
    let error = store
        .put_session_with_environment_if_current(
            &session,
            version,
            expected,
            &[],
            &environment_policy(),
        )
        .unwrap_err();
    assert!(matches!(
        error.downcast_ref::<StateGuardError>(),
        Some(StateGuardError::EnvironmentAuthority)
    ));
    assert_eq!(store.session(session.id).unwrap().unwrap().1, version);
    assert_eq!(store.events(&task.scope(), 0, 100).unwrap().len(), before);
}
