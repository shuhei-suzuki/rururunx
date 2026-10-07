# Issue 43: PR1 successful original Native phase continuation (SC) authority/contract HOW

## 1. Status, pins and categories

1. **What this document is.** This is a proposed authority/contract HOW for prerequisite PR1: the whole successful continuation of the ORIGINAL first-Executor native phase. It is not a requirements change. The governing WHAT is the approved [managed binding requirements](../requirements/issue-43-managed-binding-requirements.md): MB7–MB9 and MB-AC1, AC4, AC4.a and AC5–AC8. All of their acceptance conditions stay unchanged.
2. **Documents it refines.** It refines, and does not replace:
   - the [managed binding design](issue-43-managed-binding-design.md) §§2.2, 4–7, 8 (writer inventory), 9 (finite budgets) and 10 (sequence);
   - the [prepared producer HOW](issue-43-native-prepared-producer-design.md) §9.3;
   - the [composed activation HOW (CA)](issue-43-native-composed-activation-design.md);
   - the [non-success closure HOW (RN1)](issue-43-native-nonsuccess-closure-design.md).

   It claims nothing is implemented. It updates no master design, README, license or global rule. Facts in master designs are updated only by the future implementing PR.
3. **Immutable pins.** Paths are relative to `crates/rrx/src/`. A line reference without a prefix is at **R**.

   | Label | Exact Git object | Use |
   | --- | --- | --- |
   | R | `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` | This worktree's clean tree: the frozen Root integration source |
   | CA | `95ef0e783c0b18c90dfef23ba1b77b9dc67735f1` | Composed-activation source, read with `git show` only |
   | RN | `1d783440b7f4d705bf8cf4c667e022f5cec87777` | Final RN1 source, read with `git show` only |

   CA and RN are independent descendants of R (`git merge-base` of all three is R). They are not composed, tested or qualified together.
4. **Provenance update for the inline consumer map.** That map cited RN at `e32b378e…`. Its diff to RN-final `1d78344` touches exactly four files:
   - `execution/native/nonsuccess.rs`: `issue` now returns `Result` and validates the original actor, the no-dispatch value and the closure;
   - `execution/native/preparation.rs`: a `closure.is_none()` conjunct is added;
   - `state/execution/native_phase/nonsuccess.rs`: a full Unit CAS is added, and a selected-database refusal is now a `Conflict`;
   - `state/managed_binding/closure.rs`: the `attempt()` accessor is used.

   None of these touch a success API. Every RN reference below is at `1d78344`.
5. **Contract objects re-hashed for this HOW (V, SHA-256 of the complete files).** All hashes equal the inventory's values.
   - R managed-binding requirements: `11ec2b92…7297387`
   - R managed-binding design: `f03efbb1…dd4d00`
   - R prepared producer: `02d30312…39a326`
   - CA HOW (object `b45c2367…:doc/design/issue-43-native-composed-activation-design.md`): `b9129c0b…7f96`
   - RN1 HOW (object `9d20ee8d…:doc/design/issue-43-native-nonsuccess-closure-design.md`): `00cc4c53…bcca5cb`
6. **Categories.**
   - **V (verified):** read in source at the stated pin. No build, test, lint, Agent, fixture or OS run was executed for this HOW.
   - **P (proposed):** this contract; not implemented.
   - **A (acknowledged):** reported by Root and relied on without re-verification.
7. **Never authority here.** None of the following ever grants authority:
   - deleting a guard;
   - binding by Task.version or refreshing pins;
   - a public, row-derived, copied or serializable grant (`ExecutionAuthority`, `ManagedSessionRef`, `NativeStatus`, `NativeResultReceipt`, Session IDs, receipt IDs);
   - a fake capability, a preflight, a fabricated accepted row or control setup;
   - a test-only constructor, or SQL seeded to make a positive pass;
   - an UPDATE of immutable history;
   - inferring death or replay from an observation timeout;
   - a SourceStop, Cancel or R2 human-decision bypass.

   rrx is not a security sandbox. Under the latest user direction, official CLIs run on the HOST with unchanged HOME, authentication, settings and hooks; no VM is used; cleanup is best effort; and there is no ownership-proof launch barrier. Optional historical Strong PR79 is not a PR1 or MVP prerequisite. Native authentication, settings, home, hook and token stores are never read. Rust `unsafe` stays prohibited.
8. **Composition prerequisite (P).** Implementation starts only from a reviewed integration commit that contains R, CA (marker reachability requires the composed contract) and RN-final (PR1 must coexist with its sweep and closure). Producing and reviewing that composition is a separate step, not part of this HOW.

### 1.1 One-line acceptance condition (P)

For the SAME original operation, a successful terminal (`OwnedPhaseSettlement`) is factually bound exactly once, by normal return or late reconciliation. The SAME Driver then captures an independent real commit/base graph, writes a factual `gate_claim` and a fused `gate_observed`, and for a Passed gate commits ONE typed successful `phase_closed` Immediate. That Immediate atomically advances exactly the enumerated Task, Workflow, Context, artifact, Unit, operation and Driver rows, and it leaves the Driver current and marker-free for the next lane. Every other outcome stays Waiting or Held, with no success credit.

### 1.2 Why the existing WHAT is sufficient

| Needed PR1 behavior | Existing requirement that already mandates it |
| --- | --- |
| Normal and late binding through the same binder; only a sealed genuine settlement proof enables late; NotDispatched cannot late-bind | MB3, MB7, MB-AC4.a; HOW §§4 and 6 |
| Deferred or uncertain commit; dropped futures; lost notifications; one exact link | MB7, MB8, MB-AC4; HOW §5.3 |
| Typed `gate_claim`, `gate_observed` (Waiting/held), `gate_hold` and `phase_closed` with exact projections | MB9; HOW §7 table |
| Source7 and Driver recognize only an original-anchored successor; closure advances pins only through its own typed transaction | MB9; HOW §7 lines 581–604 |
| Finite budgets, mandatory closure reserve, no work under Store | MB9; HOW §9 |
| Missing production evidence waits; transport success is not acceptance | MB1, MB7 ("terminal receipt does not certify them"); production gates design |

Remaining gaps that stay unmet (§18), not redefined: the diagnostic port, bound non-success closure, restore, Reviewer lanes and full-MVP gates. **Gap I name in this HOW:** prepared-producer §9.3 words PR-1 as "Driver continuation after initial Evidence". Under Root's current definition, PR1 delivers the first Executor's own successful continuation. Execution of later Evidence phases (Commit, Tests, Pr, …) after closure stays a separate named lane, **SC-N** (§18). Until SC-N exists, PR1 replaces the worker-ending error with a read-only Waiting only for the post-success-closure state.

## 2. Verified source map

