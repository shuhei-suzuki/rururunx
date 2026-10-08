use super::*;
use crate::{
    adapter::{AgentRegistry, InputKind, PreparedInput},
    config::{AgentConfig, Config, WorkflowClass},
    domain::{RiskClass, Task},
    execution::{ExecutionUnit, attempts::AttemptManager, native::ManagedInput},
    project::{AddProject, ProjectRegistry},
    runtime::{
        Runtime,
        control::{ControlAction, ControlRequest, ControlResponse},
        goal::*,
    },
};
use std::{collections::BTreeMap, os::unix::fs::PermissionsExt, path::Path};

/// Actual trusted ingress, legacy selected allocation and EMPTY jobs only.
/// No accepted Source, marker, prepared Native input or installed issuer.
#[tokio::test]
async fn actual_empty_job_reservation_removes_only_its_fresh_original() {
    let (f, groups) = Fixture::new(4, &[("empty-reservation", 1)]).await;
    let tasks = groups.into_iter().next().unwrap();
    let (allocation, _guard, unit) = f.allocation(&tasks[0], "codex").await;
    let allocation = Arc::new(allocation);
    let jobs = &f.runtime.phase_jobs;
    let fresh = jobs.reserve(&allocation).unwrap();
    let reused = jobs.reserve(&allocation).unwrap();
    let original = jobs.preparation_custody(&fresh).unwrap();
    assert!(Arc::ptr_eq(
        &original,
        &jobs.preparation_custody(&reused).unwrap()
    ));
    assert!(
        !jobs.remove_unstarted(&reused).unwrap(),
        "reuse removed original job"
    );
    jobs.ready(&fresh).unwrap();
    assert!(Arc::ptr_eq(
        &original,
        &jobs.preparation_custody(&fresh).unwrap()
    ));
    let foreign_jobs = crate::runtime::phase_jobs::PhaseJobs::default();
    let foreign = foreign_jobs.reserve(&allocation).unwrap();
    assert!(
        jobs.remove_unstarted(&foreign).is_err(),
        "foreign entry token removed same-ID job"
    );
    jobs.ready(&fresh).unwrap();
    assert!(jobs.remove_unstarted(&fresh).unwrap());
    assert!(jobs.ready(&reused).is_err());
    jobs.ensure_shutdown_complete().unwrap();
    assert!(foreign_jobs.remove_unstarted(&foreign).unwrap());
    f.unchanged(&unit);
    f.runtime.shutdown().await.unwrap();
}

/// Nongrant policy-only control on an actual Unit/guard. Calling the retention
/// policy here does not fabricate the actual Source seal or qualify acceptance.
#[tokio::test]
async fn nongrant_guard_policy_is_one_way_across_restore_and_final_drop() {
    let (f, groups) = Fixture::new(4, &[("guard-policy", 1)]).await;
    let tasks = groups.into_iter().next().unwrap();
    let (_allocation, mut guard, unit) = f.allocation(&tasks[0], "codex").await;
    guard.hold_accepted_source();
    guard.hold_marker_publication();
    guard.restore_unmarked_retirement();
    drop(guard);
    f.unchanged(&unit);
    f.runtime.shutdown().await.unwrap();
}

/// C-S1d P4 (nongrant): an accepted-policy original keeps shutdown pending,
/// typed at its site, without claiming the Source seal was reached.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn c_s1d_p4_accepted_original_stop_is_pending() {
    let (f, groups) = Fixture::new(4, &[("accepted-original", 1)]).await;
    let tasks = groups.into_iter().next().unwrap();
    let (allocation, guard, unit) = f.allocation(&tasks[0], "codex").await;
    let capacity = f
        .runtime
        .phases
        .reserve_source(allocation, guard)
        .map_err(|_| "SETUP: accepted original refused")
        .unwrap();
    assert_eq!(
        crate::runtime::tests::stop_pending_site(&f.runtime).await,
        crate::runtime::stop::ShutdownSite::AcceptedOriginals
    );
    f.unchanged(&unit);
    drop(capacity);
}

