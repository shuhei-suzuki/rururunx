# Issue43: actual retained Runtime Driver producer

Status: proposed STRICT integration supplement, design only. This chooses the
missing producer/advance seams of the approved [managed binding design](issue-43-managed-binding-design.md)
§2.1–2.2/5.3/7 and [Runtime operational design](runtime-operational-integration-design.md)
§4–5. It changes no requirements, makes no Native readiness claim, and supplies
no production constructor at this commit. Root's marker/binder and B's selected
Native protocol remain separate actual producers. Schema mechanics are not a
Driver, composition, input, settlement or result credential.

Fixed inspected source: `20478c77d3dab2a9524021b335a5abbb1b46576c`, based on
`c1d65f581f3c3f55596d8007c6f6af988f0c4bd2`. This local development ancestry
contains Runtime9, not a release of it. Binding10 mechanics corrective candidate
`4713fc80903e35dd76348a19d3ba7ae928f0f1ac` is separate and immutable;
this supplement does not change its files or certify its guards. Its independent
follow-up review remains unapproved: an earlier same-version10 layout can lack the
new native identity trigger. Exact current-layout validation/refusal or a narrowly
verified atomic upgrade is a separate required correction, not an 11 allocation.

## 1. Actual missing connections

| Fixed source | Current behavior / selected change |
| --- | --- |
| `runtime/mod.rs:16–36`, `runtime/driver.rs:13–56` | Registry is retained/attached, but has no insertion producer. Add private retained worker ownership and registration; never populate it from SQL or a caller ID. |
| `runtime/service.rs:35–71` | Existing loop reconciles attention only. Keep that loop/readonly controls; add genuinely retained Task worker jobs that call the actual Sources/Engine, rather than relabel this loop as a Driver. |
| `state/runtime/driver.rs::snapshot/validate` | Existing validation requires an already live row and complete pinned content. Add a distinct initial claim transaction, followed by exact private registration, and typed pre-marker advance. Initial claim must not call this existing-driver validator. |
| `AttemptManager::prepare_workflow_source` → `state/execution.rs:710–748` | First Unit reservation calls existing Driver validation before helpers and fails without a claim. Initial Driver must exist and be live before this path; no unregistered Git or bypass of validation. |
| `state/execution.rs:850–893`, `adopt_prepared_workflow_execution` | Actual reservation/adoption writes Unit/Task/Workflow. Couple these genuine writes to pre/post Driver and applicable Source7 plans in that same transaction. |
| `state/mod.rs:541–565,930`, `execution/workflow_source.rs:192–248` | Workflow initialize/transitions and source preparation consume current ownership but cannot advance a Driver binding. Provide private exact transition plans; generic writers still cannot ratify changed authority. |
| `state/execution/source_recovery.rs:330–368` | Existing Source7 whole-row before/after pins are source-only. Driver-bound preparation uses one prevalidated combined plan; after marker, ordinary after_write cannot refresh either authority. |
| `state/managed_binding/marker_plan.rs` | Actual allocation/read plan is nongrant. Final marker transaction must consume the genuine current Driver, preserve the exact planned frame, and freeze its source/Driver anchors. |

For an accepted Goal, the current initial reserve fails with missing task_drivers
row; inventing a row would then fail the actual live registry callback. Neither
failure is a reason to remove either fence. No existing authenticated/native or
empty-registry control proves these new connections.

## 2. Non-circular initial claim and real worker registration

Private proposed types are non-Clone/non-Deserialize with inaccessible fields:
`PendingDriverClaim`, `RetainedTaskDriver`, `DriverReadTicket`,
`PreMarkerAdvancePlan` and `CommittedDriverAdvance`. These names specify new
implementation seams, not callable APIs at the inspected baseline. Readable IDs,
JSON, diagnostics, a JoinHandle alone or a named callback cannot construct them.

