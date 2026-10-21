# Issue #109 verification

Branch `chore/issue-109-ci`. Verified code SHA: `de7543b304bc9a17d9ffbf3a31a546577c960004`
(`de7543b3`). `origin/main` at every run: `79e4d27357ae5666c50797bfd9bde1631a30ffba`
(the merge-base; fetched before each run). This record is committed after the
verified SHA; `git diff --name-only --no-renames de7543b3 HEAD` lists only files
under `doc/verification/`.

## Change scope

`git diff --name-only 79e4d273 de7543b3`: `.github/workflows/ci.yml`, `CLAUDE.md`,
`doc/design/issue-109-design.md`, `doc/design/master/architecture.md`,
`doc/requirements/issue-109-requirements.md`, `scripts/local-ci.sh`.
No file under `crates/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
`.cargo/` or `README.md` changes, so the Rust code and test sources are identical
to `79e4d273`.

## Static checks (local, `de7543b3` content)

| Check | Result |
|---|---|
| `actionlint -shellcheck= .github/workflows/ci.yml` | PASS |
| ShellCheck of each inline workflow script (standalone) | PASS (3 scripts) |
| `actionlint` with built-in ShellCheck | did not finish (hung, stopped after 90 s); replaced by the two rows above |
| `bash -n scripts/local-ci.sh`, `shellcheck scripts/local-ci.sh` | PASS |
| Change-detection tests (ignored helper) | PASS: 9 cases; all-doc-only mutant rejected by 6 cases |
| Linux result-marker parser test (ignored helper, exact extracted code) | PASS: 17 cases + negative control |
| Repository-path test (ignored helper, exact extracted code) | PASS: 11 cases + 2 negative controls |
| `origin/main` resolution test (ignored helper, exact extracted code) | PASS: 4 cases + negative control |
| `git diff --check` | PASS |

## Final `scripts/local-ci.sh` run at `de7543b3`

Metadata: Date 2026-10-21T03:36:55Z; Darwin arm64; rustc 1.91.1 (ed61e7d7e 2025-11-07);
cargo 1.91.1 (ea2d97820 2025-10-10); Docker 29.4.0; Linux image
`rust@sha256:867f1d1162913c401378a8504fb17fe2032c760dc316448766f150a130204aad`.
1-minute load average: 13.5 at start, 64.2 at end (other work on the machine).

| Step | Result |
|---|---|
| metadata, main-contained | PASS |
| host-fmt, host-clippy, host-release | PASS |
| host-test (`--no-fail-fast`) | FAIL: `rrx` lib 801 passed / 14 failed / 27 ignored; `tests/context` 2 / 14; `tests/grok` 15 / 1; all other 22 targets pass |
| linux-archive, linux-cache-setup, linux-image | PASS |
| linux-setup, linux-components, linux-fmt, linux-clippy | PASS |
| linux-test (`--no-fail-fast`) | FAIL: `rrx` lib 778 passed / 1 failed / 20 ignored; all other targets pass |
| linux-container | FAIL (reports the linux-test failure) |
| source-unchanged | PASS |

### Host failures re-run alone at `de7543b3`

Each re-run: `cargo test --locked -p rrx <--lib | --test context | --test grok> -- --exact <name>`
on a clean worktree at `de7543b3`, started after the 1-minute load average fell
below 20. Every re-run reported exactly one test.

- Passed alone (25): `rrx` lib `adapter::git_owner::tests::destructive_private_controls_leave_fresh_production_pool_unchanged`,
  `adapter::grok::environment_tests::{initial_and_resume_selection_keep_invalid_and_live_ownership_guards, native_references_and_ordinary_callers_keep_exact_scoped_alternatives, unrelated_foreign_changes_do_not_revoke_checkpoint, unrelated_foreign_changes_do_not_revoke_resume, unrelated_foreign_changes_do_not_revoke_start}`,
  `adapter::inspection::tests::{actual_owned_zombie_leader_and_live_member_cannot_be_declared_dead, actual_selected_group_excludes_other_owned_group_and_accepts_only_all_zombies}`,
  `execution::docker::tests::{docker_engine_change_refuses_cleanup_and_native_requalification, docker_filter_and_operation_identity_are_independently_checked, registered_docker_cleanup_preserves_three_sibling_units_and_known_work}`,
  `execution::native::tests::version_preparation_abort_and_retirement_do_not_leave_launch_authority`,
  `runtime::installation::tests::success::sc10_four_projects_close_independently`;
  `tests/context` 11 tests; `tests/grok` `native_auth_inventory_config_parser_and_tool_evidence_fail_closed`.
- Passed on a later alone attempt (3, `tests/context`): `explicit_file_symbol_callers_and_callees_are_scoped`
  and `mandatory_rules_evidence_and_expansion_never_silently_drop` (2nd attempt),
  `dirty_sources_added_deleted_and_head_changes_invalidate` (3rd attempt). Every
  failed attempt was the 250 ms native process-inspection deadline. All three
  passed in the full host and Linux runs at `affbae27` (identical Rust code).
- Failed alone (1): `execution::native::tests::workflow_source_bootstrap_file_reader_enforces_type_literal_path_and_size`
  (fixture `git commit` exits 1 on this macOS host; passes on Linux). It also fails
  alone at the merge-base `79e4d273` (below). Tracked by #110.

The failure messages of the load-dependent tests are the process-inspection
deadline (250 ms), watchdog expiry, or lost/timeout outcomes under load; this
class is tracked by #89.

### Linux failure

`runtime::installation::tests::fm_rb::unpublished_ready_and_foreign_retained_dtos_never_select_recovery`
("another Runtime owns this state root"). The full Linux steps passed at `5fadc8b4`
and at `affbae27`. `git diff --name-only affbae27 de7543b3` lists only `CLAUDE.md`
and three `doc/` files: the Rust code, `scripts/local-ci.sh` and the container
settings are identical, so this failure is load-dependent, not caused by the change.

## Merge-base comparison (same tests, alone)

Throwaway detached worktree at `79e4d273` (removed afterwards), 1-minute load
average about 10, and the same four tests at `de7543b3` right after:

| Test | `79e4d273` | `de7543b3` |
|---|---|---|
| `adapter::inspection::tests::actual_selected_group_excludes_other_owned_group_and_accepts_only_all_zombies` | pass | pass |
| `adapter::grok::environment_tests::unrelated_foreign_changes_do_not_revoke_start` | pass | pass |
| `execution::native::tests::workflow_source_bootstrap_file_reader_enforces_type_literal_path_and_size` | fail | fail |
| `execution::docker::tests::docker_filter_and_operation_identity_are_independently_checked` | pass | pass |

## Earlier runs on this branch

| SHA | Result |
|---|---|
| `3337f39f` | Real run found R1: the lock descriptor (fd 9) leaked into tests and failed six `rrx-spawn-hygiene` `api_wiring` tests on Linux. Fixed in `5fadc8b4`. |
| `5fadc8b4` | All Linux steps PASS (R1 and `--init` confirmed). host-test FAIL (load). |
| `affbae27` | All Linux steps PASS. host-test with `--no-fail-fast`: only `rrx` lib failed (9 tests); all other host targets pass. |

## Owner exception for this PR (2026-10-21)

The owner chose plan S: the general rule for known host failures is deferred to
#111, `CLAUDE.md` keeps the strict rule, and this PR alone may be pushed and merged
with the failures above. Scope of the exception:

- Branch `chore/issue-109-ci`, verified code SHA `de7543b3` plus commits that change
  only `doc/verification/`.
- Only the failures listed in this record. Every other step passed.
- Basis: no Rust, Cargo, toolchain, `.cargo/` or README change; the one test that
  still fails alone also fails at the merge-base; every other failure passed alone
  (or passed in the identical-code runs at `affbae27`).
- It covers pushing this branch and merging this PR only. It is not a precedent.

## Reviews

| Round | Reviewer (high) | IDs | Status |
|---|---|---|---|
| 1 | Sol, Opus | M1, M2, L1, L2 | Fixed |
| 2 | Opus | O-M1, O-M2, O-M3, O-L1..O-L4 | Fixed |
| 2 | Sol | S-L1, S-L2 | Fixed |
| 3 | Opus | O3-M1, O3-L1, O3-L2, O3-L3 | Fixed |
| 3 | Sol | S-L2 (trailing newline) | Fixed |
| 4 | Opus | N1..N4 (Low); R1 (Medium, found by the real run) | Fixed |
| 5 | Opus | L1 (Low) | Fixed |
| 6 | Opus | F1 (High), F2..F4 (Medium), F5 (Low) | Resolved by withdrawing the rule; deferred to #111 |
| 7 | Opus | L1, L2 (Low, this record) | Fixed by this record |

## Docker resources

Created and kept as standing project caches (per `CLAUDE.md`):
`rururunx-local-ci-registry`, `rururunx-local-ci-git`, `rururunx-local-ci-target`.
All containers ran with `--rm`; no image was built or pulled (`rust:1.91.1` was
already present).
