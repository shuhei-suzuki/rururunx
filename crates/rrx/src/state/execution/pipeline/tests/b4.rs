use super::*;
use crate::execution::pipeline::{
    LaunchResult, PipelineRole, StepCustody, StepInput, step_arguments, test_support::FakeCli,
};
use std::{collections::BTreeSet, path::PathBuf, sync::Arc};

struct Harness {
    f: Fixture,
    runtime: Arc<Runtime>,
    id: String,
    work: PathBuf,
    fake: FakeCli,
    prompt: Vec<u8>,
}
impl Harness {
    async fn new(kind: &str) -> Self {
        Self::with_prompt(kind, b"pipeline\n").await
    }
    async fn with_prompt(kind: &str, prompt: &[u8]) -> Self {
        Self::with_alias(kind, prompt, false).await
    }
    async fn with_alias(kind: &str, prompt: &[u8], alias: bool) -> Self {
        Self::build(kind, prompt, alias, None).await
    }
    async fn build(kind: &str, prompt: &[u8], alias: bool, pin: Option<&str>) -> Self {
        if kind == "review" {
            let h = super::b7::ReviewFixture::new(
                "{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"structured_output\":{}}",
            )
            .await;
            let input = h.prepare().await.unwrap();
            return Self {
                f: h.f,
                runtime: h.runtime,
                id: h.id,
                work: h.path,
                fake: h.fake,
                prompt: input.prompt,
            };
        }
        let f = if let Some(pin) = pin {
            let inputs = FixtureInputs {
                max_fix_rounds: None,
                clis: None,
                codex_pin: Some(pin.into()),
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
                qualification: Arc::new(std::sync::Mutex::new(Value::Null)),
            };
            fixture_build(true, true, None, None, alias, None, Some(inputs))
                .await
                .unwrap()
        } else if alias {
            fixture_with_service_and_alias(true, true, None, None, true).await
        } else {
            fixture().await
        };
        let runtime = Arc::new(Runtime::new(f.owner.clone(), f.config.clone()).unwrap());
        runtime.start_qualified_service().await.unwrap();
        let pins = f.config.pipeline.as_ref().unwrap();
        let work = pins
            .work_root
            .join(format!("work_{}", Uuid::new_v4().simple()));
        std::fs::create_dir(&work).unwrap();
        let id = f.claim_in(Some(&work));
        f.advance_prepare(&id);
        let mut next = planned(&id, 2, 1, kind);
        next.input_sha256 = digest(prompt);
        let cursor = f.cursor(&id);
        f.store()
            .pipeline_plan_next(&f.owner, &cursor, &next, true)
            .unwrap();
        let (role, cli) = match kind {
            "implement" => (PipelineRole::Implement, &pins.clis.codex),
            "review" => (PipelineRole::Review, &pins.clis.claude),
            _ => panic!("unsupported harness role"),
        };
        let fake = FakeCli {
            directory: cli.executable.parent().unwrap().to_owned(),
            cli: cli.clone(),
        };
        let attempt = pins.work_root.join("runs").join(&id).join("2-1");
        let args = step_arguments(role, cli, &work, &attempt, "{}").unwrap();
        fake.expect_arguments(&args.iter().map(String::as_str).collect::<Vec<_>>());
        fake.control(
            "exec-output",
            if kind == "review" {
                "{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"structured_output\":{}}"
            } else {
                "{\"type\":\"turn.started\"}\n{\"type\":\"turn.completed\"}\n"
            },
        );
        Self {
            f,
            runtime,
            id,
            work,
            fake,
            prompt: prompt.to_vec(),
        }
    }
    fn input(&self) -> StepInput {
        StepInput {
            work: self.work.clone(),
            prompt: self.prompt.clone(),
            schema: if self.step().kind == "review" {
                crate::execution::pipeline::review::REVIEW_SCHEMA
            } else {
                "{}"
            }
            .into(),
        }
    }
    async fn launch(&self) -> LaunchResult {
        if self.step().kind == "review" {
            self.runtime.wait_pipeline_test_boundary().await;
            return self
                .runtime
                .launch_pipeline_review(self.f.cursor(&self.id))
                .await
                .unwrap();
        }
        self.runtime
            .launch_pipeline_step(self.f.cursor(&self.id), self.input())
            .await
            .unwrap()
    }
    fn step(&self) -> PipelineStep {
        self.f
            .store()
            .pipeline_task_view(self.f.goal, self.f.task)
            .unwrap()
            .steps
            .last()
            .unwrap()
            .clone()
    }
    async fn started(&self) -> StepCustody {
        match self.launch().await {
            LaunchResult::Started(custody) => custody,
            _ => panic!("expected started"),
        }
    }
    async fn close(self) {
        self.runtime.shutdown().await.unwrap();
        if self.step().kind == "review" {
            crate::execution::pipeline::review_snapshot::remove(&self.work).unwrap();
            let step = self.step();
            let path = crate::execution::pipeline::review_snapshot::snapshot_path(
                &self.f.config.pipeline.as_ref().unwrap().work_root,
                &self.id,
                step.seq,
                step.attempt,
            )
            .unwrap();
            crate::execution::pipeline::review_snapshot::remove(&path).unwrap();
        }
    }

