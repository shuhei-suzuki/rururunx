# Issue 19 design: typed working context with explicit authority

Use a `context_pack` library service backed by existing ContextVersion envelopes
and scoped Checkpoint records. Pack data is typed and bounded; reads verify envelope
scope/version/digest and exact pointer ownership before trusting it. Context bodies
reference authoritative files by owned scope/path/SHA256, not replacement summaries.

Task authority projects only meaningful Goal/Task/Project fields. Database revision,
timestamps and context pointers are CAS guards, not semantic inputs: publishing
v1 must leave v1 usable. Native repository authority comes from Issue 18's bounded
FD/Git snapshot and content hashes; HEAD alone is insufficient. Rebuild outside
SharedStore, compare authority, then use atomic publication/current-state guards.

Packing places the full typed mandatory pack frame before a budgeted repository
slice. Standalone budgets cap the complete rendered payload. In the actual Workflow
port, `ContextBudget.discretionary_tokens` bounds optional repository sections;
mandatory Task/Goal/checkpoint metadata and Engine-owned rules are separate
overhead. The durable artifact records mandatory/optional UTF-8 byte estimates,
and the final native payload counts both plus rules exactly once. A 1 MiB absolute
rendered phase-input cap rejects oversized mandatory state before publication.
Provider measurements remain null. Rules and explicit evidence/expansions remain
complete or return a budget failure. Stored
packs keep compact metadata/references, not full source payloads. Native input
preparation re-fetches current source through the selector and never trusts a
serialized caller-provided repository map.

Goal packs contain all bounded owned Task descriptors, including Tasks not yet in
the DAG; exact Task membership and versions are rechecked atomically. They contain and exact pack version/digest refs,
DAG edges, cross-Task facts and aggregate metrics. They do not embed Task packs,
source slices or Task chat. The exact registered primary root anchors read-only rule/config and explicit Goal
artifact observation before Tasks exist and after their worktrees are disposed.
Goal descriptors validate exact immutable Task envelopes/pointers, and label active
Task refs as requiring source validation. A Goal summary remains available during
executor edits; Task preparation separately rejects stale owned sources. Terminal
Task refs retain immutable provenance, are marked historical and never launch. No dummy
Task or protected-root execution is authorized by Goal observation.

Deterministic condensation operates on typed event classifications. Semantic events
are retained with exact sequence/provenance. No semantic deduplication or reset
operation is implemented; a Task chain is bounded to 4096 retained events and a
1 MiB checkpoint. Reaching either limit explicitly stops checkpointing until work
is decomposed into a new Task; it never erases prior constraints to continue. Only explicitly transient notes can leave the recent
window. This strategy cannot infer safety meaning from arbitrary unclassified chat;
callers must normalize native events honestly. Mandatory state overflow is explicit,
not silent truncation. Checkpoints record input digest, consumed sequence range,
retained facts and bounded verbatim tail, with audit metadata excluding text bodies.
Existing checkpoint authority and previously retained semantic facts must survive
incremental checkpointing even after they leave the recent window.

Checkpoint append may observe a running owned Session but never changes its launch
pack or attempt identity. Standalone pack publication rejects live/Lost sessions,
locks and workflow-owned Tasks atomically; Workflow Engine owns phase publication.
Consultant checkpoints promote only within the same Project/Goal after source
reference validation, retaining historical source Session/Task/HEAD provenance.
Target rules/constraints and source hashes are rebuilt; no full chat is copied.

Keep Record kinds unchanged. An atomic ordered 3→4 migration fences older writers from modifying typed
checkpoint authority and installs an atomically maintained indexed checkpoint head;
preserves earlier 1→2→3 migrations. The scoped primary-key head lookup joins one
immutable record and does not sort/decode checkpoint history during runtime. It
survives SQLite VACUUM, and index updates roll back with failed append/audit. Add
narrow Store transactions beside existing
helpers and coordinate their workflow fences with Issue 8. The PhaseContext port
uses `WorkflowPackSources` to capture a phase-stable source snapshot, then an
additive `WorkflowSources::pack_artifact` hook binds typed metadata to the exact
scope/HEAD/hash set/payload/phase/budget. Engine alone wraps and atomically publishes
ContextVersion/Task pointers and attempt lifecycle. A restarted provider restores
durable scoped facts/artifact references from the latest owned artifact; the own
checkpoint is always resolved freshly. Cleanup preserves the exact reserved
artifact and real disposal evidence after worktree removal, without recapture. Provider operations remain in adapters.

Validation uses isolated native repositories, restart/version idempotence,
same-length dirty/rule/config/artifact changes, two Projects with identical names,
Goal DAG refs, budget/mandatory preservation, incremental condensation, malformed
references, consultation provenance and atomic cross-connection/session fences.
Important guarantees receive detached mutation proof and immutable native review.

Typed checkpoint records are immutable through generic Store writes; only the
owned append transaction creates them. Promotion rederives exact compact content
from an exact immutable checkpoint. Same-Task preparation and publication require
the current chain head, including a final atomic preparation audit; source versions
always include its digest or the explicit `none` sentinel. The caller cannot omit
an existing own chain: drafts resolve its head automatically, and readers, final
audits and publication compare the complete optional head. Own history and the
separate `promoted_consultation` snapshot can coexist. Prepared native inputs carry
the head key for launch freshness. Cross-Task consultation promotion remains a fixed snapshot when
the source Task appends later checkpoints. Mixed
Executor/Consultant chains retain all source facts, while cross-Task promotion
selects only actual Consultant events. Omitted transient history records count,
first/last sequence and a rolling SHA256 commitment, without storing omitted text.
Checkpoints also preserve original Task criteria alongside mandatory Goal/rules.
Cross-Task promotion carries only source Task id/title/criteria, rather than its
runtime fields. Generic Context writes and pointer changes are fenced once a typed
pack owns the scope; readers require both the current pointer and latest version.
The v1 pack formats are new in this unreleased feature; intermediate branch payloads
are not a supported persisted format.

