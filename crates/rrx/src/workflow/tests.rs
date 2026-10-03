use crate::{
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
        atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
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
    native_mode: AtomicBool,
    native_completions: Mutex<BTreeSet<SessionId>>,
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
            native_mode: AtomicBool::new(false),
            native_completions: Mutex::new(BTreeSet::new()),
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
    fn transport_succeeded(&self, status: &SessionStatus) -> bool {
        if status.session.state != SessionState::Exited || status.failure.is_some() {
            return false;
        }
        if status.exit_code == Some(0) {
            return true;
        }
        self.native_completions
            .lock()
            .unwrap()
            .contains(&status.session.id)
            && self
                .statuses
                .lock()
                .unwrap()
                .get(&status.session.id)
                .is_some_and(|saved| {
                    serde_json::to_value(&saved.session).unwrap()
                        == serde_json::to_value(&status.session).unwrap()
                        && saved.exit_code == status.exit_code
                        && saved.failure == status.failure
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
                status.exit_code = if status.session.state == SessionState::Exited
                    && !self.native_mode.load(Ordering::SeqCst)
                {
                    Some(0)
                } else {
                    None
                };
                let mut store = self.store.lock().unwrap();
                let version = store.session(reference.id).unwrap().unwrap().1;
                store.put_session(&status.session, version).unwrap();
                statuses.insert(reference.id, status.clone());
                if self.native_mode.load(Ordering::SeqCst)
                    && status.session.state == SessionState::Exited
                {
                    self.native_completions.lock().unwrap().insert(reference.id);
                }
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
type CaptureHook = Box<dyn FnOnce() + Send>;
struct Sources {
    snapshot: Mutex<SourceSnapshot>,
    on_capture: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    captures: AtomicUsize,
    on_numbered_capture: Mutex<Option<(usize, CaptureHook)>>,
    capture_error: AtomicBool,
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
            captures: AtomicUsize::new(0),
            on_numbered_capture: Mutex::new(None),
            capture_error: AtomicBool::new(false),
        }
    }
}
impl WorkflowSources for Sources {
    fn capture(
        &self,
        _: Project,
        _: Task,
        phase: Phase,
        budget: ContextBudget,
    ) -> WorkflowFuture<'_, SourceSnapshot> {
        Box::pin(async move {
            let number = self.captures.fetch_add(1, Ordering::SeqCst) + 1;
            if self
                .on_numbered_capture
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|(target, _)| *target == number)
            {
                let (_, hook) = self.on_numbered_capture.lock().unwrap().take().unwrap();
                hook();
            }
            anyhow::ensure!(
                !self.capture_error.load(Ordering::SeqCst),
                "fixture capture unavailable"
            );
            if let Some(hook) = self.on_capture.lock().unwrap().take() {
                hook();
            }
            let mut snapshot = self.snapshot.lock().unwrap().clone();
            snapshot.payload.push_str(&format!(
                " selected-phase={} selected-class={:?} selected-tokens={}",
                phase.key(),
                budget.class,
                budget.discretionary_tokens
            ));
            Ok(snapshot)
        })
    }
}
type CompleteHook = Box<dyn FnOnce(&PhaseInvocation) -> Option<SourceSnapshot> + Send>;
struct Gates {
    approved: AtomicBool,
    waiting: AtomicBool,
    calls: Mutex<Vec<PhaseInvocation>>,
    hold: Mutex<Option<Arc<Notify>>>,
    entered: Notify,
    corrupt: AtomicU8,
    on_complete: Mutex<Option<CompleteHook>>,
    unknown: AtomicBool,
}
impl Gates {
    fn new() -> Self {
        Self {
            approved: AtomicBool::new(true),
            waiting: AtomicBool::new(false),
            calls: Mutex::new(vec![]),
            hold: Mutex::new(None),
            entered: Notify::new(),
            corrupt: AtomicU8::new(0),
            on_complete: Mutex::new(None),
            unknown: AtomicBool::new(false),
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
            if self.unknown.load(Ordering::SeqCst) {
                anyhow::bail!("fixture outcome unknown");
            }
            let replacement = self
                .on_complete
                .lock()
                .unwrap()
                .take()
                .and_then(|hook| hook(&invocation));
            let mut invocation = invocation;
            if let Some(source) = replacement {
                invocation.sources = source;
            }
            if self.waiting.load(Ordering::SeqCst) {
                return Ok(GateOutcome::Waiting(
                    "test evidence deliberately withheld".into(),
                ));
            }
            let mut evidence = Evidence {
                scope: invocation.task.scope(),
                phase: invocation.phase,
                revision: invocation.sources.revision,
                dependencies: invocation.sources.source_versions.clone(),
                source_versions: invocation.sources.source_versions,
                artifacts: vec![format!("fixture://{}", invocation.phase.key())],
                review_approved: if invocation.phase.actor() == Actor::Reviewer {
                    Some(self.approved.load(Ordering::SeqCst))
                } else {
                    None
                },
                session_id: status.map(|s| s.session.id),
                context_version: invocation.context.version,
            };
            match self.corrupt.load(Ordering::SeqCst) {
                1 => evidence.scope.project_id = ProjectId::new(),
                2 => evidence.revision = "stale-target".into(),
                3 => evidence.source_versions.clear(),
                4 => evidence.context_version = 0,
                5 => evidence.session_id = Some(SessionId::new()),
                6 => evidence.artifacts.clear(),
                _ => {}
            }
            Ok(GateOutcome::Passed(evidence))
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
        for version in 1..=workflow.context_version {
            let context = fixture
                .store
                .lock()
                .unwrap()
                .context(&fixture.task.scope(), Some(version))
                .unwrap()
                .unwrap();
            let phase: Phase = serde_json::from_value(context.data["phase"].clone()).unwrap();
            let budget: ContextBudget =
                serde_json::from_value(context.data["budget"].clone()).unwrap();
            let payload = context.data["payload"].as_str().unwrap();
            assert!(
                payload.contains(&format!(
                    "selected-phase={} selected-class={:?} selected-tokens={}",
                    phase.key(),
                    budget.class,
                    budget.discretionary_tokens
                )),
                "pack selection disagrees with phase/budget: {context:?}"
            );
        }
        let restored = Store::open(&fixture.dir.path().join("state.db")).unwrap();
        let saved = restored
            .records(&fixture.task.scope(), RecordKind::Workflow)
            .unwrap();
        let authority_version = saved[0].version;
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
            events
                .iter()
                .filter(|event| event.kind == "workflow.saved")
                .count() as u64,
            authority_version
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| event.kind == "workflow.gate_observed")
                .count(),
            saved.history.len()
        );
        assert!(
            saved
                .history
                .iter()
                .all(|attempt| attempt.observations.len() == 1
                    && attempt.dispatch_started == (attempt.phase.actor() != Actor::EvidencePort))
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
    let result = fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap();
    assert!(
        matches!(result, StepResult::Failed { reason, .. } if reason.contains("explicit approved verdict"))
    );
    let failed = fixture.engine.snapshot(fixture.task.id).unwrap();
    assert_eq!(
        failed.history[failed.active.unwrap()].state,
        AttemptState::Failed
    );
    fixture
        .engine
        .retry(fixture.task.id, "review remediation complete".into())
        .unwrap();
    fixture.gates.approved.store(true, Ordering::SeqCst);
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
            .downcast_ref::<crate::state::StateGuardError>()
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
            dispatch_started: false,
            observations: vec![],
            claimed_observations: 0,
            agent: None,
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
        let mut wf: WorkflowSnapshot = serde_json::from_value(record.data.clone()).unwrap();
        wf.history[0].state = AttemptState::Evaluating;
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
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            fixture.engine.step(id, BTreeMap::new())
        )
        .await
        .expect("concurrent poll must not enter already reserved gate")
        .unwrap(),
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
    assert!(matches!(
        poll.await.unwrap().unwrap(),
        StepResult::Invalidated { .. }
    ));
    let wf = fixture.engine.snapshot(id).unwrap();
    assert!(wf.completed.is_empty());
    assert!(wf.active.is_none());
    assert_eq!(wf.generation, 2);
    assert!(
        wf.invalidations
            .last()
            .unwrap()
            .cause
            .contains("sources changed")
    );
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

#[tokio::test]
async fn atomic_context_pointer_rejects_wrong_revision_phase_or_source_versions() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    let mut store = fixture.store.lock().unwrap();
    let original_task = store.task(fixture.task.id).unwrap().unwrap();
    let original_record = store
        .records(&fixture.task.scope(), RecordKind::Workflow)
        .unwrap()
        .remove(0);
    for mutation in 0..3 {
        let mut task = original_task.clone();
        let mut record = original_record.clone();
        let mut context = store.context(&task.scope(), None).unwrap().unwrap();
        context.version += 1;
        task.context_version = context.version;
        record.data["context_version"] = json!(context.version);
        match mutation {
            0 => context.revision = "foreign revision".into(),
            1 => {
                context
                    .source_hashes
                    .insert("workflow:phase".into(), "pr".into());
            }
            _ => {
                context
                    .source_hashes
                    .insert("requirements".into(), "stale hash".into());
            }
        }
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
        assert_eq!(
            store.context(&task.scope(), None).unwrap().unwrap().version,
            1
        );
        assert_eq!(
            store.task(task.id).unwrap().unwrap().version,
            original_task.version
        );
    }
}

