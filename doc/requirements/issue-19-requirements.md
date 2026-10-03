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
