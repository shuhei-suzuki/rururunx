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

7. **Correction scope.** This revision resolves the seven confirmed findings RN1-SOL-01 to RN1-SOL-07 of the independent Sol high review of `b03b6622`. The source pin is unchanged. The G5 component `c51af44` was read only to locate the replaced bail and the driven entry (§§9, 15); it is not a source pin of this HOW.

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
| Start path | `execution/native.rs:208–215` always bails after `begin_phase_preparation` returns Prepared ("transport composition unavailable"). `start_prepared_transport` (`execution/native/transport.rs:498`) has no caller (G5). G5 (component `c51af44`, not this pin) replaces the bail with that call, so the bail is not an RN-1 producer (§15 item 2) |
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

By reading, once G5 composition and the authentic composed Workflow activation producer exist (§17), the following reach it:
- an S6 quota Store error whose own confirm returned RolledBack (`prepared.rs:145–147`);
- an S7 conjunct failure;
- a refusal in `start_prepared_transport` before `retain_transport` (`transport.rs:503–562`), such as `qualified_physical_profile` (`version.rs:376–383`);
- a revocation while parked or in conflict backoff (`prepared.rs:173,179`), where `closed` is already produced in task; only an existing genuine revoker reaches this (G3 is unresolved).

The `execution/native.rs:214` bail is replaced by G5 and is not an RN-1 producer. At this source pin composition refuses earlier (`state/managed_binding/composition.rs:70–75`), so **no production path reaches RN-1 today**.

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

All new types are crate-private, non-Clone, not Serialize/Deserialize, with private fields and no row/ID/DTO constructor. A restricted `pub(in path)` must name an ancestor of its defining module, so an item read across the `execution` and `state` trees is `pub(crate)` and sealed by an unforgeable argument: a pointer-checked witness, or the `NonSuccessReader` token that only `state::execution::native_phase::nonsuccess` can construct. No accessor returns a reference derived from a temporary `Weak` upgrade or lock guard; every exported reference borrows an Arc field of the value itself.

