# Agent Adapter Design

**Status:** Draft
**Scope:** MVP adapter contract

## 1. Goal

Allow rururunx to use heterogeneous native coding agents without hard-coding workflow logic to a provider.

Claude Code, Codex, Grok, Gemini, and open-source agents are represented through the same capability-oriented contract.

## 2. Principles

- adapter is thin
- native CLI remains authoritative
- unsupported capability is explicit
- adapter must not simulate a capability it cannot safely provide
- interactive and non-interactive launch modes are distinct
- permission interception is optional capability
- reviewer execution is separate from executor execution

## 3. Capability model

Suggested capability flags:

```text
execute
consult
review
inspect_diff
inspect_command
interactive
non_interactive
attach
resume
permission_interception
structured_output
usage_telemetry
prompt_cache_telemetry
context_checkpoint
native_goal
native_goal_status
native_goal_resume
```

## 4. Logical interface

The core implementation language is Rust. Logical API:

```text
probe() -> AgentInfo
startExecute(taskContext) -> SessionRef
startConsult(context) -> SessionRef
startReview(reviewRequest) -> ReviewRunRef
attach(sessionRef)
resume(sessionRef)
stop(sessionRef)
status(sessionRef) -> AgentSessionStatus
submitApprovalDecision(sessionRef, decision)
capabilities() -> CapabilitySet
```

Optional:

```text
subscribeEvents(sessionRef)
parsePermissionRequest(event)
parseStructuredReview(result)
usage(sessionRef) -> UsageSnapshot
cacheUsage(sessionRef) -> CacheUsageSnapshot
checkpoint(sessionRef, contextCheckpoint)
startNativeGoal?(goalContext) -> NativeGoalRef
nativeGoalStatus?(nativeGoalRef) -> NativeGoalStatus
resumeNativeGoal?(nativeGoalRef, goalContext)
```

## 5. AgentInfo

Contains:

- adapter name/version
- provider
- available executable
- model configuration support
- effort/reasoning configuration support
- capabilities
- current authentication/readiness status where detectable
- usage/token telemetry support
- provider/native prompt-cache telemetry support
- native context/checkpoint support where available
- native goal capability support where available

## 6. Session model

A session must have a stable rururunx identity independent from OS PID.

Fields:

- session ID
- project ID
- goal ID
- task ID
- agent ID
- role: executor / reviewer / consultant
- native session/process reference
- worktree
- repository/project root identity
- start timestamp
- status
- restart/reconnect metadata

## 7. Review contract

Input should include only the context needed for the review:

- task/phase
- target revision/worktree
- review instructions
- known project rules
- required model/effort
- previous round findings when applicable

Output should preserve:

- reviewer identity
- completion state
- findings
- severity
- evidence/references
- reviewer summary
- structured failure if review could not run

The Review Engine, not the adapter, determines whether 1/2/3+ reviewer results satisfy completion policy.

## 8. Approval review contract

For permission review, the adapter should accept a normalized Approval Request and produce:

```text
APPROVE | DENY | ESCALATE
reason
confidence
risk
evidence/context
```

The reviewer process must not execute the requested operation.

## 9. Native permission integration

Each adapter may integrate differently:

- native permission callback/tool
- hooks/events
- structured stdout/event stream
- PTY prompt detection as a last resort

Provider-specific interception belongs inside the adapter; Approval Broker remains provider-neutral.

## 10. Generic CLI adapter

A generic adapter should be possible for tools that support:

- command template
- working directory
- stdin/stdout
- exit code
- optional structured response parser

Generic adapters may have fewer capabilities than first-party adapters.

## 11. MVP adapters

Required:

- Claude Code
- Codex
- Grok

Desired:

- Gemini
- generic CLI adapter
- local/OpenAI-compatible review adapter

## 12. Context-efficiency integration

Adapters do not decide what repository context is relevant. That belongs to the Context Efficiency Layer.

Adapters are responsible for:

- accepting a prepared Context Pack / Review Bundle reference or payload
- exposing native context limits where detectable
- reporting input/output/cached-token usage where available
- reporting provider-native cache hit/use metadata where available
- supporting native session checkpoint/summary primitives where available
- preserving stable prompt ordering when rururunx controls prompt construction and provider caching benefits from stable prefixes

If a native CLI owns its own context retrieval/caching internally, the adapter should expose what is observable without duplicating the provider implementation.

Absence of telemetry must not make the adapter unusable; it is an explicit capability limitation.

## 13. Native Goal integration

A provider may offer a native long-running goal mechanism.

