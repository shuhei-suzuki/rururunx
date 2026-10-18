use super::round2::{copy, permitted, rejected, update};
use super::*;

pub(super) fn latest(f: &Fixture, id: &str) -> PipelineStep {
    let c = f.cursor(id);
    read_step(&f.store().connection, id, c.seq, c.attempt).unwrap()
}
pub(super) fn cancel(f: &Fixture) {
    let mut store = f.store();
    let goal = store.goal(f.goal).unwrap().unwrap();
    store
        .set_runtime_goal_lifecycle(
            &f.ingress,
            &ControlRequest {
                request_id: Uuid::new_v4(),
                instance: f.owner.instance.clone(),
                epoch: f.owner.epoch,
                action: ControlAction::SetGoalLifecycle {
                    project: f.project,
                    goal: f.goal,
                    expected_goal: goal.version,
                    target: crate::runtime::control::GoalControl::Cancel,
                    reason: "cancel fixture".into(),
                },
            },
            &f.profile.policy_sha256,
        )
        .unwrap();
}
fn held_process(f: &Fixture, id: &str) {
    f.launching(id, "implement");
    let c = f.cursor(id);
    f.store()
        .pipeline_record_running(&f.owner, &c, 123, "birth")
        .unwrap();
    let c = f.cursor(id);
    f.store()
        .pipeline_hold(&f.owner, &c, "custody_lost", None)
        .unwrap();
}

/// A held process cannot acquire death evidence through a repeated hold API call.
#[tokio::test]
async fn b2_fix1_rehold_api() {
    let f = fixture().await;
    let id = f.claim();
    held_process(&f, &id);
    let c = f.cursor(&id);
    let before = latest(&f, &id);
    let run_before = read_run(&f.store().connection, &id).unwrap();
    for evidence in [Some(death()), None] {
        let error = f
            .store()
            .pipeline_hold(&f.owner, &c, "dead", evidence.as_ref())
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "pipeline stop source is held or terminal"
        );
        assert_eq!(latest(&f, &id), before);
        assert_eq!(read_run(&f.store().connection, &id).unwrap(), run_before);
    }
}
/// SQL cannot turn unconfirmed held custody into confirmed death.
#[tokio::test]
async fn b2_fix1_rehold_sql() {
    let f = fixture().await;
    let id = f.claim();
    held_process(&f, &id);
    rejected(
        &f.store().connection,
        &update(
            "pipeline_steps",
            ",body=json_set(body,'$.version',version+1,'$.death_confirmed',json('true'))",
        ),
        "invalid pipeline step transition",
    );
}

