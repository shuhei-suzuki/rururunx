# Issue 23: Persistent Goal model and Task DAG

Workflow: STRICT (shared persistence, lifecycle and completion authority).
Status: proposed requirements; independent review and design/source gates pending.
Baseline: main4851fcd. Depends on merged #2, #8 and #26; part of #22.

## Purpose and current gap

Provide the provider-independent Goal model used by scheduler #14, continuous
execution #24 and global multi-project scheduler #27. Goal completion remains
explicit; completing one Task never implicitly completes its Goal.

The current Goal already persists objective, criteria, constraints/non-goals,
source references, states, DAG, follow-ups and context_version. Store validates
unique DAG nodes, declared edge endpoints and Task ownership. It does not detect
cycles, evaluate readiness/criteria, validate completion evidence, or prevent an
arbitrary satisfied flag from being treated as completion. Those existing fields
are foundations, not evidence that this issue is implemented.

Source of truth: Issue23, Product Requirements sections4–7, and
[Goal Runtime](../design/master/goal-runtime.md). Requirements/design precede source.

## Ownership, persistence and bounds

- A Goal belongs to one registered Project, with immutable Project/Goal identity.
  All nodes, edges, criteria evidence, follow-ups and context references stay in
  that exact Project/Goal scope; UUID/Issue/title similarity grants no ownership.
- Persist and restore objective/title, explicit identifiable criteria, constraints,
  non-goals, source-of-truth refs, lifecycle, DAG, proposal disposition and context
  reference/version. A Goal Context Pack reference identifies a real scoped
  artifact/version; a caller-supplied integer alone proves nothing. Producing the
  pack remains #19/#24 integration, not a placeholder acceptance result here.
- Accepted writes use transactional current-version checks and scoped audit. A
  conflict or validation/audit/write failure changes no row, version or audit and
  does not silently retry with refreshed authority. Readiness is computed from a
  coherent durable snapshot and records the versions used; it grants no dispatch
  permit. Admission rechecks current authority in scheduler/Workflow consumers.
- Define finite serialized-size, node, edge, criterion, proposal, reference and
  individual-text bounds in design. Enforce them before costly parsing/traversal
  and before publication. Checked versions never wrap; oversized/malformed input
  rejects without writes. Evaluation is iterative and bounded by graph size.

## Graph and readiness

- Nodes refer to actual Tasks in the same Project and Goal. Reject duplicates,
  missing/foreign nodes or endpoints, duplicate edges and self-dependencies.
  Do not silently attach an unlisted Task or alter its owner/worktree/Workflow.
  Adding an accepted Task/node/dependencies must be atomic, without a durable
  intermediate foreign/orphaned node. Existing unrelated Tasks are not adopted.
- Hard edges require the prerequisite's verified success. Reject every hard-edge
  cycle on creation or modification, including through existing paths. Soft edges
  are explicitly advisory preferences; soft-only or mixed advisory cycles cannot
  deadlock hard readiness. Define deterministic ordering/ties in design.
- Compute at least ready, running, waiting-human, blocked, completed and failed
  results with actionable reasons. Readiness requires an active registered
  Project, Running Goal, admissible Task and satisfied hard prerequisites.
  Paused, inactive, failed or cancelled owners never gain a ready classification.
- Native preparation/dispatch reservations, live Sessions, Lost/unknown outcomes,
  review/approval waits and active recovery reservations remain owned. A missing
  public Session ID or PID is not proof of idle/dead/completed work. Incorporate
  the actual Workflow/Session/lock facts; blocked descendants explain which hard
  prerequisite or owned uncertainty prevents progress.
- Failed/cancelled prerequisites are not successful. Verified Workflow completion,
  including configured PR/merge/cleanup obligations, is required; Task Merged
  alone does not prove cleanup. Retries/recovery require their explicit owning
  ports; readiness cannot reset terminal rows or release claims.
- Resource limits/fairness belong to #14/#27. Logical ready does not mean a permit
  is acquired or a process is running, and this issue does not claim 4+ execution.

## Explicit completion and lifecycle

