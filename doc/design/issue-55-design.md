# Issue 55 design: Grok-local terminal cleanup receipt

Risk: STRICT. Design4 approved by both independent reviewers atadcb0637. The implemented
contract and verified correction rounds are recorded in [verification](../verification/issue-55.md).
Unavailable shared cleanup cause remains explicitly unavailable. No schema/native environment/process-policy
change. Public main80452f4 was the implementation baseline; source reviews included changed
consumer/helper/test source, with final narrow delta approved at66729b2.

## Diagnostic snapshot, not ownership authority

Add Grok-private serializable CleanupReceipt (new local module or private types) and
supervise-local measurement variables. Extend the existing grok.turn_observed JSON with
one cleanup_receipt object. Existing fields/primary diagnostic/reconciliation/exit values
and terminal Session/watch/audit ordering remain. The receipt contains fixed bounded enum
strings, booleans and null only. Never copy AdapterError.message, JoinError text, native
output/body, filesystem paths, environment/config, credentials, foreign identities or
process inventories. No new record/permission/runtime API; public Session/recovery and
native transport success are unchanged. Receipt fields cannot become a grant, reconnect,
reservation release or process-death predicate. Missing receipt fails diagnostics tests.

The bounded object has:

| Fact | Exact meaning |
| --- | --- |
| owned_process_group_created | Existing process Some: successful owned ProcessGroup construction, independent of stderr task startup; not an OS-spawn attestation. |
| cleanup_ok | Exact cleanup.is_ok operand, true for current no-process Ok(None) too. |
| cleanup_state | not_attempted, group_cleanup_failed_unclassified, reap_timeout, reap_error, succeeded. |
| reap_io_kind | Crate-owned mapping only for actual locally observed reap_error; null means not applicable for every other cleanup_state. |
| output_verified | Exact existing boolean, with its deliberately limited meaning. |
| stderr_drain_state | not_started, joined_returned, joined_panic, joined_cancelled, budget_elapsed_abort_requested. |
| stderr_read_error | Always the string unavailable, never null: shared drain returns (), so reading is not independently attested. |
| ownership_uncertain | Exact final total operand, OR of every ownership flag regardless of its label, with each flag loaded once at the final clean point. |
| uncertainty_by_stage | Fixed native_child/pre_spawn/in_session_binding/reconciliation booleans. |
| dispatched/native_outcome | Actual existing Actor booleans, independent of clean or Lost. |

The exact key set is owned_process_group_created, cleanup_ok, cleanup_state,
reap_io_kind, output_verified, stderr_drain_state, stderr_read_error,
ownership_uncertain, uncertainty_by_stage, dispatched and native_outcome. The nested
stage key set is exactly native_child, pre_spawn, in_session_binding and reconciliation.
No other key or string vocabulary is accepted by actual consumer assertions. All fields
except reap_io_kind are non-null. The reap mapping explicitly matches ErrorKind variants
NotFound=>not_found, PermissionDenied=>permission_denied, Interrupted=>interrupted,
InvalidInput=>invalid_input, InvalidData=>invalid_data, TimedOut=>timed_out,
WouldBlock=>would_block, UnexpectedEof=>unexpected_eof, BrokenPipe=>broken_pipe,
OutOfMemory=>out_of_memory and WriteZero=>write_zero, with all other/non-exhaustive
variants=>other. Never use Debug/Display text. A reap_error always has one mapped string;
null for other cleanup_state values means no applicable reap error, not proof of no IO
failure elsewhere. Unit mapping tests and exact consumer key/vocabulary assertions
prevent added message/body-derived fields. Preserve the existing event outside this object.

Retain existing prompt string; empty means absent. No new persisted ordinal or recovery
field is added. The requirements' private operation lookup is a caller-owned attempted
operation bounded by before-launch and terminal-observation sequence watermarks; it is
not a positional ordinal of best-effort events. Those window bounds identify the diagnostic
attempt even when an earlier audit is absent. Event sequence/Session/prompt never become
native process identity, permission evidence or reservation-release authority.

## Measurements at actual boundaries

