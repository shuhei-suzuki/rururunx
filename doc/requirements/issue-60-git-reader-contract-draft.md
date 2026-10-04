# Issue60 Git reader/driver contract draft — UNAPPROVED

Risk: STRICT. Status: unapproved partial contract; requirements/design/source gates
have NOT passed. This is NOT part of the diagnostic component
implementation/acceptance. Combined Requirements2 and the separate reader Requirements1
both had request_changes from both independent reviewers. Candidate clarifications
below address verified source/lifetime issues but have not been re-reviewed. No
pool/driver/publication code may be inferred approved from the diagnostic gate. Parent
Issue60 full runtime workload/effect/delegation/durable settlement remains open.

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
   exact Vec allocation. Four operations must genuinely progress concurrently. Do not
   assume16 covers default peak load: wait for atomic admission only within the
   existing caller deadline, using no new per-waiter task/thread. An already-expired
   deadline wins before admission; admission expiry returns original Timeout before
   effects, with finite capacity_unavailable facts. Report saturating active and
   retained-unresolved counts, no identities; retained occupancy denotes derived
   pressure, not a fresh inspection. Proven pre-effect refusal creates no child/job or
   new uncertainty. All production callers resolve the single global pool, regardless
   of caller Arc/runtime. Only cfg(test) permits an explicit private pool for
   saturation, shutdown, panic and unresolved cleanup experiments; production cannot
   construct it. Route existing seven adapter unknown-plan calls and the Context
   unknown-plan call, plus new Generic/Grok saturation or Unknown consumers, through
   this cfg(test)-only private pool using the actual consumer path, not a copied
   helper. Production retained occupancy must be unchanged by these experiments.
   Private-pool teardown either settles actual owned anchors on its reserved non-poller
   lane before release or deliberately retains them; it never invokes blocking group
   Drop on an async poller or releases unresolved resources. No numeric fixture rescue.
   These private experiments cannot consume the production pool or serialize other
   tests. In production unresolved holders can permanently consume capacity until
   actual observed settlement; no recovery/release port exists here. Exhaustion can
   persist for the process lifetime. Process restart does not prove old workload death
   or recover those process-local anchors; full Issue60 durable recovery remains
   pending.
8. The pool retains the actual native Child BEFORE ALL fallible runtime wrapping, stdio/
   signal registration and group binding, then group and reader anchors independently
   of caller future/runtime. A Tokio Command::spawn Err is not uniformly pre-effect:
   locked1.53.1 first creates a std Child, then fallibly builds its runtime wrapper.
   Design must preserve that actual native Child, through a Git-local std Child owner
   or a genuinely equivalent reviewed ownership primitive; no conversion back from
   numeric PID, no outside waitpid/reaping detached from the owning Child. Child,
   stdio, readers, exit observation and reap are created and driven by the independent
   owning context that survives caller runtime shutdown. Moving handles from the caller
   runtime alone is insufficient; verify against locked Tokio basis. A bounded owner
   driver and its reserved cleanup execution lane must be available before effects;
   startup failure is pre-effect LaunchFailure. Caller runtime shutdown or supervisor
   cancellation/panic must not remove that lane. Before spawn an atomic
   admitted→spawning transition publishes caller uncertainty=true. Caller Drop winning
   admitted→cancelled prevents spawn and never sets uncertainty. Drop after spawning
   freezes the flag true unless a live in-budget return had already cleared it; driver
   observation or result-ready alone cannot clear it. Mandatory clear points are the
   live in-budget complete-settlement return and the live in-budget genuine native
   OS-spawn failure return proving no surviving child remains under the verified
   native spawn profile. Err/no returned handle alone is not that proof. Runtime/stdio/signal
   wrapping failure AFTER native spawn is not that exception: retain its actual Child
   and original ProcessFailure or LaunchFailure; it is not a pre-effect refusal.
   Unknown remains until actual complete settlement is observed on a live in-budget
   return; a genuinely settled initialization error MUST clear the resource flag while
   preserving that original error. Merely returning Err without a Tokio Child never
   proves such settlement. Late no-effect observation cannot clear a dropped/returned
   flag. Group binding failure after spawn retains the actual Child as Unknown;
   original LaunchFailure survives and never becomes a pre-effect refusal. Never adopt
   a PID. The Git owner uses an INTERNAL flag, not the caller's published flag.
   Locked Tokio cannot supply a tokio Child without consuming its native std Child
   through fallible wrapping. The Git anchor therefore stays a std Child; it cannot
   be the current Tokio-Child-bound ProcessGroup unless a reviewed public ownership-
   preserving API is established. Preserve the existing selected-group semantics
   through the independently reviewed port/equivalent below. Do not silently change
   shared ProcessGroup::new/reap/Drop or group/observation primitives. Design must
   explicitly resolve actual Child retention across binding failure and nonblocking
   caller cancellation, through a Git-local owner or separately impact-reviewed
   additive shared port. No such port or implementation is approved by the prior
   diagnostic gate. Those two explicit live return clear points are the sole
   linearization; never clear from a late internal reap. A caller that observed/dropped
   Unknown freezes its flag true permanently. One supervisor drives the operation,
   transferring exclusive actual anchors to the reserved blocking job for native
   kill/inspection and back by result channel. Caller Drop uses a nonblocking
   cancellation signal. Admission, Drop, supervision and result bookkeeping never
   lock/wait on state held across kill_group, inspection or reap; neither Store locks
   nor async polling workers may block on native cleanup. This addresses blocking
   pre-cleanup caller Drop, unreaped anchor loss after reap failure, and both-reader
   detach gaps, with distinct consumer evidence for each actual site.
