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

The connection is implemented at `ec5cc6e1a3cd7a234ccec810b597c008a6f93842`.
Its closed three-family design review found publication-outcome and shutdown
retention gaps. Section 7 specifies their proposed correction. The repaired
ordering and retention below are requirements, not claims that those corrections
already run. The installed issuer, full regression and lint remain open gates.

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
4. Save original publication custody beside the SAME plan before the second
   admission await. On that SAME dispatcher, acquire the original admission and
   check accepting state, original Source linkage and plan/capacity pointers.
   Validate and save the SAME slot plan and reserve the real unstarted Native
   job/EMPTY preparation custody before latching Publishing. Retain the resulting
   SAME publication retention before any marker transaction; a partial refusal
   preserves its actual owned assets and never deletes a pre-existing job.
5. Call the private `publish_managed_marker` with unchanged exact SQL guards
   under SharedStore. The
   actual exact-image permissions and current/Driver/Unit/input/owner conditions
   remain conjunctions. Release Store before known-marker handoff. Save first
   genuine known marker in independent original publication custody before any
   fallible handoff lookup; retain it in the original slot, obtain SAME real job
   preparation custody, and create the one-time launch with Weak custody linkage.
6. Call actual PhaseJobs start, whose original registry owns its real handle and
   whose eager guard/result cells preserve launch/Native actor/plan/error. Save
   only the observation needed by its actual consumer. Engine currently observes
   handoff transfer state and remains Waiting until genuine binding/terminal
   consumers supply their own result; no label creates a Workflow success.

Proposed specific private seams are:

