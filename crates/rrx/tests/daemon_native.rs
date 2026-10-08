//! #81 S1 installed-lane controls under the detached service: independence
//! from the invoker (C-S1a) and Native success/refusal equal to `serve`
//! (C-S1f). Only the external CLI is the local account-free protocol
//! fixture; its release and ready markers are moved to fixed test paths.
use rrx::{
    cli::client,
    config::{AgentConfig, Config, NativeCompatConfig, WorkflowClass},
    domain::{CriterionEvaluator, ProjectId, RiskClass},
    runtime::{
        control::{ControlAction, ControlResponse},
        goal::{CriterionDefinition, GoalDefinition, GoalPlan, TaskDefinition},
    },
};
use serde_json::Value;
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

const LINKS: &str = "SELECT kind FROM audit WHERE kind LIKE 'rrx.private.workflow.%' AND task_id=?1 ORDER BY sequence";
const SUCCESS: [&str; 4] = [
    "session_bound",
    "gate_claim",
    "gate_observed",
    "phase_closed",
];

struct Lane {
    _dir: tempfile::TempDir,
    base: PathBuf,
    state: PathBuf,
    config: PathBuf,
    gate: PathBuf,
    ready: PathBuf,
    serve: Option<Child>,
}
impl Lane {
    fn new(cli_version: &str) -> Self {
        let dir = tempfile::Builder::new()
            .prefix("rrx-n-")
            .tempdir_in("/tmp")
            .unwrap();
        let base = dir.path().canonicalize().unwrap();
        let gate = base.join("gate");
        let ready = base.join("ready.json");
        let source = include_str!("../src/execution/native/native_fixture.py");
        let release = r#"release = os.path.join(os.environ["RRX_OUTPUT_DIR"], "fixture-release")"#;
        let announce = "        ready.write(\"ready\\n\")\n";
        assert!(
            source.contains(release) && source.contains(announce),
            "SETUP"
        );
        let source = source.replacen(release, &format!("release = {:?}", gate.display().to_string()), 1).replacen(
            announce,
            &format!(
                "{announce}    with open({tmp:?}, \"w\") as marker: marker.write(json.dumps({{\"pid\": os.getpid()}}))\n    os.rename({tmp:?}, {ready:?})\n",
                tmp = format!("{}.tmp", ready.display()),
                ready = ready.display().to_string(),
            ),
            1,
        );
        let fixture = base.join("protocol-fixture");
        std::fs::write(
            &fixture,
            format!("#!/usr/bin/python3\nPROVIDER=\"claude\"\n{source}"),
        )
        .unwrap();
        std::fs::set_permissions(&fixture, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut config = Config {
            minimum_workflow: WorkflowClass::Quick,
            ..Default::default()
        };
        config.workflow.risk_mapping = [WorkflowClass::Quick; 4];
        let agent = AgentConfig {
            provider: Some("claude".into()),
            command: vec![fixture.to_string_lossy().into()],
            compatibility: Some(NativeCompatConfig {
                profile: "rrx-native-inherited-v1".into(),
                cli_version: cli_version.into(),
                settings: "inherited".into(),
                user_hooks: vec![],
            }),
            ..Default::default()
        };
        for alias in ["worker", "rev-a", "rev-b"] {
            config.agents.insert(alias.into(), agent.clone());
        }
        let config_path = base.join("config.toml");
        std::fs::write(&config_path, toml::to_string(&config).unwrap()).unwrap();
        let lane = Self {
            state: base.join("state.db"),
            config: config_path,
            base,
            gate,
            ready,
            serve: None,
            _dir: dir,
        };
        lane.register();
        lane
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_rrx"));
        command
            .arg("--state")
            .arg(&self.state)
            .arg("--config")
            .arg(&self.config)
            .current_dir(&self.base);
        command
    }
    fn register(&self) {
        let root = self.base.join("source");
        std::fs::create_dir(&root).unwrap();
        for args in [
            vec!["init", "-b", "main", "--quiet"],
            vec![
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--allow-empty",
                "--quiet",
                "-m",
                "fixture",
            ],
        ] {
            assert!(
                Command::new("git")
                    .current_dir(&root)
                    .args(args)
                    .status()
                    .unwrap()
                    .success()
            );
        }
        let out = self
            .command()
            .args(["project", "add"])
            .arg(&root)
            .args(["--name", "lane"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    fn daemon(&self, verb: &str) -> Value {
        let out = self
            .command()
            .args(["daemon", verb, "--json"])
            .output()
            .unwrap();
        serde_json::from_slice(&out.stdout).unwrap_or_else(|_| {
            panic!(
                "daemon {verb}: {}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )
        })
    }
    async fn run_serve(&mut self) {
        self.serve = Some(
            self.command()
                .arg("serve")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(30);
        while !matches!(
            client::request(&self.state, ControlAction::RuntimeStatus).await,
            Ok(ControlResponse::RuntimeMetadata { .. })
        ) {
            assert!(Instant::now() < deadline, "SETUP: serve not ready");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
    fn run_daemon(&self) -> Value {
        let started = self.daemon("start");
        assert_eq!(started["start"], "running", "{started}");
        started["identity"].clone()
    }
    async fn create(&self) -> String {
        let ControlResponse::ProjectResolved {
            project, version, ..
        } = client::request(
            &self.state,
            ControlAction::ResolveProject {
                selector: Some("lane".into()),
                cwd: self.base.clone(),
            },
        )
        .await
        .unwrap()
        else {
            panic!("SETUP: Project routing");
        };
        let project: ProjectId = project;
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
            tasks: vec![TaskDefinition {
                key: "task-0".into(),
                title: "work-0".into(),
                acceptance_criteria: vec!["verified result".into()],
                executor: "worker".into(),
                reviewers: vec!["rev-a".into(), "rev-b".into()],
                workflow: WorkflowClass::Quick,
                risk: RiskClass::R1,
            }],
            dependencies: vec![],
        };
        let accepted = client::request(
            &self.state,
            ControlAction::CreateGoal {
                project,
                expected_project: version,
                plan,
            },
        )
        .await
        .unwrap();
        assert!(
            matches!(
                accepted,
                ControlResponse::GoalAccepted { task_count: 1, .. }
            ),
            "{accepted:?}"
        );
        self.read(|c| c.query_row("SELECT id FROM tasks", [], |r| r.get(0)))
    }
    fn read<T>(&self, query: impl FnOnce(&rusqlite::Connection) -> rusqlite::Result<T>) -> T {
        let connection = rusqlite::Connection::open_with_flags(
            &self.state,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        connection.busy_timeout(Duration::from_secs(5)).unwrap();
        query(&connection).unwrap()
    }
    fn links(&self, task: &str) -> Vec<String> {
        self.read(|c| {
            c.prepare(LINKS)?
                .query_map([task], |r| r.get::<_, String>(0))?
                .map(|k| k.map(|k| k.trim_start_matches("rrx.private.workflow.").to_owned()))
                .collect()
        })
    }
    fn count(&self, table: &str) -> i64 {
        self.read(|c| c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0)))
    }
    async fn wait(&self, mut condition: impl FnMut() -> bool, label: &str) {
        let deadline = Instant::now() + Duration::from_secs(90);
        while !condition() {
            assert!(Instant::now() < deadline, "{label}");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
    /// The success chain through the gate release, as observed durably.
    async fn success_links(&self, task: &str) -> Vec<String> {
        self.wait(
            || self.ready.exists(),
            "SETUP: implement never reached the peer",
        )
        .await;
        std::fs::write(&self.gate, "release").unwrap();
        self.wait(
            || self.links(task).last().map(String::as_str) == Some("phase_closed"),
            "success phase_closed absent",
        )
        .await;
        self.links(task)
    }
    /// The durable projection of a refused preparation once it has settled.
    async fn refusal(&self, task: &str) -> (Vec<String>, i64, i64, String) {
        self.wait(
            || {
                self.read(|c| {
                    c.query_row(
                        "SELECT count(*) FROM managed_phase_readiness WHERE state='preparing'",
                        [],
                        |r| r.get::<_, i64>(0),
                    )
                }) > 0
            },
            "SETUP: preparation never began",
        )
        .await;
        tokio::time::sleep(Duration::from_secs(3)).await;
        (
            self.links(task),
            self.count("native_invocations"),
            self.read(|c| {
                c.query_row(
                    "SELECT count(*) FROM records WHERE kind='session'",
                    [],
                    |r| r.get(0),
                )
            }),
            self.read(|c| {
                c.query_row(
                    "SELECT group_concat(state) FROM managed_phase_readiness",
                    [],
                    |r| r.get(0),
                )
            }),
        )
    }
    /// One RuntimeStop. A later phase may hold it pending; the service then
    /// exits unsuccessfully on its own and releases the owner.
    async fn stop(&mut self) {
        let response = client::request(&self.state, ControlAction::RuntimeStop)
            .await
            .unwrap();
        assert!(
            matches!(
                response,
                ControlResponse::RuntimeStopped { .. } | ControlResponse::RuntimeStopPending { .. }
            ),
            "{response:?}"
        );
        if let Some(mut child) = self.serve.take() {
            child.wait().unwrap();
        }
        self.wait(
            || self.daemon("status")["status"] == "not_running",
            "owner not released",
        )
        .await;
    }
}
impl Drop for Lane {
    fn drop(&mut self) {
        if let Some(mut child) = self.serve.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = self.command().args(["daemon", "stop"]).output();
    }
}

/// C-S1f: success links and a compatibility refusal are identical under
/// `serve` and under `daemon start`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn c_s1f_native_success_and_refusal_equal_under_serve_and_daemon() {
    let mut outcomes = Vec::new();
    for detached in [false, true] {
        let mut lane = Lane::new("2.1.294");
        if detached {
            lane.run_daemon();
        } else {
            lane.run_serve().await;
        }
        let task = lane.create().await;
        let links = lane.success_links(&task).await;
        assert_eq!(links, SUCCESS, "detached={detached}");
        lane.stop().await;

        let mut refused = Lane::new("2.1.283");
        if detached {
            refused.run_daemon();
        } else {
            refused.run_serve().await;
        }
        let task = refused.create().await;
        let refusal = refused.refusal(&task).await;
        assert!(refusal.0.is_empty(), "refused lane linked: {refusal:?}");
        assert_eq!(refusal.1, 0, "refused lane invoked Native");
        refused.stop().await;
        outcomes.push((links, refusal));
    }
    assert_eq!(outcomes[0], outcomes[1], "serve and daemon differ");
}

/// The detached service of a lane, found only to inspect it.
fn service_pid(state: &Path) -> i32 {
    let output = Command::new("pgrep")
        .arg("-f")
        .arg(format!("{} --config .* serve --detached", state.display()))
        .output()
        .unwrap();
    let pids: Vec<i32> = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .map(|p| p.parse().unwrap())
        .collect();
    assert_eq!(pids.len(), 1, "SETUP: one detached service: {pids:?}");
    pids[0]
}
fn parent_of(pid: i32) -> Option<i32> {
    let output = Command::new("ps")
        .args(["-o", "ppid=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}
fn sid(pid: i32) -> i32 {
    rustix::process::getsid(rustix::process::Pid::from_raw(pid))
        .unwrap()
        .as_raw_nonzero()
        .get()
}

/// The invoker: a pty session running `daemon start`, then idling. On
/// "kill" it SIGKILLs the invoker's process group and closes the terminal.
const INVOKER: &str = r#"
import os, pty, select, signal, sys
command = sys.argv[1:]
pid, fd = pty.fork()
if pid == 0:
    os.execv("/bin/sh", ["/bin/sh", "-c", '"$@" > invoker-start.json 2>&1; touch invoker-started; exec sleep 600', "sh"] + command)
while not os.path.exists("invoker-started"):
    if select.select([fd], [], [], 0.05)[0]:
        try:
            os.read(fd, 4096)
        except OSError:
            pass
print(pid, os.getsid(pid), flush=True)
sys.stdin.readline()
os.killpg(pid, signal.SIGKILL)
os.close(fd)
os.waitpid(pid, 0)
print("closed", flush=True)
"#;

/// C-S1a: start from a pty invoker while a Task is in flight; SIGKILL the
/// invoker's process group and close its terminal. The same epoch keeps
/// running, its session differs from the invoker's, the in-flight native
/// child belongs to the daemon, and the Task's phase closes normally.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn c_s1a_daemon_survives_invoker_group_and_terminal() {
    let mut lane = Lane::new("2.1.294");
    let mut invoker = Command::new("/usr/bin/python3")
        .arg("-c")
        .arg(INVOKER)
        .arg(env!("CARGO_BIN_EXE_rrx"))
        .arg("--state")
        .arg(&lane.state)
        .arg("--config")
        .arg(&lane.config)
        .args(["daemon", "start", "--json"])
        .current_dir(&lane.base)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(invoker.stdout.take().unwrap()).lines();
    let header = lines.next().unwrap().unwrap();
    let mut header = header.split_whitespace().map(|v| v.parse::<i32>().unwrap());
    let (invoker_pid, invoker_sid) = (header.next().unwrap(), header.next().unwrap());
    let started: Value =
        serde_json::from_slice(&std::fs::read(lane.base.join("invoker-start.json")).unwrap())
            .unwrap();
    assert_eq!(started["start"], "running", "{started}");
    let identity = started["identity"].clone();
    let task = lane.create().await;
    lane.wait(|| lane.ready.exists(), "SETUP: Task not in flight")
        .await;
    let daemon = service_pid(&lane.state);
    let marker: Value = serde_json::from_slice(&std::fs::read(&lane.ready).unwrap()).unwrap();
    let native = i32::try_from(marker["pid"].as_i64().unwrap()).unwrap();
    // The in-flight native child descends from the daemon, not the invoker.
    let mut ancestor = Some(native);
    let mut owned = false;
    while let Some(pid) = ancestor.filter(|p| *p > 1) {
        if pid == daemon {
            owned = true;
            break;
        }
        assert_ne!(pid, invoker_pid, "native child owned by the invoker");
        ancestor = parent_of(pid);
    }
    assert!(
        owned,
        "native child {native} is not owned by daemon {daemon}"
    );
    assert_eq!(sid(daemon), daemon, "daemon is not its own session leader");
    assert_ne!(
        sid(daemon),
        invoker_sid,
        "daemon shares the invoker session"
    );

    let mut stdin = invoker.stdin.take().unwrap();
    writeln!(stdin, "kill").unwrap();
    assert_eq!(lines.next().unwrap().unwrap(), "closed");
    assert!(invoker.wait().unwrap().success());

    let status = lane.daemon("status");
    assert_eq!(status["status"], "running", "{status}");
    assert_eq!(status["identity"], identity, "the epoch changed");
    assert_eq!(service_pid(&lane.state), daemon);
    let links = lane.success_links(&task).await;
    assert_eq!(links, SUCCESS, "the in-flight phase did not close normally");
    assert_eq!(lane.daemon("status")["identity"], identity);
    lane.stop().await;
}
