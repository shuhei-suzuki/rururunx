# Issue19 retained preparation lifetime component

Status: selected MECHANICAL Design3 gate, not whole Issue19 authority/native readiness.
Parent Design37 at75cdae974da1ce9afa0a6de4e2cafc5065e60748 was rejected by both native
reviews. Its broad open findings are retained in the adjacent ledger. Absence of a
lifetime High/Medium in those rejected reviews is NOT this component's approval.
Selected Designs1/2 were also rejected. Design3 below corrects the remaining
installation/completion/queue-delivery and exact-consumer seams; TWO explicit
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
actual file outside the adapter future; worker completion plus actual joins/close
is the mechanical proof. No additional bootstrap, reader, reaper or waiter is
created. Ordinary unavailable native resources remain unavailable; no CLOSED native
profile constructor exists. A missing complete declaration refuses before creation. The real native profile/6F1/60 job sets are unavailable,
not inferred from the fixture. Reserve atomically; insufficient capacity starts
zero jobs/effects. Release only genuinely joined jobs; uncertainty retains their
reservation. Explicit pool counters are NOT Drop-released RAII permits. Each
fixture injects an isolated private pool through the SAME constructor/reservation/
consumer code; production has one process-wide pool. Unknown fixtures cannot
consume another test's capacity or require serial Cargo execution. Queue memory/resource reservation has its own finite bound below.

Actual creation goes INTO the supervisor's outer anchor before the adapter can
await, fail or panic at an attach step. BOTH created custodian and resource worker wait for separate Begin handshakes
until their actual handles/heap inventory are installed. Begin disconnection exits
without resource effect and remains joinable, never means proceed. Effect-capable factory
creation runs on that registered custodian; resource handles never live solely in
an adapter local waiting for later transfer. The resource holder, worker handle and actual custodian handle live in retained
Control/pool heap inventory, NOT the custodian or adapter stack. Custodian callback
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

For a custodied Control: reserve declared three frames; create/install custodian
handle under Begin; mark TaskOwner::Installing; spawn the actor WRAPPED to wait for
its own Begin; store the actor JoinHandle into that same heap inventory and its
AbortHandle/identity in a new Observed state; then send Begin. The actor cannot
create resources before installation. NO pool, inventory, Control.task or registry
lock is held across runtime.spawn, Begin send/disconnection, actor future Drop or
other synchronously dropping operations. Separate post-spawn acquisition installs
the handle even if the closed runtime already dropped the actor. No lock is held
across create/await/join/resource destruction. TaskGuard/release_task marks its logical
Released state but NEVER drops the custodian-owned actor JoinHandle. Closed/missing
runtime and no-custody legacy paths preserve their existing Running/Released tests.
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
request. CallerGuard::drop ALWAYS sets job revocation while armed, including Consumed and
Failing, but preserves existing consumed-turn no-interrupt behavior: no new native
stop message and no Preparation::Consumed reset. Normal return disarms CallerGuard,
so a successful post-return actor endpoint remains usable until explicit revocation. Authoritative local cleanup facts are returned by their OWN JoinHandle outcome,
not this saturable request inbox. Already-owned jobs retain that outcome until the
custodian observes the join; optional progress refusal cannot lose cleanup facts. The handler
checks the latch/generation immediately before effect; no recreated endpoint or queue
message can clear it. No native semantic grant implements this mechanical latch.

Actual reused session consumers MUST check outstanding effect/resources/accepted
requests: refused_before_work, prepare_launch/checkpoint context-error outcomes,
RegisteredTransition::drop restore/remove, register_existing and terminal eviction.
No RestoredBeforeAdmission/FreshUnpublished/CheckpointCommitted no-work classification
or old-control restoration/removal is allowed while such custody remains. Retain the
exact same registry Control/exclusion and explicit held outcome; registration refuses.
Control execution/custodian bookkeeping alone is not a native effect or death proof.
No source restoration permission is inferred from a slot counter/terminal label.
A genuine later local join may release THAT component job slot only; it cannot clear
published Lost or authorize native resume. Existing already-no-effect paths with zero
created resources/requests retain their previous behavior. These guards introduce no
managed receipt, current-frame refresh or native recovery exception.


## Single completion owner and concrete fixture consumer

Phase remains the provider actor's single-final logical result. Custody has a
SEPARATE private fact: Installing/Tracking/Joined/Unknown, plus registered jobs,
accepted requests and in-flight callbacks. Only the custodian derives Joined after
actual joins and drained requests; actual custodian loss derives Unknown, never
provider terminal success. It NEVER overwrites Phase::Finished or published Lost.
When an actor would select a no-work/restored result while effect resources remain,
it selects one private Outcome::CustodyHeld with cause and retained non-authoritative
snapshot, whose snapshot()/no-work selectors return NONE. Phase is Finished with
that held result; TaskGuard does not fabricate Lost merely for a KNOWN joinable worker.
wait_finished returns this exact held result. The later normal worker join makes the
separate custody fact Joined, leaves Phase unchanged and releases only genuinely
joined component slots. Worker panic/unobservable completion makes custody Unknown;
TaskGuard's independent genuine actor panic/drop continues its existing Lost behavior.
No held Phase authorizes native resume/recovery/success. register_existing and terminal
eviction reject Held/Unknown/outstanding facts; RegisteredTransition preserves the
same Control/exclusion, never restores/removes it from a no-work label. This is
mechanical bookkeeping, not a managed settlement or new native recovery exception.

Lock discipline uses short isolated acquisitions. Custodian never takes registry,
Store or Preparation.admission while holding pool/inventory/Control.task. Registry
consumers read compact custody counters/status with a leaf snapshot; they do not
reap/join there. Pool admission collects finished handles, unlocks, joins/reconciles,
then reacquires counters. Synchronous actor Drop can safely query custody or take
registry because runtime.spawn holds NONE of those locks. Add both custodied
closed-runtime and missing-runtime fixtures, not only old no-custody tests.

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
control reaches this seam. Wrapped never-Begun/cancelled actor and opaque thread
spawn failure conservatively retain isolated-pool attention/slots unless actual
join observations prove that component frame closed; this availability cost is
explicit, no native Lost clearing or absence inferred from Err.

## Causal checks and source gate

Commit source before checks/review; preserve exact head and default parallel failures.
Selected unit/actual controlled fixtures must prove all of these at the real consumer:

- Custodied closed/missing runtime: no lock/drop deadlock; actual actor/custodian
  join and held counters observed through the same protocol. Omit-lock-release fails.
- Actor context Err with worker normal return gives exact Phase::Finished(Outcome::CustodyHeld) plus
  custody::Joined (Phase unchanged); worker panic gives custody::Unknown.
- Full revoked request inbox cannot lose cleanup carried in the worker join value.
  A queue-only-cleanup mutant fails this actual consumer.
- Caller Drop/cancellation during preparation leaves the actual resource custodian
  alive; the inventory/permits remain until real cleanup AND joins are observed.
- Fail/panic after resource creation but before adapter return retains the installed
  outer anchor. An omitted anchor/Begin handshake mutant loses custody at this seam.
- Return from the starting call, then a registered actor callback succeeds through
  the same endpoint; cancelled/foreign/reconstructed endpoints cannot effect/publish.
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
- Same-Session postcreation Err with previous snapshot refuses re-registration;
  omitted registry restore/remove guard cannot hide the outstanding resource.
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
