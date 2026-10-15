# rururunx Product Requirements

**Version:** 0.9
**Status:** Draft
**Project:** rururunx
**CLI:** `rrx`
**Distribution:** Open source / CLI-first / local-first

## 1. Purpose

rururunx is a lightweight workflow runtime and process supervisor for coding agents.

It coordinates existing native agents such as Claude Code, Codex, Grok, Gemini, and open-source coding agents instead of replacing them. The runtime manages task parallelism, Git worktrees, workflow progression, reviews, approvals, escalation, recovery, and observability.

Primary outcome:

> Multiple coding agents can keep developing in parallel without a human repeatedly checking terminals or answering routine prompts.

## 2. Problem

Coding agents can already perform substantial development work autonomously. The remaining bottleneck is the human supervisor who must watch several terminals, answer permission prompts, launch reviewers, coordinate review/fix rounds, protect worktrees during review, check tests/CI, manage PRs, and decide how much process each change needs.

rururunx automates that supervision layer while preserving native agent behavior and project safety rules.

## 3. Product principles

1. **Native agent first** — use native agents through adapters; do not build another coding agent.
2. **Thin execution path** — avoid unnecessary planner/supervisor LLM hops on the normal executor path.
3. **Provider independent** — Claude, Codex, Grok, Gemini, OSS, and future agents are peers.
4. **Parallel by default** — independent tasks run concurrently in separate Git worktrees.
5. **Cross-agent review and approval** — an executor can be reviewed by other agents.
6. **Human on escalation** — humans handle high-risk, unresolved, or disputed decisions.
7. **Existing safety remains authoritative** — never bypass repository hooks, agent permissions, Git safety controls, or project rules.
8. **CLI-first / local-first** — MVP requires no desktop app, web dashboard, or hosted control plane.
9. **Context-efficient by design** — parallelism must not blindly multiply repository/history tokens. Stable context is reused; detailed context is retrieved progressively; long sessions are condensed; re-review prefers deltas.
10. **Quality before savings** — token reduction must never remove mandatory safety rules, hide required evidence, or weaken independent review.
11. **High-performance runtime** — the MVP core runtime is implemented in **Rust** and must minimize orchestration overhead, idle resource usage, and process-management latency.
12. **Goal-oriented autonomy** — rururunx owns long-running Goals above Tasks so the runtime can continue selecting, scheduling, and verifying work until explicit completion criteria are satisfied.
13. **Multi-project by default** — one rururunx Runtime can supervise multiple repositories/projects, Goals, and Tasks concurrently while strictly isolating project rules, context, worktrees, credentials, and state.
14. **Own execution before effects** — every runtime/Agent workload must enter an exact Execution Domain before external execution; cleanup settlement is distinct from execution success, and unsupported containment/delegation fails closed.

## 4. MVP success definition

The MVP is complete when the development workflow currently performed manually across multiple terminal sessions can be operated from one `rrx` runtime.

The user must not need to patrol terminals merely to notice that an agent is waiting.

The MVP includes the current strict development practices, not only process multiplexing.

### 4.1 Scope classes

Work is classified so optional features do not delay the core runtime.

**MVP Core / release-blocking**

