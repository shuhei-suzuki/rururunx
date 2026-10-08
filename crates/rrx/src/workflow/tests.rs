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
    start_pause: Mutex<Option<Arc<Pause>>>,
    start_error: AtomicBool,
    start_error_session: AtomicBool,
    managed_unit: Mutex<Option<crate::execution::UnitId>>,
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
            start_pause: Mutex::new(None),
            start_error: AtomicBool::new(false),
            start_error_session: AtomicBool::new(false),
            managed_unit: Mutex::new(None),
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
            let pause = self.start_pause.lock().unwrap().take();
            if let Some(pause) = pause {
                pause.hold().await;
            }
            let start_error = self.start_error.load(Ordering::SeqCst);
            if start_error && !self.start_error_session.load(Ordering::SeqCst) {
                return Err(adapter_error("fake start outcome unknown"));
            }
            let session = Session {
                id: SessionId::new(),
                scope: request.scope,
                agent: self.name.clone(),
                provider: "fake".into(),
                role: request.role,
                native_ref: None,
                pid: None,
                worktree: request.worktree,
                state: if start_error {
                    SessionState::Failed
                } else {
                    SessionState::Running
                },
                model: None,
                effort: None,
                recovery: Value::Null,
                started_at: now_ms(),
            };
            let mut session = session;
            if let Some(id) = *self.managed_unit.lock().unwrap() {
                let mut store = self.store.lock().unwrap();
                let unit = store.execution_unit(id).unwrap();
                session.state = SessionState::Starting;
                let unit = store
                    .register_execution_session(&unit.authority(), &session)
                    .unwrap();
                session.state = SessionState::Running;
                store
                    .update_execution_session(&unit.authority(), &session, 1)
                    .unwrap();
            } else {
                self.store
                    .lock()
                    .unwrap()
                    .put_session(&session, 0)
                    .map_err(|e| adapter_error(&e.to_string()))?;
            }
            if start_error {
                // Generic-shaped terminal persistence without Workflow binding.
                // This synthetic row supplies no native settlement authority.
                return Err(adapter_error("fake start outcome unknown"));
            }
            self.statuses.lock().unwrap().insert(
                session.id,
                SessionStatus {
                    execution: None,
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
                if let Some(unit) = store.session_execution_unit(reference.id).unwrap() {
                    store
                        .finish_execution(
                            &unit.authority(),
                            if status.session.state == SessionState::Exited {
                                crate::execution::WorkOutcome::Success
                            } else {
                                crate::execution::WorkOutcome::Unknown
                            },
                            if status.session.state == SessionState::Exited {
                                crate::execution::Disposition::Completed
                            } else {
                                crate::execution::Disposition::Lost
                            },
                        )
                        .unwrap();
                    store
                        .close_execution_session(unit.id, &status.session, version)
                        .unwrap();
                } else {
                    store.put_session(&status.session, version).unwrap();
                }
                if let Some(unit) = store.session_execution_unit(reference.id).unwrap() {
                    status.execution = Some(crate::execution::native::NativeStatus {
                        handle: reference
                            .execution
                            .clone()
                            .expect("ledger fixture managed reference"),
                        authority: unit.authority(),
                        session: status.session.clone(),
                        work: unit.work,
                        disposition: unit.disposition,
                        cleanup: unit.cleanup,
                        wait_reason: unit.wait_reason,
                        pending: vec![],
                        result: None,
                        receipt: None,
                        observed_work: None,
                        metrics: None,
                        diagnostic: None,
                        failure: None,
                    });
                }
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
#[derive(Default)]
struct Pause {
    entered: Notify,
    resume: Notify,
}
impl Pause {
    async fn hold(&self) {
        self.entered.notify_one();
        self.resume.notified().await;
    }
    /// FM §8.3: the held operation was never reached (a `hold` would have
    /// stored its `entered` permit).
    async fn assert_never_entered(&self) {
        assert!(
            tokio::time::timeout(
                std::time::Duration::from_millis(200),
                self.entered.notified()
            )
            .await
            .is_err(),
            "FM F2: the held operation must not be reached"
        );
    }
}
type CaptureHook = Box<dyn FnOnce() + Send>;
struct Sources {
    snapshot: Mutex<SourceSnapshot>,
    on_capture: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    captures: AtomicUsize,
    on_numbered_capture: Mutex<Option<(usize, CaptureHook)>>,
    capture_error: AtomicBool,
    pauses: Mutex<BTreeMap<usize, Arc<Pause>>>,
}
impl Sources {
    fn new(scope: Scope) -> Self {
        Self {
            snapshot: Mutex::new(SourceSnapshot {
                scope,
                revision: "head-1".into(),
                artifact: None,
                source_versions: BTreeMap::from([("requirements".into(), "sha256-1".into())]),
                payload: "factual fixture context".into(),
            }),
            on_capture: Mutex::new(None),
            captures: AtomicUsize::new(0),
            on_numbered_capture: Mutex::new(None),
            capture_error: AtomicBool::new(false),
            pauses: Mutex::new(BTreeMap::new()),
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
            let pause = self.pauses.lock().unwrap().remove(&number);
            if let Some(pause) = pause {
                pause.hold().await;
            }
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
    retained_artifact: Mutex<Option<crate::execution::ArtifactId>>,
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
            retained_artifact: Mutex::new(None),
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
            if let Some(id) = *self.retained_artifact.lock().unwrap() {
                evidence.artifacts = vec![format!("rrx-artifact:{id}")];
            }
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
/// FM §8.3 F2: the typed preflight refusal, compared by variant.
fn assert_refusal(error: &anyhow::Error, expected: &NativePreflightRefusal) {
    let actual = error.downcast_ref::<NativePreflightRefusal>();
    assert_eq!(
        actual.map(std::mem::discriminant),
        Some(std::mem::discriminant(expected)),
        "FM F2: typed refusal {expected:?}: {error:#}"
    );
}
struct Fixture {
    dir: crate::runtime::LegacyFixture,
    _owner: Arc<crate::execution::RuntimeOwner>,
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
        Self::with_binding(class, true)
    }
    /// FM §8.1 L: legacy rows from accepted ingress through the ordered
    /// historical migration; a bound Task has the genuine
    /// `WorktreeManager::create` worktree and branch.
    fn with_binding(class: WorkflowClass, bound: bool) -> Self {
        let (dir, owner) = crate::runtime::legacy_fixture_blocking(
            crate::runtime::LegacyLayout::default(),
            |_| {},
            vec![crate::runtime::LegacyTask {
                key: "fixture-task",
                executor: "executor",
                workflow: class,
                risk: RiskClass::R0,
                reviewers: &["reviewer"],
            }],
        );
        let store = owner.store();
        let (project, task) = {
            let mut store = store.lock().unwrap();
            let task = dir.task();
            assert_eq!(task.workflow, class, "SETUP: accepted class");
            if bound {
                crate::git::WorktreeManager::create(&mut store, task.id).unwrap();
            }
            (
                store.project(task.project_id).unwrap().unwrap(),
                store.task(task.id).unwrap().unwrap(),
            )
        };
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
            _owner: owner,
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
    async fn through(&self, target: Phase) {
        for _ in 0..100 {
            if matches!(self.engine.step(self.task.id,BTreeMap::new()).await.unwrap(),StepResult::Completed{phase} if phase==target)
            {
                return;
            }
        }
        panic!("phase not reached");
    }
    /// FM §8.3 F2 observation: the Task body, Workflow record, audit, Units,
    /// Sessions, completed Git outputs and both adapters' callback counts.
    fn preimage(&self) -> Value {
        let store = self.store.lock().unwrap();
        let scope = self.task.scope();
        json!({
            "task": store.task(self.task.id).unwrap(),
            "workflow": store.records(&scope, RecordKind::Workflow).unwrap(),
            "events": store.events(&scope, 0, 1000).unwrap().len(),
            "units": store.execution_units(Some(&scope)).unwrap().len(),
            "sessions": store.records(&scope, RecordKind::Session).unwrap().len(),
            "git": crate::git::observed_git_outputs(),
            "launches": self.executor.launches.lock().unwrap().len()
                + self.reviewer.launches.lock().unwrap().len(),
        })
    }
    /// FM §8.3 F2: this legacy-Engine step into a Native phase is refused
    /// before source capture with `ManagedBindingUnavailable`, and nothing in
    /// the preimage changes.
    async fn f2_refused(&self) {
        self.refused_as(NativePreflightRefusal::ManagedBindingUnavailable)
            .await;
    }
    /// The same boundary when §8.3 selects another typed preflight refusal
    /// first (e.g. `AdapterUnavailable` for an unregistered executor).
    async fn refused_as(&self, expected: NativePreflightRefusal) {
        let before = self.preimage();
        let error = self
            .engine
            .step(self.task.id, BTreeMap::new())
            .await
            .expect_err("FM F2: the legacy Native step must be refused");
        assert_refusal(&error, &expected);
        assert_eq!(self.preimage(), before, "FM F2: preimage unchanged");
    }
    /// Steps the non-Native phases, then asserts `f2_refused` at the first
    /// Native one.
    async fn to_f2(&self) {
        for _ in 0..100 {
            let before = self.preimage();
            match self.engine.step(self.task.id, BTreeMap::new()).await {
                Ok(StepResult::Completed { .. }) => {}
                Ok(other) => panic!("FM F2: unexpected legacy step {other:?}"),
                Err(error) => {
                    assert_refusal(&error, &NativePreflightRefusal::ManagedBindingUnavailable);
                    assert_eq!(self.preimage(), before, "FM F2: preimage unchanged");
                    return;
                }
            }
        }
        panic!("FM F2: no Native phase reached");
    }
}

/// FM §8.3 F2 (R). Former subject: a Workflow result cannot publish ledger-only Ready metadata. The legacy Engine
/// (no production caller, S1) now stops at its legacy-Engine step into the Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn workflow_result_cannot_publish_ledger_only_ready_metadata() {
    use crate::execution::*;
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture.sources.snapshot.lock().unwrap().revision = "a".repeat(40);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    let id = UnitId::new();
    let unit = {
        let mut store = fixture.store.lock().unwrap();
        let (_, epoch) = store.begin_execution_epoch().unwrap();
        let task = store.task(fixture.task.id).unwrap().unwrap();
        let at = now_ms();
        let spec = ExecutionUnit {
            id,
            scope: task.scope(),
            kind: UnitKind::Executor,
            generation: 0,
            owner_epoch: epoch,
            version: 0,
            phase: Phase::Implement.key().into(),
            provider: "fake".into(),
            state: UnitState::Reserved,
            native_effects_open: true,
            result_finalization_open: true,
            work: None,
            cleanup: CleanupOutcome::Unknown,
            disposition: Disposition::Active,
            worktree: fixture.project.worktree_root.join(id.to_string()),
            branch: Some(format!("rrx/{}/{id}", task.id)),
            base_sha: "a".repeat(40),
            profile_digest: "b".repeat(64),
            cookie: uuid::Uuid::new_v4().to_string(),
            session_id: None,
            artifact_id: None,
            wait_reason: None,
            capacity_retry_at: None,
            created_at: at,
            updated_at: at,
        };
        let reserved = store.reserve_execution(spec, task.version).unwrap();
        store
            .transition_execution(&reserved.authority(), UnitState::Preparing)
            .unwrap()
    };
    *fixture.executor.managed_unit.lock().unwrap() = Some(id);
    fixture.f2_refused().await;
    // The reserved Unit in reach is unchanged by the refused step.
    assert_eq!(
        serde_json::to_value(fixture.store.lock().unwrap().execution_unit(id).unwrap()).unwrap(),
        serde_json::to_value(&unit).unwrap()
    );
}

/// FM §8.3 F2 (R). Former subject: every preset drives real adapter calls and persists phase/context history. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
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
        fixture.to_f2().await;
    }
}
/// FM §8.3 F2 (R). Former subject: escalation restarts mandatory prerequisites without erasing history or downgrading. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn escalation_restarts_mandatory_prerequisites_without_erasing_history_or_downgrading() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}
/// FM §8.3 F2 (R). Former subject: a stale revision invalidates evidence and a wait requires explicit retry. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn stale_revision_invalidates_evidence_and_waiting_requires_explicit_retry() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}
/// FM §8.3 F2 (R). Former subject: exit zero without review approval cannot satisfy the review gate. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn exit_zero_without_review_approval_cannot_satisfy_review_gate() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}
/// FM §8.3 F2 (R). Former subject: a lost Native executor fences explicit retry and other phase dispatch. The legacy Engine
/// (no production caller, S1) now stops at its legacy-Engine step into the Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
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
    fixture.f2_refused().await;
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
/// FM §8.4 S4 split. A blocked Project cannot progress or publish Context.
/// The former paused-Goal half is refused on legacy rows (S4-W); it is kept
/// as a Task-level change on the same legacy row: the Task cancelled through
/// the typed Workflow `cancel` cannot progress either, and no executor
/// launches.
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
    let blocked = fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap_err();
    assert!(
        blocked.to_string().contains("Project inactive"),
        "{blocked:#}"
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
        crate::runtime::assert_goal_change_refused(&mut store, goal);
    }
    fixture
        .engine
        .cancel(fixture.task.id, "Task-level stop".into())
        .unwrap();
    let inactive = fixture
        .engine
        .step(fixture.task.id, BTreeMap::new())
        .await
        .unwrap_err();
    assert!(
        inactive.to_string().contains("Task terminal"),
        "{inactive:#}"
    );
    assert_eq!(
        fixture
            .engine
            .snapshot(fixture.task.id)
            .unwrap()
            .context_version,
        version
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
/// FM §8.3 F2 (R). Former subject: a second Engine cannot dispatch a duplicate phase, and an interrupt does not auto-retry (both concurrent steps are refused). The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn second_engine_cannot_dispatch_duplicate_phase_and_interrupt_does_not_auto_retry() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    let before = fixture.preimage();
    let (a, b) = tokio::join!(
        fixture.engine.step(fixture.task.id, BTreeMap::new()),
        fixture.engine.step(fixture.task.id, BTreeMap::new())
    );
    for result in [a, b] {
        let error = result.expect_err("FM F2: both legacy Native steps are refused");
        assert_refusal(&error, &NativePreflightRefusal::ManagedBindingUnavailable);
    }
    assert_eq!(fixture.preimage(), before, "FM F2: preimage unchanged");
}
/// FM §8.3 F2 (R). Former subject: Project minimum and rule refresh override defaults and force a new generation (its pre-Native part is unchanged). The legacy Engine
/// (no production caller, S1) now stops at its legacy-Engine step into the Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
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
    fixture.f2_refused().await;
}
#[tokio::test]
async fn ordered_format_migration_preserves_v2_state_and_unknown_future_is_rejected() {
    let (fixture, task) = crate::runtime::accepted_goal_fixture().await;
    let db = fixture.state_path().with_file_name("legacy-v2.db");
    // Only the historical nongrant tables are copied. Accepted authority,
    // scheduler/Driver state and Native execution rows are not migration input.
    crate::runtime::write_historical_copy(&fixture.state_path(), &db);
    let restored = Store::open(&db).unwrap();
    assert_eq!(
        restored.schema_version().unwrap(),
        crate::state::SCHEMA_VERSION
    );
    assert_eq!(
        restored.task(task.id).unwrap().unwrap().scope(),
        task.scope()
    );
    assert_eq!(
        serde_json::to_value(restored.task(task.id).unwrap().unwrap()).unwrap(),
        serde_json::to_value(&task).unwrap(),
        "historical migration preserves the actual Task body and version"
    );
    drop(restored);
    let connection = rusqlite::Connection::open(&db).unwrap();
    for table in [
        "goal_authority",
        "scheduler_tasks",
        "task_drivers",
        "execution_units",
    ] {
        let count: u64 = connection
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "legacy copy must not restore {table} authority");
    }
    connection
        .pragma_update(None, "user_version", crate::state::SCHEMA_VERSION + 1)
        .unwrap();
    drop(connection);
    assert!(Store::open(&db).is_err());
}

