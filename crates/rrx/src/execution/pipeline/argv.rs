use crate::config::PipelineCli;
use anyhow::{Context, Result, ensure};
use std::path::Path;

pub(super) const CODEX_FLAGS: &[&str] = &[
    "--sandbox",
    "-c",
    "-m",
    "-C",
    "-o",
    "--json",
    "--color",
    "--output-schema",
];
pub(super) const CLAUDE_FLAGS: &[&str] = &[
    "-p",
    "--permission-mode",
    "--tools",
    "--disallowedTools",
    "--strict-mcp-config",
    "--mcp-config",
    "--setting-sources",
    "--disable-slash-commands",
    "--model",
    "--output-format",
    "--json-schema",
];

#[derive(Clone, Copy)]
pub enum PipelineRole {
    Implement,
    Fix,
    Confirm,
    Review,
}

/// Fixed templates: configuration supplies values, never arbitrary switches.
pub fn step_arguments(
    role: PipelineRole,
    cli: &PipelineCli,
    work: &Path,
    attempt: &Path,
    schema: &str,
) -> Result<Vec<String>> {
    for path in [work, attempt] {
        ensure!(
            path.is_absolute()
                && path
                    .as_os_str()
                    .as_encoded_bytes()
                    .iter()
                    .all(|b| b.is_ascii_alphanumeric() || b"_/-".contains(b)),
            "pipeline template path refused"
        );
    }
    let work = work.to_str().context("non-UTF8 work path")?;
    let attempt = attempt.to_str().context("non-UTF8 attempt path")?;
    let values: Vec<String> = match role {
        PipelineRole::Review => vec![
            "-p".into(),
            "--permission-mode".into(),
            "plan".into(),
            "--tools".into(),
            "Read,Grep,Glob".into(),
            "--disallowedTools".into(),
            "Bash,Edit,Write,NotebookEdit,WebFetch,WebSearch,Task".into(),
            "--strict-mcp-config".into(),
            "--mcp-config".into(),
            format!("{attempt}/mcp.json"),
            "--setting-sources".into(),
            "".into(),
            "--disable-slash-commands".into(),
            "--model".into(),
            cli.model.clone(),
            "--output-format".into(),
            "json".into(),
            "--json-schema".into(),
            schema.into(),
        ],
        _ => {
            let confirm = matches!(role, PipelineRole::Confirm);
            let mut args = vec![
                "exec".into(),
                "--sandbox".into(),
                if confirm {
                    "read-only"
                } else {
                    "workspace-write"
                }
                .into(),
            ];
            if !confirm {
                args.extend([
                    "-c".into(),
                    "sandbox_workspace_write.network_access=false".into(),
                ]);
            }
            args.extend([
                "-c".into(),
                format!("projects.{work}.trust_level=\"untrusted\""),
                "-m".into(),
                cli.model.clone(),
                "-c".into(),
                format!(
                    "model_reasoning_effort={}",
                    cli.effort.as_ref().context("codex effort missing")?
                ),
                "-C".into(),
                work.into(),
            ]);
            if confirm {
                args.extend([
                    "--output-schema".into(),
                    format!("{attempt}/confirm.schema.json"),
                ]);
            }
            args.extend([
                "-o".into(),
                format!("{attempt}/last.{}", if confirm { "json" } else { "md" }),
                "--json".into(),
                "--color".into(),
                "never".into(),
                "-".into(),
            ]);
            args
        }
    };
    Ok(values)
}

#[cfg(test)]
mod tests;