```text
PhaseDispatcher::plan_original_marker(
    SAME allocation Arc, SAME Source origin Arc, SAME ticket Arc, workflow)
    -> Result<Arc<MarkerPublicationPlan>>
PhaseDispatcher::publish_planned_marker(
    independently retained original publication custody)
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
| Job full, origin/plan mismatch or stopped admission | Refuse before marker SQL; preserve actual stage/assets; do not leave Publishing without its plan |
| Definitive refusal before marker commit | SAME custody/plan; existing successful original unpublished proof may restore Unmarked and remove only SAME unstarted job |
| Commit or postcommit-cache result is uncertain | SAME custody/plan and actual outcome; reconcile exact original postimages, never infer rollback from error text |
| Known marker, handoff error | Preserve SAME known marker/job/plan; existing original reconciliation only |
| Native start returns error, Waiting or is unpolled/aborted | Existing actual job/preparation custody; no Task failure or new source offer from the observation |

The publication continuation runs after the Source transfer returns, so the
Source map/slot/assets locks and first control guard are absent before readonly
planning and the second admission await. No publisher awaits child I/O. The
selected Native job owns its own async preparation and actual permission checks.

Accepted-Unmarked slots, filled/held handoffs, marked capacity and Native jobs
remain charged pending
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

## 6. Impact analysis and implementation boundary

`Runtime::new` creates the dispatcher from the same original objects. Existing
`publish_phase_marker`, `reconcile_phase_marker`, `retain_marker_publication`
and `rollback_marker_publication` delegate to it; pending reservation retains
the existing Runtime wake path. The concrete Source invocation now calls the
planner and saved-plan publisher after its transfer function returns. Handoff
assets independently retain the original plan and returned job observation.

Consumers are `runtime/phase_handoffs.rs`, `runtime/phase_supervisor.rs`, the
Runtime constructor and the existing supervisor/ingress controls. The existing
EMPTY graph control also checks the actual dispatcher lifetime. The master
Workflow description records this internal connection and its unavailable
installation boundary. No public API, SQL schema/guard, admission capability,
threshold, environment variable, helper command, authentication or license
changes are included. Capacity and backoff constants and selected Native ports
remain the existing consumers. The new private dispatcher adds one shared Arc
and does not add a polling loop or another database connection.

Security self-check: Source input reaches the existing readonly marker planner,
then the unchanged private exact marker transaction and actual selected job.
The original managed actor/ticket may proceed only with original accepted Source,
current Unit/Driver and stop admission. Another Task/owner/ticket, an ID-only
caller, an unauthenticated external caller or an administrator label cannot
produce these private originals or bypass the checks. The local CLI has no Web
user/admin roles; those labels do not imply extra authority. Receipt/status/watch
data never grant Native or Session/input permission. Current positive composition
is unavailable, so this source-level analysis is not empirical security approval.
UI/browser/STG verification is inapplicable to this internal backend-only change;
authenticated CLI/both-OS qualification and regression remain open gates.

## 7. Publication outcome and accepted-custody amendment

The verified review groups are publication outcome loss, pre-latch partial
installation, first known-marker loss, accepted-Unmarked shutdown drainage,
unused diagnostic/wrapper surfaces, and contradictory legacy-binder wording.
This amendment fixes those contracts together. It does not introduce a new
allocation, marker permission, schema, capability or host ownership guarantee.

**Independent originals.** Before the second admission await, the Handoff saves
one private publication custody containing its actual capacity, SAME plan and
original origin. A typed stage records no transaction attempted, transaction
attempted/uncertain, committed/cache pending, known marker, or handed off. The
actual producer advances the stage; error text and persisted rows cannot
construct it. The private Store producer preserves the observed successful
commit in that independently retained cell before fallible Driver-cache
publication. This adds no arbitrary callback or permission exemption. SQL
guards and mutations remain unchanged; the outcome API retains these facts
instead of reducing every boundary to one unqualified `anyhow` error.
This custody retains an actual unstarted-job reservation, publication retention
and first genuine marker as they become available. It has no strong Runtime,
Handoff registry, own handle or return edge to its owning Handoff. Do not put a
strong retention back inside its own queue slot. No custody/queue/job mutex spans
Store access, readonly planning, child I/O or an await.

**Pre-latch order.** Under SAME control admission, validate the original slot,
origin and plan, install its SAME plan, and reserve the actual unstarted job and
EMPTY preparation custody before changing Unmarked to Publishing. Save that
original retention synchronously before entering Store. If any earlier step
fails, preserve its actual stage and assets, without falsely entering Publishing
or deleting a previously existing job. Apply this order to the test-only
retained-publication port as well. Narrow the unused Runtime publication wrapper
to its real scope; future reconciliation still requires the saved originals.

**Actual outcomes and rollback.** Publication distinguishes its observed
precommit, commit-attempt and postcommit-cache boundaries. A known precommit
refusal permits an attempt at the existing same-original unpublished protocol;
it is not itself a rollback proof. `plan_unpublished_marker` uses the retained
allocation's original Unit snapshot. Its successfully ended Immediate checks
SAME owner/epoch/generation, exact original open Unit and absence of the allocated
operation/pair/Session/invocation. Goal-row drift alone is not one of those
predicates. Only that private proof can restore the SAME Unmarked capacity.
Remove only its genuine unstarted job; preserve partial rollback/removal failures.
The restored accepted Source and its stale plan remain Held. This does not refresh
pins, reoffer Source, replay publication, authorize a new dispatch, settle a Task
or prove process death. Commit uncertainty, Unit drift, known marker or Native
history never becomes rollback from a label. They retain originals for exact
postimage confirmation, with no replacement plan or fresh start.

**Known marker.** Save the first returned genuine marker Arc in independent
original custody in the same synchronous return path, before fallible origin,
queue, job or preparation lookup. Poison recovery is allowed only to preserve
this factual object, never to authorize handoff. All existing original/current/
Driver/Unit/permission checks and the once-only launch CAS still precede start.
The queue preserves that first marker when confirmation revisits its SAME plan.

**Accepted-Unmarked shutdown.** The actual transfer mechanically preserves its
original slot/armed guard under accepted custody before `finish_accepted`, under
the first control admission. This cannot rely only on subsequent origin
installation, which can fail. Shutdown may drain never-accepted legacy Unmarked
slots, but retains accepted-Unmarked, Publishing and marked slots pending genuine
closure and reports that pending state. Driver stop, guard Drop, missing rows or
a shutdown error does not release that responsibility or declare completion.
Runtime/all-owner loss retains the documented uncertainty; recovery is a separate
required MVP gate, not supplied by this amendment.

**Actual consumers and qualification.** Remove the unused Handoff invocation
receiver; PhaseJobs already owns the launch and results. Engine's handoff state
is a transfer diagnostic and stays Waiting, not an Agent success or complete
launch observer. Fix the master binder paragraph to distinguish the implemented
private Task-preserving binder from ordinary legacy Task-writing binding.
Inspect all publication/retention callers and the accepted-slot shutdown consumer
when implementing these changes. Required genuine controls cover Goal drift plus
original unpublished rollback, failure before Publishing, known-marker lookup
failure, and shutdown before/after Publishing, with causal compiled omissions.
EMPTY graph evidence does not qualify them. Composition, full native preparation,
normal/late binding, terminal recovery, lint/regression, real Agents and four-Task
both-OS validation remain required before MVP completion or merge.
