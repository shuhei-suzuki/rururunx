# Issue60 selected Git owner/readers — Design2 candidate

Risk: STRICT. Requirements4 at a8e8a8b has two independent APPROVE/no C/H/M;
its four verified Low refinements appear in this design and the requirements
outcome ledger. Design1 at8f95adb returned request_changes from both independent reviewers.
All16 findings are recorded and verified below; this Design2 is UNAPPROVED. No reader source, test, availability
or F1 acceptance exists. Existing shared native ProcessGroup/new/reap/Drop stays
unchanged. Current source references originally used public8f95adb; Design2 normally
composes main47830b0. Incoming common changes are additive visibility/trait methods,
Grok structured delegation and locked Tokio1.53.1 net features/dependencies, not
changed selected Git/cleanup logic. Codex EMPTY ordinary gates precede Git/process;
its private Git copies remain unqualified/unmigrated inventory, not a new ordinary
consumer. No shared primitive or safe bootstrap proof is supplied by that partial.

## 1. Private boundaries and actual consumers

Add `adapter/git_owner.rs` as a child module of adapter. Preserve the existing
crate-private async `bounded_git_raw` and scalar `bounded_git` signatures:

```rust
bounded_git_raw(
    executable: &Path, cwd: &Path, args: &[String],
    environment: Vec<(OsString, OsString)>,
    deadline: tokio::time::Instant, uncertainty: Arc<AtomicBool>,
) -> AdapterResult<Vec<u8>>
```

The wrapper owns the arguments before handing them to the independent owner;
borrowed caller data never survives through a dangling reference. `CallGuard`
is installed before admission starts. No Store/schema or native API is added.
Actual callers route through this wrapper: Generic validate_git; Grok ownership
initial/refresh/checkpoint and index_digest; Context git_value_owned/git_value;
scalar bounded_git. Existing cfg(test) macOS TestPlan follows the same production
owner path with a private pool. No copied test helper replaces those callers.

The sole ordinary consumer equation change is Generic's live launch Err at
adapter.rs630–646: Lost when ErrorKind==SessionLost OR the call's own uncertainty
is true; otherwise Failed. Preserve the original API ErrorKind and persistence
precedence. Reservation::drop remains conservative. Context's sticky latch and
Grok sampled receipt equations remain unchanged. Grok checkpoint has no durable
receipt/Session sink: Err plus retained pool only, stronger resume fencing OPEN.
Non-Git Generic/Grok Tokio spawn/new/Drop/readers, synchronous git.rs and future
provider copies are unchanged/unmigrated, including Tokio post-spawn wrapper Err.

## 2. Four counted jobs per operation; no hidden driver

`GitPool` is process-local, private and shared by all production callers. Exactly
64 work permits; admission reserves FOUR atomically before any OS Git spawn:

| Reserved slot | Actual execution / resources |
| --- | --- |
| supervisor | one std thread execution frame, owning one current-thread Tokio runtime and borrowing its inner supervisor future |
| stdout | at most one reader task created on that runtime |
| stderr | at most one reader task created on that runtime |
| native worker | one std thread, created before Git spawn, owning actual std Child and executing spawn/first KILL/inspection/reap |

A thread driving its one frame/future is ONE job, not an additional uncounted
worker. The current-thread runtime creates no Tokio worker thread. There is no
separate permanent pool driver, dispatcher, monitor, reaper, per-waiter task,
spawn_blocking bootstrap, replacement supervisor or hold-join job. Operation
setup runs in its reserved supervisor frame. Actual reader creation uses the two
reserved slots. Native worker creation and all its later commands use its ONE
reserved slot throughout. At most16 operations,32 readers,16 supervisor frames
and16 native workers; these are component work limits, not a universal count of
OS/framework/helper threads or a whole native-workload inventory. The trusted
selected-group inspector owns its own actual child/pipe resources as currently
implemented; these are observed on the already counted native worker, never
new supervisor/read/cleanup jobs of this component. Existing inspector bounds
and mandatory cleanup are unchanged.

