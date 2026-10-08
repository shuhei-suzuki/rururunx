//! Account-free controls through the production committed source port and Workflow.
//! Only the early evidence gates and the local native protocol peer are fixtures.
use super::*;
use crate::execution::{self, WorkOutcome, results, workflow_source::ManagedWorkflowSources};
use sha2::{Digest, Sha256};
use std::{path::Path, time::Duration};

const RULE_A: &str = "MANDATORY_COMMITTED_RULE_A: preserve the answer.\n";
const RULE_B: &str = "MANDATORY_LIVE_RULE_B: different instructions.\n";

struct InitialEvidence {
    owner: Arc<execution::RuntimeOwner>,
}
impl PhaseGates for InitialEvidence {
    fn complete(
        &self,
        invocation: PhaseInvocation,
        transport: Option<SessionStatus>,
    ) -> WorkflowFuture<'_, GateOutcome> {
        Box::pin(async move {
            let marker = if invocation.phase.actor() == Actor::Executor {
                let native = transport
                    .as_ref()
                    .context("fixture native terminal missing")?
                    .execution
                    .as_ref()
                    .context("fixture managed execution missing")?;
                ensure!(
                    native.work == Some(WorkOutcome::Success),
                    "fixture requires owned successful work"
                );
                let id = invocation
                    .sources
                    .artifact
                    .context("fixture requires real retained artifact")?;
                let artifact = self.owner.store.lock().unwrap().result_artifact(id)?;
                ensure!(
                    artifact.unit_id == native.handle.unit
                        && artifact.revision == invocation.sources.revision,
                    "fixture evidence must match actual native work and captured revision"
                );
                format!("rrx-artifact:{id}")
            } else {
                ensure!(
                    matches!(invocation.phase, Phase::Issue | Phase::Worktree)
                        && transport.is_none(),
                    "fixture only attests initial evidence and the first successful Executor"
                );
                "fixture-initial-evidence".into()
            };
            Ok(GateOutcome::Passed(Evidence {
                scope: invocation.task.scope(),
                phase: invocation.phase,
                revision: invocation.sources.revision,
                source_versions: invocation.sources.source_versions.clone(),
                artifacts: vec![marker],
                dependencies: invocation.sources.source_versions,
                review_approved: None,
                session_id: transport.as_ref().map(|s| s.session.id),
                context_version: invocation.context.version,
            }))
        })
    }
}