```rust
// execution/native/nonsuccess.rs (new); execution/native.rs adds `mod nonsuccess;`
// and `pub(crate) use nonsuccess::{NativeClosureStep, NativeNoDispatchClosureProof};`
pub(crate) struct NativeNoDispatchClosureProof {
    custody: Weak<NativePreparationCustody>,           // SAME job custody, never strong
    actor: Arc<NativePreparationActor>,                // SAME original actor, revoked
    no_dispatch: Arc<PreparedPhaseNoCurrentDispatch>,  // SAME issued S5 value
    closure: Arc<NativeQuotaClosurePlan>,              // SAME Arc as the custody `closure`
    closed: Arc<NativePreparationClosureCommit>,       // closed.matches_plan(&closure)
}
pub(crate) enum NativeClosureStep {
    NotEligible,                                       // not revoked, or transport installed
    PreparationPending,                                // closure conflict or rolled back; later turn
    Held(anyhow::Error),                               // bounded reason; nothing written
    Proof(Arc<NativeNoDispatchClosureProof>),
}
impl NativeNoDispatchClosureProof {
    /// Visible only inside execution::native; its sole caller is `nonsuccess_step`.
    pub(super) fn issue(custody: &Arc<NativePreparationCustody>, actor: Arc<NativePreparationActor>,
        no_dispatch: Arc<PreparedPhaseNoCurrentDispatch>, closure: Arc<NativeQuotaClosurePlan>,
        closed: Arc<NativePreparationClosureCommit>) -> Result<Self>;
    pub(crate) fn launch(&self) -> &Arc<PhaseLaunchParts>;                  // actor.launch()
    pub(crate) fn validate_original(&self) -> Result<()>;                   // §5, memory only
    pub(crate) fn closure(&self, _: &crate::state::NonSuccessReader) -> &Arc<NativeQuotaClosurePlan>;
}

// execution/native/preparation.rs (extended; CustodyState stays private here).
// CustodyState gains `nonsuccess: Option<Arc<NativeNoDispatchClosureProof>>`, set once.
impl NativePreparationCustody {
    /// Sole issuer. `ended` is constructible only inside runtime::phase_jobs.
    pub(crate) fn nonsuccess_step(self: &Arc<Self>, ended: &crate::runtime::StartEnded)
        -> NativeClosureStep;
}

// execution/native/prepared.rs (extended)
pub(super) enum PreparationStep { Known, Conflict, RolledBack, Held(anyhow::Error) }
impl NativeSessions {
    /// One bounded step; `close_prepared_on_revocation` loops it with its unchanged backoff.
    pub(super) fn close_prepared_step(&self, custody: &Arc<NativePreparationCustody>)
        -> Result<PreparationStep>;
}

// runtime/phase_jobs.rs (extended); runtime/mod.rs adds `pub(crate) use phase_jobs::StartEnded;`
pub(crate) struct StartEnded { allocation: Arc<NativeAllocation> }   // built only in phase_jobs
impl StartEnded {
    pub(crate) fn matches_allocation(&self, a: &Arc<NativeAllocation>) -> bool;   // Arc::ptr_eq
}
pub(crate) enum InvocationObservation { /* existing */ ClosedNonSuccess }

// state/managed_binding/closure.rs (new). ManagedMarkerPlan fields are pub(super),
// so marker-derived images are planned and validated inside managed_binding.
pub(in crate::state) struct UnlinkedPhaseClosure {   // compact, retained (§7.5)
    attempt: usize,
    workflow_version_after: u64,
    workflow_growth: u64,
    at: i64,
    workflow_after_sha256: String,                    // WORKFLOW_DOMAIN digest, 64 hex
    operation_body_after_sha256: String,              // 64 hex
}
pub(in crate::state) struct PhaseClosureImages {     // transient, never retained
    workflow_after: Record,
    workflow_after_raw: String,
    operation_after: Vec<SqlValue>,                   // 32 columns
}
pub(in crate::state) enum PhaseImage<'a> {
    Open,
    Closed { images: &'a PhaseClosureImages, at: i64, data: &'a str },
}
pub(in crate::state) fn plan_unlinked_closure(tx: &Transaction<'_>, marker: &OriginalMarker, at: i64)
    -> Result<UnlinkedPhaseClosure>;
pub(in crate::state) fn validate_original_phase_tx(tx: &Transaction<'_>, marker: &OriginalMarker,
    unit: (&ExecutionUnit, &str), image: PhaseImage<'_>) -> Result<()>;      // §7.2 item 2, §8
impl UnlinkedPhaseClosure {
    pub(in crate::state) fn materialize(&self, marker: &OriginalMarker) -> Result<PhaseClosureImages>;
    pub(in crate::state) fn mutations(&self, marker: &OriginalMarker, images: &PhaseClosureImages)
        -> Result<[ExactRowMutation; 2]>;             // operation and records UPDATE
    pub(in crate::state) fn validate_budget_tx(&self, tx: &Transaction<'_>, marker: &OriginalMarker,
        audit_bytes: u64) -> Result<()>;
}

// state/execution/native_phase/nonsuccess.rs (new); re-exported like `native_phase.rs:23–25`
pub(crate) struct NonSuccessReader(());              // private field: built only here
pub(crate) struct NativeNonSuccessClosurePlan {
    proof: Arc<NativeNoDispatchClosureProof>,
    images: UnlinkedPhaseClosure,
    unit_after: Vec<SqlValue>,                       // 13 columns; body <= 16 KiB
    audit_data: String,                              // <= 4096 bytes
}
pub(crate) struct NativeNonSuccessMaterial {         // one turn; consumed by one Store call
    plan: Arc<NativeNonSuccessClosurePlan>,
    images: PhaseClosureImages,
}
pub(crate) struct PhaseClosedAcknowledgment { plan: Arc<NativeNonSuccessClosurePlan> }
pub(crate) enum NativeNonSuccessWrite { Known(PhaseClosedAcknowledgment), Conflict(anyhow::Error) }
pub(crate) enum NativeNonSuccessConfirmation { Known(PhaseClosedAcknowledgment), RolledBack }
impl Store {
    pub(crate) fn plan_phase_nonsuccess_closure(owner: &Arc<RuntimeOwner>,
        proof: Arc<NativeNoDispatchClosureProof>) -> Result<Arc<NativeNonSuccessClosurePlan>>;
    pub(crate) fn materialize_phase_nonsuccess(plan: &Arc<NativeNonSuccessClosurePlan>)
        -> Result<NativeNonSuccessMaterial>;         // no Store mutex
    pub(crate) fn close_phase_nonsuccess(&mut self, material: NativeNonSuccessMaterial)
        -> Result<NativeNonSuccessWrite>;            // Err = uncertain
    pub(crate) fn confirm_phase_nonsuccess(&mut self, material: NativeNonSuccessMaterial)
        -> Result<NativeNonSuccessConfirmation>;     // Err = Held
}
impl PhaseClosedAcknowledgment {
    pub(crate) fn matches_allocation(&self, a: &Arc<NativeAllocation>) -> bool;      // Arc::ptr_eq
    pub(crate) fn matches_marker(&self, m: &Arc<OriginalMarker>) -> bool;           // Arc::ptr_eq
}

// runtime (extended). Job stays private to phase_jobs, so the per-job sweep lives there.
impl PhaseDispatcher { pub(super) fn reconcile_nonsuccess(&self) -> Result<bool>; }  // delegates
impl PhaseJobs {
    pub(super) fn reconcile_nonsuccess(&self, phases: &PhaseSupervisor, stopping: &AtomicBool)
        -> Result<bool>;
    fn closure_snapshot(&self) -> Result<Vec<(OperationId, Arc<Job>, bool)>>;      // bool: handle finished
    fn retire_closed(&self, job: &Arc<Job>) -> Result<()>;
}
impl PhaseSupervisor {
    pub(super) fn retire_closed_marked(&self, ack: &PhaseClosedAcknowledgment) -> Result<()>;
}
```

Supporting accessors, each at a legal and narrowest visibility:
- `quota.rs`: `NativeQuotaClosurePlan::validate_original` becomes `pub(super)` (visible in `native_phase` and its child `nonsuccess`); new `pub(super)` `unit()` and `validate_after_tx(tx)` (readiness equals the `closed` image, own waiter absent, own lease equal to the after-image or absent); new `pub(crate)` `matches_no_dispatch(&Arc<PreparedPhaseNoCurrentDispatch>) -> bool` (pointer check only).
- `publication.rs`: `OriginalMarker::validate_unadvanced_tx(tx)` (operation, input and owner rows equal their retained insert images, reusing `Insert::validate_tx`, `marker_rows.rs:59–77`) and `original_operation_image()`, both `pub(super)`.
- `successor.rs`: the ledger reader `rows` and `KINDS` become `pub(super)` for `closure.rs`. `CurrentWorkflowSuccessor::workflow_raw` stays `pub(super)`; RN-1 builds no successor (§7.1).

Nothing in `runtime` can build a proof, plan, material, reader or acknowledgment.

