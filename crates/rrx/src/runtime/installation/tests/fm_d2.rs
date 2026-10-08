//! FM §9 D2 analogs of the legacy-Engine `workflow::committed_source_tests`
//! positives (`:301`, `:470`-manifest, `:631` fronts), on the installed lane:
//! accepted ingress, a real Git Project with committed `config_ref` /
//! `rule_refs`, the installed issuer, the genuine Driver and the configured
//! protocol fixture in commit mode. The fixture is protocol wiring only, never
//! Native qualification. Each test names the part its legacy case keeps and
//! what is deferred (SC-N / evidence integration).
use super::success::{
    bound_unit, links, release_completion, release_unit, stored_task, wait_for, wait_normal_bound,
    workflow,
};
use super::*;
use crate::domain::ContextVersion;
use sha2::{Digest, Sha256};

const RULE_A: &str = "MANDATORY_COMMITTED_RULE_A: preserve the answer.\n";
const RULE_B: &str = "MANDATORY_LIVE_RULE_B: different instructions.\n";

fn git(root: &std::path::Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "SETUP: git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The installed protocol fixture (as `fixture_mode`, commit mode) that also
/// records the actual outgoing Native input bytes.
fn recording_fixture(provider: &str) -> ControlFixture {
    ControlFixture::configured(|dir| {
        let path = dir.join("configured-protocol-fixture");
        let source = include_str!("../../../execution/native/native_fixture.py");
        let source = source.replacen(
            "while True: time.sleep(0.02)",
            "while not os.path.exists(os.path.join(os.environ[\"RRX_OUTPUT_DIR\"], \"fixture-bootstrap-release\")): time.sleep(0.02)",
            1,
        );
        let record = "    value = json.loads(line)\n    if value.get('method') == 'turn/start' or value.get('type') == 'user':\n        with open(os.path.join(os.environ['RRX_OUTPUT_DIR'], 'fixture-native-input.json'), 'w') as recorded:\n            json.dump(value, recorded)\n";
        assert!(source.contains("    value = json.loads(line)\n"));
        let source = source.replacen("    value = json.loads(line)\n", record, 1);
        std::fs::write(
            &path,
            format!("#!/usr/bin/python3\nPROVIDER={provider:?}\nBOOTSTRAP_HOLD=True\n{source}"),
        )
        .unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut config = crate::config::Config {
            minimum_workflow: crate::config::WorkflowClass::Quick,
            ..Default::default()
        };
        config.workflow.risk_mapping = [crate::config::WorkflowClass::Quick; 4];
        config.agents.insert(
            "worker".into(),
            AgentConfig {
                provider: Some(provider.into()),
                command: vec![path.to_string_lossy().into()],
                compatibility: Some(NativeCompatConfig {
                    profile: "rrx-native-inherited-v1".into(),
                    cli_version: if provider == "claude" {
                        "2.1.294"
                    } else {
                        "codex-cli 0.160.0"
                    }
                    .into(),
                    settings: "inherited".into(),
                    user_hooks: vec![],
                }),
                ..Default::default()
            },
        );
        for alias in ["rev-a", "rev-b"] {
            config
                .agents
                .insert(alias.into(), config.agents["worker"].clone());
        }
        config
    })
}

pub(super) struct Committed {
    pub(super) f: ControlFixture,
    pub(super) task: Task,
    config_a: String,
}

/// A real Git Project whose committed `config_ref` (policy `class`,
/// `repo_map_tokens = 321`) and `rule_refs` (rule A) are registered before
/// `accept`; then live B (STRICT, 777, rule B) is written uncommitted before
/// any preparation (legitimate drift, §8.2).
pub(super) async fn committed(provider: &str, class: crate::config::WorkflowClass) -> Committed {
    let mut f = recording_fixture(provider);
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
    let root = f.project.root.clone();
    let class_name = match class {
        crate::config::WorkflowClass::Quick => "QUICK",
        crate::config::WorkflowClass::Standard => "STANDARD",
        crate::config::WorkflowClass::Strict => "STRICT",
    };
    let config_a =
        format!("minimum_workflow = \"{class_name}\"\n[context]\nrepo_map_tokens = 321\n");
    std::fs::write(root.join("workflow.toml"), &config_a).unwrap();
    std::fs::write(root.join("rules.md"), RULE_A).unwrap();
    git(&root, &["add", "workflow.toml", "rules.md"]);
    git(
        &root,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=.git/hooks",
            "commit",
            "-m",
            "committed input A",
        ],
    );
    f.project.config_ref = Some(root.join("workflow.toml"));
    f.project.rule_refs = vec![root.join("rules.md")];
    f.owner
        .store()
        .lock()
        .unwrap()
        .put_project(&mut f.project)
        .unwrap();
    let (_, tasks) = accept(&f, 1).await;
    // Live B exists before preparation: the registered Git producer must
    // consume the committed A.
    std::fs::write(
        root.join("workflow.toml"),
        "minimum_workflow = \"STRICT\"\n[context]\nrepo_map_tokens = 777\n",
    )
    .unwrap();
    std::fs::write(root.join("rules.md"), RULE_B).unwrap();
    Committed {
        f,
        task: tasks[0].clone(),
        config_a,
    }
}

