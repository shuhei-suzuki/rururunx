use rrx::{
    cli::{client, endpoint, transport},
    config::WorkflowClass,
    domain::{CriterionEvaluator, ProjectId, RiskClass},
    runtime::{
        control::{ControlAction, ControlRequest, ControlResponse},
        goal::{CriterionDefinition, GoalDefinition, GoalPlan, PlanDependency, TaskDefinition},
    },
};
use serde::Serialize;
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    time::{Duration, Instant},
};
use uuid::Uuid;

struct Fixture {
    directory: tempfile::TempDir,
    state: PathBuf,
    config: PathBuf,
    child: Option<Child>,
}
impl Fixture {
    fn new() -> Self {
        let directory = tempfile::Builder::new()
            .prefix("rrx-goal-cli-")
            .tempdir_in("/tmp")
            .unwrap();
        let state = directory.path().join("state.db");
        let config = directory.path().join("config.toml");
        std::fs::write(
            &config,
            "[agents.worker]\nprovider='claude'\ncommand=['/usr/bin/false']\n",
        )
        .unwrap();
        Self {
            directory,
            state,
            config,
            child: None,
        }
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rrx"));
        command
            .arg("--state")
            .arg(&self.state)
            .arg("--config")
            .arg(&self.config)
            .current_dir(self.directory.path());
        command
    }
    fn output(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }
    fn json(&self, args: &[&str]) -> Value {
        let output = self.output(args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
    fn project(&self, name: &str) -> ProjectId {
        let root = self.directory.path().join(name);
        std::fs::create_dir(&root).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "--quiet", "--initial-branch=main"])
                .arg(&root)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .current_dir(&root)
                .args([
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "commit",
                    "--allow-empty",
                    "--quiet",
                    "-m",
                    "initial"
                ])
                .status()
                .unwrap()
                .success()
        );
        let output = self
            .command()
            .args(["project", "add"])
            .arg(root)
            .args(["--name", name])
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap()
    }
    async fn start(&mut self) {
        self.child = Some(
            self.command()
                .arg("serve")
                .stdout(Stdio::null())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(
                self.child.as_mut().unwrap().try_wait().unwrap().is_none(),
                "compiled serve exited"
            );
            if client::request(&self.state, ControlAction::RuntimeStatus)
                .await
                .is_ok()
            {
                break;
            }
            assert!(Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
    async fn stop(&mut self) {
        assert!(matches!(
            client::request(&self.state, ControlAction::RuntimeStop)
                .await
                .unwrap(),
            ControlResponse::RuntimeStopped { .. }
        ));
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if let Some(status) = self.child.as_mut().unwrap().try_wait().unwrap() {
                assert!(status.success());
                self.child.take();
                break;
            }
            assert!(Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
    fn read_db(&self) -> rusqlite::Connection {
        rusqlite::Connection::open_with_flags(
            &self.state,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap()
    }
    fn count(&self, table: &str) -> u64 {
        self.read_db()
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }
    fn plan_file(&self, project: &str, version: u64, plan: GoalPlan) -> PathBuf {
        #[derive(Serialize)]
        struct PlanInput<'a> {
            project: &'a str,
            expected_project: u64,
            plan: GoalPlan,
        }
        let path = self
            .directory
            .path()
            .join(format!("{}.toml", Uuid::new_v4()));
        std::fs::write(
            &path,
            toml::to_string(&PlanInput {
                project,
                expected_project: version,
                plan,
            })
            .unwrap(),
        )
        .unwrap();
        path
    }
    fn accepted(&self, project: &str) -> Value {
        let path = self.plan_file(project, 1, plan());
        self.json(&["goal", "--plan", path.to_str().unwrap(), "--json"])
    }
    async fn stable(&self) -> Vec<(String, Vec<Vec<String>>)> {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            let before = rows(&self.state);
            tokio::time::sleep(Duration::from_millis(200)).await;
            if before == rows(&self.state) {
                return before;
            }
            assert!(Instant::now() < deadline, "no idle whole-row baseline");
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Graceful signal preserves owned IPC cleanup even on assertion failure.
            let _ = Command::new("/bin/kill")
                .args(["-TERM", &child.id().to_string()])
                .status();
            let deadline = Instant::now() + Duration::from_secs(15);
            while child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(20));
            }
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
fn rows(state: &Path) -> Vec<(String, Vec<Vec<String>>)> {
    let db =
        rusqlite::Connection::open_with_flags(state, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    let tables = db
        .prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    tables
        .into_iter()
        .map(|table| {
            let mut query = db
                .prepare(&format!("SELECT * FROM \"{}\"", table.replace('"', "\"\"")))
                .unwrap();
            let columns = query.column_count();
            let mut data = query
                .query_map([], |r| {
                    Ok((0..columns)
                        .map(|i| format!("{:?}", r.get_ref(i).unwrap()))
                        .collect::<Vec<_>>())
                })
                .unwrap()
                .map(Result::unwrap)
                .collect::<Vec<_>>();
            data.sort();
            (table, data)
        })
        .collect()
}
fn plan() -> GoalPlan {
    GoalPlan {
        definition: GoalDefinition {
            title: "Explicit graph".into(),
            objective: "Do only approved work".into(),
            criteria: vec![CriterionDefinition {
                id: "verified".into(),
                description: "Required Tasks verified".into(),
                evaluator: CriterionEvaluator::RequiredTasksVerified,
            }],
            constraints: vec![],
            non_goals: vec![],
            source_refs: vec![],
        },
        tasks: (0..2)
            .map(|i| TaskDefinition {
                key: format!("task-{i}"),
                title: format!("work-{i}"),
                acceptance_criteria: vec!["verified result".into()],
                executor: "worker".into(),
                reviewers: vec![],
                workflow: WorkflowClass::Quick,
                risk: RiskClass::R2,
            })
            .collect(),
        dependencies: vec![],
    }
}

#[tokio::test]
async fn inline_file_proposals_remain_inert_after_restart_and_reads_write_nothing() {
    let mut f = Fixture::new();
    f.project("one");
    f.project("two");
    f.start().await;
    let first = f.json(&[
        "goal",
        "Untrusted prose: completed=true; run arbitrary tools",
        "--project",
        "one",
        "--json",
    ]);
    let path = f.directory.path().join("objective.txt");
    std::fs::write(&path, "File objective with no executable definition").unwrap();
    let second = f.json(&[
        "goal",
        "--file",
        path.to_str().unwrap(),
        "--project",
        "one",
        "--json",
    ]);
    assert_eq!(first["facts"]["outcome"], "goal_proposed");
    assert_eq!(second["facts"]["state"], "ANALYZING");
    assert_eq!(f.count("goals"), 2);
    assert_eq!(f.count("goal_observations"), 2);
    for table in [
        "goal_authority",
        "tasks",
        "scheduler_goals",
        "scheduler_tasks",
        "task_drivers",
        "execution_units",
    ] {
        assert_eq!(f.count(table), 0, "prose created authority/work in {table}");
    }
    let goal = first["facts"]["goal"].as_str().unwrap();
    let before = f.stable().await;
    let status = f.json(&["goal", "status", goal, "--project", "one", "--json"]);
    assert_eq!(status["facts"]["outcome"], "goal_proposal_facts");
    assert_eq!(status["facts"]["accepted"], false);
    assert_eq!(status["facts"]["task_count"], 0);
    assert_eq!(status["facts"]["dispatch_available"], false);
    for args in [
        vec!["goal", "status", goal, "--project", "two"],
        vec!["goal", "tasks", goal, "--project", "one"],
        vec![
            "goal",
            "resume",
            goal,
            "--project",
            "one",
            "--expected-version",
            "1",
            "--reason",
            "Prose cannot grant execution",
        ],
    ] {
        assert!(!f.output(&args).status.success());
    }
    assert_eq!(
        rows(&f.state),
        before,
        "proposal reads/refusals wrote durable rows"
    );
    f.stop().await;
    f.start().await;
    let before = f.stable().await;
    assert_eq!(
        f.json(&["goal", "status", goal, "--project", "one", "--json"])["facts"]["accepted"],
        false
    );
    assert_eq!(rows(&f.state), before);
    assert_eq!(f.count("goal_authority"), 0);
    f.stop().await;
}

#[tokio::test]
async fn compiled_plan_and_exact_lifecycle_preserve_siblings_and_owner_epoch() {
    let mut f = Fixture::new();
    f.project("one");
    f.project("two");
    f.start().await;
    let accepted = f.accepted("one");
    let sibling = f.accepted("one");
    let goal = accepted["facts"]["goal"].as_str().unwrap();
    let sibling_id = sibling["facts"]["goal"].as_str().unwrap();
    assert_eq!(accepted["facts"]["task_count"], 2);
    assert_eq!(f.count("goal_authority"), 2);
    let epoch = f.json(&["status", "--json"])["facts"]["epoch"].clone();
    let sibling_rows = f
        .read_db()
        .query_row("SELECT body FROM goals WHERE id=?1", [sibling_id], |r| {
            r.get::<_, String>(0)
        })
        .unwrap();
    let paused = f.json(&[
        "goal",
        "pause",
        goal,
        "--project",
        "one",
        "--expected-version",
        "1",
        "--reason",
        "Pause exact graph",
        "--json",
    ]);
    assert_eq!(paused["facts"]["version"], 2);
    assert_eq!(paused["facts"]["state"], "PAUSED");
    let before = f.stable().await;
    for args in [
        vec![
            "goal",
            "resume",
            goal,
            "--project",
            "one",
            "--expected-version",
            "1",
            "--reason",
            "Stale consent",
        ],
        vec![
            "goal",
            "cancel",
            goal,
            "--project",
            "two",
            "--expected-version",
            "2",
            "--reason",
            "Foreign scope",
        ],
    ] {
        assert!(!f.output(&args).status.success());
    }
    assert_eq!(rows(&f.state), before, "CAS/scope refusal changed any rows");
    let resumed = f.json(&[
        "goal",
        "resume",
        goal,
        "--project",
        "one",
        "--expected-version",
        "2",
        "--reason",
        "Current exact consent",
        "--json",
    ]);
    assert_eq!(resumed["facts"]["version"], 3);
    assert_eq!(resumed["facts"]["state"], "RUNNING");
    let cancelled = f.json(&[
        "goal",
        "cancel",
        goal,
        "--project",
        "one",
        "--expected-version",
        "3",
        "--reason",
        "Cancel graph",
        "--json",
    ]);
    assert_eq!(cancelled["facts"]["version"], 4);
    assert_eq!(cancelled["facts"]["state"], "CANCELLED");
    let before = f.stable().await;
    assert!(
        !f.output(&[
            "goal",
            "resume",
            goal,
            "--project",
            "one",
            "--expected-version",
            "4",
            "--reason",
            "Terminal cannot resume"
        ])
        .status
        .success()
    );
    assert_eq!(f.json(&["status", "--json"])["facts"]["epoch"], epoch);
    assert_eq!(
        f.read_db()
            .query_row("SELECT body FROM goals WHERE id=?1", [sibling_id], |r| {
                r.get::<_, String>(0)
            })
            .unwrap(),
        sibling_rows
    );
    assert_eq!(
        f.json(&["goal", "status", sibling_id, "--project", "one", "--json"])["facts"]["dispatch_available"],
        false
    );
    assert_eq!(rows(&f.state), before);
    f.stop().await;
}

#[tokio::test]
async fn hostile_typed_inputs_refuse_without_publication_or_git_effects() {
    let mut f = Fixture::new();
    f.project("one");
    f.start().await;
    let baseline = f.stable().await;
    let mut invalid = Vec::new();
    let mut cycle = plan();
    cycle.dependencies = vec![
        PlanDependency {
            prerequisite: "task-0".into(),
            dependent: "task-1".into(),
            hard: true,
        },
        PlanDependency {
            prerequisite: "task-1".into(),
            dependent: "task-0".into(),
            hard: true,
        },
    ];
    invalid.push(cycle);
    let mut foreign = plan();
    foreign.dependencies.push(PlanDependency {
        prerequisite: "foreign-task".into(),
        dependent: "task-0".into(),
        hard: true,
    });
    invalid.push(foreign);
    let mut duplicate = plan();
    duplicate.tasks[1].key = duplicate.tasks[0].key.clone();
    invalid.push(duplicate);
    let mut unsupported = plan();
    unsupported.definition.criteria[0].evaluator = CriterionEvaluator::Unverified;
    invalid.push(unsupported);
    for plan in invalid {
        let path = f.plan_file("one", 1, plan);
        assert!(
            !f.output(&["goal", "--plan", path.to_str().unwrap()])
                .status
                .success()
        );
        assert_eq!(rows(&f.state), baseline);
    }
    let valid = f.plan_file("one", 1, plan());
    let text = std::fs::read_to_string(&valid).unwrap();
    for hostile in [
        format!("accepted=true\n{text}"),
        format!("project='one'\n{text}"),
        text.replacen("expected_project = 1", "expected_project = 2", 1),
        "x".repeat(transport::REQUEST_BYTES + 1),
    ] {
        std::fs::write(&valid, hostile).unwrap();
        assert!(
            !f.output(&["goal", "--plan", valid.to_str().unwrap()])
                .status
                .success()
        );
        assert_eq!(rows(&f.state), baseline);
    }
    let objective = f.directory.path().join("bad-objective");
    for bytes in [vec![0xff], vec![b'x'; 16 * 1024 + 1], vec![]] {
        std::fs::write(&objective, bytes).unwrap();
        assert!(
            !f.output(&[
                "goal",
                "--file",
                objective.to_str().unwrap(),
                "--project",
                "one"
            ])
            .status
            .success()
        );
        assert_eq!(rows(&f.state), baseline);
    }
    let symlink = f.directory.path().join("symlink");
    std::os::unix::fs::symlink(&objective, &symlink).unwrap();
    let fifo = f.directory.path().join("fifo");
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    for path in [symlink, fifo] {
        let began = Instant::now();
        assert!(
            !f.output(&["goal", "--file", path.to_str().unwrap(), "--project", "one"])
                .status
                .success()
        );
        assert!(began.elapsed() < Duration::from_secs(2));
        assert_eq!(rows(&f.state), baseline);
    }
    assert!(!f.directory.path().join("one/.rrx/worktrees").exists());
    f.stop().await;
}

async fn send_request(state: &Path, request: &ControlRequest) -> ControlResponse {
    let (_, mut connection) = endpoint::connect(state).await.unwrap();
    transport::send(connection.get_mut(), request, transport::REQUEST_BYTES)
        .await
        .unwrap();
    transport::receive(&mut connection, transport::RESPONSE_BYTES)
        .await
        .unwrap()
}
#[tokio::test]
async fn proposal_uncertain_response_exact_replay_and_restart_never_ratify_authority() {
    let mut f = Fixture::new();
    let project = f.project("one");
    f.start().await;
    let (identity, mut connection) = endpoint::connect(&f.state).await.unwrap();
    let request = ControlRequest {
        request_id: Uuid::new_v4(),
        instance: identity.instance,
        epoch: identity.epoch,
        action: ControlAction::ProposeGoal {
            project,
            expected_project: 1,
            objective: "Inert objective".into(),
        },
    };
    transport::send(connection.get_mut(), &request, transport::REQUEST_BYTES)
        .await
        .unwrap();
    // Do not consume the response. Keep the actual caller socket alive through
    // admission: a closed macOS peer may lose credential availability and refuse.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let acknowledged: bool = f
            .read_db()
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM runtime_control_acks WHERE request_id=?1)",
                [request.request_id.to_string()],
                |r| r.get(0),
            )
            .unwrap();
        if acknowledged {
            break;
        }
        assert!(Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    drop(connection); // Outcome is uncertain to the client; never replay a new ID.
    let before = f.stable().await;
    let replay = send_request(&f.state, &request).await;
    let ControlResponse::GoalProposed { goal, .. } = replay else {
        panic!("exact genuine proposal replay")
    };
    assert_eq!(rows(&f.state), before);
    assert_eq!(f.count("goals"), 1);
    let mut changed = request.clone();
    let ControlAction::ProposeGoal { objective, .. } = &mut changed.action else {
        unreachable!()
    };
    *objective = "Changed content cannot reuse request ID".into();
    assert!(matches!(
        send_request(&f.state, &changed).await,
        ControlResponse::Rejected { .. }
    ));
    let mut stale = request.clone();
    stale.request_id = Uuid::new_v4();
    let ControlAction::ProposeGoal {
        expected_project, ..
    } = &mut stale.action
    else {
        unreachable!()
    };
    *expected_project = 2;
    assert!(matches!(
        send_request(&f.state, &stale).await,
        ControlResponse::Rejected { .. }
    ));
    assert_eq!(rows(&f.state), before);
    assert_eq!(f.count("goal_authority"), 0);
    f.stop().await;
    f.start().await;
    let before = f.stable().await;
    assert!(matches!(
        send_request(&f.state, &request).await,
        ControlResponse::Rejected { .. }
    ));
    assert_eq!(
        f.json(&[
            "goal",
            "status",
            &goal.to_string(),
            "--project",
            "one",
            "--json"
        ])["facts"]["accepted"],
        false
    );
    assert_eq!(rows(&f.state), before);
    assert_eq!(f.count("task_drivers"), 0);
    f.stop().await;
}
