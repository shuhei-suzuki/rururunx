# M0 U2 verification — re-scoped provider refusal

Input HEAD: `77d3b016` (`wip(m0): U2 first draft with temporary admission 0 and legacy refusal`). Unmodified merge base: `25545c27843bc4cb95b9d8c2b50377d4153b6269`. Role revision approved at `2a9114bc` (B1–B4/B2a). This report describes the uncommitted re-scope requested by the orchestrator; the orchestrator owns commits and independent review. Local macOS, offline; no network, Git writes in this worktree, worktrees, external LLM, CI or live Agent qualification.

## Implemented boundary

B1 / §8a(0) / EX-1 provider part / AC-04: a Task Executor resolving to provider `claude` returns finite `CapabilityRefusal::ClaudeExecutorUnavailable`. The main refusal points are Goal plan validation before persistence, saved-Task admission before composition, and Driver composition before graph/installation lookup. Defence in depth remains at Executor allocation and first-Executor Source/Native handoff. Alias spelling is never provider identity. Claude alias registration is unchanged; the same aliases are accepted as Reviewers by genuine Goal ingress and Driver composition.

The temporary `require_m0_capabilities` / admission-0 gate and its `M0CapabilityChainUnavailable` diagnostic are removed. Valid Codex Tasks use the existing main Driver admission path. Existing exact CLI version pins remain; S/P/managed Codex replacement producers are not claimed by U2. Runtime status carries an optional finite installation diagnostic: successful installation reports `None`, not a fabricated missing-chain refusal. `operational` retains its pre-existing meaning/value and does not certify M0 qualification.

The pre-start legacy refusals are removed from NativeSessions, NativeAdapter, CodexAdapter launch/resume/checkpoint, and legacy transport. Their original #19/EMPTY/factory/custody/owner behavior is restored. The seven files listed below are byte-identical to `25545c27`; shared original tests in them were restored, not deleted. No auth/CAS/owner/marker/fencing/#19/process-custody check, pin, timeout, warning or existing ignore was weakened.

## Deferred to U7 and frozen ledger

The orchestrator's re-scope supersedes the original U2 admission-0 row in `.rrx/reviews/m0-unit-plan/unit-plan.md`. Legacy pre-start refusal and its shared-positive ports land in U7 with the actual managed-path replacements (§12a). They are planned, not done. Reviewer runtime acceptance remains M3.

The frozen ledger preserves all **560 original IDs**, their order, source SHA256, original producer/assertions and classification: **187 移植 / 371 残置 / 2 機能廃止**. All 187 shared-positive migrations and the two planned example retirements are explicitly unfinished U7 work (189 pending entries). Refusal does not count as positive safety coverage. The original Native main inventory remains 26 IDs, 24 direct `start_inner` consumers / 29 calls; no ID was added or omitted to match the plan's estimate. Five current U2 tests are recorded separately from the frozen union. Current source hashes are refreshed separately from frozen baseline hashes.

## Files

| Files | Result |
|---|---|
| `adapter.rs`, `adapter/native.rs`, `adapter/native/ownership_tests.rs` | Keep finite provider error mapping/allocation refusal and its test; remove draft legacy gate/test. |
| `execution/capability.rs`, `execution/mod.rs`, `execution/workflow_source/native_handoff.rs` | Keep only provider refusal type/export and handoff check; remove temporary chain/legacy refusal APIs and variants. |
| `execution/phase.rs` | Codex nongrant allocation positives retain identity, encoding, pins and no-effect assertions; Claude Executor branch checks finite refusal. |
| `execution/native/phase_fence_tests.rs` | Accepted Goal uses Codex; original direct AuthorityUnavailable / adapter OwnershipMismatch safety checks restored for both retained ports. |
| `runtime/{goal,admission,installation,task_driver,control,mod}.rs` | Keep provider refusal and finite classification/status plumbing; restore main admission behavior. |
| `runtime/installation/tests/u2.rs`, `workflow.rs` | Four Runtime tests plus existing allocation test cover provider/alias/Reviewer/order/status/stop. Existing activation observation now borrows the actual context, enabling handoff testing with a real production Driver association/input; it creates no authority. |
| `runtime/installation/tests.rs`, its `{activation,epoch_fence,fm_d2,nonsuccess,pin,preparation,project_limit,read_surface,success}.rs`, `runtime/tests.rs`, `state/managed_binding/tests.rs`, `tests/runtime_cli.rs`, `tests/support/legacy.rs` | Keep necessary finite-error consumers and accepted-Goal role fixture conversions; shared assertions stay in source. |
| `codex/{availability,mod,session,transport}.rs`, `execution/native.rs`, `execution/native/tests.rs`, `tests/codex_unavailable.rs` | Entire files restored byte-for-byte to merge base; unrelated legacy gates/counters/tests/expectation edits removed. |
| `doc/verification/macos-minimum-m0-legacy-ledger.json`, this report | Frozen coverage and truthful current evidence/deferred scope. |

