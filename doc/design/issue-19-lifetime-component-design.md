# Issue19 retained preparation lifetime component

Status: selected MECHANICAL Design5 approved by TWO independent native reviewers;
selected source follows, not whole Issue19 authority/native readiness.
Parent Design37 at75cdae974da1ce9afa0a6de4e2cafc5065e60748 was rejected by both native
reviews. Its broad open findings are retained in the adjacent ledger. Absence of a
lifetime High/Medium in those rejected reviews is NOT this component's approval.
Selected Designs1/2 were rejected. Design3 obtained one approval and one local
registry-disposition Medium. Design4 fixed that disposition, obtained one approval
and one local uncreated-worker accounting Medium. Design5 fixes ONLY that
reservation disposition and verified local precision; TWO explicit
selected approvals precede code. Parent OPEN findings remain outside this gate.
It implements existing retained-custody/cancellation requirements; no new WHAT,
public native capability, migration, managed trait or receipt facade is introduced.

## Actual consumer and scope

Reuse the actual private Codex Control/Preparation paths imported from main478:

- `attempt.rs`: Control::spawn retains its Tokio task; CallerGuard::drop requests
  cancellation; TaskGuard::drop records conservative Lost before release_task.
- `session.rs`: spawn_launch and checkpoint dispatch their preparation through that
  same Control, then return independently of the owned supervision task.
- `preparation.rs`: bounded_git keeps Child/reader handles inside its future today.
  It is an UNMIGRATED effect consumer, not a full-resource producer. This component
  does not replace60's pending Git custody, helper API or prove native descendants.
- `availability.rs`: production EMPTY refuses before effects. No ready constructor
  is added. Existing selected-group/F4 fixtures remain mechanics only.

The selected source slice adds OUTER retained job custody and typed actor request
handling to Control, rather than a second operation supervisor. Existing ordinary
availability/refusal, native policy, preparation first-cause and own-Session CAS
stay authoritative. No managed marker/allocation/original authority/settlement is
implemented. Mechanical Control/job identity is not a substitute for the future
actual OriginalMarkerFrame/OwnedNativeOperation. Any semantic native request remains
unavailable until that real producer/private port composes.

The actual controlled resource consumer uses the SAME Control custody/actor/drop
implementation through a fixture-private CLOSED file/worker resource factory. It can create
actual explicitly inventoried jobs/resources in an isolated fixture and retain them
when the waiting caller/future ends. It cannot return a production ready backend,
managed admission or cleanup certificate. It exercises actual file ownership and worker threads; it
executes NO native/Git/helper subprocess and earns NO process-descendant proof. No public bool, Session/SQL/JSON or fake
receipt enables the factory. Scope the proof to observed custody, callback ordering
and cancellation; no all-native/cohort/OS sandbox claim.

## Retained ownership and creation order

Control owns the inventory outside the adapter/preparation future. Each admitted job
has a private generated identity/generation, exact owning Control, closed resource
kind, actual worker/join handle and monotonic cancellation/observed state. The actual
custodian retains a strong owner until every known job is joined; unknown job/resource
custody stays retained. A dropping caller only requests cancellation, never takes or
joins resources. Existing TaskGuard may report factual Lost; it cannot clear the
inventory, infer cleanup or produce managed closure. Late known local joins never
clear a published Lost/Unknown or manufacture native completion. The resource
custodian is a job of the SAME Control, not another operation authority; it
retains actual handles beyond caller/runtime cancellation. No native-side IO
borrowed from the caller runtime is qualified by this file-only component.