9. If first cleanup has not run, cancellation, supervisor failure or caller-runtime
   shutdown must dispatch the first selected-group cleanup on the already-reserved,
   pool-owned non-poller cleanup lane. Preserve existing semantics: KILL to the exact
   group derived from the actual unreaped owned Child; Linux success/SRCH succeeds;
   macOS PERM succeeds only after valid-dead trusted inspection, otherwise Unknown;
   group ownership clears only on success; unchanged250ms inspector/query/environment;
   no implicit Drop retry. This preserves the existing bounded group result, not a
   new whole-workload death certificate. Use either an impact-gated additive shared
   primitive separating group signal/exit observation/inspection-plan seam from the
   Tokio Child, or a reviewed Git-local equivalent with differential controls and
   parity mutants on both OS. The current7+1 Unknown fixtures must reach the same
   inspection plan through actual consumer routes. A production Git-local equivalent
   is an explicitly reviewed implementation choice; the copied-helper exclusion in
   acceptance bars test substitutes, not that choice. This is first cleanup, not a
   retry. Require an
   actual shutdown-before-dispatch consumer; mere static-pool retention cannot replace
   the baseline shutdown first kill. When the reserved supervisor remains healthy,
   successful selected-group cleanup proceeds to owning-Child reap and both joins. The
   SAME reserved supervisor continues observing those existing resources after a
   caller-visible timeout/Unknown; no new job is created. Continuing a pending
   owning-Child exit observation/settlement or existing JoinHandles is
   observation, not command/signal retry. Preserve the unreaped leader until group
   cleanup is established: non-reaping waitid NOWAIT/SIGCHLD observation then the
   actual std Child try_wait, or owning Child wait on counted reserved capacity, must
   not introduce early reap, a second deadline, outside-Child reap or a hidden job.
   Design must choose the mechanism explicitly. A returned reap error is terminal retained
   Unknown: no re-wait/PID rescue after lost wait authority. When the supervisor is
   cancelled/panics, the reserved cleanup lane still performs first cleanup if needed,
   but no replacement observer or reap/join completion is promised: retain all four
   permits/anchors as Unknown. Cleanup failure likewise retains the actual
   anchor/permits/Unknown without the common helper's implicit retry. No command or
   signal retry, PID adoption or process-restart recovery. If the independent driver
   itself fails, retain anchors in process-local Unknown; availability/settlement is
   not promised under total driver loss. Pool initialization, panic and poisoned
   bookkeeping must fail closed without unwinding/dropping anchors or silently
   resetting permits. No unbounded fallback/hold-join thread/task is allowed. Pending
   anchors remain held on cleanup-worker failure or unavailable executor; runtime
   shutdown is not proof of settlement. Actual driver lifecycle/first-cleanup delivery
   and retained panic anchors need design/source gates. Common native ProcessGroup
   changes require separately coordinated full actual native/Generic impact and source
   gates; this candidate authorizes no shared source edit before those
   requirements/design gates.
