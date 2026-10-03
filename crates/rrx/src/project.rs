//! Runtime-global persistent registry. Validate identity before loading scoped inputs.
use crate::{
    config::Config,
    domain::*,
    git::{WorktreeManager, git_text, repository_identity},
    state::{Store, goal_terminal, task_terminal},
};
use anyhow::{Context, Result, bail, ensure};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    env,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Default)]
pub struct AddProject {
    pub name: Option<String>,
    pub base_branch: Option<String>,
    pub config_ref: Option<PathBuf>,
    pub rule_refs: Vec<PathBuf>,
    pub environment_refs: Vec<String>,
    pub worktree_root: Option<PathBuf>,
    pub max_tasks: Option<usize>,
    pub clear_config: bool,
    pub clear_rules: bool,
    pub clear_environment: bool,
}
#[derive(Debug, Serialize)]
pub struct ProjectStatus {
    pub project: Project,
    pub goals: Vec<Goal>,
    pub tasks: Vec<Task>,
    pub sessions: Vec<Session>,
}

pub struct ProjectRegistry<'a> {
    store: &'a mut Store,
}
impl<'a> ProjectRegistry<'a> {
    pub fn new(store: &'a mut Store) -> Self {
        Self { store }
    }

    /// Registration/reactivation is explicit and preserves an existing root's ID.
    pub fn add(&mut self, path: &Path, options: AddProject, runtime: &Config) -> Result<Project> {
        runtime.validate()?;
        let root = source_root(path)?;
        let previous = self.store.projects()?.into_iter().find(|p| p.root == root);
        let base = options
            .base_branch
            .clone()
            .or_else(|| previous.as_ref().map(|p| p.base_branch.clone()))
            .map(Ok)
            .unwrap_or_else(|| default_base(&root))?;
        let identity = repository_identity(&root, &base)?;
        let mut project = match previous {
            Some(p) => {
                ensure!(
                    p.repository_identity == identity && p.base_branch == base,
                    "registered repository identity/base changed; refusing silent rebinding"
                );
                p
            }
            None => Project::new(
                root.file_name()
                    .context("repository needs a name")?
                    .to_string_lossy()
                    .into(),
                root.clone(),
                identity,
                base,
            ),
        };
        let before = serde_json::to_value(&project)?;
        ensure!(
            !options.clear_config || options.config_ref.is_none(),
            "cannot set and clear project config together"
        );
        ensure!(
            !options.clear_rules || options.rule_refs.is_empty(),
            "cannot set and clear rules together"
        );
        ensure!(
            !options.clear_environment || options.environment_refs.is_empty(),
            "cannot set and clear environment refs together"
        );
        if options.clear_config {
            project.config_ref = None;
        }
        if options.clear_rules {
            project.rule_refs.clear();
        }
        if options.clear_environment {
            project.environment_refs.clear();
        }

        if let Some(name) = options.name {
            project.name = name;
        }
        if let Some(config) = options.config_ref {
            project.config_ref = Some(resolve_file(&project, &config)?);
        }
        if !options.rule_refs.is_empty() {
            project.rule_refs = options
                .rule_refs
                .iter()
                .map(|p| resolve_file(&project, p))
                .collect::<Result<_>>()?;
        }
        if !options.environment_refs.is_empty() {
            project.environment_refs = options.environment_refs;
        }
        if let Some(namespace) = options.worktree_root {
            project.worktree_root = if namespace.is_absolute() {
                namespace
            } else {
                root.join(namespace)
            };
        }
        ensure!(
            project.name.trim() == project.name
                && !project.name.is_empty()
                && !project.name.chars().any(char::is_control)
                && project.name.parse::<ProjectId>().is_err(),
            "invalid project display name"
        );
        validate_inputs(&project)?;
        let effective = match &project.config_ref {
            Some(path) => runtime.with_project_file(&resolve_file(&project, path)?)?,
            None => runtime.clone(),
        };
        project.max_tasks = options.max_tasks.unwrap_or({
            if project.version == 0 {
                effective.scheduler.max_tasks_per_project
            } else {
                project.max_tasks
            }
        });
        ensure!(project.max_tasks > 0, "project task limit must be positive");
        project.state = ProjectState::Registered;
        project.blocked_reason = None;
        if project.version == 0 || serde_json::to_value(&project)? != before {
            self.store.put_project(&mut project)?;
        }
        Ok(project)
    }