Draft preparation returns a source DTO without Adapter InputKind or native version;
only the Engine binds it to a persisted phase context. Terminal Task/Goal preparation
fails even if its worktree remains. Final pack preparation audits complete rendered
bytes/estimates separately from repository selection. Goal limits are 128 Tasks,
4096 DAG edges and 128 artifact/metric refs. Caller-supplied metrics are labeled
observations without provider provenance. Workflow phase/budget/risk decisions are
separate authority; the Engine compares exact HEAD as well as source hashes.

Explicit artifact/additional-source refs include durable lexical metadata (up to
8 symbols/imports each, with a truncation flag) and exact content digests; binary
sources retain only inventory/skipped metadata. Native preparation can select
broader file/symbol slices under its budget. Verification facts are caller-supplied
structured pack facts; authoritative verification files remain scoped digest refs.

Native reservation validates prepared `checkpoint:head` (including `none`) with
the scoped indexed head. Typed Issue 19 native inputs cannot omit that key; legacy
non-pack callers retain their existing contract. The same comparison applies to
first/new consumed dispatch-intent publication before the native wire, including
Starting→Starting, and Starting→Running acknowledgement, so an intervening
checkpoint prevents stale launch.
Existing admitted Running/terminal observations remain valid and preserve immutable active
attempt identity. `Store::validate_checkpoint_source` exposes the pure bounded
comparison for native adapters; it performs no Git/filesystem work.

Admitted native Sessions keep their immutable launch head through approval/human/
Lost state reentry. New reservations, new consumed dispatch intents and
Starting→Running publication compare the current head. Finalized frozen Cleanup binds its original historical head; a
checkpoint appended during Waiting Cleanup remains durable and cannot prevent
final persistence after actual disposal. Goal descriptors preserve exact opaque
legacy context envelopes with `typed_context=false`, without asserting typed pack
semantics. Provider configurations can be released with `clear_inputs`; configured
own references are validated then normalized to fresh current-head capture.

A bound owned worktree provisioned by WorktreeManager is a precondition before
Engine initialization, including STANDARD/STRICT workflows.
Checkpoint/fact changes remain conservative source-authority changes in the current
Engine: they can invalidate earlier generations/evidence, and changes after PR may
hold for explicit recovery. The implementation does not silently exclude newly
added constraints or facts from formal freshness. Lifecycle versus source-version
separation and post-PR reconciliation remain explicit Issue 23/13 integrations.

Checkpoint admission projects the complete mandatory Task/phase artifact plus
current captured rules before committing a new head. The checkpoint's 1 MiB
envelope limit is an upper bound, not a promise that all such envelopes fit a
rendered input: combined mandatory-state capacity can reject earlier. Existing
scoped facts/references are included when available. Rejection leaves the old
head/checkpoint-publication audit unchanged (source observation may be audited); it does not accept an unrenderable new mandatory history.
Later explicit input or rule changes can still require decomposition/extra budget.

New Cleanup evaluation claims atomically compare the live checkpoint head before
an external gate is invoked. A preclaim change preserves the worktree and requires
explicit recovery; a checkpoint arriving after the admitted claim is historical
evidence and does not prevent terminal persistence after actual disposal.

An admitted input's revision/version/byte count/source versions remain pinned
through all Session observations. An owned terminal→Starting continuation may
install a strictly higher input version after latest typed context/head validation.
It first binds `pre_dispatch_restore_sha256` to SHA256 of the exact serialized
prior terminal Session. Exact terminal restoration is allowed only before any
consumed dispatch intent/unobserved boundary; after dispatch, input rebinding or
rollback fails. Native adapters preserve this protocol without rewriting active
attempts. The checksum helper is pure and performs no Git/filesystem work.

Protected Sessions cannot bypass fresh continuation through terminal→Running/
Waiting/Lost or by creating an initial Waiting/Lost record. Every new admission
binds its declared input version and checkpoint head to the exact latest typed
ContextVersion and Task pointer. Historical restore proofs are installed only
from a persisted terminal Session and remain immutable until the next verified
fresh continuation; exact prewire restoration of a historical consumed intent
is restoration, not a new model dispatch. Pr/MergeGate/Cleanup all compare the
live checkpoint head atomically at each new irreversible evaluation claim.

Workflow checkpoint acceptance has a concrete liveness dependency: before external
effects, any new checkpoint source authority can restart the entire generation;
after PR/merge effects, an append may hold the Task until explicit Issue 13/23
reconciliation is available. This core demonstrates auditable condensation and
constraint preservation, but does not claim automatic post-PR recovery. A bound
owned worktree created by the existing WorktreeManager remains necessary before Engine initialization.
Cross-Task promotion copies Consultant-origin facts only; it preserves the target
Goal/Project/rule constraints and does not implicitly copy source Executor facts.

Ordered schema 4→5 adds private immutable prepared-frame authority for standalone
selection. Preparation publishes scope/context version/HEAD/source versions/UTF-8
byte count/lowerhex SHA256 of the exact complete PreparedInput.payload in the
same Immediate owner/source/head CAS and audit transaction. Up to 128 variants
per context version are retained; prompt bodies are not duplicated. Generic audit
or Record writes cannot fabricate this authority. Workflow authority uses its
immutable ContextVersion.data.payload, including mandatory Engine rule prefix.
Store::validate_context_input checks actual request bytes; Session admission pins
input_sha256 and consumed intent must match that hash. Provider-specific RPC
envelopes/fixed prefixes are a distinct transport digest, never this input hash.
Older public schema 4 writers refuse schema 5. Existing schema4 standalone packs
must be prepared again under the new private publication contract before launch.

Once the same dispatch intent is consumed before wire delivery, its Starting
status/Running acknowledgement is a historical admitted observation. Later
checkpoint updates cannot rewrite or wedge that acknowledgement. A changed/new
consumed intent remains a new admission and must compare live head/frame authority.

New native admissions of typed Workflow frames require the current active Running,
dispatch-started attempt with the exact context, phase and generation. Session
publication also binds its owned Session ID. Inactive owners, terminal Tasks,
frozen final packs and EvidencePort phases are non-launchable. Already-consumed
observations retain their historical input instead of re-admitting it. Stable
Project scoped references and Goal/Task instruction hashes are source authority;
Goal criterion satisfaction, DAG progress and raw row counters are bookkeeping.

