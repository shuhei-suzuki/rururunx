# Issue 60 inspection/reader component verification

Requirements4 normative diagnostic approvals, corrected source provenance; Design2 candidate; no design/source approval or implementation. Isolated main054
baseline, no production edits or new process tests. Frozen Issue51 final18c CI remains
red; first bounded native inspection timeout and derived latch failures retained, cause
unknown. No rerun, deadline/latch/permission/Unknown relaxation.

## Requirements1 findings verified

Two independent immutable public-source native reviews at d1dfb5c completed request_changes
with actual cleanup verified. [Disposition](issue-60-requirements1-findings.json) retains
all14 findings and raw result digests. Observation:4Medium+3Low; lifetime:2High+3Medium+2Low.
Verified exact static timeout/check sites, cleanup Ok after relinquished signal authority
is not verified reap, missing owner/admission/Drop-driver contract, undecided reader-kind/
latch/budget precedence and observed Cancelled/Panic taskfuture closure qualification.

Requirements2 proposed definitions before design (both reviews requested changes):64global job permits cover supervisor/
2readers/cleanup (4perop,max16), no unstarted uncertainty on capacity refusal; actual
anchors outlive caller/runtime, no async blocking Group Drop or unbounded fallback;
Git-specific failed group stays retained, common native Drop remains separately gated.
One existing250ms output/abortjoin budget, frozen primary result ordering and intentional
not_observed→stickyflag/latch are explicit. Late actual resource settlement can release
permits but cannot reset a previously latched Unknown. Observed Panic/Cancelled joins
prove endpoint termination, not reads/death. Fully settled and in-budget abort Context
positives must detect an over-conservative latch mutant.

Every inspector check/guard maps to a finite site/refusal and safe facts; status-error
relinquishment is distinct from cleanup success. First facts survive existing Drop retry;
safe retained Context failure rendering is required. Missing endpoint/setup/read/syscall
errors are injected/unit observations, not claims real ps naturally reached them.
Requirements2 independent delta reviews completed request_changes with cleanup verified.
Observation:1High+3Medium+5Low; lifetime:1High+4Medium+2Low. All16 findings were
verified against actual source/contracts and recorded in
[Requirements2 dispositions](issue-60-requirements2-findings.json). No approval inferred.

Requirements3 corrects the inherited Drop attribution and Git capture limit: group
cleanup success disables blocking Drop, reap timeout loses the unreaped anchor, and
observation failure after reap has reader-only gap. OUTPUT_LIMIT is64KiB, distinct
from inspector1MiB. Production selected ps is env_clear with no COMMAND_MODE override;
legacy is only a negative fixture. Prior GIT-OWNER-4 disposition is qualified accordingly.

The separate UNAPPROVED reader/driver draft proposes preserving shutdown FIRST cleanup on a preavailable
reserved non-poller lane, retains Unknown under total driver failure, and claims no
restart recovery. Git internal flag is separated from frozen caller publication;
Context626/662, Generic Lost/Failed and Grok receipt operands get explicit causal
controls. Native common ProcessGroup behavior stays unchanged. Bounded admission
uses the existing caller deadline, with safe active/retained counts; cfg(test)-only
private pools isolate destructive controls while production routes one singleton.
The draft forbids locks across native cleanup. Output_open not_observed would remain deliberately
Unknown, with immediate peer abort in existing ordered primary collection and no late
flag reset. Those behavior changes are not diagnostic implementation or approval.

Requirements3 formal scope is inspector diagnostic facts/safe error rendering ONLY,
with exact stage/refusal, EOF/counters/finite exit, cleanup-vs-relinquishment and original
kind/priority constraints. Existing reader behavior/flags/admission/driver/Drop and all
resource equations stay unchanged. Requirements3/4 results are recorded below;
source-aside correction does not change the approved real-inspector criterion;
Design1 request_changes recorded below; Design2 delta gates pending before source. Full60/shared availability/native16 are open.

