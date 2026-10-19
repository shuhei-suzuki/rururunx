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
        let f = fixture().await;
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
                "{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false}"
            } else {
                "{\"type\":\"turn.completed\"}\n"
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
            schema: "{}".into(),
        }
    }
    async fn launch(&self) -> LaunchResult {
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
    h.started().await.wait().await.unwrap();
    let step = h.step();
    assert_eq!(step.state, "exited");
    assert_eq!(step.body["pgid"], step.pid.unwrap());
    assert!(step.birth.as_ref().is_some_and(|b| b.contains(':')));
    assert_eq!(step.body["provider_outcome"], "success");
    let proof: PipelineDeathEvidence =
        serde_json::from_value(step.body["death_evidence"].clone()).unwrap();
    proof.validate().unwrap();
    assert_eq!(
        std::fs::read(h.fake.directory.join("stdin")).unwrap(),
        b"pipeline\n"
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
    h.fake.control("exec-pre-output", &" ".repeat(256 * 1024));
    h.started().await.wait().await.unwrap();
    assert_eq!(h.step().body["provider_outcome"], "success");
    assert_eq!(
        h.step().body["exit"]["stdout"]["bytes"],
        256 * 1024 + b"{\"type\":\"turn.completed\"}\n".len()
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

/// A failed running-record transaction cannot reject an already spawned attempt.
#[tokio::test]
async fn b4_record_failure_death() {
    let h = Harness::new("implement").await;
    h.f.store().connection.execute_batch("CREATE TEMP TRIGGER b4_fail_running BEFORE UPDATE ON pipeline_steps WHEN NEW.state='running' BEGIN SELECT RAISE(ABORT,'injected running failure'); END;").unwrap();
    let _result = h.launch().await;
    let step = h.step();
    assert_eq!(step.state, "exited");
    assert_eq!(step.body["exit"], "killed_after_record_failure");
    assert_eq!(step.body["death_confirmed"], true);
    h.close().await;
}

/// A spawn error remains unknown even when the executable's qualified bytes match.
#[tokio::test]
async fn b4_spawn_error_is_unknown() {
    let h = Harness::new("review").await;
    let mut input = h.input();
    // Exceeds macOS ARG_MAX without changing Q, filesystem identity or authority.
    input.schema = "x".repeat(2 * 1024 * 1024);
    let result = h
        .runtime
        .launch_pipeline_step(h.f.cursor(&h.id), input)
        .await
        .unwrap();
    assert!(matches!(result, LaunchResult::Unknown));
    assert_eq!(h.step().state, "unknown");
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
        .pipeline_begin_launch(&other.owner, &cursor, None, &json!({}))
        .unwrap();
    h.started().await.wait().await.unwrap();
    assert_eq!(h.step().body["provider_outcome"], "success");
    h.close().await;
}

/// Internal running release steps do not consume the daemon's process slot.
#[tokio::test]
async fn b4_release_not_counted() {
    let h = Harness::new("implement").await;
    let other = fixture_with_service(
        true,
        true,
        Some(h.f.owner.clone()),
        Some((&h.runtime, &h.f.config)),
    )
    .await;
    let id = other.claim();
    other.advance_prepare(&id);
    let cursor = other.cursor(&id);
    other
        .store()
        .pipeline_plan_next(&other.owner, &cursor, &planned(&id, 2, 1, "release"), true)
        .unwrap();
    let cursor = other.cursor(&id);
    other
        .store()
        .pipeline_begin_launch(&other.owner, &cursor, None, &json!({}))
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
    let mut retry = planned(&h.id, 2, 2, "review");
    retry.input_sha256 = digest(b"pipeline\n");
    assert_eq!(std::fs::read_dir(&h.work).unwrap().count(), 0);
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
    let args = step_arguments(PipelineRole::Review, &h.fake.cli, &h.work, &attempt, "{}").unwrap();
    h.fake
        .expect_arguments(&args.iter().map(String::as_str).collect::<Vec<_>>());
    h.fake.control("exec-status", "0\n");
    h.fake.control(
        "exec-output",
        "{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false}",
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