1. Runtime's retained service checks actual installed managed composition BEFORE
   claim, Source prepare, Unit/resource reservation or any helper. This static
   composition proves that the real selected Native protocol, retained supervisor,
   marker/binder, Driver implementation and writer guards are wired; it does not
   require a particular Task's Driver already to exist. Per-Task current Driver
   validation is a later dynamic check. Thus missing implementation still returns
   NativeBindingUnavailable with no writes/effects; installed producer availability
   is not synthesized from capability/configuration, SQL or a runtime test switch.
2. Store plans current accepted definition/policy, full P/G/T and their indexed
   identity/versions, exact zero-or-one Workflow/latest Context, execution
   generation, complete prerequisite/lock/conflict inventory and current epoch.
   Root's bounded coherent read/canonical planning can be reused. Readiness is
   only an expected plan, not permission. New source bootstrap requires exactly
   no prior Workflow/Context/prepared/effect history requiring recovery; existing
   Published history selects the explicit Source7 route. Ready/unknown/pre-artifact
   history is not treated as fresh. Missing/overflow/malformed evidence holds.
3. A private `claim_initial_driver` Immediate transaction rechecks that plan,
   current accepted Running Goal/Registered Project, service instance/epoch,
   scheduler rank and the actual distinct-Task Project/capacity union, plus no
   competing live/held claim. It inserts the initial Driver row and bounded audit
   and advances fairness only on successful commit. The private registry reserves
   a pending slot belonging to this service, but `is_current` is FALSE while
   pending. This special initial transaction does not need an existing Driver,
   Unit, source capability, Native allocation or Agent success. It cannot authorize
   any helper, and its durable row alone is never liveness.
4. Only after known claim commit does the retained Runtime spawn its actual Task
   worker. Runtime owns the JoinHandle, cancellation channel and pending claim
   independently of the control request/Engine caller. The worker is the actual
   serialized Sources/Engine dispatcher; its first action waits at private startup
   registration, before Sources/Engine/effects. It acknowledges the exact claim
   through an internal nonserializable channel and a retained lifetime guard. A
   detached empty task, user-supplied worker ID or is_finished=false alone is not
   this producer. Runtime must retain and observe that genuine task; its exit/panic
   revokes association even if user/control futures have disappeared.
5. Private `activate_retained_driver` verifies that exact startup acknowledgement,
   current unchanged committed row/full original plan and retained service/worker.
   Under Runtime control-admission serialization and SharedStore exclusion, it
   publishes the exact registry binding only after current read/commit is known.
   Registry activation is the last step before releasing the admission boundary;
   no helper is sent until it succeeds. Shutdown/stop ordering prevents an earlier
   stopping decision from racing a later activation. The row stays driving, but
   actual liveness requires both row and this live exact binding.

Commit uncertainty only reconciles this SAME pending claim's exact planned row,
ID/version/body/audit; it never claims a new UUID or accepts a fresh snapshot.
Confirmed rollback/spawn failure closes its own still-no-effect claim under exact
CAS. Cancellation before activation revokes the pending slot and invalidates that
claim; failure/uncertainty retains bounded attention. It does not certify Native
work or release an existing unrelated operation. Row replacement from parked or
invalid history requires a new genuine admission and proof that no old current
worker/effect remains; unsupported recovery stays held. No claim TTL steals it. Initial claim/advance/invalidating row writes use the
actual private Store producer and exact mutation guards, not a public row-writing
API. Their task_drivers protection and strict optional preparation/source metadata
are part of the composed Binding10 contract; same-version cached binaries and
missing/mismatched guard layouts must refuse before effects. This design does not
assume that the current schema primitive candidate already implements that port.

## 3. Pre-marker advance and actual helper currency

The worker owns one serialized task-invocation lane. A private read ticket retains
Driver ID/epoch/exact version/full encoded row, plus original full authority pins
and exact Unit/preparation/source binding when present. Helper admission, periodic
fence and receipt closure validate this SAME ticket and applicable SourceReadBinding
inside their existing transactions. An await never recaptures newer authority to
make itself current. Outstanding helpers/queued continuations are tracked by the
actual worker; conflicting pin advance waits for genuine operation retirement or
revokes admission without inventing settlement.