- Rust runtime, durable state, Git/worktree safety, and Project isolation
- Execution Domain ownership/containment, platform-specific Strong backends, delegated-operation settlement, and Task-local cancellation (#75/#76/#77/#78)
- Claude Code, Codex, and Grok native adapters
- QUICK / STANDARD / STRICT workflow execution with risk-based escalation
- configurable Review Engine and cross-agent Approval Broker
- persistent Goal model, Task DAG, scheduler, 4+ Task concurrency, restart/recovery, and multi-project fairness
- Context Efficiency: repository map, Context Packs, deterministic Review Bundles, delta re-review, and usage/cache telemetry where exposed
- provider-neutral verification gates including headed browser verification, staging where configured, PR/merge/cleanup, audit, CLI/TUI, and Human escalation

**MVP Optional / Stretch — never release-blocking**

- adaptive/agentic browser backends such as Stagehand
- Jev or similar browser fast paths
- provider-native Goal delegation
- additional adapters beyond the MVP MUST set

Optional work may land before MVP if it does not delay or destabilize the core path.

**Post-MVP**

- iOS/iPadOS companion PWA and `rrx serve` remote status UI
- Tailscale-specific companion setup beyond documentation needed for later remote viewing
- remote write actions such as approve/merge/deploy from a companion UI
- hosted dashboard/control plane

### 4.2 MVP critical path

Implementation priority is:

```text
Execution Domain contract/backends (#75/#76/#77/#78)
  ↓
Native adapters (#5/#6/#7)
  ↓
Workflow Engine (#8)
  ↓
Review Engine (#9)
  ↓
Task/Goal Context Packs + Review Bundles (#19/#20)
  ↓
Approval Broker (#10)
  ↓
Goal model / Task DAG (#23)
  ↓
Scheduler + restart/recovery (#14)
  ↓
rrx goal continuous loop (#24)
  ↓
Global multi-project scheduler (#27)
  ↓
Full dogfood (#16)
```

Independent work may proceed in parallel, but MVP Optional/Post-MVP work must not become a dependency of this path.

## 5. Core entities

### Project

A registered repository/workspace managed by rururunx.

A Project contains at least:

- project ID / display name
- canonical repository/workspace path
- repository identity / remote when available
- default/base branch
- project-scoped configuration
- project rule/source-of-truth references
- worktree root/convention
- active Goals / Tasks
- per-project concurrency policy
- project-level audit/metrics references

Project identity is stable across Runtime restarts and must prevent state/context from one repository from being accidentally reused in another.

### Goal

A long-running objective above individual Tasks and Issues.

For MVP, a Goal belongs to exactly one Project. A Project may have multiple Goals. Cross-project Goals are not required for MVP; the Runtime itself provides cross-project concurrency.

A Goal contains at least:

- goal ID / title
- objective
- explicit completion criteria
- constraints / non-goals
- source-of-truth references
- escalation policy
- Task DAG and dependency state
- current Goal state
- generated/follow-up Tasks
- Goal-level Context Pack
- progress and metrics
- audit events / timestamps

Goal state is owned by rururunx and must survive agent replacement and runtime restart.

### Task

A development unit containing at least:

- task ID / title
- repository
- GitHub Issue reference when applicable
- worktree / branch
- workflow class / risk class
- executor
- reviewers
- current state
- pending approvals
- review rounds
- artifacts
- audit events / timestamps

### Agent

A native coding agent reachable through an adapter.

Capabilities may include:

- `execute`
- `consult`
- `review`
- `inspect_diff`
- `inspect_command`
- `resume`
- `interactive`
- `non_interactive`
- `permission_interception`

### Workflow

Required phases, gates, reviewer policy, and escalation behavior for a Task.

### Execution Domain

A durable Project/Goal/Task/attempt-scoped ownership boundary for externally executing work.

An Execution Domain contains at least:

- stable domain ID and exact attempt scope
- execution profile and selected platform backend
- backend identity established before external effects
- revocation/lifecycle state
- execution outcome independent from cleanup outcome
- cleanup evidence/reference
- links to persistent delegated operations
- owner epoch for restart fencing

Strong profiles must contain every reachable local execution path and govern or pre-effect block persistent external delegation. Native/Compatibility profiles remain separately qualified and must not inherit Strong guarantees from PID/process-group cleanup alone. See Issue #75 and the Execution Domain master design.

### Approval Request

A normalized request containing action type, command/operation, executor, task, worktree, branch, risk context, policy result, reviewer decisions, and final decision.

## 6. Goal mode

Goal mode is a core MVP capability.

The user can create a persistent development objective:

```bash
rrx goal "Complete the project MVP"
rrx goal --file goal.md
```

A Goal is different from a single `rrx run` Task:

```text
Project
  ↓
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

### 6.1 Goal project binding

For MVP, every Goal is bound to one Project/repository.

Goal execution must load rules, source-of-truth documents, worktree policy, and context only from that Project unless an explicit external reference is allowed.

Multiple Goals from different Projects may run concurrently in one Runtime.

### 6.2 Goal completion

A Goal must define explicit completion criteria.

Completing one Task or one GitHub Issue never implicitly completes the Goal unless all Goal completion criteria are satisfied.

After meaningful Task/phase transitions, rururunx reevaluates:

- Goal completion criteria
- completed / ready / blocked Tasks
- dependency state
- newly discovered required work
- unresolved review/security blockers
- Human escalation conditions

If the Goal is incomplete and runnable work exists, execution continues without requiring the user to re-prompt the runtime.

### 6.3 Goal Task DAG

A Goal may contain:

- existing GitHub Issues
- rururunx Tasks not yet represented by Issues
- follow-up Tasks created during execution

Dependencies should be represented as a DAG when possible.

The scheduler may run independent ready Tasks in parallel and must not start a Task whose declared hard dependencies are incomplete.

### 6.4 Goal-driven follow-up work

During implementation or review, rururunx may discover work required to satisfy Goal completion criteria.

The runtime may propose or create follow-up Task/Issue entries when allowed by policy.

New work must:

- be linked to the Goal
- state why it is necessary for Goal completion
- preserve Goal constraints/non-goals
- avoid silent unrelated scope expansion

Material scope expansion requires Human escalation.

### 6.5 Goal lifecycle

Minimum Goal states:

- CREATED
- ANALYZING
- RUNNING
- WAITING_HUMAN
- PAUSED
- BLOCKED
- COMPLETED
- CANCELLED
- FAILED

### 6.6 Goal CLI

MVP supports at least:

```bash
rrx goal "<objective>"
rrx goal --file <path>
rrx goal status [goal-id]
rrx goal pause [goal-id]
rrx goal resume [goal-id]
rrx goal cancel [goal-id]
rrx goal attach [goal-id]
```

A default/current Goal may be inferred when unambiguous.

### 6.7 Goal progress

Goal status must expose:

- completion criteria status
- Task DAG summary
- active / ready / blocked / completed Tasks
- active agents/reviewers
- Human attention required
- aggregate token/cost metrics when available
- Goal elapsed time

### 6.8 Goal context

Goal context must be compact and durable.

A Goal-level Context Pack should contain:

- objective
- completion criteria
- constraints
- source-of-truth references
- current Task DAG summary
- important cross-Task decisions
- unresolved Goal blockers
- current next actions

Task-level details remain in Task Context Packs rather than being blindly duplicated into Goal context.

### 6.9 Native agent goal support

Goal correctness must not depend on any provider-specific goal feature.

If an Agent Adapter exposes a native goal capability, rururunx may delegate or synchronize suitable Goal instructions to it as an optimization.

For example, a Codex adapter may use native goal functionality where safely available.

However:

- rururunx remains the source of truth for Goal state
- native agent goal state must not replace the rururunx Goal
- another provider must be able to continue the Goal
- runtime restart must recover without depending on one provider session

### 6.10 Consultation to Goal promotion

A consultation may be promoted directly into a Goal:

```text
Consultation
    ↓
Goal
    ↓
Task DAG
    ↓
Parallel development
```

The user must be able to review or edit Goal objective/completion criteria before execution when policy requires it.

## 7. Consultation mode

The user must be able to start repository-aware consultation with any configured interactive agent:

```bash
rrx consult
rrx consult --agent claude
rrx consult --agent codex
rrx consult --agent grok
```

Consultation does not automatically require an Issue, worktree, PR, or full review workflow.

A consultation can be promoted into a development Task, reusing context to draft an Issue and requirements. Consultation output must not silently become final requirements when a formal requirements phase is required.

## 8. Workflow classes

### QUICK

For trivial or clearly local changes.

Typical path:

```text
Worktree → Implement → Relevant Test → Review → Commit → PR
```

### STANDARD

For normal features, bug fixes, refactoring, and multi-file changes.

```text
Issue → Worktree → Requirements → Design → Implement
→ Impact Analysis → Tests → Review → PR → Merge → Cleanup
```

### STRICT

For authentication, authorization, DB schema, payment, secrets, infrastructure, deployment, public API breaking changes, security boundaries, and configured broad-impact changes.

STRICT adds security review, broader regression coverage, adversarial review, and staging/browser verification where applicable.

## 9. Risk classes

At minimum:

- R0 Trivial
- R1 Local
- R2 Cross-module
- R3 Security / Data / Infrastructure

Default mapping is configurable.

The runtime must support dynamic escalation such as QUICK → STANDARD or STANDARD → STRICT when new risk or broader impact is discovered. Automatic workflow downgrade is not allowed.

Verification depth must remain proportional to the effective workflow class. STRICT-only gates such as security review, broad mutation/regression, staging, or headed browser checks must not become universal requirements for QUICK/STANDARD work unless project policy or discovered risk escalates the Task. rururunx development/dogfood should exercise this distinction rather than treating every change as maximum-strictness work.

## 10. GitHub Issue

When required by workflow, rururunx must support Issue creation containing:

- title
- background
- purpose
- acceptance criteria
- non-scope

## 11. Worktree and branch management

Each development Task must use an isolated worktree unless an explicit project exception applies.

Default convention:

```text
worktree/issue-<number>
```

Requirements:

- create worktree and branch
- track Issue ↔ branch ↔ worktree
- detect dirty state
- prevent development directly on main/master
- remove worktree and merged local branch after completion
- respect project branch naming rules

## 12. Requirements phase

When required, create:

```text
doc/requirements/issue-<number>-requirements.md
```

Minimum contents:

- purpose
- scope
- non-scope
- acceptance criteria
- constraints

The authoring agent is configurable.

## 13. Design phase

Issue-level design must support:

- architecture
- API changes
- DB changes
- test strategy
- impact analysis
- project-required sections

### Two-layer design

Support both:

1. issue design as decision/history
2. master design as current system truth

Changes to current architecture should update the relevant master design in the same PR.

## 14. Impact analysis

Impact analysis is part of the MVP.

The workflow must be able to inspect consumers of:

- changed symbols
- DB columns/schema
- APIs
- shared modules
- constants
- thresholds
- environment variables
- input data sets/lists
- paths
- identifier shape/length

It must consider both consumers of the defect/behavior and consumers of the mechanism used to fix it.

## 15. Implementation and commit discipline

The executor implements against the approved design.

If implementation materially diverges from design, the workflow must support updating design before completion.

The runtime must support project policies requiring:

- coherent commits
- clean worktree before review/test/mutation
- staged-diff/stat verification before commit
- no uncommitted edits carried into the next review round

## 16. Testing

The MVP must orchestrate project-configured:

- unit tests
- integration tests
- E2E tests
- regression tests
- type checks
- lint
- build

Regression verification must include affected features discovered through impact analysis.

## 17. Mutation verification

The MVP must support mutation-style verification when required:

1. add/modify a test
2. intentionally break the protected behavior or wiring
3. verify the test fails
4. restore implementation
5. verify clean state and passing test

## 18. Browser verification

For UI/browser-facing changes, the workflow must support headed-browser verification and preservation of configured evidence such as screenshots.

Passing automated tests alone is insufficient when project policy requires headed verification.

MVP correctness is **browser-backend neutral**. At least one deterministic backend must be able to perform required headed verification and capture evidence without requiring an LLM. Playwright is an acceptable implementation.

Adaptive/agentic browser backends such as Stagehand are **MVP Optional / Stretch**. They may improve resilience or semantic exploration, but MVP acceptance, #12 verification workflow, and #16 dogfood must not depend on Stagehand, Jev, or any other adaptive browser backend.

## 19. Staging verification

The workflow must support staging verification for configured classes such as:

- API response changes
- authentication/session changes
- DB migrations
- environment variables/secrets
- infrastructure/deployment

The workflow should be able to verify that staging corresponds to the expected commit/build.

## 20. Review orchestration

### Reviewer count is configurable

A review phase must support **1, 2, 3, or more reviewers**.

Examples:

```yaml
review:
  reviewers: [claude, codex]
  completion: all
```

```yaml
review:
  reviewers: [claude, codex, grok]
  completion: quorum
  quorum: 2
```

```yaml
review:
  reviewers: [claude, codex, grok]
  completion: all
```

Supported completion policies must include at least:

- `all` — every configured reviewer must complete without an unresolved blocking result
- `quorum` — configurable N-of-M reviewers
- `any` — at least one reviewer is sufficient, for lightweight workflows

A **two-reviewer setup is a first-class configuration**, not a workaround.

The existing Claude + Codex + Grok Triple Adversarial Review is provided as a preset, not hard-coded into the runtime.

### Parallel review

Configured reviewers should run concurrently whenever their review inputs are independent.

### Review phases

The workflow must support review after:

- requirements
- design
- implementation
- security-relevant work

Reviewer model and reasoning-effort configuration must be explicit and configurable.

## 21. Review integrity

Before launching review, rururunx must support checks for:

- clean worktree
- expected HEAD/revision
- configured reviewer identity/model
- configured reasoning level
- required review instructions/skills

Existing project guards and hooks remain authoritative.

## 22. Review finding verification

AI review findings are not automatically facts.

The workflow must support:

```text
Finding → Verify in repository → Verified defect / False positive / Human judgment
```

For class-wide defects, the workflow should enumerate all matching locations and record:

```text
N locations inspected / M locations changed
```

## 23. Multi-round review

The runtime must support repeated:

```text
Review → Verify → Fix → Commit → Re-review
```

## 24. Review-time worktree lock

When a review requires immutable HEAD, the reviewed worktree must be write-locked against the executor.

Parallel fixing requires a separate worktree.

## 25. Security review

The runtime must support project-defined security review for areas including:

- authentication/session
- authorization
- input handling
- SQL/HTML output
- files/paths
- external commands/URLs
- APIs
- payment/state transitions
- logs
- dependencies
- deployment configuration

Existing project security skills/rules must be usable by reviewers.

## 26. Pull Request and merge

The runtime must support PR creation with project-required content, including by default:

- Summary
- Test plan
- regression result
- impact-analysis result
- Issue linkage

Automatic merge is allowed only when required gates pass.

Human escalation must be supported for conditions such as:

- unresolved Critical/High finding
- incomplete/failed security review
- required test failure
- draft PR
- non-main target
- production-impacting change
- explicit human-merge policy

## 27. Git and command safety

The MVP must integrate with or enforce project policies such as:

- no direct push to main/master
- no force push to main/master
- restricted force push elsewhere
- protected paths/secrets
- staged diff/file-count verification
- configurable command allow/review/human/deny policy

rururunx must never transform/wrap an operation solely to evade an existing permission boundary.

## 28. Approval Broker

Approval Broker is an MVP core component.

Default pipeline:

```text
Executor request
→ Deterministic Policy
→ Cross-Agent Reviewer
→ Optional Additional Reviewer(s)
→ Human
```

Deterministic policy should handle actions that do not need semantic judgment.

Cross-agent approval must support arbitrary configured mappings and must be able to forbid self-review.

Reviewer decisions are normalized as:

- `APPROVE`
- `DENY`
- `ESCALATE`

with reason, confidence, risk, and relevant evidence/context.

A reviewer must not execute the requested action.

## 29. Human escalation

Escalate when:

- policy requires human judgment
- reviewer returns ESCALATE
- confidence is below threshold
- reviewers disagree under the configured completion policy
- action is production/security critical
- agent enters unknown/failure state
- decision cannot be safely classified

## 30. Agent adapters

### MVP MUST

- Claude Code
- Codex
- Grok

### MVP SHOULD

- Gemini
- generic CLI-agent adapter
- local/OpenAI-compatible reviewer

Adapter interfaces must be extensible by third parties.

## 31. Parallel development

MVP must support at least 4 simultaneous Tasks in independent worktrees.

Tasks are not bound conceptually to terminal windows. The unit is:

```text
Task → Worktree → Agent Session → Adapter
```

## 32. Multi-project Runtime

Multi-project orchestration is an MVP requirement.

One `rrx` Runtime must manage multiple registered Projects simultaneously:

```text
rururunx Runtime
  ├─ Project A
  │    ├─ Goal A1
  │    │    └─ Task DAG
  │    └─ Goal A2
  ├─ Project B
  │    └─ Goal B1
  │         └─ Task DAG
  └─ Project C
       └─ Goal C1
            └─ Task DAG
```

### 32.1 Project registry

The Runtime must maintain a persistent registry of Projects.

MVP CLI should support at least:

```bash
rrx project add <path>
rrx project list
rrx project status [project]
rrx project remove <project>
```

Starting `rrx` inside an unregistered Git repository may offer or perform project registration according to policy.

### 32.2 Project isolation

Each Project must isolate:

- repository/worktree paths
- project configuration
- project rules/skills
- Goal/Task Context Packs
- Review Bundles
- credentials/environment references
- Git operations
- audit events

Context from another Project must never be injected into an agent session unless an explicit cross-project reference exists.

### 32.3 Global scheduler

Concurrency is governed globally as well as per Project and per Agent.

Configuration must support concepts equivalent to:

```yaml
scheduler:
  global_max_sessions: 12
  max_tasks_per_project: 4

agents:
  claude:
    max_concurrent: 4
  codex:
    max_concurrent: 6
```

The Scheduler must enforce:

- global session limits
- per-project Task/session limits
- per-agent/provider limits where configured
- Goal Task-DAG readiness
- reviewer concurrency
- fair progress across Projects so one large Goal does not unintentionally starve every other Project

### 32.4 Multi-project status and TUI

The global TUI/status view must group activity by Project and Goal.

It must expose at least:

- Project
- Goal
- Task
- Agent
- phase/state
- reviewer state
- Human attention
- elapsed/wait time
- token/cost metrics when available

`rrx status --all` or an equivalent command must provide a Runtime-wide view.

### 32.5 Multi-project recovery

After Runtime restart, rururunx must restore:

- Project registry
- active Goals per Project
- Task DAG state
- active/recoverable sessions
- worktree state
- pending approval/review state
- global/per-project concurrency accounting

### 32.6 Multi-project dogfood

MVP dogfooding must demonstrate at least two separate repositories/Projects progressing concurrently, with multiple Tasks overall and no cross-project context/worktree contamination.

## 33. Scheduler and states

The scheduler manages running, idle, waiting, review, human-waiting, failure, resume, and concurrency.

Minimum Task states:

- CREATED
- CONSULTING
- PLANNING
- IMPLEMENTING
- TESTING
- WAITING_APPROVAL
- WAITING_REVIEW
- REVIEWING
- FIXING
- WAITING_HUMAN
- READY_FOR_PR
- PR_CREATED
- MERGED
- FAILED
- COMPLETED
- CANCELLED

## 34. CLI and TUI

Primary commands:

```bash
rrx
rrx project add
rrx project list
rrx project status
rrx goal
rrx goal status
rrx consult
rrx run
rrx status
rrx attach
rrx agents
rrx review
rrx approve
rrx stop
rrx resume
rrx logs
```

Running `rrx` without a subcommand should open a TUI.

The TUI must expose active tasks, agents, phases, waiting states, reviewer status, and human-attention items.

`rrx attach <task>` must attach to the native agent session rather than replacing it with a proprietary chat UI.

## 35. Restart and recovery

Runtime restart must not lose Task state.

Persist at least:

- goal
- Goal completion criteria / Task DAG / Goal state
- task
- agent/session reference
- worktree
- branch
- workflow
- state
- review state
- pending approvals
- audit trail needed for recovery

## 36. Master design review

The runtime must support periodic master-design review triggered by configurable thresholds such as:

- quarter boundary
- major release
- number of new master-design files
- accumulated line-change threshold

## 37. Audit and observability

Record:

- state transitions
- executor/reviewer identity
- approval request
- policy result
- reviewer decision
- confidence/risk
- human decision
- relevant command/action
- timestamps

Measure at least:

- Goal completion time
- Goal Task throughput / blocked time
- task completion time
- execution time
- review time
- wait time
- human interruptions
- human attention time
- auto approvals
- cross-agent approvals
- escalations
- retries/failures

Primary KPI:

> **Human Interruptions per Task**

Secondary KPI:

> **Human Attention Time**

## 38. Local-first

MVP must operate without a hosted control plane.

The MVP core runtime is implemented in **Rust**.

Local state may use SQLite or another lightweight local store.

The Rust implementation should target low startup latency, low idle overhead, efficient async process supervision, reliable PTY/process handling, and single-binary distribution where practical.

## 39. MVP non-goals

Do not require for MVP:

- a new coding agent
- a new LLM
- desktop application
- web dashboard or remote companion PWA
- `rrx serve` / iOS remote status access
- hosted SaaS requirement
- proprietary IDE
- proprietary cloud VM layer
- long-term vector-memory platform
- proprietary RAG stack

These may be explored as explicitly Optional/Post-MVP work, but they must not gate MVP completion.

## 40. Context and token efficiency

Context/token efficiency is a **core MVP requirement**, not a post-MVP optimization.

Parallel Tasks and 1/2/3+ reviewer workflows can otherwise multiply identical repository, design, rule, and conversation context. rururunx must reduce redundant context while preserving correctness, safety, and independent review.

### 40.1 Repository Map / Context Index

Issue 18's initial implementation is a Task-scoped local Rust library with a CLI
inspection example, lexical graphs, deterministic budgeted selection/expansion,
mandatory-context budget failures and content-based freshness. Its byte-derived
token estimates are explicitly separate from measured provider usage. Full
workflow/Context Pack integration remains pending; implementation requirements
and bounds are in [Issue 18 requirements](issue-18-requirements.md).

The runtime must maintain a compact local representation of the repository sufficient to select relevant context without injecting the full repository into every agent call.

The index should support, where practical:

- file paths
- important symbols/signatures
- imports/dependencies
- callers/references
- changed-file/symbol relationships

Only a budgeted, task-relevant slice is injected by default.

### 40.2 Versioned Task Context Pack

Each Task must maintain a versioned Context Pack containing durable working context such as:

- purpose and acceptance criteria
- workflow/risk class
- relevant project constraints
- architecture/design references
- relevant files/symbols
- current revision/diff summary
- impact-analysis summary
- test/verification evidence
- unresolved findings
- blockers and next action

The Context Pack is a compact coordination artifact, not a substitute for source code or authoritative requirements/design documents.

### 40.3 Progressive disclosure

Optional rules, skills, design sections, and repository detail should be loaded only when relevant.

Mandatory safety rules and project-required constraints must not be omitted to save tokens.

Agents must be able to request context expansion when selected context is insufficient.

### 40.4 Conversation condensation

Long-running consultation/execution sessions must support auditable condensation/checkpointing.

A checkpoint must preserve at least:

- current goal
- confirmed decisions
- completed work
- current worktree/revision
- important files/symbols
- tests/commands already executed and results
- unresolved findings/errors
- next action
- mandatory safety/project constraints

Recent context may remain verbatim for a configurable window.

### 40.5 Deterministic Review Bundle

Reviewers must receive a deterministic factual Review Bundle rather than the executor's full conversation transcript.

A bundle may include:

- immutable target revision
- required requirements/design artifacts
- relevant project rules
- relevant repository-map slice
- diff/changed files
- impact-analysis artifact
- test/verification results
- review instructions

Reviewers in the same independent review round should receive equivalent factual inputs unless specialization explicitly requires extra material.

One reviewer's conclusions must not be injected into another independent reviewer before that reviewer completes.

### 40.6 Delta re-review

Review round N > 1 should prefer:

- previous reviewed revision
- new revision
- revision delta
- prior verified unresolved findings
- claimed fixes
- new verification evidence

instead of replaying all transient context from previous rounds.

A reviewer may request broader context.

### 40.7 Provider cache awareness

Adapters should expose and use native prompt/context caching where the provider/agent supports it.

rururunx should keep stable prompt components stable when it controls prompt construction, but correctness must never depend on a cache hit.

Caching and logical context minimization are distinct:

- caching reduces repeated provider processing/cost/latency
- context selection/condensation reduces the amount of logical context sent

### 40.8 Token budgets

The runtime must allow configurable context budgets at least for:

- repository-map injection
- review bundle
- recent uncondensed history
- optional supporting artifacts

Budget exhaustion must trigger context prioritization or explicit expansion behavior, not silent truncation of mandatory material.

### 40.9 Token/cost telemetry

When exposed by an adapter/provider, record:

- input tokens
- cached input tokens
- output tokens
- estimated cost
- context size
- Context Pack size/version
- repository-map injected size
- condensation events
- context expansion events

Metrics must be attributable by Task, phase, review round, and agent.

### 40.10 Optimization KPI

In addition to Human Interruptions per Task, rururunx should report:

- Input Tokens per Task
- Input Tokens per Review Round
- Cached Token Ratio where available
- Estimated Cost per Task
- Token amplification from reviewer count
- Context-efficiency reduction versus disabled/baseline mode

Dogfooding must compare a representative workflow with Context Efficiency enabled and disabled, and report quality/safety regressions as well as savings.

See `doc/design/master/context-efficiency.md`.

## 41. MVP acceptance criteria

The MVP is accepted when all of the following are demonstrable:

1. 4+ independent tasks run concurrently in separate worktrees.
2. Claude, Codex, and Grok can act as executors/reviewers.
3. `rrx consult` supports repository-aware consultation.
4. Consultation can be promoted into Issue/requirements/task context.
5. QUICK, STANDARD, and STRICT workflows execute.
6. Workflow can escalate on newly discovered risk.
7. Requirements and design artifacts can be generated.
8. Issue design and master design can be maintained together.
9. Impact analysis is integrated.
10. Unit/integration/E2E/regression/type/lint/build commands can be orchestrated.
11. Mutation verification is supported.
12. Provider-neutral headed browser verification is supported through at least one deterministic backend; adaptive Stagehand/Jev support is not required for MVP.
13. Relevant staging verification is supported.
14. Review phases support configurable 1/2/3+ reviewers.
15. Two reviewers can be selected and run in parallel.
16. Triple Claude/Codex/Grok review is available as a preset.
17. N-of-M/quorum review completion is supported.
18. Multiple review/fix/re-review rounds are supported.
19. Review-time worktree lock works.
20. Review findings are verified before fixing.
21. Cross-agent permission approval works.
22. Unresolved approvals escalate to humans.
23. Existing Git/command safety controls are not bypassed.
24. PR/merge/cleanup can be orchestrated.
25. Existing project hooks/skills/rules can be invoked.
26. Runtime restart can recover Tasks.
27. Approval/review decisions are auditable.
28. Human interruption metrics are available.
29. The user can manage the current multi-terminal workflow from one `rrx` runtime.
30. A compact repository map/context index can be produced and selected under a configurable token budget.
31. A versioned Task Context Pack is reused across phases without substituting for authoritative source/design artifacts.
32. Long-running sessions can be condensed/checkpointed while preserving required task state and safety constraints.
33. Independent reviewers use deterministic Review Bundles rather than another agent's full chat history.
34. Re-review can use revision deltas plus unresolved findings instead of replaying all transient history.
35. Provider cache usage/token telemetry is recorded when exposed.
36. Dogfooding reports token/cost/time differences with Context Efficiency enabled versus disabled.
37. `rrx goal` can create and persist a Goal with explicit completion criteria.
38. A Goal can manage a dependency-aware Task DAG and run ready independent Tasks in parallel.
39. Goal execution continues to the next runnable Task without requiring a new user prompt after every Task.
40. Goal may add clearly justified follow-up Tasks while preserving constraints and auditing scope changes.
41. Goal state, DAG, progress, and completion criteria recover after runtime restart.
42. Goal can be paused, resumed, cancelled, attached, and inspected from CLI/TUI.
43. Goal context is represented compactly without duplicating all Task histories.
44. Provider-native goal support is optional optimization; Goal execution remains provider-independent.
45. The core `rrx` runtime is implemented in Rust and demonstrates acceptable orchestration overhead under 4+ concurrent Tasks.
46. At least two separate Projects/repositories can be registered and active concurrently in one Runtime.
47. Project rules, context, worktrees, Git operations, and environment references remain isolated across Projects.
48. Multiple Goals from different Projects can make progress concurrently.
49. Global, per-project, and per-agent concurrency limits are enforced.
50. Runtime scheduling provides basic fairness so one Project does not unintentionally starve all others.
51. Runtime-wide status/TUI groups activity by Project → Goal → Task.
52. Runtime restart restores Project registry and multi-project Goal/Task scheduling state.
53. Dogfooding demonstrates at least two repositories progressing concurrently with no cross-project context/worktree contamination.