struct Fixture {
    _dir: crate::runtime::LegacyFixture,
    owner: Arc<execution::RuntimeOwner>,
    task: Task,
    config: Config,
    registry: Arc<AgentRegistry>,
    sources: Arc<ManagedWorkflowSources>,
    config_a: String,
    project_root: std::path::PathBuf,
}
impl Fixture {
    fn production_engine(&self, control: &'static str) -> WorkflowEngine {
        {
            let mut store = self.owner.store.lock().unwrap();
            let mut task = store.task(self.task.id).unwrap().unwrap();
            task.acceptance_criteria = vec!["retain the committed answer".into()];
            store.put_task(&mut task).unwrap();
        }
        WorkflowEngine::new(
            self.owner.store(),
            self.registry.clone(),
            self.config.clone(),
            self.sources.clone(),
            Arc::new(ProductionControl {
                gate: execution::workflow_gates::ManagedWorkflowGates::new(
                    self.owner.clone(),
                    self.sources.clone(),
                )
                .unwrap(),
                owner: self.owner.clone(),
                control,
            }),
        )
        .unwrap()
    }
    async fn new(provider: &str, class: WorkflowClass) -> Self {
        let (dir, owner, seed_task) = results::tests::fixture().await;
        let mut task = Task::new(
            seed_task.project_id,
            seed_task.goal_id,
            "committed source fixture".into(),
            "native-alias".into(),
        );
        let program = execution::native::tests::program(dir.path(), provider);
        let script = std::fs::read_to_string(&program).unwrap();
        // Record actual outgoing native bytes before the fixture's optional scenario rewrite.
        let record = "    value = json.loads(line)\n    if value.get('method') == 'turn/start' or value.get('type') == 'user':\n        with open(os.path.join(os.environ['RRX_OUTPUT_DIR'], 'fixture-native-input.json'), 'w') as recorded:\n            json.dump(value, recorded)\n";
        assert!(script.contains("    value = json.loads(line)\n"));
        std::fs::write(
            &program,
            script.replacen("    value = json.loads(line)\n", record, 1),
        )
        .unwrap();
        let mut config = Config {
            minimum_workflow: WorkflowClass::Quick,
            ..Default::default()
        };
        config.workflow.risk_mapping = [WorkflowClass::Quick; 4];
        config.agents.insert(
            "native-alias".into(),
            crate::config::AgentConfig {
                provider: Some(provider.into()),
                command: vec![program.to_string_lossy().into()],
                ..Default::default()
            },
        );
        task.executor = "native-alias".into();
        task.workflow = class;
        let root = owner
            .store
            .lock()
            .unwrap()
            .project(task.project_id)
            .unwrap()
            .unwrap()
            .root;
        let class_name = match class {
            WorkflowClass::Quick => "QUICK",
            WorkflowClass::Standard => "STANDARD",
            WorkflowClass::Strict => "STRICT",
        };
        let config_a =
            format!("minimum_workflow = \"{class_name}\"\n[context]\nrepo_map_tokens = 321\n");
        std::fs::write(root.join("workflow.toml"), &config_a).unwrap();
        std::fs::write(root.join("rules.md"), RULE_A).unwrap();
        results::git(&root, ["add", "workflow.toml", "rules.md"])
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
                "committed input A",
            ],
        )
        .await
        .unwrap();
        {
            let mut store = owner.store.lock().unwrap();
            let mut project = store.project(task.project_id).unwrap().unwrap();
            project.config_ref = Some(root.join("workflow.toml"));
            project.rule_refs = vec![root.join("rules.md")];
            store.put_project(&mut project).unwrap();
            store.put_task(&mut task).unwrap();
        }
        let registry =
            Arc::new(AgentRegistry::from_managed_config(&config, owner.clone()).unwrap());
        let sources = Arc::new(ManagedWorkflowSources::new(owner.clone(), config.clone()).unwrap());
        Self {
            _dir: dir,
            owner,
            task,
            config,
            registry,
            sources,
            config_a,
            project_root: root,
        }
    }
    fn engine(&self, sources: Arc<dyn WorkflowSources>) -> WorkflowEngine {
        WorkflowEngine::new(
            self.owner.store(),
            self.registry.clone(),
            self.config.clone(),
            sources,
            Arc::new(InitialEvidence {
                owner: self.owner.clone(),
            }),
        )
        .unwrap()
    }
    fn live_b(&self) {
        std::fs::write(
            self.project_root.join("workflow.toml"),
            "minimum_workflow = \"STRICT\"\n[context]\nrepo_map_tokens = 777\n",
        )
        .unwrap();
        std::fs::write(self.project_root.join("rules.md"), RULE_B).unwrap();
    }
    fn assert_context_a(&self, version: u64, base: &str) -> ContextVersion {
        let context = self
            .owner
            .store
            .lock()
            .unwrap()
            .context(&self.task.scope(), Some(version))
            .unwrap()
            .unwrap();
        assert_eq!(context.revision, base);
        assert_eq!(
            context.source_hashes["rules:config"],
            format!("{:x}", Sha256::digest(self.config_a.as_bytes()))
        );
        assert_eq!(
            context.source_hashes["rules:rules.md"],
            format!("{:x}", Sha256::digest(RULE_A.as_bytes()))
        );
        assert_eq!(context.data["budget"]["discretionary_tokens"], 321);
        let payload = context.data["payload"].as_str().unwrap();
        assert!(payload.contains(RULE_A.trim()));
        assert!(!payload.contains(RULE_B.trim()));
        context
    }
    fn assert_no_native(&self, id: execution::UnitId) {
        let store = self.owner.store.lock().unwrap();
        let unit = store.execution_unit(id).unwrap();
        assert!(unit.session_id.is_none() && unit.artifact_id.is_none());
        assert!(
            store
                .result_artifacts(&self.task.scope())
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .records(&self.task.scope(), RecordKind::Session)
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .managed_effects(id)
                .unwrap()
                .iter()
                .all(|e| e.kind == "git_helper")
        );
        assert!(
            store
                .quota_observations(&unit.provider, "unknown")
                .unwrap()
                .is_empty()
        );
    }
}

struct ProductionControl {
    gate: execution::workflow_gates::ManagedWorkflowGates,
    owner: Arc<execution::RuntimeOwner>,
    control: &'static str,
}
impl PhaseGates for ProductionControl {
    fn complete(
        &self,
        invocation: PhaseInvocation,
        mut transport: Option<SessionStatus>,
    ) -> WorkflowFuture<'_, GateOutcome> {
        Box::pin(async move {
            if invocation.phase == Phase::Implement {
                if self.control == "terminal" {
                    transport.as_mut().unwrap().execution.as_mut().unwrap().work =
                        Some(WorkOutcome::Failure);
                }
                if self.control == "manifest" {
                    let artifact = self
                        .owner
                        .store
                        .lock()
                        .unwrap()
                        .result_artifact(invocation.sources.artifact.unwrap())
                        .unwrap();
                    std::fs::write(artifact.manifest, "corrupt").unwrap();
                }
            }
            self.gate.complete(invocation, transport).await
        })
    }
}

