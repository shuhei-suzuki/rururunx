# Issue 19 requirements: durable context packs and checkpoints

Implement typed, Project/Goal/Task-scoped durable working context on existing
append-only ContextVersion and Checkpoint records. Rust remains provider independent;
no supervisor model or transcript summarization model is introduced.

- Task packs preserve purpose, criteria, workflow/risk/phase, constraints, owned
  repository/worktree/revision, scoped authoritative artifact references, selected
  file/symbol metadata, decisions, completed work, failures, verification/findings,
  blockers and next action. Source bodies remain authoritative outside summaries.
- Goal packs preserve objective/criteria/constraints/source references, DAG and
  all owned Task status/context references (including Tasks outside the DAG), cross-Task decisions, blockers/next runnable work
  and explicitly supplied aggregate metrics. They do not copy Task histories.
- Relevant authority/source/state changes create consecutive versions. Pointer,
  ContextVersion and audit publication is atomic and version guarded. Equal inputs
  reuse a version. State CAS versions are distinct from semantic authority hashes;
  publishing a context pointer must not immediately stale its own pack.
- Freshness includes actual owned worktree/HEAD, dirty/admitted ignored bytes,
  primary rules/config and artifact digests, plus semantic Task/Goal state. Stale,
  foreign, malformed or dangling references fail before native preparation.
- Checkpoints accept typed, ordered execution/consultation events. Condensation
  retains all semantic goals/decisions/completed work/failures/findings/next actions/
  constraints/verification/critical references. Recent transient history remains
  verbatim under a configurable byte estimate window; omitted old transient data
  has an auditable count/range/digest. Excess mandatory state fails explicitly.
- Checkpoint creation appends evidence without overwriting an active launch's
  ContextVersion. Consultation promotion is explicit, scoped and compact; it copies
  preserved facts with historical provenance, never the full transcript or target
  source authority. Cross-Task promotion is an explicit immutable snapshot; later
  source checkpoints do not silently replace it. Own checkpoint history is automatically included even when its reference is
  omitted. Same-Task preparation/publication requires the latest checkpoint head,
  including the absence of a chain. Own history and consulted snapshots coexist. No cross-Project or cross-Goal implicit
  promotion is allowed.
- Mandatory Project rules and Goal/Task constraints survive packing and promotion.
  The source Task criteria are preserved in checkpoints as well.
  Standalone byte/estimated-token budgets cap the fully rendered payload. Workflow
  discretionary budgets cap optional repository slices; mandatory Task/Goal/
  checkpoint metadata and Engine-owned rules are separate overhead, counted in
  total UTF-8 estimates. A 1 MiB absolute rendered phase-input cap still rejects
  oversized mandatory state. Provider metrics stay nullable; no fact is dropped
  to fit a budget.
- Native I/O stays outside SharedStore. Publication rechecks current versions and
  live/Lost Session/lock/workflow ownership inside the transaction. Workflow-owned
  publication uses the workflow phase port; standalone publication cannot rewrite
  its phase/attempt context. The ordered format 3→4 migration
  makes older writers refuse the authoritative checkpoint write contract. SQL
  checkpoint-head indexing is maintained atomically with append. Generic Context
  writes and pointer changes cannot bypass typed publication or restore an older
  pack. Record kinds remain unchanged.

Goal-only packs use read-only exact primary-root identity/source validation and
atomic Project/Goal/Task-membership-and-summary CAS, including before Tasks and after worktree
cleanup. Finalized Task refs remain historical and non-launchable.

Core checkpoints use a registered Goal and bound Task, including consultation
Sessions. Goal-before-Task consultation creation and CLI/TUI/session transport are
separate integrations. Native checkpoint capability is not fabricated. The real
workflow PhaseContext port is integrated against merged Issue 8, before acceptance.

Goal summaries report exact immutable Task refs and whether source validation is
required; concurrent dirty execution does not disable the Goal summary. Task
launch preparation separately validates current sources. Core limits are 128 Goal
Tasks/4096 DAG edges and 4096 retained semantic events/1 MiB per checkpoint. There
is no destructive checkpoint reset or semantic deduplication; overflow is explicit.

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

## First admission and compatibility requirements

Protected Task input stays pending until an exact consumed dispatch or a private,
current-frame/head-validated Running publication. WaitingApproval/WaitingHuman/Lost
and native_dispatch_unobserved do not grant admission. Pending reentry revalidates
live authority. Already admitted input retains its historical pins. New consumed
intent always revalidates, and admitted nonterminal input cannot visit Starting to
roll back to an older terminal frame. Exact prewire restoration requires no
admission, consumption or uncertainty. Higher-input continuation starts only from
an owned terminal Session and installs fresh authority before native dispatch.

