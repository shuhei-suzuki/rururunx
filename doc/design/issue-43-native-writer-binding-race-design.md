# Issue 43: Native owner writers and the concurrent normal binding (BR) HOW

## 1. Status, scope and pins

- **Status:** proposed STRICT HOW, revision R3. It answers the Sol review of R2 (Issue #43 comment 6038649745: BR-M01–M03, BR-L01–L02). R2 superseded R1 `8f66fa2`. Nothing below is implemented on the branch. It is a prerequisite found while implementing the approved SC HOW (`a40355d`, §19 (g)): SC1 cannot reach an owned terminal.
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
| F5 | A Native writer error inside the Core's protocol future ends it. The category depends on the writer: owner validation carries `AuthorityUnavailable` context (`execution/native.rs:1362–1369`), which `Core::run` prefers (`:1815–1836`); other writer errors (e.g. a dispatch receipt) map to `ProtocolFailure`. Separately, at the pin the terminal Session index compared `SESSION` with the stored `session` (`2e07cfc` `terminal.rs:358–364`, fixed in `8c9d4e5`), so the owned terminal could not persist either; an Unknown protocol terminal can instead be saved as HistoricalDraft (HEAD `terminal.rs:535–580`). These are three distinct facts: the writer category, terminal persistence, and the absent `OwnedPhaseSettlement`. | `execution/native.rs` `Core::run`; `native_phase/terminal.rs` |

**Reproduction (both providers, historical at `2e07cfc`):** an owner writer planned at Workflow v6 (unlinked) is refused after the binding commits v7 with its `session_bound` link ("managed Workflow changed"). The Core ends (`AuthorityUnavailable` for owner validation, `ProtocolFailure` for e.g. a receipt), the Unit stays `DispatchPending` with native effects open, and no settlement exists. Independently, the terminal index defect above blocked terminal persistence at that pin. The success continuation (SC) is therefore unreachable, and so is every post-bind Native effect.

This is a race between two legitimate writers of the SAME operation. The binding is correct (it never writes Native rows); the Native writers are correct to refuse a stale plan; what is missing is the re-plan of the Native writer after the only legitimate concurrent ledger advance before its terminal.

## 3. Design (P)

### 3.1 Which concurrent advance is legitimate

Before the Native terminal, the only writer that advances the open operation's Workflow and ledger is the first `session_bound` link (normal or late). Claim, observation and closure require the settled terminal; RN non-success requires an unbound, ended start. The trigger caps `session_bound` at 1 per operation (`state/managed_binding/schema.rs:397`). An unlinked → singly-linked transition can therefore happen at most once per operation.

### 3.2 Re-plan once on that transition (R2: one typed mechanism)

R1 proposed two mechanisms (a planning retry and a write retry) and listed four Core writers. The prototype (§3.4) showed the list was incomplete and that one typed refusal covers planning and writing alike.

1. **Typed refusal.** `UnlinkedOwnerStale { unit }` (nongrant marker, `state/execution/native_phase.rs`) is attached as error context only where an owner plan's exact currency is first checked, and only when that plan has no Workflow link. It carries the failed plan's exact Unit encoding (`unit_raw`), a nongrant comparison value. It is attached at:
   - the second snapshot of `plan_owner_currency` (`validate_current_tx`), **only when `terminal_ending` is false** (BR-L02), and
   - `NativeOwnerPlan::validate_tx` (`validate_current_tx`), the first check of every owner writer's Immediate and of each later planning snapshot that revalidates the owner (for example `plan_phase_dispatch`'s invocation snapshot).

   It is never attached by `validate_terminal_tx` or by terminal planning, so the terminal path stays unchanged. Every attachment point precedes the first DB mutation of its transaction (BEGIN and SELECT only), which is the safety basis of a retry (§3.2 item 4).
2. **Predicate (BR-M01).** `binding_advanced(runtime, phase, error)` holds only when all of these hold:
   - the error carries the marker;
   - a fresh `plan_current_phase` of the SAME marker has exactly one link, the first `session_bound` (`CurrentWorkflowSuccessor::is_sole_binding`, nongrant);
   - the fresh successor's exact Unit encoding equals the marker's `unit`.

   The failed plan was unlinked, so its Workflow was the original `workflow_after` (enforced by the unlinked-bytes check in `plan_current_phase`). The fresh ledger chains from that same original endpoint. The only admitted difference is therefore the first binding link and its Workflow successor. Any Unit change, alone or together with the binding, makes the predicate false.

3. **One helper.** `native_write(owner, phase, attempt)` in `execution/native.rs` runs `attempt` (plan + write) once more only when the predicate holds; otherwise it returns the original error. It wraps every owner writer reached from the Core:
   - owner validation (`actual_native_authority` → `validate_phase_owner`),
   - Session projection (`project_phase_session`),
   - dispatch admission, dispatch receipt, input acknowledgment,
   - the four live-quota writers (read, observation, wait, resume).
