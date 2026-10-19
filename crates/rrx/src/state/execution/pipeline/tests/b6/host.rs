use super::*;
use crate::execution::pipeline::{
    LaunchResult, PipelineGit, PipelineTestRunner,
    test_support::{LaunchPhase, pause_launch},
};
use std::sync::Arc;

async fn planned_test(
    code: &str,
    timeout: u64,
    out: usize,
    err: usize,
) -> (Fixture, Arc<Runtime>, String) {
    let mut command = proposal("/usr/bin/perl".into()).commands.remove(0);
    command.args = vec![
        "-e".into(),
        format!("while (!-e 'go') {{ select undef,undef,undef,0.01; }} {code}"),
    ];
    command.timeout_seconds = timeout;
    command.stdout_bytes = out;
    command.stderr_bytes = err;
    let f = fixture_with_catalog(true, true, None, None, false, Some(command)).await;
    let (f, runtime, id) = super::super::b5::prepare_fixture_from(f, true).await;
    assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let mut step = planned(&id, 2, 1, "test");
    step.step_base = view.runs[0].head_commit.clone();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &step, true)
        .unwrap();
    (f, runtime, id)
}
async fn finish(f: &Fixture, runtime: &Runtime, id: &str, release: bool) {
    let LaunchResult::Started(custody) = runtime.launch_pipeline_test(f.cursor(id)).await.unwrap()
    else {
        panic!("test did not start");
    };
    if release {
        let view = f.store().pipeline_goal_view(f.goal).unwrap();
        let checkout = PipelineTestRunner::checkout(
            f.config.pipeline.as_ref().unwrap(),
            view.steps.last().unwrap(),
        )
        .unwrap();
        std::fs::write(checkout.join("go"), b"go").unwrap();
    }
    custody.wait().await.unwrap();
}
async fn assert_held(result: LaunchResult) {
    let held = matches!(&result, LaunchResult::Held);
    if let LaunchResult::Started(custody) = result {
        custody.request_stop();
        custody.wait().await.unwrap();
    }
    assert!(held, "unexpected test launch");
}
fn corrupt(f: &Fixture) {
    // Negative tampering: preserve both keys while invalidating the body digest.
    f.store().connection.execute("INSERT OR REPLACE INTO verification_profiles(project_id,digest,body) SELECT project_id,digest,body || ' ' FROM verification_profiles WHERE project_id=?1", [f.project.to_string()]).unwrap();
}

/// Runtime test launching uses pipeline custody and adopts a verified passing receipt.
#[tokio::test]
async fn b6_test_flow() {
    let (f, runtime, id) = planned_test("print 'ok'; exit 0;", 10, 1024, 1024).await;
    finish(&f, &runtime, &id, true).await;
    assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let step = view.steps.last().unwrap();
    assert_eq!(step.state, "adopted");
    assert_eq!(step.body["adoption"]["result"], "pass");
    assert!(
        !PipelineTestRunner::checkout(f.config.pipeline.as_ref().unwrap(), step)
            .unwrap()
            .exists()
    );
    let n: u32 = f
        .store()
        .connection
        .query_row("SELECT count(*) FROM verification_runs", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 0);
    runtime.shutdown().await.unwrap();
}

/// Nonzero exit is a correctly observed adopted failure.
#[tokio::test]
async fn b6_test_nonzero() {
    let (f, runtime, id) = planned_test("exit 3;", 10, 1024, 1024).await;
    finish(&f, &runtime, &id, true).await;
    assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(
        view.steps.last().unwrap().body["adoption"]["result"],
        "fail"
    );
    runtime.shutdown().await.unwrap();
}

/// Reaching the admitted runtime limit records an adoptable failure.
#[tokio::test]
async fn b6_test_timeout() {
    let (f, runtime, id) = planned_test("exit 0;", 1, 1024, 1024).await;
    finish(&f, &runtime, &id, false).await;
    assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let step = view.steps.last().unwrap();
    assert_eq!(step.body["exit"]["timed_out"], true);
    assert_eq!(step.body["adoption"]["result"], "fail");
    runtime.shutdown().await.unwrap();
}