Admission has no per-waiter task: check the caller deadline first, acquire the
pool lock briefly, register/await Notify without losing a release wake, and retry
until the existing deadline. No resources/effects or new flag on expiry. Permit
allocation and terminal release mutate counters under a short pool mutex; that
mutex is NEVER held across std spawn, signal, inspection, Child observation/reap,
thread join, runtime drive/drop or reader polling. Counters saturate and expose
only active and retained-unresolved occupancy plus finite admission reason.
At least four operations must genuinely progress concurrently with healthy driver
and free capacity; there is no fairness/Project isolation under permanent holds.

## 3. Ticket, flag publication and pre-effect startup

Proposed private data, not exported capabilities or durable workload authority:

```rust
struct OpTicket {
    publication: Mutex<Publication>, cancel: AtomicBool, owner_lost: AtomicBool,
    result_wake: Notify, cancel_wake: Notify, native_commands: BoundedSender,
}
enum Publication { Admitted, CancelledBeforeSpawn, Spawning, Live, FrozenUnknown, ReturnedSettled }
struct OpAssets { readers: ReaderPair, native: NativeLink, /* retained runtime/stdio */ }
struct NativeAssets {
    child: Option<std::process::Child>, group: GroupBinding, signal_issued: bool,
}
struct Outcome { primary: AdapterResult<Vec<u8>>, resource_settled: bool, facts: GitFacts }
```

`GitPool::admit(deadline)` yields a private `OpReservation`, never a Store lease.
`OpReservation::start(request,ticket)` starts the supervisor std thread with
fallible Builder::spawn. The actual JoinHandle is stored in the reserved pool
record BEFORE a one-shot Begin handshake permits the thread to initialize/start
Git. A start failure, or caller cancellation before Begin, proves no Git attempt.
A created supervisor does not perform native Git work while the caller is storing
its handle. The supervisor builds its private current-thread Tokio runtime and
registers its SIGCHLD listener, then creates/stores the actual native worker
JoinHandle before authorizing any Git spawn. The bounded command Sender is installed in outer NativeLink/ticket BEFORE
worker creation; a worker created during a failing setup still receives Cancel/
Lost, or sender-disconnect, without Git authorization. The worker waits for that
authorization; setup failure requests no-attempt stop and joins the worker on the supervisor thread before publishing the error.
A failure BEFORE any supervisor thread was created can explicitly release its
unstarted reservation after measured no-job/no-attempt proof; this is not a
releasing Drop/unwind path. Once a frame/thread exists, only its explicit healthy
terminal action can release after actual job/resource settlement.
No caller-runtime handle, signal registration or stdio registration is borrowed.
The pool retains its own strong operation record before effects, so caller Drop
cannot destroy the ticket, native link or supervisor assets.

A frame-owned liveness/publication guard is installed before Begin. Its Drop and
outer catch_unwind Err arm set owner_lost and publish loss without native work,
then result_wake.notify_one() stores a permit even before a waiter exists. Worker
result disconnect/panic takes the same path. The caller registers/enables its
result waiter before rechecking ready-or-owner_lost; cancel has a DIFFERENT Notify,
so neither direction consumes the other's notification.

Loss while still Admitted atomically changes it to CancelledBeforeSpawn, prevents
any later Spawning, returns pre-effect LaunchFailure and leaves the flag untouched;
all four slots remain retained fail-closed until the healthy terminal release,
which loss does not grant. Loss after Spawning returns SessionLost/frozen true,
feeding actual Context latch/Generic Lost/Grok unclean. Loss never replaces an
already established cleanup/reap SessionLost with a lower-priority error. A poisoned
publication mutex is inspected via into_inner solely for retention/notification:
freeze Unknown/true, return SessionLost, never clear/release or resume admission.
No normal result path may wait forever for a lost owner to publish ResultReady.
This does not add a total operation-time bound to legitimate pending native cleanup.
Startup and each fallible command keep assets in outer storage, including unsent
pipe endpoints. A poisoned pool denies new admission and preserves existing records;
no counters are reset. Loss handlers are inside this same reserved frame/job.