Admission facts are privately indexed, scoped to Task/Session/input/actor, bounded
to one row per Session and atomic with Session plus audit. JSON or generic APIs
cannot manufacture them. Unprotected legacy and Project/Goal-only Sessions retain
their existing contract. Standalone Executor frames cannot be adopted by another
agent, role or worktree. Workflow frames preserve their active attempt authority.

Standalone prepared payloads render stable scoped instruction and Task policy
hashes and publish exact private byte/hash authority. Sibling progress, blockers,
pointer-only and sibling bookkeeping cannot strand pending semantic input;
constraints, refs, identity, own standalone directives/policy and lifecycle changes remain fenced. Physical
HEAD/source hashes remain authoritative. Explicit idle consecutive republish must
allow safe old-frame migration and higher-version terminal continuation; default
publication remains idempotent and active launches immutable.

Schema6 migration is ordered, atomic and makes no historical admission claims.
Unsupported old5 opens AND old5 connections held open before migration must reject
all subsequent application writes, including private metadata/audit, without
changing state. Existing domain/immutability triggers remain effective. Native
compiled old-writer evidence must cover a live connection, not only fresh opens.

Consultant checkpoint history is supported; live role-specific consultation needs
its own exact prepared-frame port. ApprovalReviewer uses a separate operation-free
decision Task through the Approval Broker, never an Executor-writable Task's native
rules/config/hooks. Arbitrary role changes cannot bypass frame authority.

Migration must refuse before mutation while any live/Lost Session, active lock or
active/nonterminal unfinished Workflow ownership remains. Terminal non-owning
cancelled Workflow history remains migratable. Complete or explicitly cancel/drain
with the compatible old runtime first. New semantic projection definitions
are versioned and exhaustively classified. Every typed actor, including Reviewer,
binds the exact Task worktree and single-actor phase allocation atomically; two
Sessions cannot consume the same attempt before Engine Session binding.

Fresh input admission must reject a Task put on hold after Starting, even when its
frame hashes are unchanged. Protection cannot change underneath a live legacy
Session. Fabricated terminal history cannot unlock forced consecutive publication;
private validated preparation/admission must bind the exact terminal input.
Provider lifecycle/lock/version fences remain independent of semantic input checks.


Initial native actor binding is monotonic per input: requested None may bind one
effective model/effort value, explicit Some cannot silently change, and a known
native_ref remains immutable across the Session UUID. Exact admitted metadata
pins and private same-attempt binding updates remain atomic. Higher-input fresh
continuation resets requested model/effort before Starting and keeps exact prior
terminal rollback proof. Claude5, Codex6 and Grok7 require reviewed complete prepared-frame preflight,
input SHA/source pins before Starting, atomic consumed admission before wire,
native binding compatibility and causal typed-frame caller fixtures before
integrated acceptance.

Private phase allocation covers initial INSERT and fresh terminal-to-Starting
UPDATE; Engine binding/closure must use that exact allocated owner. Ordered
migration preflight covers every older supported version, and an old Lost owner
without verified recovery remains an explicit upgrade limitation. Existing
checkpoint-v1 and terminal restore hash encodings remain byte-compatible. Backup
rollback is permitted only before any post-upgrade application/external effect.


A typed irreversible claim must carry an explicit own checkpoint head; absence
fails closed. Single-actor ownership survives the gap before Engine Session
binding: an allocated live Reviewer prevents claim closure. The native dispatch
intent denotes actual input delivery; an own-checkpoint append requires fresh
higher-input continuation before another delivery. Historical observation alone
sends no new model input. Variant-cap exhaustion supports explicit idle consecutive
republish without discarding history. Idle unfinished Workflow hot-upgrade is not
supported; compatible-runtime completion or explicit terminal cancellation is
required, with remaining external effects reconciled.


Integrated native Workflow acceptance also requires Issue43's reviewed record-only
actor binding to preserve the admitted caller's Task currency while retaining
exact scope/lifecycle/claim/private-slot CAS. Independent context core validation
does not claim that pending cross-provider integration already works.


A native actor allocation cannot certify a different historical input after
prewire restoration: binding matches the exact current prepared frame, and success
requires that input's admitted/consumed proof plus authoritative terminal outcome.
Future instruction-authority changes require incompatible-writer exclusion as
well as new semantic versioning, including already-open older runtimes.


Generic history cannot turn persisted Lost/uncertain ownership into terminal native
completion; verified owned recovery remains an explicit Issue14 prerequisite.
Private preparation/admission proves exact input currency, not native transcript
truth. Provider private Session ownership must reject unregistered historical
UUID/native_ref before native resume or wire delivery. Factual Consultant/history
checkpoint append remains supported with caller-classified provenance and cannot
certify Workflow success or native completion. Pending Starting initial actor
binding revalidates the latest full frame/head/lifecycle and updates its private
preparation pair atomically; it never grants delivery. Migration verifies all
legacy typed checkpoint references before mutation, retaining actual old-producer
checksums and refusing inconsistent history.
