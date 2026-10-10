//! S4 (HOW §4 D2–D9): read-only projections behind the shared surface.
//!
//! Every reader runs in one read transaction under the caller's short Store
//! lock, checks the owner and the exact indexed scope before projecting, and
//! writes nothing on success or refusal. No reader decodes `Record.data`,
//! `AuditEvent.data`, Usage or stored attention text into a response.
use super::super::*;
use super::goals::{
    current_goal, goal_attention, owner_current, preparation_history, project_limit_stored,
    scoped_tasks,
};
use crate::runtime::control::{
    AttentionCursor, AttentionFact, AttentionItem, AttentionLane, AttentionOperationKind,
    AttentionOperationView, ControlResponse, EventKindView, EventView, GOAL_TITLE_BYTES, GoalView,
    HumanIngress, OperationAvailability, READ_PAGE_MAXIMUM, ReviewCursor, ReviewDecisionView,
    ReviewKind, ReviewRecordView, UnavailableReason, WorkflowWait, WorkflowWaitKind,
};
use crate::workflow::PhaseWaitingObservation;
use std::collections::BTreeMap;

/// D3: the per-response budget, well below `RESPONSE_BYTES`.
pub(crate) const READ_PAGE_BYTES: usize = 256 * 1024;
/// A Session body larger than this is refused, never projected.
const SESSION_BODY_BYTES: usize = 1024 * 1024;

/// A read answer: a page, or one of the finite typed unavailabilities.
pub(crate) type ReadAnswer = std::result::Result<ControlResponse, UnavailableReason>;

/// The current resume predicate inputs owned by the Runtime, not the Store.
pub(crate) struct ResumeInputs<'a> {
    pub(crate) policy_sha256: &'a str,
    pub(crate) stopping: bool,
}

pub(crate) enum Packed<T> {
    /// The packed rows and whether more rows remain after them.
    Page(Vec<T>, bool),
    /// The first row alone does not fit (D3: never truncated).
    TooLarge,
}
/// D3: packs `rows` in order while `fits(rows, more)` holds. `more_beyond`
/// says whether rows exist after the last candidate. A row that does not fit
/// ends the page there; a first row that does not fit is `TooLarge`.
pub(crate) fn pack<T>(
    rows: Vec<T>,
    more_beyond: bool,
    fits: impl Fn(&[T], bool) -> bool,
) -> Packed<T> {
    let total = rows.len();
    let mut page = Vec::with_capacity(total);
    for row in rows {
        page.push(row);
        let more = more_beyond || page.len() < total;
        if !fits(&page, more) {
            page.pop();
            if page.is_empty() {
                return Packed::TooLarge;
            }
            return Packed::Page(page, true);
        }
    }
    Packed::Page(page, more_beyond)
}
fn fits(response: &ControlResponse) -> bool {
    super::recorded::fits(response, READ_PAGE_BYTES)
}
fn maximum_valid(maximum: usize) -> Result<()> {
    ensure!(
        (1..=READ_PAGE_MAXIMUM).contains(&maximum),
        "read page maximum must be 1..128"
    );
    Ok(())
}
fn project_version(tx: &Transaction<'_>, project: ProjectId) -> Result<u64> {
    tx.query_row(
        "SELECT version FROM projects WHERE id=?1",
        [project.to_string()],
        |r| r.get(0),
    )
    .optional()?
    .context("unknown Project")
}
/// D9: the exact indexed Project/Goal/Task scope exists.
fn scope_exists(tx: &Transaction<'_>, scope: &Scope) -> Result<()> {
    validate_scope(scope)?;
    project_version(tx, scope.project_id)?;
    if let Some(goal) = scope.goal_id {
        let found: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM goals WHERE id=?1 AND project_id=?2)",
            params![goal.to_string(), scope.project_id.to_string()],
            |r| r.get(0),
        )?;
        ensure!(found, "unknown Goal in Project");
    }
    if let Some(task) = scope.task_id {
        task_version(tx, scope, task)?;
    }
    Ok(())
}
fn task_version(tx: &Transaction<'_>, scope: &Scope, task: TaskId) -> Result<u64> {
    let goal = scope.goal_id.context("Task scope requires Goal")?;
    tx.query_row(
        "SELECT version FROM tasks WHERE id=?1 AND goal_id=?2 AND project_id=?3",
        params![
            task.to_string(),
            goal.to_string(),
            scope.project_id.to_string()
        ],
        |r| r.get(0),
    )
    .optional()?
    .context("unknown Task in Goal")
}
/// D4/D5: a Task reader requires matching Project+Goal+Task.
fn exact_task(tx: &Transaction<'_>, scope: &Scope) -> Result<(GoalId, TaskId, u64)> {
    validate_scope(scope)?;
    let goal = scope.goal_id.context("Task reader requires Goal")?;
    let task = scope.task_id.context("Task reader requires Task")?;
    Ok((goal, task, task_version(tx, scope, task)?))
}
fn parse<T: std::str::FromStr>(value: &str) -> Result<T> {
    value
        .parse()
        .map_err(|_| anyhow::anyhow!("stored identity invalid"))
}
/// A typed state stored as its serialized name.
fn stored_state<T: serde::de::DeserializeOwned>(value: Option<String>) -> Result<T> {
    serde_json::from_value(Value::String(value.context("stored state missing")?))
        .map_err(|_| anyhow::anyhow!("stored state invalid"))
}
/// Ends a read transaction that wrote nothing, then returns `answer`.
fn finish(tx: Transaction<'_>, answer: ReadAnswer) -> Result<ReadAnswer> {
    tx.commit()?;
    Ok(answer)
}

