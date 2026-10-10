//! S4 (HOW §4 D1): CLI consumers of the shared read surface.
//!
//! A read decides its route once: a valid endpoint means the control API.
//! An absent endpoint and an invalid one (an older protocol included) are
//! both service-unavailable refusals, reported distinctly. There is no
//! direct-Store or offline read path, so neither opens or writes a Store.
use super::{client, endpoint, project::EXIT_ROUTING_REFUSED};
use crate::{
    domain::{GoalId, ProjectId, RecordId, Scope, SessionId, TaskId},
    runtime::control::*,
};
use anyhow::{Context, Result, bail, ensure};
use std::{fmt::Write, path::Path, process::ExitCode};

/// A typed unavailable answer (cursor, size, unqualified surface).
pub const EXIT_READ_UNAVAILABLE: u8 = 3;

/// The Task a Task reader selects.
pub struct TaskSelector {
    pub project: Option<String>,
    pub goal: GoalId,
    pub task: TaskId,
}
/// The Project/Goal/Task scope a scoped reader selects.
pub struct ScopeSelector {
    pub project: Option<String>,
    pub goal: Option<GoalId>,
    pub task: Option<TaskId>,
}
/// One S4 read, already parsed.
pub enum ReadRequest {
    Goals {
        project: Option<String>,
        after: Option<GoalId>,
        maximum: usize,
    },
    Review {
        task: TaskSelector,
        after: Option<ReviewCursor>,
        maximum: usize,
    },
    Sessions {
        task: TaskSelector,
        after: Option<SessionId>,
        maximum: usize,
    },
    /// `project: None` is the runtime-wide queue.
    Attention {
        project: Option<String>,
        after: Option<AttentionCursor>,
        maximum: usize,
    },
    Events {
        scope: ScopeSelector,
        after: Option<i64>,
        maximum: usize,
    },
    Routing {
        scope: ScopeSelector,
    },
    Metrics {
        scope: ScopeSelector,
    },
}

/// D1: one discovery decision, then the read through the control API.
pub async fn run(state: &Path, request: ReadRequest, json: bool) -> Result<ExitCode> {
    match endpoint::discover(state).await {
        endpoint::Discovery::Valid(_, connection) => drop(connection),
        endpoint::Discovery::Absent => {
            eprintln!(
                "rrx: service unavailable: no Runtime service for this state; start one with `rrx daemon start`"
            );
            return Ok(ExitCode::from(EXIT_ROUTING_REFUSED));
        }
        endpoint::Discovery::Invalid(error) => {
            eprintln!("rrx: service unavailable: discovery refused: {error:#}");
            return Ok(ExitCode::from(EXIT_ROUTING_REFUSED));
        }
    }
    let action = action(state, request).await?;
    let response = client::request(state, action).await?;
    if let ControlResponse::Unavailable { reason, .. } = response {
        eprintln!("rrx: unavailable: {}", wire(&reason));
        return Ok(ExitCode::from(EXIT_READ_UNAVAILABLE));
    }
    println!("{}", render(&response, json)?);
    Ok(ExitCode::SUCCESS)
}

async fn resolve(state: &Path, selector: Option<String>) -> Result<ProjectId> {
    let response = client::request(
        state,
        ControlAction::ResolveProject {
            selector,
            cwd: std::env::current_dir()?,
        },
    )
    .await?;
    let ControlResponse::ProjectResolved { project, .. } = response else {
        bail!("unexpected Project routing response");
    };
    Ok(project)
}
async fn task_scope(state: &Path, task: TaskSelector) -> Result<Scope> {
    Ok(Scope::task(
        resolve(state, task.project).await?,
        task.goal,
        task.task,
    ))
}
async fn scope(state: &Path, scope: ScopeSelector) -> Result<Scope> {
    ensure!(
        scope.task.is_none() || scope.goal.is_some(),
        "--task requires --goal"
    );
    Ok(Scope {
        project_id: resolve(state, scope.project).await?,
        goal_id: scope.goal,
        task_id: scope.task,
    })
}
async fn action(state: &Path, request: ReadRequest) -> Result<ControlAction> {
    Ok(match request {
        ReadRequest::Goals {
            project,
            after,
            maximum,
        } => ControlAction::ProjectGoals {
            project: resolve(state, project).await?,
            after,
            maximum,
        },
        ReadRequest::Review {
            task,
            after,
            maximum,
        } => ControlAction::TaskReview {
            scope: task_scope(state, task).await?,
            after,
            maximum,
        },
        ReadRequest::Sessions {
            task,
            after,
            maximum,
        } => ControlAction::TaskSessions {
            scope: task_scope(state, task).await?,
            after,
            maximum,
        },
        ReadRequest::Attention {
            project,
            after,
            maximum,
        } => ControlAction::AttentionQueue {
            project: match project {
                Some(selector) => Some(resolve(state, Some(selector)).await?),
                None => None,
            },
            after,
            maximum,
        },
        ReadRequest::Events {
            scope: selected,
            after,
            maximum,
        } => ControlAction::Events {
            scope: scope(state, selected).await?,
            after,
            maximum,
        },
        ReadRequest::Routing { scope: selected } => ControlAction::Routing {
            scope: scope(state, selected).await?,
        },
        ReadRequest::Metrics { scope: selected } => ControlAction::Metrics {
            scope: scope(state, selected).await?,
        },
    })
}

