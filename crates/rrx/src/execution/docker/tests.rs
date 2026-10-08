use super::*;
use crate::{domain::Task, execution::attempts::AttemptManager};
use serde_json::{Value, json};
use std::{os::unix::fs::PermissionsExt, path::PathBuf};

const OWN: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SIBLING: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const UNREGISTERED: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

struct Fixture {
    _dir: crate::runtime::LegacyFixture,
    owner: Arc<RuntimeOwner>,
    unit: ExecutionUnit,
    siblings: Vec<ExecutionUnit>,
    program: PathBuf,
    state: PathBuf,
    operation: OperationId,
    name: String,
    profile: resources::ResourceProfile,
}
impl Fixture {
    async fn new() -> Self {
        let (dir, owner, task) = results::tests::fixture().await;
        let attempts = AttemptManager::new(owner.clone());
        let (unit, _) = attempts
            .prepare(task.id, "codex", "Implement", None)
            .await
            .unwrap();
        let mut siblings = Vec::new();
        for i in 0..3 {
            let mut sibling = Task::new(
                task.project_id,
                task.goal_id,
                format!("sibling {i}"),
                "claude".into(),
            );
            owner.store.lock().unwrap().put_task(&mut sibling).unwrap();
            siblings.push(
                attempts
                    .prepare(sibling.id, "claude", "Implement", None)
                    .await
                    .unwrap()
                    .0,
            );
        }
        let program = dir.path().join("docker-fixture.py");
        let state = program.with_extension("json");
        std::fs::write(&program, include_str!("fixture.py")).unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut profile = resources::ResourceManager::new(owner.clone())
            .profile(&unit)
            .unwrap();
        profile.real_tools.insert("docker".into(), program.clone());
        let operation = OperationId::new();
        let plan = tools::plan(
            &profile,
            &unit,
            "docker",
            &["run".into(), "-d".into(), "fixture-image".into()],
            operation,
        )
        .unwrap();
        assert!(plan.args.windows(2).any(|a|
            a[0] == "--label" && a[1] == format!("org.rururunx.operation={operation}")));
        let name = plan.docker_name.unwrap();
        owner
            .store
            .lock()
            .unwrap()
            .reserve_managed_effect(
                &unit.authority(),
                &ManagedEffect {
                    id: operation,
                    unit_id: unit.id,
                    scope: unit.scope.clone(),
                    kind: "docker_create".into(),
                    idempotency_key: format!("tool-{operation}"),
                    expected_target: name.clone(),
                    state: EffectState::Pending,
                    receipt: BTreeMap::new(),
                    version: 1,
                },
            )
            .unwrap();
        let other_profile = resources::ResourceManager::new(owner.clone())
            .profile(&siblings[0])
            .unwrap();
        std::fs::write(&state, serde_json::to_vec(&json!({
            "database": dir.path().join("state.db"), "unit": unit.id, "cookie": unit.cookie,
            "calls": [], "containers": {
                OWN: {"name":name,"operation":operation,"labels":profile.docker_labels,"running":true},
                SIBLING: {"name":"sibling","operation":OperationId::new(),"labels":other_profile.docker_labels,"running":true}
            }, "networks":{}, "volumes":{}
        })).unwrap()).unwrap();
        Self {
            _dir: dir,
            owner,
            unit,
            siblings,
            program,
            state,
            operation,
            name,
            profile,
        }
    }
    fn read(&self) -> Value {
        serde_json::from_slice(&std::fs::read(&self.state).unwrap()).unwrap()
    }
    fn update(&self, change: impl FnOnce(&mut Value)) {
        let mut value = self.read();
        change(&mut value);
        std::fs::write(&self.state, serde_json::to_vec(&value).unwrap()).unwrap();
    }
    async fn qualify(&self) -> Result<DockerTarget> {
        qualify_inner(self.owner.clone(), &self.unit, Some(&self.program)).await
    }
    fn close(&self) -> crate::state::CleanupClaim {
        let mut store = self.owner.store.lock().unwrap();
        let current = store.execution_unit(self.unit.id).unwrap();
        store.retire_execution(&current.authority(), false).unwrap();
        store
            .claim_execution_cleanup(self.unit.id, self.owner.epoch, crate::domain::now_ms())
            .unwrap()
            .unwrap()
    }
    fn assert_siblings_open(&self) {
        let store = self.owner.store.lock().unwrap();
        for unit in &self.siblings {
            store
                .validate_execution(&unit.authority(), true, false)
                .unwrap();
        }
        assert_eq!(self.read()["containers"][SIBLING]["running"], true);
    }
}

