//! Durable Task phases. Evidence providers, not process exits, authorize gates.
use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    adapter::{
        AgentAdapter, AgentRegistry, Capability, InputKind, LaunchMode, LaunchRequest,
        PreparedInput, SessionRef, SessionStatus, SharedStore,
    },
    config::{Config, WorkflowClass},
    domain::*,
    state::{StateGuardError, Store, WorkflowAccess, goal_terminal, task_terminal},
};

pub type WorkflowFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

/// A pre-reservation configuration refusal; callers must not hot-retry it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NativePreflightRefusal {
    MissingReviewer,
    AdapterUnavailable,
    MissingCapability(Capability),
    IdentityMismatch,
    ProbeFailed,
    ManagedBindingUnavailable,
}
impl std::fmt::Display for NativePreflightRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingReviewer => f.write_str("reviewer not configured"),
            Self::AdapterUnavailable => f.write_str("selected adapter unavailable"),
            Self::MissingCapability(capability) => {
                write!(f, "missing prepared native phase capability {capability:?}")
            }
            Self::IdentityMismatch => f.write_str("selected adapter probe identity mismatch"),
            Self::ProbeFailed => f.write_str("selected adapter probe failed"),
            Self::ManagedBindingUnavailable => {
                f.write_str("managed native binding and private admission are not composed")
            }
        }
    }
}
impl std::error::Error for NativePreflightRefusal {}