#[tokio::test]
async fn production_gates_observe_real_preparation_native_result_and_commit_but_wait_for_tests() {
    for provider in ["claude", "codex"] {
        let f = Fixture::new(provider, WorkflowClass::Quick).await;
        let engine = f.production_engine("pass");
        let prepared = f.sources.prepare(f.task.id, provider).await.unwrap();
        engine.initialize(f.task.id, None).await.unwrap();
        early_phases(&engine, f.task.id, WorkflowClass::Quick).await;
        f.assert_no_native(prepared.id);
        assert!(matches!(
            engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Started {
                phase: Phase::Implement,
                session: Some(_)
            }
        ));
        let snapshot = engine.snapshot(f.task.id).unwrap();
        let identity = snapshot.history[snapshot.active.unwrap()]
            .execution
            .as_ref()
            .unwrap()
            .clone();
        assert_eq!(identity.unit, prepared.id);
        let current = f
            .owner
            .store
            .lock()
            .unwrap()
            .execution_unit(identity.unit)
            .unwrap();
        let output = execution::resources::ResourceManager::new(f.owner.clone())
            .profile(&current)
            .unwrap()
            .output;
        wait_file(&output.join("fixture-ready")).await;
        std::fs::write(output.join("fixture-release"), "done").unwrap();
        let adapter = f.registry.get("native-alias").unwrap();
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
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        assert!(matches!(
            engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Completed {
                phase: Phase::Implement
            }
        ));
        assert!(matches!(
            engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Completed {
                phase: Phase::Commit
            }
        ));
        assert!(matches!(
            engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Waiting {
                phase: Phase::Tests,
                ..
            }
        ));
        let artifacts = {
            let store = f.owner.store.lock().unwrap();
            let artifacts = store.result_artifacts(&f.task.scope()).unwrap();
            assert_eq!(artifacts.len(), 1);
            assert_eq!(artifacts[0].state, execution::ArtifactState::Published);
            let receipts = store
                .records(&f.task.scope(), RecordKind::Verification)
                .unwrap();
            assert_eq!(receipts.len(), 3);
            for record in receipts {
                assert_eq!(record.data["schema"], "managed_workflow_gate_v1");
                assert_eq!(
                    record.data["context_data_sha256"].as_str().unwrap().len(),
                    64
                );
                assert_eq!(store.record(record.id).unwrap().unwrap().data, record.data);
            }
            artifacts
        };
        let reopened = Store::open(&f._dir.path().join("state.db")).unwrap();
        assert_eq!(
            reopened
                .records(&f.task.scope(), RecordKind::Verification)
                .unwrap()
                .len(),
            3
        );
        drop(reopened);
        // A survivor can alter its abandoned workspace; commit evidence stays pinned.
        std::fs::write(prepared.worktree.join("answer.txt"), "survivor\n").unwrap();
        results::ResultStore::new(f.owner.clone())
            .verify(&artifacts[0])
            .await
            .unwrap();
        engine.cancel(f.task.id, "fixture complete".into()).unwrap();
    }
}

#[tokio::test]
async fn production_initial_gates_refuse_dirty_preparation_and_wait_for_missing_specification() {
    for control in ["dirty", "spec", "standard"] {
        let class = if control == "standard" {
            WorkflowClass::Standard
        } else {
            WorkflowClass::Quick
        };
        let f = Fixture::new("codex", class).await;
        let engine = f.production_engine("pass");
        if control == "spec" {
            let mut store = f.owner.store.lock().unwrap();
            let mut task = store.task(f.task.id).unwrap().unwrap();
            task.acceptance_criteria.clear();
            store.put_task(&mut task).unwrap();
        }
        let prepared = f.sources.prepare(f.task.id, "codex").await.unwrap();
        engine.initialize(f.task.id, None).await.unwrap();
        if control == "dirty" {
            std::fs::write(prepared.worktree.join("answer.txt"), "dirty\n").unwrap();
        }
        let result = engine.step(f.task.id, BTreeMap::new()).await.unwrap();
        if control == "standard" {
            assert!(matches!(
                result,
                StepResult::Completed {
                    phase: Phase::Issue
                }
            ));
            early_phases(&engine, f.task.id, WorkflowClass::Quick).await;
        } else {
            assert!(
                matches!(
                    result,
                    StepResult::Waiting {
                        phase: Phase::Worktree,
                        ..
                    }
                ),
                "{control}: {result:?}"
            );
            assert!(
                f.owner
                    .store
                    .lock()
                    .unwrap()
                    .records(&f.task.scope(), RecordKind::Verification)
                    .unwrap()
                    .is_empty()
            );
        }
        f.assert_no_native(prepared.id);
        engine.cancel(f.task.id, "fixture complete".into()).unwrap();
    }
}

