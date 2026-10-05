# Issue #19 curated preparation custody integration

Status: queued-abandonment defect reproduced and fixed; the current d938 inventory
passed full local macOS Debug/Release and every Linux/macOS CI step. Earlier RED
observations and unknown causes are retained. Independent finite-source reviews
completed; limited merge disposition is being reviewed. Whole Issue #19 and
native readiness remain open.

## Immutable inputs and scope

Original Rust base main: `768f84319cd2a73e14cd39336eb12d99e9be81a7`.
The head also composes main `26f3a48fedce976b353a6a9fa899bc6fb13ca72a`
(PR70), adding only two Issue67 status documents. The observed difference
`f6b36d3c64313a73a6ce84b077a934053796d91f` to
`f6b18af1f27c56ee1d32957b8da5b2979588c946` is empty for crates, Cargo
and .github; no Rust or CI behavior is credited to that main composition.
Selected approved source: `857950094ae9732d31d6c11d773e7fe96adbae02`.
Historical source evidence: `a8aa95108bfe2fc098f5333f7bb5cf445c4c74a9`.

The initial composition imported only five code paths: Codex attempt.rs, custody.rs,
custody_tests.rs, mod.rs and session.rs. Every imported code blob is identical
to the selected source at initial code head94350f. The later verified abandonment
fix and bounded closed-runtime control are recorded below. Main attempt/session/mod blobs match the approved
Design5 baseline `3a12ef787e6bcf1cdb5f6732fdc18b2c482b783a` exactly.
Thus the main-to-source code diff is the selected mechanical component.
The broad parent PR #39's other 29 paths, including Context Pack, Context,
Store and schema prototypes, are excluded. Three selected design/evidence
documents are retained; README and master adapter design describe this limited
composition. No source approval of the broad parent is inferred.

Two independent native reviewers approved selected Source2 with no
Critical/High/Medium findings. Their bounded pending hardening limits and
rejected prior rounds remain in the [historical evidence](../design/issue-19-lifetime-component-evidence.md).
The current Create factory is test-only. Its unreproduced actual worker-spawn
error plus pending Note combination and hypothetical future pre-created handler
panic require qualification before future factory/handler activation.

## Regression history and required gates

Exact selected-source CI 37256266809 failed on macOS: seven library failures,
including three independent inspection deadline observations and four subsequent
Context uncertainty refusals; the eighteen selected mechanics passed. Linux
debug passed and release build was cancelled by matrix fail-fast. The retained
last-poll facts do not establish child status or output at the deadline, because
the loop checks its deadline before the next poll/read. Cause and component
contribution remain unknown. Earlier Grok/watchdog/inspection failures remain
recorded, and no same-head retry or timeout/parallelism/latch change is used.

This smaller composition must compile and pass its own focused Debug/Release
controls, workspace regression, fmt, warnings-denied all-target clippy,
Debug/Release builds, relevant compiled mutation controls, two independent
composition reviews and exact final-head macOS/Linux CI before a limited merge.
Any later green run records its own observation and does not explain old failures.

No ordinary native capability, ready backend, native cleanup certificate, private
producer, schema upgrade or full parent acceptance is added. Production custody
selection remains None. The closed resource factory itself creates no native/Git
subprocess. Fixture setup does execute git init/add/commit, and checkpoint setup
uses real Git/process inspection; those consumers can contribute shared host load.
No causal contribution to the failed regressions has been established.

## Actual curated code checks

Clean code head `94350fbb55d35f305cbdc3ff6b35d3c6a74e23f3`, tree
`02df0958f193266e321f91615db62e9ae5dd8149`, was committed before execution.
Rust 1.91.1, locked dependencies, default test parallelism and original internal
deadlines were used. The first sandboxed attempt passed16 controls and failed2
checkpoint setup controls with an explicit inspection spawn PermissionDenied.
That log is retained separately. A host-authorized execution subsequently passed
all18 selected Debug controls in3.70s and all18 Release controls in2.75s.
The broader affected Release Codex suite passed121, ignored1, in49.42s.

The host full default Debug run failed: library282 passed,7 failed,23 ignored,
73.63s; later workspace targets were not run. All18 selected mechanics passed.
The failed Grok consumers were:

- live_reference_change_rejects_actual_start_before_spawn: expected environment
  denial, received Timeout: Git ownership preflight timed out.
- live_reference_change_rejects_actual_resume_before_spawn and
  owning_reference_change_after_snapshot_rejects_resume: actual preflight Timeout,
  with nondispatched/not-attempted receipt facts.
