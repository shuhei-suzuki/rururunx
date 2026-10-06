//! Effect-free descriptor negatives only. No prepared owner, Driver, marker,
//! successful Session or positive Native availability is fabricated here.
use super::*;
use crate::adapter::{AdapterError, AdapterFuture, AdapterResult, AgentInfo, ErrorKind};
use std::{
    collections::BTreeSet,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

struct Descriptor {
    advertised: BTreeSet<Capability>,
    info: AgentInfo,
    probe_fails: bool,
    capability_calls: AtomicUsize,
    probes: AtomicUsize,
}
impl AgentAdapter for Descriptor {
    fn capabilities(&self) -> BTreeSet<Capability> {
        self.capability_calls.fetch_add(1, Ordering::SeqCst);
        self.advertised.clone()
    }
    fn probe(&self) -> AdapterResult<AgentInfo> {
        self.probes.fetch_add(1, Ordering::SeqCst);
        if self.probe_fails {
            return Err(AdapterError {
                kind: ErrorKind::UnsupportedCapability,
                message: "controlled metadata probe failure".into(),
            });
        }
        Ok(self.info.clone())
    }
    fn start(&self, _: LaunchRequest) -> AdapterFuture<'_, Session> {
        panic!("descriptor cannot start")
    }
    fn status(&self, _: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        panic!("descriptor has no Session")
    }
    fn stop(&self, _: SessionRef) -> AdapterFuture<'_, SessionStatus> {
        panic!("descriptor has no Session")
    }
    fn attach(&self, _: SessionRef) -> AdapterFuture<'_, ()> {
        panic!("descriptor has no Session")
    }
    fn resume(&self, _: SessionRef) -> AdapterFuture<'_, Session> {
        panic!("descriptor cannot resume")
    }
    fn release(&self, _: SessionRef) -> AdapterResult<()> {
        panic!("descriptor has no Session")
    }
    fn subscribe(
        &self,
        _: SessionRef,
    ) -> AdapterResult<tokio::sync::watch::Receiver<SessionStatus>> {
        panic!("descriptor has no Session")
    }
    fn usage(&self, _: SessionRef, _: String, _: Option<u32>) -> AdapterFuture<'_, Usage> {
        panic!("descriptor has no Session")
    }
}
struct NoSources;
impl WorkflowSources for NoSources {
    fn capture(
        &self,
        _: Project,
        _: Task,
        _: Phase,
        _: ContextBudget,
    ) -> WorkflowFuture<'_, SourceSnapshot> {
        panic!("preflight must not capture sources")
    }
}
fn engine(descriptor: Option<Arc<Descriptor>>) -> WorkflowEngine {
    let mut registry = AgentRegistry::default();
    if let Some(descriptor) = descriptor {
        registry.register("selected".into(), descriptor).unwrap();
    }
    WorkflowEngine::new(
        Arc::new(Mutex::new(Store::memory().unwrap())),
        Arc::new(registry),
        Config::default(),
        Arc::new(NoSources),
        Arc::new(PendingGates),
    )
    .unwrap()
}
fn task() -> Task {
    let mut task = Task::new(
        ProjectId::new(),
        GoalId::new(),
        "descriptor negative".into(),
        "selected".into(),
    );
    task.reviewers = vec!["selected".into()];
    task
}
fn descriptor(role: Capability) -> Descriptor {
    let caps = BTreeSet::from([role, Capability::PreparedInputAdmission]);
    Descriptor {
        advertised: caps.clone(),
        info: AgentInfo {
            agent: "selected".into(),
            provider: "controlled-descriptor".into(),
            adapter_version: "test".into(),
            executable: "not-executed".into(),
            authenticated: None,
            model_configuration: false,
            effort_configuration: false,
            capabilities: caps,
        },
        probe_fails: false,
        capability_calls: AtomicUsize::new(0),
        probes: AtomicUsize::new(0),
    }
}
fn refusal(engine: &WorkflowEngine, task: &Task, phase: Phase) -> anyhow::Error {
    engine
        .preflight_native_adapter(task, phase)
        .err()
        .expect("metadata must never authorize a Native phase")
}
#[test]
fn preflight_missing_actor_or_adapter_is_configuration_refusal() {
    let engine = engine(None);
    let mut task = task();
    task.reviewers.clear();
    assert_eq!(
        refusal(&engine, &task, Phase::ImplementationReview)
            .downcast_ref::<NativePreflightRefusal>(),
        Some(&NativePreflightRefusal::MissingReviewer)
    );
    let error = refusal(&engine, &task, Phase::Implement);
    assert_eq!(
        error.downcast_ref::<NativePreflightRefusal>(),
        Some(&NativePreflightRefusal::AdapterUnavailable)
    );
    assert_eq!(
        error.downcast_ref::<AdapterError>().unwrap().kind,
        ErrorKind::InvalidConfiguration
    );
}
#[test]
fn preflight_descriptor_negatives_do_not_admit_either_native_role() {
    for (phase, role) in [
        (Phase::Implement, Capability::Execute),
        (Phase::ImplementationReview, Capability::Review),
    ] {
        for case in 0..8 {
            let mut d = descriptor(role);
            match case {
                0 => {
                    d.advertised.remove(&role);
                }
                1 => {
                    d.advertised.remove(&Capability::PreparedInputAdmission);
                }
                2 => {
                    d.probe_fails = true;
                }
                3 => {
                    d.info.agent = "foreign".into();
                }
                4 => {
                    d.info.provider = "  ".into();
                }
                5 => {
                    d.info.capabilities.remove(&role);
                }
                6 => {
                    d.info
                        .capabilities
                        .remove(&Capability::PreparedInputAdmission);
                }
                7 => {}
                _ => unreachable!(),
            }
            let d = Arc::new(d);
            let engine = engine(Some(d.clone()));
            let task = task();
            let before = serde_json::to_vec(&task).unwrap();
            for _ in 0..2 {
                let error = refusal(&engine, &task, phase);
                assert_eq!(
                    error.downcast_ref::<NativePreflightRefusal>(),
                    Some(&NativePreflightRefusal::ManagedBindingUnavailable),
                    "role {role:?}, case {case}: {error:#}"
                );
            }
            assert_eq!(d.capability_calls.load(Ordering::SeqCst), 0);
            assert_eq!(d.probes.load(Ordering::SeqCst), 0);
            assert_eq!(serde_json::to_vec(&task).unwrap(), before);
        }
    }
}
#[test]
fn preflight_all_native_phases_keep_private_composition_unavailable() {
    for phase in [
        Phase::Requirements,
        Phase::Design,
        Phase::Implement,
        Phase::ImpactAnalysis,
        Phase::RequirementsReview,
        Phase::DesignReview,
        Phase::ImplementationReview,
        Phase::SecurityReview,
    ] {
        let role = if phase.actor() == Actor::Executor {
            Capability::Execute
        } else {
            Capability::Review
        };
        let descriptor = Arc::new(descriptor(role));
        let engine = engine(Some(descriptor.clone()));
        assert_eq!(
            refusal(&engine, &task(), phase).downcast_ref::<NativePreflightRefusal>(),
            Some(&NativePreflightRefusal::ManagedBindingUnavailable)
        );
        assert_eq!(descriptor.capability_calls.load(Ordering::SeqCst), 0);
        assert_eq!(descriptor.probes.load(Ordering::SeqCst), 0);
    }
    assert_eq!(
        require_managed_native_binding_composed()
            .unwrap_err()
            .downcast_ref::<NativePreflightRefusal>(),
        Some(&NativePreflightRefusal::ManagedBindingUnavailable)
    );
}
