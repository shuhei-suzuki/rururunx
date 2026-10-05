# Goal Runtime Design

**Status:** Draft
**Scope:** MVP first-class Goal orchestration

## 1. Goal

Provide a provider-independent long-running objective above individual Tasks.

For MVP, every Goal belongs to exactly one registered Project. Multiple Projects and multiple Goals may execute concurrently under the global Runtime scheduler.

A Goal owns the durable objective, completion criteria, constraints, dependency graph, progress, and escalation state.

```text
Goal
  ↓
Task DAG
  ↓
Tasks / Issues
  ↓
Workflow
  ↓
Agent Sessions
```

A Goal must survive agent replacement, native session loss, and rururunx restart.

## 2. Goal entity

Suggested fields:

- goal_id
- project_id
- title
- objective
- completion_criteria[]
- constraints[]
- non_goals[]
- source_of_truth_refs[]
- escalation_policy_ref
- state
- task_nodes[]
- task_edges[]
- active_task_ids[]
- goal_context_pack_version
- created_at / updated_at
- completed_at
- audit metadata

Task nodes/edges, active Task IDs and per-Task progress/timestamps are derived
from Task-scoped records. Updating these views or collecting each Task event
must not routinely rewrite the shared Goal row. Goal Context Pack publication
must likewise respect the Workflow CAS constraint in section 14; this design
does not establish a separate semantic-version policy.

## 3. Goal states

```text
CREATED
ANALYZING
RUNNING
WAITING_HUMAN
PAUSED
BLOCKED
COMPLETED
CANCELLED
FAILED
```

State transition must be audited.

## 4. Project binding

Goal `project_id` is immutable for MVP after execution begins.

All Task DAG nodes inherit that Project unless explicitly rejected as invalid.

Goal Runtime obtains project rules, repository root, worktree namespace, and source-of-truth references from Project Manager.

Cross-project Goal DAG edges are not supported in MVP.

## 5. Task DAG

A Goal graph contains Task nodes and hard/soft dependency edges.

A Task becomes runnable when:

- the Goal is RUNNING
- all hard dependencies are satisfied
- required project/workflow prerequisites are available
- concurrency/resource policy permits scheduling
- it is not blocked by Human escalation

Independent ready Tasks may run in parallel.

Cycle detection is required before accepting hard dependency changes.

## 6. Goal loop

Logical control loop:

```text
Load Goal
   ↓
Reconcile persisted/runtime state
   ↓
Evaluate completion criteria
   ├─ complete → COMPLETED
   ↓
Recompute ready/blocked Tasks
   ↓
Schedule ready Tasks
   ↓
Collect Task/review/approval events
   ↓
Update derived Task-scoped progress / publish Goal context with CAS coordination
   ↓
Discover required follow-up work if any
   ↓
Evaluate Human escalation
   ↓
repeat
```

The loop is event-driven where possible; it must not busy-poll.

## 7. Completion evaluation

Completion is explicit, not inferred only from Task count.

Completion criteria may reference:

- required Issues/Tasks completed
- required acceptance criteria
- required test/review gates
- required dogfood/scenario result
- no unresolved severity threshold findings
- documentation consistency
- measured KPI/benchmark artifact existence

A Goal may have tasks completed but remain incomplete.

## 8. Goal planning / Task discovery

The MVP may begin with a Goal that references an existing Issue graph or explicit Task list.

Where planning is needed, rururunx may ask a configured planning-capable agent to propose:

- Tasks
- dependencies
- rationale
- acceptance criteria
- risks

The proposal is normalized into rururunx Task/DAG data.

The planning agent is advisory; rururunx owns the resulting graph.

## 9. Follow-up Task creation

During execution, a Task may produce a follow-up proposal.

A proposal must include:

- why the work is required for Goal completion
- suggested title/scope
- dependencies
- risk/workflow recommendation
- whether a GitHub Issue should be created

Policy decides whether to auto-create, queue for review, or escalate.

Unrelated scope expansion is not allowed.

## 10. Goal Context Pack

Goal Context Pack is separate from Task Context Packs.

It contains cross-Task durable information:

- objective / completion criteria
- constraints / non-goals
- source-of-truth refs
- summarized Task DAG state
- important cross-Task decisions
- unresolved Goal blockers
- next runnable work
- aggregate review/security status
- aggregate token/cost/human-interruption metrics where available

It does not include every Task transcript.

## 11. Provider-native Goal capability

Adapters may expose:

```text
native_goal
native_goal_status
native_goal_resume
```

A provider-native Goal is optional.

Rules:

- rururunx Goal remains source of truth
- provider-native goal ID/session is stored as an execution hint/reference only
- failure/loss of native goal session cannot destroy Goal state
- another agent/provider can continue from Goal + Task Context Packs
- native goal output/events are normalized into rururunx events

For Codex, native `/goal` may be used where programmatically/safely available; otherwise rururunx sends the Goal Context Pack/instructions through normal adapter mechanisms.

## 12. CLI

Initial interface:

```bash
rrx goal "<objective>"
rrx goal --file goal.md
rrx goal status [goal-id]
rrx goal pause [goal-id]
rrx goal resume [goal-id]
rrx goal cancel [goal-id]
rrx goal attach [goal-id]
```