#[tokio::test]
async fn production_gate_rejects_forged_success_and_corrupted_retained_manifest() {
    for control in ["terminal", "manifest"] {
        let f = Fixture::new("codex", WorkflowClass::Quick).await;
        let engine = f.production_engine(control);
        let prepared = f.sources.prepare(f.task.id, "codex").await.unwrap();
        engine.initialize(f.task.id, None).await.unwrap();
        early_phases(&engine, f.task.id, WorkflowClass::Quick).await;
        engine.step(f.task.id, BTreeMap::new()).await.unwrap();
        let snapshot = engine.snapshot(f.task.id).unwrap();
        let identity = snapshot.history[snapshot.active.unwrap()]
            .execution
            .as_ref()
            .unwrap()
            .clone();
        let current = f
            .owner
            .store
            .lock()
            .unwrap()
            .execution_unit(prepared.id)
            .unwrap();
        let output = execution::resources::ResourceManager::new(f.owner.clone())
            .profile(&current)
            .unwrap()
            .output;
        wait_file(&output.join("fixture-ready")).await;
        std::fs::write(output.join("fixture-release"), "done").unwrap();
        let adapter = f.registry.get("native-alias").unwrap();
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
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        let result = engine.step(f.task.id, BTreeMap::new()).await.unwrap();
        assert!(
            matches!(
                result,
                StepResult::Waiting {
                    phase: Phase::Implement,
                    ..
                }
            ),
            "{control}: {result:?}"
        );
        let snapshot = engine.snapshot(f.task.id).unwrap();
        assert!(!snapshot.completed.contains_key(&Phase::Implement));
        assert!(
            f.owner
                .store
                .lock()
                .unwrap()
                .result_artifacts(&f.task.scope())
                .unwrap()
                .iter()
                .all(|a| a.state != execution::ArtifactState::Published)
        );
        engine.cancel(f.task.id, "fixture complete".into()).unwrap();
    }
}

