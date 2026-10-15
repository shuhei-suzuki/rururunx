# Issue60 selected Git reader lifetime verification

Status: Requirements4 approved at a8e8a8b; Design2 approved at46f819b with verified Low precision.
Selected reader/owner implementation is committed; Source review, mutations and required gates are pending. No source acceptance. Earlier diagnostic/environment partial gates are separate.
Base2c6ae9d includes diagnostic partialPR61 and environment partialPR53. Their
independent approvals do not authorize this reader/owner-driver change. Parent60
and native16/F1/runtime-workload settlement remain OPEN.

Historical18c/b90/555 failures and every later CI failure remain FAILED/RED with
cause AND regression UNKNOWN. A reader JoinHandle source gap is verified
separately; no historical EOF writer, scheduling/inspection cause or regression
is identified by that observation.

Preparation: exclusive worktree issue-60, branchfix/issue-60-readers, from clean
main2c6ae9d. No source changes. Agent6 explicitly confirms approved StageA has no
common ProcessGroup/Drop/retained-child implementation/approval; StageB remains
required OPEN. Shared impact is consequently part of this independent design,
not an assumed dependency or silent helper edit.

Primary Tokio contract check: repository Cargo.lock resolves1.53.1. The fetched
locked crate's runtime/task/join.rs documents task detachment on dropped handle
and that cancellation must complete before the finished state is observed;
SHA256 `85484895de0c7f38a6ce49cb42ed4ba7b680b674deaf27aeb77467dc987b3ee7`. Native blocking work already
running is not cancellable by a Tokio abort request. These are source/API facts,
not observed evidence that a particular old failure left a live task.
[Published Tokio JoinHandle documentation](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html)
was checked2026-10-05JST; that page currently labels1.53.2. The baseline statement
is independently checked against locked1.53.1 source; no dependency/version change
is proposed or inferred from the latest-page label.

Reader Requirements1 at20c1fdf completed two independent native reviews:
ownership `e0bdd4dc-5102-4f3e-bc44-0a21c01f9689`, capacity
`0f60fea3-8864-40f4-ad83-2c70aceb359a`. Both returned request_changes;
both actual review wrappers completed with cleanup_verified=true. This wrapper
fact is not full enabled-workload/F1 certification. The complete15 findings,
raw-result hashes and verified candidate dispositions are preserved in
[the findings ledger](issue-60-reader-requirements1-findings.json). Requirements2
corrections have no implementation or approval yet. Diagnostic and environment
partial reviews do not substitute for this reader gate.

Verified consumer correction: Generic's live returned launch error at
adapter.rs630–646 uses ErrorKind::SessionLost only and disarms its reservation;
only Reservation::drop457–468 reads process_uncertain. Requirements2 explicitly
adds the live own-uncertainty Lost predicate while preserving the API error kind,
with Unknown-versus-settled error controls still pending. Grok checkpoint's local
ownership flags atmod.rs596–599 have no receipt/Session/reservation sink; its Err
and proposed pool retention cannot claim durable protection or a checkpoint-to-
resume fence. That stronger fence and non-Git native constructor/reader sites
remain OPEN. Global capacity's Project coupling/fairness acceptance also remains
OPEN.

