# Issue 41 verification

Risk: STRICT for shared Workflow reservation ownership and release authority.
The owned preparation implementation is committed. Documentation review is not
runtime verification. Both first independent source reviews approved with Low
refinements; their verified fixes passed Source2 as recorded below. The narrow final optional delta passed both Source3 reviews. Historical failed CI remains failed; exact bab6 CI
is separately green and does not establish inspection-timeout causality.

## Provenance and formal gates

The original observed CI failure was Ubuntu run `37123017200` at `e67c38b`: the
concurrent-step regression observed zero adapter launches instead of one. macOS
was cancelled through matrix failfast, not an independently failing macOS result.
This does not establish the cause of the earlier Issue 8 native cleanup failure,
and no shared inspection deadline, death guard or uncertainty latch is relaxed.

Public requirements `42462bc` were approved by independent native read-only review
before implementation. Both independent Design 3 reviews of public `c516014`
approved the design with no blockers; their native owned-process cleanup is
verified. The owner review identifies a Medium alignment refinement: requirements
6/7/11 and verification must limit marker rollback release to the owning tasks-row
SnapshotChanged. The transferred branch was normally rebased onto merged native
Grok main `4851fcd`, preserving the reviewed design and changing no Workflow code.

The current documentation delta verifies that typed marker eligibility matches
both table `tasks` and the owning Task ID; parent/Record-version conflicts, untyped
marker errors and release CAS/termination fences retain recovery 14 reservations.
The Low design refinements specify a direct Store TerminalRecovery consumer for
its independently reachable fence, deterministic pre-refresh lifecycle ABA,
overlapping held-start binding rejection and the precise Project/Goal writer and
13/14 recovery-reference audits. Project status/list may reconcile a newly invalid
Project to Blocked; only their underlying Store snapshot queries are pure reads.
No code behavior is inferred from a review verdict. The narrow immutable native
requirements Round 5 at public `a480610` approved this alignment with no blockers
in 547.524 seconds of reported native API duration; owned process cleanup was
verified. Raw resumed-session token/cost attribution is unverified. The first
runner preflight aborted before launch because its interpreter lacked waitid;
the successful runner used `/opt/homebrew/bin/python3.14` with waitid/WNOWAIT.

Its three optional Low refinements were checked against actual source: coordinated
StateOnly writes Task before Record, but factual gate observations can write only
the Record and require Evaluating. The Running preparation test uses the former;
it proves the token-version fence, not an unreachable records-table marker error.
Other Task IDs are unreachable in this marker; direct classifier coverage carries
only defense-in-depth credit. Requirements, README and master now carry the same
retained #14 classes, naming Project/Goal metadata ABA, Record token mismatch and
unknown reversible evaluation. Workflow and Goal masters warn future progress
writers about shared Goal-version noise. A narrow immutable delta re-review must
verify these refinements before implementation. Round 6 at public `2113df3`
approved them with no blockers and verified owned cleanup. Its resumed native
API duration and token/cost counters have unverified per-round attribution.
Two optional Low precision fixes were verified: parent row versions fence
definitive publications and Session binding as well as the marker; coordinated
Record re-persist rejects at refresh before that point, or at the Task marker CAS
after refresh, and both release-token consumers must retain the changed claim.
These documentation fixes will be included in the immutable implementation review.

Normal merge `8a31f81` restored original public `c516014` ancestry after the rebase;
its committed tree was byte-identical to reviewed `a480610`. No further force
push is used.

## Required implementation evidence

Clean runtime head `1418b36` passed all 65 Workflow regressions, all-target
Clippy with warnings denied, workspace debug/release builds and the full SERIAL
workspace: 187 Rust tests and 2 doctests, with 2 installed-native Grok tests
intentionally ignored. Serial execution is constrained local evidence; default
Linux/macOS CI remains required. The first full-test escalation timed out before
approval and was retried once successfully; no test process started on that timeout.

Seventeen new preparation regressions drive actual synthetic captures/start and
independent SQLite connections. They cover both main capture awaits, both internal
invalidation branches, same/fresh observers, dropped owners, pre-commit competitor
barriers with identical proposed timestamps/context, actor overrides, immutable
assigned bindings, pause/cancel/recovery/ABA, parent and Record replacement,
release CAS/executor/Lost fences, typed owning marker rollback, definitive
publication conflicts, untyped marker abort and post-dispatch binding conflicts.
The existing EvidencePort race now verifies unchanged history/retries/context
before evaluating the same attempt. Hooks and timestamps are per Engine/fixture.

