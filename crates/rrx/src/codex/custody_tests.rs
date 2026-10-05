// Included in session's existing tests: actual private registration/actor paths.
mod custody_mechanics {
    use super::super::super::custody::{Factory, FileSpec, Pool};
    use super::*;
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
        let _entered = handle.enter();
        let (control, _) = Control::new(None);
        let pool = Pool::isolated();
        control.install_custody(pool.clone()).unwrap();
        let guard = TaskGuard(control.clone());
        control
            .spawn(async move {
                let _guard = guard;
                std::future::pending::<()>().await;
            })
            .unwrap();
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
}