struct Fixture {
    _dir: tempfile::TempDir,
    owner: Arc<RuntimeOwner>,
    runtime: Arc<Runtime>,
    registry: AgentRegistry,
    counter: std::path::PathBuf,
}
async fn git(path: &Path, args: &[&str]) {
    let result = tokio::process::Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .await
        .unwrap();
    assert!(
        result.status.success(),
        "Git fixture setup failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}
/// One accepted-ingress Goal (`count` Strict codex Tasks) on its own Git
/// Project `<dir>/<name>`.
async fn create(
    dir: &Path,
    owner: &Arc<RuntimeOwner>,
    runtime: &Arc<Runtime>,
    config: &Config,
    name: &str,
    count: usize,
) -> crate::domain::GoalId {
    let root = dir.join(name);
    std::fs::create_dir(&root).unwrap();
    git(&root, &["init", "-b", "main"]).await;
    std::fs::write(root.join("base.txt"), "original\n").unwrap();
    git(&root, &["add", "base.txt"]).await;
    git(
        &root,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            "fixture",
        ],
    )
    .await;
    let project = {
        let mut store = owner.store.lock().unwrap();
        ProjectRegistry::new(&mut store)
            .add(&root, AddProject::default(), config)
            .unwrap()
    };
    let (socket, _peer) = tokio::net::UnixStream::pair().unwrap();
    let result = runtime
        .handle_control(
            &socket,
            ControlRequest {
                request_id: uuid::Uuid::new_v4(),
                instance: owner.instance_id().into(),
                epoch: owner.epoch(),
                action: ControlAction::CreateGoal {
                    project: project.id,
                    expected_project: project.version,
                    plan: GoalPlan {
                        definition: GoalDefinition {
                            title: name.into(),
                            objective: "retained nongrant queue".into(),
                            criteria: vec![CriterionDefinition {
                                id: "exact".into(),
                                description: "same original allocation".into(),
                                evaluator: CriterionEvaluator::RequiredTasksVerified,
                            }],
                            constraints: vec![],
                            non_goals: vec![],
                            source_refs: vec![],
                        },
                        tasks: (0..count)
                            .map(|n| TaskDefinition {
                                key: format!("task{n}"),
                                title: format!("task{n}"),
                                acceptance_criteria: vec!["exact original".into()],
                                executor: "codex".into(),
                                reviewers: vec![],
                                workflow: WorkflowClass::Strict,
                                risk: RiskClass::R3,
                            })
                            .collect(),
                        dependencies: vec![],
                    },
                },
            },
        )
        .await
        .unwrap();
    let ControlResponse::GoalAccepted { goal, .. } = result else {
        panic!("genuine accepted ingress unavailable")
    };
    goal
}
impl Fixture {
    async fn new(global: usize, groups: &[(&str, usize)]) -> (Self, Vec<Vec<Task>>) {
        let dir = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::open(&dir.path().join("state.db")).unwrap();
        let counter = dir.path().join("native-must-not-run");
        let program = dir.path().join("native-counter");
        std::fs::write(
            &program,
            format!(
                "#!/bin/sh\nprintf invoked > '{}'\nexit 7\n",
                counter.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut config = Config::default();
        config.scheduler.global_max_sessions = global;
        for provider in ["claude", "codex"] {
            config.agents.insert(
                provider.into(),
                AgentConfig {
                    provider: Some(provider.into()),
                    command: vec![program.to_string_lossy().into()],
                    ..Default::default()
                },
            );
        }
        let ingress = Arc::new(Runtime::new(owner.clone(), config.clone()).unwrap());
        // FM §8.5 L: every Project and Goal comes from accepted ingress on
        // one database first; only then is it migrated (§8.1) and the
        // Runtime opened over the migrated legacy rows.
        let mut goals = vec![];
        for (name, count) in groups {
            goals.push(create(dir.path(), &owner, &ingress, &config, name, *count).await);
        }
        let owner = crate::runtime::migrate_legacy(dir.path(), owner, ingress, "state.db").await;
        let tasks = {
            let store = owner.store.lock().unwrap();
            goals
                .into_iter()
                .map(|goal| {
                    store
                        .goal(goal)
                        .unwrap()
                        .unwrap()
                        .dag
                        .nodes
                        .iter()
                        .map(|id| store.task(*id).unwrap().unwrap())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        };
        let registry = AgentRegistry::from_managed_config(&config, owner.clone()).unwrap();
        let runtime = Arc::new(Runtime::new(owner.clone(), config.clone()).unwrap());
        assert!(
            runtime.installed.is_err(),
            "separate registry must refuse installation"
        );
        runtime.start().await.unwrap();
        (
            Self {
                _dir: dir,
                owner,
                runtime,
                registry,
                counter,
            },
            tasks,
        )
    }
    async fn allocation(
        &self,
        task: &Task,
        provider: &str,
    ) -> (NativeAllocation, PreparationGuard, ExecutionUnit) {
        let (unit, _) = AttemptManager::new(self.owner.clone())
            .prepare(task.id, provider, "Implement", None)
            .await
            .unwrap();
        let allocation = self
            .registry
            .native_phase_port(provider)
            .unwrap()
            .allocate(
                ManagedInput {
                    agent: provider.into(),
                    authority: unit.authority(),
                    artifact: None,
                    input: PreparedInput {
                        scope: unit.scope.clone(),
                        kind: InputKind::ContextPack,
                        revision: unit.base_sha.clone(),
                        version: 1,
                        source_versions: BTreeMap::new(),
                        payload: "exact original payload".into(),
                    },
                },
                None,
                None,
            )
            .unwrap();
        let preparation = PreparationGuard::new(self.owner.clone(), &unit);
        (allocation, preparation, unit)
    }
    async fn reserve(&self, task: &Task, provider: &str) -> (PendingPhaseCapacity, ExecutionUnit) {
        let (allocation, guard, unit) = self.allocation(task, provider).await;
        let capacity = self
            .runtime
            .reserve_pending_phase(allocation, guard)
            .await
            .unwrap();
        (capacity, unit)
    }
    fn unchanged(&self, unit: &ExecutionUnit) {
        let store = self.owner.store.lock().unwrap();
        let current = store.execution_unit(unit.id).unwrap();
        assert_eq!(
            serde_json::to_value(current).unwrap(),
            serde_json::to_value(unit).unwrap()
        );
        assert!(
            store
                .records(&unit.scope, crate::domain::RecordKind::Session)
                .unwrap()
                .is_empty()
        );
        assert!(
            !self.counter.exists(),
            "retention executed a Native helper/start"
        );
    }
}

#[tokio::test]
async fn actual_selected_allocations_survive_caller_drop_and_guard_is_drained_on_shutdown() {
    let (f, groups) = Fixture::new(4, &[("project", 2)]).await;
    let tasks = groups.into_iter().next().unwrap();
    let (capacity, unit) = f.reserve(&tasks[0], "claude").await;
    let original = capacity.allocation().facts().operation_id;
    assert!(capacity.is_retained());
    assert_eq!(
        capacity.observation().unwrap(),
        PendingObservation::PendingMarker
    );
    f.unchanged(&unit);
    drop(capacity);
    let page = f.runtime.phases.fair_page().unwrap();
    assert_eq!(
        page.len(),
        1,
        "Engine handle Drop removed Runtime-owned pending allocation"
    );
    assert_eq!(page[0].allocation.facts().operation_id, original);
    f.unchanged(&unit);
    // A nongrant Arc read remains live during shutdown; its armed guard must
    // still be removed from the slot rather than wait for this reader to Drop.
    let read = page[0].allocation.clone();
    let (second, second_unit) = f.reserve(&tasks[1], "codex").await;
    f.runtime.shutdown().await.unwrap();
    assert!(!second.is_retained());
    assert_eq!(second.observation().unwrap(), PendingObservation::Stopping);
    assert!(f.runtime.phases.fair_page().unwrap().is_empty());
    let store = f.owner.store.lock().unwrap();
    for id in [unit.id, second_unit.id] {
        assert!(
            !store.execution_unit(id).unwrap().native_effects_open,
            "unmarked armed preparation survived shutdown"
        );
    }
    assert_eq!(read.facts().operation_id, original);
    assert!(!f.counter.exists());
}

#[tokio::test]
async fn actual_capacity_refusal_returns_original_objects_and_other_project_still_admits() {
    // O2: the per-Project bound counts pending operations, PHASE_SLOTS_PER_PROJECT
    // of them, with global headroom; one more global slot is left for b.
    let slots = super::PHASE_SLOTS_PER_PROJECT;
    let (f, groups) = Fixture::new(slots + 1, &[("a", slots + 1), ("b", 1), ("c", 1)]).await;
    let [a, b, c]: [Vec<Task>; 3] = groups.try_into().unwrap();
    let (first, first_unit) = f.reserve(&a[0], "claude").await;
    let mut held = Vec::new();
    for task in &a[1..slots] {
        held.push(f.reserve(task, "claude").await);
    }
    let (allocation, preparation, rejected_unit) = f.allocation(&a[slots], "codex").await;
    let original = allocation.facts().operation_id;
    let refused = f
        .runtime
        .reserve_pending_phase(allocation, preparation)
        .await
        .err()
        .expect("per-Project pending capacity bypassed");
    assert_eq!(refused.allocation.facts().operation_id, original);
    assert!(
        refused
            .preparation
            .matches(&f.owner, &rejected_unit)
            .unwrap()
    );
    f.unchanged(&first_unit);
    f.unchanged(&rejected_unit);
    let (other, other_unit) = f.reserve(&b[0], "codex").await;
    let (allocation, preparation, global_unit) = f.allocation(&c[0], "claude").await;
    let global_refused = f
        .runtime
        .reserve_pending_phase(allocation, preparation)
        .await
        .err()
        .expect("global pending capacity bypassed");
    f.unchanged(&global_unit);
    first.abandon_unmarked().unwrap();
    f.unchanged(&other_unit);
    assert!(other.is_retained());
    let resumed = f
        .runtime
        .reserve_pending_phase(refused.allocation, refused.preparation)
        .await
        .unwrap();
    assert_eq!(
        resumed.allocation().facts().operation_id,
        original,
        "capacity retry replaced original operation"
    );
    assert!(resumed.is_retained());
    drop(global_refused);
    drop(held);
    f.runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn fair_actual_pending_pages_rotate_projects_and_retain_owner_change_as_hold() {
    let (f, groups) = Fixture::new(8, &[("many", 3), ("sibling", 1)]).await;
    let [a, b]: [Vec<Task>; 2] = groups.try_into().unwrap();
    let mut handles = Vec::new();
    for task in &a {
        handles.push(f.reserve(task, "codex").await.0);
    }
    let (sibling, unit) = f.reserve(&b[0], "claude").await;
    let p = f.runtime.phases.fair_page().unwrap();
    assert_eq!(p.len(), 4);
    assert_ne!(
        p[0].allocation.facts().scope.project_id,
        p[1].allocation.facts().scope.project_id,
        "one Project monopolized fair sweep"
    );
    let prior = p[0].allocation.facts().operation_id;
    let next = f.runtime.phases.fair_page().unwrap();
    assert_ne!(
        prior,
        next[0].allocation.facts().operation_id,
        "same-Project cursor did not advance"
    );
    f.owner
        .store
        .lock()
        .unwrap()
        .retire_execution_as(
            &unit.authority(),
            false,
            crate::execution::Disposition::Cancelled,
        )
        .unwrap();
    f.runtime.phases.reconcile_pending().unwrap();
    assert_eq!(
        sibling.observation().unwrap(),
        PendingObservation::HeldOwnerChanged
    );
    assert!(
        sibling.is_retained(),
        "owner drift discarded pending ownership"
    );
    for handle in &handles {
        assert_eq!(
            handle.observation().unwrap(),
            PendingObservation::PendingMarker
        );
    }
    f.runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn foreign_preparation_and_stopped_service_refuse_without_consuming_inputs() {
    let (f, groups) = Fixture::new(4, &[("project", 3)]).await;
    let tasks = groups.into_iter().next().unwrap();
    let (allocation, right, unit) = f.allocation(&tasks[0], "claude").await;
    let (_, wrong, other) = f.allocation(&tasks[1], "codex").await;
    let refused = f
        .runtime
        .reserve_pending_phase(allocation, wrong)
        .await
        .err()
        .expect("foreign prepared Unit accepted");
    assert_eq!(refused.allocation.facts().unit_id, unit.id);
    f.unchanged(&unit);
    f.unchanged(&other);
    let capacity = f
        .runtime
        .reserve_pending_phase(refused.allocation, right)
        .await
        .unwrap();
    drop(refused.preparation);
    // Prepare the exact stopped-admission pair while the service can still
    // dispatch. Shutdown must not require a fresh preparation or allocation.
    let (allocation, guard, stopped_unit) = f.allocation(&tasks[2], "codex").await;
    let facts = allocation.facts();
    let identity = (
        facts.operation_id,
        facts.session_id,
        facts.invocation_id,
        facts.pair_id,
    );
    let input = facts.input_bytes.to_vec();
    f.runtime.shutdown().await.unwrap();
    assert!(!capacity.is_retained());
    let refused = f
        .runtime
        .reserve_pending_phase(allocation, guard)
        .await
        .err()
        .expect("stopped Runtime accepted pending work");
    let facts = refused.allocation.facts();
    assert_eq!(
        (
            facts.operation_id,
            facts.session_id,
            facts.invocation_id,
            facts.pair_id,
        ),
        identity
    );
    assert_eq!(facts.input_bytes, input);
    assert!(
        refused
            .preparation
            .matches(&f.owner, &stopped_unit)
            .unwrap()
    );
    f.unchanged(&stopped_unit);
    assert!(f.runtime.phases.fair_page().unwrap().is_empty());
    assert!(!f.counter.exists());
}

#[test]
fn pending_fallback_is_finite_and_idle_resets_backoff() {
    let mut step = 0;
    for expected in [100, 200, 400, 800, 1600, 3200, 5000, 5000, 5000] {
        assert_eq!(
            PhaseSupervisor::delay(true, &mut step),
            Duration::from_millis(expected)
        );
    }
    assert_eq!(
        PhaseSupervisor::delay(false, &mut step),
        Duration::from_secs(5)
    );
    assert_eq!(
        PhaseSupervisor::delay(true, &mut step),
        Duration::from_millis(100)
    );
}

#[tokio::test]
async fn actual_publication_drop_and_shutdown_preserve_unknown_slot_until_proven_rollback() {
    let (f, groups) = Fixture::new(4, &[("publication", 3)]).await;
    let tasks = groups.into_iter().next().unwrap();
    let (capacity, unit) = f.reserve(&tasks[0], "claude").await;
    let operation = capacity.allocation().facts().operation_id;
    let publication = f.runtime.retain_marker_publication(capacity).await.unwrap();
    assert!(publication.is_retained());
    drop(publication);
    f.runtime.phases.reconcile_pending().unwrap();
    let page = f.runtime.phases.fair_page().unwrap();
    let slot = page
        .iter()
        .find(|s| s.allocation.facts().operation_id == operation)
        .unwrap();
    assert_eq!(
        *slot.observation.lock().unwrap(),
        PendingObservation::MarkerPublicationPending
    );
    assert!(f.runtime.phases.remove_unmarked(slot).is_err());
    f.unchanged(&unit);

    let (rollback, rollback_unit) = f.reserve(&tasks[1], "codex").await;
    let rollback_operation = rollback.allocation().facts().operation_id;
    let rollback = f.runtime.retain_marker_publication(rollback).await.unwrap();
    let (unmarked, unmarked_unit) = f.reserve(&tasks[2], "claude").await;
    // Retained publication custody keeps shutdown pending (12304a9): the
    // shutdown reports it instead of claiming completion or releasing it.
    // C-S1d P5: through RuntimeStop, typed at the retained-jobs site.
    assert_eq!(
        crate::runtime::tests::stop_pending_site(&f.runtime).await,
        crate::runtime::stop::ShutdownSite::NativeJobs
    );
    assert!(!unmarked.is_retained());
    assert!(rollback.is_retained());
    assert!(f.runtime.phases.contains(slot));
    f.unchanged(&unit);
    f.unchanged(&rollback_unit);
    assert!(
        !f.owner
            .store
            .lock()
            .unwrap()
            .execution_unit(unmarked_unit.id)
            .unwrap()
            .native_effects_open
    );
    let returned = f
        .runtime
        .rollback_marker_publication(rollback)
        .await
        .unwrap();
    assert_eq!(
        returned.allocation().facts().operation_id,
        rollback_operation
    );
    assert!(
        !returned.is_retained(),
        "stopped supervisor re-admitted an unmarked slot"
    );
    assert!(
        !f.owner
            .store
            .lock()
            .unwrap()
            .execution_unit(rollback_unit.id)
            .unwrap()
            .native_effects_open
    );
    f.unchanged(&unit);
    assert!(!f.counter.exists());
    // Final Runtime/queue Drop is also not evidence of non-publication. Keep
    // the actual owner and temporary DB alive while releasing every slot read.
    let owner = f.owner.clone();
    drop(page);
    drop(f.runtime);
    assert!(
        owner
            .store
            .lock()
            .unwrap()
            .execution_unit(unit.id)
            .unwrap()
            .native_effects_open,
        "final supervisor Drop retired an uncertain publishing Unit"
    );
}
