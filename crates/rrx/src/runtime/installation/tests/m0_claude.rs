//! CL-10 through installed ingress -> genuine preparation -> Session -> Core
//! -> terminal capture and atomic closure. These peers are synthetic protocol
//! controls, never evidence of CLI compatibility or startup/P qualification.
use super::*;
use crate::adapter::{AgentAdapter, SessionRef};
use crate::domain::SessionState;
use crate::execution::native::{ManagedSessionRef, NativeStatus};
use crate::execution::native_result::AcquisitionStatus;
use crate::execution::{Disposition, NativeFailure, UnitState, WorkOutcome};

fn status(f: &ControlFixture, handle: &ManagedSessionRef) -> NativeStatus {
    f.runtime
        .installed
        .as_ref()
        .ok()
        .unwrap()
        .registry
        .native_phase_port("worker")
        .unwrap()
        .selected_adapter()
        .unwrap()
        .sessions
        .status(handle)
        .unwrap()
}

async fn run_scenario(scenario: &str, invalid: bool, work: WorkOutcome, unfinished: bool) {
    run_scenario_until_receipt(scenario, invalid, work, unfinished, invalid).await;
}

async fn run_scenario_until_receipt(
    scenario: &str,
    invalid: bool,
    work: WorkOutcome,
    unfinished: bool,
    await_failure: bool,
) {
    let mut f = fixture_mode("claude", true, Some(scenario), |_| {});
    f.register_real_git_project();
    assert!(f.runtime.installed.is_ok());
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    f.runtime.start().await.unwrap();
    success::wait_normal_bound(&f, task).await;
    let unit = success::bound_unit(&f, task);
    // Weak observations borrow the genuine job originals without becoming
    // their custody owner or fabricating authority from the copied Task/Unit.
    let custody = f.runtime.phase_jobs.observed_preparation(unit.id).unwrap();
    let allocation = Arc::downgrade(&f.runtime.phase_jobs.original_allocation(unit.id).unwrap());
    let (_, snapshot) = success::workflow(&f, task);
    let handle = snapshot
        .history
        .iter()
        .find_map(|attempt| attempt.execution.clone())
        .unwrap();
    success::release_unit(&f, &unit);
    if scenario == "m0-control-request-input" {
        success::wait_for(
            || {
                status(&f, &handle)
                    .pending
                    .iter()
                    .any(|p| p["request_id"] == "m0-permission")
            },
            "normal control request did not reach the real Pending consumer",
            10,
        )
        .await;
        let published = status(&f, &handle);
        assert_eq!(published.pending.len(), 1);
        assert_eq!(published.pending[0]["tool_use_id"], "m0-tool");
        assert_eq!(published.pending[0]["tool_name"], "Bash");
        assert!(published.pending[0].get("input").is_none());
        assert!(published.pending[0]["operation_hash"].as_str().is_some());
        assert!(published.receipt.is_none() && published.metrics.is_none());
        let profile = crate::execution::resources::ResourceManager::new(f.owner.clone())
            .profile(&unit)
            .unwrap();
        std::fs::write(profile.output.join("fixture-control-release"), "release").unwrap();
    }
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        let status = status(&f, &handle);
        if status.receipt.is_some()
            && (!await_failure || status.failure == Some(NativeFailure::ProtocolFailure))
        {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "{scenario}: terminal status {status:?}; jobs {:?}",
            f.runtime.phase_jobs.observed_jobs()
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let actual = status(&f, &handle);
    let usage = {
        let adapter = f
            .runtime
            .installed
            .as_ref()
            .ok()
            .unwrap()
            .registry
            .native_phase_port("worker")
            .unwrap()
            .selected_adapter()
            .unwrap();
        let mut reference = SessionRef::from(&actual.session);
        reference.execution = Some(handle.clone());
        adapter
            .usage(reference, "Implement".into(), None)
            .await
            .unwrap()
    };
    let receipt = f
        .owner
        .store
        .lock()
        .unwrap()
        .native_session_result(handle.session)
        .unwrap()
        .unwrap();
    assert_eq!(receipt.observed_work, work, "{scenario}");
    assert_eq!(actual.work, Some(work), "{scenario}");
    if scenario.starts_with("m0-rate-") {
        let observations = f
            .owner
            .store
            .lock()
            .unwrap()
            .quota_observations("claude", "unknown")
            .unwrap();
        if invalid || scenario.ends_with("-foreign") {
            assert!(
                observations.is_empty(),
                "malformed or foreign event cannot update the pool"
            );
        } else {
            assert_eq!(observations.len(), 1);
            assert_eq!(observations[0].source_version, "claude/rate_limit_event");
        }
    }
    if invalid {
        assert_eq!(actual.disposition, Disposition::ProtocolError, "{scenario}");
        assert_eq!(
            actual.failure,
            Some(NativeFailure::ProtocolFailure),
            "{scenario}"
        );
        assert_eq!(actual.session.state, SessionState::Lost, "{scenario}");
        let closed = f
            .owner
            .store
            .lock()
            .unwrap()
            .execution_unit(unit.id)
            .unwrap();
        assert_eq!(closed.state, UnitState::WorkUnknown, "{scenario}");
        assert!(
            !closed.native_effects_open,
            "no new input/grant after rejection"
        );
        assert!(
            actual.metrics.is_none(),
            "invalid frame metrics adoption must be zero"
        );
        assert_eq!(
            (
                usage.input_tokens,
                usage.output_tokens,
                usage.cached_input_tokens,
                usage.estimated_cost
            ),
            (None, None, None, None),
            "invalid telemetry cannot reach Usage"
        );
        assert!(usage.missing_reason.is_some());
        assert_eq!(
            crate::runtime::phase_jobs::counted(task.id, "claude confirm_complete"),
            0,
            "invalid/unfinished result cannot be confirmed"
        );
        assert_ne!(
            receipt.acquisition,
            AcquisitionStatus::Complete,
            "no confirm of invalid or unfinished answer"
        );
        assert!(receipt.text.is_none() && receipt.answer_sha256.is_none());
        if !unfinished {
            assert!(
                receipt.terminal_sha256.is_none(),
                "invalid result must not reach Collector"
            );
        }
        assert!(
            !success::links(&f, task).iter().any(|link| matches!(
                link.as_str(),
                "gate_claim" | "gate_observed" | "phase_closed"
            )),
            "no owned-success continuation"
        );
        // Explicit Waiting/Held task classification remains a later M0 unit:
        // current settlement_turn polls a nonsuccess terminal. Do not count
        // these assertions as proof that the Task consumer is already wired.
    } else {
        assert_eq!(
            actual.disposition,
            Disposition::Completed,
            "paired normal control must reach the terminal consumer"
        );
        assert_eq!(actual.failure, None);
        assert_eq!(
            crate::runtime::phase_jobs::counted(task.id, "claude confirm_complete"),
            1,
            "normal terminal reaches the real confirm consumer"
        );
        if work == WorkOutcome::Success {
            assert_eq!(receipt.acquisition, AcquisitionStatus::Complete);
            assert_eq!(receipt.text.as_deref(), Some("M0 validated answer"));
            assert!(actual.metrics.is_some());
            assert_eq!(
                usage.input_tokens,
                scenario.ends_with("-input").then_some(0)
            );
            assert_eq!(
                usage.output_tokens,
                scenario.ends_with("-output").then_some(0)
            );
            assert_eq!(
                usage.cached_input_tokens,
                scenario.ends_with("-read").then_some(0)
            );
            assert_eq!(
                usage.estimated_cost,
                scenario.ends_with("-cost").then_some(0.0)
            );
            assert!(usage.missing_reason.is_none());
        } else {
            assert!(actual.metrics.is_none());
            assert_eq!(
                (
                    usage.input_tokens,
                    usage.output_tokens,
                    usage.cached_input_tokens,
                    usage.estimated_cost
                ),
                (None, None, None, None)
            );
        }
    }
    // These wire controls stop at the terminal consumer. They do not produce
    // a committed Git artifact or the later M0 Task non-success closure.
    // Terminal capture/confirm alone cannot retire accepted Source originals.
    let stopped = f.runtime.shutdown().await;
    assert_eq!(
        stopped
            .as_ref()
            .err()
            .and_then(crate::runtime::stop::pending_site),
        Some(crate::runtime::stop::ShutdownSite::AcceptedOriginals),
        "{scenario}: premature Source retirement or unexpected shutdown: {stopped:?}"
    );
    let jobs = f.runtime.phase_jobs.observed_jobs();
    assert_eq!(
        jobs.len(),
        1,
        "{scenario}: genuine job must remain retained"
    );
    assert_eq!(jobs[0].unit, unit.id, "{scenario}: retained job changed");
    assert!(jobs[0].bound && !jobs[0].success_closed, "{scenario}");
    assert!(
        custody.upgrade().is_some(),
        "{scenario}: custody ended at stop"
    );
    assert!(
        std::sync::Weak::ptr_eq(
            &custody,
            &f.runtime.phase_jobs.observed_preparation(unit.id).unwrap()
        ),
        "{scenario}: original preparation custody replaced"
    );
    assert!(
        std::sync::Weak::ptr_eq(
            &allocation,
            &Arc::downgrade(&f.runtime.phase_jobs.original_allocation(unit.id).unwrap())
        ),
        "{scenario}: original allocation replaced"
    );
    finish(f).await;
    success::wait_for(
        || custody.upgrade().is_none() && allocation.upgrade().is_none(),
        &format!("{scenario}: original custody/allocation survived Runtime drop"),
        10,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn o03_installed_normal_control_responses_fail_closed_and_requests_remain_consumed() {
    for scenario in ["m0-control-success-input", "m0-control-request-input"] {
        run_scenario(scenario, false, WorkOutcome::Success, false).await;
    }
    for scenario in [
        "m0-control-success-response",
        "m0-control-error-response",
        "m0-control-initialize-response",
    ] {
        // Receipt is the causal barrier. Assert the actual failure immediately
        // even if a source mutant accepted the response and completed instead.
        run_scenario_until_receipt(scenario, true, WorkOutcome::Unknown, false, false).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cl10_terminal_retry_preserves_the_original_failure_in_status_and_watch() {
    let mut f = fixture_mode(
        "claude",
        true,
        Some("m0-wire-success-invalid-usage"),
        |_| {},
    );
    f.register_real_git_project();
    let (_, tasks) = accept(&f, 1).await;
    let task = &tasks[0];
    f.runtime.start().await.unwrap();
    success::wait_normal_bound(&f, task).await;
    let unit = success::bound_unit(&f, task);
    let (_, snapshot) = success::workflow(&f, task);
    let handle = snapshot
        .history
        .iter()
        .find_map(|a| a.execution.clone())
        .unwrap();
    let custody = f.runtime.phase_jobs.observed_preparation(unit.id).unwrap();
    let profile = crate::execution::resources::ResourceManager::new(f.owner.clone())
        .profile(&unit)
        .unwrap();
    // Fail only receipt persistence, after the real Core has decoded/rejected
    // the fixture frame. This creates no successful rows or authority.
    let connection = raw(&f);
    connection.execute_batch(&format!(
        "CREATE TRIGGER m0_receipt_fault BEFORE INSERT ON native_results WHEN NEW.unit_id='{}' BEGIN SELECT RAISE(ABORT,'synthetic M0 receipt failure'); END;",
        unit.id
    )).unwrap();
    let release = profile.output.join("fixture-publication-release");
    struct Release(std::path::PathBuf);
    impl Drop for Release {
        fn drop(&mut self) {
            std::fs::write(&self.0, "release").unwrap();
        }
    }
    let release = Release(release);
    std::fs::write(profile.output.join("fixture-delay-publication"), "delay").unwrap();
    success::release_unit(&f, &unit);
    success::wait_for(
        || profile.output.join("fixture-publication-ready").exists(),
        "Core did not reach publication after capture",
        5,
    )
    .await;
    let adapter = f
        .runtime
        .installed
        .as_ref()
        .ok()
        .unwrap()
        .registry
        .native_phase_port("worker")
        .unwrap()
        .selected_adapter()
        .unwrap();
    let mut updates = adapter.sessions.subscribe(&handle).unwrap();
    assert!(status(&f, &handle).receipt.is_none());
    connection
        .execute_batch("DROP TRIGGER m0_receipt_fault")
        .unwrap();
    let recovered = status(&f, &handle);
    assert_eq!(recovered.failure, Some(NativeFailure::ProtocolFailure));
    assert_eq!(
        recovered.diagnostic,
        Some(NativeFailure::ProtocolFailure.diagnostic())
    );
    assert_eq!(recovered.work, Some(WorkOutcome::Unknown));
    assert_eq!(recovered.disposition, Disposition::ProtocolError);
    assert_eq!(recovered.session.state, SessionState::Lost);
    assert!(recovered.metrics.is_none());
    let receipt = f
        .owner
        .store
        .lock()
        .unwrap()
        .native_session_result(handle.session)
        .unwrap()
        .unwrap();
    assert!(receipt.text.is_none() && receipt.answer_sha256.is_none());
    assert_ne!(receipt.acquisition, AcquisitionStatus::Complete);
    assert_eq!(
        crate::runtime::phase_jobs::counted(task.id, "claude confirm_complete"),
        0
    );
    assert_eq!(
        updates.borrow().failure,
        Some(NativeFailure::ProtocolFailure)
    );
    assert_eq!(updates.borrow().receipt, recovered.receipt);
    updates.borrow_and_update();
    drop(release);
    tokio::time::timeout(Duration::from_secs(5), updates.changed())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        updates.borrow().failure,
        Some(NativeFailure::ProtocolFailure)
    );
    assert_eq!(updates.borrow().receipt, recovered.receipt);
    assert!(updates.borrow().metrics.is_none());
    drop(updates);
    drop(adapter);
    drop(connection);
    let stopped = f.runtime.shutdown().await;
    assert_eq!(
        stopped
            .as_ref()
            .err()
            .and_then(crate::runtime::stop::pending_site),
        Some(crate::runtime::stop::ShutdownSite::AcceptedOriginals)
    );
    assert!(custody.upgrade().is_some());
    finish(f).await;
    success::wait_for(
        || custody.upgrade().is_none(),
        "retry custody survived Runtime drop",
        10,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cl10_installed_success_telemetry_controls_and_mutants_fail_closed() {
    for field in [
        "usage", "input", "output", "read", "write", "cost", "duration",
    ] {
        run_scenario(
            &format!("m0-wire-success-control-{field}"),
            false,
            WorkOutcome::Success,
            false,
        )
        .await;
        run_scenario(
            &format!("m0-wire-success-invalid-{field}"),
            true,
            WorkOutcome::Unknown,
            false,
        )
        .await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cl10_installed_error_telemetry_controls_and_mutants_fail_closed() {
    for field in [
        "usage", "input", "output", "read", "write", "cost", "duration",
    ] {
        run_scenario(
            &format!("m0-wire-error-control-{field}"),
            false,
            WorkOutcome::Failure,
            false,
        )
        .await;
        run_scenario(
            &format!("m0-wire-error-invalid-{field}"),
            true,
            WorkOutcome::Unknown,
            false,
        )
        .await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cl10_valid_unfinished_candidate_is_not_a_prior_captured_success() {
    run_scenario(
        "m0-wire-success-unfinished-invalid-input",
        true,
        WorkOutcome::Unknown,
        true,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn r1_installed_rate_control_and_mutant_validate_before_foreign_filter() {
    for ownership in ["owned", "foreign"] {
        run_scenario(
            &format!("m0-rate-control-{ownership}"),
            false,
            WorkOutcome::Success,
            false,
        )
        .await;
        run_scenario(
            &format!("m0-rate-invalid-{ownership}"),
            true,
            WorkOutcome::Unknown,
            false,
        )
        .await;
    }
}