    /// Validation never automatically clears BLOCKED. `add` validates explicit recovery.
    pub fn reconcile(&mut self) -> Result<()> {
        // A missing executable is runtime infrastructure failure, not project corruption.
        git_text(&env::current_dir()?, &["--version"])?;
        for mut project in self.store.projects()? {
            if project.state != ProjectState::Registered {
                continue;
            }
            if let Err(error) = validate(&project) {
                project.state = ProjectState::Blocked;
                project.blocked_reason = Some(format!("{error:#}"));
                self.store.put_project(&mut project)?;
            }
        }
        Ok(())
    }
    pub fn list(&mut self, include_removed: bool) -> Result<Vec<Project>> {
        self.reconcile()?;
        Ok(self
            .store
            .projects()?
            .into_iter()
            .filter(|p| include_removed || p.state != ProjectState::Removed)
            .collect())
    }
    pub fn resolve(&mut self, selector: Option<&str>, cwd: &Path) -> Result<Project> {
        self.reconcile()?;
        let projects = self.store.projects()?;
        if let Some(selector) = selector {
            // A UUID-shaped name never aliases an identity (including removed IDs).
            if let Ok(id) = selector.parse::<ProjectId>() {
                return projects
                    .into_iter()
                    .find(|p| p.id == id)
                    .context("unknown project ID");
            }
            let mut matches = projects
                .into_iter()
                .filter(|p| p.state != ProjectState::Removed && p.name == selector);
            let project = matches.next().context("unknown project name")?;
            ensure!(
                matches.next().is_none(),
                "ambiguous project name; use a stable project ID"
            );
            return Ok(project);
        }
        let cwd = cwd
            .canonicalize()
            .context("cannot resolve current directory")?;
        let mut matches = vec![];
        for project in projects
            .into_iter()
            .filter(|p| p.state != ProjectState::Removed)
        {
            if !cwd.starts_with(&project.root) {
                continue;
            }
            // Namespace descendants must be an actual, task-bound owned worktree.
            if cwd.starts_with(&project.worktree_root) {
                let mut found = false;
                for task in self.store.tasks(project.id, None)? {
                    if task.worktree.as_ref().is_some_and(|p| cwd.starts_with(p))
                        && WorktreeManager::status(self.store, task.id).is_ok()
                    {
                        found = true;
                        break;
                    }
                }
                if !found {
                    continue;
                }
            } else {
                let owned = (|| -> Result<bool> {
                    let top = PathBuf::from(git_text(&cwd, &["rev-parse", "--show-toplevel"])?)
                        .canonicalize()?;
                    Ok(top == project.root
                        && repository_identity(&project.root, &project.base_branch)?
                            == project.repository_identity)
                })()
                .unwrap_or(false);
                if !owned {
                    continue;
                }
            }
            matches.push(project);
        }
        ensure!(
            matches.len() <= 1,
            "ambiguous current project; use a stable project ID"
        );
        matches
            .pop()
            .context("current directory is not a registered project source or owned task worktree")
    }
    pub fn status(&mut self, selector: Option<&str>, cwd: &Path) -> Result<ProjectStatus> {
        let project = self.resolve(selector, cwd)?;
        let goals = self.store.goals(project.id)?;
        let tasks = self.store.tasks(project.id, None)?;
        let sessions = self
            .store
            .records(&Scope::project(project.id), RecordKind::Session)?
            .into_iter()
            .map(|r| serde_json::from_value(r.data).map_err(Into::into))
            .collect::<Result<_>>()?;
        Ok(ProjectStatus {
            project,
            goals,
            tasks,
            sessions,
        })
    }
    /// Soft removal preserves all provenance. Store checks live activity atomically.
    pub fn remove(&mut self, selector: &str, cwd: &Path) -> Result<Project> {
        let mut project = self.resolve(Some(selector), cwd)?;
        if project.state != ProjectState::Removed {
            project.state = ProjectState::Removed;
            self.store.put_project(&mut project)?;
        }
        Ok(project)
    }
    pub fn active_counts(status: &ProjectStatus) -> (usize, usize) {
        (
            status
                .goals
                .iter()
                .filter(|g| !goal_terminal(g.state))
                .count(),
            status
                .tasks
                .iter()
                .filter(|t| !task_terminal(t.state))
                .count(),
        )
    }
}

