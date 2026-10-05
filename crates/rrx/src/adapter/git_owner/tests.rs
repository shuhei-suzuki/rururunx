use super::*;

async fn shell(
    context: TestGitContext,
    script: &str,
    flag: Arc<AtomicBool>,
) -> AdapterResult<Vec<u8>> {
    run(
        Path::new("/bin/sh"),
        Path::new("/tmp"),
        &["-c".into(), script.into()],
        vec![],
        tokio::time::Instant::now() + Duration::from_secs(5),
        flag,
        Some(context),
    )
    .await
}
async fn released(context: &TestGitContext) {
    let pool = context.pool.as_ref().unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let waiter = pool.available.notified();
            tokio::pin!(waiter);
            waiter.as_mut().enable();
            if pool.state.lock().unwrap().records.is_empty() {
                return;
            }
            waiter.await;
        }
    })
    .await
    .unwrap();
}
#[tokio::test]
async fn binary_output_and_settled_error_clear_live_uncertainty() {
    let context = TestGitContext::isolated();
    let flag = Arc::new(AtomicBool::new(false));
    assert_eq!(
        shell(context.clone(), "printf 'a\\000 b\\n'", flag.clone())
            .await
            .unwrap(),
        b"a\0 b\n"
    );
    assert!(!flag.load(Ordering::SeqCst));
    assert_eq!(
        shell(context.clone(), "exit 7", flag.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::OwnershipMismatch
    );
    assert!(!flag.load(Ordering::SeqCst));
    released(&context).await;
}
#[tokio::test]
async fn expired_refusal_precedes_capacity_and_has_no_job_or_flag() {
    let context = TestGitContext::isolated();
    let flag = Arc::new(AtomicBool::new(false));
    let result = run(
        Path::new("/nonexistent"),
        Path::new("/tmp"),
        &[],
        vec![],
        tokio::time::Instant::now(),
        flag.clone(),
        Some(context.clone()),
    )
    .await
    .unwrap_err();
    assert_eq!(result.kind, ErrorKind::Timeout);
    assert!(!flag.load(Ordering::SeqCst));
    released(&context).await;
}
#[tokio::test]
async fn actual_std_spawn_then_initialization_failure_is_settled() {
    let mut context = TestGitContext::isolated();
    context.context.hooks.initialized_error = true;
    let flag = Arc::new(AtomicBool::new(false));
    let failure = shell(context.clone(), "exec /bin/sleep 30", flag.clone())
        .await
        .unwrap_err();
    assert_eq!(failure.kind, ErrorKind::LaunchFailure);
    assert!(!flag.load(Ordering::SeqCst));
    released(&context).await;
}
#[tokio::test]
async fn opaque_attempted_spawn_error_retains_exact_four_slots() {
    let context = TestGitContext::isolated();
    let flag = Arc::new(AtomicBool::new(false));
    let failure = run(
        Path::new("/rrx-no-such-executable"),
        Path::new("/tmp"),
        &[],
        vec![],
        tokio::time::Instant::now() + Duration::from_secs(5),
        flag.clone(),
        Some(context.clone()),
    )
    .await
    .unwrap_err();
    assert_eq!(failure.kind, ErrorKind::ProcessFailure);
    assert!(flag.load(Ordering::SeqCst));
    let records = context.pool.as_ref().unwrap().state.lock().unwrap();
    assert_eq!(records.records.len() * JOBS, 4);
    assert!(records.records[0].native.lock().unwrap().child.is_none());
}

#[tokio::test]
async fn unchanged_scalar_contract_trims_metadata() {
    let flag = Arc::new(AtomicBool::new(false));
    let output = super::super::bounded_git(
        Path::new("/bin/sh"),
        Path::new("/tmp"),
        &["-c".into(), "printf ' a \n'".into()],
        vec![],
        tokio::time::Instant::now() + Duration::from_secs(5),
        flag.clone(),
    )
    .await
    .unwrap();
    assert_eq!(output, "a");
    assert!(!flag.load(Ordering::SeqCst));
}

#[tokio::test]
async fn cancelled_admitted_call_never_spawns_or_sets_flag() {
    let mut context = TestGitContext::isolated();
    let pause = Arc::new(TestPause::default());
    let release = TestRelease(pause.clone());
    context.context.hooks.before_authorize = Some(pause.clone());
    let flag = Arc::new(AtomicBool::new(false));
    let task = tokio::spawn(shell(context.clone(), "exit 0", flag.clone()));
    pause.reached().await;
    task.abort();
    let _ = task.await;
    assert!(!flag.load(Ordering::SeqCst));
    drop(release);
    released(&context).await;
    assert!(!flag.load(Ordering::SeqCst));
}
#[tokio::test]
async fn cancelled_live_call_does_not_drop_runtime_or_clear_its_flag() {
    let mut context = TestGitContext::isolated();
    let pause = Arc::new(TestPause::default());
    let release = TestRelease(pause.clone());
    context.context.hooks.after_spawn = Some(pause.clone());
    let flag = Arc::new(AtomicBool::new(false));
    let task = tokio::spawn(shell(context.clone(), "exec /bin/sleep 30", flag.clone()));
    pause.reached().await;
    task.abort();
    let _ = task.await;
    assert!(flag.load(Ordering::SeqCst));
    drop(release);
    released(&context).await;
    assert!(flag.load(Ordering::SeqCst));
}
#[tokio::test]
async fn actual_worker_join_precedes_success_and_capacity_release() {
    let mut context = TestGitContext::isolated();
    let pause = Arc::new(TestPause::default());
    let release = TestRelease(pause.clone());
    context.context.hooks.after_reap_send = Some(pause.clone());
    let flag = Arc::new(AtomicBool::new(false));
    let task = tokio::spawn(shell(context.clone(), "printf joined", flag.clone()));
    pause.reached().await;
    tokio::task::yield_now().await;
    let premature = task.is_finished();
    let held = context
        .pool
        .as_ref()
        .unwrap()
        .state
        .lock()
        .unwrap()
        .records
        .len()
        * JOBS;
    let uncertain = flag.load(Ordering::SeqCst);
    drop(release);
    assert_eq!(task.await.unwrap().unwrap(), b"joined");
    released(&context).await;
    assert!(!premature);
    assert_eq!(held, 4);
    assert!(uncertain);
    assert!(!flag.load(Ordering::SeqCst));
}
#[tokio::test]
async fn inner_frame_loss_wakes_caller_and_retains_outer_runtime_and_readers() {
    let mut context = TestGitContext::isolated();
    context.context.hooks.supervisor_panic = true;
    let flag = Arc::new(AtomicBool::new(false));
    let error = tokio::time::timeout(
        Duration::from_secs(3),
        shell(context.clone(), "exit 0", flag.clone()),
    )
    .await
    .unwrap()
    .unwrap_err();
    assert_eq!(error.kind, ErrorKind::SessionLost);
    assert!(flag.load(Ordering::SeqCst));
    let record = context.pool.as_ref().unwrap().state.lock().unwrap().records[0].clone();
    assert!(record.runtime.lock().unwrap_err().into_inner().is_some());
    let readers = record
        .readers
        .lock()
        .err()
        .expect("reader vault must survive unwind")
        .into_inner();
    assert!(readers.stdout.is_some() && readers.stderr.is_some());
}
#[tokio::test]
async fn established_primary_survives_later_inner_frame_loss() {
    let mut context = TestGitContext::isolated();
    context.context.hooks.after_primary_panic = true;
    let flag = Arc::new(AtomicBool::new(false));
    let error = shell(context.clone(), "exit 7", flag.clone())
        .await
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::OwnershipMismatch);
    assert!(flag.load(Ordering::SeqCst));
    assert_eq!(
        context
            .pool
            .as_ref()
            .unwrap()
            .state
            .lock()
            .unwrap()
            .records
            .len()
            * JOBS,
        4
    );
}
#[tokio::test]
async fn sixteen_actual_owner_records_bound_all_four_job_lanes() {
    let mut context = TestGitContext::isolated();
    let pause = Arc::new(TestPause::default());
    let release = TestRelease(pause.clone());
    context.context.hooks.after_spawn = Some(pause.clone());
    let mut tasks = Vec::new();
    for _ in 0..CAPACITY / JOBS {
        tasks.push(tokio::spawn(shell(
            context.clone(),
            "exit 0",
            Arc::new(AtomicBool::new(false)),
        )));
    }
    // Every record's readers are started before the common private pause. Holding
    // the reader vault proves the sixteen independent runtime frames have reached it.
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let records = context
                .pool
                .as_ref()
                .unwrap()
                .state
                .lock()
                .unwrap()
                .records
                .clone();
            if records.len() == 16 && records.iter().all(|r| r.readers.try_lock().is_err()) {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let rejected_flag = Arc::new(AtomicBool::new(false));
    let refused = run(
        Path::new("/rrx-must-not-spawn"),
        Path::new("/tmp"),
        &[],
        vec![],
        tokio::time::Instant::now() + Duration::from_millis(30),
        rejected_flag.clone(),
        Some(context.clone()),
    )
    .await;
    let actual = context
        .pool
        .as_ref()
        .unwrap()
        .state
        .lock()
        .unwrap()
        .records
        .clone();
    let all_workers = actual.iter().all(|r| r.worker.lock().unwrap().is_some());
    drop(release);
    for task in tasks {
        task.await.unwrap().unwrap();
    }
    released(&context).await;
    assert_eq!(actual.len() * JOBS, 64);
    assert!(all_workers);
    assert_eq!(refused.unwrap_err().kind, ErrorKind::Timeout);
    assert!(!rejected_flag.load(Ordering::SeqCst));
}

#[tokio::test]
async fn actual_stdout_cap_failure_aborts_and_observes_pending_peer() {
    let mut context = TestGitContext::isolated();
    context.context.hooks.pending_stderr = true;
    let flag = Arc::new(AtomicBool::new(false));
    let error = shell(context.clone(), "printf '%65537s' ''", flag.clone())
        .await
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::InvalidInput);
    assert_eq!(error.message, "Git metadata exceeds output budget");
    assert!(!flag.load(Ordering::SeqCst));
    released(&context).await;
    let facts = context.context.hooks.facts.lock().unwrap().unwrap();
    assert_eq!(facts.stdout.join, JoinState::Returned);
    assert_eq!(facts.stdout.read_error, Some(ErrorKind::InvalidInput));
    assert!(!facts.stdout.abort_requested);
    assert_eq!(facts.stderr.join, JoinState::Cancelled);
    assert!(facts.stderr.abort_requested);
    assert!(!facts.stderr.read_ok);
    assert!(facts.stderr.read_error.is_none());
}

