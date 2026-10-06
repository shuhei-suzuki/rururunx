//! Nongrant ingress ownership controls. These create EMPTY capacity only, never
//! Driver tickets, Source allocations, marker rows or Native preparation proof.
use super::*;
use crate::{
    adapter::AgentRegistry,
    config::{AgentConfig, Config},
    runtime::Runtime,
};

#[tokio::test]
async fn original_empty_reservation_does_not_retain_queue_or_jobs() {
    let dir = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
    let mut config = Config::default();
    config.agents.insert(
        "selected".into(),
        AgentConfig {
            provider: Some("codex".into()),
            command: vec!["/usr/bin/false".into()],
            ..Default::default()
        },
    );
    let agents = AgentRegistry::from_managed_config(&config, owner.clone()).unwrap();
    let selected = agents.native_phase_port("selected").unwrap();
    let runtime = Arc::new(Runtime::new(owner, config).unwrap());
    runtime.start().await.unwrap();
    let phases = Arc::downgrade(&runtime.phases);
    let jobs = Arc::downgrade(&runtime.phase_jobs);
    let dispatcher = Arc::downgrade(&runtime.phase_dispatcher);
    let ingress = runtime.phase_handoffs.clone();
    // This actual constructor creates EMPTY bookkeeping; the test does not
    // claim the ticket-checked ingress/Source positive route was reached.
    let task = TaskId::new();
    let reservation = ingress
        .reserve_empty(task, runtime.source_consumer(&selected))
        .unwrap();
    assert_eq!(
        ingress.observe(task).unwrap().unwrap().state(),
        SourceHandoffState::AwaitingOffer
    );
    // The still-owned original EMPTY prevents a false shutdown completion.
    let error = runtime.shutdown().await.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Source handoff shutdown remains pending")
    );
    drop(runtime);
    assert!(
        dispatcher.upgrade().is_none(),
        "EMPTY reservation retained actual dispatcher"
    );
    assert!(
        phases.upgrade().is_none(),
        "EMPTY reservation retained actual supervisor"
    );
    assert!(
        jobs.upgrade().is_none(),
        "EMPTY reservation retained actual jobs"
    );
    drop(reservation);
    assert!(ingress.observe(task).unwrap().is_none());
    ingress.ensure_shutdown_complete().unwrap();
}

#[test]
fn poisoned_ingress_lookup_is_not_absence() {
    let ingress = Arc::new(PhaseHandoffs::default());
    let original = ingress.clone();
    let thread = std::thread::spawn(move || {
        let _entries = original.entries.lock().unwrap();
        panic!("intentional original ingress poison");
    });
    assert!(thread.join().is_err());
    assert!(
        ingress.observe(TaskId::new()).is_err(),
        "poison became absence and permitted a new offer"
    );
    assert!(ingress.ensure_shutdown_complete().is_err());
}
