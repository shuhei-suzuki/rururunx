//! S4 controls (HOW §4, C-S4a–h): the read-only shared surface through the
//! actual Runtime and accepted Unix ingress. Positive rows are written only
//! through production Store/control paths; the one SQL write is the legacy
//! attention sentinel, a negative control that must never be reproduced.
use super::*;
use crate::domain::{
    Project, Record, Scope, Session, SessionId, SessionRole, SessionState, Task, Usage,
};

async fn answer(f: &ControlFixture, action: ControlAction) -> anyhow::Result<ControlResponse> {
    f.runtime.handle_control(&f.socket, f.request(action)).await
}
async fn read(f: &ControlFixture, action: ControlAction) -> ControlResponse {
    answer(f, action).await.unwrap()
}
fn unavailable(response: &ControlResponse) -> Option<UnavailableReason> {
    match response {
        ControlResponse::Unavailable { reason, .. } => Some(*reason),
        _ => None,
    }
}
fn text(response: &ControlResponse) -> String {
    let encoded = serde_json::to_string(response).unwrap();
    assert!(
        encoded.len() <= crate::state::READ_PAGE_BYTES,
        "response exceeds the S4 page budget"
    );
    encoded
}
/// A second registered Project with one accepted Goal of `count` Tasks.
async fn other_project(f: &ControlFixture, count: usize) -> (Project, GoalId, Vec<Task>) {
    let root = f._dir.path().join("other-source");
    std::fs::create_dir(&root).unwrap();
    let mut project = Project::new(
        "other".into(),
        root.canonicalize().unwrap(),
        "account-free-no-Git-other".into(),
        "main".into(),
    );
    f.store().lock().unwrap().put_project(&mut project).unwrap();
    let mut p = plan();
    let original = p.tasks[0].clone();
    p.tasks = (0..count)
        .map(|i| {
            let mut task = original.clone();
            task.key = format!("other-{i}");
            task
        })
        .collect();
    let ControlResponse::GoalAccepted { goal, .. } = read(
        f,
        ControlAction::CreateGoal {
            project: project.id,
            expected_project: project.version,
            plan: p,
        },
    )
    .await
    else {
        panic!("SETUP: other Project Goal")
    };
    let store = f.store();
    let store = store.lock().unwrap();
    let tasks = store
        .goal(goal)
        .unwrap()
        .unwrap()
        .dag
        .nodes
        .iter()
        .map(|id| store.task(*id).unwrap().unwrap())
        .collect();
    (project, goal, tasks)
}
fn record(f: &ControlFixture, scope: &Scope, kind: RecordKind, data: serde_json::Value) -> Record {
    let mut record = Record::new(scope.clone(), kind, data);
    f.store().lock().unwrap().put_record(&mut record).unwrap();
    record
}
fn session(f: &ControlFixture, scope: &Scope, role: SessionRole, marker: &str) -> Session {
    let session = Session {
        id: SessionId::new(),
        scope: scope.clone(),
        agent: "worker".into(),
        provider: "claude".into(),
        role,
        native_ref: Some(format!("NATIVE-{marker}")),
        pid: Some(424242),
        worktree: format!("/WORKTREE-{marker}").into(),
        state: SessionState::Running,
        model: Some(format!("MODEL-{marker}")),
        effort: Some(format!("EFFORT-{marker}")),
        recovery: serde_json::json!({ "secret": format!("RECOVERY-{marker}") }),
        started_at: 1234,
    };
    f.store().lock().unwrap().put_session(&session, 0).unwrap();
    session
}
async fn goal_pages(f: &ControlFixture, project: crate::domain::ProjectId) -> Vec<GoalView> {
    let mut after = None;
    let mut all = Vec::new();
    loop {
        let ControlResponse::ProjectGoalPage {
            project: id,
            goals,
            next,
            ..
        } = read(
            f,
            ControlAction::ProjectGoals {
                project,
                after,
                maximum: 2,
            },
        )
        .await
        else {
            panic!("ProjectGoalPage")
        };
        assert_eq!(id, project);
        all.extend(goals);
        match next {
            Some(next) => after = Some(next),
            None => return all,
        }
    }
}
fn proposal_of(response: ControlResponse) -> GoalId {
    match response {
        ControlResponse::GoalProposed { goal, .. } => goal,
        other => panic!("SETUP: proposal {other:?}"),
    }
}

