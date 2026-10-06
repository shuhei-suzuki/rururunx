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
async fn workflow_source_bootstrap_reads_commit_and_refuses_native_before_effects() {
    for provider in ["claude", "codex"] {
        let (dir, owner, task) = results::tests::fixture().await;
        let attempts = attempts::AttemptManager::new(owner.clone());
        assert!(
            attempts
                .prepare(task.id, provider, WORKFLOW_SOURCE_BOOTSTRAP, None)
                .await
                .is_err()
        );
        let snapshot_error = attempts
            .prepare_snapshot(
                task.id,
                ArtifactId::new(),
                UnitKind::Reviewer,
                provider,
                WORKFLOW_SOURCE_BOOTSTRAP,
            )
            .await
            .err()
            .unwrap();
        assert!(
            snapshot_error
                .to_string()
                .contains("reserved preparation phase")
        );
        assert!(
            owner
                .store
                .lock()
                .unwrap()
                .execution_units(Some(&task.scope()))
                .unwrap()
                .is_empty()
        );
        let prepared = attempts
            .prepare_workflow_source(task.id, provider)
            .await
            .unwrap();
        let unit = prepared.unit().clone();
        assert_eq!(unit.phase, WORKFLOW_SOURCE_BOOTSTRAP);
        assert_eq!(unit.state, UnitState::Preparing);
        assert_eq!(unit.work, None);
        assert_eq!(unit.session_id, None);
        assert_eq!(unit.artifact_id, None);
        assert!(valid_oid(&unit.base_sha));
        assert_eq!(
            prepared.read_committed_file("answer.txt").await.unwrap(),
            b"base\n"
        );
        // The file and branch HEAD can change; the input object does not.
        std::fs::write(unit.worktree.join("answer.txt"), "later\n").unwrap();
        results::git(&unit.worktree, ["add", "answer.txt"])
            .await
            .unwrap();
        results::git(
            &unit.worktree,
            [
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-m",
                "later",
            ],
        )
        .await
        .unwrap();
        assert_ne!(
            results::text(
                &results::git(&unit.worktree, ["rev-parse", "HEAD"])
                    .await
                    .unwrap()
            )
            .unwrap(),
            unit.base_sha
        );
        assert_eq!(
            prepared.read_committed_file("answer.txt").await.unwrap(),
            b"base\n"
        );
        let before = owner
            .store
            .lock()
            .unwrap()
            .managed_effects(unit.id)
            .unwrap();
        for path in ["../answer.txt", "/answer.txt", "./answer.txt", ""] {
            assert!(prepared.read_committed_file(path).await.is_err());
        }
        assert_eq!(
            owner
                .store
                .lock()
                .unwrap()
                .managed_effects(unit.id)
                .unwrap()
                .len(),
            before.len()
        );
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let error = sessions
            .start_inner(
                input(&unit, "complete"),
                None,
                None,
                Some(program(dir.path(), provider)),
            )
            .await
            .err()
            .unwrap();
        assert!(error.to_string().contains("native input identity mismatch"));
        {
            let store = owner.store.lock().unwrap();
            let current = store.execution_unit(unit.id).unwrap();
            assert_eq!(current.work, None);
            assert_eq!(current.session_id, None);
            assert_eq!(current.state, UnitState::Preparing);
            assert!(current.native_effects_open && current.result_finalization_open);
            let effects = store.managed_effects(unit.id).unwrap();
            assert_eq!(effects.len(), before.len());
            assert!(
                effects
                    .iter()
                    .all(|e| e.kind == "git_helper" && e.state == EffectState::Confirmed)
            );
            let events = store.events(&unit.scope, 0, 1000).unwrap();
            let reserved = events
                .iter()
                .position(|e| e.kind == "execution.attempt_reserved")
                .unwrap();
            let bound = events
                .iter()
                .position(|e| e.kind == "execution.base_bound")
                .unwrap();
            assert!(reserved < bound);
        }
        drop(prepared);
        let retired = owner.store.lock().unwrap().execution_unit(unit.id).unwrap();
        assert!(!retired.native_effects_open && !retired.result_finalization_open);
        assert_eq!(retired.work, Some(WorkOutcome::Unknown));
        assert_eq!(retired.disposition, Disposition::Lost);
        assert!(unit.worktree.exists()); // retirement is not successful disposal
        let (retry, _) = attempts
            .prepare(task.id, provider, "Implement", None)
            .await
            .unwrap();
        assert_ne!(retry.id, unit.id);
        assert_ne!(retry.worktree, unit.worktree);
        assert!(retry.generation > unit.generation);
        attempts.retire(&retry.authority(), false).unwrap();
    }
}

