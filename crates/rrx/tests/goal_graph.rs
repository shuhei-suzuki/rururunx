use rrx::{domain::*, state::Store};
use serde_json::to_value;
use uuid::Uuid;

fn id(value: u128) -> TaskId {
    TaskId(Uuid::from_u128(value))
}

fn edge(prerequisite: TaskId, dependent: TaskId, hard: bool) -> Dependency {
    Dependency {
        prerequisite,
        dependent,
        hard,
    }
}

#[test]
fn hard_order_retains_independent_nodes_and_ignores_advisory_cycles() {
    let [a, b, c, d, independent] = [1, 2, 3, 4, 5].map(id);
    let mut dag = TaskDag {
        nodes: vec![independent, d, c, b, a],
        edges: vec![
            edge(a, c, true),
            edge(c, d, true),
            edge(a, b, true),
            edge(b, d, true),
        ],
    };
    assert_eq!(dag.hard_order().unwrap(), [a, b, c, d, independent]);
    // A hard path and an opposite advisory edge do not create a hard cycle.
    dag.edges
        .extend([edge(d, a, false), edge(b, c, false), edge(c, b, false)]);
    assert_eq!(dag.hard_order().unwrap(), [a, b, c, d, independent]);
    dag.nodes.reverse();
    dag.edges.reverse();
    assert_eq!(dag.hard_order().unwrap(), [a, b, c, d, independent]);
    dag.edges
        .iter_mut()
        .find(|e| e.prerequisite == d && e.dependent == a)
        .unwrap()
        .hard = true;
    assert!(
        dag.hard_order()
            .unwrap_err()
            .to_string()
            .contains("hard dependency cycle")
    );
    assert!(TaskDag::default().hard_order().unwrap().is_empty());
}

#[test]
fn graph_bounds_have_valid_boundary_controls() {
    let mut dag = TaskDag {
        nodes: (1..=4096).map(id).collect(),
        edges: (1..4096).map(|n| edge(id(n), id(n + 1), true)).collect(),
    };
    assert_eq!(dag.hard_order().unwrap().len(), 4096);
    dag.nodes.push(id(4097));
    assert!(
        dag.hard_order()
            .unwrap_err()
            .to_string()
            .contains("node limit")
    );

    // Unique, declared, non-self advisory pairs: removing the edge cap reaches
    // a successful traversal rather than a duplicate/endpoint/cycle refusal.
    let pairs: Vec<_> = (1..=256)
        .flat_map(|a| {
            (1..=256)
                .filter(move |&b| b != a)
                .map(move |b| edge(id(a), id(b), false))
        })
        .take(16385)
        .collect();
    dag = TaskDag {
        nodes: (1..=256).map(id).collect(),
        edges: pairs[..16384].to_vec(),
    };
    assert_eq!(dag.hard_order().unwrap().len(), 256);
    dag.edges.push(pairs[16384].clone());
    assert!(
        dag.hard_order()
            .unwrap_err()
            .to_string()
            .contains("edge limit")
    );
}

#[test]
fn store_rejects_invalid_graph_changes_without_rows_versions_or_audit() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("state.db");
    let mut store = Store::open(&path).unwrap();
    let mut project = Project::new(
        "graph".into(),
        temporary.path().to_owned(),
        "repo:graph".into(),
        "main".into(),
    );
    store.put_project(&mut project).unwrap();
    let mut goal = Goal::new(
        project.id,
        "all required work".into(),
        vec![CompletionCriterion {
            id: "work".into(),
            description: "verified work".into(),
            evidence: None,
            satisfied: false,
        }],
    );
    store.put_goal(&mut goal).unwrap();
    let mut tasks: Vec<_> = (1..=5)
        .map(|n| {
            let mut task = Task::new(project.id, goal.id, format!("task {n}"), "fake".into());
            task.id = id(n);
            store.put_task(&mut task).unwrap();
            task
        })
        .collect();
    let [a, b, c, d, independent] = [1, 2, 3, 4, 5].map(id);
    goal.dag = TaskDag {
        nodes: vec![independent, d, c, b, a],
        edges: vec![
            edge(a, b, true),
            edge(a, c, true),
            edge(b, d, true),
            edge(c, d, true),
        ],
    };
    store.put_goal(&mut goal).unwrap();
    let baseline = to_value(&goal).unwrap();
    let audits = store.events(&goal.scope(), 0, 100).unwrap().len();
    for (change, expected_error) in [
        (edge(d, a, true), "hard dependency cycle"),
        (edge(a, b, false), "duplicate goal DAG edge pair"),
        (edge(a, b, true), "duplicate goal DAG edge pair"),
        (edge(a, a, true), "self edge"),
        (edge(a, a, false), "self edge"),
        (edge(a, id(999), true), "declared nodes"),
        (edge(id(999), a, true), "declared nodes"),
        (edge(id(999), a, false), "declared nodes"),
        (edge(a, id(999), false), "declared nodes"),
    ] {
        let mut rejected = goal.clone();
        rejected.dag.edges.push(change);
        let input = to_value(&rejected).unwrap();
        let error = store.put_goal(&mut rejected).unwrap_err().to_string();
        assert!(
            error.contains(expected_error),
            "unexpected refusal: {error}"
        );
        assert_eq!(to_value(&rejected).unwrap(), input);
        assert_eq!(
            to_value(store.goal(goal.id).unwrap().unwrap()).unwrap(),
            baseline
        );
        assert_eq!(store.events(&goal.scope(), 0, 100).unwrap().len(), audits);
    }
    let mut duplicate = goal.clone();
    duplicate.dag.nodes.push(a);
    assert!(
        store
            .put_goal(&mut duplicate)
            .unwrap_err()
            .to_string()
            .contains("duplicate goal DAG nodes")
    );
    assert_eq!(
        to_value(store.goal(goal.id).unwrap().unwrap()).unwrap(),
        baseline
    );
    assert_eq!(store.events(&goal.scope(), 0, 100).unwrap().len(), audits);

    // The same graph update is accepted when the back edge is advisory.
    goal.dag.edges.push(edge(d, a, false));
    store.put_goal(&mut goal).unwrap();
    assert_eq!(goal.version, baseline["version"].as_u64().unwrap() + 1);
    assert_eq!(
        store.events(&goal.scope(), 0, 100).unwrap().len(),
        audits + 1
    );
    drop(store);
    let store = Store::open(&path).unwrap();
    let restored = store.goal(goal.id).unwrap().unwrap();
    assert_eq!(to_value(&restored).unwrap(), to_value(&goal).unwrap());
    assert_eq!(
        restored.dag.hard_order().unwrap(),
        [a, b, c, d, independent]
    );
    // Task persistence is untouched by structural validation.
    for task in tasks.drain(..) {
        assert_eq!(
            to_value(store.task(task.id).unwrap().unwrap()).unwrap(),
            to_value(task).unwrap()
        );
    }
}

