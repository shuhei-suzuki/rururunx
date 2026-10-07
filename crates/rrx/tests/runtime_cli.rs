use rrx::{
    cli::{client, endpoint, transport},
    config::WorkflowClass,
    domain::{CriterionEvaluator, GoalId, ProjectId, RiskClass},
    runtime::{
        control::{ControlAction, ControlRequest, ControlResponse},
        goal::{CriterionDefinition, GoalDefinition, GoalPlan, TaskDefinition},
    },
};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

struct Fixture {
    dir: tempfile::TempDir,
    state: PathBuf,
    config: PathBuf,
    child: Option<Child>,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("rrx-cli-")
            .tempdir_in("/tmp")
            .unwrap();
        let state = dir.path().join("state.db");
        let config = dir.path().join("config.toml");
        std::fs::write(
            &config,
            "[agents.worker]\nprovider='claude'\ncommand=['/usr/bin/false']\n",
        )
        .unwrap();
        Self {
            dir,
            state,
            config,
            child: None,
        }
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rrx"));
        command
            .args(["--state"])
            .arg(&self.state)
            .arg("--config")
            .arg(&self.config)
            .current_dir(self.dir.path());
        command
    }
    fn project(&self, name: &str) -> ProjectId {
        let root = self.dir.path().join(name);
        std::fs::create_dir(&root).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "--initial-branch=main", "--quiet"])
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
        let out = self
            .command()
            .args(["project", "add"])
            .arg(root)
            .args(["--name", name])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout)
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
        let until = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(
                self.child.as_mut().unwrap().try_wait().unwrap().is_none(),
                "compiled serve exited during startup"
            );
            if client::request(&self.state, ControlAction::RuntimeStatus)
                .await
                .is_ok()
            {
                break;
            }
            assert!(Instant::now() < until, "compiled serve startup deadline");
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
        let until = Instant::now() + Duration::from_secs(15);
        loop {
            if let Some(status) = self.child.as_mut().unwrap().try_wait().unwrap() {
                assert!(status.success());
                self.child.take();
                break;
            }
            assert!(
                Instant::now() < until,
                "compiled service failed to join shutdown"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(!descriptor(&self.state).exists());
    }
    fn cli_json(&self, args: &[&str]) -> Value {
        let out = self.command().args(args).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).unwrap()
    }
    async fn create(&self, project: ProjectId, tasks: usize) -> GoalId {
        let resolved = client::request(
            &self.state,
            ControlAction::ResolveProject {
                selector: Some(project.to_string()),
                cwd: self.dir.path().into(),
            },
        )
        .await
        .unwrap();
        let ControlResponse::ProjectResolved { version, .. } = resolved else {
            panic!("real Project routing");
        };
        let plan = GoalPlan {
            definition: GoalDefinition {
                title: "Goal".into(),
                objective: "Explicit work".into(),
                criteria: vec![CriterionDefinition {
                    id: "verified".into(),
                    description: "All required work verified".into(),
                    evaluator: CriterionEvaluator::RequiredTasksVerified,
                }],
                constraints: vec![],
                non_goals: vec![],
                source_refs: vec![],
            },
            tasks: (0..tasks)
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
        };
        match client::request(
            &self.state,
            ControlAction::CreateGoal {
                project,
                expected_project: version,
                plan,
            },
        )
        .await
        .unwrap()
        {
            ControlResponse::GoalAccepted {
                goal, task_count, ..
            } => {
                assert_eq!(task_count, tasks);
                goal
            }
            other => panic!("genuine accepted Unix Goal: {other:?}"),
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
fn descriptor(state: &Path) -> PathBuf {
    state
        .parent()
        .unwrap()
        .join("state.db.execution/control/endpoint.json")
}
fn rows(state: &Path) -> Vec<(String, Vec<Vec<String>>)> {
    let db =
        rusqlite::Connection::open_with_flags(state, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    let names = db
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    names
        .into_iter()
        .map(|name| {
            let mut statement = db
                .prepare(&format!("SELECT * FROM \"{}\"", name.replace('"', "\"\"")))
                .unwrap();
            let count = statement.column_count();
            let mut data = statement
                .query_map([], |r| {
                    Ok((0..count)
                        .map(|i| format!("{:?}", r.get_ref(i).unwrap()))
                        .collect::<Vec<_>>())
                })
                .unwrap()
                .map(Result::unwrap)
                .collect::<Vec<_>>();
            data.sort();
            (name, data)
        })
        .collect()
}

#[tokio::test]
async fn compiled_service_goal_pages_routing_and_reads_preserve_all_rows() {
    let mut f = Fixture::new();
    let project = f.project("one");
    let foreign = f.project("two");
    f.start().await;
    let goal = f.create(project, 65).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let before = rows(&f.state);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        rows(&f.state),
        before,
        "idle reconciliation baseline is not stable"
    );
    let metadata = f.cli_json(&["status", "--all", "--json"]);
    assert_eq!(metadata["facts"]["operational"], false);
    assert_eq!(metadata["complete"], false);
    assert!(
        metadata["unavailable_fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "effective_native_limits")
    );
    let goal_text = goal.to_string();
    let status = f.cli_json(&["goal", "status", &goal_text, "--project", "one", "--json"]);
    assert_eq!(status["facts"]["task_count"], 65);
    assert_eq!(status["facts"]["dispatch_available"], false);
    assert_eq!(status["facts"]["attention"], "native_binding_unavailable");
    let first = f.cli_json(&["goal", "tasks", &goal_text, "--project", "one", "--json"]);
    assert_eq!(first["facts"]["tasks"].as_array().unwrap().len(), 64);
    assert_eq!(first["complete"], false);
    let cursor = first["facts"]["next"].as_str().unwrap();
    let second = f.cli_json(&[
        "goal",
        "tasks",
        &goal_text,
        "--project",
        &project.to_string(),
        "--after",
        cursor,
        "--json",
    ]);
    assert_eq!(second["facts"]["tasks"].as_array().unwrap().len(), 1);
    assert!(second["facts"]["next"].is_null());
    assert_eq!(
        second["complete"], false,
        "last page is still an independent observation"
    );
    for args in [
        vec!["goal", "status", &goal_text, "--project", "two"],
        vec![
            "goal",
            "tasks",
            &goal_text,
            "--project",
            "one",
            "--after",
            "ffffffff-ffff-ffff-ffff-ffffffffffff",
        ],
    ] {
        assert!(!f.command().args(args).output().unwrap().status.success());
    }
    assert!(
        client::request(
            &f.state,
            ControlAction::GoalStatus {
                view: None,
                project: foreign,
                goal
            }
        )
        .await
        .is_err()
    );
    assert_eq!(
        rows(&f.state),
        before,
        "successful/refused compiled reads changed durable rows"
    );
    f.stop().await;
}

#[tokio::test]
async fn missing_service_reads_create_nothing_and_preserve_stale_descriptor() {
    let mut f = Fixture::new();
    for args in [
        vec!["status", "--all"],
        vec!["goal", "status", "ffffffff-ffff-ffff-ffff-ffffffffffff"],
    ] {
        let output = f.command().args(args).output().unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("rrx serve"));
        assert!(!f.state.exists());
        assert!(!f.state.with_extension("db.execution").exists());
    }
    f.start().await;
    let path = descriptor(&f.state);
    let bytes = std::fs::read(&path).unwrap();
    f.stop().await;
    std::fs::write(&path, &bytes).unwrap();
    let before = rows(&f.state);
    let output = f.command().args(["status", "--json"]).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("rrx serve"));
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(rows(&f.state), before);
}

#[tokio::test]
async fn compiled_server_refuses_raw_frames_and_old_identity_then_keeps_serving() {
    let mut f = Fixture::new();
    f.start().await;
    let before = rows(&f.state);
    let (identity, connection) = endpoint::connect(&f.state).await.unwrap();
    drop(connection);
    let nominal = serde_json::to_string(&ControlRequest {
        request_id: Uuid::new_v4(),
        instance: identity.instance.clone(),
        epoch: identity.epoch,
        action: ControlAction::RuntimeStatus,
    })
    .unwrap();
    // Each nominal frame is otherwise accepted: one rejection condition is added.
    let object = nominal.strip_suffix('}').unwrap();
    for bytes in [
        format!("{object},\"epoch\":{}}}\n", identity.epoch).into_bytes(),
        format!("{object},\"principal\":\"administrator\"}}\n").into_bytes(),
        format!("{nominal} {{}}\n").into_bytes(),
        vec![0xff, b'\n'],
    ] {
        let (_, mut connection) = endpoint::connect(&f.state).await.unwrap();
        connection.get_mut().write_all(&bytes).await.unwrap();
        assert!(matches!(
            transport::receive::<ControlResponse>(&mut connection, transport::RESPONSE_BYTES)
                .await
                .unwrap(),
            ControlResponse::Rejected { request_id: None }
        ));
    }
    let (identity, mut connection) = endpoint::connect(&f.state).await.unwrap();
    transport::send(
        connection.get_mut(),
        &ControlRequest {
            request_id: Uuid::new_v4(),
            instance: identity.instance,
            epoch: identity.epoch + 1,
            action: ControlAction::RuntimeStop,
        },
        transport::REQUEST_BYTES,
    )
    .await
    .unwrap();
    assert!(matches!(
        transport::receive::<ControlResponse>(&mut connection, transport::RESPONSE_BYTES)
            .await
            .unwrap(),
        ControlResponse::Rejected { .. }
    ));
    let live = f.cli_json(&["status", "--json"]);
    assert_eq!(live["facts"]["epoch"], identity.epoch);
    assert_eq!(rows(&f.state), before, "refused frames published state");
    let race = f.command().arg("serve").output().unwrap();
    assert!(!race.status.success());
    assert_eq!(rows(&f.state), before, "second owner advanced epoch");
    let old = std::fs::read(descriptor(&f.state)).unwrap();
    f.stop().await;
    f.start().await;
    assert_ne!(std::fs::read(descriptor(&f.state)).unwrap(), old);
    assert_eq!(
        f.cli_json(&["status", "--json"])["facts"]["epoch"],
        identity.epoch + 1
    );
    f.stop().await;
}

#[tokio::test]
async fn actual_service_loop_failure_is_refused_and_exits_unsuccessfully() {
    let mut f = Fixture::new();
    f.start().await;
    // Destructive fault in this isolated fixture only. This installs no Goal,
    // driver, native capability or acceptance evidence.
    rusqlite::Connection::open(&f.state)
        .unwrap()
        .execute_batch("DROP TABLE scheduler_tasks")
        .unwrap();
    // The idle controller deliberately sleeps up to five seconds. Observe its
    // fault separately from the subsequent service shutdown budget.
    let until = Instant::now() + Duration::from_secs(8);
    loop {
        if matches!(
            client::request(&f.state, ControlAction::RuntimeStatus)
                .await
                .unwrap(),
            ControlResponse::RuntimeMetadata {
                service_running: false,
                ..
            }
        ) {
            break;
        }
        assert!(
            Instant::now() < until,
            "faulted service loop remained running"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(
        client::request(&f.state, ControlAction::RuntimeStop)
            .await
            .is_err()
    );
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = f.child.as_mut().unwrap().try_wait().unwrap() {
            assert!(
                !status.success(),
                "initial shutdown error was erased by repeated shutdown"
            );
            f.child.take();
            break;
        }
        assert!(
            Instant::now() < until,
            "failed service shutdown exceeded deadline"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(!descriptor(&f.state).exists());
}

#[tokio::test]
async fn actual_connection_limit_and_idle_shutdown_are_bounded() {
    let mut f = Fixture::new();
    f.start().await;
    let mut held = Vec::new();
    for _ in 0..rrx::cli::service::MAX_CONNECTIONS {
        held.push(endpoint::connect(&f.state).await.unwrap());
    }
    assert!(
        tokio::time::timeout(Duration::from_secs(2), endpoint::connect(&f.state))
            .await
            .unwrap()
            .is_err(),
        "overloaded client received an admitted Hello"
    );
    held.pop();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let began = Instant::now();
    f.stop().await;
    assert!(
        began.elapsed() < Duration::from_secs(12),
        "idle socket waits prevented bounded shutdown"
    );
    drop(held);
}
