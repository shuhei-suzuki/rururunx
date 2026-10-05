//! Account-free controls using actual Workflow publication and source recovery.
use super::*;
use crate::{
    adapter::{AgentRegistry, SessionRef},
    config::AgentConfig,
    execution::{native, resources::ResourceManager, results},
    workflow::{StepResult, WorkflowEngine, budget},
};
use std::time::Duration;

struct Published {
    dir: tempfile::TempDir,
    owner: Arc<RuntimeOwner>,
    config: Config,
    task: TaskId,
    artifact: ResultArtifact,
}
async fn wait_file(path: &Path) {
    tokio::time::timeout(Duration::from_secs(15), async {
        while !path.exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}
async fn published() -> Published {
    let (dir, owner, seed) = results::tests::fixture().await;
    let program = native::tests::program(dir.path(), "codex");
    let mut config = Config {
        minimum_workflow: WorkflowClass::Quick,
        ..Default::default()
    };
    config.workflow.risk_mapping = [WorkflowClass::Quick; 4];
    config.agents.insert(
        "native-alias".into(),
        AgentConfig {
            provider: Some("codex".into()),
            command: vec![program.to_string_lossy().into()],
            ..Default::default()
        },
    );
    let root = owner
        .store
        .lock()
        .unwrap()
        .project(seed.project_id)
        .unwrap()
        .unwrap()
        .root;
    std::fs::write(root.join("rules.md"), "MANDATORY_RECOVERY_RULE_A\n").unwrap();
    std::fs::write(
        root.join("workflow.toml"),
        "minimum_workflow = \"QUICK\"\n[context]\nrepo_map_tokens = 321\n",
    )
    .unwrap();
    results::git(&root, ["add", "rules.md", "workflow.toml"])
        .await
        .unwrap();
    results::git(
        &root,
        [
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            "recovery A",
        ],
    )
    .await
    .unwrap();
    let task = {
        let mut store = owner.store.lock().unwrap();
        let mut project = store.project(seed.project_id).unwrap().unwrap();
        project.config_ref = Some(root.join("workflow.toml"));
        project.rule_refs = vec![root.join("rules.md")];
        store.put_project(&mut project).unwrap();
        let mut goal = store.goal(seed.goal_id).unwrap().unwrap();
        goal.state = GoalState::Running;
        store.put_goal(&mut goal).unwrap();
        let mut task = Task::new(
            seed.project_id,
            seed.goal_id,
            "answer".into(),
            "native-alias".into(),
        );
        task.workflow = WorkflowClass::Quick;
        task.acceptance_criteria = vec!["retain answer".into()];
        store.put_task(&mut task).unwrap();
        task
    };
    let registry = Arc::new(AgentRegistry::from_managed_config(&config, owner.clone()).unwrap());
    let sources = Arc::new(ManagedWorkflowSources::new(owner.clone(), config.clone()).unwrap());
    let gates = Arc::new(
        crate::execution::workflow_gates::ManagedWorkflowGates::new(owner.clone(), sources.clone())
            .unwrap(),
    );
    let engine = WorkflowEngine::new(
        owner.store(),
        registry.clone(),
        config.clone(),
        sources.clone(),
        gates,
    )
    .unwrap();
    let prepared = sources.prepare(task.id, "codex").await.unwrap();
    engine.initialize(task.id, None).await.unwrap();
    assert!(matches!(
        engine.step(task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Completed {
            phase: Phase::Worktree
        }
    ));
    assert!(matches!(
        engine.step(task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Started {
            phase: Phase::Implement,
            session: Some(_)
        }
    ));
    let wf = engine.snapshot(task.id).unwrap();
    let identity = wf.history[wf.active.unwrap()].execution.clone().unwrap();
    let unit = owner
        .store
        .lock()
        .unwrap()
        .execution_unit(prepared.id)
        .unwrap();
    let output = ResourceManager::new(owner.clone())
        .profile(&unit)
        .unwrap()
        .output;
    wait_file(&output.join("fixture-ready")).await;
    std::fs::write(output.join("fixture-release"), "done").unwrap();
    let adapter = registry.get("native-alias").unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            if adapter
                .status(SessionRef {
                    id: identity.session,
                    scope: identity.scope.clone(),
                    execution: Some(identity.clone()),
                })
                .await
                .unwrap()
                .terminal()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(matches!(
        engine.step(task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Completed {
            phase: Phase::Implement
        }
    ));
    let wf = engine.snapshot(task.id).unwrap();
    let artifact = owner
        .store
        .lock()
        .unwrap()
        .result_artifact(wf.sources.artifact.unwrap())
        .unwrap();
    assert_eq!(artifact.state, ArtifactState::Published);
    // The ordinary live Project copy cannot replace tracked committed rules.
    std::fs::write(root.join("rules.md"), "LIVE_B_MUST_NOT_REPLACE_A\n").unwrap();
    drop(adapter);
    drop(engine);
    drop(registry);
    drop(sources);
    Published {
        dir,
        owner,
        config,
        task: task.id,
        artifact,
    }
}
fn owners(owner: &Arc<RuntimeOwner>, task: TaskId) -> (Project, Task) {
    let store = owner.store.lock().unwrap();
    let task = store.task(task).unwrap().unwrap();
    let project = store.project(task.project_id).unwrap().unwrap();
    (project, task)
}
async fn capture(
    sources: &ManagedWorkflowSources,
    project: Project,
    task: Task,
) -> Result<SourceSnapshot> {
    sources
        .capture(
            project,
            task,
            Phase::Commit,
            budget(WorkflowClass::Quick, Phase::Commit, &Config::default()),
        )
        .await
}
#[tokio::test]
async fn published_recovery_reopens_exact_frame_and_advances_only_typed_workflow_pins() {
    let f = published().await;
    let old_epoch = f.owner.epoch;
    let path = f.dir.path().join("state.db");
    drop(f.owner);
    let owner = RuntimeOwner::open(&path).unwrap();
    assert!(owner.epoch > old_epoch);
    let sources = Arc::new(ManagedWorkflowSources::new(owner.clone(), f.config.clone()).unwrap());
    let (project, task) = owners(&owner, f.task);
    assert!(
        capture(&sources, project.clone(), task.clone())
            .await
            .is_err()
    );
    sources.recover_retained(f.task).await.unwrap();
    let input = capture(&sources, project.clone(), task.clone())
        .await
        .unwrap();
    assert_eq!(input.artifact, Some(f.artifact.id));
    assert_eq!(input.revision, f.artifact.revision);
    assert!(input.payload.contains("MANDATORY_RECOVERY_RULE_A"));
    assert!(!input.payload.contains("LIVE_B_MUST_NOT_REPLACE_A"));
    assert_eq!(input.source_versions, f.artifact.dependencies);
    assert_eq!(
        sources
            .committed_input(project.clone(), task.clone())
            .await
            .unwrap()
            .unwrap()
            .rule_text
            .trim(),
        "Mandatory Project rule rules.md:\nMANDATORY_RECOVERY_RULE_A"
    );
    assert!(
        sources
            .take_initial_executor(
                project.clone(),
                task.clone(),
                Phase::Commit,
                budget(WorkflowClass::Quick, Phase::Commit, &f.config)
            )
            .await
            .unwrap()
            .is_none()
    );
    let registry = Arc::new(AgentRegistry::from_managed_config(&f.config, owner.clone()).unwrap());
    let gates = Arc::new(
        crate::execution::workflow_gates::ManagedWorkflowGates::new(owner.clone(), sources.clone())
            .unwrap(),
    );
    let engine =
        WorkflowEngine::new(owner.store(), registry, f.config, sources.clone(), gates).unwrap();
    assert!(matches!(
        engine.step(f.task, BTreeMap::new()).await.unwrap(),
        StepResult::Completed {
            phase: Phase::Commit
        }
    ));
    let (project, task) = owners(&owner, f.task);
    assert!(
        capture(&sources, project, task).await.is_ok(),
        "typed Workflow transition must advance exact pins"
    );
    let mut stale = owner.store.lock().unwrap().task(f.task).unwrap().unwrap();
    stale.title = "generic instruction drift".into();
    owner.store.lock().unwrap().put_task(&mut stale).unwrap();
    let (project, task) = owners(&owner, f.task);
    assert!(
        capture(&sources, project, task).await.is_err(),
        "generic Task write must not ratify old frame"
    );
    assert!(engine.step(f.task, BTreeMap::new()).await.is_err());
    // Conservative cancellation must remain available and cannot regrant pins.
    engine.cancel(f.task, "test completed".into()).unwrap();
}
#[tokio::test]
async fn source_claim_checks_helper_admission_competition_and_full_dto_currency() {
    let f = published().await;
    let sources = ManagedWorkflowSources::new(f.owner.clone(), f.config);
    let claim = f
        .owner
        .store
        .lock()
        .unwrap()
        .begin_retained_source_recovery(f.task, f.owner.epoch)
        .unwrap();
    assert!(
        f.owner
            .store
            .lock()
            .unwrap()
            .begin_retained_source_recovery(f.task, f.owner.epoch)
            .is_err()
    );
    let binding = claim.binding();
    let io = RetainedGit::for_recovery(f.owner.clone(), &f.artifact, binding.clone()).unwrap();
    let (project, mut task) = owners(&f.owner, f.task);
    task.title = "stale full body".into();
    f.owner.store.lock().unwrap().put_task(&mut task).unwrap();
    let before = f
        .owner
        .store
        .lock()
        .unwrap()
        .managed_effects(f.artifact.unit_id)
        .unwrap()
        .len();
    assert!(
        io.run(["fsck", "--full", "--strict", "--no-dangling"])
            .await
            .is_err()
    );
    assert_eq!(
        f.owner
            .store
            .lock()
            .unwrap()
            .managed_effects(f.artifact.unit_id)
            .unwrap()
            .len(),
        before,
        "stale source helper must refuse before effect intent"
    );
    f.owner
        .store
        .lock()
        .unwrap()
        .abandon_retained_source_recovery(&binding)
        .unwrap();
    assert!(
        sources.recover_retained(f.task).await.is_err(),
        "changed instructions cannot recover original frame"
    );
    assert!(capture(&sources, project, task).await.is_err());
}
