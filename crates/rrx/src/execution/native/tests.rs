use super::*;
use crate::adapter::InputKind;
use std::{os::unix::fs::PermissionsExt, path::Path};

fn program(root: &Path, provider: &str) -> std::path::PathBuf {
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
fn input(unit: &ExecutionUnit, payload: &str) -> ManagedInput {
    ManagedInput {
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
