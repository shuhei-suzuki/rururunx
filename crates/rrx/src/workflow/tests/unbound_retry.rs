//! Synthetic consumer controls; no native death or managed-input certificate.
use super::*;
use crate::workflow::{Snapshot, UNBOUND_NATIVE_RECOVERY_REQUIRED};

pub(super) fn full_snapshot(fixture: &Fixture) -> Value {
    // The fixture has its own database. Capture all rows/columns, including every
    // context and audit event, rather than the latest version or a page prefix.
    let db = rusqlite::Connection::open(fixture.dir.path().join("state.db")).unwrap();
    let mut tables = serde_json::Map::new();
    for (table, order) in [
        ("projects", "id"),
        ("goals", "id"),
        ("tasks", "id"),
        ("records", "id"),
        ("context_versions", "project_id,owner,version"),
        ("usage", "sequence"),
        ("audit", "sequence"),
    ] {
        let mut statement = db
            .prepare(&format!("SELECT * FROM {table} ORDER BY {order}"))
            .unwrap();
        let columns = statement.column_count();
        let rows: Vec<Vec<Value>> = statement
            .query_map([], |row| {
                (0..columns)
                    .map(|column| {
                        Ok(match row.get_ref(column)? {
                            rusqlite::types::ValueRef::Null => Value::Null,
                            rusqlite::types::ValueRef::Integer(n) => json!(n),
                            rusqlite::types::ValueRef::Real(n) => json!(n),
                            rusqlite::types::ValueRef::Text(s) => {
                                json!(std::str::from_utf8(s).unwrap())
                            }
                            rusqlite::types::ValueRef::Blob(_) => panic!("unexpected fixture blob"),
                        })
                    })
                    .collect()
            })
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        tables.insert(table.into(), json!(rows));
    }
    tables.insert("launches".into(), json!(launch_counts(fixture)));
    Value::Object(tables)
}

fn launch_counts(fixture: &Fixture) -> [usize; 2] {
    [
        fixture.executor.launches.lock().unwrap().len(),
        fixture.reviewer.launches.lock().unwrap().len(),
    ]
}

fn agent(fixture: &Fixture, review: bool) -> &Arc<FakeAgent> {
    if review {
        &fixture.reviewer
    } else {
        &fixture.executor
    }
}

fn assert_unbound(fixture: &Fixture, review: bool, state: AttemptState, marked: bool) -> usize {
    let workflow = fixture.engine.snapshot(fixture.task.id).unwrap();
    let index = workflow.active.unwrap();
    let attempt = &workflow.history[index];
    assert_eq!(
        attempt.phase.actor(),
        if review {
            Actor::Reviewer
        } else {
            Actor::Executor
        }
    );
    assert_eq!(attempt.state, state);
    assert_eq!(attempt.dispatch_started, marked);
    assert!(attempt.session_id.is_none());
    index
}

fn retry_builder(fixture: &Fixture) -> Snapshot {
    let mut next = fixture.engine.read(fixture.task.id).unwrap();
    let index = next.workflow.active.unwrap();
    let attempt = &mut next.workflow.history[index];
    if attempt.state == AttemptState::Running {
        attempt.state = AttemptState::Failed;
    }
    attempt.completed_at.get_or_insert_with(now_ms);
    crate::workflow::remove_attempt_blocker(&mut next.task, attempt);
    next.workflow.retries.push(RetryEvent {
        prior_attempt: index,
        reason: "explicit component retry".into(),
        at: now_ms(),
    });
    next.workflow.active = None;
    next
}

fn generation_builder(fixture: &Fixture) -> (Snapshot, ContextVersion) {
    let mut next = fixture.engine.read(fixture.task.id).unwrap();
    let index = next.workflow.active.unwrap();
    assert_eq!(next.workflow.history[index].state, AttemptState::Running);
    next.workflow.history[index].state = AttemptState::Interrupted;
    next.workflow.history[index].completed_at = Some(now_ms());
    next.workflow.active = None;
    let source = next.workflow.sources.clone();
    crate::workflow::invalidate(&mut next.workflow, &source, "component invalidation").unwrap();
    let phase = crate::workflow::next_phase(&next.workflow).unwrap();
    let context = crate::workflow::make_context(
        &next.task,
        &source,
        phase,
        next.workflow.workflow,
        next.workflow.generation,
        ContextBudget {
            class: BudgetClass::Small,
            discretionary_tokens: 1,
        },
        next.task.context_version + 1,
    );
    crate::workflow::set_context(&mut next, &context);
    (next, context)
}

fn commit(
    fixture: &Fixture,
    next: &mut Snapshot,
    context: Option<&ContextVersion>,
    access: WorkflowAccess,
) -> anyhow::Result<()> {
    next.record.data = serde_json::to_value(&next.workflow).unwrap();
    fixture.store.lock().unwrap().put_workflow_transition(
        &mut next.task,
        &mut next.record,
        context,
        next.project.version,
        next.goal.version,
        access,
    )
}

