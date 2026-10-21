# rururunx Architecture

Proposed native-execution policy: see [result protection](agent-execution.md).
It supersedes conflicting ownership/live-worktree prerequisites only when the
new profile is implemented; historical/current implementation sections below
remain baseline descriptions, not acceptance of that proposal.

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
                    │ Project Mgr    │
                    └───────┬────────┘
                            │
                    ┌───────▼────────┐
                    │ Goal Manager   │
                    └───────┬────────┘
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

### Project Manager

Owns registered repository/workspace identity and project-scoped configuration.

Responsibilities:

- persistent Project registry
- canonical repository/workspace identity
- project rules/source-of-truth references
- project-scoped Goal/Task lookup
- worktree namespace
- project concurrency policy
- cross-project isolation guarantees

See `multi-project-runtime.md`.

### Goal Manager

Owns persistent long-running objectives above Tasks.

Responsibilities:

- Goal objective / completion criteria / constraints
- Goal state and audit history
- Task DAG and dependency readiness
- follow-up Task proposals
- continuous Goal loop
- Goal-level Context Pack
- Goal completion evaluation
- pause/resume/cancel/recovery
- provider-native goal references as optional adapter hints

See `goal-runtime.md`.

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

Recommended MVP implementation: **Rust** runtime with SQLite plus append-only logical events.

Rust is the fixed MVP implementation language for the core CLI/runtime. The implementation should favor low startup latency, low idle overhead, async process supervision, reliable PTY handling, and single-binary distribution where practical.

Persist:

- Goal snapshot / Task DAG / completion criteria
- Task snapshot
- workflow/state
- agent sessions
- worktree/branch
- review sets/rounds
- pending approvals
- decision/audit events
- recovery metadata

## 4. Goal and Task state model

Goal state is defined in `goal-runtime.md` and sits above Task state.

### Task state

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

The Runtime schedules across multiple Projects. A Goal exposes dependency-ready Tasks inside one Project, and the global Scheduler selects runnable Tasks across Projects.

The unit of parallel execution is a Task, not a terminal window.

Scheduler resource scopes include Runtime-global, Project, Agent/provider, Goal dependency readiness, and reviewer availability.

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

- Project registry/provider
- global scheduler/fairness policy
- Goal planner/evaluator policy
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

## 12. Implemented Rust foundation

Rust 2024 / minimum Rust 1.91 is the core standard; development and CI pin 1.91.1.
Rust CI checks pull requests and pushes to main with one Ubuntu job named `check`.
Code changes run fmt and locked all-targets workspace clippy with warnings denied.
Actions does not run tests, macOS or release builds; its independent successful
check covers fmt/clippy only.
Only nonempty PR merge diffs entirely under `doc/` skip those steps;
uncertain diffs and README changes run them. The job still reports success for
documentation-only PRs. All workflow actions use full 40-character SHA pins.
OS/architecture/toolchain/lockfile-keyed Cargo caches are restored for code
checks and saved only after successful main push clippy without an exact hit.
Superseded PR runs share a group per PR and are cancelled; main push runs use
unique run-ID groups and are not cancelled.

Before push, `bash scripts/local-ci.sh` runs from a clean macOS repository root,
except for `doc/`-only PRs, which change no code and retain a successful Actions
documentation check. `--host-only` does not meet the pre-push requirement.
The script records full HEAD, UTC date, OS and Rust/Cargo/Docker versions. Its
`main-contained` step obtains the exact ref's object once with
`git show-ref --verify --hash refs/remotes/origin/main` and peels it with
`git rev-parse --verify --end-of-options "<sha>^{commit}"`.
It records `origin/main <sha>` (the commit SHA) in the summary and checks ancestry
against the captured HEAD SHA. Branches/tags cannot substitute for a missing
remote-tracking ref. Non-ancestry or a comparison error fails; a missing ref or
either resolution failure is NOT RUN, failing the run, with errors in
`main-tip.log`. The final
local run before merge follows fetch and integration of current `origin/main`
by merge/rebase if needed, covering integrated code because Actions does not test
main pushes or PR merge results. Advancing main requires a new final local run.
Host steps are
fmt, locked all-targets workspace clippy with warnings denied, locked workspace
tests and a locked release workspace build. Both host and Linux workspace tests
use `--no-fail-fast`, so every test target runs and the record shows all failures.
This does not change PASS/FAIL or the exit status.
Host Cargo calls remove the four
GitHub token variables (`GITHUB_TOKEN`, `GH_TOKEN`, `GH_ENTERPRISE_TOKEN`,
`GITHUB_ENTERPRISE_TOKEN`). Host build scripts, proc macros and tests still run
with the user's privileges, credentials and network; removing four tokens is not
isolation. Only commits produced by this workflow or with reviewed diffs may run;
external PR code requires review or an isolated environment before local execution.

