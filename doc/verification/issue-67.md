# Issue67 isolated Grok fixture diagnostics

Related: #67, #51, #60, #6, #16.

**Current status: the limited test-only diagnostic slice merged through PR68 at
`768f84319cd2a73e14cd39336eb12d99e9be81a7`; Issue67 is closed. Exact final-source
CI37251714742 and subsequent main CI37252707188 passed every macOS/Linux step.
The earlier failed CI37250379945 and full Release failures remain retained with
UNKNOWN cause/regression.** The [observed final gate and merge record](issue-67-merged-status.json)
qualifies only this diagnostic slice. This QUICK test-only change makes failures
of existing isolated environment fixtures observable. It does not fix or
establish the cause of earlier full Release watchdog failures or main CI
ProcessStatus deadline failures.

`isolated_with` retains its env-cleared owned child, synthetic environment, two existing bounded readers, 60-second observation deadline, cleanup/reap order and original success predicate. Its failure message adds a finite observation classification, elapsed observation milliseconds and actual reaped exit status. No environment values or filesystem/config/credential contents are added. Observation I/O errors are reported as a category without their error message.

For the existing nine irrelevant foreign-change cases, progress lines use only boundary 0..2, case 0..8 and eight static stage labels. At most 72 lines per child (under 8 KiB) fit the existing 64 KiB stderr reader bound. Output uses the existing child stderr pipe; no new owner, process, thread, timer, file or poller is introduced. The last emitted stage indicates entry into an operation, not its successful completion or which nested activity caused a delay. Existing final child-completion assertions remain required. These are synthetic fixtures, never native acceptance or proof that every nested job is reaped.

Clean committed source verification is recorded below; two independent finite source reviews APPROVE with zero Critical/High/Medium and verified selected-group cleanup. Final composition/metadata CI and root checks precede limited merge. Original full Release failure observations at graph source `364139d5dedf35e154597417efb0c5e9ac7927fd` remain retained in the Issue23 composed1 ledger: checkpoint and resume parents each reached the 60-second watchdog with only `running 1 test` child stdout and empty stderr. No increase to deadlines, serial execution, ignored assertion, production policy, environment selection, native defaults or ownership protocol is made here.

## Clean source verification

Clean `87ec5eea34f93a49907b5dd941eb49121bd5e679` passes default-parallel full Debug tests: 378 top-level Rust tests and two doctests, 26 ignored direct entries; one nested ordinary witness child is counted separately. All 15 affected Release environment tests pass (14 ignored direct child entries, which are entered through existing isolated parents). Formatting, warnings-denied all-target Clippy, and Debug/Release all-target builds pass. Exact commands, result rows and logs' SHA-256 are recorded in `issue-67-quality.json`. This affected Release run exercises progress emission, eager exit_observed/elapsed computation and completion-assertion tolerance. At that initial verification head, failure-message rendering and the watchdog_expired/observation_io_error classifications were compile-checked only, not dynamically observed; its success does not explain, erase or fix the earlier full Release watchdog failures. No full Release suite success is claimed. No new tests mirror the diagnostic formatting or change a production assurance condition; existing actual fixture consumers and final completion assertions are exercised. Two independent source approvals and review-head actual-checkout CI are recorded below. Final composition/metadata CI and root checks remain required.

## Independent source outcome and precise limits

[Two source reviews and verified Low dispositions](issue-67-source1-reviews.json) APPROVE immutable `1bea366385c07f1186adc931d027ee391f7c8da5`, with zero Critical/High/Medium and verified selected-group cleanup. The complete changed-path list versus maincf8 contains exactly environment_tests.rs, this ledger and quality.json; the initial packet supplied all three full files but its diff label omitted the quality path. Tested87 to reviewed1bea changes only the two verification files. The private pub(super) isolated helper consumer in environment_workflow_tests.rs101 also preserves its original predicate and prefix; full Debug ran it.

