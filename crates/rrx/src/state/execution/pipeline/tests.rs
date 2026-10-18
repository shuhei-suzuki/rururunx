use super::*;
use crate::config::{AgentConfig, Config, WorkflowClass};
use crate::execution::verification::{
    AdmittedProfile, Applicability, Category, TestCommand, TestsProfile,
};
use crate::runtime::{
    Runtime,
    control::{ControlAction, ControlRequest, ControlResponse, HumanIngress},
    goal::*,
};
use rusqlite::hooks::{AuthAction, AuthContext, Authorization};

fn historical(path: &Path, version: i64) {
    let mut c = Connection::open(path).unwrap();
    let _permits = register_writer_contract(&c).unwrap();
    if version == 0 {
        return;
    }
    let tx = c.transaction().unwrap();
    if version == 9 {
        for sql in [
            include_str!("../../schema.sql"),
            include_str!("../../execution.sql"),
            include_str!("../native_results.sql"),
            include_str!("../source_recovery.sql"),
            include_str!("../verification.sql"),
            include_str!("../../runtime/schema.sql"),
        ] {
            tx.execute_batch(sql).unwrap();
        }
        tx.execute(
            "INSERT INTO runtime_epoch VALUES(1,?1,0)",
            [Uuid::new_v4().to_string()],
        )
        .unwrap();
        for table in MUTABLE_TABLES
            .iter()
            .filter(|t| !managed_binding::schema::TABLES.contains(t))
        {
            for action in ["INSERT", "UPDATE", "DELETE"] {
                tx.execute_batch(&format!("CREATE TRIGGER writer_{table}_{action} BEFORE {action} ON {table} WHEN rrx_writer_contract_version()<>9 BEGIN SELECT RAISE(ABORT,'incompatible rrx writer contract'); END;")).unwrap();
            }
        }
    } else {
        tx.execute_batch(include_str!("../../schema.sql")).unwrap();
        runtime::install_schema(&tx).unwrap();
        managed_binding::install_schema(&tx).unwrap();
        super::super::install_schema(&tx).unwrap();
        managed_binding::install_retained_guards(&tx).unwrap();
        if version >= 11 {
            install_read_indexes(&tx).unwrap();
        }
        super::super::install_writer_guards_at(&tx, version).unwrap();
    }
    tx.pragma_update(None, "user_version", version).unwrap();
    tx.pragma_update(None, "application_id", APPLICATION_ID)
        .unwrap();
    tx.commit().unwrap();
}
fn catalog(c: &Connection) -> Vec<(String, String)> {
    let mut q = c
        .prepare("SELECT name,sql FROM sqlite_schema WHERE sql IS NOT NULL ORDER BY name")
        .unwrap();
    q.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}
fn upgrade(version: i64, fail: bool) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.db");
    historical(&path, version);
    let c = Connection::open(&path).unwrap();
    let before = catalog(&c);
    if fail {
        c.authorizer(Some(|ctx: AuthContext<'_>| match ctx.action {
            AuthAction::CreateTable {
                table_name: "pipeline_steps",
            } => Authorization::Deny,
            _ => Authorization::Allow,
        }));
        assert!(Store::initialize(c).is_err());
        let c = Connection::open(&path).unwrap();
        assert_eq!(catalog(&c), before);
        assert_eq!(
            c.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            version
        );
        assert_eq!(
            c.pragma_query_value(None, "application_id", |r| r.get::<_, i64>(0))
                .unwrap(),
            if version == 0 { 0 } else { APPLICATION_ID }
        );
    } else {
        let store = Store::initialize(c).unwrap();
        assert_eq!(store.schema_version().unwrap(), 12);
        managed_binding::validate_current_layout(&store.connection).unwrap();
        for table in PIPELINE_TABLES {
            assert_eq!(
                store
                    .connection
                    .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                        .get::<_, u64>(0))
                    .unwrap(),
                0
            );
        }
    }
}
/// Fresh databases publish the complete pipeline layout atomically.
#[test]
fn b2_upgrade_v0() {
    upgrade(0, false);
}
/// Historical unprotected databases retain the ordered upgrade path.
#[test]
fn b2_upgrade_v9() {
    upgrade(9, false);
}
/// Protected binding layouts gain the pipeline without historical drift.
#[test]
fn b2_upgrade_v10() {
    upgrade(10, false);
}
/// Protected indexed layouts gain the pipeline without legacy classification.
#[test]
fn b2_upgrade_v11() {
    upgrade(11, false);
}
/// A failed fresh installation publishes neither tables nor version markers.
#[test]
fn b2_rollback_v0() {
    upgrade(0, true);
}
/// A failed historical upgrade restores its complete original catalog.
#[test]
fn b2_rollback_v9() {
    upgrade(9, true);
}
/// A failed binding upgrade restores the original writer guards and version.
#[test]
fn b2_rollback_v10() {
    upgrade(10, true);
}
/// A failed indexed upgrade retains the original complete indexed layout.
#[test]
fn b2_rollback_v11() {
    upgrade(11, true);
}

