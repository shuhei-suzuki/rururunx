# PR1 narrow authority/contract HOW correction author task

You are official claude-opus-5-5 HIGH, the non-finding family contract author. This is ONE narrow correction batch after a completed Sol review and Root's factual disposition. It is design body authoring, not an independent review, implementation or approval.

Correct ONLY doc/design/issue-43-native-success-continuation-design.md at frozen H 0c35653c7a8aa05c78c5285c2f4b5370db520e70. Preserve the governing WHAT, existing authority and acceptance, document organization where practical, and all remaining broader PR1/SC-N/Reviewer/retry/recovery/official-CLI/both-OS/four-Task/full-MVP gates. Implement-only closure is an intermediate dependency, never full qualification. Do not expand WHAT or silently narrow acceptance. If a necessary correction genuinely cannot be made under governing WHAT, report the exact incompatibility instead of authoring a workaround.

Root-confirmed REQUIRED: SC-01,02,03,04,05,06,08,09,10,11,12. SC-07 is optional design-level producer-local API/privacy clarification; ordinary implementation privacy remains mandatory. SC-13/14 are optional factual corrections. The original reviewer called SC-07 mandatory; Root's verified disposition supersedes that classification. No source or mutant execution occurred during factual verification. Address same-pattern consumers of each required correction, with implementable sealed APIs/producer delivery/caller ownership and complete cost/impact maps; do not add generic placeholders or guess absent APIs. Preserve bounds, atomicity, original parent/owner/input/marker/source/lock/epoch currency, typed commit/rollback facts, genuine process fencing and the SAME uncertain plan. Record factual comparisons separately from new write authority. Status observations remain bounded nongrant facts and must reach actual Goal/CLI consumers. Fix cost arithmetic honestly from real command catalogue/corpus/effect writers and retained representations; shared Arcs, independently owned SQL encodings, borrowed turn material and actual encoded inventories differ from heap/RSS. Distinguish reachable genuine admitted boundaries, mechanical representation measurements, defense-only checks and honest SETUP prerequisites. Named compiled mutants need a causal reachable first assertion, never compile/setup/earlier redundant refusal as a kill.

Pins are independent immutable source lanes, not a composed qualified build. Preserve old provenance R/CA/RN and describe CA-current and RN-current deltas where pertinent. The Root-verified excerpts carry commit/path/range/blob/whole-file and excerpt hashes. Their text is unmodified Git content. Source derived facts are static; proposed APIs and controls remain proposed. No UUID chronology or runtime fact may be invented.

You have complete inline selected inputs. Use no tools, subprocesses, filesystem reads/writes, network workloads, GitHub operations, tests, providers, subagents or reviewers. Do not inspect/copy authentication/configuration/hooks/settings/token/HOME/stores. No permissions changes, guard removal, capability advertisement, fabricated custody/rows/fixture positives, public grant/constructor, Task-version binding, new launch ownership barrier or timeout reconstruction. No SourceStop/Cancel/R2/G3 policy decisions; RN1 stays distinct. Production unsafe remains forbidden. Do not use rtk, heredocs, cd &&, PATH exports, active shell variables, command substitutions, backticks or globs. Normal official parent CLI bootstrap is orchestration only; you must author solely from inline inputs. No repository files may change. Root alone applies and commits returned document bytes after full read.

Return the complete corrected English HOW, not a plan, independent review verdict or source implementation. Output EXACTLY one strict JSON object, no markdown fence or diagnostic narration, with keys:
  what_status: "EXISTING_WHAT_UNCHANGED" or "CONTRACT_BLOCKED";
  document_path: "doc/design/issue-43-native-success-continuation-design.md";
  body: complete corrected Markdown including unchanged sections (empty only if genuinely blocked);
  correction_map: array of {finding_id, disposition, changed_sections, concrete_source_refs, rationale, residual_limits};
  impact_map: concrete changed APIs/fields/constants/consumers/caller ownership/lock lifetime, generic compatibility and regression/control implications;
  unverified_limits: honest outstanding prerequisites and qualification gates.
Minimal applicable correction is the goal. Do not add arbitrary new acceptance gates, unchecked numeric heap ceilings, broad rewrites or optional abstractions. No self-approval. A nonauthor finding-side Sol HIGH delta review follows Root's artifact admission later.


## INLINE INPUT FULL-HOW
{"id": "FULL-HOW", "bytes": 68120, "sha256": "2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6", "commit": "0c35653c7a8aa05c78c5285c2f4b5370db520e70", "source_path": "doc/design/issue-43-native-success-continuation-design.md", "blob": "fd9010eb78c3b7f91dad165447e754166a7daaa7"}
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


## INLINE INPUT FULL-WHAT
{"id": "FULL-WHAT", "bytes": 14618, "sha256": "11ec2b9281fa38558ce10b8edcfaa58e2a7c3c419f8e5e9b3cffa655ad297387", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "doc/requirements/issue-43-managed-binding-requirements.md", "blob": "0731deb88550750ece8a3f81e7aa08b7501f891d"}
# Issue 43: managed native Session binding requirements

