//! Real Runtime/Unix ingress/Source controls. Only the external CLI is a local
//! account-free protocol fixture; these never qualify official Native Agents.
use super::*;
use crate::{
    config::{AgentConfig, NativeCompatConfig},
    domain::{GoalId, RecordKind, RiskClass},
    runtime::{
        control::*,
        tests::{ControlFixture, plan},
    },
};
use std::{os::unix::fs::PermissionsExt, time::Duration};

mod sweep;

fn fixture(provider: &str, declared: bool) -> ControlFixture {
    ControlFixture::configured(|dir| {
        let path = dir.join("configured-protocol-fixture");
        let source = include_str!("../../execution/native/native_fixture.py");
        // Hold only the external protocol peer's bootstrap, after a real spawn.
        // This is not an rrx admission/authority or permission switch.
        let source=source.replacen("while True: time.sleep(0.02)","while not os.path.exists(os.path.join(os.environ[\"RRX_OUTPUT_DIR\"], \"fixture-bootstrap-release\")): time.sleep(0.02)",1);
        std::fs::write(&path,format!("#!/usr/bin/python3\nPROVIDER={provider:?}\nBOOTSTRAP_HOLD=True\nWORKFLOW_SCENARIO='answer-normal'\n{source}")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut config = crate::config::Config {
            minimum_workflow: crate::config::WorkflowClass::Quick,
            ..Default::default()
        };
        config.workflow.risk_mapping = [crate::config::WorkflowClass::Quick; 4];
        config.agents.insert(
            "worker".into(),
            AgentConfig {
                provider: Some(provider.into()),
                command: vec![path.to_string_lossy().into()],
                compatibility: declared.then(|| NativeCompatConfig {
                    profile: "rrx-native-inherited-v1".into(),
                    cli_version: if provider == "claude" {
                        "2.1.283"
                    } else {
                        "codex-cli 0.160.0"
                    }
                    .into(),
                    settings: "inherited".into(),
                    user_hooks: vec![],
                }),
                ..Default::default()
            },
        );
        config
    })
}
async fn accept(f: &ControlFixture, count: usize) -> (GoalId, Vec<Task>) {
    let mut p = plan();
    p.tasks[0].risk = RiskClass::R1;
    let original = p.tasks[0].clone();
    p.tasks = (0..count)
        .map(|i| {
            let mut task = original.clone();
            task.key = format!("task-{i}");
            task.title = format!("work-{i}");
            task
        })
        .collect();
    let goal = f.create(p).await;
    let store = f.owner.store.lock().unwrap();
    let tasks = store
        .goal(goal)
        .unwrap()
        .unwrap()
        .dag
        .nodes
        .iter()
        .map(|id| store.task(*id).unwrap().unwrap())
        .collect();
    (goal, tasks)
}
fn raw(f: &ControlFixture) -> rusqlite::Connection {
    crate::state::current_test_writer(f.owner.state_path()).unwrap()
}
fn count(f: &ControlFixture, table: &str) -> usize {
    raw(f)
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
async fn finish(f: ControlFixture) {
    let weak = Arc::downgrade(&f.runtime);
    let engine = f
        .runtime
        .installed
        .as_ref()
        .ok()
        .map(|graph| Arc::downgrade(&graph.engine));
    drop(f.runtime);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while weak.upgrade().is_some()
        || engine
            .as_ref()
            .is_some_and(|engine| engine.upgrade().is_some())
    {
        assert!(
            tokio::time::Instant::now() < deadline,
            "actual Runtime/graph custody did not drop"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    // TempDir, socket and owner remain alive until their real jobs drop.
}

#[tokio::test]
async fn c1_installed_graph_reuses_same_engine_registry_and_port_without_cli_launch() {
    let f = fixture("claude", true);
    let (_, tasks) = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    let guard = f.runtime.control_admission.lock().await;
    let one = f.runtime.installed_driver_composition(&tasks[0]).unwrap();
    let two = f.runtime.installed_driver_composition(&tasks[0]).unwrap();
    let graph = f.runtime.installed.as_ref().ok().unwrap();
    assert!(Arc::ptr_eq(one.engine(), two.engine()));
    assert!(Arc::ptr_eq(one.selected(), two.selected()));
    assert!(Arc::ptr_eq(one.engine(), &graph.engine));
    assert!(Arc::ptr_eq(
        one.selected(),
        &graph.registry.native_phase_port("worker").unwrap()
    ));
    assert_eq!(
        count(&f, "managed_effects"),
        0,
        "construction ran helpers/CLI"
    );
    assert_eq!(count(&f, "task_drivers"), 0);
    drop(one);
    drop(two);
    drop(guard);
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}

#[tokio::test]
async fn c2_paired_undeclared_protocol_fixture_has_no_claim_or_helper_effects() {
    let f = fixture("claude", false);
    let (_, tasks) = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    let guard = f.runtime.control_admission.lock().await;
    let error = f
        .runtime
        .installed_driver_composition(&tasks[0])
        .err()
        .unwrap();
    assert!(
        error
            .to_string()
            .contains("compatibility declaration absent")
    );
    drop(guard);
    tokio::time::sleep(Duration::from_millis(100)).await;
    for table in ["task_drivers", "execution_units", "managed_effects"] {
        assert_eq!(count(&f, table), 0);
    }
    let attention: String = raw(&f)
        .query_row(
            "SELECT attention FROM scheduler_tasks WHERE task_id=?1",
            [tasks[0].id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        attention,
        "{\"dispatch_available\":false,\"kind\":\"native_binding_unavailable\"}"
    );
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}

#[tokio::test]
async fn c5_c6_actual_ingress_source_prepared_transport_and_record_only_binding() {
    for provider in ["claude", "codex"] {
        let mut f = fixture(provider, true);
        f.register_real_git_project();
        let (goal, tasks) = accept(&f, 1).await;
        f.runtime.start().await.unwrap();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(40);
        let bound = loop {
            let rows = f
                .owner
                .store
                .lock()
                .unwrap()
                .records(&tasks[0].scope(), RecordKind::Workflow)
                .unwrap();
            let bound = rows
                .first()
                .and_then(|record| {
                    serde_json::from_value::<crate::workflow::WorkflowSnapshot>(record.data.clone())
                        .ok()
                })
                .filter(|workflow| {
                    workflow
                        .history
                        .iter()
                        .any(|attempt| attempt.execution.is_some())
                });
            if let Some(bound) = bound {
                break bound;
            }
            if tokio::time::Instant::now() >= deadline {
                let events = f
                    .owner
                    .store
                    .lock()
                    .unwrap()
                    .events(&tasks[0].scope(), 0, 1000)
                    .unwrap();
                let kinds: Vec<_> = events
                    .iter()
                    .map(|event| (&event.kind, &event.data))
                    .collect();
                let handoff = f
                    .runtime
                    .observe_source_handoff(tasks[0].id)
                    .unwrap()
                    .map(|v| v.state());
                panic!(
                    "SETUP: actual producer did not reach bound {provider}; handoff={handoff:?}; events={kinds:?}"
                );
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        };
        assert_eq!(count(&f, "task_drivers"), 1);
        assert_eq!(
            raw(&f)
                .query_row(
                    "SELECT rotation FROM scheduler_projects WHERE project_id=?1",
                    [f.project.id.to_string()],
                    |r| r.get::<_, u64>(0)
                )
                .unwrap(),
            1
        );
        let attempt = bound
            .history
            .iter()
            .find(|attempt| attempt.execution.is_some())
            .unwrap();
        assert_eq!(attempt.agent.as_deref(), Some("worker"));
        assert_eq!(count(&f, "native_invocations"), 1);
        let task = f
            .owner
            .store
            .lock()
            .unwrap()
            .task(tasks[0].id)
            .unwrap()
            .unwrap();
        let marker_version: u64 = raw(&f)
            .query_row(
                "SELECT marker_task_version FROM managed_phase_operations WHERE task_id=?1",
                [task.id.to_string()],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            task.version, marker_version,
            "record-only binder changed Task.version"
        );
        let attention: Option<String> = raw(&f)
            .query_row(
                "SELECT attention FROM scheduler_tasks WHERE task_id=?1",
                [task.id.to_string()],
                |r| r.get(0),
            )
            .unwrap();
        assert!(attention.is_none(), "genuine claim did not clear hold");
        let facts = f
            .runtime
            .handle_control(
                &f.socket,
                f.request(ControlAction::GoalStatus {
                    project: f.project.id,
                    goal,
                }),
            )
            .await
            .unwrap();
        assert!(matches!(
            facts,
            ControlResponse::GoalFacts {
                dispatch_available: false,
                attention: UnavailableReason::NativeContinuationUnavailable,
                ..
            }
        ));
        let reference = attempt.execution.as_ref().unwrap();
        let unit = f
            .owner
            .store
            .lock()
            .unwrap()
            .execution_unit(reference.unit)
            .unwrap();
        // Locate the actual configured resource output, never an invented grant.
        let profile = crate::execution::resources::ResourceManager::new(f.owner.clone())
            .profile(&unit)
            .unwrap();
        std::fs::write(profile.output.join("fixture-bootstrap-release"), "release").unwrap();
        finish(f).await;
    }
}