/// A live process can acquire death confirmation only with complete death evidence.
#[tokio::test]
async fn b2_fix1_death_requires_evidence() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    rejected(
        &f.store().connection,
        &update(
            "pipeline_steps",
            ",state='exited',body=json_set(body,'$.version',version+1,'$.state','exited','$.death_confirmed',json('true'))",
        ),
        "invalid pipeline step transition",
    );
    let c = f.cursor(&id);
    f.store()
        .pipeline_record_exit(&f.owner, &c, &death(), Some("success"), &json!({}))
        .unwrap();
    assert_eq!(latest(&f, &id).body["death_confirmed"], json!(true));
}
/// Settlement never accepts a process cursor for a Goal creation journal.
#[tokio::test]
async fn b2_fix1_settle_process_cursor() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    let c = f.cursor(&id);
    f.store()
        .pipeline_record_running(&f.owner, &c, 123, "birth")
        .unwrap();
    let j = f.store().pipeline_goal_view(f.goal).unwrap().journals[0].clone();
    let c = f.cursor(&id);
    let run = read_run(&f.store().connection, &id).unwrap();
    let step = latest(&f, &id);
    assert!(!journal_step_matches(&j, &run, &step));
    assert!(
        f.store()
            .pipeline_release_settle(&f.owner, &c, &j.id, j.version, &"b".repeat(40))
            .is_err()
    );
    assert_eq!(latest(&f, &id).state, "running");
}
/// Settlement is bound to the creator run's running prepare step.
#[tokio::test]
async fn b2_fix1_settle_internal_scope() {
    let f = fixture().await;
    let id = f.claim();
    let c = f.cursor(&id);
    let j = f.store().pipeline_goal_view(f.goal).unwrap().journals[0].clone();
    assert!(
        f.store()
            .pipeline_release_settle(&f.owner, &c, &j.id, 1, &"b".repeat(40))
            .is_err()
    );
    f.store()
        .pipeline_begin_launch(&f.owner, &c, None, &json!({}))
        .unwrap();
    let c = f.cursor(&id);
    f.store()
        .pipeline_release_settle(&f.owner, &c, &j.id, 1, &"b".repeat(40))
        .unwrap();
    let step = latest(&f, &id);
    assert_eq!(step.state, "held");
    assert!(step.body.get("death_confirmed").is_none());
    let c = f.cursor(&id);
    let before = step;
    assert!(
        f.store()
            .pipeline_release_settle(&f.owner, &c, &j.id, 2, &j.new_commit)
            .is_err()
    );
    assert_eq!(latest(&f, &id), before);
}
/// Rowid aliases cannot replace history even with disjoint declared keys.
#[tokio::test]
async fn b2_fix1_rowid_replace() {
    for table in PIPELINE_TABLES {
        let f = if *table == "pipeline_steps" {
            let f = fixture().await;
            let id = f.claim();
            f.advance_prepare(&id);
            f
        } else {
            super::round2::populated(table).await
        };
        let native = fixture_with_owner(false, Some(f.owner.clone())).await;
        let goal = format!("'{}'", native.goal);
        let project = format!("'{}'", native.project);
        let ref_name = format!("'refs/heads/rrx/goal/{}'", native.goal);
        let request = format!("'{}'", native.accepted.request_id);
        let run = f
            .store()
            .pipeline_goal_view(f.goal)
            .unwrap()
            .runs
            .iter()
            .find(|r| r.kind == "task")
            .unwrap()
            .id
            .clone();
        let check = format!("'{run}'");
        let store = f.store();
        let c = &store.connection;
        c.pragma_update(None, "recursive_triggers", false).unwrap();
        let changes: Vec<(&str, &str)> = match *table {
            "goal_execution_profiles" => vec![("goal_id", &goal), ("project_id", &project)],
            "pipeline_runs" => vec![
                ("id", "'other-run'"),
                ("kind", "'task'"),
                ("task_id", "'other-task'"),
                ("worktree", "'/other'"),
                ("branch", "'other-branch'"),
                ("version", "1"),
                ("round", "0"),
                ("state", "'running'"),
                (
                    "body",
                    "json_set(body,'$.id','other-run','$.state','running','$.version',1)",
                ),
            ],
            "pipeline_steps" => vec![
                ("seq", "2"),
                ("attempt", "1"),
                ("version", "1"),
                ("state", "'planned'"),
                ("cookie", "NULL"),
                ("pid", "NULL"),
                ("birth", "NULL"),
                ("result_commit", "NULL"),
                ("result_tree", "NULL"),
                (
                    "body",
                    "json_object('version',1,'state','planned','epoch',owner_epoch)",
                ),
            ],
            "pipeline_recoveries" => vec![("id", "'other-recovery'"), ("request_id", &request)],
            "pipeline_ref_journal" => vec![
                ("id", "'other-journal'"),
                ("goal_id", &goal),
                ("project_id", &project),
                ("run_id", "NULL"),
                ("kind", "'goal_create'"),
                ("ref_name", &ref_name),
                ("old_commit", "printf('%040d',0)"),
                ("state", "'pending'"),
                ("version", "1"),
            ],
            "pipeline_qualifications" => vec![("subject", "'platform'")],
            "pipeline_goal_completions" => vec![
                ("goal_id", &goal),
                ("project_id", &project),
                ("goal_check_run", &check),
            ],
            "pipeline_limit_wait" => vec![], // INTEGER PRIMARY KEY is the rowid itself.
            _ => unreachable!(),
        };
        for alias in ["rowid", "_rowid_", "oid"] {
            let sql = copy(c, table, &changes);
            let sql = sql
                .replacen(&format!("{table}("), &format!("{table}({alias},"), 1)
                .replacen(" SELECT ", &format!(" SELECT {alias},"), 1);
            rejected(c, &sql, "pipeline row cannot be replaced");
        }
    }
}
/// Cancelling before prepare settles Goal creation before abandoning the only Task.
#[tokio::test]
async fn b2_fix1_cancel_before_prepare() {
    let f = fixture().await;
    let id = f.claim();
    cancel(&f);
    let c = f.cursor(&id);
    f.store().pipeline_cancel_run(&f.owner, &c, None).unwrap();
    let view = f.store().pipeline_goal_view(f.goal).unwrap();
    assert_eq!(view.runs[0].state, "abandoned");
    assert_eq!(view.journals[0].state, "aborted");
}
/// Abandoning held prepare either settles its creation journal or refuses atomically.
#[tokio::test]
async fn b2_fix1_abandon_held_prepare() {
    let f = fixture().await;
    let id = f.claim();
    let c = f.cursor(&id);
    f.store()
        .pipeline_begin_launch(&f.owner, &c, None, &json!({}))
        .unwrap();
    let c = f.cursor(&id);
    f.store()
        .pipeline_hold(&f.owner, &c, "prepare_failed", None)
        .unwrap();
    let request = recovery(&f, &id, "abandon_run");
    assert!(
        f.store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
            .is_err()
    );
    let j = f.store().pipeline_goal_view(f.goal).unwrap().journals[0].clone();
    let mut e = recovery_evidence();
    e.journal = Some(PipelineRecoveryJournal {
        id: j.id,
        version: j.version,
        observed_commit: j.new_commit,
    });
    let response = f
        .store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &e)
        .unwrap();
    assert_eq!(response.state, "abandoned");
    assert!(!response.release_applied);
    assert_eq!(
        f.store().pipeline_goal_view(f.goal).unwrap().journals[0].state,
        "done"
    );
}
/// Prepare cannot be adopted while Goal creation is still open.
#[tokio::test]
async fn b2_fix1_prepare_requires_create_done() {
    let f = fixture().await;
    let id = f.claim();
    let c = f.cursor(&id);
    f.store()
        .pipeline_begin_launch(&f.owner, &c, None, &json!({}))
        .unwrap();
    let c = f.cursor(&id);
    assert!(f.store().pipeline_adopt(&f.owner, &c, &json!({})).is_err());
}
/// Internal steps never occupy the daemon process gate in either storage layer.
#[tokio::test]
async fn b2_fix1_internal_process_gate() {
    for kind in ["prepare", "release"] {
        for hold in [false, true] {
            let a = fixture().await;
            let b = fixture_with_owner(true, Some(a.owner.clone())).await;
            let ar = a.claim();
            let br = b.claim();
            b.process_step(&br, "test");
            if kind == "release" {
                a.advance_prepare(&ar);
                let c = a.cursor(&ar);
                a.store()
                    .pipeline_plan_next(&a.owner, &c, &planned(&ar, 2, 1, "release"), true)
                    .unwrap();
            }
            let c = a.cursor(&ar);
            a.store()
                .pipeline_begin_launch(&a.owner, &c, None, &json!({}))
                .unwrap();
            if hold {
                let c = a.cursor(&ar);
                a.store()
                    .pipeline_hold(&a.owner, &c, "internal_failed", None)
                    .unwrap();
            }
            assert!(latest(&a, &ar).body.get("death_confirmed").is_none());
            let c = b.cursor(&br);
            assert_eq!(
                b.store()
                    .pipeline_begin_launch(&b.owner, &c, Some("0123456789abcdef"), &json!({}))
                    .unwrap(),
                PipelineLaunch::Started
            );
        }
    }
}
/// A v11 initializer rechecks application identity after waiting for the write lock.
#[test]
fn b2_fix1_v11_application_race() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("race.db");
    historical(&path, 11);
    let before = catalog(&Connection::open(&path).unwrap());
    let mut blocker = Connection::open(&path).unwrap();
    let tx = blocker
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    let (send, receive) = std::sync::mpsc::sync_channel(0);
    let worker = std::thread::spawn(move || {
        let c = Connection::open(&path).unwrap();
        c.authorizer(Some(move |ctx: AuthContext<'_>| {
            if matches!(ctx.action, AuthAction::Transaction { .. }) {
                let _ = send.send(());
            }
            Authorization::Allow
        }));
        Store::initialize(c).map(|_| ()).map_err(|e| e.to_string())
    });
    receive.recv_timeout(Duration::from_secs(5)).unwrap();
    tx.pragma_update(None, "application_id", 12345).unwrap();
    tx.commit().unwrap();
    // Release the callback sender on every later COMMIT/ROLLBACK as well.
    drop(receive);
    assert!(
        worker
            .join()
            .unwrap()
            .unwrap_err()
            .contains("not an rrx state database")
    );
    let c = Connection::open(dir.path().join("race.db")).unwrap();
    assert_eq!(catalog(&c), before);
    assert_eq!(
        c.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        11
    );
}
/// Adoption only accepts each kind's permitted source state and excludes release.
#[tokio::test]
async fn b2_fix1_adopt_source() {
    for kind in ["test", "review", "confirm", "release"] {
        let f = fixture().await;
        let id = f.claim();
        f.process_step(&id, kind);
        let c = f.cursor(&id);
        let error = f
            .store()
            .pipeline_adopt(&f.owner, &c, &json!({"command_id":"test"}))
            .unwrap_err();
        let expected = if kind == "release" {
            "pipeline release is adopted only by journal settlement"
        } else {
            "pipeline adoption needs exited step"
        };
        assert_eq!(error.to_string(), expected);
    }
}
/// A terminal step cannot be rewritten in place by adoption or direct SQL.
#[tokio::test]
async fn b2_fix1_terminal_rewrite() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "test");
    let c = f.cursor(&id);
    f.store()
        .pipeline_begin_launch(&f.owner, &c, Some("0123456789abcdef"), &json!({}))
        .unwrap();
    let c = f.cursor(&id);
    f.store()
        .pipeline_record_exit(&f.owner, &c, &death(), None, &json!({"exit_code":0}))
        .unwrap();
    let c = f.cursor(&id);
    f.store()
        .pipeline_adopt(&f.owner, &c, &json!({"command_id":"test"}))
        .unwrap();
    let c = f.cursor(&id);
    assert!(
        f.store()
            .pipeline_adopt(&f.owner, &c, &json!({"command_id":"unrun"}))
            .is_err()
    );
    rejected(
        &f.store().connection,
        &update(
            "pipeline_steps",
            ",body=json_set(body,'$.version',version+1,'$.adoption.command_id','unrun')",
        ),
        "invalid pipeline step transition",
    );
}
/// A planned test command identity is immutable before and after execution.
#[tokio::test]
async fn b2_fix1_command_identity() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "test");
    rejected(
        &f.store().connection,
        &update(
            "pipeline_steps",
            ",body=json_set(body,'$.version',version+1,'$.command_id','unrun')",
        ),
        "invalid pipeline step transition",
    );
    let c = f.cursor(&id);
    f.store()
        .pipeline_begin_launch(&f.owner, &c, Some("0123456789abcdef"), &json!({}))
        .unwrap();
    let c = f.cursor(&id);
    f.store()
        .pipeline_record_exit(&f.owner, &c, &death(), None, &json!({"exit_code":0}))
        .unwrap();
    let c = f.cursor(&id);
    assert!(
        f.store()
            .pipeline_adopt(&f.owner, &c, &json!({"command_id":"unrun"}))
            .is_err()
    );
}
/// Run head changes require an adopted executor commit from that run.
#[tokio::test]
async fn b2_fix1_head_commit_evidence() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    rejected(
        &f.store().connection,
        &update("pipeline_runs", ",head_commit=printf('%040d',2)"),
        "invalid pipeline run transition",
    );
    let c = f.cursor(&id);
    f.store()
        .pipeline_record_exit(&f.owner, &c, &death(), Some("success"), &json!({}))
        .unwrap();
    let c = f.cursor(&id);
    f.store()
        .pipeline_record_commit(&f.owner, &c, &"b".repeat(40), &"c".repeat(40), &json!({}))
        .unwrap();
    let c = f.cursor(&id);
    f.store().pipeline_adopt(&f.owner, &c, &json!({})).unwrap();
    assert_eq!(
        f.store().pipeline_goal_view(f.goal).unwrap().runs[0].head_commit,
        "b".repeat(40)
    );
}
/// A result tree cannot be recorded before its result commit.
#[tokio::test]
async fn b2_fix1_tree_requires_commit() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    rejected(
        &f.store().connection,
        &update("pipeline_steps", ",result_tree=printf('%040d',2)"),
        "invalid pipeline step transition",
    );
}
/// Journal intent cannot target another ref or give Goal creation a nonzero old OID.
#[tokio::test]
async fn b2_fix1_journal_ref_scope() {
    let f = fixture().await;
    let id = f.claim();
    f.advance_prepare(&id);
    let c = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &c, &planned(&id, 2, 1, "release"), true)
        .unwrap();
    let mut j = release_journal(&f, &id);
    j.ref_name = "refs/heads/main".into();
    let c = f.cursor(&id);
    let error = f
        .store()
        .pipeline_release_begin(&f.owner, &c, &j, &json!({}))
        .unwrap_err();
    assert_eq!(error.to_string(), "pipeline journal ref mismatch");
    let store = f.store();
    let changes = vec![
        ("id", "'other'"),
        ("goal_id", "'other-goal'"),
        ("ref_name", "'refs/heads/main'"),
        ("version", "1"),
        ("state", "'pending'"),
    ];
    rejected(
        &store.connection,
        &copy(&store.connection, "pipeline_ref_journal", &changes),
        "CHECK constraint failed",
    );
}

