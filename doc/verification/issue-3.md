# Issue #3 verification

## Scope and impact

STRICT library implementation: Git/worktree lifecycle, Store task binding uniqueness, Project isolation, session/lock reservation contracts. Git CWD/path/ref/routing consumers and SQLite concurrent writers were checked. Project CLI, scheduler workers, live recovery and GitHub merge evidence belong to dependent Issues; no runtime end-to-end completion is claimed.

## Regression and native behavior

37 tests pass: 3 config unit, 5 actual CLI, 12 SQLite persistence and 17 real temporary Git integration tests. Fixtures prove distinct Project Issue #7/#42 identities, independent worktrees, protected/detached/mismatched branches, tracked/untracked/ignored dirty files, source repository identity and linked-root rejection, no symlink adoption, native post-checkout failure without bypass, immutable clean-HEAD changes, Starting/Lost reservations, alias rejection, actual concurrent duplicate Issue creation with one durable owner, and safe merged cleanup. Existing state/config/CLI regressions remain passing; the state usage fixture now binds its executor Task before recording the session.

Commands (Rust 1.91.1, isolated CARGO_HOME/RUSTUP_HOME): cargo test --locked --offline --workspace; cargo clippy --locked --offline --workspace --all-targets -- -D warnings; cargo fmt --all -- --check; cargo build --locked --offline --release --workspace. 37 tests passed at 0f0765f; style-only 09c3896 applies clippy let-chain formatting. Final fmt/clippy/debug/release checks ran at 09c3896. Linux/macOS CI evidence is attached in the PR.

## Independent review

Native Claude, configured default model, --tools '' --effort medium, factual code/design/diff inputs without executor transcript. Session 8d51bd93-a742-440b-99b8-4e659516d8e9.

Initial immutable 0b04f3a review found High alias ownership and Medium overlapping Project/cleanup problems. Real tests at 9c555d2 reproduced three failures before fixes. c032449 reserves unique Task paths/branches transactionally, validates executor identity, compares primary repository identity, rejects overlapping roots, prechecks native branch deletion and releases failed provisional lock acquisition. 58d5ce6 includes Lost-as-reserved semantics coordinated with adapter review. Ambiguous base refs fixed and regression tested. Direct internal Store lock release is audited and cannot establish review acceptance; external edits remain outside advisory lock control; cleanup currently requires ancestry-preserving merge. Native Git runtime configuration remains authoritative, and no bypass flags are introduced.

Second review at 58d5ce6 verified H1/M1/M2/M3 fixed and no Critical/High; Medium N1 (pruned tracking branch fallback) reproduced by a failing test at bc630fd and fixed at 0f0765f. Namespace binding, no-optional-locks and release diagnostics were also hardened. Git's upstream-or-HEAD fallback was checked against [primary Git code](https://github.com/git/git/blob/master/builtin/branch.c). Final independent delta review is recorded on PR #30 before merge. No unresolved review acceptance is inferred from executor claims.

## Mutation verification

A separate detached temporary worktree protected immutable review sources. The first clean-HEAD mutation survived, exposing a test gap; a new clean-head-change regression then detected it. Final 18 mutations each caused the protected test to fail, restoration passed all 37 tests, and mutation worktree was clean. Initial mutations used 0b04f3a/22b2e75; additional ownership/recovery/cleanup mutations used 58d5ce6; pruning/namespace used 0f0765f. These are test failures, not compile-only detections.

| Broken guarantee | Protected regression | Evidence |
|---|---|---|
| runtime_lock | `review_locks_block_executor_and_detect_external_changes` | test failed, restored pass |
| transaction_lock | `review_locks_block_executor_and_detect_external_changes` | test failed, restored pass |
| executor_reservation | `reservations_and_locks_serialize_across_connections` | test failed, restored pass |
| ignored_files | `dirty_tracked_untracked_and_ignored_files_block_review_cleanup` | test failed, restored pass |
| repository_identity | `rejects_foreign_repository_and_existing_path_or_branch` | test failed, restored pass |
| immutable_revision | `immutable_review_detects_clean_head_change` | test failed, restored pass |
| protected_base | `protected_base_and_git_metadata_namespace_are_rejected` | test failed, restored pass |
| git_metadata | `protected_base_and_git_metadata_namespace_are_rejected` | test failed, restored pass |
| duplicate_executor | `duplicate_executors_and_malformed_locks_fail_closed` | test failed, restored pass |
| unique_binding | `aliased_worktree_bindings_and_goal_scoped_executors_are_rejected` | test failed, restored pass |
| executor_worktree | `aliased_worktree_bindings_and_goal_scoped_executors_are_rejected` | test failed, restored pass |
| project_overlap | `linked_and_wrong_identity_project_roots_are_rejected` | test failed, restored pass |
| repository_swap | `qualified_base_ref_wins_over_same_named_tag_and_replacement_is_blocked` | test failed, restored pass |
| lost_reservation | `reservations_and_locks_serialize_across_connections` | test failed, restored pass |
| executor_scope_guard | `aliased_worktree_bindings_and_goal_scoped_executors_are_rejected` | test failed, restored pass |
| native_delete_predicate | `cleanup_checks_native_branch_delete_predicate_before_removal` | test failed, restored pass |
| pruned_upstream_fallback | `cleanup_after_upstream_tracking_ref_is_pruned` | test failed, restored pass |
| namespace_binding | `protected_base_and_git_metadata_namespace_are_rejected` | test failed, restored pass |

## Limits

Logical locks coordinate the runtime, not OS-level writes. External editors/Git can change sources between checks; immutable review verification detects observed revision/dirty changes. Ignored files present during safety checks block removal. Missing/moved/replaced/rewritten source repositories reject operations; explicit registry/recovery integration is pending. Create/cleanup native hook/errors retain intent/locks instead of force deletion. Root commits and common-dir define repository identity; a clone with identical history restored at the identical common-dir remains the same identity. Squash/rebase merge cleanup requires future explicit merge evidence.
