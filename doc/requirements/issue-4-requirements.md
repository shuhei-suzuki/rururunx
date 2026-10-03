# Issue #4 — Adapter contract requirements

Workflow: STRICT, because native process ownership, environment/context isolation
and review locks are security boundaries. Depends on merged #1/#2; #3's atomic
worktree-lock/session exclusion is an integration gate before this PR merges.

The runtime must expose a provider-neutral, extensible capability-oriented Rust
adapter contract and registry, with stable domain Session IDs independent of PIDs.
Launch/status/stop/attach/resume, event subscription, prepared Context Pack/Review
Bundle inputs, usage/cache telemetry, permission/checkpoint and optional native
Goal operations have explicit contracts. An unsupported operation must fail
explicitly, without substituting a weaker operation or adding an LLM supervisor.

Generic CLI baseline executes configured argv directly, with stdin/stdout/stderr,
exit status, a task-owned worktree and explicit environment. It supports only
noninteractive execution: arbitrary commands cannot guarantee read-only consult,
review or approval, PTY attachment, resume, structured results or native Goals.
Generic telemetry is nullable with an unavailable reason; native adapters may
report real observed values through the same Usage contract.

Project/Goal/Task and prepared artifact ownership must match persisted state.
Mutating launch must reserve a Starting Session, reject persisted review locks,
verify exact Task worktree/branch and owning Git repository, and reject base/main/
master or detached execution. Environment must be explicit, must not inherit
other projects' credentials/config, and must reject Git ownership/config override
variables. Callers supply intentional native baseline HOME/PATH/config/auth
environment; the adapter neither creates a safety-free home nor bypass flags.

The supervisor must drain output without deadlock, bound memory, report truncation
and I/O failure, terminate the owned group before reaping on both stop and natural exit, cap
terminal retention, preserve stable
identity on terminal state persistence and report SessionLost after restart
instead of trusting an old PID. It must avoid high-frequency process polling.

Verification uses real fake subprocesses in isolated temporary Git repositories,
tests ownership/locks/failure/stop/output limits/concurrent isolation, and mutation
proof for critical boundaries. Native Claude/Codex/Grok implementation and user
execution CLI belong to #5/#6/#7 and later runtime issues.

Lost/uncertain executor state keeps the worktree reserved until explicit verified-dead
recovery; an unavailable supervisor must not silently release review safety. Native
Goal references must carry Project/Goal ownership. Session writes must retain their
optimistic version and surface concurrent state changes without overwrite.
