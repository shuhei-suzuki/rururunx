use super::*;

/// SQL preserves the admitted command index on every step update.
#[tokio::test]
async fn b6_f1_command_index_sql() {
    use super::super::super::round2::{permitted, rejected, update};
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "test");
    for change in [
        "json_set(body,'$.command_index',1)",
        "json_remove(body,'$.command_index')",
        "json_set(body,'$.command_index',null)",
    ] {
        rejected(
            &f.store().connection,
            &update(
                "pipeline_steps",
                &format!(",body=json_set({change},'$.version',version+1)"),
            ),
            "invalid pipeline step transition",
        );
    }
    permitted(&f.store().connection, &update("pipeline_steps", ""));
}

/// A checkout with a forged gitdir cannot be reused even with the same common repository.
#[tokio::test]
async fn b6_f1_forged_gitdir() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    let cursor = f.cursor(&id);
    let profile = f
        .store()
        .pipeline_test_context(&f.owner, &cursor)
        .unwrap()
        .unwrap();
    let (run, step) = f
        .store()
        .pipeline_launch_snapshot(&f.owner, &cursor)
        .unwrap();
    let config = f.config.pipeline.as_ref().unwrap();
    let prepared = PipelineTestRunner::prepare(&f.owner, config, &run, &step, profile.clone())
        .await
        .unwrap();
    let gitfile = prepared.checkout.join(".git");
    let original = std::fs::read_to_string(&gitfile).unwrap();
    let real = std::path::PathBuf::from(original.trim().strip_prefix("gitdir: ").unwrap());
    let forged = prepared.checkout.join("forged");
    std::fs::create_dir(&forged).unwrap();
    for name in ["HEAD", "index"] {
        std::fs::copy(real.join(name), forged.join(name)).unwrap();
    }
    let common = real.parent().unwrap().parent().unwrap();
    std::fs::write(forged.join("commondir"), common.to_str().unwrap()).unwrap();
    std::fs::write(forged.join("gitdir"), gitfile.to_str().unwrap()).unwrap();
    std::fs::write(&gitfile, format!("gitdir: {}\n", forged.display())).unwrap();
    let error = PipelineTestRunner::prepare(&f.owner, config, &run, &step, profile)
        .await
        .err()
        .expect("forged checkout reused");
    assert!(error.to_string().contains("gitfile changed"), "{error:#}");
    std::fs::write(gitfile, original).unwrap();
    let project = f.store().project(f.project).unwrap().unwrap();
    PipelineTestRunner::remove(&project.root, config, &step)
        .await
        .unwrap();
    runtime.shutdown().await.unwrap();
}

/// Committed adoption retains a cleanup obligation that the same entry point can finish.
#[tokio::test]
async fn b6_f1_cleanup_after_adoption() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    finish(&f, &runtime, &id, true).await;
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_adopt(
            &f.owner,
            &cursor,
            &json!({"command_id":"test","result":"pass"}),
        )
        .unwrap();
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let receipt = view.steps.last().unwrap().clone();
    let checkout =
        PipelineTestRunner::checkout(f.config.pipeline.as_ref().unwrap(), &receipt).unwrap();
    assert!(checkout.is_dir());
    assert!(view.runs[0].body["test_cleanup"].is_object());
    assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    let after = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(after.steps.last().unwrap(), &receipt);
    assert!(after.runs[0].body["test_cleanup"].is_null());
    assert!(!checkout.exists());
    runtime.shutdown().await.unwrap();
}

/// A batch with a rewritten checkout gitfile completes without trusting that gitfile.
#[tokio::test]
async fn b6_f1_cleanup_partial_batch() {
    let mut commands = proposal("/usr/bin/perl".into()).commands;
    commands[0].args = vec![
        "-e".into(),
        "while (!-e 'go') { select undef,undef,undef,0.01; } exit 0;".into(),
    ];
    let mut second = commands[0].clone();
    second.id = "second".into();
    commands.push(second);
    let f = fixture_with_commands(true, true, None, None, false, Some(commands)).await;
    let (f, runtime, id) = super::super::super::b5::prepare_fixture_from(f, true).await;
    assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    let head = f.store().pipeline_goal_view(f.goal).unwrap().runs[0]
        .head_commit
        .clone();
    let mut first = planned(&id, 2, 1, "test");
    first.step_base = head.clone();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &first, true)
        .unwrap();
    finish(&f, &runtime, &id, true).await;
    assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    let mut second = planned(&id, 3, 1, "test");
    second.step_base = head;
    second.body["command_index"] = json!(1);
    second.body["command_id"] = json!("second");
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &second, true)
        .unwrap();
    finish(&f, &runtime, &id, true).await;
    let config = f.config.pipeline.as_ref().unwrap();
    let first_co = PipelineTestRunner::checkout(config, &first).unwrap();
    let second_co = PipelineTestRunner::checkout(config, &second).unwrap();
    let gitfile = second_co.join(".git");
    std::fs::write(&gitfile, b"gitdir: /nonexistent\n").unwrap();
    assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    let partial = f.store().pipeline_goal_view(f.goal).unwrap();
    let receipt = partial.steps.last().unwrap().clone();
    assert_eq!(receipt.state, "adopted");
    assert!(!first_co.exists());
    assert!(!second_co.exists());
    assert!(partial.runs[0].body["test_cleanup"].is_null());
    let after = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(after.steps.last().unwrap(), &receipt);
    assert!(after.runs[0].body["test_cleanup"].is_null());
    assert!(!second_co.exists());
    let project = f.store().project(f.project).unwrap().unwrap();
    let listing = PipelineGit::new(&project.root)
        .unwrap()
        .text(&["worktree", "list", "--porcelain"])
        .await
        .unwrap();
    assert!(!listing.contains(first_co.to_str().unwrap()));
    assert!(!listing.contains(second_co.to_str().unwrap()));
    runtime.shutdown().await.unwrap();
}

