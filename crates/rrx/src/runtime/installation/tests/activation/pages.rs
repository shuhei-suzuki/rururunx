use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn ca4i_whole_registry_counts_actual_held_plan_across_two_pages() {
    for one_held in [true, false] {
        let mut f = fixture_with("claude", true, |config| {
            config.scheduler.global_max_sessions = 128;
        });
        let mut tasks = Vec::new();
        for i in 0..128 {
            f.register_real_git_project_named(&format!("ca4i-project-{i}"));
            tasks.extend(accepted_plan(&f, task_plan(&["rev-a", "rev-b"])).await);
        }
        assert_eq!(
            tasks
                .iter()
                .map(|t| t.project_id)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            128
        );
        // SAME Runtime with 128 genuine accepted Projects. Original whole-Project
        // namespace pins remain unchanged; same-Project contention stays open.
        let held = tasks[0].id;
        let (park, _open) = service_park(&f);
        let release_held = Arc::new(tokio::sync::Semaphore::new(0));
        let release_rollback = Arc::new(tokio::sync::Semaphore::new(0));
        let observations = Arc::new(Mutex::new(Vec::new()));
        let precommit_calls = Arc::new(AtomicUsize::new(0));
        let counted = precommit_calls.clone();
        let seams = Arc::new(ActivationSeams {
            precommit: Some(Arc::new(move |task| {
                counted.fetch_add(1, Ordering::SeqCst);
                if !one_held || task != held {
                    anyhow::bail!("CA4i genuine precommit rollback");
                }
                Ok(())
            })),
            postcommit: Some(Arc::new(|_| anyhow::bail!("CA4i genuine Deferred"))),
            deferred: Some(Arc::new(|_| {
                Box::pin(async { anyhow::bail!("CA4i actual worker exit") })
            })),
            ..Default::default()
        });
        let captures = observations.clone();
        let held_permits = release_held.clone();
        let rollback_permits = release_rollback.clone();
        engine(&f).set_activation_hooks(
            Some(Arc::new(move |probe| {
                captures
                    .lock()
                    .unwrap()
                    .push(ObservedActivation::from_probe(&probe));
                let permits = if one_held && probe.task().id == held {
                    held_permits.clone()
                } else {
                    rollback_permits.clone()
                };
                Box::pin(async move {
                    permits.acquire().await.unwrap().forget();
                })
            })),
            Some(seams),
        );
        f.runtime.start().await.unwrap();
        let setup_end = tokio::time::Instant::now() + Duration::from_secs(120);
        let mut progress = tokio::time::Instant::now();
        while observations.lock().unwrap().len() != 128 {
            if tokio::time::Instant::now() >= progress {
                let exits = f.runtime._drivers.pending_exits().unwrap();
                let labels = exits.iter().map(|e| e.label()).collect::<Vec<_>>();
                eprintln!(
                    "CA4i actual setup progress: claimed={} S1={} unfinished={} exits_page={labels:?} retained={} units={} S2={}",
                    count(&f, "task_drivers"),
                    observations.lock().unwrap().len(),
                    f.runtime._drivers.observe_finished().unwrap(),
                    f.runtime._drivers.retained_preparations().unwrap(),
                    count(&f, "execution_units"),
                    precommit_calls.load(Ordering::SeqCst)
                );
                progress = tokio::time::Instant::now() + Duration::from_secs(10);
            }
            if tokio::time::Instant::now() >= setup_end {
                let claims = count(&f, "task_drivers");
                let s1 = observations.lock().unwrap().len();
                let unfinished = f.runtime._drivers.observe_finished().unwrap();
                eprintln!(
                    "CA4i SETUP snapshot before teardown: claims={claims} S1={s1} unfinished={unfinished}"
                );
                let shutdown = f.runtime.shutdown().await;
                eprintln!("CA4i controlled setup shutdown: {shutdown:?}");
                finish(f).await;
                panic!(
                    "SETUP: fewer than 128 genuine retained S1 preparations; claims={claims} S1={s1} unfinished={unfinished}"
                );
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert_eq!(
            count(&f, "task_drivers"),
            128,
            "SETUP: fewer than 128 genuine claims"
        );
        assert_eq!(f.runtime._drivers.retained_preparations().unwrap(), 128);
        park.close();
        f.runtime.wake.notify_one();
        wait_for(|| park.parked(), "SETUP: S5 absent").await;
        // All rollback segments run before the single Task-changing commit.
        // This preserves every original whole-Project namespace snapshot.
        release_rollback.add_permits(if one_held { 127 } else { 128 });
        wait_for(
            || precommit_calls.load(Ordering::SeqCst) == if one_held { 127 } else { 128 },
            "CA4i rollback workers did not reach their genuine S2",
        )
        .await;
        if one_held {
            wait_for(
                || f.runtime._drivers.observe_finished().unwrap() == 1,
                "CA4i rollback jobs remain unfinished",
            )
            .await;
            release_held.add_permits(1);
        }
        wait_for(
            || f.runtime._drivers.observe_finished().unwrap() == 0,
            "CA4i actual jobs remain unfinished",
        )
        .await;
        let before = observations
            .lock()
            .unwrap()
            .iter()
            .find(|o| o.expected["task_id"] == held.to_string())
            .unwrap()
            .clone();
        let first = f.runtime.observe_task_drivers().unwrap();
        let second = f.runtime.observe_task_drivers().unwrap();
        assert!(
            first >= 1,
            "CA4i first page omitted remaining registry custody"
        );
        if one_held {
            assert!(
                second >= 1,
                "CA4i page excluding Held omitted registry custody"
            );
            assert_eq!(f.runtime._drivers.retained_preparations().unwrap(), 1);
            let actual = f.runtime._drivers.pending_preparations().unwrap();
            assert_eq!(actual.len(), 1);
            assert!(
                Arc::ptr_eq(&actual[0], &before.plan),
                "CA4i substituted Held plan"
            );
            assert!(before.plan.is_retained().unwrap());
            assert_eq!(count(&f, "workflow_native_contracts"), 1);
            assert_contract(&f, &before, "CA4i original Held");
        } else {
            assert_eq!(
                second, 0,
                "CA4i count read before second rollback reconciliation"
            );
            assert_eq!(f.runtime._drivers.retained_preparations().unwrap(), 0);
            assert_eq!(count(&f, "workflow_native_contracts"), 0);
        }
        park.open();
        if one_held {
            let error = f.runtime.shutdown().await.unwrap_err();
            eprintln!("CA4i shutdown pending, no timeout credit: {error:#}");
        } else {
            f.runtime.shutdown().await.unwrap();
        }
        finish(f).await;
    }
}