`PreMarkerAdvancePlan` is produced only by the actual designated transition, from
one coherent original snapshot and its concrete bounded write plan. The permit is
an exact database mutation permission, not a grant or a body-hash credential.
Inside Immediate the consumer validates the old live binding, complete original
P/G/T/Workflow/Context/Unit/index/prerequisite/lock pins, source frame and operation
identity. It checks exact allowed old→new projection BEFORE writes. Afterwards it
asserts the planned indexed metadata/full resulting bytes and updates Driver and
applicable Source7 pins in that same transaction. It returns the precomputed exact
new binding only after known commit; never re-authenticates arbitrary current rows
through generic put_record/after_write. Source and Driver share one old validation,
not one authorizing the other's already advanced snapshot.

| Producing operation before original marker | Permitted resulting change |
| --- | --- |
| First registered source Unit reservation | Actual new Unit/task_execution generation and prescribed Task worktree/branch/version/time; Driver records exact generated Unit and resulting Task; no Workflow/Context invented. |
| Registered preparation state/base qualification | Exact same Unit identity, checked consecutive version and allowed Reserved→Preparing/base binding; actual helper receipts remain separately owned. Complete governing P/G pins never change. |
| Actual Sources frame accepted + Workflow initialize | Consume the genuine retained initial preparation/frame; prescribed Task and new sole Workflow/Context/pointer fields. Atomically store exact resulting Driver pins/frame/Unit reference; Source map install is serialized and must validate the committed result before use. |
| Exact Workflow reserve/initial Unit adoption | Same prepared Unit and real single-use adoption proof; actual phase/provider/unit reference and prescribed Task/W/Context changes, all existing namespace/budget/risk/claim checks retained. |
| Published Source7 begin/registered read/accept | Genuine current Driver-bound source recovery, exact Published artifact/complete graph/frame and original snapshot; no old preparation/native handle recreated. Accept installs corresponding exact Driver/source pins atomically before serialized map installation. |

The strict bounded Driver row extends its typed private pins with optional actual
preparation Unit/reference/frame and Source7 claim identity, deriving them only in
these producing transactions. Complete original body bytes/indexes stay available
for CAS; all hash/complete read planning occurs outside held Store. Existing
128-KiB Driver metadata and declared full-owner/body/inventory bounds remain; an
oversize plan refuses before effects. Generated field/row counts are finite.

A generic same-Task write, new governing rules/accepted definition, Project/Goal
version drift, foreign generation/provider, changed instruction/source, arbitrary
Unit update or Context head cannot be ratified as this bookkeeping. None of these
ports allows P/G refresh or cross-Task pin advancement. Registered helper effect
rows that do not change pinned bodies need no Driver refresh; their actual grants
still validate the old ticket. Nongrant historical cleanup/facts are not denied
merely because Driver proof is absent.

After each known pre-marker commit, SharedStore remains excluded while the private
registry replaces precisely old binding with precisely preplanned new binding.
No registry mutex is held while acquiring SharedStore; SQL callbacks may acquire
registry/binding mutexes only for short pure equality checks. Actor/control paths
never hold those mutexes over DB/FS/await. Before publication the old binding fails
against the committed new row, so the gap is fail-closed. On publication failure or
uncertain commit the worker retains both exact plans, refuses effects and reconciles
only its own expected outcome; a currently different row is never adopted. Cancel
revocation wins through the same service admission boundary. Callback poison/loss
returns false. The retained worker, not a caller Drop, owns reconciliation.

## 4. Marker freeze and Root/B composition

Before any marker publication, the actual worker transfers its selected
`NativeAllocation` and armed `PreparationGuard` to the same Runtime's
`reserve_pending_phase`. The real bounded supervisor retains that operation's
allocation and guard and returns its private `PendingPhaseCapacity`. A full,
closed or mismatched queue returns the original objects; no marker Task/W,
operation, owner/input or audit write is allowed. Initial distinct-Task Driver
admission is a separate capacity contract and cannot replace this phase slot.

