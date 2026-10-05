// Included in session's existing tests: actual private registration/actor paths.
mod custody_mechanics {
    use super::super::super::custody::{Factory, FileSpec, Inventory, Pool};
    use super::*;
    use futures_util::FutureExt;
    use std::time::{Duration, Instant};

    struct Release(Option<std::sync::mpsc::Sender<()>>);
    impl Release {
        fn release(&mut self) {
            if let Some(send) = self.0.take() {
                let _ = send.send(());
            }
        }
    }
    impl Drop for Release {
        fn drop(&mut self) {
            self.release();
        }
    }
    struct RetirePause(Arc<Inventory>);
    impl Drop for RetirePause {
        fn drop(&mut self) {
            self.0.pause_retirement(false);
            self.0.pause(false);
        }
    }
    fn spec(
        path: PathBuf,
        panic: bool,
        panic_actor: bool,
        panic_before_begin: bool,
    ) -> (FileSpec, Release) {
        let (send, release) = std::sync::mpsc::channel();
        (
            FileSpec {
                path,
                started: Arc::new(AtomicBool::new(false)),
                release,
                panic,
                panic_actor,
                panic_before_begin,
            },
            Release(Some(send)),
        )
    }
    fn adapter() -> (Fixture, Arc<CodexAdapter>, Arc<Factory>) {
        let owned = Fixture::new(false);
        let mut adapter = CodexAdapter::new(
            "codex".into(),
            "/no-native-custody-fixture".into(),
            owned.store.clone(),
        )
        .unwrap()
        .component_fixture();
        let factory = Factory::new(Pool::isolated());
        adapter.file_factory = Some(factory.clone());
        (owned, Arc::new(adapter), factory)
    }
    async fn wait(mut predicate: impl FnMut() -> bool) {
        let limit = Instant::now() + Duration::from_secs(2);
        while !predicate() {
            assert!(
                Instant::now() < limit,
                "owned mechanical observation did not arrive"
            );
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
    async fn joined(pool: &Pool, control: &Control) {
        wait(|| {
            pool.drain();
            control.custody().unwrap().joined()
        })
        .await;
        assert_eq!(pool.used(), 0);
    }
    fn closed_runtime_observation<T: Send + 'static>(
        operation: impl FnOnce() -> T + Send + 'static,
    ) -> T {
        let (sent, received) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            let value = operation();
            sent.send(value).unwrap();
        });
        match received.recv_timeout(Duration::from_secs(2)) {
            Ok(value) => {
                worker.join().unwrap();
                value
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                match worker.join() {
                    Err(payload) => std::panic::resume_unwind(payload),
                    Ok(()) => panic!("closed-runtime helper returned no observation"),
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                panic!("closed scheduler cannot deadlock custodied task installation")
            }
        }
    }

    #[tokio::test]
    async fn actual_err_return_retains_worker_then_join_reconciles_fresh_entry() {
        let (owned, adapter, factory) = adapter();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("owned-file");
        let (file, mut release) = spec(path.clone(), false, false, false);
        let started = file.started.clone();
        factory.enqueue(file);
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        let id = registered.transition.id;
        let control = registered.transition.control.clone();
        let error = adapter.spawn_launch(registered).await.unwrap_err();
        assert_eq!(error.message, "injected closed fixture context refusal");
        assert!(matches!(
            control.wait_finished().await.unwrap(),
            Outcome::CustodyHeld { .. }
        ));
        wait(|| started.load(Ordering::SeqCst)).await;
        assert_eq!(std::fs::read(&path).unwrap(), b"owned");
        assert_eq!(factory.pool.used(), 3);
        assert!(adapter.registry().unwrap().contains_key(&id));
        assert!(
            adapter
                .registry()
                .unwrap()
                .get(&id)
                .unwrap()
                .transition
                .load(Ordering::SeqCst)
        );
        assert_eq!(
            adapter.availability.sites(),
            [0; 10],
            "factory refusal cannot reach Git/native/Store preflight"
        );
        // Returning Err disarmed CallerGuard. The SAME non-Clone endpoint still
        // permits its fixed mechanical callback while the worker is owned.
        wait(|| control.notes() > 0).await;
        let notes = control.notes();
        let mut endpoint = control.take_endpoint().unwrap();
        endpoint.try_note("after Err return").unwrap();
        wait(|| control.notes() > notes).await;
        endpoint.corrupt_generation();
        assert!(endpoint.try_note("foreign").is_err());
        let stop = adapter
            .stop(SessionRef {
                id,
                scope: owned.request.scope.clone(),
            })
            .await
            .unwrap_err();
        assert_eq!(stop.kind, ErrorKind::StateConflict);
        assert!(control.jobs_revoked());
        assert!(matches!(
            control.wait_finished().await.unwrap(),
            Outcome::CustodyHeld { .. }
        ));
        release.release();
        joined(&factory.pool, &control).await;
        assert!(matches!(
            control.wait_finished().await.unwrap(),
            Outcome::CustodyHeld { .. }
        ));
        adapter.reconcile_custody().unwrap();
        assert!(!adapter.registry().unwrap().contains_key(&id));
        assert!(owned.store.lock().unwrap().session(id).unwrap().is_none());
    }