impl Store {
    /// D2: Project → Goal page.
    pub(crate) fn runtime_project_goals(
        &self,
        ingress: &HumanIngress,
        project: ProjectId,
        after: Option<GoalId>,
        maximum: usize,
    ) -> Result<ReadAnswer> {
        maximum_valid(maximum)?;
        let tx = self.connection.unchecked_transaction()?;
        owner_current(&tx, ingress)?;
        let project_version = project_version(&tx, project)?;
        if let Some(after) = after {
            let present: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM goals WHERE id=?1 AND project_id=?2)",
                params![after.to_string(), project.to_string()],
                |r| r.get(0),
            )?;
            if !present {
                return finish(tx, Err(UnavailableReason::ReadCursorInvalid));
            }
        }
        let mut statement = tx.prepare(PROJECT_GOALS_PAGE_SQL)?;
        let rows = statement
            .query_map(
                params![
                    project.to_string(),
                    after.map(|g| g.to_string()).unwrap_or_default(),
                    GOAL_TITLE_BYTES as i64,
                    maximum as i64 + 1
                ],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, u64>(1)?,
                        r.get::<_, Option<String>>(2)?,
                        r.get::<_, Option<String>>(3)?,
                        r.get::<_, Option<u64>>(4)?,
                        r.get::<_, Option<String>>(5)?,
                        r.get::<_, Option<String>>(6)?,
                        r.get::<_, String>(7)?,
                        r.get::<_, bool>(8)?,
                        r.get::<_, usize>(9)?,
                    ))
                },
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(statement);
        let more_beyond = rows.len() > maximum;
        let mut goals = Vec::with_capacity(maximum);
        for (
            id,
            version,
            body_id,
            body_project,
            body_version,
            state,
            title,
            title_type,
            accepted,
            task_count,
        ) in rows.into_iter().take(maximum)
        {
            ensure!(
                body_id.as_deref() == Some(id.as_str())
                    && body_project.as_deref() == Some(project.to_string().as_str())
                    && body_version == Some(version)
                    && title_type == "text",
                "Goal body/index identity differs"
            );
            goals.push(GoalView {
                id: parse(&id)?,
                project,
                version,
                state: stored_state(state)?,
                title,
                accepted,
                task_count,
            });
        }
        let page = |goals: &[GoalView], more: bool| ControlResponse::ProjectGoalPage {
            project,
            project_version,
            goals: goals.to_vec(),
            next: more.then(|| goals.last().map(|g| g.id)).flatten(),
        };
        let answer = match pack(goals, more_beyond, |goals, more| fits(&page(goals, more))) {
            Packed::Page(goals, more) => Ok(page(&goals, more)),
            Packed::TooLarge => Err(UnavailableReason::ReadProjectionTooLarge),
        };
        finish(tx, answer)
    }

    /// D4: Task Review/Approval metadata page.
    pub(crate) fn runtime_task_review(
        &self,
        ingress: &HumanIngress,
        scope: &Scope,
        after: Option<ReviewCursor>,
        maximum: usize,
    ) -> Result<ReadAnswer> {
        maximum_valid(maximum)?;
        let tx = self.connection.unchecked_transaction()?;
        owner_current(&tx, ingress)?;
        let (goal, task, task_version) = exact_task(&tx, scope)?;
        if let Some(cursor) = after {
            let present: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM records WHERE id=?4 AND kind=?5 AND project_id=?1 AND goal_id=?2 AND task_id=?3)",
                params![
                    scope.project_id.to_string(),
                    goal.to_string(),
                    task.to_string(),
                    cursor.id.to_string(),
                    cursor.kind.key()
                ],
                |r| r.get(0),
            )?;
            if !present {
                return finish(tx, Err(UnavailableReason::ReadCursorInvalid));
            }
        }
        let order = "CASE kind WHEN 'review' THEN 0 ELSE 1 END";
        let mut statement = tx.prepare(&format!(
            "SELECT id,kind,version,json_extract(body,'$.id'),json_extract(body,'$.kind'),
             json_extract(body,'$.version'),json_extract(body,'$.created_at'),json_extract(body,'$.updated_at'),
             json_extract(body,'$.scope.project_id'),json_extract(body,'$.scope.goal_id'),json_extract(body,'$.scope.task_id')
             FROM records WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind IN ('review','approval')
             AND (?4 IS NULL OR {order}>?4 OR ({order}=?4 AND id>?5))
             ORDER BY {order},id LIMIT ?6"
        ))?;
        let rows = statement
            .query_map(
                params![
                    scope.project_id.to_string(),
                    goal.to_string(),
                    task.to_string(),
                    after.map(|c| match c.kind {
                        ReviewKind::Review => 0,
                        ReviewKind::Approval => 1,
                    }),
                    after.map(|c| c.id.to_string()),
                    maximum as i64 + 1
                ],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, u64>(2)?,
                        r.get::<_, Option<String>>(3)?,
                        r.get::<_, Option<String>>(4)?,
                        r.get::<_, Option<u64>>(5)?,
                        r.get::<_, Option<i64>>(6)?,
                        r.get::<_, Option<i64>>(7)?,
                        r.get::<_, Option<String>>(8)?,
                        r.get::<_, Option<String>>(9)?,
                        r.get::<_, Option<String>>(10)?,
                    ))
                },
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(statement);
        let more_beyond = rows.len() > maximum;
        let mut items = Vec::with_capacity(maximum);
        for (id, kind, version, body_id, body_kind, body_version, created, updated, p, g, t) in
            rows.into_iter().take(maximum)
        {
            let (kind, stored) = match kind.as_str() {
                "review" => (ReviewKind::Review, "REVIEW"),
                "approval" => (ReviewKind::Approval, "APPROVAL"),
                _ => bail!("Review record kind differs"),
            };
            ensure!(
                body_id.as_deref() == Some(id.as_str())
                    && body_kind.as_deref() == Some(stored)
                    && body_version == Some(version)
                    && p == Some(scope.project_id.to_string())
                    && g == Some(goal.to_string())
                    && t == Some(task.to_string()),
                "Review record body/index identity differs"
            );
            items.push(ReviewRecordView {
                id: parse(&id)?,
                kind,
                version,
                created_at: created.context("Review record time missing")?,
                updated_at: updated.context("Review record time missing")?,
            });
        }
        let decision = ReviewDecisionView::Unavailable {
            reason: UnavailableReason::ReviewDecisionUnqualified,
        };
        let page = |items: &[ReviewRecordView], more: bool| ControlResponse::TaskReviewPage {
            task: scope.clone(),
            task_version,
            items: items.to_vec(),
            next: more
                .then(|| {
                    items.last().map(|i| ReviewCursor {
                        kind: i.kind,
                        id: i.id,
                    })
                })
                .flatten(),
            decision,
        };
        let answer = match pack(items, more_beyond, |items, more| fits(&page(items, more))) {
            Packed::Page(items, more) => Ok(page(&items, more)),
            Packed::TooLarge => Err(UnavailableReason::ReadProjectionTooLarge),
        };
        finish(tx, answer)
    }

    /// D5: the Task's own Sessions as S3 `SessionView`s.
    pub(crate) fn runtime_task_sessions(
        &self,
        ingress: &HumanIngress,
        scope: &Scope,
        after: Option<SessionId>,
        maximum: usize,
    ) -> Result<ReadAnswer> {
        maximum_valid(maximum)?;
        let tx = self.connection.unchecked_transaction()?;
        owner_current(&tx, ingress)?;
        let (goal, task, task_version) = exact_task(&tx, scope)?;
        if let Some(after) = after {
            let present: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM records WHERE id=?4 AND kind='session' AND project_id=?1 AND goal_id=?2 AND task_id=?3)",
                params![
                    scope.project_id.to_string(),
                    goal.to_string(),
                    task.to_string(),
                    after.to_string()
                ],
                |r| r.get(0),
            )?;
            if !present {
                return finish(tx, Err(UnavailableReason::ReadCursorInvalid));
            }
        }
        let mut statement = tx.prepare(
            "SELECT id,version,length(CAST(body AS BLOB)),CASE WHEN length(CAST(body AS BLOB))<=?5 THEN body END
             FROM records WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind='session'
             AND (?4 IS NULL OR id>?4) ORDER BY id LIMIT ?6",
        )?;
        let rows = statement
            .query_map(
                params![
                    scope.project_id.to_string(),
                    goal.to_string(),
                    task.to_string(),
                    str_id(after),
                    SESSION_BODY_BYTES as i64,
                    maximum as i64 + 1
                ],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, u64>(1)?,
                        r.get::<_, usize>(2)?,
                        r.get::<_, Option<String>>(3)?,
                    ))
                },
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(statement);
        let more_beyond = rows.len() > maximum;
        let mut sessions = Vec::with_capacity(maximum);
        for (id, version, _, body) in rows.into_iter().take(maximum) {
            let record: Record = decode(body.context("Session body exceeds bound")?)
                .map_err(|_| anyhow::anyhow!("invalid persisted Session"))?;
            ensure!(
                record.id.to_string() == id
                    && record.kind == RecordKind::Session
                    && record.version == version
                    && record.scope == *scope,
                "Session body/index identity differs"
            );
            let session: Session = serde_json::from_value(record.data)
                .map_err(|_| anyhow::anyhow!("invalid persisted Session"))?;
            ensure!(
                session.id.0 == record.id.0 && session.scope == *scope,
                "Session body/index identity differs"
            );
            sessions.push(crate::project::SessionView::from(&session));
        }
        let page = |sessions: &[crate::project::SessionView], more: bool| {
            ControlResponse::TaskSessionPage {
                task: scope.clone(),
                task_version,
                sessions: sessions.to_vec(),
                next: more.then(|| sessions.last().map(|s| s.id)).flatten(),
            }
        };
        let answer = match pack(sessions, more_beyond, |s, more| fits(&page(s, more))) {
            Packed::Page(sessions, more) => Ok(page(&sessions, more)),
            Packed::TooLarge => Err(UnavailableReason::ReadProjectionTooLarge),
        };
        finish(tx, answer)
    }

    /// D7: audit summaries with the existing descendant scope semantics.
    pub(crate) fn runtime_events(
        &self,
        ingress: &HumanIngress,
        scope: &Scope,
        after: Option<i64>,
        maximum: usize,
    ) -> Result<ReadAnswer> {
        maximum_valid(maximum)?;
        let tx = self.connection.unchecked_transaction()?;
        owner_current(&tx, ingress)?;
        scope_exists(&tx, scope)?;
        let filter = "project_id=?1 AND (?2 IS NULL OR goal_id=?2) AND (?3 IS NULL OR task_id=?3)";
        if let Some(after) = after {
            let present: bool = tx.query_row(
                &format!("SELECT EXISTS(SELECT 1 FROM audit WHERE sequence=?4 AND {filter})"),
                params![
                    scope.project_id.to_string(),
                    str_id(scope.goal_id),
                    str_id(scope.task_id),
                    after
                ],
                |r| r.get(0),
            )?;
            if !present {
                return finish(tx, Err(UnavailableReason::ReadCursorInvalid));
            }
        }
        // `data` is never selected; `kind` is mapped through the allowlist.
        let mut statement = tx.prepare(&format!(
            "SELECT sequence,goal_id,task_id,kind,at FROM audit WHERE {filter} AND sequence>?4 ORDER BY sequence LIMIT ?5"
        ))?;
        let rows = statement
            .query_map(
                params![
                    scope.project_id.to_string(),
                    str_id(scope.goal_id),
                    str_id(scope.task_id),
                    after.unwrap_or(0),
                    maximum as i64 + 1
                ],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, Option<String>>(1)?,
                        r.get::<_, Option<String>>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, i64>(4)?,
                    ))
                },
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(statement);
        let more_beyond = rows.len() > maximum;
        let mut events = Vec::with_capacity(maximum);
        for (sequence, goal, task, kind, at) in rows.into_iter().take(maximum) {
            events.push(EventView {
                sequence,
                scope: Scope {
                    project_id: scope.project_id,
                    goal_id: goal.as_deref().map(parse).transpose()?,
                    task_id: task.as_deref().map(parse).transpose()?,
                },
                kind: EventKindView::of(&kind),
                at,
            });
        }
        let page = |events: &[EventView], more: bool| ControlResponse::EventPage {
            scope: scope.clone(),
            events: events.to_vec(),
            next: more.then(|| events.last().map(|e| e.sequence)).flatten(),
        };
        let answer = match pack(events, more_beyond, |e, more| fits(&page(e, more))) {
            Packed::Page(events, more) => Ok(page(&events, more)),
            Packed::TooLarge => Err(UnavailableReason::ReadProjectionTooLarge),
        };
        finish(tx, answer)
    }

    /// D8: Routing and Metrics check the scope and read nothing else.
    pub(crate) fn runtime_unqualified_read(
        &self,
        ingress: &HumanIngress,
        scope: &Scope,
        reason: UnavailableReason,
    ) -> Result<ReadAnswer> {
        ensure!(
            matches!(
                reason,
                UnavailableReason::MetricsUnqualified | UnavailableReason::RoutingUnavailable
            ),
            "unqualified read reason invalid"
        );
        let tx = self.connection.unchecked_transaction()?;
        owner_current(&tx, ingress)?;
        scope_exists(&tx, scope)?;
        finish(tx, Err(reason))
    }

    /// D6 / D6-R3: the derived Goal+Task attention queue. It never calls
    /// `reconcile_runtime_attention` and never writes attention. One request
    /// evaluates at most `ATTENTION_GOALS` Goals and `ATTENTION_TASKS`
    /// Tasks (the first Goal always), seeking `goals_by_project` after its
    /// position; Project rows are read only for fetched Goals.
    pub(crate) fn runtime_attention_queue(
        &self,
        ingress: &HumanIngress,
        project: Option<ProjectId>,
        after: Option<AttentionCursor>,
        maximum: usize,
        resume: &ResumeInputs<'_>,
    ) -> Result<ReadAnswer> {
        maximum_valid(maximum)?;
        #[cfg(test)]
        observation::reset();
        let tx = self.connection.unchecked_transaction()?;
        owner_current(&tx, ingress)?;
        let epoch = ingress.identity().1;
        if let Some(project) = project {
            project_version(&tx, project)?;
        }
        // The cursor's Goal must be in the selected view's inventory.
        if let Some(cursor) = after {
            let (cursor_project, goal) = cursor.goal_position();
            let present: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM goals WHERE id=?1 AND project_id=?2)",
                params![goal.to_string(), cursor_project.to_string()],
                |r| r.get(0),
            )?;
            if !present || project.is_some_and(|p| p != cursor_project) {
                return finish(tx, Err(UnavailableReason::ReadCursorInvalid));
            }
        }
        // An `Item` position re-evaluates its own (validated) Goal first;
        // either position then seeks strictly after its Goal.
        let mut rows = Vec::new();
        if let Some(AttentionCursor::Item { project, goal, .. }) = after {
            #[cfg(test)]
            observation::goal_row();
            rows.push((project, goal));
        }
        let start = after.map(|c| c.goal_position());
        rows.extend(goal_seek(
            &tx,
            project,
            start,
            ATTENTION_GOALS + 1 - rows.len(),
        )?);
        let mut projects: BTreeMap<ProjectId, Project> = BTreeMap::new();
        let mut items: Vec<AttentionItem> = Vec::new();
        let (mut goals_evaluated, mut tasks_evaluated) = (0, 0);
        let mut last_evaluated = None;
        let mut budget_ended = rows.len() > ATTENTION_GOALS;
        for (goal_project, goal) in rows.iter().take(ATTENTION_GOALS).copied() {
            let task_count: usize = tx.query_row(
                "SELECT count(*) FROM tasks WHERE goal_id=?1 AND project_id=?2",
                params![goal.to_string(), goal_project.to_string()],
                |r| r.get(0),
            )?;
            if goals_evaluated > 0 && tasks_evaluated + task_count > ATTENTION_TASKS {
                budget_ended = true;
                break;
            }
            if let std::collections::btree_map::Entry::Vacant(slot) = projects.entry(goal_project) {
                let stored: Project = read_tx(&tx, "projects", &goal_project.to_string())?
                    .context("unknown Project")?;
                ensure!(
                    stored.id == goal_project,
                    "Project body/index identity differs"
                );
                #[cfg(test)]
                observation::project_row();
                slot.insert(stored);
            }
            let mut derived = goal_items(&tx, &projects[&goal_project], goal, epoch, resume)?;
            if goals_evaluated == 0
                && let Some(cursor @ AttentionCursor::Item { .. }) = after
            {
                if !derived.iter().any(|i| i.cursor() == cursor) {
                    return finish(tx, Err(UnavailableReason::ReadCursorInvalid));
                }
                let AttentionCursor::Item {
                    project,
                    goal,
                    lane,
                    task,
                } = cursor
                else {
                    unreachable!("matched as an item position")
                };
                derived.retain(|i| i.key() > (project, goal, lane, task));
            }
            goals_evaluated += 1;
            tasks_evaluated += task_count;
            #[cfg(test)]
            observation::evaluated(task_count);
            last_evaluated = Some((goal_project, goal));
            items.extend(derived);
            if items.len() > maximum {
                break;
            }
        }
        // Where the next page starts: after the last item when items remain
        // or were cut, else after the last evaluated Goal when the budget
        // ended the scan, else nowhere.
        let items_remain = items.len() > maximum;
        items.truncate(maximum);
        let total = items.len();
        let scanned = (!items_remain && budget_ended)
            .then_some(last_evaluated)
            .flatten()
            .map(|(project, goal)| AttentionCursor::ScannedThrough { project, goal });
        let next = |items: &[AttentionItem], more: bool| -> Option<AttentionCursor> {
            if !more {
                return None;
            }
            match scanned {
                Some(position) if items.len() == total => Some(position),
                _ => items.last().map(AttentionItem::cursor),
            }
        };
        let page = |items: &[AttentionItem], more: bool| ControlResponse::AttentionPage {
            project,
            items: items.to_vec(),
            next: next(items, more),
        };
        let more_beyond = items_remain || scanned.is_some();
        let answer = match pack(items, more_beyond, |i, more| fits(&page(i, more))) {
            Packed::Page(items, more) => Ok(page(&items, more)),
            Packed::TooLarge => Err(UnavailableReason::ReadProjectionTooLarge),
        };
        finish(tx, answer)
    }
}