The worker borrows the original allocation from that retained slot when producing
Root's `ManagedMarkerPlan`. The marker consumes the actual current Driver/final
pre-marker handoff, that exact original allocation, the plan and the SAME genuine
pending-capacity handoff in ONE Immediate transaction. It validates the live
supervisor and exact retained slot/operation identity and preserves its retention
across the transaction; an ID, observation or named capacity DTO cannot substitute.
Its separate durable Project/distinct-Task union and mandatory ledger allowance
checks remain required. Native quota remains separately enforced by its actual
admission producer, including genuine waiting; a pending slot does not bypass it.
The marker writes the already planned post-marker
Task/W/operation/owner/input/audit facts. The Driver's
original marker anchor and Source7 current anchor are installed/checked in that
same transaction from their validated pre-state. No extra binder write is introduced.
OriginalMarker is issued only after known commit; PhaseLaunch carries the SAME
retained capacity slot, original allocation, prepared pair and preparation guard
through the actual supervisor's single handoff. Marker rollback leaves that slot
unmarked for its exact owner to abandon or retry. An uncertain commit, cancelled
caller or failed post-commit handoff retains the same slot and original plans for
owned reconciliation; it never reallocates by ID or calls unmarked abandonment on
a possibly marked operation. Stop is ordered at the same Runtime admission
boundary, so it cannot drain a marker-protected slot as unmarked. The worker's job
set retains that supervisor before an Engine/start future can be dropped. B's real
selected protocol alone registers Session, consumes input and produces
transport/terminal facts.

After marker, original P/G/T/Context/source/Unit selection/full locks and marker_W
are immutable. Root's genuine CurrentWorkflowSuccessor validates only enumerated
bounded factual W/audit successors against that anchor; bind/diagnostic never calls
ordinary Source7 after_write or updates Driver/source rows. This supplement does
not approve late body refresh or make a new row match the original by definition.
New marker-time Driver registration/anchor writes belong to the marker transaction,
not the record-only binder. Any further privileged writer needs the corresponding
actual source/terminal/closure producer; incomplete ports remain unavailable.

## 5. Drop, stop, restart and verification gates

Control/Engine future Drop does not drop the Runtime-owned Driver job or any real
phase supervisor. Worker Drop/panic closes admission through its lifetime guard,
revokes live association, and causes exact owned stop/fence/held reconciliation;
no terminal label/dead task alone proves external/native cleanup. Service shutdown
retains and waits its actual handles with finite deadlines. Timeout/caller Drop
keeps them retained and reports pending; cancel closes new grants independently of
factual result finalization/retained cleanup. Existing provider-specific DENY and
factual observation remain separately gated. Sibling Tasks/Goals are preserved.

A new epoch invalidates every old live association. Durable Driver/Unit/Source7
rows never reconstruct a live worker, OriginalMarker, preparation or Native owner.
Only fresh legitimate admission for an actually eligible source path can make a
new Driver. Published-source reconstruction verifies the complete actual graph;
old unknown/native/held operation recovery still requires its real producer and
cannot silently fall back to fresh bootstrap. Section5.2 pre-artifact recovery and
authentic operation restore remain explicit unsupported gates.

Before enabling production composition, actual account-free consumers must prove:

- Accepted trusted Goal → initial claim → real retained worker acknowledgement →
  registered source Unit/Git → initialized exact frame → same Unit adoption →
  actual marker; no SQL-seeded private owner/Driver or fabricated Agent success.
- The claim can be created without prior Driver/Unit; no helper before activation;
  no-spawn/failed spawn/stop-before-ack/caller Drop/worker panic/poison/uncertain
  commit each preserves ownership and forbids new effects, without invalid joins.
