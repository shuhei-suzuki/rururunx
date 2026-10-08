//! Account-free controls through Registry and the legacy Workflow Engine.
//! Evidence ports here are test integrations, not a shipped Runtime gate policy.
//!
//! FM §7.4 D4 (a) / §8.3 F2 (R): every scenario here stepped the legacy
//! Engine (no production caller, S1) into a Native phase and then past it
//! (wait, retry, Commit, Tests, Review), which the D2 lane does not reach
//! (S2). Each now keeps its setup up to that step and asserts the typed
//! `ManagedBindingUnavailable` refusal with the §8.3 preimage unchanged. The
//! former positive subjects are owned by the SC-N continuation.
use super::*;
use crate::execution::{self, ArtifactState, WorkOutcome, results};
use std::path::Path;

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
/// FM §8.1 L: the accepted Quick plan's single `native-alias` Task on legacy
/// rows (no legacy `put_task` sibling), and the seed revision.
async fn legacy(
    reviewers: &'static [&'static str],
) -> (
    crate::runtime::LegacyFixture,
    Arc<execution::RuntimeOwner>,
    Task,
    String,
) {
    let (dir, owner) = crate::runtime::legacy_fixture(
        results::tests::seed,
        vec![crate::runtime::LegacyTask {
            workflow: WorkflowClass::Quick,
            risk: crate::domain::RiskClass::R0,
            reviewers,
            ..crate::runtime::LegacyTask::standard("workflow", "native-alias")
        }],
    )
    .await;
    let task = dir.task();
    assert_eq!(task.workflow, WorkflowClass::Quick, "SETUP: accepted class");
    let seed = results::text(
        &results::git(&dir.path().join("repo"), ["rev-parse", "HEAD"])
            .await
            .unwrap(),
    )
    .unwrap();
    (dir, owner, task, seed)
}
/// The fixture program for `provider`, with its former Workflow scenario.
fn program(dir: &Path, provider: &str, scenario: Option<&str>) -> std::path::PathBuf {
    let program = execution::native::tests::program(dir, provider);
    if let Some(scenario) = scenario {
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
    }
    program
}
fn engine(
    owner: &Arc<execution::RuntimeOwner>,
    config: Config,
    seed: String,
    control: &'static str,
) -> WorkflowEngine {
    let registry = Arc::new(AgentRegistry::from_managed_config(&config, owner.clone()).unwrap());
    WorkflowEngine::new(
        owner.store(),
        registry,
        config,
        Arc::new(Sources {
            owner: owner.clone(),
            seed,
            result: results::ResultStore::new(owner.clone()),
        }),
        Arc::new(Gates {
            owner: owner.clone(),
            control,
        }),
    )
    .unwrap()
}
/// FM §8.3 F2: initialize, complete Worktree, then the step into the Native
/// phase is refused with the typed `ManagedBindingUnavailable` before source
/// capture. Preimage: Task body, Workflow record, audit, Units, Sessions,
/// result artifacts and completed Git outputs (a launched Native fixture
/// would add a Unit/Session).
async fn f2(owner: &Arc<execution::RuntimeOwner>, engine: &WorkflowEngine, task: &Task) {
    engine.initialize(task.id, None).await.unwrap();
    assert!(matches!(
        engine.step(task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Completed {
            phase: Phase::Worktree
        }
    ));
    let preimage = || {
        let store = owner.store.lock().unwrap();
        let scope = task.scope();
        json!({
            "task": store.task(task.id).unwrap(),
            "workflow": store.records(&scope, RecordKind::Workflow).unwrap(),
            "events": store.events(&scope, 0, 1000).unwrap().len(),
            "units": store.execution_units(Some(&scope)).unwrap(),
            "sessions": store.records(&scope, RecordKind::Session).unwrap().len(),
            "artifacts": store.result_artifacts(&scope).unwrap().len(),
            "git": crate::git::observed_git_outputs(),
        })
    };
    let before = preimage();
    let error = engine
        .step(task.id, BTreeMap::new())
        .await
        .expect_err("FM F2: the legacy Native step must be refused");
    assert!(
        matches!(
            error.downcast_ref::<NativePreflightRefusal>(),
            Some(NativePreflightRefusal::ManagedBindingUnavailable)
        ),
        "FM F2: typed refusal: {error:#}"
    );
    assert_eq!(preimage(), before, "FM F2: preimage unchanged");
}

/// FM D4 R. Former subject: a live quota/subscription wait retains the same
/// Native attempt across steps until cancelled, and a live recovery resumes
/// it (claude/codex). Refused at the legacy Native step; SC-N owns the wait.
#[tokio::test]
async fn workflow_live_subscription_wait_retains_the_same_attempt_until_cancelled() {
    for (provider, scenario) in [
        ("claude", "quota-retry-held"),
        ("codex", "quota-retry-held"),
        ("claude", "quota-live-recovery-held"),
    ] {
        let (dir, owner, task, seed) = legacy(&[]).await;
        let config = configuration(provider, &program(dir.path(), provider, Some(scenario)));
        f2(&owner, &engine(&owner, config, seed, "publish"), &task).await;
    }
}

/// FM D4 R. Former subject: a Native quota/capacity terminal moves the Task
/// to WaitingQuota/WaitingCapacity and the retry uses fresh resources instead
/// of failing. Refused at the legacy Native step; SC-N owns retry.
#[tokio::test]
async fn workflow_native_capacity_terminal_waits_with_fresh_resources_instead_of_failing() {
    for (provider, scenario) in [
        ("codex", "quota-terminal"),
        ("codex", "capacity-rate"),
        ("claude", "quota-stale-available"),
        ("claude", "capacity-rate"),
    ] {
        let (dir, owner, task, seed) = legacy(&[]).await;
        let config = configuration(provider, &program(dir.path(), provider, Some(scenario)));
        f2(&owner, &engine(&owner, config, seed, "publish"), &task).await;
    }
}

/// FM D4 R. Former subject: the managed Review reads the retained snapshot
/// (after Commit and an admitted Tests command) and refuses a changed review
/// input or artifact, or a cancelled executor. Refused at the legacy Native
/// Implement step with the Tests profile admitted and the source prepared;
/// SC-N owns Commit, Tests and Review.
#[tokio::test]
async fn managed_workflow_reviews_retained_snapshot_and_rejects_changed_review_input() {
    use execution::verification::{Applicability, Category, TestCommand, TestsProfile};
    let (dir, owner, task, _) = legacy(&["native-reviewer"]).await;
    assert_eq!(
        task.reviewers,
        ["native-reviewer"],
        "SETUP: planned reviewer"
    );
    let executor = program(dir.path(), "codex", None);
    let reviewer = program(dir.path(), "claude", Some("readonly-review"));
    let mut config = configuration("codex", &executor);
    config.agents.insert(
        "native-reviewer".into(),
        crate::config::AgentConfig {
            provider: Some("claude".into()),
            command: vec![reviewer.to_string_lossy().into()],
            ..Default::default()
        },
    );
    let registry = Arc::new(AgentRegistry::from_managed_config(&config, owner.clone()).unwrap());
    let sources = Arc::new(
        execution::workflow_source::ManagedWorkflowSources::new(owner.clone(), config.clone())
            .unwrap(),
    );
    let verifier = Arc::new(
        execution::verification::ManagedVerifier::new(owner.clone(), sources.clone()).unwrap(),
    );
    let project_version = owner
        .store
        .lock()
        .unwrap()
        .project(task.project_id)
        .unwrap()
        .unwrap()
        .version;
    let not_applicable = |reason: &str| Applicability::NotApplicable {
        reason: reason.into(),
    };
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
                        not_applicable("fixture has no typed sources"),
                    ),
                    (
                        Category::Lint,
                        not_applicable("fixture has no lint toolchain"),
                    ),
                    (
                        Category::Build,
                        not_applicable("fixture contains data only"),
                    ),
                ]),
            },
        )
        .unwrap();
    sources.prepare(task.id, "codex").await.unwrap();
    let engine = WorkflowEngine::new(
        owner.store(),
        registry,
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
    f2(&owner, &engine, &task).await;
}

