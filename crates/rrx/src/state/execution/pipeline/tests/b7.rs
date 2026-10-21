//! Host cases use real authority and the pinned native fixtures, never SQL success seeds.
use super::*;
use crate::execution::pipeline::{
    LaunchResult, PipelineGit, PipelineRole, ReviewSeverity, StepInput, review, review_prompt,
    review_snapshot, step_arguments, test_support::FakeCli,
};
use std::{os::unix::fs::PermissionsExt, path::PathBuf, sync::Arc};

pub(super) struct ReviewFixture {
    pub(super) f: Fixture,
    pub(super) runtime: Arc<Runtime>,
    pub(super) id: String,
    pub(super) fake: FakeCli,
    pub(super) path: PathBuf,
    pub(super) attempt: PathBuf,
}
impl ReviewFixture {
    pub(super) async fn new(output: &str) -> Self {
        Self::build(output, None).await
    }
    /// Plans the review on an adopted Executor commit that adds `source` to the base tree.
    async fn after_commit(output: &str, source: &str) -> Self {
        Self::build(output, Some(source)).await
    }
    async fn build(output: &str, source: Option<&str>) -> Self {
        let (f, runtime, id) = super::b5::prepare_fixture_from(fixture().await, true).await;
        runtime.wait_pipeline_test_boundary().await;
        assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
        let seq = match source {
            Some(source) => {
                adopt_executor_commit(&f, &runtime, &id, source).await;
                3
            }
            None => 2,
        };
        let view = f.store().pipeline_goal_view(f.goal).unwrap();
        let task = f.store().task(f.task).unwrap().unwrap();
        let data = json!({"title":task.title,"acceptance_criteria":task.acceptance_criteria,"scope":["src/**"]});
        let nonce = Uuid::new_v4().simple().to_string();
        let project = f.store().project(f.project).unwrap().unwrap();
        let diff = PipelineGit::new(&project.root)
            .unwrap()
            .diff(&view.runs[0].base_commit, &view.runs[0].head_commit)
            .await
            .unwrap();
        let prompt = review_prompt(&nonce, &data, &diff).unwrap();
        let mut next = planned(&id, seq, 1, "review");
        next.step_base = view.runs[0].head_commit.clone();
        next.input_sha256 = crate::execution::verification::hash(&prompt);
        next.body["review_nonce"] = json!(nonce);
        f.seed_next(&id, &next);
        let config = f.config.pipeline.as_ref().unwrap();
        let path = review_snapshot::snapshot_path(&config.work_root, &id, seq, 1).unwrap();
        let attempt = config
            .work_root
            .join("runs")
            .join(&id)
            .join(format!("{seq}-1"));
        let fake = FakeCli {
            directory: config.clis.claude.executable.parent().unwrap().into(),
            cli: config.clis.claude.clone(),
        };
        let args = step_arguments(
            PipelineRole::Review,
            &fake.cli,
            &path,
            &attempt,
            review::REVIEW_SCHEMA,
        )
        .unwrap();
        fake.expect_arguments(&args.iter().map(String::as_str).collect::<Vec<_>>());
        let output = output.replace("PINNED_MODEL", &fake.cli.model);
        fake.control("exec-output", &output);
        Self {
            f,
            runtime,
            id,
            fake,
            path,
            attempt,
        }
    }
    fn step(&self) -> PipelineStep {
        self.f
            .store()
            .pipeline_goal_view(self.f.goal)
            .unwrap()
            .steps
            .last()
            .unwrap()
            .clone()
    }
    pub(super) async fn prepare(&self) -> Option<StepInput> {
        let before = self.step();
        if let Some(input) = self
            .runtime
            .prepare_pipeline_review(self.f.cursor(&self.id))
            .await
            .unwrap()
        {
            return Some(input);
        }
        if self.step() != before {
            return None;
        }
        self.runtime.wait_pipeline_test_boundary().await;
        self.runtime
            .prepare_pipeline_review(self.f.cursor(&self.id))
            .await
            .unwrap()
    }
    async fn launch(&self) -> LaunchResult {
        let before = self.step();
        let result = self
            .runtime
            .launch_pipeline_review(self.f.cursor(&self.id))
            .await
            .unwrap();
        if !matches!(result, LaunchResult::Waiting) || self.step() != before {
            return result;
        }
        self.runtime.wait_pipeline_test_boundary().await;
        let result = self
            .runtime
            .launch_pipeline_review(self.f.cursor(&self.id))
            .await
            .unwrap();
        assert!(
            !matches!(result, LaunchResult::Waiting),
            "unchanged launch still waiting with free boundary"
        );
        result
    }
    async fn exited(&self) {
        let LaunchResult::Started(custody) = self.launch().await else {
            panic!("review did not start: {:?}", self.step());
        };
        custody.wait().await.unwrap();
    }
    async fn adopt(&self) -> bool {
        let before = self.step();
        let result = self
            .runtime
            .adopt_pipeline_round_review(self.f.cursor(&self.id))
            .await
            .unwrap();
        if result || self.step() != before {
            return result;
        }
        self.runtime.wait_pipeline_test_boundary().await;
        self.runtime
            .adopt_pipeline_round_review(self.f.cursor(&self.id))
            .await
            .unwrap()
    }
    async fn close(self) {
        self.runtime.shutdown().await.unwrap();
        let steps = self
            .f
            .store()
            .pipeline_goal_view(self.f.goal)
            .unwrap()
            .steps;
        for step in steps {
            if step.run_id == self.id && step.kind == "review" {
                let path = review_snapshot::snapshot_path(
                    &self.f.config.pipeline.as_ref().unwrap().work_root,
                    &self.id,
                    step.seq,
                    step.attempt,
                )
                .unwrap();
                review_snapshot::remove(&path).unwrap();
            }
        }
    }
}