- Every listed pre-marker Task/W/Context/Unit/frame advance succeeds only via its
  genuine producer and exact post-plan. Same-version/body/index/definition/source/
  generation drift and arbitrary generic write fail; omit advance or omit original
  CAS variants reach intended actual consumer failures.
- A helper paused across revocation or forbidden advance cannot publish/grant;
  known-current completed read remains valid. Driver/Source7/mapping races and
  Published restart use the real Sources consumer and preserve siblings/history.
- Marker freezes exact originals; bind/diagnostic succeeds via genuine successor
  with W+one audit only, no third authority-row write. Raw drift cannot be refreshed.
- A genuinely full/stopped pending supervisor refuses BEFORE marker publication,
  returning the same allocation/guard and leaving Task/W/operation/audit unchanged.
  Marker rollback, uncertain commit, caller cancellation and handoff failure keep
  the same real slot under its appropriate unmarked/possibly-marked ownership;
  no slot is replaced or lost between commit and PhaseLaunch handoff.
- Real fair distinct-Task capacity and Native quota admission, retained due/held
  processing and at least four Tasks/two Projects remain required acceptance.
  Existing configured-cap generalization/native profile gates are not waived.

Independent exact design/source review, default-parallel full regressions, old
cached-writer/migration controls and actual Native/CLI qualification remain required.
This initial design commit runs no tests/helpers/native accounts and changes no
production, schema, main, license or README. Public scalar IDs/JSON/test switches
never issue the proposed authority. All existing failed Runtime fixture matrices
and unavailable native/MVP gates remain explicitly open.

## Finite preparation implementation correction

The first-preparation, Reserved→Preparing and initial base-OID advances retain the
same `Arc<DriverPreparationAdvance>` in the actual DriverSlot before entering SQL.
There is at most one pending advance per slot. A not-yet-attempted plan is not
eligible for rollback observation while its worker is live: actual Store-call
return/unwind or actual WorkerLifetime Drop opens reconciliation. The service
rotates bounded pages of 64 actual retained plans; registry capacity remains 4096.
No registry/custody lock is held across acquiring SharedStore, FS or await. A
PreparationGuard whose same advance is pending does not rewrite its Unit to Lost
and thereby destroy the captured post image. Other existing guard behavior stays
in place. Poison, changed rows, stop/revocation and uncertain post publication
retain the plan; neither a generic Err nor a terminal label retires it.

Reconciliation uses the original plan, never a fresh ticket/current-row image.
It may publish the exact captured post rows to the same live association, or prove
this plan's complete original own rows and new-Unit absence still hold and retire
only this plan's custody. Neither path releases any other effects, resource lease,
claim or native work, and stopped post-image publication cannot reactivate a
worker. Retained stopped/uncertain plans can keep their actual owner/registry
alive; authentic recovery remains necessary rather than silently dropping them.

The initial Task namespace inventory is planned in the coherent read snapshot
outside SharedStore before any body copy/decode: at most 4096 Task rows, at most
1MiB encoded bytes per Task and at most 16MiB cumulative encoded Task bytes per
Project. The header pass charges every complete length, refusing surplus rather
than truncating. It validates ID, indexed Project/Goal/version/issue and body,
plus paired path/branch. Inside Immediate, complete row count and each original
indexed identity/full body must still match; the same parsed inventory supplies
all existing path/branch collision checks. This replaces only the managed initial
preparation's unbounded foreign-Task decode loop. Other inherited Task writers
remain unchanged. Own/prerequisite/namespace profiles are separate finite costs;
this does not claim a mutex latency or physical-memory guarantee.

The OriginalMarker-owned `DriverMarkerAdvance::validate_live_tx` validates the
same actual association and captured post Driver/Source bytes/current epoch in
Root's transaction. Root must conjunct its original-derived Workflow successor,
current Unit/pair and lifecycle/effect checks. Workflow-only binding/diagnostic
links do not advance Driver or Source7 and therefore cannot refresh these pins.
This validator is nongrant and cannot be reconstructed from SQL/DTO/PhaseSlot.
The genuine installed composition issuer, initialized Context/adoption lane and
positive Driver cancellation/uncertain-commit controls remain qualification gates;
historical namespace controls do not manufacture their authority.


