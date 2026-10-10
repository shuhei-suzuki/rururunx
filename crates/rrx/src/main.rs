use anyhow::{Result, ensure};
use clap::{Args, Parser, Subcommand};
use rrx::{
    cli::{
        client,
        project::{self, ProjectRequest},
        read, service,
    },
    config::Config,
    domain::{GoalId, ProjectId, SessionId, TaskId},
    project::{AddProject, default_state_path},
    runtime::{
        control::{ControlAction, ControlResponse, GoalControl},
        goal::GoalPlan,
    },
};
use serde::Deserialize;
use std::{
    io::Read,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    process::ExitCode,
};

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
    Serve {
        /// Internal to `daemon start`: detach the session and announce readiness.
        #[arg(long, hide = true)]
        detached: bool,
    },
    /// Start, inspect or stop the detached service for this state root.
    Daemon {
        #[command(subcommand)]
        command: DaemonCommand,
    },
    /// Inspect current metadata (full hierarchy and native capacity remain unavailable).
    Status {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        json: bool,
    },
    /// Propose inert work, accept an explicit plan, or control a scoped Goal.
    #[command(args_conflicts_with_subcommands = true)]
    Goal {
        #[command(flatten)]
        input: GoalInput,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        json: bool,
        #[command(subcommand)]
        command: Option<GoalCommand>,
    },
    /// Validate configuration without starting sessions or creating state.
    ConfigCheck,
    /// Register and inspect isolated repository projects.
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
    /// Read one Task's Review/Approval metadata or Sessions (one page).
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    /// One page of the derived attention queue: all Projects, or one with --project.
    Attention {
        #[arg(long)]
        project: Option<String>,
        /// `<project>:<goal>:goal` or `<project>:<goal>:task:<task>` from a previous page.
        #[arg(long)]
        after: Option<String>,
        #[arg(long, default_value_t = 64, value_parser = clap::value_parser!(u16).range(1..=128))]
        maximum: u16,
        #[arg(long)]
        json: bool,
    },
    /// One page of audit summaries (fixed kinds, never event data).
    Events {
        #[command(flatten)]
        scope: ScopeArgs,
        /// The last audit sequence of a previous page.
        #[arg(long, value_parser = clap::value_parser!(i64).range(1..))]
        after: Option<i64>,
        #[arg(long, default_value_t = 64, value_parser = clap::value_parser!(u16).range(1..=128))]
        maximum: u16,
        #[arg(long)]
        json: bool,
    },
    /// Native/session routing metadata (unavailable in this release).
    Routing {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        json: bool,
    },
    /// Token, cache and cost metrics (unavailable: stored Usage is unqualified).
    Metrics {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        json: bool,
    },
}
#[derive(Args)]
struct ScopeArgs {
    #[arg(long)]
    project: Option<String>,
    #[arg(long)]
    goal: Option<GoalId>,
    #[arg(long, requires = "goal")]
    task: Option<TaskId>,
}
impl From<ScopeArgs> for read::ScopeSelector {
    fn from(args: ScopeArgs) -> Self {
        Self {
            project: args.project,
            goal: args.goal,
            task: args.task,
        }
    }
}
#[derive(Subcommand)]
enum TaskCommand {
    /// Review/Approval metadata; never a decision.
    Review {
        task: TaskId,
        #[arg(long)]
        goal: GoalId,
        #[arg(long)]
        project: Option<String>,
        /// `review:<id>` or `approval:<id>` from a previous page.
        #[arg(long)]
        after: Option<String>,
        #[arg(long, default_value_t = 64, value_parser = clap::value_parser!(u16).range(1..=128))]
        maximum: u16,
        #[arg(long)]
        json: bool,
    },
    /// The Task's own Sessions.
    Sessions {
        task: TaskId,
        #[arg(long)]
        goal: GoalId,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        after: Option<SessionId>,
        #[arg(long, default_value_t = 64, value_parser = clap::value_parser!(u16).range(1..=128))]
        maximum: u16,
        #[arg(long)]
        json: bool,
    },
}
#[derive(Subcommand)]
enum DaemonCommand {
    /// Start one detached service; returns once its endpoint validates.
    Start {
        #[arg(long)]
        json: bool,
    },
    /// Report running, owner busy, discovery unavailable or not running.
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Request RuntimeStop through the endpoint; never a signal.
    Stop {
        #[arg(long)]
        json: bool,
    },
}
#[derive(Args)]
#[group(multiple = false)]
struct GoalInput {
    /// Inert objective only; no executable Tasks or criteria are inferred.
    objective: Option<String>,
    #[arg(long)]
    file: Option<PathBuf>,
    /// TOML with explicit project, expected_project and typed plan.
    #[arg(long)]
    plan: Option<PathBuf>,
}
#[derive(Args)]
struct GoalLifecycle {
    goal: GoalId,
    #[arg(long)]
    project: Option<String>,
    /// Exact observed Goal version; conflicts are never refreshed or retried.
    #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
    expected_version: u64,
    #[arg(long)]
    reason: String,
    #[arg(long)]
    json: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GoalPlanInput {
    project: String,
    expected_project: u64,
    plan: GoalPlan,
}
#[derive(Subcommand)]
enum GoalCommand {
    Pause(GoalLifecycle),
    Resume(GoalLifecycle),
    Cancel(GoalLifecycle),
    Status {
        goal: GoalId,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// One page of the Project's Goals by stable Goal ID.
    List {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        after: Option<GoalId>,
        #[arg(long, default_value_t = 64, value_parser = clap::value_parser!(u16).range(1..=128))]
        maximum: u16,
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
fn read_goal_input(path: &Path, maximum: usize) -> Result<String> {
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags((rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::NOFOLLOW).bits() as i32)
        .open(path)
        .map_err(|_| anyhow::anyhow!("Goal input cannot be opened"))?;
    ensure!(
        file.metadata()?.is_file(),
        "Goal input must be a regular file"
    );
    let mut bytes = Vec::new();
    file.by_ref()
        .take((maximum + 1) as u64)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= maximum, "Goal input exceeds bound");
    String::from_utf8(bytes).map_err(|_| anyhow::anyhow!("Goal input must be UTF-8"))
}
async fn resolve_project(state: &Path, selector: Option<String>) -> Result<(ProjectId, u64)> {
    let response = client::request(
        state,
        ControlAction::ResolveProject {
            selector,
            cwd: std::env::current_dir()?,
        },
    )
    .await?;
    let ControlResponse::ProjectResolved {
        project, version, ..
    } = response
    else {
        anyhow::bail!("unexpected Project routing response");
    };
    Ok((project, version))
}
fn print_decision(response: ControlResponse, json: bool) -> Result<()> {
    let value = serde_json::json!({
        "observation": "committed_control_decision", "native_dispatch_available": false,
        "complete": false, "facts": response,
    });
    if json {
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else {
        println!(
            "{}\nNative dispatch unavailable; full Goal operation remains incomplete.",
            serde_json::to_string_pretty(&value["facts"])?
        );
    }
    Ok(())
}
async fn goal_control(
    state: &Path,
    input: GoalInput,
    selector: Option<String>,
    json: bool,
    command: Option<GoalCommand>,
) -> Result<()> {
    if let Some(command) = command {
        let (args, target) = match command {
            GoalCommand::Pause(args) => (args, GoalControl::Pause),
            GoalCommand::Resume(args) => (args, GoalControl::Resume),
            GoalCommand::Cancel(args) => (args, GoalControl::Cancel),
            read => {
                let (goal, selector, json, page) = match read {
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
                    _ => unreachable!(),
                };
                let (project, _) = resolve_project(state, selector).await?;
                let action = match page {
                    Some((after, maximum)) => ControlAction::GoalTasks {
                        project,
                        goal,
                        after,
                        maximum: usize::from(maximum),
                        view: Some(rrx::runtime::control::GoalReadView::RecordedV1),
                    },
                    None => ControlAction::GoalStatus {
                        project,
                        goal,
                        view: Some(rrx::runtime::control::GoalReadView::RecordedV1),
                    },
                };
                let response = client::request(state, action).await.map_err(|e| {
                    anyhow::anyhow!(
                        "{e}; if this service predates recorded_v1, restart a matching rrx serve"
                    )
                })?;
                ensure!(
                    matches!(
                        (&response, page),
                        (
                            ControlResponse::GoalFacts { .. }
                                | ControlResponse::GoalProposalFacts { .. },
                            None
                        ) | (ControlResponse::GoalTaskPage { .. }, Some(_))
                    ),
                    "unexpected Goal response"
                );
                rrx::cli::goal_facts::validate(&response, project, goal)?;
                println!("{}", rrx::cli::goal_facts::render(&response, json)?);
                return Ok(());
            }
        };
        ensure!(
            !args.reason.trim().is_empty() && args.reason.len() <= 16 * 1024,
            "Goal lifecycle reason empty/over budget"
        );
        let (project, _) = resolve_project(state, args.project).await?;
        let response = client::request(
            state,
            ControlAction::SetGoalLifecycle {
                project,
                goal: args.goal,
                expected_goal: args.expected_version,
                target,
                reason: args.reason,
            },
        )
        .await?;
        ensure!(
            matches!(response, ControlResponse::GoalLifecycleChanged { .. }),
            "Goal lifecycle unavailable; no committed decision"
        );
        return print_decision(response, args.json);
    }
    let (action, proposed) = if let Some(path) = input.plan {
        ensure!(
            selector.is_none(),
            "--plan supplies its own explicit Project selection"
        );
        let text = read_goal_input(&path, rrx::cli::transport::REQUEST_BYTES)?;
        let plan: GoalPlanInput =
            toml::from_str(&text).map_err(|_| anyhow::anyhow!("Invalid typed Goal plan"))?;
        ensure!(
            !plan.project.trim().is_empty()
                && plan.project.len() <= 4096
                && plan.expected_project > 0,
            "Goal plan requires explicit Project and expected version"
        );
        let (project, version) = resolve_project(state, Some(plan.project)).await?;
        ensure!(
            version == plan.expected_project,
            "Goal plan Project version conflict"
        );
        (
            ControlAction::CreateGoal {
                project,
                expected_project: plan.expected_project,
                plan: plan.plan,
            },
            false,
        )
    } else {
        let objective = match (input.objective, input.file) {
            (Some(objective), None) => objective,
            (None, Some(path)) => read_goal_input(&path, 16 * 1024)?,
            _ => anyhow::bail!("Goal requires an objective, --file, --plan or subcommand"),
        };
        ensure!(
            !objective.trim().is_empty() && objective.len() <= 16 * 1024,
            "Goal objective empty/over budget"
        );
        let (project, expected_project) = resolve_project(state, selector).await?;
        (
            ControlAction::ProposeGoal {
                project,
                expected_project,
                objective,
            },
            true,
        )
    };
    let response = client::request(state, action).await?;
    ensure!(
        matches!(
            (&response, proposed),
            (ControlResponse::GoalProposed { .. }, true)
                | (ControlResponse::GoalAccepted { .. }, false)
        ),
        "Goal creation unavailable; no committed decision"
    );
    print_decision(response, json)
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
fn run(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        Some(Command::Daemon { command }) => {
            ensure!(
                cli.project_config.is_none(),
                "Runtime commands use the registered Project configuration; --project-config is not accepted"
            );
            let state = cli.state.map(Ok).unwrap_or_else(default_state_path)?;
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()?;
            return runtime.block_on(daemon(&state, cli.config.as_deref(), command));
        }
        Some(
            command @ (Command::Task { .. }
            | Command::Attention { .. }
            | Command::Events { .. }
            | Command::Routing { .. }
            | Command::Metrics { .. }
            | Command::Goal {
                command: Some(GoalCommand::List { .. }),
                ..
            }),
        ) => {
            ensure!(
                cli.project_config.is_none(),
                "Runtime commands use the registered Project configuration; --project-config is not accepted"
            );
            let (request, json) = read_request(command)?;
            let state = cli.state.map(Ok).unwrap_or_else(default_state_path)?;
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()?;
            return runtime.block_on(read::run(&state, request, json));
        }
        Some(command @ (Command::Serve { .. } | Command::Status { .. } | Command::Goal { .. })) => {
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
                    Command::Serve { detached } => service::serve(&state, config, detached).await,
                    Command::Status { all: _, json } => {
                        let response =
                            client::request(&state, ControlAction::RuntimeStatus).await?;
                        ensure!(
                            matches!(response, ControlResponse::RuntimeMetadata { .. }),
                            "unexpected Runtime response"
                        );
                        print_control(response, json, true)
                    }
                    Command::Goal {
                        input,
                        project,
                        json,
                        command,
                    } => goal_control(&state, input, project, json, command).await,
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
            // C-S2d: refused before reconcile and before Store::open writes.
            if let ProjectCommand::Add {
                max_tasks: Some(requested),
                ..
            } = &command
            {
                rrx::config::ensure_mvp_project_tasks(
                    rrx::config::LimitOrigin::CliFlag,
                    *requested,
                )?;
            }
            let runtime = Config::load(cli.config.as_deref(), None)?;
            ensure!(
                cli.project_config.is_none() || matches!(&command, ProjectCommand::Add { .. }),
                "--project-config is only accepted by project add or config-check; selected projects use their registered reference"
            );
            let state = cli.state.map(Ok).unwrap_or_else(default_state_path)?;
            if let Some(parent) = state.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)?;
            }
            let request = match command {
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
                } => ProjectRequest::Add {
                    path,
                    options: AddProject {
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
                },
                ProjectCommand::List { all, json } => ProjectRequest::List { all, json },
                ProjectCommand::Status { project, json } => ProjectRequest::Status {
                    selector: project,
                    json,
                },
                ProjectCommand::Remove { project } => ProjectRequest::Remove { selector: project },
            };
            let executor = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()?;
            return executor.block_on(project::run(&state, runtime, request));
        }
        None => {
            Config::load(cli.config.as_deref(), cli.project_config.as_deref())?;
            use clap::CommandFactory;
            Cli::command().print_help()?;
            println!();
        }
    }
    Ok(ExitCode::SUCCESS)
}
/// S4: the parsed read and its output mode.
fn read_request(command: Command) -> Result<(read::ReadRequest, bool)> {
    use read::{ReadRequest, TaskSelector};
    Ok(match command {
        Command::Goal {
            command:
                Some(GoalCommand::List {
                    project,
                    after,
                    maximum,
                    json,
                }),
            ..
        } => (
            ReadRequest::Goals {
                project,
                after,
                maximum: usize::from(maximum),
            },
            json,
        ),
        Command::Task {
            command:
                TaskCommand::Review {
                    task,
                    goal,
                    project,
                    after,
                    maximum,
                    json,
                },
        } => (
            ReadRequest::Review {
                task: TaskSelector {
                    project,
                    goal,
                    task,
                },
                after: after
                    .as_deref()
                    .map(read::parse_review_cursor)
                    .transpose()?,
                maximum: usize::from(maximum),
            },
            json,
        ),
        Command::Task {
            command:
                TaskCommand::Sessions {
                    task,
                    goal,
                    project,
                    after,
                    maximum,
                    json,
                },
        } => (
            ReadRequest::Sessions {
                task: TaskSelector {
                    project,
                    goal,
                    task,
                },
                after,
                maximum: usize::from(maximum),
            },
            json,
        ),
        Command::Attention {
            project,
            after,
            maximum,
            json,
        } => (
            ReadRequest::Attention {
                project,
                after: after
                    .as_deref()
                    .map(read::parse_attention_cursor)
                    .transpose()?,
                maximum: usize::from(maximum),
            },
            json,
        ),
        Command::Events {
            scope,
            after,
            maximum,
            json,
        } => (
            ReadRequest::Events {
                scope: scope.into(),
                after,
                maximum: usize::from(maximum),
            },
            json,
        ),
        Command::Routing { scope, json } => (
            ReadRequest::Routing {
                scope: scope.into(),
            },
            json,
        ),
        Command::Metrics { scope, json } => (
            ReadRequest::Metrics {
                scope: scope.into(),
            },
            json,
        ),
        _ => unreachable!("not an S4 read"),
    })
}
fn print_outcome<T: serde::Serialize>(json: bool, label: &str, outcome: &T) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(outcome)?);
    } else {
        let detail = serde_json::to_value(outcome)?;
        println!("{label}\n{}", serde_json::to_string_pretty(&detail)?);
    }
    Ok(())
}
async fn daemon(state: &Path, config: Option<&Path>, command: DaemonCommand) -> Result<ExitCode> {
    use rrx::cli::daemon::{self, DaemonStatus, StartOutcome, StopOutcome};
    Ok(match command {
        DaemonCommand::Start { json } => {
            let outcome = daemon::start(state, config).await;
            print_outcome(json, outcome.label(), &outcome)?;
            if matches!(outcome, StartOutcome::Running { .. }) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        DaemonCommand::Status { json } => {
            let status = daemon::status(state).await;
            print_outcome(json, status.label(), &status)?;
            // LSB-style: 0 running, 3 not running, 4 unknown.
            ExitCode::from(match status {
                DaemonStatus::Running { .. } => 0,
                DaemonStatus::NotRunning => 3,
                DaemonStatus::OwnerBusy | DaemonStatus::DiscoveryUnavailable { .. } => 4,
            })
        }
        DaemonCommand::Stop { json } => {
            let outcome = daemon::stop(state).await;
            print_outcome(json, outcome.label(), &outcome)?;
            if matches!(outcome, StopOutcome::Completed { .. }) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    })
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
    let cli = Cli::parse();
    if matches!(cli.command, Some(Command::Serve { detached: true })) {
        // The first action of a detached service, before config or owner.
        if let Err(error) = service::detach() {
            eprintln!("rrx: {error:#}");
            return ExitCode::from(service::EXIT_DETACH_FAILED);
        }
    }
    match run(cli) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("rrx: {error:#}");
            ExitCode::from(service::exit_code(&error))
        }
    }
}
