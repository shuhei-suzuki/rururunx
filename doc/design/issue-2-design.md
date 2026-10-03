# Issue #2 — State and journal design

## Architecture / domain

Add `domain` and `state` library modules. Strongly typed UUID IDs distinguish
Project, Goal, Task and Session at compile time; IDs are independent of Issue
numbers, directory names and PIDs. Snapshot structs include extensible typed
phase/recovery/criteria/DAG fields. Phase is separate from Task state.

SQLite stores indexed identity/ownership columns plus versioned JSON snapshots.
Session/review/approval/agent configuration and later extension payloads use a
scoped Record envelope; Session payload identity is checked against the envelope.
Review and approval decision payloads are preserved in journal events on updates.
Versioned context has a dedicated immutable table.
Local scope is Runtime → Project → Goal → Task. Composite foreign keys enforce
ownership for Goals, Tasks and child orchestration records. Before saving a Goal,
all DAG node/edge/follow-up Task references are verified against its Project/Goal;
cycle/readiness evaluation remains in #23. A Goal's Project and
a Task's Project/Goal cannot be silently rebound by an update.

## Schema / migration

Use rusqlite with bundled SQLite. `application_id = 0x52525831` identifies rrx
databases. Foreign/unmarked v1 databases and nonempty v0 databases are rejected.
`PRAGMA user_version` is the transactional
schema version; v0 → v1 initializes tables/indexes/triggers. An unsupported future
version is rejected before migrations. Foreign keys and a bounded busy timeout
are enabled on every connection; WAL permits readers while state is written.
Future migrations are ordered, atomic, tested against real older fixtures and
preserve IDs/ownership/history. This pre-merge schema is not a shipped format;
unmarked databases from intermediate #2 commits are rejected and isolated test
fixtures are recreated rather than silently adopted. JSON snapshot versioning follows database
migration, not best-effort deserialization of unknown formats.

Each mutation and its `AuditEvent` are written in one immediate transaction.
Optimistic snapshot revisions reject stale writers. Audit rows have a monotonic
sequence, scoped identity, kind, timestamp and JSON evidence. SQLite triggers
reject audit UPDATE/DELETE/REPLACE and backdated sequence insertion. Public audit
callers cannot emit Store-reserved save/context/usage event kinds. Context SQL
triggers reject mutation/replacement and enforce consecutive owner versions. This is an application journal, not a tamper-proof
security log against a user who controls the database file.

## Context / telemetry / recovery

Context records are keyed by owning identity plus monotonically increasing
version. They preserve source revision, hashes and payload without implementing
selection/condensation yet. Usage rows retain task/session/phase/review-round
attribution and nullable input/output/cached/cost fields. Session recovery holds
native identity, process reference and reconnect metadata; reopening the store
does not assert liveness.

Review/Approval snapshots preserve each round/decision and never overwrite audit
history. Project-scoped agent configuration stores executable argv, capability
metadata and environment variable names; caller-provided secrets do not belong
in these records.

## Impact / verification

Consumers: every downstream runtime component. Tests exercise real temporary
SQLite databases, reopen/recovery, composite foreign keys, atomic rollback,
stale-update rejection, append-only triggers, nullable telemetry and future
schema rejection. Mutation verification removes ownership enforcement and audit
immutability in isolated worktrees to prove corresponding tests fail. No headed
browser/staging target applies to this local storage component.
