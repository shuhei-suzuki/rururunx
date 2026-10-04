# Issue60 component requirements: bounded inspector failure diagnostics

Risk: STRICT for shared native process observation/error and uncertainty boundaries.
Status: Requirements4 diagnostic-only approvals at26de8da; source-aside correction
verified without changing its normative real-inspector criterion. Design2 approved
twice at201bd89; diagnostic source implemented. Source1 corrections/independent
re-review and final CI pending. This status update changes no normative criterion.
Parent [Issue60](https://github.com/shuhei-suzuki/rururunx/issues/60) retains its full
runtime workload/effect/delegation/settlement acceptance. This component does not close it.

## Verified current source and failures

Baseline main054aefd3a22a02951fa72b9fa397bed58d156bbf invokes trusted env-cleared
/bin/ps -g OWNED_LEADER -o pid=,pgid=,stat= on macOS, with no COMMAND_MODE
override (production UNIX2003 semantics; legacy is a negative fixture only). Existing post-spawn250ms
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
but do not observe joins. Caller future Drop also loses these reader handles. Caller
Drop during observe_exit, before cleanup, can run blocking ProcessGroup Drop on the
async polling worker. Successful kill_group sets group_owned=false: subsequent reap
timeout/error loses the unreaped Child anchor to Tokio kill_on_drop/orphan reaping
while uncertainty remains, without a blocking group Drop. After successful reap an
observation error has the reader-handle gap only. Cleanup error runs its existing Drop
retry on the blocking worker; a worker JoinError does not prove group settlement. These are verified lifetime gaps,
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
   distinguish pending EOF/status without inferring cause. Keep a separate finite observed
   exit category: unavailable, success, nonzero_exit or signaled/other; no PID or arbitrary
   status text. Count each actually invoked child.try_wait separately, including
   returned pending, Interrupted or error; cached exit requires no new poll count.
   Count every invoked read,
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
   retry cannot overwrite its facts or count as new cleanup success. ProcessGroup Drop
   discards its result: its retry outcome/facts are always unavailable at caller sinks
   in this component, including caller Drop before cleanup. No new log/stderr/global
   sink is added for discarded results. Derived Context latch refusals
   report no fresh inspection. Common native cleanup/Drop policy stays unchanged.
5. Carry typed/static bounded facts through existing cleanup/AdapterError and first
   Context failure rendering without message-substring classification. Fixed fact
   formatting must reach retained test/Context error diagnostics and cannot change
   cleanup on formatting failure. New bounded fixed fact text may enter EXISTING failure
   messages; their exact bytes are not frozen. Inventory Context first-error rendering,
   Generic adapter.launch_failure.reason audit and runtime SessionStatus.failure, and
   Grok turn_observed.diagnostic, turn_observed.reconciliation_error and runtime
   SessionStatus.failure, including the existing unknown-dispatch diagnostic wrapper.
   Shared native agent-group kill_group cleanup users can reach these same sinks;
   enumerate them alongside bounded Git. session.saved projects exactly state, agent,
   provider, role, native_ref and recovery.dispatch_intent, not failure facts, and
   must remain unchanged.
   Copied future provider consumers require composition review. No new audit authority
   or event schema follows from these facts. Public AdapterError fields need not change. Existing
   session.saved, generic session failure, and grok.turn_observed scoped event equations
   and schema stay unchanged; Grok cleanup receipt remains unclassified until a separately
   reviewed typed consumer adopts new facts. Update its master rationale accordingly.
6. Exclude argv, executable/repository paths, environment/auth/config, all PID numbers,
   ps rows/stderr text, arbitrary child bytes/native bodies and foreign IDs/names from
   new facts. Counters, enums and elapsed duration only. Separate observation elapsed
   from outside-budget spawn/cleanup timing; elapsed does not prove scheduling, retained
   FD identity, native outcome, workload death or permission. Missing facts never guess.

## Diagnostic component boundary

7. This component changes only inspector failure observations and their safe existing
   error rendering. Existing Git reader handles, abort/join behavior, ProcessGroup
   new/reap/Drop, caller flags, pool/admission/runtime drivers and primary ordering are
   UNCHANGED. ALL verified gaps above remain open: detached readers, unreaped anchor
   loss, pre-cleanup caller Drop blocking kill/inspector on the async poller, cleanup
   Drop retry on its blocking worker, and uncertain worker JoinError.
   The [Git reader/driver contract draft](issue-60-git-reader-contract-draft.md) is explicitly
   unapproved; no diagnostic acceptance grants it requirements/design/source approval.
8. Diagnostics cannot change any group_owned/unreaped/exit transition, signal choice,
   selected argv/env, framing result, EOF requirement, direct-child status check,
   original ErrorKind/priority, stream cap, deadline or mandatory wait. Keep the observed
   first failure's kind when later cleanup fails; capture that cleanup uncertainty
   separately. No extra attempt, wait, retry, cached-PID signal, new worker or global
   process dump is introduced. A prior existing Drop retry cannot overwrite first facts.
9. Preserve shared caller outcome equations and inputs for this component: Context's
   sticky latch, Generic Reservation Lost/Failed choice, Grok uncertainty_by_stage and
   clean receipt operand. Fact fields are never parsed to manufacture cleanup_ok,
   process/native outcome, freshness, replay permission or reservation release. Grok's
   existing unclassified cleanup-error provenance stays unclassified unless a later
   separately reviewed typed consumer adopts the facts. Native audit schemas/ownership,
   auth/hooks/defaults/permission/environment, scheduler and Store are unchanged.
10. Inventory every actual inspector route: resolve_macos_signal_result EPERM handling,
    ordinary cleanup_group and ProcessGroup Drop; Generic post-spawn Running-write
    failure cleanup (adapter668), Generic terminal supervision cleanup (1512), Grok
    native terminal supervision cleanup (grok/mod.rs1138), and Generic/Grok ownership/
    index/Context bounded Git error propagation. Drop-route errors stay discarded. New fixed facts can decorate existing
    bounded failure messages, not change their authority. Linux native semantics remain
    unchanged; macOS-only fact collection must not add a Linux inspector. Synchronous
    git.rs and future provider copies remain separate composition/inventory work.
11. Diagnostic collection is bounded and value-free even on its own failures. Missing
    or unallocatable measurements are unavailable; diagnostic formatting failure must
    neither suppress original error/cleanup nor grant success. Observation elapsed is
    separate from outside-budget spawn/mandatory cleanup. No250ms total-time promise
    or performance claim. Keep finite bytes/read/status counters and static categories;
    no raw frame/stderr/config/path/PID appears in new errors or retained fact artifacts.

## Causal verification and gates

- Real controlled inspector subprocess fixtures reach retained stdout/stderr endpoints,
  pending/observed direct-child status and valid/invalid frames. Input/spawn failures
  use unstarted paths. Missing endpoint/fcntl/read/allocation/status errors require
  labelled private injection/unit seams, not claims real ps naturally reached them.
  Each adjacent deadline site needs a deterministic private clock/site seam, labelled
  injected; timing fixtures cannot earn exact-site credit. Preserve original category
  and masked validation separately. A status-error seam proves
  relinquished_without_cleanup != actual reap, and never creates cached-PID authority.
- Tests observe safe counters/EOF/exit/cleanup states at their actual inspector source
  before rendering. Compiled omissions of site, fact or relinquishment guard must fail
  intended assertions; unit facts earn unit credit only. First bounded Git/Context
  own-failure rendering must carry safe facts from the REAL inspection::inspect_command
  executed through kill_group→resolve_macos_signal_result EPERM handling. A private
  per-operation labelled seam may substitute only controlled executable/prefix or
  failure/clock site; preserve the production selected argv/env/framing/cleanup code.
  Existing TestPlan::inspect (inspection.rs872–914) ALREADY invokes the actual
  inspect_with_prefix→inspect_command through a controlled /bin/sh prefix, preserving
  selected argv/env/stdio/guards/cleanup; it does not fabricate an observation result.
  KillAndUnknown plan.signal performs actual KILL on the owned group, then injects
  PERM to reach the resolver: label that injected signal result and shim frame, never
  claim actual OS permission failure or installed /bin/ps acceptance. That existing
  real-inspector route is eligible; no new executable seam is required merely to reach
  it. Only a future hand-built fact/error/result bypass would earn pass-through credit.
  No hand-built fact error satisfies this gate. A compiled omission of fact construction
  or attachment at inspector→resolver must fail the actual Context consumer assertion.
  Derived latch refusal has no fresh sample.
  Reach actual Generic/Grok propagation where accessible and disclose any test-only
  adapter seam or historical consumer left unexercised. No inventing diagnostic reach.
- Positive native/fixture valid_live/valid_dead, cleanup and original primary-error
  controls keep source outcomes unchanged. Mutants turning diagnostic unavailable or
  relinquished status into cleanup success must fail a real authority consumer or be
  labelled as unit guard coverage. Diagnostics-only tests cannot earn reader-retention,
  workload death, speed or whole F1 credit. Existing timeout/ownership evidence survives.
- Commit requirements before two independent narrowed native requirement gates, then
  committed diagnostic design before two independent design gates. Source/impact commit
  precedes tests/immutable independent source review. Verify/fix/re-review findings.
  Default affected/full debug/release, fmt/Clippy/builds and final both-OS public CI retain
  raw failure logs and actual checkout provenance. A new reviewed diagnostic source can
  be measured by its own CI once; no old-head rerun, serialization, larger deadline or
  weaker Unknown/PID/permission guard. Any final required red remains a merge blocker.

## Acceptance limit

This component adds bounded diagnostic observability only. Every baseline Drop/retry/
reader/unreaped-anchor/JoinError gap remains unchanged and open. Git reader/driver retention
and published-flag corrections remain an explicitly unapproved separate draft. It does not itself fix the current inspection availability cause or prove a
new backend faster/complete. Native separate-PGID/SID descendants, partial kill success,
non-atomic fork/exit samples, actual runtime conflict/effect enforcement, all reachable
Git delegation, durable workload reservations, recovery14 and aggregate native16 remain
explicit pending Issue60/provider requirements. Diagnostic completeness or process-group
death alone cannot close those gates.
