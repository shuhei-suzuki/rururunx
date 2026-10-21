use super::*;
use std::os::unix::fs::PermissionsExt;

/// Read-only test leftovers remain visible maintenance while the Task run completes.
#[tokio::test]
async fn b6_f5_cleanup_never_holds() {
    let code = "for my $d ('blocked', \"$ENV{TMPDIR}/blocked\") { mkdir $d or die; open(my $f, '>', \"$d/keep\") or die; close $f; chmod 0555, $d or die; } exit 0;";
    let (f, runtime, id) = planned_test(code, 10, 1024, 1024).await;
    finish(&f, &runtime, &id, true).await;
    let step = f
        .store()
        .pipeline_goal_view(f.goal)
        .unwrap()
        .steps
        .last()
        .unwrap()
        .clone();
    let checkout =
        PipelineTestRunner::checkout(f.config.pipeline.as_ref().unwrap(), &step).unwrap();
    let tmp = std::path::PathBuf::from(step.body["intent"]["tmpdir"].as_str().unwrap());
    assert!(adopt_ready(&f, &runtime, f.cursor(&id)).await.unwrap());
    runtime.sweep_pipeline_test_cleanup().await.unwrap();
    // Wait for the sweep's first filesystem observation, not a fixed launch delay.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let view = f.store().pipeline_goal_view(f.goal).unwrap();
        assert_eq!(view.runs[0].state, "running");
        if view
            .pending_cleanup
            .iter()
            .any(|c| c.failure_class.as_deref() == Some("checkout_remove"))
        {
            let pending = &view.pending_cleanup[0];
            assert_eq!(pending.path.as_deref(), checkout.to_str());
            break;
        }
        assert!(tokio::time::Instant::now() < deadline);
        tokio::task::yield_now().await;
    }
    let run = f.store().pipeline_goal_view(f.goal).unwrap().runs[0].clone();
    let mut next = planned(&id, 3, 1, "release");
    next.step_base = run.head_commit.clone();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap();
    let mut journal = release_journal(&f, &id);
    journal.old_commit = run.head_commit.clone();
    journal.new_commit = run.head_commit;
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_release_begin(&f.owner, &cursor, &journal, &json!({"approved":true}))
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_release_settle(&f.owner, &cursor, &journal.id, 1, &journal.new_commit)
        .unwrap();
    let before = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(before.runs[0].state, "completed");
    assert!(!before.pending_cleanup.is_empty());
    let mut successor = before.runs[0].clone();
    successor.id = Uuid::new_v4().to_string();
    successor.kind = "goal".into();
    successor.task_id = None;
    successor.worktree = None;
    successor.branch = None;
    successor.version = 0;
    successor.state = "running".into();
    successor.body = json!({});
    let mut first = planned(&successor.id, 1, 1, "test");
    first.step_base = successor.head_commit.clone();
    let goal_version = f.store().goal(f.goal).unwrap().unwrap().version;
    f.store()
        .pipeline_claim_run(&f.owner, &successor, goal_version, &first, None)
        .unwrap();
    let before = f.store().pipeline_goal_view(f.goal).unwrap();
    for path in [checkout.join("blocked"), tmp.join("blocked")] {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    r4fix::wait_clean(&f.owner, f.goal, &id, &checkout, Some(&tmp)).await;
    let after = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(after.runs, before.runs);
    assert_eq!(after.steps, before.steps);
    runtime.shutdown().await.unwrap();
}

/// A terminal run cannot reenter held even with legacy cleanup fields.
#[tokio::test]
async fn b6_f5_terminal_cannot_hold() {
    use super::super::super::round2::{rejected, update};
    let f = fixture().await;
    let id = complete_task(&f);
    let c = &f.store().connection;
    // Legacy cleanup data must not reopen the state transition.
    c.execute("UPDATE pipeline_runs SET version=version+1,body=json_set(body,'$.version',version+1,'$.test_cleanup',json('{}')) WHERE id=?1", [&id]).unwrap();
    rejected(
        c,
        &update(
            "pipeline_runs",
            ",state='held',body=json_set(body,'$.version',version+1,'$.state','held','$.held.reason','test_cleanup_failed','$.cleanup_terminal_state','completed')",
        ),
        "invalid pipeline run transition",
    );
}