Requirements-only CI37219380561 passes bothOS on unchanged production; source/reader/
availability acceptance is not inferred. Public actual-checkout provenance will be
recorded independently of trigger head. Apple libproc/sysctl source-only recon establishes
no bounded complete replacement or exact installed-XNU equivalence. Full Issue60 workload/
effect/delegation/settlement and native16 remain pending.

Requirements2-only CI37220390302 also passed bothOS, actual checkout
709f4aaba3797a2ef21e5513bcea8f30bd0104ea; unchanged production. No implementation,
shared availability or failed Issue51 final-context acceptance credit follows.

## Requirements3 narrowed results

At820a29e observation requested changes (1Medium+3Low); lifetime approved with3Low.
Both actual owned review cleanups verified. [All7 verified findings](issue-60-requirements3-findings.json)
are preserved with raw digests. Requirements4 adds the real inspector→resolver→Context
causal route: private per-operation executable/site seam and attachment mutant at
that actual consumer. The claim that the existing plan fabricates results is corrected
below after reading the complete source omitted from the prior packets. It enumerates reconciliation_error/unknown-dispatch/native-group
sinks, exact session.saved fields, actual try_wait counts and discarded Drop/retry
facts unavailable with no new log. Every baseline lifetime gap stays open.
Requirements4 independent narrowed delta gates pending; no design/source approval.

Req3 documentation-only CI37221599113 all individual required steps bothOS success.
Actual both checkoutc706885a7097b5d0de21918a9499dbad11285f97 parents054aefd/820a29e,
tested tree equals trigger. Unchanged production has no diagnostics/availability credit.

## Requirements4 approvals and source qualification

At26de8da both independent narrowed requirement reviewers approved, each2Low, with
actual cleanup verified. [Findings/digests](issue-60-requirements4-findings.json) preserve
the scope: no design/source/availability/full60 credit. Low fixes enumerate native
agent cleanup routes, exact gate status and labelled PERM injection.

Reading complete existing inspection.rs800–914 found a factual premise error shared
by reviewers and author: TestPlan::inspect returns the REAL inspect_with_prefix result,
not a fabricated ioError/observation. KillAndUnknown first sends actual KILL to the
owned group, then injects PERM; controlled shell feeds the real production selected
argv/env/framing/cleanup. Prior packets omitted this definition. Existing Context plan
therefore can reach real inspector fact construction; attachment assertion/mutant is
still required. Req3 findings are qualified rather than falsely credited as verified
bypass. The finite source-aside correction withdraws that false factual premise; the normative
real-inspector criterion is unchanged, so no new requirements round is inferred. Full
TestPlan source is included in Design1 packets. No production or completed-fixture claim.

Design1 proposes only typed bounded inspector facts and existing safe error rendering.
Requirements4 two approvals/noC/H/M and corrected source provenance are preserved;
no reader/flag/ProcessGroup behavior enters this design. Native design gates pending.

## Design1 verified findings

Atab2ac23 two fresh independent native design reviews completed request_changes with
actual cleanup verified. Facts2Medium+5Low; authority3Medium+4Low. [All14 dispositions](issue-60-design1-findings.json)
retain raw digests. Design2 fixes selected deferred-validation site, shared-only fact
attachment/pass-through wrappers, invariant real-consumer assertions and labelled
deadline alternatives, and UNIT status-error injection AFTER real direct Child reap.
Numeric callback/PID rescue is rejected; recorded testops protect synthetic mutations.
The formatter originates no fmt::Error; underlying status kind and unavailable counts
are explicit. Decorated IO text/outer raw errno replacement is disclosed; existing
inner ISDIR and unchanged live PERM controls remain. Master unclassified policy and
synchronous-git exclusion are explicit. No native/source behavior changes or evidence
credit from plans. Design2 independent delta reviews pending before implementation.

Design1 documentation-only CI37222832843 all required steps bothOS success, actual
checkout07a7d6b2d08329bcd25fb151fb45653b98ecf9ac parents054aefd/ab2ac23 tree=head.
Requirements4/source-aside CI37222227095/37222545671 likewise unchanged-source green;
provenance artifacts preserve actual both checkouts64c31906/87716538 and parents.
No diagnostics source/availability or failed Issue51 acceptance follows.

