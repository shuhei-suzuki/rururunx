//! Controls call the same production readers/evaluation and observe actual
//! service-loop claims. Raw SQL below is corruption only; it issues no permits.
use super::*;
use crate::{
    runtime::task_driver::{AdmitOutcome, SkipReason},
    state::{CandidateKey, CandidatePage},
};

fn keys(f: &ControlFixture) -> Vec<CandidateKey> {
    match f
        .owner
        .store
        .lock()
        .unwrap()
        .ready_driver_candidates(
            f.owner.instance_id(),
            f.owner.epoch(),
            None,
            f.runtime.config.scheduler.global_max_sessions,
        )
        .unwrap()
    {
        CandidatePage::Rows { keys, .. } => keys,
        CandidatePage::GlobalFull => panic!("unexpected global saturation"),
    }
}
async fn wait_claims(f: &ControlFixture, expected: usize) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while count(f, "task_drivers") != expected {
        assert!(
            tokio::time::Instant::now() < deadline,
            "actual service claim count did not reach {expected}"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
fn body(f: &ControlFixture, key: &CandidateKey) -> String {
    raw(f)
        .query_row("SELECT body FROM tasks WHERE id=?1", [&key.task_id], |r| {
            r.get(0)
        })
        .unwrap()
}
fn replace_body(f: &ControlFixture, key: &CandidateKey, body: rusqlite::types::Value) {
    assert_eq!(
        raw(f)
            .execute(
                "UPDATE tasks SET body=?1 WHERE id=?2",
                rusqlite::params![body, key.task_id]
            )
            .expect("SETUP: schema refused raw corruption"),
        1
    );
}
fn bounded_body(mut body: String, len: usize) -> String {
    assert!(body.len() < len);
    body.extend(std::iter::repeat_n(' ', len - body.len()));
    body
}

#[tokio::test]
async fn c13_exact_bound_reaches_actual_service_claim() {
    let f = fixture("claude", true);
    accept(&f, 1).await;
    let key = keys(&f).remove(0);
    replace_body(&f, &key, bounded_body(body(&f, &key), 1048576).into());
    let read = f
        .owner
        .store
        .lock()
        .unwrap()
        .current_task_bounded(&key)
        .unwrap();
    assert_eq!(read.id.to_string(), key.task_id);
    f.runtime.start().await.unwrap();
    wait_claims(&f, 1).await;
    assert_eq!(
        count(&f, "task_drivers"),
        1,
        "exact bound did not reach admission"
    );
    finish(f).await;
}

#[tokio::test]
async fn c13_early_bound_identity_and_decode_refusals_preserve_sibling_progress() {
    for case in ["over", "blob", "decode", "id", "version"] {
        let f = fixture("claude", true);
        accept(&f, 2).await;
        let page = keys(&f);
        let bad = &page[0];
        let sibling = &page[1];
        let before = body(&f, bad);
        let corrupt = match case {
            "over" => bounded_body(before.clone(), 1048577).into(),
            "blob" => rusqlite::types::Value::Blob(before.as_bytes().to_vec()),
            "decode" => "{}".to_owned().into(),
            "id" | "version" => {
                let mut json: serde_json::Value = serde_json::from_str(&before).unwrap();
                if case == "id" {
                    json["id"] = uuid::Uuid::new_v4().to_string().into();
                } else {
                    json["version"] = 999.into();
                }
                serde_json::to_string(&json).unwrap().into()
            }
            _ => unreachable!(),
        };
        replace_body(&f, bad, corrupt);
        f.runtime.start().await.unwrap();
        let guard = f.runtime.control_admission.lock().await;
        let writes = count(&f, "audit");
        let attention: Option<String> = raw(&f)
            .query_row(
                "SELECT attention FROM scheduler_tasks WHERE task_id=?1",
                [&bad.task_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            matches!(
                f.runtime.evaluate_driver_candidate(&guard, bad).unwrap(),
                AdmitOutcome::Skipped(SkipReason::CurrentTaskBounded)
            ),
            "{case}: refusal moved beyond current_task_bounded"
        );
        assert_eq!(count(&f, "audit"), writes, "bounded refusal wrote facts");
        assert_eq!(
            raw(&f)
                .query_row(
                    "SELECT attention FROM scheduler_tasks WHERE task_id=?1",
                    [&bad.task_id],
                    |r| r.get::<_, Option<String>>(0)
                )
                .unwrap(),
            attention
        );
        assert_eq!(count(&f, "task_drivers"), 0);
        drop(guard);
        wait_claims(&f, 1).await;
        let claimed: String = raw(&f)
            .query_row("SELECT task_id FROM task_drivers", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            claimed, sibling.task_id,
            "{case}: refused head stopped actual sweep"
        );
        finish(f).await;
    }
}

#[tokio::test]
async fn c13_original_composition_bounded_reread_refuses_before_validate_for() {
    let f = fixture("claude", true);
    accept(&f, 1).await;
    let key = keys(&f).remove(0);
    f.runtime.start().await.unwrap();
    let guard = f.runtime.control_admission.lock().await;
    let original = f
        .owner
        .store
        .lock()
        .unwrap()
        .current_task_bounded(&key)
        .unwrap();
    let composition = f.runtime.installed_driver_composition(&original).unwrap();
    let mut changed: serde_json::Value = serde_json::from_str(&body(&f, &key)).unwrap();
    changed["title"] = "reread changed body".into();
    replace_body(
        &f,
        &key,
        bounded_body(serde_json::to_string(&changed).unwrap(), 1048577).into(),
    );
    let writes = count(&f, "audit");
    assert!(
        matches!(
            f.runtime
                .admit_task_driver(&guard, &key, original.id, composition)
                .unwrap(),
            AdmitOutcome::Skipped(SkipReason::BoundedReread)
        ),
        "refusal moved beyond bounded reread"
    );
    assert_eq!(count(&f, "audit"), writes);
    assert_eq!(count(&f, "task_drivers"), 0);
    drop(guard);
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}

#[tokio::test]
async fn c9_busy_admission_sweep_is_synchronous_and_shutdown_stops_claims() {
    let f = fixture("claude", true);
    accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    let guard = f.runtime.control_admission.lock().await;
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(5), async {
            f.runtime.admit_ready_tasks().unwrap()
        })
        .await
        .expect("busy sweep awaited admission"),
        0
    );
    let runtime = f.runtime.clone();
    let shutdown = tokio::spawn(async move { runtime.shutdown().await });
    // Shutdown waits for the actual already-held control operation to finish.
    // Stopping is set under that admission, not by a speculative observer.
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(!shutdown.is_finished());
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(5), async {
            f.runtime.admit_ready_tasks().unwrap()
        })
        .await
        .expect("busy sweep awaited admission"),
        0
    );
    assert_eq!(count(&f, "task_drivers"), 0);
    drop(guard);
    tokio::time::timeout(Duration::from_secs(5), shutdown)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(count(&f, "task_drivers"), 0);
    finish(f).await;
}

