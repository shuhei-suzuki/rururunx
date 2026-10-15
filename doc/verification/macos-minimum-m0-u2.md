# M0 U2 verification

Base HEAD: `25545c27843bc4cb95b9d8c2b50377d4153b6269`. Owner-approved role revision: `2a9114bc` (B1–B4/B2a). Local macOS only; no network, Git write, commit, push, CI, external LLM review or live Agent qualification. The orchestrator owns commits/review.

Implementation contains only U2 refusal boundaries and coverage freezing. Task admission remains 0 in the real final Driver consumer. Existing auth/CAS/owner/marker/fencing, #19 pool/ordinary None/factory/public EMPTY, and Native entries/starts/preparations/status/cancel/cleanup remain. **Verification is not green and this is not an M0/L-1 full acceptance claim.**

## Files and controls

| File | Implementation / control |
|---|---|
| `crates/rrx/src/adapter.rs` | C-1/L-1: typed CapabilityUnavailable(CapabilityRefusal), preserved across adapter boundary. |
| `crates/rrx/src/adapter/native.rs` | EX-1/L-1: Executor allocation provider guard, obsolete start_managed refusal, typed mapping and effect-0 test. |
| `crates/rrx/src/adapter/native/ownership_tests.rs` | EX-1: real provisional Unit/selected-port Claude Executor refusal; no helper/effect added. |
| `crates/rrx/src/codex/availability.rs` | #19/AC-11: retain EMPTY filesystem/Git coverage; obsolete transport refusal now finite legacy type. |
| `crates/rrx/src/codex/mod.rs` | L-1: finite legacy refusal adapter, independent of retained #19 availability inputs. |
| `crates/rrx/src/codex/session.rs` | L-1/AC-11: launch/resume/checkpoint close before registration; observation-only test counter and retained-owner no-effect test. Private #19 registration/factory/status/stop remain. |
| `crates/rrx/src/codex/transport.rs` | L-1: obsolete transport closes before availability/path/spawn; effect-0 property test. |
| `crates/rrx/src/execution/capability.rs` | B1/EX-1, AC-04/11, C-1: finite provider/legacy/capability-chain nongrant refusals. |
| `crates/rrx/src/execution/mod.rs` | Export finite nongrant capability diagnostic type. |
| `crates/rrx/src/execution/native.rs` | L-1: start → start_inner → start_legacy closes before starts/entries/preparations; their retention consumers remain. |
| `crates/rrx/src/execution/native/phase_fence_tests.rs` | AC-04/11: accepted Codex Goal plus both retained legacy-provider ports; finite refusal and existing Unit/Session/event 0 assertions. |
| `crates/rrx/src/execution/native/tests.rs` | L-1/AC-11: all Native legacy entry chains effect-0 test; bootstrap legacy subassertion changed to typed refusal; shared assertions retained. |
| `crates/rrx/src/execution/phase.rs` | EX-1: old Claude Executor allocation branch becomes typed refusal; all original Codex nongrant identity/encoding/no-effect positive assertions retained. |
| `crates/rrx/src/execution/workflow_source/native_handoff.rs` | EX-1 defence: selected provider checked before Source/ticket handoff work. |
| `crates/rrx/src/runtime/admission.rs` | AC-04/C-1: saved-row provider check before composition; retain finite installation/capability classifications. |
| `crates/rrx/src/runtime/control.rs` | C-1: finite admission_refusal in RuntimeMetadata; status/stop remain operational. |
| `crates/rrx/src/runtime/goal.rs` | B1/EX-1/AC-04: provider-based Executor refusal before validated Goal can be saved; Reviewer aliases accepted. |
| `crates/rrx/src/runtime/installation.rs` | C-1/EX-1: finite InstallationRefusal enum and typed error propagation; composition provider check. |
| `crates/rrx/src/runtime/installation/tests.rs` | C-1/EX-1: new U2 test module; existing graph-only controls use Codex Executor. |
| `crates/rrx/src/runtime/installation/tests/activation.rs` | C-1: consume finite InstallationRefusal without .0/string-backed diagnostic; remaining shared execution assertions retained. |
| `crates/rrx/src/runtime/installation/tests/epoch_fence.rs` | C-1: consume finite InstallationRefusal without .0/string-backed diagnostic; remaining shared execution assertions retained. |
| `crates/rrx/src/runtime/installation/tests/fm_d2.rs` | C-1: consume finite InstallationRefusal without .0/string-backed diagnostic; remaining shared execution assertions retained. |
| `crates/rrx/src/runtime/installation/tests/nonsuccess.rs` | C-1: consume finite InstallationRefusal without .0/string-backed diagnostic; remaining shared execution assertions retained. |
| `crates/rrx/src/runtime/installation/tests/pin.rs` | C-1: consume finite InstallationRefusal without .0/string-backed diagnostic; remaining shared execution assertions retained. |
| `crates/rrx/src/runtime/installation/tests/preparation.rs` | C-1: consume finite InstallationRefusal without .0/string-backed diagnostic; remaining shared execution assertions retained. |
| `crates/rrx/src/runtime/installation/tests/project_limit.rs` | C-1: consume finite InstallationRefusal without .0/string-backed diagnostic; remaining shared execution assertions retained. |
| `crates/rrx/src/runtime/installation/tests/read_surface.rs` | AC-08: use approved Codex Executor fixture; preserve read-only/security assertions. Claim-dependent case remains red/pending. |
| `crates/rrx/src/runtime/installation/tests/success.rs` | C-1: consume finite InstallationRefusal without .0/string-backed diagnostic; remaining shared execution assertions retained. |
| `crates/rrx/src/runtime/installation/tests/u2.rs` | EX-1/AC-04/08/C-1: genuine ingress, negative saved-row corruption, Reviewer positive, admission 0 and status/stop tests. |
| `crates/rrx/src/runtime/mod.rs` | C-1: public re-export of finite InstallationRefusal for status consumers. |
| `crates/rrx/src/runtime/task_driver.rs` | U2/AC-08/C-1: production final Driver admission always refuses missing M0 capability chain, before plan/reservation/claim. |
| `crates/rrx/src/runtime/tests.rs` | AC-04/15: role-correct Codex Goal fixtures; retain aliases, CAS/owner/read/control and historical nongrant assertions. |
| `crates/rrx/src/state/managed_binding/tests.rs` | AC-04/15: role-correct Codex Goal fixtures; retain aliases, CAS/owner/read/control and historical nongrant assertions. |
| `crates/rrx/tests/codex_unavailable.rs` | #19/AC-11/15: ordinary-library public legacy routes expect finite refusal; existing wrapper still executes its ignored child. |
| `crates/rrx/tests/runtime_cli.rs` | AC-04/15: role-correct Codex Goal fixtures; retain aliases, CAS/owner/read/control and historical nongrant assertions. |
| `crates/rrx/tests/support/legacy.rs` | AC-04/15: role-correct Codex Goal fixtures; retain aliases, CAS/owner/read/control and historical nongrant assertions. |
| `doc/verification/macos-minimum-m0-legacy-ledger.json` | §11/AC-11/15: frozen 560-ID union with source hashes, original producers/assertions, classifications/current assertions; 187 shared ports unfinished. |
| `doc/verification/macos-minimum-m0-u2.md` | U2 evidence, file/control mapping, commands/counts, mutation/reversion table and explicit gaps. |