#[tokio::test]
async fn running_attempt_preserves_native_actor_when_task_reviewer_choice_changes() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Tests).await;
    let StepResult::Started {
        session: Some(session),
        ..
    } = fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap()
    else {
        panic!("review not started");
    };
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
        StepResult::Completed {
            phase: Phase::ImplementationReview
        }
    ));
    let wf = fixture.engine.snapshot(fixture.task.id).unwrap();
    let review = wf
        .history
        .iter()
        .find(|a| a.phase == Phase::ImplementationReview)
        .unwrap();
    assert_eq!(review.agent.as_deref(), Some("reviewer"));
    assert_eq!(review.session_id, Some(session));
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .task(fixture.task.id)
            .unwrap()
            .unwrap()
            .reviewers,
        ["executor"]
    );
}
#[tokio::test]
async fn lost_reviewer_also_requires_verified_recovery_before_retry() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Tests).await;
    fixture.reviewer.fail.store(true, Ordering::SeqCst);
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
        StepResult::Failed {
            phase: Phase::ImplementationReview,
            ..
        }
    ));
    assert!(
        fixture
            .engine
            .retry(fixture.task.id, "retry".into())
            .is_err()
    );
    assert_eq!(fixture.reviewer.launches.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn foreign_stale_unbound_or_empty_gate_evidence_never_advances_review() {
    for corruption in 1..=6 {
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
        fixture.gates.corrupt.store(corruption, Ordering::SeqCst);
        assert!(
            fixture
                .engine
                .step(fixture.task.id, BTreeMap::new())
                .await
                .is_ok_and(|result| matches!(result, StepResult::Failed { .. })),
            "gate corruption {corruption} accepted"
        );
        let wf = fixture.engine.snapshot(fixture.task.id).unwrap();
        assert!(!wf.completed.contains_key(&Phase::ImplementationReview));
        assert_eq!(wf.history[wf.active.unwrap()].state, AttemptState::Failed);
        fixture
            .engine
            .retry(fixture.task.id, "corrected evidence".into())
            .unwrap();
    }
}

#[tokio::test]
async fn native_turn_completion_preserves_real_exit_and_still_requires_gate_evidence() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    fixture.executor.native_mode.store(true, Ordering::SeqCst);
    fixture.gates.waiting.store(true, Ordering::SeqCst);
    let StepResult::Started {
        session: Some(id), ..
    } = fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap()
    else {
        panic!("agent not started");
    };
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Waiting {
            phase: Phase::Implement,
            ..
        }
    ));
    let status = fixture
        .executor
        .statuses
        .lock()
        .unwrap()
        .get(&id)
        .unwrap()
        .clone();
    assert_eq!(status.exit_code, None);
    assert_eq!(status.session.state, SessionState::Exited);
    assert!(fixture.executor.transport_succeeded(&status));
    let mut unowned = status.clone();
    unowned.session.scope.project_id = ProjectId::new();
    assert!(!fixture.executor.transport_succeeded(&unowned));
    let mut metadata_claim = status.clone();
    metadata_claim.session.recovery = json!({"native_turn":{"status":"completed"}});
    fixture.executor.native_completions.lock().unwrap().clear();
    assert!(!fixture.executor.transport_succeeded(&metadata_claim));
    assert!(
        !fixture
            .engine
            .snapshot(fixture.task.id)
            .unwrap()
            .completed
            .contains_key(&Phase::Implement)
    );
}

#[tokio::test]
async fn generic_completion_default_requires_real_zero_exit_and_no_failure() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Implement).await;
    let mut status = fixture
        .executor
        .statuses
        .lock()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .clone();
    let generic =
        GenericCliAdapter::new("generic".into(), vec!["true".into()], fixture.store.clone())
            .unwrap();
    assert!(generic.transport_succeeded(&status));
    status.failure = Some("I/O failure".into());
    assert!(!generic.transport_succeeded(&status));
    status.failure = None;
    status.exit_code = None;
    status.session.recovery = json!({"native_turn":{"status":"completed"}});
    assert!(!generic.transport_succeeded(&status));
    status.exit_code = Some(1);
    assert!(!generic.transport_succeeded(&status));
    status.exit_code = Some(0);
    status.session.state = SessionState::Lost;
    assert!(!generic.transport_succeeded(&status));
}

#[tokio::test]
async fn completed_attempt_rewrite_and_atomic_owner_version_or_activity_changes_are_rejected() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Implement).await;
    let mut store = fixture.store.lock().unwrap();
    let task = store.task(fixture.task.id).unwrap().unwrap();
    let record = store
        .records(&task.scope(), RecordKind::Workflow)
        .unwrap()
        .remove(0);
    let mut changed_task = task.clone();
    let mut changed_record = record.clone();
    changed_record.data["history"][0]["detail"] = json!("rewritten prior result");
    assert!(
        store
            .put_workflow_transition(
                &mut changed_task,
                &mut changed_record,
                None,
                fixture.project.version,
                1,
                WorkflowAccess::StateOnly
            )
            .unwrap_err()
            .to_string()
            .contains("completed phase history")
    );
    for (project_version, goal_version) in [
        (fixture.project.version + 1, 1),
        (fixture.project.version, 2),
    ] {
        let mut changed_task = task.clone();
        let mut changed_record = record.clone();
        assert!(
            store
                .put_workflow_transition(
                    &mut changed_task,
                    &mut changed_record,
                    None,
                    project_version,
                    goal_version,
                    WorkflowAccess::StateOnly
                )
                .is_err()
        );
    }
    let mut project = fixture.project.clone();
    project.state = ProjectState::Blocked;
    project.blocked_reason = Some("fixture inactive".into());
    store.put_project(&mut project).unwrap();
    let mut changed_task = task;
    let mut changed_record = record;
    assert!(
        store
            .put_workflow_transition(
                &mut changed_task,
                &mut changed_record,
                None,
                project.version,
                1,
                WorkflowAccess::StateOnly
            )
            .is_err()
    );
}

