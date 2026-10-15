//! EX-1/AC-04/08 and C-1: real ingress and saved-row consumers, no grant seeds.
use super::*;
use crate::{
    execution::capability::CapabilityRefusal,
    runtime::task_driver::{AdmitOutcome, SkipReason},
};

fn configured() -> ControlFixture {
    configured_with(true)
}
fn configured_with(available: bool) -> ControlFixture {
    fixture_with("codex", true, |config| {
        let executor = config.agents["worker"].clone();
        let mut reviewer = executor.clone();
        reviewer.provider = Some("claude".into());
        // Deliberately misleading names exercise provider identity, not spelling.
        config.agents.insert("claude".into(), executor);
        config.agents.insert("codex".into(), reviewer.clone());
        config.agents.insert("review-model".into(), reviewer);
        if !available {
            config.agents.get_mut("worker").unwrap().command =
                vec!["/missing-u2-executable".into()];
        }
    })
}
fn assert_no_effects(f: &ControlFixture) {
    for table in [
        "task_drivers",
        "execution_units",
        "managed_effects",
        "native_invocations",
        "quota_leases",
    ] {
        assert_eq!(
            count(f, table),
            0,
            "AC-04/08: {table} created before refusal"
        );
    }
    assert_eq!(
        raw(f)
            .query_row(
                "SELECT count(*) FROM records WHERE kind='session'",
                [],
                |r| r.get::<_, usize>(0)
            )
            .unwrap(),
        0
    );
}

/// EX-1: role selection uses the configured provider before Goal persistence.
#[tokio::test]
async fn goal_refuses_claude_provider_before_save_and_accepts_same_alias_as_reviewer() {
    let f = configured();
    for alias in ["codex", "review-model"] {
        let mut p = plan();
        p.tasks[0].executor = alias.into();
        let refusal = f
            .runtime
            .handle_control(
                &f.socket,
                f.request(ControlAction::CreateGoal {
                    project: f.project.id,
                    expected_project: f.project.version,
                    plan: p,
                }),
            )
            .await
            .expect_err("Claude Executor Goal accepted");
        assert_eq!(
            refusal.downcast_ref::<CapabilityRefusal>(),
            Some(&CapabilityRefusal::ClaudeExecutorUnavailable)
        );
        assert_eq!(count(&f, "goals"), 0);
        assert_eq!(count(&f, "tasks"), 0);
        assert_no_effects(&f);
    }
    let mut p = plan();
    p.tasks[0].executor = "claude".into();
    p.tasks[0].reviewers = vec!["codex".into(), "review-model".into()];
    let goal = f.create(p).await;
    let task = {
        let store = f.owner.store.lock().unwrap();
        store
            .task(store.goal(goal).unwrap().unwrap().dag.nodes[0])
            .unwrap()
            .unwrap()
    };
    f.runtime.start().await.unwrap();
    // Reviewer acceptance is tested by the real composition, not only parsing.
    assert!(f.runtime.installed_driver_composition(&task).is_ok());
    assert_no_effects(&f);
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}

/// AC-04: saved Tasks are checked before composition, Unit, Held and Admit.
#[tokio::test]
async fn saved_task_provider_refusal_preserves_finite_classification_before_composition() {
    for available in [true, false] {
        let f = configured_with(available);
        let (_, tasks) = accept(&f, 1).await;
        f.runtime.start().await.unwrap();
        let guard = f.runtime.control_admission.lock().await;
        for alias in ["codex", "review-model"] {
            let mut task = f
                .owner
                .store
                .lock()
                .unwrap()
                .task(tasks[0].id)
                .unwrap()
                .unwrap();
            task.executor = alias.into();
            // Negative saved-row corruption only: never a success/authority seed.
            raw(&f)
                .execute(
                    "UPDATE tasks SET body=json_set(body,'$.executor',?1) WHERE id=?2",
                    [alias, &task.id.to_string()],
                )
                .unwrap();
            let error = f
                .runtime
                .installed_driver_composition(&task)
                .err()
                .unwrap_or_else(|| panic!("Claude composition accepted"));
            assert_eq!(
                error.downcast_ref::<CapabilityRefusal>(),
                Some(&CapabilityRefusal::ClaudeExecutorUnavailable)
            );
            let page = f
                .owner
                .store
                .lock()
                .unwrap()
                .ready_driver_candidates(f.owner.instance_id(), f.owner.epoch(), None, 8)
                .unwrap();
            let crate::state::CandidatePage::Rows { keys, .. } = page else {
                panic!("no candidate")
            };
            let result = f
                .runtime
                .evaluate_driver_candidate(&guard, &keys[0])
                .unwrap();
            assert!(
                matches!(
                    result,
                    AdmitOutcome::Skipped(SkipReason::Capability(
                        CapabilityRefusal::ClaudeExecutorUnavailable
                    ))
                ),
                "unexpected candidate result: {result:?}"
            );
            assert_no_effects(&f);
        }
        drop(guard);
        f.runtime.shutdown().await.unwrap();
        finish(f).await;
    }
}

/// C-1: missing capability and installation failures remain distinct typed facts.
#[tokio::test]
async fn capability_chain_stops_normal_admission_while_status_and_stop_work() {
    for installed in [true, false] {
        let f = fixture_with("codex", true, |config| {
            if !installed {
                config.agents.get_mut("worker").unwrap().command =
                    vec!["/missing-u2-executable".into()];
            }
        });
        let (_, tasks) = accept(&f, 1).await;
        f.runtime.start().await.unwrap();
        assert_eq!(f.runtime.admit_ready_tasks().unwrap(), 0);
        if !installed {
            let error = f
                .runtime
                .installed_driver_composition(&tasks[0])
                .err()
                .unwrap();
            assert_eq!(
                error.downcast_ref::<InstallationRefusal>(),
                Some(&InstallationRefusal::Registry)
            );
        }
        let guard = f.runtime.control_admission.lock().await;
        let page = f
            .owner
            .store
            .lock()
            .unwrap()
            .ready_driver_candidates(f.owner.instance_id(), f.owner.epoch(), None, 8)
            .unwrap();
        let crate::state::CandidatePage::Rows { keys, .. } = page else {
            panic!("no candidate")
        };
        let outcome = f
            .runtime
            .evaluate_driver_candidate(&guard, &keys[0])
            .unwrap();
        let expected = if installed {
            assert!(
                matches!(
                    outcome,
                    AdmitOutcome::Skipped(SkipReason::Capability(
                        CapabilityRefusal::M0CapabilityChainUnavailable
                    ))
                ),
                "unexpected candidate outcome: {outcome:?}"
            );
            AdmissionRefusal::Capability(CapabilityRefusal::M0CapabilityChainUnavailable)
        } else {
            assert!(matches!(
                outcome,
                AdmitOutcome::Skipped(SkipReason::Installation(InstallationRefusal::Registry))
            ));
            AdmissionRefusal::Installation(InstallationRefusal::Registry)
        };
        drop(guard);
        let response = f
            .runtime
            .handle_control(&f.socket, f.request(ControlAction::RuntimeStatus))
            .await
            .unwrap();
        let wire = serde_json::to_vec(&response).unwrap();
        let decoded: ControlResponse = serde_json::from_slice(&wire).unwrap();
        assert!(
            matches!(decoded, ControlResponse::RuntimeMetadata { admission_refusal, service_running: true, .. } if admission_refusal == expected)
        );
        assert_no_effects(&f);
        f.runtime.shutdown().await.unwrap();
        assert!(!f.runtime.service_running());
        assert_no_effects(&f);
        finish(f).await;
    }
}
