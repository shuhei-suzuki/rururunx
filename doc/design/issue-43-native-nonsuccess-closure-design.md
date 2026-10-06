# Issue 43: Native genuine non-success phase closure (RN-1) authority/contract HOW

## 1. Status, fixed source and categories

1. This is a proposed authority/contract HOW supplement for prerequisite RN-1. RN-1 is already required by:
   - the approved [prepared-producer HOW](issue-43-native-prepared-producer-design.md) §§5 item 3, 7.4 item 2, 11.2 and 15;
   - the approved [managed binding design](issue-43-managed-binding-design.md) §§2.4, 3.1, 5.2, 5.3 (observation table), 6 and 7 (`phase_closed`).

   It refines, and does not replace, those documents and the [preparation HOW](issue-43-native-preparation-integration-design.md), [transport HOW](issue-43-native-transport-integration-design.md) and [marker/dispatch HOW](issue-43-original-source-marker-dispatch-design.md). It is not a requirements change. It does not update master design and claims nothing is implemented. Prepared-HOW corrections B1 and B2 stay closed and are not reopened.
2. **Exact source.** `7be25fec1c7c2ce7ed1daee5bdf347818c9e31df`, clean. Paths are relative to `crates/rrx/src/`; every line reference is to this commit.
3. Categories are kept separate: **A** (actual at the source), **P** (proposed here, not implemented), **V** (verified only by reading this source). No build, test, lint, Agent, fixture or OS run was executed for this HOW.
4. **Inputs.** The Root reconnaissance `rn1-dependency-recon.md` (recorded SHA-256 `affa1bd74d98f37aedda4f597b275a1865843b2e5af5c5d49674bb7ad9dc8c8e`) was outside the directories readable by the authoring session. It was not read or hashed. Every fact in §2 was re-derived from source. The independent review should diff §2 against that report; a disagreement is a finding against this document.
5. **Evidence state (reported by Root, not re-run).** At `7be25fec`:
   - main regression: 334 outer passed, 438 failed, 32 ignored;
   - scoped primitives plus ingress: 50 PASS; separate endpoint prerequisite: 1 PASS;
   - fmt/check/build PASS; strict Clippy RED.

   No Native Agent, N1/N4, authentication, hooks, quota recovery, both-OS, install or dogfood qualification exists. There is no release or merge approval. Official-Agent/four-Task/both-OS remains an explicit MERGE gate (marker HOW lines 160–161, 335–336). Full MVP, PR-1 to PR-4, continuous DAG, four Tasks, both OS and immutable artifacts stay in scope, unchanged.
6. **Not decided here.** The following are not decided, fixed or widened:
   - Root SourceStop/Cancel author and permission decisions;
   - the R2 shared Git environment author exception;
   - the G3 Root `request_stop` caller;
   - G5 composition and its pending HOW correction;
   - transport closure or settlement;
   - successful capture, gate or artifact;
   - TerminalRecovery and `terminal_decision`;
   - across-epoch restore;
   - explicit protected retry (RN-R, §17).

   rrx is not a security sandbox. Native Agents and hooks keep the user's host authority. No process-death claim is made or needed.

### 1.1 One-line acceptance condition (P)

After the SAME selected Native start has ended, the Root's SAME retained PhaseJobs job may close the operation. Its SAME custody must hold a known nongrant preparation closure of the SAME issued `PreparedPhaseNoCurrentDispatch`, with no transport ever installed. The job then obtains one `NativeNoDispatchClosureProof` from that custody and commits ONE exact typed Immediate that:
- retires the complete open Unit image;
- moves the complete 32-column operation image `phase_open` 1→0;
- moves the complete Workflow Record with only the active attempt Running→Failed (awaiting explicit retry);
- inserts the unique `phase_closed` link.

All of this happens under unchanged original P/G/T/Workflow/Context/marker/owner/Source/Driver/epoch currency and an empty ledger. An uncertain commit is resolved only by exact whole post- or preimages of the SAME plan. Only the resulting typed acknowledgment releases the marked PhaseSupervisor slot and the PhaseJobs entry. Every other case stays Held.

### 1.2 Unmarked rollback versus marked no-current-dispatch (V)

| | Truly pre-marker Unmarked rollback | MARKED pre-transport no-current-dispatch |
|---|---|---|
| Durable facts | No marker committed; proved by `may_attempt_unpublished` (`runtime/phase_supervisor.rs:134–141`) | OriginalMarker committed; operation `phase_open=1`; owner, input and readiness rows exist |
| Slot | `PublicationState::Unmarked`, `accepted_source=false` | `Publishing`, `launch_handed_off=true`, accepted Source |
| Existing removal | `remove_unmarked` (`:582–618`), `close_unmarked` (`:703–736`), `remove_unstarted` (`runtime/phase_jobs.rs:315–348`, fresh and `Reserved` only) | All three refuse: "publishing operation needs proven rollback or genuine closure" (`:595–603`); "started job cannot roll back" (`phase_jobs.rs:333–342`) |
| Closure | None needed | RN-1 only (this HOW) |

RN-1 never calls, widens or reuses `remove_unmarked`, `remove_unstarted`, `close_unmarked` or `restore_unpublished`. They are unchanged.

## 2. Revalidated source map at 7be25fec (V)