/// C-S4a: `ProjectGoals` pages only the selected Project; `GoalTasks` pages
/// only its Goal; foreign/missing cursors refuse; a mutation between pages
/// shows as a version change, never as one combined snapshot.
#[tokio::test]
async fn c_s4a_hierarchy_pages_are_scoped_and_cursors_checked() {
    let f = fixture("claude", true);
    let (g1, tasks1) = accept(&f, 2).await;
    let (g2, _) = accept(&f, 1).await;
    let g3 = proposal_of(
        read(
            &f,
            ControlAction::ProposeGoal {
                project: f.project.id,
                expected_project: f.project.version,
                objective: "inert".into(),
            },
        )
        .await,
    );
    let (other, h1, other_tasks) = other_project(&f, 1).await;
    let mut expected = vec![g1, g2, g3];
    expected.sort();
    let goals = goal_pages(&f, f.project.id).await;
    assert_eq!(goals.iter().map(|g| g.id).collect::<Vec<_>>(), expected);
    assert!(goals.iter().all(|g| g.project == f.project.id));
    let by_id = |id| goals.iter().find(|g| g.id == id).unwrap();
    assert!(by_id(g1).accepted && by_id(g1).task_count == 2);
    assert!(!by_id(g3).accepted && by_id(g3).task_count == 0);
    assert_eq!(by_id(g1).title.as_deref(), Some("t"));
    assert_eq!(
        goal_pages(&f, other.id)
            .await
            .iter()
            .map(|g| g.id)
            .collect::<Vec<_>>(),
        [h1]
    );
    for after in [h1, GoalId::new()] {
        let response = read(
            &f,
            ControlAction::ProjectGoals {
                project: f.project.id,
                after: Some(after),
                maximum: 2,
            },
        )
        .await;
        assert_eq!(
            unavailable(&response),
            Some(UnavailableReason::ReadCursorInvalid)
        );
    }
    // The existing Goal→Task reader stays scoped to its Goal.
    for after in [other_tasks[0].id, crate::domain::TaskId::new()] {
        assert!(
            answer(
                &f,
                ControlAction::GoalTasks {
                    project: f.project.id,
                    goal: g1,
                    after: Some(after),
                    maximum: 2,
                    view: None,
                },
            )
            .await
            .is_err(),
            "foreign Task cursor accepted"
        );
    }
    let ControlResponse::GoalTaskPage { tasks, .. } = read(
        &f,
        ControlAction::GoalTasks {
            project: f.project.id,
            goal: g1,
            after: None,
            maximum: 128,
            view: None,
        },
    )
    .await
    else {
        panic!("GoalTaskPage")
    };
    let mut ids = tasks1.iter().map(|t| t.id).collect::<Vec<_>>();
    ids.sort();
    assert_eq!(
        tasks
            .iter()
            .map(|t| t.scope.task_id.unwrap())
            .collect::<Vec<_>>(),
        ids
    );
    // A change between pages is visible only through versions.
    let ControlResponse::ProjectGoalPage { goals: first, .. } = read(
        &f,
        ControlAction::ProjectGoals {
            project: f.project.id,
            after: None,
            maximum: 3,
        },
    )
    .await
    else {
        panic!("ProjectGoalPage")
    };
    let observed = first.iter().find(|g| g.id == g1).unwrap().version;
    assert!(matches!(
        read(
            &f,
            ControlAction::SetGoalLifecycle {
                project: f.project.id,
                goal: g1,
                expected_goal: observed,
                target: GoalControl::Pause,
                reason: "between pages".into(),
            },
        )
        .await,
        ControlResponse::GoalLifecycleChanged { .. }
    ));
    let again = goal_pages(&f, f.project.id).await;
    let changed = again.iter().find(|g| g.id == g1).unwrap();
    assert!(changed.version > observed && changed.state == crate::domain::GoalState::Paused);
    finish(f).await;
}