/// Cleanup does not sweep live runs or invalidate Agent and test monitor cursors.
#[tokio::test]
async fn b6_f5_live_step_cursor() {
    for kind in ["test", "review"] {
        let f = fixture().await;
        let id = f.claim();
        f.launching(&id, "test");
        let cursor = f.cursor(&id);
        f.store()
            .pipeline_record_exit(&f.owner, &cursor, &death(), None, &observed().body["exit"])
            .unwrap();
        let cursor = f.cursor(&id);
        f.store()
            .pipeline_adopt(
                &f.owner,
                &cursor,
                &json!({"command_id":"test","result":"pass"}),
            )
            .unwrap();
        let mut next = planned(&id, 3, 1, kind);
        next.step_base = f.store().pipeline_goal_view(f.goal).unwrap().runs[0]
            .head_commit
            .clone();
        let cursor = f.cursor(&id);
        f.store()
            .pipeline_plan_next(&f.owner, &cursor, &next, true)
            .unwrap();
        let cursor = f.cursor(&id);
        f.store()
            .pipeline_begin_launch(
                &f.owner,
                &cursor,
                Some("f5-live-cookie-123456"),
                Some(&rrx_process_tracker::launch_mark().unwrap()),
                &json!({}),
            )
            .unwrap();
        let cursor = f.cursor(&id);
        let before = f.store().pipeline_goal_view(f.goal).unwrap();
        assert_eq!(before.pending_cleanup.len(), 1);
        assert!(
            f.store()
                .pipeline_test_cleanup(&f.owner, &id)
                .unwrap()
                .is_empty()
        );
        f.store()
            .pipeline_record_running(&f.owner, &cursor, 4242, "fixture-birth")
            .unwrap();
        let cursor = f.cursor(&id);
        assert!(
            f.store()
                .pipeline_test_cleanup(&f.owner, &id)
                .unwrap()
                .is_empty()
        );
        assert_eq!(f.cursor(&id).run_version, cursor.run_version);
        f.store()
            .pipeline_record_exit(
                &f.owner,
                &cursor,
                &death(),
                (kind == "review").then_some("success"),
                &observed().body["exit"],
            )
            .unwrap();
        assert_eq!(
            f.store()
                .pipeline_goal_view(f.goal)
                .unwrap()
                .steps
                .last()
                .unwrap()
                .state,
            "exited"
        );
        // Even maintenance bookkeeping outside a live interval cannot change a cursor.
        let obligation = before.pending_cleanup[0].clone();
        let cursor = f.cursor(&id);
        f.store()
            .pipeline_test_cleanup_result(
                &f.owner,
                &obligation,
                Some(("checkout_remove", std::path::Path::new("/derived/fixture"))),
            )
            .unwrap();
        assert_eq!(f.cursor(&id).run_version, cursor.run_version);
    }
}

/// A later commit's final adoption preserves all earlier cleanup obligations.
#[tokio::test]
async fn b6_f5_cleanup_cumulative() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "test");
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(&f.owner, &cursor, &death(), None, &observed().body["exit"])
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_adopt(
            &f.owner,
            &cursor,
            &json!({"command_id":"test","result":"pass"}),
        )
        .unwrap();
    let prior = f
        .store()
        .pipeline_goal_view(f.goal)
        .unwrap()
        .pending_cleanup;
    let cursor = f.cursor(&id);
    let mut fix = planned(&id, 3, 1, "fix");
    fix.round = 1;
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &fix, true)
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_begin_launch(
            &f.owner,
            &cursor,
            Some("f5-fix-cookie-1234567"),
            Some(&rrx_process_tracker::launch_mark().unwrap()),
            &json!({}),
        )
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(
            &f.owner,
            &cursor,
            &death(),
            Some("success"),
            &json!({"issue":null}),
        )
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
    let mut next = planned(&id, 4, 1, "test");
    next.round = 1;
    next.step_base = "b".repeat(40);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_begin_launch(
            &f.owner,
            &cursor,
            Some("f5-test-cookie-123456"),
            Some(&rrx_process_tracker::launch_mark().unwrap()),
            &json!({}),
        )
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(&f.owner, &cursor, &death(), None, &observed().body["exit"])
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_adopt(
            &f.owner,
            &cursor,
            &json!({"command_id":"test","result":"pass"}),
        )
        .unwrap();
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_ne!(
        view.steps[1].step_base,
        view.steps.last().unwrap().step_base
    );
    assert_eq!(view.pending_cleanup.len(), 2);
    assert_eq!(view.pending_cleanup[0], prior[0]);
}

