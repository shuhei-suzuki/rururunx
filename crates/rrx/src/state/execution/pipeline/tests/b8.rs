//! Runtime cases use owned fixtures, admitted tests and native fake CLIs without provider traffic.
use super::*;
use crate::execution::pipeline::{
    LaunchResult, PipelineRole, RoundDecision, VerdictDecoder, confirm, review, review_snapshot,
    step_arguments, test_support::FakeCli,
};
use std::{os::unix::fs::PermissionsExt, path::PathBuf, sync::Arc};

struct Harness {
    f: Fixture,
    runtime: Arc<Runtime>,
    id: String,
    codex: FakeCli,
    claude: FakeCli,
    source: String,
}
impl Harness {
    async fn new(max: u32, fail: bool) -> Self {
        Self::with_pin(max, fail, None).await
    }
    async fn with_pin(max: u32, fail: bool, pin: Option<&str>) -> Self {
        let qualification = Arc::new(std::sync::Mutex::new(Value::Null));
        let inputs = FixtureInputs {
            max_fix_rounds: Some(max),
            clis: None,
            codex_pin: pin.map(str::to_owned),
            path_prefix: vec![],
            task: TaskDefinition {
                key: "task".into(),
                title: "task".into(),
                acceptance_criteria: vec!["pass".into()],
                executor: "codex".into(),
                reviewers: vec!["claude".into()],
                workflow: WorkflowClass::Standard,
                risk: RiskClass::R2,
            },
            qualification: qualification.clone(),
        };
        let mut command = crate::execution::pipeline::tests_runner::tests::proposal(
            if fail {
                "/usr/bin/false"
            } else {
                "/usr/bin/true"
            }
            .into(),
        )
        .commands
        .remove(0);
        command.timeout_seconds = 10;
        let f = fixture_build(
            true,
            true,
            None,
            None,
            false,
            Some(vec![command]),
            Some(inputs),
        )
        .await
        .unwrap_or_else(|error| {
            panic!(
                "fixture qualification: {error}; {:?}",
                qualification.lock().unwrap()
            )
        });
        let (f, runtime, id) = super::b5::prepare_fixture_from(f, true).await;
        runtime.wait_pipeline_test_boundary().await;
        assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
        let config = f.config.pipeline.as_ref().unwrap();
        let codex = FakeCli {
            directory: config.clis.codex.executable.parent().unwrap().into(),
            cli: config.clis.codex.clone(),
        };
        let claude = FakeCli {
            directory: config.clis.claude.executable.parent().unwrap().into(),
            cli: config.clis.claude.clone(),
        };
        let mut h = Self {
            f,
            runtime,
            id,
            codex,
            claude,
            source: String::new(),
        };
        h.executor(false, 0).await;
        h
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
    fn path(&self) -> PathBuf {
        let step = self.step();
        review_snapshot::snapshot_path(
            &self.f.config.pipeline.as_ref().unwrap().work_root,
            &self.id,
            step.seq,
            step.attempt,
        )
        .unwrap()
    }
    fn attempt(&self) -> PathBuf {
        let step = self.step();
        self.f
            .config
            .pipeline
            .as_ref()
            .unwrap()
            .work_root
            .join("runs")
            .join(&self.id)
            .join(format!("{}-{}", step.seq, step.attempt))
    }
    fn expected(&self, fake: &FakeCli, role: PipelineRole, work: &Path, schema: &str) {
        let args = step_arguments(role, &fake.cli, work, &self.attempt(), schema).unwrap();
        fake.expect_arguments(&args.iter().map(String::as_str).collect::<Vec<_>>());
    }
    async fn decide(&self) -> RoundDecision {
        for _ in 0..2 {
            self.runtime.wait_pipeline_test_boundary().await;
            if let Some(d) = self
                .runtime
                .decide_pipeline_round(self.f.cursor(&self.id))
                .await
                .unwrap()
            {
                return d;
            }
        }
        panic!("decision remained busy")
    }
    async fn executor(&mut self, fix: bool, revision: u32) {
        self.source = format!("pub fn broken_{revision}() {{}}\n");
        let prompt = b"Implement the accepted Task.\n";
        if !fix {
            let view = self.f.store().pipeline_goal_view(self.f.goal).unwrap();
            let mut next = planned(&self.id, 2, 1, "implement");
            next.step_base = view.runs[0].head_commit.clone();
            next.input_sha256 = crate::execution::verification::hash(prompt);
            let cursor = self.f.cursor(&self.id);
            self.f
                .store()
                .pipeline_plan_next(&self.f.owner, &cursor, &next, true)
                .unwrap();
        }
        self.codex.control("exec-source", &self.source);
        self.codex
            .control("exec-last", "Implementation complete.\n");
        self.codex.control(
            "exec-output",
            "{\"type\":\"turn.started\"}\n{\"type\":\"turn.completed\"}\n",
        );
        let work = PathBuf::from(
            self.f.store().pipeline_goal_view(self.f.goal).unwrap().runs[0]
                .worktree
                .as_ref()
                .unwrap(),
        );
        self.expected(
            &self.codex,
            if fix {
                PipelineRole::Fix
            } else {
                PipelineRole::Implement
            },
            &work,
            "",
        );
        self.runtime.wait_pipeline_test_boundary().await;
        let result = if fix {
            self.runtime
                .launch_pipeline_fix(self.f.cursor(&self.id))
                .await
        } else {
            self.runtime
                .launch_pipeline_executor(self.f.cursor(&self.id), prompt.to_vec())
                .await
        }
        .unwrap();
        let LaunchResult::Started(custody) = result else {
            panic!("executor did not launch: {:?}", self.step());
        };
        custody.wait().await.unwrap();
        self.runtime.wait_pipeline_test_boundary().await;
        assert!(
            self.runtime
                .adopt_pipeline_executor(self.f.cursor(&self.id))
                .await
                .unwrap(),
            "{:?}",
            self.step()
        );
        assert!(self.step().result_commit.is_some());
    }
    async fn test(&self) -> RoundDecision {
        self.runtime.wait_pipeline_test_boundary().await;
        assert!(
            self.runtime
                .plan_pipeline_round_test(self.f.cursor(&self.id))
                .await
                .unwrap()
        );
        self.runtime.wait_pipeline_test_boundary().await;
        let LaunchResult::Started(custody) = self
            .runtime
            .launch_pipeline_test(self.f.cursor(&self.id))
            .await
            .unwrap()
        else {
            panic!("test did not launch");
        };
        custody.wait().await.unwrap();
        self.runtime.wait_pipeline_test_boundary().await;
        assert!(
            self.runtime
                .adopt_pipeline_round_test(self.f.cursor(&self.id))
                .await
                .unwrap(),
            "{:?}",
            self.step()
        );
        self.decide().await
    }
    async fn review_exit(&self, findings: Value, verdict: &str) {
        assert_eq!(self.step().kind, "review");
        let envelope = json!({"type":"result","subtype":"success","is_error":false,"modelUsage":{&self.claude.cli.model:{}},"structured_output":{"schema":"rrx.review.v1","verdict":verdict,"summary":"reviewed","findings":findings}});
        self.claude
            .control("exec-output", &serde_json::to_string(&envelope).unwrap());
        self.expected(
            &self.claude,
            PipelineRole::Review,
            &self.path(),
            review::REVIEW_SCHEMA,
        );
        self.runtime.wait_pipeline_test_boundary().await;
        let LaunchResult::Started(custody) = self
            .runtime
            .launch_pipeline_review(self.f.cursor(&self.id))
            .await
            .unwrap()
        else {
            panic!("review did not launch");
        };
        custody.wait().await.unwrap();
    }
    async fn reviewed(&self, findings: Value, verdict: &str) {
        self.review_exit(findings, verdict).await;
        self.runtime.wait_pipeline_test_boundary().await;
        assert!(
            self.runtime
                .adopt_pipeline_round_review(self.f.cursor(&self.id))
                .await
                .unwrap(),
            "{:?}",
            self.step()
        );
    }
    fn findings(&self) -> Value {
        json!([{"id":"F1","severity":"high","blocking":false,"file":"src/output.rs","line":1,"title":"fix this defect","rationale":"source demonstrates defect"}])
    }
    fn labels(&self, label: &str) -> Value {
        json!({"schema":"rrx.confirm.v1","labels":[{"id":"F1","label":label,"evidence":{"file":"src/output.rs","line":1,"quote":self.source.trim_end()},"reason":"checked source"}]})
    }
    async fn confirmation_exit(&self, output: &str) {
        let source = self.codex.directory.join("exec-source");
        if source.exists() {
            std::fs::remove_file(source).unwrap();
        }
        self.codex.control("exec-last", output);
        self.codex.control(
            "exec-output",
            "{\"type\":\"turn.started\"}\n{\"type\":\"turn.completed\"}\n",
        );
        self.expected(
            &self.codex,
            PipelineRole::Confirm,
            &self.path(),
            confirm::CONFIRM_SCHEMA,
        );
        self.runtime.wait_pipeline_test_boundary().await;
        let LaunchResult::Started(custody) = self
            .runtime
            .launch_pipeline_confirm(self.f.cursor(&self.id))
            .await
            .unwrap()
        else {
            panic!("confirm did not launch: {:?}", self.step());
        };
        custody.wait().await.unwrap();
    }
    async fn confirmed(&self, labels: Value) {
        self.confirmation_exit(&serde_json::to_string(&labels).unwrap())
            .await;
        assert_eq!(self.step().state, "exited", "{:?}", self.step());
        assert!(self.step().cookie.is_some() && self.step().birth.is_some());
        self.runtime.wait_pipeline_test_boundary().await;
        assert!(
            self.runtime
                .adopt_pipeline_confirm(self.f.cursor(&self.id))
                .await
                .unwrap(),
            "{:?}",
            self.step()
        );
    }
    async fn to_confirm(&self) {
        assert_eq!(self.test().await, RoundDecision::Review);
        self.reviewed(self.findings(), "request_changes").await;
        assert_eq!(self.decide().await, RoundDecision::Confirm);
    }
    async fn close(self) {
        self.runtime.shutdown().await.unwrap();
        let view = self.f.store().pipeline_goal_view(self.f.goal).unwrap();
        for s in view
            .steps
            .iter()
            .filter(|s| matches!(s.kind.as_str(), "review" | "confirm"))
        {
            let path = review_snapshot::snapshot_path(
                &self.f.config.pipeline.as_ref().unwrap().work_root,
                &self.id,
                s.seq,
                s.attempt,
            )
            .unwrap();
            review_snapshot::remove(&path).unwrap();
        }
    }
}

/// F-w4: Confirmer last-output failures override every wire outcome while absence preserves failures.
#[tokio::test]
async fn f_w4_confirmer_last_output() {
    for output in [
        "{\"type\":\"turn.started\"}\n{\"type\":\"turn.completed\"}\n",
        "{\"type\":\"turn.failed\",\"error\":{\"code\":\"rate_limit_exceeded\",\"message\":\"Rate limit reached\"}}\n",
        "{\"type\":\"unknown.event\"}\n",
    ] {
        for last in [
            "missing",
            "valid",
            "symlink",
            "hardlink",
            "oversized",
            "directory",
        ] {
            let h = Harness::with_pin(1, false, Some("codex-cli 0.164.0")).await;
            h.to_confirm().await;
            std::fs::remove_file(h.codex.directory.join("exec-source")).unwrap();
            std::fs::remove_file(h.codex.directory.join("exec-last")).unwrap();
            h.codex.control("exec-output", output);
            h.codex.control("exec-pause", "");
            h.expected(
                &h.codex,
                PipelineRole::Confirm,
                &h.path(),
                confirm::CONFIRM_SCHEMA,
            );
            h.runtime.wait_pipeline_test_boundary().await;
            let LaunchResult::Started(custody) = h
                .runtime
                .launch_pipeline_confirm(h.f.cursor(&h.id))
                .await
                .unwrap()
            else {
                panic!("confirm did not launch");
            };
            let root = h.attempt();
            let path = root.join("last.json");
            match last {
                "missing" => {}
                "valid" => {
                    std::fs::write(&path, serde_json::to_vec(&h.labels("confirmed")).unwrap())
                        .unwrap()
                }
                "symlink" | "hardlink" => {
                    let target = root.join("target");
                    std::fs::write(&target, b"{}").unwrap();
                    if last == "symlink" {
                        std::os::unix::fs::symlink(&target, &path).unwrap();
                    } else {
                        std::fs::hard_link(&target, &path).unwrap();
                    }
                }
                "oversized" => std::fs::write(&path, vec![b'x'; 256 * 1024 + 1]).unwrap(),
                _ => std::fs::create_dir(&path).unwrap(),
            }
            h.codex.control("exec-release", "");
            custody.wait().await.unwrap();
            let step = h.step();
            let success = output.contains("turn.completed");
            let limit = output.contains("rate_limit_exceeded");
            let invalid = !matches!(last, "missing" | "valid") || (last == "missing" && success);
            assert_eq!(
                step.body["provider_outcome"],
                if invalid {
                    "other"
                } else if success {
                    "success"
                } else if limit {
                    "limit"
                } else {
                    "other"
                },
                "{last}"
            );
            assert_eq!(
                step.body["exit"]["issue"],
                if invalid {
                    json!("last_output_invalid")
                } else {
                    Value::Null
                }
            );
            assert_eq!(step.body["exit"]["last"].is_object(), last == "valid");
            if last == "missing" && limit {
                h.runtime.wait_pipeline_test_boundary().await;
                assert!(
                    !h.runtime
                        .adopt_pipeline_confirm(h.f.cursor(&h.id))
                        .await
                        .unwrap()
                );
                assert_eq!(h.step().state, "limited");
            }
            h.close().await;
        }
    }
}

/// Only confirmed IDs reach a fix, whose rrx commit is re-tested and reviewed before planned release.
#[tokio::test]
async fn b8_runtime_fix_flow() {
    let mut h = Harness::new(1, false).await;
    assert_eq!(h.test().await, RoundDecision::Review);
    let mut findings = h.findings();
    let mut rejected = findings[0].clone();
    rejected["id"] = json!("F2");
    rejected["title"] = json!("rejected data");
    let mut non = findings[0].clone();
    non["id"] = json!("F3");
    non["severity"] = json!("low");
    non["title"] = json!("informational data");
    let mut invalid = findings[0].clone();
    invalid["id"] = json!("F4");
    invalid["file"] = json!("src/missing.rs");
    findings
        .as_array_mut()
        .unwrap()
        .extend([rejected, non, invalid]);
    h.reviewed(findings, "request_changes").await;
    let review_path = h.path();
    assert_eq!(h.decide().await, RoundDecision::Confirm);
    let view = h.f.store().pipeline_goal_view(h.f.goal).unwrap();
    let review = view.steps.iter().find(|s| s.kind == "review").unwrap();
    assert_eq!(review.body["filtering"]["invalid"][0]["id"], "F4");
    assert_eq!(review.body["filtering"]["non_blocking"], json!(["F3"]));
    assert_ne!(review_path, h.path());
    let mut labels = h.labels("confirmed");
    let mut rejected = labels["labels"][0].clone();
    rejected["id"] = json!("F2");
    rejected["label"] = json!("rejected");
    labels["labels"].as_array_mut().unwrap().push(rejected);
    h.confirmed(labels).await;
    assert_eq!(
        std::fs::read_to_string(h.path().join("src/output.rs")).unwrap(),
        h.source
    );
    assert_eq!(h.decide().await, RoundDecision::Fix);
    let fix = h.step();
    assert_eq!(fix.round, 1);
    assert_eq!(fix.body["confirmed_ids"], json!(["F1"]));
    assert_eq!(
        fix.body["fix_input"]["findings"].as_array().unwrap().len(),
        1
    );
    let input: crate::execution::pipeline::rounds::FixInput =
        serde_json::from_value(fix.body["fix_input"].clone()).unwrap();
    assert_eq!(
        fix.body["fix_input_sha256"],
        crate::execution::verification::hash(&serde_json::to_vec(&input).unwrap())
    );
    let prompt = crate::execution::pipeline::rounds::fix_prompt(
        fix.body["fix_nonce"].as_str().unwrap(),
        &input,
    )
    .unwrap();
    assert_eq!(
        crate::execution::verification::hash(&prompt),
        fix.input_sha256
    );
    assert!(!String::from_utf8(prompt).unwrap().contains("rejected data"));
    h.executor(true, 1).await;
    assert_eq!(h.test().await, RoundDecision::Review);
    h.reviewed(json!([]), "approve").await;
    assert_eq!(h.decide().await, RoundDecision::Release);
    assert_eq!(h.step().kind, "release");
    assert_eq!(h.step().state, "planned");
    h.close().await;
}

/// Limits zero through two permit exactly N fixes and retain the final confirmation as held.
#[tokio::test]
async fn b8_runtime_round_limits() {
    for max in 0..=2 {
        let mut h = Harness::new(max, false).await;
        for round in 0..=max {
            h.to_confirm().await;
            h.confirmed(h.labels("confirmed")).await;
            if round < max {
                assert_eq!(h.decide().await, RoundDecision::Fix);
                h.executor(true, round + 1).await;
            } else {
                assert_eq!(h.decide().await, RoundDecision::Hold("round_limit"));
                assert_eq!(h.step().state, "held");
                assert_eq!(h.step().body["reason"], "round_limit");
                assert_eq!(h.step().round, max);
            }
        }
        let view = h.f.store().pipeline_goal_view(h.f.goal).unwrap();
        assert_eq!(
            view.steps.iter().filter(|s| s.kind == "fix").count(),
            max as usize
        );
        h.close().await;
    }
}

/// A confirmed label without matching source evidence becomes unclear and holds its own step.
#[tokio::test]
async fn b8_runtime_unclear() {
    let h = Harness::new(2, false).await;
    h.to_confirm().await;
    let mut labels = h.labels("confirmed");
    labels["labels"][0]["evidence"]["quote"] = json!("not in the source");
    h.confirmed(labels).await;
    assert_eq!(
        h.step().body["adoption"]["confirmation"]["labels"][0]["label"],
        "unclear"
    );
    assert_eq!(h.decide().await, RoundDecision::Hold("unclear_finding"));
    assert_eq!(h.step().state, "held");
    assert_eq!(h.step().body["reason"], "unclear_finding");
    h.close().await;
}

/// Invalid findings cannot support request_changes or cause an automatic fix or re-review.
#[tokio::test]
async fn b8_runtime_unsupported() {
    let h = Harness::new(2, false).await;
    assert_eq!(h.test().await, RoundDecision::Review);
    let mut findings = h.findings();
    findings[0]["file"] = json!("src/missing.rs");
    h.reviewed(findings, "request_changes").await;
    assert_eq!(
        h.decide().await,
        RoundDecision::Hold("unsupported_request_changes")
    );
    assert_eq!(h.step().kind, "review");
    assert_eq!(h.step().state, "held");
    assert_eq!(h.step().body["reason"], "unsupported_request_changes");
    assert_eq!(
        h.step().body["filtering"]["invalid"][0]["reason"],
        "missing_file"
    );
    h.close().await;
}

/// Reviewer path aliases are recorded as missing from the reviewed tree before confirmation.
#[tokio::test]
async fn b8_runtime_filter_path_alias() {
    let h = Harness::new(1, false).await;
    assert_eq!(h.test().await, RoundDecision::Review);
    let mut findings = h.findings();
    findings[0]["file"] = json!("src/OUTPUT.rs");
    h.reviewed(findings, "request_changes").await;
    assert_eq!(
        h.decide().await,
        RoundDecision::Hold("unsupported_request_changes")
    );
    let step = h.step();
    assert_eq!(step.state, "held");
    assert_eq!(step.body["reason"], "unsupported_request_changes");
    assert_eq!(
        step.body["filtering"]["invalid"],
        json!([{"id":"F1","reason":"missing_file"}])
    );
    h.close().await;
}

/// Failed tests skip review and supply only bounded stored observations to the fix.
#[tokio::test]
async fn b8_runtime_failed_test() {
    let mut h = Harness::new(1, true).await;
    assert_eq!(h.test().await, RoundDecision::Fix);
    assert_eq!(h.step().body["fix_input"]["kind"], "test_failure");
    assert_eq!(h.step().body["fix_input"]["exit_code"], 1);
    assert_eq!(h.step().body["confirmed_ids"], json!([]));
    h.executor(true, 1).await;
    assert_eq!(h.test().await, RoundDecision::Hold("round_limit"));
    assert_eq!(h.step().kind, "test");
    assert_eq!(h.step().state, "held");
    assert_eq!(h.step().body["reason"], "round_limit");
    assert!(
        !h.f.store()
            .pipeline_goal_view(h.f.goal)
            .unwrap()
            .steps
            .iter()
            .any(|s| matches!(s.kind.as_str(), "review" | "confirm"))
    );
    h.close().await;
}

/// Extra labels reject the confirmation with a fixed reason and no adopted label data.
#[tokio::test]
async fn b8_runtime_label_rejection() {
    let h = Harness::new(1, false).await;
    h.to_confirm().await;
    let mut labels = h.labels("confirmed");
    let mut extra = labels["labels"][0].clone();
    extra["id"] = json!("F2");
    labels["labels"].as_array_mut().unwrap().push(extra);
    h.confirmation_exit(&serde_json::to_string(&labels).unwrap())
        .await;
    h.runtime.wait_pipeline_test_boundary().await;
    assert!(
        !h.runtime
            .adopt_pipeline_confirm(h.f.cursor(&h.id))
            .await
            .unwrap()
    );
    assert_eq!(h.step().state, "rejected");
    assert_eq!(h.step().body["reason"], "confirm_label_ids");
    assert!(h.step().body["adoption"].is_null());
    h.close().await;
}

/// Secrets in confirmation output hold with only a pattern and offset digest.
#[tokio::test]
async fn b8_runtime_confirmation_secrets() {
    for source in ["last", "stdout", "stderr"] {
        let h = Harness::new(1, false).await;
        h.to_confirm().await;
        let mut labels = h.labels("confirmed");
        if source == "stderr" {
            h.codex.control(
                "exec-stderr",
                &format!("EXAMPLE_TOKEN=fixture\n{}", "x".repeat(1024 * 1024)),
            );
        } else if source == "stdout" {
            h.codex
                .control("exec-pre-output", "EXAMPLE_TOKEN=fixture\n");
        } else {
            labels["labels"][0]["reason"] = json!("EXAMPLE_TOKEN=fixture");
        }
        h.confirmation_exit(&serde_json::to_string(&labels).unwrap())
            .await;
        if source == "last" {
            h.runtime.wait_pipeline_test_boundary().await;
            assert!(
                !h.runtime
                    .adopt_pipeline_confirm(h.f.cursor(&h.id))
                    .await
                    .unwrap()
            );
        }
        assert_eq!(h.step().state, "held");
        assert_eq!(h.step().body["reason"], "secret_detected");
        assert_eq!(h.step().body["secret"]["pattern"], "secret_assignment");
        assert_eq!(
            h.step().body["secret"]["offset_sha256"]
                .as_str()
                .unwrap()
                .len(),
            64
        );
        assert!(
            !serde_json::to_string(&h.step().body)
                .unwrap()
                .contains("fixture")
        );
        h.close().await;
    }
}

/// Secrets in fix output hold with only a pattern and offset digest before adoption.
#[tokio::test]
async fn b8_runtime_fix_output_secrets() {
    for source in ["stdout", "stderr"] {
        let h = Harness::new(1, true).await;
        assert_eq!(h.test().await, RoundDecision::Fix);
        h.codex.control(
            if source == "stdout" {
                "exec-pre-output"
            } else {
                "exec-stderr"
            },
            "EXAMPLE_TOKEN=fixture\n",
        );
        let work = PathBuf::from(
            h.f.store().pipeline_goal_view(h.f.goal).unwrap().runs[0]
                .worktree
                .as_ref()
                .unwrap(),
        );
        h.expected(&h.codex, PipelineRole::Fix, &work, "");
        h.runtime.wait_pipeline_test_boundary().await;
        let LaunchResult::Started(custody) = h
            .runtime
            .launch_pipeline_fix(h.f.cursor(&h.id))
            .await
            .unwrap()
        else {
            panic!("fix did not launch: {:?}", h.step());
        };
        custody.wait().await.unwrap();
        let step = h.step();
        assert_eq!(step.state, "held");
        assert_eq!(step.body["reason"], "secret_detected");
        assert_eq!(
            step.body["secret"],
            json!({"pattern":"secret_assignment","offset_sha256":step.body["exit"]["secret"]["offset_sha256"]})
        );
        assert_eq!(
            step.body["secret"]["offset_sha256"].as_str().unwrap().len(),
            64
        );
        assert!(
            !serde_json::to_string(&step.body)
                .unwrap()
                .contains("fixture")
        );
        assert!(step.result_commit.is_none());
        h.close().await;
    }
}

/// Generic confirmation launch cannot bypass a second snapshot digest check.
#[tokio::test]
async fn b8_runtime_generic_snapshot_guard() {
    let h = Harness::new(1, false).await;
    h.to_confirm().await;
    h.runtime.wait_pipeline_test_boundary().await;
    let input = h
        .runtime
        .prepare_pipeline_confirm(h.f.cursor(&h.id))
        .await
        .unwrap()
        .unwrap();
    let file = input.work.join("src/output.rs");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::write(&file, "changed\n").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).unwrap();
    h.runtime.wait_pipeline_test_boundary().await;
    assert!(matches!(
        h.runtime
            .launch_pipeline_step(h.f.cursor(&h.id), input)
            .await
            .unwrap(),
        LaunchResult::Held
    ));
    assert_eq!(h.step().body["reason"], "review_snapshot_changed");
    h.close().await;
}

