//! Schema3 factual-history controls. These never construct a ReviewSet/native lease.
use super::*;
use rusqlite::OpenFlags;

fn refused<T: std::fmt::Debug>(result: Result<T>) {
    let error = result.unwrap_err();
    assert!(error.is::<ReviewGatingUnavailable>(), "{error:#}");
}

fn database(f: &Fixture) -> Vec<(String, Vec<Vec<String>>)> {
    let db = rusqlite::Connection::open_with_flags(
        f.dir.path().join("state.db"),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let tables = db
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    tables
        .into_iter()
        .map(|table| {
            let mut query = db
                .prepare(&format!("SELECT * FROM \"{table}\" ORDER BY rowid"))
                .unwrap();
            let width = query.column_count();
            let rows = query
                .query_map([], |row| {
                    (0..width)
                        .map(|i| Ok(format!("{:?}", row.get_ref(i)?)))
                        .collect::<rusqlite::Result<Vec<_>>>()
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap();
            (table, rows)
        })
        .collect()
}

fn counts(f: &Fixture) -> (usize, usize, usize, usize, usize) {
    (
        f.sources.captures.load(Ordering::SeqCst),
        f.gates.calls.lock().unwrap().len(),
        f.executor.launches.lock().unwrap().len(),
        f.reviewer.launches.lock().unwrap().len(),
        f.reviewer.status_calls.load(Ordering::SeqCst),
    )
}

fn context(f: &Fixture, s: &mut Snapshot, phase: Phase) -> ContextVersion {
    let pack = make_context(
        &s.task,
        &s.workflow.sources,
        phase,
        s.workflow.workflow,
        s.workflow.generation,
        budget(s.workflow.workflow, phase, &f.config),
        f.engine.next_context(&s.task.scope()).unwrap(),
    );
    set_context(s, &pack);
    s.task.phase = Some(phase.key().into());
    s.task.state = phase.task_state();
    pack
}

/// Replays the audited OLD grammar only; an Exited row is factual fixture history,
/// not a cleanup certificate. No raw SQL writes or readiness overrides.
fn reserve_history(f: &Fixture) -> usize {
    let mut s = f.engine.read(f.task.id).unwrap();
    let phase = next_phase(&s.workflow).unwrap();
    let pack = context(f, &mut s, phase);
    let index = s.workflow.history.len();
    s.workflow.history.push(PhaseAttempt {
        phase,
        generation: s.workflow.generation,
        context_version: pack.version,
        budget: budget(s.workflow.workflow, phase, &f.config),
        state: AttemptState::Running,
        session_id: None,
        dispatch_started: false,
        observations: vec![],
        claimed_observations: 0,
        agent: match phase.actor() {
            Actor::Reviewer => Some("reviewer".into()),
            Actor::Executor => Some("executor".into()),
            Actor::EvidencePort => None,
        },
        started_at: now_ms(),
        completed_at: None,
        detail: None,
    });
    s.workflow.active = Some(index);
    f.engine.reserve(&mut s, &pack, phase).unwrap();
    if phase.actor() != Actor::EvidencePort {
        s.workflow.history[index].dispatch_started = true;
        f.engine.persist(&mut s, None).unwrap();
        let session = Session {
            id: SessionId::new(),
            scope: s.task.scope(),
            agent: s.workflow.history[index].agent.clone().unwrap(),
            provider: "fake".into(),
            role: if phase.actor() == Actor::Reviewer {
                SessionRole::Reviewer
            } else {
                SessionRole::Executor
            },
            native_ref: None,
            pid: None,
            worktree: s.task.worktree.clone(),
            state: SessionState::Exited,
            model: None,
            effort: None,
            recovery: Value::Null,
            started_at: now_ms(),
        };
        f.store.lock().unwrap().put_session(&session, 0).unwrap();
        s.workflow.history[index].session_id = Some(session.id);
        f.engine.persist(&mut s, None).unwrap();
        let status = SessionStatus {
            session,
            exit_code: Some(0),
            stdout: vec![],
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
            failure: None,
        };
        let agent = if phase.actor() == Actor::Reviewer {
            &f.reviewer
        } else {
            &f.executor
        };
        agent
            .statuses
            .lock()
            .unwrap()
            .insert(status.session.id, status);
    }
    index
}

fn evidence(s: &Snapshot, index: usize) -> Evidence {
    let a = &s.workflow.history[index];
    Evidence {
        scope: s.task.scope(),
        phase: a.phase,
        revision: s.workflow.sources.revision.clone(),
        source_versions: s.workflow.sources.source_versions.clone(),
        dependencies: s.workflow.sources.source_versions.clone(),
        artifacts: vec![format!("historical-fixture://{}", a.phase.key())],
        review_approved: (a.phase.actor() == Actor::Reviewer).then_some(true),
        session_id: a.session_id,
        context_version: a.context_version,
    }
}

fn journal(f: &Fixture, outcome: GateOutcome) {
    let mut s = f.engine.read(f.task.id).unwrap();
    let i = s.workflow.active.unwrap();
    s.workflow.history[i].state = AttemptState::Evaluating;
    s.workflow.history[i].claimed_observations = s.workflow.history[i].observations.len();
    f.engine.persist(&mut s, None).unwrap();
    let sources = authority_only(&s.workflow.sources);
    f.engine
        .observe_gate(
            &mut s,
            i,
            GateObservation {
                sources,
                outcome: Some(outcome),
                error: None,
                at: now_ms(),
            },
        )
        .unwrap();
}

fn complete_history(f: &Fixture) {
    let s = f.engine.read(f.task.id).unwrap();
    let i = s.workflow.active.unwrap();
    let e = evidence(&s, i);
    journal(f, GateOutcome::Passed(e.clone()));
    let mut s = f.engine.read(f.task.id).unwrap();
    let phase = s.workflow.history[i].phase;
    s.workflow.completed.insert(phase, e);
    s.workflow.history[i].state = AttemptState::Succeeded;
    s.workflow.history[i].completed_at = Some(now_ms());
    s.workflow.active = None;
    let next = next_phase(&s.workflow);
    s.workflow.finished = next.is_none();
    let pack = context(f, &mut s, next.unwrap_or(phase));
    s.task.phase = next.map(|p| p.key().into());
    if phase == Phase::Pr {
        s.task.state = TaskState::PrCreated;
    }
    if phase == Phase::Cleanup {
        s.task.state = TaskState::Completed;
    }
    f.engine.persist(&mut s, Some(&pack)).unwrap();
}

async fn history_before(class: WorkflowClass, target: Phase) -> Fixture {
    let f = Fixture::new(class);
    f.engine.initialize(f.task.id, None).await.unwrap();
    while next_phase(&f.engine.snapshot(f.task.id).unwrap()) != Some(target) {
        reserve_history(&f);
        complete_history(&f);
    }
    f
}

#[tokio::test]
async fn formal_entry_refuses_all_reviews_and_downstream_presets_without_effects() {
    for (class, targets) in [
        (
            WorkflowClass::Quick,
            vec![Phase::ImplementationReview, Phase::Pr],
        ),
        (
            WorkflowClass::Standard,
            vec![
                Phase::RequirementsReview,
                Phase::DesignReview,
                Phase::ImplementationReview,
                Phase::Pr,
                Phase::MergeGate,
                Phase::Cleanup,
            ],
        ),
        (
            WorkflowClass::Strict,
            vec![
                Phase::SecurityReview,
                Phase::Pr,
                Phase::MergeGate,
                Phase::Cleanup,
            ],
        ),
    ] {
        for phase in targets {
            let f = history_before(class, phase).await;
            let before = database(&f);
            let calls = counts(&f);
            refused(f.engine.step(f.task.id, BTreeMap::new()).await);
            assert_eq!(database(&f), before, "{class:?}/{phase:?}");
            assert_eq!(counts(&f), calls, "{class:?}/{phase:?}");
        }
    }
}

#[tokio::test]
async fn active_zero_journal_external_gates_refuse_first_effect() {
    for phase in [Phase::Pr, Phase::MergeGate, Phase::Cleanup] {
        let f = history_before(WorkflowClass::Standard, phase).await;
        reserve_history(&f);
        let before = database(&f);
        let calls = counts(&f);
        refused(f.engine.step(f.task.id, BTreeMap::new()).await);
        assert_eq!(database(&f), before);
        assert_eq!(counts(&f), calls);
    }
}

#[tokio::test]
async fn protected_passed_journal_and_successor_refuse_before_capture_and_replay() {
    for (class, phase) in [
        (WorkflowClass::Standard, Phase::RequirementsReview),
        (WorkflowClass::Standard, Phase::DesignReview),
        (WorkflowClass::Quick, Phase::ImplementationReview),
        (WorkflowClass::Strict, Phase::SecurityReview),
        (WorkflowClass::Quick, Phase::Tests),
        (WorkflowClass::Standard, Phase::RequirementsCommit),
        (WorkflowClass::Standard, Phase::DesignCommit),
        (WorkflowClass::Strict, Phase::Mutation),
        (WorkflowClass::Standard, Phase::Pr),
        (WorkflowClass::Standard, Phase::MergeGate),
        (WorkflowClass::Standard, Phase::Cleanup),
    ] {
        let f = history_before(class, phase).await;
        let i = reserve_history(&f);
        let s = f.engine.read(f.task.id).unwrap();
        journal(&f, GateOutcome::Passed(evidence(&s, i)));
        let before = database(&f);
        let calls = counts(&f);
        for _ in 0..2 {
            refused(f.engine.step(f.task.id, BTreeMap::new()).await);
            assert_eq!(database(&f), before, "{phase:?}");
            assert_eq!(counts(&f), calls, "{phase:?}");
        }
    }
}

#[tokio::test]
async fn resume_refuses_prior_waiting_failed_and_bound_review_before_status() {
    for phase in [
        Phase::ImplementationReview,
        Phase::Pr,
        Phase::MergeGate,
        Phase::Cleanup,
    ] {
        for failed in [false, true] {
            if failed && phase.actor() == Actor::Reviewer {
                continue;
            }
            let f = history_before(WorkflowClass::Standard, phase).await;
            reserve_history(&f);
            journal(
                &f,
                if failed {
                    GateOutcome::Failed("historical failed effect".into())
                } else {
                    GateOutcome::Waiting("historical wait".into())
                },
            );
            let result = f.engine.step(f.task.id, BTreeMap::new()).await.unwrap();
            assert!(matches!(
                result,
                StepResult::Waiting { .. } | StepResult::Failed { .. }
            ));
            let before = database(&f);
            let calls = counts(&f);
            refused(f.engine.resume_gate(f.task.id).await);
            assert_eq!(database(&f), before);
            assert_eq!(counts(&f), calls);
            // Ordinary status retention still reports the factual wait/failure.
            assert!(matches!(
                f.engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
                StepResult::Waiting { .. } | StepResult::Failed { .. }
            ));
            assert_eq!(database(&f), before);
        }
    }
}

#[tokio::test]
async fn retry_review_refuses_but_existing_unknown_owner_fence_keeps_precedence() {
    let f = history_before(WorkflowClass::Quick, Phase::ImplementationReview).await;
    reserve_history(&f);
    journal(&f, GateOutcome::Failed("old review failed".into()));
    assert!(matches!(
        f.engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Failed { .. }
    ));
    let before = database(&f);
    let calls = counts(&f);
    refused(f.engine.retry(f.task.id, "fresh review requested".into()));
    assert_eq!(database(&f), before);
    assert_eq!(counts(&f), calls);

    let f = history_before(WorkflowClass::Quick, Phase::ImplementationReview).await;
    let mut s = f.engine.read(f.task.id).unwrap();
    let phase = Phase::ImplementationReview;
    let pack = context(&f, &mut s, phase);
    let i = s.workflow.history.len();
    s.workflow.history.push(PhaseAttempt {
        phase,
        generation: s.workflow.generation,
        context_version: pack.version,
        budget: budget(s.workflow.workflow, phase, &f.config),
        state: AttemptState::Running,
        session_id: None,
        dispatch_started: false,
        observations: vec![],
        claimed_observations: 0,
        agent: Some("reviewer".into()),
        started_at: now_ms(),
        completed_at: None,
        detail: None,
    });
    s.workflow.active = Some(i);
    f.engine.reserve(&mut s, &pack, phase).unwrap();
    s.workflow.history[i].dispatch_started = true;
    f.engine.persist(&mut s, None).unwrap();
    s.workflow.history[i].state = AttemptState::Failed;
    f.engine.persist(&mut s, None).unwrap();
    let before = database(&f);
    let error = f
        .engine
        .retry(f.task.id, "unproven recovery".into())
        .unwrap_err();
    assert!(!error.is::<ReviewGatingUnavailable>());
    assert!(error.to_string().contains(UNBOUND_NATIVE_RECOVERY_REQUIRED));
    assert_eq!(database(&f), before);
}

#[tokio::test]
async fn quick_finished_history_is_readable_but_finalization_cannot_grant() {
    let f = history_before(WorkflowClass::Quick, Phase::Pr).await;
    reserve_history(&f);
    complete_history(&f);
    let before = database(&f);
    let calls = counts(&f);
    assert!(matches!(
        f.engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Finished
    ));
    refused(
        f.engine
            .request_finalization(f.task.id, "explicit finalization".into())
            .await,
    );
    assert_eq!(database(&f), before);
    assert_eq!(counts(&f), calls);
}

#[tokio::test]
async fn central_pack_refuses_risk_only_escalation_and_direct_successor_capture() {
    let f = history_before(WorkflowClass::Quick, Phase::ImplementationReview).await;
    let before = database(&f);
    let calls = counts(&f);
    refused(
        f.engine
            .escalate(
                f.task.id,
                RiskClass::R1,
                None,
                "stronger risk".into(),
                "operator evidence".into(),
            )
            .await,
    );
    assert_eq!(database(&f), before);
    assert_eq!(f.sources.captures.load(Ordering::SeqCst), calls.0 + 1); // old policy inspection retained
    let s = f.engine.read(f.task.id).unwrap();
    let calls = counts(&f);
    refused(
        f.engine
            .prepare_pack(
                &s.project,
                &s.task,
                &s.workflow.sources,
                Phase::ImplementationReview,
                s.workflow.workflow,
                s.workflow.generation,
            )
            .await,
    );
    assert_eq!(database(&f), before);
    assert_eq!(counts(&f), calls);
}

#[tokio::test]
async fn early_refusal_preserves_drift_and_malformed_passed_history() {
    let f = history_before(WorkflowClass::Standard, Phase::Pr).await;
    f.sources.snapshot.lock().unwrap().revision = "changed-head".into();
    let before = database(&f);
    let calls = counts(&f);
    refused(f.engine.step(f.task.id, BTreeMap::new()).await);
    assert_eq!(database(&f), before);
    assert_eq!(counts(&f), calls);
    let i = reserve_history(&f);
    let s = f.engine.read(f.task.id).unwrap();
    let mut e = evidence(&s, i);
    e.revision = "stale-head".into();
    journal(&f, GateOutcome::Passed(e));
    let before = database(&f);
    let calls = counts(&f);
    refused(f.engine.step(f.task.id, BTreeMap::new()).await);
    assert_eq!(database(&f), before);
    assert_eq!(counts(&f), calls);
}

#[tokio::test]
async fn native_owner_status_and_cancel_remain_actual_consumers_without_certifying() {
    let f = history_before(WorkflowClass::Quick, Phase::ImplementationReview).await;
    let i = reserve_history(&f);
    let id = f.engine.snapshot(f.task.id).unwrap().history[i]
        .session_id
        .unwrap();
    // An already-persisted factual Exited owner is observed and then refused.
    let before = database(&f);
    let calls = counts(&f);
    refused(f.engine.step(f.task.id, BTreeMap::new()).await);
    assert_eq!(f.reviewer.status_calls.load(Ordering::SeqCst), calls.4 + 1);
    assert_eq!(database(&f), before);
    // Missing native status still uses the original diagnostic path.
    f.reviewer.statuses.lock().unwrap().remove(&id);
    assert!(matches!(
        f.engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Waiting { .. }
    ));
    assert!(
        f.engine.snapshot(f.task.id).unwrap().history[i]
            .detail
            .as_deref()
            .unwrap()
            .contains("native status unavailable")
    );
    f.engine
        .cancel(f.task.id, "explicit controller cancellation".into())
        .unwrap();
    assert_eq!(
        f.store
            .lock()
            .unwrap()
            .task(f.task.id)
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    // Existing terminal reservation grammar is preserved, not certified cleanup.
    f.engine
        .release_terminal_reservation(
            f.task.id,
            "existing historical terminal reconciliation".into(),
        )
        .unwrap();
    assert!(f.engine.snapshot(f.task.id).unwrap().active.is_none());
}

#[tokio::test]
async fn intermediate_strict_evidence_remains_callable_but_pr_successor_is_held() {
    let f = history_before(WorkflowClass::Strict, Phase::ExpandedRegression).await;
    assert!(matches!(
        f.engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Started {
            phase: Phase::ExpandedRegression,
            session: None
        }
    ));
    assert!(matches!(
        f.engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Completed {
            phase: Phase::ExpandedRegression
        }
    ));
    assert!(matches!(
        f.engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Started {
            phase: Phase::Mutation,
            session: None
        }
    ));
    refused(f.engine.step(f.task.id, BTreeMap::new()).await);
    assert_eq!(
        f.gates
            .calls
            .lock()
            .unwrap()
            .iter()
            .map(|i| i.phase)
            .collect::<Vec<_>>(),
        vec![Phase::ExpandedRegression, Phase::Mutation]
    );
    assert!(
        f.engine
            .snapshot(f.task.id)
            .unwrap()
            .completed
            .contains_key(&Phase::ExpandedRegression)
    );
    assert!(
        !f.engine
            .snapshot(f.task.id)
            .unwrap()
            .completed
            .contains_key(&Phase::Mutation)
    );
    let before = database(&f);
    let calls = counts(&f);
    refused(f.engine.step(f.task.id, BTreeMap::new()).await);
    assert_eq!(database(&f), before);
    assert_eq!(counts(&f), calls);
}