/// C-S4b: >128 Review/Approval rows are paged to the end exactly once in
/// stable order with metadata only; the decision stays unqualified.
#[tokio::test]
async fn c_s4b_review_pages_every_row_once_without_data_or_decision() {
    let f = fixture("claude", true);
    let (_, tasks) = accept(&f, 2).await;
    let scope = tasks[0].scope();
    let data = |i: usize| {
        serde_json::json!({
            "decision": "APPROVE",
            "reason": format!("REVIEW-SENTINEL-{i}"),
            "findings": [{"severity": "high", "text": "FINDING-SENTINEL"}],
            "reviewer": "REVIEWER-SENTINEL",
        })
    };
    let mut expected = Vec::new();
    for i in 0..140 {
        let kind = if i % 2 == 0 {
            RecordKind::Review
        } else {
            RecordKind::Approval
        };
        let r = record(&f, &scope, kind, data(i));
        expected.push((
            if kind == RecordKind::Review {
                ReviewKind::Review
            } else {
                ReviewKind::Approval
            },
            r.id,
            r.version,
        ));
    }
    expected.sort_by_key(|(kind, id, _)| (*kind, id.to_string()));
    let foreign = record(&f, &tasks[1].scope(), RecordKind::Review, data(999));
    let mut after = None;
    let mut seen = Vec::new();
    let mut pages = 0;
    loop {
        let response = read(
            &f,
            ControlAction::TaskReview {
                scope: scope.clone(),
                after,
                maximum: 64,
            },
        )
        .await;
        let encoded = text(&response);
        // `reason` is also the typed decision's own key; its value is fixed.
        for absent in [
            "SENTINEL",
            "APPROVE",
            "\"findings\"",
            "\"reviewer\"",
            "severity",
        ] {
            assert!(!encoded.contains(absent), "Review data leaked: {absent}");
        }
        let ControlResponse::TaskReviewPage {
            task,
            items,
            next,
            decision,
            ..
        } = response
        else {
            panic!("TaskReviewPage")
        };
        assert_eq!(task, scope);
        assert_eq!(
            decision,
            ReviewDecisionView::Unavailable {
                reason: UnavailableReason::ReviewDecisionUnqualified
            }
        );
        seen.extend(items.iter().map(|i| (i.kind, i.id, i.version)));
        pages += 1;
        match next {
            Some(next) => after = Some(next),
            None => break,
        }
    }
    assert_eq!(pages, 3, "140 rows at 64 per page");
    assert_eq!(seen, expected, "every row exactly once, in stable order");
    for cursor in [
        ReviewCursor {
            kind: ReviewKind::Review,
            id: foreign.id,
        },
        ReviewCursor {
            kind: if expected[0].0 == ReviewKind::Review {
                ReviewKind::Approval
            } else {
                ReviewKind::Review
            },
            id: expected[0].1,
        },
    ] {
        let response = read(
            &f,
            ControlAction::TaskReview {
                scope: scope.clone(),
                after: Some(cursor),
                maximum: 8,
            },
        )
        .await;
        assert_eq!(
            unavailable(&response),
            Some(UnavailableReason::ReadCursorInvalid)
        );
    }
    // A Task reader requires the exact Project+Goal+Task.
    let mut wrong = scope.clone();
    wrong.goal_id = Some(GoalId::new());
    assert!(
        answer(
            &f,
            ControlAction::TaskReview {
                scope: wrong,
                after: None,
                maximum: 8,
            },
        )
        .await
        .is_err()
    );
    finish(f).await;
}