/// Cleanup keeps unrelated missing worktree registrations and removes only its own.
#[tokio::test]
async fn b6_f5_registration_isolated() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    let prepared = r3fix::prepare_again(&f, &id).await;
    let project = f.store().project(f.project).unwrap().unwrap();
    let git = PipelineGit::new(&project.root).unwrap();
    let unrelated = f._dir.path().join("unrelated");
    git.run(&[
        "worktree",
        "add",
        "--detach",
        unrelated.to_str().unwrap(),
        "HEAD",
    ])
    .await
    .unwrap();
    let moved = unrelated.with_file_name("moved-unrelated");
    std::fs::rename(&unrelated, &moved).unwrap();
    let common = git
        .text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .await
        .unwrap();
    let admin = std::path::Path::new(&common).join("worktrees/unrelated");
    let index = std::fs::read(admin.join("index")).unwrap();
    let step = f
        .store()
        .pipeline_goal_view(f.goal)
        .unwrap()
        .steps
        .last()
        .unwrap()
        .clone();
    PipelineTestRunner::remove(&project.root, f.config.pipeline.as_ref().unwrap(), &step)
        .await
        .unwrap();
    assert_eq!(std::fs::read(admin.join("index")).unwrap(), index);
    let listing = git
        .text(&["worktree", "list", "--porcelain"])
        .await
        .unwrap();
    assert!(listing.contains(unrelated.to_str().unwrap()));
    assert!(!listing.contains(prepared.checkout.to_str().unwrap()));
    std::fs::rename(moved, unrelated).unwrap();
    runtime.shutdown().await.unwrap();
}

