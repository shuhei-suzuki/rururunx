//! Synthetic consumer controls; no native death or managed-input certificate.
//! FM §8.3 F2 (R): every scenario here that reaches a Native start (unknown
//! start, marked/unmarked running, a reviewer phase) is now refused by the
//! legacy Engine's preflight before source capture; those positives are
//! retired, their managed equivalent is SC-N continuation work (D3/D4). The
//! reviewer variants cannot reach their phase at all (Implement is refused
//! first). The former pre-dispatch definitive failure (unregistered executor)
//! is now refused with `AdapterUnavailable` before any attempt is recorded.
use super::*;

fn launch_counts(fixture: &Fixture) -> [usize; 2] {
    [
        fixture.executor.launches.lock().unwrap().len(),
        fixture.reviewer.launches.lock().unwrap().len(),
    ]
}

/// R: the executor's unknown start (start error, held start) is never
/// reached; the step is refused and the adapter is never called.
#[tokio::test]
async fn public_unknown_start_refuses_both_actors_without_replay() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.executor.start_error.store(true, Ordering::SeqCst);
    let pause = Arc::new(Pause::default());
    *fixture.executor.start_pause.lock().unwrap() = Some(pause.clone());
    fixture.f2_refused().await;
    pause.assert_never_entered().await;
    assert_eq!(launch_counts(&fixture), [0, 0]);
}

/// R: as above with a matching terminal Session row armed.
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
    fixture.f2_refused().await;
    pause.assert_never_entered().await;
    assert_eq!(launch_counts(&fixture), [0, 0]);
}

/// R: a marked-running start (failing or not) is never reached.
#[tokio::test]
async fn marked_running_raw_closures_refuse_but_original_owner_can_bind_or_fail() {
    for fail in [false, true] {
        let fixture = Fixture::ready_agent(false).await;
        fixture.executor.start_error.store(fail, Ordering::SeqCst);
        let pause = Arc::new(Pause::default());
        *fixture.executor.start_pause.lock().unwrap() = Some(pause.clone());
        fixture.f2_refused().await;
        pause.assert_never_entered().await;
        assert_eq!(launch_counts(&fixture), [0, 0]);
    }
}

/// R: no running attempt exists for a terminal decision to retain.
#[tokio::test]
async fn marked_running_terminal_decision_retains_original_attempt() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.executor.start_error.store(true, Ordering::SeqCst);
    let pause = Arc::new(Pause::default());
    *fixture.executor.start_pause.lock().unwrap() = Some(pause.clone());
    fixture.f2_refused().await;
    assert!(
        fixture
            .engine
            .snapshot(fixture.task.id)
            .unwrap()
            .active
            .is_none()
    );
}

/// R: the pre-marker capture pause is never reached.
#[tokio::test]
async fn pre_marker_raw_retry_and_generation_commit_and_stale_owners_do_nothing() {
    let fixture = Fixture::ready_agent(false).await;
    let captures = fixture.sources.captures.load(Ordering::SeqCst);
    let pause = fixture.capture_pause(3);
    fixture.f2_refused().await;
    pause.assert_never_entered().await;
    assert_eq!(fixture.sources.captures.load(Ordering::SeqCst), captures);
}

/// R: every former kind (bound review, waiting or failed EvidencePort) needs
/// a completed Native Implement, which is now refused.
#[tokio::test]
async fn public_retry_keeps_bound_and_port_outcomes_available() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}

/// R (§8.3, the refusal selected first): with an unregistered executor the
/// legacy step is refused with `AdapterUnavailable` before any attempt is
/// recorded, so the former definitive pre-dispatch failure and its public
/// retry/repair (and launch once) are not reached; retired with the legacy
/// Engine (S1).
#[tokio::test]
async fn pre_marker_public_retry_repairs_configuration_and_launches_once() {
    let fixture = Fixture::ready_agent(false).await;
    let mut task = fixture
        .store
        .lock()
        .unwrap()
        .task(fixture.task.id)
        .unwrap()
        .unwrap();
    task.executor = "unregistered".into();
    fixture.store.lock().unwrap().put_task(&mut task).unwrap();
    fixture
        .refused_as(NativePreflightRefusal::AdapterUnavailable)
        .await;
    assert_eq!(launch_counts(&fixture), [0, 0]);
}

/// R: the former outcomes (pre-marker definitive failure, bound review,
/// waiting or failed EvidencePort) are not reached: the unregistered
/// executor is refused with `AdapterUnavailable`, and the others need a
/// completed Native Implement, refused by F2.
#[tokio::test]
async fn identical_raw_retry_builder_commits_on_pre_marker_bound_and_port_outcomes() {
    let fixture = Fixture::ready_agent(false).await;
    let mut task = fixture
        .store
        .lock()
        .unwrap()
        .task(fixture.task.id)
        .unwrap()
        .unwrap();
    task.executor = "unregistered".into();
    fixture.store.lock().unwrap().put_task(&mut task).unwrap();
    fixture
        .refused_as(NativePreflightRefusal::AdapterUnavailable)
        .await;
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}
