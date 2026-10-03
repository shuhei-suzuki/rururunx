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

1. Ordinary observation cannot infer owner absence from missing Session or
   `dispatch_started=false`. Preserve the preparation reservation and report
   Waiting with an explicit preparing/recovery explanation.
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
   snapshot alone. EvidencePort claim recovery semantics stay separate.

## Verification and completion

Pause the actual source-capture consumer after phase reservation, observe from a
second Engine, verify snapshots/audit unchanged, then release and verify exactly
one adapter launch. A fresh Engine must conservatively retain an orphaned
undispatched attempt. A compiled mutation restoring the premature reset must
fail the interleaving regression. Run Workflow/shared-state regressions and
Linux/macOS build/test/lint CI; record original failures separately. Requirements,
issue design and master design must match, with independent immutable review.

## Non-goals

No generic process-identity adoption, new recovery API, lease schema, scheduler
implementation, LLM orchestration, native agent changes or Git cleanup change.
This fix preserves truthful durable state while Issue 14 supplies explicit
recovery; it does not claim that orphan recovery or the MVP is complete.
