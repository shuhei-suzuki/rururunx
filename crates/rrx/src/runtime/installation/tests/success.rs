//! SC controls. Every positive uses accepted ingress, a real Git Project, the
//! installed issuer and the configured protocol fixture in commit mode. The
//! fixture is protocol wiring only, never Native qualification.
use super::*;

const LINKS: &str = "SELECT kind FROM audit WHERE kind LIKE 'rrx.private.workflow.%' AND task_id=?1 ORDER BY sequence";

pub(super) async fn wait_for(mut condition: impl FnMut() -> bool, label: &str, seconds: u64) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(seconds);
    while !condition() {
        assert!(tokio::time::Instant::now() < deadline, "{label}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
pub(super) fn links(f: &ControlFixture, task: &Task) -> Vec<String> {
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
pub(super) fn workflow(
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
pub(super) fn stored_task(f: &ControlFixture, task: &Task) -> Task {
    f.owner
        .store
        .lock()
        .unwrap()
        .task(task.id)
        .unwrap()
        .unwrap()
}
pub(super) fn bound_unit(f: &ControlFixture, task: &Task) -> crate::execution::ExecutionUnit {
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
pub(super) fn release_completion(f: &ControlFixture, task: &Task) {
    release_unit(f, &bound_unit(f, task));
}
pub(super) fn release_unit(f: &ControlFixture, unit: &crate::execution::ExecutionUnit) {
    let profile = crate::execution::resources::ResourceManager::new(f.owner.clone())
        .profile(unit)
        .unwrap();
    std::fs::write(profile.output.join("fixture-bootstrap-release"), "release").unwrap();
    std::fs::write(profile.output.join("fixture-release"), "release").unwrap();
}
pub(super) async fn wait_normal_bound(f: &ControlFixture, task: &Task) {
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
                view: Some(crate::runtime::control::GoalReadView::RecordedV1),
                project: closed.project_id,
                goal: closed.goal_id,
                after: None,
                maximum: 16,
            }),
        )
        .await
        .unwrap();
    let printed = crate::cli::goal_facts::render(&response, false).unwrap();
    assert!(
        printed.contains("Workflow wait: \"next_phase_unavailable\""),
        "SC12 {provider}: CLI prints the wait: {printed}"
    );
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
    crate::issue87_trace::mark(&format!("SC7 {provider} start"));
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    crate::issue87_trace::mark(&format!("SC7 {provider} normal bound"));
    let marked = stored_task(&f, task);
    release_completion(&f, task);
    crate::issue87_trace::mark(&format!("SC7 {provider} completion released"));
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
    crate::issue87_trace::mark(&format!("SC7 {provider} job finished"));
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
    crate::issue87_trace::mark(&format!(
        "SC7 {provider} window end: polls={polls} turns={:?} jobs={jobs:?}",
        f.runtime.phase_jobs.observed_success_turns()
    ));
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
    let issue87_shutdown = f.runtime.shutdown().await;
    crate::issue87_trace::mark(&format!("SC7 {provider} shutdown {issue87_shutdown:?}"));
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
    use crate::runtime::phase_jobs::{OWNER_IMMEDIATE_LINKED, counted};
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (goal, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    // Held only by a plan that already carries the binding link: an earlier
    // unlinked plan still in flight passes this site and is not measured.
    let core = held(task, OWNER_IMMEDIATE_LINKED);
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
                view: Some(crate::runtime::control::GoalReadView::RecordedV1),
                project: marked.project_id,
                goal,
                after: None,
                maximum: 16,
            }),
        )
        .await
        .unwrap();
    // The CLI prints the same read-only wait.
    let printed = crate::cli::goal_facts::render(&response, false).unwrap();
    assert!(
        printed.contains("Workflow wait: \"evidence_integration_unavailable\""),
        "SC12 {provider}: CLI prints the wait: {printed}"
    );
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
    // C-S4d (3): the S4 queue derives the same production-written wait as a
    // typed Task fact, with Task operations unavailable.
    let ControlResponse::AttentionPage { items, .. } = f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::AttentionQueue {
                project: Some(marked.project_id),
                after: None,
                maximum: 128,
            }),
        )
        .await
        .unwrap()
    else {
        panic!("C-S4d {provider}: AttentionPage")
    };
    let item = items
        .iter()
        .find(|i| i.task == Some(task.id))
        .expect("C-S4d: waiting Task item");
    assert!(
        item.facts.iter().any(|f| matches!(
            f,
            crate::runtime::control::AttentionFact::WorkflowWait { wait }
                if wait.kind == crate::runtime::control::WorkflowWaitKind::EvidenceIntegrationUnavailable
        )),
        "C-S4d {provider}: workflow wait fact: {:?}",
        item.facts
    );
    assert!(item.operations.iter().all(|o| o.availability
        == crate::runtime::control::OperationAvailability::Unavailable {
            reason: UnavailableReason::TaskDriverUnavailable
        }));
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
    // The uncertain write was confirmed Known: the Driver is published
    // marker-free and the job's custody released (as in SC1).
    wait_for(
        || {
            f.owner
                .store
                .lock()
                .unwrap()
                .validate_task_driver(task.id)
                .is_ok()
                && f.runtime.phase_jobs.observed_jobs().is_empty()
        },
        &format!("SC4 {provider} {site} {fault:?}: confirmation, publication and release"),
        30,
    )
    .await;
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