#[tokio::test]
async fn post_signal_worker_panic_retains_child_without_a_second_signal() {
    let mut context = TestGitContext::isolated();
    context.context.hooks.worker_panic = true;
    let flag = Arc::new(AtomicBool::new(false));
    let error = tokio::time::timeout(
        Duration::from_secs(3),
        shell(context.clone(), "exit 0", flag.clone()),
    )
    .await
    .unwrap()
    .unwrap_err();
    assert_eq!(error.kind, ErrorKind::SessionLost);
    assert!(flag.load(Ordering::SeqCst));
    let record = context.pool.as_ref().unwrap().state.lock().unwrap().records[0].clone();
    let native = record.native.lock().unwrap_or_else(|p| p.into_inner());
    assert!(native.child.is_some());
    assert!(native.signal_issued);
    assert_eq!(native.signals, 1);
    assert!(native.cleanup.is_none());
}
#[tokio::test]
async fn bookkeeping_poison_after_settlement_holds_slots_without_false_native_unknown() {
    let mut context = TestGitContext::isolated();
    let pause = Arc::new(TestPause::default());
    let release = TestRelease(pause.clone());
    context.context.hooks.after_reap_send = Some(pause.clone());
    let flag = Arc::new(AtomicBool::new(false));
    let task = tokio::spawn(shell(context.clone(), "printf settled", flag.clone()));
    pause.reached().await;
    let pool = context.pool.as_ref().unwrap();
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _guard = pool.state.lock().unwrap();
            panic!("synthetic bookkeeping poison");
        }))
        .is_err()
    );
    drop(release);
    assert_eq!(task.await.unwrap().unwrap(), b"settled");
    assert!(!flag.load(Ordering::SeqCst));
    assert_eq!(
        pool.state.lock().err().unwrap().into_inner().records.len() * JOBS,
        4
    );
    let refused_flag = Arc::new(AtomicBool::new(false));
    assert_eq!(
        shell(context, "exit 0", refused_flag.clone())
            .await
            .unwrap_err()
            .kind,
        ErrorKind::LaunchFailure
    );
    assert!(!refused_flag.load(Ordering::SeqCst));
}
#[tokio::test]
async fn active_ticket_poison_returns_unknown_without_releasing_its_record() {
    let mut context = TestGitContext::isolated();
    let pause = Arc::new(TestPause::default());
    let release = TestRelease(pause.clone());
    context.context.hooks.after_spawn = Some(pause.clone());
    let flag = Arc::new(AtomicBool::new(false));
    let task = tokio::spawn(shell(context.clone(), "exit 0", flag.clone()));
    pause.reached().await;
    let record = context.pool.as_ref().unwrap().state.lock().unwrap().records[0].clone();
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _guard = record.ticket.publication.lock().unwrap();
            panic!("synthetic active ticket poison");
        }))
        .is_err()
    );
    drop(release);
    let error = tokio::time::timeout(Duration::from_secs(3), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::SessionLost);
    assert!(flag.load(Ordering::SeqCst));
    assert_eq!(
        context
            .pool
            .as_ref()
            .unwrap()
            .state
            .lock()
            .unwrap()
            .records
            .len()
            * JOBS,
        4
    );
}

