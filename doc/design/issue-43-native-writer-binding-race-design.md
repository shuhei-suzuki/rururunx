# Issue 43: Native owner writers and the concurrent normal binding (BR) HOW

## 1. Status, scope and pins

- **Status:** proposed STRICT HOW, revision R1. Nothing below is implemented. It is a prerequisite found while implementing the approved SC HOW (`a40355d`, §19 (g)): SC1 cannot reach an owned terminal.
- **Pins.** Paths are relative to `crates/rrx/src/`. Line references are at `9e01c7c`-based HEAD `2e07cfc` (branch `feature/issue-43-native-nonsuccess-controls`) unless prefixed. The composition reviewed in Issue #43 comment 6032847626 (`732a88c`) contains the same code paths.
- **Unchanged WHAT.** Managed binding requirements MB1–MB9 and MB-AC1…AC8. No guard is removed or widened; no pin is refreshed from rows; no Task.version binding.
- **MVP rule.** At most one active Task per Project (#43 / #81); nothing here depends on same-Project parallelism.

## 2. Verified defect (Linux, reproduced at `2e07cfc` with the SC1 control)

| # | Fact | Source |
| --- | --- | --- |
| F1 | Every Native owner writer plans its currency with `plan_owner_currency` → `plan_current_phase` (one read snapshot), then re-validates the SAME successor with `validate_current_tx` in a second snapshot, and again inside its write Immediate (`plan.owner.validate_tx`). | `state/execution/native_phase.rs:290–345` (`plan_native_owner` callers at `:619` dispatch receipt, `:680` dispatch admission, `:872`, `:1030`, `:1409`; `native_phase/live_quota.rs:158`; the terminal's ended-owner mode at `native_phase/terminal.rs:296`) |
| F2 | `validate_current_tx` requires the exact current Workflow and ledger endpoint (`validate_projection` "managed Workflow changed"; ledger count/head). | `state/managed_binding/successor.rs:330–380`, `snapshot.rs:398–404` |
| F3 | The normal binding commits from the start task right after `start_phase` returns (`bind_returned`), advancing the Workflow (v→v+1) and adding the operation's first `session_bound` link. The Core is already spawned and running (`execution/native/transport.rs:758`). | `runtime/phase_jobs.rs` start task, `bind_returned` `:1095` |
| F4 | The Core records a dispatch receipt for every effect frame, including the first `initialize`/bootstrap frame  (`Core::send_effect` `:1436` → `record_dispatch_observation` `:1553` → `plan_phase_dispatch_receipt`). | `execution/native.rs` `send_effect`, `record_dispatch_observation` |
| F5 | A Native writer error inside the Core's protocol future is fatal: `Core::run` maps it to `NativeFailure::ProtocolFailure`, the terminal capture then fails to persist, and the owner is revoked with no settlement. | `execution/native.rs` `Core::run` (≈`:1805–1890`) |

**Reproduction (both providers):** the dispatch receipt of an early effect is planned at Workflow v6 (unlinked); the binding commits v7 with its `session_bound` link; the receipt's second snapshot then refuses ("managed Workflow changed", actual v7, expected v6). The Core ends `ProtocolFailure` while the Unit stays `DispatchPending` with native effects open; the job is bound but no settlement ever exists. The success continuation (SC) is therefore unreachable, and so is every post-bind Native effect.

This is a race between two legitimate writers of the SAME operation. The binding is correct (it never writes Native rows); the Native writers are correct to refuse a stale plan; what is missing is the re-plan of the Native writer after the only legitimate concurrent ledger advance before its terminal.

## 3. Design (P)

### 3.1 Which concurrent advance is legitimate

Before the Native terminal, the only writer that advances the open operation's Workflow and ledger is the first `session_bound` link (normal or late). Claim, observation and closure require the settled terminal; RN non-success requires an unbound, ended start. The trigger caps `session_bound` at 1 per operation (`state/managed_binding/schema.rs:397`). An unlinked → singly-linked transition can therefore happen at most once per operation.

### 3.2 Re-plan once on that transition

1. `NativeOwnerPlan` exposes a nongrant `linked()` (whether its planned successor has any link).
2. **Inside planning:** `plan_owner_currency` keeps its two reads but, if the second snapshot's `validate_current_tx` fails, plans the successor once more; it retries exactly once, and only if the first plan was unlinked and the fresh successor has exactly one link whose kind is `session_bound` for the SAME operation. Otherwise the original error is returned.
3. **Write call sites:** each Native owner writer reached from the Core (dispatch admission, dispatch receipt, input acknowledgment and the live-quota writer) is wrapped by one helper, `Core::native_write(plan_fn, write_fn)`: plan, write; on a write `Err`, re-plan once and retry only under the same condition (the failed plan unlinked, the fresh plan linked by the sole `session_bound`). A writer whose Immediate already executed a statement is not retried (its outcome is uncertain and stays the existing error). Each writer's first check is its owner/currency validation, before any write, so a retried attempt never doubles a write.
4. **Terminal:** unchanged. Its existing protocol already handles one absent-confirmed replan (`persist_saved_terminal`: `confirmed_absent` → `replan_terminal_after_absence`), which covers a binding commit between the terminal plan and its Immediate.
5. **Never:** no check is skipped or relaxed; the retried attempt re-validates the exact current successor, owner, pair, Driver and Unit. No retry for any other change (parent drift, Unit/Session drift, a second link kind), no loop, no wait, no timeout inference.

### 3.3 Cost

At most one extra `plan_current_phase` read and one extra Immediate per writer call, and only on the single binding transition per operation.

## 4. Controls and mutants (P; none executed)

- **BR1 (positive, genuine):** SC1's lane (claude and codex, commit mode). A cfg(test) timing seam holds the Core's first dispatch receipt between its plan and its write until the normal `session_bound` link is committed (Task-scoped; parks and resumes only). Assertions: the writer re-planned exactly once (cfg(test) counter), the receipt is recorded, the Core reaches its owned terminal, and SC1's later assertions hold.
- **BR2 (negative):** the same seam, but the concurrent change is a parent Task edit through an existing legitimate writer instead of the binding: no retry, the original refusal stands (Held as today).
- **Mutants:** remove the planning retry (BR1: "managed Workflow changed", Core `ProtocolFailure`); remove the write retry (BR1: receipt refusal, Core `ProtocolFailure`); widen the condition to any link change (BR2: a retry occurs).

## 5. Impact

| Changed | Consumers | Handling |
| --- | --- | --- |
| `plan_owner_currency` | every Native owner writer | one guarded re-plan; same errors otherwise |
| Core writer call sites | dispatch admission/receipt, input ack, live quota | one helper; behavior identical when no binding commits concurrently |
| Tests | BR1, BR2 | new; existing Native and RN controls unchanged |

Not affected: binding (normal/late), RN, preparation, SC ports, schema, triggers, permits.

## 6. Review

Requested from Sol 6.1 high through the `codex-ready` flow on Issue #43. No source change for BR lands before that review closes without Critical/High/Medium findings.
