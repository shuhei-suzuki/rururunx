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