The host archives only tracked content from the captured HEAD SHA with
`git archive --format=tar <sha>`
into a fresh per-run log directory. Linux fmt/clippy/tests run in one
`rust:1.91.1` verification container with `--rm`, label `rururunx-local-ci=1`,
the host UID/GID, that tar mounted read-only, a writable per-run results mount,
fixed non-secret HOME/USER/LOGNAME, and no forwarded host environment. The
worktree and ignored `worktree/`, `.rrx/` and `target/` are not mounted. Before
extraction or any rustup/Cargo call, a parent flock takes the shared-volume lock
through `flock --exclusive --close /target/.local-ci.lock bash -c '...'`, holding
it until all Linux steps finish without passing its descriptor to the step shell
or checked processes. Open or lock failure exits before later Linux steps can
PASS. `docker run --rm --init` provides PID 1 signal forwarding and child reaping;
the step script's exit status becomes the container's exit status.
All in-container Cargo calls run
under this lock, serializing Linux verification and shared registry/Git/target
access across worktrees. Setup then extracts the tar with fresh mtimes into a
fresh tmpfs at fixed `/src`, invalidating source fingerprints from other worktrees
while allowing dependency artifact reuse. Overlapping runs of the same worktree
are unsupported. A Docker VM clock moving backwards between runs is an accepted
residual risk for mtime-based artifact reuse.
HOME, TMPDIR and CARGO_HOME use per-run tmpfs, so Cargo configuration cannot
persist. RUSTUP_HOME is not overridden. The image's `/usr/local/rustup` and toolchain
directory are world-writable (777), including for the verification UID. After
extraction, `linux-components` runs `rustup component add rustfmt clippy` in `/src`,
honoring `rust-toolchain.toml`, before fmt/clippy/tests and while holding the lock.
Every run installs these components into its container's own writable layer,
discarded by `--rm`; no toolchain changes or caches persist between runs. Each
run needs network for these components and for dependencies on cache misses.
The repo image digest, or image ID if absent, is recorded in metadata and summary.

Named volumes `rururunx-local-ci-registry`, `rururunx-local-ci-git` and
`rururunx-local-ci-target` are standing project caches for Cargo registry, Git
databases and Linux target only; registry/Git mount at `$CARGO_HOME/registry`
and `$CARGO_HOME/git`. A separate auto-removed helper mounts only those volumes,
uses per-run HOME/TMPDIR tmpfs and sets their root directory permissions for the
host UID without mounting host files or source. These caches are exempt from
generic task cleanup and trusted only for trusted commits. After running
anything untrusted, remove all three when unused with
`docker volume rm rururunx-local-ci-registry rururunx-local-ci-git rururunx-local-ci-target`.
All steps are attempted despite failures, except that lock open/acquisition
failure stops the Linux container before extraction or Cargo work.
Per-step status/seconds and HEAD are
printed and retained in `.rrx/local-ci/<sha>.txt`, with logs under
`.rrx/local-ci/<sha>/run.XXXXXX/`. Linux result files must be exactly one line
`PASS <digits>` or `FAIL <digits>` plus a single newline. Failed reads, missing
newlines or any extra fields/lines/bytes produce NOT RUN and fail the run.
Malicious code under test can forge any marker; the commit trust rule is required.
Repository and physical working-directory paths retain trailing newlines during
validation; commas, double quotes or any newlines are refused before file creation,
Cargo or Docker work to protect Docker mount CSV parsing.
Failures or missing checks fail the script. A run with any failed or unexecuted
step does not meet the pre-push verification requirement. Do not push while any
such failure remains.