/// Exceeding either admitted stream limit records an adoptable failure.
#[tokio::test]
async fn b6_test_overflow() {
    for code in ["print 'x' x 4096;", "print STDERR 'x' x 4096;"] {
        let (f, runtime, id) = planned_test(code, 10, 32, 32).await;
        finish(&f, &runtime, &id, true).await;
        assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
        let view = f.store().pipeline_goal_view(f.goal).unwrap();
        let step = view.steps.last().unwrap();
        assert_eq!(step.body["exit"]["overflow"], true);
        assert_eq!(step.body["adoption"]["result"], "fail");
        runtime.shutdown().await.unwrap();
    }
}

/// Retained stream tampering holds an exited result instead of adopting it.
#[tokio::test]
async fn b6_test_evidence_changed() {
    use std::os::unix::fs::PermissionsExt;
    let (f, runtime, id) = planned_test("print 'ok';", 10, 1024, 1024).await;
    finish(&f, &runtime, &id, true).await;
    let file = f
        .config
        .pipeline
        .as_ref()
        .unwrap()
        .work_root
        .join("runs")
        .join(&id)
        .join("2-1/stdout.log");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::write(file, b"no").unwrap();
    assert!(!runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    assert_eq!(
        f.store()
            .pipeline_goal_view(f.goal)
            .unwrap()
            .steps
            .last()
            .unwrap()
            .state,
        "held"
    );
    runtime.shutdown().await.unwrap();
}

/// Planning turns a changed catalog into a death-confirmed held step.
#[tokio::test]
async fn b6_plan_profile_changed() {
    let f = fixture().await;
    let id = f.claim();
    let cursor = f.cursor(&id);
    // The fixture's prepare is an internal step with no child.
    f.store()
        .pipeline_begin_launch(&f.owner, &cursor, None, &json!({}))
        .unwrap();
    let cursor = f.cursor(&id);
    let j = f.store().pipeline_goal_view(f.goal).unwrap().journals[0].clone();
    f.store()
        .pipeline_release_settle(&f.owner, &cursor, &j.id, j.version, &j.new_commit)
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_adopt(&f.owner, &cursor, &json!({}))
        .unwrap();
    corrupt(&f);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &planned(&id, 2, 1, "test"), true)
        .unwrap();
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let step = view.steps.last().unwrap();
    assert_eq!(step.state, "held");
    assert_eq!(step.body["reason"], "tests_profile_changed");
    assert_eq!(step.body["death_confirmed"], true);
}

/// The intent transaction rechecks the profile even when callers bypass Runtime preflight.
#[tokio::test]
async fn b6_tx1_profile_changed() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    corrupt(&f);
    let cursor = f.cursor(&id);
    assert_eq!(
        f.store()
            .pipeline_begin_launch(&f.owner, &cursor, Some("b6-cookie-123456789"), &json!({}))
            .unwrap(),
        PipelineLaunch::Held
    );
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    assert!(view.steps.last().unwrap().cookie.is_none());
    assert_eq!(
        view.steps.last().unwrap().body["reason"],
        "tests_profile_changed"
    );
    runtime.shutdown().await.unwrap();
}

/// Runtime profile preflight holds corruption before creating a checkout.
#[tokio::test]
async fn b6_launch_profile_changed() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    corrupt(&f);
    assert_held(runtime.launch_pipeline_test(f.cursor(&id)).await.unwrap()).await;
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    assert!(
        !PipelineTestRunner::checkout(
            f.config.pipeline.as_ref().unwrap(),
            view.steps.last().unwrap()
        )
        .unwrap()
        .exists()
    );
    runtime.shutdown().await.unwrap();
}

/// Adoption rechecks a catalog whose keys survived a body replacement.
#[tokio::test]
async fn b6_adopt_profile_changed() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    finish(&f, &runtime, &id, true).await;
    corrupt(&f);
    assert!(!runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    assert_eq!(
        f.store()
            .pipeline_goal_view(f.goal)
            .unwrap()
            .steps
            .last()
            .unwrap()
            .body["reason"],
        "tests_profile_changed"
    );
    runtime.shutdown().await.unwrap();
}