- Criteria have stable unique IDs, a nonempty description and a defined evaluator
  or explicit Human-attestation path. Each result references durable scoped
  evidence and its target/source versions. Unsupported evaluators and absent,
  foreign, stale or unverifiable evidence remain unsatisfied/blocked, with reason.
  LLM statements, arbitrary strings, counts and public satisfied=true are claims,
  not verified completion authority. No hidden supervisor/model call is required.
- Required Task success is necessary and does not replace the explicit criteria.
  Completion requires all current criteria satisfied, no unresolved required
  follow-up/material-scope decision, and no live/uncertain native ownership,
  pending reviews/approvals, unfinished Workflow or active locks/recovery claims.
  Evidence checking and terminal publication compare the same current scope and
  version frame atomically. External checks occur outside the Store transaction;
  their exact observed result is revalidated before commit, never guessed from
  a URL/path or exit zero. Proposed evaluator contracts must identify the actual
  trusted evidence producer, not a generic JSON bypass.
- Existing Created/Analyzing/Running/WaitingHuman/Paused/Blocked/Completed/
  Cancelled/Failed states get an explicit legal transition contract. Ordinary
  persistence cannot forge Completed around the completion evaluator. Terminal
  Goals cannot silently reopen, change owners or regain native dispatch authority.
  Pause/cancel/failure do not assert process death or release held resources;
  #14 reconciles actual owned Sessions before release. Lifecycle controls remain
  explicit, independently audited authority changes.
- Restart recomputes readiness/completion from durable facts. A stored ready list,
  stale criterion bit or native session hint does not substitute for evaluation.
  Source/criterion/DAG changes invalidate affected evaluation results. Agent
  replacement/native session loss cannot erase objective, graph or evidence.

## Follow-ups and native currency

- A proposal records stable identity, title/scope, Goal-related rationale,
  acceptance criteria, same-scope dependencies, risk and material-expansion flag.
  Its typed disposition distinguishes proposed, accepted, rejected and requiring
  Human decision. Accepted proposals link to the actual resulting Task; repeated
  acceptance is idempotent and cannot create duplicate Tasks or edges.
- Material/unrelated scope expansion requires recorded Human disposition; agent
  proposals cannot approve themselves. Automatic creation policy and external
  GitHub Issue creation belong to #24/#13; this issue provides their validated
  model/transaction boundary without silently executing either operation.
- Preserve existing native raw Project/Goal/Task and complete lock-set currency.
  Derived progress/status, proposal observations, metrics/audit and sibling Task
  completion must not rewrite a native-fenced Goal/Task just to publish facts,
  invalidate a sibling's unchanged input, or relax native scope checks. Design
  must provide a coherent persistence boundary for these observations versus
  explicit authority changes. Do not duplicate competing authoritative DAGs or
  smuggle input/permission authority into a progress record. Inventory actual
  Claude/Codex/Grok consumers and #19 frames before source integration.

## Verification and delivery

Use isolated Projects/SQLite connections. Cover persistence/reopen, independent
ready nodes and a diamond graph, hard cycles/back-edge updates, advisory cycles,
missing/duplicate/foreign references, failed dependencies, owner inactivity,
preparing/Lost reservations, verified success versus Task Merged, stale evidence,
criteria-not-satisfied despite completed Tasks, actual current completion and
terminal reopening refusal. Exercise transactional fault/CAS races with a second
writer, idempotent proposal acceptance and material expansion requiring Human.
Test two Projects with overlapping Issue numbers and unrelated DAGs/evidence.

Meaningful compiled mutants must reach actual graph/evaluation/Store consumers:
remove cycle/scope/current-evidence checks; count Task completion as Goal
completion; treat failed/Lost ownership as ready/success; duplicate an accepted
proposal; publish observations by rewriting native-fenced Goal authority. Include
passing controls and restored source, with no setup/compilation failure credit.

Review requirements, then design, then immutable implementation/security scope.
Run appropriate shared-state/native/Workflow regressions, fmt/clippy/build and
exact-head Linux/macOS CI. Coordinate schema/projection/writer fencing with #19;
legacy data is never silently marked verified or ready. Main receives a compatible
reviewed migration and updated Goal master design/README in the source PR.
CLI/event loop #24, scheduling/recovery #14/#27, packs #19, review/broker integrations
and final multi-project dogfood #16 retain their own required acceptance.