/// Adopts one real Executor commit so the run head differs from its base.
async fn adopt_executor_commit(f: &Fixture, runtime: &Runtime, id: &str, source: &str) {
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let work = PathBuf::from(view.runs[0].worktree.as_ref().unwrap());
    let prompt = b"Implement the accepted Task.\n";
    let mut next = planned(id, 2, 1, "implement");
    next.step_base = view.runs[0].base_commit.clone();
    next.input_sha256 = crate::execution::verification::hash(prompt);
    let cursor = f.cursor(id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap();
    let config = f.config.pipeline.as_ref().unwrap();
    let codex = FakeCli {
        directory: config.clis.codex.executable.parent().unwrap().into(),
        cli: config.clis.codex.clone(),
    };
    let attempt = config.work_root.join("runs").join(id).join("2-1");
    let args = step_arguments(PipelineRole::Implement, &codex.cli, &work, &attempt, "").unwrap();
    codex.expect_arguments(&args.iter().map(String::as_str).collect::<Vec<_>>());
    codex.control("exec-source", source);
    codex.control("exec-last", "Implementation complete.\n");
    codex.control("exec-output", "{\"type\":\"turn.completed\"}\n");
    runtime.wait_pipeline_test_boundary().await;
    let LaunchResult::Started(custody) = runtime
        .launch_pipeline_executor(f.cursor(id), prompt.to_vec())
        .await
        .unwrap()
    else {
        panic!("Executor did not start");
    };
    custody.wait().await.unwrap();
    runtime.wait_pipeline_test_boundary().await;
    assert!(runtime.adopt_pipeline_executor(f.cursor(id)).await.unwrap());
}

/// The generic entry point refuses every missing preflight or substituted input before tx1.
#[tokio::test]
async fn b7_generic_review_preflight() {
    for (case, reason) in [
        ("unprepared", "review_snapshot_missing"),
        ("tree_type", "review_snapshot_missing"),
        ("tree_mismatch", "review_snapshot_changed"),
        ("nonce_missing", "review_nonce_invalid"),
        ("unverified", "review_nonce_unverified"),
        ("work", "review_snapshot_path"),
        ("schema", "review_schema_mismatch"),
        ("prompt", "review_input_changed"),
        ("secret", "secret_detected"),
        ("escaped_secret", "secret_detected"),
        ("oversized_prompt", "review_prompt_too_large"),
    ] {
        let h = ReviewFixture::new(&success()).await;
        h.runtime.wait_pipeline_test_boundary().await;
        let mut input = if case == "unprepared" {
            StepInput {
                work: h.f.config.pipeline.as_ref().unwrap().work_root.clone(),
                prompt: vec![],
                schema: "{}".into(),
            }
        } else {
            h.prepare().await.unwrap()
        };
        if matches!(
            case,
            "tree_type" | "tree_mismatch" | "nonce_missing" | "unverified"
        ) {
            let mut step = h.step();
            match case {
                "tree_type" => step.body["snapshot_tree"] = json!(7),
                "tree_mismatch" => step.body["snapshot_tree"] = json!("0".repeat(40)),
                "nonce_missing" => {
                    step.body.as_object_mut().unwrap().remove("review_nonce");
                }
                "unverified" => {
                    step.body
                        .as_object_mut()
                        .unwrap()
                        .remove("review_nonce_verified");
                }
                _ => unreachable!(),
            }
            let mut store = h.f.store();
            let tx = store.connection.transaction().unwrap();
            write_step(&tx, &h.f.owner, &step).unwrap();
            tx.commit().unwrap();
        }
        match case {
            "work" => input.work = h.f.config.pipeline.as_ref().unwrap().work_root.clone(),
            "schema" => input.schema = "{}".into(),
            "prompt" => input.prompt = b"substituted prompt".to_vec(),
            "secret" => input.prompt = b"EXAMPLE_TOKEN=fixture".to_vec(),
            "escaped_secret" => input.prompt = br#"{"k":"gh\u0070_fixture"}"#.to_vec(),
            "oversized_prompt" => input.prompt = vec![b'x'; 2 * 1024 * 1024],
            _ => {}
        }
        h.runtime.wait_pipeline_test_boundary().await;
        let result = h
            .runtime
            .launch_pipeline_step(h.f.cursor(&h.id), input)
            .await
            .unwrap();
        assert!(matches!(result, LaunchResult::Held), "{case}");
        let step = h.step();
        assert_eq!(step.state, "held", "{case}");
        assert_eq!(step.body["reason"], reason, "{case}");
        assert!(
            step.cookie.is_none() && step.pid.is_none() && step.body["intent"].is_null(),
            "{case}"
        );
        assert!(!h.fake.directory.join("exec-entered").exists(), "{case}");
        h.close().await;
    }
}

/// Executor prompt escapes cannot conceal secrets from the pre-launch scan.
#[tokio::test]
async fn b7_executor_prompt_escaped_secret() {
    let (f, runtime, id) = super::b5::prepare_fixture_from(fixture().await, true).await;
    runtime.wait_pipeline_test_boundary().await;
    assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    let prompt = br#"{"k":"gh\u0070_fixture"}"#;
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let mut next = planned(&id, 2, 1, "implement");
    next.step_base = view.runs[0].head_commit.clone();
    next.input_sha256 = crate::execution::verification::hash(prompt);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap();
    runtime.wait_pipeline_test_boundary().await;
    assert!(matches!(
        runtime
            .launch_pipeline_executor(f.cursor(&id), prompt.to_vec())
            .await
            .unwrap(),
        LaunchResult::Held
    ));
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let step = view.steps.last().unwrap();
    assert_eq!(step.state, "held");
    assert_eq!(step.body["reason"], "secret_detected");
    assert_eq!(step.body["secret"]["pattern"], "fixed_secret");
    assert!(step.cookie.is_none() && step.pid.is_none() && step.body["intent"].is_null());
    assert!(
        !serde_json::to_string(&step.body)
            .unwrap()
            .contains("fixture")
    );
    let cli = f
        .config
        .pipeline
        .as_ref()
        .unwrap()
        .clis
        .codex
        .executable
        .parent()
        .unwrap();
    assert!(!cli.join("exec-entered").exists());
    runtime.shutdown().await.unwrap();
}

/// Oversized Executor prompts are rejected by launch preflight without entering the CLI.
#[tokio::test]
async fn b7_executor_oversized_prompt_rejected() {
    let (f, runtime, id) = super::b5::prepare_fixture_from(fixture().await, true).await;
    runtime.wait_pipeline_test_boundary().await;
    assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    let prompt = vec![b'x'; 2 * 1024 * 1024];
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let mut next = planned(&id, 2, 1, "implement");
    next.step_base = view.runs[0].head_commit.clone();
    next.input_sha256 = crate::execution::verification::hash(&prompt);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap();
    runtime.wait_pipeline_test_boundary().await;
    assert!(matches!(
        runtime
            .launch_pipeline_executor(f.cursor(&id), prompt)
            .await
            .unwrap(),
        LaunchResult::Held
    ));
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let step = view.steps.last().unwrap();
    assert_eq!(step.state, "rejected");
    assert_eq!(step.body["reason"], "launch_preflight_failed");
    let config = f.config.pipeline.as_ref().unwrap();
    let cli = config.clis.codex.executable.parent().unwrap();
    assert!(!cli.join("exec-entered").exists());
    assert!(step.cookie.is_some() && step.pid.is_none());
    assert!(step.body["intent"].is_object());
    assert!(
        !config
            .work_root
            .join("runs")
            .join(&id)
            .join("2-1/prompt.md")
            .exists()
    );
    runtime.shutdown().await.unwrap();
}

/// A large review diff holds during preparation with its prompt budget reason.
#[tokio::test]
async fn b7_runtime_prompt_size_reason() {
    let (f, runtime, id) = super::b5::prepare_fixture_from(fixture().await, true).await;
    runtime.wait_pipeline_test_boundary().await;
    assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    // Added-line prefixes produce a ~2 MiB diff within the 1 MiB file budget.
    adopt_executor_commit(&f, &runtime, &id, &"\n".repeat(1024 * 1024 - 1024)).await;
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let project = f.store().project(f.project).unwrap().unwrap();
    let diff = PipelineGit::new(&project.root)
        .unwrap()
        .diff(&view.runs[0].base_commit, &view.runs[0].head_commit)
        .await
        .unwrap();
    assert!((1900 * 1024..2100 * 1024).contains(&diff.len()));
    let mut next = planned(&id, 3, 1, "review");
    next.step_base = view.runs[0].head_commit.clone();
    next.body["review_nonce"] = json!(Uuid::new_v4().simple().to_string());
    f.seed_next(&id, &next);
    runtime.wait_pipeline_test_boundary().await;
    assert!(
        runtime
            .prepare_pipeline_review(f.cursor(&id))
            .await
            .unwrap()
            .is_none()
    );
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let step = view.steps.last().unwrap();
    assert_eq!(step.state, "held");
    assert_eq!(step.body["reason"], "review_prompt_too_large");
    assert!(step.cookie.is_none() && step.pid.is_none() && step.body["intent"].is_null());
    let path =
        review_snapshot::snapshot_path(&f.config.pipeline.as_ref().unwrap().work_root, &id, 3, 1)
            .unwrap();
    assert!(!path.exists());
    runtime.shutdown().await.unwrap();
}

/// Stream scan failures hold the review with only their distinct failure marker.
#[tokio::test]
async fn b7_store_stream_scan_failure() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "review");
    let mut scan = review::ReviewStreamScan::default();
    scan.append(&vec![b'x'; review::REVIEW_BYTES + 1537]);
    let marker = scan.hit.unwrap();
    let reason = review::stream_scan_reason(&marker);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_finish_step(
            &f.owner,
            &cursor,
            &death(),
            PipelineExit {
                outcome: Some("other"),
                evidence: &json!({"issue":reason,"secret":marker}),
                issue: Some(reason),
            },
        )
        .unwrap();
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let step = view.steps.last().unwrap();
    assert_eq!(step.state, "held");
    assert_eq!(step.body["reason"], "secret_scan_failed");
    assert_eq!(step.body["secret"], json!({"pattern":"scan_failed"}));
    assert_eq!(
        step.body["exit"]["secret"],
        json!({"pattern":"scan_failed"})
    );
}

