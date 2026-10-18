use super::*;

async fn refuse_pipeline(f: ControlFixture, subject: &str) {
    let path = f.state_path();
    let dir = f._dir;
    let retained = Arc::downgrade(&f.owner);
    drop(f.runtime);
    drop(f.owner);
    drop(f.socket);
    tokio::time::timeout(Duration::from_secs(10), async {
        while retained.upgrade().is_some() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("fixture owner custody did not finish");
    let store = crate::state::Store::open(&path).unwrap();
    let before = store.pipeline_mode_inventory().unwrap();
    match subject {
        "driver" => assert_eq!(before.native_drivers, 1),
        "phase" => assert_eq!(before.native_phases, 1),
        _ => unreachable!(),
    }
    let rows = || {
        let connection = rusqlite::Connection::open(&path).unwrap();
        [
            "runtime_epoch",
            "task_drivers",
            "managed_phase_operations",
            "execution_units",
        ]
        .map(|table| {
            let mut q = connection
                .prepare(&format!("SELECT * FROM {table} ORDER BY rowid"))
                .unwrap();
            let n = q.column_count();
            q.query_map([], |r| {
                (0..n)
                    .map(|i| r.get::<_, rusqlite::types::Value>(i))
                    .collect::<rusqlite::Result<Vec<_>>>()
            })
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
        })
    };
    let state = rows();
    let error = RuntimeOwner::open_mode(&path, crate::config::ExecutionMode::CliPipeline)
        .err()
        .unwrap();
    assert!(
        error
            .downcast_ref::<crate::execution::owner::ExecutionModeNotQuiescent>()
            .is_some(),
        "{error:#}"
    );
    assert_eq!(store.pipeline_mode_inventory().unwrap(), before);
    assert_eq!(rows(), state);
    drop(store);
    drop(dir);
}

/// A genuinely committed initial driver blocks mode switching before epoch or row changes.
#[tokio::test]
async fn b3_owner_native_driver() {
    let f = fixture("claude", true);
    let (_, tasks) = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    // No yield between startup and admission: the service cannot claim first.
    let admission = f.runtime.control_admission.lock().await;
    let composition = f.runtime.installed_driver_composition(&tasks[0]).unwrap();
    let plan = crate::state::plan_initial_driver(f.owner.clone(), tasks[0].id, 1).unwrap();
    let pending = f
        .owner
        .store
        .lock()
        .unwrap()
        .claim_initial_driver(plan, composition)
        .unwrap();
    drop(pending);
    drop(admission);
    refuse_pipeline(f, "driver").await;
}

/// A real held native phase remains an obstruction after its preparation has finished.
#[tokio::test]
async fn b3_owner_native_phase() {
    use crate::execution::native::PreparationObservation;
    let mut f = fixture("claude", true);
    f.register_real_git_project();
    let adapter = f
        .runtime
        .installed
        .as_ref()
        .ok()
        .unwrap()
        .registry
        .native_phase_port("worker")
        .unwrap()
        .selected_adapter()
        .unwrap();
    let arrived = Arc::new(tokio::sync::Semaphore::new(0));
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let (on_arrival, on_release) = (arrived.clone(), release.clone());
    adapter
        .sessions
        .set_preparation_observer(Some(Arc::new(move |observation| {
            let (arrived, release) = (on_arrival.clone(), on_release.clone());
            Box::pin(async move {
                if observation == PreparationObservation::BeforeTransport {
                    arrived.add_permits(1);
                    release.acquire().await.unwrap().forget();
                }
            })
        })));
    let (goal, _) = accept(&f, 1).await;
    f.runtime.start().await.unwrap();
    tokio::time::timeout(Duration::from_secs(40), arrived.acquire())
        .await
        .unwrap()
        .unwrap()
        .forget();
    let version = f
        .owner
        .store
        .lock()
        .unwrap()
        .goal(goal)
        .unwrap()
        .unwrap()
        .version;
    let response = f
        .runtime
        .handle_control(
            &f.socket,
            f.request(ControlAction::SetGoalLifecycle {
                project: f.project.id,
                goal,
                expected_goal: version,
                target: GoalControl::Pause,
                reason: "retain held phase".into(),
            }),
        )
        .await
        .unwrap();
    assert!(matches!(
        response,
        ControlResponse::GoalLifecycleChanged { .. }
    ));
    release.add_permits(1);
    tokio::time::timeout(Duration::from_secs(40), async {
        while !f
            .runtime
            .phase_jobs
            .observed_jobs()
            .iter()
            .any(|job| job.finished && job.attention == Some("original non-success plan Held"))
        {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    drop(adapter);
    refuse_pipeline(f, "phase").await;
}