async fn wait_file(path: &Path) {
    tokio::time::timeout(Duration::from_secs(15), async {
        while !path.exists() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
}
async fn early_phases(engine: &WorkflowEngine, task: TaskId, class: WorkflowClass) {
    let expected: &[Phase] = if class == WorkflowClass::Quick {
        &[Phase::Worktree]
    } else {
        &[Phase::Issue, Phase::Worktree]
    };
    for &phase in expected {
        assert!(
            matches!(engine.step(task, BTreeMap::new()).await.unwrap(), StepResult::Completed { phase: actual } if actual == phase)
        );
    }
}

#[tokio::test]
async fn committed_sources_require_prepare_before_initialize_and_refuse_foreign_scope() {
    let f = Fixture::new("codex", WorkflowClass::Quick).await;
    let engine = f.engine(f.sources.clone());
    let error = engine.initialize(f.task.id, None).await.unwrap_err();
    assert!(format!("{error:#}").contains("source preparation required"));
    assert!(
        f.owner
            .store
            .lock()
            .unwrap()
            .execution_units(Some(&f.task.scope()))
            .unwrap()
            .is_empty()
    );
    let before = f
        .owner
        .store
        .lock()
        .unwrap()
        .task(f.task.id)
        .unwrap()
        .unwrap()
        .version;
    let prepared = f.sources.prepare(f.task.id, "codex").await.unwrap();
    let after = f
        .owner
        .store
        .lock()
        .unwrap()
        .task(f.task.id)
        .unwrap()
        .unwrap()
        .version;
    assert!(after > before);
    let workflow = engine.initialize(f.task.id, None).await.unwrap();
    assert_eq!(workflow.sources.revision, prepared.base_sha);
    assert!(workflow.sources.artifact.is_none());
    f.assert_no_native(prepared.id);
    let project = f
        .owner
        .store
        .lock()
        .unwrap()
        .project(f.task.project_id)
        .unwrap()
        .unwrap();
    let mut foreign = f.task.clone();
    foreign.goal_id = GoalId::new();
    assert!(
        f.sources
            .capture(
                project,
                foreign,
                Phase::Worktree,
                budget(WorkflowClass::Quick, Phase::Worktree, &f.config)
            )
            .await
            .is_err()
    );
    f.assert_no_native(prepared.id);
    engine.cancel(f.task.id, "fixture complete".into()).unwrap();
}

#[tokio::test]
async fn actual_workflow_uses_committed_rules_policy_and_adopts_same_initial_unit() {
    for provider in ["claude", "codex"] {
        for class in [
            WorkflowClass::Quick,
            WorkflowClass::Standard,
            WorkflowClass::Strict,
        ] {
            let f = Fixture::new(provider, class).await;
            // B exists even before preparation: the registered Git producer must consume A.
            f.live_b();
            let prepared = f.sources.prepare(f.task.id, provider).await.unwrap();
            let engine = f.engine(f.sources.clone());
            let initialized = engine.initialize(f.task.id, None).await.unwrap();
            assert_eq!(
                initialized.workflow, class,
                "live B must not change selected policy"
            );
            f.assert_context_a(initialized.context_version, &prepared.base_sha);
            f.assert_no_native(prepared.id);
            early_phases(&engine, f.task.id, class).await;
            f.assert_no_native(prepared.id);
            let started = engine.step(f.task.id, BTreeMap::new()).await;
            assert!(
                matches!(
                    started,
                    Ok(StepResult::Started {
                        session: Some(_),
                        ..
                    })
                ),
                "{provider}/{class:?}: {started:?}; helpers: {:?}",
                f.owner
                    .store
                    .lock()
                    .unwrap()
                    .managed_effects(prepared.id)
                    .unwrap()
                    .iter()
                    .map(|e| (&e.kind, &e.state, &e.receipt))
                    .collect::<Vec<_>>()
            );
            let snapshot = engine.snapshot(f.task.id).unwrap();
            let attempt = &snapshot.history[snapshot.active.unwrap()];
            let identity = attempt.execution.as_ref().unwrap();
            assert_eq!(
                attempt.phase,
                if class == WorkflowClass::Quick {
                    Phase::Implement
                } else {
                    Phase::Requirements
                }
            );
            assert_eq!(identity.unit, prepared.id);
            assert_eq!(identity.generation, prepared.generation);
            let current = f
                .owner
                .store
                .lock()
                .unwrap()
                .execution_unit(identity.unit)
                .unwrap();
            assert_eq!(current.worktree, prepared.worktree);
            assert_eq!(current.branch, prepared.branch);
            assert_eq!(current.profile_digest, prepared.profile_digest);
            assert_eq!(
                f.owner
                    .store
                    .lock()
                    .unwrap()
                    .execution_units(Some(&f.task.scope()))
                    .unwrap()
                    .len(),
                1
            );
            let context = f.assert_context_a(attempt.context_version, &prepared.base_sha);
            let output = execution::resources::ResourceManager::new(f.owner.clone())
                .profile(&current)
                .unwrap()
                .output;
            wait_file(&output.join("fixture-ready")).await;
            let wire: serde_json::Value = serde_json::from_slice(
                &std::fs::read(output.join("fixture-native-input.json")).unwrap(),
            )
            .unwrap();
            let payload = if provider == "codex" {
                wire["params"]["input"][0]["text"].as_str().unwrap()
            } else {
                wire["message"]["content"].as_str().unwrap()
            };
            assert_eq!(payload, serde_json::to_string(&context.data).unwrap());
            assert!(payload.contains(RULE_A.trim()) && !payload.contains(RULE_B.trim()));
            assert_eq!(
                f.owner
                    .store
                    .lock()
                    .unwrap()
                    .managed_effects(current.id)
                    .unwrap()
                    .iter()
                    .filter(|e| e.kind == "native_input")
                    .count(),
                1
            );
            std::fs::write(output.join("fixture-release"), "complete").unwrap();
            let adapter = f.registry.get("native-alias").unwrap();
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
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            })
            .await
            .unwrap();
            assert!(
                matches!(engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
                StepResult::Completed { phase } if phase == attempt.phase)
            );
            let published = f
                .owner
                .store
                .lock()
                .unwrap()
                .result_artifacts(&f.task.scope())
                .unwrap();
            assert_eq!(published.len(), 1);
            let artifact = &published[0];
            assert_eq!(artifact.state, execution::ArtifactState::Published);
            assert_eq!(artifact.unit_id, prepared.id);
            assert_ne!(artifact.revision, prepared.base_sha);
            results::ResultStore::new(f.owner.clone())
                .verify(artifact)
                .await
                .unwrap();
            let done = f
                .owner
                .store
                .lock()
                .unwrap()
                .execution_unit(prepared.id)
                .unwrap();
            assert_eq!(done.work, Some(WorkOutcome::Success));
            assert!(!done.native_effects_open && !done.result_finalization_open);
            assert_eq!(
                f.owner
                    .store
                    .lock()
                    .unwrap()
                    .execution_units(Some(&f.task.scope()))
                    .unwrap()
                    .len(),
                1
            );
            if class == WorkflowClass::Quick {
                {
                    let mut store = f.owner.store.lock().unwrap();
                    if provider == "claude" {
                        let mut goal = store.goal(f.task.goal_id).unwrap().unwrap();
                        goal.objective
                            .push_str(" fixture changed accepted instruction");
                        store.put_goal(&mut goal).unwrap();
                    } else {
                        let mut project = store.project(f.task.project_id).unwrap().unwrap();
                        project.rule_refs.clear();
                        store.put_project(&mut project).unwrap();
                    }
                }
                let error = engine.step(f.task.id, BTreeMap::new()).await.unwrap_err();
                assert!(
                    format!("{error:#}").contains("instructions changed"),
                    "{error:#}"
                );
                let unchanged = f
                    .owner
                    .store
                    .lock()
                    .unwrap()
                    .result_artifact(artifact.id)
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(&unchanged).unwrap(),
                    serde_json::to_value(artifact).unwrap()
                );
                assert_eq!(
                    f.owner
                        .store
                        .lock()
                        .unwrap()
                        .managed_effects(prepared.id)
                        .unwrap()
                        .iter()
                        .filter(|e| e.kind == "native_input")
                        .count(),
                    1
                );
            }
        }
    }
}

