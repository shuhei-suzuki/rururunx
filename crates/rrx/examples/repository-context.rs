//! Inspect existing durable Task bindings without starting an agent.
use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use rrx::{
    context::{Budget, Expansion, RepositoryContext, SelectionRequest},
    domain::TaskId,
    state::Store,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
#[derive(Parser)]
struct Args {
    #[arg(long)]
    state: PathBuf,
    #[arg(long)]
    task: TaskId,
    /// Explicit source-relative ignored file admission (repeatable).
    #[arg(long = "include")]
    includes: Vec<String>,
    #[arg(long, default_value_t = 12000)]
    estimated_tokens: usize,
    #[arg(long, default_value_t = 12000)]
    bytes: usize,
    #[arg(long = "evidence")]
    evidence: Vec<String>,
    #[command(subcommand)]
    command: Operation,
}
#[derive(Subcommand)]
enum Operation {
    Map,
    Select {
        text: String,
    },
    Expand {
        #[arg(value_parser=["file","symbol","callers","callees"])]
        kind: String,
        target: String,
    },
}
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    ensure!(args.state.is_file(), "state database must already exist");
    let store = Store::open(&args.state)?;
    let scope = store.task(args.task)?.context("unknown Task")?.scope();
    let engine = RepositoryContext::new(Arc::new(Mutex::new(store)));
    let mut paths = args.includes;
    paths.extend(args.evidence.iter().cloned());
    let map = engine.index(&scope, paths).await?;
    let budget = Budget {
        estimated_tokens: args.estimated_tokens,
        bytes: args.bytes,
    };
    let request = SelectionRequest {
        mandatory_evidence: args.evidence,
        ..Default::default()
    };
    let output = match args.command {
        Operation::Map => serde_json::to_value(&map)?,
        Operation::Select { text } => serde_json::to_value(
            engine
                .select(
                    &map,
                    &SelectionRequest {
                        task_text: text,
                        ..request
                    },
                    budget,
                )
                .await?,
        )?,
        Operation::Expand { kind, target } => {
            let expansion = match kind.as_str() {
                "file" => Expansion::File { path: target },
                "symbol" => Expansion::Symbol { name: target },
                "callers" => Expansion::Callers { name: target },
                "callees" => Expansion::Callees { name: target },
                _ => unreachable!("validated by clap"),
            };
            serde_json::to_value(engine.expand(&map, &request, &expansion, budget).await?)?
        }
    };
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
