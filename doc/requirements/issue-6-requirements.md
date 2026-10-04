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
   bounded. Supervision is event-driven. Confirm the actual owned native workload
   cleanup before marking a runtime Session stopped/exited; one selected parent
   process group is insufficient when native commands can create other groups or
   sessions. Uncertain termination stays reserved.
8. Record observed native token/cache metrics with exact attribution. Missing
   cost/telemetry stays null with a reason; deduplicate cumulative turn updates.
9. Atomically fence owning parent versions, scoped lock versions and Session CAS
   with durable metadata-only input consumption before inference. After possible
   dispatch, absent exact authoritative native terminal evidence stays Lost even
   after verified process death or an interrupt acknowledgement. Never implicitly
   replay the consumed input; fresh continuation metadata belongs to a new
   higher-version Starting attempt and is immutable during that attempt.
10. Register one private attempt control before any fresh start, resume or
    checkpoint await/publication. The owned task outlives caller cancellation and
    completes bounded cleanup plus exact terminal restoration/publication. Stop
    targets its captured attempt, joins its level-triggered factual outcome and
    never cancels a later attempt. Consumed-input CAS and checkpoint replacement
    share their admission mutex with cancellation; failed CAS preserves the actual
    first cause. Queued consumed-turn stop survives acknowledgement and transfer
    to the sole supervisor. Unpublished or uncertain completion remains explicit.
11. Acquire a genuine private attempt-bound native workload owner before the
    first native side effect, including configuration discovery or server startup.
    Its supported profile must cover all processes that profile can start: setup,
    native helpers, preserved hooks/MCP, tool commands and frontend children where
    enabled. Unsupported ownership fails explicitly before that side effect and
    before operation/input consumption; no app-server probe may bootstrap the
    proof it already needs. Native process IDs, parent/group/session hints,
    termination acknowledgements and background-terminal inventories do not
    create ownership or whole-workload cleanup authority. After possible dispatch,
    absent cleanup authority remains Lost with private completion false even if
    the native turn and every known parent group have ended. Clearing a proven
    dead parent PID does not release the uncertain workload reservation.

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

## F1 workload ownership and platform acceptance (requirements correction)

This STRICT correction closes an identified guarantee gap; its design and source
gates remain pending. It does not waive the native Codex MVP requirement or claim
that the current adapter has enforced the new workload boundary.

[README](../../README.md) names macOS and Linux as MVP hosts. The product's
native adapter MUST set includes Codex, and Issue 16 still requires real native
dogfooding. A Linux-only candidate or an explicit unsupported result is not
macOS native-execution acceptance. Local-first operation and unchanged native
auth/defaults/hooks/trust remain required; no privileged system installation,
account/config mutation or substitute model API is implied by this correction.

Workload readiness is profile-specific and must be established at the real
launch/operation consumer. Executor and any command-capable frontend require
ownership of native tool workloads and arbitrary child group/session creation.
Decision Review, ApprovalReviewer and no-command Consult can use a smaller
profile only when its enforceable no-command inventory and its actual setup,
hook, MCP and helper lifecycle are independently covered. A zero model-tool
count cannot prove startup cleanup. Mandatory hooks and rules stay preserved;
their absence or supported lifecycle cannot be assumed or achieved by disabling
them. Unknown setup inventory/ownership fails unsupported before native startup.

For an unsupported pre-operation/pre-dispatch attempt, registered Preparing
cleanup and exact historical rollback/factual failure remain authoritative.
Where native input or an operation may already have been admitted, unknown
workload cleanup retains Lost and cannot certify transport success, settle an
operation, permit replay or release its execution reservation. The actual
private Issue 19 operation/settlement port must compose current native outcome
with current workload cleanup; neither a source-version record nor JSON fields
mint that authority. No model input is consumed merely to discover ownership.

A backend acceptance claim requires first-party platform contracts, actual
installed conformance and a private owned handle that cannot be deserialized or
adopted from recorded process numbers. Startup/fork/exec enrollment must be
covered from the first possible child, cross-project/workload migration must be
excluded, and bounded event/inspection loss stays Unknown. Selected-group proof
continues to be useful for that group; it is never extended to unobserved jobs.

Current installed Codex0.160.0 evidence establishes a separate native PTY SID and
a synthetic second-SID child surviving both native command termination and
owned stdio-server exit. The finite child later self-completes; only the wrapper's
selected owned server group is independently death-verified and reaped. This is
standalone command conformance, not a successful unified_exec/adapter cleanup
test. No currently verified macOS backend provides the required entire workload
authority. Linux delegated-cgroup and macOS scoped event mechanisms are design
candidates only; platform prerequisites and actual ownership/cancellation proof
remain acceptance blockers until their own gates succeed.

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
- Genuine profile-specific workload authority before the actual first native
  startup/operation consumer, including configuration discovery; unsupported
  readiness sends no native startup/model frame or consumed-operation marker.
- A causal real-consumer fixture in which known parent-group death and a valid
  native terminal cannot certify an escaped/unverified workload, plus compiled
  mutations removing the caller readiness/cleanup guard.
- Installed supported-host proof for nested child sessions, retained ownership
  during stop/caller drop, bounded uncertainty, exact private Issue 19 settlement
  composition and real Issue 16 native execution. An unsupported fallback and
  synthetic no-subprocess control do not satisfy platform/native MVP acceptance.
