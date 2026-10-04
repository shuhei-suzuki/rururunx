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

Design2 dfaa2e0 exact CI37203856158 passed Linux/macOS fmt, clippy, tests and
debug/release builds (individual steps inspected); it does not repair earlier red
cap-test evidence. Both independent native Design2 reviews completed request_changes
with actual v3 owned cleanup verified and confirmed Design1 gaps closed. Verified
remaining Medium classes: unowned interval continuity needs positive durable facts
outside optional telemetry; reference apparatus must not warm lane provider caches;
mandatory producer delivery evidence must be distinct from its optional projection,
so dropping both re-delivery and its record cannot become measurement-unverified.
Design3 defines those source/cutoff/order/safety distinctions and actual consumer
controls/mutants. Native-configuration digests exclude credential/user-config files;
missing attestation stays unknown. Design3 review pending; no Rust/producer/profile/
native benchmark implementation or acceptance is claimed.

Design3 4644afa exact CI37204343866 passed Linux/macOS fmt/clippy/tests/debug and
release builds (individual steps inspected). Both independent native reviews
completed with actual v3 owned cleanup verified; B approved with no findings, A
verified all prior fixes and requested one Medium. Verified gap: PriceSchedule
thresholds need exact native billing-unit granularity; a per-request tier cannot
be applied to a multi-call turn aggregate. Design4 requires exact unit or proven
per-call tier partition, otherwise computed cost unavailable, with real cost/report
controls and mutants. Optional producer-version snapshot and lane cache-warmth
precision are adopted. Historical red cap-test evidence remains; merged55 actual
source integration is distinct. Design4 review pending; no Rust/native producer/
benchmark implementation or acceptance is claimed.

Design4 8a4016a completed two independent native approvals, zero findings and no
unresolved blockers; both v3 owned selected-group cleanup/reap checks passed before
review acceptance. Public delta packets included only each reviewer's own completed
prior findings, exact public old/new files and verified unchanged-context manifests.
This proves observed review completion, not native memory/private OS isolation or
full #9/#16 independent-profile conformance. Raw resumed-session usage/cost/duration
are attribution-unverified and are neither summed nor labeled per-round measures.
Exact CI37205121902 passed both Linux/macOS fmt, clippy, tests and debug/release
builds (all individual steps inspected). Historical reds remain unchanged.
Approval is design only. Private producer/Store/epoch integrations, coherent
versioned producer outcomes, actual native counters/benchmark and #16 remain pending.
No helper, unavailable measurement or fixture can close the whole telemetry issue.

## Initial source component: legacy integer storage boundary

Source head b47b773 (guard introduced in5a2f47a) contains only a pre-write signed-i64
range guard for the six typed Usage integers: input_tokens, cached_input_tokens,
output_tokens, context_pack_version, context_pack_size and repo_map_size. Values at
i64::MAX for the other five typed fields and the real cached_input_tokens zero
remain exact; each typed field at
i64::MAX+1 or u64::MAX
rejects before any Usage/audit or P/G/T/Session mutation. No clamp/cast/relabeling.
Legacy Usage, collection-time phase/round and arbitrary cache_metadata remain
UNQUALIFIED; this narrow component neither measures native tokens nor completes21.
Integers inside arbitrary cache_metadata, including Grok cache_creation_tokens,
are explicitly unguarded and unqualified pending raw-writer retirement.
Raw-writer retirement, typed operation provenance, bounded projections/queries and
all coordinated private producer/epoch/native benchmark gates remain pending.

Actual consumer inventory at the integrated main d87faec: domain::Usage;
AgentAdapter::usage and Generic/Grok implementations; Store::put_usage/usage/audit;
state integration tests and Generic adapter's Store persistence test; external
Grok usage tests read adapter counters but never write Store. There is currently
no production runtime caller of put_usage or aggregation/query/report/benchmark.
Unmerged Claude f9b671f claude/{protocol,session} distinguishes invocation counters
and resumed gauges; unmerged Codex native code codex/{protocol,session} preserves
turn identity/baseline/reset distinctions. Neither collector-provided labels nor
those unmerged sources constitute the planned managed operation/CounterContract.
Current main remains schema3; no telemetry schema or private owner is introduced.