/// A pre-spawn program replacement is rejected without creating a child.
#[tokio::test]
async fn b6_launch_recheck() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let program = dir.path().join("test-program");
    std::fs::write(&program, b"#!/usr/bin/perl\nexit 0;\n").unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    let command = proposal(program.clone()).commands.remove(0);
    let f = fixture_with_catalog(true, true, None, None, false, Some(command)).await;
    let (f, runtime, id) = super::super::b5::prepare_fixture_from(f, false).await;
    runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap();
    let mut next = planned(&id, 2, 1, "test");
    next.step_base = f.store().pipeline_goal_view(f.goal).unwrap().runs[0]
        .head_commit
        .clone();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap();
    let (release, arrived) = pause_launch(LaunchPhase::Verified);
    let launch = runtime.launch_pipeline_test(f.cursor(&id));
    let replace = async {
        arrived.await.unwrap();
        std::fs::write(program, b"#!/usr/bin/perl\nexit 1;\n").unwrap();
        release.send(()).unwrap();
    };
    let (result, ()) = tokio::join!(launch, replace);
    assert_held(result.unwrap()).await;
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let step = view.steps.last().unwrap();
    assert_eq!(step.state, "rejected");
    assert!(step.pid.is_none());
    runtime.shutdown().await.unwrap();
}

/// A dirty registered leftover is rebuilt only while its test step is still planned.
#[tokio::test]
async fn b6_checkout_rebuild() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    let cursor = f.cursor(&id);
    let (run, step) = f
        .store()
        .pipeline_launch_snapshot(&f.owner, &cursor)
        .unwrap();
    let profile = f
        .store()
        .pipeline_test_context(&f.owner, &cursor)
        .unwrap()
        .unwrap();
    let test = PipelineTestRunner::prepare(
        &f.owner,
        f.config.pipeline.as_ref().unwrap(),
        &run,
        &step,
        profile,
    )
    .await
    .unwrap();
    std::fs::write(test.checkout.join("drift"), b"dirty").unwrap();
    finish(&f, &runtime, &id, true).await;
    assert!(!test.checkout.join("drift").exists());
    assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    runtime.shutdown().await.unwrap();
}

/// An unregistered path at the checkout location is retained and holds launch.
#[tokio::test]
async fn b6_checkout_partial() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
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
    std::fs::create_dir_all(&checkout).unwrap();
    std::fs::write(checkout.join("keep"), b"keep").unwrap();
    assert_held(runtime.launch_pipeline_test(f.cursor(&id)).await.unwrap()).await;
    assert_eq!(std::fs::read(checkout.join("keep")).unwrap(), b"keep");
    runtime.shutdown().await.unwrap();
}

/// Checkout-specific drivers are refused before materializing tracked files.
#[tokio::test]
async fn b6_checkout_driver() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    let project = f.store().project(f.project).unwrap().unwrap();
    let git = PipelineGit::new(&project.root).unwrap();
    let common = git
        .text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .await
        .unwrap();
    let include = f._dir.path().join("checkout.inc");
    std::fs::write(&include, b"[filter \"evil\"]\nsmudge = false\n").unwrap();
    git.run(&[
        "config",
        "--local",
        &format!("includeIf.gitdir:{common}/worktrees/**.path"),
        include.to_str().unwrap(),
    ])
    .await
    .unwrap();
    assert_held(runtime.launch_pipeline_test(f.cursor(&id)).await.unwrap()).await;
    let step = f
        .store()
        .pipeline_goal_view(f.goal)
        .unwrap()
        .steps
        .last()
        .unwrap()
        .clone();
    assert!(
        !PipelineTestRunner::checkout(f.config.pipeline.as_ref().unwrap(), &step)
            .unwrap()
            .join("tracked")
            .exists()
    );
    runtime.shutdown().await.unwrap();
}