/// Generic Executor launch cannot substitute arbitrary text for the confirmed-only fix input.
#[tokio::test]
async fn b8_runtime_fix_input_guard() {
    let h = Harness::new(1, false).await;
    h.to_confirm().await;
    h.confirmed(h.labels("confirmed")).await;
    assert_eq!(h.decide().await, RoundDecision::Fix);
    h.runtime.wait_pipeline_test_boundary().await;
    assert!(matches!(
        h.runtime
            .launch_pipeline_executor(h.f.cursor(&h.id), b"substituted prompt".to_vec())
            .await
            .unwrap(),
        LaunchResult::Held
    ));
    assert_eq!(h.step().body["reason"], "fix_input_changed");
    h.close().await;
}

/// Fix planning permits a single round increment and rejects same-round and skipped-round fixes.
#[tokio::test]
async fn b8_runtime_round_increment() {
    let h = Harness::new(2, false).await;
    h.to_confirm().await;
    h.confirmed(h.labels("confirmed")).await;
    let before = h.f.store().pipeline_goal_view(h.f.goal).unwrap();
    for round in [0, 2] {
        let mut next = planned(&h.id, h.step().seq + 1, 1, "fix");
        next.round = round;
        next.step_base = before.runs[0].head_commit.clone();
        let cursor = h.f.cursor(&h.id);
        assert_eq!(
            h.f.store()
                .pipeline_apply_round(&h.f.owner, &cursor, Some(&next))
                .unwrap_err()
                .to_string(),
            "round_next_changed"
        );
        assert_eq!(
            h.f.store().pipeline_goal_view(h.f.goal).unwrap().runs,
            before.runs
        );
    }
    h.close().await;
}

