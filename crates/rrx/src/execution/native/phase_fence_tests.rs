//! Genuine accepted ingress, then attempted public-ID Native bypass. These
//! negative controls do not create a phase owner or certify an admitted start.
use super::*;
use crate::{
    adapter::{AgentRegistry, InputKind, LaunchMode, LaunchRequest},
    config::{AgentConfig, Config, WorkflowClass},
    domain::{Project, RiskClass, Scope},
    runtime::{
        Runtime,
        control::{ControlAction, ControlRequest, ControlResponse},
        goal::*,
    },
};
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn genuine_accepted_goal_cannot_enter_legacy_native_before_private_phase_exists() {
    for provider in ["claude", "codex"] {
        for entry in ["direct", "adapter"] {
            let dir = tempfile::tempdir().unwrap();
            let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
            let root = dir.path().join("project");
            std::fs::create_dir(&root).unwrap();
            let mut project = Project::new(
                "protected-native".into(),
                root.canonicalize().unwrap(),
                "account-free-no-source-proof".into(),
                "main".into(),
            );
            owner
                .store
                .lock()
                .unwrap()
                .put_project(&mut project)
                .unwrap();
            let counter = dir.path().join("must-not-run");
            let program = dir.path().join("counter-native");
            std::fs::write(
                &program,
                format!(
                    "#!/bin/sh\nprintf invoked > '{}'\nexit 7\n",
                    counter.display()
                ),
            )
            .unwrap();
            std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
            let mut config = Config::default();
            config.agents.insert(
                "selected".into(),
                AgentConfig {
                    provider: Some(provider.into()),
                    command: vec![program.to_string_lossy().into()],
                    ..Default::default()
                },
            );
            // These remain standalone legacy-refusal consumers. Their own
            // server deliberately occupies IPC before production installation.
            let sessions = (entry == "direct").then(|| NativeSessions::new(owner.clone()).unwrap());
            let registry = (entry == "adapter")
                .then(|| AgentRegistry::from_managed_config(&config, owner.clone()).unwrap());
            let runtime = Arc::new(Runtime::new(owner.clone(), config.clone()).unwrap());
            let (socket, _peer) = tokio::net::UnixStream::pair().unwrap();
            let response = runtime
                .handle_control(
                    &socket,
                    ControlRequest {
                        request_id: uuid::Uuid::new_v4(),
                        instance: owner.instance_id().into(),
                        epoch: owner.epoch(),
                        action: ControlAction::CreateGoal {
                            project: project.id,
                            expected_project: project.version,
                            plan: GoalPlan {
                                definition: GoalDefinition {
                                    title: "protected".into(),
                                    objective: "preserve original phase".into(),
                                    criteria: vec![CriterionDefinition {
                                        id: "verified".into(),
                                        description: "verified result".into(),
                                        evaluator: CriterionEvaluator::RequiredTasksVerified,
                                    }],
                                    constraints: vec![],
                                    non_goals: vec![],
                                    source_refs: vec![],
                                },
                                tasks: vec![TaskDefinition {
                                    key: "one".into(),
                                    title: "owned only".into(),
                                    acceptance_criteria: vec!["exact phase".into()],
                                    executor: "selected".into(),
                                    reviewers: vec![],
                                    workflow: WorkflowClass::Strict,
                                    risk: RiskClass::R3,
                                }],
                                dependencies: vec![],
                            },
                        },
                    },
                )
                .await
                .unwrap();
            let ControlResponse::GoalAccepted { goal, .. } = response else {
                panic!("genuine accepted ingress prerequisite failed")
            };
            let task = {
                let store = owner.store.lock().unwrap();
                let task_id = store.goal(goal).unwrap().unwrap().dag.nodes[0];
                store.task(task_id).unwrap().unwrap()
            };
            assert!(
                runtime
                    .installed_driver_composition(&task)
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("managed Registry installation refused")
            );
            let scope: Scope = task.scope();
            let input = ManagedInput {
                agent: "selected".into(),
                // Public caller IDs intentionally carry no allocated phase or Unit.
                authority: ExecutionAuthority {
                    scope: scope.clone(),
                    unit_id: UnitId::new(),
                    generation: 1,
                    owner_epoch: owner.epoch(),
                    session_id: None,
                    record_version: 1,
                },
                artifact: None,
                input: PreparedInput {
                    scope: scope.clone(),
                    kind: InputKind::ContextPack,
                    revision: "a".repeat(40),
                    version: 1,
                    source_versions: BTreeMap::new(),
                    payload: "must refuse without private phase".into(),
                },
            };
            let before = owner
                .store
                .lock()
                .unwrap()
                .events(&scope, 0, 1000)
                .unwrap()
                .len();
            if entry == "direct" {
                let sessions = sessions.as_ref().unwrap();
                let error = sessions
                    .start_inner(input.clone(), None, None, Some(program.clone()))
                    .await
                    .err()
                    .expect("protected direct entry accepted");
                assert_eq!(
                    error.downcast_ref::<NativeFailure>(),
                    Some(&NativeFailure::AuthorityUnavailable)
                );
            } else {
                let registry = registry.as_ref().unwrap();
                let request = LaunchRequest {
                    project: project.clone(),
                    scope: scope.clone(),
                    worktree: project.root.clone(),
                    role: SessionRole::Executor,
                    mode: LaunchMode::NonInteractive,
                    input: input.input.clone(),
                    environment: BTreeMap::new(),
                    model: None,
                    effort: None,
                };
                let error = registry
                    .get("selected")
                    .unwrap()
                    .start_managed(request, input)
                    .await
                    .err()
                    .expect("protected adapter entry accepted");
                assert_eq!(error.kind, ErrorKind::OwnershipMismatch);
            }
            assert!(
                !counter.exists(),
                "legacy entry started a version/native executable"
            );
            let store = owner.store.lock().unwrap();
            assert!(store.execution_units(Some(&scope)).unwrap().is_empty());
            assert!(
                store
                    .records(&scope, crate::domain::RecordKind::Session)
                    .unwrap()
                    .is_empty()
            );
            assert_eq!(store.task(task.id).unwrap().unwrap().version, task.version);
            assert_eq!(store.events(&scope, 0, 1000).unwrap().len(), before);
        }
    }
}