## Design2 approvals and fresh unchanged-source red

At201bd89 both independent delta reviews approve/noCritical/High/Medium (facts2Low,
authority4Low), actual cleanup verified. [All6 verified carryovers](issue-60-design2-findings.json)
pin final attachment to observe_command/inspect_command only, unchanged inner assertions,
Context anyhow chain and returned-IO-kind discriminator, shared failure-cleanup fact
helper and logical-clock-open UNIT injection after actual Child reap with recorded ops.
These are source-phase precision fixes; no native/reader/deadline/authority expansion.

CI37224052464 is RED: macOS Context whitespace_head_paths_staged_deletions_and_path_aliases_are_explicit
atcontext110 has one own inspection timeout; Context15PASS1FAIL24.29s, no derived
Context failures. Earlier lib120PASS6ignored, adapter19PASS,CLI5PASS; mac builds skipped.
Actual both checkoutca7bc11210e50ebb49954d7666e540b57530f096 parents054aefd/201bd89,
tested tree=head. Raw logs/step/checkout JSON preserved in private verification artifacts.
This is unchanged production, no diagnosis of cause, no rerun/waiver or availability
acceptance. New reviewed diagnostic source will be measured freshly; final red blocks
merge. Frozen Issue51 final18c remains red/draft. Full60/reader/driver/native16 remain open.


## First diagnostic source: implementation and impact, gates pending

Design2 has two independent approvals at201bd89. The six Low clarifications are
carried into the implementation and design without expanding ownership/reader scope.
Only macOS inspection source adds a typed finite diagnostic collector, static framing
refusals, common error attachment and private deterministic seams. The production
selected command, signal resolver, ProcessGroup/Drop/reap, Git reader scheduling,
Store, native controls, deadlines and byte bounds are unchanged. Complete/Stream/
validate inner error assertions preserve their original kind/Display; decorated
inspect failures deliberately replace raw IO text with static site text and preserve
kind, not raw errno. The directory-read ISDIR and valid-live resolver PERM remain
unwrapped. Deferred validation retains the masked validation result separately and
selects only the actually returned post-validation deadline or framing error.

The Context consumer adds an invocation-local test-only capture of the original
inspector IO kind before cleanup_group erases it into AdapterError text. This records
the first real plan observation, including the Drop-retry distinction, and has no
production field/API/authority. It permits independent timeout-versus-guard checks
rather than selecting its own acceptance path by the reported diagnostic site. The
actual plan still performs KILL then injected PERM and real shared shell inspection;
no fabricated snapshot/result substitutes for that path.

Planned controls include exact adjacent deadline/validation unit seams, settled-child
status relinquishment, actual inner directory read, safe maximum formatter, actual
Context uncertainty/latch transport and pre-effect missing-executable entry hop. Unit
injections/open logical clocks are labelled separately from native scheduling; there
is no exact read-throughput/elapsed/OS-permission credit. Source commit precedes
compilation/test/review gates. No passing source verification is claimed yet.
Design2 documentation CI37224052464 remains RED macOS: one own Context inspection
timeout,15PASS/1FAIL, unchanged baseline implementation. No old-head rerun or
cause attribution; fresh changed-source CI is required. Full60, reader/driver draft,
recovery14, native16 and Issue51 final RED remain open.


First-source70705e9 compiled successfully before execution. Scoped default inspector
controls passed17/17 (nine existing plus eight new). Initial all-target Clippy failed
on two test-only style checks (`int_plus_one` and nested first-kind capture); preserved
in `/private/tmp/rururunx-issue60-source707-clippy.log`. The following source commit
corrects those styles and completes finite-enum formatter branch coverage. No broader
suite, Context consumer, mutation or independent source approval is claimed yet.


