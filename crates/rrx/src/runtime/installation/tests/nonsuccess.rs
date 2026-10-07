//! RN controls use accepted ingress and the installed issuer. The external
//! configured peer is account-free protocol wiring, never Native qualification.
use super::*;
use crate::execution::{Disposition, UnitState, native::PreparationObservation};
use std::sync::Mutex;

#[derive(Default)]
struct ServicePark {
    state: Mutex<u8>,
    changed: std::sync::Condvar,
}
impl ServicePark {
    fn close(&self) {
        *self.state.lock().unwrap() = 1;
    }
    fn visit(&self) {
        let mut state = self.state.lock().unwrap();
        if *state == 1 {
            *state = 2;
            self.changed.notify_all();
            while *state == 2 {
                state = self.changed.wait(state).unwrap();
            }
        }
    }
    fn parked(&self) -> bool {
        *self.state.lock().unwrap() == 2
    }
    fn open(&self) {
        *self.state.lock().unwrap() = 3;
        self.changed.notify_all();
    }
}
struct OpenPark(Arc<ServicePark>);
impl Drop for OpenPark {
    fn drop(&mut self) {
        self.0.open();
    }
}
async fn wait_for(mut condition: impl FnMut() -> bool, label: &str) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(40);
    while !condition() {
        assert!(tokio::time::Instant::now() < deadline, "{label}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
fn links(f: &ControlFixture) -> usize {
    raw(f)
        .query_row(
            "SELECT count(*) FROM audit WHERE kind='rrx.private.workflow.phase_closed'",
            [],
            |r| r.get(0),
        )
        .unwrap()
}
fn workflow(f: &ControlFixture, task: &Task) -> crate::domain::Record {
    f.owner
        .store
        .lock()
        .unwrap()
        .records(&task.scope(), RecordKind::Workflow)
        .unwrap()
        .remove(0)
}
/// Only the configured executable is changed, after genuine helpers completed.
/// Neither hook supplies an error: the production physical-profile check does.
async fn physical_refusal(provider: &str, stage: PreparationObservation) {
    let mut f = fixture(provider, true);
    f.register_real_git_project();
    if let Err(refusal) = &f.runtime.installed {
        // Diagnose only the physical fixture-local IPC syscall. This probe
        // does not supply Runtime installation, an owner or Native admission.
        let diagnostic =
            std::os::unix::net::UnixListener::bind(f._dir.path().join("rn-ipc-diagnostic.sock"));
        panic!(
            "SETUP: {}; fixture-local Unix bind: {diagnostic:?}",
            refusal.0
        );
    }
    let port = f
        .runtime
        .installed
        .as_ref()
        .ok()
        .unwrap_or_else(|| panic!("SETUP: Runtime installation unavailable"))
        .registry
        .native_phase_port("worker")
        .unwrap();
    let adapter = port.selected_adapter().unwrap();
    let arrived = Arc::new(tokio::sync::Semaphore::new(0));
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let hook_arrived = arrived.clone();
    let hook_release = release.clone();
    adapter
        .sessions
        .set_preparation_observer(Some(Arc::new(move |observed| {
            let arrived = hook_arrived.clone();
            let release = hook_release.clone();
            Box::pin(async move {
                if observed == stage {
                    arrived.add_permits(1);
                    release.acquire().await.unwrap().forget();
                }
            })
        })));
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    f.runtime.start().await.unwrap();
    tokio::time::timeout(Duration::from_secs(40), arrived.acquire())
        .await
        .expect("SETUP: real helper completion did not reach physical fault point")
        .unwrap()
        .forget();
    let park = Arc::new(ServicePark::default());
    let open = OpenPark(park.clone());
    let hook = park.clone();
    *f.runtime.before_service_reconcile.lock().unwrap() = Some(Arc::new(move || hook.visit()));
    park.close();
    f.runtime.wake.notify_one();
    wait_for(|| park.parked(), "SETUP: Root observation did not park").await;

    let before = f.runtime.phase_jobs.observed_jobs();
    assert_eq!(before.len(), 1, "SETUP: genuine retained start absent");
    assert!(before[0].preparation.helpers_completed);
    assert!(!before[0].finished && !before[0].preparation.revoked);
    assert!(!before[0].preparation.transport);
    assert_eq!(
        before[0].preparation.no_dispatch,
        stage == PreparationObservation::BeforeTransport
    );
    let unit_id = before[0].unit;
    let allocation = f.runtime.phase_jobs.original_allocation(unit_id).unwrap();
    let unit_before = f
        .owner
        .store
        .lock()
        .unwrap()
        .execution_unit(unit_id)
        .unwrap();
    let workflow_before = workflow(&f, task);
    let task_before = f
        .owner
        .store
        .lock()
        .unwrap()
        .task(task.id)
        .unwrap()
        .unwrap();
    let operation_before: (i64, i64, String) = raw(&f)
        .query_row(
            "SELECT phase_open,version,body FROM managed_phase_operations",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(operation_before.0, 1);
    assert_eq!(links(&f), 0);
    let program = adapter.program.clone();
    let backup = program.with_extension("original");
    std::fs::rename(&program, &backup).unwrap();
    std::fs::create_dir(&program).unwrap();
    release.add_permits(1);
    wait_for(
        || {
            f.runtime
                .phase_jobs
                .observed_jobs()
                .iter()
                .any(|j| j.finished && j.refusal.is_some())
        },
        "SETUP: actual retained start did not finish with physical refusal",
    )
    .await;
    let refused = f.runtime.phase_jobs.observed_jobs();
    assert!(
        refused[0]
            .refusal
            .as_ref()
            .unwrap()
            .contains("original Native program/worktree physical profile differs"),
        "wrong refusal: {refused:?}"
    );
    assert!(refused[0].preparation.revoked && !refused[0].preparation.transport);
    assert!(!refused[0].preparation.closure && !refused[0].preparation.closed);
    assert!(
        f.owner
            .store
            .lock()
            .unwrap()
            .records(&task.scope(), RecordKind::Session)
            .unwrap()
            .is_empty(),
        "physical refusal registered a Session"
    );
    if stage == PreparationObservation::BeforeTransport {
        assert!(
            f.runtime.phase_dispatcher.reconcile_nonsuccess().unwrap(),
            "N first turn did not yield after preparation closure"
        );
        let k = f.runtime.phase_jobs.observed_jobs();
        assert_eq!(k.len(), 1);
        assert!(
            k[0].preparation.closure && k[0].preparation.closed,
            "N did not become genuine K: {k:?}"
        );
        assert_eq!(links(&f), 0, "N ran RN write in preparation turn");
        f.runtime.phase_dispatcher.reconcile_nonsuccess().unwrap();
        assert_eq!(
            links(&f),
            1,
            "RN typed phase_closed missing after genuine K"
        );
        assert!(
            f.runtime.phase_jobs.observed_jobs().is_empty(),
            "ack did not retire same finished job"
        );
        let after = f
            .owner
            .store
            .lock()
            .unwrap()
            .execution_unit(unit_id)
            .unwrap();
        assert_eq!(after.state, UnitState::Retired);
        assert_eq!(after.disposition, Disposition::Refused);
        assert!(!after.native_effects_open && !after.result_finalization_open);
        assert!(after.session_id.is_none() && after.work.is_none());
        assert_eq!(after.version, unit_before.version + 1);
        let record = workflow(&f, task);
        assert_eq!(record.version, workflow_before.version + 1);
        let snapshot: crate::workflow::WorkflowSnapshot =
            serde_json::from_value(record.data).unwrap();
        let attempt = &snapshot.history[snapshot.active.unwrap()];
        assert_eq!(attempt.state, crate::workflow::AttemptState::Failed);
        assert!(attempt.session_id.is_none() && attempt.execution.is_none());
        assert_eq!(
            attempt.detail.as_deref(),
            Some("native start ended before dispatch; explicit retry required")
        );
        let original: crate::workflow::WorkflowSnapshot =
            serde_json::from_value(workflow_before.data.clone()).unwrap();
        assert!(!snapshot.finished);
        assert_eq!(
            serde_json::to_value(&snapshot.completed).unwrap(),
            serde_json::to_value(&original.completed).unwrap(),
            "RN changed genuine initial gate completions"
        );
        let operation_after: (i64, i64, String) = raw(&f)
            .query_row(
                "SELECT phase_open,version,body FROM managed_phase_operations",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!((operation_after.0, operation_after.1), (0, 2));
        let audit: String = raw(&f)
            .query_row(
                "SELECT data FROM audit WHERE kind='rrx.private.workflow.phase_closed'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let audit: serde_json::Value = serde_json::from_str(&audit).unwrap();
        assert_eq!(audit["unit_id"], unit_id.to_string());
        assert_eq!(audit["closure"], "non_success");
        assert_eq!(audit["proof_source"], "no_current_dispatch");
        assert_eq!(audit["task_version_preserved"], task_before.version);
        assert_eq!(audit["session_bound"], false);
        // Repeating the real consumer is idempotent, not a second close.
        f.runtime.phase_dispatcher.reconcile_nonsuccess().unwrap();
        assert_eq!(links(&f), 1);
    } else {
        f.runtime.phase_dispatcher.reconcile_nonsuccess().unwrap();
        let held = f.runtime.phase_jobs.observed_jobs();
        assert_eq!(held.len(), 1, "H1 pre-S5 job was retired");
        assert_eq!(held[0].attention, Some("original preparation Held"));
        assert!(!held[0].preparation.no_dispatch && !held[0].preparation.closure);
        assert_eq!(links(&f), 0, "H1 missing S5 gained a closure");
        assert_eq!(
            serde_json::to_value(
                f.owner
                    .store
                    .lock()
                    .unwrap()
                    .execution_unit(unit_id)
                    .unwrap()
            )
            .unwrap(),
            serde_json::to_value(&unit_before).unwrap()
        );
        assert_eq!(
            serde_json::to_value(workflow(&f, task)).unwrap(),
            serde_json::to_value(workflow_before).unwrap()
        );
        let operation_after: (i64, i64, String) = raw(&f)
            .query_row(
                "SELECT phase_open,version,body FROM managed_phase_operations",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(operation_after, operation_before);
    }
    assert_eq!(
        serde_json::to_value(
            f.owner
                .store
                .lock()
                .unwrap()
                .task(task.id)
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(task_before).unwrap(),
        "RN changed Task marker image"
    );
    std::fs::remove_dir(&program).unwrap();
    std::fs::rename(backup, program).unwrap();
    adapter.sessions.set_preparation_observer(None);
    drop(allocation);
    drop(adapter);
    drop(port);
    park.open();
    drop(open);
    let shutdown = f.runtime.shutdown().await.unwrap_err();
    assert!(
        shutdown
            .to_string()
            .contains("Source phase shutdown remains pending")
            || shutdown
                .to_string()
                .contains("Source handoff shutdown remains pending")
            || shutdown
                .to_string()
                .contains("Native phase shutdown remains pending"),
        "unexpected shutdown: {shutdown}"
    );
    finish(f).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn rn_h1_genuine_normal_n_closes_and_pre_s5_refusal_stays_held() {
    for provider in ["claude", "codex"] {
        physical_refusal(provider, PreparationObservation::BeforeTransport).await;
        physical_refusal(provider, PreparationObservation::BeforeCommand).await;
    }
}
