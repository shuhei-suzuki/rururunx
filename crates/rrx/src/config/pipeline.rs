use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    #[default]
    ManagedNative,
    CliPipeline,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ExecutionConfig {
    pub mode: ExecutionMode,
}
impl ExecutionConfig {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineConfig {
    pub work_root: PathBuf,
    #[serde(default)]
    pub path_prefix: Vec<PathBuf>,
    /// Operator-declared writable caches for sandboxed test commands.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub test_cache_dirs: Vec<PathBuf>,
    pub clis: PipelineClis,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineClis {
    pub codex: PipelineCli,
    pub claude: PipelineCli,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineCli {
    pub executable: PathBuf,
    pub version: String,
    pub model: String,
    pub effort: Option<ReasoningEffort>,
    pub codex_home: Option<PathBuf>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
    Xhigh,
}
impl std::fmt::Display for ReasoningEffort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
        })
    }
}
impl PipelineConfig {
    pub(crate) fn check_project_root(&self, project: &std::path::Path) -> Result<()> {
        for private in [Some(&self.work_root), self.clis.codex.codex_home.as_ref()]
            .into_iter()
            .flatten()
        {
            let private = private.canonicalize()?;
            ensure!(
                !project.starts_with(&private) && !private.starts_with(project),
                "Project overlaps pipeline private directory"
            );
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.test_cache_dirs.len() <= 32
                && self.test_cache_dirs.iter().all(|p| p.is_absolute()),
            "invalid pipeline test cache directories"
        );
        let root = self.work_root.as_os_str().as_encoded_bytes();
        ensure!(
            self.work_root.is_absolute()
                && root.len() <= 48
                && root
                    .iter()
                    .all(|b| b.is_ascii_alphanumeric() || b"_/-".contains(b))
                && self.work_root.components().all(|part| matches!(
                    part,
                    std::path::Component::RootDir | std::path::Component::Normal(_)
                )),
            "invalid pipeline work_root"
        );
        for dir in &self.path_prefix {
            ensure!(
                dir.is_absolute(),
                "pipeline PATH directories must be absolute"
            );
        }
        for cli in [&self.clis.codex, &self.clis.claude] {
            ensure!(
                cli.executable.is_absolute(),
                "pipeline executable must be absolute"
            );
            for text in [Some(&cli.version), Some(&cli.model)].into_iter().flatten() {
                ensure!(
                    !text.trim().is_empty()
                        && !text.starts_with('-')
                        && !text.chars().any(char::is_control),
                    "blank or invalid CLI pin"
                );
            }
        }
        ensure!(self.clis.codex.effort.is_some(), "codex effort is required");
        ensure!(
            self.clis
                .codex
                .codex_home
                .as_ref()
                .is_some_and(|p| p.is_absolute()),
            "absolute dedicated CODEX_HOME is required"
        );
        ensure!(
            self.clis.claude.codex_home.is_none() && self.clis.claude.effort.is_none(),
            "claude has no CODEX_HOME or effort override"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests;