`--host-only` reports Linux NOT RUN, including `linux-components` between setup
and fmt, and requires `--allow-partial` to permit an
otherwise successful partial exit. Full local commands, SHA, test counts and
outcomes are committed in `doc/verification/issue-<number>.md` and referenced
from the PR's Test plan; public PR and issue comments carry only IDs, titles,
severities and statuses. The record includes commands, SHAs, counts, outcomes,
the script's metadata (UTC date, OS, Rust/Cargo/Docker versions) and the image
digest;
no secrets, tokens, environment dumps, raw logs, real
resource IDs or absolute home paths (use repository-relative paths).
Follow-up commits changing only any files under `doc/verification/` keep the
recorded run valid for the verified SHA without a re-run; nothing in `crates/`,
`scripts/` or `.github/` reads that directory. Verify this with a successful
`git diff --name-only --no-renames <verified SHA> HEAD`, requiring every printed
line to start with `doc/verification/`. This recorded run counts as the final
pre-merge run only if, after a fresh fetch, `refs/remotes/origin/main` still equals
its recorded `origin/main <sha>`; otherwise integrate current main and re-run the
whole script on a clean HEAD. Partial runs do not meet the pre-push policy.

The Cargo workspace contains `crates/rrx` with a library and a CLI binary,
`crates/rrx-process-tracker` for cooperative discovery, and
`crates/rrx-spawn-hygiene` for consuming process launch boundaries.
Implemented library modules are `config`, `domain`, and `state`; CLI help and config validation
do not construct an async runtime or start native agents.

The compiled CLI also composes `rrx serve` as a foreground Runtime owner and
private Unix control service. It accepts real same-UID peers, emits exact current
instance/epoch Hello and retains bounded connection tasks through shutdown.
`rrx status [--all]`, `rrx goal status` and `rrx goal tasks` connect as clients;
Project routing precedes scoped Goal reads. These clients never create state,
start an owner, advance an epoch, acknowledge controls or remove stale discovery.
Absent/refused discovery directs the user to explicit `rrx serve`.

Status is explicitly incomplete Runtime metadata; each bounded Goal Task page
is an independent observation, not an aggregated complete snapshot. Native
dispatch remains unavailable and no output establishes effective execution
capacity. The CLI exposes inert inline/file objectives, explicit bounded TOML
plan acceptance and Goal pause/resume/cancel with mandatory expected version and
reason. Proposal and accepted control decisions remain distinct. CLI logs/attach,
complete operational Goal execution and TUI remain unfinished; source presence
alone is not operational acceptance.

Initial dependencies are clap, serde/serde_json, TOML and anyhow. Config loading
accepts explicit runtime and single-project inputs, validates typed limits and
does not act as a native permission grant. A separate ProjectOverlay schema
excludes runtime-wide session limits and agent executable/concurrency settings;
project workflow minimum can only escalate the runtime minimum. Project model/effort
overrides apply only to runtime-defined agents; unknown agent names are rejected. Diagnostics
retain the input file path. macOS/Linux source installation produces one release
`rrx` binary with Cargo; no hosted service is required.

## 13. Durable state implementation

The Rust domain uses distinct UUID types for Project/Goal/Task/Session/Record. Issue
numbers and PIDs are metadata, not global identity. Goal snapshots preserve explicit
criteria/DAG data; task workflow and process state are separate from phase.

SQLite (bundled via rusqlite) stores indexed ownership columns with JSON snapshots.
`PRAGMA user_version = 3` marks the current snapshot format and
`application_id = 0x52525831` identifies rrx databases. Tagged v1/v2 databases upgrade
atomically through ordered v2/v3 markers; Project blocked_reason defaults to None,
v3 adds authoritative Task Workflow records, and SQL
ownership/audit layout is unchanged. Foreign/unmarked v1 and nonempty v0 databases
are rejected. New databases initialize in one transaction; unsupported future
versions fail before migration. Each connection
enables foreign keys and a bounded busy timeout; WAL supports concurrent readers.
Composite foreign keys enforce Goal/Task ownership and scoped records/context/usage.
Project roots/identity and Goal/Task ownership cannot silently change on updates.
Goal DAG/edge/follow-up references are checked for exact Project/Goal Task ownership
before saving. Bounded structural validation rejects undeclared endpoints, self
edges, duplicate ordered pairs and hard cycles; advisory cycles are permitted.
TaskDag::hard_order returns deterministic hard ordering only. Managed readiness
and verified completion authority remain pending.
Snapshot revision checks reject stale writers.