| Item | Actual fact |
|---|---|
| Custody slots | `execution/native/preparation.rs:16–45`: `no_dispatch`, `transport`, `closure` (`NativeQuotaClosurePlan`), `closed` (`NativePreparationClosureCommit`). `abandon` (`:455–474`) only revokes and notifies; it performs no closure, Store write or Unit change |
| Root export | Root holds only `Arc<NativePreparationCustody>` (`runtime/phase_jobs.rs:33–42`). No custody method returns `closed` or `no_dispatch` outside `execution::native`; `closure_original` is `pub(super)` (`preparation.rs:88–110`) |
| No-dispatch value | `PreparedPhaseNoCurrentDispatch` (`execution/native/prepared.rs:4–22`): private `Weak` custody, SAME completion, issued Initial lineage. Its sole issuer is `issue_no_current_dispatch` (`:186–212`), inline after S4b |
| Preparation closure | `close_prepared_on_revocation` (`prepared.rs:24–101`) returns `Result<()>`. Its callers are the parked and backoff revocation arms (`:173,179`), which then bail "Root typed non-success closure unavailable", and `reconcile_known_commit` (`preparation.rs:665–671`). `reconcile_known_commit` returns `Ok(())` early when `closed` is set (`:656–658`) and has **no caller** |
| Closure plan/commit | `state/execution/native_phase/quota.rs:29–52,61–72,1228–1395` retain the SAME actor, no-dispatch, lineage, `LatestUnitImage` and exact pool/waiter/lease images, plus readiness `closed`, `start_ended=1`. `close_phase_quota` writes pool/waiter/lease and readiness under one readiness permit. Confirm accepts an exact postimage, or an exact preimage (rolled back); anything else is Held. **The Unit is never written** |
| Transport invariant | `retain_transport` (`preparation.rs:51–72`) does **not** clear `no_dispatch` (prepared HOW §5 item 3 says it does). `closure_original` does not check that `transport` is empty |
| Start path | `execution/native.rs:208–215` always bails after `begin_phase_preparation` returns Prepared ("transport composition unavailable"). `start_prepared_transport` (`execution/native/transport.rs:498`) has no caller (G5) |
| Root jobs | `MAX_JOBS=128` (`phase_jobs.rs:19`). On start `Err`, the observation becomes `Failed`, the outcome is retained, then `preparation.abandon()` runs (`:276–292`). `RunningJob` drop gives `Uncertain` plus abandon (`:385–404`). Entries are never removed after start |
| Root slots | A marked slot keeps its `PreparationGuard`. `ensure_accepted_shutdown_complete` (`phase_supervisor.rs:746–756`) and `ensure_shutdown_complete` (`phase_jobs.rs:350–360`) fail while any marked slot or job remains. Dropping an accepted guard is a no-op (`execution/owner.rs:102–112`) |
| Service loop | `runtime/service.rs:62–99` runs synchronous sweeps (`reconcile_pending`, Driver observation) with a capped 100 ms–5 s backoff |
| Operation row | 32 columns (`state/managed_binding/schema.sql:15–49`; `permits.rs:37–70`). The core trigger allows only `version=OLD+1` and non-increasing `phase_open` (`schema.sql:148`). The body carries `"phase_open":true,"version":1` (`marker_rows.rs:149–159`). `validate_open_tx` requires the original insert image (`marker_rows.rs:97–108`) |
| Ledger | `phase_closed` is declared (`schema.rs:30–38`) with no producer. The chain trigger (`schema.rs:398–414`) enforces: ≤4096 bytes; required header keys; at most 1 `phase_closed` per operation; `after=before+1`; `after−marker ∈ [1,256]` and `= 1+links`; `phase_closed` iff `phase_open=0`; predecessor rule; Workflow `version=after`. Private kinds need an exact audit permit (`:366`) |
| Record guard | Protected Workflow writes need an exact `records` permit and `version+1` (`schema.rs:317–336`). Task and Context are not permit tables (`permits.rs:23–180`) |
| Reservation | Each `phase_open=1` operation charges `1 MiB − spent` (`publication.rs:365–372`; `LINK_RESERVE_BYTES`, `marker_rows.rs:13`). Closing releases it, derived |
| Currency | `plan_current_phase` / `validate_current_tx` (`successor.rs:206–377`) require the open marker rows. The projection covers runtime instance/epoch, P/G/T versions and bodies, Workflow, locks and Context (`snapshot.rs:349–374`). `validate_driver_live_tx` (`publication.rs:201–211`) checks the Driver is live. This validator suits the OPEN preimage, not a CLOSED postimage |
| Unit flags | `managed_attempt_retired` = both flags false (`state/execution.rs:357–376`). Its consumers: `state/mod.rs:935,946`, `Engine::retry` (`workflow.rs:3104–3157`), cleanup worker (`state/execution/cleanup.rs:82`, which also needs a job row). Flag writers: `fence_task_tx` (`execution.rs:323–356`) is called only for a terminal Task (`state/mod.rs:1791–1793`) or a Goal pause/cancel (`state/runtime/goals.rs:440–446`), always with parent drift; `retire_execution_as` (`execution.rs:1033–1069`, generic authority); terminal after registration. **No producer exists for pre-Session no-dispatch** |
| Workflow | `AttemptState` (`workflow.rs:346–353`). `validate_transition` (`:3388`) allows Running→Failed and a Failed active attempt (`:3578–3615,3766–3779`). `poll` returns `Failed` read-only for a non-Running attempt (`:1940–1945`). `Engine::fail` writes Task `WaitingHuman` and blockers (`:2571–2581`) |
| Binder template | `bind_managed_phase` (`binding.rs:382–465`): Workflow CAS plus audit INSERT under two exact permits; audit `sequence` is allocated inside the transaction |
| Negative facts | `no_registration` (`state/execution/native_phase.rs:1173–1179`) covers Session record, `session_units`, `native_invocations` and admissions. `registration_unit` (`:1180–`) checks the open Unit shape. `LatestUnitImage` (`native_phase/version/closure.rs:11–160`) gives an exact 13-column Unit CAS |

## 3. Covered and uncovered cases (P)

### 3.1 The covered case

RN-1 closes an operation only when all of the following hold:
1. **Start ended.** The SAME job's start is over: `JobState.outcome` is `Some(Err(_))`, or the `RunningJob` drop recorded `Uncertain` with outcome `None` and the Entry's `JoinHandle` has finished.
2. **Actor revoked.** The custody is abandoned and its actor is revoked by an already existing producer: the PhaseJobs post-outcome abandon (`phase_jobs.rs:288–292`), the `RunningJob` drop (`:400–402`), or a later genuine revocation (G3). RN-1 never revokes, stops or abandons anything itself.
3. **Custody facts.**
   - `no_dispatch` is set and `transport` is empty.
   - `closed` is a known commit of the custody's SAME retained closure plan.
   - That plan's actor and no-dispatch are pointer-equal to the custody's.
4. **Store currency** per §7.2.

By reading, the following reach it at `7be25fec` once composition exists:
- every start that issues Prepared (S7) and then bails at `execution/native.rs:214`, while holding its lease and maybe its own probe, which the preparation closure releases;
- an S6 Store error whose own confirm returned RolledBack (`prepared.rs:145–147`);
- an S7 conjunct failure;
- a revocation while parked or in conflict backoff (`prepared.rs:173,179`), where `closed` is already produced in task.

G5 composition always refuses, so **no production path reaches RN-1 today**.

### 3.2 Not covered: Held, or the case's own unchanged producer

