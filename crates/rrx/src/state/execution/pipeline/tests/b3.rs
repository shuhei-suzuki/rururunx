use super::*;
use crate::config::ExecutionMode;

fn refuses_pipeline(legacy: &crate::runtime::LegacyFixture, store: &Store) {
    let before = store.pipeline_mode_inventory().unwrap();
    let epoch = store.connection_epoch_for_test();
    let error =
        RuntimeOwner::open_mode(&legacy.path().join("state.db"), ExecutionMode::CliPipeline)
            .err()
            .unwrap();
    assert!(
        error
            .downcast_ref::<crate::execution::owner::ExecutionModeNotQuiescent>()
            .is_some(),
        "{error:#}"
    );
    assert_eq!(store.connection_epoch_for_test(), epoch);
    assert_eq!(store.pipeline_mode_inventory().unwrap(), before);
}

/// An open native Unit is detected through its normal reservation and blocks pipeline startup.
#[test]
fn b3_owner_native_unit() {
    let (legacy, store, unit) = round2::native_open_inventory_fixture();
    assert!(unit.native_effects_open && unit.result_finalization_open);
    assert_eq!(store.pipeline_mode_inventory().unwrap().native_units, 1);
    let before = store.execution_unit(unit.id).unwrap();
    refuses_pipeline(&legacy, &store);
    assert_eq!(
        serde_json::to_value(store.execution_unit(unit.id).unwrap()).unwrap(),
        serde_json::to_value(before).unwrap()
    );
}

/// A real active native quota lease is inventoried and retained on pipeline startup refusal.
#[test]
fn b3_owner_native_lease() {
    let (legacy, mut store, unit) = round2::native_open_inventory_fixture();
    store
        .reserve_execution_quota(&unit.authority(), "codex", "unknown", 6, 2, 3, now_ms())
        .unwrap();
    assert_eq!(store.pipeline_mode_inventory().unwrap().native_leases, 1);
    let leases: Vec<(String, bool)> = store
        .connection
        .prepare("SELECT unit_id,active FROM quota_leases ORDER BY unit_id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    refuses_pipeline(&legacy, &store);
    let after: Vec<(String, bool)> = store
        .connection
        .prepare("SELECT unit_id,active FROM quota_leases ORDER BY unit_id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(after, leases);
}

fn native_effect(unknown: bool) {
    let (legacy, mut store, unit) = round2::native_open_inventory_fixture();
    let effect = ManagedEffect {
        id: OperationId::new(),
        unit_id: unit.id,
        scope: unit.scope.clone(),
        kind: "publish".into(),
        idempotency_key: "publish-once".into(),
        expected_target: "fixture-target".into(),
        state: EffectState::Pending,
        receipt: BTreeMap::new(),
        version: 1,
    };
    store
        .reserve_managed_effect(&unit.authority(), &effect)
        .unwrap();
    if unknown {
        store
            .reconcile_managed_effect(effect.id, 1, EffectState::Unknown, BTreeMap::new())
            .unwrap();
    }
    let before = store.managed_effect(effect.id).unwrap();
    assert_eq!(
        before.state,
        if unknown {
            EffectState::Unknown
        } else {
            EffectState::Pending
        }
    );
    assert_eq!(store.pipeline_mode_inventory().unwrap().native_effects, 1);
    refuses_pipeline(&legacy, &store);
    assert_eq!(
        serde_json::to_value(store.managed_effect(effect.id).unwrap()).unwrap(),
        serde_json::to_value(before).unwrap()
    );
}

/// A pending native effect is detected from its real reservation and blocks pipeline startup.
#[test]
fn b3_owner_native_pending_effect() {
    native_effect(false);
}

/// An unknown native effect is detected from reconciliation and blocks pipeline startup.
#[test]
fn b3_owner_native_unknown_effect() {
    native_effect(true);
}

/// A native owner cannot reserve pipeline work even for an otherwise accepted Goal.
#[tokio::test]
async fn b3_native_owner_claim_refused() {
    let accepted = fixture().await;
    accepted.claim();
    let f = fixture_mode(false).await;
    let id = Uuid::new_v4().to_string();
    let run = PipelineRun {
        id: id.clone(),
        kind: "task".into(),
        task_id: Some(f.task),
        goal_id: f.goal,
        project_id: f.project,
        owner_epoch: f.owner.epoch(),
        version: 0,
        state: "running".into(),
        round: 0,
        base_commit: "a".repeat(40),
        head_commit: "a".repeat(40),
        worktree: Some(format!("/pipeline/{id}")),
        branch: Some(format!("rrx/pipeline/{id}")),
        body: json!({}),
    };
    let error = f
        .store()
        .pipeline_claim_run(
            &f.owner,
            &run,
            1,
            &planned(&id, 1, 1, "prepare"),
            Some(&create_journal(&f)),
        )
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("pipeline requires CliPipeline owner"),
        "{error:#}"
    );
    assert!(
        f.store()
            .pipeline_goal_view(f.goal)
            .unwrap()
            .runs
            .is_empty()
    );
}

/// A native owner cannot write launch intent through the pipeline Store API.
#[tokio::test]
async fn b3_native_owner_launch_refused() {
    let accepted = fixture().await;
    let id = accepted.claim();
    let cursor = accepted.cursor(&id);
    accepted
        .store()
        .pipeline_begin_launch(&accepted.owner, &cursor, None, &json!({}))
        .unwrap();
    let f = fixture_mode(false).await;
    let before = f.store().pipeline_goal_view(f.goal).unwrap();
    let error = f
        .store()
        .pipeline_begin_launch(&f.owner, &cursor, None, &json!({}))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("pipeline requires CliPipeline owner"),
        "{error:#}"
    );
    assert_eq!(f.store().pipeline_goal_view(f.goal).unwrap(), before);
}

