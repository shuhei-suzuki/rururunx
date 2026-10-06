use anyhow::{Result, ensure};
use clap::{Parser, Subcommand};
use rrx::{
    cli::{client, service},
    config::Config,
    domain::{GoalId, TaskId},
    project::{AddProject, ProjectRegistry, default_state_path},
    runtime::control::{ControlAction, ControlResponse},
    state::Store,
};
use std::{path::PathBuf, process::ExitCode};

#[derive(Parser)]
#[command(version, about = "Local workflow runtime for native coding agents")]
struct Cli {
    /// Explicit runtime-global TOML configuration.
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    /// TOML overlay for config-check, or a source-relative reference for project add.
    #[arg(long, global = true)]
    project_config: Option<PathBuf>,
    /// Runtime-global SQLite state (default: RRX_STATE_PATH or XDG/home state directory).
    #[arg(long, global = true)]
    state: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}
#[derive(Subcommand)]
enum Command {
    /// Serve dedicated local controls in the foreground; native dispatch is unavailable.
    Serve,
    /// Inspect current metadata (full hierarchy and native capacity remain unavailable).
    Status {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        json: bool,
    },
    /// Read scoped Goal facts from the existing control service.
    Goal {
        #[command(subcommand)]
        command: GoalCommand,
    },
    /// Validate configuration without starting sessions or creating state.
    ConfigCheck,
    /// Register and inspect isolated repository projects.
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
}
#[derive(Subcommand)]
enum GoalCommand {
    Status {
        goal: GoalId,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Tasks {
        goal: GoalId,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        after: Option<TaskId>,
        /// One independent page; no automatic aggregation.
        #[arg(long, default_value_t = 64, value_parser = clap::value_parser!(u16).range(1..=128))]
        maximum: u16,
        #[arg(long)]
        json: bool,
    },
}
fn print_control(response: ControlResponse, json: bool, metadata: bool) -> Result<()> {
    let observation = if metadata {
        "runtime_metadata"
    } else {
        "independent_scoped_observation"
    };
    let unavailable = if metadata {
        vec![
            "project_goal_task_hierarchy",
            "effective_native_limits",
            "unit_provider_evidence",
            "wait_cleanup_details",
        ]
    } else {
        vec![
            "native_dispatch",
            "unit_provider_evidence",
            "wait_cleanup_details",
        ]
    };
    let output = serde_json::json!({
        "observation": observation, "complete": false,
        "unavailable_fields": unavailable, "facts": response,
    });
    if json {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!(
            "{}\nIncomplete observation: {}",
            serde_json::to_string_pretty(&output["facts"])?,
            unavailable.join(", ")
        );
    }
    Ok(())
}
#[derive(Subcommand)]
enum ProjectCommand {
    /// Register an exact repository root; repeat to validate recovery/reactivation.
    Add {
        path: PathBuf,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        base: Option<String>,
        /// Rule/source-of-truth file relative to the source root (repeatable).
        #[arg(long = "rule")]
        rules: Vec<PathBuf>,
        /// Environment reference name only, never NAME=value (repeatable).
        #[arg(long = "env-ref")]
        environment_refs: Vec<String>,
        #[arg(long)]
        worktree_root: Option<PathBuf>,
        #[arg(long)]
        max_tasks: Option<usize>,
        #[arg(long)]
        clear_project_config: bool,
        #[arg(long)]
        clear_rules: bool,
        #[arg(long)]
        clear_env_refs: bool,
    },
    /// Validate and list registered/blocked projects.
    List {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        json: bool,
    },
    /// Inspect one project by ID/name or infer the owning registered CWD.
    Status {
        project: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Soft-remove an idle project, retaining audit/history and filesystem contents.
    Remove { project: String },
}
fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(command @ (Command::Serve | Command::Status { .. } | Command::Goal { .. })) => {
            ensure!(
                cli.project_config.is_none(),
                "Runtime commands use the registered Project configuration; --project-config is not accepted"
            );
            let config = Config::load(cli.config.as_deref(), None)?;
            let state = cli.state.map(Ok).unwrap_or_else(default_state_path)?;
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()?;
            runtime.block_on(async {
                match command {
                    Command::Serve => service::serve(&state, config).await,
                    Command::Status { all: _, json } => {
                        let response =
                            client::request(&state, ControlAction::RuntimeStatus).await?;
                        ensure!(
                            matches!(response, ControlResponse::RuntimeMetadata { .. }),
                            "unexpected Runtime response"
                        );
                        print_control(response, json, true)
                    }
                    Command::Goal { command } => {
                        let (goal, selector, json, page) = match command {
                            GoalCommand::Status {
                                goal,
                                project,
                                json,
                            } => (goal, project, json, None),
                            GoalCommand::Tasks {
                                goal,
                                project,
                                after,
                                maximum,
                                json,
                            } => (goal, project, json, Some((after, maximum))),
                        };
                        let resolved = client::request(
                            &state,
                            ControlAction::ResolveProject {
                                selector,
                                cwd: std::env::current_dir()?,
                            },
                        )
                        .await?;
                        let ControlResponse::ProjectResolved { project, .. } = resolved else {
                            anyhow::bail!("unexpected Project routing response");
                        };
                        let action = match page {
                            Some((after, maximum)) => ControlAction::GoalTasks {
                                project,
                                goal,
                                after,
                                maximum: usize::from(maximum),
                            },
                            None => ControlAction::GoalStatus { project, goal },
                        };
                        let response = client::request(&state, action).await?;
                        ensure!(
                            matches!(
                                (&response, page),
                                (ControlResponse::GoalFacts { .. }, None)
                                    | (ControlResponse::GoalTaskPage { .. }, Some(_))
                            ),
                            "unexpected Goal response"
                        );
                        print_control(response, json, false)
                    }
                    _ => unreachable!(),
                }
            })?;
        }
        Some(Command::ConfigCheck) => {
            let config = Config::load(cli.config.as_deref(), cli.project_config.as_deref())?;
            println!(
                "Configuration valid: global sessions {}, tasks/project {}, context {}",
                config.scheduler.global_max_sessions,
                config.scheduler.max_tasks_per_project,
                if config.context.enabled {
                    "enabled"
                } else {
                    "baseline"
                }
            );
        }
        Some(Command::Project { command }) => {
            let runtime = Config::load(cli.config.as_deref(), None)?;
            ensure!(
                cli.project_config.is_none() || matches!(&command, ProjectCommand::Add { .. }),
                "--project-config is only accepted by project add or config-check; selected projects use their registered reference"
            );
            let state = cli.state.map(Ok).unwrap_or_else(default_state_path)?;
            if let Some(parent) = state.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)?;
            }
            let mut store = Store::open(&state)?;
            let mut registry = ProjectRegistry::new(&mut store);
            let cwd = std::env::current_dir()?;
            match command {
                ProjectCommand::Add {
                    path,
                    name,
                    base,
                    rules,
                    environment_refs,
                    worktree_root,
                    max_tasks,
                    clear_project_config,
                    clear_rules,
                    clear_env_refs,
                } => {
                    registry.reconcile()?;
                    let project = registry.add(
                        &path,
                        AddProject {
                            name,
                            base_branch: base,
                            config_ref: cli.project_config,
                            rule_refs: rules,
                            environment_refs,
                            worktree_root,
                            max_tasks,
                            clear_config: clear_project_config,
                            clear_rules,
                            clear_environment: clear_env_refs,
                        },
                        &runtime,
                    )?;
                    println!(
                        "{}\t{}\t{:?}\t{}",
                        project.id,
                        project.name,
                        project.state,
                        project.root.display()
                    );
                }
                ProjectCommand::List { all, json } => {
                    let projects = registry.list(all)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&projects)?);
                    } else {
                        for p in projects {
                            println!(
                                "{}\t{}\t{:?}\t{}{}",
                                p.id,
                                p.name,
                                p.state,
                                p.root.display(),
                                p.blocked_reason
                                    .as_ref()
                                    .map(|r| format!("\t{r}"))
                                    .unwrap_or_default()
                            );
                        }
                    }
                }
                ProjectCommand::Status { project, json } => {
                    let status = registry.status(project.as_deref(), &cwd)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&status)?);
                    } else {
                        let (goals, tasks) = ProjectRegistry::active_counts(&status);
                        println!(
                            "{}\t{}\t{:?}\nRoot: {}\nBase: {}\nNamespace: {}\nActive goals: {goals}; active tasks: {tasks}; sessions: {}",
                            status.project.id,
                            status.project.name,
                            status.project.state,
                            status.project.root.display(),
                            status.project.base_branch,
                            status.project.worktree_root.display(),
                            status.sessions.len()
                        );
                        if let Some(reason) = status.project.blocked_reason {
                            println!("Blocked: {reason}");
                        }
                    }
                }
                ProjectCommand::Remove { project } => {
                    let p = registry.remove(&project, &cwd)?;
                    println!("{}\t{}\t{:?}", p.id, p.name, p.state);
                }
            }
        }
        None => {
            Config::load(cli.config.as_deref(), cli.project_config.as_deref())?;
            use clap::CommandFactory;
            Cli::command().print_help()?;
            println!();
        }
    }
    Ok(())
}
fn main() -> ExitCode {
    match rrx::execution::ipc::tool_entry() {
        Ok(Some(code)) => return ExitCode::from(u8::try_from(code).unwrap_or(125)),
        Ok(None) => {}
        Err(error) => {
            eprintln!("rrx task tool: {error:#}");
            return ExitCode::from(125);
        }
    }
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rrx: {error:#}");
            ExitCode::FAILURE
        }
    }
}
