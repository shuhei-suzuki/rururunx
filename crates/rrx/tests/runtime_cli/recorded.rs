use super::*;
use rrx::runtime::{
    control::{GoalControl, GoalReadView},
    goal::PlanDependency,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

fn plan(count: usize) -> GoalPlan {
    GoalPlan {
        definition: GoalDefinition {
            title: "Recorded facts".into(),
            objective: "Observe accepted work".into(),
            criteria: vec![
                CriterionDefinition {
                    id: "id\u{7f}\u{9b}".into(),
                    description: "description\u{9b}\u{7f}é".into(),
                    evaluator: CriterionEvaluator::RequiredTasksVerified,
                },
                CriterionDefinition {
                    id: "human".into(),
                    description: "A declared human criterion".into(),
                    evaluator: CriterionEvaluator::Human {
                        goal_pack_input: true,
                    },
                },
            ],
            constraints: vec![],
            non_goals: vec![],
            source_refs: vec![],
        },
        tasks: (0..count)
            .map(|i| TaskDefinition {
                key: format!("t{i}"),
                title: format!("Task {i}"),
                acceptance_criteria: vec!["Verified work".into()],
                executor: "worker".into(),
                reviewers: vec![],
                workflow: WorkflowClass::Quick,
                risk: RiskClass::R2,
            })
            .collect(),
        dependencies: vec![],
    }
}
impl Fixture {
    async fn accept_plan(&self, project: ProjectId, plan: GoalPlan) -> GoalId {
        let ControlResponse::ProjectResolved { version, .. } = client::request(
            &self.state,
            ControlAction::ResolveProject {
                selector: Some(project.to_string()),
                cwd: self.dir.path().into(),
            },
        )
        .await
        .unwrap() else {
            panic!("Project routing");
        };
        let action = ControlAction::CreateGoal {
            project,
            expected_project: version,
            plan,
        };
        assert!(
            serde_json::to_vec(&action).unwrap().len() + 1024 < transport::REQUEST_BYTES,
            "positive plan must fit genuine transport request"
        );
        match client::request(&self.state, action).await.unwrap() {
            ControlResponse::GoalAccepted { goal, .. } => goal,
            other => panic!("genuine accepted plan required: {other:?}"),
        }
    }
    async fn idle(&self) -> Vec<(String, Vec<Vec<String>>)> {
        tokio::time::sleep(Duration::from_millis(300)).await;
        let before = rows(&self.state);
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(
            rows(&self.state),
            before,
            "idle baseline must be stable across every table"
        );
        before
    }
    fn plain(&self, args: &[&str]) -> String {
        let out = self.command().args(args).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }
    /// One compiled plain Task page compared byte-for-byte with text built from the
    /// genuine stored Goal/Task rows. The paired JSON page supplies only its budget
    /// decisions (returned length, incoming availability) and the effective state.
    fn plain_page(
        &self,
        project: ProjectId,
        goal: GoalId,
        after: Option<&str>,
        maximum: u16,
    ) -> (String, Vec<String>, Option<String>) {
        let (project_text, goal_text, maximum) =
            (project.to_string(), goal.to_string(), maximum.to_string());
        let mut args = vec![
            "goal",
            "tasks",
            goal_text.as_str(),
            "--project",
            project_text.as_str(),
            "--maximum",
            maximum.as_str(),
        ];
        if let Some(after) = after {
            args.extend(["--after", after]);
        }
        let plain = self.plain(&args);
        args.push("--json");
        let json = self.cli_json(&args);
        let goals = stored(&self.state, "goals");
        let raw_goal = &goals[&goal_text];
        let raw = stored(&self.state, "tasks");
        let inventory = raw
            .iter()
            .filter(|(_, task)| task["goal_id"] == goal_text.as_str())
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>();
        let start = after.map_or(0, |after| {
            inventory.iter().position(|id| *id == after).unwrap() + 1
        });
        let tasks = json["facts"]["tasks"].as_array().unwrap();
        let nodes = json["facts"]["recorded"]["nodes"].as_array().unwrap();
        assert!(!tasks.is_empty() && tasks.len() == nodes.len());
        let returned = &inventory[start..start + tasks.len()];
        let next = (start + tasks.len() < inventory.len()).then(|| returned[returned.len() - 1]);
        let edges = raw_goal["dag"]["edges"].as_array().unwrap();
        let hard_edges = edges.iter().filter(|e| e["hard"] == true).count();
        let mut expected = format!(
            "Project: {project}\nGoal: {goal} version {}\nDAG: {} nodes, {hard_edges} hard edges, {} soft edges\n",
            raw_goal["version"],
            inventory.len(),
            edges.len() - hard_edges
        );
        let mut overflow = false;
        for ((id, fact), node) in returned.iter().zip(tasks).zip(nodes) {
            let task = &raw[*id];
            assert_eq!(fact["scope"]["task_id"], *id);
            assert!(
                task["phase"].is_null() && fact["phase"].is_null(),
                "only the genuine null phase baseline has a producer"
            );
            let mut incoming = edges
                .iter()
                .filter(|e| e["dependent"] == *id)
                .collect::<Vec<_>>();
            incoming.sort_by(|a, b| a["prerequisite"].as_str().cmp(&b["prerequisite"].as_str()));
            let hard = incoming.iter().filter(|e| e["hard"] == true).count();
            writeln!(
                expected,
                "Task: {id} version {} stored state {} effective state {} phase null",
                task["version"], task["state"], fact["state"]
            )
            .unwrap();
            writeln!(
                expected,
                "Structural dependencies: \"{}\"; incoming: {} (hard: {hard})",
                if hard == 0 {
                    "unconstrained"
                } else {
                    "requires_prerequisite_evidence"
                },
                incoming.len()
            )
            .unwrap();
            if node["incoming"]["availability"] == "unavailable" {
                overflow = true;
                expected.push_str("  Incoming detail: unavailable (projection_budget)\n");
            } else {
                for edge in incoming {
                    let prerequisite = edge["prerequisite"].as_str().unwrap();
                    writeln!(
                        expected,
                        "  {} prerequisite: {prerequisite} version {} stored state {}",
                        if edge["hard"] == true { "hard" } else { "soft" },
                        raw[prerequisite]["version"],
                        raw[prerequisite]["state"]
                    )
                    .unwrap();
                }
            }
        }
        let mut incomplete = vec![
            "native_dispatch",
            "unit_provider_evidence",
            "wait_cleanup_details",
            "verified_criterion_completion",
            "runnable_admission",
        ];
        if overflow {
            incomplete.push("task_incoming_relationships");
        }
        assert_eq!(json["complete"], false);
        assert_eq!(json["unavailable_fields"], serde_json::json!(incomplete));
        writeln!(
            expected,
            "Next: {}\nverified criterion completion: unavailable\nrunnable admission: unknown\nIndependent scoped observation; incomplete: {}\n",
            next.unwrap_or("null"),
            incomplete.join(", ")
        )
        .unwrap();
        assert_eq!(plain, expected, "plain Task page diverged from stored rows");
        (
            plain,
            returned.iter().map(|id| id.to_string()).collect(),
            next.map(str::to_owned),
        )
    }
}
fn stored(state: &Path, table: &str) -> BTreeMap<String, Value> {
    assert!(table == "goals" || table == "tasks");
    let db =
        rusqlite::Connection::open_with_flags(state, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    db.prepare(&format!("SELECT id,body FROM {table}"))
        .unwrap()
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                serde_json::from_str::<Value>(&r.get::<_, String>(1)?).unwrap(),
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}
fn keys(value: &Value, expected: &[&str]) {
    assert_eq!(
        value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        expected.iter().copied().collect::<BTreeSet<_>>()
    );
}
fn no_terminal_controls(text: &str) {
    assert!(
        !text
            .chars()
            .any(|c| c == '\u{7f}' || ('\u{80}'..='\u{9f}').contains(&c)),
        "plain sink emitted DEL/C1"
    );
    assert!(text.contains("\\u007f") && text.contains("\\u009b"));
}

#[tokio::test]
async fn recorded_accepted_criteria_dag_scope_and_rows_are_exact() {
    let mut f = Fixture::new();
    let project = f.project("one");
    let foreign = f.project("two");
    f.start().await;
    let mut p = plan(4);
    p.dependencies = vec![
        PlanDependency {
            prerequisite: "t0".into(),
            dependent: "t2".into(),
            hard: true,
        },
        PlanDependency {
            prerequisite: "t1".into(),
            dependent: "t2".into(),
            hard: false,
        },
    ];
    let goal = f.accept_plan(project, p).await;
    let foreign_goal = f.create(foreign, 1).await;
    let before = f.idle().await;
    let raw_goals = stored(&f.state, "goals");
    let raw_goal = &raw_goals[&goal.to_string()];
    let raw_tasks = stored(&f.state, "tasks");
    let args = [
        "goal",
        "status",
        &goal.to_string(),
        "--project",
        "one",
        "--json",
    ];
    let status = f.cli_json(&args);
    let r = &status["facts"]["recorded"];
    keys(
        r,
        &[
            "view",
            "project",
            "accepted",
            "criteria",
            "dag",
            "criterion_evaluation",
            "runnable_admission",
        ],
    );
    assert_eq!(r["view"], "recorded_v1");
    assert_eq!(r["accepted"], true);
    assert_eq!(r["project"], project.to_string());
    assert_eq!(status["complete"], false);
    assert_eq!(
        r["dag"],
        serde_json::json!({"node_count":4,"hard_edge_count":1,"soft_edge_count":1})
    );
    assert_eq!(r["criterion_evaluation"], "unavailable");
    assert_eq!(r["runnable_admission"], "unknown");
    assert_eq!(status["facts"]["version"], raw_goal["version"]);
    let criteria = r["criteria"]["items"].as_array().unwrap();
    assert_eq!(r["criteria"]["recorded_count"], criteria.len());
    for (got, actual) in criteria
        .iter()
        .zip(raw_goal["completion_criteria"].as_array().unwrap())
    {
        keys(
            got,
            &[
                "id",
                "description",
                "evaluator",
                "recorded_satisfied",
                "recorded_evidence",
            ],
        );
        assert_eq!(got["id"], actual["id"]);
        assert_eq!(got["description"], actual["description"]);
        assert_eq!(got["evaluator"], actual["evaluator"]);
        assert_eq!(got["recorded_satisfied"], actual["satisfied"]);
        assert_eq!(got["recorded_evidence"], actual["evidence"]);
    }
    let plain = f.plain(&args[..args.len() - 1]);
    no_terminal_controls(&plain);
    assert!(
        plain.contains("recorded satisfied: false")
            && plain.contains("verified criterion completion: unavailable")
    );
    let page = f.cli_json(&[
        "goal",
        "tasks",
        &goal.to_string(),
        "--project",
        "one",
        "--json",
    ]);
    let nodes = page["facts"]["recorded"]["nodes"].as_array().unwrap();
    let tasks = page["facts"]["tasks"].as_array().unwrap();
    assert_eq!(nodes.len(), 4);
    assert_eq!(nodes.len(), tasks.len());
    for (node, fact) in nodes.iter().zip(tasks) {
        keys(
            node,
            &[
                "task",
                "version",
                "stored_state",
                "incoming_count",
                "hard_incoming_count",
                "structural_dependencies",
                "incoming",
            ],
        );
        let id = node["task"].as_str().unwrap();
        let actual = &raw_tasks[id];
        assert_eq!(node["version"], actual["version"]);
        assert_eq!(node["stored_state"], actual["state"]);
        assert_eq!(fact["scope"]["task_id"], node["task"]);
        assert_eq!(fact["version"], node["version"]);
        assert_eq!(
            fact["state"], actual["state"],
            "no genuine wait-state producer in this fixture"
        );
        let edges = raw_goal["dag"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["dependent"] == node["task"])
            .collect::<Vec<_>>();
        let incoming = node["incoming"]["items"].as_array().unwrap();
        assert_eq!(incoming.len(), edges.len());
        assert_eq!(node["incoming_count"], edges.len());
        let hard = edges.iter().filter(|e| e["hard"] == true).count();
        assert_eq!(node["hard_incoming_count"], hard);
        assert_eq!(
            node["structural_dependencies"],
            if hard == 0 {
                "unconstrained"
            } else {
                "requires_prerequisite_evidence"
            }
        );
        assert!(
            incoming
                .windows(2)
                .all(|pair| pair[0]["task"].as_str() < pair[1]["task"].as_str())
        );
        for endpoint in incoming {
            keys(endpoint, &["task", "version", "stored_state", "hard"]);
            let actual = &raw_tasks[endpoint["task"].as_str().unwrap()];
            assert_eq!(endpoint["version"], actual["version"]);
            assert_eq!(endpoint["stored_state"], actual["state"]);
            assert!(edges.iter().any(|e|e["prerequisite"]==endpoint["task"]&&e["hard"]==endpoint["hard"]));
        }
    }
    let (full, ids, next) = f.plain_page(project, goal, None, 64);
    assert_eq!((ids.len(), next), (4, None));
    for label in [
        "  hard prerequisite: ",
        "  soft prerequisite: ",
        "Structural dependencies: \"unconstrained\"; incoming: 0 (hard: 0)\n",
        "Structural dependencies: \"requires_prerequisite_evidence\"; incoming: 2 (hard: 1)\n",
        "\nNext: null\n",
    ] {
        assert!(full.contains(label), "genuine plain page lacks {label:?}");
    }
    let (first, mut paged, cursor) = f.plain_page(project, goal, None, 3);
    let cursor = cursor.unwrap();
    assert_eq!(paged.len(), 3);
    assert_eq!(cursor, paged[2]);
    assert!(first.contains(&format!("\nNext: {cursor}\n")));
    let (_, rest, end) = f.plain_page(project, goal, Some(&cursor), 3);
    assert_eq!((rest.len(), end), (1, None));
    paged.extend(rest);
    assert_eq!(
        paged, ids,
        "plain cursor pages omitted or duplicated a Task"
    );
    eprintln!(
        "UNVERIFIED: nondefault Task phase and effective/stored state divergence lack a genuine producer; plain labels are pinned only at the null/equal baseline"
    );
    let foreign_page = client::request(
        &f.state,
        ControlAction::GoalTasks {
            project: foreign,
            goal: foreign_goal,
            after: None,
            maximum: 1,
            view: None,
        },
    )
    .await
    .unwrap();
    let ControlResponse::GoalTaskPage { tasks, .. } = foreign_page else {
        panic!()
    };
    assert!(
        client::request(
            &f.state,
            ControlAction::GoalTasks {
                project,
                goal,
                after: tasks[0].scope.task_id,
                maximum: 1,
                view: Some(GoalReadView::RecordedV1)
            }
        )
        .await
        .is_err()
    );
    assert!(
        client::request(
            &f.state,
            ControlAction::GoalStatus {
                project: foreign,
                goal,
                view: Some(GoalReadView::RecordedV1)
            }
        )
        .await
        .is_err()
    );
    let (identity, connection) = endpoint::connect(&f.state).await.unwrap();
    drop(connection);
    let nominal = serde_json::to_string(&ControlRequest {
        request_id: Uuid::new_v4(),
        instance: identity.instance.clone(),
        epoch: identity.epoch,
        action: ControlAction::GoalStatus {
            project,
            goal,
            view: Some(GoalReadView::RecordedV1),
        },
    })
    .unwrap();
    for raw in [
        nominal.replace("recorded_v1", "unknown_view"),
        nominal.replace(
            "\"view\":\"recorded_v1\"",
            "\"view\":\"recorded_v1\",\"view\":null",
        ),
        nominal.replace(
            "\"view\":\"recorded_v1\"",
            "\"view\":\"recorded_v1\",\"principal\":\"administrator\"",
        ),
        nominal.replace(
            &format!("\"epoch\":{}", identity.epoch),
            &format!("\"epoch\":{}", identity.epoch + 1),
        ),
        nominal.replace(&identity.instance, &Uuid::new_v4().to_string()),
    ] {
        let (_, mut connection) = endpoint::connect(&f.state).await.unwrap();
        connection
            .get_mut()
            .write_all(format!("{raw}\n").as_bytes())
            .await
            .unwrap();
        assert!(matches!(
            transport::receive::<ControlResponse>(&mut connection, transport::RESPONSE_BYTES)
                .await
                .unwrap(),
            ControlResponse::Rejected { .. }
        ));
    }
    let legacy = client::request(
        &f.state,
        ControlAction::GoalStatus {
            project,
            goal,
            view: None,
        },
    )
    .await
    .unwrap();
    assert!(
        serde_json::to_value(legacy)
            .unwrap()
            .get("recorded")
            .is_none()
    );
    assert_eq!(
        rows(&f.state),
        before,
        "recorded success/refusal must preserve complete durable row images"
    );
    f.stop().await;
}

#[tokio::test]
async fn recorded_proposal_maximum_escapes_and_legacy_consumers_remain_readable() {
    let mut f = Fixture::new();
    let project = f.project("one");
    f.start().await;
    let objective = "\u{1}".repeat(16 * 1024);
    let path = f.dir.path().join("objective.txt");
    std::fs::write(&path, &objective).unwrap();
    let proposed = f.cli_json(&[
        "goal",
        "--file",
        path.to_str().unwrap(),
        "--project",
        "one",
        "--json",
    ]);
    let goal = proposed["facts"]["goal"].as_str().unwrap();
    let before = f.idle().await;
    let args = ["goal", "status", goal, "--project", "one", "--json"];
    let status = f.cli_json(&args);
    assert_eq!(
        status["facts"]["objective"], objective,
        "recorded maximum proposal must remain byte-exact/readable"
    );
    assert_eq!(status["facts"]["recorded"]["accepted"], false);
    assert_eq!(status["facts"]["recorded"]["criteria"], "not_accepted");
    let plain = f.plain(&args[..args.len() - 1]);
    let quoted = plain
        .lines()
        .find_map(|line| line.strip_prefix("Objective: "))
        .unwrap();
    assert_eq!(serde_json::from_str::<String>(quoted).unwrap(), objective);
    let legacy = client::request(
        &f.state,
        ControlAction::GoalStatus {
            project,
            goal: goal.parse().unwrap(),
            view: None,
        },
    )
    .await
    .unwrap();
    let legacy = serde_json::to_value(legacy).unwrap();
    assert_eq!(legacy["objective"], objective);
    assert!(legacy.get("recorded").is_none());
    if let Some(binary) = std::env::var_os("RRX_LEGACY_CLI") {
        for json in [false, true] {
            let mut command = Command::new(binary.clone());
            command
                .arg("--state")
                .arg(&f.state)
                .arg("--config")
                .arg(&f.config)
                .current_dir(f.dir.path())
                .args(&args[..args.len() - 1]);
            if json {
                command.arg("--json");
            }
            let out = command.output().unwrap();
            assert!(
                out.status.success(),
                "genuine c979 legacy consumer refused valid proposal"
            );
            let value = serde_json::Deserializer::from_slice(&out.stdout)
                .into_iter::<Value>()
                .next()
                .unwrap()
                .unwrap();
            assert_eq!(
                if json {
                    &value["facts"]["objective"]
                } else {
                    &value["objective"]
                },
                &Value::String(objective.clone())
            );
        }
    } else {
        eprintln!(
            "UNVERIFIED: c979 compiled legacy plain/JSON artifact not supplied; omitted-wire positive remains covered"
        );
    }
    assert!(
        !f.command()
            .args(["goal", "tasks", goal, "--project", "one"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(rows(&f.state), before);
    let small = f.cli_json(&["goal", "small\u{7f}\u{9b}é", "--project", "one", "--json"]);
    let small_goal = small["facts"]["goal"].as_str().unwrap();
    let before = f.idle().await;
    no_terminal_controls(&f.plain(&["goal", "status", small_goal, "--project", "one"]));
    assert_eq!(
        f.cli_json(&["goal", "status", small_goal, "--project", "one", "--json"])["facts"]["objective"],
        "small\u{7f}\u{9b}é"
    );
    assert_eq!(rows(&f.state), before);
    f.stop().await;
}

#[tokio::test]
async fn recorded_budgets_pack_whole_sets_and_fresh_independent_pages() {
    let mut f = Fixture::new();
    let project = f.project("one");
    f.start().await;
    let mut p = plan(128);
    p.definition.criteria = (0..16)
        .map(|i| CriterionDefinition {
            id: format!("c{i}"),
            description: "large criterion é".repeat(700),
            evaluator: CriterionEvaluator::RequiredTasksVerified,
        })
        .collect();
    for dependent in 80..128 {
        for prerequisite in 0..60 {
            p.dependencies.push(PlanDependency {
                prerequisite: format!("t{prerequisite}"),
                dependent: format!("t{dependent}"),
                hard: true,
            });
        }
    }
    for prerequisite in 60..127 {
        p.dependencies.push(PlanDependency {
            prerequisite: format!("t{prerequisite}"),
            dependent: "t127".into(),
            hard: false,
        });
    }
    let goal = f.accept_plan(project, p).await;
    let before = f.idle().await;
    let status = f.cli_json(&[
        "goal",
        "status",
        &goal.to_string(),
        "--project",
        "one",
        "--json",
    ]);
    assert_eq!(
        status["facts"]["recorded"]["criteria"],
        serde_json::json!({"availability":"unavailable","recorded_count":16,"reason":"projection_budget"})
    );
    assert!(
        status["unavailable_fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "recorded_criteria")
    );
    assert!(serde_json::to_vec(&status["facts"]).unwrap().len() <= 128 * 1024);
    let mut after = None;
    let mut seen = Vec::new();
    let mut overflow = false;
    let mut reduced = false;
    let mut pages = 0;
    loop {
        let response = client::request(
            &f.state,
            ControlAction::GoalTasks {
                project,
                goal,
                after,
                maximum: 128,
                view: Some(GoalReadView::RecordedV1),
            },
        )
        .await
        .unwrap();
        assert!(
            serde_json::to_vec(&response).unwrap().len() <= 64 * 1024,
            "paired page byte cap"
        );
        let value = serde_json::to_value(&response).unwrap();
        let nodes = value["recorded"]["nodes"].as_array().unwrap();
        let tasks = value["tasks"].as_array().unwrap();
        assert_eq!(nodes.len(), tasks.len());
        reduced |= pages == 0 && nodes.len() < 128;
        for (node, task) in nodes.iter().zip(tasks) {
            assert_eq!(node["task"], task["scope"]["task_id"]);
            seen.push(node["task"].as_str().unwrap().to_owned());
            if node["incoming"]["availability"] == "unavailable" {
                overflow = true;
                assert_eq!(node["incoming_count"], 127);
                assert_eq!(node["hard_incoming_count"], 60);
                keys(&node["incoming"], &["availability", "reason"]);
            } else {
                assert_eq!(
                    node["incoming_count"],
                    node["incoming"]["items"].as_array().unwrap().len()
                );
                assert!(serde_json::to_vec(&node["incoming"]).unwrap().len() <= 8 * 1024);
            }
        }
        let ControlResponse::GoalTaskPage { next, .. } = response else {
            panic!()
        };
        pages += 1;
        if let Some(next) = next {
            assert_eq!(next.to_string(), *seen.last().unwrap());
            after = Some(next);
        } else {
            break;
        }
        assert!(pages <= 128);
    }
    assert!(
        reduced && overflow && pages > 1,
        "genuine DAG must reach page/incoming caps"
    );
    assert_eq!(seen.len(), 128);
    assert!(
        seen.windows(2).all(|w| w[0] < w[1]),
        "no omitted/duplicate Task"
    );
    let actual = stored(&f.state, "tasks");
    assert_eq!(
        seen.iter().cloned().collect::<BTreeSet<_>>(),
        actual.keys().cloned().collect()
    );
    let t127 = actual
        .iter()
        .find(|(_, task)| task["title"] == "Task 127")
        .unwrap()
        .0
        .clone();
    let mut cursor = None::<String>;
    let mut plain_seen = Vec::new();
    let mut plain_overflow = false;
    loop {
        let (plain, ids, next) = f.plain_page(project, goal, cursor.as_deref(), 128);
        if plain_seen.is_empty() {
            assert!(
                ids.len() < 128 && next.is_some(),
                "genuine DAG must reduce the first plain page"
            );
        }
        if ids.contains(&t127) {
            plain_overflow = true;
            assert!(plain.contains(
                "Structural dependencies: \"requires_prerequisite_evidence\"; incoming: 127 (hard: 60)\n  Incoming detail: unavailable (projection_budget)\n"
            ));
            assert!(plain.contains(", task_incoming_relationships\n"));
        }
        plain_seen.extend(ids);
        assert!(plain_seen.len() <= 128);
        match next {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }
    assert!(
        plain_overflow,
        "scoped cursors must reach the 127-incoming Task"
    );
    assert_eq!(plain_seen, seen);
    assert_eq!(rows(&f.state), before);
    client::request(
        &f.state,
        ControlAction::SetGoalLifecycle {
            project,
            goal,
            expected_goal: 1,
            target: GoalControl::Pause,
            reason: "independent read version control".into(),
        },
    )
    .await
    .unwrap();
    let before = f.idle().await;
    let fresh = f.cli_json(&[
        "goal",
        "tasks",
        &goal.to_string(),
        "--project",
        "one",
        "--json",
    ]);
    assert_eq!(fresh["facts"]["version"], 2);
    assert_eq!(fresh["complete"], false);
    assert_eq!(rows(&f.state), before);
    f.stop().await;
}

#[tokio::test]
async fn recorded_new_cli_refuses_old_service_without_downgrade_or_writes() {
    let Some(binary) = std::env::var_os("RRX_LEGACY_CLI") else {
        eprintln!("UNVERIFIED: compiled c979 old-service artifact absent");
        return;
    };
    let mut f = Fixture::new();
    let project = f.project("one");
    f.child = Some(
        Command::new(binary)
            .arg("--state")
            .arg(&f.state)
            .arg("--config")
            .arg(&f.config)
            .arg("serve")
            .current_dir(f.dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(f.child.as_mut().unwrap().try_wait().unwrap().is_none());
        if client::request(&f.state, ControlAction::RuntimeStatus)
            .await
            .is_ok()
        {
            break;
        }
        assert!(Instant::now() < until);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let goal = f.create(project, 1).await;
    let before = f.idle().await;
    for verb in ["status", "tasks"] {
        let out = f
            .command()
            .args([
                "goal",
                verb,
                &goal.to_string(),
                "--project",
                "one",
                "--json",
            ])
            .output()
            .unwrap();
        assert!(
            !out.status.success(),
            "new CLI must not silently downgrade old service"
        );
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8_lossy(&out.stderr).contains("restart a matching rrx serve"));
    }
    assert!(
        client::request(
            &f.state,
            ControlAction::GoalStatus {
                project,
                goal,
                view: None
            }
        )
        .await
        .is_ok()
    );
    assert_eq!(rows(&f.state), before);
    f.stop().await;
}
