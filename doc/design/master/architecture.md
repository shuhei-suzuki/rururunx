# rururunx Architecture

**Status:** Draft
**Scope:** MVP master architecture

## 1. Architectural goal

rururunx is a thin local workflow runtime around native coding agents.

It must coordinate agents without becoming another agent framework. Executor work should travel through the shortest practical path:

```text
User / Workflow
      ↓
   Scheduler
      ↓
 Agent Adapter
      ↓
 Native Agent CLI
```

Additional LLM calls are reserved for explicit review, approval, classification, or consultation steps.

## 2. Main components

```text
                   ┌──────────────────┐
                   │   CLI / TUI      │
                   └────────┬─────────┘
                            │
                    ┌───────▼────────┐
                    │ Task Manager   │
                    └───────┬────────┘
                            │
              ┌─────────────▼─────────────┐
              │ Workflow Engine / FSM     │
              └───────┬─────────┬─────────┘
                      │         │
             ┌────────▼───┐ ┌───▼────────────┐
             │ Scheduler  │ │ Approval Broker │
             └──────┬─────┘ └──────┬─────────┘
                    │              │
             ┌──────▼──────────────▼──────┐
             │      Agent Registry         │
             │ + Adapter Capability Map    │
             └──────┬──────────────┬───────┘
                    │
             ┌──────▼───────────────┐
             │ Context Efficiency   │
             │ Map / Pack / Condense│
             └──────────────────────┘
                    │              │
           ┌────────▼───┐     ┌────▼────────┐
           │ Executors  │     │ Reviewers   │
           └────────────┘     └─────────────┘

      ┌──────────────────────────────────────┐
      │ Local State / Audit / Event Journal  │
      └──────────────────────────────────────┘

      ┌──────────────────────────────────────┐
      │ Git / Worktree / Project Rule Layer  │
      └──────────────────────────────────────┘
```

## 3. Component responsibilities

### CLI / TUI

Owns user interaction only.

Responsibilities:

- start consultation or development tasks
- show active tasks and phase/state
- display human-attention queue
- attach to native agent sessions
- request stop/resume/review actions

The TUI must not become a proprietary replacement chat interface.

### Task Manager

Owns Task identity and lifecycle metadata.

Responsibilities:

- create/load/update Task
- associate Issue, branch, worktree, workflow and agents
- expose Task snapshot to CLI/TUI
- preserve Task identity across runtime restart

### Workflow Engine

Owns phase ordering and gates.

Responsibilities:

- QUICK / STANDARD / STRICT
- project-specific workflow overrides
- dynamic workflow escalation
- required review phases
- required verification phases
- merge/cleanup gates
- state transitions

### Scheduler

Owns runnable work.

Responsibilities:

- concurrency limit
- executor/reviewer scheduling
- background parallel review
- retries/timeouts
- waiting state detection
- task priority
- resume after process/runtime restart

### Agent Registry / Adapter layer

Owns native-agent integration.

Responsibilities:

- configured agents
- capability discovery
- launch/attach/resume
- structured review/approval request/response where supported
- process/session identity

### Approval Broker

Owns permission decisions.

Responsibilities:

1. normalize approval request
2. deterministic policy decision
3. choose cross-agent reviewer(s)
4. aggregate reviewer decision
5. escalate unresolved/high-risk decisions to human
6. audit all decisions

### Review Engine

Owns review sets and review rounds.

Responsibilities:

- select 1, 2, 3, or more configured reviewers
- run reviewers concurrently where possible
- support completion policy: `all`, `quorum`, `any`
- support N-of-M quorum
- preserve individual findings
- coordinate verify → fix → commit → re-review rounds
- enforce review-time worktree lock where configured

### Context Efficiency Layer

Owns context selection and token-efficiency artifacts without becoming a new planning agent.

Responsibilities:

- repository map/context index
- versioned Task Context Pack
- progressive rule/skill loading
- conversation condensation/checkpoints
- deterministic Review Bundles
- delta-based re-review context
- provider cache-awareness metadata
- token/cost telemetry when available

Correctness and safety take precedence over token savings. Agents can request context expansion when selected context is insufficient.

See `context-efficiency.md`.

### Git / Worktree Manager

Responsibilities:

- create task worktree/branch
- verify protected-branch rules
- detect dirty state
- lock immutable review worktrees
- track HEAD used by each review
- remove worktree and merged local branch at completion

### Local State / Event Journal

Recommended MVP implementation: SQLite plus append-only logical events.

Persist:

- Task snapshot
- workflow/state
- agent sessions
- worktree/branch
- review sets/rounds
- pending approvals
- decision/audit events
- recovery metadata

## 4. Task state model

Minimum states:

```text
CREATED
CONSULTING
PLANNING
IMPLEMENTING
TESTING
WAITING_APPROVAL
WAITING_REVIEW
REVIEWING
FIXING
WAITING_HUMAN
READY_FOR_PR
PR_CREATED
MERGED
FAILED
COMPLETED
CANCELLED
```

Workflow phases and process state are related but should not be encoded as one fragile enum. The implementation should keep a Task state plus phase-specific metadata.

## 5. Parallelism model

The unit of parallelism is a Task, not a terminal window.

```text
Task
 ├─ worktree
 ├─ branch
 ├─ executor session
 ├─ review set(s)
 └─ approval queue
```

MVP target: at least four simultaneous independent Tasks.

Within one Task, review agents may also run in parallel.

## 6. Review model

A Review Set contains:

- review phase
- immutable target revision
- reviewer list
- completion policy
- quorum when applicable
- model/effort requirements
- findings/results
- round number

Examples:

```yaml
reviewers: [claude, codex]
completion: all
```

```yaml
reviewers: [claude, codex, grok]
completion: quorum
quorum: 2
```

Triple review is a preset, not an architectural primitive.

## 7. Worktree locking

A review that depends on immutable HEAD creates a logical write lock for that Task worktree.

The runtime must block executor continuation that would mutate the locked worktree.

If a user wants parallel remediation before a review ends, it must occur in another worktree.

## 8. Consultation model

`rrx consult` creates a consultation session rather than a development Task workflow.

A consultation may be promoted into a Task.

Promotion copies explicit context/artifacts, but formal workflow phases still run when required.

## 9. Recovery model

Runtime restart must recover orchestration state without assuming a child process is still alive.

On recovery, rururunx should reconcile:

- persisted Task/session state
- actual process/session existence
- worktree/branch state
- pending reviews
- pending approvals

The result may be resume, reconnect, retry, fail, or human escalation.

## 10. Safety boundaries

rururunx must not:

- bypass native agent permission controls
- weaken repository hooks
- silently downgrade project workflow
- mutate protected branches contrary to project policy
- allow reviewers to execute the action they are only reviewing
- mark review complete without satisfying configured completion policy

## 11. Extension boundaries

MVP extensibility points:

- Agent Adapter
- Workflow definition/preset
- Approval policy
- Review policy
- project rule loader
- verification runner
- context selector/indexer
- context condensation strategy
- provider cache telemetry

Cloud execution, web dashboards, and long-term memory are outside MVP and should not shape the core architecture.
