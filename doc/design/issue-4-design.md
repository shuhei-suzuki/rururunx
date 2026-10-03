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

Generic execute checks persisted Project/Goal/Task, exact canonical Task worktree,
worktree namespace, owning Git common directory/top-level and exact non-base branch.
Git environment overrides fail before preflight. No shell interpolation is added:
configured argv is passed directly; a user explicitly configuring a shell owns its
script. Model/effort are rejected by generic adapters rather than silently ignored.

Generic adapters require a shared Store. Starting is persisted before async Git
preflight; launch failure/cancellation records Failed. #3's Store transaction
atomically excludes active WorktreeLock and reserved/live executor Session for the
same Task, so a reviewer cannot start between preflight and spawn. Running and
terminal snapshots are persisted through existing optimistic Store/audit APIs.
The original SessionId survives PID/native-reference changes and SQLite reopen.
Lost supervisors are explicit: old snapshots/PIDs are insufficient to reconnect.

## Async process boundary

Tokio process/sync/io/time runs native processes and event-driven supervision.
Piped noninteractive stdin is written once, stdout/stderr drained concurrently to
64 KiB tails, and exit/stop waits through select rather than polling. Subscribers
receive latest SessionStatus via watch channels; this is status observation, not
a lossless native event journal. Terminal state carries exit code, I/O failure,
captured byte tails and explicit truncation flags. Raw output is memory-only and
may contain provider text; it is not written into Session/audit by this adapter.

macOS/Linux processes have a private process group. Stop kills the owned group,
waits/reaps its direct child and persists Stopped; dropping an adapter closes the
control channel and performs the same cleanup. Natural exit observes the direct
child; providers leaving background processes/output descriptors produce an
explicit capture failure after a bounded drain deadline. Stopping a finished
Session is idempotent and cannot signal a reused PID. PTY/interactive attach is a
separate native capability, never approximated by these pipes.

`env_clear` prevents accidental project leakage. Runtime callers must intentionally
provide native baseline HOME/PATH/config/auth and project environment. Nothing
creates temporary empty native configuration, disables hooks or supplies unsafe
permission flags. Provider-specific safe environment/launch semantics belong in
#5/#6/#7. Generic capabilities are execute/non_interactive only; telemetry fields
are null with a reason. Native Goals, checkpoints and approval submission default
to explicit unsupported failures and never control rururunx Goal truth.

## Impact and limitations

Adds adapter library and Tokio/nix dependencies without changing durable entity
shapes, SQLite schema, existing project overlays or CLI behavior. Registry/config,
Session Store, future scheduler/review/approval/CLI/recovery and native adapters
are consumers. Tests use native subprocesses, temporary Git fixtures and Store.
No browser/staging deployment target exists for this library boundary.
