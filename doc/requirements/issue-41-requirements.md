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
   preserves it. Explicit recovery must establish preparation-owner absence and
   no dispatch before releasing it; that integration remains Issue 14.
5. Dispatched or unknown native outcomes remain reserved. No automatic replay,
   fabricated process death, timeout relaxation or permission change.
6. Existing final-claim CAS losses retain changed user metadata. A failed owner
   is distinguishable only through trusted recovery evidence, not a polling
   snapshot alone.
7. An owner returning an error before the dispatch marker successfully commits
   has first-hand no-dispatch proof. It must re-read and CAS-release only its own
   unchanged Running attempt (same generation/index/context/agent, no Session,
   no dispatch marker), recording Failed and a factual RetryEvent while keeping
   concurrent user metadata. Conflicting attempts, live/Lost Sessions or failed
   release remain reserved. Never release after the marker may have committed.
   Process crash and dropped futures remain explicit-recovery cases.
8. Running EvidencePort attempts retain the existing claim-CAS evaluation path;
   its state mutation/port invocation is not covered by requirements 1–4. Unknown
   evaluating claims and irreversible outcomes retain their existing fences.
9. Both no-Session observer reasons must state preparation/launch may still be
   active and must not assert interruption. Status/Scheduler use durable attempt
   actor/state/Session/dispatch fields to distinguish these waits without parsing
   human-facing reason text. A genuine orphan retains its phase/reservation;
   cancellation, Project removal and conflicting execution remain fenced until
   trusted recovery resolves it. Process/owner restart recovery belongs to #14;
   external GitHub/irreversible port outcome reconciliation belongs to #13.

## Verification and completion

Pause both actual source-capture awaits after reservation and the adapter-start
await, observe from the same and a second Engine, verify Task/Workflow versions,
ContextVersion pointer and audit unchanged, then release and verify exactly one
adapter launch/Session. A fresh Engine must conservatively retain an orphaned
undispatched attempt. Owner capture errors and final-claim CAS loss release only
the proven undispatched owned attempt and let the next step progress; an injected
release conflict retains Waiting. EvidencePort CAS-loss behavior stays unchanged.
Compiled mutations restoring premature reset, making observer state changes or
removing owner-local release must fail the actual consumer regressions. Run Workflow/shared-state regressions and
Linux/macOS build/test/lint CI; record original failures separately. Requirements,
issue design and master design must match, with independent immutable review.

## Non-goals

No generic process-identity adoption, new recovery API, lease schema, scheduler
implementation, LLM orchestration, native agent changes or Git cleanup change.
This fix preserves truthful durable state while Issue 14 supplies explicit
recovery; it does not claim that orphan recovery or the MVP is complete.
