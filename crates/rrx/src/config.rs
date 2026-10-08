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
    /// Explicit official native protocol. A configured alias is never a provider identity.
    pub provider: Option<String>,
    pub command: Vec<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub max_concurrent: Option<usize>,
    pub compatibility: Option<NativeCompatConfig>,
}

/// Cooperative, nonsecret declarations. They never change native permissions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCompatConfig {
    pub profile: String,
    pub cli_version: String,
    pub settings: String,
    #[serde(default)]
    pub user_hooks: Vec<NativeUserHook>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeHookWrites {
    Worktree,
    None,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeUserHook {
    pub label: String,
    pub reference: String,
    pub writes: NativeHookWrites,
}
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NativeProjectConfig {
    pub required_hooks: Vec<NativeRequiredHook>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRequiredHook {
    pub path: String,
    pub writes: NativeHookWrites,
}

fn hook_text(s: &str) -> bool {
    !s.is_empty() && s.len() <= 1024 && !s.chars().any(char::is_control)
}
impl NativeCompatConfig {
    pub(crate) fn canonical(&self) -> Result<Vec<u8>> {
        if self.user_hooks.len() > 16
            || [&self.profile, &self.cli_version, &self.settings]
                .iter()
                .any(|s| !hook_text(s))
        {
            bail!("native compatibility declaration exceeds profile");
        }
        let mut labels = std::collections::BTreeSet::new();
        for hook in &self.user_hooks {
            if hook.label.is_empty()
                || hook.label.len() > 64
                || !hook
                    .label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                || !labels.insert(&hook.label)
                || !hook_text(&hook.reference)
            {
                bail!("invalid native user hook declaration");
            }
        }
        let encoded = serde_json::to_vec(self)?;
        if encoded.len() > 16 * 1024 {
            bail!("native compatibility canonical encoding exceeds bound");
        }
        Ok(encoded)
    }
}
impl NativeProjectConfig {
    fn validate(&self) -> Result<()> {
        if self.required_hooks.len() > 16 {
            bail!("too many native required hooks");
        }
        let mut paths = std::collections::BTreeSet::new();
        for hook in &self.required_hooks {
            if !hook_text(&hook.path)
                || hook.path.starts_with('/')
                || hook.path.contains('\\')
                || hook.path.split('/').any(|s| matches!(s, "" | "." | ".."))
                || !paths.insert(&hook.path)
            {
                bail!("native required hook path must be unique and normalized");
            }
        }
        Ok(())
    }
}

/// Phase policy is data. External evidence providers perform these gates.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct WorkflowConfig {
    /// Reserved Task-creation fallback (#11); phase engine uses stored class.
    pub default: WorkflowClass,
    pub risk_mapping: [WorkflowClass; 4],
    pub browser_verification: bool,
    pub staging_verification: bool,
}
impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            default: WorkflowClass::Standard,
            risk_mapping: [
                WorkflowClass::Quick,
                WorkflowClass::Standard,
                WorkflowClass::Standard,
                WorkflowClass::Strict,
            ],
            browser_verification: false,
            staging_verification: false,
        }
    }
}
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WorkflowOverlay {
    pub default: Option<WorkflowClass>,
    pub risk_mapping: Option<[WorkflowClass; 4]>,
    pub browser_verification: Option<bool>,
    pub staging_verification: Option<bool>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub scheduler: SchedulerConfig,
    pub context: ContextConfig,
    pub minimum_workflow: WorkflowClass,
    pub workflow: WorkflowConfig,
    pub agents: BTreeMap<String, AgentConfig>,
    pub native: NativeProjectConfig,
}