    fn review_retry(&self) -> PipelineStep {
        let cursor = self.f.cursor(&self.id);
        let (_, task, _) = self
            .f
            .store()
            .pipeline_review_context(&self.f.owner, &cursor)
            .unwrap();
        let step = self.step();
        let nonce = Uuid::new_v4().simple().to_string();
        let prompt = crate::execution::pipeline::review_prompt(&nonce, &task, b"").unwrap();
        let mut retry = planned(&self.id, 2, 2, "review");
        retry.step_base = step.step_base;
        retry.body["review_nonce"] = json!(nonce);
        retry.input_sha256 = digest(&prompt);
        retry
    }
}

fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

/// A launched native CLI records identity and settles only with every death observation.
#[tokio::test]
async fn b4_launch_receipt_and_death() {
    let h = Harness::new("implement").await;
    h.fake.control("exec-last", "complete\n");
    h.fake.control("exec-pause", "");
    let custody = h.started().await;
    let running = h.step();
    let pid = running.pid.unwrap();
    let cookie = rrx_process_tracker::Cookie::new(running.cookie.as_ref().unwrap()).unwrap();
    let identity = tokio::task::spawn_blocking(move || {
        rrx_process_tracker::discover(&cookie, Default::default())
            .unwrap()
            .processes
            .iter()
            .map(|p| p.identity())
            .find(|id| id.pid == pid)
            .unwrap()
    })
    .await
    .unwrap();
    assert_eq!(
        running.birth.as_deref(),
        Some(format!("{}:{}", identity.birth.0, identity.birth.1).as_str())
    );
    let pgid = rustix::process::getpgid(rustix::process::Pid::from_raw(pid as i32)).unwrap();
    assert_eq!(pgid.as_raw_nonzero().get() as u32, pid);
    assert_eq!(running.body["pgid"], pgid.as_raw_nonzero().get());
    h.fake.control("exec-release", "");
    custody.wait().await.unwrap();
    let step = h.step();
    assert_eq!(step.state, "exited");
    assert_eq!(step.body["pgid"], step.pid.unwrap());
    assert_eq!(step.body["provider_outcome"], "success");
    let proof: PipelineDeathEvidence =
        serde_json::from_value(step.body["death_evidence"].clone()).unwrap();
    proof.validate(true).unwrap();
    assert_eq!(
        std::fs::read(h.fake.directory.join("stdin")).unwrap(),
        b"pipeline\n"
    );
    h.close().await;
}

/// The native CLI receives stdin EOF and exits successfully without a stop request.
#[tokio::test]
async fn b4_stdin_eof() {
    let h = Harness::new("implement").await;
    h.fake.control("exec-last", "complete\n");
    let custody = h.started().await;
    // Bound the regression test well below the production input and step deadlines.
    let result = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while !h.fake.directory.join("stdin-eof").exists() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await;
    if result.is_err() {
        // Clean up the stalled child only on failure, never to produce success.
        custody.request_stop();
        custody.wait().await.unwrap();
        h.close().await;
        panic!("native CLI did not receive stdin EOF and finish");
    }
    custody.wait().await.unwrap();
    let step = h.step();
    assert_eq!(step.state, "exited");
    assert_eq!(step.body["exit"]["code"], 0);
    assert!(step.body["exit"]["issue"].is_null());
    assert_eq!(step.body["provider_outcome"], "success");
    assert_eq!(
        std::fs::read(h.fake.directory.join("stdin")).unwrap(),
        h.prompt
    );
    h.close().await;
}

/// Agent launches never inherit credentials or environment keys outside the allowlist.
#[tokio::test]
async fn b4_launch_environment_subset() {
    let h = Harness::new("implement").await;
    h.started().await.wait().await.unwrap();
    let text = std::fs::read_to_string(h.fake.directory.join("environment")).unwrap();
    let allowed = BTreeSet::from([
        "HOME",
        "USER",
        "LOGNAME",
        "PATH",
        "LANG",
        "TMPDIR",
        "TERM",
        "NO_COLOR",
        "RRX_PROCESS_COOKIE",
        "CODEX_HOME",
    ]);
    let actual: BTreeSet<_> = text
        .lines()
        .map(|line| line.split_once('=').unwrap().0)
        .collect();
    assert!(actual.is_subset(&allowed), "{actual:?}");
    assert!(actual.contains("RRX_PROCESS_COOKIE"));
    h.close().await;
}

