# Issue #4 — Adapter and generic supervisor design

## Contracts and ownership

`adapter::AgentAdapter` is an object-safe, asynchronous, Send/Sync Rust trait using
boxed futures. `AgentRegistry` accepts third-party implementations and can create
generic adapters from existing runtime Config command vectors. Native adapters
will replace generic registrations without provider conditionals in the workflow.
Configuration never probes authentication or launches an agent. `probe` resolves
an executable using the runtime's absolute PATH entries; it does not run it.

`LaunchRequest` contains a registered Project snapshot, exact Scope, worktree,
role/mode, prepared artifact and explicit environment. PreparedInput includes kind,
revision, version, source versions and already-selected payload; adapters do not
retrieve, condense, cross-project cache or replay executor conversations. Review
Bundle input is distinct from Context Pack; generic review is unsupported until a
native adapter can enforce a read-only mode. Review quorum belongs upstream.

Generic execute checks persisted Project/Goal/Task and collects native Git metadata
outside the shared Store mutex, with a five-second total deadline and bounded output.
The shared pure `git::validate_worktree_ownership` validates source identity, unique
Task binding, namespace, common directory/top-level and exact non-base branch.
Project/Goal/Task versions are checked again before spawn. Native status/fsmonitor
hooks are not needed for launch ownership; no unbounded Git runs under the Store lock.
Git environment overrides fail before preflight. No shell interpolation is added:
configured argv is passed directly; a user explicitly configuring a shell owns its
script. Model/effort are rejected by generic adapters rather than silently ignored.
Configured argv[0] is preserved even for executable symlinks. Paused/terminal Goal
and completed/PR-ready Task launch is rejected; DAG/resource eligibility remains
the scheduler responsibility and executor fallback must update Task assignment.

Generic adapters require a shared Store. Starting is persisted before async Git
preflight; confirmed cleanup after launch failure/cancellation records Failed.
Unconfirmed cleanup retains a Lost/Starting reservation, including cancellation
while a blocking cleanup worker still owns the process group. The Store transaction
atomically excludes active WorktreeLock and reserved/live executor Session for the
same Task, so a reviewer cannot start between preflight and spawn. Running and
terminal snapshots are persisted through existing optimistic Store/audit APIs with the last written version; concurrent
Session updates fail explicitly instead of being overwritten.
The original SessionId survives PID/native-reference changes and SQLite reopen.
Lost supervisors are explicit: old snapshots/PIDs are insufficient to reconnect.

## Async process boundary

Tokio process/sync/io/time runs native processes and event-driven supervision.
Piped noninteractive stdin is written once, stdout/stderr drained concurrently to
64 KiB tails, and exit/stop waits through select rather than polling. Subscribers
receive latest SessionStatus via watch channels; this is status observation, not
a lossless native event journal. Terminal state carries exit code, I/O failure,
captured byte tails and explicit truncation flags. At most 32 terminal Sessions
are retained per adapter; release collects output sooner. Evicted/released sessions
remain durable and status explicitly returns SessionLost. Active concurrency is
governed by the scheduler. Raw output is memory-only and
may contain provider text; it is not written into Session/audit by this adapter.

macOS/Linux processes have a private process group. Stop kills the owned group,
waits/reaps its direct child and persists Stopped; dropping an adapter closes the
control channel and performs the same cleanup. Natural exit is observed through safe rustix waitid with NOWAIT, driven by
Tokio SIGCHLD. The unreaped leader reserves its PID until the owned group is
killed; only then is the direct child reaped and its terminal state persisted.
A process-group Drop guard also kills descendants during Tokio shutdown or
post-spawn failure. Deliberate setsid/setpgid escapes require future stronger OS
containment. Runtime SIGKILL/crash cannot prove process death: persisted Running
or Lost remains reserved until recovery verifies death or a human resolves it. Stopping a finished
Session is idempotent and cannot signal a reused PID. PTY/interactive attach is a
separate native capability, never approximated by these pipes.

`env_clear` prevents accidental project leakage. Runtime callers must intentionally
provide native baseline HOME/PATH/config/auth and project environment. Nothing
creates temporary empty native configuration, disables hooks or supplies unsafe
permission flags. Provider-specific safe environment/launch semantics belong in
#5/#6/#7. Generic capabilities are execute/non_interactive only; provider token/cache fields
are null with a reason, while prepared payload size is reported in bytes. Native
Goal references carry exact Scope and optional owning SessionId; native implementations
must validate persisted ownership and checkpoint Scope before acting. Native Goals, checkpoints and approval submission default
to explicit unsupported failures and never control rururunx Goal truth.

## Impact and limitations

Adds adapter library and Tokio/rustix dependencies without changing durable entity
shapes, SQLite schema, existing project overlays or CLI behavior. Registry/config,
Session Store, future scheduler/review/approval/CLI/recovery and native adapters
are consumers. Tests use native subprocesses, temporary Git fixtures and Store.
No browser/staging deployment target exists for this library boundary.

On macOS, XNU excludes zombie members from group signalling and may return EPERM
for a zombie-only group. The adapter accepts that result only after /bin/ps confirms
no live member of the still-reserved PGID; ordinary inspection runs on a blocking
worker. The Drop fallback stays synchronous. Actual permission/inspection failure
remains Lost. See [Apple XNU killpg1](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_sig.c).