#[tokio::test]
async fn default_is_fallback_not_a_floor_and_finished_workflow_cannot_reopen() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    let mut config = fixture.config.clone();
    config.workflow.default = WorkflowClass::Standard;
    let mut registry = AgentRegistry::default();
    registry
        .register("executor".into(), fixture.executor.clone())
        .unwrap();
    registry
        .register("reviewer".into(), fixture.reviewer.clone())
        .unwrap();
    let engine = WorkflowEngine::new(
        fixture.store.clone(),
        Arc::new(registry),
        config,
        fixture.sources.clone(),
        fixture.gates.clone(),
    )
    .unwrap();
    assert_eq!(
        engine
            .initialize(fixture.task.id, None)
            .await
            .unwrap()
            .workflow,
        WorkflowClass::Quick
    );
    fixture.finish().await;
    assert!(
        engine
            .escalate(
                fixture.task.id,
                RiskClass::R3,
                None,
                "after PR".into(),
                "fixture".into()
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("finished workflow")
    );
}
#[tokio::test]
async fn gate_owner_updates_preserve_issue_binding_goal_metadata_and_task_blockers() {
    let fixture = Fixture::new(WorkflowClass::Standard);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    // Exercise an actual Issue port Task write and a concurrent Goal update.
    let store = fixture.store.clone();
    *fixture.gates.on_complete.lock().unwrap() = Some(Box::new(move |invocation| {
        assert_eq!(invocation.phase, Phase::Issue);
        let mut store = store.lock().unwrap();
        let mut task = store.task(invocation.task.id).unwrap().unwrap();
        task.issue = Some(42);
        task.next_action = Some("external next action".into());
        task.blockers.push("unrelated blocker".into());
        store.put_task(&mut task).unwrap();
        let mut goal = store.goal(task.goal_id).unwrap().unwrap();
        goal.constraints.push("concurrent constraint".into());
        store.put_goal(&mut goal).unwrap();
        None
    }));
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Completed {
            phase: Phase::Issue
        }
    ));
    let task = fixture
        .store
        .lock()
        .unwrap()
        .task(fixture.task.id)
        .unwrap()
        .unwrap();
    assert_eq!(task.issue, Some(42));
    assert_eq!(task.next_action.as_deref(), Some("external next action"));
    assert_eq!(task.blockers, ["unrelated blocker"]);
    // Worktree port can bind a previously unbound Task through the normal Store.
    let fixture = Fixture::new(WorkflowClass::Quick);
    let mut task = fixture.task.clone();
    task.worktree = None;
    task.branch = None;
    // Existing assigned bindings are immutable, so use a fresh unbound Task.
    task.id = TaskId::new();
    task.version = 0;
    fixture.store.lock().unwrap().put_task(&mut task).unwrap();
    let sources = Arc::new(Sources::new(task.scope()));
    let gates = Arc::new(Gates::new());
    let store = fixture.store.clone();
    let path = fixture.project.worktree_root.join("fresh-task");
    let assigned = path.clone();
    *gates.on_complete.lock().unwrap() = Some(Box::new(move |invocation| {
        let mut store = store.lock().unwrap();
        let mut task = store.task(invocation.task.id).unwrap().unwrap();
        task.worktree = Some(assigned);
        task.branch = Some("feature/fresh-task".into());
        store.put_task(&mut task).unwrap();
        None
    }));
    let engine = WorkflowEngine::new(
        fixture.store.clone(),
        Arc::new(AgentRegistry::default()),
        fixture.config,
        sources,
        gates,
    )
    .unwrap();
    engine.initialize(task.id, None).await.unwrap();
    assert!(matches!(
        engine.step(task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Completed {
            phase: Phase::Worktree
        }
    ));
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .task(task.id)
            .unwrap()
            .unwrap()
            .worktree,
        Some(path)
    );
}
#[tokio::test]
async fn commit_publishes_new_head_before_tests_review_and_pr_without_restart() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Implement).await;
    let sources = fixture.sources.clone();
    *fixture.gates.on_complete.lock().unwrap() = Some(Box::new(move |invocation| {
        assert_eq!(invocation.phase, Phase::Commit);
        sources.snapshot.lock().unwrap().revision = "committed-head".into();
        Some(sources.snapshot.lock().unwrap().clone())
    }));
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Completed {
            phase: Phase::Commit
        }
    ));
    fixture.finish().await;
    let workflow = fixture.engine.snapshot(fixture.task.id).unwrap();
    assert_eq!(workflow.generation, 1);
    for phase in [Phase::Tests, Phase::ImplementationReview, Phase::Pr] {
        assert_eq!(workflow.completed[&phase].revision, "committed-head");
    }
    for class in [WorkflowClass::Standard, WorkflowClass::Strict] {
        let list = phases(class, &fixture.config);
        for (commit, review) in [
            (Phase::RequirementsCommit, Phase::RequirementsReview),
            (Phase::DesignCommit, Phase::DesignReview),
            (Phase::Commit, Phase::Tests),
        ] {
            assert!(
                list.iter().position(|p| *p == commit).unwrap()
                    < list.iter().position(|p| *p == review).unwrap()
            );
        }
    }
}
#[tokio::test]
async fn changed_approved_artifact_invalidates_formal_reviews_and_known_drift_never_calls_gate() {
    let fixture = Fixture::new(WorkflowClass::Standard);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::DesignReview).await;
    fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap(); // Implement
    fixture
        .sources
        .snapshot
        .lock()
        .unwrap()
        .source_versions
        .insert("requirements".into(), "changed-approved-artifact".into());
    assert!(
        matches!(fixture.engine.step(fixture.task.id, BTreeMap::new()).await.unwrap(), StepResult::Invalidated { reason } if reason.contains("approved artifact"))
    );
    let workflow = fixture.engine.snapshot(fixture.task.id).unwrap();
    assert!(workflow.completed.is_empty());
    assert_eq!(workflow.generation, 2);
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
        .unwrap(); // Reviewer native run
    let calls = fixture.gates.calls.lock().unwrap().len();
    fixture.sources.snapshot.lock().unwrap().revision = "known-stale".into();
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Invalidated { .. }
    ));
    assert_eq!(fixture.gates.calls.lock().unwrap().len(), calls);
}
#[tokio::test]
async fn waiting_gate_reuses_native_session_and_unknown_outcome_stays_reserved() {
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
    fixture.gates.waiting.store(true, Ordering::SeqCst);
    fixture.sources.snapshot.lock().unwrap().revision = "agent-produced-target".into();
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Waiting { .. }
    ));
    let count = fixture.executor.launches.lock().unwrap().len();
    fixture.gates.waiting.store(false, Ordering::SeqCst);
    assert!(matches!(
        fixture.engine.resume_gate(fixture.task.id).await.unwrap(),
        StepResult::Completed {
            phase: Phase::Implement
        }
    ));
    assert_eq!(fixture.executor.launches.lock().unwrap().len(), count);
    assert_eq!(
        fixture
            .engine
            .snapshot(fixture.task.id)
            .unwrap()
            .sources
            .revision,
        "agent-produced-target"
    );
    fixture.gates.unknown.store(true, Ordering::SeqCst);
    assert!(
        matches!(fixture.engine.step(fixture.task.id, BTreeMap::new()).await.unwrap(), StepResult::Waiting { reason, .. } if reason.contains("outcome unknown"))
    );
    let workflow = fixture.engine.snapshot(fixture.task.id).unwrap();
    assert_eq!(
        workflow.history[workflow.active.unwrap()].state,
        AttemptState::Evaluating
    );
    assert!(
        workflow.history[workflow.active.unwrap()]
            .observations
            .last()
            .unwrap()
            .error
            .as_ref()
            .unwrap()
            .contains("outcome unknown")
    );
    assert!(
        fixture
            .engine
            .retry(fixture.task.id, "unsafe retry".into())
            .is_err()
    );
    assert!(fixture.engine.resume_gate(fixture.task.id).await.is_err());
}
#[tokio::test]
async fn restarted_native_attempt_reports_durable_recovery_without_rebinding_or_launching() {
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
    let fresh = Arc::new(FakeAgent::new("executor", fixture.store.clone(), false));
    let mut registry = AgentRegistry::default();
    registry.register("executor".into(), fresh.clone()).unwrap();
    let engine = WorkflowEngine::new(
        fixture.store.clone(),
        Arc::new(registry),
        fixture.config,
        fixture.sources,
        fixture.gates,
    )
    .unwrap();
    assert!(
        matches!(engine.step(fixture.task.id, BTreeMap::new()).await.unwrap(), StepResult::Waiting { reason, .. } if reason.contains("recovery required"))
    );
    let workflow = engine.snapshot(fixture.task.id).unwrap();
    let attempt = &workflow.history[workflow.active.unwrap()];
    assert_eq!(attempt.state, AttemptState::Running);
    assert!(
        attempt
            .detail
            .as_ref()
            .unwrap()
            .contains("recovery required")
    );
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .session(attempt.session_id.unwrap())
            .unwrap()
            .unwrap()
            .0
            .state,
        SessionState::Running
    );
    assert!(fresh.launches.lock().unwrap().is_empty());
    assert!(
        engine
            .retry(fixture.task.id, "unverified death".into())
            .is_err()
    );
}
#[tokio::test]
async fn ordinary_store_cannot_override_workflow_fields_or_context_and_retry_preserves_other_blockers()
 {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    for field in 0..7 {
        let mut store = fixture.store.lock().unwrap();
        let mut task = store.task(fixture.task.id).unwrap().unwrap();
        match field {
            0 => task.state = TaskState::Completed,
            1 => task.workflow = WorkflowClass::Strict,
            2 => task.risk = RiskClass::R3,
            3 => task.context_version += 1,
            4 => task.revision = Some("forged".into()),
            5 => task.phase = Some("cleanup".into()),
            _ => task.artifacts.push("forged".into()),
        }
        assert!(store.put_task(&mut task).is_err());
    }
    let mut context = fixture
        .store
        .lock()
        .unwrap()
        .context(&fixture.task.scope(), None)
        .unwrap()
        .unwrap();
    context.version += 1;
    assert!(fixture.store.lock().unwrap().put_context(&context).is_err());
    fixture.through(Phase::Tests).await;
    fixture.gates.approved.store(false, Ordering::SeqCst);
    fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap();
    fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap();
    let old = fixture.engine.snapshot(fixture.task.id).unwrap();
    let old_attempt = &old.history[old.active.unwrap()];
    {
        let mut store = fixture.store.lock().unwrap();
        let mut task = store.task(fixture.task.id).unwrap().unwrap();
        task.blockers.insert(0, "unrelated blocker".into());
        store.put_task(&mut task).unwrap();
    }
    fixture
        .engine
        .retry(fixture.task.id, "fix rejected review".into())
        .unwrap();
    let retried = fixture.engine.snapshot(fixture.task.id).unwrap();
    assert_eq!(
        retried.history[old.active.unwrap()].completed_at,
        old_attempt.completed_at
    );
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .task(fixture.task.id)
            .unwrap()
            .unwrap()
            .blockers,
        ["unrelated blocker"]
    );
}
#[tokio::test]
async fn atomic_authority_rejects_invented_initial_history_completion_and_finished_flags() {
    let base = Fixture::new(WorkflowClass::Quick);
    let template = base.engine.initialize(base.task.id, None).await.unwrap();
    let template_context = base
        .store
        .lock()
        .unwrap()
        .context(&base.task.scope(), None)
        .unwrap()
        .unwrap();
    for mutation in 0..5 {
        let fixture = Fixture::new(WorkflowClass::Quick);
        let mut snapshot = template.clone();
        snapshot.sources.scope = fixture.task.scope();
        match mutation {
            0 => snapshot.generation = 2,
            1 => snapshot.finished = true,
            2 => snapshot.history.push(PhaseAttempt {
                phase: Phase::Worktree,
                generation: 1,
                context_version: 1,
                budget: ContextBudget {
                    class: BudgetClass::Small,
                    discretionary_tokens: 1,
                },
                state: AttemptState::Succeeded,
                session_id: None,
                dispatch_started: false,
                observations: vec![],
                claimed_observations: 0,
                agent: None,
                started_at: 0,
                completed_at: Some(1),
                detail: None,
            }),
            3 => {
                snapshot.completed.insert(
                    Phase::Pr,
                    Evidence {
                        scope: fixture.task.scope(),
                        phase: Phase::Pr,
                        revision: snapshot.sources.revision.clone(),
                        source_versions: snapshot.sources.source_versions.clone(),
                        artifacts: vec!["forged".into()],
                        dependencies: snapshot.sources.source_versions.clone(),
                        review_approved: None,
                        session_id: None,
                        context_version: 1,
                    },
                );
            }
            _ => {}
        }
        let mut task = fixture.task.clone();
        task.context_version = 1;
        task.revision = Some(snapshot.sources.revision.clone());
        let mut record = Record::new(
            task.scope(),
            RecordKind::Workflow,
            serde_json::to_value(snapshot).unwrap(),
        );
        let mut context = template_context.clone();
        context.scope = task.scope();
        let goal_version = fixture
            .store
            .lock()
            .unwrap()
            .goal(task.goal_id)
            .unwrap()
            .unwrap()
            .version;
        let result = fixture.store.lock().unwrap().put_workflow_transition(
            &mut task,
            &mut record,
            Some(&context),
            fixture.project.version,
            goal_version,
            WorkflowAccess::StateOnly,
        );
        if mutation == 4 {
            result.unwrap();
            assert_eq!(
                fixture
                    .store
                    .lock()
                    .unwrap()
                    .records(&task.scope(), RecordKind::Workflow)
                    .unwrap()
                    .len(),
                1
            );
        } else {
            let error = result.unwrap_err().to_string();
            assert!(
                error.contains(if mutation == 1 {
                    "finished requires all configured gates"
                } else {
                    "initial workflow must contain no invented"
                }),
                "mutation {mutation}: {error}"
            );
            assert!(
                fixture
                    .store
                    .lock()
                    .unwrap()
                    .records(&task.scope(), RecordKind::Workflow)
                    .unwrap()
                    .is_empty()
            );
        }
    }
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
        .unwrap(); // Running Implement
    for mutation in 0..5 {
        let mut store = fixture.store.lock().unwrap();
        let mut task = store.task(fixture.task.id).unwrap().unwrap();
        let mut record = store
            .records(&task.scope(), RecordKind::Workflow)
            .unwrap()
            .remove(0);
        let mut snapshot: WorkflowSnapshot = serde_json::from_value(record.data.clone()).unwrap();
        match mutation {
            0 => snapshot.finished = true,
            1 => {
                let evidence = snapshot.completed[&Phase::Worktree].clone();
                snapshot.completed.insert(Phase::Pr, evidence);
            }
            2 => snapshot.history[snapshot.active.unwrap()].state = AttemptState::Succeeded,
            3 => snapshot.active = None,
            _ => {}
        }
        record.data = serde_json::to_value(snapshot).unwrap();
        let goal_version = store.goal(task.goal_id).unwrap().unwrap().version;
        let result = store.put_workflow_transition(
            &mut task,
            &mut record,
            None,
            fixture.project.version,
            goal_version,
            WorkflowAccess::StateOnly,
        );
        if mutation == 4 {
            result.unwrap();
        } else {
            let error = result.unwrap_err().to_string();
            assert!(
                error.contains(match mutation {
                    0 => "finished requires all configured gates",
                    1 => "completion requires Evaluating",
                    2 => "attempt state regression",
                    _ => "active reservation can only close",
                }),
                "mutation {mutation}: {error}"
            );
        }
    }
}

