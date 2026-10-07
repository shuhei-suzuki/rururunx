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
    let unit = bound_unit(f, task);
    let profile = crate::execution::resources::ResourceManager::new(f.owner.clone())
        .profile(&unit)
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
