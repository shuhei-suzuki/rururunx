//! Operator content validation. A validated plan/digest grants no Goal or native authority.
use crate::{
    config::{Config, WorkflowClass},
    domain::{Dependency, RiskClass, TaskDag, TaskId},
};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub const MAX_DEFINITION_BYTES: usize = 1024 * 1024;
pub const MAX_TEXT_BYTES: usize = 16 * 1024;
pub const MAX_CRITERIA: usize = 128;
pub const MAX_SOURCE_REFS: usize = 128;
const MAX_LIST: usize = 4096;

pub use crate::domain::CriterionEvaluator;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriterionDefinition {
    pub id: String,
    pub description: String,
    pub evaluator: CriterionEvaluator,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalDefinition {
    pub title: String,
    pub objective: String,
    pub criteria: Vec<CriterionDefinition>,
    pub constraints: Vec<String>,
    pub non_goals: Vec<String>,
    pub source_refs: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskDefinition {
    /// A plan-local key. Actual scoped Task identities are assigned by the Store.
    pub key: String,
    pub title: String,
    pub acceptance_criteria: Vec<String>,
    pub executor: String,
    pub reviewers: Vec<String>,
    pub workflow: WorkflowClass,
    pub risk: RiskClass,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanDependency {
    pub prerequisite: String,
    pub dependent: String,
    pub hard: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalPlan {
    pub definition: GoalDefinition,
    pub tasks: Vec<TaskDefinition>,
    pub dependencies: Vec<PlanDependency>,
}
/// Immutable validated content, not an accepted definition or dispatch capability.
pub struct ValidatedGoalPlan {
    plan: GoalPlan,
    definition_sha256: String,
}
impl ValidatedGoalPlan {
    pub fn plan(&self) -> &GoalPlan {
        &self.plan
    }
    pub fn definition_sha256(&self) -> &str {
        &self.definition_sha256
    }
}
fn text(value: &str) -> Result<()> {
    ensure!(
        !value.trim().is_empty() && value.len() <= MAX_TEXT_BYTES,
        "Goal text empty/over budget"
    );
    Ok(())
}
fn texts(values: &[String], limit: usize) -> Result<()> {
    ensure!(values.len() <= limit, "Goal list over budget");
    for value in values {
        text(value)?;
    }
    Ok(())
}
struct Canonical {
    hash: Sha256,
    bytes: usize,
}
impl Canonical {
    fn raw(&mut self, bytes: &[u8]) -> Result<()> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .ok_or_else(|| anyhow::anyhow!("Goal encoding overflow"))?;
        ensure!(
            self.bytes <= MAX_DEFINITION_BYTES,
            "Goal definition encoding over budget"
        );
        self.hash.update(bytes);
        Ok(())
    }
    fn count(&mut self, n: usize) -> Result<()> {
        self.raw(&u64::try_from(n)?.to_be_bytes())
    }
    fn field(&mut self, value: &str) -> Result<()> {
        self.count(value.len())?;
        self.raw(value.as_bytes())
    }
    fn list(&mut self, values: &[String]) -> Result<()> {
        self.count(values.len())?;
        for value in values {
            self.field(value)?;
        }
        Ok(())
    }
}
impl GoalDefinition {
    /// Fixed field order, UTF-8 byte lengths and explicit evaluator variants.
    /// List order is meaningful; JSON object ordering is not.
    pub fn canonical_digest(&self) -> Result<String> {
        text(&self.title)?;
        text(&self.objective)?;
        ensure!(
            !self.criteria.is_empty() && self.criteria.len() <= MAX_CRITERIA,
            "Goal needs bounded explicit criteria"
        );
        texts(&self.constraints, MAX_LIST)?;
        texts(&self.non_goals, MAX_LIST)?;
        texts(&self.source_refs, MAX_SOURCE_REFS)?;
        let mut ids = BTreeSet::new();
        let mut c = Canonical {
            hash: Sha256::new(),
            bytes: 0,
        };
        c.field("rrx.goal.definition.v1")?;
        c.field(&self.title)?;
        c.field(&self.objective)?;
        c.count(self.criteria.len())?;
        for criterion in &self.criteria {
            text(&criterion.id)?;
            text(&criterion.description)?;
            ensure!(ids.insert(&criterion.id), "duplicate Goal criterion ID");
            c.field(&criterion.id)?;
            c.field(&criterion.description)?;
            match criterion.evaluator {
                CriterionEvaluator::RequiredTasksVerified => c.raw(&[0])?,
                CriterionEvaluator::Human { goal_pack_input } => {
                    c.raw(&[1, u8::from(goal_pack_input)])?
                }
                CriterionEvaluator::Unverified => {
                    anyhow::bail!("Unverified criterion cannot be accepted")
                }
            }
        }
        c.list(&self.constraints)?;
        c.list(&self.non_goals)?;
        c.list(&self.source_refs)?;
        Ok(format!("{:x}", c.hash.finalize()))
    }
}
impl GoalPlan {
    pub fn validate(mut self, config: &Config) -> Result<ValidatedGoalPlan> {
        config.validate()?;
        let definition_sha256 = self.definition.canonical_digest()?;
        ensure!(
            !self.tasks.is_empty() && self.tasks.len() <= TaskDag::MAX_NODES,
            "Goal requires bounded explicit Tasks"
        );
        ensure!(
            self.dependencies.len() <= TaskDag::MAX_EDGES,
            "Goal dependency limit"
        );
        let mut keys = BTreeMap::new();
        let mut dag = TaskDag::default();
        for (index, task) in self.tasks.iter_mut().enumerate() {
            text(&task.key)?;
            text(&task.title)?;
            text(&task.executor)?;
            texts(&task.acceptance_criteria, MAX_CRITERIA)?;
            ensure!(
                !task.acceptance_criteria.is_empty(),
                "Task needs explicit acceptance criteria"
            );
            texts(&task.reviewers, MAX_CRITERIA)?;
            ensure!(
                config.agents.contains_key(&task.executor),
                "Task executor not configured"
            );
            for reviewer in &task.reviewers {
                ensure!(
                    config.agents.contains_key(reviewer),
                    "Task reviewer not configured"
                );
            }
            let risk = match task.risk {
                RiskClass::R0 => 0,
                RiskClass::R1 => 1,
                RiskClass::R2 => 2,
                RiskClass::R3 => 3,
            };
            task.workflow = task
                .workflow
                .max(config.minimum_workflow)
                .max(config.workflow.risk_mapping[risk]);
            // Temporary deterministic symbols solely for the existing structural validator.
            let symbol = TaskId(Uuid::from_u128(index as u128 + 1));
            ensure!(
                keys.insert(task.key.clone(), symbol).is_none(),
                "duplicate plan Task key"
            );
            dag.nodes.push(symbol);
        }
        for edge in &self.dependencies {
            let prerequisite = keys
                .get(&edge.prerequisite)
                .ok_or_else(|| anyhow::anyhow!("unknown plan prerequisite"))?;
            let dependent = keys
                .get(&edge.dependent)
                .ok_or_else(|| anyhow::anyhow!("unknown plan dependent"))?;
            dag.edges.push(Dependency {
                prerequisite: *prerequisite,
                dependent: *dependent,
                hard: edge.hard,
            });
        }
        dag.hard_order()?;
        Ok(ValidatedGoalPlan {
            plan: self,
            definition_sha256,
        })
    }
}
