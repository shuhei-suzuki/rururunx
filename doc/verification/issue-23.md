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

Requirements2 8a1e3f8 review completed with owned cleanup verified. Prior ingress,
graph membership/mutation and lifecycle findings were verified resolved. One High
remained: generic Goal definition/criterion replacement could weaken the evaluator's
completion target. Requirements3 classifies initial definition creation as trusted
authority and makes accepted definitions immutable in MVP. Generic/agent definition
changes are unapplied requiring-Human proposals; even Human/controller edits reject
rather than silently settle the original Goal. Tests and an actual completion-
consumer criterion-replacement mutant are required. Bounds include unlisted scoped
Tasks, edge identity is ordered, and explicit graph authority changes must have
designed sibling-currency consequences. Requirements3 review remains pending; no
source or evaluator acceptance is claimed. Raw resumed native usage has unverified
per-round attribution and is not an incremental cost measurement.

Requirements3 1fc82b1 independently approved with no findings and verified owned
cleanup; native resumed meter attribution remains unverified. Exact CI37188124469
passed Linux/macOS. Design1 proposes accepted-definition authority, separate
observation ledgers, bounded coherent DAG evaluation and an actual Workflow success
extractor. Legacy origin/evaluators stay unverified, definition proposals cannot
apply, explicit graph writes retain native raw Goal currency invalidation, and
coordinated schema/writer fencing remains required before source integration.
Design/source reviews and actual acceptance remain pending.