/// FM §8.3 F2 (R). Former subject: concurrent Native polls claim evidence evaluation before calling the external gate. The legacy Engine
/// (no production caller, S1) now stops at its legacy-Engine step into the Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn concurrent_native_polls_claim_evidence_evaluation_before_calling_external_gate() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    fixture.f2_refused().await;
}
/// FM §8.3 F2 (R). Former subject: revision changes during the review gate preserve the reservation and never accept a stale verdict. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn revision_changes_during_review_gate_preserve_reservation_and_never_accept_stale_verdict() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
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
/// FM §8.3 F2 (R). Former subject: Workflow authority and decision history require atomic monotonic updates. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn workflow_authority_and_decision_history_require_atomic_monotonic_updates() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}
/// FM §8.3 F2 (R). Former subject: explicit stricter selection and an unsupported review capability are honest (its explicit-selection scenario is unchanged). The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
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
    fixture.to_f2().await;
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

/// FM §8.3 F2 (R). Former subject: a running attempt preserves its Native actor when the reviewer choice changes. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn running_attempt_preserves_native_actor_when_task_reviewer_choice_changes() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}
/// FM §8.3 F2 (R). Former subject: a lost reviewer also requires verified recovery before retry. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn lost_reviewer_also_requires_verified_recovery_before_retry() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}

