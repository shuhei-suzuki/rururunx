# M0 U2 verification — B1 refusal and fixture classification

Input HEAD: `7d840a361fb80ed3b1e387cc53ead2a859683fe5` (`wip(m0): fix U2 review round 1 findings`), on top of the fixture conversion `80c670ff`. Approved role revision: `2a9114bc`, B1–B4/B2a. Local macOS; Cargo `/Users/shuheisuzuki/.cargo/bin/cargo`; all dependency operations locked/offline. The orchestrator owns commits and independent review. No network, CI, deployment, or live Agent qualification was performed.

## Result and scope

**85 of the 86 previously B1-blocked test IDs pass; SC10 remains failing without weakening its protected property.** The complete per-ID classification and result are below and in `macos-minimum-m0-legacy-ledger.json` (`u2_b1` on every original entry; `refusal_only` separates refusal checks from positive verification).

| Classification of the original 86 | Count | Disposition |
|---|---:|---|
| Provider-independent safety/behaviour | 80 | 77 converted to Codex fixtures and passing. `pin_c3` is ported to Codex and `pin_c2` maps to an equivalent existing Codex control; their number-pin replacement is AC-01 / U7 pending. SC10 is BLOCKED (below). |
| Provider-independent terminal persistence with a Claude telemetry part | 1 | `cl10_terminal_retry_…` runs the shared retry/republication property through Codex and passes. The Claude malformed-telemetry producer is 移植 (M3, Reviewer), not done. |
| Claude Executor-specific wire/telemetry/permission (refusal-only) | 5 | All map to one executable typed-refusal control with zero effects. Original property is **移植 (M3, Reviewer), not done**; not credited as positive verification. |

Every original ID stays in the ledger with its source hash and assertions; none was ignored or skipped. Six original executables no longer exist on their own: the five refusal-only IDs share one refusal control, and `pin_c2` maps to the equivalent existing Codex control (arrows in the mapping table). Historical `_claude_` names identify provenance: the provider-independent cases now select Codex. Shared assertions still cover CAS, owner/epoch fencing, markers, custody, admission, persistence, read surfaces, and Project isolation through the existing genuine production consumers.

The B1 production boundary refuses every Executor provider except Codex at Goal validation, saved-Task admission/composition, allocation, and first-Executor handoff through one predicate (`execution/capability.rs::require_executor`). No auth/CAS/owner/marker/fencing/#19/process-custody or quota check was changed. No Task.version authority, test authority constructor, SQL success seed, `allow(dead_code)`, suppression, new ignore/skip/serialization, or timeout-only change was introduced.

## Review round 1 corrections

Round 1 (HEAD `80c670ff`) had independent Sol (implementation) and Opus (security/contract) reviews. Confirmed findings were fixed by Sol; the fix diff is `80c670ff..7d840a36`.

| Finding (reporters) | Resulting behaviour |
|---|---|
| Typed refusal does not reach consumers (Sol U2-I01, Opus S-1; 2/2) | The shared socket service loop (`cli/service.rs::serve_runtime`, used by foreground and detached entry points) answers a genuine capability refusal with `ControlResponse::CapabilityRejected { request_id, reason }`; other errors keep the generic `Rejected`. The CLI client turns it back into the typed `CapabilityRefusal`. Saved-Task capability skips from the last finite admission pass appear as `RuntimeMetadata.admission_refusal = Capability(reason)`; this is an advisory observation, never authority, and an installation refusal takes precedence. The dead `ErrorKind::CapabilityUnavailable` is removed. `execution/native.rs::classify_failure` is the single finite classifier for terminal supervision and raw handoff errors and maps `CapabilityRefusal` to `UnsupportedCapability`, not `ProtocolFailure`. |
| Shared safety tests misclassified as B1 (Sol U2-I02, Opus S-2; 2/2) | The cl10 retry property runs through accepted ingress and the installed Codex Core: a malformed owned frame gives `ProtocolFailure`, a trigger fails only receipt persistence, and after recovery and delayed watch publication the failure, diagnostic and receipt are preserved; work stays Unknown, the Session Lost, custody retained through pending shutdown. `pin_c3` declares Codex 0.160.0 against an executable reporting 0.159.0 and is refused before NativeInput with zero invocations and links. `pin_c2` maps to `daemon_native::c_s1f_native_success_and_refusal_equal_under_serve_and_daemon`. |
| Denylist instead of allowlist (Opus S-4) | `require_executor` accepts only `Some("codex")`; Claude keeps `ClaudeExecutorUnavailable`, None/unknown get `ExecutorProviderUnavailable`. Reviewer registration and provider support are unchanged. |
| Saved-Task negative by raw SQL (Opus S-5) | A Task is accepted while alias `worker` is Codex; the alias is changed to Claude and the Runtime and owner are rebuilt at a new epoch (`runtime/tests.rs::ControlFixture::restart_with_config`). The unchanged saved Task is skipped by the production candidate/admission path with the capability reason in status and zero Driver/Unit/helper/native/quota/Session effects. |
| Ledger accuracy and duplicated refusal checks (Opus S-6) | `refusal_only` marks the five refusal mappings; they share one executable refusal control instead of repeating identical refusal tests. New U2 refusal controls also carry `refusal_only: true`. |