struct Fixture {
    _dir: tempfile::TempDir,
    owner: std::sync::Arc<RuntimeOwner>,
    ingress: HumanIngress,
    goal: GoalId,
    task: TaskId,
    project: ProjectId,
}
async fn fixture() -> Fixture {
    fixture_mode(true).await
}
async fn fixture_mode(pipeline: bool) -> Fixture {
    fixture_with_owner(pipeline, None).await
}
async fn fixture_with_owner(
    pipeline: bool,
    shared: Option<std::sync::Arc<RuntimeOwner>>,
) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("source");
    std::fs::create_dir(&root).unwrap();
    let owner = shared.unwrap_or_else(|| RuntimeOwner::open(&dir.path().join("state.db")).unwrap());
    let mut config = Config::default();
    config.agents.insert(
        "codex".into(),
        AgentConfig {
            provider: Some("codex".into()),
            command: vec!["/usr/bin/true".into()],
            ..Default::default()
        },
    );
    config.agents.insert(
        "claude".into(),
        AgentConfig {
            provider: Some("claude".into()),
            command: vec!["/usr/bin/true".into()],
            ..Default::default()
        },
    );
    let runtime = Runtime::new(owner.clone(), config.clone()).unwrap();
    let (stream, _peer) = tokio::net::UnixStream::pair().unwrap();
    let ingress = HumanIngress::from_connection(&runtime, &stream).unwrap();
    let mut store = owner.store.lock().unwrap();
    let mut project = Project::new(
        "pipeline".into(),
        root.canonicalize().unwrap(),
        format!("local-{}", Uuid::new_v4()),
        "main".into(),
    );
    store.put_project(&mut project).unwrap();
    let proposal = TestsProfile {
        commands: vec![TestCommand {
            id: "test".into(),
            category: Category::Tests,
            program: "/usr/bin/true".into(),
            args: vec![project.id.to_string()],
            cwd: ".".into(),
            timeout_seconds: 1,
            drain_seconds: 1,
            stdout_bytes: 1024,
            stderr_bytes: 1024,
        }],
        applicability: BTreeMap::from([
            (Category::Tests, Applicability::Required),
            (
                Category::Typecheck,
                Applicability::NotApplicable {
                    reason: "fixture".into(),
                },
            ),
            (
                Category::Lint,
                Applicability::NotApplicable {
                    reason: "fixture".into(),
                },
            ),
            (
                Category::Build,
                Applicability::NotApplicable {
                    reason: "fixture".into(),
                },
            ),
        ]),
    };
    let admitted = AdmittedProfile::admit(proposal).unwrap();
    let digest = store
        .admit_verification_profile(project.id, project.version, owner.epoch, &admitted)
        .unwrap();
    let plan = GoalPlan {
        definition: GoalDefinition {
            title: "goal".into(),
            objective: "objective".into(),
            criteria: vec![CriterionDefinition {
                id: "verified".into(),
                description: "pass".into(),
                evaluator: CriterionEvaluator::RequiredTasksVerified,
            }],
            constraints: vec![],
            non_goals: vec![],
            source_refs: vec![],
        },
        tasks: vec![TaskDefinition {
            key: "task".into(),
            title: "task".into(),
            acceptance_criteria: vec!["pass".into()],
            executor: "codex".into(),
            reviewers: vec!["claude".into()],
            workflow: WorkflowClass::Standard,
            risk: RiskClass::R2,
        }],
        dependencies: vec![],
    };
    let validated = plan.clone().validate(&config).unwrap();
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: owner.instance.clone(),
        epoch: owner.epoch,
        action: ControlAction::CreateGoal {
            project: project.id,
            expected_project: project.version,
            plan,
        },
    };
    let response = if pipeline {
        store
            .pipeline_accept_goal(
                &ingress,
                &request,
                &validated,
                &PipelineProfile {
                    policy_sha256: "a".repeat(64),
                    tests_profile_digest: digest,
                    body: json!({"max_fix_rounds":2,"max_limit_waits":3}),
                },
            )
            .unwrap()
    } else {
        store
            .accept_runtime_goal(&ingress, &request, &validated, &"a".repeat(64))
            .unwrap()
    };
    let ControlResponse::GoalAccepted { goal, .. } = response else {
        panic!("Goal acceptance failed")
    };
    let task = store.tasks(project.id, Some(goal)).unwrap()[0].id;
    drop(store);
    Fixture {
        _dir: dir,
        owner,
        ingress,
        goal,
        task,
        project: project.id,
    }
}
impl Fixture {
    fn store(&self) -> std::sync::MutexGuard<'_, Store> {
        self.owner.store.lock().unwrap()
    }
    fn claim(&self) -> String {
        let id = Uuid::new_v4().to_string();
        let run = PipelineRun {
            id: id.clone(),
            kind: "task".into(),
            task_id: Some(self.task),
            goal_id: self.goal,
            project_id: self.project,
            owner_epoch: self.owner.epoch,
            version: 0,
            state: "running".into(),
            round: 0,
            base_commit: "a".repeat(40),
            head_commit: "a".repeat(40),
            worktree: Some(format!("/pipeline/{id}")),
            branch: Some(format!("rrx/pipeline/{id}")),
            body: json!({}),
        };
        let first = planned(&id, 1, 1, "prepare");
        self.store()
            .pipeline_claim_run(&self.owner, &run, 1, &first, Some(&create_journal(self)))
            .unwrap();
        id
    }
    fn cursor(&self, id: &str) -> PipelineCursor {
        let store = self.store();
        let view = store.pipeline_task_view(self.goal, self.task).unwrap();
        let run = view.runs.iter().find(|r| r.id == id).unwrap();
        let step = view.steps.last().unwrap();
        PipelineCursor {
            run_id: id.into(),
            run_version: run.version,
            seq: step.seq,
            attempt: step.attempt,
            step_version: step.version,
            wait_version: view.provider_wait.map(|w| w.version),
        }
    }
    fn advance_prepare(&self, id: &str) {
        let cursor = self.cursor(id);
        self.store()
            .pipeline_begin_launch(&self.owner, &cursor, None, &json!({}))
            .unwrap();
        let create = self
            .store()
            .pipeline_goal_view(self.goal)
            .unwrap()
            .journals
            .into_iter()
            .find(|j| j.kind == "goal_create" && j.state == "pending");
        if let Some(create) = create {
            let cursor = self.cursor(id);
            self.store()
                .pipeline_release_settle(
                    &self.owner,
                    &cursor,
                    &create.id,
                    create.version,
                    &create.new_commit,
                )
                .unwrap();
        }
        let cursor = self.cursor(id);
        self.store()
            .pipeline_adopt(&self.owner, &cursor, &json!({}))
            .unwrap();
    }
    fn process_step(&self, id: &str, kind: &str) {
        self.advance_prepare(id);
        let cursor = self.cursor(id);
        self.store()
            .pipeline_plan_next(&self.owner, &cursor, &planned(id, 2, 1, kind), true)
            .unwrap();
        if let Some(provider) = &planned(id, 2, 1, kind).provider {
            let qualified: bool = self.store().connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM pipeline_qualifications WHERE owner_epoch=?1 AND subject=?2)",
                params![self.owner.epoch, provider], |r| r.get(0),
            ).unwrap();
            if qualified {
                return;
            }
            self.store()
                .pipeline_record_qualification(
                    &self.owner,
                    &PipelineQualification {
                        subject: provider.clone(),
                        state: "qualified".into(),
                        executable_sha256: Some("a".repeat(64)),
                        version_text: Some("fixture".into()),
                        body: json!({}),
                    },
                )
                .unwrap();
        }
    }
    fn launching(&self, id: &str, kind: &str) {
        self.process_step(id, kind);
        let cursor = self.cursor(id);
        self.store()
            .pipeline_begin_launch(&self.owner, &cursor, Some("0123456789abcdef"), &json!({}))
            .unwrap();
    }
    fn unknown(&self, id: &str) {
        self.launching(id, "implement");
        let cursor = self.cursor(id);
        self.store()
            .pipeline_record_launch_unknown(&self.owner, &cursor, "lost")
            .unwrap();
    }
}
fn planned(run: &str, seq: u32, attempt: u32, kind: &str) -> PipelineStep {
    PipelineStep {
        run_id: run.into(),
        seq,
        attempt,
        kind: kind.into(),
        provider: match kind {
            "implement" | "fix" | "confirm" => Some("codex".into()),
            "review" => Some("claude".into()),
            _ => None,
        },
        owner_epoch: 0,
        round: 0,
        state: "planned".into(),
        version: 0,
        cookie: None,
        pid: None,
        birth: None,
        step_base: "a".repeat(40),
        input_sha256: "a".repeat(64),
        result_commit: None,
        result_tree: None,
        body: json!({}),
    }
}
fn death() -> PipelineDeathEvidence {
    PipelineDeathEvidence {
        leader_exit_observed: true,
        group_kill_delivered: true,
        leader_reaped: true,
        group_empty: true,
        discovery_complete: true,
        matching_processes: 0,
    }
}
fn raw_step(store: &Store, step: &PipelineStep, epoch: u64, state: &str) -> Result<()> {
    let version = step.version + 1;
    let body = stamped(&step.body, version, state, epoch)?;
    store.connection.execute("UPDATE pipeline_steps SET owner_epoch=?1,version=?2,state=?3,body=?4 WHERE run_id=?5 AND seq=?6 AND attempt=?7",params![epoch,version,state,body,step.run_id,step.seq,step.attempt])?;
    Ok(())
}
fn recovery(f: &Fixture, id: &str, action: &str) -> PipelineRecoveryRequest {
    PipelineRecoveryRequest {
        request_id: Uuid::new_v4(),
        cursor: f.cursor(id),
        then_action: action.into(),
        reason: "operator decision".into(),
    }
}
fn recovery_evidence() -> PipelineRecoveryEvidence {
    PipelineRecoveryEvidence {
        discovery_complete: true,
        matching_processes: 0,
        workspace_verified: true,
        journal: None,
        body: json!({}),
    }
}

