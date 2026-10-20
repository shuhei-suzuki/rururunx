use super::*;
use crate::execution::pipeline::test_support::spawned_leader;
use std::time::{Duration, Instant};

const SCRATCH: &str = "open(my $t, '>', \"$ENV{TMPDIR}/scratch\") or die; print $t 'x'; close $t;";
const ESCAPE: &str = "use POSIX; pipe(my $r, my $w) or die; defined(my $g = fork) or die; if ($g) { close $w; sysread($r, my $b, 1) == 1 or die; exit } close $r; POSIX::setsid() >= 0 or die; open(my $f, '>', 'escaped.pid') or die; print $f \"$$\\n\"; close $f; syswrite($w, 'r') == 1 or die; close $w; close STDOUT; close STDERR; sleep 30;";
const GATED: &str = "while (!-e 'go') { select undef,undef,undef,0.01; } exit(-e 'fail' ? 3 : 0);";

fn tmpdir(f: &Fixture) -> std::path::PathBuf {
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let step = view.steps.last().unwrap();
    std::path::PathBuf::from(step.body["intent"]["tmpdir"].as_str().unwrap())
}

/// A platform leader that is already a zombie at the birth read is recorded and passes.
#[tokio::test]
async fn he08_zombie_leader_before_record() {
    let (f, runtime, id) = planned_script("exit 0", 10, 1024, 1024).await;
    let (release, arrived) = pause_launch(LaunchPhase::Spawned);
    let launch = runtime.launch_pipeline_test(f.cursor(&id));
    let exited = async {
        arrived.await.unwrap();
        let pid = spawned_leader().unwrap();
        // A zombie is accepted without reading its environment, so any cookie observes it.
        let cookie = rrx_process_tracker::Cookie::new("he8-zombie-cookie-1234").unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let leader = rrx_process_tracker::owned_test_leader(pid, &cookie);
            if leader.is_ok_and(|l| l.zombie) {
                break;
            }
            assert!(Instant::now() < deadline, "leader stayed alive");
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        release.send(()).unwrap();
    };
    let (result, ()) = tokio::join!(launch, exited);
    let LaunchResult::Started(custody) = result.unwrap() else {
        panic!("zombie test leader was not recorded");
    };
    custody.wait().await.unwrap();
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let step = view.steps.last().unwrap();
    assert_eq!(step.state, "exited");
    assert!(step.birth.as_ref().is_some_and(|b| b.contains(':')));
    assert_eq!(step.body["death_evidence"]["launch_checked"], true);
    assert_eq!(step.body["death_evidence"]["leader_birth_matched"], true);
    assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let step = view.steps.last().unwrap();
    assert_eq!(step.body["adoption"]["result"], "pass");
    runtime.shutdown().await.unwrap();
}

/// Certified passing and failing adoptions reclaim the TMPDIR their command wrote.
#[tokio::test]
async fn b6_s3_tmpdir_reclaimed() {
    for (exit, result) in [("exit 0;", "pass"), ("exit 3;", "fail")] {
        let (f, runtime, id) = planned_test(&format!("{SCRATCH} {exit}"), 10, 1024, 1024).await;
        finish(&f, &runtime, &id, true).await;
        let tmp = tmpdir(&f);
        assert!(tmp.join("scratch").is_file());
        assert!(runtime.adopt_pipeline_test(f.cursor(&id)).await.unwrap());
        let view = f.store().pipeline_goal_view(f.goal).unwrap();
        let step = view.steps.last().unwrap();
        assert_eq!(step.body["adoption"]["result"], result);
        assert!(view.runs[0].body["test_cleanup"].is_null());
        assert!(!tmp.exists());
        runtime.shutdown().await.unwrap();
    }
}

/// A held test without certified death keeps its TMPDIR, and reclamation refuses it.
#[tokio::test]
async fn b6_s3_tmpdir_kept_unconfirmed() {
    let (f, runtime, id) = planned_test(&format!("{SCRATCH} {ESCAPE}"), 10, 1024, 1024).await;
    let config = f.config.pipeline.as_ref().unwrap();
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let checkout = PipelineTestRunner::checkout(config, view.steps.last().unwrap()).unwrap();
    finish(&f, &runtime, &id, true).await;
    let pid = std::fs::read_to_string(checkout.join("escaped.pid"))
        .unwrap()
        .trim()
        .parse::<u32>()
        .unwrap();
    let cookie = rrx_process_tracker::Cookie::new("he8-fixture-cookie-1234").unwrap();
    let birth = rrx_process_tracker::owned_test_leader(pid, &cookie)
        .unwrap()
        .birth;
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let step = view.steps.last().unwrap();
    let held = step.state == "held" && step.body["reason"] == "death_unconfirmed";
    let refused = PipelineTestRunner::reclaim_tmpdir(config, step).is_err();
    let kept = tmpdir(&f).join("scratch").is_file();
    assert_eq!(
        rrx_process_tracker::owned_test_leader(pid, &cookie)
            .unwrap()
            .birth,
        birth
    );
    rustix::process::kill_process(
        rustix::process::Pid::from_raw(pid as i32).unwrap(),
        rustix::process::Signal::KILL,
    )
    .unwrap();
    assert!(held && refused && kept);
    runtime.shutdown().await.unwrap();
}