- owning_reference_change_after_snapshot_rejects_start: expected StateConflict
  prefix failed; the underlying failure string was not printed.
- dispatched_clean_receipt_reaches_actual_supervise_stages: Failed versus expected
  Lost, with nondispatched/not-attempted receipt; underlying cause unknown.
- unrelated_foreign_changes_do_not_revoke_checkpoint: original60001ms watchdog,
  last finite boundary1/case6/resumed_terminal; actual child SIGKILL/reap.
- unrelated_foreign_changes_do_not_revoke_resume: original60002ms watchdog,
  last finite boundary2/case6/checkpoint; actual child SIGKILL/reap.

Those stage facts do not establish the blocking operation or complete nested
settlement. Cause and contribution of this component remain unknown. This head
was not rerun to seek a green full regression. Fmt, all-target clippy with warnings
denied, Debug build and Release build passed separately. Full Release workspace
tests were not run; the affected Release controls are a separate scoped result.
Full Debug log SHA256:
`c0e67616f2b86a37c163cb61e5d7dbd045fb9cc616c0b3f3b2e7e1ea57452ce3`.
Affected Release log SHA256:
`2a1c5273f56170517e5775ac25c895be3151a682303ed8fa9d148542589bea6e`.
The private check manifest is `/private/tmp/rururunx-root-custody-host-checks.json`;
failed sandbox artifacts remain separate and are not counted as passing gates.

## Actual composition mutation controls

An independently owned detached worktree based on the exact94350f code executed
the same16 distinct boundary operators against this smaller assembly. Each mutant
was normally committed before testing, compiled, and failed at its intended
consumer assertion. These are16 operators verified on this composition, not16
new operators in addition to the historical16. The mutants remove/alter:

| Boundary | Actual assertion consequence |
| --- | --- |
| Armed caller revocation | Caller Drop leaves jobs unrevoked |
| Endpoint generation | Foreign endpoint note succeeds |
| Handler revocation check | Revoked queued notes change the count |
|4096-byte metadata cap | Encoded +1 metadata succeeds |
|64-entry inbox cap | Over-capacity note succeeds |
| NotCreated worker credit | Actual no-created completion cannot replenish |
| Registry sweep | Same adapter reaches32 retained entries |
| Unknown worker accounting | Required unresolved slot count is not observed |
| Worker Begin wait | File opens before installed Begin |
| Held entry retention | Resource-owning error loses the registry entry |
| Late worker observer | Pending worker observer is discarded |
| Opaque creator error accounting | Unproved3-slot reservation refunds |
| Bookkeeping versus resource hold | Original zero-effect removal is stranded |
| Reply after retirement | Actor freezes uncreated Held instead of Fresh |
| Metadata versus effect request | Queued fixed Note freezes genuine no-work |
| No-work registry restoration | Exact previous Control is not restored |

After all16 compiled assertion kills, the owned mutation worktree returned clean
to94350f and the identical complete tree02df0958; all18 restored controls passed
in3.18s. All owned mutation cargo commands exited and were waited; no Root
mutation command remained live before normal worktree removal. This is not a
certificate of all delegated/native resource settlement. The per-mutant
commit/test/exit/log-hash record is retained privately at
`/private/tmp/rururunx-root-custody-composition-mutation-verified.json`.
This verifies selected mechanical test sensitivity, not native backend acceptance.

## Later parent observation

The parent evidence-only head a8aa951 has unchanged approved selected code.
Its distinct exact CI37256989270 also failed on macOS: library315 passed with
23 ignored (all18 mechanics passed), followed by context5 passed/11 failed.
Three context consumers reported inspection deadline observations301350/301853/
300488us; eight subsequently refused on the conservative context uncertainty
latch. The saved Pending/WouldBlock facts are prior polls, not facts at the
deadline. Mac builds were skipped; Linux every CI step succeeded. No same-head
retry or source correction was performed. This observation remains separate
from the original857 failure and the curated94350f full regression.

## First composition reviews and verified delta

Both independent reviews of public895a621 closed with command exit0 and verified
owned wrapper cleanup before either result was inspected. ReviewerA session
04d0dcad-0800-40ec-8242-875a3dc398dc approved with5 Low; reviewerB session
9d8ba90b-bd6c-46b1-b28e-628425eae632 approved with4 Low. Neither approved merge,
full regression, native activation or MVP acceptance. The exact first-round
findings and scope limits are retained in
[the review record](issue-19-custody-source1-reviews.json).

