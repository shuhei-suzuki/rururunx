use super::*;

fn last_step(f: &Fixture, id: &str) -> PipelineStep {
    f.store()
        .pipeline_goal_view(f.goal)
        .unwrap()
        .steps
        .into_iter()
        .filter(|s| s.run_id == id)
        .max_by_key(|s| (s.seq, s.attempt))
        .unwrap()
}

/// Removal of an earlier attempt leaves the same run's next launch and adoption admitted.
#[tokio::test]
async fn b6_f6_cleanup_boundary() {
    let (f, runtime, id) = r1fix2::goal_commands(&["first", "second"]).await;
    r1fix2::run_goal_test(&f, &runtime, &id, false).await;
    let first = last_step(&f, &id);
    let checkout =
        PipelineTestRunner::checkout(f.config.pipeline.as_ref().unwrap(), &first).unwrap();
    let (release, arrived) = pause_launch(LaunchPhase::Cleanup);
    assert!(
        adopt_ready(&f, &runtime, goal_cursor(&f, &id))
            .await
            .unwrap()
    );
    tokio::time::timeout(std::time::Duration::from_secs(10), arrived)
        .await
        .unwrap()
        .unwrap();
    assert!(
        checkout.is_dir(),
        "checkpoint must precede checkout removal"
    );
    r1fix2::plan_goal(&f, &id, 2, 1, "second");
    let result = runtime
        .launch_pipeline_test(goal_cursor(&f, &id))
        .await
        .unwrap();
    let LaunchResult::Started(custody) = result else {
        release.send(()).unwrap();
        panic!("cleanup held the launch boundary");
    };
    let second = last_step(&f, &id);
    let second_checkout =
        PipelineTestRunner::checkout(f.config.pipeline.as_ref().unwrap(), &second).unwrap();
    std::fs::write(second_checkout.join("go"), b"go").unwrap();
    custody.wait().await.unwrap();
    // Deliberately call the product entry point once: a retry would mask boundary retention.
    let adopted = runtime
        .adopt_pipeline_test(goal_cursor(&f, &id))
        .await
        .unwrap();
    release.send(()).unwrap();
    assert!(adopted, "cleanup held the adoption boundary");
    r4fix::wait_clean(&f.owner, f.goal, &id, &checkout, None).await;
    runtime.shutdown().await.unwrap();
}

/// An epoch takeover during removal refuses the old sweep's result publication.
#[tokio::test]
async fn b6_f6_cleanup_takeover() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    finish(&f, &runtime, &id, true).await;
    let (release, arrived) = pause_launch(LaunchPhase::Cleanup);
    assert!(adopt_ready(&f, &runtime, f.cursor(&id)).await.unwrap());
    tokio::time::timeout(std::time::Duration::from_secs(10), arrived)
        .await
        .unwrap()
        .unwrap();
    let before = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(
        before.pending_cleanup.len(),
        1,
        "removal must still be pending"
    );
    let step = before.steps.last().unwrap();
    let checkout = PipelineTestRunner::checkout(f.config.pipeline.as_ref().unwrap(), step).unwrap();
    assert!(checkout.is_dir());
    // Use the real epoch invalidation transaction, never a success/authority seed.
    let (_, epoch) = f.store().begin_execution_epoch().unwrap();
    assert!(epoch > f.owner.epoch());
    release.send(()).unwrap();
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    while runtime.pipeline_test_service_running() {
        assert!(
            tokio::time::Instant::now() < deadline,
            "retired sweep did not finish"
        );
        tokio::task::yield_now().await;
    }
    assert!(
        !checkout.exists(),
        "removal must finish before refusing the stale result"
    );
    let after = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(after.pending_cleanup, before.pending_cleanup);
    assert_eq!(after.steps, before.steps);
    assert_eq!(after.runs, before.runs);
    assert!(
        runtime.shutdown().await.is_err(),
        "retired qualification must stop the service"
    );
}