| Case | Why no RN-1 proof | Result / dependency |
|---|---|---|
| Errors before S5: `claim_start`, same-Unit gate, preparation index, preparation plan/commit, S1, S2, S3, S4, S4b, S5 Immediate (`preparation.rs:840–908`) | No SAME-custody no-dispatch value. Readiness may be `allocated` v1 or `preparing` v2. Helpers may be uncertain. `closure_original` refuses (`:97–100`) | Held and visible. **RN-1b** (§17) |
| Transport installed (S8 onward): registered or uncertain transport, `Offered` handoff, setup/input uncertainty | Not no-dispatch | Transport closure/settlement (transport HOW); TerminalRecovery |
| Genuine successful result, gate or artifact | Not non-success | Unchanged success consumers |
| Unknown irreversible external effect | Before S8 only readonly helpers exist; an uncertain helper prevents S5 | Held |
| Trusted Task-terminal decision, Task fence, Goal pause or cancel | Parent drift (fence callers above) | Held; unchanged TerminalRecovery / `terminal_decision` |
| New epoch or restart | Custody gone; epoch conjunct fails | Held; across-epoch restore gate |
| Non-empty ledger for the operation | Any link implies binding or other activity | Held |
| Start still running | No `StartEnded` witness | Nothing; the in-task path continues |
| Unit factual image changed after the preparation closure | Image differs | Held |
| Generic Failed/Error/Waiting, SQL or audit metadata, Session/process absence, forged flags, copied input | Not authority | No constructor exists (compile refusal) |

## 4. Types, ownership and API (P)

All new types are crate-private, non-Clone, not Serialize/Deserialize, with private fields and no row/ID/DTO constructor.

```rust
// execution/native/nonsuccess.rs (new)
pub(crate) struct NativeNoDispatchClosureProof {
    custody: Weak<NativePreparationCustody>,           // SAME job custody, never strong
    actor: Arc<NativePreparationActor>,                // SAME original actor, revoked
    no_dispatch: Arc<PreparedPhaseNoCurrentDispatch>,  // SAME issued S5 value
    closed: Arc<NativePreparationClosureCommit>,       // SAME known nongrant closure
}
pub(crate) enum NativeClosureStep {
    NotEligible,                                       // start not ended / custody not revoked
    PreparationPending,                                // closure conflict or rolled back; later wake
    Held(anyhow::Error),                               // bounded reason; nothing written
    Proof(Arc<NativeNoDispatchClosureProof>),
}
impl NativePreparationCustody {
    /// Sole issuer. `ended` is constructible only inside runtime::phase_jobs.
    pub(crate) fn nonsuccess_step(self: &Arc<Self>, ended: &crate::runtime::StartEnded)
        -> NativeClosureStep;
}
impl NativeNoDispatchClosureProof {
    pub(crate) fn launch(&self) -> &Arc<PhaseLaunchParts>;
    pub(crate) fn validate_original(&self) -> Result<()>;      // in-memory pointer checks only
    pub(in crate::state) fn closure(&self) -> &Arc<NativeQuotaClosurePlan>;
}
impl NativeSessions {
    /// One bounded step; the existing async loop wraps it with unchanged backoff.
    pub(super) fn close_prepared_step(&self, custody: &Arc<NativePreparationCustody>)
        -> Result<PreparationStep>;                    // Known | Conflict | RolledBack
}

// runtime/phase_jobs.rs (extended)
pub(crate) struct StartEnded { allocation: Arc<NativeAllocation> } // constructor private to phase_jobs
pub(crate) enum InvocationObservation { /* existing */ ClosedNonSuccess }

// state/execution/native_phase/nonsuccess.rs (new)
pub(crate) struct NativeNonSuccessClosurePlan {
    proof: Arc<NativeNoDispatchClosureProof>,
    current: CurrentWorkflowSuccessor,                 // fresh, original-anchored, zero links
    unit_after: Body<ExecutionUnit>,                   // preimage = proof.closure().unit (exact 13 cols)
    operation_after: Vec<SqlValue>,                    // 32 cols; preimage = marker original image
    workflow_after: Body<Record>,
    audit_data: String,                                // <= 4096 bytes
    at: i64,
    operation_mutation: ExactRowMutation,              // managed_phase_operations UPDATE
    record_mutation: ExactRowMutation,                 // records UPDATE; audit permit built in TX
}
pub(crate) struct PhaseClosedAcknowledgment { plan: Arc<NativeNonSuccessClosurePlan> }
pub(crate) enum NativeNonSuccessWrite { Known(PhaseClosedAcknowledgment), Conflict(anyhow::Error) }
pub(crate) enum NativeNonSuccessConfirmation { Known(PhaseClosedAcknowledgment), RolledBack }
impl Store {
    pub(crate) fn plan_phase_nonsuccess_closure(owner: &Arc<RuntimeOwner>,
        proof: Arc<NativeNoDispatchClosureProof>) -> Result<Arc<NativeNonSuccessClosurePlan>>;
    pub(crate) fn close_phase_nonsuccess(&mut self, plan: Arc<NativeNonSuccessClosurePlan>)
        -> Result<NativeNonSuccessWrite>;
    pub(crate) fn confirm_phase_nonsuccess(&mut self, plan: Arc<NativeNonSuccessClosurePlan>)
        -> Result<NativeNonSuccessConfirmation>;
}
impl PhaseClosedAcknowledgment {
    pub(crate) fn matches_allocation(&self, a: &Arc<NativeAllocation>) -> bool;      // Arc::ptr_eq
    pub(crate) fn matches_marker(&self, m: &Arc<OriginalMarker>) -> bool;           // Arc::ptr_eq
}

// runtime/phase_supervisor.rs and runtime/phase_jobs.rs (extended, pub(super))
impl PhaseDispatcher { pub(super) fn reconcile_nonsuccess(&self) -> Result<bool>; }
impl PhaseSupervisor { fn retire_closed_marked(&self, ack: &PhaseClosedAcknowledgment) -> Result<()>; }
impl PhaseJobs {
    pub(super) fn closure_page(&self) -> Result<Vec<(Arc<Job>, Option<StartEnded>)>>; // <= 64
    pub(super) fn retire_closed(&self, job: &Arc<Job>) -> Result<()>;
}
```

Supporting accessors, all `pub(in crate::state)` or narrower:
- `OriginalMarker::original_operation_image()` returns the retained 32-column insert image;
- on `NativeQuotaClosurePlan`: `unit()`, `readiness_after()`, `own_after()` and `no_dispatch()`;
- `NativeQuotaClosurePlan::validate_original` becomes `pub(in native_phase)`.

