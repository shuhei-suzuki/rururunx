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
ps source, `-g` increments nselectors and xkeep_implied; the owner-default list is
inserted only when no selector was specified, before the single-group kernel
optimization resets nselectors. Subsequent keepit bypasses UID filtering and
xkeep avoids the TTY filter. The XNU PGRP path leaves uidcheck/ruidcheck/ttycheck
zero and matches only p_pgrpid. This is published-source scoped evidence, not an
actual privileged cross-UID fixture or installed binary/source identity proof. `-g` is process-group selection; `-G` is unrelated real-group
selection. Leading-minus options avoid legacy mode and cleared environment avoids
inherited CMD_ENV/COMPAT_MODE. The inspector is a trusted direct child, not an
agent and not a recursively supervised process group.

Only this known unreaped ownership contract permits the expected leader proof.
Recovery APIs may not call it on persisted PID/PGID hints and convert absence into
verified death. Missing leader is Unknown even after exit0. Successful kill/ESRCH
paths need no added inspection, so Linux behavior and normal successful signals
are preserved. Existing owned macOS Zombie observation is retained until cleanup
and only then `reap` clears uncertainty.

## Framing and failure

Split bounded process execution from a pure exact-row validator. A result frame
contains completed stdout/stderr plus actual child exit; no data is trusted before
both readers complete and exit succeeds. Start one monotonic 250ms deadline for
inspection, keep existing bounded poll interval, and check deadline at every
blocking-result boundary. Reader creation is fallible and communicates through
completion channels; joining an unfinished reader must not extend the deadline.
Each stream reads at most 1MiB+1 bytes to detect overflow. Reader failures,
overflow, timeout, nonzero exit or nonempty stderr are Unknown. Failure terminates
and reaps the owned inspector direct child without invoking group inspection;
completed reader handles are joined and owned handles remain accounted for.
Implementation must prove trusted ps pipe closure and failure-path ownership;
it cannot claim this direct-child contract contains arbitrary forking executables.
Test shims must not spawn detached descendants or hide pipe ownership.

The validator requires a newline-complete nonempty UTF-8 frame; each nonblank row
has exactly three fields, positive PID/PGID, the expected group and a recognized
macOS process-state spelling. Exact PID rows cannot duplicate. Require one exact
leader PID row. Recognize documented primary states with legitimate suffix flags;
only primary Z counts as dead. Unknown states are errors. Validate all rows first,
then return false if any primary state is non-Z, otherwise true. An early live
row must not mask malformed trailing data. State suffix compatibility is verified
against installed man/source and actual installed fixtures before code.

Any stderr byte rejects proof; no locale-specific parsing or raw diagnostic audit.
Errors identify timeout/size/exit/diagnostic/missing-leader/framing categories.
Primary installed macOS state characters are I/R/S/T/U/Z; documented suffixes
are +, <, >, A, E, L, N, S, s, V, W and X. Source/installed compatibility must
be checked before finalizing the validator.

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
  applies to live and zombie lists; required size includes KERN_PROCSLOP.
- Apple [kern_proc.c](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_proc.c),
  blob `a08362b2e777ee607e226d869131efd5e164e526`: proc_iterate still allocates
  against global nprocs+1 and traverses global lists before adding filtered PIDs.
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
macOS EPERM resolver. Inspect bounded_git_raw and Context Git callers so Unknown
still becomes SessionLost/sticky refusal. Include native Grok's copied process
ownership integration and incoming Claude/Codex shared helper consumers only when
those exact reviewed sources are normally integrated. No private provider branch
is silently modified.

Real owned-group acceptance compares selected membership with a global table only
as diagnostic fixture evidence. It checks a second owned group is excluded,
leader+live children refuse death, zombie leader+live children refuse death and
owned zombie members accept proof while the leader remains unreaped. Failure
fixtures exercise empty/exit0 stderr, timeout, both output overflows, incomplete
and malformed rows, foreign groups, duplicate PIDs, absent leader, reader failure
and fallible reader spawn. Direct-child cleanup is checked using its actual owned
PID before reap; no unrelated PID is signaled.

Mutants must compile. Assert real consumer death/refusal/Lost state before human
error wording. Selection removal needs an actual owned second group to expose
foreign rows, not just an argv matcher. Leader/error/frame guards use the actual
observer and cleanup boundary; parser-only tests carry unit credit. If successful
KILL masks EPERM inspection, use an explicitly injected signal-result seam with
actual observation and label that scope; never call it full native signal proof.
Restored control must pass. Preserve timing/resource samples and any CI failures.

Default-concurrency required suite and Linux/macOS exact-head CI remain gates.
No deadline/latch resets, default serialization, replay policy, detached-process
containment or native inference outcome changes. Cross-issue integration and
master documentation are reviewed with the final immutable helper/consumer diff.