Reserve the complete declared finite job set BEFORE creation. Count the Control
execution and every component custodian/worker/reader frame once; no uncounted
per-waiter/bootstrap/reaper/monitor task. The selected component's maximum64 active
or held jobs PER shared runtime pool is a work limit, not a count of arbitrary OS/helpers/native tools. A
closed file fixture declares exactly three component frames: existing Control
Tokio actor, one retained custodian and one resource worker. The custodian creates
its own current-thread runtime only if needed to observe the actor JoinHandle,
with no extra framework worker/thread. It owns/joins the resource worker and drains
the actor inbox while observing that actor. One finite resource holder owns the
actual file: the worker opens and owns it ONLY after its installed Begin;
there is no custodian-stack File waiting for later transfer. Worker completion plus actual joins/close
is the mechanical proof. No additional bootstrap, reader, reaper or waiter is
created. Ordinary unavailable native resources remain unavailable; no CLOSED native
profile constructor exists. A missing complete declaration refuses before creation. The real native profile/6F1/60 job sets are unavailable,
not inferred from the fixture. Reserve atomically; insufficient capacity starts
zero jobs/effects. Release only genuinely joined jobs OR the explicitly observed creator-owned
NotCreated disposition below; uncertainty retains its own reservation. Explicit pool counters are NOT Drop-released RAII permits. Each
fixture injects an isolated private pool through the SAME constructor/reservation/
consumer code; production has one process-wide pool. Unknown fixtures cannot
consume another test's capacity or require serial Cargo execution. Queue memory/resource reservation has its own finite bound below.

Actual creation goes INTO the supervisor's outer anchor before the adapter can
await, fail or panic at an attach step. BOTH created custodian and resource worker wait for separate Begin handshakes
until their actual handles/heap inventory are installed. Begin disconnection exits
without resource effect and remains joinable, never means proceed. Effect-capable factory
creation runs on that registered custodian; resource handles never live solely in
an adapter local waiting for later transfer. The registered worker owns the file until actual close/return; its JoinHandle and
the actual custodian handle live in retained Control/pool heap inventory, NOT an
adapter local. A worker panic closes its local File through unwind but still yields
Unknown, not a normal cleanup proof. Detached-before-install loss retains Unknown
for the entire declared holder/job set, not only the worker handle. Custodian callback
handlers and adapter-future polling have separate catch-unwind boundaries. A
callback panic records uncertainty while preserving the actual outer holder;
whole custodian loss retains handles/counters in the pool for attention. A pure
error, post-creation panic, caller Drop or runtime shutdown cannot erase it. Unclear
spawn/observer results retain ownership; Err is not no-effect authority. A proven
closed fixture resource can be cleaned/joined on its real custodian, with actual
observations, without claiming managed NoCurrentDispatch or settlement.

## Actor access after return and finite queue

A registered actor receives one non-Clone/nonserializable private request endpoint
bound to the SAME Control/job/generation. It only submits typed messages; Control's
retained handler owns resources and executes the actual mechanical callback. A
returned Starting/Err or a reconstructed Session does not mint another endpoint.
Mechanical callback success is not model admission, ALLOW, HostCallbackCurrent,
Session publication or settlement. Those semantic requests require their future
actual original-frame/native producer and remain unavailable here.

Use a bounded64-request inbox. Each canonical JSON metadata encoding is at most4096
actual UTF8 bytes, checked by a bounded streaming JSON writer before enqueue, so queued ENCODED metadata≤256KiB (not physical RSS/allocator-capacity proof); referenced immutable payload/resource
bytes retain their existing separately charged bounds, never an unbounded hidden
message. No serialized closure/operation credential or copied prepared frame enters
the inbox. Job registration/actual resource ownership precedes the first request.
Generation/control mismatch, revoked endpoint, over-bound metadata or queue capacity
refuses before the requested effect. Cancellation revokes new effects; already-owned
jobs still report cleanup/outcome facts with their exact identity.

Control keeps draining the inbox while awaiting jobs and deriving observed completion;
it NEVER awaits a join while abandoning a sender blocked on that inbox. A std-worker request uses bounded try-enqueue ONLY; full means typed refusal, not
a blocked sender. Async actor sends are cancellation-aware and remain bounded by
the same queue; no blocking Drop or hidden per-send task. Held resources do not cause healthy actors to retain unrelated partial
job reservations. Endpoint EOF is ONLY an endpoint-closed fact, never an immediate Unknown. The
endpoint may close before its frame returns or its JoinHandle is observed. Ordinary
completion requires every bound job genuinely joined with completed outcome, the
inbox drained and no in-flight callback. Outstanding means unjoined jobs PLUS
accepted-unhandled requests PLUS in-flight callbacks. Panic/cancelled/unobservable
join or genuine runtime/custodian loss while outstanding yields Unknown. A worker
that closes its endpoint then sleeps and exits normally MUST remain ordinary
completion, not absorbing Unknown. Runtime/custodian loss with outstanding resources retains
Unknown attention and permits; no row/terminal label can recreate it.