/// Goal creation always compares against a zero OID in both Store and schema.
#[tokio::test]
async fn b2_fix1_goal_create_zero_oid() {
    let f = fixture().await;
    let mut j = create_journal(&f);
    j.old_commit = "a".repeat(40);
    {
        let mut store = f.store();
        let tx = store.connection.transaction().unwrap();
        let error = insert_journal(&tx, &f.owner, &j).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("pipeline Goal create old is not zero"),
            "{error:#}"
        );
    }
    f.claim();
    let store = f.store();
    let changes = vec![
        ("id", "'other'"),
        ("goal_id", "'other-goal'"),
        ("ref_name", "'refs/heads/rrx/goal/other-goal'"),
        ("old_commit", "printf('%040d',1)"),
        ("version", "1"),
        ("state", "'pending'"),
    ];
    rejected(
        &store.connection,
        &copy(&store.connection, "pipeline_ref_journal", &changes),
        "CHECK constraint failed",
    );
}
/// Retry never consumes an open release journal and preserves the held run.
#[tokio::test]
async fn b2_fix1_retry_open_release() {
    let f = fixture().await;
    let (id, _) = held_release(&f);
    let request = recovery(&f, &id, "retry");
    let before = f.store().pipeline_goal_view(f.goal).unwrap();
    assert!(
        f.store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
            .is_err()
    );
    assert_eq!(
        f.store().pipeline_goal_view(f.goal).unwrap().journals,
        before.journals
    );
}
/// Mode verdicts keep live database work distinct from cookie discovery requirements.
#[tokio::test]
async fn b2_fix1_quiescence_verdict() {
    let f = fixture().await;
    assert_eq!(
        f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .to_managed_native,
        PipelineQuiescenceVerdict::Quiescent
    );
    let id = f.claim();
    f.launching(&id, "implement");
    let v = f.store().pipeline_mode_inventory().unwrap();
    assert_eq!(v.to_managed_native, PipelineQuiescenceVerdict::Busy);
    assert_eq!(v.to_cli_pipeline, PipelineQuiescenceVerdict::Quiescent);
    let c = f.cursor(&id);
    f.store()
        .pipeline_record_exit(&f.owner, &c, &death(), Some("success"), &json!({}))
        .unwrap();
    assert!(
        f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .pipeline_cookies
            .is_empty()
    );
}
/// Missing provider-wait limits consistently use the documented default and reject malformed limits.
#[test]
fn b2_fix1_limit_policy_shape() {
    assert_eq!(pipeline_max_limit_waits(&json!({})).unwrap(), 3);
    for value in [json!(null), json!(-1), json!(7), json!("3")] {
        assert!(pipeline_max_limit_waits(&json!({"max_limit_waits":value})).is_err());
    }
}

