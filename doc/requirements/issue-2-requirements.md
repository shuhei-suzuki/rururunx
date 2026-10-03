# Issue #2 — Durable runtime state

Sources: Product Requirements v0.7 sections 5/35/37/40, Architecture,
Goal Runtime, Multi-Project Runtime and GitHub #2. Dependency: #1.
Workflow: STRICT (new SQLite schema and shared domain boundary).

## Purpose / scope

Persist Project, Goal, completion criteria/DAG metadata, Task, configured Agent,
native Session, workflow phase, review rounds, pending Approval, Context Pack
versions, per-session/phase usage and append-only audit events locally.

## Non-scope

Project Git/path registration validation (#26), DAG cycle/readiness/criterion
evaluation (#23), process recovery policy (#14), phase execution (#8) and native
permission handling (#10) remain separate. This store must preserve their data
without pretending that persisted process references are currently alive.

## Acceptance criteria

- Typed Project/Goal/Task snapshots support create/read/update and durable IDs.
- A reopened database preserves Goal criteria, DAG metadata and Task recovery data.
- Reviews/approvals, versioned context and nullable usage metrics round-trip.
- State mutations and scoped audit events commit atomically.
- Cross-project Goal/Task/session references are rejected even for the same Issue number.
- Audit history is queryable and cannot be updated/deleted through normal SQL.
- Schema migration/version policy is documented; unsupported future versions fail.
- Unit/integration/regression checks and relevant mutation checks pass.

## Constraints

SQLite is local and bundled for portable single-binary builds. Project identity
is mandatory on all scoped entities. Never store a secret value in agent config
or audit payloads; environment credential references are names only. Missing
token/cost telemetry is explicit absence, not a measured zero.
