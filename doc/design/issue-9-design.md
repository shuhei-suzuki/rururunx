# Issue 9 design preparation: Review Set authority and rounds

Status: proposed interfaces, not implementation. Base is merged main 1c44316.
Integrate the reviewed context-pack/frame authority and native provider contracts
before production work; do not guess an unmerged kernel signature.

## Types and boundaries

Use typed ReviewPolicy {slots, completion, quorum, parallelism, timeout,
require_clean, require_lock}, immutable ReviewerSlot {id, agent, model, effort},
and scoped ReviewSet/ReviewRound envelopes. Validate distinct slot IDs, capability
and agent registration, bounded counts, positive timeouts and 1 <= N <= M.
`all` requires M eligible approvals; `any` requires one; `quorum` requires N.
Keep opinion eligibility separate from settled native ownership and verified
blocking findings. Exact bounds and configuration plumbing are finalized before
implementation and recorded in the persisted policy snapshot.

A round captures its factual bundle through a provider port with exact scope,
revision, source hashes, complete-frame hash, context version and bounded payload.
Freeze one factual bundle for the independent roster. Different per-slot native
model/effort are transport configuration, not edits to facts. A later-round bundle
may carry explicit baseline/new revisions, verified unresolved findings, fix and
verification refs through the deterministic delta provider. Never derive a delta
from transient chat or simply relabel an old bundle with a new HEAD.

Per-slot state is Reserved → Starting → Running/Waiting → terminal outcome. Record
input-consumption intent before native wire through the adapter's actual private
reservation. Session ID/configuration/frame ownership comes from an atomic slot
reservation, not from arbitrary Session.recovery JSON. Agent outputs are parsed
into bounded typed findings/result; malformed or ambiguous output is a failure
or human-held result. Native transport completion alone is not review approval.

## Actual Workflow port

Add a real ReviewRunner delegation for Reviewer phases. The Workflow owns one
exact dispatch claim and immutable context; the ReviewRunner owns its bounded
roster of sessions. A single PhaseAttempt.session_id cannot represent that roster.
Publish a scoped ReviewSet reference/attempt identity atomically and poll the
ReviewRunner result. Default legacy single-reviewer behavior remains explicitly
separate until this port is integrated and tested.

Extend kernel frame admission narrowly: an exact active ReviewSet/round/slot may
bind a native Reviewer to the owned Workflow claim and the same factual frame.
Use private authoritative slot allocation, expected agent/model/effort, exact
Session ID once bound, target hashes and context version. Do not remove the
existing phase-agent/session guard to make parallel launches pass. Executor,
Consultant and ApprovalReviewer roles cannot use this delegation. Formal approval
review remains on its separate operation-free decision Task.

Lock/canonical Git/FD/source checks run outside SharedStore. Snapshot authority,
observe native sources with bounded helpers, then revalidate the exact live
Workflow/ReviewSet/slot/source CAS immediately before publication and input
consumption. Read-only native permissions plus clean/source checks enforce the
review target; a lock record does not itself prove external processes cannot
write. Once a dispatch could have reached a backend, cancellation/drop retains
its ownership until authoritative terminal/cleanup evidence.

## Rounds and certificates

Persist every slot outcome before policy evaluation. Preserve raw individual
findings independently until the round completes; then use scoped verifier
records {finding_ref, target, decision, evidence, inspected_count, changed_count}.
Verified blockers veto success even if other slots approve. False positives do
not trigger fixes. Human judgment holds the round. The remediation path creates
explicit verified-fix/commit/required-check evidence before the next immutable
round; no old outcome is overwritten or promoted to a new revision.

Issue one immutable review certificate only after policy eligibility, required
finding verification, current target integrity and safe session settlement all
hold. Bind it to ReviewSet/round, actual target, bundle hash, context version,
roster/policy, verifier/evidence refs and individual terminal outcomes. The
Workflow gate validates that certificate against its actual phase claim; the
certificate is evidence, not permission to merge or execute a command.

## Persistence and tests

Use private scoped transaction helpers for new authoritative Review records/slot
reservations, with one active round per ReviewSet and one live launch per slot.
Generic old Record writers must not mutate this contract. Add an ordered schema
marker after the actual merged context base, with migration/reopen and native
old-writer refusal proof. Scope indexes keep current lookup bounded; history is
append-only and carries immutable provenance. Uncertain restart remains held for
recovery rather than silently scheduling replacement sessions.

Tests combine fast controlled policy/verification cases with real temporary Git,
actual Store/Workflow dispatch, native bounded subprocess fixtures and immutable
source/lock races. Record all failure categories and mutation controls. Final
integration must show the real multi-reviewer Workflow path, and later native
model dogfood must show two/triple reviewer sessions and equivalent factual
inputs rather than counting independent development reviews as runtime behavior.
