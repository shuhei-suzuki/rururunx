# Issue60 Git reader/driver contract draft — UNAPPROVED

Risk: STRICT. Status: unapproved partial contract; requirements/design/source gates
have NOT passed. This is NOT part of the diagnostic component implementation/acceptance.
Combined Requirements2 had request_changes from both independent reviewers. Candidate
clarifications below address verified source/lifetime issues but have not been re-reviewed.
No pool/driver/publication code may be inferred approved from the diagnostic gate.
Parent Issue60 full runtime workload/effect/delegation/durable settlement remains open.

## Git operation ownership, admission and result rules

7. Define a private process-local in-memory owner pool, not a Store record or workload
   lease. Exactly64 job permits globally bound ALL component supervisor/reader/cleanup
   work. Before ANY Git spawn reserve four permits atomically: one supervisor, two
   readers, one blocking cleanup job. All four remain reserved while any group/reader/
   cleanup ownership is unresolved; no extra replacement, fallback, monitor or retry
   job per cancellation/timeout. Driver setup must fit these reserved capacities; it
   cannot hide a further unbounded queue or worker. At most16 operations and32 readers
   are admitted. Git captures at most OUTPUT_LIMIT+1 bytes per stream (OUTPUT_LIMIT =
   64*1024 at adapter.rs486; inspector LIMIT =1024*1024 is separate), not a claim about
   exact Vec allocation. Four operations must genuinely progress concurrently.
   Do not assume16 covers default peak load: wait for atomic admission only within the
   existing caller deadline, using no new per-waiter task/thread. An already-expired
   deadline wins before admission; admission expiry returns original Timeout before
   effects, with finite capacity_unavailable facts. Report saturating active and
   retained-unresolved counts, no identities; retained occupancy denotes derived
   pressure, not a fresh inspection. Proven pre-effect refusal creates no child/job
   or new uncertainty. All production callers resolve the single global pool, regardless
   of caller Arc/runtime. Only cfg(test) permits an explicit private pool for saturation,
   shutdown, panic and unresolved cleanup experiments; production cannot construct it.
   These private experiments cannot consume the production pool or serialize other tests.
   In production unresolved holders can permanently consume capacity until actual
   observed settlement; no recovery/release port exists here. Exhaustion can persist
   for the process lifetime. Process restart does not prove old workload death or
   recover those process-local anchors; full Issue60 durable recovery remains pending.
8. The pool retains the actual spawned Child before fallible group binding, then the
   ProcessGroup and both reader JoinHandles independently of caller future/runtime.
   A bounded owner driver and its reserved cleanup execution lane must be available
   before effects; startup failure is pre-effect LaunchFailure. Caller runtime shutdown
   or supervisor cancellation/panic must not remove that lane. Before spawn an atomic
   admitted→spawning transition publishes caller uncertainty=true. Caller Drop winning
   admitted→cancelled prevents spawn and publishes no new uncertainty; Drop after
   spawning conservatively freezes uncertainty until complete settlement was already
   observed. Spawn failure may clear only on the live caller's proven no-effect return.
   Group binding failure after spawn retains the actual Child as Unknown; original
   LaunchFailure survives and never becomes a pre-effect refusal. Never adopt a PID.
   Git supplies ProcessGroup an INTERNAL flag, not the caller's published flag. Shared
   ProcessGroup::new/reap/Drop behavior is unchanged. The caller flag clears once only
   on the live, in-budget complete-settlement return, never from a late internal reap.
   A caller that observed/dropped Unknown freezes its flag true permanently.
   One supervisor drives the operation, transferring exclusive actual anchors to the
   reserved blocking job for native kill/inspection and back by result channel. Caller
   Drop uses a nonblocking cancellation signal. Admission, Drop, supervision and result
   bookkeeping never lock/wait on state held across kill_group, inspection or reap;
   neither Store locks nor async polling workers may block on native cleanup. This
   addresses blocking pre-cleanup caller Drop, unreaped anchor loss after reap failure,
   and both-reader detach gaps, with distinct consumer evidence for each actual site.
