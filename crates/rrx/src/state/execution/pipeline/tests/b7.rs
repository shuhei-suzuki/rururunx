//! Host cases use real authority and the pinned native fixtures, never SQL success seeds.
use super::*;
use crate::execution::pipeline::{
    LaunchResult, PipelineGit, PipelineRole, ReviewSeverity, StepInput, review, review_prompt,
    review_snapshot, step_arguments, test_support::FakeCli,
};
use std::{os::unix::fs::PermissionsExt, path::PathBuf, sync::Arc};

struct ReviewFixture {
    f: Fixture,
    runtime: Arc<Runtime>,
    id: String,
    fake: FakeCli,
    path: PathBuf,
    attempt: PathBuf,
}
impl ReviewFixture {
    async fn new(output: &str) -> Self {
        let (f, runtime, id) = super::b5::prepare_fixture_from(fixture().await, true).await;
        runtime.wait_pipeline_test_boundary().await;
        assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
        let view = f.store().pipeline_goal_view(f.goal).unwrap();
        let task = f.store().task(f.task).unwrap().unwrap();
        let data = json!({"title":task.title,"acceptance_criteria":task.acceptance_criteria,"scope":["src/**"]});
        let nonce = Uuid::new_v4().simple().to_string();
        let prompt = review_prompt(&nonce, &data, b"").unwrap();
        let mut next = planned(&id, 2, 1, "review");
        next.step_base = view.runs[0].head_commit.clone();
        next.input_sha256 = crate::execution::verification::hash(&prompt);
        next.body["review_nonce"] = json!(nonce);
        let cursor = f.cursor(&id);
        f.store()
            .pipeline_plan_next(&f.owner, &cursor, &next, true)
            .unwrap();
        let config = f.config.pipeline.as_ref().unwrap();
        let path = review_snapshot::snapshot_path(&config.work_root, &id, 2, 1).unwrap();
        let attempt = config.work_root.join("runs").join(&id).join("2-1");
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
    async fn prepare(&self) -> Option<StepInput> {
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
            .adopt_pipeline_review(self.f.cursor(&self.id))
            .await
            .unwrap();
        if result || self.step() != before {
            return result;
        }
        self.runtime.wait_pipeline_test_boundary().await;
        self.runtime
            .adopt_pipeline_review(self.f.cursor(&self.id))
            .await
            .unwrap()
    }
    async fn close(self) {
        self.runtime.shutdown().await.unwrap();
        review_snapshot::remove(&self.path).unwrap();
    }
}
fn success() -> String {
    serde_json::to_string(&json!({"type":"result","subtype":"success","is_error":false,"model":"PINNED_MODEL",
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
    assert_eq!(step.state, "adopted");
    assert_eq!(step.body["adoption"]["verdict"]["verdict"], "approve");
    assert_eq!(
        std::fs::read_to_string(h.fake.directory.join("cwd")).unwrap(),
        h.path.to_str().unwrap()
    );
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
    for key in ["type", "subtype", "is_error", "model", "structured_output"] {
        let mut v: Value = serde_json::from_str(&success()).unwrap();
        v.as_object_mut().unwrap().remove(key);
        let h = ReviewFixture::new(&serde_json::to_string(&v).unwrap()).await;
        h.exited().await;
        assert!(!h.adopt().await);
        assert_eq!(h.step().state, "held");
        assert_eq!(
            h.step().body["reason"],
            if key == "model" {
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
            .adopt_pipeline_review(h.f.cursor(&h.id))
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
    let project = h.f.store().project(h.f.project).unwrap().unwrap();
    let git = PipelineGit::new(&project.root).unwrap();
    let step = h.step();
    let tree = git
        .text(&["rev-parse", &format!("{}^{{tree}}", step.step_base)])
        .await
        .unwrap();
    let cursor = h.f.cursor(&h.id);
    h.f.store()
        .pipeline_review_preflight(&h.f.owner, &cursor, &tree)
        .unwrap();
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
            .pipeline_adopt(&h.f.owner, &cursor, &evidence)
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
            .pipeline_adopt(&h.f.owner, &cursor, &evidence)
            .unwrap_err();
    assert!(
        error
            .downcast_ref::<crate::execution::pipeline::SecretHit>()
            .is_some()
    );
    assert_eq!(h.step().state, "exited");
    h.close().await;
}

/// Escaped secrets in decoded envelope diagnostics hold without persisting those diagnostics.
#[tokio::test]
async fn b7_runtime_decoded_secret() {
    let mut v: Value = serde_json::from_str(&success()).unwrap();
    v["diagnostic"] = json!("EXAMPLE_TOKEN=fixture");
    let bytes = serde_json::to_string(&v)
        .unwrap()
        .replace("EXAMPLE_TOKEN", r"EXAMPLE\u005fTOKEN");
    let h = ReviewFixture::new(&bytes).await;
    h.exited().await;
    assert_eq!(h.step().state, "exited");
    assert!(!h.adopt().await);
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
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap_err();
    assert!(
        error
            .downcast_ref::<crate::execution::pipeline::SecretHit>()
            .is_some()
    );
    assert_eq!(f.store().pipeline_goal_view(f.goal).unwrap().steps.len(), 1);
    runtime.shutdown().await.unwrap();
}
