//! Persisted scope and native Git authority, with no filesystem/Git work under Store.
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::sync::Semaphore;

use super::protocol::failure;
use crate::{
    adapter::{
        AdapterResult, ErrorKind, InputKind, LaunchRequest, SharedStore, bounded_git,
        resolve_executable,
    },
    domain::{
        Goal, GoalState, Project, ProjectState, Record, RecordKind, Session, SessionRole, Task,
        TaskState,
    },
    git::{WorktreeLock, WorktreeOwnershipFacts},
};

static FILESYSTEM_WORKERS: Semaphore = Semaphore::const_new(16);

/// Abandoned native preflight must never free an executor reservation on a guess.
/// A separate flag per process prevents another Git child clearing a live server's flag.
#[derive(Default)]
pub(super) struct ProcessOwnership {
    flags: Vec<Arc<AtomicBool>>,
}
impl ProcessOwnership {
    pub fn group(&mut self) -> Arc<AtomicBool> {
        self.flags.retain(|flag| flag.load(Ordering::SeqCst));
        let flag = Arc::new(AtomicBool::new(false));
        self.flags.push(flag.clone());
        flag
    }
    pub fn uncertain(&self) -> bool {
        self.flags.iter().any(|flag| flag.load(Ordering::SeqCst))
    }
}

#[derive(Clone)]
pub(super) struct ScopeSnapshot {
    pub project: Project,
    pub projects: Vec<Project>,
    pub goal: Option<Goal>,
    pub task: Option<Task>,
    locks: Vec<Record>,
}

fn state_error(error: anyhow::Error) -> crate::adapter::AdapterError {
    use crate::state::StateGuardError;
    let kind = match error.downcast_ref::<StateGuardError>() {
        Some(StateGuardError::WorktreeLocked) => ErrorKind::Locked,
        Some(StateGuardError::ExecutorReserved | StateGuardError::SnapshotChanged { .. }) => {
            ErrorKind::StateConflict
        }
        Some(StateGuardError::ProjectInactive) => ErrorKind::InvalidInput,
        None => ErrorKind::StateFailure,
    };
    failure(kind, error.to_string())
}