fn goal_step(run: &str, seq: u32, index: u64, command: &str, base: &str) -> PipelineStep {
    let mut step = planned(run, seq, 1, "test");
    step.step_base = base.into();
    step.body = json!({"command_id":command,"command_index":index});
    step
}
fn goal_last(f: &Fixture, id: &str) -> PipelineStep {
    let steps = f.store().pipeline_goal_view(f.goal).unwrap().steps;
    let own = steps.into_iter().filter(|s| s.run_id == id);
    own.max_by_key(|s| (s.seq, s.attempt)).unwrap()
}
/// Claims a Goal check run over gated commands after one released Task run.
async fn goal_commands(ids: &[&str]) -> (Fixture, Arc<Runtime>, String) {
    let template = proposal("/usr/bin/perl".into()).commands.remove(0);
    let commands = ids
        .iter()
        .map(|id| TestCommand {
            id: (*id).into(),
            args: vec!["-e".into(), GATED.into()],
            timeout_seconds: 10,
            ..template.clone()
        })
        .collect();
    let f = fixture_with_commands(true, true, None, None, false, Some(commands)).await;
    let (f, runtime, id) = super::super::super::b5::prepare_fixture_from(f, true).await;
    assert!(runtime.prepare_pipeline_task(f.cursor(&id)).await.unwrap());
    let mut run = f.store().pipeline_goal_view(f.goal).unwrap().runs[0].clone();
    let mut release = planned(&id, 2, 1, "release");
    release.step_base = run.head_commit.clone();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &release, true)
        .unwrap();
    let mut journal = release_journal(&f, &id);
    journal.old_commit = run.head_commit.clone();
    journal.new_commit = run.head_commit.clone();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_release_begin(&f.owner, &cursor, &journal, &json!({"approved":true}))
        .unwrap();
    let cursor = f.cursor(&id);
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
    let first = goal_step(&run.id, 1, 0, ids[0], &run.head_commit);
    f.store()
        .pipeline_claim_run(&f.owner, &run, 1, &first, None)
        .unwrap();
    (f, runtime, run.id)
}
/// Launches the last planned Goal check; `fail` exists before `go` releases the command.
async fn run_goal_test(f: &Fixture, runtime: &Runtime, id: &str, fail: bool) {
    let LaunchResult::Started(custody) = runtime
        .launch_pipeline_test(goal_cursor(f, id))
        .await
        .unwrap()
    else {
        panic!("Goal test did not start");
    };
    let config = f.config.pipeline.as_ref().unwrap();
    let checkout = PipelineTestRunner::checkout(config, &goal_last(f, id)).unwrap();
    if fail {
        std::fs::write(checkout.join("fail"), b"fail").unwrap();
    }
    std::fs::write(checkout.join("go"), b"go").unwrap();
    custody.wait().await.unwrap();
}
async fn adopt_goal(f: &Fixture, runtime: &Runtime, id: &str) -> bool {
    runtime
        .adopt_pipeline_test(goal_cursor(f, id))
        .await
        .unwrap()
}
fn plan_goal(f: &Fixture, id: &str, seq: u32, index: u64, command: &str) {
    let next = goal_step(id, seq, index, command, &goal_last(f, id).step_base);
    let cursor = goal_cursor(f, id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap();
}

/// A failing middle Goal check holds its own step, and retrying it lets the Goal complete.
#[tokio::test]
async fn b6_s1_goal_middle_failure_retried() {
    let (f, runtime, id) = goal_commands(&["first", "second", "third"]).await;
    run_goal_test(&f, &runtime, &id, false).await;
    assert!(adopt_goal(&f, &runtime, &id).await);
    plan_goal(&f, &id, 2, 1, "second");
    run_goal_test(&f, &runtime, &id, true).await;
    assert!(!adopt_goal(&f, &runtime, &id).await);
    // The failure is held where it was observed, and no later command is planned.
    let failed = goal_last(&f, &id);
    assert_eq!((failed.seq, failed.attempt), (2, 1));
    assert_eq!(failed.state, "held");
    assert_eq!(failed.body["reason"], "goal_check_failed");
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    let run = view.runs.into_iter().find(|r| r.id == id).unwrap();
    assert_eq!(run.state, "held");
    let request = PipelineRecoveryRequest {
        request_id: Uuid::new_v4(),
        cursor: goal_cursor(&f, &id),
        then_action: "retry".into(),
        reason: "retry the failed Goal check".into(),
    };
    f.store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
        .unwrap();
    let retried = goal_last(&f, &id);
    assert_eq!((retried.seq, retried.attempt), (2, 2));
    run_goal_test(&f, &runtime, &id, false).await;
    assert!(adopt_goal(&f, &runtime, &id).await);
    plan_goal(&f, &id, 3, 2, "third");
    run_goal_test(&f, &runtime, &id, false).await;
    assert!(adopt_goal(&f, &runtime, &id).await);
    assert_eq!(
        f.store().goal(f.goal).unwrap().unwrap().state,
        GoalState::Completed
    );
    runtime.shutdown().await.unwrap();
}