## Frozen coverage ledger

`macos-minimum-m0-legacy-ledger.json`: 560 unique original IDs; 移植 187 / 残置 371 / 機能廃止 2; unclassified 0. All 187 migration entries remain 未完了. Original safety assertions are preserved in source and in the ledger; refusal is not counted as their positive coverage. New U2 tests (8) are separate from the frozen union.

The union includes the known session60/custody20/ownership7 subset, transport8/attempt4/preparation2/availability1/policy7/protocol16, integration/examples, complete Native/adapter/Workflow/Runtime modules and direct installed-consumer closure. The JSON records the exact module-closure rule and per-source SHA256. Native main source differs from the plan estimate: 26 test IDs, of which 24 directly call start_inner (29 call sites). All 26 plus adjacent modules are included; no invented/omitted ID makes it match “25”.

## Local checks

Cargo executable: `/Users/shuheisuzuki/.cargo/bin/cargo`. Every test row uses `cargo test --locked --offline -p rrx` plus the arguments below. No new ignore/skip/serialization, authority constructor or SQL success seed. The saved-row UPDATE is a negative corruption stimulus only, after real accepted ingress.

`cargo fmt --all`, `cargo fmt --all -- --check`, `cargo clippy --locked --offline --workspace --all-targets -- -D warnings`, and `cargo build --locked --offline --workspace`: PASS.

| Test arguments | Passed / failed / existing ignored |
|---|---|
| `--lib (earlier U2 snapshot, before fixture repairs)` | 626 / 226 / 27 |
| `--lib -- codex:: execution::native:: runtime:: adapter::native:: execution::phase:: state::managed_binding::` | 241 / 186 / 3 |
| `--lib runtime::installation::tests::read_surface` | 13 / 1 / 0 |
| `--lib state::managed_binding::tests` | 5 / 0 / 0 |
| `--lib execution::phase::tests` | 3 / 0 / 0 |
| `--lib execution::native::phase_fence_tests` | 1 / 0 / 0 |
| `--lib runtime::installation::tests::c` | 2 / 1 / 0 |
| `--test adapter` | 19 / 0 / 0 |
| `--test runtime_cli` | 9 / 0 / 0 |
| `--test project_api` | 14 / 0 / 0 |
| `--test daemon_native` | 0 / 2 / 0 |
| `--test codex_unavailable` | 1 / 0 / 1 |

The final affected-module run passed all 8 new U2 tests and all 20 retained #19 custody tests. The ordinary-library wrapper also executed its ignored child successfully (1 child pass); its ignored harness entry was not added by U2. The later test-only ordering/property change was rechecked by the six applicable source operators and restoration tests.