The `finished` progress label marks entry to scope teardown after release, before locals are dropped. A missing next case's fixture line or missing final child_completed can indicate teardown. It is not a claim that Drops, the operation or all nested jobs completed. Successful runs capture/discard child output; they do not observe the parent failure message or watchdog/I/O-error classification. No forced-failure or causal mutation proof was claimed in that initial review round. The later explicitly forced diagnostic probe is recorded separately below and earns no assurance-operator credit.

[Review-head CI37248013333](issue-67-source1-ci.json) passes EVERY Linux/macOS step: actual686f9ed3 parentscf8/1bea, complete tree5602ae9 and all242 tracked blobs match. [Original code-head CI37247900919 failure](issue-67-code-ci-failure.json) remains: Linux all-step success, macOS Context2PASS14FAIL after one bounded process inspection deadline299506us with zero stdout/stderr and pending status, then concurrent/earlier uncertainty latches. Original cleanup reports kill requested then wait reaped; no all-nested-job claim. Cause/regression UNKNOWN. No rerun, serial execution, timeout change or assertion waiver. The later metadata green has identical source and does not explain or erase the failure. #60/#6 retain supervisor work. This cfg(test) lib module adds no diagnostics to that separate Context integration binary or its failing path; cause and regression, including possible indirect effects, remain UNKNOWN. Final metadata/composed CI must independently prove the exact actual tree before any limited merge. Whole native/F1/MVP acceptance remains OPEN.

## Latest main composition (two independent delta approvals)

Normal no-conflict main5b4 composition is clean tested `7634d7547a16dad543fde0a62e0055a60843091b`. [Composition provenance/main merge and final CI](issue-67-composition-provenance.json) records the separately approved limited structural DAG validation. All Grok adapter, Cargo and CI bytes remain unchanged from source1 review1bea; production delta versus new main is still only the cfg(test) diagnostic module. The actual shared Fixture constructs Goal::new's empty TaskDag, which both structural validators accept. No environment case edits its DAG. These static facts are not native authority or a failure-cause proof.

[Clean composed consumer gates](issue-67-composed1-quality.json): default-parallel full Debug382 top-level Rust+2doctests/26ignored PASS (nested ordinary witness1 separate); affected Release environment15PASS/14ignored, graph4PASS and State17PASS. Formatting, warnings-denied all-target Clippy and Debug/Release all-target builds pass. New composition justifies these actual new-head tests; the earlier failed full Release and first Issue67CI are retained, not rerun or erased. Those passing diagnostics exercised progress and completion tolerance only; failure branches/message rendering were then compile-checked.

[Prior final metadata CI37248732187 proof](issue-67-final-before-composition-ci.json): EVERY Linux/macOS step successful, actual7d202bb parentscf8/bd0a, complete tree/all245 blobs match. This does not qualify the new composed head. Two independent delta source reviews now APPROVE with zero Critical/High/Medium and verified selected-group cleanup. Exact final metadata both-OS CI/root checks remain required before limited merge. No full Release suite, supervisor cause/fix, nativeF1 or wholeMVP completion is claimed.

## Current source/composition gate and finite metadata delta

[Actual independent composition reviews](issue-67-composition1-reviews.json) both APPROVE at `9c7d4311b78eeff25aff77db7f35c7d470b4b925`, zero Critical/High/Medium, no unresolved blockers and verified selected-group cleanup. A has no findings; B's one verified optional Low narrows the Context failure paragraph: this changed cfg(test) module is not compiled into that separate integration target, so it adds no diagnostics to its failing path. Root cause/regression and possible indirect effects remain UNKNOWN, not excused by that static fact. Private-helper and related-Issue wording are corrected. No code, tests, Cargo or CI bytes change after these approvals.

[Clean composed test-head CI37249244094](issue-67-composed1-ci.json) and [composition review-head CI37249554165](issue-67-composition1-ci.json) pass EVERY Linux/macOS step. Actualc79f47ca parents5b4/7634 has the complete test-head tree and all266 blobs; the review-head proof supplies its exact checkout/tree/all269 blobs. CI tests Debug and builds Release only. This is no new failure-path, full Release suite or native cohort proof. Final metadata-head CI must independently match its actual complete tree before pinned limited merge.