pub(crate) const UNBOUND_NATIVE_RECOVERY_REQUIRED: &str = "native launch outcome unknown; launch may or may not have begun; original-attempt recovery required (#14)";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Issue,
    Worktree,
    Requirements,
    RequirementsCommit,
    RequirementsReview,
    Design,
    DesignCommit,
    DesignReview,
    Implement,
    ImpactAnalysis,
    Commit,
    Tests,
    ImplementationReview,
    SecurityReview,
    ExpandedRegression,
    Mutation,
    Browser,
    Staging,
    Pr,
    MergeGate,
    Cleanup,
}
impl Phase {
    pub fn key(self) -> &'static str {
        match self {
            Self::Issue => "issue",
            Self::Worktree => "worktree",
            Self::Requirements => "requirements",
            Self::RequirementsCommit => "requirements_commit",
            Self::RequirementsReview => "requirements_review",
            Self::Design => "design",
            Self::DesignCommit => "design_commit",
            Self::DesignReview => "design_review",
            Self::Implement => "implement",
            Self::ImpactAnalysis => "impact_analysis",
            Self::Tests => "tests",
            Self::ImplementationReview => "implementation_review",
            Self::SecurityReview => "security_review",
            Self::ExpandedRegression => "expanded_regression",
            Self::Mutation => "mutation",
            Self::Browser => "browser",
            Self::Staging => "staging",
            Self::Commit => "commit",
            Self::Pr => "pr",
            Self::MergeGate => "merge_gate",
            Self::Cleanup => "cleanup",
        }
    }
    pub fn actor(self) -> Actor {
        match self {
            Self::Requirements | Self::Design | Self::Implement | Self::ImpactAnalysis => {
                Actor::Executor
            }
            Self::RequirementsReview
            | Self::DesignReview
            | Self::ImplementationReview
            | Self::SecurityReview => Actor::Reviewer,
            _ => Actor::EvidencePort,
        }
    }
    pub(crate) fn task_state(self) -> TaskState {
        match self {
            Self::Requirements | Self::Design | Self::Issue | Self::Worktree => TaskState::Planning,
            Self::Implement => TaskState::Implementing,
            Self::ImpactAnalysis => TaskState::Testing,
            Self::RequirementsReview
            | Self::DesignReview
            | Self::ImplementationReview
            | Self::SecurityReview => TaskState::Reviewing,
            Self::Tests
            | Self::ExpandedRegression
            | Self::Mutation
            | Self::Browser
            | Self::Staging => TaskState::Testing,
            Self::RequirementsCommit | Self::DesignCommit => TaskState::Planning,
            Self::Commit | Self::Pr => TaskState::ReadyForPr,
            Self::MergeGate | Self::Cleanup => TaskState::PrCreated,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Actor {
    Executor,
    Reviewer,
    EvidencePort,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetClass {
    Small,
    Normal,
    Broad,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBudget {
    pub class: BudgetClass,
    pub discretionary_tokens: usize,
}

pub fn phases(class: WorkflowClass, config: &Config) -> Vec<Phase> {
    use Phase::*;
    let mut result = if class == WorkflowClass::Quick {
        vec![Worktree, Implement, Commit, Tests, ImplementationReview]
    } else {
        vec![
            Issue,
            Worktree,
            Requirements,
            RequirementsCommit,
            RequirementsReview,
            Design,
            DesignCommit,
            DesignReview,
            Implement,
            ImpactAnalysis,
            Commit,
            Tests,
            ImplementationReview,
        ]
    };
    if class == WorkflowClass::Strict {
        result.extend([SecurityReview, ExpandedRegression, Mutation]);
        if config.workflow.browser_verification {
            result.push(Browser);
        }
        if config.workflow.staging_verification {
            result.push(Staging);
        }
    }
    result.push(Pr);
    if class != WorkflowClass::Quick {
        result.extend([MergeGate, Cleanup]);
    }
    result
}
fn retain_phases(previous: &[Phase], requested: Vec<Phase>) -> Vec<Phase> {
    // A policy refresh may add gates; previously required gates stay required.
    previous
        .iter()
        .copied()
        .chain(requested)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}
pub(crate) fn budget(class: WorkflowClass, phase: Phase, config: &Config) -> ContextBudget {
    ContextBudget {
        class: match class {
            WorkflowClass::Quick => BudgetClass::Small,
            WorkflowClass::Standard => BudgetClass::Normal,
            WorkflowClass::Strict => BudgetClass::Broad,
        },
        discretionary_tokens: if phase.actor() == Actor::Reviewer {
            config.context.review_context_tokens
        } else {
            config.context.repo_map_tokens
        },
    }
}
fn risk_workflow(config: &Config, risk: RiskClass) -> WorkflowClass {
    config.workflow.risk_mapping[match risk {
        RiskClass::R0 => 0,
        RiskClass::R1 => 1,
        RiskClass::R2 => 2,
        RiskClass::R3 => 3,
    }]
}
fn risk_max(a: RiskClass, b: RiskClass) -> RiskClass {
    let rank = |r| match r {
        RiskClass::R0 => 0,
        RiskClass::R1 => 1,
        RiskClass::R2 => 2,
        RiskClass::R3 => 3,
    };
    if rank(a) >= rank(b) { a } else { b }
}

/// Source capture is an integration port (#18). Implementors inspect owning
/// revision and authoritative source versions off the SharedStore mutex.
/// Source versions describe the same authority set across phases; phase/budget
/// affect payload selection, not which authority changes can be detected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSnapshot {
    pub scope: Scope,
    pub revision: String,
    /// Actual retained input identity, qualified by the source integration port.
    #[serde(default)]
    pub artifact: Option<crate::execution::ArtifactId>,
    pub source_versions: BTreeMap<String, String>,
    pub payload: String,
}
/// Private fields preserve the committed producer's complete policy/input frame.
pub struct CommittedWorkflowInput {
    pub(crate) config: Config,
    pub(crate) source: SourceSnapshot,
    pub(crate) budget: ContextBudget,
}
pub trait WorkflowSources: Send + Sync {
    fn committed_input(
        &self,
        _project: Project,
        _task: Task,
        _phase: Phase,
        _class: WorkflowClass,
    ) -> WorkflowFuture<'_, Option<CommittedWorkflowInput>> {
        Box::pin(async { Ok(None) })
    }
    fn take_initial_executor(
        &self,
        _project: &Project,
        _task: &Task,
        _phase: Phase,
        _budget: &ContextBudget,
    ) -> WorkflowFuture<'_, Option<crate::execution::workflow_source::InitialWorkflowExecutor>>
    {
        Box::pin(async { Ok(None) })
    }
    fn retire_initial(&self, _scope: &Scope) -> Result<()> {
        Ok(())
    }

    fn capture(
        &self,
        project: Project,
        task: Task,
        phase: Phase,
        budget: ContextBudget,
    ) -> WorkflowFuture<'_, SourceSnapshot>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub scope: Scope,
    pub phase: Phase,
    pub revision: String,
    pub source_versions: BTreeMap<String, String>,
    /// Durable references; callers attest real artifacts, never fabricated paths.
    pub artifacts: Vec<String>,
    /// Exact source dependencies approved by a review; all mandatory rules are
    /// included. Ports select artifact keys; the engine never invents coverage.
    pub dependencies: BTreeMap<String, String>,
    /// Only an external review integration can attest a decision.
    pub review_approved: Option<bool>,
    /// Binds an agent-produced artifact/verdict to the launched Session.
    pub session_id: Option<SessionId>,
    pub context_version: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateOutcome {
    Passed(Evidence),
    Waiting(String),
    Failed(String),
}
#[derive(Debug, Clone)]
pub struct PhaseInvocation {
    pub project: Project,
    pub task: Task,
    pub phase: Phase,
    pub context: ContextVersion,
    pub sources: SourceSnapshot,
    pub budget: ContextBudget,
    /// Includes only earlier valid prerequisites from this workflow generation.
    pub prerequisites: Vec<Evidence>,
    /// Actual earlier outcomes for this attempt; side-effect ports must reconcile
    /// these references rather than create duplicate PRs/merges/cleanup actions.
    pub prior_observations: Vec<GateObservation>,
}
/// #9/#12/GitHub/test integrations supply actual evidence. Absent evidence waits.
pub trait PhaseGates: Send + Sync {
    fn complete(
        &self,
        invocation: PhaseInvocation,
        transport: Option<SessionStatus>,
    ) -> WorkflowFuture<'_, GateOutcome>;
}
pub struct PendingGates;
impl PhaseGates for PendingGates {
    fn complete(
        &self,
        invocation: PhaseInvocation,
        _: Option<SessionStatus>,
    ) -> WorkflowFuture<'_, GateOutcome> {
        Box::pin(async move {
            Ok(GateOutcome::Waiting(format!(
                "{} evidence integration pending",
                invocation.phase.key()
            )))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttemptState {
    Running,
    Evaluating,
    Waiting,
    Failed,
    Succeeded,
    Interrupted,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseAttempt {
    pub phase: Phase,
    pub generation: u64,
    pub context_version: u64,
    pub budget: ContextBudget,
    pub state: AttemptState,
    pub session_id: Option<SessionId>,
    #[serde(default)]
    pub execution: Option<crate::execution::native::ManagedSessionRef>,
    #[serde(default)]
    pub unit: Option<crate::execution::ManagedUnitRef>,
    #[serde(default)]
    pub native_wait: Option<crate::execution::WaitReason>,
    #[serde(default)]
    pub next_due: Option<i64>,
    pub dispatch_started: bool,
    pub observations: Vec<GateObservation>,
    /// Observation count at the exact current evaluation claim. A prior round's
    /// result cannot resolve a resumed in-flight operation.
    pub claimed_observations: usize,
    pub agent: Option<String>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub detail: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateObservation {
    /// Exact pre-operation authority; retained for applying a known outcome
    /// without repeating the external operation after a transient error.
    pub sources: SourceSnapshot,
    pub outcome: Option<GateOutcome>,
    pub error: Option<String>,
    pub at: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalDecision {
    pub scope: Scope,
    pub generation: u64,
    pub attempt: Option<usize>,
    pub state: TaskState,
    pub reason: String,
    pub at: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizationRequest {
    pub scope: Scope,
    pub generation: u64,
    pub reason: String,
    pub at: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escalation {
    pub from: WorkflowClass,
    pub to: WorkflowClass,
    pub reason: String,
    pub evidence: String,
    pub at: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryEvent {
    pub prior_attempt: usize,
    pub reason: String,
    pub at: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invalidation {
    pub generation: u64,
    pub cause: String,
    pub previous_revision: String,
    pub revision: String,
    pub previous_sources: BTreeMap<String, String>,
    pub sources: BTreeMap<String, String>,
    pub at: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSnapshot {
    pub workflow: WorkflowClass,
    pub risk: RiskClass,
    pub generation: u64,
    pub context_version: u64,
    pub context_fresh: bool,
    pub active: Option<usize>,
    pub completed: BTreeMap<Phase, Evidence>,
    pub history: Vec<PhaseAttempt>,
    pub escalations: Vec<Escalation>,
    pub retries: Vec<RetryEvent>,
    pub invalidations: Vec<Invalidation>,
    pub terminal_decision: Option<TerminalDecision>,
    pub finalizations: Vec<FinalizationRequest>,
    pub sources: SourceSnapshot,
    pub configured_phases: Vec<Phase>,
    pub finished: bool,
    pub held_reason: Option<String>,
}
#[derive(Debug, Clone)]
pub enum StepResult {
    Started {
        phase: Phase,
        session: Option<SessionId>,
    },
    Running {
        phase: Phase,
        session: SessionId,
    },
    Completed {
        phase: Phase,
    },
    Waiting {
        phase: Phase,
        reason: String,
    },
    Failed {
        phase: Phase,
        reason: String,
    },
    Invalidated {
        reason: String,
    },
    Finished,
}
struct Snapshot {
    project: Project,
    goal: Goal,
    task: Task,
    record: Record,
    workflow: WorkflowSnapshot,
    publication: Option<crate::execution::WorkflowPublication>,
    readonly_completion: Option<crate::execution::ReadonlyCompletion>,
    verification_completion: Option<crate::execution::verification::VerificationCompletion>,
    verification_activation: Option<crate::execution::verification::ManagedVerificationActivation>,
}

/// Invocation-local proof minted only from a successfully committed agent reservation.
struct PreparationClaim {
    scope: Scope,
    record_id: RecordId,
    record_version: u64,
    generation: u64,
    index: usize,
    context_version: u64,
    worktree: Option<std::path::PathBuf>,
    branch: Option<String>,
    attempt: PhaseAttempt,
}
impl PreparationClaim {
    fn committed(snapshot: &Snapshot, index: usize) -> Self {
        Self {
            scope: snapshot.task.scope(),
            record_id: snapshot.record.id,
            record_version: snapshot.record.version,
            generation: snapshot.workflow.generation,
            index,
            context_version: snapshot.workflow.context_version,
            worktree: snapshot.task.worktree.clone(),
            branch: snapshot.task.branch.clone(),
            attempt: snapshot.workflow.history[index].clone(),
        }
    }
}
struct AgentPreparation {
    context: ContextVersion,
    config: Config,
    environment: BTreeMap<String, String>,
    selected: NativeAdapterSelection,
}
struct NativeAdapterSelection {
    adapter: Arc<dyn AgentAdapter>,
    agent: String,
    provider: String,
}
enum PhaseAdapterSelection {
    EvidencePort,
    Native(NativeAdapterSelection),
}
fn require_managed_native_binding_composed() -> Result<()> {
    // Capability metadata cannot authorize the legacy Task-rewriting binder.
    // Replace this refusal only when the actual private admission and record-only
    // binding producers are composed, with genuine positive controls.
    Err(NativePreflightRefusal::ManagedBindingUnavailable.into())
}
#[cfg(test)]
#[derive(Default)]
struct EngineHooks {
    before_publication: std::sync::Mutex<Option<Box<dyn FnOnce() + Send>>>,
    before_release: std::sync::Mutex<Option<Box<dyn FnOnce() + Send>>>,
    before_wait_claim: std::sync::Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send>>>>,
    before_reserve: std::sync::Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send>>>>,
    attempt_started_at: std::sync::Mutex<Option<i64>>,
}

/// Raw Workflow Store mutation stays private to the engine implementation.
/// ```compile_fail,E0624
/// use rrx::state::Store;
/// let _ = Store::put_workflow_transition;
/// ```
/// ```no_run
/// let _ = rrx::state::Store::put_record;
/// ```
pub struct WorkflowEngine {
    store: SharedStore,
    registry: Arc<AgentRegistry>,
    runtime: Config,
    sources: Arc<dyn WorkflowSources>,
    gates: Arc<dyn PhaseGates>,
    verifier: Option<Arc<crate::execution::verification::ManagedVerifier>>,
    managed_snapshots: std::sync::Mutex<
        BTreeMap<crate::execution::UnitId, crate::execution::results::ResultSnapshot>,
    >,
    #[cfg(test)]
    hooks: EngineHooks,
}
impl WorkflowEngine {
    pub fn new(
        store: SharedStore,
        registry: Arc<AgentRegistry>,
        runtime: Config,
        sources: Arc<dyn WorkflowSources>,
        gates: Arc<dyn PhaseGates>,
    ) -> Result<Self> {
        runtime.validate()?;
        if let Some(owner) = registry.managed_owner() {
            ensure!(
                Arc::ptr_eq(&owner.store(), &store),
                "managed registry and Workflow must share one Runtime store"
            );
        }
        Ok(Self {
            store,
            registry,
            runtime,
            sources,
            gates,
            verifier: None,
            managed_snapshots: std::sync::Mutex::new(BTreeMap::new()),
            #[cfg(test)]
            hooks: EngineHooks::default(),
        })
    }
    pub fn with_verifier(
        mut self,
        verifier: Arc<crate::execution::verification::ManagedVerifier>,
    ) -> Result<Self> {
        let owner = self
            .registry
            .managed_owner()
            .context("verifier requires a managed registry")?;
        ensure!(
            verifier.belongs_to(&owner),
            "verifier and Workflow Runtime differ"
        );
        self.verifier = Some(verifier);
        Ok(self)
    }
    pub fn snapshot(&self, task_id: TaskId) -> Result<WorkflowSnapshot> {
        Ok(self.read(task_id)?.workflow)
    }
    fn read(&self, task_id: TaskId) -> Result<Snapshot> {
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state store poisoned"))?;
        let (project, goal, task) = owners(&store, task_id)?;
        let records = store.records(&task.scope(), RecordKind::Workflow)?;
        ensure!(records.len() == 1, "Task must own exactly one workflow");
        let record = records.into_iter().next().expect("one workflow");
        let workflow: WorkflowSnapshot = serde_json::from_value(record.data.clone())?;
        ensure!(
            workflow.workflow == task.workflow && workflow.context_version == task.context_version,
            "workflow/Task pointers disagree"
        );
        Ok(Snapshot {
            publication: None,
            readonly_completion: None,
            verification_completion: None,
            verification_activation: None,
            project,
            goal,
            task,
            record,
            workflow,
        })
    }
    fn persist(&self, snapshot: &mut Snapshot, context: Option<&ContextVersion>) -> Result<()> {
        snapshot.record.data = serde_json::to_value(&snapshot.workflow)?;
        let mut store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state store poisoned"))?;
        if let Some(activation) = snapshot.verification_activation.as_ref() {
            return store.activate_managed_workflow(
                &mut snapshot.task,
                &mut snapshot.record,
                context.context("managed activation requires initial Context")?,
                snapshot.project.version,
                snapshot.goal.version,
                activation,
            );
        }
        if let Some(completion) = snapshot.verification_completion.as_ref() {
            return store.put_workflow_verification_transition(
                &mut snapshot.task,
                &mut snapshot.record,
                context.context("verification acceptance requires fresh Context")?,
                snapshot.project.version,
                snapshot.goal.version,
                completion,
            );
        }
        if snapshot.workflow.active.is_none()
            && let Some(previous) = store.record(snapshot.record.id)?
        {
            let before: WorkflowSnapshot = serde_json::from_value(previous.data)?;
            if let Some(index) = before.active {
                let attempt = before
                    .history
                    .get(index)
                    .context("invalid persisted phase reservation index")?;
                if attempt.phase.actor() == Actor::Executor
                    && snapshot
                        .workflow
                        .history
                        .get(index)
                        .context("phase reservation history missing")?
                        .state
                        == AttemptState::Succeeded
                    && let Some(session) = attempt.session_id
                    && let Some(unit) = store.session_execution_unit(session)?
                {
                    let evidence = snapshot
                        .workflow
                        .completed
                        .get(&attempt.phase)
                        .context("managed executor lacks passed evidence")?;
                    let artifacts = evidence
                        .artifacts
                        .iter()
                        .filter_map(|s| s.strip_prefix("rrx-artifact:"))
                        .collect::<Vec<_>>();
                    ensure!(
                        artifacts.len() == 1,
                        "managed executor requires one exact retained artifact"
                    );
                    let publication = snapshot
                        .publication
                        .as_ref()
                        .context("managed publication requires retained graph verification")?;
                    ensure!(
                        publication.authority() == &unit.authority()
                            && publication.artifact().id == artifacts[0].parse()?,
                        "verified publication identity mismatch"
                    );
                    return store.put_workflow_result_transition(
                        &mut snapshot.task,
                        &mut snapshot.record,
                        context.context("managed publication requires fresh context")?,
                        snapshot.project.version,
                        snapshot.goal.version,
                        publication,
                    );
                }
                if attempt.phase.actor() == Actor::Reviewer
                    && snapshot.workflow.history[index].state == AttemptState::Succeeded
                    && let Some(session) = attempt.session_id
                    && let Some(unit) = store.session_execution_unit(session)?
                {
                    let proof = snapshot.readonly_completion.as_ref().context(
                        "managed readonly acceptance requires verified snapshot provenance",
                    )?;
                    ensure!(
                        proof.authority() == &unit.authority(),
                        "readonly verified authority changed"
                    );
                    return store.put_workflow_readonly_transition(
                        &mut snapshot.task,
                        &mut snapshot.record,
                        context.context("readonly acceptance requires fresh context")?,
                        snapshot.project.version,
                        snapshot.goal.version,
                        proof,
                    );
                }
            }
        }
        store.put_workflow_transition(
            &mut snapshot.task,
            &mut snapshot.record,
            context,
            snapshot.project.version,
            snapshot.goal.version,
            WorkflowAccess::StateOnly,
        )
    }
    fn reserve(
        &self,
        snapshot: &mut Snapshot,
        context: &ContextVersion,
        phase: Phase,
    ) -> Result<()> {
        snapshot.record.data = serde_json::to_value(&snapshot.workflow)?;
        self.store
            .lock()
            .map_err(|_| anyhow::anyhow!("state store poisoned"))?
            .put_workflow_transition(
                &mut snapshot.task,
                &mut snapshot.record,
                Some(context),
                snapshot.project.version,
                snapshot.goal.version,
                if phase.actor() == Actor::Reviewer {
                    WorkflowAccess::ReadOnly
                } else {
                    WorkflowAccess::Mutating
                },
            )
    }
    async fn inputs(
        &self,
        project: &Project,
        task: &Task,
        phase: Phase,
        class: WorkflowClass,
    ) -> Result<(Config, SourceSnapshot, ContextBudget)> {
        if let Some(frame) = self
            .sources
            .committed_input(project.clone(), task.clone(), phase, class)
            .await?
        {
            ensure!(
                frame.source.scope == task.scope()
                    && crate::execution::valid_oid(&frame.source.revision)
                    && frame
                        .source
                        .source_versions
                        .keys()
                        .all(|key| !key.starts_with("workflow:")),
                "committed Workflow frame scope/revision mismatch"
            );
            frame.config.validate()?;
            return Ok((frame.config, frame.source, frame.budget));
        }
        let project_for_rules = project.clone();
        let runtime = self.runtime.clone();
        let (config, rules, versions) =
            tokio::task::spawn_blocking(move || load_rules(&project_for_rules, runtime)).await??;
        let selected_budget = budget(class, phase, &config);
        let mut source = self
            .sources
            .capture(
                project.clone(),
                task.clone(),
                phase,
                selected_budget.clone(),
            )
            .await?;
        ensure!(
            source.scope == task.scope() && !source.revision.trim().is_empty(),
            "source scope/revision mismatch"
        );
        ensure!(
            source
                .source_versions
                .keys()
                .all(|key| !key.starts_with("workflow:") && !key.starts_with("rules:")),
            "context source cannot overwrite workflow/rule versions"
        );
        source.source_versions.extend(versions);
        // Mandatory rules are retained outside discretionary repository budgeting.
        source.payload = format!("{rules}\n{}", source.payload);
        Ok((config, source, selected_budget))
    }
    async fn prepare_pack(
        &self,
        project: &Project,
        task: &Task,
        expected: &SourceSnapshot,
        phase: Phase,
        class: WorkflowClass,
        generation: u64,
    ) -> Result<ContextVersion> {
        // Evidence/artifact bookkeeping may already be added to this private
        // projected Task. Recovered Sources require the complete durable DTO;
        // its claim validates whole P/G/Workflow/Context and governing pins.
        let durable = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .recovered_source_task(task.id)?;
        if let Some(current) = &durable {
            ensure!(
                current.scope() == task.scope()
                    && current.version == task.version
                    && crate::execution::workflow_source::task_digest(current)?
                        == crate::execution::workflow_source::task_digest(task)?,
                "projected Context Task instructions/authority changed"
            );
            ensure!(
                expected.source_versions.get("instructions:task")
                    == Some(&crate::execution::workflow_source::task_digest(current)?),
                "projected Context differs from actual source instructions"
            );
        }
        let (config, source, selected_budget) = self
            .inputs(project, durable.as_ref().unwrap_or(task), phase, class)
            .await?;
        ensure!(
            same_sources(expected, &source),
            "authority changed while preparing phase Context Pack"
        );
        ensure!(
            class >= config.minimum_workflow && class >= risk_workflow(&config, task.risk),
            "workflow policy changed while preparing Context Pack"
        );
        Ok(make_context(
            task,
            &source,
            phase,
            class,
            generation,
            selected_budget,
            self.next_context(&task.scope())?,
        ))
    }
    pub async fn initialize(
        &self,
        task_id: TaskId,
        stricter: Option<WorkflowClass>,
    ) -> Result<WorkflowSnapshot> {
        let (project, goal, mut task) = {
            let store = self
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state store poisoned"))?;
            owners(&store, task_id)?
        };
        active(&project, &goal, &task)?;
        let (config, source, _) = self
            .inputs(&project, &task, Phase::Worktree, task.workflow)
            .await?;
        let workflow = task
            .workflow
            .max(config.minimum_workflow)
            .max(stricter.unwrap_or(WorkflowClass::Quick))
            .max(risk_workflow(&config, task.risk));
        let configured_phases = phases(workflow, &config);
        let phase = configured_phases[0];
        let context = self
            .prepare_pack(&project, &task, &source, phase, workflow, 1)
            .await?;
        task.workflow = workflow;
        task.context_version = context.version;
        task.revision = Some(source.revision.clone());
        let workflow_state = WorkflowSnapshot {
            workflow,
            risk: task.risk,
            generation: 1,
            context_version: context.version,
            context_fresh: true,
            active: None,
            completed: BTreeMap::new(),
            history: vec![],
            escalations: vec![],
            retries: vec![],
            invalidations: vec![],
            terminal_decision: None,
            finalizations: vec![],
            sources: SourceSnapshot {
                payload: context.data["payload"]
                    .as_str()
                    .context("prepared payload missing")?
                    .into(),
                ..source
            },
            configured_phases,
            finished: false,
            held_reason: None,
        };
        let record = Record::new(
            task.scope(),
            RecordKind::Workflow,
            serde_json::to_value(&workflow_state)?,
        );
        let mut snapshot = Snapshot {
            publication: None,
            readonly_completion: None,
            verification_completion: None,
            verification_activation: None,
            project,
            goal,
            task,
            record,
            workflow: workflow_state,
        };
        if let Some(owner) = self.registry.managed_owner() {
            snapshot.verification_activation = Some(
                crate::execution::verification::ManagedVerificationActivation::from_owner(
                    &owner,
                    &snapshot.record,
                )?,
            );
        }
        self.persist(&mut snapshot, Some(&context))?;
        Ok(snapshot.workflow)
    }
    fn next_context(&self, scope: &Scope) -> Result<u64> {
        self.store
            .lock()
            .map_err(|_| anyhow::anyhow!("state store poisoned"))?
            .context(scope, None)?
            .map_or(Ok(1), |c| {
                c.version.checked_add(1).context("context overflow")
            })
    }
    fn context(&self, snapshot: &Snapshot) -> Result<ContextVersion> {
        self.store
            .lock()
            .map_err(|_| anyhow::anyhow!("state store poisoned"))?
            .context(&snapshot.task.scope(), Some(snapshot.task.context_version))?
            .context("workflow context missing")
    }
    /// Explicit risk/user escalation never downgrades. Historical evidence stays
    /// visible, while all active prerequisites restart conservatively.
    pub async fn escalate(
        &self,
        task_id: TaskId,
        risk: RiskClass,
        stricter: Option<WorkflowClass>,
        reason: String,
        evidence: String,
    ) -> Result<WorkflowSnapshot> {
        ensure!(
            !reason.trim().is_empty() && !evidence.trim().is_empty(),
            "escalation needs reason/evidence"
        );
        let mut snapshot = self.read(task_id)?;
        let previous_generation = snapshot.workflow.generation;
        ensure!(
            !snapshot.workflow.finished,
            "finished workflow requires a new Task, not escalation"
        );
        ensure!(
            !has_external_effect(&snapshot.workflow),
            "external side effect requires reconciliation before escalation"
        );

        active(&snapshot.project, &snapshot.goal, &snapshot.task)?;
        ensure!(
            snapshot.workflow.active.is_none(),
            "resolve active phase before escalation"
        );
        let (config, source, _) = self
            .inputs(
                &snapshot.project,
                &snapshot.task,
                Phase::Worktree,
                snapshot.workflow.workflow,
            )
            .await?;
        let risk = risk_max(snapshot.workflow.risk, risk);
        let class = snapshot
            .workflow
            .workflow
            .max(config.minimum_workflow)
            .max(stricter.unwrap_or(WorkflowClass::Quick))
            .max(risk_workflow(&config, risk));
        if class > snapshot.workflow.workflow {
            snapshot.workflow.escalations.push(Escalation {
                from: snapshot.workflow.workflow,
                to: class,
                reason,
                evidence,
                at: now_ms(),
            });
            snapshot.workflow.workflow = class;
            snapshot.task.workflow = class;
            snapshot.workflow.configured_phases =
                retain_phases(&snapshot.workflow.configured_phases, phases(class, &config));
            invalidate(
                &mut snapshot.workflow,
                &source,
                "workflow policy or target changed",
            )?;
        }
        if snapshot.workflow.generation == previous_generation
            && !same_sources(&source, &snapshot.workflow.sources)
        {
            invalidate(
                &mut snapshot.workflow,
                &source,
                "source drift during explicit escalation",
            )?;
        }
        snapshot.workflow.configured_phases =
            retain_phases(&snapshot.workflow.configured_phases, phases(class, &config));
        snapshot.workflow.risk = risk;
        snapshot.task.risk = risk;
        let phase = next_phase(&snapshot.workflow).unwrap_or(Phase::Pr);
        let context = self
            .prepare_pack(
                &snapshot.project,
                &snapshot.task,
                &source,
                phase,
                class,
                snapshot.workflow.generation,
            )
            .await?;
        snapshot.workflow.sources = source;
        set_context(&mut snapshot, &context);
        self.persist(&mut snapshot, Some(&context))?;
        Ok(snapshot.workflow)
    }
    /// One bounded orchestration action. Agent execution itself is polled across
    /// steps; an interrupted durable reservation is not automatically retried.
    pub async fn step(
        &self,
        task_id: TaskId,
        environment: BTreeMap<String, String>,
    ) -> Result<StepResult> {
        let mut snapshot = self.read(task_id)?;
        if snapshot.workflow.finished {
            return Ok(StepResult::Finished);
        }
        active(&snapshot.project, &snapshot.goal, &snapshot.task)?;
        if let Some(index) = snapshot.workflow.active {
            return self.poll(snapshot, index).await;
        }
        let Some(phase) = next_phase(&snapshot.workflow) else {
            snapshot.workflow.finished = true;
            snapshot.task.phase = None;
            self.persist(&mut snapshot, None)?;
            return Ok(StepResult::Finished);
        };
        // Refuse before source capture: managed Sources can admit Git helpers.
        // Metadata is not the missing private composition or prepared owner proof.
        let selected = if phase.actor() == Actor::EvidencePort {
            PhaseAdapterSelection::EvidencePort
        } else {
            PhaseAdapterSelection::Native(self.preflight_native_adapter(&snapshot.task, phase)?)
        };
        let (config, source, selected_budget) = self
            .inputs(
                &snapshot.project,
                &snapshot.task,
                phase,
                snapshot.workflow.workflow,
            )
            .await?;
        let class = snapshot
            .workflow
            .workflow
            .max(config.minimum_workflow)
            .max(risk_workflow(&config, snapshot.workflow.risk));
        let configured =
            retain_phases(&snapshot.workflow.configured_phases, phases(class, &config));
        if has_external_effect(&snapshot.workflow)
            && (class > snapshot.workflow.workflow
                || configured != snapshot.workflow.configured_phases
                || !same_sources(&source, &snapshot.workflow.sources))
        {
            return self.hold(
                snapshot,
                None,
                phase,
                "existing PR/merge evidence held after drift; reconcile explicitly (#13)",
            );
        }
        if class > snapshot.workflow.workflow || configured != snapshot.workflow.configured_phases {
            let old = snapshot.workflow.workflow;
            snapshot.workflow.workflow = class;
            snapshot.task.workflow = class;
            snapshot.workflow.configured_phases = configured;
            if class > old {
                snapshot.workflow.escalations.push(Escalation {
                    from: old,
                    to: class,
                    reason: "scoped policy refreshed".into(),
                    evidence: serde_json::to_string(&source.source_versions)?,
                    at: now_ms(),
                });
            }
            invalidate(
                &mut snapshot.workflow,
                &source,
                "workflow policy or target changed",
            )?;
            let first = next_phase(&snapshot.workflow).expect("presets nonempty");
            let context = self
                .prepare_pack(
                    &snapshot.project,
                    &snapshot.task,
                    &source,
                    first,
                    class,
                    snapshot.workflow.generation,
                )
                .await?;
            snapshot.workflow.sources = source;
            set_context(&mut snapshot, &context);
            self.persist(&mut snapshot, Some(&context))?;
            return Ok(StepResult::Invalidated {
                reason: "workflow policy escalated; prerequisites restarted".into(),
            });
        }
        if !same_sources(&source, &snapshot.workflow.sources) {
            invalidate(
                &mut snapshot.workflow,
                &source,
                "workflow policy or target changed",
            )?;
            snapshot.workflow.sources = source.clone();
            let first = next_phase(&snapshot.workflow).expect("presets nonempty");
            let context = self
                .prepare_pack(
                    &snapshot.project,
                    &snapshot.task,
                    &source,
                    first,
                    class,
                    snapshot.workflow.generation,
                )
                .await?;
            set_context(&mut snapshot, &context);
            self.persist(&mut snapshot, Some(&context))?;
            return Ok(StepResult::Invalidated {
                reason: "revision/source/rules changed; stale evidence invalidated".into(),
            });
        }
        clear_hold(&mut snapshot);
        let context = self
            .prepare_pack(
                &snapshot.project,
                &snapshot.task,
                &source,
                phase,
                class,
                snapshot.workflow.generation,
            )
            .await?;
        set_context(&mut snapshot, &context);
        snapshot.workflow.sources = source;
        snapshot.task.phase = Some(phase.key().into());
        snapshot.task.state = phase.task_state();
        let index = snapshot.workflow.history.len();
        snapshot.workflow.history.push(PhaseAttempt {
            phase,
            generation: snapshot.workflow.generation,
            context_version: context.version,
            budget: selected_budget,
            state: AttemptState::Running,
            session_id: None,
            execution: None,
            unit: None,
            native_wait: None,
            next_due: None,
            dispatch_started: false,
            observations: vec![],
            claimed_observations: 0,
            agent: match phase.actor() {
                Actor::Executor => Some(snapshot.task.executor.clone()),
                Actor::Reviewer => snapshot.task.reviewers.first().cloned(),
                Actor::EvidencePort => None,
            },
            started_at: self.attempt_started_at(),
            completed_at: None,
            detail: None,
        });
        snapshot.workflow.active = Some(index);
        #[cfg(test)]
        {
            let hook = self
                .hooks
                .before_reserve
                .lock()
                .expect("reserve hook")
                .take();
            if let Some(hook) = hook {
                hook.await;
            }
        }
        self.reserve(&mut snapshot, &context, phase)?;
        let selected = match selected {
            PhaseAdapterSelection::EvidencePort => {
                return self.evaluate(snapshot, index, None).await;
            }
            PhaseAdapterSelection::Native(selected) => selected,
        };
        let claim = PreparationClaim::committed(&snapshot, index);
        let mut eligible = true;
        let result = self
            .prepare_agent(
                snapshot,
                AgentPreparation {
                    context,
                    config,
                    environment,
                    selected,
                },
                &claim,
                &mut eligible,
            )
            .await;
        match result {
            Err(error) if eligible => match self.release_preparation(&claim) {
                Ok(()) => Err(error),
                Err(release) => Err(error.context(format!(
                    "owner-local preparation release not performed; unresolved claims require recovery (#14): {release:#}"
                ))),
            },
            other => other,
        }
    }
    fn attempt_started_at(&self) -> i64 {
        #[cfg(test)]
        if let Some(at) = *self.hooks.attempt_started_at.lock().expect("test clock") {
            return at;
        }
        now_ms()
    }
    fn release_preparation(&self, claim: &PreparationClaim) -> Result<()> {
        let mut snapshot = self.read(claim.scope.task_id.context("claim requires Task")?)?;
        active(&snapshot.project, &snapshot.goal, &snapshot.task)?;
        ensure!(
            snapshot.task.scope() == claim.scope
                && snapshot.record.id == claim.record_id
                && snapshot.record.version == claim.record_version
                && snapshot.workflow.generation == claim.generation
                && snapshot.workflow.active == Some(claim.index)
                && snapshot.workflow.context_version == claim.context_version
                && claim
                    .worktree
                    .as_ref()
                    .is_none_or(|path| snapshot.task.worktree.as_ref() == Some(path))
                && claim
                    .branch
                    .as_ref()
                    .is_none_or(|branch| snapshot.task.branch.as_ref() == Some(branch)),
            "preparation claim changed"
        );
        let attempt = snapshot
            .workflow
            .history
            .get_mut(claim.index)
            .context("preparation attempt missing")?;
        ensure!(
            attempt.state == AttemptState::Running
                && attempt.session_id.is_none()
                && !attempt.dispatch_started
                && serde_json::to_value(&*attempt)? == serde_json::to_value(&claim.attempt)?,
            "preparation attempt no longer owned and undispatched"
        );
        let detail = "owned pre-dispatch preparation failed; no native dispatch";
        attempt.state = AttemptState::Failed;
        attempt.completed_at = Some(now_ms());
        attempt.detail = Some(detail.into());
        snapshot.workflow.retries.push(RetryEvent {
            prior_attempt: claim.index,
            reason: detail.into(),
            at: now_ms(),
        });
        snapshot.workflow.active = None;
        #[cfg(test)]
        if let Some(hook) = self
            .hooks
            .before_release
            .lock()
            .expect("release hook")
            .take()
        {
            hook();
        }
        // Fresh metadata is preserved; the existing immediate transaction checks
        // owners, Task/Record CAS and all native executor/Lost closing fences.
        self.persist(&mut snapshot, None)
    }
    fn preflight_native_adapter(
        &self,
        task: &Task,
        phase: Phase,
    ) -> Result<NativeAdapterSelection> {
        let (agent, needed) = match phase.actor() {
            Actor::Executor => (&task.executor, Capability::Execute),
            Actor::Reviewer => (
                task.reviewers
                    .first()
                    .ok_or(NativePreflightRefusal::MissingReviewer)?,
                Capability::Review,
            ),
            Actor::EvidencePort => anyhow::bail!("evidence phase does not select a native adapter"),
        };
        let adapter = self.registry.get(agent).map_err(|error| {
            anyhow::Error::new(error).context(NativePreflightRefusal::AdapterUnavailable)
        })?;
        // Registry lookup is internal metadata. Public trait callbacks (including
        // capabilities) have no effect-free seal, so the genuine static composition
        // check must precede every callback. Diagnostics below stay unreachable
        // until the implementation-owned joint protocol is installed.
        let _ = adapter.probe();
        require_managed_native_binding_composed()?;
        let capabilities = adapter.capabilities();
        for required in [needed, Capability::PreparedInputAdmission] {
            if !capabilities.contains(&required) {
                return Err(NativePreflightRefusal::MissingCapability(required).into());
            }
        }
        let info = adapter.probe().map_err(|error| {
            anyhow::Error::new(error).context(NativePreflightRefusal::ProbeFailed)
        })?;
        if info.agent != *agent || info.provider.trim().is_empty() {
            return Err(NativePreflightRefusal::IdentityMismatch.into());
        }
        for required in [needed, Capability::PreparedInputAdmission] {
            if !info.capabilities.contains(&required) {
                return Err(NativePreflightRefusal::MissingCapability(required).into());
            }
        }
        Ok(NativeAdapterSelection {
            adapter,
            agent: agent.clone(),
            provider: info.provider,
        })
    }
    async fn prepare_agent(
        &self,
        mut snapshot: Snapshot,
        preparation: AgentPreparation,
        claim: &PreparationClaim,
        eligible: &mut bool,
    ) -> Result<StepResult> {
        // No private caller may enter the old Task-writing binder through metadata.
        require_managed_native_binding_composed()?;
        let AgentPreparation {
            context,
            config,
            environment,
            selected,
        } = preparation;
        let index = claim.index;
        let phase = claim.attempt.phase;
        let class = snapshot.workflow.workflow;
        let (fresh_config, fresh_source, _) = self
            .inputs(&snapshot.project, &snapshot.task, phase, class)
            .await?;
        if !same_sources(&fresh_source, &snapshot.workflow.sources)
            || phases(class, &fresh_config)
                .iter()
                .any(|p| !snapshot.workflow.configured_phases.contains(p))
        {
            snapshot.workflow.history[index].state = AttemptState::Interrupted;
            snapshot.workflow.history[index].completed_at = Some(now_ms());
            snapshot.workflow.active = None;
            invalidate(
                &mut snapshot.workflow,
                &fresh_source,
                "workflow policy or target changed",
            )?;
            snapshot.workflow.sources = fresh_source.clone();
            let first = next_phase(&snapshot.workflow).expect("presets nonempty");
            let context = self
                .prepare_pack(
                    &snapshot.project,
                    &snapshot.task,
                    &fresh_source,
                    first,
                    class,
                    snapshot.workflow.generation,
                )
                .await?;
            set_context(&mut snapshot, &context);
            *eligible = false;
            self.persist(&mut snapshot, Some(&context))?;
            return Ok(StepResult::Invalidated {
                reason: "sources changed before native dispatch".into(),
            });
        }
        self.refresh_owners(&mut snapshot)?;
        let (_, observed, _) = self
            .inputs(&snapshot.project, &snapshot.task, phase, class)
            .await?;
        if !same_sources(&observed, &snapshot.workflow.sources) {
            return self
                .invalidate_attempt_preparation(
                    snapshot,
                    index,
                    observed,
                    "authority changed before native dispatch",
                    Some(eligible),
                )
                .await;
        }
        let selected_agent = if phase.actor() == Actor::Reviewer {
            snapshot.task.reviewers.first().cloned()
        } else {
            Some(snapshot.task.executor.clone())
        };
        ensure!(
            claim.attempt.agent == selected_agent
                && snapshot.task.worktree == claim.worktree
                && snapshot.task.branch == claim.branch,
            "reserved agent or Task binding changed during preparation"
        );
        ensure!(
            selected_agent.as_ref() == Some(&selected.agent),
            "selected adapter changed during preparation"
        );
        let NativeAdapterSelection {
            adapter,
            agent,
            provider,
        } = selected;
        let agent_config = config.agents.get(&agent);
        let input = PreparedInput {
            scope: snapshot.task.scope(),
            kind: if phase.actor() == Actor::Reviewer {
                InputKind::ReviewBundle
            } else {
                InputKind::ContextPack
            },
            revision: context.revision.clone(),
            version: context.version,
            source_versions: context.source_hashes.clone(),
            payload: serde_json::to_string(&context.data)?,
        };
        if adapter.managed_provider().is_some() {
            *eligible = false;
            return self
                .prepare_managed(snapshot, index, input, config, environment, adapter)
                .await;
        }
        let Some(worktree) = snapshot.task.worktree.clone() else {
            *eligible = false;
            return self.fail(
                snapshot,
                index,
                "agent phase requires bound worktree".into(),
            );
        };
        snapshot.workflow.history[index].dispatch_started = true;
        *eligible = false;
        if let Err(error) = self.persist(&mut snapshot, None) {
            *eligible = own_task_marker_rollback(&error, snapshot.task.id);
            return Err(error);
        }
        let request = LaunchRequest {
            project: snapshot.project.clone(),
            scope: snapshot.task.scope(),
            worktree: worktree.clone(),
            role: if phase.actor() == Actor::Reviewer {
                SessionRole::Reviewer
            } else {
                SessionRole::Executor
            },
            mode: LaunchMode::NonInteractive,
            input,
            environment,
            model: agent_config.and_then(|c| c.model.clone()),
            effort: agent_config.and_then(|c| c.effort.clone()),
        };
        match adapter.start(request).await {
            Ok(session) => {
                ensure!(
                    session.scope == snapshot.task.scope()
                        && session.agent == agent
                        && session.provider == provider
                        && session.worktree == worktree
                        && session.role
                            == if phase.actor() == Actor::Reviewer {
                                SessionRole::Reviewer
                            } else {
                                SessionRole::Executor
                            },
                    "adapter returned foreign session"
                );
                snapshot.workflow.history[index].session_id = Some(session.id);
                if let Some(unit) = self
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .session_execution_unit(session.id)?
                {
                    ensure!(
                        unit.scope == snapshot.task.scope(),
                        "foreign managed Session binding"
                    );
                    snapshot.workflow.history[index].execution =
                        Some(crate::execution::native::ManagedSessionRef {
                            scope: unit.scope,
                            unit: unit.id,
                            generation: unit.generation,
                            epoch: unit.owner_epoch,
                            session: session.id,
                        });
                }
                // Adapter may persist Session, never rewrite Task/history. CAS loss
                // preserves the reservation; #14 reconciles the durable Session.
                self.persist(&mut snapshot, None)?;
                Ok(StepResult::Started {
                    phase,
                    session: Some(session.id),
                })
            }
            Err(error) => self.fail(snapshot, index, error.to_string()),
        }
    }
    async fn prepare_managed(
        &self,
        mut snapshot: Snapshot,
        index: usize,
        input: PreparedInput,
        config: Config,
        environment: BTreeMap<String, String>,
        adapter: Arc<dyn crate::adapter::AgentAdapter>,
    ) -> Result<StepResult> {
        use crate::execution::{self, native::NativeStart};
        // Due-wait and private reentry must not bypass the missing joint protocol.
        require_managed_native_binding_composed()?;
        ensure!(
            environment.is_empty(),
            "managed native overrides require a qualified resource profile"
        );
        let owner = self
            .registry
            .managed_owner()
            .context("managed Runtime owner missing")?;
        let provider = adapter
            .managed_provider()
            .context("native provider missing")?;
        let phase = snapshot.workflow.history[index].phase;
        let agent = snapshot.workflow.history[index]
            .agent
            .clone()
            .context("native agent missing")?;
        let attempts = execution::attempts::AttemptManager::new(owner.clone());
        let already_reserved = snapshot.workflow.history[index].unit.is_some();
        let unit = if let Some(identity) = &snapshot.workflow.history[index].unit {
            let unit = self
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .execution_unit(identity.unit)?;
            ensure!(
                identity == &execution::ManagedUnitRef::from(&unit)
                    && unit.session_id.is_none()
                    && unit.provider == provider
                    && unit.phase == phase.key(),
                "managed preparation identity changed"
            );
            unit
        } else {
            let reservation = execution::model::WorkflowReservation {
                record: snapshot.record.id,
                version: snapshot.record.version,
                index,
                workflow_generation: snapshot.workflow.generation,
                context: input.version,
                project_version: snapshot.project.version,
                goal_version: snapshot.goal.version,
            };
            if phase.actor() == Actor::Executor {
                if let Some(prepared) = self
                    .sources
                    .take_initial_executor(
                        &snapshot.project,
                        &snapshot.task,
                        phase,
                        &snapshot.workflow.history[index].budget,
                    )
                    .await?
                {
                    prepared
                        .adopt(
                            snapshot.task.version,
                            &reservation,
                            phase.key(),
                            provider,
                            &input,
                        )
                        .await?
                } else {
                    attempts
                        .prepare_workflow(
                            snapshot.task.id,
                            provider,
                            phase.key(),
                            &input.revision,
                            &reservation,
                        )
                        .await?
                        .0
                }
            } else {
                let artifact = snapshot
                    .workflow
                    .sources
                    .artifact
                    .context("managed reviewer requires a retained artifact")?;
                let (unit, _) = attempts
                    .prepare_workflow_snapshot(
                        snapshot.task.id,
                        artifact,
                        provider,
                        phase.key(),
                        &reservation,
                    )
                    .await?;
                let input_snapshot = execution::results::ResultStore::new(owner.clone())
                    .snapshot(&unit)
                    .await?;
                self.managed_snapshots
                    .lock()
                    .map_err(|_| anyhow::anyhow!("snapshot state poisoned"))?
                    .insert(unit.id, input_snapshot);
                self.store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .execution_unit(unit.id)?
            }
        };
        let mut preparation = execution::owner::PreparationGuard::new(owner.clone(), &unit);
        // Unit identity and Task projection were committed with the exact phase
        // reservation before any preparation helper. Reload that transaction;
        // do not adopt a Unit discovered from a Task or recovery hint.
        if !already_reserved {
            snapshot = self.read(snapshot.task.id)?;
        }
        ensure!(
            snapshot.workflow.active == Some(index)
                && snapshot.workflow.history[index].unit.as_ref()
                    == Some(&execution::ManagedUnitRef::from(&unit)),
            "managed reservation changed during preparation"
        );
        let (_, sources, _) = self
            .inputs(
                &snapshot.project,
                &snapshot.task,
                phase,
                snapshot.workflow.workflow,
            )
            .await?;
        ensure!(
            same_sources(&sources, &snapshot.workflow.sources),
            "managed sources changed during preparation"
        );
        self.refresh_owners(&mut snapshot)?;
        if unit.kind != execution::UnitKind::Executor {
            let input_snapshot = self
                .managed_snapshots
                .lock()
                .map_err(|_| anyhow::anyhow!("snapshot state poisoned"))?
                .get(&unit.id)
                .cloned()
                .context("readonly preparation provenance unavailable; fresh retry required")?;
            input_snapshot.verify().await?;
        }
        let attempt = &mut snapshot.workflow.history[index];
        attempt.dispatch_started = true;
        attempt.native_wait = None;
        attempt.next_due = None;
        snapshot.task.state = phase.task_state();
        self.persist(&mut snapshot, None)?;
        let request = LaunchRequest {
            project: snapshot.project.clone(),
            scope: snapshot.task.scope(),
            worktree: unit.worktree.clone(),
            role: if phase.actor() == Actor::Reviewer {
                SessionRole::Reviewer
            } else {
                SessionRole::Executor
            },
            mode: LaunchMode::NonInteractive,
            input: input.clone(),
            environment,
            model: config.agents.get(&agent).and_then(|a| a.model.clone()),
            effort: config.agents.get(&agent).and_then(|a| a.effort.clone()),
        };
        let result = adapter
            .start_managed(
                request,
                execution::native::ManagedInput {
                    agent,
                    authority: unit.authority(),
                    artifact: unit.artifact_id,
                    input,
                },
            )
            .await;
        match result {
            Ok(NativeStart::Launched(identity)) => {
                ensure!(
                    execution::ManagedUnitRef {
                        scope: identity.scope.clone(),
                        unit: identity.unit,
                        generation: identity.generation,
                        epoch: identity.epoch
                    } == execution::ManagedUnitRef::from(&unit),
                    "native launch returned foreign execution identity"
                );
                snapshot.workflow.history[index].session_id = Some(identity.session);
                snapshot.workflow.history[index].execution = Some(identity.clone());
                self.refresh_owners(&mut snapshot)?;
                self.persist(&mut snapshot, None)?;
                preparation.disarm();
                Ok(StepResult::Started {
                    phase,
                    session: Some(identity.session),
                })
            }
            Ok(NativeStart::Waiting {
                unit: waiting,
                reason,
                next_due,
            }) => {
                ensure!(
                    execution::ManagedUnitRef::from(waiting.as_ref())
                        == execution::ManagedUnitRef::from(&unit)
                        && waiting.session_id.is_none(),
                    "foreign native wait identity"
                );
                let attempt = &mut snapshot.workflow.history[index];
                attempt.native_wait = Some(reason);
                attempt.next_due = Some(next_due);
                attempt.detail = Some(format!("native admission waiting: {reason:?}"));
                snapshot.task.state = native_wait_state(reason);
                self.refresh_owners(&mut snapshot)?;
                self.persist(&mut snapshot, None)?;
                preparation.disarm();
                Ok(StepResult::Waiting {
                    phase,
                    reason: format!("native admission waiting: {reason:?}"),
                })
            }
            Err(error) => {
                // The own preparation/native guards close abandoned authority.
                // Do not turn unknown native launch work into Task failure.
                drop(preparation);
                snapshot.workflow.history[index].detail =
                    Some(format!("native launch unavailable: {:?}", error.kind));
                snapshot.task.state = TaskState::WaitingHuman;
                self.refresh_owners(&mut snapshot)?;
                self.persist(&mut snapshot, None)?;
                Ok(StepResult::Waiting {
                    phase,
                    reason: format!(
                        "native launch unavailable: {:?}; retry requires explicit action",
                        error.kind
                    ),
                })
            }
        }
    }
    async fn poll(&self, snapshot: Snapshot, index: usize) -> Result<StepResult> {
        let attempt = snapshot
            .workflow
            .history
            .get(index)
            .context("invalid active phase index")?;
        let phase = attempt.phase;
        if attempt.state == AttemptState::Waiting {
            return Ok(StepResult::Waiting {
                phase,
                reason: attempt.detail.clone().unwrap_or_default(),
            });
        }
        if let Some(observation) = known_gate_observation(attempt)
            && let Some(outcome) = &observation.outcome
        {
            let source = observation.sources.clone();
            let outcome = outcome.clone();
            return self.apply_outcome(snapshot, index, source, outcome).await;
        }
        if matches!(
            attempt.state,
            AttemptState::Evaluating | AttemptState::Interrupted
        ) {
            return Ok(StepResult::Waiting {
                phase,
                reason: format!(
                    "evidence evaluation may still be active; unknown outcome requires explicit recovery ({})",
                    recovery_issue(phase)
                ),
            });
        }
        if attempt.state != AttemptState::Running {
            return Ok(StepResult::Failed {
                phase,
                reason: attempt.detail.clone().unwrap_or_default(),
            });
        }
        let Some(id) = attempt.session_id else {
            if phase.actor() == Actor::EvidencePort {
                return self.evaluate(snapshot, index, None).await;
            }
            if attempt.unit.is_some() && attempt.native_wait.is_some() {
                if attempt.next_due.is_some_and(|due| due > now_ms()) {
                    return Ok(StepResult::Waiting {
                        phase,
                        reason: attempt.detail.clone().unwrap_or_default(),
                    });
                }
                // Before clearing/claiming the due waiter or preparing any helper.
                let selected = self.preflight_native_adapter(&snapshot.task, phase)?;
                ensure!(
                    attempt.agent.as_deref() == Some(selected.agent.as_str()),
                    "managed waiting actor changed"
                );
                let adapter = selected.adapter;
                let context = self.context(&snapshot)?;
                let input = PreparedInput {
                    scope: context.scope.clone(),
                    kind: if phase.actor() == Actor::Reviewer {
                        InputKind::ReviewBundle
                    } else {
                        InputKind::ContextPack
                    },
                    revision: context.revision.clone(),
                    version: context.version,
                    source_versions: context.source_hashes.clone(),
                    payload: serde_json::to_string(&context.data)?,
                };
                #[cfg(test)]
                {
                    let barrier = self.hooks.before_wait_claim.lock().unwrap().take();
                    if let Some(barrier) = barrier {
                        barrier.await;
                    }
                }
                // Claim the exact due waiter before any async source/native work.
                // A stale poll never adopts the winner's record or arms its guard.
                let owner = self
                    .registry
                    .managed_owner()
                    .context("managed owner missing")?;
                let unit = self
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .execution_unit(attempt.unit.as_ref().unwrap().unit)?;
                let mut snapshot = snapshot;
                snapshot.workflow.history[index].native_wait = None;
                snapshot.workflow.history[index].next_due = None;
                if let Err(error) = self.persist(&mut snapshot, None) {
                    if error
                        .downcast_ref::<crate::state::StateGuardError>()
                        .is_some_and(|e| {
                            matches!(e, crate::state::StateGuardError::SnapshotChanged { .. })
                        })
                    {
                        return Ok(StepResult::Waiting {
                            phase,
                            reason: "native admission already claimed; poll current reservation"
                                .into(),
                        });
                    }
                    return Err(error);
                }
                let mut claim = crate::execution::owner::PreparationGuard::new(owner, &unit);
                let (config, _, _) = self
                    .inputs(
                        &snapshot.project,
                        &snapshot.task,
                        phase,
                        snapshot.workflow.workflow,
                    )
                    .await?;
                let result = self
                    .prepare_managed(snapshot, index, input, config, BTreeMap::new(), adapter)
                    .await;
                if result.is_ok() {
                    claim.disarm();
                }
                return result;
            }
            return Ok(StepResult::Waiting {
                phase,
                reason: if attempt.dispatch_started {
                    "native launch may still be active; unreconciled ownership requires recovery (#14)"
                } else {
                    "native preparation may still be active; abandoned ownership requires recovery (#14)"
                }.into(),
            });
        };
        let agent = attempt
            .agent
            .as_ref()
            .context("native attempt actor missing")?;
        let adapter = self.registry.get(agent)?;
        let status = match adapter
            .status(SessionRef {
                execution: attempt.execution.clone(),
                id,
                scope: snapshot.task.scope(),
            })
            .await
        {
            Ok(status) => status,
            Err(error) => {
                let mut snapshot = snapshot;
                let detail = format!("native status unavailable; recovery required: {error}");
                if snapshot.workflow.history[index].detail.as_ref() != Some(&detail) {
                    snapshot.workflow.history[index].detail = Some(detail);
                    self.refresh_owners(&mut snapshot)?;
                    self.persist(&mut snapshot, None)?;
                }
                return Ok(StepResult::Waiting {
                    phase,
                    reason: format!("native status unavailable; recovery required: {error}"),
                });
            }
        };
        validate_status(&status, &snapshot.task, attempt)?;
        if let Err(error) = self.validate_persisted_status(&status) {
            let mut snapshot = snapshot;
            let reason = format!("native status unavailable; recovery required: {error:#}");
            if snapshot.workflow.history[index].detail.as_ref() != Some(&reason) {
                snapshot.workflow.history[index].detail = Some(reason.clone());
                self.refresh_owners(&mut snapshot)?;
                self.persist(&mut snapshot, None)?;
            }
            return Ok(StepResult::Waiting { phase, reason });
        }
        if let Some(native) = &status.execution {
            use crate::execution::{Disposition, WorkOutcome};
            if !status.terminal() && native.wait_reason == Some(crate::execution::WaitReason::Quota)
            {
                let mut snapshot = snapshot;
                snapshot.task.state = TaskState::WaitingQuota;
                self.refresh_owners(&mut snapshot)?;
                self.persist(&mut snapshot, None)?;
                return Ok(StepResult::Waiting {
                    phase,
                    reason: "native turn is waiting; input has not been resent".into(),
                });
            }
            if status.terminal()
                && matches!(
                    native.disposition,
                    Disposition::QuotaInterrupted | Disposition::CapacityInterrupted
                )
            {
                let mut snapshot = snapshot;
                snapshot.workflow.history[index].state = AttemptState::Interrupted;
                snapshot.workflow.history[index].completed_at = Some(now_ms());
                snapshot.workflow.history[index].detail = Some(format!(
                    "native turn interrupted: {:?}; next attempt uses fresh resources",
                    native.disposition
                ));
                snapshot.workflow.retries.push(RetryEvent {
                    prior_attempt: index,
                    reason: "native capacity wait; work remains unknown".into(),
                    at: now_ms(),
                });
                snapshot.workflow.active = None;
                snapshot.task.state = if native.disposition == Disposition::QuotaInterrupted {
                    TaskState::WaitingQuota
                } else {
                    TaskState::WaitingCapacity
                };
                self.refresh_owners(&mut snapshot)?;
                self.persist(&mut snapshot, None)?;
                return Ok(StepResult::Waiting {
                    phase,
                    reason: "native capacity wait; next admission uses fresh resources".into(),
                });
            }
            if status.terminal() && native.work == Some(WorkOutcome::Unknown) {
                let mut snapshot = snapshot;
                snapshot.task.state = TaskState::WaitingHuman;
                snapshot.workflow.history[index].detail =
                    Some("native work unknown; explicit fresh retry required".into());
                self.refresh_owners(&mut snapshot)?;
                self.persist(&mut snapshot, None)?;
                return Ok(StepResult::Waiting {
                    phase,
                    reason: "native work unknown; explicit fresh retry required".into(),
                });
            }
            if status.terminal() && native.work == Some(WorkOutcome::Failure) {
                self.store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .close_result_as_draft(
                        &native.authority,
                        "known native failure has no publishable result",
                    )?;
            }
        }
        if !status.terminal() {
            if snapshot.task.state == TaskState::WaitingQuota
                && status
                    .execution
                    .as_ref()
                    .is_some_and(|native| native.wait_reason.is_none())
            {
                let mut snapshot = snapshot;
                snapshot.task.state = phase.task_state();
                self.refresh_owners(&mut snapshot)?;
                self.persist(&mut snapshot, None)?;
            }
            return Ok(StepResult::Running { phase, session: id });
        }
        if !adapter.transport_succeeded(&status) {
            return self.fail(
                snapshot,
                index,
                format!(
                    "agent transport {:?}: {:?}",
                    status.session.state, status.failure
                ),
            );
        }
        if let Some(identity) = &attempt.unit
            && phase.actor() == Actor::Reviewer
        {
            let input_snapshot = self
                .managed_snapshots
                .lock()
                .map_err(|_| anyhow::anyhow!("snapshot state poisoned"))?
                .get(&identity.unit)
                .cloned()
                .context("readonly result provenance unavailable; fresh retry required")?;
            input_snapshot.verify().await?;
        }
        self.evaluate(snapshot, index, Some(status)).await
    }
    fn validate_persisted_status(&self, status: &SessionStatus) -> Result<()> {
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state store poisoned"))?;
        let (saved, _) = store
            .session(status.session.id)?
            .context("native status requires persisted owned Session")?;
        ensure!(
            serde_json::to_value(saved)? == serde_json::to_value(&status.session)?,
            "native status differs from persisted owned Session"
        );
        if let Some(native) = &status.execution {
            let unit = store.execution_unit(native.handle.unit)?;
            ensure!(
                unit.scope == native.handle.scope
                    && unit.generation == native.handle.generation
                    && unit.owner_epoch == native.handle.epoch
                    && unit.session_id == Some(status.session.id)
                    && unit.worktree == status.session.worktree
                    && unit.provider == status.session.provider,
                "managed Session status differs from durable unit binding"
            );
        }
        Ok(())
    }
    fn observe_gate(
        &self,
        snapshot: &mut Snapshot,
        index: usize,
        observation: GateObservation,
    ) -> Result<()> {
        self.store
            .lock()
            .map_err(|_| anyhow::anyhow!("state store poisoned"))?
            .observe_workflow_gate(
                &mut snapshot.record,
                index,
                &snapshot.workflow.history[index],
                observation,
            )?;
        snapshot.workflow = serde_json::from_value(snapshot.record.data.clone())?;
        Ok(())
    }
    /// Audited caller decision. Native processes and unresolved gate reservations
    /// remain owned until their separate verified recovery/cleanup succeeds.
    pub fn cancel(&self, task_id: TaskId, reason: String) -> Result<()> {
        self.terminate(task_id, TaskState::Cancelled, reason)
    }
    pub fn fail_task(&self, task_id: TaskId, reason: String) -> Result<()> {
        self.terminate(task_id, TaskState::Failed, reason)
    }
    fn terminate(&self, task_id: TaskId, state: TaskState, reason: String) -> Result<()> {
        ensure!(
            !reason.trim().is_empty(),
            "terminal decision requires reason"
        );
        let mut snapshot = self.read(task_id)?;
        ensure!(
            snapshot.project.state != ProjectState::Removed,
            "Project removed"
        );
        ensure!(!task_terminal(snapshot.task.state), "Task terminal");
        ensure!(
            snapshot.workflow.terminal_decision.is_none(),
            "terminal decision already recorded"
        );
        snapshot.workflow.terminal_decision = Some(TerminalDecision {
            scope: snapshot.task.scope(),
            generation: snapshot.workflow.generation,
            attempt: snapshot.workflow.active,
            state,
            reason,
            at: now_ms(),
        });
        snapshot.task.state = state;
        self.persist_decision(&mut snapshot, WorkflowAccess::TerminalDecision)?;
        self.sources.retire_initial(&snapshot.task.scope())
    }
    fn persist_decision(&self, snapshot: &mut Snapshot, access: WorkflowAccess) -> Result<()> {
        snapshot.record.data = serde_json::to_value(&snapshot.workflow)?;
        self.store
            .lock()
            .map_err(|_| anyhow::anyhow!("state store poisoned"))?
            .put_workflow_transition(
                &mut snapshot.task,
                &mut snapshot.record,
                None,
                snapshot.project.version,
                snapshot.goal.version,
                access,
            )
    }
    /// Close only the cancelled/failed decision's reservation after the trusted
    /// Session store proves termination. An unknown external gate remains held
    /// for the explicit outcome reconciliation port in #13.
    pub fn release_terminal_reservation(&self, task_id: TaskId, reason: String) -> Result<()> {
        ensure!(!reason.trim().is_empty(), "recovery requires reason");
        let mut snapshot = self.read(task_id)?;
        ensure!(
            snapshot.workflow.terminal_decision.is_some(),
            "terminal decision required"
        );
        let index = snapshot
            .workflow
            .active
            .context("no terminal reservation")?;
        let attempt = &mut snapshot.workflow.history[index];
        ensure!(
            attempt.state != AttemptState::Evaluating || known_gate_observation(attempt).is_some(),
            "unknown {} outcome requires explicit recovery ({})",
            if irreversible(attempt.phase) {
                "external"
            } else {
                "reversible evaluation"
            },
            recovery_issue(attempt.phase)
        );
        ensure!(
            attempt.session_id.is_some() || !attempt.dispatch_started,
            "unbound native dispatch requires explicit recovery (#14)"
        );
        attempt.state = AttemptState::Interrupted;
        attempt.completed_at.get_or_insert_with(now_ms);
        attempt.detail = Some(format!("terminal reservation reconciled: {reason}"));
        snapshot.workflow.active = None;
        self.persist_decision(&mut snapshot, WorkflowAccess::TerminalRecovery)
    }
    /// QUICK ends at PR-created, which is nonterminal. Request the actual merge
    /// and cleanup gates; only their scoped evidence can make the Task complete.
    pub async fn request_finalization(
        &self,
        task_id: TaskId,
        reason: String,
    ) -> Result<WorkflowSnapshot> {
        ensure!(!reason.trim().is_empty(), "finalization requires reason");
        let mut snapshot = self.read(task_id)?;
        active(&snapshot.project, &snapshot.goal, &snapshot.task)?;
        ensure!(
            snapshot.workflow.finished
                && snapshot.workflow.workflow == WorkflowClass::Quick
                && snapshot.workflow.active.is_none()
                && snapshot.workflow.terminal_decision.is_none()
                && snapshot.workflow.completed.contains_key(&Phase::Pr),
            "finalization requires finished QUICK PR evidence"
        );
        let (_, source, _) = self
            .inputs(
                &snapshot.project,
                &snapshot.task,
                Phase::MergeGate,
                snapshot.workflow.workflow,
            )
            .await?;
        ensure!(
            same_sources(&source, &snapshot.workflow.sources),
            "finalization target changed; new reviewed Task required"
        );
        snapshot.workflow.configured_phases = retain_phases(
            &snapshot.workflow.configured_phases,
            vec![Phase::MergeGate, Phase::Cleanup],
        );
        snapshot.workflow.finished = false;
        snapshot.workflow.finalizations.push(FinalizationRequest {
            scope: snapshot.task.scope(),
            generation: snapshot.workflow.generation,
            reason,
            at: now_ms(),
        });
        let context = self
            .prepare_pack(
                &snapshot.project,
                &snapshot.task,
                &source,
                Phase::MergeGate,
                snapshot.workflow.workflow,
                snapshot.workflow.generation,
            )
            .await?;
        set_context(&mut snapshot, &context);
        snapshot.task.phase = Some(Phase::MergeGate.key().into());
        snapshot.task.state = Phase::MergeGate.task_state();
        self.persist(&mut snapshot, Some(&context))?;
        Ok(snapshot.workflow)
    }
    fn refresh_owners(&self, snapshot: &mut Snapshot) -> Result<()> {
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state store poisoned"))?;
        let (project, goal, mut task) = owners(&store, snapshot.task.id)?;
        active(&project, &goal, &task)?;
        ensure!(
            store
                .record(snapshot.record.id)?
                .is_some_and(|r| r.version == snapshot.record.version),
            "workflow changed during external operation"
        );
        // Workflow fields are authoritative, other fields are latest owner data.
        task.workflow = snapshot.task.workflow;
        task.risk = snapshot.task.risk;
        task.state = snapshot.task.state;
        task.phase = snapshot.task.phase.clone();
        task.context_version = snapshot.task.context_version;
        task.revision = snapshot.task.revision.clone();
        task.artifacts = snapshot.task.artifacts.clone();
        snapshot.project = project;
        snapshot.goal = goal;
        snapshot.task = task;
        Ok(())
    }
    async fn invalidate_attempt(
        &self,
        snapshot: Snapshot,
        index: usize,
        source: SourceSnapshot,
        reason: &str,
    ) -> Result<StepResult> {
        self.invalidate_attempt_preparation(snapshot, index, source, reason, None)
            .await
    }
    fn fence_managed_attempt(&self, snapshot: &Snapshot, index: usize) -> Result<()> {
        let attempt = &snapshot.workflow.history[index];
        if let Some(identity) = &attempt.unit {
            let mut store = self
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let unit = store.execution_unit(identity.unit)?;
            ensure!(
                identity == &crate::execution::ManagedUnitRef::from(&unit)
                    && attempt
                        .session_id
                        .is_none_or(|s| unit.session_id == Some(s)),
                "managed invalidation identity changed"
            );
            if unit.native_effects_open || unit.result_finalization_open {
                store.retire_execution(&unit.authority(), false)?;
            }
        }
        Ok(())
    }
    async fn invalidate_attempt_preparation(
        &self,
        mut snapshot: Snapshot,
        index: usize,
        source: SourceSnapshot,
        reason: &str,
        mut eligible: Option<&mut bool>,
    ) -> Result<StepResult> {
        if has_external_effect(&snapshot.workflow)
            || (irreversible(snapshot.workflow.history[index].phase)
                && !snapshot.workflow.history[index].observations.is_empty())
        {
            let phase = snapshot.workflow.history[index].phase;
            if let Some(eligible) = eligible.as_deref_mut() {
                *eligible = false;
            }
            return self.hold(
                snapshot,
                Some(index),
                phase,
                "external side effect held; reconcile explicitly (#13)",
            );
        }
        let (config, fresh, _) = self
            .inputs(
                &snapshot.project,
                &snapshot.task,
                snapshot.workflow.history[index].phase,
                snapshot.workflow.workflow,
            )
            .await?;
        ensure!(
            same_sources(&source, &fresh),
            "authority changed during invalidation"
        );
        let class = snapshot
            .workflow
            .workflow
            .max(config.minimum_workflow)
            .max(risk_workflow(&config, snapshot.workflow.risk));
        if class > snapshot.workflow.workflow {
            snapshot.workflow.escalations.push(Escalation {
                from: snapshot.workflow.workflow,
                to: class,
                reason: reason.into(),
                evidence: serde_json::to_string(&source.source_versions)?,
                at: now_ms(),
            });
        }
        snapshot.workflow.workflow = class;
        snapshot.task.workflow = class;
        snapshot.workflow.configured_phases =
            retain_phases(&snapshot.workflow.configured_phases, phases(class, &config));
        self.fence_managed_attempt(&snapshot, index)?;
        snapshot.workflow.history[index].state = AttemptState::Interrupted;
        snapshot.workflow.history[index].completed_at = Some(now_ms());
        snapshot.workflow.history[index].detail = Some(reason.into());
        snapshot.workflow.active = None;
        invalidate(&mut snapshot.workflow, &source, reason)?;
        snapshot.workflow.sources = source.clone();
        let phase = next_phase(&snapshot.workflow).context("workflow has no phases")?;
        let context = self
            .prepare_pack(
                &snapshot.project,
                &snapshot.task,
                &source,
                phase,
                snapshot.workflow.workflow,
                snapshot.workflow.generation,
            )
            .await?;
        set_context(&mut snapshot, &context);
        if let Some(eligible) = eligible {
            *eligible = false;
        }
        self.persist(&mut snapshot, Some(&context))?;
        Ok(StepResult::Invalidated {
            reason: reason.into(),
        })
    }
    /// Re-evaluate a known waiting gate without launching another native Session.
    /// Unknown irreversible evaluation needs outcome reconciliation (#13);
    /// reversible owner/restart claims need explicit recovery (#14).
    pub async fn resume_gate(&self, task_id: TaskId) -> Result<StepResult> {
        let snapshot = self.read(task_id)?;
        active(&snapshot.project, &snapshot.goal, &snapshot.task)?;
        let index = snapshot.workflow.active.context("no waiting attempt")?;
        let attempt = snapshot.workflow.history[index].clone();
        ensure!(
            attempt.state == AttemptState::Waiting
                || (irreversible(attempt.phase) && attempt.state == AttemptState::Failed),
            "only a definitive waiting gate or failed external gate can resume"
        );
        let status = if let Some(id) = attempt.session_id {
            let agent = self
                .registry
                .get(attempt.agent.as_deref().context("native actor missing")?)?;
            let status = agent
                .status(SessionRef {
                    execution: attempt.execution.clone(),
                    id,
                    scope: snapshot.task.scope(),
                })
                .await?;
            validate_status(&status, &snapshot.task, &attempt)?;
            self.validate_persisted_status(&status)?;
            ensure!(
                agent.transport_succeeded(&status),
                "native completion unavailable; recovery required"
            );
            Some(status)
        } else {
            None
        };
        self.evaluate(snapshot, index, status).await
    }
    fn hold(
        &self,
        mut snapshot: Snapshot,
        index: Option<usize>,
        phase: Phase,
        reason: &str,
    ) -> Result<StepResult> {
        if snapshot.workflow.held_reason.as_deref() != Some(reason) {
            self.refresh_owners(&mut snapshot)?;
            if let Some(index) = index
                && matches!(
                    snapshot.workflow.history[index].state,
                    AttemptState::Waiting | AttemptState::Failed
                )
            {
                remove_attempt_blocker(&mut snapshot.task, &snapshot.workflow.history[index]);
            }
            clear_hold(&mut snapshot);
            snapshot.workflow.held_reason = Some(reason.into());
            snapshot.task.state = TaskState::WaitingHuman;
            snapshot.task.blockers.push(reason.into());
            if let Some(index) = index {
                snapshot.workflow.history[index].detail = Some(reason.into());
            }
            self.persist(&mut snapshot, None)?;
        }
        Ok(StepResult::Waiting {
            phase,
            reason: reason.into(),
        })
    }
    fn fail(&self, mut snapshot: Snapshot, index: usize, reason: String) -> Result<StepResult> {
        let attempt = &mut snapshot.workflow.history[index];
        attempt.state = AttemptState::Failed;
        attempt.completed_at.get_or_insert_with(now_ms);
        attempt.detail = Some(reason.clone());
        let phase = attempt.phase;
        snapshot.task.state = TaskState::WaitingHuman;
        snapshot.task.blockers.push(reason.clone());
        self.persist(&mut snapshot, None)?;
        Ok(StepResult::Failed { phase, reason })
    }
    async fn evaluate(
        &self,
        mut snapshot: Snapshot,
        index: usize,
        status: Option<SessionStatus>,
    ) -> Result<StepResult> {
        self.refresh_owners(&mut snapshot)?;
        let phase = snapshot.workflow.history[index].phase;
        if matches!(
            snapshot.workflow.history[index].state,
            AttemptState::Waiting | AttemptState::Failed
        ) {
            remove_attempt_blocker(&mut snapshot.task, &snapshot.workflow.history[index]);
        }
        let (config, source, selected_budget) = if phase == Phase::Cleanup {
            (
                self.runtime.clone(),
                snapshot.workflow.sources.clone(),
                snapshot.workflow.history[index].budget.clone(),
            )
        } else {
            self.inputs(
                &snapshot.project,
                &snapshot.task,
                phase,
                snapshot.workflow.workflow,
            )
            .await?
        };
        let class = if phase == Phase::Cleanup {
            snapshot.workflow.workflow
        } else {
            snapshot
                .workflow
                .workflow
                .max(config.minimum_workflow)
                .max(risk_workflow(&config, snapshot.workflow.risk))
        };
        if phase != Phase::Cleanup
            && (has_external_effect(&snapshot.workflow)
                || (irreversible(phase)
                    && !snapshot.workflow.history[index].observations.is_empty()))
            && (class > snapshot.workflow.workflow
                || !same_sources(&source, &snapshot.workflow.sources))
        {
            return self.hold(
                snapshot,
                Some(index),
                phase,
                "external side effect held after policy/target drift; reconcile explicitly (#13)",
            );
        }
        if phase != Phase::Cleanup
            && (class > snapshot.workflow.workflow
                || retain_phases(&snapshot.workflow.configured_phases, phases(class, &config))
                    != snapshot.workflow.configured_phases
                || rules_changed(&source, &snapshot.workflow.sources))
        {
            let old = snapshot.workflow.workflow;
            snapshot.workflow.workflow = class;
            snapshot.task.workflow = class;
            snapshot.workflow.configured_phases =
                retain_phases(&snapshot.workflow.configured_phases, phases(class, &config));
            if class > old {
                snapshot.workflow.escalations.push(Escalation {
                    from: old,
                    to: class,
                    reason: "policy/rules changed during phase".into(),
                    evidence: serde_json::to_string(&source.source_versions)?,
                    at: now_ms(),
                });
            }
            self.fence_managed_attempt(&snapshot, index)?;
            snapshot.workflow.history[index].state = AttemptState::Interrupted;
            snapshot.workflow.history[index].completed_at = Some(now_ms());
            snapshot.workflow.active = None;
            invalidate(
                &mut snapshot.workflow,
                &source,
                "workflow policy or target changed",
            )?;
            snapshot.workflow.sources = source.clone();
            let first = next_phase(&snapshot.workflow).expect("presets nonempty");
            let context = self
                .prepare_pack(
                    &snapshot.project,
                    &snapshot.task,
                    &source,
                    first,
                    class,
                    snapshot.workflow.generation,
                )
                .await?;
            set_context(&mut snapshot, &context);
            self.persist(&mut snapshot, Some(&context))?;
            return Ok(StepResult::Invalidated {
                reason: "mandatory policy/rules changed during phase".into(),
            });
        }
        if !target_producing(phase) && !same_sources(&source, &snapshot.workflow.sources) {
            return self
                .invalidate_attempt(
                    snapshot,
                    index,
                    source,
                    "target changed before evidence invocation",
                )
                .await;
        }
        // Agent implementation may intentionally change revision/artifacts. The
        // gate must attest that observed target, not the obsolete launch revision.
        self.refresh_owners(&mut snapshot)?;
        // Recapture after owner refresh before any external side effect.
        let refreshed_source = if phase == Phase::Cleanup {
            source.clone()
        } else {
            self.inputs(
                &snapshot.project,
                &snapshot.task,
                phase,
                snapshot.workflow.workflow,
            )
            .await?
            .1
        };
        if !same_sources(&source, &refreshed_source) {
            return self
                .invalidate_attempt(
                    snapshot,
                    index,
                    refreshed_source,
                    "authority changed before gate claim",
                )
                .await;
        }
        if matches!(
            snapshot.workflow.history[index].state,
            AttemptState::Waiting | AttemptState::Failed
        ) {
            remove_attempt_blocker(&mut snapshot.task, &snapshot.workflow.history[index]);
        }
        snapshot.workflow.history[index].claimed_observations =
            snapshot.workflow.history[index].observations.len();
        snapshot.workflow.history[index].state = AttemptState::Evaluating;
        self.persist(&mut snapshot, None)?;
        let invocation = PhaseInvocation {
            project: snapshot.project.clone(),
            task: snapshot.task.clone(),
            phase,
            context: self.context(&snapshot)?,
            sources: source.clone(),
            budget: selected_budget.clone(),
            prerequisites: snapshot.workflow.completed.values().cloned().collect(),
            prior_observations: snapshot.workflow.history[index].observations.clone(),
        };
        let managed_contract = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .workflow_requires_verification(snapshot.record.id)?;
        let managed_contract = managed_contract || self.registry.managed_owner().is_some();
        let gate_result = if managed_contract && phase == Phase::Tests {
            if let Some(verifier) = &self.verifier {
                match verifier.evaluate(invocation).await {
                    Ok(result) => {
                        self.adopt_verification_successor(&mut snapshot, index, &result.successor)?;
                        snapshot.verification_completion = result.completion;
                        Ok(result.outcome)
                    }
                    Err(_) => Ok(GateOutcome::Waiting(
                        "managed Tests prerequisites or catalog unavailable; no command acceptance"
                            .into(),
                    )),
                }
            } else {
                Ok(GateOutcome::Waiting(
                    "managed Tests verifier integration not configured".into(),
                ))
            }
        } else if managed_contract
            && matches!(
                phase,
                Phase::ExpandedRegression | Phase::Mutation | Phase::Browser | Phase::Staging
            )
        {
            Ok(GateOutcome::Waiting(format!(
                "managed {} verifier integration pending",
                phase.key()
            )))
        } else {
            self.gates.complete(invocation, status).await
        };
        let outcome = match gate_result {
            Ok(outcome) => {
                self.observe_gate(
                    &mut snapshot,
                    index,
                    GateObservation {
                        sources: authority_only(&source),
                        outcome: Some(outcome.clone()),
                        error: None,
                        at: now_ms(),
                    },
                )?;
                outcome
            }
            Err(error) => {
                // External side effects are unknown: never infer process death or
                // retry permission from an integration error.
                self.observe_gate(
                    &mut snapshot,
                    index,
                    GateObservation {
                        sources: authority_only(&source),
                        outcome: None,
                        error: Some(format!("gate outcome unknown: {error:#}")),
                        at: now_ms(),
                    },
                )?;
                return Ok(StepResult::Waiting {
                    phase,
                    reason: format!("gate outcome unknown; explicit recovery required: {error:#}"),
                });
            }
        };
        self.apply_outcome(snapshot, index, source, outcome).await
    }
    fn adopt_verification_successor(
        &self,
        snapshot: &mut Snapshot,
        index: usize,
        successor: &Record,
    ) -> Result<()> {
        let next: WorkflowSnapshot = serde_json::from_value(successor.data.clone())?;
        let identity = next
            .history
            .get(index)
            .and_then(|a| a.unit.clone())
            .context("verification successor lacks actual unit")?;
        let mut expected = snapshot.record.clone();
        let mut workflow = snapshot.workflow.clone();
        ensure!(
            workflow.active == Some(index)
                && workflow.history[index].state == AttemptState::Evaluating
                && workflow.history[index].phase == Phase::Tests
                && workflow.history[index].unit.is_none(),
            "verification successor differs from original claim"
        );
        workflow.history[index].unit = Some(identity.clone());
        expected.data = serde_json::to_value(&workflow)?;
        expected.version = expected
            .version
            .checked_add(1)
            .context("verification successor version overflow")?;
        expected.updated_at = successor.updated_at;
        ensure!(
            serde_json::to_value(&expected)? == serde_json::to_value(successor)?,
            "verification successor changed fields outside private reservation"
        );
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        let actual = store
            .record(successor.id)?
            .context("verification successor missing")?;
        let unit = store.execution_unit(identity.unit)?;
        ensure!(
            serde_json::to_value(&actual)? == serde_json::to_value(successor)?
                && crate::execution::ManagedUnitRef::from(&unit) == identity
                && unit.scope == snapshot.task.scope()
                && unit.kind == crate::execution::UnitKind::Verifier
                && unit.provider == "verifier"
                && unit.phase == Phase::Tests.key()
                && unit.session_id.is_none()
                && serde_json::to_value(
                    store
                        .task(snapshot.task.id)?
                        .context("verification Task missing")?
                )? == serde_json::to_value(&snapshot.task)?,
            "verification successor current ledger/Task identity changed"
        );
        snapshot.record = successor.clone();
        snapshot.workflow = next;
        Ok(())
    }
    async fn apply_outcome(
        &self,
        mut snapshot: Snapshot,
        index: usize,
        source: SourceSnapshot,
        outcome: GateOutcome,
    ) -> Result<StepResult> {
        let phase = snapshot.workflow.history[index].phase;
        let source = if phase == Phase::Cleanup {
            snapshot.workflow.sources.clone()
        } else {
            source
        };
        let selected_budget = snapshot.workflow.history[index].budget.clone();
        self.refresh_owners(&mut snapshot)?;
        let final_source = if phase == Phase::Cleanup {
            source.clone()
        } else {
            self.inputs(
                &snapshot.project,
                &snapshot.task,
                phase,
                snapshot.workflow.workflow,
            )
            .await?
            .1
        };
        if irreversible(phase) && !same_sources(&source, &final_source) {
            return self.hold(
                snapshot,
                Some(index),
                phase,
                "observed external side effect held after source drift; reconcile explicitly (#13)",
            );
        }
        if !target_producing(phase) && !same_sources(&source, &final_source) {
            return self
                .invalidate_attempt(
                    snapshot,
                    index,
                    final_source,
                    "sources changed while gate evaluated",
                )
                .await;
        }
        if rules_changed(&source, &final_source) {
            return self
                .invalidate_attempt(
                    snapshot,
                    index,
                    final_source,
                    "mandatory policy/rules changed while gate evaluated",
                )
                .await;
        }
        let source = final_source;
        clear_hold(&mut snapshot);
        snapshot.workflow.context_fresh = same_sources(&source, &snapshot.workflow.sources);
        match outcome {
            GateOutcome::Waiting(reason) => {
                snapshot.workflow.context_fresh = same_sources(&source, &snapshot.workflow.sources);
                let attempt = &mut snapshot.workflow.history[index];
                attempt.state = AttemptState::Waiting;
                attempt.detail = Some(reason.clone());
                snapshot.task.state = TaskState::WaitingHuman;
                snapshot.task.blockers.push(reason.clone());
                self.persist(&mut snapshot, None)?;
                Ok(StepResult::Waiting { phase, reason })
            }
            GateOutcome::Failed(reason) => self.fail(snapshot, index, reason),
            GateOutcome::Passed(evidence) => {
                if phase.actor() == Actor::Reviewer
                    && let Some(identity) = &snapshot.workflow.history[index].unit
                {
                    let input_snapshot = self
                        .managed_snapshots
                        .lock()
                        .map_err(|_| anyhow::anyhow!("snapshot state poisoned"))?
                        .get(&identity.unit)
                        .cloned()
                        .context("readonly result provenance unavailable; fresh retry required")?;
                    input_snapshot.verify().await?;
                }
                if let Some(identity) = &snapshot.workflow.history[index].unit {
                    let store = self
                        .store
                        .lock()
                        .map_err(|_| anyhow::anyhow!("state poisoned"))?;
                    let unit = store.execution_unit(identity.unit)?;
                    ensure!(
                        identity == &crate::execution::ManagedUnitRef::from(&unit),
                        "gate managed unit identity changed"
                    );
                    store.validate_execution(&unit.authority(), false, true)?;
                }
                if let Err(error) = validate_evidence(
                    &evidence,
                    &snapshot.task.scope(),
                    &source,
                    &snapshot.workflow.history[index],
                ) {
                    return self.fail(snapshot, index, error.to_string());
                }
                if snapshot.workflow.completed.values().any(|approved| {
                    approved.phase.actor() == Actor::Reviewer
                        && approved
                            .dependencies
                            .iter()
                            .any(|(key, value)| source.source_versions.get(key) != Some(value))
                }) {
                    if irreversible(phase) {
                        return self.hold(snapshot, Some(index), phase, "observed external side effect held after approved dependency drift; reconcile explicitly (#13)");
                    }
                    return self
                        .invalidate_attempt(
                            snapshot,
                            index,
                            source,
                            "approved artifact dependency changed",
                        )
                        .await;
                }
                snapshot
                    .task
                    .artifacts
                    .extend(evidence.artifacts.iter().cloned());
                snapshot.workflow.completed.insert(phase, evidence);
                let attempt = &mut snapshot.workflow.history[index];
                attempt.state = AttemptState::Succeeded;
                attempt.completed_at.get_or_insert_with(now_ms);
                snapshot.workflow.active = None;
                snapshot.workflow.sources = source.clone();
                let next = next_phase(&snapshot.workflow);
                let context = if phase == Phase::Cleanup {
                    make_context(
                        &snapshot.task,
                        &source,
                        phase,
                        snapshot.workflow.workflow,
                        snapshot.workflow.generation,
                        selected_budget,
                        self.next_context(&snapshot.task.scope())?,
                    )
                } else {
                    self.prepare_pack(
                        &snapshot.project,
                        &snapshot.task,
                        &source,
                        next.unwrap_or(phase),
                        snapshot.workflow.workflow,
                        snapshot.workflow.generation,
                    )
                    .await?
                };
                set_context(&mut snapshot, &context);
                snapshot.task.phase = next.map(|p| p.key().into());
                snapshot.task.state = phase.task_state();
                snapshot.workflow.finished = next.is_none();
                if phase == Phase::Pr {
                    snapshot.task.state = TaskState::PrCreated;
                }
                if phase == Phase::Cleanup {
                    snapshot.task.state = TaskState::Completed;
                }
                if phase.actor() == Actor::Executor
                    && let Some(session) = snapshot.workflow.history[index].session_id
                {
                    let unit = self
                        .store
                        .lock()
                        .map_err(|_| anyhow::anyhow!("state poisoned"))?
                        .session_execution_unit(session)?;
                    if let Some(unit) = unit {
                        let owner = self
                            .registry
                            .managed_owner()
                            .context("managed publication Runtime missing")?;
                        let evidence = snapshot
                            .workflow
                            .completed
                            .get(&phase)
                            .context("publication evidence missing")?;
                        let markers = evidence
                            .artifacts
                            .iter()
                            .filter_map(|s| s.strip_prefix("rrx-artifact:"))
                            .collect::<Vec<_>>();
                        ensure!(
                            markers.len() == 1,
                            "managed completion requires one retained artifact"
                        );
                        snapshot.publication = Some(
                            crate::execution::results::ResultStore::new(owner)
                                .workflow_publication(&unit.authority(), markers[0].parse()?)
                                .await?,
                        );
                    }
                }
                if phase.actor() == Actor::Reviewer
                    && let Some(identity) = &snapshot.workflow.history[index].unit
                {
                    let provenance = self
                        .managed_snapshots
                        .lock()
                        .map_err(|_| anyhow::anyhow!("snapshot provenance poisoned"))?
                        .get(&identity.unit)
                        .cloned()
                        .context("readonly snapshot provenance missing")?;
                    snapshot.readonly_completion = Some(provenance.completion().await?);
                }
                #[cfg(test)]
                if (snapshot.publication.is_some() || snapshot.readonly_completion.is_some())
                    && let Some(hook) = self
                        .hooks
                        .before_publication
                        .lock()
                        .expect("publication hook")
                        .take()
                {
                    hook();
                }
                self.persist(&mut snapshot, Some(&context))?;
                Ok(StepResult::Completed { phase })
            }
        }
    }
    /// Resume only a resolved waiting/failed attempt. Native Lost/live executor
    /// reservations remain authoritative and reject subsequent launch.
    pub fn retry(&self, task_id: TaskId, reason: String) -> Result<()> {
        ensure!(!reason.trim().is_empty(), "retry needs reason");
        let mut snapshot = self.read(task_id)?;
        active(&snapshot.project, &snapshot.goal, &snapshot.task)?;
        let index = snapshot
            .workflow
            .active
            .context("no active attempt to retry")?;
        if let Some(identity) = &snapshot.workflow.history[index].unit {
            ensure!(
                !irreversible(snapshot.workflow.history[index].phase)
                    && snapshot.workflow.history[index]
                        .observations
                        .iter()
                        .all(|o| o.outcome.is_some()),
                "unknown evidence/external outcome requires reconciliation, not native retry"
            );
            let mut unit = self
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .execution_unit(identity.unit)?;
            ensure!(
                identity == &crate::execution::ManagedUnitRef::from(&unit)
                    && !unit.native_effects_open,
                "live managed authority requires cancellation before retry"
            );
            if unit.result_finalization_open && unit.work.is_some() {
                let mut store = self
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?;
                store.close_result_as_draft(
                    &unit.authority(),
                    "explicit retry retains the prior result as a draft",
                )?;
                unit = store.execution_unit(unit.id)?;
            }
            ensure!(
                identity == &crate::execution::ManagedUnitRef::from(&unit)
                    && !unit.native_effects_open
                    && !unit.result_finalization_open
                    && snapshot.workflow.history[index]
                        .session_id
                        .is_none_or(|s| unit.session_id == Some(s)),
                "managed retry requires retired exact native authority"
            );
            if snapshot.workflow.history[index].state != AttemptState::Failed {
                snapshot.workflow.history[index].state = AttemptState::Interrupted;
            }
            snapshot.workflow.history[index]
                .completed_at
                .get_or_insert_with(now_ms);
            snapshot.workflow.history[index].detail = Some(reason.clone());
            snapshot.workflow.retries.push(RetryEvent {
                prior_attempt: index,
                reason,
                at: now_ms(),
            });
            snapshot.workflow.active = None;
            snapshot.task.state = snapshot.workflow.history[index].phase.task_state();
            return self.persist(&mut snapshot, None);
        }
        ensure!(
            matches!(
                snapshot.workflow.history[index].state,
                AttemptState::Waiting | AttemptState::Failed
            ),
            "running/interrupted phase requires recovery"
        );
        let attempt = &snapshot.workflow.history[index];
        ensure!(
            !(attempt.phase.actor() != Actor::EvidencePort
                && attempt.dispatch_started
                && attempt.session_id.is_none()),
            UNBOUND_NATIVE_RECOVERY_REQUIRED
        );
        ensure!(
            !irreversible(snapshot.workflow.history[index].phase),
            "external side effect must resume/reconcile the same attempt, not retry"
        );
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state store poisoned"))?;
        for record in store.records(&snapshot.task.scope(), RecordKind::Session)? {
            let session: Session = serde_json::from_value(record.data)?;
            ensure!(
                !crate::git::executor_reserved(&session)
                    && session.state != SessionState::Lost
                    && (snapshot.workflow.history[index].session_id != Some(session.id)
                        || matches!(
                            session.state,
                            SessionState::Exited | SessionState::Stopped | SessionState::Failed
                        )),
                "reserved/Lost session requires verified recovery"
            );
        }
        drop(store);
        snapshot.workflow.retries.push(RetryEvent {
            prior_attempt: index,
            reason,
            at: now_ms(),
        });
        snapshot.workflow.history[index]
            .completed_at
            .get_or_insert_with(now_ms);
        snapshot.workflow.active = None;
        remove_attempt_blocker(&mut snapshot.task, &snapshot.workflow.history[index]);
        self.persist(&mut snapshot, None)
    }
}
pub(crate) fn known_gate_observation(attempt: &PhaseAttempt) -> Option<&GateObservation> {
    (attempt.state == AttemptState::Evaluating
        && attempt.observations.len() == attempt.claimed_observations.checked_add(1)?)
    .then(|| attempt.observations.get(attempt.claimed_observations))
    .flatten()
    .filter(|entry| entry.outcome.is_some())
}
fn clear_hold(snapshot: &mut Snapshot) {
    if let Some(reason) = snapshot.workflow.held_reason.take()
        && snapshot
            .task
            .blockers
            .iter()
            .filter(|b| *b == &reason)
            .count()
            == 1
    {
        snapshot.task.blockers.retain(|b| b != &reason);
    }
}
fn authority_only(source: &SourceSnapshot) -> SourceSnapshot {
    SourceSnapshot {
        payload: String::new(),
        ..source.clone()
    }
}
fn has_external_effect(snapshot: &WorkflowSnapshot) -> bool {
    snapshot.completed.keys().any(|phase| irreversible(*phase))
}
fn irreversible(phase: Phase) -> bool {
    matches!(phase, Phase::Pr | Phase::MergeGate | Phase::Cleanup)
}
fn target_producing(phase: Phase) -> bool {
    matches!(
        phase,
        Phase::Issue
            | Phase::Worktree
            | Phase::Requirements
            | Phase::RequirementsCommit
            | Phase::Design
            | Phase::DesignCommit
            | Phase::Implement
            | Phase::ImpactAnalysis
            | Phase::Commit
    )
}
fn remove_attempt_blocker(task: &mut Task, attempt: &PhaseAttempt) {
    if let Some(detail) = &attempt.detail
        && task
            .blockers
            .iter()
            .filter(|blocker| *blocker == detail)
            .count()
            == 1
        && let Some(index) = task.blockers.iter().rposition(|blocker| blocker == detail)
    {
        task.blockers.remove(index);
    }
}
fn validate_status(status: &SessionStatus, task: &Task, attempt: &PhaseAttempt) -> Result<()> {
    ensure!(
        Some(status.session.id) == attempt.session_id
            && status.session.scope == task.scope()
            && status.session.agent
                == *attempt
                    .agent
                    .as_ref()
                    .context("native attempt actor missing")?
            && (if let Some(identity) = &attempt.execution {
                status.execution.as_ref().is_some_and(|native| {
                    &native.handle == identity
                        && serde_json::to_value(&native.session).ok()
                            == serde_json::to_value(&status.session).ok()
                })
            } else {
                Some(&status.session.worktree) == task.worktree.as_ref()
            })
            && status.session.role
                == if attempt.phase.actor() == Actor::Reviewer {
                    SessionRole::Reviewer
                } else {
                    SessionRole::Executor
                },
        "foreign session status"
    );
    Ok(())
}
fn validate_evidence(
    evidence: &Evidence,
    scope: &Scope,
    source: &SourceSnapshot,
    attempt: &PhaseAttempt,
) -> Result<()> {
    ensure!(
        evidence.scope == *scope
            && evidence.phase == attempt.phase
            && evidence.revision == source.revision
            && evidence.source_versions == source.source_versions
            && !evidence.artifacts.is_empty()
            && evidence.artifacts.iter().all(|p| !p.trim().is_empty()),
        "gate evidence is foreign/stale/incomplete"
    );
    ensure!(
        evidence.session_id == attempt.session_id
            && evidence.context_version == attempt.context_version,
        "gate session binding mismatch"
    );
    ensure!(
        evidence
            .dependencies
            .iter()
            .all(|(k, v)| source.source_versions.get(k) == Some(v))
            && source
                .source_versions
                .iter()
                .filter(|(k, _)| k.starts_with("rules:"))
                .all(|(k, v)| evidence.dependencies.get(k) == Some(v)),
        "gate dependencies are foreign or omit mandatory rules"
    );
    if attempt.phase.actor() == Actor::Reviewer {
        ensure!(
            evidence.review_approved == Some(true),
            "review requires explicit approved verdict"
        );
        ensure!(
            evidence
                .dependencies
                .keys()
                .any(|k| !k.starts_with("rules:")),
            "review must bind approved artifact dependencies"
        );
    }
    Ok(())
}
pub(crate) fn validate_context(
    task: &Task,
    record: &Record,
    context: &ContextVersion,
) -> Result<()> {
    let workflow: WorkflowSnapshot = serde_json::from_value(record.data.clone())?;
    ensure!(
        context.scope == task.scope()
            && context.version == task.context_version
            && task.revision.as_ref() == Some(&context.revision),
        "Workflow ContextVersion ownership/revision mismatch"
    );
    let authority = context
        .source_hashes
        .iter()
        .filter(|(k, _)| !k.starts_with("workflow:"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<BTreeMap<_, _>>();
    ensure!(
        authority == workflow.sources.source_versions
            && context.source_hashes.get("workflow:artifact")
                == workflow
                    .sources
                    .artifact
                    .as_ref()
                    .map(|a| a.to_string())
                    .as_ref()
            && context.source_hashes.get("workflow:generation")
                == Some(&workflow.generation.to_string()),
        "Workflow ContextVersion source/generation mismatch"
    );
    let phase = workflow
        .active
        .map(|i| workflow.history[i].phase)
        .or_else(|| next_phase(&workflow))
        .or_else(|| workflow.configured_phases.last().copied())
        .context("workflow has no phases")?;
    ensure!(
        context
            .source_hashes
            .get("workflow:phase")
            .is_some_and(|p| p == phase.key()),
        "Workflow ContextVersion phase mismatch"
    );
    Ok(())
}
pub(crate) fn validate_transition(
    task: &Task,
    record: &Record,
    previous: Option<&Record>,
) -> Result<()> {
    let next: WorkflowSnapshot =
        serde_json::from_value(record.data.clone()).context("invalid Workflow authority")?;
    ensure!(
        next.workflow == task.workflow
            && next.risk == task.risk
            && next.context_version == task.context_version
            && next.sources.scope == task.scope()
            && task.revision.as_ref() == Some(&next.sources.revision),
        "Workflow authority differs from Task/context ownership"
    );
    ensure!(
        next.active.is_none_or(|i| i < next.history.len()),
        "invalid workflow active index"
    );
    ensure!(
        !next.configured_phases.is_empty()
            && next.configured_phases.windows(2).all(|p| p[0] < p[1])
            && phases(next.workflow, &Config::default())
                .iter()
                .all(|p| next.configured_phases.contains(p)),
        "workflow mandatory preset/order differs"
    );
    ensure!(
        next.finished == next_phase(&next).is_none(),
        "finished requires all configured gates"
    );
    if task_terminal(task.state) {
        let cancelled = next.terminal_decision.as_ref().is_some_and(|decision| {
            decision.scope == task.scope()
                && decision.generation == next.generation
                && (decision.attempt == next.active
                    || (next.active.is_none()
                        && decision.attempt.is_some_and(|i| {
                            next.history
                                .get(i)
                                .is_some_and(|a| a.state == AttemptState::Interrupted)
                        })))
                && decision.state == task.state
                && matches!(task.state, TaskState::Cancelled | TaskState::Failed)
                && !decision.reason.trim().is_empty()
        });
        ensure!(
            cancelled
                || (task.state == TaskState::Completed
                    && next.finished
                    && next.completed.contains_key(&Phase::MergeGate)
                    && next.completed.contains_key(&Phase::Cleanup)),
            "terminal Task requires audited cancellation/failure or scoped merge and cleanup evidence"
        );
    }
    if previous.is_none() {
        ensure!(
            next.generation == 1
                && next.history.is_empty()
                && next.completed.is_empty()
                && next.escalations.is_empty()
                && next.retries.is_empty()
                && next.invalidations.is_empty()
                && next.terminal_decision.is_none()
                && next.held_reason.is_none()
                && next.finalizations.is_empty()
                && next.active.is_none()
                && !next.finished,
            "initial workflow must contain no invented history/evidence"
        );
    }
    if let Some(previous) = previous {
        let old: WorkflowSnapshot = serde_json::from_value(previous.data.clone())?;
        ensure!(
            next.workflow >= old.workflow
                && next.generation >= old.generation
                && next.generation <= old.generation.saturating_add(1),
            "workflow downgrade/generation rewrite forbidden"
        );
        ensure!(
            risk_max(next.risk, old.risk) == next.risk,
            "risk downgrade forbidden"
        );
        ensure!(
            old.configured_phases
                .iter()
                .all(|p| next.configured_phases.contains(p)),
            "mandatory gate removal forbidden"
        );
        ensure!(
            next.history.len() >= old.history.len(),
            "phase history truncation forbidden"
        );
        for (index, before) in old.history.iter().enumerate() {
            let after = &next.history[index];
            if old.active == Some(index) {
                ensure!(
                    before.phase == after.phase
                        && before.generation == after.generation
                        && before.context_version == after.context_version
                        && before.budget == after.budget
                        && before.started_at == after.started_at
                        && before.agent == after.agent
                        && before
                            .unit
                            .as_ref()
                            .is_none_or(|id| after.unit.as_ref() == Some(id))
                        && before
                            .execution
                            .as_ref()
                            .is_none_or(|identity| after.execution.as_ref() == Some(identity))
                        && before
                            .session_id
                            .is_none_or(|id| after.session_id == Some(id)),
                    "active attempt identity is immutable"
                );
                ensure!(
                    before
                        .completed_at
                        .is_none_or(|time| after.completed_at == Some(time)),
                    "attempt completion time is immutable"
                );
                ensure!(
                    before.session_id.is_some()
                        || after.session_id.is_none()
                        || (before.state == AttemptState::Running
                            && after.state == AttemptState::Running
                            && after.phase.actor() != Actor::EvidencePort),
                    "Session may only bind during native launch"
                );
                ensure!(
                    before.unit.is_some()
                        || after.unit.is_none()
                        || (before.state == AttemptState::Running
                            && after.state == AttemptState::Running
                            && before.session_id.is_none()
                            && !before.dispatch_started
                            && after.phase.actor() != Actor::EvidencePort)
                        || (before.state == AttemptState::Evaluating
                            && after.state == AttemptState::Evaluating
                            && before.phase == Phase::Tests
                            && before.session_id.is_none()
                            && before.execution.is_none()
                            && !before.dispatch_started),
                    "managed unit may only bind before native dispatch or private Tests reservation"
                );
                ensure!(
                    before.execution.is_some()
                        || after.execution.is_none()
                        || (before.session_id.is_none()
                            && before.state == AttemptState::Running
                            && after.state == AttemptState::Running
                            && after.dispatch_started),
                    "managed identity may only bind with native launch"
                );
                ensure!(
                    !before.dispatch_started || after.dispatch_started,
                    "native dispatch marker is immutable"
                );
                ensure!(
                    before.dispatch_started == after.dispatch_started
                        || (before.state == AttemptState::Running
                            && after.state == AttemptState::Running
                            && before.session_id.is_none()
                            && before.phase.actor() != Actor::EvidencePort),
                    "native dispatch marker may only precede launch"
                );
                ensure!(
                    serde_json::to_value(&after.observations)?
                        == serde_json::to_value(&before.observations)?,
                    "gate observations require audited observer"
                );
                ensure!(
                    after.claimed_observations
                        == if before.state != AttemptState::Evaluating
                            && after.state == AttemptState::Evaluating
                        {
                            before.observations.len()
                        } else {
                            before.claimed_observations
                        }
                        && after.observations.len() <= after.claimed_observations.saturating_add(1),
                    "evaluation claim/observation round mismatch"
                );
                ensure!(
                    before.state != AttemptState::Evaluating
                        || after.state == AttemptState::Evaluating
                        || known_gate_observation(before).is_some(),
                    "unknown gate reservation requires explicit recovery"
                );
                let valid = match before.state {
                    AttemptState::Running => matches!(
                        after.state,
                        AttemptState::Running
                            | AttemptState::Evaluating
                            | AttemptState::Failed
                            | AttemptState::Interrupted
                    ),
                    AttemptState::Waiting => matches!(
                        after.state,
                        AttemptState::Waiting
                            | AttemptState::Evaluating
                            | AttemptState::Interrupted
                    ),
                    AttemptState::Evaluating => matches!(
                        after.state,
                        AttemptState::Evaluating
                            | AttemptState::Waiting
                            | AttemptState::Failed
                            | AttemptState::Succeeded
                            | AttemptState::Interrupted
                    ),
                    AttemptState::Failed
                        if next
                            .terminal_decision
                            .as_ref()
                            .is_some_and(|d| d.attempt == Some(index)) =>
                    {
                        after.state == AttemptState::Failed
                            || after.state == AttemptState::Interrupted
                    }
                    AttemptState::Failed if irreversible(before.phase) => matches!(
                        after.state,
                        AttemptState::Failed | AttemptState::Evaluating | AttemptState::Interrupted
                    ),
                    _ => before.state == after.state,
                };
                ensure!(valid, "attempt state regression forbidden");
            } else {
                ensure!(
                    serde_json::to_value(before)? == serde_json::to_value(after)?,
                    "completed phase history is immutable"
                );
            }
        }
        ensure!(
            next.history.len() <= old.history.len() + 1,
            "only one phase may be reserved per transition"
        );
        if next.history.len() > old.history.len() {
            let attempt = next.history.last().expect("new attempt");
            ensure!(
                old.active.is_none()
                    && next.active == Some(old.history.len())
                    && attempt.phase
                        == next_phase(&old).context("finished workflow cannot launch")?
                    && attempt.generation == next.generation
                    && attempt.context_version == next.context_version
                    && attempt.state == AttemptState::Running
                    && attempt.session_id.is_none()
                    && !attempt.dispatch_started
                    && attempt.observations.is_empty()
                    && attempt.claimed_observations == 0
                    && attempt.completed_at.is_none()
                    && attempt.detail.is_none(),
                "new attempt must reserve the next phase"
            );
        }
        if let Some(before) = &old.terminal_decision {
            ensure!(
                serde_json::to_value(before)? == serde_json::to_value(&next.terminal_decision)?,
                "terminal decision is immutable"
            );
        }
        if next.generation > old.generation {
            ensure!(
                !has_external_effect(&old)
                    && !old
                        .active
                        .is_some_and(|i| irreversible(old.history[i].phase)
                            && !old.history[i].observations.is_empty()),
                "external side effect cannot be invalidated into a duplicate operation"
            );
            if let Some(index) = old.active {
                ensure!(
                    next.history[index].state == AttemptState::Interrupted,
                    "invalidation must interrupt the old active attempt"
                );
                ensure!(
                    old.history[index].state != AttemptState::Evaluating
                        || known_gate_observation(&old.history[index]).is_some(),
                    "unknown gate reservation requires explicit recovery"
                );
            }
            ensure!(
                next.completed.is_empty()
                    && next.active.is_none()
                    && next.invalidations.len() == old.invalidations.len() + 1,
                "new generation must invalidate evidence with durable cause"
            );
        } else {
            ensure!(
                next.invalidations.len() == old.invalidations.len(),
                "invalidation requires generation change"
            );
            for (phase, before) in &old.completed {
                ensure!(
                    next.completed
                        .get(phase)
                        .is_some_and(|after| serde_json::to_value(before).ok()
                            == serde_json::to_value(after).ok()),
                    "completed evidence is immutable within a generation"
                );
            }
            let additions = next
                .completed
                .iter()
                .filter(|(phase, _)| !old.completed.contains_key(phase))
                .collect::<Vec<_>>();
            ensure!(
                additions.len() <= 1,
                "one evaluation can complete only its own phase"
            );
            if !same_sources(&old.sources, &next.sources) {
                ensure!(
                    additions.len() == 1
                        && additions
                            .first()
                            .is_some_and(|(phase, _)| target_producing(**phase)),
                    "same-generation authority change requires target-producing completion"
                );
            }
            if let Some((phase, evidence)) = additions.first() {
                let index = old
                    .active
                    .context("completion requires active evaluation")?;
                let before = &old.history[index];
                let after = &next.history[index];
                ensure!(
                    before.state == AttemptState::Evaluating
                        && after.state == AttemptState::Succeeded
                        && after.generation == next.generation
                        && after.phase == **phase
                        && next.active.is_none()
                        && after.completed_at.is_some()
                        && (after.phase.actor() == Actor::EvidencePort
                            || after.session_id.is_some()),
                    "completion requires Evaluating to Succeeded with owned native Session"
                );
                ensure!(known_gate_observation(before).is_some_and(|o| matches!(&o.outcome,
                    Some(GateOutcome::Passed(observed)) if serde_json::to_value(observed).ok() == serde_json::to_value(evidence).ok())),
                    "completion requires actual current-claim Passed observation");
                validate_evidence(evidence, &task.scope(), &next.sources, after)?;
            }
            if let Some(index) = old.active
                && next.history[index].state == AttemptState::Succeeded
            {
                ensure!(
                    additions.len() == 1,
                    "succeeded attempt requires matching evidence"
                );
            }
        }
        if let Some(index) = old.active
            && next.active != Some(index)
            && next.generation == old.generation
        {
            let state = &next.history[index].state;
            ensure!(
                *state == AttemptState::Succeeded
                    || (*state == AttemptState::Interrupted && next.terminal_decision.is_some())
                    || (*state == AttemptState::Interrupted
                        && old.history[index].unit.is_some()
                        && next.retries.len() == old.retries.len() + 1
                        && next
                            .retries
                            .last()
                            .is_some_and(|event| event.prior_attempt == index
                                && !event.reason.trim().is_empty()))
                    || (matches!(state, AttemptState::Waiting | AttemptState::Failed)
                        && next.retries.len() == old.retries.len() + 1
                        && next
                            .retries
                            .last()
                            .is_some_and(|event| event.prior_attempt == index)),
                "active reservation can only close by completion or explicit resolved retry"
            );
        }
        if let Some(index) = next.active {
            ensure!(
                next.history[index].context_version == next.context_version,
                "active context identity differs"
            );
            ensure!(
                next.history[index].generation == next.generation
                    && !matches!(
                        next.history[index].state,
                        AttemptState::Succeeded | AttemptState::Interrupted
                    ),
                "active attempt must be unresolved current generation"
            );
        }
        for (old, new) in [
            (
                serde_json::to_value(&old.finalizations)?,
                serde_json::to_value(&next.finalizations)?,
            ),
            (
                serde_json::to_value(&old.escalations)?,
                serde_json::to_value(&next.escalations)?,
            ),
            (
                serde_json::to_value(&old.invalidations)?,
                serde_json::to_value(&next.invalidations)?,
            ),
            (
                serde_json::to_value(&old.retries)?,
                serde_json::to_value(&next.retries)?,
            ),
        ] {
            ensure!(
                new.as_array()
                    .expect("array")
                    .starts_with(old.as_array().expect("array")),
                "workflow decision history is append-only"
            );
        }
    }
    Ok(())
}
fn own_task_marker_rollback(error: &anyhow::Error, task_id: TaskId) -> bool {
    matches!(
        error.downcast_ref::<StateGuardError>(),
        Some(StateGuardError::SnapshotChanged { table, id, .. })
            if table == "tasks" && *id == task_id.to_string()
    )
}
fn recovery_issue(phase: Phase) -> &'static str {
    if irreversible(phase) { "#13" } else { "#14" }
}

fn owners(store: &Store, id: TaskId) -> Result<(Project, Goal, Task)> {
    let task = store.task(id)?.context("unknown Task")?;
    let project = store.project(task.project_id)?.context("unknown Project")?;
    let goal = store.goal(task.goal_id)?.context("unknown Goal")?;
    ensure!(goal.project_id == project.id, "foreign Goal ownership");
    Ok((project, goal, task))
}
fn active(project: &Project, goal: &Goal, task: &Task) -> Result<()> {
    ensure!(
        project.state == ProjectState::Registered,
        "Project inactive"
    );
    ensure!(
        !goal_terminal(goal.state)
            && matches!(
                goal.state,
                GoalState::Created | GoalState::Analyzing | GoalState::Running
            ),
        "Goal inactive"
    );
    ensure!(!task_terminal(task.state), "Task terminal");
    Ok(())
}
fn rules_changed(a: &SourceSnapshot, b: &SourceSnapshot) -> bool {
    let rules = |s: &SourceSnapshot| {
        s.source_versions
            .iter()
            .filter(|(k, _)| k.starts_with("rules:"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<BTreeMap<_, _>>()
    };
    rules(a) != rules(b)
}
fn same_sources(a: &SourceSnapshot, b: &SourceSnapshot) -> bool {
    a.scope == b.scope
        && a.revision == b.revision
        && a.artifact == b.artifact
        && a.source_versions == b.source_versions
}
fn native_wait_state(reason: crate::execution::WaitReason) -> TaskState {
    match reason {
        crate::execution::WaitReason::Quota => TaskState::WaitingQuota,
        crate::execution::WaitReason::Capacity => TaskState::WaitingCapacity,
        crate::execution::WaitReason::Resource => TaskState::WaitingResource,
        _ => TaskState::WaitingHuman,
    }
}
fn next_phase(workflow: &WorkflowSnapshot) -> Option<Phase> {
    workflow
        .configured_phases
        .iter()
        .copied()
        .find(|p| !workflow.completed.contains_key(p))
}
fn invalidate(workflow: &mut WorkflowSnapshot, source: &SourceSnapshot, cause: &str) -> Result<()> {
    workflow.invalidations.push(Invalidation {
        generation: workflow.generation,
        cause: cause.into(),
        previous_revision: workflow.sources.revision.clone(),
        revision: source.revision.clone(),
        previous_sources: workflow.sources.source_versions.clone(),
        sources: source.source_versions.clone(),
        at: now_ms(),
    });
    workflow.generation = workflow
        .generation
        .checked_add(1)
        .context("workflow generation overflow")?;
    workflow.context_fresh = false;
    workflow.completed.clear();
    workflow.finished = false;
    Ok(())
}
fn set_context(snapshot: &mut Snapshot, context: &ContextVersion) {
    snapshot.task.context_version = context.version;
    snapshot.task.revision = Some(context.revision.clone());
    snapshot.workflow.context_version = context.version;
    snapshot.workflow.context_fresh = true;
    snapshot.workflow.sources.payload = context.data["payload"].as_str().unwrap_or_default().into();
}
fn make_context(
    task: &Task,
    source: &SourceSnapshot,
    phase: Phase,
    class: WorkflowClass,
    generation: u64,
    budget: ContextBudget,
    version: u64,
) -> ContextVersion {
    let mut source_hashes = source.source_versions.clone();
    source_hashes.insert("workflow:phase".into(), phase.key().into());
    source_hashes.insert("workflow:generation".into(), generation.to_string());
    if let Some(artifact) = source.artifact {
        source_hashes.insert("workflow:artifact".into(), artifact.to_string());
    }
    ContextVersion {
        scope: task.scope(),
        version,
        revision: source.revision.clone(),
        source_hashes,
        data: json!({"phase":phase,"workflow":class,"generation":generation,"budget":budget,"payload":source.payload}),
    }
}
fn load_rules(
    project: &Project,
    runtime: Config,
) -> Result<(Config, String, BTreeMap<String, String>)> {
    let mut versions = BTreeMap::new();
    let mut config = runtime;
    if let Some(reference) = &project.config_ref {
        let path = crate::project::resolve_file(project, reference)?;
        let text = std::fs::read_to_string(&path)?;
        versions.insert(
            "rules:config".into(),
            format!("{:x}", Sha256::digest(text.as_bytes())),
        );
        config = config.with_project_text(&text)?;
    }
    let mut rules = String::new();
    for reference in &project.rule_refs {
        let path = crate::project::resolve_file(project, reference)?;
        let content = std::fs::read_to_string(&path)?;
        versions.insert(
            format!("rules:{}", path.display()),
            format!("{:x}", Sha256::digest(content.as_bytes())),
        );
        rules.push_str(&format!(
            "\nMandatory Project rule {}:\n{}\n",
            path.display(),
            content
        ));
    }
    config.validate()?;
    Ok((config, rules, versions))
}

#[cfg(test)]
#[path = "workflow/committed_source_tests.rs"]
mod committed_source_tests;
#[cfg(test)]
#[path = "workflow/managed_tests.rs"]
mod managed_tests;
#[cfg(test)]
#[path = "workflow/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "workflow/native_preflight_tests.rs"]
mod native_preflight_tests;
