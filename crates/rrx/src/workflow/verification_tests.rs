//! Actual managed Workflow commands, immutable evidence, and caller-Passed refusal.
use super::*;
use crate::execution::verification::{
    Applicability, Category, ManagedVerifier, TestCommand, TestsProfile,
};
use std::collections::BTreeSet;

fn profile(code: &str) -> TestsProfile {
    let python = std::fs::canonicalize("/usr/bin/python3").unwrap();
    let categories = [
        Category::Tests,
        Category::Typecheck,
        Category::Lint,
        Category::Build,
    ];
    TestsProfile {
        commands: categories
            .iter()
            .enumerate()
            .map(|(i, category)| TestCommand {
                id: format!("check_{i}"),
                category: *category,
                program: python.clone(),
                args: vec!["-c".into(), code.replace("COMMAND_INDEX", &i.to_string())],
                cwd: ".".into(),
                timeout_seconds: 10,
                drain_seconds: 1,
                stdout_bytes: 4096,
                stderr_bytes: 4096,
            })
            .collect(),
        applicability: categories
            .into_iter()
            .map(|c| (c, Applicability::Required))
            .collect(),
    }
}
async fn ready(
    provider: &str,
    proposal: Option<TestsProfile>,
) -> (
    Fixture,
    WorkflowEngine,
    Arc<ManagedVerifier>,
    execution::ArtifactId,
) {
    let f = Fixture::new(provider, WorkflowClass::Quick).await;
    let verifier = Arc::new(ManagedVerifier::new(f.owner.clone(), f.sources.clone()).unwrap());
    if let Some(proposal) = proposal {
        let version = f
            .owner
            .store
            .lock()
            .unwrap()
            .project(f.task.project_id)
            .unwrap()
            .unwrap()
            .version;
        verifier
            .admit_tests(f.task.project_id, version, proposal)
            .unwrap();
    }
    let engine = f
        .production_engine("pass")
        .with_verifier(verifier.clone())
        .unwrap();
    let artifact = publish(&f, &engine, f.task.id, provider).await;
    (f, engine, verifier, artifact)
}
async fn publish(
    f: &Fixture,
    engine: &WorkflowEngine,
    task: TaskId,
    provider: &str,
) -> execution::ArtifactId {
    let prepared = f.sources.prepare(task, provider).await.unwrap();
    engine.initialize(task, None).await.unwrap();
    early_phases(engine, task, WorkflowClass::Quick).await;
    assert!(matches!(
        engine.step(task, BTreeMap::new()).await.unwrap(),
        StepResult::Started {
            phase: Phase::Implement,
            ..
        }
    ));
    let snapshot = engine.snapshot(task).unwrap();
    let identity = snapshot.history[snapshot.active.unwrap()]
        .execution
        .clone()
        .unwrap();
    let unit = f
        .owner
        .store
        .lock()
        .unwrap()
        .execution_unit(prepared.id)
        .unwrap();
    let output = execution::resources::ResourceManager::new(f.owner.clone())
        .profile(&unit)
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
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(matches!(
        engine.step(task, BTreeMap::new()).await.unwrap(),
        StepResult::Completed {
            phase: Phase::Implement
        }
    ));
    assert!(matches!(
        engine.step(task, BTreeMap::new()).await.unwrap(),
        StepResult::Completed {
            phase: Phase::Commit
        }
    ));
    engine.snapshot(task).unwrap().sources.artifact.unwrap()
}