/// Terminal cleanup remains in the takeover inventory and needs no Goal advancement.
#[tokio::test]
async fn b6_f1_terminal_cleanup() {
    let (f, runtime, id) = goal_exited("exit 0;").await;
    let cursor = goal_cursor(&f, &id);
    let version = f.store().goal(f.goal).unwrap().unwrap().version;
    f.store()
        .pipeline_complete_goal(&f.owner, &cursor, version, "test")
        .unwrap();
    let inventory = f.store().pipeline_takeover_view().unwrap();
    assert!(
        inventory
            .runs
            .iter()
            .any(|r| r.id == id && r.state == "completed" && r.body["test_cleanup"].is_object())
    );
    assert!(
        runtime
            .adopt_pipeline_test(goal_cursor(&f, &id))
            .await
            .unwrap()
    );
    assert!(
        f.store()
            .pipeline_goal_view(f.goal)
            .unwrap()
            .runs
            .iter()
            .find(|r| r.id == id)
            .unwrap()
            .body["test_cleanup"]
            .is_null()
    );
    assert_eq!(
        f.store().goal(f.goal).unwrap().unwrap().state,
        GoalState::Completed
    );
    runtime.shutdown().await.unwrap();
}

/// Checkout preparation wires all CLI control paths into the Seatbelt forbidden set.
#[tokio::test]
async fn b6_f1_prepare_control_paths() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    let cursor = f.cursor(&id);
    let profile = f
        .store()
        .pipeline_test_context(&f.owner, &cursor)
        .unwrap()
        .unwrap();
    let (run, step) = f
        .store()
        .pipeline_launch_snapshot(&f.owner, &cursor)
        .unwrap();
    let config = f.config.pipeline.as_ref().unwrap();
    let prepared = PipelineTestRunner::prepare(&f.owner, config, &run, &step, profile)
        .await
        .unwrap();
    for path in config.test_protected_paths().unwrap() {
        assert!(prepared.forbidden.contains(&path), "{path:?}");
    }
    runtime.shutdown().await.unwrap();
}

/// Admission refuses executable programs in every pipeline writable namespace.
#[tokio::test]
async fn b6_f1_admission_writable_program() {
    use std::os::unix::fs::PermissionsExt;
    let f = fixture().await;
    let mut config = f.config.clone();
    let pipeline = config.pipeline.as_mut().unwrap();
    let cache = pipeline
        .work_root
        .with_file_name(format!("cache-{}", uuid::Uuid::new_v4()));
    // Keep the fixture cache in the fixture's owned private root.
    let cache = f.owner.root.join(cache.file_name().unwrap());
    std::fs::create_dir(&cache).unwrap();
    pipeline.test_cache_dirs = vec![cache];
    let forbidden = [
        pipeline.work_root.clone(),
        pipeline.test_cache_dirs[0].clone(),
        pipeline.clis.codex.codex_home.clone().unwrap(),
    ];
    let sources = Arc::new(
        crate::execution::workflow_source::ManagedWorkflowSources::new(f.owner.clone(), config)
            .unwrap(),
    );
    let verifier =
        crate::execution::verification::ManagedVerifier::new(f.owner.clone(), sources).unwrap();
    let version = f.store().project(f.project).unwrap().unwrap().version;
    for namespace in forbidden {
        let program = namespace.join("admission-program");
        std::fs::write(&program, b"#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
        let error = verifier
            .admit_tests(f.project, version, proposal(program.clone()))
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("test writable namespace"),
            "{error:#}"
        );
        std::fs::remove_file(program).unwrap();
    }
}