    #[tokio::test]
    async fn same_adapter_more_than32_joined_workers_replenish_registry_and_pool() {
        let (owned, adapter, factory) = adapter();
        let directory = tempfile::tempdir().unwrap();
        for index in 0..40 {
            let (file, mut release) = spec(
                directory.path().join(format!("worker-{index}")),
                false,
                false,
                false,
            );
            factory.enqueue(file);
            let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
            let control = registered.transition.control.clone();
            adapter.spawn_launch(registered).await.unwrap_err();
            assert!(matches!(
                control.wait_finished().await.unwrap(),
                Outcome::CustodyHeld { .. }
            ));
            release.release();
            joined(&factory.pool, &control).await;
            // The NEXT real register_fresh must sweep every previous retained
            // intent; calling reconcile directly here would mask that mutant.
        }
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        registered.refused_before_work(failure(ErrorKind::StateConflict, "fixture ended"));
        drop(registered);
        assert!(adapter.registry().unwrap().is_empty());
        assert_eq!(factory.pool.used(), 0);
    }

    #[tokio::test]
    async fn caller_drop_before_factory_releases_not_created_slot_and_registry() {
        let (owned, adapter, factory) = adapter();
        let directory = tempfile::tempdir().unwrap();
        for index in 0..40 {
            let path = directory.path().join(format!("never-created-{index}"));
            let (file, _release) = spec(path.clone(), false, false, false);
            factory.enqueue(file);
            let gate = adapter.gates.install(TestPoint::BeforeCustodyFactory);
            let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
            let control = registered.transition.control.clone();
            let mut calling = Box::pin(adapter.spawn_launch(registered));
            tokio::select! { _ = gate.reached() => {}, value = &mut calling => panic!("caller returned before barrier: {value:?}") }
            assert_eq!(factory.pool.used(), 3);
            drop(calling);
            assert!(control.jobs_revoked());
            gate.release();
            assert!(matches!(
                control.wait_finished().await.unwrap(),
                Outcome::FreshUnpublished { .. }
            ));
            joined(&factory.pool, &control).await;
            assert!(!path.exists());
            assert!(adapter.registry().unwrap().is_empty());
            assert_eq!(factory.pool.used(), 0);
        }
    }