Adapters may expose this as an optional capability, but the rururunx Goal entity remains authoritative.

Requirements:

- native goal use is optional
- native goal/session identifiers are persisted only as execution references
- provider-specific goal state is normalized into rururunx events
- loss of native goal state must not lose objective/completion criteria/Task DAG
- another provider can continue the Goal
- adapter fallback uses ordinary execute/consult/review interfaces with Goal/Task Context Packs

For Codex, native `/goal` may be used when technically and programmatically appropriate, but the core runtime must not require it.

## 14. Failure behavior

Adapters must return explicit failure categories such as:

- executable missing
- authentication unavailable
- unsupported capability
- launch failure
- native session lost
- parse failure
- timeout
- permission interception unavailable

Workflow Engine decides whether to retry, choose a fallback agent, or escalate.

## 15. Implemented Rust baseline (Issue #4)

Native Grok Issue 7's scoped ACP file-executor and zero-tool decision modes are specified
in [its design](../issue-7-grok-adapter.md) and [requirements](../../requirements/issue-7-requirements.md).
The explicit GrokAdapter implements a private bounded ACP actor with two scoped file
tools and zero-tool decisions. Installed Grok 1.0.46 acceptance performed owned edits,
fresh-input same-UUID continuation and a locally schema-validated DENY. Native sandbox
labels alone do not prove Task-only filesystem scope; descriptor-based ACP FS checks,
exact owner/lock/Session dispatch CAS and post-turn inventory enforce this baseline.
Prepared text is enclosed beneath a non-command first line, never sent as native control
metadata. Unknown dispatched outcomes remain Lost even after verified process death.
Only private exact native completion plus cleanup/persistence authorizes transport success;
actual OS exit code is preserved. Native aggregate usage is collected per owned prompt.
Executor writes require one unfinished owned search_replace before any syscall; a failed
host write with possible effects denies transport success even after native end_turn and
matching inventory.

The additive object-safe `AgentAdapter::start_structured(request, schema)` accepts
caller-owned per-turn constraints. Its default returns UnsupportedCapability;
native Codex and Grok explicitly implement it. Grok delegates the registry's trait
dispatch to its existing bounded schema check and native launch path, preserving
local enum, required-property and additional-property validation. Structured event
collection alone does not establish support for arbitrary caller schemas. Review
Set policy and approval semantics remain caller responsibilities.
Shell/PTY, ApprovalReviewer, universal permission interception and restart recovery remain
unsupported. See [verification](../../verification/issue-7.md) for final review/CI status.

`adapter::AgentAdapter` supplies an object-safe async contract and extensible
`AgentRegistry`. Generic registrations load runtime argv unchanged; model/effort
settings require a native implementation and fail explicitly in the generic
baseline. PreparedInput carries exact Scope, kind, revision/version, source
versions and selected payload. ReviewBundle and ContextPack are distinct inputs.

Generic CLI advertises only `execute` and `non_interactive`. It cannot promise
read-only consultation/review/approval, interactive PTY/attach, resume, native
Goals, checkpointing or token/cache telemetry. These operations fail explicitly;
Usage uses nullable fields with an unavailable reason. Provider authentication is
unknown rather than guessed from executable presence. Native #5/#6/#7 adapters
must enforce provider-specific safety modes before advertising capabilities.

Generic launch requires persisted Project/Goal/Task ownership, exact canonical
Task worktree/branch and owning Git common directory. Base/main/master/detached
execution and Git environment overrides are rejected. Starting reserves the Task
before bounded async Git metadata preflight outside the shared Store mutex.
The adapter rechecks original lifecycle-validation Project/Goal/Task versions
before spawn and uses the same runtime-native Git environment as Git management,
while Store's task-level lock/session exclusion prevents
review/executor races. Native processes receive only explicit environment; callers
must include intentional native baseline HOME/PATH/config/auth so safety settings
remain authoritative. The adapter never supplies bypass flags or an empty native
configuration home.

Tokio supervises private process groups without busy polling. Concurrent output
drains retain bounded 64 KiB byte tails and report truncation/I/O failures. Natural exit uses waitid NOWAIT and SIGCHLD to kill the owned group before
reaping the leader, avoiding PID reuse. Stop/drop kills the owned group; terminal
stop is idempotent. Runtime crash/SIGKILL leaves reservations conservatively
blocked, including Lost, until verified-dead recovery; intentional process-group
escapes require stronger future containment. Up to 32 terminal Sessions are
retained; release or eviction drops transient output but preserves durable state. Persisted
Session IDs remain independent from PID and native references. Restart without a
live supervisor yields SessionLost, not a fabricated reconnect. See
[issue design](../issue-4-design.md) for lifecycle and current boundaries.