B's queued-abandonment finding was reproduced at test-onlye712cefb: actual
actor abort returned SessionLost and disarmed CallerGuard; a previously accepted
Create subsequently opened the file. The test failed at the intended file-effect
assertion after actual worker/custodian joins, not at a setup or deadline failure.
Reproduction log SHA256:
`baa6f29102b4b2f65b02a6432bbecfbf09dd0f6db1167e47b2950fe6f8657e15`.

Code fix876024296c79b6e2d6479112d860a8640443e701 adds exactly two calls:
TaskGuard and RegisteredTransition's genuine abandonment branches revoke new
mechanical jobs before publishing Lost. Either guard can publish first, so both
must defend that boundary. Normal Finished completion and the normal disarmed
error-return endpoint remain unchanged. The cancelled actor slot remains Unknown;
actual NotCreated worker and custodian joins free only their own two slots.
The exact retained entry has no deferred recovery disposition. No native stop,
new capability, producer, preparation-consumption or first-cause change is added.

The two missing design-required mutation controls were addressed by test-only
bb81801594a0f540291bea89c51cebdce8049db9. The actual custodied closed-runtime
spawn now runs on a helper thread with the existing2s observation bound; its
normal completion joins that helper. The fixture never queues a resource factory
or starts an OS child. A deliberately deadlocked mutant can fail within the bound;
its detached synthetic thread is terminated by test-process exit, which earns no
join, native cleanup or complete resource-settlement credit.

Three additional distinct operators were compiled on exactbb818015 in an owned
mutation worktree: remove both abandonment revocations (one boundary), omit the
pre-reservation runtime check, and move the actual Control.task mutex release
until after runtime.spawn. Each failed at its intended consumer assertion.
The first driver's expected assertion text was wrong: it reported false for the
abandonment kill and exited1 at its final classification assertion, although the
compiled test had exited101 at the actual file-effect assertion. This raw record
is retained. Manual inspection of its immutable log established the intended
failure; no mutant execution was rerun to obtain a passing driver classification.
The verified per-mutant commit/patch/log hashes are in
[the mutation record](issue-19-custody-source2-mutations.json).

Exact full treeee8a541be2f714fb2e526471e2959b91e1bc4f44 was restored clean to
bb818015. Restored Debug19PASS2.77s, Release19PASS2.67s, fmt and all-target
warnings-denied clippy passed. All owned mutation commands closed before normal
mutation-worktree removal. The earlier16 operators retain their original94350f
observation; these3 additions make19 distinct operators across those respective
baselines, not a new19-operator run on bb818015.

A's early-reply2s overlap oracle remains timing-dependent: if a broken actor is
not scheduled within its detection window, that mutant can survive. The earlier
observed kill is retained with this limitation; no deadline/parallelism increase,
serial run or timing-independent guarantee is claimed. The worker-spawn-Err plus
queued Note and future pre-created handler-panic issues remain unreproduced
activation requirements. The resource-already-joined/context-error, Lost with no
deferred intent, nonexistent terminal eviction and fixture-versus-factory Git
wording have been corrected in the design/master/current evidence.

## Exact fix regression and prior CI observation

At clean8760242, focused Debug19 and Release19 passed, affected Release
Codex122 passed/1 ignored (42.53s), and fmt, all-target warnings-denied clippy,
Debug build and Release build passed. Full default Debug remains RED:
library284 passed/6 failed/23 ignored in65.46s, with later workspace targets
unrun. All19 mechanics passed. This distinct required run follows an actual
production fix; it is not a same-head retry of94350f. Full Release workspace tests
were not run. The later bb818015 changes only the bounded test control; its
restored focused/lint checks do not replace this failed workspace gate.

- live_reference_change_rejects_actual_resume_before_spawn: Git preflight Timeout,
  nondispatched/not-attempted receipt, actual child exit101 observed10687ms.
- absent_native_baseline_cannot_be_introduced_by_declared_caller: expected transport
  success failed; actual failure string unprinted, child exit101 observed10696ms.
- owning_reference_change_after_snapshot_rejects_start: expected StateConflict
  prefix failed; actual failure string unprinted, child exit101 observed10706ms.
- unrelated_foreign_changes_do_not_revoke_start: kind0 Git preflight Timeout,
  nondispatched/not-attempted receipt, actual exit101 observed10718ms.
- unrelated_foreign_changes_do_not_revoke_resume: original60002ms watchdog;
  last boundary2/case7/terminal, actual child SIGKILL/reap.
- unrelated_foreign_changes_do_not_revoke_checkpoint: original60002ms watchdog;
  last boundary1/case8/terminal, actual child SIGKILL/reap.