#[tokio::test]
async fn registered_docker_cleanup_preserves_three_sibling_units_and_known_work() {
    let fixture = Fixture::new().await;
    fixture.qualify().await.unwrap();
    let before_tasks: Vec<_> = fixture
        .siblings
        .iter()
        .map(|unit| {
            serde_json::to_value(
                fixture
                    .owner
                    .store
                    .lock()
                    .unwrap()
                    .task(unit.scope.task_id.unwrap())
                    .unwrap(),
            )
            .unwrap()
        })
        .collect();
    let known = fixture
        .owner
        .store
        .lock()
        .unwrap()
        .finish_execution(
            &fixture.unit.authority(),
            WorkOutcome::Success,
            Disposition::Completed,
        )
        .unwrap();
    let artifact = results::ResultStore::new(fixture.owner.clone())
        .capture(&known.authority(), &known.base_sha, BTreeMap::new())
        .await
        .unwrap();
    let claim = fixture.close();
    let report = cleanup_for(fixture.owner.clone(), &claim, Some(&fixture.program)).await;
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert!(report.remaining.is_empty());
    assert_eq!(report.actions.len(), 2);
    assert_eq!(report.actions[0].action, CleanupActionKind::DockerKill);
    assert_eq!(report.actions[0].confirmation, CleanupConfirmation::Exited);
    assert_eq!(report.actions[1].action, CleanupActionKind::DockerRemove);
    assert_eq!(report.actions[1].confirmation, CleanupConfirmation::Absent);
    assert!(fixture.read()["containers"].get(OWN).is_none());
    fixture.assert_siblings_open();
    let store = fixture.owner.store.lock().unwrap();
    assert_eq!(
        store.execution_unit(fixture.unit.id).unwrap().work,
        Some(WorkOutcome::Success)
    );
    assert_eq!(
        serde_json::to_value(store.result_artifact(artifact.id).unwrap()).unwrap(),
        serde_json::to_value(&artifact).unwrap()
    );
    for (unit, before) in fixture.siblings.iter().zip(before_tasks) {
        assert_eq!(
            serde_json::to_value(store.task(unit.scope.task_id.unwrap()).unwrap()).unwrap(),
            before
        );
    }
    assert!(
        store
            .managed_effects(fixture.unit.id)
            .unwrap()
            .iter()
            .filter(|effect| effect.kind == "docker_cleanup")
            .all(|effect| effect.state == EffectState::Confirmed)
    );
}