Clean2ffb89b all-target Clippy and actual Context causal consumer passed. Full default
DEBUG passed229 Rust tests+2doctests,8 explicit ignored (128/19/5/16/18/13/16/14 by
target). Log `/private/tmp/rururunx-issue60-source2ff-full-debug.log`; no timing/cause
explanation for old reds. The next test-only commit keeps the synthetic status fixture's
actual settled Child proof in a cfg(test) Inspector field, so cleanup-guard omission
mutants cannot call a physical kill/wait during either the measured helper or Drop.
It also checks the no-spawn input guard. Production has no additional Inspector field
or changed kill/reap behavior. RELEASE/latest changed controls/mutations remain pending.


## Diagnostic source verification before independent Source1

Public clean6db2cc1 preserves both normal branch ancestry and baseline054aefd.
Default RELEASE at6db passed230 Rust+2doctests,8 explicit ignored; default DEBUG
at2ff passed229 Rust+2doctests before the test-only settled-fixture/input-control
addition. Latest6db inspector scoped DEFAULT controls18/18, fmtcheck, all-target
Clippy and debug/release builds passed. Actual Context first-cause/derived-latch
consumer passed separately at2ff and in the exact-restored6db mutation controls.
No default serialization, deadline/budget/latch reset, or old-red rerun was used.

[Mutation ledger](issue-60-inspection-mutations.json) records12 compiled assertion-killed
cases: actual Context attachment/site/byte/EOF/cleanup/Unknown authority controls6,
and labelled unit/seam status/kind/validation/byte/no-second-operation guards6.
These are12 cases, not a claim of12 distinct operator families. All failures reached
the cited intended assertions; no compile failure, timeout-only kill or masked no-credit
case is counted. Physical EOF and Inspector signal authority remain unchanged in the
metadata omission mutants. The sole cleanup-guard omission executes only the already
reaped Child unit fixture, with recorded test operations protecting helper and Drop.
Exact source was restored and committed before restored Context1/1 and inspector18/18
controls; the clean owned private mutation worktree was normally removed afterwards.

[Changed-source CI provenance](issue-60-inspection-source-ci.json): run37226585207
triggered6db; BOTH jobs actually checked outf536deba297dca5ef27353199e65573d150c073d,
parents054aefd+6db; tested tree32a498dd4fa4f20821367613c31b875ada9d654e equals
trigger tree. Linux/macOS every fmt/Clippy/default-debug-test/debug-build/release-build
step succeeded. This is diagnostic source coverage only, not native inference or
an explanation/repair of any earlier timeout. CI has no release-test step; local
RELEASE above is distinct evidence. Independent Source1 and final reviewed-head
CI remain pending. All earlier reds/Unknown evidence, frozen RED Issue51, shared
availability, reader/driver draft, recovery14, full60 and native16 limits remain open.


## Source1 outcomes and verified narrow corrections (re-review pending)

Both independent public immutable Source1 reviews at03cd688 completed with actual
owned cleanup verified. Facts reviewer requested changes (2Medium+6Low); authority
reviewer approved (5Low). [Per-finding disposition](issue-60-inspection-source1-findings.json)
retains overlap and raw hashes. [CI37227213174 provenance](issue-60-inspection-source1-ci.json): both-OS success;
actual checkout7439d9d parents054+03c, tested tree equals trigger tree. Reviewer launch-time CI blockers
were factual pending gates, not defects ignored after the run completed.

Verified Medium I60-SR-01: retained-writer fixture had replaced cleanup().unwrap with
a result-discard helper, weakening its separate reap check. Shared cleanup recording
now returns its existing result; production explicitly discards it preserving the
original first-error priority, while fixtures unwrap it before send/join/assertions.
Verified Medium I60-SR-02 and overlapping Low L3: accessible sink transport needed
assertions/disclosure. Existing Generic terminal-plan test now asserts actual facts;
a held-start Blocked write fixture targets launch-failure audit reason. A completed
successful synthetic Grok turn targets cleanup diagnostic and runtime failure while
requiring Lost/reserved/nontransport/unclassified receipt and clean=false. The older
predispatch-cat/unowned-read cases do NOT transport later cleanup facts: their earlier
result errors win by unchanged priority (grok/mod.rs result.err().or_else cleanup).
The reviewer correctly found coverage missing but its claim those existing cases
already exposed facts was overbroad. Reconciliation_error host-Git transport has no
current stage-specific plan injection; explicitly no direct transport assertion (unchanged forwarding is source-only), no new
production stage port or priority change to manufacture evidence.

