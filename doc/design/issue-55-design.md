# Issue 55 design: Grok-local terminal cleanup receipt

Risk: STRICT. Proposed Design2, no Issue55 implementation. Requirements2 approved immutable
2ebbbfd. Two independent Design1 reviews requested changes at14050d3. This design
corrects their verified five Medium findings and adopts their Low precision refinements.
Unavailable shared cleanup cause remains explicitly unavailable. No schema/native environment/process-policy
change. Public main80452f4 code is the baseline; source reviews must include full changed
consumer/helper/test source after actual implementation, not this proposed design.

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

Grok-local call-site tests execute successful preflight, owned native construction,
in-session verify and reconciliation helpers and inspect the last retained entry's
label before later group() drops its false flag. Non-native-stage omission/mislabel
operators earn unit/call-site credit only: the existing forced plan affects only the
native child, not Git helpers. A native-stage omission/mislabel can earn actual receipt
metadata credit on the dispatched forcedUnknown path; cleanup Err already forces
Lost, so it earns no independent clean/false-death safety credit. No new Git inspection
seam is introduced solely to overstate coverage.

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
shift identity. Empty prompt is absent. Prompt equality is an additional check only
when durable prompt_id is known, since actor.prompt may exist even when dispatching
publish failed. Resume cannot match a prior or later receipt.

Build allowlisted projection lookup/shape as a Result, without asserting or panicking
first. Run the existing strict pid.is_none assertion FIRST, with either the safe receipt
and fixed own state or fixed missing/duplicate/invalid reason. Then separately assert
receipt presence, exact key sets/vocabulary and equation. A missing/invalid receipt
must never hide the concrete PID assertion. Never include arbitrary diagnostic/
reconciliation text, whole Event JSON or child output in that failure projection.
Later modes remain unobserved after a panic. Durably retain verbatim/paraphrased failed
CI fields, head/run/job/image and finite-retention log digests.

## Fixture isolation and dispatch coverage

Existing macOS sanitized parent remains env_clear: owned temp HOME, PATH=/usr/bin:/bin,
existing private entry marker only; no ambient variable/config/auth passthrough. Resolve
python3 only under that PATH. On macOS /usr/bin/python3 can be a CLT/xcrun shim;
record the interpreter identity that actually ran as fixture provenance, independently
of env_clear. Shim failure/prompt blocks the fixture; never widen ambient settings.
Fake ACP runs in owned temp executable/worktree/file-backed
Store with synthetic data. Do not run installed Grok/model/auth. Put the dispatch-capable
fake source in a shared test fixture file consumed by external tests and in-crate cfg(test)
consumer, preserving script bytes/protocol/argv; report content digest/source provenance.
Current fixture RRX metadata is only owned synthetic test data; migrate if normally
integrated Issue51 rejects it, without reopening a public runtime env channel. No new
Grok production environment policy belongs here. If interpreter/dispatch cannot run in
this isolation, leave acceptance blocked/source-only rather than widen ambient input.

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
has dispatch_state=dispatching with durable prompt_id matching the own turn event; and
exactly one own grok.fs_observed event in the same attempt window matches Session,
prompt and the unseen.txt fs/read_text_file before evidence.callback rejects it.
The private Actor.dispatched flag alone attests durable intent plus attempted send,
not wire receipt; those independent facts must also pass in every credited mutant run.
No receipt-derived dispatch fact can serve as its own removal-mutant reachability oracle.

ForcedUnknown must reach actor.dispatched=true/native_outcome=false and existing
unknown-outcome rewrite. Assert group_cleanup_failed_unclassified, native-stage
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

