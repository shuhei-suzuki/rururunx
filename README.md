# rururunx

**rururunx** is a high-performance, local-first, CLI-first workflow runtime for coding agents. The MVP core runtime is written in **Rust**.

It does not replace Claude Code, Codex, Grok, Gemini, or open-source coding agents. It keeps them running in parallel, coordinates reviews and approvals, isolates work with Git worktrees, and escalates to a human only when needed.

CLI command:

```bash
rrx
```

Long-running objectives can be launched as persistent Goals:

```bash
rrx goal "Complete the project MVP"
rrx goal status
```

## Why

Coding agents can already complete substantial development work autonomously, but parallel use still leaves a human acting as the supervisor:

- watching multiple terminals
- answering permission prompts
- starting reviewers
- waiting for review rounds
- preventing concurrent edits to a reviewed worktree
- checking tests, CI, PRs, merges, and cleanup
- deciding which workflow is appropriate for a change

rururunx automates that supervision layer while keeping native agent CLIs and existing project rules intact.

## Core principles

1. **Do not build another coding agent.** Use native agents as executors and reviewers.
2. **Stay thin.** Avoid unnecessary planner/supervisor LLM hops on the normal execution path.
3. **Parallel by default.** Run independent tasks in isolated worktrees.
4. **Cross-agent approval.** An agent's permission request can be reviewed by a different agent before escalating to a human.
5. **Human on escalation.** Humans handle important or unresolved decisions, not routine prompts.
6. **Respect existing safety controls.** Never bypass agent permissions, repository hooks, project rules, or Git safety policies.
7. **CLI first, local first.** A desktop app, web dashboard, or cloud control plane is not required.
8. **Spend context deliberately.** Reuse stable context, retrieve only relevant repository detail, condense long sessions, and measure token use.
9. **Goal-oriented autonomy.** Persist objectives above Tasks and keep moving through the dependency graph until explicit completion criteria are satisfied.
10. **Fast core runtime.** Use Rust to keep orchestration overhead and idle resource usage low.

## Intended workflow

```text
Consult (optional)
    ↓
Issue
    ↓
Worktree + Branch
    ↓
Requirements
    ↓
Design
    ↓
Implementation
    ↓
Impact Analysis
    ↓
Tests / Regression / Mutation Verification
    ↓
Parallel Adversarial Review
    ↓
Fix / Re-review rounds
    ↓
PR / CI
    ↓
Merge
    ↓
Cleanup
```

The runtime supports lighter workflows for small changes and stricter workflows for security-, data-, infrastructure-, or production-sensitive changes.

## Workflow classes

### QUICK

For trivial or local changes such as typos, copy changes, small CSS fixes, obvious local bugs, and small test changes.

Typical path:

```text
Worktree → Implement → Relevant Test → Review → Commit → PR
```

### STANDARD

For ordinary features, bug fixes, refactoring, and multi-file changes.

```text
Issue → Worktree → Requirements → Design → Implement
→ Impact Analysis → Tests → Review → PR
```

### STRICT

For authentication, authorization, DB schema, payment, secrets, infrastructure, deployment, public API breaking changes, and other security-boundary changes.

STRICT adds security review, broader regression coverage, required adversarial review, and staging/browser verification where applicable.

A workflow may be escalated during execution when new risk or broader impact is discovered. Automatic downgrade is not allowed.

## Cross-agent approval

```text
Executor
   ↓
Deterministic Policy
   ↓
Different Agent Reviewer
   ↓
Optional Second Reviewer
   ↓
Human
```

Examples:

```text
Claude → Codex
Codex  → Claude
Grok   → Gemini
Local agent → Claude
```

Review sets may use 1, 2, 3, or more agents with `all`, `quorum` (N-of-M), or `any` completion policies. Two-reviewer mode is a first-class configuration; Claude / Codex / Grok Triple Review is a preset, not a hard-coded requirement.

Reviewers return a decision such as:

```text
APPROVE
DENY
ESCALATE
```

with reason, confidence, and risk metadata. A reviewer does not execute the requested action itself.

## CLI direction

Planned commands include:

```bash
rrx
rrx goal "Complete the project MVP"
rrx goal --file goal.md
rrx goal status
rrx goal pause
rrx goal resume
rrx consult --agent claude
rrx run "#201" --agent codex
rrx status
rrx attach 201
rrx agents
rrx review
rrx approve
rrx stop 201
rrx resume 201
rrx logs 201
```

`rrx consult` launches a native agent for repository-aware discussion without forcing the full development workflow. A consultation can later be promoted into an Issue / Requirements / Task.

## MVP definition

The MVP is complete when the development workflow currently performed across multiple terminal sessions can be managed from one `rrx` session, including:

- first-class persistent Goals with explicit completion criteria and Task DAGs
- `rrx goal` continuous execution across Tasks
- 4+ parallel tasks in independent Git worktrees
- Claude Code, Codex, and Grok adapters
- consultation mode
- QUICK / STANDARD / STRICT workflows
- requirements and design generation
- impact analysis
- unit / integration / E2E / regression checks
- mutation verification where required
- headed browser verification for UI/browser-facing changes
- staging verification for relevant API/auth/DB/infra changes
- configurable parallel review with 1, 2, 3, or more reviewers (including 2-reviewer mode and Claude / Codex / Grok Triple Review)
- multi-round verify → fix → commit → re-review
- cross-agent permission approval
- human escalation
- review-time worktree locking
- PR / merge / cleanup orchestration
- Git and command safety integration
- master design maintenance and periodic master-design review
- restart / resume
- audit log and human-interruption metrics
- TUI and native session attach
- context/token efficiency: repository maps, Context Packs, progressive loading, condensation, delta re-review, cache awareness, and token telemetry

See [Product Requirements](doc/requirements/product-requirements.md), [Architecture](doc/design/master/architecture.md), [Goal Runtime](doc/design/master/goal-runtime.md), and [Context Efficiency](doc/design/master/context-efficiency.md).

## Status

Early design / bootstrap phase.

## License

TBD.