#[test]
fn legacy_invalid_graph_hold_and_terminal_refusals_preserve_original_history() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("state.db");
    let mut store = Store::open(&path).unwrap();
    let mut project = Project::new(
        "legacy graph".into(),
        temporary.path().to_owned(),
        "repo:legacy-graph".into(),
        "main".into(),
    );
    store.put_project(&mut project).unwrap();
    let mut goal = Goal::new(
        project.id,
        "preserve legacy history".into(),
        vec![CompletionCriterion {
            id: "legacy".into(),
            description: "requires future typed reconciliation".into(),
            evidence: None,
            satisfied: false,
        }],
    );
    store.put_goal(&mut goal).unwrap();
    let mut tasks = Vec::new();
    for name in ["a", "b"] {
        let mut task = Task::new(project.id, goal.id, name.into(), "fake".into());
        store.put_task(&mut task).unwrap();
        tasks.push(task);
    }
    let [a, b] = [tasks[0].id, tasks[1].id];
    goal.dag = TaskDag {
        nodes: vec![a, b],
        edges: vec![edge(a, b, true)],
    };
    store.put_goal(&mut goal).unwrap();
    let connection = rusqlite::Connection::open(&path).unwrap();
    for (legacy_edge, expected_error) in [
        (edge(b, a, true), "hard dependency cycle"),
        (edge(a, a, false), "self edge"),
        (edge(a, b, false), "duplicate goal DAG edge pair"),
    ] {
        // Seed bytes accepted by the old node/endpoint validator. This neither
        // ratifies the graph nor constructs managed Goal authority.
        let mut legacy = goal.clone();
        legacy.dag.edges.push(legacy_edge);
        let original = serde_json::to_string(&legacy).unwrap();
        connection
            .execute(
                "UPDATE goals SET body=?1 WHERE id=?2",
                rusqlite::params![original, goal.id.to_string()],
            )
            .unwrap();
        for project_state in [ProjectState::Registered, ProjectState::Blocked] {
            project.state = project_state;
            project.blocked_reason =
                (project_state == ProjectState::Blocked).then(|| "synthetic project hold".into());
            store.put_project(&mut project).unwrap();
            let events = store.events(&Scope::project(project.id), 0, 100).unwrap();
            assert!(events.len() < 100);
            let audits = to_value(events).unwrap();
            for state in [
                GoalState::Paused,
                GoalState::Blocked,
                GoalState::WaitingHuman,
                GoalState::Cancelled,
                GoalState::Failed,
            ] {
                let mut requested = legacy.clone();
                requested.state = state;
                requested.blockers = vec!["synthetic conservative decision".into()];
                let input = to_value(&requested).unwrap();
                let error = store.put_goal(&mut requested).unwrap_err().to_string();
                assert!(
                    error.contains(expected_error),
                    "unexpected refusal: {error}"
                );
                assert_eq!(to_value(requested).unwrap(), input);
                let (version, body): (u64, String) = connection
                    .query_row(
                        "SELECT version,body FROM goals WHERE id=?1",
                        [goal.id.to_string()],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .unwrap();
                assert_eq!(version, legacy.version);
                assert_eq!(body, original);
                assert_eq!(
                    to_value(store.events(&Scope::project(project.id), 0, 100).unwrap()).unwrap(),
                    audits
                );
                for task in &tasks {
                    assert_eq!(
                        to_value(store.task(task.id).unwrap().unwrap()).unwrap(),
                        to_value(task).unwrap()
                    );
                }
            }
        }
    }
}
