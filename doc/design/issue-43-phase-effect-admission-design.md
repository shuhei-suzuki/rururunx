# Issue 43: Retained source handoff and actual effect admission

## 1. Scope and source

This is Root's finite implementation supplement to the approved Source handoff
design (`0f13284`) and corrected Native preparation design (`c4d581a`). Root
baseline is `e1f52cd5bd2676effb6ec9d95ac9fe6becf29646`; the concrete Source API
is the separately reviewed candidate at `63507400f9751b3d11b29de9e9cc37bb4db2dd91`.
Its qualification is separate from this proposed consumer. No native availability,
whole Runtime, actual four-Task or macOS/Linux qualification is implied.

The representation fixes already retain the original Arc Driver ticket through
marker planning and keep the worker composition's queue reference Weak. The
actual installed composition issuer remains Err. This supplement connects real
objects; no callback, readable row, arbitrary future or readiness switch creates
handoff, preparation, input or Native permission.

## 2. Shared actual admission

Runtime creates one private `PhaseEffectAdmission` from its actual owner, actual
control-admission mutex and actual running/stopping state. Runtime control and
trusted stop use the SAME mutex. The existing mutex and stopping state become
shared Arcs; these retain exactly the existing objects, not copied state or a
second lock. The running Arc remains the one updated by the real service guard.
PhaseSupervisor retains the same admission object; admission has no strong
Runtime, queue, Source, Driver registry or job reference.

The actual PhaseLaunchParts exposes that same object through its genuine retained
publication supervisor. `PhaseEffectAdmission::enter` accepts only an Arc of the
actual launch, acquires the actual owned mutex asynchronously, and checks actual
service state, retained original launch/queue, selected owner and pointer-equal
admission origin. It returns a private non-Clone `PhaseEffectAdmissionGuard` which
retains that mutex guard and the SAME launch. No new launch is built from IDs.

The guard's `validate_for` checks the actual launch/admission/owner identity and
stopping state. It is nongrant by itself. Every Native transaction must separately
check original current successor, live Driver, preparation, Unit, pair, lifecycle
and exact mutations. Private helper/transport consumers hold admission only over
their synchronous last-check, known intent/registration commit, Store release,
spawn and immediate actual child-retention section. They release it before I/O,
capture, handshake or any further await. No Store/queue/Source/child mutex spans
an await. A native start gate is separate and cannot substitute for stop admission.

## 3. Synchronous envelope and independently owned invocation

Runtime's bounded `PhaseHandoffs` is a sibling of Sources, PhaseSupervisor and
PhaseJobs. It retains at most 128 original operations, including Held entries;
no fullness policy evicts assets. Operation and Unit duplicate checks inspect the
actual private envelope and original ticket, not metadata as authority. The
concrete synchronous port is:

```text
Runtime::retain_source_handoff(SourceNativeHandoff)
    -> Result<SourceHandoffObservation, SourceNativeHandoff>
```

It returns the SAME envelope on ordinary refusal. On acceptance it wraps and
retains the actual envelope before constructing the admission future, installs
an eager running/abandonment guard BEFORE Tokio spawn, and saves the actual
JoinHandle before returning. The Engine calls it in the SAME poll after the
Source producer returns, before an await. Engine cancellation owns only the
nongrant observation; it cannot abort or dispose the sole producer future.

The retained job's concrete consumer parts contain original owner, phase queue,
phase jobs and actual admission/state Arcs, never a strong Runtime. PhaseJobs
becomes a shared Arc of the existing registry. The future captures its inner
state/envelope and concrete parts; its JoinHandle lives only in the outer registry
entry. Inner state never owns that handle or registry. Driver composition keeps
its queue Weak, and the Engine receives only a Weak/watch observation. Neither
the Source capsule nor queue slot stores the handoff envelope/worker registry.

## 4. Original ownership transfer and marker

The consumer first acquires actual control admission, checks service and the
original retained envelope/ticket, and revalidates that SAME ticket outside Source
locks. It upgrades Source's transient original origin and obtains its exact
map -> tried slot -> custody transfer borrow. With no intervening await or
SharedStore acquisition it takes the original allocation/guard and calls the
actual synchronous PhaseSupervisor reserve. Provisional allocation checks use
the existing separate read-only SQLite snapshot; they are not marker authority.

