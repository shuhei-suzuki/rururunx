//! Projection after existing transactional validation. All output is nongrant data.
use crate::execution::strict_json::{self, Limits};
use crate::{
    domain::{Dependency, Goal, ProjectId, Task, TaskId},
    runtime::control::*,
};
use anyhow::{Result, ensure};
use serde::{Serialize, ser::SerializeSeq};
use std::{collections::BTreeMap, io::Write};

pub(super) const GOAL_RECORDED_STATUS_BYTES: usize = 128 * 1024;
pub(super) const GOAL_RECORDED_INCOMING_BYTES: usize = 8 * 1024;
pub(super) const GOAL_TASK_PAGE_BYTES: usize = 64 * 1024;

struct Capped {
    bytes: Vec<u8>,
    maximum: usize,
}
impl Write for Capped {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.maximum.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("Goal projection budget"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn fits_profile(value: &impl Serialize, maximum: usize, profile: Limits) -> bool {
    let mut output = Capped {
        bytes: Vec::new(),
        maximum,
    };
    serde_json::to_writer(&mut output, value).is_ok()
        && strict_json::decode(&output.bytes, profile).is_ok()
}
pub(super) fn fits(value: &impl Serialize, maximum: usize) -> bool {
    fits_profile(value, maximum, Limits::CEILINGS)
}
pub(super) fn dag(goal: &Goal) -> DagCounts {
    let hard = goal.dag.edges.iter().filter(|e| e.hard).count();
    DagCounts {
        node_count: goal.dag.nodes.len(),
        hard_edge_count: hard,
        soft_edge_count: goal.dag.edges.len() - hard,
    }
}
pub(super) fn enrich_status(
    response: &mut ControlResponse,
    goal: &Goal,
    project: ProjectId,
) -> Result<()> {
    // Only small references are collected before capped serialization. Raw domain
    // objects and over-budget strings are never cloned for a size measurement.
    let items = goal
        .completion_criteria
        .iter()
        .map(|c| RecordedCriterion {
            id: c.id.as_str(),
            description: c.description.as_str(),
            evaluator: c.evaluator.clone(),
            recorded_satisfied: c.satisfied,
            recorded_evidence: c.evidence.as_deref(),
        })
        .collect();
    let borrowed = RecordedGoalStatus {
        view: GoalReadView::RecordedV1,
        project,
        accepted: true,
        criteria: RecordedCriteria::Available {
            recorded_count: goal.completion_criteria.len(),
            items,
        },
        dag: dag(goal),
        criterion_evaluation: CriterionEvaluation::Unavailable,
        runnable_admission: RunnableAdmission::Unknown,
    };
    #[derive(Serialize)]
    struct Candidate<'a> {
        #[serde(flatten)]
        facts: &'a ControlResponse,
        recorded: &'a RecordedGoalStatus<&'a str>,
    }
    let available = fits(
        &Candidate {
            facts: response,
            recorded: &borrowed,
        },
        GOAL_RECORDED_STATUS_BYTES,
    );
    let criteria = if available {
        RecordedCriteria::Available {
            recorded_count: goal.completion_criteria.len(),
            items: goal
                .completion_criteria
                .iter()
                .map(|c| RecordedCriterion {
                    id: c.id.clone(),
                    description: c.description.clone(),
                    evaluator: c.evaluator.clone(),
                    recorded_satisfied: c.satisfied,
                    recorded_evidence: c.evidence.clone(),
                })
                .collect(),
        }
    } else {
        RecordedCriteria::Unavailable {
            recorded_count: goal.completion_criteria.len(),
            reason: ProjectionReason::ProjectionBudget,
        }
    };
    if let ControlResponse::GoalFacts { recorded, .. } = response {
        *recorded = Some(RecordedGoalStatus {
            criteria,
            view: borrowed.view,
            project,
            accepted: true,
            dag: borrowed.dag,
            criterion_evaluation: borrowed.criterion_evaluation,
            runnable_admission: borrowed.runnable_admission,
        });
    } else {
        anyhow::bail!("accepted Goal response required");
    }
    ensure!(
        fits(response, GOAL_RECORDED_STATUS_BYTES),
        "Goal status budget exceeded"
    );
    Ok(())
}
pub(super) fn enrich_proposal(response: &mut ControlResponse, project: ProjectId) -> Result<()> {
    if let ControlResponse::GoalProposalFacts { recorded, .. } = response {
        *recorded = Some(RecordedProposalStatus {
            view: GoalReadView::RecordedV1,
            project,
            accepted: false,
            criteria: ProposalPart::NotAccepted,
            dag: ProposalPart::NotAccepted,
            criterion_evaluation: CriterionEvaluation::Unavailable,
            runnable_admission: RunnableAdmission::Unknown,
        });
    } else {
        anyhow::bail!("proposal response required");
    }
    ensure!(
        fits(response, GOAL_RECORDED_STATUS_BYTES),
        "Goal proposal status budget exceeded"
    );
    Ok(())
}
struct IncomingItems<'a> {
    edges: &'a [&'a Dependency],
    tasks: &'a BTreeMap<TaskId, &'a Task>,
}
impl Serialize for IncomingItems<'_> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.edges.len()))?;
        for edge in self.edges {
            let task = self
                .tasks
                .get(&edge.prerequisite)
                .ok_or_else(|| serde::ser::Error::custom("validated endpoint missing"))?;
            sequence.serialize_element(&RecordedPrerequisite {
                task: task.id,
                version: task.version,
                stored_state: task.state,
                hard: edge.hard,
            })?;
        }
        sequence.end()
    }
}
pub(super) fn node(
    task: &Task,
    goal: &Goal,
    tasks: &BTreeMap<TaskId, &Task>,
) -> Result<RecordedDagNode> {
    let mut edges = goal
        .dag
        .edges
        .iter()
        .filter(|e| e.dependent == task.id)
        .collect::<Vec<_>>();
    edges.sort_by_key(|e| e.prerequisite);
    let hard = edges.iter().filter(|e| e.hard).count();
    #[derive(Serialize)]
    struct Candidate<'a> {
        availability: &'static str,
        items: IncomingItems<'a>,
    }
    let candidate = Candidate {
        availability: "available",
        items: IncomingItems {
            edges: &edges,
            tasks,
        },
    };
    let incoming = if fits(&candidate, GOAL_RECORDED_INCOMING_BYTES) {
        let mut items = Vec::new();
        for edge in &edges {
            let prerequisite = tasks
                .get(&edge.prerequisite)
                .ok_or_else(|| anyhow::anyhow!("validated endpoint missing"))?;
            items.push(RecordedPrerequisite {
                task: prerequisite.id,
                version: prerequisite.version,
                stored_state: prerequisite.state,
                hard: edge.hard,
            });
        }
        RecordedIncoming::Available { items }
    } else {
        RecordedIncoming::Unavailable {
            reason: ProjectionReason::ProjectionBudget,
        }
    };
    Ok(RecordedDagNode {
        task: task.id,
        version: task.version,
        stored_state: task.state,
        incoming_count: edges.len(),
        hard_incoming_count: hard,
        structural_dependencies: if hard == 0 {
            StructuralDependencies::Unconstrained
        } else {
            StructuralDependencies::RequiresPrerequisiteEvidence
        },
        incoming,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projection_fit_checks_exact_encoded_boundary_and_defensive_profile() {
        let text = "é\u{1}";
        let size = serde_json::to_vec(text).unwrap().len();
        assert!(fits(&text, size));
        assert!(!fits(&text, size - 1));
        assert!(!fits_profile(
            &text,
            size,
            Limits {
                string_bytes: 1,
                ..Limits::CEILINGS
            }
        ));
        assert!(!fits_profile(
            &vec![1, 2],
            64,
            Limits {
                array_entries: 1,
                ..Limits::CEILINGS
            }
        ));
    }
}
