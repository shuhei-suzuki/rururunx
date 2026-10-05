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