/// `review:<id>` or `approval:<id>`.
pub fn parse_review_cursor(value: &str) -> Result<ReviewCursor> {
    let (kind, id) = value
        .split_once(':')
        .context("review cursor is review:<id> or approval:<id>")?;
    let kind = match kind {
        "review" => ReviewKind::Review,
        "approval" => ReviewKind::Approval,
        _ => bail!("review cursor kind is review or approval"),
    };
    Ok(ReviewCursor {
        kind,
        id: id
            .parse::<RecordId>()
            .map_err(|_| anyhow::anyhow!("review cursor ID invalid"))?,
    })
}
/// `<project>:<goal>:goal`, `<project>:<goal>:task:<task>` or
/// `<project>:<goal>:scanned` (a position after that Goal).
pub fn parse_attention_cursor(value: &str) -> Result<AttentionCursor> {
    let parts: Vec<&str> = value.split(':').collect();
    let id = |part: &str| -> Result<uuid::Uuid> {
        uuid::Uuid::parse_str(part).map_err(|_| anyhow::anyhow!("attention cursor ID invalid"))
    };
    let item = |lane, task| -> Result<AttentionCursor> {
        Ok(AttentionCursor::Item {
            project: ProjectId(id(parts[0])?),
            goal: GoalId(id(parts[1])?),
            lane,
            task,
        })
    };
    match parts.as_slice() {
        [_, _, "goal"] => item(AttentionLane::Goal, None),
        [_, _, "task", task] => item(AttentionLane::Task, Some(TaskId(id(task)?))),
        [_, _, "scanned"] => Ok(AttentionCursor::ScannedThrough {
            project: ProjectId(id(parts[0])?),
            goal: GoalId(id(parts[1])?),
        }),
        _ => bail!(
            "attention cursor is <project>:<goal>:goal, <project>:<goal>:task:<task> or <project>:<goal>:scanned"
        ),
    }
}
fn attention_cursor(cursor: &AttentionCursor) -> String {
    match *cursor {
        AttentionCursor::Item {
            project,
            goal,
            task: Some(task),
            ..
        } => format!("{project}:{goal}:task:{task}"),
        AttentionCursor::Item { project, goal, .. } => format!("{project}:{goal}:goal"),
        AttentionCursor::ScannedThrough { project, goal } => format!("{project}:{goal}:scanned"),
    }
}

/// Stored user text is quoted with control characters escaped.
fn quoted(value: &str) -> String {
    let mut output = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            ch if ch.is_control() => write!(output, "\\u{:04x}", ch as u32).unwrap(),
            ch => output.push(ch),
        }
    }
    output.push('"');
    output
}
/// Only fixed typed values use this, never user text.
fn wire(value: &impl serde::Serialize) -> String {
    serde_json::to_string(value).expect("typed value serialization")
}
fn scope_text(scope: &Scope) -> String {
    let mut text = scope.project_id.to_string();
    if let Some(goal) = scope.goal_id {
        write!(text, "/{goal}").unwrap();
    }
    if let Some(task) = scope.task_id {
        write!(text, "/{task}").unwrap();
    }
    text
}

