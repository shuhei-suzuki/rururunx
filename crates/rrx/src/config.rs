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

impl Config {
    /// Load runtime configuration and one explicitly selected project overlay.
    /// Missing explicit inputs are errors; absence of optional inputs uses defaults.
    pub fn load(global: Option<&Path>, project: Option<&Path>) -> Result<Self> {
        let mut value = toml::Value::try_from(Self::default())?;
        for path in [global, project].into_iter().flatten() {
            let text = fs::read_to_string(path)
                .with_context(|| format!("cannot read config {}", path.display()))?;
            let overlay: toml::Value = toml::from_str(&text)
                .with_context(|| format!("invalid TOML config {}", path.display()))?;
            merge(&mut value, overlay);
        }
        let result: Self = value.try_into().context("invalid configuration fields")?;
        result.validate()?;
        Ok(result)
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
                bail!("agent name and concurrency must be nonempty/positive");
            }
            if agent.command.first().is_some_and(|s| s.trim().is_empty()) {
                bail!("agent {name} executable must not be empty");
            }
        }
        Ok(())
    }
}

fn merge(base: &mut toml::Value, overlay: toml::Value) {
    match (base, overlay) {
        (toml::Value::Table(base), toml::Value::Table(overlay)) => {
            for (key, value) in overlay {
                if let Some(existing) = base.get_mut(&key) {
                    merge(existing, value);
                } else {
                    base.insert(key, value);
                }
            }
        }
        (base, overlay) => *base = overlay,
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
    fn overlay_preserves_unmodified_global_fields() {
        let mut value = toml::Value::try_from(Config::default()).unwrap();
        merge(
            &mut value,
            toml::from_str(
                "[scheduler]\nglobal_max_sessions = 8\n[agents.codex]\nmodel = 'configured'\n",
            )
            .unwrap(),
        );
        merge(
            &mut value,
            toml::from_str(
                "[scheduler]\nmax_tasks_per_project = 2\n[agents.codex]\neffort = 'high'\n",
            )
            .unwrap(),
        );
        let config: Config = value.try_into().unwrap();
        assert_eq!(config.scheduler.global_max_sessions, 8);
        assert_eq!(config.scheduler.max_tasks_per_project, 2);
        assert_eq!(config.agents["codex"].model.as_deref(), Some("configured"));
        assert_eq!(config.agents["codex"].effort.as_deref(), Some("high"));
    }

    #[test]
    fn unknown_fields_and_invalid_limits_fail_closed() {
        assert!(toml::from_str::<Config>("minimum_workflow = 'LOOSE'").is_err());
        assert!(toml::from_str::<Config>("unexpected = true").is_err());
        let mut config = Config::default();
        config.scheduler.global_max_sessions = 0;
        assert!(config.validate().is_err());
        config.scheduler.global_max_sessions = 4;
        config.agents.insert(
            "codex".into(),
            AgentConfig {
                max_concurrent: Some(0),
                ..Default::default()
            },
        );
        assert!(config.validate().is_err());
    }
}
