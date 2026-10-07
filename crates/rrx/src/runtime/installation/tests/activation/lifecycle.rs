use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca4b_activation_segment_linearizes_before_shutdown() {
    let mut f = fixture("claude", true);
    f.register_real_git_project();
    accept(&f, 1).await;
    let block = Arc::new(ServicePark::default());
    block.close();
    let _release_on_drop = OpenPark(block.clone());
    let segment = block.clone();
    let captures = Arc::new(Mutex::new(None::<ObservedActivation>));
    let observed = captures.clone();
    let published = Arc::new(AtomicBool::new(false));
    let signal = published.clone();
    let seams = Arc::new(ActivationSeams {
        precommit: Some(Arc::new(move |_| {
            segment.visit();
            Ok(())
        })),
        published: Some(Arc::new(move || {
            let guard = observed.lock().unwrap();
            let original = guard.as_ref().unwrap();
            let expected = original.plan.planned_binding();
            assert_eq!(
                original.association.binding().unwrap(),
                (expected.0, expected.1, expected.2, expected.3.to_owned()),
                "CA4b SAME exact cache publication inside activation segment"
            );
            assert!(!original.plan.is_retained().unwrap());
            signal.store(true, Ordering::SeqCst);
        })),
        ..Default::default()
    });
    let (observations, release) = install_pause(&f, Some(seams));
    f.runtime.start().await.unwrap();
    wait_for(
        || observations.lock().unwrap().len() == 1,
        "SETUP: S1 absent",
    )
    .await;
    let original = observations.lock().unwrap()[0].clone();
    *captures.lock().unwrap() = Some(original.clone());
    release.add_permits(1);
    block.wait_parked();
    let runtime = f.runtime.clone();
    let executor = tokio::runtime::Handle::current();
    let queued = Arc::new(AtomicBool::new(false));
    let waiting = queued.clone();
    let shutdown = std::thread::spawn(move || {
        let _entered = executor.enter();
        let mut future = Box::pin(runtime.shutdown());
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        match std::future::Future::poll(future.as_mut(), &mut context) {
            std::task::Poll::Pending => waiting.store(true, Ordering::SeqCst),
            std::task::Poll::Ready(result) => return result,
        }
        // Continue the SAME already-polled shutdown future. Its mutex waiter is
        // queued before the test releases S2; scheduling alone is not evidence.
        executor.block_on(future)
    });
    let queue_deadline = std::time::Instant::now() + Duration::from_secs(2);
    while !queued.load(Ordering::SeqCst) {
        assert!(
            !f.runtime.is_stopping(),
            "CA4b stop passed the original activation admission"
        );
        assert!(
            std::time::Instant::now() < queue_deadline,
            "SETUP: actual shutdown admission wait not polled"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let end = std::time::Instant::now() + Duration::from_secs(1);
    while std::time::Instant::now() < end {
        assert!(
            !f.runtime.is_stopping(),
            "CA4b stop passed the original activation admission"
        );
        assert!(
            !shutdown.is_finished(),
            "CA4b shutdown completed inside admitted segment"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    block.open();
    shutdown.join().unwrap().unwrap();
    assert!(
        published.load(Ordering::SeqCst),
        "CA4b no exact SAME cache publication in segment"
    );
    assert_eq!(
        count(&f, "workflow_native_contracts"),
        1,
        "CA4b admitted activation did not commit"
    );
    assert_contract(&f, &original, "CA4b linearized commit");
    assert!(
        !original.plan.is_retained().unwrap(),
        "CA4b cache not published before stop"
    );
    finish(f).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca4b_segment_keeps_runtime_alive_after_external_drop() {
    let mut f = fixture("claude", true);
    f.register_real_git_project();
    accept(&f, 1).await;
    let block = Arc::new(ServicePark::default());
    block.close();
    let _release_on_drop = OpenPark(block.clone());
    let segment = block.clone();
    let seams = Arc::new(ActivationSeams {
        precommit: Some(Arc::new(move |_| {
            segment.visit();
            Ok(())
        })),
        ..Default::default()
    });
    let (observations, release) = install_pause(&f, Some(seams));
    f.runtime.start().await.unwrap();
    wait_for(
        || observations.lock().unwrap().len() == 1,
        "SETUP: S1 absent",
    )
    .await;
    release.add_permits(1);
    block.wait_parked();
    let weak = Arc::downgrade(&f.runtime);
    drop(f.runtime);
    assert!(
        weak.upgrade().is_some(),
        "CA4b Runtime ended within synchronous activation segment"
    );
    block.open();
    wait_for(
        || weak.upgrade().is_none(),
        "CA4b Runtime failed to end after segment",
    )
    .await;
    // This observation does not isolate a strong-retention mutant: the service
    // may also own a strong reference, as the approved HOW records.
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca4a_stop_at_genuine_retained_pre_admission_rolls_back() {
    let mut f = fixture("claude", true);
    f.register_real_git_project();
    accept(&f, 1).await;
    let (observations, _release) = install_pause(&f, None);
    f.runtime.start().await.unwrap();
    wait_for(
        || observations.lock().unwrap().len() == 1,
        "SETUP: S1 absent",
    )
    .await;
    let original = observations.lock().unwrap()[0].clone();
    assert!(original.plan.is_retained().unwrap());
    f.runtime.shutdown().await.unwrap();
    assert_eq!(
        count(&f, "workflow_native_contracts"),
        0,
        "CA4a activated after stop"
    );
    assert_eq!(count(&f, "records"), 0, "CA4a rollback left records");
    assert!(
        !original.plan.is_retained().unwrap(),
        "CA4a SAME rollback plan not reconciled"
    );
    assert_eq!(f.runtime._drivers.retained_preparations().unwrap(), 0);
    finish(f).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca4c_precommit_failure_proves_same_plan_rollback() {
    let mut f = fixture("claude", true);
    f.register_real_git_project();
    accept(&f, 1).await;
    let seams = Arc::new(ActivationSeams {
        precommit: Some(Arc::new(|_| {
            anyhow::bail!("CA4c injected precommit failure")
        })),
        ..Default::default()
    });
    let (observations, release) = install_pause(&f, Some(seams));
    f.runtime.start().await.unwrap();
    wait_for(
        || observations.lock().unwrap().len() == 1,
        "SETUP: S1 absent",
    )
    .await;
    let original = observations.lock().unwrap()[0].clone();
    release.add_permits(1);
    wait_for(
        || !original.plan.is_retained().unwrap(),
        "CA4c SAME plan rollback not reconciled",
    )
    .await;
    assert_eq!(count(&f, "workflow_native_contracts"), 0);
    assert_eq!(count(&f, "records"), 0);
    assert_eq!(
        count(&f, "task_drivers"),
        1,
        "CA4c re-claimed/replayed activation"
    );
    f.runtime.shutdown().await.unwrap();
    finish(f).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca4d_contained_fault_recovers_in_same_live_worker() {
    for panic in [false, true] {
        let mut f = fixture("claude", true);
        f.register_real_git_project();
        let (_, tasks) = accept(&f, 1).await;
        let (park, _open) = service_park(&f);
        let deferred = Arc::new(AtomicBool::new(false));
        let resume = Arc::new(tokio::sync::Semaphore::new(0));
        let arrived = deferred.clone();
        let permits = resume.clone();
        let seams = Arc::new(ActivationSeams {
            postcommit: Some(Arc::new(move |_| {
                if panic {
                    panic!("CA4d injected contained panic");
                }
                anyhow::bail!("CA4d injected contained failure")
            })),
            deferred: Some(Arc::new(move |_| {
                arrived.store(true, Ordering::SeqCst);
                let permits = permits.clone();
                Box::pin(async move {
                    permits.acquire().await.unwrap().forget();
                    Ok(())
                })
            })),
            ..Default::default()
        });
        let (observations, release) = install_pause(&f, Some(seams));
        f.runtime.start().await.unwrap();
        wait_for(
            || observations.lock().unwrap().len() == 1,
            "SETUP: S1 absent",
        )
        .await;
        let original = observations.lock().unwrap()[0].clone();
        park.close();
        f.runtime.wake.notify_one();
        wait_for(|| park.parked(), "SETUP: service S5 absent").await;
        release.add_permits(1);
        wait_for(
            || deferred.load(Ordering::SeqCst),
            "CA4d failed to return Deferred",
        )
        .await;
        assert!(f.owner.store.lock().is_ok(), "CA4d SharedStore poisoned");
        assert_eq!(
            f.runtime._drivers.observe_finished().unwrap(),
            1,
            "CA4d SAME worker exited"
        );
        assert!(f.runtime._drivers.pending_exits().unwrap().is_empty());
        assert!(
            original.plan.is_retained().unwrap(),
            "CA4d lost SAME retained plan"
        );
        assert_eq!(
            original.association.binding().unwrap().2,
            original.plan.original_binding().2,
            "CA4d cache published before worker recovery"
        );
        assert_contract(&f, &original, "CA4d committed Deferred");
        resume.add_permits(1);
        let (_, workflow) = wait_bound(&f, &tasks[0]).await;
        assert!(
            !original.plan.is_retained().unwrap(),
            "CA4d worker failed to publish SAME plan"
        );
        assert_eq!(
            count(&f, "workflow_native_contracts"),
            1,
            "CA4d duplicate INSERT"
        );
        assert_contract(&f, &original, "CA4d recovered");
        assert_eq!(f.runtime._drivers.retained_preparations().unwrap(), 0);
        park.open();
        assert_eq!(
            f.runtime.observe_task_drivers().unwrap(),
            1,
            "CA4d service found a retained preparation"
        );
        release_peer(&f, &workflow);
        finish(f).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca4g_actual_exited_worker_keeps_same_plan_held() {
    for panic in [false, true] {
        let mut f = fixture("claude", true);
        f.register_real_git_project();
        accept(&f, 1).await;
        let (park, _open) = service_park(&f);
        let seams = Arc::new(ActivationSeams {
            postcommit: Some(Arc::new(|_| {
                anyhow::bail!("CA4g injected postcommit failure")
            })),
            deferred: Some(Arc::new(move |_| {
                Box::pin(async move {
                    if panic {
                        panic!("CA4g actual deferred worker panic");
                    }
                    anyhow::bail!("CA4g actual deferred worker exit")
                })
            })),
            ..Default::default()
        });
        let (observations, release) = install_pause(&f, Some(seams));
        f.runtime.start().await.unwrap();
        wait_for(
            || observations.lock().unwrap().len() == 1,
            "SETUP: S1 absent",
        )
        .await;
        let original = observations.lock().unwrap()[0].clone();
        park.close();
        f.runtime.wake.notify_one();
        wait_for(|| park.parked(), "SETUP: service S5 absent").await;
        release.add_permits(1);
        wait_exited(&f, 1).await;
        let exits = f.runtime._drivers.pending_exits().unwrap();
        assert_eq!(exits[0].label(), if panic { "panicked" } else { "failed" });
        assert!(f.owner.store.lock().is_ok(), "CA4g SharedStore poisoned");
        assert!(
            !f.owner
                .store
                .lock()
                .unwrap()
                .reconcile_driver_preparation(&original.plan)
                .unwrap(),
            "CA4g revoked worker cache resurrected"
        );
        assert!(
            original.plan.is_retained().unwrap(),
            "CA4g SAME Held plan lost"
        );
        assert_eq!(
            original.association.binding().unwrap().2,
            original.plan.original_binding().2,
            "CA4g revoked cache advanced"
        );
        assert_contract(&f, &original, "CA4g Held");
        park.open();
        assert!(
            f.runtime.observe_task_drivers().unwrap() >= 1,
            "CA4g Held plan omitted from pending count"
        );
        assert_eq!(f.runtime._drivers.observe_finished().unwrap(), 0);
        let error = f.runtime.shutdown().await.unwrap_err();
        assert!(
            error
                .to_string()
                .contains("Task Driver shutdown remains pending"),
            "{error:#}"
        );
        eprintln!("CA4g recorded shutdown pending: {error:#}");
        finish(f).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca4h_precommit_panic_stays_poisoned_and_rolls_back() {
    let mut f = fixture("claude", true);
    f.register_real_git_project();
    accept(&f, 1).await;
    let (park, _open) = service_park(&f);
    let seams = Arc::new(ActivationSeams {
        precommit: Some(Arc::new(|_| panic!("CA4h uncontained precommit panic"))),
        ..Default::default()
    });
    let (observations, release) = install_pause(&f, Some(seams));
    f.runtime.start().await.unwrap();
    wait_for(
        || observations.lock().unwrap().len() == 1,
        "SETUP: S1 absent",
    )
    .await;
    let original = observations.lock().unwrap()[0].clone();
    park.close();
    f.runtime.wake.notify_one();
    wait_for(|| park.parked(), "SETUP: service S5 absent").await;
    release.add_permits(1);
    wait_exited(&f, 1).await;
    assert_eq!(
        f.runtime._drivers.pending_exits().unwrap()[0].label(),
        "panicked"
    );
    assert!(
        f.owner.store.lock().is_err(),
        "CA4h uncontained panic did not poison SharedStore"
    );
    let readonly = rusqlite::Connection::open_with_flags(
        f.owner.state_path(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    for table in ["records", "workflow_native_contracts"] {
        assert_eq!(
            readonly
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                    .get::<_, u64>(0))
                .unwrap(),
            0,
            "CA4h rolled-back transaction left {table}"
        );
    }
    assert!(original.plan.is_retained().unwrap());
    park.open();
    assert!(
        f.runtime
            .observe_task_drivers()
            .unwrap_err()
            .to_string()
            .contains("state poisoned"),
        "CA4h service recovered poison"
    );
    assert!(
        f.runtime
            .shutdown()
            .await
            .unwrap_err()
            .to_string()
            .contains("state poisoned")
    );
    finish(f).await;
}