/// Preparation returns an input only after recording evidence accepted by generic launch.
#[tokio::test]
async fn b7_prepared_generic_launch() {
    let h = ReviewFixture::new(&success()).await;
    let input = h.prepare().await.unwrap();
    let step = h.step();
    assert!(step.body["snapshot_tree"].is_string());
    assert_eq!(
        step.body["review_nonce_verified"],
        step.body["review_nonce"]
    );
    h.runtime.wait_pipeline_test_boundary().await;
    let LaunchResult::Started(custody) = h
        .runtime
        .launch_pipeline_step(h.f.cursor(&h.id), input)
        .await
        .unwrap()
    else {
        panic!("prepared generic review did not launch");
    };
    custody.wait().await.unwrap();
    assert!(h.adopt().await);
    h.close().await;
}

async fn lifecycle(h: &ReviewFixture, pause: bool) {
    let goal = h.f.store().goal(h.f.goal).unwrap().unwrap();
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: h.f.owner.instance.clone(),
        epoch: h.f.owner.epoch,
        action: ControlAction::SetGoalLifecycle {
            project: h.f.project,
            goal: h.f.goal,
            expected_goal: goal.version,
            target: if pause {
                crate::runtime::control::GoalControl::Pause
            } else {
                crate::runtime::control::GoalControl::Resume
            },
            reason: "review lifecycle test".into(),
        },
    };
    let (socket, _peer) = tokio::net::UnixStream::pair().unwrap();
    h.runtime.wait_pipeline_test_boundary().await;
    h.runtime.handle_control(&socket, request).await.unwrap();
}