## Total installation, join and registry protocol

The custodian is the SOLE inbox Receiver owner and keeps it outside adapter unwind.
It drains requests while waiting on a bounded observation tick (no spawn_blocking).
It calls std join ONLY after is_finished/completion is observed, outside pool/Store
locks; no universal OS latency bound is claimed. Accepted-but-unhandled messages
remain charged/outstanding after actor exit. Fixed typed handlers, not arbitrary
serialized callbacks, run under unwind capture; unrelated native Store/ALLOW/host
requests still have NO producer here.

For a custodied Control: obtain Handle::try_current BEFORE reservation, custodian
creation or capturing Registered/TaskGuard in the actor future. Missing runtime
refuses with zero created frames/effects via the existing no-work path. Then reserve
declared three frames; create/install custodian
handle under Begin; mark TaskOwner::Installing; spawn the actor WRAPPED to wait for
its own Begin; store the actor JoinHandle into that same heap inventory and its
AbortHandle/identity in a new Observed state; then send Begin. The actor cannot
create resources before installation. NO pool, inventory, Control.task or registry
lock is held across runtime.spawn, Begin send/disconnection, actor future Drop or
other synchronously dropping operations. Separate post-spawn acquisition installs
the handle even if the closed runtime already dropped the actor. No lock is held
across create/await/join/resource destruction. TaskGuard/release_task marks its logical
Released state but NEVER drops the custodian-owned actor JoinHandle. Closed-runtime custody uses actual joins or conservative held attention;
no-custody legacy paths preserve their existing Running/Released tests.
The observed branch is total over Installing/Observed/Released, including completion
before installation and panic. No second attempt steals a handle.

The pool retains the custodian JoinHandle after its body returns. Admission or
explicit existing-frame status/drain housekeeping first collects is_finished handles,
then joins them OUTSIDE pool/Store locks, and only then decrements those slots.
No extra reaper task/thread or self-release. Healthy sequential fixtures beyond64/3
prove replenishment; detached/no-reap mutant exhausts capacity. The fixture owns its
pool until safe cleanup/reap completes. Lost/unjoinable handles stay held, not refunded
by pool/caller/RAII Drop. This proves local joined resources only; it never clears
native Lost or managed ownership. Unexpected custodian loss before Begin retains
counters, workers receive disconnected Begin and perform zero resource effects.

A DISTINCT monotonic Control job-revocation latch prevents every NEW endpoint effect,
independently of Preparation's first-cause/Consumed/CheckpointCommitted/Failing states.
Explicit stop/custody cancellation revokes synchronously before publishing the stop
request. A Held entry returns typed held/no-native-stop refusal after revocation;
its closed native stop channel never skips the latch or fabricates Lost. CallerGuard::drop ALWAYS sets job revocation while armed, including Consumed and
Failing, but preserves existing consumed-turn no-interrupt behavior: no new native
stop message and no Preparation::Consumed reset. Normal return disarms CallerGuard,
so the disarmed Err-return fixture endpoint remains usable until explicit revocation.
The Ok/Starting post-return endpoint remains UNEXERCISED until a real producer composes. Authoritative local cleanup facts are returned by their OWN JoinHandle outcome,
not this saturable request inbox. Already-owned jobs retain that outcome until the
custodian observes the join; optional progress refusal cannot lose cleanup facts. The handler
checks the latch/generation immediately before effect; no recreated endpoint or queue
message can clear it. No native semantic grant implements this mechanical latch.

