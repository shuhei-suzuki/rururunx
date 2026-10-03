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
