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

Design1 aad526b review completed with owned cleanup verified: High generic Goal
DAG/pack/legacy-observation writer bypass, Medium missing managed-Goal predicate
at real dispatch consumers, and Medium Human fingerprint invalidated by hold
resolution/pack refresh. Primary put_goal/version, Workflow active and native
owner code verify the gaps. Design2 rejects every generic Goal diff, inventories
actual protected Task admission predicates, and separates semantic attestation
DAG/source/Task pins from raw lifecycle currency, with declared pack dependency.
Causal controls/mutants are required; Design2 approval and source remain pending.

Exact Design1 CI37188880032: Ubuntu succeeded; macOS Context binary9PASS/7FAIL.
First failure was bounded-context Git cleanup SessionLost from inspection timeout;
six later failures reported the sticky prior-uncertainty latch. They are not seven
independent observer failures. Full external log preserved at temporary
rururunx-issue23-design1-ci-failed.log; #46 owns the observer fix/regression.
No rerun, serialization, relaxed timeout or root-cause attribution is claimed.

Design2 7023ee8 completed with verified owned cleanup: reviewer approved with a
Medium D4 still reported. Do not treat that as no outstanding verified defect.
D1-D3 are verified closed; D4 actual consumer revalidation lacked listed Task/hard
prerequisite checks, and generic put_task could create unlisted managed Tasks.
Primary Store/Workflow/native code verifies D4. Design3 extends actual admission
and generic Task insertion fences, with causal tests/mutants; review pending.

Independent #19 primary inspection found Engine terminal release/retry uses DB
labels, not actual adapter cleanup authority. Design3 makes private allocated-owner
settlement producer integration necessary for the WorkflowSuccessProof. No existing
cleanup certificate, schema addition or native outcome proof is falsely claimed.
#19 owns that shared design/source; its exact durable callback/receipt and #41/#14
claim consumer gates remain pending.

Design3 38417d4 completed with owned cleanup verified: D4 closed, one verified
Medium D5 remained. Before Workflow creation, generic put_task could lower class/
risk or change executor/worktree ownership; primary owns_workflow-only fence
confirms it. Design4 pins accepted Task policy before any Workflow, rejects those
generic changes, binds real Workflow success to the approved minimum gates, and
allows only audited strengthening/provider/Git ownership transitions. Actual
pre-Workflow downgrade consumer controls and compiled mutation remain required.
No managed dispatch precedes composed private settlement/epoch integration.
Exact Design3 CI37190102126 succeeded; individual jobs still to be checked.

Design4 f4b4926 independently approved with no Critical/High/Medium and verified
owned cleanup. One Low D6 asks explicit protection of the new accepted Task policy
origin/digest/minimum gates; clarification requires byte-identical generic Task
replay only, all actual fields typed-port-owned, and weaker-policy rewrite controls
with compiled reduced-gate consumer mutant. It does not allow generic blockers/
next_action (#19 writer pins). Legacy unlisted wording and revocation-only
continuation are clarified. Source/security review must verify these obligations.
Exact CI37190847079 succeeded; individual jobs remain to be checked.

Design4 exact CI37190847079 and latest documentation7ad14c4 CI37192020953
passed all Linux/macOS jobs. Main80452f4 observer integration is present in
5ac5e6f and7ad14c4; the earlier Design1 failure remains preserved above.

Initial source component (tests/review pending): bounded TaskDag::hard_order is
used by the existing real Store Goal save transaction. Controls cover a diamond,
disconnected required nodes, advisory cycles, boundary-size graphs and actual
Store rollback/reopen for cycles, self/duplicate edges and missing endpoints.
This does not claim managed Goal authority, verified readiness/completion, native
settlement, scheduling or CLI acceptance. Source review and causal mutations
remain required before this component or the composed implementation is accepted.

Initial graph source33e34d2 verified from clean worktree: fmt/clippy, full default
DEBUG186 Rust tests +2 documentation tests passed (3 installed-native tests remain
ignored), debug/release builds passed. RELEASE graph3 and Store13 controls passed.
The existing native/Workflow tests are regressions, not acceptance of managed Goal
admission or actual #19 settlement. Six compiled operators in
[graph mutation ledger](issue-23-graph-mutations.json) each reached the intended
unwrap_err assertion on an actual successful mutated call: G01 cycle, G02 ordered
pair, G03 advisory self edge and G04 Store wiring at the real save transaction;
G05/G06 finite node/edge caps at the structural graph API. Thus4 Store kills and2
structural API kills, with no native/evidence/authority credit. Exact-base source
was restored, graph3/Store13 controls passed, and the clean detached mutation
worktree was removed normally. The validator preserves bounded allocation after
caller deserialization; bounded DB decoding/all-scoped Task scans remain pending
in the complete managed implementation. Independent source/security reviews and
exact public-head Linux/macOS CI remain pending.

Full debug regression log SHA256: `db5c3576a449d12657bdfea04bdad9317bd7d838970762bdd4e219ef356beb34`; release graph/Store log SHA256: `e281f267ae34f7c028bc1609e5651aae82925e6322df305630abaa9312c05286`.