fn complete_task(f: &Fixture) -> String {
    let id = f.claim();
    f.advance_prepare(&id);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &planned(&id, 2, 1, "release"), true)
        .unwrap();
    let journal = release_journal(f, &id);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_release_begin(&f.owner, &cursor, &journal, &json!({"approved":true}))
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_release_settle(&f.owner, &cursor, &journal.id, 1, &journal.new_commit)
        .unwrap();
    id
}
fn complete_goal(f: &Fixture) {
    complete_task(f);
    let id = Uuid::new_v4().to_string();
    let run = PipelineRun {
        id: id.clone(),
        kind: "goal".into(),
        task_id: None,
        goal_id: f.goal,
        project_id: f.project,
        owner_epoch: 0,
        version: 0,
        state: "running".into(),
        round: 0,
        base_commit: "a".repeat(40),
        head_commit: "a".repeat(40),
        worktree: None,
        branch: None,
        body: json!({}),
    };
    f.store()
        .pipeline_claim_run(&f.owner, &run, 1, &planned(&id, 1, 1, "test"), None)
        .unwrap();
    let cursor = PipelineCursor {
        run_id: id.clone(),
        run_version: 1,
        seq: 1,
        attempt: 1,
        step_version: 1,
        wait_version: None,
    };
    f.store()
        .pipeline_begin_launch(&f.owner, &cursor, Some("0123456789abcdef"), &json!({}))
        .unwrap();
    let mut cursor = cursor;
    cursor.step_version = 2;
    f.store()
        .pipeline_record_exit(&f.owner, &cursor, &death(), None, &json!({"exit_code":0}))
        .unwrap();
    cursor.step_version = 3;
    f.store()
        .pipeline_complete_goal(&f.owner, &cursor, 1, "test")
        .unwrap();
}
fn no_replace(f: &Fixture, table: &str) {
    let store = f.store();
    store
        .connection
        .pragma_update(None, "recursive_triggers", false)
        .unwrap();
    let before: u64 = store
        .connection
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap();
    assert!(before > 0);
    let error = store
        .connection
        .execute(
            &format!("INSERT OR REPLACE INTO {table} SELECT * FROM {table} LIMIT 1"),
            [],
        )
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("pipeline row cannot be replaced"),
        "{table}: {error}"
    );
    assert_eq!(
        store
            .connection
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        before
    );
}
/// Accepted mode records reject replacement with recursive triggers disabled.
#[tokio::test]
async fn b2_profile_no_replace() {
    let f = fixture().await;
    no_replace(&f, "goal_execution_profiles");
}
/// Run identity cannot be replaced through a conflicting primary key.
#[tokio::test]
async fn b2_run_no_replace() {
    let f = fixture().await;
    f.claim();
    no_replace(&f, "pipeline_runs");
}
/// Step attempts cannot be replaced through a conflicting primary key.
#[tokio::test]
async fn b2_step_no_replace() {
    let f = fixture().await;
    f.claim();
    no_replace(&f, "pipeline_steps");
}
/// Operator recovery records reject replacement after their response is stored.
#[tokio::test]
async fn b2_recovery_no_replace() {
    let f = fixture().await;
    let id = f.claim();
    f.unknown(&id);
    let request = recovery(&f, &id, "retry");
    f.store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
        .unwrap();
    no_replace(&f, "pipeline_recoveries");
}
/// Ref journals reject replacement after release settlement.
#[tokio::test]
async fn b2_journal_no_replace() {
    let f = fixture().await;
    f.claim();
    no_replace(&f, "pipeline_ref_journal");
}
/// Epoch qualification records reject replacement of the same subject.
#[tokio::test]
async fn b2_qualification_no_replace() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    no_replace(&f, "pipeline_qualifications");
}
/// Goal completion evidence cannot be replaced after completion.
#[tokio::test]
async fn b2_completion_no_replace() {
    let f = fixture().await;
    complete_goal(&f);
    no_replace(&f, "pipeline_goal_completions");
}
/// The global provider wait cannot be reset by replacing its singleton row.
#[tokio::test]
async fn b2_wait_no_replace() {
    let f = fixture().await;
    {
        let mut store = f.store();
        let tx = store.connection.transaction().unwrap();
        write_wait(&tx, &f.owner, &initial_wait(&f.owner)).unwrap();
        tx.commit().unwrap();
    }
    no_replace(&f, "pipeline_limit_wait");
}