#[tokio::test]
async fn native_git_commit_port_freezes_owning_worktree_head_for_review_and_pr() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    let root = fixture.project.root.clone();
    let worktree = fixture.task.worktree.clone().unwrap();
    let git = |cwd: &std::path::Path, args: &[&str]| {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };
    git(&root, &["init", "-b", "main"]);
    git(&root, &["config", "user.name", "Fixture"]);
    git(&root, &["config", "user.email", "fixture@example.invalid"]);
    git(&root, &["commit", "--allow-empty", "-m", "source fixture"]);
    git(
        &root,
        &[
            "worktree",
            "add",
            "-b",
            "feature/task",
            worktree.to_str().unwrap(),
            "main",
        ],
    );
    let original = git(&worktree, &["rev-parse", "HEAD"]);
    fixture.sources.snapshot.lock().unwrap().revision = original.clone();
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Implement).await;
    std::fs::write(
        worktree.join("implementation.txt"),
        "actual fixture implementation\n",
    )
    .unwrap();
    let sources = fixture.sources.clone();
    let source_root = root.clone();
    let owned_worktree = worktree.clone();
    *fixture.gates.on_complete.lock().unwrap() = Some(Box::new(move |invocation| {
        assert_eq!(invocation.task.worktree.as_ref(), Some(&owned_worktree));
        assert_eq!(invocation.phase, Phase::Commit);
        for args in [
            vec!["add", "implementation.txt"],
            vec!["commit", "-m", "implementation fixture"],
        ] {
            assert!(
                std::process::Command::new("git")
                    .args(args)
                    .current_dir(&owned_worktree)
                    .output()
                    .unwrap()
                    .status
                    .success()
            );
        }
        let output = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&owned_worktree)
            .output()
            .unwrap();
        assert!(output.status.success());
        sources.snapshot.lock().unwrap().revision =
            String::from_utf8(output.stdout).unwrap().trim().into();
        // The primary source branch remains untouched by the scoped native port.
        assert!(
            std::process::Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(&source_root)
                .output()
                .unwrap()
                .status
                .success()
        );
        Some(sources.snapshot.lock().unwrap().clone())
    }));
    fixture.through(Phase::Commit).await;
    let committed = git(&worktree, &["rev-parse", "HEAD"]);
    assert_ne!(committed, original);
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), original);
    assert!(git(&worktree, &["status", "--porcelain"]).is_empty());
    fixture.finish().await;
    let workflow = fixture.engine.snapshot(fixture.task.id).unwrap();
    assert_eq!(workflow.generation, 1);
    assert_eq!(
        workflow.completed[&Phase::ImplementationReview].revision,
        committed
    );
    assert_eq!(workflow.completed[&Phase::Pr].revision, committed);
}