The first focused run passed 8/9: the held-start test incorrectly expected the
entire audit unchanged after the adapter persisted its factual owned Session.
It now verifies unchanged Task/Workflow/context plus exactly one `session.saved`
event. Terminal and stale binding fences overlap, so no single-fence credit is
claimed. The first full Workflow run passed 63/65; two tests matched an obsolete
diagnostic string. Recovery now names #13/#14 from the durable phase, and tests
verify explicit recovery rather than deriving ownership from human reason text.
All 65 subsequently passed. Test-only `46dd6f0` orders durable retained-state
assertions before diagnostics for causal release-CAS mutation attribution.

At #41's reviewed source, the `retry` API admitted Failed+dispatch_started+no-Session
with no native outcome proof. Its passing characterization established ordinary
observation did not replay but explicit retry closed it; this historical gap and
evidence remain in Git. The later limited #14 source candidate renames that test to
preparation_explicit_retry_refuses_unbound_dispatch_without_recovery_proof and
asserts refusal with complete snapshots. Current source qualification is pending,
and genuine #14 recovery remains open. The new non-TerminalRecovery closure guard
preserves the earlier #41 TerminalRecovery guards and their causal mutant oracles.
Independent native #5 integration review also exposed that successful Workflow
Session binding writes an unchanged Task through `put_task_tx`, incrementing its
raw version and invalidating provider admission. Source inspection confirms only
the Workflow `session_id` changes after the marker; no existing Record-only API
permits Running native binding with the required guards. Shared binding integration
is pending in #43, and this issue claims synthetic adapter coverage, not native Workflow
completion acceptance.

Controlled source-capture/start suspension must prove passive same/fresh-Engine
observation, exact committed Record-version ownership, no reserve-loser release,
metadata-preserving eligible release and conservative retained conflicts. Direct
Store and actual Engine consumers must distinguish causal fence mutants from
masked defense-in-depth cases. Every native/fixture operation uses an isolated
owned repository/Store and bounded synchronization. Committed targeted and full
Workflow/shared-state regressions and fmt/clippy/debug/release passed as reported
above. Exact Linux/macOS CI and immutable independent source fix/re-review remain
pending; no runtime success is inferred from formal documentation approval.

The detached `46dd6f0` mutation worktree compiled all 18 candidates. Fifteen actual
consumer mutations were assertion-killed (including two combined defects), and
the exact owning-ID classifier mutation was killed only by its direct unit test,
with defense-in-depth credit. Pre-commit loser token alone and Engine-only
TerminalRecovery-fence removal survived because Record-version and Store fences
mask them; no kill credit is assigned. Restored source was clean and all 16 then-
present preparation regressions passed. A separate post-refresh Record replacement
test pins its actual typed owning Task-row marker error, so token-version credit
can be verified independently at both capture timings.


Test/docs head `5835702` added the separate post-refresh Record consumer. Its
commit title did not distinguish the new test from documentation. That targeted head passed all 17 preparation controls and all-target
Clippy. M19 independently removed the Record-token-version check for the
post-refresh consumer: it compiled and failed the durable state equality,
confirming actual typed owning Task-marker rollback does not waive the changed
Record token. The earlier M04 independently killed the pre-refresh consumer.
Across 19 compiled runs of 18 distinct operators: 16 actual consumer kills from
15 distinct operators, 1 defense-in-depth unit kill and 2 documented masked
survivors. M19 repeats M04 against a separate post-refresh consumer; it is not a
second operator. Exact final restored source was clean,
17 controls passed, and the detached mutation worktree was normally removed.
The machine-readable [mutation ledger](issue-41-mutants.json) preserves local
committed heads and outcomes. No compile error is counted as a kill.


## Independent source review and exact CI

Both immutable public `888d86d` SourceReview1 reviews approved with no Critical,
High or Medium findings. Owner session `2e78ed32-c15e-40b6-8009-349b5d013f1f` and
observer session `b8c5653e-49e2-4a30-9492-2a345b3752b3` completed; owned cleanup was
verified. Their inputs were the verified public thirteen-file source bundle and
immutable diff. A later attempt to add adapter/Git excerpts failed before replacing
that bundle; those excerpts were not silently claimed as reviewed inputs.