#[tokio::test]
async fn caller_runtime_shutdown_keeps_independent_owner_and_frozen_flag() {
    let mut context = TestGitContext::isolated();
    let pause = Arc::new(TestPause::default());
    let release = TestRelease(pause.clone());
    context.context.hooks.after_spawn = Some(pause.clone());
    let flag = Arc::new(AtomicBool::new(false));
    let own_context = context.clone();
    let own_flag = flag.clone();
    let own_pause = pause.clone();
    let (finished, observed) = oneshot::channel();
    let (stop, stop_requested) = oneshot::channel();
    // One explicitly bounded fixture caller thread; this is not a component job.
    let caller = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.spawn(shell(own_context, "exec /bin/sleep 30", own_flag));
        runtime.block_on(async {
            tokio::select! { _ = own_pause.reached() => {}, _ = stop_requested => {} }
        });
        drop(runtime); // This drops the CALLER future, never the owner's runtime.
        let _ = finished.send(());
    });
    pause.reached().await;
    let stopped = tokio::time::timeout(Duration::from_secs(3), observed).await;
    // Own caller control only: settle the fixture runtime even when its positive
    // wake assertion fails. No PID rescue, extra job or production budget exists.
    let _ = stop.send(());
    caller.join().unwrap();
    let record = context.pool.as_ref().unwrap().state.lock().unwrap().records[0].clone();
    assert!(flag.load(Ordering::SeqCst));
    drop(release);
    released(&context).await;
    assert!(flag.load(Ordering::SeqCst));
    assert!(record.runtime.lock().unwrap().is_none());
    assert!(record.worker.lock().unwrap().is_none());
    assert!(record.native.lock().unwrap().child.is_none());
    stopped.unwrap().unwrap();
}
#[tokio::test]
async fn generic_live_launch_error_uses_actual_git_flag_without_retyping_error() {
    for unknown in [false, true] {
        let (_directory, store, project, task, worktree) = super::super::tests::preflight_fixture();
        let context = if unknown {
            TestGitContext::missing_executable()
        } else {
            TestGitContext::initializer_failure()
        };
        let mut adapter =
            GenericCliAdapter::new("fake".into(), vec!["/bin/cat".into()], store.clone()).unwrap();
        adapter.git_context = Some(context.clone());
        let result = adapter
            .start(super::super::tests::fixture_request(
                project, &task, worktree,
            ))
            .await
            .unwrap_err();
        assert_eq!(
            result.kind,
            if unknown {
                ErrorKind::ProcessFailure
            } else {
                ErrorKind::LaunchFailure
            }
        );
        let state = store.lock().unwrap();
        let records = state.records(&task.scope(), RecordKind::Session).unwrap();
        assert_eq!(records.len(), 1);
        let session: Session = serde_json::from_value(records[0].data.clone()).unwrap();
        assert_eq!(
            session.state,
            if unknown {
                SessionState::Lost
            } else {
                SessionState::Failed
            }
        );
        assert!(session.pid.is_none());
        assert!(session.native_ref.is_none());
        assert_eq!(crate::git::executor_reserved(&session), unknown);
        drop(state);
        if unknown {
            assert_eq!(context.held_jobs(), 4);
        } else {
            released(&context).await;
        }
    }
}

