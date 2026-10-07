//! CA controls enter only through the actual Runtime and accepted Unix ingress.
//! The configured peer is account-free protocol wiring, never qualification.
use super::*;
use crate::runtime::goal::GoalPlan;
use crate::state::managed_binding::ActivationSeams;
use sha2::{Digest, Sha256};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
mod composition;
mod lifecycle;
mod pages;
mod protection;

#[derive(Clone)]
struct ObservedActivation {
    plan: Arc<crate::state::DriverPreparationAdvance>,
    association: Arc<crate::runtime::driver::DriverAssociation>,
    expected: serde_json::Value,
}
struct InitialRecordObservation {
    id: crate::domain::RecordId,
    raw: String,
}
impl ObservedActivation {
    fn from_probe(probe: &crate::workflow::ActivationProbe<'_>) -> Self {
        Self {
            plan: probe.plan().clone(),
            association: Arc::new(probe.lifetime().association().unwrap()),
            expected: expected_contract(probe),
        }
    }
}
/// One service arrival parks. Later direct observation calls use the genuine
/// production sweep without being blocked by this test timing seam.
#[derive(Default)]
struct ServicePark {
    state: Mutex<u8>, // 0 open, 1 closed, 2 parked, 3 released
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
    fn wait_parked(&self) {
        let end = std::time::Instant::now() + Duration::from_secs(40);
        let mut state = self.state.lock().unwrap();
        while *state != 2 {
            let remaining = end.saturating_duration_since(std::time::Instant::now());
            assert!(
                !remaining.is_zero(),
                "SETUP: synchronous segment did not park"
            );
            state = self.changed.wait_timeout(state, remaining).unwrap().0;
        }
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
fn service_park(f: &ControlFixture) -> (Arc<ServicePark>, OpenPark) {
    let park = Arc::new(ServicePark::default());
    let hook = park.clone();
    *f.runtime.before_service_reconcile.lock().unwrap() = Some(Arc::new(move || hook.visit()));
    (park.clone(), OpenPark(park))
}
fn install_pause(
    f: &ControlFixture,
    seams: Option<Arc<ActivationSeams>>,
) -> (
    Arc<Mutex<Vec<ObservedActivation>>>,
    Arc<tokio::sync::Semaphore>,
) {
    let observations = Arc::new(Mutex::new(Vec::new()));
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    let capture = observations.clone();
    let permits = release.clone();
    engine(f).set_activation_hooks(
        Some(Arc::new(move |probe| {
            capture
                .lock()
                .unwrap()
                .push(ObservedActivation::from_probe(&probe));
            let permits = permits.clone();
            Box::pin(async move {
                permits.acquire().await.unwrap().forget();
            })
        })),
        seams,
    );
    (observations, release)
}
async fn accepted_plan(f: &ControlFixture, p: GoalPlan) -> Vec<Task> {
    let goal = f.create(p).await;
    let store = f.owner.store.lock().unwrap();
    store
        .goal(goal)
        .unwrap()
        .unwrap()
        .dag
        .nodes
        .iter()
        .map(|id| store.task(*id).unwrap().unwrap())
        .collect()
}
fn task_plan(reviewers: &[&str]) -> GoalPlan {
    let mut p = plan();
    p.tasks[0].risk = RiskClass::R1;
    p.tasks[0].reviewers = reviewers.iter().map(|s| (*s).into()).collect();
    p
}
fn assert_contract(f: &ControlFixture, observed: &ObservedActivation, label: &str) {
    let contract: String = raw(f)
        .query_row(
            "SELECT body FROM workflow_native_contracts WHERE task_id=?1",
            [observed.expected["task_id"].as_str().unwrap()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        contract.as_bytes(),
        canonical(&observed.expected),
        "{label}: exact original contract"
    );
}
async fn wait_exited(f: &ControlFixture, expected: usize) {
    wait_for(
        || {
            f.runtime._drivers.observe_finished().unwrap() == 0
                && f.runtime._drivers.pending_exits().unwrap().len() >= expected
        },
        "actual Driver jobs did not finish",
    )
    .await;
}
fn engine(f: &ControlFixture) -> &Arc<crate::workflow::WorkflowEngine> {
    &f.runtime
        .installed
        .as_ref()
        .unwrap_or_else(|refusal| panic!("SETUP: {}", refusal.0))
        .engine
}

fn canonical(value: &serde_json::Value) -> Vec<u8> {
    fn ordered(v: &serde_json::Value) -> serde_json::Value {
        match v {
            serde_json::Value::Object(m) => {
                let mut keys = m.keys().collect::<Vec<_>>();
                keys.sort();
                serde_json::Value::Object(
                    keys.into_iter()
                        .map(|k| (k.clone(), ordered(&m[k])))
                        .collect(),
                )
            }
            serde_json::Value::Array(a) => {
                serde_json::Value::Array(a.iter().map(ordered).collect())
            }
            v => v.clone(),
        }
    }
    serde_json::to_vec(&ordered(value)).unwrap()
}
fn independent_digest(domain: &[u8], value: &serde_json::Value) -> String {
    let mut h = Sha256::new();
    h.update(domain);
    h.update(canonical(value));
    format!("{:x}", h.finalize())
}
fn expected_contract(probe: &crate::workflow::ActivationProbe<'_>) -> serde_json::Value {
    let task = probe.task();
    let mut members = Vec::new();
    let composition = probe.composition();
    for (role, alias) in
        std::iter::once((crate::domain::SessionRole::Executor, task.executor.as_str())).chain(
            task.reviewers
                .iter()
                .map(|a| (crate::domain::SessionRole::Reviewer, a.as_str())),
        )
    {
        let port = composition.engine().installed_native_port(alias).unwrap();
        let adapter = port.selected_adapter().unwrap();
        members.push(independent_digest(b"rrx.native-roster-member/v1\0",&serde_json::json!({
            "role":role,"alias":port.alias(),"provider":port.provider(),"program":port.program(),
            "declaration_digest":adapter.compatibility.as_ref().unwrap().digest(),"port_origin":port.origin_id()
        })));
    }
    members.sort();
    let profile = independent_digest(b"rrx.native-roster/v1\0", &serde_json::json!(members));
    serde_json::json!({"workflow_id":probe.record().id,"project_id":task.project_id,"goal_id":task.goal_id,
        "task_id":task.id,"owner_epoch":composition.selected().owner().epoch(),
        "origin":composition.selected().installation_id(),"profile_digest":profile,"contract_state":"composed",
        "version":1,"members":members})
}
async fn wait_for(mut condition: impl FnMut() -> bool, label: &str) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(40);
    while !condition() {
        assert!(tokio::time::Instant::now() < deadline, "{label}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
async fn wait_bound(
    f: &ControlFixture,
    task: &Task,
) -> (crate::domain::Record, crate::workflow::WorkflowSnapshot) {
    wait_for(
        || {
            let store = f.owner.store.lock().unwrap();
            store
                .records(&task.scope(), RecordKind::Workflow)
                .unwrap()
                .first()
                .is_some_and(|r| {
                    let w: crate::workflow::WorkflowSnapshot =
                        serde_json::from_value(r.data.clone()).unwrap();
                    w.history.iter().any(|a| a.execution.is_some())
                })
        },
        "CA1 gate/first-Executor stage: original marker and Bound absent",
    )
    .await;
    let record = f
        .owner
        .store
        .lock()
        .unwrap()
        .records(&task.scope(), RecordKind::Workflow)
        .unwrap()
        .remove(0);
    let workflow = serde_json::from_value(record.data.clone()).unwrap();
    (record, workflow)
}
fn release_peer(f: &ControlFixture, workflow: &crate::workflow::WorkflowSnapshot) {
    let reference = workflow
        .history
        .iter()
        .find_map(|a| a.execution.as_ref())
        .unwrap();
    let unit = f
        .owner
        .store
        .lock()
        .unwrap()
        .execution_unit(reference.unit)
        .unwrap();
    let profile = crate::execution::resources::ResourceManager::new(f.owner.clone())
        .profile(&unit)
        .unwrap();
    std::fs::write(profile.output.join("fixture-bootstrap-release"), "release").unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca1_actual_activation_gates_marker_and_record_only_bound() {
    for provider in ["claude", "codex"] {
        let mut f = fixture(provider, true);
        f.register_real_git_project();
        let (_, tasks) = accept(&f, 1).await;
        let expected = Arc::new(Mutex::new(None));
        let arrived = Arc::new(AtomicBool::new(false));
        let capture = expected.clone();
        let signal = arrived.clone();
        let initial = Arc::new(Mutex::new(
            None::<(
                InitialRecordObservation,
                (String, u64, u64, String),
                (String, String),
            )>,
        ));
        let original = initial.clone();
        let reader = Mutex::new(
            rusqlite::Connection::open_with_flags(
                f.owner.state_path(),
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .unwrap(),
        );
        let atomic = Arc::new(AtomicBool::new(false));
        let atomic_observed = atomic.clone();
        let observed = initial.clone();
        let seams = Arc::new(ActivationSeams {
            published: Some(Arc::new(move || {
                let original = observed.lock().unwrap();
                let (record, binding, contract) = original.as_ref().unwrap();
                let reader = reader.lock().unwrap();
                let mut statement = reader.prepare("SELECT r.version,r.body,c.version,d.id,d.owner_epoch,d.version,d.body,v.owner_epoch,n.owner_epoch,n.origin,n.body FROM records r JOIN context_versions c ON c.project_id=r.project_id AND c.goal_id=r.goal_id AND c.task_id=r.task_id AND c.version=1 JOIN task_drivers d ON d.task_id=r.task_id JOIN workflow_verification_contracts v ON v.workflow_id=r.id AND v.project_id=r.project_id AND v.goal_id=r.goal_id AND v.task_id=r.task_id JOIN workflow_native_contracts n ON n.workflow_id=r.id AND n.project_id=r.project_id AND n.goal_id=r.goal_id AND n.task_id=r.task_id WHERE r.id=?1").unwrap();
                let rows = statement
                    .query_map([record.id.to_string()], |r| {
                        Ok((
                            r.get::<_, u64>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, u64>(2)?,
                            r.get::<_, String>(3)?,
                            r.get::<_, u64>(4)?,
                            r.get::<_, u64>(5)?,
                            r.get::<_, String>(6)?,
                            r.get::<_, u64>(7)?,
                            r.get::<_, u64>(8)?,
                            r.get::<_, String>(9)?,
                            r.get::<_, String>(10)?,
                        ))
                    })
                    .unwrap()
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .unwrap();
                assert_eq!(rows.len(), 1, "CA1 atomic activation five scoped rows");
                let row = &rows[0];
                assert_eq!(
                    (row.0, row.1.as_str(), row.2),
                    (1, record.raw.as_str(), 1),
                    "CA1 original first Workflow and Context"
                );
                assert_eq!(
                    (&row.3, row.4, row.5, &row.6),
                    (&binding.0, binding.1, binding.2, &binding.3),
                    "CA1 original exact committed Driver"
                );
                assert_eq!(
                    (row.7, row.8),
                    (binding.1, binding.1),
                    "CA1 runtime epoch contracts"
                );
                assert_eq!(
                    (&row.9, &row.10),
                    (&contract.0, &contract.1),
                    "CA1 original installation and contract"
                );
                atomic_observed.store(true, Ordering::SeqCst);
            })),
            ..Default::default()
        });
        engine(&f).set_activation_hooks(
            Some(Arc::new(move |probe| {
                let expected = expected_contract(&probe);
                let mut record = probe.record().clone();
                record.version += 1;
                record.updated_at = record.created_at;
                let binding = probe.plan().planned_binding();
                *original.lock().unwrap() = Some((
                    InitialRecordObservation {
                        id: record.id,
                        raw: serde_json::to_string(&record).unwrap(),
                    },
                    (
                        binding.0.to_string(),
                        binding.1,
                        binding.2,
                        binding.3.to_owned(),
                    ),
                    (
                        expected["origin"].as_str().unwrap().to_owned(),
                        String::from_utf8(canonical(&expected)).unwrap(),
                    ),
                ));
                *capture.lock().unwrap() = Some(expected);
                signal.store(true, Ordering::SeqCst);
                Box::pin(async {})
            })),
            Some(seams),
        );
        f.runtime.start().await.unwrap();
        wait_for(
            || arrived.load(Ordering::SeqCst),
            "SETUP: genuine retained activation input not reached",
        )
        .await;
        wait_for(
            || {
                count(&f, "workflow_native_contracts") == 1
                    || f.runtime._drivers.observe_finished().unwrap() == 0
            },
            "CA1 activation stage: composed contract absent",
        )
        .await;
        assert_eq!(
            count(&f, "workflow_native_contracts"),
            1,
            "CA1 activation stage: composed contract absent after actual worker exit"
        );
        let expected = expected.lock().unwrap().clone().unwrap();
        let contract: String = raw(&f)
            .query_row("SELECT body FROM workflow_native_contracts", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            contract,
            String::from_utf8(canonical(&expected)).unwrap(),
            "CA1 members and exact activation image"
        );
        let (record, workflow) = wait_bound(&f, &tasks[0]).await;
        assert!(
            atomic.load(Ordering::SeqCst),
            "CA1 atomic activation not observed"
        );
        {
            let store = f.owner.store.lock().unwrap();
            let windows = store.record_window_observations();
            assert_eq!(
                windows.len(),
                4,
                "CA1 Reserve/Claim/Complete/first-Executor windows"
            );
            for (index, &(id, old_version, planned_at, actual_version, actual_at, consumed)) in
                windows.iter().enumerate()
            {
                assert_eq!(id, record.id);
                assert_eq!(
                    old_version,
                    index as u64 + 1,
                    "CA1 original persisted previous version"
                );
                assert_eq!(
                    actual_version,
                    old_version + 1,
                    "CA1 consecutive Record version"
                );
                assert_eq!(actual_at, planned_at, "CA1 original planned timestamp");
                assert_eq!(consumed, 1, "CA1 actual records ensure_consumed event");
            }
        }
        let stored = f
            .owner
            .store
            .lock()
            .unwrap()
            .task(tasks[0].id)
            .unwrap()
            .unwrap();
        let (task_version, workflow_version): (u64, u64) = raw(&f)
            .query_row(
                "SELECT marker_task_version,marker_workflow_version FROM managed_phase_operations",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(
            stored.version, task_version,
            "CA1 record-only binder changed Task"
        );
        assert_eq!(
            record.version,
            workflow_version + 1,
            "CA1 binder record version"
        );
        assert_eq!(
            workflow_version, 6,
            "CA1 initial + QUICK gate Reserve/Claim/Complete + Executor reservation + marker"
        );
        let advances:u64=raw(&f).query_row("SELECT count(*) FROM audit WHERE kind='rrx.private.runtime.driver_initial_input_advanced'",[],|r|r.get(0)).unwrap();
        assert_eq!(
            advances, 5,
            "CA1 activation, Reserve/Claim/Complete and Executor record windows"
        );
        assert_eq!(
            count(&f, "native_invocations"),
            1,
            "CA1 actual registered launch"
        );
        let after: String = raw(&f)
            .query_row("SELECT body FROM workflow_native_contracts", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            after, contract,
            "CA1 immutable contract changed at marker/bind"
        );
        release_peer(&f, &workflow);
        finish(f).await;
    }
}

/// Separate stage control: activation's asserting observer would detect an
/// absent contract before the first Reserve, masking the permit-window cause.
/// This follows the identical genuine producer chain without that observer.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca1_gate_and_first_executor_windows_reach_original_bound() {
    for provider in ["claude", "codex"] {
        let mut f = fixture(provider, true);
        f.register_real_git_project();
        let (_, tasks) = accept(&f, 1).await;
        let (observations, release) = install_pause(&f, None);
        f.runtime.start().await.unwrap();
        wait_for(
            || observations.lock().unwrap().len() == 1,
            "SETUP: genuine retained activation S1 absent",
        )
        .await;
        release.add_permits(1);
        wait_for(
            || {
                let complete = f
                    .owner
                    .store
                    .lock()
                    .unwrap()
                    .record_window_observations()
                    .len()
                    == 4;
                complete || f.runtime._drivers.observe_finished().unwrap() == 0
            },
            "CA1 gate/first-Executor stage did not settle",
        )
        .await;
        {
            let store = f.owner.store.lock().unwrap();
            let windows = store.record_window_observations();
            assert_eq!(
                windows.len(),
                4,
                "CA1 gate/first-Executor stage: prescribed Record windows absent after actual worker exit"
            );
            for (index, &(_, before, planned_at, after, actual_at, consumed)) in
                windows.iter().enumerate()
            {
                assert_eq!(
                    before,
                    index as u64 + 1,
                    "CA1 original persisted gate previous version"
                );
                assert_eq!(after, before + 1, "CA1 gate consecutive Record version");
                assert_eq!(actual_at, planned_at, "CA1 gate original planned timestamp");
                assert_eq!(consumed, 1, "CA1 gate actual records ensure_consumed event");
            }
        }
        let (_, workflow) = wait_bound(&f, &tasks[0]).await;
        release_peer(&f, &workflow);
        finish(f).await;
    }
}