#[tokio::test]
async fn equal_class_escalation_cannot_rebind_unverified_head_or_raw_authority() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::ImplementationReview).await;
    let calls = fixture.gates.calls.lock().unwrap().len();
    fixture.sources.snapshot.lock().unwrap().revision = "unverified-drift".into();
    let snapshot = fixture
        .engine
        .escalate(
            fixture.task.id,
            RiskClass::R0,
            None,
            "same class".into(),
            "explicit request".into(),
        )
        .await
        .unwrap();
    assert_eq!(snapshot.generation, 2);
    assert!(snapshot.completed.is_empty());
    assert_eq!(fixture.gates.calls.lock().unwrap().len(), calls);
    assert!(
        snapshot
            .invalidations
            .last()
            .unwrap()
            .cause
            .contains("source drift")
    );
    assert!(snapshot.escalations.is_empty());
    // A matching context cannot make a raw same-generation source rebind valid.
    let mut store = fixture.store.lock().unwrap();
    let mut task = store.task(fixture.task.id).unwrap().unwrap();
    let mut record = store
        .records(&task.scope(), RecordKind::Workflow)
        .unwrap()
        .remove(0);
    let mut workflow: WorkflowSnapshot = serde_json::from_value(record.data.clone()).unwrap();
    workflow.sources.revision = "forged-raw-head".into();
    let context = super::make_context(
        &task,
        &workflow.sources,
        Phase::Worktree,
        workflow.workflow,
        workflow.generation,
        ContextBudget {
            class: BudgetClass::Small,
            discretionary_tokens: 1,
        },
        task.context_version + 1,
    );
    task.revision = Some(context.revision.clone());
    task.context_version = context.version;
    workflow.context_version = context.version;
    record.data = serde_json::to_value(workflow).unwrap();
    let goal_version = store.goal(task.goal_id).unwrap().unwrap().version;
    assert!(
        store
            .put_workflow_transition(
                &mut task,
                &mut record,
                Some(&context),
                fixture.project.version,
                goal_version,
                WorkflowAccess::StateOnly
            )
            .unwrap_err()
            .to_string()
            .contains("authority change requires target-producing completion")
    );
}
#[tokio::test]
async fn concurrent_metadata_before_gate_claim_or_native_dispatch_preserves_progress() {
    for native in [false, true] {
        let fixture = Fixture::new(WorkflowClass::Quick);
        fixture
            .engine
            .initialize(fixture.task.id, None)
            .await
            .unwrap();
        fixture
            .through(if native {
                Phase::Worktree
            } else {
                Phase::Implement
            })
            .await;
        let store = fixture.store.clone();
        let id = fixture.task.id;
        let target = fixture.sources.captures.load(Ordering::SeqCst) + 3;
        *fixture.sources.on_numbered_capture.lock().unwrap() = Some((
            target,
            Box::new(move || {
                let mut store = store.lock().unwrap();
                let mut task = store.task(id).unwrap().unwrap();
                task.next_action = Some("concurrent metadata".into());
                store.put_task(&mut task).unwrap();
                let mut goal = store.goal(task.goal_id).unwrap().unwrap();
                goal.blockers.push("unrelated goal note".into());
                store.put_goal(&mut goal).unwrap();
            }),
        ));
        let result = fixture.engine.step(id, BTreeMap::new()).await.unwrap();
        assert!(if native {
            matches!(
                result,
                StepResult::Started {
                    phase: Phase::Implement,
                    ..
                }
            )
        } else {
            matches!(
                result,
                StepResult::Completed {
                    phase: Phase::Commit
                }
            )
        });
        assert_eq!(
            fixture
                .store
                .lock()
                .unwrap()
                .task(id)
                .unwrap()
                .unwrap()
                .next_action
                .as_deref(),
            Some("concurrent metadata")
        );
    }
}
#[tokio::test]
async fn observed_gate_outcome_survives_paused_blocked_or_missing_postgate_sources() {
    for failure in 0..3 {
        let fixture = Fixture::new(WorkflowClass::Quick);
        fixture
            .engine
            .initialize(fixture.task.id, None)
            .await
            .unwrap();
        fixture.through(Phase::Implement).await;
        let store = fixture.store.clone();
        let sources = fixture.sources.clone();
        *fixture.gates.on_complete.lock().unwrap() = Some(Box::new(move |invocation| {
            if failure == 2 {
                sources.capture_error.store(true, Ordering::SeqCst);
            } else {
                let mut store = store.lock().unwrap();
                if failure == 0 {
                    let mut goal = store.goal(invocation.task.goal_id).unwrap().unwrap();
                    goal.state = GoalState::Paused;
                    store.put_goal(&mut goal).unwrap();
                } else {
                    let mut project = store.project(invocation.project.id).unwrap().unwrap();
                    project.state = ProjectState::Blocked;
                    project.blocked_reason = Some("source unavailable".into());
                    store.put_project(&mut project).unwrap();
                }
            }
            None
        }));
        assert!(
            fixture
                .engine
                .step(fixture.task.id, BTreeMap::new())
                .await
                .is_err()
        );
        let workflow = fixture.engine.snapshot(fixture.task.id).unwrap();
        let attempt = &workflow.history[workflow.active.unwrap()];
        assert_eq!(attempt.state, AttemptState::Evaluating);
        assert!(
            matches!(&attempt.observations.last().unwrap().outcome, Some(GateOutcome::Passed(evidence)) if evidence.phase == Phase::Commit && evidence.artifacts == ["fixture://commit"])
        );
        let events = fixture
            .store
            .lock()
            .unwrap()
            .events(&fixture.task.scope(), 0, 10000)
            .unwrap();
        assert!(
            events
                .iter()
                .any(|event| event.kind == "workflow.gate_observed"
                    && event.data["observation"]["outcome"]["Passed"]["artifacts"][0]
                        == "fixture://commit")
        );
        assert!(!workflow.completed.contains_key(&Phase::Commit));
        let calls = fixture.gates.calls.lock().unwrap().len();
        if failure == 2 {
            fixture.sources.capture_error.store(false, Ordering::SeqCst);
        } else {
            let mut store = fixture.store.lock().unwrap();
            if failure == 0 {
                let mut goal = store.goal(fixture.task.goal_id).unwrap().unwrap();
                goal.state = GoalState::Running;
                store.put_goal(&mut goal).unwrap();
            } else {
                let mut project = store.project(fixture.project.id).unwrap().unwrap();
                project.state = ProjectState::Registered;
                project.blocked_reason = None;
                store.put_project(&mut project).unwrap();
            }
        }
        assert!(matches!(
            fixture
                .engine
                .step(fixture.task.id, BTreeMap::new())
                .await
                .unwrap(),
            StepResult::Completed {
                phase: Phase::Commit
            }
        ));
        assert_eq!(
            fixture.gates.calls.lock().unwrap().len(),
            calls,
            "known outcome must not repeat operation"
        );
    }
}
#[tokio::test]
async fn live_consultant_does_not_strand_readonly_review_but_mutating_gate_remains_fenced() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Tests).await;
    let consultant = Session {
        id: SessionId::new(),
        scope: fixture.task.scope(),
        agent: "consultant".into(),
        provider: "fake".into(),
        role: SessionRole::Consultant,
        native_ref: None,
        pid: None,
        worktree: fixture.task.worktree.clone().unwrap(),
        state: SessionState::Running,
        model: None,
        effort: None,
        recovery: Value::Null,
        started_at: now_ms(),
    };
    fixture
        .store
        .lock()
        .unwrap()
        .put_session(&consultant, 0)
        .unwrap();
    fixture.through(Phase::ImplementationReview).await;
    let calls = fixture.gates.calls.lock().unwrap().len();
    assert!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .is_err()
    );
    assert_eq!(fixture.gates.calls.lock().unwrap().len(), calls);
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .session(consultant.id)
            .unwrap()
            .unwrap()
            .0
            .state,
        SessionState::Running
    );
}
#[tokio::test]
async fn unknown_gate_generation_cannot_be_released_by_raw_transition() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Implement).await;
    fixture.gates.unknown.store(true, Ordering::SeqCst);
    fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap();
    let mut store = fixture.store.lock().unwrap();
    let mut task = store.task(fixture.task.id).unwrap().unwrap();
    let mut record = store
        .records(&task.scope(), RecordKind::Workflow)
        .unwrap()
        .remove(0);
    let mut workflow: WorkflowSnapshot = serde_json::from_value(record.data.clone()).unwrap();
    let index = workflow.active.unwrap();
    workflow.history[index].state = AttemptState::Interrupted;
    workflow.history[index].completed_at = Some(now_ms());
    workflow.active = None;
    let source = workflow.sources.clone();
    super::invalidate(&mut workflow, &source, "forged recovery").unwrap();
    let context = super::make_context(
        &task,
        &source,
        Phase::Worktree,
        workflow.workflow,
        workflow.generation,
        ContextBudget {
            class: BudgetClass::Small,
            discretionary_tokens: 1,
        },
        task.context_version + 1,
    );
    task.context_version = context.version;
    workflow.context_version = context.version;
    record.data = serde_json::to_value(workflow).unwrap();
    let goal_version = store.goal(task.goal_id).unwrap().unwrap().version;
    assert!(
        store
            .put_workflow_transition(
                &mut task,
                &mut record,
                Some(&context),
                fixture.project.version,
                goal_version,
                WorkflowAccess::StateOnly
            )
            .unwrap_err()
            .to_string()
            .contains("unknown gate reservation requires explicit recovery")
    );
}
#[tokio::test]
async fn quick_requires_actual_merge_cleanup_before_terminal_and_cancel_never_implies_native_death()
{
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.finish().await;
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .task(fixture.task.id)
            .unwrap()
            .unwrap()
            .state,
        TaskState::PrCreated
    );
    let mut removed = fixture.project.clone();
    removed.state = ProjectState::Removed;
    assert!(
        fixture
            .store
            .lock()
            .unwrap()
            .put_project(&mut removed)
            .is_err()
    );
    {
        let mut store = fixture.store.lock().unwrap();
        let mut task = store.task(fixture.task.id).unwrap().unwrap();
        task.state = TaskState::WaitingHuman;
        store.put_task(&mut task).unwrap();
    }
    fixture
        .engine
        .request_finalization(fixture.task.id, "actual merge/cleanup requested".into())
        .await
        .unwrap();
    fixture.gates.waiting.store(true, Ordering::SeqCst);
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Waiting {
            phase: Phase::MergeGate,
            ..
        }
    ));
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .task(fixture.task.id)
            .unwrap()
            .unwrap()
            .state,
        TaskState::WaitingHuman
    );
    fixture.gates.waiting.store(false, Ordering::SeqCst);
    fixture.engine.resume_gate(fixture.task.id).await.unwrap();
    assert!(
        fixture
            .store
            .lock()
            .unwrap()
            .task(fixture.task.id)
            .unwrap()
            .unwrap()
            .blockers
            .is_empty()
    );
    fixture.finish().await;
    let workflow = fixture.engine.snapshot(fixture.task.id).unwrap();
    assert!(
        workflow.completed.contains_key(&Phase::MergeGate)
            && workflow.completed.contains_key(&Phase::Cleanup)
    );
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .task(fixture.task.id)
            .unwrap()
            .unwrap()
            .state,
        TaskState::Completed
    );
    {
        let mut store = fixture.store.lock().unwrap();
        assert!(store.put_project(&mut removed).is_err());
        let mut goal = store.goal(fixture.task.goal_id).unwrap().unwrap();
        goal.state = GoalState::Cancelled;
        goal.blockers.push("fixture Goal explicitly closed".into());
        store.put_goal(&mut goal).unwrap();
        store.put_project(&mut removed).unwrap();
    }
    for lost in [false, true] {
        let fixture = Fixture::new(WorkflowClass::Quick);
        fixture
            .engine
            .initialize(fixture.task.id, None)
            .await
            .unwrap();
        fixture.through(Phase::Worktree).await;
        let StepResult::Started {
            session: Some(id), ..
        } = fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap()
        else {
            panic!("native not launched")
        };
        if lost {
            let mut store = fixture.store.lock().unwrap();
            let (mut session, version) = store.session(id).unwrap().unwrap();
            session.state = SessionState::Lost;
            store.put_session(&session, version).unwrap();
        }
        fixture
            .engine
            .cancel(fixture.task.id, "explicit cancellation".into())
            .unwrap();
        let mut store = fixture.store.lock().unwrap();
        assert_eq!(
            store.task(fixture.task.id).unwrap().unwrap().state,
            TaskState::Cancelled
        );
        assert_eq!(
            store.session(id).unwrap().unwrap().0.state,
            if lost {
                SessionState::Lost
            } else {
                SessionState::Running
            }
        );
        let mut project = fixture.project.clone();
        project.state = ProjectState::Removed;
        assert!(store.put_project(&mut project).is_err());
    }
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture
        .engine
        .fail_task(fixture.task.id, "explicit terminal failure".into())
        .unwrap();
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .task(fixture.task.id)
            .unwrap()
            .unwrap()
            .state,
        TaskState::Failed
    );
}