#[tokio::test]
async fn initial_sources_retire_on_cancel_failure_and_last_owner_drop_before_native() {
    for outcome in ["cancel", "fail", "drop"] {
        let f = Fixture::new("codex", WorkflowClass::Standard).await;
        let prepared = f.sources.prepare(f.task.id, "codex").await.unwrap();
        let engine = f.engine(f.sources.clone());
        engine.initialize(f.task.id, None).await.unwrap();
        assert!(matches!(
            engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Completed {
                phase: Phase::Issue
            }
        ));
        match outcome {
            "cancel" => engine
                .cancel(f.task.id, "cancel before native".into())
                .unwrap(),
            "fail" => engine
                .fail_task(f.task.id, "fail before native".into())
                .unwrap(),
            _ => {}
        }
        let owner = f.owner.clone();
        let scope = f.task.scope();
        if outcome == "drop" {
            drop(engine);
            drop(f.sources);
        }
        let store = owner.store.lock().unwrap();
        let unit = store.execution_unit(prepared.id).unwrap();
        assert!(
            !unit.native_effects_open && !unit.result_finalization_open,
            "{outcome}"
        );
        assert_eq!(unit.work, Some(WorkOutcome::Unknown));
        assert!(unit.session_id.is_none() && unit.artifact_id.is_none());
        assert!(store.result_artifacts(&scope).unwrap().is_empty());
        assert!(
            store
                .managed_effects(unit.id)
                .unwrap()
                .iter()
                .all(|e| e.kind == "git_helper")
        );
        assert!(prepared.worktree.exists());
    }
}

enum Drift {
    Payload,
    TaskCas,
    StoredContext,
}
struct DriftingSources {
    inner: Arc<ManagedWorkflowSources>,
    owner: Arc<execution::RuntimeOwner>,
    database: std::path::PathBuf,
    drift: Drift,
}
impl WorkflowSources for DriftingSources {
    fn committed_input(
        &self,
        project: Project,
        task: Task,
        phase: Phase,
        class: WorkflowClass,
    ) -> WorkflowFuture<'_, Option<CommittedWorkflowInput>> {
        Box::pin(async move {
            let mut input = self
                .inner
                .committed_input(project, task, phase, class)
                .await?;
            if matches!(self.drift, Drift::Payload) && phase.actor() == Actor::Executor {
                input
                    .as_mut()
                    .unwrap()
                    .source
                    .payload
                    .push_str("\nfixture injected payload drift\n");
            }
            Ok(input)
        })
    }
    fn capture(
        &self,
        p: Project,
        t: Task,
        phase: Phase,
        b: ContextBudget,
    ) -> WorkflowFuture<'_, SourceSnapshot> {
        self.inner.capture(p, t, phase, b)
    }
    fn take_initial_executor(
        &self,
        p: &Project,
        t: &Task,
        phase: Phase,
        b: &ContextBudget,
    ) -> WorkflowFuture<'_, Option<execution::workflow_source::InitialWorkflowExecutor>> {
        let p = p.clone();
        let t = t.clone();
        let b = b.clone();
        Box::pin(async move {
            let prepared = self.inner.take_initial_executor(&p, &t, phase, &b).await?;
            if matches!(self.drift, Drift::TaskCas) {
                let mut store = self.owner.store.lock().unwrap();
                let mut task = store.task(t.id)?.unwrap();
                task.next_action = Some("fixture concurrent bookkeeping".into());
                store.put_task(&mut task)?;
            }
            if matches!(self.drift, Drift::StoredContext) {
                // Deliberate corruption of this test's private DB after outgoing input
                // construction; production append-only protection is not relaxed.
                let writer = crate::state::current_test_writer(&self.database)?;
                writer.execute_batch("DROP TRIGGER context_no_update")?;
                assert_eq!(writer.execute(
                    "UPDATE context_versions SET body=json_set(body,'$.data.payload','fixture corrupted stored payload') WHERE project_id=?1 AND version=?2 AND json_extract(body,'$.scope.task_id')=?3",
                    rusqlite::params![t.project_id.to_string(), t.context_version, t.id.to_string()],
                )?, 1);
            }
            Ok(prepared)
        })
    }
    fn retire_initial(&self, scope: &Scope) -> Result<()> {
        self.inner.retire_initial(scope)
    }
}

#[tokio::test]
async fn actual_first_adoption_refuses_payload_drift_and_stale_task_cas_before_native() {
    for drift in [Drift::Payload, Drift::TaskCas, Drift::StoredContext] {
        let f = Fixture::new("codex", WorkflowClass::Quick).await;
        let prepared = f.sources.prepare(f.task.id, "codex").await.unwrap();
        let expected = match drift {
            Drift::Payload => "first native input differs from prepared committed frame",
            Drift::TaskCas => "prepared Task CAS changed",
            Drift::StoredContext => "prepared source/rule frame differs from Context",
        };
        let sources = Arc::new(DriftingSources {
            inner: f.sources.clone(),
            owner: f.owner.clone(),
            database: f._dir.path().join("state.db"),
            drift,
        });
        let engine = f.engine(sources);
        engine.initialize(f.task.id, None).await.unwrap();
        early_phases(&engine, f.task.id, WorkflowClass::Quick).await;
        let error = engine.step(f.task.id, BTreeMap::new()).await.unwrap_err();
        assert!(format!("{error:#}").contains(expected), "{error:#}");
        let snapshot = engine.snapshot(f.task.id).unwrap();
        let attempt = &snapshot.history[snapshot.active.unwrap()];
        assert!(
            attempt.unit.is_none()
                && attempt.execution.is_none()
                && attempt.session_id.is_none()
                && !attempt.dispatch_started
        );
        let current = f
            .owner
            .store
            .lock()
            .unwrap()
            .execution_unit(prepared.id)
            .unwrap();
        assert_eq!(current.phase, execution::model::WORKFLOW_SOURCE_BOOTSTRAP);
        assert!(!current.native_effects_open && !current.result_finalization_open);
        f.assert_no_native(prepared.id);
        assert_eq!(
            f.owner
                .store
                .lock()
                .unwrap()
                .execution_units(Some(&f.task.scope()))
                .unwrap()
                .len(),
            1
        );
    }
}

