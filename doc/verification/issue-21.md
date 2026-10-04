# Issue 21 verification

Requirements draft only, baseline main80452f4/schema3. Public Issue21 depends on
merged2/4 and is runnable requirements work within MVP tracking parent17.

Actual AgentAdapter::usage contract is adapter.rs223–228; GenericAdapter838–864
returns scoped nullable metrics and prepared-input byte fields with explicit
missing native telemetry. Store::put_usage828–876 validates finite cost and own
Session scope/agent, then appends usage/audit; Session lookup precedes the write
transaction. Store::usage879–892 returns raw scoped rows without qualified
aggregation or pagination. Usage domain stores nullable fields and arbitrary
cache_metadata. These are existing foundations, not21 acceptance.

Context master sections9/12/15 and Product40.7–40.10 require native cache awareness,
attributable telemetry and representative ON/OFF comparison preserving safety and
quality. Existing external native resumed review counters have unverified per-round
attribution; this draft specifically requires semantics/boundaries before summing.
No native default, credential, permission or hook is changed.

Pending: requirements review, bounded durable/query/normalization design with19
epoch/producers, actual implementation and causal mutation/regression controls,
native quality comparison, exact-head CI and independent source review. No token
savings, billed cost, cache correctness, TUI or final16 dogfood claim.

Req1 two independent native reviews requested changes with V2 actual owned cleanup
verified. Verified High classes: inventory-based missing-operation coverage and all
reduction-provider overhead; operator pricing/currency/category provenance; and
immutable dispatch-derived phase/round attribution (current usage parameters are
collection-time caller labels). Medium clarifications: model/tokenizer and nested
measurement units, scope-bound operator query ingress, exact event replay identity,
retired bounded legacy raw surfaces, genuine artifact sizes, predeclared equivalent
paired workloads/dispersion/quality oracle and explicit acceptance/KPI closure.
Req2 incorporates these contracts with real consumer mutations. The review's
history-free native-state suggestion is narrowed to honest known-channel exposure/
confounds and owned fixture cleanup, preserving native defaults and user history;
no OS sandbox or secret-state copying. New per-run approval/spend requirement is
not adopted: the active user Goal already authorizes native final dogfood. Respect
requested enforceable budgets and finite time bounds, without inventing a dollar
cap from unqualified native counters. Req2 review remains pending; no implementation.

Exact Req1 30a994b CI37193793804 succeeded; documentation-only main804 runtime
coverage, not21 telemetry or native savings acceptance.

Req2 11d5a4f CI37195027084 passed fmt/clippy/tests/debug and release builds on BOTH
Linux and macOS. Two independent native Req2 reviews requested changes, each with
actual V2 selected owned-group cleanup and child reap verified. Their raw resumed
usage/cost counters remain unattributed, never an incremental cost sum.

Verified Medium classes refined in Req3: actual consuming model/effort/tokenizer
observed separately from immutable requested settings; beneficiary links and
descriptive scoped metrics prevent hiding Goal/runtime shared provider overhead;
each credited reduction feature needs an omission-sensitive independent oracle;
mandatory rule/evidence delivery must be proven for every dispatch; controllable
path/session identity reuse and known-channel exposure invalidate causal claims;
and all-unavailable native tokens cannot close native token comparison acceptance.
Actual Issue21 requires estimated cost where available/configurable, so optional
unavailable monetary/cache fields do not become invented mandatory measurements.
Cost/cache savings still require their own complete qualified comparable fields.
These changes match Product40.10 and actual adapter/native provenance gaps; no
source, model/default, permission, hook or credential change. Req3 review pending.

Req3 38c3786 exact CI37196041124 passed all Linux/macOS fmt/clippy/test/debug/release
steps. Two independent native Req3 reviews requested changes with verified actual
owned cleanup. Verified remaining Medium classes: survivor-cohort bias, an untested
feature hidden inside a whole-lane result, exclusive uncached input mislabeled as
logical-context reduction, and a mandatory-set oracle computed by the reduction
loader itself or satisfied by stale delivery. Req4 makes all required repetitions
part of the claim denominator, requires every exercised feature's sensitive oracle,
names inclusive/exclusive token definitions, and fixes reduction-independent policy
and actual per-dispatch sent/retained delivery provenance. Native self-loading is
separate and unverified, never a substitute or an OS internal-attention claim.
Unknown effort/model produces honest non-comparability rather than requiring a
positive causal savings outcome; required token/quality/safety evidence remains.
Req4 review pending; no runtime/source or benchmark acceptance is claimed.

