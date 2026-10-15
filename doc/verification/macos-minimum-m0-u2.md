# M0 U2 verification — B1 refusal and fixture classification

Input HEAD: `1b2ca73acd17c7915fd6995f84316e6e1533fe48` (`refactor(m0): re-scope U2 to the Executor provider refusal`). Approved role revision: `2a9114bc`, B1–B4/B2a. Local macOS; Cargo `/Users/shuheisuzuki/.cargo/bin/cargo`; all dependency operations locked/offline. The orchestrator owns commits and independent review. No network, Git write, new worktree, external LLM, CI, deployment, or live Agent qualification was performed.

## Result and scope

**85 of the 86 previously B1-blocked test IDs now pass; one remains failing without weakening its protected property.** The complete per-ID classification and result are below and in `macos-minimum-m0-legacy-ledger.json` (`u2_b1` on every original entry).

| Classification of the original 86 | Count | Disposition |
|---|---:|---|
| Provider-independent safety/behaviour | 78 | 77 converted and passing; SC10 cannot be converted under the fixed provider quota and remains failing. |
| Claude Executor-specific wire/telemetry/permission | 6 | Current ID checks typed B1 refusal and zero effects. Original property is **移植 (M3, Reviewer), not done**. |
| Claude Executor-specific exact-version pin | 2 | **機能廃止 (B1)**; current ID checks typed B1 refusal and zero effects. AC-01 removes number pins, so these are not future Reviewer pin requirements. Current production pins remain unchanged. |

No original ID was deleted, ignored, skipped, or renamed. Historical `_claude_` names identify provenance: the provider-independent cases now select Codex. Shared assertions still cover CAS, owner/epoch fencing, markers, custody, admission, persistence, read surfaces, and Project isolation through the existing genuine production consumers. The eight refusal cases are not credited as positive verification of their removed/deferred Claude properties.

The existing B1 production boundary is unchanged: `CapabilityRefusal::ClaudeExecutorUnavailable` at Goal validation, saved-Task admission/composition, allocation, and first-Executor handoff. No production source change remains. No auth/CAS/owner/marker/fencing/#19/process-custody or quota check was changed. No Task.version authority, test authority constructor, SQL success seed, `allow(dead_code)`, suppression, new ignore/skip/serialization, or timeout change was introduced.

## Fixture changes and retained coverage

All running installed/daemon positive fixtures use the existing, unchanged `execution/native/native_fixture.py` Codex app-server implementation. Executor aliases resolve to provider Codex; actual Reviewer aliases have a separate Claude executable/provider/version declaration. The FM committed-source test keeps its original front selector (including the extra accepted-Goal mutation refusal on the historical Claude-labelled front) while changing only the actual fixture provider. No conditional protected assertion was lost.

The five non-starting phase-supervisor controls retain their original must-not-run executable sentinel: replacing that sentinel with a working Session would change what they protect. Both selected aliases resolve to Codex, and normal `AttemptManager::prepare` prepares a Codex Unit matching the selected port; no row or authority is fabricated. Their original counter/identity/custody assertions remain. The corrected entire supervisor module passes 9/0/0.

A token-aware macro audit compared the pre-conversion snapshots with current files: **all 754 original assert/assert_eq/assert_ne/panic macros across 18 non-Claude-specific participating files are unchanged** (`.rrx/u2/green/assertions.json`). Incidental daemon version fixture values change from Claude 2.1.294/2.1.283 to Codex 0.160.0/0.159.0; its success/refusal/zero-invocation comparisons are unchanged. Historical two-provider loops now enumerate the one supported Executor provider; their assertion bodies are unchanged.

The six Claude wire tests retain their IDs but explicitly enforce the earlier typed role refusal. The shared refusal assertion checks Goal/Task, Driver, Unit, managed helper/effects, native invocation, quota lease, phase operation, native contract and Held-job effects are zero after genuine refused ingress and an actual zero-admission service pass. Pin cases retain their stale declared/observed fixture variations but never probe the peer because B1 is earlier. The ledger preserves the original source SHA256/assertions and, additionally, the complete pre-conversion Claude wire/pin source including delegated helper assertions for future M3 work.

## Frozen ledger

The baseline union remains **560 original IDs**, in original order, with original producers, source hashes, and assertions intact. Baseline classification was 187 移植 / 371 残置 / 2 機能廃止. Current classification is **185 移植 / 371 残置 / 4 機能廃止**, reflecting the two B1 pin retirements. The six wire properties are explicitly unfinished M3 Reviewer ports. Current pending count is **187**: 179 other migration entries plus two planned example retirements remain U7 work; six Reviewer migrations remain M3 work. Fixture conversion does not certify completion of the new managed-path replacements or M0 acceptance. The five current U2 refusal tests remain separately inventoried.

## Unconvertible control: SC10

`runtime::installation::tests::success::sc10_four_projects_close_independently` protects four overlapping Executor completions in four Projects, two aliases, with independent links, Task versions, operation identities and Driver publication. It waits for all four to bind before releasing any completion. Originally two peers were Claude and two were Codex.

