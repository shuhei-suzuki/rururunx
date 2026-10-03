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
```

## 4. Logical interface

The exact implementation language is TBD. Logical API:

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

## 6. Session model

A session must have a stable rururunx identity independent from OS PID.

Fields:

- session ID
- task ID
- agent ID
- role: executor / reviewer / consultant
- native session/process reference
- worktree
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

## 13. Failure behavior

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