Risk: STRICT. Status: requirements approved by two independent round2 reviews
of `7e47421a24225583795245b307e574fb3156dd7e`; see the
[review evidence](../verification/issue-43-managed-binding-requirements-round2.json).
No binder, allocation producer, migration or native qualification is delivered
by this file.
Source baseline: `5b6e8a8152926b643c6926b87e853de12b9319fc`. The reviewed legacy
contract is [Requirements9](https://github.com/shuhei-suzuki/rururunx/blob/b191b466d5dea303585cfaf6968c6fb178be79cd/doc/requirements/issue-43-requirements.md)
and [Design10](https://github.com/shuhei-suzuki/rururunx/blob/b191b466d5dea303585cfaf6968c6fb178be79cd/doc/design/issue-43-binding-mechanics.md).
This supplement connects that contract to the
[managed execution profile](agent-execution-requirements.md); it does not claim
that its ExecutionUnit or NativeInvocation already implements private #19 admission.

## Verified problem and dependencies

At the source baseline, the normal managed Workflow return sets session_id and
execution, refreshes Project/Goal/Task snapshots, then uses the ordinary Workflow
transaction (`workflow.rs::prepare_managed`). That transaction unconditionally
calls put_task_tx. The unmanaged return also uses the ordinary transaction.
Neither is the required factual record-only binding. A later snapshot refresh
can also erase the interval in which an owner changed during start.

The actual native registration transaction already writes Session, session_units
and NativeInvocation without writing Task. It checks exact input/context bytes.
It does not capture the original marker P/G/T/Workflow/full-lock frame, allocate
the #19 private phase owner/input pair, or prove the sole binding writer. Public
ExecutionAuthority/ManagedSessionRef, serialized rows, source hashes and a
returned Session ID are not that missing credential. Current managed authority
validation uses Unit/generation/epoch and governing content; no observed old
Task-version native failure is asserted for this new profile.

Runtime9 Driver readiness and Review member execution depend on actual binding
composition. Accepted Goal content, read-only status, cancellation fencing and
other independently qualified consumers may progress without advertising native
Driver readiness. Schema8 verifier and schema9 Runtime migrations must compose;
the binding implementation cannot independently reuse a reserved schema number.

## Required behavior

MB1. A successful first binding is one factual SQLite Immediate transaction.
It changes only the active native attempt's absent session_id and its previously
absent managed execution reference to the exact private allocated Session/Unit,
the Workflow record version/updated_at, and one reserved bounded factual audit.
The managed reference is a derived redundant projection; it grants no authority.
Task, Project, Goal, Session, Context and every scoped WorktreeLock body/version
remain byte-for-byte unchanged. Binding never claims work success, native death,
cleanup success, a Review verdict or phase completion. Ordinary phase/context/
state transitions keep their existing separately authorized Task writes.

MB2. Before the native start interval, the actual dispatch-marker transaction
captures an immutable private frame containing exact P/G/T identities/versions,
Workflow identity/version/complete body hash, scope, active index/generation,
Runtime state-root instance/epoch, Unit identity/execution generation,
phase/actor/provider/role, exact input/context/source/revision pins and the complete
scoped WorktreeLock identity/version set. It captures the resulting post-marker
Task/Workflow versions in that same transaction. Later current reads, a Driver
snapshot, a retry or a native returned identity cannot replace this frame. First
binding requires the original Workflow marker version/body, with no intervening
diagnostic or bookkeeping link. Current activity and every exact frame predicate
must still hold; unrelated parent version changes are not exempted.

MB3. Every fresh native Workflow attempt requires the genuine private allocated
phase Session owner and prepared-input preparation/admission/consumption pair
specified by #19. Its actual producer must be connected to the real registered
adapter's owned invocation and Store transaction. Marker, allocation and input
intent identities correlate exactly; later permission/transport/terminal journals
cannot overwrite or manufacture the consumed input. No public IDs/JSON, optional
caller flag, SQL fixture seed, synthesized hash, Unit DTO, terminal receipt or
capability advertisement can supply this credential. A genuine private producer
is mandatory on both normal return and closed-success late binding. Normal factual
binding permits the genuine allocated/validated/prepared pair in its defined
pre-input/pre-ACK state: it does not require or manufacture delivery, consumption
or acknowledgment. That same NotDispatched preparation cannot authorize late
successful binding, which additionally requires the actual correlated consumed
input and current owned successful terminal. Standalone
non-Workflow native fixtures do not establish this acceptance criterion.

MB4. The normal returned identity is checked against the selected registered
adapter and latest durable Session: exact ID/scope/actor/provider/role/worktree,
plus returned native_ref when present. The worktree is the genuinely prepared
Unit namespace: executor Task projection or a separate immutable-input reviewer
snapshot, never a live executor worktree substituted for review. This worktree
adaptation follows result protection; all other private owner/identity checks
remain mandatory. Starting may legitimately advance PID/native UUID/lifecycle
before binding; those observations neither grant authority nor invalidate an
otherwise identical return. Lost, absent, foreign or ambiguous Sessions reject.
The latest scoped provider/native UUID must have no distinct conflicting Session,
including Lost/history, and malformed/non-indexable scoped identities cannot
prove absence. Current Session is read under the transaction and never rewritten.

MB5. One crate-private binder is the sole existing-attempt Session-ID/managed
execution writer. Every ordinary Workflow access mode, generic record writer,
raw supported writer and recovery writer rejects that delta. New attempts start
unbound; repeated same-operation delivery can only return a proved AlreadyBound
fact with no second write/audit. A complete before/after projection check rejects
extra history, detail, context, marker, completion, actor or authority changes.
No broadly reusable record-only write option or caller-selectable access mode is
introduced. Reserved private audit names cannot be forged by generic audit APIs.

MB6. Missing composition refuses before fresh native phase Context/reservation,
marker, preparation helpers, process spawn or input effects. Genuine composition
availability is implementation-owned and absent by default, not inferred from
CLI presence. Config/FakeAgent cannot enable it. The final profile preserves
official authentication, settings, required hooks, permissions, policy, protected
base and source checks. No ownership sampling extension, outer sandbox, root,
VM, container or credential copying is introduced. rururunx is not a security
sandbox; cleanup remains best effort and separate from work outcome.

MB7. A post-marker binding refusal preserves the exact reservation and actual
owned invocation. It cannot fail the Task, release/retry/transfer, recapture
authority, send another input, stop another Task, or fabricate completion.
Storage contention/uncertain commit produces a typed deferred factual binding
handled by the actual retained invocation and authoritative Driver, independently
of a dropped Engine future. Unknown start delivery stays held. No-current-dispatch
and known failure use their separately proved non-success closure, not invented
binding. The successful late path uses the same binder and only a sealed genuine
current allocated/input/success-settlement proof, never a caller receipt ID.

For this managed profile, success-settlement means the actual private allocated
phase owner and prepared/admitted/actually consumed input, the exact correlated
current owned Native terminal with known successful work under the original
marker/instance, and recorded logical native-terminal/input-settlement evidence.
It is neither a physical-death/full-cleanup proof nor an accepted Workflow success.
Runtime-only result finalization may remain open for genuine later capture; it
grants no further native input/effects. This explicitly supersedes the legacy
late-binding physical full-settlement and unknown-cleanup refusal for this profile
only. Cleanup Unknown/Leftovers alone neither creates authority nor blocks an
otherwise eligible factual late binding or independent progress. Unknown work,
HistoricalDraft or ambiguous/missing input/current proof, stale/restored authority
and irreversible external-operation uncertainty keep their holds. Genuine result
capture/publication and success closure remain separately required; the terminal
receipt does not certify them.

MB8. Correctness does not depend on a notification edge: reconciliation readiness
is durable before/with settlement, rechecked on Driver registration/wake/return
and after owned start ends (normal, error, timeout, abort/drop or cancel). Finite
fair timer fallback covers lost/full/closed notifications without busy polling.
Passive status/poll observers do not bind or acquire ownership. Restart uses the
actual reviewed fencing/restore protocol; a durable row cannot reconstruct a
live owner. Unsupported/unknown recovery stays held and visible. Duplicate and
delayed normal/late delivery produce one exact binding/audit.

MB9. Private current-successor ledger and separately authorized bound-live
diagnostic/gate/closure ports retain original source/actor/lock/native currency.
Binding never authorizes their wider deltas. Preserve Design10's reserved-key
protection, immutable chain, finite per-class allowances and mandatory closure
capacity. Optional diagnostics cannot consume mandatory closure reserve or make
first binding tolerate Workflow drift. Full body planning/hashing occurs outside
the Store mutex; publication has no await/filesystem/native operation and enforces
explicit finite owner/body/lock/identity/ledger cost bounds. Bounds are admission
conditions, not truncation or a weakened empty-set proof.

SourceRecovery7's exact Workflow pins and Runtime9's future Driver pins must
recognize only a genuine private permitted successor anchored at their existing
immutable authority. Binding cannot update those authority rows, call the generic
after_write refresh, or silently recapture current pins to avoid a conflict.
The design must inventory every native live wait/status arm, including quota
Task projections and pre-Session NativeStart::Waiting: they are not automatically
the legacy detail-only diagnostic delta. Any legitimate state-changing port needs
its own exact typed authority/write contract; factual binding grants none.

## Acceptance and evidence

| ID | Required actual consumer evidence |
| --- | --- |
| MB-AC1 | Engine + actual private #19 producer + registered native invocation holds a turn across marker/start/bind. Exact Task/P/G/Session/full locks stay unchanged; one Workflow/audit advance; real current callback/completion remains eligible. |
| MB-AC2 | Separate SQLite writer changes each P/G/T/Workflow/context/lock predicate during held start, including same-version raw body/REPLACE and added/deleted locks. Binding refuses with no binding/audit writes or new native input. |
| MB-AC3 | Foreign/missing/Lost/ambiguous Session, wrong actor/provider/role/worktree/native UUID/private owner/input pair, extra projection and generic writer attempts all refuse at the actual consumer. Passing producer prerequisites precede each negative. |
| MB-AC4 | Actual owned successful settlement before lost start delivery; normal/late race, deferred commit, dropped Engine future, lost/full notifications and actual Driver wake converge once. Authentic restart preserves required fencing; rows alone never grant admission. |
| MB-AC4.a | Normal returned Starting before input/ACK binds its genuine prepared owner without delivery/success claims. The same NotDispatched owner cannot late-bind. Actual current consumed successful terminal with cleanup Unknown/Leftovers can late-bind once; Unknown work, HistoricalDraft, wrong input/marker/instance and ambiguous external outcome cannot. |
| MB-AC5 | Genuine bind through diagnostic/gate/success and failure closure; exhausted optional allowance still permits mandatory closure; malformed/disconnected/replayed ledger rejects without effects. |
| MB-AC6 | Migrate positive Workflow fixtures through actual producer ports. Capability-only advertisers remain negative. No disabled tests, fake owner/pair, copied authority, fixture exemption or weakened native mechanism. |
| MB-AC7 | Compile clean committed mutations restoring Task bump, refreshing captured frame, dropping each CAS/identity/private-proof/sole-writer/delta/ledger check or inferring success. Each fails at its intended actual consumer assertion; exact source restoration passes. |
| MB-AC8 | Independent immutable requirements/design/source reviews; composed current-main fmt/Clippy/build/full Workflow regressions and macOS/Ubuntu CI. Controlled protocol peers and authenticated CLI/hook/four-Task qualification are reported separately. |

## Delivery and open gates

Requirements approval precedes managed binding design and production changes.
The design must specify actual allocation/frame producers, every writer/consumer,
schema/migration composition, finite cost classes, typed failure/reconciliation
states and current Driver integration. Native readiness remains unavailable until
those real ports compose; an isolated binder test is not a completed delivery.

The legacy approved private #19 ports and this managed profile are not declared
equivalent. An implementation must either compose the actual #19 producer or
deliver its complete prepared-owner/input protocol through the real managed
adapter and all protected writers, with explicit joint design/source review and
the above causal controls. Renaming Unit/Invocation DTOs is insufficient. The
normal/late/diagnostic/recovery requirements remain open until their actual
consumers are verified. No README, license or whole-MVP claim is changed here.


## INLINE INPUT FULL-BINDING-HOW
{"id": "FULL-BINDING-HOW", "bytes": 58938, "sha256": "f03efbb1b2ab7986936cac72be5580c939f31c223a4178c74d0a5d5e79dd4d00", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "doc/design/issue-43-managed-binding-design.md", "blob": "cf19dcc12cf2d0410727a457e5e4d2eb5625f399"}
# Issue 43: managed native Session binding design

Risk: STRICT. Status: component design approved by independent Root and C reviews
of `cfd1800f8801fecfa47854fbaa0b08ec1f377985`; see the
[review evidence](../verification/issue-43-managed-binding-design-review.json).
Requirements approved at
`7e47421a24225583795245b307e574fb3156dd7e`, with the approval checkpoint at
`079f5fd37184240c08fbca3859ec6a0ac78ae139`. No production implementation,
native Driver readiness, migration qualification or authenticated CLI acceptance
is delivered by this document.

Normative requirements: [managed binding](../requirements/issue-43-managed-binding-requirements.md).
Inherited contracts: [Requirements9](https://github.com/shuhei-suzuki/rururunx/blob/b191b466d5dea303585cfaf6968c6fb178be79cd/doc/requirements/issue-43-requirements.md)
and [Design10 mechanics](https://github.com/shuhei-suzuki/rururunx/blob/b191b466d5dea303585cfaf6968c6fb178be79cd/doc/design/issue-43-binding-mechanics.md).
The factual binding, original frame, sole writer, complete ledger, negative identity
checks and finite cost contracts are preserved. The managed requirements explicitly
adapt reviewer worktrees and late logical settlement; they do not make current
Unit, NativeInvocation or NativeResult DTOs equivalent to the private #19 protocol.

## 1. Actual baseline and delivery boundary

The source inspected here is immutable
`802764f03893d0ee8e08ee19045ccb333387ad9b` (schema8). Native6 supplies genuine owned
stdio acquisition and result receipts; SourceRecovery7 supplies current retained
source recovery; Verifier8 supplies command-only Tests receipts and private acceptance.
None supplies the missing complete phase owner/prepared-input protocol or binder.

At that source, Workflow's managed Launched arm sets Session/execution, refreshes
owners, performs ordinary persistence and only then disarms its preparation guard
(`workflow.rs:1653–1657`). Ordinary persistence writes Task before Workflow
(`state/mod.rs:905–910`). A save refusal can drop the preparation guard for an
already launched invocation. Pre-Session Waiting and due polling also persist
Task/Workflow and re-enter preparation (`workflow.rs:1679–1695,1751–1818`).
Those are actual replacement sites, not valid record-only binding producers.

Native's Session registration is after readonly Git/version helpers and quota
admission (`execution/native.rs:185–342,353–427`). Its private NativeSeed binds
Context/payload to Unit/Session, but has no original Workflow/full-lock frame.
`reserve_native_input` has real before-wire intent and current Context checks
(`state/execution/native_results.rs:257–328`); that is retained and extended,
not treated as an already complete phase admission credential.

This component must deliver the complete new managed private protocol through
the real selected adapter and all protected writers. The current legacy #19
allocated owner and the new owner remain different types. There is no public
conversion, receipt-to-owner constructor, SQL fixture capability or shared fake
authority. Account-free peers exercise the real production producer paths.

Migration order is **Verifier8 → actual Runtime9 → Binding10 → ReviewRound11**.
Runtime9 is under development, not inferred from a schema number or the current
read-only controller. Binding10 DDL/guards are installed only after joint protocol
design/source review and composition with actual9. The previously fixed Review
draft remains immutable; its later allocation update uses11. Native-ready Driver
is a separate actual private consumer gate, described below.

## 2. Types, allocation and original marker

All authority types below are crate-private, non-Deserialize, non-publicly
constructible and have private fields. Allocation/launch/consumption ownership
handles are non-Clone. Scalar IDs and diagnostic read DTOs are freely readable
but cannot instantiate a proof. Store code validates current rows even when a
genuine proof is supplied. Type names in this document specify new ports, not
APIs present at the baseline.

| Type | Sole actual producer and permitted consumer |
| --- | --- |
| ManagedNativeComposition | Registry installation after real Claude/Codex vtable, private protocol, sole writers, Driver retention and migration guards compose; Workflow preflight only |
| OriginalMarker | Dispatch-marker Immediate transaction; immutable original frame and operation identity used by every subsequent phase port |
| PhaseSessionOwner | That marker transaction allocates exact Session ID to the actual selected NativeSessions/Unit/phase; readiness is completed only by the genuine Session registration transaction |
| PreparedPhaseInput | Marker transaction accepts the Store-validated actual committed Context/input plan; actual Native preparation and wire builder validate this same pair |
| PhaseLaunch | Marker producer hands the exact owner/input pair to an independently retained managed phase supervisor and actual adapter start; cannot be reminted from ManagedInput |
| ValidatedPhaseSession | Actual Native register transaction after qualification, before owned child spawn; binds the allocated Session and NativeInvocation to the pair |
| ConsumedPhaseInput | Actual `reserve_native_input` transaction consumes the one-shot private input admission; represents recorded consumption/intent, not OS delivery |
| OwnedPhaseSettlement | Actual owned terminal transaction after exact consumed input and acknowledged invocation/result checks; late factual binding only |
| CurrentWorkflowSuccessor | Store-owned coherent complete chain/body planner; current currency for specifically enumerated ports, never native grant authority by itself |
| DeferredBinding | Actual retained supervisor with original owner/operation; exact retry/reconciliation, no new dispatch |

### 2.1 Preflight and prepared namespace

Workflow checks implementation-owned composition before hold clearance, a fresh
phase Context, reservation, Unit preparation, marker, helper or spawn. Missing
composition returns typed UnsupportedComposition with no effects. Capability
metadata, CLI existence, configuration, FakeAgent and an owned-looking result
cannot enable it. Standalone Native6 remains a separate profile; a Workflow scope
protected by Binding10 cannot bypass preflight through that entry.

Actual registered adapter selection fixes alias, provider, native role, model/effort
and supported protocol profile. A later registry lookup cannot substitute a vtable.
Initial Source preparation and fresh Unit reservation use their existing exact
private source/Worktree contracts. Executor uses its actual fresh Unit namespace;
reviewer uses the genuinely prepared immutable-source readonly snapshot with
separate output namespace, artifact/profile and pre/post qualification. Task's
executor worktree is not used as a reviewer identity. No new sandbox, user, VM,
container, root requirement, HOME change or credential handling is introduced.
Required hooks/settings remain qualified prerequisites; unsupported profiles refuse.
The actual Driver checks this preflight before calling initial
ManagedWorkflowSources::prepare as well as before Workflow::step. Source preparation
or a public registry lookup alone cannot enable native dispatch. Protected Task/
Workflow activation routes require the same private composition; standalone source
fixtures are not evidence of that route. No supported native entry may hide an
effectful probe before this static implementation check.

### 2.2 Marker transaction and pre-Session allocation

Outside the Store mutex, the actual supervisor plans bounded canonical bodies,
input bytes and complete scoped lock inventory from a coherent read snapshot.
The marker Immediate transaction checks the exact current snapshots, Unit/profile,
Context/input source and complete lock set; applies only the authorized initial
dispatch-marker Task/Workflow transition; allocates the phase owner/input pair;
reserves the operation's mandatory audit/quota budget; and commits OriginalMarker.
The owner exists **before all Native start helpers**, not only when Session is
registered later. It reserves a new exact Session ID, selected adapter origin,
actual prepared Unit/path and input identity. It cannot bind before real Session
registration, send input or represent successful work while merely allocated.
The plan fixes exact post-marker encoded Task/Workflow bytes, version increments
and timestamps before Immediate. The transaction publishes those planned bytes
under exact old-body/version CAS, then records the already planned resulting
frame; it does not hash a newly reread full body inside the lock. Source7's
separately authorized pre-marker advance similarly consumes an exact sealed
pre/post plan and current row CAS, rather than generic recapture after a write.

OriginalMarker contains the resulting **post-marker** P/G/T versions and exact
body digests; Workflow Record ID/version/complete canonical body digest; scope;
active attempt index/phase/actor and Workflow generation; state-root instance and
current epoch; Unit ID/execution generation/profile/path; selected provider/alias/
role; allocated Session ID; exact launch/latest Context versions/digests, revision,
source map, artifact/governing/instruction identities and prepared payload digest;
and the complete sorted scoped WorktreeLock ID/version/body-digest set. Array
order, inactive history, budgets, retries, detail, completion and all other Workflow
fields are included in the complete body hash. Empty lock sets require a complete
query, not a truncated scan. Full scoped means the existing owning Task scope's
entire WorktreeLock set, without selecting only one visible lock.

No later refresh_owners, returned Session, Driver snapshot, pause, helper outcome
or current read can replace these pins. The immutable input pair points to exact
Context/payload bytes and the selected native wire profile; dynamic owned thread/
turn identifiers join the admitted wire receipt later, without replacing input.
Actual start receives the private PhaseLaunch, not a public tuple of IDs.

### 2.3 Preparation, registration and one-shot input

The actual NativeSessions start path validates PhaseLaunch before readonly Git,
version and other scoped helper intents. Every helper remains registered to the
same Unit before effects and rechecks original currency, epoch and cancellation.
Resource admission gate coverage remains draft → Unit registration → lease reserve;
it is released before materialization, Git, child wait or command await. Neither
that gate nor a SQLite mutex spans wire IO. Registration uses the originally
allocated Session ID, not a fresh UUID generated after helpers.

`register_native_session` gains a private managed branch that, in one Immediate
transaction, checks OriginalMarker, current full owners/locks, actual prepared
namespace/profile, selected adapter and exact input pair; writes the Starting
Session/session_units and NativeInvocation; and records the pair as validated.
Its Unit/Session/Context/receipt protections and Verifier8 command-only refusal
remain. Merely copying a NativeSeed or allocating a public ExecutionAuthority
cannot enter this branch. It must refuse before child spawn on stale currency.

The actual Core owns the validated pair. Codex initialize/thread/turn and Claude
stream initiation keep their real bounded raw protocol checks. Before input wire
bytes, `reserve_native_input` atomically validates the original frame plus permitted
factual Workflow successors, actual latest Context pointer/full payload, Session,
Unit generation/epoch and one-shot pair; records exact frame hash, Native6 input
effect ID and owned thread/turn metadata; and consumes admission once. Permissions,
hooks and later transport journals cannot change the consumed input row. ALLOW
and delegated effects also require original currency and real current own Session;
DENY/revocation retains its separate nongrant authority. Admission is a before-wire
authorization, not an atomic transaction with the OS or a delivery guarantee.
Cancel races are ordered at actual private admission; already admitted partial
wire delivery remains uncertain and is never replayed automatically.

Confirmed delivery/ACK update only their own typed evidence, retaining immutable
pair/frame/effect identity. A partial write, lost ACK, conflicting thread/turn,
duplicate changed input or stale epoch cannot be converted to NotDispatched.
Actual Native6 collector and terminal projection remain the producers of owned
work and retained answer content; text/exit0 alone does not authorize binding or
Review approval.

### 2.4 Actual adapter and closure seams

Registry's actual Claude/Codex installation keeps a crate-private ManagedNativePort
backed by the same NativeSessions instance as the selected public adapter. Its
start_phase consumes PhaseLaunch and returns an owned invocation channel to the
supervisor; its control/settlement ports address only that allocation. Private
descriptor origin includes the selected registry instance and actual provider
vtable. A caller cannot ask any public Adapter or another alias to adopt it.
Public start_managed/SessionRef paths detect the protected contract and refuse
without the matching private route; their readable DTOs are not alternate grants.
Legacy Codex, GenericCLI, Grok and Fake implementations do not expose this new
port. Interactive/standalone entry cannot attach to a protected phase owner.

The actual marker producer calls NativeSessions' private allocation factory before
its commit and checks selected origin in the transaction; the factory's provisional
handle is unusable until that exact marker commits. The real registration branch
validates the allocated handle and consumes the preparation handle once. On commit
uncertainty neither producer retries a new UUID: the supervisor reads the unique
operation/allocation/Session fact before proceeding. Actual terminal processing
hands its existing private NativeTerminal correlation to the logical settlement
producer in the same native-result/Session transaction. The binder never constructs
that terminal proof. A typed non-success producer can close an actual allocation
with no current dispatch or known owned failure; it cannot mint late success.

## 3. Durable schema and writer guards

Binding10 adds the following private tables and reserved indices on existing audit.
Scope/identity columns are indexed
and checked against strict bounded body decoding. Maximums are complete encoded
bytes, not selected fields. Foreign keys and uniqueness bind to existing actual
Unit, Context, Session/native invocation and Workflow identities. Diagnostic read
APIs expose records, never a handle with dispatch/binding authority.

| Table | Immutable core / mutable typed facts / limits |
| --- | --- |
| workflow_native_contracts | Actual activation Workflow/scope/profile/epoch origin; legacy-held or composed contract; never configuration-synthesized; one per Workflow, ≤4096 bytes |
| managed_phase_operations | OriginalMarker, pair/owner IDs, allocated Session, original Workflow body hash/reference and reserved allowance; immutable; phase-open/closure and supervisor readiness facts typed separately; one per marker, operation metadata≤4 MiB |
| managed_marker_bodies | One immutable original complete encoded Workflow body≤8 MiB per operation; referenced digest/index; full extraction outside Store, no extra full body read in binder Immediate |
| managed_phase_owners | Operation/Unit/epoch/generation/provider/alias/role/actual path/Session origin; immutable; actual registration readiness/NativeInvocation correlation typed; one per operation and unique Session, body≤2 MiB |
| managed_phase_inputs | Exact preparation Context/template/profile/source pins; immutable; one per owner, body≤2 MiB; payload itself≤1 MiB and original existing frame budgets still apply |
| managed_phase_admissions | One actual consumed input intent per pair, exact Native6 effect/frame/owned thread-turn binding; immutable core, separate confirmed/uncertain/settled status; body≤8192 bytes |
| managed_phase_readiness | One actual supervisor-owned bounded row per operation with immutable origin, current parking/reconcile state and monotonic start-ended/known-terminal observations; ≤4096 complete bytes; no unbounded notification history or reconstructed authority |
| existing audit: reserved factual links | ONE immutable audit row is the link; expression indices enforce operation/local ordinal/predecessor identity; ≤4096 bytes/link, ≤256 links; no second mirror-table write |
| scoped_session_identities | All scoped Session/provider/native UUID identity records, including Lost/history; actual Session writer transaction maintains indexed identity/malformed flag; metadata≤4096 bytes/entry |

Existing Native6 NativeInvocation, results and effects remain canonical native
transport/answer evidence. These tables do not create an independent owned result
or mutable copied authority. A terminal correlation references the exact existing
receipt/version/hash; HistoricalDraft cannot be upgraded. Persisted owner rows
alone cannot reconstruct the in-memory producer/private allocator. Actual activation
protects fresh Workflow native attempts before a Unit or Session exists, following
Verifier8's independent managed-contract principle.

All mutable tables participate in connection-local schema contract10 guards;
cached older connections cannot write by using raw SQL. All generic Record,
Session, audit, WorkflowAccess and recovery writes reject an existing protected
attempt's session_id/execution None→Some or change. New attempts start both absent.
The binder alone uses a narrowly private fixed SQL/typed projection path, not a
caller-selectable RecordOnly access mode. Paired Session/execution must be absent
or exact together; malformed partial tuples reject. Existing Unit reference is
already fixed before marker and is not changed by binding.

Guards reject same-version body mutation and existing-key INSERT/REPLACE on
protected Workflow Records and all reserved audit/operation/link identities;
reserved append-only UPDATE/DELETE and conflicting predecessor/sequence also
reject with recursive_triggers OFF. Generic audit APIs cannot use compiled private
names. Trigger CHECKs bound actual OLD/NEW complete bodies. Connection-local
private port tokens authorize only a derived exact delta and are scoped/reset for
that transaction, never reusable public writer permissions. Raw SQL by arbitrary
applications or the same OS user is outside a security guarantee; supported rrx
writers and cached rrx binaries must fail closed.

The fresh/migrated10 transaction verifies actual9 namespace/layout, installs
all10 tables/guards atomically and updates schema only after success. Collision,
partial or wrong9 layout leaves9 untouched. Existing active operations become
LegacyHeld; migration cannot manufacture owners, consumed input or OriginalMarker.
Newopen9 binaries reject10; already-open9 APIs and raw cached writers reject every
new/old mutable table. Newopen10 and cached10 similarly reject actual11 after
Review migration composition. The actual8→9→10→11 matrix, concurrent initializers,
fresh namespace and pre-open writers require compiled controls. Installing10 is
not evidence that operational Runtime9/Driver or Review11 is implemented.

### 3.1 Keys, state facts and transaction tokens

Operations have primary key operation_id, unique
(workflow_id, workflow_generation, attempt_index), explicit scoped P/G/T columns,
owner_epoch, Unit/execution_generation and allocated Session ID; the full immutable
marker metadata and digest agree with those columns. Owner and preparation IDs
are unique foreign keys to that operation; allocation has unique Session ID and
Unit+phase-operation identity. Registration adds only its exact existing invocation
ID and validated fact once. Admissions have unique pair_id, Native6 input effect
ID and invocation ID, with foreign keys to all three actual producers. Confirmed
and settled facts are monotonic and cannot replace frame/thread/turn/pair identity.
Current NoCurrentDispatch parking and consumed admission are mutually exclusive;
successful exact due-claim/registration invalidates the old parking claim. A retained
historical parked fact cannot authorize a later resume or consumption. Session index
keys include Session ID, scope, provider, nullable native_ref and an explicit
malformed flag; no unique(provider,native_ref) constraint is used to hide historical
conflicts. Complete scoped negative queries read all distinct matching identities.

The negative lookup reads the protected identity projection/index, not 4096 full
Session recovery bodies. Actual Session writers validate bounded body→projection
agreement atomically; raw supported Session writes without that exact projection
reject or mark the scope non-indexable. Legacy indexing validates complete bodies
in bounded pages and produces no grant while incomplete/oversized/malformed;
there is no truncated-success certificate. Fresh negative queries check indexed
record IDs/versions and invalidity flags; own latest Session's full body is still
read and checked inside Immediate. The final migration must declare/instrument
its separate total byte/row bound, refusing qualification rather than loading an
unbounded historical corpus.

The existing audit.sequence is the globally unique audit identity. A reserved
link's local ordinal (1..256) derives from consecutive Workflow versions relative
to OriginalMarker, without adding a new field to Design10's payload. Expression
indices on compiled factual kinds enforce unique operation+Workflow-after-version
and operation+prior_ledger_digest. Links carry original marker digest, Workflow
versions/body digests before/after and their exact typed payload. They enforce a
single append-only chain; ordinal1 predecessor is OriginalMarker, not
an arbitrary current audit. SQL before-insert checks reject replacement conflicts
before deletion regardless of recursive_triggers. No separately mutable ledger
head/count is written by binding: sequence/head/allowance usage derives from the
bounded immutable indexed chain. Operation closure/budget-release facts belong
to the separate phase_closed transaction, not binder's write set.

Store plans a PrivateWritePermit containing exact old/new encoded body, row IDs,
versions and audit payload for one port. The connection-local guard checks exact
planned bytes and metadata rather than accepting a broad access-mode name. The
permit is installed only by crate-private Store code for that transaction and
cleared on commit/rollback/error; callbacks perform no nested DB or filesystem
reads. Supported public writer APIs cannot construct a permit. Owner P/G/T,
Context and lock checks compare exact bounded encoded bodies/version inventories
against the sealed coherent read plan; same-version mutation and index/body
inconsistency refuse. No in-lock rehash is needed to grant currency.

The native_diagnostic payload is exactly the Design10 header: kind, scoped
P/G/T/Workflow IDs, Workflow generation/attempt/phase, own Session, private
operation, OriginalMarker digest, Workflow versions/body digests before/after,
prior ledger digest, canonical body recipe, allowlisted reason, detail≤128 UTF-8
bytes, Context version and existing audit timestamp. Complete encoding≤4096 bytes.
It carries no raw CLI error, credentials, argv/environment, grants or resource list.
Other port schemas use that same predecessor header plus only their enumerated
typed projection. The final DDL and writer CHECKs must be reviewed together with
the actual producer; table presence or a token function name alone is insufficient.

## 4. Record-only binder

The sole private `bind_managed_phase` consumer accepts an actual normal-return
ValidatedPhaseSession or actual OwnedPhaseSettlement, OriginalMarker and a Store
sealed plan. No caller constructs this argument from SessionRef, receipt ID,
execution status, NativeStart DTO or arbitrary JSON. Normal and late use the same
transaction and exact projection; only the private eligibility predicate differs.

Full canonical planning/ledger extraction occurs in a coherent snapshot outside
held SharedStore. Recursive object keys sort by UTF-8 byte order, arrays retain
order and compact JSON retains exact validated value semantics. Domains are
`rrx.workflow-body-sha256/v1\0` and `rrx.workflow-ledger-sha256/v1\0` as Design10;
the literal final NUL is part of hashing. Typed envelopes include canonicalization
version. Original captured Record body bytes also remain retained, so an encoded
body mutation cannot evade the current version guards.

The actual Store-owned planner uses a separate read-only/query-only connection
to the retained owner's canonical selected state database, with a coherent read
transaction; it does not call RuntimeOwner::open or increment epoch. Read snapshot
creation/decoding and body/chain planning happen before acquiring the writer Store
mutex. Private origin/current-index checks qualify its result; publication compares
exact current metadata/body bytes under Immediate. Reading unrelated rows in
separate unlocked calls is not a coherent snapshot. Native original-currency and
Source7 private validation receive this same sealed plan and check its exact head
inside their own effect transaction; they do not recompute full body hashes inside
the mutex. A changed head requires a fresh genuine successor proof before the
still-unconsumed effect, not fresh original pins or a replayed wire effect.

The Immediate transaction checks:

1. Exact current state-root instance/epoch, operation/phase-open identity, Unit/
   generation/profile and original allocated/prepared/actual registered pair.
2. Exact original current P/G/T identities, versions and full bounded bodies,
   active owner states, Context/current pointer/source/revision/governing/instruction,
   complete scoped locks and active attempt identity. Unrelated parent version
   advancement is not exempted. No current-pins refresh is performed.
3. For **first binding**, current Workflow version/body equals the original
   post-marker frame, both fields are absent and there are zero factual links.
   Diagnostic/gate/bookkeeping before first binding cannot become a predecessor.
4. Latest actual own Session matches allocated ID/scope/actor/alias/provider/role/
   prepared Unit worktree/model/effort and immutable native origin. Returned
   native_ref, when Some, equals latest. Starting PID/native UUID/lifecycle may
   legitimately advance, so no stale returned Session-version CAS is imposed.
   Missing/Lost/foreign/ambiguous Session refuses.
5. A complete scoped identity query proves no distinct Session with the same
   provider/native UUID, including Lost/history. All body/index identities agree;
   malformed/non-indexable entries or scan overflow refuse, never count as absence.
6. Normal eligibility requires genuine prepared+validated registration, even
   before input/ACK; it does not manufacture consumption or delivery. Late
   eligibility additionally requires actual consumed pair, current owned successful
   terminal and logical input/terminal settlement at the same original epoch.
   If terminal publication won the race with normal delivery, the successful
   terminal must pass this genuine settlement predicate too. Known Failure/Unknown
   follows its non-success/held path; a prepared normal handle cannot bypass it.
7. The planned before/after complete Workflow projection differs **only** at this
   active attempt's absent session_id and execution reference. Execution is derived
   from the exact private Unit/Session relation, not caller authority. Record
   version/updated_at are the only changed Record metadata.

It writes the Workflow Record directly with CAS and ONE reserved session_bound
audit/link in the same transaction. No Task, P/G, Session, Context, WorktreeLock,
owner, count, readiness, SourceRecovery or Driver authority row is updated. Generic
workflow.saved/gate events are suppressed for this port. Audit retains Design10's
exact field list below; Unit/execution is bound through the private operation and
hashed complete Workflow, not an extra public grant. No provider payload,
credentials or process list is copied.
Readiness acknowledgement and supervisor progress are later typed facts, independent
of factual binding's atomic write set.

The binding payload fields are exactly: kind=session_bound, project_id, goal_id,
task_id, workflow_id, generation, attempt_index, phase, session_id, provider,
actor, role, workflow_version_before/after, task_version_preserved,
dispatch_started=true, marker_identity (scope/Workflow/captured version/generation/
index/Context version), context_version, proof_source (normal_return or
closed_settlement), private_operation_ref, private_receipt_ref (absent normal;
exact genuine settlement late), original_marker_frame_sha256,
workflow_body_sha256_before/after, prior_ledger_digest,
canonical_body_recipe=rrx.workflow-body-sha256/v1, and existing audit timestamp.
The compiled kind is rrx.private.workflow.session_bound. Complete encoding≤4096
bytes includes keys/envelopes. The link digest hashes the complete immutable
payload and excludes its own digest. There is no duplicate ledger-row insert.

AlreadyBound requires the same immutable operation/pair and exact committed
session_bound link/body/derived tuple, plus a complete proved current permitted
successor. It performs no write, audit or new input. A different operation, partial
tuple, absent audit, matching current IDs without proof or foreign suffix refuses.
Normal/late races converge at this check. A binding refusal leaves the exact live
reservation/invocation held; it never calls ordinary fail/retry/release/refresh.

## 5. Retained supervision, parking and readiness

### 5.1 Ownership handoff before any launch result

A real `ManagedPhaseSupervisor`, retained by RuntimeOwner/actual Driver, receives
PhaseLaunch and the preparation guard **before** adapter start. Engine awaits a
result from this owned invocation; it does not own the only start future or armed
guard after marker. The supervisor retains exact selected adapter, owner/pair,
original frame and invocation outcome across Engine timeout/drop/abort. Actual
Launched handoff transfers/disarms preparation guard before any binding error can
unwind; Core owns the process/terminal proof and supervisor owns factual reconciliation.
It never detaches an unobserved helper or turns an unknown start into a new start.

Actual Core terminal processing continues to preserve observed work and durable
receipt on storage retry, projects current Session coherently, and uses unreaped
OwnedProcess identity for group hygiene before reaping. Binder retry does not
kill Core, discard known successful terminal or send input. Core's existing saved
private terminal proof can retry result persistence; that is distinct from passive
status binding. Process/cookie/Docker cleanup stays best effort and cannot certify
logical result, native death or Review approval.

### 5.2 Pre-Session NoCurrentDispatch and quota parking

Version/readonly helpers can run before quota admits a Session. Only the actual
supervisor with the original PhaseLaunch and actual helper settlement can produce
`NoCurrentDispatch`: no Session registration, no native child start/input/ALLOW,
no unresolved helper/delegated effect and no consumed pair. Merely Session None,
an error string or a public Waiting DTO does not prove this state.

The typed parking transaction changes only existing Unit/quota waiter/lease facts
and bounded supervisor readiness. It does **not** write Task or Workflow, append
a factual Workflow link, clear dispatch_started, recapture marker/pins or relinquish
the original operation. Actual Driver and Workflow status derive effective quota/
capacity waiting from these facts; stored Task remains at its original marker
state/version. Usage classification and exact bucket/probe/fair waiter controls
remain the actual quota producer's responsibility. Due-claim consumes the exact
Unit/waiter/version/due record once, revalidates OriginalMarker and resumes only
the retained not-yet-dispatched launch. Helpers are not replayed when unsettled.
No new Unit, Context, Session owner or PhaseAttempt is created by this resume.

This replaces the current pre-Session persist/re-entry branch. A missing retained
producer, stale original owners/locks or unknown helper/start outcome parks Held
with attention; it cannot use generic pause/retry to mint a new marker. Proven
non-success closure can later close this original operation and allow an explicit
fresh attempt, with fresh worktree/resource semantics retained.

### 5.3 Binding deferred states and active Driver

Durable readiness is derived from phase-open marked/unbound operations plus
actual allocation/registration/input/terminal facts and typed start-ended observations.
It is present before/with successful logical settlement, not created solely by
the caller's notification. Supervisor registration precedes launch. Completion
(normal/error/drop/timeout/cancel), Driver registration/wake and explicit active
reconcile recheck this level predicate. Wake messages carry only bounded IDs and
are hints. Lost/full/closed channels cannot remove readiness.

Transient BUSY/LOCKED with confirmed rollback yields DeferredBinding retaining
the exact invocation. Uncertain commit yields CommitUncertain: read exact Workflow
and unique reserved link to prove AlreadyBound or confirmed rollback before retry.
Constraint/schema/proof/bounds errors are not contention. They park Held with a
bounded reason and no new grant. Durable terminal persistence retries retain the
actual private known-terminal proof, never infer success from stored text.

The **actual Runtime9 native-ready Driver integration port** must retain these
supervisors independently of Engine futures, provide bounded fair per-Project
capacity and enumerate ≤64 pending operations/page with a stable fair cursor.
It has finite fallback timers (100 ms–5 s capped backoff) and retries only predicate
change/actual due time; unchanged Held predicates are not a busy write loop.
Passive status/poll cannot bind, claim a launch or reconstruct ownership. Native
availability remains refused until this genuine Driver path exists and is tested;
read-only ResolveProject, accepted Goal metadata and a named Driver DTO do not
satisfy it. Pending queue capacity is reserved before marker, not after losing a
launch result. Same-Project fairness is tested independently of global quota.

| Actual operation observation | Allowed next action, without authority recapture |
| --- | --- |
| Allocated / start retained | Genuine same supervisor prepares or qualifies helpers; otherwise Held |
| Proved NoCurrentDispatch / due wait | Park original pair; exact due-claim can resume once after original currency revalidation |
| Registered / normal identity available | Binder normal path, or defer/hold preserving actual invocation |
| Input intent consumed / outcome uncertain | No redispatch or late success; real collector/own proof may resolve, otherwise Held |
| Owned known successful logical settlement | Same binder late predicate, then separately genuine result/gate closure |
| Owned known failure / proved no dispatch failure | Typed non-success closure; absent binding remains absent |
| CommitUncertain | Exact original Workflow/audit read proves prior commit or rollback before retry |
| Trusted terminal decision / revoked | No further grant/binder/gate; actual nongrant stop/eligible TerminalRecovery only |
| New epoch / missing actual producer | Held visible; authentic factual-only restore gate or explicit qualified fresh closure/retry |

Readiness row state updates are bounded current observations, not a copied grant or
unbounded per-notification journal. They cannot replace original marker, immutable
owner/pair or native evidence. Terminal proof retention belongs to the actual actor;
status cannot read a row and recreate it. Driver never steals a different supervisor's
operation, and cancellation targets exact owned operation/Unit, preserving siblings.

## 6. Late settlement, closure and restart

Successful late binding requires the same real allocated/validated pair and
consumed admission; exact current owned Native6 successful terminal with own
Session/Unit/input/ACK/frame; and recorded logical terminal/input settlement.
Native effects are closed. Result-finalization may remain open for the genuine
Runtime capture; binding neither closes it nor publishes an artifact. Cleanup
Unknown/Leftovers alone does not block or authorize this path. Result capture and
immutable artifact acceptance remain separate gated consumers. HistoricalDraft,
Unknown work, NotDispatched pair, stale instance/epoch, ambiguous external effect
or partial/uncertain input remains held. Known Failure uses genuine non-success
closure without inventing absent Session binding or an approved result.

Task-terminal cancel/fail_task first performs its actual trusted lifecycle
revocation even if factual bookkeeping is unprovable. The typed terminal_decision
port validates the original chain and authorized Task-before/after lifecycle delta,
writes that decision plus one reserved factual link, and prohibits subsequent
binding/diagnostic/gate admission. Revocation is not native permission and does
not forgive source/Workflow drift. If its chain cannot be proved, real stop/fence
remains available while bookkeeping is held; cancellation cannot launder drift.

`phase_closed` consumes a genuine successful captured-result/readonly/gate proof,
known non-success/no-current-dispatch proof, or eligible TerminalRecovery decision,
with complete prefix and all logical applicable effects settled. The corresponding
typed port enumerates exact Task/Workflow/Context/artifact changes and releases
unspent ledger reservation atomically. It does not require proving process death;
uncertain irreversible external effects still hold. Destructive namespace cleanup
waits for genuine capture/finalization/draft retention; physical cleanup jobs may
continue afterwards without changing work. Closed unbound failures retain Session
None. Engine::fail is non-success phase closure/awaiting_explicit_retry, not an
invented Task-terminal decision. Unknown work stays visible for explicit recovery.

Restart currently calls `begin_execution_epoch` (`state/execution.rs:485–557`): it
retains state-root instance, advances epoch, closes Native/finalization permissions,
fences Sessions/effects/leases and invalidates SourceRecovery7. Binding cannot
rewrite OriginalMarker epoch or manufacture a current owner from these rows.
Active/unknown old operations remain Held; an actual fresh retry follows genuine
non-success closure and new worktree/Unit/generation, not a copied input replay.

An across-epoch factual-only restore requires the actual reviewed #14 fencing and
restore producer, with evidence of original allocated/consumed **owned known terminal
committed before fencing**, authentic operation origin, exact unchanged original
P/G/T/W/Context/locks and complete old ledger. A private RestoreFactualBinding must
record original epoch versus the actual fenced recovery epoch, bind no live owner,
open no permissions and send no input. It is not produced by receipt reads or a
Current Runtime ID. This baseline has no such complete port: implementation keeps
that restore Unsupported/Held until its distinct joint source review and causal
controls qualify it. Logical known-work preservation and ordinary historical result
inspection remain available without falsely restoring native authority.

## 7. Factual successor consumers and SourceRecovery7

First binding has zero predecessor links. Thereafter a complete ≤256-link chain
anchors every permitted Workflow change to the immutable OriginalMarker. Store
planning validates all original pins and exact canonical allowed deltas in a
coherent snapshot, producing a sealed CurrentWorkflowSuccessor outside held Store.
Inside Immediate, exact current Workflow version/body identity and latest immutable
link ID/sequence/digest must match the plan. Reserved guards make version/head CAS
meaningful; an arbitrary current body hash or audit tail cannot mint this proof.

| Port / reserved kind | Exact permitted projection and producer |
| --- | --- |
| session_bound | §4 only; actual normal/late private producer, first link |
| native_diagnostic | Bound live status_unavailable or persisted_status_mismatch only: active.detail≤128 UTF-8 bytes, Record version/time and one link; no Task/owner/native wait write |
| gate_claim | Actual private gate invocation: Evaluating + exact claimed_observations; no Session binding/source refresh |
| gate_observed | Actual compact claim outcome + exact Waiting/held disposition fused into ONE link/transaction, including bounded detail/held_reason; repeated identity may change only checked occurrence count/last_time |
| gate_hold | Actual typed hold/clear policy, exact held_reason/detail; no Task WaitingHuman rewrite |
| terminal_decision | Trusted cancel/fail_task action and its separately checked Task lifecycle delta; no success/binding/input |
| phase_closed | Actual eligible typed result/gate/non-success/TerminalRecovery proof and enumerated final phase/body/Task/Context changes; no transport-derived approval |

Native quota live wait/recovery does **not** use native_diagnostic or these Workflow
links. It updates its genuine Unit/pool/waiter/lease facts and derived status only;
no Task/W rewrite occurs while open. Terminal quota/capacity interruption uses
typed non-success phase closure with preserved Unknown work and explicit wait/fresh
retry policy, not live quota as failure. This choice preserves fixed ledger budgets.
Typed derived waiting feeds actual Workflow/Goal/Runtime/CLI status and scheduling;
it is not cosmetic text masking a hidden Task update.

SourceRecovery7 currently compares serialized full snapshot pins exactly
(`source_recovery.rs:286–293`) and generic after_write refreshes its row
(`342–367`). A binder Workflow+1 therefore needs a **new bounded private successor
recognizer** in validate_task/validate_binding/recovered_source_task and the actual
inputs/prepare_pack/registered Git consumers. Its anchor is the existing saved
SourceRecovery Workflow pin at the operation marker, advanced only by the genuine
pre-marker typed reserve/marker transaction. It may resolve that exact anchored
Workflow pin to the chain-proved current Workflow while every other saved source
pin remains exact. It neither mutates row.pins/version nor invokes generic
after_write. Subsequent original operation source authority stays the original
one; the factual resolver is not a new native admission credential.

Runtime9 Driver's actual saved Workflow pin must use the same chain-recognition
contract anchored at its existing private claim, with exact scope/operation/epoch.
If that claim predates marker, the actual marker receipt must prove the separately
authorized exact pre/post marker transition from that saved claim; a current
post-marker snapshot is not substituted for the claim. Source7 may perform its
existing separately authorized pre-marker advance, but binding itself never does.
Binding cannot refresh Driver/Goal authority rows. If Driver9 cannot expose this
genuine consumer, native-ready composition refuses. Current read-only routing does
not supply it. Source7's later ordinary typed publication/closure can advance
Task/Context/artifact pins only through its separately authorized transaction,
after validating the genuine original chain; this is not binder bookkeeping.
Source/body drift or a disconnected chain refuses effects. Trusted revocation
keeps its independent nongrant path and does not adopt changed parent authority.

## 8. Writer and actual consumer replacement inventory

All locations refer to802764f. Ordinary writes remain available for unprotected
legacy scopes and separately authorized closed-operation transitions. Every protected
open native phase writer must enter a named private port or refuse; no catch-all
refresh_owners/persist route remains. Source patches must re-inventory all call sites
against the final composed commit, including new Runtime9/Review11 callers.

| Actual baseline caller | Required protected-profile replacement |
| --- | --- |
| Workflow initialize/reserve/context (`workflow.rs:566–712,832–910,1101–1125`) | Composed activation before native phase effects; ordinary authorized pre-marker Context/Task reserve remains exact; no retroactive owner creation |
| prepare_agent legacy return (`1273–1453`) | No protected-scope use without genuine private protocol; generic/Fake/capability-only legacy entries refuse preflight |
| prepare_managed refresh/marker (`1599–1615`) | Actual private marker/allocation TX; no later refresh of OriginalMarker |
| Launched (`1653–1657`) | Supervisor already owns launch/guard; exact private binder; errors defer/hold instead of retiring launched Unit |
| pre-Session Waiting/error (`1679–1695`) / due claim (`1751–1818`) | §5.2 proved NoCurrentDispatch parking and exact original-operation resume; no ordinary Task/W save or marker re-entry |
| status error (`1854–1855`) / persisted mismatch (`1869–1870`) | Separate bound-only native_diagnostic port; capped reasons, no raw provider error; before binding keep supervisor readiness/attention without W links |
| live quota (`1880–1881`) / recovery (`1949–1950`) | Genuine quota facts + derived WaitingQuota/running status; no Task version/state rewrite |
| terminal quota/capacity (`1911–1912`) / Unknown (`1923–1924`) | Actual typed non-success close/hold policy; no invented work Failure or native retry/input replay |
| actual Native known failure/result finalization (`1954` onwards) | Genuine terminal, retained draft/result and typed closure ports; no absent ID fabricated binding |
| gate observe (`2004–2013`), evaluate/apply (`2378–2885`) | Complete private gate claim + fused observed/disposition + atomic completion chain, retaining Verifier8/private result acceptance |
| cancel/fail_task/terminal recovery (`2024–2074`) | Trusted nongrant terminal_decision/recovery, complete chain; no native currency refresh |
| finalization/escalation/invalidate/hold/fail/retry (`931–1018,2108–2375,2892–3001`) | Actual closed-operation source transition, typed hold/decision/closure, or refusal; no open operation ordinary persist |
| ordinary transition validator (`3184–3340`) / WorkflowAccess and Store (`state/mod.rs:427–910`) | Reject every existing Session/execution bind delta in all modes; only private binder changes them; native factual ports bypass Task writer with exact projections |
| observe_workflow_gate (`state/mod.rs:919–985`) | Protected private fused gate observation replaces workflow.gate_observed and generic workflow.saved; no generic Source after_write |
| execution reserve/adopt (`state/execution.rs:574–872`) | Pre-marker Unit relation, exact private reservation; protected original marker prohibits later adoption/reallocation |
| retirement Task projection (`state/execution.rs:999–1018`) / artifact Task publication (`artifacts.rs:323`) | Genuine nongrant/closed-result typed transaction only; no inadvertent Task write on binding/refusal/live quota |
| Verifier8 reserve/accept (`verification.rs:698–795,982–1110`) | Preserve command-only grant and no-native marker; exact factual gate successor/closure consumption when composing adjacent native phases |

### 8.1 Every Session writer

| Actual writer | Required rule |
| --- | --- |
| register_session_tx (`state/execution/sessions.rs:200–251`) / register_native_session (`native_results.rs:230–255`) | Exact original allocation/pair branch; atomic Session identity index; cannot register a second Session for one owner; keep command-only refusal |
| Core ACK update (`sessions.rs:21–51`, `native.rs:1177`) | Latest Session CAS/immutable identity plus original private current phase validation; positive PID/native UUID advancement preserved |
| close_session_tx / close_execution_session (`sessions.rs:55–70,253–279`) | Actual own nongrant terminal evidence, no new admission, preserve known work; index includes terminal/Lost history |
| fence_epoch_sessions (`sessions.rs:139–179`) / Owner preparation Drop (`execution/owner.rs:65–81`) | Factual Lost/fencing only, no owner reconstruction; guard ownership distinguishes never-launched from already retained actual invocation |
| private Native result/registration guard (`native_results.rs:365–503`, Native Core/drop) | Actual observed proof/correlation retained across storage errors; own Session updates and logical terminal facts atomically coherent |
| public put_session (`state/mod.rs:990`), conditional/environment APIs (`1007–1156`), generic put_record/write_record_tx (`412,1631`) | No protected private owner creation or native Session mutation/binding; reserved scopes use actual typed writers only |
| Generic CLI (`adapter.rs:1123`), Grok (`adapter/grok/mod.rs:425,844`) | Protected scope refusal until actual complete private protocol; no capability/config bypass |
| Legacy Codex (`codex/session.rs:521,555,577,1321,2268–2270`) | Existing legacy authority remains distinct; public legacy entry cannot mutate new managed owner Session or mint phase pair |
| any recovery/Session importer/raw supported writer | Complete identity index transaction and typed fencing/restore contract; malformed history refuses negative proof; never create live proof from body |

Inline fixture-only Session writers and cfg(test) register_execution_session do
not qualify the real phase producer. Public Record and Session query DTOs remain
diagnostic. Every actual Session write maintains identity indices including
replacement attempts and lifecycle changes, with positive body/index consistency
validation; migrations scan complete legacy scope or mark it malformed/held.
The negative check is scoped and finite, LIMIT4097 overflow refuses. Current own
Session is checked inside Immediate; identity alias/provider native UUID equality
uses validated exact strings, never untrusted normalization dropping a conflict.

## 9. Explicit finite allowances and transactional cost

The inherited per-operation256 links are reserved before effects:99 gate cycles
×two links=198, one binding, one terminal decision, one closure, eight hold/clear
links (four pairs),47 optional diagnostics. No quota status link is added. Each
gate reserves its pair before effect; each hold reserves its clear. The100th gate
refuses before claim/effect. After99 Waiting, only trusted cancellation and eligible
TerminalRecovery remain; no native redispatch/budget reset. An admitted99th Passed
can close. Hold/clear exhaustion refuses further gates and successful closure;
mandatory cancellation/closure slots remain available. Diagnostics exhaustion
cannot consume those slots. Repeated no-write facts spend no second link; repeated
actual gate calls do spend their pair even when compact outcomes coalesce. Closure
releases only unspent reserve; spent retained bytes remain charged. Held retains
reserve. All new durable bodies count toward the actual128-MiB Workflow budget.

Complete chain extraction outside Store is≤256 links/≤1 MiB. Sealed compact
transaction proof is≤128 metadata rows×4096 bytes≤512 KiB, including all actual
certificates/endpoints/scalars. This is **not** the total SQL/JSON/write cost.
Every port declares only the surfaces it reads and instruments complete encoded
rows, repeated reads, JSON/CHECK and writes; unlisted/oversized surfaces refuse.

| Additional mandatory surface | Maximum complete encoded cost class |
| --- | --- |
| P/G/T | One each≤8 MiB; existing Goal DAG limits remain |
| Workflow | One≤8 MiB; OLD/NEW full projection and body-write/CHECK cost measured |
| launch/latest Context | At most two rows≤8 MiB each; input payload remains≤1 MiB |
| own Session | One≤4 MiB; recovery≤3 MiB/depth32/nodes32768; current read inside Immediate |
| original operation / frozen settlement | One≤4 MiB / one≤64 KiB; Native6 answer text≤1 MiB separately, receipt's existing complete encoding bounds retained |
| preparation/admission/owner | One each≤2 MiB; stricter8192-byte consumed admission remains |
| scoped WorktreeLock set | Complete≤256×16384 bytes≤4 MiB; sorted original ID/version/digest inventory≤64 KiB |
| scoped identity / retained physical owner negative lookup | Complete indexed≤4096×16384 bytes≤64 MiB, paths≤4096 bytes; overflow refuses; no physical acquisition by binder |
| applicable logical host-effect closure | ≤256 admissions×8 KiB +256 outcomes×2 KiB; all worker joins outside Store; stricter existing Unit/effect limits still apply |

Mandatory indices/scope constraints prevent unlimited irrelevant global scans;
complete own-scope index validation and bounded malformed flags are required.
Full body planning/hash/chain proof and filesystem/path preparation occur outside
Store. Publication has no await, filesystem, process or native call. Current row
CAS and full scoped lock/own Session/negative checks stay inside Immediate.
Finite cost is neither a measured latency bound nor a guarantee of mutex fairness;
near-bound controls must report other-Project contention honestly.

## 10. Implementation and qualification sequence

1. Compose fixed actual Runtime9 and Verifier8 baseline; re-inventory every writer.
   Implement private phase allocation/preparation/registration/input/terminal ports
   and Binding10 immutable schema/guards, without claiming readiness.
2. Implement retained supervisor, exact NoCurrentDispatch parking, derived quota
   status and private current-source/Driver successor recognition; no generic
   after_write authority refresh. Keep unsupported actual Driver/restart gates closed.
3. Connect Workflow preflight/marker/normal and late binder; replace all open-phase
   writers with exact private diagnostic/gate/decision/closure ports. Requalify
   Native6, Source7 and Verifier8 production consumers with the composed source.
4. Run causal account-free actual-adapter/stdio controls, compiled mutation controls,
   old cached writer/migration matrix and clean fixed full regression. Independent
   source reviewers must review actual producer/consumer composition, not only SQL.
5. Native official CLI authentication/settings/hooks/four-Task tests remain Phase3
   user-approved qualification on macOS/Linux, reported separately. No cleanup,
   universal compatibility, process-death or OSS-readiness claim follows from fixtures.

| Required control group | Actual causal consumer |
| --- | --- |
| Normal held start | Actual Engine/registered Claude and Codex private producer: preinput/preACK Starting binds once; Task/P/G/Session/Context/locks unchanged; real callback remains eligible |
| Exact source/CAS | Second DB changes each original owner/context/W/lock body/version, added/deleted locks, same-version mutation/REPLACE; binder and prewire admission refuse without new input/audit |
| Identity and writers | Actual prepared reviewer path; foreign/missing/Lost/duplicate historical UUID/malformed index; every generic protected writer; paired delta and extra history/detail reject |
| Guard handoff | Fault first binder save after real Launched and abort Engine awaiting start: supervisor keeps exact own Core, known result and input count; no preparation Drop retires live invocation |
| Pre-Session wait | Actual settled version/helper+quota refusal, exact due waiter/resume original marker/unit/pair, unchanged Task/W; stale waiter and unsettled helper refuse without recapture |
| Deferred/late | Actual current consumed successful terminal before lost return with cleanup Unknown/Leftovers binds once; normal/late race; uncertain commit and lost/full wake; NOT NotDispatched/HistoricalDraft/Unknown/foreign epoch |
| Quota/live consumers | Genuine wait/recovery produces derived Workflow/Driver status, unchanged open Task/W, no resend; terminal interruption closes non-success with fresh retry; siblings unaffected |
| Successor/closure | Genuine binding→diagnostic→gate→result/Verifier→closure chain; Source7 and actual Driver original pins remain untouched/current; cancel with drift does not launder it |
| Finite/DDL | 99/100 gates, four hold pairs, diagnostics exhaustion, mandatory closure, closed reserve release; near8-MiB body/4-MiB Session/256 locks; raw guards with recursive_triggers OFF; cached9/10/newopen refusal matrix |
| Restart | Actual fencing preserves known work and holds unknown; row-only owner/receipt cannot restore; authentic #14 factual-only path tested only after its real producer exists |

Each positive fixture first reaches genuine actual allocation/registration/admission
prerequisites. Mutation controls remove exact checks/restore Task bump/refresh pins/
remove sole writer or predecessor/negative index/private pair/guard handoff/readiness,
and fail at the intended consumer assertion. Compile/setup failure is not a kill.
Record clean exact source commit, mutation/restore trees, commands and limits. Full
fixtures cannot be disabled, exempted or replaced with seeded private authority.

## 11. Requirements mapping and human gates

| Requirement | Concrete design sections |
| --- | --- |
| MB1 | §4 fixed atomic write set, §7 exact factual links |
| MB2 | §2.2 immutable full post-marker frame, §4 original CAS |
| MB3 | §2 complete actual owner/input protocol, §3 protected origin, §6 settlement |
| MB4 | §2 prepared namespace, §4 latest own and complete negative identity checks |
| MB5 | §3 sole writer guards, §8 complete call-site inventory |
| MB6 | §2.1 before-effects preflight, §1 actual composition boundary |
| MB7 | §5 retained supervision/deferred parking, §6 late/non-success split |
| MB8 | §5.3 actual Driver/durable level, §6 fenced authentic restore boundary |
| MB9 | §7 genuine successor/typed ports, §8 wait/writer inventory, §9 budget/cost |

Independent joint design approval is required before production source development;
independent source approval is required before installation/native availability.
Actual Runtime9 Driver and #14 recovery capabilities cannot be
invented to bypass dependencies. Unsupported restore remains held and visible.
The schema order is agreed allocation, not evidence of available implementations.
This proposal changes no license, README, authenticated account state or main.
rururunx is not a security sandbox. Work outcome, immutable result certification
and best-effort cleanup remain separate throughout.


## INLINE INPUT sol-independent-review-result.md
{"id": "sol-independent-review-result.md", "bytes": 19757, "sha256": "dc5749ceff265c831b599387ef47781bf1623f851a88cfe88401ec9a808890f7"}
### 🔍 Codex レビュー

**REQUEST_CHANGES — 12 confirmed mandatory findings: 4 High, 8 Medium.** Two additional Low findings are optional factual corrections.

I FULL-read the 695-line [proposed HOW](/Users/shuheisuzuki/Documents/dev/rururunx/worktree/issue-43-native-success-continuation-design/doc/design/issue-43-native-success-continuation-design.md), the governing managed-binding requirements/design, and prepared-producer §9.3. I inspected relevant production consumers and master descriptions through immutable Git objects.

References below use:

- **H:** proposed HOW at `0c35653c7a8aa05c78c5285c2f4b5370db520e70`.
- **R:** `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4`.
- **CA:** `95ef0e783c0b18c90dfef23ba1b77b9dc67735f1`.
- **RN:** final `1d783440b7f4d705bf8cf4c667e022f5cec87777`.
- Source paths are relative to `crates/rrx/src/`.
- **WHAT:** R `doc/requirements/issue-43-managed-binding-requirements.md`.

The document SHA-256 matches `2022e85329253dfbbb7ae5f84f22e0b8d32c674cfe8bcca5cb` **only if shortened incorrectly**; the actual verified hash is **`2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6`**. HEAD remains frozen and the worktree remains clean.

**Mandatory findings**

**SC-01 — High / mandatory: normal Bound jobs cannot become eligible for success discovery.**

**H L306–316, L334–358.** The sweep considers a job due when binding is unacknowledged or an existing `SuccessContinuation` needs housekeeping. A normally acknowledged job that has not yet settled satisfies neither condition. Consequently, step 3—which discovers its later settlement and installs that continuation—is never reached.

RN `runtime/phase_jobs.rs:L683–711` retains the returned binding and performs the one-shot binder. Settlement is subsequently projected through the Native owner, not by changing the job’s retained start outcome: R `execution/native/phase_protocol.rs:L416–429`, `execution/native.rs:L1257–1270`.

**Scenario/impact:** SC1 binds while the peer is held. The peer later completes successfully. The job remains acknowledged with no continuation; Driver lookup remains Pending, so capture and closure never begin. Lost-notification fallback does not repair this.

**Correction needed:** make acknowledged, launched jobs awaiting settlement discoverable through the bounded timer path, without relying on notifications or passive observers. This is required by WHAT MB8, L135–142, and MB-AC1/4, L167–170.

**SC-02 — High / mandatory: protected Git capture still inherits a marker-refusing process monitor.**

**H L377–384, L393–404, L596.** The replacement inventory covers helper admission but omits the running helper’s currency monitor.

R `execution/git_io.rs:L193–200` calls `process::capture_scoped_pinned`. Its timer invokes `store.validate_execution(..., native, !native)` at R `execution/process.rs:L357–378`, particularly L375. Passing no Driver ticket does not remove that invocation. Generic finalization reaches the marker refusal at R `state/runtime/driver.rs:L339–342`.

**Scenario/impact:** a legitimate protected Git helper runs long enough for the monitor tick. It refuses the original marked Driver, and the helper becomes Unknown through `git_io.rs:L220–238`. H L404 then holds capture. Fast commands occasionally finishing before the monitor do not establish a valid positive path.

**Correction needed:** include the running-process fence and its retained currency/lifetime in the protected consumer replacement. Preserve fencing rather than removing it. WHAT MB9, L144–161, and managed-binding HOW §8 require coverage of actual consumers.

**SC-03 — High / mandatory: Implement evaluation retains a second generic finalization refusal.**

**H L418–425.** `evaluate_settled` is specified as existing `evaluate` with only three substitutions; everything else remains unchanged.

After historical artifact verification and source recapture, R `execution/workflow_gates.rs:L355–376` performs another `store.validate_execution(&unit.authority(), false, true)` at L366. This is separate from `terminal`, which the proposed substitutions replace.

**Scenario/impact:** even after fixing capture and obtaining a genuine Ready artifact, Implement evaluation reaches this final check and refuses the marker-bound Driver. No Passed completion is issued; SC1 cannot close successfully.

**Correction needed:** map this final Unit recheck to the protected original-anchored validator while preserving the exact claim and artifact rechecks. Required by WHAT MB9 and MB-AC5, L172.

**SC-04 — High / mandatory: normal-binding uncertainty cannot converge after genuine Native progress.**

**H L111, L308–311, L325–328, L519–524.** Confirmation requires the full original currency of the plan’s kind. For a normal plan, that currency includes an open native Unit, absent work, a non-closed invocation and live Session state.

Those conditions are explicit in R `state/managed_binding/binding.rs:L91–97, L158–178, L183–220, L398–408`. Genuine terminal persistence changes the Unit, invocation and Session at R `state/execution/native_phase/terminal.rs:L472–551`.

**Scenario/impact:** normal binding commits but returns uncertain. Before its confirmation, the same invocation successfully settles. The exact binding Workflow/link still exists, but normal-kind native images no longer match. Confirmation becomes Held. The sweep cannot replace the retained plan with a late one until RolledBack or definitive refusal. The corresponding precommit-uncertain case also cannot classify rollback once Native has progressed.

**Correction needed:** distinguish write eligibility from factual confirmation, and specify how the SAME genuine native lineage proves a binding’s commit/rollback after legitimate native advancement. Do not relax parent, lock, original input or foreign-owner checks. WHAT MB7/8 and MB-AC4 explicitly require this convergence.

**SC-05 — Medium / mandatory: the settlement cannot supply the promised complete terminal row images.**

**H L245–257, L292–298.** Every expected late-binding image is said to come from the SAME settlement, using existing accessors unchanged. The settlement does not retain the complete invocation, admission or readiness postimages.

R `execution/native/phase_protocol.rs:L305–315, L642–670` exposes Unit, receipt, Session/version, thread/turn and consumed input. Complete terminal images instead reside privately in `NativeTerminalPlan`, R `state/execution/native_phase/terminal.rs:L116–135`. Readiness and admission versions/bodies depend on their actual preimages, L553–595. `NativeTerminalCommit` and its consumption do not transfer those images into settlement: terminal L140–169; `execution/native.rs:L1257–1270`.

**Scenario/impact:** the late planner cannot construct the required complete expected images from its specified inputs. Reading current rows and treating them as those expected images would abandon the stated producer-owned comparison.

**Correction needed:** identify a legal, sealed path retaining the actual known terminal postimages and delivering them to the late planner. Keep the original terminal/settlement constructors sealed. This follows WHAT MB7, L120–133, and MB9’s complete-image obligations.

**SC-06 — Medium / mandatory: closure admission is asserted without an implementable owner/caller contract.**

**H L191–196, L239, L475, L502–505, L531.** The synchronous `&mut Store` port is said to acquire asynchronous `control_admission` before acquiring Store. Its signature receives neither Runtime nor admission, and the synchronous Root reconciliation API likewise receives neither.

R `state/mod.rs:L88–92` contains no Runtime/admission handle. CA’s actual pattern uses a Runtime-owned async method returning a guard that retains both admission and a strong Runtime: CA `runtime/installation.rs:L84–86, L148–193`. CA service sweeps are not implicitly admission-held: `runtime/service.rs:L73–92`.

**Scenario/impact:** the declared API cannot perform the specified acquisition. Implementing only a mutex guard also does not reproduce CA’s protection against `Runtime::drop`. Root publication retries need the same ordering and lifetime contract.

**Correction needed:** specify the actual Runtime-owned acquisition path, guard ownership and release order for the Driver and Root consumers, including publication retries. Preserve acquisition before Store and strong Runtime custody across the admitted segment. This is an implementation gap against H’s own stop-linearization promise, not a request for a new launch barrier.

**SC-07 — Medium / mandatory: two proposed cross-module consumers lack legal sealed APIs.**

**H L202–207, L223–226, L182–183.**

- `DriverClosureAdvance` is located in the new sibling `driver::closure` module, but its implementation needs the original advance’s private ticket/post-row/body. R `state/runtime/driver/marker.rs:L10–19` keeps those fields private to `marker`.
- `SettledGateCompletion` has private outcome/receipt/claim fields in `execution::workflow_gates`, but the `state::managed_binding::success` planner must consume them. No consuming/accessor API is declared.

**Scenario/impact:** the stated module placement and private-field contract do not permit the proposed consumers to obtain their inputs. Crate-private type visibility does not expose private fields to siblings.

**Correction needed:** specify legal narrow consuming/accessor boundaries or producer-local operations, while retaining field and constructor sealing. No build was run; this is a source/privacy mismatch, not compile-refusal control credit. Governing references: WHAT MB9 and managed-binding HOW §8’s actual writer/consumer inventory.

**SC-08 — Medium / mandatory: capture’s helper and transaction bounds are false.**

**H L350, L377, L393–404, L478, L541–544.** The preserved capture algorithm exceeds 14 helpers even before corpus reads and historical verification.

At R:

- `execution/git_io.rs:L249–294`: eight ownership commands.
- `execution/results.rs:L235–265`: format, ancestry and exact-commit commands.
- `results.rs:L777–810`: three content scans plus one command per `.gitattributes`.
- `results.rs:L301–345`: optional repository initialization, destination format, fetch, fsck and two graph traversals.
- `execution/workflow_source.rs:L1220–1244`: tree read and one command per eligible blob.

The existing-repository capture alone contains at least **19 commands** in those listed steps. `state/execution/effects.rs:L41–89` does not impose the claimed aggregate helper count.

**Scenario/impact:** valid repositories exceed the stated snapshot cost. Larger inventories can also consume effect capacity before closure’s ≤257-row check. L350’s “one durable write per call” is likewise false for a call that captures, stages/marks Ready, journals helpers and claims a gate.

**Correction needed:** account for the actual command catalogue/cardinality, complete effect inventory and admission capacity, with truthful refusal and overflow behavior. Do not substitute blanket refusal or an invented 14-command limit. WHAT MB9 L149–152 and managed-binding HOW §9 L673–695 mandate complete finite costs.

**SC-09 — Medium / mandatory: retained binding memory is materially undercounted.**

**H L549–552.** The stated 12 MiB binding-plan allowance omits owned representations already retained by the unchanged plan.

R `state/managed_binding/binding.rs:L34–44` retains the current successor, Session body, invocation body, after body and exact mutation. `CurrentWorkflowSuccessor` owns a Workflow body through an Arc: `successor.rs:L25–30`. `Body<T>` owns both raw and canonical encodings plus the parsed value: `canonical.rs:L18–37`. The retained mutation owns old/new SQL images: `permits.rs:L185–190`; binding constructs these at L359–375.

**Scenario/impact:** the advertised approximately 2.5 GiB encoded upper bound excludes complete encodings and retained row copies, independently of allocator/RSS overhead. The author’s proposed policy choice is therefore based on an incomplete number.

**Correction needed:** distinguish shared existing custody from additional owned copies and count each retained encoding/image. No new cap is prescribed. WHAT MB9 and managed-binding HOW §9 require accurate complete-cost accounting.

**SC-10 — Medium / mandatory: Workflow Waiting/Held is not connected to Goal and CLI status.**

**H L434–439, L599–603.** Driver Waiting is specified, but the claim that Goal and CLI derive the new Workflow disposition has no consumer replacement.

R `state/runtime/goals.rs:L318–319, L540–546` uses `waiting::observe/effective_state`. That reader is a pre-terminal quota/readiness join: `state/runtime/waiting.rs:L35–60`. Successful settlement violates its native-open/start-not-ended predicates. `effective_state`, L132–145, only maps Quota/Capacity and otherwise returns stored Task state.

**Scenario/impact:** Requirements’ actual missing-evidence Waiting is persisted in Workflow and reported by Driver, while Goal/task-page status still reflects the old Task state or generic Held observation. It does not expose the qualified-evidence wait promised at L439.

**Correction needed:** inventory and connect bounded factual Workflow Waiting/Held observation to those status consumers, without writing Task WaitingHuman/blockers. Managed-binding HOW L578–579 explicitly requires actual Workflow/Goal/Runtime/CLI derived status.

**SC-11 — Medium / mandatory: SC3’s contention outcome and named mutant kill are not causal.**

**H L313–315, L623, L636.** SQLite writer-lock contention is described as normal binding `Conflict`. The actual Immediate acquisition propagates an error: R `state/managed_binding/binding.rs:L395–397`. H L519 classifies that as uncertain `Err`, followed by confirmation—not definitive `Conflict`.

Furthermore, the production sweep only invokes late planning when settlement is Some, H L313. Removing a late-planner None/live check alone does not make this caller invoke it for the pre-ACK owner. A direct negative planner call also does not automatically produce the claimed committed `closed_settlement` link.

**Scenario/impact:** SC3 does not establish its stated retry branch, and its named first mutant assertion can remain unchanged behind caller checks or other required settlement inputs.

**Correction needed:** use the actual contention classification and identify a compiled, reachable causal assertion for the intended predicate. Otherwise record that check as defense-only. WHAT MB-AC4.a/6/7, L171–174, disallows SETUP, earlier-refusal or impossible-link credit.

**SC-12 — Medium / mandatory: M1 is not a reachable positive fixture or a demonstrated headroom kill.**

**H L631, L650.** An original Context near 8 MiB cannot pass this Native allocation merely because the stored Context decoder permits 8 MiB.

R `execution/workflow_source/native_handoff.rs:L552–558` serializes Context data as the input payload. R `execution/phase.rs:L201–218` limits that payload to 1 MiB and the complete frame to 2 MiB; `execution/native.rs:L268–269` repeats the payload limit. An oversized original Context therefore stops before marker/binding.

Separately, near-maximum individual bodies do not by themselves put `charged_scope_bytes` close enough to 128 MiB for omission of the 17.5 MiB headroom check to change claim admission.

**Scenario/impact:** the specified M1 positive can be SETUP, and even an accepted smaller fixture need not kill the headroom mutant.

**Correction needed:** separate mechanical maximum-representation measurements from reachable original-Context and closure-Context fixtures. Specify a genuine admitted scope-budget boundary for the headroom assertion. Preserve input limits; do not fabricate rows to obtain positive credit. Governing references: WHAT MB9 and MB-AC6/7; managed-binding HOW L671–689.

**Low findings — optional factual corrections**

**SC-13 — Low / optional: cleanup observation is described as an indexed Unit advance.**

**H L80, L291.** R `state/execution.rs:L1129–1160` writes a cleanup observation/event, not an `execution_units` version/body update. Unit reads overlay cleanup independently at L249–264.

Correct the verified-source description to distinguish that overlay from an indexed Unit CAS/version advance. This does not justify blocking eligible success on cleanup Unknown; WHAT MB7 L124–129 preserves that independence.

**SC-14 — Low / optional: the waiting consumer’s path is incorrect.**

**H L603** names `runtime/waiting.rs`; the pinned implementation is `state/runtime/waiting.rs`. Correct the reference. The substantive missing status connection is SC-10.

**Investigated false positives and unchanged limits**

- **SC-N:** successful original-phase closure plus a correctly published, marker-free next Driver is a necessary dependency for the narrow increment requested here. It does **not** satisfy prepared-producer §9.3’s broader “Driver continuation after initial Evidence.” H L67, L366 and L674 explicitly leave that continuation unmet. I have not added execution of SC-N as a new implementation requirement for this narrow review; the broader prerequisite must remain open.
- **Requirements/non-Implement Waiting:** H L102 and SC6 preserve the actual missing qualified Requirements evidence. R `execution/workflow_gates.rs:L323–328` really returns integration Waiting. Ready capture is not Published acceptance, and no RequirementsCommit or full Workflow success is claimed. SC-10 concerns visibility of that truthful wait.
- **HEAD equal to base:** the existing capture/gate predicates do not require Implement to create a new commit. H L403 preserves that limit and restricts control credit to the commit-producing peer. I found no governing requirement authorizing a new-commit predicate here.
- **Weak lookup lifetime:** the ticket is also strongly retained by genuine handoff/marker custody. The Weak lookup alone is not a confirmed lifetime defect.
- **Cleanup Unknown:** it neither certifies work nor independently invalidates genuine settled success. I did not impose physical-death/full-cleanup evidence.
- **RN closure reuse:** RN’s no-dispatch proof cannot substitute for registered success. The proposed separate success lane respects that distinction.
- **Authentic positive setup:** accepted HumanIngress, configured NativeAdapter and the peer’s real commit-producing mode provide plausible genuine setup components. CA’s existing Bound control does not prove terminal, captured graph, Passed gate or closure.

**Scope and material unverified limits**

This was source inspection only. No tests, builds, mutants, provider processes or Native workloads ran. The proposed controls therefore have no executed positive or kill evidence.

CA and RN remain independent descendants of R, not a composed or qualified source. Actual normal/late races, whole-postimage closure confirmation, stop timing, artifact-corruption timing and measured memory remain unverified. In particular, closure confirmation must use its closed Task/Context/operation/Driver postimages; RN’s current-successor validator still requires the open marker projection.

No edits, Git writes, network/public writes, auth/settings/hooks/token access or live Goal-review evidence access occurred. No cleanup was necessary because no task files or checkouts were created.

This verdict grants **no source implementation, merge, Native, Issue24 or full-MVP approval**. The unchanged full-MVP and official CLI/both-OS gates remain open.

## INLINE INPUT root-verified-finding-disposition.json
{"id": "root-verified-finding-disposition.json", "bytes": 16956, "sha256": "99ed136fcd9ef24cdaf552c6fc702171ec3b4b0c4a4b45aa7c7e24693a94888d"}
{
  "recorded_at": "2026-10-07T02:53:26.652079+00:00",
  "review_head": "0c35653c7a8aa05c78c5285c2f4b5370db520e70",
  "root_verified_git_objects": 62,
  "root_verified_contiguous_excerpts": 79,
  "root_full_read_original_final_review": true,
  "root_read_factual_map_summary_lines": [
    1,
    215
  ],
  "root_literal_findings_checked": [
    "SC-01",
    "SC-02",
    "SC-03",
    "SC-04",
    "SC-05",
    "SC-06",
    "SC-07",
    "SC-08",
    "SC-09",
    "SC-10",
    "SC-11",
    "SC-12"
  ],
  "findings": [
    {
      "id": "SC-01",
      "classification": "confirmed_mandatory_contract_correction",
      "finding": "Acknowledged launched jobs awaiting settlement are excluded from the due predicate.",
      "facts": "H requires no acknowledgment or an already installed continuation for due classification. The normal start stores Launched once and binds once; subsequent terminal delivery projects into the same Native owner. It does not change JobState.outcome. Thus the timer never enters the discovery step for normal Bound followed by settlement.",
      "limits": "Static liveness contradiction, also independently confirmed by Root. No successful continuation control was executed. Preserve bounded fair discovery and avoid observer ownership.",
      "refs": [
        "H-due",
        "H-next",
        "job-shape",
        "job-return",
        "job-bind",
        "binding-snapshot",
        "terminal-delivery",
        "WHAT-MB7-9",
        "WHAT-AC"
      ],
      "root_disposition": "confirmed_required_correction",
      "runtime_control_executed": false
    },
    {
      "id": "SC-02",
      "classification": "confirmed_mandatory_contract_correction",
      "finding": "Protected Git helper admission does not replace the running helper currency monitor.",
      "facts": "UnitGit awaits capture_scoped_pinned. The 50 ms timer checks Unit identity and then generic validate_execution even with driver=None. Generic Driver validation refuses a marked row. A monitor refusal is journaled Unknown and the proposed capture requires settled helpers. This is a live protected consumer absent from H\u2019s replacement map.",
      "limits": "Do not delete the timer/fence. No executed timing or mutant credit; actual protected currency/lifetime must be connected at this consumer.",
      "refs": [
        "H-capture",
        "git-run",
        "git-monitor-call",
        "process-monitor",
        "marker-refusal",
        "WHAT-MB7-9"
      ],
      "root_disposition": "confirmed_required_correction",
      "runtime_control_executed": false
    },
    {
      "id": "SC-03",
      "classification": "confirmed_mandatory_contract_correction",
      "finding": "The final Implement Unit check still reaches generic marker refusal.",
      "facts": "H lists only three evaluate substitutions and preserves everything else. The existing evaluate path separately runs validate_execution(false,true) after historical verify and source recapture, then checks exact artifact and writes Verification. Replacing terminal alone leaves this check.",
      "limits": "Preserve claim/artifact/currentness checks and connect this separate protected check; not a request for new evidence semantics.",
      "refs": [
        "H-gate",
        "gate-final",
        "marker-refusal",
        "WHAT-AC"
      ],
      "root_disposition": "confirmed_required_correction",
      "runtime_control_executed": false
    },
    {
      "id": "SC-04",
      "classification": "confirmed_mandatory_contract_correction",
      "finding": "Full normal write currency prevents SAME-plan confirmation after genuine Native progress.",
      "facts": "Normal eligibility requires proof.is_live, no settlement, open effects/finalization and absent work; owner/invocation and Session images remain exact. Terminal persistence advances Unit, invocation, Session, readiness and admission. completed also revokes the owner, so NativePhaseBinding::is_live checks the changing owner state as well as its captured live boolean. H confirms under full original currency and forbids replacing an uncertain plan before rollback/refusal. A genuine terminal can therefore strand committed and rolled-back binding classification.",
      "limits": "Confirming factual commit/rollback is distinct from issuing a new normal write. No relaxation of original parent/input/owner/lock/epoch checks is authorized. No race control executed.",
      "refs": [
        "H-due",
        "H-uncertain",
        "normal-session",
        "normal-eligibility",
        "normal-write",
        "binding-access",
        "binding-snapshot",
        "settlement-issuer",
        "terminal-progress",
        "WHAT-MB7-9",
        "WHAT-AC"
      ],
      "root_disposition": "confirmed_required_correction",
      "runtime_control_executed": false
    },
    {
      "id": "SC-05",
      "classification": "confirmed_mandatory_contract_correction",
      "finding": "Existing settlement accessors cannot deliver the promised complete terminal postimages.",
      "facts": "NativeTerminalPlan retains full invocation/readiness/admission before/after PairRow images. NativeTerminalCommit::into_parts and phase.settled deliver Unit, receipt, Session/version and settlement only. OwnedPhaseSettlement retains consumed input and thread/turn, not those complete terminal SQL postimages. Admission/readiness versions and bodies are derived from actual preimages. H says expected images come from the SAME settlement with existing accessors unchanged; that delivery is absent.",
      "limits": "Rows read later can be compared as facts but cannot be adopted as the producer\u2019s expected images or create a grant. No new serializable image credential or constructor widening is authorized.",
      "refs": [
        "H-late",
        "H-sealed",
        "terminal-shape",
        "terminal-progress",
        "terminal-delivery",
        "settlement-shape",
        "settlement-access",
        "WHAT-MB7-9"
      ],
      "root_disposition": "confirmed_required_correction",
      "runtime_control_executed": false
    },
    {
      "id": "SC-06",
      "classification": "confirmed_mandatory_contract_correction",
      "finding": "The closure/Root publication callers have no declared async admission and Runtime lifetime bridge.",
      "facts": "H synchronously declares close_phase_success(&mut Store, material) and reconcile_success(&PhaseSupervisor,&AtomicBool), yet assigns awaited control_admission acquisition before Store and retention through publication/retries. Store contains only Connection and permits. Existing service sweep holds no admission. CA\u2019s real async admit_activation retains OwnedMutexGuard plus strong Arc<Runtime>, with explicit drop order and association/composition checks. None of those objects are arguments/owned fields in the proposed closure call path.",
      "limits": "This confirms an owner/caller/lifetime contract gap, not an observed stop exploit. Both Driver closure and Root publication retries need a concrete caller bridge. Preserve actual CA ordering; do not add a launch barrier or settle SourceStop/Cancel/R2 policy.",
      "refs": [
        "H-api",
        "H-closure",
        "H-uncertain",
        "H-lock",
        "store-shape",
        "admission-shape",
        "admission-acquire",
        "service-loop",
        "driver-publish"
      ],
      "root_disposition": "confirmed_required_correction",
      "runtime_control_executed": false
    },
    {
      "id": "SC-07",
      "classification": "verified_api_omission_mandatory_status_unverified",
      "finding": "Private ownership is real; mandatory design blockage is not independently established.",
      "facts": "DriverMarkerAdvance ticket/next/body are private to marker.rs. H places a new impl in its sibling closure.rs, but already declares the needed method on the existing type. That method can legally live in the original producer module without exposing fields. H\u2019s SettledGateCompletion private fields need a sealed consumption/accessor operation for a state-side planner; none is listed. Such an operation can also be supplied in the owning module without changing the authorized semantic contract.",
      "limits": "Do not award compile-refusal credit: proposed code was never compiled. Narrow producer-owned placement/consumption clarification is useful. Source implementation must obey Rust privacy, but this inventory alone does not prove a new mandatory WHAT/HOW gate; Root decides classification before any author batch.",
      "refs": [
        "H-api",
        "driver-private",
        "driver-ports",
        "driver-publish",
        "WHAT-MB7-9"
      ],
      "root_disposition": "implementation_api_clarification_optional_design_change",
      "runtime_control_executed": false
    },
    {
      "id": "SC-08",
      "classification": "confirmed_mandatory_cost_and_control_correction",
      "finding": "The preserved capture catalogue exceeds the stated helper and write count.",
      "facts": "Existing-repository capture contains 8 ownership + 3 format/ancestry/commit + 3 content scan + 5 destination/fetch/fsck/two traversals = 19 helper commands, plus one per .gitattributes and optional init; H also requires HEAD, historical verification, source corpus and later recapture/publication. Corpus admits at most 4096 entries and reads each eligible blob, not 14 helpers. Every UnitGit helper reserves then reconciles an effect; capture separately stages and marks Ready. Generic helper reservation has no aggregate count gate. The 256 admission gate and bounded Native inventory reader belong to distinct Native ports and do not bound this helper writer by construction. Existing workflow_publication validates generic finalization both before and after its five-command graph verification; H already replaces that publication entry point. Those two checks are inventoried here without creating another finding.",
      "limits": "Confirmed false accounting and omitted capacity-consumer inventory. Valid large workloads may truthfully become Held at an existing profile bound; this map prescribes neither a new cap nor a guarantee that every maximum repository closes. No runtime overflow/control executed.",
      "refs": [
        "H-capture",
        "H-next",
        "H-closure",
        "H-cost",
        "capture-catalogue",
        "capture-ownership",
        "capture-content",
        "capture-verify",
        "corpus-bound",
        "corpus-catalogue",
        "helper-writer",
        "input-effect-cap",
        "inventory-cap",
        "HOW-cost",
        "WHAT-MB7-9"
      ],
      "root_disposition": "confirmed_required_correction",
      "runtime_control_executed": false
    },
    {
      "id": "SC-09",
      "classification": "confirmed_mandatory_cost_correction",
      "finding": "12 MiB per binding plan and the derived 2.5 GiB total omit retained encodings/images.",
      "facts": "ManagedBindingPlan holds current successor, own Session/invocation/after Body and exact mutation. CurrentWorkflowSuccessor holds Arc<Body<Record>> and original plan. Body owns raw, canonical and parsed representations. ExactRowMutation holds independently owned old/new SqlValue vectors, and plan construction fills their body strings. Copy-for-transaction clones them again transiently. Shared Arcs do not imply repeated additional allocation, but the total retained custody and additional copies must be distinguished; the two named raw strings alone are not the whole encoded total.",
      "limits": "No RSS/heap number or new cap is asserted; H explicitly disclaims those. This is an encoded/owned-copy accounting correction under existing finite-cost requirements. No allocation measurement performed.",
      "refs": [
        "H-cost",
        "plan-memory",
        "plan-construction",
        "successor-memory",
        "body-memory",
        "mutation-memory",
        "HOW-cost",
        "WHAT-MB7-9"
      ],
      "root_disposition": "confirmed_required_correction",
      "runtime_control_executed": false
    },
    {
      "id": "SC-10",
      "classification": "confirmed_mandatory_consumer_correction",
      "finding": "The factual Workflow Waiting/Held status lacks a Goal/task-page/CLI consumer path.",
      "facts": "H\u2019s gate_observed retains Waiting/detail or Held reason without a Task rewrite and promises derived Goal/CLI visibility. Goal status counts and task pages consume waiting::observe/effective_state. That bounded reader requires native effects open and readiness start_ended=0/known_terminal=0; settled success instead returns generic Held. Its only effective state mappings are quota/capacity. Runtime control delegates to those readers; CLI prints their response. No Workflow evidence_integration_unavailable/held_reason observation is connected.",
      "limits": "Actual non-Implement integration Waiting is truthful and must remain unverified; visibility is the missing consumer. Do not write Task WaitingHuman/blocker or infer runnable/policy/completion from factual rows.",
      "refs": [
        "H-gate",
        "gate-final",
        "waiting-reader",
        "goal-status",
        "goal-page",
        "control-read",
        "cli-read",
        "cli-format",
        "HOW-status"
      ],
      "root_disposition": "confirmed_required_correction",
      "runtime_control_executed": false
    },
    {
      "id": "SC-11",
      "classification": "confirmed_mandatory_control_plan_correction",
      "finding": "SC3 does not establish the named Conflict branch or the claimed late-link mutant assertion.",
      "facts": "Current Immediate acquisition propagates Err; H calls Err uncertain and SC3 nevertheless calls genuine lock contention Conflict. A future port could explicitly classify a known pre-transaction refusal, but the preserved current port does not. H\u2019s Root calls late planning only with settlement Some; removing a None/live check inside that planner does not by itself make this caller reach it without settlement, nor furnish its other required images/receipt. A direct planner check is not a committed closed_settlement link.",
      "limits": "Correct causal-control/outcome attribution is mandatory for claimed evidence. No demonstrated production bypass and no executed mutant kill. A direct defense-only check or an explicitly genuine reachable consumer setup must be labeled honestly; this map does not choose new failure semantics.",
      "refs": [
        "H-due",
        "H-uncertain",
        "H-controls",
        "normal-write",
        "H-late",
        "WHAT-AC"
      ],
      "root_disposition": "confirmed_required_correction",
      "runtime_control_executed": false
    },
    {
      "id": "SC-12",
      "classification": "confirmed_mandatory_control_plan_correction",
      "finding": "M1\u2019s original Context maximum and headroom mutant are not genuine accepted positive setups as stated.",
      "facts": "Native Source serializes Context.data into payload and carries its source_hashes/revision/scope/version into the original input. encode_input admits payload <=1 MiB and whole escaped frame <=2 MiB. ContextVersion has precisely those fields. An ordinary compactly written original Context near the stored decoder\u2019s 8 MiB maximum cannot be counted as accepted merely from that decoder ceiling. A newly created closure Context and mechanical representation measurement are distinct. charged_scope_bytes aggregates whole scoped history plus open reservations; individual maximum bodies do not establish charged+growth+link near 128 MiB or causality for omitting 17.5 MiB headroom.",
      "limits": "M1 can measure decoder/representation maxima separately, but oversized ingress is SETUP, not PR1 coverage. No admitted boundary or mutant kill was executed. Preserve actual input/scope bounds and genuine writer setup; no synthetic positive rows.",
      "refs": [
        "H-cost",
        "H-controls",
        "context-shape",
        "context-input",
        "input-limit",
        "frame-limit",
        "scope-charge",
        "scope-bound",
        "HOW-cost",
        "WHAT-AC"
      ],
      "root_disposition": "confirmed_required_correction",
      "runtime_control_executed": false
    }
  ],
  "mandatory_ids": [
    "SC-01",
    "SC-02",
    "SC-03",
    "SC-04",
    "SC-05",
    "SC-06",
    "SC-08",
    "SC-09",
    "SC-10",
    "SC-11",
    "SC-12"
  ],
  "optional_ids": [
    "SC-07",
    "SC-13",
    "SC-14"
  ],
  "no_runtime_or_mutant_credit_from_static_verification": true,
  "design_correction_scope": "One Opus high author batch for verified findings; then the original finding family performs one delta review. Preserve governing WHAT, all permission/currency/ownership/input protections and broader Workflow/MVP acceptance criteria.",
  "source_implementation_authorized": false,
  "native_qualified": false,
  "mvp_qualified": false,
  "merge_ready": false
}


## INLINE INPUT EXCERPT-HOW-status
{"id": "EXCERPT-HOW-status", "bytes": 2367, "sha256": "eb760b4a1888aeae4026ce1ef9417cb34b5e0938f9897417cfe13c9b55060d25", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "doc/design/issue-43-managed-binding-design.md", "blob": "cf19dcc12cf2d0410727a457e5e4d2eb5625f399", "whole_file_sha256": "f03efbb1b2ab7986936cac72be5580c939f31c223a4178c74d0a5d5e79dd4d00", "start": 565, "end": 590, "excerpt_sha256": "eb760b4a1888aeae4026ce1ef9417cb34b5e0938f9897417cfe13c9b55060d25"}
| session_bound | §4 only; actual normal/late private producer, first link |
| native_diagnostic | Bound live status_unavailable or persisted_status_mismatch only: active.detail≤128 UTF-8 bytes, Record version/time and one link; no Task/owner/native wait write |
| gate_claim | Actual private gate invocation: Evaluating + exact claimed_observations; no Session binding/source refresh |
| gate_observed | Actual compact claim outcome + exact Waiting/held disposition fused into ONE link/transaction, including bounded detail/held_reason; repeated identity may change only checked occurrence count/last_time |
| gate_hold | Actual typed hold/clear policy, exact held_reason/detail; no Task WaitingHuman rewrite |
| terminal_decision | Trusted cancel/fail_task action and its separately checked Task lifecycle delta; no success/binding/input |
| phase_closed | Actual eligible typed result/gate/non-success/TerminalRecovery proof and enumerated final phase/body/Task/Context changes; no transport-derived approval |

Native quota live wait/recovery does **not** use native_diagnostic or these Workflow
links. It updates its genuine Unit/pool/waiter/lease facts and derived status only;
no Task/W rewrite occurs while open. Terminal quota/capacity interruption uses
typed non-success phase closure with preserved Unknown work and explicit wait/fresh
retry policy, not live quota as failure. This choice preserves fixed ledger budgets.
Typed derived waiting feeds actual Workflow/Goal/Runtime/CLI status and scheduling;
it is not cosmetic text masking a hidden Task update.

SourceRecovery7 currently compares serialized full snapshot pins exactly
(`source_recovery.rs:286–293`) and generic after_write refreshes its row
(`342–367`). A binder Workflow+1 therefore needs a **new bounded private successor
recognizer** in validate_task/validate_binding/recovered_source_task and the actual
inputs/prepare_pack/registered Git consumers. Its anchor is the existing saved
SourceRecovery Workflow pin at the operation marker, advanced only by the genuine
pre-marker typed reserve/marker transaction. It may resolve that exact anchored
Workflow pin to the chain-proved current Workflow while every other saved source
pin remains exact. It neither mutates row.pins/version nor invokes generic
after_write. Subsequent original operation source authority stays the original


## INLINE INPUT EXCERPT-HOW-cost
{"id": "EXCERPT-HOW-cost", "bytes": 2275, "sha256": "d9deef4717300fd45968a4a6e0da614fa78c398c8624d3e380176d88a0f6ffb5", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "doc/design/issue-43-managed-binding-design.md", "blob": "cf19dcc12cf2d0410727a457e5e4d2eb5625f399", "whole_file_sha256": "f03efbb1b2ab7986936cac72be5580c939f31c223a4178c74d0a5d5e79dd4d00", "start": 670, "end": 697, "excerpt_sha256": "d9deef4717300fd45968a4a6e0da614fa78c398c8624d3e380176d88a0f6ffb5"}
releases only unspent reserve; spent retained bytes remain charged. Held retains
reserve. All new durable bodies count toward the actual128-MiB Workflow budget.

Complete chain extraction outside Store is≤256 links/≤1 MiB. Sealed compact
transaction proof is≤128 metadata rows×4096 bytes≤512 KiB, including all actual
certificates/endpoints/scalars. This is **not** the total SQL/JSON/write cost.
Every port declares only the surfaces it reads and instruments complete encoded
rows, repeated reads, JSON/CHECK and writes; unlisted/oversized surfaces refuse.

| Additional mandatory surface | Maximum complete encoded cost class |
| --- | --- |
| P/G/T | One each≤8 MiB; existing Goal DAG limits remain |
| Workflow | One≤8 MiB; OLD/NEW full projection and body-write/CHECK cost measured |
| launch/latest Context | At most two rows≤8 MiB each; input payload remains≤1 MiB |
| own Session | One≤4 MiB; recovery≤3 MiB/depth32/nodes32768; current read inside Immediate |
| original operation / frozen settlement | One≤4 MiB / one≤64 KiB; Native6 answer text≤1 MiB separately, receipt's existing complete encoding bounds retained |
| preparation/admission/owner | One each≤2 MiB; stricter8192-byte consumed admission remains |
| scoped WorktreeLock set | Complete≤256×16384 bytes≤4 MiB; sorted original ID/version/digest inventory≤64 KiB |
| scoped identity / retained physical owner negative lookup | Complete indexed≤4096×16384 bytes≤64 MiB, paths≤4096 bytes; overflow refuses; no physical acquisition by binder |
| applicable logical host-effect closure | ≤256 admissions×8 KiB +256 outcomes×2 KiB; all worker joins outside Store; stricter existing Unit/effect limits still apply |

Mandatory indices/scope constraints prevent unlimited irrelevant global scans;
complete own-scope index validation and bounded malformed flags are required.
Full body planning/hash/chain proof and filesystem/path preparation occur outside
Store. Publication has no await, filesystem, process or native call. Current row
CAS and full scoped lock/own Session/negative checks stay inside Immediate.
Finite cost is neither a measured latency bound nor a guarantee of mutex fairness;
near-bound controls must report other-Project contention honestly.


## INLINE INPUT EXCERPT-prepared-PR1
{"id": "EXCERPT-prepared-PR1", "bytes": 1016, "sha256": "f4dd1285bd96ba94f6faf221563e231c7187b9ad04fbaa99ed8d06fd1e273f8f", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "doc/design/issue-43-native-prepared-producer-design.md", "blob": "0a541476d18081cb6bf7e03b0c2477ecd6848eba", "whole_file_sha256": "02d30312099882f7744f0f79a6aa3db46845399a8e80fca7949fe64d4b39a326", "start": 548, "end": 555, "excerpt_sha256": "f4dd1285bd96ba94f6faf221563e231c7187b9ad04fbaa99ed8d06fd1e273f8f"}
### 9.3 Prerequisites (missing; owned by A/Root; not satisfied by this HOW)

1. **PR-1:** Driver continuation after initial Evidence. Today `driven_initial.rs:56–59` refuses it.
2. **PR-2:** Source offers for a later Executor, a retry and a Reviewer. These replace the first-Executor-only predicates at `native_handoff.rs:448–519`. A retry uses a NEW Unit, worktree, branch and namespace (`attempts.rs:352–470`, `resources.rs:294–297`), and a new operation, pair, Session and marker. The prior operation must be closed or Held. It is never reused.
3. **PR-3:** a Reviewer Frame at artifact revision `A`, built from retained objects. Also `ResultSnapshot` creation moved into the Source lane (`attempts.rs:604–711`) with the `ReviewerArtifactLease` handed into Source custody.
4. **PR-4:** artifact retention release must treat an open operation whose custody references the artifact as a live dependency.

Until PR-1 to PR-4 exist, a Reviewer or retry Unit refuses before any Git intent, exactly as today.


## INLINE INPUT EXCERPT-job-shape
{"id": "EXCERPT-job-shape", "bytes": 1816, "sha256": "fd08d4dca29064e352065023deebb0684a6dc800b5b6f0b399b0ee1d16f77fea", "label": "RN", "commit": "1d783440b7f4d705bf8cf4c667e022f5cec87777", "source_path": "crates/rrx/src/runtime/phase_jobs.rs", "blob": "73efd9255e56d2a5539dca4823b0bacaeb36e399", "whole_file_sha256": "ce58e32049e06847295d3ba6933354ea62ed4cfedf46badea8c3bd1c6af494fb", "start": 132, "end": 181, "excerpt_sha256": "fd08d4dca29064e352065023deebb0684a6dc800b5b6f0b399b0ee1d16f77fea"}
struct JobState {
    // EMPTY/nongrant until the SAME actual selected Native start installs its
    // own actor and original plan. Retained BEFORE marker/start/future effects.
    preparation: Arc<NativePreparationCustody>,
    observation: InvocationObservation,
    launch: Option<Arc<PhaseLaunchParts>>,
    outcome: Option<std::result::Result<RetainedStart, NativePhaseStartError>>,
    binding_plan: Option<Arc<ManagedBindingPlan>>,
    binding_error: Option<anyhow::Error>,
    nonsuccess: Option<Arc<crate::state::NativeNonSuccessClosurePlan>>,
    closed_ack: Option<Arc<crate::state::PhaseClosedAcknowledgment>>,
    closure_due: Instant,
    closure_backoff: u64,
    uncertain: bool,
    slot_released: bool,
    attention: Option<&'static str>,
}
/// Actual returned objects, never reconstructed from DTOs or registry IDs.
enum RetainedStart {
    Launched {
        _handle: ManagedSessionRef,
        binding: Arc<NativePhaseBinding>,
    },
}
struct Job {
    allocation: Arc<NativeAllocation>,
    state: Mutex<JobState>,
    changed: watch::Sender<InvocationObservation>,
}
struct Entry {
    job: Arc<Job>,
    // Independent registry ownership, never a strong return edge from Job.
    handle: Option<JoinHandle<()>>,
}

/// Produced by the actual reservation, retaining SAME entry identity without
/// a job -> launch -> reservation -> job ownership cycle. Reuse never grants
/// this call permission to remove somebody else's existing reservation.
pub(super) struct PhaseJobReservation {
    job: Weak<Job>,
    allocation: Arc<NativeAllocation>,
    fresh: bool,
}

/// Runtime-owned sibling of PhaseSupervisor. Neither slots nor jobs own it.
#[derive(Default)]
pub(super) struct PhaseJobs {
    entries: Mutex<BTreeMap<OperationId, Entry>>,
    closure_cursor: Mutex<Option<OperationId>>,
}


## INLINE INPUT EXCERPT-job-return
{"id": "EXCERPT-job-return", "bytes": 1894, "sha256": "a35d30f1ea68e60a48b83fe5932f4444fc46b21a0765a0d23a7151c4c4709495", "label": "RN", "commit": "1d783440b7f4d705bf8cf4c667e022f5cec87777", "source_path": "crates/rrx/src/runtime/phase_jobs.rs", "blob": "73efd9255e56d2a5539dca4823b0bacaeb36e399", "whole_file_sha256": "ce58e32049e06847295d3ba6933354ea62ed4cfedf46badea8c3bd1c6af494fb", "start": 683, "end": 723, "excerpt_sha256": "a35d30f1ea68e60a48b83fe5932f4444fc46b21a0765a0d23a7151c4c4709495"}
            let outcome = result.map(|start| match start {
                NativePhaseStart::Launched { handle, binding } => RetainedStart::Launched {
                    _handle: handle,
                    binding: Arc::from(binding),
                },
            });
            let refused = outcome.is_err();
            let (observation, binding) = match &outcome {
                Ok(RetainedStart::Launched { binding, .. }) => {
                    (InvocationObservation::Binding, Some(binding.clone()))
                }
                Err(_) => (InvocationObservation::Failed, None),
            };
            {
                let mut state = job.state.lock().unwrap_or_else(|e| e.into_inner());
                state.outcome = Some(outcome);
                state.observation = observation;
            }
            if refused {
                // Actual outcome is already retained. Nongrant abandonment must
                // not hold the Root job mutex or retire Unit/Task from an error.
                preparation.abandon();
            }
            job.changed.send_replace(observation);
            if let Some(binding) = binding {
                // Both the actual proof and handle are already retained. No
                // fallible planning or Store access can consume their sole owner.
                let result = job.bind_returned(binding);
                let observation = if result.is_ok() {
                    InvocationObservation::Bound
                } else {
                    InvocationObservation::BindingHeld
                };
                {
                    let mut state = job.state.lock().unwrap_or_else(|e| e.into_inner());
                    state.binding_error = result.err();
                    state.observation = observation;
                }
                job.changed.send_replace(observation);
            }
        }));


## INLINE INPUT EXCERPT-job-bind
{"id": "EXCERPT-job-bind", "bytes": 1570, "sha256": "5831dcf1891e5ed541550c07b38df5aae49ab6314a6082f9d287b422608ee959", "label": "RN", "commit": "1d783440b7f4d705bf8cf4c667e022f5cec87777", "source_path": "crates/rrx/src/runtime/phase_jobs.rs", "blob": "73efd9255e56d2a5539dca4823b0bacaeb36e399", "whole_file_sha256": "ce58e32049e06847295d3ba6933354ea62ed4cfedf46badea8c3bd1c6af494fb", "start": 809, "end": 855, "excerpt_sha256": "5831dcf1891e5ed541550c07b38df5aae49ab6314a6082f9d287b422608ee959"}
    fn bind_returned(&self, proof: Arc<NativePhaseBinding>) -> Result<()> {
        let owner = self.allocation.selected_port().owner();
        let plan = Arc::new(plan_managed_binding(owner, proof)?);
        {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            ensure!(
                state.binding_plan.is_none(),
                "binding plan already retained"
            );
            state.binding_plan = Some(plan.clone());
        }
        // Short job locks above never overlap the selected owner's Store lock.
        owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("binding Store poisoned"))?
            .bind_managed_phase(&plan)?;
        Ok(())
    }
}