/// Output is consumed while a large stdin is still being delivered.
#[tokio::test]
async fn b4_concurrent_input_output() {
    let h = Harness::with_prompt("implement", &vec![b'p'; 512 * 1024]).await;
    h.fake.control("exec-last", "complete\n");
    h.fake.control("exec-pre-output", &" ".repeat(256 * 1024));
    h.started().await.wait().await.unwrap();
    assert_eq!(h.step().body["provider_outcome"], "success");
    assert_eq!(
        h.step().body["exit"]["stdout"]["bytes"],
        256 * 1024 + b"{\"type\":\"turn.started\"}\n{\"type\":\"turn.completed\"}\n".len()
    );
    h.close().await;
}

/// An executable changed after Q is rejected before any execution call.
#[tokio::test]
async fn b4_before_launch_rechecks_q() {
    let h = Harness::new("implement").await;
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(&h.fake.cli.executable)
        .unwrap()
        .write_all(b"changed")
        .unwrap();
    assert!(matches!(h.launch().await, LaunchResult::Held));
    assert_eq!(h.step().state, "rejected");
    assert!(!h.fake.directory.join("exec-entered").exists());
    h.close().await;
}

/// A prompt whose digest differs from the plan never reaches the native CLI.
#[tokio::test]
async fn b4_prompt_digest() {
    let h = Harness::new("implement").await;
    let mut input = h.input();
    input.prompt = b"different\n".to_vec();
    let result = h
        .runtime
        .launch_pipeline_step(h.f.cursor(&h.id), input)
        .await
        .unwrap();
    assert!(matches!(result, LaunchResult::Held));
    assert!(!h.fake.directory.join("exec-entered").exists());
    h.close().await;
}

/// A stopped Runtime leaves a planned step untouched and never spawns it.
#[tokio::test]
async fn b4_stopping_boundary() {
    let h = Harness::new("implement").await;
    h.runtime.shutdown().await.unwrap();
    assert!(matches!(h.launch().await, LaunchResult::Waiting));
    assert_eq!(h.step().state, "planned");
    assert!(!h.fake.directory.join("exec-entered").exists());
}

/// Stop notifications settle as interrupted only after confirmed process death.
#[tokio::test]
async fn b4_stop_requires_death() {
    let h = Harness::new("implement").await;
    h.fake.control("exec-pause", "");
    let custody = h.started().await;
    custody.request_stop();
    custody.wait().await.unwrap();
    let step = h.step();
    assert_eq!(step.state, "interrupted");
    assert_eq!(step.body["death_confirmed"], true);
    h.close().await;
}

/// Stdout overflow kills the owned process and rejects its captured result.
#[tokio::test]
async fn b4_live_output_overflow() {
    let h = Harness::new("review").await;
    h.fake
        .control("exec-pre-output", &"x".repeat(1024 * 1024 + 1));
    h.started().await.wait().await.unwrap();
    let step = h.step();
    assert_eq!(step.state, "rejected");
    assert_eq!(step.body["reason"], "output_overflow");
    assert_eq!(step.body["death_confirmed"], true);
    h.close().await;
}

/// A failed running-record transaction rejects and holds only after proven death.
#[tokio::test]
async fn b4_record_failure_death() {
    let h = Harness::new("implement").await;
    h.f.store().connection.execute_batch("CREATE TEMP TRIGGER b4_fail_running BEFORE UPDATE ON pipeline_steps WHEN NEW.state='running' BEGIN SELECT RAISE(ABORT,'injected running failure'); END;").unwrap();
    assert!(matches!(h.launch().await, LaunchResult::RecordFailed));
    let step = h.step();
    assert_eq!(step.state, "rejected");
    assert_eq!(step.body["exit"]["issue"], "record_failed");
    assert_eq!(step.body["reason"], "record_failed");
    assert_eq!(step.body["provider_outcome"], "other");
    assert_eq!(step.body["death_confirmed"], true);
    assert_eq!(
        h.f.store()
            .pipeline_task_view(h.f.goal, h.f.task)
            .unwrap()
            .runs
            .iter()
            .find(|run| run.id == h.id)
            .unwrap()
            .state,
        "held"
    );
    h.close().await;
}

/// A spawn error remains unknown even when the executable's qualified bytes match.
#[tokio::test(flavor = "current_thread")]
async fn b4_spawn_error_is_unknown() {
    use crate::execution::pipeline::test_support::{LaunchPhase, pause_launch};
    let h = Harness::new("implement").await;
    let result = {
        let (release, arrived) = pause_launch(LaunchPhase::Verified);
        let launch = h.runtime.launch_pipeline_step(h.f.cursor(&h.id), h.input());
        tokio::pin!(launch);
        tokio::select! {
            _ = &mut launch => panic!("launch bypassed Verified checkpoint"),
            arrival = arrived => arrival.unwrap(),
        }
        // Remove only this fixture's empty cwd after qualification to force spawn failure.
        std::fs::remove_dir(&h.work).unwrap();
        release.send(()).unwrap();
        launch.await.unwrap()
    };
    assert!(matches!(result, LaunchResult::Unknown));
    assert_eq!(h.step().state, "unknown");
    assert_eq!(h.step().body["reason"], "launch_unknown");
    assert_eq!(h.step().body["death_confirmed"], false);
    assert!(!h.fake.directory.join("exec-entered").exists());
    h.close().await;
}

