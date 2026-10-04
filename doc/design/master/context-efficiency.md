# Context Efficiency Design

**Status:** Draft
**Scope:** MVP token/context efficiency

## 1. Goal

Parallel agents and multi-reviewer workflows can multiply the same repository and conversation context across model calls. rururunx must reduce redundant context without reducing required review independence or safety evidence.

The Context Efficiency Layer optimizes **what context is sent**, **when it is refreshed**, and **what can be reused**, while native provider caching remains the provider's responsibility whenever possible.

## 2. Principles

1. **Retrieve, don't dump** — do not inject the entire repository or full task history by default.
2. **Progressive disclosure** — load detailed rules/skills/files only when relevant.
3. **Stable context + dynamic delta** — separate reusable stable context from frequently changing task context.
4. **Share artifacts, not chats** — reviewers receive durable task/review artifacts rather than another agent's full conversation transcript.
5. **Condense history** — bound long-running session context while preserving goals, decisions, critical files, failures, and next actions.
6. **Prefer deltas for re-review** — later review rounds focus on changes since the previously reviewed revision plus unresolved findings.
7. **Preserve independence** — token reduction must not cause reviewers to inherit another reviewer's conclusions before performing independent review.
8. **Measure everything** — token/cached-token/cost estimates should be recorded when adapters expose them.

## 3. Context Pack

Each Task maintains a versioned Context Pack containing concise durable context.

A separate Goal Context Pack contains only cross-Task state and must not duplicate every Task history.

Task Context Pack contents include:

- project ID / repository identity
- goal ID
- task purpose and acceptance criteria
- workflow/risk class
- relevant project rules
- architecture/design references
- relevant repository map slice
- changed files/symbols
- impact-analysis summary
- current diff/revision identifiers
- test/verification results
- unresolved findings
- current blockers/next actions

The Context Pack is not a replacement for source code. Agents may retrieve source files as needed.

Task Context Packs are versioned by Task state/revision so stale information can be detected.

### Goal Context Pack

Goal-level context includes:

- objective / completion criteria
- constraints / non-goals
- source-of-truth refs
- Task DAG summary
- cross-Task decisions
- Goal blockers
- next runnable work
- aggregate review/security state
- aggregate metrics where useful

Goal context references Task Context Packs rather than copying them wholesale.

## 4. Repository Map / Context Index

Issue 18 implements the local Task-scoped `RepositoryContext` library and
`repository-context` inspection example. It indexes exact bound worktrees with
SHA256 content/ignored-evidence/rule/config freshness, deterministic lexical
ranking and expansion, and scoped selection evidence. Entire mandatory rules,
evidence and requested expansions must fit; otherwise it returns `NeedsBudget`.
`utf8_bytes_v1` is a labeled packing estimate, with nullable provider measurement.
Source and Git scans run outside the shared Store mutex and recheck state versions
before audit mutation. See [bounded retrieval design](../issue-18-design.md).
Uncertain native Git cleanup remains fail-closed. Diagnostics preserve the failing
call's native error chain and distinguish an uncertainty latch set by a concurrent
call, which discards the current call's result without attributing the failure to it.
Consumers should render the full error chain (`{:#}` or Debug).
Cancellation may latch uncertainty without returning a diagnostic; verified cleanup
and recovery remain Issue 14 responsibilities.
Durable Context Pack publishing and workflow integration remain separate work.

Maintain a compact repository index inspired by repository-map approaches:

- file paths
- important symbols/signatures
- imports/dependencies
- references/call relationships where cheaply available
- optional language-specific parser data

A token/size budget limits what is injected into an agent prompt.

Selection should prioritize context relevant to:

- task text
- changed symbols/files
- dependency/reference graph
- review focus

The full index may exist locally; only selected slices are sent to agents.

## 5. Progressive Rule and Skill Loading

Do not send all global/project workflow documentation on every call.

Maintain lightweight metadata for available rules/skills and load full content only when triggered by task/phase/file scope.

Mandatory safety/project rules are never omitted merely to save tokens.

## 6. Conversation Condensation

Long sessions may be periodically condensed into a durable checkpoint.

Checkpoint must preserve at least:

- user/task goal
- confirmed requirements/decisions
- work already completed
- current revision/worktree
- critical files/symbols
- commands/tests already run and outcomes
- unresolved errors/findings
- next intended action
- important safety constraints

Recent events remain verbatim for a configurable window.

Condensation must be auditable and must not overwrite authoritative requirements/design artifacts.

## 7. Review Bundle

For each review phase, construct a deterministic Review Bundle rather than forwarding executor chat history.

Typical contents:

- project ID / repository identity
- goal ID / task ID
- immutable commit/revision
- requirement/design artifacts required for phase
- relevant project rules
- compact repo-map slice
- diff or changed-file list
- impact-analysis artifact
- test/verification evidence
- review instructions

