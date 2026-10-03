//! Durable identities and snapshots. Persisted process state is not liveness evidence.
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::config::WorkflowClass;

macro_rules! identity {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);
        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }
        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
        impl std::str::FromStr for $name {
            type Err = uuid::Error;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(value)?))
            }
        }
    };
}
identity!(ProjectId);
identity!(GoalId);
identity!(TaskId);
identity!(SessionId);
identity!(RecordId);

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(i64::MAX)
}

macro_rules! states {
    ($name:ident { $($state:ident),+ }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        pub enum $name { $($state),+ }
    };
}
states!(ProjectState {
    Registered,
    Blocked,
    Removed
});
states!(GoalState {
    Created,
    Analyzing,
    Running,
    WaitingHuman,
    Paused,
    Blocked,
    Completed,
    Cancelled,
    Failed
});
states!(TaskState {
    Created,
    Consulting,
    Planning,
    Implementing,
    Testing,
    WaitingApproval,
    WaitingReview,
    Reviewing,
    Fixing,
    WaitingHuman,
    ReadyForPr,
    PrCreated,
    Merged,
    Failed,
    Completed,
    Cancelled
});
states!(SessionState {
    Starting,
    Running,
    WaitingApproval,
    WaitingHuman,
    Exited,
    Failed,
    Lost,
    Stopped
});
states!(SessionRole {
    Executor,
    Reviewer,
    Consultant,
    ApprovalReviewer
});
states!(RiskClass { R0, R1, R2, R3 });

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub project_id: ProjectId,
    pub goal_id: Option<GoalId>,
    pub task_id: Option<TaskId>,
}
impl Scope {
    pub fn project(project_id: ProjectId) -> Self {
        Self {
            project_id,
            goal_id: None,
            task_id: None,
        }
    }
    pub fn goal(project_id: ProjectId, goal_id: GoalId) -> Self {
        Self {
            project_id,
            goal_id: Some(goal_id),
            task_id: None,
        }
    }
    pub fn task(project_id: ProjectId, goal_id: GoalId, task_id: TaskId) -> Self {
        Self {
            project_id,
            goal_id: Some(goal_id),
            task_id: Some(task_id),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub root: PathBuf,
    pub repository_identity: String,
    pub base_branch: String,
    pub config_ref: Option<PathBuf>,
    pub rule_refs: Vec<PathBuf>,
    pub worktree_root: PathBuf,
    /// Names/references only, never credential values.
    pub environment_refs: Vec<String>,
    pub max_tasks: usize,
    pub state: ProjectState,
    #[serde(default)]
    pub blocked_reason: Option<String>,
    pub version: u64,
    pub created_at: i64,
    pub updated_at: i64,
}
impl Project {
    pub fn new(
        name: String,
        root: PathBuf,
        repository_identity: String,
        base_branch: String,
    ) -> Self {
        Self {
            id: ProjectId::new(),
            name,
            worktree_root: root.join("worktree"),
            root,
            repository_identity,
            base_branch,
            config_ref: None,
            rule_refs: vec![],
            environment_refs: vec![],
            max_tasks: 4,
            state: ProjectState::Registered,
            blocked_reason: None,
            version: 0,
            created_at: now_ms(),
            updated_at: now_ms(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionCriterion {
    pub id: String,
    pub description: String,
    /// A durable evidence reference, not an LLM's unsupported assertion.
    pub evidence: Option<String>,
    pub satisfied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub prerequisite: TaskId,
    pub dependent: TaskId,
    pub hard: bool,
}
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaskDag {
    pub nodes: Vec<TaskId>,
    pub edges: Vec<Dependency>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FollowupProposal {
    pub title: String,
    pub scope: String,
    pub rationale: String,
    pub acceptance_criteria: Vec<String>,
    pub dependencies: Vec<TaskId>,
    pub risk: RiskClass,
    pub material_scope_expansion: bool,
    pub disposition: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Goal {
    pub id: GoalId,
    pub project_id: ProjectId,
    pub title: String,
    pub objective: String,
    pub completion_criteria: Vec<CompletionCriterion>,
    pub constraints: Vec<String>,
    pub non_goals: Vec<String>,
    pub source_refs: Vec<String>,
    pub state: GoalState,
    pub dag: TaskDag,
    pub followups: Vec<FollowupProposal>,
    pub context_version: u64,
    pub blockers: Vec<String>,
    pub version: u64,
    pub created_at: i64,
    pub updated_at: i64,
}
impl Goal {
    pub fn new(
        project_id: ProjectId,
        objective: String,
        completion_criteria: Vec<CompletionCriterion>,
    ) -> Self {
        Self {
            id: GoalId::new(),
            project_id,
            title: objective.clone(),
            objective,
            completion_criteria,
            constraints: vec![],
            non_goals: vec![],
            source_refs: vec![],
            state: GoalState::Created,
            dag: TaskDag::default(),
            followups: vec![],
            context_version: 0,
            blockers: vec![],
            version: 0,
            created_at: now_ms(),
            updated_at: now_ms(),
        }
    }
    pub fn scope(&self) -> Scope {
        Scope::goal(self.project_id, self.id)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub id: TaskId,
    pub project_id: ProjectId,
    pub goal_id: GoalId,
    pub issue: Option<u64>,
    pub title: String,
    pub acceptance_criteria: Vec<String>,
    pub worktree: Option<PathBuf>,
    pub branch: Option<String>,
    pub executor: String,
    pub reviewers: Vec<String>,
    pub workflow: WorkflowClass,
    pub risk: RiskClass,
    pub state: TaskState,
    pub phase: Option<String>,
    pub context_version: u64,
    pub revision: Option<String>,
    pub artifacts: Vec<String>,
    pub blockers: Vec<String>,
    pub next_action: Option<String>,
    pub version: u64,
    pub created_at: i64,
    pub updated_at: i64,
}
impl Task {
    pub fn new(project_id: ProjectId, goal_id: GoalId, title: String, executor: String) -> Self {
        Self {
            id: TaskId::new(),
            project_id,
            goal_id,
            issue: None,
            title,
            executor,
            acceptance_criteria: vec![],
            worktree: None,
            branch: None,
            reviewers: vec![],
            workflow: WorkflowClass::Standard,
            risk: RiskClass::R1,
            state: TaskState::Created,
            phase: None,
            context_version: 0,
            revision: None,
            artifacts: vec![],
            blockers: vec![],
            next_action: None,
            version: 0,
            created_at: now_ms(),
            updated_at: now_ms(),
        }
    }
    pub fn scope(&self) -> Scope {
        Scope::task(self.project_id, self.goal_id, self.id)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub id: SessionId,
    pub scope: Scope,
    pub agent: String,
    pub provider: String,
    pub role: SessionRole,
    pub native_ref: Option<String>,
    pub pid: Option<u32>,
    pub worktree: PathBuf,
    pub state: SessionState,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub recovery: Value,
    pub started_at: i64,
}

states!(RecordKind {
    AgentConfig,
    Session,
    Review,
    Approval,
    Context,
    Checkpoint,
    Verification,
    WorktreeLock,
    Workflow
});
impl RecordKind {
    pub fn key(self) -> &'static str {
        match self {
            Self::AgentConfig => "agent_config",
            Self::Session => "session",
            Self::Review => "review",
            Self::Approval => "approval",
            Self::Context => "context",
            Self::Checkpoint => "checkpoint",
            Self::Verification => "verification",
            Self::WorktreeLock => "worktree_lock",
            Self::Workflow => "workflow",
        }
    }
}

/// Typed extensions serialize in this envelope; ownership is indexed, never inferred from JSON.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: RecordId,
    pub scope: Scope,
    pub kind: RecordKind,
    pub version: u64,
    pub data: Value,
    pub created_at: i64,
    pub updated_at: i64,
}
impl Record {
    pub fn new(scope: Scope, kind: RecordKind, data: Value) -> Self {
        Self {
            id: RecordId::new(),
            scope,
            kind,
            version: 0,
            data,
            created_at: now_ms(),
            updated_at: now_ms(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextVersion {
    pub scope: Scope,
    pub version: u64,
    pub revision: String,
    pub source_hashes: std::collections::BTreeMap<String, String>,
    pub data: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    pub scope: Scope,
    pub session_id: SessionId,
    pub agent: String,
    pub phase: String,
    pub review_round: Option<u32>,
    pub input_tokens: Option<u64>,
    pub cached_input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub estimated_cost: Option<f64>,
    pub context_pack_version: Option<u64>,
    pub context_pack_size: Option<u64>,
    pub repo_map_size: Option<u64>,
    pub cache_metadata: Value,
    pub missing_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditEvent {
    pub sequence: i64,
    pub scope: Scope,
    pub kind: String,
    pub at: i64,
    pub data: Value,
}