pub fn validate(project: &Project) -> Result<()> {
    ensure!(
        source_root(&project.root)? == project.root,
        "project root moved or is no longer canonical"
    );
    ensure!(
        repository_identity(&project.root, &project.base_branch)? == project.repository_identity,
        "repository identity changed at registered root"
    );
    validate_inputs(project)
}
fn source_root(path: &Path) -> Result<PathBuf> {
    let root = path
        .canonicalize()
        .with_context(|| format!("repository root missing: {}", path.display()))?;
    ensure!(root.is_dir(), "repository root must be a directory");
    ensure!(
        git_text(&root, &["rev-parse", "--is-bare-repository"])? == "false",
        "bare repositories are not project sources"
    );
    let top = PathBuf::from(git_text(&root, &["rev-parse", "--show-toplevel"])?).canonicalize()?;
    ensure!(
        top == root,
        "project path must be the exact Git repository root"
    );
    Ok(root)
}
fn default_base(root: &Path) -> Result<String> {
    if let Ok(branch) = git_text(
        root,
        &[
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ],
    ) {
        let name = branch
            .strip_prefix("origin/")
            .context("invalid remote default branch")?;
        if git_text(
            root,
            &["show-ref", "--verify", &format!("refs/heads/{name}")],
        )
        .is_ok()
        {
            return Ok(name.into());
        }
    }
    for name in ["main", "master"] {
        if git_text(
            root,
            &["show-ref", "--verify", &format!("refs/heads/{name}")],
        )
        .is_ok()
        {
            return Ok(name.into());
        }
    }
    git_text(root, &["symbolic-ref", "--quiet", "--short", "HEAD"])
        .context("cannot infer base; supply --base")
}
fn validate_inputs(project: &Project) -> Result<()> {
    validate_namespace(project)?;
    if let Some(path) = &project.config_ref {
        resolve_file(project, path)?;
    }
    for path in &project.rule_refs {
        resolve_file(project, path)?;
    }
    validate_environment(project)?;
    Ok(())
}
/// Pure Store read for runtime preflight: release the runtime mutex before `validate`.
/// Launch reservation must recheck this version/scope under its write transaction.
pub fn registered_project(store: &Store, id: ProjectId) -> Result<Project> {
    let project = store.project(id)?.context("unknown project")?;
    ensure!(
        project.state == ProjectState::Registered,
        "project is not registered/active"
    );
    Ok(project)
}
/// Return only this Project's validated reference names; never capture/store values.
pub fn environment_names(store: &Store, id: ProjectId) -> Result<Vec<String>> {
    let project = registered_project(store, id)?;
    validate(&project)?;
    Ok(project.environment_refs)
}
/// Read latest durable state and validate owning source before exposing a reference.
pub fn scoped_file(store: &Store, id: ProjectId, reference: &Path) -> Result<PathBuf> {
    let project = registered_project(store, id)?;
    validate(&project)?;
    resolve_file(&project, reference)
}

