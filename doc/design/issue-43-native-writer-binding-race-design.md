# Issue 43: Native owner writers and the concurrent normal binding (BR) HOW

## 1. Status, scope and pins

- **Status:** proposed STRICT HOW, revision R2 (supersedes R1 `8f66fa2`; §3.2 and §4 changed after an uncommitted prototype, §3.4). Nothing below is implemented on the branch. It is a prerequisite found while implementing the approved SC HOW (`a40355d`, §19 (g)): SC1 cannot reach an owned terminal.
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

### 3.2 Re-plan once on that transition (R2: one typed mechanism)

R1 proposed two mechanisms (a planning retry and a write retry) and listed four Core writers. The prototype (§3.4) showed the list was incomplete and that one typed refusal covers planning and writing alike.

1. **Typed refusal.** `UnlinkedOwnerStale` (nongrant marker, `state/execution/native_phase.rs`) is attached as error context only where an owner plan's exact currency is first checked, and only when that plan has no Workflow link:
   - the second snapshot of `plan_owner_currency` (`validate_current_tx`), and
   - `NativeOwnerPlan::validate_tx` (`validate_current_tx`), the first check of every owner writer's Immediate and of each later planning snapshot that revalidates the owner (for example `plan_phase_dispatch`'s invocation snapshot).

   It is never attached by `validate_terminal_tx`, so the terminal path stays unchanged.
2. **Predicate.** `binding_advanced(runtime, phase, error)` holds only when the error carries the marker AND a fresh `plan_current_phase` of the SAME marker has exactly one link, the first `session_bound` (`CurrentWorkflowSuccessor::is_sole_binding`, nongrant).
3. **One helper.** `native_write(owner, phase, attempt)` in `execution/native.rs` runs `attempt` (plan + write) once more only when the predicate holds; otherwise it returns the original error. It wraps every owner writer reached from the Core:
   - owner validation (`actual_native_authority` → `validate_phase_owner`),
   - Session projection (`project_phase_session`),
   - dispatch admission, dispatch receipt, input acknowledgment,
   - the four live-quota writers (read, observation, wait, resume).
4. **No doubled write.** A marked refusal is raised before `commit()`, so its Immediate rolls back with nothing applied. A commit error never carries the marker and is not retried; it keeps its existing uncertain handling.
5. **Terminal.** Unchanged (its existing absent-confirmed replan applies).
6. **Never.** No check is skipped or relaxed: the retry re-plans and re-validates the exact current successor, owner, pair, Driver and Unit. No retry for a linked plan, for any other change (parent drift, Unit/Session drift, a second link kind), and no loop, wait or timeout inference. Each call retries at most once; the transition itself happens at most once per operation (§3.1).

### 3.3 Cost

At most one extra `plan_current_phase` read, and one extra plan and Immediate per writer call, and only on the single binding transition per operation.

### 3.4 Prototype evidence (uncommitted, Linux)

- **Branch base:** prototype worktree at `8c9d4e5` plus §3.2 only.
- **Result:** SC1 claude and codex pass (2 passed): `session_bound`, then `gate_claim`, then `gate_observed`, then success `phase_closed`.
- **Full lib run** (prototype on `067b9fa`): 368 passed, 284 failed, 20 ignored. Every failure is in the `2e07cfc` baseline set (trusted-ingress fixtures); there are no new failures. SC1 ×2, `rn_p1`, `rn_h1` and DC1–DC4 pass under workspace load.
- **R1 list incomplete:** with only the R1 writers wrapped, codex still ended `ProtocolFailure` from a marked refusal in owner validation and Session projection.
- **Defects found and fixed separately in `8c9d4e5`** (implementation defects of approved designs, not BR):
  - the terminal Session index compared kind `SESSION` with the stored `session`;
  - `gate_observed` needed the Driver gate writer's virtual-observer predecessor;
  - the success closure's typed-link audit sequence collided after W1–W4 appends.

## 4. Controls and mutants (P; none executed)

- **BR1 (positive, genuine):**
  - Lane: SC1 (claude and codex, commit mode).
  - Seam: a cfg(test), Task-scoped timing seam parks the Core's first owner writer after its plan and before its Immediate, until the normal `session_bound` link commits. It only parks and resumes.
  - Assertions: exactly one marked refusal and one retry (cfg(test) counter); the writer's row is recorded; the Core reaches its owned terminal; SC1's later assertions hold.
- **BR2 (negative):**
  - Same seam, but the concurrent change is a parent Task edit through an existing legitimate writer, not the binding.
  - Assertions: the predicate is false, there is no retry, and the original refusal stands (Held as today).
- **BR3 (negative):**
  - A plan made after the binding (linked) and refused for another change.
  - Assertion: no marker and no retry.
- **Mutants:**
  - Drop the marker at `validate_tx`. BR1 fails on the writer refusal and Core `ProtocolFailure`.
  - Drop the helper on one listed writer (owner validation). BR1 fails.
  - Widen the predicate to any link change. BR2 shows a retry.
  - Attach the marker for linked plans. BR3 shows a retry.

## 5. Impact

| Changed | Consumers | Handling |
| --- | --- | --- |
| `plan_owner_currency`, `NativeOwnerPlan::validate_tx` | every Native owner writer | marker context on an unlinked plan's currency refusal; message and control flow otherwise unchanged |
| `CurrentWorkflowSuccessor::is_sole_binding` | BR predicate | nongrant accessor |
| Core writer call sites (§3.2 item 3) | owner validation, Session projection, dispatch admission/receipt, input ack, live quota | one helper; behavior identical when no binding commits concurrently |
| Tests | BR1, BR2, BR3 | new; existing Native and RN controls unchanged |

Not affected: binding (normal/late), RN, preparation, SC ports, schema, triggers, permits.

## 6. Review

R1 was requested from Sol 6.1 high through the `codex-ready` flow on Issue #43; R2 supersedes it in the same request. No source change for BR lands before that review closes without Critical/High/Medium findings.
