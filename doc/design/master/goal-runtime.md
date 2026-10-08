# Goal Runtime Design

Proposed native-execution policy: see [result protection](agent-execution.md).
It supersedes conflicting ownership/live-worktree prerequisites only when the
new profile is implemented; historical/current implementation sections below
remain baseline descriptions, not acceptance of that proposal.

**Status:** Draft
**Scope:** MVP first-class Goal orchestration

The independently reviewed [Runtime/Scheduler integration](../runtime-scheduler-integration-design.md)
defines typed accepted Goal authority, actual prerequisite evidence, fair driver
claims and private retained/pre-input source recovery. These ports and the
complete operational CLI remain implementation work; structural DAG support alone does
not execute or complete a Goal.

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

## Current implementation foundation

Goal snapshots and scoped Task references persist through the SQLite Store.
TaskDag::hard_order validates finite node/edge bounds, declared unique nodes,
non-self unique ordered edge pairs and hard dependency cycles. Store::put_goal
uses it before publishing the Goal and its audit; soft advisory cycles are allowed.
The deterministic order includes all declared nodes and grants no readiness or
dispatch authority. The Runtime control foundation includes typed Goal creation,
scoped status and bounded Task reads. The compiled foreground `rrx serve`
and `rrx goal status/tasks` now consume the actual private Unix endpoint with
Project routing and current-owner authentication. Task pages carry a Goal version
and individual Task versions, with a scope-checked bare cursor; separate pages
remain independent observations. A final page does not assert a complete snapshot.

These read clients do not construct owners or write rows, versions, epochs,
acknowledgements, audits or attention. Native dispatch is explicitly unavailable.
Goal status and Task reads explicitly request the strict `recorded_v1` view.
Omitted/null view requests retain legacy wire output. Status projects exact
recorded criteria and DAG counts from the validated Goal; Task pages pair the
original effective state with exact stored state/version and incoming hard/soft
endpoint identities, versions and stored states from the same validated inventory.
Recorded flags/evidence are not verified completion; criterion evaluation remains
unavailable and runnable admission unknown. Evidence stays opaque, without reads.
The complete recorded status/proposal response is capped at 128 KiB, preserving
every existing valid 16 KiB proposal objective including JSON escape expansion.
Task pages retain 64 KiB and incoming detail is capped at 8 KiB. Whole criteria or
incoming sets that exceed their budget are explicitly unavailable with exact
recorded counts; page packing preserves matched Task/node pairs and bare cursor
membership/order. Transport and strict output profiles retain their existing limits.
Plain formatting escapes every user-controlled Unicode control character including
DEL/C1; JSON preserves exact decoded stored strings. Both formats label independent,
incomplete observations and never infer combined completeness, evaluation or capacity.
Inline and bounded UTF-8 file objectives now persist through actual Human ingress
as inert Analyzing proposals. They have no criteria, Tasks, accepted GoalAuthority
or scheduler entries. Existing protected Goal observation rows pin nongrant
proposal provenance and the exact proposed Goal version independently of their
own observation revision; they grant no acceptance. Status distinguishes an inert
proposal from an accepted definition. Explicit bounded TOML plans include Project
selection, expected Project version and the typed definition/graph; validation and
acceptance remain service-owned. Prose is never promoted into that path implicitly.

Goal pause/resume/cancel CLI requires explicit expected Goal version and reason.
CAS and scope failures surface without current-row refresh/retry. Resume retains
the existing accepted-policy and genuinely never-prepared-history checks and
cannot revive a prior Unit/Session/grant. Typed additive followups, Task stop/retry,
logs/actual attach, verified continuous Goal execution and the TUI remain pending
with native producer integration. This partial source does not satisfy full Goal
or MVP acceptance.

### Published source reconstruction component checkpoint

The private Published committed-source route is implemented and qualified at
`5c070d4aa4502e9ef737fe5960e74fbd03cdb92b`; see the
[source recovery checkpoint](../../verification/agent-execution-phase2-source-recovery-checkpoint.json).
It reconstructs source bytes through registered retained reads and exact current
pins, installs no old Unit/Session/native permission, and supports legitimate typed
Workflow bookkeeping. It does not construct a Runtime Driver or make restart
scheduling available. Fresh pre-artifact recovery and full operational Goal/CLI
construction remain required.


The unmerged Runtime source constructs one managed Registry/Sources/Gates/Verifier/
Engine graph once. Installation refusal retains usable Goal controls. A synchronous
try-lock sweep evaluates at most 32 bounded Task bodies, retains a finite-pass
cursor and rereads current ranks and distinct-Task capacity after each claim.
Only a genuine claim clears the exact native binding hold in its transaction;
reconcile preserves driving attention and restores the hold after closure or an
epoch change. Goal facts name native_continuation_unavailable for a current driving
Task with cleared attention, while dispatch_available remains false. These display
facts grant nothing. The original first Executor lane remains unmerged and
unqualified; continuation, Root stop integration and full Native acceptance are open.

Scoped installed-graph, paired undeclared CLI, bounded Task read/reread, shutdown
admission and finite-pass controls pass, along with the six existing compiled Goal
CLI controls. These are account-free wiring/admission controls. The genuine Source
lane remains Held before its Native marker because a composed Workflow activation
producer is absent; no Native execution or full Goal completion is established.