#[tokio::test]
async fn workflow_source_bootstrap_file_reader_enforces_type_literal_path_and_size() {
    let (_dir, owner, task) = results::tests::fixture().await;
    let source = owner
        .store
        .lock()
        .unwrap()
        .project(task.project_id)
        .unwrap()
        .unwrap()
        .root;
    std::fs::write(source.join("limit.txt"), vec![b'x'; 256 * 1024]).unwrap();
    std::fs::write(source.join("large.txt"), vec![b'x'; 256 * 1024 + 1]).unwrap();
    std::fs::write(source.join(":(glob)*"), "literal\n").unwrap();
    std::fs::create_dir(source.join("nested")).unwrap();
    std::fs::write(source.join("nested/item.txt"), "nested\n").unwrap();
    results::git(&source, ["add", "--all"]).await.unwrap();
    results::git(
        &source,
        [
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            "source kinds",
        ],
    )
    .await
    .unwrap();
    let prepared = attempts::AttemptManager::new(owner.clone())
        .prepare_workflow_source(task.id, "codex")
        .await
        .unwrap();
    let bytes = prepared.read_committed_file("limit.txt").await.unwrap();
    assert_eq!(bytes.len(), 256 * 1024);
    assert!(bytes.iter().all(|b| *b == b'x'));
    let error = prepared
        .read_committed_file("large.txt")
        .await
        .err()
        .unwrap();
    assert!(error.to_string().contains("exceeds file bound"));
    assert_eq!(
        prepared.read_committed_file(":(glob)*").await.unwrap(),
        b"literal\n"
    );
    let error = prepared.read_committed_file("nested").await.err().unwrap();
    assert!(error.to_string().contains("not an exact ordinary file"));
    let unit = prepared.unit().clone();
    drop(prepared);
    assert_eq!(
        owner
            .store
            .lock()
            .unwrap()
            .execution_unit(unit.id)
            .unwrap()
            .work,
        Some(WorkOutcome::Unknown)
    );
    // The actual preparation qualifier refuses unsupported symlinks before
    // returning a capability. Do not bypass it to reach a reader-only assertion.
    std::os::unix::fs::symlink("answer.txt", source.join("link.txt")).unwrap();
    results::git(&source, ["add", "--all"]).await.unwrap();
    results::git(
        &source,
        [
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            "unsupported symlink",
        ],
    )
    .await
    .unwrap();
    let error = attempts::AttemptManager::new(owner.clone())
        .prepare_workflow_source(task.id, "codex")
        .await
        .err()
        .unwrap();
    assert!(
        error
            .to_string()
            .contains("source symlinks require a qualified profile")
    );
    let store = owner.store.lock().unwrap();
    let units = store.execution_units(Some(&task.scope())).unwrap();
    assert_eq!(units.len(), 2);
    assert!(units.iter().all(|u| !u.native_effects_open
        && !u.result_finalization_open
        && u.work == Some(WorkOutcome::Unknown)
        && u.session_id.is_none()
        && u.artifact_id.is_none()));
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
        let invocation = owner
            .store
            .lock()
            .unwrap()
            .native_session_invocation(handle.session)
            .unwrap()
            .id;
        let frame = json!({"owned_permission":"fixture-sensitive-argument"});
        let issued = admit_native_frame(
            &owner,
            &pinned,
            handle.session,
            invocation,
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
                invocation,
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
                invocation,
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
            "quota-live-recovery-held",
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
        let (current, sibling_before) = {
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
            (current, sibling_before)
        };
        if payload == "quota-live-recovery-held" {
            std::fs::write(output.join("fixture-recovery-release"), "recover").unwrap();
            tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    if updates.borrow().pending.len() == 2 {
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
            assert!(
                store
                    .quota_observations("claude", "unknown")
                    .unwrap()
                    .iter()
                    .all(|o| o.status == QuotaStatus::Available)
            );
            let recovered = store.execution_unit(unit.id).unwrap();
            assert_eq!(watched.wait_reason, None, "accepted live recovery: watch");
            assert_eq!(durable.wait_reason, None, "accepted live recovery: status");
            assert_eq!(
                recovered.wait_reason, None,
                "accepted live recovery: ledger"
            );
            assert_eq!(recovered.state, UnitState::Running);
            assert_eq!(recovered.session_id, current.session_id);
            assert_eq!(recovered.generation, current.generation);
            assert_eq!(recovered.worktree, current.worktree);
            assert!(recovered.native_effects_open && recovered.result_finalization_open);
            assert_eq!(recovered.work, None);
            assert_eq!(watched.authority, durable.authority);
        }
        std::fs::write(output.join("fixture-quota-release"), "release").unwrap();
        let end = terminal(&sessions, &handle).await;
        assert_eq!(end.work, Some(work), "{payload}");
        assert_eq!(end.disposition, disposition, "{payload}");
        {
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
        }
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

#[tokio::test]
async fn native_answers_are_owned_durable_and_failure_text_is_not_approval() {
    use crate::execution::native_result::{AcquisitionStatus, InvocationState, ReceiptAuthority};
    for provider in ["claude", "codex"] {
        let cases = if provider == "codex" {
            vec![
                ("answer-ok", AcquisitionStatus::Complete),
                ("answer-commentary", AcquisitionStatus::Complete),
                ("answer-foreign", AcquisitionStatus::Complete),
                ("answer-duplicate", AcquisitionStatus::Complete),
                ("answer-change", AcquisitionStatus::Ambiguous),
                ("answer-null-phase", AcquisitionStatus::Unsupported),
                ("answer-two-finals", AcquisitionStatus::Ambiguous),
                ("answer-terminal-only", AcquisitionStatus::Ambiguous),
                ("answer-summary", AcquisitionStatus::Complete),
                ("answer-delta-only", AcquisitionStatus::Partial),
                ("answer-failed", AcquisitionStatus::Complete),
                ("answer-encoded-overflow", AcquisitionStatus::Overflow),
            ]
        } else {
            vec![
                ("answer-ok", AcquisitionStatus::Complete),
                ("answer-failed", AcquisitionStatus::Complete),
                ("answer-structured", AcquisitionStatus::Unsupported),
                ("answer-encoded-overflow", AcquisitionStatus::Overflow),
            ]
        };
        for (payload, expected) in cases {
            let (dir, owner, task) = results::tests::fixture().await;
            let sessions = NativeSessions::new(owner.clone()).unwrap();
            let (unit, _) = attempts::AttemptManager::new(owner.clone())
                .prepare(task.id, provider, "Implement", None)
                .await
                .unwrap();
            let handle = match sessions
                .start_inner(
                    input(&unit, payload),
                    None,
                    None,
                    Some(program(dir.path(), provider)),
                )
                .await
                .unwrap()
            {
                NativeStart::Launched(handle) => handle,
                _ => panic!("fixture admission should launch"),
            };
            let status = terminal(&sessions, &handle).await;
            let expected_work = if payload == "answer-failed" {
                WorkOutcome::Failure
            } else {
                WorkOutcome::Success
            };
            assert_eq!(
                status.work,
                Some(expected_work),
                "{provider}/{payload}: {status:?}"
            );
            let receipt_id = status
                .receipt
                .expect("terminal watch follows durable receipt TX");
            let store = owner.store.lock().unwrap();
            let receipt = store.native_result(receipt_id).unwrap();
            let invocation = store.native_session_invocation(handle.session).unwrap();
            assert_eq!(invocation.state, InvocationState::Closed);
            assert!(invocation.input_operation.is_some() && invocation.frame_sha256.is_some());
            assert_eq!(
                invocation.payload_sha256,
                native_result::digest(payload.as_bytes())
            );
            assert_eq!(
                invocation.context_version, None,
                "standalone fixtures do not mint a Context"
            );
            assert_eq!(receipt.acquisition, expected, "{provider}/{payload}");
            assert_eq!(receipt.authority, ReceiptAuthority::OwnedTerminal);
            assert_eq!(receipt.observed_work, expected_work);
            assert_eq!(receipt.unit_id, handle.unit);
            assert_eq!(receipt.session_id, handle.session);
            assert_eq!(receipt.generation, handle.generation);
            assert_eq!(receipt.owner_epoch, handle.epoch);
            assert_eq!(
                store.session(handle.session).unwrap().unwrap().0.state,
                if payload == "answer-failed" {
                    SessionState::Failed
                } else {
                    SessionState::Exited
                }
            );
            assert!(!store.execution_unit(unit.id).unwrap().native_effects_open);
            assert!(
                !store
                    .execution_is_quota_probe(unit.id, provider, "unknown")
                    .unwrap()
            );
            let serialized = serde_json::to_string(&receipt).unwrap();
            assert!(!serialized.contains("FOREIGN must not persist"));
            assert!(!serialized.contains("commentary must not become final"));
            if expected == AcquisitionStatus::Complete {
                assert_eq!(receipt.text.as_deref(), Some("APPROVE actual answer"));
                assert_eq!(
                    status.result.as_ref().unwrap()["schema"],
                    "native_answer_v1"
                );
                assert_eq!(
                    status.result.as_ref().unwrap()["text"],
                    "APPROVE actual answer"
                );
            } else {
                assert!(receipt.text.is_none() && receipt.answer_sha256.is_none());
            }
            // A complete owned receipt preserves native Failure separately from its text.
            // It is never a reviewer/member grant, read-only proof or round certificate.
            if payload == "answer-failed" {
                assert_eq!(status.result.unwrap()["work"], "failure");
            }
        }
    }
}

#[tokio::test]
async fn native_raw_duplicate_and_depth_frames_cannot_mint_complete_content() {
    use crate::execution::native_result::AcquisitionStatus;
    for provider in ["claude", "codex"] {
        for payload in ["answer-raw-duplicate", "answer-depth"] {
            let (dir, owner, task) = results::tests::fixture().await;
            let sessions = NativeSessions::new(owner.clone()).unwrap();
            let (unit, _) = attempts::AttemptManager::new(owner.clone())
                .prepare(task.id, provider, "Implement", None)
                .await
                .unwrap();
            let handle = match sessions
                .start_inner(
                    input(&unit, payload),
                    None,
                    None,
                    Some(program(dir.path(), provider)),
                )
                .await
                .unwrap()
            {
                NativeStart::Launched(handle) => handle,
                _ => panic!("fixture launch"),
            };
            let status = terminal(&sessions, &handle).await;
            assert_eq!(status.work, Some(WorkOutcome::Unknown));
            assert_eq!(status.failure, Some(NativeFailure::ProtocolFailure));
            let receipt = owner
                .store
                .lock()
                .unwrap()
                .native_result(status.receipt.unwrap())
                .unwrap();
            assert_ne!(receipt.acquisition, AcquisitionStatus::Complete);
            assert!(receipt.text.is_none() && receipt.answer_sha256.is_none());
            let wire = receipt
                .wire
                .as_ref()
                .expect("bounded non-content wire evidence");
            assert_eq!(
                wire.category,
                if payload == "answer-depth" {
                    "depth"
                } else {
                    "duplicate_key"
                }
            );
            assert!(wire.exact_length && wire.observed_bytes > 0);
            assert!(!serde_json::to_string(&receipt).unwrap().contains("second"));
        }
    }
}

#[tokio::test]
async fn native_late_draft_preserves_cancellation_and_epoch_fences() {
    use crate::execution::native_result::{InvocationState, ReceiptAuthority};
    for restart in [false, true] {
        let (dir, owner, task) = results::tests::fixture().await;
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, "codex", "Implement", None)
            .await
            .unwrap();
        let handle = match sessions
            .start_inner(
                input(&unit, "answer-hold-after-final"),
                None,
                None,
                Some(program(dir.path(), "codex")),
            )
            .await
            .unwrap()
        {
            NativeStart::Launched(handle) => handle,
            _ => panic!("fixture launch"),
        };
        let output = owner
            .root
            .join("units")
            .join(unit.id.to_string())
            .join("output");
        tokio::time::timeout(Duration::from_secs(5), async {
            while !output.join("fixture-answer-ready").exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        // This owned permission notification follows the final on the same stdio pipe.
        // Its public pending status is the causal barrier that proves final consumption.
        tokio::time::timeout(Duration::from_secs(5), async {
            while !sessions
                .status(&handle)
                .unwrap()
                .pending
                .iter()
                .any(|p| p["id"] == "answer-barrier")
            {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        if restart {
            owner.store.lock().unwrap().begin_execution_epoch().unwrap();
        } else {
            sessions.cancel(&handle).await.unwrap();
        }
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if owner
                    .store
                    .lock()
                    .unwrap()
                    .native_session_invocation(handle.session)
                    .unwrap()
                    .state
                    == InvocationState::Closed
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        let status = terminal(&sessions, &handle).await;
        let store = owner.store.lock().unwrap();
        let receipt = store.native_result(status.receipt.unwrap()).unwrap();
        assert_eq!(receipt.authority, ReceiptAuthority::HistoricalDraft);
        assert!(receipt.text.is_none() && receipt.answer_sha256.is_none());
        assert_eq!(
            receipt.prefix.as_ref().unwrap().prefix,
            "APPROVE actual answer"
        );
        let current = store.execution_unit(unit.id).unwrap();
        assert_eq!(current.work, Some(WorkOutcome::Unknown));
        assert_eq!(
            current.disposition,
            if restart {
                Disposition::Lost
            } else {
                Disposition::Cancelled
            }
        );
        assert!(!current.native_effects_open && !current.result_finalization_open);
        assert_eq!(current.generation, unit.generation);
    }
}

#[tokio::test]
async fn native_invocation_binds_actual_durable_context_and_refuses_altered_input_before_spawn() {
    use crate::domain::ContextVersion;
    for provider in ["claude", "codex"] {
        let (dir, owner, mut task) = results::tests::fixture().await;
        let revision = results::text(
            &results::git(&dir.path().join("repo"), ["rev-parse", "HEAD"])
                .await
                .unwrap(),
        )
        .unwrap();
        let context = ContextVersion {
            scope: task.scope(),
            version: 1,
            revision,
            source_hashes: BTreeMap::from([("fixture:source".into(), "immutable_a".into())]),
            data: json!({"instruction":"committed A"}),
        };
        {
            let mut store = owner.store.lock().unwrap();
            store.put_context(&context).unwrap();
            task.context_version = 1;
            store.put_task(&mut task).unwrap();
        }
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, provider, "Implement", None)
            .await
            .unwrap();
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let path = program(dir.path(), provider);
        let script = std::fs::read_to_string(&path).unwrap().replace(
            "# Local protocol fixture only:",
            "WORKFLOW_SCENARIO = 'answer-ok'\n# Local protocol fixture only:",
        );
        std::fs::write(&path, script).unwrap();
        let mut prepared = input(&unit, &serde_json::to_string(&context.data).unwrap());
        prepared.input.source_versions = context.source_hashes.clone();
        let mut bad = prepared.clone();
        bad.input.payload = "live B".into();
        assert!(
            sessions
                .start_inner(bad, None, None, Some(path.clone()))
                .await
                .is_err()
        );
        let mut bad = prepared.clone();
        bad.input
            .source_versions
            .insert("fixture:source".into(), "live_b".into());
        assert!(
            sessions
                .start_inner(bad, None, None, Some(path.clone()))
                .await
                .is_err()
        );
        {
            let store = owner.store.lock().unwrap();
            assert_eq!(store.execution_unit(unit.id).unwrap().session_id, None);
            assert!(
                store
                    .managed_effects(unit.id)
                    .unwrap()
                    .iter()
                    .all(|e| e.kind != "native_version" && e.kind != "native_input")
            );
        }
        let handle = match sessions
            .start_inner(prepared, None, None, Some(path))
            .await
            .unwrap()
        {
            NativeStart::Launched(h) => h,
            _ => panic!("fixture launch"),
        };
        let status = terminal(&sessions, &handle).await;
        assert_eq!(status.work, Some(WorkOutcome::Success));
        let invoice = owner
            .store
            .lock()
            .unwrap()
            .native_session_invocation(handle.session)
            .unwrap();
        assert_eq!(invoice.context_version, Some(1));
        assert_eq!(
            invoice.context_sha256,
            Some(native_result::digest(
                &serde_json::to_vec(&context).unwrap()
            ))
        );
        assert_eq!(invoice.source_versions, context.source_hashes);
        assert_eq!(
            invoice.payload_sha256,
            native_result::digest(serde_json::to_string(&context.data).unwrap().as_bytes())
        );
        let receipt = owner
            .store
            .lock()
            .unwrap()
            .native_result(status.receipt.unwrap())
            .unwrap();
        let reopened = crate::state::Store::open(&dir.path().join("state.db")).unwrap();
        assert_eq!(
            reopened.native_session_invocation(handle.session).unwrap(),
            invoice
        );
        assert_eq!(reopened.native_result(receipt.id).unwrap(), receipt);
        let (session, version) = owner
            .store
            .lock()
            .unwrap()
            .session(handle.session)
            .unwrap()
            .unwrap();
        // Private actor replay path: identical durable receipt is idempotent, changes refuse.
        // These test-only private constructors do not expose a public minting API.
        let mut proof = NativeTerminal {
            receipt: receipt.clone(),
            session,
            failure: None,
            session_version: version,
        };
        let before = owner.store.lock().unwrap().execution_unit(unit.id).unwrap();
        owner
            .store
            .lock()
            .unwrap()
            .finish_native_result(&proof)
            .unwrap();
        proof.receipt.text = Some("changed second result".into());
        proof.receipt.answer_sha256 = Some(native_result::digest(b"changed second result"));
        assert!(
            owner
                .store
                .lock()
                .unwrap()
                .finish_native_result(&proof)
                .is_err()
        );
        assert_eq!(
            serde_json::to_value(owner.store.lock().unwrap().execution_unit(unit.id).unwrap())
                .unwrap(),
            serde_json::to_value(before).unwrap()
        );
        assert_eq!(reopened.native_result(receipt.id).unwrap(), receipt);
    }
}

#[tokio::test]
async fn native_claude_running_hold_does_not_replace_original_answer_with_later_result() {
    use crate::execution::native_result::AcquisitionStatus;
    for payload in [
        "answer-changing-held",
        "answer-sameuuid-changing-held",
        "answer-identical-held",
    ] {
        let (dir, owner, task) = results::tests::fixture().await;
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, "claude", "Implement", None)
            .await
            .unwrap();
        let handle = match sessions
            .start_inner(
                input(&unit, payload),
                None,
                None,
                Some(program(dir.path(), "claude")),
            )
            .await
            .unwrap()
        {
            NativeStart::Launched(h) => h,
            _ => panic!("fixture launch"),
        };
        let status = terminal(&sessions, &handle).await;
        assert_eq!(status.work, Some(WorkOutcome::Success));
        let receipt = owner
            .store
            .lock()
            .unwrap()
            .native_result(status.receipt.unwrap())
            .unwrap();
        if payload == "answer-identical-held" {
            assert_eq!(receipt.acquisition, AcquisitionStatus::Complete);
            assert_eq!(receipt.text.as_deref(), Some("APPROVE A"));
        } else {
            assert_eq!(receipt.acquisition, AcquisitionStatus::Ambiguous);
            assert!(receipt.text.is_none() && receipt.answer_sha256.is_none());
            assert_eq!(receipt.prefix.unwrap().prefix, "APPROVE A");
        }
    }
}

#[tokio::test]
async fn native_collector_overflow_stops_a_peer_that_never_sends_a_terminal() {
    use crate::execution::native_result::AcquisitionStatus;
    for payload in ["answer-item-overflow", "answer-event-overflow"] {
        let (dir, owner, task) = results::tests::fixture().await;
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, "codex", "Implement", None)
            .await
            .unwrap();
        let handle = match sessions
            .start_inner(
                input(&unit, payload),
                None,
                None,
                Some(program(dir.path(), "codex")),
            )
            .await
            .unwrap()
        {
            NativeStart::Launched(h) => h,
            _ => panic!("fixture launch"),
        };
        let status = tokio::time::timeout(Duration::from_secs(5), terminal(&sessions, &handle))
            .await
            .unwrap();
        assert_eq!(status.work, Some(WorkOutcome::Unknown));
        let store = owner.store.lock().unwrap();
        let receipt = store.native_result(status.receipt.unwrap()).unwrap();
        assert_eq!(receipt.acquisition, AcquisitionStatus::Overflow);
        assert!(!store.execution_unit(unit.id).unwrap().native_effects_open);
        assert!(
            !store
                .execution_is_quota_probe(unit.id, "codex", "unknown")
                .unwrap()
        );
        assert!(matches!(
            store.session(handle.session).unwrap().unwrap().0.state,
            SessionState::Lost
        ));
    }
}

#[tokio::test]
async fn native_transient_receipt_failure_retains_frozen_success_and_reconciles_watch() {
    use crate::execution::native_result::{AcquisitionStatus, ReceiptAuthority};
    let (dir, owner, task) = results::tests::fixture().await;
    let sessions = NativeSessions::new(owner.clone()).unwrap();
    let (unit, _) = attempts::AttemptManager::new(owner.clone())
        .prepare(task.id, "codex", "Implement", None)
        .await
        .unwrap();
    let handle = match sessions
        .start_inner(
            input(&unit, "answer-hold-after-final"),
            None,
            None,
            Some(program(dir.path(), "codex")),
        )
        .await
        .unwrap()
    {
        NativeStart::Launched(h) => h,
        _ => panic!("fixture launch"),
    };
    tokio::time::timeout(Duration::from_secs(5), async {
        while !sessions
            .status(&handle)
            .unwrap()
            .pending
            .iter()
            .any(|p| p["id"] == "answer-barrier")
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let connection = rusqlite::Connection::open(dir.path().join("state.db")).unwrap();
    connection.execute_batch(&format!("CREATE TRIGGER native_receipt_fixture_fault BEFORE INSERT ON native_results WHEN NEW.unit_id='{}' BEGIN SELECT RAISE(ABORT,'synthetic bounded receipt failure'); END;",unit.id)).unwrap();
    let output = owner
        .root
        .join("units")
        .join(unit.id.to_string())
        .join("output");
    std::fs::write(output.join("fixture-release"), "release").unwrap();
    let mut updates = sessions.subscribe(&handle).unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let status = updates.borrow().clone();
            if status.diagnostic == Some("native terminal pending persistence") {
                break;
            }
            updates.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    let held = sessions.status(&handle).unwrap();
    assert_eq!(held.observed_work, Some(WorkOutcome::Success));
    assert_eq!(held.work, None);
    assert!(held.receipt.is_none());
    assert!(sessions.release(&handle).is_err());
    connection
        .execute_batch("DROP TRIGGER native_receipt_fixture_fault")
        .unwrap();
    let restored = sessions.status(&handle).unwrap();
    assert_eq!(restored.work, Some(WorkOutcome::Success));
    let receipt = owner
        .store
        .lock()
        .unwrap()
        .native_result(restored.receipt.unwrap())
        .unwrap();
    assert_eq!(receipt.observed_work, WorkOutcome::Success);
    assert_eq!(receipt.acquisition, AcquisitionStatus::Complete);
    assert_eq!(receipt.authority, ReceiptAuthority::OwnedTerminal);
    assert_eq!(receipt.text.as_deref(), Some("APPROVE actual answer"));
    assert_eq!(updates.borrow().work, Some(WorkOutcome::Success));
    assert_eq!(updates.borrow().receipt, restored.receipt);
    sessions.release(&handle).unwrap();
}

#[tokio::test]
async fn native_preinput_context_supersession_refuses_before_input_effect() {
    use crate::domain::ContextVersion;
    for durable in [false, true] {
        let (dir, owner, mut task) = results::tests::fixture().await;
        let revision = results::text(
            &results::git(&dir.path().join("repo"), ["rev-parse", "HEAD"])
                .await
                .unwrap(),
        )
        .unwrap();
        let original = ContextVersion {
            scope: task.scope(),
            version: 1,
            revision: revision.clone(),
            source_hashes: BTreeMap::from([("source".into(), "A".into())]),
            data: json!({"instruction":"A"}),
        };
        if durable {
            let mut store = owner.store.lock().unwrap();
            store.put_context(&original).unwrap();
            task.context_version = 1;
            store.put_task(&mut task).unwrap();
        }
        let (unit, _) = attempts::AttemptManager::new(owner.clone())
            .prepare(task.id, "codex", "Implement", None)
            .await
            .unwrap();
        let sessions = NativeSessions::new(owner.clone()).unwrap();
        let path = program(dir.path(), "codex");
        let script=std::fs::read_to_string(&path).unwrap().replace("def hold_bootstrap():","BOOTSTRAP_HOLD = True\n\ndef hold_bootstrap():").replacen("        while True: time.sleep(0.02)","        while not os.path.exists(os.path.join(os.environ[\"RRX_OUTPUT_DIR\"],\"fixture-bootstrap-release\")): time.sleep(0.02)",1);
        std::fs::write(&path, script).unwrap();
        let mut prepared = input(&unit, if durable { "" } else { "answer-ok" });
        if durable {
            prepared.input.payload = serde_json::to_string(&original.data).unwrap();
            prepared.input.source_versions = original.source_hashes.clone();
        }
        let handle = match sessions
            .start_inner(prepared, None, None, Some(path))
            .await
            .unwrap()
        {
            NativeStart::Launched(h) => h,
            _ => panic!("fixture launch"),
        };
        let output = owner
            .root
            .join("units")
            .join(unit.id.to_string())
            .join("output");
        tokio::time::timeout(Duration::from_secs(5), async {
            while !output.join("fixture-bootstrap-ready").exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        {
            let mut store = owner.store.lock().unwrap();
            let newer = ContextVersion {
                scope: task.scope(),
                version: if durable { 2 } else { 1 },
                revision,
                source_hashes: BTreeMap::from([("source".into(), "B".into())]),
                data: json!({"instruction":"B"}),
            };
            store.put_context(&newer).unwrap();
            let mut current = store.task(task.id).unwrap().unwrap();
            current.context_version = newer.version;
            store.put_task(&mut current).unwrap();
        }
        std::fs::write(output.join("fixture-bootstrap-release"), "release").unwrap();
        let status = terminal(&sessions, &handle).await;
        assert_eq!(status.work, Some(WorkOutcome::Unknown));
        let store = owner.store.lock().unwrap();
        let invocation = store.native_session_invocation(handle.session).unwrap();
        assert!(
            invocation.input_operation.is_none(),
            "superseded input was journalled/written"
        );
        assert!(
            store
                .managed_effects(unit.id)
                .unwrap()
                .iter()
                .all(|e| e.kind != "native_input")
        );
        assert_eq!(
            store.task(task.id).unwrap().unwrap().context_version,
            if durable { 2 } else { 1 }
        );
    }
}

#[tokio::test]
async fn native_concurrent_terminal_retry_survives_supervisor_watch_publication() {
    use crate::execution::native_result::{AcquisitionStatus, ReceiptAuthority};
    let (dir, owner, task) = results::tests::fixture().await;
    let sessions = NativeSessions::new(owner.clone()).unwrap();
    let (unit, _) = attempts::AttemptManager::new(owner.clone())
        .prepare(task.id, "codex", "Implement", None)
        .await
        .unwrap();
    let handle = match sessions
        .start_inner(
            input(&unit, "answer-hold-after-final"),
            None,
            None,
            Some(program(dir.path(), "codex")),
        )
        .await
        .unwrap()
    {
        NativeStart::Launched(h) => h,
        _ => panic!("fixture launch"),
    };
    tokio::time::timeout(Duration::from_secs(5), async {
        while !sessions
            .status(&handle)
            .unwrap()
            .pending
            .iter()
            .any(|p| p["id"] == "answer-barrier")
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let connection = rusqlite::Connection::open(dir.path().join("state.db")).unwrap();
    connection.execute_batch(&format!("CREATE TRIGGER native_receipt_fixture_fault BEFORE INSERT ON native_results WHEN NEW.unit_id='{}' BEGIN SELECT RAISE(ABORT,'synthetic bounded receipt failure'); END;",unit.id)).unwrap();
    let output = owner
        .root
        .join("units")
        .join(unit.id.to_string())
        .join("output");
    std::fs::write(output.join("fixture-delay-publication"), "delay").unwrap();
    std::fs::write(output.join("fixture-release"), "release").unwrap();
    let mut updates = sessions.subscribe(&handle).unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while !output.join("fixture-publication-ready").exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let held = sessions.status(&handle).unwrap();
    assert_eq!(held.observed_work, Some(WorkOutcome::Success));
    assert_eq!(held.work, None);
    assert!(held.receipt.is_none());
    assert!(sessions.release(&handle).is_err());
    connection
        .execute_batch("DROP TRIGGER native_receipt_fixture_fault")
        .unwrap();
    let restored = sessions.status(&handle).unwrap();
    assert_eq!(restored.work, Some(WorkOutcome::Success));
    let receipt = owner
        .store
        .lock()
        .unwrap()
        .native_result(restored.receipt.unwrap())
        .unwrap();
    assert_eq!(receipt.observed_work, WorkOutcome::Success);
    assert_eq!(receipt.acquisition, AcquisitionStatus::Complete);
    assert_eq!(receipt.authority, ReceiptAuthority::OwnedTerminal);
    assert_eq!(receipt.text.as_deref(), Some("APPROVE actual answer"));
    assert_eq!(updates.borrow().work, Some(WorkOutcome::Success));
    assert_eq!(updates.borrow().receipt, restored.receipt);
    assert_eq!(updates.borrow().session.state, SessionState::Exited);
    // Consume the status retry's watch update. The next observation must come
    // from the actual supervisor's publication/Drop, without another status read.
    updates.borrow_and_update();
    std::fs::write(output.join("fixture-publication-release"), "release").unwrap();
    tokio::time::timeout(Duration::from_secs(5), updates.changed())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updates.borrow().work, Some(WorkOutcome::Success));
    assert_eq!(updates.borrow().receipt, restored.receipt);
    assert_eq!(updates.borrow().session.state, SessionState::Exited);
    assert_eq!(
        updates.borrow().result.as_ref().unwrap()["text"],
        "APPROVE actual answer"
    );
    assert_ne!(
        updates.borrow().diagnostic,
        Some("terminal persistence conflict")
    );
    sessions.release(&handle).unwrap();
}

mod terminal_lock_tests;