The publication mutex linearizes admitted→spawning against CallGuard::drop:
Drop wins Admitted→CancelledBeforeSpawn and prevents native spawn, without
setting the caller flag. Spawning wins sets caller uncertainty=true BEFORE native
spawn; later Drop freezes true and signals cancel nonblocking. Drop never sends OS signals,
waits, joins, takes native assets or invokes group Drop. It uses atomics, separate
cancel Notify and a nonblocking try_send to the already bounded native command
lane; it never waits on a mutex held by native work. CallGuard is owned INSIDE
the awaited bounded_git_raw future: on cancellation
its admitted/spawning transition resolves before outer Context GitObservation or
Generic Reservation reads the flag. Actual race controls must prove this ordering;
a copied guard control cannot substitute. Driver ResultReady cannot clear.
The live caller consumes Outcome under the same publication lock:
complete in-budget resource settlement (including settled errors) clears; proven
no Git spawn attempt stays false if never set or clears if set. Once FrozenUnknown
or returned Unknown, nothing late clears the caller flag/Context latch.

Phase budgets preserve current semantics: caller deadline bounds admission and
non-reaping exit observation; owning-Child reap keeps its existing250ms window;
output collection/abort/both joins use ONE existing250ms window. They do not make
one total operation deadline. A fully settled observe-Timeout can clear only if
cleanup/reap and both reader joins were actually observed within their settlement
windows on the still-live return. Reap timeout, output_open or unobserved jobs
publish Unknown; a later completion is never renamed an in-budget clear.

## 4. Actual std Child anchor and first group cleanup

The native worker calls `std::process::Command` with exactly current executable,
argv, CWD, env_clear/env pairs, stdin null, stdout/stderr piped and process_group(0).
It stores a returned actual Child in outer `NativeAssets` BEFORE any fallible pipe
wrapping, signal registration or group binding. `NativeAssets` is borrowed into
fallible/panic-contained inner work; unwinding does not drop the outer anchor.
No Tokio Child conversion, raw-PID adoption or outside owning-Child reaping.

Private `GroupBinding` derives the signal identity from that actual unreaped Child
with current valid-PID checks. Binding Err retains Child/endpoints and original
LaunchFailure/Unknown; no invented signal identity/action. Std spawn Err returning
no Child is opaque terminal Unknown: preserve ProcessFailure, true flag and all
four slots, finish/destroy the supervisor future with no fictional anchor/observer,
release or clear. Successful std spawn is allowed. No safe Err profile/probe/pin,
blanket Unsupported, CI-only recipe or POSIX/libc no-survivor claim is introduced.

The worker takes the pipe endpoints, retains ownership during transfer and sends
them to the supervisor. Their `tokio::process::ChildStdout/ChildStderr::from_std`
registration occurs on the private runtime; this wrapper may fail but the actual
Child remains in NativeAssets. Any unstarted endpoint is closed on its owner,
and has a measured no-task fact. A started reader always retains its JoinHandle.
Missing-pipe/init errors enter first cleanup and the same settlement flow.

Production group behavior is a Git-local equivalent of adapter.rs365–405, using
the existing trusted inspector/resolver and cfg(test) TestPlan, without changing
shared ProcessGroup. Exact unreaped group KILL: Ok/SRCH succeed on BOTH OSes;
macOS PERM succeeds only on valid-dead trusted inspection; every other errno is
Unknown, including Linux PERM. Group-owned clears ONLY on success. Existing
TestPlan forces Ok/SRCH/PERM through the actual PERM inspection route. Query stays
`/bin/ps -g OWNEDPGID -o pid=,pgid=,stat=` env_clear, NO COMMAND_MODE,250ms/1MiB.
No implicit group Drop retry or second signal. First cleanup performs native work
on the already reserved worker, never async poller/Store/pool lock.

