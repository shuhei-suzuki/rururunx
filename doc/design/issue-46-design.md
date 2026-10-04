# Issue 46 design: targeted macOS death observation

Risk: STRICT. Proposed only; no helper changes before requirements/design gates.
Source baseline `4851fcd`. Preserve the existing ProcessGroup leader, sticky
uncertainty, blocking-worker boundary, rustix signal semantics and 250ms/1MiB
limits. No schema or public provider interface changes.

## Ownership and query

`ProcessGroup::kill_group` remains the signal authority. Its unreaped child leader
prevents PID/PGID reuse. The macOS EPERM resolver passes that owned leader to a
private observer, which invokes `/bin/ps -g <decimal leader> -o pid=,pgid=,stat=`
with env_clear, null stdin and piped stdout/stderr. No global `-a` flag or second
selector remains. Cross-UID and non-TTY completeness is required: in the pinned
ps source with UNIX2003 u03 true, `-g` increments nselectors and xkeep_implied; the owner-default list is
inserted only when no selector was specified, before the single-group kernel
optimization resets nselectors. Subsequent keepit bypasses UID filtering and
xkeep avoids the TTY filter. The XNU PGRP path leaves uidcheck/ruidcheck/ttycheck
zero and matches only p_pgrpid. This is published-source scoped evidence, not an
actual privileged cross-UID fixture or installed binary/source identity proof. `-g` is process-group selection; `-G` is unrelated real-group
selection. UNIX2003 u03 is an explicit selection precondition. Installed compat(5) says
absent COMMAND_MODE defaults unix2003, supported by pinned Apple Libc
get_compat.c default true/check_env_var and get_compat.h macro. env_clear removes
COMMAND_MODE, and leading-minus `-g` retains that mode through legacy-option
rewriting. No additional configuration override is supplied. The inspector is a trusted direct child, not an
agent and not a recursively supervised process group.

Only this known unreaped ownership contract permits the expected leader observation.
Recovery APIs may not call it on persisted PID/PGID hints and convert absence into
verified death. Missing leader is Unknown even after exit0. Successful kill/ESRCH
paths need no added inspection, so Linux behavior and normal successful signals
are preserved. Existing owned macOS Zombie observation is retained until cleanup
and only then `reap` clears uncertainty.

## Framing and failure

Split bounded process execution from a pure exact-row validator. A result frame
contains completed stdout/stderr plus actual child exit; success requires both EOFs
and a successful exit within one monotonic observation deadline. Start the unchanged
250ms budget immediately after spawn returns, before pipe configuration/observation;
spawn itself remains outside that existing post-spawn budget. Check the deadline
before/after each drain and process-status boundary, retaining bounded5ms idle polling; data-flow iterations do not sleep.

Use a single-thread nonblocking drain, not reader threads/channels. Existing rustix
fs support provides safe fcntl_getfl/fcntl_setfl on each owned stdout/stderr AsFd;
retain flags and add O_NONBLOCK. Each iteration drains both streams with bounded
chunks, distinguishes WouldBlock from EOF, and checks the shared deadline between
chunks so a continuous writer cannot starve the other pipe or deadline. EINTR does
not restart the budget. Each stream retains the existing 1MiB limit and reads at most
1MiB+1 to detect overflow. Configuration/read errors, overflow, timeout, nonzero
exit or nonempty stderr are Unknown. No unfinished reader join or detached reader
can survive the inspection call; dropping read FDs closes this observer's endpoints.

On failure, close both read endpoints; if try_wait already reaped the child, never
signal that numeric PID. Otherwise kill the exact owned direct Child and always attempt mandatory wait even
if kill reports failure; a completed wait can resolve a benign exit-vs-kill race,
whereas wait failure is explicit cleanup uncertainty. A non-retriable try_wait error
(including ECHILD) makes reap status unknown: send neither signal nor another numeric wait; drop the direct-child handle and
report explicit cleanup ownership uncertainty. This
blocking kill/reap remains outside the 250ms observation budget and can extend total
call duration; it is an explicit existing residual, not a hard realtime promise.
Syscall/spawn scheduling likewise has no hard bound. The inspector is trusted ps,
not recursively group-inspected or a contract for arbitrary forking programs.