/// D2 / D6-R2 R2'': the Project→Goal page, a `goals_by_project` seek; only
/// fixed fields are extracted and the body is never decoded whole.
pub(crate) const PROJECT_GOALS_PAGE_SQL: &str = "SELECT g.id,g.version,json_extract(g.body,'$.id'),json_extract(g.body,'$.project_id'),
             json_extract(g.body,'$.version'),json_extract(g.body,'$.state'),
             CASE WHEN typeof(json_extract(g.body,'$.title'))='text'
              AND length(CAST(json_extract(g.body,'$.title') AS BLOB))<=?3
              THEN json_extract(g.body,'$.title') END,
             typeof(json_extract(g.body,'$.title')),
             EXISTS(SELECT 1 FROM goal_authority a WHERE a.goal_id=g.id AND a.project_id=g.project_id),
             (SELECT count(*) FROM tasks t WHERE t.goal_id=g.id AND t.project_id=g.project_id)
             FROM goals g WHERE g.project_id=?1 AND g.id>?2 ORDER BY g.id LIMIT ?4";
/// D6-R3 R1: the per-request evaluation budget.
pub(crate) const ATTENTION_GOALS: usize = 64;
pub(crate) const ATTENTION_TASKS: usize = 1024;
/// D6-R2 R2'': the Goal seeks over `goals_by_project`, shared with the
/// query-plan controls so the checked text is the production text.
pub(crate) const PROJECT_GOAL_SEEK_SQL: &str =
    "SELECT id,project_id FROM goals WHERE project_id=?1 AND id>?2 ORDER BY id LIMIT ?3";