Three initial fixture preparation failures are retained: exploratory missing Task
worktree binding; committed5a2f47a lacked paired branch; committedc0267f4 used a path
outside Project.worktree_root. Each failed before the intended numeric assertion
and earns no control/mutation/native credit. The corrected actual put_task/session
fixture at b47b773 passed all14 state tests. Full default workspace tests passed
221 Rust tests plus2 doctests,8 deliberately ignored native/owned entry points;
fmt, all-target clippy -D warnings and debug/release builds passed. Raw logs were
retained; no real native inference was run.

Six isolated compiled mutants each omit one field from the actual Store guard,
with clean private commits and a separate Cargo target. Every mutant reaches its
field-named 'overflow stored for ...' assertion after the real positive prerequisites
and fails; original source/tree bytes are restored and the same control passes.
Public patches and structured outcomes accompany this ledger. These are actual
legacy Store consumer kills, not qualified telemetry/report/native benchmark tests.
The normal main55/41 integration commit63a6a29 has CI37207493561 all steps green
on both OS; that precedes this new source and does not test its guard. Independent
source component review and new CI remain pending; no whole21 source acceptance.

Source1 component head41c60fe received two independent native approvals with no
Critical/High/Medium or blockers; actual owned cleanup/reap verified for both.
Three/two Low findings were verified: incomplete restoration hashes, missing
ordering mutant and overbroad guard/commit/zero wording. Each original private
mutation commit is retained in Git objects; its tree OID and path-labeled hashes
for state/mod.rs and tests/state.rs now accompany every check. The before/restored
control trees and both file hashes match exactly. An additional compiled mutant
moves the guard after actual Usage/audit commit. It returns the intended range
error but fails at 'overflow published data for input_tokens', exercising the
snapshot comparison omitted by field-removal kills. Its exact restored41c60fe tree
passes the same real control. No production Rust bytes changed for these fixes.

Trigger-head CI37209367789 succeeded for both OS, every fmt/clippy/test/debug and
release-build step inspected. Actual checkoutcd3f6e6da0f30f463fd77dca300ca5022b36dc0b
is the synthetic PR merge with parents d87faec and41c60fe. Its complete tree matches
the reviewed41c60fe tree, including every Rust source/test and Cargo/CI input blob;
this is tested-source provenance, distinct from run.headSha. Public structured
provenance accompanies this ledger. Prior failures remain. This is narrow component
approval, not qualified #21 telemetry/source/native benchmark or MVP acceptance.
Evidence/wording delta re-review remains pending.

Evidence delta Source2 at8918130: B approved without C/H/M, one optional wording Low
now corrected to state i64::MAX on FIVE fields and cached_input_tokens zero. A did
NOT produce a review terminal within the1200s harness bound and is not an accepted
verdict. Owned cleanup recorded sticky cleanup_unverified, then later selected-group
death confirmed with no retained leader/inspector anchors; acceptance remained false.
This is a failed native review/harness outcome, not source failure or full descendant
cleanup proof. Original failure is retained; no same-session result is fabricated.
Fresh independent review of the complete public component/evidence is required.
TriggerCI37211035771 at8918130 is green; actual checkout proof not inferred from its
trigger SHA. Earlier fully compared reviewed-source CI remains the qualified record.

Final complete-public-component Source3 at78676c9 received TWO fresh independent
native approvals, no C/H/M/blockers; both actual v3 owned cleanup/reap verified.
A's one optional Low is resolved in this final evidence: every control/mutant records
retained log SHA256, observed compile/test summary, executed count and the exact
observed failing assertion separately from intended_kill. All seven are actual
compiled consumer assertions, not compiler/setup101 credit; restored controls pass.
No Rust/test bytes changed since b47b773. Final evidence publication is metadata only.