Before successful ProcessGroup construction, owned_process_group_created is false.
Successful construction sets true from the existing process assignment, not from a PID
lookup. command.spawn() can succeed while ProcessGroup::new rejects the PID; then the
Tokio child is dropped under existing kill_on_drop, owned_process_group_created stays
false and no owned group cleanup/reap is attested. Do not call this no OS process. If there is no
process, keep existing cleanup Ok(None) and state not_attempted, with no fabricated reap.
On cleanup_group Err, retain the actual AdapterError for existing behavior, while receipt
state is group_cleanup_failed_unclassified. Both native kill/inspection and blocking
worker errors are folded by the shared helper into SessionLost plus formatted text;
no message substring/prefix classification and no finer IO/PS cause claim. A separately
coordinated typed shared API would be a future issue, not silently copied here.

If group cleanup succeeds, classify the existing local timeout(child.reap()) match at
its measurement site: timeout => reap_timeout; actual child.wait Err => reap_error and
its bounded io::ErrorKind; Ok => succeeded/actual existing exit.code. Build the same
AdapterErrors and Result<Option<i32>> as baseline. Classification never clears a flag,
changes a timeout, retries, reaps an uncertain numeric PID, or authorizes completion.

Await existing shared drain JoinHandle under the unchanged250ms budget. Shared drain
returns (), and swallows read errors after recording status.failure. Thus joined_returned
means only task returned within budget without panic/cancel; stderr_read_error remains
unavailable, not null-as-no-error. JoinError.is_panic()/is_cancelled() supplies fixed
states without its message. Timeout requests abort exactly as baseline; termination
unobserved. not_started covers stderr None, even if a native process spawned but failed
before RPC/drain setup. Do not replace shared drain or strengthen output_verified:
true by default and for any completed join; false only for timeout. The two facts are
explicitly distinct. No claim of complete output capture or absence of read failure.

## Stage labels and exact clean operands

Only this supervise attempt's ProcessOwnership is projected into the receipt.
Replace its private Vec<Arc<AtomicBool>> with Vec<(Arc<AtomicBool>, OwnershipStage)>;
no shared ProcessGroup or Store change. group(stage) requires an explicit stage, with
no cursor/default label. verify_git(request, ownership, stage), verify_binding(..., stage)
and index_digest(root, ownership, stage) thread that required argument to every flag
creation, including closures. Checkpoint passes an explicit Checkpoint variant which
never gets a turn receipt; its lifecycle/flag behavior stays unchanged. An omitted
argument is a compile error. No label is inferred from PID, diagnostic or call position.

Keep the actual retain predicate: group(stage) drops entries whose flag currently loads
false (resolved), retains every true (still uncertain) entry together with its label,
then creates/pushes the same new false flag. Never alter set/clear behavior, group
ownership or process lifecycle. Initial verify_git/index preflight uses PreSpawn;
ProcessGroup::new uses NativeChild; verify_binding before prompt uses InSessionBinding;
post-cleanup verify_binding/index uses Reconciliation. All call sites, including
checkpoint and every bounded Git observe closure, enter source review.

AFTER reconciliation, at the SAME final clean point, load every retained flag ONCE with
SeqCst. Build the authoritative total by OR of all loaded values regardless of label,
and the four diagnostic stage booleans from those same values. An unprojected Checkpoint
label, wrong label or future variant cannot remove a true flag from total. Preserve
cleanup_ok && !total_uncertain && output_verified exactly; label metadata cannot weaken
clean. Unit tests prove total equals legacy uncertain() for arbitrary labelled flag
sets, including Checkpoint, and retain keeps Arc/label pairs together. Per-stage false
means only no retained uncertain entry for that stage, never stage reached/verified/
resolved. reconciliation_attempted remains the independent reachability fact. The
single sample is not kernel death proof. Native flag remains correlated with cleanup/
reap failure. A true other-stage flag may explain unclean despite native cleanup success,
without claiming an unavailable native IO cause.