    #[tokio::test]
    async fn full_revoked_inbox_cannot_drop_join_cleanup_and_encoded_bounds_are_exact() {
        let (owned, adapter, factory) = adapter();
        let directory = tempfile::tempdir().unwrap();
        let (file, mut release) = spec(directory.path().join("worker"), false, false, false);
        factory.enqueue(file);
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        let control = registered.transition.control.clone();
        adapter.spawn_launch(registered).await.unwrap_err();
        let custody = control.custody().unwrap();
        wait(|| custody.accepted() == 0).await;
        custody.pause(true);
        let endpoint = control.take_endpoint().unwrap();
        let overhead = serde_json::to_vec(&json!({"kind":"note","text":""}))
            .unwrap()
            .len();
        let exact = "x".repeat(4096 - overhead);
        endpoint.try_note(&exact).unwrap();
        assert_eq!(
            endpoint.try_note(&(exact.clone() + "x")).unwrap_err().kind,
            ErrorKind::InvalidInput
        );
        // Escaping and UTF8 count actual encoded bytes, not source char count.
        assert_eq!(
            endpoint.try_note(&"\n".repeat(3000)).unwrap_err().kind,
            ErrorKind::InvalidInput
        );
        for _ in 1..64 {
            endpoint.try_note("bounded").unwrap();
        }
        assert!(endpoint.try_note("over capacity").is_err());
        assert_eq!(custody.accepted(), 64);
        control.revoke_jobs();
        let notes = control.notes();
        release.release();
        wait(|| custody.worker_joined()).await;
        assert_eq!(
            custody.accepted(),
            64,
            "cleanup observed independently of full inbox"
        );
        custody.pause(false);
        joined(&factory.pool, &control).await;
        assert_eq!(
            control.notes(),
            notes,
            "queued NEW effects must recheck revocation"
        );
        assert_eq!(custody.accepted(), 0);
        assert!(!custody.unknown());
    }