/// The required fix-round limit cannot be omitted from an accepted execution policy.
#[test]
fn b2_fix1_fix_policy_required() {
    let mut p = fixture_profile("a".repeat(64));
    p.body.as_object_mut().unwrap().remove("max_fix_rounds");
    assert!(p.policy_digest().is_err());
}

/// Goal acceptance binds the exact profile body and CLI pins to its policy digest.
#[tokio::test]
async fn b2_fix1_policy_acceptance() {
    let f = fixture().await;
    let mut request = f.accepted.clone();
    request.request_id = Uuid::new_v4();
    let mut profile = f.profile.clone();
    profile.body["max_fix_rounds"] = json!(1);
    let error = f
        .store()
        .pipeline_accept_goal(&f.ingress, &request, &f.validated, &profile)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("pipeline accepted policy mismatch"),
        "{error:#}"
    );
    profile.policy_sha256 = profile.policy_digest().unwrap();
    assert!(
        f.store()
            .pipeline_accept_goal(&f.ingress, &request, &f.validated, &profile)
            .is_ok()
    );
}

/// Empty criteria are refused before any pipeline Goal can be accepted.
#[tokio::test]
async fn b2_fix1_empty_criteria() {
    let f = fixture().await;
    let ControlAction::CreateGoal { mut plan, .. } = f.accepted.action.clone() else {
        panic!()
    };
    plan.definition.criteria.clear();
    let error = plan.validate(&Config::default()).err().unwrap();
    assert!(
        error
            .to_string()
            .contains("Goal needs bounded explicit criteria"),
        "{error:#}"
    );
}

