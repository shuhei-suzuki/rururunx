# Issue 23 verification

Current limited component: structural graph validation implemented, with two prior
source approvals at7c1fd88. Latest maincf8 composition364139d verification is
recorded below; independent current-composition review remains required.
Whole23/managed authority/evaluation/native/MVP remain OPEN.

Historical requirements baseline4851fcd (the following chronology is retained). Current domain.rs:195–287 has
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

Initial graph component independent Source1a/Source1b reviews both approved exact
7c1fd88 with no Critical/High/Medium and actual V2 owned cleanup verified. Native
sessions e9c97fa8-8d91-4a2e-bbd9-6ba13a593f5a and
2bee19c2-fd93-4262-a95e-70d3c743063e received identical public-byte-verified inputs;
neither received the other's current result. Only the structural graph component
is approved, not complete23/MVP. Reported tests were reviewed, not independently
rerun by native reviewers. Raw resumed usage/cost/API duration attribution stays
unverified and is not added as per-round metrics.

One Low G23-SRC-L1 is verified from the actual unconditional put_goal validator:
previously accepted legacy cycles/self/duplicate pairs fail an unchanged-graph
Cancel/Fail save. Current generic code can repair a graph and terminate, but that
is not the required future authorized preservation of frozen legacy history.
The composed typed legacy Cancel/Fail port must preserve original invalid graph
bytes and reconcile without graph edits/ratification or a dispatch grant. Add a
real legacy fixture control there; no such typed authority acceptance is claimed
by this structural component. In-memory caps do not cover prior deserialization,
legacy followup vectors or total scoped Task scans; those remain explicit work.

Exact public7c1fd88 CI37192966866 failed macOS lib86PASS/1FAIL/1IGN; only existing
inspection::each_output_stream_has_a_causal_size_failure at442 failed with actual
TimedOut. Ubuntu fmt/clippy passed; tests were cancelled by fail-fast, builds were
skipped, so no complete Linux success. Original log is preserved externally at
rururunx-issue23-graph-source1-ci-failed.log. No rerun or cause attribution. The
reviewed #41 test correction must integrate and pass its own final gates; its
later Grok cleanup failure is separate and not silently erased. Source review
approval does not replace failed CI or complete23's missing native producers.

Documentation639a25c CI37193789536 passed all Linux/macOS fmt/clippy/test/debug/
release build steps. Rust source is unchanged from the reviewed initial graph
component; this later success does not explain the original7c1fd88 timeout or
substitute for #41/#55 cleanup diagnostics. The issue design now identifies the
component's actual reviewed status and preserves the typed legacy Cancel/Fail
fixture obligation. Managed model, native admission, settlement, completion and
full Issue23 remain pending.


## Current structural-component composition364139d

Normal main478 then maincf8 integration completed without conflicts; all original
goal.rs and goal_graph.rs bytes equal prior reviewed7c1fd88. Store's validator
retains the same hard_order call, now at1450, and all current environmental, nullable
Usage, native and Workflow unbound-retry guards remain. Relative to currentmain,
production changes are only the64-line structural method, private module inclusion
and replacement of old node/endpoint checks by hard_order; no schema, Goal lifecycle,
controller authority, acceptance certificate, native dispatch or readiness producer.

On clean364139d default full debug passes381 top-level Rust tests plus2doctests,
26ignored; the nested ordinary witness child1 is separate. Fmt, all-target Clippy
with warnings denied and debug/release all-target builds pass. A full workspace
release attempt FAILS with269 lib passes,2 failures,23ignored: unrelated Grok env
checkpoint/resume fixture parents hit their60s synthetic-child watchdog with only
'running1test' stdout and empty stderr. Cause/regression remains UNKNOWN; no
rerun, serialization, timeout increase or waiver. No graph/Store release tests were
reached in that full attempt. Separate affected release verification passes all
3graph and17Store controls. [Exact gates](issue-23-graph-composed1-gates.json).

All six ORIGINAL compiled operators were re-executed against364139d: four actual
Store calls and two structural API calls commit/return Ok incorrectly, then fail
unwrap_err at goal_graph145,61 or84. Restored4bd9c8b has complete364139d tree,
clean, and all3graph controls pass. These are the same six operators, not twelve
distinct claims. [Mutants](issue-23-graph-composed1-mutants.json) and
[exact patches](issue-23-graph-composed1-mutant-patches.json).

CI37244971523 passes every Linux/macOS step on actual current complete source: see
[commit/tree/all-blob proof](issue-23-graph-composed1-ci.json). This is default debug
testing and release BUILD, not full release TEST success. Separate main478 and
maincf8 CI failures and the local release failure are preserved in
[failure observations](issue-23-graph-composed1-failure-observations.json). Main478
first Context PS deadline273075us led to latch failures; maincf8 one Codex immutable
review ownership assertion saw PS332580us. Linux passed both. The current and PR65
green runs neither explain nor erase these failures. Do not certify the observer,
reader custody or all-native cleanup.