Cause and component contribution remain unknown. Prior parent94350f/857/a8 failures
and current8760242 failures are separate observations, without a budget/latch or
acceptance waiver. Full Debug log SHA256:
`513b4ed900c90392c090e95351088043ff278b574e247e121bb7a3b486fd8651`.
Affected Release log SHA256:
`45ad5f6ffb79790d78c8d989b03d18a75b8f0c4aa7ea84d248a7fa0ffc95e329`.

Prior exact895a621 CI37258294171 succeeded at every macOS/Linux step. Actual
checkout e3355fe9b377d56d8bf9f5266c3f2039aef7638c has parents768f843/895a621
and complete treedcc8a5008b6a4e51ecde376dca8a2d7f5acf4439 identical to895a621.
It ran workspace Debug tests and Release builds, not full Release tests; it does
not cover the later fix/test delta or explain the local failures. Log SHA256:
`977b6d8daed6d8dc591ab25c730415ff35843dd0d8b70fa939ba98d9e1db5936`.
The new immutable delta still requires independent review and final-head CI;
workspace failure kept this draft composition unmerged at that phase; the later
d938 current-inventory observations are recorded separately below.


## Finite source review and control corrections

The f6 native delta round does not supply two qualified approvals. Reviewer A's
terminal source approval returned through a wrapper that exited1 with
owned_cleanup_unverified; later observed group death and reaped anchors do not
clear the prior Unknown. That result is retained as unqualified content.
Reviewer B's wrapper exited0 with verified cleanup and approved only the source
with five Low, without merge/full-gate/native approval.

At clean226d1ab, both actual custodied closed-runtime consumers use the same2s
finite helper. Disconnected joins the helper and resumes its actual panic;
Timeout fails explicitly and earns no synthetic-thread join/cleanup credit.
The bare Control consumer captures only TaskGuard and observes its revocation,
so RegisteredTransition cannot mask that call. Caller and finished observations
also have the existing2s bound. A compiled single TaskGuard-call omission failed
at that exact assertion. A second variant of the prior task-lock regression ran
all19 controls:17 passed and BOTH closed-runtime consumers failed within the
bound. These are sensitivity variants of existing boundary families, not two
new guarantees. Clean exact226/tree563fac41 was restored; Debug19, Release19,
fmt and warnings-denied all-target clippy passed before owned worktree removal.

Two existing agents independently reviewed the actual source/consumers at226:
both approved without Critical/High/Medium; one Low identified that the caller's
revocation observation occurred after cleanup. Four test lines at9687bae move
that assertion immediately after the actual caller Err observation, before
error classification, wait_finished and drain. Both agents independently approved
that four-line delta without findings; the targeted Debug and Release controls
passed. These were read-only code reviews, with no new native inference or
independently executed tests. Exact records and log hashes are in
[the finite control ledger](issue-19-custody-source3-controls.json).

The fix/control proves the queued-before-handler case. The custodian samples
permission once before create_worker; thread creation, handle installation and
Begin follow without a shared revocation/commit lock. Revocation after that
sample can therefore precede a later file effect. This source-verifiable window
was reported in unqualified review A content, not reproduced by a control;
in-flight work remains retained and charged. Effect commitment and revocation
must be linearized and tested before any factory activation. No after-Lost
ordering-mutant guarantee is claimed. Zero-created Lost retention also depends
on accepted/in-flight counters at drop: a queued request later refused can retain
an entry without disposition, while the no-queued-work case removes it. Recovery
for that documented availability cost remains pending.

## Distinct comparison and current Release observations

The separate main26f3 default Debug comparison passed383 Rust tests plus2
doctests. A distinct f6 diagnostic with19 mechanics filtered remained RED:
library259 passed/12 failed/23 ignored/19 filtered, later targets unrun. This
filtered inventory is not a workspace gate or retry. Removing those mechanics
was not sufficient for a pass; causes/component contribution remain unknown.

At clean9687bae, the previously unrun full default Release workspace test
exited101: library288 passed/2 failed/23 ignored in54.12s; later targets unrun.
All19 mechanics passed. Grok own-reference-refresh failed because its current
attempt canary was absent (actual child exit101 observed9244ms); the before-spawn
stop child failed after9.74s with Timeout, owned native stop not yet verified.
The blocking operation and component contribution remain unknown. No same-head
retry or budget/parallelism/latch waiver follows. Earlier full Debug failures
remain separate and undetermined causes remain open.

