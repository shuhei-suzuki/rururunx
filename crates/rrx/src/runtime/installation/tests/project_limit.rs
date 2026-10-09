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
    // L2 (Sol 6071558338, 6072320376): more legacy-limit Tasks than the
    // eligible LIMIT (65) sort before the supported Task, so excluding them
    // after the LIMIT would leave the supported Task off the first page. Two
    // genuinely registered Projects are assigned by their observed ID order:
    // the smaller holds the legacy Tasks, the larger is supported.
    const LEGACY_TASKS: usize = 70;
    f.register_real_git_project_named("ordered-a");
    let first = f.project.clone();
    f.register_real_git_project_named("ordered-b");
    let second = f.project.clone();
    let (mut large_legacy, supported_project) = if first.id.to_string() < second.id.to_string() {
        (first, second)
    } else {
        (second, first)
    };
    store_legacy_limit(&f, &mut large_legacy, 4);
    f.project = large_legacy.clone();
    accept(&f, LEGACY_TASKS).await;
    let mut legacy = vec![(
        large_legacy.id.to_string(),
        project_row(&f, &large_legacy.id.to_string()),
    )];
    for i in 0..2 {
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
    let supported_id = supported_project.id;
    f.project = supported_project;
    let (_, supported) = accept(&f, 1).await;
    let rotations: Vec<(String, u64)> = {
        let c = raw(&f);
        let mut q = c
            .prepare(
                "SELECT project_id,rotation FROM scheduler_projects ORDER BY rotation,project_id",
            )
            .unwrap();
        q.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let position = |id: String| rotations.iter().position(|(p, _)| *p == id).unwrap();
    assert!(
        position(large_legacy.id.to_string()) < position(supported_id.to_string()),
        "SETUP: the large legacy Project does not sort first: {rotations:?}"
    );
    let listed = keys(&f);
    assert_eq!(
        listed.iter().map(|k| k.task_id.clone()).collect::<Vec<_>>(),
        vec![supported[0].id.to_string()],
        "only the supported Project's Task is a candidate, on the first page"
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

/// Every row, version and audit entry a Goal status read could touch.
fn status_rows(f: &ControlFixture) -> Vec<String> {
    let c = raw(f);
    let mut out = Vec::new();
    for table in [
        "projects",
        "goals",
        "tasks",
        "records",
        "scheduler_projects",
        "scheduler_goals",
        "scheduler_tasks",
        "task_drivers",
        "execution_units",
        "audit",
    ] {
        let mut q = c
            .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
            .unwrap();
        let columns = q.column_count();
        let rows = q
            .query_map([], |r| {
                (0..columns)
                    .map(|i| r.get::<_, rusqlite::types::Value>(i))
                    .collect::<rusqlite::Result<Vec<_>>>()
            })
            .unwrap()
            .map(|r| format!("{table}: {:?}", r.unwrap()))
            .collect::<Vec<_>>();
        out.extend(rows);
    }
    out
}
/// A status read that must change nothing it could touch.
async fn read_only_status(f: &ControlFixture, goal: GoalId) -> (UnavailableReason, Option<usize>) {
    let before = status_rows(f);
    let answer = status(f, goal).await;
    assert_eq!(status_rows(f), before, "status read changed state");
    answer
}
/// Runs the production attention reconciliation over every Task to its end.
fn reconcile_attention(f: &ControlFixture) {
    let mut after = 0;
    loop {
        let (next, more) = f
            .owner
            .store
            .lock()
            .unwrap()
            .reconcile_runtime_attention(f.owner.instance_id(), f.owner.epoch(), after)
            .unwrap();
        if !more {
            break;
        }
        after = next;
    }
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
    let expected = (UnavailableReason::ProjectLimitUnsupported, Some(4));
    // L3 (Sol 6071558338): every read leaves every related row, version and
    // audit unchanged, before and after the attention reconciliation runs.
    let row = project_row(&f, &project.id.to_string());
    assert_eq!(read_only_status(&f, accepted).await, expected);
    assert_eq!(read_only_status(&f, proposed).await, expected);
    reconcile_attention(&f);
    assert_eq!(
        read_only_status(&f, accepted).await,
        expected,
        "changed after reconcile"
    );
    assert_eq!(
        read_only_status(&f, proposed).await,
        expected,
        "changed after reconcile"
    );
    f.runtime.start().await.unwrap();
    reconcile_attention(&f);
    assert_eq!(read_only_status(&f, accepted).await, expected);
    assert_eq!(read_only_status(&f, proposed).await, expected);
    assert_eq!(
        project_row(&f, &project.id.to_string()),
        row,
        "the stored legacy row changed"
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

/// The three occupancy branches the candidate reader and the final claim
/// count for `task`: a driving Driver, an open Unit, an open phase operation.
fn occupancy(f: &ControlFixture, task: &Task) -> (usize, usize, usize) {
    raw(f)
        .query_row(
            "SELECT (SELECT count(*) FROM task_drivers WHERE task_id=?1 AND state='driving'),(SELECT count(*) FROM execution_units WHERE task_id=?1 AND (native_effects_open=1 OR result_finalization_open=1)),(SELECT count(*) FROM managed_phase_operations WHERE task_id=?1 AND phase_open=1)",
            [task.id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap()
}
async fn wait_until(mut condition: impl FnMut() -> bool, label: &str) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(40);
    while !condition() {
        assert!(tokio::time::Instant::now() < deadline, "{label}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// C-S2a, held occupancy (Sol 6071558338 L1): A's Goal is paused through the
/// lifecycle writer while A's phase operation is open. The pause retires A's
/// Unit and invalidates its Driver, so the open phase operation is the only
/// occupancy left. B, a runnable Task of the same Project in another Goal, is
/// never a candidate until that operation closes.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn c_s2a_held_phase_operation_alone_keeps_second_task_out() {
    use crate::execution::native::PreparationObservation;
    let mut f = fixture("claude", true);
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
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
    let arrived = Arc::new(tokio::sync::Semaphore::new(0));
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let (hook_arrived, hook_release) = (arrived.clone(), release.clone());
    adapter
        .sessions
        .set_preparation_observer(Some(Arc::new(move |observed| {
            let (arrived, release) = (hook_arrived.clone(), hook_release.clone());
            Box::pin(async move {
                if observed == PreparationObservation::BeforeTransport {
                    arrived.add_permits(1);
                    release.acquire().await.unwrap().forget();
                }
            })
        })));
    let (goal_a, tasks) = accept(&f, 1).await;
    let a = tasks[0].clone();
    f.runtime.start().await.unwrap();
    tokio::time::timeout(Duration::from_secs(40), arrived.acquire())
        .await
        .expect("SETUP: A's preparation did not reach the hold")
        .unwrap()
        .forget();
    let (_, tasks) = accept(&f, 1).await;
    let b = tasks[0].clone();
    assert_eq!(b.project_id, a.project_id);
    eprintln!(
        "C-S2a held: A occupancy while preparing {:?}",
        occupancy(&f, &a)
    );
    let expected_goal = f
        .owner
        .store
        .lock()
        .unwrap()
        .goal(goal_a)
        .unwrap()
        .unwrap()
        .version;
    let paused = f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::SetGoalLifecycle {
                project: f.project.id,
                goal: goal_a,
                expected_goal,
                target: crate::runtime::control::GoalControl::Pause,
                reason: "C-S2a held occupancy".into(),
            }),
        )
        .await
        .unwrap();
    assert!(
        matches!(paused, ControlResponse::GoalLifecycleChanged { .. }),
        "SETUP: {paused:?}"
    );
    assert_eq!(
        occupancy(&f, &a),
        (0, 0, 1),
        "SETUP: the open phase operation is not A's only occupancy"
    );
    assert!(
        keys(&f).iter().all(|k| k.task_id != b.id.to_string()),
        "B listed while A's held phase operation occupies the Project"
    );
    assert_eq!(driver_for(&f, &b.id.to_string()), 0);
    // Released, A's start ends before dispatch. The production non-success
    // consumer keeps the plan Held (the paused Goal changed its owners), so
    // the operation stays open and B stays out however often it runs.
    release.add_permits(1);
    wait_until(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.finished && j.attention == Some("original non-success plan Held"))
        },
        "SETUP: A's released start was not held by the non-success consumer",
    )
    .await;
    for _ in 0..3 {
        f.runtime.phase_dispatcher.reconcile_nonsuccess().unwrap();
    }
    assert_eq!(occupancy(&f, &a), (0, 0, 1), "SETUP: held operation closed");
    assert!(
        keys(&f).iter().all(|k| k.task_id != b.id.to_string()),
        "B listed after reconciliation while A's held operation is open"
    );
    // The reader and the service still admit a Task of another Project.
    f.register_real_git_project_named("c-s2a-other-project");
    let (_, tasks) = accept(&f, 1).await;
    let c = tasks[0].clone();
    assert_ne!(c.project_id, a.project_id);
    wait_until(
        || driver_for(&f, &c.id.to_string()) == 1,
        "another Project's Task was not admitted beside the held operation",
    )
    .await;
    assert_eq!(driver_for(&f, &b.id.to_string()), 0, "B was admitted");
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

/// C-S2a, unbound occupancy (Sol 6071558338 L1): A's Driver is claimed and
/// its Source preparation fails on a Project root that is not a Git
/// repository. The Unit is retired and the Driver is retained, so the driving
/// Driver is A's only occupancy. B, in the same Goal, is never listed.
#[tokio::test]
async fn c_s2a_retained_driver_alone_keeps_second_task_out() {
    let f = fixture("claude", true);
    let (_, tasks) = accept(&f, 2).await;
    f.runtime.start().await.unwrap();
    wait_claims(&f, 1).await;
    let claimed: String = raw(&f)
        .query_row("SELECT task_id FROM task_drivers", [], |r| r.get(0))
        .unwrap();
    let (a, b) = if tasks[0].id.to_string() == claimed {
        (&tasks[0], &tasks[1])
    } else {
        (&tasks[1], &tasks[0])
    };
    wait_until(
        || occupancy(&f, a) == (1, 0, 0),
        "SETUP: A's retained Driver did not become its only occupancy",
    )
    .await;
    assert!(
        keys(&f).iter().all(|k| k.task_id != b.id.to_string()),
        "B listed while A's retained Driver occupies the Project"
    );
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(occupancy(&f, a), (1, 0, 0));
    assert_eq!(driver_for(&f, &b.id.to_string()), 0, "B was admitted");
    finish(f).await;
}