10. Abort is cancellation requested, not completion. Join vocabulary for each reader:
    not_started, joined_returned (read outcome separately ok/IO/budget error),
    joined_panic, joined_cancelled, not_observed. Each reader has an independent
    abort_requested bit, not inferred from the join category. Observed Panic/Cancelled
    JoinResult proves task future and its pipe endpoint dropped, never successful
    reading or native group death. A reader's runtime-shutdown Cancelled result can
    count only when actually observed; shutdown with not_observed handles remains
    uncertain. Cleanup-worker failure has its own group/worker fact and is never a
    reader join. Both joins must be observed, including a pending peer after the first
    reader errors.
11. Freeze primary result precedence, with reader settlement a separate diagnostic:

    Startup order: already-expired caller deadline→Timeout; bounded admission expiry
    →Timeout with capacity facts; unavailable driver before effects→LaunchFailure;
    genuine native OS-spawn failure/proven no surviving child→ProcessFailure. This
    proof is specific to the actual Linux/macOS CI toolchain and native command
    recipe, not general std or Tokio Err. Design must verify exact primary spawn-
    error paths; Rust1.91.1 optional Linux create_pidfd has a post-spawn Err path,
    whereas the default command uses create_pidfd=false and no pre_exec callbacks.
    Unsupported/unverified spawn profiles refuse before effects. No surviving child
    is a scoped resource fact, not a durable no-effect certificate; std fork/exec
    error can create then internally wait/reap a child before returning Err.
    Tokio wrapper,
    stdio/signal or reader initialization failure after an actual Child was produced
    retains that Child and original initialization kind; incomplete settlement remains
    Unknown. A live in-budget fully observed settlement MUST clear the resource flag,
    but it is never a pre-effect refusal. After spawn, failed group binding retains the
    actual child/Unknown and original LaunchFailure. Cancellation that wins before
    spawn prevents effects; later cancellation is not a successful pre-effect refusal.
    For post-spawn initialization error, first attempt the same reserved cleanup and
    settlement: cleanup/reap failure wins as SessionLost; otherwise preserve the
    original initialization error with separate settlement facts. A group-binding
    error with no established signal identity remains original LaunchFailure/Unknown,
    with no invented group action. Once initialized, keep the following original
    primary priority:

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
    Immediately abort a pending peer after the first primary read/join error is
    observed in this stdout-before-stderr collection order; observe both joins within
    the single deadline. Do not let earlier uncollected stderr failure reorder the
    primary result.
12. One existing250ms output/reader-settlement budget is used once per operation after
    cleanup/reap or their failure. Output collection, requested abort and observation
    of both joins share the SAME deadline; never add a second250ms abort wait. Early
    cleanup/reap/child-observation errors AND caller-observe-deadline Timeout request
    both aborts then use that one budget for joins. At budget expiry request abort
    without resetting the deadline and retain unobserved handles in the admitted owner.
    Cleanup-dispatch await and mandatory inspector reap are not hard-bounded; the
    caller Git deadline is not a total operation-time bound. No extra deadline
    extension or success from an unobserved abort. output_open at deadline has
    not_observed by construction, even if a task terminates just afterwards; that
    conservative published Unknown is intentional, not a cleanup-success claim.
