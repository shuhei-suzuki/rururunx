# Issue 23 verification

Requirements draft only at main baseline4851fcd. Current domain.rs:195–287 has
Criterion/Dependency/DAG/Proposal/Goal fields. state/mod.rs:257–312 persists the
Goal and validates ownership through validate_goal_references:1338–1370, which
checks unique nodes/declared endpoints/Task ownership but has no cycle/evaluation
consumer. Current main CLI has config/project commands only; Goal execution is
not claimed.

Dependency inspection: public Issue23 depends on #2/#8/#26, all merged. Independent
requirements work is runnable. Shared-state source, native scope preservation and
schema/projection migration need coordination with #19 and #43; this draft edits
neither implementation nor their immutable review worktrees.

Pending: requirements review, design gate, implementation, real consumer controls
and compiled mutations, regression/build/lint, migration and exact-head CI.
No Goal completion, automatic scheduling, CLI/TUI or dogfood credit is claimed.


Requirements1 ed1624d native e9c97fa8-8d91-4a2e-bbd9-6ba13a593f5a completed,
owned cleanup verified. One High and two Medium classes verified: Human ingress
provenance versus public disposition strings, in-flight graph mutation/required
membership, and computed versus authoritative lifecycle. Requirements2 names
trusted ingress/controller boundaries, conservative graph mutation rules and all
Goal-scoped required Tasks, and leaves derived readiness independent of persisted
holds. Legacy completion bits remain unverified. Numeric/storage/evaluator details
remain design work. Exact Req1 CI37185979164 succeeded; this covers main's existing
runtime at a documentation head, not Goal model implementation acceptance.