#[derive(Clone, Copy, Debug)]
enum GateClaimFault {
    TaskVersion,
    ProjectVersion,
    FullContext,
    ForeignScope,
    Phase,
    Prerequisites,
    PriorObservations,
    SourcePayload,
}
struct GateClaimControl {
    gate: execution::workflow_gates::ManagedWorkflowGates,
    owner: Arc<execution::RuntimeOwner>,
    fault: GateClaimFault,
    checked: std::sync::Mutex<bool>,
}
impl PhaseGates for GateClaimControl {
    fn complete(
        &self,
        mut invocation: PhaseInvocation,
        transport: Option<SessionStatus>,
    ) -> WorkflowFuture<'_, GateOutcome> {
        Box::pin(async move {
            if invocation.phase != Phase::Worktree {
                return self.gate.complete(invocation, transport).await;
            }
            let scope = invocation.task.scope();
            let (unit, effects, receipts) = {
                let store = self.owner.store.lock().unwrap();
                let units = store.execution_units(Some(&scope)).unwrap();
                assert_eq!(units.len(), 1);
                let unit = units[0].clone();
                (
                    unit.clone(),
                    serde_json::to_value(store.managed_effects(unit.id).unwrap()).unwrap(),
                    serde_json::to_value(store.records(&scope, RecordKind::Verification).unwrap())
                        .unwrap(),
                )
            };
            match self.fault {
                GateClaimFault::TaskVersion => invocation.task.version += 1,
                GateClaimFault::ProjectVersion => invocation.project.version += 1,
                GateClaimFault::FullContext => {
                    invocation.context.data["generation"] = serde_json::json!(999_999);
                }
                GateClaimFault::ForeignScope => {
                    invocation.sources.scope.goal_id = Some(GoalId::new());
                }
                GateClaimFault::Phase => invocation.phase = Phase::Issue,
                GateClaimFault::Prerequisites => {
                    assert_eq!(invocation.prerequisites.len(), 1);
                    assert_eq!(invocation.prerequisites[0].phase, Phase::Issue);
                    invocation.prerequisites.clear();
                }
                GateClaimFault::PriorObservations => {
                    assert!(invocation.prior_observations.is_empty());
                    invocation.prior_observations.push(GateObservation {
                        sources: invocation.sources.clone(),
                        outcome: None,
                        error: Some("injected foreign observation claim".into()),
                        at: now_ms(),
                    });
                }
                GateClaimFault::SourcePayload => {
                    invocation
                        .sources
                        .payload
                        .push_str("\ninjected source body\n");
                }
            }
            let result = self.gate.complete(invocation, transport).await;
            assert!(result.is_err(), "{:?} was accepted", self.fault);
            let expected = match self.fault {
                GateClaimFault::TaskVersion
                | GateClaimFault::ProjectVersion
                | GateClaimFault::ForeignScope => "foreign, stale or inactive gate owners",
                GateClaimFault::FullContext
                | GateClaimFault::Phase
                | GateClaimFault::Prerequisites
                | GateClaimFault::PriorObservations => {
                    "exact Evaluating Context/claim/prerequisites"
                }
                GateClaimFault::SourcePayload => {
                    "initial gate differs from prepared committed input"
                }
            };
            let error = result.unwrap_err();
            assert!(
                format!("{error:#}").contains(expected),
                "{:?}: {error:#}",
                self.fault
            );
            {
                let store = self.owner.store.lock().unwrap();
                assert_eq!(
                    serde_json::to_value(store.managed_effects(unit.id).unwrap()).unwrap(),
                    effects,
                    "{:?} added or changed a managed helper effect",
                    self.fault
                );
                assert_eq!(
                    serde_json::to_value(store.records(&scope, RecordKind::Verification).unwrap())
                        .unwrap(),
                    receipts,
                    "{:?} recorded a successful receipt",
                    self.fault
                );
                assert_eq!(
                    serde_json::to_value(store.execution_unit(unit.id).unwrap()).unwrap(),
                    serde_json::to_value(&unit).unwrap(),
                    "{:?} changed the prepared unit",
                    self.fault
                );
                assert!(
                    store
                        .records(&scope, RecordKind::Session)
                        .unwrap()
                        .is_empty()
                );
                assert!(store.result_artifacts(&scope).unwrap().is_empty());
            }
            *self.checked.lock().unwrap() = true;
            Err(error)
        })
    }
}