#[tokio::test]
async fn known_irreversible_drift_holds_single_operation_and_cleanup_reuses_frozen_authority() {
    for phase in [Phase::Pr, Phase::MergeGate] {
        let fixture = Fixture::new(WorkflowClass::Standard);
        fixture
            .engine
            .initialize(fixture.task.id, None)
            .await
            .unwrap();
        fixture
            .through(if phase == Phase::Pr {
                Phase::ImplementationReview
            } else {
                Phase::Pr
            })
            .await;
        let sources = fixture.sources.clone();
        *fixture.gates.on_complete.lock().unwrap() = Some(Box::new(move |_| {
            sources.snapshot.lock().unwrap().revision = "changed-after-external-effect".into();
            None
        }));
        let generation = fixture.engine.snapshot(fixture.task.id).unwrap().generation;
        assert!(
            matches!(fixture.engine.step(fixture.task.id, BTreeMap::new()).await.unwrap(), StepResult::Waiting { phase: actual, .. } if actual == phase)
        );
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
        let snapshot = fixture.engine.snapshot(fixture.task.id).unwrap();
        assert_eq!(snapshot.generation, generation);
        assert!(snapshot.held_reason.is_some());
        let stored = fixture
            .store
            .lock()
            .unwrap()
            .task(fixture.task.id)
            .unwrap()
            .unwrap();
        assert_eq!(stored.state, TaskState::WaitingHuman);
        assert!(
            stored
                .blockers
                .iter()
                .any(|b| Some(b) == snapshot.held_reason.as_ref())
        );
        assert!(
            snapshot.history[snapshot.active.unwrap()]
                .observations
                .last()
                .unwrap()
                .outcome
                .is_some()
        );
        assert!(
            fixture
                .engine
                .retry(fixture.task.id, "must not duplicate external effect".into())
                .is_err()
        );
    }
    for failed in [false, true] {
        let fixture = Fixture::new(WorkflowClass::Standard);
        fixture
            .engine
            .initialize(fixture.task.id, None)
            .await
            .unwrap();
        fixture.through(Phase::MergeGate).await;
        let sources = fixture.sources.clone();
        *fixture.gates.on_complete.lock().unwrap() = Some(Box::new(move |_| {
            sources.capture_error.store(true, Ordering::SeqCst);
            None
        }));
        if failed {
            fixture.gates.corrupt.store(1, Ordering::SeqCst);
        } else {
            fixture.gates.waiting.store(true, Ordering::SeqCst);
        }
        let result = fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap();
        assert!(matches!(
            result,
            StepResult::Waiting {
                phase: Phase::Cleanup,
                ..
            } | StepResult::Failed {
                phase: Phase::Cleanup,
                ..
            }
        ));
        let captures = fixture.sources.captures.load(Ordering::SeqCst);
        fixture.gates.waiting.store(false, Ordering::SeqCst);
        fixture.gates.corrupt.store(0, Ordering::SeqCst);
        assert!(matches!(
            fixture.engine.resume_gate(fixture.task.id).await.unwrap(),
            StepResult::Completed {
                phase: Phase::Cleanup
            }
        ));
        assert_eq!(fixture.sources.captures.load(Ordering::SeqCst), captures);
        assert_eq!(
            fixture
                .gates
                .calls
                .lock()
                .unwrap()
                .last()
                .unwrap()
                .prior_observations
                .len(),
            1
        );
        assert!(
            fixture
                .store
                .lock()
                .unwrap()
                .task(fixture.task.id)
                .unwrap()
                .unwrap()
                .blockers
                .is_empty()
        );
        assert_eq!(
            fixture
                .store
                .lock()
                .unwrap()
                .task(fixture.task.id)
                .unwrap()
                .unwrap()
                .state,
            TaskState::Completed
        );
    }
}