Prior f6 CI37261604727 passed every macOS/Linux step. Actual checkout
abaa6ea9c097951fb4de2fafc1576255f13d7dc6 has parents26f3a48/f6b18af and
fulltree396ce6618ace6fc76c41261d9811f6bd65ff9847 equal to f6. It executed
Debug workspace tests and Release builds, not full Release tests. It does not
cover the later226/968 test changes or discharge the local failed gates.
At that phase final-head CI remained pending and PR69 stayed draft/unmerged;
later current-inventory observations are recorded below.


## Direct response ordering control

At clean5b7c235, one53-line test addition polls the actual Factory.run/Create
oneshot response after the handler's retirement_reached gate. The response must
remain Pending while its own accepted/effect request remains1. An early-send
mutant already published its real oneshot result before this gate; direct polling
observes that result without relying on scheduling of an adapter actor. No
implementation-side early-reply flag is used. After releasing retirement, the
actual StateConflict response arrives, the retained actor/custodian join and
NotCreated accounting returns the isolated pool to0; the file is absent.

The direct control passed Debug/Release in0.01s each. The single compiled mutant
785bb57f failed at the intended actual-response assertion in0.01s. Its patch is
a sensitivity variant of the historical reply-before-own-request-retirement
boundary, adding no distinct operator/guarantee count. Exact5b7 was restored clean;
all20 controls passed Debug2.51s/Release2.57s, with fmt and warnings-denied
all-target clippy passing. Root-owned cargo commands exited and were waited
before normal mutation-worktree removal. Failed-assertion RAII releases pauses
and the actor sender, but earns no failure-path complete join/cleanup credit.

Two existing independent read-only reviewers approved exactly the53-line test
delta with no findings; neither independently ran tests/native inference.
[The finite record](issue-19-custody-direct-response.json) binds source, mutant,
patch/log hashes and their exact review scope. The existing adapter error-arm
control and its timing Low remain separate; direct response sensitivity does not
prove in-flight effect commitment, native cleanup or whole consumer acceptance.

BothOS CI37265599043 succeeded on reviewedfa61: actualcheckout4b213df has
parents26f3/fa61 and fulltreeccb1017a/all284pathblobs equal to fa61. This was
Debug workspace TEST plus Debug/Release BUILD, not full Release TEST. It does
not cover the later5b7 test addition. At that phase current-source final CI was
pending; local full Debug876 and Release968 failures retain unknown causes.
No same-head retry, deadline/parallelism change or full-gate waiver is applied.


## Current inventory gates and limited disposition

At clean d938610e705271396d27a760813297eaf77f19f9, the first full default
Debug and distinct Release workspace test commands for the newly added actual
response control both exited0. Each executed403 Rust tests plus2 doctests, with
26 ignored (library291 passed/0 failed/23 ignored). Command elapsed time was
117.21s Debug and119.63s Release. Original internal deadlines and default test
parallelism were preserved. All workspace targets executed in both modes.

CI37267268133 passed every step on Linux and macOS. Both actual checkout logs
identify7a45bf1963f104ec7eac5e33b595feb5ba35eeaf, whose parents are26f3a48/d938
and complete tree123f0d2cffafc055716e383cae29c7a44c6223c8 (all285 mode/type/blob/path
entries) equals d938. This supplies Debug workspace TEST, fmt, warnings-denied
all-target clippy and Debug/Release BUILD. The distinct local Release command
supplies Release TEST; CI does not. Details are bound in the
[current gate record](issue-19-custody-current-gates.json).

The changed test inventory is not a same-failed-head retry. Production has not
changed since the two abandonment revocation calls at876. The53-line addition
observes a real response boundary and has compiled sensitivity evidence; it
does not change Grok preflight/inspection/watchdog consumers. These successful
observations neither explain nor erase the historical943/876/968 and filtered
f6 failures. Their causes, contribution and regression attribution remain unknown.
Shared fixture/host load was not isolated or established as a cause.

The candidate disposition is a limited mechanical composition only: test-only
Create factory, production custody selection None, retained in-flight charging
and refusal ordering. No native backend is enabled. The documented effect-commit
linearization window, future worker-spawn/handler hardening, zero-created Lost
retention/recovery, private producers and settlement remain unqualified before
factory activation. Whole Issues19/6/43, workflow/context integration, native
cleanup acceptance and the MVP remain open. The prior unqualified native review
wrapper result stays unqualified; current gate success does not adopt its cleanup
or elevate source-only approvals to merge approval. Final docs-only head CI and
independent limited disposition review remain required before merge.