async fn refuses_native(f: Fixture) {
    let path = f.owner.state_path().to_path_buf();
    let before = f.store().pipeline_mode_inventory().unwrap();
    let epoch = f.owner.epoch();
    let retained = std::sync::Arc::downgrade(&f.owner);
    drop(f.ingress);
    drop(f.owner);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    while retained.upgrade().is_some() {
        assert!(tokio::time::Instant::now() < deadline, "owner retained");
        tokio::task::yield_now().await;
    }
    let error = RuntimeOwner::open(&path)
        .err()
        .expect("non-quiescent native startup");
    assert!(
        error
            .downcast_ref::<crate::execution::owner::ExecutionModeNotQuiescent>()
            .is_some()
    );
    let store = Store::open(&path).unwrap();
    assert_eq!(store.connection_epoch_for_test(), epoch);
    assert_eq!(store.pipeline_mode_inventory().unwrap(), before);
}

/// A held pipeline run prevents native startup without advancing epoch or changing its evidence.
#[tokio::test]
async fn b3_owner_held_run() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_hold(&f.owner, &cursor, "operator", None)
        .unwrap();
    assert_eq!(
        f.store().pipeline_goal_view(f.goal).unwrap().runs[0].state,
        "held"
    );
    refuses_native(f).await;
}
/// An unknown process step prevents native startup without using the next epoch as death proof.
#[tokio::test]
async fn b3_owner_unknown_step() {
    let f = fixture().await;
    let id = f.claim();
    f.unknown(&id);
    assert_eq!(
        f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .pipeline_processes,
        1
    );
    refuses_native(f).await;
}
/// An open pipeline ref journal prevents native startup and remains unsettled.
#[tokio::test]
async fn b3_owner_open_journal() {
    let f = fixture().await;
    f.claim();
    assert_eq!(
        f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .pipeline_journals,
        1
    );
    refuses_native(f).await;
}
/// An actual provider-limit wait survives cancelled runs and still prevents native startup.
#[tokio::test]
async fn b3_owner_provider_wait() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(&f.owner, &cursor, &death(), Some("limit"), &json!({}))
        .unwrap();
    fix1::cancel(&f);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_cancel_run(&f.owner, &cursor, None)
        .unwrap();
    let inventory = f.store().pipeline_mode_inventory().unwrap();
    assert_eq!(
        inventory.pipeline_runs + inventory.pipeline_journals + inventory.pipeline_processes,
        0
    );
    assert_eq!(inventory.provider_wait.unwrap().state, "waiting");
    refuses_native(f).await;
}

/// An expired real provider wait still blocks a mode change without changing its evidence.
#[tokio::test]
async fn b3_owner_expired_wait() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    let cursor = f.cursor(&id);
    with_pipeline_clock(0, || {
        f.store()
            .pipeline_record_exit(&f.owner, &cursor, &death(), Some("limit"), &json!({}))
            .unwrap()
    });
    fix1::cancel(&f);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_cancel_run(&f.owner, &cursor, None)
        .unwrap();
    let inventory = f.store().pipeline_mode_inventory().unwrap();
    assert_eq!(
        inventory.pipeline_runs + inventory.pipeline_journals + inventory.pipeline_processes,
        0
    );
    let wait = inventory.provider_wait.unwrap();
    assert_eq!(wait.state, "waiting");
    assert!(wait.not_before < now_ms());
    refuses_native(f).await;
}