#[tokio::test]
async fn actual_tests_commands_complete_on_retained_commit_without_agent_sessions() {
    for provider in ["claude", "codex"] {
        let proposal = profile(
            "import os,pathlib,sys; assert 'MANDATORY_COMMITTED_RULE_A' in pathlib.Path('rules.md').read_text(); assert pathlib.Path(os.environ['CARGO_TARGET_DIR']).is_absolute(); print('actual-check-COMMAND_INDEX'); print('diagnostic-COMMAND_INDEX',file=sys.stderr)",
        );
        let (f, engine, verifier, artifact) = ready(provider, Some(proposal)).await;
        let before_sessions = f
            .owner
            .store
            .lock()
            .unwrap()
            .records(&f.task.scope(), RecordKind::Session)
            .unwrap()
            .len();
        assert!(matches!(
            engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Completed {
                phase: Phase::Tests
            }
        ));
        let workflow = engine.snapshot(f.task.id).unwrap();
        let attempt = workflow
            .history
            .iter()
            .find(|a| a.phase == Phase::Tests)
            .unwrap();
        assert_eq!(attempt.state, AttemptState::Succeeded);
        assert!(attempt.session_id.is_none() && attempt.execution.is_none());
        let unit = attempt.unit.as_ref().unwrap().unit;
        let run = f
            .owner
            .store
            .lock()
            .unwrap()
            .verification_run(unit)
            .unwrap();
        assert!(run.certifying && !run.historical);
        assert_eq!(run.artifact, artifact);
        assert_eq!(run.commands.len(), 4);
        assert_eq!(run.work, execution::WorkOutcome::Success);
        for i in 0..4 {
            assert_eq!(
                verifier.inspect_stream(unit, i, false, 4096).unwrap(),
                format!("actual-check-{i}\n").as_bytes()
            );
            assert_eq!(
                verifier.inspect_stream(unit, i, true, 4096).unwrap(),
                format!("diagnostic-{i}\n").as_bytes()
            );
        }
        {
            let store = f.owner.store.lock().unwrap();
            assert_eq!(
                store
                    .records(&f.task.scope(), RecordKind::Session)
                    .unwrap()
                    .len(),
                before_sessions
            );
            let u = store.execution_unit(unit).unwrap();
            assert!(
                u.session_id.is_none() && !u.native_effects_open && !u.result_finalization_open
            );
        }
        let original = f
            .owner
            .store
            .lock()
            .unwrap()
            .result_artifact(artifact)
            .unwrap();
        assert_eq!(
            workflow.completed[&Phase::Tests].revision,
            original.revision
        );
        // An Executor survivor cannot replace the accepted immutable source.
        let task = f
            .owner
            .store
            .lock()
            .unwrap()
            .task(f.task.id)
            .unwrap()
            .unwrap();
        std::fs::write(
            task.worktree.unwrap().join("rules.md"),
            "survivor changed workspace",
        )
        .unwrap();
        execution::results::ResultStore::new(f.owner.clone())
            .verify(&original)
            .await
            .unwrap();
        assert_eq!(
            verifier.inspect_stream(unit, 0, false, 4096).unwrap(),
            b"actual-check-0\n"
        );
        engine.cancel(f.task.id, "test complete".into()).unwrap();
    }
}

#[tokio::test]
async fn activated_tests_contract_refuses_generic_passed_even_with_no_unit() {
    let (f, engine, _, _) = ready("codex", None).await;
    assert!(matches!(
        engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Waiting {
            phase: Phase::Tests,
            ..
        }
    ));
    let mut snapshot = engine.read(f.task.id).unwrap();
    let index = snapshot.workflow.active.unwrap();
    assert!(snapshot.workflow.history[index].unit.is_none());
    remove_attempt_blocker(&mut snapshot.task, &snapshot.workflow.history[index]);
    snapshot.workflow.history[index].claimed_observations =
        snapshot.workflow.history[index].observations.len();
    snapshot.workflow.history[index].state = AttemptState::Evaluating;
    engine.persist(&mut snapshot, None).unwrap();
    let source = engine
        .inputs(
            &snapshot.project,
            &snapshot.task,
            Phase::Tests,
            snapshot.workflow.workflow,
        )
        .await
        .unwrap()
        .1;
    let fake = GateOutcome::Passed(Evidence {
        scope: snapshot.task.scope(),
        phase: Phase::Tests,
        revision: source.revision.clone(),
        source_versions: source.source_versions.clone(),
        artifacts: vec!["fabricated-negative-only".into()],
        dependencies: source.source_versions.clone(),
        review_approved: None,
        session_id: None,
        context_version: snapshot.task.context_version,
    });
    engine
        .observe_gate(
            &mut snapshot,
            index,
            GateObservation {
                sources: authority_only(&source),
                outcome: Some(fake.clone()),
                error: None,
                at: now_ms(),
            },
        )
        .unwrap();
    let error = engine
        .apply_outcome(snapshot, index, source, fake)
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("managed Tests success requires actual private verification completion"),
        "actual generic writer guard must reject: {error:#}"
    );
    let workflow = engine.snapshot(f.task.id).unwrap();
    assert!(!workflow.completed.contains_key(&Phase::Tests));
    assert!(workflow.history[index].unit.is_none());
    engine
        .cancel(f.task.id, "negative complete".into())
        .unwrap();
}