Class check: every production caller of the Executor provider check goes through `require_executor` (Goal validation, saved-Task evaluation/composition, first-Executor handoff, allocation); every former `CapabilityUnavailable` producer/consumer is removed; the two raw-error classifiers in `execution/native.rs` are replaced by the one `classify_failure`.

### Mutation checks for the corrections

Each mutation changes the production check, runs the exact test, restores the original bytes, and reruns the same test successfully. They ran on committed HEAD `97f6eab4` (same Rust source as `7d840a36`) with a clean tree before and after (evidence: `.rrx/u2-fixes/mutations-final2/`, one `result.json` per operator with the observed panic). All eleven mutants failed the intended assertion (exit 101); all restored runs passed (exit 0). An earlier batch (`.rrx/u2-fixes/mutations/`) ran before the last test edit and without the wire operator; it is superseded.

| Operator | Exact test |
|---|---|
| wire: collapse `CapabilityRejected` to generic `Rejected` in the service loop | `runtime::installation::tests::u2::goal_capability_refusal_round_trips_through_socket_service` |
| client: drop typed conversion of `CapabilityRejected` | same socket round-trip test |
| status: drop the saved-Task capability reason | `runtime::installation::tests::u2::saved_task_provider_refusal_preserves_finite_classification_before_composition` |
| allowlist: revert to the Claude-only denylist | `runtime::installation::tests::u2::executor_provider_allowlist_refuses_missing_and_unknown_provider` |
| retry: break the shared terminal retry preservation | `runtime::installation::tests::m0_claude::cl10_terminal_retry_preserves_the_original_failure_in_status_and_watch` |
| watch: break delayed watch republication | same cl10 retry test |
| pin: remove observed-version qualification | `runtime::installation::tests::pin::pin_c3_old_observed_cli_version_is_refused_before_native` |
| restart: remove the restart-path capability skip | `runtime::installation::tests::u2::saved_task_provider_refusal_preserves_finite_classification_before_composition` |
| classification-allocation: lose the finite allocation classification | `adapter::native::ownership_tests::claude_executor_allocation_refuses_before_provisional_assets` |
| classification-handoff: lose the finite handoff classification | `runtime::installation::tests::u2::native_handoff_refuses_claude_provider_before_ticket_and_asset_transfer` |
| refusal: remove the shared refusal effects check | `runtime::installation::tests::m0_claude::claude_executor_refusal_has_no_effects` |

## Fixture changes and retained coverage

All running installed/daemon positive fixtures use the existing, unchanged `execution/native/native_fixture.py` Codex app-server implementation. Executor aliases resolve to provider Codex; actual Reviewer aliases have a separate Claude executable/provider/version declaration. The FM committed-source test keeps its original front selector (including the extra accepted-Goal mutation refusal on the historical Claude-labelled front) while changing only the actual fixture provider. No conditional protected assertion was lost.