Only a finite metadata delta follows the current approvals: sanitized actual review/disposition and CI proof files, this precise current status/limits and related-Issue/private-helper wording. Existing source gates, unknown failures, default parallelism, deadlines, predicates and historical Issue6 harness hold remain intact. No blanket native cleanup or MVP completion is implied.


## Actual watchdog diagnostic probe and restored control

[Committed forced-watchdog diagnostic probe](issue-67-watchdog-diagnostic-probe.json) starts from exact final5ca, intentionally parks the first checkpoint irrelevant-change child after `boundary=1 case=0 stage=fixture`, before `Fixture::new` and all nested Git/native fixture effects. The unchanged parent reached its original 60-second watchdog and emitted `observation=watchdog_expired`, `observation_elapsed_ms=60001`, actual reaped `signal: 9 (SIGKILL)` and that captured last stage. Existing cleanup/reap and both reader joins returned before the original failure assertion. This dynamically checks watchdog classification, failure-message rendering and captured stage availability.

The probe-only six-line park was fully restored in a committed clean detached worktree; its complete tree equals final5ca. The original same parent then passed in33.81s with unchanged deadline/predicate/cleanup. This intentional park is a diagnostic exercise, not an assurance mutant (zero operator credit), reproduction or cause/fix of earlier failures, all-native cleanup proof or native/F1/MVP acceptance. `observation_io_error` rendering remains compile-checked only. A stage names entry to an operation, not successful completion or the cause of a delay.

## Prior failed final CI and publication-time merge hold

[Final-head CI37250379945 failure and exact provenance](issue-67-final1-ci-failure.json) tests head5ca against main5b4. Both jobs actually checked out c526d6b; parents5b4/5ca and complete tree7b274cd/all tracked blobs match head5ca. Linux passed every step. macOS formatting and Clippy passed, but the library tests ended269PASS/2FAIL/23ignored; later Debug/Release builds were skipped.

The two failing Codex synthetic-owner tests each recorded an actual independent process inspection deadline: terminal-publication at session.rs5873 observed295241us (spawn1266us/cleanup12us), and owner-saturation at5605 observed261412us (spawn1641us/cleanup3us). Both had zero stdout/stderr, one WouldBlock read per stream, pending exit status and validation not reached; the inspector reported kill requested then wait reaped. These are two actual observations, not the earlier Context cohort of one observation plus sticky uncertainty failures. No all-nested-job or native ownership certificate follows from inspector cleanup.

Cause and regression, including indirect effects, remain UNKNOWN. No rerun,
deadline increase, serial test execution or assertion waiver was used. At this
publication point, the existing source/composition approvals and earlier green
runs did not override the failed gate: PR68 was draft and Issue67 was open.
Publishing the actual probe/failure records was a finite evidence update, not a
remedy for the failure. The subsequent exact final gate and merge are recorded
below. #51/#60/#6 and whole MVP acceptance remain open.

## Observed final gate and limited merge

Final evidence head `3d6ac5fb40f50239f0d0c69795f9c11247d263e1` recorded the
forced-watchdog probe and failed prior CI without changing the approved code.
Its distinct CI37251714742 passed every step on both hosts. Both jobs actually
checked out `1e3fc29444a0a2473e8531f2e3ea50f117e88bf4`, parents main5b4/final3d6;
complete tree `b671a403343dd2e99c0158b894e4fa60e6a5d949` and all274 tracked blobs
match final3d6. This is a final-head gate observation, not a cause/fix proof or
a retry of failed5ca.

PR68 merged on2026-10-05T01:45:50Z to main768; Issue67 closed on01:50:58Z.
The subsequent push CI37252707188 passed every macOS/Linux step at actual main768,
with the same complete tree. These workflows run workspace Debug tests and
Debug/Release builds; they do not run full Release tests. This status correction
adds no new code/test verification, native acceptance, complete nested cleanup
certificate or failure-cause claim. Historical HOLD fields in earlier immutable
evidence files describe their publication-time state and remain unchanged.