All reviewers in the same independent review round should receive equivalent factual inputs unless reviewer specialization explicitly requires additional material.

Reviewer findings are hidden from peer reviewers until independent reviews complete, unless the workflow explicitly defines a consensus/reconciliation phase.

## 8. Delta Re-review

For review round N > 1, avoid resending all prior transient context when possible.

Provide:

- baseline reviewed revision
- new revision
- delta/diff
- prior verified unresolved findings
- fixes claimed
- new test/verification evidence
- stable requirements/design references

Reviewer may request broader context if needed.

## 9. Prompt / Context Cache Awareness

Adapters may expose provider-native cache capabilities and usage telemetry.

rururunx should:

- keep stable instructions/tool definitions ordered consistently when it controls prompts
- keep reusable repository/project context in stable prefixes when provider semantics make this useful
- avoid invalidating stable prefixes with volatile data unnecessarily
- use provider-native explicit cache only when the adapter/provider supports it and policy allows it
- never depend on cache presence for correctness

Caching reduces provider computation/cost; it does not necessarily reduce logical context-window token count. Therefore caching and context minimization are separate optimizations.

## 10. Token Budgets

Configurable budgets may exist per:

- task
- phase
- review
- reviewer
- context-pack/repo-map injection

Example:

```yaml
context:
  repo_map_tokens: 2000
  review_context_tokens: 12000
  recent_history_tokens: 8000
  condensation_threshold: 0.70
```

Exact defaults are implementation decisions and must be validated empirically.

## 11. Context Request / Expansion

An agent must be able to request more context rather than being forced to guess.

Examples:

- fetch file
- expand symbol
- include caller/callee
- include design section
- include previous review evidence

Context expansion is logged for observability.

## 12. Cost and token telemetry

When exposed by native agents/providers, record:

- input tokens
- cached input tokens
- output tokens
- context size
- estimated cost
- context-pack size
- repo-map injected size
- condensation events
- context expansion events

Metrics should be attributable by Task, phase, and agent.

Implemented legacy storage boundary: Store::put_usage rejects any of its six typed
integer fields above i64::MAX before Usage/audit writes; valid values and missing
values are preserved without clamping or zero substitution. Arbitrary cache_metadata
integers remain unguarded/unqualified. This check does not qualify raw observations
or provide phase/round provenance, aggregation, native counters or benchmark results;
those remain open under #21's approved requirements/design and actual producer gates.
The legacy read component validates the requested Scope shape and requires decoded
Project/Goal/Task/Session identities to equal their selected SQLite row columns before
returning history; any mismatch refuses the whole read without repairing state.
The legacy Usage reader projects body decode failure to static text without a raw
serde cause; valid raw metadata and other readers remain unqualified.
This is an isolation/integrity check, not bounded raw-metadata retirement or a
qualified measurement/query capability. Details: [legacy scope component](../issue-21-legacy-scope.md), [legacy decode-error component](../issue-21-legacy-decode.md).


## 13. Invalidation and freshness

Context artifacts must carry enough metadata to detect staleness.

At minimum consider:

- repository
- branch/worktree
- HEAD/commit
- workflow phase
- source artifact hashes/versions
- generation timestamp
- Context Pack version

Invalidation examples:

- HEAD changed → diff/review bundle becomes stale
- requirements/design changed → dependent Context Pack sections become stale
- workflow escalated → newly mandatory rules/evidence must be added
- project rule file changed → relevant rule snapshot must refresh

A stale artifact must not be silently reused for a gate whose correctness depends on freshness.

Context artifacts from one Project must never be reused in another Project merely because file paths, Issue numbers, or symbols look similar.

## 14. Context selection pipeline

Logical pipeline:

```text
Task + Phase + Target Revision
          ↓
Mandatory rules/evidence
          ↓
Context Pack
          ↓
Repository-map candidate retrieval
          ↓
Relevance ranking / graph expansion
          ↓
Budget packing
          ↓
Agent-specific formatting
          ↓
Native Agent
```

Mandatory items are packed before discretionary repository context.

## 15. Baseline mode

For measurement and debugging, rururunx should support a baseline/disabled mode that bypasses optional context-reduction behavior where practical.

This allows dogfood comparison of:

- token use
- cost
- latency
- review findings
- task success
- safety/quality regressions

Baseline mode must still preserve mandatory safety/project rules.

## 16. MVP acceptance

MVP must demonstrate:

- repository-map/context-index generation
- token-budgeted relevant context selection
- versioned Context Pack
- progressive loading of optional rules/skills
- session condensation/checkpointing
- deterministic Review Bundle
- delta-based re-review
- provider cache-awareness hooks
- per-task/per-agent token telemetry when available

A dogfood comparison should measure the same representative workflow with and without Context Efficiency features and report token/cost/time differences plus any quality regressions.