CI37212685260 passed every individual Linux/macOS fmt/clippy/test/debug/release-build
step. Actual synthetic checkoutffa074e27f7e68db184f1747c05013f3142bcd36 parentsd87/786,
full tree equal to reviewed786 and all36source/test/build inputs individually compared.
The complete public provenance is retained. Final metadata CI remains a distinct gate.
This approves ONLY the legacy six typed integer Store boundary; whole21 stays open.
Qualified provenance/raw retirement/native counters/price/report/benchmark/epoch and
#16 acceptance are still mandatory. No failed review/cleanup or earlier CI failure
was relabeled. This scoped component may merge without closing the tracking issue.


## Staged legacy usage read identity component

Baseline main054aefd/schema3, under approved Requirements8/Design4. Store::usage
previously filtered row scope columns but returned decoded body without matching
Project/Goal/Task/Session identity, and omitted validate_scope. The component adds
these read-only checks; genuine legacy rows remain unqualified with unchanged values.
Six controlled body corruptions and invalid-scope checks exercise actual Store::usage,
Task/Goal/Project reads and unchanged history/owner/Session/audit snapshots. Source
verification and independent review pending. No native execution/aggregation, private
writer epoch, raw-metadata retirement, benchmark or whole21 acceptance is claimed.


Legacy read component source b1b643bc8ac3b43c4c0affefa6d9e9a29b9debd1:
all15 state integration tests PASS; fmt and locked all-target workspace Clippy
-Dwarnings PASS. Five compiled clean isolated mutants remove each row/body identity
check or the initial scope-shape check, and fail at the actual public Store::usage
consumer. Exact full tree and source restored, focused control PASS; public patches,
commit/tree/source/test/log hashes and observed assertion failures retained. No
compiler/setup failure counted. Full regression/builds/CI and two independent source
reviews remain pending; no qualified telemetry/native/whole21 acceptance claim.


Initial b1b643b source (artifact5b7d36e/source equal) also passed full DEFAULT host
workspace regression:222 Rust tests plus2 doctests,8 explicitly ignored. Debug and
release builds passed. Earlier Source2 #43 sandbox PS EPERM failures are a different
branch/environment, not these tests. Public zero-context patches reproduce identical
executed mutant hashes and retain original private patch hashes; apply with
`git apply --unidiff-zero`.

Historical b29aee8 attempt (SUPERSEDED AND WITHDRAWN, see below): read review
proposed agreeing corrupted row/body could both omit Goal while retaining Task. That
attempt added decoded Scope shape validation after identity matching, a Project query
control and complete row-column snapshot. Its checks failed during fixture setup;
none of that attempt earned consumer, mutation or regression credit.


The b29aee8 agreeing-malformed-scope addition is WITHDRAWN. Both its attempted
focused control and workspace test failed BEFORE the intended read assertion:
schema3 already has CHECK(task_id IS NULL OR goal_id IS NOT NULL). This is fixture
setup failure, not consumer/mutation/native credit. Its full run passed every prior
consumer but14state passed/1fixture failed; no rerun erased it. The extra body guard
was redundant for supported schema3 columns once all body identities are matched.
Exact production AND test blobs are restored to the initial b1b643b verified bytes;
restoration/failed-log hashes are retained. Initial5 compiled mutations,222 Rust+2docs
pass and debug/release builds apply through this explicit byte equality; no testing
claim applies to b29 itself. Unsupported/corrupted schema and qualified query/epoch
producers remain beyond this narrow component. Fresh independent source review and
exact-source CI still required. No new read/write authority or legacy waiver.


Legacy read Source1 f7457df: two independent native reviewers both identified a
verified Medium test gap for NULL Goal/Task columns, plus bounded wording/snapshot
Lows. Actual production comparison is correct; no new authority/policy proposed.
Clean follow-up301a885 normally incorporates merged main diagnostic f7baad and adds
legitimate Project/Goal/Task non-executor history, exact ordered selected rows, both
NULL-column/body-Some refusal directions, filtered unaffected views, complete usage
columns/all rows and global audit-row snapshots with unchanged owners/Sessions.
Withdrawn b29 paragraph is explicitly historical; static errors concern only identity
and requested-scope refusals; pre-existing serde decode detail remains pending safe
projection. No production Store byte changed from verified b1b.