Snapshot mutations and their scoped audit events commit in the same immediate
transaction. Audit sequences are monotonic and SQL triggers reject UPDATE/DELETE/REPLACE and
backdated sequence insertion. Public audit callers cannot emit Store-reserved kinds.
Review/approval evidence and session-state metadata are journaled so decisions are
not lost when snapshots change. This journal does not protect against an owner
who alters the database file or drops triggers.

Context versions are append-only and consecutive per Goal/Task under both Store
API and SQL triggers; source revision and
hashes persist. Usage records retain project/goal/task/session/phase/round/agent
attribution; missing metrics remain null with explicit unavailability reasons.
Native session references, PID hints and recovery metadata persist, but reopening
state does not establish process liveness. Project Git registration, DAG readiness,
workflow execution and process reconciliation are separate components.

Future migrations must be ordered/transactional, preserve identity/audit/context
and telemetry, and test real old-version fixtures. Persistence schema and snapshot
format evolve together; typed snapshots reject unknown fields instead of dropping
them during read/write. Free-form extension payloads retain their fields.

## 14. Scoped Git/worktree implementation

`git::WorktreeManager` creates independent task worktrees and attached branches,
checks exact canonical Project root/worktree top-level and common-dir, rejects
main/master/configured base, detached HEAD, mismatched bindings and symlink paths.
Supported namespaces are descendants of the Project root. Status includes ignored
files so cleanup cannot silently discard them. Existing hooks/config remain active;
Git arguments never request force/no-verify. Inherited repository-routing environment
variables are removed to enforce scoped CWD.

Task binding and create/cleanup intents persist before native side effects. Typed
WorktreeLock records reserve maintenance/review operations; SQLite immediate
transactions serialize active locks against Starting/Running/WaitingApproval/
WaitingHuman/Lost Executor reservations across connections. Assigned task path/branch
are immutable. Logical locks block runtime mutation; external changes are detected
by exact HEAD/clean verification. Cleanup requires owned clean merged worktrees,
uses ordinary Git removal/local branch deletion, and retains durable provenance.
Failures preserve intent/lock for explicit recovery; dependent runtime recovery
integration is not yet implemented. Git calls are synchronous local operations and
must run on a blocking worker when integrated with async scheduling.

Git boundary hardening additionally reserves unique Project-scoped task paths/branches
in the Store transaction, requires task-scoped executors at the exact bound path,
and rejects overlapping Project roots/namespaces. `git::repository_identity` supplies
canonical primary common-dir plus base root commits; Git operations reject linked
source roots or changed/replaced identity. Registry integration must use this helper.
Failed review acquisition releases its provisional lock and audits the failure.
Cleanup prechecks native branch deletion's upstream/root-HEAD predicate and currently
supports ancestry-preserving merges. External writers remain outside advisory lock
control; ignored files present at the safety check block cleanup.

Cleanup follows native HEAD fallback when a tracking ref has been pruned. Project
namespace changes are rejected after a Task has bound its worktree. Status avoids
optional Git index locks, and provisional release errors retain diagnostic lock IDs.
Cleanup retains Task bindings for audit provenance; naming reuse, source-history
validation cost, explicit existing-worktree adoption and independent state databases
must be addressed by the dependent Project registry/recovery policy where applicable.

Project registry foundation is implemented by `project::ProjectRegistry` (Issue
26), including runtime-global CLI state, identity validation, sticky BLOCKED
recovery, scoped input APIs and transactional soft removal. Native context/env
forwarding and global scheduling remain dependent integrations. See
`../issue-26-design.md`.
