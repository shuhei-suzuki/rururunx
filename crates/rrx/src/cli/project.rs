//! S3 (R3.4, R3.5): `rrx project …` routing (HOW §3.5 D2, D3, D10).
//!
//! The CLI decides once per command: a valid endpoint means the control API;
//! an absent one means the offline path under `OwnerLock::exclusive`; a busy
//! lock or an invalid endpoint is refused, typed, with no write. There is no
//! direct-Store fallback after any API error.
use super::{client, endpoint};
use crate::{
    config::Config,
    domain::{Project, ProjectId},
    execution::OwnerLock,
    project::{AddProject, ProjectRegistry, ProjectStatusView},
    runtime::control::{
        ControlAction, ControlResponse, ProjectOptions, ReconcileMark, UnavailableReason,
    },
    state::Store,
};
use anyhow::{Context, Result, bail};
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

/// The exit code of a routing refusal (owner busy, discovery unavailable).
pub const EXIT_ROUTING_REFUSED: u8 = 4;

/// One `rrx project` subcommand, already parsed.
pub enum ProjectRequest {
    Add {
        path: PathBuf,
        options: AddProject,
    },
    List {
        all: bool,
        json: bool,
    },
    Status {
        selector: Option<String>,
        json: bool,
    },
    Remove {
        selector: String,
    },
}

/// D2: decides the route, then runs the command on it.
pub async fn run(state: &Path, config: Config, request: ProjectRequest) -> Result<ExitCode> {
    match endpoint::discover(state).await {
        endpoint::Discovery::Valid(_, connection) => {
            drop(connection);
            api(state, request).await?;
        }
        endpoint::Discovery::Invalid(error) => {
            eprintln!("rrx: discovery unavailable: {error:#}");
            return Ok(ExitCode::from(EXIT_ROUTING_REFUSED));
        }
        endpoint::Discovery::Absent => match OwnerLock::exclusive(state) {
            Ok(lock) => offline(&lock, &config, request)?,
            Err(error) if would_block(&error) => {
                eprintln!("rrx: owner busy: another Runtime owns this state root");
                return Ok(ExitCode::from(EXIT_ROUTING_REFUSED));
            }
            Err(error) => return Err(error),
        },
    }
    Ok(ExitCode::SUCCESS)
}
fn would_block(error: &anyhow::Error) -> bool {
    error.chain().any(|e| {
        e.downcast_ref::<rustix::io::Errno>() == Some(&rustix::io::Errno::WOULDBLOCK)
            || e.downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::WouldBlock)
    })
}

/// R3.5: every offline write, Store initialization and migration included,
/// happens while `lock` is held.
fn offline(lock: &OwnerLock, config: &Config, request: ProjectRequest) -> Result<()> {
    let mut store = Store::open(lock.state_path())?;
    let mut registry = ProjectRegistry::new(&mut store);
    let cwd = std::env::current_dir()?;
    match request {
        ProjectRequest::Add { path, options } => {
            registry.reconcile()?;
            print_saved(&registry.add(&path, options, config)?);
        }
        ProjectRequest::List { all, json } => print_list(&registry.list(all)?, json)?,
        ProjectRequest::Status { selector, json } => {
            let status = registry.status(selector.as_deref(), &cwd)?;
            print_status(&ProjectStatusView::from(&status), json)?;
        }
        ProjectRequest::Remove { selector } => {
            let project = registry.remove(&selector, &cwd)?;
            println!("{}\t{}\t{:?}", project.id, project.name, project.state);
        }
    }
    Ok(())
}

