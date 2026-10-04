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
but do not observe joins. Caller future Drop also loses these reader handles. Drop during observe_exit, reap
timeout/error or observation error can run blocking ProcessGroup Drop on the async
polling worker; cleanup_group error runs its Drop retry on the blocking worker. These are verified lifetime gaps,
not proof they caused any CI timeout or that an escaped workload existed.

## Required inspector observations

1. New observations are diagnostic only. Preserve selected argv/env, original error kind
   and priority, existing250ms post-spawn observation,1MiB per stream, framing/leader/
   EOF/status gates and existing Unknown/latch equations. Spawn and mandatory inspector
   cleanup/reap remain outside that budget and can exceed it. No total250ms promise.
2. Attribute every failure to an exact static site; stage alone is insufficient:

   | Site | Finite stage | Separate refusal category |
   | --- | --- | --- |
   | leader input before spawn | input_guard | invalid_leader |
   | Command::spawn | spawn | io_kind or unavailable |
   | missing stdout/stderr; nonblocking setup | endpoint_setup + stream | missing_endpoint or io_kind |
   | complete loop entry | deadline_loop_entry | deadline |
   | Stream::drain entry | deadline_drain_entry + stream | deadline |
   | read syscall | stream_read + stream | io_kind |
   | try_reserve | stream_allocation + stream | allocation |
   | deadline after successful read | deadline_after_read + stream | deadline |
   | individual byte cap | stream_budget + stream | overflow |
   | direct-child try_wait | direct_child_status | io_kind |
   | deadline after status poll | deadline_after_status | deadline |
   | exit-success guard | exit_guard | exit_failed |
   | nonempty stderr guard | stderr_guard | stderr_nonempty |
   | validate(frame) | frame_validation | static framing category |
   | deadline after validate | deadline_after_validation | deadline |

   A timeout during loop/reads retains actual stdout/stderr EOF and exit flags; these
   distinguish pending EOF/status without inferring cause. Count every invoked read,
   including WouldBlock and Interrupted; also separate successful-byte count and those
   two finite result counts. Saturating integer counters never overflow or grant success.
   If post-validation deadline masks a validation result, return original TimedOut but
   retain separate unavailable/valid_live/valid_dead/framing category. No arbitrary
   validation text or process row is copied. Unreached measurements are unavailable.
3. Snapshot facts before endpoints drop and before mandatory cleanup; separately record
   actual post-error cleanup fact, never infer it from cleanup() returning Ok. Vocabulary:
   not_reached, reaped_by_status_observation, kill_requested_then_wait_reaped,
   wait_failed_uncertain, relinquished_without_cleanup, unavailable. Record kill outcome
   separately (not_requested, returned_ok, returned_error with finite IO kind, unavailable).
   try_wait ownership error sets unreaped=false: this relinquishes signal authority and
   is NOT verified reap, even when subsequent cleanup returns Ok/no-op. Do not repair
   this ownership loss by cached-PID signals, waits or synthetic cleanup_ok.
4. The first owned observation is authoritative. A later existing ProcessGroup Drop
   retry cannot overwrite its facts or count as new cleanup success; at most record
   a separate unavailable/observed finite retry outcome. Derived Context latch refusals
   report no fresh inspection. Common native cleanup/Drop policy stays unchanged.
5. Carry typed/static bounded facts through existing cleanup/AdapterError and first
   Context failure rendering without message-substring classification. Fixed fact
   formatting must reach retained test/Context error diagnostics and cannot change
   cleanup on formatting failure. Public AdapterError fields need not change. Existing
   Session.saved, generic session failure, and grok.turn_observed scoped event equations
   and schema stay unchanged; Grok cleanup receipt remains unclassified until a separately
   reviewed typed consumer adopts new facts. Update its master rationale accordingly.
6. Exclude argv, executable/repository paths, environment/auth/config, all PID numbers,
   ps rows/stderr text, arbitrary child bytes/native bodies and foreign IDs/names from
   new facts. Counters, enums and elapsed duration only. Separate observation elapsed
   from outside-budget spawn/cleanup timing; elapsed does not prove scheduling, retained
   FD identity, native outcome, workload death or permission. Missing facts never guess.

## Git operation ownership, admission and result rules