impl ScopeSnapshot {
    pub fn versions(&self) -> [u64; 3] {
        [
            self.project.version,
            self.goal.as_ref().map_or(0, |goal| goal.version),
            self.task.as_ref().map_or(0, |task| task.version),
        ]
    }
    pub fn lock_versions(&self) -> Vec<(crate::domain::RecordId, u64)> {
        self.locks
            .iter()
            .map(|record| (record.id, record.version))
            .collect()
    }
    pub fn capture(
        store: &SharedStore,
        request: &LaunchRequest,
        agent: &str,
    ) -> AdapterResult<Self> {
        if request.scope != request.input.scope
            || request.scope.project_id != request.project.id
            || (request.scope.task_id.is_some() && request.scope.goal_id.is_none())
        {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "native launch/context scope mismatch",
            ));
        }
        let review = matches!(
            request.role,
            SessionRole::Reviewer | SessionRole::ApprovalReviewer
        );
        if review != (request.input.kind == InputKind::ReviewBundle)
            || (request.role == SessionRole::Executor && request.scope.task_id.is_none())
            || (request.scope.task_id.is_none() && request.role != SessionRole::Consultant)
        {
            return Err(failure(
                ErrorKind::InvalidInput,
                "native role/context/workspace binding is invalid",
            ));
        }
        if request.input.version == 0
            || request.input.payload.is_empty()
            || request.input.payload.len() > 900_000
            || !matches!(request.input.revision.len(), 40 | 64)
            || !request
                .input
                .revision
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || request
                .model
                .as_ref()
                .is_some_and(|s| s.trim().is_empty() || s.len() > 256 || s.contains('\0'))
            || request
                .effort
                .as_ref()
                .is_some_and(|s| s.trim().is_empty() || s.len() > 64 || s.contains('\0'))
        {
            return Err(failure(
                ErrorKind::InvalidInput,
                "native launch requires bounded versioned input and an immutable commit OID",
            ));
        }
        let store = store
            .lock()
            .map_err(|_| failure(ErrorKind::StateFailure, "state store poisoned"))?;
        let project = store
            .project(request.scope.project_id)
            .map_err(state_error)?
            .ok_or_else(|| failure(ErrorKind::OwnershipMismatch, "native Project is unknown"))?;
        if project.state != ProjectState::Registered {
            return Err(failure(
                ErrorKind::InvalidInput,
                "native Project is inactive",
            ));
        }
        if serde_json::to_value(&project)
            .map_err(|_| failure(ErrorKind::StateFailure, "Project serialization failed"))?
            != serde_json::to_value(&request.project)
                .map_err(|_| failure(ErrorKind::StateFailure, "Project serialization failed"))?
        {
            return Err(failure(
                ErrorKind::StateConflict,
                "native Project snapshot is stale or foreign",
            ));
        }
        let goal = request
            .scope
            .goal_id
            .map(|id| {
                store.goal(id).map_err(state_error).and_then(|goal| {
                    goal.ok_or_else(|| {
                        failure(ErrorKind::OwnershipMismatch, "native Goal is unknown")
                    })
                })
            })
            .transpose()?;
        if let Some(goal) = &goal {
            if goal.project_id != project.id {
                return Err(failure(ErrorKind::OwnershipMismatch, "foreign native Goal"));
            }
            if matches!(
                goal.state,
                GoalState::Paused | GoalState::Completed | GoalState::Cancelled | GoalState::Failed
            ) {
                return Err(failure(
                    ErrorKind::InvalidInput,
                    "native Goal lifecycle is inactive",
                ));
            }
        }
        let task = request
            .scope
            .task_id
            .map(|id| {
                store.task(id).map_err(state_error).and_then(|task| {
                    task.ok_or_else(|| {
                        failure(ErrorKind::OwnershipMismatch, "native Task is unknown")
                    })
                })
            })
            .transpose()?;
        if let Some(task) = &task {
            if task.scope() != request.scope
                || task.worktree.as_ref() != Some(&request.worktree)
                || (request.role == SessionRole::Executor && task.executor != agent)
            {
                return Err(failure(
                    ErrorKind::OwnershipMismatch,
                    "native Task/agent/worktree binding mismatch",
                ));
            }
            if matches!(
                task.state,
                TaskState::Completed | TaskState::Cancelled | TaskState::Merged | TaskState::Failed
            ) || (request.role == SessionRole::Executor
                && matches!(task.state, TaskState::ReadyForPr | TaskState::PrCreated))
            {
                return Err(failure(
                    ErrorKind::InvalidInput,
                    "native Task lifecycle is inactive",
                ));
            }
        } else if request.worktree != project.root {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "source consultation requires the exact registered root",
            ));
        }
        let locks = store
            .records(&request.scope, RecordKind::WorktreeLock)
            .map_err(state_error)?;
        let active = locks
            .iter()
            .map(|record| {
                serde_json::from_value::<WorktreeLock>(record.data.clone())
                    .map_err(|_| failure(ErrorKind::StateFailure, "malformed native worktree lock"))
            })
            .collect::<AdapterResult<Vec<_>>>()?;
        if request.role == SessionRole::Executor && active.iter().any(|lock| lock.active) {
            return Err(failure(
                ErrorKind::Locked,
                "native executor worktree is locked",
            ));
        }
        if request.role == SessionRole::Reviewer {
            let task = task.as_ref().expect("validated review Task");
            let matches = |lock: &WorktreeLock| {
                lock.active
                    && lock.revision == request.input.revision
                    && lock.worktree == request.worktree
                    && Some(&lock.branch) == task.branch.as_ref()
            };
            if !active.iter().any(matches)
                || active.iter().any(|lock| lock.active && !matches(lock))
            {
                return Err(failure(
                    ErrorKind::Locked,
                    "native review requires an exact immutable active worktree lock",
                ));
            }
            for record in store
                .records(&request.scope, RecordKind::Session)
                .map_err(state_error)?
            {
                let session: Session = serde_json::from_value(record.data)
                    .map_err(|_| failure(ErrorKind::StateFailure, "malformed native Session"))?;
                if crate::git::executor_reserved(&session) {
                    return Err(failure(
                        ErrorKind::Locked,
                        "native review conflicts with an active or Lost executor",
                    ));
                }
            }
        }
        Ok(Self {
            projects: store.projects().map_err(state_error)?,
            project,
            goal,
            task,
            locks,
        })
    }

    /// Includes lock versions so unlock/relock cannot silently reuse an old review snapshot.
    pub fn recheck(
        &self,
        store: &SharedStore,
        request: &LaunchRequest,
        agent: &str,
    ) -> AdapterResult<()> {
        self.recheck_authority(store, request, agent, true)
    }

    /// Credentials are already scoped in a running child. An approval must
    /// validate its own authority without depending on another Project's roster.
    pub fn recheck_scope(
        &self,
        store: &SharedStore,
        request: &LaunchRequest,
        agent: &str,
    ) -> AdapterResult<()> {
        self.recheck_authority(store, request, agent, false)
    }

    fn recheck_authority(
        &self,
        store: &SharedStore,
        request: &LaunchRequest,
        agent: &str,
        environment_roster: bool,
    ) -> AdapterResult<()> {
        let next = Self::capture(store, request, agent)?;
        if self.goal.as_ref().map(|g| g.version) != next.goal.as_ref().map(|g| g.version)
            || self.task.as_ref().map(|t| t.version) != next.task.as_ref().map(|t| t.version)
            || self
                .locks
                .iter()
                .map(|r| (r.id, r.version))
                .collect::<Vec<_>>()
                != next
                    .locks
                    .iter()
                    .map(|r| (r.id, r.version))
                    .collect::<Vec<_>>()
            || (environment_roster
                && serde_json::to_value(&self.projects).map_err(|_| {
                    failure(ErrorKind::StateFailure, "Project serialization failed")
                })? != serde_json::to_value(&next.projects).map_err(|_| {
                    failure(ErrorKind::StateFailure, "Project serialization failed")
                })?)
        {
            return Err(failure(
                ErrorKind::StateConflict,
                "native authority changed during preflight",
            ));
        }
        Ok(())
    }

    pub async fn verify_binding(
        &self,
        request: &LaunchRequest,
        ownership: &mut ProcessOwnership,
        expected: &Value,
    ) -> AdapterResult<()> {
        if self.verify_git(request, ownership).await? != *expected {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "native workspace replaced during session initialization",
            ));
        }
        Ok(())
    }

    pub async fn verify_git(
        &self,
        request: &LaunchRequest,
        ownership: &mut ProcessOwnership,
    ) -> AdapterResult<Value> {
        let project = self.project.clone();
        let workspace = request.worktree.clone();
        let before = filesystem(move || canonical_binding(&project.root, &workspace)).await?;
        let executable = filesystem(|| resolve_executable("git")).await?;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        let mut observe = |cwd: PathBuf, args: Vec<String>| {
            let flag = ownership.group();
            let executable = executable.clone();
            async move {
                bounded_git(
                    &executable,
                    &cwd,
                    &args,
                    crate::git::native_environment(),
                    deadline,
                    flag,
                )
                .await
            }
        };
        let args = |args: &[&str]| args.iter().map(|s| (*s).to_owned()).collect();
        observe(
            self.project.root.clone(),
            args(&["check-ref-format", "--branch", &self.project.base_branch]),
        )
        .await?;
        let top = observe(
            self.project.root.clone(),
            args(&["rev-parse", "--show-toplevel"]),
        )
        .await?;
        let git_dir = observe(
            self.project.root.clone(),
            args(&["rev-parse", "--path-format=absolute", "--git-dir"]),
        )
        .await?;
        let common = observe(
            self.project.root.clone(),
            args(&["rev-parse", "--path-format=absolute", "--git-common-dir"]),
        )
        .await?;
        let roots = observe(
            self.project.root.clone(),
            vec![
                "rev-list".into(),
                "--max-parents=0".into(),
                format!("refs/heads/{}", self.project.base_branch),
            ],
        )
        .await?;
        let revision = observe(
            request.worktree.clone(),
            args(&["rev-parse", "--verify", "HEAD^{commit}"]),
        )
        .await?;
        if revision != request.input.revision {
            return Err(failure(
                ErrorKind::StateConflict,
                "native prepared revision differs from observed HEAD",
            ));
        }
        if let Some(task) = &self.task {
            let task_top = observe(
                request.worktree.clone(),
                args(&["rev-parse", "--show-toplevel"]),
            )
            .await?;
            let task_common = observe(
                request.worktree.clone(),
                args(&["rev-parse", "--path-format=absolute", "--git-common-dir"]),
            )
            .await?;
            let branch = observe(
                request.worktree.clone(),
                args(&["symbolic-ref", "--quiet", "--short", "HEAD"]),
            )
            .await?;
            let facts = WorktreeOwnershipFacts {
                source_top: top.into(),
                source_git_dir: git_dir.into(),
                source_common: common.into(),
                source_roots: roots.lines().map(str::to_owned).collect(),
                task_top: task_top.into(),
                task_common: task_common.into(),
                branch,
                revision,
            };
            let project = self.project.clone();
            let task = task.clone();
            filesystem(move || {
                crate::git::validate_worktree_ownership(&project, &task, facts)
                    .map(|_| ())
                    .map_err(|e| failure(ErrorKind::OwnershipMismatch, e.to_string()))
            })
            .await?;
        } else {
            let project = self.project.clone();
            filesystem(move || {
                let common = PathBuf::from(common).canonicalize().map_err(|_| {
                    failure(
                        ErrorKind::OwnershipMismatch,
                        "native common directory unavailable",
                    )
                })?;
                let mut roots = roots.lines().map(str::to_owned).collect::<Vec<_>>();
                roots.sort();
                if PathBuf::from(top).canonicalize().ok().as_ref() != Some(&project.root)
                    || PathBuf::from(git_dir).canonicalize().ok().as_ref() != Some(&common)
                    || roots.is_empty()
                    || serde_json::to_string(&(common, roots)).map_err(|_| {
                        failure(
                            ErrorKind::StateFailure,
                            "native Git identity serialization failed",
                        )
                    })? != project.repository_identity
                {
                    return Err(failure(
                        ErrorKind::OwnershipMismatch,
                        "native primary repository identity changed",
                    ));
                }
                Ok(())
            })
            .await?;
        }
        if request.role == SessionRole::Reviewer {
            let status = observe(
                request.worktree.clone(),
                args(&[
                    "status",
                    "--porcelain=v1",
                    "--untracked-files=all",
                    "--ignored=matching",
                ]),
            )
            .await?;
            if !status.is_empty() {
                return Err(failure(
                    ErrorKind::Locked,
                    "native immutable review worktree is dirty",
                ));
            }
        }
        let project = self.project.clone();
        let workspace = request.worktree.clone();
        let after = filesystem(move || canonical_binding(&project.root, &workspace)).await?;
        if before != after {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "native root/worktree inode identity changed",
            ));
        }
        Ok(before)
    }
}