#[tokio::test]
async fn actual_command_failure_overflow_timeout_and_source_drift_never_certify_tests() {
    for scenario in ["exit", "overflow", "timeout", "source"] {
        let code = match scenario {
            "exit" => "import sys; print('known failure'); sys.exit(3)",
            "overflow" => "print('x'*100000)",
            "timeout" => "import time; time.sleep(30)",
            "source" => {
                "import pathlib; p=pathlib.Path('rules.md'); p.chmod(0o600); p.write_text('changed input'); print('source writer exited zero')"
            }
            _ => unreachable!(),
        };
        let mut proposal = profile(code);
        if scenario == "overflow" {
            for c in &mut proposal.commands {
                c.stdout_bytes = 128
            }
        }
        if scenario == "timeout" {
            for c in &mut proposal.commands {
                c.timeout_seconds = 1
            }
        }
        let (f, engine, verifier, artifact) = ready("codex", Some(proposal)).await;
        let result = engine.step(f.task.id, BTreeMap::new()).await.unwrap();
        if scenario == "exit" {
            assert!(matches!(
                result,
                StepResult::Failed {
                    phase: Phase::Tests,
                    ..
                }
            ))
        } else {
            assert!(
                matches!(
                    result,
                    StepResult::Waiting {
                        phase: Phase::Tests,
                        ..
                    }
                ),
                "{scenario}: {result:?}"
            )
        }
        let workflow = engine.snapshot(f.task.id).unwrap();
        assert!(!workflow.completed.contains_key(&Phase::Tests));
        let attempt = workflow
            .history
            .iter()
            .find(|a| a.phase == Phase::Tests)
            .unwrap();
        let unit = attempt.unit.as_ref().unwrap().unit;
        let run = f
            .owner
            .store
            .lock()
            .unwrap()
            .verification_run(unit)
            .unwrap();
        assert!(!run.certifying);
        let current = f.owner.store.lock().unwrap().execution_unit(unit).unwrap();
        assert!(
            !current.native_effects_open
                && !current.result_finalization_open
                && current.session_id.is_none()
        );
        if scenario == "exit" {
            assert_eq!(run.work, execution::WorkOutcome::Failure);
            assert_eq!(run.commands[0].exit, Some(3));
        }
        if scenario == "overflow" {
            assert_eq!(
                run.commands[0].issue,
                Some(execution::verification::CaptureIssue::OutputOverflow)
            );
            assert_eq!(
                verifier.inspect_stream(unit, 0, false, 128).unwrap().len(),
                128
            );
        }
        if scenario == "timeout" {
            assert_eq!(
                run.commands[0].issue,
                Some(execution::verification::CaptureIssue::Timeout)
            );
        }
        if scenario == "source" {
            // A same-user writer may change modes; validation, rather than a
            // security-containment claim, prevents acceptance of changed input.
            assert_eq!(run.work, execution::WorkOutcome::Unknown);
            assert_eq!(run.commands.len(), 1);
            assert_eq!(run.commands[0].exit, Some(0));
        }
        let original = f
            .owner
            .store
            .lock()
            .unwrap()
            .result_artifact(artifact)
            .unwrap();
        execution::results::ResultStore::new(f.owner.clone())
            .verify(&original)
            .await
            .unwrap();
        engine
            .cancel(f.task.id, "negative complete".into())
            .unwrap();
    }
}

#[tokio::test]
async fn accepted_raw_stream_retrieval_rejects_tampering_and_short_budget_after_reopen() {
    let (f, engine, verifier, _) = ready(
        "claude",
        Some(profile("print('independent retained evidence')")),
    )
    .await;
    assert!(matches!(
        engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Completed {
            phase: Phase::Tests
        }
    ));
    let workflow = engine.snapshot(f.task.id).unwrap();
    let unit = workflow
        .history
        .iter()
        .find(|a| a.phase == Phase::Tests)
        .unwrap()
        .unit
        .as_ref()
        .unwrap()
        .unit;
    assert!(verifier.inspect_stream(unit, 0, false, 1).is_err());
    let reopened = Store::open(&f._dir.path().join("state.db")).unwrap();
    assert_eq!(reopened.verification_run(unit).unwrap().commands.len(), 4);
    drop(reopened);
    let path = f
        .owner
        .root
        .join("verification-evidence")
        .join(unit.to_string())
        .join("0")
        .join("stdout");
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::write(&path, "changed retained bytes").unwrap();
    assert!(
        verifier
            .inspect_stream(unit, 0, false, 4096)
            .unwrap_err()
            .to_string()
            .contains("digest mismatch")
    );
    assert_eq!(
        engine.snapshot(f.task.id).unwrap().completed[&Phase::Tests].revision,
        workflow.completed[&Phase::Tests].revision
    );
    engine
        .cancel(f.task.id, "retrieval complete".into())
        .unwrap();
}

