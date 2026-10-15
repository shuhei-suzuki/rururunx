//! Original Claude wire/telemetry/permission properties are pending M3 Reviewer ports.
//! These retained IDs now enforce the B1 Executor boundary; see the frozen ledger.
use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn o03_installed_normal_control_responses_fail_closed_and_requests_remain_consumed() {
    assert_claude_executor_refused(fixture("claude", true)).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cl10_terminal_retry_preserves_the_original_failure_in_status_and_watch() {
    assert_claude_executor_refused(fixture("claude", true)).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cl10_installed_success_telemetry_controls_and_mutants_fail_closed() {
    assert_claude_executor_refused(fixture("claude", true)).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cl10_installed_error_telemetry_controls_and_mutants_fail_closed() {
    assert_claude_executor_refused(fixture("claude", true)).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cl10_valid_unfinished_candidate_is_not_a_prior_captured_success() {
    assert_claude_executor_refused(fixture("claude", true)).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn r1_installed_rate_control_and_mutant_validate_before_foreign_filter() {
    assert_claude_executor_refused(fixture("claude", true)).await;
}
