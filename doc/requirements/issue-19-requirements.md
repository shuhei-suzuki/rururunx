# Issue 19 requirements: durable context packs and checkpoints

Implement typed, Project/Goal/Task-scoped durable working context on existing
append-only ContextVersion and Checkpoint records. Rust remains provider independent;
no supervisor model or transcript summarization model is introduced.

- Task packs preserve purpose, criteria, workflow/risk/phase, constraints, owned
  repository/worktree/revision, scoped authoritative artifact references, selected
  file/symbol metadata, decisions, completed work, failures, verification/findings,
  blockers and next action. Source bodies remain authoritative outside summaries.
- Goal packs preserve objective/criteria/constraints/source references, DAG and
  Task status/context references, cross-Task decisions, blockers/next runnable work
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
  Byte/estimated-token budgets apply to fully rendered payloads; provider metrics
  stay nullable. Insufficient budget has no launchable input.
- Native I/O stays outside SharedStore. Publication rechecks current versions and
  live/Lost Session/lock/workflow ownership inside the transaction. Workflow-owned
  publication uses the workflow phase port; standalone publication cannot rewrite
  its phase/attempt context. The ordered format 3→4 migration
  makes older writers refuse the authoritative checkpoint write contract. SQL
  checkpoint-head indexing is maintained atomically with append. Generic Context
  writes and pointer changes cannot bypass typed publication or restore an older
  pack. Record kinds remain unchanged.

Goal-only packs use read-only exact primary-root identity/source validation and
atomic Project/Goal/Task-summary CAS, including before Tasks and after worktree
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