## Initial input implementation milestone (availability remains closed)

The actual retained dispatcher now calls `Engine::initialize_driven`, which
requires the Engine's installed Sources Arc to be the same producer as its
composition. Existing committed configuration/rules, risk escalation, phases,
mandatory input selection and Context budget construction remain the consumers.
A private `InitialInputFrame` retains the actual Sources slot/frame and armed
`PreparedExecutor`; it compares the complete rendered payload/dependencies and
Context metadata before SharedStore. The corresponding plan is an InitialInput
variant of the existing retained `DriverPreparationAdvance`, rather than a
second registry or worker. It is placed in the same actual slot before SQL or
another await, and retains its genuine preparation through uncertain commit,
cache publication failure and dispatcher Drop.

The same existing Workflow activation transaction validates original owner,
Task, Workflow/Context absence, Unit/generation, prerequisites, Source7 absence,
complete scoped locks and actual association. It preserves the ordinary
verification activation/lifecycle/transition guards and collision checks, using
an off-Store bounded complete namespace image. Exact precomputed Task/Workflow
versions/timestamps/body bytes and the first Context are written together with
the exact private Driver mutation. Only known commit plus those same post-images
can publish the cached Driver binding. Reconciliation reuses the same retained
plan and may prove exact original rollback; neither outcome recreates authority
from rows. Original P/G/rules/config/governing/artifact pins are not refreshed.
Existing original/marker projection readers retain their old Context semantics;
a separate sealed initial projection checks only original-none to first Context.

This first publication lane explicitly refuses prior Workflow/Context or an
installed Source7 recovery. Subsequent non-native phase transitions, reservation,
same-Unit adoption and complete marker/native composition remain prerequisites.
`InstalledDriverComposition` still has no successful issuer. Temporary historical
projection test images qualify nongrant exact-read checks only, never a Driver,
prepared input, native lifecycle or end-to-end positive dispatcher. No current
code milestone alone qualifies Runtime/Binding43/Phase2/native/MVP readiness.

### Initial Evidence continuation source increment

The real worker next calls `WorkflowEngine::step_driven_initial`, without falling
back to ordinary `step`/`persist`. The unchanged actual worker association mints
one coherent ticket per authorized edge. It carries the same original full
P/G/T/Workflow/Context/locks, same bootstrap Unit and actual Sources slot/frame.
Only Issue/Worktree before any native history are supported: Running reservation
with its next Context, Evaluating claim with unchanged Context, then an atomic
actual result observation plus Passed closure/next Context or Waiting. Each edge
is retained as the SAME `DriverPreparationAdvance` in the actual Driver slot
before SQL; it uses existing exact-post publication or proven original rollback
on error, cancellation, uncertain commit or cache failure. No generic authority
write is accepted as a new input or current Driver binding.

The private `InitialGateCompletion` is constructed only by the concrete
ManagedWorkflowGates after its existing claim, registered namespace/source and
receipt checks. A public GateOutcome, receipt identifier or JSON cannot create
it. Its receipt's complete scoped/versioned bytes remain part of the closure CAS.
Passed observation and closure are fused in one Immediate transaction; the exact
sealed virtual observer image is supplied only to the existing transition
validator, and ordinary observer/writer semantics are unchanged. Driver/Task/W
post images are precomputed outside Store and preserve original governing and
Unit pins. No native work/result or review certificate is inferred from an
initial preparation gate. Interrupted Evaluating and Waiting do not replay a
helper or reinterpret a receipt as the lost private completion.

Native first-Executor reservation/adoption and later evidence/result/lifecycle
continuations remain unavailable, as do genuine initial Driver issuer/startup
and full producer-backed positive qualification. The existing composition issuer
and native preflight remain closed. Nongrant complete Context projection controls
use rollback-only factual images; they do not seed Driver/Unit/input authority.