The five non-starting phase-supervisor controls retain their original must-not-run executable sentinel: replacing that sentinel with a working Session would change what they protect. Both selected aliases resolve to Codex, and normal `AttemptManager::prepare` prepares a Codex Unit matching the selected port; no row or authority is fabricated. Their original counter/identity/custody assertions remain. The corrected entire supervisor module passes 9/0/0.

A token-aware macro audit compared the pre-conversion snapshots with current files: **all 754 original assert/assert_eq/assert_ne/panic macros across 18 non-Claude-specific participating files are unchanged** (`.rrx/u2/green/assertions.json`). Incidental daemon version fixture values change from Claude 2.1.294/2.1.283 to Codex 0.160.0/0.159.0; its success/refusal/zero-invocation comparisons are unchanged. Historical two-provider loops now enumerate the one supported Executor provider; their assertion bodies are unchanged.

The five Claude wire/telemetry IDs map to one executable typed-refusal control (`m0_claude::claude_executor_refusal_has_no_effects`). It checks Goal/Task, Driver, Unit, managed helper/effects, native invocation, quota lease, phase operation, native contract and Held-job effects are zero after genuine refused ingress and an actual zero-admission service pass. The ledger preserves the original source SHA256/assertions and, additionally, the complete pre-conversion Claude wire/pin source including delegated helper assertions for future M3 work.

## Frozen ledger

The baseline union remains **560 original IDs**, in original order, with original producers, source hashes, and assertions intact. Classification is **187 移植 / 371 残置 / 2 機能廃止**, the same as the baseline: the two pin cases are shared guards whose number-pin replacement is AC-01 / U7, not B1 retirements. Pending count is **189**: 181 migration entries for later units, five M3 Reviewer refusal-only mappings, the two AC-01 / U7 pin replacements, and the M3 telemetry part of the cl10 retry entry. Fixture conversion does not certify completion of the new managed-path replacements or M0 acceptance. New U2 controls are inventoried separately (`new_u2_tests`).

## Unconvertible control: SC10

`runtime::installation::tests::success::sc10_four_projects_close_independently` protects four overlapping Executor completions in four Projects, two aliases, with independent links, Task versions, operation identities and Driver publication. It waits for all four to bind before releasing any completion. Originally two peers were Claude and two were Codex.

B1 permits only Codex Executor. `execution/quota.rs::QuotaScheduler::new` fixes `provider_executor=2` and `provider_total=3`; `execution/native.rs::NativeLimits::scheduler` only tightens those caps. Aliases share the provider quota. Four Codex Executors cannot all reach this setup simultaneously, so the Codex-only fixture fails: the exact run stops at **`SETUP: normal Bound absent`**; the full run reaches **`SC10: Task … did not close`** with two bound peers lacking owned-success settlement. Later admission after earlier terminals does not establish four concurrent successful peers. Both are failures, not positive coverage. Raising the production quota, reducing the peer count, using a second/false provider identity, bypassing admission, or releasing completions before all four bind would alter the protected property or serialize it. None was done. The original assertions are retained; the ledger marks this control **BLOCKED pending issue #100**, the owner decision to make provider limits configurable, with no positive credit.

## Final verification

Verification ran on the committed HEAD `7d840a36` with a clean working tree before and after all suites (`.rrx/u2-fixes/final2/`: `input-head.txt`, `input-status.txt`, `output-status.txt`, per-command logs and exit codes, `full-results.json`). Socket and process tests need host permissions, so the suites ran outside the command sandbox under the standing local-verification approval; an earlier in-sandbox attempt (`.rrx/u2-fixes/final/`) could not bind sockets or inspect processes, did not finish, and is not counted. No permission/hook bypass or approval rejection occurred.

| Command (absolute Cargo path above) | Passed / failed / ignored | Result |
|---|---|---|
| `cargo fmt --all -- --check` | n/a | PASS |
| `cargo clippy --locked --offline --workspace --all-targets -- -D warnings` | n/a | PASS |
| `cargo test --locked --offline -p rrx --lib` | 844 / 2 / 27 (nested child 44 / 0 / 1) | FAIL |
| `cargo test --locked --offline -p rrx --test adapter` | 19 / 0 / 0 | PASS |
| `cargo test --locked --offline -p rrx --test runtime_cli` | 9 / 0 / 0 | PASS |
| `cargo test --locked --offline -p rrx --test project_api` | 14 / 0 / 0 | PASS |
| `cargo test --locked --offline -p rrx --test daemon_native` | 2 / 0 / 0 | PASS |
| `cargo test --locked --offline -p rrx --test codex_unavailable` | 1 / 0 / 1 (nested child 1 / 0 / 0) | PASS |