/// Rejected labels remain recorded and never authorize remediation of request_changes.
#[tokio::test]
async fn b8_runtime_rejected_only() {
    let h = Harness::new(2, false).await;
    h.to_confirm().await;
    h.confirmed(h.labels("rejected")).await;
    assert_eq!(
        h.decide().await,
        RoundDecision::Hold("unsupported_request_changes")
    );
    assert_eq!(h.step().state, "held");
    assert_eq!(h.step().body["reason"], "unsupported_request_changes");
    assert_eq!(
        h.step().body["adoption"]["confirmation"]["labels"][0]["label"],
        "rejected"
    );
    h.close().await;
}

/// A paused Goal cannot advance a staged confirmation or change its step during decision.
#[tokio::test]
async fn b8_runtime_decision_rule_g() {
    let h = Harness::new(1, false).await;
    h.to_confirm().await;
    h.confirmed(h.labels("confirmed")).await;
    let before = h.step();
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
    h.runtime.wait_pipeline_test_boundary().await;
    assert!(
        h.runtime
            .decide_pipeline_round(h.f.cursor(&h.id))
            .await
            .is_err()
    );
    assert_eq!(h.step(), before);
    h.close().await;
}

/// Secret-bearing confirmation diagnostics cannot enter a staged Store body.
#[tokio::test]
async fn b8_runtime_store_scan() {
    let h = Harness::new(1, false).await;
    h.to_confirm().await;
    let labels = h.labels("confirmed");
    let bytes = serde_json::to_vec(&labels).unwrap();
    h.confirmation_exit(std::str::from_utf8(&bytes).unwrap())
        .await;
    let before = h.step();
    let findings =
        serde_json::from_value::<Vec<crate::execution::pipeline::ReviewFinding>>(h.findings())
            .unwrap();
    let mut evidence = confirm::decode(&bytes, &findings, &h.path()).unwrap();
    evidence["snapshot_tree"] = before.body["snapshot_tree"].clone();
    evidence["diagnostic"] = json!({"nested":["EXAMPLE_TOKEN=fixture"]});
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
    assert_eq!(h.step(), before);
    h.close().await;
}