/// A pipeline claim refuses Goals without an accepted pipeline mode record.
#[tokio::test]
async fn b2_pipeline_refuses_native_goal() {
    let f = fixture_mode(false).await;
    let id = Uuid::new_v4().to_string();
    let run = PipelineRun {
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
    assert!(
        f.store()
            .pipeline_claim_run(
                &f.owner,
                &run,
                1,
                &planned(&id, 1, 1, "prepare"),
                Some(&create_journal(&f))
            )
            .unwrap_err()
            .to_string()
            .contains("ExecutionModeMismatch")
    );
}
/// A terminal step cannot return to a runnable state through raw SQL.
#[tokio::test]
async fn b2_terminal_step_cannot_restart() {
    let f = fixture().await;
    let id = f.claim();
    f.advance_prepare(&id);
    let store = f.store();
    let step = read_step(&store.connection, &id, 1, 1).unwrap();
    assert!(raw_step(&store, &step, f.owner.epoch, "running").is_err());
}
/// Release settlement preserves terminal journal decisions.
#[tokio::test]
async fn b2_terminal_journal_cannot_reopen() {
    let f = fixture().await;
    let id = complete_task(&f);
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    let mut journal = tx
        .query_row(
            &format!("SELECT {JOURNAL_COLUMNS} FROM pipeline_ref_journal WHERE run_id=?1"),
            [&id],
            journal_row,
        )
        .unwrap();
    journal.state = "pending".into();
    assert!(write_journal(&tx, &f.owner, &journal).is_err());
}

/// Recovery retry commits its first response once and replays it before stale versions.
#[tokio::test]
async fn b2_recover_retry_replays_once() {
    let f = fixture().await;
    let id = f.claim();
    f.unknown(&id);
    let request = recovery(&f, &id, "retry");
    let first = f
        .store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
        .unwrap();
    let second = f
        .store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
        .unwrap();
    assert_eq!(first, second);
    assert_eq!(first.next_attempt, Some(2));
    let store = f.store();
    assert_eq!(
        store
            .connection
            .query_row("SELECT count(*) FROM pipeline_recoveries", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        store
            .pipeline_task_view(f.goal, f.task)
            .unwrap()
            .steps
            .len(),
        3
    );
}
/// Recovery abandonment commits its first response once and replays it after termination.
#[tokio::test]
async fn b2_recover_abandon_replays_once() {
    let f = fixture().await;
    let id = f.claim();
    f.unknown(&id);
    let request = recovery(&f, &id, "abandon_run");
    let first = f
        .store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
        .unwrap();
    let second = f
        .store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
        .unwrap();
    assert_eq!(first, second);
    assert_eq!(first.state, "abandoned");
    assert_eq!(
        f.store()
            .connection
            .query_row("SELECT count(*) FROM pipeline_recoveries", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        1
    );
}
/// A recovery with stale independent step version writes no acknowledgement.
#[tokio::test]
async fn b2_recover_stale_step_rolls_back() {
    let f = fixture().await;
    let id = f.claim();
    f.unknown(&id);
    let mut request = recovery(&f, &id, "retry");
    request.cursor.step_version -= 1;
    assert!(
        f.store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
            .is_err()
    );
    assert_eq!(
        f.store()
            .connection
            .query_row("SELECT count(*) FROM pipeline_recoveries", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        0
    );
}
/// A live matching process prevents operator recovery even with current versions.
#[tokio::test]
async fn b2_recover_requires_empty_discovery() {
    let f = fixture().await;
    let id = f.claim();
    f.unknown(&id);
    let request = recovery(&f, &id, "retry");
    let mut evidence = recovery_evidence();
    evidence.matching_processes = 1;
    assert!(
        f.store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &evidence)
            .is_err()
    );
}
/// Unknown steps cannot become runnable without an operator recovery record.
#[tokio::test]
async fn b2_unknown_cannot_restart() {
    let f = fixture().await;
    let id = f.claim();
    f.unknown(&id);
    let store = f.store();
    let step = store
        .pipeline_task_view(f.goal, f.task)
        .unwrap()
        .steps
        .pop()
        .unwrap();
    assert!(raw_step(&store, &step, f.owner.epoch, "running").is_err());
}
/// Agent steps reject a NULL provider independently of Store validation.
#[tokio::test]
async fn b2_agent_null_provider_rejected() {
    let f = fixture().await;
    let id = f.claim();
    f.advance_prepare(&id);
    let mut next = planned(&id, 2, 1, "implement");
    next.provider = None;
    let cursor = f.cursor(&id);
    assert!(
        f.store()
            .pipeline_plan_next(&f.owner, &cursor, &next, true)
            .is_err()
    );
}
/// A pipeline run cannot be abandoned while an unknown step may still be alive.
#[tokio::test]
async fn b2_abandon_rejects_unknown() {
    let f = fixture().await;
    let id = f.claim();
    f.unknown(&id);
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    let mut run = read_run(&tx, &id).unwrap();
    run.state = "abandoned".into();
    assert!(write_run(&tx, &f.owner, &run).is_err());
}
/// Native candidate enumeration excludes accepted pipeline Goals.
#[tokio::test]
async fn b2_native_candidates_exclude_pipeline() {
    let f = fixture().await;
    let page = f
        .store()
        .ready_driver_candidates(&f.owner.instance, f.owner.epoch, None, 4)
        .unwrap();
    assert!(matches!(page,crate::state::CandidatePage::Rows{keys,..} if keys.is_empty()));
}

/// A takeover fences stale step writes at the trigger without invoking the API guard.
#[tokio::test]
async fn b2_stale_step_trigger_after_takeover() {
    let f = fixture().await;
    let id = f.claim();
    f.unknown(&id);
    let epoch = f.owner.epoch;
    let path = f.owner.state_path().to_path_buf();
    let goal = f.goal;
    let task = f.task;
    drop(f.ingress);
    let released = std::sync::Arc::downgrade(&f.owner);
    drop(f.owner);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    while released.upgrade().is_some() {
        assert!(
            tokio::time::Instant::now() < deadline,
            "owner custody not released"
        );
        tokio::task::yield_now().await;
    }
    let owner = RuntimeOwner::open(&path).unwrap();
    assert!(owner.epoch > epoch);
    let mut store = owner.store.lock().unwrap();
    let expected = store.pipeline_takeover_view().unwrap();
    store.pipeline_take_over(&owner, &expected).unwrap();
    let step = store
        .pipeline_task_view(goal, task)
        .unwrap()
        .steps
        .pop()
        .unwrap();
    assert!(
        raw_step(&store, &step, epoch, &step.state)
            .unwrap_err()
            .to_string()
            .contains("pipeline owner retired")
    );
    raw_step(&store, &step, owner.epoch, &step.state).unwrap();
}

mod round2;
/// Step versions advance exactly once on every permitted update.
#[tokio::test]
async fn b2_step_version_is_monotonic() {
    let f = fixture().await;
    let id = f.claim();
    let store = f.store();
    assert!(
        store
            .connection
            .execute(
                "UPDATE pipeline_steps SET version=version WHERE run_id=?1",
                [&id]
            )
            .is_err()
    );
    let step = read_step(&store.connection, &id, 1, 1).unwrap();
    raw_step(&store, &step, f.owner.epoch, "planned").unwrap();
}
/// Run versions advance exactly once on every permitted update.
#[tokio::test]
async fn b2_run_version_is_monotonic() {
    let f = fixture().await;
    let id = f.claim();
    let mut store = f.store();
    assert!(
        store
            .connection
            .execute(
                "UPDATE pipeline_runs SET version=version WHERE id=?1",
                [&id]
            )
            .is_err()
    );
    let tx = store.connection.transaction().unwrap();
    let run = read_run(&tx, &id).unwrap();
    write_run(&tx, &f.owner, &run).unwrap();
}
/// A held run requires an operator recovery record before becoming running.
#[tokio::test]
async fn b2_held_run_requires_recovery() {
    let f = fixture().await;
    let id = f.claim();
    f.unknown(&id);
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    let mut run = read_run(&tx, &id).unwrap();
    run.state = "running".into();
    assert!(write_run(&tx, &f.owner, &run).is_err());
}
/// A completed task run requires a done release journal in the same transaction.
#[tokio::test]
async fn b2_task_completion_requires_journal() {
    let f = fixture().await;
    let id = f.claim();
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    let mut run = read_run(&tx, &id).unwrap();
    run.state = "completed".into();
    assert!(write_run(&tx, &f.owner, &run).is_err());
}
/// A new step sequence requires an adopted predecessor.
#[tokio::test]
async fn b2_step_sequence_requires_adoption() {
    let f = fixture().await;
    let id = f.claim();
    f.advance_prepare(&id);
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    assert!(insert_step(&tx, &f.owner, &planned(&id, 3, 1, "test")).is_err());
}
/// A new attempt requires the previous attempt to be retryable.
#[tokio::test]
async fn b2_step_attempt_requires_retryable_predecessor() {
    let f = fixture().await;
    let id = f.claim();
    f.advance_prepare(&id);
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    assert!(insert_step(&tx, &f.owner, &planned(&id, 1, 2, "prepare")).is_err());
}
/// A recorded provider outcome cannot be changed after exit classification.
#[tokio::test]
async fn b2_provider_outcome_is_immutable() {
    let f = fixture().await;
    let id = f.claim();
    f.launching(&id, "implement");
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(
            &f.owner,
            &cursor,
            &death(),
            Some("limit"),
            &json!({"exit_code":1}),
        )
        .unwrap();
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    let mut step = read_step(&tx, &id, 2, 1).unwrap();
    step.body["provider_outcome"] = json!("success");
    assert!(write_step(&tx, &f.owner, &step).is_err());
}
fn initial_wait(owner: &RuntimeOwner) -> PipelineLimitWait {
    PipelineLimitWait {
        owner_epoch: owner.epoch,
        version: 0,
        state: "waiting".into(),
        not_before: 1,
        backoff_ms: 120000,
        probe_run: None,
        probe_seq: None,
        probe_attempt: None,
        body: json!({}),
    }
}
/// Time passage alone cannot clear a provider wait.
#[tokio::test]
async fn b2_wait_cannot_clear_from_waiting() {
    let f = fixture().await;
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    write_wait(&tx, &f.owner, &initial_wait(&f.owner)).unwrap();
    let mut wait = read_wait(&tx).unwrap().unwrap();
    wait.state = "clear".into();
    wait.backoff_ms = 60000;
    assert!(write_wait(&tx, &f.owner, &wait).is_err());
}
/// A provider wait deadline cannot move backwards.
#[tokio::test]
async fn b2_wait_deadline_is_monotonic() {
    let f = fixture().await;
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    let mut wait = initial_wait(&f.owner);
    wait.not_before = 10;
    write_wait(&tx, &f.owner, &wait).unwrap();
    let mut wait = read_wait(&tx).unwrap().unwrap();
    wait.not_before = 9;
    wait.backoff_ms *= 2;
    assert!(write_wait(&tx, &f.owner, &wait).is_err());
    wait.not_before = 11;
    write_wait(&tx, &f.owner, &wait).unwrap();
}
/// A probe with a classified known success clears the global provider wait.
#[tokio::test]
async fn b2_wait_probe_success_clears() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    {
        let mut store = f.store();
        let tx = store.connection.transaction().unwrap();
        write_wait(&tx, &f.owner, &initial_wait(&f.owner)).unwrap();
        tx.commit().unwrap();
    }
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_begin_launch(&f.owner, &cursor, Some("0123456789abcdef"), &json!({}))
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(
            &f.owner,
            &cursor,
            &death(),
            Some("success"),
            &json!({"exit_code":0}),
        )
        .unwrap();
    assert_eq!(
        f.store()
            .pipeline_mode_inventory()
            .unwrap()
            .provider_wait
            .unwrap()
            .state,
        "clear"
    );
}
/// A probe with an unclassified failure extends rather than clears the wait.
#[tokio::test]
async fn b2_wait_probe_other_extends() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    {
        let mut store = f.store();
        let tx = store.connection.transaction().unwrap();
        write_wait(&tx, &f.owner, &initial_wait(&f.owner)).unwrap();
        tx.commit().unwrap();
    }
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_begin_launch(&f.owner, &cursor, Some("0123456789abcdef"), &json!({}))
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(
            &f.owner,
            &cursor,
            &death(),
            Some("other"),
            &json!({"exit_code":1}),
        )
        .unwrap();
    let wait = f
        .store()
        .pipeline_mode_inventory()
        .unwrap()
        .provider_wait
        .unwrap();
    assert_eq!(wait.state, "waiting");
    assert_eq!(wait.backoff_ms, 240000);
}
/// An unresolved probe cannot be cleared before its recorded successful exit.
#[tokio::test]
async fn b2_wait_clear_requires_successful_probe() {
    let f = fixture().await;
    let id = f.claim();
    f.process_step(&id, "implement");
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    write_wait(&tx, &f.owner, &initial_wait(&f.owner)).unwrap();
    let mut wait = read_wait(&tx).unwrap().unwrap();
    wait.state = "probing".into();
    wait.probe_run = Some(id);
    wait.probe_seq = Some(2);
    wait.probe_attempt = Some(1);
    write_wait(&tx, &f.owner, &wait).unwrap();
    let mut wait = read_wait(&tx).unwrap().unwrap();
    wait.state = "clear".into();
    wait.backoff_ms = 60000;
    wait.probe_run = None;
    wait.probe_seq = None;
    wait.probe_attempt = None;
    assert!(write_wait(&tx, &f.owner, &wait).is_err());
}
/// An open release journal prevents run abandonment until recovery settles it.
#[tokio::test]
async fn b2_abandon_requires_closed_journal() {
    let f = fixture().await;
    let id = f.claim();
    f.advance_prepare(&id);
    let next = planned(&id, 2, 1, "release");
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap();
    let journal = release_journal(&f, &id);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_release_begin(&f.owner, &cursor, &journal, &json!({"approved":true}))
        .unwrap();
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    let mut step = read_step(&tx, &id, 2, 1).unwrap();
    step.state = "interrupted".into();
    write_step(&tx, &f.owner, &step).unwrap();
    let mut run = read_run(&tx, &id).unwrap();
    run.state = "abandoned".into();
    assert!(write_run(&tx, &f.owner, &run).is_err());
}
fn create_journal(f: &Fixture) -> PipelineJournal {
    let mut journal = release_journal(f, "");
    journal.run_id = None;
    journal.kind = "goal_create".into();
    journal.old_commit = "0".repeat(40);
    journal
}
fn release_journal(f: &Fixture, id: &str) -> PipelineJournal {
    PipelineJournal {
        id: Uuid::new_v4().to_string(),
        goal_id: f.goal,
        project_id: f.project,
        run_id: Some(id.into()),
        kind: "task_release".into(),
        ref_name: format!("refs/heads/rrx/goal/{}", f.goal),
        old_commit: "a".repeat(40),
        new_commit: "a".repeat(40),
        owner_epoch: 0,
        state: "pending".into(),
        version: 0,
        body: json!({}),
    }
}
/// A recovery failure after writing its ack rolls back the entire ordered transaction.
#[tokio::test]
async fn b2_recovery_midwrite_rollback() {
    let f = fixture().await;
    let id = f.claim();
    f.unknown(&id);
    let request = recovery(&f, &id, "retry");
    {
        let store = f.store();
        store
            .connection
            .authorizer(Some(|ctx: AuthContext<'_>| match ctx.action {
                AuthAction::Insert {
                    table_name: "pipeline_recoveries",
                } => Authorization::Deny,
                _ => Authorization::Allow,
            }));
    }
    assert!(
        f.store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
            .is_err()
    );
    let store = f.store();
    assert_eq!(
        store
            .connection
            .query_row(
                "SELECT count(*) FROM runtime_control_acks WHERE request_id=?1",
                [request.request_id.to_string()],
                |r| r.get::<_, u64>(0)
            )
            .unwrap(),
        0
    );
}
/// Request ID replay requires identical recovery action and arguments.
#[tokio::test]
async fn b2_recovery_replay_checks_identity() {
    let f = fixture().await;
    let id = f.claim();
    f.unknown(&id);
    let mut request = recovery(&f, &id, "retry");
    f.store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
        .unwrap();
    request.reason = "changed action".into();
    assert!(
        f.store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &recovery_evidence())
            .is_err()
    );
}
/// Native legacy classification refuses pipeline Goals at its owning gate.
#[tokio::test]
async fn b2_native_gate_excludes_pipeline() {
    let f = fixture().await;
    assert!(
        f.store()
            .managed_phase_required(&Scope {
                project_id: f.project,
                goal_id: Some(f.goal),
                task_id: Some(f.task)
            })
            .is_err()
    );
}
/// Inventory preserves unknown process evidence without advancing the epoch.
#[tokio::test]
async fn b2_inventory_is_read_only() {
    let f = fixture().await;
    let id = f.claim();
    f.unknown(&id);
    let before = f.cursor(&id);
    let out = f.store().pipeline_mode_inventory().unwrap();
    assert_eq!(out.pipeline_processes, 1);
    assert_eq!(out.pipeline_cookies, vec!["0123456789abcdef"]);
    assert_eq!(f.cursor(&id).run_version, before.run_version);
}

