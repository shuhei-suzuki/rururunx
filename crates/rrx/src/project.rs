//! Runtime-global persistent registry. Validate identity before loading scoped inputs.
use crate::{
    config::Config,
    domain::*,
    git::{WorktreeManager, git_text, repository_identity},
    state::{Store, goal_terminal, task_terminal},
};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
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

/// S3 D1 (M1): the non-secret Session projection. `native_ref`, `pid`,
/// `recovery`, `worktree`, `model` and `effort` are never returned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionView {
    pub id: SessionId,
    pub task: Option<TaskId>,
    pub agent: String,
    pub provider: String,
    pub role: SessionRole,
    pub state: SessionState,
    pub started_at: i64,
}
impl From<&Session> for SessionView {
    fn from(session: &Session) -> Self {
        Self {
            id: session.id,
            task: session.scope.task_id,
            agent: session.agent.clone(),
            provider: session.provider.clone(),
            role: session.role,
            state: session.state,
            started_at: session.started_at,
        }
    }
}
/// S3 D1 (M1): what `project status` returns, on the API and offline. No
/// Goal, Task or Session body is serialized.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectStatusView {
    pub project: Project,
    pub active_goals: usize,
    pub active_tasks: usize,
    pub sessions: Vec<SessionView>,
}
impl From<&ProjectStatus> for ProjectStatusView {
    fn from(status: &ProjectStatus) -> Self {
        let (active_goals, active_tasks) = ProjectRegistry::active_counts(status);
        Self {
            project: public_project(status.project.clone()),
            active_goals,
            active_tasks,
            sessions: status.sessions.iter().map(SessionView::from).collect(),
        }
    }
}