13. Selected-group cleanup AND reap plus both reader terminations must be observed before
    this operation's uncertainty can clear. Both-joined read errors and in-budget
    abort/cancelled joins may be settled resource outcomes while the original error
    remains. Unknown group or not_observed joins keep the per-call flag true and
    deliberately propagate Context's existing sticky latch. Later completion cannot
    clear a flag already returned/observed uncertain or that process-wide latch. No
    false latch on fully observed successful settlement/in-budget abort or proven
    pre-effect refusal. A healthy reserved supervisor MUST continue late observation
    and release all four permits only after actual group/Child/readers/cleanup jobs
    have settled and the supervisor itself has completed. The reserved supervisor
    slot is driver-owned: its bounded execution frame observes completion and destroys
    the inner supervisor future before the terminal slot-release action. No extra
    observer/monitor job is created. Driver/core setup and this frame stay within
    reserved accounting, with no native/blocking work after release; publishing data
    alone is not completion. Design must verify this exact linearization. A terminal group/reap error or lost
    supervisor/driver retains all four with no late observation guarantee. Healthy late
    settlement cannot clear the already latched flag or mint replay/freshness
    authority. These are selected-group/reader facts only, not whole
    helper/service/hook settlement. Caller outcomes are explicit: not_observed now
    keeps Context sticky uncertainty. Explicitly change and impact-gate the Generic
    live launch-error consumer: persist Lost when ErrorKind is SessionLost OR its own
    bounded-call uncertainty flag is retained true; preserve the original API error
    kind. Existing Reservation::drop already uses the flag, but the live error path
    currently ignores it and disarms. No rewrite to SessionLost as a substitute. Grok
    sampled launch/reconciliation ownership makes its receipt clean operand false. Grok
    checkpoint uses a discarded local ownership object: Unknown returns Err plus pool
    retention only, with no receipt/Session/durable reservation effect;
    checkpoint-to-resume conflict enforcement is explicitly OPEN, not implicitly fixed.
    Both-joined read failures/in-budget observed aborts with proven group cleanup/reap
    remain resource-settled: preserve original failure and avoid falsely forcing those
    callers to Lost/unclean. All other equations stay unchanged; the explicit Generic
    live-error safety change is the exception; these conservative input changes require
    real consumer tests. Context flag reads at626/662 must not see a late internal
    clear between them.
14. All current callers consume this same bound: Generic validate_git preflight; Grok
    ScopeSnapshot::verify_git ownership through ownership.rs bounded Git
    (initial/refresh AND checkpoint consumers); Grok index_digest through mod.rs
    bounded_git_raw; Context git_value_owned/git_value; scalar bounded_git wrapper and
    private regression helper. Current native/Context flag isolation remains;
    operation-local facts must not confuse a peer's live flag with its own uncertainty.
    Linux KILL/ESRCH semantics remain; Git error-retention changes are reviewed
    explicitly across both platforms. Synchronous git.rs commands and future #5/#6/#19
    copied callers are inventory/composition work, not silently migrated or claimed
    fully owned by this component. Generic native executor
    constructor/abort-without-join and Grok native constructor/stderr abort-
    without-join are also explicitly unchanged/unmigrated; shared-port visibility or a
    Git-local owner never certifies their lifetime. The Tokio spawn-wrapper Err
    paths at Generic adapter.rs623–625 and Grok mod.rs1154 are also OPEN: an actual
    native process may exist before its native owner/flag is registered, so the
    current reservation can be released. The proposed Generic Git-uncertainty Lost
    predicate does not cover executor spawn. No actual occurrence is claimed.
    Global64 capacity couples Projects;
    retained holders can reduce/exhaust peer capacity. Project isolation and fairness
    (#50), and actual four-progress under unresolved global holds remain OPEN. Require
    at least four progress with healthy driver and sufficient free capacity, not an
    unconditional claim under permanent exhaustion.
15. No schema/Store ownership, native auth/hooks/defaults, permission/protocol/environment,
    scheduler policy or kernel backend changes. Full Issue60 private workload owner,
    durable reservation, readiness/default-deny delegation and effect matrix remain
    pending. A process-local reader holder is not that workload certificate.

## Separate pending verification

Actual group/reader holders, first cleanup on shutdown, poison/panic retention, no lock
across native work, private-test pool isolation, bounded admission/routing,
caller/publication linearization, Context626/662 late-clear race, Generic Lost/Failed
and Grok receipt input changes each require their own formal requirements/design and
source consumer gates. Test ordinary success and resource-settled errors as well as
Unknown. Use independently reached consumer mutants, preserving default concurrency,
64 actual job capacities and finite existing budgets. No production implementation,
availability improvement, recovery or full enabled-workload settlement is claimed.
Shared native Drop/new/reap changes require separately coordinated actual-consumer
impact/design/source gates; the formal reader candidate resolves the previously stated
unchanged-common assumption explicitly before implementation.