/// A classified native limit blocks an Agent in another Project and provider.
#[tokio::test]
async fn b4_limit_is_daemon_wide() {
    let h = Harness::new("implement").await;
    h.fake.control("exec-output", "{\"type\":\"turn.failed\",\"error\":{\"code\":\"rate_limit_exceeded\",\"message\":\"Rate limit reached\"}}\n");
    h.fake.control("exec-status", "1\n");
    h.started().await.wait().await.unwrap();
    assert_eq!(h.step().body["provider_outcome"], "limit");
    assert!(h.step().body["exit"]["issue"].is_null());
    let other = fixture_with_service(
        true,
        true,
        Some(h.f.owner.clone()),
        Some((&h.runtime, &h.f.config)),
    )
    .await;
    let id = other.claim();
    other.process_step(&id, "review");
    let cursor = other.cursor(&id);
    assert_eq!(
        other
            .store()
            .pipeline_begin_launch(
                &other.owner,
                &cursor,
                Some("b4-other-cookie-1234"),
                Some(&rrx_process_tracker::launch_mark().unwrap()),
                &json!({})
            )
            .unwrap(),
        PipelineLaunch::Waiting
    );
    assert_eq!(other.cursor(&id).step_version, cursor.step_version);
    h.close().await;
}

/// An unresolved process in another Project blocks the launch caller globally.
#[tokio::test]
async fn b4_unknown_blocks_caller() {
    let h = Harness::new("implement").await;
    let other = fixture_with_service(
        true,
        true,
        Some(h.f.owner.clone()),
        Some((&h.runtime, &h.f.config)),
    )
    .await;
    let id = other.claim();
    other.unknown(&id);
    assert!(matches!(h.launch().await, LaunchResult::Waiting));
    assert_eq!(h.step().state, "planned");
    assert!(!h.fake.directory.join("exec-entered").exists());
    h.close().await;
}

/// Internal running prepare steps do not consume the daemon's process slot.
#[tokio::test]
async fn b4_prepare_not_counted() {
    let h = Harness::new("implement").await;
    h.fake.control("exec-last", "complete\n");
    let other = fixture_with_service(
        true,
        true,
        Some(h.f.owner.clone()),
        Some((&h.runtime, &h.f.config)),
    )
    .await;
    let id = other.claim();
    let cursor = other.cursor(&id);
    other
        .store()
        .pipeline_begin_launch(&other.owner, &cursor, None, None, &json!({}))
        .unwrap();
    h.started().await.wait().await.unwrap();
    assert_eq!(h.step().body["provider_outcome"], "success");
    h.close().await;
}

/// Internal running release steps do not consume the daemon's process slot.
#[tokio::test]
async fn b4_release_not_counted() {
    let h = Harness::new("implement").await;
    h.fake.control("exec-last", "complete\n");
    let other = fixture_with_service(
        true,
        true,
        Some(h.f.owner.clone()),
        Some((&h.runtime, &h.f.config)),
    )
    .await;
    let id = other.claim();
    other.advance_prepare(&id);
    other.seed_next(&id, &planned(&id, 2, 1, "release"));
    let cursor = other.cursor(&id);
    other
        .store()
        .pipeline_begin_launch(&other.owner, &cursor, None, None, &json!({}))
        .unwrap();
    h.started().await.wait().await.unwrap();
    assert_eq!(h.step().body["provider_outcome"], "success");
    h.close().await;
}

fn exhausted() -> crate::execution::QuotaObservation {
    crate::execution::QuotaObservation {
        provider: "foreign-provider".into(),
        account_key: "fixture-account".into(),
        bucket: "primary".into(),
        window_id: "old".into(),
        status: crate::execution::QuotaStatus::Exhausted,
        used_percent: Some(100.0),
        resets_at: Some(2),
        observed_at: 1,
        source_version: "fixture".into(),
        confirmed_subscription: true,
    }
}

/// An expired native Exhausted observation still holds an Agent before spawn.
#[tokio::test]
async fn b4_native_exhaustion_never_expires() {
    let h = Harness::new("implement").await;
    h.f.store().observe_quota(&exhausted()).unwrap();
    assert!(matches!(h.launch().await, LaunchResult::Held));
    let step = h.step();
    assert_eq!(step.state, "held");
    assert_eq!(step.body["reason"], "native_quota_exhausted");
    assert_eq!(step.body["death_confirmed"], true);
    assert!(!h.fake.directory.join("exec-entered").exists());
    h.close().await;
}

