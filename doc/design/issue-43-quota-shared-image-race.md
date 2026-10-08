# Issue 43: cross-Project live-quota shared-image race (SC10 CI failure) — approved (Sol 6054910774), implemented `9dc2648`

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
   - On `QuotaSharedImagesChanged`, it re-plans from fresh images and re-applies, at most 8 attempts per call.
   - The bound is per BR leg (B-SOL-L1). BR re-invokes the inner call once, so one quota callback makes at most 16 attempts in total.
   - Every attempt re-reads and re-validates the complete images exactly. The CAS, the lease, owner and epoch checks are not relaxed.
   - Exhaustion returns the last error (today's behaviour).
   - It composes with BR: `native_write(… quota_write(…))`, keeping BR's single re-plan for its own cause.
3. **No grant change.** No new caller can mint quota authority. The plan is still built only by `plan(...)` from the SAME Live actor and own lease, and Mutation contents are recomputed from the fresh images on each attempt.

## 3. Controls

- **C1 (deterministic race, new).** Two Codex Tasks in two Projects on the installed lane.
  - A `cfg(test)` pause between plan and apply holds Unit A. Unit B's observation write is then released.
  - Positive: A re-plans (counted `quota re-plan` = 1), its session is not Lost, and both close.
  - The pause (`QUOTA_APPLY`) is a one-shot test hold between plan and apply. It parks and resumes only; no store lock is held across it.
  - Negative mutant: no re-plan → A ends `protocol_error` / Unknown (today's failure).
- **C2.** A changed own lease still refuses without a re-plan (re-plan count 0, nothing written).
- **C3.** SC10 under the 6-way concurrent load used in §1: 0 failures in at least 5 rounds (30 runs), before and after.
- **Mutants.**
  - Treat a lease difference as shared drift: C2 fails.
  - Drop the bound: an exhaustion control with a perpetually changing pool fails the bound assertion.

## 4. Open question for review

The alternative is a single IMMEDIATE transaction for plan and apply. It would avoid the re-plan, but it moves `plan(...)`'s snapshot and InventoryBudget reads into the write transaction and holds the Store write lock longer. Recommended: the bounded typed re-plan, which matches BR.

## 5. Review outcome and implementation

| Item | Verdict (6054910774) | Implemented |
| --- | --- | --- |
| HOW §2 (typed refusal, bounded re-plan, no grant change) | APPROVE LIMITED | `9dc2648` |
| B-SOL-L1 (bound under BR) | Low, optional | §2 wording above: 8 per BR leg, at most 16 in total |
| Alternative (single IMMEDIATE transaction) | Not required | Not taken |

Implementation notes:
- **Classification order.** `NativeLiveQuotaPlan::validate_tx` keeps the owner, custody and epoch checks (`owner.validate_tx`) and the own-lease check first. `read` still fails untyped on a malformed pool, window or lease. Only a pool/window inequality after those checks returns `QuotaSharedImagesChanged`, which is private to `live_quota.rs`. `InventoryWorkLimit` wins in `InventoryBudget::finish`, so it is never re-planned.
- **Bounded loop.** `replan_shared_quota` lives in the same module, so the typed error stays unconstructible outside it. Each attempt is the caller's complete closure, so the plan, its derived result (read result, observation decision and backoff, Unit update, resume decision) and the apply are all fresh. Nothing on the wire is resent.
- **Call sites.** `Core::quota_write` (`execution/native.rs`) composes it inside `native_write` (BR) for `quota_read`, `observe_quota`, `quota_wait` and `quota_resume`. `quota_apply` holds no lock across the test pause.

Controls (non-root fmtest, umask 022, subreaper, toolchain 1.91.1):

| Control | Result |
| --- | --- |
| C1 `sc10_cross_project_quota_drift_replans_from_fresh_images` | pass; A `quota re-plan` = 1; both close with four links |
| C2 `sc10_own_lease_change_refuses_without_replan` | pass; re-plan 0; shared images byte-identical; links `[session_bound]` |
| Bound `shared_quota_replan_is_bounded_and_typed_only` | pass; 8 attempts, 7 re-plans; other refusals 1 attempt |
| Mutant Q1: no re-plan | C1 FAIL "C1: A did not re-plan"; SC10 FAIL unloaded, with the CI symptom (`session_bound` only, `owned success settlement not observed`); bound FAIL |
| Mutant Q2: lease difference as shared drift | C2 FAIL "C2: no re-plan" |
| Mutant Q3: unbounded | bound FAIL "exhaustion returns the refusal" |
| C3: SC10, 6-way concurrent, 5 rounds | see the #43 checkpoint |