/// FM §8.3 F2 (R). Former subject: foreign, stale, unbound or empty gate evidence never advances Review. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn foreign_stale_unbound_or_empty_gate_evidence_never_advances_review() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}

/// FM §8.3 F2 (R). Former subject: Native turn completion preserves the real exit and still requires gate evidence. The legacy Engine
/// (no production caller, S1) now stops at its legacy-Engine step into the Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
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
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: generic completion requires a real zero exit and no failure. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn generic_completion_default_requires_real_zero_exit_and_no_failure() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}

/// FM §8.3 F2 (R). Former subject: completed-attempt rewrites and atomic owner version/activity changes are rejected. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn completed_attempt_rewrite_and_atomic_owner_version_or_activity_changes_are_rejected() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}

/// FM §8.3 F2 (R). Former subject: the default class is a fallback, not a floor, and a finished Workflow cannot reopen. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
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
    fixture.to_f2().await;
}
/// FM §8.4 S4 split. An Issue gate's own Task write (issue, next action,
/// blockers; legacy writer, S5) is preserved by the Workflow transition. The
/// former concurrent Goal constraint update is refused on legacy rows
/// (S4-W); the concurrent owner update is kept as a Project registry change
/// (`name`, §8.2), which the transition also preserves. The Worktree port
/// binds a previously unbound Task, now the accepted plan's unbound Task
/// bound through the genuine `WorktreeManager::create` (no forged
/// `feature/fresh-task` binding, §8.1.4).
#[tokio::test]
async fn gate_owner_updates_preserve_issue_binding_goal_metadata_and_task_blockers() {
    let fixture = Fixture::new(WorkflowClass::Standard);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    // Exercise an actual Issue port Task write and a concurrent owner update.
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
        crate::runtime::assert_goal_change_refused(&mut store, goal);
        let mut project = store.project(task.project_id).unwrap().unwrap();
        project.name = "concurrent registry name".into();
        store.put_project(&mut project).unwrap();
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
    assert_eq!(
        fixture
            .store
            .lock()
            .unwrap()
            .project(task.project_id)
            .unwrap()
            .unwrap()
            .name,
        "concurrent registry name"
    );
    // Worktree port can bind a previously unbound Task through the normal Store.
    let fixture = Fixture::with_binding(WorkflowClass::Quick, false);
    let task = fixture.task.clone();
    assert!(
        task.worktree.is_none() && task.branch.is_none(),
        "SETUP: unbound"
    );
    let store = fixture.store.clone();
    let assigned = Arc::new(Mutex::new(None));
    let created = assigned.clone();
    *fixture.gates.on_complete.lock().unwrap() = Some(Box::new(move |invocation| {
        let mut store = store.lock().unwrap();
        crate::git::WorktreeManager::create(&mut store, invocation.task.id).unwrap();
        let task = store.task(invocation.task.id).unwrap().unwrap();
        *created.lock().unwrap() = Some((task.worktree, task.branch));
        None
    }));
    fixture.engine.initialize(task.id, None).await.unwrap();
    assert!(matches!(
        fixture.engine.step(task.id, BTreeMap::new()).await.unwrap(),
        StepResult::Completed {
            phase: Phase::Worktree
        }
    ));
    let (worktree, branch) = assigned.lock().unwrap().clone().expect("Worktree port ran");
    let bound = fixture
        .store
        .lock()
        .unwrap()
        .task(task.id)
        .unwrap()
        .unwrap();
    assert!(worktree.is_some() && branch.is_some());
    assert_eq!((bound.worktree, bound.branch), (worktree, branch));
}
/// FM §8.3 F2 (R). Former subject: Commit publishes the new HEAD before Tests, Review and PR without a restart. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn commit_publishes_new_head_before_tests_review_and_pr_without_restart() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}
/// FM §8.3 F2 (R). Former subject: a changed approved artifact invalidates formal reviews and known drift never calls the gate. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn changed_approved_artifact_invalidates_formal_reviews_and_known_drift_never_calls_gate() {
    let fixture = Fixture::new(WorkflowClass::Standard);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}
