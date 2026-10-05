use super::*;

async fn shell(context: TestGitContext, script: &str, flag: Arc<AtomicBool>) -> AdapterResult<Vec<u8>> {
    run(Path::new("/bin/sh"), Path::new("/private/tmp"), &["-c".into(), script.into()], vec![], tokio::time::Instant::now() + Duration::from_secs(5), flag, Some(context)).await
}
#[tokio::test]
async fn binary_output_and_settled_error_clear_live_uncertainty() {
    let context = TestGitContext::isolated();
    let flag = Arc::new(AtomicBool::new(false));
    assert_eq!(shell(context.clone(), "printf 'a\\000 b\\n'", flag.clone()).await.unwrap(), b"a\0 b\n");
    assert!(!flag.load(Ordering::SeqCst));
    assert_eq!(shell(context.clone(), "exit 7", flag.clone()).await.unwrap_err().kind, ErrorKind::OwnershipMismatch);
    assert!(!flag.load(Ordering::SeqCst));
    assert!(context.pool.as_ref().unwrap().state.lock().unwrap().records.is_empty());
}
#[tokio::test]
async fn expired_refusal_precedes_capacity_and_has_no_job_or_flag() {
    let context = TestGitContext::isolated();
    let flag = Arc::new(AtomicBool::new(false));
    let result = run(Path::new("/nonexistent"), Path::new("/private/tmp"), &[], vec![], tokio::time::Instant::now(), flag.clone(), Some(context.clone())).await.unwrap_err();
    assert_eq!(result.kind, ErrorKind::Timeout);
    assert!(!flag.load(Ordering::SeqCst));
    assert!(context.pool.as_ref().unwrap().state.lock().unwrap().records.is_empty());
}
#[tokio::test]
async fn actual_std_spawn_then_initialization_failure_is_settled() {
    let mut context = TestGitContext::isolated();
    context.context.hooks.initialized_error = true;
    let flag = Arc::new(AtomicBool::new(false));
    let failure = shell(context.clone(), "exec sleep 30", flag.clone()).await.unwrap_err();
    assert_eq!(failure.kind, ErrorKind::LaunchFailure);
    assert!(!flag.load(Ordering::SeqCst));
    assert!(context.pool.as_ref().unwrap().state.lock().unwrap().records.is_empty());
}
#[tokio::test]
async fn opaque_attempted_spawn_error_retains_exact_four_slots() {
    let context = TestGitContext::isolated();
    let flag = Arc::new(AtomicBool::new(false));
    let failure = run(Path::new("/rrx-no-such-executable"), Path::new("/private/tmp"), &[], vec![], tokio::time::Instant::now() + Duration::from_secs(5), flag.clone(), Some(context.clone())).await.unwrap_err();
    assert_eq!(failure.kind, ErrorKind::ProcessFailure);
    assert!(flag.load(Ordering::SeqCst));
    let records = context.pool.as_ref().unwrap().state.lock().unwrap();
    assert_eq!(records.records.len() * JOBS, 4);
    assert!(records.records[0].native.lock().unwrap().child.is_none());
}