#[tokio::test]
async fn inactive_owners_allow_cancel_but_terminal_release_requires_verified_session() {
    for owner in 0..3 {
        let fixture = Fixture::new(WorkflowClass::Quick);
        fixture
            .engine
            .initialize(fixture.task.id, None)
            .await
            .unwrap();
        fixture.through(Phase::Worktree).await;
        let StepResult::Started {
            session: Some(id), ..
        } = fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap()
        else {
            panic!("native required")
        };
        {
            let mut store = fixture.store.lock().unwrap();
            if owner == 2 {
                let mut project = store.project(fixture.project.id).unwrap().unwrap();
                project.state = ProjectState::Blocked;
                project.blocked_reason = Some("fixture blocked".into());
                store.put_project(&mut project).unwrap();
            } else {
                let mut goal = store.goal(fixture.task.goal_id).unwrap().unwrap();
                goal.state = if owner == 0 {
                    GoalState::Paused
                } else {
                    GoalState::Cancelled
                };
                store.put_goal(&mut goal).unwrap();
            }
        }
        fixture
            .engine
            .cancel(fixture.task.id, "cancel under inactive owners".into())
            .unwrap();
        assert!(
            fixture
                .engine
                .release_terminal_reservation(fixture.task.id, "cannot imply death".into())
                .unwrap_err()
                .to_string()
                .contains("verified native termination")
        );
        {
            let mut store = fixture.store.lock().unwrap();
            let (mut session, version) = store.session(id).unwrap().unwrap();
            session.state = SessionState::Lost;
            store.put_session(&session, version).unwrap();
        }
        assert!(
            fixture
                .engine
                .release_terminal_reservation(fixture.task.id, "Lost is still reserved".into())
                .is_err()
        );
        {
            let mut store = fixture.store.lock().unwrap();
            let (mut session, version) = store.session(id).unwrap().unwrap();
            session.state = SessionState::Stopped;
            session.pid = None;
            store.put_session(&session, version).unwrap(); // trusted recovery boundary; fixture has no OS child
        }
        fixture
            .engine
            .release_terminal_reservation(
                fixture.task.id,
                "fixture recovery verified termination".into(),
            )
            .unwrap();
        let snapshot = fixture.engine.snapshot(fixture.task.id).unwrap();
        assert!(snapshot.active.is_none());
        assert_eq!(
            snapshot.history.last().unwrap().state,
            AttemptState::Interrupted
        );
        assert!(snapshot.terminal_decision.is_some());
        assert_eq!(
            fixture
                .store
                .lock()
                .unwrap()
                .task(fixture.task.id)
                .unwrap()
                .unwrap()
                .state,
            TaskState::Cancelled
        );
    }
}

#[tokio::test]
async fn final_claim_cas_loss_recovers_only_proven_undispatched_reservations() {
    for native in [false, true] {
        let fixture = Fixture::new(WorkflowClass::Quick);
        fixture
            .engine
            .initialize(fixture.task.id, None)
            .await
            .unwrap();
        fixture
            .through(if native {
                Phase::Worktree
            } else {
                Phase::Implement
            })
            .await;
        let store = fixture.store.clone();
        let id = fixture.task.id;
        let target = fixture.sources.captures.load(Ordering::SeqCst) + 4;
        *fixture.sources.on_numbered_capture.lock().unwrap() = Some((
            target,
            Box::new(move || {
                let mut store = store.lock().unwrap();
                let mut task = store.task(id).unwrap().unwrap();
                task.next_action = Some("last window CAS race".into());
                store.put_task(&mut task).unwrap();
            }),
        ));
        assert!(fixture.engine.step(id, BTreeMap::new()).await.is_err());
        let calls = fixture.gates.calls.lock().unwrap().len();
        let result = fixture.engine.step(id, BTreeMap::new()).await.unwrap();
        if native {
            assert!(matches!(result, StepResult::Invalidated { .. }));
            assert!(matches!(
                fixture.engine.step(id, BTreeMap::new()).await.unwrap(),
                StepResult::Started {
                    phase: Phase::Implement,
                    ..
                }
            ));
            assert_eq!(
                fixture
                    .store
                    .lock()
                    .unwrap()
                    .records(&fixture.task.scope(), RecordKind::Session)
                    .unwrap()
                    .len(),
                1
            );
        } else {
            assert!(matches!(
                result,
                StepResult::Completed {
                    phase: Phase::Commit
                }
            ));
            assert_eq!(fixture.gates.calls.lock().unwrap().len(), calls + 1);
        }
        assert_eq!(
            fixture
                .store
                .lock()
                .unwrap()
                .task(id)
                .unwrap()
                .unwrap()
                .next_action
                .as_deref(),
            Some("last window CAS race")
        );
    }
}

#[tokio::test]
async fn all_public_audit_entrypoints_reject_reserved_gate_journal_kinds() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    let mut store = fixture.store.lock().unwrap();
    let goal = store.goal(fixture.task.goal_id).unwrap().unwrap();
    let versions = [fixture.project.version, goal.version, fixture.task.version];
    for kind in [
        "project.saved",
        "workflow.saved",
        "context.created",
        "usage.recorded",
        "workflow.gate_observed",
    ] {
        assert!(
            store
                .audit(&fixture.task.scope(), kind, json!({"forged":true}))
                .unwrap_err()
                .to_string()
                .contains("reserved")
        );
        assert!(
            store
                .audit_if_current(
                    &fixture.task.scope(),
                    versions,
                    kind,
                    json!({"forged":true})
                )
                .unwrap_err()
                .to_string()
                .contains("reserved")
        );
    }
    store
        .audit_if_current(
            &fixture.task.scope(),
            versions,
            "fixture.observed",
            json!({"positive":true}),
        )
        .unwrap();
}

#[tokio::test]
async fn native_gate_requires_persisted_owned_status_and_sessionless_completion_is_rejected() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    let StepResult::Started {
        session: Some(id), ..
    } = fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap()
    else {
        panic!("native required")
    };
    fixture
        .executor
        .status(SessionRef {
            id,
            scope: fixture.task.scope(),
        })
        .await
        .unwrap();
    let connection = rusqlite::Connection::open(fixture.dir.path().join("state.db")).unwrap();
    connection
        .execute("DELETE FROM records WHERE id=?1", [id.to_string()])
        .unwrap();
    let calls = fixture.gates.calls.lock().unwrap().len();
    assert!(
        matches!(fixture.engine.step(fixture.task.id, BTreeMap::new()).await.unwrap(), StepResult::Waiting { reason, .. } if reason.contains("persisted owned Session"))
    );
    assert_eq!(fixture.gates.calls.lock().unwrap().len(), calls);

    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    let store = fixture.store.clone();
    let task_id = fixture.task.id;
    let target = fixture.sources.captures.load(Ordering::SeqCst) + 4;
    *fixture.sources.on_numbered_capture.lock().unwrap() = Some((
        target,
        Box::new(move || {
            let mut store = store.lock().unwrap();
            let mut task = store.task(task_id).unwrap().unwrap();
            task.next_action = Some("reserve without launch".into());
            store.put_task(&mut task).unwrap();
        }),
    ));
    assert!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .is_err()
    );
    let mut store = fixture.store.lock().unwrap();
    let mut task = store.task(fixture.task.id).unwrap().unwrap();
    let mut record = store
        .records(&task.scope(), RecordKind::Workflow)
        .unwrap()
        .remove(0);
    let mut snapshot: WorkflowSnapshot = serde_json::from_value(record.data.clone()).unwrap();
    let index = snapshot.active.unwrap();
    assert!(snapshot.history[index].session_id.is_none());
    snapshot.history[index].state = AttemptState::Evaluating;
    record.data = serde_json::to_value(&snapshot).unwrap();
    let goal_version = store.goal(task.goal_id).unwrap().unwrap().version;
    store
        .put_workflow_transition(
            &mut task,
            &mut record,
            None,
            fixture.project.version,
            goal_version,
            WorkflowAccess::StateOnly,
        )
        .unwrap(); // valid control
    snapshot.history[index].state = AttemptState::Succeeded;
    snapshot.history[index].completed_at = Some(now_ms());
    snapshot.active = None;
    snapshot.completed.insert(
        Phase::Implement,
        Evidence {
            scope: task.scope(),
            phase: Phase::Implement,
            revision: snapshot.sources.revision.clone(),
            source_versions: snapshot.sources.source_versions.clone(),
            artifacts: vec!["forged-sessionless-output".into()],
            dependencies: snapshot.sources.source_versions.clone(),
            review_approved: None,
            session_id: None,
            context_version: snapshot.context_version,
        },
    );
    record.data = serde_json::to_value(snapshot).unwrap();
    assert!(
        store
            .put_workflow_transition(
                &mut task,
                &mut record,
                None,
                fixture.project.version,
                goal_version,
                WorkflowAccess::StateOnly
            )
            .unwrap_err()
            .to_string()
            .contains("owned native Session")
    );
}