/// FM §8.3 F2 (R). Former subject: a waiting gate reuses the Native Session and an unknown outcome stays reserved. The legacy Engine
/// (no production caller, S1) now stops at its legacy-Engine step into the Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn waiting_gate_reuses_native_session_and_unknown_outcome_stays_reserved() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    fixture.f2_refused().await;
}
/// FM §8.3 F2 (R). Former subject: a restarted Native attempt reports durable recovery without rebinding or launching. The legacy Engine
/// (no production caller, S1) now stops at its legacy-Engine step into the Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn restarted_native_attempt_reports_durable_recovery_without_rebinding_or_launching() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    fixture.f2_refused().await;
}
/// FM §8.3 F2 (R). Former subject: the ordinary Store cannot override Workflow fields or context, and retry preserves other blockers (its pre-Native part is unchanged). The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
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
    fixture.to_f2().await;
}
/// FM §8.3 F2 (R). Former subject: the atomic authority rejects invented initial history, completion and finished flags (its pre-Native scenarios are unchanged). The legacy Engine
/// (no production caller, S1) now stops at its legacy-Engine step into the Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
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
                execution: None,
                unit: None,
                native_wait: None,
                next_due: None,
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
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: the Native Git commit port freezes the
/// owning worktree HEAD for Review and PR. The legacy Engine (no production
/// caller, S1) now stops at its first step into the Native Implement phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture, with
/// the Task, Workflow, audit, Units, Sessions, Git outputs and adapter
/// callbacks unchanged. Its managed equivalent is SC-N continuation work
/// (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn native_git_commit_port_freezes_owning_worktree_head_for_review_and_pr() {
    let fixture = Fixture::new(WorkflowClass::Quick);
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
    // The legacy fixture's genuine repository and `WorktreeManager` worktree.
    let original = git(&worktree, &["rev-parse", "HEAD"]);
    fixture.sources.snapshot.lock().unwrap().revision = original.clone();
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}