Nothing in `runtime` can build a proof, plan or acknowledgment.

```text
Runtime -> PhaseDispatcher -> PhaseJobs -> Entry -> Job -> JobState
  JobState.preparation  -> NativePreparationCustody                    (A)
  JobState.nonsuccess   -> Option<Arc<NativeNonSuccessClosurePlan>>    (P, at most one live)
  JobState.closed_ack   -> Option<Arc<PhaseClosedAcknowledgment>>      (P)
  JobState.closure_due, closure_backoff, uncertain, slot_released, attention (P, scalars)
NativePreparationCustody.state.nonsuccess -> Option<Arc<Proof>>        (P, set once)
Proof -Weak-> custody; Proof -> actor, no_dispatch, closed             (existing siblings)
Plan -> proof; Ack -> plan; neither -> Job, PhaseJobs, PhaseSupervisor or Runtime
PhaseSupervisor.queue -> Slot (A); never -> Job or Ack
```

There is no ownership cycle. The custody's proof edge returns to the custody only through `Weak`. Actor → launch → retention → supervisor has no edge to Job.

## 5. Proof issuer (P)

`nonsuccess_step(ended)` performs these steps in order:
1. Require `Arc::ptr_eq(ended.allocation, custody.allocation)`.
2. Under the custody mutex, clone the Arcs and drop the lock. Require:
   - `abandoned`;
   - actor present and `actor.is_revoked()`;
   - `transport.is_none()`;
   - `no_dispatch` present.

   If `nonsuccess` is already set, return the SAME Arc. A missing `no_dispatch` is `Held("pre-no-dispatch: RN-1b")`. A present `transport` is `NotEligible`.
3. If `closed` is empty, run exactly one `close_prepared_step` through `actor.sessions.upgrade()`; an ended `Weak` is `Held`. The step:
   - first confirms a retained closure plan, then plans and closes once;
   - on Known: `retain_closed` and `release_gate`;
   - on Conflict or RolledBack: `PreparationPending`;
   - on a mixed image: `Held`.

   The step never sleeps; Root owns the backoff. No custody lock spans the Store.
4. With `closed` set, require all of:
   - `closed.matches_plan(state.closure)`;
   - `closure.matches(actor, lineage)`;
   - `Arc::ptr_eq(closure.no_dispatch(), state.no_dispatch)`;
   - `actor.validate_original()` (SAME selected sessions, launch ↔ custody link).
5. Build the proof and install it once, pointer-checked.

**Not inputs:** error text, observation labels, rows, IDs, readiness `closed`, `start_ended`, Session or process absence, `NativePhaseStartError`. The proof grants no dispatch, input, binding, settlement, refresh, retry or stop. It has no persisted form; Runtime exit drops it, which leaves the operation Held across epochs.

## 6. Root sweep, fairness and lock order (P)

`PhaseDispatcher::reconcile_nonsuccess()` is synchronous. The service loop calls it right after `reconcile_pending()` (`service.rs:80`), and its `bool` ORs into `pending`. Each sweep:

1. **Page.** Under the `entries` lock only, `closure_page()` collects at most `PAGE = 64` started jobs. It resumes after a persisted `closure_cursor` (operation-ID round robin, wrapping), so every job is considered within ⌈128/64⌉ sweeps. It derives `StartEnded` there: outcome `Some(Err)`, or observation `Uncertain` with a finished handle. It then drops the lock.
2. **Job state.** Per job, under a short `job.state` lock:
   - if `closed_ack` is set, go to step 7;
   - if `closure_due > now`, skip;
   - otherwise clone the custody and any retained plan, then drop the lock.
3. **Caps.** At most `LIVE_PLANS = 4` plans may be retained Root-wide (an atomic counter); jobs beyond the cap wait for their fair turn. Each sweep runs at most 8 preparation-closure steps.
4. **Proof.** Call `custody.nonsuccess_step(&ended)`.
   - `Held`: record bounded attention (an allowlisted category, ≤128 bytes) and set due to now+5 s.
   - `PreparationPending`: set due by backoff.
5. **Plan.** If no plan is retained, run `Store::plan_phase_nonsuccess_closure` on query-only snapshots; the Store mutex is not held. Install the plan once, pointer-checked.
6. **Write.** Under the Store mutex only:
   - if `uncertain` is set, call `confirm_phase_nonsuccess(SAME plan)`; otherwise `close_phase_nonsuccess(SAME plan)`;
   - `Known` goes to step 7;
   - `Conflict(cause)` means a definitive pre-write refusal. The plan is dropped. Contention backs off 100 ms–5 s; currency drift becomes Held attention with a read-only re-probe at most every 5 s;
   - `Err` sets `uncertain`, then the SAME plan is confirmed in the same sweep. `Known` goes to step 7. `RolledBack` keeps the SAME plan and retries later; this is bookkeeping retry only. `Err` stays Held with the plan retained.
7. **Acknowledge** (§11): install `closed_ack`, then `retire_closed_marked`, then `retire_closed`.

**Lock order.** At most one of {`entries`, `job.state`, custody state, `queue`, Store mutex} is held at a time. No await or child/gate action happens under any of them. Removed values are dropped after the lock is released. The existing `start()`, which nests `entries` and `job.state`, is unchanged.

**Cancellation and shutdown.** The sweep checks `stopping` before each job. Retained proof, plan and acknowledgment live in `JobState` until consumed, independent of Engine or Driver futures. Shutdown runs no extra RN-1 sweep. Jobs that are closed but unacknowledged keep `ensure_*shutdown_complete` failing, as today.

## 7. The `phase_closed` transaction (P)

### 7.1 Plan, outside SharedStore

1. **Original and owner.** `proof.validate_original()`. The selected owner must be pointer-equal (as `quota.rs:1237–1243`) and `owner.epoch() == f.epoch`.
2. **Successor.** `current = plan_current_phase(owner, marker)`. Require `!current.has_links()` and `current.workflow_raw()` equal to the marker's `workflow_after` raw bytes.
3. **Snapshot checks.** In one query-only snapshot:
   - `launch.validate_preparation_origin_tx(tx, &current)` (original Source, currency, Driver live);
   - `closure.validate_original(tx)`: revoked actor, no-dispatch, `no_registration`, complete inventory equal to the completion's `after`, original Unit identity;
   - `closure.unit().validate_tx(tx)`;
   - closure readiness postimage `validate_tx`;
   - own waiter absent; own lease equal to the closure's after-image, or absent;
   - `current.unit_raw()` byte-equal to the closure Unit body.
