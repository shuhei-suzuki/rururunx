//! Shared terminal persistence is exercised through the installed Codex path.
//! Claude telemetry-specific assertions remain pending M3 in the frozen ledger.
use super::*;
use crate::{
    domain::SessionState,
    execution::{
        Disposition, NativeFailure, WorkOutcome,
        native::{ManagedSessionRef, NativeStatus},
        native_result::AcquisitionStatus,
    },
};

fn status(f: &ControlFixture, handle: &ManagedSessionRef) -> NativeStatus {
    f.runtime
        .installed
        .as_ref()
        .ok()
        .unwrap()
        .registry
        .native_phase_port("worker")
        .unwrap()
        .selected_adapter()
        .unwrap()
        .sessions
        .status(handle)
        .unwrap()
}

/// Claude Executor refusal is checked once for all deferred wire/telemetry IDs.
#[tokio::test]
async fn claude_executor_refusal_has_no_effects() {
    assert_claude_executor_refused(fixture("claude", true)).await;
}

/// Receipt retry and delayed watch publication preserve the original terminal failure.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cl10_terminal_retry_preserves_the_original_failure_in_status_and_watch() {
    let mut f = fixture_mode("codex", true, Some("answer-raw-duplicate"), |_| {});
    f.register_real_git_project();
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    f.runtime.start().await.unwrap();
    success::wait_normal_bound(&f, task).await;
    let unit = success::bound_unit(&f, task);
    let (_, snapshot) = success::workflow(&f, task);
    let handle = snapshot
        .history
        .iter()
        .find_map(|a| a.execution.clone())
        .unwrap();
    let custody = f.runtime.phase_jobs.observed_preparation(unit.id).unwrap();
    let profile = crate::execution::resources::ResourceManager::new(f.owner.clone())
        .profile(&unit)
        .unwrap();
    // Fail only receipt persistence, after the real Core has decoded/rejected
    // the fixture frame. This creates no successful rows or authority.
    let connection = raw(&f);
    connection.execute_batch(&format!(
        "CREATE TRIGGER m0_receipt_fault BEFORE INSERT ON native_results WHEN NEW.unit_id='{}' BEGIN SELECT RAISE(ABORT,'synthetic M0 receipt failure'); END;",
        unit.id
    )).unwrap();
    let release = profile.output.join("fixture-publication-release");
    struct Release(std::path::PathBuf);
    impl Drop for Release {
        fn drop(&mut self) {
            std::fs::write(&self.0, "release").unwrap();
        }
    }
    let release = Release(release);
    std::fs::write(profile.output.join("fixture-delay-publication"), "delay").unwrap();
    success::release_unit(&f, &unit);
    success::wait_for(
        || profile.output.join("fixture-publication-ready").exists(),
        "Core did not reach publication after capture",
        5,
    )
    .await;
    let adapter = f
        .runtime
        .installed
        .as_ref()
        .ok()
        .unwrap()
        .registry
        .native_phase_port("worker")
        .unwrap()
        .selected_adapter()
        .unwrap();
    let mut updates = adapter.sessions.subscribe(&handle).unwrap();
    assert!(status(&f, &handle).receipt.is_none());
    connection
        .execute_batch("DROP TRIGGER m0_receipt_fault")
        .unwrap();
    let recovered = status(&f, &handle);
    assert_eq!(recovered.failure, Some(NativeFailure::ProtocolFailure));
    assert_eq!(
        recovered.diagnostic,
        Some(NativeFailure::ProtocolFailure.diagnostic())
    );
    assert_eq!(recovered.work, Some(WorkOutcome::Unknown));
    assert_eq!(recovered.disposition, Disposition::ProtocolError);
    assert_eq!(recovered.session.state, SessionState::Lost);
    assert!(recovered.metrics.is_none());
    let receipt = f
        .owner
        .store
        .lock()
        .unwrap()
        .native_session_result(handle.session)
        .unwrap()
        .unwrap();
    assert!(receipt.text.is_none() && receipt.answer_sha256.is_none());
    assert_ne!(receipt.acquisition, AcquisitionStatus::Complete);
    assert_eq!(recovered.receipt, Some(receipt.id));
    assert_eq!(
        updates.borrow().failure,
        Some(NativeFailure::ProtocolFailure)
    );
    assert_eq!(updates.borrow().receipt, recovered.receipt);
    assert_eq!(updates.borrow().diagnostic, recovered.diagnostic);
    updates.borrow_and_update();
    drop(release);
    tokio::time::timeout(Duration::from_secs(5), updates.changed())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        updates.borrow().failure,
        Some(NativeFailure::ProtocolFailure)
    );
    assert_eq!(updates.borrow().receipt, recovered.receipt);
    assert_eq!(updates.borrow().diagnostic, recovered.diagnostic);
    assert!(updates.borrow().metrics.is_none());
    drop(updates);
    drop(adapter);
    drop(connection);
    let stopped = f.runtime.shutdown().await;
    assert_eq!(
        stopped
            .as_ref()
            .err()
            .and_then(crate::runtime::stop::pending_site),
        Some(crate::runtime::stop::ShutdownSite::AcceptedOriginals)
    );
    assert!(custody.upgrade().is_some());
    finish(f).await;
    success::wait_for(
        || custody.upgrade().is_none(),
        "retry custody survived Runtime drop",
        10,
    )
    .await;
}
