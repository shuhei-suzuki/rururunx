# Issue 55 design: Grok-local terminal cleanup receipt

Risk: STRICT. Proposed, no Issue55 implementation. Requirements2 approved immutable
2ebbbfd. This design pins its six optional precision findings and leaves unavailable
shared cleanup cause explicitly unavailable. No schema/native environment/process-policy
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
| native_process_spawned | Existing process Some, independent of stderr task startup. |
| cleanup_ok | Exact cleanup.is_ok operand, true for current no-process Ok(None) too. |
| cleanup_state | not_attempted, group_cleanup_failed_unclassified, reap_timeout, reap_error, succeeded. |
| reap_io_kind | Fixed io::ErrorKind name only when locally measured; otherwise null. |
| output_verified | Exact existing boolean, with its deliberately limited meaning. |
| stderr_drain_state | not_started, joined_returned, joined_panic, joined_cancelled, budget_elapsed_abort_requested. |
| stderr_read_error | null/unavailable: shared drain returns (), so reading is not independently attested. |
| ownership_uncertain | Exact final total operand, derived once from labelled supervisor flags. |
| uncertainty_by_stage | Fixed native_child/pre_spawn/in_session_binding/reconciliation booleans. |
| dispatched/native_outcome | Actual existing Actor booleans, independent of clean or Lost. |

Retain existing prompt string; empty means absent for attempt matching. No new persisted
ordinal or recovery field is added. Attempt ordinal is a test/diagnostic lookup's position
of this Session among fully paged turn_observed events, not private runtime authorization.
Existing event sequence is only journal identity; Session/prompt/ordinal is never native
process identity or permission evidence.

## Measurements at actual boundaries

Before native spawn, native_process_spawned is false. Successful ProcessGroup creation
sets true from the existing process assignment, not from a PID lookup. If there is no
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

Only this supervise attempt's ProcessOwnership is labelled; checkpoint's separate
instance remains behaviorally unchanged and produces no turn receipt. Grok-local flag
metadata carries an enum stage alongside its existing Arc<AtomicBool>. Existing group()
retains resolved flags, creates the same new false flag and returns the same Arc. Stage
metadata never sets/clears a flag, changes ownership or creates another process. Fixed
call sites select stage before flag creation: initial verify_git/index preflight =>
pre_spawn; ProcessGroup::new => native_child; verify_binding before prompt =>
in_session_binding; verify_binding/index after cleanup => reconciliation. Avoid inferring
stage from a remaining PID, error message or later call position.

After reconciliation completes, evaluate the labelled flags once at the SAME final clean
point. Read their existing atomic values with the existing ordering, aggregate total OR
and fixed stage booleans, and use that exact total as clean's operand. Capture cleanup_ok
and output_verified at that same point. Equation stays cleanup_ok && !total_uncertain &&
output_verified; tests assert it from receipt and existing cleanup_verified. Sampling is
not kernel death proof. Correlated native flag remains true after cleanup/reap failure;
report it separately, not as a second independent failure. Other-stage true flags can
explain an unclean attempt even with native cleanup succeeded; receipt does not assign
an unavailable native IO cause to them. Full flag retention/clearing semantics and all
stage call sites enter source review. Unit stage projection is unit-only; real native
reconciliation-error causality remains unproven unless an actual consumer reaches it.

Existing result diagnostic priority (result, cleanup, reconciliation) and subsequent
unknown-dispatch wrapper remain byte-for-byte semantic equivalents. The independent
receipt survives regardless of that primary error. Existing clean still chooses PID
clear/Lost/Stopped/Exited/Failed; original native_outcome semantics remain (observed
owned stop reasons are not necessarily successful end_turn). Terminal save failure and
existing best-effort audit behavior are not converted into success; missing audit is a
failing/unavailable receipt, not reconstructed from Session state.

## Actual consumers and safe failure receipts

Use exact Task scope. Paginate Store.events(scope, after,100) by last sequence until
fewer than100 events, reject nonprogress/truncation or a safety bound rather than silently
accept partial results. Filter exact scope/kind/Session. Nonempty actual prompt matches
that attempt; empty prompt means absent, so use the ordered per-Session event ordinal
with complete paging and a before-launch sequence watermark. Require exactly one event
for the expected completed attempt; resume must not use an earlier receipt. For ordinary
fixture mode failures, exactly one own event exists; assert that rather than kind-only
find(). Duplicate/missing events fail. Lookup metadata is diagnostic only.