4. **No doubled write.** Every marker point precedes the first DB mutation of its transaction, and the transaction is dropped without commit, so nothing is applied. Being "before commit" alone is not the basis. A commit error or an after-write error never carries the marker and is not retried; it keeps its existing uncertain handling. The retry covers plan plus Store write only. It never resends a wire frame and never re-runs post-commit steps (`retain_dispatch`, owner projection).
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

## 4. Controls and mutants (P; none executed on the branch)

**Seams.** These are cfg(test), Task-scoped and named. They park and resume only, outside every Store and job lock, and grant nothing.
- `NORMAL_WRITE` parks the start task between its normal binding plan and write (exists).
- `OWNER_IMMEDIATE` parks the Core's first owner-validation consumer (`actual_native_authority`, the first consumer for both providers, `execution/native.rs:2061,2211`) after its plan and before its Immediate.
- `OWNER_PLANNING` parks `plan_owner_currency` between `plan_current_phase` and its second snapshot.
- Two cfg(test) counters: marker attachments per site (planning or Immediate), and helper retries.

**BR1 (positive, genuine; BR-M02).**
- Lane: SC1 (claude and codex, commit mode).
- Fixed order:
  1. Arm `NORMAL_WRITE` and `OWNER_IMMEDIATE`.
  2. The Core reaches `OWNER_IMMEDIATE`. Its plan is unlinked, because the binding is parked before its write.
  3. Release `NORMAL_WRITE` and wait for the `session_bound` link.
  4. Release the Core.
- Assertions on the owner-validation lane: exactly one marker attachment at the Immediate site and exactly one retry; the authority is obtained (the Core continues); the Core reaches its owned terminal; SC1's later assertions hold.

**BR1-P (positive, genuine).**
- Same as BR1, with `OWNER_PLANNING` instead of `OWNER_IMMEDIATE`.
- Assertions: the marker attachment is at the planning site; there is one retry; the same terminal assertions hold.

**BR2 (negative; BR-M03).**
- Stimulus: while the Core is parked as in BR1, the Goal is paused through the existing lifecycle writer (`SetGoalLifecycle` Pause; `Store::put_task` refuses accepted Task edits, `state/mod.rs:450–489`). The binding stays parked.
- Assertions: the predicate is false (the fresh planner refuses on the original projection); zero retries; zero links; the writer's refusal stands.
- This control asserts parent-drift no-retry/zero-write only. It is not claimed to observe a widened link predicate.

**BR3 (negative).**
- Stimulus: a plan made after the binding (linked), then refused for the BR2 stimulus.
- Assertions: zero marker attachments (counter), zero retries.

**Mutants (compiled):**
- (a) Drop the marker at `validate_tx`. BR1 fails: no owned terminal, Core category `AuthorityUnavailable`.
- (b) Drop the helper on owner validation. BR1 fails, as in (a).
- (c) Drop the marker at the planning snapshot. BR1-P fails.
- (d) Attach the marker for linked plans. BR3's marker counter is non-zero.
- (e) Drop the Unit-equality condition. Defense-only: no genuine concurrent Unit writer was identified while the registered Core is live on this lane (its own writers run serially in one protocol future, and the binding writes no Unit). It is not claimed killed by a genuine control.
- (f) Widen sole-binding to any link change. Also defense-only (BR-M03): no other link kind is reachable before the owned terminal (§3.1), and a parent drift is refused by the fresh planner first. It is not claimed killed.

## 5. Impact

| Changed | Consumers | Handling |
| --- | --- | --- |
| `plan_owner_currency` (non-terminal mode only), `NativeOwnerPlan::validate_tx` | every Native owner writer | marker context (with the plan's Unit encoding) on an unlinked plan's currency refusal; message and control flow otherwise unchanged; terminal planning never marked |
| `CurrentWorkflowSuccessor::is_sole_binding` | BR predicate | nongrant accessor |
| Core writer call sites (§3.2 item 3) | owner validation, Session projection, dispatch admission/receipt, input ack, live quota | one helper; behavior identical when no binding commits concurrently |
| Tests | BR1, BR1-P, BR2, BR3 and three cfg(test) seams/counters | new; existing Native and RN controls unchanged |

Not affected: binding (normal/late), RN, preparation, SC ports, schema, triggers, permits.

## 6. Review

R1 was requested from Sol 6.1 high through the `codex-ready` flow on Issue #43; R2 superseded it; R3 answers the R2 review (comment 6038649745). No source change for BR lands before that review closes without Critical/High/Medium findings.