Verified Low fixes accept unbound-to-bound metadata only for release of an
originally unbound token, preserve strict dispatch pins and definitive-publication
conflict recovery, report release-not-performed truthfully after terminal recovery,
remove stale line references and distinguish mutation runs from operators. A new
actual caller fixture covers assignment before and after refresh. The held-start
fixture now asserts durable dispatch intent before observation or diagnostics;
its marker-order mutant was rechecked as M06-r2 against that causal assertion.

Exact `888d86d` CI run `37169413513` failed on macOS. fmt and Clippy passed;
all 92 library tests, 19 adapter tests and 5 CLI tests passed. Context ran 5 passing
and 11 failing tests; the primary bounded-Git diagnostic reported
`native process inspection timed out`, followed by uncertainty-latch cascades.
Linux tests and debug build passed, but the release job was cancelled by matrix
failfast, so neither the cancelled matrix nor macOS is a green final gate.
No deadline, uncertainty latch or test concurrency was relaxed and no historical
root cause is inferred. STRICT shared inspection follow-up #46 remains separately required for shared
inspection/native integration; #41 readiness depends on its own exact reviewed
source and new exact CI. #43 native binding and #14 recovery remain explicit.

The first Low-fix targeted run at `2b7f480` passed 17/18. Its new diagnostic
assertion incorrectly expected the owner-local release wrapper at post-refresh
terminal recovery, where marker publication rejects after eligibility is disabled.
The fixture now checks the release diagnostic only before refresh and verifies
unchanged recovered state at both timings. The final refinement asserts absence
of the actual release wrapper after refresh rather than the old removed wording. No production boundary is weakened.

## Low-fix controls and mutation attribution

Committed `7a8c0e5` passed all67 Workflow tests and the unchanged default-concurrency
workspace: 189 Rust tests plus2 doctests;2 installed-native Grok tests remain
ignored. All-target Clippy with warnings denied, fmt check and workspace debug/release
builds passed. Local default concurrency
is now explicit, rather than relabeling the older constrained serial run. The
exact historical macOS CI failure remains failed; fresh exact CI is still required.

M06-r2 recompiled the marker-order operator against the strengthened actual
adapter-start await. It failed the durable `dispatch_started` assertion before
any human diagnostic, establishing causal marker-order credit independently of
its first round's wording failure. M20 restores strict original-None binding
equality and fails actual fresh release (`active.is_none`) after a concurrent
pre-refresh assignment. The post-refresh control retains the definitive-publication
conflict. Both compiled; restored committed source passed18 controls and its
clean detached mutation worktree was normally removed. Across21 compiled runs
there are19 distinct operators:18 consumer kills from16 distinct operators,1
defense-in-depth unit kill and2 masked survivors. Repeated M04/M06 runs are not
new operators. The public patch artifact and SHA256 ledger preserve each exact
mutation; no compile error or overlapping lone fence earns kill credit.

Owner/observer Source1 Low fixes are verified against source and actual consumers:
original unbound binding release, truthful terminal-recovery diagnostics, current
function-based impact audit and precise gate/mutation provenance. Independent
immutable scoped Source2 must confirm them before readiness; #14/#43/#46 limitations
remain explicit and no native Workflow completion acceptance is claimed.

## Source2 and final precision delta

Both independent Source2 native sessions completed approve at public `bab6f54`,
with no Critical/High/Medium code findings and verified owned cleanup. Raw resumed
native API/token/cost counters have unverified per-round attribution. The supplied
immutable delta/full Workflow and Store plus exact fixture/regression excerpts
confirmed release/marker/definitive boundaries and public mutation hashes.
`7a8c0e5..bab6f54` changes documentation only; production and tests are byte-identical,
so the67/default189 controls cover the reviewed code.

Optional findings were verified: impact names the actual changed recovery tests
and Evaluating/Interrupted classifier, master records late binding, the offset4
comment names assigned-binding immutability before Task CAS, and the diagnostic
fixture checks the actual wrapper absence after refresh. Observer's assessment
called the late-binding rejection TaskCAS; owner identified the earlier immutable
binding ensure correctly. No safety behavior changes. Final scoped delta review
and targeted consumer verify these refinements.

Exact `bab6f54` CI37175637709 has both required contexts SUCCESS:
`check (ubuntu-latest)` job111357558406 and `check (macos-latest)` job111357558917.
This new head run is separate from failed888 run, no old job rerun. Low fixes do
not explain/repair ps timeout. #46 is a separate required follow-up, not an
automatically inferred prerequisite for merging the independently green41 head.