/// Project inputs intentionally cannot alter runtime-wide slots or executables.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProjectOverlay {
    pub minimum_workflow: Option<WorkflowClass>,
    pub workflow: WorkflowOverlay,
    pub scheduler: ProjectSchedulerOverlay,
    pub context: ContextOverlay,
    pub agents: BTreeMap<String, ProjectAgentOverlay>,
    pub native: NativeProjectConfig,
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
        if !result.native.required_hooks.is_empty() {
            bail!("native required_hooks are project-only");
        }
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

    pub fn with_project_file(&self, path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)?;
        self.with_project_text(&text)
    }

    pub(crate) fn with_project_text(&self, text: &str) -> Result<Self> {
        let mut result = self.clone();
        result.apply_project(toml::from_str(text)?)?;
        result.validate()?;
        Ok(result)
    }

    fn apply_project(&mut self, project: ProjectOverlay) -> Result<()> {
        project.native.validate()?;
        self.native = project.native;
        if let Some(minimum) = project.minimum_workflow {
            self.minimum_workflow = self.minimum_workflow.max(minimum);
        }
        if let Some(default) = project.workflow.default {
            self.workflow.default = default;
        }
        if let Some(mapping) = project.workflow.risk_mapping {
            for (baseline, configured) in self.workflow.risk_mapping.iter_mut().zip(mapping) {
                *baseline = (*baseline).max(configured);
            }
        }
        if let Some(browser) = project.workflow.browser_verification {
            self.workflow.browser_verification = browser;
        }
        if let Some(staging) = project.workflow.staging_verification {
            self.workflow.staging_verification = staging;
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
        self.native.validate()?;
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
        if self
            .workflow
            .risk_mapping
            .windows(2)
            .any(|pair| pair[0] > pair[1])
        {
            bail!("risk workflow mapping must be monotonic");
        }
        for (name, agent) in &self.agents {
            if let Some(compatibility) = &agent.compatibility {
                compatibility.canonical()?;
            }
            if agent
                .provider
                .as_deref()
                .is_some_and(|p| !matches!(p, "claude" | "codex"))
            {
                bail!("agent {name} provider must be claude or codex");
            }
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
    #[test]
    fn nongrant_native_declaration_placement_closed_writes_and_canonical_bounds() {
        for input in [
            "[agents.a.compatibility]\nprofile='rrx-native-inherited-v1'\ncli_version='2.1.294'\nsettings='inherited'",
            "[[native.required_hooks]]\npath='hook.sh'\nwrites='output_only'",
            "[[native.required_hooks]]\npath='hook.sh'\nwrites='none'\nextra=true",
        ] {
            assert!(toml::from_str::<ProjectOverlay>(input).is_err(), "{input}");
        }
        let mut config = Config::default();
        for path in ["/absolute", "../parent", "a/./b", "a//b", "a\\b", ""] {
            assert!(
                config
                    .with_project_text(&format!(
                        "[[native.required_hooks]]\npath='{path}'\nwrites='none'"
                    ))
                    .is_err(),
                "{path}"
            );
        }
        config = config
            .with_project_text("[[native.required_hooks]]\npath='hooks/project.sh'\nwrites='none'")
            .unwrap();
        assert_eq!(config.native.required_hooks.len(), 1);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runtime.toml");
        std::fs::write(
            &path,
            "[[native.required_hooks]]\npath='hook.sh'\nwrites='none'",
        )
        .unwrap();
        assert!(Config::load(Some(&path), None).is_err());
        let raw = "profile='rrx-native-inherited-v1'\ncli_version='2.1.294'\nsettings='inherited'\n[[user_hooks]]\nlabel='ok'\nreference='opaque'\nwrites='none'";
        let mut declaration: NativeCompatConfig = toml::from_str(raw).unwrap();
        let bytes = declaration.canonical().unwrap();
        assert_eq!(bytes, declaration.clone().canonical().unwrap());
        declaration.user_hooks[0].reference = "x".repeat(1024);
        assert!(declaration.canonical().is_ok());
        declaration.user_hooks[0].reference.push('x');
        assert!(declaration.canonical().is_err());
        declaration.user_hooks[0].reference = "opaque".into();
        declaration.user_hooks[0].label = "x".repeat(64);
        assert!(declaration.canonical().is_ok());
        declaration.user_hooks[0].label.push('x');
        assert!(declaration.canonical().is_err());
        for n in 0..16 {
            declaration.user_hooks.push(NativeUserHook {
                label: format!("hook{n}"),
                reference: "opaque".into(),
                writes: NativeHookWrites::None,
            });
        }
        assert!(declaration.canonical().is_err());
    }
}