/// Test children receive only the fixed environment without CODEX_HOME or credentials.
#[tokio::test]
async fn b6_test_environment() {
    let (f, runtime, id) =
        planned_test("print join(qq(\\n), sort keys %ENV);", 10, 4096, 1024).await;
    finish(&f, &runtime, &id, true).await;
    let file = f
        .config
        .pipeline
        .as_ref()
        .unwrap()
        .work_root
        .join("runs")
        .join(&id)
        .join("2-1/stdout.log");
    let names = std::fs::read_to_string(file).unwrap();
    let allowed = [
        "HOME",
        "USER",
        "LOGNAME",
        "PATH",
        "LANG",
        "TMPDIR",
        "TERM",
        "NO_COLOR",
        "RRX_PROCESS_COOKIE",
    ];
    assert!(names.lines().all(|name| allowed.contains(&name)));
    assert!(names.lines().any(|name| name == "RRX_PROCESS_COOKIE"));
    assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    runtime.shutdown().await.unwrap();
}

/// Observation I/O failure holds the test despite a complete death receipt.
#[tokio::test]
async fn b6_test_io_held() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_begin_launch(&f.owner, &cursor, Some("b6-io-cookie-123456"), &json!({}))
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_finish_step(
            &f.owner,
            &cursor,
            &death(),
            PipelineExit {
                outcome: None,
                evidence: &json!({"issue":"output_io"}),
                issue: Some("output_io"),
            },
        )
        .unwrap();
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(view.steps.last().unwrap().state, "held");
    assert_eq!(view.runs[0].state, "held");
    runtime.shutdown().await.unwrap();
}

/// Pausing a Goal prevents test adoption under the Runtime's execution boundary.
#[tokio::test]
async fn b6_test_pause() {
    let (f, runtime, id) = planned_test("exit 0;", 10, 1024, 1024).await;
    finish(&f, &runtime, &id, true).await;
    let (socket, _peer) = tokio::net::UnixStream::pair().unwrap();
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: f.owner.instance.clone(),
        epoch: f.owner.epoch,
        action: ControlAction::SetGoalLifecycle {
            project: f.project,
            goal: f.goal,
            expected_goal: 1,
            target: crate::runtime::control::GoalControl::Pause,
            reason: "pause".into(),
        },
    };
    runtime.handle_control(&socket, request).await.unwrap();
    let cursor = f.cursor(&id);
    assert!(runtime.adopt_pipeline_test(cursor.clone()).await.is_err());
    assert_eq!(f.cursor(&id).step_version, cursor.step_version);
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
    runtime.shutdown().await.unwrap();
}

fn admission_fixture() -> (tempfile::TempDir, Runtime, Project, tokio::net::UnixStream) {
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open(&dir.path().join("state/state.db")).unwrap();
    let runtime = Runtime::new(owner.clone(), Config::default()).unwrap();
    let root = dir.path().join("source");
    std::fs::create_dir(&root).unwrap();
    let mut project = Project::new(
        "admit".into(),
        root.canonicalize().unwrap(),
        "local".into(),
        "main".into(),
    );
    owner
        .store
        .lock()
        .unwrap()
        .put_project(&mut project)
        .unwrap();
    let (socket, _peer) = tokio::net::UnixStream::pair().unwrap();
    (dir, runtime, project, socket)
}
fn admission_request(runtime: &Runtime, project: &Project) -> ControlRequest {
    let identity = runtime.control_identity();
    ControlRequest {
        request_id: Uuid::new_v4(),
        instance: identity.instance,
        epoch: identity.epoch,
        action: ControlAction::ProjectAdmitTests {
            project: project.id,
            expected_project: project.version,
            profile: proposal("/usr/bin/true".into()),
        },
    }
}