| Item | Fact (V unless marked) |
| --- | --- |
| First Executor per class | `phases()` (`workflow.rs:153–188`): QUICK = `[Worktree, Implement, Commit, Tests, ImplementationReview, Pr]`; STANDARD/STRICT start `[Issue, Worktree, Requirements, …]`. Actor map `:103–114`: Requirements, Design, Implement and ImpactAnalysis are Executor phases. `preflight_installed_native` (`workflow/driven_initial.rs:11–51`) admits only the first configured Executor at generation 1. Design and ImpactAnalysis are never a first Executor. |
| CA control class | The R `plan()` Task is QUICK (`runtime/tests.rs:34–48`). CA's fixture sets minimum QUICK, risk mapping QUICK and reviewers `rev-a`/`rev-b` (CA diff to `runtime/installation/tests.rs`). The CA first Executor is therefore **Implement**. Accepted Goals take `max(task, minimum, risk_mapping)` (`runtime/goal.rs:205–208`), so default policy yields **Requirements**. |
| Fixture modes | CA's fixture injects `WORKFLOW_SCENARIO='answer-normal'`. For codex this emits an `APPROVE` item with no commit (`native_fixture.py:83–117`). Without a scenario, `complete()` waits for `fixture-release`, writes `fixture-result.txt`, commits with `/usr/bin/git`, then emits correlated completion (`:21–44,130–134,205–207`). |
| Root job custody | `JobState` retains the custody, observation, launch, `outcome: RetainedStart::Launched{_handle, binding: Arc<NativePhaseBinding>}`, `binding_plan` and `binding_error` (`runtime/phase_jobs.rs:33–49`). The start task records the outcome, then calls `bind_returned` once; failure leaves `BindingHeld` with no retry (`:270–309,364–382`). |
| Settlement issuer | `PhaseActor::settled` (`execution/native/phase_protocol.rs:257–301`) is called only from `persist_saved_terminal` when `owned_success` (`execution/native.rs:1257–1270`). `OwnedPhaseSettlement::completed` (`:553–641`, `pub(super)`) checks the exact receipt, WorkKnown+Success, closed effects, the Exited Session, the allocation and the real ACK. It then `revoke`s the owner (`:619`) and projects a Weak. `PhaseActor.retained.settlement` holds it strongly (`:91–96`). |
| Post-settlement read path | `NativePhaseSession::binding_snapshot` (`:416–429`, `pub(crate)`) upgrades the projection's consumed and settlement Weaks into a new `NativePhaseBinding`. The Root job already retains `binding.owner_arc()` (`:432–434`). `NativeSessions::release` refuses phase entries (`execution/native.rs:700–703`), so the strong settlement stays in the registry Entry (`:102–109`). |
| Normal binder | `normal_eligibility` (`state/managed_binding/binding.rs:158–181`) requires live, no settlement, open native effects, no work and the Unit in DispatchPending/Running/WaitingQuota. Its SQL also requires a non-closed invocation (`:148,207`) and a live Session state (`:91–97`). For an existing link, `plan_managed_binding` accepts only the exact first `normal_return` tuple (`:331–358`). The header states that closed-settlement reconciliation is separate (`:1–3`). **No late binder exists in R, CA or RN.** |
| Terminal postimages | For an owned Completed success, the Unit becomes WorkKnown/Success with `native_effects_open=false`; `result_finalization_open` **stays true** (`state/execution/native_phase/terminal.rs:472–510`). Readiness becomes `closed`/`start_ended`/`known_terminal` (`:563–571`). Admission becomes `settled = confirmed ∧ input effect Confirmed` (`:585–593`). After that commit, Core calls `record_execution_cleanup` (`execution/native.rs:1874`), so the Unit cleanup axis can advance after settlement. |
| Successor reader | `plan_current_phase` and `validate_current_tx` (`state/managed_binding/successor.rs:206–377`) require the operation open, the exact marker Task, the P/G/T, Workflow, lock and Context projection (`state/managed_binding/snapshot.rs:349–400`), the exact Unit index and the ledger head. Every link must carry the original `context_version` (`successor.rs:270`). |
| Ledger triggers | `state/managed_binding/schema.rs:358–416` enforces: per-operation allowances (`session_bound` 1, `native_diagnostic` 47, `gate_claim` 99, `gate_observed` 99, `gate_hold` 8, `terminal_decision` 1, `phase_closed` 1) (`:397`); `context_version` equals the operation's (`:404`); `phase_closed` iff `phase_open=0` (`:408`); the Workflow version equals `workflow_version_after` (`:411`); `gate_observed` immediately follows a `gate_claim` (`:413`). |
| Record and permit guards | Contracted Workflow and Session records need an exact `records` permit (`schema.rs:317–337`). The permit catalogue covers contracts, operations, marker bodies, owners, inputs, admissions, readiness, `task_drivers`, `source_recoveries`, `records` and `audit` (`state/managed_binding/permits.rs:23–180`). Tasks, Context, result artifacts and Units are **not** permit tables. |
| Generic refusals | Generic finalize authority `validate_authority(…, finalize)` calls the generic Driver `validate` (`state/execution.rs:458–497`), which refuses `row.marker.is_some()` (`state/runtime/driver.rs:339–342`). Generic Source7 `validate_row` refuses marker anchors (`state/execution/source_recovery.rs:296–308`). Generic `put_workflow_transition_inner` calls the Driver `validate`, Source `before_write`/`after_write`, `put_task_tx` and `put_record_tx` (`state/mod.rs:697–699,1078–1099`). |
| Capture | `ManagedWorkflowSources::frame` (`execution/workflow_source.rs:760–921`) captures for a successful Executor Unit with open finalization (`:839–863`), via `UnitGit` (`execution/git_io.rs:135–241`, whose helpers call `validate_execution(…, false, true)` at `:169`) and `ResultStore::capture` (`execution/results.rs:195–370`). Capture checks base ancestry and the exact commit, stages the artifact, fetches commit and base into `results.git` with fsck, writes a durable manifest, then marks it Ready. Capture helpers write only `managed_effects` rows (`state/execution/effects.rs:41–89`), not the Unit. `committed_input` reaches `frame()` (`workflow_source.rs:939–956`). |
| Publication | `WorkflowPublication` has a private constructor; its only issuer is `ResultStore::workflow_publication` (`results.rs:387–432`), which uses generic `validate_execution`. `publish_workflow_result_tx` (`state/execution/artifacts.rs:392–481`) calls `validate_authority`, publishes the artifact and writes the Unit. Historical `RetainedGit` inspection needs only epoch and artifact currency (`artifacts.rs:5–40`). |
| Gates | `ManagedWorkflowGates::evaluate` (`execution/workflow_gates.rs:203–410`) supports Issue/Worktree, Implement (terminal plus Ready artifact, `:262–287`) and the Commit family (`:288–322`). Every other phase, including Requirements, returns `Waiting("<phase> requires its qualified production evidence integration")` (`:323–328`). Implement's `terminal` uses a `SessionStatus` DTO and generic `validate_execution` (`:157–202`). Passed writes a `Verification` record through `put_record` (`:378–394`); that kind is not record-guarded. |
| Generic apply | `apply_outcome(Waiting)` writes Task `WaitingHuman` and a blocker (`workflow.rs:2927–2935`). The `Passed` branch (`:2938–3090`) sets Succeeded, completed evidence, sources, a new Context (`set_context` `:3892–3898`), Task phase, state and artifacts, and builds the publication. |
| Driver step | `step_driven_initial` returns RN's read-only Failed branch first (RN `workflow/driven_initial.rs:79–87`). It then observes the Source handoff as Waiting (`:170–202`). For other non-initial phases it bails (`:96`), and `drive` propagates the error and ends the worker (`runtime/task_driver.rs:119–137`). |
| Driver row | `Row.pins` covers Project, Goal, Task, Workflow, Context, generation, prerequisites, the **preparation Unit pin (version+digest)** and the Source pin, plus `marker` (`state/runtime/driver.rs:61–69,98–255`). `plan_marker_advance` updates the Task, Workflow and Source pins and sets the marker but keeps the pre-marker preparation pin (`state/runtime/driver/marker.rs:67–123`). `DriverAssociation::publish_exact` is idempotent for the planned postimage and requires a live worker (`runtime/driver.rs:271–314`). |
| Root lookup path | `PhaseHandoffs.entries[task]` reaches `Handoff.assets.origin: Arc<PhasePreparationOrigin>{allocation, ticket: Weak}` (`runtime/phase_handoffs.rs:41–56,112–127`). PhaseJobs entries are keyed by operation and pointer-checked by allocation (`phase_jobs.rs:105–160`). |
| Budget | `charged_scope_bytes` charges every scoped body and `max(0, 1 MiB − spent)` for each open operation, against 128 MiB `WORKFLOW_BYTES` (`state/managed_binding/publication.rs:304–375`). |
| RN reuse points | RN `state/managed_binding/closure.rs:115–141` (`operation_delta`: exact 32 columns, phase_open 1→0, version 2), `:278–` (`write_tx` sealed by `NonSuccessReader`). RN `successor.rs:23–62` (`ClosureLedger`). RN `phase_jobs.rs:304` (`reconcile_nonsuccess`, ≤8 turns, ≤1 Store transaction per turn). RN `phase_supervisor.rs:629` (`retire_closed_marked`). |

## 3. Case matrix (P)

### 3.1 First-Executor phases

| First Executor | PR1 result | Success credit |
| --- | --- | --- |
| **Implement** (QUICK; the CA control) | Bind, capture, claim, Implement gate. Passed → `gate_observed` Passed, then success `phase_closed`. | Yes, only through the full chain |
| **Requirements** (STANDARD/STRICT default) | Bind, capture (Ready artifact only), claim, gate → `Waiting("requirements requires its qualified production evidence integration")` → fused `gate_observed` Waiting. The phase stays open. The Driver reports Waiting read-only. There is no re-claim, no closure, no RequirementsCommit and no Task rewrite. | **None.** Missing producer: a qualified Requirements (and later Design) evidence integration that issues Passed evidence binding the captured artifact, plus its explicit gate-resume lane (§18). |
| Design, ImpactAnalysis | Never a first Executor (`preflight_installed_native`). Later-Executor offers are PR-2. | n/a |

### 3.2 Terminal and binding cases