Optional source sections use the smaller of the selected discretionary budget and
the remaining absolute 1 MiB capacity after mandatory pack metadata and Engine
rule bytes. Zero remaining optional bytes is valid; mandatory overflow still fails
closed. The selected phase budget remains unchanged and reported. New generic
Context writes reject reserved task_pack/frozen_task_pack envelopes. Readers
classify historical legacy envelopes by the explicit typed format. Pointer-only
Goal contention is a typed SnapshotChanged on goals.context_version. Reserved
preparation/index/selection audit events require their private producer paths.

Blocked-owner Lost updates may add only native_dispatch_unobserved=true and clear
PID while preserving exact actor, scope, input, restore proof and consumed intent.
This flag is conservative uncertainty, not a new dispatch admission. New
checkpoints record their configured transient window; historical checkpoints with
no recorded policy retain an explicit unknown value.

A successful native actor acknowledgement may refresh only its Project/Goal CAS
rows after sibling bookkeeping changes. It first verifies the exact unchanged
Task/Workflow versions, active attempt/context/generation and stable semantic
instruction hashes, with active lifecycle guards. Session ID publication then
uses the new CAS in the same existing atomic Workflow transition. Changed
constraints, scoped references, Task/attempt authority or paused owners remain
fenced; acknowledgement is not another model dispatch. Legacy sources retain
strict Project/Goal version equality.

## Private input admission and schema 6

This section specifies the new contract before its implementation. A private input
admission record is distinct from Engine actor acknowledgement, which binds a
returned native Session to its immutable Workflow attempt. Neither record proves
provider wire delivery; provider adapters retain their private payload-to-wire and
consumed-dispatch protocol.

A private `session_input_acks` table holds one indexed row per protected, Task-scoped
Session: Session ID, Project/Goal/Task IDs, optional admitted input_version/metadata
SHA256, and private validated-preparation input_version/metadata SHA256. Each pair
is both null or both present; present hashes are length64 lowerhex. Initial terminal
history does not create a row. A validated Starting may register the preparation
pair without asserting admission; only validated/consumed-historical Running sets
the admitted pair. This distinction permits verified prewire-failure continuation
without claiming model delivery. It references
the Session Record and owned Task; its helper derives all IDs from the validated
Session, never from caller-supplied ack metadata. Scope equality is checked against
the actual persisted Session and Task. Project/Goal-only Sessions and unprotected
legacy inputs receive no admission row. Generic Records, audit events and caller
JSON cannot create or update it.

Hash a domain-tagged typed tuple: exact Session ID, Scope, agent, provider, role,
worktree, model, effort, native_ref, and
input_version, input_revision, input_bytes, input_sha256 and source_versions. Parse
those input fields into their bounded types; source_versions is a sorted BTreeMap.
PID, Session state, per-turn response IDs and diagnostic fields are excluded.
Native session identity is distinct from per-turn response IDs. Do not hash
arbitrary JSON serialization order. Only the previous persisted Session's digest matching the admitted pair
can establish prior admission; a caller cannot install an ack by selecting fields
that match an unrelated row.

The launch guard returns a typed outcome: `Validated`, `ConsumedHistorical`,
`Acked`, `BoundHistorical`, or `NotAdmission`. A changed/new actually consumed dispatch_intent ALWAYS
validates the latest frame/head first, even if the same input has an ack. For other
admissions, actual consumed intent or a matching private row permits historical
observation of the same pinned input. Otherwise Starting/Running validates current
frame/head regardless of the previous waiting/Lost state. Waiting/Lost never writes
an ack. A new or changed Running input writes its admitted pair only after `Validated` or
`ConsumedHistorical`; `Acked` preserves the existing row. `BoundHistorical` is
the sole equal-version digest update: the persisted old digest must match the
private admission row, every input/scope/actor pin remains exact, and only the
initial monotonic native binding described below may change. It grants no new
dispatch and never makes an unadmitted input historical. Row, Session and audit
commit atomically, including rollback on a failing audit.

Keep two distinct predicates: actual consumed intent authorizes historical
observation; actual consumed intent OR native_dispatch_unobserved forbids input
rebinding/restoration. Uncertainty alone is never admission. Preserve the monotonic
uncertainty flag during same-attempt observations. An owned terminal fresh
continuation may clear historical diagnostics only while installing a strictly
higher input version, exact prior terminal checksum and freshly validated frame.

A nonterminal Running/Waiting/Lost Session cannot return to Starting. Same pending
Starting observations retain all input pins; only owned terminal-to-Starting is fresh.
A pending Starting-to-Starting write that initially binds model, effort or
native_ref must pass the monotonic None-to-Some predicate, revalidate the latest
complete private frame/head and owner lifecycle, and atomically replace the
validated-preparation digest with the digest of the new exact actor fields. It
never creates an admitted pair. A head/lifecycle change between the observations
rejects the binding; no unconditional pending digest refresh is allowed.
Exact prewire terminal restoration requires the stored previous-terminal checksum,
no consumed intent, no uncertainty, AND no private admission row matching the
pending input. An input once admitted as Running cannot be restored to an older
terminal input by visiting Starting. Previous historical ack rows can remain while
a higher pending input is prepared; their digest does not admit that new input.

Standalone prepared frames are Executor inputs only. Session admission requires
exact Task.executor, Executor role and Task.worktree as well as scoped frame bytes,
revision, version, hashes, owner lifecycle and existing lock/worktree gates. This
applies to initial Starting and Running. Workflow frames retain their exact active
attempt agent/role/Session fences AND require Session.worktree == Task.worktree
for every native actor, including Reviewer, on initial Starting and Running. No arbitrary Consultant/ApprovalReviewer role
can adopt an Executor frame. Existing scoped Consultant history may be condensed;
live consultation needs a separate prepared-frame port. ApprovalReviewer requires
an operation-free decision Task through the Approval Broker.

