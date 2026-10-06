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

The marker consumes the actual current Driver and its final pre-marker handoff,
selected NativeAllocation and Root's ManagedMarkerPlan in ONE Immediate transaction.
It reserves actual project/capacity and mandatory ledger allowance and writes the
already planned post-marker Task/W/operation/owner/input/audit facts. The Driver's
original marker anchor and Source7 current anchor are installed/checked in that
same transaction from their validated pre-state. No extra binder write is introduced.
OriginalMarker is issued only after known commit; actual retained phase supervisor
then receives PhaseLaunch. The worker's job set retains that supervisor before an
Engine/start future can be dropped. B's real selected protocol alone registers
Session, consumes input and produces transport/terminal facts.

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
- Real fair distinct-Task capacity and Native quota admission, retained due/held
  processing and at least four Tasks/two Projects remain required acceptance.
  Existing configured-cap generalization/native profile gates are not waived.

Independent exact design/source review, default-parallel full regressions, old
cached-writer/migration controls and actual Native/CLI qualification remain required.
This initial design commit runs no tests/helpers/native accounts and changes no
production, schema, main, license or README. Public scalar IDs/JSON/test switches
never issue the proposed authority. All existing failed Runtime fixture matrices
and unavailable native/MVP gates remain explicitly open.