| Case | Result |
| --- | --- |
| Owned success after normal Bound | Continuation (§6) |
| Owned success before normal binding (lost or late return) | Normal refuses; Root late-binds once (§5); then continuation |
| Normal binding uncertain commit | Confirm the SAME retained plan (§5.3); never replan before RolledBack |
| NotDispatched or pre-ACK owner, no settlement | Normal binding only; late planning refuses (no settlement) |
| Known failure, Unknown work, HistoricalDraft, non-owned receipt, capacity/quota interruption after Bound | No settlement exists. Held: "bound non-success closure unavailable" (§18). RN1 does not apply (it needs unbound, zero links) |
| Cleanup Unknown/Leftovers | Neither blocks nor authorizes (cleanup-axis Unit drift tolerated, §7) |
| Parent, Context, lock, Workflow or Source drift; stale epoch | Conflict, then Held. No pin refresh or laundering |
| Restart (new epoch) | Custody gone; Held. RestoreFactualBinding is absent; nothing is inferred from timeouts |
| Source7 row present at the marker (recovered source) | Binding and gates follow the existing contract; **success closure is Held** ("Source7-anchored success closure unsupported"). Unreachable for a fresh generation-1 first Executor (Source7 needs a Published source artifact, `source_recovery.rs:209–222`) |
| Gate Failed outcome, or gate error/unknown | Fused `gate_observed` with a held disposition. No closure (§9) |
| SourceStop, Cancel, R2 decisions | Not decided here. A genuine fence or parent change is drift, so Held |

## 4. Types, ownership and API

All new types are crate-private, non-Clone, not Serialize/Deserialize, have private fields, and have no row/ID/DTO constructor. "Existing" means present at R (V). "Proposed" means P.

```rust
// ---- state/managed_binding/binding.rs (extended) ----
// Existing, unchanged predicate: plan_managed_binding(owner, Arc<NativePhaseBinding>) (normal).
// Proposed: the plan records which private predicate built it (not caller-selectable).
enum BindingKind { Normal, Late }                       // private field of ManagedBindingPlan
pub(crate) fn plan_late_binding(owner: &RuntimeOwner, proof: Arc<NativePhaseBinding>)
    -> Result<ManagedBindingPlan>;                       // requires proof.settlement().is_some() (§5.1)
pub(crate) struct BindingAcknowledgment { plan: Arc<ManagedBindingPlan> } // built only here
pub(crate) enum ManagedBindingWrite { Known(Arc<BindingAcknowledgment>), Conflict(anyhow::Error) } // Err = uncertain
pub(crate) enum ManagedBindingConfirmation { Known(Arc<BindingAcknowledgment>), RolledBack }        // Err = Held
impl Store {
    // Signature change; the only caller is runtime::phase_jobs.
    pub(crate) fn bind_managed_phase(&mut self, plan: &Arc<ManagedBindingPlan>) -> Result<ManagedBindingWrite>;
    pub(crate) fn confirm_managed_binding(&mut self, plan: &Arc<ManagedBindingPlan>)
        -> Result<ManagedBindingConfirmation>;
}
impl BindingAcknowledgment {
    pub(crate) fn matches_owner(&self, owner: &NativePhaseSession) -> bool; // ptr
    pub(crate) fn marker(&self) -> &OriginalMarker;
}

// ---- state/managed_binding/success.rs (new) ----
pub(crate) struct SettledPhase {
    binding: Arc<NativePhaseBinding>,        // snapshot whose settlement is Some (strong)
    bound: Arc<BindingAcknowledgment>,       // SAME owner, committed session_bound
}
impl SettledPhase {
    /// Inputs are unforgeable: the binding comes only from NativePhaseSession::binding_snapshot,
    /// the acknowledgment only from the binder. Pointer and identity checks only.
    pub(crate) fn issue(binding: Arc<NativePhaseBinding>, bound: Arc<BindingAcknowledgment>)
        -> Result<Arc<Self>>;
    pub(crate) fn marker(&self) -> &OriginalMarker;
    pub(crate) fn settlement(&self) -> &OwnedPhaseSettlement;
    pub(crate) fn allocation(&self) -> &Arc<NativeAllocation>;
    pub(crate) fn phase(&self) -> Phase;     // the marker's active attempt
}
/// Query-only planned currency; nongrant.
pub(crate) struct SettledCurrency { settled: Arc<SettledPhase>, current: CurrentWorkflowSuccessor }
pub(crate) fn plan_settled_currency(owner: &RuntimeOwner, settled: &Arc<SettledPhase>)
    -> Result<SettledCurrency>;
pub(in crate::state) fn validate_settled_tx(tx: &Transaction<'_>, c: &SettledCurrency)
    -> Result<()>;                               // §7: exact recheck, no decode or hash
pub(crate) struct GateClaimPlan { /* compact: digests, at, attempt, headroom; Arc<SettledPhase> */ }
pub(crate) struct GateClaimAcknowledgment { plan: Arc<GateClaimPlan> }
pub(crate) struct GateObservedPlan { /* claim ack, retained outcome, observation ≤64 KiB, digests */ }
pub(crate) struct GateObservedAcknowledgment { plan: Arc<GateObservedPlan> }
pub(crate) struct SuccessClosurePlan { /* §10.1 */ }
pub(crate) struct SuccessClosureMaterial { /* one turn; borrowed, freed after the Store guard */ }
pub(crate) struct SuccessClosureAcknowledgment { plan: Arc<SuccessClosurePlan> }
pub(crate) enum SuccessWrite<A> { Known(Arc<A>), Conflict(anyhow::Error) }   // Err = uncertain
pub(crate) enum SuccessConfirmation<A> { Known(Arc<A>), RolledBack }         // Err = Held
impl Store {
    pub(crate) fn plan_settled_gate_claim(owner: &RuntimeOwner, c: SettledCurrency, at: i64)
        -> Result<Arc<GateClaimPlan>>;
    pub(crate) fn claim_settled_gate(&mut self, p: &Arc<GateClaimPlan>) -> Result<SuccessWrite<GateClaimAcknowledgment>>;
    pub(crate) fn confirm_settled_gate_claim(&mut self, p: &Arc<GateClaimPlan>)
        -> Result<SuccessConfirmation<GateClaimAcknowledgment>>;
    pub(crate) fn plan_settled_gate_observed(owner: &RuntimeOwner, claim: &Arc<GateClaimAcknowledgment>,
        outcome: SettledGateCompletion, at: i64) -> Result<Arc<GateObservedPlan>>;
    pub(crate) fn observe_settled_gate(&mut self, p: &Arc<GateObservedPlan>)
        -> Result<SuccessWrite<GateObservedAcknowledgment>>;
    pub(crate) fn confirm_settled_gate_observed(&mut self, p: &Arc<GateObservedPlan>)
        -> Result<SuccessConfirmation<GateObservedAcknowledgment>>;
    pub(crate) fn plan_phase_success(owner: &RuntimeOwner, observed: &Arc<GateObservedAcknowledgment>,
        publication: WorkflowPublication, context: ContextVersion, at: i64) -> Result<Arc<SuccessClosurePlan>>;
    pub(crate) fn materialize_phase_success(p: &Arc<SuccessClosurePlan>) -> Result<SuccessClosureMaterial>;
    pub(crate) fn close_phase_success(&mut self, m: &SuccessClosureMaterial)
        -> Result<SuccessWrite<SuccessClosureAcknowledgment>>;
    pub(crate) fn confirm_phase_success(&mut self, m: &SuccessClosureMaterial)
        -> Result<SuccessConfirmation<SuccessClosureAcknowledgment>>;
    /// Exact committed-row check plus the SAME association's publish_exact (mirrors publish_driver_marker).
    pub(crate) fn publish_success_driver(&mut self, ack: &Arc<SuccessClosureAcknowledgment>) -> Result<()>;
    pub(crate) fn reserve_settled_helper(&mut self, c: &SettledCurrency, id: OperationId, path: &Path)
        -> Result<()>;                            // kind "git_helper", native=false only
    pub(crate) fn stage_settled_result(&mut self, c: &SettledCurrency, a: &ResultArtifact) -> Result<()>;
}

// ---- state/runtime/driver/closure.rs (new; sibling of marker.rs) ----
pub(crate) struct DriverClosureAdvance { /* old = SAME DriverMarkerAdvance post row; new = marker-free row */ }
impl DriverMarkerAdvance {               // existing type
    pub(crate) fn plan_success_closure(&self, task_after: &Task, workflow_after: &Record,
        context_after: &ContextVersion, unit_after: &ExecutionUnit) -> Result<DriverClosureAdvance>;
}

// ---- execution/results.rs, git_io.rs, workflow_source.rs, workflow_gates.rs (extended) ----
impl ResultStore {
    pub(crate) async fn capture_settled(&self, s: &Arc<SettledPhase>, revision: &str,
        sources: BTreeMap<String, String>) -> Result<ResultArtifact>;
    /// The sole sibling issuer of the existing private WorkflowPublication.
    pub(crate) async fn settled_publication(&self, s: &Arc<SettledPhase>, artifact: ArtifactId)
        -> Result<WorkflowPublication>;
}
impl UnitGit { pub(crate) fn for_settled(owner: Arc<RuntimeOwner>, s: Arc<SettledPhase>) -> Result<Self>; }
impl ManagedWorkflowSources {
    pub(crate) async fn capture_settled(&self, s: &Arc<SettledPhase>, project: &Project, task: &Task)
        -> Result<Arc<Frame>>;
    pub(crate) async fn retire_closed_handoff(&self, task: TaskId, ack: &ClosedPhaseAck) -> Result<()>;
}
pub(crate) struct SettledGateCompletion { outcome: GateOutcome, receipt: Option<Record>, claim: Arc<GateClaimAcknowledgment> }
impl ManagedWorkflowGates {
    pub(crate) async fn evaluate_settled(&self, s: &Arc<SettledPhase>, claim: &Arc<GateClaimAcknowledgment>,
        invocation: PhaseInvocation) -> Result<SettledGateCompletion>;
}

// ---- runtime (extended) ----
pub(crate) struct SuccessContinuation {                     // runtime::phase_jobs; held in JobState
    settled: Arc<SettledPhase>,
    stage: Mutex<SuccessStage>,                               // leaf mutex, single-flight token
}
pub(crate) enum ClosedPhaseAck { NonSuccess(Arc<PhaseClosedAcknowledgment>), Success(Arc<SuccessClosureAcknowledgment>) }
pub(crate) enum SettledLookup { NoHandoff, Pending, Held(&'static str), Settled(Arc<SuccessContinuation>), Closed(ClosedPhaseAck) }
impl Runtime {
    pub(crate) fn settled_phase(&self, task: TaskId, association: &DriverAssociation) -> Result<SettledLookup>;
}
impl PhaseJobs { pub(super) fn reconcile_success(&self, phases: &PhaseSupervisor, stopping: &AtomicBool) -> Result<bool>; }
impl PhaseHandoffs { pub(super) fn retire_closed(&self, task: TaskId, ack: &ClosedPhaseAck) -> Result<()>; }
// RN's retire_closed_marked / retire_closed accept ClosedPhaseAck (generalized; RN behavior unchanged).
pub(crate) enum InvocationObservation { /* existing */ ClosedSuccess }
```