## 5. Exit observation, reap, cancellation and worker lifetime

The healthy supervisor registers SIGCHLD during PRE-EFFECT runtime setup; its
failure is a no-attempt LaunchFailure. After spawn/binding/from_std it starts BOTH
readers before beginning existing non-reaping waitid(EXITED|NOWAIT|NOHANG) observation,
preserving full-pipe progress. The pre-created stream is used for that observer.
The identity is a private
non-adoptable token derived from the worker's pinned actual Child. The worker is
not permitted to reap while this observation is active. The supervisor ends/drops
that observer before sending the CleanupAndReap command. Only the native worker
calls actual Child::wait AFTER successful first group cleanup, on its ALREADY
counted reserved thread. The supervisor observes that one wait-result channel
within the existing250ms reap window. Timeout publishes the same SessionLost,
retains the still-running actual wait/Child/slot and proceeds to the one output
settlement window; the SAME supervisor can late-await the same result afterwards.
Cancellation still publishes its atomic fact and nonblocking bounded message
while that wait is blocked; it does not interrupt/terminate the actual wait or
release its Child/worker/runtime. A control must assert caller cancellation and
cutoff can complete while the actual owned wait remains pending. Use an owned
scoped child on an explicitly owned alternate fixture group with path-driven
exit to hold a genuine wait pending, never a foreign/unknown PID signal or outside
Child reap. This fixture demonstrates selected-group cleanup is weaker than
direct-child settlement, not stronger containment.
No new wait/signal/command/job or deadline reset. A returned wait Err is terminal
Unknown/lost wait authority; never re-wait/PID rescue. There is no try_wait polling
loop, late periodic reap polling, blocking wait or outside waitpid on an async
poller. Mandatory/native wait is not a hard total operation-time bound.

Every worker message/endpoint transfer is bounded to its one operation. Use
finite one-shot stage channels/nonblocking sends; a full/disconnected channel
retains the unsent actual endpoints/outcome on the owner rather than blocking
first-cleanup delivery. There is no unbounded queue of commands/results.
The worker parks on a capacity-bounded command channel (capacity4) rather than
5ms polling. Its finite message set is Begin, CleanupAndReap, Cancel and Lost;
no recurring retries/monitor messages. Begin is consumed before later commands.
CallGuard Drop sends Cancel once, owner_lost CAS coalesces Lost once even when
both catch_unwind and guard Drop run, and the
healthy supervisor sends CleanupAndReap once. Nonblocking try_send plus the
atomic fact ensures full/disconnected delivery cannot erase cancellation/loss.
A full channel already contains at most the finite earlier commands, which must
be consumed with cancel/lost state rechecked before any effect; disconnect is
owner loss. Neither caller nor supervisor blocks sending. Worker handles its
atomic states after every received command. There is no parked polling cadence.
If cancellation/failed supervisor wins before a healthy cleanup command, it
performs FIRST cleanup once and caches that result; a later healthy command uses
the cached result without signalling again. It leaves Child unreaped until the
healthy supervisor stopped the observer. On lost supervisor, first cleanup alone
cannot grant completion/reap/reader success; retain remaining actual anchors,
finish the worker frame and keep all4 slots. No replacement observer. Caller
cancellation alone does NOT lose the healthy supervisor, which continues existing
resource observation late.
Caller runtime shutdown cannot stop either independent thread/runtime. Total owner
loss/panic/poison remains Unknown; no restart/PID rescue/replacement worker.