Actual reused session consumers MUST check outstanding effect/resources/accepted
requests: refused_before_work, prepare_launch/checkpoint context-error outcomes,
RegisteredTransition::drop restore/remove and register_existing. This Codex
registry has no terminal eviction implementation; saturation refuses new entries
and public release remains unsupported.
No RestoredBeforeAdmission/FreshUnpublished/CheckpointCommitted no-work classification
or old-control restoration/removal is allowed while such custody remains. Retain the
exact same registry Control/exclusion and explicit held outcome; registration refuses.
Control execution/custodian bookkeeping alone is not a native effect or death proof.
No source restoration permission is inferred from a slot counter/terminal label.
A genuine later local join releases THAT component job slot. The exact closed-file
fixture may additionally reconcile its original registry disposition below; it
cannot clear published Lost or supply native resume authority. Existing already-no-effect paths with zero
created resources/requests retain their previous behavior. These guards introduce no
managed receipt, current-frame refresh or native recovery exception.


## Single completion owner and concrete fixture consumer

Phase remains the provider actor's single-final logical result. Custody has a
SEPARATE private fact: Installing/Tracking/Joined/Unknown, plus registered jobs,
accepted requests and in-flight callbacks. The custodian records genuine actor/worker join and drained-request facts.
Only pool housekeeping, after also joining the custodian itself outside locks,
derives final Joined over all actually created declared frames (and the observed
NotCreated disposition for a never-created worker); actual custodian loss derives
Unknown, never provider terminal success. It NEVER overwrites Phase::Finished or published Lost.
When an actor would select a no-work/restored result while effect resources remain,
it selects one private Outcome::CustodyHeld with cause and retained non-authoritative
snapshot, whose snapshot()/no-work selectors return NONE. Phase is Finished with
that held result; TaskGuard does not fabricate Lost merely for a KNOWN joinable worker.
wait_finished returns this exact held result. Normal worker/actor joins followed by the actual pool-observed custodian join make
the separate custody fact Joined, leave Phase unchanged and release only genuinely
joined component slots. Worker panic/unobservable completion makes custody Unknown;
TaskGuard's independent genuine actor panic/drop continues its existing Lost behavior.
No held Phase authorizes native resume/recovery/success. For Finished(CustodyHeld) or outstanding CREATED effects/requests, while custody
is Installing, Tracking or Unknown, register_existing rejects it and
RegisteredTransition preserves the same Control/exclusion. Zero-effect bookkeeping-only custody retains the existing
no-work registry remove/restore behavior even while actor/custodian joins are pending.
Closed-runtime unpublished Lost therefore removes its fresh registry entry, while
its one uncertain actor slot remains held. An immutable Held
Phase is never overwritten by a later join.

For this CLOSED fixture consumer ONLY, the existing context-error arm also derives
one original delayed registry disposition: FreshUnpublished remove, or exact
previous_control restoration. It binds that disposition to the exact new Arc<Control>
and SessionId before RegisteredTransition leaves; previous_control is genuine
original in-memory identity, not a reconstructed Session. The arm is before
Starting publication, Git or native effects and cannot record this disposition on
any other outcome. Store/body publication is unchanged. Entry owns this retained
intent on this normal context-error path. Genuine actor abandonment publishes Lost
without a restoration intent and remains recovery-pending. Before taking the
registry lock, register_fresh and register_existing perform existing-frame pool housekeeping to observe finished
custodian joins. Under the registry lock they then call ONE reconciliation predicate.
register_fresh sweeps ALL retained intents (bounded by32 entries) BEFORE its len()
capacity check; register_existing applies it at least to its target. It requires the same current Control,
its exact Finished(CustodyHeld) outcome/disposition, genuine custody::Joined (ALL three
declared frames actually joined, inbox drained, in-flight zero), and no published
Lost. It then removes the fresh entry or restores exact previous_control/stop and
clears transition. This is a registry-only mechanical disposition; no joins, Store
writes or synchronous handle destruction occur under that lock. Retired Control
and intent handles are returned for drop after unlock. The old Held Phase stays
unchanged, with no receipt or current native identity. After restoration, all
existing confirmed-terminal/scope/Store-body guards still run before any later
registration; the join itself grants no native permission. Foreign/advanced Arc,
Lost, Unknown or outstanding requests retain the intent and exclusion. No generic
terminal label can execute the intent. More than32 sequential Joined fixtures on
the SAME adapter must keep fresh registration available. This prevents the verified
Design3 permanent retention leak without creating native recovery authority.

