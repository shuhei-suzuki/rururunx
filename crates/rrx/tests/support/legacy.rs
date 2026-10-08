//! FM §8.1 L harness for the integration crate, through the public route only:
//! `ProjectRegistry::add`, `cli::service::serve`, `cli::client::request`
//! (`CreateGoal`, then `RuntimeStop`), then the ordered historical migration.
//! Only the historical nongrant tables are copied; `Store::open` migrates.
use rrx::{
    adapter::SharedStore,
    config::{AgentConfig, Config, WorkflowClass},
    domain::{RiskClass, Task},
    project::{AddProject, ProjectRegistry},
    runtime::{
        control::{ControlAction, ControlResponse},
        goal::{CriterionDefinition, GoalDefinition, GoalPlan, TaskDefinition},
    },
    state::Store,
};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Keeps the fixture directory alive: `<root>/` is the Project root and
/// `<state>` the migrated legacy database.
pub struct Holder {
    dir: tempfile::TempDir,
}
impl Holder {
    pub fn path(&self) -> &Path {
        self.dir.path()
    }
}

/// One planned Task: plan key, executor, accepted class.
pub type Planned = (&'static str, &'static str, WorkflowClass, RiskClass);

const COPIED: [&str; 7] = [
    "projects",
    "goals",
    "tasks",
    "records",
    "context_versions",
    "usage",
    "audit",
];

fn git(root: &Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "SETUP: git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

async fn accepted(root: &Path, ingress: &Path, tasks: &[Planned]) -> rrx::domain::GoalId {
    let mut config = Config::default();
    for (_, executor, _, _) in tasks {
        // Ingress-only config: accepted plans admit claude|codex providers.
        // The copied legacy row carries only the executor name, exactly as
        // `Task::new(.., executor)` wrote it (e.g. "grok").
        let provider = match *executor {
            "codex" => "codex",
            _ => "claude",
        };
        config.agents.insert(
            (*executor).into(),
            AgentConfig {
                provider: Some(provider.into()),
                command: vec!["/bin/true".into()],
                ..Default::default()
            },
        );
    }
    // A Quick plan needs minimum Quick (Quick = R0 + minimum Quick).
    config.minimum_workflow = tasks.iter().map(|t| t.2).min().unwrap_or_default();
    let project = {
        let mut store = Store::open(ingress).unwrap();
        ProjectRegistry::new(&mut store)
            .add(root, AddProject::default(), &config)
            .unwrap()
    };
    let state = ingress.to_path_buf();
    let service = {
        let state = state.clone();
        tokio::spawn(async move { rrx::cli::service::serve(&state, config).await })
    };
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        let probe = rrx::cli::client::request(
            &state,
            ControlAction::ResolveProject {
                selector: Some(project.id.to_string()),
                cwd: root.to_path_buf(),
            },
        )
        .await;
        if probe.is_ok() {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "SETUP: Runtime service endpoint unavailable: {probe:?}"
        );
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let plan = GoalPlan {
        definition: GoalDefinition {
            title: "t".into(),
            objective: "o".into(),
            criteria: vec![CriterionDefinition {
                id: "c".into(),
                description: "d".into(),
                evaluator: rrx::domain::CriterionEvaluator::RequiredTasksVerified,
            }],
            constraints: vec![],
            non_goals: vec![],
            source_refs: vec![],
        },
        tasks: tasks
            .iter()
            .map(|(key, executor, workflow, risk)| TaskDefinition {
                key: (*key).into(),
                title: (*key).into(),
                acceptance_criteria: vec!["verified result".into()],
                executor: (*executor).into(),
                reviewers: vec![],
                workflow: *workflow,
                risk: *risk,
            })
            .collect(),
        dependencies: vec![],
    };
    let goal = match rrx::cli::client::request(
        &state,
        ControlAction::CreateGoal {
            project: project.id,
            expected_project: project.version,
            plan,
        },
    )
    .await
    .unwrap()
    {
        ControlResponse::GoalAccepted { goal, .. } => goal,
        other => panic!("SETUP: actual accepted Goal required, got {other:?}"),
    };
    rrx::cli::client::request(&state, ControlAction::RuntimeStop)
        .await
        .unwrap();
    service.await.unwrap().unwrap();
    goal
}

fn historical_copy(source: &Path, target: &Path) {
    let connection = rusqlite::Connection::open(target).unwrap();
    connection
        .execute_batch(include_str!("../../src/state/schema.sql"))
        .unwrap();
    connection
        .execute(
            "ATTACH DATABASE ?1 AS current_fixture",
            [source.to_str().unwrap()],
        )
        .unwrap();
    for table in COPIED {
        connection
            .execute_batch(&format!(
                "INSERT INTO {table} SELECT * FROM current_fixture.{table}"
            ))
            .unwrap();
    }
    connection
        .execute_batch("DETACH DATABASE current_fixture")
        .unwrap();
    connection
        .pragma_update(None, "application_id", rrx::state::APPLICATION_ID)
        .unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
}

/// Legacy rows for `tasks` on a Git Project at `<dir>/<root>` (branch `main`,
/// one empty commit, then `seed`), migrated into `<dir>/<state>`.
pub fn blocking(
    root: &'static str,
    state: &'static str,
    seed: impl FnOnce(&Path) + Send + 'static,
    tasks: Vec<Planned>,
) -> (Holder, SharedStore, Vec<Task>) {
    let dir = tempfile::tempdir().unwrap();
    let base: PathBuf = dir.path().canonicalize().unwrap();
    let project_root = base.join(root);
    std::fs::create_dir(&project_root).unwrap();
    git(&project_root, &["init", "-b", "main"]);
    git(
        &project_root,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--allow-empty",
            "-m",
            "fixture",
        ],
    );
    seed(&project_root);
    let ingress = base.join("ingress.db");
    let goal = {
        let (root, ingress) = (project_root.clone(), ingress.clone());
        std::thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move { accepted(&root, &ingress, &tasks).await })
        })
        .join()
        .unwrap()
    };
    {
        // The copy source never executed: no Unit, Session or Driver.
        let c = rusqlite::Connection::open(&ingress).unwrap();
        for table in ["execution_units", "session_units", "task_drivers"] {
            let n: i64 = c
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            assert_eq!(n, 0, "SETUP: the copy source executed ({table})");
        }
    }
    let target = base.join(state);
    historical_copy(&ingress, &target);
    for leftover in ["ingress.db", "ingress.db-wal", "ingress.db-shm"] {
        let _ = std::fs::remove_file(base.join(leftover));
    }
    let _ = std::fs::remove_dir_all(base.join("ingress.db.execution"));
    let store = Store::open(&target).unwrap();
    {
        let c = rusqlite::Connection::open(&target).unwrap();
        let mut statement = c
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND (name IN ('goal_authority','scheduler_projects','scheduler_goals','scheduler_tasks','task_drivers','execution_units','session_units','task_execution','source_recoveries') OR name LIKE 'managed\\_%' ESCAPE '\\' OR name LIKE 'native\\_%' ESCAPE '\\')")
            .unwrap();
        let tables = statement
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        for table in tables {
            let n: i64 = c
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            assert_eq!(n, 0, "SETUP: legacy migration restored {table}");
        }
    }
    let goal = store.goal(goal).unwrap().unwrap();
    let tasks = goal
        .dag
        .nodes
        .iter()
        .map(|id| store.task(*id).unwrap().unwrap())
        .collect::<Vec<_>>();
    assert!(tasks.iter().all(|t| t.worktree.is_none()));
    (Holder { dir }, Arc::new(Mutex::new(store)), tasks)
}