NativeGoalRef contains Scope, optional SessionId and opaque provider reference.
Implementations must validate the persisted owner on status/resume and require
checkpoint input Scope to equal Session Scope. Generic prepared payload size is
observable in bytes; provider token/cache values remain unknown.

On macOS, native /bin/ps inspection must be available. Its trusted direct-child
observation and EOF/output are bounded by the post-spawn250ms budget with nonblocking drains; mandatory post-KILL direct-child reap is not hard bounded. Failed inspection preserves Lost reservations. Launch
post-spawn executor cleanup diagnostics use scoped audit events and preserve
Blocked Project native ownership metadata. Preflight/cancelled Lost may lack
native PID evidence; generic reconnection is unsupported and explicit recovery
remains necessary.

### Grok terminal supervision receipts

The existing scoped `grok.turn_observed` event carries `cleanup_receipt` after
reconciliation. Its exact eleven keys are `owned_process_group_created`,
`cleanup_ok`, `cleanup_state`, `reap_io_kind`, `output_verified`,
`stderr_drain_state`, `stderr_read_error`, `ownership_uncertain`,
`uncertainty_by_stage`, `dispatched`, and `native_outcome`. Values are measured
facts; no receipt field feeds process ownership, PID clearing, Session state,
reservation release or transport completion.

`cleanup_state` is one of `not_attempted`,
`group_cleanup_failed_unclassified`, `reap_timeout`, `reap_error`, or `succeeded`.
`reap_io_kind` is null except on `reap_error`, where the finite crate-owned vocabulary
is `not_found`, `permission_denied`, `interrupted`, `invalid_input`, `invalid_data`,
`timed_out`, `would_block`, `unexpected_eof`, `broken_pipe`, `out_of_memory`,
`write_zero`, or `other`. No error body/debug rendering is copied. Group-cleanup
failure cause is unclassified because the shared helper erases its structured cause;
message parsing cannot recover authority.

`stderr_drain_state` is `not_started`, `joined_returned`, `joined_panic`,
`joined_cancelled`, or `budget_elapsed_abort_requested`. `stderr_read_error` is
always `unavailable`: a joined drain, including a panic/cancellation join result,
does not prove successful reads. The original output verification bit becomes false
only when the existing250ms join budget elapses and abort is requested. No receipt
claims verified drain completion from that abort. `not_attempted` cleanup and
`not_started` drain on a pre-spawn stop are vacuous facts, not agent execution proof.

`uncertainty_by_stage` has exactly `native_child`, `pre_spawn`,
`in_session_binding`, and `reconciliation` booleans. Each retained flag is loaded
once after reconciliation; the all-flags total determines the original clean
operand independently of labels. A stage false means no retained uncertain entry
in that stage, not that the stage ran or was verified. Checkpoint verification
uses a separate ownership object that is never sampled into a receipt or clean
operand; a Checkpoint-labelled flag would still count if present in sampled
ownership. This sampler behavior has unit-only coverage and grants no checkpoint
runtime total or supervise-stage projection. Native dispatch/outcome are provisional
facts; matching native completion still needs every original terminal gate.

These measurements preserve the existing native250ms reap/drain budgets, group
inspection limits, error priority and unknown-dispatch rewrite. Synthetic tests cover
clean receipt/attempt windows on both OSes; forcedUnknown and stage-creation trace
proof are macOS-only. The bounded test-only projection and creation trace are excluded
from runtime authority. Historical cleanup uncertainty is not explained by receipts.

### Provider transport completion (Issue #8 integration)

`AgentAdapter::transport_succeeded(&SessionStatus)` defaults to Exited, no failure,
and actual OS exit zero. Persistent native server providers can override using their
private owned turn-completion journal after verified cleanup/terminal persistence,
without fabricating exit codes or trusting caller recovery JSON. Workflow verifies
saved SessionId/Scope/actor/role/worktree before consulting this provider method;
transport completion never replaces review/test/acceptance gate evidence.

### Bounded macOS owned-group observation (Issue #46)

STRICT [requirements](../../requirements/issue-46-requirements.md) and
[design](../issue-46-design.md) define the implemented exact owned process-group query with
bounded stderr and expected unreaped-leader evidence under explicit UNIX2003
selection semantics. Empty/error/malformed or
incomplete observations must remain Unknown; exit0 alone is insufficient.
250ms, uncertainty and kill-before-reap remain authoritative. Published XNU still
traverses/allocates against global processes. This contract does not explain prior
CI timeouts or contain detached native descendants. Non-atomic fork/exit sampling and existing partial-success
KILL with non-signalable survivors remain residual limits; Linux cleanup semantics
remain unchanged.

