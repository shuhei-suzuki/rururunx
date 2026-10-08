//! Explicit recorded view consumer; rendering makes no reads or decisions.
use crate::{
    domain::{GoalId, ProjectId},
    runtime::control::*,
};
use anyhow::{Result, ensure};
use std::fmt::Write;

pub fn validate(response: &ControlResponse, project: ProjectId, goal: GoalId) -> Result<()> {
    let valid = match response {
        ControlResponse::GoalFacts {
            goal: id,
            recorded: Some(r),
            ..
        } => *id == goal && r.project == project && r.accepted,
        ControlResponse::GoalProposalFacts {
            goal: id,
            accepted,
            recorded: Some(r),
            ..
        } => *id == goal && r.project == project && !accepted && !r.accepted,
        ControlResponse::GoalTaskPage {
            goal: id,
            tasks,
            recorded: Some(r),
            ..
        } => {
            *id == goal
                && r.project == project
                && tasks.len() == r.nodes.len()
                && tasks.iter().zip(&r.nodes).all(|(t, n)| {
                    t.scope.project_id == project
                        && t.scope.goal_id == Some(goal)
                        && t.scope.task_id == Some(n.task)
                        && t.version == n.version
                })
        }
        _ => false,
    };
    ensure!(
        valid,
        "recorded_v1 unsupported/inconsistent; if this service predates the view, restart a matching rrx serve"
    );
    Ok(())
}
fn quoted(value: &str) -> String {
    let mut output = String::from("\"");
    for ch in value.chars() {
        if ch.is_control() {
            write!(output, "\\u{:04x}", ch as u32).unwrap();
        } else {
            match ch {
                '"' => output.push_str("\\\""),
                '\\' => output.push_str("\\\\"),
                _ => output.push(ch),
            }
        }
    }
    output.push('"');
    output
}
fn wire(value: &impl serde::Serialize) -> String {
    // Only fixed typed enums/booleans use this helper, never user text.
    serde_json::to_string(value).expect("typed enum serialization")
}
fn counts(output: &mut String, dag: DagCounts) {
    writeln!(
        output,
        "DAG: {} nodes, {} hard edges, {} soft edges",
        dag.node_count, dag.hard_edge_count, dag.soft_edge_count
    )
    .unwrap();
}
pub fn render(response: &ControlResponse, json: bool) -> Result<String> {
    let mut unavailable = vec![
        "native_dispatch",
        "unit_provider_evidence",
        "wait_cleanup_details",
        "verified_criterion_completion",
        "runnable_admission",
    ];
    match response {
        ControlResponse::GoalFacts {
            recorded: Some(r), ..
        } => {
            if matches!(r.criteria, RecordedCriteria::Unavailable { .. }) {
                unavailable.push("recorded_criteria");
            }
        }
        ControlResponse::GoalTaskPage {
            recorded: Some(r), ..
        } => {
            if r.nodes
                .iter()
                .any(|n| matches!(n.incoming, RecordedIncoming::Unavailable { .. }))
            {
                unavailable.push("task_incoming_relationships");
            }
        }
        ControlResponse::GoalProposalFacts {
            recorded: Some(_), ..
        } => {}
        _ => anyhow::bail!("recorded_v1 response missing"),
    }
    if json {
        return Ok(serde_json::to_string_pretty(&serde_json::json!({
            "observation": "independent_scoped_observation", "complete": false,
            "unavailable_fields": unavailable, "facts": response,
        }))?);
    }
    let mut output = String::new();
    match response {
        ControlResponse::GoalFacts {
            goal,
            version,
            state,
            recorded: Some(r),
            ..
        } => {
            writeln!(
                output,
                "Project: {}\nGoal: {} version {} state {}\nAccepted: true",
                r.project,
                goal,
                version,
                wire(state)
            )?;
            match &r.criteria {
                RecordedCriteria::Available {
                    recorded_count,
                    items,
                } => {
                    writeln!(output, "Recorded criteria: {recorded_count}")?;
                    for c in items {
                        writeln!(
                            output,
                            "Criterion {}: {} declaration {} recorded satisfied: {} recorded evidence: {}",
                            quoted(&c.id),
                            quoted(&c.description),
                            wire(&c.evaluator),
                            c.recorded_satisfied,
                            c.recorded_evidence
                                .as_deref()
                                .map(quoted)
                                .unwrap_or_else(|| "null".into())
                        )?;
                    }
                }
                RecordedCriteria::Unavailable { recorded_count, .. } => writeln!(
                    output,
                    "Recorded criteria: unavailable (projection_budget), recorded count: {recorded_count}"
                )?,
            }
            counts(&mut output, r.dag);
        }
        ControlResponse::GoalProposalFacts {
            goal,
            version,
            state,
            objective,
            recorded: Some(r),
            ..
        } => {
            writeln!(
                output,
                "Project: {}\nGoal: {} version {} state {}\nnot accepted\nObjective: {}\nCriteria/DAG: not_accepted",
                r.project,
                goal,
                version,
                wire(state),
                quoted(objective)
            )?;
        }
        ControlResponse::GoalTaskPage {
            goal,
            version,
            tasks,
            next,
            recorded: Some(r),
        } => {
            writeln!(
                output,
                "Project: {}\nGoal: {} version {}",
                r.project, goal, version
            )?;
            counts(&mut output, r.dag);
            for (t, n) in tasks.iter().zip(&r.nodes) {
                writeln!(
                    output,
                    "Task: {} version {} stored state {} effective state {} phase {}",
                    n.task,
                    n.version,
                    wire(&n.stored_state),
                    wire(&t.state),
                    t.phase
                        .as_deref()
                        .map(quoted)
                        .unwrap_or_else(|| "null".into())
                )?;
                if let Some(wait) = &t.workflow_wait {
                    writeln!(
                        output,
                        "Workflow wait: {} {}",
                        wire(&wait.kind),
                        quoted(&wait.detail)
                    )?;
                }
                writeln!(
                    output,
                    "Structural dependencies: {}; incoming: {} (hard: {})",
                    wire(&n.structural_dependencies),
                    n.incoming_count,
                    n.hard_incoming_count
                )?;
                match &n.incoming {
                    RecordedIncoming::Available { items } => {
                        for p in items {
                            writeln!(
                                output,
                                "  {} prerequisite: {} version {} stored state {}",
                                if p.hard { "hard" } else { "soft" },
                                p.task,
                                p.version,
                                wire(&p.stored_state)
                            )?;
                        }
                    }
                    RecordedIncoming::Unavailable { .. } => {
                        writeln!(output, "  Incoming detail: unavailable (projection_budget)")?
                    }
                }
            }
            writeln!(
                output,
                "Next: {}",
                next.map(|n| n.to_string()).unwrap_or_else(|| "null".into())
            )?;
        }
        _ => unreachable!(),
    }
    writeln!(
        output,
        "verified criterion completion: unavailable\nrunnable admission: unknown\nIndependent scoped observation; incomplete: {}",
        unavailable.join(", ")
    )?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quoted_text_round_trips_all_controls_quotes_and_unicode() {
        let input = (0u32..=0x9f).filter_map(char::from_u32).collect::<String>() + "é\"\\";
        let output = quoted(&input);
        assert!(!output.chars().any(char::is_control));
        assert_eq!(serde_json::from_str::<String>(&output).unwrap(), input);
        assert!(output.contains("\\u007f") && output.contains("\\u009b"));
    }
}
