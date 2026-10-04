# Issue 46 requirements: bounded owned-group inspection

Risk: STRICT. Shared process ownership/death evidence affects all native adapters,
Git preflight, Context and Workflow. Requirements and design need independent
immutable native review before code; implementation/fixes require causal tests,
committed mutants and independent source re-review before merge.

## Problem and evidence

Exact Issue41 head `888d86d` CI `37169413513` failed on macOS Context with
`native process inspection timed out`; later failures included the sticky
uncertainty cascade. Linux was cancelled by matrix failfast. Native Claude runs
separately observed default-parallel Git preflight timeouts. These observations
do not establish a shared root cause. No timeout/latch/concurrency change is
accepted to turn these failures green.

Installed macOS26.6.2 own-group observations show explicit `/bin/ps -g PGID -o
pid=,pgid=,stat=` selects live and zombie non-TTY members. Published Apple source
also reveals that sysctl failures can emit stderr and return zero. The existing
discarded stderr plus empty-table-as-dead combination can therefore hide an
inspection failure. A flag substitution alone does not close this safety gap.

## Required behavior

1. macOS inspection queries one exact owned process group with the absolute
   trusted `/bin/ps`, explicit leading-minus UNIX options and cleared environment.
   No `-a`, additional PID/group selector, inherited compatibility mode or shell
   expansion may broaden/change the selection. Numeric query hints do not confer
   signal authority; only the existing owned unreaped ProcessGroup can signal.
   Selection must include every visible member regardless of real/effective/saved
   UID or controlling TTY, without implicit owner filtering. Verify pinned Apple
   selector/default-owner and XNU PGRP callback logic. Same-UID unprivileged
   fixtures cannot prove cross-credential visibility; that evidence is source-
   scoped unless a privileged fixture is explicitly authorized. If platform
   completeness cannot be established, observation is Unknown.
2. Use the owned still-unreaped leader PID equal to its group ID as an expected
   observation. Preserve its waitid NOWAIT/kill-before-reap lifecycle. A death
   result requires a complete successful observation containing that exact leader,
   every returned row belonging to the selected group, and all members zombies.
   Any live/non-zombie member refuses death. Missing expected leader, empty output,
   unknown/malformed state, invalid/nonpositive PID/group, duplicate rows,
   unexpected group, extra/missing fields, invalid UTF-8 or incomplete/truncated
   frame is Unknown, never dead. Evaluate the entire frame before returning;
   a live row must not conceal malformed later data.
3. Exit failure or any nonempty stderr is Unknown, including exit-zero sysctl
   diagnostics. Do not parse diagnostic wording/localization to decide safety.
   stdout/stderr are concurrently drained, individually bounded by the existing
   1MiB frame ceiling; exceeding either ceiling is Unknown. Stderr is not copied
   into durable unscoped output or used as instructions. Diagnostics can identify
   the bounded failure category without revealing the process table.
4. Preserve the existing 250ms observation deadline, including pipe completion.
   Use fallible reader creation, bounded completion and owned direct-child
   kill/reap on spawn/reader/poll/read/timeout/exit failure. Reader allocation,
   inspector PID or arbitrary stale hints cannot authorize unrelated signals.
   Trusted `/bin/ps` direct-child cleanup remains separate from supervised-agent
   group cleanup and must not recursively invoke its own inspection.
5. Successful KILL and ESRCH behavior remains unchanged. macOS EPERM may be
   treated as cleanup success only after the owned-group death proof above.
   Unknown inspection preserves permission refusal, uncertainty and Lost
   reservation; never reset the latch or imply native turn completion from death.
6. Linux keeps its existing rustix KILL/ESRCH/permission-error behavior. BSD
   `-g` is not reused with GNU/procps, where personality/selection semantics differ.
   Existing Linux consumers must pass unchanged; no new platform-specific parser
   silently becomes a generic process ownership authority.
7. Published Apple sources establish process-group kernel filtering and its
   limits. KERN_PROC_PGRP filters returned live/zombie records and reduces user
   allocation/formatting; XNU still traverses global process lists and allocates
   its temporary PID list using global nprocs. No global-kernel-work elimination,
   strict latency improvement or explanation of the earlier CI failure is claimed.
   The sample is non-atomic: XNU collects PIDs under a list lock, excludes processes
   still being forked, then releases the lock before reading records. Fork/exit
   races can omit a newly created member while a sampled parent becomes zombie.
   The result is observational evidence under the existing fallback contract,
   not atomic group-death or complete descendant-containment proof.
8. Actual isolated fixtures cover two independently owned groups, exact selected
   membership against a global diagnostic baseline, live members, zombie leader
   with live children, all-zombie owned group and absence/malformed/truncated/error
   output. Fixture cleanup signals only owned unreaped groups and verifies cleanup
   before reaping. Shim tests are failure-injection tests, not real ps acceptance.
9. Causal compiled mutants remove selection, leader, stderr, framing/all-members
   guards or uncertainty preservation. Prefer actual ProcessGroup/cleanup and
   bounded-Git/Context callers; report masked and unit-only mutants honestly.
   Restore committed original source and control fixtures after each experiment.
10. Commit before tests/reviews; run default-concurrency regression suites, required
    fmt/Clippy/debug/release and exact final Linux/macOS CI. Preserve every failure,
    distinguish local constrained observations, and never serialize default tests,
    widen deadlines, suppress diagnostics or relax Lost fences for a pass.

## Boundaries and completion

Successful killpg can signal only some members when credentials differ. Its
existing success path is unchanged and is not covered by the new EPERM observation.
Non-signalable survivors, the non-atomic fork/exit sampling window and intentional
group/session escapes remain explicit residual limitations; this follow-up cannot
claim whole-native-descendant containment or permission-proof atomic death.

No persisted schema, agent permissions/auth/hooks, native protocol, dispatch/retry
policy or process containment claim changes. Detached sessions/daemons and unknown
native outcomes remain separate #14/provider acceptance concerns. Requirements,
design, README, master adapter and verification must describe the same contract.
Integrate the independently approved helper through normal shared-source merges
into #5/#6/#19/#41 without claiming their unrelated acceptance is complete.