Lock discipline uses short isolated acquisitions. Custodian never takes registry,
Store or Preparation.admission while holding pool/inventory/Control.task. Registry
consumers read compact custody counters/status with a leaf snapshot; they do not
reap/join there. Pool admission collects finished handles, unlocks, joins/reconciles,
then reacquires counters. Synchronous actor Drop can safely query custody or take
registry because runtime.spawn holds NONE of those locks. Add custodied closed-runtime and pre-reservation missing-runtime fixtures,
not only old no-custody tests.

Exact opt-in is a private complete declaration on Control, injected with its pool
and CLOSED file factory through a fixture-private CodexAdapter input. Ordinary
production has NO factory and EMPTY remains first. Existing selected native/F4 paths
with no custody opt-in are explicitly UNMIGRATED and retain their existing behavior;
none earns this component's retained resource proof. spawn_launch/spawn_checkpoint
reserve the declared three frames BEFORE capturing Registered/TaskGuard in a future.
Known capacity refusal routes through existing refused_before_work only when the
actual inventory has zero effects/requests, and starts zero frames/resources.

With opt-in, ONE named call seam in each actual prepare_launch/prepare_checkpoint
runs the factory through the registered custodian immediately after availability
require and BEFORE context capture/Git/native preflight. The controlled factory
creates the actual file/worker in outer custody, then returns a labelled injected
context Err into the SAME existing context-error outcome arm. These negative
registry controls do not continue to native/Git effects. No-factory legacy paths
have no resource call; ordinary public EMPTY rejects before reaching either path.
Previous-snapshot registry construction is an EXPLICIT fixture-only factual history
seam, not SQL/private native allocation, validated UUID ownership or a production
constructor. It earns NO native provenance/binding/admission/settlement credit. It
allows the actual RegisteredTransition/register_existing/outcome guard to be tested
with a real newly owned file/worker; omission must rearm/hide that actual custody,
not merely trigger an unrelated lifecycle/currency guard. Managed producer and
native previous-snapshot provenance remain unavailable.

Cleanup results are fixed typed std worker return values stored until join. New-effect
requests check revocation BEFORE enqueue and again at handler effect. Fill the inbox
with refused/optional requests while the worker finishes: its join result still
carries cleanup, no dropped fact or spurious Unknown. Counters for accepted requests
retire only after handling. Detached-before-install handle loss is explicitly
Unknown with retained counters, NEVER a fake no-work proof; the bounded creation
control reaches this seam. The sole custodian returns a typed WorkerDisposition::NotCreated ONLY if factory
creation was never requested, or its revocation/generation check refused before
ANY worker thread spawn attempt. This fact is part of the creator
JoinHandle return value, never request JSON or a caller assertion. The pool
releases that unused worker slot only AFTER actually joining the custodian outside
locks; no extra job/reaper is introduced. All actually created frames must still
be joined and requests drained before final Joined. An attempted thread spawn with
opaque Err, detached-before-install handle loss or creator panic NEVER means
NotCreated and retains the uncertain frame's slot. Actor join Cancelled/panic
stays custody::Unknown and retains that actor slot; it does not prevent release of
a separately proven worker-NotCreated or normal custodian-joined slot.

Custodied closed runtime has an exact mechanical result: the cancelled actor join
is Unknown and keeps ONE actor slot; a normally joined custodian returning
WorkerDisposition::NotCreated releases its own and the unused worker slot. Missing
runtime still reserves/creates ZERO slots. Wrapped never-Begun actor/opaque spawn
results are observed through that same total per-frame disposition, never
absence inferred from Err. This availability cost is explicit and no Lost is
cleared. If custodian thread creation returns opaque Err before actor/worker
creation is attempted, the selected component deliberately retains ALL3 reserved
slots as a bounded isolated-fixture availability cost. No worker/resource effect
exists, so existing refused_before_work handles the registry disposition; unknown
job accounting stays in the pool. The injected-creation-error control observes3
held, and an Err-to-free mutant fails it. No new release authority is introduced.
The delayed registry disposition remains stricter: it requires an
actually CREATED and genuinely joined worker plus joined actor/custodian, so
NotCreated cannot launder a held resource or execute that original intent.

