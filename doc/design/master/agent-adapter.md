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
review/executor races. Generic processes receive only explicit environment; Generic callers
must include intentional native baseline HOME/PATH/config/auth so safety settings
remain authoritative. Grok separately retains its constructor native control/auth
baseline; pending Issue51 rejects caller control replacement and scopes eligible ordinary
caller refs without inheriting all ambient variables. The adapter never supplies bypass
flags or an empty native configuration home.

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

## Pending Issue51 native environment authority

Grok currently whitelists intentional native constructor baseline but does not bind
ordinary caller keys/known foreign baseline references to persisted owning Project
refs. A synthetic actual-child probe verified two cross-Project paths; it did not use
installed native Grok or real credentials. Issue51 requires owning caller declarations,
opaque rejection for known foreign non-control baseline references and fresh all-lifecycle
name-only decision admission, preserving global
native auth/hooks/config/safety and immutable controls. Requirements/design/source gates
are pending; #5/#6 patterns are comparison inputs, not universal safety acceptance.
No child isolation fix or schema change is claimed in the current baseline. The proposed
[Issue51 design](../issue-51-design.md) adds pure registry/control predicates and a
fresh relevant environment decision in the scoped Session transaction before actual
native spawn; approval/source implementation is pending.