/// Pausing at each review boundary fails without changing the step (pipeline resume is B10).
#[tokio::test]
async fn b7_paused_boundaries_unchanged() {
    for phase in ["prepare", "launch", "adopt"] {
        let h = ReviewFixture::new(&success()).await;
        if phase == "adopt" {
            h.exited().await;
        }
        let before = h.step();
        lifecycle(&h, true).await;
        h.runtime.wait_pipeline_test_boundary().await;
        let cursor = h.f.cursor(&h.id);
        let error = match phase {
            "prepare" => h.runtime.prepare_pipeline_review(cursor).await.err(),
            "launch" => h.runtime.launch_pipeline_review(cursor).await.err(),
            _ => h.runtime.adopt_pipeline_round_review(cursor).await.err(),
        }
        .expect("paused review must fail without mutation");
        assert_eq!(error.to_string(), "pipeline Goal is not Running");
        assert_eq!(h.step(), before, "{phase}");
        h.close().await;
    }
}

/// Stale cursors fail at every review boundary without converting CAS loss to held.
#[tokio::test]
async fn b7_stale_review_boundaries() {
    let h = ReviewFixture::new(&success()).await;
    let stale = h.f.cursor(&h.id);
    assert!(h.prepare().await.is_some());
    let before = h.step();
    h.runtime.wait_pipeline_test_boundary().await;
    assert!(
        h.runtime
            .prepare_pipeline_review(stale.clone())
            .await
            .is_err()
    );
    assert!(
        h.runtime
            .launch_pipeline_review(stale.clone())
            .await
            .is_err()
    );
    assert!(h.runtime.adopt_pipeline_round_review(stale).await.is_err());
    assert_eq!(h.step(), before);
    h.close().await;
}

