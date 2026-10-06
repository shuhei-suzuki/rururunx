use super::{control::*, goal::*, *};
use crate::{
    config::{AgentConfig, WorkflowClass},
    domain::RiskClass,
};
use uuid::Uuid;

fn config() -> Config {
    let mut c = Config::default();
    c.agents.insert(
        "worker".into(),
        AgentConfig {
            provider: Some("claude".into()),
            command: vec!["/bin/true".into()],
            ..Default::default()
        },
    );
    c
}
fn definition() -> GoalDefinition {
    GoalDefinition {
        title: "t".into(),
        objective: "o".into(),
        criteria: vec![CriterionDefinition {
            id: "c".into(),
            description: "d".into(),
            evaluator: CriterionEvaluator::RequiredTasksVerified,
        }],
        constraints: vec![],
        non_goals: vec![],
        source_refs: vec![],
    }
}
fn plan() -> GoalPlan {
    GoalPlan {
        definition: definition(),
        tasks: vec![TaskDefinition {
            key: "one".into(),
            title: "work".into(),
            acceptance_criteria: vec!["verified result".into()],
            executor: "worker".into(),
            reviewers: vec![],
            workflow: WorkflowClass::Quick,
            risk: RiskClass::R2,
        }],
        dependencies: vec![],
    }
}
#[test]
fn canonical_goal_definition_has_framed_order_and_variant_identity() {
    let original = definition();
    let first = original.canonical_digest().unwrap();
    let reversed:GoalDefinition=serde_json::from_str(r#"{"source_refs":[],"non_goals":[],"constraints":[],"criteria":[{"evaluator":{"kind":"required_tasks_verified"},"description":"d","id":"c"}],"objective":"o","title":"t"}"#).unwrap();
    assert_eq!(first, reversed.canonical_digest().unwrap());
    let mut a = original.clone();
    let mut b = original.clone();
    a.constraints = vec!["x".into(), "yz".into()];
    b.constraints = vec!["xy".into(), "z".into()];
    assert_ne!(a.canonical_digest().unwrap(), b.canonical_digest().unwrap());
    b.constraints = vec!["yz".into(), "x".into()];
    assert_ne!(a.canonical_digest().unwrap(), b.canonical_digest().unwrap());
    a.criteria[0].evaluator = CriterionEvaluator::Human {
        goal_pack_input: false,
    };
    b = a.clone();
    b.criteria[0].evaluator = CriterionEvaluator::Human {
        goal_pack_input: true,
    };
    assert_ne!(a.canonical_digest().unwrap(), b.canonical_digest().unwrap());
}
#[test]
fn accepted_definition_never_uses_unverified_boolean_or_duplicate_criterion() {
    let mut d = definition();
    d.criteria.push(d.criteria[0].clone());
    assert!(d.canonical_digest().is_err());
    d.criteria.pop();
    d.criteria[0].evaluator = CriterionEvaluator::Unverified;
    assert!(d.canonical_digest().is_err());
    let mut raw = serde_json::to_value(definition()).unwrap();
    raw["criteria"][0]["satisfied"] = true.into();
    assert!(serde_json::from_value::<GoalDefinition>(raw).is_err());
}
#[test]
fn definition_exact_encoded_byte_budget_and_utf8_text_boundary() {
    let mut d = definition();
    d.constraints = vec!["x".repeat(MAX_TEXT_BYTES); 63];
    // Independent count of the declared framing: domain/title/objective, criterion count,
    // ID/description/variant, and three list counts. Each remaining text has its length prefix.
    let fixed = 8
        + "rrx.goal.definition.v1".len()
        + 9
        + 9
        + 8
        + 9
        + 9
        + 1
        + 8
        + 63 * (8 + MAX_TEXT_BYTES)
        + 8
        + 8
        + 8;
    let remaining = MAX_DEFINITION_BYTES - fixed;
    assert!(remaining < MAX_TEXT_BYTES);
    d.constraints.push("x".repeat(remaining));
    assert!(d.canonical_digest().is_ok(), "inclusive exact1MiB must fit");
    d.constraints.last_mut().unwrap().push('x');
    assert!(d.canonical_digest().is_err(), "encoded1MiB+1 must refuse");
    let mut d = definition();
    d.title = "界".repeat(MAX_TEXT_BYTES / 3) + "a";
    assert_eq!(d.title.len(), MAX_TEXT_BYTES);
    assert!(d.canonical_digest().is_ok());
    d.title.push('x');
    assert!(d.canonical_digest().is_err());
}
#[test]
fn actual_plan_validator_reuses_hard_dag_and_preserves_risk_floor() {
    let configured = config();
    let valid = plan().validate(&configured).unwrap();
    assert_eq!(valid.plan().tasks[0].workflow, WorkflowClass::Standard);
    let mut p = plan();
    p.tasks.push(p.tasks[0].clone());
    assert!(p.validate(&configured).is_err());
    let mut p = plan();
    let mut second = p.tasks[0].clone();
    second.key = "two".into();
    p.tasks.push(second);
    p.dependencies = vec![
        PlanDependency {
            prerequisite: "one".into(),
            dependent: "two".into(),
            hard: true,
        },
        PlanDependency {
            prerequisite: "two".into(),
            dependent: "one".into(),
            hard: true,
        },
    ];
    assert!(p.clone().validate(&configured).is_err());
    p.dependencies[1].hard = false;
    assert!(p.clone().validate(&configured).is_ok());
    p.dependencies[1].dependent = "foreign".into();
    assert!(p.validate(&configured).is_err());
    let mut p = plan();
    p.tasks[0].executor = "foreign".into();
    assert!(p.validate(&configured).is_err());
}
#[tokio::test]
async fn actual_local_ingress_checks_epoch_and_never_mints_goal_before_schema9() {
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
    let runtime = Runtime::new(owner.clone(), config()).unwrap();
    let identity = runtime.control_identity();
    assert_eq!(
        identity.state,
        dir.path().join("state.db").canonicalize().unwrap()
    );
    assert_eq!(identity.instance, owner.instance_id());
    assert_eq!(identity.epoch, owner.epoch());
    identity.validate().unwrap();
    assert_eq!(runtime.control_identity(), identity);
    let (server, _client) = tokio::net::UnixStream::pair().unwrap();
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: owner.instance_id().into(),
        epoch: owner.epoch(),
        action: ControlAction::CreateGoal {
            project: crate::domain::ProjectId::new(),
            expected_project: 1,
            plan: plan(),
        },
    };
    let mut stale = request.clone();
    stale.epoch += 1;
    assert!(runtime.handle_control(&server, stale).await.is_err());
    assert!(matches!(
        runtime.handle_control(&server, request).await.unwrap(),
        ControlResponse::Unavailable {
            reason: UnavailableReason::RuntimeSchemaPending,
            ..
        }
    ));
    assert!(owner.store().lock().unwrap().projects().unwrap().is_empty());
}