/// FM §8.3 F2 (R). Former subject: equal-class escalation cannot rebind an unverified HEAD or raw authority. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn equal_class_escalation_cannot_rebind_unverified_head_or_raw_authority() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}
/// FM §8.3 F2 (R). Former subject: concurrent metadata before a gate claim or Native dispatch preserves progress. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn concurrent_metadata_before_gate_claim_or_native_dispatch_preserves_progress() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}
/// FM §8.3 F2 (R). Former subject: an observed gate outcome survives paused, blocked or missing post-gate sources. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn observed_gate_outcome_survives_paused_blocked_or_missing_postgate_sources() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}
/// FM §8.3 F2 (R). Former subject: a live consultant does not strand a read-only review but a mutating gate stays fenced. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn live_consultant_does_not_strand_readonly_review_but_mutating_gate_remains_fenced() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}
/// FM §8.3 F2 (R). Former subject: an unknown gate generation cannot be released by a raw transition. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn unknown_gate_generation_cannot_be_released_by_raw_transition() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}
/// FM §8.3 F2 (R). Former subject: Quick requires an actual merge/cleanup before terminal, and cancel never implies Native death. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn quick_requires_actual_merge_cleanup_before_terminal_and_cancel_never_implies_native_death()
{
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}

/// FM §8.3 F2 (R). Former subject: known irreversible drift holds a single operation and cleanup reuses frozen authority. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
/// Its cleanup-reuse scenario (MergeGate, Cleanup) stops at the same step.
#[tokio::test]
async fn known_irreversible_drift_holds_single_operation_and_cleanup_reuses_frozen_authority() {
    let fixture = Fixture::new(WorkflowClass::Standard);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}