Ordinary refusal restores the SAME returned objects under the Source borrow.
Held restoration remains Held. A protocol-error Box is saved in the retained job
before releasing the borrow, then kept or disposed only outside Source locks.
On success the job saves the actual capacity before Source's infallible
finish_accepted. An unfinished/unwound transfer remains Held; no rows, new guard,
fresh allocation or current ticket reconstruct its continuation.

After releasing all Source locks/origins, Root prepares the original marker from
the SAME capacity/allocation and producer-held Arc ticket. Planning failure keeps
all original assets. Root saves the completed SAME plan before marker SQL, reserves
the actual PhaseJob, and uses the existing private marker writer/handoff. A
confirmed marker starts the selected actual Native port once. Uncertainty uses
the exact saved original plan and existing absence/postimage checks. An Engine
observation, Session None or a result label never authorizes rollback/replay.

## 5. Genuine preparation origin and Native seam

Root creates a private non-Deserialize `PhasePreparationOrigin` only in that
concrete successful Source-to-queue transfer, from the actual envelope/capsule
and returned actual capacity. The queue retains this original origin alongside
the SAME armed guard/allocation before marker publication. The origin retains
immutable Source custody and original allocation identity, with only a Weak
reference to the SAME original Arc ticket. Marker's advance strongly retains that
ticket. It owns no capacity, marker, Runtime or registry; no return cycle is added.

Native receives the original origin only through the genuine PhaseLaunchParts.
Origin validation compares the original allocation/marker/ticket objects and
original complete encoded input. It never builds proof from current Source rows,
an Accepted observation, copied Unit facts or a fresh ticket. Full encoding and
guard inspection finish outside SharedStore. Native's Immediate conjoins sealed
original-origin identity with existing current-successor and Driver-live checks,
complete Unit/input/pair/readiness eligibility and compiled exact permissions.
No Source/queue mutex is acquired under SharedStore. Real stop admission prevents
concurrent queue ownership mutation during the synchronous effect section.

The private Native stage may use a zero-helper preparation plan only when that
actual origin and admission exist. Its known commit changes readiness alone;
it grants no helper, transport spawn, input or ACK. Helper/transport/quota writers
remain their separately owned, reviewed concrete consumers. Their shared factoring
must preserve ordinary authority predicates and cannot add public bypass modes.

## 6. Stop, reconciliation and controls

Stop sets the SAME stopping state while holding admission and prevents new
handoff/effect admission. Unmarked removal uses the existing actual queue policy;
publishing/uncertain/native assets remain charged until genuine reconciliation.
Shutdown includes retained handoff jobs/assets in its pending result and cannot
report completion while Held originals remain. Runtime Drop fences the shared
state without requiring a strong Runtime in the worker. No cleanup guarantee or
success/failure inference follows from worker exit or a reclaimed process group.

Required genuine controls cover observer Drop/unpolled future, capacity refusal
restoration, protocol-error retention, Source removal and duplicate admission,
marker planning failure with SAME ticket retained, saved-plan uncertainty, stop
before/after registration and before spawn, post-child pre-Core faults, and four
Task independence. Standalone gate/Weak/removal mechanical controls are labelled
nongrant and cannot qualify these positives. Compiler/setup failures are not
successful controls. Whole composition/preflight stays refused until genuine
producers and consumers are installed and positively qualified; existing full
regression/lint and authenticated compatibility gates remain open.

## 7. First implementation increment

Runtime now shares its actual control mutex and actual running/stopping Arcs with
the original queue's private PhaseEffectAdmission. Genuine launch exposes that
same object. Enter checks service, original owner/admission pointer and retained
launch before returning the local owned lock guard. Its transaction-side validator
checks original object/state only and acquires no queue/Source mutex. PhaseJobs is
an Arc of the same existing registry for the forthcoming concrete consumer.

This increment installs neither PhaseHandoffs nor PhasePreparationOrigin nor a
Native caller. No Native effect is authorized by admission alone; the composition
issuer remains Err. Cooperative stop is serialized by the shared mutex. Runtime
Drop or service-loop failure can still race an already admitted synchronous
section; state/Driver checks fence subsequent permission and retain uncertainty,
but no universal no-effect, child-death or cleanup guarantee follows. Genuine
enter/drop/stop/effect ordering remains unqualified until real producers work.