#[tokio::test]
async fn docker_filter_and_operation_identity_are_independently_checked() {
    let fixture = Fixture::new().await;
    fixture.qualify().await.unwrap();
    fixture.update(|value| {
        value["ignore_filter"] = json!(true);
        // The creation/name tuple alone cannot authorize a resource whose unit labels changed.
        value["containers"][SIBLING]["operation"] = json!(fixture.operation);
        value["containers"][SIBLING]["name"] = json!(fixture.name);
        value["containers"][UNREGISTERED] = json!({"name":"unregistered",
            "operation":OperationId::new(), "labels":fixture.profile.docker_labels,"running":true});
        value["networks"][SIBLING] = value["containers"][SIBLING].clone();
        value["volumes"]["foreign-volume"] = value["containers"][SIBLING].clone();
        value["volumes"]["unregistered-own-volume"] = value["containers"][UNREGISTERED].clone();
    });
    let report = cleanup_for(
        fixture.owner.clone(),
        &fixture.close(),
        Some(&fixture.program),
    )
    .await;
    assert!(
        report
            .errors
            .iter()
            .any(|e| e == "docker_label_filter_mismatch")
    );
    assert!(
        report
            .errors
            .iter()
            .any(|e| e == "docker_identity_unregistered_or_changed")
    );
    assert!(
        report
            .remaining
            .contains(&format!("container:{UNREGISTERED}"))
    );
    assert!(
        report
            .remaining
            .contains(&"volume:unregistered-own-volume".to_owned())
    );
    assert!(
        !report
            .remaining
            .iter()
            .any(|id| id.contains(SIBLING) || id.contains("foreign-volume"))
    );
    assert!(
        !report
            .actions
            .iter()
            .any(|action| action.target.contains(SIBLING) || action.target.contains(UNREGISTERED))
    );
    assert_eq!(fixture.read()["containers"][UNREGISTERED]["running"], true);
    fixture.assert_siblings_open();
}

#[tokio::test]
async fn docker_engine_change_refuses_cleanup_and_native_requalification() {
    let fixture = Fixture::new().await;
    fixture.qualify().await.unwrap();
    fixture.update(|value| value["engine"] = json!("different-engine"));
    assert!(fixture.qualify().await.is_err());
    let report = cleanup_for(
        fixture.owner.clone(),
        &fixture.close(),
        Some(&fixture.program),
    )
    .await;
    assert!(report.actions.is_empty());
    assert!(
        report
            .errors
            .iter()
            .any(|e| e == "docker_reconciliation_incomplete")
    );
    assert_eq!(fixture.read()["containers"][OWN]["running"], true);
    fixture.assert_siblings_open();
}

#[tokio::test]
async fn docker_remote_transport_is_rejected_before_daemon_probe() {
    let fixture = Fixture::new().await;
    fixture.update(|value| value["transport"] = json!("ssh"));
    assert!(fixture.qualify().await.is_err());
    let calls = fixture.read()["calls"].as_array().unwrap().clone();
    assert_eq!(calls.len(), 2);
    assert!(
        !calls
            .iter()
            .any(|call| matches!(call[0].as_str(), Some("version" | "info")))
    );
    assert!(fixture.qualify().await.is_err()); // unknown qualification never replays probes
    assert_eq!(fixture.read()["calls"].as_array().unwrap().len(), 2);
    fixture.assert_siblings_open();
}

#[tokio::test]
async fn docker_timeout_retains_partial_kill_and_uncertain_helper_receipt() {
    let fixture = Fixture::new().await;
    fixture.qualify().await.unwrap();
    fixture.update(|value| value["delay_after_kill"] = json!(true));
    let report = cleanup_for(
        fixture.owner.clone(),
        &fixture.close(),
        Some(&fixture.program),
    )
    .await;
    assert_eq!(report.actions.len(), 1);
    assert_eq!(report.actions[0].action, CleanupActionKind::DockerKill);
    assert_eq!(report.actions[0].outcome, CleanupActionOutcome::Sent);
    assert_eq!(
        report.actions[0].confirmation,
        CleanupConfirmation::NotAttempted
    );
    assert!(
        report
            .errors
            .iter()
            .any(|e| e == "docker_reconciliation_incomplete")
    );
    assert!(
        fixture
            .owner
            .store
            .lock()
            .unwrap()
            .managed_effects(fixture.unit.id)
            .unwrap()
            .iter()
            .any(|effect| effect.kind == "docker_cleanup" && effect.state == EffectState::Unknown)
    );
    fixture.assert_siblings_open();
}

