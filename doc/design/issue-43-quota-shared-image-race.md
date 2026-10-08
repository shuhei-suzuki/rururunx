# Issue 43: cross-Project live-quota shared-image race (SC10 CI failure) — draft for review

Base: `86a3149`. This is a production change in `state/execution/native_phase/live_quota.rs` and `execution/native.rs`, so it is STRICT (shared persistence). Design review comes before code.

## 1. Facts (reproduced at `86a3149`, non-root, 6 concurrent SC10 runs)

- **Symptom.** In PR #80 Linux CI at `d82dc03`, and locally in 1 of 6 concurrent runs, one SC10 Task stays at `session_bound`. Its job is `finished: true`, `success_attention: "owned success settlement not observed"`, `owner_live: false`, `settled: false`.
- **Durable state of the stuck Task.**
  - Unit: `work: unknown`, `disposition: protocol_error`.
  - Session: `LOST` with `native_ref: null`.
  - The held Unknown non-success is the intended conservative outcome; the defect is that the session failed at all.
- **Error.** Captured with a temporary debug print, not committed. The Native actor's error was `registered quota complete plan images changed` (`live_quota.rs:149`). The actor classifies it as `NativeFailure::ProtocolFailure` → `Disposition::ProtocolError` (`execution/native.rs:1858-1888`).
- **Mechanism.**
  1. `quota_read` / `observe_quota` / `quota_wait` / `quota_resume` (`execution/native.rs:1484-1585`) first make a plan: `plan(...)` reads the own lease plus the shared `quota_pools` row and `quota_windows` rows for `(provider, 'unknown')` in one snapshot transaction.
  2. They then call `apply_phase_live_quota` in a separate IMMEDIATE transaction, whose `validate_tx` requires those images to be byte-identical.
  3. The pool row and windows are **shared by every Unit of the provider**, across Projects. SC10 runs two Codex Tasks in two Projects concurrently.
  4. One Unit's quota observation write (`INSERT … quota_windows`, `UPDATE quota_pools SET next_probe_at…`) landing between the other Unit's plan and its apply refuses the other Unit's apply.
  5. That refusal ends the other Unit's Native session.
- **Why it matters.** Cross-Project parallel execution is an MVP requirement (#81). A legitimate write by another Project's Task must not end a Task's Native session as Unknown.
- **Existing precedent.** `native_write` (`execution/native.rs:1071-1083`, BR) already allows one typed re-plan when the SAME operation's binding link advanced between plan and check.

## 2. HOW (proposed)

1. **A typed refusal for shared-image drift only.** In `NativeLiveQuotaPlan::validate_tx`:
   - If the own lease image differs, or the owner check fails, the error stays as today (fatal, no re-plan).
   - If only the shared pool and/or window images differ, return a private typed error `QuotaSharedImagesChanged` (`state` module, not constructible outside it). No row is written.
2. **Bounded re-plan.** A `quota_write` wrapper next to `native_write`, used by the four live-quota calls:
   - On `QuotaSharedImagesChanged`, it re-plans from fresh images and re-applies, at most 8 attempts.
   - Every attempt re-reads and re-validates the complete images exactly. The CAS, the lease, owner and epoch checks are not relaxed.
   - Exhaustion returns the last error (today's behaviour).
   - It composes with BR: `native_write(… quota_write(…))`, keeping BR's single re-plan for its own cause.
3. **No grant change.** No new caller can mint quota authority. The plan is still built only by `plan(...)` from the SAME Live actor and own lease, and Mutation contents are recomputed from the fresh images on each attempt.

## 3. Controls

- **C1 (deterministic race, new).** Two Codex Tasks in two Projects on the installed lane.
  - A `cfg(test)` pause between plan and apply holds Unit A. Unit B's observation write is then released.
  - Positive: A re-plans (counted `quota re-plan` = 1), its session is not Lost, and both close.
  - Negative mutant: no re-plan → A ends `protocol_error` / Unknown (today's failure).
- **C2.** A changed own lease still refuses without a re-plan (re-plan count 0, nothing written).
- **C3.** SC10 under the 6-way concurrent load used in §1: 0 failures in at least 5 rounds (30 runs), before and after.
- **Mutants.**
  - Treat a lease difference as shared drift: C2 fails.
  - Drop the bound: an exhaustion control with a perpetually changing pool fails the bound assertion.

## 4. Open question for review

The alternative is a single IMMEDIATE transaction for plan and apply. It would avoid the re-plan, but it moves `plan(...)`'s snapshot and InventoryBudget reads into the write transaction and holds the Store write lock longer. Recommended: the bounded typed re-plan, which matches BR.