/// An S4 page: an independent scoped observation, never a combined snapshot.
pub fn render(response: &ControlResponse, json: bool) -> Result<String> {
    if json {
        return Ok(serde_json::to_string_pretty(&serde_json::json!({
            "observation": "independent_scoped_observation",
            "complete": false,
            "facts": response,
        }))?);
    }
    let mut output = String::new();
    let next = |output: &mut String, next: Option<String>| {
        if let Some(next) = next {
            writeln!(output, "More: --after {next}").unwrap();
        }
    };
    match response {
        ControlResponse::ProjectGoalPage {
            project,
            project_version,
            goals,
            next: after,
        } => {
            writeln!(output, "Project: {project} version {project_version}")?;
            for goal in goals {
                writeln!(
                    output,
                    "Goal {} version {} state {} accepted {} tasks {} title {}",
                    goal.id,
                    goal.version,
                    wire(&goal.state),
                    goal.accepted,
                    goal.task_count,
                    goal.title.as_deref().map_or("unavailable".into(), quoted)
                )?;
            }
            next(&mut output, after.map(|g| g.to_string()));
        }
        ControlResponse::TaskReviewPage {
            task,
            task_version,
            items,
            next: after,
            decision,
        } => {
            writeln!(output, "Task: {} version {task_version}", scope_text(task))?;
            for item in items {
                writeln!(
                    output,
                    "{} {} version {} created {} updated {}",
                    wire(&item.kind),
                    item.id,
                    item.version,
                    item.created_at,
                    item.updated_at
                )?;
            }
            writeln!(output, "Decision: {}", wire(decision))?;
            next(
                &mut output,
                after.map(|c| format!("{}:{}", c.kind.key(), c.id)),
            );
        }
        ControlResponse::TaskSessionPage {
            task,
            task_version,
            sessions,
            next: after,
        } => {
            writeln!(output, "Task: {} version {task_version}", scope_text(task))?;
            for session in sessions {
                writeln!(
                    output,
                    "Session {} agent {} provider {} role {} state {} started {}",
                    session.id,
                    quoted(&session.agent),
                    quoted(&session.provider),
                    wire(&session.role),
                    wire(&session.state),
                    session.started_at
                )?;
            }
            next(&mut output, after.map(|s| s.to_string()));
        }
        ControlResponse::AttentionPage {
            project,
            items,
            next: after,
        } => {
            match project {
                Some(project) => writeln!(output, "Attention: Project {project}")?,
                None => writeln!(output, "Attention: all Projects")?,
            }
            for item in items {
                writeln!(
                    output,
                    "{} goal state {}{}",
                    attention_cursor(&item.cursor()),
                    wire(&item.goal_state),
                    item.task_state
                        .map(|s| format!(" task state {}", wire(&s)))
                        .unwrap_or_default()
                )?;
                for fact in &item.facts {
                    writeln!(output, "  fact {}", wire(fact))?;
                }
                for operation in &item.operations {
                    writeln!(
                        output,
                        "  operation {} expected version {} {}",
                        wire(&operation.kind),
                        operation.expected_version,
                        wire(&operation.availability)
                    )?;
                }
            }
            writeln!(
                output,
                "Operations are advisory; each is a new control request checked again."
            )?;
            next(&mut output, after.as_ref().map(attention_cursor));
        }
        ControlResponse::EventPage {
            scope,
            events,
            next: after,
        } => {
            writeln!(output, "Events: {}", scope_text(scope))?;
            for event in events {
                writeln!(
                    output,
                    "{} at {} {} {}",
                    event.sequence,
                    event.at,
                    wire(&event.kind),
                    scope_text(&event.scope)
                )?;
            }
            next(&mut output, after.map(|s| s.to_string()));
        }
        _ => bail!("unexpected read response"),
    }
    Ok(output.trim_end().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursors_round_trip_and_refuse_malformed_text() {
        let (project, goal) = (ProjectId::new(), GoalId::new());
        for cursor in [
            AttentionCursor::Item {
                project,
                goal,
                lane: AttentionLane::Task,
                task: Some(TaskId::new()),
            },
            AttentionCursor::Item {
                project,
                goal,
                lane: AttentionLane::Goal,
                task: None,
            },
            AttentionCursor::ScannedThrough { project, goal },
        ] {
            assert_eq!(
                parse_attention_cursor(&attention_cursor(&cursor)).unwrap(),
                cursor
            );
        }
        let id = RecordId::new();
        assert_eq!(
            parse_review_cursor(&format!("approval:{id}")).unwrap(),
            ReviewCursor {
                kind: ReviewKind::Approval,
                id
            }
        );
        for bad in ["", "x:y:goal", "a:b:task", "a:b:c:d:e"] {
            assert!(parse_attention_cursor(bad).is_err(), "{bad}");
        }
        for bad in ["", "review", "decision:x", &format!("REVIEW:{id}")] {
            assert!(parse_review_cursor(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn stored_title_text_is_quoted_with_controls_escaped() {
        let page = ControlResponse::ProjectGoalPage {
            project: ProjectId::new(),
            project_version: 1,
            goals: vec![GoalView {
                id: GoalId::new(),
                project: ProjectId::new(),
                version: 1,
                state: crate::domain::GoalState::Running,
                title: Some("a\u{1b}[2J\"b".into()),
                accepted: true,
                task_count: 0,
            }],
            next: None,
        };
        let text = render(&page, false).unwrap();
        assert!(text.contains("title \"a\\u001b[2J\\\"b\""), "{text}");
        assert!(!text.contains('\u{1b}'));
    }
}