#[tokio::test]
async fn actual_service_resolves_names_uuid_and_cwd_without_source_grant() {
    use crate::domain::{Project, ProjectState};
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
    let root = dir.path().join("source");
    std::fs::create_dir_all(root.join("sub")).unwrap();
    let root = root.canonicalize().unwrap();
    let mut project = Project::new(
        "route".into(),
        root.clone(),
        "routing-metadata-fixture".into(),
        "main".into(),
    );
    owner
        .store()
        .lock()
        .unwrap()
        .put_project(&mut project)
        .unwrap();
    let runtime = Runtime::new(owner.clone(), config()).unwrap();
    let (server, _client) = tokio::net::UnixStream::pair().unwrap();
    for selector in [Some("route".into()), Some(project.id.to_string()), None] {
        let request = ControlRequest {
            request_id: Uuid::new_v4(),
            instance: owner.instance_id().into(),
            epoch: owner.epoch(),
            action: ControlAction::ResolveProject {
                selector,
                cwd: root.join("sub"),
            },
        };
        match runtime.handle_control(&server, request).await.unwrap() {
            ControlResponse::ProjectResolved {
                project: id,
                version,
                display_name,
                state,
            } => {
                assert_eq!(id, project.id);
                assert_eq!(version, project.version);
                assert_eq!(display_name, "route");
                assert_eq!(state, ProjectState::Registered);
            }
            _ => panic!("actual service routing result missing"),
        }
    }
    std::fs::create_dir_all(project.worktree_root.join("unregistered")).unwrap();
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: owner.instance_id().into(),
        epoch: owner.epoch(),
        action: ControlAction::ResolveProject {
            selector: None,
            cwd: project.worktree_root.join("unregistered"),
        },
    };
    assert!(
        runtime.handle_control(&server, request).await.is_err(),
        "unregistered worktree namespace must not route as source"
    );
    assert!(
        owner
            .store()
            .lock()
            .unwrap()
            .goals(project.id)
            .unwrap()
            .is_empty(),
        "routing must not create a Goal/Source/Unit grant"
    );
}