The worker borrows `NativeAssets` into a catch_unwind inner function. Panic retains
actual Child/group/endpoints in outer storage. Set signal_issued=true in outer
NativeAssets IMMEDIATELY BEFORE KILL/TestPlan.signal, not after its result. Outer
panic handling may run first cleanup ONLY if this bit is false. If true without a
cached result, retain terminal Unknown with no second signal; it is not proof the
signal succeeded or the group died. Returned cleanup/reap Err is terminal
retention; no automatic retry. It sends bounded measured outcome, then returns.
The healthy supervisor frame joins this actual native worker BEFORE publishing
ANY resource-settled/success Outcome as well as before slot release. A result
channel alone is NOT worker completion; failed join publishes lost/Unknown.
Worker launch/poison/panic/exit facts never substitute for observed group/Child.

## 6. Both reader joins and frozen error precedence

ReaderPair retains each handle in OUTER OpAssets; the inner supervisor borrows
handles for await. Catching inner unwind cannot drop/detach them. The owning runtime
and outer assets survive failure by moving into bounded retained storage. A healthy
supervisor collects stdout then stderr; read_git_output retains OUTPUT_LIMIT+1
capture/InvalidInput behavior and the original read/join error kinds. Stderr failure
observed by the runtime does not reorder a still-pending stdout's primary outcome.
After the first primary collected read/join error, abort a pending peer and observe
BOTH joins within the SAME output deadline. Early cleanup/reap/exit-observe errors
abort both and use that one window. Expiry requests abort, freezes not_observed
at the published outcome and keeps actual handles; it never creates another budget.

Join fact states: not_started (no task plus endpoint closed/never registered),
joined_returned (read result separately ok/IO/budget), joined_panic,
joined_cancelled, not_observed; independent abort_requested. A cancelled/panicked
JoinResult proves future/endpoint destruction only. No read/native-death inference.

Primary precedence remains cleanup SessionLost, reap SessionLost, observe deadline
Timeout, observation SessionLost, stdout read/join, stderr read/join, output_open
ProcessFailure, unsuccessful Git exit OwnershipMismatch, then success. Post-spawn
init error first attempts cleanup/reap; their failure wins, otherwise original init
kind survives. Binding failure lacks signal authority and stays original
LaunchFailure/Unknown. Reader settlement facts are separate from the primary error.

A healthy same supervisor continues observing pending already-created Child/readers
after publishing caller Unknown, releasing capacity only on actual complete resource
settlement. It may await the already-running owning-Child wait result (never after a returned
Err) or the actual reader JoinHandles without resetting the published deadline.
No new job/signal/command/replacement observer. Late success never clears the flag.
Opaque no-Child spawn Err, terminal group/reap Err or lost supervisor/driver cannot
late-release. Readers on a retained but no-longer-driven runtime remain unobserved.

## 7. Terminal execution frame and retention storage

Outer supervisor frame owns runtime, assets, ticket and permit bundle; its std thread
runs the inner future by mutable borrow inside catch_unwind. On healthy completion
it observes/destroys that future, joins the actual completed native worker, verifies
no resource/job is pending, and drops reader/runtime/native settled resources BEFORE
terminal permit release. The permit bundle has NO releasing Drop/RAII path: unwind/Drop retains all4.
ONLY the explicit healthy terminal frame action releases. It first publishes a
settled Outcome after the actual worker join/resource destruction. Its final
action only updates pool/counters/Notify. It validates unpoisoned pool/ticket
bookkeeping before settled publication and release; poison instead publishes loss
and retains slots. No native wait/cleanup/runtime destruction/blocking join follows release. The thread's
nonblocking return epilogue is not a new work job; no OS-thread-join certificate is
claimed for the supervisor. Its actual thread handle is accounted/stored during the
job and may be dropped only at this terminal frame action, after work is complete.
This explicit inner-future/frame boundary is the clause13 design choice; no extra
reaper/monitor job is introduced to observe its own execution.