#[tokio::test]
async fn slow_cleanup_ack_does_not_consume_the_distinct_reap_window() {
    let mut context = TestGitContext::isolated();
    let pause = Arc::new(TestPause::default());
    let release = TestRelease(pause.clone());
    context.context.hooks.before_cleanup_ack = Some(pause.clone());
    let flag = Arc::new(AtomicBool::new(false));
    let task = tokio::spawn(shell(context.clone(), "printf cleanup", flag.clone()));
    pause.reached().await;
    // Hold the already-counted native worker beyond the reap window. Its cleanup
    // result is established, but no stage acknowledgement has been delivered.
    tokio::time::sleep(WINDOW + Duration::from_millis(30)).await;
    let premature = task.is_finished();
    drop(release);
    let result = task.await.unwrap();
    released(&context).await;
    assert!(!premature);
    assert_eq!(result.unwrap(), b"cleanup");
    assert!(!flag.load(Ordering::SeqCst));
}

#[test]
#[ignore = "explicit child entry for the owned process-group fixture"]
fn owned_alternate_group_fixture_child() {
    let mode = std::env::var("RRX_GIT_OWNER_FIXTURE_MODE").unwrap();
    let root = PathBuf::from(std::env::var_os("RRX_GIT_OWNER_FIXTURE_ROOT").unwrap());
    if mode == "moved" {
        let raw: i32 = std::env::var("RRX_GIT_OWNER_FIXTURE_GROUP")
            .unwrap()
            .parse()
            .unwrap();
        let group = Pid::from_raw(raw).unwrap();
        rustix::process::setpgid(None, Some(group)).unwrap();
    } else {
        assert_eq!(mode, "anchor");
    }
    std::fs::write(root.join(format!("{mode}.ready")), b"ready").unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !root.join(format!("{mode}.release")).exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "owned fixture release missing"
        );
        thread::sleep(Duration::from_millis(5));
    }
    std::process::exit(0);
}

