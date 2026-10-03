use rrx::{
    adapter::*,
    config::{Config, WorkflowClass},
    domain::*,
    state::{Store, WorkflowAccess},
    workflow::*,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::{Notify, watch};

fn adapter_error(message: &str) -> AdapterError {
    AdapterError {
        kind: ErrorKind::UnsupportedCapability,
        message: message.into(),
    }
}
struct FakeAgent {
    name: String,
    store: SharedStore,
    review: bool,
    fail: AtomicBool,
    launches: Mutex<Vec<LaunchRequest>>,
    statuses: Mutex<BTreeMap<SessionId, SessionStatus>>,
}
impl FakeAgent {
    fn new(name: &str, store: SharedStore, review: bool) -> Self {
        Self {
            name: name.into(),
            store,
            review,
            fail: AtomicBool::new(false),
            launches: Mutex::new(vec![]),
            statuses: Mutex::new(BTreeMap::new()),
        }
    }
}
impl AgentAdapter for FakeAgent {
    fn capabilities(&self) -> BTreeSet<Capability> {
        BTreeSet::from([
            if self.review {
                Capability::Review
            } else {
                Capability::Execute
            },
            Capability::NonInteractive,
        ])
    }
    fn probe(&self) -> AdapterResult<AgentInfo> {
        Ok(AgentInfo {
            agent: self.name.clone(),
            provider: "fake".into(),
            adapter_version: "1".into(),
            executable: "fake".into(),
            authenticated: None,
            model_configuration: false,
            effort_configuration: false,
            capabilities: self.capabilities(),
        })
    }
    fn start(&self, request: LaunchRequest) -> AdapterFuture<'_, Session> {
        Box::pin(async move {
            self.launches.lock().unwrap().push(request.clone());
            let session = Session {
                id: SessionId::new(),
                scope: request.scope,
                agent: self.name.clone(),
                provider: "fake".into(),
                role: request.role,
                native_ref: None,
                pid: None,
                worktree: request.worktree,
                state: SessionState::Running,
                model: None,
                effort: None,
                recovery: Value::Null,
                started_at: now_ms(),
            };
            self.store
                .lock()
                .unwrap()
                .put_session(&session, 0)
                .map_err(|e| adapter_error(&e.to_string()))?;
            self.statuses.lock().unwrap().insert(
                session.id,
                SessionStatus {
                    session: session.clone(),
                    exit_code: None,
                    stdout: vec![],
                    stderr: vec![],
                    stdout_truncated: false,
                    stderr_truncated: false,
                    failure: None,
                },
            );
            Ok(session)
        })
    }
    fn status(&self, reference: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        Box::pin(async move {
            let mut statuses = self.statuses.lock().unwrap();
            let mut status = statuses
                .get(&reference.id)
                .cloned()
                .ok_or_else(|| adapter_error("unknown session"))?;
            assert_eq!(status.session.scope, reference.scope);
            if status.session.state == SessionState::Running {
                status.session.state = if self.fail.load(Ordering::SeqCst) {
                    SessionState::Lost
                } else {
                    SessionState::Exited
                };
                status.exit_code = if status.session.state == SessionState::Exited {
                    Some(0)
                } else {
                    None
                };
                let mut store = self.store.lock().unwrap();
                let version = store.session(reference.id).unwrap().unwrap().1;
                store.put_session(&status.session, version).unwrap();
                statuses.insert(reference.id, status.clone());
            }
            Ok(status)
        })
    }
    fn stop(&self, _: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        Box::pin(async { Err(adapter_error("stop")) })
    }
    fn attach(&self, _: SessionRef) -> AdapterFuture<'_, ()> {
        Box::pin(async { Err(adapter_error("attach")) })
    }
    fn resume(&self, _: SessionRef) -> AdapterFuture<'_, Session> {
        Box::pin(async { Err(adapter_error("resume")) })
    }
    fn release(&self, _: SessionRef) -> AdapterResult<()> {
        Ok(())
    }
    fn subscribe(&self, reference: SessionRef) -> AdapterResult<watch::Receiver<SessionStatus>> {
        let (_, receiver) = watch::channel(
            self.statuses
                .lock()
                .unwrap()
                .get(&reference.id)
                .unwrap()
                .clone(),
        );
        Ok(receiver)
    }
    fn usage(&self, _: SessionRef, _: String, _: Option<u32>) -> AdapterFuture<'_, Usage> {
        Box::pin(async { Err(adapter_error("usage")) })
    }
}
struct Sources {
    snapshot: Mutex<SourceSnapshot>,
    on_capture: Mutex<Option<Box<dyn FnOnce() + Send>>>,
}
impl Sources {
    fn new(scope: Scope) -> Self {
        Self {
            snapshot: Mutex::new(SourceSnapshot {
                scope,
                revision: "head-1".into(),
                source_versions: BTreeMap::from([("requirements".into(), "sha256-1".into())]),
                payload: "factual fixture context".into(),
            }),
            on_capture: Mutex::new(None),
        }
    }
}
impl WorkflowSources for Sources {
    fn capture(
        &self,
        _: Project,
        _: Task,
        _: Phase,
        _: ContextBudget,
    ) -> WorkflowFuture<'_, SourceSnapshot> {
        Box::pin(async move {
            if let Some(hook) = self.on_capture.lock().unwrap().take() {
                hook();
            }
            Ok(self.snapshot.lock().unwrap().clone())
        })
    }
}
struct Gates {
    approved: AtomicBool,
    waiting: AtomicBool,
    calls: Mutex<Vec<PhaseInvocation>>,
    hold: Mutex<Option<Arc<Notify>>>,
    entered: Notify,
}
impl Gates {
    fn new() -> Self {
        Self {
            approved: AtomicBool::new(true),
            waiting: AtomicBool::new(false),
            calls: Mutex::new(vec![]),
            hold: Mutex::new(None),
            entered: Notify::new(),
        }
    }
}
impl PhaseGates for Gates {
    fn complete(
        &self,
        invocation: PhaseInvocation,
        status: Option<SessionStatus>,
    ) -> WorkflowFuture<'_, GateOutcome> {
        Box::pin(async move {
            self.calls.lock().unwrap().push(invocation.clone());
            let hold = self.hold.lock().unwrap().clone();
            if let Some(hold) = hold {
                self.entered.notify_one();
                hold.notified().await;
            }
            if self.waiting.load(Ordering::SeqCst) {
                return Ok(GateOutcome::Waiting(
                    "test evidence deliberately withheld".into(),
                ));
            }
            Ok(GateOutcome::Passed(Evidence {
                scope: invocation.task.scope(),
                phase: invocation.phase,
                revision: invocation.sources.revision,
                source_versions: invocation.sources.source_versions,
                artifacts: vec![format!("fixture://{}", invocation.phase.key())],
                review_approved: if invocation.phase.actor() == Actor::Reviewer {
                    Some(self.approved.load(Ordering::SeqCst))
                } else {
                    None
                },
                session_id: status.map(|s| s.session.id),
            }))
        })
    }
}
struct Fixture {
    dir: tempfile::TempDir,
    store: SharedStore,
    project: Project,
    task: Task,
    engine: Arc<WorkflowEngine>,
    executor: Arc<FakeAgent>,
    reviewer: Arc<FakeAgent>,
    sources: Arc<Sources>,
    gates: Arc<Gates>,
    config: Config,
}
impl Fixture {
    fn new(class: WorkflowClass) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let mut store = Store::open(&dir.path().join("state.db")).unwrap();
        let mut project = Project::new(
            "fixture".into(),
            root,
            "fixture-identity".into(),
            "main".into(),
        );
        store.put_project(&mut project).unwrap();
        let mut goal = Goal::new(
            project.id,
            "fixture objective".into(),
            vec![CompletionCriterion {
                id: "workflow".into(),
                description: "workflow evidence recorded".into(),
                evidence: None,
                satisfied: false,
            }],
        );
        store.put_goal(&mut goal).unwrap();
        let mut task = Task::new(
            project.id,
            goal.id,
            "fixture task".into(),
            "executor".into(),
        );
        task.workflow = class;
        task.risk = RiskClass::R0;
        task.reviewers = vec!["reviewer".into()];
        task.worktree = Some(project.worktree_root.join("task"));
        task.branch = Some("feature/task".into());
        store.put_task(&mut task).unwrap();
        let store = Arc::new(Mutex::new(store));
        let executor = Arc::new(FakeAgent::new("executor", store.clone(), false));
        let reviewer = Arc::new(FakeAgent::new("reviewer", store.clone(), true));
        let mut registry = AgentRegistry::default();
        registry
            .register("executor".into(), executor.clone())
            .unwrap();
        registry
            .register("reviewer".into(), reviewer.clone())
            .unwrap();
        let mut config = Config {
            minimum_workflow: WorkflowClass::Quick,
            ..Config::default()
        };
        config.workflow.default = WorkflowClass::Quick;
        let sources = Arc::new(Sources::new(task.scope()));
        let gates = Arc::new(Gates::new());
        let engine = Arc::new(
            WorkflowEngine::new(
                store.clone(),
                Arc::new(registry),
                config.clone(),
                sources.clone(),
                gates.clone(),
            )
            .unwrap(),
        );
        Self {
            dir,
            store,
            project,
            task,
            engine,
            executor,
            reviewer,
            sources,
            gates,
            config,
        }
    }
    async fn finish(&self) {
        for _ in 0..100 {
            match self
                .engine
                .step(self.task.id, BTreeMap::new())
                .await
                .unwrap()
            {
                StepResult::Finished => return,
                StepResult::Completed { .. }
                | StepResult::Started { .. }
                | StepResult::Running { .. } => {}
                other => panic!("unexpected workflow result {other:?}"),
            }
        }
        panic!("workflow never finished");
    }
    async fn through(&self, target: Phase) {
        for _ in 0..100 {
            if matches!(self.engine.step(self.task.id,BTreeMap::new()).await.unwrap(),StepResult::Completed{phase} if phase==target)
            {
                return;
            }
        }
        panic!("phase not reached");
    }
}

