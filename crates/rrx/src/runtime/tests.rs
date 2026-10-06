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
async fn actual_local_ingress_checks_epoch_and_refuses_unknown_project() {
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
    assert!(runtime.handle_control(&server, request).await.is_err());
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

// Actual accepted Unix peer ingress; this fixture never constructs Human/Driver authority.
struct ControlFixture {
    _dir: tempfile::TempDir,
    owner: Arc<RuntimeOwner>,
    runtime: Arc<Runtime>,
    project: crate::domain::Project,
    socket: tokio::net::UnixStream,
    _peer: tokio::net::UnixStream,
}
impl ControlFixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        let root = dir.path().join("source");
        std::fs::create_dir(&root).unwrap();
        let mut project = crate::domain::Project::new(
            "control-fixture".into(),
            root.canonicalize().unwrap(),
            "account-free-no-Git".into(),
            "main".into(),
        );
        owner
            .store()
            .lock()
            .unwrap()
            .put_project(&mut project)
            .unwrap();
        let runtime = Arc::new(Runtime::new(owner.clone(), config()).unwrap());
        let (socket, peer) = tokio::net::UnixStream::pair().unwrap();
        Self {
            _dir: dir,
            owner,
            runtime,
            project,
            socket,
            _peer: peer,
        }
    }
    fn request(&self, action: ControlAction) -> ControlRequest {
        ControlRequest {
            request_id: Uuid::new_v4(),
            instance: self.owner.instance_id().into(),
            epoch: self.owner.epoch(),
            action,
        }
    }
    async fn create(&self, plan: GoalPlan) -> crate::domain::GoalId {
        match self
            .runtime
            .handle_control(
                &self.socket,
                self.request(ControlAction::CreateGoal {
                    project: self.project.id,
                    expected_project: self.project.version,
                    plan,
                }),
            )
            .await
            .unwrap()
        {
            ControlResponse::GoalAccepted { goal, .. } => goal,
            _ => panic!("actual accepted Goal required"),
        }
    }
}
#[tokio::test]
async fn actual_accepted_goal_transaction_idempotency_and_generic_writers() {
    let f = ControlFixture::new();
    let request = f.request(ControlAction::CreateGoal {
        project: f.project.id,
        expected_project: f.project.version,
        plan: plan(),
    });
    let response = f
        .runtime
        .handle_control(&f.socket, request.clone())
        .await
        .unwrap();
    let ControlResponse::GoalAccepted {
        goal,
        version,
        task_count,
    } = response
    else {
        panic!("accepted result")
    };
    assert_eq!((version, task_count), (1, 1));
    let mut original = f.owner.store().lock().unwrap().goal(goal).unwrap().unwrap();
    let mut task = f
        .owner
        .store()
        .lock()
        .unwrap()
        .task(original.dag.nodes[0])
        .unwrap()
        .unwrap();
    let events = f
        .owner
        .store()
        .lock()
        .unwrap()
        .events(&original.scope(), 0, 100)
        .unwrap()
        .len();
    assert!(
        matches!(f.runtime.handle_control(&f.socket,request.clone()).await.unwrap(),ControlResponse::GoalAccepted{goal:id,..} if id==goal)
    );
    let mut reused = request.clone();
    if let ControlAction::CreateGoal { plan, .. } = &mut reused.action {
        plan.definition.title = "changed".into();
    }
    assert!(f.runtime.handle_control(&f.socket, reused).await.is_err());
    {
        let shared = f.owner.store();
        let mut store = shared.lock().unwrap();
        assert_eq!(store.goals(f.project.id).unwrap().len(), 1);
        store.put_goal(&mut original).unwrap();
        store.put_task(&mut task).unwrap();
        assert_eq!(
            store.events(&original.scope(), 0, 100).unwrap().len(),
            events
        );
        original.objective = "arbitrary JSON authority".into();
        assert!(store.put_goal(&mut original).is_err());
        task.title = "arbitrary managed Task write".into();
        assert!(store.put_task(&mut task).is_err());
        let mut new = crate::domain::Goal::new(f.project.id, "untrusted proposal".into(), vec![]);
        assert!(store.put_goal(&mut new).is_err());
        assert_eq!(store.goals(f.project.id).unwrap().len(), 1);
    }
}
#[tokio::test]
async fn actual_accepted_goal_pages_scope_complete_inventory_and_native_hold() {
    let f = ControlFixture::new();
    let mut many = plan();
    for i in 1..129 {
        let mut t = many.tasks[0].clone();
        t.key = format!("task-{i}");
        many.tasks.push(t);
    }
    let goal = f.create(many).await;
    let response = f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::GoalTasks {
                project: f.project.id,
                goal,
                after: None,
                maximum: 128,
            }),
        )
        .await
        .unwrap();
    assert!(serde_json::to_vec(&response).unwrap().len() <= 65536);
    let ControlResponse::GoalTaskPage {
        tasks,
        next: Some(next),
        ..
    } = response
    else {
        panic!("first bounded page")
    };
    assert_eq!(tasks.len(), 128);
    assert!(
        tasks
            .windows(2)
            .all(|p| p[0].scope.task_id < p[1].scope.task_id)
    );
    let last = f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::GoalTasks {
                project: f.project.id,
                goal,
                after: Some(next),
                maximum: 128,
            }),
        )
        .await
        .unwrap();
    assert!(matches!(last,ControlResponse::GoalTaskPage{tasks,next:None,..} if tasks.len()==1));
    for (after, maximum) in [
        (Some(crate::domain::TaskId::new()), 1),
        (None, 0),
        (None, 129),
    ] {
        assert!(
            f.runtime
                .handle_control(
                    &f.socket,
                    f.request(ControlAction::GoalTasks {
                        project: f.project.id,
                        goal,
                        after,
                        maximum
                    })
                )
                .await
                .is_err()
        );
    }
    let task = tasks[0].scope.task_id.unwrap();
    let sources =
        crate::execution::workflow_source::ManagedWorkflowSources::new(f.owner.clone(), config())
            .unwrap();
    let error = sources.prepare(task, "claude").await.unwrap_err();
    // The accepted controller graph must not escape to registered Git without a real Driver.
    assert!(
        format!("{error:#}").contains("Query returned no rows")
            || format!("{error:#}").contains("Driver"),
        "{error:#}"
    );
    let connection = crate::state::current_test_writer(f.owner.state_path()).unwrap();
    for table in [
        "execution_units",
        "managed_effects",
        "task_drivers",
        "native_invocations",
    ] {
        let count: u64 = connection
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "no actual {table} producer may be invented");
    }
    assert!(matches!(
        f.runtime
            .handle_control(
                &f.socket,
                f.request(ControlAction::GoalStatus {
                    project: f.project.id,
                    goal
                })
            )
            .await
            .unwrap(),
        ControlResponse::GoalFacts {
            dispatch_available: false,
            attention: UnavailableReason::NativeBindingUnavailable,
            ..
        }
    ));
}
#[tokio::test]
async fn actual_goal_lifecycle_cas_policy_fence_and_never_prepared_resume() {
    let f = ControlFixture::new();
    let goal = f.create(plan()).await;
    let paused = f.request(ControlAction::SetGoalLifecycle {
        project: f.project.id,
        goal,
        expected_goal: 1,
        target: GoalControl::Pause,
        reason: "explicit lifecycle hold".into(),
    });
    assert!(matches!(
        f.runtime
            .handle_control(&f.socket, paused.clone())
            .await
            .unwrap(),
        ControlResponse::GoalLifecycleChanged {
            version: 2,
            state: crate::domain::GoalState::Paused,
            ..
        }
    ));
    assert!(matches!(
        f.runtime.handle_control(&f.socket, paused).await.unwrap(),
        ControlResponse::GoalLifecycleChanged { version: 2, .. }
    ));
    let stale = f.request(ControlAction::SetGoalLifecycle {
        project: f.project.id,
        goal,
        expected_goal: 1,
        target: GoalControl::Resume,
        reason: "stale".into(),
    });
    assert!(f.runtime.handle_control(&f.socket, stale).await.is_err());
    let resumed = f.request(ControlAction::SetGoalLifecycle {
        project: f.project.id,
        goal,
        expected_goal: 2,
        target: GoalControl::Resume,
        reason: "no Unit has ever been prepared".into(),
    });
    assert!(matches!(
        f.runtime.handle_control(&f.socket, resumed).await.unwrap(),
        ControlResponse::GoalLifecycleChanged {
            version: 3,
            state: crate::domain::GoalState::Running,
            ..
        }
    ));
    let cancel = f.request(ControlAction::SetGoalLifecycle {
        project: f.project.id,
        goal,
        expected_goal: 3,
        target: GoalControl::Cancel,
        reason: "trusted cancel".into(),
    });
    assert!(matches!(
        f.runtime.handle_control(&f.socket, cancel).await.unwrap(),
        ControlResponse::GoalLifecycleChanged {
            version: 4,
            state: crate::domain::GoalState::Cancelled,
            ..
        }
    ));
    assert!(
        f.runtime
            .handle_control(
                &f.socket,
                f.request(ControlAction::SetGoalLifecycle {
                    project: f.project.id,
                    goal,
                    expected_goal: 4,
                    target: GoalControl::Resume,
                    reason: "terminal cannot reactivate".into()
                })
            )
            .await
            .is_err()
    );
}
#[tokio::test]
async fn actual_control_loop_persists_named_hold_without_authority_bumps() {
    let f = ControlFixture::new();
    let goal = f.create(plan()).await;
    let before = f.owner.store().lock().unwrap().goal(goal).unwrap().unwrap();
    let task = f
        .owner
        .store()
        .lock()
        .unwrap()
        .task(before.dag.nodes[0])
        .unwrap()
        .unwrap();
    f.runtime.start().await.unwrap();
    assert!(f.runtime.start().await.is_err());
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let connection = crate::state::current_test_writer(f.owner.state_path()).unwrap();
            let attention: Option<String> = connection
                .query_row(
                    "SELECT attention FROM scheduler_tasks WHERE task_id=?1",
                    [task.id.to_string()],
                    |r| r.get(0),
                )
                .unwrap();
            if attention.is_some() {
                assert!(attention.unwrap().contains("native_binding_unavailable"));
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(f.owner.store().lock().unwrap().goal(goal).unwrap().unwrap()).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&task).unwrap(),
        serde_json::to_value(
            f.owner
                .store()
                .lock()
                .unwrap()
                .task(task.id)
                .unwrap()
                .unwrap()
        )
        .unwrap()
    );
    assert!(matches!(
        f.runtime
            .handle_control(&f.socket, f.request(ControlAction::RuntimeStatus))
            .await
            .unwrap(),
        ControlResponse::RuntimeMetadata {
            operational: false,
            service_running: true,
            ..
        }
    ));
    f.runtime.shutdown().await.unwrap();
    assert!(matches!(
        f.runtime
            .handle_control(&f.socket, f.request(ControlAction::RuntimeStatus))
            .await
            .unwrap(),
        ControlResponse::RuntimeMetadata {
            operational: false,
            service_running: false,
            ..
        }
    ));
    assert!(
        f.runtime
            .handle_control(
                &f.socket,
                f.request(ControlAction::CreateGoal {
                    project: f.project.id,
                    expected_project: f.project.version,
                    plan: plan()
                })
            )
            .await
            .is_err()
    );
}
