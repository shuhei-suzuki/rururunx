use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// Resolve a protected namespace even before its normal suffix has been created.
pub(crate) fn resolve_protected_path(path: &Path) -> Result<PathBuf> {
    ensure!(
        path.is_absolute()
            && path
                .components()
                .all(|c| matches!(c, Component::RootDir | Component::Normal(_)))
            && path
                .as_os_str()
                .as_encoded_bytes()
                .split(|b| *b == b'/')
                .all(|c| c != b"." && c != b".."),
        "protected path must be absolute with normal components"
    );
    let mut ancestor = path;
    let mut suffix = Vec::new();
    let mut resolved = loop {
        match ancestor.canonicalize() {
            Ok(real) => break real,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                // A dangling symlink is an existing alias, not an absent normal component.
                match std::fs::symlink_metadata(ancestor) {
                    Err(missing) if missing.kind() == std::io::ErrorKind::NotFound => {}
                    Err(other) => return Err(other.into()),
                    Ok(_) => return Err(error.into()),
                }
                suffix.push(
                    ancestor
                        .file_name()
                        .ok_or_else(|| anyhow::anyhow!("protected path has no ancestor"))?,
                );
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("protected path has no ancestor"))?;
            }
            Err(error) => return Err(error.into()),
        }
    };
    if !suffix.is_empty() {
        ensure!(
            resolved.is_dir(),
            "protected path ancestor is not a directory"
        );
    }
    for component in suffix.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

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
    /// Runtime controls and executable search paths must never become test write grants.
    pub(crate) fn test_protected_paths(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        if let Some(home) = &self.clis.codex.codex_home {
            paths.push(resolve_protected_path(home)?);
        }
        for prefix in &self.path_prefix {
            paths.push(resolve_protected_path(prefix)?);
        }
        for cli in [&self.clis.codex, &self.clis.claude] {
            let executable = cli.executable.canonicalize()?;
            paths.push(
                executable
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("CLI parent missing"))?
                    .to_path_buf(),
            );
            paths.push(executable);
        }
        Ok(paths)
    }

    pub(crate) fn test_writable_paths(&self) -> Result<Vec<PathBuf>> {
        std::iter::once(&self.work_root)
            .chain(self.test_cache_dirs.iter())
            .chain(self.clis.codex.codex_home.iter())
            .map(|p| resolve_protected_path(p))
            .collect()
    }

    pub(crate) fn check_project_root(&self, project: &std::path::Path) -> Result<()> {
        for private in [Some(&self.work_root), self.clis.codex.codex_home.as_ref()]
            .into_iter()
            .flatten()
        {
            let private = resolve_protected_path(private)?;
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
