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

The existing `retry` API admits Failed+dispatch_started+no-Session with no native
outcome proof. A direct passing characterization confirms both that ordinary
observation does not replay and that explicit retry still closes it. This is an
unresolved #14 acceptance gap, not safe recovery evidence or a #41 API fix.
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