The single-actor Workflow port also allocates one private `phase_session_owners`
row per (Project, Goal, Task, context_version), containing Session ID. Context
version uniquely identifies the immutable native phase attempt. Every first valid
typed admission write allocates it atomically with Record/audit: INSERT or UPDATE
to Starting/Running, or first/new consumed dispatch. This includes a terminal
legacy Session updated to a fresh typed Starting. Later writes
must be by that same Session. Another Session cannot reserve or consume the frame
even before Engine binds attempt.session_id. Allocation is immutable across
terminal history, foreign keys bind the Session and Task, and generic writes have
no allocator API. A failed Session/audit transaction leaves no allocation. The atomic Workflow transition enforces that every newly appended attempt uses
an owned context_version strictly greater than every prior native Executor/Reviewer
attempt when appending a new native attempt; a None-context transition cannot
reuse an old version for a new native attempt. EvidencePort phases retain their
existing separately validated context contract. The same Workflow transition must
check any attempt.session_id binding or closure against the private allocation
in the same transaction. Whenever an allocation exists, closing/replacing the
active attempt requires that exact allocated Session to be persisted terminal,
regardless of whether attempt.session_id has been bound; Succeeded additionally
requires Exited AND current-attempt admitted/consumed input proof. Owner UUID and
terminal state alone never identify the input. Any session_id bind/verify must
match that Session's exact input version == attempt.context_version, payload SHA
and byte count == immutable ContextVersion.data.payload, revision and complete
source_versions == context authority, plus a matching private validated preparation
or admitted pair. Pending Starting preparation may bind an actor without asserting
delivery (Grok start returns before inference); Succeeded requires a matching
private ADMITTED pair or actual consumed intent for those exact input pins.
A Session restored to its older terminal snapshot after fresh prewire failure may
close this allocation only as Failed/Interrupted with explicit not-admitted owner
provenance. It cannot bind as the fresh input actor or satisfy Succeeded; retain
session_id=None when no current input binding exists. The private preparation row
for a failed pending input alone cannot certify delivery. Compare this proof under
the same Workflow transaction, including the unbound-owner case. A Running
Reviewer is operational ownership even though it is not an Executor reservation.
An unbound claim cannot close around it, and another terminal Session is never
release proof. Binding may update only the Workflow Record while retaining exact
Task/Project/Goal/Record CAS; this check does not require a Task version bump. Expose a pure scoped
allocation reader for explicit recovery. The recovery port must resolve an unbound
claim using this private owner, actual persisted terminal Session and verified
native cleanup; it must not invent another actor or clear Lost/uncertain ownership.
An absent allocation is factual proof that no schema6 typed Session/input was
admitted through Store for that exact context, because every such write allocates
atomically and older writers are fenced. It does not by itself invoke or broaden
Workflow release policy: ordinary observer polling remains read-only, and owner
release still requires its own exact claim token and applicable pre/post-marker
rules. Explicit recovery can use this absence to prove no typed admission, while
an allocated Starting/Running/Lost or uncertain owner remains held until genuine
terminal/cleanup evidence. Current unbound post-marker failure may remain held
until the explicit recovery port is integrated; this is an availability limit,
not a claim that absence means unknown model dispatch. No new automatic release
policy is introduced by the index.
Future
multi-reviewer rounds require their own reviewed slot authority; they cannot use
this single-actor allocation as a blanket role bypass.

### Semantic input inventory

Define projection version2 as a fixed-order typed struct, serialized as compact
UTF-8 JSON with a domain/version tag. Nested maps use recursively sorted keys;
paths are exact UTF-8 strings, Options encode explicit null, enums retain their
serde spelling. Reject nonrepresentable paths; never normalize a scoped identity
into another path. Exhaustively destructure domain structs when forming the
projection, so adding a field fails to compile until its authority classification
is reviewed. EVERY projection-authority change requires BOTH a projection version change AND
an ordered schema/writer-fence version change with reviewed drain policy. A
same-schema concurrent older projection writer must never continue admitting its
weaker view. Schema6 pairs exactly with projection2; a compile/test golden maps
that supported pair and fails when the projection changes without the corresponding
persistence/fence bump. Future versions install a new all-table required function,
so actual older open writers fail; changing only a key or reclassifying an existing
field is insufficient. They cannot silently redefine an existing source key.
New durable Task packs record `instruction_projection_version=2` (historical
packs decode a missing value as unknown, preserved without asserting v2 authority).
This marker makes migration-only consecutive republish eligibility observable;
it is not private launch authority by itself. All fresh validation and post-start semantic refresh require the runtime's single
current projection version2; old versions remain readable history, never a fresh
admission. Future authoritative additions cannot keep admitting old weak versions.
Exhaustive classification includes nested domain structs such as completion
criteria; any new nested authority field must be explicitly classified.
New frames use versioned instruction keys (`instruction:project.v2`,
`instruction:goal.v2`, `instruction:task.v2`) and explicit projection_version=2.

Project instructions include ID/root/identity/base/worktree namespace/config/rules/
environment refs. Project name and max_tasks are label/scheduler bookkeeping;
version/timestamps/state/blocked_reason are separately guarded lifecycle metadata.
Goal instructions include ID/scope/title/objective, criterion IDs/descriptions,
constraints/non_goals/source_refs. DAG, criterion satisfied/evidence, followup
proposal/disposition, blockers, version/timestamps/context pointer and active-state
progress are bookkeeping. An accepted followup's actual scope constraint must be
promoted explicitly into authoritative constraints or its own Task before launch;
a sibling proposal or disposition alone cannot strand input.
Task instructions include ID/scope/title/criteria/worktree/branch/executor/reviewers.
Issue linking is query-label bookkeeping, as in the existing Task writer. Raw
Engine-owned Task.artifacts is capture-status metadata; authoritative verification
artifacts remain separately scoped typed refs with physical content hashes. Phase/source-synchronized revision, workflow/
risk, blockers/next_action, lifecycle/pointers/versions/timestamps are separate
phase or captured status. Task.revision is verified against physical captured Git
HEAD and frame revision. Engine effective workflow/risk/budget is immutable phase
wrapper/attempt authority, not a phase-stable source hash.

