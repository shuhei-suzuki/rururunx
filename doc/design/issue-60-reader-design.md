# Issue60 selected Git owner/readers — Design1 candidate

Risk: STRICT. Requirements4 at a8e8a8b has two independent APPROVE/no C/H/M;
its four verified Low refinements appear in this design and the requirements
outcome ledger. This Design1 is UNAPPROVED. No reader source, test, availability
or F1 acceptance exists. Existing shared native ProcessGroup/new/reap/Drop stays
unchanged. Current source references are the public a8e8a8b composition of main
efe9774, not a future provider branch.

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
struct OpTicket { publication: Mutex<Publication>, cancel: AtomicBool, wake: Notify }
enum Publication { Admitted, Spawning, Live, FrozenUnknown, ReturnedSettled }
struct OpAssets { readers: ReaderPair, native: NativeLink, /* retained runtime/stdio */ }
struct NativeAssets { child: Option<std::process::Child>, group: GroupBinding }
struct Outcome { primary: AdapterResult<Vec<u8>>, resource_settled: bool, facts: GitFacts }
```

`GitPool::admit(deadline)` yields a private `OpReservation`, never a Store lease.
`OpReservation::start(request,ticket)` starts the supervisor std thread with
fallible Builder::spawn. The actual JoinHandle is stored in the reserved pool
record BEFORE a one-shot Begin handshake permits the thread to initialize/start
Git. A start failure, or caller cancellation before Begin, proves no Git attempt.
A created supervisor does not perform native Git work while the caller is storing
its handle. The supervisor builds its private current-thread Tokio runtime and
creates/stores the actual native worker JoinHandle before authorizing any Git
spawn. The worker waits for that authorization; setup failure requests no-attempt
stop and joins the worker on the supervisor thread before publishing the error.
No caller-runtime handle, signal registration or stdio registration is borrowed.
The pool retains its own strong operation record before effects, so caller Drop
cannot destroy the ticket, native link or supervisor assets.

A supervisor-liveness guard marks lost before its execution frame unwinds; the
native worker checks that fact on its already-running lane. Startup and each
fallible command keep assets in outer storage, including unsent pipe endpoints.

The publication mutex linearizes admitted→spawning against CallGuard::drop:
Drop wins Admitted→FrozenUnknown/cancelled and prevents native spawn, without
setting the caller flag. Spawning wins sets caller uncertainty=true BEFORE native
spawn; later Drop freezes true and signals cancel nonblocking. Drop never signals,
waits, joins, takes native assets or invokes group Drop. Cancellation uses atomics/
Notify only, and never waits on a mutex held by native work. Driver ResultReady
cannot clear. The live caller consumes Outcome under the same publication lock:
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

The healthy supervisor registers SIGCHLD on its private runtime and uses the
existing non-reaping waitid(EXITED|NOWAIT|NOHANG) observer. The identity is a private
non-adoptable token derived from the worker's pinned actual Child. The worker is
not permitted to reap while this observation is active. The supervisor ends/drops
that observer before sending the CleanupAndReap command. Only the native worker
calls actual Child::try_wait, after successful first group cleanup; repeated pending
try_wait observation is not command/signal retry. Its existing250ms reap budget
is measured on that worker; terminal Err loses wait authority and retains Child.
No blocking Child::wait or outside waitpid on an async poller.

Every worker message/endpoint transfer is bounded to its one operation. Use
finite one-shot stage channels/nonblocking sends; a full/disconnected channel
retains the unsent actual endpoints/outcome on the owner rather than blocking
first-cleanup delivery. There is no unbounded queue of commands/results. When parked awaiting its supervisor, the
native worker checks nonblocking cancellation/failed-supervisor facts at a finite
5ms cadence. If cancellation/failed supervisor wins before a healthy cleanup
command, it performs FIRST cleanup once and caches that result; a later healthy
command uses the cached result without signalling again. It leaves the Child
unreaped until the healthy supervisor has stopped the observer. On lost supervisor,
first cleanup alone does not grant completion/reap/reader success; transfer all
remaining anchors to retained storage, finish the worker frame and keep all4 slots.
No replacement observer. A cancelled caller alone does NOT lose the supervisor:
that same supervisor continues existing cleanup/Child/readers observation late.
Caller runtime shutdown cannot stop either independent thread/runtime. Total owner
loss/panic/poison remains Unknown; no restart/PID rescue/replacement worker.

The worker borrows `NativeAssets` into a catch_unwind inner function. Panic retains
actual Child/group/endpoints in outer storage; a valid first cleanup remains the
reserved lane's responsibility if not run. Returned cleanup/reap Err is terminal
retention; no automatic retry. It sends bounded measured outcome, then returns.
The healthy supervisor frame joins this actual native worker before terminal
resource-settled slot release. A result channel alone is NOT worker completion.
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
settlement. It may continue a pending try_wait observation (never after a returned
Err) or await the actual reader JoinHandles without resetting the published deadline.
No new job/signal/command/replacement observer. Late success never clears the flag.
Opaque no-Child spawn Err, terminal group/reap Err or lost supervisor/driver cannot
late-release. Readers on a retained but no-longer-driven runtime remain unobserved.

## 7. Terminal execution frame and retention storage

Outer supervisor frame owns runtime, assets, ticket and permit bundle; its std thread
runs the inner future by mutable borrow inside catch_unwind. On healthy completion
it observes/destroys that future, joins the actual completed native worker, verifies
no resource/job is pending, and drops reader/runtime/native settled resources BEFORE
terminal permit release. The final frame action only updates pool/counters/Notify;
no native wait/cleanup/runtime destruction/blocking join follows release. The thread's
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

cfg(test)-only pool injection follows all ACTUAL consumer routes; production cannot
create a private pool or bypass global64. Existing7 adapter+1 Context Unknown controls
and EVERY new destructive/Unknown control (Context/direct Git, Generic/Grok included)
use isolated pools and assert production retained occupancy unchanged. Private-pool
teardown settles actual anchors on its reserved non-poller lane where existing authority
permits, or deliberately retains them; no numeric fixture rescue, async blocking group
Drop, production counter pollution or test serialization.

Controls and independently compiled mutants must exercise:64-slot saturation/no-spawn
and expired-deadline precedence; four real concurrent Git operations; caller cancel
before/after spawn and shutdown-before-cleanup; private-own-runtime reader survival;
actual std-spawned Child then forced initializer Err with fully-settled/Unknown pairs;
missing exec opaque Err through Generic and Context; no-attempt setup refusal;
invalid-binding retained actual Child; direct native worker panic before first cleanup;
inner-supervisor unwind with outer reader/runtime anchors; cleanup/returned reap error;
pending peer after primary error; abort requested vs observed cancelled joins;
late resource release without flag/latch clear; worker result vs actual worker join;
frame resource destruction before release; each production cleanup signal outcome/
TestPlan differential both OS; Generic Lost/original-kind vs settled Failed; Grok
initial/refresh/index receipt input and checkpoint Err with pool-only retention.
Masked/redundant guards, pure units and source-only controls receive only that credit.
Default internal test concurrency, deadlines, inspector and latch rules remain intact.

Two fresh independent immutable native Design reviews must approve before Source.
Source gates include two independent native reviews with verified fixes/rereviews,
meaningful actual controls/mutants, appropriate default debug/release tests, fmt/
all-target Clippy/builds, both OS required CI with actual checkout/parents/tree proof.
Every old red/release failure remains FAILED, cause AND regression UNKNOWN.
This design certifies no new availability backend, complete native workload, escaped
helper/service, effect/delegation authority, durable hold/replay/source freshness,
restart recovery, default-deny enabled profile or native3/runtime16 MVP acceptance.
Whole60/F1/native16 remain OPEN even if this bounded process-local component passes.
