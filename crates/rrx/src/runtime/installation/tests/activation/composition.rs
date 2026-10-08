use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca2_four_tasks_two_projects_retain_own_scope_and_roster() {
    let mut f = fixture_with("claude", true, |config| {
        let mut codex = config.agents["worker"].clone();
        let old_path = std::path::Path::new(&codex.command[0]);
        let codex_path = old_path.with_file_name("configured-codex-protocol-fixture");
        let source = std::fs::read_to_string(old_path).unwrap().replacen(
            "PROVIDER=\"claude\"",
            "PROVIDER=\"codex\"",
            1,
        );
        std::fs::write(&codex_path, source).unwrap();
        std::fs::set_permissions(&codex_path, std::fs::Permissions::from_mode(0o700)).unwrap();
        codex.command = vec![codex_path.to_string_lossy().into()];
        codex.provider = Some("codex".into());
        codex.compatibility.as_mut().unwrap().cli_version = "codex-cli 0.160.0".into();
        config.agents.insert("codex-worker".into(), codex);
    });
    let observations = Arc::new(Mutex::new(Vec::new()));
    let captures = observations.clone();
    engine(&f).set_activation_hooks(
        Some(Arc::new(move |probe| {
            captures
                .lock()
                .unwrap()
                .push(ObservedActivation::from_probe(&probe));
            Box::pin(async {})
        })),
        None,
    );
    f.runtime.start().await.unwrap();
    let mut tasks = Vec::new();
    let mut workflows = Vec::new();
    // R4.1/R4.4 (S2): one active Task per Project, so the four concurrent
    // Tasks are in four Projects, both executors in each provider pair.
    for (project, executor) in [
        ("ca2-project-a", "worker"),
        ("ca2-project-b", "codex-worker"),
        ("ca2-project-c", "worker"),
        ("ca2-project-d", "codex-worker"),
    ] {
        f.register_real_git_project_named(project);
        let mut p = task_plan(&["rev-a", "rev-b"]);
        p.tasks[0].executor = executor.into();
        let task = accepted_plan(&f, p).await.remove(0);
        let (_, workflow) = wait_bound(&f, &task).await;
        tasks.push(task);
        workflows.push(workflow);
    }
    assert_eq!(
        count(&f, "workflow_native_contracts"),
        4,
        "CA2 own activation count"
    );
    assert_eq!(
        count(&f, "native_invocations"),
        4,
        "CA2 configured peer launch count"
    );
    let actual = observations.lock().unwrap().clone();
    assert_eq!(actual.len(), 4);
    for o in &actual {
        assert_contract(&f, o, "CA2 original scoped contract");
    }
    assert_eq!(
        tasks
            .iter()
            .map(|t| t.project_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4,
        "CA2 one active Task per Project"
    );
    let unique = actual
        .iter()
        .map(|o| o.expected["workflow_id"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), 4, "CA2 cross-Workflow reuse");
    for workflow in &workflows {
        release_peer(&f, workflow);
    }
    finish(f).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca3_representable_roster_refusals_preserve_claimed_sibling() {
    use crate::runtime::task_driver::{AdmitOutcome, SkipReason};
    use crate::state::CandidatePage;
    for case in ["empty", "undeclared", "nine"] {
        let mut f = fixture_with("claude", true, |config| {
            if case == "undeclared" {
                config.agents.get_mut("rev-a").unwrap().compatibility = None;
            }
            if case == "nine" {
                for i in 0..9 {
                    config
                        .agents
                        .insert(format!("r-{i}"), config.agents["worker"].clone());
                }
            }
        });
        f.register_real_git_project();
        let mut p = task_plan(&["rev-b"]);
        let mut bad = p.tasks[0].clone();
        bad.key = "bad".into();
        bad.title = "bad roster".into();
        bad.reviewers = match case {
            "empty" => vec![],
            "undeclared" => vec!["rev-a".into()],
            "nine" => (0..9).map(|i| format!("r-{i}")).collect(),
            _ => unreachable!(),
        };
        p.tasks.push(bad);
        let tasks = accepted_plan(&f, p).await;
        let bad = tasks.iter().find(|t| t.title == "bad roster").unwrap();
        let sibling = tasks.iter().find(|t| t.id != bad.id).unwrap();
        let page = f
            .owner
            .store
            .lock()
            .unwrap()
            .ready_driver_candidates(f.owner.instance_id(), f.owner.epoch(), None, 12)
            .unwrap();
        let CandidatePage::Rows { keys, .. } = page else {
            panic!("SETUP: candidates absent");
        };
        let key = keys
            .iter()
            .find(|k| k.task_id == bad.id.to_string())
            .unwrap();
        let (observations, release) = install_pause(&f, None);
        f.runtime.start().await.unwrap();
        let admission = f.runtime.control_admission.lock().await;
        assert!(
            matches!(
                f.runtime
                    .evaluate_driver_candidate(&admission, key)
                    .unwrap(),
                AdmitOutcome::Skipped(SkipReason::Composition)
            ),
            "CA3 {case}: named composition refusal"
        );
        drop(admission);
        wait_for(
            || observations.lock().unwrap().len() == 1,
            "SETUP: genuine sibling did not reach S1",
        )
        .await;
        assert_eq!(
            observations.lock().unwrap()[0].expected["task_id"],
            sibling.id.to_string()
        );
        for table in [
            "task_drivers",
            "execution_units",
            "source_recoveries",
            "workflow_native_contracts",
        ] {
            let n: u64 = raw(&f)
                .query_row(
                    &format!("SELECT count(*) FROM {table} WHERE task_id=?1"),
                    [bad.id.to_string()],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(n, 0, "CA3 {case}: refusal wrote {table}");
        }
        assert!(
            f.owner
                .store
                .lock()
                .unwrap()
                .records(&bad.scope(), RecordKind::Workflow)
                .unwrap()
                .is_empty(),
            "CA3 {case}: refusal wrote Workflow"
        );
        release.add_permits(1);
        let (_, workflow) = wait_bound(&f, sibling).await;
        release_peer(&f, &workflow);
        finish(f).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca3_missing_alias_is_rejected_at_accepted_ingress() {
    let f = fixture("claude", true);
    let error = f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::CreateGoal {
                project: f.project.id,
                expected_project: f.project.version,
                plan: task_plan(&["missing"]),
            }),
        )
        .await
        .unwrap_err();
    assert!(
        error.to_string().contains("Task reviewer not configured"),
        "{error:#}"
    );
    assert_eq!(count(&f, "goals"), 0);
    assert_eq!(count(&f, "task_drivers"), 0);
    // This earlier-ingress negative is not CA3 composition-stage credit.
    finish(f).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca3b_original_composition_cross_pairs_refuse_before_store() {
    let mut f = fixture("claude", true);
    f.register_real_git_project_named("ca3b-project-a");
    let task_a = accepted_plan(&f, task_plan(&["rev-a"])).await.remove(0);
    f.register_real_git_project_named("ca3b-project-b");
    let task_b = accepted_plan(&f, task_plan(&["rev-b"])).await.remove(0);
    let (a, b) = (task_a.id, task_b.id);
    let tasks = [task_a, task_b];
    // SAME Runtime/registry, distinct genuine Projects. The original same-Project
    // failure is preserved in evidence; no namespace image is refreshed here.
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let checks = Arc::new(AtomicBool::new(false));
    let captures = Arc::new(Mutex::new(Vec::new()));
    let signal = checks.clone();
    let observed = captures.clone();
    let permits = release.clone();
    engine(&f).set_activation_hooks(
        Some(Arc::new(move |probe| {
            observed
                .lock()
                .unwrap()
                .push(ObservedActivation::from_probe(&probe));
            let permits = permits.clone();
            let signal = signal.clone();
            Box::pin(async move {
                if probe.task().id == b {
                    let runtime = probe.composition().runtime().upgrade().unwrap();
                    let task_a = runtime
                        .owner
                        .store
                        .lock()
                        .unwrap()
                        .task(a)
                        .unwrap()
                        .unwrap();
                    let task_b = runtime
                        .owner
                        .store
                        .lock()
                        .unwrap()
                        .task(b)
                        .unwrap()
                        .unwrap();
                    let fresh_a = runtime.installed_driver_composition(&task_a).unwrap();
                    let fresh_b = runtime.installed_driver_composition(&task_b).unwrap();
                    let cross = crate::state::managed_binding::plan_native_activation(
                        fresh_a.activation_roster(&task_a).unwrap(),
                        probe.record(),
                        probe.task(),
                    );
                    assert!(cross.is_err(), "CA3bi: cross-pair constructed a plan");
                    assert!(
                        fresh_a
                            .admit_activation(probe.plan(), probe.lifetime())
                            .await
                            .is_err(),
                        "CA3bii: sibling composition admitted plan"
                    );
                    assert!(
                        fresh_b
                            .admit_activation(probe.plan(), probe.lifetime())
                            .await
                            .is_err(),
                        "CA3biii: fresh same-Task identity admitted plan"
                    );
                    let store = runtime.owner.store.lock().unwrap();
                    assert!(
                        store
                            .records(&task_a.scope(), RecordKind::Workflow)
                            .unwrap()
                            .is_empty()
                    );
                    assert!(
                        store
                            .records(&task_b.scope(), RecordKind::Workflow)
                            .unwrap()
                            .is_empty()
                    );
                    drop(store);
                    drop(runtime);
                    drop(fresh_a);
                    drop(fresh_b);
                    signal.store(true, Ordering::SeqCst);
                }
                permits.acquire().await.unwrap().forget();
            })
        })),
        None,
    );
    f.runtime.start().await.unwrap();
    wait_for(
        || checks.load(Ordering::SeqCst) && captures.lock().unwrap().len() == 2,
        "SETUP: both genuine sibling preparations/cross checks absent",
    )
    .await;
    assert_eq!(count(&f, "workflow_native_contracts"), 0);
    release.add_permits(2);
    for task in &tasks {
        let (_, workflow) = wait_bound(&f, task).await;
        release_peer(&f, &workflow);
    }
    let observations = captures.lock().unwrap().clone();
    for o in &observations {
        assert_contract(&f, o, "CA3b valid pair");
    }
    assert_ne!(
        observations[0].expected["profile_digest"], observations[1].expected["profile_digest"],
        "CA3b own reviewer rosters reused"
    );
    assert_eq!(count(&f, "workflow_native_contracts"), 2);
    finish(f).await;
}