Verified Low refinements: allocation-injection writer stays alive through its unit
failure; Open/non-target unit clocks have distinct10s fixture-loop watchdog and keep
5ms idle instead of busy-spin after250ms (production unchanged). Interrupted result
facts now mark pending via a shared fact-only helper, with prepared-result unit credit
only. Maximum nonframing formatter cases populate maximum counters/timing and all
site prefixes; incomplete-prefix check is independent of appended fact tokens.
Requirements status and master wrapped errno/text impact are corrected without new
normative requirements. Current attachment sites always know selected stream/none,
cleanup and kill stages, so optional unavailable vocabulary for these three fields
is unreachable; design documents every current site instead of adding unused states.
No old red rerun, deadline/Unknown/native authority or reader/driver correction. The
following commit precedes affected tests/mutants and own-session source re-review.


## Source1 fixes verified at555098d; Source2 pending

Affected default controls passed: inspector20, actual Context1, Generic terminal
and post-spawn audit2, completed synthetic Grok ACP receipt1. fmtcheck/all-target
Clippy/debug+release builds passed. The synthetic child is entered only by its
env-cleared owning parent; no installed native/auth/model acceptance is claimed.

[Delta mutation ledger](issue-60-inspection-delta-mutations.json) records10 compiled
assertion-killed cases:6 actual consumers (including2 audit-only projections) and
4 labelled fixture/prepared-result/private-clock unit guards. Broad attachment
omissions fail at preceding returned/status assertions; independent audit-only
omissions D09/D10 reach the actual scoped audit assertions while preserving earlier
facts. These are10 cases, not10 distinct operator families. The helper-result
refusal is injected only AFTER successful real child settlement, not an actual
OS wait error. Exact source restoration controls passed; both private worktrees
were normally removed clean after fixtures completed.

[Current full-suite outcomes](issue-60-inspection-source555-tests.json): DEFAULT
DEBUG passed234 Rust+2doctests/9 explicit ignored. Distinct DEFAULT RELEASE FAILED
at existing Grok executor pre-spawn inspection: lib133/Generic19/CLI5/Context16/
Git18 passed; Grok12pass1fail2ignored, later targets/doctests not executed. Actual
new facts froze deadline_loop_entry: inspector direct-child exit success and stderr
EOF observed, stdout EOF pending after43 reads/17bytes/42WouldBlock;250870us
observation,1012us spawn; validation not reached; inspector cleanup reaped by status,
kill not requested. Grok receipt marks pre_spawn uncertainty before native group
creation/dispatch. This distinguishes missing stdout EOF from missing observed exit;
it does NOT identify retained endpoint owner, scheduling/pipe-inheritance cause,
target-group death or repaired availability. The local suite exited101; no same-head
rerun, default-concurrency change, deadline relaxation or sticky-Unknown reset.
This red is retained alongside every old red and frozen Issue51; full60/runtime
availability/native16 remain OPEN. Source2 and final required both-OS CI pending.


## Source2 outcomes and verified disclosure precision

Both own-session independent Source2 reviews atd930b5a APPROVED, no C/H/M; actual
owned cleanup verified. [Raw result hashes/dispositions](issue-60-inspection-source2-findings.json)
record3 Low findings. Source1 fixes were independently confirmed. Remaining verified
Low corrections are documentation only: Generic preflight bounded-Git and Grok
pre-spawn/binding ownership/index Git, reconciliation host-Git and cleanup-selected
unknown-dispatch wrapper are unasserted source-forwarding routes. Incidental release
pre-spawn facts are uncontrolled observation, not assertion coverage. Positive
WouldBlock/status pending/interrupted counts, pending-stderr and Interrupted drain
wiring are residual unit gaps; the helper test is prepared-result credit only.
Conditional pending-stdout assertion and count invariants do not imply universal
increment/wiring omission kills. No requirement authority, Rust/tests or production
behavior changed in this disclosure delta. Scoped own-session docs re-review pending.

