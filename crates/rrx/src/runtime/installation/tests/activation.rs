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

#[derive(Clone)]
struct ObservedActivation {
    plan: Arc<crate::state::DriverPreparationAdvance>,
    association: Arc<crate::runtime::driver::DriverAssociation>,
    expected: serde_json::Value,
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
        engine(&f).set_activation_hooks(
            Some(Arc::new(move |probe| {
                *capture.lock().unwrap() = Some(expected_contract(&probe));
                signal.store(true, Ordering::SeqCst);
                Box::pin(async {})
            })),
            None,
        );
        f.runtime.start().await.unwrap();
        wait_for(
            || arrived.load(Ordering::SeqCst),
            "SETUP: genuine retained activation input not reached",
        )
        .await;
        wait_for(
            || count(&f, "workflow_native_contracts") == 1,
            "CA1 activation stage: composed contract absent",
        )
        .await;
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
