# Issue 19 design: typed working context with explicit authority

Status: actual merged main is schema3; this unmerged component source is5. Proposed
schema6 remains fixture-only/pre-code until its independent gate and actual composed
source/native production release. No Phase::DecisionReview or native producer is
created by prose. The schema6 canonical writer table below plus named Design16 and Design18–36 private
port/bounds define proposed native authority. Earlier schema5 protocol examples are
historical component context ONLY and cannot authorize a managed/public writer;
artifact/encoding contracts continue where this table does not supersede them.

## Canonical schema6 native writer predicate table

Each managed row below requires actual nonserializable owned operation authority,
exact scope/attempt/frame/pair/remembered Session version, complete pinned lock set,
actor/worktree identity and the row-specific private producer. Public ScopeSnapshot,
role, JSON, receipt ID or row existence is never an alternative credential. No generic
write may bump an allocated managed Session even with identical body, including after
phase closure. Existing activity/worktree/CAS restrictions remain; factual settlement
is separately permitted under authorized lifecycle holds, never launch/ALLOW.

| Transition | Only permitted writer | Additional authority / result |
| --- | --- | --- |
| Initial unmanaged Task or NoTask nonterminal Session | None | Unsupported before effects; future58 NoTask private port is separate |
| Initial terminal unallocated factual history | Generic factual writer | Unverified captured classification, no launch/cleanup authority; no Lost or allocated-owner replacement |
| Managed initial Starting / native setup binding | Actual operation-bound private preparation | Same-tx composed23 admission; current full frame/head/lifecycle/CAS; monotonic requested→effective binding; effects_started before Session/effect |
| First/new consumed input (including Starting→Starting) | Actual operation-bound NativeCAS | Same-tx composed23 admission; current full CPP/head/frame/source + P/G/T/own Session/locks/activity; unique consumption committed before bytes |
| Consumed Starting→Running acknowledgement | Actual owned historical acknowledgement | Exact current admitted tuple/consumed UUID, same attempt; no re-admission or new payload |
| Admitted Waiting/pending permission or fixed DENY observation | Private Acked/DenyHistorical | Original input/actor pins and provider-specific activity; no prepared consumption or grant |
| Host-executed native callback effect (ACP filesystem read/write) | Private HostCallbackCurrent with actual registered effect authorization | Current exact method/path/bytes/effect scope, operation/frame/locks, lifecycle/governing/head predicates BEFORE host IO; no native JSON credential, Broker fiction or model consumption |
| Tool/action ALLOW | Private AllowCurrent + actual trusted grant proof | Same-tx composed23 admission; current immutable CPP/head/governing/reference source authority + P/G/T/own Session/locks/activity before wire; Executor mutable content excluded as Design26 specifies; no historical fallback |
| Current managed non-Lost terminal / receipt | Actual owned supervisor settlement | Actual current native outcome or eligible NoCurrentDispatch + complete required owned cleanup; immutable exact receipt/body |
| Exact prior terminal restoration before current dispatch | Actual owned settlement producer | Same Immediate transaction as the new operation's NoCurrentDispatch receipt/body/audit; exact private prior checksum, no current admit/consume/uncertainty; never current success |
| Fresh higher-input continuation | Sealed managed Fresh/Continue selector + new operation | Exact latest closed predecessor/receipt; Continue actual registry proof; new context/frame/full current CAS |
| Managed monotonic Lost diagnostics | Actual private owned observation | Actor/input/intent pinned, conservative flags only; held shares/locks, no receipt or completion |
| Any Lost→non-Lost or consumed-uncertain release | None here | Actual14 recovery required; generic terminal labels and late diagnostics cannot release |
| Workflow native SessionID None→Some binding | Sole private43 record-only binder | Exact allocation/private prep/admit OR sealed known-current closed-receipt proof; Store-derived ORIGINAL resulting marker P/G/T/postmarker Workflow/full-lock frame, never refreshed expectations; full active-owner CAS; ordinary Workflow Record version/updated_at increment + one bounded reserved audit only, no Session/Task/P/G/lock write |
| Bound-live native diagnostic observation | Sole private43 record-only observe_native_diagnostic | Exact actual private operation/attempt/Workflow CAS; bounded diagnostic + Workflow version/updated_at + audit ONLY, duplicate no-write; no P/G/T/Session/full-lock/native-pin change, authority refresh, fail/retry or input |
| Engine stop request for invalidation/escalation | Actual owned supervisor request, bounded request metadata only | No Task/Workflow authority rewrite or fabricated outcome; ACK alone is not settlement; actual NCD or current terminal+full cleanup required before closure |
| Managed terminal decision | Separate private19/8 terminal-decision port + actual23 lifecycle authority | Exact original operation/frame and complete legitimate predecessor proof; authorized Task terminal delta plus terminal_decision-only W projection and one reserved link; no grant, success or release |
| Workflow non-success/TerminalRecovery closure | Exact private receipt + legitimate complete successor predicate | Required actual settlement; separately authorized terminal decision may be retained without raw marker refresh; unbound ID stays None; unknown/revoked consumed failure held14 |
| Later closed-attempt retry/escalation/invalidation | Genuine trusted Human ingress OR recorded finite frozen-policy authority | Exact already closed awaiting_explicit_retry attempt, current lifecycle/source and shared lineage budget; NEW admission only; no bare reason/RetryEvent JSON credential; #41 unmarked invocation-owner policy separate |

The dispatch-marker transaction also invokes the SAME private composed23 admission
predicate as preparation, first/new consumption and AllowCurrent in its Immediate
transaction: Registered Project, exact accepted goal_authority/definition digest,
Running Goal without a whole-Goal hold, exact listed DAG Task membership and verified
compact genuine WorkflowSuccessProof authority for EVERY hard prerequisite within the
Design32 transaction-read bound. Full evidence extraction happens outside SharedStore
and is published through the actual23 private certificate producer/CAS; no repeated
full Workflow decode inside native admission transactions. Design35 distinguishes compact evidence proof
from bounded mandatory owner/body writes and SQL validation; no total512-KiB mutex
claim follows. A ready report, Merged label or
Created/Analyzing Goal is insufficient. Continuation follows23 revocation-only
current prerequisite/input predicates; self-owned operation is not a competing claim.
Absent actual composed23 predicate means Unsupported before effects, not a weaker
active-Goal fallback. Historical acknowledgement/DENY/observations and actual factual
settlement retain their separate guards and do not require new dispatch readiness.

All CPP/rule/source integrity checks are bounded observations outside SharedStore,
followed by exact same-transaction authority revalidation; no OS-atomic FS claim.
Unknown/absent authority refuses rather than selecting a historical generic branch.
The same table is acceptance inventory for real Generic/Claude/Codex/Grok/Fake writers;
synthetic no-subprocess proof is mechanics only, never a production native declaration.

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
Starting→Starting or a COMBINED unconsumed first-Running admission, so an intervening
checkpoint prevents stale launch. Already consumed Starting→Running acknowledgement
is historical under the canonical table and never re-compares the live head.
Existing admitted Running/terminal observations remain valid and preserve immutable active
attempt identity. `Store::validate_checkpoint_source` exposes the pure bounded
comparison for native adapters; it performs no Git/filesystem work.

Admitted native Sessions keep their immutable launch head through approval/human
observations and Lost diagnostics. Under the schema6 transition contract below,
Lost is absorbing and cannot reenter Running through generic writes. New reservations, new consumed dispatch intents and
combined unconsumed first-Running admission compare the current head; consumed ACK
retains its immutable admitted head. Finalized frozen Cleanup binds its original historical head; a
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

Historical schema5 continuation protocol (NOT a schema6 launch exception; use
the canonical table below): an admitted input's revision/version/byte count/source
versions remain pinned through observations. An owned terminal→Starting continuation may
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
Cross-Task promotion selects caller-classified Consultant-origin history only; it
preserves the target Goal/Project/rule constraints and does not implicitly select
source Executor history. Initial terminal Consultant role is unverified caller
classification, not evidence of a native Consultant process or truth of its prose.
Promoted content must render that classification explicitly (Design22 below).

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
needed fresh private preparation under schema5. Under proposed schema6, standalone
preparation remains an artifact operation; protected standalone native launch is
Unsupported before reservation or process creation (Design18 below).

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

Historical schema5 generic blocked-owner protocol; schema6 managed observations
require the actual private handle (canonical table): Lost updates may add only
native_dispatch_unobserved=true and clear
PID while preserving exact actor, scope, input, restore proof and consumed intent.
This flag is conservative uncertainty, not a new dispatch admission. New
checkpoints record their configured transient window; historical checkpoints with
no recorded policy retain an explicit unknown value.

HISTORICAL UNMERGED COMPONENT5 ONLY (superseded by schema6 original43 frame):
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
SHA256, consumed dispatch UUID required whenever that admitted input pair is present, and private
validated-preparation input_version/metadata SHA256, and optional frozen
preparation_restore_sha256 for a fresh terminal continuation. That checksum is
verified against the previous persisted terminal Session at private preparation;
initial creation has None. Each pair
is both null or both present; present hashes are length64 lowerhex. Initial terminal
history does not create a row. A validated Starting may register the preparation
pair without asserting admission. A freshly validated consumed-dispatch publication
(including Starting before wire) writes the admitted pair together with its consumed UUID. First Running either atomically consumes the frame or observes the already consumed exact private pair; plain protected Running without consumption is rejected.
An admitted pair means Store-authorized consumption committed before wire, never native wire acknowledgement. This distinction permits verified prewire-failure continuation
without claiming model delivery. It references
the Session Record and owned Task; its helper derives all IDs from the validated
Session, never from caller-supplied ack metadata. Scope equality is checked against
the actual persisted Session and Task. Project/Goal-only Sessions and unprotected
legacy inputs receive no admission row. Their history contract remains, except
for the explicitly new universal absorbing-Lost safety rule below. Generic Records, audit events and caller
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

The current managed classifier exposes only Validated, ConsumedHistorical, Acked,
BoundHistorical and NotAdmission to the appropriate private preparation/admission or
observation port. The single schema6 order is structural bounds → universal Lost /
terminal-to-Lost refusal → private port dispatch → strict branch DTO parsing → that
branch's current or retained predicates (Design29). Exact prior-terminal restoration
is NOT a NativeCAS outcome: only actual settlement may restore it in the same receipt
transaction under the frozen private checksum and no-current-admission/consumption/
uncertainty predicates. It creates no input pair, consumption or success. Generic
managed writers cannot route into this transaction. Component5's earlier
bounds→RestoredPrior→parse launch guard is historical only. New consumed UUIDs always
validate the latest full frame/head before consumption; historical pair/UUID proof
never authorizes another dispatch.
Waiting/Lost never writes
an ack. Only the private native CAS port may create an admitted pair or consumed UUID,
after `Validated`. `ConsumedHistorical` is read-only proof requiring an existing
exact indexed pair/UUID; it cannot seed or replace a row. A new Running input
without matching already-indexed or atomically published consumption is rejected; `Acked` requires the NEW exact Session digest to equal the admitted pair and
preserves the existing row. `BoundHistorical` is
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

Standalone prepared frames retain Executor artifact provenance but are not
launchable under schema6; native Starting/Running is Unsupported. Managed Session
admission requires exact attempt agent/role and Task.worktree plus actual operation
handle, scoped frame, lifecycle and lock authority. Workflow frames retain their exact active
attempt agent/role/Session fences AND require Session.worktree == Task.worktree
for every native actor, including Reviewer, on initial Starting and Running. No arbitrary Consultant/ApprovalReviewer role
can adopt an Executor frame. Existing scoped Consultant history may be condensed;
live consultation needs a separate prepared-frame port. ApprovalReviewer requires
an action-decision-only Task through the Approval Broker; native ownership still
requires an actual managed phase/member operation and separate native settlement.

The single-actor Workflow port also allocates one private `phase_session_owners`
row per (Project, Goal, Task, context_version), containing Session ID. Context
version uniquely identifies the immutable native phase attempt. Every first private native CAS-validated
preparation/admission write for a typed Workflow native phase (not standalone or
EvidencePort) allocates it atomically with Record/audit:
INSERT or UPDATE to Starting/Running, or first/new consumed dispatch. This includes only a terminal
managed Session entering a sealed Continue with its exact closed-operation receipt;
unverified legacy terminal history cannot enter that path. Later writes
must be by that same Session. Another Session cannot reserve or consume the frame
even before Engine binds attempt.session_id. Allocation is immutable across
terminal history, foreign keys bind the Session and Task, and generic writes have
no allocator API. A failed Session/audit transaction leaves no allocation. The atomic Workflow transition enforces that every newly appended attempt uses
an owned context_version strictly greater than every prior native Executor/Reviewer
attempt when appending a new native attempt; a None-context transition cannot
reuse an old version for a new native attempt. EvidencePort phases retain their
existing separately validated context contract. The same Workflow transition must
check fresh attempt.session_id binding against the private allocation in the
same transaction; closure uses the separately defined current-input or restored
not-admitted predicate below. Whenever an allocation exists, closing/replacing the
active attempt requires that exact allocated Session to be persisted terminal,
regardless of whether attempt.session_id has been bound; Succeeded additionally
requires Store-persisted non-uncertain Exited AND the exact current admitted pair with a
privately committed consumed dispatch UUID for that input. Owner UUID and
terminal state alone never identify the input. Any fresh session_id binding must
match that Session's exact input version == attempt.context_version, payload SHA
and byte count == immutable ContextVersion.data.payload, revision and complete
source_versions == context authority, plus a matching private validated preparation
or admitted-and-consumed pair. Pending Starting preparation may bind an actor without asserting
delivery (Grok start returns before inference); Succeeded requires the matching
private ADMITTED pair AND privately indexed consumed intent for those exact input
pins, plus the adapter-owned terminal outcome; neither preparation nor Store consumption
certifies native delivery.
A Session restored to its older terminal snapshot after fresh prewire failure uses
an explicit closure-only `RestoredPriorNotAdmitted` predicate. In the same Workflow
transaction require: allocation owner == the immutable already-bound session_id
(if any); current persisted owner is terminal, non-Lost and non-uncertain; private
preparation version == the attempt context version with the current private preparation digest, frozen from restoration/terminal write until
this allocated attempt closes; and neither an admitted pair nor consumed UUID exists for
that attempt version. If the current terminal has that same input version, its
exact tuple must match the preparation pair. For the older restored input, require
a present private preparation_restore_sha256 equal to the canonical checksum of
the current entire terminal Session. This checksum was derived/checked against
the prior persisted terminal during private fresh preparation, not copied as an
unverified caller claim. Successful prewire restoration was separately checked
at its Session update; retaining this frozen checksum permits causal closure
verification after the pending Session body has been replaced.
The current older terminal digest/version need NOT match the fresh attempt; this
exception applies to non-success failure/interruption or explicit retry replacement of that allocated attempt, never fresh binding or
Succeeded. Every removal/replacement of an allocated active attempt under every WorkflowAccess invokes this same owner/terminal/current-input-or-restored predicate, including retry from Waiting or Failed: there is no state-label exemption. Retain an already bound session_id unchanged; leave None when no binding
occurred. Do not clear/rebind the ID to make the predicate pass. The failed pending
preparation alone is not delivery proof. Test restoration both before and after
legitimate PreparedPending binding, using independent Store connections and a
mutant removing the no-current-admission check. A Running
Reviewer is operational ownership even though it is not an Executor reservation.
An unbound claim cannot close around it, and another terminal Session is never
release proof. Binding may update only the Workflow Record while retaining exact
Task/Project/Goal/Record CAS; this check does not require a Task version bump. Expose a pure scoped
allocation reader for explicit recovery. The recovery port must resolve an unbound
claim using this private owner, actual persisted terminal Session and verified
native cleanup; it must not invent another actor or clear Lost/uncertain ownership.
For a proven typed Workflow native-phase context only, an absent allocation is
factual proof that no schema6 typed Workflow Session/input was admitted through
Store for that exact context, because every such private write allocates
atomically and older writers are fenced. It does not by itself invoke or broaden
Workflow release policy: ordinary observer polling remains read-only, and owner
release still requires its own exact claim token and applicable pre/post-marker
rules. The reader requires typed native-phase ancestry before reporting absence;
standalone/evidence/opaque history returns NotApplicable rather than no-admission
proof. Applicable absence proves no typed admission, but an outstanding managed
operation can still own native startup/setup before a first Session. Release also
requires the actual operation settlement receipt or the separately applicable
pre-marker invocation-owner proof; absence never settles native setup. Explicit recovery can use applicable absence to prove no typed admission, while
an allocated Starting/Running/Lost or uncertain owner remains held until genuine
terminal/cleanup evidence. An unbound post-marker failure without its exact managed
NoCurrentDispatch receipt remains held until actual settlement/private recovery;
with that no-Session receipt the explicit non-success closure row below applies.
This is an availability limit,
not a claim that absence means unknown model dispatch. This index alone introduces
no automatic release policy. The separate managed settlement closure row below
allows exact no-Session NoCurrentDispatch receipt closure; without it an unbound
post-marker operation remains held for private recovery.
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
phase or captured status. For Workflow-owned Tasks, blockers and next_action are Engine-owned writer-pinned fields: generic put_task/put_task_tx rejects changes, exactly as it pins workflow/risk/context/phase/artifacts. Only the atomic Workflow writer may synchronize them. This is required because they are excluded from the phase-stable instruction digest, and no generic operator update may become an unfenced model-visible instruction. Task.revision is verified against physical captured Git
HEAD and frame revision. Engine effective workflow/risk/budget is immutable phase
wrapper/attempt authority, not a phase-stable source hash.

For new managed admissions, require Task.state and Task.phase to equal the
immutable native phase's expected state/key, in addition to active attempt guards.
This is fresh input admission, not historical acknowledgement. An authorized
lifecycle hold after Starting but before first consumption fences input. The
schema5 standalone state allowlist and admission policy are historical component
behavior only; schema6 native standalone is Unsupported. Standalone artifact
preparation still includes OWN Task blockers/next_action/artifacts and workflow/risk
in its versioned policy digest, because these can affect artifact selection, but
that digest never authorizes a Session or wire. Sibling progress/status remains
captured bookkeeping rather than an admission credential. Accepted Goal definitions
remain immutable under23; authorized lifecycle stops or actual typed Task/graph
ports cannot be replaced by generic Goal instruction edits.

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
version for artifact-contract migration or bounded variant-cap recovery. Forced
standalone publication is permitted only when the current artifact lacks the
required v2 instruction contract or its private variant count reaches cap128;
apply the same idle/source/head/owner CAS and audit, retaining all history.
A schema5 terminal Session/private admission pair is not a schema6 eligibility
branch. Otherwise identical reuse remains required. Test old-contract republish
and cap128 recovery as non-launchable artifacts, and native launch refusal before
reservation; no new-UUID admission positive is required or supported.

### Migration and already-open writers

Schema6 does not implement hot migration of active native work. Before ANY schema
mutation, under the same Immediate lock, inspect persisted scopes and REFUSE
migration if ANY Session in ANY scope is nonterminal (including Lost), any active
or malformed WorktreeLock remains, or any Workflow has active.is_some() OR owns a
nonterminal Task REGARDLESS of finished (including idle, held, QUICK PrCreated or
post-PR phases). A
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
databases can exercise isolated schema6 component mechanics without claiming that
recovery integration or production deployment readiness. An explicitly terminal Task's non-owning Workflow
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
BEFORE INSERT/UPDATE/DELETE fence triggers to EVERY non-sqlite_ application table,
including projects/goals/tasks/records/context_versions/usage/audit/checkpoint_heads,
prepared_pack_inputs/session_input_acks/phase_session_owners/context_admission_epochs,
native_phase_operations/native_phase_settlements/workflow_context_counters,
workflow_observation_sources/workflow_observation_outcomes and EVERY actually composed
authority table such as goal_authority. This is an open exhaustive schema-enumeration
rule, not a closed literal table allowlist. Final DDL goldens enumerate the actual
composed source schema, never this candidate component list alone. Each requires this function to return
6. Register UTF8|DETERMINISTIC|INNOCUOUS, never DIRECTONLY, and verify
writes with trusted_schema=OFF. A schema-enumerating regression requires every non-sqlite_ application
table to have all three compatibility triggers; new tables cannot silently escape.
Preserve all existing immutable/append-only/domain triggers; fences only add
writer compatibility checks. Include private runtime metadata and audit writes,
not just Session updates. An already-open old5 connection cannot resolve the
function and its SQL write fails atomically after schema6 migration. Public Store
has no caller function registration or trigger-bypass API. Privileged arbitrary
SQLite schema editing is outside the Store contract. Future migrations must retain
required earlier functions and install their own exact-version write fence.

The actual old5 native fixture is compiled from immutable public source
`e6cf75dc61d0c9c9a6a225c64c8f9aaf7d6ffd26` and, separately, the latest schema5
production source `79af00a603149729fe26bd1079aa4d49c1932cad` after the reviewed
Grok/CAS merge (two distinct unchanged-source binaries, never one patched build),
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

Tests cover pending protected Starting→Waiting rejection and fresh managed consumed head checks and absorbing
Lost-to-every-other-state rejection, unknown
flag forgery, caller JSON ack forgery, successful first Running then historical
reentry, forbidden Running-to-Starting-to-old-terminal rollback, exact prewire
restore ONLY through actual atomic settlement (not the historical component5 launch
guard), higher-version continuation, Session/ack/audit rollback and reopening.
Standalone native admission must be Unsupported before reservation; wrong managed
Workflow role/agent/worktree must fail on initial Starting AND Running. Two independent Store connections racing to
reserve the same Reviewer attempt must yield exactly one Session allocation;
neither JSON nor prebinding session_id=None can admit a second actor.
Changed instructions/policy/ref hashes fail; sibling progress remains usable.
Typed Workflow-owned contexts cannot silently downgrade to opaque legacy payloads.
Important guards receive caller-level mutation proof and immutable source review.

The private ack row additionally stores input_version and checks SHA length64.
Its ADMITTED pair may be created/replaced only by private `Validated` with a
strictly higher input_version when replacing a present pair; equal matching metadata is idempotent, unequal
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
not permission to launch a model. Initial TERMINAL Project/Goal-only factual history
remains allowed, never nonterminal native ownership through a generic caller.
Schema6 NoTask launch requires the future58 port, and Issue19 prepares no launchable Goal input or standalone
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
terminal/cleanup evidence. An unallocated post-marker failure follows the exact
managed no-Session receipt row below; without a receipt it remains held. Pre-marker
release follows the existing invocation-owner policy; no automatic poll release or forged
terminal actor is introduced here.


### Dispatch cardinality and irreversible typed claims

A consumed dispatch_intent denotes one PreparedInput-frame delivery for one
attempt, not every native protocol frame. Historical acknowledgements, stream
chunks, usage and per-turn response IDs do not create a new PreparedInput intent.
Structured protocol replies (an exact pending operation's fixed Approve/Deny/Cancel
choice or native-owned tool results) are a separate protocol class, not another
PreparedInput frame. Runtime free-text Human/operator replies or new instructions
are not supported on protected live Sessions: settle terminal ownership and
publish a higher-input frame that embeds the typed attributed reply fact. They
cannot be routed as a diagnostic or protocol choice. Broker operation approval
retains its own exact operation/turn/permission/lifecycle gates; this input index
does not grant an operation or attest arbitrary native tool-result text. The actual
Codex Reply request exposes only typed IDs/hash plus OperationDecision, not a
free-text reply channel. Test all actual public provider input entry points and
reject a model-visible free-text shortcut before wire. An actual new input/turn dispatch must take latest
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

Under schema5, standalone new-UUID re-admission used fresh lifecycle/lock/Git/source
and private-frame validation. This is historical component behavior, not proposed
schema6 authority: Design18 rejects protected standalone native launch before any
reservation or process. One delivery remains per (Session UUID,input version) for
supported managed input, with higher-version same-UUID continuation and immutable
Workflow native phase allocation. No terminal label certifies owned cleanup.

The explicit migration entry performs migration only, without application
reconciliation/audit or native effects after the migration transaction. Plain
Store::open refuses older application schemas unchanged (Design20). A runtime that invokes additional
startup reconciliation closes the manual backup window immediately. The manual
rollback precondition remains operator-attested; no automatic restore method or
private safe-rollback certificate is introduced by this context contract.


### Native ownership, terminal uncertainty and factual checkpoint provenance

For every persisted Lost Session, Lost cannot transition to another state. Schema6
managed diagnostic Lost-to-Lost requires the actual private owned observation;
generic put_session/put_record cannot write a managed owner at all. Lost cannot exit to ANY other state,
including Running, Waiting or terminal, regardless of historical admission. This
absorbing predicate runs before every launch/outcome classification. For a protected or allocated Session
with native_dispatch_unobserved=true, generic writes cannot manufacture terminal
settlement either. Conservative diagnostic updates preserve the monotonic flag,
input, actor and consumed intent; an ack/preparation row does not prove native
completion. Issue14's future private owned-recovery transaction must verify
authoritative native terminal or cleanup evidence before settling such ownership.
No such port is advertised in this Issue. Succeeded/allocated closure therefore
cannot obtain terminal proof by converting Lost/uncertain history through a
generic write. Historical schema5 normal non-uncertain Starting/Running-to-terminal
caller updates retained their native protocol; schema6 managed terminal publication
is ONLY the private owned settlement producer in the canonical table. In either case, Store does not attest arbitrary
Session JSON as native output. Test allocated admitted Lost-to-Exited, Lost-to-Running-to-Exited and
Lost-to-Waiting-to-Running-to-Exited rejection through independent connections
and unchanged Workflow closure, including a mutant that removes the absorbing
transition guard. Lost observations also preserve exact actor pins; late binding
cannot act as recovery. Legitimate initial terminal Consultant
history remains recordable; it is not recovery of a persisted Lost owner.

Native_ref preservation is conditional on the provider's private live ownership
registry, not permission inferred from a generic terminal Record or a checksum.
Before fresh continuation reservation, setup or wire input, each native provider
must resolve the exact Session UUID in its own registry and match Scope, agent,
provider, role, Task worktree and the exact persisted prior terminal snapshot. An
unregistered historical UUID cannot attach/resume an arbitrary native_ref, even
when it names another Project's conversation. Known references remain immutable
on legitimate same-UUID continuation. The current Codex AgentAdapter::resume first calls current(), which compares
its private registry watch snapshot byte-for-value with the persisted Session,
then launch checks exact owner metadata/persisted equality and registry.get(previous.id)
before Reservation.persist/native setup; a forged terminal Record has no registry entry.
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


### Complete update predicates and proof limits

Every protected Session update, regardless of target state, first compares its
actor pins with the old persisted Session: model, effort and native_ref must remain
exact. The only exceptions are a validated higher-input owned terminal-to-Starting
continuation, exact private prewire restoration, pending initial binding with
latest-frame/head/lifecycle validation, combined Validated first-Running/first-
consumption monotonic initial binding, or BoundHistorical initial None-to-Some
binding proved by the old exact admitted pair. BoundHistorical requires a nonterminal old state (Starting, Running,
WaitingApproval or WaitingHuman). For an open managed operation its NativeCAS
target must remain nonterminal; actual terminal binding is performed atomically
by the sole private settlement port with its receipt (Design19 below). Unmanaged
historical ordinary terminal observations remain separate;
terminal-to-terminal native binding is forbidden; it cannot
leave or change actor fields of Lost. All other actor fields remain exact on those
exceptions. Acked requires digest(new)==admitted_digest, not just the old digest.
A permitted Waiting initial binding after checkpoint append updates the admitted
and applicable preparation pair atomically using historical input, without new
wire or revalidating it as a fresh input. Model/effort/native_ref substitution or
clearing during Waiting/Lost/terminal targets rejects. Source/restore input fields
retain their existing same-attempt pins. Higher-input model/effort request reset
never resets the Session UUID's privately owned known native_ref. Tests include
every target-state actor substitution, late Waiting binding after checkpoint
append and audit failure rollback; mutate the global actor check and new-digest
equality separately.

Before hashing a protected admission tuple, enforce explicit limits: at most8192
source_versions entries; UTF-8 keys at most8192 bytes and values at most256 bytes;
agent/provider/model strings at most256 bytes each, effort at most128 bytes and
native_ref at most1024 bytes; exact nonempty UTF-8 worktree at most4096 bytes;
revision at most128 bytes and payload SHA exactly64 lowerhex bytes. Reject invalid
IDs, absent Task scope, zero or out-of-range versions, nonrepresentable paths and
arithmetic overflow. The entire typed encoded admission tuple is at most2 MiB;
input_bytes remains bounded by the complete typed model-frame 1 MiB cap. Cardinality
and aggregate string-byte accounting precede digest serialization/hash, so a
large map cannot force unbounded hashing. These are simultaneous caps, not a
promise that maximum entries/strings fit together. Actual prepared authority must
still equal the complete source map without silently dropping keys.