Future-friendly but not required for MVP:

```bash
rrx goal graph [goal-id]
rrx goal add-task ...
rrx goal replan ...
```

## 13. TUI

Goal view should show:

- Goal title/state
- completion criteria
- progress summary
- active/ready/blocked/completed Tasks
- dependencies
- agent/reviewer activity
- Human attention queue
- aggregate token/cost/time metrics

## 14. Persistence and recovery

Persist:

- Goal entity
- criteria/constraints
- Task nodes/edges
- task-to-Goal relationships
- provider-native goal references
- Goal Context Pack version
- Goal state/audit history

On restart:

1. load Goal
2. reconcile Task/process/session state
3. recompute graph readiness
4. re-evaluate completion criteria
5. resume or move to BLOCKED/WAITING_HUMAN as appropriate

Workflow checks Project/Goal row versions in marker, definitive-publication and
post-dispatch Session-binding CAS. Per-Task
progress should use Task-scoped records rather than bumping the shared Goal row
on every step; otherwise concurrent sibling preparation claims can remain
reserved and require Issue #14 recovery. This is a current ownership constraint,
not a new semantic Goal-version policy.

## 15. Performance

Goal orchestration must be lightweight.

The Rust runtime should use event-driven process/state updates and avoid a high-frequency supervisor polling loop.

Goal scheduling overhead should remain small relative to native agent runtime.

## 16. Safety

Goal autonomy never overrides:

- project safety rules
- native permission boundaries
- approval policy
- workflow minimums
- Human-required operations

Goal may continue automatically only inside those boundaries.

## 17. MVP acceptance

- persistent Goal entity
- explicit completion criteria
- Task DAG with cycle detection
- ready Task scheduling
- 4+ parallel Tasks when graph permits
- continuous Goal loop across Task completion
- follow-up Task proposal/creation policy
- Goal pause/resume/cancel/status/attach
- restart recovery
- Goal Context Pack
- TUI Goal view
- optional native-goal adapter capability
- provider-independent continuation

Current proposed19/43 exact native/marker authority uses raw Goal versions. The
mandatory 4+ parallel Tasks/follow-up/two-Project availability gate is not met merely
by safe stale-version refusal. Candidate23/24 controller composition derives factual
progress from Task facts/separate bounded Records, avoiding Goal-row rewrites on
every poll/completion; ready accepted Tasks may run concurrently. Authoritative
DAG/follow-up acceptance queues until exact owned native phase closure (binding
plus closure/non-success closure), not settlement receipt alone, or
deliberately revokes under existing admission gates. It does not refresh original
marker pins or exempt external bookkeeping. Actual composed controller/native tests
must demonstrate progress, exact currency and safe held/recovery outcomes before
whole-source/MVP acceptance; this proposed path is not an implemented controller.
A durable pending authoritative-change barrier blocks new native markers for that
Goal without rewriting Goal.version; healthy in-flight phases drain before the actual
trusted controller atomically applies/clears it. Lost/unknown closure retains the
barrier for real recovery rather than certifying progress.

Typed Goal context publication has separate metadata authority: `Goal.version`
tracks semantic/lifecycle changes, while `ContextVersion.version` is a consecutive
immutable head and the Goal's context pointer changes in the same Immediate
transaction with its audit. Pointer-only publication does not increment the Goal
semantic version or invalidate admitted Task input authority. Generic stale Goal
writes cannot restore an older typed pointer. Concurrent differing publications
compete on the latest ContextVersion head; identical publications reuse that head.
Consumers of a Goal pack must validate its exact head/pointer/digest, independently
of Goal semantic version. Goal packs are non-launchable summaries.

### Native input authority

Goal progress/pointer publication cannot substitute for Task input or lifecycle
authority. The context design owns complete frame publication, source/head currency,
continuation and native ownership boundaries. See
[context publication and native input authority](context-efficiency.md#17-context-publication-and-native-input-authority)
for the normative contract and its implementation boundary; the Goal design does
not duplicate those predicates.


The incompatible managed native/Goal authority remains the pending
[Issue19 contract](../issue-19-design.md#canonical-schema6-native-writer-predicate-table).
Actual main is schema3; unmerged component5 pack publication is not accepted managed
Goal/native settlement. Real23 typed Goal writers must preserve the latest pack pointer
separately from semantic version; actual source composition/production gates are pending.

The pending composed typed Goal-writer inventory explicitly includes19 non-launchable
context-pointer publication (context_version/updated_at only, unchanged semantic
version), preserving accepted definitions and separate consecutive head CAS; see the
[proposed composition contract](../issue-19-design.md#design34-complete-factual-workflow-succession-and-scoped-authority-exits).
It is not an implemented23 authority exception or native launch grant.

## Current implementation foundation

Goal snapshots and scoped Task references persist through the SQLite Store.
TaskDag::hard_order validates finite node/edge bounds, declared unique nodes,
non-self unique ordered edge pairs and hard dependency cycles. Store::put_goal
uses it before publishing the Goal and its audit; soft advisory cycles are allowed.
The deterministic order includes all declared nodes and grants no readiness or
dispatch authority. Managed definitions, lifecycle, verified completion, controller
loop and CLI remain pending in #23/#24 with #19/#43 producer integration.