For new admissions, require an explicit Task-state allowlist. Standalone admits
Created/Consulting/Planning/Implementing/Testing/Reviewing/Fixing, and excludes
WaitingHuman/WaitingApproval/WaitingReview, ReadyForPr/PrCreated and every terminal
state. Typed Workflow admission requires Task.state and Task.phase to equal the
immutable native phase's expected state/key, in addition to active attempt guards.
This is fresh input admission, not historical Acked/ConsumedHistorical observation.
An operator Task hold after Starting but before first Running/new consumption fences
the model input. Historical terminal/Lost diagnostics remain recordable.

For standalone admission, additionally bind workflow/risk and its OWN Task
blockers/next_action and standalone Task.artifacts in a versioned policy digest.
Unlike Engine-owned artifacts, standalone artifacts may carry direct operator
instructions. Thus a new
operator directive on that standalone Task fences its pending input. Sibling
Task status, Goal criterion satisfaction/DAG/followups and Project scheduling or
label changes remain usable. Phase packs describe Engine-owned hold/blocker state
as nonauthoritative status at capture; authoritative new stop/scope constraints
must use Goal.constraints/Task.acceptance_criteria or lifecycle pause, which are
always fenced. The rendered phase header labels capture status explicitly. No
status field can grant launch, approval, cleanup or other authority.

Standalone admission REPLACES the current full authority_digest/pack.task/pack.goal
projection-equality checks with exact equality of freshly derived version2
instruction/policy keys against the private frame source map. Its private frame
must still match exact scope/version/revision/bytes/SHA256/complete source map;
latest pointer, active lifecycle, physical/source revision and checkpoint head
checks remain. Full projections and authority_digest remain mandatory during
pack preparation/publication snapshot CAS, not pending-input admission.

Standalone preparation renders the canonical instruction and policy digest map in
its complete mandatory payload and privately publishes the same map with frame
SHA256. The private transactional publisher itself derives these keys from the
current typed owners and verifies the exact mandatory rendered header map against
the published source map; it does not trust a service-supplied map. Header parsing
is bounded, position-specific and format/version-tagged, never substring search.
The digest is therefore part of the immutable prepared bytes, preventing
same-payload old-schema collisions. Old pending inputs without these hashes fail
closed. Add an explicit forced consecutive Task pack publication option (default
idempotent reuse unchanged). It uses the existing idle/owner/source/head CAS and
audit, never changes an active launch, and provides a strictly higher context
version for terminal continuation or migration recovery. Permit forced publication only when the latest context is actually referenced by
an owned terminal Session whose exact pinned metadata matches its PRIVATE validated
preparation/admission pair, or lacks the required v2 instruction contract
for migration, OR the private prepared variant count for that current context
has reached its cap128. The cap-exhaustion branch performs the same idle/source/
head/owner CAS and starts a fresh bounded variant index without deleting history.
Otherwise identical reuse remains required. Private variants stay
bounded per context; unused identical force calls cannot create endless versions. Test recovery from an actual schema5 prepared frame through
terminal state, forced higher publication, preparation and fresh admission.

### Migration and already-open writers

Schema6 does not implement hot migration of active native work. Before ANY schema
mutation, under the same Immediate lock, inspect persisted scopes and REFUSE
migration if ANY Session in ANY scope is nonterminal (including Lost), any active
or malformed WorktreeLock remains, or any Workflow has active.is_some() OR owns a
nonterminal Task while unfinished (including idle, held, or post-PR phases). A
terminal Task plus active=None is non-owning historical Workflow even if finished
is false: existing explicit cancellation/release produces this state. Validate
history consistency and refuse contradictory/malformed ownership; never equate
Task terminality with release of an active claim. No PID inference or automatic release is allowed. The operator must finish
or explicitly cancel/drain using the compatible old runtime, retaining actual
native terminal/cleanup evidence, then retry. Lost or orphan ownership requires
verified recovery, not migration. The current Core has no general verified Lost
recovery port yet: an old database containing Lost ownership cannot upgrade until
that port is implemented or the compatible runtime produces genuine authoritative
terminal/cleanup evidence. Editing Lost to Stopped, relying on a dead PID, or
manually deleting a lock is not a supported upgrade procedure. Fresh drained
databases can use schema6 without claiming that recovery integration. An explicitly terminal Task's non-owning Workflow
history is preserved and nonlaunchable. Existing terminal Goal/project-only Consultant
history remains readable; live ownership in that scope must also drain. Cancel is a terminal disposition: interrupted Task history cannot be resumed
as that Task. Idle unfinished and post-effect Workflows are deliberately refused
too, although they may have no live actor; this upgrade does not reinterpret their
generation/source authority or move external PR effects to a new runtime. The
operator must complete them under the compatible runtime or explicitly accept
terminal cancellation and reconcile remaining external effects. No lossless idle
Workflow hot-upgrade is claimed. Apply the preflight to every
older supported schema path (v1 through v5), regardless of whether it contains
typed authority. Run it before any earlier migration in that same transaction,
not only before direct5-to-6. Decode ownership with source-version-aware minimal
validated JSON fields before the earlier schema migrations: bounded Session scope/
ID/state, lock scope/active and (v3+) Workflow active/history/finished with its Task
state. Require well-typed fields and consistent ownership; do not demand later
optional Project metadata from a v1 body. Golden legitimate v1–v5 drained fixtures
migrate and live/contradictory equivalents refuse.
Refusal rolls back without table/marker/audit changes and is tested on real old5
Running-plus-checkpoint and pending/post-effect Workflow fixtures. This avoids
both silently accepting old weaker projections and gratuitously invalidating an
in-flight post-effect generation. Existing unlaunched standalone frames require
explicit v2 preparation/republish; no model is already running to strand.

Ordered 5-to-6 migration creates the empty admission/allocation tables and write fences in one
transaction, preserving earlier migrations. Fresh databases install the identical
final schema. After successful preflight no nonterminal Session remains. Terminal history retains
its exact pins and receives no synthetic admission rows. A later permitted fresh
continuation must bind a new current frame. No old Running Session is grandfathered
through the preflight. Opening an unsupported version rejects before mutation.