#[tokio::test]
async fn docker_stale_cleanup_claim_never_launches_a_probe() {
    let fixture = Fixture::new().await;
    fixture.qualify().await.unwrap();
    let mut claim = fixture.close();
    claim.version += 1;
    let before = fixture.read()["calls"].as_array().unwrap().len();
    let report = cleanup_for(fixture.owner.clone(), &claim, Some(&fixture.program)).await;
    assert!(report.actions.is_empty());
    assert!(!report.errors.is_empty());
    assert_eq!(fixture.read()["calls"].as_array().unwrap().len(), before);
}

#[test]
fn docker_protocol_profile_rejects_missing_and_unsupported_responses() {
    assert!(versions(b"\"28.0.0\"\n\"28.0.0\"\n\"1.48\"\n").is_ok());
    for invalid in [
        b"[]".as_slice(),
        b"28.0.0\n28.0.0\n1.48",
        b"\"29.0.0\"\n\"28.0.0\"\n\"1.48\"",
        b"\"28.0.0\"\n\"28.0.0\"\n\"1.47\"",
    ] {
        assert!(versions(invalid).is_err());
    }
    assert!(engine_digest(b"\"\"").is_err());
    assert!(engine_digest(b"\"fixture-engine\"").is_ok());
    assert!(!context_name("ssh://example.invalid"));
}

#[tokio::test]
async fn docker_mutation_ack_timeout_preserves_exact_durable_target_and_action() {
    for (delay, expected) in [
        ("kill", CleanupActionKind::DockerKill),
        ("remove", CleanupActionKind::DockerRemove),
    ] {
        let fixture = Fixture::new().await;
        fixture.qualify().await.unwrap();
        fixture.update(|value| value["delay_ack"] = json!(delay));
        let claim = fixture.close();
        let report = cleanup_for(fixture.owner.clone(), &claim, Some(&fixture.program)).await;
        let action = report
            .actions
            .iter()
            .find(|action| action.action == expected)
            .unwrap();
        assert_eq!(action.target, format!("container:{OWN}"));
        assert_eq!(action.outcome, CleanupActionOutcome::Unknown);
        assert_eq!(action.confirmation, CleanupConfirmation::NotAttempted);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error == "docker_reconciliation_incomplete")
        );
        let effects = fixture
            .owner
            .store
            .lock()
            .unwrap()
            .managed_effects(fixture.unit.id)
            .unwrap();
        let prefix = format!(
            "container-{}:{OWN}:",
            if delay == "kill" { "kill" } else { "remove" }
        );
        let effect = effects
            .iter()
            .find(|effect| {
                effect.kind == "docker_cleanup" && effect.expected_target.starts_with(&prefix)
            })
            .unwrap();
        assert_eq!(effect.state, EffectState::Unknown);
        let mut observation = CleanupObservation {
            unit_id: fixture.unit.id,
            at: crate::domain::now_ms(),
            outcome: CleanupOutcome::Unknown,
            coverage: BTreeMap::new(),
            remaining: vec![],
            errors: vec![],
            actions: vec![],
        };
        super::super::cleanup::append_docker(&mut observation, report);
        fixture
            .owner
            .store
            .lock()
            .unwrap()
            .finish_execution_cleanup(&claim, &observation)
            .unwrap();
        let persisted = fixture
            .owner
            .store
            .lock()
            .unwrap()
            .execution_cleanup_observations(fixture.unit.id, 1)
            .unwrap();
        assert_eq!(
            serde_json::to_value(&persisted[0]).unwrap(),
            serde_json::to_value(&observation).unwrap()
        );
        fixture.assert_siblings_open();
    }
}