7. Define a private process-local in-memory owner pool, not a Store record or workload
   lease. Exactly64 job permits globally bound these component jobs. Before ANY Git
   spawn reserve four permits atomically for one operation: one supervisor, two readers,
   one blocking cleanup job. All four remain reserved while any group/reader/cleanup
   ownership is unresolved; no extra job per timeout/cancel/retry. This caps admitted
   operations at16, reader jobs at32 and all jobs at64, with two output streams capped at1MiB+1 captured bytes each
   per operation, without claiming exact Vec allocation capacity. It permits four-plus concurrent operations, not serialization.
   Saturation refuses before spawn with fixed LaunchFailure/capacity_unavailable, no
   process/readers/cleanup jobs and no new uncertainty for that unstarted operation.
   No caller-provided Arc/new runtime bypasses the process-global cap.
8. The pool retains actual ProcessGroup and both JoinHandles independently of caller
   future or runtime lifetime. One admitted supervisor drives each operation. Native
   blocking kill/inspector work is dispatched only on its pre-reserved blocking job;
   asynchronous Child wait may run on the supervisor while its actual anchor remains
   pool-owned. A
   synchronous caller Drop signals cancellation and retains anchors, never performs
   blocking ProcessGroup Drop on the async poller. This fixes verified observe_exit-
   await/reap-error/reap-timeout Drop gaps, in addition to detached reader handles.
   A Git-specific cleanup failure retains its actual group anchor/flag and pool permits;
   it does not drop it through the common helper's implicit retry. No new group retry,
   adoption or recovery is introduced. Common native ProcessGroup Drop changes require
   separately coordinated impact/source gates and are not silently included here.
9. The actual holder stays in the pool when a supervisor/cleanup executor is unavailable
   or fails, including runtime shutdown. No unbounded fallback task/thread, async-worker
   blocking cleanup or destroying the group's anchor on worker unwind. Runtime shutdown
   alone is not settlement; pending states/permits remain held for separately governed
   recovery. Private observable owner occupancy and join/group states permit causal tests.
   Cancellation before effects must prevent later spawn; cancellation racing dispatch
   retains the operation's uncertainty before caller observation can disappear.
10. Abort is cancellation requested, not completion. Join vocabulary for each reader:
    not_started, joined_returned (read outcome separately ok/IO/budget error), joined_panic,
    joined_cancelled, not_observed. Observed Panic/Cancelled JoinResult proves task future
    and its pipe endpoint dropped, never successful reading or native group death. A
    reader's runtime-shutdown Cancelled result can count only when actually observed;
    shutdown with not_observed handles remains uncertain. Cleanup-worker failure has
    its own group/worker fact and is never a reader join. Both joins must be observed,
    including a pending peer after the first reader errors.
11. Freeze primary result precedence, with reader settlement a separate diagnostic:

    | Highest to lowest existing primary | Required original kind |
    | --- | --- |
    | group cleanup error | SessionLost |
    | reap timeout/error | SessionLost |
    | caller observe deadline | Timeout |
    | child observation error | SessionLost |
    | stdout returned read/join error | original reader kind |
    | stderr returned read/join error after stdout | original reader kind |
    | shared output wait elapsed before prior reads returned | ProcessFailure/output_open |
    | observed unsuccessful Git exit | OwnershipMismatch |
    | successful bytes | success |

    Keep stdout-before-stderr primary ordering: an already-failed stderr cannot replace
    output_open while stdout never returns. Cleanup/reap remain prior to reader errors.
    New pending-peer facts never replace an established primary error with SessionLost.
12. One existing250ms output/reader-settlement budget is used once per operation after
    cleanup/reap or their failure. Output collection, requested abort and observation
    of both joins share the SAME deadline; never add a second250ms abort wait. Early
    cleanup/reap/observe errors request both aborts then use that one budget for joins.
    At budget expiry request abort without resetting the deadline and retain unobserved
    handles in the admitted owner. Cleanup-dispatch await and mandatory inspector reap
    are not hard-bounded; the caller Git deadline is not a total operation-time bound.
    No extra deadline extension or success from an unobserved abort.