fn refuse(
    fixture: &Fixture,
    next: &mut Snapshot,
    context: Option<&ContextVersion>,
    access: WorkflowAccess,
) {
    let before = full_snapshot(fixture);
    let task = serde_json::to_value(&next.task).unwrap();
    next.record.data = serde_json::to_value(&next.workflow).unwrap();
    let record = serde_json::to_value(&next.record).unwrap();
    let error = commit(fixture, next, context, access).unwrap_err();
    assert_eq!(error.to_string(), UNBOUND_NATIVE_RECOVERY_REQUIRED);
    assert_eq!(
        full_snapshot(fixture),
        before,
        "refusal must preserve all durable state and launches"
    );
    assert_eq!(serde_json::to_value(&next.task).unwrap(), task);
    assert_eq!(serde_json::to_value(&next.record).unwrap(), record);
}

fn assert_retry_committed(fixture: &Fixture, index: usize, retries: usize, launches: [usize; 2]) {
    let saved = fixture.engine.snapshot(fixture.task.id).unwrap();
    assert!(saved.active.is_none());
    assert_eq!(saved.retries.len(), retries + 1);
    assert_eq!(saved.retries.last().unwrap().prior_attempt, index);
    assert!(saved.history[index].completed_at.is_some());
    assert_eq!(launch_counts(fixture), launches);
}

#[tokio::test]
async fn public_unknown_start_refuses_both_actors_without_replay() {
    for review in [false, true] {
        let fixture = Fixture::ready_agent(review).await;
        let adapter = agent(&fixture, review);
        adapter.start_error.store(true, Ordering::SeqCst);
        let pause = Arc::new(Pause::default());
        *adapter.start_pause.lock().unwrap() = Some(pause.clone());
        let owner = fixture.spawn_step();
        pause.wait().await;
        let index = assert_unbound(&fixture, review, AttemptState::Running, true);
        let records = fixture
            .store
            .lock()
            .unwrap()
            .records(&fixture.task.scope(), RecordKind::Session)
            .unwrap();
        assert_eq!(records.len(), usize::from(review));
        for record in records {
            let session: Session = serde_json::from_value(record.data).unwrap();
            assert_eq!(session.role, SessionRole::Executor);
            assert_eq!(session.state, SessionState::Exited);
        }
        pause.release();
        assert!(matches!(
            owner.await.unwrap().unwrap(),
            StepResult::Failed { .. }
        ));
        assert_eq!(
            assert_unbound(&fixture, review, AttemptState::Failed, true),
            index
        );
        let before = full_snapshot(&fixture);
        assert_eq!(
            fixture
                .engine
                .retry(fixture.task.id, "not outcome proof".into())
                .unwrap_err()
                .to_string(),
            UNBOUND_NATIVE_RECOVERY_REQUIRED
        );
        assert_eq!(full_snapshot(&fixture), before);
        assert!(matches!(
            fixture
                .engine
                .step(fixture.task.id, BTreeMap::new())
                .await
                .unwrap(),
            StepResult::Failed { .. }
        ));
        assert_eq!(full_snapshot(&fixture), before);
        refuse(
            &fixture,
            &mut retry_builder(&fixture),
            None,
            WorkflowAccess::StateOnly,
        );
    }
}

#[tokio::test]
async fn matching_terminal_row_does_not_resolve_unknown_start() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.executor.start_error.store(true, Ordering::SeqCst);
    fixture
        .executor
        .start_error_session
        .store(true, Ordering::SeqCst);
    let pause = Arc::new(Pause::default());
    *fixture.executor.start_pause.lock().unwrap() = Some(pause.clone());
    let owner = fixture.spawn_step();
    pause.wait().await;
    assert_unbound(&fixture, false, AttemptState::Running, true);
    pause.release();
    assert!(matches!(
        owner.await.unwrap().unwrap(),
        StepResult::Failed { .. }
    ));
    assert_unbound(&fixture, false, AttemptState::Failed, true);
    let records = fixture
        .store
        .lock()
        .unwrap()
        .records(&fixture.task.scope(), RecordKind::Session)
        .unwrap();
    assert_eq!(records.len(), 1);
    let session: Session = serde_json::from_value(records[0].data.clone()).unwrap();
    assert_eq!(session.scope, fixture.task.scope());
    assert_eq!(session.agent, "executor");
    assert_eq!(session.role, SessionRole::Executor);
    assert_eq!(session.state, SessionState::Failed);
    let before = full_snapshot(&fixture);
    assert_eq!(
        fixture
            .engine
            .retry(fixture.task.id, "terminal row is not a certificate".into())
            .unwrap_err()
            .to_string(),
        UNBOUND_NATIVE_RECOVERY_REQUIRED
    );
    assert_eq!(full_snapshot(&fixture), before);
    refuse(
        &fixture,
        &mut retry_builder(&fixture),
        None,
        WorkflowAccess::StateOnly,
    );
}