/// Retained permits bound abandoned blocking work even after timeout/cancellation.
pub(super) async fn filesystem<T: Send + 'static>(
    action: impl FnOnce() -> AdapterResult<T> + Send + 'static,
) -> AdapterResult<T> {
    let permit = FILESYSTEM_WORKERS
        .try_acquire()
        .map_err(|_| failure(ErrorKind::Timeout, "native filesystem worker limit reached"))?;
    tokio::time::timeout(
        Duration::from_secs(5),
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            action()
        }),
    )
    .await
    .map_err(|_| failure(ErrorKind::Timeout, "native filesystem preflight timed out"))?
    .map_err(|_| failure(ErrorKind::ProcessFailure, "native filesystem worker failed"))?
}

fn canonical_binding(root: &Path, workspace: &Path) -> AdapterResult<Value> {
    use std::os::unix::fs::MetadataExt;
    let identity = |path: &Path| -> AdapterResult<Value> {
        let canonical = path
            .canonicalize()
            .map_err(|_| failure(ErrorKind::OwnershipMismatch, "native workspace unavailable"))?;
        let metadata = path.symlink_metadata().map_err(|_| {
            failure(
                ErrorKind::OwnershipMismatch,
                "native workspace metadata unavailable",
            )
        })?;
        if canonical != path || !metadata.is_dir() {
            return Err(failure(
                ErrorKind::OwnershipMismatch,
                "native workspace must be an exact canonical directory",
            ));
        }
        Ok(serde_json::json!([
            canonical,
            metadata.dev(),
            metadata.ino()
        ]))
    };
    Ok(serde_json::json!([identity(root)?, identity(workspace)?]))
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::{
        adapter::{LaunchMode, PreparedInput},
        domain::{CompletionCriterion, Scope},
        git::WorktreeManager,
        state::Store,
    };
    use std::{collections::BTreeMap, process::Command, sync::Mutex};

    pub(in crate::codex) struct Fixture {
        _temp: tempfile::TempDir,
        pub(in crate::codex) store: SharedStore,
        pub(in crate::codex) request: LaunchRequest,
    }
    fn git(root: &Path, args: &[&str]) -> String {
        let output = Command::new("/usr/bin/git")
            .current_dir(root)
            .args(args)
            .env_clear()
            .envs(crate::git::native_environment())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }
    impl Fixture {
        pub(in crate::codex) fn new(task_scoped: bool) -> Self {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().canonicalize().unwrap().join("project");
            std::fs::create_dir(&root).unwrap();
            git(&root, &["init", "-b", "main"]);
            std::fs::write(root.join(".gitignore"), "worktree/\n").unwrap();
            git(&root, &["add", ".gitignore"]);
            git(
                &root,
                &[
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "commit",
                    "-m",
                    "fixture",
                ],
            );
            let mut project = Project::new(
                "fixture".into(),
                root.clone(),
                crate::git::repository_identity(&root, "main").unwrap(),
                "main".into(),
            );
            let mut store = Store::open(&temp.path().join("state.sqlite3")).unwrap();
            store.put_project(&mut project).unwrap();
            let (scope, worktree, role) = if task_scoped {
                let mut goal = Goal::new(
                    project.id,
                    "fixture".into(),
                    vec![CompletionCriterion {
                        id: "fixture".into(),
                        description: "native scope proof".into(),
                        evidence: None,
                        satisfied: false,
                    }],
                );
                store.put_goal(&mut goal).unwrap();
                let mut task = Task::new(project.id, goal.id, "fixture".into(), "codex".into());
                task.issue = Some(42);
                store.put_task(&mut task).unwrap();
                let worktree = WorktreeManager::create(&mut store, task.id)
                    .unwrap()
                    .worktree;
                (task.scope(), worktree, SessionRole::Executor)
            } else {
                (Scope::project(project.id), root, SessionRole::Consultant)
            };
            let revision = git(&worktree, &["rev-parse", "HEAD"]);
            let request = LaunchRequest {
                project,
                scope: scope.clone(),
                worktree,
                role,
                mode: LaunchMode::NonInteractive,
                input: PreparedInput {
                    scope,
                    kind: InputKind::ContextPack,
                    revision,
                    version: 1,
                    source_versions: BTreeMap::from([("rules".into(), "v1".into())]),
                    payload: "only prepared context".into(),
                },
                environment: BTreeMap::new(),
                model: None,
                effort: None,
            };
            Self {
                _temp: temp,
                store: Arc::new(Mutex::new(store)),
                request,
            }
        }
        fn review(&mut self) {
            WorktreeManager::lock_review(
                &mut self.store.lock().unwrap(),
                self.request.scope.task_id.unwrap(),
                &self.request.input.revision,
                "immutable test",
            )
            .unwrap();
            self.request.role = SessionRole::Reviewer;
            self.request.input.kind = InputKind::ReviewBundle;
        }
    }
    #[tokio::test]
    async fn primary_consultation_needs_no_worktree_namespace_and_keeps_exact_git_identity() {
        let mut fixture = Fixture::new(false);
        assert!(!fixture.request.project.worktree_root.exists());
        let snapshot = ScopeSnapshot::capture(&fixture.store, &fixture.request, "codex").unwrap();
        snapshot
            .verify_git(&fixture.request, &mut ProcessOwnership::default())
            .await
            .unwrap();
        fixture.request.worktree = fixture.request.project.root.join("subdirectory");
        std::fs::create_dir(&fixture.request.worktree).unwrap();
        assert!(
            matches!(ScopeSnapshot::capture(&fixture.store,&fixture.request,"codex"),Err(error) if error.kind==ErrorKind::OwnershipMismatch)
        );
    }
    #[tokio::test]
    async fn a_valid_same_head_worktree_replacement_cannot_reuse_the_initial_binding() {
        let fixture = Fixture::new(true);
        let snapshot = ScopeSnapshot::capture(&fixture.store, &fixture.request, "codex").unwrap();
        let mut ownership = ProcessOwnership::default();
        let binding = snapshot
            .verify_git(&fixture.request, &mut ownership)
            .await
            .unwrap();
        snapshot
            .verify_binding(&fixture.request, &mut ownership, &binding)
            .await
            .unwrap();
        let original = fixture.request.worktree.with_extension("original");
        std::fs::rename(&fixture.request.worktree, &original).unwrap();
        std::fs::create_dir(&fixture.request.worktree).unwrap();
        for name in [".git", ".gitignore"] {
            std::fs::copy(original.join(name), fixture.request.worktree.join(name)).unwrap();
        }
        // Path, Git common directory, native registration and HEAD still match;
        // only the previously authenticated directory inode changed.
        snapshot
            .verify_git(&fixture.request, &mut ownership)
            .await
            .unwrap();
        assert_eq!(
            snapshot
                .verify_binding(&fixture.request, &mut ownership, &binding)
                .await
                .unwrap_err()
                .kind,
            ErrorKind::OwnershipMismatch
        );
    }

    #[tokio::test]
    async fn task_branch_common_directory_and_exact_prepared_head_are_required() {
        let mut fixture = Fixture::new(true);
        let snapshot = ScopeSnapshot::capture(&fixture.store, &fixture.request, "codex").unwrap();
        let mut ownership = ProcessOwnership::default();
        snapshot
            .verify_git(&fixture.request, &mut ownership)
            .await
            .unwrap();
        assert!(!ownership.uncertain());
        fixture.request.input.revision = "0".repeat(40);
        assert_eq!(
            snapshot
                .verify_git(&fixture.request, &mut ownership)
                .await
                .unwrap_err()
                .kind,
            ErrorKind::StateConflict
        );
        fixture.request.input.revision = git(&fixture.request.worktree, &["rev-parse", "HEAD"]);
        git(
            &fixture.request.worktree,
            &["switch", "-c", "feature/foreign-binding"],
        );
        assert_eq!(
            snapshot
                .verify_git(&fixture.request, &mut ownership)
                .await
                .unwrap_err()
                .kind,
            ErrorKind::OwnershipMismatch
        );
    }
    #[tokio::test]
    async fn immutable_review_rejects_untracked_and_ignored_changes_and_unlock_relock_aba() {
        let mut fixture = Fixture::new(true);
        fixture.request.role = SessionRole::Reviewer;
        fixture.request.input.kind = InputKind::ReviewBundle;
        assert!(
            matches!(ScopeSnapshot::capture(&fixture.store,&fixture.request,"codex"),Err(error) if error.kind==ErrorKind::Locked)
        );
        fixture.review();
        let snapshot = ScopeSnapshot::capture(&fixture.store, &fixture.request, "codex").unwrap();
        snapshot
            .verify_git(&fixture.request, &mut ProcessOwnership::default())
            .await
            .unwrap();
        std::fs::write(
            fixture.request.worktree.join("untracked"),
            "changed after lock",
        )
        .unwrap();
        assert_eq!(
            snapshot
                .verify_git(&fixture.request, &mut ProcessOwnership::default())
                .await
                .unwrap_err()
                .kind,
            ErrorKind::Locked
        );
        std::fs::remove_file(fixture.request.worktree.join("untracked")).unwrap();
        std::fs::create_dir(fixture.request.worktree.join("worktree")).unwrap();
        std::fs::write(
            fixture.request.worktree.join("worktree/ignored"),
            "ignored change",
        )
        .unwrap();
        assert_eq!(
            snapshot
                .verify_git(&fixture.request, &mut ProcessOwnership::default())
                .await
                .unwrap_err()
                .kind,
            ErrorKind::Locked
        );
        std::fs::remove_dir_all(fixture.request.worktree.join("worktree")).unwrap();
        let id = snapshot
            .locks
            .iter()
            .find(|record| record.data["active"] == true)
            .unwrap()
            .id;
        WorktreeManager::unlock_review(&mut fixture.store.lock().unwrap(), id).unwrap();
        fixture.review();
        assert_eq!(
            snapshot
                .recheck(&fixture.store, &fixture.request, "codex")
                .unwrap_err()
                .kind,
            ErrorKind::StateConflict
        );
    }
    #[test]
    fn persisted_authority_and_native_inputs_cannot_be_forged_or_silently_reused() {
        let mut fixture = Fixture::new(true);
        let snapshot = ScopeSnapshot::capture(&fixture.store, &fixture.request, "codex").unwrap();
        fixture
            .request
            .project
            .environment_refs
            .push("FOREIGN_SECRET".into());
        assert!(
            matches!(ScopeSnapshot::capture(&fixture.store,&fixture.request,"codex"),Err(error) if error.kind==ErrorKind::StateConflict)
        );
        fixture.request.project = snapshot.project.clone();
        fixture.request.input.scope = Scope::project(fixture.request.project.id);
        assert!(
            matches!(ScopeSnapshot::capture(&fixture.store,&fixture.request,"codex"),Err(error) if error.kind==ErrorKind::OwnershipMismatch)
        );
        fixture.request.input.scope = fixture.request.scope.clone();
        let mut store = fixture.store.lock().unwrap();
        let mut task = store
            .task(fixture.request.scope.task_id.unwrap())
            .unwrap()
            .unwrap();
        task.title.push_str(" changed");
        store.put_task(&mut task).unwrap();
        drop(store);
        assert_eq!(
            snapshot
                .recheck(&fixture.store, &fixture.request, "codex")
                .unwrap_err()
                .kind,
            ErrorKind::StateConflict
        );
        fixture.request.input.revision = "main".into();
        assert!(
            matches!(ScopeSnapshot::capture(&fixture.store,&fixture.request,"codex"),Err(error) if error.kind==ErrorKind::InvalidInput)
        );
    }
    #[test]
    fn independently_reaped_git_child_cannot_clear_live_native_server_uncertainty() {
        let mut ownership = ProcessOwnership::default();
        let native = ownership.group();
        native.store(true, Ordering::SeqCst);
        let git = ownership.group();
        git.store(true, Ordering::SeqCst);
        git.store(false, Ordering::SeqCst);
        assert!(ownership.uncertain());
        native.store(false, Ordering::SeqCst);
        assert!(!ownership.uncertain());
    }
}