/// A secret in a decided fix body rolls back the step adoption, insertion and round increment.
#[tokio::test]
async fn b8_runtime_plan_scan() {
    let h = Harness::new(1, false).await;
    h.to_confirm().await;
    h.confirmed(h.labels("confirmed")).await;
    let before = h.f.store().pipeline_goal_view(h.f.goal).unwrap();
    let cursor = h.f.cursor(&h.id);
    let ctx =
        h.f.store()
            .pipeline_round_context(&h.f.owner, &cursor)
            .unwrap();
    let input = round_fix_input(&ctx).unwrap();
    let nonce = Uuid::new_v4().simple().to_string();
    let prompt = crate::execution::pipeline::rounds::fix_prompt(&nonce, &input).unwrap();
    let mut next = planned(&h.id, h.step().seq + 1, 1, "fix");
    next.round = 1;
    next.step_base = before.runs[0].head_commit.clone();
    next.input_sha256 = crate::execution::verification::hash(&prompt);
    let digest = crate::execution::verification::hash(&serde_json::to_vec(&input).unwrap());
    next.body = json!({"fix_nonce":nonce,"fix_input":input,"fix_input_sha256":digest,
        "confirmed_ids":input.confirmed_ids(),"diagnostic":"EXAMPLE_TOKEN=fixture"});
    let error =
        h.f.store()
            .pipeline_apply_round(&h.f.owner, &cursor, Some(&next))
            .unwrap_err();
    assert!(
        error
            .downcast_ref::<crate::execution::pipeline::SecretHit>()
            .is_some()
    );
    let after = h.f.store().pipeline_goal_view(h.f.goal).unwrap();
    assert_eq!(after.steps, before.steps);
    assert_eq!(after.runs, before.runs);
    h.close().await;
}

