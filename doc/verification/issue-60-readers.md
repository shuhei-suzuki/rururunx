# Issue60 selected Git reader lifetime verification

Status: Requirements4 approved at a8e8a8b; Design2 approved at46f819b with verified Low precision; no reader
implementation or acceptance. Earlier diagnostic/environment partial gates are separate.
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