    #[tokio::test]
    async fn actor_and_callback_panics_retain_actual_resources_and_unknown_accounting() {
        for (actor_panic, callback_panic, worker_panic) in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            let (owned, adapter, factory) = adapter();
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("worker");
            let (file, mut release) = spec(path.clone(), worker_panic, actor_panic, callback_panic);
            factory.enqueue(file);
            let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
            let id = registered.transition.id;
            let control = registered.transition.control.clone();
            adapter.spawn_launch(registered).await.unwrap_err();
            let custody = control.custody().unwrap();
            assert!(custody.created());
            assert!(adapter.registry().unwrap().contains_key(&id));
            release.release();
            wait(|| {
                factory.pool.drain();
                custody.unknown() && factory.pool.used() == 1
            })
            .await;
            assert!(!custody.joined());
            adapter.reconcile_custody().unwrap();
            assert!(adapter.registry().unwrap().contains_key(&id));
            if callback_panic {
                assert!(!path.exists(), "worker cannot open before installed Begin");
            } else {
                assert!(path.exists());
            }
            assert!(control.original_disposition().is_none());
        }
    }

    #[test]
    fn custodied_missing_and_closed_runtime_have_exact_no_effect_accounting() {
        let (missing, _) = Control::new(None);
        let empty = Pool::isolated();
        assert_eq!(
            missing.install_custody(empty.clone()).unwrap_err().kind,
            ErrorKind::LaunchFailure
        );
        assert_eq!(empty.used(), 0);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let handle = runtime.handle().clone();
        drop(runtime);
        let (control, _) = Control::new(None);
        let pool = Pool::isolated();
        let owner = control.clone();
        let reserved = pool.clone();
        // This future contains only TaskGuard, so its revocation is independently
        // observed even when RegisteredTransition is absent.
        closed_runtime_observation(move || {
            let _entered = handle.enter();
            owner.install_custody(reserved).unwrap();
            let guard = TaskGuard(owner.clone());
            owner.spawn(async move {
                let _guard = guard;
                std::future::pending::<()>().await;
            })
        })
        .unwrap();
        assert!(control.jobs_revoked(), "TaskGuard must revoke before Lost");
        let deadline = Instant::now() + Duration::from_secs(2);
        while pool.used() != 1 {
            pool.drain();
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(control.custody().unwrap().unknown());
        assert!(!control.custody().unwrap().joined());
        assert!(
            matches!(&*control.subscribe().borrow(),Phase::Finished(outcome) if matches!(outcome.as_ref(), Outcome::Lost { .. }))
        );
    }

    #[tokio::test]
    async fn abandoned_actor_revokes_queued_creation_before_caller_error_return() {
        let (owned, adapter, factory) = adapter();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("late-worker-after-abandonment");
        let (file, mut release) = spec(path.clone(), false, false, false);
        factory.enqueue(file);
        let before = adapter.gates.install(TestPoint::BeforeCustodyFactory);
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        let id = registered.transition.id;
        let control = registered.transition.control.clone();
        let mut calling = Box::pin(adapter.spawn_launch(registered));
        tokio::select! { _ = before.reached() => {}, value = &mut calling => panic!("early return: {value:?}") }
        let custody = control.custody().unwrap();
        let pause = RetirePause(custody.clone());
        custody.pause(true);
        before.release();
        wait(|| custody.accepted() == 1).await;
        assert!(custody.outstanding_effects());
        assert!(!custody.created());
        control.abort_owned_task();
        let error = tokio::time::timeout(Duration::from_secs(2), calling)
            .await
            .expect("abandoned actor must return its caller error")
            .unwrap_err();
        assert!(
            control.jobs_revoked(),
            "caller error observation must already see abandoned jobs revoked"
        );
        assert_eq!(error.kind, ErrorKind::SessionLost);
        assert!(matches!(
            tokio::time::timeout(Duration::from_secs(2), control.wait_finished())
                .await
                .expect("abandoned actor must publish Lost")
                .unwrap(),
            Outcome::Lost { .. }
        ));
        // Let every actual created job finish before asserting the refusal.
        // The cancelled actor remains Unknown; its slot is never refunded.
        release.release();
        drop(pause);
        wait(|| {
            factory.pool.drain();
            factory.pool.used() == 1
        }).await;
        assert!(!path.exists(), "abandoned actor admitted a new queued file effect");
        assert!(!custody.created(), "abandoned queued worker must be NotCreated");
        assert!(control.jobs_revoked());
        assert!(custody.unknown());
        assert!(!custody.joined());
        assert!(!custody.outstanding());
        assert!(adapter.registry().unwrap().contains_key(&id));
        assert!(control.original_disposition().is_none());
        assert_eq!(adapter.availability.sites(), [0; 10]);
        assert_eq!(control.stop.capacity(), 1, "abandonment sends no native stop");
    }

    #[tokio::test]
    async fn checkpoint_err_worker_restores_only_genuine_previous_control_after_join() {
        let mut history = ApprovalFixture::new(false).await;
        let (mut adapter, reference) = history.terminal_adapter();
        let factory = Factory::new(Pool::isolated());
        Arc::get_mut(&mut adapter).unwrap().file_factory = Some(factory.clone());
        let previous = adapter
            .registry()
            .unwrap()
            .get(&reference.id)
            .unwrap()
            .control
            .clone();
        let directory = tempfile::tempdir().unwrap();
        let (file, mut release) = spec(directory.path().join("worker"), false, false, false);
        factory.enqueue(file);
        let registered = adapter.register_existing(&reference).unwrap();
        let control = registered.transition.control.clone();
        let input = registered.request.input.clone();
        adapter
            .spawn_checkpoint(registered, input)
            .await
            .unwrap_err();
        assert!(adapter.register_existing(&reference).is_err());
        assert!(Arc::ptr_eq(
            &adapter
                .registry()
                .unwrap()
                .get(&reference.id)
                .unwrap()
                .control,
            &control
        ));
        release.release();
        joined(&factory.pool, &control).await;
        adapter.reconcile_custody().unwrap();
        assert!(Arc::ptr_eq(
            &adapter
                .registry()
                .unwrap()
                .get(&reference.id)
                .unwrap()
                .control,
            &previous
        ));
        assert!(
            !adapter
                .registry()
                .unwrap()
                .get(&reference.id)
                .unwrap()
                .transition
                .load(Ordering::SeqCst)
        );
        let next = adapter.register_existing(&reference).unwrap();
        next.refused_before_work(failure(ErrorKind::StateConflict, "fixture done"));
        drop(next);
        assert!(Arc::ptr_eq(
            &adapter
                .registry()
                .unwrap()
                .get(&reference.id)
                .unwrap()
                .control,
            &previous
        ));
    }
    #[tokio::test]
    async fn actual_caller_drop_after_creation_revokes_without_losing_owned_worker() {
        let (owned, adapter, factory) = adapter();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("worker");
        let (file, mut release) = spec(path.clone(), false, false, false);
        let started = file.started.clone();
        factory.enqueue(file);
        let gate = adapter.gates.install(TestPoint::AfterCustodyFactory);
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        let id = registered.transition.id;
        let control = registered.transition.control.clone();
        let mut calling = Box::pin(adapter.spawn_launch(registered));
        tokio::select! { _ = gate.reached() => {}, value = &mut calling => panic!("early return: {value:?}") }
        wait(|| started.load(Ordering::SeqCst)).await;
        let endpoint = control.take_endpoint().unwrap();
        drop(calling);
        assert!(control.jobs_revoked());
        assert!(endpoint.try_note("after caller Drop").is_err());
        assert_eq!(factory.pool.used(), 3);
        assert!(path.exists());
        gate.release();
        assert!(matches!(
            control.wait_finished().await.unwrap(),
            Outcome::CustodyHeld {
                cause: Cause::Cancelled,
                ..
            }
        ));
        assert!(adapter.registry().unwrap().contains_key(&id));
        release.release();
        joined(&factory.pool, &control).await;
        adapter.reconcile_custody().unwrap();
        assert!(adapter.registry().unwrap().is_empty());
    }

    #[tokio::test]
    async fn endpoint_close_before_worker_join_is_factual_not_unknown() {
        let (owned, adapter, factory) = adapter();
        let directory = tempfile::tempdir().unwrap();
        let (file, mut release) = spec(directory.path().join("worker"), false, false, false);
        factory.enqueue(file);
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        let control = registered.transition.control.clone();
        adapter.spawn_launch(registered).await.unwrap_err();
        let custody = control.custody().unwrap();
        drop(control.take_endpoint().unwrap());
        assert!(custody.endpoint_closed());
        assert!(!custody.unknown());
        assert!(!custody.worker_joined());
        release.release();
        joined(&factory.pool, &control).await;
        assert!(!custody.unknown());
    }

    #[tokio::test]
    async fn armed_caller_drop_revokes_consumed_and_failing_endpoint_without_new_native_stop() {
        for consumed in [true, false] {
            let (owned, adapter, factory) = adapter();
            let directory = tempfile::tempdir().unwrap();
            let (file, mut release) = spec(directory.path().join("worker"), false, false, false);
            factory.enqueue(file);
            let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
            let control = registered.transition.control.clone();
            if consumed {
                // This is a preparation-latch mechanics fixture, not native
                // admission: no Session/wire/managed grant is manufactured.
                control.preparation.consume(|| Ok(())).unwrap();
            } else {
                control
                    .preparation
                    .failed(failure(ErrorKind::StateConflict, "fixed first cause"));
            }
            adapter.spawn_launch(registered).await.unwrap_err();
            let endpoint = control.take_endpoint().unwrap();
            drop(CallerGuard::new(control.clone()));
            assert!(control.jobs_revoked());
            assert!(endpoint.try_note("new callback").is_err());
            assert_eq!(control.consumed(), consumed);
            if !consumed {
                assert_eq!(
                    control.preparation.check().unwrap_err().message,
                    "fixed first cause"
                );
            }
            assert_eq!(
                control.stop.capacity(),
                1,
                "job revocation queued no native stop message"
            );
            assert_eq!(adapter.availability.sites(), [0; 10]);
            release.release();
            joined(&factory.pool, &control).await;
            adapter.reconcile_custody().unwrap();
        }
    }

    #[tokio::test]
    async fn opaque_custodian_creation_error_retains_complete_reservation_without_effect() {
        let (owned, adapter, factory) = adapter();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("never-created");
        let (file, _release) = spec(path.clone(), false, false, false);
        factory.enqueue(file);
        factory.pool.fail_creator_once();
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        let error = adapter.spawn_launch(registered).await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::SessionLost);
        assert_eq!(error.message, "preparation custodian creation unverified");
        assert_eq!(factory.pool.used(), 3);
        factory.pool.drain();
        assert_eq!(
            factory.pool.used(),
            3,
            "opaque Err cannot refund unproved frames"
        );
        assert!(adapter.registry().unwrap().is_empty());
        assert!(!path.exists());
        assert_eq!(adapter.availability.sites(), [0; 10]);
    }

    #[tokio::test]
    async fn actor_panic_keeps_late_worker_join_observer_until_cleanup() {
        let (owned, adapter, factory) = adapter();
        let directory = tempfile::tempdir().unwrap();
        let (file, mut release) = spec(directory.path().join("worker"), false, true, false);
        let started = file.started.clone();
        factory.enqueue(file);
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        let control = registered.transition.control.clone();
        adapter.spawn_launch(registered).await.unwrap_err();
        let custody = control.custody().unwrap();
        wait(|| custody.unknown() && started.load(Ordering::SeqCst)).await;
        factory.pool.drain();
        assert_eq!(
            factory.pool.used(),
            3,
            "unknown actor cannot discard a pending worker observer"
        );
        assert!(!custody.worker_joined());
        release.release();
        wait(|| {
            factory.pool.drain();
            factory.pool.used() == 1
        })
        .await;
        assert!(custody.worker_joined());
        assert!(custody.unknown());
        assert!(!custody.joined());
        assert!(control.original_disposition().is_none());
    }

    #[tokio::test]
    async fn worker_join_before_error_preserves_zero_effect_registry_disposition() {
        let (owned, adapter, factory) = adapter();
        let directory = tempfile::tempdir().unwrap();
        let (file, mut release) = spec(directory.path().join("worker"), false, false, false);
        factory.enqueue(file);
        let gate = adapter.gates.install(TestPoint::AfterCustodyFactory);
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        let control = registered.transition.control.clone();
        let mut calling = Box::pin(adapter.spawn_launch(registered));
        tokio::select! { _ = gate.reached() => {}, value = &mut calling => panic!("early return: {value:?}") }
        let custody = control.custody().unwrap();
        release.release();
        wait(|| custody.worker_joined()).await;
        assert!(!custody.outstanding());
        assert!(
            !custody.joined(),
            "actor is still at its actual error-arm barrier"
        );
        gate.release();
        calling.await.unwrap_err();
        joined(&factory.pool, &control).await;
        assert!(matches!(
            control.wait_finished().await.unwrap(),
            Outcome::FreshUnpublished { .. }
        ));
        assert!(
            adapter.registry().unwrap().is_empty(),
            "bookkeeping-only joins cannot strand original no-work removal"
        );
        assert_eq!(adapter.availability.sites(), [0; 10]);
    }

    async fn revoked_create_with_note(note_pending: bool) {
        let (owned, adapter, factory) = adapter();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("never-created");
        let (file, _release) = spec(path.clone(), false, false, false);
        factory.enqueue(file);
        let before = adapter.gates.install(TestPoint::BeforeCustodyFactory);
        let after = adapter.gates.install(TestPoint::AfterCustodyFactory);
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        let control = registered.transition.control.clone();
        let mut calling = Box::pin(adapter.spawn_launch(registered));
        tokio::select! { _ = before.reached() => {}, value = &mut calling => panic!("early return: {value:?}") }
        let custody = control.custody().unwrap();
        let pause = RetirePause(custody.clone());
        custody.pause(true);
        custody.pause_retirement(true);
        custody.retirement_target(if note_pending { 2 } else { 1 });
        before.release();
        wait(|| custody.accepted() == 1).await;
        assert!(custody.outstanding_effects());
        assert!(control.holds_resources());
        if note_pending {
            control
                .take_endpoint()
                .unwrap()
                .try_note("metadata queued before revoke")
                .unwrap();
            assert_eq!(custody.accepted(), 2);
        }
        drop(calling);
        assert!(control.jobs_revoked());
        custody.pause(false);
        wait(|| custody.retirement_reached()).await;
        // This logical fixture bound lets an incorrectly early reply reach its
        // actual error arm while retirement is held. It is not OS/native proof.
        let early_reply = tokio::time::timeout(Duration::from_secs(2), after.reached())
            .await
            .is_ok();
        assert!(!note_pending || early_reply, "actual queued Note must overlap the actor error arm");
        if early_reply {
            after.release();
            let outcome = control.wait_finished().await.unwrap();
            if note_pending {
                assert!(matches!(outcome, Outcome::FreshUnpublished { .. }));
                wait(|| adapter.registry().unwrap().is_empty()).await;
                // No-work registry bookkeeping is independent of actual job
                // custody: the blocked custodian cannot have joined/refunded.
                assert_eq!(factory.pool.used(), 3);
                assert!(!custody.joined());
                assert!(custody.outstanding());
                assert!(!custody.outstanding_effects());
                assert!(control.original_disposition().is_none());
                assert!(!matches!(control.preparation.state().unwrap(), Admission::Consumed));
            }
        }
        drop(pause);
        after.reached().await;
        after.release();
        let outcome = control.wait_finished().await.unwrap();
        joined(&factory.pool, &control).await;
        adapter.reconcile_custody().unwrap();
        assert!(!custody.created());
        assert!(!path.exists());
        assert_eq!(adapter.availability.sites(), [0; 10]);
        assert!(matches!(outcome, Outcome::FreshUnpublished { .. }));
        assert!(
            adapter.registry().unwrap().is_empty(),
            "zero-effect reply cannot strand a created=false Held entry"
        );
    }

    #[tokio::test]
    async fn revoked_create_response_is_pending_until_own_request_retires() {
        // Poll the actual Factory/Create response directly. The retirement gate
        // establishes that the handler reached the cut; no actor scheduling
        // window or implementation-side "reply was early" flag is an oracle.
        let (control, _) = Control::new(None);
        let pool = Pool::isolated();
        control.install_custody(pool.clone()).unwrap();
        let custody = control.custody().unwrap();
        let (end_actor, actor_ended) = tokio::sync::oneshot::channel();
        control
            .spawn(async move {
                let _ = actor_ended.await;
            })
            .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("never-created");
        let (file, _release) = spec(path.clone(), false, false, false);
        let factory = Factory::new(pool.clone());
        factory.enqueue(file);
        let pause = RetirePause(custody.clone());
        custody.pause(true);
        custody.pause_retirement(true);
        let mut response = Box::pin(factory.run(&control));
        assert!(response.as_mut().now_or_never().is_none());
        assert_eq!(custody.accepted(), 1);
        assert!(custody.outstanding_effects());
        drop(CallerGuard::new(control.clone()));
        assert!(control.jobs_revoked());
        custody.pause(false);
        wait(|| custody.retirement_reached()).await;
        // An early-send mutant has already published the real oneshot result
        // before reaching this gate. Direct polling cannot miss that result
        // merely because an adapter actor has not yet been scheduled.
        assert!(
            response.as_mut().now_or_never().is_none(),
            "Create response published before its own request retired"
        );
        assert_eq!(custody.accepted(), 1);
        assert!(custody.outstanding_effects());
        drop(pause);
        let error = tokio::time::timeout(Duration::from_secs(2), response)
            .await
            .expect("retired Create must deliver its actual response");
        assert_eq!(error.kind, ErrorKind::StateConflict);
        let _ = end_actor.send(());
        joined(&pool, &control).await;
        assert!(!custody.created());
        assert!(!path.exists());
        // Joined mechanical frames do not supply registry/native/settlement
        // authority; the existing adapter error-arm controls cover that layer.
    }

    #[tokio::test]
    async fn queued_revoked_create_retires_before_actor_error_and_removes_no_work_entry() {
        revoked_create_with_note(false).await;
    }

    #[tokio::test]
    async fn queued_metadata_after_revoked_create_preserves_no_work_registry_disposition() {
        revoked_create_with_note(true).await;
    }

    #[tokio::test]
    async fn queued_metadata_after_revoked_checkpoint_restores_exact_previous_without_job_refund() {
        let mut history = ApprovalFixture::new(false).await;
        let (mut adapter, reference) = history.terminal_adapter();
        let factory = Factory::new(Pool::isolated());
        Arc::get_mut(&mut adapter).unwrap().file_factory = Some(factory.clone());
        let previous = adapter.registry().unwrap()[&reference.id].control.clone();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("never-created");
        let (file, _release) = spec(path.clone(), false, false, false);
        factory.enqueue(file);
        let before = adapter.gates.install(TestPoint::BeforeCustodyFactory);
        let after = adapter.gates.install(TestPoint::AfterCustodyFactory);
        let registered = adapter.register_existing(&reference).unwrap();
        let control = registered.transition.control.clone();
        let input = registered.request.input.clone();
        let mut calling = Box::pin(adapter.spawn_checkpoint(registered, input));
        tokio::select! { _ = before.reached() => {}, value = &mut calling => panic!("early return: {value:?}") }
        let custody = control.custody().unwrap();
        let pause = RetirePause(custody.clone());
        custody.pause(true);
        custody.pause_retirement(true);
        custody.retirement_target(2);
        before.release();
        wait(|| custody.accepted() == 1).await;
        control.take_endpoint().unwrap().try_note("checkpoint metadata").unwrap();
        drop(calling);
        custody.pause(false);
        wait(|| custody.retirement_reached()).await;
        after.reached().await;
        after.release();
        assert!(matches!(control.wait_finished().await.unwrap(), Outcome::RestoredBeforeAdmission { .. }));
        wait(|| Arc::ptr_eq(&adapter.registry().unwrap()[&reference.id].control, &previous)).await;
        assert_eq!(factory.pool.used(), 3);
        assert!(!custody.joined());
        assert!(custody.outstanding());
        assert!(!custody.outstanding_effects());
        assert!(control.original_disposition().is_none());
        assert_eq!(adapter.availability.sites(), [0; 10]);
        assert!(!path.exists());
        drop(pause);
        joined(&factory.pool, &control).await;
        assert!(Arc::ptr_eq(&adapter.registry().unwrap()[&reference.id].control, &previous));
    }

    #[tokio::test]
    async fn complete_job_capacity_refuses_before_factory_and_recovers_after_actual_joins() {
        let (owned, adapter, factory) = adapter();
        let directory = tempfile::tempdir().unwrap();
        let mut held = Vec::new();
        for index in 0..21 {
            let (file, release) = spec(
                directory.path().join(format!("worker-{index}")),
                false,
                false,
                false,
            );
            factory.enqueue(file);
            let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
            let control = registered.transition.control.clone();
            adapter.spawn_launch(registered).await.unwrap_err();
            held.push((control, release));
        }
        assert_eq!(factory.pool.used(), 63);
        let path = directory.path().join("capacity-refused");
        let (file, mut release) = spec(path.clone(), false, false, false);
        factory.enqueue(file);
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        let id = registered.transition.id;
        let error = adapter.spawn_launch(registered).await.unwrap_err();
        assert_eq!(error.kind, ErrorKind::StateConflict);
        assert_eq!(error.message, "preparation job capacity exhausted");
        assert_eq!(factory.pool.used(), 63);
        assert!(!path.exists());
        assert!(!adapter.registry().unwrap().contains_key(&id));
        for (_, release) in &mut held {
            release.release();
        }
        for (control, _) in held {
            wait(|| {
                factory.pool.drain();
                control.custody().unwrap().joined()
            })
            .await;
        }
        assert_eq!(factory.pool.used(), 0);
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        let control = registered.transition.control.clone();
        adapter.spawn_launch(registered).await.unwrap_err();
        release.release();
        joined(&factory.pool, &control).await;
        assert!(path.exists());
        adapter.reconcile_custody().unwrap();
        assert!(adapter.registry().unwrap().is_empty());
    }

    #[test]
    fn actual_registration_missing_and_closed_runtime_have_no_native_effects() {
        let (owned, adapter, factory) = adapter();
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        let error = adapter
            .spawn_launch(registered)
            .now_or_never()
            .unwrap()
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::LaunchFailure);
        assert_eq!(factory.pool.used(), 0);
        assert!(adapter.registry().unwrap().is_empty());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let handle = runtime.handle().clone();
        drop(runtime);
        let registered = adapter.register_fresh(owned.request.clone(), None).unwrap();
        let control = registered.transition.control.clone();
        let caller = adapter.clone();
        let error = closed_runtime_observation(move || {
            let _entered = handle.enter();
            caller
                .spawn_launch(registered)
                .now_or_never()
                .unwrap()
                .unwrap_err()
        });
        assert_eq!(error.kind, ErrorKind::SessionLost);
        assert!(adapter.registry().unwrap().is_empty());
        let deadline = Instant::now() + Duration::from_secs(2);
        while factory.pool.used() != 1 {
            factory.pool.drain();
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(control.custody().unwrap().unknown());
        assert_eq!(adapter.availability.sites(), [0; 10]);
    }
}
