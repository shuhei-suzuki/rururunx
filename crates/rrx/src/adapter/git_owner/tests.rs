use super::*;

async fn shell(
    context: TestGitContext,
    script: &str,
    flag: Arc<AtomicBool>,
) -> AdapterResult<Vec<u8>> {
    run(
        Path::new("/bin/sh"),
        Path::new("/private/tmp"),
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
        Path::new("/private/tmp"),
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
    let failure = shell(context.clone(), "exec sleep 30", flag.clone())
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
        Path::new("/private/tmp"),
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
        Path::new("/private/tmp"),
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
        Path::new("/private/tmp"),
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
}
