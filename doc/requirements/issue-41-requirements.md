# Issue 41: preserve live pre-dispatch Workflow reservations

Risk: STRICT, shared Workflow concurrency and durable reservation boundary.
Depends on Issue 8. Consumers include Review, Scheduler, Goal loop and recovery.

## Problem

A Running attempt may have neither Session nor dispatch marker while its owner
is awaiting source/policy preparation after reserving the phase. Ordinary `poll`
currently assumes that this owner stopped, marks Failed and clears the active
attempt. A concurrent observer thereby prevents the live owner's launch CAS.
Public CI 37123017200 at e67c38b observed zero launches instead of one in the
existing concurrent-step regression; macOS was cancelled through matrix failfast.

## Required behavior

1. For Executor/Reviewer attempts, ordinary observation cannot infer owner absence
   from missing Session or `dispatch_started=false`. Preserve the preparation
   reservation and report Waiting with an explicit preparing/recovery explanation.
2. Such observation does not change Task, Workflow, ContextVersion or audit.
3. While one owner is suspended after reservation, a concurrent `step` must
   leave it able to publish exactly one native Session after preparation resumes.
4. A new Engine or process observing an abandoned undispatched reservation also
   preserves it. For nonterminal Tasks, explicit recovery must establish
   preparation-owner absence and no dispatch before releasing it; that
   integration remains Issue 14. The existing explicit TerminalRecovery path
   remains valid after a committed terminal decision: its transactional terminal
   Task fence prevents further owner dispatch, and no Session/dispatch marker
   plus executor/Lost fences prove safe closure without owner-absence inference.
5. Dispatched or unknown native outcomes remain reserved. No automatic replay,
   fabricated process death, timeout relaxation or permission change.
6. A typed, pre-commit marker CAS loss for the owning Task row may release only
   its proven undispatched claim while preserving changed user metadata.
   Project/Goal/Record version conflicts do not qualify. A failed owner is
   distinguishable only through trusted recovery evidence, not a polling snapshot.
7. For Executor/Reviewer attempts only, owner-local release is available only
   after this invocation's own reservation
   commit has returned success, and only for preparation errors without a
   definitive Failed/Invalidated decision. Reservation CAS losers and unknown
   reservation commit outcomes release nothing. Capture the committed Workflow
   record identity/version and exact original attempt data as this invocation's
   in-memory ownership token; release requires that committed Record version
   still matches. No persisted attempt field or schema change is introduced.
   The generation/index/context/agent tuple alone is insufficient.
   Before returning a preparation error, re-read and CAS-release only that exact
   still-Running attempt, with no Session or dispatch marker, recording Failed
   and a factual RetryEvent while preserving concurrent user metadata. Project,
   Goal and Task must still be active. Inactive/terminal owners retain the
   reservation without owner-local changes to Task, decision, context or history;
   explicit TerminalRecovery remains available as described in requirement 4. Pause
   then resume alone does not release it. Explicit recovery in #14 must resolve
   this retained reservation. Conflicting attempts, live/Lost Sessions or failed
   release remain reserved. Never release after dispatch may have committed.
   Process crash and dropped futures remain explicit-recovery cases. A CAS loss
   while publishing a definitive decision retains the reservation for recovery;
   it does not convert the decision into an automatic retry.
   Disable release eligibility before fail/invalidate/hold and dispatch-marker
   publication. Only the marker transaction's typed StateGuardError::SnapshotChanged
   with table `tasks` AND id equal to the owning Task ID proves the rollback that
   restores eligibility. Marker Project/Goal/Record conflicts, other Task IDs,
   untyped or unknown marker/commit errors, and every definitive-publication error
   remain reserved. Release CAS conflicts and executor/Lost fences never retry
   their release.
8. Running EvidencePort attempts retain the existing claim-CAS evaluation path;
   its state mutation/port invocation is not covered by requirements 1–4. Unknown
   evaluating claims and irreversible outcomes retain their existing fences.
   Pre-claim capture/refresh/CAS errors leave the same Running attempt, history,
   retries and ContextVersion intact. They never use agent owner-local release.