Pinned Rust1.91.1 std unix anon_pipe uses pipe followed by separate FD_CLOEXEC on
macOS, unlike supported pipe2 targets. A concurrent fork between these operations
can inherit a pipe writer; ps exit alone consequently cannot prove EOF. Nonblocking
reads plus the common EOF deadline handle retained writers without unbounded joining.
Both read and write endpoints can leak in that inheritance window: exclusive endpoint
provenance remains an assumption, and foreign injection/read-side theft is not closed
by EOF/strict parsing. Do not serialize spawns or claim this race is eliminated. The causal
fixture supplies owned pipe endpoints to the same private completion runner and
retains a duplicate write endpoint after its direct child exits; it owns that handle
without descendants. The child emits a valid all-Z expected-leader frame. Run completion on a fixture-owned
thread; a bounded watchdog (for example5s) receives its result. Whether completion
returns or the watchdog expires, close the retained writer and join that thread before
asserting. The control is Unknown by the observation deadline; a mutant discarding EOF
returns dead and fails, and an unconditional read/join mutant is released by watchdog
cleanup then fails an assertion rather than hanging the test job. Verify owned child
reap separately; do not claim a scheduler-hard upper bound.

The validator requires a newline-complete nonempty UTF-8 frame; each nonblank row
has exactly three fields, positive PID/PGID, the expected group and a recognized
macOS process-state spelling. Exact PID rows cannot duplicate. Require one exact
leader PID row. Recognize documented primary states with the pinned emitter suffix grammar;
only primary Z counts as dead. Restrict recognized suffix spelling to the pinned emitter/documented intersection: <, N, X, E, V, L, s and +; E is invalid after Z, < and N are mutually exclusive, and suffixes are ordered/unique: [<|N]?X?E?V?L?s?+?. Unknown states are errors. Validate all rows first,
then return false if any primary state is non-Z, otherwise true. An early live
row must not mask malformed trailing data. State suffix compatibility is verified
against installed man/source and actual installed fixtures before code.

Any stderr byte rejects observational success; no locale-specific parsing or raw diagnostic audit.
Errors identify timeout/size/exit/diagnostic/missing-leader/framing categories.
Primary accepted states are I/R/S/T/U/Z. Pinned mach_state_table is " RUSITH?",
with no Z; print.c emits Z only for SZOMB. H/blank/? and other unsupported primaries
remain framing Unknown, including inaccessible cross-credential task information.
The manual lists additional suffixes not emitted by pinned ps; those remain Unknown
rather than expanding the accepted grammar. Real installed fixture spellings must
be recorded against this documented/emitter set; source/binary identity is unverified.

Sticky uncertainty and SessionLost remain unchanged. Unknown returns its bounded
inspection category, valid live data returns original EPERM; both map to Lost.
No additional raw EPERM/stderr audit field is required. Inspector cleanup errors
must remain explicit and never become an agent-death success.

## Primary platform evidence

Installed macOS26.6.2 build25G83 `/usr/share/man/man1/ps.1` documents process-group
selection, non-TTY behavior and compatibility mode. Published source is not
byte-matched to the installed binary. Cached immutable blob provenance:

- Apple [ps.c](https://github.com/apple-oss-distributions/adv_cmds/blob/main/ps/ps.c),
  blob `f77896791aec508282a72eed31cfc797cfb21370`: one selector becomes
  KERN_PROC_PGRP; returned-record allocation/formatting is targeted. Initial or
  exhausted sysctl failure emits stderr yet returns0; non-ENOMEM retry sleeps1s.
- Apple [kern_sysctl.c](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_sysctl.c),
  blob `1e9baecfcbe6e56d8deac42fe59022266ab7db46`: exact p_pgrpid filtering
  applies to live and zombie lists; required size includes KERN_PROCSLOP. The
  post-iteration handler returns ENOMEM if needed exceeds copied oldlen; ps
  resizes/retries and exhausted failure emits stderr even on exit0.
- Apple [kern_proc.c](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_proc.c),
  blob `a08362b2e777ee607e226d869131efd5e164e526`: proc_iterate still allocates
  against global nprocs+1 and traverses global lists before adding filtered PIDs.
- Apple [get_compat.c](https://github.com/apple-oss-distributions/Libc/blob/main/gen/get_compat.c),
  blob `4a1f140f640ffff4817b15bd5a228768fa4de1cd`, and
  [get_compat.h](https://github.com/apple-oss-distributions/Libc/blob/main/gen/get_compat.h),
  blob `b26ae60c8b3ca9d44043a94e8f821c17d95060d2`: UNIX2003 defaults true;
  only explicit COMMAND_MODE legacy disables it. Published
  [compat.5](https://github.com/apple-oss-distributions/Libc/blob/main/gen/compat.5),
  blob `1366bf160ac76501481195075d2989b49d1070cf`, agrees with installed manual.
- Apple [print.c](https://github.com/apple-oss-distributions/adv_cmds/blob/main/ps/print.c),
  blob `a6f02a86eb29dec5dc39e54ff3692f191c203879`: state prints Z for SZOMB
  and T for SSTOP, otherwise Mach state; documented suffixes follow primary state.
  [keyword.c](https://github.com/apple-oss-distributions/adv_cmds/blob/main/ps/keyword.c),
  blob `aad756d34520c213286959a12615592376f919d7`, binds stat to state.
- Apple [tasks.c](https://github.com/apple-oss-distributions/adv_cmds/blob/main/ps/tasks.c),
  blob `d2fcb07b422cfbb12718f048856e0575eb0237d0`: mach_state_table is
  " RUSITH?" (no Z); get_task_info/task-access failures can produce unsupported
  state information. Those remain Unknown, not fabricated zombie evidence.
- Rust [unix pipe.rs at1.91.1](https://github.com/rust-lang/rust/blob/1.91.1/library/std/src/sys/pal/unix/pipe.rs),
  blob `4798acf9dad6b152d158d044e560798417751f1e`: macOS fallback pipe then
  separate close-on-exec configuration; atomic pipe2 is used on other listed targets.
- Linux [procps ps manual source](https://gitlab.com/procps-ng/procps/-/blob/master/man/ps.1)
  has personality-sensitive selection. This change deliberately keeps existing
  Linux rustix cleanup; no BSD selector is introduced there.

The sample is non-atomic. proc_iterate collects PIDs under proc_list_lock, skips
SIDL/forking processes, unlocks, then fetches records with live/zombie lookup. A
fork/exit race can omit a new member while its sampled parent becomes Z. The
observer does not prove atomic whole-group death. Successful killpg also preserves
its existing partial-success semantics if non-signalable members survive; this
EPERM change does not cover that path. These residual limits remain alongside
intentional process-group/session escapes and are not claimed as closed.

These support a smaller returned table and stricter observation, not a measured CI
cause, no-allocation claim or guaranteed250ms completion under global load.

## Consumer verification and impact

Review all ProcessGroup::kill_group callers, Drop, cleanup_group, reap and the
macOS EPERM resolver. Generic execution, bounded_git_raw and native Grok directly
construct/use the shared ProcessGroup; Grok is already a direct consumer, not a
conditional copied integration. Incoming Claude/Codex count only after their exact
reviewed sources are normally integrated. Every construction site must set
Command.process_group(0) before spawning: ProcessGroup::new currently checks PID>1,
not getpgid equality. This issue does not silently change signal/ESRCH semantics.

Ordinary cleanup remains on its blocking worker. A failed cleanup drops the group
there and may retry inspection once, so two 250ms observation windows plus spawn
and mandatory reap can occur. Cancellation Drop may run on an async worker; that
existing boundary remains an explicit residual, not a newly claimed bounded total
cleanup or closed async-blocking issue.

Real owned-group acceptance uses Rust Command.process_group(0), matching production,
and records actual exit status, stderr byte count, raw state spellings and exact
argv/environment. A global-table comparison is diagnostic fixture evidence only.
Selected membership must equal the known fixture members and exclude a second owned
group. Live leader/children and unreaped zombie leader/live children refuse death;
owned all-zombie members satisfy observational acceptance before leader reap. Same-UID
fixtures do not prove cross-credential completeness/signalability. Pinned ps/XNU source
provides the explicitly limited cross-UID selection argument; any read-only installed
foreign-UID query adds visibility evidence only and never signals returned PID hints.

The exact argument order is -g <PGID> BEFORE -o pid=,pgid=,stat=. Production env_clear
removes COMMAND_MODE; the test-only inspection executable may be a shim that execs
/bin/ps with received argv plus only COMMAND_MODE=legacy. No production environment
override is added; a legacy negative control uses this same ordering and must
report actual exit1 with nonempty stderr (exit/diagnostic Unknown). Reordering format before -g in legacy mode can
interpret the numeric argument as a PID-only selector; a dedicated zombie-leader/live-
child control kills that defense mutant. This is a mode/order defense fixture, not
an assertion that production env_clear inherits legacy mode.

Retain the existing bounded5ms sleep-poll contract without an additional poll-module
change. Overflow fixtures provide otherwise-valid whitespace-padded all-Z stdout
with the expected leader, and bounded stderr overflow; assert size category before
wording. If timing or another guard masks a compiled size mutant, record it without
credit rather than claim a guaranteed overflow kill under load.

Failure fixtures cover exit0 stderr, empty output, timeout, both stream overflows,
malformed/incomplete rows, foreign groups, duplicate PIDs, absent leader, nonblocking
configuration/read failure and the retained pipe writer. Parser-only assertions
carry unit credit; inspector/owned cleanup and actual consumers supply integration
credit. No detached descendants or unknown PID signals are used by test shims.

### Per-invocation consumer seam and mutant causality

Use cfg(all(test, target_os = "macos")) owned test plans only; no global/thread-local
override, runtime configuration or public provider option. An individual ProcessGroup
can carry an inspection executable plus a signal-result plan. For Unknown-consumer
cases, the plan performs the actual KILL on its owned group and then feeds PERM into
the unchanged real resolver; the actual shim inspection returns malformed/missing-
leader/stderr/timeout evidence. Unknown consumer shims are single-process leaders with no group members. Their
consumer performs the reap, so whole-fixture cleanup needs no lost descendant handle.
Consumer plans must actually kill their owned group
before it can be consumed/reaped, including Drop retry; they cannot leave a live
member requiring numeric PGID cleanup after losing their leader handle.

Live-child selection and legacy-reorder safety operators run at the ProcessGroup
boundary instead: the fixture retains the actual ProcessGroup and unreaped leader,
feeds PERM without KILL only for that observation, and asserts refusal/group_owned.
Before any return/reap, remove the injected plan, perform fixture-local direct rustix killpg on the same still-owned unreaped group
even if a mutant incorrectly cleared group_owned. A live member is a direct test
child, spawned with process_group(leader) while the leader was alive, retaining its
Child handle. Verify member death with that independently owned handle/fixed diagnostic
query independent of mutated observer construction; repeat direct KILL on uncertainty
while retaining the leader and fail rather than silently reaping survivors. Reap only
after this independent cleanup. A guard enforces this cleanup on assertion/
error paths. No bounded Git/Grok consumer is allowed to consume this special plan;
its credit is real ProcessGroup/resolver safety, not terminal Lost consumer coverage.
Fake observation never grants arbitrary PID authority.
Existing fail_cleanup bypasses the resolver and cannot count as this evidence.

A private internal bounded_git_raw core receives the per-invocation test plan; its
production wrapper passes no override. A private Context Git core receives its
normal latch from the public caller, while tests supply a fresh isolated latch plus
this plan. Actual GitObservation/confirm_git_cleanup must retain SessionLost and
sticky refusal of the next operation; tests do not reset/poison the process-global
production latch. Native Grok's private test-only instance/Actor plan installs on its
native child after ProcessGroup creation, leaving preflight Git unchanged, and the
real supervise→cleanup_group→terminal persistence path must publish Lost/reservation.
A corresponding generic-launch instance plan covers the same Unknown/Lost consumer,
with existing fail_cleanup tests retained as narrower worker/cleanup coverage.
All plan fields/functions are absent on production and Linux. macOS all-target
clippy lints these seams/tests; macOS non-test debug/release builds validate production
without them. Linux clippy/build/tests validate cross-platform code without macOS
references, not the lint cleanliness of compiled-out items. No shared schema is added.

Distinguish selection operators: replacing -g with -p <leader> is the safety mutant:
unreaped zombie leader plus live child yields a superficially valid leader-only frame,
so the actual owned ProcessGroup/resolver must refuse death without clearing group_owned. Separately, Unknown plans assert real consumer Lost/sticky reservations. Removing the selector
(default UID/TTY narrowing) or replacing it with global -A usually yields rejection;
positive all-zombie controls kill those as liveness/completeness operators, not false-
death proof. Removing foreign-row validation may require a shim/pure-validator test
because real global selection is otherwise rejected/masked; record that narrower
credit. Frame/stderr/leader/error guards must reach actual observer and consumer,
with Lost/sticky/death state asserted before error wording. Every operator compiles,
has an exact patch/head/base, a restored passing control, and explicit masked/survived
outcomes. No deadline/latch/default serialization relaxation earns mutant credit.

Default-concurrency required suite and Linux/macOS exact-head CI remain gates.
No deadline/latch resets, default serialization, replay policy, detached-process
containment or native inference outcome changes. Cross-issue integration and
master documentation are reviewed with the final immutable helper/consumer diff.
