# Issue #4 verification

Workflow: STRICT. Host: macOS arm64, pinned Rust 1.91.1.
Foundation: merged #3 `e51e069`; integrated adapter code at `9d8b13f`.
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
- 53 workspace tests pass after #3 integration: 16 native adapter regressions plus
  37 existing Git/state/config/CLI tests. Clippy/fmt/debug/release and final CI gates
  are recorded before merge. On this macOS host real signal/ps tests require native
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
Final delta re-review verifies the fixes and the merged #3 safety boundary.

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
Additional lifecycle/CAS mutations are recorded before final review.

## Limits

Generic CLI does not claim read-only review/consultation or native recovery/PTY.
Intentional setsid/setpgid escapes need stronger future OS containment. A runtime
SIGKILL/crash cannot prove group death; its persisted reservation remains blocking.
Process-group inspection/permission failure becomes Lost rather than asserting safe
termination. Native adapters must preserve their own safety/config/auth settings
when supplying the explicit environment. No browser/staging target applies to this
local Rust library; runtime recovery, scheduler and provider adapters remain their
dependent Issues.