#[tokio::test]
async fn marked_running_raw_closures_refuse_but_original_owner_can_bind_or_fail() {
    for review in [false, true] {
        for fail in [false, true] {
            let fixture = Fixture::ready_agent(review).await;
            let adapter = agent(&fixture, review);
            adapter.start_error.store(fail, Ordering::SeqCst);
            let pause = Arc::new(Pause::default());
            *adapter.start_pause.lock().unwrap() = Some(pause.clone());
            let owner = fixture.spawn_step();
            pause.wait().await;
            let index = assert_unbound(&fixture, review, AttemptState::Running, true);
            refuse(
                &fixture,
                &mut retry_builder(&fixture),
                None,
                WorkflowAccess::StateOnly,
            );
            let (mut next, context) = generation_builder(&fixture);
            refuse(
                &fixture,
                &mut next,
                Some(&context),
                WorkflowAccess::StateOnly,
            );
            if !review {
                for access in [WorkflowAccess::ReadOnly, WorkflowAccess::Mutating] {
                    refuse(&fixture, &mut retry_builder(&fixture), None, access);
                }
            }
            let count = launch_counts(&fixture);
            pause.release();
            let result = owner.await.unwrap().unwrap();
            let saved = fixture.engine.snapshot(fixture.task.id).unwrap();
            assert_eq!(saved.active, Some(index));
            assert_eq!(launch_counts(&fixture), count);
            if fail {
                assert!(matches!(result, StepResult::Failed { .. }));
                assert_eq!(saved.history[index].state, AttemptState::Failed);
                assert!(saved.history[index].session_id.is_none());
            } else {
                assert!(matches!(result, StepResult::Started { .. }));
                assert_eq!(saved.history[index].state, AttemptState::Running);
                assert!(saved.history[index].session_id.is_some());
            }
        }
    }
}

#[tokio::test]
async fn marked_running_terminal_decision_retains_original_attempt() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.executor.start_error.store(true, Ordering::SeqCst);
    let pause = Arc::new(Pause::default());
    *fixture.executor.start_pause.lock().unwrap() = Some(pause.clone());
    let owner = fixture.spawn_step();
    pause.wait().await;
    let index = assert_unbound(&fixture, false, AttemptState::Running, true);
    fixture
        .engine
        .cancel(fixture.task.id, "decision is not native termination".into())
        .unwrap();
    let saved = fixture.engine.snapshot(fixture.task.id).unwrap();
    assert_eq!(saved.active, Some(index));
    assert!(saved.terminal_decision.is_some());
    let before = full_snapshot(&fixture);
    pause.release();
    assert!(owner.await.unwrap().is_err());
    assert_eq!(full_snapshot(&fixture), before);
}

#[tokio::test]
async fn pre_marker_raw_retry_and_generation_commit_and_stale_owners_do_nothing() {
    for review in [false, true] {
        for offset in [3, 4] {
            for generation in [false, true] {
                let fixture = Fixture::ready_agent(review).await;
                let pause = fixture.capture_pause(offset);
                let owner = fixture.spawn_step();
                pause.wait().await;
                let index = assert_unbound(&fixture, review, AttemptState::Running, false);
                let saved = fixture.engine.snapshot(fixture.task.id).unwrap();
                let count = launch_counts(&fixture);
                if generation {
                    let (mut next, context) = generation_builder(&fixture);
                    commit(
                        &fixture,
                        &mut next,
                        Some(&context),
                        WorkflowAccess::StateOnly,
                    )
                    .unwrap();
                    let after = fixture.engine.snapshot(fixture.task.id).unwrap();
                    assert!(after.active.is_none());
                    assert_eq!(after.generation, saved.generation + 1);
                    assert_eq!(after.invalidations.len(), saved.invalidations.len() + 1);
                    assert!(after.completed.is_empty());
                    assert_eq!(after.history[index].state, AttemptState::Interrupted);
                    assert_eq!(after.context_version, context.version);
                } else {
                    commit(
                        &fixture,
                        &mut retry_builder(&fixture),
                        None,
                        WorkflowAccess::StateOnly,
                    )
                    .unwrap();
                    assert_retry_committed(&fixture, index, saved.retries.len(), count);
                }
                let after = full_snapshot(&fixture);
                pause.release();
                assert!(owner.await.unwrap().is_err());
                assert_eq!(
                    full_snapshot(&fixture),
                    after,
                    "stale owner cannot write, audit or launch"
                );
            }
        }
    }
}

