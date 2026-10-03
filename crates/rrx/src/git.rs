//! Scoped Git operations and durable advisory locks. Native Git guards remain authoritative.
use crate::{domain::*, state::Store};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorktreeLock {
    pub active: bool,
    pub revision: String,
    pub worktree: PathBuf,
    pub branch: String,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct WorktreeStatus {
    pub worktree: PathBuf,
    pub branch: String,
    pub revision: String,
    /// Includes ignored files: cleanup must not silently remove build/user artifacts.
    pub dirty: bool,
}

/// Observed Git metadata, validated identically by synchronous management and
/// bounded asynchronous adapter preflight. Collectors must use native Git.
pub struct WorktreeOwnershipFacts {
    pub source_top: PathBuf,
    pub source_git_dir: PathBuf,
    pub source_common: PathBuf,
    pub source_roots: Vec<String>,
    pub task_top: PathBuf,
    pub task_common: PathBuf,
    pub branch: String,
    pub revision: String,
}

/// Read-only primary Project ownership facts; this does not authorize execution.
pub struct ProjectOwnershipFacts {
    pub top: PathBuf,
    pub git_dir: PathBuf,
    pub common: PathBuf,
    pub roots: Vec<String>,
}
pub fn validate_project_ownership(
    project: &Project,
    facts: ProjectOwnershipFacts,
) -> Result<(PathBuf, PathBuf)> {
    let root = project.root.canonicalize()?;
    ensure!(
        root == project.root && facts.top.canonicalize()? == root,
        "Project root must be exact canonical Git top-level"
    );
    let common = facts.common.canonicalize()?;
    ensure!(
        facts.git_dir.canonicalize()? == common,
        "Project source root cannot be a linked worktree"
    );
    let mut roots = facts.roots;
    roots.sort();
    ensure!(
        !roots.is_empty()
            && serde_json::to_string(&(common.clone(), roots))? == project.repository_identity,
        "Project repository identity changed"
    );
    Ok((root, common))
}

pub fn validate_worktree_ownership(
    project: &Project,
    task: &Task,
    facts: WorktreeOwnershipFacts,
) -> Result<WorktreeStatus> {
    let (root, common) = validate_project_ownership(
        project,
        ProjectOwnershipFacts {
            top: facts.source_top,
            git_dir: facts.source_git_dir,
            common: facts.source_common,
            roots: facts.source_roots,
        },
    )?;
    ensure!(task.project_id == project.id, "foreign Task Project");
    let ns = namespace(project, &root)?;
    let path = task.worktree.as_ref().context("task has no worktree")?;
    ensure!(
        path.parent() == Some(ns.as_path()) && path.canonicalize()? == *path,
        "task path must be canonical direct child of namespace"
    );
    ensure!(
        facts.task_top.canonicalize()? == *path && *path != root,
        "Task must use exact independent worktree"
    );
    ensure!(
        facts.task_common.canonicalize()? == common,
        "Task worktree belongs to another repository"
    );
    protect_branch(&facts.branch, &project.base_branch)?;
    ensure!(
        task.branch.as_deref() == Some(facts.branch.as_str()),
        "task branch mismatch"
    );
    ensure!(
        !facts.revision.trim().is_empty(),
        "missing worktree revision"
    );
    Ok(WorktreeStatus {
        worktree: path.clone(),
        branch: facts.branch,
        revision: facts.revision,
        dirty: false,
    })
}

pub struct WorktreeManager;
impl WorktreeManager {
    pub fn create(store: &mut Store, task_id: TaskId) -> Result<WorktreeStatus> {
        let mut task = task(store, task_id)?;
        let project = project(store, &task)?;
        let root = project_root(&project)?;
        ensure!(
            task.worktree.is_none() && task.branch.is_none(),
            "task already has a worktree binding; reconcile existing intent instead"
        );
        ensure_no_executor(store, &task.scope())?;
        ensure_no_lock(store, &task.scope())?;
        let suffix = task
            .issue
            .map(|n| format!("issue-{n}"))
            .unwrap_or_else(|| format!("task-{}", task.id));
        let branch = format!("feature/{suffix}");
        protect_branch(&branch, &project.base_branch)?;
        git(&root, &["check-ref-format", "--branch", &branch])?;
        git(
            &root,
            &[
                "show-ref",
                "--verify",
                &format!("refs/heads/{}", project.base_branch),
            ],
        )?;
        ensure!(
            !git_success(
                &root,
                &[
                    "show-ref",
                    "--quiet",
                    "--verify",
                    &format!("refs/heads/{branch}")
                ]
            )?,
            "branch already exists"
        );
        let namespace = namespace(&project, &root)?;
        let path = namespace.join(suffix);
        ensure!(
            !path.exists() && path.symlink_metadata().is_err(),
            "worktree path already exists"
        );
        // Persist intent before Git/hook side effects. Never erase a failed hook's worktree.
        task.worktree = Some(path.clone());
        task.branch = Some(branch.clone());
        store.put_task(&mut task)?;
        store.audit(
            &task.scope(),
            "worktree.create_intent",
            json!({"path":path,"branch":branch}),
        )?;
        let base_revision = git_text(
            &root,
            &["rev-parse", &format!("refs/heads/{}", project.base_branch)],
        )?;
        let mut reservation = Record::new(
            task.scope(),
            RecordKind::WorktreeLock,
            serde_json::to_value(WorktreeLock {
                active: true,
                revision: base_revision,
                worktree: path.clone(),
                branch: branch.clone(),
                reason: "worktree creation".into(),
            })?,
        );
        store.put_record(&mut reservation)?;
        std::fs::create_dir_all(&namespace)?;
        git(
            &root,
            &[
                "worktree",
                "add",
                "-b",
                &branch,
                text_path(&path)?,
                &format!("refs/heads/{}", project.base_branch),
            ],
        )?;
        let status = Self::status(store, task_id)?;
        release(store, &mut reservation)?;
        store.audit(
            &task.scope(),
            "worktree.created",
            serde_json::to_value(&status)?,
        )?;
        Ok(status)
    }

    pub fn status(store: &Store, task_id: TaskId) -> Result<WorktreeStatus> {
        let task = task(store, task_id)?;
        let project = project(store, &task)?;
        owned_status(&project, &task)
    }

    /// Call before every runtime-controlled mutating operation, including continuation.
    pub fn ensure_mutation_allowed(store: &Store, task_id: TaskId) -> Result<WorktreeStatus> {
        let task = task(store, task_id)?;
        ensure_no_lock(store, &task.scope())?;
        Self::status(store, task_id)
    }

    pub fn lock_review(
        store: &mut Store,
        task_id: TaskId,
        expected_revision: &str,
        reason: &str,
    ) -> Result<RecordId> {
        ensure!(!reason.trim().is_empty(), "lock reason required");
        let task = task(store, task_id)?;
        let status = Self::status(store, task_id)?;
        ensure!(!status.dirty, "review requires a clean worktree");
        ensure!(
            status.revision == expected_revision,
            "review revision changed"
        );
        let lock = WorktreeLock {
            active: true,
            revision: status.revision,
            worktree: status.worktree,
            branch: status.branch,
            reason: reason.into(),
        };
        let mut record = Record::new(
            task.scope(),
            RecordKind::WorktreeLock,
            serde_json::to_value(lock)?,
        );
        // Store checks sessions/locks under the same SQLite write transaction as reservation.
        store.put_record(&mut record)?;
        if let Err(error) = Self::verify_review(store, record.id) {
            release(store, &mut record).with_context(|| {
                format!(
                    "failed to release provisional lock {} after verification failed: {error}",
                    record.id
                )
            })?;
            store.audit(
                &task.scope(),
                "worktree.lock_acquisition_failed",
                json!({"lock_id":record.id,"error":error.to_string()}),
            )?;
            return Err(error);
        }
        Ok(record.id)
    }

    pub fn verify_review(store: &Store, lock_id: RecordId) -> Result<WorktreeStatus> {
        let record = store.record(lock_id)?.context("unknown review lock")?;
        ensure!(
            record.kind == RecordKind::WorktreeLock,
            "not a worktree lock"
        );
        let lock: WorktreeLock = serde_json::from_value(record.data)?;
        ensure!(lock.active, "review lock is inactive");
        let status = Self::status(
            store,
            record.scope.task_id.context("lock needs task scope")?,
        )?;
        ensure!(
            status.worktree == lock.worktree
                && status.branch == lock.branch
                && status.revision == lock.revision
                && !status.dirty,
            "immutable review worktree changed externally"
        );
        Ok(status)
    }

    pub fn unlock_review(store: &mut Store, lock_id: RecordId) -> Result<()> {
        let mut record = store.record(lock_id)?.context("unknown review lock")?;
        ensure!(
            record.kind == RecordKind::WorktreeLock,
            "not a worktree lock"
        );
        let mut lock: WorktreeLock = serde_json::from_value(record.data.clone())?;
        // Verify before release; changed source requires explicit recovery, never accepted review.
        Self::verify_review(store, lock_id)?;
        lock.active = false;
        record.data = serde_json::to_value(lock)?;
        store.put_record(&mut record)
    }

    /// Only clean, merged, task-owned worktrees can be removed. Binding remains as provenance.
    pub fn cleanup(store: &mut Store, task_id: TaskId) -> Result<()> {
        let task = task(store, task_id)?;
        ensure_no_executor(store, &task.scope())?;
        let status = Self::ensure_mutation_allowed(store, task_id)?;
        ensure!(
            !status.dirty,
            "refusing cleanup of dirty/ignored worktree files"
        );
        let project = project(store, &task)?;
        let root = project_root(&project)?;
        ensure!(
            git_success(
                &root,
                &[
                    "merge-base",
                    "--is-ancestor",
                    &status.revision,
                    &format!("refs/heads/{}", project.base_branch)
                ]
            )?,
            "task branch is not merged into project base"
        );
        // Native branch -d uses upstream when set, otherwise root HEAD. Check it first.
        let upstream = command(
            &root,
            &[
                "rev-parse",
                "--verify",
                &format!("{}@{{upstream}}", status.branch),
            ],
        )?
        .output()?;
        let delete_target = if upstream.status.success() {
            String::from_utf8(upstream.stdout)?.trim().to_owned()
        } else {
            git_text(&root, &["rev-parse", "HEAD"])?
        };
        ensure!(
            git_success(
                &root,
                &[
                    "merge-base",
                    "--is-ancestor",
                    &status.revision,
                    &delete_target
                ]
            )?,
            "native branch deletion would reject task revision"
        );
        let lock_id = Self::lock_review(store, task_id, &status.revision, "worktree cleanup")?;
        Self::verify_review(store, lock_id)?;
        store.audit(
            &task.scope(),
            "worktree.cleanup_intent",
            serde_json::to_value(&status)?,
        )?;
        git(&root, &["worktree", "remove", text_path(&status.worktree)?])?;
        git(&root, &["branch", "-d", &status.branch])?;
        let mut reservation = store.record(lock_id)?.context("cleanup lock disappeared")?;
        release(store, &mut reservation)?;
        store.audit(
            &task.scope(),
            "worktree.cleaned",
            serde_json::to_value(&status)?,
        )?;
        Ok(())
    }
}

fn release(store: &mut Store, record: &mut Record) -> Result<()> {
    let mut lock: WorktreeLock = serde_json::from_value(record.data.clone())?;
    lock.active = false;
    record.data = serde_json::to_value(lock)?;
    store.put_record(record)
}

pub(crate) fn executor_reserved(session: &Session) -> bool {
    session.role == SessionRole::Executor
        && matches!(
            session.state,
            SessionState::Starting
                | SessionState::Running
                | SessionState::WaitingApproval
                | SessionState::WaitingHuman
                | SessionState::Lost
        )
}
fn ensure_no_executor(store: &Store, scope: &Scope) -> Result<()> {
    for record in store.records(scope, RecordKind::Session)? {
        let session: Session = serde_json::from_value(record.data)?;
        ensure!(
            !executor_reserved(&session),
            "executor is reserved/live; reconcile stale sessions before Git changes"
        );
    }
    Ok(())
}
fn ensure_no_lock(store: &Store, scope: &Scope) -> Result<()> {
    for record in store.records(scope, RecordKind::WorktreeLock)? {
        let lock: WorktreeLock = serde_json::from_value(record.data)?;
        ensure!(!lock.active, "worktree is locked for immutable review");
    }
    Ok(())
}
fn task(store: &Store, id: TaskId) -> Result<Task> {
    store.task(id)?.context("unknown task")
}
fn project(store: &Store, task: &Task) -> Result<Project> {
    let project = store.project(task.project_id)?.context("unknown project")?;
    ensure!(
        project.state == ProjectState::Registered,
        "project is not registered/active"
    );
    let goal = store.goal(task.goal_id)?.context("unknown goal")?;
    ensure!(goal.project_id == project.id, "foreign goal");
    Ok(project)
}
fn project_root(project: &Project) -> Result<PathBuf> {
    let root = project.root.canonicalize()?;
    ensure!(root == project.root, "project root must be canonical");
    let git_root =
        PathBuf::from(git_text(&root, &["rev-parse", "--show-toplevel"])?).canonicalize()?;
    ensure!(git_root == root, "project root must be exact Git top-level");
    ensure!(
        repository_identity(&root, &project.base_branch)? == project.repository_identity,
        "Project repository identity changed"
    );
    Ok(root)
}

/// Registry identity: canonical primary common-dir plus configured base's root commits.
/// Missing/moved/replaced or rewritten repositories require explicit reconciliation.
pub fn repository_identity(root: &Path, base_branch: &str) -> Result<String> {
    git(root, &["check-ref-format", "--branch", base_branch])?;
    let resolve = |arg| -> Result<PathBuf> {
        Ok(PathBuf::from(git_text(
            root,
            &["rev-parse", "--path-format=absolute", arg],
        )?)
        .canonicalize()?)
    };
    let common = resolve("--git-common-dir")?;
    ensure!(
        resolve("--git-dir")? == common,
        "Project source root cannot be a linked worktree"
    );
    let mut roots: Vec<String> = git_text(
        root,
        &[
            "rev-list",
            "--max-parents=0",
            &format!("refs/heads/{base_branch}"),
        ],
    )?
    .lines()
    .map(str::to_owned)
    .collect();
    roots.sort();
    ensure!(!roots.is_empty(), "repository has no base history");
    Ok(serde_json::to_string(&(common, roots))?)
}
fn namespace(project: &Project, root: &Path) -> Result<PathBuf> {
    let path = &project.worktree_root;
    let (common, _): (PathBuf, Vec<String>) = serde_json::from_str(&project.repository_identity)?;
    ensure!(
        !path.starts_with(&common) && !common.starts_with(path),
        "Task namespace overlaps Git metadata"
    );
    ensure!(
        !path.starts_with(root.join(".git")),
        "worktree namespace cannot use Git metadata"
    );
    ensure!(
        path.is_absolute() && path.starts_with(root) && path != root,
        "worktree namespace must be below project root"
    );
    ensure!(
        path.components().all(|c| !matches!(
            c,
            std::path::Component::ParentDir | std::path::Component::CurDir
        )),
        "non-normal worktree namespace"
    );
    let mut ancestor = path.as_path();
    while !ancestor.exists() {
        ancestor = ancestor
            .parent()
            .context("namespace has no existing parent")?;
    }
    ensure!(
        ancestor.canonicalize()? == ancestor,
        "symlink worktree namespace rejected"
    );
    Ok(path.clone())
}
fn owned_status(project: &Project, task: &Task) -> Result<WorktreeStatus> {
    let root = project_root(project)?;
    let ns = namespace(project, &root)?;
    let path = task.worktree.as_ref().context("task has no worktree")?;
    ensure!(
        path.parent() == Some(ns.as_path()),
        "task path outside project worktree namespace"
    );
    ensure!(path.canonicalize()? == *path, "symlink task path rejected");
    let top = PathBuf::from(git_text(path, &["rev-parse", "--show-toplevel"])?).canonicalize()?;
    ensure!(
        top == *path && top != root,
        "task must use exact independent worktree"
    );
    let common = |cwd: &Path| -> Result<PathBuf> {
        Ok(PathBuf::from(git_text(
            cwd,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        )?)
        .canonicalize()?)
    };
    ensure!(
        common(path)? == common(&root)?,
        "task worktree belongs to another repository"
    );
    let branch = git_text(path, &["symbolic-ref", "--quiet", "--short", "HEAD"])?;
    let revision = git_text(path, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    // project_root has just verified these source facts against native Git.
    let (source_common, source_roots): (PathBuf, Vec<String>) =
        serde_json::from_str(&project.repository_identity)?;
    let mut status = validate_worktree_ownership(
        project,
        task,
        WorktreeOwnershipFacts {
            source_top: root.clone(),
            source_git_dir: source_common.clone(),
            source_common,
            source_roots,
            task_top: top,
            task_common: common(path)?,
            branch,
            revision,
        },
    )?;
    let dirty = !git(
        path,
        &[
            "--no-optional-locks",
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignored=matching",
        ],
    )?
    .stdout
    .is_empty();
    status.dirty = dirty;
    Ok(status)
}
fn protect_branch(branch: &str, base: &str) -> Result<()> {
    ensure!(
        branch != "main" && branch != "master" && branch != base,
        "protected branch cannot be used for development"
    );
    Ok(())
}
fn text_path(path: &Path) -> Result<&str> {
    path.to_str().context("Git path is not UTF-8")
}
/// Runtime/native configuration is authoritative for all Git ownership collectors.
pub(crate) fn native_environment() -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    const ROUTING: &[&str] = &[
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_NAMESPACE",
        "GIT_CEILING_DIRECTORIES",
        "GIT_DISCOVERY_ACROSS_FILESYSTEM",
    ];
    std::env::vars_os()
        .filter(|(name, _)| !ROUTING.iter().any(|route| name == route))
        .collect()
}
fn command(cwd: &Path, args: &[&str]) -> Result<Command> {
    let executable = crate::adapter::resolve_executable("git")
        .map_err(|e| anyhow::anyhow!("native Git unavailable: {e:?}"))?;
    let mut cmd = Command::new(executable);
    cmd.current_dir(cwd)
        .args(args)
        .env_clear()
        .envs(native_environment());
    Ok(cmd)
}

fn git(cwd: &Path, args: &[&str]) -> Result<Output> {
    let output = command(cwd, args)?.output().context("cannot start Git")?;
    ensure!(
        output.status.success(),
        "Git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output)
}
fn git_success(cwd: &Path, args: &[&str]) -> Result<bool> {
    let status = command(cwd, args)?.output()?.status;
    ensure!(
        matches!(status.code(), Some(0 | 1)),
        "Git check failed: {status}"
    );
    Ok(status.success())
}
pub(crate) fn git_text(cwd: &Path, args: &[&str]) -> Result<String> {
    Ok(String::from_utf8(git(cwd, args)?.stdout)?
        .trim_end_matches(['\n', '\r'])
        .to_owned())
}
