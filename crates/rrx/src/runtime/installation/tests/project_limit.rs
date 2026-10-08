//! S2 controls (HOW §2, §2.1): the fixed limit of 1 distinct active Task per
//! Project at candidate selection, at the final claim, for stored legacy rows
//! and in status. Each control uses the production reader and evaluator.
use super::*;
use crate::{
    runtime::task_driver::{AdmitOutcome, SkipReason},
    state::{CandidateKey, CandidatePage, CapacityScope, DriverCapacityUnavailable},
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
fn driver_for(f: &ControlFixture, task: &str) -> usize {
    raw(f)
        .query_row(
            "SELECT count(*) FROM task_drivers WHERE task_id=?1",
            [task],
            |r| r.get(0),
        )
        .unwrap()
}
fn claim_audits(f: &ControlFixture, task: &str) -> usize {
    raw(f)
        .query_row(
            "SELECT count(*) FROM audit WHERE task_id=?1 AND kind='rrx.private.runtime.driver_claimed'",
            [task],
            |r| r.get(0),
        )
        .unwrap()
}
fn project_row(f: &ControlFixture, project: &str) -> (u64, String) {
    raw(f)
        .query_row(
            "SELECT version,body FROM projects WHERE id=?1",
            [project],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
}
/// A row registered before S2 kept the old default limit of 4. It is written
/// through the production Store write every pre-S2 registration used.
fn store_legacy_limit(f: &ControlFixture, project: &mut crate::domain::Project, stored: usize) {
    project.max_tasks = stored;
    f.owner
        .store()
        .lock()
        .unwrap()
        .put_project(project)
        .unwrap();
}

/// C-S2a: with 2 runnable Tasks in 1 Project, the second is never a candidate
/// while the first is occupied.
#[tokio::test]
async fn c_s2a_candidate_never_lists_second_task_while_first_is_occupied() {
    let f = fixture("claude", true);
    let (_, tasks) = accept(&f, 2).await;
    assert_eq!(
        keys(&f).len(),
        2,
        "SETUP: both Tasks are candidates while idle"
    );
    f.runtime.start().await.unwrap();
    wait_claims(&f, 1).await;
    let listed = keys(&f);
    assert!(
        listed.is_empty(),
        "second Task listed while the first is occupied: {:?}",
        listed.iter().map(|k| &k.task_id).collect::<Vec<_>>()
    );
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(
        count(&f, "task_drivers"),
        1,
        "service claimed a second Task"
    );
    assert_eq!(tasks.len(), 2);
    finish(f).await;
}

/// C-S2a2: B's key is read before A claims; B then reaches the claim
/// transaction through the production evaluator and is refused there, typed,
/// with no Driver row and no claim audit for B.
#[tokio::test]
async fn c_s2a2_final_claim_refuses_second_task_typed() {
    let f = fixture("claude", true);
    accept(&f, 2).await;
    let page = keys(&f);
    assert_eq!(page.len(), 2, "SETUP: both Tasks are candidates while idle");
    // Hold nothing while A claims through the normal service path.
    f.runtime.start().await.unwrap();
    wait_claims(&f, 1).await;
    let claimed: String = raw(&f)
        .query_row("SELECT task_id FROM task_drivers", [], |r| r.get(0))
        .unwrap();
    let b = page
        .iter()
        .find(|k| k.task_id != claimed)
        .expect("SETUP: second key")
        .clone();
    let guard = f.runtime.control_admission.lock().await;
    let error = match f.runtime.evaluate_driver_candidate(&guard, &b) {
        Err(error) => error,
        Ok(AdmitOutcome::Claimed) => panic!("second same-Project Task claimed"),
        Ok(AdmitOutcome::Skipped(_)) => panic!("refused before the claim transaction"),
    };
    assert_eq!(
        error.downcast_ref::<DriverCapacityUnavailable>(),
        Some(&DriverCapacityUnavailable {
            scope: CapacityScope::Project
        }),
        "{error:#}"
    );
    drop(guard);
    assert_eq!(driver_for(&f, &b.task_id), 0, "B has a Driver row");
    assert_eq!(claim_audits(&f, &b.task_id), 0, "B has a claim audit");
    assert_eq!(count(&f, "task_drivers"), 1);
    finish(f).await;
}

/// C-S2a3 and D3: Projects storing a legacy limit are never candidates, never
/// claimed and never rewritten; the supported Project still runs.
#[tokio::test]
async fn c_s2a3_stored_legacy_limits_are_excluded_and_never_rewritten() {
    let mut f = fixture("claude", true);
    let mut legacy = Vec::new();
    for i in 0..3 {
        f.register_real_git_project_named(&format!("legacy-{i}"));
        let mut project = f.project.clone();
        store_legacy_limit(&f, &mut project, 4);
        f.project = project.clone();
        accept(&f, 1).await;
        legacy.push((
            project.id.to_string(),
            project_row(&f, &project.id.to_string()),
        ));
    }
    f.register_real_git_project_named("supported");
    let (_, supported) = accept(&f, 1).await;
    let listed = keys(&f);
    assert_eq!(
        listed.iter().map(|k| k.task_id.clone()).collect::<Vec<_>>(),
        vec![supported[0].id.to_string()],
        "only the supported Project's Task is a candidate"
    );
    f.runtime.start().await.unwrap();
    wait_claims(&f, 1).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(count(&f, "task_drivers"), 1);
    assert_eq!(driver_for(&f, &supported[0].id.to_string()), 1);
    for (id, row) in &legacy {
        assert_eq!(&project_row(&f, id), row, "legacy Project row changed");
    }
    finish(f).await;
}

/// D3 snapshot guard: a key read while the Project stored 1, evaluated after
/// it stores 4, is refused at planning, typed, with nothing written.
#[tokio::test]
async fn d3_plan_guard_refuses_stored_legacy_limit_without_writes() {
    let f = fixture("claude", true);
    accept(&f, 1).await;
    let key = keys(&f).remove(0);
    let mut project = f.project.clone();
    store_legacy_limit(&f, &mut project, 4);
    let row = project_row(&f, &project.id.to_string());
    let audits = count(&f, "audit");
    f.runtime.start().await.unwrap();
    let guard = f.runtime.control_admission.lock().await;
    assert!(matches!(
        f.runtime.evaluate_driver_candidate(&guard, &key).unwrap(),
        AdmitOutcome::Skipped(SkipReason::ProjectLimitUnsupported)
    ));
    drop(guard);
    assert_eq!(count(&f, "task_drivers"), 0);
    assert_eq!(count(&f, "audit"), audits, "planning refusal wrote audit");
    assert_eq!(project_row(&f, &project.id.to_string()), row);
    finish(f).await;
}

async fn status(f: &ControlFixture, goal: GoalId) -> (UnavailableReason, Option<usize>) {
    match f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::GoalStatus {
                view: None,
                project: f.project.id,
                goal,
            }),
        )
        .await
        .unwrap()
    {
        ControlResponse::GoalFacts {
            attention,
            project_limit_stored,
            ..
        }
        | ControlResponse::GoalProposalFacts {
            attention,
            project_limit_stored,
            ..
        } => (attention, project_limit_stored),
        other => panic!("unexpected status {other:?}"),
    }
}

