# rururunx

**rururunx** is a high-performance, local-first, CLI-first workflow runtime for coding agents. The MVP core runtime is written in **Rust**.

It does not replace Claude Code, Codex, Grok, Gemini, or open-source coding agents. It keeps them running in parallel, coordinates reviews and approvals, isolates work with Git worktrees, and escalates to a human only when needed.

CLI command:

```bash
rrx
```

Long-running objectives can be launched as persistent Goals:

```bash
rrx project add ~/src/project-a
rrx project add ~/src/project-b
rrx goal "Complete the project MVP"
rrx goal status
rrx status --all
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

rururunx automates that supervision layer while keeping native agent CLIs and existing project rules intact. One runtime can supervise multiple repositories/projects at the same time.

## Core principles

1. **Do not build another coding agent.** Use native agents as executors and reviewers.
2. **Stay thin.** Avoid unnecessary planner/supervisor LLM hops on the normal execution path.
3. **Parallel by default.** Run independent tasks in isolated worktrees.
4. **Cross-agent approval.** An agent's permission request can be reviewed by a different agent before escalating to a human.
5. **Human on escalation.** Humans handle important or unresolved decisions, not routine prompts.
6. **Respect existing safety controls.** Never bypass agent permissions, repository hooks, project rules, or Git safety policies.
7. **CLI first, local first.** A desktop app, web dashboard, or cloud control plane is not required.
8. **Spend context deliberately.** Reuse stable context, retrieve only relevant repository detail, condense long sessions, and measure token use.
9. **Multi-project by default.** Run multiple repositories, Goals, and Tasks concurrently without mixing their context or rules.
10. **Goal-oriented autonomy.** Persist objectives above Tasks and keep moving through the dependency graph until explicit completion criteria are satisfied.
11. **Fast core runtime.** Use Rust to keep orchestration overhead and idle resource usage low.

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
Worktree → Implement → Commit → Relevant Test → Review → PR
```

### STANDARD

For ordinary features, bug fixes, refactoring, and multi-file changes.

```text
Issue → Worktree → Requirements → Commit Requirements → Review Requirements
→ Design → Commit Design → Review Design → Implement
→ Impact Analysis → Commit → Tests → Review → PR
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

## MVP scope and priority

The release-blocking path is the supervision runtime itself:

```text
Claude / Codex / Grok adapters
  ↓
Workflow
  ↓
Review
  ↓
Cross-agent Approval
  ↓
Goal / Scheduler / Recovery
  ↓
