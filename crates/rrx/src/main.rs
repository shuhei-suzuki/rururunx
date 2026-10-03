use std::{path::PathBuf, process::ExitCode};

use anyhow::Result;
use clap::{Parser, Subcommand};
use rrx::config::Config;

#[derive(Parser)]
#[command(version, about = "Local workflow runtime for native coding agents")]
struct Cli {
    /// Explicit runtime-global TOML configuration.
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    /// TOML overlay for the selected project only.
    #[arg(long, global = true)]
    project_config: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Validate configuration without starting sessions or creating state.
    ConfigCheck,
}

fn run(cli: Cli) -> Result<()> {
    let config = Config::load(cli.config.as_deref(), cli.project_config.as_deref())?;
    match cli.command {
        Some(Command::ConfigCheck) => {
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
        None => {
            use clap::CommandFactory;
            Cli::command().print_help()?;
            println!();
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rrx: {error:#}");
            ExitCode::FAILURE
        }
    }
}