/// FM §8.3 F2 (R). Former subject: inactive owners allow cancel but terminal release requires a verified Session. The legacy Engine
/// (no production caller, S1) now stops at its legacy-Engine step into the Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn inactive_owners_allow_cancel_but_terminal_release_requires_verified_session() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: a final claim CAS loss recovers only proven undispatched reservations. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn final_claim_cas_loss_recovers_only_proven_undispatched_reservations() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}

#[tokio::test]
async fn all_public_audit_entrypoints_reject_reserved_gate_journal_kinds() {
    let (fixture, task) = crate::runtime::accepted_goal_fixture().await;
    let shared = fixture.store();
    let mut store = shared.lock().unwrap();
    let project = store.project(task.project_id).unwrap().unwrap();
    let goal = store.goal(task.goal_id).unwrap().unwrap();
    let versions = [project.version, goal.version, task.version];
    let before = audit_control_rows(&fixture.state_path());
    for kind in [
        "project.saved",
        "workflow.saved",
        "context.created",
        "usage.recorded",
        "workflow.gate_observed",
    ] {
        assert!(
            store
                .audit(&task.scope(), kind, json!({"forged":true}))
                .unwrap_err()
                .to_string()
                .contains("reserved")
        );
        assert_eq!(
            audit_control_rows(&fixture.state_path()),
            before,
            "reserved audit refusal must leave all durable rows unchanged"
        );
        assert!(
            store
                .audit_if_current(&task.scope(), versions, kind, json!({"forged":true}))
                .unwrap_err()
                .to_string()
                .contains("reserved")
        );
        assert_eq!(
            audit_control_rows(&fixture.state_path()),
            before,
            "reserved current audit refusal must leave all durable rows unchanged"
        );
    }
    store
        .audit_if_current(
            &task.scope(),
            versions,
            "fixture.observed",
            json!({"positive":true}),
        )
        .unwrap();
    let events = store.events(&task.scope(), 0, 10000).unwrap();
    let positive = events.last().unwrap();
    assert_eq!(positive.kind, "fixture.observed");
    assert_eq!(positive.scope, task.scope());
    assert_eq!(positive.data, json!({"positive":true}));
    assert_eq!(store.task(task.id).unwrap().unwrap().version, versions[2]);
}

// This fixture never starts Runtime scheduling. Bracket refused audit calls with
// exact sorted row images from every durable table, including private authority.
fn audit_control_rows(path: &std::path::Path) -> Vec<(String, Vec<Vec<String>>)> {
    let connection =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    let names = connection
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    names
        .into_iter()
        .map(|name| {
            let mut query = connection
                .prepare(&format!("SELECT * FROM \"{}\"", name.replace('"', "\"\"")))
                .unwrap();
            let columns = query.column_count();
            let mut rows = query
                .query_map([], |r| {
                    (0..columns)
                        .map(|i| r.get_ref(i).map(|v| format!("{v:?}")))
                        .collect::<rusqlite::Result<Vec<_>>>()
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap();
            rows.sort();
            (name, rows)
        })
        .collect()
}

/// FM §8.3 F2 (R). Former subject: the Native gate requires persisted owned status and sessionless completion is rejected. The legacy Engine
/// (no production caller, S1) now stops at its legacy-Engine step into the Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn native_gate_requires_persisted_owned_status_and_sessionless_completion_is_rejected() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    fixture.f2_refused().await;
}

/// FM §8.4 S4 split. A cancelled Task with an unknown gate outcome cannot
/// release its terminal reservation without explicit recovery, and its
/// Project cannot be removed. The former terminal-Goal step is refused on
/// legacy rows (S4-W), so removal is refused by the still-active Goal. The
/// former subject "the unknown gate's records keep the Project reserved
/// after the Goal is terminal" has no legacy producer for a terminal Goal;
/// that half is R and the loss is recorded here (no new path is added).
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
            .contains("requires explicit recovery")
    );
    let mut store = fixture.store.lock().unwrap();
    let mut goal = store.goal(fixture.task.goal_id).unwrap().unwrap();
    goal.state = GoalState::Cancelled;
    crate::runtime::assert_goal_change_refused(&mut store, goal);
    let mut project = store.project(fixture.project.id).unwrap().unwrap();
    project.state = ProjectState::Removed;
    let refused = store.put_project(&mut project).unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("cannot remove project with active goals"),
        "{refused:#}"
    );
}

/// FM §8.3 F2 (R). Former subject: a resumed round claim never replays a prior outcome during poll, restart or cancel. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn resumed_round_claim_never_replays_prior_outcome_during_poll_restart_or_cancel() {
    let fixture = Fixture::new(WorkflowClass::Standard);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}

