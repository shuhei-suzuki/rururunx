# Issue 43: original Source to marker and Native job dispatch

## 1. Concrete missing execution edge

Source baseline is `db756600ec5ab0660973de27780603a35071a264`. The actual
Driver/Engine reserves EMPTY ingress before the actual inline Source offer.
Independent custody saves its genuine envelope/ticket before a concrete transfer
accepts the original allocation/guard into the pending queue. Transfer currently
ends at capacity/original lineage. Neither this source nor its two independent
component reviews establishes a working Agent route.

This increment connects that SAME accepted capacity/ticket to the existing exact
marker producer and selected Native job. It adds no public writer, capability,
prepared proof, new schema, Session binding rule or successful-work projection.
The full managed binding and preparation contracts remain mandatory. Actual
helper/transport and the installed composition issuer remain separate open work.

## 2. Specific original dispatcher, without a Runtime return edge

The actual Runtime creates one private `PhaseDispatcher` from its SAME owner,
supervisor, PhaseJobs, control-admission mutex and running/stopping atoms. It
stores that Arc beside those original components. The dispatcher has no strong
Runtime, handoff registry, Driver, Source or own JoinHandle reference. It shares
the original objects, never constructs replacement queues/jobs/epochs.

The existing concrete marker publication, known-marker handoff and exact saved
plan reconciliation move into this dispatcher. Existing Runtime ports delegate
to the SAME object. The dispatcher preserves all existing original ticket/origin/
allocation comparisons, current/Driver CAS, actual job reservation, same saved
plan, retained known marker and one-time launch flag. There is no alternative
Store writer or optimistic wrapper around a generic start.

The pre-offer reservation retains only a Weak of this SAME dispatcher. Its owner,
control mutex and state atoms may remain strong; the selected port remains Weak.
The dispatcher strongly owns the queue/jobs, so retaining it in a Driver worker
would recreate the cross-Task queue->marker->ticket->ENTIRE Driver registry
return cycle. It is upgraded only after actual envelope retention and moved into
the independently owned invocation. No strong dispatcher is returned to Engine
or captured by the inline offer/Driver future.

The existing EMPTY ownership control must continue to exercise the real Runtime
constructor and show that neither supervisor nor jobs survive solely through the
pre-offer reservation. This does not qualify genuine marked cross-Task execution.

## 3. Actual transfer, planning and publication callsite

The independent concrete invocation has this order:

1. Enter the actual control admission and perform the unchanged original ticket
   read, Source origin/map/slot/assets borrow and concrete queue reserve. Save
   actual capacity before Source acceptance. Save and install original lineage
   as before. End EVERY Source/custody/queue borrow and the transfer admission
   before the publication continuation; no nested admission acquisition.
2. Borrow only immutable SAME allocation/origin Arcs from the retained capacity
   under a short custody lock, then release that lock. Validate the Source's
   SAME original ticket before planning. Construct one original
   `MarkerPublicationPlan` on the existing separate bounded readonly snapshot;
   no Store/Source/queue/custody lock spans planning, encoding or hashing.
3. Save that SAME plan in the independent Handoff assets BEFORE the publication
   admission await. The actual envelope still strongly retains its original
   ticket. No new ticket/Source offer/capacity/allocation is captured. Duplicate
   original plan installation is an error, never replacement with current rows.
4. Move the actual original capacity to `publish_planned_marker(capacity, plan)`
   on that SAME dispatcher. It acquires the original control admission, checks
   accepting state, exact original Source linkage and plan/capacity pointers,
   enters actual publication retention, reserves the real Native job and its
   EMPTY preparation custody, and saves SAME plan in the original queue slot.
5. Call the unchanged private `publish_managed_marker` under SharedStore. The
   actual exact-image permissions and current/Driver/Unit/input/owner conditions
   remain conjunctions. Release Store before known-marker handoff. Save first
   genuine known marker in the original slot, obtain SAME real job preparation
   custody, and create the original one-time launch with Weak custody linkage.
