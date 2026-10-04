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