## 8. Retained Source transfer increment

The actual Runtime now owns bounded PhaseHandoffs separately from Sources and
the phase queue. Its concrete synchronous port accepts only the non-Clone actual
Source envelope. The same envelope/ticket reaches independent registry ownership
before the invocation future exists; an eager abandonment guard precedes spawn,
and the actual JoinHandle is saved before returning a nongrant watch observation.
The future has original owner/queue/control/state objects but no strong Runtime,
outer registry or own JoinHandle. Ordinary admission refusal returns the SAME
envelope; a future Engine caller must explicitly retain it, not discard it.

The first-Executor lane conservatively holds at most one handoff per actual Task
and at most128 total, including refused/Held entries. It never evicts originals
because an observer or future ended. Under SAME control admission, the consumer
revalidates the original ticket before Source locks, obtains original Source
map/tried-slot/custody, takes original allocation/guard, and calls actual queue
reserve without await or SharedStore acquisition. Actual capacity is saved before
Source acceptance. Ordinary queue refusal restores SAME assets; malformed returns
are retained in the independent entry before Source locks end. Abandonment remains
Held. Shutdown cannot report completion while any such originals remain held.

This increment ends at original capacity custody. It installs no preparation
origin seal, marker continuation, Engine caller or successful composition issuer.
In particular, Reserved is a nongrant observation, not proof of prepared input,
current marker, Native dispatch or Session binding. Genuine offer/observer-drop/
refusal/stop/uncertainty controls require the missing installed producer; no SQL
seeds or manufactured tickets qualify them. Full regression/lint and actual
four-Task/both-OS/authenticated verification remain open.

## 9. Original preparation lineage increment

The concrete successful Source transfer now creates PhasePreparationOrigin only
after it has saved the actual capacity and finished original Source acceptance.
Private fields retain the original immutable Source capsule and SAME allocation
Arc, which already owns complete canonical input bytes. The ticket edge is Weak
to the SAME original Arc ticket; there is no strong marker/capacity/Runtime/registry
backlink. The independent Handoff retains the origin before queue installation;
queue failure therefore leaves actual capacity/origin held without remint.

The same actual unmarked queue slot retains that origin beside the armed guard.
Original marker planning requires it before SQL effects, and known marker handoff
captures the SAME origin into PhaseLaunchParts after checking the same allocation
and original marker-owned Driver ticket. IDs and Source Accepted labels alone
cannot construct it. An unmarked removal or speculative capacity never creates a
replacement origin. The real control mutex continues to serialize cooperative
queue transfer/marker/stop, with inherited Runtime Drop race limitations.

PhaseLaunchParts::validate_preparation_origin_tx performs only immutable original
lineage checks and existing exact current-successor/Driver-live checks in the
caller's transaction. No Source/queue mutex, encoding or recaptured read is used
under SharedStore. Native must additionally validate the actual effect admission,
selected owner/port, complete Unit/input/pair/readiness/lifecycle and compiled
exact mutations; this conjunction is not permission or prepared completion itself.

The real Native consumer, retained marker continuation and Engine caller remain
uninstalled, and composition issuer remains Err. Source acceptance/lineage and
all genuine cancellation, capacity, marker uncertainty and stage controls remain
unqualified until that actual composition exists. No grant-bearing fixture or
SQL-seeded Native allocation/Driver proof is introduced by this increment.

The corrected pre-planning consumer requires the supplied original ticket Arc
to match that accepted origin BEFORE making a marker plan or performing SQL.
A fresh genuine ticket with equal rows is refused before Task/Workflow/Driver/
operation effects; checking only the eventual marker at launch is too late.
The completed plan is also checked against the original allocation/ticket before
admission, and saved-plan reconciliation repeats that same original linkage before
Store confirmation. The post-commit launch check remains an additional condition.
This corrects verified independent SPO-A-M1/B-SPO-M1; the genuine second-ticket
no-effects control and its compiled mutation remain unqualified while the actual
composition issuer is absent. No setup refusal is credited as such a control.