4. **Unit preimage.** The decoded preimage must pass `registration_unit` (Preparing, Session None, both flags 1, work None, disposition Active, original identity) with `wait_reason ∈ {None, Quota, Capacity}`. Build `unit_after` (§10). The identity predicate is that of `with_known_unit` (`successor.rs:48–57`), with a strictly increasing version.
5. **Operation.** The preimage is the marker's original insert image; there is no row read. The postimage copies it with `phase_open` 0 and `version` 2. Its body is the canonical re-encode of the original body object with exactly `phase_open:false` and `version:2` changed; every other key is byte-identical after decode. The body stays ≤4 MiB.
6. **Workflow.** Decode the Record and `WorkflowSnapshot` with a complete typed roundtrip (as `binding.rs:244–250`).
   - The active index must be the marker attempt.
   - Preconditions: state Running; `dispatch_started`; `session_id` and `execution` None; `unit == ManagedUnitRef::from(original)`; `completed_at`, `native_wait` and `next_due` None; no observations; `claimed_observations` 0.
   - Apply only: state Failed, `completed_at=at`, `detail=REASON_DETAIL`; Record `version+1`, `updated_at=at`.
   - Check `workflow::validate_transition(task_after, &after, Some(&before))`.
   - Check that the decoded before and after bodies, with exactly these fields neutralized, are canonically byte-equal.
7. **Payload.** Build the payload (§7.4) and the two exact mutations. `at` is bookkeeping time and is not re-checked under the lock.

### 7.2 Immediate conjuncts (`close_phase_nonsuccess`), all before any write

One `TransactionBehavior::Immediate` under `InventoryBudget`:
1. `selected_database`; `proof.validate_original()` (in memory).
2. `launch.validate_preparation_origin_tx(&tx, &plan.current)`. This covers: open marker operation and input preimages; runtime instance/epoch; P/G/T versions and bodies; Workflow; locks; Context; the complete Unit index equal to the closure factual image; ledger count 0 with no head; Driver live.
3. `closure.validate_original(&tx)`: actor revoked, no-dispatch, `no_registration`, inventory equal to the completion's `after`, Unit original identity.
4. Exact closure readiness postimage. Own waiter absent. Own lease equal to the closure after-image, or absent. `closure.unit().validate_tx` (all 13 columns).
5. `charged_scope_bytes(scope) + workflow growth + audit bytes ≤ WORKFLOW_BYTES`.

Any failure rolls back and returns `Conflict(cause)`, with no write. No hashing, encoding, policy or image building runs under SharedStore.

### 7.3 Writes, images and guards

Writes run in this order, each with rowcount exactly 1; then `ensure_consumed` and commit.

| # | Row | Preimage (complete) | Postimage | Guard |
|---|---|---|---|---|
| W1 | `execution_units` (13 columns) | Closure factual open image, version Uₖ | §10 delta, version Uₖ+1 | Exact CAS `WHERE` all 13 `IS` preimage; writer-contract guard; not a permit table |
| W2 | `managed_phase_operations` (32) | Marker original image, `phase_open` 1, version 1 | `phase_open` 0, version 2, body delta | Permit row 1; core trigger (`schema.sql:148`) |
| W3 | `records` Workflow (7) | Marker `workflow_after`, version Vₘ | Version Vₘ+1, §7.1 item 6 delta | Permit row 2; identity/version trigger (`schema.rs:334`) |
| W4 | `audit` (7) | No link for the operation | One `phase_closed`; `sequence` = `sqlite_sequence`+1 under TX | Permit row 3 built in TX; private, no_replace and chain triggers (`schema.rs:366–414`), which need W2 and W3 first |
| — | `managed_phase_readiness` | Closure `closed` image | Unchanged, compared | — |
| — | Own `quota_waiters` / `quota_leases` | Absent / closure after-image | Unchanged, compared | — |
| — | `quota_pools` | Not compared (foreign writers legitimately change it after closure) | Unchanged | — |
| — | Tasks, Projects, Goals, Contexts, locks, `task_drivers`, `source_recoveries`, owners, inputs, admissions, Session records, `native_invocations`, `managed_effects`, `task_execution`, `cleanup_jobs` | Compared where §7.2 lists them | **Never written** | — |

### 7.4 `phase_closed` payload

Header (the trigger-required keys, as `binding.rs:298–310`):
- `kind="phase_closed"`, `project_id`, `goal_id`, `task_id`, `workflow_id`, `generation`, `attempt_index`, `phase`;
- `workflow_version_before` (Vₘ), `workflow_version_after` (Vₘ+1), `context_version`;
- `private_operation_ref`, `original_marker_frame_sha256`, `workflow_body_sha256_before`, `workflow_body_sha256_after`, `prior_ledger_digest` (= the marker digest);
- `canonical_body_recipe="rrx.workflow-body-sha256/v1"`, `at`.

