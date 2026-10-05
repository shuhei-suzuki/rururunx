//! Selected Git ownership reaches the actual spawned Grok actor, without inference.
use super::fixture_support::Fixture;
use super::*;

#[tokio::test]
async fn actual_pre_spawn_git_unknown_reaches_receipt_and_session_reservation() {
    let fixture = Fixture::new();
    let context = TestGitContext::missing_executable();
    let mut adapter = fixture.adapter();
    adapter.git_context = Some(context.clone());
    let session = fixture.start(&adapter).await.unwrap();
    let mut watch = adapter.subscribe((&session).into()).unwrap();
    let status = tokio::time::timeout(Duration::from_secs(10), async {
        while !watch.borrow().terminal() {
            watch.changed().await.unwrap();
        }
        watch.borrow().clone()
    })
    .await
    .unwrap();
    fixture.observe(&status);
    let receipt = fixture.observation(&status).receipt.unwrap();
    assert_eq!(status.session.state, SessionState::Lost);
    assert!(status.failure.as_ref().unwrap().contains("ProcessFailure"));
    assert_eq!(receipt["owned_process_group_created"], false);
    assert_eq!(receipt["ownership_uncertain"], true);
    assert_eq!(receipt["uncertainty_by_stage"]["pre_spawn"], true);
    assert_eq!(receipt["dispatched"], false);
    assert_eq!(context.held_jobs(), 4);
    let store = fixture.store.lock().unwrap();
    let saved = store.session(session.id).unwrap().unwrap().0;
    assert_eq!(saved.state, SessionState::Lost);
    assert!(crate::git::executor_reserved(&saved));
}

async fn terminal(adapter: &GrokAdapter, fixture: &Fixture, session: &Session) -> SessionStatus {
    let mut watch = adapter.subscribe(session.into()).unwrap();
    let status = tokio::time::timeout(Duration::from_secs(10), async {
        while !watch.borrow().terminal() {
            watch.changed().await.unwrap();
        }
        watch.borrow().clone()
    })
    .await
    .unwrap();
    fixture.observe(&status);
    status
}

#[tokio::test]
async fn actual_late_git_reap_keeps_grok_lost_receipt_and_reservation_after_slot_release() {
    let fixture = Fixture::new();
    let context = TestGitContext::reap_after_cutoff();
    let mut adapter = fixture.adapter();
    adapter.git_context = Some(context.clone());
    let session = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &fixture, &session).await;
    let receipt = fixture.observation(&status).receipt.unwrap();
    assert_eq!(
        status.session.state,
        SessionState::Lost,
        "{:?}",
        status.failure
    );
    assert!(
        status
            .failure
            .as_ref()
            .unwrap()
            .contains("Git child death not confirmed after cleanup")
    );
    assert_eq!(receipt["uncertainty_by_stage"]["pre_spawn"], true);
    assert_eq!(receipt["ownership_uncertain"], true);
    assert_eq!(receipt["owned_process_group_created"], false);
    assert_eq!(receipt["dispatched"], false);
    context.wait_until_released().await;
    let saved = fixture
        .store
        .lock()
        .unwrap()
        .session(session.id)
        .unwrap()
        .unwrap()
        .0;
    assert_eq!(saved.state, SessionState::Lost);
    assert!(crate::git::executor_reserved(&saved));
    assert_eq!(context.held_jobs(), 0);
}

#[tokio::test]
async fn actual_index_git_unknown_reaches_pre_spawn_and_reconciliation_receipts() {
    for (ordinal, stage, dispatched) in [(1, "pre_spawn", false), (2, "reconciliation", true)] {
        let fixture = Fixture::new();
        let context = TestGitContext::missing_matching(&["ls-files", "--stage", "-z"], ordinal);
        let mut adapter = fixture.adapter();
        adapter.git_context = Some(context.clone());
        let session = fixture.start(&adapter).await.unwrap();
        let status = terminal(&adapter, &fixture, &session).await;
        let receipt = fixture.observation(&status).receipt.unwrap();
        assert_eq!(
            status.session.state,
            SessionState::Lost,
            "{:?}",
            status.failure
        );
        assert!(status.failure.as_ref().unwrap().contains("ProcessFailure"));
        assert_eq!(receipt["ownership_uncertain"], true);
        assert_eq!(receipt["uncertainty_by_stage"][stage], true);
        assert_eq!(receipt["dispatched"], dispatched);
        assert_eq!(receipt["owned_process_group_created"], dispatched);
        assert_eq!(context.held_jobs(), 4);
        let saved = fixture
            .store
            .lock()
            .unwrap()
            .session(session.id)
            .unwrap()
            .unwrap()
            .0;
        assert_eq!(saved.state, SessionState::Lost);
        assert!(crate::git::executor_reserved(&saved));
    }
}

#[tokio::test]
async fn actual_binding_refresh_git_unknown_prevents_prompt_and_preserves_reservation() {
    let fixture = Fixture::new();
    let context = TestGitContext::missing_matching(&["check-ref-format"], 2);
    let mut adapter = fixture.adapter();
    adapter.git_context = Some(context.clone());
    let session = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &fixture, &session).await;
    let receipt = fixture.observation(&status).receipt.unwrap();
    assert_eq!(
        status.session.state,
        SessionState::Lost,
        "{:?}",
        status.failure
    );
    assert!(status.failure.as_ref().unwrap().contains("ProcessFailure"));
    assert_eq!(receipt["owned_process_group_created"], true);
    assert_eq!(receipt["ownership_uncertain"], true);
    assert_eq!(receipt["uncertainty_by_stage"]["in_session_binding"], true);
    assert_eq!(receipt["dispatched"], false);
    assert!(status.session.recovery.get("dispatch_intent").is_none());
    assert_eq!(context.held_jobs(), 4);
    assert!(crate::git::executor_reserved(
        &fixture
            .store
            .lock()
            .unwrap()
            .session(session.id)
            .unwrap()
            .unwrap()
            .0
    ));
}

#[tokio::test]
async fn actual_checkpoint_git_unknown_keeps_original_error_and_does_not_publish_input() {
    let fixture = Fixture::new();
    // Successful actor checks binding three times: pre-spawn, in-session and
    // reconciliation. The next exact command belongs to explicit checkpoint.
    let context = TestGitContext::missing_matching(&["check-ref-format"], 4);
    let mut adapter = fixture.adapter();
    adapter.git_context = Some(context.clone());
    let session = fixture.start(&adapter).await.unwrap();
    let status = terminal(&adapter, &fixture, &session).await;
    assert!(
        adapter.transport_succeeded(&status),
        "{:?}; {}",
        status.failure,
        fixture.receipt_message(&status)
    );
    let previous = fixture
        .store
        .lock()
        .unwrap()
        .session(session.id)
        .unwrap()
        .unwrap();
    let mut input = fixture.request.input.clone();
    input.version += 1;
    input.payload = "explicit checkpoint fixture continuation".into();
    let error = adapter
        .checkpoint((&session).into(), input)
        .await
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::ProcessFailure);
    assert_eq!(context.held_jobs(), 4);
    let after = fixture
        .store
        .lock()
        .unwrap()
        .session(session.id)
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::to_value(&previous).unwrap(),
        serde_json::to_value(&after).unwrap()
    );
    let refused = adapter.resume((&session).into()).await.unwrap_err();
    assert_eq!(refused.kind, ErrorKind::InvalidInput);
    // This proves current fresh-input rejection, not a durable checkpoint Unknown
    // fence or the complete recovery contract. The retained pool remains Unknown.
}