#[tokio::test]
async fn all_presets_drive_real_adapter_calls_and_persist_phase_context_history() {
    for class in [
        WorkflowClass::Quick,
        WorkflowClass::Standard,
        WorkflowClass::Strict,
    ] {
        let fixture = Fixture::new(class);
        let initial = fixture
            .engine
            .initialize(fixture.task.id, None)
            .await
            .unwrap();
        assert_eq!(initial.context_version, 1);
        fixture.finish().await;
        let workflow = fixture.engine.snapshot(fixture.task.id).unwrap();
        let expected = phases(class, &fixture.config);
        assert_eq!(
            workflow.history.iter().map(|a| a.phase).collect::<Vec<_>>(),
            expected
        );
        assert!(
            workflow
                .history
                .iter()
                .all(|a| a.state == AttemptState::Succeeded)
        );
        assert!(
            workflow
                .history
                .windows(2)
                .all(|a| a[0].context_version < a[1].context_version)
        );
        let mut launched = fixture.executor.launches.lock().unwrap().clone();
        launched.extend(fixture.reviewer.launches.lock().unwrap().clone());
        assert_eq!(
            launched.len(),
            expected
                .iter()
                .filter(|p| p.actor() != Actor::EvidencePort)
                .count()
        );
        for request in launched {
            assert_eq!(request.scope, fixture.task.scope());
            let context = fixture
                .store
                .lock()
                .unwrap()
                .context(&request.scope, Some(request.input.version))
                .unwrap()
                .unwrap();
            assert_eq!(request.input.source_versions, context.source_hashes);
            assert_eq!(request.input.revision, context.revision);
            assert_eq!(
                request.input.kind,
                if request.role == SessionRole::Reviewer {
                    InputKind::ReviewBundle
                } else {
                    InputKind::ContextPack
                }
            );
            assert_eq!(
                context.data["budget"]["class"],
                json!(match class {
                    WorkflowClass::Quick => BudgetClass::Small,
                    WorkflowClass::Standard => BudgetClass::Normal,
                    WorkflowClass::Strict => BudgetClass::Broad,
                })
            );
        }
        let restored = Store::open(&fixture.dir.path().join("state.db")).unwrap();
        let saved = restored
            .records(&fixture.task.scope(), RecordKind::Workflow)
            .unwrap();
        let saved: WorkflowSnapshot = serde_json::from_value(saved[0].data.clone()).unwrap();
        assert!(saved.finished);
        assert_eq!(saved.history.len(), expected.len());
        assert_eq!(
            restored
                .task(fixture.task.id)
                .unwrap()
                .unwrap()
                .context_version,
            saved.context_version
        );
        assert_eq!(
            restored
                .context(&fixture.task.scope(), None)
                .unwrap()
                .unwrap()
                .version,
            saved.context_version
        );
        let events = restored.events(&fixture.task.scope(), 0, 1000).unwrap();
        assert_eq!(
            events.iter().filter(|e| e.kind == "workflow.saved").count(),
            saved.history.len() * 3
                + fixture.executor.launches.lock().unwrap().len()
                + fixture.reviewer.launches.lock().unwrap().len()
                + 1
        );
    }
}
#[tokio::test]
async fn escalation_restarts_mandatory_prerequisites_without_erasing_history_or_downgrading() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Implement).await;
    let before = fixture.engine.snapshot(fixture.task.id).unwrap();
    let standard = fixture
        .engine
        .escalate(
            fixture.task.id,
            RiskClass::R2,
            None,
            "shared module".into(),
            "symbol consumers".into(),
        )
        .await
        .unwrap();
    assert_eq!(standard.workflow, WorkflowClass::Standard);
    assert_eq!(standard.generation, 2);
    assert!(standard.completed.is_empty());
    assert_eq!(standard.history.len(), before.history.len());
    assert!(standard.context_version > before.context_version);
    fixture.through(Phase::DesignReview).await;
    let strict = fixture
        .engine
        .escalate(
            fixture.task.id,
            RiskClass::R3,
            None,
            "schema boundary".into(),
            "schema diff".into(),
        )
        .await
        .unwrap();
    assert_eq!(strict.workflow, WorkflowClass::Strict);
    assert_eq!(strict.generation, 3);
    let retained = fixture
        .engine
        .escalate(
            fixture.task.id,
            RiskClass::R0,
            Some(WorkflowClass::Quick),
            "lower suggestion".into(),
            "user input".into(),
        )
        .await
        .unwrap();
    assert_eq!(retained.workflow, WorkflowClass::Strict);
    assert_eq!(retained.risk, RiskClass::R3);
    fixture.finish().await;
    assert_eq!(
        fixture
            .engine
            .snapshot(fixture.task.id)
            .unwrap()
            .escalations
            .len(),
        2
    );
}
#[tokio::test]
async fn stale_revision_invalidates_evidence_and_waiting_requires_explicit_retry() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Implement).await;
    fixture.sources.snapshot.lock().unwrap().revision = "head-2".into();
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Invalidated { .. }
    ));
    let workflow = fixture.engine.snapshot(fixture.task.id).unwrap();
    assert!(workflow.completed.is_empty());
    assert_eq!(workflow.generation, 2);
    fixture.gates.waiting.store(true, Ordering::SeqCst);
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Waiting { .. }
    ));
    let calls = fixture.gates.calls.lock().unwrap().len();
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Waiting { .. }
    ));
    assert_eq!(fixture.gates.calls.lock().unwrap().len(), calls);
    fixture
        .engine
        .retry(fixture.task.id, "evidence now available".into())
        .unwrap();
    fixture.gates.waiting.store(false, Ordering::SeqCst);
    fixture.finish().await;
}
#[tokio::test]
async fn exit_zero_without_review_approval_cannot_satisfy_review_gate() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Tests).await;
    fixture.gates.approved.store(false, Ordering::SeqCst);
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Started {
            phase: Phase::ImplementationReview,
            ..
        }
    ));
    let error = fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap_err();
    assert!(error.to_string().contains("explicit approved verdict"));
    assert!(
        !fixture
            .engine
            .snapshot(fixture.task.id)
            .unwrap()
            .completed
            .contains_key(&Phase::ImplementationReview)
    );
}
#[tokio::test]
async fn lost_native_executor_fences_explicit_retry_and_other_phase_dispatch() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    fixture.executor.fail.store(true, Ordering::SeqCst);
    fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap();
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Failed { .. }
    ));
    assert!(
        fixture
            .engine
            .retry(fixture.task.id, "retry".into())
            .unwrap_err()
            .to_string()
            .contains("verified recovery")
    );
    assert_eq!(fixture.executor.launches.lock().unwrap().len(), 1);
}
#[tokio::test]
async fn source_scope_and_concurrent_task_changes_fail_closed_without_orphan_context() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture.sources.snapshot.lock().unwrap().scope.project_id = ProjectId::new();
    assert!(
        fixture
            .engine
            .initialize(fixture.task.id, None)
            .await
            .is_err()
    );
    assert!(
        fixture
            .store
            .lock()
            .unwrap()
            .context(&fixture.task.scope(), None)
            .unwrap()
            .is_none()
    );
    fixture.sources.snapshot.lock().unwrap().scope = fixture.task.scope();
    let store = fixture.store.clone();
    let id = fixture.task.id;
    *fixture.sources.on_capture.lock().unwrap() = Some(Box::new(move || {
        let mut store = store.lock().unwrap();
        let mut task = store.task(id).unwrap().unwrap();
        task.next_action = Some("concurrent update".into());
        store.put_task(&mut task).unwrap();
    }));
    let error = fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap_err();
    assert!(
        error
            .downcast_ref::<rrx::state::StateGuardError>()
            .is_some()
    );
    let store = fixture.store.lock().unwrap();
    assert_eq!(
        store.task(id).unwrap().unwrap().next_action.as_deref(),
        Some("concurrent update")
    );
    assert!(
        store
            .context(&fixture.task.scope(), None)
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .records(&fixture.task.scope(), RecordKind::Workflow)
            .unwrap()
            .is_empty()
    );
}
#[tokio::test]
async fn blocked_project_and_paused_goal_cannot_progress_or_publish_context() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    let version = fixture
        .engine
        .snapshot(fixture.task.id)
        .unwrap()
        .context_version;
    let mut project = fixture.project.clone();
    project.state = ProjectState::Blocked;
    project.blocked_reason = Some("missing source".into());
    fixture
        .store
        .lock()
        .unwrap()
        .put_project(&mut project)
        .unwrap();
    assert!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .is_err()
    );
    assert_eq!(
        fixture
            .engine
            .snapshot(fixture.task.id)
            .unwrap()
            .context_version,
        version
    );
    project.state = ProjectState::Registered;
    project.blocked_reason = None;
    fixture
        .store
        .lock()
        .unwrap()
        .put_project(&mut project)
        .unwrap();
    {
        let mut store = fixture.store.lock().unwrap();
        let mut goal = store.goal(fixture.task.goal_id).unwrap().unwrap();
        goal.state = GoalState::Paused;
        store.put_goal(&mut goal).unwrap();
    }
    assert!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .is_err()
    );
    assert!(fixture.executor.launches.lock().unwrap().is_empty());
}
#[tokio::test]
async fn existing_live_executor_and_review_lock_reject_mutating_phase_before_evidence_port() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    let mut lock = Record::new(
        fixture.task.scope(),
        RecordKind::WorktreeLock,
        json!({"active":true,"revision":"head-1","worktree":fixture.task.worktree,"branch":fixture.task.branch,"reason":"immutable review"}),
    );
    fixture.store.lock().unwrap().put_record(&mut lock).unwrap();
    assert!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .is_err()
    );
    assert!(fixture.gates.calls.lock().unwrap().is_empty());
    {
        let mut store = fixture.store.lock().unwrap();
        lock.data["active"] = json!(false);
        store.put_record(&mut lock).unwrap();
    }
    let request = LaunchRequest {
        project: fixture.project.clone(),
        scope: fixture.task.scope(),
        worktree: fixture.task.worktree.clone().unwrap(),
        role: SessionRole::Executor,
        mode: LaunchMode::NonInteractive,
        input: PreparedInput {
            scope: fixture.task.scope(),
            kind: InputKind::ContextPack,
            revision: "head-1".into(),
            version: 1,
            source_versions: BTreeMap::new(),
            payload: "test".into(),
        },
        environment: BTreeMap::new(),
        model: None,
        effort: None,
    };
    fixture.executor.start(request).await.unwrap();
    assert!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .is_err()
    );
    assert!(fixture.gates.calls.lock().unwrap().is_empty());
}
#[tokio::test]
async fn second_engine_cannot_dispatch_duplicate_phase_and_interrupt_does_not_auto_retry() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    let a = fixture.engine.step(fixture.task.id, BTreeMap::new());
    let b = fixture.engine.step(fixture.task.id, BTreeMap::new());
    let (a, b) = tokio::join!(a, b);
    assert!(a.is_ok() || b.is_ok());
    assert_eq!(fixture.executor.launches.lock().unwrap().len(), 1);
    // Persist an interrupted external phase reservation: no native handle means
    // recovery owns it; repeated steps must not dispatch the gate again.
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    {
        let mut store = fixture.store.lock().unwrap();
        let mut task = store.task(fixture.task.id).unwrap().unwrap();
        let mut record = store
            .records(&task.scope(), RecordKind::Workflow)
            .unwrap()
            .remove(0);
        let mut wf: WorkflowSnapshot = serde_json::from_value(record.data.clone()).unwrap();
        wf.active = Some(0);
        wf.history.push(PhaseAttempt {
            phase: Phase::Worktree,
            generation: 1,
            context_version: 1,
            budget: ContextBudget {
                class: BudgetClass::Small,
                discretionary_tokens: 1,
            },
            state: AttemptState::Running,
            session_id: None,
            started_at: now_ms(),
            completed_at: None,
            detail: None,
        });
        record.data = serde_json::to_value(wf).unwrap();
        store
            .put_workflow_transition(
                &mut task,
                &mut record,
                None,
                fixture.project.version,
                1,
                WorkflowAccess::StateOnly,
            )
            .unwrap();
    }
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Waiting { .. }
    ));
    assert!(fixture.gates.calls.lock().unwrap().is_empty());
    assert!(
        fixture
            .engine
            .retry(fixture.task.id, "retry".into())
            .is_err()
    );
}
#[tokio::test]
async fn project_minimum_and_rule_refresh_override_defaults_and_force_new_generation() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    let root = &fixture.project.root;
    for args in [
        vec!["init", "-b", "main"],
        vec!["config", "user.name", "Fixture"],
        vec!["config", "user.email", "fixture@example.invalid"],
        vec!["commit", "--allow-empty", "-m", "fixture"],
    ] {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap();
        assert!(status.success());
    }
    std::fs::write(
        root.join("policy.toml"),
        "minimum_workflow = 'STANDARD'\n[context]\nrepo_map_tokens = 321\n",
    )
    .unwrap();
    std::fs::write(root.join("rules.md"), "Mandatory: keep safety controls.").unwrap();
    let mut project = fixture.project.clone();
    project.config_ref = Some(root.join("policy.toml"));
    project.rule_refs = vec![root.join("rules.md")];
    fixture
        .store
        .lock()
        .unwrap()
        .put_project(&mut project)
        .unwrap();
    let workflow = fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    assert_eq!(workflow.workflow, WorkflowClass::Standard);
    let context = fixture
        .store
        .lock()
        .unwrap()
        .context(&fixture.task.scope(), None)
        .unwrap()
        .unwrap();
    assert!(
        context.data["payload"]
            .as_str()
            .unwrap()
            .contains("keep safety controls")
    );
    assert_eq!(context.data["budget"]["discretionary_tokens"], 321);
    fixture.through(Phase::Worktree).await;
    std::fs::write(
        root.join("rules.md"),
        "Mandatory: keep safety controls and new production gate.",
    )
    .unwrap();
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Invalidated { .. }
    ));
    assert!(
        fixture
            .engine
            .snapshot(fixture.task.id)
            .unwrap()
            .completed
            .is_empty()
    );
    std::fs::write(root.join("policy.toml"),"minimum_workflow = 'STRICT'\n[workflow]\nbrowser_verification = true\nstaging_verification = true\n").unwrap();
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Invalidated { .. }
    ));
    let wf = fixture.engine.snapshot(fixture.task.id).unwrap();
    assert_eq!(wf.workflow, WorkflowClass::Strict);
    assert!(wf.configured_phases.contains(&Phase::Browser));
    assert!(wf.configured_phases.contains(&Phase::Staging));
    std::fs::write(
        root.join("policy.toml"),
        "minimum_workflow = 'QUICK'\n[workflow]\ndefault = 'QUICK'\n",
    )
    .unwrap();
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Invalidated { .. }
    ));
    assert_eq!(
        fixture.engine.snapshot(fixture.task.id).unwrap().workflow,
        WorkflowClass::Strict
    );
}
#[test]
fn ordered_format_migration_preserves_v2_state_and_unknown_future_is_rejected() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    let db = fixture.dir.path().join("state.db");
    let connection = rusqlite::Connection::open(&db).unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
    drop(connection);
    let restored = Store::open(&db).unwrap();
    assert_eq!(restored.schema_version().unwrap(), 3);
    assert_eq!(
        restored.task(fixture.task.id).unwrap().unwrap().scope(),
        fixture.task.scope()
    );
    drop(restored);
    let connection = rusqlite::Connection::open(&db).unwrap();
    connection.pragma_update(None, "user_version", 4).unwrap();
    drop(connection);
    assert!(Store::open(&db).is_err());
}