#[tokio::test]
async fn cancelled_unknown_gate_keeps_project_reserved_after_goal_terminal() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.gates.unknown.store(true, Ordering::SeqCst);
    assert!(matches!(
        fixture
            .engine
            .step(fixture.task.id, BTreeMap::new())
            .await
            .unwrap(),
        StepResult::Waiting { .. }
    ));
    fixture
        .engine
        .cancel(fixture.task.id, "explicit stop requested".into())
        .unwrap();
    assert!(
        fixture
            .engine
            .release_terminal_reservation(
                fixture.task.id,
                "unknown side effect cannot be cleared".into()
            )
            .unwrap_err()
            .to_string()
            .contains("unknown external outcome")
    );
    let mut store = fixture.store.lock().unwrap();
    let mut goal = store.goal(fixture.task.goal_id).unwrap().unwrap();
    goal.state = GoalState::Cancelled;
    store.put_goal(&mut goal).unwrap();
    let mut project = store.project(fixture.project.id).unwrap().unwrap();
    project.state = ProjectState::Removed;
    assert!(
        store
            .put_project(&mut project)
            .unwrap_err()
            .to_string()
            .contains("active records")
    );
}

#[tokio::test]
async fn resumed_round_claim_never_replays_prior_outcome_during_poll_restart_or_cancel() {
    for cancelled in [false, true] {
        let fixture = Fixture::new(WorkflowClass::Standard);
        fixture
            .engine
            .initialize(fixture.task.id, None)
            .await
            .unwrap();
        fixture.through(Phase::Pr).await;
        fixture.gates.waiting.store(true, Ordering::SeqCst);
        assert!(matches!(
            fixture
                .engine
                .step(fixture.task.id, BTreeMap::new())
                .await
                .unwrap(),
            StepResult::Waiting {
                phase: Phase::MergeGate,
                ..
            }
        ));
        fixture.gates.waiting.store(false, Ordering::SeqCst);
        let hold = Arc::new(Notify::new());
        *fixture.gates.hold.lock().unwrap() = Some(hold.clone());
        let engine = fixture.engine.clone();
        let id = fixture.task.id;
        let pending = tokio::spawn(async move { engine.resume_gate(id).await });
        fixture.gates.entered.notified().await;
        let snapshot = fixture.engine.snapshot(id).unwrap();
        let index = snapshot.active.unwrap();
        assert_eq!(snapshot.history[index].claimed_observations, 1);
        assert_eq!(snapshot.history[index].observations.len(), 1);
        assert!(super::known_gate_observation(&snapshot.history[index]).is_none());
        let calls = fixture.gates.calls.lock().unwrap().len();
        assert!(
            matches!(fixture.engine.step(id, BTreeMap::new()).await.unwrap(), StepResult::Waiting { reason, .. } if reason.contains("explicit recovery"))
        );
        let restarted = WorkflowEngine::new(
            fixture.store.clone(),
            fixture.engine.registry.clone(),
            fixture.config.clone(),
            fixture.sources.clone(),
            fixture.gates.clone(),
        )
        .unwrap();
        assert!(
            matches!(restarted.step(id, BTreeMap::new()).await.unwrap(), StepResult::Waiting { reason, .. } if reason.contains("explicit recovery"))
        );
        if cancelled {
            fixture
                .engine
                .cancel(id, "cancel in-flight resumed round".into())
                .unwrap();
            assert!(
                fixture
                    .engine
                    .release_terminal_reservation(
                        id,
                        "old Waiting must not prove current outcome".into()
                    )
                    .unwrap_err()
                    .to_string()
                    .contains("unknown external outcome")
            );
        }
        hold.notify_one();
        let result = pending.await.unwrap();
        if cancelled {
            assert!(result.unwrap_err().to_string().contains("Task terminal"));
        } else {
            assert!(matches!(
                result.unwrap(),
                StepResult::Completed {
                    phase: Phase::MergeGate
                }
            ));
        }
        assert_eq!(fixture.gates.calls.lock().unwrap().len(), calls);
        let snapshot = fixture.engine.snapshot(id).unwrap();
        assert_eq!(snapshot.history[index].observations.len(), 2);
        assert!(matches!(
            snapshot.history[index].observations[1].outcome,
            Some(GateOutcome::Passed(_))
        ));
        let events = fixture
            .store
            .lock()
            .unwrap()
            .events(&fixture.task.scope(), 0, 10000)
            .unwrap();
        assert!(events.iter().any(|e| e.kind == "workflow.gate_observed"
            && e.data["observation"]["outcome"]["Passed"]["phase"] == "merge_gate"));
        assert!(
            snapshot.history[index]
                .observations
                .iter()
                .all(|o| o.sources.payload.is_empty())
        );
        if cancelled {
            fixture
                .engine
                .release_terminal_reservation(id, "known completed second round".into())
                .unwrap();
        }
    }
}

#[tokio::test]
async fn rejected_review_cancel_releases_only_verified_terminal_attempt_and_project() {
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
        StepResult::Started { .. }
    ));
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
    fixture
        .engine
        .cancel(
            fixture.task.id,
            "rejected review explicitly abandoned".into(),
        )
        .unwrap();
    fixture
        .engine
        .release_terminal_reservation(fixture.task.id, "review native already Exited".into())
        .unwrap();
    let snapshot = fixture.engine.snapshot(fixture.task.id).unwrap();
    assert!(snapshot.active.is_none());
    assert_eq!(
        snapshot.history.last().unwrap().state,
        AttemptState::Interrupted
    );
    let mut store = fixture.store.lock().unwrap();
    let mut goal = store.goal(fixture.task.goal_id).unwrap().unwrap();
    goal.state = GoalState::Cancelled;
    store.put_goal(&mut goal).unwrap();
    let mut project = store.project(fixture.project.id).unwrap().unwrap();
    project.state = ProjectState::Removed;
    store.put_project(&mut project).unwrap();
}

#[tokio::test]
async fn persisted_status_mismatch_is_durable_waiting_without_gate_or_rebinding() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    let StepResult::Started {
        session: Some(id), ..
    } = fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap()
    else {
        panic!("native required")
    };
    fixture
        .executor
        .status(SessionRef {
            id,
            scope: fixture.task.scope(),
        })
        .await
        .unwrap();
    {
        let mut store = fixture.store.lock().unwrap();
        let (mut saved, version) = store.session(id).unwrap().unwrap();
        saved.state = SessionState::Lost;
        store.put_session(&saved, version).unwrap();
    }
    let calls = fixture.gates.calls.lock().unwrap().len();
    assert!(
        matches!(fixture.engine.step(fixture.task.id, BTreeMap::new()).await.unwrap(), StepResult::Waiting { reason, .. } if reason.contains("status differs"))
    );
    let snapshot = fixture.engine.snapshot(fixture.task.id).unwrap();
    assert!(
        snapshot.history[snapshot.active.unwrap()]
            .detail
            .as_ref()
            .unwrap()
            .contains("recovery required")
    );
    let version = fixture
        .store
        .lock()
        .unwrap()
        .records(&fixture.task.scope(), RecordKind::Workflow)
        .unwrap()[0]
        .version;
    fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap();
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .records(&fixture.task.scope(), RecordKind::Workflow)
            .unwrap()[0]
            .version,
        version
    );
    assert_eq!(fixture.gates.calls.lock().unwrap().len(), calls);
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .session(id)
            .unwrap()
            .unwrap()
            .0
            .state,
        SessionState::Lost
    );
}

#[test]
fn project_risk_recommendation_can_strengthen_but_cannot_weaken_runtime_mapping() {
    let runtime = Config::default();
    let selected = runtime
        .with_project_text("[workflow]\nrisk_mapping = ['QUICK','QUICK','QUICK','QUICK']")
        .unwrap();
    assert_eq!(
        selected.workflow.risk_mapping,
        runtime.workflow.risk_mapping
    );
    let strengthened = runtime
        .with_project_text("[workflow]\nrisk_mapping = ['STRICT','STRICT','STRICT','STRICT']")
        .unwrap();
    assert_eq!(
        strengthened.workflow.risk_mapping,
        [WorkflowClass::Strict; 4]
    );
}