/// A native Available observation permits launch only after valid window replacement.
#[tokio::test]
async fn b4_native_available_replacement() {
    let h = Harness::new("implement").await;
    h.fake.control("exec-last", "complete\n");
    let mut observation = exhausted();
    h.f.store().observe_quota(&observation).unwrap();
    observation.status = crate::execution::QuotaStatus::Available;
    observation.observed_at = 3;
    h.f.store().observe_quota(&observation).unwrap();
    assert!(
        !h.f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .native_exhaustion
            .is_empty()
    );
    observation.window_id = "new".into();
    h.f.store().observe_quota(&observation).unwrap();
    h.started().await.wait().await.unwrap();
    assert_eq!(h.step().body["provider_outcome"], "success");
    h.close().await;
}

async fn pause(h: &Harness) {
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: h.f.owner.instance.clone(),
        epoch: h.f.owner.epoch,
        action: ControlAction::SetGoalLifecycle {
            project: h.f.project,
            goal: h.f.goal,
            expected_goal: 1,
            target: crate::runtime::control::GoalControl::Pause,
            reason: "fixture pause".into(),
        },
    };
    let (socket, _peer) = tokio::net::UnixStream::pair().unwrap();
    assert!(matches!(
        h.runtime.handle_control(&socket, request).await.unwrap(),
        ControlResponse::GoalLifecycleChanged {
            state: GoalState::Paused,
            ..
        }
    ));
}

/// A paused Goal cannot cross the launch transaction's Running precondition.
#[tokio::test]
async fn b4_paused_goal_never_spawns() {
    let h = Harness::new("implement").await;
    pause(&h).await;
    assert!(
        h.runtime
            .launch_pipeline_step(h.f.cursor(&h.id), h.input())
            .await
            .is_err()
    );
    assert_eq!(h.step().state, "planned");
    assert!(!h.fake.directory.join("exec-entered").exists());
    h.close().await;
}

/// Post-step progress leaves an exited limit untouched while its Goal is paused.
#[tokio::test]
async fn b4_post_step_rule_g() {
    let h = Harness::new("implement").await;
    h.fake.control("exec-output", "{\"type\":\"turn.failed\",\"error\":{\"code\":\"rate_limit_exceeded\",\"message\":\"Rate limit reached\"}}\n");
    h.started().await.wait().await.unwrap();
    let before = h.f.cursor(&h.id);
    pause(&h).await;
    assert!(h.runtime.record_pipeline_limit(&before).await.is_err());
    assert_eq!(h.step().state, "exited");
    assert_eq!(h.f.cursor(&h.id).step_version, before.step_version);
    h.close().await;
}

/// A due global wait clears only after its single native probe is classified successful.
#[tokio::test]
async fn b4_probe_known_success() {
    use std::future::Future;
    let h = Harness::new("review").await;
    h.fake.control("exec-output", "{\"type\":\"result\",\"is_error\":true,\"api_error\":{\"status\":429},\"result\":\"Rate limit exceeded\"}");
    h.fake.control("exec-status", "1\n");
    h.started().await.wait().await.unwrap();
    let cursor = h.f.cursor(&h.id);
    assert!(h.runtime.record_pipeline_limit(&cursor).await.unwrap());
    let cursor = h.f.cursor(&h.id);
    let retry = h.review_retry();
    h.f.store()
        .pipeline_plan_next(&h.f.owner, &cursor, &retry, true)
        .unwrap();
    let wait =
        h.f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .provider_wait
            .unwrap();
    let attempt =
        h.f.config
            .pipeline
            .as_ref()
            .unwrap()
            .work_root
            .join("runs")
            .join(&h.id)
            .join("2-2");
    let snapshot = crate::execution::pipeline::review_snapshot::snapshot_path(
        &h.f.config.pipeline.as_ref().unwrap().work_root,
        &h.id,
        2,
        2,
    )
    .unwrap();
    let args = step_arguments(
        PipelineRole::Review,
        &h.fake.cli,
        &snapshot,
        &attempt,
        crate::execution::pipeline::review::REVIEW_SCHEMA,
    )
    .unwrap();
    h.fake
        .expect_arguments(&args.iter().map(String::as_str).collect::<Vec<_>>());
    h.fake.control("exec-status", "0\n");
    h.fake.control(
        "exec-output",
        "{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"structured_output\":{}}",
    );
    let result = {
        let launch = h.launch();
        tokio::pin!(launch);
        std::future::poll_fn(|cx| with_pipeline_clock(wait.not_before, || launch.as_mut().poll(cx)))
            .await
    };
    let LaunchResult::Started(custody) = result else {
        panic!("due probe did not start")
    };
    custody.wait().await.unwrap();
    let wait =
        h.f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .provider_wait
            .unwrap();
    assert_eq!(wait.state, "clear");
    assert_eq!(h.step().body["provider_outcome"], "success");
    h.close().await;
}