Writer-v6 SQL triggers fence all state writes; they do not sandbox an old binary's
external filesystem, Git or native RPC operations. The supported migration
procedure requires every old runtime and its owned groups to be verifiably stopped
and drained before opening the new runtime. The minimal held-connection fixture
is an idle Store writer, not permission for an old native supervisor to remain
live across upgrade. Its post-migration public-write failure proves incompatible
state-writer exclusion only. Operators check compatible-runtime status/owned
cleanup and close runtimes before upgrade; no automatic discovery of every
same-user process or global OS execution fence is claimed by Store. Existing
provider before-input Session/CAS gates remain mandatory, and native integration
tests must prove no model input is sent after that publication is rejected.
Worktree provisioning must not run concurrently with supported migration; SQL
fences alone cannot undo a Git side effect performed before a rejected write.
This restriction makes the public drain-only upgrade contract explicit rather
than asserting a database trigger attests external effects.

Besides per-table writer-fence coverage, compare normalized sqlite_schema tuples
(type,name,tbl_name,sql) for a fresh schema6 database and every legitimate migrated
v1–v5 fixture. Private-table definitions, domain/immutability triggers, indexes and
CHECK constraints must match, ignoring only SQLite internal objects and harmless
SQL whitespace. Native old-writer fixtures also verify marker refusal and
unchanged logical authority, not only trigger counts. Checkpoint append callers
are trusted runtime integrations/operators with explicit fact-classification
responsibility; the in-process Store API is not an authentication boundary against
a malicious same-user caller. Such callers can append facts that consume retained
capacity or hold future claims. Preserve attribution/audit and surface those holds;
do not claim private admission makes arbitrary event text trustworthy or silently
drop it to manufacture availability.


### Typed dispatch identity and explicit compatibility impact

Protected consumed intent is a fixed deny-unknown-fields DTO, containing id
(a nonnil UUID), origin=runtime, consumed=true, input_version, input_bytes,
input_sha256 and authority_versions (exact three bounded Project/Goal/Task
versions). It contains no progress/diagnostic keys; those belong outside the intent.
Its version/bytes/SHA match the pinned complete prepared input, and first consumption
checks current owner versions through the actual atomic native reservation. The
private admission row stores consumed_dispatch_id for the admitted input version.
First consumption validates latest full frame/head and stores that UUID with the
admitted pair before wire, even while Session is Starting. A second different
UUID for the same Session/input version rejects unconditionally, even when HEAD
and checkpoint are unchanged. A strictly higher owned continuation can install
its new pair/UUID. There is no protected admitted pair with null consumed UUID: preparation is a separate state; Running cannot authorize wire without consumption. SQL/typed helpers reject same-version consumed UUID substitution
or clearing. Historical observation requires exact intent identity and matching
private row, not merely equality of arbitrary recovery Values. DTO authority_versions
must equal the private CAS port's expected tuple AND the current Project/Goal/Task
rows; generic writes cannot turn a self-declared tuple into consumption.

Identical intent is observation only, never permission for another wire delivery.
Every actual provider delivery generates a fresh private runtime UUID and invokes
consumption immediately before the wire; provider dispatch is the private causal
port, not a generic Session writer. The native fixture proves exactly one PreparedInput-bearing wire
frame (separately from protocol replies) and rejects the second same-version dispatch, including unchanged-source
and after-checkpoint cases. Nonidentity metadata does not live inside this DTO,
so a diagnostic update cannot masquerade as novelty or wedge a historical ack.
Preserved legacy terminal intents are historical compatibility data, never
synthesized into new private consumption proof. Test intent substitution, unknown
keys, malformed UUID, duplicated delivery and private pair/audit rollback.

Exhaustively destructure Session when building actor/admission predicates: id,
scope, agent, provider, role, worktree are immutable ownership; model/effort and
native_ref are monotonic-initial pins; state is explicit lifecycle; pid and
started_at are diagnostic/timing; recovery is explicitly split into the typed
input/restore/dispatch namespaces and bounded nonauthoritative diagnostics. No
Session rest-pattern can silently ignore a new field. Its shape golden and
compile-time field inventory require a new authority field's explicit
classification and the same projection/schema/incompatible-writer-fence review.
Unknown authoritative input/intent keys reject rather than becoming diagnostics.

Apply applicable source-map/revision/version/byte limits at private standalone
preparation and Workflow phase publication before any pack/index/audit commits.
Actor-specific string/path limits also run on the actual launch request before
Starting/admission. Unlaunchable oversized authority cannot burn a variant slot
and repeatedly fail only later. Preparation does not invent model/effort values
that only a subsequent native request supplies. All provider consumers use the
shared pure session_restore_sha256(&Session) helper instead of their own
Sha256(serde_json::to_vec(previous)), including Claude5, Codex6 and Grok7; their
coordinated acceptance includes a preserve_order-enabled actual caller fixture
and fractional diagnostics. Its deterministic sorted encoding never asserts
native outcome proof.

Universal Lost is an intentional new safety change, including unprotected legacy
Task and Project/Goal-only history. The previous generic Lost-to-Stopped contract
is NOT retained: neither runtime JSON nor a dead PID proves unknown native death.
An actual58-produced Lost Goal/Project owner will hold Goal publication/Project
removal until trusted14 recovery. Candidate6 has no such positive producer: its
Goal/Project fixtures reject initial nonterminal/Lost and terminal→Lost, and
migration refuses old live/Lost unchanged. Only actual58 integration proves the
positive NoTask hold; never seed a generic Starting owner or relabel Lost as Stopped.
Initial legitimate terminal Consultant history remains
recordable and may contribute factual history; it was never a persisted Lost
owner and does not manufacture native completion. These availability changes
are explicit, not an assertion that old source already enforced the rule.


### Historical schema5 Native CAS structure and complete input bounds

This is the current unmerged component5 structure, NOT schema6 managed writer
permission. The canonical table and handle-bound ports replace every generic
positive exception; retained bounds/encoding constraints still apply.

`Store::put_session_if_current` is the crate-private native Session publication
port. Its existing Immediate transaction checks expected Session version, exact
Project/Goal/Task versions, active ownership and the complete scoped lock ID/version
set. It alone constructs a private `NativeAdmission` write mode for the internal
record helper. The mode is never a public enum/JSON parameter and carries the
already-checked current version tuple into the admission helper. A new consumed
DTO must have authority_versions exactly equal to that tuple. Outcome-specific checks below distinguish current admission from historical
observation. Fresh `Validated` checks latest full frame/head, actor, lifecycle and
phase allocation before it writes Session/preparation/admitted/consumed/owner rows
plus audit atomically.
No Git or external I/O occurs inside this transaction. Native callers use this
port for initial/fresh Starting, pending binding refresh, first Running admission,
first consumption and private BoundHistorical digest changes. This is a trusted
runtime boundary, not authentication against malicious code inside the crate.

Generic `put_record`/`put_session`, including other transactions that use the
ordinary record helper, use private `ObservationOnly` mode. For protected input
they reject new or changed consumed intent outside `RestoredPrior`, first/new preparation or admission,
phase allocation and BoundHistorical row mutation, including the frozen prior
terminal restore checksum. They may retain exact privately
indexed historical observations, monotonic conservative Lost diagnostics,
permitted terminal observations, or exact no-dispatch prewire restoration without
changing any private pair/allocation, for schema5 historical standalone or unmanaged factual history
Sessions only. An open managed operation prohibits all generic Session writes;
its private observation/settlement and post-receipt freeze rules below apply first.
Initial legitimate terminal history remains
recordable without private rows, never binding a fresh Workflow owner. Generic
fresh typed Starting/Running is rejected rather than implicitly reserving a frame.
A real provider's unmanaged historical terminal observation does not require live head currency;
it cannot grant another PreparedInput dispatch. Test initial creation and UPDATE
paths with independent connections, stale DTO versions and unchanged-head variants;
assert Session/rows/allocation/audit unchanged on rejection. Mutate only the mode
selection at the generic caller and require those tests to fail.

The Store predicates certify input currency and exclusive allocation. Even a
Store-persisted Exited plus private consumed UUID is not native completion proof:
ordinary non-uncertain terminal observations remain a trusted provider-integration
contract. Actual Workflow completion additionally consumes the adapter's owned
native outcome; native fixture proves that outcome causally. Caller-authored JSON
alone is not advertised as an authoritative native terminal transcript. Lost and
uncertainty remain absorbing regardless of this distinction.

One `MAX_TYPED_INPUT_BYTES = 1 MiB` applies to non-launchable standalone artifacts and
Workflow complete PreparedInput frame, including mandatory facts/rules. Standalone
Budget values above that cap reject before selection or variant counting; estimated
budget caps remain distinct estimates and cannot raise the byte cap. Private
publication independently checks rendered bytes before indexing, and admission
checks the same cap. Schema6 adds an input-byte BEFORE INSERT/UPDATE trigger to the
unchanged prepared_pack_inputs table, so migrated and fresh table definitions
remain equal. Previously persisted schema5 variants over 1 MiB are readable
non-launchable history; migration does not invent admission or silently rewrite
them. They cannot be reused for protected launch. Explicit idle consecutive
republish under the bounded current producer is allowed under the existing owner
and variant-exhaustion rules. Test oversized Budget and actual rendered bytes
reject before any pack/pointer/variant/audit publication, plus historical old5
oversized-variant preservation and denied launch.

Before any protected Session write or restore hashing, enforce at most 4 MiB exact
serialized Session bytes, at most 3 MiB recovery JSON bytes, depth 32 and 32768 JSON
nodes. Walk JSON with checked byte/node/depth accounting before serialization;
combine these with the stricter actor/source-map/2-MiB tuple bounds already stated.
These simultaneous caps do not promise every maximum fits. The shared pure restore
helper rejects overflow before hashing; private and generic protected writers
apply the same check before transaction work and again at the actual record boundary.
Existing oversized idle legacy terminal history is readable, not new admission;
a protected continuation requiring its exact checksum fails explicitly rather
than hashing an unbounded object or truncating diagnostics. No automatic rewrite
or forged terminal settlement is provided. Boundary fixtures prove limits, nested
overflow, no pair/audit writes, and preserve-order fractional checksum compatibility.


### Historical component5 restoration ordering; current managed freeze and scope

The component5 launch-guard ordering and its rollback-order mutant are historical
source context only. Schema6 uses the sole Design29 ordering; restoration is reachable
only inside actual settlement, never NativeCAS or ObservationOnly. At managed fresh
preparation, bound/checksum the prior terminal before storing private restore proof;
an oversized/unrestorable body cannot prepare. Its exact old consumed intent is
historical body content, not a new schema6 consumed DTO. Settlement checks frozen
canonical checksum, current preparation/operation and no current admission/consumption/
uncertainty atomically with receipt/body/audit. S6-01/04/08 mutate this single routing
and prove exact prior-consumed terminal→fresh preparation→actual settled prewire rollback
without second wire or new admitted pair.

Once an older restored terminal owns an open allocated Workflow attempt, its exact
body is frozen until that attempt closes. Reject ALL Session body changes (including
terminal state/PID/diagnostics/new Starting) even if ordinarily allowed by terminal
observation; exact no-change reads are allowed. Active allocated attempt identity
is derived from the current scoped Workflow Record/phase owner inside the same
transaction; malformed/contradictory references refuse. Diagnostics/timings/usage
can be recorded separately with attribution, never altering the restore body.
Closure can therefore compare the frozen checksum without a new restoration
marker or overwritten provenance. Once Failed/Interrupted closes that exact
attempt, managed Session history still rejects generic body writes; only a new
validated managed Continue may change it. Standalone artifacts allocate no Session.
Test a post-restoration diagnostic attempt is rejected, separate Usage/audit is
retained, Failed closure succeeds, and generic terminal diagnostics remain rejected;
mutate the freeze predicate to demonstrate the checksum wedge.

This private outcome table is subordinate to the canonical writer table and Design29
branch predicates: current grants and Running-target observations check their exact
current/remembered P/G/T and full locks; fixed unchanged Waiting Decline and conservative
Lost diagnostics use retained provenance plus their separately specified own-operation,
Session/activity/lock checks. Factual settlement is separate. Ordinary scope/actor
ownership and universal Lost rules remain global.

| Outcome | Head/frame and lifecycle | Private writes |
| --- | --- | --- |
| Validated fresh Starting/pending binding/first consumed | Latest complete frame, live checkpoint head; same-tx composed23 Registered Project/exact accepted Running Goal/no hold/listed DAG/hard-prerequisite proofs, nonterminal admissible Task; Executor ReadyForPr/PrCreated refused | Preparation/admission/consumed ID as applicable; typed Workflow owner allocation |
| BoundHistorical | Exact old admitted digest, same pinned input/intent, old nonterminal non-Lost; no live head/frame comparison. Existing CAS active Goal/nonterminal Task/Executor ReadyForPr fences remain | Only permitted monotonic initial binding digest update, no new consumption |
| Acked/ConsumedHistorical | Exact existing pair/UUID and unchanged actor/input/intent; no current head comparison, no fresh input | None |
| NotAdmission | Applicable ordinary history predicates, never fresh protected Starting/Running | None |

Paused/inactive Goal or Executor ReadyForPr may hold private BoundHistorical
publication even when its checkpoint is historical; report that as passive binding
publication held, not native outcome failure or permission to dispatch. Existing
fully settled factual observations/audit/usage remain separately possible with
pinned actors. Do not relax those lifecycle checks to hide a binding conflict.
The latest-head exemption is only for history; a new consumed UUID always follows
Validated and requires current active ownership. Mutate the per-outcome selector
and cover late Waiting binding after checkpoint append plus paused/ReadyForPr
rejection with no new model input.

After schema6, every fresh Workflow native Executor/Reviewer phase requires actual
typed19 frame publication, explicit own checkpoint head (including none), complete
mandatory task_pack plus rule/frame source authority, and its private native CAS
preparation/allocation. A new opaque native-phase context/launch is rejected before
process or dispatch marker even for a Task with no typed ancestry. Existing opaque
historical records and separately validated EvidencePort phases remain readable,
not new native launch authority. Standalone artifact frames NEVER allocate a phase_session_owners row or authorize
native admission. Tests cover opaque fresh native phase refusal, evidence/history
reading, and standalone new-UUID Unsupported before reservation/process. Generic
Task nonterminal Session refusal also applies without typed ancestry (Design20).

Native caller migration inventory includes GenericCliAdapter, Claude5, Codex6,
Grok7 and positive Workflow FakeAgent fixtures. Generic first Starting/Running
save_session currently calls the generic writer and must migrate to the actual
private prep/admit CAS; FakeAgent must use real typed producer/private pair too,
never bare recoveryNull, seeded private SQL or a test exemption. All production
adapters require complete pins/current tuple/lock CAS and causal before-wire proof.
The runtime's implementation-owned PreparedInputAdmission capability defaults
absent and is checked before dispatch claim/process; unmigrated adapters reject.
A misadvertising adapter that returns an owner without private current proof still
fails the narrow binder and retains uncertain ownership. Capability declaration
alone is not proof. Co-integration with #43 preserves both reviewed ancestries and
requires combined source review/exact-head CI; standalone helpers are not actual
native acceptance.

Two old5 writer artifacts are required independently: record each exact source
commit/tree, executable SHA and absence of writer-v6 registration, then fresh-open
refusal and held-open owner/Session/metadata/audit/prepared-statement refusal. Do
not substitute the earlier binary's result for the later merged-CAS binary.
Before checkpoint-v1 digest verification, reject any non-integer JSON number in
actual typed checkpoint bodies with distinct actionable migration refusal; this
is malformed legacy authority outside the typed old producer, not permission to
rewrite its chain. Preserve DB bytes/marker on refusal and compare valid native
old5 checkpoint bytes/digests under both JSON feature configurations.


### Combined binding, unconditional closure and consumption freshness

Validated first-Running or first-consumption may combine monotonic initial binding
with admission: old None may become one effective Some model/effort/native_ref,
while every old explicit Some remains exact. This exception applies to both
Starting→Starting consumption and Starting→Running. Latest frame/head/lifecycle
and native owner/lock CAS are mandatory; atomically refresh preparation digest to
digest(new) and write admitted pair=digest(new) plus the required consumed UUID for new admission.
It does not permit arbitrary actor substitution or a later same-input Some change.
Claude's actual combined native UUID/Running/consumption before-wire path need not
insert a fictitious intermediate write. Test this actual caller order and a mutant
removing monotonic checks ONLY from the combined first-consumption branch.

For EVERY fresh typed Workflow native phase, Succeeded has an unconditional Store
predicate: allocation PRESENT; attempt.session_id == that owner; persisted owner
non-uncertain Exited; exact admitted digest/input version == attempt.context_version;
and private consumed UUID for that input. Absence of allocation/session_id is NEVER
success, even for a dispatch-started phase with no Session written. Failed/Interrupted
with absent allocation requires the exact managed no-Session NoCurrentDispatch
receipt for post-marker closure; before marker the separate invocation-owner
policy applies. The
absence reader does not itself release ownership or certify native death. EvidencePort
phases retain their distinct actual-evidence predicate. Test no-allocation/no-bound-ID
Succeeded rejection under every WorkflowAccess and mutate only that absence branch.

Store Validated certifies current DATABASE authority, never physical filesystem
freshness. Every native caller (Generic CLI, Claude, Codex and Grok) performs full
bound-source revalidation outside SharedStore AFTER native setup and immediately
BEFORE first/fresh consumed CAS and PreparedInput delivery: actual root/FD identity,
HEAD, relevant dirty/admitted ignored bytes, rule/config bytes and all saved source
hashes must match the privately published snapshot. Re-capture with the real context
source validator, not HEAD-only verify_binding or caller JSON. Recheck scope/owner/
versions/locks in the subsequent CAS; no inference/wire proceeds after mismatch.
Record the observed snapshot identity/hash and ordering as provenance, not a JSON
credential. A same-length dirty edit during setup and rule/config/ignored-byte edits
must send zero PreparedInput frames for each production caller. Mutate each actual
pre-consumption validator, not only its helper. Source checks are bounded observations;
Store cannot atomically lock external filesystem mutations. The immutable review lock
and native read-only/owned worktree permissions remain separate safeguards, and this
contract never claims OS-atomic Git/filesystem plus SQLite transactions.

All Workflow native context consumers are inventoried: initial reserve, next phase,
explicit retry, escalation and source invalidation/generation restart, plus Cleanup's
frozen evidence path. Every NEW native attempt publishes a forced consecutive owned
phase ContextVersion, even for byte-identical unchanged source, before append. Existing
attempt reentries/observations retain their exact version; EvidencePort transitions
cannot manufacture native authority. Phase publication deliberately advances identity,
whereas standalone idempotent source publication remains separate. Terminal previous
native owners and actual claim/Task/source CAS must be checked first; no active input
is rewritten. Tests cover unchanged-source retry/new-phase and escalated/invalidation
paths, strictly greater versions, actual native allocation, failed-preparation closure
and no same-version reuse. The default observer/owner-release policy remains #41's
separate contract, not silently changed to satisfy these tests.

Schema5 permitted generic Lost→Stopped settlement. Drain preflight can reject
currently live/Lost owners but cannot authenticate old terminal history or prove an
operator did not previously relabel Lost. Its session.saved audit contains declared
states, not native cleanup truth; no audit JSON is a trusted recovery marker. Supported
upgrade therefore explicitly requires operator-attested genuine old-runtime terminal/
cleanup evidence and closed owned processes, including any earlier Lost history.
A native schema5 Lost→Stopped fixture documents that migration sees terminal history
and cannot infer verified recovery; preserve that limitation rather than claiming
it is an enforced universal past-tense guarantee. New schema6 Lost transitions are
absorbing and future trusted recovery remains required. No automated historical
laundering detector or synthesized recovery certificate is advertised.

Production source here means current implementation code, not a released deployment.
There is no installed/deployed schema5 release asserted by this feature. The two
public exact-source old-writer artifacts independently prove their stated builds;
a future deployed hotfix/feature-set artifact must join the compatibility matrix
with its own source tree/binary identity before supported upgrade. Never infer the
identity of an operator's binary from a commit label.


### Complete observation and failed-publication contract

Workflow-owned blockers/next_action are mutable only through the existing atomic
Workflow transition and must remain writer-pinned in generic Task updates. Operators
may request a lifecycle hold through the existing permitted WaitingHuman state
transition with these fields unchanged, or make an explicitly authorized additive
graph/lifecycle change. Accepted Goal objective, criteria, evaluators, constraints,
nonGoals and sourceRefs remain immutable under Issue23; this example does not
authorize edits to the accepted Goal definition. Task acceptance constraints remain
subject to their own authority and semantic-input fences. No
new generic directive field is excluded from semantic hashing without an explicit
owner-writer fence. Test a generic blockers/next_action update after reservation but
before adapter ScopeSnapshot capture: it rejects, source authority stays unchanged
and no new directive reaches the model. Actual Engine status synchronization and
operator lifecycle holds remain positive controls. Mutate each writer pin separately.

A rejected pending native-binding CAS is not permission to publish the in-memory
mutated Session as Failed/Lost. Each provider tracks the last successfully persisted
owned Session and attempt/version. Following rejection, it observes/rechecks that
exact same owned attempt; a newer foreign or higher-input attempt is never adopted
or overwritten. Its cleanup/terminal body preserves those persisted actor/input
pins. A privately prepared, current-input pending Starting with no admission, no
consumption and no native-dispatch uncertainty may become Failed after authoritative
owned no-dispatch cleanup, matching its preparation tuple without a live-head check.
This records failure, not new permission. Actually consumed or uncertain input keeps
its monotonic intent/uncertainty and conservative Lost ownership; restoration is
not allowed. If own Session CAS/ownership changed, retain owned process bookkeeping
and surface attention rather than clobbering another attempt. Historical binding
publication holds likewise do not silently adopt rejected Some actor fields.

Test actual Generic/Claude/Codex/Grok callers: head or lifecycle changes immediately
before pending/combined binding CAS, rejected in-memory None→Some fields, zero new
PreparedInput wire, and successful exact-persisted-pins Failed cleanup where no
dispatch occurred. Repeat with uncertainty/consumption (held), a concurrent higher
attempt (unchanged), and pending-before/after legitimate Workflow binding. These
controls use real private preparation, not seeded admission rows.

Every non-Succeeded removal/replacement of an allocated attempt checks exact owner,
terminal non-Lost/non-uncertain state and current preparation/admission pins, or the
exact RestoredPriorNotAdmitted alternative. The component active-Waiting/Failed retry example is HISTORICAL ONLY. Managed retry
requires the already closed awaiting_explicit_retry hold; escalation/invalidation and
all WorkflowAccess paths retain their actual settlement predicates.
An unbound allocated live actor is still ownership. Preserve the separate #41
claim-owner/observer-release policy; this predicate does not manufacture owner death.
Test the old component retry(active) rejects EVERY marked post-epoch native attempt.
Exact current failed preparation/frozen restoration permits the private non-success
closure, followed only later by an independently authorized closed-attempt retry.
Succeeded always uses its stronger unconditional admitted+consumed predicate.
These input/terminal predicates are necessary but insufficient: every managed
operation also needs its actual owned settlement receipt below, including before
first Session and current failed/restored preparation. No terminal label is cleanup.

### Post-consumption operation decisions and transport journal

Canonical recovery.dispatch_intent is exclusively the typed prepared-frame consumed
DTO, immutable for its (Session,input_version). Permission/tool replies and terminal
transport startup/input must not replace it. Their separately scoped diagnostic
operation journal uses recovery.operation_intent or recovery.transport_intent, with
operation ID, decision/transport kind and reply attribution. That journal is never a
credential for model-input admission, phase allocation, native ACK or recovery.

ALLOW retains the actual Broker operation authority plus full current NativeCAS
P/G/T/lock/own-Session checks immediately before wire. It may record a separate
operation intent while the input outcome is Acked, but no historical exemption may
grant ALLOW. Managed DENY does not grant authority: actual private DenyHistorical
observation through observe_managed_native requires the exact admitted actor/input
tuple and consumed UUID unchanged. Generic ObservationOnly here is historical
component5/unallocated factual context ONLY and cannot update a managed Session. Existing Project activity/Blocked, worktree and own-Session CAS/write
constraints remain; a Task metadata/version revocation can still permit the existing
owned denial path. Preserve the real provider distinction: Claude's prewire
WaitingApproval→Running publication rejects ProjectBlocked and sends no reply;
Codex Decline can publish an unchanged WaitingApproval observation and send the
fixed denial under ProjectBlocked, then its postreply Running publication rejects.
This contract adds no universal DENY privilege or new universal DENY fence.
WaitingApproval→Running after a fixed typed reply is historical Acked, not
a new prepared consumption; preserve current activity/worktree constraints.

Actual writer inventory to migrate and causally test: Claude automatic DENY and
Broker ALLOW/DENY overwrite sites, ordinary post-reply lifecycle publication,
terminal_start and terminal_input, plus any SQL test/audit consumers of those keys;
Codex answer_approval already preserves dispatch_intent but its post-reply generic
Running publication must retain the exact admitted tuple; Grok callbacks retain
the one prepared consumption and separately attributed audit/filesystem evidence.
Both pre-reply Running→WaitingApproval/pending publication and postreply Running
publication retain the exact historical Acked pair. Session.saved audit adds bounded
operation_intent/transport_intent projections without replacing dispatch_intent.
Retarget Claude SQL consumers at public f9b671f session lines2486/2493/3005/3022;
positive and negative SQL assertions must continue testing the actual decision wire.
Test stale Task metadata DENY→wire with unchanged private input pair, provider-specific
ProjectBlocked denial controls, stale ALLOW→no wire, current ALLOW→wire, after-reply historical Running,
and transport-only input→no fabricated prepared ACK/consumed replacement. Assert
byte-identical canonical consumed DTO and private allocation/pairs in every control.
No free-text protected live input becomes permitted by this journal distinction.

### Epoch, universal history and compatibility impact

Lost remains absorbing for every Scope; an already terminal Session also cannot
be relabeled Lost to fabricate new operational ownership. Legitimate initial
terminal factual/Consultant history remains allowed. Only future reviewed private
#14 recovery can change held Lost authority; no JSON terminal label is recovery.
Every generic and native new-Session INSERT rejects initial Lost, WaitingApproval
and WaitingHuman in every scope, including Project/Goal and legacy Task history.
Lost is a conservative transition of a persisted nonterminal owned Session, never
an initial record that creates an unresolvable fabricated hold. Candidate6 has no
legitimate NoTask live owner before58; Goal/Project fixtures therefore prove initial
nonterminal/Lost and terminal→Lost refusal and old live/Lost migration refusal. The
positive Goal-publication/Project-removal Lost hold fixture belongs to actual58
owned producer integration. Managed Task fixtures use the actual19 operation to
reach Lost and prove its hold. Initial terminal factual history is a separate control.

The phase-owner reader needs an explicit migration epoch to distinguish old typed
contexts without allocations from post6 absence violations. Private fenced table
context_admission_epochs(Project,Goal,Task,boundary_context_version) captures each
existing Task's largest persisted owned ContextVersion at migration (zero if none),
in the same transaction. A candidate6 AFTER INSERT trigger on the physical Task table
installs exactly one boundary-zero scoped epoch row for EVERY new Task insert, including
put_task/put_task_tx, future typed23 creation and controlled fixtures. The schema-owned trigger is the private new-Task epoch producer. The trusted exact-
version connection fence applies to both trigger and Task write; any epoch insert
failure rolls back the entire Task transaction. No individual caller may omit or
replace this row. Migration installs existing rows before enabling the new-Task trigger.
Schema goldens and every actual Task-creation path verify exactly one correct row,
plus rollback and missing-row mutation controls. Rows
are immutable, scoped by FKs and never caller JSON. Missing rows on an existing Task
are corruption, not an optional fallback. Pre6 versions ≤ boundary return
NotApplicable as read-only history, never fresh admission/binding/success. All new
typed native attempts require version > boundary plus actual allocation; old
contexts must publish a consecutive higher attempt rather than being readopted.
Include this table in every writer fence, migration schema equality and old-writer
coverage. Standalone and EvidencePort retain their separately classified scope.
SQL immutable-row triggers reject UPDATE, DELETE and INSERT OR REPLACE on epochs,
including same-value changes; the writer-version fence alone is not immutability.

Enabling serde_json float_roundtrip is an application-wide parse change, not only a
restore hash option. Inventory and golden-test all persisted JSON round-trip/equality
consumers: nested Session diagnostics, Usage estimated_cost, native owned-map
current()/resume equality and checkpoint legacy digest preservation. Compare prior
and enabled feature artifacts with fractional and large-integer fixtures. Unsupported
legacy checkpoint numeric authority refuses migration unchanged; do not silently
rewrite its digest. Verify explicit initial native UUID/startup binding with public
provider source/caller fixtures, retaining pending-vs-actual acceptance labels.

