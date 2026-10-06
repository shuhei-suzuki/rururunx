# Issue 43: Source handoff ingress before the actual offer

## 1. Source and finite gap

Root source is `ddc66a2b8a7a00ed58e7497e677098bf5280222f`. Its actual retained
Source transfer ends at capacity and original preparation lineage. Engine still
waits at an active first Executor; no installed composition issuer exists.

The post-offer Runtime port returns the SAME envelope on ordinary stopped/full/
duplicate/poison refusal. A prospective Engine must not discard that returned
envelope: Source has already installed the custody with a Weak original ticket,
and losing the sole producer ticket would lose original marker continuation.
Keeping a strong Runtime across the Source await is also unsuitable: the actual
Runtime Driver registry owns that worker and could form a return ownership cycle.
This supplement reserves nongrant ingress space BEFORE Source offer. It does
not create Source, Driver, preparation, marker or Native authority from metadata.

## 2. Concrete independent ingress reservation

Actual Runtime's existing PhaseHandoffs becomes an Arc of the SAME registry,
not a second registry or issuer. It holds at most128 original Task slots, including
filled/refused/Held slots. Each slot has a nongrant original Task/scope key, one
optional actual Handoff, an outer optional actual JoinHandle and a watch state.
The actual invocation future captures only the filled inner Handoff and concrete
owner/phases/jobs/control/running/stopping objects. It owns neither its outer slot,
registry, JoinHandle nor a strong Runtime.

The specific private ports are:

```text
Runtime::reserve_source_handoff(&DriverReadTicket, &Arc<NativePhasePort>)
    -> Result<SourceHandoffReservation>
SourceHandoffReservation::install(self, SourceNativeHandoff)
    -> SourceHandoffObservation
```

Reserve checks the actual Runtime accepting state, actual owner/selected port,
original ticket Task selection and complete current read, then bounded same-Task
space. Read/encoding finishes before the registry mutex. Refusal precedes Source
offer and cannot move its original allocation/guard. The reserved EMPTY slot owns
no envelope, ticket, preparation, plan or permission; its metadata is bookkeeping.
Only the actual private Source producer returns the typed non-Clone envelope.
The pre-offer reservation retains a Weak of the original registry, the original
empty slot Arc, and only Weak references to the SAME PhaseSupervisor and PhaseJobs.
It may retain the original owner, control mutex and running/stopping atoms strongly:
these have no queue/launch/Driver return edge. It must not retain the concrete
Consumer, a strong Runtime, or any other component that leads back to the Driver
registry while the actual worker awaits Source offer.

Install consumes that non-Clone reservation exactly once. Before fallible origin
checks, readiness decisions, effect admission or future construction, it retains
the SAME actual envelope in that independent slot. Poison recovery is permitted
only to save real ownership. There is no ordinary envelope-returning failure after
this move: stopped service or a wrong original Task/port/envelope is retained Held,
never treated as accepted preparation or automatically disposed. No observer,
generic future or caller-supplied validator can replace the real producer.

Only AFTER this independent envelope retention, install synchronously upgrades
the SAME original Weak supervisor/jobs and constructs the invocation's concrete
Consumer. This upgrade has no await or Source/Store access. Ended original parts,
poison, or mismatched original owner/component identity leave the real envelope
Held in its slot. No replacement registry, supervisor or jobs are constructed,
and row/ID equality cannot substitute for these original objects. The transient
install call does not return a strong Consumer to the Driver worker.

Only a matching actual envelope is given to the existing concrete transfer.
Its eager abandonment guard exists BEFORE Tokio spawn, and its actual handle
reaches the independently retained outer slot before install returns. Every await
is outside registry/Source/Store locks. Actual stop admission and the original
ticket, Source borrow, concrete reserve, restoration, lineage and marker checks
remain mandatory. EMPTY, install/watch labels and capacity alone grant nothing.

## 3. Actual Engine and original worker callsite