pub(crate) const RUNTIME_GOAL_SEEK_SQL: &str =
    "SELECT id,project_id FROM goals WHERE (project_id,id)>(?1,?2) ORDER BY project_id,id LIMIT ?3";

/// Goal positions strictly after `start`, in `(project, goal)` order, at
/// most `limit` rows, by an index seek on `goals_by_project`.
fn goal_seek(
    tx: &Transaction<'_>,
    project: Option<ProjectId>,
    start: Option<(ProjectId, GoalId)>,
    limit: usize,
) -> Result<Vec<(ProjectId, GoalId)>> {
    let after = |goal: Option<GoalId>| goal.map(|g| g.to_string()).unwrap_or_default();
    let (sql, first, second) = match project {
        Some(project) => (
            PROJECT_GOAL_SEEK_SQL,
            project.to_string(),
            after(start.map(|(_, g)| g)),
        ),
        None => (
            RUNTIME_GOAL_SEEK_SQL,
            start.map(|(p, _)| p.to_string()).unwrap_or_default(),
            after(start.map(|(_, g)| g)),
        ),
    };
    let mut statement = tx.prepare(sql)?;
    let mut rows = statement.query(params![first, second, limit as i64])?;
    let mut positions = Vec::new();
    while let Some(row) = rows.next()? {
        #[cfg(test)]
        observation::goal_row();
        let (id, project): (String, String) = (row.get(0)?, row.get(1)?);
        positions.push((parse(&project)?, parse(&id)?));
    }
    Ok(positions)
}