/// C-S4c: exactly the S3 `SessionView` key set, for the requested Task's
/// own Sessions only.
#[tokio::test]
async fn c_s4c_sessions_project_only_the_task_session_view() {
    let f = fixture("claude", true);
    let (goal, tasks) = accept(&f, 2).await;
    let scope = tasks[0].scope();
    let own = session(&f, &scope, SessionRole::Reviewer, "OWN");
    session(&f, &tasks[1].scope(), SessionRole::Reviewer, "SIBLING");
    session(
        &f,
        &Scope::goal(f.project.id, goal),
        SessionRole::Consultant,
        "GOAL",
    );
    let response = read(
        &f,
        ControlAction::TaskSessions {
            scope: scope.clone(),
            after: None,
            maximum: 128,
        },
    )
    .await;
    let encoded = text(&response);
    for absent in [
        "NATIVE-",
        "WORKTREE-",
        "MODEL-",
        "EFFORT-",
        "RECOVERY-",
        "424242",
    ] {
        assert!(!encoded.contains(absent), "Session field leaked: {absent}");
    }
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    let sessions = value["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 1, "only the Task's own Session");
    let keys: std::collections::BTreeSet<_> =
        sessions[0].as_object().unwrap().keys().cloned().collect();
    assert_eq!(
        keys,
        [
            "agent",
            "id",
            "provider",
            "role",
            "started_at",
            "state",
            "task"
        ]
        .map(String::from)
        .into()
    );
    let ControlResponse::TaskSessionPage { sessions, next, .. } = response else {
        panic!("TaskSessionPage")
    };
    assert_eq!(sessions, [crate::project::SessionView::from(&own)]);
    assert_eq!(next, None);
    let response = read(
        &f,
        ControlAction::TaskSessions {
            scope,
            after: Some(SessionId::new()),
            maximum: 8,
        },
    )
    .await;
    assert_eq!(
        unavailable(&response),
        Some(UnavailableReason::ReadCursorInvalid)
    );
    finish(f).await;
}

fn items(response: ControlResponse) -> (Vec<AttentionItem>, Option<AttentionCursor>) {
    match response {
        ControlResponse::AttentionPage { items, next, .. } => (items, next),
        other => panic!("AttentionPage: {other:?}"),
    }
}
fn stored_attention(f: &ControlFixture, task: crate::domain::TaskId) -> Option<String> {
    raw(f)
        .query_row(
            "SELECT attention FROM scheduler_tasks WHERE task_id=?1",
            [task.to_string()],
            |r| r.get(0),
        )
        .unwrap()
}

/// C-S4d: the derived Goal+Task queue: a taskless proposal as a Goal item,
/// a paused Goal with its lifecycle operations, typed Task binding facts,
/// the Project-limit repair, and an arbitrary stored attention sentinel only
/// as `stored_unqualified`. Operations are advisory and re-checked.
#[tokio::test]
async fn c_s4d_attention_queue_derives_typed_items_and_operations() {
    let f = fixture("claude", true);
    let (running, running_tasks) = accept(&f, 2).await;
    let (paused, _) = accept(&f, 1).await;
    let proposal = proposal_of(
        read(
            &f,
            ControlAction::ProposeGoal {
                project: f.project.id,
                expected_project: f.project.version,
                objective: "no tasks yet".into(),
            },
        )
        .await,
    );
    let ControlResponse::GoalLifecycleChanged { version, .. } = read(
        &f,
        ControlAction::SetGoalLifecycle {
            project: f.project.id,
            goal: paused,
            expected_goal: 1,
            target: GoalControl::Pause,
            reason: "attention".into(),
        },
    )
    .await
    else {
        panic!("SETUP: pause")
    };
    // The production sweep writes the fixed binding hold.
    f.store()
        .lock()
        .unwrap()
        .reconcile_runtime_attention(f.owner.instance_id(), f.owner.epoch(), 0)
        .unwrap();
    assert!(stored_attention(&f, running_tasks[0].id).is_some());
    // Negative control: a legacy arbitrary value no typed reader reproduces.
    let sentinel = "{\"kind\":\"LEGACY-ATTENTION-SENTINEL\"}";
    raw(&f)
        .execute(
            "UPDATE scheduler_tasks SET attention=?1 WHERE task_id=?2",
            rusqlite::params![sentinel, running_tasks[1].id.to_string()],
        )
        .unwrap();
    let response = read(
        &f,
        ControlAction::AttentionQueue {
            project: Some(f.project.id),
            after: None,
            maximum: 128,
        },
    )
    .await;
    assert!(!text(&response).contains("LEGACY-ATTENTION-SENTINEL"));
    let (queue, next) = items(response);
    assert_eq!(next, None);
    let keys = queue.iter().map(|i| i.cursor().key()).collect::<Vec<_>>();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "queue order");
    let goal_item = |goal| {
        queue
            .iter()
            .find(|i| i.goal == goal && i.lane == AttentionLane::Goal)
    };
    let task_item = |task| queue.iter().find(|i| i.task == Some(task));
    // (1) The taskless proposal is a Goal item; no scheduler row is needed.
    let item = goal_item(proposal).expect("taskless proposal item");
    assert_eq!(
        item.facts,
        [
            AttentionFact::GoalWithoutTasks,
            AttentionFact::Unavailable {
                reason: UnavailableReason::PlanningUnavailable
            }
        ]
    );
    assert!(item.operations.is_empty(), "proposals have no lifecycle");
    // (2) The paused Goal lists its current lifecycle operations.
    let item = goal_item(paused).expect("paused Goal item");
    assert_eq!(item.goal_version, version);
    assert!(item.facts.contains(&AttentionFact::GoalPaused));
    let kinds = item
        .operations
        .iter()
        .map(|o| (o.kind, o.expected_version, o.availability))
        .collect::<Vec<_>>();
    for kind in [
        AttentionOperationKind::GoalPause,
        AttentionOperationKind::GoalCancel,
        AttentionOperationKind::GoalFail,
        AttentionOperationKind::GoalResume,
    ] {
        assert!(
            kinds.contains(&(kind, version, OperationAvailability::Available)),
            "{kind:?} at the observed version"
        );
    }
    // A running Goal with Tasks has no Goal item.
    assert!(goal_item(running).is_none());
    // (3) Task facts: the reproduced hold is typed; the sentinel is not.
    let item = task_item(running_tasks[0].id).expect("held Task item");
    assert!(item.facts.contains(&AttentionFact::Unavailable {
        reason: UnavailableReason::NativeBindingUnavailable
    }));
    for operation in &item.operations {
        assert_eq!(
            operation.availability,
            OperationAvailability::Unavailable {
                reason: UnavailableReason::TaskDriverUnavailable
            }
        );
        assert_eq!(operation.expected_version, running_tasks[0].version);
    }
    assert_eq!(
        item.operations.iter().map(|o| o.kind).collect::<Vec<_>>(),
        [
            AttentionOperationKind::TaskRetry,
            AttentionOperationKind::TaskCancel
        ]
    );
    let item = task_item(running_tasks[1].id).expect("sentinel Task item");
    assert_eq!(item.facts, [AttentionFact::StoredUnqualified]);
    assert_eq!(
        stored_attention(&f, running_tasks[1].id).as_deref(),
        Some(sentinel),
        "the read changed stored attention"
    );
    // Paging by cursor returns the same suffix; a stale cursor refuses.
    let first = read(
        &f,
        ControlAction::AttentionQueue {
            project: None,
            after: None,
            maximum: 1,
        },
    )
    .await;
    let (one, cursor) = items(first);
    assert_eq!(one.len(), 1);
    let (rest, _) = items(
        read(
            &f,
            ControlAction::AttentionQueue {
                project: Some(f.project.id),
                after: cursor,
                maximum: 128,
            },
        )
        .await,
    );
    assert_eq!(
        one.iter()
            .chain(&rest)
            .map(|i| i.cursor())
            .collect::<Vec<_>>(),
        queue.iter().map(|i| i.cursor()).collect::<Vec<_>>()
    );
    let stale = AttentionCursor {
        project: f.project.id,
        goal: running,
        lane: AttentionLane::Goal,
        task: None,
    };
    let response = read(
        &f,
        ControlAction::AttentionQueue {
            project: None,
            after: Some(stale),
            maximum: 8,
        },
    )
    .await;
    assert_eq!(
        unavailable(&response),
        Some(UnavailableReason::ReadCursorInvalid)
    );
    // Executing an operation is a new CAS-checked request: the observed
    // version resumes once; the same stale version then refuses.
    let resume = ControlAction::SetGoalLifecycle {
        project: f.project.id,
        goal: paused,
        expected_goal: version,
        target: GoalControl::Resume,
        reason: "operation".into(),
    };
    assert!(matches!(
        read(&f, resume.clone()).await,
        ControlResponse::GoalLifecycleChanged { .. }
    ));
    assert!(answer(&f, resume).await.is_err(), "stale operation applied");
    // Task retry stays unavailable through its own control.
    let response = read(
        &f,
        ControlAction::TaskRetry {
            scope: running_tasks[0].scope(),
            expected_task: running_tasks[0].version,
            reason: "operation".into(),
        },
    )
    .await;
    assert_eq!(
        unavailable(&response),
        Some(UnavailableReason::TaskDriverUnavailable)
    );
    finish(f).await;
}