#[tokio::test]
async fn concurrent_native_polls_claim_evidence_evaluation_before_calling_external_gate() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap();
    let hold = Arc::new(Notify::new());
    *fixture.gates.hold.lock().unwrap() = Some(hold.clone());
    let engine = fixture.engine.clone();
    let id = fixture.task.id;
    let poll = tokio::spawn(async move { engine.step(id, BTreeMap::new()).await });
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        fixture.gates.entered.notified(),
    )
    .await
    .unwrap();
    let calls = fixture.gates.calls.lock().unwrap().len();
    assert!(matches!(
        fixture.engine.step(id, BTreeMap::new()).await.unwrap(),
        StepResult::Waiting {
            phase: Phase::Implement,
            ..
        }
    ));
    assert_eq!(fixture.gates.calls.lock().unwrap().len(), calls);
    hold.notify_one();
    assert!(matches!(
        poll.await.unwrap().unwrap(),
        StepResult::Completed {
            phase: Phase::Implement
        }
    ));
}
#[tokio::test]
async fn revision_changes_during_review_gate_preserve_reservation_and_never_accept_stale_verdict() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Tests).await;
    fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap();
    let hold = Arc::new(Notify::new());
    *fixture.gates.hold.lock().unwrap() = Some(hold.clone());
    let engine = fixture.engine.clone();
    let id = fixture.task.id;
    let poll = tokio::spawn(async move { engine.step(id, BTreeMap::new()).await });
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        fixture.gates.entered.notified(),
    )
    .await
    .unwrap();
    fixture.sources.snapshot.lock().unwrap().revision = "new-review-target".into();
    hold.notify_one();
    assert!(
        poll.await
            .unwrap()
            .unwrap_err()
            .to_string()
            .contains("sources changed")
    );
    let wf = fixture.engine.snapshot(id).unwrap();
    assert!(!wf.completed.contains_key(&Phase::ImplementationReview));
    assert_eq!(
        wf.history[wf.active.unwrap()].state,
        AttemptState::Evaluating
    );
    assert!(fixture.engine.retry(id, "unverified retry".into()).is_err());
}
#[tokio::test]
async fn record_cas_failure_rolls_back_context_task_and_all_transition_audit() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    let mut store = fixture.store.lock().unwrap();
    let mut task = store.task(fixture.task.id).unwrap().unwrap();
    let original_version = task.version;
    let mut record = store
        .records(&task.scope(), RecordKind::Workflow)
        .unwrap()
        .remove(0);
    record.version += 1;
    let mut context = store.context(&task.scope(), None).unwrap().unwrap();
    context.version += 1;
    task.context_version = context.version;
    let mut wf: WorkflowSnapshot = serde_json::from_value(record.data.clone()).unwrap();
    wf.context_version = context.version;
    record.data = serde_json::to_value(wf).unwrap();
    let events = store.events(&task.scope(), 0, 1000).unwrap().len();
    assert!(
        store
            .put_workflow_transition(
                &mut task,
                &mut record,
                Some(&context),
                fixture.project.version,
                1,
                WorkflowAccess::StateOnly
            )
            .is_err()
    );
    assert_eq!(task.version, original_version);
    assert_eq!(store.task(task.id).unwrap().unwrap().context_version, 1);
    assert_eq!(
        store.context(&task.scope(), None).unwrap().unwrap().version,
        1
    );
    assert_eq!(store.events(&task.scope(), 0, 1000).unwrap().len(), events);
}
#[tokio::test]
async fn workflow_authority_and_decision_history_require_atomic_monotonic_updates() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Implement).await;
    let mut store = fixture.store.lock().unwrap();
    let mut task = store.task(fixture.task.id).unwrap().unwrap();
    let mut record = store
        .records(&task.scope(), RecordKind::Workflow)
        .unwrap()
        .remove(0);
    assert!(store.put_record(&mut record).is_err());
    let mut wf: WorkflowSnapshot = serde_json::from_value(record.data.clone()).unwrap();
    wf.history.clear();
    record.data = serde_json::to_value(wf).unwrap();
    assert!(
        store
            .put_workflow_transition(
                &mut task,
                &mut record,
                None,
                fixture.project.version,
                1,
                WorkflowAccess::StateOnly
            )
            .unwrap_err()
            .to_string()
            .contains("history truncation")
    );
}
#[tokio::test]
async fn explicit_stricter_selection_and_unsupported_review_capability_are_honest() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    assert_eq!(
        fixture
            .engine
            .initialize(fixture.task.id, Some(WorkflowClass::Strict))
            .await
            .unwrap()
            .workflow,
        WorkflowClass::Strict
    );
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Tests).await;
    {
        let mut store = fixture.store.lock().unwrap();
        let mut task = store.task(fixture.task.id).unwrap().unwrap();
        task.reviewers = vec!["executor".into()];
        store.put_task(&mut task).unwrap();
    }
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Failed {
            phase: Phase::ImplementationReview,
            ..
        }
    ));
    assert!(fixture.reviewer.launches.lock().unwrap().is_empty());
    assert_eq!(fixture.executor.launches.lock().unwrap().len(), 1);
}