/// D6-R3 R5'/R10: per-request observations for the controls only. They
/// count rows and evaluations; no authority, ownership or outcome reads them.
#[cfg(test)]
pub(crate) mod observation {
    use std::cell::Cell;
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub(crate) struct Read {
        pub(crate) goal_rows: usize,
        pub(crate) project_rows: usize,
        pub(crate) goals_evaluated: usize,
        pub(crate) tasks_evaluated: usize,
    }
    thread_local! { static LAST: Cell<Read> = Cell::new(Read::default()); }
    fn update(f: impl FnOnce(&mut Read)) {
        LAST.with(|last| {
            let mut read = last.get();
            f(&mut read);
            last.set(read);
        });
    }
    pub(super) fn reset() {
        LAST.with(|last| last.set(Read::default()));
    }
    pub(super) fn goal_row() {
        update(|r| r.goal_rows += 1);
    }
    pub(super) fn project_row() {
        update(|r| r.project_rows += 1);
    }
    pub(super) fn evaluated(tasks: usize) {
        update(|r| {
            r.goals_evaluated += 1;
            r.tasks_evaluated += tasks;
        });
    }
    pub(crate) fn last() -> Read {
        LAST.with(Cell::get)
    }
}

/// D6: the queue items of one Goal, Goal lane first, then Tasks by ID.
fn goal_items(
    tx: &Transaction<'_>,
    project: &Project,
    goal: GoalId,
    epoch: u64,
    resume: &ResumeInputs<'_>,
) -> Result<Vec<AttentionItem>> {
    let limit = project_limit_stored(tx, project.id)?;
    let accepted: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM goal_authority WHERE goal_id=?1 AND project_id=?2)",
        params![goal.to_string(), project.id.to_string()],
        |r| r.get(0),
    )?;
    let repair = |goal: GoalId| AttentionOperationView {
        kind: AttentionOperationKind::ProjectSetTaskLimit,
        project: project.id,
        goal: Some(goal),
        task: None,
        expected_version: project.version,
        availability: OperationAvailability::Available,
    };
    let limit_fact = |stored| AttentionFact::ProjectLimitUnsupported { stored };
    if !accepted {
        // The same qualified reader `GoalStatus` uses for proposals.
        let ControlResponse::GoalProposalFacts {
            version,
            state,
            attention,
            project_limit_stored,
            ..
        } = super::proposals::proposal_facts(tx, project.id, goal)?
        else {
            bail!("proposal facts differ");
        };
        if goal_terminal(state) {
            return Ok(Vec::new());
        }
        let mut facts = vec![AttentionFact::GoalWithoutTasks];
        match project_limit_stored {
            Some(stored) => facts.push(limit_fact(stored)),
            None => facts.push(AttentionFact::Unavailable { reason: attention }),
        }
        // The lifecycle path refuses unaccepted Goals, so only the Project
        // repair is an operation here.
        let operations = project_limit_stored
            .map(|_| repair(goal))
            .into_iter()
            .collect();
        return Ok(vec![AttentionItem {
            project: project.id,
            goal,
            lane: AttentionLane::Goal,
            task: None,
            project_version: project.version,
            goal_version: version,
            goal_state: state,
            task_version: None,
            task_state: None,
            facts,
            operations,
        }]);
    }
    let current = current_goal(tx, project.id, goal)?;
    let tasks = scoped_tasks(tx, &current)?;
    let mut items = Vec::new();
    let state_fact = goal_state_fact(current.state);
    // A terminal Goal has no Goal item or lifecycle operation, but its
    // Tasks' unresolved facts stay in the queue (S4-SOL-M2): cancelling or
    // failing a Goal does not resolve a Task's wait.
    if !goal_terminal(current.state)
        && (tasks.is_empty() || state_fact.is_some() || limit.is_some())
    {
        let mut facts = Vec::new();
        if tasks.is_empty() {
            facts.push(AttentionFact::GoalWithoutTasks);
        }
        facts.extend(state_fact);
        match limit {
            Some(stored) => facts.push(limit_fact(stored)),
            None => facts.push(AttentionFact::Unavailable {
                reason: goal_attention(tx, project.id, goal, epoch, limit)?,
            }),
        }
        let mut operations = goal_operations(tx, project, &current, resume)?;
        operations.extend(limit.map(|_| repair(goal)));
        items.push(AttentionItem {
            project: project.id,
            goal,
            lane: AttentionLane::Goal,
            task: None,
            project_version: project.version,
            goal_version: current.version,
            goal_state: current.state,
            task_version: None,
            task_state: None,
            facts,
            operations,
        });
    }
    for task in tasks.iter().filter(|t| !task_terminal(t.state)) {
        let facts = task_facts(tx, task, &current, epoch)?;
        if facts.is_empty() {
            continue;
        }
        let observation = super::waiting::observe(tx, task, &current);
        // Retry/cancel are listed but unavailable until their genuine
        // producer exists; no Review/Approve operation is listed.
        let operations = [
            AttentionOperationKind::TaskRetry,
            AttentionOperationKind::TaskCancel,
        ]
        .into_iter()
        .map(|kind| AttentionOperationView {
            kind,
            project: project.id,
            goal: Some(goal),
            task: Some(task.id),
            expected_version: task.version,
            availability: OperationAvailability::Unavailable {
                reason: UnavailableReason::TaskDriverUnavailable,
            },
        })
        .collect();
        items.push(AttentionItem {
            project: project.id,
            goal,
            lane: AttentionLane::Task,
            task: Some(task.id),
            project_version: project.version,
            goal_version: current.version,
            goal_state: current.state,
            task_version: Some(task.version),
            task_state: Some(super::waiting::effective_state(task, observation.as_ref())),
            facts,
            operations,
        });
    }
    Ok(items)
}