The supported old-writer matrix includes actual already-open old4 and both identified
old5 binaries: held connections/cached statements created before migration, all
public owner/Session/metadata/audit mutations and private table fence enumeration.
Fresh old-binary refusal is an additional test, not a substitute for alive writers.
All old runtimes/native services still must be explicitly drained before upgrade.

Checkpoint condensation is an artifact contract. Ongoing autonomous Workflow
checkpoint publication/progress is not claimed until #23 integrates lifecycle/source
version separation and phase checkpoint ordering; an own checkpoint head append can
legitimately invalidate a current pending phase. Deterministic condensation fixtures
do not prove that progress integration. After schema6 source/integration acceptance,
move implemented normative invariants back into the master current-state contract;
pre-code design discussion never relabels current schema5 source as schema6-ready.


### Noninteractive protected input, consumption and exact compatibility fixtures

Protected Task input supports only LaunchMode::NonInteractive. Interactive mode, a
PTY native UI and terminal_input are refused before initial reservation/Starting,
process spawn or model-visible bytes whenever the Task/context/private frame is
protected. The actual caller derives mode from LaunchRequest/native transport; no
recovery JSON declares it safe. Native private preparation preflight and every
provider entry point enforce this mode; terminal_input also independently rejects
an existing protected actor before wire. A pending unconsumed protected Starting
cannot enter WaitingHuman/WaitingApproval to open a live model UI. Native setup
without inference may run under the supported noninteractive bounded startup
contract, but unresolved interactive auth/permission UI fails setup and retains
owned cleanup, never changes input mode or forwards operator bytes. Standalone
Consultant/live terminal behavior remains outside protected frame admission and
cannot hold/adopt a protected Task's Executor reservation or produce its phase
evidence. Actual Claude launch_terminal/terminal_start/terminal_input and Generic
Interactive rejection are inventory, with zero-spawn/zero-byte controls and a
mutant at each entry-point mode check. No fabricated transport journal is acceptance.

Private admitted version/digest/consumed UUID form an all-null or all-present triple
in SQL and typed helpers, with a nonnil bounded UUID when present. Preparation
remains distinct. Every first protected Running must carry a new consumed DTO
committed in the same NativeCAS or match an already indexed exact consumption;
plain Running with null consumption rejects. Thus GenericCLI and every real/Fake
positive consumer must commit consumption before its first payload write, not
publish Running then infer and retrofit a marker. A Running→first-consume same-V
fixture rejects because the unconsumed Running creation itself rejects; consumed
Starting→historical Running is a positive control. Running/consumption atomicity
never certifies actual delivery, and no ACK/terminal JSON replaces owned native
completion evidence. SQL triple/equal-version monotonic guards and actual caller
wire ordering mutants protect this invariant.

Golden migration databases for EVERY supported v1–v5 come from exact historical
source binaries/public Store APIs or source-identified historical SQL dumps. Pin
commit/tree/binary/helper/dump hashes and record initial schema objects plus actual
legacy JSON shapes. Initial versions may not be made by changing user_version on
a newer schema. Use historical sources e51e06979f0a0a198036ef314bbeb4c8c4a2693b(v1),
5f442ab1d3075d4170fc73393fdfdd5e800b6d3c(v2),
1c44316ee5d4092c3d519782350c6463aa94269f(v3),
e622c1db435bb68f7d1fd44597b88e63bbdb3f7d(v4), and the two already identified old5
artifacts. Native fixtures assert each binary's own supported version before
production. Compare object/source hashes for historically changed objects and
verify v1 Project bodies genuinely lack later optional metadata; unchanged legacy
objects need not differ from current DDL. Then prove drain/source-aware decoding,
ordered atomic migration, data/head/digest preservation and fresh-vs-migrated
sqlite_schema equality. Relabeled current-schema fixtures are negative controls,
not legitimate compatibility evidence. Held-open old4/old5 refusal is separate.

Mandatory phase accounting includes the exact Engine prefix encoding: rule UTF-8
bytes, the single newline separator, mandatory header/facts and fixed wrapper
bytes. Count the separator once even with zero optional bytes, then independently
validate actual complete frame bytes. Exactly-1MiB and one-byte-over goldens cover
no-rule/nonempty-rule cases without raising default map budgets. Version2
standalone policy and instruction:pack_facts digests use the same explicit typed
sorted-key encoding contract as other semantic projections, independent of JSON
feature unification. Pin preserve_order and float_roundtrip feature fixtures,
source-map and publication-CAS hashes. Legacy stored hashes remain historical;
new schema6 launch authority must be version2 or explicitly republished.

Historical component5 restoration-chain controls include admitted v1→unadmitted
failed preparation v2→fresh v3→exact rollback to Failed-v2, both bound/unbound
allocated owners. This same-UUID chain is NOT a candidate6 managed acceptance path.
Canonical candidate6 proves admitted/settled v1→v2 Continue prewire failure→exact
restoration to v1 with genuine NoCurrentDispatch/full cleanup→v3 Fresh on a NEW private
UUID. Retain v1 admitted pins/checksum, never make the non-delivering v2 a Continue
predecessor. Separately, a v2 published as current failed preparation may non-success
close with genuine NCD, but likewise forces the next Fresh selector. Retained
admitted v1 pins must not be confused with current unadmitted v2; if restored body
contains an actual consumed intent it must match its retained private consumption.
The exact prior checksum, no current admission/uncertainty and immutable actor
ownership still gate restoration. No old intent is parsed as new consumption.

### Managed native settlement authority (Design16)

This is a new private producer/consumer contract, pending the immutable pre-code
design gate. Existing terminal Session labels, recovery JSON, PID absence,
transport_succeeded/status return values and generic operator observations are
not managed native settlement proof. Input admission proves input currency; a
separate adapter-owned settlement proves that the exact operation no longer owns
unresolved native Task work. Neither proves that reviewed/tested Task artifacts
meet acceptance criteria. Goal completion additionally needs its scoped current
evidence and accepted definition predicates from Issue23.

Use the following conceptual Rust interfaces; names may move internally without
changing the authority contract. Types crossing the public adapter trait are
opaque public types with private fields, no public constructor and no Serialize/
Deserialize. Store/producer helpers and receipt predicates are crate-private.

```rust
// Only the Engine's successful exact dispatch-marker transaction creates this.
pub struct ManagedPhaseLaunch { request: LaunchRequest, marker: OriginalMarkerFrame,
    continuation: ManagedContinuation /* private Fresh or validated Continue */ }
// Owned by the retained supervisor; adapter only borrows this setup surface.
pub struct ManagedSetup { /* private actor/lifetime-bound operation access */ }
// Runtime-owned supervisor identity; cannot be reconstructed from persisted JSON.
pub struct OwnedNativeOperation { /* private operation, runtime and claim pins */ }
pub struct OwnedNativeSettlement { /* private supervisor outcome and captured pins */ }

// Additive adapter entry point: default Unsupported; legacy start is not a fallback.
fn start_managed<'a>(
    &'a self, launch: ManagedPhaseLaunch, setup: &'a mut ManagedSetup,
) -> AdapterFuture<'a, Session>;

// One Immediate transaction: exact owned observation + receipt + bounded audit.
pub(crate) fn publish_native_settlement(
    &mut self, proof: &OwnedNativeSettlement,
) -> Result<SettlementReceiptId>;

// Exact supervisor-owned nonterminal/uncertain observation, never settlement.
pub(crate) fn observe_managed_native(
    &mut self, operation: &OwnedNativeOperation, observation: OwnedNativeObservation,
) -> Result<OwnedObservationVersion>;

// Pure reader of this transaction; never observes processes or grants admission.
pub(super) fn validate_phase_settlement_tx(
    tx: &Transaction<'_>, scope: &Scope, context_version: u64,
    attempt: &PhaseAttemptIdentity, session_id: Option<SessionId>,
    requirement: SettlementRequirement,
) -> Result<ValidatedNativeSettlement>;
```

PhaseAttemptIdentity is the immutable exact Workflow Record ID, generation,
history index, phase and context version. Its original successful reservation
Record-version token is retained for the separate Issue41 invocation-owner
release policy; later dispatch/binding Record versions do not change this attempt
identity. A losing reserve invocation never receives a managed operation handle.
The operation insertion and dispatch_started marker commit together with exact
P/G/T/Workflow/lock-set CAS, after actual implementation-owned admission/settlement
capability validation and before adapter startup. There is no IO under this
transaction. The context must be post-epoch typed native authority. No operation
exists for EvidencePort or historical/unmanaged Consultant records.
These are Store invariants, not just promises of Engine control flow. Ordinary
Workflow writes cannot set a post-epoch native marker without the dedicated private
managed-dispatch transaction. A false→true dispatch_started transition inserts
exactly one scoped operation in that transaction; operation uniqueness covers
(Workflow Record,generation,index,phase,context_version). Marked native attempts
must reference that exact operation on every later Store write. Preparation,
allocation and admission require this existing unsettled operation AND the actual nonserializable owned operation
handle/runtime identity, and bind its Session once in the same transaction; missing/settled/foreign operation rejects.
Allocation present with missing operation is corruption, never unmanaged fallback.
Every marker/allocation/operation row rolls back together on Session/audit/CAS error.

The private native_phase_operations row contains operation UUID, runtime-instance
UUID, exact P/G/T, Workflow Record ID, immutable attempt fields and successful
reservation token, context version, full authoritative payload SHA256/bytes,
revision and canonical complete source-map digest. It also pins registered agent,
implementation-owned provider family/profile-contract digest, role, exact worktree,
launch mode and requested model/effort. They derive from the validated runtime
selection and exact managed request, never from returned Session JSON. Explicit
requested Some pins remain exact; the separately validated initial None→effective
Some binding belongs to the private input pair and never changes requested pins.
It references the immutable
ContextVersion rather than duplicating prompt/source text. Its optional Session ID
is bound once in the same first private preparation/allocation transaction. This
operation exists before the first Session: an adapter error or dropped future with
no Session does not make a dispatch-marked operation absent. It remains an owned
lease until the actual tracked startup invocation settles. All Task/Project idle,
pack force/publication, removal and fresh-phase admission checks include outstanding
operations, so a generic forged terminal label cannot release the same worktree.
The row additionally has a monotonic operation version, its current privately
persisted Session Record version when bound, and phase_closed=false. Native private
observation/admission updates the remembered Session/operation versions together.
The only later phase_closed false→true update is the successful exact receipt-checked
Workflow closure transaction; immutable identity/frame/claim fields never change.
Receipts settle native ownership but do not silently close an active Workflow claim.
Both unsettled operation leases and receipt-pending open phase ownership are exposed
distinctly in status. Fresh native entry does not interpret either as legacy history.

All generic Session/Record writers, including public scoped CAS paths with ordinary
ObservationOnly access, reject every write to a Session bound to an open managed
operation, even an identical body that would bump its Record version. This includes
state, diagnostics, actor fields, recovery journals, terminal observations,
RestoredPrior and Lost updates. Separate attributed Usage/audit do not change native
authority. Private managed entries are bounded and charged as Design30 specifies;
generic application-wide storage is outside the per-operation authority bound.
The actual supervisor uses observe_managed_native for nonterminal historical Acked
observations, pending permission publication, fixed DENY reentry and
monotonic uncertain/Lost diagnostics. This private nonserializable observation is
bound to the exact operation/Session/remembered versions and unchanged prepared or
admitted actor/input pins. It cannot create admission, consumption, allocation,
terminal restoration or settlement. Those use their dedicated private ports.
Post-consumption ALLOW uses the explicit handle-bound AllowCurrent branch with
full-current scoped NativeCAS/Broker before wire (Design26 below); private historical DENY
preserves each provider's existing activity/write constraints and
does not grant authority. No broad ProjectBlocked/lifecycle exemption is added.
All five managed writers migrate every Session publication, not just first admission.

Operation-aware reservation checks are mandatory independently of labels: worktree
lock acquisition, validate_worktree_exclusion, every executor_reserved/Lost consumer,
Git worktree create/adopt/remove/ownership validation, Task/Project idle and registry
remove, context prepare/force/publication, competing native admission,
Workflow bind/close/replace and Goal terminal/completion surfaces. Native observation
does not remove an operation lease. A lock cannot be acquired around a managed
Executor startup merely because someone attempted to restore an Exited body.
Inventory source call sites exhaustively; pure session-label helpers cannot remain
the sole authority for a scoped ownership decision. Actual FS/Git/cleanup stays
outside SharedStore; transaction readers use the durable operation facts.

The marker-committing Engine/runtime must synchronously move the operation handle
into an actual spawned owned supervisor BEFORE the first await after marker commit,
including before polling start_managed. The supervisor performs async startup work;
its ownership is independent of the dropped Engine step future. No adapter may await
a probe before this transfer. Panic/runtime teardown remains conservative durable
attention, not receipt proof. A cancellation-at-the-first-await control proves that
the actual supervisor can finish its pristine refusal/settlement after caller Drop.
The operation handle transfers into the actual owned supervisor before startup
work. It is not Clone; any internal shared bookkeeping stays private and cannot
produce more than one terminal receipt. A dropped start future, returned error,
timeout or public release call does not mint a proof. The owned supervisor retains
the handle/process/native-turn cleanup responsibility until it can publish proof
or durable attention; publishing failure retains the owned bookkeeping. Before
any Session/process/native connection, a supervisor may certify NoCurrentDispatch
only from its tracked invocation's actual no-dispatch/settled-setup state. Absence
of Session, PID or allocation is insufficient. After preparation, this class also
requires no current private admitted/consumed input and no dispatch uncertainty.
Dropping the handle before supervisor transfer or while unsettled may enqueue a
bounded in-memory abandonment hint only; a separate actual runtime worker may later
emit rrx.private.operation.abandoned. Drop emits no Store/SQLite write, receipt or release. Lock poisoning/publication failure cannot erase the operation;
status derives durable attention from the outstanding operation even if the Drop
hint/audit fails or the bounded queue is full. A cancelled-before-transfer controlled
case proves this path.

KnownCurrentTerminal requires the exact privately owned current native outcome,
known end of Task inference and completed owned resource cleanup/settlement.
Unknown outcomes, Lost, partial/unobserved input, interruption acknowledgements
without authoritative terminal outcome, escaped/uncertain process groups and
unsettled setup do not produce a receipt. The narrow managed MVP policy requires
actual owned server cleanup before a releasable Codex receipt: a known phase result
may be observed while the service remains owned, but an idle retained server never
releases operation, Goal/removal or capacity. The current public Codex674 producer
already calls native.shutdown before terminal publication. A fresh owned server may
later resume the retained native thread UUID with fresh exact admission. Reusing
an idle live server across released phases is outside this contract and would need
a separately reviewed durable service lease; no service-lease table is added here.
Providers requiring group cleanup must complete it; a dead leader/PID or selected
process-group Dead alone is insufficient for escaped command descendants. Actual
owned tool/setup/decision tasks must be settled, with no active or pending relevant
operations. Native6's verified escaped-command containment/interrupt limitation is
a real producer readiness gate: an unsupported profile or unresolved escape cannot
advertise managed settlement capability or issue a receipt. This is an owned native
operation contract, not an OS same-UID sandbox or arbitrary descendant-death theorem. The receipt
records the actual settlement class and bounded outcome attribution, never a
uniform fabricated exit zero or universal process-death claim.

native_phase_settlements is append-only, one receipt per operation. It contains
receipt/operation UUIDs, exact attempt/scope/context/frame pins, optional exact
Session ID and persisted Record version, private preparation version/digest,
optional admitted version/digest/consumed UUID as an all-present/all-null triple,
optional frozen prior-restore digest, settlement class and known outcome,
bounded native turn/ref attribution, runtime-instance UUID and observed timestamp.
Fixed fields plus bounded attribution fit 64 KiB; refs are at most 8192 UTF-8 bytes,
reason at most 8192 bytes. It contains no model transcript, prompt, environment or
JSON credential. Receipts accept NO UPDATE, including Session NULL→Some; only
operation rows have the separately constrained one-time allocator binding.
Receipts and epoch rows reject UPDATE, DELETE and INSERT OR REPLACE; inserts require
their private producer and one scoped unique identity, in addition to every
table's v6 writer fence. One receipt is not a new native input/permission grant.
The receipt also freezes canonical complete terminal Session-body SHA256 and the
exact resulting persisted Record version. From receipt publication until that
exact phase closure, ALL Session-body writes are rejected, including private
terminal diagnostics/relabels, generic same-body updates and fresh continuation.
Usage/audit are separate. Closure requires current Record version == receipt version,
complete body checksum == receipt checksum and current applicable preparation/
admission/restore pins == receipt pins. Version equality is explicit, not inferred
from an Exited label. After successful phase_closed=true, immutable receipt/history
remain frozen but a legitimate higher-input continuation can use a new operation.
Historical receipts are not compared to that later current Session body.

Operation/receipt source digests use SHA256 over explicit compact UTF-8 typed
encoding with domain rrx.native-phase-source-map and encoding_version=2, exact
sorted-key source map and explicit scope/revision. No normalization, arbitrary
Value key ordering or floating fields enter this encoding. Operation/frame identity
encoding is domain rrx.native-phase-operation version2 with exact typed fields;
complete restored Session bodies use the existing canonical restore encoder.
The golden projection/schema pair includes these domains and versions; an authority
encoding change requires the reviewed projection and writer-schema bump together.

Settlement publication performs the exact remembered operation/Session Record
version and input/actor/preparation/admission/consumed pins CAS, then publishes the
actual terminal observation and receipt atomically. The private port is the sole
managed terminal/RestoredPrior writer and permits those only after actual tracked
settlement proof. Conservative Lost observation remains private and yields no receipt.
A no-Session receipt requires
no allocator owner and the actual tracked no-dispatch proof. A generic concurrent
terminal write or newer/foreign attempt cannot be adopted; conflicts retain the
operation and require attention/recovery. Publication may record actual cleanup
under a paused/blocked owner without granting input or altering its lifecycle:
this is a separate private factual observation port, not a general BoundHistorical
CAS bypass. A Lost persisted owner remains held; late cleanup can be separately
audited but cannot settle/release it until reviewed private Issue14 recovery.

An older terminal restored after failed fresh preparation is still NotAdmitted.
Its closure requires the new operation's actual NoCurrentDispatch receipt, the
frozen new preparation and exact prior-terminal checksum. An older Exited label
or older operation receipt never certifies the new attempt. Success requires
KnownCurrentTerminal with known successful native outcome plus exact current
admitted/consumed pins and Exited owner; NoCurrentDispatch cannot certify success.
Failed/Interrupted replacement requires its applicable receipt and existing
metadata/claim predicates. A receipt does not erase a definitive terminal decision
or unknown external side effect. Already closed historical receipts retain frozen
provenance when that Session UUID later has a legitimate higher-input attempt;
new-open-attempt validation never substitutes a historical receipt for current pins.

| Exact managed phase state | Allowed closure/replacement |
| --- | --- |
| Pre-marker, no operation | Separately reviewed Issue41 invocation-owner rule only; never receipt fiction |
| Post-marker, no Session/allocation, matching NoCurrentDispatch receipt | Failed/Interrupted non-success closure or TerminalRecovery; managed retry is later closed-attempt-only under Design32, never active replacement; same-tx exact operation/claim/frame and no owner. Succeeded impossible |
| Post-marker, no Session/allocation, no receipt | Held for actual supervisor settlement or private Issue14 recovery |
| Allocated current failed preparation with NoCurrentDispatch receipt | Non-success closure/TerminalRecovery with exact frozen preparation/terminal receipt body; unbound session_id may stay None |
| Allocated older restored terminal with NoCurrentDispatch receipt | RestoredPriorNotAdmitted failure/interruption/TerminalRecovery only; exact frozen checksum and no current admitted/consumed input; bound session_id retained unchanged OR unbound session_id stays None; never clear/rebind |
| Allocated known current terminal receipt | Exact active current unbound known failure may use separate KnownCurrentFailureClosure without SessionID write (Design27); Cancelled/Failed Task TerminalRecovery remains separate; Succeeded additionally bound Exited + actual successful outcome + exact current admitted/consumed pair |
| Missing operation for marked/allocated post-epoch native context, stale/foreign receipt, Lost/uncertain owner | Corruption/hold; never generic label fallback |

Both no-Session and allocated-but-unbound receipt rows update non-success closure and
TerminalRecovery's old session_id.is_some() || !dispatch_started check. Managed retry
then consumes ONLY the closed awaiting_explicit_retry hold, never the active entry. The sole
non-success alternative derives exact allocation (or its proved absence), operation,
current receipt/body/version and input/actor pins in that same transaction. It may
release a Cancelled/Failed decision under inactive owners after actual owned cleanup,
leaving session_id None; it neither binds a Session nor grants dispatch/success.
Allocated NoCurrentDispatch requires tracked no current admit/consume/uncertainty;
KnownCurrentTerminal requires an actual owned current outcome and complete cleanup,
not cancellation bookkeeping. Stale/foreign/missing proof or Lost retains the claim. No automatic
ordinary-observer release is added. Original #41 pre-marker reserve-owner policy
is not a substitute or prohibition for this post-marker managed receipt row.

The same transaction predicate is mandatory for every allocated/managed native
attempt removal, replacement or operational release under every WorkflowAccess:
ordinary completion/failure, retry (including Waiting), escalate/invalidate,
TerminalRecovery/release_terminal_reservation and source-drift generation changes.
It also gates operation-aware Task idle/new preparation, Project removal and
Issue23 managed Goal completion/evidence extraction. Binding uses Issue43's
separate exact owner/preparation/admission predicate; settlement cannot fabricate
binding or authorize new dispatch. Ordinary poll remains read-only. Issue41
pre-marker release still requires that invocation's exact successful reserve token
and unchanged committed Record version; this receipt does not weaken it. A
post-marker startup failure needs actual tracked settlement, including before a
first Session. Unknown external EvidencePort gates keep their separate reconciliation.
Generic put_goal is explicitly inventoried: Completed is unavailable for managed
Goals through a generic row update and needs the actual private Issue23 completion
consumer with current accepted policy/criteria/gates, exact required receipts and
no outstanding operations. Cancelled/Failed lifecycle dispositions can revoke
admission while native operations still exist, but never release their claims,
locks/capacity or certify native completion. The owned factual cleanup receipt
may still publish without altering that terminal Goal disposition. Issue23's
managed success extractor remains a separate actual integration gate.

After restart, durable receipts remain usable as scoped immutable historical proof.
An outstanding operation from another runtime is unknown even if its Session is
terminal; no reconstructible JSON token recreates live native ownership. Issue14
must supply actual private recovery/cleanup and an exact operation/attempt/epoch
CAS before settling such ownership. Issue23's managed completion stays unavailable
until this real producer and consumer compose; no fake SQL acceptance earns credit.
Drain migration refuses outstanding managed operations in future upgrade paths;
the initial v5→v6 migration creates none for old history and never backfills native
settlement from labels. Preserve the already declared operator-attested old-runtime
upgrade limitation rather than retroactively claiming older proof.

Actual integration inventory is GenericCliAdapter's child/group supervisor,
Claude print supervisor and startup cleanup, Codex owned turn completion/setup
cleanup (actual server shutdown and owned tool settlement), Grok Actor's native protocol/owned group
settlement and failures before first Session. Each uses the actual private managed
entry point and produces receipts only after its owned lifecycle condition. Positive
FakeAgent uses a controlled owned transport/task with real preparation/admission
and settlement callbacks; neither a raw seeded receipt nor terminal JSON passes.
Its post-preparation/pre-return barrier independently exercises Issue43 binding CAS.

Readiness is explicit per provider/profile. Arbitrary Generic CLI and unproven
tool-running native profiles do not advertise managed admission/settlement and
reject before clear_hold/reserve/context/marker/Session/process/model input, with
no legacy start fallback. Their tested unsupported negative path proves correct
composition, not a positive receipt or native runtime readiness. Native6 F1's
actual detached heartbeat surviving native terminate plus stdio-server closure is
an unresolved concrete negative; selected group Dead does not weaken it. Synthetic
scoped no-subprocess peers, identified by reviewed controlled fixture code and
declared no-subprocess contract, may prove producer/Store/consumer mechanics but
are labelled synthetic and never promoted to native production containment proof.

A production positive profile must have reviewed implementation-owned operation
cohort/cleanup authority and actual causal evidence for its stated tools/setup/native
tasks: admission→tracked cohort→known outcome→all relevant owned task/resource
settlement→receipt. Unknown/untracked escaped work is unsupported/held. The profile's
cohort, concrete cleanup primitive, permission boundary and residual visibility are
public acceptance artifacts before enabling the capability; a configuration label,
native terminal/list/cleanup ACK or caller PID hint cannot supply it. This design
does not invent a sandbox/OS mechanism or claim current native6 can satisfy F1.
Final real native Executor/MVP readiness remains gated on that actual containment
correction and Issue16 evidence. Report which exact profiles can issue success
receipts and which refuse; Issue19's deterministic acceptance cannot close these
separate native/MVP obligations or claim all production profiles positive.

For each enabled profile, acceptance and causal mutants cover actual callers;
unsupported profiles instead prove the pre-marker refusal and remain unready.
Positive mechanics use real controlled owned producers, never seeded rows:
no-Session startup
failure, failed setup after preparation, partial/uncertain dispatch (held), current
known terminal (receipt), prior restored terminal (non-success only), consumed
interruption with no authoritative terminal (held), publication failure (retain
supervisor), generic forged terminal (all release/replacement paths reject),
foreign/higher-input attempt (unchanged), paused factual settlement (no new grant),
Lost late terminal (held), and old receipt replay against a fresh attempt (reject).
Each predicate mutant must reach the real consumer; SQL row seeding or an earlier
startup rejection cannot prove a later guard. Protected terminal_input uses a
reachable unprotected-era Interactive Consultant now protected by a ReadOnly frame
or a narrow actual entry guard seam, with an unchanged unprotected positive control.
Additional cases: live setup→generic exact restoration/terminal/diagnostic attempt
rejects and cannot acquire a lock; subsequent real settlement succeeds. Post-receipt
generic/private Session body updates reject until exact phase closure while Usage/
audit remain recordable. Independently remove marker/operation, allocator/operation,
operation-aware lock/executor checks, receipt Session-version/body pin and closure
row predicates; record killed and redundant-guard survivors honestly. Allocation-
without-operation is corruption, never a permissive unmanaged path.

### Remaining capture and scope precision

Standalone rendered Goal bookkeeping is explicitly captured status/as-of metadata,
not a live directive; its omission from semantic authority cannot authorize stale
instructions. Sibling-progress tolerance here means Store semantic frame validity;
actual native adapters' full raw P/G/T version CAS may still hold publication and
must not be advertised as automatic native liveness. One-delivery is per exact
(Session UUID,input_version) for supported managed input. Protected standalone
native launch is Unsupported under schema6; Workflow context allocation remains
single-owner. Standalone artifact preparation never creates an operation lease.
Every new Project/Goal-only Session uses the exact registered
canonical primary Project.root path and cannot name a Task worktree or worktree
namespace. Compare stored canonical paths and scope under Store; actual FD/Git/
filesystem ownership checks stay bounded outside SharedStore. Generic updates pin
that worktree; legacy history remains read-only rather than silently rebound.


### Design18: minimum supported launch boundary and settlement precision

These refinements govern the proposed schema6 contract and supersede earlier
schema5 standalone launch/retry examples. No schema6 source or producer readiness
is claimed by this design document.

**Protected standalone is an artifact path, not a native operation path.**
A Task input protected by typed pack ancestry/private prepared-frame authority
cannot enter standalone native launch/resume/checkpoint dispatch. All actual
Generic/Claude/Codex/Grok entry points and private admission paths return explicit
Unsupported before reservation, Starting publication, native connection, process
creation or model-visible bytes unless invoked through the exact managed operation
entry point. No caller-selected role, UUID, legacy start, missing operation or
ReadOnly alias bypasses this check. prepare_task can still publish/return a bounded
non-launchable draft artifact; it does not acquire a retained native operation.
Historical component standalone positives from native5/6 remain source evidence
for those components, not schema6 production admission/cleanup proof. Unprotected
legacy factual history remains explicitly separate; migration drains old live/Lost
owners and does not silently re-adopt them. A protected generic Session publication
cannot create live native authority or relabel a terminal record to release work.
A future standalone lease producer would require its own approved contract.

Positive managed profiles retain all capability, exact frame, lifecycle, ownership
and native cleanup requirements. It is not acceptable to pass a production migration
or MVP gate by rejecting every real native path. Until the required activated Core
production profiles have real reviewed managed producers and causal positive
admission/outcome/cleanup/receipt proofs, schema6 is a nondeployable integration
candidate. Normal production release/automatic DB upgrade remains gated; isolated
synthetic schema/migration tests prove mechanics only. Release evidence names the
exact supported profiles and required configurations, including the actual native
review/Executor paths required by the MVP. Unsupported positives cannot replace
them. This is a release/deployment gate, not a new config boolean or SQL credential
that enables an unproven profile. Managed Goal dispatch/completion additionally
waits for the actual Issue23 accepted-definition/admission/completion consumer;
component completion or historical terminal labels do not establish that consumer.
Do not advertise the schema6 production path in README before these gates close.

