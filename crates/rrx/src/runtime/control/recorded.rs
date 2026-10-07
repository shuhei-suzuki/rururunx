//! Narrow observations only. No evaluator result, scheduling or authority.
use crate::domain::{CriterionEvaluator, ProjectId, TaskId, TaskState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalReadView {
    RecordedV1,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriterionEvaluation {
    Unavailable,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnableAdmission {
    Unknown,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionReason {
    ProjectionBudget,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalPart {
    NotAccepted,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuralDependencies {
    Unconstrained,
    RequiresPrerequisiteEvidence,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DagCounts {
    pub node_count: usize,
    pub hard_edge_count: usize,
    pub soft_edge_count: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedCriterion<T = String> {
    pub id: T,
    pub description: T,
    pub evaluator: CriterionEvaluator,
    pub recorded_satisfied: bool,
    pub recorded_evidence: Option<T>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "availability", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecordedCriteria<T = String> {
    Available {
        recorded_count: usize,
        items: Vec<RecordedCriterion<T>>,
    },
    Unavailable {
        recorded_count: usize,
        reason: ProjectionReason,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedGoalStatus<T = String> {
    pub view: GoalReadView,
    pub project: ProjectId,
    pub accepted: bool,
    pub criteria: RecordedCriteria<T>,
    pub dag: DagCounts,
    pub criterion_evaluation: CriterionEvaluation,
    pub runnable_admission: RunnableAdmission,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedProposalStatus {
    pub view: GoalReadView,
    pub project: ProjectId,
    pub accepted: bool,
    pub criteria: ProposalPart,
    pub dag: ProposalPart,
    pub criterion_evaluation: CriterionEvaluation,
    pub runnable_admission: RunnableAdmission,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedGoalTaskPage {
    pub view: GoalReadView,
    pub project: ProjectId,
    pub dag: DagCounts,
    pub nodes: Vec<RecordedDagNode>,
    pub criterion_evaluation: CriterionEvaluation,
    pub runnable_admission: RunnableAdmission,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedDagNode {
    pub task: TaskId,
    pub version: u64,
    pub stored_state: TaskState,
    pub incoming_count: usize,
    pub hard_incoming_count: usize,
    pub structural_dependencies: StructuralDependencies,
    pub incoming: RecordedIncoming,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "availability", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecordedIncoming {
    Available { items: Vec<RecordedPrerequisite> },
    Unavailable { reason: ProjectionReason },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedPrerequisite {
    pub task: TaskId,
    pub version: u64,
    pub stored_state: TaskState,
    pub hard: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::control::ControlAction;
    #[test]
    fn omitted_and_null_view_preserve_legacy_bytes_and_strict_shapes() {
        let old = r#"{"action":"goal_status","project":"11111111-1111-1111-1111-111111111111","goal":"22222222-2222-2222-2222-222222222222"}"#;
        let action: ControlAction = serde_json::from_str(old).unwrap();
        assert_eq!(serde_json::to_string(&action).unwrap(), old);
        let null = old.strip_suffix('}').unwrap().to_owned() + ",\"view\":null}";
        let action: ControlAction = serde_json::from_str(&null).unwrap();
        assert_eq!(serde_json::to_string(&action).unwrap(), old);
        for value in ["\"unknown\"", "false", "{}", "[]"] {
            let raw = old.strip_suffix('}').unwrap().to_owned() + ",\"view\":" + value + "}";
            assert!(serde_json::from_str::<ControlAction>(&raw).is_err());
        }
        let extra = r#"{"availability":"unavailable","recorded_count":7,"reason":"projection_budget","items":[]}"#;
        assert!(serde_json::from_str::<RecordedCriteria>(extra).is_err());
        assert!(
            serde_json::from_str::<RecordedIncoming>(
                r#"{"availability":"available","items":[],"grant":true}"#
            )
            .is_err()
        );
    }
}