B1 permits only Codex Executor. `execution/quota.rs::QuotaScheduler::new` fixes `provider_executor=2` and `provider_total=3`; `execution/native.rs::NativeLimits::scheduler` only tightens those caps. Aliases share the provider quota. Four Codex Executors cannot all reach this setup simultaneously. The Codex-only fixture therefore still fails: the precursor exact run stops at **`SETUP: normal Bound absent`**; the final full run instead reaches **`SC10: Task … did not close`** after earlier peers have already lost their owned-success settlement. Its log shows two terminal jobs lacking owned-success settlement; the exact terminal failure subtypes are not recorded by that assertion. Later admission after earlier terminals does not establish four concurrent successful peers. Both are failures; neither is positive coverage. Raising the production quota, reducing the peer count, using a second/false provider identity, bypassing admission, or releasing completions before all four bind would alter the protected property or serialize it. None was done. The original assertions are retained; the ledger explicitly marks this control BLOCKED / not converted, with no positive credit. A different accepted test scope or product quota decision is needed to make this particular four-concurrent control green.

## Final verification

Verification covers the working-tree source manifest `.rrx/u2/green/final/source_hashes.json`, not a new commit. The final runner verifies that every recorded source SHA256 is unchanged after all suites and alone reruns. Full commands, exit codes, all nested/outer summaries, failures and exact reruns are in `.rrx/u2/green/final/results.json`; logs are adjacent. Host execution preserves hooks and custody checks; the previous U2 report documented sandbox process-inspection failures. No permission/hook bypass or approval rejection occurred.

| Command (absolute Cargo path above) | Passed / failed / ignored | Result |
|---|---|---|
| `cargo fmt --all -- --check` | n/a | PASS |
| `cargo clippy --locked --offline --workspace --all-targets -- -D warnings` | n/a | PASS |
| `cargo test --locked --offline -p rrx --lib` | 842 / 7 / 27 | FAIL |
| `cargo test --locked --offline -p rrx --test adapter` | 19 / 0 / 0 | PASS |
| `cargo test --locked --offline -p rrx --test runtime_cli` | 9 / 0 / 0 | PASS |
| `cargo test --locked --offline -p rrx --test project_api` | 14 / 0 / 0 | PASS |
| `cargo test --locked --offline -p rrx --test daemon_native` | 2 / 0 / 0 | PASS |
| `cargo test --locked --offline -p rrx --test codex_unavailable` | 1 / 0 / 1 | PASS |

Every observed failure (including nested-child failures) was rerun in its own invocation with `--exact --nocapture`: **8 exact invocations, 6 passed / 2 failed / 0 ignored**. Each outer exact harness ran one test; existing wrapper children remain separate nested results. The 27 library ignores and the one `codex_unavailable` ignore predate this work; that integration parent still runs its ignored child. No ignore was added. Successful alone reruns resolve only the observed parallel-run failure, not future flakiness.

### Failures remaining alone

| Target / exact ID | Attribution |
|---|---|
| `lib` / `execution::native::tests::workflow_source_bootstrap_file_reader_enforces_type_literal_path_and_size` | Known environmental managed-Git/hook failure; supplied baseline says unmodified main fails too. Its source/production path are unchanged. Main was not rerun or checked out in this task. |
| `lib` / `runtime::installation::tests::success::sc10_four_projects_close_independently` | Unconvertible four-concurrent-Executor fixture under B1 and the unchanged Codex provider quota; explained above. Not attributed to unmodified main. |

Full-run failures that pass alone:

- `lib::adapter::git_owner::tests::actual_release_wake_after_expiry_keeps_waited_diagnostic_without_new_effects` — exact outer 1/0/0; parallel-run-only failure in this observation.
- `lib::adapter::git_owner::tests::destructive_private_controls_leave_fresh_production_pool_unchanged` — exact outer 1/0/0; parallel-run-only failure in this observation.
- `lib::adapter::grok::environment_tests::unrelated_foreign_changes_do_not_revoke_checkpoint` — exact outer 1/0/0; parallel-run-only failure in this observation.
- `lib::adapter::grok::environment_tests::unrelated_foreign_changes_do_not_revoke_resume` — exact outer 1/0/0; parallel-run-only failure in this observation.
- `lib::execution::owner::tests::database_aliases_cannot_create_a_second_owner_or_advance_epoch` — exact outer 1/0/0; parallel-run-only failure in this observation.
- `lib::runtime::installation::tests::epoch_fence::ef1_codex_managed_session_fenced_lost_exactly` — exact outer 1/0/0; parallel-run-only failure in this observation.

Preliminary, superseded evidence is retained under `.rrx/u2/green/`: initial lib 837/12/27; all 14 observed failures including two nested children rerun with `--exact` (7 pass / 7 fail). The five supervisor alone failures were real fixture provider/Unit namespace mismatches; after fixing only that fixture preparation provider, the module passes 9/0/0. Those precursor reruns overlapped the first full run's last long control; the final reruns above are after completion of all full suites. SC10 also fails alone in the precursor run. No precursor result substitutes for final-source verification.

## Protected-property mutation sample