fn binding_proof(f: &ControlFixture, task: &Task) -> String {
    raw(f)
        .query_row(
            "SELECT json_extract(data,'$.proof_source') FROM audit WHERE task_id=?1 AND kind='rrx.private.workflow.session_bound'",
            [task.id.to_string()],
            |r| r.get(0),
        )
        .unwrap()
}

/// SC3: while the normal write is held, the genuine Native owner advances its
/// own Session record (bootstrap released). The held write returns the typed
/// pre-write `Conflict`; a later normal plan binds `normal_return` once.
async fn sc3_conflict(provider: &str) {
    use crate::runtime::phase_jobs::NORMAL_WRITE;
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
    let unit = f.runtime.phase_jobs.observed_jobs()[0].unit;
    let session_version = |f: &ControlFixture| -> i64 {
        raw(f)
            .query_row(
                "SELECT version FROM records WHERE kind='session' AND task_id=?1",
                [task.id.to_string()],
                |r| r.get(0),
            )
            .unwrap()
    };
    let before = session_version(&f);
    // Only the bootstrap: the owner projects its Session; completion stays held.
    let profile = crate::execution::resources::ResourceManager::new(f.owner.clone())
        .profile(&f.owner.store.lock().unwrap().execution_unit(unit).unwrap())
        .unwrap();
    std::fs::write(profile.output.join("fixture-bootstrap-release"), "release").unwrap();
    let advanced = {
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        loop {
            if session_version(&f) > before {
                break true;
            }
            if std::time::Instant::now() > deadline {
                break false;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    };
    if !advanced {
        panic!("SETUP: SC3 {provider}: the genuine owner produced no Session record advance");
    }
    binding.0.release();
    wait_normal_bound(&f, task).await;
    let jobs = f.runtime.phase_jobs.observed_jobs();
    assert!(
        jobs[0]
            .binding_error
            .as_deref()
            .is_some_and(|e| e.contains("binding definitively refused")),
        "SC3 {provider}: typed Conflict from the held write; jobs {jobs:?}"
    );
    assert_eq!(
        binding_proof(&f, task),
        "normal_return",
        "SC3 {provider}: retried normal"
    );
    release_completion(&f, task);
    wait_for(
        || links(&f, task).last().map(String::as_str) == Some("phase_closed"),
        "SC3: chain did not close",
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
        "SC3 {provider}: no closed_settlement link, one binding"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc3_claude_owner_session_advance_conflicts_then_binds_normal() {
    sc3_conflict("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc3_codex_owner_session_advance_conflicts_then_binds_normal() {
    sc3_conflict("codex").await;
}

/// SC3 (lock variant): a second connection holds the SQLite writer lock
/// across the held normal write: the outcome is uncertain (not Conflict),
/// confirmation proves RolledBack and a later plan binds `normal_return`.
async fn sc3_lock(provider: &str) {
    use crate::runtime::phase_jobs::NORMAL_WRITE;
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
    let writer = raw(&f);
    writer.execute_batch("BEGIN IMMEDIATE").unwrap();
    binding.0.release();
    wait_for(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.binding_error.is_some())
        },
        "SC3-L: held write did not return",
        60,
    )
    .await;
    let jobs = f.runtime.phase_jobs.observed_jobs();
    assert!(
        jobs[0]
            .binding_error
            .as_deref()
            .is_some_and(|e| e.contains("binding outcome uncertain")),
        "SC3-L {provider}: uncertain, not Conflict; jobs {jobs:?}"
    );
    assert!(
        links(&f, task).is_empty(),
        "SC3-L {provider}: nothing written"
    );
    writer.execute_batch("ROLLBACK").unwrap();
    drop(writer);
    // Design SC3-L ends at the confirmation: the held lock may also refuse the
    // live Core's own writes, so a later rebinding is accepted, not required.
    wait_for(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.binding_state == "rolled back" || j.bound)
        },
        "SC3-L: confirmation absent",
        60,
    )
    .await;
    assert!(
        f.runtime
            .phase_jobs
            .observed_success_turns()
            .iter()
            .any(|(_, l)| *l == "binding confirm"),
        "SC3-L {provider}: confirmed by the Root sweep"
    );
    let jobs = f.runtime.phase_jobs.observed_jobs();
    if jobs[0].bound {
        // Confirmed RolledBack, then a fresh normal plan bound while the
        // owner was still live.
        assert_eq!(binding_proof(&f, task), "normal_return", "SC3-L {provider}");
    } else {
        assert_eq!(
            jobs[0].binding_state, "rolled back",
            "SC3-L {provider}: RolledBack; jobs {jobs:?}"
        );
        assert!(links(&f, task).is_empty(), "SC3-L {provider}: zero links");
    }
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc3l_claude_writer_lock_is_uncertain_then_rolled_back() {
    sc3_lock("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc3l_codex_writer_lock_is_uncertain_then_rolled_back() {
    sc3_lock("codex").await;
}

/// SOL-M02: a sealed completion survives a refused observed plan and a typed
/// observed Conflict: the gate is evaluated once, the SAME completion is
/// planned again, and the chain closes with exactly one link of each kind.
async fn retained_completion(
    provider: &str,
    site: &'static str,
    fault: crate::state::managed_binding::fault::CommitFault,
) {
    use crate::runtime::phase_jobs::counted;
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
    release_completion(&f, task);
    wait_for(
        || links(&f, task).last().map(String::as_str) == Some("phase_closed"),
        "SC-R: chain did not close",
        90,
    )
    .await;
    assert!(
        !crate::state::managed_binding::fault::armed(task.id, site),
        "SETUP: SC-R {provider} {site} {fault:?}: the fault never fired"
    );
    assert_eq!(
        counted(task.id, "settled evaluation"),
        1,
        "SC-R {provider} {site} {fault:?}: evaluated once"
    );
    assert_eq!(
        links(&f, task),
        [
            "session_bound",
            "gate_claim",
            "gate_observed",
            "phase_closed"
        ],
        "SC-R {provider} {site} {fault:?}: one link of each kind"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
async fn retained_all(provider: &str) {
    use crate::state::managed_binding::fault::{CommitFault, OBSERVED, OBSERVED_PLAN};
    retained_completion(provider, OBSERVED_PLAN, CommitFault::BeforeCommit).await;
    retained_completion(provider, OBSERVED, CommitFault::Conflict).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn scr_claude_completion_survives_refused_plan_and_conflict() {
    retained_all("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn scr_codex_completion_survives_refused_plan_and_conflict() {
    retained_all("codex").await;
}

/// A second configured agent: the same protocol fixture as Codex.
fn codex_worker(config: &mut crate::config::Config) {
    let mut codex = config.agents["worker"].clone();
    let claude_script = std::path::PathBuf::from(&codex.command[0]);
    let codex_script = claude_script.with_file_name("configured-protocol-fixture-codex");
    let source = std::fs::read_to_string(&claude_script).unwrap();
    std::fs::write(
        &codex_script,
        source.replacen("PROVIDER=\"claude\"", "PROVIDER=\"codex\"", 1),
    )
    .unwrap();
    std::fs::set_permissions(&codex_script, std::fs::Permissions::from_mode(0o700)).unwrap();
    codex.provider = Some("codex".into());
    codex.command = vec![codex_script.to_string_lossy().into()];
    if let Some(compat) = codex.compatibility.as_mut() {
        compat.cli_version = "codex-cli 0.160.0".into();
    }
    config.agents.insert("worker-codex".into(), codex);
}
/// One accepted Task per new real Git Project, in distinct Projects.
async fn project_tasks(f: &mut ControlFixture, executors: &[&str]) -> Vec<Task> {
    let mut tasks = Vec::new();
    for (i, executor) in executors.iter().enumerate() {
        f.register_real_git_project_named(&format!("sc10-project-{i}"));
        let mut p = plan();
        p.tasks[0].risk = RiskClass::R1;
        p.tasks[0].reviewers = vec!["rev-a".into(), "rev-b".into()];
        p.tasks[0].executor = (*executor).into();
        let goal = f.create(p).await;
        let task = {
            let store = f.owner.store.lock().unwrap();
            let id = store.goal(goal).unwrap().unwrap().dag.nodes[0];
            store.task(id).unwrap().unwrap()
        };
        tasks.push(task);
    }
    let projects: std::collections::BTreeSet<_> = tasks.iter().map(|t| t.project_id).collect();
    assert_eq!(projects.len(), executors.len(), "SETUP: distinct Projects");
    tasks
}
/// SC10: four Tasks in four distinct Projects (two Claude, two Codex; one
/// active Task per Project) close independently in commit mode, with no
/// cross-job acknowledgment or Driver publication.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc10_four_projects_close_independently() {
    let mut f = fixture_mode("claude", true, None, codex_worker);
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let tasks = project_tasks(
        &mut f,
        &["worker", "worker-codex", "worker", "worker-codex"],
    )
    .await;
    crate::issue87_trace::mark("SC10 start");
    f.runtime.start().await.unwrap();
    for task in &tasks {
        wait_normal_bound(&f, task).await;
        crate::issue87_trace::mark(&format!("SC10 {} normal bound", task.id));
    }
    let marked: Vec<_> = tasks.iter().map(|t| stored_task(&f, t)).collect();
    for task in &tasks {
        release_completion(&f, task);
    }
    crate::issue87_trace::mark("SC10 completions released");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(120);
    for task in &tasks {
        let mut issue87_seen = Vec::new();
        while links(&f, task).last().map(String::as_str) != Some("phase_closed") {
            let issue87_links = links(&f, task);
            if issue87_links != issue87_seen {
                crate::issue87_trace::mark(&format!("SC10 {} links {issue87_links:?}", task.id));
                issue87_seen = issue87_links;
            }
            if tokio::time::Instant::now() > deadline {
                crate::issue87_trace::mark(&format!(
                    "SC10 timeout; turns={:?} shutdown={:?}",
                    f.runtime.phase_jobs.observed_success_turns(),
                    f.runtime.shutdown().await
                ));
                let (_, snapshot) = workflow(&f, task);
                panic!(
                    "SC10: Task {} did not close; links {:?}; attempt {:?}; jobs {:?}",
                    task.id,
                    links(&f, task),
                    snapshot.active.map(|i| (
                        snapshot.history[i].state.clone(),
                        snapshot.history[i].detail.clone()
                    )),
                    f.runtime.phase_jobs.observed_jobs()
                );
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
    crate::issue87_trace::mark("SC10 all closed");
    for (task, marked) in tasks.iter().zip(&marked) {
        assert_eq!(
            links(&f, task),
            [
                "session_bound",
                "gate_claim",
                "gate_observed",
                "phase_closed"
            ],
            "SC10: independent typed links"
        );
        assert_eq!(
            stored_task(&f, task).version,
            marked.version + 1,
            "SC10: bumped once"
        );
        // Each closure acknowledges only its own operation.
        let operations: i64 = raw(&f)
            .query_row(
                "SELECT count(DISTINCT json_extract(data,'$.private_operation_ref')) FROM audit WHERE task_id=?1 AND kind LIKE 'rrx.private.workflow.%'",
                [task.id.to_string()],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(operations, 1, "SC10: no cross-job link");
    }
    for task in &tasks {
        wait_for(
            || {
                f.owner
                    .store
                    .lock()
                    .unwrap()
                    .validate_task_driver(task.id)
                    .is_ok()
            },
            "SC10: own Driver publication",
            30,
        )
        .await;
    }
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

/// Releases only the external protocol peer's bootstrap hold (Codex
/// `initialize`); its completion stays held.
fn release_bootstrap(f: &ControlFixture, task: &Task) {
    let profile = crate::execution::resources::ResourceManager::new(f.owner.clone())
        .profile(&bound_unit(f, task))
        .unwrap();
    std::fs::write(profile.output.join("fixture-bootstrap-release"), "release").unwrap();
}
/// Exact provider-shared quota images (`quota_pools`, `quota_windows`).
fn shared_quota(f: &ControlFixture) -> Vec<String> {
    let raw = raw(f);
    let mut statement = raw
        .prepare(
            "SELECT 'pool',provider,next_probe_at,probe_unit,backoff,last_role FROM quota_pools \
             UNION ALL SELECT 'window',provider,bucket,observed_at,body,NULL FROM quota_windows \
             ORDER BY 1,2,3",
        )
        .unwrap();
    statement
        .query_map([], |r| {
            Ok((0..6)
                .map(|i| format!("{:?}", r.get::<_, rusqlite::types::Value>(i).unwrap()))
                .collect::<Vec<_>>()
                .join("|"))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}
/// SC10 C1: two Codex Tasks in two Projects. A's first live quota write is
/// held between its plan and apply while B, released, writes the
/// provider-shared quota window. A re-plans once from fresh images, its
/// Session is not Lost and both close.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc10_cross_project_quota_drift_replans_from_fresh_images() {
    use crate::runtime::phase_jobs::{QUOTA_APPLY, counted};
    let mut f = fixture_mode("claude", true, None, codex_worker);
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let tasks = project_tasks(&mut f, &["worker-codex", "worker-codex"]).await;
    let (a, b) = (&tasks[0], &tasks[1]);
    let hold_a = held(a, QUOTA_APPLY);
    let hold_b = held(b, QUOTA_APPLY);
    f.runtime.start().await.unwrap();
    for task in &tasks {
        wait_normal_bound(&f, task).await;
        release_bootstrap(&f, task);
    }
    wait_for(
        || hold_a.0.reached() && hold_b.0.reached(),
        "SETUP: both live quota plans not reached",
        60,
    )
    .await;
    let planned = shared_quota(&f);
    hold_b.0.release();
    wait_for(
        || shared_quota(&f) != planned,
        "SETUP: B wrote the shared quota images after A planned",
        60,
    )
    .await;
    hold_a.0.release();
    wait_for(
        || counted(a.id, "quota re-plan") > 0,
        "C1: A did not re-plan",
        60,
    )
    .await;
    for task in &tasks {
        release_completion(&f, task);
    }
    for task in &tasks {
        wait_for(
            || links(&f, task).last().map(String::as_str) == Some("phase_closed"),
            "C1: both Tasks close",
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
            "C1: independent typed links"
        );
    }
    assert_eq!(
        counted(a.id, "quota re-plan"),
        1,
        "C1: A re-planned once from fresh images"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
/// SC10 C2: the own active lease changes while the first live quota write is
/// held between plan and apply. The refusal is not shared drift: no re-plan,
/// the shared quota images are not written, and the Native owner ends with
/// no gate link.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc10_own_lease_change_refuses_without_replan() {
    use crate::runtime::phase_jobs::{QUOTA_APPLY, counted};
    let mut f = fixture_mode("claude", true, None, codex_worker);
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let tasks = project_tasks(&mut f, &["worker-codex"]).await;
    let task = &tasks[0];
    let hold = held(task, QUOTA_APPLY);
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    release_bootstrap(&f, task);
    wait_for(
        || hold.0.reached(),
        "SETUP: live quota plan not reached",
        60,
    )
    .await;
    let unit = f.runtime.phase_jobs.observed_jobs()[0].unit;
    let images = shared_quota(&f);
    // Fault injection on the own lease (a negative only; nothing is granted).
    assert_eq!(
        raw(&f)
            .execute(
                "UPDATE quota_leases SET epoch=epoch+1 WHERE unit_id=?1 AND active=1",
                [unit.to_string()],
            )
            .unwrap(),
        1,
        "SETUP: own active lease present"
    );
    hold.0.release();
    wait_for(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.owner_live == Some(false))
        },
        "C2: Native owner did not end after the refusal",
        60,
    )
    .await;
    assert_eq!(counted(task.id, "quota re-plan"), 0, "C2: no re-plan");
    assert_eq!(shared_quota(&f), images, "C2: shared images not written");
    assert_eq!(links(&f, task), ["session_bound"], "C2: no gate link");
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

/// SC9(a): shutdown takes the control admission and cancels the worker while
/// it is held before its closure; resumed, the worker's cancel-biased
/// `admit_success` returns cancelled and the closure never commits.
async fn sc9a_stop(provider: &str) {
    use crate::runtime::phase_jobs::SUCCESS_ADMISSION;
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    let closure = held(task, SUCCESS_ADMISSION);
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    let marked = stored_task(&f, task);
    release_completion(&f, task);
    wait_for(
        || closure.0.reached(),
        "SETUP: success admission not reached",
        90,
    )
    .await;
    let runtime = f.runtime.clone();
    let shutdown = tokio::spawn(async move { runtime.shutdown().await });
    // Let shutdown take the admission and cancel the worker's lifetime.
    wait_for(
        || f.runtime.stopping.load(std::sync::atomic::Ordering::SeqCst),
        "SETUP: shutdown did not begin",
        10,
    )
    .await;
    closure.0.release();
    let _ = shutdown.await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert_eq!(
        links(&f, task),
        ["session_bound", "gate_claim", "gate_observed"],
        "SC9(a) {provider}: no closure after shutdown"
    );
    assert_eq!(
        stored_task(&f, task).version,
        marked.version,
        "SC9(a) {provider}: Task unchanged"
    );
    assert_eq!(
        crate::runtime::phase_jobs::counted(task.id, "success admission cancelled"),
        1,
        "SC9(a) {provider}: the cancel-biased admission returned cancelled"
    );
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc9a_claude_shutdown_cancels_success_admission() {
    sc9a_stop("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc9a_codex_shutdown_cancels_success_admission() {
    sc9a_stop("codex").await;
}

fn effect_states(f: &ControlFixture, task: &Task) -> Vec<String> {
    let raw = raw(f);
    let mut statement = raw
        .prepare("SELECT state FROM managed_effects WHERE task_id=?1")
        .unwrap();
    statement
        .query_map([task.id.to_string()], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// SC11: a genuine protected Git helper of the settled capture is held in
/// flight for >= 3 fence ticks; the fence runs `validate_settled_helper` on
/// each tick, the helper completes, its effect is Confirmed and the chain
/// closes. Negative: a parent Goal edit while it is held makes the next tick
/// refuse: no claim, no closure.
async fn sc11_fence(provider: &str, parent_edit: bool) {
    use crate::runtime::phase_jobs::{HelperHold, counted};
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (goal, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    let hold = HelperHold::arm(task.id);
    struct Released(Arc<HelperHold>);
    impl Drop for Released {
        fn drop(&mut self) {
            self.0.release();
        }
    }
    let hold = Released(hold);
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    let marked = stored_task(&f, task);
    release_completion(&f, task);
    wait_for(|| hold.0.reached(), "SETUP: settled helper not held", 90).await;
    let base = counted(task.id, "settled fence tick");
    if parent_edit {
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
                    reason: "SC11 parent change".into(),
                }),
            )
            .await
            .unwrap();
        assert!(
            matches!(changed, ControlResponse::GoalLifecycleChanged { .. }),
            "SETUP: legitimate Goal writer refused: {changed:?}"
        );
        tokio::time::sleep(Duration::from_secs(2)).await;
        hold.0.release();
        tokio::time::sleep(Duration::from_secs(4)).await;
        assert_eq!(
            links(&f, task),
            ["session_bound"],
            "SC11 {provider}: no claim, no closure"
        );
        assert_eq!(
            stored_task(&f, task).version,
            marked.version,
            "SC11 {provider}: Task unchanged"
        );
        assert!(
            !effect_states(&f, task).iter().all(|s| s == "confirmed"),
            "SC11 {provider}: the refused helper's effect is not Confirmed"
        );
    } else {
        wait_for(
            || counted(task.id, "settled fence tick") >= base + 3,
            "SC11: fewer than three fence ticks while held",
            10,
        )
        .await;
        hold.0.release();
        wait_for(
            || links(&f, task).last().map(String::as_str) == Some("phase_closed"),
            "SC11: chain did not close",
            90,
        )
        .await;
        let states = effect_states(&f, task);
        assert!(
            !states.iter().any(|s| s == "unknown" || s == "pending"),
            "SC11 {provider}: helper effects settled; {states:?}"
        );
    }
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc11_claude_settled_helper_fence_holds_and_confirms() {
    sc11_fence("claude", false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc11_codex_settled_helper_fence_holds_and_confirms() {
    sc11_fence("codex", false).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc11n_claude_parent_edit_while_held_refuses_next_tick() {
    sc11_fence("claude", true).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc11n_codex_parent_edit_while_held_refuses_next_tick() {
    sc11_fence("codex", true).await;
}

/// SC5(d): the sweep runs while `completed` is held between the owner's
/// revoke and the settlement Weak: it records no conclusion (no continuation,
/// no closure); after release the continuation is installed and closes.
async fn sc5d_gap(provider: &str) {
    use crate::runtime::phase_jobs::SETTLEMENT_GAP;
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    let gap = held(task, SETTLEMENT_GAP);
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    release_completion(&f, task);
    wait_for(|| gap.0.reached(), "SETUP: settlement gap not reached", 90).await;
    let started = std::time::Instant::now();
    while started.elapsed() < Duration::from_secs(3) {
        f.runtime.wake.notify_one();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let jobs = f.runtime.phase_jobs.observed_jobs();
    assert!(
        jobs.iter().all(|j| !j.success && !j.success_closed),
        "SC5(d) {provider}: no conclusion in the gap; jobs {jobs:?}"
    );
    assert_eq!(
        links(&f, task),
        ["session_bound"],
        "SC5(d) {provider}: no link in the gap"
    );
    gap.0.release();
    wait_for(
        || links(&f, task).last().map(String::as_str) == Some("phase_closed"),
        "SC5(d): continuation not installed after the gap",
        90,
    )
    .await;
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc5d_claude_revoke_settle_gap_records_no_conclusion() {
    sc5d_gap("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc5d_codex_revoke_settle_gap_records_no_conclusion() {
    sc5d_gap("codex").await;
}

/// SC4-N: a normal binding whose commit is reported uncertain, then the
/// genuine owned terminal before the Root confirmation: confirmation is Known
/// (committed variant) or RolledBack (pre-commit variant) despite the
/// advanced Unit/invocation/Session/readiness/admission rows; the RolledBack
/// variant then late-binds once with `closed_settlement`.
async fn sc4n(provider: &str, fault: crate::state::managed_binding::fault::CommitFault) {
    use crate::runtime::phase_jobs::BINDING_CONFIRM;
    use crate::state::managed_binding::fault::{BIND, CommitFault, arm_commit_fault};
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    arm_commit_fault(task.id, BIND, fault);
    let confirm = held(task, BINDING_CONFIRM);
    f.runtime.start().await.unwrap();
    wait_for(
        || confirm.0.reached(),
        "SETUP: binding confirmation not reached",
        60,
    )
    .await;
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
        "SETUP: owned terminal before confirmation",
        60,
    )
    .await;
    confirm.0.release();
    wait_for(
        || links(&f, task).last().map(String::as_str) == Some("phase_closed"),
        "SC4-N: chain did not close",
        90,
    )
    .await;
    let expected = match fault {
        CommitFault::AfterCommit => "normal_return",
        _ => "closed_settlement",
    };
    assert_eq!(
        binding_proof(&f, task),
        expected,
        "SC4-N {provider} {fault:?}"
    );
    assert_eq!(
        links(&f, task),
        [
            "session_bound",
            "gate_claim",
            "gate_observed",
            "phase_closed"
        ],
        "SC4-N {provider} {fault:?}: one binding"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc4n_claude_uncertain_bind_then_terminal_confirms() {
    use crate::state::managed_binding::fault::CommitFault;
    sc4n("claude", CommitFault::AfterCommit).await;
    sc4n("claude", CommitFault::BeforeCommit).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc4n_codex_uncertain_bind_then_terminal_confirms() {
    use crate::state::managed_binding::fault::CommitFault;
    sc4n("codex", CommitFault::AfterCommit).await;
    sc4n("codex", CommitFault::BeforeCommit).await;
}

/// SC4-C: after a closure commit reported uncertain, a later Context version
/// of the same owner is inserted (negative-only raw stimulus) before the Root
/// confirmation: the postimage branch fails the latest-Context head relation
/// and the outcome stays Held, never Known (no publication, no release).
async fn sc4c(provider: &str) {
    use crate::runtime::phase_jobs::CLOSURE_CONFIRM;
    use crate::state::managed_binding::fault::{CLOSURE, CommitFault, arm_commit_fault};
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    arm_commit_fault(task.id, CLOSURE, CommitFault::AfterCommit);
    let confirm = held(task, CLOSURE_CONFIRM);
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    release_completion(&f, task);
    wait_for(
        || confirm.0.reached(),
        "SETUP: closure confirmation not reached",
        90,
    )
    .await;
    assert_eq!(
        links(&f, task).last().map(String::as_str),
        Some("phase_closed")
    );
    // Negative-only raw stimulus: one more Context version of the same owner.
    let writer = raw(&f);
    let (owner, version, body): (String, i64, String) = writer
        .query_row(
            "SELECT owner,version,body FROM context_versions WHERE task_id=?1 ORDER BY version DESC LIMIT 1",
            [task.id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    let mut next: serde_json::Value = serde_json::from_str(&body).unwrap();
    next["version"] = serde_json::json!(version + 1);
    writer
        .execute(
            "INSERT INTO context_versions(project_id,goal_id,task_id,owner,version,body) SELECT project_id,goal_id,task_id,owner,?2,?3 FROM context_versions WHERE task_id=?1 AND owner=?4 AND version=?5",
            rusqlite::params![task.id.to_string(), version + 1, next.to_string(), owner, version],
        )
        .unwrap();
    drop(writer);
    confirm.0.release();
    let started = std::time::Instant::now();
    while started.elapsed() < Duration::from_secs(6) {
        f.runtime.wake.notify_one();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let jobs = f.runtime.phase_jobs.observed_jobs();
    assert!(!jobs.is_empty(), "SC4-C {provider}: never released");
    assert!(
        jobs.iter().all(|j| !j.success_closed),
        "SC4-C {provider}: never Known; jobs {jobs:?}"
    );
    assert!(
        f.owner
            .store
            .lock()
            .unwrap()
            .validate_task_driver(task.id)
            .is_err(),
        "SC4-C {provider}: the Driver was not published"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc4c_claude_later_context_keeps_uncertain_closure_held() {
    sc4c("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc4c_codex_later_context_keeps_uncertain_closure_held() {
    sc4c("codex").await;
}

/// SC7 (peer killed): with the normal write held, the genuine fixture peer is
/// killed before any terminal: the owner ends without an owned success
/// settlement, the held normal write cannot bind an ended owner and no late
/// binding exists without a settlement: zero links, no continuation.
async fn sc7_killed(provider: &str) {
    use crate::runtime::phase_jobs::NORMAL_WRITE;
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
    // The genuine peer of THIS Unit: the process whose environment carries
    // this Unit's own output directory (tests run in parallel).
    let unit = f.runtime.phase_jobs.observed_jobs()[0].unit;
    let unit = f.owner.store.lock().unwrap().execution_unit(unit).unwrap();
    let output = crate::execution::resources::ResourceManager::new(f.owner.clone())
        .profile(&unit)
        .unwrap()
        .output;
    let needle = format!("RRX_OUTPUT_DIR={}", output.display());
    // BSD-style `ps axeww` appends each process's environment to its command
    // line on both Linux (procps) and macOS, so no procfs is required.
    let listing = std::process::Command::new("ps")
        .args(["axeww", "-o", "pid=,command="])
        .output()
        .unwrap();
    assert!(listing.status.success(), "SETUP: ps listing");
    let pid = String::from_utf8_lossy(&listing.stdout)
        .lines()
        .filter(|line| line.split_whitespace().any(|word| word == needle))
        .filter_map(|line| line.split_whitespace().next()?.parse::<u32>().ok())
        .find(|pid| *pid != std::process::id())
        .expect("SETUP: genuine peer process");
    let killed = std::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .status()
        .unwrap();
    assert!(killed.success(), "SETUP: kill the genuine peer");
    wait_for(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.owner_live == Some(false))
        },
        "SETUP: owner did not end after the peer was killed",
        60,
    )
    .await;
    binding.0.release();
    let started = std::time::Instant::now();
    while started.elapsed() < Duration::from_secs(6) {
        f.runtime.wake.notify_one();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let jobs = f.runtime.phase_jobs.observed_jobs();
    assert!(
        links(&f, task).is_empty(),
        "SC7-K {provider}: no binding; jobs {jobs:?}"
    );
    assert!(
        jobs.iter().all(|j| !j.success && j.settled != Some(true)),
        "SC7-K {provider}: no settlement, no continuation; jobs {jobs:?}"
    );
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc7k_claude_killed_peer_never_late_binds() {
    sc7_killed("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc7k_codex_killed_peer_never_late_binds() {
    sc7_killed("codex").await;
}

/// SC9(c): the closure is committed and Known but its Driver publication keeps
/// failing (fault-only); shutdown leaves it Held and counted pending (the job
/// is retained) and nothing revives it: the Driver is never published.
async fn sc9c(provider: &str) {
    use crate::state::managed_binding::fault::{CommitFault, PUBLISH, arm_commit_fault};
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    for _ in 0..1000 {
        arm_commit_fault(task.id, PUBLISH, CommitFault::BeforeCommit);
    }
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    release_completion(&f, task);
    wait_for(
        || links(&f, task).last().map(String::as_str) == Some("phase_closed"),
        "SC9(c): closure did not commit",
        90,
    )
    .await;
    // Root publication retries keep failing before shutdown.
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(
        f.owner
            .store
            .lock()
            .unwrap()
            .validate_task_driver(task.id)
            .is_err(),
        "SETUP: publication must still be pending"
    );
    let shutdown = f.runtime.shutdown().await;
    assert!(
        shutdown
            .as_ref()
            .is_err_and(|e| e.to_string().contains("pending")),
        "SC9(c) {provider}: shutdown counts the pending publication: {shutdown:?}"
    );
    assert!(
        !f.runtime.phase_jobs.observed_jobs().is_empty(),
        "SC9(c) {provider}: the job is retained, not released"
    );
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(
        f.owner
            .store
            .lock()
            .unwrap()
            .validate_task_driver(task.id)
            .is_err(),
        "SC9(c) {provider}: no revival after shutdown"
    );
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc9c_claude_pending_publication_stays_held_after_shutdown() {
    sc9c("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc9c_codex_pending_publication_stays_held_after_shutdown() {
    sc9c("codex").await;
}

/// SC9(b): shutdown takes the control admission first and joins the service
/// task while the Root holds a pending closure confirmation: the Root's
/// non-blocking admission returns None without awaiting, the service exits on
/// `stopping` and shutdown completes promptly; no confirmation is written.
async fn sc9b(provider: &str) {
    use crate::runtime::phase_jobs::{ROOT_ADMISSION, counted};
    use crate::state::managed_binding::fault::{CLOSURE, CommitFault, arm_commit_fault};
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    arm_commit_fault(task.id, CLOSURE, CommitFault::AfterCommit);
    let root = held(task, ROOT_ADMISSION);
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    release_completion(&f, task);
    wait_for(
        || root.0.reached(),
        "SETUP: Root closure action not reached",
        90,
    )
    .await;
    let runtime = f.runtime.clone();
    let shutdown = tokio::spawn(async move { runtime.shutdown().await });
    wait_for(
        || f.runtime.stopping.load(std::sync::atomic::Ordering::SeqCst),
        "SETUP: shutdown did not take the admission",
        10,
    )
    .await;
    let released = std::time::Instant::now();
    root.0.release();
    let _ = tokio::time::timeout(Duration::from_secs(8), shutdown)
        .await
        .expect("SC9(b): shutdown did not complete")
        .unwrap();
    assert!(
        released.elapsed() < Duration::from_secs(5),
        "SC9(b) {provider}: shutdown completed without the 5 s timeout ({:?})",
        released.elapsed()
    );
    assert_eq!(
        counted(task.id, "root admission refused"),
        1,
        "SC9(b) {provider}: the Root's try-admission returned None"
    );
    assert!(
        f.runtime
            .phase_jobs
            .observed_jobs()
            .iter()
            .all(|j| !j.success_closed),
        "SC9(b) {provider}: no confirmation was written"
    );
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc9b_claude_shutdown_preempts_root_success_action() {
    sc9b("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc9b_codex_shutdown_preempts_root_success_action() {
    sc9b("codex").await;
}

/// SC9(d): shutdown's admission acquisition cannot interleave inside an
/// admitted closure turn: while the worker holds its success admission,
/// shutdown waits; the closure commit and its Driver publication are then
/// observed together.
async fn sc9d(provider: &str) {
    use crate::runtime::phase_jobs::SUCCESS_ADMITTED;
    let mut f = fixture_mode(provider, true, None, |_| {});
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    let admitted = held(task, SUCCESS_ADMITTED);
    f.runtime.start().await.unwrap();
    wait_normal_bound(&f, task).await;
    release_completion(&f, task);
    wait_for(
        || admitted.0.reached(),
        "SETUP: admitted closure turn not reached",
        90,
    )
    .await;
    let runtime = f.runtime.clone();
    let shutdown = tokio::spawn(async move { runtime.shutdown().await });
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(
        !f.runtime.stopping.load(std::sync::atomic::Ordering::SeqCst),
        "SC9(d) {provider}: shutdown waits for the admitted closure turn"
    );
    assert!(
        links(&f, task).last().map(String::as_str) != Some("phase_closed"),
        "SETUP: closure not yet committed"
    );
    admitted.0.release();
    let _ = tokio::time::timeout(Duration::from_secs(8), shutdown)
        .await
        .expect("SC9(d): shutdown did not complete")
        .unwrap();
    let closed = links(&f, task).last().map(String::as_str) == Some("phase_closed");
    // The registry is stopped after shutdown, so publication is observed by
    // its own cfg(test) counter rather than by a later cache read.
    let published = crate::runtime::phase_jobs::counted(task.id, "success published") == 1;
    assert!(
        closed && published,
        "SC9(d) {provider}: commit {closed} and publication {published} together"
    );
    finish(f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc9d_claude_shutdown_cannot_interleave_admitted_closure() {
    sc9d("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sc9d_codex_shutdown_cannot_interleave_admitted_closure() {
    sc9d("codex").await;
}