**Factual cleanup survives lifecycle cancellation without granting input.**
Private settlement may publish an exact owned NoCurrentDispatch or
KnownCurrentTerminal receipt under Task WaitingHuman or any nonterminal Workflow
lifecycle hold, Task Cancelled/Failed, Goal Paused/Blocked/
Cancelled/Failed, or Project Blocked. These states prohibit new launch/admission;
they do not prevent recording completed owned cleanup. The receipt preserves the
existing Task/Goal/Project lifecycle. Factual settlement does NOT apply launch
admission lifecycle/P/G/T version currency; it still requires exact retained scoped
operation/Session/input/actor identity and full factual cleanup proof. It requires exact Session
Record/version, immutable frame/actor and applicable input pair/checksum. A removed
Project is rejected (removal must already require no open ownership). Lost remains
absorbing/held for Issue14. No receipt alone completes accepted Task/Goal criteria,
changes a cancelled Task to success, or grants operation permission. Test actual
owned Task cancellation followed by cleanup receipt and exact claim release, plus
rejected native admission and unchanged lifecycle; a lifecycle-predicate mutant
must reach this factual producer rather than fail earlier setup.

**Pre-effect refusal is an owned transition, not an Err-shaped certificate.**
All fallible pure Engine validation runs before managed marker/operation commit
where possible. After that commit, the trusted producer initially owns a private
Pristine operation handle. Before its first Session publication, process creation,
native connection or other owned effect, it durably transitions effects_started
false→true in the operation transaction and consumes Pristine into the supervisor's
effect-owning state. External Git/process startup is an effect requiring tracked
cleanup; it is not a pure validation exemption. A private consuming
settle_refused_before_effect(Pristine, reason) can atomically publish the no-Session
NoCurrentDispatch receipt only when that exact handle has never crossed the effect
boundary and the transaction also confirms no Session/allocation/admission or
consumption. The actual synchronous validation/refusal sites use this port; they
must not silently drop a committed operation. The durable false flag alone cannot
mint proof. A stale/foreign handle, public error JSON or dropped future cannot call
this transition. After effects_started, every failure goes through the actual
tracked supervisor cleanup/outcome; uncertainty remains held. Drop still records
attention without release. Exercise pure validation failure immediately after
marker commit, successful receipt/non-success closure, and effect-started failure
held until actual cleanup. Marker failure rolls back the lease and needs no receipt.

**Operation SQL constrains each allowed mutation independently of code.**
All identity/frame/scope/claim/agent/profile/requested fields are immutable.
Session NULL→Some occurs once, in the private allocator transaction, and requires
its matching exact phase_session_owners row; it cannot be cleared or changed.
Operation version strictly increases on each real allowed update. Remembered
Session Record version is NULL before binding, then present and strictly increases
when that value changes; an update of another field cannot clear/decrease it.
effects_started changes only false→true. phase_closed changes only false→true with
a matching receipt and exact Workflow closure in that same transaction; receipt
publication alone never closes the phase. No UPDATE can rewrite immutable fields,
reopen a closed phase or detach its Session. BEFORE INSERT rejects an existing ID
or natural attempt key, including INSERT OR REPLACE with recursive_triggers OFF;
DELETE rejects. Equivalent no-replace fences apply to admission/allocation/epoch/
receipt authority. Canonical fresh/migrated DDL goldens and raw SQL mutants cover
all columns/transitions, not just immutable identity or ordinary API updates.

Every post-epoch native history entry must have dispatch_started iff it has exactly
one matching durable operation. Ordinary insertion of a new already-marked entry,
false→true and true→false are all rejected outside the dedicated transaction.
Later operation settlement/phase closure preserves marker and immutable history.
A marked entry without its exact operation, or an operation with an unmarked/
foreign entry, is corruption and holds. Test all three ordinary writer shapes and
operation insert/marker/audit rollback; never infer validity just from current active.

**Managed Goal completion has durable classification.**
The transaction that handles generic Goal terminal publication identifies managed
native ancestry from owned post-epoch Workflow context/history or retained scoped
operation history, including settled/closed operations. A Workflow context/history
reference counts only with its exact owned ContextVersion and Task epoch; malformed
or contradictory references refuse. Epoch rows alone do not classify legacy Goals,
since all old Tasks receive a boundary. A Workflow's post-boundary context counts
before its first native marker; finished history counts after its last operation
closes. Removing active/outstanding operations cannot make that Goal generic again.
Generic Goal Completed cannot bypass this predicate by observing a between-phases
window, adding/removing DAG membership or presenting terminal Task labels.

When Issue23's actual goal_authority table composes, its accepted scoped definition
row is an additional durable managed-Goal predicate even before Workflow creation.
That row attests acceptance; it does not duplicate or permit mutation of the Goal
snapshot definition. Missing accepted authority makes a Goal definition-unverified,
never managed ready/complete, exactly as approved Issue23 specifies. The integrated
Issue23 typed Goal ports reject every generic snapshot change and require trusted
lifecycle authority, while Issue19 never invents a Human/creation authority or
allows generic Cancel/Fail to evade that contract. A factual native receipt can
publish after an authorized lifecycle hold/cancel/fail without rewriting that
accepted definition or terminal decision. Acceptance covers before-first-marker,
between phases, all-closed history, actual accepted authority before Workflow,
legacy unverified histories, and malformed references. The completion producer
requires exact required receipts, no outstanding/open operations and current accepted
criteria/evidence/policy; durable classification is necessary, not sufficient.

**Publication and observation are distinguished from new dispatch.**
During an open managed phase, capture bounded attributable condensation events as
staged evidence ONLY; do not append a same-Task checkpoint or advance its live head.
Publish the checkpoint through current atomic head/source/semantic CAS only after
genuine native settlement AND exact phase_closed. Admitted frame/pair/history stays
immutable, so own staging cannot self-revoke AllowCurrent/HostCallbackCurrent.
New preparation still requires the latest genuinely published checkpoint head.
Design36 supplies the actual stage/publish consumer controls; earlier live-head
append positives are historical component5 evidence, not managed acceptance.

Private historical observation mirrors the existing target-state activity check.
Registered Project normally permits it. For Blocked Project, only the existing
safe metadata-preserving WaitingHuman/WaitingApproval or monotonic Lost observation
is available: exact actor/input/recovery remain pinned, except permitted PID clearing
and conservative native_dispatch_unobserved addition. A Running target is not a
safe update. Thus Codex Decline's unchanged Waiting publication can precede its fixed
wire reply under Blocked, but its later Running write rejects; Claude's prewire
Running write rejects before reply. Other Goal/Task/worktree/own-Session constraints
remain in force. No private historical port confers ALLOW or native input authority.
Terminal factual cleanup uses the separate settlement table above, not this port.

Issue43 late record-only Session binding can succeed after an exact receipt freezes
the current owner: match the private allocation/frame/pair and receipt's Session
version/body checksum in the binding transaction, with its existing full P/G/T,
Workflow Record, lifecycle and lock-set CAS. It changes no Session body/version and
therefore does not violate receipt freeze. An older restored terminal is still
NotAdmitted/nonbinding. Current success requires exact admitted/consumed pins and
actual successful terminal receipt; Pending does not claim delivery. Exercise actual
start-return/binder-CAS delay while supervisor settles, then fresh record-only
binding and closure; stale/foreign/restore cases remain held.


### Design19: managed continuation and admission possession

This is a refinement of Design18, not schema6 source. ManagedPhaseLaunch has a
private continuation selector: Fresh, or Continue with a transaction-validated
prior Session/closed-operation identity. Public LaunchRequest JSON cannot set it.
The Engine selects Continue only from the exact closed prior native attempt of
this Task/Workflow, not from a user-provided UUID/native_ref or an adapter registry
search by loosely matching scope. Fresh chooses a private unexposed UUID immediately before the first preparation
transaction; Session INSERT, allocator and operation Session binding commit together.
No Session UUID is reserved/exposed in marker/audit/descriptor before that transaction.
Any existing factual collision makes private allocation fail before Session/native
input and requires actual tracked setup settlement, never adopts the factual body.
After allocation every generic INSERT/UPDATE of that UUID is fenced; test concurrent
factual INSERT versus allocation commits exactly one, never false managed provenance. Continue
is optional per actual production profile; unsupported continuation rejects before
new marker/operation, while its supported Fresh path remains available.

The marker/operation transaction for Continue validates the persisted prior Session
as terminal, non-Lost and non-uncertain, its latest managed operation phase_closed,
and that operation's exact immutable receipt against the current complete Session
body checksum/Record version and applicable pair. Scope, registered agent, native
family/profile, role and exact worktree must match the new request/operation; a
role-changing next phase uses Fresh. The new input/context version is strictly
higher. The sealed continuation descriptor contains exact prior identities/version/
checksum and receipt, not native permission JSON. Actual adapter startup additionally
checks its private owned registry/current Session/native UUID before any effect.
A historical unowned native_ref or older receipt cannot be resumed. A service/server
already cleaned up may reconnect the retained privately owned native thread UUID;
no idle-service resource release is implied.

Terminal→Starting occurs only through this Continue descriptor and the new owned
operation's private admission transaction. It binds the new operation's Session
None→the exact prior UUID, installs fresh requested model/effort and full frame/
source/bytes/hash, preserves its privately owned known native_ref, records exact
prior canonical restore checksum, and clears prior dispatch uncertainty/consumed
markers only for this genuinely fresh input. The old receipt/history remains
immutable; it is no longer compared to the new current body. The new operation
pins its own preparation/admitted tuple. Before new consumption, actual tracked
prewire failure can atomically restore that exact old terminal body with the new
operation's NoCurrentDispatch receipt; it is RestoredPriorNotAdmitted for this
attempt, never a delivery/success claim. Receipt-based non-success closure frees
the new claim only after exact cleanup. Post-consumption rollback is forbidden.
All protected legacy resume/checkpoint/terminal-to-Starting callers without this
sealed managed entry remain Unsupported. Causal managed consumer fixtures cover
same-UUID fresh Starting, successful fresh consumption, failed prewire exact restore,
foreign/unowned UUID refusal, changed receipt/body refusal and new-role Fresh.
Earlier component resume fixtures alone do not prove this actual managed producer.

Post-epoch managed preparation, allocation, first admission, consumed publication
and actor binding require &OwnedNativeOperation (or a private nonserializable token
borrowed from that same handle). Generic put_session_if_current with ordinary
ScopeSnapshot/lock CAS is not that authority. In the same transaction, check exact
operation ID/runtime-instance, immutable claim/context/frame pins, unsettled/open
state and effects_started=true for any Session write, then current P/G/T/lock CAS
and the applicable full input/head predicate. Exact Session agent/provider/profile,
role/worktree/mode and explicit requested model/effort match operation authority;
only the separately validated monotonic effective binding may differ from None.
A second adapter instance or restarted runtime without the handle cannot take its
Session allocation or commit consumption merely because the row exists. Provider
capability declarations, scope versions and recovery JSON do not reconstruct the
handle. Observe/settlement ports check the same identity/remembered versions.
Mutants must reach the actual preparation and final-wire consumers without a handle
and reject before authority publication/input, with the real handle positive.

NativeCAS BoundHistorical can only publish nonterminal managed targets. If a known
native terminal reports its first permissible None→Some model/effort/native_ref,
the actual private settlement transaction performs that same monotonic predicate
using the old exact admitted digest, refreshes applicable preparation/admitted actor
digests without new consumption, and pins the resulting actor/body/digest in the
receipt atomically. Explicit Some cannot change; lost/foreign actor binding rejects.
Actual owned terminal outcome/cleanup is still required. A terminal NativeCAS write
without settlement is forbidden even if its actor binding alone is valid. Test
late-ref terminal binding+receipt positive and no-receipt BoundHistorical→terminal
negative through the real consumer, not generic terminal JSON.

Schema6 refuses all unmanaged Task-worktree native Executor/Reviewer launches,
including not-yet-typed Tasks, before reservation/Starting/native connection/process.
Workflow initialization cannot convert a post-boundary unmanaged native operation
into trusted history or silently rely on its Exited label. No generic/public history
write creates such launch authority. Legacy pre-migration native resource drain
retains the documented old-runtime/operator-attested limit; factual Consultant
history is not a cleanup certificate. This closes the untyped-launch→label-release→
managed-worktree sequence. Test actual legacy entry rejection on an untyped Task
with a managed positive control; no escape/worktree ownership is inferred from
historical terminal facts. Project/Goal Consultant factual scope remains primary
Project.root, never the Task worktree namespace. Native operation escape proof
continues to gate the managed profile as Design18 specifies.

Pristine handle cancellation/drop or a crash before effects_started remains durable
attention/held ownership until the actual private Issue14 recovery port. Durable
false alone is not proof while an old runtime could later cross the effect boundary.
This design deliberately adds no generic revocation/quarantine or Drop-issued
receipt. The real synchronous pristine refusal port remains available to the actual
still-owned producer; cancellation-before-transfer and restart-orphan cases prove
held status, retained resource/worktree ownership and migration refusal. Production
deployment additionally requires the actual reviewed Issue14 owned recovery handoff
for these restart-unknown/Lost/pristine-orphan operations, with exact runtime/
operation/attempt/epoch CAS and actual native cleanup or safely revoked no-effect
ownership. That is an explicit release prerequisite, not completed Issue19 source
or an out-of-scope fake receipt. No DB-edit, Human opinion, terminal label or late
selected-group cleanup releases them. Component tests can prove this conservative
boundary; they cannot claim production availability before recovery exists.

Engine poll/evaluate uses the private receipt class/outcome reader to select native
phase outcome: NoCurrentDispatch/restored prior maps only to Failed/Interrupted;
KnownCurrentTerminal known success plus current admitted/consumed Exited is necessary
but NOT sufficient for Succeeded. Exact phase PhaseGates Passed evidence and durable
owned result artifact are additionally required (Design29). It never selects success solely from a restored Exited label. Missing/
unknown receipt stays held. Test restored-Exited full Engine progression to the
non-success closure, not merely a Store rejection looping forever.

SQL enforces relational marker↔exact operation and allowed column changes, not
which Rust function was called. Nonserializable authority and absence of a public
producer enforce code-path ownership. Exact closure statement order in one Immediate
transaction is Workflow Record update first, then operation phase_closed update;
the latter trigger checks the matching scoped Workflow history index/generation/
phase/context is closed, active is not that index, and exact receipt exists. A failed
second statement rolls back Workflow/audit/operation together. Concrete JSON paths/
variant tags are pinned by canonical schema goldens. Active None alone or any receipt
for another entry is insufficient. Reversed-order and weakened-index/receipt mutants
exercise the actual multi-statement consumer.


### Design20: Store launch fence, locks and deliberate migration

The shared Store record helper rejects ANY generic nonterminal Task-scope Session
INSERT or UPDATE, regardless of public role, typed ancestry or recovery keys.
This includes Consultant/ApprovalReviewer on a Task, not only Executor/Reviewer.
Generic scoped CAS without the actual private managed operation handle also rejects.
Only the real operation-bound managed port can create/advance Task native ownership;
its allowed actor remains the exact typed phase's Executor/Reviewer. Initial terminal
Consultant/factual history remains unverified observation, never a launch/cleanup
credential. Every actual unmanaged Task native entry likewise returns Unsupported
before reservation/Starting/connection/process/bytes, regardless of role or path.
A public Consultant label is not read-only/hook/filesystem proof. There is no Task
Consultant exception through Project-only scope or a copied ReadOnly frame.
Independent Store connections and actual caller mutants cover initial Starting/
Running, terminal→live and untyped Task Consultant, with real managed positives.
Missing operation/handle or generic record wrappers cannot evade this shared fence.