9. If first cleanup has not run, cancellation, supervisor failure or caller-runtime
   shutdown must dispatch the EXISTING first kill_group on the already-reserved,
   pool-owned non-poller cleanup lane. This is first cleanup, not a retry. Require an
   actual shutdown-before-dispatch consumer; mere static-pool retention cannot replace
   the baseline shutdown first kill. Successful selected-group cleanup proceeds to
   observed reap and joins; failure retains the actual anchor/permits/Unknown, without
   dropping through the common helper's implicit retry. No new retry, PID adoption or
   process-restart recovery is introduced. If the independent driver itself fails,
   retain anchors in process-local Unknown; availability/settlement is not promised
   under total driver loss. Pool initialization, panic and poisoned bookkeeping must
   fail closed without unwinding/dropping anchors or silently resetting permits.
   No unbounded fallback/hold-join thread/task is allowed. Pending anchors remain held
   on cleanup-worker failure or unavailable executor; runtime shutdown is not proof
   of settlement. Actual driver lifecycle/first-cleanup delivery and retained panic
   anchors need design/source gates. Common native ProcessGroup Drop changes require
   separately coordinated impact/source gates and are not silently included here.
10. Abort is cancellation requested, not completion. Join vocabulary for each reader:
    not_started, joined_returned (read outcome separately ok/IO/budget error), joined_panic,
    joined_cancelled, not_observed. Each reader has an independent abort_requested bit,
    not inferred from the join category. Observed Panic/Cancelled JoinResult proves task future
    and its pipe endpoint dropped, never successful reading or native group death. A
    reader's runtime-shutdown Cancelled result can count only when actually observed;
    shutdown with not_observed handles remains uncertain. Cleanup-worker failure has
    its own group/worker fact and is never a reader join. Both joins must be observed,
    including a pending peer after the first reader errors.
11. Freeze primary result precedence, with reader settlement a separate diagnostic:

    Pre-effect order: already-expired caller deadline→Timeout; bounded admission expiry
    →Timeout with capacity facts; unavailable driver before effects→LaunchFailure;
    Command::spawn failure→ProcessFailure. After spawn, failed group binding retains
    the actual child/Unknown and original LaunchFailure. Cancellation that wins before
    spawn prevents effects; later cancellation is not a successful pre-effect refusal.
    Once anchored, keep the following original primary priority:

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
    Immediately abort a pending peer after the first primary read/join error is observed
    in this stdout-before-stderr collection order; observe both joins within the single
    deadline. Do not let earlier uncollected stderr failure reorder the primary result.
12. One existing250ms output/reader-settlement budget is used once per operation after
    cleanup/reap or their failure. Output collection, requested abort and observation
    of both joins share the SAME deadline; never add a second250ms abort wait. Early
    cleanup/reap/observe errors request both aborts then use that one budget for joins.
    At budget expiry request abort without resetting the deadline and retain unobserved
    handles in the admitted owner. Cleanup-dispatch await and mandatory inspector reap
    are not hard-bounded; the caller Git deadline is not a total operation-time bound.
    No extra deadline extension or success from an unobserved abort. output_open at
    deadline has not_observed by construction, even if a task terminates just afterwards;
    that conservative published Unknown is intentional, not a cleanup-success claim.
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
    Caller outcomes are explicit: not_observed now keeps Context sticky uncertainty,
    Generic Reservation chooses Lost rather than Failed, and Grok stage uncertainty
    makes its receipt clean operand false. Both-joined read failures/in-budget observed
    aborts with proven group cleanup/reap remain resource-settled: preserve original
    failure and avoid falsely forcing those callers to Lost/unclean. Existing equations
    stay unchanged; these conservative input changes require real consumer tests.
    Context flag reads at626/662 must not see a late internal clear between them.
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

## Separate pending verification

Actual group/reader holders, first cleanup on shutdown, poison/panic retention,
no lock across native work, private-test pool isolation, bounded admission/routing,
caller/publication linearization, Context626/662 late-clear race, Generic Lost/Failed
and Grok receipt input changes each require their own formal requirements/design and
source consumer gates. Test ordinary success and resource-settled errors as well as
Unknown. Use independently reached consumer mutants, preserving default concurrency,
64 actual job capacities and finite existing budgets. No production implementation,
availability improvement, recovery or full enabled-workload settlement is claimed.
Shared native Drop/new/reap changes require separately coordinated impact/source gates.
