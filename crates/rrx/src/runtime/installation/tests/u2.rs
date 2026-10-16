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
    assert_no_execution_effects(f);
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
    let guard = f.runtime.control_admission.lock().await;
    // U3 retains provider acceptance while the new S boundary closes composition.
    let graph = f.runtime.installed.as_ref().ok().unwrap();
    for alias in &task.reviewers {
        assert_eq!(
            graph.registry.native_phase_port(alias).unwrap().provider(),
            "claude"
        );
    }
    let refusal = f
        .runtime
        .installed_driver_composition(&task)
        .err()
        .expect("U3 composition bypassed startup gate");
    assert!(refusal.to_string().contains("startup"));
    assert!(refusal.downcast_ref::<CapabilityRefusal>().is_none());
    let response = f
        .runtime
        .handle_control(&f.socket, f.request(ControlAction::RuntimeStatus))
        .await
        .unwrap();
    assert!(matches!(
        response,
        ControlResponse::RuntimeMetadata {
            admission_refusal: None,
            ..
        }
    ));
    assert_no_effects(&f);
    drop(guard);
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}

/// Saved Tasks follow the current alias provider after an actual owner restart.
#[tokio::test]
async fn saved_task_provider_refusal_preserves_finite_classification_before_composition() {
    let f = configured();
    let (_, tasks) = accept(&f, 1).await;
    let f = f
        .restart_with_config(|config| {
            config.agents.get_mut("worker").unwrap().provider = Some("claude".into());
        })
        .await;
    f.runtime.start().await.unwrap();
    let guard = f.runtime.control_admission.lock().await;
    let task = f
        .owner
        .store
        .lock()
        .unwrap()
        .task(tasks[0].id)
        .unwrap()
        .unwrap();
    assert_eq!(task.executor, "worker");
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
    drop(guard);
    assert_eq!(f.runtime.admit_ready_tasks().unwrap(), 0);
    let response = f
        .runtime
        .handle_control(&f.socket, f.request(ControlAction::RuntimeStatus))
        .await
        .unwrap();
    let wire = serde_json::to_vec(&response).unwrap();
    let decoded: ControlResponse = serde_json::from_slice(&wire).unwrap();
    assert!(
        matches!(
            decoded,
            ControlResponse::RuntimeMetadata {
                admission_refusal: Some(AdmissionRefusal::Capability(
                    CapabilityRefusal::ClaudeExecutorUnavailable
                )),
                ..
            }
        ),
        "saved Task reason missing: {decoded:?}"
    );
    assert_no_effects(&f);
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}

/// A removed Executor alias skips saved Tasks without a capability refusal or effects.
#[tokio::test]
async fn saved_task_removed_executor_alias_is_not_a_capability_refusal() {
    let f = configured();
    accept(&f, 1).await;
    let f = f
        .restart_with_config(|config| {
            config.agents.remove("worker");
        })
        .await;
    f.runtime.start().await.unwrap();
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
    let result = f
        .runtime
        .evaluate_driver_candidate(&guard, &keys[0])
        .unwrap();
    assert!(
        matches!(&result, AdmitOutcome::Skipped(SkipReason::Composition)),
        "unexpected candidate result: {result:?}"
    );
    drop(guard);
    assert_no_effects(&f);
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}

/// Only a configured Codex provider can select the Executor role.
#[tokio::test]
async fn executor_provider_allowlist_refuses_missing_and_unknown_provider() {
    for provider in [None, Some("unknown-provider")] {
        let error = crate::execution::capability::require_executor(provider)
            .expect_err("unapproved Executor provider accepted");
        assert_eq!(error, CapabilityRefusal::ExecutorProviderUnavailable);
    }
}