Req4 363b91b CI37196429498 passed all Linux/macOS steps. Both independent native
Req4 reviewers completed with actual owned cleanup verified, each requesting one
verified Medium correction. Req5 requires registered append-only plan/run history,
including aborted/superseded unfavorable plans, justified successor scope and no
silent unchanged-source plan shopping. It also corrects Req4's overly broad
failure/safety wording: measurement-invalid is NOT task/safety/quality failure;
actual execution and verified safety/quality have distinct evidence/score axes.
Either lane's measurement invalidity blocks whole-run claims without awarding the
other quality credit. Unknown model/tokenizer cannot satisfy qualified token units;
unknown effort alone permits honest non-causal reporting. Real closure/report
mutants cover the new distinctions. Req5 review pending; no source changed.

Req5 3548f08 CI37196835048 passed all Linux/macOS steps. Both independent native
Req5 reviews completed with actual owned cleanup verified and requested changes.
Verified remaining Medium classes: own runtime mandatory non-delivery cannot hide
as measurement unverified; absolute oracle miss in either lane is not automatically
a relative Context Efficiency quality regression; and unchanged-source plans need
pooled history under a predeclared stopping rule. Req6 separates actual absolute
safety from paired oracle detection limits/regression, defines strict own-delivery
loss, and pools same-source/configuration valid repetitions without deleting invalid
attempts. Fixing source/configuration creates a separate narrower claim, with prior
history retained after fixture cleanup; unregistered runs cannot close acceptance.
Requirements6 review pending. No source/runtime, benchmark or MVP completion claim.

Req6 f24862a CI37197492421 passed both Linux/macOS fmt, clippy, tests and
debug/release builds (individual job steps inspected). Two independent
native Req6 reviews completed with actual owned cleanup verified and request_changes.
Verified remaining Medium classes: valid-only pooling could hide attempted invalid
repetitions; a known-regression-tuned fix needs fresh held-out sensitive quality
items; and narrowed benchmark settings must not leave the shipped default with an
unfixed regressing feature. Req7 pools ALL attempts and fixes stopping before runs,
preserves unknown/failure denominators, separates independently evidenced environment
reasons, requires independent held-out checks and actual default-configuration pins.
Default-off never waives required MVP producer acceptance. Requirements7 review
pending; no runtime/source or native benchmark acceptance is claimed.

Req7 3bceca6 CI37199271288 passed both Linux/macOS fmt, clippy, tests and debug/
release builds (individual steps inspected). Independent native Req7A returned one
Medium on initial certifying oracle authorship; Req7B approved with no findings.
Both completed with actual owned cleanup verified using the hardened private v3
runner. Root synthetic startup-record/thread-start failures had already proved
owned selected-group cleanup/reap before error return; this is harness evidence,
not production detached-descendant conformance. Req8 applies independent creator,
pre-run registration and feature-author exposure rules to ALL certifying plans;
known tuning-exposed items remain regression evidence only. Shared runtime defects
are not environment exclusions merely because both lanes fail. Requirements8 review
pending; no implementation, native benchmark or MVP acceptance is claimed.

Req8 e001483 completed two independent native approvals with no findings and actual
v3 owned cleanup verified. Requirements approval covers no design or implementation.
Exact CI37199736228 is RED: macOS lib86 passed/1 failed/1 ignored; old
adapter::inspection::tests::each_output_stream_has_a_causal_size_failure at442
observed stdout202.449042ms with byte-budget failure and stderr261.674333ms TimedOut.
Ubuntu fmt/clippy/tests/debug build passed, release build cancelled by fail-fast;
macOS builds skipped. Original failed log retained, no rerun or invented PS cause.
The narrow cap-test component already being integrated with #55 is the related
pending source gate; a later green result alone does not erase this failure.
Design1 starts from approved Req8 and actual main/native/context inventory. Its
private operation/measurement/epoch/producers, real controls, native benchmark and
source/security gates remain unimplemented and pending.

Design1 ab57c77 completed two independent native request_changes reviews, both
with actual v3 owned cleanup verified. Verified causal gaps: cumulative snapshots/
status-final views could double count; telemetry capacity/write failures could gate
native/context effects; unowned attach intervals could be misattributed; static
contract model facts could hide fallback; lane/repetition Project identities and
actual effective/default configuration pins were missing; stable-prefix/cache and
amplification-reference architecture was incomplete. Design2 corrects these, with
producer-record inventory and optional post-admission measurement projection,
one finalized per-event/per-field effective value, unknown unowned intervals,
runtime-derived configuration checks and fully separate lane registry identities.
It adds exact cache-independent/stable-prefix/reference consumers and causal mutants.
Optional high-water paging/cutoff/oracle answer commitments are adopted; no source
or native benchmark/profile/producer implementation is claimed.

Exact Design1 CI37201118890 is RED: macOS lib86 passed/1 failed/1 ignored;
the unchanged old inspection442 stdout overflow fixture instead reached TimedOut
at283.380875ms. Ubuntu fmt/clippy passed, tests cancelled, builds skipped by
fail-fast; macOS builds skipped. Failed log retained, no rerun or claimed causal
fix. The related #55 cap-test/source component remains a separate integration gate;
later green docs cannot erase this result. Design2 independent review pending.