InstalledDriverComposition exposes only its SAME original Weak Runtime observation.
The retained actual worker passes that Weak to step_driven_initial; no strong
Runtime is kept in its claim, Engine, Source, invocation future or parked state.

At a running first Executor, the actual Engine first looks for an existing
original Root handoff observation. Lookup returns Result<Option<Observation>>;
poison cannot look like absence and prompt a new offer. An existing watch state
is diagnostic only and never authorizes another allocation, retry or terminal
Workflow write. Marker/binding/terminal consumers remain separate genuine ports.

If no entry exists, Engine reads its actual current Task/Workflow/Context and
genuine same worker ticket, upgrades the original Weak Runtime only for the
synchronous reserve call, and drops that strong Runtime BEFORE Source offer await.
It then calls the actual Sources::offer_first_executor with that original ticket
and SAME selected port. On success it invokes reservation.install in the SAME
poll before any await, callback, fallible status projection or return.
The actual Source producer must remain inline: it has no independently spawned
offer whose result might arrive after the sole reservation owner disappears.

Source's accepted capsule installation already has no subsequent await or ordinary
fallible step. Tokio cancellation cannot interrupt a single synchronous poll
between that actual return and original envelope retention. A final process abort
or panic cannot be claimed as universal memory preservation; actual unpolled
guard/owned-handle controls and the all-owner-loss limitations still apply.

Source error or caller Drop before install cannot manufacture an envelope, new
ticket or successful custody. The EMPTY reservation may be removed only after
the concrete inline offer has ended/dropped and the slot still owns neither actual
envelope nor invocation/handle. Removal compares SAME slot Arc under the actual
registry lock, moves it out, then drops it outside locks. There is no general
remove-by-Task/timeout/status path. If this original-empty proof is unavailable
or any mutex is poisoned, retain visibly Held instead. Source's own preparation
remains its responsibility; EMPTY removal does not retire Unit/Task or infer
no external effects from earlier preparation helpers.

## 4. Failure and ownership boundaries

| Actual boundary | Required outcome |
| --- | --- |
| Stopped/full/duplicate/foreign reserve before offer | Refuse before new Source handoff; no input/preparation/native effect |
| Source offer pending, observer/worker cancelled | Original inline producer is dropped; no detached replacement; EMPTY removal only by original-empty proof |
| Source returns original envelope | Save SAME envelope in reserved independent slot in SAME poll; no intervening fallible action or await |
| Stop after reserve/before transfer | Filled original remains Held; real admission refuses effects; no ticket recapture |
| Queue capacity or ownership refusal | Existing original restoration/boxed retention; no automatic second offer or fresh worktree owner |
| Marker planning/commit uncertainty | Retain SAME ticket/capacity/origin/plan in independent consumer; original marker protocol only |
| All real Runtime/executor owners end | No universal memory/child-death/cleanup guarantee; durable uncertainty does not reconstruct private proofs |

The ownership graphs are deliberately different before and after install:

* Before offer: Driver registry -> actual worker future -> original reservation
  -> empty slot and nongrant owner/admission atoms; references from that reservation
  to the original handoff registry, supervisor and jobs are Weak. Neither the
  empty slot nor its watch retains the Consumer or a ticket. The worker's genuine
  ticket remains its pre-existing worker proof; the reservation adds no strong
  queue/job return edge.
* After install: Runtime -> handoff registry -> filled outer slot -> actual handle;
  the independent invocation owns the filled inner Handoff and concrete Consumer.
  The Consumer may now strongly own the SAME supervisor/jobs. It is never retained
  in the Driver worker, outer slot or envelope. The future does not own its outer
  slot, handoff registry, handle or Runtime. Dropping or refusing the observer
  does not drop the actual envelope or recreate a producer.

This distinction matters across Tasks. A supervisor's saved publication plan or
marker retains a ticket whose Driver association owns the ENTIRE shared Driver
registry, including other workers. PhaseJobs can similarly lead through a retained
launch to that supervisor and ticket. Therefore retaining either of those parts
strongly in a pending pre-offer Driver future would close a return cycle even when
the retained marker belongs to a different Task. The earlier concrete-consumer
proposal did not establish this boundary. This is a verified source ownership
graph, not an executed Native retention failure or a universal cancellation claim.

