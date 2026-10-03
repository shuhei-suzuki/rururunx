# Issue #4 verification

Workflow: STRICT. Host: macOS arm64, pinned Rust 1.91.1.
Foundation: merged #3 `e51e069` and #26 `5f442ab`; schema2 and conservative
Blocked Project Session guards are integrated.
Native provider adapters and runtime CLI orchestration remain separate.

## Acceptance and regression evidence

- Capability-oriented object-safe async trait and extensible/config-loaded registry.
- Generic noninteractive execution receives exact prepared payload, explicit
  environment and canonical owning Task CWD. Authentication is unknown; unsupported
  review/consult/PTY/attach/resume/permission/checkpoint/native Goal fail explicitly.
- Stable SessionId survives Starting/Running/terminal snapshots and real SQLite
  reopen. Missing supervisor returns SessionLost and never trusts a persisted PID.
- Prepared context and Session actions reject foreign Project/Goal/Task identity;
  shared Git preflight rejects foreign source/common-dir/worktree, stale binding,
  detached/base/main/master, and injected Git environment variables.
- Starting reservation atomically excludes duplicate same-Task executors and review
  locks. Lost stays reserved until explicit verified-dead resolution.
- Natural exit uses waitid NOWAIT/SIGCHLD, kills the owned group before reaping,
  and prevents redirected background descendants surviving terminal persistence.
  Stop, immediate stop, adapter drop and Tokio shutdown also terminate owned groups.
- Input/output are asynchronous; large unread stdin is an explicit failure. Each
  output stream retains a 64 KiB tail with truncation flags. Terminal retention is
  capped at 32 Sessions and can be explicitly released; durable snapshots remain.
- Session CAS versions retain concurrent operator/recovery updates instead of
  overwriting them. Provider token/cache metrics are null with a reason; prepared
  payload size is observable in bytes and Usage remains attributable through Store.
- Two isolated Projects execute concurrently with distinct context/environment.
- A Project blocked during native execution rejects a fresh launch but allows the
  owned process to stop and persist Stopped without changing native recovery hints.