/// The Store refuses a decoded verdict whose source bytes differ from the stdout receipt.
#[tokio::test]
async fn b7_store_result_digest() {
    let h = ReviewFixture::new(&success()).await;
    h.exited().await;
    let before = h.step();
    let mut bytes = success()
        .replace("PINNED_MODEL", &h.fake.cli.model)
        .into_bytes();
    bytes.push(b' ');
    let mut evidence =
        review::VerdictDecoder::decode(&bytes, &h.fake.cli.model, ReviewSeverity::High).unwrap();
    evidence["snapshot_tree"] = before.body["snapshot_tree"].clone();
    let cursor = h.f.cursor(&h.id);
    let error =
        h.f.store()
            .pipeline_stage_round_result(&h.f.owner, &cursor, &evidence)
            .unwrap_err();
    assert_eq!(error.to_string(), "review_output_changed");
    assert_eq!(h.step(), before);
    assert!(h.adopt().await);
    h.close().await;
}

/// Escaped secrets in malformed streamed output hold the review with a receipt.
#[tokio::test]
async fn b7_runtime_escaped_secret() {
    let h =
        ReviewFixture::new(r#"{"summary":"EXAMPLE\u005fTOKEN=fixture","summary":"safe"}"#).await;
    h.exited().await;
    assert_eq!(h.step().state, "held");
    assert_eq!(h.step().body["reason"], "secret_detected");
    assert_eq!(h.step().body["secret"]["pattern"], "secret_assignment");
    h.close().await;
}

/// An uncertain death keeps the secret receipt while retaining the death-unconfirmed reason.
#[tokio::test]
async fn b7_death_unconfirmed_receipt() {
    let h = ReviewFixture::new(&success()).await;
    let receipt = json!({"pattern":"secret_assignment","offset_sha256":"0".repeat(64)});
    let cursor = h.f.cursor(&h.id);
    h.f.store()
        .pipeline_hold_death_unconfirmed(&h.f.owner, &cursor, Some(&receipt))
        .unwrap();
    assert_eq!(h.step().state, "held");
    assert_eq!(h.step().body["reason"], "death_unconfirmed");
    assert_eq!(h.step().body["secret"], receipt);
    h.close().await;
}

/// A prior nonce cannot be reused by changing its case in stored history.
#[tokio::test]
async fn b7_nonce_reuse_casefold() {
    let h = ReviewFixture::new(&success()).await;
    h.exited().await;
    let mut prior = h.step();
    let nonce = "abcdef1234567890abcdef1234567890".to_owned();
    prior.body["review_nonce"] = json!(nonce.to_ascii_uppercase());
    {
        let mut store = h.f.store();
        let tx = store.connection.transaction().unwrap();
        write_step(&tx, &h.f.owner, &prior).unwrap();
        tx.commit().unwrap();
    }
    assert!(h.adopt().await);
    h.f.adopt_staged(&h.id);
    let cursor = h.f.cursor(&h.id);
    let (_, task, _) =
        h.f.store()
            .pipeline_review_context(&h.f.owner, &cursor)
            .unwrap();
    let mut next = planned(&h.id, 3, 1, "review");
    next.step_base = prior.step_base;
    next.body["review_nonce"] = json!(nonce);
    next.input_sha256 =
        crate::execution::verification::hash(&review_prompt(&nonce, &task, b"").unwrap());
    h.f.seed_next(&h.id, &next);
    assert!(matches!(h.launch().await, LaunchResult::Held));
    assert_eq!(h.step().body["reason"], "review_nonce_reused");
    h.close().await;
}
fn success() -> String {
    serde_json::to_string(&json!({"type":"result","subtype":"success","is_error":false,"modelUsage":{"PINNED_MODEL":{}},
        "structured_output":{"schema":"rrx.review.v1","verdict":"approve","summary":"safe","findings":[]}})).unwrap()
}

/// Runtime launches the fixed Reviewer in a commit snapshot and stores only a checked verdict.
#[tokio::test]
async fn b7_runtime_review_flow() {
    let h = ReviewFixture::new(&success()).await;
    let view = h.f.store().pipeline_goal_view(h.f.goal).unwrap();
    let task = Path::new(view.runs[0].worktree.as_ref().unwrap());
    std::fs::write(task.join("tracked"), b"untrusted Task drift").unwrap();
    h.exited().await;
    assert_eq!(h.step().state, "exited");
    assert_eq!(std::fs::read(h.path.join("tracked")).unwrap(), b"safe\n");
    assert!(h.step().cookie.is_some() && h.step().birth.is_some());
    assert!(h.adopt().await);
    let step = h.step();
    assert!(step.state == "exited" && step.body["round_result_validated"] == true);
    assert_eq!(step.body["adoption"]["verdict"]["verdict"], "approve");
    assert_eq!(
        std::fs::read_to_string(h.fake.directory.join("cwd")).unwrap(),
        h.path.to_str().unwrap()
    );
    let environment = std::fs::read_to_string(h.fake.directory.join("environment")).unwrap();
    for entry in [
        "GIT_CONFIG_COUNT=1",
        "GIT_CONFIG_KEY_0=safe.bareRepository",
        "GIT_CONFIG_VALUE_0=explicit",
        "GIT_CONFIG_NOSYSTEM=1",
    ] {
        assert!(environment.lines().any(|line| line == entry));
    }
    assert_eq!(
        std::fs::read(h.attempt.join("prompt.md")).unwrap(),
        std::fs::read(h.fake.directory.join("stdin")).unwrap()
    );
    assert_eq!(
        std::fs::read(h.attempt.join("mcp.json")).unwrap(),
        b"{\"mcpServers\":{}}"
    );
    for name in ["prompt.md", "mcp.json", "result.json", "stderr.log"] {
        assert_eq!(
            std::fs::metadata(h.attempt.join(name))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o400
        );
    }
    assert_eq!(
        std::fs::metadata(&h.attempt).unwrap().permissions().mode() & 0o777,
        0o700
    );
    h.close().await;
}

/// The Reviewer's stdin carries the base..head diff of the run's adopted commit.
#[tokio::test]
async fn b7_runtime_review_diff() {
    let h = ReviewFixture::after_commit(&success(), "pub fn reviewed() {}\n").await;
    let view = h.f.store().pipeline_goal_view(h.f.goal).unwrap();
    assert_ne!(view.runs[0].head_commit, view.runs[0].base_commit);
    h.exited().await;
    let stdin = std::fs::read_to_string(h.fake.directory.join("stdin")).unwrap();
    assert!(
        stdin.contains("+++ b/src/output.rs\n@@ -0,0 +1 @@\n+pub fn reviewed() {}\n"),
        "{stdin}"
    );
    h.close().await;
}

/// Rebuilding discards a partial snapshot and seals all directories and file modes.
#[tokio::test]
async fn b7_snapshot_rebuild_readonly() {
    let h = ReviewFixture::new(&success()).await;
    let input = h.prepare().await.unwrap();
    assert_eq!(input.work, h.path);
    assert_eq!(input.schema, review::REVIEW_SCHEMA);
    for path in [&h.path, &h.path.join("tracked")] {
        assert_eq!(
            std::fs::metadata(path).unwrap().permissions().mode() & 0o222,
            0
        );
    }
    assert!(std::fs::write(h.path.join("tracked"), b"write attempt").is_err());
    assert!(std::fs::write(h.path.join("new"), b"write attempt").is_err());
    std::fs::set_permissions(&h.path, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(h.path.join("partial"), b"discard").unwrap();
    assert!(h.prepare().await.is_some());
    assert!(!h.path.join("partial").exists());
    h.close().await;
}

/// Post-death content drift holds even when a valid approve envelope was returned.
#[tokio::test]
async fn b7_runtime_snapshot_digest() {
    let h = ReviewFixture::new(&success()).await;
    h.exited().await;
    let file = h.path.join("tracked");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::write(&file, b"changed\n").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).unwrap();
    assert!(!h.adopt().await);
    assert_eq!(h.step().state, "held");
    assert_eq!(h.step().body["reason"], "review_snapshot_changed");
    h.close().await;
}

/// An unexpected envelope cannot become an adopted or rejected schema result.
#[tokio::test]
async fn b7_runtime_envelope() {
    for key in [
        "type",
        "subtype",
        "is_error",
        "modelUsage",
        "structured_output",
    ] {
        let mut v: Value = serde_json::from_str(&success()).unwrap();
        v.as_object_mut().unwrap().remove(key);
        let h = ReviewFixture::new(&serde_json::to_string(&v).unwrap()).await;
        h.exited().await;
        assert!(!h.adopt().await);
        assert_eq!(h.step().state, "held");
        assert_eq!(
            h.step().body["reason"],
            if key == "modelUsage" {
                "review_model_mismatch"
            } else {
                "review_envelope_mismatch"
            }
        );
        h.close().await;
    }
}

/// Schema violations are recorded as rejected without storing untrusted verdict excerpts.
#[tokio::test]
async fn b7_runtime_schema() {
    let mut v: Value = serde_json::from_str(&success()).unwrap();
    v["structured_output"]["extra"] = json!(true);
    let h = ReviewFixture::new(&serde_json::to_string(&v).unwrap()).await;
    h.exited().await;
    assert!(!h.adopt().await);
    assert_eq!(h.step().state, "rejected");
    assert_eq!(h.step().body["reason"], "verdict_schema");
    assert!(h.step().body["adoption"].is_null());
    h.close().await;
}

/// Retained output presence, digest and file mode are prerequisites to decoding.
#[tokio::test]
async fn b7_runtime_output_evidence() {
    for name in ["result.json", "stderr.log"] {
        let h = ReviewFixture::new(&success()).await;
        h.exited().await;
        std::fs::remove_file(h.attempt.join(name)).unwrap();
        assert!(!h.adopt().await);
        assert_eq!(h.step().state, "held");
        assert_eq!(h.step().body["reason"], "review_output_changed");
        h.close().await;
    }
}

/// A streamed secret holds before adoption and retains only its pattern and position digest.
#[tokio::test]
async fn b7_runtime_stream_secrets() {
    for stderr in [false, true] {
        let output = if stderr {
            success()
        } else {
            "EXAMPLE_SECRET=fixture".into()
        };
        let h = ReviewFixture::new(&output).await;
        if stderr {
            h.fake.control(
                "exec-stderr",
                &format!("EXAMPLE_TOKEN=fixture\n{}", "x".repeat(1024 * 1024)),
            );
        }
        h.exited().await;
        assert_eq!(h.step().state, "held");
        assert_eq!(h.step().body["reason"], "secret_detected");
        let body = h.step().body;
        assert_eq!(body["secret"]["pattern"], "secret_assignment");
        assert_eq!(body["secret"]["offset_sha256"].as_str().unwrap().len(), 64);
        assert!(!serde_json::to_string(&body).unwrap().contains("fixture"));
        h.close().await;
    }
}

/// The immutable severity policy reaches the decoder through the Runtime adoption caller.
#[tokio::test]
async fn b7_runtime_blocking() {
    let mut v: Value = serde_json::from_str(&success()).unwrap();
    v["structured_output"]["findings"] = json!([{"id":"F1","severity":"high","blocking":false,"file":"tracked","line":1,"title":"title","rationale":"reason"}]);
    let h = ReviewFixture::new(&serde_json::to_string(&v).unwrap()).await;
    h.exited().await;
    assert!(h.adopt().await);
    assert_eq!(
        h.step().body["adoption"]["verdict"]["verdict"],
        "request_changes"
    );
    h.close().await;
}

/// A known provider limit uses the existing limited transition only after snapshot validation.
#[tokio::test]
async fn b7_runtime_limit() {
    let h=ReviewFixture::new(r#"{"type":"result","is_error":true,"api_error":{"status":429},"result":"Rate limit exceeded"}"#).await;
    h.fake.control("exec-status", "1\n");
    h.exited().await;
    assert!(!h.adopt().await);
    assert_eq!(h.step().state, "limited");
    assert_eq!(h.step().body["provider_outcome"], "limit");
    h.close().await;
}

/// Pausing the Goal before adoption leaves an exited review unadopted.
#[tokio::test]
async fn b7_runtime_rule_g() {
    let h = ReviewFixture::new(&success()).await;
    h.exited().await;
    let goal = h.f.store().goal(h.f.goal).unwrap().unwrap();
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: h.f.owner.instance.clone(),
        epoch: h.f.owner.epoch,
        action: ControlAction::SetGoalLifecycle {
            project: h.f.project,
            goal: h.f.goal,
            expected_goal: goal.version,
            target: crate::runtime::control::GoalControl::Pause,
            reason: "pause".into(),
        },
    };
    let (socket, _peer) = tokio::net::UnixStream::pair().unwrap();
    h.runtime.wait_pipeline_test_boundary().await;
    h.runtime.handle_control(&socket, request).await.unwrap();
    assert!(
        h.runtime
            .adopt_pipeline_round_review(h.f.cursor(&h.id))
            .await
            .is_err()
    );
    assert_eq!(h.step().state, "exited");
    h.close().await;
}

/// A pre-launch tree digest mismatch prevents the common launcher from spawning.
#[tokio::test]
async fn b7_launcher_snapshot_guard() {
    let h = ReviewFixture::new(&success()).await;
    let input = h.prepare().await.unwrap();
    let file = h.path.join("tracked");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::write(&file, b"changed").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).unwrap();
    h.runtime.wait_pipeline_test_boundary().await;
    let result = h
        .runtime
        .launch_pipeline_step(h.f.cursor(&h.id), input)
        .await
        .unwrap();
    assert!(matches!(result, LaunchResult::Held));
    assert_eq!(h.step().body["reason"], "review_snapshot_changed");
    assert!(!h.fake.directory.join("exec-entered").exists());
    h.close().await;
}

