# Issue19 retained preparation lifetime component

Status: selected MECHANICAL design gate, not whole Issue19 authority/native readiness.
Parent Design37 at75cdae974da1ce9afa0a6de4e2cafc5065e60748 was rejected by both native
reviews. Its broad open findings are retained in the adjacent ledger. Absence of a
lifetime High/Medium in those rejected reviews is NOT this component's approval.
This document receives TWO explicit independent selected-component gates before code.
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
or held jobs is a work limit, not a count of arbitrary OS/helpers/native tools. A
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
reservation. Queue memory/resource reservation has its own finite bound below.

Actual creation goes INTO the supervisor's outer anchor before the adapter can
await, fail or panic at an attach step. A created worker waits for a Begin handshake
until its JoinHandle and resource holder are installed. Effect-capable factory
creation runs on that registered custodian; resource handles never live solely in
an adapter local waiting for later transfer. The custodian's resource holder is
outside a catch-unwind boundary around polling the adapter's inner future. A pure
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

Use a bounded64-request inbox. Each COMPLETE encoded metadata request is at most4096
bytes before enqueue, so queued metadata≤256KiB; referenced immutable payload/resource
bytes retain their existing separately charged bounds, never an unbounded hidden
message. No serialized closure/operation credential or copied prepared frame enters
the inbox. Job registration/actual resource ownership precedes the first request.
Generation/control mismatch, revoked endpoint, over-bound metadata or queue capacity
refuses before the requested effect. Cancellation revokes new effects; already-owned
jobs still report cleanup/outcome facts with their exact identity.

Control keeps draining the inbox while awaiting jobs and deriving observed completion;
it NEVER awaits a join while abandoning a sender blocked on that inbox. A request
uses bounded try-enqueue or cancellation-aware send; no blocking Drop and no hidden
per-send task. Held resources do not cause healthy actors to retain unrelated partial
job reservations. Channel loss is uncertainty ONLY while registered jobs/callbacks
remain outstanding. A normally closed endpoint after actual job completion/join is
ordinary completion. Runtime/custodian loss with outstanding resources retains
Unknown attention and permits; no row/terminal label can recreate it.

## Causal checks and source gate

Commit source before checks/review; preserve exact head and default parallel failures.
Selected unit/actual controlled fixtures must prove all of these at the real consumer:

- Caller Drop/cancellation during preparation leaves the actual resource custodian
  alive; the inventory/permits remain until real cleanup AND joins are observed.
- Fail/panic after resource creation but before adapter return retains the installed
  outer anchor. An omitted anchor/Begin handshake mutant loses custody at this seam.
- Return from the starting call, then a registered actor callback succeeds through
  the same endpoint; cancelled/foreign/reconstructed endpoints cannot effect/publish.
- Fill the bounded inbox, request cancellation and join: handler drains or refuses,
  no join/send deadlock; omitted drain and absent size/capacity controls fail here.
- Normal actor closure after all jobs joined does not become Lost. Outstanding job
  channel/custodian loss stays owned/Unknown; omit-outstanding predicate fails.
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