- Before #26 integration, 56 workspace tests passed at `45a1e01`: 16 native integration regressions, three
  adapter unit regressions (one macOS-specific), and 37 existing tests. Clippy
  `-D warnings`, fmt-check, locked debug/release builds also pass. Earlier integrated
  `5ecfff7` passed Linux and macOS [CI](https://github.com/shuhei-suzuki/rururunx/actions/runs/37094557400).
  Integrated `2b56a53` passes 77 macOS tests: six adapter unit, 18 adapter native,
  18 Git, 15 Project registry, 12 state, five CLI, and three config tests. Two
  inspector tests are macOS-specific (75 Linux total). Clippy all-targets
  `-D warnings` passed; exact final-head build/CI gates are recorded before merge. On this macOS host real signal/ps tests require native
  OS access; sandbox ps was denied and Tokio subprocess waits timed out. Authorized
  `require_escalated` verification uses temporary Git fixtures only.

## Independent review: verify → fix → re-review

Native Claude reviewed immutable `e9059d2` using factual requirements/design/diff,
with operation tools disabled and no executor conversation. Its F1 High (natural
exit descendants) and F2 Medium (Session concurrency overwrite) each reproduced
as a failing real subprocess regression against isolated `73e0162`; known fixture
descendants were killed and the original tree restored clean.

| Finding | Repository verification and disposition |
| --- | --- |
| F1 natural-exit descendants | Verified; group cleanup before leader reap + regression |
| F2 version overwrite | Verified; tracked optimistic versions + operator/reopen regression |
| F3 unbounded terminal retention | Verified static map lifetime; cap32/release + sequential regression |
| F4 crash/recovery ownership | Drop guard and shutdown regression; uncertain/Lost reservations stay blocked via #3 |
| F5/F6 aliased/nonatomic review gate | Known #3 integration prerequisite; merged transactional exclusion/unique bindings + direct regression |
| F7 bare native Goal references | Contract corrected with scoped NativeGoalRef and persisted-owner/checkpoint requirements |
| F8 argv[0] symlink semantics | Corrected with native arg0; executable alias regression |
| F9 lifecycle/assignment | Cheap fail-closed launch checks added; paused Goal regression; DAG/resources stay scheduler-owned |
| F10 error categories | Ownership/Locked/StateFailure/StateConflict are explicit |
| F11 unsupported priority | Generic attach/resume always report UnsupportedCapability, including old/lost references |
| F12 post-spawn cancellation | Owned ProcessGroup Drop guard terminates descendants before releasing reservation |
| F13 duplicate failed audit | Explicit failure path disarms its reservation; CAS prevents stale overwrite |
| F14 payload size | Generic reports prepared bytes; provider token/cache fields remain unknown |
| F15 coverage/evidence | Cancellation, file reopen, unread large stdin, natural exit/shutdown, retention and atomic launch tests added |

The reviewer requested Triple Review evidence as a merge blocker. That requirement
does not apply to this one-reviewer implementation round: the user explicitly
allows configurable reviewer counts. Triple Review dogfood remains #16's obligation.
The second immutable delta review at `5ecfff7` verified the original fixes and
merged #3 safety boundary, and raised one Medium liveness issue plus four Low
cleanup/coverage/error details. Each was verified in code and corrected:

| Finding | Verification and correction |
| --- | --- |
| D1 unbounded Git under Store mutex | Real hanging Git regression; bounded native metadata outside mutex, shared pure ownership validation, and Project/Goal/Task version recheck |
| D2 unconfirmed post-spawn cleanup released reservation | Discarded error verified; failed cleanup retains Starting, cancellation retains Lost if process death is uncertain |
| D3 EPERM negative paths untested | Pure liveness/signal helpers cover live, zombie, malformed and failed inspection; unconditional PERM-success mutation must fail |
| D4 atomic rejection categories | Typed Store guards distinguish lock, duplicate executor and stale snapshot; adapter maps Locked/StateConflict and atomic regression asserts category |
| D5 synchronous normal cleanup inspection | Ordinary cleanup moved to blocking worker; synchronous Drop fallback remains necessary |

A native metadata gate regression also changes the Goal during preflight and
proves StateConflict occurs before the executor spawns. Final delta review covers
these corrections and full unchanged adapter source.

The third immutable review at published `ba80c9b` fetched source directly from
PUBLIC GitHub and checked byte identity. It used native Claude with tools and MCP
disabled through capability restriction, preserving normal hooks/rules/auth. It
reported no Critical/High; verified Medium/Low residuals were corrected:

| Finding | Disposition |
| --- | --- |
| R1/R3 unbounded macOS inspection/deadline claim | Native inspector drains bounded output and kills/reaps after 250 ms; direct Git child reap/output also bounded. Five-second observation budget plus separate cleanup bounds documented. Trusted /bin/ps availability is an explicit macOS host requirement; denied inspection retains Lost. |
| R2 initial lifecycle version gap | Original validation versions retained across Starting and both Git snapshot checks; pause-before-snapshot regression |
| R4 collector environment mismatch | Shared runtime-native Git environment builder retains existing Git config/hooks and excludes routing overrides; deliberately malformed agent HOME Git config regression proves it cannot alter ownership preflight |
| R5 uncertain diagnostics/Blocked hints | State-only Lost plus scoped PID/PGID/error audit; original categories retained on failed state save. Reviewer proposal to expand Blocked native metadata writes rejected to preserve #26 safety contract. |
| R6 Running-write failure async/coverage | Cleanup uses blocking worker; deterministic native post-spawn gate verifies operator CAS preservation/whole-group death and injected cleanup failure preserves Lost with scoped audit under Blocked Project. |
| R7 inactive Project category | Typed ProjectInactive maps to InvalidInput, with direct launch and actual post-spawn guard assertions |
| R8 final evidence/integration | Full combined gates and integration mutations recorded below; integrated schema2 is explicitly from #26 |

Linux CI at `ba80c9b` exposed an outdated cancellation test assertion: the native
Git cleanup worker may still own the group when the launch future is dropped. The
correct conservative result is Lost, and the updated regression asserts that it
continues blocking another executor. Child death is confirmed only after reap; an
unread-input case uses a separate isolated Task fixture. No production safety
guard was weakened to make that assertion pass.

## Isolated mutation evidence

At detached `73e0162`, each intentional mutation failed the protected test, then
original bytes were restored, all 28 baseline tests passed and the tree was clean:

| Removed guarantee | Failing regression |
| --- | --- |
| Prepared input Scope comparison | project_goal_task_context_cwd_branch_and_lock_boundaries_fail_closed |
| Explicit environment clearing | configured_command_receives_only_prepared_context_explicit_environment_and_task_cwd |
| Canonical Git common-dir equality | foreign_git_repository_inside_namespace_detached_and_protected_branches_are_rejected |
| main/master protection | foreign_git_repository_inside_namespace_detached_and_protected_branches_are_rejected |
| Session action Scope check | project_goal_task_context_cwd_branch_and_lock_boundaries_fail_closed |

Shared Git/Store boundary mutation evidence is also retained in #3 verification.
At integrated detached `5ff14a5`, eight additional protected regressions each
failed under intentional mutations; originals were restored and all 53 workspace
tests passed with a clean tree:

| Removed guarantee | Failing regression |
| --- | --- |
| Owned group kill before terminal state | natural_exit_cleans_redirected_background_descendants_before_terminal_persistence |
| Session expected CAS version | concurrent_snapshot_updates_are_not_overwritten_and_identity_survives_reopen |
| Shared authoritative Git preflight | foreign_git_repository_inside_namespace_detached_and_protected_branches_are_rejected |
| GIT_* environment rejection | project_goal_task_context_cwd_branch_and_lock_boundaries_fail_closed |
| Environment clearing | configured_command_receives_only_prepared_context_explicit_environment_and_task_cwd |
| Session Scope | project_goal_task_context_cwd_branch_and_lock_boundaries_fail_closed |
| Prepared context Scope | project_goal_task_context_cwd_branch_and_lock_boundaries_fail_closed |
| Terminal retention eviction | terminal_retention_is_bounded_and_output_can_be_released |


At pre-#26 immutable `45a1e01`, five review-fix mutations each failed its protected
test. Original source bytes were restored; all 56 tests passed and the isolated
tree was clean:

| Removed guarantee | Failing regression |
| --- | --- |
| EPERM requires verified dead group | signal_permission_failure_requires_verified_dead_group |
| Five-second Git deadline | hanging_git_preflight_times_out_without_holding_shared_store |
| Goal version recheck | ownership_change_during_native_preflight_prevents_executor_launch |
| Shared pure Git common directory check | foreign_git_repository_inside_namespace_detached_and_protected_branches_are_rejected |
| Typed transactional contention | atomic_starting_reservation_excludes_duplicate_executor_and_review_acquisition |

## Limits

Generic CLI does not claim read-only review/consultation or native recovery/PTY.
Intentional setsid/setpgid escapes need stronger future OS containment. A runtime
SIGKILL/crash cannot prove group death; its persisted reservation remains blocking.
On macOS, trusted /bin/ps process inspection must be available. Process-group
inspection/permission failure becomes Lost rather than asserting safe
termination. Native adapters must preserve their own safety/config/auth settings
when supplying the explicit environment. No browser/staging target applies to this
local Rust library; runtime recovery, scheduler and provider adapters remain their
dependent Issues.