struct AlternateFixture {
    child: StdChild,
    root: PathBuf,
}
impl Drop for AlternateFixture {
    fn drop(&mut self) {
        // These are solely our fixture's release files and actual direct Child.
        // This guard runs on the synchronous test thread, never an async poller.
        let _ = std::fs::write(self.root.join("moved.release"), b"release");
        let _ = std::fs::write(self.root.join("anchor.release"), b"release");
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
async fn fixture_ready(path: &Path) {
    tokio::time::timeout(Duration::from_secs(3), async {
        while !path.exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
}

#[test]
fn blocked_owning_child_wait_freezes_then_same_jobs_settle_late() {
    let directory = tempfile::tempdir().unwrap();
    let executable = std::env::current_exe().unwrap();
    let args = vec![
        "--ignored".to_owned(),
        "--exact".to_owned(),
        "adapter::git_owner::tests::owned_alternate_group_fixture_child".to_owned(),
        "--nocapture".to_owned(),
    ];
    let child = StdCommand::new(&executable)
        .args(&args)
        .env_clear()
        .env("RRX_GIT_OWNER_FIXTURE_MODE", "anchor")
        .env("RRX_GIT_OWNER_FIXTURE_ROOT", directory.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .unwrap();
    let anchor = AlternateFixture {
        child,
        root: directory.path().to_owned(),
    };
    let context = TestGitContext::isolated();
    let flag = Arc::new(AtomicBool::new(false));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let (failure, record, frozen, retained, facts, late_facts) = runtime.block_on(async {
        fixture_ready(&directory.path().join("anchor.ready")).await;
        let environment = vec![
            ("RRX_GIT_OWNER_FIXTURE_MODE".into(), "moved".into()),
            (
                "RRX_GIT_OWNER_FIXTURE_ROOT".into(),
                directory.path().as_os_str().to_owned(),
            ),
            (
                "RRX_GIT_OWNER_FIXTURE_GROUP".into(),
                anchor.child.id().to_string().into(),
            ),
        ];
        // Deadline applies to observation, followed by the unchanged 250ms owning
        // reap window. The original group becomes empty but this direct child lives.
        let call_executable = executable.clone();
        let call_root = directory.path().to_owned();
        let call_args = args.clone();
        let call_flag = flag.clone();
        let call_context = context.clone();
        let call = tokio::spawn(async move {
            run(
                &call_executable,
                &call_root,
                &call_args,
                environment,
                tokio::time::Instant::now() + Duration::from_secs(1),
                call_flag,
                Some(call_context),
            )
            .await
        });
        fixture_ready(&directory.path().join("moved.ready")).await;
        let record = context.pool.as_ref().unwrap().state.lock().unwrap().records[0].clone();
        let failure = tokio::time::timeout(Duration::from_secs(3), call)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        let frozen = flag.load(Ordering::SeqCst);
        let retained = context.held_jobs();
        let facts = context.context.hooks.facts.lock().unwrap().unwrap();
        // The native vault is intentionally held by the actual blocking wait.
        assert!(record.native.try_lock().is_err());
        // Cancellation remains a short signal while that wait is blocked.
        record.ticket.cancel();
        std::fs::write(directory.path().join("moved.release"), b"release").unwrap();
        released(&context).await;
        let late_facts = context.context.hooks.facts.lock().unwrap().unwrap();
        (failure, record, frozen, retained, facts, late_facts)
    });
    drop(runtime);
    drop(anchor);
    assert_eq!(failure.kind, ErrorKind::SessionLost);
    assert_eq!(
        failure.message,
        "Git child death not confirmed after cleanup"
    );
    assert!(frozen && flag.load(Ordering::SeqCst));
    assert_eq!(retained, JOBS);
    assert_eq!(facts, late_facts);
    assert!(facts.stdout.join.settled() && facts.stderr.join.settled());
    assert!(record.runtime.lock().unwrap().is_none());
    assert!(record.worker.lock().unwrap().is_none());
    assert!(record.native.lock().unwrap().child.is_none());
}