## 16. Native Codex implementation in progress (Issue #6)

The following records the earlier selected-group component baseline. Section17
records the unresolved whole-workload eligibility limit: the current source remains
unfixed for whole-workload ownership until its design/source/consumer gates finish.
These implemented protocol pieces do not advertise a genuinely supported native
production workload profile or satisfy mandatory both-host/native acceptance.

The explicit `codex::CodexAdapter` uses an owned private Unix WebSocket app-server
and native thread UUID. Noninteractive execute, consultation, structured review,
owned resume, checkpoint and nullable token/cache usage are implemented. Exact
Project/Goal/Task, canonical Git ownership, prepared HEAD, native CWD/profile and
directory inode binding are rechecked before inference. Immutable review requires
a clean locked worktree; source consultation requires the registered primary root.

Native authentication/configuration/hooks/trust and unspecified model/effort
remain authoritative. Executor selects the verified built-in local environment
and its own writable worktree profile; decision roles have no execution
environment or target operations. Native turn completion is separate from
confirmed process-group cleanup and durable terminal state. Resume adopts only
the current in-memory supervisor's exact owned UUID, never a persisted hint.

Pending native approval callbacks expose exact identities, bounded operation
contents and a reviewed-operation digest. Unknown patch facts and unscoped targets
cannot be granted; resolved native requests retire without a runtime grant. The
default Human route cannot be auto-granted. An explicit trusted runtime-broker
integration requires the existing native client reviewer route and revalidates
ownership before one-time audited replies; automatic/managed review is never
replaced. Unknown/persistent/expanded grants are unsupported. Caller preparation,
review quorum, cross-agent policy and command completion remain upstream concerns.

Per-turn usage includes every native model/tool cycle, subtracting the previous
owned cumulative gauge on resume without adding repeated notifications. Missing
or reset counters and monetary cost stay null. Interactive/attach and final native
source review/mutation/CI evidence are pending; this section does not claim Issue
#6 completion. See [issue design](../issue-6-design.md).


Codex experimental protocol conformance currently targets native CLI 0.160.0;
unknown versions fail before inference. Selected native provider credential/header
references are scoped to the owning Project without extracting native credentials.
Native Codex input admission atomically checks owning Project/Goal/Task versions,
the full scoped lock/version set and Session CAS with metadata-only consumed-input
audit. Unobserved native outcomes retain Lost reservations even after confirmed
process death; interrupt acknowledgement alone cannot authorize replay. A fresh
continuation pins its new source metadata in Starting before dispatch.
Owned resume requires a freshly checkpointed continuation rather than an implicit
repeat of the old mutating prompt. Checkpoint can refresh mutable own Project
metadata but cannot rebind its repository/worktree. Transport completion and
telemetry compare the private owned turn journal with persisted Session authority;
caller recovery metadata and a server's OS exit zero are never completion proof.

Native Codex preparations install an exact per-attempt control before their first
await or Starting write. Its independently owned task survives caller drop, keeps
owned children through asynchronous cleanup, and publishes a level-triggered
factual outcome only after final persistence or explicit publication failure.
Cancellation and consumed-input/checkpoint CAS share one admission order. Stop
captures that control once; queued interrupts retain the same receiver through
native acknowledgement and supervision. A later registry attempt cannot steal the
completion or receive the old stop. Exact Session snapshots distinguish restoration
from advancement without treating Store version increments as different attempts.
This implementation is under independent source review and does not resolve native
descendant containment, decision CWD or configuration-provenance prerequisites.

## 17. Native workload ownership limit

Section16 records the selected-process-group component baseline. Current native
whole-workload readiness and cleanup remain unproven: tools/helpers/hooks/frontend
routes can create other groups/sessions or delegated work. Native terminal/interrupt
ACK, empty native inventory and one selected group's death do not prove all such
resources complete. Existing source capability flags are component claims, not
whole-workload eligibility or a supported-platform release certificate.

The [strict ownership requirements](../../requirements/issue-6-requirements.md) and
[proposed correction](../issue-6-design.md#f1-correction-owned-workload-and-real-consumer-gates-proposed)
retain the implementation/backend/shared-consumer gates. The correction is
unimplemented; existing generic terminal-label and Executor-only exclusion limits
remain. No native defaults/hooks/auth are disabled, and required both-host native
roles, continuation/stop/Task attachment and four-plus concurrency remain open.