/// A direct Store adoption cannot authorize explicit-blocking approve data.
#[tokio::test]
async fn b7_store_adoption_guard() {
    let h = ReviewFixture::new(&success()).await;
    h.exited().await;
    let step = h.step();
    let mut evidence = review::VerdictDecoder::decode(
        success()
            .replace("PINNED_MODEL", &h.fake.cli.model)
            .as_bytes(),
        &h.fake.cli.model,
        ReviewSeverity::High,
    )
    .unwrap();
    evidence["snapshot_tree"] = step.body["snapshot_tree"].clone();
    evidence["verdict"]["findings"] = json!([{"id":"F1","severity":"low","blocking":true,"file":"tracked","line":1,"title":"title","rationale":"reason"}]);
    let cursor = h.f.cursor(&h.id);
    assert!(
        h.f.store()
            .pipeline_stage_round_result(&h.f.owner, &cursor, &evidence)
            .is_err()
    );
    assert_eq!(h.step().state, "exited");
    h.close().await;
}

/// Store writes scan nested strings even when a caller supplies additional diagnostics.
#[tokio::test]
async fn b7_store_body_scan() {
    let h = ReviewFixture::new(&success()).await;
    h.exited().await;
    let mut evidence = review::VerdictDecoder::decode(
        success()
            .replace("PINNED_MODEL", &h.fake.cli.model)
            .as_bytes(),
        &h.fake.cli.model,
        ReviewSeverity::High,
    )
    .unwrap();
    evidence["snapshot_tree"] = h.step().body["snapshot_tree"].clone();
    evidence["extra"] = json!({"nested":["EXAMPLE_TOKEN=fixture"]});
    let cursor = h.f.cursor(&h.id);
    let error =
        h.f.store()
            .pipeline_stage_round_result(&h.f.owner, &cursor, &evidence)
            .unwrap_err();
    assert!(
        error
            .downcast_ref::<crate::execution::pipeline::SecretHit>()
            .is_some()
    );
    assert_eq!(h.step().state, "exited");
    h.close().await;
}