/// The Context the bound attempt used is A, not B (revision = the Unit's
/// base, both source hashes, the budget and the payload).
fn assert_context_a(c: &Committed, unit: &crate::execution::ExecutionUnit) -> ContextVersion {
    let (_, snapshot) = workflow(&c.f, &c.task);
    let attempt = snapshot
        .history
        .iter()
        .find(|a| a.execution.as_ref().is_some_and(|e| e.unit == unit.id))
        .expect("bound attempt");
    let context =
        c.f.owner
            .store
            .lock()
            .unwrap()
            .context(&c.task.scope(), Some(attempt.context_version))
            .unwrap()
            .unwrap();
    assert_eq!(
        context.revision, unit.base_sha,
        "context revision is the base"
    );
    assert_eq!(
        context.source_hashes["rules:config"],
        format!("{:x}", Sha256::digest(c.config_a.as_bytes())),
        "committed config A"
    );
    assert_eq!(
        context.source_hashes["rules:rules.md"],
        format!("{:x}", Sha256::digest(RULE_A.as_bytes())),
        "committed rule A"
    );
    assert_eq!(context.data["budget"]["discretionary_tokens"], 321);
    let payload = context.data["payload"].as_str().unwrap();
    assert!(payload.contains(RULE_A.trim()) && !payload.contains(RULE_B.trim()));
    context
}

/// The recorded outgoing Native input equals the Context data exactly.
fn assert_wire(c: &Committed, unit: &crate::execution::ExecutionUnit, context: &ContextVersion) {
    let output = crate::execution::resources::ResourceManager::new(c.f.owner.clone())
        .profile(unit)
        .unwrap()
        .output;
    let wire: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.join("fixture-native-input.json")).unwrap())
            .unwrap();
    let payload = wire["params"]["input"][0]["text"]
        .as_str()
        .or_else(|| wire["message"]["content"].as_str())
        .unwrap();
    assert_eq!(payload, serde_json::to_string(&context.data).unwrap());
    assert_eq!(
        c.f.owner
            .store
            .lock()
            .unwrap()
            .managed_effects(unit.id)
            .unwrap()
            .iter()
            .filter(|e| e.kind == "native_input")
            .count(),
        1,
        "one native_input effect"
    );
}

/// Release only the fixture's bootstrap hold and wait for the recorded
/// outgoing Native input (the completion stays held).
async fn recorded_input(c: &Committed, unit: &crate::execution::ExecutionUnit) {
    let output = crate::execution::resources::ResourceManager::new(c.f.owner.clone())
        .profile(unit)
        .unwrap()
        .output;
    std::fs::write(output.join("fixture-bootstrap-release"), "release").unwrap();
    wait_for(
        || output.join("fixture-native-input.json").exists(),
        "SETUP: Native input not recorded",
        60,
    )
    .await;
}

pub(super) async fn wait_closed(c: &Committed, label: &str) {
    wait_for(
        || links(&c.f, &c.task).last().map(String::as_str) == Some("phase_closed"),
        &format!(
            "{label}: phase_closed absent; links {:?}; jobs {:?}",
            links(&c.f, &c.task),
            c.f.runtime.phase_jobs.observed_jobs()
        ),
        90,
    )
    .await;
}

