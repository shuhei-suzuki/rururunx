//! Read planning controls use real trusted ingress, never seeded phase authority.
use super::*;
use crate::{
    config::{AgentConfig, Config, WorkflowClass},
    domain::*,
    execution::RuntimeOwner,
    runtime::{
        Runtime,
        control::{ControlAction, ControlRequest, ControlResponse},
        goal::*,
    },
};
use std::sync::Arc;
use uuid::Uuid;

struct Fixture {
    _dir: tempfile::TempDir,
    owner: Arc<RuntimeOwner>,
    _runtime: Runtime,
    scope: Scope,
}
impl Fixture {
    async fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        let root = dir.path().join("source");
        std::fs::create_dir(&root).unwrap();
        let mut project = Project::new(
            "read-plan".into(),
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
        let mut config = Config::default();
        config.agents.insert(
            "worker".into(),
            AgentConfig {
                provider: Some("claude".into()),
                command: vec!["/bin/true".into()],
                ..Default::default()
            },
        );
        let runtime = Runtime::new(owner.clone(), config).unwrap();
        let (socket, _peer) = tokio::net::UnixStream::pair().unwrap();
        let request = ControlRequest {
            request_id: Uuid::new_v4(),
            instance: owner.instance_id().into(),
            epoch: owner.epoch(),
            action: ControlAction::CreateGoal {
                project: project.id,
                expected_project: project.version,
                plan: GoalPlan {
                    definition: GoalDefinition {
                        title: "read only".into(),
                        objective: "retain immutable pins".into(),
                        criteria: vec![CriterionDefinition {
                            id: "verified".into(),
                            description: "actual checks pass".into(),
                            evaluator: CriterionEvaluator::RequiredTasksVerified,
                        }],
                        constraints: vec![],
                        non_goals: vec![],
                        source_refs: vec![],
                    },
                    tasks: vec![TaskDefinition {
                        key: "one".into(),
                        title: "work".into(),
                        acceptance_criteria: vec!["immutable result".into()],
                        executor: "worker".into(),
                        reviewers: vec![],
                        workflow: WorkflowClass::Strict,
                        risk: RiskClass::R3,
                    }],
                    dependencies: vec![],
                },
            },
        };
        let goal = match runtime.handle_control(&socket, request).await.unwrap() {
            ControlResponse::GoalAccepted { goal, .. } => goal,
            _ => panic!("genuine trusted ingress prerequisite failed"),
        };
        let task = owner
            .store()
            .lock()
            .unwrap()
            .goal(goal)
            .unwrap()
            .unwrap()
            .dag
            .nodes[0];
        Self {
            _dir: dir,
            owner,
            _runtime: runtime,
            scope: Scope {
                project_id: project.id,
                goal_id: Some(goal),
                task_id: Some(task),
            },
        }
    }
}