async fn definitive_failure(review: bool) -> Fixture {
    let fixture = Fixture::ready_agent(review).await;
    let mut task = fixture
        .store
        .lock()
        .unwrap()
        .task(fixture.task.id)
        .unwrap()
        .unwrap();
    if review {
        task.reviewers = vec!["executor".into()];
    } else {
        task.executor = "unregistered".into();
    }
    fixture.store.lock().unwrap().put_task(&mut task).unwrap();
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Failed { .. }
    ));
    assert_unbound(&fixture, review, AttemptState::Failed, false);
    fixture
}

#[tokio::test]
async fn pre_marker_public_retry_repairs_configuration_and_launches_once() {
    for review in [false, true] {
        let fixture = definitive_failure(review).await;
        let saved = fixture.engine.snapshot(fixture.task.id).unwrap();
        let index = saved.active.unwrap();
        let count = launch_counts(&fixture);
        fixture
            .engine
            .retry(fixture.task.id, "repair configuration".into())
            .unwrap();
        assert_retry_committed(&fixture, index, saved.retries.len(), count);
        let mut task = fixture
            .store
            .lock()
            .unwrap()
            .task(fixture.task.id)
            .unwrap()
            .unwrap();
        if review {
            task.reviewers = vec!["reviewer".into()];
        } else {
            task.executor = "executor".into();
        }
        fixture.store.lock().unwrap().put_task(&mut task).unwrap();
        assert!(matches!(
            fixture
                .engine
                .step(fixture.task.id, BTreeMap::new())
                .await
                .unwrap(),
            StepResult::Started { .. }
        ));
        let mut expected = count;
        expected[usize::from(review)] += 1;
        assert_eq!(launch_counts(&fixture), expected);
    }
}

async fn retry_positive(kind: usize) -> Fixture {
    match kind {
        0 => definitive_failure(false).await,
        1 => {
            let fixture = Fixture::ready_agent(true).await;
            fixture.gates.approved.store(false, Ordering::SeqCst);
            assert!(matches!(
                fixture
                    .engine
                    .step(fixture.task.id, BTreeMap::new())
                    .await
                    .unwrap(),
                StepResult::Started { .. }
            ));
            assert!(matches!(
                fixture
                    .engine
                    .step(fixture.task.id, BTreeMap::new())
                    .await
                    .unwrap(),
                StepResult::Failed { .. }
            ));
            let saved = fixture.engine.snapshot(fixture.task.id).unwrap();
            let attempt = &saved.history[saved.active.unwrap()];
            assert!(attempt.session_id.is_some());
            let (session, _) = fixture
                .store
                .lock()
                .unwrap()
                .session(attempt.session_id.unwrap())
                .unwrap()
                .unwrap();
            assert_eq!(session.state, SessionState::Exited);
            fixture
        }
        2 | 3 => {
            let fixture = Fixture::new(WorkflowClass::Quick);
            fixture
                .engine
                .initialize(fixture.task.id, None)
                .await
                .unwrap();
            fixture.through(Phase::Implement).await;
            fixture.gates.waiting.store(kind == 2, Ordering::SeqCst);
            fixture
                .gates
                .corrupt
                .store(if kind == 3 { 6 } else { 0 }, Ordering::SeqCst);
            let result = fixture
                .engine
                .step(fixture.task.id, BTreeMap::new())
                .await
                .unwrap();
            assert!(matches!(
                result,
                StepResult::Waiting { .. } | StepResult::Failed { .. }
            ));
            let saved = fixture.engine.snapshot(fixture.task.id).unwrap();
            let attempt = &saved.history[saved.active.unwrap()];
            assert_eq!(attempt.phase.actor(), Actor::EvidencePort);
            assert!(!attempt.dispatch_started && attempt.session_id.is_none());
            fixture
        }
        _ => panic!("unknown positive fixture"),
    }
}

#[tokio::test]
async fn identical_raw_retry_builder_commits_on_pre_marker_bound_and_port_outcomes() {
    for kind in 0..4 {
        let fixture = retry_positive(kind).await;
        let saved = fixture.engine.snapshot(fixture.task.id).unwrap();
        let index = saved.active.unwrap();
        let count = launch_counts(&fixture);
        commit(
            &fixture,
            &mut retry_builder(&fixture),
            None,
            WorkflowAccess::StateOnly,
        )
        .unwrap();
        assert_retry_committed(&fixture, index, saved.retries.len(), count);
    }
}

#[tokio::test]
async fn public_retry_keeps_bound_and_port_outcomes_available() {
    for kind in 1..4 {
        let fixture = retry_positive(kind).await;
        let saved = fixture.engine.snapshot(fixture.task.id).unwrap();
        let index = saved.active.unwrap();
        let count = launch_counts(&fixture);
        fixture
            .engine
            .retry(fixture.task.id, "resolved retry".into())
            .unwrap();
        assert_retry_committed(&fixture, index, saved.retries.len(), count);
    }
}
