# Issue #8 requirements

Issue: https://github.com/shuhei-suzuki/rururunx/issues/8

## Scope and acceptance

Implement a Rust Task-scoped workflow runner below Goal Runtime. QUICK, STANDARD,
and STRICT must execute phase actions through AgentAdapter and explicit evidence
ports; presets alone are insufficient. QUICK may escalate to STANDARD and STANDARD
to STRICT. Existing Task workflow, Project/runtime minimum, stricter user choice,
and risk recommendation form a monotonic maximum. Project config and scoped rules
are reloaded before phase entry; mandatory rules survive context budget selection.

Commit requirements/design/implementation milestones before tests/review under explicit
user Goal §35. Persist phase attempts, evidence references, escalation/invalidation reasons, and context
versions. Context freshness includes exact ownership, phase, revision, source
versions, and rules. Phase completion, workflow history and Task ContextVersion
pointer commit atomically under snapshot CAS. Inactive Projects/Goals and reserved
or Lost executors cannot be bypassed. Concurrent source capture must not hold the
shared Store mutex or overwrite unrelated state changes.

## Boundaries

Review scheduling/verdict reconciliation (#9), approval routing (#12), crash recovery
(#13), repository-map selection (#18), GitHub operations and CLI/TUI are explicit
ports or later integrations. Agent exit success is transport evidence, never a
review verdict, test result, merge permission, or acceptance assertion. Missing
external evidence blocks progress. No built-in LLM planner, permission bypass,
automatic downgrade, automatic retry after uncertain launches, or automatic merge.

## Verification

Drive all three presets through dedicated fake executor/reviewer adapters and
explicit fake evidence ports; assert actual launches, ordering, durable history,
version propagation, failures and unsupported capabilities. Regressions cover
escalation, rule changes, stale revisions, concurrent CAS, restart, cross-Project
inputs, inactive ownership, and Lost reservations. Verify meaningful mutations,
clean immutable independent review, and Linux/macOS CI before root merges.

Reviewed acceptance regressions also cover definitive rejected verdict remediation,
Issue/worktree output binding and concurrent owner metadata, new HEAD publication,
formal artifact dependency invalidation, known stale zero-call gates, same-Session
waiting reevaluation, restart diagnostics, ordinary Store authority bypass rejection,
and impossible initial/completed/finished/history transitions. Unknown outcomes retain
reservations; no state transition substitutes for verified native termination.