File paths above are relative to `crates/rrx/src/` except explicit `tests/` and `doc/` paths. The draft's four new legacy-only tests were removed with their deferred behavior; no frozen shared test ID was removed. The former admission-0 test was rewritten to retain its installation/status/stop property. No test-only authority constructor, Task.version authority, SQL success seed, allow(dead_code), skip/ignore/serialization or timeout-only change was added. The saved-row UPDATE is negative corruption after real accepted ingress.

## Final local verification

Cargo: `/Users/shuheisuzuki/.cargo/bin/cargo`. Commands ran sequentially on restored source; the exact source SHA256 set and every command/exit/count/log are in `.rrx/u2/rescope-verification/results.json`. HEAD stays the input draft because the user prohibits Git writes; verification covers the recorded working-tree bytes.

| Command (Cargo path above) | Passed / failed / ignored | Result |
|---|---|---|
| `cargo fmt --all -- --check` | n/a | PASS |
| `cargo clippy --locked --offline --workspace --all-targets -- -D warnings` | n/a | PASS |
| `cargo test --locked --offline -p rrx --lib` | 759 / 90 / 27 | FAIL |
| `cargo test --locked --offline -p rrx --test adapter` | 19 / 0 / 0 | PASS |
| `cargo test --locked --offline -p rrx --test runtime_cli` | 9 / 0 / 0 | PASS |
| `cargo test --locked --offline -p rrx --test project_api` | 14 / 0 / 0 | PASS |
| `cargo test --locked --offline -p rrx --test daemon_native` | 0 / 2 / 0 | FAIL |
| `cargo test --locked --offline -p rrx --test codex_unavailable` | 1 / 0 / 1 | PASS |

Every failure in this final six-suite pass was rerun alone with `--exact --nocapture`: **92 reruns**, **5 passed / 87 failed / 0 ignored**. Each exact invocation ran one test. Library reruns use `cargo test --locked --offline -p rrx --lib <full ID> -- --exact --nocapture`; integration reruns replace `--lib` with `--test <target>`. No failure was skipped. Successful reruns resolve only the observed full-run flake, not arbitrary future runs.

The existing `codex_unavailable` parent executes its ignored child; its child pass is recorded inside the log and is not added to the outer harness count. The same distinction applies to existing library wrappers and their nested harness results. The 27 library ignores and one integration ignore predate U2.

The focused post-re-scope Runtime run (`--lib runtime::installation::tests::u2::`) passed 4/0/0. Together with the allocation test, all five current U2 tests pass, including exact restoration reruns. All 15 source operators below fail their intended assertion and then pass restored (1/0/0 each).

Preliminary, superseded diagnostics are retained under `.rrx/u2/`: the sandbox full run was 510/338/27 and exposed process-inspection spawn `permission_denied` / SessionLost; the authorized normal-host route preserves all hooks/custody checks. A preliminary host snapshot (before the new handoff test) was 755/93/27. Neither is substituted for the final-source run above. The mutation script's first preflight found a non-unique anchor and exited before edits/tests; it was corrected before starting mutations. No approval rejection or safety-setting change occurred.

## Failures that remain when run alone

The complete list follows. Attribution is based on reading baseline/current code, as permitted by the orchestrator; **the merge base was not executed and no worktree was created**. The daemon tests expose the generic control refusal at `daemon_native.rs:249`; their `Lane::new` config selects provider Claude for worker, and `create` submits that worker as Executor. The control boundary hides the internal error, so their B1 attribution follows that concrete source path rather than a typed wire diagnostic.

