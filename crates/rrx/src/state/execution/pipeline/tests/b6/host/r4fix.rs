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

/// Cleanup unlinks a replaced checkout without touching the outside target.
#[tokio::test]
async fn b6_f4_cleanup_checkout_symlink() {
    let (f, runtime, id, checkout) = checkout_after_exit().await;
    let outside = f._dir.path().join("outside");
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("keep"), b"outside").unwrap();
    std::fs::remove_dir_all(&checkout).unwrap();
    symlink(&outside, &checkout).unwrap();
    assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    assert!(std::fs::symlink_metadata(&checkout).is_err());
    assert_eq!(std::fs::read(outside.join("keep")).unwrap(), b"outside");
    assert!(f.store().pipeline_goal_view(f.goal).unwrap().runs[0].body["test_cleanup"].is_null());
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
    assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    assert!(!checkout.exists());
    assert_eq!(std::fs::read(outside.join("keep")).unwrap(), b"outside");
    runtime.shutdown().await.unwrap();
}

/// A cleanup failure holds durably and operator recovery clears it without changing receipts.
#[tokio::test]
async fn b6_f4_cleanup_failure_recoverable() {
    let (f, runtime, id, checkout) = checkout_after_exit().await;
    let parent = checkout.parent().unwrap();
    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(view.runs[0].state, "held");
    assert_eq!(view.runs[0].body["held"]["reason"], "test_cleanup_failed");
    let receipt = view.steps.last().unwrap().clone();
    assert_eq!(receipt.state, "adopted");
    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).unwrap();
    let request = PipelineRecoveryRequest {
        request_id: Uuid::new_v4(),
        cursor: f.cursor(&id),
        then_action: "abandon_run".into(),
        reason: "retry cleanup".into(),
    };
    f.store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
        .unwrap();
    wait_clean(&f.owner, f.goal, &id, &checkout, None).await;
    assert_eq!(
        f.store()
            .pipeline_goal_view(f.goal)
            .unwrap()
            .steps
            .last()
            .unwrap(),
        &receipt
    );
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
            view.runs.into_iter().find(|r| r.id == id).unwrap().body["test_cleanup"].is_object()
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

/// Terminal cleanup holds require the pending obligation and preserve completed outcomes on recovery.
#[tokio::test]
async fn b6_f4_terminal_cleanup_hold() {
    use super::super::super::round2::{rejected, update};
    let (f, runtime, id) = goal_exited("exit 0;").await;
    let cursor = goal_cursor(&f, &id);
    let version = f.store().goal(f.goal).unwrap().unwrap().version;
    f.store()
        .pipeline_complete_goal(&f.owner, &cursor, version, "test")
        .unwrap();
    for body in [
        "json_remove(body,'$.held','$.cleanup_terminal_state')",
        "json_set(body,'$.held.reason','other','$.cleanup_terminal_state','completed')",
        "json_set(body,'$.held.reason','test_cleanup_failed','$.cleanup_terminal_state','abandoned')",
        "json_remove(json_set(body,'$.held.reason','test_cleanup_failed','$.cleanup_terminal_state','completed'),'$.test_cleanup')",
    ] {
        rejected(
            &f.store().connection,
            &update(
                "pipeline_runs",
                &format!(
                    ",state='held',body=json_set({body},'$.version',version+1,'$.state','held')"
                ),
            ),
            "invalid pipeline run transition",
        );
    }
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let receipt = view.steps.iter().find(|s| s.run_id == id).unwrap().clone();
    let checkout =
        PipelineTestRunner::checkout(f.config.pipeline.as_ref().unwrap(), &receipt).unwrap();
    let parent = checkout.parent().unwrap();
    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        runtime
            .adopt_pipeline_test(goal_cursor(&f, &id))
            .await
            .unwrap()
    );
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let held = view.runs.iter().find(|r| r.id == id).unwrap();
    assert_eq!(held.state, "held");
    assert_eq!(held.body["held"]["reason"], "test_cleanup_failed");
    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).unwrap();
    let request = PipelineRecoveryRequest {
        request_id: Uuid::new_v4(),
        cursor: goal_cursor(&f, &id),
        then_action: "abandon_run".into(),
        reason: "finish terminal cleanup".into(),
    };
    f.store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
        .unwrap();
    wait_clean(&f.owner, f.goal, &id, &checkout, None).await;
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(
        view.runs.iter().find(|r| r.id == id).unwrap().state,
        "completed"
    );
    assert_eq!(
        view.steps.iter().find(|s| s.run_id == id).unwrap(),
        &receipt
    );
    runtime.shutdown().await.unwrap();
}