/// D6: Goal lifecycle availability from the lifecycle path's own predicates.
fn goal_operations(
    tx: &Transaction<'_>,
    project: &Project,
    goal: &Goal,
    resume: &ResumeInputs<'_>,
) -> Result<Vec<AttentionOperationView>> {
    let operation = |kind, availability| AttentionOperationView {
        kind,
        project: project.id,
        goal: Some(goal.id),
        task: None,
        expected_version: goal.version,
        availability,
    };
    // A non-terminal Goal can always be paused, cancelled or failed.
    let mut operations = vec![
        operation(
            AttentionOperationKind::GoalPause,
            OperationAvailability::Available,
        ),
        operation(
            AttentionOperationKind::GoalCancel,
            OperationAvailability::Available,
        ),
        operation(
            AttentionOperationKind::GoalFail,
            OperationAvailability::Available,
        ),
    ];
    if matches!(
        goal.state,
        GoalState::Paused | GoalState::Blocked | GoalState::WaitingHuman
    ) {
        let saved_policy: String = tx.query_row(
            "SELECT policy_sha256 FROM goal_authority WHERE goal_id=?1 AND project_id=?2",
            params![goal.id.to_string(), project.id.to_string()],
            |r| r.get(0),
        )?;
        let held = OperationAvailability::Unavailable {
            reason: UnavailableReason::GoalResumeHeld,
        };
        let availability = if resume.stopping || saved_policy != resume.policy_sha256 {
            held
        } else if preparation_history(tx, project.id, goal.id)? > 0 {
            OperationAvailability::Unavailable {
                reason: UnavailableReason::FreshBootstrapRecoveryUnavailable,
            }
        } else if project.state != ProjectState::Registered {
            held
        } else {
            OperationAvailability::Available
        };
        operations.push(operation(AttentionOperationKind::GoalResume, availability));
    }
    Ok(operations)
}