/// Escaped secrets in envelope diagnostics hold before adoption without storing diagnostics.
#[tokio::test]
async fn b7_runtime_decoded_secret() {
    let mut v: Value = serde_json::from_str(&success()).unwrap();
    v["diagnostic"] = json!("EXAMPLE_TOKEN=fixture");
    let bytes = serde_json::to_string(&v)
        .unwrap()
        .replace("EXAMPLE_TOKEN", r"EXAMPLE\u005fTOKEN");
    let h = ReviewFixture::new(&bytes).await;
    h.exited().await;
    assert_eq!(h.step().state, "held");
    assert_eq!(h.step().body["reason"], "secret_detected");
    assert!(h.step().body["adoption"].is_null());
    h.close().await;
}

/// A file made writable after launch holds even if its Git content is unchanged.
#[tokio::test]
async fn b7_snapshot_writable_file() {
    let h = ReviewFixture::new(&success()).await;
    h.exited().await;
    std::fs::set_permissions(
        h.path.join("tracked"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    assert!(!h.adopt().await);
    assert_eq!(h.step().state, "held");
    assert_eq!(h.step().body["reason"], "review_snapshot_changed");
    h.close().await;
}

/// A snapshot directory made writable after launch holds independently of its files.
#[tokio::test]
async fn b7_snapshot_writable_directory() {
    let h = ReviewFixture::new(&success()).await;
    h.exited().await;
    std::fs::set_permissions(&h.path, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(!h.adopt().await);
    assert_eq!(h.step().state, "held");
    assert_eq!(h.step().body["reason"], "review_snapshot_changed");
    h.close().await;
}

/// Review planning refuses secret-bearing body strings before a row can be inserted.
#[tokio::test]
async fn b7_store_plan_scan() {
    let (f, runtime, id) = super::b5::prepare_fixture_from(fixture().await, true).await;
    runtime.wait_pipeline_test_boundary().await;
    assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let mut next = planned(&id, 2, 1, "review");
    next.step_base = view.runs[0].head_commit.clone();
    next.body["diagnostic"] = json!("EXAMPLE_SECRET=fixture");
    let cursor = f.cursor(&id);
    let error = f
        .store()
        .pipeline_seed_step(&f.owner, &cursor, &next)
        .unwrap_err();
    assert!(
        error
            .downcast_ref::<crate::execution::pipeline::SecretHit>()
            .is_some()
    );
    assert_eq!(f.store().pipeline_goal_view(f.goal).unwrap().steps.len(), 1);
    runtime.shutdown().await.unwrap();
}

/// Planner input cannot claim Store preflight, adoption or secret evidence.
#[tokio::test]
async fn b7_store_plan_reserved_fields() {
    let (f, runtime, id) = super::b5::prepare_fixture_from(fixture().await, true).await;
    runtime.wait_pipeline_test_boundary().await;
    assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    let before = f.store().pipeline_goal_view(f.goal).unwrap();
    let mut next = planned(&id, 2, 1, "review");
    next.step_base = before.runs[0].head_commit.clone();
    let cursor = f.cursor(&id);
    for key in [
        "review_nonce_verified",
        "snapshot_tree",
        "adoption",
        "secret",
    ] {
        for value in [Value::Null, json!(true), json!("claimed")] {
            next.body[key] = value;
            let error = f
                .store()
                .pipeline_plan_next(&f.owner, &cursor, &next, true)
                .unwrap_err();
            assert_eq!(error.to_string(), "review_store_fields_reserved", "{key}");
            let after = f.store().pipeline_goal_view(f.goal).unwrap();
            assert_eq!(after.steps, before.steps);
            assert_eq!(after.runs, before.runs);
            next.body.as_object_mut().unwrap().remove(key);
        }
    }
    f.store()
        .pipeline_seed_step(&f.owner, &cursor, &next)
        .unwrap();
    assert_eq!(f.store().pipeline_goal_view(f.goal).unwrap().steps.len(), 2);
    runtime.shutdown().await.unwrap();
}