/// A nonzero Confirmer exit rejects the step instead of accepting otherwise valid labels.
#[tokio::test]
async fn b8_runtime_agent_failure() {
    let h = Harness::new(1, false).await;
    h.to_confirm().await;
    h.codex.control("exec-status", "1\n");
    h.confirmation_exit(&serde_json::to_string(&h.labels("confirmed")).unwrap())
        .await;
    h.runtime.wait_pipeline_test_boundary().await;
    assert!(
        !h.runtime
            .adopt_pipeline_confirm(h.f.cursor(&h.id))
            .await
            .unwrap()
    );
    assert_eq!(h.step().state, "rejected");
    assert_eq!(h.step().body["reason"], "confirm_agent_failed");
    h.close().await;
}

/// A review body that fits only without the staging mark is held with review_storage_limit.
#[tokio::test]
async fn b8_r2_review_staging_bound() {
    let h = Harness::new(1, false).await;
    assert_eq!(h.test().await, RoundDecision::Review);
    h.review_exit(json!([]), "approve").await;
    let mut step = h.step();
    let cursor = h.f.cursor(&h.id);
    let (_, _, threshold) =
        h.f.store()
            .pipeline_review_context(&h.f.owner, &cursor)
            .unwrap();
    let bytes = review::output(&step, &h.attempt()).unwrap();
    let model = &h.claude.cli.model;
    let mut evidence = VerdictDecoder::decode(&bytes, model, threshold).unwrap();
    evidence["snapshot_tree"] = step.body["snapshot_tree"].clone();
    let unstaged = |n: usize| {
        let mut body = step.body.clone();
        body["pad"] = json!("x".repeat(n));
        body["adoption"] = evidence.clone();
        body
    };
    let (mut low, mut high) = (0usize, 262_144usize);
    while low < high {
        let mid = (low + high).div_ceil(2);
        if step_body_fits(&unstaged(mid)).unwrap() {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    let mut staged = unstaged(low);
    staged["round_result_validated"] = json!(true);
    assert!(step_body_fits(&unstaged(low)).unwrap());
    assert!(!step_body_fits(&staged).unwrap());
    step.body["pad"] = json!("x".repeat(low));
    {
        let mut store = h.f.store();
        let tx = store.connection.transaction().unwrap();
        write_step(&tx, &h.f.owner, &step).unwrap();
        tx.commit().unwrap();
    }
    h.runtime.wait_pipeline_test_boundary().await;
    assert!(
        !h.runtime
            .adopt_pipeline_round_review(h.f.cursor(&h.id))
            .await
            .unwrap()
    );
    assert_eq!(h.step().state, "held");
    assert_eq!(h.step().body["reason"], "review_storage_limit");
    h.close().await;
}

fn passing_exit() -> Value {
    json!({"exit_code":0,"signal":null,"timed_out":false,"overflow":false,"issue":null,
        "stdout":{"bytes":0,"sha256":crate::execution::verification::hash(b""),"complete":true},
        "stderr":{"bytes":0,"sha256":crate::execution::verification::hash(b""),"complete":true}})
}

/// Exits one step of `kind` after an adopted prepare with a certified passing observation.
fn store_exited(f: &Fixture, kind: &str) -> String {
    let id = f.claim();
    f.launching(&id, kind);
    let cursor = f.cursor(&id);
    let outcome = (kind != "test").then_some("success");
    f.store()
        .pipeline_record_exit(&f.owner, &cursor, &death(), outcome, &passing_exit())
        .unwrap();
    id
}

/// Task-run test, review and confirmation results cannot be adopted without a decision.
#[tokio::test]
async fn b8_r1_round_adoption_refused() {
    for kind in ["test", "review", "confirm"] {
        let f = fixture().await;
        let id = store_exited(&f, kind);
        let cursor = f.cursor(&id);
        let evidence = json!({"command_id":"test","result":"pass"});
        let error = f
            .store()
            .pipeline_adopt(&f.owner, &cursor, &evidence)
            .unwrap_err();
        assert_eq!(error.to_string(), "round_adoption_required", "{kind}");
        assert_eq!(f.cursor(&id).step_version, cursor.step_version, "{kind}");
    }
}

/// A staged round result cannot be staged a second time.
#[tokio::test]
async fn b8_r1_stage_once() {
    let f = fixture().await;
    let id = store_exited(&f, "test");
    let evidence = json!({"command_id":"test","result":"pass"});
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_stage_round_result(&f.owner, &cursor, &evidence)
        .unwrap();
    let staged = f.cursor(&id);
    let error = f
        .store()
        .pipeline_stage_round_result(&f.owner, &staged, &evidence)
        .unwrap_err();
    assert_eq!(error.to_string(), "round_result_staged");
    assert_eq!(f.cursor(&id).step_version, staged.step_version);
}

/// Undecided planning after prepare refuses every step kind except implement.
#[tokio::test]
async fn b8_r1_plan_requires_decision() {
    let f = fixture().await;
    let id = f.claim();
    f.advance_prepare(&id);
    for kind in ["test", "review", "confirm", "fix", "release"] {
        let mut next = planned(&id, 2, 1, kind);
        next.round = u32::from(kind == "fix");
        let cursor = f.cursor(&id);
        let error = f
            .store()
            .pipeline_plan_next(&f.owner, &cursor, &next, true)
            .unwrap_err();
        assert_eq!(error.to_string(), "round_decision_required", "{kind}");
        assert_eq!(f.cursor(&id).step_version, cursor.step_version, "{kind}");
    }
}

/// The decision path cannot plan a review that carries caller-supplied filtering.
#[tokio::test]
async fn b8_r1_decided_filtering_reserved() {
    let f = fixture().await;
    let id = store_exited(&f, "test");
    let cursor = f.cursor(&id);
    let evidence = json!({"command_id":"test","result":"pass"});
    f.store()
        .pipeline_stage_round_result(&f.owner, &cursor, &evidence)
        .unwrap();
    let before = f.store().pipeline_task_view(f.goal, f.task).unwrap();
    let mut next = planned(&id, 3, 1, "review");
    next.body["review_nonce"] = json!(Uuid::new_v4().simple().to_string());
    next.body["filtering"] = json!({"blocking":[],"invalid":[],"non_blocking":[]});
    let cursor = f.cursor(&id);
    let error = f
        .store()
        .pipeline_apply_round(&f.owner, &cursor, Some(&next))
        .unwrap_err();
    assert_eq!(error.to_string(), "review_store_fields_reserved");
    assert_eq!(
        f.store().pipeline_task_view(f.goal, f.task).unwrap().steps,
        before.steps
    );
}

/// The largest body the bound admits survives a later hold, and one more byte is refused.
#[tokio::test]
async fn b8_r1_body_bound_boundary() {
    let fits = |n: usize| step_body_fits(&json!({"pad":"x".repeat(n)})).unwrap();
    let (mut low, mut high) = (0usize, 262_144usize);
    while low < high {
        let mid = (low + high).div_ceil(2);
        if fits(mid) {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    assert!(!fits(low + 1) && (261_000..262_144).contains(&low));
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    let mut step = f
        .store()
        .pipeline_task_view(f.goal, f.task)
        .unwrap()
        .steps
        .pop()
        .unwrap();
    step.body["pad"] = json!("x".repeat(low));
    {
        let mut store = f.store();
        let tx = store.connection.transaction().unwrap();
        write_step(&tx, &f.owner, &step).unwrap();
        tx.commit().unwrap();
    }
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_hold(&f.owner, &cursor, "confirm_storage_limit", None)
        .unwrap();
    let held = f
        .store()
        .pipeline_task_view(f.goal, f.task)
        .unwrap()
        .steps
        .pop()
        .unwrap();
    assert_eq!(held.body["reason"], "confirm_storage_limit");
}

/// Stages one passing Task-round test result as the round decision source.
fn staged_test(f: &Fixture) -> String {
    let id = store_exited(f, "test");
    let cursor = f.cursor(&id);
    let evidence = json!({"command_id":"test","result":"pass"});
    f.store()
        .pipeline_stage_round_result(&f.owner, &cursor, &evidence)
        .unwrap();
    id
}

/// Changes the stored TestsProfile catalog body so it no longer verifies against its digest.
fn change_catalog(f: &Fixture) {
    f.store().connection.execute("INSERT OR REPLACE INTO verification_profiles(project_id,digest,body) SELECT project_id,digest,body || ' ' FROM verification_profiles WHERE project_id=?1", [f.project.to_string()]).unwrap();
}

/// Asserts the staged test, with no planned successor, is held with its run.
fn assert_profile_hold(f: &Fixture) {
    let view = f.store().pipeline_task_view(f.goal, f.task).unwrap();
    let step = view.steps.last().unwrap();
    assert_eq!((step.kind.as_str(), step.state.as_str()), ("test", "held"));
    assert_eq!(step.body["reason"], "tests_profile_changed");
    assert_eq!(view.runs[0].state, "held");
}

/// Applying a decision directly after a catalog change holds tests_profile_changed.
#[tokio::test]
async fn b8_r2_apply_profile_changed() {
    let f = fixture().await;
    let id = staged_test(&f);
    change_catalog(&f);
    let cursor = f.cursor(&id);
    let error = f
        .store()
        .pipeline_apply_round(&f.owner, &cursor, None)
        .unwrap_err();
    assert_eq!(error.to_string(), "tests_profile_changed");
    assert_profile_hold(&f);
}

/// A catalog change after the decision's context read holds at its final transaction.
#[tokio::test]
async fn b8_r2_decision_profile_changed() {
    let f = fixture().await;
    let id = staged_test(&f);
    let cursor = f.cursor(&id);
    let ctx = f.store().pipeline_round_context(&f.owner, &cursor).unwrap();
    assert_eq!(round_action(&ctx).unwrap(), RoundDecision::Review);
    let mut next = planned(&id, ctx.step.seq + 1, 1, "review");
    next.body["review_nonce"] = json!(Uuid::new_v4().simple().to_string());
    change_catalog(&f);
    let error = f
        .store()
        .pipeline_apply_round(&f.owner, &cursor, Some(&next))
        .unwrap_err();
    assert_eq!(error.to_string(), "tests_profile_changed");
    assert_profile_hold(&f);
}

/// A catalog mismatch before a re-test plans nothing runnable and holds tests_profile_changed.
#[tokio::test]
async fn b8_r1_retest_profile_changed() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(&f.owner, &cursor, &death(), Some("success"), &json!({}))
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_commit(
            &f.owner,
            &cursor,
            &"b".repeat(40),
            &"c".repeat(40),
            &json!({}),
        )
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_adopt(&f.owner, &cursor, &json!({}))
        .unwrap();
    change_catalog(&f);
    let cursor = f.cursor(&id);
    let error = f
        .store()
        .pipeline_round_context(&f.owner, &cursor)
        .err()
        .unwrap();
    assert_eq!(error.to_string(), "tests_profile_changed");
    let view = f.store().pipeline_task_view(f.goal, f.task).unwrap();
    let step = view.steps.last().unwrap();
    assert_eq!((step.kind.as_str(), step.state.as_str()), ("test", "held"));
    assert_eq!(step.body["reason"], "tests_profile_changed");
    assert_eq!(view.runs[0].state, "held");
}
