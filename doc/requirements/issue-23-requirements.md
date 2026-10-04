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
  reference/version. A Goal Context Pack reference identifies a typed scoped artifact/version. This
  issue validates/persists that reference and its monotonic version; unresolved
  producer references remain visibly unresolved and grant no readiness/completion
  authority. #19/#24 must resolve the real artifact before pack use. A caller-
  supplied integer alone proves nothing; no producer acceptance is claimed here.
- Accepted writes use transactional current-version checks and scoped audit. A
  conflict or validation/audit/write failure changes no row, version or audit and
  does not silently retry with refreshed authority. Readiness is computed from a
  coherent durable snapshot and records the versions used; it grants no dispatch
  permit. Admission rechecks current authority in scheduler/Workflow consumers.
- Define finite serialized-size, node, edge, criterion, proposal, reference and
  individual-text bounds in design, including all Goal-scoped Tasks scanned even
  when absent from dag.nodes. Enforce them before costly parsing/traversal
  and before publication. Checked versions never wrap; oversized/malformed input
  rejects without writes. Evaluation is iterative and bounded by graph size.

## Graph and readiness

- Nodes refer to actual Tasks in the same Project and Goal. Reject duplicates,
  missing/foreign nodes or endpoints, duplicate edges and self-dependencies.
  Do not silently attach an unlisted Task or alter its owner/worktree/Workflow.
  Adding an accepted Task/node/dependencies must be atomic, without a durable
  intermediate foreign/orphaned node. Existing unrelated Tasks are not adopted.
- Required Tasks are all actual Tasks in the exact Project/Goal scope, including
  legacy Tasks absent from dag.nodes; completion/ownership scans never omit them.
  A new managed Task must be atomically registered as a DAG node. Legacy unlisted
  Tasks are reported as unresolved membership and need explicit validated
  reconciliation, never automatic adoption or success. MVP has no implicit
  descoping: node removal, edge removal and hard-to-soft weakening are unsupported
  explicit errors. Failed/cancelled required Tasks keep the Goal incomplete with
  a reason requiring controller/Human action. Retry through the owning port of
  the same Task identity may produce fresh verified success; a replacement Task
  does not supersede the old requirement. Otherwise explicit authorized Goal
  failure/cancellation is required. The same applies to unreconciled legacy Tasks.
- Adding a hard dependency requires its dependent to be unstarted: Created, no
  Workflow attempt/history, Session launch or dispatch/recovery ownership. Reject
  retroactive prerequisites for running, preparing, Lost or terminal dependents;
  neither a Human label nor an audit alone makes prior execution meet a new gate.
  Edge identity is the ordered (prerequisite, dependent) pair, independent of the
  hard flag; soft-to-hard strengthening uses the same unstarted-dependent rule.
  Parallel hard/soft duplicates reject. Soft preferences cannot remove a required Task.
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

- Goal definition is authority: objective/title, the complete criterion set and
  each criterion ID/description/evaluator, constraints, non-goals and source-of-
  truth references. Initial accepted creation requires the Human-authority ingress
  or trusted controller acting under an explicitly Human-approved creation policy,
  with current scoped transaction checks and recorded origin/policy. Agent, native,
  Workflow and generic persistence cannot create an accepted definition from their
  own labels or change one; they may submit requiring-Human proposals only.
- An accepted Goal definition is immutable in MVP. Adding/removing/replacing
  criteria, changing an evaluator or Human-attestation path, reusing criterion IDs,
  or editing objective/constraints/non-goals/source references rejects explicitly,
  including through a controller/Human edit request. There is no silent reduction
  of scope or conversion of a Human criterion to Task-only success. Proposed
  definition changes are not applied and grant no readiness/completion authority;
  a distinct explicitly accepted Goal does not settle the original Goal. Referenced
  source contents may change, invalidating evidence, without rewriting the fixed
  reference set or original definition.