Committed `d124c21` targeted terminal-recovery consumer passed (0.13s), fmt and
all-target Clippy passed. Its new post-refresh absence-of-wrapper assertion was
causally tested by compiled M13-r2 (same any-marker-error-eligible operator M13):
it fails that actual assertion, while restored exact source/control passes. The
clean detached worktree was normally removed. Total22 compiled runs retain19
distinct operators:19 consumer-kill runs from16 operators,1 unit and2 masked.
This final refinement changes tests/comments/docs only; reviewed production
workflow.rs remains byte-identical to `7a8c0e5`/`bab6f54`. Broad local runtime gates
are reused from that actual tested source; final exact CI runs all required checks.

## Final source gates and documentation outcome

Exact immutable public `ba70c3f` Source3 owner session
`0154607b-2dc1-4b7a-9205-d0238b447aca` and observer session
`6e7cfdba-a79b-4b78-8474-123d71093e7d` completed approve, with no Critical/High/Medium
findings and verified owned cleanup. Both inspected the byte-verified public
optional test/documentation delta and actual causal-consumer excerpts. Their
merge-gate concerns about unobserved current CI were checked independently:
exact `ba70c3f` CI37176902117 completed SUCCESS on both
`check (ubuntu-latest)` job111361302972 and `check (macos-latest)` job111361303146.
No production/test bytes changed after this reviewed and fully CI-tested head.

Optional final document precision is verified: each mutation now records its
actual committed parent/base and full production blob identity; restored controls
name18 preparation tests separately from the final1 terminal consumer. Master
explicitly says eligible release returns the preparation error and only a later
ordinary step reserves again, with active-owner/releaseCAS guards unchanged.
Final evidence/master/ledger-only delta needs independent direct inspection and
its own exact CI; these outcome updates do not claim a new native source review
or a self-referential runtime check. The prior888 CI remains failed and #14/#43/#46
remain limitations. PR-required contexts are the two names above.


## Reviewed Issue46 integration and meaningful current gates

Final pre-integration `63bbd1d9b918d06f37c18654a28ff877208b14bd` exact CI37177404219 remained red: Ubuntu job111362800381 succeeded, macOS job111362800524 failed only two Grok fixtures at nativeexecute line277/authinventory line413 with SessionLost/native process group cleanup failed/native process inspection timed out. Library93, adapter19, CLI5, Context16 and Git18 had passed before Grok7passed/2failed/2ignored. No failed-job rerun, deadline/latch relaxation, or causal explanation is inferred from earlier green ba70/bab runs.

Issue46 PR50 merged reviewed source as main `80452f4169806e507b2ae13faab20b24d2f93de3` after two independent Source2 approvals and exact final ec8c14a Linux/macOS CI37190081343 success. This branch normally merged that main at `5bb5a0f91ad317983e069b12d3c45eb6240d511f`, preserving its published ancestry. New bounded selected-group observation is an actual dependency integration, not a rerun of the old failing head. No Issue41 Workflow/test source changed: workflow.rs blob `294c4629e55d3bc0b7b7be1e451835aff1418e6b`, workflow/tests.rs `8ad306f922676369afc34dc4997ef445e0d2cac4` match the reviewed pre-integration head; inspection.rs blob `befa637bbe4379c629ffc6a9f200ea7abdc721a5` matches normally merged reviewed main. Both independent component source gates and their exact scope remain attributable; this does not assert a new native Workflow completion review.

At clean committed5bb5a0f, full default-concurrency debug and release workspace suites both passed201Rust tests+2doctests: lib105, adapter19, CLI5, Context16, Git18, Grok9, Project16 and State13. Three explicit ignored entries remain (two installed Grok/auth acceptances, one sanitized child-only entry invoked by its ordinary parent). The original67 Workflow controls are included in the library target. All required fmt, macOS all-target Clippy-Dwarnings, non-test debug and release builds passed (clippy2.33s/debug0.09s/release5.67s). These gates exercised the actual Workflow/Context/Git/Generic/Grok consumers against the integrated helper with unchanged internal parallelism and budgets; no unknown process/latch guard was reset. Peer6 was compiling/focused-testing during portions of this window; its later full suite began after these owned suites completed, so no quiet-host or strictly isolated load claim is made.

Full logs are retained at `/private/tmp/rururunx-issue41-integrated46-debug.log` and `/private/tmp/rururunx-issue41-integrated46-release.log`. Historical22 compiled mutation runs/19operators remain scoped to their original source/fixtures; no new mutant credit is invented for the integration. Source46 has its own independently reviewed actual consumer/selection and mutation evidence. Existing #14 explicit unknown retry, #43 unchanged-Task native binding/version issue and native5/6/#16 actual integration remain open limitations. New source inspection does not retroactively establish the cause of either historical macOS failure.