/// A Goal check run starts only at the last completed release's commit.
#[tokio::test]
async fn b2_fix1_goal_base_release() {
    let f = fixture().await;
    let task_run = complete_task(&f);
    let id = Uuid::new_v4().to_string();
    let mut run = read_run(&f.store().connection, &task_run).unwrap();
    run.id = id.clone();
    run.kind = "goal".into();
    run.task_id = None;
    run.worktree = None;
    run.branch = None;
    run.version = 0;
    run.state = "running".into();
    run.base_commit = "b".repeat(40);
    run.head_commit = run.base_commit.clone();
    let first = planned(&id, 1, 1, "test");
    let error = f
        .store()
        .pipeline_claim_run(&f.owner, &run, 1, &first, None)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("pipeline Goal base differs from last release"),
        "{error:#}"
    );
    run.base_commit = "a".repeat(40);
    run.head_commit = run.base_commit.clone();
    f.store()
        .pipeline_claim_run(&f.owner, &run, 1, &first, None)
        .unwrap();
}

/// Completion cannot relabel the final command's stored execution evidence.
#[tokio::test]
async fn b2_fix1_complete_command_identity() {
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
    let mut c = PipelineCursor {
        run_id: run.id,
        run_version: 1,
        seq: 1,
        attempt: 1,
        step_version: 1,
        wait_version: None,
    };
    f.store()
        .pipeline_begin_launch(&f.owner, &c, Some("0123456789abcdef"), &json!({}))
        .unwrap();
    c.step_version = 2;
    f.store()
        .pipeline_record_exit(&f.owner, &c, &death(), None, &json!({"exit_code":0}))
        .unwrap();
    c.step_version = 3;
    let error = f
        .store()
        .pipeline_complete_goal(&f.owner, &c, 1, "unrun")
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("pipeline final command differs from plan"),
        "{error:#}"
    );
    f.store()
        .pipeline_complete_goal(&f.owner, &c, 1, "test")
        .unwrap();
}

