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
Existing Running/terminal observations remain valid and preserve immutable active
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

A bound owned worktree is a precondition of this source provider; Issue 9 must
provision it before Engine initialization, including STANDARD/STRICT workflows.
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
owned worktree supplied by Issue 9 remains necessary before Engine initialization.
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