## Causal checks and source gate

Commit source before checks/review; preserve exact head and default parallel failures.
Selected unit/actual controlled fixtures must prove all of these at the real consumer:

- Custodied closed runtime: no lock/drop deadlock; actual actor/custodian joins or
  held attention through the same protocol: cancelled actor keeps1, normal creator
  join+NotCreated free2. Missing runtime starts zero frames,
  leaves counters zero and preserves original no-work registration. Omit-precheck
  and omit-lock-release alter the corresponding actual consumer.
- Hold the worker on an explicit release gate until the actor publishes exact
  Phase::Finished(Outcome::CustodyHeld); then release and observe custody::Joined
  with Phase unchanged. Reverse ordering (worker join recorded by the custodian, zero accepted/in-flight requests, BEFORE
  the error arm) yields the original no-work result; actor/custodian slots remain
  reserved until housekeeping. Worker panic gives custody::Unknown.
- Full revoked request inbox cannot lose cleanup carried in the worker join value.
  A queue-only-cleanup mutant fails this actual consumer.
- Caller Drop/cancellation during preparation leaves the actual resource custodian
  alive; the inventory/permits remain until real cleanup AND joins are observed.
- Fail/panic after resource creation but before adapter return retains the installed
  outer anchor. An omitted anchor/Begin handshake mutant loses custody at this seam.
- Return injected Err with the CallerGuard disarmed, then a registered worker
  callback succeeds through the same endpoint; cancelled/foreign/reconstructed
  endpoints cannot effect/publish. Ok/Starting coverage remains explicitly pending.
- Fill the bounded inbox, request cancellation and join: handler drains or refuses,
  no join/send deadlock; omitted drain and absent size/capacity controls fail here.
- Endpoint closes BEFORE join, then normal worker completes: no spurious Lost.
  Enqueue-then-exit is processed before ordinary completion.
- Normal actor closure after all jobs joined does not become Lost. Outstanding job
  channel/custodian loss stays owned/Unknown; omit-outstanding predicate fails.
- Custodian panic after worker creation BEFORE Begin leaves file untouched and
  permits held; callback panic retains actual heap-owned file/worker until cleanup.
- Explicit revoke after Consumed AND Failing prevents new endpoint effects without
  injecting consumed native interruption or resetting first-cause.
- Same-Session postcreation Err with previous snapshot refuses re-registration
  while Tracking/Unknown. Genuine Joined reconciles only its exact original
  disposition. More than32 sequential fresh Joined fixtures on the SAME adapter
  remain available; omitted reconciliation fails actual capacity. Foreign/advanced
  Control and published Lost never restore/remove. Omitted outstanding guard hides
  the actual worker and fails the prereconciliation negative control.
- More than21 sequential caller-Drop-before-factory attempts on the SAME isolated
  pool (and more than32 on the SAME adapter) replenish after actual actor/custodian
  joins and observed NotCreated. Each iteration explicitly waits for the custodian
  finished fact and drains outside locks, asserting zero counters/registry entries
  BEFORE the next admission;
  omitted NotCreated disposition exhausts actual capacity. Opaque spawn Err remains
  held; an Err-to-NotCreated mutant cannot refund its unknown slot.
- More than64/3 sequential completed fixtures replenish through actual custodian
  reaping; no-reap/self-release and Drop-permit mutants fail real accounting.
- Capacity exhaustion starts zero jobs. Unknown jobs retain slots; genuinely joined
  jobs release only their own reservation. Counters include all actual component jobs.
- Existing production EMPTY public routes still refuse before Store/process/Git/bytes;
  existing native own-Session and first-cause tests remain unchanged where applicable.

Use closed synthetic fixtures with explicit actual effect inventory and safe owned
cleanup. Do not seed private SQL/native leases. Report actual process/thread ownership
and observations, not fixture labels as proof. Compiled omission controls must alter
this consumer; an unused helper or a blanket Unsupported result cannot earn custody
credit. Future real managed producer/original-frame/callback/native settlement and
60/58/14/23/43 integration require separate actual source/conformance gates. This
component cannot make PR39 ready or close19/6/60.