Before the existing pid.is_none assertion, construct an allowlisted safe projection of
cleanup_receipt and fixed own state from this exact event. The assert remains strict;
its failure message includes these facts, not arbitrary diagnostic/reconciliation text,
whole Event JSON or child output. Missing fields fail separately. Later modes remain
unobserved after a panic and are not called passes. Committed CI excerpts identify
verbatim/paraphrased fields, head/run/job/image and retention-limited log digests.

## Fixture isolation and dispatch coverage

Existing macOS sanitized parent remains env_clear: owned temp HOME, PATH=/usr/bin:/bin,
existing private entry marker only; no ambient variable/config/auth passthrough. Resolve
python3 only under that PATH. Fake ACP runs in owned temp executable/worktree/file-backed
Store with synthetic data. Do not run installed Grok/model/auth. Put the dispatch-capable
fake source in a shared test fixture file consumed by external tests and in-crate cfg(test)
consumer, preserving script bytes/protocol/argv; report content digest/source provenance.
Current fixture RRX metadata is only owned synthetic test data; migrate if normally
integrated Issue51 rejects it, without reopening a public runtime env channel. No new
Grok production environment policy belongs here. If interpreter/dispatch cannot run in
this isolation, leave acceptance blocked/source-only rather than widen ambient input.

The in-crate consumer uses the existing private ProcessInspectionPlan. On forcedUnknown
it delivers real KILL to the exact owned PGID, then a forced Unknown inspection result.
Adapter does not call verified reap on that failure; native grandchild reap is unobserved
(Tokio orphan handling). The parent owns/cleans/reaps only the sanitized test process,
not every native descendant's independent group. Do not claim forced cleanup succeeded
or use numeric PID to invent a verified native death. These measurement limits survive
receipt assertions. No shared test seam/process policy is added.

Pair three actual attempts: pre-dispatch protocol failure (historical cat, no dispatch
credit); dispatch-capable unowned-read/result failure with normally verified cleanup;
and the same dispatched result failure with forcedUnknown cleanup. The last reaches
Actor.dispatched=true/native_outcome=false and existing unknown-outcome rewrite. It must
record group_cleanup_failed_unclassified, native-stage uncertainty and clean=false,
while preserving the original protocol denial in existing diagnostic. The clean pair
may still be Lost because native outcome is unknown; its clean=true distinguishes that
cause without changing Lost or permitting replay. Assert durable executor reservation,
transport refusal and exact equation/event before wording. Normal known-completed fake
positive records succeeded, false uncertainty and existing actual exit/PID/transport
behavior. Reap-error/timeout/notstarted/join-state unit seams are labelled unit-only;
no injected syscall/native dead proof unless actually exercised safely.

ForcedUnknown and its consumer mutants are macOS-only. Linux runs clean receipts,
shared compilation and ordinary gates, not forced failure categories. Record each
causal run's platform. #16 installed native acceptance stays separate.

## Causal mutants and gates

On clean committed source, mutate removal of cleanup_receipt or its independent cleanup
state in the dispatched forcedUnknown actual consumer; kill the intended exact event
assertion, not preflight/error wording. A separate native-stage label omission/mislabel
operator must fail its real dispatched receipt assertion if meaningful. Stage aggregation
unit operators earn unit-only credit; unreachable/masked/equivalent failures earn none.
Compile each operator, record exact patch/base/mutant/control/source restoration and
failure, then restore exact source/control and normally remove isolated worktrees.
No diagnostic alone resolves real unclean native children or the current41red gate.

Two independent immutable Design1 gates precede implementation. Then commit scoped
code, targeted/default debug/release, fmt/all-target Clippy-Dwarnings, non-test builds,
causal mutants, two independent full actual-source reviews and verified fix/rereview.
Exact final Linux/macOS CI remains required. Preserve any genuine Unknown/timeout/latch/
PID failure and full original log; no deadline change, latch reset, test serialization,
same-head rerun/dummy commit or private config/auth bypass. Normally coordinate latest
main/native6/#51 without overlapping source edits or force pushes. Root owns merge/close.