/// D3: `path` and `cwd` are canonicalized against the client's working
/// directory; source references are sent exactly as given.
async fn api(state: &Path, request: ProjectRequest) -> Result<()> {
    let cwd = std::env::current_dir()?
        .canonicalize()
        .context("cannot resolve current directory")?;
    match request {
        ProjectRequest::Add { path, options } => {
            let root = path
                .canonicalize()
                .with_context(|| format!("repository root missing: {}", path.display()))?;
            let options = project_options(options);
            // D10: exact-root lookup over every row, Removed included.
            let found = match send(
                state,
                ControlAction::ProjectLookupRoot { root: root.clone() },
            )
            .await?
            {
                ControlResponse::ProjectLookup { found } => found,
                other => bail!("unexpected Project lookup response {other:?}"),
            };
            let action = match found {
                None => ControlAction::ProjectRegister {
                    path: root,
                    options,
                },
                Some(found) => ControlAction::ProjectUpdate {
                    project: found.id,
                    expected_project: found.version,
                    options,
                },
            };
            match send(state, action).await? {
                ControlResponse::ProjectSaved { project, .. } => print_saved(&project),
                other => bail!("unexpected Project add response {other:?}"),
            }
        }
        ProjectRequest::List { all, json } => {
            match send(state, ControlAction::ProjectList { all }).await? {
                ControlResponse::ProjectRows { rows } => {
                    // C-S3g(iii): a row whose check did not finish fails the
                    // command typed; it is never printed as current.
                    let unavailable: Vec<String> = rows
                        .iter()
                        .filter(|r| r.reconcile == ReconcileMark::Unavailable)
                        .map(|r| r.project.id.to_string())
                        .collect();
                    if !unavailable.is_empty() {
                        bail!(
                            "{}: {}",
                            reason_name(UnavailableReason::ProjectPreflightUnavailable),
                            unavailable.join(", ")
                        );
                    }
                    let projects: Vec<Project> = rows.into_iter().map(|r| r.project).collect();
                    print_list(&projects, json)?;
                }
                other => bail!("unexpected Project list response {other:?}"),
            }
        }
        ProjectRequest::Status { selector, json } => {
            match send(state, ControlAction::ProjectStatus { selector, cwd }).await? {
                ControlResponse::ProjectStatusFacts { status, .. } => print_status(&status, json)?,
                other => bail!("unexpected Project status response {other:?}"),
            }
        }
        ProjectRequest::Remove { selector } => {
            let (project, version) = resolve(state, &selector, &cwd).await?;
            match send(
                state,
                ControlAction::ProjectRemove {
                    project,
                    expected_project: version,
                },
            )
            .await?
            {
                ControlResponse::ProjectSaved { project, .. } => {
                    println!("{}\t{}\t{:?}", project.id, project.name, project.state);
                }
                other => bail!("unexpected Project remove response {other:?}"),
            }
        }
    }
    Ok(())
}
async fn resolve(state: &Path, selector: &str, cwd: &Path) -> Result<(ProjectId, u64)> {
    match send(
        state,
        ControlAction::ResolveProject {
            selector: Some(selector.into()),
            cwd: cwd.to_path_buf(),
        },
    )
    .await?
    {
        ControlResponse::ProjectResolved {
            project, version, ..
        } => Ok((project, version)),
        other => bail!("unexpected Project routing response {other:?}"),
    }
}
/// Sends one action; typed refusals become errors carrying their reason.
async fn send(state: &Path, action: ControlAction) -> Result<ControlResponse> {
    match client::request(state, action).await? {
        ControlResponse::Unavailable { reason, .. } => bail!("{}", reason_name(reason)),
        ControlResponse::ProjectRefused { reason } => bail!("{reason}"),
        response => Ok(response),
    }
}
fn reason_name(reason: UnavailableReason) -> String {
    serde_json::to_value(reason)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{reason:?}"))
}
fn project_options(o: AddProject) -> ProjectOptions {
    ProjectOptions {
        name: o.name,
        base_branch: o.base_branch,
        config_ref: o.config_ref,
        rule_refs: o.rule_refs,
        environment_refs: o.environment_refs,
        worktree_root: o.worktree_root,
        max_tasks: o.max_tasks,
        clear_config: o.clear_config,
        clear_rules: o.clear_rules,
        clear_environment: o.clear_environment,
    }
}

fn print_saved(project: &Project) {
    println!(
        "{}\t{}\t{:?}\t{}",
        project.id,
        project.name,
        project.state,
        project.root.display()
    );
}
fn print_list(projects: &[Project], json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(projects)?);
        return Ok(());
    }
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
    Ok(())
}
fn print_status(status: &ProjectStatusView, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(status)?);
        return Ok(());
    }
    let project = &status.project;
    println!(
        "{}\t{}\t{:?}\nRoot: {}\nBase: {}\nNamespace: {}\nActive goals: {}; active tasks: {}; sessions: {}",
        project.id,
        project.name,
        project.state,
        project.root.display(),
        project.base_branch,
        project.worktree_root.display(),
        status.active_goals,
        status.active_tasks,
        status.sessions.len()
    );
    if let Some(reason) = &project.blocked_reason {
        println!("Blocked: {reason}");
    }
    Ok(())
}
