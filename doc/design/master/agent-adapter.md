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
remain authoritative. This Generic baseline obligation is not enforced by the generic
wrapper; automatic mixed-provider map prediction remains unsupported until the #11/
#14/#16 driver/recovery/dogfood integration supplies a reviewed provider-bound handoff.
This is not a claimed current CLI/native3 exposure. Grok separately retains its constructor native control/auth
baseline; caller-supplied controls are rejected even with identical values
and scopes eligible ordinary
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

### Inspector failure diagnostics: source gates pending

[Issue60 diagnostic requirements](../../requirements/issue-60-inspection-requirements.md)
have two narrowed approvals; [component design](../issue-60-inspection-design.md) has
two independent Design2 approvals. The diagnostic-only implementation now attaches
finite value-free site/stream/EOF/status/cleanup facts at common inspector error exits,
preserving selected argv/env,250ms observation, Unknown and original authority.
Source2 twice approved scoped source with no C/H/M; final gates remain pending.
Local release555 FAILED (inspector deadline, stdout EOF pending) remains open;
cause and regression status unknown; not repaired.
Wrapped inspector syscall errors now render static site text plus facts, preserve kind
and have raw_os_error()=None. Inner Complete/Stream/validate and valid-live resolver
PERM retain raw values. Fact assertions target actual Context, Generic terminal/
launch-failure audit, and Grok completed-turn cleanup diagnostic/runtime failure.
Grok earlier primary errors still mask cleanup text. Generic preflight bounded-Git,
Grok pre-spawn/binding ownership/index Git and reconciliation_error host-Git remain
source-forwarding-only, unasserted routes without a stage seam. The unknown-dispatch
wrapper's cleanup-selected form has the existing native-child seam but no dedicated
fixture/fact assertion. Incidental release555 pre-spawn failure facts
are uncontrolled observation, not fixture assertion. No universal sink/counter
coverage: positive WouldBlock/status pending/interrupted counts, pending stderr and
Interrupted drain-site wiring have residual unit gaps; helper ISR test is prepared
result credit only.
Reader/driver retention and every existing Drop/reap gap remain separate/open. The
Grok receipt below stays unclassified; facts cannot grant cleanup or alter clean.

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
failure cause stays unclassified as policy until a separately reviewed typed consumer
exists. The shared helper erases its structured cause; bounded inspector fact text may
remain in diagnostics, but message parsing cannot recover or grant authority.

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

## Grok environment admission

The adapter freezes its intentional whitelisted native baseline at construction.
Caller values require current owning Project declarations: LANG, LC_ALL, LC_CTYPE,
TERM, COLORTERM and TZ may use their own ordinary values; native-whitelisted non-control
keys must exactly match the frozen value. Declared unsupported keys are harmless until
passed. RRX_ and controls cannot be passed by callers, including identical control values.
Existing syntax/Git errors retain InvalidInput; environment admission failures have a
fixed opaque InvalidConfiguration category. No environment values are audited or retained
in the name-only admission DTO.

A retained non-control baseline name declared by another Project but not by the owner
rejects launch; it is never silently stripped to select cached/default authentication.
Shared declarations permit only the same intentional runtime value. Undeclared global
native auth/settings/hooks remain intentional. All lifecycle states contribute references.
Controls keep the existing routing/identity/TLS/proxy/prefix policy plus the finite
NODE_TLS_REJECT_UNAUTHORIZED addition. Registry-valid credential-bearing/locating keys
are non-controls, including SSLKEYLOGFILE, XAI_API_KEY and unknown valid native references.
This can make combined private-provider credential configurations unsupported; use
undeclared global native auth or genuinely shared names/values. There is no privacy flag
or distinct per-Project credential-value routing.

ScopeSnapshot reads only own Project/Goal/Task/lock authority. Environment policy projects
only environment_refs through SQLite, streaming every valid name without a foreign-count
cap or foreign Project decoding/filesystem/config reads. Invalid strings do not hide
valid conflicting names; missing/non-array/non-string reference authority fails opaquely.
The bounded value-free DTO admits512baseline names/256bytes each/64KiB total and128caller
names/256bytes each, rejecting oversized configuration rather than truncating candidates.

Initial selection and final scoped Session admission use the same pure name decision.
The final Immediate transaction preserves exact P/G/T/nullable-lock and Session identity,
activity/exclusion/version guards before environment policy. Actor retains its prior
version/watch on failure and publishes the exact saved candidate on success under its
transition lock. Final own-scope/non-executor reservation checks remain before admission;
this does not introduce a new atomic non-executor fence. Production checks stop immediately
before spawn with no intervening await. SQLite and OS exec are not atomic; changes after
admission are explicitly non-retroactive. Explicit higher-version checkpoint refresh and
resume re-admission preserve native UUID/no implicit prompt replay, PID/death/Lost safety.

Explicit Project-scoped candidate inspection reports only that Project's own non-control
reference names present in the frozen baseline, including inactive Projects; it reads no
foreign inventory/activity or values. Never call it automatically from another Project's
rejection. Corrupt or mixed registry-invalid owning refs make the complete operator projection unreadable and must be repaired/cleared. An own native-control declaration blocks that Project's Grok selection even when the caller does not pass the key; another Project's identical control declaration does not revoke the intentional global native control. If
owning source cannot be reactivated to clear refs, reconstruct the adapter without the
intentional runtime variable; do not erase another Project or fall back to cached auth.

The implementation has synthetic actual-child and transaction tests; exact source review,
mutation/default full gates and native multi-Project dogfood remain separate acceptance
requirements in [verification](../../verification/issue-51.md). No schema/ACP/permission,
factory/attach or global native-control bypass is introduced. All future authoritative
state migrations must include this port in the same writer epoch/fences.