```text
Runtime -> PhaseDispatcher -> PhaseJobs -> Entry -> Job -> JobState
  JobState.preparation  -> NativePreparationCustody                    (A)
  JobState.nonsuccess   -> Option<Arc<NativeNonSuccessClosurePlan>>    (P, compact, at most one)
  JobState.closed_ack   -> Option<Arc<PhaseClosedAcknowledgment>>      (P)
  JobState.closure_due, closure_backoff, uncertain, slot_released, attention (P, scalars)
PhaseJobs.closure_cursor -> Option<OperationId>                        (P, leaf mutex)
NativePreparationCustody.state.nonsuccess -> Option<Arc<Proof>>        (P, set once)
Proof -Weak-> custody; Proof -> actor, no_dispatch, closure, closed    (existing siblings)
Plan -> proof; Material -> plan (one turn); Ack -> plan
None of them -> Job, PhaseJobs, PhaseSupervisor or Runtime
PhaseSupervisor.queue -> Slot (A); never -> Job or Ack
```

There is no ownership cycle. The custody's proof edge returns to the custody only through `Weak`. Actor → launch → retention → supervisor has no edge to Job.

## 5. Proof issuer (P)

`nonsuccess_step(ended)` performs these steps in order. No step holds the custody mutex across another lock, the Store or an await.
1. Require `ended.matches_allocation(&self.allocation)`; otherwise `Held`.
2. Under the custody mutex, clone the Arcs and flags, then drop the lock:
   - `nonsuccess` already set: return the SAME Arc;
   - not `abandoned`, or the actor absent or not revoked: `NotEligible`;
   - `transport` present: `NotEligible` (transport paths);
   - `no_dispatch` absent: `Held("pre-no-dispatch: RN-1b")`.
3. If `closed` is empty, run exactly one `close_prepared_step` through `actor.sessions.upgrade()`; an ended `Weak` is `Held`. The step:
   - first confirms a retained closure plan, otherwise plans and closes once;
   - on Known: `retain_closed` and `release_gate`, then continues to step 4;
   - on Conflict or RolledBack: `PreparationPending`;
   - on a mixed image: `Held`.

   The step never sleeps; Root owns the backoff.
4. Under the custody mutex, clone `closure` and `closed`, then drop the lock. Require:
   - `closed.matches_plan(&closure)`;
   - `closure.matches(&actor, lineage)` and `closure.matches_no_dispatch(&no_dispatch)`;
   - `transport` still empty.
5. Without any lock: `actor.validate_original()` (SAME selected sessions, launch ↔ custody link) and `no_dispatch.validate_original(&actor)`.
6. `NativeNoDispatchClosureProof::issue` re-checks closed ↔ closure and closure ↔ actor/no-dispatch. Install the Arc under the custody mutex only if `nonsuccess` is empty; otherwise return the installed Arc.

`NativeNoDispatchClosureProof::validate_original()` is memory-only. It upgrades the custody `Weak`. Under the custody mutex it requires `nonsuccess`, `closure`, `closed` and `no_dispatch` pointer-equal to the proof's own fields, and `transport` empty. Then, without that lock, it requires `actor.validate_original()`, `actor.is_revoked()` and `no_dispatch.validate_original(&actor)`.

**Not inputs:** error text, observation labels, rows, IDs, readiness `closed`, `start_ended`, Session or process absence, `NativePhaseStartError`. The proof grants no dispatch, input, binding, settlement, refresh, retry or stop. It has no persisted form; Runtime exit drops it, which leaves the operation Held across epochs.

## 6. Root sweep, fairness and lock order (P)

`PhaseDispatcher::reconcile_nonsuccess()` is synchronous and delegates to `PhaseJobs::reconcile_nonsuccess(&self.phases, &self.stopping)`, because `Job` is private to `phase_jobs`. The service loop calls it right after `reconcile_pending()` (`service.rs:80`), and its `bool` ORs into `pending`. Each sweep:

1. **Snapshot.** Under `entries` only, `closure_snapshot()` clones `(operation, Arc<Job>, handle_finished)` for every started entry, at most `MAX_JOBS = 128`, in operation-ID order from `closure_cursor`, wrapping. `handle_finished` is `JoinHandle::is_finished`, which never reverts. It then drops the lock.
2. **Classify.** Per job, checking `stopping` first, under `job.state` only:
   - `closed_ack` set: retry only the removal (step 8); this uses no Store and no turn;
   - derive `StartEnded` from this job alone: `outcome` is `Some(Err(_))`, or `observation` is `Uncertain` with `outcome` `None` and `handle_finished`; otherwise skip;
   - `closure_due > now`: skip;
   - otherwise the job is **due**: clone the custody, the retained plan and `uncertain`, then drop the lock.
3. **Turns.** A sweep grants at most `TURNS = 8` execution turns, one per due job, in snapshot order. If a due job finds no turn left, `closure_cursor` is set to its operation and the sweep stops; otherwise the cursor is cleared after the pass. Jobs that are not ended, not due or only awaiting removal take no turn. Every due job therefore gets a turn within ⌈128/8⌉ = 16 sweeps, whatever precedes it. A turn runs steps 4–7 until its first Store outcome.
4. **Proof** (only without a retained plan). Call `custody.nonsuccess_step(&ended)`.
   - `NotEligible` or `Held`: for `Held`, record bounded attention (an allowlisted category, ≤128 bytes); due now+5 s.
   - `PreparationPending`: due by backoff 100 ms–5 s.
   - `Proof`: continue.
5. **Plan.** `Store::plan_phase_nonsuccess_closure` on a query-only snapshot, without the Store mutex. Install it once under `job.state`, pointer-checked. A planning refusal is Held attention; due now+5 s.
6. **Material.** `Store::materialize_phase_nonsuccess(&plan)`, outside every lock (§7.5).
7. **Write or confirm**, under the Store mutex only:
   - `uncertain` unset: `close_phase_nonsuccess`. `Known` goes to step 8. `Conflict(cause)` is a definitive pre-write refusal: the plan is dropped, attention is recorded, due now+5 s. A later turn may plan again from the SAME proof, but plans derive only from the marker's original images, so drift is never laundered (no pin refresh). `Err` sets `uncertain`; due now.
   - `uncertain` set: `confirm_phase_nonsuccess`. `Known` goes to step 8. `RolledBack` clears `uncertain` and keeps the SAME plan and `at` for a bookkeeping write in a later turn. `Err` is Held with the plan retained; due now+5 s.