**Existing APIs consumed unchanged (V):**
- `NativePhaseSession::binding_snapshot`, `is_live` and `validate_known_registration`;
- the accessors of `NativePhaseBinding` and `OwnedPhaseSettlement`, and `belongs_to`;
- `ConsumedPhaseInput::{belongs_to, effect, frame_sha256, acknowledgement}`;
- `plan_current_phase` and `validate_current_tx`;
- `OriginalMarker::validate_driver_live_tx`;
- `ExactRowMutation` and `PrivatePermitManager::with_exact_permit`;
- `charged_scope_bytes`;
- `ResultStore::verify` (historical `RetainedGit`) and `ready_result`;
- `DriverAssociation::{same_association, publish_exact}`;
- `prepare_pack`, `validate_transition`, `validate_context` and `validate_evidence`.

No existing private constructor is widened. `OwnedPhaseSettlement::completed` stays `pub(super)`, and `WorkflowPublication`'s fields stay private.

**Shared pure refactors (P; generic behavior unchanged):**
- `workflow::succeed_attempt(...)`: the pure Passed transform of `apply_outcome` (`:2990–3031`), used by both the generic and the protected path;
- `artifacts::publish_result_core(...)`: the field checks and postimages of `publish_workflow_result_tx`, without its `validate_authority` call. The generic path keeps calling `validate_authority` first;
- RN `closure::operation_delta`: re-exported `pub(super)` for success closure, unchanged.

**Ownership graph.** No cycles:

```text
Runtime -> PhaseDispatcher -> PhaseJobs -> Entry -> Job -> JobState
  JobState.outcome  -> RetainedStart::Launched{binding: Arc<NativePhaseBinding>}  (A)
  JobState.binding_plan -> Arc<ManagedBindingPlan> (A; at most ONE: normal or late)
  JobState.success  -> Option<Arc<SuccessContinuation>> (P, set once)
SuccessContinuation -> SettledPhase -> {snapshot binding -> NativePhaseSession -> launch -> marker/allocation,
                                       BindingAcknowledgment -> plan}
SuccessStage -> retained compact plans and acks (claim, observed, closure); none -> Job/PhaseJobs/Runtime
NativeSessions.Entry.phase -> PhaseActor -> retained settlement (A, strong)
```

## 5. Binding: normal, late and confirmation (P)

### 5.1 Late eligibility (`plan_late_binding`)

The late planner shares `projection` and the Workflow, Session and identity planning with the normal binder.

- **Memory conjuncts.**
  - `!proof.is_live()`, and `proof.owner().validate_known_registration()` is Ok.
  - `s = proof.settlement()` is Some, with `s.belongs_to(owner)` and `s.consumed().belongs_to(owner)`.
  - `proof.consumed()`, when Some, is pointer-equal to `s.consumed()`.
  - `s.allocation()` is pointer-equal to `marker.allocation()`.
  - The receipt is `OwnedTerminal` with Success.
  - `s.unit()` is WorkKnown/Success with closed effects, open finalization and session = allocated.
  - `s.session()` is Exited with `native_ref == s.thread()`.
- **Unit lineage.** The current successor Unit must equal `s.unit()` except for the **cleanup axis**: `version`, `updated_at` and `cleanup` only. This tolerates Core's `record_execution_cleanup` and nothing else.
- **Immediate conjuncts.** These run in addition to the binder's `validate_current_tx`, `validate_driver_live_tx`, the exact Session row and negative identities. Every expected image is planned outside Store from the SAME settlement:
  - the registered owner row is exact, with the invocation required to be `closed`. This is a variant of `validate_registered_owner_tx`; the normal variant keeps `<>'closed'`;
  - the invocation row is exact (Closed, with `native_thread`/`native_turn` equal to `s.thread()`/`s.turn()`);
  - the `native_results` row for the invocation is exact, body = the planned `serde_json::to_string(s.receipt())`;
  - the `managed_phase_admissions` row is exact with `confirmed=1, settled=1, uncertain=0`, and `input_effect_id`/`frame_sha256` equal to `s.consumed()`;
  - the `managed_phase_readiness` row is exact (`closed`, `start_ended=1`, `known_terminal=1`);
  - the latest Session is Exited. The late variant of `latest_session` admits only Exited, and the normal variant is unchanged.
- **Projection and payload.** The projection is identical to normal (`session_id` plus derived `execution`). The payload is identical except `proof_source="closed_settlement"` and `private_receipt_ref=<receipt id>`, as managed binding §4 already specifies.
- **Fast terminal.** A terminal that commits during a normal binding Immediate changes the Unit index, so normal refuses as a definitive `Conflict`, and a later turn plans late.

### 5.2 Root reconciliation sweep (`PhaseJobs::reconcile_success`)

The service loop calls it after RN's `reconcile_nonsuccess`; its `bool` ORs into `pending`. It reuses RN's sweep discipline: it snapshots ≤128 entries under `entries`, classifies under `job.state` only, uses its own `success_cursor`, takes ≤8 turns per sweep, runs ≤1 query-only snapshot plus ≤1 Store-mutex transaction per turn, and backs off 100 ms → 5 s.

A job is **due** when its `outcome` is `Launched` and its binding is not yet acknowledged, or its `SuccessContinuation` has a pending uncertain write, an unpublished Driver closure, or releasable acknowledgments. Per turn, in order:

1. **Uncertain retained binding plan.** Run `confirm_managed_binding(plan)`:
   - Known → install `BindingAcknowledgment`;
   - RolledBack → clear the uncertain mark and keep the plan;
   - Err → Held, re-probed read-only every 5 s.
2. **No acknowledgment.** Take `snap = binding.owner_arc().binding_snapshot()?`:
   - settlement Some → `plan_late_binding`, then `bind_managed_phase`. The new plan replaces the retained one **only** after that retained plan is RolledBack or definitively refused; never both;
   - settlement None, owner live, last refusal a definitive `Conflict` → retry `plan_managed_binding` after backoff;
   - otherwise not due.
3. **Acknowledgment plus settlement.** `SettledPhase::issue(snap, ack)`, then install `SuccessContinuation` once (pointer-checked) and send a watch hint. This is memory only.
4. **Continuation housekeeping.** At most one of: confirm a retained uncertain claim, observed or closure plan (§11); publish the Driver closure (§10.4); or release acknowledgments (§10.5).

The start task's existing one-shot `bind_returned` remains the fast path. It now installs its plan before Store, as today, and records `Known`, `Conflict` or uncertain instead of only Ok/Err. Its outcome is never retried in the start task.