enum TurnResult {
    Known(crate::state::PhaseClosedAcknowledgment),
    Conflict(anyhow::Error),
    RolledBack,
}

struct RunningJob(Arc<Job>);
impl Drop for RunningJob {
    fn drop(&mut self) {
        let preparation = {
            let mut state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
            if state.outcome.is_none() || state.observation == InvocationObservation::Binding {
                state.observation = InvocationObservation::Uncertain;
                self.0
                    .changed
                    .send_replace(InvocationObservation::Uncertain);
                Some(state.preparation.clone())
            } else {
                None
            }
        };
        if let Some(preparation) = preparation {
            preparation.abandon();
        }
    }
}


## INLINE INPUT EXCERPT-binding-snapshot
{"id": "EXCERPT-binding-snapshot", "bytes": 1284, "sha256": "5436e3067f16a111452603025ac51ea4dc6217971d2eaaafa0a355ea458889ab", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/native/phase_protocol.rs", "blob": "26e8aec7f42316ea3f0e5828ff3d4e287a6af372", "whole_file_sha256": "96b3fa95c27c776be5f8d8991bbb5f9add3276554738f335493cdc9934d4fee8", "start": 398, "end": 429, "excerpt_sha256": "5436e3067f16a111452603025ac51ea4dc6217971d2eaaafa0a355ea458889ab"}
    pub(super) fn revoke(&self) {
        self.state.store(REVOKED, Ordering::SeqCst);
    }
    pub(crate) fn is_live(&self) -> bool {
        self.state.load(Ordering::SeqCst) == LIVE && self.ack.get().is_some()
    }
    pub(crate) fn registration_ack(&self) -> Option<RegistrationAck> {
        self.ack.get().copied()
    }
    pub(crate) fn origin(&self) -> &Arc<NativeTransportStartPlan> {
        &self.origin
    }
    pub(crate) fn validate_known_registration(&self) -> Result<RegistrationAck> {
        known_ack(&self.state, &self.ack)
    }
    pub(crate) fn registered_readiness(&self) -> Result<u64> {
        self.origin.registered_readiness()
    }
    pub(crate) fn binding_snapshot(self: &Arc<Self>) -> Result<NativePhaseBinding> {
        let projection = self
            .projection
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase projection unavailable"))?;
        Ok(NativePhaseBinding {
            owner: self.clone(),
            session: projection.session.clone(),
            record_version: projection.record_version,
            consumed: projection.consumed.as_ref().and_then(Weak::upgrade),
            settlement: projection.settlement.as_ref().and_then(Weak::upgrade),
            live: self.is_live(),
        })
    }


## INLINE INPUT EXCERPT-binding-access
{"id": "EXCERPT-binding-access", "bytes": 852, "sha256": "6962faacd291b855efb9970fe986a7b455749b462f22614d79c3c2b78c7702c0", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/native/phase_protocol.rs", "blob": "26e8aec7f42316ea3f0e5828ff3d4e287a6af372", "whole_file_sha256": "96b3fa95c27c776be5f8d8991bbb5f9add3276554738f335493cdc9934d4fee8", "start": 430, "end": 458, "excerpt_sha256": "6962faacd291b855efb9970fe986a7b455749b462f22614d79c3c2b78c7702c0"}
}
impl NativePhaseBinding {
    pub(crate) fn owner_arc(&self) -> &Arc<NativePhaseSession> {
        &self.owner
    }
    pub(crate) fn owner(&self) -> &NativePhaseSession {
        &self.owner
    }
    pub(crate) fn marker(&self) -> &OriginalMarker {
        self.owner.marker()
    }
    pub(crate) fn allocation(&self) -> &NativeAllocation {
        self.owner.allocation()
    }
    pub(crate) fn session(&self) -> &Session {
        &self.session
    }
    pub(crate) fn record_version(&self) -> u64 {
        self.record_version
    }
    pub(crate) fn consumed(&self) -> Option<&ConsumedPhaseInput> {
        self.consumed.as_deref()
    }
    pub(crate) fn settlement(&self) -> Option<&OwnedPhaseSettlement> {
        self.settlement.as_deref()
    }
    pub(crate) fn is_live(&self) -> bool {
        self.live && self.owner.is_live()
    }


## INLINE INPUT EXCERPT-settlement-shape
{"id": "EXCERPT-settlement-shape", "bytes": 482, "sha256": "c8b19a3bc4a7cf3152bbdd629adac8f845c0719fac6e27a164e01af1a029bcc7", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/native/phase_protocol.rs", "blob": "26e8aec7f42316ea3f0e5828ff3d4e287a6af372", "whole_file_sha256": "96b3fa95c27c776be5f8d8991bbb5f9add3276554738f335493cdc9934d4fee8", "start": 301, "end": 315, "excerpt_sha256": "c8b19a3bc4a7cf3152bbdd629adac8f845c0719fac6e27a164e01af1a029bcc7"}
    }
}
/// Only an actual saved NativeTerminal followed by successful own logical
/// closure can issue this. It is not a review opinion or a cleanup guarantee.
pub(crate) struct OwnedPhaseSettlement {
    owner: Arc<NativePhaseSession>,
    consumed: Arc<ConsumedPhaseInput>,
    terminal: Arc<NativeTerminal>,
    unit: ExecutionUnit,
    receipt: native_result::NativeResultReceipt,
    session: Session,
    record_version: u64,
    thread: String,
    turn: Option<String>,
}


## INLINE INPUT EXCERPT-settlement-issuer
{"id": "EXCERPT-settlement-issuer", "bytes": 4016, "sha256": "ec32b19509b1cc89f326efbbba4bb076d2fa7cdb49f47dde013068c9bf2e1a68", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/native/phase_protocol.rs", "blob": "26e8aec7f42316ea3f0e5828ff3d4e287a6af372", "whole_file_sha256": "96b3fa95c27c776be5f8d8991bbb5f9add3276554738f335493cdc9934d4fee8", "start": 553, "end": 641, "excerpt_sha256": "ec32b19509b1cc89f326efbbba4bb076d2fa7cdb49f47dde013068c9bf2e1a68"}
    pub(super) fn completed(
        consumed: Arc<ConsumedPhaseInput>,
        terminal: Arc<NativeTerminal>,
        unit: ExecutionUnit,
        receipt: native_result::NativeResultReceipt,
        session: Session,
        record_version: u64,
    ) -> Result<Arc<Self>> {
        let owner = consumed.owner.clone();
        let facts = owner.allocation().facts();
        // The actual terminal transaction changes only receipt authority. Keep
        // every bounded answer/protocol/diagnostic byte tied to that SAME saved
        // terminal instead of accepting matching selected hashes or public IDs.
        let mut expected_receipt = terminal.receipt().clone();
        expected_receipt.authority = native_result::ReceiptAuthority::OwnedTerminal;
        ensure!(
            terminal.receipt().observed_work == WorkOutcome::Success
                && receipt == expected_receipt
                && receipt.observed_work == WorkOutcome::Success
                && receipt.authority == native_result::ReceiptAuthority::OwnedTerminal
                && receipt.invocation_id == facts.invocation_id
                && receipt.unit_id == facts.unit_id
                && receipt.session_id == facts.session_id
                && receipt.scope == *facts.scope
                && receipt.generation == facts.generation
                && receipt.owner_epoch == facts.epoch
                && receipt.provider == facts.provider
                && unit.id == facts.unit_id
                && unit.scope == *facts.scope
                && unit.generation == facts.generation
                && unit.owner_epoch == facts.epoch
                && unit.provider == facts.provider
                && unit.worktree == facts.path
                && unit.profile_digest == facts.profile_digest
                && unit.work == Some(WorkOutcome::Success)
                && !unit.native_effects_open
                && unit.state == UnitState::WorkKnown
                && unit.session_id == Some(facts.session_id)
                && session.id == facts.session_id
                && session.scope == *facts.scope
                && session.agent == facts.alias
                && session.provider == facts.provider
                && session.role == facts.role
                && session.worktree == facts.path
                && session.model.as_deref() == facts.model
                && session.effort.as_deref() == facts.effort
                && session.state == SessionState::Exited,
            "native phase lacks actual owned logical success"
        );
        let (thread, turn) = {
            let acknowledgement = consumed
                .acknowledgement
                .lock()
                .map_err(|_| anyhow::anyhow!("native phase acknowledgement unavailable"))?;
            let ack = acknowledgement
                .as_ref()
                .context("native phase input lacks actual acknowledgement")?;
            ensure!(
                receipt.native_thread.as_deref() == Some(ack.thread.as_str())
                    && receipt.native_turn == ack.turn
                    && session.native_ref.as_deref() == Some(ack.thread.as_str()),
                "native phase terminal acknowledgement changed"
            );
            (ack.thread.clone(), ack.turn.clone())
        };
        owner.project(&session, record_version)?;
        owner.revoke();
        let settlement = Arc::new(Self {
            owner: owner.clone(),
            consumed,
            terminal,
            unit,
            receipt,
            session,
            record_version,
            thread,
            turn,
        });
        let mut projection = owner
            .projection
            .lock()
            .map_err(|_| anyhow::anyhow!("native phase projection unavailable"))?;
        ensure!(
            projection.settlement.is_none(),
            "native phase terminal already projected"
        );
        projection.settlement = Some(Arc::downgrade(&settlement));
        Ok(settlement)
    }


## INLINE INPUT EXCERPT-settlement-access
{"id": "EXCERPT-settlement-access", "bytes": 845, "sha256": "9be56dd2fc727e9792d6aebf25957129ccce7782fbd42377327ac9d421c8a0f2", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/native/phase_protocol.rs", "blob": "26e8aec7f42316ea3f0e5828ff3d4e287a6af372", "whole_file_sha256": "96b3fa95c27c776be5f8d8991bbb5f9add3276554738f335493cdc9934d4fee8", "start": 642, "end": 672, "excerpt_sha256": "9be56dd2fc727e9792d6aebf25957129ccce7782fbd42377327ac9d421c8a0f2"}
    pub(crate) fn marker(&self) -> &OriginalMarker {
        self.owner.marker()
    }
    pub(crate) fn allocation(&self) -> &NativeAllocation {
        self.owner.allocation()
    }
    pub(crate) fn consumed(&self) -> &ConsumedPhaseInput {
        &self.consumed
    }
    pub(crate) fn terminal(&self) -> &NativeTerminal {
        &self.terminal
    }
    pub(crate) fn unit(&self) -> &ExecutionUnit {
        &self.unit
    }
    pub(crate) fn receipt(&self) -> &native_result::NativeResultReceipt {
        &self.receipt
    }
    pub(crate) fn session(&self) -> &Session {
        &self.session
    }
    pub(crate) fn record_version(&self) -> u64 {
        self.record_version
    }
    pub(crate) fn thread(&self) -> &str {
        &self.thread
    }
    pub(crate) fn turn(&self) -> Option<&str> {
        self.turn.as_deref()
    }
}


## INLINE INPUT EXCERPT-terminal-shape
{"id": "EXCERPT-terminal-shape", "bytes": 1501, "sha256": "5175e2282100f5ee85edad6e3d14da697121f42d50bb0b4ee2cd64a206cdb571", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/execution/native_phase/terminal.rs", "blob": "703082c30db807852839d3108341a18ca97dc90c", "whole_file_sha256": "37838036a6c0c45dd911c0e4e0a64ff53eeb9fc8503c995013f1f37060bbbf1d", "start": 116, "end": 169, "excerpt_sha256": "5175e2282100f5ee85edad6e3d14da697121f42d50bb0b4ee2cd64a206cdb571"}
pub(crate) struct NativeTerminalPlan {
    phase: Arc<NativePhaseSession>,
    terminal: Arc<NativeTerminal>,
    normal: Option<NativeOwnerPlan>,
    unit: UnitImage,
    unit_after: UnitImage,
    invocation: InvocationImage,
    invocation_after: InvocationImage,
    session: PairRow,
    session_after: PairRow,
    closed_session: Session,
    owner: PairRow,
    readiness: PairRow,
    readiness_after: PairRow,
    admission: Option<PairRow>,
    admission_after: Option<PairRow>,
    receipt: NativeResultReceipt,
    receipt_raw: String,
    session_version: u64,
    input_effect: Option<(OperationId, String, u64, EffectState)>,
}

/// Only a known transaction/confirmation of the SAME retained plan creates this.
/// It is consumed by the actual actor, not serialized or used as review approval.
pub(crate) struct NativeTerminalCommit {
    terminal: Arc<NativeTerminal>,
    phase: Arc<NativePhaseSession>,
    unit: ExecutionUnit,
    receipt: NativeResultReceipt,
    session: Session,
    version: u64,
    owned_success: bool,
}
impl NativeTerminalCommit {
    pub(crate) fn into_parts(
        self,
    ) -> (
        Arc<NativeTerminal>,
        Arc<NativePhaseSession>,
        ExecutionUnit,
        NativeResultReceipt,
        Session,
        u64,
        bool,
    ) {
        (
            self.terminal,
            self.phase,
            self.unit,
            self.receipt,
            self.session,
            self.version,
            self.owned_success,
        )


## INLINE INPUT EXCERPT-terminal-progress
{"id": "EXCERPT-terminal-progress", "bytes": 5321, "sha256": "7718159610f663c1ec08358ba834baf7e01f8ef8cf95289312c9727c0f924743", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/execution/native_phase/terminal.rs", "blob": "703082c30db807852839d3108341a18ca97dc90c", "whole_file_sha256": "37838036a6c0c45dd911c0e4e0a64ff53eeb9fc8503c995013f1f37060bbbf1d", "start": 472, "end": 595, "excerpt_sha256": "7718159610f663c1ec08358ba834baf7e01f8ef8cf95289312c9727c0f924743"}
    let mut after = unit.value.clone();
    if after.native_effects_open || after.work.is_none() {
        if after.work.is_none() {
            after.work = Some(if owned {
                receipt.observed_work
            } else {
                WorkOutcome::Unknown
            });
            after.disposition = if owned
                || (receipt.observed_work == WorkOutcome::Unknown
                    && receipt.disposition != Disposition::Completed)
            {
                receipt.disposition
            } else {
                Disposition::Lost
            };
            after.state = if after.work == Some(WorkOutcome::Unknown) {
                UnitState::WorkUnknown
            } else {
                UnitState::WorkKnown
            };
        }
        after.native_effects_open = false;
        if !owned || after.disposition != Disposition::Completed {
            after.result_finalization_open = false;
        }
        after.wait_reason = match after.disposition {
            Disposition::QuotaInterrupted => Some(WaitReason::Quota),
            Disposition::CapacityInterrupted => Some(WaitReason::Capacity),
            _ => None,
        };
        after.capacity_retry_at = (after.disposition == Disposition::CapacityInterrupted)
            .then(|| now_ms().saturating_add(60_000));
        after.version = after
            .version
            .checked_add(1)
            .context("terminal Unit version exhausted")?;
        after.updated_at = now_ms();
    }
    let unit_after = UnitImage::encoded(after)?;
    let mut n = invocation.value.clone();
    // This is only factual projection of the retained actual ACK. An unconfirmed
    // input remains historical and cannot become an OwnedPhaseSettlement.
    n.native_thread = receipt.native_thread.clone();
    n.native_turn = receipt.native_turn.clone();
    n.state = InvocationState::Closed;
    n.version = n
        .version
        .checked_add(1)
        .context("terminal invocation version exhausted")?;
    let invocation_after = InvocationImage::encoded(n)?;
    let mut record: Record = serde_json::from_value(session.body()?)?;
    let mut closed_session: Session = serde_json::from_value(record.data.clone())?;
    if closed_session.native_ref.is_none() {
        closed_session.native_ref = receipt.native_thread.clone();
    }
    let mut session_after = PairRow {
        table: session.table,
        values: session.values.clone(),
    };
    if !session_terminal(closed_session.state) && closed_session.state != SessionState::Lost {
        closed_session.state = if unit_after.value.disposition == Disposition::Cancelled {
            SessionState::Stopped
        } else if unit_after.value.work == Some(WorkOutcome::Failure) {
            SessionState::Failed
        } else if unit_after.value.work == Some(WorkOutcome::Unknown) {
            SessionState::Lost
        } else {
            SessionState::Exited
        };
    }
    if serde_json::to_value(&closed_session)? != record.data {
        record.version = record
            .version
            .checked_add(1)
            .context("terminal Session version exhausted")?;
        record.updated_at = now_ms();
        record.data = serde_json::to_value(&closed_session)?;
        session_after.replace("version", SqlValue::Integer(i64::try_from(record.version)?))?;
        session_after.set_body(&serde_json::to_value(&record)?)?;
    }
    let mut readiness_after = PairRow {
        table: readiness.table,
        values: readiness.values.clone(),
    };
    let mut body = readiness_after.body()?;
    let version = body["version"]
        .as_u64()
        .context("readiness version absent")?
        .checked_add(1)
        .context("readiness version exhausted")?;
    body["state"] = json!("closed");
    body["start_ended"] = json!(true);
    body["known_terminal"] = json!(true);
    body["version"] = json!(version);
    readiness_after.replace("state", SqlValue::Text("closed".into()))?;
    readiness_after.replace("start_ended", SqlValue::Integer(1))?;
    readiness_after.replace("known_terminal", SqlValue::Integer(1))?;
    readiness_after.replace("version", SqlValue::Integer(i64::try_from(version)?))?;
    readiness_after.set_body(&body)?;
    let admission_after = admission
        .as_ref()
        .map(|a| -> Result<PairRow> {
            let mut next = PairRow {
                table: a.table,
                values: a.values.clone(),
            };
            let mut body = next.body()?;
            let version = body["version"]
                .as_u64()
                .context("admission version absent")?
                .checked_add(1)
                .context("admission version exhausted")?;
            let settled = body["confirmed"] == true
                && input_effect
                    .as_ref()
                    .is_some_and(|e| e.3 == EffectState::Confirmed);
            body["settled"] = json!(settled);
            body["uncertain"] = json!(!settled);
            body["version"] = json!(version);
            next.replace("settled", SqlValue::Integer(i64::from(settled)))?;
            next.replace("uncertain", SqlValue::Integer(i64::from(!settled)))?;
            next.replace("version", SqlValue::Integer(i64::try_from(version)?))?;
            next.set_body(&body)?;


## INLINE INPUT EXCERPT-terminal-delivery
{"id": "EXCERPT-terminal-delivery", "bytes": 2001, "sha256": "d5ade4c0d687600f5eee5aaee69a4334853a48d3e38e94fd940b5d84842cd4e4", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/native.rs", "blob": "5648a9559a29cef3d68d134ed54d004b38087bb1", "whole_file_sha256": "4095b9d0a45a40412c25ea1bdc8a59c4615cf27fad94858bf04b5dcda8388f30", "start": 1214, "end": 1270, "excerpt_sha256": "d5ade4c0d687600f5eee5aaee69a4334853a48d3e38e94fd940b5d84842cd4e4"}
    if let Some(original) = frozen.as_ref() {
        ensure!(
            Arc::ptr_eq(original, proof),
            "native terminal compare-clear origin changed"
        );
        *frozen = None;
    }
    Ok(())
}
fn persist_saved_terminal(
    owner: &RuntimeOwner,
    phase: Option<&Arc<phase_protocol::PhaseActor>>,
    terminal: &Arc<NativeTerminal>,
) -> Result<(
    ExecutionUnit,
    native_result::NativeResultReceipt,
    Session,
    u64,
)> {
    if let Some(phase) = phase {
        let original = phase.terminal_plan(owner, terminal)?;
        let first = owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .finish_phase_terminal(&original);
        let commit = match first {
            Ok(commit) => commit,
            Err(error) => {
                // A failed commit might actually have committed. Only actual
                // receipt absence permits replacing its original plan. At most
                // one confirmed-rollback replan is attempted per observation.
                if !original.confirmed_absent(owner)? {
                    return Err(error);
                }
                let next = phase.replan_terminal_after_absence(owner, &original, terminal)?;
                owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .finish_phase_terminal(&next)?
            }
        };
        let (saved, actual_owner, unit, receipt, session, version, owned_success) =
            commit.into_parts();
        ensure!(
            Arc::ptr_eq(&saved, terminal) && Arc::ptr_eq(&actual_owner, &phase.owner),
            "native terminal transaction changed private origin"
        );
        if owned_success {
            phase.settled(
                saved,
                unit.clone(),
                receipt.clone(),
                session.clone(),
                version,
            )?;


## INLINE INPUT EXCERPT-normal-session
{"id": "EXCERPT-normal-session", "bytes": 1361, "sha256": "fb913ecd41fe74c38bcabdfdb0f4b3560149d186ae32dd16715c2a541ad272cd", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/managed_binding/binding.rs", "blob": "310a55a2a30ed1dc09ecc7b807561af10428790a", "whole_file_sha256": "3ec2e6c63efdbea440a7f9f2473ff8af8ed7787aa81cd836ada30d83de915978", "start": 75, "end": 103, "excerpt_sha256": "fb913ecd41fe74c38bcabdfdb0f4b3560149d186ae32dd16715c2a541ad272cd"}
            && record.version > 0
            && record.version <= i64::MAX as u64
            && session.id == f.session_id
            && session.scope == *f.scope
            && session.agent == f.alias
            && session.provider == f.provider
            && session.role == f.role
            && session.worktree == f.path
            && session.model.as_deref() == f.model
            && session.effort.as_deref() == f.effort
            && session.started_at == proof.session().started_at
            && proof
                .session()
                .native_ref
                .as_ref()
                .is_none_or(|id| session.native_ref.as_ref() == Some(id))
            && matches!(
                session.state,
                SessionState::Starting
                    | SessionState::Running
                    | SessionState::WaitingApproval
                    | SessionState::WaitingHuman
            ),
        "own latest Session immutable identity or normal eligibility changed"
    );
    // This current snapshot is not a stale returned Session-version CAS. PID,
    // initial native UUID and lifecycle can have advanced before this read.
    let exact: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM records WHERE id=?1 AND kind='session' AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND version=?5 AND body=?6)",


## INLINE INPUT EXCERPT-normal-eligibility
{"id": "EXCERPT-normal-eligibility", "bytes": 2865, "sha256": "d5950e8f10abbd90f87a684b2ea89ebda29e9197a171ced7194bdbbba171903e", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/managed_binding/binding.rs", "blob": "310a55a2a30ed1dc09ecc7b807561af10428790a", "whole_file_sha256": "3ec2e6c63efdbea440a7f9f2473ff8af8ed7787aa81cd836ada30d83de915978", "start": 158, "end": 223, "excerpt_sha256": "d5950e8f10abbd90f87a684b2ea89ebda29e9197a171ced7194bdbbba171903e"}
fn normal_eligibility(
    proof: &NativePhaseBinding,
    current: &CurrentWorkflowSuccessor,
) -> Result<()> {
    let f = proof.allocation().facts();
    let unit = current.unit();
    ensure!(
        proof.is_live()
            && proof.owner().launch_parts().is_retained()
            && std::ptr::eq(proof.marker().allocation().as_ref(), proof.allocation())
            && proof.settlement().is_none()
            && unit.session_id == Some(f.session_id)
            && unit.native_effects_open
            && unit.result_finalization_open
            && unit.work.is_none()
            && unit.disposition == Disposition::Active
            && matches!(
                unit.state,
                UnitState::DispatchPending | UnitState::Running | UnitState::WaitingQuota
            ),
        "normal binding lacks genuine live registration; terminal settlement requires its own predicate"
    );
    Ok(())
}

fn binding_invocation(
    c: &Connection,
    proof: &NativePhaseBinding,
    current: &CurrentWorkflowSuccessor,
) -> Result<Body<NativeInvocation>> {
    let f = proof.allocation().facts();
    let raw: Option<String> = c.query_row(
        "SELECT CASE WHEN length(CAST(body AS BLOB))<=?2 THEN body END FROM native_invocations WHERE id=?1",
        params![f.invocation_id.to_string(),native_result::INVOCATION_BYTES],|r|r.get(0),
    )?;
    let body = Body::<NativeInvocation>::decode(
        raw.context("registered invocation body over bound")?,
        native_result::INVOCATION_BYTES,
    )?;
    let invocation = body.parsed();
    invocation.validate()?;
    ensure!(
        invocation.id == f.invocation_id
            && invocation.unit_id == f.unit_id
            && invocation.session_id == f.session_id
            && invocation.scope == *f.scope
            && invocation.generation == f.generation
            && invocation.owner_epoch == f.epoch
            && invocation.provider == f.provider
            && invocation.state != InvocationState::Closed
            && invocation.unit_version >= f.unit_version
            && invocation.unit_version <= current.unit().version
            && invocation.context_version == Some(f.input.version)
            && invocation.context_sha256.as_deref()
                == Some(
                    native_result::digest(&serde_json::to_vec(
                        proof.marker().original_plan().context().0
                    )?)
                    .as_str()
                )
            && invocation.source_versions == f.input.source_versions
            && invocation.source_sha256
                == native_result::digest(&serde_json::to_vec(&f.input.source_versions)?)
            && invocation.revision == f.input.revision
            && invocation.artifact_id == f.artifact
            && invocation.payload_sha256 == native_result::digest(f.input.payload.as_bytes()),


## INLINE INPUT EXCERPT-normal-write
{"id": "EXCERPT-normal-write", "bytes": 1241, "sha256": "6e54723ac15c8bde0710c6c8b8308dd7ec2eade427fc9d1d8e87fdd578873df6", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/managed_binding/binding.rs", "blob": "310a55a2a30ed1dc09ecc7b807561af10428790a", "whole_file_sha256": "3ec2e6c63efdbea440a7f9f2473ff8af8ed7787aa81cd836ada30d83de915978", "start": 389, "end": 408, "excerpt_sha256": "6e54723ac15c8bde0710c6c8b8308dd7ec2eade427fc9d1d8e87fdd578873df6"}
                .facts()
                .state_path
                .to_str()
                .is_some_and(|path| self.connection.path() == Some(path)),
            "Session binder is not the selected owner's database"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        validate_current_tx(&tx, plan.proof.marker(), &plan.current)?;
        plan.proof.marker().validate_driver_live_tx(&tx)?;
        normal_eligibility(&plan.proof, &plan.current)?;
        validate_registered_owner_tx(&tx, &plan.proof, &plan.owner_raw)?;
        validate_invocation_tx(&tx, &plan.invocation)?;
        let record = plan.session.parsed();
        let exact: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE id=?1 AND kind='session' AND project_id=?2 AND goal_id IS ?3 AND task_id IS ?4 AND version=?5 AND body=?6)",params![record.id.to_string(),record.scope.project_id.to_string(),record.scope.goal_id.map(|v|v.to_string()),record.scope.task_id.map(|v|v.to_string()),record.version,plan.session.raw()],|r|r.get(0))?;
        ensure!(
            exact,
            "current latest Session advanced; plan again without refreshing original pins"
        );


## INLINE INPUT EXCERPT-git-run
{"id": "EXCERPT-git-run", "bytes": 1571, "sha256": "4d03995fdd99544646ca02dc3b9cca7bef3cef5a7c666d9d57b5ec58df493860", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/git_io.rs", "blob": "a8c0d50371c3a344833bcebb4f64cab1c1ba79e6", "whole_file_sha256": "07942394dcef7dc45b05086532ed9d195bfe69dd1183841a6a96f4308f0dc2e7", "start": 135, "end": 173, "excerpt_sha256": "4d03995fdd99544646ca02dc3b9cca7bef3cef5a7c666d9d57b5ec58df493860"}
    async fn run_command(
        &self,
        root: &Path,
        mut command: tokio::process::Command,
        kind: &str,
    ) -> Result<process::CommandCapture> {
        command
            .envs(
                self.profile
                    .environment(&self.unit.cookie, self.owner.ipc_path())?,
            )
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let operation = OperationId::new();
        if let Some(lease) = &self.git_lease {
            command.env("RRX_GIT_GATE_TOKEN", lease.id.to_string());
        } else {
            command.env_remove("RRX_GIT_GATE_TOKEN");
        }
        let child = {
            let mut store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            let current = store.execution_unit(self.unit.id)?;
            ensure!(
                current.scope == self.unit.scope
                    && current.generation == self.unit.generation
                    && current.owner_epoch == self.unit.owner_epoch
                    && current.session_id == self.unit.session_id,
                "Git preparation authority retired"
            );
            store.validate_execution(&current.authority(), self.native, !self.native)?;
            // Capture/inspection has Runtime-only finalization authority; never grant native tools.
            store.reserve_execution_helper_pinned(
                &current.authority(),
                operation,


## INLINE INPUT EXCERPT-git-monitor-call
{"id": "EXCERPT-git-monitor-call", "bytes": 1475, "sha256": "2cda2b8d5787009ff7ed7bbf05c6730859c80835d83de6d3c8e3d8b02aa448c2", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/git_io.rs", "blob": "a8c0d50371c3a344833bcebb4f64cab1c1ba79e6", "whole_file_sha256": "07942394dcef7dc45b05086532ed9d195bfe69dd1183841a6a96f4308f0dc2e7", "start": 192, "end": 239, "excerpt_sha256": "2cda2b8d5787009ff7ed7bbf05c6730859c80835d83de6d3c8e3d8b02aa448c2"}
        let mut helper_guard = owner::HelperGuard::new(self.owner.clone(), operation);
        let observed = process::capture_scoped_pinned(
            child,
            &self.owner,
            &self.unit,
            self.native,
            self.driver.as_ref(),
        )
        .await;
        let mut receipt = BTreeMap::new();
        if let Ok(o) = &observed {
            receipt.insert(
                "exit".into(),
                o.receipt
                    .status
                    .code()
                    .map_or_else(|| "signal".into(), |c| c.to_string()),
            );
            receipt.insert(
                "group_cleanup".into(),
                if o.receipt.group_error.is_some() {
                    "unknown"
                } else {
                    "requested"
                }
                .into(),
            );
        }
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .reconcile_managed_effect_pinned(
                operation,
                1,
                if observed.is_ok() {
                    EffectState::Confirmed
                } else {
                    EffectState::Unknown
                },
                receipt,
                if observed.is_ok() {
                    self.driver.as_ref()
                } else {
                    None
                },
            )?;
        helper_guard.disarm();


## INLINE INPUT EXCERPT-process-monitor
{"id": "EXCERPT-process-monitor", "bytes": 1086, "sha256": "ae774bf579377ae2da6ebe9de1b95f1cff7c567d5e546d47834ed4cf33e40eb8", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/process.rs", "blob": "f5a63f7dcfb890142bba6b849f0962cec39f71b5", "whole_file_sha256": "249f046912cad6b757f063b3547f262043f2119c304e86c710b4fb0add2bf91f", "start": 357, "end": 379, "excerpt_sha256": "ae774bf579377ae2da6ebe9de1b95f1cff7c567d5e546d47834ed4cf33e40eb8"}
pub(crate) async fn capture_scoped_pinned(
    child: OwnedProcess,
    owner: &super::RuntimeOwner,
    pinned: &super::ExecutionUnit,
    native: bool,
    driver: Option<&crate::state::DriverReadTicket>,
) -> Result<CommandCapture> {
    let capture = capture_child(child);
    tokio::pin!(capture);
    let mut fence = tokio::time::interval(Duration::from_millis(50));
    loop {
        tokio::select! {
            observed = &mut capture => return observed,
            _ = fence.tick() => {
                let mut store = owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?;
                if let Some(ticket) = driver { store.validate_driver_read(ticket)?; }
                let current = store.execution_unit(pinned.id)?;
                ensure!(current.scope == pinned.scope && current.generation == pinned.generation && current.owner_epoch == pinned.owner_epoch && current.session_id == pinned.session_id, "helper execution identity changed");
                store.validate_execution(&current.authority(), native, !native)?;
            }
        }
    }
}


## INLINE INPUT EXCERPT-marker-refusal
{"id": "EXCERPT-marker-refusal", "bytes": 530, "sha256": "31714e68e8f350e8784d2cbbd2bcb147f3652b440af468291ccf84f3f223d007", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/runtime/driver.rs", "blob": "18db97d7437cd3612355926002003fba5cbc3cba", "whole_file_sha256": "629e1c2dadd192e9597d5b87982f0961e99e2de6a827065c06613d7d5f66e153", "start": 331, "end": 343, "excerpt_sha256": "31714e68e8f350e8784d2cbbd2bcb147f3652b440af468291ccf84f3f223d007"}
    ensure!(epoch > 0 && epoch == current_epoch, "Driver epoch retired");
    let live: bool = c.query_row(
        "SELECT rrx_live_task_driver(?1,?2,?3,?4,?5)",
        params![task_id.to_string(), id, epoch, version, body],
        |r| r.get(0),
    )?;
    ensure!(live, "actual retained Task Driver unavailable");
    let row: Row = decode(body)?;
    ensure!(
        row.marker.is_none(),
        "marker-bound Driver requires the genuine managed successor/lifecycle reader"
    );
    let current = snapshot(c, task_id)?;


## INLINE INPUT EXCERPT-gate-final
{"id": "EXCERPT-gate-final", "bytes": 2793, "sha256": "3fb694d83e9d002f9ed20eb69d8403e24c16db98a83a00b1a1e4271bf77d5a5f", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/workflow_gates.rs", "blob": "8bc39714fad0d82fb7df6527a8e0d2d2109a194b", "whole_file_sha256": "83244e016d798395aece728b3d61d60e210514b7e65e09ceb719586413b34785", "start": 323, "end": 388, "excerpt_sha256": "3fb694d83e9d002f9ed20eb69d8403e24c16db98a83a00b1a1e4271bf77d5a5f"}
            _ => {
                return Ok(GateOutcome::Waiting(format!(
                    "{} requires its qualified production evidence integration",
                    phase.key()
                )));
            }
        }
        if let Some(artifact) = &checked_artifact {
            ensure!(
                invocation.sources.artifact == Some(artifact.id)
                    && artifact.scope == invocation.task.scope()
                    && artifact.revision == invocation.sources.revision
                    && artifact.dependencies == invocation.sources.source_versions,
                "gate retained artifact differs from observed input"
            );
            ResultStore::new(self.owner.clone())
                .verify(artifact)
                .await?;
        }
        let current = self
            .sources
            .capture(
                invocation.project.clone(),
                invocation.task.clone(),
                phase,
                invocation.budget.clone(),
            )
            .await?;
        ensure!(
            current == invocation.sources,
            "gate source changed during checks"
        );
        let mut store = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?;
        ensure!(
            Self::claim(&store, &invocation)?.0 == claim,
            "gate claim changed during checks"
        );
        if let Some(unit) = &checked_unit {
            ensure!(
                serde_json::to_value(store.validate_execution(&unit.authority(), false, true)?)?
                    == serde_json::to_value(unit)?,
                "gate unit changed during checks"
            );
        }
        if let Some(artifact) = &checked_artifact {
            ensure!(
                serde_json::to_value(store.result_artifact(artifact.id)?)?
                    == serde_json::to_value(artifact)?,
                "gate artifact changed during checks"
            );
        }
        let mut receipt = Record::new(
            invocation.task.scope(),
            RecordKind::Verification,
            json!({"schema":"managed_workflow_gate_v1","claim":claim,"phase":phase,
                "revision":invocation.sources.revision,"sources":invocation.sources.source_versions,
                "launch_revision":invocation.context.revision,
                "context_data_sha256":digest(&serde_json::to_vec(&invocation.context.data)?),
                "unit":checked_unit.as_ref().map(ManagedUnitRef::from),
                "profile_digest":checked_unit.as_ref().map(|u| &u.profile_digest),
                "artifact":checked_artifact.as_ref().map(|a| a.id),
                "manifest_sha256":checked_artifact.as_ref().map(|a| &a.manifest_sha256)}),


## INLINE INPUT EXCERPT-store-shape
{"id": "EXCERPT-store-shape", "bytes": 205, "sha256": "76ac62d064cd62baff0d8dbd60f8d620d2a6fb9e6ed8df7e1425328730bf1197", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/mod.rs", "blob": "f8bec61c9ab5258adaabad10525d4ce6fbe2cef0", "whole_file_sha256": "fbbd7f790809bf9fbaa4b9250af2aa589511e930d1883b1de3778f76cd98bb3f", "start": 88, "end": 92, "excerpt_sha256": "76ac62d064cd62baff0d8dbd60f8d620d2a6fb9e6ed8df7e1425328730bf1197"}
pub struct Store {
    connection: Connection,
    #[allow(dead_code)] // Actual managed marker/binder is composed separately.
    binding_permits: std::sync::Arc<managed_binding::PrivatePermitManager>,
}


## INLINE INPUT EXCERPT-admission-shape
{"id": "EXCERPT-admission-shape", "bytes": 254, "sha256": "2bb86fcdb723f19565e7178ad14766bc1cec54d7c5d6e7ef7352cb1358ae31d7", "label": "CA", "commit": "95ef0e783c0b18c90dfef23ba1b77b9dc67735f1", "source_path": "crates/rrx/src/runtime/installation.rs", "blob": "e0dab8d453a6bfa10700228f930d64b4ba291462", "whole_file_sha256": "8e39e03597afeb20a34e4e69101c071ad82432c09748f6daa82b603feb30ae3e", "start": 82, "end": 87, "excerpt_sha256": "2bb86fcdb723f19565e7178ad14766bc1cec54d7c5d6e7ef7352cb1358ae31d7"}
/// Stop exclusion only: no SQL permission or Native grant. Field order drops
/// admission before the last strong Runtime reference.
pub(crate) struct ActivationAdmission {
    _admission: tokio::sync::OwnedMutexGuard<()>,
    _runtime: Arc<Runtime>,
}


## INLINE INPUT EXCERPT-admission-acquire
{"id": "EXCERPT-admission-acquire", "bytes": 1781, "sha256": "7ac782cf815bbb224feac13c653065364724c236ab07dbfaaedd7468130b39af", "label": "CA", "commit": "95ef0e783c0b18c90dfef23ba1b77b9dc67735f1", "source_path": "crates/rrx/src/runtime/installation.rs", "blob": "e0dab8d453a6bfa10700228f930d64b4ba291462", "whole_file_sha256": "8e39e03597afeb20a34e4e69101c071ad82432c09748f6daa82b603feb30ae3e", "start": 148, "end": 194, "excerpt_sha256": "7ac782cf815bbb224feac13c653065364724c236ab07dbfaaedd7468130b39af"}
    pub(crate) async fn admit_activation(
        &self,
        plan: &Arc<DriverPreparationAdvance>,
        lifetime: &WorkerLifetime,
    ) -> Result<ActivationAdmission> {
        self.admit_activation_inner(plan, lifetime, true).await
    }
    pub(crate) async fn admit_activation_recovery(
        &self,
        plan: &Arc<DriverPreparationAdvance>,
        lifetime: &WorkerLifetime,
    ) -> Result<ActivationAdmission> {
        self.admit_activation_inner(plan, lifetime, false).await
    }
    async fn admit_activation_inner(
        &self,
        plan: &Arc<DriverPreparationAdvance>,
        lifetime: &WorkerLifetime,
        require_retained: bool,
    ) -> Result<ActivationAdmission> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("activation Runtime ended"))?;
        let admission = tokio::select! { biased;
            ()=lifetime.cancelled()=>return Err(anyhow::anyhow!("activation Driver cancelled")),
            guard=runtime.control_admission.clone().lock_owned()=>guard,
        };
        ensure!(
            runtime.service_running() && self.is_current(),
            "Runtime stopped before activation"
        );
        let association = lifetime.association()?;
        ensure!(
            plan.activation_roster()?.is_same_composition(self)
                && self.original_task.id == association.task()
                && plan.belongs_to(&association),
            "activation original composition/worker linkage differs"
        );
        if require_retained {
            ensure!(plan.is_retained()?, "activation preparation lost custody");
        }
        Ok(ActivationAdmission {
            _admission: admission,
            _runtime: runtime,
        })
    }


## INLINE INPUT EXCERPT-service-loop
{"id": "EXCERPT-service-loop", "bytes": 1415, "sha256": "3726adbdcdf75f24e09d356289a90b724deddcac296f71c6b724f1eed9af25ae", "label": "CA", "commit": "95ef0e783c0b18c90dfef23ba1b77b9dc67735f1", "source_path": "crates/rrx/src/runtime/service.rs", "blob": "6d1353ffe5731ff12a6ec75d6527f2bc10d52cde", "whole_file_sha256": "e6820a65d8d54781ee0e30f73027e98c2d6c1b847eac69b63546d0355063c758", "start": 73, "end": 105, "excerpt_sha256": "3726adbdcdf75f24e09d356289a90b724deddcac296f71c6b724f1eed9af25ae"}
            loop {
                let Some(runtime) = retained.upgrade() else {
                    return Ok(());
                };
                if runtime.stopping.load(Ordering::SeqCst) {
                    return Ok(());
                }
                let (next, more) = runtime
                    .owner
                    .store
                    .lock()
                    .map_err(|_| anyhow::anyhow!("state poisoned"))?
                    .reconcile_runtime_attention(
                        runtime.owner.instance_id(),
                        runtime.owner.epoch(),
                        sequence,
                    )?;
                sequence = next;
                let pending = runtime.phases.reconcile_pending()?;
                let driver_pending = runtime.observe_task_drivers()?;
                // A refusal after reservation ends only this saved-cursor
                // sweep. Its retained claim/closure owns the outcome.
                let claims = if more {
                    0
                } else {
                    runtime.admit_ready_tasks().unwrap_or_default()
                };
                let delay = super::phase_supervisor::PhaseSupervisor::delay(
                    pending || driver_pending > 0 || claims > 0,
                    &mut backoff,
                );
                let wake = runtime.wake.clone();
                drop(runtime);


## INLINE INPUT EXCERPT-driver-private
{"id": "EXCERPT-driver-private", "bytes": 656, "sha256": "b5dcfc01396b770e2e775b4717787bad6dd22b94204ec69b8f04bd3101e098ba", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/runtime/driver/marker.rs", "blob": "66771f508f0dd146f56b4465c75bbf4575851d10", "whole_file_sha256": "96ebf29fcf391d8b560173a7a5e3667924e5ad2e4d066a9e07aef6befc4e6862", "start": 10, "end": 30, "excerpt_sha256": "b5dcfc01396b770e2e775b4717787bad6dd22b94204ec69b8f04bd3101e098ba"}
pub(crate) struct DriverMarkerAdvance {
    ticket: Arc<DriverReadTicket>,
    next: Row,
    body: String,
    task: Task,
    task_body: String,
    workflow: Record,
    workflow_body: String,
    source: Option<SourceMarkerAdvance>,
}
/// Created only by checking a completed planned mutation on the same Store
/// after commit. Not Clone/Deserialize and never an input/Native credential.
pub(crate) struct DriverPublication {
    pub(super) task: TaskId,
    pub(super) id: Uuid,
    pub(super) epoch: u64,
    pub(super) before_version: u64,
    pub(super) before_body: String,
    pub(super) after_version: u64,
    pub(super) after_body: String,
}


## INLINE INPUT EXCERPT-driver-ports
{"id": "EXCERPT-driver-ports", "bytes": 1593, "sha256": "879536a4957ca30b8ffd3c1a0e2a4984c0681ff8eeb7595bb86e16ac850c8aea", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/runtime/driver/marker.rs", "blob": "66771f508f0dd146f56b4465c75bbf4575851d10", "whole_file_sha256": "96ebf29fcf391d8b560173a7a5e3667924e5ad2e4d066a9e07aef6befc4e6862", "start": 155, "end": 186, "excerpt_sha256": "879536a4957ca30b8ffd3c1a0e2a4984c0681ff8eeb7595bb86e16ac850c8aea"}
    pub(in crate::state) fn exact_mutations(&self) -> Result<Vec<ExactRowMutation>> {
        let mut mutations = vec![ExactRowMutation::new(
            "task_drivers",
            "UPDATE",
            Some(image(&self.ticket.row, &self.ticket.body)?),
            Some(image(&self.next, &self.body)?),
        )?];
        if let Some(source) = &self.source {
            mutations.push(source.mutation()?);
        }
        Ok(mutations)
    }
    /// Required BEFORE Root's Task/Workflow marker writes, in their same TX.
    pub(crate) fn validate_current_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        self.ticket.validate_current_tx(tx)
    }
    /// Required AFTER the fixed Task/W writes. Root supplies ONE exact permit
    /// batch covering every planned row; no nested manager or generic refresh.
    pub(crate) fn freeze_marker_tx(&self, tx: &Transaction<'_>) -> Result<()> {
        self.ticket.validate_ancillary_tx(tx)?;
        self.ticket.scope.validate_projection(
            tx,
            &self.task,
            &self.task_body,
            Some((&self.workflow, &self.workflow_body)),
        )?;
        if let Some(source) = &self.source {
            source.write_tx(tx)?;
        }
        ensure!(tx.execute("UPDATE task_drivers SET version=?1,body=?2 WHERE task_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='driving' AND body=?7",params![self.next.version,self.body,self.task.id.to_string(),self.next.id.to_string(),self.next.epoch,self.ticket.row.version,self.ticket.body])?==1,"Driver marker freeze CAS changed");
        Ok(())
    }


## INLINE INPUT EXCERPT-driver-publish
{"id": "EXCERPT-driver-publish", "bytes": 1637, "sha256": "84260e0f3d6a1e3696d4d4a6fc7e003adbdb21b381b036c60c6150a75a754ecf", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/runtime/driver/marker.rs", "blob": "66771f508f0dd146f56b4465c75bbf4575851d10", "whole_file_sha256": "96ebf29fcf391d8b560173a7a5e3667924e5ad2e4d066a9e07aef6befc4e6862", "start": 188, "end": 223, "excerpt_sha256": "84260e0f3d6a1e3696d4d4a6fc7e003adbdb21b381b036c60c6150a75a754ecf"}
impl Store {
    /// Known-commit publication while SharedStore remains excluded. A failed
    /// check leaves this exact owned plan with its caller, never adopts newer rows.
    pub(crate) fn publish_driver_marker(&mut self, plan: &DriverMarkerAdvance) -> Result<()> {
        ensure!(
            self.connection.is_autocommit(),
            "Driver publication precedes commit"
        );
        let tx = self.connection.transaction()?;
        plan.ticket.scope.validate_projection(
            &tx,
            &plan.task,
            &plan.task_body,
            Some((&plan.workflow, &plan.workflow_body)),
        )?;
        plan.ticket.validate_selection_tx(&tx)?;
        if let Some(source) = &plan.source {
            source.validate_result_tx(&tx)?;
        } else {
            plan.ticket.validate_source_tx(&tx)?;
        }
        let exact:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM task_drivers WHERE task_id=?1 AND id=?2 AND owner_epoch=?3 AND version=?4 AND state='driving' AND body=?5)",params![plan.task.id.to_string(),plan.next.id.to_string(),plan.next.epoch,plan.next.version,plan.body],|r|r.get(0))?;
        ensure!(exact, "planned committed Driver row differs");
        tx.commit()?;
        let publication = DriverPublication {
            task: plan.task.id,
            id: plan.next.id,
            epoch: plan.next.epoch,
            before_version: plan.ticket.row.version,
            before_body: plan.ticket.body.clone(),
            after_version: plan.next.version,
            after_body: plan.body.clone(),
        };
        plan.ticket.association.publish_exact(&publication)
    }
}


## INLINE INPUT EXCERPT-capture-catalogue
{"id": "EXCERPT-capture-catalogue", "bytes": 5294, "sha256": "09aece4e065afd151dcb5553362c88c0b4ebd07fe0d0ad5f0d713b8608314011", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/results.rs", "blob": "c4bdfc9c018b45479d327b764737f093c55074f2", "whole_file_sha256": "290baf091d847c9fd2ba3e900d67e5d9326cd50c7c741e45e3742534f8cec847", "start": 224, "end": 369, "excerpt_sha256": "09aece4e065afd151dcb5553362c88c0b4ebd07fe0d0ad5f0d713b8608314011"}
            .project(unit.scope.project_id)?
            .context("Project missing")?;
        let source = self
            .owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .task(unit.scope.task_id.context("Task missing")?)?
            .context("Task missing")?;
        let io = UnitGit::new(self.owner.clone(), &unit, false)?;
        io.ownership(&project, &source).await?;
        let format = io
            .text(&unit.worktree, ["rev-parse", "--show-object-format"])
            .await?;
        ensure!(
            matches!(format.as_str(), "sha1" | "sha256"),
            "unsupported Git object format"
        );
        ensure!(
            revision.len() == if format == "sha1" { 40 } else { 64 },
            "OID format mismatch"
        );
        io.run(
            &unit.worktree,
            [
                "merge-base",
                "--is-ancestor",
                unit.base_sha.as_str(),
                revision,
            ],
        )
        .await?;
        ensure!(
            io.text(
                &unit.worktree,
                ["rev-parse", "--verify", &format!("{revision}^{{commit}}")]
            )
            .await?
                == revision,
            "result is not exact commit"
        );
        qualified_content_scoped(&unit.worktree, revision, &io).await?;
        let _guard = self.gate.lock().await;
        let repository = self
            .owner
            .root
            .join("projects")
            .join(project.id.to_string())
            .join("results.git");
        let id = ArtifactId::new();
        let directory = self.owner.root.join("artifacts").join(id.to_string());
        let manifest = directory.join("manifest.json");
        let mut artifact = ResultArtifact {
            id,
            scope: unit.scope.clone(),
            unit_id: unit.id,
            state: ArtifactState::Staging,
            revision: revision.into(),
            base_sha: unit.base_sha.clone(),
            object_format: format.clone(),
            repository: repository.clone(),
            manifest,
            manifest_sha256: String::new(),
            dependencies: sources.clone(),
            version: 1,
            created_at: crate::domain::now_ms(),
        };
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .stage_result(authority, &artifact)?;
        std::fs::create_dir_all(repository.parent().context("result parent missing")?)?;
        ensure!(
            !repository.starts_with(&unit.worktree) && !unit.worktree.starts_with(&repository),
            "results overlap executor"
        );
        if !repository.exists() {
            io.run(
                repository.parent().context("repository parent missing")?,
                [
                    "init",
                    "--bare",
                    &format!("--object-format={format}"),
                    repository.to_str().context("repository UTF-8")?,
                ],
            )
            .await?;
        }
        retained_storage(&repository)?;
        ensure!(
            io.text(&repository, ["rev-parse", "--show-object-format"])
                .await?
                == format,
            "repository format changed"
        );
        // fetch copies object graphs over upload-pack. No clone-local hardlinks or alternates.
        io.run(
            &repository,
            [
                "-c",
                "fetch.fsckObjects=true",
                "fetch",
                "--no-tags",
                "--no-write-fetch-head",
                "--no-recurse-submodules",
                "--",
                unit.worktree.to_str().context("worktree UTF-8")?,
                &format!("{revision}:refs/rrx/{id}/commit"),
                &format!("{}:refs/rrx/{id}/base", unit.base_sha),
            ],
        )
        .await?;
        retained_storage(&repository)?;
        io.run(&repository, ["fsck", "--full", "--strict", "--no-dangling"])
            .await?;
        for oid in [revision, unit.base_sha.as_str()] {
            io.run(
                &repository,
                ["rev-list", "--objects", "--missing=error", oid],
            )
            .await?;
        }
        std::fs::create_dir_all(directory.parent().context("artifact root missing")?)?;
        std::fs::create_dir(&directory)?;
        let bytes = serde_json::to_vec(&ResultManifest {
            artifact: id,
            unit: unit.id,
            revision: revision.into(),
            base: unit.base_sha.clone(),
            object_format: format,
            sources,
        })?;
        durable_file(&artifact.manifest, &bytes)?;
        sync_tree(&repository)?;
        File::open(repository.parent().context("repository parent missing")?)?.sync_all()?;
        File::open(directory.parent().context("artifact parent missing")?)?.sync_all()?;
        artifact.manifest_sha256 = hex(&bytes);
        artifact.state = ArtifactState::Ready;
        artifact.version = 2;
        self.owner
            .store
            .lock()
            .map_err(|_| anyhow::anyhow!("state poisoned"))?
            .ready_result(&artifact, 1)?;
        Ok(artifact)


## INLINE INPUT EXCERPT-capture-ownership
{"id": "EXCERPT-capture-ownership", "bytes": 1635, "sha256": "5c5ba89e10eb4816469d4c87c70dcef879a638549e2c96048b325cc40c77cfa8", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/git_io.rs", "blob": "a8c0d50371c3a344833bcebb4f64cab1c1ba79e6", "whole_file_sha256": "07942394dcef7dc45b05086532ed9d195bfe69dd1183841a6a96f4308f0dc2e7", "start": 249, "end": 294, "excerpt_sha256": "5c5ba89e10eb4816469d4c87c70dcef879a638549e2c96048b325cc40c77cfa8"}
    pub(crate) async fn ownership(
        &self,
        project: &crate::domain::Project,
        task: &crate::domain::Task,
    ) -> Result<()> {
        let root = &project.root;
        let path = task.worktree.as_deref().context("Task worktree missing")?;
        let source_top = PathBuf::from(self.text(root, ["rev-parse", "--show-toplevel"]).await?);
        let source_git_dir = PathBuf::from(
            self.text(root, ["rev-parse", "--path-format=absolute", "--git-dir"])
                .await?,
        );
        let source_common = PathBuf::from(
            self.text(
                root,
                ["rev-parse", "--path-format=absolute", "--git-common-dir"],
            )
            .await?,
        );
        let source_roots = self
            .text(
                root,
                [
                    "rev-list",
                    "--max-parents=0",
                    &format!("refs/heads/{}", project.base_branch),
                ],
            )
            .await?
            .lines()
            .map(str::to_owned)
            .collect();
        let task_top = PathBuf::from(self.text(path, ["rev-parse", "--show-toplevel"]).await?);
        let task_common = PathBuf::from(
            self.text(
                path,
                ["rev-parse", "--path-format=absolute", "--git-common-dir"],
            )
            .await?,
        );
        let branch = self
            .text(path, ["symbolic-ref", "--quiet", "--short", "HEAD"])
            .await?;
        let revision = self
            .text(path, ["rev-parse", "--verify", "HEAD^{commit}"])
            .await?;


## INLINE INPUT EXCERPT-capture-content
{"id": "EXCERPT-capture-content", "bytes": 1522, "sha256": "2fb2417b8eaada5db2f3ceea2a226b5f11213366d64424ea73ef5fbab76f4250", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/results.rs", "blob": "c4bdfc9c018b45479d327b764737f093c55074f2", "whole_file_sha256": "290baf091d847c9fd2ba3e900d67e5d9326cd50c7c741e45e3742534f8cec847", "start": 777, "end": 818, "excerpt_sha256": "2fb2417b8eaada5db2f3ceea2a226b5f11213366d64424ea73ef5fbab76f4250"}
async fn qualified_content_inner(root: &Path, revision: &str, io: &UnitGit) -> Result<()> {
    let tree = read_git(root, ["ls-tree", "-r", "-z", revision], io).await?;
    for line in tree.split(|b| *b == 0).filter(|s| !s.is_empty()) {
        ensure!(
            !line.starts_with(b"160000 "),
            "submodules require a qualified profile"
        );
        ensure!(
            !line.starts_with(b"120000 "),
            "source symlinks require a qualified profile"
        );
    }
    // LFS pointers are ordinary Git blobs, not their required external content.
    let args = [
        "grep",
        "-l",
        "-I",
        "-e",
        "version https://git-lfs.github.com/spec/v1",
        revision,
        "--",
    ];
    let observed = io.run_observed(root, args).await?;
    ensure!(
        observed.receipt.status.code() == Some(1)
            || (observed.receipt.status.success() && observed.stdout.is_empty()),
        "LFS pointer scan found unsupported content or failed"
    );
    let attrs = read_git(root, ["ls-tree", "-r", "--name-only", revision], io).await?;
    for name in std::str::from_utf8(&attrs)?
        .lines()
        .filter(|n| n.ends_with(".gitattributes"))
    {
        let bytes = read_git(root, ["show", &format!("{revision}:{name}")], io).await?;
        ensure!(
            !bytes
                .windows(b"filter=lfs".len())
                .any(|w| w == b"filter=lfs"),
            "LFS attributes require a qualified profile"
        );
    }
    Ok(())


## INLINE INPUT EXCERPT-capture-verify
{"id": "EXCERPT-capture-verify", "bytes": 5011, "sha256": "988c46f2cc48fb4eb1ec723a389f5d28b784cab1054b1b40d12b86fef8e41871", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/results.rs", "blob": "c4bdfc9c018b45479d327b764737f093c55074f2", "whole_file_sha256": "290baf091d847c9fd2ba3e900d67e5d9326cd50c7c741e45e3742534f8cec847", "start": 371, "end": 509, "excerpt_sha256": "988c46f2cc48fb4eb1ec723a389f5d28b784cab1054b1b40d12b86fef8e41871"}
    pub async fn verify(&self, artifact: &ResultArtifact) -> Result<()> {
        let io = RetainedGit::new(self.owner.clone(), artifact)?;
        self.verify_inner(artifact, RetainedReader::Historical(&io))
            .await?;
        io.validate()
    }
    pub(crate) async fn verify_recovery(
        &self,
        artifact: &ResultArtifact,
        binding: &crate::state::SourceReadBinding,
    ) -> Result<()> {
        let io = RetainedGit::for_recovery(self.owner.clone(), artifact, binding.clone())?;
        self.verify_inner(artifact, RetainedReader::Historical(&io))
            .await?;
        io.validate()
    }
    pub(crate) async fn workflow_publication(
        &self,
        authority: &ExecutionAuthority,
        artifact: ArtifactId,
    ) -> Result<WorkflowPublication> {
        let (unit, artifact) = {
            let store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            (
                store.validate_execution(authority, false, true)?,
                store.result_artifact(artifact)?,
            )
        };
        ensure!(
            unit.kind == UnitKind::Executor
                && unit.work == Some(WorkOutcome::Success)
                && !unit.native_effects_open
                && artifact.unit_id == unit.id
                && artifact.scope == unit.scope
                && artifact.state == ArtifactState::Ready,
            "publication verification requires exact successful executor artifact"
        );
        let io = UnitGit::new(self.owner.clone(), &unit, false)?;
        self.verify_inner(&artifact, RetainedReader::Current(&io))
            .await?;
        {
            let store = self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?;
            store.validate_execution(authority, false, true)?;
            ensure!(
                serde_json::to_value(store.result_artifact(artifact.id)?)?
                    == serde_json::to_value(&artifact)?,
                "artifact changed during publication verification"
            );
        }
        Ok(WorkflowPublication {
            authority: authority.clone(),
            artifact,
        })
    }
    async fn verify_inner(&self, artifact: &ResultArtifact, io: RetainedReader<'_>) -> Result<()> {
        ensure!(
            matches!(
                artifact.state,
                ArtifactState::Ready | ArtifactState::Published
            ),
            "artifact is not usable"
        );
        ensure!(
            artifact.repository
                == self
                    .owner
                    .root
                    .join("projects")
                    .join(artifact.scope.project_id.to_string())
                    .join("results.git")
                && artifact.manifest
                    == self
                        .owner
                        .root
                        .join("artifacts")
                        .join(artifact.id.to_string())
                        .join("manifest.json"),
            "artifact storage mismatch"
        );
        let mut bytes = Vec::new();
        File::open(&artifact.manifest)?
            .take(128 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() <= 128 * 1024 && hex(&bytes) == artifact.manifest_sha256,
            "artifact manifest corruption"
        );
        let m: ResultManifest = serde_json::from_slice(&bytes)?;
        ensure!(
            m.artifact == artifact.id
                && m.unit == artifact.unit_id
                && m.revision == artifact.revision
                && m.base == artifact.base_sha
                && m.object_format == artifact.object_format
                && m.sources == artifact.dependencies,
            "artifact manifest identity mismatch"
        );
        retained_storage(&artifact.repository)?;
        for (name, oid) in [("commit", &artifact.revision), ("base", &artifact.base_sha)] {
            ensure!(
                text(
                    &retained_git(
                        &artifact.repository,
                        &io,
                        [
                            "rev-parse",
                            "--verify",
                            &format!("refs/rrx/{}/{name}", artifact.id)
                        ]
                    )
                    .await?
                )? == *oid,
                "retained reference changed"
            );
            retained_git(
                &artifact.repository,
                &io,
                ["rev-list", "--objects", "--missing=error", oid],
            )
            .await?;
        }
        retained_git(
            &artifact.repository,
            &io,
            ["fsck", "--full", "--strict", "--no-dangling"],
        )
        .await?;
        Ok(())
    }
    pub async fn publish(
        &self,


## INLINE INPUT EXCERPT-corpus-bound
{"id": "EXCERPT-corpus-bound", "bytes": 1732, "sha256": "e4361d599c030290aa0f610fc0a502bb2029fa1488602c707cca2e2c0b05e694", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/workflow_source.rs", "blob": "15e3513b8abe19d94a1817ea34252bcdc31efb68", "whole_file_sha256": "4b57e5afa4e66a51c7e2a828e9bb07ce1dae763a6ac58dbbaebb04c76aa9a00f", "start": 86, "end": 134, "excerpt_sha256": "e4361d599c030290aa0f610fc0a502bb2029fa1488602c707cca2e2c0b05e694"}
pub(crate) fn parse_committed_tree(bytes: &[u8]) -> Result<BTreeMap<String, CommittedTreeEntry>> {
    ensure!(
        bytes.len() <= 16 * 1024 * 1024 && (bytes.is_empty() || bytes.last() == Some(&0)),
        "committed tree bound/frame differs"
    );
    let mut entries = BTreeMap::new();
    for entry in bytes.split(|b| *b == 0).filter(|e| !e.is_empty()) {
        ensure!(
            entries.len() < 4096,
            "committed inventory exceeds 4096 files"
        );
        let text = std::str::from_utf8(entry)?;
        let (header, path) = text
            .split_once('\t')
            .context("committed tree path absent")?;
        relative(path)?;
        let fields: Vec<_> = header.split_whitespace().collect();
        ensure!(
            fields.len() == 4
                && valid_oid(fields[2])
                && matches!(
                    (fields[0], fields[1]),
                    ("100644" | "100755" | "120000", "blob") | ("160000", "commit")
                ),
            "committed tree physical type/OID differs"
        );
        let size = if fields[1] == "blob" {
            Some(fields[3].parse::<usize>()?)
        } else {
            ensure!(fields[3] == "-", "committed gitlink size differs");
            None
        };
        ensure!(
            entries
                .insert(
                    path.to_owned(),
                    CommittedTreeEntry {
                        mode: fields[0].into(),
                        kind: fields[1].into(),
                        oid: fields[2].into(),
                        size,
                    }
                )
                .is_none(),
            "duplicate committed tree path"
        );
    }
    Ok(entries)
}


## INLINE INPUT EXCERPT-corpus-catalogue
{"id": "EXCERPT-corpus-catalogue", "bytes": 1450, "sha256": "804a38e25bc39d27f5561f5284345382356b71203ec052a4f0ac0b45b54c9340", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/workflow_source.rs", "blob": "15e3513b8abe19d94a1817ea34252bcdc31efb68", "whole_file_sha256": "4b57e5afa4e66a51c7e2a828e9bb07ce1dae763a6ac58dbbaebb04c76aa9a00f", "start": 1220, "end": 1258, "excerpt_sha256": "804a38e25bc39d27f5561f5284345382356b71203ec052a4f0ac0b45b54c9340"}
async fn read_corpus(io: CorpusReader<'_>, path: &Path, revision: &str) -> Result<CommittedCorpus> {
    ensure!(
        valid_oid(revision),
        "committed inventory requires exact OID"
    );
    let tree = io
        .run(path, ["ls-tree", "-r", "-z", "-l", "--full-tree", revision])
        .await?;
    let mut files = Vec::new();
    let mut total = 0usize;
    let native_tree = parse_committed_tree(&tree)?;
    for (name, entry) in &native_tree {
        let ordinary = matches!(entry.mode.as_str(), "100644" | "100755") && entry.kind == "blob";
        let size = entry.size.unwrap_or(0);
        let (bytes, skipped) = if !ordinary {
            (None, Some(UNSUPPORTED_ENTRY.into()))
        } else if size > 256 * 1024 {
            (None, Some("file exceeds 256 KiB".into()))
        } else {
            total = total
                .checked_add(size)
                .context("committed corpus size overflow")?;
            ensure!(total <= 16 * 1024 * 1024, "committed corpus exceeds 16 MiB");
            let bytes = io.run(path, ["cat-file", "blob", &entry.oid]).await?;
            ensure!(bytes.len() == size, "committed blob size mismatch");
            (Some(bytes), None)
        };
        files.push(CommittedFile {
            path: name.clone(),
            oid: entry.oid.clone(),
            bytes,
            skipped,
        });
    }
    Ok(CommittedCorpus {
        files,
        tree: native_tree,
    })
}


## INLINE INPUT EXCERPT-helper-writer
{"id": "EXCERPT-helper-writer", "bytes": 2383, "sha256": "8d99350865249ce03e97177ef66d9191ede99b90bc299f234bf826481664e159", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/execution/effects.rs", "blob": "6fcfea86d5b0cca54142a4d564f20a11495c1993", "whole_file_sha256": "76275722e63f1b7158272d0d88c2524e5d5576d0bda33c645e2827590547febb", "start": 41, "end": 100, "excerpt_sha256": "8d99350865249ce03e97177ef66d9191ede99b90bc299f234bf826481664e159"}
    pub(crate) fn reserve_execution_helper_pinned(
        &mut self,
        authority: &ExecutionAuthority,
        id: OperationId,
        native: bool,
        path: &std::path::Path,
        kind: &str,
        driver: Option<&DriverReadTicket>,
    ) -> Result<()> {
        ensure!(path.is_absolute(), "helper path must be absolute");
        ensure!(
            matches!(kind, "git_helper" | "native_version" | "docker_probe"),
            "unsupported unit helper kind"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(ticket) = driver {
            ticket.validate_current_tx(&tx)?;
        }
        let unit = validate_authority(&tx, authority, native, !native)?;
        ensure!(
            !verification::is_command_unit(&tx, unit.id)? || kind == "git_helper",
            "command-only verifier only permits scoped snapshot Git helpers"
        );
        ensure!(
            unit.phase != WORKFLOW_SOURCE_BOOTSTRAP || kind == "git_helper",
            "source preparation only permits registered Git helpers"
        );
        if let Some(ticket) = driver {
            ticket.preparation_matches(&unit)?;
        }
        let effect = ManagedEffect {
            id,
            unit_id: authority.unit_id,
            scope: authority.scope.clone(),
            kind: kind.into(),
            idempotency_key: format!("helper-{id}"),
            expected_target: path.to_string_lossy().into(),
            state: EffectState::Pending,
            receipt: BTreeMap::new(),
            version: 1,
        };
        let (p, g, t) = scope_keys(&effect.scope)?;
        tx.execute("INSERT INTO managed_effects(id,unit_id,project_id,goal_id,task_id,idempotency_key,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,'pending',1,?7)",
            params![id.to_string(),effect.unit_id.to_string(),p,g,t,effect.idempotency_key,serde_json::to_string(&effect)?])?;
        tx.commit()?;
        Ok(())
    }
    pub(crate) fn reserve_managed_effect(
        &mut self,
        authority: &ExecutionAuthority,
        effect: &ManagedEffect,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        reserve_effect_tx(&tx, authority, effect)?;
        tx.commit()?;
        Ok(())


## INLINE INPUT EXCERPT-input-effect-cap
{"id": "EXCERPT-input-effect-cap", "bytes": 709, "sha256": "f24e6cc248638ea1d570aecf5b0abecd98cd04b9f957be5ed453be52b13a2974", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/execution/native_phase.rs", "blob": "b60f5f0fa002f113e7dcf82734652a09e533995d", "whole_file_sha256": "6186f883b541edf15d99764914f79c0fafa2692147a924ddc21fe08bf9349473", "start": 1330, "end": 1346, "excerpt_sha256": "f24e6cc248638ea1d570aecf5b0abecd98cd04b9f957be5ed453be52b13a2974"}
        let mutations = plan
            .admission
            .as_ref()
            .map(PairRow::insert_permission)
            .transpose()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let write = || -> Result<()> {
            plan.owner.validate_tx(&tx)?;
            plan.before.validate_tx(&tx)?;
            let count: usize = tx.query_row(
                "SELECT count(*) FROM (SELECT 1 FROM managed_effects WHERE unit_id=?1 LIMIT 257)",
                [plan.effect.unit_id.to_string()],
                |r| r.get(0),
            )?;
            ensure!(count < 256, "Native effect admission profile exhausted");


## INLINE INPUT EXCERPT-inventory-cap
{"id": "EXCERPT-inventory-cap", "bytes": 808, "sha256": "060216d5c483c016af123b0ae5ec1361c3fa3f28537548dadede482bb4b02a98", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/execution/native_phase/version.rs", "blob": "769c7f68b01511f12dc91b4f7b38cbd2a049c39f", "whole_file_sha256": "33114dcadedce4dacaf9f85789d66e4eb05b7bed7aa521477947e408bb3f3845", "start": 18, "end": 22, "excerpt_sha256": "060216d5c483c016af123b0ae5ec1361c3fa3f28537548dadede482bb4b02a98"}
const ROWS: usize = 256;
const BODY_BYTES: usize = 8192;
const INVENTORY_BYTES: usize = 2 * 1024 * 1024;
const SELECT: &str = "SELECT id,unit_id,project_id,goal_id,task_id,idempotency_key,state,body,version FROM managed_effects WHERE unit_id=?1 ORDER BY id LIMIT 257";
const SHAPES: &str = "SELECT typeof(id),length(CAST(id AS BLOB)),typeof(unit_id),length(CAST(unit_id AS BLOB)),typeof(project_id),length(CAST(project_id AS BLOB)),typeof(goal_id),length(CAST(goal_id AS BLOB)),typeof(task_id),length(CAST(task_id AS BLOB)),typeof(idempotency_key),length(CAST(idempotency_key AS BLOB)),typeof(state),length(CAST(state AS BLOB)),typeof(body),length(CAST(body AS BLOB)),typeof(version),version,state IN ('pending','confirmed','resolved','unknown') FROM managed_effects WHERE unit_id=?1 ORDER BY id LIMIT 257";


## INLINE INPUT EXCERPT-plan-memory
{"id": "EXCERPT-plan-memory", "bytes": 330, "sha256": "9bcd7a8f8023cacf2b239f0568466e1566c7c70702bd44a5f2b631b7069a8876", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/managed_binding/binding.rs", "blob": "310a55a2a30ed1dc09ecc7b807561af10428790a", "whole_file_sha256": "3ec2e6c63efdbea440a7f9f2473ff8af8ed7787aa81cd836ada30d83de915978", "start": 34, "end": 44, "excerpt_sha256": "9bcd7a8f8023cacf2b239f0568466e1566c7c70702bd44a5f2b631b7069a8876"}
pub(crate) struct ManagedBindingPlan {
    proof: Arc<NativePhaseBinding>,
    current: CurrentWorkflowSuccessor,
    session: Body<Record>,
    owner_raw: String,
    invocation: Body<NativeInvocation>,
    after: Body<Record>,
    audit_data: String,
    at: i64,
    record_mutation: ExactRowMutation,
    already_bound: bool,


## INLINE INPUT EXCERPT-plan-construction
{"id": "EXCERPT-plan-construction", "bytes": 445, "sha256": "2ca11577c1b490dd9f8073c3030d15a5187d8e2ca9d7aaff8677431cb45e0d7e", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/managed_binding/binding.rs", "blob": "310a55a2a30ed1dc09ecc7b807561af10428790a", "whole_file_sha256": "3ec2e6c63efdbea440a7f9f2473ff8af8ed7787aa81cd836ada30d83de915978", "start": 359, "end": 377, "excerpt_sha256": "2ca11577c1b490dd9f8073c3030d15a5187d8e2ca9d7aaff8677431cb45e0d7e"}
    let record_mutation = ExactRowMutation::new(
        "records",
        "UPDATE",
        Some(record_image(current.workflow(), current.workflow_raw())?),
        Some(record_image(after.parsed(), after.raw())?),
    )?;
    Ok(ManagedBindingPlan {
        proof,
        current,
        session,
        owner_raw,
        invocation,
        after,
        audit_data,
        at,
        record_mutation,
        already_bound,
    })
}


## INLINE INPUT EXCERPT-successor-memory
{"id": "EXCERPT-successor-memory", "bytes": 732, "sha256": "0f7813eb2de2c235000c16c0c116c5ad28d6791b2b98fddee8dd37f191c72de1", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/managed_binding/successor.rs", "blob": "b6779dd5f67acae63a4574d0f94f31d66a7dcf4d", "whole_file_sha256": "8ea5e1c24686a5c5afc56036c18c81063084d2afae3fdc5e234c38efe76efcc4", "start": 23, "end": 44, "excerpt_sha256": "0f7813eb2de2c235000c16c0c116c5ad28d6791b2b98fddee8dd37f191c72de1"}
/// Private coherent read product, not Clone/Deserialize or Native authority.
/// Keeping the original actual plan prevents SQL/current-row origin replacement.
pub(crate) struct CurrentWorkflowSuccessor {
    original: Arc<MarkerPublicationPlan>,
    workflow: Arc<Body<Record>>,
    unit: Arc<Body<ExecutionUnit>>,
    count: usize,
    head: Option<Arc<Link>>,
}
struct Link {
    event: AuditEvent,
    raw: String,
}
impl CurrentWorkflowSuccessor {
    pub(in crate::state) fn copy_original(&self) -> Self {
        Self {
            original: self.original.clone(),
            workflow: self.workflow.clone(),
            unit: self.unit.clone(),
            count: self.count,
            head: self.head.clone(),
        }


## INLINE INPUT EXCERPT-body-memory
{"id": "EXCERPT-body-memory", "bytes": 995, "sha256": "dad70beb7bbf33b981d335b15152bfe092801ae51a3ad34aeeda142288bc61a6", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/managed_binding/canonical.rs", "blob": "31e2aaa302ec65208b0b3679281a4a9fbfb9d944", "whole_file_sha256": "6f77f8b0a0218d47bd5ee6df77c147c518d6c84573ecd42a19bac9a6886e32a2", "start": 18, "end": 48, "excerpt_sha256": "dad70beb7bbf33b981d335b15152bfe092801ae51a3ad34aeeda142288bc61a6"}
pub(in crate::state) struct Body<T> {
    raw: String,
    parsed: T,
    canonical: Vec<u8>,
}
impl<T: DeserializeOwned + Serialize> Body<T> {
    pub(in crate::state) fn decode(raw: String, limit: usize) -> Result<Self> {
        let value = decode_value(&raw, limit)?;
        let parsed: T = serde_json::from_value(value.clone())
            .map_err(|_| anyhow::anyhow!("managed body does not match its typed schema"))?;
        let canonical = encode(&value, limit)?;
        ensure!(
            encode(&serde_json::to_value(&parsed)?, limit)? == canonical,
            "managed body typed roundtrip drops or changes fields"
        );
        Ok(Self {
            raw,
            parsed,
            canonical,
        })
    }
    pub(in crate::state) fn raw(&self) -> &str {
        &self.raw
    }
    pub(in crate::state) fn parsed(&self) -> &T {
        &self.parsed
    }
    pub(super) fn digest(&self, domain: &[u8]) -> String {
        digest(domain, &self.canonical)
    }
}


## INLINE INPUT EXCERPT-mutation-memory
{"id": "EXCERPT-mutation-memory", "bytes": 1873, "sha256": "b4ce05e1c6b302ab06dff8758cc4ebfac6b8684ef2d480721154598910e3fc4a", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/managed_binding/permits.rs", "blob": "635bef05a90bf7b509351a7bfd8a65ae901885fc", "whole_file_sha256": "24b5c2fffc50c40a102f4cb37bedb88113b40c6c690cfb6eed8ff83c0453aace", "start": 182, "end": 230, "excerpt_sha256": "b4ce05e1c6b302ab06dff8758cc4ebfac6b8684ef2d480721154598910e3fc4a"}
/// Not serializable; only managed-binding Store code can build a full row plan.
/// Value::Real is rejected, rather than conflating SQLite numeric encodings.
#[allow(dead_code)] // Actual marker/binder producer is composed separately.
pub(in crate::state) struct ExactRowMutation {
    table: &'static str,
    action: &'static str,
    old: Vec<Value>,
    new: Vec<Value>,
}
#[allow(dead_code)]
impl ExactRowMutation {
    /// Copy only this exact already validated SQL image for retained-plan retry.
    /// This never issues a Native proof and is private to managed-binding code.
    pub(in crate::state) fn copy_for_transaction(&self) -> Result<Self> {
        Self::new(
            self.table,
            self.action,
            (self.action != "INSERT").then(|| self.old.clone()),
            (self.action != "DELETE").then(|| self.new.clone()),
        )
    }
    pub(in crate::state) fn new(
        table: &'static str,
        action: &'static str,
        old: Option<Vec<Value>>,
        new: Option<Vec<Value>>,
    ) -> Result<Self> {
        let n = columns(table)
            .ok_or_else(|| anyhow::anyhow!("unknown private permit table"))?
            .len();
        ensure!(
            matches!(
                (action, old.is_some(), new.is_some()),
                ("INSERT", false, true) | ("UPDATE", true, true) | ("DELETE", true, false)
            ),
            "invalid exact mutation action"
        );
        for image in [&old, &new].into_iter().flatten() {
            ensure!(
                image.len() == n && image.iter().all(|v| !matches!(v, Value::Real(_))),
                "incomplete exact mutation image"
            );
        }
        Ok(Self {
            table,
            action,
            old: old.unwrap_or_else(|| vec![Value::Null; n]),
            new: new.unwrap_or_else(|| vec![Value::Null; n]),
        })


## INLINE INPUT EXCERPT-waiting-reader
{"id": "EXCERPT-waiting-reader", "bytes": 5366, "sha256": "3b6209725296b6a6b956084f389c24c0378c949d0e6f5dc37019f0071f2ebc0d", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/runtime/waiting.rs", "blob": "6bfce9f521cd909b40fd42e7208e411fb5ff6a91", "whole_file_sha256": "68b495c983534332b8de38f1ad0570161c861dc03aee91508f2da919e3470818", "start": 31, "end": 159, "excerpt_sha256": "3b6209725296b6a6b956084f389c24c0378c949d0e6f5dc37019f0071f2ebc0d"}
fn observe_bounded(
    connection: &Connection,
    task: &Task,
) -> Result<Option<PhaseWaitingObservation>> {
    let mut query = connection.prepare_cached(
        "SELECT CASE WHEN typeof(u.body)='text' AND length(CAST(u.body AS BLOB))<=16384 THEN u.body END,
         CASE WHEN typeof(r.body)='text' AND length(CAST(r.body AS BLOB))<=4096 THEN r.body END,
         r.state,r.version,r.parking_version,
         CASE WHEN length(w.reason)<=16 THEN w.reason END,w.next_due,
         COALESCE(u.project_id=o.project_id AND u.goal_id=o.goal_id AND u.task_id=o.task_id
          AND u.generation=o.execution_generation AND u.owner_epoch=o.owner_epoch
          AND u.owner_epoch=e.epoch AND u.native_effects_open=1
          AND o.marker_task_version=?4
          AND ow.operation_id=o.operation_id AND ow.unit_id=o.unit_id
          AND ow.project_id=o.project_id AND ow.goal_id=o.goal_id AND ow.task_id=o.task_id
          AND ow.owner_epoch=o.owner_epoch AND ow.execution_generation=o.execution_generation
          AND ow.origin=o.origin AND ow.provider=o.provider
          AND d.owner_epoch=o.owner_epoch AND d.state='driving'
          AND r.owner_epoch=o.owner_epoch AND r.origin=o.origin
          AND r.start_ended=0 AND r.known_terminal=0,0),
         COALESCE(w.provider=o.provider AND w.account_key='unknown' AND w.resume_state='preparing',0),
         o.unit_id,o.provider,o.phase,o.execution_generation,o.owner_epoch,u.version,o.operation_id,o.origin
         FROM managed_phase_operations o
         LEFT JOIN execution_units u ON u.id=o.unit_id
         LEFT JOIN managed_phase_owners ow ON ow.owner_id=o.owner_id
         LEFT JOIN managed_phase_readiness r ON r.operation_id=o.operation_id
         LEFT JOIN quota_waiters w ON w.unit_id=o.unit_id
         LEFT JOIN task_drivers d ON d.task_id=o.task_id AND d.goal_id=o.goal_id AND d.project_id=o.project_id
         LEFT JOIN runtime_epoch e ON e.singleton=1
         WHERE o.project_id=?1 AND o.goal_id=?2 AND o.task_id=?3 AND o.phase_open=1 LIMIT 2"
    )?;
    let mut rows = query.query(params![
        task.project_id.to_string(),
        task.goal_id.to_string(),
        task.id.to_string(),
        task.version
    ])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    let observation = (|| -> Result<Option<PhaseWaitingObservation>> {
        if !row.get::<_, bool>(7)? {
            return Ok(Some(HELD));
        }
        let state: String = row.get(2)?;
        if matches!(state.as_str(), "held" | "deferred") {
            return Ok(Some(HELD));
        }
        if state != "parked" {
            return Ok(None);
        }
        let raw: String = row.get(0)?;
        let unit: ExecutionUnit = serde_json::from_str(&raw)?;
        let ready: Value = serde_json::from_str(&row.get::<_, String>(1)?)?;
        let version: u64 = row.get(3)?;
        let parking: Option<u64> = row.get(4)?;
        let reason: String = row.get(5)?;
        let due: i64 = row.get(6)?;
        let expected = match reason.as_str() {
            "quota" => WaitReason::Quota,
            "capacity" => WaitReason::Capacity,
            _ => return Ok(Some(HELD)),
        };
        let matches = row.get::<_, bool>(8)?
            && parking == Some(version)
            && ready["state"] == "parked"
            && ready["version"] == version
            && ready["parking_version"] == version
            && ready["start_ended"] == false
            && ready["known_terminal"] == false
            && unit.id.to_string() == row.get::<_, String>(9)?
            && unit.scope == task.scope()
            && unit.state == UnitState::Preparing
            && unit.native_effects_open
            && unit.wait_reason == Some(expected)
            && unit.session_id.is_none()
            && unit.provider == row.get::<_, String>(10)?
            && unit.phase == row.get::<_, String>(11)?
            && unit.generation == row.get::<_, u64>(12)?
            && unit.owner_epoch == row.get::<_, u64>(13)?
            && unit.version == row.get::<_, u64>(14)?
            && ready["operation_id"] == row.get::<_, String>(15)?
            && ready["origin"] == row.get::<_, String>(16)?
            && ready["owner_epoch"] == unit.owner_epoch;
        Ok(Some(if matches {
            PhaseWaitingObservation::Waiting {
                reason: expected,
                next_due: due,
            }
        } else {
            HELD
        }))
    })()
    .unwrap_or(Some(HELD));
    Ok(if rows.next()?.is_some() {
        Some(HELD)
    } else {
        observation
    })
}

pub(super) fn effective_state(
    task: &Task,
    observation: Option<&PhaseWaitingObservation>,
) -> TaskState {
    match observation {
        Some(PhaseWaitingObservation::Waiting {
            reason: WaitReason::Quota,
            ..
        }) => TaskState::WaitingQuota,
        Some(PhaseWaitingObservation::Waiting {
            reason: WaitReason::Capacity,
            ..
        }) => TaskState::WaitingCapacity,
        _ => task.state,
    }
}

impl Store {
    pub(crate) fn phase_waiting_observation(
        &self,
        task: &Task,
        goal: &Goal,
    ) -> Result<Option<PhaseWaitingObservation>> {
        let tx = self.connection.unchecked_transaction()?;
        let observation = observe(&tx, task, goal);
        tx.commit()?;
        Ok(observation)
    }


## INLINE INPUT EXCERPT-goal-status
{"id": "EXCERPT-goal-status", "bytes": 1230, "sha256": "43d5c628fdddbcbcf090174d125230575a96f973862bb474d943921c60d80e91", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/runtime/goals.rs", "blob": "a1a84ad03f6735d720384a08c5ffc082cf72e634", "whole_file_sha256": "d5622623d1689545c67365ae41dd78055240161bc5306c96468d160843142529", "start": 312, "end": 335, "excerpt_sha256": "43d5c628fdddbcbcf090174d125230575a96f973862bb474d943921c60d80e91"}
            return Ok(response);
        }
        let goal = current_goal(&tx, project, id)?;
        let tasks = scoped_tasks(&tx, &goal)?;
        let mut states = BTreeMap::new();
        for task in &tasks {
            let observation = super::waiting::observe(&tx, task, &goal);
            let state = super::waiting::effective_state(task, observation.as_ref());
            *states
                .entry(
                    serde_json::to_value(state)?
                        .as_str()
                        .context("Task state encoding")?
                        .to_owned(),
                )
                .or_insert(0) += 1;
        }
        let initial_driver:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM scheduler_tasks s JOIN task_drivers d ON d.task_id=s.task_id AND d.goal_id=s.goal_id AND d.project_id=s.project_id WHERE s.goal_id=?1 AND s.project_id=?2 AND s.attention IS NULL AND d.owner_epoch=?3 AND d.state='driving')",params![id.to_string(),project.to_string(),ingress.identity().1],|r|r.get(0))?;
        let response = ControlResponse::GoalFacts {
            goal: id,
            version: goal.version,
            state: goal.state,
            task_count: tasks.len(),
            states,


## INLINE INPUT EXCERPT-goal-page
{"id": "EXCERPT-goal-page", "bytes": 1029, "sha256": "8d6099449be6244998d9e09d932f596d2bdcde47ae246516baa91a47d6dc3149", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/runtime/goals.rs", "blob": "a1a84ad03f6735d720384a08c5ffc082cf72e634", "whole_file_sha256": "d5622623d1689545c67365ae41dd78055240161bc5306c96468d160843142529", "start": 529, "end": 555, "excerpt_sha256": "8d6099449be6244998d9e09d932f596d2bdcde47ae246516baa91a47d6dc3149"}
        let eligible = all
            .iter()
            .filter(|t| after.is_none_or(|a| t.id > a))
            .collect::<Vec<_>>();
        let mut facts = Vec::new();
        let mut next = None;
        for task in eligible.iter().take(maximum) {
            ensure!(
                task.phase.as_ref().is_none_or(|p| p.len() <= 128),
                "Task phase exceeds status bound"
            );
            facts.push(TaskFacts {
                scope: task.scope(),
                version: task.version,
                state: super::waiting::effective_state(
                    task,
                    super::waiting::observe(&tx, task, &goal).as_ref(),
                ),
                phase: task.phase.clone(),
            });
            let more = eligible.len() > facts.len();
            let candidate = ControlResponse::GoalTaskPage {
                goal: id,
                version: goal.version,
                tasks: facts.clone(),
                next: more.then_some(task.id),
            };


## INLINE INPUT EXCERPT-control-read
{"id": "EXCERPT-control-read", "bytes": 726, "sha256": "8608b5e1c407420cc613ca19795056fb0ca149c93836f8dea3e2ab49248e5e74", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/runtime/control.rs", "blob": "e7ddbd3ea59d45a8da2dfb33905ae1c9448b000f", "whole_file_sha256": "03ffcab929390bb5d603bcc668215a386580f629662c9344ffc2f8907b749d2f", "start": 342, "end": 363, "excerpt_sha256": "8608b5e1c407420cc613ca19795056fb0ca149c93836f8dea3e2ab49248e5e74"}
        if let ControlAction::GoalStatus { project, goal } = &request.action {
            return self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .runtime_goal_facts(&ingress, *project, *goal);
        }
        if let ControlAction::GoalTasks {
            project,
            goal,
            after,
            maximum,
        } = &request.action
        {
            return self
                .owner
                .store
                .lock()
                .map_err(|_| anyhow::anyhow!("state poisoned"))?
                .runtime_goal_task_page(&ingress, *project, *goal, *after, *maximum);
        }


## INLINE INPUT EXCERPT-cli-read
{"id": "EXCERPT-cli-read", "bytes": 1666, "sha256": "976bbaa0d47aed6ddea8a1a7282c46f89c5dd3891556fcf436cadb60449811a4", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/main.rs", "blob": "e40bb9c3d690b09c98c42d501af9bb778ed1530b", "whole_file_sha256": "f88469d7e5dd97c2ebeaa54813d601e892f1ebfd25ee64550b71a1dcd7b606f9", "start": 185, "end": 224, "excerpt_sha256": "976bbaa0d47aed6ddea8a1a7282c46f89c5dd3891556fcf436cadb60449811a4"}
            read => {
                let (goal, selector, json, page) = match read {
                    GoalCommand::Status {
                        goal,
                        project,
                        json,
                    } => (goal, project, json, None),
                    GoalCommand::Tasks {
                        goal,
                        project,
                        after,
                        maximum,
                        json,
                    } => (goal, project, json, Some((after, maximum))),
                    _ => unreachable!(),
                };
                let (project, _) = resolve_project(state, selector).await?;
                let action = match page {
                    Some((after, maximum)) => ControlAction::GoalTasks {
                        project,
                        goal,
                        after,
                        maximum: usize::from(maximum),
                    },
                    None => ControlAction::GoalStatus { project, goal },
                };
                let response = client::request(state, action).await?;
                ensure!(
                    matches!(
                        (&response, page),
                        (
                            ControlResponse::GoalFacts { .. }
                                | ControlResponse::GoalProposalFacts { .. },
                            None
                        ) | (ControlResponse::GoalTaskPage { .. }, Some(_))
                    ),
                    "unexpected Goal response"
                );
                return print_control(response, json, false);
            }


## INLINE INPUT EXCERPT-cli-format
{"id": "EXCERPT-cli-format", "bytes": 1018, "sha256": "28f77c0a02c5dcbf843322df5b13e5ccd9763d145899e4407cea462da9c39e9d", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/main.rs", "blob": "e40bb9c3d690b09c98c42d501af9bb778ed1530b", "whole_file_sha256": "f88469d7e5dd97c2ebeaa54813d601e892f1ebfd25ee64550b71a1dcd7b606f9", "start": 306, "end": 339, "excerpt_sha256": "28f77c0a02c5dcbf843322df5b13e5ccd9763d145899e4407cea462da9c39e9d"}
fn print_control(response: ControlResponse, json: bool, metadata: bool) -> Result<()> {
    let observation = if metadata {
        "runtime_metadata"
    } else {
        "independent_scoped_observation"
    };
    let unavailable = if metadata {
        vec![
            "project_goal_task_hierarchy",
            "effective_native_limits",
            "unit_provider_evidence",
            "wait_cleanup_details",
        ]
    } else {
        vec![
            "native_dispatch",
            "unit_provider_evidence",
            "wait_cleanup_details",
        ]
    };
    let output = serde_json::json!({
        "observation": observation, "complete": false,
        "unavailable_fields": unavailable, "facts": response,
    });
    if json {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!(
            "{}\nIncomplete observation: {}",
            serde_json::to_string_pretty(&output["facts"])?,
            unavailable.join(", ")
        );
    }
    Ok(())


## INLINE INPUT EXCERPT-context-input
{"id": "EXCERPT-context-input", "bytes": 1110, "sha256": "1707241753f4503e7138f18bc830c9b2a22374e977a8e9908430ca1fce9ac75f", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/workflow_source/native_handoff.rs", "blob": "8acfa9a0aa5eb1415e86ec0e27ca577d1bb920cd", "whole_file_sha256": "6fb8610c82d5844109ce61ddabaf961da8efb54b570c34c15cc0829f12bc155a", "start": 538, "end": 563, "excerpt_sha256": "1707241753f4503e7138f18bc830c9b2a22374e977a8e9908430ca1fce9ac75f"}
        // This temporary frame checker is not stored in the Source custody.
        let proof = InitialInputFrame {
            producer: self.clone(),
            slot: slot.clone(),
            frame: state.frame.clone(),
            unit: unit.clone(),
        };
        proof.validate_context(task, record, context)?;
        let config = state.frame.config.agents.get(&task.executor);
        let allocation = selected.allocate(
            ManagedInput {
                agent: task.executor.clone(),
                authority: unit.authority(),
                artifact: None,
                input: PreparedInput {
                    scope: context.scope.clone(),
                    kind: InputKind::ContextPack,
                    revision: context.revision.clone(),
                    version: context.version,
                    source_versions: context.source_hashes.clone(),
                    payload: serde_json::to_string(&context.data)?,
                },
            },
            config.and_then(|c| c.model.clone()),
            config.and_then(|c| c.effort.clone()),
        )?;


## INLINE INPUT EXCERPT-input-limit
{"id": "EXCERPT-input-limit", "bytes": 2329, "sha256": "e0862d9964e0fee32c63c6c4412dbb147316535a8116348648d25bfe0b6472df", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/phase.rs", "blob": "dac28d202bf05b4b257086844fbd944746b3480b", "whole_file_sha256": "ac02ccb3dc583f4e0b99ac23aad0cdc1adbc45314a805100bcf51d0a114991fe", "start": 199, "end": 260, "excerpt_sha256": "e0862d9964e0fee32c63c6c4412dbb147316535a8116348648d25bfe0b6472df"}
/// Encode all input fields into a finite original template. A bounded writer
/// enforces the complete escaped encoding cost, including keys and overhead.
pub(crate) fn encode_input(input: &PreparedInput) -> Result<Vec<u8>> {
    ensure!(
        input.scope.goal_id.is_some()
            && input.scope.task_id.is_some()
            && input.version > 0
            && super::valid_oid(&input.revision)
            && !input.payload.is_empty()
            && input.payload.len() <= 1024 * 1024
            && input.source_versions.len() <= 4096
            && input.source_versions.iter().all(|(key, value)| {
                !key.is_empty()
                    && key.len() <= 4096
                    && !key.chars().any(char::is_control)
                    && !value.is_empty()
                    && value.len() <= 128
                    && !value.chars().any(char::is_control)
            }),
        "native allocation input bounds unavailable"
    );
    #[derive(Serialize)]
    struct Frame<'a> {
        scope: &'a Scope,
        kind: &'static str,
        revision: &'a str,
        version: u64,
        source_versions: &'a std::collections::BTreeMap<String, String>,
        payload: &'a str,
    }
    struct Limited(Vec<u8>);
    impl Write for Limited {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > INPUT_BYTES.saturating_sub(self.0.len()) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "native input encoding exceeds bound",
                ));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut encoded = Limited(Vec::new());
    serde_json::to_writer(
        &mut encoded,
        &Frame {
            scope: &input.scope,
            kind: match input.kind {
                crate::adapter::InputKind::ContextPack => "context_pack",
                crate::adapter::InputKind::ReviewBundle => "review_bundle",
            },
            revision: &input.revision,
            version: input.version,
            source_versions: &input.source_versions,
            payload: &input.payload,
        },
    )?;
    Ok(encoded.0)


## INLINE INPUT EXCERPT-frame-limit
{"id": "EXCERPT-frame-limit", "bytes": 171, "sha256": "2341b9135c4de339591dcabb8ea395294ed2cb3c4767b385b308dfc4a2477d05", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/phase.rs", "blob": "dac28d202bf05b4b257086844fbd944746b3480b", "whole_file_sha256": "ac02ccb3dc583f4e0b99ac23aad0cdc1adbc45314a805100bcf51d0a114991fe", "start": 114, "end": 120, "excerpt_sha256": "2341b9135c4de339591dcabb8ea395294ed2cb3c4767b385b308dfc4a2477d05"}
    }
}

const UNIT_BYTES: usize = 16 * 1024;
const INPUT_BYTES: usize = 2 * 1024 * 1024;

/// Provisional current-unit snapshot, on a separate read-only connection. Full


## INLINE INPUT EXCERPT-context-shape
{"id": "EXCERPT-context-shape", "bytes": 266, "sha256": "e357bf680bdbf29e81a8bfbdda71d5f44b122989085df6a8f4258d5a623370d2", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/domain.rs", "blob": "a320327fff917543dca49101f6c86616884540db", "whole_file_sha256": "7903da7d3c61c607cf14d01f849983584e41b1f22e8b2cce69a3aa2218615df3", "start": 428, "end": 436, "excerpt_sha256": "e357bf680bdbf29e81a8bfbdda71d5f44b122989085df6a8f4258d5a623370d2"}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextVersion {
    pub scope: Scope,
    pub version: u64,
    pub revision: String,
    pub source_hashes: std::collections::BTreeMap<String, String>,
    pub data: Value,
}


## INLINE INPUT EXCERPT-scope-charge
{"id": "EXCERPT-scope-charge", "bytes": 3860, "sha256": "78a8609c699f5c9c4ad78118fe562974d79330aa796d621867c9ed317d5836d8", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/managed_binding/publication.rs", "blob": "4f71978344a6aabed5b1372c2c5a59f3f9a3e68e", "whole_file_sha256": "2a9317663dc0ed7f986d21275b0dfb3e4501f8de6e9f3b5eb157eff8d7999bb6", "start": 301, "end": 375, "excerpt_sha256": "78a8609c699f5c9c4ad78118fe562974d79330aa796d621867c9ed317d5836d8"}
/// Complete scoped encoded body costs and open audit reservations. Aggregate
/// queries copy no bodies; every surface has an explicit row/byte refusal bound.
/// This is accounting, not a Native/Driver permission or a latency guarantee.
pub(super) fn charged_scope_bytes(tx: &Transaction<'_>, scope: &Scope) -> Result<u64> {
    let p = scope.project_id.to_string();
    let g = scope.goal_id.context("budget Goal missing")?.to_string();
    let t = scope.task_id.context("budget Task missing")?.to_string();
    let mut used = 0u64;
    // Compiled table names only, all values parameterized. This includes the
    // entire Task history rather than hiding older operation/Session costs.
    for table in [
        "records",
        "context_versions",
        "execution_units",
        "managed_phase_operations",
        "managed_phase_owners",
        "managed_phase_inputs",
        "workflow_native_contracts",
        "task_drivers",
        "source_recoveries",
        "native_invocations",
        "native_results",
        "scoped_session_identities",
        "result_artifacts",
        "resource_leases",
        "managed_effects",
        "verification_runs",
    ] {
        let (count, bytes): (u64, u64) = tx.query_row(
            &format!("SELECT count(*),COALESCE(sum(bytes),0) FROM (SELECT length(CAST(body AS BLOB)) bytes FROM {table} WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 LIMIT 4097)"),
            params![p, g, t], |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        ensure!(
            count <= 4096 && bytes <= WORKFLOW_BYTES,
            "complete marker budget surface exceeds bound"
        );
        used = used.checked_add(bytes).context("marker budget overflow")?;
    }
    for table in [
        "managed_marker_bodies",
        "managed_phase_admissions",
        "managed_phase_readiness",
    ] {
        let (count, bytes): (u64, u64) = tx.query_row(
            &format!("SELECT count(*),COALESCE(sum(bytes),0) FROM (SELECT length(CAST(s.body AS BLOB)) bytes FROM {table} s JOIN managed_phase_operations o ON o.operation_id=s.operation_id WHERE o.project_id=?1 AND o.goal_id=?2 AND o.task_id=?3 LIMIT 4097)"),
            params![p, g, t], |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        ensure!(
            count <= 4096 && bytes <= WORKFLOW_BYTES,
            "complete marker budget relation exceeds bound"
        );
        used = used.checked_add(bytes).context("marker budget overflow")?;
    }
    let (count, audit_bytes): (u64, u64) = tx.query_row(
        "SELECT count(*),COALESCE(sum(bytes),0) FROM (SELECT length(CAST(data AS BLOB)) bytes FROM audit WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind GLOB 'rrx.private.workflow.*' LIMIT 65537)",
        params![p, g, t], |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    ensure!(
        count <= 65536 && audit_bytes <= WORKFLOW_BYTES,
        "complete marker audit budget exceeds bound"
    );
    used = used
        .checked_add(audit_bytes)
        .context("marker budget overflow")?;
    let (open, reserved): (u64, u64) = tx.query_row(
        "WITH spent AS (SELECT json_extract(data,'$.private_operation_ref') operation_id,sum(length(CAST(data AS BLOB))) bytes FROM audit WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind GLOB 'rrx.private.workflow.*' GROUP BY json_extract(data,'$.private_operation_ref')) SELECT count(*),COALESCE(sum(max(0,1048576-COALESCE(s.bytes,0))),0) FROM managed_phase_operations o LEFT JOIN spent s ON s.operation_id=o.operation_id WHERE o.project_id=?1 AND o.goal_id=?2 AND o.task_id=?3 AND o.phase_open=1",
        params![p, g, t], |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    ensure!(
        open <= 128 && reserved <= WORKFLOW_BYTES,
        "open marker audit reservation exceeds bound"
    );
    used.checked_add(reserved)
        .context("complete scope budget overflow")
}


## INLINE INPUT EXCERPT-scope-bound
{"id": "EXCERPT-scope-bound", "bytes": 285, "sha256": "978ea472e1016ee4ba63f2ad538e8e06944af90c6bb806b4dd358297353ad172", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/managed_binding/marker_rows.rs", "blob": "22aa02fa8c6413f88dd7464a242bffbaeed06799", "whole_file_sha256": "caedca52a25d245fc9b10473bfe9d79a536a616cb4fd47fa4e5a4ea4a8f5a5b2", "start": 10, "end": 16, "excerpt_sha256": "978ea472e1016ee4ba63f2ad538e8e06944af90c6bb806b4dd358297353ad172"}
use rusqlite::{Transaction, params_from_iter, types::Value as SqlValue};
use serde_json::json;

pub(super) const LINK_RESERVE_BYTES: u64 = 256 * 4096;
pub(super) const WORKFLOW_BYTES: u64 = 128 * 1024 * 1024;

/// Only this module can construct a row. Table/column names are compiled;


## INLINE INPUT EXCERPT-cleanup-write
{"id": "EXCERPT-cleanup-write", "bytes": 1184, "sha256": "ee79481cbbdac6ad1e5c14fcbe751d18c43a90dcea13fd366c9fc78e24dd0ed9", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/execution.rs", "blob": "ba58dbbe685924efaf87bea0885eb6b52a0866fe", "whole_file_sha256": "f30614eacadc146368e5c0545060c0691a77b685271a4dc87f8476f502719d0a", "start": 1129, "end": 1160, "excerpt_sha256": "ee79481cbbdac6ad1e5c14fcbe751d18c43a90dcea13fd366c9fc78e24dd0ed9"}
    pub(crate) fn record_execution_cleanup(
        &mut self,
        observation: &CleanupObservation,
    ) -> Result<()> {
        ensure!(
            observation.coverage.len() <= 32
                && observation.remaining.len() <= 1024
                && observation.errors.len() <= 32
                && cleanup::valid_cleanup_actions(&observation.actions),
            "cleanup observation bound exceeded"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut unit = unit_tx(&tx, observation.unit_id)?;
        unit.cleanup = observation.outcome;
        tx.execute(
            "INSERT INTO cleanup_observations(unit_id,at,body) VALUES(?1,?2,?3)",
            params![
                unit.id.to_string(),
                observation.at,
                serde_json::to_string(observation)?
            ],
        )?;
        append_event(
            &tx,
            &unit.scope,
            "execution.cleanup_observed",
            json!({"unit":unit.id,"work":unit.work,"cleanup":unit.cleanup,"coverage":observation.coverage}),
        )?;
        tx.commit()?;
        Ok(())


## INLINE INPUT EXCERPT-cleanup-overlay
{"id": "EXCERPT-cleanup-overlay", "bytes": 725, "sha256": "9dc1550eb90067363b268c4f64c042d883844db3f9cf453917b0e52c2e92e0ad", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/state/execution.rs", "blob": "ba58dbbe685924efaf87bea0885eb6b52a0866fe", "whole_file_sha256": "f30614eacadc146368e5c0545060c0691a77b685271a4dc87f8476f502719d0a", "start": 249, "end": 267, "excerpt_sha256": "9dc1550eb90067363b268c4f64c042d883844db3f9cf453917b0e52c2e92e0ad"}
    // Cleanup is an independent factual projection. Appending a janitor receipt
    // cannot supersede native/finalization authority or invalidate a capture CAS.
    if let Some((at, body)) = tx
        .query_row(
            "SELECT at,body FROM cleanup_observations WHERE unit_id=?1 ORDER BY rowid DESC LIMIT 1",
            [id.to_string()],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?
    {
        let observation: CleanupObservation = decode(body)?;
        ensure!(
            observation.unit_id == id && observation.at == at,
            "cleanup observation indexed/body mismatch"
        );
        unit.cleanup = observation.outcome;
    }
    Ok(unit)
}


## INLINE INPUT EXCERPT-fixture-setup
{"id": "EXCERPT-fixture-setup", "bytes": 2962, "sha256": "c6de09829908164d5e8e4e30649bdcf6fcd2deaed8c6c7c9b6662a8412203fa5", "label": "CA", "commit": "95ef0e783c0b18c90dfef23ba1b77b9dc67735f1", "source_path": "crates/rrx/src/runtime/installation/tests.rs", "blob": "0e3b5973f81720e8b073feb252093e493d3b3f8d", "whole_file_sha256": "0ee62dffb2e55aaf3f562029be40aea5e0982ab58287a0c97fe990304adca369", "start": 17, "end": 91, "excerpt_sha256": "c6de09829908164d5e8e4e30649bdcf6fcd2deaed8c6c7c9b6662a8412203fa5"}
fn fixture(provider: &str, declared: bool) -> ControlFixture {
    fixture_with(provider, declared, |_| {})
}
fn fixture_with(
    provider: &str,
    declared: bool,
    configure: impl FnOnce(&mut crate::config::Config),
) -> ControlFixture {
    ControlFixture::configured(|dir| {
        let path = dir.join("configured-protocol-fixture");
        let source = include_str!("../../execution/native/native_fixture.py");
        // Hold only the external protocol peer's bootstrap, after a real spawn.
        // This is not an rrx admission/authority or permission switch.
        let source=source.replacen("while True: time.sleep(0.02)","while not os.path.exists(os.path.join(os.environ[\"RRX_OUTPUT_DIR\"], \"fixture-bootstrap-release\")): time.sleep(0.02)",1);
        std::fs::write(&path,format!("#!/usr/bin/python3\nPROVIDER={provider:?}\nBOOTSTRAP_HOLD=True\nWORKFLOW_SCENARIO='answer-normal'\n{source}")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut config = crate::config::Config {
            minimum_workflow: crate::config::WorkflowClass::Quick,
            ..Default::default()
        };
        config.workflow.risk_mapping = [crate::config::WorkflowClass::Quick; 4];
        config.agents.insert(
            "worker".into(),
            AgentConfig {
                provider: Some(provider.into()),
                command: vec![path.to_string_lossy().into()],
                compatibility: declared.then(|| NativeCompatConfig {
                    profile: "rrx-native-inherited-v1".into(),
                    cli_version: if provider == "claude" {
                        "2.1.283"
                    } else {
                        "codex-cli 0.160.0"
                    }
                    .into(),
                    settings: "inherited".into(),
                    user_hooks: vec![],
                }),
                ..Default::default()
            },
        );
        for alias in ["rev-a", "rev-b"] {
            config
                .agents
                .insert(alias.into(), config.agents["worker"].clone());
        }
        configure(&mut config);
        config
    })
}
async fn accept(f: &ControlFixture, count: usize) -> (GoalId, Vec<Task>) {
    let mut p = plan();
    p.tasks[0].risk = RiskClass::R1;
    p.tasks[0].reviewers = vec!["rev-a".into(), "rev-b".into()];
    let original = p.tasks[0].clone();
    p.tasks = (0..count)
        .map(|i| {
            let mut task = original.clone();
            task.key = format!("task-{i}");
            task.title = format!("work-{i}");
            task
        })
        .collect();
    let goal = f.create(p).await;
    let store = f.owner.store.lock().unwrap();
    let tasks = store
        .goal(goal)
        .unwrap()
        .unwrap()
        .dag
        .nodes
        .iter()
        .map(|id| store.task(*id).unwrap().unwrap())
        .collect();
    (goal, tasks)
}


## INLINE INPUT EXCERPT-fixture-bound
{"id": "EXCERPT-fixture-bound", "bytes": 3441, "sha256": "5063773132fd837fb2800179cb384703b026f603b842a1aa455a9df63a48a312", "label": "CA", "commit": "95ef0e783c0b18c90dfef23ba1b77b9dc67735f1", "source_path": "crates/rrx/src/runtime/installation/tests/activation.rs", "blob": "d1c8d2c81842d857db789ea7cbfe4d9d13f89721", "whole_file_sha256": "2cbc0ac2d83b68e2df5681001adf64b512620c36a754b89d4eac46deac757695", "start": 119, "end": 215, "excerpt_sha256": "5063773132fd837fb2800179cb384703b026f603b842a1aa455a9df63a48a312"}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ca1_actual_activation_gates_marker_and_record_only_bound() {
    for provider in ["claude", "codex"] {
        let mut f = fixture(provider, true);
        f.register_real_git_project();
        let (_, tasks) = accept(&f, 1).await;
        let expected = Arc::new(Mutex::new(None));
        let arrived = Arc::new(AtomicBool::new(false));
        let capture = expected.clone();
        let signal = arrived.clone();
        f.runtime
            .installed
            .as_ref()
            .ok()
            .unwrap()
            .engine
            .set_activation_hooks(
                Some(Arc::new(move |probe| {
                    *capture.lock().unwrap() = Some(expected_contract(&probe));
                    signal.store(true, Ordering::SeqCst);
                    Box::pin(async {})
                })),
                None,
            );
        f.runtime.start().await.unwrap();
        wait_for(
            || arrived.load(Ordering::SeqCst),
            "SETUP: genuine retained activation input not reached",
        )
        .await;
        wait_for(
            || count(&f, "workflow_native_contracts") == 1,
            "CA1 activation stage: composed contract absent",
        )
        .await;
        let expected = expected.lock().unwrap().clone().unwrap();
        let contract: String = raw(&f)
            .query_row("SELECT body FROM workflow_native_contracts", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            contract,
            String::from_utf8(canonical(&expected)).unwrap(),
            "CA1 members and exact activation image"
        );
        let (record, workflow) = wait_bound(&f, &tasks[0]).await;
        let stored = f
            .owner
            .store
            .lock()
            .unwrap()
            .task(tasks[0].id)
            .unwrap()
            .unwrap();
        let (task_version, workflow_version): (u64, u64) = raw(&f)
            .query_row(
                "SELECT marker_task_version,marker_workflow_version FROM managed_phase_operations",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(
            stored.version, task_version,
            "CA1 record-only binder changed Task"
        );
        assert_eq!(
            record.version,
            workflow_version + 1,
            "CA1 binder record version"
        );
        assert_eq!(
            workflow_version, 9,
            "CA1 initial + six gate edges + Executor reservation + marker"
        );
        let advances:u64=raw(&f).query_row("SELECT count(*) FROM audit WHERE kind='rrx.private.runtime.driver_initial_input_advanced'",[],|r|r.get(0)).unwrap();
        assert_eq!(
            advances, 8,
            "CA1 activation, Reserve/Claim/Complete and Executor record windows"
        );
        assert_eq!(
            count(&f, "native_invocations"),
            1,
            "CA1 actual registered launch"
        );
        let after: String = raw(&f)
            .query_row("SELECT body FROM workflow_native_contracts", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            after, contract,
            "CA1 immutable contract changed at marker/bind"
        );
        release_peer(&f, &workflow);
        finish(f).await;
    }


## INLINE INPUT EXCERPT-fixture-commit
{"id": "EXCERPT-fixture-commit", "bytes": 1369, "sha256": "91cbfd8f141d9c9a8e657e51f64436c9eba41006d5aad475c548df3b2b7a145c", "label": "R", "commit": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4", "source_path": "crates/rrx/src/execution/native/native_fixture.py", "blob": "5802c5f50b7a196008836aa6debef092eb61fb26", "whole_file_sha256": "0e46f8e3ec2cd6b3ab0248174eafbf134de0b621e493bb5ac478bcbdfd3fc5d2", "start": 21, "end": 44, "excerpt_sha256": "91cbfd8f141d9c9a8e657e51f64436c9eba41006d5aad475c548df3b2b7a145c"}
def complete(payload):
    if payload == "readonly-review":
        if PROVIDER == "claude":
            assert sys.argv[sys.argv.index("--permission-mode") + 1] == "plan"
        else:
            assert NATIVE_SANDBOX == "read-only"
        subprocess.run(["/usr/bin/git", "rev-parse", "HEAD"], check=True, stdout=subprocess.DEVNULL)
        try:
            with open("fixture-review-write", "w") as out: out.write("must be refused")
        except PermissionError:
            pass
        else:
            raise RuntimeError("readonly source accepted a normal write")
        return
    with open(os.path.join(os.environ["RRX_OUTPUT_DIR"], "fixture-ready"), "w") as ready:
        ready.write("ready\n")
    release = os.path.join(os.environ["RRX_OUTPUT_DIR"], "fixture-release")
    while not os.path.exists(release):
        time.sleep(0.02)
    with open("fixture-result.txt", "w") as result:
        result.write(os.environ["RRX_UNIT_ID"] + "\n")
    # The fixture qualifies protocol correlation, not the installed shim/native-agent matrix.
    subprocess.run(["/usr/bin/git", "add", "fixture-result.txt"], check=True, stdout=subprocess.DEVNULL)
    subprocess.run(["/usr/bin/git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "-c", "commit.gpgsign=false", "commit", "-m", "protocol fixture"], check=True, stdout=subprocess.DEVNULL)


## INLINE INPUT EXCERPT-RN-current-link
{"id": "EXCERPT-RN-current-link", "bytes": 388, "sha256": "5b2c0f5974f6cdadd4cf6e68aa33dc5f3c1879f8dfd48210d9f5e625ec577b86", "label": "RN-current", "commit": "e70faa9dc900c5df0dd26d9983425cf6d63876a3", "source_path": "crates/rrx/src/state/managed_binding/closure.rs", "blob": "32490867b2e2989591a7add30d9e89a0a1e8d1df", "whole_file_sha256": "e7b91f77e5609c95b39763f4d50fdba9869e12313c369d9dee35174b12b42519", "start": 54, "end": 64, "excerpt_sha256": "5b2c0f5974f6cdadd4cf6e68aa33dc5f3c1879f8dfd48210d9f5e625ec577b86"}
/// Borrowed W4 inputs of the SAME retained plan for one writer call.
/// Fields and construction stay in this module; owns no images or payload.
pub(in crate::state) struct PhaseClosedLink<'a> {
    closure: &'a UnlinkedPhaseClosure,
    audit_data: &'a str,
}

fn workflow_delta(
    before: &Record,
    task: &crate::domain::Task,
    original_unit: &crate::execution::ExecutionUnit,


## INLINE INPUT EXCERPT-RN-current-link-issuer
{"id": "EXCERPT-RN-current-link-issuer", "bytes": 304, "sha256": "31fc41e69771fcf1ab3b863059e7b93216c7ee9dfdf9323b2b65ff49169bd163", "label": "RN-current", "commit": "e70faa9dc900c5df0dd26d9983425cf6d63876a3", "source_path": "crates/rrx/src/state/managed_binding/closure.rs", "blob": "32490867b2e2989591a7add30d9e89a0a1e8d1df", "whole_file_sha256": "e7b91f77e5609c95b39763f4d50fdba9869e12313c369d9dee35174b12b42519", "start": 224, "end": 235, "excerpt_sha256": "31fc41e69771fcf1ab3b863059e7b93216c7ee9dfdf9323b2b65ff49169bd163"}
    })
}
impl UnlinkedPhaseClosure {
    pub(in crate::state) fn link<'a>(&'a self, audit_data: &'a str) -> PhaseClosedLink<'a> {
        PhaseClosedLink {
            closure: self,
            audit_data,
        }
    }
    pub(in crate::state) fn attempt(&self) -> usize {
        self.attempt
    }


## INLINE INPUT EXCERPT-RN-current-consumer
{"id": "EXCERPT-RN-current-consumer", "bytes": 757, "sha256": "f8c33dfc9758aa6f49782de50fe2add6c2821655b326c913074411ef021d35af", "label": "RN-current", "commit": "e70faa9dc900c5df0dd26d9983425cf6d63876a3", "source_path": "crates/rrx/src/state/execution/native_phase/nonsuccess.rs", "blob": "ea9d14630dad972851702fbad715878342fe6c64", "whole_file_sha256": "9cdf2556b4b54f4d6ce3f114d6118cc858c4073a37ebbca1c6d30dfb85952b71", "start": 155, "end": 176, "excerpt_sha256": "f8c33dfc9758aa6f49782de50fe2add6c2821655b326c913074411ef021d35af"}
        {
            let budget = InventoryBudget::new(&tx)?;
            if let Err(cause) = budget.finish(validate_open(&tx, plan)) {
                return Ok(NativeNonSuccessWrite::Conflict(cause));
            }
        }
        let reader = NonSuccessReader(());
        material.images.write_tx(
            &reader,
            &tx,
            &self.binding_permits,
            plan.proof.launch().marker(),
            plan.images.link(&plan.audit_data),
            |tx| {
                plan.proof
                    .closure(&reader)
                    .unit()
                    .write_retired_tx(tx, &plan.unit_after)
            },
        )?;
        tx.commit()?;
        Ok(NativeNonSuccessWrite::Known(PhaseClosedAcknowledgment {


## INLINE INPUT EXCERPT-CA-current-version-monitor
{"id": "EXCERPT-CA-current-version-monitor", "bytes": 1908, "sha256": "3d4bfea5270f8989539621f9ff537428a951f838032e0d4ce0703a584d5fcccd", "label": "CA-current", "commit": "2c24f3f6e66f45692e416c77dc3e8d64fcbc9276", "source_path": "crates/rrx/src/execution/native/version.rs", "blob": "bf1ee4ffe2ccd151e85e48a7b501a5f89d077ca2", "whole_file_sha256": "45c8884f33ed2f3599af0dbf21ac6363650e2c9ac143fba97bcbfeb3702a7141", "start": 705, "end": 745, "excerpt_sha256": "3d4bfea5270f8989539621f9ff537428a951f838032e0d4ce0703a584d5fcccd"}
                        Ok(0) => out_open = false,
                        Ok(n) => {
                            if !output.observe(&out_buffer[..n],true) { break; }
                        }, Err(_) => break,
                    }
                },
                read = err.read(&mut err_buffer[..read_len]), if err_open => {
                    match read { Ok(0) => err_open = false, Ok(n) => {
                        if !output.observe(&err_buffer[..n],false) { break; }
                    }, Err(_) => break }
                },
                _ = tokio::time::sleep_until(end) => break,
                _ = fence.tick() => {
                    let valid = owner.upgrade().is_some_and(|owner| {
                        owner.store.lock().ok().is_some_and(|mut store|
                            store.validate_phase_version_fence(&helper.plan,&intent).is_ok())
                    });
                    if !valid { break; }
                    if !exited {
                        // End the raw guard before the arm may run hygiene,
                        // which takes the same custody mutex again.
                        let observed_exit = { helper.raw().exited_unreaped() };
                        match observed_exit {
                            Ok(true) => {
                                exited = true;
                                let _ = helper.raw().hygiene();
                                drain_deadline = Some(tokio::time::Instant::now()+Duration::from_secs(2));
                            }, Ok(false) => {}, Err(_) => break,
                        }
                    }
                },
            }
        }
    }
    let group_hygiene = helper.raw().hygiene();
    let stop_deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        if !helper.raw().has_child() {
            break;
        }
        match helper.raw().reap() {


## INLINE INPUT CURRENT-PIN-INTERPRETATION
{"id": "CURRENT-PIN-INTERPRETATION", "bytes": 3117, "sha256": "14785ace3a836d0ef6bf818e1da8be9edc5c5539a48abf229fbd53e777b1931b"}
{
  "pins": {
    "H": "0c35653c7a8aa05c78c5285c2f4b5370db520e70",
    "R": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4",
    "CA": "95ef0e783c0b18c90dfef23ba1b77b9dc67735f1",
    "RN": "1d783440b7f4d705bf8cf4c667e022f5cec87777",
    "CA-current": "2c24f3f6e66f45692e416c77dc3e8d64fcbc9276",
    "RN-current": "e70faa9dc900c5df0dd26d9983425cf6d63876a3"
  },
  "current_delta_interpretation": {
    "CA-current": "Direct source deltas inspected: test-only observations/accessors and publication probes, service observe_task_drivers visibility, and nongrant Native contract decision extraction preserving original planner predicates. The actual Native version helper monitor also now releases its raw custody mutex guard before hygiene reacquires it (CA-current-version-monitor); this fixes setup liveness and must not be omitted from pin provenance. It is a different monitor from UnitGit capture_scoped_pinned and changes neither SC-02’s generic finalization fence nor successful settlement delivery/binding/gate/status consumers. CA95 binding/setup controls are inventory, not execution evidence here.",
    "RN-current": "Borrowed PhaseClosedLink retains SAME closure and audit_data for nonsuccess write; one corresponding consumer now passes link(). No successful owner/input/terminal/gate or normal binding consumer changed."
  },
  "unchanged_limits": [
    "This is independent factual verification of the one completed review, not another review, contract authoring or approval.",
    "No source, WHAT/HOW/master, tests, permissions, hooks or configuration changed; no new model, provider, native workload, build, test or mutant ran.",
    "The original final review body is preserved byte-for-byte. Its mandatory labels are claims being checked, not approval evidence.",
    "H is proposed only. CA and RN are independent immutable descendants of R; this report does not treat them as composed or qualified source.",
    "Original-phase successful closure and legal marker-free next Driver/Source currentness are the common necessary dependency. Broader prepared HOW9.3 continuation, later offers/Reviewer/retries/retention/restart/evidence remain open; SC-N is not silently completed.",
    "Requirements/non-Implement evidence integration is absent; existing evaluate genuinely returns Waiting. Ready capture is not Published/Passed acceptance. Implement-only success cannot qualify the full contract or full MVP.",
    "Weak lookup alone is not a lifetime defect: actual handoff/marker ticket custody is retained. RN nonsuccess borrowed link cannot act as registered successful settlement proof.",
    "Cleanup Unknown/Leftovers remain independent from work outcome and do not create authority or impose a physical-death launch/closure barrier.",
    "SourceStop, Cancel and R2 human decisions remain unresolved; no reader, timeout, row-derived grant, synthetic owner or guard relaxation settles them.",
    "source_implementation_authorized=false; native_qualified=false; mvp_qualified=false; merge_ready=false. Root must independently verify classification before any nonfinding Opus author batch."
  ]
}

## INLINE INPUT CA-current-delta.patch
{"id": "CA-current-delta.patch", "bytes": 9202, "sha256": "5674626df99af643f8abc3e5a2642563bf5865812a84cdb43e87d3e6b7a392db"}
diff --git a/crates/rrx/src/runtime/phase_jobs.rs b/crates/rrx/src/runtime/phase_jobs.rs
index b1a892b6..3ad59682 100644
--- a/crates/rrx/src/runtime/phase_jobs.rs
+++ b/crates/rrx/src/runtime/phase_jobs.rs
@@ -100,6 +100,21 @@ impl PhaseInvocation {
 }
 
 impl PhaseJobs {
+    /// Test-only borrowing of an existing original retained object. A copied
+    /// Unit ID can select a reader result, never construct an allocation.
+    #[cfg(test)]
+    pub(super) fn original_allocation(
+        &self,
+        unit: crate::execution::UnitId,
+    ) -> Result<Arc<NativeAllocation>> {
+        self.entries
+            .lock()
+            .map_err(|_| anyhow::anyhow!("phase jobs poisoned"))?
+            .values()
+            .find(|entry| entry.job.allocation.facts().unit_id == unit)
+            .map(|entry| entry.job.allocation.clone())
+            .ok_or_else(|| anyhow::anyhow!("original retained allocation absent"))
+    }
     /// Caller already holds actual Runtime admission and publishing capacity.
     /// Called before SQL effects; duplicate IDs cannot substitute an allocation.
     pub(super) fn reserve(
diff --git a/crates/rrx/src/runtime/service.rs b/crates/rrx/src/runtime/service.rs
index 6d1353ff..8db43c4a 100644
--- a/crates/rrx/src/runtime/service.rs
+++ b/crates/rrx/src/runtime/service.rs
@@ -17,7 +17,7 @@ impl Drop for Running {
 }
 
 impl Runtime {
-    fn observe_task_drivers(&self) -> Result<usize> {
+    pub(super) fn observe_task_drivers(&self) -> Result<usize> {
         #[cfg(test)]
         {
             let hook = self
diff --git a/crates/rrx/src/state/managed_binding/permits.rs b/crates/rrx/src/state/managed_binding/permits.rs
index 635bef05..92b8e157 100644
--- a/crates/rrx/src/state/managed_binding/permits.rs
+++ b/crates/rrx/src/state/managed_binding/permits.rs
@@ -242,6 +242,8 @@ struct State {
 #[derive(Default)]
 pub(in crate::state) struct PrivatePermitManager {
     state: Mutex<State>,
+    #[cfg(test)]
+    consumed_observations: std::sync::atomic::AtomicU64,
 }
 struct Revoke<'a> {
     manager: &'a PrivatePermitManager,
@@ -309,8 +311,16 @@ impl PrivatePermitManager {
                 .is_some_and(|a| a.rows.iter().all(Option::is_none)),
             "exact mutation plan not consumed"
         );
+        #[cfg(test)]
+        self.consumed_observations
+            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
         Ok(())
     }
+    #[cfg(test)]
+    pub(in crate::state) fn consumed_observations(&self) -> u64 {
+        self.consumed_observations
+            .load(std::sync::atomic::Ordering::SeqCst)
+    }
     fn matches(&self, ctx: &rusqlite::functions::Context<'_>) -> bool {
         let Some(table) = ctx.get_raw(0).as_str().ok() else {
             return false;
diff --git a/crates/rrx/src/state/managed_binding/publication.rs b/crates/rrx/src/state/managed_binding/publication.rs
index 748851f0..c2761231 100644
--- a/crates/rrx/src/state/managed_binding/publication.rs
+++ b/crates/rrx/src/state/managed_binding/publication.rs
@@ -243,7 +243,30 @@ pub(crate) fn plan_marker_publication(
             profile,
         ))
     })?;
-    // Strict decoding and both digests occur only after query-only snapshot ends.
+    let body = decide_contract(&allocation, workflow, raw, &contract_profile, &member)?;
+    let contract_body = body.raw().to_owned();
+    Ok(Arc::new(MarkerPublicationPlan {
+        owner,
+        allocation,
+        marker,
+        driver,
+        rows,
+        contract_body,
+        contract_origin,
+        contract_profile,
+    }))
+}
+// Nongrant decision used by the real marker planner, after its query-only
+// snapshot ends. The returned bytes remain pinned by validate_contract.
+fn decide_contract(
+    allocation: &NativeAllocation,
+    workflow: RecordId,
+    raw: String,
+    contract_profile: &str,
+    member: &str,
+) -> Result<Body<NativeContract>> {
+    let f = allocation.facts();
+    let contract_origin = allocation.selected_port().installation_id();
     let body = Body::<NativeContract>::decode(raw, 4096)?;
     let contract = body.parsed();
     ensure!(
@@ -263,20 +286,23 @@ pub(crate) fn plan_marker_publication(
         "actual Workflow roster digest differs"
     );
     ensure!(
-        contract.members.binary_search(&member).is_ok(),
+        contract
+            .members
+            .binary_search_by(|candidate| candidate.as_str().cmp(member))
+            .is_ok(),
         "selected Native port absent from original roster"
     );
-    let contract_body = body.raw().to_owned();
-    Ok(Arc::new(MarkerPublicationPlan {
-        owner,
-        allocation,
-        marker,
-        driver,
-        rows,
-        contract_body,
-        contract_origin,
-        contract_profile,
-    }))
+    Ok(body)
+}
+#[cfg(test)]
+pub(crate) fn check_native_contract_integrity(
+    allocation: &NativeAllocation,
+    workflow: RecordId,
+    raw: String,
+    contract_profile: &str,
+) -> Result<()> {
+    let member = member_digest(allocation.facts().role, allocation.selected_port())?;
+    decide_contract(allocation, workflow, raw, contract_profile, &member).map(|_| ())
 }
 impl MarkerPublicationPlan {
     /// Immutable original-object linkage, never a currency or effect grant.
diff --git a/crates/rrx/src/state/managed_binding/snapshot.rs b/crates/rrx/src/state/managed_binding/snapshot.rs
index 7420dce9..22e2cbff 100644
--- a/crates/rrx/src/state/managed_binding/snapshot.rs
+++ b/crates/rrx/src/state/managed_binding/snapshot.rs
@@ -229,6 +229,10 @@ pub(in crate::state) fn read_scope(
 }
 
 impl ScopePlan {
+    #[cfg(test)]
+    pub(in crate::state) fn original_workflow_version(&self) -> Option<u64> {
+        self.workflow.as_ref().map(|body| body.parsed().version)
+    }
     pub(in crate::state) fn workflow_record_mutation(
         &self,
         record: &Record,
diff --git a/crates/rrx/src/state/mod.rs b/crates/rrx/src/state/mod.rs
index f494623b..707cd77a 100644
--- a/crates/rrx/src/state/mod.rs
+++ b/crates/rrx/src/state/mod.rs
@@ -89,6 +89,9 @@ pub struct Store {
     connection: Connection,
     #[allow(dead_code)] // Actual managed marker/binder is composed separately.
     binding_permits: std::sync::Arc<managed_binding::PrivatePermitManager>,
+    /// Finite read-only observations of real committed test-build windows.
+    #[cfg(test)]
+    record_window_observations: Vec<(RecordId, u64, i64, u64, i64, u64)>,
 }
 
 fn register_writer_contract(
@@ -272,9 +275,16 @@ impl Store {
         Ok(Self {
             connection,
             binding_permits,
+            #[cfg(test)]
+            record_window_observations: Vec::new(),
         })
     }
 
+    #[cfg(test)]
+    pub(crate) fn record_window_observations(&self) -> &[(RecordId, u64, i64, u64, i64, u64)] {
+        &self.record_window_observations
+    }
+
     pub fn schema_version(&self) -> Result<i64> {
         Ok(self
             .connection
@@ -1082,6 +1092,8 @@ impl Store {
                 }
             }
         }
+        #[cfg(test)]
+        let mut window_observation = None;
         let (next_task, next_workflow) = if let Some(plan) = driver_input {
             let next_task = put_task_tx_at_with_namespace(
                 &tx,
@@ -1089,7 +1101,20 @@ impl Store {
                 plan.input_timestamp(),
                 Some(plan.input_namespace()?),
             )?;
+            #[cfg(test)]
+            let consumed_before = self.binding_permits.consumed_observations();
             let next_workflow = plan.write_input_record_tx(&tx, &self.binding_permits, workflow)?;
+            #[cfg(test)]
+            if let Some((id, version, timestamp)) = plan.record_window_probe() {
+                window_observation = Some((
+                    id,
+                    version,
+                    timestamp,
+                    next_workflow.version,
+                    next_workflow.updated_at,
+                    self.binding_permits.consumed_observations() - consumed_before,
+                ));
+            }
             (next_task, next_workflow)
         } else {
             (put_task_tx(&tx, task)?, put_record_tx(&tx, workflow)?)
@@ -1114,6 +1139,12 @@ impl Store {
             plan.activation_precommit()?;
         }
         tx.commit()?;
+        #[cfg(test)]
+        if let Some(observation) = window_observation
+            && self.record_window_observations.len() < 64
+        {
+            self.record_window_observations.push(observation);
+        }
         let mut outcome = managed_binding::ActivationCommit::Published;
         if let Some(plan) = driver_input {
             if matches!(
@@ -1123,7 +1154,10 @@ impl Store {
                 match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                     #[cfg(test)]
                     plan.activation_postcommit()?;
-                    self.publish_driver_preparation(plan)
+                    self.publish_driver_preparation(plan)?;
+                    #[cfg(test)]
+                    plan.activation_observe_published();
+                    Ok(())
                 })) {
                     Ok(Ok(())) => {}
                     Ok(Err(error)) => {


## INLINE INPUT CA-current-monitor-delta.patch
{"id": "CA-current-monitor-delta.patch", "bytes": 3466, "sha256": "2bd8c54583e4b6e75e8fa6c03786b71f34bf29d345e8f6d3ec57adffca3127dc"}
diff --git a/crates/rrx/src/execution/native/version.rs b/crates/rrx/src/execution/native/version.rs
index 39f680d5..bf1ee4ff 100644
--- a/crates/rrx/src/execution/native/version.rs
+++ b/crates/rrx/src/execution/native/version.rs
@@ -721,7 +721,10 @@ async fn capture_version(guard: CaptureOwner, owner: Weak<RuntimeOwner>) {
                     });
                     if !valid { break; }
                     if !exited {
-                        match helper.raw().exited_unreaped() {
+                        // End the raw guard before the arm may run hygiene,
+                        // which takes the same custody mutex again.
+                        let observed_exit = { helper.raw().exited_unreaped() };
+                        match observed_exit {
                             Ok(true) => {
                                 exited = true;
                                 let _ = helper.raw().hygiene();
diff --git a/crates/rrx/src/state/managed_binding/activation.rs b/crates/rrx/src/state/managed_binding/activation.rs
index 49544da6..f8ffe0c7 100644
--- a/crates/rrx/src/state/managed_binding/activation.rs
+++ b/crates/rrx/src/state/managed_binding/activation.rs
@@ -91,6 +91,9 @@ pub(crate) struct ActivationSeams {
     pub(crate) precommit: Option<ActivationFault>,
     pub(crate) postcommit: Option<ActivationFault>,
     pub(crate) deferred: Option<ActivationDeferred>,
+    /// Borrowed observation after exact publication, inside the same S3
+    /// contained segment. Returns no value and issues no authority.
+    pub(crate) published: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
 }
 pub(crate) fn plan_native_activation(
     roster: ActivationRoster,
@@ -191,6 +194,12 @@ impl NativeActivationPlan {
         Ok(())
     }
     #[cfg(test)]
+    pub(in crate::state) fn observe_published(&self) {
+        if let Some(hook) = self.seams.as_ref().and_then(|s| s.published.as_ref()) {
+            hook();
+        }
+    }
+    #[cfg(test)]
     pub(crate) async fn deferred(&self) -> Result<()> {
         if let Some(hook) = self.seams.as_ref().and_then(|s| s.deferred.as_ref()) {
             hook(self.roster.task()).await?;
diff --git a/crates/rrx/src/state/runtime/driver/preparation.rs b/crates/rrx/src/state/runtime/driver/preparation.rs
index 2ab35a50..f1fb9d7a 100644
--- a/crates/rrx/src/state/runtime/driver/preparation.rs
+++ b/crates/rrx/src/state/runtime/driver/preparation.rs
@@ -343,6 +343,12 @@ impl DriverPreparationAdvance {
         Ok(())
     }
     #[cfg(test)]
+    pub(in crate::state) fn activation_observe_published(&self) {
+        if let Some(activation) = self.input.as_ref().and_then(|i| i.activation.as_ref()) {
+            activation.observe_published();
+        }
+    }
+    #[cfg(test)]
     pub(crate) async fn activation_deferred(&self) -> Result<()> {
         self.input
             .as_ref()
@@ -397,6 +403,15 @@ impl DriverPreparationAdvance {
             write()
         }
     }
+    #[cfg(test)]
+    pub(in crate::state) fn record_window_probe(&self) -> Option<(RecordId, u64, i64)> {
+        self.input.as_ref()?.records.as_ref()?;
+        Some((
+            self.input.as_ref()?.record.id,
+            self.ticket.scope.original_workflow_version()?,
+            self.input_timestamp(),
+        ))
+    }
     pub(in crate::state) fn begin_input(self: &Arc<Self>) -> Result<Applying<'_>> {
         ensure!(
             self.input.is_some() && self.is_retained()?,


## INLINE INPUT RN-current-delta.patch
{"id": "RN-current-delta.patch", "bytes": 3480, "sha256": "2ff28c4e268d77e96d702f314fce742d80ce9d61bfb49855cf7f49d10ebc193f"}
diff --git a/crates/rrx/src/state/managed_binding/closure.rs b/crates/rrx/src/state/managed_binding/closure.rs
index 7f3895e7..32490867 100644
--- a/crates/rrx/src/state/managed_binding/closure.rs
+++ b/crates/rrx/src/state/managed_binding/closure.rs
@@ -51,6 +51,13 @@ pub(in crate::state) enum PhaseImage<'a> {
     },
 }
 
+/// Borrowed W4 inputs of the SAME retained plan for one writer call.
+/// Fields and construction stay in this module; owns no images or payload.
+pub(in crate::state) struct PhaseClosedLink<'a> {
+    closure: &'a UnlinkedPhaseClosure,
+    audit_data: &'a str,
+}
+
 fn workflow_delta(
     before: &Record,
     task: &crate::domain::Task,
@@ -217,6 +224,12 @@ pub(in crate::state) fn plan_unlinked_closure(
     })
 }
 impl UnlinkedPhaseClosure {
+    pub(in crate::state) fn link<'a>(&'a self, audit_data: &'a str) -> PhaseClosedLink<'a> {
+        PhaseClosedLink {
+            closure: self,
+            audit_data,
+        }
+    }
     pub(in crate::state) fn attempt(&self) -> usize {
         self.attempt
     }
@@ -281,8 +294,7 @@ impl PhaseClosureImages {
         tx: &Transaction<'_>,
         permits: &PrivatePermitManager,
         marker: &OriginalMarker,
-        closure: &UnlinkedPhaseClosure,
-        audit_data: &str,
+        link: PhaseClosedLink<'_>,
         w1: impl FnOnce(&Transaction<'_>) -> Result<()>,
     ) -> Result<()> {
         let before = marker.original_plan().workflow_after();
@@ -305,8 +317,8 @@ impl PhaseClosureImages {
                 .task_id
                 .map_or(SqlValue::Null, |v| SqlValue::Text(v.to_string())),
             SqlValue::Text(KIND.into()),
-            SqlValue::Integer(closure.at),
-            SqlValue::Text(audit_data.into()),
+            SqlValue::Integer(link.closure.at),
+            SqlValue::Text(link.audit_data.into()),
         ];
         let writes = vec![
             ExactRowMutation::new(
@@ -333,7 +345,7 @@ impl PhaseClosureImages {
             let predicate = names.iter().enumerate().map(|(i,n)| format!("{n} IS ?{}",i+33)).collect::<Vec<_>>().join(" AND ");
             ensure!(tx.execute(&format!("UPDATE managed_phase_operations SET {set} WHERE {predicate}"), params_from_iter(self.operation_after.iter().chain(marker.original_operation_image()?)))? == 1, "non-success operation complete CAS changed");
             ensure!(tx.execute("UPDATE records SET version=?1,body=?2 WHERE id=?3 AND kind='workflow' AND project_id=?4 AND goal_id IS ?5 AND task_id IS ?6 AND version=?7 AND body=?8", params![self.workflow_after.version,self.workflow_after_raw,before.0.id.to_string(),scope.project_id.to_string(),scope.goal_id.map(|v|v.to_string()),scope.task_id.map(|v|v.to_string()),before.0.version,before.1])? == 1, "non-success Workflow CAS changed");
-            ensure!(tx.execute("INSERT INTO audit(sequence,project_id,goal_id,task_id,kind,at,data) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![sequence,scope.project_id.to_string(),scope.goal_id.map(|v|v.to_string()),scope.task_id.map(|v|v.to_string()),KIND,closure.at,audit_data])? == 1, "non-success link missing");
+            ensure!(tx.execute("INSERT INTO audit(sequence,project_id,goal_id,task_id,kind,at,data) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![sequence,scope.project_id.to_string(),scope.goal_id.map(|v|v.to_string()),scope.task_id.map(|v|v.to_string()),KIND,link.closure.at,link.audit_data])? == 1, "non-success link missing");
             permits.ensure_consumed()
         })
     }


## INLINE INPUT RN-current-nonsuccess-delta.patch
{"id": "RN-current-nonsuccess-delta.patch", "bytes": 615, "sha256": "27ef3aaca9bfd85b10d11a216999985ef35b2c74f20ab9cb2937477a3d95a8f2"}
diff --git a/crates/rrx/src/state/execution/native_phase/nonsuccess.rs b/crates/rrx/src/state/execution/native_phase/nonsuccess.rs
index 2be1061c..ea9d1463 100644
--- a/crates/rrx/src/state/execution/native_phase/nonsuccess.rs
+++ b/crates/rrx/src/state/execution/native_phase/nonsuccess.rs
@@ -164,8 +164,7 @@ impl Store {
             &tx,
             &self.binding_permits,
             plan.proof.launch().marker(),
-            &plan.images,
-            &plan.audit_data,
+            plan.images.link(&plan.audit_data),
             |tx| {
                 plan.proof
                     .closure(&reader)