On terminal Unknown, move remaining actual assets/runtime into a private bounded
retained record BEFORE the frame completes. All four permits remain held even if
some resource did not start or its job already ended. Opaque no-Child Err stores
measured absence plus the terminal uncertainty, not a fictional Child. Pool storage
is at most16 operation records; ownership movement under its mutex never performs
native cleanup or Runtime drop. Poison uses its existing storage/anchors fail-closed,
no counter reset/release. Fatal process termination is outside process-local recovery.

## 8. Private controls, causal mutants and gates

cfg(test) `GitTestContext { pool: Arc<GitPool>, plan: Option<ProcessInspectionPlan>,
hooks: TestHooks }` is an owned explicit carrier, never a process-global override
or caller-task-local assumption. Optional plan exists only on macOS; pool/hooks
work on both OS. Production resolves ONLY its static global pool and preserves
both public crate signatures. cfg(test)-only `_with_context` wrappers/scalar/raw
inner trailing carrier dispatch the SAME owner, not copied helper behavior.

| Actual carrier route | Control / disposition |
| --- | --- |
| GenericCliAdapter cfg(test) field → validate_git → scalar/raw with_context | live returned Unknown Lost/original kind vs settled Failed; before/after-spawn caller Drop |
| GrokAdapter cfg(test) field → OwnedEntry → Actor → verify_git and index_digest | initial/refresh/index sampled receipt uncertainty, including separately tokio::spawn'ed supervise |
| same GrokAdapter carrier → checkpoint → verify_git | checkpoint Err/private holds only, no fabricated receipt or durable fence |
| git_value_owned explicit cfg(test) carrier parameter → raw with_context | existing Unknown TestPlan, new nonexistent-exec and Drop/latch controls |
| bounded_git_raw_with_plan builds a private context for its actual raw inner route | current7+1 plan calls remain private and use the same plan/production logic |
| new direct bounded_git/scalar cfg(test) with_context | saturation/late joins/panic/worker settlement controls |

Production cannot create a private pool or bypass global64. Existing7 adapter+1 Context Unknown controls
and EVERY new destructive/Unknown control (Context/direct Git, Generic/Grok included)
use isolated pools and assert production retained occupancy unchanged. Private-pool
teardown settles actual anchors on its reserved non-poller lane where existing authority
permits, or deliberately retains them; no numeric fixture rescue, async blocking group
Drop, production counter pollution or test serialization. Settled runtimes drop
on their own supervisor frame BEFORE release. Deliberately retained records move
into a cfg(test) process-lifetime holder of the ACTUAL private pool/runtime/handles;
async test teardown never drops a retained Runtime or anchors. The holder is a
finite registered test inventory, not an unbounded worker/queue or production
release port. Each destructive control registers only its admitted bounded pool;
actual teardown assertions prove no async Runtime drop and unchanged production
retained counters. Retention is disclosed, not cleanup success.

Controls and independently compiled mutants must exercise:64-slot saturation/no-spawn
and expired-deadline precedence; four real concurrent Git operations; caller cancel
before/after spawn and shutdown-before-cleanup; private-own-runtime reader survival;
actual std-spawned Child then forced initializer Err with fully-settled/Unknown pairs;
missing exec opaque Err through Generic and Context; no-attempt setup refusal;
invalid-binding retained actual Child; direct native worker panic before first cleanup;
inner-supervisor unwind/outer-frame loss/poison with live caller typed return
and outer reader/runtime anchors; concurrent result/cancel waiter race; cancellation
ordering at Context latch/Generic guard; cleanup/returned reap error;
pending peer after primary error; abort requested vs observed cancelled joins;
late resource release without flag/latch clear; worker result vs actual worker join;
frame resource destruction before release; panic after attempted KILL/no retry; each production cleanup signal outcome/
TestPlan differential both OS; Generic Lost/original-kind vs settled Failed; Grok
initial/refresh/index receipt input and checkpoint Err with pool-only retention.
Masked/redundant guards, pure units and source-only controls receive only that credit.
Default internal test concurrency, deadlines, inspector and latch rules remain intact.

## 9. Impact analysis and parity