- **P (86 tests): no, this exact refusal does not occur on `25545c27`.** Their existing setup chooses a Claude provider as Task Executor. The new B1 Goal/Driver refusal stops that setup with `ClaudeExecutorUnavailable`; baseline Goal validation/composition/allocation has no such guard. Their original deeper assertions were not reached, so those safety properties are not counted as verified. Their migration is unfinished U7 work; the table does not replace them with refusal positives.
- **E (1 test): yes, same failure on unmodified main by the supplied environmental diagnosis and source-path inspection.** `workflow_source_bootstrap_file_reader_enforces_type_literal_path_and_size` fails at its fixture managed Git command under the local Git hook. Its source and Native implementation were restored byte-for-byte to `25545c27`; this test does not consume the new provider allocation/handoff path. No hook was disabled.

| Target | Exact test ID | Attribution / identical on merge base? |
|---|---|---|
| `lib` | `execution::native::tests::workflow_source_bootstrap_file_reader_enforces_type_literal_path_and_size` | E / Yes (environment; inferred) |
| `lib` | `runtime::installation::tests::activation::ca1_actual_activation_gates_marker_and_record_only_bound` | P / No |
| `lib` | `runtime::installation::tests::activation::composition::ca2_four_tasks_two_projects_retain_own_scope_and_roster` | P / No |
| `lib` | `runtime::installation::tests::activation::composition::ca3_missing_alias_is_rejected_at_accepted_ingress` | P / No |
| `lib` | `runtime::installation::tests::activation::composition::ca3_representable_roster_refusals_preserve_claimed_sibling` | P / No |
| `lib` | `runtime::installation::tests::activation::composition::ca3b_original_composition_cross_pairs_refuse_before_store` | P / No |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4a_last_external_drop_at_s1_cancels_original_worker` | P / No |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4a_stop_at_genuine_retained_pre_admission_rolls_back` | P / No |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4b_activation_segment_linearizes_before_shutdown` | P / No |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4b_segment_keeps_runtime_alive_after_external_drop` | P / No |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4c_precommit_failure_proves_same_plan_rollback` | P / No |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4d_contained_fault_recovers_in_same_live_worker` | P / No |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4g_actual_exited_worker_keeps_same_plan_held` | P / No |
| `lib` | `runtime::installation::tests::activation::lifecycle::ca4h_precommit_panic_stays_poisoned_and_rolls_back` | P / No |
| `lib` | `runtime::installation::tests::activation::pages::ca4i_whole_registry_counts_actual_held_plan_across_two_pages` | P / No |
| `lib` | `runtime::installation::tests::activation::protection::ca5d_integrity_decision_uses_genuine_allocation_and_negative_raw_members` | P / No |
| `lib` | `runtime::installation::tests::activation::protection::ca6_genuine_contract_refuses_generic_workflow_and_contract_writers` | P / No |
| `lib` | `runtime::installation::tests::activation::protection::ca7_postcommit_cache_fault_reconciles_same_custody_once` | P / No |
| `lib` | `runtime::installation::tests::activation::protection::nongrant_ca4e_task_controls_cannot_supply_drift_producer` | P / No |
| `lib` | `runtime::installation::tests::c5_c6_actual_ingress_source_prepared_transport_and_record_only_binding` | P / No |
| `lib` | `runtime::installation::tests::epoch_fence::ef1_claude_managed_session_fenced_lost_exactly` | P / No |
| `lib` | `runtime::installation::tests::epoch_fence::ef3_wrong_or_missing_permit_rolls_back_the_whole_epoch` | P / No |
| `lib` | `runtime::installation::tests::epoch_fence::ef4_terminal_managed_session_is_skipped` | P / No |
| `lib` | `runtime::installation::tests::epoch_fence::ef_setup_control_claude_terminal_under_custody_closes_the_session` | P / No |
| `lib` | `runtime::installation::tests::fm_d2::fm_d2_committed_source_301_claude_gate_receipts` | P / No |
| `lib` | `runtime::installation::tests::fm_d2::fm_d2_committed_source_631_claude_quick_front` | P / No |
| `lib` | `runtime::installation::tests::fm_d2::fm_d2_committed_source_631_claude_standard_front` | P / No |
| `lib` | `runtime::installation::tests::fm_d2::fm_d2_committed_source_631_claude_strict_front` | P / No |
| `lib` | `runtime::installation::tests::m0_claude::cl10_installed_error_telemetry_controls_and_mutants_fail_closed` | P / No |
| `lib` | `runtime::installation::tests::m0_claude::cl10_installed_success_telemetry_controls_and_mutants_fail_closed` | P / No |
| `lib` | `runtime::installation::tests::m0_claude::cl10_terminal_retry_preserves_the_original_failure_in_status_and_watch` | P / No |
| `lib` | `runtime::installation::tests::m0_claude::cl10_valid_unfinished_candidate_is_not_a_prior_captured_success` | P / No |
| `lib` | `runtime::installation::tests::m0_claude::o03_installed_normal_control_responses_fail_closed_and_requests_remain_consumed` | P / No |
| `lib` | `runtime::installation::tests::m0_claude::r1_installed_rate_control_and_mutant_validate_before_foreign_filter` | P / No |
| `lib` | `runtime::installation::tests::nonsuccess::rn_h1_genuine_normal_n_closes_and_pre_s5_refusal_stays_held` | P / No |
| `lib` | `runtime::installation::tests::nonsuccess::rn_p1_genuine_writer_contention_confirms_rollback_clears_u_then_closes` | P / No |
| `lib` | `runtime::installation::tests::pin::pin_c2_old_declared_cli_version_is_refused_before_native` | P / No |
| `lib` | `runtime::installation::tests::pin::pin_c3_old_observed_cli_version_is_refused_before_native` | P / No |
| `lib` | `runtime::installation::tests::preparation::dc1_postcommit_preparation_error_is_confirmed_known_once` | P / No |
| `lib` | `runtime::installation::tests::preparation::dc2_dc4_precommit_error_stays_held_with_capped_backoff` | P / No |
| `lib` | `runtime::installation::tests::preparation::dc3_shutdown_holding_admission_makes_selected_try_not_attempted` | P / No |
| `lib` | `runtime::installation::tests::project_limit::c_s2a2_final_claim_refuses_second_task_typed` | P / No |
| `lib` | `runtime::installation::tests::project_limit::c_s2a3_stored_legacy_limits_are_excluded_and_never_rewritten` | P / No |
| `lib` | `runtime::installation::tests::project_limit::c_s2a_candidate_never_lists_second_task_while_first_is_occupied` | P / No |
| `lib` | `runtime::installation::tests::project_limit::c_s2a_held_phase_operation_alone_keeps_second_task_out` | P / No |
| `lib` | `runtime::installation::tests::project_limit::c_s2a_retained_driver_alone_keeps_second_task_out` | P / No |
| `lib` | `runtime::installation::tests::project_limit::d3_plan_guard_refuses_stored_legacy_limit_without_writes` | P / No |
| `lib` | `runtime::installation::tests::project_limit::d4_status_reports_stored_legacy_limit_read_only_until_repair` | P / No |
| `lib` | `runtime::installation::tests::success::br1_claude_owner_immediate_retries_once_after_binding` | P / No |
| `lib` | `runtime::installation::tests::success::br1p_claude_owner_planning_retries_once_after_binding` | P / No |
| `lib` | `runtime::installation::tests::success::br2_claude_parent_drift_never_retries` | P / No |
| `lib` | `runtime::installation::tests::success::br3_claude_linked_plan_is_never_marked` | P / No |
| `lib` | `runtime::installation::tests::success::sc10_four_projects_close_independently` | P / No |
| `lib` | `runtime::installation::tests::success::sc11_claude_settled_helper_fence_holds_and_confirms` | P / No |
| `lib` | `runtime::installation::tests::success::sc11n_claude_parent_edit_while_held_refuses_next_tick` | P / No |
| `lib` | `runtime::installation::tests::success::sc1_claude_quick_implement_closes_once_through_the_full_chain` | P / No |
| `lib` | `runtime::installation::tests::success::sc2_claude_late_binding_after_settlement_closes_once` | P / No |
| `lib` | `runtime::installation::tests::success::sc3_claude_owner_session_advance_conflicts_then_binds_normal` | P / No |
| `lib` | `runtime::installation::tests::success::sc3l_claude_writer_lock_is_uncertain_then_rolled_back` | P / No |
| `lib` | `runtime::installation::tests::success::sc3p_claude_parent_change_during_hold_refuses_binding` | P / No |
| `lib` | `runtime::installation::tests::success::sc3p_t_claude_terminal_before_turn_still_refuses_binding` | P / No |
| `lib` | `runtime::installation::tests::success::sc4_claude_uncertain_writes_confirm_and_close_once` | P / No |
| `lib` | `runtime::installation::tests::success::sc4c_claude_later_context_keeps_uncertain_closure_held` | P / No |
| `lib` | `runtime::installation::tests::success::sc4n_claude_uncertain_bind_then_terminal_confirms` | P / No |
| `lib` | `runtime::installation::tests::success::sc5d_claude_revoke_settle_gap_records_no_conclusion` | P / No |
| `lib` | `runtime::installation::tests::success::sc6_claude_standard_requirements_waits_without_closure` | P / No |
| `lib` | `runtime::installation::tests::success::sc7_claude_ordinary_failure_installs_no_settlement` | P / No |
| `lib` | `runtime::installation::tests::success::sc7k_claude_killed_peer_never_late_binds` | P / No |
| `lib` | `runtime::installation::tests::success::sc8_claude_results_graph_corruption_blocks_closure` | P / No |
| `lib` | `runtime::installation::tests::success::sc9a_claude_shutdown_cancels_success_admission` | P / No |
| `lib` | `runtime::installation::tests::success::sc9b_claude_shutdown_preempts_root_success_action` | P / No |
| `lib` | `runtime::installation::tests::success::sc9c_claude_pending_publication_stays_held_after_shutdown` | P / No |
| `lib` | `runtime::installation::tests::success::sc9d_claude_shutdown_cannot_interleave_admitted_closure` | P / No |
| `lib` | `runtime::installation::tests::success::scr_claude_completion_survives_refused_plan_and_conflict` | P / No |
| `lib` | `runtime::installation::tests::success::scu_claude_post_claim_evaluation_failure_is_durable_unknown` | P / No |
| `lib` | `runtime::installation::tests::sweep::c13_early_bound_identity_and_decode_refusals_preserve_sibling_progress` | P / No |
| `lib` | `runtime::installation::tests::sweep::c13_exact_bound_reaches_actual_service_claim` | P / No |
| `lib` | `runtime::installation::tests::sweep::c13_original_composition_bounded_reread_refuses_before_validate_for` | P / No |
| `lib` | `runtime::installation::tests::sweep::c15_actual_service_pass_keeps_cursor_after_32_paused_tasks` | P / No |
| `lib` | `runtime::installation::tests::sweep::c9_busy_admission_sweep_is_synchronous_and_shutdown_stops_claims` | P / No |
| `lib` | `runtime::phase_supervisor::tests::actual_capacity_refusal_returns_original_objects_and_other_project_still_admits` | P / No |
| `lib` | `runtime::phase_supervisor::tests::actual_publication_drop_and_shutdown_preserve_unknown_slot_until_proven_rollback` | P / No |
| `lib` | `runtime::phase_supervisor::tests::actual_selected_allocations_survive_caller_drop_and_guard_is_drained_on_shutdown` | P / No |
| `lib` | `runtime::phase_supervisor::tests::fair_actual_pending_pages_rotate_projects_and_retain_owner_change_as_hold` | P / No |
| `lib` | `runtime::phase_supervisor::tests::foreign_preparation_and_stopped_service_refuse_without_consuming_inputs` | P / No |
| `daemon_native` | `c_s1a_daemon_survives_invoker_group_and_terminal` | P / No |
| `daemon_native` | `c_s1f_native_success_and_refusal_equal_under_serve_and_daemon` | P / No |

Full-run-only failures that pass alone:

- `lib`: `adapter::git_owner::tests::destructive_private_controls_leave_fresh_production_pool_unchanged` — 1/0/0.
- `lib`: `adapter::grok::environment_tests::unrelated_foreign_changes_do_not_revoke_checkpoint` — 1/0/0.
- `lib`: `adapter::grok::environment_tests::unrelated_foreign_changes_do_not_revoke_resume` — 1/0/0.
- `lib`: `cli::endpoint::tests::actual_endpoint_connects_without_epoch_effect_and_refuses_second_publisher` — 1/0/0.
- `lib`: `cli::endpoint::tests::c_s1e2_bind_replaces_known_leftovers_and_refuses_foreign` — 1/0/0.

## Provider-refusal mutations

Each operator was applied to source, run against the named exact real test, restored in `finally` to SHA256-checked original bytes, and rerun successfully. Compiler errors are not counted. For handoff operators the explicit finite-type assertion fails inside the actual activation callback (the callback panic also triggers its outer wait); this is not counted solely from a timeout. `goal_after_persistence` returns the correct type after saving, and the test catches the nonzero Goal row count. `admission_after_composition` catches ordering/classification through the unavailable-installation control.

Test keys: G=`runtime::installation::tests::u2::goal_refuses_claude_provider_before_save_and_accepts_same_alias_as_reviewer`; S=`runtime::installation::tests::u2::saved_task_provider_refusal_preserves_finite_classification_before_composition`; C=`runtime::installation::tests::u2::installation_refusal_preserves_status_and_stop`; H=`runtime::installation::tests::u2::native_handoff_refuses_claude_provider_before_ticket_and_asset_transfer`; A=`adapter::native::ownership_tests::claude_executor_allocation_refuses_before_provisional_assets`.

| Operator | Test | Mutant failed intended assertion | Exact bytes restored | Restored passed / failed / ignored |
|---|---|---|---|---|
| `goal_remove` | G | yes | yes | 1 / 0 / 0 |
| `goal_alias_name` | G | yes | yes | 1 / 0 / 0 |
| `reviewer_also_refused` | G | yes | yes | 1 / 0 / 0 |
| `admission_provider_remove` | S | yes | yes | 1 / 0 / 0 |
| `admission_alias_name` | S | yes | yes | 1 / 0 / 0 |
| `composition_provider_remove` | S | yes | yes | 1 / 0 / 0 |
| `composition_alias_name` | S | yes | yes | 1 / 0 / 0 |
| `composition_refusal_string` | C | yes | yes | 1 / 0 / 0 |
| `admission_classification_lost` | C | yes | yes | 1 / 0 / 0 |
| `allocation_provider_remove` | A | yes | yes | 1 / 0 / 0 |
| `allocation_alias_name` | A | yes | yes | 1 / 0 / 0 |
| `native_handoff_remove` | H | yes | yes | 1 / 0 / 0 |
| `native_handoff_alias_name` | H | yes | yes | 1 / 0 / 0 |
| `admission_after_composition` | S | yes | yes | 1 / 0 / 0 |
| `goal_after_persistence` | G | yes | yes | 1 / 0 / 0 |

Commands, original/mutant SHA256, exit codes, logs and restoration proof: `.rrx/u2/rescope-mutations/results.json`. All five refusal points have a remove-guard operator. Alias-name comparisons, also refusing Reviewers, finite classification loss and late refusal are covered. Draft legacy and missing-chain mutations are superseded by the re-scope and are not claimed as current coverage. No mutation remains in the tree.

## Limits and cleanup

The original unit plan still describes admission 0; this report follows the explicit orchestrator re-scope. The remaining Claude-Executor fixtures conflict with retained B1, and their ports are recorded as unfinished U7 work. Some generic controls may admit an earlier Codex fixture conversion; the re-scope did not perform that broader port. Wire-specific Claude cases need their actual replacement producer; changing their provider or replacing their expectations with refusal would lose the old coverage. The preserved tests remain visibly red. U2 is not full M0/L-1 acceptance or a green-suite claim.

No UI change or browser verification applies. Independent review, Linux/macOS CI, S/P/Executor implementation and real Mac Agent acceptance are not performed or claimed here. No worktree/checkout/Docker resource was created; no existing worktree was deleted. Scratch/scripts/evidence are only under `.rrx/u2/`; normal build output and pre-existing test harness temporary-fixture handling remain with those harnesses. No task-owned Docker resource or cache was created/deleted; reclaimed capacity 0. No Git write/commit/push occurred in this worktree. The worktree and intended uncommitted changes remain for the orchestrator.
