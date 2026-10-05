use super::*;
use crate::adapter::InputKind;
use std::{os::unix::fs::PermissionsExt, path::Path};

pub(crate) fn program(root: &Path, provider: &str) -> std::path::PathBuf {
    let path = root.join(format!("fixture-{provider}"));
    let script = format!("#!/usr/bin/python3\nPROVIDER = {provider:?}\n");
    std::fs::write(
        &path,
        format!("{script}{}", include_str!("native_fixture.py")),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}
pub(crate) fn input(unit: &ExecutionUnit, payload: &str) -> ManagedInput {
    ManagedInput {
        agent: unit.provider.clone(),
        authority: unit.authority(),
        artifact: unit.artifact_id,
        input: PreparedInput {
            scope: unit.scope.clone(),
            kind: InputKind::ContextPack,
            revision: unit.base_sha.clone(),
            version: 1,
            source_versions: BTreeMap::new(),
            payload: payload.into(),
        },
    }
}
async fn terminal(sessions: &NativeSessions, handle: &ManagedSessionRef) -> NativeStatus {
    let mut updates = sessions.subscribe(handle).unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let status = updates.borrow().clone();
            if status.work.is_some() {
                break status;
            }
            updates.changed().await.unwrap();
        }
    })
    .await
    .unwrap()
}
#[tokio::test]
async fn four_protocol_fixture_sessions_keep_sibling_work_when_one_is_cancelled() {
    let (dir, owner, task) = results::tests::fixture().await;
    let sessions = NativeSessions::new(owner.clone()).unwrap();
    let attempts = attempts::AttemptManager::new(owner.clone());
    let mut handles = Vec::new();
    let mut units = Vec::new();
    for (index, provider) in ["claude", "codex", "claude", "codex"]
        .into_iter()
        .enumerate()
    {
        let task = if index == 0 {
            task.clone()
        } else {
            let mut next = crate::domain::Task::new(
                task.project_id,
                task.goal_id,
                format!("fixture-{index}"),
                provider.into(),
            );
            owner.store.lock().unwrap().put_task(&mut next).unwrap();
            next
        };
        let (unit, _) = attempts
            .prepare(task.id, provider, "Implement", None)
            .await
            .unwrap();
        let payload = if index == 0 { "cancel-me" } else { "complete" };
        let NativeStart::Launched(handle) = sessions
            .start_inner(
                input(&unit, payload),
                None,
                None,
                Some(program(dir.path(), provider)),
            )
            .await
            .unwrap()
        else {
            panic!("fixture unexpectedly queued")
        };
        handles.push(handle);
        units.push(unit);
    }
    tokio::time::timeout(Duration::from_secs(10), async {
        while units.iter().any(|u| {
            !owner
                .root
                .join("units")
                .join(u.id.to_string())
                .join("output/fixture-ready")
                .exists()
        }) {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    for handle in &handles {
        assert_eq!(
            sessions.status(handle).unwrap().session.state,
            SessionState::Running
        );
        assert_eq!(sessions.status(handle).unwrap().work, None);
    }
    let foreign = ManagedSessionRef {
        unit: handles[1].unit,
        ..handles[0].clone()
    };
    assert!(sessions.status(&foreign).is_err());
    sessions.cancel(&handles[0]).await.unwrap();
    for unit in units.iter().skip(1) {
        std::fs::write(
            owner
                .root
                .join("units")
                .join(unit.id.to_string())
                .join("output/fixture-release"),
            "release\n",
        )
        .unwrap();
    }
    let stopped = terminal(&sessions, &handles[0]).await;
    assert_eq!(stopped.disposition, Disposition::Cancelled);
    assert_eq!(stopped.work, Some(WorkOutcome::Unknown));
    let results = results::ResultStore::new(owner.clone());
    for (unit, handle) in units.iter().zip(&handles).skip(1) {
        let status = terminal(&sessions, handle).await;
        assert_eq!(status.work, Some(WorkOutcome::Success));
        assert_eq!(status.cleanup, CleanupOutcome::Unknown); // Fixture completion is not descendant-death evidence.
        let current = owner.store.lock().unwrap().execution_unit(unit.id).unwrap();
        let sha = results::text(
            &results::git(&unit.worktree, ["rev-parse", "HEAD"])
                .await
                .unwrap(),
        )
        .unwrap();
        let artifact = results
            .capture(&current.authority(), &sha, BTreeMap::new())
            .await
            .unwrap();
        let version = owner
            .store
            .lock()
            .unwrap()
            .task(unit.scope.task_id.unwrap())
            .unwrap()
            .unwrap()
            .version;
        let current = owner.store.lock().unwrap().execution_unit(unit.id).unwrap();
        let published = results
            .publish(&current.authority(), &artifact, version)
            .await
            .unwrap();
        results.verify(&published).await.unwrap();
    }
    assert!(!units[0].worktree.join("fixture-result.txt").exists());
}

#[tokio::test]
async fn structured_quota_terminals_wait_without_converting_ordinary_failures() {
    for (provider, payload, work, disposition, exhausted) in [
        (
            "codex",
            "quota-terminal",
            WorkOutcome::Unknown,
            Disposition::QuotaInterrupted,
            true,
        ),
        (
            "codex",
            "quota-retry-terminal",
            WorkOutcome::Unknown,
            Disposition::QuotaInterrupted,
            true,
        ),
        (
            "codex",
            "ordinary-failure",
            WorkOutcome::Failure,
            Disposition::Completed,
            false,
        ),
        (
            "claude",
            "quota-other-bucket",
            WorkOutcome::Unknown,
            Disposition::QuotaInterrupted,
            true,
        ),
        (
            "claude",
            "quota-stale-available",
            WorkOutcome::Unknown,
            Disposition::QuotaInterrupted,
            true,
        ),
        (
            "claude",
            "quota-budget-failure",
            WorkOutcome::Failure,
            Disposition::Completed,
            true,
        ),
        (
            "claude",
            "quota-foreign",
            WorkOutcome::Failure,
            Disposition::Completed,
            false,
        ),
    ] {
        let (dir, owner, task) = results::tests::fixture().await;
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, provider, "Implement", None)
            .await
            .unwrap();
        let NativeStart::Launched(handle) = sessions
            .start_inner(
                input(&unit, payload),
                None,
                None,
                Some(program(dir.path(), provider)),
            )
            .await
            .unwrap()
        else {
            panic!("fixture queued")
        };
        let result = terminal(&sessions, &handle).await;
        assert_eq!(result.work, Some(work), "{provider}/{payload}");
        assert_eq!(result.disposition, disposition, "{provider}/{payload}");
        assert_eq!(
            owner
                .store
                .lock()
                .unwrap()
                .quota_observations(provider, "unknown")
                .unwrap()
                .iter()
                .any(|o| o.status == QuotaStatus::Exhausted),
            exhausted,
            "{provider}/{payload}"
        );
    }
}

#[tokio::test]
async fn version_preparation_abort_and_retirement_do_not_leave_launch_authority() {
    for abort in [true, false] {
        let (dir, owner, task) = results::tests::fixture().await;
        let sessions = Arc::new(NativeSessions::new(owner.clone()).unwrap());
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, "claude", "Implement", None)
            .await
            .unwrap();
        let path = program(dir.path(), "claude");
        let script = std::fs::read_to_string(&path).unwrap().replace("if sys.argv[1:] == [\"--version\"]:","if sys.argv[1:] == [\"--version\"]:\n    with open(os.path.join(os.environ[\"RRX_OUTPUT_DIR\"], \"fixture-version-ready\"), \"w\") as ready: ready.write(\"ready\")\n    while True: time.sleep(0.02)");
        std::fs::write(&path, script).unwrap();
        let start_sessions = sessions.clone();
        let start_input = input(&unit, "must not dispatch");
        let start = tokio::spawn(async move {
            start_sessions
                .start_inner(start_input, None, None, Some(path))
                .await
        });
        let ready = owner
            .root
            .join("units")
            .join(unit.id.to_string())
            .join("output/fixture-version-ready");
        tokio::time::timeout(Duration::from_secs(5), async {
            while !ready.exists() {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        assert!(
            attempts::AttemptManager::new(owner.clone())
                .prepare(task.id, "claude", "Implement", None)
                .await
                .is_err()
        );
        assert!(
            owner
                .store
                .lock()
                .unwrap()
                .execution_unit(unit.id)
                .unwrap()
                .native_effects_open
        );
        if abort {
            start.abort();
            assert!(matches!(start.await, Err(e) if e.is_cancelled()));
        } else {
            {
                let mut store = owner.store.lock().unwrap();
                let current = store.execution_unit(unit.id).unwrap();
                store.retire_execution(&current.authority(), false).unwrap();
            }
            assert!(
                tokio::time::timeout(Duration::from_secs(5), start)
                    .await
                    .unwrap()
                    .unwrap()
                    .is_err()
            );
        }
        {
            let store = owner.store.lock().unwrap();
            let closed = store.execution_unit(unit.id).unwrap();
            assert!(!closed.native_effects_open && !closed.result_finalization_open);
            assert_eq!(closed.work, Some(WorkOutcome::Unknown));
            assert_eq!(
                closed.disposition,
                if abort {
                    Disposition::Lost
                } else {
                    Disposition::Cancelled
                }
            );
            assert!(closed.session_id.is_none());
            let effects = store.managed_effects(unit.id).unwrap();
            assert!(effects.iter().all(|e| e.state != EffectState::Pending));
            assert!(
                effects
                    .iter()
                    .any(|e| e.kind == "native_version" && e.state == EffectState::Unknown)
            );
            assert!(effects.iter().all(|e| e.kind != "native_input"));
        }
        let (fresh, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, "claude", "Implement", None)
            .await
            .unwrap();
        assert_ne!(fresh.worktree, unit.worktree);
        assert_ne!(fresh.branch, unit.branch);
    }
}

#[tokio::test]
async fn competing_native_start_cannot_retire_the_live_winner() {
    let (dir, owner, task) = results::tests::fixture().await;
    let sessions = Arc::new(NativeSessions::new(owner.clone()).unwrap());
    let (unit, _) = attempts::AttemptManager::new(owner.clone())
        .prepare(task.id, "claude", "Implement", None)
        .await
        .unwrap();
    let path = program(dir.path(), "claude");
    let script=std::fs::read_to_string(&path).unwrap().replace("if sys.argv[1:] == [\"--version\"]:","if sys.argv[1:] == [\"--version\"]:\n    with open(os.path.join(os.environ[\"RRX_OUTPUT_DIR\"], \"fixture-version-ready\"), \"w\") as ready: ready.write(\"ready\")\n    while not os.path.exists(os.path.join(os.environ[\"RRX_OUTPUT_DIR\"], \"fixture-version-release\")): time.sleep(0.02)");
    std::fs::write(&path, script).unwrap();
    let first_sessions = sessions.clone();
    let first_input = input(&unit, "hold");
    let first_path = path.clone();
    let first = tokio::spawn(async move {
        first_sessions
            .start_inner(first_input, None, None, Some(first_path))
            .await
    });
    let output = owner
        .root
        .join("units")
        .join(unit.id.to_string())
        .join("output");
    tokio::time::timeout(Duration::from_secs(5), async {
        while !output.join("fixture-version-ready").exists() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    let second_sessions = sessions.clone();
    let second_input = input(&unit, "duplicate");
    let second = tokio::spawn(async move {
        second_sessions
            .start_inner(second_input, None, None, Some(path))
            .await
    });
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(!second.is_finished());
    std::fs::write(output.join("fixture-version-release"), "release").unwrap();
    let NativeStart::Launched(handle) = first.await.unwrap().unwrap() else {
        panic!("fixture queued")
    };
    assert!(
        tokio::time::timeout(Duration::from_secs(5), second)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    {
        let store = owner.store.lock().unwrap();
        let current = store.execution_unit(unit.id).unwrap();
        assert!(current.native_effects_open);
        assert_eq!(current.session_id, Some(handle.session));
        assert_eq!(current.work, None);
        assert_eq!(
            store
                .managed_effects(unit.id)
                .unwrap()
                .iter()
                .filter(|e| e.kind == "native_version")
                .count(),
            1
        );
    }
    sessions.cancel(&handle).await.unwrap();
    assert_eq!(
        terminal(&sessions, &handle).await.disposition,
        Disposition::Cancelled
    );
}

#[tokio::test]
async fn authentication_and_unclassified_capacity_terminals_have_distinct_outcomes() {
    for (provider, payload, failure, work, disposition) in [
        (
            "claude",
            "authentication-terminal",
            Some(NativeFailure::AuthenticationUnavailable),
            WorkOutcome::Unknown,
            Disposition::Refused,
        ),
        (
            "claude",
            "authentication-structured",
            Some(NativeFailure::AuthenticationUnavailable),
            WorkOutcome::Unknown,
            Disposition::Refused,
        ),
        (
            "claude",
            "unsupported-control",
            Some(NativeFailure::UnsupportedCapability),
            WorkOutcome::Unknown,
            Disposition::Refused,
        ),
        (
            "codex",
            "authentication-terminal",
            Some(NativeFailure::AuthenticationUnavailable),
            WorkOutcome::Unknown,
            Disposition::Refused,
        ),
        (
            "claude",
            "capacity-rate",
            None,
            WorkOutcome::Unknown,
            Disposition::CapacityInterrupted,
        ),
        (
            "claude",
            "capacity-http",
            None,
            WorkOutcome::Unknown,
            Disposition::CapacityInterrupted,
        ),
        (
            "claude",
            "capacity-unknown-window",
            None,
            WorkOutcome::Unknown,
            Disposition::CapacityInterrupted,
        ),
        (
            "claude",
            "capacity-cap-control",
            None,
            WorkOutcome::Failure,
            Disposition::Completed,
        ),
        (
            "claude",
            "capacity-background-control",
            None,
            WorkOutcome::Failure,
            Disposition::Completed,
        ),
        (
            "claude",
            "capacity-retry-success",
            None,
            WorkOutcome::Success,
            Disposition::Completed,
        ),
        (
            "codex",
            "capacity-rate",
            None,
            WorkOutcome::Unknown,
            Disposition::CapacityInterrupted,
        ),
        (
            "codex",
            "capacity-flex",
            None,
            WorkOutcome::Unknown,
            Disposition::CapacityInterrupted,
        ),
        (
            "codex",
            "capacity-overload",
            None,
            WorkOutcome::Unknown,
            Disposition::CapacityInterrupted,
        ),
        (
            "codex",
            "capacity-http",
            None,
            WorkOutcome::Unknown,
            Disposition::CapacityInterrupted,
        ),
        (
            "codex",
            "capacity-retry-success",
            None,
            WorkOutcome::Success,
            Disposition::Completed,
        ),
    ] {
        let (dir, owner, task) = results::tests::fixture().await;
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, provider, "Implement", None)
            .await
            .unwrap();
        let NativeStart::Launched(handle) = sessions
            .start_inner(
                input(&unit, payload),
                None,
                None,
                Some(program(dir.path(), provider)),
            )
            .await
            .unwrap()
        else {
            panic!("fixture queued")
        };
        let status = terminal(&sessions, &handle).await;
        assert_eq!(
            (status.failure, status.work, status.disposition),
            (failure, Some(work), disposition),
            "{provider}/{payload}"
        );
        let store = owner.store.lock().unwrap();
        let current = store.execution_unit(unit.id).unwrap();
        assert!(!current.native_effects_open);
        if disposition == Disposition::CapacityInterrupted {
            assert_eq!(current.wait_reason, Some(WaitReason::Capacity));
            assert!(!current.result_finalization_open);
            let due = current.capacity_retry_at.unwrap();
            assert!(due >= current.created_at && due <= now_ms() + 60_000);
        }
        assert!(
            store
                .quota_observations(provider, "unknown")
                .unwrap()
                .iter()
                .all(|o| o.status != QuotaStatus::Exhausted)
        );
        let events = serde_json::to_string(&store.events(&unit.scope, 0, 1000).unwrap()).unwrap();
        assert!(!events.contains("PRIVATE_FIXTURE_ERROR_MUST_NOT_PERSIST"));
    }
}

#[tokio::test]
async fn codex_two_window_recovery_refresh_keeps_correlated_probe_through_turn_dispatch() {
    let (dir, owner, task) = results::tests::fixture().await;
    let (unit, _) = attempts::AttemptManager::new(owner.clone())
        .prepare(task.id, "codex", "Implement", None)
        .await
        .unwrap();
    let at = now_ms();
    for bucket in ["fixture/primary", "fixture/secondary"] {
        owner
            .store
            .lock()
            .unwrap()
            .observe_quota(&QuotaObservation {
                provider: "codex".into(),
                account_key: "unknown".into(),
                bucket: bucket.into(),
                window_id: "old".into(),
                status: QuotaStatus::Exhausted,
                used_percent: Some(100.0),
                resets_at: Some(at - 1),
                observed_at: at - 60001,
                source_version: "fixture".into(),
                confirmed_subscription: true,
            })
            .unwrap();
    }
    let path = program(dir.path(), "codex");
    let script = std::fs::read_to_string(&path).unwrap().replace(
        r#""secondary": None"#,
        r#""secondary": {"usedPercent": 30}"#,
    );
    std::fs::write(&path, script).unwrap();
    std::fs::write(
        owner
            .root
            .join("units")
            .join(unit.id.to_string())
            .join("output/fixture-release"),
        "release",
    )
    .unwrap();
    let sessions = NativeSessions::new(owner.clone()).unwrap();
    let NativeStart::Launched(handle) = sessions
        .start_inner(input(&unit, "complete"), None, None, Some(path))
        .await
        .unwrap()
    else {
        panic!("probe was not admitted")
    };
    let result = terminal(&sessions, &handle).await;
    assert_eq!(result.work, Some(WorkOutcome::Success));
    assert_eq!(result.disposition, Disposition::Completed);
    let store = owner.store.lock().unwrap();
    let windows = store.quota_observations("codex", "unknown").unwrap();
    assert_eq!(windows.len(), 2);
    assert!(windows.iter().all(|w| w.status == QuotaStatus::Available));
    assert!(
        !store
            .execution_is_quota_probe(unit.id, "codex", "unknown")
            .unwrap()
    );
    let effects = store.managed_effects(unit.id).unwrap();
    assert_eq!(
        effects
            .iter()
            .filter(|e| e.kind == "native_input" && e.state == EffectState::Confirmed)
            .count(),
        1
    );
}

#[tokio::test]
async fn readonly_review_profile_is_selected_in_the_actual_fixture_launch() {
    for provider in ["claude", "codex"] {
        let (dir, owner, task) = results::tests::fixture().await;
        let attempts = attempts::AttemptManager::new(owner.clone());
        let (executor, _) = attempts
            .prepare(task.id, provider, "Implement", None)
            .await
            .unwrap();
        let done = owner
            .store
            .lock()
            .unwrap()
            .finish_execution(
                &executor.authority(),
                WorkOutcome::Success,
                Disposition::Completed,
            )
            .unwrap();
        let retained = results::ResultStore::new(owner.clone());
        let artifact = retained
            .capture(&done.authority(), &done.base_sha, BTreeMap::new())
            .await
            .unwrap();
        let version = owner
            .store
            .lock()
            .unwrap()
            .task(task.id)
            .unwrap()
            .unwrap()
            .version;
        let artifact = retained
            .publish(&done.authority(), &artifact, version)
            .await
            .unwrap();
        let (reviewer, _) = attempts
            .prepare_snapshot(task.id, artifact.id, UnitKind::Reviewer, provider, "Review")
            .await
            .unwrap();
        retained.snapshot(&reviewer).await.unwrap();
        let reviewer = owner
            .store
            .lock()
            .unwrap()
            .execution_unit(reviewer.id)
            .unwrap();
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let NativeStart::Launched(handle) = sessions
            .start_inner(
                input(&reviewer, "readonly-review"),
                None,
                None,
                Some(program(dir.path(), provider)),
            )
            .await
            .unwrap()
        else {
            panic!("review fixture queued")
        };
        let status = terminal(&sessions, &handle).await;
        assert_eq!(status.work, Some(WorkOutcome::Success), "{provider}");
        assert!(!reviewer.worktree.join("fixture-review-write").exists());
        results::verify_readonly_source(&reviewer.worktree).unwrap();
    }
}

#[tokio::test]
async fn native_dispatch_admission_has_a_durable_winner_against_retirement() {
    for provider in ["claude", "codex"] {
        let (dir, owner, task) = results::tests::fixture().await;
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, provider, "Implement", None)
            .await
            .unwrap();
        let NativeStart::Launched(handle) = sessions
            .start_inner(
                input(&unit, "cancel-me"),
                None,
                None,
                Some(program(dir.path(), provider)),
            )
            .await
            .unwrap()
        else {
            panic!("fixture queued")
        };
        tokio::time::timeout(Duration::from_secs(10), async {
            while !owner
                .root
                .join("units")
                .join(unit.id.to_string())
                .join("output/fixture-ready")
                .is_file()
            {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        let pinned = owner.store.lock().unwrap().execution_unit(unit.id).unwrap();
        // Exercise the same private producer used for actual input/ALLOW frames.
        // The fixture does not claim native permission conformance or revoke issued bytes.
        let frame = json!({"owned_permission":"fixture-sensitive-argument"});
        let issued = admit_native_frame(
            &owner,
            &pinned,
            handle.session,
            Some(&pinned.authority()),
            "native_permission",
            &frame,
        )
        .unwrap();
        let effect = owner.store.lock().unwrap().managed_effect(issued).unwrap();
        assert_eq!(effect.state, EffectState::Pending);
        assert!(
            !serde_json::to_string(&effect)
                .unwrap()
                .contains("fixture-sensitive-argument")
        );
        sessions.cancel(&handle).await.unwrap();
        assert!(
            admit_native_frame(
                &owner,
                &pinned,
                handle.session,
                Some(&pinned.authority()),
                "native_permission",
                &frame
            )
            .is_err()
        );
        assert!(
            admit_native_frame(
                &owner,
                &pinned,
                handle.session,
                None,
                "native_input",
                &json!({"late_input":true})
            )
            .is_err()
        );
        assert_eq!(
            owner
                .store
                .lock()
                .unwrap()
                .managed_effect(issued)
                .unwrap()
                .state,
            EffectState::Pending
        );
        assert_eq!(
            terminal(&sessions, &handle).await.disposition,
            Disposition::Cancelled
        );
    }
}

#[tokio::test]
async fn cancellation_during_bootstrap_closes_the_session_without_waiting_for_rpc_deadline() {
    for provider in ["claude", "codex"] {
        let (dir, owner, task) = results::tests::fixture().await;
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, provider, "Implement", None)
            .await
            .unwrap();
        let path = program(dir.path(), provider);
        let script = std::fs::read_to_string(&path).unwrap().replace(
            "# Local protocol fixture only:",
            "BOOTSTRAP_HOLD = True\n# Local protocol fixture only:",
        );
        std::fs::write(&path, script).unwrap();
        let NativeStart::Launched(handle) = sessions
            .start_inner(input(&unit, "cancel-me"), None, None, Some(path))
            .await
            .unwrap()
        else {
            panic!("fixture queued")
        };
        tokio::time::timeout(Duration::from_secs(10), async {
            while !owner
                .root
                .join("units")
                .join(unit.id.to_string())
                .join("output/fixture-bootstrap-ready")
                .is_file()
            {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        sessions.cancel(&handle).await.unwrap();
        let result = tokio::time::timeout(Duration::from_secs(5), terminal(&sessions, &handle))
            .await
            .unwrap();
        assert_eq!(result.disposition, Disposition::Cancelled);
        assert_eq!(result.work, Some(WorkOutcome::Unknown));
        assert!(
            !owner
                .store
                .lock()
                .unwrap()
                .execution_unit(unit.id)
                .unwrap()
                .native_effects_open
        );
        assert!(
            owner
                .store
                .lock()
                .unwrap()
                .managed_effects(unit.id)
                .unwrap()
                .iter()
                .all(|e| e.kind != "native_input")
        );
    }
}

#[tokio::test]
async fn bootstrap_failure_categories_survive_without_retaining_native_error_payloads() {
    for (case, expected, disposition) in [
        (
            "auth",
            NativeFailure::AuthenticationUnavailable,
            Disposition::Refused,
        ),
        (
            "api",
            NativeFailure::AuthenticationUnavailable,
            Disposition::Refused,
        ),
        (
            "environment",
            NativeFailure::UnsupportedCapability,
            Disposition::Refused,
        ),
        (
            "rpc-unsupported",
            NativeFailure::UnsupportedCapability,
            Disposition::Refused,
        ),
        (
            "metadata",
            NativeFailure::MetadataUnavailable,
            Disposition::Refused,
        ),
        (
            "protocol",
            NativeFailure::ProtocolFailure,
            Disposition::ProtocolError,
        ),
        ("transport", NativeFailure::TransportLost, Disposition::Lost),
    ] {
        let (dir, owner, task) = results::tests::fixture().await;
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, "codex", "Implement", None)
            .await
            .unwrap();
        let path = program(dir.path(), "codex");
        let script = std::fs::read_to_string(&path).unwrap().replace(
            "# Local protocol fixture only:",
            &format!("BOOTSTRAP_CASE = {case:?}\n# Local protocol fixture only:"),
        );
        std::fs::write(&path, script).unwrap();
        let NativeStart::Launched(handle) = sessions
            .start_inner(input(&unit, "must not dispatch"), None, None, Some(path))
            .await
            .unwrap()
        else {
            panic!("fixture queued")
        };
        let status = terminal(&sessions, &handle).await;
        assert_eq!(status.failure, Some(expected), "{case}");
        assert_eq!(status.diagnostic, Some(expected.diagnostic()), "{case}");
        assert_eq!(status.disposition, disposition, "{case}");
        assert_eq!(status.work, Some(WorkOutcome::Unknown));
        let store = owner.store.lock().unwrap();
        let events = store.events(&unit.scope, 0, 1000).unwrap();
        let event = events
            .iter()
            .find(|event| event.kind == "execution.work_terminal")
            .unwrap();
        assert_eq!(event.data["native_failure"], json!(expected), "{case}");
        assert!(
            !serde_json::to_string(&events)
                .unwrap()
                .contains("PRIVATE_FIXTURE_ERROR_MUST_NOT_PERSIST")
        );
        assert!(
            store
                .managed_effects(unit.id)
                .unwrap()
                .iter()
                .all(|e| e.kind != "native_input")
        );
    }
}

#[tokio::test]
async fn native_subscription_wait_reason_matches_status_during_retry_and_terminal() {
    let (dir, owner, task) = results::tests::fixture().await;
    let (unit, _) = attempts::AttemptManager::new(owner.clone())
        .prepare(task.id, "codex", "Implement", None)
        .await
        .unwrap();
    let output = resources::ResourceManager::new(owner.clone())
        .profile(&unit)
        .unwrap()
        .output;
    let sessions = NativeSessions::new(owner).unwrap();
    let NativeStart::Launched(handle) = sessions
        .start_inner(
            input(&unit, "quota-retry-held"),
            None,
            None,
            Some(program(dir.path(), "codex")),
        )
        .await
        .unwrap()
    else {
        panic!("expected launch");
    };
    let mut updates = sessions.subscribe(&handle).unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let status = updates.borrow().clone();
            if status.wait_reason == Some(WaitReason::Quota) {
                assert!(status.work.is_none());
                assert_eq!(
                    sessions.status(&handle).unwrap().wait_reason,
                    status.wait_reason
                );
                assert_eq!(
                    sessions.status(&handle).unwrap().authority,
                    status.authority
                );
                break;
            }
            updates.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    std::fs::write(output.join("fixture-quota-release"), "release").unwrap();
    let end = terminal(&sessions, &handle).await;
    assert_eq!(end.wait_reason, Some(WaitReason::Quota));
    assert_eq!(
        end.wait_reason,
        sessions.status(&handle).unwrap().wait_reason
    );
    assert_eq!(end.work, Some(WorkOutcome::Unknown));
    assert_eq!(end.disposition, Disposition::QuotaInterrupted);
}

#[tokio::test]
async fn claude_live_quota_wait_is_scoped_preserves_input_and_keeps_sibling_running() {
    for (payload, waits, work, disposition) in [
        (
            "quota-retry-held",
            true,
            WorkOutcome::Unknown,
            Disposition::QuotaInterrupted,
        ),
        (
            "quota-retry-success-held",
            true,
            WorkOutcome::Success,
            Disposition::Completed,
        ),
        (
            "quota-foreign-held",
            false,
            WorkOutcome::Failure,
            Disposition::Completed,
        ),
        (
            "quota-unknown-held",
            false,
            WorkOutcome::Unknown,
            Disposition::CapacityInterrupted,
        ),
    ] {
        let (dir, owner, task) = results::tests::fixture().await;
        let manager = attempts::AttemptManager::new(owner.clone());
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let mut sibling = crate::domain::Task::new(
            task.project_id,
            task.goal_id,
            "sibling".into(),
            "claude".into(),
        );
        owner.store.lock().unwrap().put_task(&mut sibling).unwrap();
        let (other, _) = manager
            .prepare(sibling.id, "claude", "Implement", None)
            .await
            .unwrap();
        let program = program(dir.path(), "claude");
        let NativeStart::Launched(other_handle) = sessions
            .start_inner(input(&other, "complete"), None, None, Some(program.clone()))
            .await
            .unwrap()
        else {
            panic!("sibling queued");
        };
        let (unit, _) = manager
            .prepare(task.id, "claude", "Implement", None)
            .await
            .unwrap();
        let output = resources::ResourceManager::new(owner.clone())
            .profile(&unit)
            .unwrap()
            .output;
        let NativeStart::Launched(handle) = sessions
            .start_inner(input(&unit, payload), None, None, Some(program))
            .await
            .unwrap()
        else {
            panic!("quota fixture queued");
        };
        let mut updates = sessions.subscribe(&handle).unwrap();
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                if !updates.borrow().pending.is_empty() {
                    break;
                }
                updates.changed().await.unwrap();
            }
        })
        .await
        .unwrap();
        let watched = updates.borrow().clone();
        let durable = sessions.status(&handle).unwrap();
        let store = owner.store.lock().unwrap();
        let current = store.execution_unit(unit.id).unwrap();
        let expected_wait = waits.then_some(WaitReason::Quota);
        assert_eq!(watched.wait_reason, expected_wait, "{payload}: watch");
        assert_eq!(durable.wait_reason, expected_wait, "{payload}: status");
        assert_eq!(current.wait_reason, expected_wait, "{payload}: ledger");
        assert_eq!(watched.authority, durable.authority, "{payload}");
        assert_eq!(
            current.state,
            if waits {
                UnitState::WaitingQuota
            } else {
                UnitState::Running
            }
        );
        assert!(current.native_effects_open && current.result_finalization_open);
        assert_eq!(current.work, None);
        assert!(
            !store
                .execution_is_quota_probe(unit.id, "claude", "unknown")
                .unwrap(),
            "{payload}: telemetry does not mint recovery probe authority"
        );
        let sibling_before = store.execution_unit(other.id).unwrap();
        assert_eq!(sibling_before.state, UnitState::Running);
        assert_eq!(sibling_before.wait_reason, None);
        assert_eq!(sibling_before.work, None);
        for id in [unit.id, other.id] {
            assert_eq!(
                store
                    .managed_effects(id)
                    .unwrap()
                    .iter()
                    .filter(|e| e.kind == "native_input")
                    .count(),
                1,
                "{payload}: input before release"
            );
        }
        drop(store);
        std::fs::write(output.join("fixture-quota-release"), "release").unwrap();
        let end = terminal(&sessions, &handle).await;
        assert_eq!(end.work, Some(work), "{payload}");
        assert_eq!(end.disposition, disposition, "{payload}");
        let store = owner.store.lock().unwrap();
        assert_eq!(
            serde_json::to_value(store.execution_unit(other.id).unwrap()).unwrap(),
            serde_json::to_value(sibling_before).unwrap(),
            "{payload}: sibling ledger"
        );
        assert_eq!(
            store
                .managed_effects(unit.id)
                .unwrap()
                .iter()
                .filter(|e| e.kind == "native_input")
                .count(),
            1,
            "{payload}: input not resent"
        );
        assert!(
            !store
                .execution_is_quota_probe(unit.id, "claude", "unknown")
                .unwrap()
        );
        drop(store);
        sessions.cancel(&other_handle).await.unwrap();
        assert_eq!(
            terminal(&sessions, &other_handle).await.disposition,
            Disposition::Cancelled
        );
    }
}

#[tokio::test]
async fn configured_global_provider_alias_and_project_caps_wait_before_native_spawn() {
    for limit in ["global", "provider-alias", "project"] {
        let (dir, owner, task) = results::tests::fixture().await;
        let mut config = crate::config::Config::default();
        if limit == "global" {
            config.scheduler.global_max_sessions = 1;
        }
        if limit == "project" {
            config.scheduler.max_tasks_per_project = 1;
        }
        if limit == "provider-alias" {
            for (name, cap) in [("fast", 3), ("bounded", 1)] {
                config.agents.insert(
                    name.into(),
                    crate::config::AgentConfig {
                        provider: Some("codex".into()),
                        max_concurrent: Some(cap),
                        ..Default::default()
                    },
                );
            }
        }
        let sessions =
            NativeSessions::with_limits(owner.clone(), NativeLimits::configured(&config)).unwrap();
        let manager = attempts::AttemptManager::new(owner.clone());
        let first_provider = if limit == "provider-alias" {
            "codex"
        } else {
            "claude"
        };
        let (first, _) = manager
            .prepare(task.id, first_provider, "Implement", None)
            .await
            .unwrap();
        let NativeStart::Launched(handle) = sessions
            .start_inner(
                input(&first, "complete"),
                None,
                None,
                Some(program(dir.path(), first_provider)),
            )
            .await
            .unwrap()
        else {
            panic!("first launch unexpectedly waiting");
        };
        let mut sibling = crate::domain::Task::new(
            task.project_id,
            task.goal_id,
            "cap sibling".into(),
            "codex".into(),
        );
        owner.store.lock().unwrap().put_task(&mut sibling).unwrap();
        let (second, _) = manager
            .prepare(sibling.id, "codex", "Implement", None)
            .await
            .unwrap();
        let NativeStart::Waiting { unit, reason, .. } = sessions
            .start_inner(
                input(&second, "complete"),
                None,
                None,
                Some(program(dir.path(), "codex")),
            )
            .await
            .unwrap()
        else {
            panic!("configured cap bypassed: {limit}");
        };
        assert_eq!(reason, WaitReason::Capacity);
        assert_eq!(unit.work, None);
        assert!(unit.native_effects_open);
        assert!(unit.session_id.is_none());
        assert!(
            !owner
                .store
                .lock()
                .unwrap()
                .managed_effects(unit.id)
                .unwrap()
                .iter()
                .any(|e| e.kind == "native_spawn" || e.kind == "native_input")
        );
        sessions.cancel(&handle).await.unwrap();
        assert_eq!(
            terminal(&sessions, &handle).await.disposition,
            Disposition::Cancelled
        );
    }
}

#[test]
fn dropped_executor_runtime_preserves_durable_lost_subscription_and_session() {
    for payload in ["complete", "quota-retry-held"] {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let (dir, owner, sessions, handle, updates) = runtime.block_on(async {
            let (dir, owner, task) = results::tests::fixture().await;
            let (unit, _) = attempts::AttemptManager::new(owner.clone())
                .prepare(task.id, "codex", "Implement", None)
                .await
                .unwrap();
            let sessions = NativeSessions::new(owner.clone()).unwrap();
            let NativeStart::Launched(handle) = sessions
                .start_inner(
                    input(&unit, payload),
                    None,
                    None,
                    Some(program(dir.path(), "codex")),
                )
                .await
                .unwrap()
            else {
                panic!("unexpected wait");
            };
            let mut updates = sessions.subscribe(&handle).unwrap();
            tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    let status = updates.borrow().clone();
                    if status.session.state == SessionState::Running
                        && (payload != "quota-retry-held"
                            || status.wait_reason == Some(WaitReason::Quota))
                    {
                        break;
                    }
                    updates.changed().await.unwrap();
                }
            })
            .await
            .unwrap();
            (dir, owner, sessions, handle, updates)
        });
        drop(runtime);
        let watched = updates.borrow().clone();
        let durable = sessions.status(&handle).unwrap();
        let store = owner.store.lock().unwrap();
        let unit = store.execution_unit(handle.unit).unwrap();
        let (session, _) = store.session(handle.session).unwrap().unwrap();
        assert!(!unit.native_effects_open && !unit.result_finalization_open);
        assert_eq!(unit.work, Some(WorkOutcome::Unknown));
        assert_eq!(unit.disposition, Disposition::Lost);
        assert_eq!(session.state, SessionState::Lost);
        assert_eq!(watched.session.state, session.state);
        assert_eq!(watched.authority, durable.authority);
        assert_eq!(watched.work, durable.work);
        assert_eq!(watched.disposition, durable.disposition);
        assert_eq!(watched.wait_reason, durable.wait_reason);
        assert_eq!(
            unit.wait_reason,
            if payload == "quota-retry-held" {
                Some(WaitReason::Quota)
            } else {
                None
            }
        );
        assert_eq!(
            store
                .managed_effects(unit.id)
                .unwrap()
                .iter()
                .filter(|e| e.kind == "native_input")
                .count(),
            1
        );
        drop(store);
        drop(sessions);
        drop(owner);
        drop(dir);
    }
}