The evidence-only outcome commit must publish a new exact head and pass both required Linux/macOS contexts before merge. No dummy commit/old-run retry is used. Root will verify the exact component/doc-only delta and final CI; no additional native installed acceptance or self-referential evidence review is claimed.


## Integrated CI failure and test-only observation correction

Exact integrated6982708 CI37191045378 failed on macOS job111403096190: lib104passed/1failed/1ignored; only `each_output_stream_has_a_causal_size_failure` failed atinspection.rs442. Its actual stdout overflow observation returned TimedOut with282.666125ms elapsed, before it could satisfy the fixture's exact size-category expectation. Selected/legacy/all-zombie/read-error/retained-EOF controls passed. Ubuntu job111403096068 fmt/clippy/test/debug build passed; release was cancelled by fail-fast, not an independent Linux failure or complete Linux success. Full original failed log remains `/private/tmp/rururunx-issue41-ci-37191045378-failed.log`. No same-head rerun, production fix or historical ps root-cause claim.

The production outcome was correctly Unknown: an oversized child's scheduling/throughput can reach the observation deadline before the byte-cap category. Test-only3d30399 renames that actual OS-wrapper control and permits only the exact stream-size InvalidData category or actual TimedOut; it still asserts Err/no accepted frame; the owned cleanup code path is unchanged but not independently asserted by this wrapper. It supplies no causal size-guard credit when another guard wins. Accepting TimedOut also removes incidental throughput/idle-only-sleep liveness coverage; that property avoids false Unknown and is not process-death evidence. No250ms/1MiB/default concurrency/latch/production code changes.

A separate private reader unit prepares a LIMIT-byte all-Z row prefix and reads a real owned TempDir File, adding the newline atLIMIT+1 before an unread live row. The resulting prefix separately validates as all-Z, while the actual Stream.drain must reject its cap before EOF. Both stdout/stderr category controls pass. This is prepared reader-boundary unit evidence, not an actual ps/pipe/group-death or bounded-memory-breach proof.

The separate [cap mutation ledger](issue-41-cap-mutation.json) records one new compiled shared omit-cap operator at31a5f7c, base3d30399: it failed the intended `reader accepted a truncated all-Z prefix` assertion (true is read progress, not a process-death verdict). The failing run reaches the first stdout case; no separate stderr mutant kill is claimed. Exact restoredbffe512 control passed and the clean detached worktree was normally removed. This is unit-only credit and does not alter the historical22-run/19-operator Workflow ledger or retroactively unmask Issue46's two former size runs.

At clean3d30399, default inspection9/9 passed0.26s; full default debug and release workspace both passed202Rust+2doctests (lib106 plus unchanged other targets and three explicit ignored entries). fmt/all-target clippy-Dwarnings passed1.60s; non-test debug/release builds passed0.12/0.06s. Peer6 full default suites overlapped parts of this window, and two native read-only design reviewers were active; no quiet-host claim or altered internal test concurrency. Logs `/private/tmp/rururunx-issue41-3d30399-{debug,release}.log` preserve exact outcomes. The source prefix before cfg(test) mod tests is byte-identical to6982708, with SHA256f65a450568427116167fc7f003c091ff9fa68db0f3d50b0965776d237cd371cb; Workflow production/test blobs remain unchanged from the independently reviewed baseline.

Two immutable narrow independent source delta reviews must assess this actual private-test correction, truthful guard attribution and failed/restored ledger, followed by exact final Linux/macOS CI. No previously approved source component is claimed to close unrelated #14/#43/native acceptance. Normal public ancestry remains preserved.


## Final integrated source delta gate

Both independent native Source4 delta reviewers approved public immutable `c659eeb244a9cb119e18d47385ac2be84679c413` with no Critical/High/Medium findings. Ownership reviewer0711f83d-3d32-4847-9204-fb37d599932d and bounded reviewerfa90a4cb-9c58-4f79-b39f-d3c2be3ff837 completed with owned process cleanup verified. They independently received current full inspection source, formal Issue46 requirements/design, the exact test-only delta, cap ledger and their own completed prior review. No peer's current findings or executor conversation was supplied. Raw resumed-session usage/duration/cost meters have unverified round attribution and are not presented as this round's measured wall time.