/// Adoption refuses a cursor with a later attempt or sequence.
#[tokio::test]
async fn b2_fix1_adopt_last_cursor() {
    let f = fixture().await;
    let id = f.claim();
    f.advance_prepare(&id);
    let old = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &old, &planned(&id, 2, 1, "test"), true)
        .unwrap();
    let error = f
        .store()
        .pipeline_adopt(&f.owner, &old, &json!({}))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("pipeline cursor is not last step"),
        "{error:#}"
    );
}

/// Takeover changes only owner epoch and version, retaining every other step fact.
#[tokio::test]
async fn b2_fix1_takeover_all_columns() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    let mut store = f.store();
    store.begin_execution_epoch().unwrap();
    let c = &store.connection;
    c.execute_batch("SAVEPOINT next_owner").unwrap();
    c.execute("UPDATE pipeline_runs SET owner_epoch=owner_epoch+1,version=version+1,body=json_set(body,'$.epoch',owner_epoch+1,'$.version',version+1) WHERE id=?1",[&id]).unwrap();
    let base = ",owner_epoch=owner_epoch+1,body=json_set(body,'$.epoch',owner_epoch+1,'$.version',version+1)";
    permitted(c, &update("pipeline_steps", base));
    for change in [
        ",birth='forged'",
        ",result_tree=printf('%040d',1)",
        ",body=json_set(body,'$.epoch',owner_epoch+1,'$.version',version+1,'$.death_confirmed',json('true'))",
        ",body=json_set(body,'$.epoch',owner_epoch+1,'$.version',version+1,'$.intent','forged')",
    ] {
        rejected(
            c,
            &update("pipeline_steps", &format!("{base}{change}")),
            "invalid pipeline step transition",
        );
    }
    c.execute_batch("ROLLBACK TO next_owner; RELEASE next_owner")
        .unwrap();
}

/// Held run transitions require the recovery action matching their destination.
#[tokio::test]
async fn b2_fix1_recovery_action() {
    for (action, wrong_state) in [("retry", "abandoned"), ("abandon_run", "running")] {
        let f = fixture().await;
        let id = f.claim();
        f.unknown(&id);
        let request = recovery(&f, &id, action);
        let intended = if action == "retry" {
            "running"
        } else {
            "abandoned"
        };
        f.store().connection.execute_batch(&format!("CREATE TEMP TRIGGER probe_recovery_action BEFORE UPDATE ON pipeline_runs WHEN OLD.state='held' AND NEW.state='{intended}' BEGIN UPDATE pipeline_runs SET version=OLD.version+1,state='{wrong_state}',body=json_set(OLD.body,'$.version',OLD.version+1,'$.state','{wrong_state}') WHERE id=OLD.id; END")).unwrap();
        let error = f
            .store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("invalid pipeline run transition"),
            "{error:#}"
        );
        f.store()
            .connection
            .execute_batch("DROP TRIGGER probe_recovery_action")
            .unwrap();
        f.store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
            .unwrap();
    }
}
