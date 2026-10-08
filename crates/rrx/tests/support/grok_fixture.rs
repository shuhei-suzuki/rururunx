//! Owned synthetic Grok fixture; no installed agent, config, credential or model access.
use super::receipt_support::{Attempt, Observation};
use super::*;

pub(super) struct Fixture {
    pub(super) directory: super::legacy_support::Holder,
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
        Self::with_workflow(WorkflowClass::default())
    }
    /// FM §8.1 L: legacy rows from accepted ingress through the ordered
    /// historical migration; the worktree and branch are the genuine
    /// `WorktreeManager::create` names on that legacy Task.
    pub(super) fn with_workflow(workflow: WorkflowClass) -> Self {
        let risk = match workflow {
            WorkflowClass::Quick => RiskClass::R0,
            WorkflowClass::Standard => RiskClass::R1,
            WorkflowClass::Strict => RiskClass::R3,
        };
        let (directory, store, tasks) = super::legacy_support::blocking(
            "project",
            "state.db",
            |root| {
                std::fs::write(root.join("own.txt"), "owned baseline\n").unwrap();
                git(root, &["add", "own.txt"]);
                git(
                    root,
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
            },
            vec![("native", "grok", workflow, risk)],
        );
        let (project, task, worktree) = {
            let mut store = store.lock().unwrap();
            assert_eq!(tasks[0].workflow, workflow, "SETUP: accepted class");
            let project = store.project(tasks[0].project_id).unwrap().unwrap();
            let worktree = fixture_git::WorktreeManager::create(&mut store, tasks[0].id)
                .unwrap()
                .worktree;
            let task = store.task(tasks[0].id).unwrap().unwrap();
            (project, task, worktree)
        };
        let database = directory.path().join("state.db");
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
            environment: BTreeMap::new(),
            model: Some("requested-model".into()),
            effort: Some("low".into()),
        };
        let executable = directory.path().join("fake-grok");
        std::fs::write(executable.with_extension("json"), serde_json::to_vec(&json!({"RRX_DATABASE":database,"RRX_FOREIGN":directory.path().join("foreign.txt")})).unwrap()).unwrap();
        std::fs::write(&executable, include_str!("grok_fake.py")).unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            directory,
            request,
            store,
            executable,
            attempts: Mutex::new(BTreeMap::new()),
            observations: Mutex::new(BTreeMap::new()),
        }
    }
    pub(super) fn adapter(&self) -> GrokAdapter {
        GrokAdapter::new("grok".into(), self.executable.clone(), self.store.clone()).unwrap()
    }
    pub(super) fn synthetic(&self, key: &str, value: String) {
        assert!(key.starts_with("RRX_"));
        let path = self.executable.with_extension("json");
        let mut metadata: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        metadata[key] = json!(value);
        use std::io::Write;
        let mut replacement = tempfile::NamedTempFile::new_in(path.parent().unwrap()).unwrap();
        replacement
            .write_all(&serde_json::to_vec(&metadata).unwrap())
            .unwrap();
        replacement.persist(path).unwrap();
    }
    pub(super) fn synthetic_value(&self, key: &str) -> String {
        let metadata: Value =
            serde_json::from_slice(&std::fs::read(self.executable.with_extension("json")).unwrap())
                .unwrap();
        metadata[key].as_str().unwrap().to_owned()
    }
    pub(super) fn mode(&mut self, mode: &str) {
        if mode == "unowned_read" {
            std::fs::write(
                self.request.worktree.join("unseen.txt"),
                "unseen scoped baseline\n",
            )
            .unwrap();
        }
        self.synthetic("RRX_MODE", mode.into());
    }
}

#[test]
fn synthetic_sidecar_stays_complete_for_actual_concurrent_reader() {
    // Scheduling determines whether reads overlap replacements. This is a
    // probabilistic control, not proof that any particular replacement was read.
    let fixture = Fixture::new();
    let barrier = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        let reader = scope.spawn(|| {
            barrier.wait();
            for _ in 0..2000 {
                let bytes = std::fs::read(fixture.executable.with_extension("json")).unwrap();
                let metadata: Value = serde_json::from_slice(&bytes)
                    .expect("atomic sidecar reader observed incomplete JSON");
                assert!(metadata["RRX_DATABASE"].is_string());
                assert!(metadata["RRX_FOREIGN"].is_string());
                std::thread::yield_now();
            }
        });
        barrier.wait();
        for version in 0..200 {
            fixture.synthetic("RRX_CONCURRENT", format!("{version}:{}", "x".repeat(4096)));
        }
        reader.join().unwrap();
    });
    assert!(
        fixture
            .synthetic_value("RRX_CONCURRENT")
            .starts_with("199:")
    );
}

impl Fixture {
    pub(super) async fn start(&self, adapter: &dyn AgentAdapter) -> AdapterResult<Session> {
        let lower = receipt_support::watermark(&self.store, &self.request.scope)
            .expect("before-launch audit unavailable");
        if self.request.role == SessionRole::Reviewer {
            self.synthetic("RRX_EXPECT_SCHEMA", json!({"type":"object"}).to_string());
        }
        let session = adapter.start(self.request.clone()).await?;
        self.record_attempt(&session, lower, self.request.input.version);
        Ok(session)
    }
    pub(super) async fn start_structured(
        &self,
        adapter: &GrokAdapter,
        schema: Value,
    ) -> AdapterResult<Session> {
        let lower = receipt_support::watermark(&self.store, &self.request.scope)
            .expect("before-launch audit unavailable");
        self.synthetic("RRX_EXPECT_SCHEMA", schema.to_string());
        let session = adapter
            .start_structured(self.request.clone(), schema)
            .await?;
        self.record_attempt(&session, lower, self.request.input.version);
        Ok(session)
    }
    pub(super) fn record_attempt(&self, session: &Session, lower: i64, input_version: u64) {
        self.attempts.lock().unwrap().insert(
            session.id,
            Attempt {
                lower,
                input_version,
            },
        );
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