/// The real private socket service preserves a finite Goal rejection on the wire.
#[tokio::test]
async fn goal_capability_refusal_round_trips_through_socket_service() {
    let f = configured();
    let endpoint = crate::cli::endpoint::ControlEndpoint::bind(f.owner.clone()).unwrap();
    let runtime = f.runtime.clone();
    let service = tokio::spawn(crate::cli::service::serve_runtime(runtime, endpoint));
    let mut p = plan();
    p.tasks[0].executor = "review-model".into();
    let action = ControlAction::CreateGoal {
        project: f.project.id,
        expected_project: f.project.version,
        plan: p,
    };
    let (identity, mut connection) = crate::cli::endpoint::connect(&f.state_path())
        .await
        .unwrap();
    let request_id = uuid::Uuid::new_v4();
    let request = ControlRequest {
        request_id,
        instance: identity.instance,
        epoch: identity.epoch,
        action: action.clone(),
    };
    crate::cli::transport::send(
        connection.get_mut(),
        &request,
        crate::cli::transport::REQUEST_BYTES,
    )
    .await
    .unwrap();
    let response: ControlResponse =
        crate::cli::transport::receive(&mut connection, crate::cli::transport::RESPONSE_BYTES)
            .await
            .unwrap();
    assert!(
        matches!(response, ControlResponse::CapabilityRejected { reason: CapabilityRefusal::ClaudeExecutorUnavailable, request_id: id } if id == request_id),
        "wire capability reason missing: {response:?}"
    );
    let error = crate::cli::client::request(&f.state_path(), action)
        .await
        .expect_err("client accepted capability rejection");
    assert_eq!(
        error.downcast_ref::<CapabilityRefusal>(),
        Some(&CapabilityRefusal::ClaudeExecutorUnavailable)
    );
    assert_eq!(count(&f, "goals"), 0);
    assert_eq!(count(&f, "tasks"), 0);
    assert_no_effects(&f);
    assert!(matches!(
        crate::cli::client::request(&f.state_path(), ControlAction::RuntimeStop)
            .await
            .unwrap(),
        ControlResponse::RuntimeStopped { .. }
    ));
    service.await.unwrap().unwrap();
    finish(f).await;
}

/// C-1: installation refusal stays typed while status and stop remain usable.
#[tokio::test]
async fn installation_refusal_preserves_status_and_stop() {
    let f = configured_with(false);
    let (_, tasks) = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    assert_eq!(f.runtime.admit_ready_tasks().unwrap(), 0);
    let error = f
        .runtime
        .installed_driver_composition(&tasks[0])
        .err()
        .unwrap();
    assert_eq!(
        error.downcast_ref::<InstallationRefusal>(),
        Some(&InstallationRefusal::Registry)
    );
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
    assert!(matches!(
        outcome,
        AdmitOutcome::Skipped(SkipReason::Installation(InstallationRefusal::Registry))
    ));
    drop(guard);
    let response = f
        .runtime
        .handle_control(&f.socket, f.request(ControlAction::RuntimeStatus))
        .await
        .unwrap();
    let wire = serde_json::to_vec(&response).unwrap();
    let decoded: ControlResponse = serde_json::from_slice(&wire).unwrap();
    assert!(matches!(
        decoded,
        ControlResponse::RuntimeMetadata {
            admission_refusal: Some(AdmissionRefusal::Installation(
                InstallationRefusal::Registry
            )),
            service_running: true,
            ..
        }
    ));
    assert_no_effects(&f);
    f.runtime.shutdown().await.unwrap();
    assert!(!f.runtime.service_running());
    assert_no_effects(&f);
    finish(f).await;
}

/// EX-1: handoff rejects a Claude port before consuming a real Driver input view.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn native_handoff_refuses_claude_provider_before_ticket_and_asset_transfer() {
    let mut f = configured();
    f.register_real_git_project();
    let graph = f.runtime.installed.as_ref().ok().unwrap();
    let sources = graph.sources.clone();
    let selected = graph.registry.native_phase_port("review-model").unwrap();
    let owner = f.owner.clone();
    let (sent, received) = tokio::sync::oneshot::channel();
    let sent = Arc::new(std::sync::Mutex::new(Some(sent)));
    graph.engine.set_activation_hooks(
        Some(Arc::new(move |probe| {
            // All inputs and the association come from the actual production
            // activation. No constructor, copied row or seed supplies authority.
            let ticket = crate::state::read_driver_ticket(
                owner.clone(),
                probe.lifetime().association().unwrap(),
            )
            .unwrap();
            let sources = sources.clone();
            let selected = selected.clone();
            let owner = owner.clone();
            let sent = sent.clone();
            Box::pin(async move {
                let effects = || {
                    crate::state::current_test_writer(owner.state_path())
                        .unwrap()
                        .query_row("SELECT count(*) FROM managed_effects", [], |r| {
                            r.get::<_, usize>(0)
                        })
                        .unwrap()
                };
                let before = effects();
                let error = sources
                    .offer_first_executor(
                        probe.task(),
                        probe.record(),
                        probe.context(),
                        ticket,
                        selected,
                    )
                    .await
                    .err()
                    .expect("Claude handoff accepted");
                assert_eq!(
                    error.downcast_ref::<CapabilityRefusal>(),
                    Some(&CapabilityRefusal::ClaudeExecutorUnavailable)
                );
                assert_eq!(effects(), before);
                sent.lock().unwrap().take().unwrap().send(()).unwrap();
            })
        })),
        None,
    );
    let (_, tasks) = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    tokio::time::timeout(Duration::from_secs(40), received)
        .await
        .expect("activation not reached")
        .unwrap();
    success::wait_normal_bound(&f, &tasks[0]).await;
    success::release_completion(&f, &tasks[0]);
    finish(f).await;
}
