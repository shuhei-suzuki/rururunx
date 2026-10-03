//! Explicit configuration inputs keep project discovery out of provider adapters.
use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum WorkflowClass {
    Quick,
    #[default]
    Standard,
    Strict,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SchedulerConfig {
    pub global_max_sessions: usize,
    pub max_tasks_per_project: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            global_max_sessions: 12,
            max_tasks_per_project: 4,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContextConfig {
    pub enabled: bool,
    pub repo_map_tokens: usize,
    pub review_context_tokens: usize,
    pub recent_history_tokens: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            repo_map_tokens: 2000,
            review_context_tokens: 12000,
            recent_history_tokens: 8000,
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AgentConfig {
    pub command: Vec<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub max_concurrent: Option<usize>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub scheduler: SchedulerConfig,
    pub context: ContextConfig,
    pub minimum_workflow: WorkflowClass,
    pub agents: BTreeMap<String, AgentConfig>,
}

/// Project inputs intentionally cannot alter runtime-wide slots or executables.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProjectOverlay {
    pub minimum_workflow: Option<WorkflowClass>,
    pub scheduler: ProjectSchedulerOverlay,
    pub context: ContextOverlay,
    pub agents: BTreeMap<String, ProjectAgentOverlay>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProjectSchedulerOverlay {
    pub max_tasks_per_project: Option<usize>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContextOverlay {
    pub enabled: Option<bool>,
    pub repo_map_tokens: Option<usize>,
    pub review_context_tokens: Option<usize>,
    pub recent_history_tokens: Option<usize>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProjectAgentOverlay {
    pub model: Option<String>,
    pub effort: Option<String>,
}

fn parse_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("cannot read config {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("invalid config {}", path.display()))
}

impl Config {
    /// Runtime and Project have distinct schemas. The merged config is one
    /// project's effective view; the global scheduler retains runtime slots.
    pub fn load(global: Option<&Path>, project: Option<&Path>) -> Result<Self> {
        let mut result: Self = match global {
            Some(path) => parse_file(path)?,
            None => Self::default(),
        };
        result.validate().with_context(|| {
            format!(
                "invalid runtime config {}",
                global.map_or_else(|| "defaults".into(), |p| p.display().to_string())
            )
        })?;
        if let Some(path) = project {
            result
                .apply_project(parse_file(path)?)
                .with_context(|| format!("invalid project config {}", path.display()))?;
            result
                .validate()
                .with_context(|| format!("invalid project config {}", path.display()))?;
        }
        Ok(result)
    }

    fn apply_project(&mut self, project: ProjectOverlay) -> Result<()> {
        if let Some(minimum) = project.minimum_workflow {
            self.minimum_workflow = self.minimum_workflow.max(minimum);
        }
        if let Some(limit) = project.scheduler.max_tasks_per_project {
            self.scheduler.max_tasks_per_project = limit;
        }
        if let Some(enabled) = project.context.enabled {
            self.context.enabled = enabled;
        }
        for (target, value) in [
            (
                &mut self.context.repo_map_tokens,
                project.context.repo_map_tokens,
            ),
            (
                &mut self.context.review_context_tokens,
                project.context.review_context_tokens,
            ),
            (
                &mut self.context.recent_history_tokens,
                project.context.recent_history_tokens,
            ),
        ] {
            if let Some(value) = value {
                *target = value;
            }
        }
        for (name, overlay) in project.agents {
            let agent = self.agents.get_mut(&name).with_context(|| {
                format!("project cannot configure unregistered runtime agent {name}")
            })?;
            if let Some(model) = overlay.model {
                agent.model = Some(model);
            }
            if let Some(effort) = overlay.effort {
                agent.effort = Some(effort);
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.scheduler.global_max_sessions == 0 || self.scheduler.max_tasks_per_project == 0 {
            bail!("scheduler limits must be positive");
        }
        if [
            self.context.repo_map_tokens,
            self.context.review_context_tokens,
            self.context.recent_history_tokens,
        ]
        .contains(&0)
        {
            bail!("context budgets must be positive");
        }
        for (name, agent) in &self.agents {
            if name.trim().is_empty() || agent.max_concurrent == Some(0) {
                bail!("agent {name:?} name and concurrency must be nonempty/positive");
            }
            if agent.command.first().is_some_and(|s| s.trim().is_empty()) {
                bail!("agent {name} executable must not be empty");
            }
            if [agent.model.as_deref(), agent.effort.as_deref()]
                .into_iter()
                .flatten()
                .any(|value| value.trim().is_empty())
            {
                bail!("agent {name} model/effort must not be blank");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_support_multiple_projects_and_four_tasks() {
        let config = Config::load(None, None).unwrap();
        assert_eq!(config.scheduler.global_max_sessions, 12);
        assert_eq!(config.scheduler.max_tasks_per_project, 4);
        assert!(config.context.enabled);
    }

    #[test]
    fn project_preserves_runtime_limits_and_only_escalates_minimum() {
        let mut config: Config = toml::from_str("minimum_workflow = 'STRICT'\n[scheduler]\nglobal_max_sessions = 8\n[agents.codex]\nmodel = 'original'\ncommand = ['codex']\nmax_concurrent = 6").unwrap();
        config.apply_project(toml::from_str("minimum_workflow = 'QUICK'\n[scheduler]\nmax_tasks_per_project = 2\n[agents.codex]\neffort = 'high'").unwrap()).unwrap();
        assert_eq!(config.minimum_workflow, WorkflowClass::Strict);
        assert_eq!(config.scheduler.global_max_sessions, 8);
        assert_eq!(config.scheduler.max_tasks_per_project, 2);
        assert_eq!(config.agents["codex"].model.as_deref(), Some("original"));
        assert_eq!(config.agents["codex"].command, ["codex"]);
        assert_eq!(config.agents["codex"].max_concurrent, Some(6));
        assert_eq!(config.agents["codex"].effort.as_deref(), Some("high"));
        assert!(WorkflowClass::Quick < WorkflowClass::Standard);
        assert!(WorkflowClass::Standard < WorkflowClass::Strict);
    }

    #[test]
    fn project_cannot_set_runtime_wide_keys() {
        for input in [
            "[scheduler]\nglobal_max_sessions = 9",
            "[agents.codex]\ncommand = ['other']",
            "[agents.codex]\nmax_concurrent = 7",
        ] {
            assert!(toml::from_str::<ProjectOverlay>(input).is_err(), "{input}");
        }
    }
}