fn held_release(f: &Fixture) -> (String, PipelineJournal) {
    let id = f.claim();
    f.launching(&id, "implement");
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_record_exit(
            &f.owner,
            &cursor,
            &death(),
            Some("success"),
            &json!({"exit_code":0}),
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
    let cursor = f.cursor(&id);
    let mut next = planned(&id, 3, 1, "release");
    next.step_base = "b".repeat(40);
    f.store()
        .pipeline_plan_next(&f.owner, &cursor, &next, true)
        .unwrap();
    let mut j = release_journal(f, &id);
    j.new_commit = "b".repeat(40);
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_release_begin(&f.owner, &cursor, &j, &json!({"approved":true}))
        .unwrap();
    let cursor = f.cursor(&id);
    f.store()
        .pipeline_hold(&f.owner, &cursor, "release_held", None)
        .unwrap();
    (id, j)
}
/// Abandon recovery closes an unapplied release journal before abandoning the run.
#[tokio::test]
async fn b2_recovery_aborts_unapplied_release() {
    let f = fixture().await;
    let (id, j) = held_release(&f);
    let request = recovery(&f, &id, "abandon_run");
    let mut evidence = recovery_evidence();
    evidence.journal = Some(PipelineRecoveryJournal {
        id: j.id.clone(),
        version: 1,
        observed_commit: j.old_commit,
    });
    let response = f
        .store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &evidence)
        .unwrap();
    assert_eq!(response.state, "abandoned");
    let mut store = f.store();
    let tx = store.connection.transaction().unwrap();
    let mut j = tx
        .query_row(
            &format!("SELECT {JOURNAL_COLUMNS} FROM pipeline_ref_journal WHERE id=?1"),
            [&j.id],
            journal_row,
        )
        .unwrap();
    assert_eq!(j.state, "aborted");
    assert!(write_journal(&tx, &f.owner, &j).is_err());
    j.state = "pending".into();
    assert!(write_journal(&tx, &f.owner, &j).is_err());
}
/// Abandon recovery records an already applied release as completed with stable replay.
#[tokio::test]
async fn b2_recovery_completes_applied_release() {
    let f = fixture().await;
    let (id, j) = held_release(&f);
    let request = recovery(&f, &id, "abandon_run");
    let mut evidence = recovery_evidence();
    evidence.journal = Some(PipelineRecoveryJournal {
        id: j.id,
        version: 1,
        observed_commit: j.new_commit,
    });
    let first = f
        .store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &evidence)
        .unwrap();
    let second = f
        .store()
        .pipeline_recover(&f.owner, &f.ingress, &request, &evidence)
        .unwrap();
    assert_eq!(first, second);
    assert_eq!(first.state, "completed");
    assert!(first.release_applied);
}
/// Recovery refuses an unexpected ref without recording an acknowledgement.
#[tokio::test]
async fn b2_recovery_refuses_unexpected_ref() {
    let f = fixture().await;
    let (id, j) = held_release(&f);
    let request = recovery(&f, &id, "abandon_run");
    let mut evidence = recovery_evidence();
    evidence.journal = Some(PipelineRecoveryJournal {
        id: j.id,
        version: 1,
        observed_commit: "d".repeat(40),
    });
    assert!(
        f.store()
            .pipeline_recover(&f.owner, &f.ingress, &request, &evidence)
            .is_err()
    );
    assert_eq!(
        f.store()
            .connection
            .query_row(
                "SELECT count(*) FROM runtime_control_acks WHERE request_id=?1",
                [request.request_id.to_string()],
                |r| r.get::<_, u64>(0)
            )
            .unwrap(),
        0
    );
}