6. Call actual PhaseJobs start, whose original registry owns its real handle and
   whose eager guard/result cells preserve launch/Native actor/plan/error. Save
   the returned nongrant PhaseInvocation observation in Handoff assets. Engine
   observes this handoff and remains Waiting until genuine binding/terminal
   consumers supply their own result; no label creates a Workflow success.

Proposed specific private seams are:

```text
PhaseDispatcher::plan_original_marker(
    SAME allocation Arc, SAME Source origin Arc, SAME ticket Arc, workflow)
    -> Result<Arc<MarkerPublicationPlan>>
PhaseDispatcher::publish_planned_marker(
    original PendingPhaseCapacity, SAME saved plan Arc)
    -> Result<PhaseInvocation>
PhaseDispatcher::reconcile_phase_marker(operation_hint)
    -> Result<PhaseInvocation>
```

The new planner receives only originals from actual successful transfer. An
allocation, capacity, metadata or a helper result alone is not an origin. The
queue still owns its exact allocation and armed guard independently when a
caller handle is moved or dropped. Reconciliation uses its SAME retained slot/
original plan and existing exact postimage protocol, never a row-derived plan.
This increment adds no scheduler that automatically replans or replays effects.

## 4. Refusal, cancellation and known outcomes

| Boundary | Retained outcome |
| --- | --- |
| Stop after offer but before transfer | SAME envelope Held, no source recapture |
| Actual queue refusal | Existing original restoration/boxed custody, no publication |
| Planning fails | Original envelope, accepted origin and queue allocation/guard remain; no marker effect |
| Cancellation during publication admission | Independently saved SAME plan remains; queue still owns actual originals; no inferred rollback/no-dispatch |
| Job full, origin/plan mismatch or stopped admission | Refuse before marker SQL; retain originals and any genuine unstarted job |
| Marker commit returns error/unknown | SAME plan in Handoff/queue; no fresh publication or automatic retry |
| Known marker, handoff error | Preserve SAME known marker/job/plan; existing original reconciliation only |
| Native start returns error, Waiting or is unpolled/aborted | Existing actual job/preparation custody; no Task failure or new source offer from the observation |

The publication continuation runs after the Source transfer returns, so the
Source map/slot/assets locks and first control guard are absent before readonly
planning and the second admission await. No publisher awaits child I/O. The
selected Native job owns its own async preparation and actual permission checks.

Filled/held handoffs, marked capacity and Native jobs remain charged pending
their genuine logical closure. No observer, future exit, ID or generic cleanup
call retires them. Cooperative shutdown reports pending; Runtime/process/all-owner
loss still has the documented uncertainty and best-effort hygiene limitations.

## 5. Required verification and delivery boundary

Review fixed clean source independently after the approved HOW. Run compile/fmt/
build, existing binding/preflight/Source/Driver/Native controls and full regression/
lint, retaining failures rather than adding fake capabilities or bypasses.
The EMPTY ownership control needs a compiled strong-return-edge mutation and
exact fresh restoration; it qualifies only that nongrant graph.

Genuine controls require actual accepted Goal/Driver/Source/capacity/marker/job
producers: SAME ticket versus a genuine row-equal different ticket before effects,
stop before publication, caller Drop at admission, known/uncertain marker commit,
unpolled actual Native job and exact-original handoff once. Compiled omissions of
saved plan/origin/admission/once checks must reach intended runtime assertions.
Unavailable composition is setup refusal, not a positive or mutation kill.

The installed issuer and native preflight remain unchanged by this increment.
Native version/helper/profile/quota/transport, complete prepared input, sole
normal/late binder, owned terminal/retirement, recovery and real Agent/four-Task/
both-OS acceptance remain open. Connecting this actual callsite is progress
toward those paths, not completion of the MVP or permission to merge.