Task-free Project/optional-Goal Consultant ownership is the separate pending
[Issue58](https://github.com/shuhei-suzuki/rururunx/issues/58) contract, not a managed
Task receipt or new Task Consultant permission. Its affected native profiles remain
Unsupported-before-effect until reviewed retained ownership/actual settlement and
Project/root/common-Git exclusion compose. Canonical primary-root and universalLost
checks remain applicable, but terminal labels do not prove cleanup. This inventory
adds no schema58 table/port or production readiness claim to Issue19.

Operation-aware lease classification is explicit. Open Executor operations (including
receipt-pending phase closure) exclude review-lock acquisition, competing native
admission and Mutating access. Executor native tool effects remain INSIDE its actual
owned supervisor/cohort under that operation lease and applicable AllowCurrent/Broker
permission. They never self-authorize Store Mutating/maintenance/evidence ports or
exempt a competing runtime caller. Runtime-native Git/hooks/evidence effects outside
that owned cohort require the separate actual60 producer/reservation or refuse. Open read-only
Reviewer operations exclude Mutating access and competing native admission, and may
coexist only with the exact immutable review lock IDs/versions pinned in their
operation. Acquire/validate the actual review lock before marker commit; the marker
transaction captures and compares the complete scoped lock set and pins it in the
operation encoding. No lock can be added/changed/released around an open operation.
Receipt publication keeps those pins; exact closure releases operation ownership,
and subsequent deliberate lock release retains existing lock authority. Binding,
observation, admission and closure compare the same set/version without treating
an operation's own previously acquired review lock as a foreign mutator. Worktree/
Project idle/removal still counts both roles. Test Reviewer+its exact lock positive,
Executor with lock refusal, missing/changed/foreign lock refusal and Reviewer lease
blocking mutation before first Session and after receipt/before closure.

Continue additionally requires the same Workflow generation and unchanged exact
semantic instruction:project.v2/goal.v2/task.v2, scoped config/rule/reference/environment-
name authority keys between prior operation and new frame. The immutable ContextVersion
source map supplies the exact keys; the operation/descriptor pin their versioned
projection digest. Unknown/unclassified new authority keys refuse continuation until
classified. HEAD/source artifact changes within the same accepted authority may be
new input facts; revoked reference, changed constraints/rules/policy or generation
forces Fresh before marker. Full raw prior transcript/default native memory remains
native history exposure, not part of the new authoritative payload or a claim that
the model forgot it. The new frame labels current facts/as-of context; no removed
instruction scope can be reintroduced by Continue. Actual same-generation unchanged-
authority continuation and revoked-reference/generation-change Fresh fixtures are
required, with source-key comparison mutants reaching the real selector.

Closed managed Session bodies remain generic-write immutable, including diagnostics
and terminal→terminal relabels. Usage/audit may remain separate. Only a new exact
managed Continue or actual reviewed recovery port can change that UUID's body;
receipt/history remain frozen. Generic history cannot wedge its future checksum or
reinterpret native completion. Raw SQL receipt UPDATE NULL→Session and generic
post-closure body mutants must reject; real higher-input Continue is positive.
Private native.operation.* and native.settlement.* audit namespaces are reserved
in every generic audit/audit_if_current path. Only actual ports emit inserted,
effects_started, abandoned, settlement/phase_closed events; caller JSON cannot
forge operator-facing producer history.

Actual merged main80452f4 is schema3; the unmerged Issue19 component baseline is
schema5. Neither baseline automatically becomes candidate6. The ordinary/default
build retains its exact source baseline until the actual composed release gate passes. Candidate6 construction is reachable ONLY
through an explicit private fixture constructor/test-only or non-default internal
fixture build, never ordinary rrx Store::open or shipped CLI configuration. In that
scoped candidate fixture build, the fixture constructor creates fresh candidate
state; its current-version open can reopen that exact isolated fixture. An existing
older application database returns MigrationRequired before DDL, user_version,
audit, journal-mode or reconciliation mutation. A deliberate
Store::migrate_to_current(path, expected_old_schema) library entry performs the
same ordered v1→2→3→4→5→6 drain/reference/schema preflight and writer fence in one
Immediate transaction, rechecking the expected version under its lock. There is
no status/list/launch implicit migration. No CLI migration command is claimed here;
a future trusted ingress can deliberately compose this API. Unknown application/
version and stale expected-old-version reject unchanged. Failed drain/ref/schema
preflight leaves all state/marker/history unchanged. Fresh/explicit-migrated final
DDL must match exactly; memory() remains fresh. Native old writer fixtures use this
explicit entry and also prove ordinary open refusal with unchanged DB/state digest.
A deliberate migration still does not make unsupported production profiles ready:
Design18/19 release gates, required producers, private14 recovery and actual23/43
composition remain mandatory.

Task.reviewers/executor remain classified in the declared semantic instruction
projection as well as actor allocation pins. Changing them may restart pre-effect
work or hold a post-PR/merge Task for actual13 reconciliation. This conservative
post-effect reviewer-change availability limit is explicit; do not remove a source
fence or infer safe external rollback merely because no instruction text changed.
The later9 roster/delegation contract must integrate its own reviewed policy rather
than silently weakening this single-phase source authority.


### Design21: pre-marker continuation availability and composed authority

Continue selection is runtime-aware. Before marker commit, the actual adapter's
non-mutating private registry query returns nonserializable continuation eligibility
for the exact closed prior Session/native UUID/receipt and current runtime instance.
No connection, process, Session mutation or model input occurs in this query. Its
closing operation runtime-instance must equal the current runtime; missing registry,
restart or unsupported Continue selects Fresh before any Continue marker. The
transaction rechecks exact prior receipt/body/actor/generation/semantic pins and the
query's private owner-version token; setup rechecks the actual registry again before
effects. A late owner loss rejects without dispatch and follows actual pristine/
setup settlement. A completed Continue refusal is privately recorded as Fresh-only
selection for that exact preceding operation/chain; subsequent retries cannot blindly
select it again. This bounded fact is actual producer outcome tied to that operation,
not a generic string error or caller JSON. Closing receipt attribution carries it
within the existing64 KiB bound. Test restart→Fresh without Continue marker, valid
same-runtime Continue, and raced registry loss→NoCurrentDispatch/non-success closure→
next Fresh, retaining every original attempt and budget. No current registry proof
is reconstructed from a receipt or Runtime UUID alone.

Marker statement order is operation INSERT first, Workflow marker UPDATE second,
within one Immediate transaction. Operation insertion requires exact existing active
unmarked post-epoch native attempt/context/claim and unique open Task operation.
The Workflow update SQL trigger requires each marked post-epoch native entry's exact
operation and rejects true→false, reordered/deleted/replaced marked history or a
new already-marked entry without its exact operation. Rust's private method validates
the complete final relation before commit. An operation committed by privileged raw
SQL against an unmarked entry is held corruption, not valid admission; SQL immediate
triggers cannot enforce two-way deferred commit invariants. No public API can insert
such an operation; handle possession remains required. A failed second statement
rolls back both and audit. Raw SQL mutants distinguish this enforced direction from
the final Rust relation instead of claiming impossible deferred triggers.

Add a Workflow records SQL receipt backstop: a marked managed entry leaving active,
becoming closed, or entering terminal/evaluating state requires its exact existing
settlement receipt. Preserve permitted live same-state detail and fact observations;
terminal Task cancellation can retain the active Running history/operation without
pretending closure. Existing generation/attempt/history identity checks remain.
Workflow closure UPDATE precedes operation phase_closed UPDATE, whose trigger matches
that exact closed history entry/receipt. Both halves plus full Rust closure predicate
are required; an invalid index, foreign receipt, skipped phase_closed or rollback
cannot release ownership. Exact JSON paths/state variants are schema goldens and
all actual closure consumers have causal mutants.

Until actual Issue58's typed retained-ownership port composes, the shared Store also
rejects ALL generic/non-owned nonterminal Project/optional-Goal Session INSERT/UPDATE,
including Consultant. Ordinary scoped NativeCAS has no exception. Initial terminal
factual history remains allowed but is never cleanup authority; old live/Lost owners
must drain for explicit migration. The only fresh native Task Session path is its
actual managed operation port. NoTask native profiles remain Unsupported before
reservation/process/bytes until58, which then supplies its own reviewed private port
and mutual exclusions. This is a real gate, not a role-name read-only assertion.
Project.root may CONTAIN worktree_root/root/worktree and the common Git directory;
canonical path inequality alone never excludes Task mutation. Independent connections
exercise Project/Goal Consultant refusal against both managed Executor and Reviewer,
while legitimate terminal facts remain supported. Future58 actual workload leases must
compose root/common-Git/Task exclusions atomically, not bypass this generic fence.
Physical conflict scope includes canonical root containment/overlap and shared
actual common-Git directories across Project IDs; nested primary repositories are
not assumed forbidden. Out-of-lock Git/filesystem/removal/publication effects need
symmetric durable conflict reservation before observation/effect, retained until
actual owned settlement, with atomic version/scope revalidation. A read-then-act
Store check cannot certify exclusion. Real disjoint roots/common directories remain
independent. This is a required composed58/19 consumer contract, not an invented
available private producer or permission to mutate source roots.

Candidate schema6 is never released as standalone production. The first deployable
persistence version is the explicitly reviewed composed version containing the actual
19/23/43/58 and recovery authority required by enabled profiles, with one ordered
linear migration history and final table/schema/projection goldens. Candidate6 fixture
DBs are isolated mechanics evidence; no routine status/launch or release opens real
old state with automatic migration. Later composed versions require the same complete
all-table writer fencing and drain/actual recovery policy; a held candidate operation
cannot be upgraded by relabeling it. This does not require fictitious tables or final
version numbers before those actual source ports exist.

The accepted Goal definition's CompletionCriterion evaluator is classified as immutable
semantic authority (RequiredTasksVerified or Human{goal_pack_input}, legacy Unverified),
separate from satisfied/evidence bookkeeping. Projection2's reviewed inventory includes
that classification and default-legacy meaning; actual23 adding its field/table must
supply the exact exhaustive typed encoder/golden in the composed source gate. A change
from a previously PUBLISHED projection implementation still requires its encoding and
writer-version bump together; candidate docs cannot claim a released6/projection2
pair already contains absent23 source. goal_authority attests exact scoped definition
acceptance; it does not duplicate mutable criteria. Schema enumeration includes every
actual component table (including goal_authority and any additional compiled authority),
not only the candidate6 list. Lost/restart recovery and native containment remain real
production prerequisites, not solved by the composition label.

Drop has no SharedStore/SQLite call, including try_lock audit inserts that could wait
on another connection's busy_timeout. It only performs bounded in-memory try_send or
omits the hint. Durable outstanding operation remains attention/held regardless of
queue failure. No DB/filesystem IO, process inspection, wait or async cleanup runs in
Drop; actual supervisor retains cleanup bookkeeping. Test another connection holding
an Immediate write lock, mutex poisoning and queue saturation: Drop cannot attempt a
DB write/wait or erase ownership. A later actual owned worker may publish bounded audit.

The complete2 MiB admission tuple bound, including exact/worst-case effective actor
field allowances, is enforced BEFORE phase ContextVersion publication and again in
the marker transaction; per-entry source caps alone are insufficient. The1 MiB full
payload bound remains separate. Overflow produces no ContextVersion, marker, operation
or admission and leaves previous state unchanged. Caller boundary fixtures cover
just-below/exact/above aggregate tuples and unknown new keys. No forced retry loop can
publish an inherently inadmissible authority map.

Early requirement paragraphs describing pending waiting reentry, generic blocked-
owner Lost or operation-less terminal continuation are schema5 component history only.
Schema6 pending unconsumed Task Starting→Waiting rejects; all generic Task/NoTask
nonterminal writes reject; own managed Lost uses actual private observation, stays held.
Supported continuation is only the sealed managed path with the actual private owner.
These supersede old standalone/component positives without making native readiness claims.

An ApprovalReviewer is action-decision-only, never native-ownership-free. Existing
Workflow RequirementsReview/DesignReview/ImplementationReview/SecurityReview phases
use typed managed19 Reviewer operations/receipts; no Phase::DecisionReview exists.
Cross-Agent Broker review needs the separately reviewed9/10 decision-member/slot lease
bound to original request/operation, exact input/source and its own actual native
settlement; no such DecisionTask port is claimed today. Consultant58 cannot cover it.
Until the exact owned producer composes, approval-review native launch is Unsupported
before effects; public role/operation-free wording cannot adopt an Executor frame or
release native work through a terminal label.


### Design22: current managed grants and fixture-only candidate persistence

OwnedNativeObservation has an exhaustive private branch distinction. Historical
Acked observation/DenyHistorical carries exact operation, Session and remembered
version/input/actor pins and preserves the existing provider-specific activity
predicate; it never grants an operation. AllowCurrent additionally carries actual
nonserializable permission-grant authority from the trusted native policy/Broker
integration, bound to the exact pending native request/tool operation, decision,
original admitted frame and current operation. A PermissionDecision enum, public
JSON, operation_intent string, ScopeSnapshot alone or row existence is not that proof.
The actual supervisor possesses the operation; its private grant branch publishes
bounded operation journal and any required Session lifecycle update atomically.

AllowCurrent performs the SAME full NativeCAS current P/G/T and own-Session versions,
active Goal/nonterminal Task, Executor ReadyForPr/PrCreated and existing worktree
constraints, plus composed23 Running-Goal policy, complete lock-set and exact private
operation/frame/pair predicates in one Immediate transaction before ALLOW bytes.
Current CPP pointer/checkpoint head and the original complete prepared payload/pair
remain exact. The scoped physical-source predicate in Design26 compares immutable
governing/reference families, not an Executor's mutable working-content snapshot.
No grant recaptures or replaces prepared authority. Bounded physical observations
occur outside SharedStore, then exact held expected authority/version pins are
revalidated in the transaction; no external Git/IO inside it or OS-atomic filesystem
claim. Stale governing authority/head/lifecycle/locks emits zero grant bytes and retains
ownership. Historical observation/generic CAS cannot substitute for AllowCurrent.

The private permission producer/#10 integration is not claimed present. Unsupported
managed ALLOW fails before grant effects until its actual trusted producer composes;
controlled production-consumer fixtures must call the real private operation/grant
port, not seed JSON/SQL. Inventory Claude commit_current, Codex Approve, Generic/Grok
permission surfaces, Grok ACP fs/read_text_file and fs/write_text_file host callbacks,
and every mid-turn operation journal consumer. Controls prove
current ALLOW→wire, Paused Goal/held Task/changed P/G/T/own Session/lock/current head
or source→zero wire, and a caller-reachable mutant removing the ALLOW-only current
predicate. Historical DENY keeps the Claude Blocked no-wire and Codex unchanged
Waiting fixed-Decline asymmetry; no additional DENY privilege/fence is inferred.

Candidate6 source/DDL must not be merged as a standalone default runtime upgrade.
Merged main ordinary Store/state remains its actual schema3; unmerged Issue19
component ordinary Store/state remains5 until one actual reviewed composed source/
CLI migration release is ready. No main5 state or deployed5 release is asserted.
Internal candidate fixture builds explicitly label artifacts nonproduction and
cannot open user state or advertise native capability/readiness. The default-build
control pins its exact baseline (main3 or unmerged component5) and creates/reopens
that version; isolated candidate control creates/reopens6, and
ordinary runtime refuses candidate fixture state instead of silently adopting it.
Candidate6 DBs are NOT a supported production migration origin and may not contain
real user/native state; teardown requires actual safe owned cleanup, never force
upgrade of held fixtures. The first composed deployment receives its exact final
version/DDL/projection, complete native supported-profile gates and trusted deliberate
migration ingress in the co-integration review. No stranded user6 state or default-baseline
regression is accepted merely by a docs-only release disclaimer. Native old-writer
compatibility proofs run against actual isolated candidate/composed fixture helpers.

Continue predecessor means the LATEST closed native attempt in the exact same
Workflow generation/role chain, INCLUDING Session-less attempts. If that entry has
no bound Session, selects Fresh; no skipping backwards to a convenient old UUID.
A privately recorded Fresh-only refusal remains in the chain until a successfully
admitted/settled Fresh attempt establishes a new eligible owner; pure pristine
attempts do not erase it. When Continue is eligible, the latest Session-bearing
operation must also be the operation that last actually delivered admitted input
to that exact native UUID; authority comparison uses its immutable frame/receipt
and the current semantic/source contract. Record-only historical native labels cannot
create delivery. Test A→B(Continue refusal/Fresh-only)→C(pristine no Session)→D Fresh,
unchanged chain budget/author/history, no registry/wire query for an older A bypass.

Bound new post-epoch Workflow history to256 entries and64 generations, including
native and EvidencePort attempts and failed/refused entries. This bounds SQL history
scans, but history/generation caps alone DO NOT bound context publications. The separate
canonical context counter below charges EVERY Workflow-owned ContextVersion, including
escalation/invalidation/Cleanup publications without a new history entry. Ordinary
publication/history growth cannot consume the Design25 irreversible-tail reservation;
terminal frozen publication spends its exact dedicated reserved capacity. Legacy
pre-epoch terminal history stays read-only, never adopts managed authority. Overflow
holds with exact prior owners/evidence intact; reorganization never launders review9
obligations/shared exhaustion or physical holds. Controls at255/256/257 history and
63/64/65 generations prove no overflow marker/context/operation and no reset.

Role-classified factual Consultant history is explicitly UNVERIFIED CALLER CLASSIFICATION.
EVERY retained event, own-Task historical_checkpoint AND cross-Task promotion,
includes that marker in the typed provenance and actual rendered target frame; its content/hash proves captured bytes, not native
origin or assertions. Unknown/legacy role provenance defaults to caller-classified.
No generic initial terminal record can produce owned-native role provenance; future
actual retained58 or separately reviewed producer may supply scoped native proof
only through its actual private port. This never broadens58 to Task Consultant launch.
Target accepted Goal/Task constraints remain authoritative and original scope/hash/
actor/event provenance remains immutable. A generic terminal Executor role is also
unverified; role never upgrades caller-supplied Constraint/Decision/NextAction text.
Test generic terminal Executor own checkpoint and Consultant promotion both render
the explicit marker, and caller JSON/role change
cannot render it as verified native evidence or clear a blocker.


### Design23: complete publication bounds, maintenance and closed-receipt binding

Add a private fenced workflow_context_counters row keyed by exact Workflow Record/
Project/Goal/Task, initialized at0 for post-epoch ownership. It counts EVERY new
Workflow-owned ContextVersion publication, including escalation, invalidation-only,
retry, Cleanup/frozen terminal pack and source-change-before-dispatch replacements;
no dependency on history append or generation rise. Cap256 with the private
Design25 reserved tail capacity; increment/spend reservation with the
ContextVersion/Task pointer/Workflow/audit in the same Immediate transaction; rollback
restores all. Include the table in candidate/composed all-table writer enumeration,
FK/scope/immutability triggers and fresh/migrated schema goldens. Generic context/
record/public metadata cannot install/reset it or bypass charging through another
producer. Pre-epoch history remains immutable read-only; its contexts are not fabricated
new counter entries. No-op escalation with identical effective risk/class/source/
policy/budget reuses current context and changes no Task/Workflow authority/version;
real changed context still charges. Tests repeat no-op escalation and exercise real
escalation/invalidation publications at255/256/257, checking exact no overflow pointer,
row/audit/history/operation and no ghost phase pack. This bounds fresh Workflow context
history by at most 256 complete encoded bodies of at most 8 MiB EACH, subject ALSO
to the stricter shared 128-MiB used/reserved Workflow quota (Design31). Payload bytes
are not complete stored-body bytes; standalone/Goal artifacts have separate bounds.

Task-worktree create/adopt/remove/maintenance effects require an ACTUAL durable
exclusive maintenance reservation BEFORE external Git/fs effects. Its acquisition
transaction checks exact Task/Project/physical identity, no open native operation,
existing lock/admission conditions and the complete scoped lock set. Marker/native
admission observes that same reservation and rejects; acquisition likewise rejects
an already marked/open operation. Retain reservation throughout out-of-lock Git and
actual owned settlement; timeout/uncertainty/crash retains it for14, not automatic
Drop/error release. Only actual tracked no-effect refusal or owned effect cleanup
can close it. Existing authorization/activity fences are not relaxed. Scope includes
actual root/common-Git overlap where declared effect can touch shared metadata;
composed58 physical reservations handle nested/cross-Project overlap as required.
Independent-connection barrier races between maintenance reservation and marker
prove exactly one wins BEFORE removal bytes/effects; no read-then-act precheck counts.
The actual runtime Git/Generic/Evidence producer is pending
[Issue60](https://github.com/shuhei-suzuki/rururunx/issues/60), including sessionless
effects, not invented native19 settlement. Default3/component5 remain their actual
baseline; candidate native dispatch with missing required producer is Unsupported.

For receipt-sourced late binding, ONLY the sealed Store-derived ClosedSettlementBinding
proof may supply the allocated Session ID to the same sole private43 binder. Derive
inside its transaction from exact phase_session_owners + private operation + exact
KnownCurrentTerminal receipt, current Session version/body checksum and complete
current input/admit/consumed actor/frame pins. It is not caller SessionID/receipt JSON,
returned native identity or a new live owner. Full current active P/G/T/Workflow Record,
generation/index/context/dispatch marker, agent/native identity, lifecycle and pinned
complete lock-set predicates stay identical to normal43 binding. Allowed writes are
ONLY that exact active history.session_id None→allocated ID, ordinary Workflow Record
version/updated_at increment and one bounded reserved audit;
no Task/P/G/Session/lock body/version increments and no launch/ALLOW/resume permission.
NoSession NoCurrentDispatch and restored-prior Exited are nonbinding. Dropped start
future/restart with an exact settled successful receipt can then bind and close through
ordinary current-success predicates without fabricating another dispatch; actual
success still requires consumed current input+Exited+successful owned cleanup receipt.
Failure/inactive owner/stale source-authority CAS keeps original claim/receipt; no
inactive recovery binding exemption exists before a separately reviewed19/14/43 mode.
Fixture actual supervisor completes receipt before start-result delivery, then drops
that delivery and runs the actual private binder; foreign/stale/restored/missing owner
mutants reject. A controlled fixture may not seed private SQL/claim success from labels.

A live checkpoint append leaves admitted input unchanged but can invalidate an
outstanding ALLOW. Stale-head grant sends zero ALLOW bytes and takes the existing
handle-bound fixed DENY path if that provider's existing activity predicate permits;
otherwise actual supervisor stop/cancel owns cleanup and receipt. No timeout-only
idle permission waits are invented. Claude Blocked still cannot send prewire DENY;
Codex unchanged Waiting Decline preserves its existing behavior. Cancellation with
unknown native outcome/cleanup remains Lost/held, never fabricated failure receipt.
Test live append→ALLOW reject→actual DENY/stop and its factual settlement/held outcome.

All Engine-side native Git/filesystem source observation completes BEFORE marker.
After marker the owned handle transfers directly to the actual supervisor; any required
prewire re-capture/Git/native setup runs only after effects_started under that actual
producer's retained effect ownership. Pure bounded Store revalidation remains inside
the marker transaction; no external Git under the mutex. Source drift before marker
may reject without a native operation; post-marker owned observation uncertainty
cannot mint pristine refusal. Actual60 runtime-job reservations also cover pre-marker
Git effects independently, no temporary operation-less exemption. A consumer mutant
that runs Engine Git after marker must fail ownership/order fixture.

Consumption-before-wire is authorization, never proof of delivery. Even producer-
known zero bytes after consumption fits NEITHER existing receipt class: NoCurrentDispatch
forbids current admission/consumption; KnownCurrentTerminal requires actual owned
native outcome. This intentionally stays held with bounded zero-wire observation
until the actual gated14 recovery port. No JSON never-sent flag, new receipt class,
forced uncancellable wire or restoration/Succeeded shortcut is introduced. Required
production recovery/profile gates explicitly include this boundary.

All schema6 private producer events use ONE reserved rrx.private.* prefix, including
preparation/allocation/admission/consumption, epochs/migration/refusal, continuation
Fresh-only, grants/observations, operations/settlements and43 binding. Generic audit/
audit_if_current rejects that prefix plus retained native.operation.* and native.
settlement.* historical aliases. Enumerate all compiled private event constants and
require reservation through every generic audit path; no look-alike admission kind
is treated as trusted merely by public text. Existing ordinary session.saved etc
are explicitly factual projections, not certificates. Actual audit/usage bounds
remain and publication is atomic with its owning private write.


### Design24: reachable failure release, Goal admission and factual rendering

The canonical writer and closure tables now explicitly separate non-success
TerminalRecovery from the active-only43 binder. Store derives a sealed non-success
closure proof from exact scope/Workflow generation/index/context/marker, allocation,
operation and its actual current receipt plus frozen Session Record/body/preparation/
admission/consumed/restore pins. No caller SessionID/receipt JSON or terminal label
creates it. Under an authorized Cancelled/Failed decision it releases only the exact
settled claim, leaves any unbound session_id None, preserves Task decision/definition
and grants no native permission. All ordinary unknown-gate/claim/CAS and operation
phase_closed constraints remain. This is an actual owned cleanup path, not an inactive
success-binding exemption or14 Lost recovery. Fixture barriers cancel AFTER allocation
BEFORE start-result/binder, then actual producer settles; exact failure release passes,
missing allocation/receipt class/current body/source pins fails, and Succeeded fails.
The active-only success binder remains43's sole SessionID writer with only its ordinary
Workflow Record metadata increment and bounded audit.

Actual composed23 readiness is a repeated authority gate, not a display evaluator:
marker, initial preparation, first/new consumption and each ALLOW all check the same
private predicate in their committing transaction. Missing implementation rejects
before effect. Tests/mutants exercise Created/Analyzing, absent/wrong definition digest,
whole-Goal hold, nonmember DAG Task and merely-Merged/unverified hard prerequisite at
EACH actual consumer; exact accepted Running/all prerequisite proofs is the positive.
Historical ACK/fixed DENY/actual settlement continue under their existing constraints.

Rendering never upgrades factual checkpoint text. EVERY event in own history and
promotion carries UNVERIFIED CALLER CLASSIFICATION and retains captured Session/role/
scope/hash as attribution only. Mandatory placement preserves history but does not
make it an authoritative instruction; accepted Goal/Task/rules and verified evidence
remain separately labelled. Generic terminal Executor 'skip tests' Constraint/Decision/
NextAction must render with the marker, cannot clear a blocker/replace actual constraints
or forge native output. Preserve exact stored/checkpoint hashes; the new rendered
provenance belongs to the new version2 frame projection, not rewriting old events.

Bound every new post-epoch attempt to64 unique gate observations and the complete serialized
Workflow Record body to8 MiB. Reserve a possible new unique observation slot before every EvidencePort
execution, including a suspected repeat; no free slot refuses before effects. Exact
repeat outcomes may reuse the prior entry under Design26, releasing the unused slot.
There is never a65th unique observation or an unrecorded changed outcome. Reserve64 KiB
within that body cap for exact terminal/hold/closure metadata; ordinary claim/history/
publication writes cannot consume that headroom. Existing bound text/authority/history/
context limits still apply. Every JSON-reading SQLite trigger must first enforce UTF-8 body bytes within
its OWN body/explicit sequential CASE before any JSON call (trigger ordering is not
a guarantee); a separate length trigger alone is insufficient. Rust
checks the complete rendered Record before every Workflow write; receipts remain
separate so a bounded hold cannot erase owned cleanup. Overflow holds with evidence/
operations intact, not repeated retries that reset review budgets. Boundary63/64/65 and
8-MiB byte/headroom controls prove unchanged Record/Task/audit on refusal and receipt-
backed closure using the reserved headroom. Legacy pre-epoch history remains read-only.

Pin emitted events: rrx.private.operation.created, rrx.private.operation.effects_started,
rrx.private.operation.abandoned, rrx.private.workflow.phase_closed,
rrx.private.settlement.published. All other compiled private producer constants likewise
use rrx.private.*. native.operation.* and native.settlement.* are rejected historical
aliases ONLY, never current emitted kinds; status/attention consumers use compiled
constants. Exact prewire restoration exists ONLY in the same Immediate settlement
transaction as its current NoCurrentDispatch receipt, Session body/version and audit.

A failed prewire Continue closed with restored-prior NoCurrentDispatch is deliberately
NOT transparent to predecessor selection. It is the latest closed Session-bearing
attempt but did not deliver, so the next invocation must Fresh; later successfully
admitted/settled Fresh can establish a new eligible chain. This declared availability
limit avoids skipping to an older receipt/UUID and never resets budgets/authorship.
Consumed-known-zero-wire remains held14, never this restoration alternative.


### Design25: irreversible tail reservation, candidate identity and exhaustive continuation

BEFORE the FIRST configured irreversible external claim (Pr, or any earlier producer-
declared irreversible/disposal effect), the same gate-claim transaction reserves the
EXACT remaining mandatory configured Workflow tail: every remaining ContextVersion,
remaining phase/history entry, final frozen terminal pack and required Record closure
headroom. Reservation includes exact/worst-case serialized tail attempts, bounded
gate observations/source maps and terminal metadata from the actual producer contract,
not only the64-KiB fixed final diagnostic margin. Used body+reserved tail bytes must
fit8 MiB before effect; an unbounded future representation refuses. Revalidate each
next-effect reservation against its captured source/observation contract, and final
Cleanup freezes its exact pre-disposal snapshot plus bounded outcome, so no post-disposal
recapture can spend unreserved bytes. Derive from the actual frozen effective phase plan INCLUDING every documented normal
later finalization extension (QUICK MergeGate/Cleanup/frozen terminal), even when the
currently executed QUICK plan ends at Pr. Requesting that extension later consumes
its existing reserved tail, never tries to acquire first capacity after PR effects.
Derive counts from that complete supported plan, not a public count or
fixed PR-only assumption; unknown effect/plan class refuses before external execution.
The private workflow_context_counters authority also records this immutable tail-plan
identity and remaining reserved publication/history counts. Checked used+reserved<=256
must hold for BOTH counters/history; ordinary retry/escalation/invalidation cannot spend
this reservation. Capacity is charged/consumed atomically with each exact planned tail
transition and terminal publication; failed tx restores it. New source/policy/risk plans
must acquire their additional capacity before any next effect, otherwise hold with prior
outcomes intact; no promise of automated post-PR reconciliation or mandatory-history drop.
Unused reservation is retained through uncertainty and only exact safe terminal closure/
authorized reconciliation can release it; history/used counters never reset. The reserved
64-KiB Record metadata margin remains separate from context/history capacity.

Current STANDARD example reserves the remaining MergeGate/Cleanup/frozen-terminal
publications and any missing phase entries before Pr. With used254 and3 mandatory
remaining publications, Pr refuses before bytes; a fitting253+3 sequence reaches256
and actual Cleanup disposal THEN persists terminal from its reserved last publication.
Test253/254/255/256, configured earlier effect/tail counts, both context/history saturation,
source drift/newplan before-next-effect and rollback. A mutant removing either reservation
check must reproduce post-disposal persistence failure, not merely a counter unit mismatch.
Cleanup's already-admitted claim still finalizes its exact frozen evidence after disposal;
a later checkpoint append never changes the admitted reservation/provenance.

Candidate6 fixture DB identity is APPLICATION_ID=0x52525843 (ASCII RRXC) AND version6,
distinct from production RRX1=0x52525831. Only explicit internal candidate fixture creation/
upgrade accepts this identity; every candidate open verifies it before reads/writes.
The genuine older-version target may be upgraded to RRXC only in the same isolated
fixture migration transaction, never on real user state. Every ordinary/composed runtime
rejects RRXC BEFORE version/migration dispatch even if its composed user_version is6
or greater; RRXC is NEVER an ordered production predecessor. Candidate scopes/epochs/
receipts are not retagged/imported into RRX1. This identity is durable and independent
of table names/user_version/CLI flags. Test composed open of candidate refuses unchanged
DB hash; candidate refuses RRX1 default state except the explicit isolated-fixture
migration entry. Native historical live-writer trigger/function tests still apply.

Actual production maintenance/runtime Git authority belongs60 and is a real deployment/
consumer prerequisite. Candidate mechanics positives additionally require a REVIEWED
fixture-only runtime-job producer, compiled solely under test: it acquires the SAME
real durable maintenance/effect reservation and exact version/physical conflict gates,
executes only controlled scoped synthetic jobs with actual tracked ownership and
settles their known cohort through the same private close predicate. It cannot accept
arbitrary external commands/profile, forge receipt JSON or seed private SQL. Production
callers never select it. Worktree provisioning/capture/disposal in positives occurs under
that fixture's actual reservation BEFORE each effect, not pre-created operation-less
candidate exemptions. Controlled cohort proof is mechanics only; nativeGit hooks/helpers/
fsmonitor/escaped services and production60 conformance remain pending. Tests prove marker
race, symmetric overlap/disjoint scope, pre-effect refusal, unknown-held cleanup and
actual producer-close without terminal labels. If this real fixture producer is absent,
those positives are unreachable/PENDING and cannot earn source acceptance; missing60
production always refuses affected real profiles. Source acceptance must inventory each
fixture versus actual production consumer honestly, with no cyclic claim of completed60.

Continue compares the immutable original delivered frame's typed key-family inventory,
not only string-prefix subsets. Native6/16 memory exposure remains labelled; Continue
never claims the previous native history forgot facts. Exhaustive classification:

| Produced authority family | Continue condition / Fresh rule |
| --- | --- |
| Scope/actor/provider/profile/role/worktree/repository:identity, generation | EXACT unchanged; mismatch Fresh, foreign identity rejects |
| instruction:project.v2 / goal.v2 / task.v2, governing policy/evaluator/constraints, config:* / rule:* / rules:* and environment-name/ref declarations | EXACT unchanged complete key set/digests; missing/added/changed authority Fresh |
| checkpoint:head + own historical checkpoint; instruction:pack_facts.v2 / legacy spelling under explicit projection version | ONLY typed append-only facts (every prior list is an exact prefix, old scalar facts unchanged) may be new input; any removal/replacement/reordering routes Fresh, aggregate pack_facts digest cannot route; full fresh frame/head checks still required; references below remain separate |
| promoted_consultation and artifact/reference scope/path/identity/digest manifest (typed pack fields, even if represented only inside pack_facts) | ANY revoked/removed/replaced reference Fresh; addition also Fresh unless separately reviewed monotonic reference rule; identical immutable referenced snapshot may Continue |
| actual captured HEAD/tree, repository:inventory/map manifest, worktree:* source content/missing/dirty hashes and declared Task revision | New source facts may Continue only within unchanged semantic/ref authority and exact same accepted scope/phase chain; source movement into governing/reference category uses that stricter row |
| workflow:* or raw bookkeeping versions | Not accepted semantic keys in the phase source map; malformed producer rejects, never grant Continue |
| Any other key/typed field or projection inventory change | Unclassified Fresh-only/refusal, no default semantic exclusion |

Compile producer key-family and typed referenced-field inventory with the projection2
schema golden; cross-check EVERY emitted family from18/19/Engine mandatory rules.
Comparison includes promoted/reference typed fields independently of aggregate pack_facts
hash, so dropping S1 cannot be treated as harmless checkpoint advance. Tests actual own-
head/facts advance positive, promoted S1 removal/replacement Fresh, unchanged immutable
S1 Continue, source hash/revision advance positive under same authority, config/rule/
environment/constraints/generation Fresh and unknown family refusal; each causal selector
mutant must reach pre-marker routing and actual no-Continue wire outcome.

43 reconciliation requires the original marker-time binding frame, retained immutably
separate from any later ordinary owned-observation bookkeeping CAS refresh. It includes
resulting post-marker P/G/T and Workflow Record version, all captured atomically
IN the marker transaction (including the full lock set), exact attempt/context/source/frame,
actor/native selection and full lock set. Store never builds it by matching current rows
with themselves. The active-only record binder uses those original/current exact pins;
mismatch holds for14, not an ack-induced silent refresh. Actual authoritative Task-driver/
settled notification and original managed start-error path may invoke the private
reconcile port after exact known-current success settlement; passive41step/poll remain
read-only. Restart cannot reconstruct live owner from rows and needs actual14 authentic
recovery. Non-success/NoCurrentDispatch uses its separate19 closure, never fake success.
The composed43 requirement/design gate owns this additional routing; no producer/driver
is claimed available by this paragraph.


### Design26: scoped current grants, compact observations and real composition gates

AllowCurrent preserves the immutable admitted full-frame digest, consumed UUID and
private operation/pair. It performs actual composed23 admission and exact semantic
P/G/T, own Session, complete locks, lifecycle and Broker action/request/grant checks.
Expected versions come from the supervisor's privately retained admission/owned CAS
state, never a caller JSON tuple or reading rows inside the transaction and comparing
those rows to themselves. A legitimate ownership update must be produced by its
existing reviewed private CAS path; there is no general refresh to current rows.
Goal pointer-only publication and derived progress do not bump semantic Goal authority;
a real changed DAG/hold/version still fences. Benign sibling authority drift therefore
may retain an operation for recovery; this design does not promise transparent progress.

The physical-source families for this grant predicate are exhaustive:

| Source class | AllowCurrent currency |
| --- | --- |
| Repository root/common-dir identity and exact HEAD; current CPP pointer/checkpoint head; accepted semantic instruction/policy/evaluator/constraints | EXACT original/current authority as applicable; drift emits zero ALLOW |
| Governing rule/config bytes, environment-name/reference declarations, promoted consultations, foreign/primary-root artifacts, and own-Task input references not declared current phase outputs | EXACT complete family sets and physical bytes/identity; removal/addition/change emits zero ALLOW |
| Runtime-declared current Executor phase outputs in its exact Task worktree | Mutable only under the Design32 output contract; governing/promoted/foreign classification wins and remains exact |
| Selected Executor worktree slice/inventory/content/dirty/missing references | ONLY sealed declared phase output paths are mutable; all undeclared selected input references stay exact, including removal/addition, under Design32/35 |
| Reviewer source/reference snapshot | Remains immutable under its supported read-only profile; no Executor mutable exemption |
| Unknown source family, scope or producer classification | Refuse; never infer mutable from an unrecognized prefix |

The Executor exception is a scoped permission predicate, not evidence of writer origin.
It cannot distinguish its own edits from arbitrary same-UID foreign writes to mutable
content, and makes no such attribution/isolation claim. Broker proof remains bound to
the exact action and operation; this does not approve unrelated actions, alter original
43 marker pins, recapture prepared input or permit new model consumption. Native60 owns
any external Git observation before native19 marker; source capture is not lease-free.
S6-06 requires actual own-write→ALLOW/wire plus changed governing rule, HEAD, reference,
revoked lifecycle/lock and stale semantic version→zero-wire controls. An immutable-family
comparison-removal mutant must causally reach the real grant consumer.

New post-epoch GateObservation is a compact reference, never an embedded SourceSnapshot
map. Private immutable workflow_observation_sources stores the complete sorted source map
once per exact scope/digest (maximum2 MiB, actual encoded bytes counted); its digest,
revision and producer projection bind its immutable ContextVersion/observation authority.
Private immutable workflow_observation_outcomes stores each complete outcome/evidence/error blob is bounded to1 MiB of encoded UTF-8;
counts/references share the64-unique-observation ceiling. Overflow retains the bounded
observed prefix with exact overflow/count provenance and cannot certify Passed. Large
outcome/evidence/error text is separately bounded immutable content with exact references. Workflow embeds at most512 UTF-8 bytes per observation entry: typed source-
blob/outcome/evidence references, first/last observation timestamps and checked occurrence
count. Unknown/missing referenced content holds; no public ID/hash mints source authority.
Every referenced blob is inserted atomically with its exact observed claim token before
Workflow publication and included in the schema6 all-table writer fence/migration/golden.
Generic Records/audit cannot replace it. Old embedded pre-epoch history is read-only.
Per-blob bytes, unique observation count and complete8-MiB Workflow-body cap are simultaneous
ceilings, not a promise all maximum history metadata fits. All new private observation/outcome/durable native-result blobs share a128-MiB
aggregate perWorkflow persisted-byte quota under Design29; exact owned identity dedup
never hides copies. This is not a total application DB or realtime memory bound.

Irreversible tail reservation uses this real representation: each planned tail attempt
reserves64*512 bytes for compact entries plus its bounded fixed metadata, required source/
outcome blob capacity and closure bytes. Source-map bytes are reserved in their actual
blob storage, not multiplied into the8-MiB Workflow body. A100-KiB map with three64-entry
tail attempts stays96 KiB of compact body entries, with exact separate blob charges.
Tests use realistic maps, maxima2 MiB, complete body/headroom and tail reservations;
removing compaction must reproduce pre-effect capacity refusal/post-disposal failure.

Repeated definitive Waiting observations reuse the prior unique entry only when exact
scope/revision/source-map digest, complete outcome and evidence/artifact digests ALL match.
Checked occurrence count and first/last time retain the repeat; changed source, evidence,
reason or outcome creates a new unique entry. Each explicit gate execution first reserves
one possible unique slot and actual60 effect capacity; after exact claim-token/CAS proof,
a repeat releases that unused slot. At64 unique entries no further execution is allowed,
even an expected repeat; u64 occurrence overflow also refuses before effects. Separate
monotonic claim epoch guards each execution, so summary count never replaces ownership/CAS.
Passive41 step/poll stays read-only. Every actual execution additionally consumes
one claim+one observed link: at most99 cycles under Design33/35, regardless of
unique-entry dedup or u64 occurrence room. More than64 identical Waiting executions followed by
Passed must retain two entries with exact repeat count; changed-evidence and64-unique refusal
controls retain every executed outcome. Neither repeated waiting nor dedup resets budgets.

Every actual managed reconciliation consumer (Generic/Claude/Codex/Grok) treats unexplained
physical inventory/index/source mismatch as uncertain held ownership. Successful selected-
group cleanup, native terminal or output verification alone cannot publish a clean Failed
receipt when reconciliation itself failed. A proven bounded known mismatch may only use
its reviewed actual producer evidence; no guess from index pathname or terminal labels.
The Grok branch currently computes cleanup/output success separately from reconciliation;
source acceptance must change that real consumer and test foreign index/source discrepancy
retains operation/claim with no receipt. Actual escaped/native cohort proof remains6/60/16.

ALL Goal writers, including future23 typed lifecycle/DAG/additive graph/acceptance writers,
must preserve the latest typed ContextVersion pointer read in their transaction or reject
an explicit stale pointer expectation. Semantic Goal.version alone cannot authorize a stale
whole-row rewrite after pointer-only publication. No writer silently copies its captured
older pointer. Two-connection tests through the actual23 typed port cover pointer publication
then stale typed update, simultaneous publication winner and reopen/audit invariants.
This composes with accepted immutable Goal definition; Task acceptance does not permit Goal
objective/criteria/evaluator/constraints edits.

S6-02/03/04/06/09/10 and these pointer controls require actual19/23/43 source co-integration.
The actual23 trusted acceptance/admission port and authoritative Task driver,43 sole binding
port and19 managed producer must compile and execute together before source acceptance.
A seeded goal_authority, stub readiness predicate or fixture-only fake binder earns no
positive credit. Until those ports exist, these positives are PENDING; honest negative
refusal/mechanical controls may proceed. Reviewed closed synthetic runtime jobs can gate
candidate mechanics only, using isolated repositories and closed executable/script/env/
effect inventory; selected-group death never proves nativeGit containment. S6-14 fixture
positives are source-gating mechanics, NOT production-readiness. Actual60/58/native6/14/16
production producer/conformance gates stay pending; co-review of real combined producers
is allowed without inventing a completed-merge dependency cycle.

The original43 marker binding frame remains immutable, including raw authority versions.
Hold/resume or sibling drift may strand a factual successful yet unbound operation: retain
receipt/claim and report recovery-pending, never fail/retry successful work, refresh original
pins, release by terminal label or bind under inactive owners. An actual14 recovery mode
would require its own19/43 gate; none is supplied here. A lifecycle hold→resume regression
must demonstrate that conservative held result and zero duplicate dispatch. Where this
missing production recovery prevents a required supported path, deployment stays unready.


Original43 binding pins are Store-derived from the resulting marker transaction,
including resulting P/G/T, post-marker Workflow Record version and its complete locks;
normal returned-Session binding uses this SAME frame, never a later lock read. The
actual23 authoritative Task driver supervises invocation end independently of a dropped
Engine future and derives exact sealed current settlement from durable level readiness.
Notification is only a wake hint, never the sole proof/trigger. Early-before-drop,
lost/full/closed notification channels and duplicate notifications must converge to one
binding/audit without dispatch/fail/retry; the notification-only mutant must fail.
These actual19/23/43 source-positive and14 authenticated-restart gates remain pending.


### Design27: drained legacy history and exact unbound failure closure

Migration refuses EVERY pre-epoch Workflow owning a nonterminal Task regardless of
Workflow.finished. Actual5 QUICK Pr sets finished=true/active=None with TaskPrCreated;
request_finalization later resumes MergeGate/Cleanup. This is operational post-effect
history, not drained terminal history. Refuse unchanged DB bytes/version/audit and
require compatible old-runtime completion or explicit terminal cancellation/drain
and actual external reconciliation. TerminalTask+activeNone, even finishedfalse, may
migrate as validated non-owning immutable history. No pre-epoch Workflow gets fabricated
context counter/tail/admission rows or post-epoch progression/finalization. S6-01 includes
actual old5 QUICK PrCreated, finished ReadyForPr nonterminal and terminal non-owning
controls; removing the finished-independent predicate must admit the dangerous real
fixture. Ordinary/recovery availability limits remain explicit, never a hot-upgrade claim.

A separate private KnownCurrentFailureClosure handles an allocated but unbound active
attempt after a genuine sealed KnownCurrentTerminal FAILURE and complete actual settlement.
It leaves history.session_id None and cannot invoke success-only43binding or infer outcome
from start-Err/Session label/public enum/JSON. Its transaction checks original immutable
marker frame against current P/G/T/Workflow/context/input/actor/source/full locks, exact
operation/allocation/current Session body/version/admitted-consumed provenance and genuine
known failure receipt. It writes only the policy-authorized non-success phase closure,
operation.phaseClosed and one bounded factual failure audit atomically, plus actual
existing23/8 authorized lifecycle consequences in their coordinated transaction contract.
This is not an inactive-owner exemption. TaskFail requires actual existing lifecycle policy;
ordinary PhaseFailed may preserve an admissible Task for policy-authorized retry. Subsequent
retry is a NEW independently admitted operation/input under current policy and shared review
lineage; no receipt reuse, immediate native input or unrelated claim/owner release.
Missing/stale/unknown outcome/cohort cleanup stays held14. Actual managed supervisor+
authoritative driver lost-start-result failure→exact close then fresh retry is S6-09;
forged/stale/unknown receipt, original-frame drift and public start-Err negatives retain
claim with zero retry dispatch. Consumer closure/receipt/source checks receive causal mutants.
This actual19/23/8 source port remains pending, not an implemented Phase variant/API.

S6-05 explicitly covers consumed Starting, live own checkpoint append, then historical
Running ACK succeeding with original input pins. Native-originated permission cancellation
(including Claude control_cancel_request) is private historical observation with the existing
provider-specific target-state/activity predicate; Blocked→Running still rejects. Codex
OperationDecision::Cancel is a fixed non-grant protocol response via DenyHistorical, never
ALLOW. S6-05/06 inventory both actual writer paths and their valid/currency/lifecycle no-wire
controls; failure keeps owned cleanup, no fallback to generic writes or invented permissions.

Typed per-family pack facts must preserve exact prior-list prefixes and old scalar facts
for Continue. Drop/replacement/reordering or an unclassifiable projection forces Fresh,
even if checkpoint head also advances; an aggregate pack_facts hash supplies no comparison
proof. Old Decision removal and failing→passing replacement controls and selector mutant
reach the real pre-marker routing. Caller factual classification remains unverified.

All-table SQL fencing names the new observation-source/outcome tables and composed23
Goal authority explicitly; schema enumeration includes every application table. Each
JSON-reading marker/receipt/body trigger independently checks encoded length before its
JSON calls using explicit sequential short-circuit CASE or one ordered trigger body;
SQLite BEFORE-trigger order is unspecified. S6-13 oversized rawSQL body control and
length-before-JSON mutant verify bounded rejection without table/audit changes.


### Design28: original binding pins, complete QUICK tail and held cleanup facts

Historical component5 acknowledgement refresh is labelled as such in BOTH requirements
and design. Schema6 normal/receipt43 binding always compares original resulting marker
P/G/T/postmarker Workflow/full-lock frame; no sibling CAS adoption. S6-10 actual binder
refresh-expectations mutant after sibling Goal version advance must reject and retain
claim/receipt. Ordinary historical native acknowledgement is not SessionID binding.

Before QUICK Pr, reserve its documented normal request_finalization tail (MergeGate,
Cleanup, frozen terminal context/history/closure headroom) even though QUICK execution
stops at PrCreated. S6-13 QUICK used253+3 fits versus254/255+3 refusal beforePR; actual
later finalization consumes reserved tail and terminal persists after disposal. A mutant
reserving only the immediate PR-only plan must reproduce after-effect stranding. All
supported later extensions and simultaneous body/blob/history/context bounds participate;
unknown required extension refuses before first irreversible effect.

Actual factual settlement can publish after an operator Task WaitingHuman/lifecycle hold
without requiring launch admission lifecycle or current P/G/T version expectations. Scope,
operation, current owned Session body/version/input/actor and actual cleanup proof remain
exact; no receipt means no cleanup authority. It does not rewrite hold/state, grant input,
refresh original43 pins or permit active failure closure through stale marker authority.
S6-08 hold after Starting→consumption refuses→actual cleanup receipt positive must be
reachable. Closure follows ONLY its separately reviewed23/8/19 policy: stale active frame
remains held, or actual typed terminal decision plus eligible TerminalRecovery can close
non-success. This avoids a receipt-publication deadlock without inventing generic unhold
or inactive success binding. Admission-lifecycle-in-settlement mutant must fail this test.
S6-09 also proves bound restored-prior SessionID retained byte-exact and unbound remains
None, never clearing/rebinding to satisfy a table. No restored receipt proves current success.


### Design29: historical branches, host effects and bounded durable results

The canonical observation port requires genuine prior private admission/preparation
and the actual owned operation/runtime instance, open+unsettled+effects_started,
remembered own Session Record version, exact input/actor/native/pair/consumed UUID
pins and operation-pinned complete scoped lock set. It never obtains expected authority
by reading current rows and matching them to themselves. Branches are explicit:

| Historical branch | P/G/T and activity currency |
| --- | --- |
| Consumed Starting→Running ACK, pending permission observation and any Running-target publication | Full remembered P/G/T CAS from genuine admission/own reviewed CAS, pinned full locks and existing provider-specific activity/target-state guards; semantic sibling/operator version drift may conservatively retain ownership for14, no refreshed expectations |
| Exact Codex unchanged-Waiting fixed Decline (and its no-grant audit) | Retained P/G/T are provenance ONLY; exact own Session/op/input/actor/pending request and pinned locks plus existing provider/activity/target-state guards; no raw version re-admission, refreshed tuple or grant |
| Monotonic Lost/uncertainty diagnostic | Retained P/G/T are provenance ONLY; exact own Session/op/scope/input/actor/consumed intent and pinned locks; only conservative Lost/uncertainty additions/PID clearing, never admission/release/success |
| Actual factual settlement | Separate real settlement producer, exact owner/body/input and complete actual cleanup; factual receipt under lifecycle hold, not an observation or grant |

Preserve actual provider differences: Claude Blocked Waiting→Running DENY cannot wire;
Codex unchanged Waiting Decline may wire under its existing safe own-Session predicate,
then postreply Running publication stops on Blocked/current remembered CAS. Task metadata
version revocation does not turn that fixed Decline into ALLOW. A conservative failed ACK
leaves the consumed operation owned for cleanup/recovery; it never fabricates lost delivery.
S6-05/06/07 include sibling Goal version bump, operator Task hold, per-provider Blocked
and remembered lock changes, exact unchanged-Waiting Decline positive and monotonic Lost
DB/watch consistency. Mutants swapping AllowCurrent currency into historical exceptions
or dropping remembered CAS from Running-target ACK must reach these real consumers.

HostCallbackCurrent is DISTINCT from a native permission reply and native-internal tool
execution. Actual registered policy/effect authorization plus the genuine operation handle
binds exact callback method, canonical FD-anchored scoped path, read/write byte bound and
write-content digest. Native ACP JSON alone cannot mint it; no imaginary Broker decision
or new model-input consumed UUID. Before filesystem IO its transaction admits that exact
current effect with full AllowCurrent lifecycle/semantic P/G/T/own Session/locks,
accepted23 Running/nohold/DAG predicates, CPP/checkpoint head and immutable governing/
reference authority; mutable Executor content uses only the scoped Design26 exemption.
Admission is uniquely journalled against this operation/current request before the host
performs IO outside SharedStore. The admitted effect stays owned through its actual outcome;
revocation before admission emits zero IO, later revocation stops further admissions and
retains required cleanup, never drops an already admitted effect. No OS-atomic filesystem
or foreign-writer attribution claim is made. Governing rule/config, promoted/foreign and immutable-input reference targets cannot
be overwritten through this callback. Runtime-declared current Executor outputs follow
the exact Design32 output contract; a public artifact kind/path is not permission. Reviewer reads retain exact snapshot and native
read-only profile; all runtime-private/current-peer/foreign Project/context paths remain
ineligible even when OS-readable. Actual in-process FD path/effect ownership and containment
are prerequisites, not a generic callback label. Grok actual authorize_write+owner callbacks
must migrate to this real private port; missing producer refuses before host read/write.
S6-06 actual owned write positive, Blocked/Paused/Cancelled/head/rule/lock stale negatives,
governing-target write refusal and caller-reachable host-effect admission mutant prove it.

Continue checkpoint projection separates semantic retained events from condensation:
retained must be an exact prior-list prefix and mandatory_goal/mandatory_task/rules
snapshots unchanged. Legacy snapshot semantic or otherwise unclassified changes conservatively
force Fresh; no silent exclusion of a newly added authority field. Recent window, omitted_*
summary, recent_bytes, previous/chain_version, sequence/input-digest and measured-token
bookkeeping are compared through exact validated chained provenance/consecutive head;
normal transient-window pruning does not by itself force Fresh or permit retained-event
removal. Exact field inventory is projection-versioned. Window overflow+retained Decision
append may Continue; dropping/replacing a retained Decision while advancing transient
window routes Fresh. A selector mutant must reach actual routing, not digest-only unit tests.

KnownCurrentTerminal receipt records native lifetime/outcome/cleanup attribution, not a
review verdict or PhaseGates success. Actual supervisor settlement atomically retains a
bounded immutable owned result artifact (maximum1 MiB encoded bytes or overflow prefix
with explicit nonapproval status), exact digest/projection and scope/operation/Session/
context/admitted input references. Native callback result text is unverified proposed
content until actual phase gate validation. Success after dropped result/restart needs
this exact artifact plus actual PhaseGates Passed evidence bound to the same Session/context;
missing/unavailable/malformed/overflow result stays Waiting/held, never legacy statusNone
acceptance or receipt-only success. Restart still requires actual14 private authentic
recovery, not reconstructed live owner. S6-08/10 durable successful result+gate proof,
missing result, nonapproval Reviewer verdict, receipt-only gate mutant and restart hold
controls preserve this distinction. No private executor conversation goes into public
review packets or peer inputs.

Private workflow_context_counters also charges total used/reserved immutable observation-
source/outcome/native-result blob bytes perWorkflow:128 MiB finite initial policy bound.
Every admitted effect reserves actual known worst-case source/result/metadata and complete
irreversible/factual-settlement tail capacity BEFORE IO/model bytes; missing bound refuses.
Dedup is charged once only for proven exact scoped owned identity, every actual copy costs
bytes. Optional progress cannot spend reserved final result/receipt/closure/tail capacity.
The2-MiB source/1-MiB outcome/64unique/256history ceilings are simultaneous, never a promise
maximum sizes all fit128 MiB. Large generic tail contracts may therefore refuse beforePr;
required actual supported profiles must provide fitting real bounds, not raise/default-hide
capacity or call universal Unsupported MVP success. At quota refusal retain prior evidence/
ownership with explicit NeedsContext/hold attention and zero next effect. Actual used+reserved
byte CAS, rollback/dedup/copy boundaries, realistic repeated large blobs, protected terminal
publication after disposal and quota-removal mutants belong S6-13. Application-wide storage
and OS/runtime memory remain separate bounds; no delete/reset of mandatory blob history.

The ONLY schema6 classification order is bounded bytes/depth/nodes, then universal
Lost/terminal→Lost refusal using bounded state decode, then private port dispatch,
strict branch DTO parsing and that branch's currency checks. RestoredPrior is settlement-
ONLY inside the same atomic receipt transaction, unreachable from NativeCAS or generic
managed writers. NativeCAS admits only its allowed validated/current or historical outcomes;
head-validated Running without consumed UUID never admits. S6-01/04/08 raw oversized body,
Lost before outcome branching and NativeCAS-restoration routing mutants pin this ordering.
Earlier component5 launch-guard ordering is historical and cannot authorize these branches.


### Design30: host-effect settlement, bounded journals and parallel authority

HostCallbackCurrent admits a unique effect before actual host IO into private append-only
`native_host_effect_admissions`; actual completion appends exactly one row in
`native_host_effect_outcomes`. Both are candidate6 application tables, included in
every all-table exact-version function fence, trigger semantics, migration/golden
enumeration, FK/unique guards and schema refusal controls. Generic SQL/Record JSON or
a request ID cannot mint either private producer. The actual owned runtime derives
operation/scope/attempt/frame, remembered Session pins and request identity from its
nonserializable callback handle, not native request-supplied credential fields.

Admission key is (operation UUID, monotonic effect sequence 1–256), with a unique
(operation UUID, native request identity) constraint. Each complete encoded admission
is at most 8192 bytes: registered method/effect kind, exact anchored scoped relative
path (at most 4096 UTF-8 bytes), read/write byte bound, write SHA256 where applicable,
source/current authority and actual producer/profile references. Each complete outcome
is at most 2048 bytes: the same effect key, known finished success/error classification,
actual bounded bytes/digest and factual reason. No raw file content or repeated full
source maps enter these rows; referenced owned material has its own charged blob bound.
Identical request retries NEVER repeat IO. A duplicate read returns the fixed protocol
AlreadyProcessed error: retained digest/bytes are not raw content and cannot fabricate
a read response. A duplicate write may replay only its existing known acknowledgement;
pending/unknown outcomes return their fixed held status. Identity
collision with differing payload rejects. Sequence 256 admits within quota; 257 refuses
BEFORE IO with attention. All exact encoded admission bytes plus worst-case outcome
bytes reserve under the Workflow's used/reserved 128-MiB quota before each IO. Outcome
headroom cannot be consumed by optional progress, even after lifecycle revocation.

Every admitted host effect remains owned until its actual worker is joined/settled
and its known outcome is durably recorded. Cancellation/timeout of a spawn_blocking
JoinHandle does not stop or certify that worker. Unknown late write, missing outcome
or unjoined IO prevents BOTH NoCurrentDispatch and KnownCurrentTerminal receipts
and retains operation/resources for actual14 recovery; native end_turn alone cannot
clear it. Known IO error may support the actual non-success lifecycle policy after
complete settlement, never fabricated success. S6-06/08/13 actual callback tests cover
256/257 and exact row-size/quota boundaries, duplicate no-IO, uncertain admitted write
after terminal, later actual worker finish, known error, forged outcome and caller-
reachable omitted-host-outcome/limit/quota mutants. Current Grok's incidental loop
limit is not this contract or a producer implementation.

Managed phase commits/ref/index changes are not mutable content under AllowCurrent.
EvidencePort Commit/base-sync phases belong to their actual #60 owned Git producer.
If an Executor tool nevertheless moves HEAD/index/refs, later grants refuse on exact
HEAD/governing/index checks and unexplained reconciliation stays held; this is an
explicit availability limit, not attributed safe own movement. A producer that can
prove and classify a forbidden ref/index-mutating tool refuses that effect before
wire; arbitrary shell JSON does not prove effect classification. No supported profile
may promise transparent mid-turn commit progress under this policy. Controls actual
owned content-write success versus actual ref/index movement→zero later ALLOW/held
reconciliation preserve the distinction; no hooks/config are disabled.

Private managed Usage/audit entries are at most 64 KiB encoded each and reserve their
producer-declared finite entry/count/byte envelope in the same 128-MiB used/reserved
accounting before the attributed effect. Reserved terminal receipt/result/closure
audits survive quota pressure. Unknown bounds refuse pre-effect; no loop of optional
managed audit writes bypasses quota. Generic application-wide audit/Usage storage
remains outside this per-operation authority/storage claim and never creates native
proof, changes a frozen Session or releases an operation. All its writer paths still
require the schema6 exact-version fence. Actual private producer bounds/overflow/
rollback controls are source acceptance, not current generic Store behavior.

Raw original #43 marker P/G/T/Workflow/full-lock pins and AllowCurrent/HostCallbackCurrent
authority fences remain exact. Do not replace them with an unapproved relevant-Goal
projection, later current-row self-match or allocator-time SessionID write. Adding
authoritative sibling DAG Tasks while A is live can invalidate A's grants or unbound
success; that unresolved currency/availability result cannot satisfy parallel Goal
MVP acceptance. A concrete proposed #23/#24 composition derives progress/status from
Task facts or separate bounded factual Records without rewriting Goal.version;
already accepted prepared DAG Tasks execute concurrently without Goal-row rewrites
per Task poll/completion. Authoritative DAG/follow-up acceptance and lifecycle changes
queue until applicable exact native phase closure (including binding and closure, not
receipt publication alone), or deliberately revoke under
the existing strict gates. This is a candidate controller policy, not implemented
progress, external-bookkeeping exemption or a release waiver. Actual #19/#23/#43/#24
source integration must prove 4+ Tasks, follow-up proposal/creation and two Projects
with exact grant/binding currency plus safe held/recovery outcomes; unresolved
availability remains a whole-source/MVP release gate.

The separate proposed43 bound-live observe_native_diagnostic transaction writes only
bounded factual diagnostic, Workflow Record version/updated_at and reserved audit;
it checks exact private operation/attempt/current Workflow CAS while preserving
ALL P/G/T/Session/lock/native authority pins. It never refreshes owner versions,
grants input, fails/retries a phase or reconstructs a live handle. Duplicate factual
diagnosis is no-write. Both status-error and persisted/status-mismatch callers need
actual integration/held-turn currency controls and each Task-rewrite/authority-refresh
mutant. Passive step/poll remains unable to settle/release ownership.

Protected checkpoint success is not a current native settlement class. Standalone
launch/resume/checkpoint remains Unsupported before effects. A future no-dispatch
input-update publication route needs separately reviewed actual19 atomic authority
and actual60 independent Git reservation/settlement plus any native cleanup conjunct.
Neither NoCurrentDispatch nor native KnownCurrentTerminal may be borrowed to invent
a successful checkpoint receipt. This integration remains pending.


### Design31: exhaustive authority writers and complete stored bounds

The canonical Session/Workflow table, the following table and the explicit
Design35 extension rows are the exhaustive union. Each row is a required real source consumer + rollback/control + causal mutant
in the listed S6 keys. Private handles are genuine owned producers; table/row/JSON
existence is never a constructor. All new application tables require length-first
validation, exact-version writer fences, scope/FKs, uniqueness and no-REPLACE/Delete
semantics in both fresh/migrated goldens. These are candidate6 design, not current
schema5 source or installed production ports.

| Authority write | Only actual private producer / allowed write set | Same-transaction authority | S6 keys |
| --- | --- | --- | --- |
| Operation INSERT + marker | Successful exact Engine invocation/marker owner, immutable original frame | Current composed23 admission + full scope/attempt/source/lock CAS, no maintenance conflict/pending Goal change; predicted resulting versions verified before commit | 02/03/10/14/16 |
| effects_started false→true | Owned Pristine handle transferred to actual supervisor | Exact open own operation/version, no receipt; monotonic transition before first Session/IO | 03/08 |
| Pristine refusal receipt | Consuming settle_refused_before_effect | Actual tracked pristine invocation, no effects/Session/allocation/admit/consume/uncertainty; no row-only proof | 08/09 |
| Session NULL→Some + preparation/allocation | Actual private preparation under owned effect-started operation | One-time immutable exact owner and frame, actual23 current admission, remembered version installed atomically | 02/03/04 |
| Remembered Session/operation version update | Exact private preparation/admission/historical observation or settlement | Corresponding canonical branch only; immutable original marker frame and consumed input unchanged | 04/05/06/07/08 |
| phase_closed false→true | Actual receipt-checked Workflow closure | Exact current claim/body/version/pair/receipt, allowed lifecycle; only matching phase closure, no receipt-only release | 09/10/16 |
| NoCurrentDispatch non-success closure after pre-input revocation | Actual sealed full-own-cleanup/no-current-input settlement consumer | Original marker/claim/actor retained; exact current closure CAS preserves lifecycle/decision, no binding/success/advance/grant | 08/09/16 |
| KnownCurrentFailureClosure | Actual sealed known-current failed settlement consumer | Original immutable marker/input/actor/locks and current active claim, no SessionID write, existing23/8 non-success policy | 09/10/16 |
| TerminalRecovery | Exact actual private settlement/recovery consumer | Factual full own cleanup under Cancelled/Failed hold, same attempt/body/input/receipt; no fresh grant/success/inactive binder | 08/09/16 |
| Receipt continue_refusal attribution | Actual owned Continue selector/refusal producer in settlement transaction | Immutable closing receipt field with exact predecessor/new operation/input chain and reason; bounded history256 chain, cannot be cleared by a later pristine attempt | 11 |
| Maintenance acquire/close | Actual owned maintenance/effect producer (production60 pending) | Reciprocal durable physical conflict check; Held→Closed only actual NoEffect/Settled, unknown retained | 14 |
| Host effect admission/outcome | Actual HostCallbackCurrent / owned joined-worker completion | Unique admitted request and quota before IO; outcome only matching admitted effect, actual worker-known finish, no receipt with unknown work | 06/08/13 |
| Workflow counters/quota reserve/spend | Actual context/effect/receipt/closure producer | Scope/version + full-body byte charge and terminal headroom before effect; only safe unspent reserve release, used history never reset | 13 |
| Source/outcome/native-result blob INSERT | Actual captured source/owned result/settlement producer | Exact typed immutable bytes/hash/scope + quota charge, no row-only native verdict; immutable identity dedup only | 08/10/13 |
| New-Task epoch INSERT | Schema-owned Task AFTER INSERT trigger | Exact inserted scoped Task, boundary zero, exact-version fence; Task insert rolls back if epoch fails | 01/02 |
| WorktreeLock activate/change/deactivate | Actual authorized lock/closure consumer, no generic owner release | All matching native operations/maintenance reservations counted regardless Session label; no lock mutation/release while its operation/phase owns it except exact required final closure | 03/09/14 |
| Pending Goal authority barrier | Actual trusted23/24 queued-change producer | Durable scoped accepted proposal + expected Goal version, no Goal version change merely to queue; marker refuses while pending | 16 |
| Task non-lifecycle edit while operation open | None through generic put_task | Operation-aware typed hold; issue/title/criteria/executor/reviewers cannot rewrite native-fenced Task/version during open phase | 05/10/16 |
| Project non-lifecycle edit while operation open | None through generic put_project | Project-wide open-phase hold, including name/max_tasks bookkeeping; authorized lifecycle Block remains available, remove requires actual complete idle ownership | 06/10/16 |
| Compact prerequisite certificate / revocation | Actual23 trusted full-proof publisher / authorized revocation | Out-of-lock evidence extraction plus exact publication CAS; immutable scoped proof and monotonic revocation, bounded native transaction reads | 02/04/06/16 |
| Failed closure / awaiting_explicit_retry | Actual receipt-checked non-success closure / explicit retry controller | Atomic failed history + active None + durable exact retry hold; separate explicit RetryEvent clears only that hold, never reopens receipt | 09/16 |

**Immutable original marker columns and insert ordering.** native_phase_operations
explicitly stores marker_project_version, marker_goal_version, marker_task_version,
marker_workflow_record_version and canonical full marker_lock_set plus SHA256 digest.
The immutable marker_workflow_body_sha256 additionally anchors the exact canonical
planned complete Workflow Record body; its byte recipe is projection/version pinned.
The full sorted ID/version list is bounded to 256 entries/64 KiB before marker; its
exact encoded bytes also fit the existing aggregate authority-tuple bound. The columns
are immutable under EVERY UPDATE/REPLACE/Delete guard, including private bookkeeping
updates. At operation INSERT, derive resulting values from the transaction's exact
validated write plan: unchanged P/G stay expected; each actually written Task/Workflow
row uses expected+1 with checked arithmetic, not a later current-row read. The ordinary
marker put_task_tx path writes Task and therefore records expected Task+1. Any future
change in marker write set must change this explicit derivation/golden, never guess.
The marker UPDATE trigger for false→true checks NEW Workflow.version equals recorded
marker_workflow_record_version; the Rust transaction verifies the resulting Task/P/G
versions, full lock set and complete final relation before commit. Other legitimate
Workflow observations do not rewrite these columns. A failed second statement rolls
back operation/marker/audit. Actual differing resulting version, refresh-column UPDATE
and dropped-start-result binder controls/mutants prove the immutable frame.

**Durable maintenance surface.** Private task_maintenance_reservations stores UUID,
Project/Goal/Task FKs, actual anchored canonical root/worktree/common-Git identities,
registered bounded effect kind/physical read-write conflict scope, runtime-instance
and private claim UUID, monotonic version, Held or Closed{NoEffect|Settled}, created/
closed timestamps and exact actual settlement reference. Complete body ≤64 KiB; paths
are separately bounded by the existing scope inventory. One open reservation per
exact physical conflict key has a partial unique index. Acquisition and native marker
are reciprocal Immediate transactions: acquisition rejects every overlapping open
native/58/60 effect reservation, marker/preparation/input rejects every overlapping
Held maintenance reservation. Conflict includes canonical path containment and actual
shared common-Git effect scope across Project IDs, not string equality/Task labels;
partial uniqueness is a backstop, not the nested-scope algorithm. Actual compiled
58/60 conflict producer supplies complete bounded anchored scope observations outside
Store and the same-transaction conflict reader; absent/unknown producer refuses.

The actual private owner retains Held across out-of-lock Git, timeout/crash/unknown
cleanup. Only genuine tracked no-effect refusal or full known actual owned cleanup
closes Held→Closed once, with matching immutable scope/claim/runtime and receipt plus
reserved audit. No public terminal JSON, Drop, stale claim or caller Closed enum may
close it. SQL fences reject immutable-field updates, reopen, DELETE/REPLACE and unproved
closure; actual closure rows retain history. True disjoint scopes may progress when
the actual physical producer proves them. S6-14 uses the same real candidate reservation
+ conflict transactions through explicitly closed synthetic mechanical producers,
never inserted proof rows; native60 readiness remains a separate real producer gate.
Crash reopen retains Held; acquisition/marker race has exactly one winner BEFORE
Git/removal bytes, nested-root/common-Git conflicts hold and truly disjoint controls
progress. No production containment inferred from selected group death.

**Complete ContextVersion bound.** Every newly published Workflow-owned ContextVersion
complete encoded body is ≤8 MiB, including payload escaping, task_pack copy, all source
maps, wrapper/metadata and references. Actual Rust computes full stored UTF-8 bytes
before publication; a length-first context_versions trigger enforces the same bound
for post-epoch Workflow ownership before JSON inspection. Standalone/Goal/old immutable
rows retain their separately scoped rules; no historical rewrite. The 1-MiB prepared
model frame remains independent, and worst component maxima need not jointly fit.
Every actual stored Workflow context body charges used/reserved 128-MiB quota atomically
with context counter/pointer/Task/Workflow/audit. Duplicate pack bytes are counted;
no claimed reference dedup when two copies really persist. Terminal tail reservation
includes worst complete frozen ContextVersion body bytes, not merely payload/count.
Capacity insufficient for the whole context/tail refuses before external effect with
NeedsContext, preserving current authority/history. S6-13 actual source-map/pack duplicate
and escaping fixtures exercise 8-MiB−1/exact/+1, repeated-context aggregate quota and
ContextVersion length/quota mutants. Storage ≤256×8 MiB and ≤128 MiB charged complete
context/blob/managed-authority bytes per Workflow; no unsupported 256×payload claim.

**Goal change drain barrier.** Genuine trusted23/24 queued authority changes use a
durable scoped pending-request/barrier under accepted policy and original expected
Goal version. Queueing does NOT rewrite Goal.version. ALL new native markers for that
Goal observe this same pending barrier in their admission transaction and return a
typed retryable pending-authority hold without claim/input; already owned phases retain
their exact historical observation/settlement rights. A queued authoritative mutation
may apply only after EVERY open phase that pins that Goal version has phase_closed=true
(binding+closure or genuine non-success/TerminalRecovery closure), never merely after
receipt publication. Do not strand a successful unbound receipt by applying first.

The actual23/24 controller drain/check/apply transaction revalidates complete current
ownership and expected Goal authority, applies only authorized additive DAG/lifecycle
policy, then clears the barrier atomically. New marker admissions remain blocked until
that transaction completes; repeated scheduling cannot starve the pending change with
new phase openings. Lost/unknown/held closure preserves barrier and honest14 recovery
attention; no bounded-time drain promise for unknown native work. A stale/cancelled
proposal may be retired only by its actual trusted policy, with factual reason and
no native release. Accepted Goal definitions remain immutable. Source acceptance needs
actual19/23/43/24 producer/controller: settle→pending change BEFORE lost-result binder
remains blocked, later exact bind+close permits change, and continuous healthy parallel
requests cannot prevent drain/follow-up creation. This is no completed controller or
implicit version-refresh privilege. Generic Task issue/title/criteria/executor/reviewer
updates likewise reject with operation-aware typed hold until exact phase closure;
typed lifecycle revocation remains available without treating it as cleanup.

**Historical binary/encoding evidence.** PRAGMA and DDL are outside table-trigger fences.
Every identified historical v1–v5 executable has an exact source/binary hash and native
fresh-open refusal test against candidate/composed higher-version/RRXC fixtures, with
unchanged original DB bytes/version/schema. Verify version-range check occurs BEFORE
PRAGMA/DDL mutation; already-open old4/old5 writer fence tests are additional, not a
substitute. A supported old binary that can mutate markers must be explicitly held
unavailable until actual safe drain/version coordination, never assumed protected by
a table trigger. Record each historical build's resolved serde_json feature set and
encoding recipe with its binary hash (Cargo tree/build evidence); preserve checkpoint-v1
bytes accordingly. The proved default sorted-map recipe is not evidence that an arbitrary
feature-unified preserve_order build wrote sorted bytes. Unsupported/unrecognized old
encoding explicitly refuses migration unchanged; no rehash/adoption of ambiguous
chains. Actual known historical recipes, both-configuration goldens and old binaries
prove the supported migration contract in source acceptance.


**Original frame reaches actual callers.** ManagedPhaseLaunch carries the sealed
Store-derived original marker P/G/T/full-lock frame into every adapter ScopeSnapshot,
first preparation, first/new consumption and AllowCurrent. An adapter may read current
rows to compare against that ORIGINAL frame; it cannot construct admission expectations
from newer current parent rows at start. Actual source/grant revalidation preserves
the original frame. Drift BEFORE setup/first input yields the actual pristine refusal
or tracked cleanup NoCurrentDispatch path; after consumption it cannot be called
NoCurrentDispatch. Actual each-provider caller controls/mutants must prove no changed-
parent native setup/input bytes and no later binder stranded after silently renewed
admission. Component standalone ScopeSnapshot captures are not this managed proof.

For genuine fully settled pre-input NoCurrentDispatch only, the distinct private
non-success closure may tolerate now-revoked raw P/G/T versions. It retains the exact
immutable original frame, current attempt/claim/actor/private preparation/restoration
pins, and proves no CURRENT admission/consumption/uncertainty plus complete actually
owned setup/tool/host-effect cleanup. Current rows are freshly read only for exact
closure-write CAS and preservation of their actual decisions/holds, never re-admission,
actor binding or renewed source authority. The actual23/8 lifecycle port may record
Interrupted/Failed non-success closure of that exact attempt; no phase advance,
Succeeded, fresh input, clearing Pause/Blocked/Cancelled/Failed or release of unrelated
owners. Genuine receipt/phase closure retires the exact lease, leaving any independent
policy/resource holds intact. Public Err/row labels/JSON/NoCurrentDispatch enum cannot
mint this closure. Any consumed input, unknown native/effect work, missing/stale/foreign
receipt or restored-prior input falsely presented as current success retains ownership.
Cold restart requires actual14 authentication, not a reader constructing live authority
from rows. S6-08/09/16 actual revocation-before-consumption with full cleanup positive,
revocation-after-consumption, forged/missing/unjoined cleanup and lifecycle-preservation
negatives plus causal original-frame/closure mutants are required. KnownCurrentFailure
and successful closed-receipt43 binding retain their stricter original current fences.

Proposed43 settlement readiness is derived from the exact marked/unbound/phase-open
private operation and actual receipt/body/input state; there is no separate caller-
writable ready row. Binding or genuine closure naturally retires eligibility. Actual
per-Project operation capacity is reserved INSIDE the same Immediate41 claim transaction
before commit/effects. Derive it from durable active native claims (marked or unmarked)
union phase-open19 operations, deduplicated by exact logical attempt. No standalone
preclaim token/retirement row:41 closed preparation/invalidation removes its claim;
orphan claims count, closed history does not, and activeNone with an open operation
still counts. Marker validates its already counted claim without charging a second slot.
Parked held scans use
nonauthoritative changed-version/recovery-event cache without repeated Immediate
mutations. These are actual43-owned co-integration requirements, not available APIs
or exceptions to14 restart/runtime-instance and original-frame authority.


### Design32: exact failure lifecycle, bounded gates and phase output currency

**Failed closure and explicit retry.** Current component `Engine::fail` keeps active
and bumps Task; later `retry` expects active. Those schema3/5 transitions are historical
consumer context, not the candidate managed failure state machine. Candidate managed
KnownCurrentFailureClosure and bound non-success closure validate original P/G/T/
context/source/locks and the Design33 legitimate Workflow successor BEFORE their own writes and perform one Immediate transaction: mark that exact attempt
Failed (or Interrupted under actual23/8 policy), retain result/receipt and reason,
clear its active index, publish matching phase_closed, and set durable Workflow
`awaiting_explicit_retry={attempt_index, generation, phase, reason}` while Task becomes
WaitingHuman if current policy permits. No RetryEvent is fabricated at failure time.
NoCurrentDispatch under revocation preserves actual lifecycle holds as Design31 says;
terminal Cancelled/Failed Tasks remain terminal and cannot gain retry permission.
Ordinary step/poll sees this exact retry hold and returns Waiting without a new marker,
context or automatic next_phase dispatch. Later explicit authorized retry validates
current lifecycle/source/policy, exact closed preceding attempt and genuine settlement,
records one RetryEvent, clears only this retry hold/blocker and lets the NEW operation
obtain independent admission. It does not compare current Task against the pre-failure
marker version to close the already closed operation, refresh original pins, reopen a
receipt, or reuse old input. Irreversible phases still use their actual resume/reconcile
policy, never ordinary retry. Stale/duplicate retry decisions are no-write/refused.

This is the sole managed fail→retry path in the actual Engine/23 driver, including
unbound dropped-start-result failure. Existing fail/persist cannot pre-bump Task and
leave a managed terminal-but-active attempt. S6-09 requires actual bound and unbound
failure→Waiting→explicit RetryEvent→fresh operation, step-before-retry zero marker,
revoked lifecycle preservation and rollback. A pre-failure-version closure mutant,
missing retry-hold mutant and retry-receipt-reuse mutant must reach these consumers.

**Bounded repeated prerequisite gate.** Actual23 publishes compact immutable scoped
prerequisite certificates only after its genuine complete evidence/receipt extractor
runs outside SharedStore. Each certificate pins Task/Workflow/evidence/settlement and
accepted governing dependency digests; exact snapshot/version/source revalidation and
certificate insertion are atomic. Public Merged/Completed labels, JSON hashes or a
caller-supplied certificate cannot construct this authority. Revocation is separately
monotonic, produced by actual authorized lifecycle/source/definition changes. Current
marker/preparation/consumption/AllowCurrent/HostCallbackCurrent transactions read at
most128 hard-prerequisite compact rows, each at most4096 encoded UTF-8 bytes and at
most512 KiB total, plus their bounded scalar authority/revocation guards. They do NOT
re-extract full Workflow/result/receipt blobs under the SQLite write lock. The same
exact compact identities and revocation values are pinned into the operation; later
revocation-only admission cannot silently adopt a newer certificate. Oversize, missing,
unknown or revoked proof refuses before effect. Cold restart needs actual14 runtime
ownership proof in addition, not row-based live-owner reconstruction.

The certificate source/format/revocation consumer is actual23 co-integration work, not
an invented available API. S6-16 requires the maximum row/byte boundary,128/129 and
4096/+1 cases, genuine prerequisite success/revocation, out-of-lock full extraction,
and an actual second-Project native publication while a maximal valid gate is checked.
Report observed contention/latency, not an OS scheduling guarantee. A full-proof-inside-
transaction mutant must fail the bounded-read instrumentation/consumer control.

**Project bookkeeping fence.** Design35 additionally forbids generic managed
instruction-definition edits even in idle windows. Generic put_project rejects non-lifecycle changes with
a typed operation-aware hold while ANY phase in that Project is open, even an unbound
or session-less operation. Name/max_tasks changes cannot bump the raw Project authority
and silently stop all native turns. No-op generic writes also cannot bump this version
while held. Explicit authorized Block/lifecycle revocation remains possible under its
existing predicates; it does not certify cleanup. Removed still requires complete idle
ownership and physical exclusion, not a blanket lifecycle exception. Future trusted
bookkeeping proposals may queue separately without rewriting Project.version, but no
unimplemented queue grants permission. S6-06/10/16 requires live permission wait plus
unbound-window rename/max_tasks rejection with unchanged Project body/version, genuine
Blocked zero-ALLOW, and a dropped-fence mutant reaching actual native/43 consumers.

**Declared phase outputs.** Before marker, the actual runtime phase policy constructs
and seals an exact output manifest: scope,phase,actor,Task-worktree-relative paths and
FD/root identity, tied to the immutable frame. A model/artifact-kind/path/JSON claim
cannot add outputs. Only these current Executor-owned output paths may vary during
AllowCurrent and receive HostCallbackCurrent writes after actual action/effect checks.
They remain captured in the immutable initial input and are recaptured for the next
phase; mutable mid-turn currency is not input re-publication. Governing rule/config,
accepted Goal definitions, promoted consultation, primary-root or foreign-Task targets
ALWAYS win the exact/read-only classification even if a path also appears as an output.
All undeclared own-Task input references remain exact; Reviewer references remain
immutable. A Requirements input used by Design does not become a Design output merely
because both are artifacts. Actual Requirements/Design/Implement/Fix phase policies
must declare their genuine writable documents/source outputs or refuse that write.

S6-06 requires real own-output edit→second ALLOW and own-output host-write positive,
undeclared own input/foreign/promoted/governing edit→zero ALLOW, output/rule overlap
refusal, actual HEAD/revocation/lock negatives and a caller-reachable classification
mutant. This permits ordinary artifact-producing phases without claiming same-UID
writer attribution, foreign isolation or native producer readiness.

**Byte checks and durable refusal attribution.** Every JSON-reading trigger checks
`length(CAST(NEW.body AS BLOB))` (the actual appropriate field) before ANY json_extract
in that SAME trigger, using a CASE/ordered guard whose semantics do not depend on
trigger execution order. SQLite length(TEXT) character counts are insufficient.
Canonical fresh/migrated DDL and raw SQL tests cover exact-byte multibyte bounds and
+1 for Workflow/context/source/outcome/session records, with original DB unchanged on
refusal. Rust checks actual encoded UTF-8 bytes independently. S6-13 removes each
byte-first guard meaningfully. Fresh-only refusal is NOT a separate unlisted table:
immutable settlement receipt attribution contains `continue_refusal={previous_operation,
previous_attempt,previous_input_digest,previous_frame_digest,reason_code,fresh_only:true}`,
written only by the actual owned selector/refusal settlement transaction. All-table
receipt guards freeze it. S6-11 scans at most history256 exact closed predecessors;
a later pristine no-Session attempt cannot erase this lineage fact.

**Capacity and reconciliation coordination.** The same Immediate41 reservation owns
the derived durable claim/open-operation capacity check before commit/effects; no
standalone43 capacity token. Logical attempt dedup preserves orphan/open-operation
holds and prevents closed-history leaks. A definitive original predicate mismatch
parks without writes. Transient Busy or known rolled-back audit/storage failure retries
fairly after100 ms–5 s with the SAME private operation/frame; uncertain commit performs
read-only exact reconciliation, not fail/retry/native redispatch. The actual driver
retains its bounded attention and ownership. S6-16 requires saturated/unmarked/bound/
closed/cancelled-open cases and contention/rollback/mismatch distinction. All actual
19/23/41/43/58/60 producer and native acceptance gates remain pending.


### Design33: factual Workflow successors and complete compact field inventory

**Immutable marker versus legitimate successor.** Original marker P/G/T/W versions,
body hash, context/source/frame, actor and full locks NEVER change. Unbound43 binding
and unbound failure still compare original W. Binding and bound diagnostic legitimately
advance W, so bound closure uses a sealed Store-derived factual successor proof for W,
not original-W equality and not a fresh current-row self-match. No binder-updated
operation column/table is added. Preserve43's allowed write set: Workflow Record body/
version/updated_at plus ONE bounded reserved audit in the same transaction only.

That audit is an append-only factual ledger link with exact scope/operation/attempt,
original marker-frame digest, checked predecessor/successor W versions, predecessor/
successor canonical COMPLETE Workflow body SHA256 and prior ledger digest. Binder event
is exactly `rrx.private.workflow.session_bound`; diagnostic event is exactly
`rrx.private.workflow.native_diagnostic`. At the first bind, predecessor is the immutable
planned marker W/body hash. Binder changes only the exact active history.session_id
None→allocated exact ID; diagnostic changes only that attempt's bounded detail. Each
producer checks its complete allowed projection and original immutable pins before
writing and installs the exact prevalidated serialized next W under current version
CAS. Its complete resulting body/hash is computed outside held SharedStore; compare
exact planned indexed successor/hash and checked W version before commit. Mandatory
body writes and SQL validation have the separate finite bounds of Design35, not an
assertion that SQLite handles only512 KiB.
Ordinary audit data, a reserved string in JSON, current rows or receipt IDs cannot mint
a link. Generic audit writers reject the whole reserved namespace. Canonical stored
body encoding/hash recipe is explicit; projection hashes alone cannot hide changed fields.

Generated/indexed audit operation/frame/predecessor/successor keys and uniqueness bind
exact consecutive W steps to the same private operation. Complete derivation is bounded
to256 compact links per operation, each at most4096 encoded UTF-8 bytes (aggregate1 MiB),
verified from the marker anchor through the latest genuine successor and exact current
Workflow body hash. No arbitrary history scan or current-row adoption. Within that
finite allowance reserve binding and all required gate/closure links before effects;
optional diagnostic exhaustion coalesces/no-writes with explicit attention, never spends
mandatory closure headroom or changes authority. Duplicate diagnostic is no-write.
The ledger remains charged to the existing Workflow128-MiB quota; index fields do not
hide actual retained bytes. Bind/diagnostic still never update an operation row, P/G/T,
Session, context, lock or native-grant body/version. The factual successor proof cannot
refresh any of those pins or authorize an input/ALLOW. A legitimate later Workflow
closure is not a new native binding or grant.

S6-09/10 requires genuine bind→diagnostic→failed closure→explicit retry-hold positive,
bind→successful phase-gate closure, diagnostic budget exhausted but mandatory closure
still succeeds, and rollback. Foreign Workflow version/body write, missing/different
chain, forged ordinary audit, altered marker anchor, illegal bind projection and
scope/operation replay hold without new input. A raw-SQL version/body drift mutant and
caller-reachable missing ledger/current-self-match mutants must fail. Actual43 source
and gate-owner integration is required; private SQL seeding is no proof.

**Managed gate reservation and observation.** Current component evaluate() writes
Evaluating/claimed_observations through Task+Workflow persist; observe_workflow_gate
already writes Workflow only. Candidate managed callers migrate to separate private
record-only gate-claim and factual gate-observation ports. They NEVER bump Task before
the final atomic native phase closure. They remain19/8 consumers, not43 binder or
native-diagnostic scope. Reserved ledger constants are
`rrx.private.workflow.gate_claim` and `rrx.private.workflow.gate_observed`, with the
same genuine predecessor/successor complete-body hashes, original operation/frame and
exact active index/generation/phase/Session/context. Claim changes only Evaluating and
its exact claimed_observations index; observation appends only that claim's actual
compact source/outcome/error entry and bounded detail, AND atomically applies the
actual typed result decision Evaluating→Waiting or heldFailed when required. This
fused decision is part of its complete allowed projection; no subsequent ordinary
Task+Workflow persist changes Waiting/held state. Successful Passed remains evidence
subject to the separate final phase closure, not transport-based advancement. The private Store checks each
complete allowed projection; ordinary Workflow writes cannot masquerade as a link.

Claim requires genuine current native receipt/input/result authority and actual gate
invocation ownership, full current original lifecycle/source/lock admission before any
new gate effect. Effectful gates additionally require their actual independent runtime-
job/evidence owner and physical reservation before first effect; pure evaluators require
actual typed input/result provenance, not a fabricated external job. Factual outcome
publication after admitted gate work may preserve its original claim under later
revocation, changing W/audit only; it cannot renew authority or authorize closure under
stale P/G/T. Exact genuine #9 review disposition and #12 evidence semantics determine
Passed; transport success/receipt, native text, arbitrary Evidence JSON or statusNone
cannot do so. Missing/unknown actual producer/outcome retains the gate/phase claim.
Final closure additionally requires every relevant evidence job actually settled, with
its genuine outcomes; the earlier native receipt does not settle a later independent
job. No actor/input/source refresh and no duplicate external call on observation replay.

The finite256-link budget includes at most99 gate cycles ×2 links (claim and fused
observation/decision)=198, binding1, terminal_decision1, final closure1, gate_hold8
and optional diagnostics47: sum256. Reserve required configured gate and final
terminal/closure headroom BEFORE any effect; optional facts cannot spend it. An
unchanged repeated Waiting result coalesces bounded attention instead of needless
new links, but65 genuine Waiting cycles followed by a66th Passed cycle must fit.
The100th gate cycle refuses before any gate effect with held attention; a99th
admitted Passed still has its reserved final closure. Exhaustion never fabricates
Failed, reruns native input, drops mandatory facts or strands an admitted closure.
Private gate_hold permits only bounded actual typed gate-policy detail/held_reason
updates, at most8, with W+one reserved audit and no Task bump. Such policies cannot
claim native/grant/result authority or overwrite the original input.

S6-09/10/13 require actual bind→gate claim→observed Passed/Failed→closure, genuine
Waiting→next bounded gate and65-Wait→Passed controls, Task unchanged before closure,
revoked observation retained without permission, unknown effect/result held, missing
producer refusal and claim/Waiting Task-bump, forged-Passed, lost-decision, overflow/
ledger-removal mutants. Real19/8/9/12/43/60 integration remains required; no seeded
gate owner proves acceptance.

**Consumed failure under revocation remains held.** No new revocation-tolerant
KnownCurrentFailure policy is introduced here. Its exact consumed input/native outcome
and complete cleanup are necessary but do not waive current original P/G/T/context/
source/locks/active-owner predicates. If an authorized Project Block, Goal Pause or Task
hold changed those pins after consumption, factual failure/receipt may be retained but
phase closure remains recovery-pending14, even after later resume. Success binding is
also strict. This is an explicit availability limit; do not fabricate NCD after input
or clear a hold to make retry work. The narrower genuine pre-input NCD closure under
revocation remains Design31. Expanding consumed-failure closure needs separate reviewed
requirements/native controls. S6-09 distinguishes normal genuinely current failed
closure from revoked consumed failure held, with zero duplicate dispatch. Public
Cancelled/Failed TerminalRecovery keeps only its separately authorized existing scope.

**Complete Workflow storage projection.** Every post-epoch Workflow field that can
carry a source map or large evidence uses typed immutable bounded references, not
nested maps or escaped JSON. The field inventory is exhaustive:

| Stored field | Compact representation and authoritative material |
| --- | --- |
| sources | scope/revision/source-blob digest and exact ContextVersion ref; no source_versions map or payload copy |
| history[].observations[].sources / outcome / error | source/outcome refs, typed status/count/time, bounded512-byte entry as Design26 |
| escalations[].evidence | exact source/outcome refs and at most512-byte factual reason; no serde_json source-map string |
| invalidations[].previous_sources / sources | two exact immutable source refs plus revisions/cause; no embedded maps |
| completed[*].source_versions / dependencies / artifacts | exact source ref plus dependency/artifact manifest refs with scoped digest; no map/path-list duplication |
| retries[].reason / awaiting_explicit_retry / terminal_decision / finalizations / held_reason | bounded512-byte text + exact index/revision/ref, large evidence in charged outcome blob |
| configured_phases / class/risk/generation/active/context and remaining scalar metadata | bounded typed scalars/enums; no opaque Value/source/evidence extension |

All history/change lists share the existing256-entry count ceilings and complete8-MiB
Workflow body bound. Each source/dependency/artifact manifest blob is at most2 MiB;
outcome/evidence text blobs are at most1 MiB. Proven identical immutable bytes/scope/
projection deduplicate physically once; every actual delivery remains separately charged.
Unique source/outcome authorities share the existing64-unique observation/material ceiling;
a new distinct map/evidence at that boundary returns NeedsContext BEFORE further effects.
No unlisted field can embed a large map/opaque value. Decoding old read-only historical
rows is separately bounded and never silently adopted into managed authority.

Actual Engine consumers resolve full references outside SharedStore for capture,
verification and phase decisions, then revalidate exact digests/current authority in the
mutation transaction. Compact records are not summaries replacing mandatory evidence;
missing refs refuse. Actual whole-body reservation derives worst-case bytes for EACH
of these fields and any new source/outcome blob, including escaped bounded text and
all irreversible tail contexts/history/counters/audits; it reserves before effects.
S6-13 uses realistic near2-MiB maps across escalation, invalidation, completed evidence,
retry and tail finalization: compact body/128-MiB charging stays bounded; source digests
still detect actual change. Each map-reembedding mutant and omitted tail-family mutant
must fail the real consumer/quota test. This is proposed schema6 storage/Engine work,
not current component SourceSnapshot or provider API implementation.


### Design34: complete factual Workflow succession and scoped authority exits

**Governing source origin verified against the actual reader.** Current component
`context::scan` creates separate ScopedReader instances for the Task worktree and
registered PRIMARY Project root. The rule/config loop uses the primary-root reader
(context.rs773/781,927–966); phase_rule_bytes and context_pack::rules render those
captured primary bytes. Executor-edited Task copies therefore do not become Engine
governing rules on recapture. Candidate retains this exact origin, scoped FD/root/
Git identity and immutable source pins, with Task changes treated as unverified
output diffs. S6-06/12 proves an Executor rule-copy edit cannot change the next Engine
Reviewer prefix; a mutant switching to the Task reader must fail. No automatic
acceptance of a changed primary governing ref is introduced.

Actual native default AGENTS/rule/config loading is a SEPARATE registered-profile
input channel. Its source/origin and Task-writable exposure must be concretely
inventoried/proved by the actual provider; unknown exposure is unsupported, not
qualified by this Engine reader or a CWD/read-only label. Keep native defaults/hooks.
AllowCurrent action producers and HostCallbackCurrent resolve actual effect targets: a
genuine grant cannot authorize edits to primary governing/config/accepted-definition
refs even if a public action name/path claims an output. Executor Task copies may be
reviewed diffs, never automatically activated governing authority. Actual native
auto-permit/effect coverage remains a required producer/profile gate, not invented
containment.

**Complete outside-lock ledger extraction, bounded transaction currency.** The
256-link/1-MiB complete factual ledger is derived OUTSIDE held SharedStore and SQLite
Immediate write transactions from an immutable scoped read snapshot. Only Store's
private derivation constructs the nonserializable sealed proof; a caller digest/head
is not evidence. It verifies the marker anchor, every exact allowed body projection,
consecutive checked W versions, body hashes, same operation/frame and indexed
append-only prefix through the genuine latest link. It records exact last link ID/
sequence/version/digest, marker identity and current complete canonical W body hash.
Inside Immediate, the consumer compares these exact bounded compact endpoints and
actual latest indexed prefix/count against current W and marker; concurrent W/link
change refuses or triggers genuine outside-lock rederivation. There is no current
row self-match, unverified skip or full256-row decode under the write mutex.

Design35 supersedes the overbroad ALL-reads512-KiB statement: compact authority
PROOF rows alone (hard prerequisites, ledger endpoints, revocation/index guards)
obey128 ×4096 UTF-8 bytes/512 KiB. Enumerated mandatory owner rows, full locks,
physical-conflict scans, body writes and SQL JSON validation have separately bounded
classes; they are not exempt or secretly counted as4-KiB proof rows. A full128-proof
set plus another proof endpoint still refuses before effects. The usable prerequisite maximum depends on the other
required rows, not a separate128 exemption. Full sorted lock body and canonical W
body hashing are outside-lock captured; indexed scalar identity/version/hash guards
revalidate the exact observation inside. Every W body change requires a checked
version successor; same-version body mutation is forbidden, including raw SQL. The
outside observed complete hash, exact W version and private indexed latest successor
hash therefore form the inside check; do not recompute an8-MiB W hash inside the
transaction or treat a caller-provided stored hash as authoritative. Missing/conflicting
indexed proof refuses. The original resulting marker frame still
includes the full lock set. Actual native admission instrumentation checks all reads
and bounded hashes, including source proof derivation, not merely SQL row count.
S6-16 proves fitting combined limit and129th/512-KiB+1 refusal, concurrent genuine
link race, raw drift, unchanged grants and a second Project's genuine publication;
inside-lock full derivation and caller-head mutants must fail.

Ledger indexing uses byte-guarded expression partial indexes on the CLOSED reserved
event-kind set. Nested CASE first checks length(CAST(data AS BLOB))≤4096, then
json_valid, then bounded extraction; no INDEX JSON function evaluates for an oversized or
nonreserved generic audit. Design35 separately guards the table CHECK surfaces. Do not rely on AND/optimizer order or an AFTER trigger.
The reserved insert trigger independently performs its byte guard before any JSON
call. Indexed unique operation/sequence/predecessor/successor keys and append-only
reserved-prefix update/delete guards cover raw SQL and old writers; generic audit
cannot mint or alter links. Fresh/migrated databases use the SAME expression-index
DDL (no generated-column ALTER divergence), pinned in the migration golden. Actual
SQLite function/call instrumentation verifies oversized generic/reserved audits have
zero premature JSON parsing. These are proposed source controls, not measured now.

Each record-only port uses a DEDICATED private Workflow writer retaining complete
domain/CAS/projection guards and emitting exactly ONE reserved ledger audit. It
does not call generic put_record_tx, whose actual workflow.saved audit would double
the write set. No public skip-audit flag. Goldens assert W body/version/updated_at
plus that one audit only; a shared-helper extra-audit mutant fails. The final phase
closure remains its separately declared Task/operation/receipt transaction.

**Exhaustive open-phase writers and terminal decisions.** The allowed factual chain
contains binding, diagnostic, gate_claim, fused gate_observed/decision, bounded
gate_hold, `rrx.private.workflow.terminal_decision` and the final
`rrx.private.workflow.phase_closed` link. The final closure uses its separately
reviewed Task/operation/receipt write set; these are19/8-owned ports, not extra43
binder/diagnostic writes. Ordinary
Workflow mutations while phase_open refuse. Current cancel/fail TerminalDecision
writes Task+W and must migrate: actual23 lifecycle authorization first validates
the original private operation/frame, complete genuine predecessor and exact current
write CAS, then commits ONLY its terminal_decision W projection, checked authorized
Task lifecycle delta and one reserved link. This is NOT a43 binder/diagnostic expansion.
The Task change is independently authorized; it never refreshes original marker T
or supplies native input/ALLOW authority. Later factual TerminalRecovery verifies
this exact authorized successor and complete own settlement, preserving Cancelled/
Failed and original input without binding an absent ID or granting success. Raw W
drift followed by cancellation cannot launder drift: chain derivation fails and
claim remains held. Independently authorized genuine Task/Goal lifecycle revocation
still works under actual23 policy even if an unproven Workflow bookkeeping successor
cannot commit; it supplies no chain link and leaves managed closure held14. Bound and
unbound cancel→actual cleanup→TerminalRecovery, forged
decision, missing link, Task-delta forgery and consumer-exemption mutants are S6-09/10.
Known consumed failure under unrelated lifecycle/source revocation remains held14
as Design33 states; this named terminal-decision route does not broaden it.

**Stop, invalidation, escalation and retry authority inventory.** Actual component
step polls an active native attempt before source recapture, and Engine::escalate
requires active None. There is no demonstrated automatic live-source-drift interrupt
producer. Candidate does not invent one. A genuine owned explicit stop request may
record bounded request metadata without rewriting Task/W/native authority; interrupt
ACK alone proves neither native terminal outcome nor cleanup. Before input, genuine
NCD/full cleanup may non-success close under the existing narrow revocation rules.
After input, actual current known Interrupted/Failed outcome AND complete required
cleanup AND original current pins are needed for non-success closure and retry hold.
Missing terminal, consumed uncertainty or revoked original pins stay held14. No
invalidation, policy-refresh or escalation API bypasses receipt/chain predicates or
publishes a replacement generation while its old managed owner is unsettled.

After genuine closure, retry/escalation/new generation is a NEW admission. RetryEvent
requires actual nonserializable23 trusted Human authority OR a previously trusted
Human-approved frozen policy with finite per-lineage automatic retry allowance, exact
cause/current evidence and actual budget charging. Bare Engine::retry(reason='auto'),
public JSON, driver polling or a recorded enum cannot mint it. Existing9 review
lineage/author/allowance constraints continue; this adds no per-fix Human requirement
when genuine finite policy already authorizes autonomous remediation. Duplicate/stale
authority is no-write. S6-09/16 exercises real driver stop/pre-input/current interrupted
closure→hold→genuine authorized fresh retry, unknown held, and missing receipt/retry-
principal mutants; no fake stop or automatic current-row authority recapture.

**Project authority drain and Goal pointer writer composition.** A separately typed
trusted Project pending-authority request is bounded, scoped and durable, as the Goal
barrier is. It does not change current Project definitions/source refs/version or grant
permission. Fresh markers/claims in that Project wait while pending, allowing already
admitted operations to settle under their ORIGINAL frame. Apply only after EVERY
Project phase_closed and complete relevant ownership/physical exclusion, with exact
expected Project version and genuine trusted configuration authority. Unknown/Lost
owners retain the barrier with attention until14; no timeout release or Block shortcut.
Requests cannot change accepted Goal definitions. Competing/duplicate requests use
explicit current request CAS and cannot starve a drain by opening new phases. S6-16
requires continuous parallel scheduling plus a pending rule/config/name change: no new
markers, existing genuine turns complete, apply once after close; second disjoint
Project progresses. Actual23/24/27 producer composition remains pending.

Goal context pointer publication is an EXPLICIT composed typed Goal writer, distinct
from generic put_goal and native launch. Its only Goal-field changes are context_version
and updated_at, with semantic Goal.version unchanged and consecutive exact ContextVersion
head/pointer CAS. It preserves accepted definition/lifecycle/DAG and latest pointer in
all other typed writes; stale generic/typed snapshots cannot roll it back. S6-12/16
requires actual19+23 publication, simultaneous head winner, stale rollback refusal,
raw write-set golden and admitted sibling Task unchanged. It never rewrites a Workflow
launch context or makes a Goal summary launchable. Root23 writer inventory must include
this real private port before composed source acceptance.


### Design35: honest transaction classes and exhaustive writer authority

**Two finite transaction classes.** The compact proof gate is at most128 rows,
4096 complete encoded UTF-8 bytes EACH/512 KiB aggregate, counting prerequisite
certificates, ledger endpoints and compact revocation/index guards. Full outside-
lock evidence/ledger derivation remains required; proof oversize refuses before
effects. This is NOT a bound on all owner-row decoding, SQL trigger/constraint
work, read/write bytes or SQLite/SharedStore holding time. Actual owner-row/CAS
validation and existing body-rewriting ports require a separately bounded class.
No latency, memory or OS scheduling guarantee follows from either finite cap.

The mandatory owner manifest is explicit; only the current exact scoped rows needed
by the named port are included, never all historical Sessions/contexts:

| Mandatory owner surface | Complete encoded cap / maximum read set |
| --- | --- |
| Current Project / Goal / Task | One each;8 MiB per body (Goal DAG retains actual23's4096-node/16384-edge limits) |
| Current Workflow Record | One;8 MiB body; exact current version and complete allowed projection |
| Exact owned launch/latest ContextVersion needed by port | At most two;8 MiB each; never scan historical packs or all Tasks' contexts |
| Exact own Session | One;4 MiB body,3 MiB recovery/depth32/node32768 limits retained |
| Exact private operation | One when applicable;4 MiB complete row including source/lock/actor provenance; referenced native-result bytes separate≤1 MiB |
| Exact frozen receipt | One when applicable;64 KiB COMPLETE row including every identity/attribution/continue_refusal field; no4-MiB receipt allowance |
| Private preparation / admission / phase owner | One each when applicable;2 MiB EACH complete authority tuple (existing source-map/string limits retained) |
| Exact consumed dispatch DTO | One;8192 complete bytes, never full duplicate payload |
| Exact counters/quota/epoch/pending-request rows | Closed typed scalar metadata≤4096 bytes each; actual needed rows count in compact128-proof budget |
| Full scoped WorktreeLock set | At most256 rows,16384 complete bytes each/4 MiB total; immutable marker ID/version set remains≤64 KiB |
| Host effect admissions / outcomes at settlement | At most256 admitted8-KiB rows and256 known2-KiB outcomes/2.5 MiB combined; full owned worker settlement occurs outside-lock before publication |
| Source/outcome/blob bytes | No arbitrary full blob extraction inside admission; immutable compact identities/hashes checked; new bounded writes charged to actual128-MiB quota |

These are simultaneous ceilings, not permission to exceed2-MiB source-map/authority
tuples or1-MiB actual model frame. Marker, native admission, binder, observation and
closure instrument which manifest rows they actually read/write and repeated reads/
JSON calls; an unlisted whole-body lookup or scope scan refuses. Project/Goal/Task
body caps also apply before new source validation and SQL writes, never truncating
accepted facts. Genuine oversized legacy rows refuse the deliberate candidate
upgrade with unchanged bytes/version; no history rewriting or implicit user upgrade.

Full Workflow next-body construction, complete-field/allowed-projection validation
and canonical hash computation occur outside held SharedStore from exact immutable
read observations. Inside Immediate the actual Store rechecks exact versions, private
ledger/owner identities and writes that exact planned body. SQLite may still read
OLD/NEW bodies and execute bounded native-owned SQL projection/JSON/CHECK validation
under its lock, up to the declared row caps; these operations are explicitly measured
and included in the owner/body class. No claim that every hash/JSON call or body-write
cost is outside SQLite, and no shared helper may decode unlimited history. Actual
43 binder retains W+ONE reserved audit only; it adds no operation/table update. A
raw body change without checked version is forbidden; a changed current version
forces genuine outside-lock rederivation rather than planned-body overwrite.

Own-session duplicate/exclusion detection uses an actual typed/generated indexed
managed-owner identity (exact scope,provider,validated native identity and distinct
Session ID), with a≤2 compact matching-row existence query sufficient to prove a
conflict. It does NOT scan256 complete4-MiB Session bodies or infer no conflict from
a truncated list. Unallocated factual historical native_ref strings are not native
ownership and cannot mint this index's private identity. Same-Session continuation
preserves its UUID; private provider registry/actual binding proof remains mandatory.
Native-identity index binding is nullable until actual validated native setup,
then monotonic once per native identity chain; private actor/Session matching and
provider-owned registry proof are checked in that same preparation transaction,
never by43 binder or generic Session JSON. Index insertion/binding follows genuine allocator/native-binding authority and all
writer/epoch/fresh-versus-migrated goldens; this is not a global native_ref uniqueness
claim or an exemption from the actual provider's owned registry. Loss/restart rows
do not reconstruct a live owner.

Full lock currency uses ALL scoped rows (up to256), each ID/version/body bounded
and matched against the original sorted marker set; no maintained digest is invented.
Physical nested-root/common-Git conflicts use a COMPLETE finite lookup over currently
retained native-operation/maintenance and future58 owner scopes, including Lost,
unknown, session-less and Removed-Project provenance when retained. Canonical primary/
worktree/common-Git anchored paths are explicit immutable typed scope columns (each
path≤4096 bytes; entire selected scope row≤16384). Indexed open/held selection reads
at most4096 such compact physical rows/64 MiB; LIMIT4097 detects overflow and refuses
BEFORE new grant/effect, never means the first4096 prove disjointness. Original rows
remain held and actual cleanup publication may still proceed. Lifecycle/Session
labels do not hide physical ownership. All acquire/close transitions update the
same protected scope/index in their actual owner transaction; no out-of-lock read-
then-act reservation. Existing same-directory equality indices do not replace
ancestor/common-Git containment checks. Report this explicit scale/holding-cost
limit and bounded complete scan; actual58/60 source composition remains pending.

S6-16 instruments BOTH classes, their actual maxima/refusals, DAG/body/256-full-lock
cases, physical4096/4097 including Removed/Lost/unknown nested scope, indexed duplicate
positives/negative unverified history, and actual other-Project publication under
the maximal fitting transaction. Report observed contention/latency; no inferred
512-KiB total-mutex guarantee or change to native input token budgets. Full ledger
extraction inside-lock, unlisted body lookup, truncated conflict/lock/Session scan
and scalar-only measurement mutants must fail actual consumer controls.

**Byte-first ALL JSON surfaces and fixture provenance.** Every json_valid CHECK
surface has byte-bounding BEFORE INSERT/UPDATE guards BEFORE its constraint parses:
Project/Goal/Task/Workflow/general Record/ContextVersion bodies≤8 MiB, Session Record
≤4 MiB, generic audit.data≤4 MiB and Usage.body≤1 MiB. Reserved factual link data≤4096
bytes remains stricter. Each JSON-reading trigger/index independently nests its
OWN byte guard before its own JSON call; do not rely on ordering between triggers.
OLD JSON body also must be bounded before OLD projection parsing. Every rejected
write rolls back with prior data/ownership intact. Fresh and migrated table CHECK
DDL remains canonical; additive guarded triggers/index definitions are identical.
Generic huge audit/Usage, raw SQL and all constraint/index paths have actual call-
instrumentation controls; no claim that only partial-index parsing suffices.

Candidate RRXC migration accepts only an opaque fixture-state capability created by
the actual fixture constructor: owned freshly created temporary root FD/device/inode,
exact fixture DB path/identity and retained closed synthetic creation provenance.
Historical-binary fixture writes stay inside that owned root and are verified before
opening/migrating; an arbitrary RRX1 path or caller flag cannot construct this cap.
No candidate public CLI migration/path entry exists. A real user DB outside that
actual fixture birth authority refuses before OPEN/DDL/retag; copied/forged markers
are not sufficient authority. Tests use the genuine fixture constructor and actual
external user-like DB refusal with unchanged bytes/RRX1/version. No production
legacy barrier/drain/recovery is proved or changed by fixture retagging.

**Idle instruction authority and complete writer union.** For a Project/Task with
post-epoch managed ownership/history (closed history INCLUDED), generic put_project/
put_task reject EVERY change to instruction:project.v2/instruction:task.v2 definition
fields, even while all phases are closed. Goal accepted definitions remain immutable
under23. Only actual typed trusted Project pending apply or Task definition-policy
port may change the allowed Project/Task instruction fields; trusted Human/controller
composition, current source/definition/lifecycle/CAS and retained lineage rules
remain mandatory. Task proposals/native JSON are not authority. Pure bookkeeping
name/max_tasks/issue is separately classified: it may use the actual trusted idle
bookkeeping port only after phase_closed, preserving definition digests; no generic
idle bypass or live raw-version bump. The source semantic projection must implement
that distinction, not merely assert that a hashed field is irrelevant. Initial
manual unmanaged creation stays its existing application boundary; first managed
marker captures/promotes only through actual accepted configuration/task policy.

The following rows extend Design31's exhaustive union; the canonical native table
and that table now expressly INCLUDE these rows. Each requires actual consumer,
rollback/write-set control and causal mutant, not an optional available-when port.

| Authority writer | Exact allowed set / authority | S6 keys |
| --- | --- | --- |
| Managed gate_claim | Genuine current receipt/typed gate ownership, original frame/current CAS; Evaluating+exact claim index W +one reserved link, no Task bump | 09/10/13/16 |
| Fused gate_observed/typed decision | Actual admitted gate outcome/current claim; compact observation/repeat count+last_time+bounded detail and Evaluating→Waiting/heldFailed W +one reserved link; no legacy workflow.gate_observed or workflow.saved extra audit | 09/10/13/16 |
| Managed gate_hold | Actual typed gate-policy owner; only bounded detail/held_reason W +one reserved link, at most8; no new result/grant/Task rewrite | 09/10/13 |
| Managed terminal_decision | Actual23 lifecycle authority + original private op/complete predecessor; exact terminal decision W +authorized checked Task delta +one link | 09/10/16 |
| Final phase_closed link | Genuine receipt AND actual current accepted gate/result proof; separate exact phase closure/Task/operation/receipt +one link; success never from native transport alone | 08/09/10/13/16 |
| Project pending request/apply/retire | Genuine trusted configuration; bounded pending metadata/expected version; no current P/definition change while queueing; new claims wait, apply only actual full Project drain under CAS | 06/10/16 |
| Managed idle Task definition/bookkeeping | Actual23/task policy trusted authority; allowed exact field projection, current idle CAS/lineage; generic/native changes refused even between phases | 05/10/16 |
| Typed Goal context-pointer publication | Actual19+23 private head/source CAS; only context_version/updated_at, semantic version unchanged, immutable accepted Goal definition preserved | 12/16 |
| Engine stop-request audit | Actual owned supervisor request; ONLY `rrx.private.operation.stop_requested` audit≤4096 bytes with exact retained op/frame/cause/current own CAS; no operation-column/Task/W mutation or outcome | 08/09/13/16 |

Internal fail is KnownCurrentFailureClosure with genuine current failure/cleanup;
it is not Human cancellation disguised as TerminalDecision. Stop requests and
source-change proposals do not autonomously release/refresh a marked operation.
Ordinary independent lifecycle revocation may hold native grants even when managed
bookkeeping cannot be proved; it never mints a successor link. Every actual emitted
private kind consumer, including replaced legacy gate/audit consumers, must migrate
to the compiled current constants. No duplicate extra legacy audit under ONE-link
ports. INSERT OR REPLACE must be rejected BEFORE INSERT for existing marked Workflow
Record IDs and private operation/sequence/predecessor link keys, even with recursive
triggers OFF; UPDATE/DELETE fences alone are insufficient. Raw REPLACE controls and
removed-insert-guard mutants reach these writers.

**Gate-cycle exit is finite and safe.**99 actual claim/observation executions is
the cap, independent of64 unique outcome dedup/u64 repeat counts. Optional pure
read-only status/attention coalescing cannot execute an effect or add an uncounted
claim. At8 gate_hold exhaustion refuse new gates/success with attention; the
configured next gate claim/observed pair must reserve any required hold clearance
within its exact allowed projection and body/quota headroom.99th admitted Passed
may still close with reserved proof/bytes.100th Waiting cycle refuses before gate
effect and is an explicit cancel-only exit: actual trusted cancellation then genuine
full cleanup/TerminalRecovery under its predicates, no fabricated Failed/native
retry/round refund. Unknown or revoked closure remains held14. Final genuine closure
atomically releases ONLY unused ledger reservation; spent retained bytes/facts stay
charged. A held phase retains its reservation. Actual65 identical-Wait→Passed,99/100,
hold8/9, cancellation and consumed-unknown controls/mutants are S6-09/13/16.

### Design36: retained preparation and finite D35 corrections

Status: proposed correction of verified D35 H/M and current43 obligations, not a
new requirements policy, compiled API or available native producer. Merged main is
schema3; this unmerged component is5; proposed6 remains unavailable. This section
supersedes the by-value operation in Design16 and the live checkpoint-head append
example. Other original frame/current grants/strict consumed-failure holds remain.

**Sole owner stays in the supervisor.** The actual runtime transfers the marker's
non-Clone OwnedNativeOperation into a retained supervisor before the first await.
That supervisor's private custody entry owns its preparation state and resource/job
inventory outside the adapter/start/caller future. ManagedPhaseLaunch carries only
immutable request/continuation/original marker descriptor, never the sole operation.
The adapter receives the borrowed actor/lifetime-bound ManagedSetup surface shown
in Design16. It cannot reconstruct/clone custody or move it into its returned future.
Default Unsupported and every pure pre-effect Err drop only the borrow. The real
supervisor still has Pristine and can use the genuine no-effect refusal port; an
error value, missing Session or durable false flag is not that proof.

Setup access registers an actual job/resource with that SAME retained supervisor
BEFORE its effect, commits effects_started before first Session/process/native IO,
and checks the exact immutable original frame/actor/lifetime at every private setup
use. After the boundary, failures are tracked NotCreated/Partial/Unknown outcomes
only from actual owned producers, never inferred from std spawn Err. Cancellation
closes admission of NEW setup effects and revokes borrowed access; it retains ALL
already-started jobs/resources through their actual cleanup/observation. Job joins,
reader handles, callbacks and cleanup workers remain in the genuine inventory even
when the adapter future is dropped, panics, returns Err or its waiting caller ends.
A closed borrow is neither complete cleanup nor receipt. The actual actor cannot
admit another job while settlement is being derived. Runtime teardown/restart or a
lost observer without genuine custody proof retains durable ownership/attention14;
no row, runtime UUID or late selected-group observation reconstructs it.

Only the retained supervisor's genuine complete outcome/inventory observation can
construct OwnedNativeSettlement. Pristine before effects uses the existing genuine
no-Session NoCurrentDispatch rule. After effects_started, every actual job/callback/
resource and relevant native outcome must be known and fully settled under its
supported profile. Any unknown/escaped/unjoined resource or consumed-zero-wire state
keeps the existing held14 outcome. NoCurrentDispatch is not available after input
admission/consumption. KnownCurrentTerminal does not become success/Passed merely
from native transport; original claim/frame/gate/23 predicates still apply.

The next source-ready LIFETIME component is this retained custody/borrow/cancel/job
mechanism, with no public authority constructor, schema migration, receipt or
advertised ready backend. Its actual private setup path owns real closed-inventory
synthetic jobs in isolated fixtures to prove errors/caller Drop cannot erase custody,
registration precedes effects and cancel retains jobs until observed settlement.
Such fixtures do not mint production native cohort/operation certificates. Real
19 marker/allocation/native/receipt/23/43 integration stays a separate source gate;
backend absence refuses before effect rather than routing through a legacy adapter.
The component may not count an unused helper or SQL-seeded operation as this proof.

Coordination evidence at this draft: native6 c8ce5a86ebc2278ed6ca9e4ecdf7d127e5628f18
has private production availability EMPTY, no callable managed backend/cohort; its
F4 controlled setup custody is selected-group mechanics only.60
Requirements4 at a8e8a8b21b9f11c68f12d1ade51ad5c8c14fe81a is approved; its proposed
Reader Design1 at8f95adbb8198c1c7d2a35f11fb172bca541b5a49 is UNAPPROVED and has no
source port. Private GitPool/OpReservation/OpTicket/OpAssets proposals retain actual
std Child/native worker/readers and a per-operation runtime,4 jobs from global64,
with nonblocking caller-Drop cancellation. Existing bounded_git_raw byte-vector +
Arc<AtomicBool> compatibility has no exported setup-cap or durable owner callback.
Selected-child/readers custody and the retained Unknown latch do NOT grant19
preparation/settlement authority; neither that API's returned bytes nor its bool
can manufacture a managed setup/settlement proof. Existing Git
and native probes need their ACTUAL60/effect/profile reservation before execution;
no borrowing a phase marker or pretending a local Child holder covers default hooks,
escaped native PTY descendants or unknown tools. Later concrete interfaces compose
ONE operation supervisor with those actual producers, not two competing owners.

**Receipt replay is exact and non-writing.** The supervisor fixes one receipt UUID
and complete canonical bounded proof BEFORE its first publication, retaining it
across every Store outcome. publish_native_settlement BORROWS that retained proof,
not consumes its only copy on an error. Its Immediate transaction may return the
existing receipt only for the SAME UUID/operation AND byte-identical full immutable
pins/class/body/version/attribution. That exact replay is no-write/no-audit; a
same-operation differing UUID or body is contradiction, never success/replacement.
On uncertain commit, reconcile that exact identity/body read-only before any new
publication or uncertainty classification. A differing durable result stays held.
Known rollback/transient Busy/storage errors retain this same proof and use bounded
fair100ms–5s retry of publication, never dispatch or a new receipt identity. Storage
failure alone is NOT native uncertainty and cannot mint Lost. Genuine native outcome/
cleanup uncertainty, declared timeout or already published Lost stays held14; this
retry relaxes no profile deadline, native custody or current closure predicate.

The complete receipt bound is64KiB including ALL compact source/actor/frame references,
Session/body/version/restore pins, native attribution and continue_refusal. Full source/lock bodies stay in their exact
referenced immutable authority; no hidden duplicated body bypass. The operation
bound is4MiB; no receipt inherits it. Tail/quota reservation and byte-first
SQL/encoder goldens use the64KiB actual complete bound. At64KiB+1 refuse BEFORE the
producer's irreversible effect through its existing reserved-headroom contract;
never drop mandatory terminal facts or silently trim attribution after cleanup.

**Condensation stages until phase closure.** Actual same-Task live condensation may
capture attributable bounded events/compact proposal into its operation's existing
charged evidence/headroom without advancing checkpoint_heads, rewriting admitted
ContextVersion or changing current CPP/governing source. Staging is not a checkpoint
reference, input grant or publicly forged history authority. Mandatory events remain
complete; insufficient existing reservation holds before the governed effect rather
than silently losing facts. All source capture still uses its actual60 owner.

Only after the SAME operation genuinely settles AND its exact phase_closed commits
may the actual typed checkpoint producer revalidate current source/semantic state,
Session provenance and latest own head, then append the staged ordered mandatory
history with atomic checkpoint/head/audit CAS. If another closed attempt advances
that head, stale publication refuses and rederives from exact current facts; no
snapshot rebinding or automatic receipt reuse. Cross-Task promotion remains an
explicit immutable closed historical snapshot; no promotion/head publication while
a governed open phase pins it. New preparation consumes the latest published head.
Actual own-stage→ALLOW and Grok host-read controls must preserve immutable grant
predicates; rule/HEAD/revocation drift still refuses. This design does not relax
AllowCurrent's head check to accept arbitrary same-UID writes or new input.

**Factual UUID negative lookup is separate.** Current43R9 rejects ANY distinct
Session in the exact scope with the same factual provider/native UUID, including
unallocated history or Lost. Add a bounded generated/typed body-derived lexical
negative identity lookup, with equality to the actual bounded body, covering ALL
such factual strings. It grants no private native owner and imposes no global UUID
uniqueness. At most2 compact indexed matches suffice to detect another Session;
unknown/malformed/nonindexable relevant legacy scope refuses conservatively, never
means empty index proves no conflict. D35's genuine managed-owner index remains
SEPARATE positive authority, requiring actual provider registry/allocator/native
binding. Raw body/index disagreement or replace cannot hide a factual duplicate.
No existing index/helper or source acceptance is claimed here.

**Maintenance and pending authority keep safe availability.** Actual58/60 conflict
producers declare physical read/write/effect scopes including per-worktree content,
its common-Git admin entry and shared refs/objects/index. Separation is allowed ONLY
when actual native/default-effect inventory proves it; unknown effects retain full
root/common-Git scope. Worktree provision/disposal may coexist with another Task's
operation only on that genuine nonconflicting proof, never a Task name or read-only
label. Required actual four-Task parallel/provision/disposal controls remain open.

Genuinely conflicting maintenance uses the existing bounded typed pending-request/
drain pattern with reciprocal same-transaction marker checks and fair actual27/60
selection: activation stops new overlapping claims, existing owners settle, full
scope drain precedes effect reservation/apply. It changes no governing definition,
current raw owner version or release authority merely by queueing. A proposal behind
actual held14 ownership remains recorded NONactive with bounded attention, so it
cannot install a new active whole-Project/Goal freeze while that drain is impossible.
The same classification applies to accepted Goal/Project change proposals: record
NONactive with no definition/source/P/G/T version change while held ownership prevents
activation; current genuinely revoked grants and all physical holds remain. When
activation is possible, actual trusted policy activates a drain barrier, prevents
new applicable claims, waits for EVERY applicable open phase to close and applies
under full expected source/definition/lifecycle CAS. No held operation is excluded
from drain, re-adopted or silently refreshed; accepted Goal definitions stay immutable.
Unknown physical scope still refuses, not disjoint admission. This is a safe recorded
proposal/attention exit, not completed27/58/60 fairness or guaranteed14 recovery.

**Causal controls and remaining limits.** S6-03/08/16 use default Unsupported and
pure validation Err AFTER an actual fixture marker to show retained Pristine then
one genuine no-effect receipt/non-success closure; omitted retained-owner/borrow
controls must strand or lose the actual owner at that consumer. Source of marker
and proof is the actual fixture-private producer, never inserted SQL evidence.
After-effect setup failure and caller Drop retain each actual closed-inventory job;
premature receipt/permit release fails until genuine observed cleanup. Actual native
positive needs its production cohort producer, not the synthetic selected group.

S6-07/08 controls inject rollback and uncertain COMMIT AFTER receipt durability:
exact UUID/body replay is one receipt/audit, contradictory replay refuses, no spurious
Lost; omitted reconciliation/replay reaches that consumer. S6-12 controls hold a live
head/pair unchanged during own staging, reject public live append, preserve ALLOW/
host read and immutable-source negative controls, then actual close→checkpoint
append enables the next new preparation. S6-10 includes historical/Lost factual UUID
duplicate negatives separate from genuine managed-owner positives and malformed
scope/Replace/equality controls. Bounds64KiB receipt/4MiB operation are independent.

S6-14/16 require actual conflicting healthy maintenance drain and disjoint physical
controls, held14 NONactive proposal plus unaffected existing Task progress, no
accepted-definition/current-authority rewrite and later genuine full drain/apply.
Missing real producer keeps these positives OPEN. Consumed KnownCurrentFailure under
revoked predicates remains safe held14, not a new tolerance. A closed pristine
Session-less refusal intentionally makes the next selector Fresh: it cannot skip
the latest closed operation to adopt an older receipt. Record that availability cost;
no hidden older-UUID resume or lineage/budget laundering is introduced.
