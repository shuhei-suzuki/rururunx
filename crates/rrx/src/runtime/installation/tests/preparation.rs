//! DC controls for the preparation known-commit caller. They use accepted
//! ingress, the installed issuer and the genuine service loop; the cfg(test)
//! seams only inject one commit fault or park the loop, never grant anything.
use super::*;
use crate::state::{PreparationFault, arm_preparation_fault};

async fn wait_for(mut condition: impl FnMut() -> bool, label: &str) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(40);
    while !condition() {
        assert!(tokio::time::Instant::now() < deadline, "{label}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Unit, Task and phase-operation images that no confirmation may change.
fn protected_rows(f: &ControlFixture, task: &Task, unit: crate::execution::UnitId) -> String {
    let store = f.owner.store.lock().unwrap();
    let unit = serde_json::to_string(&store.execution_unit(unit).unwrap()).unwrap();
    let task = serde_json::to_string(&store.task(task.id).unwrap().unwrap()).unwrap();
    drop(store);
    let operations: Vec<(i64, i64, String)> = {
        let raw = raw(f);
        let mut statement = raw
            .prepare("SELECT phase_open,version,body FROM managed_phase_operations ORDER BY rowid")
            .unwrap();
        statement
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    format!("{unit}\n{task}\n{operations:?}")
}

fn installed(f: &ControlFixture) {
    if let Err(refusal) = &f.runtime.installed {
        panic!("SETUP: {}", refusal.0);
    }
}

/// DC1: a genuine commit whose call returned Err is confirmed once by the
/// service loop and retained as Known, with no protected row change.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn dc1_postcommit_preparation_error_is_confirmed_known_once() {
    let mut f = fixture("claude", true);
    f.register_real_git_project();
    installed(&f);
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    arm_preparation_fault(task.id, PreparationFault::AfterCommit);
    let (reached, release) = super::super::super::phase_jobs::arm_preparation_park(task.id);
    f.runtime.start().await.unwrap();
    tokio::time::timeout(Duration::from_secs(40), reached)
        .await
        .expect("DC1: the sweep never selected the uncertain preparation commit")
        .unwrap();
    let before = f.runtime.phase_jobs.observed_jobs();
    assert_eq!(before.len(), 1, "SETUP: genuine retained start absent");
    assert!(
        before[0]
            .refusal
            .as_deref()
            .is_some_and(|r| r.contains("injected postcommit preparation fault")),
        "SETUP: start did not end with the injected postcommit fault: {before:?}"
    );
    assert!(before[0].preparation.plan && !before[0].preparation.known);
    assert!(before[0].preparation.revoked && !before[0].preparation.no_dispatch);
    let unit = before[0].unit;
    let rows = protected_rows(&f, task, unit);
    release.send(()).unwrap();
    wait_for(
        || f.runtime.phase_jobs.observed_jobs()[0].preparation.known,
        "DC1: confirmation did not retain the Known commit",
    )
    .await;
    let after = f.runtime.phase_jobs.observed_jobs();
    assert!(
        after[0].preparation_attempts.is_empty() && after[0].preparation_busy == 0,
        "DC1: Known commit took a Held or busy turn: {after:?}"
    );
    assert_eq!(
        protected_rows(&f, task, unit),
        rows,
        "DC1: confirmation wrote a protected row"
    );
    // RN still holds the job: Known adds no closure (RN-1b is out of scope).
    f.runtime.phase_dispatcher.reconcile_nonsuccess().unwrap();
    let held = f.runtime.phase_jobs.observed_jobs();
    assert_eq!(held.len(), 1, "DC1: Known preparation retired the job");
    assert!(!held[0].preparation.closure && !held[0].preparation.closed);
    f.runtime.shutdown().await.unwrap_err();
    finish(f).await;
}

/// DC2 and DC4: a fault before the commit leaves a genuine retained plan whose
/// confirmation is Held on every attempt, spaced by the capped backoff even
/// when the service is woken far more often.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn dc2_dc4_precommit_error_stays_held_with_capped_backoff() {
    let mut f = fixture("claude", true);
    f.register_real_git_project();
    installed(&f);
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    arm_preparation_fault(task.id, PreparationFault::BeforeCommit);
    let (reached, release) = super::super::super::phase_jobs::arm_preparation_park(task.id);
    f.runtime.start().await.unwrap();
    tokio::time::timeout(Duration::from_secs(40), reached)
        .await
        .expect("DC2: the sweep never selected the uncertain preparation commit")
        .unwrap();
    let before = f.runtime.phase_jobs.observed_jobs();
    assert!(
        before[0]
            .refusal
            .as_deref()
            .is_some_and(|r| r.contains("injected precommit preparation fault")),
        "SETUP: start did not end with the injected precommit fault: {before:?}"
    );
    let unit = before[0].unit;
    let rows = protected_rows(&f, task, unit);
    release.send(()).unwrap();
    const EXPECTED: [u64; 8] = [100, 200, 400, 800, 1600, 3200, 5000, 5000];
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        let attempts = f.runtime.phase_jobs.observed_jobs()[0]
            .preparation_attempts
            .len();
        if attempts >= EXPECTED.len() {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "DC4: only {attempts} Held attempts"
        );
        // Extra service wakes, far more often than the backoff.
        f.runtime.wake.notify_one();
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let observed = f.runtime.phase_jobs.observed_jobs();
    let attempts = &observed[0].preparation_attempts;
    // DC2: every confirmation is Held; nothing is retried, known or written.
    assert!(
        !observed[0].preparation.known,
        "DC2: precommit plan became Known"
    );
    assert!(observed[0].preparation.plan && !observed[0].preparation.no_dispatch);
    assert_eq!(
        protected_rows(&f, task, unit),
        rows,
        "DC2: Held confirmation wrote a protected row"
    );
    // DC4: the stored backoff sequence up to and at the cap, and no attempt
    // inside a backoff window despite the extra wakes.
    let applied: Vec<u64> = attempts.iter().take(EXPECTED.len()).map(|a| a.1).collect();
    assert_eq!(applied, EXPECTED, "DC4: backoff sequence");
    for pair in attempts.windows(2).take(EXPECTED.len() - 1) {
        let spacing = pair[1].0.duration_since(pair[0].0);
        assert!(
            spacing >= Duration::from_millis(pair[0].1),
            "DC4: attempt inside the {} ms backoff window ({spacing:?})",
            pair[0].1
        );
    }
    f.runtime.shutdown().await.unwrap_err();
    finish(f).await;
}

/// DC3: shutdown holds the control admission before the selected job's
/// admission try. The loop does not wait: NotAttempted, nothing advanced, and
/// the service join completes without its timeout.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn dc3_shutdown_holding_admission_makes_selected_try_not_attempted() {
    let mut f = fixture("claude", true);
    f.register_real_git_project();
    installed(&f);
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    arm_preparation_fault(task.id, PreparationFault::BeforeCommit);
    let (reached, release) = super::super::super::phase_jobs::arm_preparation_park(task.id);
    f.runtime.start().await.unwrap();
    tokio::time::timeout(Duration::from_secs(40), reached)
        .await
        .expect("DC3: the sweep never selected the uncertain preparation commit")
        .unwrap();
    let before = f.runtime.phase_jobs.observed_jobs();
    let runtime = f.runtime.clone();
    let started = std::time::Instant::now();
    let shutdown = tokio::spawn(async move { runtime.shutdown().await });
    wait_for(
        || f.runtime.stopping.load(std::sync::atomic::Ordering::SeqCst),
        "SETUP: shutdown did not take the control admission",
    )
    .await;
    assert!(
        f.runtime.control_admission.try_lock().is_err(),
        "SETUP: shutdown does not hold the control admission"
    );
    release.send(()).unwrap();
    let result = tokio::time::timeout(Duration::from_secs(20), shutdown)
        .await
        .expect("DC3: shutdown did not return")
        .unwrap();
    if let Err(error) = &result {
        assert!(
            !error
                .to_string()
                .contains("Runtime control loop shutdown remains pending"),
            "DC3: service join waited for its timeout: {error}"
        );
    }
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "DC3: service join took {:?}",
        started.elapsed()
    );
    let after = f.runtime.phase_jobs.observed_jobs();
    assert_eq!(after[0].preparation_busy, 1, "DC3: busy try not counted");
    assert!(
        after[0].preparation_attempts.is_empty(),
        "DC3: busy try was treated as an attempt: {after:?}"
    );
    assert_eq!(
        after[0].preparation_due, before[0].preparation_due,
        "DC3: busy try advanced the due time"
    );
    assert!(!after[0].preparation.known);
    finish(f).await;
}
