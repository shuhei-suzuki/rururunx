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