/// C-S4d (4): a stored legacy limit is a Goal item with the Project repair
/// at the observed Project version.
#[tokio::test]
async fn c_s4d_project_limit_is_a_goal_item_with_repair() {
    let mut f = fixture("claude", true);
    let (goal, _) = accept(&f, 1).await;
    f.project = f
        .store()
        .lock()
        .unwrap()
        .project(f.project.id)
        .unwrap()
        .unwrap();
    f.project.max_tasks = 4;
    f.store()
        .lock()
        .unwrap()
        .put_project(&mut f.project)
        .unwrap();
    let (queue, _) = items(
        read(
            &f,
            ControlAction::AttentionQueue {
                project: Some(f.project.id),
                after: None,
                maximum: 128,
            },
        )
        .await,
    );
    let item = queue
        .iter()
        .find(|i| i.goal == goal && i.lane == AttentionLane::Goal)
        .expect("Project limit Goal item");
    assert!(
        item.facts
            .contains(&AttentionFact::ProjectLimitUnsupported { stored: 4 })
    );
    let repair = item
        .operations
        .iter()
        .find(|o| o.kind == AttentionOperationKind::ProjectSetTaskLimit)
        .expect("repair operation");
    assert_eq!(repair.expected_version, f.project.version);
    assert_eq!(repair.availability, OperationAvailability::Available);
    finish(f).await;
}