async fn wait_for_file(path: &std::path::Path) {
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while !path.exists() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("native fixture checkpoint not reached");
}

/// A failed output save settles a due probe once and atomically holds its run.
#[tokio::test]
async fn b4_probe_output_io_atomic() {
    use std::future::Future;
    let h = Harness::new("review").await;
    h.fake.control("exec-output", "{\"type\":\"result\",\"is_error\":true,\"api_error\":{\"status\":429},\"result\":\"Rate limit exceeded\"}");
    h.fake.control("exec-status", "1\n");
    h.started().await.wait().await.unwrap();
    assert!(
        h.runtime
            .record_pipeline_limit(&h.f.cursor(&h.id))
            .await
            .unwrap()
    );
    let retry = h.review_retry();
    let cursor = h.f.cursor(&h.id);
    h.f.store()
        .pipeline_plan_next(&h.f.owner, &cursor, &retry, true)
        .unwrap();
    let wait =
        h.f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .provider_wait
            .unwrap();
    let attempt =
        h.f.config
            .pipeline
            .as_ref()
            .unwrap()
            .work_root
            .join("runs")
            .join(&h.id)
            .join("2-2");
    let snapshot = crate::execution::pipeline::review_snapshot::snapshot_path(
        &h.f.config.pipeline.as_ref().unwrap().work_root,
        &h.id,
        2,
        2,
    )
    .unwrap();
    let args = step_arguments(
        PipelineRole::Review,
        &h.fake.cli,
        &snapshot,
        &attempt,
        crate::execution::pipeline::review::REVIEW_SCHEMA,
    )
    .unwrap();
    h.fake
        .expect_arguments(&args.iter().map(String::as_str).collect::<Vec<_>>());
    h.fake.control("exec-pause", "");
    h.fake.control("exec-status", "0\n");
    h.fake.control(
        "exec-output",
        "{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"structured_output\":{}}",
    );
    let result = {
        let launch = h.launch();
        tokio::pin!(launch);
        std::future::poll_fn(|cx| with_pipeline_clock(wait.not_before, || launch.as_mut().poll(cx)))
            .await
    };
    let LaunchResult::Started(custody) = result else {
        panic!("due probe did not start")
    };
    let probing =
        h.f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .provider_wait
            .unwrap();
    assert_eq!(probing.state, "probing");
    // A directory at the retention leaf forces create_new to fail after death.
    std::fs::create_dir(attempt.join("stdout")).unwrap();
    h.fake.control("exec-release", "");
    custody.wait().await.unwrap();
    let after =
        h.f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .provider_wait
            .unwrap();
    assert_eq!(after.version, probing.version + 1);
    assert_eq!(after.state, "waiting");
    assert_eq!(after.backoff_ms, (probing.backoff_ms * 2).min(1800000));
    assert!(after.not_before > probing.not_before);
    let step = h.step();
    assert_eq!(step.state, "rejected");
    assert_eq!(step.body["reason"], "output_io");
    assert_eq!(step.body["exit"]["issue"], "output_io");
    assert_eq!(
        h.f.store()
            .pipeline_task_view(h.f.goal, h.f.task)
            .unwrap()
            .runs
            .last()
            .unwrap()
            .state,
        "held"
    );
    h.close().await;
}

/// A failed final classification rolls back the exit observation and wait settlement.
#[tokio::test]
async fn b4_exit_classification_rollback() {
    for output_save_failure in [false, true] {
        let h = Harness::new("implement").await;
        h.fake.control("exec-pause", "");
        let custody = h.started().await;
        if output_save_failure {
            let attempt =
                h.f.config
                    .pipeline
                    .as_ref()
                    .unwrap()
                    .work_root
                    .join("runs")
                    .join(&h.id)
                    .join("2-1");
            std::fs::create_dir(attempt.join("stdout")).unwrap();
        }
        h.f.store().connection.execute_batch("CREATE TEMP TRIGGER b4_fail_classification BEFORE UPDATE ON pipeline_steps WHEN NEW.state IN ('rejected','interrupted') BEGIN SELECT RAISE(ABORT,'injected classification failure'); END;").unwrap();
        custody.request_stop();
        custody.wait().await.unwrap(); // The supervisor can record unknown after rollback.
        let step = h.step();
        assert_eq!(step.state, "unknown");
        assert!(step.body["exit"].is_null());
        assert!(step.body["provider_outcome"].is_null());
        assert_eq!(step.body["death_confirmed"], false);
        h.close().await;
    }
}