/// An unreadable run's cleanup inventory cannot stop another run's sweep or the service.
#[tokio::test]
async fn b6_f5_sweep_continues() {
    let f = fixture().await;
    let runtime = Arc::new(Runtime::new(f.owner.clone(), f.config.clone()).unwrap());
    let other = fixture_with_service(
        true,
        true,
        Some(f.owner.clone()),
        Some((&runtime, &f.config)),
    )
    .await;
    let parent = f.config.pipeline.as_ref().unwrap().work_root.join("co");
    std::fs::create_dir(&parent).unwrap();
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700)).unwrap();
    for fixture in [&f, &other] {
        let project = fixture.store().project(fixture.project).unwrap().unwrap();
        PipelineGit::new(&project.root)
            .unwrap()
            .run(&["init", "-b", "main"])
            .await
            .unwrap();
    }
    let mut runs = Vec::new();
    for fixture in [&f, &other] {
        let id = fixture.claim();
        fixture.launching(&id, "test");
        let cursor = fixture.cursor(&id);
        fixture
            .store()
            .pipeline_record_exit(
                &fixture.owner,
                &cursor,
                &death(),
                None,
                &observed().body["exit"],
            )
            .unwrap();
        let cursor = fixture.cursor(&id);
        fixture
            .store()
            .pipeline_adopt(
                &fixture.owner,
                &cursor,
                &json!({"command_id":"test","result":"pass"}),
            )
            .unwrap();
        runs.push((id, fixture));
    }
    runs.sort_by(|a, b| a.0.cmp(&b.0));
    let (bad_id, bad) = &runs[0];
    let (good_id, good) = &runs[1];
    runtime.start_qualified_service().await.unwrap();
    let denied = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed = denied.clone();
    // A one-shot denial injects a real Store read error without changing persisted data.
    f.store()
        .connection
        .authorizer(Some(move |ctx: AuthContext<'_>| {
            if matches!(
                ctx.action,
                AuthAction::Read {
                    table_name: "projects",
                    column_name: "body"
                }
            ) && !observed.swap(true, std::sync::atomic::Ordering::SeqCst)
            {
                Authorization::Deny
            } else {
                Authorization::Allow
            }
        }));
    runtime.sweep_pipeline_test_cleanup().await.unwrap();
    assert!(
        f.store()
            .pipeline_goal_view(bad.goal)
            .unwrap()
            .pending_cleanup
            .iter()
            .any(|c| c.run_id == *bad_id)
    );
    assert!(
        !f.store()
            .pipeline_goal_view(good.goal)
            .unwrap()
            .pending_cleanup
            .iter()
            .any(|c| c.run_id == *good_id)
    );
    assert!(denied.load(std::sync::atomic::Ordering::SeqCst));
    let failed = f
        .store()
        .pipeline_goal_view(bad.goal)
        .unwrap()
        .pending_cleanup
        .remove(0);
    assert_eq!(failed.failure_class.as_deref(), Some("cleanup_sweep"));
    assert!(failed.not_before > 0);
    assert!(
        failed
            .path
            .as_deref()
            .unwrap()
            .ends_with(&format!("{bad_id}-2-1"))
    );
    runtime.wait_pipeline_test_boundary().await;
    f.store()
        .connection
        .authorizer(None::<fn(AuthContext<'_>) -> Authorization>);
    runtime.shutdown().await.unwrap();
}

/// Pipeline daemon exec drops proxy and Node inputs from a contaminated CLI parent.
#[tokio::test]
async fn b6_f5_daemon_minimal_environment() {
    use crate::cli::daemon::{OwnerLock, StopOutcome, environment, probe_lock, stop};
    use crate::config::ExecutionMode;
    use std::process::{Command, Stdio};
    let f = fixture().await;
    let state = f.owner.state_path().to_path_buf();
    let work_root = f.config.pipeline.as_ref().unwrap().work_root.clone();
    let config = f._dir.path().join("daemon.toml");
    std::fs::write(&config, toml::to_string(&f.config).unwrap()).unwrap();
    let binary = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("rrx");
    assert!(binary.is_file(), "build the rrx binary before host cases");
    drop(f.ingress);
    let released = Arc::downgrade(&f.owner);
    drop(f.owner);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    while released.upgrade().is_some() {
        assert!(
            tokio::time::Instant::now() < deadline,
            "fixture retained RuntimeOwner"
        );
        tokio::task::yield_now().await;
    }
    assert_eq!(
        probe_lock(&state),
        OwnerLock::Free,
        "fixture owner lock retained"
    );
    struct Stop {
        binary: std::path::PathBuf,
        state: std::path::PathBuf,
    }
    impl Drop for Stop {
        fn drop(&mut self) {
            let mut command = Command::new(&self.binary);
            command
                .env_clear()
                .envs(environment(ExecutionMode::CliPipeline))
                .arg("--state")
                .arg(&self.state)
                .args(["daemon", "stop"])
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            let _ = rrx_spawn_hygiene::status_std(command);
        }
    }
    let _stop = Stop {
        binary: binary.clone(),
        state: state.clone(),
    };
    let output = rrx_spawn_hygiene::output_std({
        let mut command = Command::new(&binary);
        command
            .env_clear()
            .envs(environment(ExecutionMode::ManagedNative))
            .env("HTTPS_PROXY", "http://nonsecret-fixture.invalid")
            .env("NODE_OPTIONS", "--no-warnings")
            .arg("--state")
            .arg(&state)
            .arg("--config")
            .arg(&config)
            .args(["daemon", "start", "--json"]);
        command
    })
    .unwrap();
    let diagnostics = format!(
        "fixture daemon stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "pipeline daemon start failed: {diagnostics}"
    );
    let value: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("invalid daemon response: {error}; {diagnostics}"));
    assert_eq!(value["start"], "running", "{diagnostics}");
    let output = rrx_spawn_hygiene::output_std({
        let mut command = Command::new("/usr/bin/pgrep");
        command.args(["-f", &format!("{} --config", state.display())]);
        command
    })
    .unwrap();
    assert!(output.status.success());
    let pids: Vec<u32> = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .map(|p| p.parse().unwrap())
        .collect();
    assert_eq!(pids.len(), 1);
    let output = rrx_spawn_hygiene::output_std({
        let mut command = Command::new("/bin/ps");
        command.args(["eww", "-p", &pids[0].to_string(), "-o", "command="]);
        command
    })
    .unwrap();
    assert!(output.status.success());
    let env = String::from_utf8_lossy(&output.stdout);
    assert!(env.contains("HOME="));
    assert!(!env.contains("HTTPS_PROXY="));
    assert!(!env.contains("NODE_OPTIONS="));
    assert!(matches!(stop(&state).await, StopOutcome::Completed { .. }));
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    while probe_lock(&state) != OwnerLock::Free {
        assert!(tokio::time::Instant::now() < deadline);
        tokio::task::yield_now().await;
    }
    std::fs::remove_dir_all(work_root).unwrap();
}