/// FM §8.3 F2 (R). Former subject: a rejected-review cancel releases only the verified terminal attempt and Project. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn rejected_review_cancel_releases_only_verified_terminal_attempt_and_project() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}

/// FM §8.3 F2 (R). Former subject: a persisted status mismatch is a durable wait without gate or rebinding. The legacy Engine
/// (no production caller, S1) now stops at its legacy-Engine step into the Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn persisted_status_mismatch_is_durable_waiting_without_gate_or_rebinding() {
    let fixture = Fixture::new(WorkflowClass::Quick);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    fixture.f2_refused().await;
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

/// FM §8.3 F2 (R). Former subject: a waiting irreversible hold then resume does not orphan its owned blocker. The legacy Engine
/// (no production caller, S1) now stops at its first legacy-Engine step into a Native phase:
/// the typed `ManagedBindingUnavailable` refusal before source capture,
/// with the Task, Workflow, audit, Units, Sessions, Git outputs and
/// adapter callbacks unchanged. Its managed equivalent is SC-N
/// continuation work (D3/D4); the rest of the former scenario is retired.
#[tokio::test]
async fn waiting_irreversible_hold_then_resume_does_not_orphan_owned_blocker() {
    let fixture = Fixture::new(WorkflowClass::Standard);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.to_f2().await;
}

// The barriers suspend actual capture/start consumers; no process-global timing hook.
impl Fixture {
    async fn ready_agent(review: bool) -> Self {
        let fixture = Self::new(WorkflowClass::Quick);
        fixture
            .engine
            .initialize(fixture.task.id, None)
            .await
            .unwrap();
        fixture
            .through(if review {
                Phase::Tests
            } else {
                Phase::Worktree
            })
            .await;
        fixture
    }
    fn fresh_engine(&self) -> Arc<WorkflowEngine> {
        Arc::new(
            WorkflowEngine::new(
                self.store.clone(),
                self.engine.registry.clone(),
                self.config.clone(),
                self.sources.clone(),
                self.gates.clone(),
            )
            .unwrap(),
        )
    }
    fn capture_pause(&self, offset: usize) -> Arc<Pause> {
        let pause = Arc::new(Pause::default());
        self.sources.pauses.lock().unwrap().insert(
            self.sources.captures.load(Ordering::SeqCst) + offset,
            pause.clone(),
        );
        pause
    }
    fn second_writer(&self) -> Store {
        Store::open(&self.dir.path().join("state.db")).unwrap()
    }
}

/// FM §8.3 F2 (R). Former subject: passive observers preserve both capture awaits and the owned single launch.
/// The legacy Engine (no production caller, S1) now stops at its step into
/// the Native phase, before source capture and before any pause/hook in
/// reach: the typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. The reviewer
/// variant cannot reach its review phase (Implement is refused first). Its
/// managed equivalent is SC-N continuation work (D3/D4).
#[tokio::test]
async fn preparation_observers_preserve_both_capture_awaits_and_owned_single_launch() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: a dispatched observation and the direct Store terminal fence. The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_dispatched_observation_and_direct_store_terminal_fence() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: a dropped owner is not released by a fresh Engine. The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_dropped_owner_is_not_released_by_a_fresh_engine() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: a capture error releases only the current owned claim and preserves metadata. The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_capture_errors_release_only_current_owned_claim_and_preserve_metadata() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: pause, cancel and terminal recovery keep the owner fenced. The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_pause_cancel_and_terminal_recovery_keep_owner_fenced() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.f2_refused().await;
    let fixture = Fixture::ready_agent(false).await;
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: a lifecycle ABA before refresh starts the same claim, after refresh retains it. The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_lifecycle_aba_before_refresh_starts_same_claim_after_refresh_retains() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: parent metadata and record replacement remain reserved. The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_parent_metadata_and_record_replacement_remain_reserved() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: release CAS and executor-lost fences do not retry. The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_release_cas_and_executor_lost_fences_do_not_retry() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.f2_refused().await;
}

#[test]
fn preparation_marker_classifier_requires_exact_table_and_owning_task_id() {
    let id = TaskId::new();
    for (table, other, expected) in [
        ("tasks", false, true),
        ("tasks", true, false),
        ("records", false, false),
        ("goals", false, false),
        ("projects", false, false),
    ] {
        let error = anyhow::Error::new(crate::state::StateGuardError::SnapshotChanged {
            table: table.into(),
            id: if other { TaskId::new() } else { id }.to_string(),
            expected: 1,
        })
        .context("transaction rollback");
        assert_eq!(super::own_task_marker_rollback(&error, id), expected);
    }
    assert!(!super::own_task_marker_rollback(
        &anyhow::anyhow!("SnapshotChanged tasks fabricated"),
        id
    ));
}

