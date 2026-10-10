//! S4 (HOW §4 D1–D9): the read-only shared surface. Every DTO here is a
//! projection: no body, data, native reference, path, raw kind or stored
//! attention text crosses it, and nothing here is authority.
use super::{UnavailableReason, WorkflowWait};
use crate::domain::{GoalId, GoalState, ProjectId, RecordId, Scope, TaskId, TaskState};
use serde::{Deserialize, Serialize};

/// D3: every S4 page holds 1..=128 rows.
pub const READ_PAGE_MAXIMUM: usize = 128;
/// D2: a Goal title is returned only when its UTF-8 encoding fits.
pub const GOAL_TITLE_BYTES: usize = 512;

/// D2: one Goal of a Project; never the Goal body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalView {
    pub id: GoalId,
    pub project: ProjectId,
    pub version: u64,
    pub state: GoalState,
    /// `None` when the stored title exceeds `GOAL_TITLE_BYTES`.
    pub title: Option<String>,
    pub accepted: bool,
    pub task_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewKind {
    Review,
    Approval,
}
impl ReviewKind {
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Review => "review",
            Self::Approval => "approval",
        }
    }
}
/// D4: a position in a Task's `(kind, RecordId)` Review/Approval order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewCursor {
    pub kind: ReviewKind,
    pub id: RecordId,
}
/// D4: Review/Approval metadata only; `Record.data` is never read into it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRecordView {
    pub id: RecordId,
    pub kind: ReviewKind,
    pub version: u64,
    pub created_at: i64,
    pub updated_at: i64,
}
/// D4: no stored Review/Approval is a qualified decision in S4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReviewDecisionView {
    Unavailable { reason: UnavailableReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionLane {
    Goal,
    Task,
}
/// D6 / D6-R3 R3: a queue position. `Item` names an item that must still
/// be present in the selected view; `ScannedThrough` says the scan resumes
/// after that Goal, which must still be in the selected view's inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "position", rename_all = "snake_case", deny_unknown_fields)]
pub enum AttentionCursor {
    Item {
        project: ProjectId,
        goal: GoalId,
        lane: AttentionLane,
        task: Option<TaskId>,
    },
    ScannedThrough {
        project: ProjectId,
        goal: GoalId,
    },
}
impl AttentionCursor {
    /// The Goal the position belongs to.
    pub(crate) fn goal_position(&self) -> (ProjectId, GoalId) {
        match *self {
            Self::Item { project, goal, .. } | Self::ScannedThrough { project, goal } => {
                (project, goal)
            }
        }
    }
}
/// D6: one typed fact derived from current rows. No raw text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "fact", rename_all = "snake_case", deny_unknown_fields)]
pub enum AttentionFact {
    /// A non-terminal accepted Goal or proposal with no Tasks.
    GoalWithoutTasks,
    GoalWaitingHuman,
    GoalPaused,
    GoalBlocked,
    /// The Project stores a Task limit other than 1 (R4.5).
    ProjectLimitUnsupported {
        stored: usize,
    },
    TaskWaitingHuman,
    TaskWaitingApproval,
    /// A qualified parked quota wait.
    QuotaWait {
        next_due: i64,
    },
    /// A qualified parked capacity wait.
    CapacityWait {
        next_due: i64,
    },
    WorkflowWait {
        wait: WorkflowWait,
    },
    /// The same typed unavailability the Goal/Task readers report.
    Unavailable {
        reason: UnavailableReason,
    },
    /// Stored `scheduler_tasks.attention` no typed reader reproduces.
    StoredUnqualified,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionOperationKind {
    GoalPause,
    GoalResume,
    GoalCancel,
    GoalFail,
    /// `ProjectUpdate --max-tasks 1` at the observed Project version.
    ProjectSetTaskLimit,
    TaskRetry,
    TaskCancel,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "availability", rename_all = "snake_case", deny_unknown_fields)]
pub enum OperationAvailability {
    Available,
    Unavailable { reason: UnavailableReason },
}
/// D6: advisory, never authority. Executing it is a new Control request
/// whose CAS/policy/owner checks all run again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttentionOperationView {
    pub kind: AttentionOperationKind,
    pub project: ProjectId,
    pub goal: Option<GoalId>,
    pub task: Option<TaskId>,
    /// The observed version the existing mutation API expects.
    pub expected_version: u64,
    pub availability: OperationAvailability,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttentionItem {
    pub project: ProjectId,
    pub goal: GoalId,
    pub lane: AttentionLane,
    pub task: Option<TaskId>,
    pub project_version: u64,
    pub goal_version: u64,
    pub goal_state: GoalState,
    pub task_version: Option<u64>,
    /// The effective Task state (qualified waits applied).
    pub task_state: Option<TaskState>,
    pub facts: Vec<AttentionFact>,
    pub operations: Vec<AttentionOperationView>,
}
impl AttentionItem {
    pub(crate) fn cursor(&self) -> AttentionCursor {
        AttentionCursor::Item {
            project: self.project,
            goal: self.goal,
            lane: self.lane,
            task: self.task,
        }
    }
    /// The queue order `(project, goal, lane, task-or-nil)`.
    pub(crate) fn key(&self) -> (ProjectId, GoalId, AttentionLane, Option<TaskId>) {
        (self.project, self.goal, self.lane, self.task)
    }
}

/// D7: fixed public values for an explicit allowlist of audited production
/// kinds. Every other stored kind is `EventUnavailable`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKindView {
    ProjectSaved,
    TaskSaved,
    SessionSaved,
    ReviewSaved,
    ApprovalSaved,
    WorkflowSaved,
    ContextCreated,
    UsageRecorded,
    WorkflowGateObserved,
    GoalAccepted,
    GoalProposed,
    GoalLifecycleChanged,
    DriverClaimed,
    DriverExited,
    DriverInvalidated,
    WorktreeCreated,
    WorktreeCleaned,
    VerificationAdmitted,
    VerificationAccepted,
    VerificationTerminal,
    VerificationAbandoned,
    EventUnavailable,
}
impl EventKindView {
    /// Exact-kind allowlist; no prefix, pattern or printable pass-through.
    pub(crate) fn of(kind: &str) -> Self {
        match kind {
            "project.saved" => Self::ProjectSaved,
            "task.saved" => Self::TaskSaved,
            "session.saved" => Self::SessionSaved,
            "review.saved" => Self::ReviewSaved,
            "approval.saved" => Self::ApprovalSaved,
            "workflow.saved" => Self::WorkflowSaved,
            "context.created" => Self::ContextCreated,
            "usage.recorded" => Self::UsageRecorded,
            "workflow.gate_observed" => Self::WorkflowGateObserved,
            "rrx.private.runtime.goal_accepted" => Self::GoalAccepted,
            "rrx.private.runtime.goal_proposed" => Self::GoalProposed,
            "rrx.private.runtime.goal_lifecycle" => Self::GoalLifecycleChanged,
            "rrx.private.runtime.driver_claimed" => Self::DriverClaimed,
            "rrx.private.runtime.driver_exited" => Self::DriverExited,
            "rrx.private.runtime.driver_invalidated" => Self::DriverInvalidated,
            "worktree.created" => Self::WorktreeCreated,
            "worktree.cleaned" => Self::WorktreeCleaned,
            "verification.admitted" => Self::VerificationAdmitted,
            "verification.accepted" => Self::VerificationAccepted,
            "verification.terminal" => Self::VerificationTerminal,
            "verification.abandoned" => Self::VerificationAbandoned,
            _ => Self::EventUnavailable,
        }
    }
}
/// D7: never `AuditEvent.data` or the stored kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventView {
    pub sequence: i64,
    pub scope: Scope,
    pub kind: EventKindView,
    pub at: i64,
}