Two independent immutable Design2 fix re-review approvals precede implementation. Then commit scoped
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
| ProcessOwnership flags/group/uncertain (ownership.rs29-44) | ownership.rs332-361 verify_binding/verify_git observe closure; mod.rs513 checkpoint, 998/999 preflight,1011 native construction,1038 in-session binding,1101/1107 reconciliation,1274-1283 index_digest | Private compatibility signature changes: required stage everywhere, retain true Arc/label pairs, authoritative OR independent of label. Checkpoint explicit unprojected variant. Actual non-native helper labels tested at call sites; flags/process semantics unchanged by proof/tests, not an assumed no-impact claim. |
| supervise cleanup/drain/clean/event (mod.rs1067-1180) | ProcessGroup cleanup_group (adapter.rs1430-1455), reap (adapter.rs group implementation), drain (adapter.rs1457-1492), completion/status/terminal save and Grok transport_succeeded | Diagnostic measurement added at existing branches/final clean point. Match same errors/results, preserve output_verified, priority, unknown rewrite, watch/save ordering. No added mutation/permission/terminal authority or schema because only existing event JSON grows. Shared helpers are read-only dependencies, not edited. |
| grok.turn_observed event shape | tests/grok.rs450 external hook_failure kind-only find; negative PID test381-486; in-crate mod.rs1290-1414 cat/unknown consumers; new dispatched consumer | Diagnostic compatibility: migrate all these lookups to exact scope+Session+bounded attempt window/complete paging, preserve original PID/state asserts before projection errors. Existing runtime has no receipt consumer; prove by source search rather than infer event data is harmless. |
| grok.fs_observed / saved recovery / marker | mod.rs882 fs audit before callback;1043-1046 dispatching publish/send; tests/grok.rs215 prompt marker and248 unowned_read | Test reachability dependency, no source behavior change there. Independent marker+durable dispatch+exact fs event precede mutated receipt assertion. Actor.dispatched alone is not wire proof. |
| Shared FAKE source move | tests/grok.rs171-264 constant and all external Fixture::new/adapter/mode consumers; new in-crate file-backed fake fixture | Test-fixture compatibility: include one shared fixture file, preserve script bytes/protocol/shebang/argv, publish digest and compare full existing consumers. No production executor factory/channel added. Existing RRX_ metadata remains synthetic until normal #51 integration; migration cannot expand caller environment grants. |
| Constants/non-code inputs | existing250ms reap mod.rs1072 and stderr join1087; shared250ms/1MiB inspection; PATH=/usr/bin:/bin, temp HOME/private parent entry marker mod.rs1294-1305; fake RRX_DATABASE/RRX_FOREIGN/RRX_MODE/RRX_PROMPT_OBSERVED | No budget/deadline/latch/serialization change. Parent env_clear preserves same intentional minimum; fixture has owned temporary script/CWD/database/canaries only. Record actual interpreter identity. File-backed Store mandatory because fake SQLite reads dispatch_intent; in-memory preflight fixture cannot silently substitute. No ambient auth/config/env inheritance or installed model test. |
| Audit append/paging | Store.audit state/mod.rs892; events976 and AuditEvent.sequence domain.rs442 | Existing best-effort event remains best-effort. No persistence API or SQL migration; diagnostics fail missing/duplicate/truncated event, never rebuild authority from status. Capture terminal upper watermark before next attempt; null/empty-prompt limitations remain explicit. |
| Unknown test plan / owned cleanup | adapter/inspection.rs macOS-only plan; adapter.rs ProcessGroup::Drop/cleanup_group; sanitized parent mod.rs1290-1332 | No plan/shared process change. Explicit group KILL, Drop retry group KILL and Tokio leader kill; native descendant reap unobserved. Parent group cleanup covers parent only. Linux clean receipts; no claimed forced-category Linux proof. |
| Integration/docs | requirements55, verification55 durable failed CI, master adapter/README; #41/#14/#43/#51/#16 | Requirements2 accepted behavior unchanged; design refines diagnostic private operation lookup into a bounded window, not event ordinal. Original #41 red and actual recovery/binding/environment/native gates remain open. No diagnostics-only claim of solving retained child. |