8. **Acknowledge** (§11): install `closed_ack`, then `retire_closed_marked`, then `retire_closed`.

**Plan lifetime.** A job retains at most one compact plan (§7.5). An uncertain plan lives until its own confirm is Known, until it is confirmed RolledBack and then refused definitively, or until the Runtime exits (Held across epochs). Its `at`, digests, Unit postimage and payload are never discarded or rebuilt while it lives. There is no Root-wide plan-credit cap, so uncertain plans of other jobs never take a due job's turn.

**Lock order.**

| Lock | While held, may take |
|---|---|
| `PhaseJobs.entries` | `job.state`, only in the existing `start()` (`phase_jobs.rs:234–244`); nothing in RN-1 |
| `job.state` | nothing |
| `PhaseJobs.closure_cursor` (new) | nothing |
| `PhaseSupervisor.queue` | the slot's `publication` and `marker` leaves, as the existing `remove_unmarked` (`phase_supervisor.rs:582–603`) |
| Store mutex | the permit manager (existing); custody `state` as the existing short leaf through `no_dispatch_matches` (`prepared.rs:10–21`, `preparation.rs:407–416`) and the proof's own check (§5) |
| custody `state` | only the existing helper leaves inside `abandon` |

RN-1 holds at most one of {`entries`, `job.state`, `closure_cursor`, `queue`, Store mutex} at a time. `StartEnded` is derived under `job.state` from the `handle_finished` value copied out of the snapshot. Removal reads the set-once `closed_ack` and `slot_released` under `job.state`, then takes `entries` alone (§11). No reverse edge is added: nothing under custody `state`, `job.state` or `queue` takes the Store, and nothing under `job.state` takes `entries`. No await, gate action or Drop of a removed Job, Slot, plan, material or acknowledgment happens under any lock. The validators' temporary custody upgrade is never the last strong reference: only this sweep thread removes a started job, after its Store call returns.

In debug builds, RN-1's lock guards increment a thread-local `ROOT_LOCK_DEPTH`; `nonsuccess_step`, materialization and the RN-1 Store ports `debug_assert!` that it is zero.

**Cancellation and shutdown.** The sweep checks `stopping` before each job. Retained proof, plan and acknowledgment live in `JobState` until consumed, independent of Engine or Driver futures. Shutdown runs no extra RN-1 sweep. Jobs that are closed but unacknowledged keep `ensure_*shutdown_complete` failing, as today.

## 7. The `phase_closed` transaction (P)

### 7.1 Plan, outside SharedStore