/// S3 D1 (Sol 6079932858 M4, 6080527668): fixed refusal and Blocked reasons
/// a Project path produces; each is returned exactly.
const PUBLIC_REASONS: &[&str] = &[
    "unknown project ID",
    "unknown project name",
    "unknown project",
    "ambiguous project name; use a stable project ID",
    "ambiguous current project; use a stable project ID",
    "current directory is not a registered project source or owned task worktree",
    "unknown or ambiguous registered Project routing",
    "registered repository identity/base changed; refusing silent rebinding",
    "repository needs a name",
    "cannot set and clear project config together",
    "cannot set and clear rules together",
    "cannot set and clear environment refs together",
    "invalid project display name",
    "project task limit must be positive",
    "project root moved or is no longer canonical",
    "repository identity changed at registered root",
    "repository root must be a directory",
    "bare repositories are not project sources",
    "project path must be the exact Git repository root",
    "invalid remote default branch",
    "cannot infer base; supply --base",
    "repository has no base history",
    "project is not registered/active",
    "environment references must be names, never assignments/values",
    "parent traversal in project reference",
    "project reference must be a source file within owning root",
    "reference has no parent",
    "project reference belongs to a nested/foreign repository",
    "worktree namespace must be a normalized path below project source root",
    "symlink or non-directory worktree namespace",
    "worktree namespace belongs to a nested/foreign repository",
    "namespace has no existing parent",
    "project name/base branch must be nonempty",
    "project root must be absolute and task limit positive",
    "project root is not UTF-8",
    "project identity/root cannot silently change",
    "cannot change Project namespace after task worktree binding",
    "repository identity already registered",
    "Project roots/worktree namespaces overlap",
    "Project removed",
    "registered Project root moved",
    "Project preflight cancelled",
    "Git path is not UTF-8",
];
/// Formatted messages: only the fixed part is returned, never the value
/// (a path, a name, a parser message or a Git command's stderr).
const PUBLIC_PREFIXES: &[(&str, &str)] = &[
    ("Git [", "Git check failed"),
    ("Git check failed", "Git check failed"),
    ("native Git unavailable", "native Git unavailable"),
    ("cannot start Git", "Git check failed"),
    ("cannot reap Git", "Git check failed"),
    ("Git process group did not exit", "Git check failed"),
    ("repository root missing", "repository root missing"),
    ("missing project reference", "missing project reference"),
    (
        "unsafe environment routing reference",
        "unsafe environment reference",
    ),
    (
        "unsafe proxy routing reference",
        "unsafe environment reference",
    ),
    (
        "duplicate environment reference",
        "duplicate environment reference",
    ),
    ("invalid project config", "invalid project config"),
    ("cannot read config", "cannot read project config"),
    ("invalid config", "invalid project config"),
    (
        "project cannot configure unregistered runtime agent",
        "invalid project config",
    ),
    (
        "cannot remove project with active",
        "cannot remove project with active work",
    ),
    (
        "MVP supports exactly",
        "MVP supports exactly 1 active Task per Project",
    ),
];
/// The reason given when nothing in the text is allowlisted.
const PUBLIC_FALLBACK: &str = "Project check failed";
/// S3 D1 (Sol 6079932858 M4, 6080527668): the public form of a refusal or
/// Blocked reason is one of a finite set: the first `: `-separated segment
/// of the error text that is an allowlisted message, or the fixed part of an
/// allowlisted formatted one, else `Project check failed`. No part of the
/// input text is copied. Applies to stored text too, so a row written
/// before S3 is projected the same way.
pub(crate) fn public_reason(text: &str) -> String {
    text.split(": ")
        .find_map(|segment| {
            let segment = segment.trim();
            PUBLIC_REASONS
                .iter()
                .find(|known| segment == **known)
                .copied()
                .or_else(|| {
                    PUBLIC_PREFIXES
                        .iter()
                        .find(|(prefix, _)| segment.starts_with(prefix))
                        .map(|(_, public)| *public)
                })
        })
        .unwrap_or(PUBLIC_FALLBACK)
        .to_owned()
}
/// The Project row as the API and `project status` return it.
pub(crate) fn public_project(mut project: Project) -> Project {
    project.blocked_reason = project.blocked_reason.as_deref().map(public_reason);
    project
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
        let snapshot = self.store.projects()?;
        let (mut project, changed) = plan_add(&snapshot, path, options, runtime)?;
        if changed {
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
            if let Some(reason) = plan_block(&project) {
                project.state = ProjectState::Blocked;
                project.blocked_reason = Some(reason);
                // Q2: a write that lost its snapshot CAS is skipped, never
                // re-applied to the newer row.
                if let Err(error) = self.store.put_project(&mut project)
                    && !crate::state::snapshot_changed(&error)
                {
                    return Err(error);
                }
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

/// S3 D4 step 2: the add/update plan from a snapshot of every row (Removed
/// included); it holds no Store. Returns the planned row and whether it
/// differs from the stored one. The commit writes it with the snapshot
/// version as its CAS.
pub(crate) fn plan_add(
    snapshot: &[Project],
    path: &Path,
    options: AddProject,
    runtime: &Config,
) -> Result<(Project, bool)> {
    if let Some(requested) = options.max_tasks {
        crate::config::ensure_mvp_project_tasks(crate::config::LimitOrigin::CliFlag, requested)?;
    }
    runtime.validate()?;
    let root = source_root(path)?;
    let previous = snapshot.iter().find(|p| p.root == root).cloned();
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
    let changed = project.version == 0 || serde_json::to_value(&project)? != before;
    Ok((project, changed))
}

/// S3 D8: the reconcile check for one Registered row; `Some(reason)` blocks it.
pub(crate) fn plan_block(project: &Project) -> Option<String> {
    validate(project)
        .err()
        .map(|error| public_reason(&format!("{error:#}")))
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

/// Registry name grammar only; no Project/state/filesystem or environment reads.
pub(crate) fn environment_name_valid(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .enumerate()
            .all(|(i, b)| b.is_ascii_alphabetic() || b == b'_' || (i > 0 && b.is_ascii_digit()))
}

fn environment_routing_forbidden(name: &str) -> bool {
    matches!(
        name,
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
    ) || [
        "GIT_",
        "LD_",
        "DYLD_",
        "RRX_",
        "CLAUDE_",
        "CODEX_",
        "ANTHROPIC_AUTH_",
        "BASH_FUNC_",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
}

/// Registry routing policy shared with native name-only admission. Membership is
/// independent of syntax; callers validate syntax separately before ownership.
pub(crate) fn environment_name_forbidden(name: &str) -> bool {
    environment_routing_forbidden(name) || name.to_ascii_uppercase().ends_with("_PROXY")
}

/// Pure owning-reference validation, also usable for inactive Project metadata.
/// Public environment_names still performs its existing registered-source checks.
pub(crate) fn validate_environment_references(names: &[String]) -> Result<Vec<String>> {
    let mut unique = BTreeSet::new();
    for name in names {
        ensure!(
            environment_name_valid(name),
            "environment references must be names, never assignments/values"
        );
        ensure!(
            !environment_routing_forbidden(name),
            "unsafe environment routing reference {name}"
        );
        ensure!(
            !environment_name_forbidden(name),
            "unsafe proxy routing reference {name}"
        );
        ensure!(
            unique.insert(name),
            "duplicate environment reference {name}"
        );
    }
    Ok(names.to_vec())
}

pub(crate) fn validate_environment(project: &Project) -> Result<Vec<String>> {
    validate_environment_references(&project.environment_refs)
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
    // R4.5 / D8: a stored limit is never copied unchecked; a legacy row is a
    // typed refusal and stays unchanged until an explicit --max-tasks 1.
    if project.max_tasks != crate::config::MVP_PROJECT_TASKS {
        return Err(crate::state::ProjectLimitUnsupported {
            stored: project.max_tasks,
        }
        .into());
    }
    result.scheduler.max_tasks_per_project = crate::config::MVP_PROJECT_TASKS;
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

#[cfg(test)]
mod environment_policy_tests {
    use super::*;

    #[test]
    fn names_only_grammar_rejects_assignments_unicode_and_invalid_leading_bytes() {
        for valid in ["A", "_", "_1", "Project_A1", "TZ", "xai_token"] {
            assert!(environment_name_valid(valid), "{valid}");
        }
        for invalid in ["", "1TOKEN", "A=B", "A\0B", "A B", "A-B", "é", "Ａ"] {
            assert!(!environment_name_valid(invalid), "{invalid:?}");
        }
    }

    #[test]
    fn registry_control_and_proxy_policy_remains_distinct_from_credential_references() {
        for control in [
            "HOME",
            "PATH",
            "NODE_OPTIONS",
            "NODE_PATH",
            "NODE_EXTRA_CA_CERTS",
            "GIT_DIR",
            "LD_PRELOAD",
            "DYLD_LIBRARY_PATH",
            "RRX_MODE",
            "CLAUDE_CONFIG_DIR",
            "CODEX_HOME",
            "ANTHROPIC_AUTH_TOKEN",
            "BASH_FUNC_fixture",
            "HTTP_PROXY",
            "https_proxy",
            "PRIVATE_PrOxY",
        ] {
            assert!(environment_name_forbidden(control), "{control}");
            assert!(validate_environment_references(&[control.to_owned()]).is_err());
        }
        for reference in [
            "TZ",
            "XAI_API_KEY",
            "GROK_PRIVATE_TOKEN",
            "NODE_TLS_REJECT_UNAUTHORIZED",
            "OPENSSL_CONF",
            "BUN_OPTIONS",
            "SSLKEYLOGFILE",
            "PROJECT_A_TOKEN",
        ] {
            assert!(!environment_name_forbidden(reference), "{reference}");
            assert_eq!(
                validate_environment_references(&[reference.to_owned()]).unwrap(),
                [reference]
            );
        }
    }

    #[test]
    fn own_references_validate_every_entry_and_duplicates_without_source_io() {
        assert_eq!(
            validate_environment_references(&[]).unwrap(),
            Vec::<String>::new()
        );
        let names = vec!["TZ".to_owned(), "XAI_API_KEY".to_owned()];
        assert_eq!(validate_environment_references(&names).unwrap(), names);
        let duplicate = vec!["TZ".to_owned(), "TZ".to_owned()];
        assert!(
            validate_environment_references(&duplicate)
                .unwrap_err()
                .to_string()
                .starts_with("duplicate environment reference")
        );
        let assignment = vec!["TZ".to_owned(), "XAI_API_KEY=synthetic".to_owned()];
        assert!(
            validate_environment_references(&assignment)
                .unwrap_err()
                .to_string()
                .starts_with("environment references must be names")
        );
        // Proxy errors keep the public registry's existing distinction.
        assert_eq!(
            validate_environment_references(&["https_proxy".to_owned()])
                .unwrap_err()
                .to_string(),
            "unsafe proxy routing reference https_proxy"
        );
        assert_eq!(
            validate_environment_references(&["HOME".to_owned()])
                .unwrap_err()
                .to_string(),
            "unsafe environment routing reference HOME"
        );
    }
}