9. Both no-Session observer reasons must state preparation/launch may still be
   active and must not assert interruption. Status/Scheduler use durable attempt
   actor/state/Session/dispatch fields to distinguish these waits without parsing
   human-facing reason text. A genuine orphan retains its phase/reservation;
   Project removal and conflicting execution remain fenced until trusted recovery
   resolves it; committed cancellation may use existing TerminalRecovery.
   Process/owner restart recovery belongs to #14;
   external GitHub/irreversible port outcome reconciliation belongs to #13.
10. Once dispatch may have committed, all errors retain the reservation. This
    includes a Session-binding CAS loss after an adapter returns an owned Session;
    #14 must reconcile it. This issue does not claim post-dispatch liveness under
    metadata conflicts or change the existing Session acknowledgement policy.
11. This PR must align README status, master Workflow design and code recovery
    references with the #13 external/irreversible versus #14 owner/restart split.
    Record the retained classes below consistently. This issue persists no
    owner-absence evidence; #14 must establish its own trusted proof.

Issue #14 must reconcile orphaned undispatched attempts, dropped futures/crashes,
inactive-owner preparation errors, definitive-decision publication conflicts,
marker Project/Goal-version conflicts (including Project/Goal metadata edits and
lifecycle ABA), release-token Record-version mismatches, untyped/unknown marker
errors, release CAS/executor-Lost fence failures, post-dispatch Session-binding
conflicts and unknown reversible Evaluating claims.

## Verification and completion

Pause both actual source-capture awaits after reservation and the adapter-start
await, observe from the same and a second Engine, verify Task/Workflow versions,
ContextVersion pointer and audit unchanged, then release and verify exactly one
adapter launch/Session. A fresh Engine must conservatively retain an orphaned
undispatched attempt. Eligible owner capture errors and only a typed owning-Task-row
marker CAS loss release the proven undispatched owned attempt and let the next step
progress. Marker Project/Goal/Record-version conflicts and untyped marker errors retain
the same reservation, including when owners are active; a release conflict/fence also
retains it. A coordinated StateOnly re-persist writes Task before Workflow Record:
after the post-refresh capture its marker conflict surfaces as an owning Task-row
error, but the changed Record version prevents release. Before refresh, the
refresh Record-version check rejects it first. Verify both actual release-token
consumers and attribute their different errors correctly. The marker
writes only its owning Task, so other Task IDs are unreachable there; verify exact
table/owning-ID classification directly as defense in depth, without claiming
consumer mutation credit for unreachable alternatives. Factual gate observation
can write only the Record, but requires an Evaluating attempt and cannot rewrite
this Running preparation claim.
EvidencePort CAS-loss behavior stays unchanged.
For its claim-CAS loss, assert the active index, history length, retries and
ContextVersion pointer unchanged, then verify the next step evaluates that same
attempt. A compiled mutant applying agent release to EvidencePort must fail.
Race two owners at reservation commit: the loser must leave the winner's claim
untouched and the winner must launch exactly once. Pause or cancel the owner in
each held pre-dispatch capture: the returning owner sends no native input and
retains the unchanged reservation; pause/resume does not silently retry. A
cancelled Task may then explicitly close its undispatched reservation with
TerminalRecovery. Resuming the suspended owner must send no native input or
write; Interrupted history and the terminal decision remain unchanged. A
definitive-decision publication CAS loss and a dispatched Session-binding loss
also remain reserved with concurrent metadata intact.
Compiled mutations restoring premature reset, making observer state changes or
removing owner-local release must fail the actual consumer regressions. Run Workflow/shared-state regressions and
Linux/macOS build/test/lint CI; record original failures separately. Requirements,
issue design and master design must match, with independent immutable review.

## Non-goals

No generic process-identity adoption, new recovery API, lease schema, scheduler
implementation, LLM orchestration, native agent changes or Git cleanup change.
This fix preserves truthful durable state while Issue 14 supplies explicit
recovery, including inactive-owner preparation failures and post-dispatch
Session-binding conflicts; it does not claim that recovery or the MVP is complete.
Existing explicit terminal-reservation recovery is preserved.