Helper-level stage tests alone earn UNIT threading/projection credit. They cannot
observe a wrong stage argument at supervise, and their passing result must not be
called supervise call-site credit. Add a cfg(test)-only append-only creation trace in
ProcessOwnership::group(stage), completely separate from flags/retain/sampling. Under
cfg(test), OwnedEntry supplies an Arc<Mutex<Vec<(input_version, OwnershipStage)>>>;
supervise installs that trace tagged with its existing PreparedInput.version. It never
clears earlier entries; fresh-input resume versions already distinguish attempts. A
checkpoint's separate ownership has no trace. No production field/API/environment,
persisted ordinal, stage cursor or ownership authority is introduced. Test trace append happens AFTER the unchanged flag push and uses try_lock with
discarded Result, ignoring poisoned/contended trace without blocking or panicking. A
poison/held-lock unit control proves group still returns the identical flag and retain/
total semantics. No unwrap on diagnostic trace. Test trace writes
never set/clear flags, affect retain/total or authorize death; source review compares
release/debug non-test paths. Trace exists only inside crate tests for the supervise attempt and is not emitted
in audit or consumer failure logs.

An actual in-crate supervise consumer filters the trace to the exact attempted input
version and asserts the ordered full phase blocks: PreSpawn+, one NativeChild,
InSessionBinding+, Reconciliation+ for the dispatched clean pair; PreSpawn+ only for
before-spawn stop. The known-completed fake also exercises the full sequence. A separate macOS in-crate
sanitized before-spawn-stop attempt uses the local owned fixture, calls stop immediately
after start (as the existing external test), waits for actual Stopped terminal and reads
this entry trace at that attempt input version. It expects PreSpawn+ only, no NativeChild/
InSessionBinding/Reconciliation/Checkpoint. Trace consumers/creation-call-site mutants
are macOS-only; external both-OS stop tests validate receipt only, never access this
cfg(test) trace. Linux has no claimed trace credit. Each +
is one-or-more flag creations from the actual bounded Git calls; order is strict and
there can be no extra Checkpoint or out-of-order labels. Order alone would miss
reconcile_binding mislabelled InSessionBinding, which merges into the previous block.
Therefore also assert block sizes from this SAME role/scope/attempt: |PreSpawn| ==
|Reconciliation| == |InSessionBinding| + 1 and |NativeChild| == 1. verify_git is the
same n-child path at these three boundaries; preflight and reconciliation additionally
run one index_digest each. Executor Task n=9 in current source (Reviewer adds status),
but the relational oracle need not hardcode n. A compiled reconcile_binding ->
InSessionBinding operator must be killed by this actual size-relation assertion.
This observes the REAL supervise argument instead of only proving a helper copied it.
Non-native-stage swaps then earn actual creation-call-site metadata credit; they still
do not attest true Git uncertainty/reap failure. The forced plan affects only the native
child. Native omission/mislabel likewise earns receipt metadata credit, never independent
false-death safety credit because cleanup Err already forces Lost.

Actual supervise receipts additionally assert total_uncertain equals OR of their four
stage booleans. Checkpoint/future unprojected true labels inside supervise thereby fail
the diagnostic oracle while remaining counted by the AUTHORITATIVE all-flags total.
Standalone projection units include Checkpoint and assert the broader authoritative
OR; they do not claim four-stage equality for a non-supervise instance. No new Git
inspection seam is introduced to fabricate true other-stage evidence.

Existing result diagnostic priority (result, cleanup, reconciliation) and subsequent
unknown-dispatch wrapper remain byte-for-byte semantic equivalents. The independent
receipt survives regardless of that primary error. Existing clean still chooses PID
clear/Lost/Stopped/Exited/Failed; original native_outcome semantics remain (observed
owned stop reasons are not necessarily successful end_turn). Terminal save failure and
existing best-effort audit behavior are not converted into success; missing audit is a
failing/unavailable receipt, not reconstructed from Session state.

## Actual consumers and safe failure receipts

Use exact Task scope. Capture a before-launch sequence watermark and a terminal-
observation watermark BEFORE any next launch/resume. Paginate Store.events(scope, after,
100) through that closed-upper-bound window by last sequence, reject nonprogress/
truncation or an exceeded safety bound rather than silently accepting partial results.
Filter exact scope/kind/Session and window (before_launch, terminal_observed]. Require
exactly one event. No positional event ordinal; missing best-effort prior audit cannot
shift identity. Empty prompt is absent. Prompt equality is an additional check only for a CURRENT durable prompt: saved
dispatch_intent.input_version equals this attempt input version and matching
session.saved dispatch_intent evidence exists inside this same attempt window. Resume
before session/load can retain prior recovery prompt/dispatching/intent; those are not
current reachability or prompt-equality evidence. If current durable evidence is absent,
skip prompt equality and retain exact Session+window lookup. actor.prompt may exist
when dispatching publish failed. Resume cannot match a prior or later receipt.

