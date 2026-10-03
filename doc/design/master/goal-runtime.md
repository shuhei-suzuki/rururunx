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
Update Goal context/progress
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