/// C-S4d (3): a real current-epoch Driver claim with no attention is the
/// typed continuation fact.
#[tokio::test]
async fn c_s4d_claimed_driver_is_native_continuation_unavailable() {
    let f = fixture("claude", true);
    let (_, tasks) = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while count(&f, "task_drivers") != 1 {
        assert!(
            tokio::time::Instant::now() < deadline,
            "SETUP: no Driver claim"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let (queue, _) = items(
        read(
            &f,
            ControlAction::AttentionQueue {
                project: None,
                after: None,
                maximum: 128,
            },
        )
        .await,
    );
    let item = queue
        .iter()
        .find(|i| i.task == Some(tasks[0].id))
        .expect("claimed Task item");
    assert!(item.facts.contains(&AttentionFact::Unavailable {
        reason: UnavailableReason::NativeContinuationUnavailable
    }));
    let _ = f.runtime.shutdown().await;
    finish(f).await;
}

/// C-S4e: known kinds map to fixed values, every other kind (short and
/// printable included) is `event_unavailable`, and no data appears.
#[tokio::test]
async fn c_s4e_event_kinds_are_allowlisted_and_data_is_never_returned() {
    let f = fixture("claude", true);
    let (goal, tasks) = accept(&f, 2).await;
    let scope = tasks[0].scope();
    record(
        &f,
        &scope,
        RecordKind::Review,
        serde_json::json!({"secret": "EVENT-RECORD-SENTINEL"}),
    );
    {
        let shared = f.store();
        let mut store = shared.lock().unwrap();
        store
            .audit(
                &scope,
                "custom.ok",
                serde_json::json!({"secret": "EVENT-DATA-SENTINEL"}),
            )
            .unwrap();
        store
            .audit(
                &scope,
                &format!("{}\u{7}", "long".repeat(80)),
                serde_json::json!({"secret": "EVENT-DATA-SENTINEL"}),
            )
            .unwrap();
    }
    let (other, _, _) = other_project(&f, 1).await;
    let all = |scope: Scope| {
        let f = &f;
        async move {
            let mut after = None;
            let mut events = Vec::new();
            loop {
                let response = read(
                    f,
                    ControlAction::Events {
                        scope: scope.clone(),
                        after,
                        maximum: 3,
                    },
                )
                .await;
                let encoded = text(&response);
                for absent in ["SENTINEL", "custom.ok", "rrx.private", ".saved", "longlong"] {
                    assert!(!encoded.contains(absent), "event leaked: {absent}");
                }
                let ControlResponse::EventPage {
                    events: page, next, ..
                } = response
                else {
                    panic!("EventPage")
                };
                events.extend(page);
                match next {
                    Some(next) => after = Some(next),
                    None => return events,
                }
            }
        }
    };
    let sequences = |filter: &str, id: String| -> Vec<i64> {
        let c = raw(&f);
        let mut s = c
            .prepare(&format!(
                "SELECT sequence FROM audit WHERE {filter}=?1 ORDER BY sequence"
            ))
            .unwrap();
        s.query_map([id], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let project_events = all(Scope::project(f.project.id)).await;
    assert_eq!(
        project_events
            .iter()
            .map(|e| e.sequence)
            .collect::<Vec<_>>(),
        sequences("project_id", f.project.id.to_string()),
        "Project scope keeps descendant semantics"
    );
    assert!(
        project_events
            .iter()
            .any(|e| e.kind == EventKindView::GoalAccepted)
    );
    assert!(
        project_events
            .iter()
            .any(|e| e.kind == EventKindView::ProjectSaved)
    );
    assert_eq!(
        all(Scope::goal(f.project.id, goal))
            .await
            .iter()
            .map(|e| e.sequence)
            .collect::<Vec<_>>(),
        sequences("goal_id", goal.to_string())
    );
    let task_events = all(scope.clone()).await;
    assert_eq!(
        task_events.iter().map(|e| e.sequence).collect::<Vec<_>>(),
        sequences("task_id", tasks[0].id.to_string())
    );
    assert!(task_events.iter().all(|e| e.scope == scope));
    assert!(
        task_events
            .iter()
            .any(|e| e.kind == EventKindView::ReviewSaved)
    );
    assert_eq!(
        task_events
            .iter()
            .filter(|e| e.kind == EventKindView::EventUnavailable)
            .count(),
        2,
        "both custom kinds are unavailable"
    );
    // A sequence outside the selected scope is not a cursor of it.
    let foreign = sequences("project_id", other.id.to_string())[0];
    let response = read(
        &f,
        ControlAction::Events {
            scope,
            after: Some(foreign),
            maximum: 8,
        },
    )
    .await;
    assert_eq!(
        unavailable(&response),
        Some(UnavailableReason::ReadCursorInvalid)
    );
    finish(f).await;
}

/// C-S4f: qualified-looking Usage still yields only `MetricsUnqualified`;
/// Routing only `RoutingUnavailable`; nothing stored appears.
#[tokio::test]
async fn c_s4f_metrics_and_routing_are_typed_unavailable() {
    let f = fixture("claude", true);
    let (_, tasks) = accept(&f, 1).await;
    let scope = tasks[0].scope();
    let session = session(&f, &scope, SessionRole::Reviewer, "USAGE");
    f.store()
        .lock()
        .unwrap()
        .put_usage(&Usage {
            scope: scope.clone(),
            session_id: session.id,
            agent: "worker".into(),
            phase: "PHASE-SENTINEL".into(),
            review_round: Some(1),
            input_tokens: Some(1111),
            cached_input_tokens: Some(2222),
            output_tokens: Some(3333),
            estimated_cost: Some(4.5),
            context_pack_version: Some(1),
            context_pack_size: Some(10),
            repo_map_size: Some(10),
            cache_metadata: serde_json::json!({"cache": "CACHE-SENTINEL"}),
            missing_reason: None,
        })
        .unwrap();
    for (action, reason) in [
        (
            ControlAction::Metrics {
                scope: scope.clone(),
            },
            UnavailableReason::MetricsUnqualified,
        ),
        (
            ControlAction::Routing {
                scope: scope.clone(),
            },
            UnavailableReason::RoutingUnavailable,
        ),
        (
            ControlAction::Metrics {
                scope: Scope::project(f.project.id),
            },
            UnavailableReason::MetricsUnqualified,
        ),
    ] {
        let response = read(&f, action).await;
        let encoded = text(&response);
        for absent in ["SENTINEL", "1111", "2222", "3333", "NATIVE-"] {
            assert!(!encoded.contains(absent), "stored value leaked: {absent}");
        }
        assert_eq!(unavailable(&response), Some(reason));
    }
    let mut unknown = scope;
    unknown.task_id = Some(crate::domain::TaskId::new());
    assert!(
        answer(&f, ControlAction::Metrics { scope: unknown })
            .await
            .is_err()
    );
    finish(f).await;
}

const NO_WRITE_TABLES: &[&str] = &[
    "runtime_epoch",
    "projects",
    "goals",
    "tasks",
    "records",
    "usage",
    "audit",
    "context_versions",
    "goal_authority",
    "goal_observations",
    "runtime_control_acks",
    "scheduler_clock",
    "scheduler_projects",
    "scheduler_goals",
    "scheduler_tasks",
    "task_drivers",
];
fn fingerprint(f: &ControlFixture) -> Vec<String> {
    let c = raw(f);
    NO_WRITE_TABLES
        .iter()
        .map(|table| {
            let mut s = c.prepare(&format!("SELECT * FROM {table}")).unwrap();
            let columns = s.column_count();
            let mut rows = s
                .query_map([], |r| {
                    Ok((0..columns)
                        .map(|i| format!("{:?}", r.get_ref(i).unwrap()))
                        .collect::<Vec<_>>()
                        .join("|"))
                })
                .unwrap()
                .map(Result::unwrap)
                .collect::<Vec<_>>();
            rows.sort();
            format!("{table}\n{}", rows.join("\n"))
        })
        .collect()
}

/// C-S4g and C-S4h: every S4 action, on success and on refusal, leaves
/// owner/epoch and every Runtime table unchanged with the Runtime idle;
/// `maximum` 0 and 129 refuse; an over-long title is null, not a refusal.
#[tokio::test]
async fn c_s4g_every_read_writes_nothing_and_bounds_hold() {
    let f = fixture("claude", true);
    let mut long = plan();
    long.definition.title = "T".repeat(GOAL_TITLE_BYTES + 1);
    let long_goal = f.create(long).await;
    let mut exact = plan();
    exact.definition.title = "E".repeat(GOAL_TITLE_BYTES);
    let exact_goal = f.create(exact).await;
    let (goal, tasks) = accept(&f, 2).await;
    let scope = tasks[0].scope();
    record(&f, &scope, RecordKind::Review, serde_json::json!({}));
    session(&f, &scope, SessionRole::Reviewer, "IDLE");
    // No attention sweep has run, so a read that ran one would write.
    let project = f.project.id;
    let paged = |maximum| {
        vec![
            ControlAction::ProjectGoals {
                project,
                after: None,
                maximum,
            },
            ControlAction::TaskReview {
                scope: scope.clone(),
                after: None,
                maximum,
            },
            ControlAction::TaskSessions {
                scope: scope.clone(),
                after: None,
                maximum,
            },
            ControlAction::AttentionQueue {
                project: None,
                after: None,
                maximum,
            },
            ControlAction::Events {
                scope: Scope::goal(project, goal),
                after: None,
                maximum,
            },
        ]
    };
    let mut actions = paged(128);
    actions.extend([
        ControlAction::Routing {
            scope: scope.clone(),
        },
        ControlAction::Metrics {
            scope: scope.clone(),
        },
        ControlAction::ProjectGoals {
            project,
            after: Some(GoalId::new()),
            maximum: 4,
        },
        ControlAction::AttentionQueue {
            project: Some(project),
            after: Some(AttentionCursor {
                project,
                goal: GoalId::new(),
                lane: AttentionLane::Goal,
                task: None,
            }),
            maximum: 4,
        },
    ]);
    let refusals = paged(0).into_iter().chain(paged(129)).chain([
        ControlAction::TaskSessions {
            scope: Scope::task(project, GoalId::new(), tasks[0].id),
            after: None,
            maximum: 4,
        },
        ControlAction::Events {
            scope: Scope::project(crate::domain::ProjectId::new()),
            after: None,
            maximum: 4,
        },
    ]);
    let before = fingerprint(&f);
    let epoch = f.owner.store().lock().unwrap().connection_epoch_for_test();
    for action in actions {
        let label = format!("{action:?}");
        let response = answer(&f, action).await.unwrap();
        text(&response);
        assert_eq!(fingerprint(&f), before, "read wrote: {label}");
    }
    for action in refusals {
        let label = format!("{action:?}");
        assert!(answer(&f, action).await.is_err(), "not refused: {label}");
        assert_eq!(fingerprint(&f), before, "refusal wrote: {label}");
    }
    assert_eq!(
        f.owner.store().lock().unwrap().connection_epoch_for_test(),
        epoch
    );
    // C-S4h: titles over the bound are null and the page stands.
    let goals = goal_pages(&f, project).await;
    let title = |id| goals.iter().find(|g| g.id == id).unwrap().title.clone();
    assert_eq!(title(long_goal), None);
    assert_eq!(title(exact_goal), Some("E".repeat(GOAL_TITLE_BYTES)));
    finish(f).await;
}