#[tokio::test]
async fn actual_command_grant_rejects_native_session_helper_delegation_and_terminal_writers() {
    let (f, engine, verifier, _) = ready(
        "codex",
        Some(profile("print('command-only actual control')")),
    )
    .await;
    let owner = f.owner.clone();
    verifier.before_commands(Box::new(move |id| {
        let mut store = owner.store.lock().unwrap();
        let unit = store.execution_unit(id).unwrap();
        assert_eq!(unit.state, execution::UnitState::Preparing);
        let sessions = store
            .records(&unit.scope, RecordKind::Session)
            .unwrap()
            .len();
        let effects = store.managed_effects(id).unwrap().len();
        let session = Session {
            id: SessionId::new(),
            scope: unit.scope.clone(),
            agent: "verifier".into(),
            provider: unit.provider.clone(),
            role: SessionRole::Consultant,
            native_ref: None,
            pid: None,
            worktree: unit.worktree.clone(),
            state: SessionState::Starting,
            model: None,
            effort: None,
            recovery: json!({}),
            started_at: now_ms(),
        };
        assert!(
            store
                .register_execution_session(&unit.authority(), &session)
                .unwrap_err()
                .to_string()
                .contains("command-only verifier")
        );
        for kind in ["native_version", "docker_probe"] {
            assert!(
                store
                    .reserve_execution_helper(
                        &unit.authority(),
                        execution::OperationId::new(),
                        true,
                        &unit.worktree,
                        kind
                    )
                    .unwrap_err()
                    .to_string()
                    .contains("command-only verifier")
            );
        }
        let effect = execution::ManagedEffect {
            id: execution::OperationId::new(),
            unit_id: id,
            scope: unit.scope.clone(),
            kind: "docker_create".into(),
            idempotency_key: "guard-negative".into(),
            expected_target: "isolated-negative-container".into(),
            state: execution::EffectState::Pending,
            receipt: BTreeMap::new(),
            version: 1,
        };
        assert!(
            store
                .reserve_managed_effect(&unit.authority(), &effect)
                .unwrap_err()
                .to_string()
                .contains("command-only verifier")
        );
        assert!(
            store
                .finish_execution(
                    &unit.authority(),
                    execution::WorkOutcome::Success,
                    execution::Disposition::Completed
                )
                .unwrap_err()
                .to_string()
                .contains("owned command collector")
        );
        assert_eq!(
            store
                .records(&unit.scope, RecordKind::Session)
                .unwrap()
                .len(),
            sessions
        );
        assert_eq!(store.managed_effects(id).unwrap().len(), effects);
        assert!(store.execution_unit(id).unwrap().work.is_none());
    }));
    assert!(matches!(
        engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Completed {
            phase: Phase::Tests
        }
    ));
    engine.cancel(f.task.id, "guard complete".into()).unwrap();
}

#[tokio::test]
async fn actual_command_preparation_then_owner_drift_refuses_before_command_intent() {
    let (f, engine, verifier, _) = ready("codex", Some(profile("print('must not start')"))).await;
    let owner = f.owner.clone();
    let goal_id = f.task.goal_id;
    verifier.before_commands(Box::new(move |_| {
        let mut store = owner.store.lock().unwrap();
        let mut goal = store.goal(goal_id).unwrap().unwrap();
        goal.title = "changed semantic Goal".into();
        store.put_goal(&mut goal).unwrap();
    }));
    let result = engine.step(f.task.id, BTreeMap::new()).await;
    assert!(
        result.is_err()
            || matches!(
                result.unwrap(),
                StepResult::Waiting {
                    phase: Phase::Tests,
                    ..
                }
            )
    );
    let workflow = engine.snapshot(f.task.id).unwrap();
    assert!(!workflow.completed.contains_key(&Phase::Tests));
    let unit = workflow
        .history
        .iter()
        .find(|a| a.phase == Phase::Tests)
        .unwrap()
        .unit
        .as_ref()
        .unwrap()
        .unit;
    assert!(
        f.owner
            .store
            .lock()
            .unwrap()
            .managed_effects(unit)
            .unwrap()
            .iter()
            .all(|e| e.kind != "verification_command")
    );
    engine.cancel(f.task.id, "drift complete".into()).unwrap();
}