/// Failed obligations back off independently up to a bounded retry interval.
#[tokio::test]
async fn b6_f5_cleanup_backoff() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "test");
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(&f.owner, &cursor, &death(), None, &observed().body["exit"])
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_adopt(
            &f.owner,
            &cursor,
            &json!({"command_id":"test","result":"pass"}),
        )
        .unwrap();
    let mut now = 1_000;
    let mut prior = 0;
    for _ in 0..40 {
        let pending = f
            .store()
            .pipeline_goal_view(f.goal)
            .unwrap()
            .pending_cleanup
            .remove(0);
        with_pipeline_clock(now, || {
            f.store().pipeline_test_cleanup_result(
                &f.owner,
                &pending,
                Some(("tmpdir_remove", std::path::Path::new("/derived/fixture"))),
            )
        })
        .unwrap();
        let next = f
            .store()
            .pipeline_goal_view(f.goal)
            .unwrap()
            .pending_cleanup
            .remove(0);
        let interval = next.not_before - now;
        assert!(interval > 0 && interval <= 300_000 && interval >= prior);
        assert!(
            with_pipeline_clock(next.not_before - 1, || f
                .store()
                .pipeline_test_cleanup(&f.owner, &id))
            .unwrap()
            .is_empty()
        );
        assert_eq!(
            with_pipeline_clock(next.not_before, || f
                .store()
                .pipeline_test_cleanup(&f.owner, &id))
            .unwrap()
            .len(),
            1
        );
        now = next.not_before;
        prior = interval;
    }
    assert_eq!(f.cursor(&id).run_version, cursor.run_version);
}

/// Cleanup retains every administrative registration that fails directory or gitdir identity checks.
#[tokio::test]
async fn b6_f5_registration_mismatch() {
    use std::os::unix::fs::symlink;
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    let prepared = r3fix::prepare_again(&f, &id).await;
    let project = f.store().project(f.project).unwrap().unwrap();
    let config = f.config.pipeline.as_ref().unwrap();
    let step = f
        .store()
        .pipeline_goal_view(f.goal)
        .unwrap()
        .steps
        .last()
        .unwrap()
        .clone();
    let text = std::fs::read_to_string(prepared.checkout.join(".git")).unwrap();
    let admin = std::path::PathBuf::from(text.trim().strip_prefix("gitdir: ").unwrap());
    let parked = admin.with_file_name("parked-registration");
    std::fs::rename(&admin, &parked).unwrap();
    symlink(&parked, &admin).unwrap();
    assert!(
        PipelineTestRunner::remove(&project.root, config, &step)
            .await
            .is_err()
    );
    assert!(parked.join("index").is_file());
    assert!(
        std::fs::symlink_metadata(&admin)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    std::fs::remove_file(&admin).unwrap();
    std::fs::rename(parked, &admin).unwrap();
    let gitdir = admin.join("gitdir");
    let original = std::fs::read(&gitdir).unwrap();
    let outside = f._dir.path().join("registration-sentinel");
    std::fs::write(&outside, &original).unwrap();
    std::fs::remove_file(&gitdir).unwrap();
    symlink(&outside, &gitdir).unwrap();
    assert!(
        PipelineTestRunner::remove(&project.root, config, &step)
            .await
            .is_err()
    );
    assert!(admin.is_dir());
    assert_eq!(std::fs::read(&outside).unwrap(), original);
    std::fs::remove_file(&gitdir).unwrap();
    std::fs::write(&gitdir, b"/unrelated/checkout/.git\n").unwrap();
    assert!(
        PipelineTestRunner::remove(&project.root, config, &step)
            .await
            .is_err()
    );
    assert!(admin.is_dir());
    std::fs::write(gitdir, original).unwrap();
    PipelineTestRunner::remove(&project.root, config, &step)
        .await
        .unwrap();
    assert!(!admin.exists());
    runtime.shutdown().await.unwrap();
}

/// Cancellation atomically retains a never-launched test's possible checkout obligation.
#[tokio::test]
async fn b6_f5_cancel_obligation() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "test");
    super::super::super::fix1::cancel(&f);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_cancel_run(&f.owner, &cursor, None)
        .unwrap();
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(view.runs[0].state, "abandoned");
    assert_eq!(view.pending_cleanup.len(), 1);
    assert_eq!(
        (view.pending_cleanup[0].seq, view.pending_cleanup[0].attempt),
        (2, 1)
    );
    assert_eq!(view.steps.last().unwrap().body["death_confirmed"], true);
}