In one query-only snapshot, without the Store mutex:
1. **Original and owner.** `proof.validate_original()`. The selected owner must be pointer-equal (as `quota.rs:1237–1243`), and the allocation's `state_path`, `instance_id` and `epoch` must equal the owner's (as `successor.rs:212–217`).
2. **Currency.** The complete open-preimage conjunct set of §7.2 items 1–4, through the same code as the Immediate, so planning and writing cannot diverge. The Workflow image is the marker's retained `workflow_after`; with zero links it equals the current row byte for byte (`successor.rs:241–246`). No `CurrentWorkflowSuccessor` is built or retained, so `workflow_raw` keeps its `pub(super)` visibility.
3. **Unit preimage.** The closure's `LatestUnitImage` must decode and pass `registration_unit` (Preparing, Session None, both flags 1, work None, disposition Active, original identity) with `wait_reason ∈ {None, Quota, Capacity}`. Build the 13-column `unit_after` (§10). The identity predicate is that of `with_known_unit` (`successor.rs:48–57`), with a strictly increasing version.
4. **Operation and Workflow** (`plan_unlinked_closure`, inside `managed_binding`):
   - Operation: the marker's original insert image with `phase_open` 0 and `version` 2. Its body is the canonical re-encode of the original body object with exactly `phase_open:false` and `version:2` changed; every other key is byte-identical after decode. The body stays ≤4 MiB.
   - Workflow: decode the marker's `workflow_after` Record and `WorkflowSnapshot` with a complete typed roundtrip (as `binding.rs:244–250`). The active index must be the marker attempt.
   - Preconditions: state Running; `dispatch_started`; `session_id` and `execution` None; `unit == ManagedUnitRef::from(original)`; `completed_at`, `native_wait` and `next_due` None; no observations; `claimed_observations` 0.
   - Apply only: state Failed, `completed_at=at`, `detail=REASON_DETAIL`; Record `version+1`, `updated_at=at`.
   - Check `workflow::validate_transition(task_after, &after, Some(&before))`, and that the decoded before and after bodies, with exactly these fields neutralized, are canonically byte-equal.
   - Retain only the compact `UnlinkedPhaseClosure`: attempt index, versions, `at`, Workflow growth, and the SHA-256 digests of the canonical Workflow postimage (`WORKFLOW_DOMAIN`, equal to the payload's `workflow_body_sha256_after`) and of the operation post-body. Every parsed and encoded postimage is dropped before the plan is installed.
5. **Payload.** Build `audit_data` (§7.4). `at` is bookkeeping time and is not re-checked under the lock.

### 7.2 Immediate conjuncts (`close_phase_nonsuccess`), all before any write

The material (§7.5) is built before the Store mutex and supplies this attempt's exact operation and Workflow postimages. Then one `TransactionBehavior::Immediate` under `InventoryBudget`:
1. **Database and memory.** `selected_database(&self.connection, launch)` (`native_phase.rs:40–50`); `proof.validate_original()`; `launch.validate_preparation_original()` (SAME Source, ticket and allocation linkage).
2. **Open original preimage**, `validate_original_phase_tx(…, PhaseImage::Open)`:
   - the operation, input **and owner** rows equal their retained original insert images in every column, through the new `OriginalMarker::validate_unadvanced_tx` (the existing exact-row validator, `marker_rows.rs:59–77`). The owner is therefore still the unregistered version-1 image, and the operation is `phase_open=1`, version 1;
   - the original projection with the marker's `task_after` and `workflow_after`: runtime instance and epoch, P/G/T versions and bodies, the Workflow row, locks and Context (`snapshot.rs:349–374`);
   - the complete Unit index equals the closure's factual image (`unit_index_matches`);
   - ledger count 0 with no head;
   - the SAME retained Driver post anchor and Source result are live (`marker.validate_driver_live_tx`, `driver/marker.rs:135–154`), which rechecks the database path too.
3. **Closure facts.** `closure.validate_original(&tx)`: actor revoked, SAME custody no-dispatch, `no_registration`, inventory equal to the completion's `after`, Unit original identity.
4. **Closure postimages.** `closure.validate_after_tx(&tx)`: readiness equals the closure `closed` image; own waiter absent; own lease equal to the closure after-image, or absent. `closure.unit().validate_tx` (all 13 columns).
5. **Budget.** `charged_scope_bytes(scope) + workflow growth + audit bytes ≤ WORKFLOW_BYTES`, through `UnlinkedPhaseClosure::validate_budget_tx`.

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

Owned representations:

| Representation | Owner and lifetime | Bytes |
|---|---|---|
| Proof | custody and plan; shared Arcs | pointers only; actor, no-dispatch, closure plan and commit already exist (§2) |
| Marker preimages: `workflow_after` (raw, canonical and parsed), operation, input and owner images | the existing marker, for the slot lifetime | existing; RN-1 retains no copy |
| Compact plan: `unit_after` (13 columns), `audit_data`, two digests, scalars | `JobState.nonsuccess`, until acknowledgment or a definitive `Conflict` | ≤16 KiB body + ≤1 KiB columns + ≤4 KiB + 128 B + scalars: ≤24 KiB |
| Acknowledgment | `JobState.closed_ack` | one Arc |
| Material | one turn; dropped before the next turn | Workflow: one deep copy of the marker's parsed Record (P_W) plus its encoding ≤8 MiB; operation: one transient parse of the ≤4 MiB original body (P_O) plus its encoding ≤4 MiB |
| Mutations | one Store call | operation old/new ≤4+4 MiB; Workflow old/new ≤8+8 MiB; audit ≤8 KiB |
| Immediate reads | one Store call | projection Workflow row ≤8 MiB plus its expected copy ≤8 MiB (`snapshot.rs:368–373`); Unit ≤16 KiB; readiness ≤4 KiB; ledger `LIMIT 257` × ≤4 KiB; own waiter and lease rows |

- **Retained.** At most one compact plan per job, so ≤128 × 24 KiB = 3 MiB Root-wide, however many plans are uncertain. There is no plan-credit cap.
- **Peak.** The sweep is synchronous, so one turn's transients exist at a time: ≤3 MiB retained + P_W + P_O + 12 MiB encodings + 24 MiB mutation copies + ≤17 MiB Immediate reads + SQLite's own copies of bound parameters (≤24 MiB). A planning turn instead holds the typed before/after `WorkflowSnapshot` pair (≤2·P_W) and encodings ≤16 MiB, all dropped before the plan is installed.
- **Parsed trees.** P_W and P_O are parsed-tree sizes; this HOW asserts no numeric expansion factor. P_W is at most the tree the marker already retains for the same operation. Control M1 (§15) measures both.
- **Materialization.** Postimages are re-derived only from the SAME marker originals and plan scalars, outside every lock, and their digests must equal the plan's. No row is read. A mismatch is Held; the plan is neither rebuilt nor discarded.

| Item | Bound |
|---|---|
| One sweep | ≤128 jobs classified; ≤8 execution turns, each ≤1 preparation-closure step (≤3 Store transactions) or ≤1 RN-1 Immediate or confirm; removals use no Store |
| Immediate reads | Existing bounded validators only (above), plus `InventoryBudget` |
| Writes | 4 rows, 3 permits (≤128) |
| Ledger | 0 → 1 link (≤256); link ≤4096 bytes |
| Reservation | `phase_open` 1→0 releases `max(0, 1 MiB − spent)` (derived) |
| Workflow growth | `REASON_DETAIL` ≤128 UTF-8 bytes, plus `completed_at` and the state token, checked against `WORKFLOW_BYTES` |

`REASON_DETAIL` is the constant `"native start ended before dispatch; explicit retry required"`.

## 8. Uncertain commit (P)

`confirm_phase_nonsuccess(material)` runs one Immediate with no write. Its material is re-derived from the SAME plan outside the Store mutex (§7.5); nothing comes from current rows. Both classifications first require the complete original currency of §7.2: item 1; the input and owner images and Driver liveness of item 2; item 3; and the readiness, waiter and lease part of item 4.

- **Known** iff, in addition, the closed branch `validate_original_phase_tx(…, PhaseImage::Closed)` holds:
  - the operation equals the materialized postimage in all 32 columns, whose other 29 columns equal the original image;
  - the projection (runtime instance and epoch, P/G/T, locks, Context) passes with the materialized Workflow postimage;
  - the Unit equals `unit_after` in all 13 columns, and its index matches;
  - exactly one link exists: `phase_closed` with the marker scope, the plan's `at` and byte-equal `audit_data`, and any `sequence` > 0.

  Returns an acknowledgment for the SAME plan Arc.
- **RolledBack** iff all of §7.2 items 1–4 hold, including the open operation, Workflow and Unit preimages and zero links. This permits a bookkeeping retry of the SAME plan only; there is no new plan and no new `at`.
- **Anything else is Held** (`Err`): mixed images; a foreign or different link; a changed database, runtime instance or epoch, owner or input image, Source linkage, Driver, P/G/T, Context or lock; or a row equal to neither image. Only a read-only confirm per 5 s follows. Nothing is rebuilt from current rows. Link presence alone is never acknowledgment. A preimage is never success. A currency change after a committed but unconfirmed write therefore stays Held for TerminalRecovery.

`validate_current_tx` is not used: it requires the OPEN operation image and a successor.

## 9. Workflow non-success and retry visibility (P)

- **Durable state.**
  - `active = Some(i)` is kept, with state Failed, `completed_at` and `REASON_DETAIL`.
  - `session_id` and `execution` stay None; no binding is invented.
  - The Task row is unchanged, at its marker version and state: no `WaitingHuman`, no blocker and no Task-terminal success, failure or cancel (unlike `Engine::fail`, `workflow.rs:2571–2581`).
  - Context, artifact, Session and permission are unchanged or absent.
- **Derived status.**
  - The production Driver calls `step_driven_initial` (`task_driver.rs:104`). For an active Executor attempt that function would reach the Source handoff consumer, which reports Waiting whenever the retained handoff exists (`driven_initial.rs:36–40,141–163`). RN-1 adds one read-only branch before that consumer: an active Executor attempt in state Failed returns `StepResult::Failed { phase, reason: detail }` from the snapshot already read. It observes no handoff, offers nothing, writes nothing and grants nothing. G5 changes this function's signature but keeps the branch point.
  - The Driver loop exits only on `Finished` (`task_driver.rs:106–108`), so the SAME Driver stays `driving` and polls every 100 ms, read-only, with the constant `REASON_DETAIL`. This also keeps §7.2's Driver conjunct satisfiable. Generic `poll` also returns `Failed` read-only (`workflow.rs:1940–1945`).
  - `runtime/waiting.rs:35–60` stops reporting the operation because it filters `phase_open=1`.
  - **Occupancy is released only in part.** At commit the capacity union (`driver/claim.rs:337`) loses its open-Unit and open-operation members, but the Task stays occupied through its `driving` Driver row, which RN-1 does not change. RN-1 releases the operation reservation, the Unit flags, the audit reservation, the PhaseSupervisor slot and the PhaseJobs entry; the preparation closure already released the own lease, waiter and probe. The Source handoff entry stays retained (`phase_handoffs.rs:417–429`). Driver and handoff retirement need their own genuine typed producers (§17).
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
  3. `PhaseJobs::retire_closed(job)`. Under `job.state` alone, read `closed_ack` and `slot_released` (both set once and never cleared), require both, and drop the lock. Under `entries` alone, require the entry pointer-equal to the job with a finished handle. Remove the entry and drop it after unlocking.
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
| Execution turns exhausted | `closure_cursor` stays at this job; it is served first in the next sweep | none |
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
| Closure-plan accessors: `validate_original`, `unit`, `validate_after_tx` (`pub(super)`); `matches_no_dispatch` | `quota.rs`; `native_phase/nonsuccess.rs` | Read-only; no new constructor |
| `OriginalMarker::validate_unadvanced_tx`, `original_operation_image`; successor `rows`/`KINDS` `pub(super)`; new `managed_binding/closure.rs` | `marker_rows.rs:59–108`; `publication.rs:186–233`; `successor.rs:19,151–190` | Read-only; `validate_open_tx` and `validate_current_tx` unchanged |
| First `phase_closed` producer | `schema.rs:358–414`; `successor.rs:19` (KINDS) | After close, `validate_open_tx` fails, so every open-currency consumer refuses: binder, registration, quota, transport, version helpers, `plan_current_phase`. Intended |
| `phase_open=0` | `publication.rs:366` (reservation release); `waiting.rs:60`; `driver/claim.rs:337`; `quota_policy.rs:118`; `quota.rs:13`; admission trigger `schema.sql:195` | Status stops; the reservation and the operation member of the occupancy union free; the Task stays occupied by its `driving` Driver (§9); consumption impossible |
| Unit flags/state | `state/mod.rs:935,946`; `workflow.rs:3104–3157`; `cleanup.rs:82`; `execution.rs:325`; `verification.rs:452`; quota WAITERS | Retired and inert; no cleanup job |
| Workflow Failed attempt | `poll` (`workflow.rs:1940`); status/CLI; `validate_transition`; TerminalRecovery (`state/mod.rs:692,767,940`) | Read-only Failed; TerminalRecovery unchanged |
| PhaseJobs, PhaseSupervisor, service loop; `ClosedNonSuccess` | `service.rs:80,134–136`; `phase_jobs.rs:84–99` (`wait` treats it as terminal); handoff drops `PhaseInvocation` (`phase_supervisor.rs:915`) | Unmarked paths and `fair_page` unchanged |
| `StartEnded` re-export; `PhaseJobs::reconcile_nonsuccess`; `closure_cursor` | `runtime/mod.rs:7`; `phase_jobs.rs` | Crate-visible type with a phase_jobs-only constructor |
| Driven Failed branch | `driven_initial.rs:36–40` (G5: same branch, new signature); `task_driver.rs:97–115`; G5 `preflight_installed_native` (not reached for a Failed attempt) | Read-only `Failed`; no handoff observation or offer for a Failed active Executor |
| Constants: `TURNS` 8, snapshot bound `MAX_JOBS` 128 (existing), backoff 100 ms–5 s, `REASON_DETAIL` ≤128, payload ≤4096, ledger ≤256 | New, RN-1 only; trigger header list `schema.rs:368–396` | Header keys satisfied; projection keys free; no plan-credit cap |
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
   - the confirm classifier: every post/pre/mixed (Unit, operation, Workflow, link) combination, and each currency member of §8 changed alone in the post and in the pre branch;
   - the Unit delta identity predicate;
   - materialization determinism and its digest refusal;
   - the turn scheduler (§6 item 3): a due job behind persistent due jobs, and four uncertain plans plus a fifth due job, each served within ⌈n/8⌉ sweeps.
2. **Normal producer.** Every genuine row needs a genuine composed start that issued the S5 no-dispatch value and then ended `Err` before `retain_transport`. That requires G5 and the authentic composed Workflow activation producer (§17); RN-1 never synthesizes an activation row or relaxes `publication.rs:244–271`. Candidates, each to be shown reachable by the implementation evidence:
   - (a) S6: the quota commit returns `Err` and its own confirm returns RolledBack (`prepared.rs:145–147`), under genuine writer contention on the selected database;
   - (b) a refusal in `start_prepared_transport` before `retain_transport` (`transport.rs:503–562`), for example `qualified_physical_profile` (`version.rs:376–383`) after the selected program stops being a regular file while the start is parked at S6 behind genuine sibling Tasks.

   The replaced `native.rs:214` bail, FirstExecutor or account-free protocol-fixture outcomes, failpoints and test constructors are not normal producers. Until (a) or (b) is demonstrated, every row below is **UNVERIFIED/SETUP**; a SETUP refusal earns no positive or kill credit.
3. **Genuine causal pairs.** These require production Runtime → Driver → Source → Frame → marker → `PhaseJobs.start` → selected `NativeSessions`/actor/custody, with the actual protocol fixture configured as an external CLI (intermediate only; official Agents remain the merge gate). There is no test-only authority constructor, fake grant or prepared metadata. The `#[cfg(test)]` failpoints exist only in the RN-1 Store ports, inject `Err`, and construct nothing.

   | ID | Normal | Single changed condition | Required evidence |
   |---|---|---|---|
   | H1 evidence handoff | normal producer → sweep → W1–W4, acknowledgment, slot and job removed | genuine S4b refusal (no S5) | Held "pre-no-dispatch"; zero RN-1 rows; slot and job retained; shutdown check fails |
   | H2 | parked revocation → in-task closure → RN-1 | start not ended | no `StartEnded`; issuer not called. Needs an existing genuine revoker (G3): UNVERIFIED until then |
   | C1 exact currency | as H1 | a genuine Project, Goal, Task, Context or lock change through its existing writer, before the Immediate | `Conflict` with the projection cause; Held; all rows byte-identical |
   | C2 | as H1 | Driver genuinely no longer live | `Conflict` with the Driver cause; Held |
   | C3 | as H1 | Runtime restart (new epoch) | no custody, no proof; Held; no reconstruction. Positive Held evidence, not a kill |
   | L1 ledger/idempotence | Known, then another sweep | — | no second write; one link; operation version 2 |
   | L2 | commit failpoint returns `Err` after commit | — | confirm Known for the SAME plan; one link |
   | L2-x | as L2 | after the commit and before the confirm, one of: confirm on a Store of another database; a C1 change; a C2 change | confirm Held; no acknowledgment; slot and job retained |
   | L3 | failpoint before commit | — | RolledBack; SAME plan and `at`; later exactly one link |
   | L3-x | as L3 | one L2-x change | confirm Held, not RolledBack |
   | U1 permission retirement | `managed_attempt_retired` true; no cleanup job | Unit fenced first | Held; RN-1 wrote nothing |
   | A1 acknowledgment | slot and job removed; `ensure_*` pass | Store refusal (C1) | both retained; `ensure_*` fail |
   | W1 caller wiring | the service loop drives RN-1 | service call removed (mutant) | H1 fails: no link |
   | W2 visibility | the production Driver reports `Failed` with `REASON_DETAIL`; no write, handoff observation or offer while it polls; waiting None; Task unchanged | driven Failed branch removed (mutant) | the Driver reports the handoff Waiting reason |
   | O1 occupancy | after acknowledgment: no open Unit or operation for the Task; own lease inactive; slot and job removed | — | the capacity union still counts the Task through its `driving` Driver |
   | F1 fairness | ≥9 genuine due jobs (Held pre-S5 jobs or failpoint-held confirms) precede a closable genuine job | — | the closable job is acknowledged within ⌈n/8⌉ sweeps |
   | F2 | four jobs with failpoint-held confirms, then a fifth genuine job | — | the fifth is acknowledged; the four keep their SAME plans and `at` |
   | M1 memory | Workflow body near 8 MiB and operation body near 4 MiB; four retained uncertain plans | — | retained ≤24 KiB per plan; measured peak within §7.5 with P_W and P_O reported; no truncation or evidence loss |
   | R1 retry | `Engine::retry` on a closed protected Workflow | — | refused; zero writes |
   | Q1 lease | own lease released; a sibling genuine Unit is admitted | — | — |

4. **Causal omission map.** A kill needs the genuine setup to succeed and the named assertion to change. Compile refusals, SETUP failures and redundant defenses earn no kill credit; redundant defenses stay in the code.

   | Mutant | Effective predicate | Setup | Assertion that changes | Credit |
   |---|---|---|---|---|
   | Projection conjunct omitted | §7.2 item 2 projection | C1 | a link is written despite the change | kill |
   | Driver-live conjunct omitted | §7.2 item 2 Driver | C2 | a link is written | kill |
   | Closed-branch currency omitted | §8 Known | L2-x | an acknowledgment is issued | kill |
   | Preimage accepted as Known | §8 | L3 | acknowledgment while `phase_open=1`; slot removed | kill |
   | Plan rebuilt or new `at` after uncertainty | §6 item 7 | L3 | the link's `at` or digest differs from the SAME plan | kill |
   | Removal on a Failed observation or without an acknowledgment | §11 | A1 | slot or job removed | kill |
   | Service call removed | W1 | H1 | no link | kill |
   | Driven Failed branch removed | §9 | W2 | Waiting reported | kill |
   | Cursor advanced past an unserved due job | §6 item 3 | F1 | the closable job exceeds the bound | kill |
   | Task written, `session_id` set, detail from error text, flags left open, or `phase_open` unchanged | §7.3, §10 images | H1 | a durable image differs from §7.3/§10 | kill |
   | Audit inserted before W2/W3 | chain trigger | H1 | the trigger refuses; no link | kill (trigger) |
   | Root lock held across a Store call | §6 lock order | H1 | the `ROOT_LOCK_DEPTH` debug assertion fires | kill (debug) |
   | `retain_transport` keeps `no_dispatch` and the issuer omits the transport check | issuer | a genuine start error after `retain_transport` | a proof is issued for a transported operation | kill (pair); UNVERIFIED until that error is shown |
   | Transport check omitted alone | redundant with C-R1 | — | — | defense only |
   | Epoch or instance check omitted alone | overlaps snapshot, projection and Driver epoch | — | — | defense only |
   | Owner exact image omitted | §7.2 item 2 owner | no genuine pre-transport owner writer exists; permits refuse others | — | defense only; required by §1.1 |
   | Ledger-zero conjunct omitted | §7.2 item 2 ledger | no pre-transport link producer exists; the trigger refuses a second `phase_closed` | — | defense only |
   | `StartEnded` of another allocation | issuer | the sweep derives it from the same job | — | defense only |
   | Issuer without `closed`; plan without proof; `remove_unmarked` for a marked slot | non-`Option` fields; `remove_unmarked` refuses `Publishing` | — | — | compile or existing refusal; none |

## 16. Corrections to approved documents (minimum, evidence-backed)

| ID | Correction | Evidence (V) |
|---|---|---|
| C-R1 | Prepared HOW §5 item 3: `retain_transport` must clear `no_dispatch`, and `closure_original` plus the RN-1 issuer must require `transport.is_none()` | `preparation.rs:51–72,88–110` |
| C-R2 | Prepared HOW §7.4 item 2, "offers the SAME no-dispatch value to Root": the handoff is pull-based. Root obtains the proof from the SAME custody after the start ended; nothing travels through `NativePhaseStartError` | `phase_jobs.rs:270–292`; `execution/native.rs:90–93` |
| C-R3 | Managed binding §6, "Engine::fail is non-success closure/awaiting_explicit_retry": for this case no Task `WaitingHuman`/blocker is written. Awaiting explicit retry means an active Failed attempt plus `phase_open=0`; retry is RN-R | `workflow.rs:2571–2581`; `schema.rs:328` |
| C-R4 | Prepared HOW §6.3 Closure row, "Unit retirement belongs to the trusted cancel/fence owner", is extended: in the RN-1 case (no fence, unchanged parents) RN-1 is that trusted producer. The nongrant preparation closure still never writes the Unit | `execution.rs:323–356`; `quota.rs:1304–1364` |

## 17. Unresolved dependencies and open gates

- **G5** composition and its pending HOW correction. Component `c51af44` compiles for all targets; its C5/C6 controls are SETUP before the marker. Until G5 lands and a §15 item 2 normal producer is demonstrated, every genuine control is SETUP/UNVERIFIED.
- **Composed Workflow activation producer** (Root, separate design). `plan_marker_publication` requires a `composed` version-1 `workflow_native_contracts` row matching the allocation scope, epoch, origin and profile (`publication.rs:244–271`); production has only the `legacy_held` migration insert (`schema.rs:519–533`). RN-1 depends on that genuine path and never synthesizes activation rows or removes the predicate.
- **G3** Root `request_stop` caller; Root SourceStop/Cancel author/permission decisions; the R2 shared Git environment author exception. None is decided or widened here. RN-1 mints no stop authorization.
- **RN-1b:** a non-success proof for errors before S5 (pre-helper refusal, settled-helper refusal at S4/S4b/S5). Until it exists, these stay Held.
- **RN-R:** explicit protected retry, including fresh worktree/Unit/generation, `task_execution` advance, readonly-generation retirement and namespace cleanup of the closed Unit.
- TerminalRecovery / `terminal_decision` for fenced or drifted operations; across-epoch restore; transport closure for S8 onward.
- No final RN-1 sweep at shutdown.
- Driver and Source handoff retirement after RN-1. The Task stays occupied by its `driving` Driver, and the handoff entry keeps `ensure_shutdown_complete` failing, until their own genuine typed producers exist.
- Regression and Clippy RED. N1/N4/H/Q/install/both-OS/dogfood qualification. The Official-Agent/four-Task/both-OS MERGE gate.
- Diff of §2 against the unread reconnaissance report.

## 18. Review plan

- **This HOW:** ONE independent Sol 6.1 (`gpt-6.1-sol`) high design review. No Grok, Fable or Astra. Re-review covers only the delta of confirmed findings, by the finding side.
- **Source increment:** implemented by Sol 6.1 high. The initial review runs three roles in parallel at high: Sol on the implementation diff, Opus on the security/contract diff, Grok on reconciliation only. The fix is made by the side that did not raise the finding.
- **Master design:** updated only by the implementation PR, with implemented facts only (`master/agent-execution.md`, `master/workflow-engine.md`). This HOW updates no master.