#[tokio::test]
async fn actual_command_cancel_retains_observed_receipt_without_accepting_tests() {
    let (f, engine, verifier, artifact) = ready(
        "codex",
        Some(profile("import os,pathlib,time; pathlib.Path(os.environ['TMPDIR']).joinpath('command-running').write_text('ready'); print('before cancellation',flush=True); time.sleep(30)")),
    ).await;
    let engine = Arc::new(engine);
    let running_engine = engine.clone();
    let task_id = f.task.id;
    let running = tokio::spawn(async move { running_engine.step(task_id, BTreeMap::new()).await });
    let unit = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let workflow = engine.snapshot(task_id).unwrap();
            if let Some(unit) = workflow
                .history
                .iter()
                .find(|a| a.phase == Phase::Tests)
                .and_then(|a| a.unit.as_ref())
            {
                let unit = f
                    .owner
                    .store
                    .lock()
                    .unwrap()
                    .execution_unit(unit.unit)
                    .unwrap();
                if !unit
                    .worktree
                    .parent()
                    .unwrap()
                    .join("profile.json")
                    .is_file()
                {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    continue;
                }
                let resources = execution::resources::ResourceManager::new(f.owner.clone())
                    .profile(&unit)
                    .unwrap();
                if resources.temp.join("command-running").is_file() {
                    break unit.id;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    engine
        .cancel(task_id, "actual command cancel control".into())
        .unwrap();
    let _conservative_result = tokio::time::timeout(Duration::from_secs(5), running)
        .await
        .unwrap()
        .unwrap();
    let workflow = engine.snapshot(task_id).unwrap();
    assert!(!workflow.completed.contains_key(&Phase::Tests));
    assert_eq!(
        f.owner
            .store
            .lock()
            .unwrap()
            .task(task_id)
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    let run = f
        .owner
        .store
        .lock()
        .unwrap()
        .verification_run(unit)
        .unwrap();
    assert!(!run.certifying && run.commands.len() == 1);
    assert_eq!(
        run.commands[0].issue,
        Some(execution::verification::CaptureIssue::Cancelled)
    );
    assert_eq!(
        verifier.inspect_stream(unit, 0, false, 4096).unwrap(),
        b"before cancellation\n"
    );
    let original = f
        .owner
        .store
        .lock()
        .unwrap()
        .result_artifact(artifact)
        .unwrap();
    execution::results::ResultStore::new(f.owner.clone())
        .verify(&original)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn four_actual_command_tasks_cancel_one_preserving_sibling_artifacts_and_receipts() {
    let f = Fixture::new("codex", WorkflowClass::Quick).await;
    let mut ids = vec![f.task.id];
    for i in 1..4 {
        let mut task = Task::new(
            f.task.project_id,
            f.task.goal_id,
            format!("command sibling {i}"),
            "native-alias".into(),
        );
        task.workflow = WorkflowClass::Quick;
        task.acceptance_criteria = vec!["retain the committed answer".into()];
        f.owner.store.lock().unwrap().put_task(&mut task).unwrap();
        ids.push(task.id);
    }
    let verifier = Arc::new(ManagedVerifier::new(f.owner.clone(), f.sources.clone()).unwrap());
    let version = f
        .owner
        .store
        .lock()
        .unwrap()
        .project(f.task.project_id)
        .unwrap()
        .unwrap()
        .version;
    let mut parallel_profile = profile(
        "import os,pathlib,time\np=pathlib.Path(os.environ['TMPDIR'])\nif COMMAND_INDEX == 0:\n p.joinpath('verification-ready').write_text('ready')\n while not p.joinpath('verification-release').exists(): time.sleep(0.01)\nprint(os.environ['RRX_UNIT_ID']+'-COMMAND_INDEX')",
    );
    // This is a correctness rendezvous, not a ten-second throughput claim when
    // other fixture controls and compiler jobs compete for the same host.
    for command in &mut parallel_profile.commands {
        command.timeout_seconds = 60;
    }
    verifier
        .admit_tests(f.task.project_id, version, parallel_profile)
        .unwrap();
    let engine = Arc::new(
        f.production_engine("pass")
            .with_verifier(verifier.clone())
            .unwrap(),
    );
    let mut artifacts = Vec::new();
    for task in &ids {
        artifacts.push(publish(&f, &engine, *task, "codex").await);
    }
    let mut running = Vec::new();
    for task in &ids {
        let engine = engine.clone();
        let task = *task;
        running.push(tokio::spawn(async move {
            engine.step(task, BTreeMap::new()).await
        }));
    }
    let profiles = tokio::time::timeout(Duration::from_secs(45), async {
        loop {
            for (i, handle) in running.iter_mut().enumerate() {
                if handle.is_finished() {
                    panic!(
                        "command Task {i} ended before four-task barrier: {:?}",
                        handle.await.unwrap()
                    );
                }
            }
            let mut profiles = Vec::new();
            for task in &ids {
                let w = engine.snapshot(*task).unwrap();
                if let Some(reference) = w
                    .history
                    .iter()
                    .find(|a| a.phase == Phase::Tests)
                    .and_then(|a| a.unit.as_ref())
                {
                    let unit = f
                        .owner
                        .store
                        .lock()
                        .unwrap()
                        .execution_unit(reference.unit)
                        .unwrap();
                    if !unit
                        .worktree
                        .parent()
                        .unwrap()
                        .join("profile.json")
                        .is_file()
                    {
                        continue;
                    }
                    let profile = execution::resources::ResourceManager::new(f.owner.clone())
                        .profile(&unit)
                        .unwrap();
                    if profile.temp.join("verification-ready").is_file() {
                        profiles.push(profile);
                    }
                }
            }
            if profiles.len() == 4 {
                break profiles;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        profiles
            .iter()
            .map(|p| &p.temp)
            .collect::<BTreeSet<_>>()
            .len(),
        4
    );
    assert_eq!(
        profiles
            .iter()
            .map(|p| (p.port_start, p.port_end))
            .collect::<BTreeSet<_>>()
            .len(),
        4
    );
    engine
        .cancel(ids[0], "one of four command Tasks cancelled".into())
        .unwrap();
    // An ordinary Executor survivor writes its discarded live namespace while
    // all sibling Verifier commands are still running against retained sources.
    for task in &ids {
        let task = f.owner.store.lock().unwrap().task(*task).unwrap().unwrap();
        std::fs::write(
            task.worktree.unwrap().join("rules.md"),
            "late executor survivor write",
        )
        .unwrap();
    }
    for profile in profiles.iter().skip(1) {
        std::fs::write(profile.temp.join("verification-release"), "continue").unwrap();
    }
    for (i, handle) in running.into_iter().enumerate() {
        let result = tokio::time::timeout(Duration::from_secs(10), handle)
            .await
            .unwrap()
            .unwrap();
        if i > 0 {
            assert!(matches!(
                result.unwrap(),
                StepResult::Completed {
                    phase: Phase::Tests
                }
            ));
        }
        let w = engine.snapshot(ids[i]).unwrap();
        let unit = w
            .history
            .iter()
            .find(|a| a.phase == Phase::Tests)
            .unwrap()
            .unit
            .as_ref()
            .unwrap()
            .unit;
        let run = f
            .owner
            .store
            .lock()
            .unwrap()
            .verification_run(unit)
            .unwrap();
        assert_eq!(run.artifact, artifacts[i]);
        if i == 0 {
            assert!(!w.completed.contains_key(&Phase::Tests) && !run.certifying);
            assert_eq!(
                run.commands[0].issue,
                Some(execution::verification::CaptureIssue::Cancelled)
            );
        } else {
            assert!(run.certifying && run.commands.len() == 4);
            for j in 0..4 {
                assert_eq!(
                    verifier.inspect_stream(unit, j, false, 4096).unwrap(),
                    format!("{unit}-{j}\n").as_bytes()
                );
            }
            assert_eq!(w.completed[&Phase::Tests].revision, run.revision);
        }
        let artifact = f
            .owner
            .store
            .lock()
            .unwrap()
            .result_artifact(artifacts[i])
            .unwrap();
        execution::results::ResultStore::new(f.owner.clone())
            .verify(&artifact)
            .await
            .unwrap();
    }
    for task in ids.into_iter().skip(1) {
        engine
            .cancel(task, "parallel control complete".into())
            .unwrap();
    }
}

#[tokio::test]
async fn actual_resource_admission_waiter_drop_and_task_cancel_create_no_unit_or_command() {
    let (f, engine, _, artifact) = ready("codex", Some(profile("print('must not start')"))).await;
    let engine = Arc::new(engine);
    let manager = execution::resources::ResourceManager::new(f.owner.clone());
    let before = f
        .owner
        .store
        .lock()
        .unwrap()
        .execution_units(Some(&f.task.scope()))
        .unwrap()
        .len();
    let admission = manager.admission().await;
    let owner = f.owner.clone();
    let task = f.task.id;
    let dropped = tokio::spawn(async move {
        execution::attempts::AttemptManager::new(owner)
            .prepare_snapshot(
                task,
                artifact,
                execution::UnitKind::Reviewer,
                "codex",
                "Review",
            )
            .await
    });
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(!dropped.is_finished());
    assert_eq!(
        f.owner
            .store
            .lock()
            .unwrap()
            .execution_units(Some(&f.task.scope()))
            .unwrap()
            .len(),
        before
    );
    dropped.abort();
    assert!(dropped.await.unwrap_err().is_cancelled());
    drop(admission);
    drop(
        tokio::time::timeout(Duration::from_secs(1), manager.admission())
            .await
            .unwrap(),
    );

    let admission = manager.admission().await;
    let running_engine = engine.clone();
    let cancelled = tokio::spawn(async move { running_engine.step(task, BTreeMap::new()).await });
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let workflow = engine.snapshot(task).unwrap();
            if workflow
                .history
                .iter()
                .any(|a| a.phase == Phase::Tests && a.state == AttemptState::Evaluating)
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(!cancelled.is_finished());
    assert_eq!(
        f.owner
            .store
            .lock()
            .unwrap()
            .execution_units(Some(&f.task.scope()))
            .unwrap()
            .len(),
        before
    );
    engine
        .cancel(task, "cancel during actual resource admission wait".into())
        .unwrap();
    drop(admission);
    let _conservative = tokio::time::timeout(Duration::from_secs(5), cancelled)
        .await
        .unwrap()
        .unwrap();
    let workflow = engine.snapshot(task).unwrap();
    assert!(!workflow.completed.contains_key(&Phase::Tests));
    assert!(
        workflow
            .history
            .iter()
            .find(|a| a.phase == Phase::Tests)
            .unwrap()
            .unit
            .is_none()
    );
    assert_eq!(
        f.owner
            .store
            .lock()
            .unwrap()
            .task(task)
            .unwrap()
            .unwrap()
            .state,
        TaskState::Cancelled
    );
    assert_eq!(
        f.owner
            .store
            .lock()
            .unwrap()
            .execution_units(Some(&f.task.scope()))
            .unwrap()
            .len(),
        before
    );
    drop(
        tokio::time::timeout(Duration::from_secs(1), manager.admission())
            .await
            .unwrap(),
    );
}

#[tokio::test]
async fn actual_collected_known_exit_survives_retention_io_failure_without_acceptance() {
    for exit in [0, 7] {
        let (f, engine, verifier, _) = ready(
            "codex",
            Some(profile(&format!(
                "import sys; print('known-before-retention',flush=True); sys.exit({exit})"
            ))),
        )
        .await;
        let owner = f.owner.clone();
        verifier.before_commands(Box::new(move |id| {
            let root = owner.root.join("verification-evidence");
            std::fs::create_dir_all(&root).unwrap();
            // Real filesystem failure at the actual retention producer, after
            // actual owned exit observation; no fake collected result is used.
            std::fs::write(root.join(id.to_string()), "block evidence directory").unwrap();
        }));
        assert!(matches!(
            engine.step(f.task.id, BTreeMap::new()).await.unwrap(),
            StepResult::Waiting {
                phase: Phase::Tests,
                ..
            }
        ));
        let workflow = engine.snapshot(f.task.id).unwrap();
        assert!(!workflow.completed.contains_key(&Phase::Tests));
        let unit = workflow
            .history
            .iter()
            .find(|a| a.phase == Phase::Tests)
            .unwrap()
            .unit
            .as_ref()
            .unwrap()
            .unit;
        let store = f.owner.store.lock().unwrap();
        let run = store.verification_run(unit).unwrap();
        assert!(!run.certifying && run.commands.len() == 1);
        assert!(run.commands[0].work_known);
        assert_eq!(run.commands[0].exit, Some(exit));
        let learned_work = if exit == 7 {
            execution::WorkOutcome::Failure
        } else {
            execution::WorkOutcome::Unknown
        };
        assert_eq!(run.work, learned_work);
        assert_eq!(
            run.commands[0].stdout.bytes,
            b"known-before-retention\n".len() as u64
        );
        assert_eq!(
            run.commands[0].stdout.sha256,
            format!("{:x}", Sha256::digest(b"known-before-retention\n"))
        );
        let current = store.execution_unit(unit).unwrap();
        assert_eq!(current.work, Some(learned_work));
        assert!(!current.native_effects_open && !current.result_finalization_open);
        assert_eq!(current.cleanup, execution::CleanupOutcome::Unknown);
        assert!(
            store
                .managed_effects(unit)
                .unwrap()
                .iter()
                .any(|e| e.kind == "verification_command"
                    && e.state == execution::EffectState::Confirmed)
        );
        drop(store);
        assert!(verifier.inspect_stream(unit, 0, false, 4096).is_err());
        engine
            .cancel(f.task.id, "retention fault complete".into())
            .unwrap();
    }
}

#[tokio::test]
async fn actual_workflow_future_abort_fences_command_unit_and_quarantines_unknown_work() {
    let (f,engine,_,artifact)=ready("codex",Some(profile("import os,pathlib,time; pathlib.Path(os.environ['TMPDIR']).joinpath('abort-ready').write_text('ready'); print('not completion',flush=True); time.sleep(30)"))).await;
    let engine = Arc::new(engine);
    let running_engine = engine.clone();
    let task = f.task.id;
    let running = tokio::spawn(async move { running_engine.step(task, BTreeMap::new()).await });
    let unit = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let workflow = engine.snapshot(task).unwrap();
            if let Some(reference) = workflow
                .history
                .iter()
                .find(|a| a.phase == Phase::Tests)
                .and_then(|a| a.unit.as_ref())
            {
                let unit = f
                    .owner
                    .store
                    .lock()
                    .unwrap()
                    .execution_unit(reference.unit)
                    .unwrap();
                if unit
                    .worktree
                    .parent()
                    .unwrap()
                    .join("profile.json")
                    .is_file()
                {
                    let profile = execution::resources::ResourceManager::new(f.owner.clone())
                        .profile(&unit)
                        .unwrap();
                    if profile.temp.join("abort-ready").is_file() {
                        break unit.id;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    running.abort();
    assert!(running.await.unwrap_err().is_cancelled());
    {
        let store = f.owner.store.lock().unwrap();
        let current = store.execution_unit(unit).unwrap();
        assert!(!current.native_effects_open && !current.result_finalization_open);
        assert_eq!(current.state, execution::UnitState::Retired);
        assert_eq!(current.disposition, execution::Disposition::Lost);
        assert_eq!(current.work, Some(execution::WorkOutcome::Unknown));
        assert_eq!(current.cleanup, execution::CleanupOutcome::Unknown);
        assert!(store.managed_effects(unit).unwrap().iter().any(|e| e.kind
            == "verification_command"
            && e.state == execution::EffectState::Unknown));
        assert!(
            store
                .execution_leases(unit)
                .unwrap()
                .iter()
                .all(|l| l.state == execution::LeaseState::Quarantined)
        );
        assert!(
            store
                .due_execution_cleanup(now_ms().saturating_add(1000), 32)
                .unwrap()
                .contains(&unit)
        );
        assert!(!store.verification_run(unit).unwrap().certifying);
    }
    assert!(
        !engine
            .snapshot(task)
            .unwrap()
            .completed
            .contains_key(&Phase::Tests)
    );
    let original = f
        .owner
        .store
        .lock()
        .unwrap()
        .result_artifact(artifact)
        .unwrap();
    execution::results::ResultStore::new(f.owner.clone())
        .verify(&original)
        .await
        .unwrap();
    engine
        .cancel(task, "aborted future accounted".into())
        .unwrap();
}