Both observed failures were rerun in their own invocation with `--exact --nocapture`: **2 exact invocations, 0 passed / 2 failed**. No other test failed in the full run, including the cl10 retry test. The 27 library ignores and the one `codex_unavailable` ignore predate this work; that integration parent still runs its ignored child. No ignore was added.

### Failures remaining alone

| Target / exact ID | Attribution |
|---|---|
| `lib` / `execution::native::tests::workflow_source_bootstrap_file_reader_enforces_type_literal_path_and_size` | Known environmental managed-Git/hook failure; supplied baseline says unmodified main fails too. Its source/production path are unchanged. Main was not rerun or checked out in this task. |
| `lib` / `runtime::installation::tests::success::sc10_four_projects_close_independently` | BLOCKED pending issue #100: four concurrent Codex Executors under the unchanged provider quota; explained above. Not attributed to unmodified main. |

Superseded evidence from the fixture conversion at `80c670ff` is retained under `.rrx/u2/green/` (lib 842/7/27, six parallel-run-only failures that passed alone). It does not substitute for the verification above.

## Protected-property mutation sample (fixture conversion)

Each mutation changes a real production check, runs one converted original test with `--exact --nocapture`, restores exact original bytes under a restoration trap, and reruns that same test successfully. Compiler failures or timeout-only outcomes are not counted. All five mutants fail the intended protected-property assertion; all restored runs are 1/0/0. These ran at `80c670ff`; the five production checks they mutate are not changed by the round 1 corrections.

| Operator / protected property | Exact converted test | Observed mutant failure | Exact bytes restored / restored result |
|---|---|---|---|
| `legacy_project_limit` | `runtime::installation::tests::project_limit::d3_plan_guard_refuses_stored_legacy_limit_without_writes` | expected ProjectLimitUnsupported stage assertion fails | yes / 1 passed, 0 failed, 0 ignored |
| `pending_capacity` | `runtime::phase_supervisor::tests::actual_capacity_refusal_returns_original_objects_and_other_project_still_admits` | `global pending capacity bypassed` | yes / 1 passed, 0 failed, 0 ignored |
| `preparation_identity` | `runtime::phase_supervisor::tests::foreign_preparation_and_stopped_service_refuse_without_consuming_inputs` | `foreign prepared Unit accepted` | yes / 1 passed, 0 failed, 0 ignored |
| `reviewer_registration` | `runtime::installation::tests::activation::composition::ca3_missing_alias_is_rejected_at_accepted_ingress` | `unwrap_err()` receives GoalAccepted for an unconfigured Reviewer | yes / 1 passed, 0 failed, 0 ignored |
| `task_body_bound` | `runtime::installation::tests::sweep::c13_early_bound_identity_and_decode_refusals_preserve_sibling_progress` | `over: refusal moved beyond current_task_bounded` | yes / 1 passed, 0 failed, 0 ignored |

Evidence: `.rrx/u2/green/mutations/*-result.json` records operator, exact target, source, original/mutant SHA256, exit codes and restoration proof; logs and `results.tsv` are adjacent. No mutation remains in the tree. Earlier U2 provider-refusal mutations remain historical evidence in `.rrx/u2/rescope-mutations/`; they were not rerun or newly claimed here.

## Complete original-86 mapping

P = provider-independent safety/behaviour; P+M3 = shared part verified through Codex, Claude telemetry part 移植 (M3, Reviewer), not done; M3 (refusal-only) = 移植 (M3, Reviewer), not done, and the result is the shared refusal control; P (U7) = shared guard verified through Codex, number-pin replacement AC-01 / U7 pending. P/FAIL is the explicit BLOCKED SC10 control, not an omitted test. Full original ledger IDs are source path plus function; the table gives executable test IDs.

