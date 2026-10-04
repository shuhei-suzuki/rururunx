//! Owned synthetic Grok fixture; no installed agent, config, credential or model access.
use super::receipt_support::{Attempt, Observation};
use super::*;

pub(super) struct Fixture {
    pub(super) directory: tempfile::TempDir,
    pub(super) request: LaunchRequest,
    pub(super) store: SharedStore,
    pub(super) executable: std::path::PathBuf,
    attempts: Mutex<BTreeMap<SessionId, Attempt>>,
    observations: Mutex<BTreeMap<SessionId, Observation>>,
}
pub(super) fn git(root: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .env_clear()
        .envs(std::env::vars_os().filter(|(key, _)| !key.to_string_lossy().starts_with("GIT_")))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
impl Fixture {
    pub(super) fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("project");
        std::fs::create_dir(&root).unwrap();
        git(&root, &["init", "-b", "main"]);
        std::fs::write(root.join("own.txt"), "owned baseline\n").unwrap();
        git(&root, &["add", "own.txt"]);
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
        let worktree = root.join("worktree/task");
        git(
            &root,
            &[
                "worktree",
                "add",
                "-b",
                "feature/task",
                worktree.to_str().unwrap(),
            ],
        );
        let root = root.canonicalize().unwrap();
        let worktree = worktree.canonicalize().unwrap();
        let mut project = Project::new(
            "fixture".into(),
            root.clone(),
            fixture_git::repository_identity(&root, "main").unwrap(),
            "main".into(),
        );
        let mut goal = Goal::new(
            project.id,
            "native".into(),
            vec![CompletionCriterion {
                id: "fixture".into(),
                description: "owned ACP edit".into(),
                satisfied: false,
                evidence: None,
            }],
        );
        let mut task = Task::new(project.id, goal.id, "native".into(), "grok".into());
        task.worktree = Some(worktree.clone());
        task.branch = Some("feature/task".into());
        let database = directory.path().join("state.db");
        let mut store = Store::open(&database).unwrap();
        store.put_project(&mut project).unwrap();
        store.put_goal(&mut goal).unwrap();
        store.put_task(&mut task).unwrap();
        let request = LaunchRequest {
            project,
            scope: task.scope(),
            worktree: worktree.clone(),
            role: SessionRole::Executor,
            mode: LaunchMode::NonInteractive,
            input: PreparedInput {
                scope: task.scope(),
                kind: InputKind::ContextPack,
                revision: git(&worktree, &["rev-parse", "HEAD"]),
                version: 1,
                source_versions: BTreeMap::from([("fixture".into(), "v1".into())]),
                payload: "prepared owned fixture".into(),
            },
            environment: BTreeMap::from([
                ("RRX_DATABASE".into(), database.to_str().unwrap().into()),
                (
                    "RRX_FOREIGN".into(),
                    directory
                        .path()
                        .join("foreign.txt")
                        .to_str()
                        .unwrap()
                        .into(),
                ),
            ]),
            model: Some("requested-model".into()),
            effort: Some("low".into()),
        };
        let executable = directory.path().join("fake-grok");
        std::fs::write(&executable, include_str!("grok_fake.py")).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            directory,
            request,
            store: Arc::new(Mutex::new(store)),
            executable,
            attempts: Mutex::new(BTreeMap::new()),
            observations: Mutex::new(BTreeMap::new()),
        }
    }
    pub(super) fn adapter(&self) -> GrokAdapter {
        GrokAdapter::new("grok".into(), self.executable.clone(), self.store.clone()).unwrap()
    }
    pub(super) fn mode(&mut self, mode: &str) {
        if mode == "unowned_read" {
            std::fs::write(
                self.request.worktree.join("unseen.txt"),
                "unseen scoped baseline\n",
            )
            .unwrap();
        }
        self.request
            .environment
            .insert("RRX_MODE".into(), mode.into());
    }
}

impl Fixture {
    pub(super) async fn start(&self, adapter: &dyn AgentAdapter) -> AdapterResult<Session> {
        let lower = receipt_support::watermark(&self.store, &self.request.scope)
            .expect("before-launch audit unavailable");
        let session = adapter.start(self.request.clone()).await?;
        self.attempts.lock().unwrap().insert(
            session.id,
            Attempt {
                lower,
                input_version: self.request.input.version,
            },
        );
        Ok(session)
    }
    pub(super) async fn start_structured(
        &self,
        adapter: &GrokAdapter,
        schema: Value,
    ) -> AdapterResult<Session> {
        let lower = receipt_support::watermark(&self.store, &self.request.scope)
            .expect("before-launch audit unavailable");
        let session = adapter
            .start_structured(self.request.clone(), schema)
            .await?;
        self.attempts.lock().unwrap().insert(
            session.id,
            Attempt {
                lower,
                input_version: self.request.input.version,
            },
        );
        Ok(session)
    }
    pub(super) async fn start_structured_trait(
        &self,
        adapter: &dyn AgentAdapter,
        schema: Value,
    ) -> AdapterResult<Session> {
        let lower = receipt_support::watermark(&self.store, &self.request.scope)
            .expect("before-launch audit unavailable");
        let session = adapter
            .start_structured(self.request.clone(), schema)
            .await?;
        self.attempts.lock().unwrap().insert(
            session.id,
            Attempt {
                lower,
                input_version: self.request.input.version,
            },
        );
        Ok(session)
    }
    pub(super) async fn resume(
        &self,
        adapter: &dyn AgentAdapter,
        session: SessionRef,
        input_version: u64,
    ) -> AdapterResult<Session> {
        let lower = receipt_support::watermark(&self.store, &self.request.scope)
            .expect("before-launch audit unavailable");
        let resumed = adapter.resume(session).await?;
        self.attempts.lock().unwrap().insert(
            resumed.id,
            Attempt {
                lower,
                input_version,
            },
        );
        Ok(resumed)
    }
    pub(super) fn observe(&self, status: &SessionStatus) {
        let attempt = self
            .attempts
            .lock()
            .unwrap()
            .get(&status.session.id)
            .copied();
        let observation = receipt_support::observe(&self.store, &status.session, attempt);
        self.observations
            .lock()
            .unwrap()
            .insert(status.session.id, observation);
    }
    pub(super) fn observation(&self, status: &SessionStatus) -> Observation {
        self.observations
            .lock()
            .ok()
            .and_then(|observations| observations.get(&status.session.id).cloned())
            .unwrap_or(Observation {
                receipt: Err("terminal_window_unavailable"),
                events: vec![],
                saved: None,
                attempt: None,
            })
    }
    pub(super) fn receipt_message(&self, status: &SessionStatus) -> String {
        receipt_support::message(&self.observation(status).receipt)
    }
}