#[tokio::test]
async fn actual_routing_refuses_ambiguity_stale_snapshot_and_body_index_corruption() {
    use crate::domain::Project;
    let dir = tempfile::tempdir().unwrap();
    let state = dir.path().join("state.db");
    let owner = RuntimeOwner::open(&state).unwrap();
    let mut projects = Vec::new();
    for index in 0..2 {
        let root = dir.path().join(format!("source{index}"));
        std::fs::create_dir(&root).unwrap();
        let mut project = Project::new(
            "duplicate".into(),
            root.canonicalize().unwrap(),
            format!("routing-metadata-fixture-{index}"),
            "main".into(),
        );
        owner
            .store()
            .lock()
            .unwrap()
            .put_project(&mut project)
            .unwrap();
        projects.push(project);
    }
    let runtime = Runtime::new(owner.clone(), config()).unwrap();
    let (server, _client) = tokio::net::UnixStream::pair().unwrap();
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: owner.instance_id().into(),
        epoch: owner.epoch(),
        action: ControlAction::ResolveProject {
            selector: Some("duplicate".into()),
            cwd: projects[0].root.clone(),
        },
    };
    assert!(runtime.handle_control(&server, request).await.is_err());
    let id = projects[0].id.to_string();
    {
        let store = owner.store();
        let mut store = store.lock().unwrap();
        let snapshot = store
            .runtime_project_routes(
                Some(&id),
                &projects[0].root,
                owner.instance_id(),
                owner.epoch(),
            )
            .unwrap();
        projects[0].name = "changed".into();
        store.put_project(&mut projects[0]).unwrap();
        assert!(
            store
                .recheck_runtime_project_routes(
                    Some(&id),
                    &projects[0].root,
                    owner.instance_id(),
                    owner.epoch(),
                    &snapshot
                )
                .is_err()
        );
        assert!(
            store
                .runtime_project_routes(
                    Some(&id),
                    &projects[0].root,
                    owner.instance_id(),
                    owner.epoch() + 1
                )
                .is_err()
        );
    }
    let connection = crate::state::current_test_writer(&state).unwrap();
    connection
        .execute(
            "UPDATE projects SET body=json_set(body,'$.version',version+1) WHERE id=?1",
            [&id],
        )
        .unwrap();
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: owner.instance_id().into(),
        epoch: owner.epoch(),
        action: ControlAction::ResolveProject {
            selector: Some(id),
            cwd: projects[0].root.clone(),
        },
    };
    assert!(
        runtime.handle_control(&server, request).await.is_err(),
        "routing body/index mismatch cannot be returned as current metadata"
    );
}