| Target | Original executable ID | Classification | Final result |
|---|---|---|---|
| `lib` | `runtime::installation::tests::activation::ca1_actual_activation_gates_marker_and_record_only_bound` | P | PASS |
| `lib` | `runtime::installation::tests::activation::composition::ca2_four_tasks_two_projects_retain_own_scope_and_roster` | P | PASS |
| `lib` | `runtime::installation::tests::activation::composition::ca3_missing_alias_is_rejected_at_accepted_ingress` | P | PASS |
| `lib` | `runtime::installation::tests::activation::composition::ca3_representable_roster_refusals_preserve_claimed_sibling` | P | PASS |
| `lib` | `runtime::installation::tests::activation::composition::ca3b_original_composition_cross_pairs_refuse_before_store` | P | PASS |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4a_last_external_drop_at_s1_cancels_original_worker` | P | PASS |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4a_stop_at_genuine_retained_pre_admission_rolls_back` | P | PASS |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4b_activation_segment_linearizes_before_shutdown` | P | PASS |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4b_segment_keeps_runtime_alive_after_external_drop` | P | PASS |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4c_precommit_failure_proves_same_plan_rollback` | P | PASS |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4d_contained_fault_recovers_in_same_live_worker` | P | PASS |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4g_actual_exited_worker_keeps_same_plan_held` | P | PASS |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4h_precommit_panic_stays_poisoned_and_rolls_back` | P | PASS |
| `lib` | `runtime::installation::tests::activation::pages::ca4i_whole_registry_counts_actual_held_plan_across_two_pages` | P | PASS |
| `lib` | `runtime::installation::tests::activation::protection::ca5d_integrity_decision_uses_genuine_allocation_and_negative_raw_members` | P | PASS |
| `lib` | `runtime::installation::tests::activation::protection::ca6_genuine_contract_refuses_generic_workflow_and_contract_writers` | P | PASS |
| `lib` | `runtime::installation::tests::activation::protection::ca7_postcommit_cache_fault_reconciles_same_custody_once` | P | PASS |
| `lib` | `runtime::installation::tests::activation::protection::nongrant_ca4e_task_controls_cannot_supply_drift_producer` | P | PASS |
| `lib` | `runtime::installation::tests::c5_c6_actual_ingress_source_prepared_transport_and_record_only_binding` | P | PASS |
| `lib` | `runtime::installation::tests::epoch_fence::ef1_claude_managed_session_fenced_lost_exactly` | P | PASS |
| `lib` | `runtime::installation::tests::epoch_fence::ef3_wrong_or_missing_permit_rolls_back_the_whole_epoch` | P | PASS |
| `lib` | `runtime::installation::tests::epoch_fence::ef4_terminal_managed_session_is_skipped` | P | PASS |
| `lib` | `runtime::installation::tests::epoch_fence::ef_setup_control_claude_terminal_under_custody_closes_the_session` | P | PASS |
| `lib` | `runtime::installation::tests::fm_d2::fm_d2_committed_source_301_claude_gate_receipts` | P | PASS |
| `lib` | `runtime::installation::tests::fm_d2::fm_d2_committed_source_631_claude_quick_front` | P | PASS |
| `lib` | `runtime::installation::tests::fm_d2::fm_d2_committed_source_631_claude_standard_front` | P | PASS |
| `lib` | `runtime::installation::tests::fm_d2::fm_d2_committed_source_631_claude_strict_front` | P | PASS |
| `lib` | `runtime::installation::tests::m0_claude::cl10_installed_error_telemetry_controls_and_mutants_fail_closed` (→ `claude_executor_refusal_has_no_effects`) | M3 (refusal-only) | PASS |
| `lib` | `runtime::installation::tests::m0_claude::cl10_installed_success_telemetry_controls_and_mutants_fail_closed` (→ `claude_executor_refusal_has_no_effects`) | M3 (refusal-only) | PASS |
| `lib` | `runtime::installation::tests::m0_claude::cl10_terminal_retry_preserves_the_original_failure_in_status_and_watch` | P+M3 | PASS |
| `lib` | `runtime::installation::tests::m0_claude::cl10_valid_unfinished_candidate_is_not_a_prior_captured_success` (→ `claude_executor_refusal_has_no_effects`) | M3 (refusal-only) | PASS |
| `lib` | `runtime::installation::tests::m0_claude::o03_installed_normal_control_responses_fail_closed_and_requests_remain_consumed` (→ `claude_executor_refusal_has_no_effects`) | M3 (refusal-only) | PASS |
| `lib` | `runtime::installation::tests::m0_claude::r1_installed_rate_control_and_mutant_validate_before_foreign_filter` (→ `claude_executor_refusal_has_no_effects`) | M3 (refusal-only) | PASS |
| `lib` | `runtime::installation::tests::nonsuccess::rn_h1_genuine_normal_n_closes_and_pre_s5_refusal_stays_held` | P | PASS |
| `lib` | `runtime::installation::tests::nonsuccess::rn_p1_genuine_writer_contention_confirms_rollback_clears_u_then_closes` | P | PASS |
| `lib` | `runtime::installation::tests::pin::pin_c2_old_declared_cli_version_is_refused_before_native` (→ `daemon_native::c_s1f_native_success_and_refusal_equal_under_serve_and_daemon`) | P (U7) | PASS |
| `lib` | `runtime::installation::tests::pin::pin_c3_old_observed_cli_version_is_refused_before_native` | P (U7) | PASS |
| `lib` | `runtime::installation::tests::preparation::dc1_postcommit_preparation_error_is_confirmed_known_once` | P | PASS |
| `lib` | `runtime::installation::tests::preparation::dc2_dc4_precommit_error_stays_held_with_capped_backoff` | P | PASS |
| `lib` | `runtime::installation::tests::preparation::dc3_shutdown_holding_admission_makes_selected_try_not_attempted` | P | PASS |
| `lib` | `runtime::installation::tests::project_limit::c_s2a2_final_claim_refuses_second_task_typed` | P | PASS |
| `lib` | `runtime::installation::tests::project_limit::c_s2a3_stored_legacy_limits_are_excluded_and_never_rewritten` | P | PASS |
| `lib` | `runtime::installation::tests::project_limit::c_s2a_candidate_never_lists_second_task_while_first_is_occupied` | P | PASS |
| `lib` | `runtime::installation::tests::project_limit::c_s2a_held_phase_operation_alone_keeps_second_task_out` | P | PASS |
| `lib` | `runtime::installation::tests::project_limit::c_s2a_retained_driver_alone_keeps_second_task_out` | P | PASS |
| `lib` | `runtime::installation::tests::project_limit::d3_plan_guard_refuses_stored_legacy_limit_without_writes` | P | PASS |
| `lib` | `runtime::installation::tests::project_limit::d4_status_reports_stored_legacy_limit_read_only_until_repair` | P | PASS |
| `lib` | `runtime::installation::tests::success::br1_claude_owner_immediate_retries_once_after_binding` | P | PASS |
| `lib` | `runtime::installation::tests::success::br1p_claude_owner_planning_retries_once_after_binding` | P | PASS |
| `lib` | `runtime::installation::tests::success::br2_claude_parent_drift_never_retries` | P | PASS |
| `lib` | `runtime::installation::tests::success::br3_claude_linked_plan_is_never_marked` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc10_four_projects_close_independently` | P | FAIL alone |
| `lib` | `runtime::installation::tests::success::sc11_claude_settled_helper_fence_holds_and_confirms` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc11n_claude_parent_edit_while_held_refuses_next_tick` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc1_claude_quick_implement_closes_once_through_the_full_chain` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc2_claude_late_binding_after_settlement_closes_once` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc3_claude_owner_session_advance_conflicts_then_binds_normal` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc3l_claude_writer_lock_is_uncertain_then_rolled_back` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc3p_claude_parent_change_during_hold_refuses_binding` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc3p_t_claude_terminal_before_turn_still_refuses_binding` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc4_claude_uncertain_writes_confirm_and_close_once` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc4c_claude_later_context_keeps_uncertain_closure_held` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc4n_claude_uncertain_bind_then_terminal_confirms` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc5d_claude_revoke_settle_gap_records_no_conclusion` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc6_claude_standard_requirements_waits_without_closure` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc7_claude_ordinary_failure_installs_no_settlement` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc7k_claude_killed_peer_never_late_binds` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc8_claude_results_graph_corruption_blocks_closure` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc9a_claude_shutdown_cancels_success_admission` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc9b_claude_shutdown_preempts_root_success_action` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc9c_claude_pending_publication_stays_held_after_shutdown` | P | PASS |
| `lib` | `runtime::installation::tests::success::sc9d_claude_shutdown_cannot_interleave_admitted_closure` | P | PASS |
| `lib` | `runtime::installation::tests::success::scr_claude_completion_survives_refused_plan_and_conflict` | P | PASS |
| `lib` | `runtime::installation::tests::success::scu_claude_post_claim_evaluation_failure_is_durable_unknown` | P | PASS |
| `lib` | `runtime::installation::tests::sweep::c13_early_bound_identity_and_decode_refusals_preserve_sibling_progress` | P | PASS |
| `lib` | `runtime::installation::tests::sweep::c13_exact_bound_reaches_actual_service_claim` | P | PASS |
| `lib` | `runtime::installation::tests::sweep::c13_original_composition_bounded_reread_refuses_before_validate_for` | P | PASS |
| `lib` | `runtime::installation::tests::sweep::c15_actual_service_pass_keeps_cursor_after_32_paused_tasks` | P | PASS |
| `lib` | `runtime::installation::tests::sweep::c9_busy_admission_sweep_is_synchronous_and_shutdown_stops_claims` | P | PASS |
| `lib` | `runtime::phase_supervisor::tests::actual_capacity_refusal_returns_original_objects_and_other_project_still_admits` | P | PASS |
| `lib` | `runtime::phase_supervisor::tests::actual_publication_drop_and_shutdown_preserve_unknown_slot_until_proven_rollback` | P | PASS |
| `lib` | `runtime::phase_supervisor::tests::actual_selected_allocations_survive_caller_drop_and_guard_is_drained_on_shutdown` | P | PASS |
| `lib` | `runtime::phase_supervisor::tests::fair_actual_pending_pages_rotate_projects_and_retain_owner_change_as_hold` | P | PASS |
| `lib` | `runtime::phase_supervisor::tests::foreign_preparation_and_stopped_service_refuse_without_consuming_inputs` | P | PASS |
| `daemon_native` | `c_s1a_daemon_survives_invoker_group_and_terminal` | P | PASS |
| `daemon_native` | `c_s1f_native_success_and_refusal_equal_under_serve_and_daemon` | P | PASS |

## Files, limits and cleanup

The fixture conversion (`80c670ff`) changed seventeen Rust test/fixture files: `runtime/installation/tests.rs`, its `activation.rs` and `activation/{composition,lifecycle,pages,protection}.rs`, `{epoch_fence,fm_d2,m0_claude,nonsuccess,pin,preparation,project_limit,success,sweep}.rs`, `runtime/phase_supervisor/tests.rs`, and integration `tests/daemon_native.rs`. The round 1 corrections (`7d840a36`) changed production `adapter.rs`, `adapter/native.rs`, `cli/{client,service}.rs`, `execution/{capability,native}.rs`, `runtime/{admission,control,mod}.rs`, and tests `adapter/native/ownership_tests.rs`, `runtime/tests.rs`, `runtime/installation/tests/{m0_claude,pin,u2}.rs`. Paths are under `crates/rrx/src/` except the integration path under `crates/rrx/`. This report and the legacy ledger are also updated. The existing Python peer is unchanged.

No UI/browser work applies. Linux/macOS CI, new S/P/Executor replacements, Claude Reviewer runtime acceptance and real Mac Agent qualification are not performed or claimed. SC10 stays BLOCKED pending issue #100.

No task-owned worktree, checkout, Docker container, volume, network, image or build cache was created. No task-owned external temporary directory was left behind; existing test harnesses retain their normal temporary-fixture handling. Docker resources deleted: none; reclaimed capacity: 0. Scratch, scripts and verification evidence are only under `.rrx/u2/` and `.rrx/u2-fixes/` (Git-ignored).