13. Selected-group cleanup AND reap plus both reader terminations must be observed before
    this operation's uncertainty can clear. Both-joined read errors and in-budget abort/
    cancelled joins may be settled resource outcomes while the original error remains.
    Unknown group or not_observed joins keep the per-call flag true and deliberately
    propagate Context's existing sticky latch. Later completion cannot clear a flag
    already returned/observed uncertain or that process-wide latch. No false latch on
    fully observed successful settlement/in-budget abort or proven pre-effect refusal.
    Actual late resource settlement may release pool permits only after every retained
    group/reader/cleanup resource is observed settled; it cannot clear the already
    latched flag or mint replay/freshness authority. These are selected-group/reader
    facts only, not whole helper/service/hook settlement.
14. All current callers consume this same bound: Generic validate_git preflight; Grok
    ScopeSnapshot::verify_git ownership through ownership.rs bounded
    Git (initial/refresh consumers); Grok index_digest through mod.rs bounded_git_raw; Context
    git_value_owned/git_value; scalar bounded_git wrapper and private regression helper.
    Current native/Context flag isolation remains; operation-local facts must not confuse
    a peer's live flag with its own uncertainty. Linux KILL/ESRCH semantics remain; Git
    error-retention changes are reviewed explicitly across both platforms. Synchronous
    git.rs commands and future #5/#6/#19 copied callers are inventory/composition work,
    not silently migrated or claimed fully owned by this component.
15. No schema/Store ownership, native auth/hooks/defaults, permission/protocol/environment,
    scheduler policy or kernel backend changes. Full Issue60 private workload owner,
    durable reservation, readiness/default-deny delegation and effect matrix remain
    pending. A process-local reader holder is not that workload certificate.

## Causal verification and gates

- Real controlled inspector subprocess fixtures reach retained stdout/stderr endpoints,
  pending/observed direct-child status and valid/invalid frames. Input/spawn failures
  use unstarted paths. Missing endpoint/fcntl/read/allocation/status errors require
  labelled private injection/unit seams, not claims real ps naturally reached them.
  Map every deadline/guard site, preserve original kind and masked validation category.
  A status-error seam asserts relinquished_without_cleanup and Unknown; removing that
  distinction must fail an intended consumer assertion. No cached-PID recovery.
- First bounded Git/Context own failure rendering contains safe facts; later latch
  refusals contain no invented new sample. Independently observe both-stream counters
  and cleanup status; discard diagnostics as authority. Compiled stage/fact guard
  omissions earn actual caller credit only when reaching that caller, units stay units.
- Real bounded Git pipes/tasks cover cleanup/reap/observation/read errors, output-open
  and caller Drop before/after dispatch/during output. Independently pending peer tests
  cannot be masked by the first reader's error. Inspect actual pool occupancy, joined
  endpoints and original kind/latch equations. Cancel/Panic join termination is distinct
  from read success; unavailable executor/shutdown retains unobserved owners/permits.
- Positive Context consumers leave latch clear after complete successful settlement and
  in-budget abort-and-observed-join. A mutant that always latches read error/cancel must
  fail those positives. A later settlement cannot undo a previously latched Unknown.
- Pre-spawn saturation of ALL64 permits creates no new child; four operations genuinely
  progress concurrently with real owned outputs. Mutants removing admission, retaining
  only readers or only groups, dropping peer handles, omitting abort/join or clearing
  uncertainty early must fail independently reached causal consumers. Shared Drop code
  stays unchanged or triggers separate explicitly reviewed impact/source gates.
- Commit requirements/design before independent native gates; implementation/impact
  commit precedes tests/source reviews. Verify/fix/re-review actual findings. Default
  affected/full debug/release, fmt/Clippy/builds and final Linux/macOS public CI preserve
  raw failure logs and actual checkout provenance. No blind green-only rerun, default
  serialization, larger deadline, or weaker Unknown/PID/permission guard.

## Acceptance limit

This component improves diagnostic/lifetime observability and actual owned reader
retention. It does not itself fix the current inspection availability cause or prove a
new backend faster/complete. Native separate-PGID/SID descendants, partial kill success,
non-atomic fork/exit samples, actual runtime conflict/effect enforcement, all reachable
Git delegation, durable workload reservations, recovery14 and aggregate native16 remain
explicit pending Issue60/provider requirements. Diagnostic completeness or process-group
death alone cannot close those gates.