Every v6 Store connection registers a private zero-argument SQLite function
`rrx_writer_v6()` returning 6 before migration or application writes. Add distinct
BEFORE INSERT/UPDATE/DELETE fence triggers to EVERY application table: projects,
goals, tasks, records, context_versions, usage, audit, checkpoint_heads,
prepared_pack_inputs and session_input_acks. Each requires this function to return
6. Register UTF8|DETERMINISTIC|INNOCUOUS, never DIRECTONLY, and verify
writes with trusted_schema=OFF. Add `phase_session_owners` to the fenced table
inventory. A schema-enumerating regression requires every non-sqlite_ application
table to have all three compatibility triggers; new tables cannot silently escape.
Preserve all existing immutable/append-only/domain triggers; fences only add
writer compatibility checks. Include private runtime metadata and audit writes,
not just Session updates. An already-open old5 connection cannot resolve the
function and its SQL write fails atomically after schema6 migration. Public Store
has no caller function registration or trigger-bypass API. Privileged arbitrary
SQLite schema editing is outside the Store contract. Future migrations must retain
required earlier functions and install their own exact-version write fence.

The actual old5 native fixture is compiled from immutable public source
`e6cf75dc61d0c9c9a6a225c64c8f9aaf7d6ffd26`, plus the latest schema5 production
source `79af00a603149729fe26bd1079aa4d49c1932cad` after the reviewed Grok/CAS merge,
never by patching the version constant in new schema6 source. This feature has
no deployed schema5 release; record exact source/binary identities rather than
inventing a deployment claim.
Save binary/source hashes and verify no writer-v6 registration in that source.
Prove both native old5 open refusal and a real compiled old5 Store held open BEFORE
migration, then released to attempt public writes AFTER migration. Test Session,
owner metadata and audit paths, including a previously prepared statement. All
must fail without changing logical rows/head/audit. Close all connections and
checkpoint/truncate WAL before comparing complete DB bytes, or compare the DB and
WAL pair; never claim equality from the main file while WAL remains live. New v6
writes and all existing domain triggers must still work. Independently opened v6
connections must pass registration and reopening tests.

### Boundary validation

Tests cover pending Starting-to-each-waiting/Lost-to-Running head rejection, unknown
flag forgery, caller JSON ack forgery, successful first Running then historical
reentry, forbidden Running-to-Starting-to-old-terminal rollback, exact prewire
restore, higher-version continuation, Session/ack/audit rollback and reopening.
Wrong standalone role/agent/worktree and wrong Workflow Reviewer worktree must
fail on initial Starting AND Running. Two independent Store connections racing to
reserve the same Reviewer attempt must yield exactly one Session allocation;
neither JSON nor prebinding session_id=None can admit a second actor.
Changed instructions/policy/ref hashes fail; sibling progress remains usable.
Typed Workflow-owned contexts cannot silently downgrade to opaque legacy payloads.
Important guards receive caller-level mutation proof and immutable source review.

The private ack row additionally stores input_version and checks SHA length64.
Its ADMITTED pair may be replaced only by Validated/ConsumedHistorical with a
strictly higher input_version; equal matching metadata is idempotent, unequal
equal-version admitted metadata is rejected, except the explicit initial native
binding transition below. The preparation pair follows only a
freshly validated Starting/Running input. It may be refreshed for a legitimate
new prewire continuation that restored an older terminal input; this never changes
the admitted pair or creates historical-observation authority. There is no plural ack history: while higher input is pending,
the single row still describes the previous admitted input until replacement.
Checkpoint format-v1 hashes remain BYTE-IDENTICAL to the existing
`serde_json::to_vec(Value)` compact UTF-8 encoding with its recursively sorted
serde_json map keys. Use an explicit recursive sorted-key encoder that produces
those exact bytes independently of serde_json feature unification, retaining
serde numeric/string encoding and array order; it is not RFC8785. Restore hashing
keeps typed Session outer serde field order and explicitly sorted nested Value
keys, producing the existing helper bytes even with preserve_order enabled. Do not apply RFC8785, normalize numbers, reorder arrays or
change existing hash prefixes. Golden persisted chain/ref fixtures prove old
checkpoint hashes still validate. New instruction/admission hashes use their own
versioned typed encoding. Keep the restore Session outer fixed serde field order
and nested serde_json map encoding byte-identical to the existing shared helper;
native adapters call that helper rather than arbitrary recovery insertion order. Payload SHA continues to hash exact actual input bytes,
not canonicalized JSON or transport envelopes.

This contract fences Issue19 typed native inputs; generic Store history writes are
not permission to launch a model. Project/Goal-only history records remain allowed
for existing callers, but Issue19 prepares no launchable Goal input or standalone
consultation/ApprovalReviewer input. Current native caller ownership validation
requires an actual Task/worktree. The future Broker decision-Task isolation remains
a required integration, not a claim that generic history records enforce it today.

Protection need not depend on a caller JSON flag. Prove stability with publication
fences: once typed, a Task context never downgrades to opaque; same-attempt source
pins are immutable; typed publication rejects any live/Lost Task Session lacking
its pinned checkpoint key. This includes Workflow ReadOnly publication, which
currently permits a live legacy Consultant. Thus a live legacy attempt cannot
become typed underneath it. After genuine terminal state, a fresh higher-input
continuation may explicitly bind new typed authority. Initial terminal history is
not a validated preparation/admission and cannot unlock forced republish. Test the
legacy Consultant to typed Workflow race using independent Store connections.

The upgrade procedure supports no live hot migration. Close/drain owners under
the compatible runtime, obtain a consistent backup with SQLite-aware backup or
closed/WAL-checkpointed files, then open with the new runtime. Refusal makes no
schema/state changes. Backup rollback is supported only immediately after migration and BEFORE any
post-upgrade application write, native dispatch, worktree/source mutation or
external PR/merge effect. It restores the complete consistent pre-upgrade state
while every writer remains closed. Once such an effect occurs, this document
authorizes no backup rollback: discarding its state would erase ownership/evidence
and strand external effects; explicit recovery is required. This is an operator-
attested manual procedure, not an enforced Store rollback API; Store exposes no
backup restore/downgrade method or assertion that the precondition is known.
Automatic database
copying/downgrade is outside this Store contract. Test cancelled terminal historical Workflow with active=None
as a successful migration, and terminal Task with active claim as refused.