/// A real designated limit probe blocks a mode change and retains its exact wait state.
#[tokio::test]
async fn b3_owner_probing_wait() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    let cursor = f.cursor(&id);
    with_pipeline_clock(0, || {
        f.store()
            .pipeline_record_exit(&f.owner, &cursor, &death(), Some("limit"), &json!({}))
            .unwrap()
    });
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_hold(&f.owner, &cursor, "limit", None)
        .unwrap();
    let request = recovery(&f, &id, "retry");
    f.store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
        .unwrap();
    let cursor = f.cursor(&id);
    assert!(matches!(
        f.store()
            .pipeline_begin_launch(&f.owner, &cursor, Some("probe-cookie-012345"), &json!({}))
            .unwrap(),
        PipelineLaunch::Started
    ));
    assert_eq!(
        f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .provider_wait
            .unwrap()
            .state,
        "probing"
    );
    refuses_native(f).await;
}
/// Expired native exhaustion across any account prevents pipeline startup before epoch retirement.
#[test]
fn b3_owner_native_exhaustion() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    let owner = RuntimeOwner::open(&path).unwrap();
    let epoch = owner.epoch();
    owner
        .store
        .lock()
        .unwrap()
        .observe_quota(&crate::execution::QuotaObservation {
            provider: "codex".into(),
            account_key: "fixture-account".into(),
            bucket: "all".into(),
            window_id: "fixture-window".into(),
            status: crate::execution::QuotaStatus::Exhausted,
            used_percent: Some(100.0),
            resets_at: Some(0),
            observed_at: now_ms(),
            source_version: "fixture".into(),
            confirmed_subscription: true,
        })
        .unwrap();
    let before = owner
        .store
        .lock()
        .unwrap()
        .pipeline_mode_inventory()
        .unwrap();
    drop(owner);
    let error = RuntimeOwner::open_mode(&path, ExecutionMode::CliPipeline)
        .err()
        .expect("exhaustion prevents startup");
    assert_eq!(
        error
            .downcast_ref::<crate::execution::owner::ExecutionModeNotQuiescent>()
            .unwrap()
            .reason,
        "native_quota_exhausted"
    );
    let store = Store::open(&path).unwrap();
    assert_eq!(store.connection_epoch_for_test(), epoch);
    assert_eq!(store.pipeline_mode_inventory().unwrap(), before);
}

/// A retained native cleanup cookie with a live process blocks pipeline startup without writes.
#[tokio::test]
async fn b3_owner_native_cookie_process() {
    // Re-exec this test binary so readiness follows process initialization,
    // with the cookie in the initial environment read by KERN_PROCARGS2.
    if let Some(marker) = std::env::var_os("RRX_B3_COOKIE_WORKER_READY") {
        std::fs::write(marker, b"ready").unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        return;
    }
    let (legacy, store, unit) = round2::native_inventory_fixture();
    let path = legacy.path().join("state.db");
    let epoch = store.connection_epoch_for_test();
    let before = store.pipeline_mode_inventory().unwrap();
    let marker = legacy.path().join("cookie-worker-ready");
    let mut command = tokio::process::Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "state::execution::pipeline::tests::b3::b3_owner_native_cookie_process",
        ])
        .env("RRX_PROCESS_COOKIE", &unit.cookie)
        .env("RRX_B3_COOKIE_WORKER_READY", &marker)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let mut child = crate::execution::process::OwnedProcess::spawn(command).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while !marker.exists() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let found = rrx_process_tracker::discover(
        &rrx_process_tracker::Cookie::new(&unit.cookie).unwrap(),
        Default::default(),
    )
    .unwrap();
    let matched = found
        .processes
        .iter()
        .any(|p| Some(p.identity().pid) == child.child.id());
    let running = rustix::process::waitid(
        rustix::process::WaitId::Pid(
            rustix::process::Pid::from_raw(child.child.id().unwrap() as i32).unwrap(),
        ),
        rustix::process::WaitIdOptions::EXITED
            | rustix::process::WaitIdOptions::NOWAIT
            | rustix::process::WaitIdOptions::NOHANG,
    )
    .unwrap()
    .is_none();
    let refused = RuntimeOwner::open_mode(&path, ExecutionMode::CliPipeline).is_err();
    child.stop_and_reap().await.unwrap();
    assert!(running, "fixture process exited before discovery");
    assert!(
        matched,
        "fixture process cookie was not visible: {:?}",
        found.coverage
    );
    assert!(refused);
    assert_eq!(store.connection_epoch_for_test(), epoch);
    assert_eq!(store.pipeline_mode_inventory().unwrap(), before);
}