**Convergence.** Normal and late cannot both commit: the trigger allows at most one `session_bound` per operation, the first-link CAS requires the original Workflow, and only one plan is retained per job. Duplicate delivery (start task and sweep, or repeated sweeps) meets the single-flight stage token and then exact confirmation. Lost, full or closed watch notifications only delay the sweep; the service timer re-classifies from retained state.

### 5.3 Confirmation

`confirm_managed_binding` runs one read Immediate with no write, under the full original currency of the plan's kind:
- **Known** iff the Workflow equals the plan's exact postimage and exactly one `session_bound` link exists with the plan's `at` and byte-equal payload. This applies equally to a committed normal or late plan.
- **RolledBack** iff the Workflow equals the original post-marker raw and there are zero links.
- **Anything else is Held.**

The fresh-plan `AlreadyBound` path (`binding.rs:331–358`) is reached only when no retained plan exists (impossible within one epoch); a mismatching `proof_source` is Held.

## 6. Driver continuation (P)

### 6.1 Lookup

`Runtime::settled_phase(task, association)` reads Root custody, pointer-checked, holding one lock at a time:
1. `PhaseHandoffs.entries[task]` → `slot.handoff` → `assets.origin`.
2. `origin.ticket.upgrade()`; its association must satisfy `same_association(caller)`. Proposed: a `pub(crate)` read-only accessor on `DriverReadTicket` that returns its retained association by reference.
3. `origin.allocation` → the PhaseJobs entry by operation, with `Arc::ptr_eq` on the allocation.
4. The job's `success` cell, or its RN or success closed acknowledgment.

It returns `NoHandoff`, `Pending` (not yet bound or settled), `Held(reason)`, `Settled` or `Closed`. Nothing is read from SQL. Passive status and Goal views do not call it.

### 6.2 Step state machine

In `step_driven_initial`, the order is:
1. RN's Failed branch (unchanged).
2. For an active Executor attempt: `Settled` → `continue_settled_executor`; `Pending`/`NoHandoff` → the existing handoff observation (unchanged); `Held` → `StepResult::Waiting{reason}` with no write.

`continue_settled_executor` dispatches on the Workflow attempt state and the retained stage. It performs at most one durable write per call, and every await happens before any Store guard:

| Attempt / stage | Action |
| --- | --- |
| Running and bound | (a) capture (§8); (b) `inputs` → captured `SourceSnapshot`; (c) drift policy: class escalation, rules change or `!same_sources` for a non-target-producing phase → Held with no write (invalidation of a bound open phase needs its own typed port); (d) plan and write `gate_claim` (§9.1) |
| Evaluating, claim acknowledged, no evaluation started | Set `evaluation_started` under the stage mutex, then `evaluate_settled` (§9.2). Retain `SettledGateCompletion`, then plan and write `gate_observed` |
| Evaluating, evaluation started, no retained outcome | Held: "gate outcome unknown; explicit recovery". No re-evaluation |
| Evaluating with a Passed observation acknowledged | `settled_publication`, then `prepare_pack(next)`, then plan, materialize and close (§10) |
| Waiting (from `gate_observed`) | Read-only `StepResult::Waiting{detail}`. **No automatic re-claim** |

The SAME retained plan is reused after an uncertain write (§11). The Driver writes or confirms only through the single-flight stage token, and the Root sweep never evaluates gates or captures.

### 6.3 After closure

Once the job holds a known success acknowledgment and the Driver is published, these run once each:
- `retire_closed_handoff` on the Sources slot (pointer-checked custody allocation);
- the read-only branch: active None, with the last attempt the first-Executor Succeeded phase → `StepResult::Waiting { phase: next, reason: "<next> typed Driver continuation unavailable (SC-N)" }`. It writes nothing, grants nothing, and replaces only the `bail!` for this state. `Finished` (no next phase) is returned unchanged.

## 7. Marker-aware currency: the protected reader (P)

`validate_settled_tx(tx, c)` is the single protected replacement for generic finalize authority on a marker-bound open phase. It is exact, with no decode, encode or hash:

1. `marker.validate_driver_live_tx(tx)`: the SAME frozen post-marker Driver row, the live association, and the exact Source7 result or ticket source.
2. `validate_current_tx(tx, marker, &c.current)`: the open operation; exact P/G/T with the Task = marker `task_after`; the Workflow endpoint and ledger head; locks and latest Context = the original; the exact current Unit index.
3. The planned Unit equals `c.settled.settlement().unit()` modulo the cleanup axis. This was checked at planning. Under Store only the exact current Unit bytes are compared, and a mismatch replans (≤3 consecutive replans per stage, then Held).
4. Finalization open, effects closed, `session_id` = allocated.

Exact P/G bodies subsume the generic parent-activity and governing-digest checks. Each protected Store call plans `SettledCurrency` query-only outside the Store mutex, so a capture with ≤14 helpers runs ≤15 snapshots plus 1 recheck.

**Consumer mapping.** Generic consumers stay unchanged and keep refusing marker-bound rows. Every protected consumer below is new and uses `validate_settled_tx`:

| Generic consumer (refuses marker) | Protected PR1 consumer |
| --- | --- |
| `UnitGit::run_command` → `validate_execution` + `reserve_execution_helper_pinned` | `UnitGit::for_settled` → `reserve_settled_helper`; `reconcile_managed_effect` (no authority check) unchanged |
| `ResultStore::capture` → `validate_execution`, `stage_result` | `capture_settled` → `stage_settled_result`; `ready_result` unchanged |
| `workflow_publication` → `validate_execution` + `UnitGit::new` | `settled_publication` → protected validator + `UnitGit::for_settled` + `verify_inner(Current)` |
| `ManagedWorkflowGates::terminal` → `validate_execution` | `settled_terminal`: the same predicates sourced from the settlement and exact rows |
| `frame()` capture branch | `capture_settled` (same slot serialization; requires the slot's `handoff` custody allocation = settlement allocation) |
| `publish_workflow_result_tx` → `validate_authority` | `publish_result_core` inside `close_phase_success` (§10) |
| Generic Driver `validate` / Source `validate_row` / `after_write` | Unused on the protected path. Source7 present at the marker → success closure Held (§3.2) |

## 8. Capture (P)

`capture_settled` keeps `capture`'s exact steps:
- HEAD of the Unit worktree via `rev-parse`; the object-format check; ownership;
- `merge-base --is-ancestor base HEAD`; exact `^{commit}`; `qualified_content_scoped`;
- staging; fetching commit and base into the per-Project `results.git` (no alternates or hardlinks); `fsck --full --strict`; `rev-list --missing=error` for both OIDs;
- the durable manifest, fsync, and Ready.

Every helper runs under `UnitGit::for_settled`; the Driver ticket is not used. `capture_settled` then runs `verify` (historical reader). The Sources slot frame becomes the captured revision and artifact, exactly as `frame()` does today.

- **Content stays untrusted.** Native output is untrusted Git content; fsck and the bounded readers are the existing safeguards.
- **A text answer is not a result.** An `APPROVE` text answer is not a result. Only the captured HEAD graph is.
- **HEAD == base.** PR1 preserves the existing predicate, which accepts HEAD == base (AUTHOR_NOTES). Controls credit only the commit-producing fixture mode.
- **Helper outcomes.** A helper `Unknown` outcome makes the capture fail, and the stage is Held: closure requires every Unit helper effect to be settled (§10.2).

## 9. Gate claim, evaluation and observed (P)

### 9.1 `gate_claim`

- **Workflow delta.** From the bound endpoint, only the active attempt changes: `state` Running → Evaluating and `claimed_observations = observations.len()`. Record `version+1` and `updated_at=at`. A typed roundtrip and neutralized byte-equality are checked as in RN's `workflow_delta`.
- **Nothing else.** No Task, blocker, Session, source or Context write.
- **Link payload.** The trigger-required header plus `{claimed_observations, gate_phase, closure_headroom_bytes}`.
- **Immediate.** `validate_settled_tx`, then the budget: `charged + growth + link + SUCCESS_CLOSURE_HEADROOM ≤ WORKFLOW_BYTES` (§12). Then exact permits for the `records` UPDATE and the `audit` INSERT, mirroring the binder (`binding.rs:435–462`).
- **Single pair.** One pair per PR1 run. The 100th claim refuses by trigger, but PR1 never re-claims.

### 9.2 `evaluate_settled`

It is `evaluate` with three substitutions:
1. The claim check is `Self::claim` (reads only).
2. Implement uses `settled_terminal`, which applies the same predicates as `terminal` (`workflow_gates.rs:172–198`) from the SAME settlement: `attempt.execution`/`unit`/`agent`/`session_id`; Exited; Success; WorkKnown; finalization open; effects closed. The Unit is compared modulo the cleanup axis.
3. The source recapture is `capture_settled` and must equal the invocation sources.

Everything else is unchanged: the Ready artifact = the invocation artifact; `ResultStore::verify`; the exact claim recheck; the `Verification` receipt via `put_record`; `Evidence.session_id = settlement Session`. Requirements and every non-supported phase return the existing `Waiting`.

`SettledGateCompletion` is constructed only here. It is retained in the stage before any write.

### 9.3 `gate_observed` (fused, one link)

- **Workflow delta.** Append exactly one `GateObservation { sources: authority_only(source), outcome, error, at }`, encoded at most 64 KiB (refused otherwise, never truncated). Then apply the disposition:

  | Outcome | Attempt after | Workflow-level | Link `outcome`, `reason_code` |
  | --- | --- | --- | --- |
  | Passed(evidence) | stays Evaluating | — | `passed`, gate receipt id, evidence SHA-256 |
  | Waiting(fixed string) | Waiting, `detail` = that compile-time string (≤128 B) | — | `waiting`, `evidence_integration_unavailable` |
  | Failed(_) | Waiting, `detail` = `"gate failed; bound non-success closure unavailable"` | `held_reason` = same constant | `held`, `gate_failed` |
  | Err (unknown) | Waiting, `detail` = `"gate outcome unknown; explicit recovery required"`; `error` = that constant | `held_reason` = same | `held`, `gate_unknown` |

- **No raw text.** Raw error and gate text never reach durable rows or payloads; they go only to bounded in-memory attention.
- **No Task writes.** No Task `WaitingHuman` or blocker is written. Waiting and Held status are derived from the Workflow for the Driver, Goal and CLI.
- **Trigger.** The trigger requires the immediately preceding `gate_claim`.

## 10. Successful `phase_closed` (P)

### 10.1 Plan (query-only, outside every lock)

Inputs:
- the SAME `GateObservedAcknowledgment` with Passed `Evidence`;
- `WorkflowPublication` from `settled_publication` (current-reader graph verification plus exact artifact recheck);
- the `ContextVersion` from `prepare_pack(next or phase)` with `version = latest + 1`;
- `at`.

Steps:
1. **Currency.** `plan_settled_currency`; Source7 present → Held; Driver marker anchor = this operation.
2. **Task/Workflow.** `succeed_attempt` (the shared pure transform) over the bound endpoint Workflow and the marker `task_after`:
   - attempt Succeeded, `completed_at = at`, `active = None`, `completed[phase] = evidence`, `sources = captured`, `set_context`, `finished = next.is_none()`;
   - Task: `artifacts += evidence.artifacts`, `phase = next.key()`, `state = phase.task_state()`, `context_version`, `revision`, `version+1`, `updated_at = at`.

   Check `validate_evidence`, `validate_transition(task_after, workflow_after, Some(before))`, `validate_context` and the `publish_result_core` field checks (artifact Ready; revisions and artifact pointers equal; dependency equality; evidence session).
3. **Unit.** Current (cleanup-axis-tolerant) image → `result_finalization_open=false`, `artifact_id`, `version+1`, `updated_at = at`.
4. **Artifact.** Ready → Published, `version+1`, exact body.
5. **Operation.** RN `operation_delta`: phase_open 1 → 0, version 2.
6. **Driver.** `DriverMarkerAdvance::plan_success_closure`: the old image is the SAME frozen post-marker row. The new row has `version+1`, state `driving` and `marker=None`. Its pins are computed **here** from the planned Task, Workflow, Context and Unit postimages plus the unchanged P/G, prerequisites and generation; `source=None`. These are exactly the pins that the generic `snapshot` will recompute after commit.
7. **Payload.** The trigger header (with the ORIGINAL `context_version`) plus:
   - `closure="success"`, `proof_source="owned_settlement_gate_passed"`;
   - `native_receipt_ref`, `gate_receipt_ref`, `artifact_id`, `artifact_revision`, `manifest_sha256`;
   - `unit_version_before/after`, `task_version_before/after`, `next_context_version`, `next_phase`, `operation_version_after=2`, `driver_version_after`;
   - `at`.

   The key set is exact, the encoding ≤4096 bytes, and there is no error text.

The plan retains compact digests and scalars, plus the Context raw (≤8 MiB, not re-derivable: rules are read from the Project) and the Arcs. Every other image is re-materialized per turn from the SAME retained inputs, and the digests must match; a mismatch is Held.

### 10.2 Immediate conjuncts (all before any write)

`close_phase_success` takes `control_admission` (awaited before Store, as CA §5.7 and marker publication do), then the synchronous SharedStore, then one `Immediate`:
1. The selected database path. `validate_settled_tx` against the planned endpoint (Workflow = the `gate_observed` postimage; ledger head = that link).
2. The Unit, artifact (Ready) and Task (the marker `task_after` raw) rows are exact. The latest Context version = the original, and no row exists at the next version.
3. Every Unit `managed_effects` row is non-Pending and non-Unknown (≤257 rows); there is no `retained_git` Pending row.
4. Ledger: count < 256 and zero `phase_closed`.
5. Budget: `charged + all growth ≤ WORKFLOW_BYTES`.

Any failure rolls back and returns `Conflict(cause)`.

### 10.3 Writes (rowcount exactly 1 each)

| # | Row | Guard |
| --- | --- | --- |
| W1 | `execution_units` exact 13-column CAS | writer contract (not a permit table) |
| W2 | `result_artifacts` Ready → Published, exact version/body CAS | existing publication semantics |
| W3 | `tasks` exact old/new CAS (the `put_task_tx_at` column set plus a `body=old` predicate) | not a permit table |
| W4 | `context_versions` INSERT (existing `put_context_tx`) | uniqueness |
| W5 | `managed_phase_operations` 32-column exact UPDATE | exact permit; core trigger |
| W6 | `records` Workflow exact UPDATE | exact permit; identity/version trigger |
| W7 | `audit` `phase_closed` INSERT (`sequence` allocated under TX) | exact permit; private, no-replace and chain triggers (need W5 and W6 first) |
| W8 | `task_drivers` exact UPDATE (marker cleared) | exact permit (`binding_driver_UPDATE`) |
| — | existing non-reserved `execution.result_published` event | unchanged semantics |

W5–W8 run inside ONE `with_exact_permit` window of four rows (≤128), followed by `ensure_consumed`, then commit. The following are never written: Project, Goal, Session, owners, inputs, admissions, readiness, Source7, locks, `task_execution`, `cleanup_jobs` and quota rows.

### 10.4 Post-commit Driver publication

Still holding the Store guard and admission, run `publish_success_driver`: an exact committed-row read, then the SAME association's `publish_exact`. It returns the `SuccessClosureAcknowledgment` with `driver_published` = the outcome.

- **On failure:** the acknowledgment and the plan are retained. A later turn by the Root sweep or the SAME live worker re-runs only `publish_success_driver` (idempotent per `publish_exact`).
- **After an actual worker exit:** Held. Nothing is revived, and no cache is published from rows.

### 10.5 Acknowledgment consumers

Order (all pointer-checked; removals are retried without a DB write):
1. Install `JobState.closed_ack = ClosedPhaseAck::Success` and send `ClosedSuccess`.
2. Driver published (§10.4).
3. The Driver step's Sources-slot `retire_closed_handoff` sets `sources_retired`.
4. Root: `retire_closed_marked` (PhaseSupervisor slot), then `PhaseHandoffs::retire_closed`, then `retire_closed` (PhaseJobs entry).

Until step 4 finishes, `ensure_*_shutdown_complete` keeps failing, as today. The Native registry phase Entry is **not** released (§18).

## 11. Uncertain commits, repeated delivery, cancellation and stop (P)

- **Uniform uncertainty rule.** Every new write (binding, claim, observed, closure) returns `Known`, `Conflict` or `Err` (uncertain). After `Err`, only the SAME retained plan is confirmed:
  - **Known** iff every postimage is exact and exactly one link with the plan's `at` and data exists;
  - **RolledBack** iff every preimage is exact and the link is absent;
  - **otherwise Held**, with a read-only re-probe every 5 s.

  After RolledBack the SAME plan, `at` and digests are retried. A `Conflict` drops the plan, and the next turn replans from the SAME proofs; drift is never laundered, because plans derive from the original and settlement images. Link presence alone is never acknowledgment.
- **Repeated delivery.** The Driver and the Root sweep share the stage's single-flight token. A duplicate step sees Known through confirmation. Trigger uniqueness, the Workflow version CAS and the per-kind allowances make a second link impossible.
- **Dropped futures.**
  - A dropped start future: the job retains the outcome and binding.
  - A dropped Driver future after a retained write: the Root sweep confirms it.
  - A Driver dropped during gate evaluation: Held (§6.2).
  - A Driver cancelled between claim and observed: the claim is retained, Evaluating; Held.
- **Stop.** Shutdown and `Runtime::drop` revoke the worker, so `validate_driver_live_tx` refuses later writes. Closure holds `control_admission` across commit and Driver publication, so stop and commit linearize as in CA §5.7.
- **Restart.** A new epoch never reconstructs `SettledPhase` or plans. Open operations stay Held; closed ones stay closed. RestoreFactualBinding is absent, and nothing is inferred from timeouts.

## 12. Finite costs (P)

All figures are encoded lengths taken from existing bounds. They are not heap or RSS bounds, and control M1 measures them. There is no truncation: an over-bound value refuses (Held).

| Transaction | Mandatory surfaces (each existing bound) |
| --- | --- |
| Late bind | The binder's surfaces (P/G/T ≤3×8 MiB, Workflow ≤8 MiB old/new, locks ≤256×16 KiB, Context ≤8 MiB, own Session ≤4 MiB, negative identities ≤4096×16 KiB, owner ≤32 KiB, invocation ≤`INVOCATION_BYTES`) plus receipt ≤`RECEIPT_BYTES`, admission ≤8192 B and readiness ≤4096 B; one link ≤4096 B |
| Settled helper | Projection set above (no Session or identity scan); one `managed_effects` insert ≤8 KiB |
| `gate_claim` | Projection set + Workflow old/new ≤2×8 MiB + link + `charged_scope_bytes` aggregates |
| `gate_observed` | as claim + observation ≤64 KiB |
| Closure | Projection set; Workflow old/new ≤2×8 MiB; Task old/new ≤2×1 MiB; Context new ≤8 MiB; artifact ≤2×128 KiB; Unit ≤2×44 KiB; operation ≤2×4 MiB; Driver ≤2×128 KiB; effects ≤257 rows ×8 KiB; link ≤4096 B; four permit rows |
| Driver publication | one Driver row ≤128 KiB |

- **Mandatory closure reservation.** `SUCCESS_CLOSURE_HEADROOM` = 8 MiB (Context) + 8 MiB (maximum Workflow postimage) + 1 MiB (Task) + 512 KiB (artifact, Unit, Driver, receipt record) = 17.5 MiB. `gate_claim` refuses unless the headroom fits. This is in addition to the existing link reserve (1 MiB − spent while `phase_open=1`), which closure releases by derivation.
- **Ledger.** PR1 spends 4 links (bind, claim, observed, closure) of 256. Diagnostics and hold allowances are untouched, and the `phase_closed` slot is never consumed early.
- **Retained memory.**
  - The binding plan (existing: Workflow after ≤8 MiB + Session ≤4 MiB) stays at most one per job.
  - The success stage retains compact plans plus at most ONE closure Context raw (≤8 MiB) and the observation (≤64 KiB).
  - Worst case: ≤128 jobs × (12 + 8.1) MiB ≈ 2.5 GiB encoded. That is finite, but large. M1 reports measured values, and AUTHOR_NOTES records the policy question.
- **Sweep.** ≤128 jobs classified, ≤8 turns and ≤1 Store transaction per turn.

## 13. Locks and lifetime (P)

| Held | May take |
| --- | --- |
| `PhaseJobs.entries` | nothing new (existing `start` only) |
| `job.state`, `SuccessContinuation.stage`, `success_cursor` | nothing (leaves) |
| `PhaseHandoffs.entries` | `slot.handoff` → `assets` (existing order) |
| Sources slot (tokio) | SharedStore briefly, never across an await (existing `frame()` discipline); Git awaits hold no Store |
| `control_admission` (await) | SharedStore (synchronous), then the permit manager, then the association `binding` mutex |
| SharedStore | the permit manager leaf; custody and actor leaves only through existing pointer checks |

- **Store calls.** No Root mutex is held across a Store call, and no await happens under SharedStore.
- **Heavy work.** Material, Context and plans are dropped after the Store guard (RN's `borrowed_store_turn`).
- **Debug assertions.** The existing debug `ROOT_LOCK_DEPTH` and per-turn transaction counters are reused for every new port.
- **Cycles.** No ownership cycle (§4 graph).

## 14. Authority surface (P)

| Actor | Late bind | Capture/gate | Closure | Release |
| --- | --- | --- | --- | --- |
| Root sweep (same owner and epoch) | yes, from the SAME job's retained owner | no | confirm or publish only | with acknowledgment |
| SAME live Driver worker | no (fast path stays in the start task) | yes, through `SettledPhase` | yes | Sources slot only |
| Other Tasks' Drivers | no (pointer and association mismatch) | no | no | no |
| Generic Engine (`persist`, `fail`, `retry`, `observe_workflow_gate`), CLI, status | no | no | refused by record guards / generic marker refusal | no |
| New epoch, raw supported writers | no | no | refused by triggers and permits | no |
| Native Agent and hooks | none | content only (untrusted Git) | none | none |

Sink rules:
- `phase_closed` success comes only from a Passed `gate_observed` acknowledgment and a `WorkflowPublication`;
- the artifact becomes Published only inside closure;
- the Driver is re-anchored only by `DriverClosureAdvance` from the SAME `DriverMarkerAdvance`.

## 15. Impact analysis

| Changed / consumed | Consumers (V) | Handling |
| --- | --- | --- |
| `bind_managed_phase` return type; `ManagedBindingPlan` kind | `runtime/phase_jobs.rs:364–382` (only caller) | Mapped to `Known`/`Conflict`/uncertain; the normal predicate is unchanged |
| Late variants of `latest_session`, `validate_registered_owner_tx`, invocation checks | `binding.rs` only | Normal variants unchanged |
| `JobState` fields; `reconcile_success`; `ClosedSuccess` | `phase_jobs.rs`; `PhaseInvocation::wait` (`:84–99`, treated as terminal); `runtime/service.rs` sweep | RN's sweep, cursor and turns are untouched; separate cursor |
| `retire_closed_marked` / `retire_closed` take `ClosedPhaseAck` | RN `phase_supervisor.rs:629`, RN `phase_jobs.rs` | The RN path passes `NonSuccess`; behavior identical |
| `PhaseHandoffs::retire_closed`; Sources `retire_closed_handoff` | `phase_handoffs.rs:413–435`; `workflow_source.rs` slot | First genuine retirement producer; shutdown checks unchanged |
| Protected Unit, Git, results, effects and artifact ports | `git_io.rs`, `results.rs`, `effects.rs`, `artifacts.rs` | Additive. Generic `capture`, `workflow_publication`, `stage_result` and helpers keep `validate_execution` and their marker refusal |
| `publish_result_core` refactor | `publish_workflow_result_tx` (`artifacts.rs:392–481`) | The generic path still calls `validate_authority` first; checks unchanged |
| `succeed_attempt` refactor | `apply_outcome` Passed (`workflow.rs:2990–3031`) | Byte-identical generic output; covered by existing Workflow tests |
| `evaluate_settled`, `settled_terminal` | `workflow_gates.rs` | `evaluate`/`terminal` unchanged |
| `DriverClosureAdvance`; Driver pins after closure | `state/runtime/driver.rs:98–255,320–355`; `marker.rs`; `runtime/driver.rs:271–314` | After closure, the generic `validate` passes on the marker-free row; the record guard still refuses generic Workflow writes |
| `driven_initial.rs` settled and post-closure branches | `task_driver.rs:119–137` | Read-only Waiting replaces worker exit only for the post-success state |
| First production `gate_claim`, `gate_observed` and success `phase_closed` | `schema.rs:358–416`; `successor.rs` `KINDS` | Payloads satisfy the header, context and predecessor rules; after closure `validate_open_tx` fails for every open-currency consumer (intended) |
| `phase_open=0` | `charged_scope_bytes` reserve; `runtime/waiting.rs` (`phase_open=1` filter); Driver claim occupancy; quota policy | Reserve released; waiting status stops; Task occupancy remains through its `driving` Driver |
| New constants | `SUCCESS_CLOSURE_HEADROOM`, 64 KiB observation, 3 currency replans, success turns 8 | PR1-local |
| Schema, `SCHEMA_VERSION`, triggers, permit catalogue | — | **Unchanged** |
| Environment, paths, input lists | — | No change. The fixture variant is test-only configuration |

Not affected: CA activation and contracts, RN non-success semantics and `NonSuccessReader` sealing, Native transport, quota, preparation, Verifier8, and legacy paths.

## 16. Controls and mutants (P; none executed)

Every positive uses the CA genuine lane: compiled `Runtime::new` and `start`, accepted Unix ingress, a real Git Project, the configured NativeAdapter, and `native_fixture.py` as `AgentConfig.command`, with a declaration and reviewers `rev-a`/`rev-b`. For the commit mode, the fixture is configured **without** `WORKFLOW_SCENARIO`. The fixture is protocol wiring, never qualification.

Ground rules:
- `#[cfg(test)]` seams inject timing or faults only.
- Raw SQL and filesystem edits are negative stimuli only.
- A refusal before the named stage, a compile failure or a SETUP stop earns no credit.

| ID | Control |
| --- | --- |
| SC1 | Claude and Codex, QUICK Implement, commit mode. The stage sequence is held start → normal `Bound` → release `fixture-release` → owned success → capture → claim → Passed → observed → closure. Assertions: <br>• Task.version equal to the marker until closure, then +1 exactly once; <br>• links `[session_bound, gate_claim, gate_observed, phase_closed]`; <br>• artifact Published, with revision = fixture HEAD ≠ base and `results.git` refs and fsck; <br>• Unit finalization closed; <br>• exactly one new Context; <br>• the Driver row marker-free and the generic `validate` passing; <br>• the next step Waiting (`commit`), with the worker still driving. |
| SC2 | Late. A cfg(test) seam delays `bind_returned` until the settlement exists. Normal refuses (`Conflict`); the Root sweep late-binds with `proof_source=closed_settlement` and `private_receipt_ref`; then SC1's assertions. Cleanup Unknown is present and binding still happens once. |
| SC3 | NotDispatched. Normal binding is forced to `Conflict` by genuine writer-lock contention while the owner is live and pre-ACK. Late planning refuses (no settlement); the later normal retry binds `normal_return`. |
| SC4 | Uncertain commits. A failpoint returns `Err` after commit, and separately before commit, in each of: bind, claim, observed, closure. Assertions: the confirmation is Known or RolledBack; the SAME `at`/digests; exactly one link of each kind; L-x variants (a C1 drift after commit) are Held. |
| SC5 | Lost notifications. A seam suppresses job watch sends; binding still converges via the service timer within ⌈n/8⌉ sweeps plus backoff. |
| SC6 | STANDARD Requirements. Bind → capture (Ready) → claim → observed Waiting (`evidence_integration_unavailable`). After ≥50 Driver polls: still one claim, no `phase_closed`, Task unchanged, Requirements not completed, artifact not Published. |
| SC7 | Negatives after genuine setup: <br>• Goal or Task edited through an existing legitimate writer during the held start → Conflict/Held and zero links; <br>• `ordinary-failure` scenario → no settlement, Held "bound non-success closure unavailable"; <br>• peer killed before terminal → Unknown/Lost → no late bind; <br>• Runtime restart → Held and no reconstruction. |
| SC8 | Graph integrity. Between capture and closure, a negative-only corruption of a `results.git` object → `settled_publication` refuses and closure is never committed. |
| SC9 | Stop. Shutdown while closure waits for admission → no commit. Shutdown after commit with publication pending → Held, counted pending, no revival. |
| SC10 | Four Tasks across two Projects (two Claude, two Codex) in commit mode → four independent closures, with no cross-job acknowledgment or Driver publication. |
| M1 | Near-bound bodies (Workflow and Context ≈8 MiB, 256 locks): report measured retained and per-turn peaks against §12 terms. Fewer than the genuine bound is SETUP. |

| Mutant (compiled) | Killed by (first assertion) |
| --- | --- |
| Restore the Task write in binding, claim or observed | SC1: Task.version unchanged before closure |
| Late planner accepts settlement None or a live owner | SC3: a `closed_settlement` link before any terminal |
| Late planner skips the receipt/admission/readiness exact rows | defense only (the issuer guarantees them; no genuine producer of a mismatch) |
| Confirm accepts a preimage as Known (any port) | SC4 pre-commit: acknowledgment while the link is absent |
| Replan with a new `at` after uncertainty | SC4: link `at` or digest differs from the SAME plan |
| Root `reconcile_success` call removed | SC2: no binding |
| Capture uses the generic `validate_execution` | SC1: capture refusal ("marker-bound Driver…"), no Ready artifact |
| Protected validator omits `validate_driver_live_tx` | SC9 variant: a helper intent row after revocation |
| Gate Waiting rewrites Task WaitingHuman/blocker | SC6: Task changed |
| Automatic re-claim after Waiting | SC6: second `gate_claim` |
| Requirements Waiting mapped to Passed | SC6: `phase_closed` present |
| Closure skips `settled_publication` graph verification | SC8: closure committed |
| Closure omits phase_open 1→0 | SC1: chain trigger refuses (trigger kill) |
| Closure omits the Driver re-anchor or its publication | SC1: generic `validate` / association `is_current` false after closure |
| Post-closure Waiting branch removed | SC1: worker exits with Err |
| Headroom check removed | M1 near-bound claim admitted then closure refused; SETUP if near-bound bodies cannot be produced genuinely |
| Cleanup-axis tolerance widened to any Unit field | defense only (no genuine Unit-only non-cleanup writer) |
| `same_association` check removed from lookup | defense only (one Driver per Task) |

Review checks (not mutants): no decode, hash or encode under the new Immediate ports; no await under SharedStore.

## 17. Acceptance and MVP mapping (no test was run)

| Condition | PR1 contribution | Status |
| --- | --- | --- |
| MB-AC1 | Real current callback and completion stay eligible after Bound (SC1) | Partial; requires execution |
| MB-AC2 | Existing binder plus new ports refuse drift (SC7) | Partial; full per-predicate matrix unmet |
| MB-AC3 | Late identity negatives on the same checks | Partial |
| MB-AC4 / 4.a | Normal/late race, uncertain commit, lost notifications, NotDispatched refusal, cleanup-Unknown late bind once | Designed (SC2–SC5, SC7); restart remains Held (unmet) |
| MB-AC5 | Gate and success closure | Partial: diagnostic, bound failure closure and optional-exhaustion control unmet |
| MB-AC6 | Positives only through CA ingress and producers | Old direct fixtures not counted |
| MB-AC7 | §16 mutants | Unexecuted |
| MB-AC8 | Independent review, composed fmt/Clippy/build/regression, macOS/Ubuntu CI | Unmet |

**Full MVP (unchanged and unmet):** ≥4 native Tasks across ≥2 Projects and Goals; ≥2 reviewers; QUICK/STANDARD/STRICT end to end; ReviewEngine/Broker quota and recovery; Context ON/OFF; official Claude and Codex with unchanged auth and hooks on macOS and Linux; install, CI and final dogfood. PR1 is a necessary dependency only. Its fixture positives qualify no official CLI, both-OS run or full Workflow.

## 18. Unresolved dependencies and open gates

- **Composition.** Composition R+CA+RN-final, reviewed (§1 item 8).
- **SC-N.** Post-closure Evidence-phase Driver continuation (Commit, Tests, Pr, MergeGate, Cleanup) under the protected Workflow guard.
- **Requirements/Design evidence.** Requirements and Design evidence integrations, plus an explicit typed gate-resume lane.
- **Bound non-success closure.** Gate Failed/unknown; Native failure, Unknown or capacity after Bound. Also `native_diagnostic` and `gate_hold`/clear ports.
- **PR-2 to PR-4** (later, retry and Reviewer offers; ReviewerArtifactLease; artifact retention). ReviewEngine fan-out.
- **Restart and quotas.** RestoreFactualBinding / restart migration; native quotas and cancellation; scheduler progress.
- **Native registry.** Release of the phase Entry (an unbounded-over-time Entry growth risk before long dogfood).
- **Source7.** Source7-anchored success closure.
- **Human decisions.** SourceStop, Cancel and R2 human decisions: untouched.

## 19. Sequence and review

1. Sol 6.1 high implements on the composed base, committing each item:
   - (a) binding write/confirm types and the late planner;
   - (b) Root sweep, `SettledPhase` and lookup;
   - (c) protected currency, helpers and capture;
   - (d) claim, evaluate and observed;
   - (e) closure, Driver advance and publication;
   - (f) releases and Driver branches;
   - (g) seams, controls and mutants;
   - (h) master updates (`master/agent-execution.md`, `master/workflow-engine.md`) with implemented facts only, in the same PR.
2. This HOW: ONE independent non-author Sol 6.1 (`gpt-6.1-sol`) high review, orchestrated by Root after a full read of the frozen authored commit. Re-review covers only the finding side's delta.
3. Source: three parallel high initial reviews (Sol on the implementation diff, Opus on security/contract, Grok on reconciliation). Fixes go to the non-finding side, and Grok does not return. This HOW grants no source, qualification or merge approval.