At301a885 DEFAULT normal-host workspace236 Rust+2 doctests PASS,9 explicitly ignored;
fmt/all-target Clippy-Dwarnings/debug and release BUILD PASS. Release TESTS not run:
merged-main555 release failure remains known, cause/regression UNKNOWN. Nine compiled
clean committed mutants (four identity removals, request shape removal, two NULL
wildcards, two non-optional column reads) fail at actual public Store::usage consumers;
exact full tree/source restored and both controls PASS. Private collector string
expectation was corrected after retaining the actual nonoptionalGoal consumer failure;
no retry-to-green or compile/setup kill. Public patches/log/source/tree hashes and
exact all source/test/build inputs retained. Fresh two-reviewer Source2/current CI
pending; whole21 and qualified metrics/native/projection producers remain OPEN.


Source2 d937db3: TWO independent native APPROVE, zero Critical/High/Medium and
no unresolved component blocker. Both actual selected review wrappers cleaned up;
resumed native counters have unverified operation attribution and are not qualified
telemetry. Verified Low test-count arithmetic corrected mechanically to236 Rust+2docs
(238 total),9ignored; logs unchanged. Optional insertion-vs-index-order discriminator
strengthening deferred: ORDER BY sequence production unchanged and no ordering mutant
claimed. No source/test change after301a885. CI37232602381 EVERY step Linux/macOS PASS;
actual logged checkout2ad5aa018c2db51052af5689cdb3f3dd97dbd60c has parentsf7baad/d937
and complete tree equal triggerhead, including all38 source/test/build inputs.

Automatic review initially rejected native re-review by asserting private repository
export. Unauthenticated GitHub API proved private:false; all19 source/evidence/previous
review files downloaded without credentials and compared to exact published bytes.
Identical command retried with this evidence was approved; no workaround or private
payload used. Public proof retained. Final evidence-only metadata commit is manually
reviewed with all38 source/test/build blobs unchanged; final current CI still required
before normal limited merge. Whole21/MVP and native acceptance remain OPEN.


Next legacy Usage decode-error component under approved Req8/Design4: actual
public Store::usage body-type corruption reproduces a synthetic canary in alternate
Display through the serde source chain at test-only17915ed. The initialacb5 control
stopped earlier on outer static-text difference and earns no leakage kill credit;
its log is retained. Only this Usage decode maps failure to a static message with
no raw source chain. Task/Goal/Project corrupt fields, exact Display/alternate
Display and canary-free Debug controls retain schema checks and snapshot all usage/
audit rows and owners/Sessions. Debug may retain ordinary anyhow backtrace behavior;
no configuration override or bound on arbitrary backtrace is claimed. Whole21/
private projection/native/epoch/raw-retirement remain OPEN. Changed-source checks,
causal mutants, two independent reviews and current CI are pending.


Legacy decode projection changed source c44cbc7: full default normal-host workspace
237 Rust+2doc tests PASS,9explicitignored; fmt/all-target Clippy-Dwarnings and debug/
release BUILD PASS. Counts derived mechanically from retained11 target summaries.
Three compiled clean committed mutants revert projection, retain raw cause, or
format raw cause into text; each ACTUALLY leaks the synthetic canary through public
Store::usage and the real consumer detects it. Full tree/source restored and focused
control PASS. Public patches/source/test/tree/log hashes and all38 build inputs
retained; compiler/setup errors earn no credit. Other shared decode readers, legacy
valid raw metadata, allocation bounds, SQL error/backtrace details, native and full21
producer gates remain outside this limited component. Two independent reviews and
current CI still required before merge. No tracking Issue closure.
