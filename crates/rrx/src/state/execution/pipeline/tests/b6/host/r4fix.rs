use super::*;
use std::os::unix::fs::{PermissionsExt, symlink};

async fn checkout_after_exit() -> (Fixture, Arc<Runtime>, String, std::path::PathBuf) {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    finish(&f, &runtime, &id, true).await;
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let checkout = PipelineTestRunner::checkout(
        f.config.pipeline.as_ref().unwrap(),
        view.steps.last().unwrap(),
    )
    .unwrap();
    (f, runtime, id, checkout)
}

/// Cleanup preserves a checkout under a nonprivate parent and records maintenance without changing the run.
#[tokio::test]
async fn b6_cleanup_parent_private() {
    let (f, runtime, id, checkout) = checkout_after_exit().await;
    std::fs::write(checkout.join("keep"), b"keep").unwrap();
    let parent = checkout.parent().unwrap();
    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(adopt_ready(&f, &runtime, f.cursor(&id)).await.unwrap());
    let before = f.store().pipeline_goal_view(f.goal).unwrap();
    runtime.sweep_pipeline_test_cleanup().await.unwrap();
    assert_eq!(std::fs::read(checkout.join("keep")).unwrap(), b"keep");
    let after = f.store().pipeline_goal_view(f.goal).unwrap();
    let pending = after
        .pending_cleanup
        .iter()
        .find(|c| c.run_id == id)
        .expect("refused cleanup must remain pending");
    assert_eq!(pending.failure_class.as_deref(), Some("checkout_remove"));
    assert_eq!(pending.path.as_deref(), checkout.to_str());
    assert_eq!(after.runs, before.runs);
    assert_eq!(after.steps, before.steps);
    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).unwrap();
    wait_clean(&f.owner, f.goal, &id, &checkout, None).await;
    runtime.shutdown().await.unwrap();
}

/// Cleanup unlinks a replaced checkout without touching the outside target.
#[tokio::test]
async fn b6_f4_cleanup_checkout_symlink() {
    let (f, runtime, id, checkout) = checkout_after_exit().await;
    let outside = f._dir.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("keep"), b"outside").unwrap();
    std::fs::remove_dir_all(&checkout).unwrap();
    symlink(&outside, &checkout).unwrap();
    assert!(adopt_ready(&f, &runtime, f.cursor(&id)).await.unwrap());
    wait_clean(&f.owner, f.goal, &id, &checkout, None).await;
    assert!(std::fs::symlink_metadata(&checkout).is_err());
    assert_eq!(std::fs::read(outside.join("keep")).unwrap(), b"outside");
    assert!(
        f.store()
            .pipeline_goal_view(f.goal)
            .unwrap()
            .pending_cleanup
            .is_empty()
    );
    runtime.shutdown().await.unwrap();
}

/// Recursive checkout cleanup does not traverse a symlink inside the checkout.
#[tokio::test]
async fn b6_f4_cleanup_inner_symlink() {
    let (f, runtime, id, checkout) = checkout_after_exit().await;
    let outside = f._dir.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("keep"), b"outside").unwrap();
    symlink(&outside, checkout.join("link")).unwrap();
    assert!(adopt_ready(&f, &runtime, f.cursor(&id)).await.unwrap());
    wait_clean(&f.owner, f.goal, &id, &checkout, None).await;
    assert!(!checkout.exists());
    assert_eq!(std::fs::read(outside.join("keep")).unwrap(), b"outside");
    runtime.shutdown().await.unwrap();
}

pub(super) async fn wait_clean(
    owner: &RuntimeOwner,
    goal: GoalId,
    id: &str,
    checkout: &std::path::Path,
    tmp: Option<&std::path::Path>,
) {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let pending = {
            let view = owner
                .store
                .lock()
                .unwrap()
                .pipeline_goal_view(goal)
                .unwrap();
            view.pending_cleanup.iter().any(|c| c.run_id == id)
        };
        if !pending {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "cleanup obligation did not finish"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(!checkout.exists());
    if let Some(tmp) = tmp {
        assert!(!tmp.exists());
    }
}

/// A prelaunch-held test's prepared checkout is reclaimed without any later adoption.
#[tokio::test]
async fn b6_f4_prelaunch_abandon_cleanup() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    let cursor = f.cursor(&id);
    let profile = f
        .store()
        .pipeline_test_context(&f.owner, &cursor)
        .unwrap()
        .unwrap();
    let (run, step) = f
        .store()
        .pipeline_launch_snapshot(&f.owner, &cursor)
        .unwrap();
    let prepared = PipelineTestRunner::prepare(
        &f.owner,
        f.config.pipeline.as_ref().unwrap(),
        &run,
        &step,
        profile,
    )
    .await
    .unwrap();
    f.store()
        .pipeline_hold(&f.owner, &cursor, "test_checkout_failed", None)
        .unwrap();
    let request = PipelineRecoveryRequest {
        request_id: Uuid::new_v4(),
        cursor: f.cursor(&id),
        then_action: "abandon_run".into(),
        reason: "abandon prelaunch hold".into(),
    };
    f.store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
        .unwrap();
    wait_clean(&f.owner, f.goal, &id, &prepared.checkout, None).await;
    runtime.shutdown().await.unwrap();
}
