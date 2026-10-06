//! Account-free controls through Registry and the existing Workflow.
//! Evidence ports here are test integrations, not a shipped Runtime gate policy.
use super::*;
use crate::execution::{self, ArtifactState, WorkOutcome, results};
use std::{path::Path, time::Duration};

struct Sources {
    owner: Arc<execution::RuntimeOwner>,
    seed: String,
    result: results::ResultStore,
}
impl WorkflowSources for Sources {
    fn capture(
        &self,
        _: Project,
        task: Task,
        _: Phase,
        _: ContextBudget,
    ) -> WorkflowFuture<'_, SourceSnapshot> {
        Box::pin(async move {
            let (unit, previous, context, artifacts) = {
                let store = self.owner.store.lock().unwrap();
                let records = store.records(&task.scope(), RecordKind::Workflow)?;
                let workflow = records
                    .first()
                    .map(|r| serde_json::from_value::<WorkflowSnapshot>(r.data.clone()))
                    .transpose()?;
                let attempt = workflow
                    .as_ref()
                    .and_then(|w| w.active.and_then(|i| w.history.get(i)));
                let unit = attempt
                    .and_then(|a| a.unit.as_ref())
                    .map(|u| store.execution_unit(u.unit))
                    .transpose()?;
                let context = attempt
                    .map(|a| store.context(&task.scope(), Some(a.context_version)))
                    .transpose()?
                    .flatten();
                (
                    unit,
                    workflow.and_then(|w| w.sources.artifact),
                    context,
                    store.result_artifacts(&task.scope())?,
                )
            };
            let artifact = if let Some(unit) = unit.filter(|u| {
                u.kind == execution::UnitKind::Executor
                    && u.work == Some(WorkOutcome::Success)
                    && u.result_finalization_open
            }) {
                if let Some(ready) = artifacts
                    .into_iter()
                    .find(|a| a.unit_id == unit.id && a.state == ArtifactState::Ready)
                {
                    Some(ready)
                } else {
                    let io = execution::git_io::UnitGit::new(self.owner.clone(), &unit, false)?;
                    let sha = io.text(&unit.worktree, ["rev-parse", "HEAD"]).await?;
                    let mut versions = context
                        .context("fixture launch context missing")?
                        .source_hashes;
                    versions.retain(|k, _| !k.starts_with("workflow:"));
                    versions.insert("code".into(), sha.clone());
                    Some(
                        self.result
                            .capture(&unit.authority(), &sha, versions)
                            .await?,
                    )
                }
            } else if let Some(id) = previous {
                Some(self.owner.store.lock().unwrap().result_artifact(id)?)
            } else {
                None
            };
            Ok(SourceSnapshot {
                scope: task.scope(),
                revision: artifact
                    .as_ref()
                    .map_or_else(|| self.seed.clone(), |a| a.revision.clone()),
                artifact: artifact.as_ref().map(|a| a.id),
                source_versions: artifact.map_or_else(
                    || BTreeMap::from([("code".into(), self.seed.clone())]),
                    |a| a.dependencies,
                ),
                payload: "complete".into(),
            })
        })
    }
}
struct Gates {
    owner: Arc<execution::RuntimeOwner>,
    control: &'static str,
}
impl PhaseGates for Gates {
    fn complete(
        &self,
        invocation: PhaseInvocation,
        status: Option<SessionStatus>,
    ) -> WorkflowFuture<'_, GateOutcome> {
        Box::pin(async move {
            let marker = if invocation.phase.actor() == Actor::Executor {
                let native = status
                    .as_ref()
                    .context("fixture native terminal missing")?
                    .execution
                    .as_ref()
                    .context("fixture managed terminal missing")?;
                ensure!(
                    native.work == Some(WorkOutcome::Success),
                    "fixture requires actual owned success"
                );
                let id = invocation
                    .sources
                    .artifact
                    .context("fixture retained artifact missing")?;
                let artifact = self.owner.store.lock().unwrap().result_artifact(id)?;
                ensure!(
                    artifact.unit_id == native.handle.unit
                        && artifact.revision == invocation.sources.revision,
                    "fixture artifact differs from native work"
                );
                match self.control {
                    "manifest-missing" => std::fs::remove_file(&artifact.manifest)?,
                    "manifest-corrupt" => std::fs::write(&artifact.manifest, "corrupt")?,
                    "ref-missing" => {
                        let unit = self
                            .owner
                            .store
                            .lock()
                            .unwrap()
                            .execution_unit(native.handle.unit)?;
                        execution::git_io::UnitGit::new(self.owner.clone(), &unit, false)?
                            .run(
                                &artifact.repository,
                                ["update-ref", "-d", &format!("refs/rrx/{id}/commit")],
                            )
                            .await?;
                    }
                    "cancel" => {
                        self.owner
                            .store
                            .lock()
                            .unwrap()
                            .retire_execution(&native.authority, false)?;
                    }
                    _ => {}
                }
                format!("rrx-artifact:{id}")
            } else {
                // Fixture evidence only; native startup follows real registered
                // preparation, rather than treating this string as custody.
                "fixture-evidence-port".into()
            };
            Ok(GateOutcome::Passed(Evidence {
                scope: invocation.task.scope(),
                phase: invocation.phase,
                revision: invocation.sources.revision,
                source_versions: invocation.sources.source_versions.clone(),
                artifacts: vec![marker],
                dependencies: invocation.sources.source_versions,
                review_approved: (invocation.phase.actor() == Actor::Reviewer).then_some(true),
                session_id: status.as_ref().map(|s| s.session.id),
                context_version: invocation.context.version,
            }))
        })
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
fn configuration(provider: &str, program: &Path) -> Config {
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
    config
}
async fn native_terminal(
    registry: &AgentRegistry,
    identity: &execution::native::ManagedSessionRef,
) {
    let adapter = registry.get("native-alias").unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let status = adapter
                .status(SessionRef {
                    id: identity.session,
                    scope: identity.scope.clone(),
                    execution: Some(identity.clone()),
                })
                .await
                .unwrap();
            if status.terminal() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn workflow_live_subscription_wait_retains_the_same_attempt_until_cancelled() {
    for (provider, scenario, recovers) in [
        ("claude", "quota-retry-held", false),
        ("codex", "quota-retry-held", false),
        ("claude", "quota-live-recovery-held", true),
    ] {
        let (dir, owner, fixture_task) = results::tests::fixture().await;
        let mut task = Task::new(
            fixture_task.project_id,
            fixture_task.goal_id,
            "live quota workflow".into(),
            "native-alias".into(),
        );
        task.workflow = WorkflowClass::Quick;
        owner.store.lock().unwrap().put_task(&mut task).unwrap();
        let seed = results::text(
            &results::git(&dir.path().join("repo"), ["rev-parse", "HEAD"])
                .await
                .unwrap(),
        )
        .unwrap();
        let program = execution::native::tests::program(dir.path(), provider);
        let text = std::fs::read_to_string(&program).unwrap();
        std::fs::write(
            &program,
            text.replacen(
                "import json",
                &format!("WORKFLOW_SCENARIO = {scenario:?}\nimport json"),
                1,
            ),
        )
        .unwrap();
        let config = configuration(provider, &program);
        let registry =
            Arc::new(AgentRegistry::from_managed_config(&config, owner.clone()).unwrap());
        let engine = WorkflowEngine::new(
            owner.store(),
            registry.clone(),
            config,
            Arc::new(Sources {
                owner: owner.clone(),
                seed,
                result: results::ResultStore::new(owner.clone()),
            }),
            Arc::new(Gates {
                owner: owner.clone(),
                control: "publish",
            }),
        )
        .unwrap();
        engine.initialize(task.id, None).await.unwrap();
        engine.step(task.id, BTreeMap::new()).await.unwrap();
        assert!(matches!(
            engine.step(task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Started { .. }
        ));
        let workflow = engine.snapshot(task.id).unwrap();
        let index = workflow.active.unwrap();
        let identity = workflow.history[index].execution.clone().unwrap();
        let adapter = registry.get("native-alias").unwrap();
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let status = adapter
                    .status(SessionRef {
                        id: identity.session,
                        scope: identity.scope.clone(),
                        execution: Some(identity.clone()),
                    })
                    .await
                    .unwrap();
                if status.execution.as_ref().unwrap().wait_reason
                    == Some(execution::WaitReason::Quota)
                {
                    assert!(!status.terminal());
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        for _ in 0..2 {
            assert!(matches!(
                engine.step(task.id, BTreeMap::new()).await.unwrap(),
                StepResult::Waiting { .. }
            ));
            let current = engine.snapshot(task.id).unwrap();
            assert_eq!(current.active, Some(index));
            assert_eq!(current.history.len(), workflow.history.len());
            assert_eq!(current.history[index].execution.as_ref(), Some(&identity));
            let store = owner.store.lock().unwrap();
            assert_eq!(
                store.task(task.id).unwrap().unwrap().state,
                TaskState::WaitingQuota
            );
            assert_eq!(store.execution_unit(identity.unit).unwrap().work, None);
            assert_eq!(
                store
                    .managed_effects(identity.unit)
                    .unwrap()
                    .iter()
                    .filter(|e| e.kind == "native_input")
                    .count(),
                1
            );
        }
        if recovers {
            let unit = owner
                .store
                .lock()
                .unwrap()
                .execution_unit(identity.unit)
                .unwrap();
            let output = execution::resources::ResourceManager::new(owner.clone())
                .profile(&unit)
                .unwrap()
                .output;
            std::fs::write(output.join("fixture-recovery-release"), "recover").unwrap();
            tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    let status = adapter
                        .status(SessionRef {
                            id: identity.session,
                            scope: identity.scope.clone(),
                            execution: Some(identity.clone()),
                        })
                        .await
                        .unwrap();
                    if status.execution.as_ref().unwrap().wait_reason.is_none() {
                        assert!(!status.terminal());
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            })
            .await
            .unwrap();
            assert!(matches!(
                engine.step(task.id, BTreeMap::new()).await.unwrap(),
                StepResult::Running { .. }
            ));
            let current = engine.snapshot(task.id).unwrap();
            assert_eq!(current.active, Some(index));
            assert_eq!(current.history.len(), workflow.history.len());
            assert_eq!(current.history[index].execution.as_ref(), Some(&identity));
            assert_eq!(
                owner
                    .store
                    .lock()
                    .unwrap()
                    .task(task.id)
                    .unwrap()
                    .unwrap()
                    .state,
                TaskState::Implementing
            );
        }
        engine
            .cancel(task.id, "cancel live quota waiter".into())
            .unwrap();
        native_terminal(&registry, &identity).await;
        let store = owner.store.lock().unwrap();
        let unit = store.execution_unit(identity.unit).unwrap();
        assert_eq!(unit.disposition, execution::Disposition::Cancelled);
        assert!(!unit.native_effects_open && !unit.result_finalization_open);
        assert_eq!(
            store.task(task.id).unwrap().unwrap().state,
            TaskState::Cancelled
        );
        assert_eq!(
            store
                .managed_effects(identity.unit)
                .unwrap()
                .iter()
                .filter(|e| e.kind == "native_input")
                .count(),
            1
        );
    }
}

#[tokio::test]
async fn workflow_native_capacity_terminal_waits_with_fresh_resources_instead_of_failing() {
    for (provider, scenario, waiting) in [
        ("codex", "quota-terminal", TaskState::WaitingQuota),
        ("codex", "capacity-rate", TaskState::WaitingCapacity),
        ("claude", "quota-stale-available", TaskState::WaitingQuota),
        ("claude", "capacity-rate", TaskState::WaitingCapacity),
    ] {
        let (dir, owner, fixture_task) = results::tests::fixture().await;
        let mut task = Task::new(
            fixture_task.project_id,
            fixture_task.goal_id,
            "quota workflow fixture".into(),
            "native-alias".into(),
        );
        task.workflow = WorkflowClass::Quick;
        owner.store.lock().unwrap().put_task(&mut task).unwrap();
        let seed = results::text(
            &results::git(&dir.path().join("repo"), ["rev-parse", "HEAD"])
                .await
                .unwrap(),
        )
        .unwrap();
        let program = execution::native::tests::program(dir.path(), provider);
        let text = std::fs::read_to_string(&program).unwrap();
        std::fs::write(
            &program,
            text.replacen(
                "import json",
                &format!("WORKFLOW_SCENARIO = {scenario:?}\nimport json"),
                1,
            ),
        )
        .unwrap();
        let config = configuration(provider, &program);
        let registry =
            Arc::new(AgentRegistry::from_managed_config(&config, owner.clone()).unwrap());
        let engine = WorkflowEngine::new(
            owner.store(),
            registry.clone(),
            config,
            Arc::new(Sources {
                owner: owner.clone(),
                seed,
                result: results::ResultStore::new(owner.clone()),
            }),
            Arc::new(Gates {
                owner: owner.clone(),
                control: "publish",
            }),
        )
        .unwrap();
        engine.initialize(task.id, None).await.unwrap();
        engine.step(task.id, BTreeMap::new()).await.unwrap();
        assert!(matches!(
            engine.step(task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Started { .. }
        ));
        let workflow = engine.snapshot(task.id).unwrap();
        let first = workflow.history[workflow.active.unwrap()]
            .execution
            .clone()
            .unwrap();
        native_terminal(&registry, &first).await;
        assert!(matches!(
            engine.step(task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Waiting { .. }
        ));
        {
            let store = owner.store.lock().unwrap();
            let unit = store.execution_unit(first.unit).unwrap();
            assert_eq!(unit.work, Some(WorkOutcome::Unknown));
            assert!(!unit.native_effects_open && !unit.result_finalization_open);
            assert_eq!(store.task(task.id).unwrap().unwrap().state, waiting);
        }
        assert!(matches!(
            engine.step(task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Waiting { .. }
        ));
        let workflow = engine.snapshot(task.id).unwrap();
        let fresh = &workflow.history[workflow.active.unwrap()];
        assert!(fresh.session_id.is_none());
        assert_ne!(fresh.unit.as_ref().unwrap().unit, first.unit);
        assert!(fresh.native_wait.is_some());
        assert_eq!(
            owner
                .store
                .lock()
                .unwrap()
                .task(task.id)
                .unwrap()
                .unwrap()
                .state,
            waiting
        );
        assert!(!workflow.completed.contains_key(&Phase::Implement));
        assert_eq!(
            owner
                .store
                .lock()
                .unwrap()
                .managed_effects(fresh.unit.as_ref().unwrap().unit)
                .unwrap()
                .iter()
                .filter(|e| e.kind == "native_input")
                .count(),
            0
        );
    }
}

#[tokio::test]
async fn managed_workflow_reviews_retained_snapshot_and_rejects_changed_review_input() {
    use std::os::unix::fs::PermissionsExt;
    for control in ["publish", "input-changed", "artifact-changed", "cancel"] {
        let (dir, owner, fixture_task) = results::tests::fixture().await;
        let mut task = Task::new(
            fixture_task.project_id,
            fixture_task.goal_id,
            "readonly workflow fixture".into(),
            "native-alias".into(),
        );
        task.workflow = WorkflowClass::Quick;
        task.reviewers = vec!["native-reviewer".into()];
        owner.store.lock().unwrap().put_task(&mut task).unwrap();
        let executor = execution::native::tests::program(dir.path(), "codex");
        let reviewer = execution::native::tests::program(dir.path(), "claude");
        let text = std::fs::read_to_string(&reviewer).unwrap();
        std::fs::write(
            &reviewer,
            text.replacen(
                "import json",
                "WORKFLOW_SCENARIO = 'readonly-review'\nimport json",
                1,
            ),
        )
        .unwrap();
        let mut config = configuration("codex", &executor);
        config.agents.insert(
            "native-reviewer".into(),
            crate::config::AgentConfig {
                provider: Some("claude".into()),
                command: vec![reviewer.to_string_lossy().into()],
                ..Default::default()
            },
        );
        let registry =
            Arc::new(AgentRegistry::from_managed_config(&config, owner.clone()).unwrap());
        let sources = Arc::new(
            execution::workflow_source::ManagedWorkflowSources::new(owner.clone(), config.clone())
                .unwrap(),
        );
        let verifier = Arc::new(
            execution::verification::ManagedVerifier::new(owner.clone(), sources.clone()).unwrap(),
        );
        use execution::verification::{Applicability, Category, TestCommand, TestsProfile};
        let project_version = owner
            .store
            .lock()
            .unwrap()
            .project(task.project_id)
            .unwrap()
            .unwrap()
            .version;
        // This managed fixture reaches review only through a real Tests command,
        // admitted before Workflow activation; caller Passed is never Tests proof.
        verifier
            .admit_tests(
                task.project_id,
                project_version,
                TestsProfile {
                    commands: vec![TestCommand {
                        id: "retained-answer".into(),
                        category: Category::Tests,
                        program: std::fs::canonicalize("/usr/bin/python3").unwrap(),
                        args: vec![
                            "-c".into(),
                            "from pathlib import Path; assert Path('fixture-result.txt').is_file()"
                                .into(),
                        ],
                        cwd: ".".into(),
                        timeout_seconds: 10,
                        drain_seconds: 1,
                        stdout_bytes: 4096,
                        stderr_bytes: 4096,
                    }],
                    applicability: BTreeMap::from([
                        (Category::Tests, Applicability::Required),
                        (
                            Category::Typecheck,
                            Applicability::NotApplicable {
                                reason: "fixture has no typed sources".into(),
                            },
                        ),
                        (
                            Category::Lint,
                            Applicability::NotApplicable {
                                reason: "fixture has no lint toolchain".into(),
                            },
                        ),
                        (
                            Category::Build,
                            Applicability::NotApplicable {
                                reason: "fixture contains data only".into(),
                            },
                        ),
                    ]),
                },
            )
            .unwrap();
        sources.prepare(task.id, "codex").await.unwrap();
        let engine = WorkflowEngine::new(
            owner.store(),
            registry.clone(),
            config,
            sources,
            Arc::new(Gates {
                owner: owner.clone(),
                control: "publish",
            }),
        )
        .unwrap()
        .with_verifier(verifier)
        .unwrap();
        engine.initialize(task.id, None).await.unwrap();
        engine.step(task.id, BTreeMap::new()).await.unwrap();
        engine.step(task.id, BTreeMap::new()).await.unwrap();
        let workflow = engine.snapshot(task.id).unwrap();
        let executor = workflow.history[workflow.active.unwrap()]
            .execution
            .clone()
            .unwrap();
        let executor_unit = owner
            .store
            .lock()
            .unwrap()
            .execution_unit(executor.unit)
            .unwrap();
        let profile = execution::resources::ResourceManager::new(owner.clone())
            .profile(&executor_unit)
            .unwrap();
        wait_file(&profile.output.join("fixture-ready")).await;
        std::fs::write(profile.output.join("fixture-release"), "release").unwrap();
        native_terminal(&registry, &executor).await;
        assert!(matches!(
            engine.step(task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Completed {
                phase: Phase::Implement
            }
        ));
        let accepted = owner
            .store
            .lock()
            .unwrap()
            .task(task.id)
            .unwrap()
            .unwrap()
            .revision
            .unwrap();
        std::fs::write(
            executor_unit.worktree.join("fixture-result.txt"),
            "survivor changes live worktree\n",
        )
        .unwrap();
        for phase in [Phase::Commit, Phase::Tests] {
            assert!(
                matches!(engine.step(task.id, BTreeMap::new()).await.unwrap(), StepResult::Completed { phase: actual } if actual == phase)
            );
        }
        assert!(matches!(
            engine.step(task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Started {
                phase: Phase::ImplementationReview,
                ..
            }
        ));
        let workflow = engine.snapshot(task.id).unwrap();
        let reviewer = workflow.history[workflow.active.unwrap()]
            .execution
            .clone()
            .unwrap();
        let adapter = registry.get("native-reviewer").unwrap();
        tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                let status = adapter
                    .status(SessionRef {
                        id: reviewer.session,
                        scope: task.scope(),
                        execution: Some(reviewer.clone()),
                    })
                    .await
                    .unwrap();
                if status.terminal() {
                    assert_eq!(status.execution.unwrap().work, Some(WorkOutcome::Success));
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        let unit = owner
            .store
            .lock()
            .unwrap()
            .execution_unit(reviewer.unit)
            .unwrap();
        assert_ne!(unit.worktree, executor_unit.worktree);
        assert_eq!(unit.base_sha, accepted);
        assert_ne!(
            std::fs::read(unit.worktree.join("fixture-result.txt")).unwrap(),
            std::fs::read(executor_unit.worktree.join("fixture-result.txt")).unwrap()
        );
        if control == "input-changed" {
            let path = unit.worktree.join("answer.txt");
            let permissions = std::fs::metadata(&path).unwrap().permissions();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
            std::fs::write(&path, "changed review input\n").unwrap();
            std::fs::set_permissions(&path, permissions).unwrap();
        }
        if matches!(control, "artifact-changed" | "cancel") {
            let owner = owner.clone();
            let authority = unit.authority();
            let artifact = unit.artifact_id.unwrap();
            *engine.hooks.before_publication.lock().unwrap() = Some(Box::new(move || {
                let mut store = owner.store.lock().unwrap();
                if control == "cancel" {
                    store.retire_execution(&authority, false).unwrap();
                } else {
                    store
                        .invalidate_execution_artifact(
                            artifact,
                            "fixture after-verification change",
                        )
                        .unwrap();
                }
            }));
        }
        let outcome = engine.step(task.id, BTreeMap::new()).await;
        if control != "publish" {
            assert!(outcome.is_err());
            assert!(
                !engine
                    .snapshot(task.id)
                    .unwrap()
                    .completed
                    .contains_key(&Phase::ImplementationReview)
            );
        } else {
            assert!(matches!(
                outcome.unwrap(),
                StepResult::Completed {
                    phase: Phase::ImplementationReview
                }
            ));
        }
        let finalized = owner.store.lock().unwrap().execution_unit(unit.id).unwrap();
        assert!(!finalized.native_effects_open);
        assert_eq!(finalized.work, Some(WorkOutcome::Success));
        assert_eq!(
            finalized.result_finalization_open,
            matches!(control, "input-changed" | "artifact-changed")
        );
        assert_eq!(
            owner
                .store
                .lock()
                .unwrap()
                .task(task.id)
                .unwrap()
                .unwrap()
                .revision
                .as_deref(),
            Some(accepted.as_str())
        );
    }
}

#[tokio::test]
async fn managed_registry_workflow_qualifies_real_retained_content_before_atomic_publication() {
    for (provider, control) in [
        ("codex", "publish"),
        ("claude", "publish"),
        ("codex", "manifest-missing"),
        ("claude", "manifest-corrupt"),
        ("codex", "ref-missing"),
        ("claude", "cancel"),
        ("codex", "artifact-version"),
    ] {
        let (dir, owner, fixture_task) = results::tests::fixture().await;
        let mut task = Task::new(
            fixture_task.project_id,
            fixture_task.goal_id,
            "workflow fixture".into(),
            "native-alias".into(),
        );
        let seed = results::text(
            &results::git(&dir.path().join("repo"), ["rev-parse", "HEAD"])
                .await
                .unwrap(),
        )
        .unwrap();
        task.workflow = WorkflowClass::Quick;
        owner.store.lock().unwrap().put_task(&mut task).unwrap();
        let program = execution::native::tests::program(dir.path(), provider);
        let mut config = Config {
            minimum_workflow: WorkflowClass::Quick,
            ..Default::default()
        };
        config.workflow.risk_mapping = [WorkflowClass::Quick; 4];
        config.agents.insert(
            task.executor.clone(),
            crate::config::AgentConfig {
                provider: Some(provider.into()),
                command: vec![program.to_string_lossy().into()],
                ..Default::default()
            },
        );
        let registry =
            Arc::new(AgentRegistry::from_managed_config(&config, owner.clone()).unwrap());
        let sources = Arc::new(Sources {
            owner: owner.clone(),
            seed,
            result: results::ResultStore::new(owner.clone()),
        });
        let engine = WorkflowEngine::new(
            owner.store(),
            registry.clone(),
            config,
            sources,
            Arc::new(Gates {
                owner: owner.clone(),
                control,
            }),
        )
        .unwrap();
        engine.initialize(task.id, None).await.unwrap();
        if control == "artifact-version" {
            let state = owner.store();
            let scope = task.scope();
            *engine.hooks.before_publication.lock().unwrap() = Some(Box::new(move || {
                let mut store = state.lock().unwrap();
                let artifact = store.result_artifacts(&scope).unwrap().remove(0);
                store
                    .invalidate_execution_artifact(
                        artifact.id,
                        "fixture metadata change after verification",
                    )
                    .unwrap();
            }));
        }
        let first = engine.step(task.id, BTreeMap::new()).await.unwrap();
        assert!(
            matches!(
                first,
                StepResult::Completed {
                    phase: Phase::Worktree
                }
            ),
            "{first:?}"
        );
        let StepResult::Started {
            session: Some(session),
            ..
        } = engine.step(task.id, BTreeMap::new()).await.unwrap()
        else {
            panic!("managed native launch missing");
        };
        let workflow = engine.snapshot(task.id).unwrap();
        let attempt = &workflow.history[workflow.active.unwrap()];
        let identity = attempt.execution.clone().unwrap();
        assert_eq!(attempt.unit.as_ref().unwrap().unit, identity.unit);
        let unit = owner
            .store
            .lock()
            .unwrap()
            .execution_unit(identity.unit)
            .unwrap();
        let profile = execution::resources::ResourceManager::new(owner.clone())
            .profile(&unit)
            .unwrap();
        wait_file(&profile.output.join("fixture-ready")).await;
        std::fs::write(profile.output.join("fixture-release"), "release").unwrap();
        let adapter = registry.get(&task.executor).unwrap();
        tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                let status = adapter
                    .status(SessionRef {
                        id: session,
                        scope: task.scope(),
                        execution: Some(identity.clone()),
                    })
                    .await
                    .unwrap();
                if status.terminal() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        let before = owner.store.lock().unwrap().task(task.id).unwrap().unwrap();
        let outcome = engine.step(task.id, BTreeMap::new()).await;
        let store = owner.store.lock().unwrap();
        let artifacts = store.result_artifacts(&task.scope()).unwrap();
        assert_eq!(artifacts.len(), 1);
        let artifact = &artifacts[0];
        let after = store.task(task.id).unwrap().unwrap();
        let context = store.context(&task.scope(), None).unwrap().unwrap();
        let workflow: WorkflowSnapshot = serde_json::from_value(
            store.records(&task.scope(), RecordKind::Workflow).unwrap()[0]
                .data
                .clone(),
        )
        .unwrap();
        if control == "publish" {
            assert!(matches!(
                outcome.unwrap(),
                StepResult::Completed {
                    phase: Phase::Implement
                }
            ));
            assert_eq!(artifact.state, ArtifactState::Published);
            assert_eq!(after.revision.as_deref(), Some(artifact.revision.as_str()));
            assert_eq!(context.revision, artifact.revision);
            assert_eq!(workflow.sources.artifact, Some(artifact.id));
            assert_eq!(workflow.context_version, after.context_version);
            assert!(
                !store
                    .execution_unit(identity.unit)
                    .unwrap()
                    .result_finalization_open
            );
        } else {
            assert!(outcome.is_err(), "{provider}/{control}: {outcome:?}");
            assert_eq!(
                artifact.state,
                if control == "artifact-version" {
                    ArtifactState::Invalid
                } else {
                    ArtifactState::Ready
                }
            );
            assert_eq!(after.revision, before.revision);
            assert_eq!(after.context_version, before.context_version);
            assert_eq!(context.version, before.context_version);
            assert!(!workflow.completed.contains_key(&Phase::Implement));
        }
    }
}

#[tokio::test]
async fn concurrent_due_wait_claim_preserves_the_native_launch_or_wait_owner() {
    use execution::{QuotaObservation, QuotaStatus, UnitState, WaitReason};
    for exhausted_again in [false, true] {
        let (dir, owner, fixture_task) = results::tests::fixture().await;
        let mut task = Task::new(
            fixture_task.project_id,
            fixture_task.goal_id,
            "due wait claim".into(),
            "native-alias".into(),
        );
        task.workflow = WorkflowClass::Quick;
        owner.store.lock().unwrap().put_task(&mut task).unwrap();
        let seed = results::text(
            &results::git(&dir.path().join("repo"), ["rev-parse", "HEAD"])
                .await
                .unwrap(),
        )
        .unwrap();
        let path = execution::native::tests::program(dir.path(), "codex");
        let config = configuration("codex", &path);
        let registry =
            Arc::new(AgentRegistry::from_managed_config(&config, owner.clone()).unwrap());
        let sources = Arc::new(Sources {
            owner: owner.clone(),
            seed,
            result: results::ResultStore::new(owner.clone()),
        });
        let gates = Arc::new(Gates {
            owner: owner.clone(),
            control: "publish",
        });
        let engine = Arc::new(
            WorkflowEngine::new(
                owner.store(),
                registry.clone(),
                config.clone(),
                sources.clone(),
                gates.clone(),
            )
            .unwrap(),
        );
        let observer = Arc::new(
            WorkflowEngine::new(owner.store(), registry.clone(), config, sources, gates).unwrap(),
        );
        let observation = |status, at| QuotaObservation {
            provider: "codex".into(),
            account_key: "unknown".into(),
            bucket: "claim-fixture".into(),
            confirmed_subscription: true,
            window_id: if status == QuotaStatus::Available {
                "recovered"
            } else {
                "fixture"
            }
            .into(),
            status,
            used_percent: None,
            resets_at: Some(at + 3_600_000),
            observed_at: at,
            source_version: "fixture".into(),
        };
        owner
            .store
            .lock()
            .unwrap()
            .observe_quota(&observation(QuotaStatus::Exhausted, now_ms()))
            .unwrap();
        engine.initialize(task.id, None).await.unwrap();
        engine.step(task.id, BTreeMap::new()).await.unwrap();
        assert!(matches!(
            engine.step(task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Waiting { .. }
        ));
        let mut waiting = engine.read(task.id).unwrap();
        let index = waiting.workflow.active.unwrap();
        let unit = waiting.workflow.history[index].unit.as_ref().unwrap().unit;
        assert_eq!(
            waiting.workflow.history[index].native_wait,
            Some(WaitReason::Quota)
        );
        waiting.workflow.history[index].next_due = Some(0);
        engine.persist(&mut waiting, None).unwrap();
        // Advance only this fixture's observed quota window, not wall time.
        // The first real wait stays closed even under a slow full-workspace run.
        let recovery_at = now_ms() + 3_600_001;
        owner
            .store
            .lock()
            .unwrap()
            .observe_quota(&observation(QuotaStatus::Available, recovery_at))
            .unwrap();
        let current = owner.store.lock().unwrap().execution_unit(unit).unwrap();
        // Refresh the old future-due waiter to its ordinary bounded capacity recheck.
        execution::quota::QuotaScheduler::new(owner.clone())
            .admission(&current.authority(), now_ms())
            .unwrap();
        tokio::time::sleep(Duration::from_millis(1100)).await;
        let waiting_unit = owner.store.lock().unwrap().execution_unit(unit).unwrap();
        let profile = execution::resources::ResourceManager::new(owner.clone())
            .profile(&waiting_unit)
            .unwrap();
        let script = std::fs::read_to_string(&path).unwrap().replace("if sys.argv[1:] == [\"--version\"]:", "if sys.argv[1:] == [\"--version\"]:\n    with open(os.path.join(os.environ[\"RRX_OUTPUT_DIR\"], \"fixture-version-ready\"), \"w\") as ready: ready.write(\"ready\")\n    while not os.path.exists(os.path.join(os.environ[\"RRX_OUTPUT_DIR\"], \"fixture-version-release\")): time.sleep(0.02)");
        std::fs::write(&path, script).unwrap();
        let (seen_tx, seen_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = tokio::sync::oneshot::channel();
        *observer.hooks.before_wait_claim.lock().unwrap() = Some(Box::pin(async move {
            seen_tx.send(()).unwrap();
            release_rx.await.unwrap();
        }));
        let loser = tokio::spawn({
            let observer = observer.clone();
            async move { observer.step(task.id, BTreeMap::new()).await }
        });
        seen_rx.await.unwrap();
        let winner = tokio::spawn({
            let engine = engine.clone();
            async move { engine.step(task.id, BTreeMap::new()).await }
        });
        wait_file(&profile.output.join("fixture-version-ready")).await;
        let claimed_version = owner
            .store
            .lock()
            .unwrap()
            .records(&task.scope(), RecordKind::Workflow)
            .unwrap()[0]
            .version;
        release_tx.send(()).unwrap();
        assert!(matches!(
            loser.await.unwrap().unwrap(),
            StepResult::Waiting { .. }
        ));
        assert_eq!(
            owner
                .store
                .lock()
                .unwrap()
                .records(&task.scope(), RecordKind::Workflow)
                .unwrap()[0]
                .version,
            claimed_version
        );
        if exhausted_again {
            owner
                .store
                .lock()
                .unwrap()
                .observe_quota(&observation(QuotaStatus::Exhausted, recovery_at + 1))
                .unwrap();
        }
        std::fs::write(profile.output.join("fixture-version-release"), "release").unwrap();
        let outcome = winner.await.unwrap().unwrap();
        let workflow = engine.snapshot(task.id).unwrap();
        let attempt = &workflow.history[workflow.active.unwrap()];
        let current = owner.store.lock().unwrap().execution_unit(unit).unwrap();
        assert!(current.native_effects_open);
        assert_ne!(current.state, UnitState::Retired);
        if exhausted_again {
            assert!(matches!(outcome, StepResult::Waiting { .. }));
            assert_eq!(attempt.native_wait, Some(WaitReason::Quota));
            assert!(attempt.session_id.is_none());
        } else {
            assert!(
                matches!(
                    outcome,
                    StepResult::Started {
                        session: Some(_),
                        ..
                    }
                ),
                "winner outcome: {outcome:?}"
            );
            let identity = attempt.execution.as_ref().unwrap();
            assert_eq!(attempt.session_id, current.session_id);
            assert_eq!(identity.session, current.session_id.unwrap());
            let adapter = registry.get("native-alias").unwrap();
            adapter
                .stop(SessionRef {
                    id: identity.session,
                    scope: identity.scope.clone(),
                    execution: Some(identity.clone()),
                })
                .await
                .unwrap();
        }
    }
}
