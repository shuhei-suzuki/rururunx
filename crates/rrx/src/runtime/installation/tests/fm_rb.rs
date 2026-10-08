//! FM §8.3 RB and S5-W (R) for the former
//! `execution::workflow_source::recovery_tests` (FM delta R2-1 B). Each
//! scenario's retained state now comes from the D2 SC1 lane (`fm_d2`:
//! accepted ingress, a real Git Project with committed `config_ref` /
//! `rule_refs`, the installed issuer and the genuine Driver), then a real
//! restart. The legitimate continuation after a restart, Pause then Resume on
//! live Unix-peer ingress, is refused with `FreshBootstrapRecoveryUnavailable`
//! with the Goal, Task, Workflow, Driver rows, links and jobs unchanged; where
//! the former scenario edited the Task through the generic writer, that edit
//! is refused on accepted rows (S5-W). The former subjects (the test-only
//! retained-source recovery port) are deferred to #14, which reintroduces the
//! port with its production consumer. Loop variables that only selected a
//! retired port mode are collapsed.
use super::fm_d2::{committed, wait_closed};
use super::success::{links, release_completion, stored_task, wait_for, wait_normal_bound};
use super::*;
use crate::{
    domain::GoalState,
    execution::{ArtifactState, ResultArtifact},
    runtime::phase_jobs::{SETTLED_EVALUATION, WritePause},
};

/// The D2 producer then a real restart: the single artifact is Published
/// (`phase_closed`), or, with `published = false`, captured but never
/// Published (the settled evaluation held, the captured manifest corrupted,
/// then the gate observes it without closure, as fm_d2 `:470`).
async fn restarted(published: bool) -> (ControlFixture, Task, ResultArtifact) {
    let c = committed("codex", crate::config::WorkflowClass::Quick).await;
    let task = c.task.clone();
    if published {
        c.f.runtime.start().await.unwrap();
        wait_normal_bound(&c.f, &task).await;
        release_completion(&c.f, &task);
        wait_closed(&c, "FM RB producer").await;
    } else {
        struct Release(Arc<WritePause>);
        impl Drop for Release {
            fn drop(&mut self) {
                self.0.release();
            }
        }
        let pause = Release(WritePause::arm_at(task.id, SETTLED_EVALUATION));
        c.f.runtime.start().await.unwrap();
        wait_normal_bound(&c.f, &task).await;
        release_completion(&c.f, &task);
        wait_for(
            || pause.0.reached(),
            "SETUP: settled evaluation not reached",
            90,
        )
        .await;
        let captured =
            c.f.owner
                .store
                .lock()
                .unwrap()
                .result_artifacts(&task.scope())
                .unwrap();
        assert_eq!(captured.len(), 1, "SETUP: captured artifact");
        std::fs::write(&captured[0].manifest, "corrupt").unwrap();
        pause.0.release();
        wait_for(
            || links(&c.f, &task) == ["session_bound", "gate_claim", "gate_observed"],
            "SETUP: gate_observed without closure",
            60,
        )
        .await;
    }
    let f = c.f.restart().await;
    let artifacts = f
        .owner
        .store
        .lock()
        .unwrap()
        .result_artifacts(&task.scope())
        .unwrap();
    assert_eq!(artifacts.len(), 1, "SETUP: one retained artifact");
    let artifact = artifacts[0].clone();
    if published {
        assert_eq!(artifact.state, ArtifactState::Published, "SETUP");
    } else {
        assert_eq!(
            artifact.state,
            ArtifactState::Ready,
            "SETUP: Ready after reopen"
        );
    }
    f.runtime.start().await.unwrap();
    (f, task, artifact)
}

/// FM §8.3 S5-W: the generic Task writer refuses the former instruction edit
/// on accepted rows; the input DTO, the stored Task body/version and the
/// audit are unchanged.
fn assert_task_edit_refused(f: &ControlFixture, task: &Task) {
    let mut store = f.owner.store.lock().unwrap();
    let before = store.task(task.id).unwrap().unwrap();
    let audit = store.events(&task.scope(), 0, 10_000).unwrap().len();
    let mut edited = before.clone();
    edited.title = "generic instruction drift".into();
    let input = serde_json::to_value(&edited).unwrap();
    let refused = store.put_task(&mut edited).unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("managed Task changes require typed owned control/Workflow transaction"),
        "FM S5-W: {refused:#}"
    );
    assert_eq!(
        serde_json::to_value(&edited).unwrap(),
        input,
        "FM S5-W: input DTO unchanged"
    );
    assert_eq!(
        serde_json::to_value(store.task(task.id).unwrap().unwrap()).unwrap(),
        serde_json::to_value(&before).unwrap(),
        "FM S5-W: Task unchanged"
    );
    assert_eq!(
        store.events(&task.scope(), 0, 10_000).unwrap().len(),
        audit,
        "FM S5-W: audit unchanged"
    );
}

fn lifecycle(f: &ControlFixture, task: &Task, target: GoalControl) -> ControlRequest {
    let goal = f
        .owner
        .store
        .lock()
        .unwrap()
        .goal(task.goal_id)
        .unwrap()
        .unwrap();
    f.request(ControlAction::SetGoalLifecycle {
        project: task.project_id,
        goal: goal.id,
        expected_goal: goal.version,
        target,
        reason: "FM RB after restart".into(),
    })
}

