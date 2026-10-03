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

See [Product Requirements](doc/requirements/product-requirements.md), [Architecture](doc/design/master/architecture.md), [Goal Runtime](doc/design/master/goal-runtime.md), [Multi-Project Runtime](doc/design/master/multi-project-runtime.md), and [Context Efficiency](doc/design/master/context-efficiency.md).

## Status

Rust executable foundation; workflow components are being implemented incrementally.
The command examples above describe the MVP target. Currently implemented:

- Library Git/worktree management with project ownership checks, protected branches,
  dirty-state checks, durable logical review locks and safe merged cleanup. CLI/workflow
  integration and interrupted-operation reconciliation are pending.
- `rrx project add <path>`, `project list`, `project status [UUID/name]`, and
  `project remove <UUID/name>` persist an isolated multi-repository registry.
  Missing/moved or changed sources become BLOCKED; repeat validated add for
  recovery. Removal keeps source/worktree files and history and rejects active
  work. This registry foundation does not yet run concurrent Goals/sessions.
- `rrx --help`, `rrx --version`, and `rrx config-check`. The library also provides
typed runtime entities and transactional SQLite state/audit persistence; native
execution and workflow commands are not yet implemented.

## License

TBD.

## Development

Install [Rust with rustup](https://www.rust-lang.org/tools/install) and Git.
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
`project add` and `--rule` paths are relative to the source root and must stay
inside it. Environment references are names only, never `NAME=value`; native
credential forwarding is pending. Status infers registered source/task CWD or
takes a unique name/UUID. `list --all` includes removed history; list/status
support `--json`. Display-name ambiguity requires a UUID.
