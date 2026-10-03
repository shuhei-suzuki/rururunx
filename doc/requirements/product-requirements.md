# rururunx Product Requirements

**Version:** 0.3
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

## 4. MVP success definition

The MVP is complete when the development workflow currently performed manually across multiple terminal sessions can be operated from one `rrx` runtime.

The user must not need to patrol terminals merely to notice that an agent is waiting.

The MVP includes the current strict development practices, not only process multiplexing.

## 5. Core entities

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

### Approval Request

A normalized request containing action type, command/operation, executor, task, worktree, branch, risk context, policy result, reviewer decisions, and final decision.

## 6. Consultation mode

The user must be able to start repository-aware consultation with any configured interactive agent:

```bash
rrx consult
rrx consult --agent claude
rrx consult --agent codex
rrx consult --agent grok
```

Consultation does not automatically require an Issue, worktree, PR, or full review workflow.

A consultation can be promoted into a development Task, reusing context to draft an Issue and requirements. Consultation output must not silently become final requirements when a formal requirements phase is required.

## 7. Workflow classes

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

## 8. Risk classes

At minimum:

- R0 Trivial
- R1 Local
- R2 Cross-module
- R3 Security / Data / Infrastructure

Default mapping is configurable.

The runtime must support dynamic escalation such as QUICK → STANDARD or STANDARD → STRICT when new risk or broader impact is discovered. Automatic workflow downgrade is not allowed.

## 9. GitHub Issue

When required by workflow, rururunx must support Issue creation containing:

- title
- background
- purpose
- acceptance criteria
- non-scope

## 10. Worktree and branch management

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

## 11. Requirements phase

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

## 12. Design phase

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

## 13. Impact analysis

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

## 14. Implementation and commit discipline

The executor implements against the approved design.

If implementation materially diverges from design, the workflow must support updating design before completion.

The runtime must support project policies requiring:

- coherent commits
- clean worktree before review/test/mutation
- staged-diff/stat verification before commit
- no uncommitted edits carried into the next review round

## 15. Testing

The MVP must orchestrate project-configured:

- unit tests
- integration tests
- E2E tests
- regression tests
- type checks
- lint
- build

Regression verification must include affected features discovered through impact analysis.

## 16. Mutation verification

The MVP must support mutation-style verification when required:

1. add/modify a test
2. intentionally break the protected behavior or wiring
3. verify the test fails
4. restore implementation
5. verify clean state and passing test

## 17. Browser verification

For UI/browser-facing changes, the workflow must support headed-browser verification and preservation of configured evidence such as screenshots.

Passing automated tests alone is insufficient when project policy requires headed verification.

## 18. Staging verification

The workflow must support staging verification for configured classes such as:

- API response changes
- authentication/session changes
- DB migrations
- environment variables/secrets
- infrastructure/deployment

The workflow should be able to verify that staging corresponds to the expected commit/build.

## 19. Review orchestration

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

## 20. Review integrity

Before launching review, rururunx must support checks for:

- clean worktree
- expected HEAD/revision
- configured reviewer identity/model
- configured reasoning level
- required review instructions/skills

Existing project guards and hooks remain authoritative.

## 21. Review finding verification

AI review findings are not automatically facts.

The workflow must support:

```text
Finding → Verify in repository → Verified defect / False positive / Human judgment
```

For class-wide defects, the workflow should enumerate all matching locations and record:

```text
N locations inspected / M locations changed
```

## 22. Multi-round review

The runtime must support repeated:

```text
Review → Verify → Fix → Commit → Re-review
```

## 23. Review-time worktree lock

When a review requires immutable HEAD, the reviewed worktree must be write-locked against the executor.

Parallel fixing requires a separate worktree.

## 24. Security review

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

## 25. Pull Request and merge

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

## 26. Git and command safety

The MVP must integrate with or enforce project policies such as:

- no direct push to main/master
- no force push to main/master
- restricted force push elsewhere
- protected paths/secrets
- staged diff/file-count verification
- configurable command allow/review/human/deny policy

rururunx must never transform/wrap an operation solely to evade an existing permission boundary.

## 27. Approval Broker

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

## 28. Human escalation

Escalate when:

- policy requires human judgment
- reviewer returns ESCALATE
- confidence is below threshold
- reviewers disagree under the configured completion policy
- action is production/security critical
- agent enters unknown/failure state
- decision cannot be safely classified

## 29. Agent adapters

### MVP MUST

- Claude Code
- Codex
- Grok

### MVP SHOULD

- Gemini
- generic CLI-agent adapter
- local/OpenAI-compatible reviewer

Adapter interfaces must be extensible by third parties.

## 30. Parallel development

MVP must support at least 4 simultaneous Tasks in independent worktrees.

Tasks are not bound conceptually to terminal windows. The unit is:

```text
Task → Worktree → Agent Session → Adapter
```

## 31. Scheduler and states

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

## 32. CLI and TUI

Primary commands:

```bash
rrx
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

## 33. Restart and recovery

Runtime restart must not lose Task state.

Persist at least:

- task
- agent/session reference
- worktree
- branch
- workflow
- state
- review state
- pending approvals
- audit trail needed for recovery

## 34. Master design review

The runtime must support periodic master-design review triggered by configurable thresholds such as:

- quarter boundary
- major release
- number of new master-design files
- accumulated line-change threshold

## 35. Audit and observability

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

## 36. Local-first

MVP must operate without a hosted control plane.

Local state may use SQLite or another lightweight local store.

## 37. MVP non-goals

Do not build in MVP:

- a new coding agent
- a new LLM
- desktop application
- web dashboard
- hosted SaaS requirement
- proprietary IDE
- proprietary cloud VM layer
- long-term vector-memory platform
- proprietary RAG stack

## 38. MVP acceptance criteria

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
12. Headed browser verification is supported.
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