[Source2 public CI37230220373](issue-60-inspection-source2-ci.json): every required
step Linux/macOS SUCCESS; actual3969a84f parents054+d930 tree19f4cf=trigger. This
closes launch-time pending CI only. CI has default debug tests and release build,
no release-test step.555 local full DEFAULT RELEASE FAILED and later release targets/
doctests were not executed; no subsequent rerun or cause/availability claim. Review
source approvals do not turn that failure into pass. The root-owner partial diagnostic gate
disposition below governs this component only; full60, frozen RED51 and native16 stay OPEN.


## Source3 precision and explicit partial-gate disposition

Source3 atd2cb1ad: both own-session independent reviewers APPROVED/no C/H/M and
confirmed Source2 Lows resolved; actual owned cleanup verified.
[Raw hashes/dispositions](issue-60-inspection-source3-findings.json) retain4 overlapping
Low records. Verified docs-only corrections: pending gate decision now published,
cause AND regression status remain unknown, and existing native-child wrapper seam
is distinguished from missing host-Git stage seams. No Rust/test/requirement change.
Root directs manual final documentation diff review rather than another native
Source4 round: these are verified factual/decision records, no normative/code/producer/
native-success-condition change. Final metadata-head required CI remains pending.

The root development coordination owner, acting under the active user Goal, made
the following explicit partial-gate disposition on2026-10-05 JST:

> RELEASE555失敗を保持し、原因・回帰有無未確定／native availability未達を明記したうえで、sourceレビュー二名が診断限定sliceをapprove・CHM/同slice blockerなし、実際のread/signal/reader/cleanup/authority/期限・判定は変更なしとdiffで検証、直接consumer controls/mutantsと最新requiredCI両OS/actualcheckoutsource比較がPASSなら、診断限定部分を通常mergeしてよいです。CIはdebugtest+releasebuildであってreleaseTEST合格ではない旨をPRと公開ledgerに明記してください。whole60、reader/EOF可用性、native16/全体release readinessはOPENのまま。追加docs-only disclosure再レビュー完了／最終metadataheadrequiredCI sourceequivalenceも確認後、PR61noClosesでmerge＋通常cleanupを進めてください。Release redを無視して全体source/MVP ready扱いにはしません。

This is a conditional diagnostic-only partial-gate exception, not a RELEASE pass.
555 full DEFAULT RELEASE remains FAILED/incomplete, cause and regression status
unknown; no same-head rerun, guard/deadline/default-concurrency relaxation. CI
measures default debug TEST plus release BUILD, never release TEST. Final required
CI red blocks this partial merge. No Closes60; full60/readerEOFavailability/native16/
whole-release readiness remain OPEN. Earlier source approval does not silently
waive them. The disposition and outcome ledger preserve the failure permanently.


Finite final source/provenance check: diagnostic code555098d and both approved
Source2d930/Source3d2cb crate trees are identical. Final changes are factual
requirements/README status, Low precision and outcome/decision records only; no
normative criterion or Rust/test change. [Source3 CI37230809943](issue-60-inspection-source3-ci.json)
BOTHOS every requiredstep SUCCESS; actualc0fc2dba parents054+d2cb, tested
tree98f463d=trigger. Local555 RELEASEFAILED remains independent/incomplete, cause
AND regression unknown. Final published metadata-head required CI and direct root
documentation diff check are the remaining partial merge gates; their raw public
Actions/PR artifacts are recorded externally to avoid a self-referential outcome
commit/re-review loop. No whole60/native16 or release TEST success claim.
