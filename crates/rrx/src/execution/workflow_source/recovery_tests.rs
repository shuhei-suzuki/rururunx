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
    artifact_fixture(true).await
}
async fn artifact_fixture(publish: bool) -> Published {
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
    let artifact = if publish {
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
        artifact
    } else {
        // This Ready artifact is produced by actual retained capture of the same
        // owned native terminal. It has not won the Workflow publication TX.
        let done = owner
            .store
            .lock()
            .unwrap()
            .execution_unit(prepared.id)
            .unwrap();
        assert_eq!(done.work, Some(WorkOutcome::Success));
        let io = UnitGit::new(owner.clone(), &done, false).unwrap();
        let revision = io
            .text(&done.worktree, ["rev-parse", "HEAD"])
            .await
            .unwrap();
        let artifact = results::ResultStore::new(owner.clone())
            .capture(&done.authority(), &revision, BTreeMap::new())
            .await
            .unwrap();
        assert_eq!(artifact.state, ArtifactState::Ready);
        artifact
    };
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
    let released = Arc::downgrade(&f.owner);
    drop(f.owner);
    // Dropped registry ToolServer cancellation is processed asynchronously;
    // wait for that actual Rust owner to release the lock, not native death.
    tokio::time::timeout(Duration::from_secs(10), async {
        while released.strong_count() > 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
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
            .committed_input(
                project.clone(),
                task.clone(),
                Phase::Commit,
                WorkflowClass::Quick
            )
            .await
            .unwrap()
            .unwrap()
            .source
            .source_versions,
        f.artifact.dependencies
    );
    assert!(
        sources
            .take_initial_executor(
                &project,
                &task,
                Phase::Implement,
                &budget(WorkflowClass::Quick, Phase::Implement, &f.config)
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
    let sources = ManagedWorkflowSources::new(f.owner.clone(), f.config).unwrap();
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

#[tokio::test]
async fn actual_recovery_future_drop_invalidates_only_its_uninstalled_claim() {
    let f = published().await;
    let sources = Arc::new(ManagedWorkflowSources::new(f.owner.clone(), f.config).unwrap());
    let (reached, observed) = tokio::sync::oneshot::channel();
    let (_release, paused) = tokio::sync::oneshot::channel();
    let worker = sources.clone();
    let task_id = f.task;
    let recovering = tokio::spawn(async move {
        worker
            .recover_retained_inner(
                task_id,
                Some(RecoveryPause {
                    after_verify: false,
                    reached,
                    release: paused,
                }),
            )
            .await
    });
    tokio::time::timeout(Duration::from_secs(15), observed)
        .await
        .unwrap()
        .unwrap();
    let effects_before = f
        .owner
        .store
        .lock()
        .unwrap()
        .managed_effects(f.artifact.unit_id)
        .unwrap();
    recovering.abort();
    assert!(recovering.await.unwrap_err().is_cancelled());
    let (project, task) = owners(&f.owner, f.task);
    assert!(capture(&sources, project, task).await.is_err());
    assert_eq!(
        serde_json::to_value(
            f.owner
                .store
                .lock()
                .unwrap()
                .managed_effects(f.artifact.unit_id)
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(effects_before).unwrap()
    );
    assert_eq!(
        f.owner
            .store
            .lock()
            .unwrap()
            .result_artifact(f.artifact.id)
            .unwrap()
            .state,
        ArtifactState::Published
    );
    // Actual next reconstruction is possible; no row reconstructs an old capability.
    sources.recover_retained(f.task).await.unwrap();
    let (project, task) = owners(&f.owner, f.task);
    assert!(capture(&sources, project, task).await.is_ok());
}

#[tokio::test]
async fn final_recovery_acceptance_refuses_instruction_and_epoch_races() {
    for epoch_change in [false, true] {
        let f = published().await;
        let sources = Arc::new(ManagedWorkflowSources::new(f.owner.clone(), f.config).unwrap());
        let (reached, observed) = tokio::sync::oneshot::channel();
        let (release, paused) = tokio::sync::oneshot::channel();
        let worker = sources.clone();
        let task_id = f.task;
        let recovering = tokio::spawn(async move {
            worker
                .recover_retained_inner(
                    task_id,
                    Some(RecoveryPause {
                        after_verify: true,
                        reached,
                        release: paused,
                    }),
                )
                .await
        });
        tokio::time::timeout(Duration::from_secs(15), observed)
            .await
            .unwrap()
            .unwrap();
        if epoch_change {
            f.owner
                .store
                .lock()
                .unwrap()
                .begin_execution_epoch()
                .unwrap();
        } else {
            let (_, mut task) = owners(&f.owner, f.task);
            task.title = "post-read instruction drift".into();
            f.owner.store.lock().unwrap().put_task(&mut task).unwrap();
        }
        let effects = f
            .owner
            .store
            .lock()
            .unwrap()
            .managed_effects(f.artifact.unit_id)
            .unwrap();
        release.send(()).unwrap();
        assert!(
            recovering.await.unwrap().is_err(),
            "complete reads cannot ratify changed acceptance authority"
        );
        let (project, task) = owners(&f.owner, f.task);
        assert!(capture(&sources, project, task).await.is_err());
        assert_eq!(
            serde_json::to_value(
                f.owner
                    .store
                    .lock()
                    .unwrap()
                    .managed_effects(f.artifact.unit_id)
                    .unwrap()
            )
            .unwrap(),
            serde_json::to_value(effects).unwrap()
        );
        assert_eq!(
            f.owner
                .store
                .lock()
                .unwrap()
                .result_artifact(f.artifact.id)
                .unwrap()
                .state,
            ArtifactState::Published
        );
    }
}

#[tokio::test]
async fn recovery_rejects_bad_manifest_and_complete_caller_dto_drift() {
    use std::os::unix::fs::PermissionsExt;
    let f = published().await;
    let sources = ManagedWorkflowSources::new(f.owner.clone(), f.config).unwrap();
    let original = std::fs::read(&f.artifact.manifest).unwrap();
    let permissions = std::fs::metadata(&f.artifact.manifest)
        .unwrap()
        .permissions();
    std::fs::set_permissions(&f.artifact.manifest, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::write(&f.artifact.manifest, b"{}\n").unwrap();
    assert!(
        sources.recover_retained(f.task).await.is_err(),
        "corrupt retained manifest cannot install a frame"
    );
    let (project, task) = owners(&f.owner, f.task);
    assert!(
        capture(&sources, project.clone(), task.clone())
            .await
            .is_err()
    );
    std::fs::write(&f.artifact.manifest, original).unwrap();
    std::fs::set_permissions(&f.artifact.manifest, permissions).unwrap();
    sources.recover_retained(f.task).await.unwrap();
    let mut different_task = task.clone();
    different_task.blockers.push("caller-only metadata".into());
    let mut different_project = project.clone();
    different_project.name = "caller-only name".into();
    let effects = f
        .owner
        .store
        .lock()
        .unwrap()
        .managed_effects(f.artifact.unit_id)
        .unwrap();
    assert!(
        capture(&sources, project.clone(), different_task)
            .await
            .is_err()
    );
    assert!(
        capture(&sources, different_project, task.clone())
            .await
            .is_err()
    );
    assert_eq!(
        serde_json::to_value(
            f.owner
                .store
                .lock()
                .unwrap()
                .managed_effects(f.artifact.unit_id)
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(effects).unwrap(),
        "caller mismatch refuses before retained helpers"
    );
    assert!(capture(&sources, project, task).await.is_ok());
}

#[tokio::test]
async fn source_bound_inflight_helper_rejects_drift_epoch_and_future_drop() {
    use std::os::unix::fs::PermissionsExt;
    for mode in 0..3 {
        let f = published().await;
        let claim = f
            .owner
            .store
            .lock()
            .unwrap()
            .begin_retained_source_recovery(f.task, f.owner.epoch)
            .unwrap();
        let binding = claim.binding();
        let ready = f.dir.path().join("source-helper-ready");
        let release = f.dir.path().join("source-helper-release");
        let program = f.dir.path().join("held-reader.py");
        // Closed isolated fixture: no subprocesses/accounts/network; self-expiry
        // bounds the helper even if this control panics. This is not native death proof.
        std::fs::write(&program, format!(
            "#!/usr/bin/env python3\nimport pathlib,time\nr=pathlib.Path({:?})\nx=pathlib.Path({:?})\nr.write_text('ready')\nend=time.monotonic()+4\nwhile time.monotonic()<end and not x.exists(): time.sleep(0.01)\n",
            ready.to_str().unwrap(), release.to_str().unwrap()
        )).unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
        let io = RetainedGit::for_recovery(f.owner.clone(), &f.artifact, binding.clone())
            .unwrap()
            .with_program(program);
        let original = f
            .owner
            .store
            .lock()
            .unwrap()
            .managed_effects(f.artifact.unit_id)
            .unwrap();
        let helper = tokio::spawn(async move {
            io.run(["fsck", "--full", "--strict", "--no-dangling"])
                .await
        });
        wait_file(&ready).await;
        if mode == 0 {
            let (_, mut task) = owners(&f.owner, f.task);
            task.title = "during registered helper".into();
            f.owner.store.lock().unwrap().put_task(&mut task).unwrap();
        } else if mode == 1 {
            f.owner
                .store
                .lock()
                .unwrap()
                .begin_execution_epoch()
                .unwrap();
        } else {
            helper.abort();
        }
        std::fs::write(&release, "done").unwrap();
        if mode == 2 {
            assert!(helper.await.unwrap_err().is_cancelled());
        } else {
            assert!(
                tokio::time::timeout(Duration::from_secs(15), helper)
                    .await
                    .unwrap()
                    .unwrap()
                    .is_err()
            );
        }
        let after = f
            .owner
            .store
            .lock()
            .unwrap()
            .managed_effects(f.artifact.unit_id)
            .unwrap();
        let added = after
            .iter()
            .filter(|e| !original.iter().any(|old| old.id == e.id))
            .collect::<Vec<_>>();
        assert_eq!(added.len(), 1, "helper owns one actual registered intent");
        assert_eq!(added[0].kind, "retained_git");
        assert_eq!(
            added[0].state,
            EffectState::Unknown,
            "stale/dropped helper must not persist a successful source receipt"
        );
        f.owner
            .store
            .lock()
            .unwrap()
            .abandon_retained_source_recovery(&binding)
            .unwrap();
        assert_eq!(
            f.owner
                .store
                .lock()
                .unwrap()
                .result_artifact(f.artifact.id)
                .unwrap()
                .state,
            ArtifactState::Published
        );
    }
}

#[tokio::test]
async fn unpublished_ready_and_foreign_retained_dtos_never_select_recovery() {
    let f = artifact_fixture(false).await;
    let sources = ManagedWorkflowSources::new(f.owner.clone(), f.config).unwrap();
    let before = f
        .owner
        .store
        .lock()
        .unwrap()
        .managed_effects(f.artifact.unit_id)
        .unwrap();
    assert!(
        sources.recover_retained(f.task).await.is_err(),
        "Ready is not Published source selection"
    );
    assert_eq!(
        serde_json::to_value(
            f.owner
                .store
                .lock()
                .unwrap()
                .managed_effects(f.artifact.unit_id)
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(before).unwrap(),
        "Ready refusal precedes reconstruction helpers"
    );
    assert_eq!(
        f.owner
            .store
            .lock()
            .unwrap()
            .result_artifact(f.artifact.id)
            .unwrap()
            .state,
        ArtifactState::Ready
    );
    let (project, task) = owners(&f.owner, f.task);
    assert!(capture(&sources, project, task).await.is_err());

    let f = published().await;
    let claim = f
        .owner
        .store
        .lock()
        .unwrap()
        .begin_retained_source_recovery(f.task, f.owner.epoch)
        .unwrap();
    let before = f
        .owner
        .store
        .lock()
        .unwrap()
        .managed_effects(f.artifact.unit_id)
        .unwrap();
    for field in 0..3 {
        let mut foreign = f.artifact.clone();
        match field {
            0 => foreign.scope.project_id = ProjectId::new(),
            1 => foreign.scope.task_id = Some(TaskId::new()),
            _ => foreign.id = ArtifactId::new(),
        }
        assert!(
            RetainedGit::for_recovery(f.owner.clone(), &foreign, claim.binding()).is_err(),
            "foreign artifact DTO is negative input, never source authority"
        );
    }
    assert_eq!(
        serde_json::to_value(
            f.owner
                .store
                .lock()
                .unwrap()
                .managed_effects(f.artifact.unit_id)
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(before).unwrap()
    );
    f.owner
        .store
        .lock()
        .unwrap()
        .abandon_retained_source_recovery(&claim.binding())
        .unwrap();
}