#[tokio::test]
async fn c15_actual_service_pass_keeps_cursor_after_32_paused_tasks() {
    let f = fixture("claude", true);
    let (paused, _) = accept(&f, 40).await;
    let version = f
        .owner
        .store
        .lock()
        .unwrap()
        .goal(paused)
        .unwrap()
        .unwrap()
        .version;
    let reply = f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::SetGoalLifecycle {
                project: f.project.id,
                goal: paused,
                expected_goal: version,
                target: GoalControl::Pause,
                reason: "pass control".into(),
            }),
        )
        .await
        .unwrap();
    assert!(!matches!(reply, ControlResponse::Rejected { .. }));
    let (ready, ready_tasks) = accept(&f, 1).await;
    // Make the paused Goal rank first using its ordinary nongrant rotation.
    raw(&f)
        .execute(
            "UPDATE scheduler_goals SET rotation=1 WHERE goal_id=?1",
            [ready.to_string()],
        )
        .unwrap();
    let page = keys(&f);
    assert_eq!(page.len(), 41);
    assert_eq!(page[0].goal_id, paused.to_string());
    // Accepted Goals left one coalesced wake hint. Consume it before start so
    // observation follows one actual service pass rather than two immediate
    // passes. A wake is nongrant; no admission/actor state is changed here.
    tokio::time::timeout(Duration::from_millis(20), f.runtime.wake.notified())
        .await
        .unwrap();
    f.runtime.start().await.unwrap();
    tokio::task::yield_now().await;
    assert_eq!(
        count(&f, "task_drivers"),
        0,
        "first sweep exceeded 32 evaluations"
    );
    let cursor = f.runtime.admission_cursor.lock().unwrap().clone();
    assert!(
        cursor.is_some(),
        "32-evaluation cursor was not stored between sweeps"
    );
    let cursor = cursor.unwrap();
    assert_eq!(
        cursor.task_id, page[31].task_id,
        "first finite pass did not retain exact cursor"
    );
    f.runtime.wake.notify_one();
    wait_claims(&f, 1).await;
    let claimed: String = raw(&f)
        .query_row("SELECT task_id FROM task_drivers", [], |r| r.get(0))
        .unwrap();
    assert_eq!(claimed, ready_tasks[0].id.to_string());
    assert!(
        f.runtime.admission_cursor.lock().unwrap().is_none(),
        "later empty read failed to reset cursor"
    );
    finish(f).await;
}