/// D4: an accepted Goal and an inert proposal both report the read-derived
/// `ProjectLimitUnsupported` with the stored value; the answer is the same
/// before and after the service reconciles attention; reading writes nothing;
/// the repair clears it.
#[tokio::test]
async fn d4_status_reports_stored_legacy_limit_read_only_until_repair() {
    let mut f = fixture("claude", true);
    let (accepted, _) = accept(&f, 1).await;
    let proposed = match f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::ProposeGoal {
                project: f.project.id,
                expected_project: f.project.version,
                objective: "inert objective".into(),
            }),
        )
        .await
        .unwrap()
    {
        ControlResponse::GoalProposed { goal, .. } => goal,
        other => panic!("SETUP: proposal refused {other:?}"),
    };
    let mut project = f.project.clone();
    store_legacy_limit(&f, &mut project, 4);
    f.project = project.clone();
    let row = project_row(&f, &project.id.to_string());
    let audits = count(&f, "audit");
    let expected = (UnavailableReason::ProjectLimitUnsupported, Some(4));
    assert_eq!(status(&f, accepted).await, expected);
    assert_eq!(status(&f, proposed).await, expected);
    assert_eq!(count(&f, "audit"), audits, "status read wrote audit");
    assert_eq!(
        project_row(&f, &project.id.to_string()),
        row,
        "read changed the row"
    );
    f.runtime.start().await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(
        status(&f, accepted).await,
        expected,
        "changed after reconcile"
    );
    assert_eq!(
        status(&f, proposed).await,
        expected,
        "changed after reconcile"
    );
    assert_eq!(
        project_row(&f, &project.id.to_string()),
        row,
        "read changed the row"
    );
    assert_eq!(count(&f, "task_drivers"), 0, "legacy Project was driven");
    store_legacy_limit(&f, &mut project, 1);
    f.project = project.clone();
    for goal in [accepted, proposed] {
        let (attention, stored) = status(&f, goal).await;
        assert_ne!(attention, UnavailableReason::ProjectLimitUnsupported);
        assert_eq!(stored, None);
    }
    finish(f).await;
}