Verified Low documentation refinements clarify unchanged cleanup versus asserted cleanup, historical wrapper test names and loss of incidental throughput coverage. The cap ledger now includes the verbatim panic excerpt, exit101 and test0.08s (compile20.68s separately). The hidden live-row consequence on a subsequent zero-length read is source reasoning, not an executed mutant assertion. Optional full-file prefix construction was deferred: the existing prepared private reader-boundary unit is honestly unit-only. The unit retains the unchanged250ms deadline; a future TimedOut there would be fixture scheduling, not causal cap evidence, and must be preserved rather than counted as a cap kill. No deadline, limit, serialization, latch or production change follows from these Low suggestions.

Exact c659eeb [CI37192243659](https://github.com/shuhei-suzuki/rururunx/actions/runs/37192243659) completed both required contexts successfully: `check (macos-latest)` job111406705564 and `check (ubuntu-latest)` job111406705695. Each passed fmt, all-target Clippy-Dwarnings, default workspace tests, non-test debug build and release build. This is the new meaningful test-correction head, not a retry of failed6982708 or63bbd1. Earlier failed/cancelled runs remain recorded above; their root causes are not retrospectively claimed.

This final outcome/precision delta changes only verification documents and ledger provenance. Production inspection and Workflow/test bytes remain the independently reviewed c659eeb bytes. Final exact-head CI is still required after publishing these documents; root must inspect the outcome-only delta before the normal merge. #14 unknown explicit retry, #43 native binding and actual native5/6/#16 integration remain separate acceptance boundaries.


## Reviewed cleanup-diagnostics dependency integration

Issue55 PR57 normally merged reviewed source at main `98d54ac955c111011693b571ff3e68ffac1b4a88`, after Requirements2/dual Design4 approval, verified Source1 corrections, dual Source2 approval and dual narrow Source3 zero-findings approval. Its exact final cd34bce CI37203944705 passed Linux/macOS and root directly approved the outcome-only documentation delta. The dependency adds bounded private Grok cleanup receipts and shared synthetic scoped projection; it preserves cleanup/PID/Lost/reservation/permission/Store/schema/process limits. The independently reviewed test-only cap correction is already present on this branch, so integration introduces no second inspection delta.

This branch normally merged that main conflict-free at `93c128df8313b1091622b26a15f43517da478de1`, preserving public ancestry. Workflow production blob `294c4629e55d3bc0b7b7be1e451835aff1418e6b` and Workflow tests blob `8ad306f922676369afc34dc4997ef445e0d2cac4` remain exactly the previously independently reviewed component. Inspection blob `cca26e8d9a86f9253ae5cca0f8c5f1f1f3a066ab` matches reviewed merged main including the prepared-cap unit/Unknown-wrapper test correction. All Grok production, helper and consumer files match reviewed merged main. No conflict resolution or new Workflow/native authority behavior is claimed.

At clean committed93c128d, local **default-concurrency** workspace DEBUG and RELEASE each passed220Rust tests plus2doctests, with8deliberate ignored entries:5owned synthetic child entrypoints,1existing synthetic entrypoint and2installed-real-agent acceptance tests. Required parent consumers ran; ignored installed tests are not native acceptance. Both builds, fmt and all-target warnings-as-errors Clippy pass. Original logs remain `/private/tmp/rururunx-issue41-93c128d-workspace-debug.log` and `/private/tmp/rururunx-issue41-93c128d-workspace-release.log`, with gate details in `/private/tmp/rururunx-issue41-full-gates.json`.

Exact93c128d [CI37204376108](https://github.com/shuhei-suzuki/rururunx/actions/runs/37204376108) passed both required contexts: Linux `check (ubuntu-latest)` job111442399018 and macOS `check (macos-latest)` job111442398878. This is a new reviewed dependency-integration head, not a rerun of the original final3d844d/63bbd/6982708 red jobs. Those failures and unavailable original causes remain preserved. Newly actionable receipts and passing meaningful integrated gates do not retroactively establish why the historical retained PID or inspection timeout occurred.

The finite final integration outcome commit changes only this verification document; root directly checks its delta and final exact Linux/macOS CI remains required. Prior22compiled-run/19operator Workflow ledger and separate prepared-cap unit mutation keep their original credit scopes; no new mutation or native source review is invented for the conflict-free merge of approved components. #14 unknown explicit retry/recovery, #43 unchanged-Task native binding, #51 environment admission and #16 actual multi-Project/native dogfood remain separate obligations.