Immediately after terminal observation/window capture, build allowlisted projection
lookup/shape as a Result, without asserting/panicking first. Preserve the ORDER and
strict predicates of every existing state, transport, failure-present, PID and related
per-attempt assertion; append the same safe projection or fixed missing/duplicate/
invalid reason to EACH message. Include external negative, execute/structured/stop/
parent-replacement consumers and in-crate cat/dispatched attempts. A Failed/Exited/
Stopped-expected case with actual Lost must expose the same receipt at its EARLIER
state assertion, not wait for the PID assertion. Keep any existing primary diagnostic
message where it was; add no arbitrary new text. Projection Err cannot hide state/PID
failure. Only afterward assert receipt presence/shape/vocabulary/equations. No whole
Event JSON/child body/environment/path is added to receipt failure messages.

Include one shared test-support file via include/path in both external tests/grok.rs
and in-crate consumers, alongside the shared fake. It contains the closed projection
formatter and STRICT state-assert helper (generic over the existing state values), with
a fixed assertion label and panic location; no production dependency/public runtime API.
All these consumers use the same formatter and state helper without changing predicates
or ordering. State-helper mutation is observed by the actual control; attachment at
individual other external sites remains source-review-only, not falsely killed by an
in-crate-only seam. Transport/failure/PID predicates stay strict with shared formatted
projection appended.

A dedicated sanitized actual Failed-expected protocol attempt with forcedUnknown
executes the existing strict state assertion (actual Lost) with projection and fails.
Its owning parent requires nonzero exit, exactly one executed test/one failure, the
fixed state-assertion label and actual shared-helper panic location, and assert_eq
left Lost/right Failed, PLUS bounded allowlisted cleanup category/total/stages in that
same failure message. An earlier/shape/incidental-print panic cannot satisfy the oracle.
Only then treat the expected STATE assertion failure as diagnostic control success. It must not call that child a passed cleanup/state attempt. A compiled
projection-omission operator AT THE SHARED state helper/formatter kills that parent
oracle after the actual state assertion; exact compiled panic/assertion location is
recorded per baseline/mutant. Do not attribute per-external-call-site kills to it. Keep this actual
message-consumer credit separate from cleanup-receipt removal mutants and true cleanup
safety. Later loop modes are unobserved after panic. Durably retain verbatim/paraphrased
failed CI fields, head/run/job/image and finite-retention log digests.

## Fixture isolation and dispatch coverage

Existing macOS sanitized parent remains env_clear: owned temp HOME, PATH=/usr/bin:/bin,
existing private entry marker only; no ambient variable/config/auth passthrough. Resolve
python3 only under that PATH. On macOS /usr/bin/python3 can be a CLT/xcrun shim;
Add one explicitly test-fixture-only line to the shared fake, gated by an owned
RRX_PYTHON_OBSERVED marker path, writing only sys.executable and sys.version_info to
that owned file. This observes the actual attempt interpreter independently of env_clear,
not a separate probe or process inventory. Record that additive line and new script
digest; all original protocol/shebang/argv behavior stays unchanged when absent. Shim failure/prompt blocks the fixture; never widen ambient settings.
Fake ACP runs in owned temp executable/worktree/file-backed
Store with synthetic data. Do not run installed Grok/model/auth. Put the dispatch-capable
fake source in a shared test fixture file consumed by external tests and in-crate cfg(test)
consumer; the sole additive interpreter-marker line is the documented script-byte
change. Verify all other original script bytes/protocol/argv, and report both old/new
digests/source provenance.
Current fixture RRX metadata is only owned synthetic test data; migrate if normally
integrated Issue51 rejects it, without reopening a public runtime env channel. No new
Grok production environment policy belongs here. If interpreter/dispatch cannot run in
this isolation, leave acceptance blocked/source-only rather than widen ambient input.

