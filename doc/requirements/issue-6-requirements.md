# Issue 6: Native Codex adapter requirements

Workflow: STRICT (native authorization, process ownership and shared adapter APIs).
Dependency: Issue 4 is merged at `52e7bbb`; persistence, Git ownership and registry
foundations are available. Source: [Issue 6](https://github.com/shuhei-suzuki/rururunx/issues/6),
product requirements and master Agent Adapter design.

## Behavior

1. Execute prepared Task context through the installed native Codex CLI in the
   exact owning worktree. Consultation can use the exact registered source without
   requiring an Issue or writable worker on the protected base branch.
2. Support noninteractive factual Review Bundles and approval review. These roles
   cannot perform the operation they review. Native filesystem/network limits and
   external tool restrictions are enforced before sending a model turn.
3. Preserve native authentication, account/provider selection, hooks, mandatory
   rules, project trust and permission policy. Do not extract OAuth tokens, replace
   native inference with direct API requests, ignore user configuration, or supply
   bypass flags. Model and effort are explicit only when configured; otherwise
   native defaults remain authoritative.
4. Use native session UUIDs with durable rrx SessionIds. Exact owned session resume
   and interactive/native TUI connection are supported only when their native
   prerequisites are established. No last-session inference, foreign UUID adoption,
   history injection or duplicate independent execution conversation.
5. Surface native permission and human-input requests where the native route
   exposes them. Preserve native automatic approval review by default. An explicit
   runtime-broker route may use the native client callback route when permitted;
   it cannot override a native denial or weaken managed requirements.
6. Fail explicitly for missing executable/auth, incompatible native policy or
   protocol, malformed/oversized messages, timeout, uncertain process termination,
   stale prepared input, immutable locks or ownership conflicts.
7. Keep subprocess I/O, output retention, IPC queues and native request maps
   bounded. Supervision is event-driven. Confirm owned whole-group cleanup before
   marking a runtime Session stopped/exited; uncertain termination stays reserved.
8. Record observed native token/cache metrics with exact attribution. Missing
   cost/telemetry stays null with a reason; deduplicate cumulative turn updates.

## Boundaries

The Rust runtime implements protocol/session supervision rather than coding-agent
reasoning. The native CLI owns inference, authentication, tool behavior and native
conversation history. Prepared context selection is owned by Issues 18–20; review
aggregation and approval policy remain Issues 9–10. CLI factory/TUI orchestration
is integrated later. Native Goal support is optional and does not block this issue.

Each session has one Project/Goal/Task scope and private owned process/IPC state.
No shared app-server daemon, global process kills, foreign browser/account changes,
or native trust/configuration writes are used to make a test pass. Tests operate on
isolated temporary Git repositories and supplied synthetic review text.

## Acceptance evidence

- Real installed Codex executes in an isolated owning Task worktree.
- Native review produces a valid structured result without target operations.
- Explicit model/effort and unchanged native defaults are observable.
- Exact owned resume preserves rrx/native identities and refuses foreign scope.
- Interactive/native TUI route retains native trust/permission prompts; any
  unavailable prerequisite has an explicit typed outcome.
- Callback correlation rejects foreign/replayed IDs and persistent grants.
- Two-Project context, environment, Git and native-reference isolation regressions.
- Bounded framing, failure cleanup, cancellation, stale revision and immutable
  review regressions fail under meaningful mutations.
- Existing workspace checks, immutable independent native review, exact-head
  Linux/macOS CI and documented native limitations before merge.