All references below are original8f95adb; incoming main478 adds11 adapter lines
and7 Grok lines without changing these bodies. Source implementation must update
citations and inspect exact input/source equality, not blindly copy offsets.

| Changed symbol/site | Actual consumers | Impact / required disposition |
| --- | --- | --- |
| bounded_git_raw_inner adapter1317–1421 → git_owner | Generic validate_git1148–1163, Grok ownership425–450/index1467–1483, Context588–622, scalar1250 | replace Git-only custody; same command/env/argv and primary kinds/text/fact forwarding, explicit Lost exception below |
| global64/4-per-op/max16/32 and admission/frame | all above | new bounded process-local pressure and conservative terminal holds; actual cap/four-progress/late-release controls; fairness50 OPEN |
| CallGuard/owner_lost/separate wakes | Generic Reservation457–468/live630–646, ContextGitObservation660–664, Grok flags/receipt | real cancellation/loss controls, no late clear, no hang, pre-effect vs post-effect precedence |
| native std Child binding/signal/reap | current ProcessGroup365–405/438–441 and cleanup1434–1455 serve differential baseline only | Git-local equivalent, no shared native new/reap/Drop behavior edit; full signal/TestPlan/errorfact parity |
| reader handles/budget64KiB+1/single250ms | read_git_output1422–1432, Generic/Grok/Context return paths | actual both joins, primary stdout ordering, observed cancellation and pending peer; cap/deadlines unchanged |
| cfg(test) GitTestContext/private holder | existing adapter1586–1631/context1422–1480 plus new real routes | actual isolation + no async retained Runtime drop/production pollution; never global or tasklocal override |
| Generic live launch Err630–646 | start/Store reservation/launch audit | ONLY equation exception Lost for SessionLost OR own Git flag; original ErrorKind/persistence/error priority preserved |
| native Generic/Grok/common ProcessGroup and Grok registry tests1508–1514 | all existing native tests/current Codex EMPTY private tests | no behavior edits; direct reused new/read_git_output/cleanup remain unchanged; compile/default suites verify composition |
| Codex private ownership/preparation Git | ordinary EMPTY refusal before Git; private paired producer/backend fixtures | unchanged/unmigrated and unqualified; no new readiness/lifetime proof |

Retain EXACT existing branch strings: `native process group cleanup failed: {e}`
(including unchanged inspection_facts transport); `Git child death not confirmed
after cleanup`; `Git child reap failed: {e}`; `Git ownership preflight timed out`;
`Git child observation failed: {e}`; `Git output remained open after cleanup`;
`Git metadata exceeds output budget`; `Git ownership preflight failed`. Actual
read/join original details and kinds remain. Differential assertions enforce these
existing branch messages/fact transport, not merely substring-based correctness.
NEW capacity/setup/owner-loss paths get static value-free stage text and typed
precedence: no-attempt setup/Admitted loss LaunchFailure; after-Spawning or poisoned
publication loss SessionLost/frozen Unknown. These new routes are not falsely
claimed byte-identical to previously nonexistent branches. Reap/cleanup known
SessionLost keeps its established primary; loss cannot reorder stdout/stderr.
No 5ms component worker/reap polling is added; the trusted inspector's existing
5ms cadence/250ms and all existing deadlines remain unchanged.

Two independent immutable native Design2 delta reviews must approve before Source.
Source gates include two independent native reviews with verified fixes/rereviews,
meaningful actual controls/mutants, appropriate default debug/release tests, fmt/
all-target Clippy/builds, both OS required CI with actual checkout/parents/tree proof.
Every old red/release failure remains FAILED, cause AND regression UNKNOWN.
This design certifies no new availability backend, complete native workload, escaped
helper/service, effect/delegation authority, durable hold/replay/source freshness,
restart recovery, default-deny enabled profile or native3/runtime16 MVP acceptance.
Whole60/F1/native16 remain OPEN even if this bounded process-local component passes.
