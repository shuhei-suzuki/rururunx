use super::*;
use crate::runtime::PipelineCleanupOutcome;

/// A busy result boundary preserves pending cleanup without failure and retries next sweep.
#[tokio::test]
async fn b6_f7_cleanup_busy_record() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    finish(&f, &runtime, &id, true).await;
    // Adoption does not yield after creating the obligation, so this explicit sweep
    // owns the sweep mutex before the notified supervisor can consume the checkpoint.
    assert!(adopt_ready(&f, &runtime, f.cursor(&id)).await.unwrap());
    let before = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(before.pending_cleanup.len(), 1);
    let obligation = &before.pending_cleanup[0];
    let step = before.steps.last().unwrap();
    let checkout = PipelineTestRunner::checkout(f.config.pipeline.as_ref().unwrap(), step).unwrap();
    let tmp = std::path::PathBuf::from(step.body["intent"]["tmpdir"].as_str().unwrap());
    let (release, arrived) = pause_launch(LaunchPhase::Cleanup);
    let sweep = runtime.sweep_pipeline_test_cleanup();
    tokio::pin!(sweep);
    tokio::select! {
        result = &mut sweep => panic!("sweep missed cleanup checkpoint: {result:?}"),
        result = arrived => result.unwrap(),
    }
    assert!(checkout.is_dir());
    let boundary = runtime.hold_pipeline_test_boundary().await;
    release.send(()).unwrap();
    let outcome = tokio::time::timeout(std::time::Duration::from_secs(10), &mut sweep)
        .await
        .expect("busy cleanup must not wait for control admission")
        .unwrap();
    assert_eq!(outcome, PipelineCleanupOutcome::BoundaryBusy);
    assert!(!checkout.exists(), "removal must reach the result boundary");
    assert!(!tmp.exists());
    let after = f.store().pipeline_goal_view(f.goal).unwrap();
    // The pending projection includes failures, not_before, failure_class and path.
    assert_eq!(after.pending_cleanup, before.pending_cleanup);
    let state: String = f
        .store()
        .connection
        .query_row(
            "SELECT state FROM pipeline_test_cleanup WHERE run_id=?1 AND seq=?2 AND attempt=?3",
            params![id, obligation.seq, obligation.attempt],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(state, "pending");
    assert_eq!(after.steps, before.steps);
    assert_eq!(after.runs, before.runs);
    drop(boundary);
    assert_eq!(
        runtime.sweep_pipeline_test_cleanup().await.unwrap(),
        PipelineCleanupOutcome::Complete
    );
    let (state, failures, not_before, class, path): (String, u32, i64, Option<String>, Option<String>) = f
        .store()
        .connection
        .query_row(
            "SELECT state,failures,not_before,failure_class,path FROM pipeline_test_cleanup WHERE run_id=?1 AND seq=?2 AND attempt=?3",
            params![id, obligation.seq, obligation.attempt],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .unwrap();
    assert_eq!(state, "done");
    assert_eq!(failures, obligation.failures);
    assert_eq!(not_before, 0);
    assert_eq!(class, None);
    assert_eq!(path, None);
    runtime.shutdown().await.unwrap();
}