Full dogfood
```

Context Efficiency, multi-project isolation, verification gates, PR/merge/cleanup, CLI/TUI and audit are part of MVP Core.

Adaptive Stagehand/Jev browser automation is **MVP Optional / Stretch**. MVP browser verification is backend-neutral and must have a deterministic headed path that does not require an LLM.

The iOS/iPadOS companion PWA, `rrx serve`, and Tailscale remote status experience are **Post-MVP** and must not block the core runtime.

## MVP definition

The MVP is complete when the development workflow currently performed across multiple terminal sessions can be managed from one `rrx` session, including:

- multi-project runtime with isolated project registry, Goals, context, rules, and worktrees
- global/per-project/per-agent concurrency control
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
- provider-neutral headed browser verification for UI/browser-facing changes through at least one deterministic backend
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

See [Product Requirements](doc/requirements/product-requirements.md), [Architecture](doc/design/master/architecture.md), [Goal Runtime](doc/design/master/goal-runtime.md), [Multi-Project Runtime](doc/design/master/multi-project-runtime.md), and [Context Efficiency](doc/design/master/context-efficiency.md).

## Status

Rust executable foundation; workflow components are being implemented incrementally.
The command examples above describe the MVP target. Currently implemented:
- Issue 18 adds the Task-scoped `RepositoryContext` Rust API for lexical repository
  maps, relevance ranking, budgeted selection, file/symbol/reference expansion,
  content-hash freshness and scoped audit evidence. Mandatory rules/evidence and
  requested expansions require sufficient budget; token values are estimates.
  Inspect existing Task bindings locally with:
  `cargo run --example repository-context -- --state STATE.db --task TASK_UUID map`
  (or `select "task text"`, `expand file src/lib.rs`; `--include` admits a specific
  ignored source, `--evidence` makes its full content mandatory). Workflow and
  durable Context Pack integration remain pending. See
  [Issue 18 design](doc/design/issue-18-design.md) for limits and lexical accuracy.
- Library Git/worktree management with project ownership checks, protected branches,
  dirty-state checks, durable logical review locks and safe merged cleanup. CLI/workflow
  integration and interrupted-operation reconciliation are pending.
- `rrx project add <path>`, `project list`, `project status [UUID/name]`, and
  `project remove <UUID/name>` persist an isolated multi-repository registry.
  Missing/moved or changed sources become BLOCKED; repeat validated add for
  recovery. Removal keeps source/worktree files and history and rejects active
  work. This registry foundation does not yet run concurrent Goals/sessions.
- `rrx --help`, `rrx --version`, and `rrx config-check`. Typed runtime entities and
  transactional SQLite state/audit persistence are available in the library.
- An extensible asynchronous Agent Adapter registry and task-scoped generic native
  process supervision. Generic adapters support noninteractive execution with
  prepared context and explicit environment. Unsupported attach/resume/review,
  interactive modes and native Goals fail explicitly; unavailable token/cache
  usage remains null. Provider adapters and execution/workflow CLI commands are
  still being implemented. See [adapter design](doc/design/issue-4-design.md).


## License

TBD.

## Development

Native Grok adapter work is specified in [Issue 7 requirements](doc/requirements/issue-7-requirements.md)
and [design](doc/design/issue-7-grok-adapter.md). Its native ACP file executor and
decision-only review are implemented through an explicitly registered native provider.
Installed Grok 1.0.46 acceptance covers owned file edits, fresh-input same-UUID resume
and schema-validated zero-tool decisions. Shell/PTY, ApprovalReviewer and restart recovery
remain unsupported. Independent review and final CI evidence are tracked in
[Issue 7 verification](doc/verification/issue-7.md).

Development of rururunx should dogfood QUICK / STANDARD / STRICT classification where project rules permit. Verification should be proportional to risk: safety-critical runtime/Git/state/process changes may require STRICT treatment, while small local/documentation changes should not inherit STRICT-only gates unless policy or discovered impact requires escalation.

Install [Rust with rustup](https://www.rust-lang.org/tools/install) and Git.
The Rust workflow library now drives QUICK/STANDARD/STRICT phases through adapters
and explicit evidence ports, commits each milestone before tests/review, persists
phase/context history atomically, and raises
workflow requirements when risk or scoped policy changes. Missing test/review/PR/
merge integrations wait for evidence; review exit zero alone cannot pass a gate.
Definitive rejection permits explicit remediation; waiting evidence can be reevaluated
without relaunching its Session. Actual gate results are journaled before postgate
checks; unknown outcomes keep recovery reservations. QUICK PR-created stays nonterminal
until requested merge/cleanup gates supply evidence. Cancellation preserves native
reservations until verified termination.
The generic CLI cannot review. Workflow CLI/TUI, independent review sets (#9),
approval routing (#10), owner/restart recovery (#14), and production Context Pack publication (#19)
remain pending. Issue #18 repository-context selection is available independently. See [Workflow Engine](doc/design/master/workflow-engine.md).

Issue #41's reviewed requirements specify passive observation of agent preparation
reservations and release only by their committing owner; implementation is pending.
Inactive-owner errors, dropped/crashed owners and CAS losses while publishing a
definitive decision or binding a dispatched Session remain recovery dependencies
of #14. Retaining a failed decision publication prevents turning an intended
definitive failure/invalidation into automatic retry. Existing explicit terminal
reservation recovery stays available under its terminal-Task/dispatch fences.
External GitHub/irreversible gate outcomes are reconciled by #13.

The repository pins Rust 1.91.1 (minimum supported Rust 1.91) with rustfmt/clippy.

```bash
cargo build --locked --workspace
cargo run --locked -p rrx -- --help
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked --release --workspace
cargo install --locked --path crates/rrx
```

MVP hosts: macOS and Linux. Cargo produces the local `rrx` binary; prebuilt release
packaging is not available yet. Native agents keep their own installation,
authentication, hooks and permission settings.

Configuration uses TOML. Inputs are explicit during bootstrap:

```bash
rrx --config runtime.toml --project-config project.toml config-check
```

Project overlays may change per-project task limits, context budgets and agent
model/effort for agents registered in the runtime config. They cannot add agents
or change global session limits or agent executable/concurrency
settings. A project minimum workflow can only make the runtime minimum stricter.
Unknown fields, unreadable explicit files and invalid limits fail with an error.
Configuration checking does not start agents or write state. Example:

```toml
minimum_workflow = "STANDARD"
[scheduler]
global_max_sessions = 12
max_tasks_per_project = 4
[context]
enabled = true
repo_map_tokens = 2000
review_context_tokens = 12000
recent_history_tokens = 8000
[agents.codex]
max_concurrent = 6
```

Measure warm help startup after the release build (Python is only a benchmark
utility, not part of the Rust runtime):

```bash
python3 scripts/measure-startup.py target/release/rrx --runs 30
```

This reports observed wall time and platform, not scheduler idle resource usage.
See [bootstrap decision](doc/design/issue-1-design.md).

Project commands share runtime-global SQLite state: `--state <file>` overrides
`RRX_STATE_PATH`, `$XDG_STATE_HOME/rururunx/state.sqlite3`, or
`$HOME/.local/state/rururunx/state.sqlite3`. The default never changes with CWD.
For registration, the supplied path must be an exact primary Git root with a
committed local base branch. `--base`, `--name`, `--max-tasks`, `--worktree-root`,
repeatable `--rule` and `--env-ref` set project metadata. `--project-config` for
`project add` and `--rule` paths resolve relative to the source root (absolute
owned paths also work) and must stay inside it. `--clear-project-config`,
`--clear-rules`, and `--clear-env-refs` explicitly remove stored references. Environment references are names only, never `NAME=value`; native
credential forwarding is pending. Status infers registered source/task CWD or
takes a unique name/UUID. `list --all` includes removed history; list/status
support `--json`. Display-name ambiguity requires a UUID.

Interrupted worktree creation or invalidated review locks can retain reservations;
explicit audited lock reconciliation CLI is pending in restart/recovery work.
Removal remains blocked until those reservations are reconciled.