#[tokio::test]
async fn production_gate_claim_inputs_refuse_before_helpers_receipts_or_native_effects() {
    for fault in [
        GateClaimFault::TaskVersion,
        GateClaimFault::ProjectVersion,
        GateClaimFault::FullContext,
        GateClaimFault::ForeignScope,
        GateClaimFault::Phase,
        GateClaimFault::Prerequisites,
        GateClaimFault::PriorObservations,
        GateClaimFault::SourcePayload,
    ] {
        let f = Fixture::new("codex", WorkflowClass::Standard).await;
        // This existing helper installs actual acceptance criteria; no fixture Passed gate is used.
        drop(f.production_engine(""));
        let prepared = f.sources.prepare(f.task.id, "codex").await.unwrap();
        let control = Arc::new(GateClaimControl {
            gate: execution::workflow_gates::ManagedWorkflowGates::new(
                f.owner.clone(),
                f.sources.clone(),
            )
            .unwrap(),
            owner: f.owner.clone(),
            fault,
            checked: std::sync::Mutex::new(false),
        });
        let engine = WorkflowEngine::new(
            f.owner.store(),
            f.registry.clone(),
            f.config.clone(),
            f.sources.clone(),
            control.clone(),
        )
        .unwrap();
        engine.initialize(f.task.id, None).await.unwrap();
        assert!(matches!(
            engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Completed {
                phase: Phase::Issue
            }
        ));
        assert!(matches!(
            engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Waiting {
                phase: Phase::Worktree,
                ..
            }
        ));
        assert!(
            *control.checked.lock().unwrap(),
            "{fault:?} did not reach actual gate"
        );
        let snapshot = engine.snapshot(f.task.id).unwrap();
        assert!(!snapshot.completed.contains_key(&Phase::Worktree));
        let attempt = &snapshot.history[snapshot.active.unwrap()];
        assert_eq!(attempt.phase, Phase::Worktree);
        assert_eq!(attempt.state, AttemptState::Evaluating);
        assert_eq!(attempt.observations.len(), 1);
        assert!(attempt.observations[0].outcome.is_none());
        f.assert_no_native(prepared.id);
        engine
            .cancel(f.task.id, "claim negative control finished".into())
            .unwrap();
    }
}

struct RememberGateClaim {
    gate: execution::workflow_gates::ManagedWorkflowGates,
    invocation: std::sync::Mutex<Option<PhaseInvocation>>,
}
impl PhaseGates for RememberGateClaim {
    fn complete(
        &self,
        invocation: PhaseInvocation,
        transport: Option<SessionStatus>,
    ) -> WorkflowFuture<'_, GateOutcome> {
        Box::pin(async move {
            if invocation.phase == Phase::Worktree {
                *self.invocation.lock().unwrap() = Some(invocation.clone());
            }
            self.gate.complete(invocation, transport).await
        })
    }
}
#[tokio::test]
async fn production_gate_closed_claim_cannot_be_replayed_into_another_receipt() {
    let f = Fixture::new("codex", WorkflowClass::Quick).await;
    drop(f.production_engine(""));
    let prepared = f.sources.prepare(f.task.id, "codex").await.unwrap();
    let control = Arc::new(RememberGateClaim {
        gate: execution::workflow_gates::ManagedWorkflowGates::new(
            f.owner.clone(),
            f.sources.clone(),
        )
        .unwrap(),
        invocation: std::sync::Mutex::new(None),
    });
    let engine = WorkflowEngine::new(
        f.owner.store(),
        f.registry.clone(),
        f.config.clone(),
        f.sources.clone(),
        control.clone(),
    )
    .unwrap();
    engine.initialize(f.task.id, None).await.unwrap();
    assert!(matches!(
        engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Completed {
            phase: Phase::Worktree
        }
    ));
    let invocation = control.invocation.lock().unwrap().take().unwrap();
    let workflow = serde_json::to_value(engine.snapshot(f.task.id).unwrap()).unwrap();
    let (effects, receipts) = {
        let store = f.owner.store.lock().unwrap();
        (
            serde_json::to_value(store.managed_effects(prepared.id).unwrap()).unwrap(),
            serde_json::to_value(
                store
                    .records(&f.task.scope(), RecordKind::Verification)
                    .unwrap(),
            )
            .unwrap(),
        )
    };
    let error = control.gate.complete(invocation, None).await.unwrap_err();
    assert!(format!("{error:#}").contains("foreign, stale or inactive gate owners"));
    {
        let store = f.owner.store.lock().unwrap();
        assert_eq!(
            serde_json::to_value(store.managed_effects(prepared.id).unwrap()).unwrap(),
            effects
        );
        assert_eq!(
            serde_json::to_value(
                store
                    .records(&f.task.scope(), RecordKind::Verification)
                    .unwrap()
            )
            .unwrap(),
            receipts
        );
    }
    assert_eq!(
        serde_json::to_value(engine.snapshot(f.task.id).unwrap()).unwrap(),
        workflow
    );
    f.assert_no_native(prepared.id);
    engine
        .cancel(f.task.id, "closed claim control finished".into())
        .unwrap();
}

#[path = "verification_tests.rs"]
mod verification_tests;