Build the dispatched in-crate fixture locally rather than reuse/modify the shared
Generic preflight_fixture: owned temporary canonical source+task worktree, repository_identity,
committed own.txt, unseen.txt precreated BEFORE launch and baseline capture, file-backed
Store and actual 40-hex HEAD input revision. Synthetic RRX_DATABASE/RRX_MODE/
RRX_PROMPT_OBSERVED/RRX_PYTHON_OBSERVED plus an owned temporary RRX_FOREIGN
canary path live only in this owned fake test. The known-completed path dereferences
that foreign canary unconditionally; assert its file is unchanged/not created after
the denied write. These are synthetic fixture inputs only; no real
auth/settings/config lookup. Generic preflight_fixture/fixture_request and all their
existing callers retain their memory Store/empty commit/PATH contract unchanged.
Separately sanitize each diagnostic attempt in its own owning child selected by a
fixed test name, using existing parent env_clear/home/PATH/entry marker and cleanup
pattern. Cat, clean dispatched, forced dispatched and expected-state-failure diagnostic
controls have separate results. A clean-pair panic cannot prevent observing the forced
consumer in a different test. Default harness concurrency stays unchanged; no broad
suite serialization or acceptance rerun. If an attempt did not run/pass its prerequisites,
report that independently rather than credit a later receipt.

The in-crate consumer uses the existing private ProcessInspectionPlan. On forcedUnknown
it delivers real KILL to the exact owned, still-unreaped PGID, then a forced Unknown
inspection result. cleanup_group drops the child after Err; ProcessGroup::Drop retries
KILL/forced inspection on that same still-owned PGID. Tokio kill_on_drop then targets
its owned leader PID. These are signal deliveries, not verified cleanup or independent
reap. Adapter does not call verified reap on that failure; native grandchild reap is unobserved
(Tokio orphan handling). The parent owns/cleans/reaps only the sanitized test process,
not every native descendant's independent group. Do not claim forced cleanup succeeded
or use numeric PID to invent a verified native death. These measurement limits survive
receipt assertions. No shared test seam/process policy is added.

Pair three actual attempts: pre-dispatch protocol failure (historical cat, no dispatch
credit); dispatch-capable unowned-read/result failure with normally verified cleanup;
and the same dispatched result failure with forcedUnknown cleanup. BEFORE ANY receipt
assertion in BOTH dispatched consumers, assert independent reachability: the owned
RRX_PROMPT_OBSERVED marker exists (actual fake received prompt); saved Session recovery
has dispatch_state=dispatching with CURRENT durable prompt_id/input_version and
matching session.saved evidence in the attempt window; and exactly one own
grok.fs_observed event in that window matches Session, durable prompt, unseen.txt,
method fs/read_text_file, succeeded=true and effect_may_have_occurred=false BEFORE
evidence.callback rejects it. No turn_observed/cleanup_receipt lookup participates
in these prerequisites; all asserted turn data belongs to the subsequent receipt
oracle. This prevents a missing unseen.txt read (succeeded=false/continued native
response) from earning result-error reachability credit.
The private Actor.dispatched flag alone attests durable intent plus attempted send,
not wire receipt; those independent facts must also pass in every credited mutant run.
No receipt-derived dispatch fact can serve as its own removal-mutant reachability oracle.