/// D2 of `:631` Quick [claude, codex]: live rules B ignored, committed A
/// applies, the same initial Unit is adopted, the wire payload equals the
/// Context, Published and verified. Tails: claude — the S4-W refusal of the
/// Goal objective change (§8.3); codex — `rule_refs` cleared then "instructions
/// changed" is SC-N (deferred, no Commit step on the D2 lane).
async fn committed_quick(provider: &str) {
    let c = committed(provider, crate::config::WorkflowClass::Quick).await;
    c.f.runtime.start().await.unwrap();
    wait_normal_bound(&c.f, &c.task).await;
    let unit = bound_unit(&c.f, &c.task);
    let (_, snapshot) = workflow(&c.f, &c.task);
    assert_eq!(
        snapshot.workflow,
        crate::config::WorkflowClass::Quick,
        "{provider}: live B must not change the selected policy"
    );
    let context = assert_context_a(&c, &unit);
    recorded_input(&c, &unit).await;
    assert_wire(&c, &unit, &context);
    release_completion(&c.f, &c.task);
    wait_closed(&c, &format!("FM-D2 :631 {provider}")).await;
    let artifact = {
        let store = c.f.owner.store.lock().unwrap();
        let units = store.execution_units(Some(&c.task.scope())).unwrap();
        assert_eq!(units.len(), 1, "{provider}: the same single Unit");
        assert_eq!(units[0].id, unit.id);
        let done = store.execution_unit(unit.id).unwrap();
        assert_eq!(done.work, Some(crate::execution::WorkOutcome::Success));
        assert!(!done.native_effects_open && !done.result_finalization_open);
        let published = store.result_artifacts(&c.task.scope()).unwrap();
        assert_eq!(published.len(), 1);
        let artifact = published[0].clone();
        assert_eq!(artifact.state, crate::execution::ArtifactState::Published);
        assert_eq!(artifact.unit_id, unit.id);
        assert_ne!(artifact.revision, unit.base_sha);
        artifact
    };
    crate::execution::results::ResultStore::new(c.f.owner.clone())
        .verify(&artifact)
        .await
        .unwrap();
    if provider == "claude" {
        // §8.3 S4-W: the generic Goal writer refuses the instruction change
        // on accepted rows; the Goal, artifact and Native input are unchanged.
        let mut store = c.f.owner.store.lock().unwrap();
        let mut goal = store.goal(c.task.goal_id).unwrap().unwrap();
        goal.objective
            .push_str(" fixture changed accepted instruction");
        crate::runtime::assert_goal_change_refused(&mut store, goal);
        assert_eq!(
            serde_json::to_value(store.result_artifact(artifact.id).unwrap()).unwrap(),
            serde_json::to_value(&artifact).unwrap()
        );
        drop(store);
        assert_wire(&c, &unit, &context);
    }
    let _ = c.f.runtime.shutdown().await;
    finish(c.f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fm_d2_committed_source_631_claude_quick_front() {
    committed_quick("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fm_d2_committed_source_631_codex_quick_front() {
    committed_quick("codex").await;
}

/// D2 of `:631` Standard / Strict [claude, codex]: the committed policy
/// selects the class over live B, Requirements binds the same Unit, and the
/// wire payload equals the committed-A Context. Deferred: Requirements
/// evidence integration (SC-N-adjacent; SC6 shows the hold).
async fn committed_requirements(provider: &str, class: crate::config::WorkflowClass) {
    let c = committed(provider, class).await;
    c.f.runtime.start().await.unwrap();
    wait_normal_bound(&c.f, &c.task).await;
    let unit = bound_unit(&c.f, &c.task);
    let (_, snapshot) = workflow(&c.f, &c.task);
    assert_eq!(snapshot.workflow, class, "{provider}: committed policy");
    let attempt = &snapshot.history[snapshot.active.expect("Requirements open")];
    assert_eq!(attempt.phase, crate::workflow::Phase::Requirements);
    let context = assert_context_a(&c, &unit);
    recorded_input(&c, &unit).await;
    assert_wire(&c, &unit, &context);
    assert_eq!(
        c.f.owner
            .store
            .lock()
            .unwrap()
            .execution_units(Some(&c.task.scope()))
            .unwrap()
            .len(),
        1
    );
    release_unit(&c.f, &unit);
    let _ = c.f.runtime.shutdown().await;
    finish(c.f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fm_d2_committed_source_631_claude_standard_front() {
    committed_requirements("claude", crate::config::WorkflowClass::Standard).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fm_d2_committed_source_631_codex_standard_front() {
    committed_requirements("codex", crate::config::WorkflowClass::Standard).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fm_d2_committed_source_631_claude_strict_front() {
    committed_requirements("claude", crate::config::WorkflowClass::Strict).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fm_d2_committed_source_631_codex_strict_front() {
    committed_requirements("codex", crate::config::WorkflowClass::Strict).await;
}

/// D2 of `:301` [claude, codex]: the Worktree gate and the Implement closure
/// each leave a `managed_workflow_gate_v1` receipt (2), which survive a
/// reopen; a survivor edit of the abandoned worktree leaves the pinned commit
/// evidence verifiable. Deferred (SC-N): Commit, the 3rd receipt, the Tests
/// wait.
async fn gate_receipts(provider: &str) {
    let c = committed(provider, crate::config::WorkflowClass::Quick).await;
    c.f.runtime.start().await.unwrap();
    wait_normal_bound(&c.f, &c.task).await;
    let unit = bound_unit(&c.f, &c.task);
    release_completion(&c.f, &c.task);
    wait_closed(&c, &format!("FM-D2 :301 {provider}")).await;
    let (artifact, receipts) = {
        let store = c.f.owner.store.lock().unwrap();
        let artifacts = store.result_artifacts(&c.task.scope()).unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(
            artifacts[0].state,
            crate::execution::ArtifactState::Published
        );
        let receipts = store
            .records(&c.task.scope(), RecordKind::Verification)
            .unwrap();
        for record in &receipts {
            assert_eq!(record.data["schema"], "managed_workflow_gate_v1");
            assert_eq!(
                record.data["context_data_sha256"].as_str().unwrap().len(),
                64
            );
            assert_eq!(store.record(record.id).unwrap().unwrap().data, record.data);
        }
        (artifacts[0].clone(), receipts)
    };
    assert_eq!(
        receipts.len(),
        2,
        "{provider}: Worktree gate + Implement receipts: {:?}",
        receipts.iter().map(|r| &r.data).collect::<Vec<_>>()
    );
    let reopened = crate::state::Store::open(&c.f.state_path()).unwrap();
    assert_eq!(
        reopened
            .records(&c.task.scope(), RecordKind::Verification)
            .unwrap()
            .len(),
        2,
        "{provider}: receipts survive a reopen"
    );
    drop(reopened);
    // A survivor can alter its abandoned workspace; commit evidence stays pinned.
    std::fs::write(unit.worktree.join("fixture-result.txt"), "survivor\n").unwrap();
    crate::execution::results::ResultStore::new(c.f.owner.clone())
        .verify(&artifact)
        .await
        .unwrap();
    let _ = c.f.runtime.shutdown().await;
    finish(c.f).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fm_d2_committed_source_301_claude_gate_receipts() {
    gate_receipts("claude").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fm_d2_committed_source_301_codex_gate_receipts() {
    gate_receipts("codex").await;
}

/// D2 of `:470` [manifest]: with the settled evaluation held, the retained
/// manifest of the captured artifact is corrupted; the gate then leaves the
/// artifact unpublished, does not close Implement, and the Task is
/// unchanged.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fm_d2_committed_source_470_corrupted_manifest_is_never_published() {
    use crate::runtime::phase_jobs::{SETTLED_EVALUATION, WritePause};
    let c = committed("codex", crate::config::WorkflowClass::Quick).await;
    struct Release(Arc<WritePause>);
    impl Drop for Release {
        fn drop(&mut self) {
            self.0.release();
        }
    }
    let pause = Release(WritePause::arm_at(c.task.id, SETTLED_EVALUATION));
    c.f.runtime.start().await.unwrap();
    wait_normal_bound(&c.f, &c.task).await;
    let marked = stored_task(&c.f, &c.task);
    release_completion(&c.f, &c.task);
    wait_for(
        || pause.0.reached(),
        "SETUP: settled evaluation not reached",
        90,
    )
    .await;
    let artifact = {
        let store = c.f.owner.store.lock().unwrap();
        let artifacts = store.result_artifacts(&c.task.scope()).unwrap();
        assert_eq!(artifacts.len(), 1, "SETUP: captured artifact");
        assert_ne!(
            artifacts[0].state,
            crate::execution::ArtifactState::Published
        );
        artifacts[0].clone()
    };
    std::fs::write(&artifact.manifest, "corrupt").unwrap();
    pause.0.release();
    wait_for(
        || links(&c.f, &c.task).len() >= 3,
        &format!(
            "FM-D2 :470: gate_observed absent; links {:?}",
            links(&c.f, &c.task)
        ),
        60,
    )
    .await;
    let started = std::time::Instant::now();
    while started.elapsed() < Duration::from_secs(3) {
        c.f.runtime.wake.notify_one();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(
        links(&c.f, &c.task),
        ["session_bound", "gate_claim", "gate_observed"],
        "FM-D2 :470: no closure"
    );
    let (_, snapshot) = workflow(&c.f, &c.task);
    assert!(
        !snapshot
            .completed
            .contains_key(&crate::workflow::Phase::Implement),
        "FM-D2 :470: Implement not completed"
    );
    assert!(
        c.f.owner
            .store
            .lock()
            .unwrap()
            .result_artifacts(&c.task.scope())
            .unwrap()
            .iter()
            .all(|a| a.state != crate::execution::ArtifactState::Published),
        "FM-D2 :470: nothing Published"
    );
    assert_eq!(
        stored_task(&c.f, &c.task).version,
        marked.version,
        "FM-D2 :470: Task unchanged"
    );
    let _ = c.f.runtime.shutdown().await;
    finish(c.f).await;
}