- Human criterion attestation and material/requiring-Human proposal disposition
  require a dedicated Human-authority ingress, distinct from agent/native/Workflow
  writers. Design must name the principal source and process/API boundary; record
  origin class/principal and exact criterion/proposal identity plus version. Labels,
  Git comments/trailers, agent output and generic persistence cannot mint Human
  authority. Target changes invalidate the attestation. Automatic lifecycle policy
  uses a separate trusted runtime-controller port, unreachable from agent, native
  and Workflow writers, never fabricated Human origin.
  Specify trust limits, including same-user machine/DB tampering; do not claim an
  OS sandbox or biological identity proof from an in-process typed port.
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
  Existing legacy terminal Goals remain terminal but completion-unverified; they
  are quarantined from managed success/dispatch, never reopened or re-certified
  from stale bits. Pause/cancel/failure do not assert process death or release held resources;
  #14 reconciles actual owned Sessions before release. Lifecycle controls remain
  explicit, independently audited authority changes.
- Persisted lifecycle states are authority, distinct from computed progress.
  Created/Analyzing/Running and explicit Blocked/WaitingHuman/Paused holds change
  only through the trusted controller/Human ports with scoped policy and current
  versions. Completed is evaluator-published; Cancelled/Failed use explicit
  controller/Human authority. Generic persistence cannot change lifecycle.
  Computed node/summary blocked or waiting-human reasons leave Goal Running:
  resolving a Lost prerequisite or proposal can restore eligibility without a
  manual Goal transition. A persisted whole-Goal hold instead requires its own
  recorded authorized resolution before Running. Requiring-Human proposals block
  acceptance/completion but do not automatically pause unrelated eligible work.
  Goal.blockers strings are display-only claims, never authority to suppress or
  permit work. Actual holds/reasons come from current typed scoped model facts;
  recording/evaluating evidence and proposals does not mutate native currency.
- Restart recomputes readiness/completion from durable facts. A stored ready list,
  stale criterion bit or native session hint does not substitute for evaluation.
  Referenced source-content, evidence-target and permitted additive DAG changes
  invalidate affected evaluation results; the accepted criteria remain fixed. Agent
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
  explicit authority changes. Proposal acceptance and node/dependency additions
  are explicit authority changes; design must specify their native-currency and
  sibling re-admission consequences without laundering changes as observations.
  Do not duplicate competing authoritative DAGs or smuggle input/permission authority into a progress record. Inventory actual
  Claude/Codex/Grok consumers and #19 frames before source integration.

## Verification and delivery

Use isolated Projects/SQLite connections. Cover persistence/reopen, independent
ready nodes and a diamond graph, hard cycles/back-edge updates, advisory cycles,
missing/duplicate/foreign references, failed dependencies, owner inactivity,
preparing/Lost reservations, verified success versus Task Merged, stale evidence,
criteria-not-satisfied despite completed Tasks, one non-Human evaluator of actual
verified required-Task success, current completion and
terminal reopening refusal. Exercise transactional fault/CAS races with a second
writer, idempotent proposal acceptance and material expansion requiring Human.
Test two Projects with overlapping Issue numbers and unrelated DAGs/evidence.
Refuse agent-originated Human attestation/disposition and lifecycle changes;
exercise target-version invalidation, retroactive hard edges, removal/weakening,
legacy unlisted/terminal-unverified records, and Lost-prerequisite resolution
restoring readiness without changing a Running Goal. Display-only blockers
cannot change completion/eligibility. Refuse generic/agent criterion removal,
replacement or evaluator substitution and edits to constraints/non-goals/source
references; verify initial definition origin and immutable accepted definitions.

Meaningful compiled mutants must reach actual graph/evaluation/Store consumers:
remove cycle/scope/current-evidence checks; count Task completion as Goal
completion; treat failed/Lost ownership as ready/success; duplicate an accepted
proposal; accept an agent-originated Human decision; permit retroactive
prerequisites/removal; treat display blockers as authority; publish observations
by rewriting native-fenced Goal authority. Include
a compiled criterion-replacement mutant that makes Task-only completion reachable
through generic persistence. Include passing controls and restored source, with no
setup/compilation failure credit.

Review requirements, then design, then immutable implementation/security scope.
Run appropriate shared-state/native/Workflow regressions, fmt/clippy/build and
exact-head Linux/macOS CI. Coordinate schema/projection/writer fencing with #19;
legacy data is never silently marked verified or ready. Main receives a compatible
reviewed migration and updated Goal master design/README in the source PR.
CLI/event loop #24, scheduling/recovery #14/#27, packs #19, review/broker integrations
and final multi-project dogfood #16 retain their own required acceptance.
