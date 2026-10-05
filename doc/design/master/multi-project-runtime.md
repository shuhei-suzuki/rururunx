# Multi-Project Runtime Design

Proposed native-execution policy: see [result protection](agent-execution.md).
It supersedes conflicting ownership/live-worktree prerequisites only when the
new profile is implemented; historical/current implementation sections below
remain baseline descriptions, not acceptance of that proposal.

**Status:** Draft
**Scope:** MVP multi-project orchestration

The independently reviewed [Runtime/Scheduler integration](../runtime-scheduler-integration-design.md)
composes Project/Goal driver fairness with actual native quota admission and
separates preparation Task capacity from native permits. Implementation and
qualification remain pending; the current effective global cap is6, and a requested
configuration of12 is not an established execution capability.

## 1. Goal

Allow one rururunx Runtime to supervise multiple repositories/projects, Goals, and Tasks concurrently without mixing project state, rules, context, worktrees, or credentials.

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

For MVP, a Goal belongs to exactly one Project.

## 2. Project entity

Suggested fields:

- project_id
- display_name
- canonical_root_path
- repository identity / remote URL when available
- default/base branch
- project configuration reference
- project rules/source-of-truth refs
- worktree root/convention
- concurrency policy
- created_at / updated_at

A Project identity is stable across Runtime restart.

## 3. Project registry

Persistently register projects.

CLI:

```bash
rrx project add <path>
rrx project list
rrx project status [project]
rrx project remove <project>
```

A project may also be inferred from the current Git repository and registered according to user/policy configuration.

Project IDs must not be derived solely from a mutable display name.

## 4. Isolation boundaries

The Project boundary isolates:

- repository root
- worktree paths
- Git operations
- project configuration
- project rules/skills
- Goals and Tasks
- Task/Goal Context Packs
- Review/Approval Bundles
- environment/credential references
- audit events and metrics

A Context Pack or Review Bundle must include its Project identity.

Cross-project context injection is forbidden by default.

## 5. Goal binding

Each MVP Goal has one project_id.

The Goal Runtime must load project rules and source-of-truth references through the Project registry.

A Project may have multiple concurrent Goals.

Cross-project Goals are outside MVP; cross-project concurrency is provided by the Runtime scheduler.

## 6. Worktree namespacing

Worktree management is project-scoped.

Two Projects may both have Issue #42 without colliding.

Internal identity therefore uses at least:

```text
project_id + task_id
```

rather than Issue number alone.

## 7. Global scheduler

Scheduling has multiple resource scopes:

1. Runtime-global
2. Project
3. Agent/provider
4. Goal/Task dependency readiness
5. reviewer/approval worker availability

Example configuration:

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

The scheduler must avoid accidental starvation of smaller Projects by one large Goal.

MVP fairness may be implemented with a simple bounded round-robin / weighted-ready-queue approach; sophisticated optimization is not required.

## 8. Global status/TUI

Runtime-wide status groups work by Project, then Goal.

Example:

```text
PROJECT       GOAL                  ACTIVE  HUMAN
repo-a        Complete auth             3      0
repo-b        Fix open issues           2      1
rururunx      Complete MVP              4      0
```

Drilldown exposes Tasks/reviewers/sessions.

## 9. Recovery

On restart:

1. load Project registry
2. validate canonical paths/repository identity
3. load active Goals per Project
4. reconcile Task/worktree/session state
5. restore global/per-project concurrency accounting
6. resume scheduling

Missing/moved Project roots become explicit BLOCKED state rather than being silently rebound to another directory.

## 10. Security

Project environment/credential references are project-scoped.

The Runtime must not leak:

- secrets/environment from Project A to Project B
- Project A rules/context into Project B agent prompts
- paths or Git operations across Project roots

## 11. Performance

The Rust Runtime maintains one lightweight global scheduler instead of one heavyweight supervisor per Project.

Inactive Projects should have near-zero active CPU usage.

## 12. MVP acceptance

- persistent Project registry
- canonical Project identity
- two or more Projects active concurrently
- project-scoped Goals and Tasks
- project-isolated worktrees/context/rules
- global and per-project concurrency limits
- per-agent concurrency limits
- basic fairness across Projects
- Runtime-wide grouped TUI/status
- restart recovery across Projects
- dogfood with at least two separate repositories

## 13. Implemented registry foundation (Issue 26)

`project::ProjectRegistry` persistently registers canonical exact primary Git
roots with UUIDs, base branch, scoped config/rule refs, environment names and
source-local worktree namespace. `rrx project add/list/status/remove` is available;
`--state` is runtime-global, defaulting to absolute RRX_STATE_PATH, XDG state or
home state independent of CWD. Help/version/config-check do not open state.

Reopening project commands validates registered paths, Git identity and scoped
references. Missing/moved/replaced identity or foreign refs persist BLOCKED with
a reason; explicit repeated add validates same-identity recovery. No silent
rebinding or automatic unblock occurs. Removal retains history/files and atomically
rejects nonterminal Goals/Tasks, unresolved sessions (including Lost) and locks;
new active writes require REGISTERED. Name ambiguity requires an explicit UUID.

Scoped input APIs revalidate identity/state and reject foreign or nested Git
source refs, symlink escapes and Git/task metadata paths. Environment values are
never loaded/stored; known Git/runtime/agent routing names are rejected. Native forwarding must also
apply provider-specific launch policy.
Context ownership and worktree Git checks use the existing Project-scoped Store
and WorktreeManager. Native forwarding, Goal loops, concurrency/fairness and
bundle generation remain dependent work. See `../issue-26-design.md` for contract
limitations and `../../requirements/issue-26-requirements.md` for acceptance.

Snapshot format version 3 migrates v1→v2→v3 atomically without changing SQL
ownership or audit history. v2 adds blocked_reason; v3 adds Task Workflow authority. Explicit clear-reference flags support recovery after file removal.
Latest-Store scoped APIs reject removed state; runtime integrations must perform
Git/filesystem preflight off shared Store locks and revalidate version/scope before
launch. Existing blocked work can record Lost/blockers without starting new work.
Interrupted active locks need the pending Issue 14 audited reconciliation flow.