/// The authenticated ProjectAdmitTests control reaches ManagedVerifier admission.
#[tokio::test]
async fn b6_admission_wiring() {
    let (_dir, runtime, project, socket) = admission_fixture();
    let response = runtime
        .handle_control(&socket, admission_request(&runtime, &project))
        .await
        .unwrap();
    assert!(
        matches!(&response, ControlResponse::ProjectTestsAdmitted { project:id, digest } if *id == project.id && digest.len() == 64)
    );
    let connection = Connection::open_with_flags(
        runtime.control_identity().state,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let stored: String = connection
        .query_row(
            "SELECT digest FROM verification_profiles WHERE project_id=?1",
            [project.id.to_string()],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        matches!(response, ControlResponse::ProjectTestsAdmitted { digest, .. } if digest == stored)
    );
}

/// Admission rejects stale owner epochs and stale Project versions without activating a catalog.
#[tokio::test]
async fn b6_admission_fenced() {
    let (_dir, runtime, project, socket) = admission_fixture();
    let mut stale = admission_request(&runtime, &project);
    stale.epoch += 1;
    assert!(runtime.handle_control(&socket, stale).await.is_err());
    let mut stale = admission_request(&runtime, &project);
    if let ControlAction::ProjectAdmitTests {
        expected_project, ..
    } = &mut stale.action
    {
        *expected_project += 1;
    }
    assert!(runtime.handle_control(&socket, stale).await.is_err());
    assert!(matches!(
        runtime
            .handle_control(&socket, admission_request(&runtime, &project))
            .await
            .unwrap(),
        ControlResponse::ProjectTestsAdmitted { .. }
    ));
}

/// An admitted catalog cannot be replaced through the control command.
#[tokio::test]
async fn b6_admission_immutable() {
    let (_dir, runtime, project, socket) = admission_fixture();
    runtime
        .handle_control(&socket, admission_request(&runtime, &project))
        .await
        .unwrap();
    assert!(
        runtime
            .handle_control(&socket, admission_request(&runtime, &project))
            .await
            .is_err()
    );
}

/// Direct Store adoption also holds a catalog replaced after test exit.
#[tokio::test]
async fn b6_store_adopt_profile() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "test");
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(&f.owner, &cursor, &death(), None, &observed().body["exit"])
        .unwrap();
    corrupt(&f);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_adopt(
            &f.owner,
            &cursor,
            &json!({"command_id":"test","result":"pass"}),
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
        "held"
    );
}

/// Planning refuses a command index outside the admitted catalog.
#[tokio::test]
async fn b6_plan_command_index() {
    let f = fixture().await;
    let id = f.claim();
    f.advance_prepare(&id);
    let mut next = planned(&id, 2, 1, "test");
    next.body["command_index"] = json!(32);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap();
    assert_eq!(
        f.store()
            .pipeline_goal_view(f.goal)
            .unwrap()
            .steps
            .last()
            .unwrap()
            .state,
        "held"
    );
}

/// A timeout cannot complete a Goal even if its leader exited zero.
#[tokio::test]
async fn b6_goal_timeout() {
    let f = fixture().await;
    let task_run = complete_task(&f);
    let mut run = read_run(&f.store().connection, &task_run).unwrap();
    run.id = Uuid::new_v4().to_string();
    run.kind = "goal".into();
    run.task_id = None;
    run.worktree = None;
    run.branch = None;
    run.version = 0;
    run.state = "running".into();
    f.store()
        .pipeline_claim_run(&f.owner, &run, 1, &planned(&run.id, 1, 1, "test"), None)
        .unwrap();
    let mut cursor = PipelineCursor {
        run_id: run.id,
        run_version: 1,
        step_version: 1,
        seq: 1,
        attempt: 1,
        wait_version: None,
    };
    f.store()
        .pipeline_begin_launch(&f.owner, &cursor, Some("b6-goal-cookie-12345"), &json!({}))
        .unwrap();
    cursor.step_version += 1;
    let mut exit = observed().body["exit"].clone();
    exit["timed_out"] = json!(true);
    f.store()
        .pipeline_record_exit(&f.owner, &cursor, &death(), None, &exit)
        .unwrap();
    cursor.step_version += 1;
    assert!(
        f.store()
            .pipeline_complete_goal(&f.owner, &cursor, 1, "test")
            .is_err()
    );
    assert_eq!(
        f.store().goal(f.goal).unwrap().unwrap().state,
        GoalState::Running
    );
}

