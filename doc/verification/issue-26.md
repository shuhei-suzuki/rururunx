# Issue 26 verification

Date: 2026-10-03. Workflow: STRICT. PR: [#34](https://github.com/shuhei-suzuki/rururunx/pull/34).

Production-code revision: `9b38c8e65fcb1e841b864e4e45c24f7e4513ba05`.
Final code/test revision: `a4529ab3f67db062ee61be0040bd35a4d0507902`.
The latter adds only the reviewed PID clear/change regression; the report commit
adds documentation only. No merge or Issue closure is claimed here.

## Checks

On final code/test revision, using pinned Rust 1.91.1:

- `cargo fmt --all -- --check`: pass.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: pass.
- `cargo test --locked --workspace`: 52 pass (3 configuration, 5 CLI,
  17 Git, 15 registry, 12 state).
- `cargo build --locked --workspace`: pass.
- `cargo build --locked --release --workspace`: pass.
- [Exact code/test-head CI](https://github.com/shuhei-suzuki/rururunx/actions/runs/37095989187): success.

Registry tests use isolated temporary Git repositories and real `rrx` commands:
stable two-project IDs/reopen, same Issue 42/branch names with separate worktrees,
foreign Goal/Task/context rejection, missing/swapped/restored roots, sticky BLOCKED,
explicit reference clearing, soft removal without file deletion, active/Lost/lock
safeguards, concurrent remove versus Goal creation, current-Store scoped input
checks, blocked-session diagnostics, terminal resurrection prevention, exact source
CWD, config/rule symlink/nested-repository escapes, actual canonical Git metadata,
environment routing rejection, runtime-global default state and old-format migration.
Fixture-only hooks/signing configuration isolates tests; production Git preserves
native repository/user hooks and settings.

## Mutation evidence

Detached temporary worktrees kept the reviewed implementation immutable. Each
mutation below failed the intended assertion; restored snapshots passed all 15
registry tests with clean Git status:

| Mutation | Targeted test |
| --- | --- |
| Bypass same-root repository identity recovery check | actual_cli_registry_restarts_recovers_and_soft_removes_without_deleting_files |
| Omit atomic idle-removal guard | removal_checks_live_goals_tasks_sessions_and_locks_under_store_transaction |
| Bypass source-reference containment and Git ownership | config_rules_environment_and_symlinks_never_cross_project |
| Disallow conservative existing BLOCKED updates | blocked_projects_accept_lost_and_blocker_updates_without_starting_new_work |
| Ignore latest durable REGISTERED state for scoped input | cleared_refs_recover_and_stale_input_snapshots_are_rejected |
| Omit prior-terminal rejection for Goal | blocked_work_cannot_resurrect_terminal_rows_or_rewrite_goal_and_task_metadata |
| Omit prior-terminal rejection for Task | same test |
| Omit prior-terminal rejection for Session | same test |
| Omit canonical common-Git-directory reference exclusion | separate_git_directory_and_symlink_namespace_are_not_source_references |
| Ignore blocked-session native/recovery metadata equality | blocked_work_cannot_resurrect_terminal_rows_or_rewrite_goal_and_task_metadata |
| Disallow Some(pid) → None clearing | same test |
| Permit unconditional PID replacement | same test |

The separate-Git-directory fixture sets `core.worktree` to the source root so
Git ancestry fallback succeeds from metadata. Its killed mutation independently
proves canonical metadata exclusion, rather than relying on a redundant Git failure.
PID mutations prove both conservative clearing and Some → different Some/None →
Some rebinding rejection. Logs remain in `/private/tmp/rururunx-issue26-mutation-*.log`
and restored final output in `rururunx-issue26-mutation-final-restored.log`.

## Independent native review

Configured native Claude (`claude-opus-5-5`, explicit medium effort), `--tools ''`,
factual source/docs/diff only, existing hooks/settings retained. Every round used
committed clean immutable inputs; fixes began after the corresponding review
completed or in a separate detached worktree. Reviewer findings were verified
against source and regressions before fixing. No native runtime review engine or
two/triple-review dogfood is claimed by these development reviews.

| Round/revision | Verified outcome | Native session / local result |
| --- | --- | --- |
| 1 / d5aa5ac | No Critical/High; fix durable inputs, blocked Lost updates, clearing, CWD and routing gaps | f334a7a4-483c-4c46-8d96-27d35930554b / `/private/tmp/rururunx-issue26-review.json` |
| 2 / 6de6107 | Prior fixes confirmed; fix terminal-row resurrection and strengthen metadata fixture | 9cd37b7d-edc0-4be9-b4e5-bfe5bd058881 / `/private/tmp/rururunx-issue26-rereview.json` |
| 3 / 09de492 | Terminal resurrection closed; fix blocked-session native/PID/recovery rebinding | 8ab8b7e1-b6b4-4268-ab07-3494f19cc44d / `/private/tmp/rururunx-issue26-final-review.json` |
| 4 / 9b38c8e | Fix confirmed, no Critical/High/Medium; strengthen PID clearing/change test | acf191a2-c5d9-444f-9798-e0651f6298d4 / `/private/tmp/rururunx-issue26-native-hints-review.json` |
| 5 / a4529ab | APPROVE, no blocking issues; PID test gap closed | 31ec8db6-5e31-4e34-9a22-2a785a0ea585 / `/private/tmp/rururunx-issue26-pid-test-review.json` |

Caller search found no Lost/wait-state writer outside current Store guards and Git
reservation checks. Native integrations must preserve identity/recovery hints;
blocked diagnostics can use separate audited events and may clear PID only.

## Integration boundaries

`registered_project(&Store, ProjectId)` is a pure durable state/version read.
Runtime code must release shared Store mutexes before synchronous Git/filesystem
preflight, then revalidate scope/version during session reservation. Convenience
`effective_config`, `scoped_file`, `environment_names` and registry reconciliation
are synchronous CLI/off-lock-worker APIs. Store format 2 migrates format 1 without
changing SQL ownership, IDs or history; older binaries explicitly refuse format 2.

Interrupted creation/review locks require Issue 14 audited reconciliation; no
force-unlock CLI is introduced. Repeated root-history validation, separate DB
coordination and completed Task naming reuse retain documented Issue 3 limits.
Native context/environment forwarding, progressive contents, bundle construction,
Goal loops, concurrency/fairness and final multi-project/Context Efficiency dogfood
remain dependent work. No unrelated user repository was changed for verification.