ForcedUnknown must reach actor.dispatched=true/native_outcome=false and existing
unknown-outcome rewrite, existing turn completed=false and receipt native_outcome=false.
Assert group_cleanup_failed_unclassified, native-stage
uncertainty and clean=false, while retaining original protocol denial in the existing
diagnostic. Assert durable executor reservation, transport refusal and exact equation.
The clean pair may still be Lost because native outcome is unknown; clean=true only
distinguishes that cause, never releases it or permits replay. If this nominal clean
pair genuinely returns clean=false/PID retained (as #41 may reproduce), record the same
safe receipt as a real failure/acceptance blocker; no fixture changes, serialization or
rerun to erase it. A known-completed fake positive records succeeded/false uncertainty
and existing actual exit/PID/transport behavior.

Use existing native_stop_permissions_foreign_refs_and_environment_guards_are_explicit
before-spawn stopped attempt on BOTH OS for a real not-attempted receipt: exact Session/
window, owned_process_group_created=false, cleanup_ok=true, cleanup_state=not_attempted,
output_verified=true, stderr_drain_state=not_started, all stage flags false,
ownership_uncertain=false, cleanup_verified=true, SessionState::Stopped; no process-
spawned event. Its actual reachability earns lifecycle credit per OS, not unit-only.
Reap error/timeout, join panic/cancel and budget_elapsed categories remain unit-only or
source-only unless safely reached; do not invent syscall or native-death observations.

ForcedUnknown and its consumer mutants are macOS-only. Linux runs clean receipts,
shared compilation and ordinary gates, not forced failure categories. Record each
causal run's platform. #16 installed native acceptance stays separate.

## Causal mutants and gates

On clean committed source, mutate removal of cleanup_receipt or its independent cleanup
state in the dispatched forcedUnknown actual consumer; after the three independent
dispatch prerequisites pass, kill the intended exact event assertion, not preflight/error
wording. Include a distinct result-error-conditional operator (e.g. publish cleanup
facts only when result.is_ok, or derive cleanup_state from the primary diagnostic) that
the same dispatched result-error oracle must kill. Record which prerequisite assertions
passed and exactly which receipt assertion failed per run. A separate native-stage label omission/mislabel
operator must fail its real dispatched receipt assertion if meaningful. Stage aggregation
unit operators earn unit-only credit; unreachable/masked/equivalent failures earn none.
Compile each operator, record exact patch/base/mutant/control/source restoration and
failure, then restore exact source/control and normally remove isolated worktrees.
No diagnostic alone resolves real unclean native children or the current41red gate.

Two independent immutable Design4 fix re-review approvals precede implementation. Then commit scoped
code, targeted/default debug/release, fmt/all-target Clippy-Dwarnings, non-test builds,
causal mutants, two independent full actual-source reviews and verified fix/rereview.
Exact final Linux/macOS CI remains required. Preserve any genuine Unknown/timeout/latch/
PID failure and full original log; no deadline change, latch reset, test serialization,
same-head rerun/dummy commit or private config/auth bypass. Normally coordinate latest
main/native6/#51 without overlapping source edits or force pushes. Root owns merge/close.


## Impact analysis (baseline main80452f4)

Line references below name existing consumers before edits; source review checks their
new locations. Every change is diagnostic/compatibility/test-fixture only. Authority,
permissions, budgets, reservations and process handling stay as in the supplied source.

| Target / input | Existing consumers | Impact and handling policy |
| --- | --- | --- |
| ProcessOwnership flags/group/uncertain (ownership.rs29-44) | ownership.rs332-361 verify_binding/verify_git observe closure; mod.rs513 checkpoint, 998/999 preflight,1011 native construction,1038 in-session binding,1101/1107 reconciliation,1274-1283 index_digest | Private compatibility signature changes: required stage everywhere, retain true Arc/label pairs, authoritative OR independent of label. Checkpoint explicit unprojected variant. Helper threading is unit-only; cfg(test) append-only trace checks actual supervise argument order; flags/process semantics unchanged by proof/tests, not an assumed no-impact claim. |
| supervise cleanup/drain/clean/event (mod.rs1067-1180) | ProcessGroup cleanup_group (adapter.rs1430-1455), reap (adapter.rs group implementation), drain (adapter.rs1457-1492), completion/status/terminal save and Grok transport_succeeded; cfg(test) OwnedEntry creation trace | Diagnostic measurement added at existing branches/final clean point. Match same errors/results, preserve output_verified, priority, unknown rewrite, watch/save ordering. No added mutation/permission/terminal authority or schema because only existing event JSON grows. Shared helpers are read-only dependencies, not edited. |
| grok.turn_observed event shape | tests/grok.rs450 external hook_failure kind-only find; negative PID test381-486; in-crate mod.rs1290-1414 cat/unknown consumers; new dispatched consumer | Diagnostic compatibility: migrate all these lookups to exact scope+Session+bounded attempt window/complete paging, preserve original predicate/order and attach bounded projection to ALL existing per-attempt state/transport/failure/PID assertion messages before separate shape assertions. Existing runtime has no receipt consumer; prove by source search rather than infer event data is harmless. |
| grok.fs_observed / saved recovery / marker | mod.rs882 fs audit before callback;1043-1046 dispatching publish/send; tests/grok.rs215 prompt marker and248 unowned_read | Test reachability dependency, no source behavior change there. Independent marker+durable dispatch+exact fs event precede mutated receipt assertion. Actor.dispatched alone is not wire proof. |
| Shared FAKE source move | tests/grok.rs171-264 constant and all external Fixture::new/adapter/mode consumers; new in-crate file-backed fake fixture | Test-fixture compatibility: include one shared fixture file, one additive env-gated interpreter marker line; all other script bytes/protocol/shebang/argv unchanged. Publish old/new digest and compare all existing consumers. No production executor factory/channel added. Existing RRX_ metadata remains synthetic until normal #51 integration; migration cannot expand caller environment grants. |
| Constants/non-code inputs | existing250ms reap mod.rs1072 and stderr join1087; shared250ms/1MiB inspection; PATH=/usr/bin:/bin, temp HOME/private parent entry marker mod.rs1294-1305; fake RRX_DATABASE/RRX_FOREIGN/RRX_MODE/RRX_PROMPT_OBSERVED/RRX_PYTHON_OBSERVED; shared Generic adapter.rs1719-1799 preflight_fixture/fixture_request and their generic/oldcat callers | No budget/deadline/latch/serialization change. Parent env_clear preserves same intentional minimum; fixture has owned temporary script/CWD/database/canaries only. Record actual interpreter identity. New Grok-local canonical file-backed fixture with committed own.txt/precreated unseen.txt/actual HEAD. Shared Generic memory/empty-commit/PATH builders and all their consumers are read-only dependencies, unchanged; never silently substitute them. No ambient auth/config/env inheritance or installed model test. |
| Audit append/paging | Store.audit state/mod.rs892; events976 and AuditEvent.sequence domain.rs442 | Existing best-effort event remains best-effort. No persistence API or SQL migration; diagnostics fail missing/duplicate/truncated event, never rebuild authority from status. Capture terminal upper watermark before next attempt; null/empty-prompt limitations remain explicit. |
| Unknown test plan / owned cleanup | adapter/inspection.rs macOS-only plan; adapter.rs ProcessGroup::Drop/cleanup_group; sanitized parent mod.rs1290-1332 | No plan/shared process change. Explicit group KILL, Drop retry group KILL and Tokio leader kill; native descendant reap unobserved. Parent group cleanup covers parent only. Linux clean receipts; no claimed forced-category Linux proof. |
| Integration/docs | requirements55, verification55 durable failed CI, doc/design/master/agent-adapter.md Grok terminal supervision section and README; #41/#14/#43/#51/#16 | Implementation PR writes current key set/vocabulary/stage semantics/measurement limits in the master terminal-supervision section without issue-number framing, plus pending integration links. Requirements2 accepted behavior unchanged; design refines diagnostic private operation lookup into a bounded window, not event ordinal. Original #41 red and actual recovery/binding/environment/native gates remain open. No diagnostics-only claim of solving retained child. |


## Independently reviewed inspection test integration

Root authorizes normal integration of the independently reviewed test-only commit
3d3039932159400017fb1562b9df8484edd5da05 (parent6982708f1f98cb43b6536f1c52cdde8a9ca57f62)
when this design gate resolves, not the unmerged full preparation-owner feature.
The actual one-file inspection.rs delta has37 insertions/4 deletions, entirely inside
cfg(test): separate oversized child-wrapper Unknown control (exact size OR actual timeout)
from deterministic prepared Stream::drain cap unit at LIMIT+1 with all-Z prefix before
hidden live row. Production250ms/1MiB/parsing/EOF/status/leader/latch/PID authority is
unchanged. Original55Design3 CI37196875050 failed unchanged legacy wrapper on macOS:
stdout InvalidData185.472708ms, stderr TimedOut277.761792ms,86PASS/1FAIL/1IGN;
Ubuntu fail-fast CANCELLED. Original log retained, cause unspecified, no rerun.
This is an independently meaningful test-boundary correction, not resolution of Grok
retained PID or inspection Unknown. Source reviews must include the integrated exact
inspection file and actual current cap-unit mutant/restored control, with wrapper
Unknown/no-accepted-frame credit distinct from deterministic cap-unit causal credit.
Default debug/release and final exact Linux/macOS CI cover the combined source. Its
production blob is compared to reviewed46; no scheduling/deadline/serialization change.