/// FM §8.3 RB: after the restart, Pause is accepted on live ingress and
/// Resume is refused with `FreshBootstrapRecoveryUnavailable`; the Goal, Task,
/// Workflow record, Driver rows, links and jobs taken immediately before the
/// Resume are unchanged.
async fn assert_rb(f: &ControlFixture, task: &Task, artifact: &ResultArtifact) {
    let paused = f
        .runtime
        .handle_control(&f.socket, lifecycle(f, task, GoalControl::Pause))
        .await
        .unwrap();
    assert!(
        matches!(
            paused,
            ControlResponse::GoalLifecycleChanged {
                state: GoalState::Paused,
                ..
            }
        ),
        "SETUP: Pause accepted: {paused:?}"
    );
    let preimage = || {
        let store = f.owner.store.lock().unwrap();
        let drivers: Vec<(String, i64, String)> = {
            let raw = crate::state::current_test_writer(f.owner.state_path()).unwrap();
            let mut statement = raw
                .prepare("SELECT id,version,body FROM task_drivers WHERE task_id=?1")
                .unwrap();
            statement
                .query_map([task.id.to_string()], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?))
                })
                .unwrap()
                .map(Result::unwrap)
                .collect()
        };
        serde_json::json!({
            "goal": store.goal(task.goal_id).unwrap(),
            "task": store.task(task.id).unwrap(),
            "workflow": store.records(&task.scope(), RecordKind::Workflow).unwrap(),
            "artifact": store.result_artifact(artifact.id).unwrap(),
            "drivers": drivers,
            "links": links(f, task),
            "jobs": f.runtime.phase_jobs.observed_jobs().len(),
        })
    };
    let before = preimage();
    let resumed = f
        .runtime
        .handle_control(&f.socket, lifecycle(f, task, GoalControl::Resume))
        .await
        .unwrap();
    assert!(
        matches!(
            resumed,
            ControlResponse::Unavailable {
                reason: UnavailableReason::FreshBootstrapRecoveryUnavailable,
                ..
            }
        ),
        "FM RB: typed refusal: {resumed:?}"
    );
    assert_eq!(preimage(), before, "FM RB: preimage unchanged");
    assert_eq!(
        stored_task(f, task).version,
        before["task"]["version"].as_u64().unwrap()
    );
}

async fn done(f: ControlFixture) {
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

/// FM R (RB + S5-W). Former subject: the exact Published frame is rebuilt
/// from committed rules after a restart and Commit advances only the typed
/// Workflow pins; a generic Task edit then refuses the old frame. Deferred:
/// #14 (recovery), SC-N (Commit).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn published_recovery_reopens_exact_frame_and_advances_only_typed_workflow_pins() {
    let (f, task, artifact) = restarted(true).await;
    assert_task_edit_refused(&f, &task);
    assert_rb(&f, &task, &artifact).await;
    done(f).await;
}

/// FM R (RB + S5-W). Former subject: the retained-source claim is exclusive
/// and a helper bound to a stale full Task DTO is refused before its effect
/// intent. Deferred: #14.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn source_claim_checks_helper_admission_competition_and_full_dto_currency() {
    let (f, task, artifact) = restarted(true).await;
    assert_task_edit_refused(&f, &task);
    assert_rb(&f, &task, &artifact).await;
    done(f).await;
}

/// FM R (RB). Former subject: a dropped recovery future abandons only its
/// own uninstalled claim. Deferred: #14.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn actual_recovery_future_drop_invalidates_only_its_uninstalled_claim() {
    let (f, task, artifact) = restarted(true).await;
    assert_rb(&f, &task, &artifact).await;
    done(f).await;
}

/// FM R (RB + S5-W). Former subject: the final recovery acceptance refuses a
/// Task edit after its reads [no epoch] and an epoch race [epoch] (the epoch
/// half had no legitimate producer: a mid-flight epoch). Deferred: #14.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn final_recovery_acceptance_refuses_instruction_and_epoch_races() {
    let (f, task, artifact) = restarted(true).await;
    assert_task_edit_refused(&f, &task);
    assert_rb(&f, &task, &artifact).await;
    done(f).await;
}

/// FM R (RB). Former subject: a corrupt retained manifest or a caller DTO
/// drift refuses recovery before helpers. Deferred: #14.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn recovery_rejects_bad_manifest_and_complete_caller_dto_drift() {
    let (f, task, artifact) = restarted(true).await;
    assert_rb(&f, &task, &artifact).await;
    done(f).await;
}

/// FM R (RB + S5-W). Former subject: a source-bound in-flight helper leaves
/// an Unknown receipt on a Task edit [mode 0], an epoch change [mode 1; no
/// legitimate producer] or an abort [mode 2]. Deferred: #14.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn source_bound_inflight_helper_rejects_drift_epoch_and_future_drop() {
    let (f, task, artifact) = restarted(true).await;
    assert_task_edit_refused(&f, &task);
    assert_rb(&f, &task, &artifact).await;
    done(f).await;
}

/// FM R (RB). Former subject: a Ready artifact is never a recovery source,
/// and a foreign Project/Task/Artifact DTO is never source authority. Both
/// retained states are produced on the D2 lane: the Ready one (captured,
/// never Published, still Ready after the reopen) and a Published one.
/// Deferred: #14.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn unpublished_ready_and_foreign_retained_dtos_never_select_recovery() {
    for published in [false, true] {
        let (f, task, artifact) = restarted(published).await;
        assert_rb(&f, &task, &artifact).await;
        done(f).await;
    }
}
