//! FM §8.1 L harness for the integration crate, through the public route only:
//! `cli::service::serve` on the test's own state database,
//! `cli::client::request` (`CreateGoal`, then `RuntimeStop`), then the ordered
//! historical migration in place. Only the 7 historical nongrant tables are
//! copied into the historical schema at `user_version = 2`; `Store::open`
//! migrates. No authority, scheduler, Driver or execution row is copied and no
//! guard or writer is changed.
use rrx::{
    config::{AgentConfig, Config, WorkflowClass},
    domain::{Goal, ProjectId, RiskClass, Task},
    runtime::{
        control::{ControlAction, ControlResponse, GoalControl},
        goal::{CriterionDefinition, GoalDefinition, GoalPlan, TaskDefinition},
    },
    state::Store,
};
use std::path::{Path, PathBuf};

/// One planned Task: plan key (also its title), executor, accepted class
/// and its acceptance criterion.
pub type Planned = (
    &'static str,
    &'static str,
    WorkflowClass,
    RiskClass,
    &'static str,
);

const COPIED: [&str; 7] = [
    "projects",
    "goals",
    "tasks",
    "records",
    "context_versions",
    "usage",
    "audit",
];

/// One Goal: its Project, objective (also the title, as `Goal::new` set
/// both) and planned Tasks.
pub type GoalSpec = (ProjectId, &'static str, Vec<Planned>);

fn config(plans: &[GoalSpec]) -> Config {
    let mut config = Config::default();
    let planned = plans.iter().flat_map(|(_, _, tasks)| tasks.iter());
    for (_, executor, _, _, _) in planned.clone() {
        // Ingress-only config: accepted plans admit claude|codex providers.
        // The copied legacy row carries only the executor name, exactly as
        // `Task::new(.., executor)` wrote it.
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
    config.minimum_workflow = planned.map(|t| t.2).min().unwrap_or_default();
    config
}

fn plan(objective: &str, tasks: &[Planned]) -> GoalPlan {
    GoalPlan {
        definition: GoalDefinition {
            title: objective.into(),
            objective: objective.into(),
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
            .map(
                |(key, executor, workflow, risk, acceptance)| TaskDefinition {
                    key: (*key).into(),
                    title: (*key).into(),
                    acceptance_criteria: vec![(*acceptance).into()],
                    executor: (*executor).into(),
                    reviewers: vec![],
                    workflow: *workflow,
                    risk: *risk,
                },
            )
            .collect(),
        dependencies: vec![],
    }
}

async fn accepted(
    db: PathBuf,
    plans: Vec<(GoalSpec, Option<GoalControl>)>,
) -> Vec<rrx::domain::GoalId> {
    let (plans, lifecycles): (Vec<_>, Vec<_>) = plans.into_iter().unzip();
    let projects = {
        let store = Store::open(&db).unwrap();
        plans
            .iter()
            .map(|(id, _, _)| store.project(*id).unwrap().unwrap())
            .collect::<Vec<_>>()
    };
    let service = {
        let (db, config) = (db.clone(), config(&plans));
        tokio::spawn(async move { rrx::cli::service::serve(&db, config, false).await })
    };
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
    // Readiness is only "the endpoint accepts and says Hello"; no control
    // action is used as a probe (refusals are legitimate for some Projects).
    loop {
        let probe = rrx::cli::endpoint::connect(&db).await;
        if probe.is_ok() {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "SETUP: Runtime service endpoint unavailable: {:?}",
            probe.err()
        );
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let mut goals = vec![];
    for ((project, (_, objective, tasks)), lifecycle) in projects.iter().zip(&plans).zip(lifecycles)
    {
        let (goal, version) = match rrx::cli::client::request(
            &db,
            ControlAction::CreateGoal {
                project: project.id,
                expected_project: project.version,
                plan: plan(objective, tasks),
            },
        )
        .await
        .unwrap()
        {
            ControlResponse::GoalAccepted { goal, version, .. } => (goal, version),
            other => panic!("SETUP: actual accepted Goal required, got {other:?}"),
        };
        // Legitimate drift (§8.2): Goal lifecycle through live ingress,
        // before the rows are migrated.
        if let Some(target) = lifecycle {
            match rrx::cli::client::request(
                &db,
                ControlAction::SetGoalLifecycle {
                    project: project.id,
                    goal,
                    expected_goal: version,
                    target,
                    reason: "fixture lifecycle".into(),
                },
            )
            .await
            .unwrap()
            {
                ControlResponse::GoalLifecycleChanged { .. } => {}
                other => panic!("SETUP: actual Goal lifecycle required, got {other:?}"),
            }
        }
        goals.push(goal);
    }
    rrx::cli::client::request(&db, ControlAction::RuntimeStop)
        .await
        .unwrap();
    service.await.unwrap().unwrap();
    goals
}

/// Creates one accepted Goal per `(Project, Tasks)` in the state database
/// `db` (its Projects already registered) through the public route, then
/// replaces `db` with the ordered historical migration of its nongrant rows.
/// Every connection to `db` must be closed by the caller. Returns each Goal
/// with its Tasks, read from the migrated database.
pub fn goals(db: &Path, plans: Vec<GoalSpec>) -> Vec<(Goal, Vec<Task>)> {
    goals_with(db, plans.into_iter().map(|plan| (plan, None)).collect())
}

/// `goals`, with an optional lifecycle control applied to each Goal through
/// the same public route before the migration.
pub fn goals_with(
    db: &Path,
    plans: Vec<(GoalSpec, Option<GoalControl>)>,
) -> Vec<(Goal, Vec<Task>)> {
    let goal_ids = {
        let db = db.to_path_buf();
        std::thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(accepted(db, plans))
        })
        .join()
        .unwrap()
    };
    {
        // The copy source never executed: no Unit, Session or Driver.
        let c = rusqlite::Connection::open(db).unwrap();
        for table in ["execution_units", "session_units", "task_drivers"] {
            let n: i64 = c
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            assert_eq!(n, 0, "SETUP: the copy source executed ({table})");
        }
    }
    let name = db.file_name().unwrap().to_string_lossy().into_owned();
    let legacy = db.with_file_name(format!("{name}.legacy-v2"));
    {
        let connection = rusqlite::Connection::open(&legacy).unwrap();
        connection
            .execute_batch(include_str!("../../src/state/schema.sql"))
            .unwrap();
        connection
            .execute(
                "ATTACH DATABASE ?1 AS current_fixture",
                [db.to_str().unwrap()],
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
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(db.with_file_name(format!("{name}{suffix}")));
    }
    let _ = std::fs::remove_dir_all(db.with_file_name(format!("{name}.execution")));
    std::fs::rename(&legacy, db).unwrap();
    let store = Store::open(db).unwrap();
    {
        let c = rusqlite::Connection::open(db).unwrap();
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
    goal_ids
        .into_iter()
        .map(|id| {
            let goal = store.goal(id).unwrap().unwrap();
            let tasks = goal
                .dag
                .nodes
                .iter()
                .map(|id| store.task(*id).unwrap().unwrap())
                .collect::<Vec<_>>();
            assert!(tasks.iter().all(|t| t.worktree.is_none()));
            (goal, tasks)
        })
        .collect()
}