Locked primary spawn evidence was byte-verified against public tagged sources:
[Tokio1.53.1 process](https://github.com/tokio-rs/tokio/blob/tokio-1.53.1/tokio/src/process/mod.rs)
lines863–866 create a std Child before wrapper construction; lines950–962 create
the kill-on-drop guard only after fallible imp::build_child.
[Unix wrapper](https://github.com/tokio-rs/tokio/blob/tokio-1.53.1/tokio/src/process/unix/mod.rs)
lines118–144 contain fallible pipe/pidfd/signal setup, and365–373 pipe registration.
[PollEvented](https://github.com/tokio-rs/tokio/blob/tokio-1.53.1/tokio/src/io/poll_evented.rs)
lines110–120 use the current scheduler handle and fallible registration.
[Rust1.91.1 Child](https://github.com/rust-lang/rust/blob/1.91.1/library/std/src/process.rs)
lines181–200 documents neither automatic stopping nor waiting on drop. Exact
public blobs/SHA256 and locked-crate equality are in the findings ledger. Therefore
Err/no returned Tokio Child is not uniformly proof of no native effect. Candidate
requires an actual native Child anchor before those fallible steps and a real
native-spawned std Child/forced-initializer-failure control; it permits neither
numeric resurrection nor reaping outside the owning Child. No old CI failure,
real wrapper error occurrence or invalid PID is attributed to this source fact.

Same already-reserved healthy supervisor must continue observing existing
resources after caller Unknown and release capacity on complete actual settlement;
it never clears the published flag or Context latch late. Returned reap error or
lost supervisor/driver instead retains Unknown without a promised replacement
observer. Resources must be created/driven in the independent owning context,
not merely moved from the caller runtime. Current seven adapter and one Context
Unknown fixtures must use private cfg(test) pools through actual consumer paths;
teardown/production-counter controls remain pending. These are candidate gate
corrections, not executed tests or an available owner implementation.

Requirements2 corrections were committed fc0ce76, then normally merged with main
efe977486693a065122d8fc177b0e83d29620bdc at49b434b. Both public ancestries are
preserved. Incoming source is the independent5-line Store::usage safe read-error
projection plus105 pure State control lines and supporting documentation; actual
adapter/common inspection/Grok/Context/Cargo.lock diff against20c is empty. No
new reader tests/source are implemented or passed. Earlier docs trigger20c CI
37235601171 finished all steps both OS success; actual checkout/tree was not yet
captured here, so this is not a claimed reader implementation or checkout proof.
Current reader requirements/design/source gates remain pending.

Reader Requirements2 at6faf26f completed both independent own-session delta
reviews: ownership and capacity request_changes, each cleanup_verified=true.
They verify the earlier15 findings addressed in the candidate, but identify the
same new Medium contradiction: pre-wrapping native std Child retention cannot
require the current Tokio-Child-only ProcessGroup. Four narrow Low topics also
need precision. Full8 findings and raw hashes are recorded in
[Requirements2 findings](issue-60-reader-requirements2-findings.json).
Requirements3 replaces the hardwired owner with a Git std Child anchor and
explicit current group-signal/inspection/TestPlan semantics, through an impact-
gated shared primitive OR a reviewed Git-local equivalent. Actual consumer
parity controls/mutants remain pending; no native/shared source edit is made.

Other corrections: live fully-settled initialization error MUST clear; cleanup/
reap error precedence is explicit. A bounded driver-owned supervisor execution
frame must observe/destroy the supervisor future before the final slot-release
without a new observer job/native work afterwards. Generic/Grok native Tokio
spawn-wrapper Err paths remain OPEN outside the Git flag predicate. The old std
source range is corrected to181–200; previous reviewed heads/findings remain
unchanged. These are candidate corrections awaiting a new requirements gate.

Exact Rust1.91.1
[Unix native spawn source](https://github.com/rust-lang/rust/blob/1.91.1/library/std/src/sys/process/unix/unix.rs)
was read from public blob11d48878727b0b2532d3a5b2724aa29898f9adc7,
SHA256 `0cc2c8548f3760be01cbde0a1825fe0e5f9a18144fa5703e04cc792175ebfbe9`.
The fork/exec Err path waits before returning at138–160; optional Linux pidfd
spawn at773–809 instead has a post-spawn PID-resolution Err. The tagged
[command defaults](https://github.com/rust-lang/rust/blob/1.91.1/library/std/src/sys/process/unix/common.rs)
at168–194 disable create_pidfd and have no pre_exec callbacks. Requirements3
therefore demands profile-specific primary proof for the actual pinned CI
Linux/macOS command recipe and a real nonexistent-executable consumer control;
it does not infer no surviving child from every std/Tokio Err. Internal fork/exec
child settlement is a resource fact, not a durable no-effect certificate. None
of these code paths is asserted observed in an old CI failure.

Current docs baselineCI37238262914 all steps both OS success. Both actual checkout
160839ef3a8bf18c91c04675e06da73ae8a87a29 have parents efe9774/6faf26f; complete
tree042d21b8f9daa4d52770470bac0283cb7c6749b5 equals the triggering candidate.
[Raw step/provenance ledger](issue-60-reader-requirements2-ci.json) retains the
log hash and full-tree comparison. This is docs/current-existing-source DEBUG
test plus release BUILD evidence; no reader source, release TEST, requirements
approval or native availability acceptance is claimed. Historical failures remain
FAILED/RED; cause and regression UNKNOWN. Whole60/native16/F1 remain OPEN.

Reader Requirements3 at7761f07 again completed two independent own-session reviews,
both request_changes/cleanup_verified=true. They verify the prior8 findings
addressed, but identify the same Medium: CI-only spawn-profile refusal has no
pre-spawn production binding and omits libc/kernel POSIX error semantics. Full6
findings and exact raw hashes are in
[Requirements3 ledger](issue-60-reader-requirements3-findings.json).
Requirements4 removes that refusal and introduces no invented profile witness,
new pin/probe/policy. Successful actual std-Child-owned Git remains allowed under
existing ownership/cleanup. An ambiguous native spawn Err without an actual Child
returns original ProcessFailure with flag/Unknown/all four permits retained; it
claims no Child anchor when none was returned. Proven no-native-spawn-attempt
refusal uses actual no-attempt/resource facts; an attempted Err cannot be renamed
no-attempt. Availability under such ambiguous errors remains explicitly OPEN.
Future safe Err no-survivor classification requires separate primary std/POSIX/
libc/kernel and actual build/recipe binding gates; nothing here grants that proof.

Other verified corrections: not_started reader counts settled only when no task
existed and its endpoint closed or never registered; a started reader still needs
an observed join. Both OS Ok/SRCH succeed; macOS PERM needs valid-dead trusted
inspection, all other errno cases remain Unknown; existing TestPlan PERM forcing
is required parity input. Prior primary-defaults evidence now includes common.rs
blob ea45b08e90a34dd3400edb41ec1ac31f43bfc96b and SHA256
`e7f50147520968b10545167a7d5aa4a50edd3efe75559bd724e19652a42c7c9a`.
The posix_spawn host-error proof remains unaccepted. Requirements4 is a candidate;
no implementation, lifetime/availability acceptance or historical cause/regression
finding is introduced by these corrections. Whole60/F1/native16 remain OPEN.

Reader Requirements4 at a8e8a8b completed BOTH independent own-session delta
reviews: ownership and capacity APPROVE, no C/H/M, two Low findings each, actual
wrapper cleanup_verified=true. This weak wrapper fact is not F1/workload proof.
[The finite findings ledger](issue-60-reader-requirements4-findings.json) records
all four raw findings, hashes and verified dispositions. Derived precision is
carried into Design1: no-Child ambiguous Err completes/destroys the supervisor
but permanently retains all four slots, with no observer/release/clear; EVERY new
Unknown/destructive consumer uses a private pool, including Context/direct Git;
no-attempt keeps false if never set or clears at live settled return if set,
while Drop after spawning freezes true. These do not add native authority,
profiles, deadlines or reader source. Design/source approval is still pending.

Candidate Design1 chooses a Git-local std Child worker plus a per-operation
current-thread runtime/supervisor frame. Its actual four work jobs are one
supervisor execution frame, stdout reader, stderr reader and one native worker;
no separate permanent dispatcher/reaper/monitor or bootstrap task is proposed.
At most16 operation frames/32 readers use64 slots; resources and initialization
stay independent of caller runtime. The design must pass its own two native
gates before source. No actual reader controls/mutants have run. Requirements3
CI37239214385 and Requirements4 CI37240132705 report all steps both OS success;
actual checkout/tree proofs are not yet captured here, so these are docs baseline
observations only, never reader Source, Release TEST or availability evidence.
All historical reds, cause AND regression UNKNOWN, full60/F1/native16 remain OPEN.

Reader Design1 at8f95adb completed BOTH fresh independent native reviews:
ownership00677738-28ea-413e-aa5b-1919c43421ac and capacity
5a3cf784-2c81-4218-a9ec-5488292ff43e, both request_changes/cleanup_verified=true.
[All16 findings](issue-60-reader-design1-findings.json) retain exact raw hashes
and verified dispositions. Two distinct Medium omissions: no live caller wake/
typed outcome on supervisor loss, and undefined private-pool/plan propagation
through Grok's spawned actor. Four counted jobs/std Child/runtime independence/
unreaped observer/terminal frame accounting were accepted by both, not Source proof.
Design2 adds the bounded loss publication and actual cfgtest carriers, distinct
wakes/cancelled-before-spawn state, attempted-signal bit/no releasing Drop,
pre-effect SIGCHLD/BOTH readers order, actual worker join before settled result,
off-poller retained Runtime holder and explicit impact/errorfact parity. Bounded
command parking and one actual owning Child.wait on reserved worker replace the
proposed5ms worker/reap polling; supervisor's existing250ms reap cutoff and late
same-wait observation stay unchanged. No new native/effect/readiness authority.

After both reviewers finished, normal main47830b0 integration preserves both
public ancestries. Incoming #6 EMPTY ordinary routes refuse before Git/process;
only additive common visibility/traits, Grok structured delegation and Tokio net/
dependencies change shared files. Actual Git/common cleanup bodies and locked
Tokio1.53.1 remain unchanged; future/private Codex Git copies remain unqualified.
Design1 CI37242481311 allstepsbothOS SUCCESS tested merge13de0884 with parents
47830b0/8f95adb; COMPLETE tree differs trigger8f because incoming6 is present.
This is merge-context/docs-baseline evidence, not reviewed-head equality or
reader Source/ReleaseTEST acceptance. Private finite CI provenance/inventory
retains that difference; Design2/source gates still pending. Historical reds,
cause AND regression UNKNOWN; whole60/F1/native16 remain OPEN.

Reader Design2 at46f819bc932096adfbd11c31ae5586fd66ac8d1f completed BOTH
independent own-session delta reviews: ownership APPROVE/3Low, capacity APPROVE/
5Low, no C/H/M or unresolved blockers; actual wrapper cleanup_verified=true.
The finite [raw-hash/findings ledger](issue-60-reader-design2-findings.json) records
all eight instances and verified precision. These weak wrapper facts certify no
full native workload. Separate cleanup-stage completion now starts the existing
250ms reap window; command authorization is atomic with Spawning; worker-frame
Senders alone disconnect on unwind; terminal results retain primary kind and
never receive a late flag write. Pool-only poison after real own settlement holds
slots without false native Unknown. Allocation/start has no intervening await;
Unknown resources remain in preallocated vaults. Actual controls/mutants are
source-pending. No requirements authority, job, deadline or backend is added.

Both reviews closed before normal main cf8a1e7 integration. Incoming28 files are
Workflow unbound-dispatch retry guard, Store marker predicate, pure controls and
docs. Adapter/common selected inspector/Context/Grok/Cargo source inputs are
IDENTICAL to reviewed46; no native/common ownership behavior is inherited.
Design2 CI37244640967 allstepsbothOS SUCCESS: actual checkout2baa4f44277304bf3a821b312adb1fcde10b33af
parents47830b0/46f819b, COMPLETE tree equals triggering46. This remains docs
DEBUG test plus release BUILD only, not reader Source/Release TEST acceptance.
Separate main cf8 CI37244736209 macOS Codex ownership inspector deadline and
Root23 composed364139d full RELEASE two Grok env isolated-child60s watchdog
failures remain FAILED, cause AND regression UNKNOWN. Cleanup returned in the
watchdog helper does not prove all nested jobs settled; no component fix or
inspector scheduling cause is inferred. Whole60/F1/native16 remain OPEN.


Implementation checkpoint (2026-10-05 JST): normal main cf8 and then5b4a314
composition preserves public ancestries. The latter introduces pure Task DAG
validation and its existing Store transaction wiring; selected Git, inspector,
Context and Grok source bodies are unchanged by that incoming merge.

The new private `adapter::git_owner` reserves four counted jobs before native
spawn: one std supervisor owning a current-thread runtime, two reader tasks,
and one precreated std worker owning the actual std Child. Global64 jobs permits
at most16 such operations. Actual Child is anchored before group/stdio/runtime
registration; opaque attempted spawn Err retains Unknown and all four slots.
Healthy resource completion requires both observed reader joins and actual native
worker join before runtime/asset destruction and terminal capacity release.
Abort-request and returned/panicked/cancelled reader observations are separate.
Caller Drop only publishes a bounded cancel fact/message. The caller runtime can
stop without destroying the owner's runtime. Lost execution retains its vaults.
This is process-local selected Git custody, not managed operation producer,
durable recovery, process containment or a qualified full native workload.

Controls at fe127ed passed19/1 explicit child-entry ignored; bdd6bac passed21/1
ignored and da0e932 passed22/1 ignored with default internal concurrency. Actual
owned alternate-group fixture demonstrates that empty original group cleanup
can precede direct Child death: the owning wait remains blocked beyond250ms,
caller returns SessionLost/frozen flag/four held slots, and the same worker and
readers later settle/release without clearing that returned flag. The fixture
owns its alternate anchor and release paths; it does not signal foreign PIDs or
claim stronger containment. Cleanup-ack delay beyond250ms preserves the separate
reap window. Real endpoint reader panic and pending-peer injection distinguish
observed join states. These pending-reader seams do not identify a historic EOF
writer or prove an actual escaped writer occurred.

At223e915 actual Grok actor controls4 PASS (2.32s): pre-spawn unknown,
raw index pre-spawn/reconciliation unknown, post-native binding-refresh unknown,
and explicit checkpoint failure. The actual receipt and persisted Lost/reserved
Session are asserted at the relevant actor stages. Checkpoint preserves its
original Err and does not publish the proposed input; only current fresh-input
resume rejection is proved, not a durable checkpoint Unknown fence. Actual
Context dropped-future control1 PASS proves CallGuard cancellation precedes
GitObservation latch retention; late resource settlement cannot clear it.

Current c0ec0a2 owner controls25 PASS/1 explicit child entry ignored (4.09s),
including actual Generic output-open/non-SessionLost API error remaining
persisted Lost/reserved after the same readers settle late. Actual Context
output-open/latch control1 PASS (0.30s). Injected pending stdout owns the real
endpoint and exercises the production output cutoff; this is not an external
writer/OS scheduling measurement. Invalid-binding injection retains a real
std-spawned Child and untransferred endpoints; it does not assert the OS supplied
such an invalid PID. Initializer failure plus actual first KILL/forced malformed
real inspector holds the actual Child and slots with cleanup Unknown.

Prior implementation failures are retained:49bcf13/63bc424 compilation failed
on env iterator/type/privacy/cfg-call mismatches;26fdfe0 compiled with an unused
context warning corrected in dacce66. At6bb9797 the caller-runtime fixture timed
out (16 PASS/1 FAIL); its two waiters shared a notify-one seam, corrected to
broadcast plus explicit caller stop/join before assertions in268cd53. That is
verified fixture wake wiring, not a historical inspector/watchdog cause.
da0e932 all-target Clippy failed on test Store guard await-lifetime;9c574a2
uses a lexical scope and passed Clippy. At90c2e59 owner24 PASS/1 FAIL/1 ignored:
a specialized stderr/timeout assertion was incorrectly applied to malformed
frame InvalidData. c0ec0a2 asserts the actual frame-validation facts; production
inspector/cleanup bodies are unchanged. No failed result is relabeled PASS.

Full default workspace DEBUG at9c574a2 FAILED: lib294 PASS/5 FAIL/24 ignored,
62.63s. Full default workspace RELEASE at the same source FAILED: lib294 PASS/
5 FAIL/24 ignored,67.55s. These lib failures stopped the remaining targets.
DEBUG failures include owning-ref start expected StateConflict without actual
reason printed; unrelated start actual pre-native `Timeout: Git ownership
preflight timed out`, receipt cleanup_ok=true/not_attempted, dispatched=false,
output_verified=true, ownership_uncertain=false; before-spawn stop actual owned
stop Timeout; checkpoint/resume child60s watchdogs with only `running 1 test`.
RELEASE failures include absent-baseline transport false, owning-ref canary
missing, live-ref resume and unrelated resume actual pre-native Git Timeout
with the same no-native/clean receipt, plus checkpoint60s watchdog. The fixture
watchdog helper's group cleanup/reap returning does not prove all nested jobs
settled. Cause AND regression UNKNOWN; no rerun-to-pass, serialization, deadline
increase, latch reset or cleanup/Unknown relaxation. These are open full-gate
blockers, not twenty-two-component-control failures or native availability proof.

Additional peer baselines remain separate: Issue19 same-source06ba908
CI37247560442 Mac first saturated owner inspector309978us (spawn1357us, both
streams0/statuspending), followed by five Context latch-derived failures; root67
code87ec5ee CI37247900919 Mac first Context299506us (spawn1652us, validation
not reached) plus derived failures, then same-source metadata1bea both OS green.
Neither cause nor regression is identified by these outcomes. No old head is
rerun or passing metadata used to erase red evidence. Whole60/F1/native16,
shared native ProcessGroup/new/reap/Drop and historical availability remain OPEN.


First causal matrix at c0ec0a2:12 distinct compiled operators,11 assertion-killed
and one survivor. No kill credit for M60-R04 (omit actual worker.join): its
positive fixture sampled only one caller yield, which did not establish progress
of the independent std supervisor. The control now waits on the actual caller
outcome while the native worker is held, releases its own fixture guard before
assertion, and will rerun this same operator at the committed corrected control.
This is a verified coverage defect, not a production join omission or a deadline
relaxation. All private mutation sources were restored exactly after each run.
No complete all-descendant settlement is inferred from mutation test termination.
The finite [matrix](issue-60-reader-source1-mutations.json) retains every result,
patch and log hash, including the survivor.


The first strengthened M60-R04 control still SURVIVED after3s (no credit).
Source verification identifies an additional masking fence: the private native
worker pause retained its NativeAssets MutexGuard; after the omitted join the
supervisor blocked acquiring that same vault for its native inspection. The
second run therefore proves no lone worker.join guarantee. The cfg(test) pause
now explicitly drops that guard after successful actual reap and recorded native
facts, before holding the actual worker. Production paths are unchanged; the
same omission operator requires a fresh actual consumer kill. Both survivors
are preserved, not retrospectively attributed to a production defect or old
CI/watchdog cause.

Unmerged test-only Issue67 dependency5ca71f2 is normally composed atcd1004a,
with its required Mac CI red and unmerged status retained. Incoming20 cfg(test)
lines add bounded static stage/boundary/case and parent observation/exit/elapsed
facts. They preserve60s watchdog, internal parallelism, readers, signals, cleanup
order and existing assertions. Neither they nor this composition identify a
historic process-inspection cause or grant complete nested settlement.


The third same M60-R04 operator now COMPILED/ASSERTION-KILLED at the explicit
`caller returned before its actual native worker joined` assertion (test0.02s).
Correct unmasked bcbd694 consumer passed1/3.02s. Matrix has14 compiled runs,
12 distinct operators:12 scoped actual-consumer kills and two retained M04
survivors with zero kill credit. Only the cfg(test) guard-drop/pause changes;
production join and cleanup are unchanged. No obsolete run is relabeled.

Timeout precision: validate_git/verify_git each share one existing5s deadline
across their whole sequence. Native dispatched=false/pre_spawn uncertainty=false
in9c receipts does NOT prove no Git process spawned; settled previous/current
Git children can leave that flag false. Admission expiry, observation expiry
and cancel share the existing timeout text. No reason for those failures is
identified by that text or by the false native-agent dispatch flag.

A new default full run at the actual committed67-composed source will capture
current stage diagnostics; the change is cfg(test) observation/source-composition,
not a production availability fix or a same-old-head rerun. The old full Debug
and Release failures remain explicit blockers until disposition is supported.

Current composed1b37c78 full DEFAULT Debug and distinct full DEFAULT Release
both FAILED: each library304 PASS/24 ignored (48.58s/48.12s), then adapter
integration18 PASS/1 FAIL (10.28s/11.14s). The sole failure is the existing
first-Pending cancellation assertion expecting LOST, observing FAILED. These
runs stopped after that target; later workspace targets did not execute.
The first Pending poll cannot establish actual spawn in the independent owner.
The timing inference is removed; unread-stdin's exact Failed assertion remains.
The private actual Generic consumer already pauses before AuthorizeSpawn and
after actual StdChild creation, requires exact Failed/unreserved vs Lost/reserved,
and verifies no late rewrite. It now additionally calls actual public second
start in the held live case and requires StateConflict. No arbitrary state
alternatives, uncertainty guard, production code, timing or cleanup changed.
Old9c environment/watchdog failures remain failed/cause AND regression UNKNOWN;
the current1b37 results do not establish their cause or full workspace readiness.

### Current composed source5b09c6d gates

After both1b37 suites closed, ff92bfb replaced only the first-Pending timing
inference with the deterministic private actual Generic consumer and actual
second-start StateConflict. Normal main768 composition brought only three
Issue67 outcome documents; its20cfg(test) stage lines are identical to the
previously composed dependency. Issue67 is now merged; its historical5ca CI red
and watchdog failures still have cause AND regression UNKNOWN.

Full DEFAULT Debug at5b09 FAILED: library290 PASS/14 FAIL/24 ignored113.47s;
later workspace targets did not execute. All26 selected owner controls passed,
including actual four Git operations/16jobs and public second-start exclusion.
Grok irrelevant checkpoint/resume observed boundary1/2 case0 fixture/start/terminal,
then initial-start Git preflight Timeout. Another existing initial-and-resume
fixture observed watchdog_expired60001ms, actual SIGKILL and only stdout
`running 1 test`, empty stderr; that case has no static progress lines. Other
failures include actual initial canary/transport expectations and pre-native
receipt operands. They do not prove no Git spawn, process scheduling cause,
all nested-job settlement, nor that the new code caused those failures.

Distinct full DEFAULT Release at the same clean source5b09 PASSED415 top-level
Rust tests plus2 doctests/27 ignored; the ordinary Codex nested child1 PASS is
separate. Library304 passed45.17s, adapter19, CLI5, ordinary Codex boundary1,
Context16, Git18, graph4, Grok15, Project16 and State17. Both modes preserved
existing internal parallelism/budgets/watchdogs/Unknown/latches. No own native
review/mutation suite overlapped; peer workloads have no measured causal role.
Fmt/all-target Clippy(-D warnings)/Debug BUILD/Release BUILD all passed.
[Finite raw-log hashes and tested source blobs](issue-60-reader-source1-gates.json)
bind these outcomes to5b09. No full Debug or native readiness is claimed from
the Release result. Formal Source1 two-reviewer gate and required public bothOS
CI/actual checkout provenance remain pending; this candidate is not merge-ready.

Both independent Source1 reviews at immutablea92c4d0 completed request_changes,
actual owned cleanup verified TRUE. The raw public-source findings and verified
dispositions are preserved in [Source1](issue-60-reader-source1-findings.json).
Two distinct Medium defects are verified: a reap timeout can be renamed settled
when actual native wait finishes during the output window; admission expiry lacks
approved finite capacity facts. Existing full-gate blockers remain open. New
cfg(test) causal controls hold the actual native worker after cleanup ACK until
the real250ms reap cutoff, release it on the same supervisor job, and observe
actual wait success before the settled decision; no synthetic successful reap,
new job, actual deadline change, or cleanup relaxation. Direct flag and actual
Context latch assertions will first run against the unfixed computation.

Source1 public CI37253446382 FAILED Ubuntu267 PASS/1 FAIL/17 ignored36.17s;
macOS was cancelled by fail-fast, both builds skipped. Actual checkouted5c43ce
parents768/a92 and completeTreeb0b89495 equals the trigger tree. The failing
four-Git fixture uses a ReadersMutex-held proxy BEFORE actual Child spawn, so
its real_native assertion can sample too early. Both native reviewers separately
verified that proxy and the same saturation-control claim. A post-initialization
actual pause-entry count will replace it, with no arbitrary state alternatives.
[CI outcome/provenance](issue-60-reader-source1-ci.json) remains failed, not rerun.

At committedcba8728 the new deterministic actual late-wait controls both compiled
and FAILED at their intended assertions: directflag `late reap was renamed
in-budget` (0.27s) and actual Context `late reap bypassed Context latch` (0.28s).
These establish the specific R1 source defect, not the original full-sweep timeout
or watchdog cause. The correction requires !reap_pending even when the same
actual worker has already finished during the output window; late actual joins
can release slots while the caller flag/Context latch remain frozen.

Capacity expiry now keeps original ErrorKind::Timeout/prefix and adds finite
capacity_unavailable facts only after actual admission waiting. Counts are sampled at expiry with a fresh pool lock, not at the last refusal;
a racing release/admission may yield less than64. Units are
derived Rust work-job slots: active_jobs and retained_unresolved_jobs, each
saturating at64, four per record; counts expose no identities or fresh process
inspection. Internal never-cleared retention marks early Unknown publication
and owner loss; late actual release removes the row. A poisoned pool reports
occupancy=unavailable rather than fabricated counts. Already-expired initial
deadline and observation/cancel texts stay unchanged. Mixed pressure requires
exact56 active/8 retained jobs, unchanged refusal flag and no new record.

Low fixes stay within the approved contract: a preallocated separate endpoint
vault transfers anchored stdio before the spawned stage, so supervisor reader
registration cannot acquire a mutex held during native spawn/KILL/inspection/wait.
All Retained outcomes force settled=false. Dead post-join caller-flag/terminal
predicates are removed without mutation credit. TestGitContext has no Default
or silent production-pool fallback. Actual pause-entry counts follow initialization;
per-record cfg(test) reader-lane facts count actual created handles, not successful
reads. Saturation asserts16 actual Children/32 handles; four-operation progress
asserts4 actual Children/8 handles, preserving original timeout budgets.


### Source1 verified fixes; Source2 candidate at35f0cda

Both raw Source1 outcomes remain request_changes. Their two distinct Medium and
four distinct Low/latent precision issues have bounded corrections. R1 now keeps
!reap_pending in the settled computation: actual wait after cutoff remains frozen
Unknown even when all original jobs later join and release. Actual direct flag,
Context latch and Grok saved Lost reservation controls cover that interval. R2
reports finite capacity facts only after actual admission waiting, in Rust job-slot
units, four per record. An actual mixed pool requires56 active/8 retained jobs.

A separate preallocated endpoint vault removes reader registration's dependency
on the native mutex held during first cleanup/inspection/wait. The actual consumer
observes both reader handles created while that mutex is held after a direct
ticket.cancel(), then aborts and awaits caller-task teardown before releasing
held stages and requiring late
actual settlement. Actual post-initialization pause entries/handle facts replace
the early mutex proxy:16 actual Children/32 handles and4 Children/8 handles.
No successful read/CPU-concurrency fact is inferred. Dead defensive predicates are
removed without credit; every Retained outcome stays unsettled, and destructive
TestGitContext cannot silently choose the production pool.

Four new compiled operators R13–R16 each assertion-killed their intended actual
consumer; R14 is diagnostic-count evidence only. Exact restored private source
matches35f0; all four restored positive controls PASS. Cumulative ledger is18
compiled runs/16 distinct operators/16 kill runs/TWO old masked survivors with
zero credit. This is finite coverage, not every guard or full native inventory.
[Baseline failures, operators and restored controls](issue-60-reader-source2-mutations.json).

Full DEFAULT Debug at35f0 FAILED: library306 PASS/3 FAIL/24 ignored68.42s;
full DEFAULT Release separately FAILED: library307 PASS/2 FAIL/24 ignored62.85s.
Both stopped at the library, later workspace targets unexecuted. Debug's own-start
child exited101 after11040ms at StateConflict-prefix assertion; the actual failure
string is absent. Both modes' unrelated checkpoint/resume children reached their
original60s watchdog and actual SIGKILL. Debug final static stages are boundary1/2
case7=resumed_terminal. Release checkpoint final stage is boundary1 case7=
resumed_terminal; resume is boundary2 case8=checkpoint. Earlier completed cases
include release/finished. These facts do not identify the wait owner, scheduling
cause, all nested-job settlement, or regression status; both remain UNKNOWN.
No old-head rerun, deadline/parallel/latch change, arbitrary-state allowance or
passing subset/mode substitution is used.

Focused owner29 PASS/1 ignored, Context8 PASS and Grok reader5 PASS at35f0;
fmt/all-target Clippy(-D warnings)/Debug BUILD/Release BUILD PASS. Peer curated19
Rust workload was reported; overlap has no measured causal attribution. The
[raw-log hashes and actual source blobs](issue-60-reader-source2-gates.json)
keep full failed TEST gates distinct from successful builds and selected controls.
Source2 independent re-review and new public bothOS CI/actual checkout proof remain
pending. PR64 remains DRAFT/unmerged; no current full-suite/source/MVP readiness.
All earlier failed gates and whole60/F1/native16/recovery/availability limits remain.


Two separate private cfg-only timing probes preserve default concurrency and all
original5s/250ms/60s limits. Probe1 environment15 PASS/14 ignored58.63s, but its
owning fixture discarded successful stderr, so accessible samples0/credit0.
Probe2 adds a bounded static/numeric-only successful-stderr sink: environment15
PASS/14 ignored57.15s;1721 capped observation samples min31602us/median60209us/
max882285us, all sampled direct-child terminal observations true. Last case8
finished at checkpoint56853ms/resume57049ms/start32437ms. These are successful
private-run measurements only: original failures NOT reproduced, cause AND
regression still UNKNOWN, no gate/native availability credit or all-operation/
nested-resource completeness. Actual private source was restored exactly to35f0
and is clean; diagnostic code never enters the production candidate. Finite
raw-report/log hashes and restored head appear in the Source2 gates artifact.


### Source2 component findings verified; limited Source3 delta pending

Both native Source2 reviews at81e7c29 completed APPROVE/no C/H/M, two Low each,
actual owned cleanup TRUE. They independently verified every prior Medium/Low
correction and exact changed-source equality to35f0. Their approvals address the
component-defect question only: they explicitly retain both full-suite failures,
required CI provenance and whole60/F1/native/recovery limitations. The raw outcomes
are preserved unchanged in Source2 findings; launch-time pending notes do not
silently become source/native acceptance when later observations complete.

Three distinct Low items are verified. The already-expired API control lacked
an exact-message assertion; it now requires the bare original Timeout message.
Its condition mutant receives diagnostic-branch credit only. A new actual
release-wake re-entry control first polls the real refused admission future, holds
it while its original30ms deadline expires, then joins/releases the16 actual
owned child operations and re-polls the notified waiter. It requires Timeout with
suffix0/0, false refused flag and no new record. No fake clock, new task, production
hook or diagnostic behavior is added.
Capacity counts are a fresh snapshot at expiry, not the last locked refusal,
so a racing release/admission may produce counts below64; this is derived pressure
only, never effect/cleanup authority. Reader-registration R15 is driven by direct
ticket.cancel(); the later actual caller abort proves teardown only. Credit is
native-mutex-independent registration, not CallGuard cancellation delivery.
Actual caller-drop wiring is covered separately by existing R05/Generic/Context
controls. No production code, deadline, reader, signal or cleanup change.

Other explicit finite coverage limits remain: production retained-occupancy is not
asserted before/after destructive private controls; the private carrier cannot
fallback in current code, but that does not execute an occupancy invariant. Missing
individual guard operators listed in the raw review retain zero credit. Selected
controls, diagnostic probes and builds cannot close current full TEST failures.


Current Source2 CI37258392978 completed EVERYstep SUCCESS bothOS. Actual checkout
8da165ce768130bb90884df0887ffa4cf1aaf223 has parents768f843/81e7c29 and complete
tested tree equals81e7. Mac420 top-level Rust+2 doctests/27 ignored; Linux384+2/
20 ignored; nested ordinary Codex child1 PASS is separate on each. CI runs Debug
TEST and Release BUILD, no Release TEST. [Actual checkout/step evidence](issue-60-reader-source2-ci.json)
does not rename the local35f0 full Debug/Release failures or establish their cause.

Narrow Source3 candidate a5aef47 changes only test/docs; production source equals
reviewed81e7 exactly. Actual pre-expired bare-text and post-release expired re-entry
controls each PASS in Debug and Release. One compiled inversion of the waited
condition assertion-kills both intended API messages; exact restoration and both
positive controls PASS. The initial short --exact selector ran0 tests and gets
zero coverage. Cumulative20 compiled consumer runs/17 operators/18 kill runs/TWO
preserved masked survivors zero credit; M17 diagnostic-only, not process authority.
Fmt/all-target Clippy PASS. [Finite controls/operator/restoration](issue-60-reader-source3-controls.json)
include the restored actual-consumer terminal evidence; its uncaptured raw-log hash
is null rather than invented. Independent Source3 limited delta gate remains
pending. No current final full Release TEST or merge/native/full60 readiness.


### Source3 finite outcome and precision disposition

Both native limited Source3 reviews at675471c completed APPROVE/no C/H/M, two Low
each, actual owned cleanup TRUE. They independently trace the actual waited-loop
re-entry and verify every earlier Low correction. Their raw outcomes remain
unchanged in [Source3 findings](issue-60-reader-source3-findings.json), including
launch-time pending notes and failed full-gate blockers; approvals address only
this test/docs delta, never merge/full source/native/MVP readiness.

Three distinct Low precision items are verified. The controls artifact now carries
git-derived tested/private-base/restored/current blob bindings for git_owner.rs
and tests.rs, plus the exhaustive a5aef47→675471c four-path docs-only inventory;
all tested/current code is identical. The uncaptured restored raw hash stays null,
with reduced terminal-only evidence rather than an invented hash. Positive suffix
alone cannot distinguish re-entry from timeout_at map_err; branch credit depends
on the bound actual M17 intended kill and locked Tokio1.53.1 Timeout's inner-first
poll at official source line217. No future-version/missed-notify coverage is claimed.

The original30ms first-Pending assumption may falsely fail under host scheduling
delay. No such failure is demonstrated here, and it cannot grant false-positive
credit. This nonblocking risk remains explicit; no deadline/clock/parallel change,
arbitrary Ready allowance or new production hook is introduced. A future failed
required context will remain failed rather than waived by this known risk.

Source3 CI37259754019 EVERYstep succeeded bothOS; the actual checkout/parents and
complete-tree equality to675471c are recorded in [CI provenance](issue-60-reader-source3-ci.json).
It covers full default DEBUG TEST plus RELEASE BUILD only. Local35f0 full Debug
and Release TEST remain FAILED/cause AND regression UNKNOWN. No current final
full Release TEST/source/merge/native acceptance is claimed; whole60/F1/native16,
bootstrap/shared-scope producers, fairness/recovery/availability remain OPEN.
Final outcome-only documentation gate and its new actual-checkout CI remain pending.


Observed outcome-only323e960 CI37260504677 EVERYstep succeeded bothOS; actual
089df3081499d442bf6a27d8abe2110a189749fb parents768/323 and complete tested tree
equals323. README and the two master-reader paragraphs are corrected from stale
5b09 Release415 PASS/current-review-pending wording to actual35f0 both full TEST
failures, completed limited Source2/3 reviews and diagnostic-only M14/M17 credit.
The earlier5b09 Release remains an older-source result. Inspector diagnostics61
are already merged under their recorded limited disposition; that does not prove
reader/EOF/inspector availability. These are finite factual corrections only,
with no normative/source/Cargo/CI behavior change. Final current docs-head gate
and Root's explicit limited disposition remain pending; PR64 stays draft/unmerged.

### Source3 mandatory-gap mapping before the next source gate

Root directly approved the eight outcome-only documents at1104412; final
CI37261291105 every step passed bothOS, actual6b1d7468 parents768/110 and complete
tested tree equals110. CI DEBUG TEST plus RELEASE BUILD does not close the local35f0
full Debug/Release TEST failures. Root explicitly holds PR64 until required gaps
are addressed. Normal incoming26f3a48 changes only two Issue67 evidence documents;
selected source/Cargo/CI identity is unchanged.

Approved Design2 §8 lines378–406 and Requirements acceptance2/4 require signal
outcome parity and actual private-pool isolation. The selected Linux PERM branch
and production retained-count invariant are genuinely missing mandatory proof.
The new candidate uses actual owned KILL followed by a cfg(test)-only resolver
result, never claims real OS permission denial; Linux PERM retains Unknown while
macOS PERM requires its existing actual selected-group observation. Both OS gates
must execute their own branch. Actual independent test subprocesses select the
private destructive controls under default libtest parallelism; test-context
creation/drop checks fresh production retained occupancy zero. This measures only
that process-local count, not native cleanup or OS/descendant settlement. No suite
serialization, deadline/production-pool override or current-source acceptance.

The worker AuthorizeSpawn cancel/lost gate maps to required pre-spawn cancellation:
a private pause after dequeue/before the gate permits cancellation with no Child,
followed by actual worker/reader settlement. The publication poison/lost release
skip maps to required retained authority: pause after actual healthy resources
ended but before terminal publication, poison it, then observe the actual final
pool action before asserting four retained slots. The existing invalid-binding
consumer already asserts both pipes remain in the actual Child; its exact
endpoint-transfer guard still needs a compiled intended mutant.

The supervisor's extra cancel bit is defense in depth beside the publication-state
transition and worker dispatch guard; a single omission may be masked and receives
no causal credit unless an actual reachable control kills it. Retained.settled=false
is likewise defensive: actual current Retained paths already yield false or frozen/
lost publication. No fabricated reachable success fixture or combined masked-guard
mutation will invent credit. Source-only/redundant attribution stays explicit.

This is implementation of existing approved WHAT only. All new controls, intended
operators, restored positives, default/source/CI gates remain pending. Production
read/signal/authority/error/deadline equations are unchanged; test hooks do not mint
native workload or cleanup proof. Both current full TEST failures remain FAILED,
cause AND regression UNKNOWN; full60/F1/native16 and missing producers remain OPEN.

### Required controls candidate at fd406b6

The preceding pending statement describes the mapping checkpoint. The new
[actual controls ledger](issue-60-reader-required-controls.json) records subsequent
observations at fd406b6; Source4 and current required CI are still pending. Source3
and Root's eight-document approval at110 do not approve these new test changes.
Production read/signal/authority/error/deadline equations remain unchanged. Hooks,
actual returned-Child counts and frame-action observation are private cfg(test)
facts, not new native ownership or settlement authority.

All five additional [compiled operators](issue-60-reader-required-mutations.json)
were assertion-killed at their intended consumers and restored exactly after each
run. R18 omits the worker cancel/lost guard after actual command dequeue and creates
one actual Child instead of zero. R19 omits the poison/lost release guard; after
healthy resources ended and the actual pool action completed, retained slots are
zero instead of four. R20 transfers endpoints despite invalid binding and removes
both pipes from the actual retained Child. R21 sends an explicit private carrier
to the production pool; a fresh isolated audit process observes four production
retained slots instead of zero, without a private held-count assertion masking it.
R22 coerces signal errors to success and wrongly accepts the ACCESS case. These
are macOS compiled observations; no Linux mutant execution is claimed. The Linux
PERM Unknown branch must still execute in the current qualified Linux CI.

The fresh audit child runs the destructive private matrix with ordinary libtest
parallelism. It passed45 tests/1ignored in both modes (Debug6.96s, Release6.35s),
separate from its parent test. Creation/drop audits measure only this process-local
production retained count. They do not assert native/descendant cleanup. The actual
owned-group KILL precedes each synthetic resolver outcome, and recorded actual /
applied results prevent an unexercised injected branch from receiving credit.
The temporary initial signal control failure was retained: it lacked the actual
outcome witness and could bypass injection on macOS PERM. The corrected private
hook records that existing result before modeling the production resolver; it does
not claim actual OS permission denial or change the production resolver.

The two remaining single-guard omissions are source-only/redundant, as mapped
above. No fake reachable settled Retained result or combined overdetermined mutant
is introduced. Cumulative coverage is25 compiled runs/22 distinct operators,
23 intended kill runs plus the two historical masked R04 survivors with zero
credit. This is not complete per-guard, native workload or F1 conformance.

Full default Debug TEST at fd406b6 FAILED: library312PASS/2FAIL/24ignored in73.25s;
later workspace targets were unrun. Both existing Grok unrelated-change child
watchdogs expired: checkpoint last boundary1/case7/fixture, resume last
boundary2/case5/resumed_terminal, actual parent-observed SIGKILL. Distinct full
default Release TEST FAILED: library308PASS/6FAIL/24ignored in66.45s, later targets
unrun. Failures were owning-start StateConflict-prefix, stop Failed versus Stopped,
irrelevant-checkpoint case0/terminal with the bare exact message
`Timeout: Git ownership preflight timed out` (no `capacity_unavailable` suffix), completed-resume
Failed versus Exited, missing expected earlier-state assertion, and resume watchdog
last boundary2/case6/fixture. Native dispatched=false/not_attempted receipts do not
prove that no Git operation spawned. Cause AND regression remain UNKNOWN. No
same-mode full retry, latch reset, budget/parallelism relaxation or passing-subset
substitution was used. Earlier35f0 and historical full failures remain separate.

Fmt, all-target Clippy-Dwarnings and both workspace BUILDs passed at fd406b6.
Release BUILD is not Release TEST. The five mandatory operators and actual45-test
matrix close their local scoped coverage gaps only. Current source approval,
required both-OS CI and Root's merge disposition remain pending; PR64 is DRAFT/HOLD.
Whole60, all failed full TEST gates, F1/native16, availability/recovery and missing
shared producer contracts remain OPEN.

### Source4 outcomes and verified Source5 Low corrections

[Source4](issue-60-reader-source4-findings.json) completed independently twice at
e29567a: both APPROVE, zero Critical/High/Medium, five/four Low findings, actual
owned review cleanup verified. Both commands closed before findings were read.
The raw outcomes preserve launch-time pending notes; subsequent
[CI37264766985](issue-60-reader-source4-ci.json) passed every step bothOS. Actual
checkout0f182fce parents26f3/e295 has the same complete tree as e295. The actual
audit child passed45/1ignored on macOS and42/1ignored on Linux; Linux executed
the PERM-to-Unknown branch. This is DEBUG TEST plus RELEASE BUILD, not RELEASE
TEST, and does not change any old/full-gate failure disposition.

The audit selected the healthy ordinary scalar contract using the production pool,
contrary to its private-only comment. f072799 explicitly skips that control in the
audit child; the original scalar test still runs in the ordinary full suite. Its
trusted compiled binary `--list` inventory now checks every applicable selector
and per-OS cardinality before the actual audit. There are45 listed tests on macOS
(44run/1ignored) and42 on Linux (41run/1ignored). The list child executes no test
body, writes an owned regular file, creates no additional pipe-reader job, and
shares the original60s test window with the audit child. Each actual Child is
anchored before observation. This checks current selectors/counts; future carriers
outside these prefixes still require source inventory review, not automatic
discovery. No new production job, native profile, ps observer or deadline change.

The existing worker gate control now aborts and awaits the actual caller while
the native worker is paused after AuthorizeSpawn dequeue, then releases it. Its
zero returned-Child count, frozen flag and late actual release now observe
CallGuard delivery. The old R18 direct-ticket run earns only the cancel operand
at the worker gate; its live-caller response was a test-only interleaving. The
new [same-R18 sensitivity](issue-60-reader-source5-mutations.json) is compiled
and killed at returned Child1versus0; this adds one run, not one distinct operator
or a lost-operand guarantee. New R23 makes a selector stale; the actual trusted
binary inventory rejects it before audited Git/native jobs start. Both sources
were restored exactly; private restored parent1PASS and child44PASS/1ignored3.03s.
The cumulative ledger is27 compiled runs/23 distinct operators/25 intended kill
runs plus the two historical masked survivors with zero credit. R23 covers only
the per-selector non-vacuity arm, not omit-skip/scalar-exclusion/cardinality
sensitivity; those remain source-traced with passing positive controls only.
R19 tests the combined poison/lost predicate with both true,
not separate lost-half sensitivity. R21 was the single-test audit-mode witness
with `RRX_PRIVATE_GIT_POOL_AUDIT=1`, not the45-test parent's filter/status oracle.
Exact commands, environment, tested blobs and fd406→e295→f072 path inventories
are now recorded. This grants no complete per-guard/native workload proof.

macOS PERM credit is narrowed to the source-traced shared resolver/callsite and
the reachable settled branch. There is no independent real-ps invocation witness
or compiled ps-call-site-omission sensitivity; no authoritative observer/profile
is added. Actual e295 Linux branch execution is separate from current Source5 CI,
which must verify the new Linux inventory. No Linux local mutant credit.

From fd406 onward, the ordinary full default suite includes a second default-
parallel process duplicating the selected controls concurrently with their parent
copies, including saturation, the actual selected inspector and Generic/Grok/
Context actors. f072 excludes one scalar copy and adds the no-body list child.
The workload composition differs from35f0. Either copy failing is a failed gate;
no cause, regression or exoneration is inferred, no skip-only diagnostic run or
serialization is used, and existing deadlines/latches remain unchanged.

[Current local macOS gates](issue-60-reader-source5-gates.json) at f072799: full default Debug
PASS425 top-level Rust tests+2docs/27ignored, with nested44audit and1ordinary-library
child counted separately. Distinct full Release TEST FAILED: library308PASS/
6FAIL/24ignored in72.22s; later workspace targets unrun. Four Grok children
observed exit101: absent-baseline transport predicate, native-reference explicit
Git5s Timeout, stop Failed versus Stopped, and resume-admission explicit Git5s
Timeout. Both exact messages are `Timeout: Git ownership preflight timed out`,
with no `capacity_unavailable` suffix or counts. Two watchdogs expired60001ms with actual parent-observed SIGKILL, both
last boundary1/2 case6/resumed_terminal. Native dispatched=false/not_attempted
does not prove no Git spawn. Cause, contribution AND regression UNKNOWN. The
audit child44PASS/1ignored7.56s does not replace this failed full gate. Fmt,
all-target Clippy-Dwarnings and both BUILDs passed. All older35f0/fd406 and historical
full failures stay failed and separately scoped. Source5 outcomes and current CI
are recorded below; Root's disposition remains pending. PR64 stays DRAFT/HOLD; no native/F1/full60
or shared-producer readiness is granted by any of these controls or metadata.

### Source5 closed outcomes, precision dispositions and failed required CI

At immutable public e30ce2f, both independent Source5 native read-only commands
completed with actual owned cleanup verified before either findings body was read.
Both approve the limited test/docs delta, with zero C/H/M: ownership one Low,
capacity two Low. The [raw findings](issue-60-reader-source5-findings.json) retain
their launch-time pending-peer/CI notes, raw-result hashes and narrow approval
scope. They do not approve full TEST, merge, native/F1, whole60 or MVP acceptance.

All three Lows were verified from the same repository and original logs, then
corrected as evidence-only changes. No normative requirement, production/test
byte, deadline, observer/profile, latch or internal parallelism is changed:

- R23's compiled stale-selector operator killed the per-selector non-vacuity
  assertion only. Omit-skip wiring, scalar-exclusion and cardinality checks have
  passing positive controls and source reasoning, but no executed omission
  sensitivity. R23 earns none of those other arms.
- f072's full Debug/Release gates are single **local macOS** observations, now
  marked as such in the gate artifact and status paragraphs. Current Linux
  execution is separately recorded in the required CI below.
- f072 Release's reference-child line437 and resume-admission line479, plus
  fd406 Release's irrelevant-checkpoint line462, each contain the exact bare
  `Timeout: Git ownership preflight timed out`. No `capacity_unavailable` suffix
  or occupancy counts were printed; unavailable counts are not zero. The suffix
  exists only after an actual capacity wait; the bare text is also used for
  already-expired admission, observation expiry and cancellation. These messages
  do not identify which bare path occurred, whether Git spawned, or a cause,
  contribution or regression. Original logs were read, without any rerun.

[Required CI37268381965](issue-60-reader-source5-ci.json) at e30 **FAILED**.
Both OS jobs actually checked out c26ff80f94a56d91a47fa443c3415543adf611df,
parents26f3a48/e30ce2f, complete tree72e5c5eb equal to the triggering head.
Linux passed every job step, including the current42-entry inventory,
audit41PASS/1ignored3.09s and signal-parity control. macOS's audit separately
passed44/1ignored6.55s, but its full library failed313PASS/1FAIL/24ignored109.52s.
The failing Codex consumer was
`abnormal_supervisor_drop_clears_the_installed_live_approval_journal`, whose
session.rs6005 unwrap returned SessionLost during owned cleanup. Inspector facts:
deadline_loop_entry; stdout/stderr each1read,0bytes,1WouldBlock,EOFpending;
status1poll/pending, exit unavailable, validation not reached; observation310711us,
spawn2245us; inspector kill requested then direct-child wait/reap25us. These
inspector-child facts grant no owned target-group cleanup. Later macOS workspace
targets were unrun and both builds skipped. Linux success and the passing audit
are no substitute for the failed macOS required context. CI is DEBUG TEST plus
RELEASE BUILD, not RELEASE TEST.

This new Codex failure, f072 Release308/6 and all previous failed full gates remain
separately FAILED. Cause, contribution AND regression remain UNKNOWN. No old-head
rerun, timeout/latch reset, skip-only acceptance, serialization or deadline waiver
was used. All own Source5 suites/mutations/review commands are closed. PR64 stays
DRAFT/HOLD; Root's merge disposition is pending, and full60/native16/F1, shared
producer qualification, availability and recovery remain OPEN.

### Normal main custody composition777 and finite disposition

Normal merge7774b937 retained both public parents996d486 and new main9d2daed,
without conflicts. [Composition evidence](issue-60-reader-custody-composition1.json)
binds all16 incoming paths and all five changed Codex code blobs exactly to the
reviewed main. The complete Codex tree equals main9d; selected Git/adapter/Grok/
Context, Cargo and CI bytes equal reviewed996. The new attempt/custody/session
mechanics add20 actual custody controls to the ordinary suite inventory. Their
pool remains separate; ordinary production custody is None, the file factory is
cfg(test) only, and ordinary Codex EMPTY refusal stays before selected effects.
No shared/native capability, schema, producer or settlement grant is introduced.
The selected audit's45mac/42Linux inventory is unchanged; new Codex controls
outside its selectors earn no selected production-pool/Context/native proof.

At clean777, the first full DEFAULT local macOS Debug TEST and distinct first
full DEFAULT local macOS Release TEST each PASS445 top-level Rust tests+2doctests/
27ignored. Debug136.103s and Release159.734s include library334PASS/24ignored
44.73/46.35s and every later workspace target. Nested audit44PASS/1ignored
4.25/4.51s and ordinary-library child1PASS are separate from445. Fmt, all-target
Clippy-Dwarnings and both BUILDs also pass. Original command/log SHA records are
in the evidence artifact. No original budget, latch, internal default concurrency
or success predicate changed. This was a new actual combined source's first
run in each mode, not a same-failed-head retry. Peer66 reported merge preparation
and planned focused controls; possible overlap is not a failure-cause finding.
All owned suites/builds closed before evidence publication.

CI37270293297 passed **EVERY step BOTHOS**, DEBUG TEST plus RELEASE BUILD only.
Both actual checkouts f7981fd67f362ed2cbabce95b9410bb67f6b7eae have parents9d2/777;
tree4ec1af554b3107b7078d90b7e1d61eadf5cc374f equals the trigger. Normal fetch and
complete mode/type/blob/path comparison of all320 entries also match. The CI
Release BUILD is distinct from the separately executed local full Release TEST.

Both independent static read-only composition reviewers closed before results
were shared: Root and existing peer9 each APPROVE, zero C/H/M/L. Root independently
checked side-specific source identity, private/ordinary route boundaries, raw
gate hashes and top-level/nested count separation, both checkout lines, parents,
complete tree and every CI step; no independent tests/native inference were run.
Peer9's verdict/confidence0.94/STRICT and current gate checks are reported by Root;
no separate raw structured peer artifact exists, and none is invented. These
reviews cover only this finite composition, retaining earlier independent native
Source5 approvals and verified evidence-only Low corrections in their own scope.

Root's finite disposition permits normal **limited component** merge only after
the final outcome-only documentation delta receives direct QUICK verification
and its new required bothOS CI passes every step with actual complete-tree proof.
At that phase these were conditions, not premature fulfillment. The subsequent
final63d qualification and normal limited merge are recorded below. The mutation ledger remains27/23/25 plus two
historical masked survivors at zero credit; none was rerun for this composition,
and all R18/R23/diagnostic-only limits remain unchanged.

Old f072 Release308/6, e30CI Codex313/1 and all older failed gates remain FAILED,
cause/contribution/regression UNKNOWN. New combined passes neither fix nor
exonerate historical failures. Whole60/native3/backend/F1/MVP, durable recovery,
shared ProcessGroup/new/Drop and genuine effect/managed producer qualification
remain OPEN. No absent privileged runtime/profile/containment service is assumed.


### Final metadata head and merged component

At clean63d548c31c13c7eabcfb4e4a6c68a20f1a315fa4, the outcome-only delta comprised
eight docs: six current outcome/status records and two incoming main72 custody
status records. No crates/Cargo/CI byte changed from the both-mode-tested777. Root
direct QUICK verification approved this delta without findings, after the two
independent finite composition reviews had closed.

CI37271712764 passed every step on Linux and macOS. Both actual checkout logs
identify281f47aad585ebc494881adf31485e962993c878, parentsa9fc5e9/63d548c; complete
tree91eab742298d174e2824572d354b85653514360d, all321 mode/type/blob/path entries,
equals the reviewed final head. Root independently checked the parents, complete
tree, per-OS checkout logs and every step. CI supplies Debug TEST plus both BUILDs;
the distinct local777 Release TEST remains its own observation.

The final conditions were fulfilled before normal PR64 merge6146b1639bf03f244d1c5b716304dc1d20146194.
Its complete tree equals the reviewed final head. No force/admin bypass or whole
Issue60 close was used. This merges the selected mechanical Git reader/caller
lifetime component only. The historical failed observations, unknown causes,
finite mutation/audit limits and whole60/native/backend/F1/MVP gaps remain open.