#[tokio::test]
async fn coherent_scope_plan_works_while_writer_mutex_is_held_and_changes_nothing() {
    let f = Fixture::new().await;
    let shared = f.owner.store();
    let store = shared.lock().unwrap();
    let before = store.events(&f.scope, 0, 1000).unwrap().len();
    // A planner that reacquires SharedStore deadlocks here. It uses a separate
    // readonly connection and preserves the actual epoch instead of reopening.
    let plan = plan_scope(&f.owner, &f.scope).unwrap();
    assert_eq!(plan.task.parsed().scope(), f.scope);
    assert!(plan.workflow.is_none() && plan.context.is_none() && plan.locks.is_empty());
    plan.validate_current(&store.connection).unwrap();
    assert!(
        store.managed_phase_required(&f.scope).unwrap(),
        "accepted Goal must fence legacy launch even before Workflow exists"
    );
    assert_eq!(store.events(&f.scope, 0, 1000).unwrap().len(), before);
    assert!(store.execution_units(Some(&f.scope)).unwrap().is_empty());
    assert!(
        store
            .records(&f.scope, RecordKind::Session)
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn exact_original_body_detects_same_version_changes_and_index_aliases() {
    let f = Fixture::new().await;
    let plan = plan_scope(&f.owner, &f.scope).unwrap();
    let connection = crate::state::current_test_writer(f.owner.state_path()).unwrap();
    let original = plan.task.raw().to_owned();
    let mut changed: Task = serde_json::from_str(&original).unwrap();
    changed.title = "same-version drift".into();
    connection
        .execute(
            "UPDATE tasks SET body=?1 WHERE id=?2",
            rusqlite::params![
                serde_json::to_string(&changed).unwrap(),
                changed.id.to_string()
            ],
        )
        .unwrap();
    assert!(
        plan.validate_current(&connection).is_err(),
        "original exact bytes cannot be refreshed by a current read"
    );
    // Restore before checking independent body/index inconsistency.
    connection
        .execute(
            "UPDATE tasks SET body=?1 WHERE id=?2",
            rusqlite::params![original, changed.id.to_string()],
        )
        .unwrap();
    plan.validate_current(&connection).unwrap();
    changed.id = TaskId::new();
    connection
        .execute(
            "UPDATE tasks SET body=?1 WHERE id=?2",
            rusqlite::params![
                serde_json::to_string(&changed).unwrap(),
                f.scope.task_id.unwrap().to_string()
            ],
        )
        .unwrap();
    assert!(plan_scope(&f.owner, &f.scope).is_err());
    assert!(
        f.owner
            .store()
            .lock()
            .unwrap()
            .managed_phase_required(&f.scope)
            .is_err()
    );
    assert!(
        f.owner
            .store()
            .lock()
            .unwrap()
            .managed_phase_required(&Scope {
                task_id: Some(TaskId::new()),
                ..f.scope.clone()
            })
            .is_err()
    );
}

#[tokio::test]
async fn current_owner_epoch_is_not_recaptured_or_reopened() {
    let f = Fixture::new().await;
    let plan = plan_scope(&f.owner, &f.scope).unwrap();
    f.owner
        .store()
        .lock()
        .unwrap()
        .begin_execution_epoch()
        .unwrap();
    assert!(plan_scope(&f.owner, &f.scope).is_err());
    assert!(
        plan.validate_current(&f.owner.store().lock().unwrap().connection)
            .is_err()
    );
}

// These rows are temporary factual projection images in a rolled-back test TX.
// No Driver, Unit, input/native contract or private authority is manufactured.
#[tokio::test]
async fn initial_input_projection_requires_exact_post_head_and_preserves_original_checks() {
    let f = Fixture::new().await;
    let plan = plan_scope(&f.owner, &f.scope).unwrap();
    let mut task = plan.task.parsed().clone();
    task.version += 1;
    task.context_version = 1;
    task.revision = Some("a".repeat(40));
    let task_raw = serde_json::to_string(&task).unwrap();
    let mut workflow = Record::new(
        f.scope.clone(),
        RecordKind::Workflow,
        serde_json::json!({"historical_projection":true}),
    );
    workflow.version = 1;
    let workflow_raw = serde_json::to_string(&workflow).unwrap();
    let context = ContextVersion {
        scope: f.scope.clone(),
        version: 1,
        revision: "a".repeat(40),
        source_hashes: Default::default(),
        data: serde_json::json!({"historical_projection":true}),
    };
    let context_raw = serde_json::to_string(&context).unwrap();
    let owner_key = crate::state::context_owner(&f.scope).unwrap();
    let mut connection = crate::state::current_test_writer(f.owner.state_path()).unwrap();
    let tx = connection.transaction().unwrap();
    assert!(
        plan.validate_input_projection(
            &tx,
            &task,
            &task_raw,
            (&workflow, &workflow_raw),
            (&context, &context_raw)
        )
        .is_err()
    );
    tx.execute(
        "UPDATE tasks SET version=?1,body=?2 WHERE id=?3",
        rusqlite::params![task.version, task_raw, task.id.to_string()],
    )
    .unwrap();
    tx.execute("INSERT INTO records(id,kind,project_id,goal_id,task_id,version,body) VALUES(?1,'workflow',?2,?3,?4,?5,?6)",rusqlite::params![workflow.id.to_string(),task.project_id.to_string(),task.goal_id.to_string(),task.id.to_string(),workflow.version,workflow_raw]).unwrap();
    tx.execute("INSERT INTO context_versions(project_id,goal_id,task_id,owner,version,body) VALUES(?1,?2,?3,?4,1,?5)",rusqlite::params![task.project_id.to_string(),task.goal_id.to_string(),task.id.to_string(),owner_key,context_raw]).unwrap();
    plan.validate_input_projection(
        &tx,
        &task,
        &task_raw,
        (&workflow, &workflow_raw),
        (&context, &context_raw),
    )
    .unwrap();
    assert!(
        plan.validate_current(&tx).is_err(),
        "ordinary original CAS must NOT accept prescribed new input"
    );
    let mut foreign = context.clone();
    foreign.scope.task_id = Some(TaskId::new());
    assert!(
        plan.validate_input_projection(
            &tx,
            &task,
            &task_raw,
            (&workflow, &workflow_raw),
            (&foreign, &context_raw)
        )
        .is_err()
    );
    let mut changed = context.clone();
    changed.data = serde_json::json!({"different":true});
    let changed_raw = serde_json::to_string(&changed).unwrap();
    assert!(
        plan.validate_input_projection(
            &tx,
            &task,
            &task_raw,
            (&workflow, &workflow_raw),
            (&changed, &changed_raw)
        )
        .is_err(),
        "same-version Context bytes remain exact"
    );
    tx.execute("INSERT INTO context_versions(project_id,goal_id,task_id,owner,version,body) VALUES(?1,?2,?3,?4,2,?5)",rusqlite::params![task.project_id.to_string(),task.goal_id.to_string(),task.id.to_string(),owner_key,context_raw]).unwrap();
    assert!(
        plan.validate_input_projection(
            &tx,
            &task,
            &task_raw,
            (&workflow, &workflow_raw),
            (&context, &context_raw)
        )
        .is_err(),
        "later head cannot ratify first input"
    );
    tx.rollback().unwrap();
    plan.validate_current(&connection).unwrap();
    assert!(
        f.owner
            .store()
            .lock()
            .unwrap()
            .execution_units(Some(&f.scope))
            .unwrap()
            .is_empty()
    );
}
