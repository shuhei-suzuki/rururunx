//! Durable Task phases. Evidence providers, not process exits, authorize gates.
use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    adapter::{
        AgentRegistry, Capability, InputKind, LaunchMode, LaunchRequest, PreparedInput, SessionRef,
        SessionStatus, SharedStore,
    },
    config::{Config, WorkflowClass},
    domain::*,
    state::{Store, WorkflowAccess, goal_terminal, task_terminal},
};

pub type WorkflowFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Issue,
    Worktree,
    Requirements,
    RequirementsReview,
    Design,
    DesignReview,
    Implement,
    ImpactAnalysis,
    Tests,
    ImplementationReview,
    SecurityReview,
    ExpandedRegression,
    Mutation,
    Browser,
    Staging,
    Commit,
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
            Self::RequirementsReview => "requirements_review",
            Self::Design => "design",
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
    fn task_state(self) -> TaskState {
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
        vec![Worktree, Implement, Tests, ImplementationReview]
    } else {
        vec![
            Issue,
            Worktree,
            Requirements,
            RequirementsReview,
            Design,
            DesignReview,
            Implement,
            ImpactAnalysis,
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
    result.extend([Commit, Pr]);
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
fn budget(class: WorkflowClass, phase: Phase, config: &Config) -> ContextBudget {
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
    pub source_versions: BTreeMap<String, String>,
    pub payload: String,
}
pub trait WorkflowSources: Send + Sync {
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
    /// Only an external review integration can attest a decision.
    pub review_approved: Option<bool>,
    /// Binds an agent-produced artifact/verdict to the launched Session.
    pub session_id: Option<SessionId>,
    pub context_version: u64,
}
#[derive(Debug, Clone)]
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
    pub agent: Option<String>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub detail: Option<String>,
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
    pub sources: SourceSnapshot,
    pub configured_phases: Vec<Phase>,
    pub finished: bool,
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
}

pub struct WorkflowEngine {
    store: SharedStore,
    registry: Arc<AgentRegistry>,
    runtime: Config,
    sources: Arc<dyn WorkflowSources>,
    gates: Arc<dyn PhaseGates>,
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
        Ok(Self {
            store,
            registry,
            runtime,
            sources,
            gates,
        })
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
            project,
            goal,
            task,
            record,
            workflow,
        })
    }
    fn persist(&self, snapshot: &mut Snapshot, context: Option<&ContextVersion>) -> Result<()> {
        snapshot.record.data = serde_json::to_value(&snapshot.workflow)?;
        self.store
            .lock()
            .map_err(|_| anyhow::anyhow!("state store poisoned"))?
            .put_workflow_transition(
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
        let (config, source, selected_budget) = self.inputs(project, task, phase, class).await?;
        ensure!(
            same_sources(expected, &source),
            "authority changed while preparing phase Context Pack"
        );
        ensure!(
            class >= config.minimum_workflow
                && class >= config.workflow.default
                && class >= risk_workflow(&config, task.risk),
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
            .max(config.workflow.default)
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
            sources: SourceSnapshot {
                payload: context.data["payload"]
                    .as_str()
                    .context("prepared payload missing")?
                    .into(),
                ..source
            },
            configured_phases,
            finished: false,
        };
        let record = Record::new(
            task.scope(),
            RecordKind::Workflow,
            serde_json::to_value(&workflow_state)?,
        );
        let mut snapshot = Snapshot {
            project,
            goal,
            task,
            record,
            workflow: workflow_state,
        };
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
            .max(config.workflow.default)
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
            invalidate(&mut snapshot.workflow)?;
        }
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
            .max(config.workflow.default)
            .max(risk_workflow(&config, snapshot.workflow.risk));
        let configured =
            retain_phases(&snapshot.workflow.configured_phases, phases(class, &config));
        if class > snapshot.workflow.workflow || configured != snapshot.workflow.configured_phases {
            let old = snapshot.workflow.workflow;
            snapshot.workflow.workflow = class;
            snapshot.task.workflow = class;
            snapshot.workflow.configured_phases = configured;
            snapshot.workflow.escalations.push(Escalation {
                from: old,
                to: class,
                reason: "scoped policy refreshed".into(),
                evidence: serde_json::to_string(&source.source_versions)?,
                at: now_ms(),
            });
            invalidate(&mut snapshot.workflow)?;
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
            invalidate(&mut snapshot.workflow)?;
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
            agent: match phase.actor() {
                Actor::Executor => Some(snapshot.task.executor.clone()),
                Actor::Reviewer => snapshot.task.reviewers.first().cloned(),
                Actor::EvidencePort => None,
            },
            started_at: now_ms(),
            completed_at: None,
            detail: None,
        });
        snapshot.workflow.active = Some(index);
        self.reserve(&mut snapshot, &context, phase)?;
        if phase.actor() == Actor::EvidencePort {
            return self.evaluate(snapshot, index, None).await;
        }
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
            invalidate(&mut snapshot.workflow)?;
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
            self.persist(&mut snapshot, Some(&context))?;
            return Ok(StepResult::Invalidated {
                reason: "sources changed before native dispatch".into(),
            });
        }
        {
            let store = self
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state store poisoned"))?;
            let (project, goal, task) = owners(&store, snapshot.task.id)?;
            ensure!(
                project.version == snapshot.project.version
                    && goal.version == snapshot.goal.version
                    && task.version == snapshot.task.version,
                "owners changed before native dispatch"
            );
            ensure!(
                store
                    .record(snapshot.record.id)?
                    .is_some_and(|r| r.version == snapshot.record.version),
                "workflow changed before native dispatch"
            );
        }
        let agent = if phase.actor() == Actor::Reviewer {
            match snapshot.task.reviewers.first() {
                Some(agent) => agent.clone(),
                None => return self.fail(snapshot, index, "reviewer not configured".into()),
            }
        } else {
            snapshot.task.executor.clone()
        };
        let adapter = match self.registry.get(&agent) {
            Ok(adapter) => adapter,
            Err(error) => return self.fail(snapshot, index, error.to_string()),
        };
        let needed = if phase.actor() == Actor::Reviewer {
            Capability::Review
        } else {
            Capability::Execute
        };
        if !adapter.capabilities().contains(&needed) {
            return self.fail(snapshot, index, format!("agent {agent} lacks {needed:?}"));
        }
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
        let Some(worktree) = snapshot.task.worktree.clone() else {
            return self.fail(
                snapshot,
                index,
                "agent phase requires bound worktree".into(),
            );
        };
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
                // Adapter may persist Session, never rewrite Task/history. CAS loss
                // preserves the reservation; #13 reconciles the durable Session.
                self.persist(&mut snapshot, None)?;
                Ok(StepResult::Started {
                    phase,
                    session: Some(session.id),
                })
            }
            Err(error) => self.fail(snapshot, index, error.to_string()),
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
        if matches!(
            attempt.state,
            AttemptState::Evaluating | AttemptState::Interrupted
        ) {
            return Ok(StepResult::Waiting {
                phase,
                reason: "interrupted evidence evaluation needs explicit recovery (#13)".into(),
            });
        }
        if attempt.state != AttemptState::Running {
            return Ok(StepResult::Failed {
                phase,
                reason: attempt.detail.clone().unwrap_or_default(),
            });
        }
        let Some(id) = attempt.session_id else {
            return Ok(StepResult::Waiting {
                phase,
                reason: "interrupted phase needs explicit recovery integration (#13)".into(),
            });
        };
        let agent = attempt
            .agent
            .as_ref()
            .context("native attempt actor missing")?;
        let adapter = self.registry.get(agent)?;
        let status = adapter
            .status(SessionRef {
                id,
                scope: snapshot.task.scope(),
            })
            .await?;
        ensure!(
            status.session.id == id
                && status.session.scope == snapshot.task.scope()
                && status.session.agent == *agent
                && Some(&status.session.worktree) == snapshot.task.worktree.as_ref()
                && status.session.role
                    == if phase.actor() == Actor::Reviewer {
                        SessionRole::Reviewer
                    } else {
                        SessionRole::Executor
                    },
            "foreign session status"
        );
        if !status.terminal() {
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
        self.evaluate(snapshot, index, Some(status)).await
    }
    fn fail(&self, mut snapshot: Snapshot, index: usize, reason: String) -> Result<StepResult> {
        let attempt = &mut snapshot.workflow.history[index];
        attempt.state = AttemptState::Failed;
        attempt.completed_at = Some(now_ms());
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
        let phase = snapshot.workflow.history[index].phase;
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
            .max(config.workflow.default)
            .max(risk_workflow(&config, snapshot.workflow.risk));
        if class > snapshot.workflow.workflow
            || retain_phases(&snapshot.workflow.configured_phases, phases(class, &config))
                != snapshot.workflow.configured_phases
            || rules_changed(&source, &snapshot.workflow.sources)
        {
            let old = snapshot.workflow.workflow;
            snapshot.workflow.workflow = class;
            snapshot.task.workflow = class;
            snapshot.workflow.configured_phases =
                retain_phases(&snapshot.workflow.configured_phases, phases(class, &config));
            snapshot.workflow.escalations.push(Escalation {
                from: old,
                to: class,
                reason: "policy/rules changed during phase".into(),
                evidence: serde_json::to_string(&source.source_versions)?,
                at: now_ms(),
            });
            snapshot.workflow.history[index].state = AttemptState::Interrupted;
            snapshot.workflow.history[index].completed_at = Some(now_ms());
            snapshot.workflow.active = None;
            invalidate(&mut snapshot.workflow)?;
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
        // Agent implementation may intentionally change revision/artifacts. The
        // gate must attest that observed target, not the obsolete launch revision.
        let invocation = PhaseInvocation {
            project: snapshot.project.clone(),
            task: snapshot.task.clone(),
            phase,
            context: self.context(&snapshot)?,
            sources: source.clone(),
            budget: selected_budget.clone(),
            prerequisites: snapshot.workflow.completed.values().cloned().collect(),
        };
        // CAS claims evaluation before an external integration can create evidence
        // or side effects. A concurrent poll cannot run the same gate twice.
        snapshot.workflow.history[index].state = AttemptState::Evaluating;
        self.persist(&mut snapshot, None)?;
        let outcome = self.gates.complete(invocation, status).await?;
        let (_, final_source, _) = self
            .inputs(
                &snapshot.project,
                &snapshot.task,
                phase,
                snapshot.workflow.workflow,
            )
            .await?;
        ensure!(
            same_sources(&source, &final_source),
            "sources changed while gate evaluated"
        );
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
                ensure!(
                    evidence.scope == snapshot.task.scope()
                        && evidence.phase == phase
                        && evidence.revision == source.revision
                        && evidence.source_versions == source.source_versions
                        && !evidence.artifacts.is_empty()
                        && evidence.artifacts.iter().all(|p| !p.trim().is_empty()),
                    "gate evidence is foreign/stale/incomplete"
                );
                ensure!(
                    evidence.session_id == snapshot.workflow.history[index].session_id
                        && evidence.context_version
                            == snapshot.workflow.history[index].context_version,
                    "gate session binding mismatch"
                );
                if phase.actor() == Actor::Reviewer {
                    ensure!(
                        evidence.review_approved == Some(true),
                        "review requires explicit approved verdict"
                    );
                }
                // Only mutating milestones may legitimately change the target.
                // A changed review/verification target invalidates its verdict.
                if !same_sources(&source, &snapshot.workflow.sources)
                    && !matches!(
                        phase,
                        Phase::Issue
                            | Phase::Worktree
                            | Phase::Requirements
                            | Phase::Design
                            | Phase::Implement
                            | Phase::ImpactAnalysis
                    )
                {
                    snapshot.workflow.history[index].state = AttemptState::Interrupted;
                    snapshot.workflow.history[index].detail =
                        Some("target changed during evidence phase".into());
                    snapshot.workflow.active = None;
                    invalidate(&mut snapshot.workflow)?;
                    snapshot.workflow.sources = source.clone();
                    let first = next_phase(&snapshot.workflow).expect("presets nonempty");
                    let context = self
                        .prepare_pack(
                            &snapshot.project,
                            &snapshot.task,
                            &source,
                            first,
                            snapshot.workflow.workflow,
                            snapshot.workflow.generation,
                        )
                        .await?;
                    set_context(&mut snapshot, &context);
                    self.persist(&mut snapshot, Some(&context))?;
                    return Ok(StepResult::Invalidated {
                        reason: "target changed during review/verification".into(),
                    });
                }
                snapshot
                    .task
                    .artifacts
                    .extend(evidence.artifacts.iter().cloned());
                snapshot.workflow.completed.insert(phase, evidence);
                let attempt = &mut snapshot.workflow.history[index];
                attempt.state = AttemptState::Succeeded;
                attempt.completed_at = Some(now_ms());
                snapshot.workflow.active = None;
                snapshot.workflow.sources = source.clone();
                let next = next_phase(&snapshot.workflow);
                let context = self
                    .prepare_pack(
                        &snapshot.project,
                        &snapshot.task,
                        &source,
                        next.unwrap_or(phase),
                        snapshot.workflow.workflow,
                        snapshot.workflow.generation,
                    )
                    .await?;
                set_context(&mut snapshot, &context);
                snapshot.task.phase = next.map(|p| p.key().into());
                snapshot.workflow.finished = next.is_none();
                if phase == Phase::Pr {
                    snapshot.task.state = TaskState::PrCreated;
                }
                if phase == Phase::Cleanup {
                    snapshot.task.state = TaskState::Completed;
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
        ensure!(
            matches!(
                snapshot.workflow.history[index].state,
                AttemptState::Waiting | AttemptState::Failed
            ),
            "running/interrupted phase requires recovery"
        );
        let store = self
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state store poisoned"))?;
        for record in store.records(&snapshot.task.scope(), RecordKind::Session)? {
            let session: Session = serde_json::from_value(record.data)?;
            ensure!(
                matches!(
                    session.state,
                    SessionState::Exited | SessionState::Stopped | SessionState::Failed
                ),
                "reserved/Lost session requires verified recovery"
            );
        }
        drop(store);
        snapshot.workflow.retries.push(RetryEvent {
            prior_attempt: index,
            reason,
            at: now_ms(),
        });
        snapshot.workflow.history[index].completed_at = Some(now_ms());
        snapshot.workflow.active = None;
        snapshot.task.blockers.clear();
        self.persist(&mut snapshot, None)
    }
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
                            .session_id
                            .is_none_or(|id| after.session_id == Some(id)),
                    "active attempt identity is immutable"
                );
                let valid = match before.state {
                    AttemptState::Running => true,
                    AttemptState::Evaluating => matches!(
                        after.state,
                        AttemptState::Evaluating
                            | AttemptState::Waiting
                            | AttemptState::Failed
                            | AttemptState::Succeeded
                            | AttemptState::Interrupted
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
        for (old, new) in [
            (
                serde_json::to_value(&old.escalations)?,
                serde_json::to_value(&next.escalations)?,
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
    a.scope == b.scope && a.revision == b.revision && a.source_versions == b.source_versions
}
fn next_phase(workflow: &WorkflowSnapshot) -> Option<Phase> {
    workflow
        .configured_phases
        .iter()
        .copied()
        .find(|p| !workflow.completed.contains_key(p))
}
fn invalidate(workflow: &mut WorkflowSnapshot) -> Result<()> {
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
