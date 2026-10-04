//! Bounded structural DAG validation. Ordering is not readiness or dispatch authority.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, ensure};

use crate::domain::{TaskDag, TaskId};

impl TaskDag {
    pub const MAX_NODES: usize = 4096;
    pub const MAX_EDGES: usize = 16384;

    /// Validate structure and return a deterministic hard-edge topological order.
    ///
    /// Soft edges are advisory and do not participate in cycle detection. Every
    /// node remains in the order, including disconnected nodes. This checks no
    /// Project/Goal ownership, Task success, lifecycle or native resources and
    /// therefore cannot be used as a ready list or an admission permit.
    pub fn hard_order(&self) -> Result<Vec<TaskId>> {
        ensure!(self.nodes.len() <= Self::MAX_NODES, "goal DAG node limit");
        ensure!(self.edges.len() <= Self::MAX_EDGES, "goal DAG edge limit");

        let nodes: BTreeSet<_> = self.nodes.iter().copied().collect();
        ensure!(nodes.len() == self.nodes.len(), "duplicate goal DAG nodes");
        // UUID order is independent of insertion order and persisted vector order.
        let nodes: Vec<_> = nodes.into_iter().collect();
        let index: BTreeMap<_, _> = nodes.iter().enumerate().map(|(i, id)| (*id, i)).collect();
        let mut pairs = BTreeSet::new();
        let mut indegree = vec![0usize; nodes.len()];
        let mut outgoing = vec![Vec::new(); nodes.len()];
        for edge in &self.edges {
            ensure!(edge.prerequisite != edge.dependent, "goal DAG self edge");
            ensure!(
                pairs.insert((edge.prerequisite, edge.dependent)),
                "duplicate goal DAG edge pair"
            );
            let (Some(&prerequisite), Some(&dependent)) =
                (index.get(&edge.prerequisite), index.get(&edge.dependent))
            else {
                anyhow::bail!("DAG edge endpoints must be declared nodes");
            };
            if edge.hard {
                outgoing[prerequisite].push(dependent);
                indegree[dependent] += 1;
            }
        }
        let mut available: BTreeSet<_> = indegree
            .iter()
            .enumerate()
            .filter_map(|(i, degree)| (*degree == 0).then_some(i))
            .collect();
        let mut order = Vec::with_capacity(nodes.len());
        while let Some(prerequisite) = available.pop_first() {
            order.push(nodes[prerequisite]);
            for &dependent in &outgoing[prerequisite] {
                indegree[dependent] -= 1;
                if indegree[dependent] == 0 {
                    available.insert(dependent);
                }
            }
        }
        ensure!(order.len() == nodes.len(), "goal DAG hard dependency cycle");
        Ok(order)
    }
}
