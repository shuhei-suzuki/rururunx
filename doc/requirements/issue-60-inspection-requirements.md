# Issue 60 component requirements: inspection diagnostics and Git reader ownership

Risk: STRICT for shared native process/reader lifetime and uncertainty boundaries.
Status: requirements candidate; no design/source approval or implementation claimed.
Parent [Issue60](https://github.com/shuhei-suzuki/rururunx/issues/60) retains its full
runtime workload/effect/delegation/settlement acceptance. This component does not close it.

## Verified current source and failures

Baseline main054aefd3a22a02951fa72b9fa397bed58d156bbf invokes trusted env-cleared
/bin/ps -g OWNED_LEADER -o pid=,pgid=,stat= on macOS. Existing post-spawn250ms
observation includes independent1MiB stdout/stderr, EOF and direct-child status.
Complete framing, exact unreaped leader, empty stderr and successful exit are required.
Unknown retains the existing process uncertainty/latch; blocking inspector cleanup/reap
may exceed the observation budget. Linux native cleanup semantics are unchanged.

Issue51 final18c117a14faddc29e220c8b097b7612200220180 CI37218568315 failed macOS
Context6PASS10FAIL13.39s: explicit_file_symbol_callers_and_callees_are_scoped at
context.rs392 received bounded Git SessionLost inspection timeout; nine other failures
were derived sticky-latch blocked launches. Actual both-OS checkout was
e9e2bda63e5699feda2f7dfb978aa6aab2e4f60b, parents054aefd/18c; tested
tree77a754be85b7b51bc38b0c9f102e5e2f4bac4ae0 equals trigger. Linux all steps pass,
mac builds skipped. Earlier local b90 debug had one inspection timeout and twelve
derived failures; release/public b90 passes do not explain either timeout. No actual
inspector PID/pipe/status/timing evidence survived these errors. Cause is unknown.

At this baseline bounded_git_raw_inner spawns stdout/stderr Tokio JoinHandles before
observation. Cleanup/reap/observation/output errors can return while dropping one or
both handles; dropping a JoinHandle detaches the task. Timeout paths request abort
but do not observe joins. Caller future Drop also loses these reader handles while
ProcessGroup cleanup follows its existing Drop path. These are verified lifetime gaps,
not proof they caused any CI timeout or that an escaped workload existed.

## Required behavior

1. Record a finite value-free failure observation at the actual inspector boundary:
   static stage (spawn, endpoint setup, stream read, direct-child status, EOF/status wait,
   frame validation or post-validation deadline), elapsed observation time, separate
   stdout/stderr bytes/read-call counts/EOF flags, whether direct-child exit was
   observed and its finite outcome category. Missing observations are unavailable,
   never guessed false/zero. Include actual observed cleanup/reap state separately;
   an abort request is not a join and cleanup requested is not cleanup verified.
2. Preserve error kind, original error priority, ps selection, parse/EOF/status guards,
   250ms post-spawn observation and1MiB per-stream caps. Label spawn/mandatory reap
   timing outside that budget explicitly; total operation time is not promised250ms.
   Diagnostics do not classify scheduling, retained FDs, ancestry, workload death or
   native completion from elapsed time/counters. No diagnostic grants kill/reap authority.
3. Carry the typed/static bounded facts through the existing cleanup/AdapterError and
   Context first-failure boundary without classifying arbitrary message substrings.
   Preserve existing scoped failure/audit behavior and error categories; no new global
   record, schema or caller-provided evidence authority. Derived latch rejections stay
   distinct from a new inspector observation.
4. Never copy argv, executable/repository paths, environment, credential/config values,
   ps rows, stderr text, arbitrary child bytes, native bodies or foreign names/IDs into
   new facts. Numeric process IDs are unnecessary for diagnostics and excluded. Use
   bounded counters/known enum values only; formatting errors cannot change cleanup.
5. A private structured Git operation/reader owner must retain both reader tasks through
   every return, observation/cleanup/reap/read/join failure, output timeout and caller
   cancellation/Drop. Abort and observed join are separate states; request cancellation
   then observe termination for both readers even if the first returns an error.
   Returning/dropping a caller must not detach an untracked reader or destroy its owner.
6. Keep the existing finite read/join/preflight budgets. If settlement exceeds a caller
   budget, return/retain Unknown as appropriate while a bounded operation owner keeps
   pending reader/task handles and actual ProcessGroup ownership until termination is
   observed. Do not spawn one unbounded cleanup task/thread per repeated timeout.
   Runtime shutdown/unavailable executor/worker panic must retain uncertainty and must
   not claim observed joins or clear ownership from Drop alone. No blocking Store lock
   or async executor worker waiting on native process cleanup is allowed.
7. The actual caller's uncertainty cannot become clean before its selected-group and
   owned-reader settlement is observed. Failed group cleanup remains uncertain even
   if readers subsequently join. Successful reader shutdown cannot certify complete
   Git helper/service/hook workload cleanup, known remote outcome or full F1 settlement.
   No implicit retry/replay, PID adoption, new source freshness, or release authority.
8. Preserve required hooks/native auth/defaults and Linux behavior. No kernel backend,
   native permission/config/environment policy, scheduler, Store ownership or schema
   change belongs to this component. Full Issue60 owner/reservation/effect matrix and
   readiness ports remain pending; existing selected-group execution does not become
   a claimed complete enabled-profile workload owner.

## Causal verification and gates

- Actual trusted inspector fixtures independently reach retained stdout, retained stderr,
  direct-child status pending/exit observed, frame validation and setup/read failures.
  Assert safe measured facts at each intended stage, original refusal, owned cleanup
  and unchanged Unknown/leader guards. Test-only seams remain private/per-operation.
  Shim delays prove controlled stage reachability, not installed ps scheduling cause.
- Actual ProcessGroup/bounded Git/Context consumer preserves first own failure facts
  while subsequent latch refusals contain no fabricated fresh inspection. Compiled
  omission mutants must reach intended diagnostic consumer; units are labelled units.
- Actual bounded Git reader consumers expose cleanup/reap/read error and caller Drop
  with real owned pipes/tasks. A pending peer reader cannot be masked by an already
  failed first reader. Observe both worker completion counters/closed endpoints and
  retained ownership at the consumer; remove owner/abort/join/fence separately with
  compiled causal assertions. A timeout/error outcome never receives false cleanup credit.
- Test caller Drop before/after cleanup dispatch and during output wait; worker shutdown
  or owner failure remains conservative. Exact restored controls preserve defaults.
- Commit requirements/design before independent native gates; implementation/impact
  commit precedes tests and immutable source reviews. Verify/fix/re-review every actual
  blocking finding. Default-concurrency affected/full debug/release, fmt/Clippy/builds
  and final Linux/macOS public CI retain all failed logs/actual tested checkout trees.
  No same-head green-only rerun, serialization, longer deadline or Unknown relaxation.

## Acceptance limit

This component improves diagnostic/lifetime observability and actual owned reader
retention. It does not itself fix the current inspection availability cause or prove a
new backend faster/complete. Native separate-PGID/SID descendants, partial kill success,
non-atomic fork/exit samples, actual runtime conflict/effect enforcement, all reachable
Git delegation, durable workload reservations, recovery14 and aggregate native16 remain
explicit pending Issue60/provider requirements. Diagnostic completeness or process-group
death alone cannot close those gates.