#[tokio::test]
async fn docker_changing_inventories_fit_the_actual_cookie_persistence_budget() {
    let fixture = Fixture::new().await;
    fixture.qualify().await.unwrap();
    fixture.update(|value| {
        value["rotating"] = json!(true);
        let initial = (0..32).map(|n| format!("{n:064x}")).collect::<Vec<_>>();
        let final_ids = (32..64).map(|n| format!("{n:064x}")).collect::<Vec<_>>();
        value["initial_ids"] = json!(initial);
        value["final_ids"] = json!(final_ids);
        for (n, id) in initial.iter().chain(&final_ids).enumerate() {
            value["containers"][id] = json!({"name":format!("unregistered-{n}"),
                "operation":OperationId::new(),"labels":fixture.profile.docker_labels,"running":true});
        }
        for n in 0..32 {
            value["networks"][format!("{:064x}", n + 64)] = json!({"labels":fixture.profile.docker_labels});
            value["volumes"][format!("own-volume-{n}")] = json!({"labels":fixture.profile.docker_labels});
        }
    });
    let claim = fixture.close();
    let mut report = DockerCleanup {
        coverage: "fixture bounded inventory".into(),
        remaining: vec![],
        errors: vec![],
        actions: vec![],
    };
    // Exercise all four independently valid inventories without a timing-based
    // assumption about this host's 128 Python process startups.
    cleanup_inner(
        fixture.owner.clone(),
        &claim,
        Some(&fixture.program),
        &mut report,
    )
    .await
    .unwrap();
    normalize_report(&mut report);
    assert_eq!(report.remaining.len(), MAX_REMAINING);
    let mut observation = CleanupObservation {
        unit_id: fixture.unit.id,
        at: crate::domain::now_ms(),
        outcome: CleanupOutcome::Unknown,
        coverage: BTreeMap::new(),
        // Controlled maximum process observation; these are synthetic receipt
        // identities, not live processes or a tracker conformance claim.
        remaining: (0..super::super::cleanup::MAX_COOKIE_MATCHES)
            .map(|n| format!("pid:{}@1:0", n + 1))
            .collect(),
        errors: vec![],
        actions: vec![],
    };
    super::super::cleanup::append_docker(&mut observation, report);
    fixture
        .owner
        .store
        .lock()
        .unwrap()
        .finish_execution_cleanup(&claim, &observation)
        .unwrap();
    assert_eq!(observation.remaining.len(), 1024);
    let persisted = fixture
        .owner
        .store
        .lock()
        .unwrap()
        .execution_cleanup_observations(fixture.unit.id, 1)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&persisted[0]).unwrap(),
        serde_json::to_value(&observation).unwrap()
    );
    fixture.assert_siblings_open();
}

#[test]
fn docker_partial_reports_are_capped_and_keep_unknown_coverage() {
    let mut report = DockerCleanup {
        coverage: "partial".into(),
        remaining: (0..200).map(|n| format!("container:{n:064x}")).collect(),
        errors: vec![],
        actions: vec![],
    };
    normalize_report(&mut report);
    assert_eq!(report.remaining.len(), MAX_REMAINING);
    assert!(
        report
            .errors
            .iter()
            .any(|error| error == "docker_remaining_limit_exceeded")
    );
    assert!(report.coverage.contains("no full release confirmation"));
}

#[test]
fn docker_configuration_reference_is_nonsecret_exact_and_absolute() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    std::fs::create_dir(&a).unwrap();
    std::fs::create_dir(&b).unwrap();
    let first = routing_reference(&a, false).unwrap();
    assert_ne!(first, routing_reference(&b, false).unwrap());
    assert_ne!(first, routing_reference(&a, true).unwrap());
    let alias = dir.path().join("alias");
    std::os::unix::fs::symlink(&a, &alias).unwrap();
    assert_eq!(first, routing_reference(&alias, false).unwrap());
    assert_eq!(
        routing_reference(&a.join("missing"), false).unwrap(),
        routing_reference(&alias.join("missing"), false).unwrap()
    );
    assert!(routing_reference(Path::new("relative"), false).is_err());
    assert!(
        !serde_json::to_string(&first)
            .unwrap()
            .contains(a.to_str().unwrap())
    );
}