Failures remain visible: old Native/Claude Executor/Driver-positive tests are blocked by the intended refusal boundaries; the known `workflow_source_bootstrap_file_reader_enforces_type_literal_path_and_size` environmental Git-hook failure also occurred, and the broad suite showed load-sensitive existing failures. Its test body was not changed. No independent unmodified-main rerun was performed. The full-suite earlier result is not represented as a final-source pass. The read-only surface claim-dependent case and daemon_native’s 2 old execution positives remain red. Other currently failing safety tests are not silently reclassified as safe.

Logs and exact commands: `.rrx/u2/`. One automatic local-test permission review timed out before execution; the explicitly permitted one-time same-command resubmission succeeded. No hook/sandbox setting was weakened.

## Source mutations

Each row: source edit → the named real test fails → original file bytes restored → the same test passes (1 passed / 0 failed). Compiler errors are not counted. Twenty unique operators; additional targeted executions corrected effect-boundary causality after the initial availability-negative masked the late-refusal assertion. The latest six records use the enabled component case first.

| Mutation | Test function | Killed | Exact bytes restored / test passes |
|---|---|---|---|
| `goal_remove` | `goal_refuses_claude_provider_before_save_and_accepts_same_alias_as_reviewer` | yes | yes / yes |
| `goal_alias_name` | `goal_refuses_claude_provider_before_save_and_accepts_same_alias_as_reviewer` | yes | yes / yes |
| `reviewer_also_refused` | `goal_refuses_claude_provider_before_save_and_accepts_same_alias_as_reviewer` | yes | yes / yes |
| `admission_provider_remove` | `saved_task_provider_refusal_preserves_finite_classification_before_composition` | yes | yes / yes |
| `admission_alias_name` | `saved_task_provider_refusal_preserves_finite_classification_before_composition` | yes | yes / yes |
| `composition_provider_remove` | `saved_task_provider_refusal_preserves_finite_classification_before_composition` | yes | yes / yes |
| `composition_refusal_string` | `capability_chain_stops_normal_admission_while_status_and_stop_work` | yes | yes / yes |
| `admission_classification_lost` | `capability_chain_stops_normal_admission_while_status_and_stop_work` | yes | yes / yes |
| `m0_admission_remove` | `capability_chain_stops_normal_admission_while_status_and_stop_work` | yes | yes / yes |
| `allocation_provider_remove` | `claude_executor_allocation_refuses_before_provisional_assets` | yes | yes / yes |
| `allocation_alias_name` | `claude_executor_allocation_refuses_before_provisional_assets` | yes | yes / yes |
| `native_legacy_remove` | `legacy_native_entry_chains_refuse_before_helper_session_and_input` | yes | yes / yes |
| `native_legacy_after_start_map` | `legacy_native_entry_chains_refuse_before_helper_session_and_input` | yes | yes / yes |
| `adapter_legacy_remove` | `legacy_managed_adapter_refuses_before_preparation` | yes | yes / yes |
| `codex_launch_remove` | `legacy_routes_refuse_before_effects_even_with_retained_component_inputs` | yes | yes / yes |
| `codex_resume_remove` | `legacy_routes_refuse_before_effects_even_with_retained_component_inputs` | yes | yes / yes |
| `codex_checkpoint_remove` | `legacy_routes_refuse_before_effects_even_with_retained_component_inputs` | yes | yes / yes |
| `codex_launch_after_registration` | `legacy_routes_refuse_before_effects_even_with_retained_component_inputs` | yes | yes / yes |
| `legacy_transport_remove` | `legacy_transport_refuses_before_availability_and_effects` | yes | yes / yes |
| `legacy_transport_after_effect_record` | `legacy_transport_refuses_before_availability_and_effects` | yes | yes / yes |

For `codex_launch_after_registration`, the final assertion detects increased registration attempts even if a transient registry entry is dropped. For `legacy_transport_after_effect_record`, the final assertion detects the nonzero transport-attempt observation. `native_legacy_after_start_map` detects the changed retained start map. Thus the late-refusal kills are effect-boundary assertions, not an earlier unavailable fixture.

Exact command/source SHA256/exit/restoration records: `.rrx/u2/mutation-final-results.json`; initial logs `.rrx/u2/mutations/`, final causal logs `.rrx/u2/causal_mutations_final/`.

## Not completed / clarification

- The defence-in-depth `SourceNativeHandoff` provider guard is wired but its own remove-guard mutation is NOT RUN: U2 admission 0 cannot supply a genuine protected Source ticket. No test-only authority or success seed was added to manufacture one. This remains an explicit verification gap for the real producer connection.
- Shared legacy-positive test bodies remain intact and red where U2 makes their producer unavailable. Clarification requested: keep these original tests/red interim results until U5/U6/U7 (current conservative treatment), or replace their current expectations with refusals while preserving the original safety assertions as pending ledger entries. This report does not grant that change itself.
- Reviewer runtime positives still belong to M3; no S/P/Executor success or full L-1 coverage is claimed. U2 must not be independently merged to main as a completed capability switch.

No worktree/checkouts/Docker resources were created. Existing worktree and intended uncommitted changes are preserved for the orchestrator. Scratch/evidence was created only under `.rrx/u2/`; its Python bytecode cache is removed. Existing test harnesses own their temporary fixture cleanup. Docker removal: none, recovered capacity 0.