pub(crate) fn validate_environment(project: &Project) -> Result<Vec<String>> {
    let mut unique = BTreeSet::new();
    for name in &project.environment_refs {
        ensure!(
            !name.is_empty()
                && name
                    .bytes()
                    .enumerate()
                    .all(|(i, b)| b.is_ascii_alphabetic()
                        || b == b'_'
                        || (i > 0 && b.is_ascii_digit())),
            "environment references must be names, never assignments/values"
        );
        ensure!(
            !matches!(
                name.as_str(),
                "PATH"
                    | "HOME"
                    | "PWD"
                    | "OLDPWD"
                    | "SHELL"
                    | "CDPATH"
                    | "ENV"
                    | "BASH_ENV"
                    | "ZDOTDIR"
                    | "XDG_CONFIG_HOME"
                    | "XDG_STATE_HOME"
                    | "XDG_DATA_HOME"
                    | "CARGO_HOME"
                    | "RUSTUP_HOME"
                    | "CODEX_HOME"
                    | "CLAUDE_CONFIG_DIR"
                    | "PYTHONPATH"
                    | "NODE_OPTIONS"
                    | "NODE_PATH"
                    | "PYTHONHOME"
                    | "PYTHONSTARTUP"
                    | "EDITOR"
                    | "VISUAL"
                    | "PAGER"
                    | "SSH_ASKPASS"
                    | "SSH_AUTH_SOCK"
                    | "PERL5OPT"
                    | "RUBYOPT"
                    | "JAVA_TOOL_OPTIONS"
                    | "JDK_JAVA_OPTIONS"
                    | "TMPDIR"
                    | "TEMP"
                    | "TMP"
                    | "IFS"
                    | "PROMPT_COMMAND"
                    | "XDG_CACHE_HOME"
                    | "XDG_RUNTIME_DIR"
                    | "ANTHROPIC_BASE_URL"
                    | "OPENAI_BASE_URL"
                    | "OPENAI_API_BASE"
                    | "SSL_CERT_FILE"
                    | "SSL_CERT_DIR"
                    | "NODE_EXTRA_CA_CERTS"
                    | "REQUESTS_CA_BUNDLE"
                    | "CURL_CA_BUNDLE"
            ) && ![
                "GIT_",
                "LD_",
                "DYLD_",
                "RRX_",
                "CLAUDE_",
                "CODEX_",
                "ANTHROPIC_AUTH_",
                "BASH_FUNC_"
            ]
            .iter()
            .any(|prefix| name.starts_with(prefix)),
            "unsafe environment routing reference {name}"
        );
        ensure!(
            !name.to_ascii_uppercase().ends_with("_PROXY"),
            "unsafe proxy routing reference {name}"
        );
        ensure!(
            unique.insert(name),
            "duplicate environment reference {name}"
        );
    }
    Ok(project.environment_refs.clone())
}
/// Resolve only a file inside this source root; validate symlinks before every read.
pub(crate) fn resolve_file(project: &Project, reference: &Path) -> Result<PathBuf> {
    let path = if reference.is_absolute() {
        reference.to_path_buf()
    } else {
        project.root.join(reference)
    };
    ensure!(
        !path.components().any(|c| matches!(c, Component::ParentDir)),
        "parent traversal in project reference"
    );
    let canonical = path
        .canonicalize()
        .with_context(|| format!("missing project reference {}", path.display()))?;
    ensure!(
        canonical.starts_with(&project.root)
            && !canonical.starts_with(project.root.join(".git"))
            && !canonical.starts_with(git_metadata(&project.root)?)
            && !canonical.starts_with(&project.worktree_root)
            && canonical.is_file(),
        "project reference must be a source file within owning root"
    );
    // An embedded repository is a separate source, even if physically below this root.
    let top = PathBuf::from(git_text(
        canonical.parent().context("reference has no parent")?,
        &["rev-parse", "--show-toplevel"],
    )?)
    .canonicalize()?;
    ensure!(
        top == project.root,
        "project reference belongs to a nested/foreign repository"
    );
    Ok(canonical)
}
fn git_metadata(root: &Path) -> Result<PathBuf> {
    Ok(PathBuf::from(git_text(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?)
    .canonicalize()?)
}

fn validate_namespace(project: &Project) -> Result<()> {
    let path = &project.worktree_root;
    ensure!(
        path.is_absolute()
            && path.starts_with(&project.root)
            && path != &project.root
            && !path.starts_with(project.root.join(".git"))
            && !path.starts_with(git_metadata(&project.root)?)
            && !path
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir)),
        "worktree namespace must be a normalized path below project source root"
    );
    let mut ancestor = path.as_path();
    loop {
        match ancestor.symlink_metadata() {
            Ok(_) => {
                ensure!(
                    ancestor.canonicalize()? == ancestor && ancestor.is_dir(),
                    "symlink or non-directory worktree namespace"
                );
                let top = PathBuf::from(git_text(ancestor, &["rev-parse", "--show-toplevel"])?)
                    .canonicalize()?;
                ensure!(
                    top == project.root,
                    "worktree namespace belongs to a nested/foreign repository"
                );
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                ancestor = ancestor
                    .parent()
                    .context("namespace has no existing parent")?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}
pub fn effective_config(store: &Store, id: ProjectId, runtime: &Config) -> Result<Config> {
    let project = registered_project(store, id)?;
    validate(&project)?;
    runtime.validate()?;
    let mut result = match &project.config_ref {
        Some(path) => runtime.with_project_file(&resolve_file(&project, path)?)?,
        None => runtime.clone(),
    };
    result.scheduler.max_tasks_per_project = project.max_tasks;
    Ok(result)
}
/// Runtime-global default, independent of CWD. Resolving a path has no filesystem effects.
pub fn default_state_path() -> Result<PathBuf> {
    if let Some(path) = env::var_os("RRX_STATE_PATH") {
        let path = PathBuf::from(path);
        ensure!(path.is_absolute(), "RRX_STATE_PATH must be absolute");
        return Ok(path);
    }
    if let Some(path) = env::var_os("XDG_STATE_HOME") {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            return Ok(path.join("rururunx/state.sqlite3"));
        }
    }
    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        ensure!(home.is_absolute(), "HOME must be absolute");
        return Ok(home.join(".local/state/rururunx/state.sqlite3"));
    }
    bail!("no runtime state location; supply --state")
}