Each mutation changes a real production check, runs one converted original test with `--exact --nocapture`, restores exact original bytes under a restoration trap, and reruns that same test successfully. Compiler failures or timeout-only outcomes are not counted. All five mutants fail the intended protected-property assertion; all restored runs are 1/0/0. Recorded production snapshots are still byte-identical after the final suites.

| Operator / protected property | Exact converted test | Observed mutant failure | Exact bytes restored / restored result |
|---|---|---|---|
| `legacy_project_limit` | `runtime::installation::tests::project_limit::d3_plan_guard_refuses_stored_legacy_limit_without_writes` | expected ProjectLimitUnsupported stage assertion fails | yes / 1 passed, 0 failed, 0 ignored |
| `pending_capacity` | `runtime::phase_supervisor::tests::actual_capacity_refusal_returns_original_objects_and_other_project_still_admits` | `global pending capacity bypassed` | yes / 1 passed, 0 failed, 0 ignored |
| `preparation_identity` | `runtime::phase_supervisor::tests::foreign_preparation_and_stopped_service_refuse_without_consuming_inputs` | `foreign prepared Unit accepted` | yes / 1 passed, 0 failed, 0 ignored |
| `reviewer_registration` | `runtime::installation::tests::activation::composition::ca3_missing_alias_is_rejected_at_accepted_ingress` | `unwrap_err()` receives GoalAccepted for an unconfigured Reviewer | yes / 1 passed, 0 failed, 0 ignored |
| `task_body_bound` | `runtime::installation::tests::sweep::c13_early_bound_identity_and_decode_refusals_preserve_sibling_progress` | `over: refusal moved beyond current_task_bounded` | yes / 1 passed, 0 failed, 0 ignored |

Evidence: `.rrx/u2/green/mutations/*-result.json` records operator, exact target, source, original/mutant SHA256, exit codes and restoration proof; logs and `results.tsv` are adjacent. No mutation remains in the tree. Earlier U2 provider-refusal mutations remain historical evidence in `.rrx/u2/rescope-mutations/`; they were not rerun or newly claimed here.

## Complete original-86 mapping

P = provider-independent safety/behaviour; M3 = 移植 (M3, Reviewer), not done; B1 = 機能廃止 (B1). P/FAIL is the explicit unconvertible SC10 control, not an omitted test. Full original ledger IDs are source path plus function; the table gives executable test IDs.

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
| `lib` | `runtime::installation::tests::m0_claude::cl10_installed_error_telemetry_controls_and_mutants_fail_closed` | M3 | PASS |
| `lib` | `runtime::installation::tests::m0_claude::cl10_installed_success_telemetry_controls_and_mutants_fail_closed` | M3 | PASS |
| `lib` | `runtime::installation::tests::m0_claude::cl10_terminal_retry_preserves_the_original_failure_in_status_and_watch` | M3 | PASS |
| `lib` | `runtime::installation::tests::m0_claude::cl10_valid_unfinished_candidate_is_not_a_prior_captured_success` | M3 | PASS |
| `lib` | `runtime::installation::tests::m0_claude::o03_installed_normal_control_responses_fail_closed_and_requests_remain_consumed` | M3 | PASS |
| `lib` | `runtime::installation::tests::m0_claude::r1_installed_rate_control_and_mutant_validate_before_foreign_filter` | M3 | PASS |
| `lib` | `runtime::installation::tests::nonsuccess::rn_h1_genuine_normal_n_closes_and_pre_s5_refusal_stays_held` | P | PASS |
| `lib` | `runtime::installation::tests::nonsuccess::rn_p1_genuine_writer_contention_confirms_rollback_clears_u_then_closes` | P | PASS |
| `lib` | `runtime::installation::tests::pin::pin_c2_old_declared_cli_version_is_refused_before_native` | B1 | PASS |
| `lib` | `runtime::installation::tests::pin::pin_c3_old_observed_cli_version_is_refused_before_native` | B1 | PASS |
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

Seventeen Rust test/fixture files changed: `runtime/installation/tests.rs`, its `activation.rs` and `activation/{composition,lifecycle,pages,protection}.rs`, `{epoch_fence,fm_d2,m0_claude,nonsuccess,pin,preparation,project_limit,success,sweep}.rs`, `runtime/phase_supervisor/tests.rs`, and integration `tests/daemon_native.rs`. Paths are under `crates/rrx/src/` except the integration path under `crates/rrx/`. The other two changed files are this report and the legacy ledger. The existing Python peer and all production source remain unchanged.

No UI/browser work applies. Independent review, Linux/macOS CI, new S/P/Executor replacements, Claude Reviewer runtime acceptance and real Mac Agent qualification are not performed or claimed. The remaining ambiguity is the accepted replacement scope for the four-concurrent SC10 control; this change does not decide it or weaken quota.

No task-owned worktree, checkout, Docker container, volume, network, image or build cache was created. No task-owned external temporary directory was created or left behind; existing test harnesses retain their normal temporary-fixture handling. Docker resources deleted: none; reclaimed capacity: 0. Scratch, snapshots, scripts and verification evidence are only under `.rrx/u2/green/` for the orchestrator. No Git write/commit/push occurred. Intended working-tree changes and the worktree remain for the orchestrator; no worktree was deleted.