/// D6: one Task's typed facts from current Task/Goal/scheduler/execution rows.
fn task_facts(
    tx: &Transaction<'_>,
    task: &Task,
    goal: &Goal,
    epoch: u64,
) -> Result<Vec<AttentionFact>> {
    let mut facts = Vec::new();
    facts.extend(super::waiting::observe(tx, task, goal).and_then(|o| wait_fact(&o)));
    facts.extend(task_state_fact(task.state));
    // The same fallback `GoalTasks` applies to an unreadable Workflow.
    let wait = super::waiting::workflow_wait(tx, task).unwrap_or_else(|_| {
        Some(WorkflowWait {
            kind: WorkflowWaitKind::Held,
            detail: "workflow held".into(),
        })
    });
    facts.extend(wait.map(|wait| AttentionFact::WorkflowWait { wait }));
    // Stored text is classified in SQL and never read out: 0 none, 1 the
    // fixed binding hold, 2 anything else.
    let scheduler: Option<(u8, bool)> = tx
        .query_row(
            "SELECT CASE WHEN s.attention IS NULL THEN 0 WHEN s.attention=?4 THEN 1 ELSE 2 END,EXISTS(SELECT 1 FROM task_drivers d WHERE d.task_id=s.task_id AND d.goal_id=s.goal_id AND d.project_id=s.project_id AND d.owner_epoch=?5 AND d.state='driving') FROM scheduler_tasks s WHERE s.task_id=?1 AND s.goal_id=?2 AND s.project_id=?3",
            params![
                task.id.to_string(),
                task.goal_id.to_string(),
                task.project_id.to_string(),
                super::service::native_binding_hold(),
                epoch
            ],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    if let Some((attention, driving)) = scheduler {
        facts.extend(scheduler_fact(attention, driving));
    }
    Ok(facts)
}

fn goal_state_fact(state: GoalState) -> Option<AttentionFact> {
    match state {
        GoalState::WaitingHuman => Some(AttentionFact::GoalWaitingHuman),
        GoalState::Paused => Some(AttentionFact::GoalPaused),
        GoalState::Blocked => Some(AttentionFact::GoalBlocked),
        _ => None,
    }
}
fn task_state_fact(state: TaskState) -> Option<AttentionFact> {
    match state {
        TaskState::WaitingHuman => Some(AttentionFact::TaskWaitingHuman),
        TaskState::WaitingApproval => Some(AttentionFact::TaskWaitingApproval),
        _ => None,
    }
}
/// `observe` qualifies only quota and capacity parking; `Held` is no fact.
fn wait_fact(observation: &PhaseWaitingObservation) -> Option<AttentionFact> {
    match observation {
        PhaseWaitingObservation::Waiting {
            reason: crate::execution::WaitReason::Quota,
            next_due,
        } => Some(AttentionFact::QuotaWait {
            next_due: *next_due,
        }),
        PhaseWaitingObservation::Waiting {
            reason: crate::execution::WaitReason::Capacity,
            next_due,
        } => Some(AttentionFact::CapacityWait {
            next_due: *next_due,
        }),
        _ => None,
    }
}
/// Stored attention class (0 none, 1 the fixed binding hold, else other)
/// and whether a current-epoch Driver is driving. Only combinations the
/// typed readers reproduce are typed; every other one is unqualified.
fn scheduler_fact(attention: u8, driving: bool) -> Option<AttentionFact> {
    match (attention, driving) {
        (0, false) => None,
        (0, true) => Some(AttentionFact::Unavailable {
            reason: UnavailableReason::NativeContinuationUnavailable,
        }),
        (1, false) => Some(AttentionFact::Unavailable {
            reason: UnavailableReason::NativeBindingUnavailable,
        }),
        _ => Some(AttentionFact::StoredUnqualified),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::WaitReason;

    /// C-S4d (pure): states and waits with no producer reachable from a
    /// Runtime control test map to their fixed facts; nothing else does.
    #[test]
    fn c_s4d_fact_mappings_are_fixed() {
        assert_eq!(
            goal_state_fact(GoalState::WaitingHuman),
            Some(AttentionFact::GoalWaitingHuman)
        );
        assert_eq!(
            goal_state_fact(GoalState::Blocked),
            Some(AttentionFact::GoalBlocked)
        );
        assert_eq!(goal_state_fact(GoalState::Running), None);
        assert_eq!(
            task_state_fact(TaskState::WaitingHuman),
            Some(AttentionFact::TaskWaitingHuman)
        );
        assert_eq!(
            task_state_fact(TaskState::WaitingApproval),
            Some(AttentionFact::TaskWaitingApproval)
        );
        assert_eq!(task_state_fact(TaskState::Implementing), None);
        let waiting = |reason| PhaseWaitingObservation::Waiting {
            reason,
            next_due: 77,
        };
        assert_eq!(
            wait_fact(&waiting(WaitReason::Quota)),
            Some(AttentionFact::QuotaWait { next_due: 77 })
        );
        assert_eq!(
            wait_fact(&waiting(WaitReason::Capacity)),
            Some(AttentionFact::CapacityWait { next_due: 77 })
        );
        assert_eq!(wait_fact(&waiting(WaitReason::Approval)), None);
        assert_eq!(wait_fact(&PhaseWaitingObservation::Held), None);
        assert_eq!(scheduler_fact(0, false), None);
        assert_eq!(
            scheduler_fact(1, true),
            Some(AttentionFact::StoredUnqualified)
        );
        assert_eq!(
            scheduler_fact(2, false),
            Some(AttentionFact::StoredUnqualified)
        );
        assert_eq!(
            scheduler_fact(2, true),
            Some(AttentionFact::StoredUnqualified)
        );
    }

    #[derive(serde::Serialize)]
    struct Synthetic {
        blob: String,
    }
    fn budget_fits(budget: usize) -> impl Fn(&[Synthetic], bool) -> bool {
        move |rows, more| {
            super::super::recorded::fits(&serde_json::json!({"rows": rows, "more": more}), budget)
        }
    }
    fn size(rows: &[Synthetic], more: bool) -> usize {
        serde_json::to_vec(&serde_json::json!({"rows": rows, "more": more}))
            .unwrap()
            .len()
    }

    /// C-S4h (pure packer): a synthetic non-elidable public DTO larger than
    /// the item budget is `TooLarge`, never truncated; a page exactly at the
    /// byte boundary succeeds; a later row that does not fit ends the page.
    #[test]
    fn c_s4h_packer_refuses_oversize_and_packs_to_the_boundary() {
        let big = || Synthetic {
            blob: "x".repeat(READ_PAGE_BYTES),
        };
        assert!(matches!(
            pack(vec![big()], false, budget_fits(READ_PAGE_BYTES)),
            Packed::TooLarge
        ));
        let row = |n| Synthetic {
            blob: "y".repeat(n),
        };
        let exact = size(&[row(1000)], false);
        match pack(vec![row(1000)], false, budget_fits(exact)) {
            Packed::Page(rows, more) => {
                assert_eq!((rows.len(), more), (1, false));
                assert_eq!(rows[0].blob.len(), 1000, "row was altered");
            }
            Packed::TooLarge => panic!("boundary page refused"),
        }
        assert!(matches!(
            pack(vec![row(1000)], false, budget_fits(exact - 1)),
            Packed::TooLarge
        ));
        // Two rows where only the first fits: one row, more remain.
        let one = size(&[row(1000)], true);
        match pack(vec![row(1000), row(1000)], false, budget_fits(one)) {
            Packed::Page(rows, more) => assert_eq!((rows.len(), more), (1, true)),
            Packed::TooLarge => panic!("first row fits"),
        }
    }
}
