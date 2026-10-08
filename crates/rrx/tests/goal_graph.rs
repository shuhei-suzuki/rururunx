use rrx::{domain::*, state::Store};
use serde_json::to_value;
use uuid::Uuid;
#[path = "support/current_writer.rs"]
mod current_writer;
#[path = "support/legacy.rs"]
mod legacy;

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

/// Legacy rows for `planned` Tasks on a fresh Project, through the public
/// route (FM §8.1). Returns the open Store, the Goal and its Tasks.
fn legacy_goal(
    path: &std::path::Path,
    root: &std::path::Path,
    name: &str,
    objective: &'static str,
    keys: &[&'static str],
) -> (Store, Goal, Vec<Task>) {
    let mut project = Project::new(
        name.into(),
        root.to_owned(),
        format!("repo:{name}"),
        "main".into(),
    );
    Store::open(path)
        .unwrap()
        .put_project(&mut project)
        .unwrap();
    let (goal, tasks) = legacy::goals(
        path,
        vec![(
            project.id,
            objective,
            keys.iter()
                .map(|key| {
                    (
                        *key,
                        "fake",
                        rrx::config::WorkflowClass::Standard,
                        RiskClass::R1,
                        "verified work",
                    )
                })
                .collect(),
        )],
    )
    .remove(0);
    (Store::open(path).unwrap(), goal, tasks)
}

fn refused(store: &mut Store, requested: &mut Goal) {
    let input = to_value(&*requested).unwrap();
    let error = store.put_goal(requested).unwrap_err().to_string();
    assert!(
        error.contains("Goal changes require trusted typed control ingress"),
        "unexpected refusal: {error}"
    );
    assert_eq!(to_value(&*requested).unwrap(), input);
}

/// FM §8.3 S4-W. Every Goal change is refused by the generic Goal writer on
/// any row before graph validation. The former proof — the writer's DAG
/// validation (hard cycle, duplicate pair, self edge, undeclared endpoint,
/// duplicate node) and its acceptance of an advisory back edge — has no
/// legitimate producer now (Goal changes require typed control ingress), so
/// it is LOST and recorded here. `TaskDag::hard_order` itself is still proved
/// by the two graph tests above. Each attempt asserts the typed refusal, an
/// unchanged Goal row and audit, and untouched Tasks across a restart.
#[test]
fn store_rejects_invalid_graph_changes_without_rows_versions_or_audit() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("state.db");
    let (mut store, goal, tasks) = legacy_goal(
        &path,
        temporary.path(),
        "graph",
        "all required work",
        &["a", "b", "c", "d", "independent"],
    );
    let baseline = to_value(store.goal(goal.id).unwrap().unwrap()).unwrap();
    let audits = store.events(&goal.scope(), 0, 100).unwrap().len();
    let [a, b, c, d, independent] = [0, 1, 2, 3, 4].map(|n| tasks[n].id);
    let mut proposed = goal.clone();
    proposed.dag = TaskDag {
        nodes: vec![independent, d, c, b, a],
        edges: vec![
            edge(a, b, true),
            edge(a, c, true),
            edge(b, d, true),
            edge(c, d, true),
        ],
    };
    for change in [
        None,
        Some(edge(d, a, true)),
        Some(edge(a, b, false)),
        Some(edge(a, a, true)),
        Some(edge(a, id(999), true)),
        Some(edge(id(999), a, false)),
        // Formerly accepted: the same update with an advisory back edge.
        Some(edge(d, a, false)),
    ] {
        let mut requested = proposed.clone();
        requested.dag.edges.extend(change);
        refused(&mut store, &mut requested);
        assert_eq!(
            to_value(store.goal(goal.id).unwrap().unwrap()).unwrap(),
            baseline
        );
        assert_eq!(store.events(&goal.scope(), 0, 100).unwrap().len(), audits);
    }
    let mut duplicate = proposed.clone();
    duplicate.dag.nodes.push(a);
    refused(&mut store, &mut duplicate);
    drop(store);
    let store = Store::open(&path).unwrap();
    assert_eq!(
        to_value(store.goal(goal.id).unwrap().unwrap()).unwrap(),
        baseline
    );
    // Task persistence is untouched.
    for task in tasks {
        assert_eq!(
            to_value(store.task(task.id).unwrap().unwrap()).unwrap(),
            to_value(task).unwrap()
        );
    }
}

/// FM §8.3 S4-W. The former proof — hold and terminal requests refused on a
/// historically invalid graph seeded into the Goal row by SQL — is LOST and
/// recorded here: FM adds no SQL-seeded path, and a legacy Goal cannot be
/// changed through the generic writer at all. What remains is the generic
/// writer refusal for every hold/terminal request, under a registered and a
/// blocked Project, with the Goal row, audit and Tasks unchanged.
#[test]
fn legacy_invalid_graph_hold_and_terminal_refusals_preserve_original_history() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("state.db");
    let (mut store, goal, tasks) = legacy_goal(
        &path,
        temporary.path(),
        "legacy graph",
        "preserve legacy history",
        &["a", "b"],
    );
    let mut project = store.project(goal.project_id).unwrap().unwrap();
    let connection = current_writer::open(&path).unwrap();
    let original: (u64, String) = connection
        .query_row(
            "SELECT version,body FROM goals WHERE id=?1",
            [goal.id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
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
            let mut requested = goal.clone();
            requested.state = state;
            requested.blockers = vec!["synthetic conservative decision".into()];
            refused(&mut store, &mut requested);
            let current: (u64, String) = connection
                .query_row(
                    "SELECT version,body FROM goals WHERE id=?1",
                    [goal.id.to_string()],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();
            assert_eq!(current, original);
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