Provider full owner-version/lock CAS remains independent of semantic frame checks.
Store frame usability after sibling bookkeeping is not a claim that every native
approval/grant survives a concurrent owner-version change. No lifecycle authority
or native cancellation boundary is weakened to provide that liveness.


### Initial native binding and consumer compatibility

Within each input attempt, model/effort/native_ref are actor metadata pins. A
pending input may initially bind model or effort from None to one bounded Some
value; explicit requested Some must remain exactly equal, and Some cannot change
or clear. Initial native_ref None may bind one owned UUID/reference; once known,
it is immutable for the Session UUID, including higher-input continuation. A
late None-to-Some observation on an already admitted input is permitted only by
BoundHistorical: exact old admitted digest, identical input and other actor pins,
monotonic initial binding, and atomic updated Session/admitted digest/audit. It
uses historical checkpoint authority, not a fresh launch exemption. A new
consumed intent still takes the ordinary latest-head validation path. Diagnostic
JSON cannot assert an admission or clear a binding.

A genuine terminal-to-Starting continuation may reset model/effort to the newly
requested values for its strictly higher input version. It retains known
native_ref and the exact prior terminal snapshot checksum. Pending native binding
then follows the same None-to-Some rule. Exact prewire restoration alone may
restore that original terminal Session, including old model/effort metadata; the
private preparation pair does not assert admission. Once dispatch/admission or
uncertainty occurs, restoration is denied.

Required coordinated Codex6 change: pin_starting_input resets Session.model/effort
to request.model/request.effort BEFORE initial Starting persistence, retaining the
exact original Session for rollback. Native thread/start or resume response may
bind a default only when the request was None; a different response to explicit
Some must reject before model input. Running keeps these effective fields pinned.
The actual caller fixture must observe fresh Starting metadata before native
setup, effective binding before dispatch, explicit mismatch rejection with no
wire input, higher-input same-UUID continuation and exact prewire restoration.
Issue19 cannot claim this consumer behavior merely from Store synthetic tests.

Claude5 keeps requested model/effort unchanged and publishes native_ref with
Running and consumed input before wire. Its coordinated caller now retains the
privately owned known UUID in fresh resume Starting, with a causal before-update
fixture and exact failed-prewire rollback. This does not yet prove complete typed
frame acceptance; that reviewed caller integration remains mandatory.
Grok7 keeps requested model/effort and must retain its existing native identity
semantics. Its current generic Starting records do not yet contain the complete
protected input pins, and its dispatch intent is not the typed consumed contract.
It must adopt exact full-frame preflight, SHA/source pins before Starting, and
atomic consumed admission before actual wire delivery in its reviewed integration.
All three providers require causal typed-frame caller fixtures. These are pending
consumer changes until reviewed integration; this Issue does not edit another
owned worktree or advertise generic fixture success as typed input acceptance.

Recovery JSON keys cannot silently extend authoritative prepared input. Parse
reserved input_* keys into the exact current typed namespace and reject unknown
ones on protected input; newly authoritative fields require projection/schema
and old-writer fence review. Unrelated bounded provider diagnostics remain allowed.
The private publisher derives semantic hashes inside its transaction; adding an
untrusted JSON field is never equivalent to extending the typed source contract.

Boundary proofs additionally cover first allocation on terminal-to-Starting UPDATE,
second actor rejection, fake terminal Session binding/closure, native-only context
uniqueness, admitted metadata substitution, allowed initial binding, explicit Some
mismatch, historical binding after checkpoint append and exact prewire rollback.
An allocated unbound failed native owner remains durably held until verified
terminal/cleanup evidence. An unallocated post-marker failure follows the existing
release policy and may need explicit recovery; no automatic poll release or forged
terminal actor is introduced here.


### Dispatch cardinality and irreversible typed claims

A consumed dispatch_intent denotes one model-input delivery for one PreparedInput
attempt, not each native progress/response event. Historical acknowledgements,
stream chunks, approvals, usage and per-turn response IDs do not create a new
intent or send new input. An actual new input/turn dispatch must take latest
frame/head admission, even on an already admitted Session. If that Session has
appended a checkpoint, its old pinned frame is stale: supported continuation is
authoritative terminal settlement, idle consecutive pack publication and a fresh
higher-input Starting. Live multi-turn input against the old checkpoint is not a
supported Core continuation shortcut. This availability limit applies to standalone
as well as Workflow Sessions; it cannot be hidden by relabeling a new dispatch as
an acknowledgement. Test own-checkpoint append then reject a changed consumed
intent with old pins, and prove explicit higher-input continuation succeeds.

Every new irreversible Pr/MergeGate/Cleanup evaluation on a typed owner requires
an explicit checkpoint:head key, including the none sentinel, before comparing the
indexed live head. Missing or malformed keys fail closed; validate(None)'s legacy
compatibility cannot admit a typed claim. Same historical admitted claim can finish
after a later append under the existing immutable claim rules. Legacy-only owners
retain their explicitly separate contract. No typed-to-opaque publication is
allowed in either generic context writes or the private Workflow transition path.
Protection derives from the owner's durable typed ancestry (an indexed/latest
non-downgrade invariant), never a caller-selected JSON flag. Verify both publication
paths, missing-key irreversible caller boundary and allocation-aware unbound
Reviewer closure with independent Store connections and meaningful mutants.


Issue43 supplies the narrow record-only Workflow native binding transaction so a
successful adapter.start does not invalidate its own Task-version snapshot. Its
exact owner/Task/Workflow CAS must compose with this design's private allocation
check in the same transaction, retaining canonical Task semantic projection and
native-only phase context ownership. Integrated native acceptance requires the
reviewed binding port and combined caller regression; a synthetic Store proof
cannot substitute for that integration. Issue41 observer/release policy remains
independent and unchanged by this context authority.


Boundary proof binds allocation to the exact input, not UUID alone: reuse an owned
Session UUID at a fresh native phase, fail prewire and restore its older Exited
snapshot. Binding/Succeeded for the fresh phase must reject, while explicit
not-admitted Failed/Interrupted closure remains possible with allocated-owner
provenance. A legitimate exact private preparation can bind pending Starting,
but not claim delivery. Verify both with actual caller ordering and mutation
controls; the record-only binding port retains the same proof.

