//! SC controls. Every positive uses accepted ingress, a real Git Project, the
//! installed issuer and the configured protocol fixture in commit mode. The
//! fixture is protocol wiring only, never Native qualification.
use super::*;

const LINKS: &str = "SELECT kind FROM audit WHERE kind LIKE 'rrx.private.workflow.%' AND task_id=?1 ORDER BY sequence";

async fn wait_for(mut condition: impl FnMut() -> bool, label: &str, seconds: u64) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(seconds);
    while !condition() {
        assert!(tokio::time::Instant::now() < deadline, "{label}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
fn links(f: &ControlFixture, task: &Task) -> Vec<String> {
    let raw = raw(f);
    let mut statement = raw.prepare(LINKS).unwrap();
    statement
        .query_map([task.id.to_string()], |r| r.get::<_, String>(0))
        .unwrap()
        .map(|kind| {
            kind.unwrap()
                .trim_start_matches("rrx.private.workflow.")
                .to_owned()
        })
        .collect()
}
fn workflow(
    f: &ControlFixture,
    task: &Task,
) -> (crate::domain::Record, crate::workflow::WorkflowSnapshot) {
    let record = f
        .owner
        .store
        .lock()
        .unwrap()
        .records(&task.scope(), RecordKind::Workflow)
        .unwrap()
        .remove(0);
    let snapshot = serde_json::from_value(record.data.clone()).unwrap();
    (record, snapshot)
}
fn stored_task(f: &ControlFixture, task: &Task) -> Task {
    f.owner
        .store
        .lock()
        .unwrap()
        .task(task.id)
        .unwrap()
        .unwrap()
}
fn bound_unit(f: &ControlFixture, task: &Task) -> crate::execution::ExecutionUnit {
    let (_, snapshot) = workflow(f, task);
    let reference = snapshot
        .history
        .iter()
        .find_map(|a| a.execution.as_ref())
        .expect("bound attempt")
        .clone();
    f.owner
        .store
        .lock()
        .unwrap()
        .execution_unit(reference.unit)
        .unwrap()
}
/// The fixture's bootstrap hold, then its commit-mode completion.
fn release_completion(f: &ControlFixture, task: &Task) {
    release_unit(f, &bound_unit(f, task));
}
fn release_unit(f: &ControlFixture, unit: &crate::execution::ExecutionUnit) {
    let profile = crate::execution::resources::ResourceManager::new(f.owner.clone())
        .profile(unit)
        .unwrap();
    std::fs::write(profile.output.join("fixture-bootstrap-release"), "release").unwrap();
    std::fs::write(profile.output.join("fixture-release"), "release").unwrap();
}
async fn wait_normal_bound(f: &ControlFixture, task: &Task) {
    wait_for(
        || links(f, task).first().map(String::as_str) == Some("session_bound"),
        "SETUP: normal Bound absent",
        60,
    )
    .await;
}

/// SC1: the full SAME-operation chain, held start -> normal Bound -> owned
/// success -> capture -> claim -> Passed -> observed -> closure.
async fn sc1_success_lane(provider: &str) {
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    let marked = stored_task(&f, task);
    let unit = bound_unit(&f, task);
    let contexts_before: i64 = raw(&f)
        .query_row(
            "SELECT count(*) FROM context_versions WHERE task_id=?1",
            [task.id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    release_completion(&f, task);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    while links(&f, task).last().map(String::as_str) != Some("phase_closed") {
        if tokio::time::Instant::now() > deadline {
            let jobs = f.runtime.phase_jobs.observed_jobs();
            let (_, snapshot) = workflow(&f, task);
            let now = f
                .owner
                .store
                .lock()
                .unwrap()
                .execution_unit(unit.id)
                .unwrap();
            let handle = snapshot
                .history
                .iter()
                .find_map(|a| a.execution.clone())
                .unwrap();
            let status = f
                .runtime
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
                .status(&handle)
                .map(|s| {
                    format!(
                        "{:?} {:?} {:?} {:?}",
                        s.session.state, s.work, s.failure, s.diagnostic
                    )
                });
            panic!(
                "SC1 {provider}: success phase_closed absent; native {status:?}; links {:?}; jobs {jobs:?}; attempt {:?}; unit {:?}/{:?}/{:?}",
                links(&f, task),
                snapshot.active.map(|i| (
                    snapshot.history[i].state.clone(),
                    snapshot.history[i].detail.clone()
                )),
                now.state,
                now.work,
                now.native_effects_open
            );
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(
        links(&f, task),
        [
            "session_bound",
            "gate_claim",
            "gate_observed",
            "phase_closed"
        ],
        "SC1 {provider}: typed links"
    );
    let closed = stored_task(&f, task);
    assert_eq!(
        closed.version,
        marked.version + 1,
        "SC1 {provider}: Task bumped once"
    );
    assert_eq!(
        closed.phase.as_deref(),
        Some("commit"),
        "SC1 {provider}: next phase"
    );
    let (_, snapshot) = workflow(&f, task);
    assert!(snapshot.active.is_none() && !snapshot.finished);
    assert!(
        snapshot
            .completed
            .contains_key(&crate::workflow::Phase::Implement)
    );
    let after = f
        .owner
        .store
        .lock()
        .unwrap()
        .execution_unit(unit.id)
        .unwrap();
    assert!(
        !after.result_finalization_open,
        "SC1 {provider}: finalization closed"
    );
    let artifact = f
        .owner
        .store
        .lock()
        .unwrap()
        .result_artifact(after.artifact_id.expect("published artifact"))
        .unwrap();
    assert_eq!(artifact.state, crate::execution::ArtifactState::Published);
    assert_ne!(
        artifact.revision, unit.base_sha,
        "SC1 {provider}: commit-mode HEAD"
    );
    let contexts_after: i64 = raw(&f)
        .query_row(
            "SELECT count(*) FROM context_versions WHERE task_id=?1",
            [task.id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        contexts_after,
        contexts_before + 1,
        "SC1 {provider}: one new Context"
    );
    let marker: Option<String> = raw(&f)
        .query_row(
            "SELECT json_extract(body,'$.marker') FROM task_drivers WHERE task_id=?1 AND state='driving'",
            [task.id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert!(marker.is_none(), "SC1 {provider}: Driver marker-free");
    wait_for(
        || {
            f.owner
                .store
                .lock()
                .unwrap()
                .validate_task_driver(task.id)
                .is_ok()
        },
        &format!("SC1 {provider}: generic Driver validate after closure"),
        20,
    )
    .await;
    // results.git holds the artifact's commit/base refs and passes fsck.
    let repository = f
        .owner
        .root
        .join("projects")
        .join(closed.project_id.to_string())
        .join("results.git");
    for (name, expected) in [("commit", &artifact.revision), ("base", &unit.base_sha)] {
        let output = std::process::Command::new("git")
            .arg("--git-dir")
            .arg(&repository)
            .args(["rev-parse", &format!("refs/rrx/{}/{name}", artifact.id)])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "SC1 {provider}: results.git {name} ref"
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            expected.as_str(),
            "SC1 {provider}: results.git {name} ref target"
        );
    }
    let fsck = std::process::Command::new("git")
        .arg("--git-dir")
        .arg(&repository)
        .args(["fsck", "--strict", "--no-dangling"])
        .output()
        .unwrap();
    assert!(
        fsck.status.success(),
        "SC1 {provider}: results.git fsck: {}",
        String::from_utf8_lossy(&fsck.stderr)
    );
    // SC12: status reports the read-only next-phase wait; the closed Task's
    // state and blockers stay as the closure wrote them.
    let response = f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::GoalTasks {
                view: None,
                project: closed.project_id,
                goal: closed.goal_id,
                after: None,
                maximum: 16,
            }),
        )
        .await
        .unwrap();
    let ControlResponse::GoalTaskPage { tasks, .. } = response else {
        panic!("SC12 {provider}: GoalTaskPage")
    };
    let facts = tasks
        .iter()
        .find(|t| t.scope.task_id == Some(task.id))
        .expect("SC12: task facts");
    assert_eq!(
        facts.workflow_wait.as_ref().map(|w| w.kind),
        Some(crate::runtime::control::WorkflowWaitKind::NextPhaseUnavailable),
        "SC12 {provider}: next_phase_unavailable"
    );
    let now = stored_task(&f, task);
    assert_eq!(
        (now.version, &now.state, &now.blockers),
        (closed.version, &closed.state, &closed.blockers),
        "SC12 {provider}: status is read-only"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc1_claude_quick_implement_closes_once_through_the_full_chain() {
    sc1_success_lane("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc1_codex_quick_implement_closes_once_through_the_full_chain() {
    sc1_success_lane("codex").await;
}

/// SC7 (ordinary failure): a bound non-success terminal installs no settled
/// continuation, writes no gate link and leaves the Task as marked.
/// The fixture peer's genuine non-success terminal for each provider: Codex
/// `ordinary-failure`; Claude has no such payload, so its error terminal.
async fn sc7_ordinary_failure(provider: &str) {
    let scenario = if provider == "codex" {
        "ordinary-failure"
    } else {
        "authentication-terminal"
    };
    let mut f = fixture_mode(provider, true, Some(scenario), |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    let marked = stored_task(&f, task);
    release_completion(&f, task);
    wait_for(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.finished)
        },
        "SETUP: ordinary-failure job did not finish",
        60,
    )
    .await;
    // Give the Root sweep and the Driver several passes.
    // Wake the service far more often than 5 s: only the job's own re-poll
    // rule, not the service cadence, may bound the settlement polls.
    let polled = std::time::Instant::now();
    while polled.elapsed() < Duration::from_secs(8) {
        f.runtime.wake.notify_one();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let jobs = f.runtime.phase_jobs.observed_jobs();
    // SC5(c): the non-success job is re-polled at most once per 5 s and
    // reports its attention text.
    let polls = f
        .runtime
        .phase_jobs
        .observed_success_turns()
        .iter()
        .filter(|(_, label)| *label == "settlement poll")
        .count();
    let bound = polled.elapsed().as_secs().div_ceil(5) as usize + 1;
    assert!(
        (1..=bound).contains(&polls),
        "SC5(c) {provider}: {polls} settlement polls, bound {bound}"
    );
    assert!(
        jobs.iter()
            .all(|j| j.success_attention == Some("owned success settlement not observed")),
        "SC5(c) {provider}: attention text; jobs {jobs:?}"
    );
    let (_, snapshot) = workflow(&f, task);
    eprintln!(
        "SC7 {provider}: jobs {jobs:?}; attempt {:?}",
        snapshot.active.map(|i| (
            snapshot.history[i].state.clone(),
            snapshot.history[i].detail.clone()
        ))
    );
    assert_eq!(
        links(&f, task),
        ["session_bound"],
        "SC7 {provider}: no gate link"
    );
    assert!(
        jobs.iter().all(|j| !j.success),
        "SC7 {provider}: no settled continuation"
    );
    assert!(
        jobs.iter().all(|j| !j.success_closed
            && j.observation != crate::runtime::phase_jobs::InvocationObservation::ClosedSuccess),
        "SC7 {provider}: never success-closed; jobs {jobs:?}"
    );
    assert_eq!(
        stored_task(&f, task).version,
        marked.version,
        "SC7 {provider}: Task unchanged"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc7_claude_ordinary_failure_installs_no_settlement() {
    sc7_ordinary_failure("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc7_codex_ordinary_failure_installs_no_settlement() {
    sc7_ordinary_failure("codex").await;
}

/// SC2 (late): the normal write is held until the owned success settlement
/// exists; normal then refuses and the Root sweep late-binds once with
/// `closed_settlement`, and the SAME chain closes.
async fn sc2_late(provider: &str) {
    use crate::runtime::phase_jobs::WritePause;
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    // A failed assertion must not leave the start task parked forever.
    struct Release(Arc<WritePause>);
    impl Drop for Release {
        fn drop(&mut self) {
            self.0.release();
        }
    }
    let pause = Release(WritePause::arm(task.id));
    f.runtime.start().await.unwrap();
    wait_for(|| pause.0.reached(), "SETUP: normal write not reached", 60).await;
    assert!(
        links(&f, task).is_empty(),
        "SC2 {provider}: unbound while held"
    );
    let unit = f.runtime.phase_jobs.observed_jobs()[0].unit;
    let unit = f.owner.store.lock().unwrap().execution_unit(unit).unwrap();
    release_unit(&f, &unit);
    wait_for(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.settled == Some(true))
        },
        "SETUP: owned settlement absent while normal held",
        60,
    )
    .await;
    pause.0.release();
    wait_for(
        || links(&f, task).last().map(String::as_str) == Some("phase_closed"),
        &format!("SC2 {provider}: late chain did not close"),
        90,
    )
    .await;
    assert_eq!(
        links(&f, task),
        [
            "session_bound",
            "gate_claim",
            "gate_observed",
            "phase_closed"
        ],
        "SC2 {provider}: typed links"
    );
    let (source, receipt): (String, Option<String>) = raw(&f)
        .query_row(
            "SELECT json_extract(data,'$.proof_source'),json_extract(data,'$.private_receipt_ref') FROM audit WHERE task_id=?1 AND kind='rrx.private.workflow.session_bound'",
            [task.id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(source, "closed_settlement", "SC2 {provider}: late proof");
    assert!(receipt.is_some(), "SC2 {provider}: private receipt ref");
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc2_claude_late_binding_after_settlement_closes_once() {
    sc2_late("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc2_codex_late_binding_after_settlement_closes_once() {
    sc2_late("codex").await;
}

/// SC3-P: a Goal change through the existing lifecycle writer while the normal
/// write is held makes that write refuse and every later plan refuse on the
/// original parent projection: zero links, no frame refresh.
async fn sc3p_parent_change(provider: &str) {
    use crate::runtime::phase_jobs::WritePause;
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (goal, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    struct Release(Arc<WritePause>);
    impl Drop for Release {
        fn drop(&mut self) {
            self.0.release();
        }
    }
    let pause = Release(WritePause::arm(task.id));
    f.runtime.start().await.unwrap();
    wait_for(|| pause.0.reached(), "SETUP: normal write not reached", 60).await;
    // The existing lifecycle writer's Pause. (Resume of a prepared Goal is
    // unavailable, so no legitimate writer leaves a Running Goal with only a
    // changed projection; the paused-state refusal and the projection check
    // both stand here and are not separated by this control.)
    for target in [crate::runtime::control::GoalControl::Pause] {
        let expected_goal = f
            .owner
            .store
            .lock()
            .unwrap()
            .goal(goal)
            .unwrap()
            .unwrap()
            .version;
        let changed = f
            .runtime
            .handle_control(
                &f.socket,
                f.request(ControlAction::SetGoalLifecycle {
                    project: f.project.id,
                    goal,
                    expected_goal,
                    target,
                    reason: "SC3-P parent change".into(),
                }),
            )
            .await
            .unwrap();
        assert!(
            matches!(changed, ControlResponse::GoalLifecycleChanged { .. }),
            "SETUP: legitimate Goal writer refused: {changed:?}"
        );
    }
    pause.0.release();
    let started = std::time::Instant::now();
    while started.elapsed() < Duration::from_secs(6) {
        f.runtime.wake.notify_one();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let jobs = f.runtime.phase_jobs.observed_jobs();
    assert!(
        links(&f, task).is_empty(),
        "SC3-P {provider}: zero links; jobs {jobs:?}"
    );
    assert!(
        jobs.iter().all(|j| !j.bound),
        "SC3-P {provider}: unbound; jobs {jobs:?}"
    );
    let turns = f.runtime.phase_jobs.observed_success_turns();
    assert!(
        turns.iter().any(|(_, l)| *l == "normal plan refused"),
        "SC3-P {provider}: later normal plan refuses; turns {turns:?}"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc3p_claude_parent_change_during_hold_refuses_binding() {
    sc3p_parent_change("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc3p_codex_parent_change_during_hold_refuses_binding() {
    sc3p_parent_change("codex").await;
}

/// SC-U (SC-FIX-M01): a genuine evaluation failure after the SAME claim's
/// check, with currency intact, is stored as the sealed Unknown disposition.
/// The stimulus is negative-only: the results.git commit ref is moved.
async fn scu_unknown(provider: &str) {
    use crate::runtime::phase_jobs::{SETTLED_EVALUATION, WritePause};
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    struct Release(Arc<WritePause>);
    impl Drop for Release {
        fn drop(&mut self) {
            self.0.release();
        }
    }
    let pause = Release(WritePause::arm_at(task.id, SETTLED_EVALUATION));
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    let marked = stored_task(&f, task);
    let unit = bound_unit(&f, task);
    release_completion(&f, task);
    wait_for(
        || pause.0.reached(),
        "SETUP: settled evaluation not reached",
        90,
    )
    .await;
    assert_eq!(links(&f, task), ["session_bound", "gate_claim"]);
    let terminal = f
        .owner
        .store
        .lock()
        .unwrap()
        .execution_unit(unit.id)
        .unwrap();
    let artifact: String = raw(&f)
        .query_row(
            "SELECT id FROM result_artifacts WHERE unit_id=?1",
            [terminal.id.to_string()],
            |r| r.get(0),
        )
        .expect("SETUP: Ready artifact");
    let repository = f
        .owner
        .root
        .join("projects")
        .join(marked.project_id.to_string())
        .join("results.git");
    let moved = std::process::Command::new("git")
        .arg("--git-dir")
        .arg(&repository)
        .args([
            "update-ref",
            &format!("refs/rrx/{artifact}/commit"),
            &terminal.base_sha,
        ])
        .output()
        .unwrap();
    assert!(moved.status.success(), "SETUP: negative-only ref move");
    pause.0.release();
    wait_for(
        || links(&f, task).len() == 3,
        &format!("SC-U {provider}: gate_observed absent"),
        60,
    )
    .await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert_eq!(
        links(&f, task),
        ["session_bound", "gate_claim", "gate_observed"],
        "SC-U {provider}: no closure"
    );
    let (_, snapshot) = workflow(&f, task);
    let attempt = &snapshot.history[snapshot.active.expect("SC-U: phase stays open")];
    assert_eq!(attempt.state, crate::workflow::AttemptState::Waiting);
    assert_eq!(
        attempt.detail.as_deref(),
        Some(crate::state::managed_binding::GATE_UNKNOWN_DETAIL)
    );
    assert_eq!(
        snapshot.held_reason.as_deref(),
        Some(crate::state::managed_binding::GATE_UNKNOWN_DETAIL)
    );
    let last = attempt.observations.last().unwrap();
    assert!(last.outcome.is_none(), "SC-U {provider}: no known outcome");
    assert_eq!(
        stored_task(&f, task).version,
        marked.version,
        "SC-U {provider}: Task unchanged"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn scu_claude_post_claim_evaluation_failure_is_durable_unknown() {
    scu_unknown("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn scu_codex_post_claim_evaluation_failure_is_durable_unknown() {
    scu_unknown("codex").await;
}

struct Held(Arc<crate::runtime::phase_jobs::WritePause>);
impl Drop for Held {
    fn drop(&mut self) {
        self.0.release();
    }
}
fn held(task: &Task, site: &'static str) -> Held {
    Held(crate::runtime::phase_jobs::WritePause::arm_at(
        task.id, site,
    ))
}

/// BR1 / BR1-P: unlinked owner plan -> normal binding commit -> Core resumes.
/// The owner-validation consumer is refused once (marker at `site`), retries
/// once, obtains its authority and the SAME chain closes.
async fn br1(provider: &str, site: &'static str, mark: &'static str) {
    use crate::runtime::phase_jobs::{NORMAL_WRITE, counted};
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    let binding = held(task, NORMAL_WRITE);
    f.runtime.start().await.unwrap();
    wait_for(
        || binding.0.reached(),
        "SETUP: normal write not reached",
        60,
    )
    .await;
    // Armed only after the start returned and the binding is parked before
    // its write: the Core's next owner plan at `site` is necessarily unlinked.
    let core = held(task, site);
    let unit = f.runtime.phase_jobs.observed_jobs()[0].unit;
    let unit = f.owner.store.lock().unwrap().execution_unit(unit).unwrap();
    release_unit(&f, &unit);
    wait_for(
        || core.0.reached(),
        "SETUP: Core owner plan not reached",
        60,
    )
    .await;
    assert!(links(&f, task).is_empty(), "SETUP: plan must be unlinked");
    binding.0.release();
    wait_normal_bound(&f, task).await;
    core.0.release();
    wait_for(
        || links(&f, task).last().map(String::as_str) == Some("phase_closed"),
        &format!(
            "BR1 {provider} {site}: chain did not close; jobs {:?}",
            f.runtime.phase_jobs.observed_jobs()
        ),
        90,
    )
    .await;
    assert_eq!(
        counted(task.id, mark),
        1,
        "BR1 {provider}: one marker at {mark}"
    );
    assert_eq!(counted(task.id, "br retry"), 1, "BR1 {provider}: one retry");
    let other = if mark == "mark planning" {
        "mark immediate"
    } else {
        "mark planning"
    };
    assert_eq!(
        counted(task.id, other),
        0,
        "BR1 {provider}: no marker at {other}"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn br1_claude_owner_immediate_retries_once_after_binding() {
    br1(
        "claude",
        crate::runtime::phase_jobs::OWNER_IMMEDIATE,
        "mark immediate",
    )
    .await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn br1_codex_owner_immediate_retries_once_after_binding() {
    br1(
        "codex",
        crate::runtime::phase_jobs::OWNER_IMMEDIATE,
        "mark immediate",
    )
    .await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn br1p_claude_owner_planning_retries_once_after_binding() {
    br1(
        "claude",
        crate::runtime::phase_jobs::OWNER_PLANNING,
        "mark planning",
    )
    .await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn br1p_codex_owner_planning_retries_once_after_binding() {
    br1(
        "codex",
        crate::runtime::phase_jobs::OWNER_PLANNING,
        "mark planning",
    )
    .await;
}

/// BR2: the parent changes (Goal Pause through the lifecycle writer) while the
/// Core's unlinked owner plan is parked and the binding stays held: no retry,
/// zero links, the refusal stands.
async fn br2(provider: &str) {
    use crate::runtime::phase_jobs::{NORMAL_WRITE, OWNER_IMMEDIATE, counted};
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (goal, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    let binding = held(task, NORMAL_WRITE);
    let core = held(task, OWNER_IMMEDIATE);
    f.runtime.start().await.unwrap();
    wait_for(
        || binding.0.reached(),
        "SETUP: normal write not reached",
        60,
    )
    .await;
    wait_for(
        || core.0.reached(),
        "SETUP: Core owner plan not reached",
        60,
    )
    .await;
    let expected_goal = f
        .owner
        .store
        .lock()
        .unwrap()
        .goal(goal)
        .unwrap()
        .unwrap()
        .version;
    let changed = f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::SetGoalLifecycle {
                project: f.project.id,
                goal,
                expected_goal,
                target: crate::runtime::control::GoalControl::Pause,
                reason: "BR2 parent change".into(),
            }),
        )
        .await
        .unwrap();
    assert!(
        matches!(changed, ControlResponse::GoalLifecycleChanged { .. }),
        "SETUP: legitimate Goal writer refused: {changed:?}"
    );
    core.0.release();
    wait_for(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.owner_live == Some(false))
        },
        "BR2: Core did not end after the refusal",
        60,
    )
    .await;
    assert_eq!(counted(task.id, "br retry"), 0, "BR2 {provider}: no retry");
    assert!(links(&f, task).is_empty(), "BR2 {provider}: zero links");
    drop(binding);
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn br2_claude_parent_drift_never_retries() {
    br2("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn br2_codex_parent_drift_never_retries() {
    br2("codex").await;
}

/// BR3: a plan made after the binding (linked), refused for the parent change:
/// no marker and no retry.
async fn br3(provider: &str) {
    use crate::runtime::phase_jobs::{OWNER_IMMEDIATE, counted};
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (goal, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    let core = held(task, OWNER_IMMEDIATE);
    release_completion(&f, task);
    wait_for(
        || core.0.reached(),
        "SETUP: linked owner plan not reached",
        60,
    )
    .await;
    // The natural pre-binding race may already have marked and retried an
    // earlier plan; only this linked plan's outcome is measured.
    let base = |label| counted(task.id, label);
    let (immediate, planning, retries) = (
        base("mark immediate"),
        base("mark planning"),
        base("br retry"),
    );
    let expected_goal = f
        .owner
        .store
        .lock()
        .unwrap()
        .goal(goal)
        .unwrap()
        .unwrap()
        .version;
    let changed = f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::SetGoalLifecycle {
                project: f.project.id,
                goal,
                expected_goal,
                target: crate::runtime::control::GoalControl::Pause,
                reason: "BR3 parent change".into(),
            }),
        )
        .await
        .unwrap();
    assert!(
        matches!(changed, ControlResponse::GoalLifecycleChanged { .. }),
        "SETUP: legitimate Goal writer refused: {changed:?}"
    );
    core.0.release();
    wait_for(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.owner_live == Some(false))
        },
        "BR3: Core did not end after the refusal",
        60,
    )
    .await;
    assert_eq!(
        counted(task.id, "mark immediate"),
        immediate,
        "BR3 {provider}: no marker"
    );
    assert_eq!(
        counted(task.id, "mark planning"),
        planning,
        "BR3 {provider}: no marker"
    );
    assert_eq!(
        counted(task.id, "br retry"),
        retries,
        "BR3 {provider}: no retry"
    );
    assert_eq!(
        links(&f, task),
        ["session_bound"],
        "BR3 {provider}: no gate link"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn br3_claude_linked_plan_is_never_marked() {
    br3("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn br3_codex_linked_plan_is_never_marked() {
    br3("codex").await;
}

/// SC6: STANDARD Requirements. Bind -> capture -> claim -> observed Waiting
/// (evidence integration unavailable). After many Driver polls: one claim, no
/// closure, Task unchanged, Requirements not completed.
async fn sc6_requirements(provider: &str) {
    let mut f = fixture_mode(provider, true, None, |config| {
        config.minimum_workflow = crate::config::WorkflowClass::Standard;
        config.workflow.risk_mapping = [crate::config::WorkflowClass::Standard; 4];
    });
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (goal, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    let marked = stored_task(&f, task);
    release_completion(&f, task);
    wait_for(
        || links(&f, task).len() >= 3,
        &format!(
            "SC6 {provider}: gate_observed absent; links {:?}",
            links(&f, task)
        ),
        90,
    )
    .await;
    let started = std::time::Instant::now();
    while started.elapsed() < Duration::from_secs(6) {
        f.runtime.wake.notify_one();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(
        links(&f, task),
        ["session_bound", "gate_claim", "gate_observed"],
        "SC6 {provider}: one claim, no closure"
    );
    let (_, snapshot) = workflow(&f, task);
    let attempt = &snapshot.history[snapshot.active.expect("SC6: phase stays open")];
    assert_eq!(attempt.phase, crate::workflow::Phase::Requirements);
    assert_eq!(attempt.state, crate::workflow::AttemptState::Waiting);
    assert!(
        !snapshot
            .completed
            .contains_key(&crate::workflow::Phase::Requirements),
        "SC6 {provider}: Requirements not completed"
    );
    assert_eq!(
        stored_task(&f, task).version,
        marked.version,
        "SC6 {provider}: Task unchanged"
    );
    // SC12: the Requirements wait is reported read-only.
    let response = f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::GoalTasks {
                view: None,
                project: marked.project_id,
                goal,
                after: None,
                maximum: 16,
            }),
        )
        .await
        .unwrap();
    let ControlResponse::GoalTaskPage { tasks: facts, .. } = response else {
        panic!("SC12 {provider}: GoalTaskPage")
    };
    let facts = facts
        .iter()
        .find(|t| t.scope.task_id == Some(task.id))
        .expect("SC12: task facts");
    assert_eq!(
        facts.workflow_wait.as_ref().map(|w| w.kind),
        Some(crate::runtime::control::WorkflowWaitKind::EvidenceIntegrationUnavailable),
        "SC12 {provider}: evidence_integration_unavailable"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc6_claude_standard_requirements_waits_without_closure() {
    sc6_requirements("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc6_codex_standard_requirements_waits_without_closure() {
    sc6_requirements("codex").await;
}

/// SC8: graph integrity. Between the Passed observation and the closure, a
/// negative-only results.git ref move makes `settled_publication` refuse; the
/// closure never commits and the Task stays as marked.
async fn sc8_graph(provider: &str) {
    use crate::runtime::phase_jobs::SETTLED_CLOSURE;
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    let closure = held(task, SETTLED_CLOSURE);
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    let marked = stored_task(&f, task);
    let unit = bound_unit(&f, task);
    release_completion(&f, task);
    wait_for(
        || closure.0.reached(),
        "SETUP: settled closure not reached",
        90,
    )
    .await;
    assert_eq!(
        links(&f, task),
        ["session_bound", "gate_claim", "gate_observed"]
    );
    let artifact: String = raw(&f)
        .query_row(
            "SELECT id FROM result_artifacts WHERE unit_id=?1",
            [unit.id.to_string()],
            |r| r.get(0),
        )
        .expect("SETUP: Ready artifact");
    let repository = f
        .owner
        .root
        .join("projects")
        .join(marked.project_id.to_string())
        .join("results.git");
    let moved = std::process::Command::new("git")
        .arg("--git-dir")
        .arg(&repository)
        .args([
            "update-ref",
            &format!("refs/rrx/{artifact}/commit"),
            &unit.base_sha,
        ])
        .output()
        .unwrap();
    assert!(moved.status.success(), "SETUP: negative-only ref move");
    closure.0.release();
    let started = std::time::Instant::now();
    while started.elapsed() < Duration::from_secs(6) {
        f.runtime.wake.notify_one();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(
        links(&f, task),
        ["session_bound", "gate_claim", "gate_observed"],
        "SC8 {provider}: closure never commits"
    );
    assert_eq!(
        stored_task(&f, task).version,
        marked.version,
        "SC8 {provider}: Task unchanged"
    );
    let state: String = raw(&f)
        .query_row(
            "SELECT state FROM result_artifacts WHERE id=?1",
            [&artifact],
            |r| r.get(0),
        )
        .unwrap();
    assert_ne!(state, "published", "SC8 {provider}: artifact not Published");
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc8_claude_results_graph_corruption_blocks_closure() {
    sc8_graph("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc8_codex_results_graph_corruption_blocks_closure() {
    sc8_graph("codex").await;
}

/// SC4: an uncertain write (an `Err` before the transaction, or after a
/// genuine commit) at each SC writer is confirmed Known or RolledBack, and
/// the SAME chain still closes with exactly one link of each kind.
async fn sc4_uncertain(
    provider: &str,
    site: &'static str,
    fault: crate::state::managed_binding::fault::CommitFault,
) {
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    crate::state::managed_binding::fault::arm_commit_fault(task.id, site, fault);
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    let marked = stored_task(&f, task);
    release_completion(&f, task);
    wait_for(
        || links(&f, task).last().map(String::as_str) == Some("phase_closed"),
        &format!(
            "SC4 {provider} {site} {fault:?}: chain did not close; links {:?}",
            links(&f, task)
        ),
        90,
    )
    .await;
    assert!(
        !crate::state::managed_binding::fault::armed(task.id, site),
        "SETUP: SC4 {provider} {site} {fault:?}: the fault never fired"
    );
    assert_eq!(
        links(&f, task),
        [
            "session_bound",
            "gate_claim",
            "gate_observed",
            "phase_closed"
        ],
        "SC4 {provider} {site} {fault:?}: exactly one link of each kind"
    );
    assert_eq!(
        stored_task(&f, task).version,
        marked.version + 1,
        "SC4 {provider} {site} {fault:?}: Task bumped once"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
async fn sc4_all(provider: &str) {
    use crate::state::managed_binding::fault::{BIND, CLAIM, CLOSURE, CommitFault, OBSERVED};
    for site in [BIND, CLAIM, OBSERVED, CLOSURE] {
        for fault in [CommitFault::BeforeCommit, CommitFault::AfterCommit] {
            sc4_uncertain(provider, site, fault).await;
        }
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc4_claude_uncertain_writes_confirm_and_close_once() {
    sc4_all("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc4_codex_uncertain_writes_confirm_and_close_once() {
    sc4_all("codex").await;
}