The outer registry owns its handle; the inner invocation and Source capsule never
own that registry/handle. Native preparation custody is a separate real PhaseJob
sibling and is not constructed or granted by this ingress. Its launch reference
to that custody remains Weak, preserving the independently reviewed cycle limit.
Stop/shutdown cannot report completion from a filled Held slot's future exit.

## 5. Required controls and delivery limits

Use the genuine retained Driver/Source/Engine producer, not SQL-seeded tickets,
allocation/custody DTOs, FakeAgent capabilities or an always-ready callback.
Controls must cover full/stopped reserve without offer, observer Drop/unpolled
invocation, inline offer cancellation, successful offer immediately retained,
stop between reserve and install, duplicate admission, actual queue refusal,
source retirement, original marker planning/uncertainty and four-Task independence.
Add a nongrant ownership/drop control for the empty reservation: ending the real
external supervisor/jobs owners must not leave them alive through that reservation.
It must not fabricate Driver/Native objects. Once the genuine composition exists,
also retain one Task's genuine marker while another Task's actual inline Source
offer is pending, then end that second worker and verify the absence of a strong
queue return edge without disturbing the first Task's original custody. These
genuine controls remain unexecuted while the installed issuer is absent.
Compiled removal of pre-offer reservation or SAME-poll retention must fail the
intended original-asset survival/no-effects control; missing issuer is setup,
not a successful control or mutation kill.

This is a proposed concrete ingress/Engine increment. Source availability,
successful Driver composition, authenticated native execution, helper/transport
permissions, full regression/lint and both-OS/four-Task acceptance remain open.
No code, capability, schema or README change follows from this design.

## 6. First source increment and remaining execution boundary

The implementation increment is based on the ordinary composition
`707554327520635ee3b8b62f2761210399d18e59`, which includes the independently
reviewed Native readiness-only source and corrected ingress design. Runtime now
owns an Arc of its SAME handoff registry. The actual Driver passes its original
Weak Runtime observation to Engine, and the active first Executor can reserve
EMPTY ingress before the concrete Source offer. The unchanged genuine preflight
still precedes every new offer and public adapter callback.

`SourceHandoffReservation::offer_first_executor` constructs a private eager
`InlineSourceOffer` from the actual Sources method, not a caller-supplied future.
Its poll saves a successful envelope in SAME poll before any fallible action.
Its Drop explicitly drops the sole inline offer before dropping the reservation,
so generator field destruction order is not the original-empty proof. There is
no spawned offer, generic producer injection or exported install operation.

The pre-offer consumer has Weak original supervisor and selected-port references,
and no PhaseJobs reference: this increment ends at capacity/original lineage.
Any future job/marker continuation must preserve the Weak pre-offer rule. Strong
consumer construction follows independent envelope retention; the original
producer ticket, port and owner pointers must match. Poisoned lookup returns Err,
not absence. Filled or Held slots are never released by observer/drop labels.

The new ownership controls exercise only actual EMPTY bookkeeping, real Runtime
service shutdown, queue/jobs Drop and poisoned registry lookup. They do not mint
tickets, allocations, Source custody, marker, Native actor or prepared-input proof.
The ticket-checked Source success/cancellation, genuine cross-Task marked queue,
compiled mutation controls and authenticated Agent routes need their own evidence.
An executable-missing setup error is not an ownership-control failure or kill.

Actual transfer still ends at capacity/original lineage. It does not call the
original marker publication, Native invocation, binding or terminal consumers.
The installed composition issuer remains unavailable, and Native helper/quota/
transport and genuine positive Workflow fixtures remain unfinished. This source
increment grants no capability or new SQL writer permission. It cannot qualify
actual Agent launch, full regression/lint, four-Task independence or both-OS CI.