Enable serde_json float_roundtrip for the shared checksum contract and use explicit
sorted nested Value encoding. Golden fractional/large-integer diagnostics must
survive serialize→Store read→checksum identically; payload hashes still use exact
bytes and old checkpoint-v1 hashes remain unchanged. Any future JSON encoding or
feature change affecting authority needs a reviewed persistence/fence change.
Native consumers compute the pure shared helper on their exact prior Session
snapshot, never on independently assembled recovery JSON.

Private phase allocations additionally have no-update/no-delete/no-replace SQL
triggers, preserving immutable scope/context/Session identity. Admission SQL
constraints enforce pair nullability, lowerhex SHA, immutable owner scope and
nondecreasing admitted version (an admitted pair cannot be cleared). Equal-version
digest change remains the narrow private BoundHistorical code path with old-digest
proof and permitted monotonic actor binding; SQL version constraints do not pretend
to independently infer those typed actor fields from a hash.

Variant cap recovery is bounded per context, not a global retention quota: each
explicit idle cap-exhaustion republish appends one historical pack and at most128
new variants. Repeated intentional preparations can grow audited history; no
automatic force loop is permitted. Runtime retention/quota remains separate,
without deleting authoritative history to manufacture availability. The earlier
claim about unused force calls applies only when the cap has not been reached.

Standalone re-admission by a new Session UUID is explicitly allowed after prior
ownership is genuinely terminal, with fresh lifecycle/lock/Git/source/private-frame
validation. One delivery per input attempt means (Session UUID,input version),
not globally one delivery per Task/context. Same-UUID continuation needs a higher
version; Workflow native phase slots retain single-owner context allocation.
This permits explicit standalone retry without pretending that source freshness
or persisted Task history is native wire proof.

Store::open performs migration only, without application reconciliation/audit or
native effects after the migration transaction. A runtime that invokes additional
startup reconciliation closes the manual backup window immediately. The manual
rollback precondition remains operator-attested; no automatic restore method or
private safe-rollback certificate is introduced by this context contract.


### Native ownership, terminal uncertainty and factual checkpoint provenance

For every persisted Lost Session, generic put_session/put_record cannot move it
to Exited/Stopped or another terminal state. For a protected or allocated Session
with native_dispatch_unobserved=true, generic writes cannot manufacture terminal
settlement either. Conservative diagnostic updates preserve the monotonic flag,
input, actor and consumed intent; an ack/preparation row does not prove native
completion. Issue14's future private owned-recovery transaction must verify
authoritative native terminal or cleanup evidence before settling such ownership.
No such port is advertised in this Issue. Succeeded/allocated closure therefore
cannot obtain terminal proof by converting Lost/uncertain history through a
generic write. Normal non-uncertain owned Starting/Running-to-terminal caller
updates retain their native adapter protocol; Store does not attest arbitrary
Session JSON as native output. Test allocated admitted Lost-to-Exited rejection
through independent connections and unchanged Workflow closure, including a
mutant that removes the transition guard. Legitimate initial terminal Consultant
history remains recordable; it is not recovery of a persisted Lost owner.

Native_ref preservation is conditional on the provider's private live ownership
registry, not permission inferred from a generic terminal Record or a checksum.
Before fresh continuation reservation, setup or wire input, each native provider
must resolve the exact Session UUID in its own registry and match Scope, agent,
provider, role, Task worktree and the exact persisted prior terminal snapshot. An
unregistered historical UUID cannot attach/resume an arbitrary native_ref, even
when it names another Project's conversation. Known references remain immutable
on legitimate same-UUID continuation. The current Codex resume path checks exact
owner metadata/persisted equality and then registry.get(previous.id) before
Reservation.persist/native setup; a forged terminal Record has no registry entry.
Include that actual producer path and causal no-wire caller rejection in the
acceptance evidence. No global native_ref index is introduced without a verified
provider-owned admission gap. Generic historical data alone grants no native
resume authority, and retained model input must still pass the complete typed
frame/source gates on each genuinely new delivery.

Checkpoint events are explicit caller-classified facts, with exact historical
Session/Task/HEAD provenance; classification is not a native transcript or model
completion attestation. Appending validates current scoped Project/Goal/Task and
Session versions, exact worktree/role, source bytes and consecutive checkpoint
head atomically. It may retain factual Consultant/earlier-session history. The
Session ID is historical attribution, not a statement that its event text was
produced by that native session, or that its private preparation/admission pair
certifies the truth of a constraint. Consumers must present retained events as
caller-supplied coordination history, distinct from current authoritative
Goal/Task/rules and independently verified outcome evidence. Neither a generic
terminal history record nor an event classified Completed authorizes Workflow
Succeeded, native launch, merge or lock release. Cross-Task Consultant promotion
retains this attribution and rederives target instructions. A later factual append
can fence future fresh claims; an already admitted irreversible claim retains its
frozen provenance and may finalize without discarding the history. A post-PR
head drift may still require explicit reconciliation before a new claim; this
declared hold is not hidden by deleting history or exempting claim freshness.
Test factual Consultant history and late-after-admitted-Cleanup append retention,
as well as rejection of foreign Session/worktree/version provenance. Native
transcript ingestion/provenance certification is a separate owned adapter port,
not an assertion made by this deterministic condensation API.

Legacy checkpoint format-v1 producers construct mandatory_goal/task only from
fixed typed Goal/Task projections. Their other fields are strings, booleans,
integer counters and those projections; Session.recovery floating diagnostics are
not part of Checkpoint. Initial floating Value examples assembled outside that
private producer are not proof of a reachable historical checksum change. Before
5-to-6 mutation, nevertheless verify every persisted typed checkpoint's shape,
digest and every CheckpointRef/head reference in owned packs/phase contexts,
including predecessor chains. Any inconsistent legacy reference refuses migration
without repair or partial schema changes. Native old5 producer fixtures plus
independent corrupted-reference fixtures establish compatibility and refusal.
Session restore checksums separately cover fractional diagnostic parse/serialize
roundtrips; their actual native producer bytes must remain identical. Encoding
feature tests must distinguish that real restore input from fabricated checkpoint
number cases.