Typed projection:
- `closure="non_success"`, `proof_source="no_current_dispatch"`, `reason="start_ended_before_dispatch"`;
- `attempt_state_after="failed"`, `awaiting="explicit_retry"`, `task_version_preserved`, `session_bound=false`;
- `allocated_session_id`, `unit_id`, `unit_version_before`, `unit_version_after`, `operation_version_after=2`;
- `readiness_version` (the closure's `closed` row), `lease_released`, `probe_released` (booleans from the closure images).

The key set is exact. The payload carries no error text, argv, environment, credential, path or grant. Complete encoding is ≤4096 bytes.

### 7.5 Budgets

| Item | Bound |
|---|---|
| One plan | Current Workflow ≤8 MiB + Workflow after ≤8 MiB + operation after ≤4 MiB (the original image is shared with the marker) + 2 Unit images ≤16 KiB each + payload ≤4 KiB ≈ ≤20.1 MiB |
| Retained plans | `LIVE_PLANS = 4` Root-wide, so ≤81 MiB worst case. Each plan is dropped on acknowledgment or definitive conflict. Uncertain plans count against the cap (liveness cost only) |
| One sweep | ≤64 jobs considered, ≤8 preparation-closure steps, ≤4 RN-1 Immediates and ≤4 confirms |
| Immediate reads | Existing bounded validators only: marker rows, projection, Driver anchor, ledger `LIMIT 257`, Unit ≤16 KiB, readiness ≤4 KiB, one own waiter and one own lease row, `InventoryBudget` |
| Writes | 4 rows, 3 permits (≤128) |
| Ledger | 0 → 1 link (≤256); link ≤4096 bytes |
| Reservation | `phase_open` 1→0 releases `max(0, 1 MiB − spent)` (derived) |
| Workflow growth | `REASON_DETAIL` ≤128 UTF-8 bytes, plus `completed_at` and the state token, checked against `WORKFLOW_BYTES` |

`REASON_DETAIL` is the constant `"native start ended before dispatch; explicit retry required"`.

## 8. Uncertain commit (P)

`confirm_phase_nonsuccess(SAME plan)` runs one Immediate with no write. It reads the Unit (13 columns), the operation (32), the Workflow (7), every link for the operation (`LIMIT 257`), the runtime epoch, readiness and the own waiter and lease.

- **Known** iff all of:
  - the epoch is the original;
  - Unit, operation and Workflow equal their postimages exactly;
  - exactly one link exists: `phase_closed` with byte-equal scope, `at` and data, and any `sequence` > 0;
  - readiness is the closed image; the own waiter is absent and the own lease is unchanged.

  Returns an acknowledgment for the SAME plan Arc.
- **RolledBack** iff the epoch is the same, Unit, operation and Workflow equal their preimages, and there are zero links. This permits a bookkeeping retry of the SAME plan only; there is no new plan and no new `at`.
- **Anything else is Held:** mixed images, a foreign or different link, a foreign epoch, or a row equal to neither image. Only a read-only probe at most every 5 s follows. Nothing is rebuilt from current rows. Link presence alone is never acknowledgment. A preimage is never success.

`validate_current_tx` is not used here because it requires the OPEN operation image.

## 9. Workflow non-success and retry visibility (P)

- **Durable state.**
  - `active = Some(i)` is kept, with state Failed, `completed_at` and `REASON_DETAIL`.
  - `session_id` and `execution` stay None; no binding is invented.
  - The Task row is unchanged, at its marker version and state: no `WaitingHuman`, no blocker and no Task-terminal success, failure or cancel (unlike `Engine::fail`, `workflow.rs:2571–2581`).
  - Context, artifact, Session and permission are unchanged or absent.
- **Derived status.**
  - `poll` yields `StepResult::Failed{detail}` read-only (`workflow.rs:1940–1945`).
  - `runtime/waiting.rs:35–60` stops reporting the operation because it filters `phase_open=1`.
  - Driver occupancy (`claim.rs:337`) frees at commit.
  - The Driver loop keeps polling every 100 ms read-only (`runtime/task_driver.rs:97–115`); control W2 verifies this.
- **Retry is not delivered.** `Engine::retry`'s managed branch (`workflow.rs:3104–3157`) passes its Unit checks once both flags are false. Its generic persist then hits the protected Record guard (`schema.rs:328`), which refuses the whole transaction.

  A typed protected retry port, **RN-R**, is a separate dependency. It must provide a fresh worktree, Unit and generation, advance `task_execution`, retire the readonly generation and clean up the namespace. There is no automatic Agent restart, worktree reuse or input replay.

## 10. Unit permission retirement (P)

**Producer.** RN-1's transaction is the trusted precise producer for this case only. It changes the flags only from the exact open image, and only under all of these facts:
- the SAME revoked actor;
- no Session, `session_units` row, invocation or admission;
- the complete inventory equals the completion's `after`, so every helper is settled per the S5 manifest;
- a known preparation closure exists (readiness `closed`; own lease, waiter and probe released);
- the ledger is empty;
- no transport was ever installed.

The retirement is never inferred from readiness `closed`, `start_ended`, a Failed observation or process absence.

**Delta.**
- Preparing → Retired.
- `native_effects_open` 1→0; `result_finalization_open` 1→0.
- `wait_reason` → None; disposition Active → `Refused`.
- `work` stays None: no work was dispatched. Unknown is not invented, and Success or Failure is not claimed.
- `version+1`, `updated_at=at`.
- Session None; `artifact_id` and `capacity_retry_at` unchanged.

**Deliberately not done here (RN-R).** `task_execution` generation and `active_unit`; `retire_readonly_generation`; the cleanup job. The worktree remains, one per closed operation.

`retire_execution_as` (`execution.rs:1033–1069`) is not used. It relies on generic `validate_authority`, sets `work=Unknown` and advances the generation.

**Consumers afterwards.**
- `managed_attempt_retired` is true for its existing callers.
- The cleanup worker still requires a job row (`cleanup.rs:82`), and none is written.
- Quota `WAITERS` and `fence_task_tx` ignore closed Units.

## 11. Acknowledgment, capacity and job release (P)

- **Issuer.** `PhaseClosedAcknowledgment` is issued only by `close_phase_nonsuccess` (commit `Ok`) or `confirm_phase_nonsuccess` (`Known`), for the SAME plan Arc.
- **Consumers, in order:**
  1. Install `JobState.closed_ack`. `ack.matches_allocation(job.allocation)` must hold. Send `ClosedNonSuccess`.
  2. `PhaseSupervisor::retire_closed_marked(ack)`, under the `queue` lock. Require:
     - the entry is pointer-equal to the slot allocation and `ack.matches_marker(slot.marker)`;
     - `Publishing` and `launch_handed_off`.

     Then remove the slot from `entries`, `projects` and `rotation`, unlock, and call `Self::release` (an accepted guard's drop is a no-op).

     This is idempotent: if the slot is already absent and `slot_released` is set, return `Ok`.
  3. `PhaseJobs::retire_closed(job)`, under the `entries` lock. Require the entry to be pointer-equal to the job, with `closed_ack` set and `slot_released` true. Remove the entry and drop it after unlocking.
- **Removal failure.** If step 2 or 3 fails, the acknowledgment is retained and only the removal is retried; the DB is not written again.
- **Nothing else releases a marked slot or started job:** not observation Failed or Uncertain, not outcome `Err`, not readiness `closed`, not `phase_open=0` read from SQL, not shutdown.

## 12. Error, cancel and drift behavior (P)

| Observation | Behavior | RN-1 writes |
|---|---|---|
| Start still running / outcome `Ok(Launched)` | Not eligible / not RN-1 | none |
| `no_dispatch` absent | Held, "pre-no-dispatch" (RN-1b) | none |
| Transport installed | Not RN-1 (transport/terminal paths) | none |
| Preparation-closure conflict or rollback | `PreparationPending`; one step per job per sweep; 100 ms–5 s backoff | only that closure's own rows |
| Preparation-closure mixed images / selected sessions ended | Held | none |
| Parent, Context, Workflow, Source or lock drift; Driver not live | `Conflict` → Held; plan dropped; read-only re-probe at most every 5 s; **no pin refresh, no drift laundering** | none |
| Unit changed after the preparation closure (fence, legacy, raw SQL) | Held | none |
| Ledger non-empty / epoch changed | Held | none |
| `SQLITE_BUSY` before the first write | `Err` → confirm → RolledBack → SAME plan later | none |
| Commit uncertain | Confirm the SAME plan (§8) | none from confirm |
| Live-plan cap reached | Wait for the next fair turn | none |
| Slot or job removal fails after Known | Acknowledgment retained; removal retried | none |
| Runtime stopping | Sweep returns; retained values die with the Runtime → Held across epochs | none |
| Later G3 `request_stop` | Not called by RN-1. If it genuinely revokes a parked start, the in-task closure runs and RN-1 follows only with unchanged parents | as above |

## 13. Authority surface and source-to-sink (P)

The roles are local Runtime and cross-Task roles; there are no web roles.

| Role | Obtain proof | Write `phase_closed` | Retire Unit flags | Release slot/job | Basis |
|---|---|---|---|---|---|
| SAME selected Native start (in task) | issues S5; runs the in-task closure | no | no | no | No Store RN-1 port; returns only `Err` |
| Root PhaseDispatcher sweep (same owner/epoch) | yes, with its own `StartEnded` | yes, via the Store port | yes, inside the RN-1 TX only | only with an acknowledgment | §§5–11 |
| Store private port | validates only | yes, SAME plan | yes, exact images | issues the acknowledgment | §7–8 |
| Same-Task Engine/Driver (generic persist, `fail`, `retry`, `refresh_owners`) | no | refused by the Record guard | no | no | `schema.rs:317–336` |
| Other Tasks' Drivers/starts (cross-Task) | no (different allocation and custody) | no | no | no | Pointer checks; quota rows are only compared |
| CLI/user: status, cancel, retry | no | no | cancel uses the fence (parent drift) → Held / TerminalRecovery | no | status read-only; retry refused (RN-R) |
| Root SourceStop/Cancel (unresolved) | no new API | no | no | no | Only an already genuine revocation, consumed as-is |
| Cleanup worker | no | no | no | no | Needs flags 0 and a job row |
| New epoch / restarted Runtime | no (custody gone) | no | no | no | Held |
| Cached old10 binaries / raw same-user SQL | no | Supported binaries are refused by triggers and permits; raw SQL is outside the cooperative model | — | — | Not a sandbox |
| Native Agent / hooks | none exist (no dispatch) | — | — | — | — |

| Sink | Only allowed source | Rejected sources |
|---|---|---|
| Operation `phase_open` 1→0 | Plan from the proof plus the marker's original image | Rows, IDs, readiness `closed`, observation |
| Workflow Failed | Plan from the proof plus a fresh original-anchored successor | Error text (detail is a constant), `Engine::fail`, CLI |
| `phase_closed` audit | Payload from proof, marker and plan | Caller JSON, generic audit API |
| Unit flags | Closure factual open image under the proof | Readiness inference, fence, legacy retire |
| Slot/job release, `ClosedNonSuccess` | Acknowledgment | Failed/Uncertain observation, outcome `Err`, SQL |

**Business-logic invariants.** Each is enforced at the final write:
- **Once:** the unique link trigger, the operation version 1→2 CAS and the single `JobState` plan.
- **Order:** the issuer needs `closed`; the Immediate needs readiness `closed`; audit needs W2 and W3 first.
- **No replay:** a stale plan after drift is `Conflict`; a second acknowledgment is impossible because the operation is at version 2.
- **No skip:** no proof means no plan.
- **No other-operation substitution:** allocation and marker are pointer-checked.

## 14. Impact analysis

| Changed or consumed | Consumers checked (V) | Impact / handling |
|---|---|---|
| Custody `nonsuccess` slot; `closure_original` + `transport.is_none()`; `retain_transport` clears `no_dispatch` (C-R1) | `prepared.rs:24–185`; `preparation.rs:51–72,88–154,636–726`; `transport.rs:563` | Pre-transport behavior is unchanged. After transport, `reconcile_known_commit` already takes the transport branch first |
| `close_prepared_on_revocation` → loop over `close_prepared_step` | `prepared.rs:173,179`; `preparation.rs:670` | Same semantics and backoff |
| Closure-plan accessors | `quota.rs` only | Read-only, crate-private |
| `OriginalMarker::original_operation_image` | `marker_rows.rs:97–108`; `publication.rs` | Read-only |
| First `phase_closed` producer | `schema.rs:358–414`; `successor.rs:19` (KINDS) | After close, `validate_open_tx` fails, so every open-currency consumer refuses: binder, registration, quota, transport, version helpers, `plan_current_phase`. Intended |
| `phase_open=0` | `publication.rs:366` (reservation release); `waiting.rs:60`; `claim.rs:337`; `quota_policy.rs:118`; `quota.rs:13`; admission trigger `schema.sql:195` | Status stops, capacity and reservation free, consumption impossible |
| Unit flags/state | `state/mod.rs:935,946`; `workflow.rs:3104–3157`; `cleanup.rs:82`; `execution.rs:325`; `verification.rs:452`; quota WAITERS | Retired and inert; no cleanup job |
| Workflow Failed attempt | `poll` (`workflow.rs:1940`); status/CLI; `validate_transition`; TerminalRecovery (`state/mod.rs:692,767,940`) | Read-only Failed; TerminalRecovery unchanged |
| PhaseJobs, PhaseSupervisor, service loop; `ClosedNonSuccess` | `service.rs:80,134–136`; `phase_jobs.rs:84–99` (`wait` treats it as terminal); handoff drops `PhaseInvocation` (`phase_supervisor.rs:915`) | Unmarked paths and `fair_page` unchanged |
| Constants: `PAGE` 64, `LIVE_PLANS` 4, backoff 100 ms–5 s, `REASON_DETAIL` ≤128, payload ≤4096, ledger ≤256 | New, RN-1 only; trigger header list `schema.rs:368–396` | Header keys satisfied; projection keys free |
| Environment variables, input lists, paths | None | No impact |

**Not affected:**
- DDL, triggers, permit catalogue and `SCHEMA_VERSION`;
- Task, Context, Session, owner, input and admission rows; quota pools;
- transport, binder, gates, TerminalRecovery and success capture;
- legacy units; B1/B2.

## 15. Controls and evidence handoff

1. **Primitive controls** (compiled, no authority, no lifecycle credit):
   - the Workflow delta builder: Running-only; refuses any other field change;
   - the operation body transform: exactly two keys change;
   - the payload: exact key set, ≤4096 bytes, no error text;
   - the confirm classifier over (Unit, operation, Workflow, link) tuples: every post/pre/mixed combination;
   - the Unit delta identity predicate;
   - cursor fairness and the live-plan cap.
2. **Genuine causal pairs.** These require production Runtime → Driver → Source → Frame → marker → `PhaseJobs.start` → selected `NativeSessions`/actor/custody, with the actual protocol fixture configured as an external CLI. There is no test-only authority constructor, fake grant or prepared metadata. G5 is closed, so every row below is **UNVERIFIED/SETUP until integration**. A SETUP refusal earns no positive or kill credit.

   | ID | Normal | Single changed condition | Required refusal evidence |
   |---|---|---|---|
   | H1 evidence handoff | S7 bail → sweep → W1–W4, acknowledgment, slot and job removed | genuine S4b refusal (no S5) | Held; zero RN-1 rows; slot and job retained; shutdown check fails |
   | H2 | parked revocation → in-task closure → RN-1 | start not ended | no proof issued |
   | C1 exact currency | as H1 | genuine Task change or Goal pause before the Immediate | `Conflict`/Held; all rows byte-identical |
   | C2 | as H1 | Driver genuinely invalidated | Held |
   | C3 | as H1 | epoch advanced | Held; no reconstruction |
   | L1 ledger/idempotence | Known, then another sweep | — | no second write; one link; operation version 2 |
   | L2 | commit failpoint returns `Err` after commit | — | confirm Known; one link |
   | L3 | failpoint before commit | — | RolledBack; SAME plan and `at`; one link |
   | U1 permission retirement | `managed_attempt_retired` true; no cleanup job | Unit fenced first | Held; RN-1 wrote nothing |
   | A1 acknowledgment | slot and job removed; `ensure_*` pass | Store refusal (C1) | both retained; `ensure_*` fail |
   | W1 caller wiring | the service loop drives RN-1 | service call removed (mutant) | H1 fails: no link |
   | W2 visibility | `poll` Failed read-only; waiting None; Task unchanged | — | no write during Driver polling |
   | R1 retry | `Engine::retry` on a closed protected Workflow | — | refused; zero writes |
   | Q1 lease | own lease released; a sibling genuine Unit is admitted | — | — |

3. **Compiled omissions.** Each mutant must fail its intended assertion:
   - the issuer without `closed`, without the transport-empty check, or with `StartEnded` from another allocation;
   - plan Unit taken from current rows instead of the closure image;
   - `validate_preparation_origin_tx`, Driver-live or epoch check omitted;
   - non-empty ledger allowed;
   - Task written, or `session_id` set;
   - detail taken from error text;
   - flags left open, or retired without the exact open image;
   - operation `phase_open` unchanged, or audit inserted before W2/W3;
   - confirm accepting a link alone or a preimage as Known, or rebuilding the plan;
   - slot or job removed without an acknowledgment, or on a Failed observation;
   - `remove_unmarked` used for a marked slot;
   - service call removed;
   - live-plan cap ignored;
   - a Root lock held across the Store (debug assertion).

   A missing constructor is a compile refusal, not a kill.

## 16. Corrections to approved documents (minimum, evidence-backed)

| ID | Correction | Evidence (V) |
|---|---|---|
| C-R1 | Prepared HOW §5 item 3: `retain_transport` must clear `no_dispatch`, and `closure_original` plus the RN-1 issuer must require `transport.is_none()` | `preparation.rs:51–72,88–110` |
| C-R2 | Prepared HOW §7.4 item 2, "offers the SAME no-dispatch value to Root": the handoff is pull-based. Root obtains the proof from the SAME custody after the start ended; nothing travels through `NativePhaseStartError` | `phase_jobs.rs:270–292`; `execution/native.rs:90–93` |
| C-R3 | Managed binding §6, "Engine::fail is non-success closure/awaiting_explicit_retry": for this case no Task `WaitingHuman`/blocker is written. Awaiting explicit retry means an active Failed attempt plus `phase_open=0`; retry is RN-R | `workflow.rs:2571–2581`; `schema.rs:328` |
| C-R4 | Prepared HOW §6.3 Closure row, "Unit retirement belongs to the trusted cancel/fence owner", is extended: in the RN-1 case (no fence, unchanged parents) RN-1 is that trusted producer. The nongrant preparation closure still never writes the Unit | `execution.rs:323–356`; `quota.rs:1304–1364` |

## 17. Unresolved dependencies and open gates

- **G5** composition and its pending HOW correction. Until it lands, no protected start reaches RN-1 and every genuine control is SETUP/UNVERIFIED.
- **G3** Root `request_stop` caller; Root SourceStop/Cancel author/permission decisions; the R2 shared Git environment author exception. None is decided or widened here. RN-1 mints no stop authorization.
- **RN-1b:** a non-success proof for errors before S5 (pre-helper refusal, settled-helper refusal at S4/S4b/S5). Until it exists, these stay Held.
- **RN-R:** explicit protected retry, including fresh worktree/Unit/generation, `task_execution` advance, readonly-generation retirement and namespace cleanup of the closed Unit.
- TerminalRecovery / `terminal_decision` for fenced or drifted operations; across-epoch restore; transport closure for S8 onward.
- No final RN-1 sweep at shutdown.
- Regression and Clippy RED. N1/N4/H/Q/install/both-OS/dogfood qualification. The Official-Agent/four-Task/both-OS MERGE gate.
- Diff of §2 against the unread reconnaissance report.

## 18. Review plan

- **This HOW:** ONE independent Sol 6.1 (`gpt-6.1-sol`) high design review. No Grok, Fable or Astra. Re-review covers only the delta of confirmed findings, by the finding side.
- **Source increment:** implemented by Sol 6.1 high. The initial review runs three roles in parallel at high: Sol on the implementation diff, Opus on the security/contract diff, Grok on reconciliation only. The fix is made by the side that did not raise the finding.
- **Master design:** updated only by the implementation PR, with implemented facts only (`master/agent-execution.md`, `master/workflow-engine.md`). This HOW updates no master.