/// An unreaped group member prevents exit proof even with no live cookie matches.
#[tokio::test]
async fn b4_group_not_empty_holds() {
    let h = Harness::new("implement").await;
    h.fake.control("exec-pause", "");
    let custody = h.started().await;
    let pid = h.step().pid.unwrap();
    let mut command = tokio::process::Command::new("/bin/sleep");
    command
        .arg("30")
        .env_clear()
        .process_group(pid as i32)
        .kill_on_drop(true)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    // Keep this direct child's handle unreaped until the monitor has refused proof.
    let mut member = rrx_spawn_hygiene::spawn_tokio(command).unwrap();
    assert_eq!(
        rustix::process::getpgid(rustix::process::Pid::from_raw(member.id().unwrap() as i32))
            .unwrap()
            .as_raw_nonzero()
            .get() as u32,
        pid
    );
    custody.request_stop();
    let result = custody.wait().await;
    let group_observation = rustix::process::test_kill_process_group(
        rustix::process::Pid::from_raw(pid as i32).unwrap(),
    );
    let member_status = tokio::time::timeout(std::time::Duration::from_secs(10), member.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(!member_status.success());
    assert_ne!(group_observation, Err(rustix::io::Errno::SRCH));
    result.unwrap();
    let step = h.step();
    assert_eq!(step.state, "held");
    assert_eq!(step.body["reason"], "death_unconfirmed");
    assert_eq!(step.body["death_confirmed"], false);
    assert!(step.body["exit"].is_null());
    h.close().await;
}

/// Pause waits across tx1/spawn/tx2, but can finish while stdin is blocked.
#[tokio::test(flavor = "current_thread")]
async fn b4_control_waits_for_launch_boundary() {
    use crate::execution::pipeline::test_support::{LaunchPhase, pause_launch};
    let h = Harness::with_prompt("implement", &vec![b'p'; 512 * 1024]).await;
    h.fake.control("exec-pause", "");
    let mut checkpoint = Some(pause_launch(LaunchPhase::Intent));
    let launch = h.launch();
    tokio::pin!(launch);
    let control = pause(&h);
    tokio::pin!(control);
    for (phase, next) in [
        (LaunchPhase::Intent, Some(LaunchPhase::Verified)),
        (LaunchPhase::Verified, Some(LaunchPhase::Recorded)),
        (LaunchPhase::Recorded, None),
    ] {
        let (release, arrived) = checkpoint.take().unwrap();
        tokio::select! {
            _ = &mut launch => panic!("launch bypassed checkpoint"),
            arrival = arrived => arrival.unwrap(),
        }
        assert_eq!(
            h.step().state,
            if phase == LaunchPhase::Recorded {
                "running"
            } else {
                "launching"
            }
        );
        if phase != LaunchPhase::Recorded {
            assert!(!h.fake.directory.join("exec-entered").exists());
        }
        assert!(futures_util::poll!(&mut control).is_pending());
        checkpoint = next.map(pause_launch);
        release.send(()).unwrap();
    }
    let (launched, state_at_pause) = tokio::join!(launch, async {
        tokio::time::timeout(std::time::Duration::from_secs(2), &mut control)
            .await
            .unwrap();
        h.step().state
    });
    let LaunchResult::Started(custody) = launched else {
        panic!("launch did not start")
    };
    wait_for_file(&h.fake.directory.join("exec-entered")).await;
    // The fake has not read stdin; a 512 KiB write cannot fit into its pipe.
    assert!(!h.fake.directory.join("stdin").exists());
    assert_eq!(state_at_pause, "running");
    custody.request_stop();
    custody.wait().await.unwrap();
    h.runtime.shutdown().await.unwrap();
}

/// Spawn uses the last verified image even if its configured symlink changes.
#[tokio::test(flavor = "current_thread")]
async fn b4_spawn_verified_realpath() {
    use crate::execution::pipeline::test_support::{LaunchPhase, pause_launch};
    use std::os::unix::fs::symlink;
    let h = Harness::with_alias("implement", b"pipeline\n", true).await;
    h.fake.control("exec-last", "complete\n");
    let alias = &h.fake.cli.executable;
    let (release, arrived) = pause_launch(LaunchPhase::Verified);
    let launch = h.runtime.launch_pipeline_step(h.f.cursor(&h.id), h.input());
    tokio::pin!(launch);
    tokio::select! {
        _ = &mut launch => panic!("launch bypassed Verified checkpoint"),
        arrival = arrived => arrival.unwrap(),
    }
    std::fs::remove_file(alias).unwrap();
    symlink("/usr/bin/false", alias).unwrap();
    release.send(()).unwrap();
    let LaunchResult::Started(custody) = launch.await.unwrap() else {
        panic!("verified image did not start")
    };
    custody.wait().await.unwrap();
    assert_eq!(h.step().body["provider_outcome"], "success");
    assert!(h.fake.directory.join("exec-entered").exists());
    h.runtime.shutdown().await.unwrap();
}

/// Executor custody wires last validation into outcomes, issues, and receipts.
#[tokio::test]
async fn f_w4_executor_last_output() {
    const SUCCESS: &str = "{\"type\":\"turn.started\"}\n{\"type\":\"turn.completed\"}\n";
    const LIMIT: &str = "{\"type\":\"turn.failed\",\"error\":{\"code\":\"rate_limit_exceeded\",\"message\":\"Rate limit reached\"}}\n";
    const OTHER: &str = "{\"type\":\"unknown.event\"}\n";
    for (output, last, outcome, issue) in [
        (SUCCESS, "valid", "success", None),
        (SUCCESS, "missing", "other", Some("last_output_invalid")),
        (LIMIT, "symlink", "other", Some("last_output_invalid")),
        (LIMIT, "missing", "limit", None),
        (OTHER, "missing", "other", None),
    ] {
        let h = Harness::build("implement", b"pipeline\n", false, Some("codex-cli 0.164.0")).await;
        h.fake.control("exec-output", output);
        h.fake.control("exec-pause", "");
        let custody = h.started().await;
        let root =
            h.f.config
                .pipeline
                .as_ref()
                .unwrap()
                .work_root
                .join("runs")
                .join(&h.id)
                .join("2-1");
        let path = root.join("last.md");
        match last {
            "missing" => {}
            "valid" => std::fs::write(&path, b"complete").unwrap(),
            "symlink" => {
                let target = root.join("target");
                std::fs::write(&target, b"complete").unwrap();
                std::os::unix::fs::symlink(&target, &path).unwrap();
            }
            _ => unreachable!(),
        }
        h.fake.control("exec-release", "");
        custody.wait().await.unwrap();
        let step = h.step();
        assert_eq!(step.body["provider_outcome"], outcome, "{last}");
        assert_eq!(
            step.body["exit"]["issue"],
            issue.map_or(Value::Null, |issue| json!(issue))
        );
        assert_eq!(step.body["exit"]["last"].is_object(), last == "valid");
        h.close().await;
    }
}

/// F-w4/F-a7: Missing Executor last output cannot clear a due daemon probe despite successful JSONL.
#[tokio::test]
async fn f_w4_executor_missing_last_extends_probe() {
    use std::future::Future;
    let h = Harness::build("implement", b"pipeline\n", false, Some("codex-cli 0.164.0")).await;
    h.fake.control("exec-output", "{\"type\":\"turn.failed\",\"error\":{\"code\":\"rate_limit_exceeded\",\"message\":\"Rate limit reached\"}}\n");
    h.started().await.wait().await.unwrap();
    assert!(
        h.runtime
            .record_pipeline_limit(&h.f.cursor(&h.id))
            .await
            .unwrap()
    );
    let retry = planned(&h.id, 2, 2, "implement");
    let mut retry = retry;
    retry.input_sha256 = digest(&h.prompt);
    let cursor = h.f.cursor(&h.id);
    h.f.store()
        .pipeline_plan_next(&h.f.owner, &cursor, &retry, true)
        .unwrap();
    let wait =
        h.f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .provider_wait
            .unwrap();
    let root =
        h.f.config
            .pipeline
            .as_ref()
            .unwrap()
            .work_root
            .join("runs")
            .join(&h.id)
            .join("2-2");
    let args = step_arguments(PipelineRole::Implement, &h.fake.cli, &h.work, &root, "{}").unwrap();
    h.fake
        .expect_arguments(&args.iter().map(String::as_str).collect::<Vec<_>>());
    h.fake.control(
        "exec-output",
        "{\"type\":\"turn.started\"}\n{\"type\":\"turn.completed\"}\n",
    );
    let result = {
        let launch = h.launch();
        tokio::pin!(launch);
        std::future::poll_fn(|cx| with_pipeline_clock(wait.not_before, || launch.as_mut().poll(cx)))
            .await
    };
    let LaunchResult::Started(custody) = result else {
        panic!("probe did not launch");
    };
    custody.wait().await.unwrap();
    let after =
        h.f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .provider_wait
            .unwrap();
    assert_ne!(after.state, "clear");
    assert!(after.not_before > wait.not_before);
    assert_eq!(h.step().body["provider_outcome"], "other");
    assert_eq!(h.step().body["exit"]["issue"], "last_output_invalid");
    h.close().await;
}

/// F-w6: Unmapped pins retain fixed exit evidence while their successful wire shape stays other.
#[tokio::test]
async fn f_w6_unmapped_exit_evidence() {
    let h = Harness::build("implement", b"pipeline\n", false, Some("codex-cli unknown")).await;
    h.fake.control("exec-last", "complete\n");
    h.started().await.wait().await.unwrap();
    assert_eq!(h.step().body["provider_outcome"], "other");
    assert_eq!(h.step().body["exit"]["wire"], "unmapped");
    h.close().await;
}