/// FM D4 R. Former subject: the managed Registry Workflow qualifies the real
/// retained content (manifest, commit ref, cancel, artifact version) before
/// the atomic publication (claude/codex). Refused at the legacy Native step;
/// the D2 analog of the corrupted-manifest hold is
/// `fm_d2_committed_source_470_corrupted_manifest_is_never_published`, and
/// SC-N owns the rest.
#[tokio::test]
async fn managed_registry_workflow_qualifies_real_retained_content_before_atomic_publication() {
    for provider in ["codex", "claude"] {
        let (dir, owner, task, seed) = legacy(&[]).await;
        let config = configuration(provider, &program(dir.path(), provider, None));
        f2(&owner, &engine(&owner, config, seed, "publish"), &task).await;
    }
}

/// FM D4 R. Former subject: two Engines racing a due quota-wait claim
/// preserve the single Native launch or wait owner. Refused at the legacy
/// Native step with the exhausted quota observed; SC-N owns the wait claim.
#[tokio::test]
async fn concurrent_due_wait_claim_preserves_the_native_launch_or_wait_owner() {
    use execution::{QuotaObservation, QuotaStatus};
    let (dir, owner, task, seed) = legacy(&[]).await;
    let config = configuration("codex", &program(dir.path(), "codex", None));
    let at = now_ms();
    owner
        .store
        .lock()
        .unwrap()
        .observe_quota(&QuotaObservation {
            provider: "codex".into(),
            account_key: "unknown".into(),
            bucket: "claim-fixture".into(),
            confirmed_subscription: true,
            window_id: "fixture".into(),
            status: QuotaStatus::Exhausted,
            used_percent: None,
            resets_at: Some(at + 3_600_000),
            observed_at: at,
            source_version: "fixture".into(),
        })
        .unwrap();
    f2(&owner, &engine(&owner, config, seed, "publish"), &task).await;
}