/// Native restart does not turn its own retained cookie decode into an opposite-mode gate.
#[test]
fn b3_owner_native_own_recovery() {
    let (legacy, store, unit) = round2::native_inventory_fixture();
    let epoch = store.connection_epoch_for_test();
    // Corruption is a refusal stimulus, never an authority or success seed.
    store
        .connection
        .execute(
            "UPDATE execution_units SET body=json_set(body,'$.cookie','malformed') WHERE id=?1",
            [unit.id.to_string()],
        )
        .unwrap();
    assert!(store.pipeline_mode_inventory().is_err());
    let owner = RuntimeOwner::open(&legacy.path().join("state.db")).unwrap();
    assert_eq!(owner.epoch(), epoch + 1);
}

/// Pipeline claims cannot reserve a run before all startup Q subjects are current.
#[tokio::test]
async fn b3_q_blocks_claim() {
    let f = fixture().await;
    let id = Uuid::new_v4().to_string();
    // Nongrant proposal over a genuinely accepted Goal, through the normal API.
    let mut run = PipelineRun {
        id: id.clone(),
        kind: "task".into(),
        task_id: Some(f.task),
        goal_id: f.goal,
        project_id: f.project,
        owner_epoch: 0,
        version: 0,
        state: "running".into(),
        round: 0,
        base_commit: "a".repeat(40),
        head_commit: "a".repeat(40),
        worktree: Some(format!("/pipeline/{id}")),
        branch: Some(format!("rrx/pipeline/{id}")),
        body: json!({}),
    };
    let create = create_journal(&f);
    let path = f.owner.state_path().to_path_buf();
    let old = std::sync::Arc::downgrade(&f.owner);
    drop(f.ingress);
    drop(f.owner);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    while old.upgrade().is_some() {
        assert!(tokio::time::Instant::now() < deadline);
        tokio::task::yield_now().await;
    }
    let owner = RuntimeOwner::open_mode(&path, ExecutionMode::CliPipeline).unwrap();
    run.owner_epoch = owner.epoch();
    let mut store = owner.store.lock().unwrap();
    assert!(
        store
            .pipeline_claim_run(
                &owner,
                &run,
                1,
                &planned(&id, 1, 1, "prepare"),
                Some(&create)
            )
            .unwrap_err()
            .to_string()
            .contains("pipeline startup Q is not current")
    );
    assert!(store.pipeline_goal_view(f.goal).unwrap().runs.is_empty());
}

/// Even internal pipeline work cannot begin with missing startup Q after takeover.
#[tokio::test]
async fn b3_q_blocks_internal_launch() {
    let f = fixture().await;
    let id = f.claim();
    let path = f.owner.state_path().to_path_buf();
    let old = std::sync::Arc::downgrade(&f.owner);
    drop(f.ingress);
    drop(f.owner);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    while old.upgrade().is_some() {
        assert!(tokio::time::Instant::now() < deadline);
        tokio::task::yield_now().await;
    }
    let owner = RuntimeOwner::open_mode(&path, ExecutionMode::CliPipeline).unwrap();
    let mut store = owner.store.lock().unwrap();
    let takeover = store.pipeline_takeover_view().unwrap();
    store.pipeline_take_over(&owner, &takeover).unwrap();
    let before = store.pipeline_goal_view(f.goal).unwrap();
    let cursor = PipelineCursor {
        run_id: id,
        run_version: before.runs[0].version,
        seq: 1,
        attempt: 1,
        step_version: before.steps[0].version,
        wait_version: None,
    };
    assert!(
        store
            .pipeline_begin_launch(&owner, &cursor, None, &json!({}))
            .unwrap_err()
            .to_string()
            .contains("pipeline startup Q is not current")
    );
    assert_eq!(store.pipeline_goal_view(f.goal).unwrap(), before);
}