Current [impact inventory](issue-23-graph-composed1-impact-rg.txt) covers every
put_goal/validator/TaskDag consumer and actual Goal lifecycle/graph readers. The
put_goal public library API is defined at Store261 with NO in-crate production
caller; all current call sites are tests, with external composition possible; native/Workflow raw Goal scope/version checks remain. All
production changes stay inside structural validation. Previously accepted invalid
legacy graph Cancel/Fail saves still reject: prior verified Low G23-SRC-L1 remains
a disclosed limitation of this component, with the typed frozen-history closure
fixture obligation in the complete managed design. No completion/dispatch is
exposed and in-memory caps do not certify bounded DB deserialization.

Current independent source/composition review and final metadata-head CI are
required before any limited merge. Complete23,19 settlement,43 binding,14/27
scheduling,24CLI and16dogfood remain OPEN. Existing Issue6 historical private
harness ownership hold is unchanged; selected normal verification closures do not
certify all native jobs.


## Composition1 source review: one verified Medium

Both actual independent source reviewers at0cbb29b completed with selected native
cleanup verified. A APPROVE with2Lows; B REQUEST_CHANGES solely for one Medium
endpoint-control/mutant omission plus2Lows. [Actual results/dispositions](issue-23-graph-composition1-reviews.json).
No source approval is inferred from A alone or from prior initial approvals.

M1 is verified: the old Store endpoint predicate was replaced, but hard missing
dependent was the only existing rejection case; soft missing endpoints and
missing prerequisite could survive two plausible mutations. Three corresponding
Store refusal cases were added to the actual save loop. The source predicate is
unchanged. Independent delta qualification requires both compiled existence
mutants to commit wrongly and fail the actual unwrap_err, followed by full restore
and passing controls. Initial uncommitted control smoke is not final-head evidence.

The legacy Low is broader than the initial G23-SRC-L1 wording: ALL unchanged
invalid-graph saves refuse, including holds/blockers/metadata and Cancel/Fail; old
over-cap and parallel hard-soft pairs are covered. Blocked-Project conservative
holds pass activity validation but fail graph validation, while metadata repair
cannot pass the activity fence. Generic repair overwrites the only current graph
bytes and is not endorsed. New raw-SQL legacy cycle/self/parallel-pair fixtures
characterize all three hold states and Cancel/Fail under both Registered and
Blocked Projects; the attempted object, exact raw Goal body/version, complete
fixture audit and all Task snapshots must remain unchanged. No invalid graph is
ratified, no validation bypass or typed reconciliation is introduced. The full
managed design carries frozen-history hold/closure producer and fixture work.

Evidence precision: hard_order is now included in the rg pattern. put_goal is a
public library API with no in-crate production caller; observed call sites are
tests. Affected release scope was graph3+Store17 only; other caller binaries passed
debug, not release in that aborted full run. Pure graph code has no cfg/debug_assert
profile branch; indegree decrement corresponds to counted hard edges. These are
static facts, not a broader release or observer qualification.

Failure owners: Grok release checkpoint/resume watchdogs remain with open
[Issue51](https://github.com/shuhei-suzuki/rururunx/issues/51), with bounded diagnostic
[follow-up67](https://github.com/shuhei-suzuki/rururunx/issues/67); inspector deadlines
with open [Issue60](https://github.com/shuhei-suzuki/rururunx/issues/60) and
[Issue6](https://github.com/shuhei-suzuki/rururunx/issues/6). The same two Grok tests
passed default debug at364139d. Actual environment fixture uses
tests/support/grok_fixture.rs70–87 Goal::new then put_goal; Goal::new initializes
TaskDag::default, and neither the fixture nor these environment cases constructs
DAG edges. Both validators accept that empty graph. No profile-dependent graph
logic, no native/process I/O in hard_order, and no fixture DAG edits are static
non-interaction facts; cause and regression of the failures remain UNKNOWN.

## Composition round 2 causal controls (2026-10-05)

At clean `153376548a94fd7482e4c9969ce2a8c064ab597b`, affected Debug and Release each pass all four graph controls and all 17 State integration controls with default test parallelism. Formatting, all-target Clippy with warnings denied, and both Debug and Release all-target builds pass. Full workspace Release is not repeated: its prior two isolated Grok watchdog failures and unknown cause remain recorded in the composed1 failure observations, assigned to #51/#67; separate main CI failures remain assigned to #60/#6. This is no full Release suite qualification.

Two additional compiled endpoint operators now fail the actual Store consumer at `goal_graph.rs:148`, where an invalid graph was committed and the expected refusal becomes `Ok`. G07 checks the hard-only endpoint bypass against soft missing endpoints; G08 checks skipping a missing prerequisite. Both isolated mutant commits are restored and the complete restored tree equals the clean tested head; all four graph controls then pass. These are two new distinct operators, alongside the six previously qualified operators, not 14 operators or native acceptance. Exact patches and hashes are retained in JSON. Production source is unchanged from composed1; the verified Medium finding was a missing causal test, now addressed pending two independent delta reviews.