/// FM §8.3 F2 (R). Former subject: a reserve loser cannot release the identical winner claim (both Engines are refused). The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_reserve_loser_cannot_release_identical_winner_claim() {
    let fixture = Fixture::ready_agent(false).await;
    let other = fixture.fresh_engine();
    *fixture.engine.hooks.attempt_started_at.lock().unwrap() = Some(42);
    *other.hooks.attempt_started_at.lock().unwrap() = Some(42);
    let before = fixture.preimage();
    let (a, b) = tokio::join!(
        fixture.engine.step(fixture.task.id, BTreeMap::new()),
        other.step(fixture.task.id, BTreeMap::new())
    );
    for result in [a, b] {
        let error = result.expect_err("FM F2: both legacy Native steps are refused");
        assert_refusal(&error, &NativePreflightRefusal::ManagedBindingUnavailable);
    }
    assert_eq!(fixture.preimage(), before, "FM F2: preimage unchanged");
}

/// FM §8.3 F2 (R). Former subject: an actor override uses a new claim before launch and the binding is immutable.
/// The legacy Engine (no production caller, S1) now stops at its step into
/// the Native phase, before source capture and before any pause/hook in
/// reach: the typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. The reviewer
/// variant cannot reach its review phase (Implement is refused first). Its
/// managed equivalent is SC-N continuation work (D3/D4).
#[tokio::test]
async fn preparation_actor_override_uses_new_claim_before_launch_and_binding_is_immutable() {
    let mut fixture = Fixture::ready_agent(false).await;
    let alternate = Arc::new(FakeAgent::new("alternate", fixture.store.clone(), false));
    let mut registry = AgentRegistry::default();
    registry
        .register("executor".into(), fixture.executor.clone())
        .unwrap();
    registry
        .register("reviewer".into(), fixture.reviewer.clone())
        .unwrap();
    registry
        .register("alternate".into(), alternate.clone())
        .unwrap();
    fixture.engine = Arc::new(
        WorkflowEngine::new(
            fixture.store.clone(),
            Arc::new(registry),
            fixture.config.clone(),
            fixture.sources.clone(),
            fixture.gates.clone(),
        )
        .unwrap(),
    );
    fixture.f2_refused().await;
    assert!(alternate.launches.lock().unwrap().is_empty());
}

/// FM §8.3 F2 (R). Former subject: invalidation internal errors release before publication only. The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_invalidation_internal_errors_release_before_publication_only() {
    // Fresh-source branch constructs its pack in capture 4. Observed-source
    // branch validates in capture 5 and constructs its pack in capture 6.
    let fixture = Fixture::ready_agent(false).await;
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: a definitive failure and both invalidation publication conflicts retain the claim (with the missing executor, §8.3 selects `AdapterUnavailable` first). The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_definitive_fail_and_both_invalidation_publication_conflicts_retain() {
    for changed in [0, 3, 4] {
        let fixture = Fixture::ready_agent(false).await;
        if changed == 0 {
            let mut store = fixture.second_writer();
            let mut task = store.task(fixture.task.id).unwrap().unwrap();
            task.executor = "missing-adapter".into();
            store.put_task(&mut task).unwrap();
        }
        if changed == 0 {
            fixture
                .refused_as(NativePreflightRefusal::AdapterUnavailable)
                .await;
        } else {
            fixture.f2_refused().await;
        }
    }
}

/// FM §8.3 F2 (R). Former subject: an untyped marker failure keeps the claim even when start is never called. The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_untyped_marker_failure_keeps_claim_even_when_start_never_called() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: a post-dispatch binding CAS conflict preserves the factual Session and claim. The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_post_dispatch_binding_cas_conflict_preserves_factual_session_and_claim() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: explicit retry refuses an unbound dispatch without recovery proof. The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_explicit_retry_refuses_unbound_dispatch_without_recovery_proof() {
    let fixture = Fixture::ready_agent(false).await;
    fixture.executor.start_error.store(true, Ordering::SeqCst);
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: a post-refresh record replacement rejects release after a typed Task marker error. The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_post_refresh_record_replacement_rejects_release_after_typed_task_marker_error()
{
    let fixture = Fixture::ready_agent(false).await;
    fixture.f2_refused().await;
}

/// FM §8.3 F2 (R). Former subject: an unbound-to-bound change before refresh releases for a new reservation. The legacy Engine
/// (no production caller, S1) now stops at its step into the Native
/// phase, before source capture and before any pause/hook in reach: the
/// typed preflight refusal with the Task, Workflow, audit, Units,
/// Sessions, Git outputs and adapter callbacks unchanged. Its managed
/// equivalent is SC-N continuation work (D3/D4); the rest of the former
/// scenario is retired.
#[tokio::test]
async fn preparation_unbound_to_bound_before_refresh_releases_for_new_reservation() {
    let fixture = Fixture::with_binding(WorkflowClass::Quick, false);
    fixture
        .engine
        .initialize(fixture.task.id, None)
        .await
        .unwrap();
    fixture.through(Phase::Worktree).await;
    fixture.f2_refused().await;
}

#[path = "tests/unbound_retry.rs"]
mod unbound_retry;