fn goal_cursor(f: &Fixture, id: &str) -> PipelineCursor {
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let run = view.runs.iter().find(|r| r.id == id).unwrap();
    let step = view
        .steps
        .iter()
        .filter(|s| s.run_id == id)
        .max_by_key(|s| (s.seq, s.attempt))
        .unwrap();
    PipelineCursor {
        run_id: id.into(),
        run_version: run.version,
        step_version: step.version,
        seq: step.seq,
        attempt: step.attempt,
        wait_version: view.provider_wait.map(|w| w.version),
    }
}
async fn goal_exited(code: &str) -> (Fixture, Arc<Runtime>, String) {
    let (f, runtime, task_id) = planned_test(code, 10, 1024, 1024).await;
    finish(&f, &runtime, &task_id, true).await;
    runtime
        .adopt_pipeline_test(f.cursor(&task_id))
        .await
        .unwrap();
    let mut run = f.store().pipeline_goal_view(f.goal).unwrap().runs[0].clone();
    let mut release = planned(&task_id, 3, 1, "release");
    release.step_base = run.head_commit.clone();
    let cursor = f.cursor(&task_id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &release, true)
        .unwrap();
    let mut journal = release_journal(&f, &task_id);
    journal.old_commit = run.head_commit.clone();
    journal.new_commit = run.head_commit.clone();
    let cursor = f.cursor(&task_id);
    f.store()
        .pipeline_release_begin(&f.owner, &cursor, &journal, &json!({"approved":true}))
        .unwrap();
    let cursor = f.cursor(&task_id);
    f.store()
        .pipeline_release_settle(&f.owner, &cursor, &journal.id, 1, &journal.new_commit)
        .unwrap();
    run.id = Uuid::new_v4().to_string();
    run.kind = "goal".into();
    run.task_id = None;
    run.worktree = None;
    run.branch = None;
    run.state = "running".into();
    run.version = 0;
    run.body = json!({});
    let mut first = planned(&run.id, 1, 1, "test");
    first.step_base = run.head_commit.clone();
    f.store()
        .pipeline_claim_run(&f.owner, &run, 1, &first, None)
        .unwrap();
    let LaunchResult::Started(custody) = runtime
        .launch_pipeline_test(goal_cursor(&f, &run.id))
        .await
        .unwrap()
    else {
        panic!("Goal test did not start");
    };
    let checkout =
        PipelineTestRunner::checkout(f.config.pipeline.as_ref().unwrap(), &first).unwrap();
    std::fs::write(checkout.join("go"), b"go").unwrap();
    custody.wait().await.unwrap();
    (f, runtime, run.id)
}

/// The final passing Goal test is adopted in the Goal completion transaction.
#[tokio::test]
async fn b6_goal_runtime_complete() {
    let (f, runtime, id) = goal_exited("exit 0;").await;
    assert!(
        runtime
            .adopt_pipeline_test(goal_cursor(&f, &id))
            .await
            .unwrap()
    );
    assert_eq!(
        f.store().goal(f.goal).unwrap().unwrap().state,
        GoalState::Completed
    );
    assert_eq!(
        f.store()
            .pipeline_goal_view(f.goal)
            .unwrap()
            .runs
            .iter()
            .find(|r| r.id == id)
            .unwrap()
            .state,
        "completed"
    );
    runtime.shutdown().await.unwrap();
}

/// The final failing Goal test holds the run and leaves the Goal incomplete.
#[tokio::test]
async fn b6_goal_runtime_failed() {
    let (f, runtime, id) = goal_exited("exit 3;").await;
    assert!(
        !runtime
            .adopt_pipeline_test(goal_cursor(&f, &id))
            .await
            .unwrap()
    );
    assert_eq!(
        f.store().goal(f.goal).unwrap().unwrap().state,
        GoalState::Running
    );
    assert_eq!(
        f.store()
            .pipeline_goal_view(f.goal)
            .unwrap()
            .runs
            .iter()
            .find(|r| r.id == id)
            .unwrap()
            .state,
        "held"
    );
    runtime.shutdown().await.unwrap();
}
