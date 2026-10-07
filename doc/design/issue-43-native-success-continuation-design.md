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
   | I | `e658af52fb6f3747c7f7a87c1570ebb49bfb7611` | Integrated development head of `feature/issue-43-native-nonsuccess-controls`; contains R, CA and RN. Used for every reference prefixed `I:` that was added in correction batch C1 (§1.3) |

   CA and RN are independent descendants of R (`git merge-base` of all three is R). They are not composed, tested or qualified together. **I** contains all three (`git merge-base --is-ancestor` holds for R, CA and RN against I); it is an unreviewed development composition and is not the reviewed integration commit required by item 8. Correction batch C1 re-verified every corrected claim against I, because several R line references drifted (for example `runtime/phase_jobs.rs` `JobState` is `I:151–167`, the start task `I:741–801`, `bind_returned` `I:887–905`, RN `reconcile_nonsuccess` `I:377`; `execution/native.rs` `persist_saved_terminal` is `I:1260–1324`).
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

### 1.3 Correction batch C1 (delta from `0c35653`)

C1 answers the independent review recorded in `doc/verification/issue-43-native-success-continuation-review/` (`b62316b`): the original review (`sol-independent-review-result.md`) and the verified disposition (`root-verified-finding-disposition.json`, mandatory SC-01…SC-06 and SC-08…SC-12; optional SC-07, SC-13, SC-14). C1 changes no WHAT (MB1–MB9, MB-AC1…AC8 unchanged) and relaxes no parent, lock, input, owner, marker, epoch or #19 admission check. It also aligns one control with the MVP concurrency clarification in Issue #43 / #81 (at most one active Task per Project; cross-Project parallelism mandatory).

| Finding | Correction | Sections |
| --- | --- | --- |
| SC-01 | A bound, launched job whose owner is no longer live becomes due for memory-only settlement discovery (one atomic load per job per sweep); a revoked owner without a settlement is re-polled at 5 s and never concluded to be non-success | §5.2 |
| SC-02 | The running-helper 50 ms currency fence becomes a parameter of the helper capture (`HelperCurrency`); the settled path keeps the fence, its first immediate tick, identity comparison and kill-on-refusal lifetime, and runs the protected validator instead of `validate_execution` | §4, §7 |
| SC-03 | `evaluate_settled` has four substitutions: the final Unit recheck before the `Verification` record (`I:execution/workflow_gates.rs:364–370`) uses the protected validator | §9.2 |
| SC-04 | Binding confirmation is a separate factual predicate from write eligibility: it tolerates legitimate SAME-lineage Native terminal progress while keeping every parent, lock, input-identity, owner-identity and marker check | §5.3 |
| SC-05 | The terminal commit hands a sealed, non-Clone, non-Serialize `SettledTerminalImages` into the settlement; the late binder can only ask it to compare exact rows inside its Immediate | §4, §5.1 |
| SC-06 | Closure and Driver publication are admitted through Runtime-owned async entry points that mirror CA's `ActivationAdmission` and `PhaseDispatcher::publish_planned_marker`; the Store ports stay synchronous and document "caller holds `control_admission`" | §4, §5.2, §10.2, §10.4, §11, §13 |
| SC-07 (optional, adopted) | `DriverClosureAdvance` lives in `driver/marker.rs` (or a child module of it); `SettledGateCompletion` gets `belongs_to` and a consuming `into_parts` | §4 |
| SC-08 | Truthful helper/command cardinality and effect-row accounting; closure checks unsettled effects with an `EXISTS` predicate instead of a ≤257-row scan; "at most one durable write per call" is narrowed to Workflow/ledger-advancing writes | §6.2, §7, §8, §10.2, §12 |
| SC-09 | Retained memory counts every owned encoding and image of the unchanged binding plan; shared custody is listed separately | §12 |
| SC-10 / SC-14 | A bounded, read-only `workflow_wait` status field connects Workflow Waiting/Held to Goal task pages and CLI; the reader path is `state/runtime/waiting.rs` | §9.3.1, §15 |
| SC-11 | `Conflict` is a typed refusal raised only by in-transaction checks before the first write; BUSY and begin/commit errors are uncertain. SC3 uses a genuine Session-advance race. The late-planner None/live check is defense-only for the production caller | §5.4, §16 |
| SC-12 | M1 is split into a mechanical measurement (not a positive), a reachable positive within the 1 MiB payload / 2 MiB frame input limits, and a headroom control that counts only with a genuine pre-acceptance Context-history boundary; otherwise the headroom mutant is defense-only | §12, §16 |
| SC-13 (optional, adopted) | Cleanup is a read overlay (`cleanup_observations`, applied by `unit_tx` at `I:state/execution.rs:249–264`), not an `execution_units` CAS (`record_execution_cleanup`, `I:state/execution.rs:1129–1160`). The "cleanup axis (version, updated_at, cleanup)" tolerance is replaced by: the stored Unit row (raw body and indexed columns, `unit_index_matches`) is compared exactly with the terminal postimage, and only the decoded `cleanup` field, which is that overlay, is excluded from decoded-value comparisons | §2, §5.1, §7, §9.2, §10.1 |
| #81 alignment | SC10 runs four Tasks in four distinct Projects (one active Task per Project) | §16 |

### 1.4 Correction batch C2 (delta from `e9a572a`)

C2 answers the Sol 6.1 high delta review of C1 (Issue #43 comment 6031710785: REQUEST CHANGES, C1-H01, C1-M01…M05, C1-L01…L03). Each finding was re-verified against pin I before correction. WHAT, guards and the MVP rule are unchanged.

| Finding | Correction | Sections |
| --- | --- | --- |
| C1-H01 | Claim, observed and closure confirmation no longer use the open-phase validator alone. Each confirms by two exclusive branches after common immutable authority checks: complete postimage plus exact link (Known), or complete preimage plus link absence (RolledBack); anything mixed is Held. Closure follows the RN closed/open classification (`RN:state/execution/native_phase/nonsuccess.rs:194–206`) | §10.6, §11 |
| C1-M01 | Every Root admission is non-blocking (`try_lock_owned`); a busy admission keeps the action for a later turn. The Root never awaits `control_admission`, so it cannot wait on a shutdown that holds the guard while joining the service task | §4, §5.2, §10.2, §10.4, §16 SC9 |
| C1-M02 | The post-closure `NextPhaseUnavailable` status is derived from durable rows (closed operation with a success `phase_closed` link, `active=None`, not finished, Task phase = next phase, marker-free `driving` Driver); ordinary Running/Evaluating attempts report no wait | §9.3.1 |
| C1-M03 | SC3's normal-retry positive is restricted to a genuine Session-record advance (otherwise SETUP); a Task/Goal change is a separate parent-drift negative that stays Held with zero links | §16 SC3 |
| C1-M04 | The "closure effect probe accepts Unknown" mutant is defense-only: no genuine producer leaves an Unknown effect after a Passed observation while closure is still reachable | §16 mutants |
| C1-M05 | `SettledGateCompletion` carries a sealed `SettledGateOutcome::{Known(GateOutcome), Unknown}`; `evaluate_settled` returns an `Unknown` completion for any failure after its claim check, so `gate_unknown` reaches `gate_observed` through the producer only | §4, §9.2, §9.3 |
| C1-L01 | A and B are each ≤4096 (the `< 4096` check runs before insertion), so N ≤ 8234; per-helper fence rechecks are ≤1401 (60 s collection plus ≤10 s reap); `retained_git` helpers use their own retained fence, not G1–G3 | §8.1, §12 |
| C1-L02 | `Conflict` is defined as a deterministic refusal with no write possible: the pre-transaction selected-database refusal (no transaction constructed) or an in-transaction check before the first `execute` | §4, §5.4 |
| C1-L03 | Implementation starts only from a separately composition-reviewed commit containing I (or a successor that satisfies the same condition) | §19 |

## 2. Verified source map

| Item | Fact (V unless marked) |
| --- | --- |
| First Executor per class | `phases()` (`workflow.rs:153–188`): QUICK = `[Worktree, Implement, Commit, Tests, ImplementationReview, Pr]`; STANDARD/STRICT start `[Issue, Worktree, Requirements, …]`. Actor map `:103–114`: Requirements, Design, Implement and ImpactAnalysis are Executor phases. `preflight_installed_native` (`workflow/driven_initial.rs:11–51`) admits only the first configured Executor at generation 1. Design and ImpactAnalysis are never a first Executor. |
| CA control class | The R `plan()` Task is QUICK (`runtime/tests.rs:34–48`). CA's fixture sets minimum QUICK, risk mapping QUICK and reviewers `rev-a`/`rev-b` (CA diff to `runtime/installation/tests.rs`). The CA first Executor is therefore **Implement**. Accepted Goals take `max(task, minimum, risk_mapping)` (`runtime/goal.rs:205–208`), so default policy yields **Requirements**. |
| Fixture modes | CA's fixture injects `WORKFLOW_SCENARIO='answer-normal'`. For codex this emits an `APPROVE` item with no commit (`native_fixture.py:83–117`). Without a scenario, `complete()` waits for `fixture-release`, writes `fixture-result.txt`, commits with `/usr/bin/git`, then emits correlated completion (`:21–44,130–134,205–207`). |
| Root job custody | `JobState` retains the custody, observation, launch, `outcome: RetainedStart::Launched{_handle, binding: Arc<NativePhaseBinding>}`, `binding_plan`, `binding_error`, RN's `nonsuccess`/`closed_ack`, `closure_due`/`closure_backoff`, `uncertain`, `slot_released` and `attention` (`I:runtime/phase_jobs.rs:151–167`; `RetainedStart` `I:169–174`). `InvocationObservation` has no settled-success value (`I:139–149`). The start task records the outcome and sets `Binding` (`I:774–778`), then calls `bind_returned` once and sets `Bound` or `BindingHeld` (`I:785–799`); `bind_returned` retains the plan before Store and never retries (`I:887–905`). RN's due predicate visits only ended starts (`outcome` Err, or no outcome + Uncertain + finished) whose `closure_due` passed (`I:398–403`); a successful `Launched` job is never visited by RN. Settlement never writes `JobState` (C1, SC-01). |
| Settlement issuer | `PhaseActor::settled` (`execution/native/phase_protocol.rs:257–301`) is called only from `persist_saved_terminal` when `owned_success` (`I:execution/native.rs:1260–1324`, call at `I:1301`). `NativePhaseSession.state` is an atomic and `is_live()` is a lock-free load (`I:phase_protocol.rs:401–403`); `completed` projects and revokes (`I:618–619`) before it sets the settlement Weak under the projection mutex (`I:631–639`). A revoked owner with no settlement can therefore be that gap, a `Core::drop` revoke (`I:native.rs:2346`) whose saved terminal is settled later (`I:native.rs:690–697`), a non-success terminal (`I:native.rs:1309–1310`) or a failed `settled` (`I:295–297`); it is never by itself evidence of non-success. `OwnedPhaseSettlement::completed` (`:553–641`, `pub(super)`) checks the exact receipt, WorkKnown+Success, closed effects, the Exited Session, the allocation and the real ACK. It then `revoke`s the owner (`:619`) and projects a Weak. `PhaseActor.retained.settlement` holds it strongly (`:91–96`). |
| Post-settlement read path | `NativePhaseSession::binding_snapshot` (`:416–429`, `pub(crate)`) upgrades the projection's consumed and settlement Weaks into a new `NativePhaseBinding`. The Root job already retains `binding.owner_arc()` (`:432–434`). `NativeSessions::release` refuses phase entries (`execution/native.rs:700–703`), so the strong settlement stays in the registry Entry (`:102–109`). |
| Normal binder | `normal_eligibility` (`state/managed_binding/binding.rs:158–181`) requires live, no settlement, open native effects, no work and the Unit in DispatchPending/Running/WaitingQuota. Its SQL also requires a non-closed invocation (`:148,207`) and a live Session state (`:91–97`). For an existing link, `plan_managed_binding` accepts only the exact first `normal_return` tuple (`:331–358`). The header states that closed-settlement reconciliation is separate (`:1–3`). **No late binder exists in R, CA or RN.** |
| Terminal postimages | For an owned Completed success, the Unit becomes WorkKnown/Success with `native_effects_open=false`; `result_finalization_open` **stays true** (`state/execution/native_phase/terminal.rs:472–510`). Readiness becomes `closed`/`start_ended`/`known_terminal` (`:563–571`). Admission becomes `settled = confirmed ∧ input effect Confirmed` (`:585–593`). These postimages are planned from the actual preimages read under the terminal snapshot (`I:terminal.rs:472–598`) and are retained only inside the private `NativeTerminalPlan` (`I:116–136`); `commit()`/`into_parts` export only the Unit value, receipt, closed Session, record version and `owned_success` (`I:150–170, 676–690`). The owner row is not changed by the terminal (`committed_tx` checks its preimage, `I:669`). After that commit, Core calls `record_execution_cleanup` (`execution/native.rs:1874`), which inserts a `cleanup_observations` row and an event only; `unit_tx` overlays the latest observation onto the decoded `cleanup` field (`I:state/execution.rs:249–264, 1129–1160`). The stored `execution_units` row is not advanced (C1, SC-13). |
| Successor reader | `plan_current_phase` and `validate_current_tx` (`state/managed_binding/successor.rs:206–377`) require the operation open, the exact marker Task, the P/G/T, Workflow, lock and Context projection (`state/managed_binding/snapshot.rs:349–400`), the exact Unit index and the ledger head. Every link must carry the original `context_version` (`successor.rs:270`). |
| Ledger triggers | `state/managed_binding/schema.rs:358–416` enforces: per-operation allowances (`session_bound` 1, `native_diagnostic` 47, `gate_claim` 99, `gate_observed` 99, `gate_hold` 8, `terminal_decision` 1, `phase_closed` 1) (`:397`); `context_version` equals the operation's (`:404`); `phase_closed` iff `phase_open=0` (`:408`); the Workflow version equals `workflow_version_after` (`:411`); `gate_observed` immediately follows a `gate_claim` (`:413`). |
| Record and permit guards | Contracted Workflow and Session records need an exact `records` permit (`schema.rs:317–337`). The permit catalogue covers contracts, operations, marker bodies, owners, inputs, admissions, readiness, `task_drivers`, `source_recoveries`, `records` and `audit` (`state/managed_binding/permits.rs:23–180`). Tasks, Context, result artifacts and Units are **not** permit tables. |
| Generic refusals | Generic finalize authority `validate_authority(…, finalize)` calls the generic Driver `validate` (`state/execution.rs:458–497`), which refuses `row.marker.is_some()` (`state/runtime/driver.rs:339–342`). Generic Source7 `validate_row` refuses marker anchors (`state/execution/source_recovery.rs:296–308`). Generic `put_workflow_transition_inner` calls the Driver `validate`, Source `before_write`/`after_write`, `put_task_tx` and `put_record_tx` (`state/mod.rs:697–699,1078–1099`). |
| Capture | `ManagedWorkflowSources::frame` (`execution/workflow_source.rs:760–921`) captures for a successful Executor Unit with open finalization (`:839–863`), via `UnitGit` (`execution/git_io.rs:135–241`). Each `UnitGit` helper reaches three generic currency checks: the pre-spawn `validate_execution(…, false, true)` (`I:git_io.rs:169`), `reserve_execution_helper_pinned` → `validate_authority` (`I:state/execution/effects.rs:61`), and the running fence of `process::capture_scoped_pinned` (`I:execution/process.rs:357–380`: a 50 ms `tokio::time::interval` whose first tick completes immediately, comparing the pinned Unit identity and calling `validate_execution(&current.authority(), native, !native)` at `I:375`). A fence refusal drops the capture future, signals the process group and journals the helper effect `Unknown` (`I:git_io.rs:220–238`). The capture then uses `ResultStore::capture` (`execution/results.rs:195–370`). Capture checks base ancestry and the exact commit, stages the artifact, fetches commit and base into `results.git` with fsck, writes a durable manifest, then marks it Ready. Capture helpers write only `managed_effects` rows (`state/execution/effects.rs:41–89`), not the Unit. `committed_input` reaches `frame()` (`workflow_source.rs:939–956`). |
| Publication | `WorkflowPublication` has a private constructor; its only issuer is `ResultStore::workflow_publication` (`results.rs:387–432`), which uses generic `validate_execution` before (`I:399`) and after (`I:421`) its five-helper `verify_inner(Current)` graph verification through `UnitGit::new` (`I:412–414`). `publish_workflow_result_tx` (`state/execution/artifacts.rs:392–481`) calls `validate_authority`, publishes the artifact and writes the Unit. Historical `RetainedGit` inspection needs only epoch and artifact currency (`artifacts.rs:5–40`). |
| Gates | `ManagedWorkflowGates::evaluate` (`execution/workflow_gates.rs:203–410`) supports Issue/Worktree, Implement (terminal plus Ready artifact, `:262–287`) and the Commit family (`:288–322`). Every other phase, including Requirements, returns `Waiting("<phase> requires its qualified production evidence integration")` (`:323–328`). Implement's `terminal` uses a `SessionStatus` DTO and generic `validate_execution` (`I:157–201`, call at `I:168`). After historical `verify` and source recapture, `evaluate` performs a second, separate generic check, `validate_execution(&unit.authority(), false, true)` compared with the checked Unit (`I:364–370`), before the `Verification` record. Passed writes a `Verification` record through `put_record` (`:378–394`); that kind is not record-guarded. |
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
| Normal binding uncertain commit (including Native terminal progress before confirmation) | Confirm the SAME retained plan with the lineage confirmation predicate (§5.3); never replan before RolledBack |
| NotDispatched or pre-ACK owner, no settlement | Normal binding only; late planning refuses (no settlement) |
| Known failure, Unknown work, HistoricalDraft, non-owned receipt, capacity/quota interruption after Bound | No settlement exists. Held: "bound non-success closure unavailable" (§18). RN1 does not apply (it needs unbound, zero links) |
| Cleanup Unknown/Leftovers | Neither blocks nor authorizes. Cleanup observations do not change the stored Unit row; the decoded `cleanup` overlay is excluded from decoded comparisons (§5.1, §7) |
| Owner revoked, no settlement observed (the revoke→settle gap, a Core drop with a saved terminal not yet settled, a non-success terminal, or a failed `settled`) | No conclusion and no success credit. The job is re-polled read-only at 5 s (§5.2) until a settlement appears or another closure retires it; a bound non-success stays Held ("bound non-success closure unavailable", §18) |
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
// Conflict is raised ONLY by a deterministic refusal with no write possible (§5.4): the
// pre-transaction selected-database refusal (no transaction constructed) or a typed refusal of
// an in-transaction check that ran before the first execute. BUSY, begin, execute-after-first-
// write and commit errors are Err (uncertain) and go to confirmation.
pub(crate) enum ManagedBindingWrite { Known(Arc<BindingAcknowledgment>), Conflict(anyhow::Error) } // Err = uncertain
pub(crate) enum ManagedBindingConfirmation { Known(Arc<BindingAcknowledgment>), RolledBack }        // Err = Held
impl Store {
    // Signature change; the only caller is runtime::phase_jobs.
    pub(crate) fn bind_managed_phase(&mut self, plan: &Arc<ManagedBindingPlan>) -> Result<ManagedBindingWrite>;
    /// Lineage confirmation (§5.3); a separate predicate from write eligibility.
    pub(crate) fn confirm_managed_binding(&mut self, plan: &Arc<ManagedBindingPlan>)
        -> Result<ManagedBindingConfirmation>;
}

impl BindingAcknowledgment {
    pub(crate) fn matches_owner(&self, owner: &NativePhaseSession) -> bool; // ptr
    pub(crate) fn marker(&self) -> &OriginalMarker;
}

// ---- state/execution/native_phase/terminal.rs (extended; C1, SC-05) ----
// Fields private to `terminal`; built only inside NativeTerminalPlan::commit
// (I:terminal.rs:676) for an owned success; non-Clone, not Serialize/Deserialize,
// no other constructor.
pub(crate) struct SettledTerminalImages {
    unit: UnitImage,               // = the plan's unit_after {value, raw} (I:terminal.rs:10–13, 121)
    invocation: InvocationImage,   // = the plan's invocation_after (Closed, thread/turn)
    receipt_raw: String,           // exact native_results body written by the terminal
    session: PairRow,              // records row of the Exited Session (session_after)
    owner: PairRow,                // unchanged registered-owner preimage (the terminal never writes it)
    readiness: PairRow,            // readiness_after
    admission: PairRow,            // admission_after (always Some for an owned success)
}
impl SettledTerminalImages {
    /// Exact row comparison only (reuses PairRow/InvocationImage::validate_tx and the
    /// receipt check factored out of committed_tx). Does not compare the Unit.
    pub(crate) fn validate_tx(&self, tx: &Transaction<'_>) -> Result<()>;
    /// Exact stored Unit row (raw body and indexed columns, unit_index_matches) against
    /// the terminal's unit_after raw. The decoded `cleanup` overlay is not part of it (SC-13).
    pub(crate) fn validate_unit_tx(&self, tx: &Transaction<'_>) -> Result<()>;
}
// NativeTerminalCommit gains `images: Option<SettledTerminalImages>` (Some iff owned_success);
// into_parts returns it. Re-exported crate-wide only as a type name (native_phase.rs, state/mod.rs).

// ---- execution/native/phase_protocol.rs (extended; C1, SC-05) ----
// PhaseActor::settled(.., images) and OwnedPhaseSettlement::completed(.., images) stay pub(super).
// OwnedPhaseSettlement gains a private `images: SettledTerminalImages`; a settled replay keeps
// the first images and drops the new ones. There is no getter that returns the images.
impl OwnedPhaseSettlement {
    pub(crate) fn validate_terminal_images_tx(&self, tx: &Transaction<'_>) -> Result<()>;
    pub(crate) fn validate_terminal_unit_tx(&self, tx: &Transaction<'_>) -> Result<()>;
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
    /// Synchronous. Caller holds `control_admission` (a `SuccessAdmission`, §4 runtime), as
    /// documented for publish_managed_marker (I:state/managed_binding/publication.rs:491–492).
    pub(crate) fn close_phase_success(&mut self, m: &SuccessClosureMaterial)
        -> Result<SuccessWrite<SuccessClosureAcknowledgment>>;
    pub(crate) fn confirm_phase_success(&mut self, m: &SuccessClosureMaterial)
        -> Result<SuccessConfirmation<SuccessClosureAcknowledgment>>;
    /// Synchronous; caller holds `control_admission`. Exact committed-row check plus the SAME
    /// association's publish_exact (mirrors publish_driver_marker).
    pub(crate) fn publish_success_driver(&mut self, ack: &Arc<SuccessClosureAcknowledgment>) -> Result<()>;
    /// Replaces reserve_execution_helper_pinned's validate_authority (I:effects.rs:61) with
    /// validate_settled_tx; kind "git_helper" only (I:effects.rs:51–53). retained_git effects
    /// keep their unchanged reservation path (I:artifacts.rs:91–108).
    pub(crate) fn reserve_settled_helper(&mut self, c: &SettledCurrency, id: OperationId, path: &Path)
        -> Result<()>;
    /// Read-only protected currency check used by the running-helper fence (§7).
    pub(crate) fn validate_settled_helper(&self, c: &SettledCurrency, pinned: &ExecutionUnit) -> Result<()>;
    pub(crate) fn stage_settled_result(&mut self, c: &SettledCurrency, a: &ResultArtifact) -> Result<()>;
}

// ---- state/runtime/driver/marker.rs (extended; C1, SC-07) ----
// Defined in marker.rs itself (or its child module marker/closure.rs), because the
// DriverMarkerAdvance fields are private to `marker` (I:driver/marker.rs:10–19). A sibling
// driver/closure.rs could see the parent's private driver::Row but not those fields. Re-exported at driver.rs like DriverMarkerAdvance.
pub(crate) struct DriverClosureAdvance { /* ticket; old = SAME post-marker Row + body; new = marker-free Row + body */ }
impl DriverMarkerAdvance {               // existing type
    pub(crate) fn plan_success_closure(&self, task_after: &Task, workflow_after: &Record,
        context_after: &ContextVersion, unit_after: &ExecutionUnit) -> Result<DriverClosureAdvance>;
}
impl DriverClosureAdvance {
    pub(in crate::state) fn exact_mutations(&self) -> Result<Vec<ExactRowMutation>>; // mirrors the marker advance
    pub(crate) fn validate_current_tx(&self, tx: &Transaction<'_>) -> Result<()>;
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
// ---- execution/process.rs (extended; C1, SC-02) ----
pub(crate) enum HelperCurrency<'a> {
    Generic { native: bool, driver: Option<&'a DriverReadTicket> },   // existing behavior, byte-identical
    Settled(&'a SettledCurrency),                                     // protected path only
}
/// The existing capture_scoped_pinned becomes a thin wrapper passing HelperCurrency::Generic.
/// Both arms keep the 50 ms interval (first tick immediate), the pinned identity comparison
/// (scope, generation, owner_epoch, session_id) and the kill-on-Err drop lifetime.
pub(crate) async fn capture_scoped_with(child: OwnedProcess, owner: &RuntimeOwner,
    pinned: &ExecutionUnit, currency: HelperCurrency<'_>) -> Result<CommandCapture>;
impl ManagedWorkflowSources {
    pub(crate) async fn capture_settled(&self, s: &Arc<SettledPhase>, project: &Project, task: &Task)
        -> Result<Arc<Frame>>;
    pub(crate) async fn retire_closed_handoff(&self, task: TaskId, ack: &ClosedPhaseAck) -> Result<()>;
}
// C2, C1-M05: the producer seals an Unknown disposition; GateOutcome (I:workflow.rs:302–306)
// has only Passed/Waiting/Failed. Built only inside evaluate_settled; no other constructor.
pub(crate) enum SettledGateOutcome { Known(GateOutcome), Unknown }
pub(crate) struct SettledGateCompletion { outcome: SettledGateOutcome, receipt: Option<Record>, claim: Arc<GateClaimAcknowledgment> }
impl SettledGateCompletion {             // C1, SC-07; precedent InitialGateCompletion (I:workflow_gates.rs:20–35)
    pub(crate) fn belongs_to(&self, claim: &Arc<GateClaimAcknowledgment>) -> bool;   // Arc::ptr_eq
    pub(crate) fn into_parts(self) -> (SettledGateOutcome, Option<Record>, Arc<GateClaimAcknowledgment>);
}
// plan_settled_gate_observed checks belongs_to(claim) before into_parts.
impl ManagedWorkflowGates {
    pub(crate) async fn evaluate_settled(&self, s: &Arc<SettledPhase>, claim: &Arc<GateClaimAcknowledgment>,
        invocation: PhaseInvocation) -> Result<SettledGateCompletion>;
}

// ---- runtime (extended) ----
pub(crate) struct SuccessContinuation {                     // runtime::phase_jobs; held in JobState
    settled: Arc<SettledPhase>,
    stage: Mutex<SuccessStage>,                               // leaf mutex, single-flight token
}
impl SuccessContinuation {                                    // C1: consumers outside phase_jobs
    pub(crate) fn settled(&self) -> &Arc<SettledPhase>;
    /// Single-flight: Some(guard) only if no other caller holds the stage.
    pub(crate) fn try_stage(&self) -> Result<Option<SuccessStageGuard<'_>>>;
}
pub(crate) enum ClosedPhaseAck { NonSuccess(Arc<PhaseClosedAcknowledgment>), Success(Arc<SuccessClosureAcknowledgment>) }
pub(crate) enum SettledLookup { NoHandoff, Pending, Held(&'static str), Settled(Arc<SuccessContinuation>), Closed(ClosedPhaseAck) }
impl Runtime {
    pub(crate) fn settled_phase(&self, task: TaskId, association: &DriverAssociation) -> Result<SettledLookup>;
}
// C1, SC-06: admission and Runtime lifetime bridge, defined in runtime/installation.rs beside
// ActivationAdmission (its fields are private to that module). Mirrors ActivationAdmission
// (I:runtime/installation.rs:84–87, 148–196) and publish_planned_marker (I:phase_supervisor.rs:1067–1152).
// Fields drop in declaration order: the admission guard first, then the strong Runtime.
pub(crate) struct SuccessAdmission { _admission: tokio::sync::OwnedMutexGuard<()>, _runtime: Arc<Runtime> }
impl InstalledDriverComposition {
    /// Driver worker path: Weak<Runtime> upgrade ("Runtime ended" refusal), then a biased
    /// select of lifetime.cancelled() vs control_admission.clone().lock_owned(), then
    /// ensure!(service_running() && is_current()) and the association/belongs_to checks.
    pub(crate) async fn admit_success(&self, lifetime: &WorkerLifetime) -> Result<SuccessAdmission>;
}
impl Runtime {
    /// Root path (C2, C1-M01): upgrade the service task's Weak<Runtime> (refuse if ended), then a
    /// NON-BLOCKING control_admission.clone().try_lock_owned() while holding that strong Arc, then
    /// ensure!(service_running() && !stopping). Ok(None) when the admission is busy: the action is
    /// kept for a later turn and nothing is attempted. The Root never awaits control_admission,
    /// because shutdown holds it while joining the service task (I:runtime/service.rs:125–143).
    /// Runtime::drop takes no admission (I:runtime/mod.rs:130–135); holding the strong Arc for
    /// the admitted segment is what prevents drop from running inside it.
    pub(super) fn try_admit_root_success(weak: &Weak<Runtime>) -> Result<Option<SuccessAdmission>>;
}
impl PhaseDispatcher {
    /// Root closure confirmation only (the Root never commits a new closure, §14): under the
    /// caller's SuccessAdmission, ONE synchronous Store turn: confirm_phase_success, then (if
    /// Known) publish_success_driver under the same Store guard.
    pub(super) fn confirm_success(&self, _admitted: &SuccessAdmission, material: SuccessClosureMaterial)
        -> Result<SuccessTurn>;
    /// Root publication retry: under the caller's SuccessAdmission, one Store turn
    /// (publish_success_driver). One admission per action; never held across a sleep or backoff.
    pub(super) fn retry_success_publication(&self, _admitted: &SuccessAdmission,
        ack: &Arc<SuccessClosureAcknowledgment>) -> Result<()>;
}
// The success sweep is split (C1, SC-06):
//  (a) synchronous classification and unadmitted Store turns (binding confirm/write, claim and
//      observed confirmation), called where reconcile_nonsuccess is called (I:runtime/service.rs:92).
//      Binding needs no admission: it never publishes a Driver cache, and the existing normal
//      bind_returned takes none (I:runtime/phase_jobs.rs:887–905).
impl PhaseJobs { pub(super) fn reconcile_success(&self, phases: &PhaseSupervisor, stopping: &AtomicBool) -> Result<SuccessSweep>; }
//  (b) the returned SuccessSweep carries at most 8 admitted actions in total with (a)'s turns
//      (closure confirmation, Driver publication retry). The service loop takes them with
//      into_actions(); for each it calls Runtime::try_admit_root_success; on Some it runs the
//      PhaseDispatcher turn and drops the admission before the next action; on None it keeps the
//      action (pending) and stops processing actions for this turn. No await occurs here.
pub(super) struct SuccessSweep { actions: Vec<SuccessAction>, pending: bool }
impl SuccessSweep { pub(super) fn into_actions(self) -> (Vec<SuccessAction>, bool); }

// ---- runtime/control.rs + state/runtime/waiting.rs (extended; C1, SC-10/SC-14) ----
#[derive(Clone, Copy, Serialize, Deserialize)] #[serde(rename_all = "snake_case")]
pub enum WorkflowWaitKind { EvidenceIntegrationUnavailable, GateFailed, GateUnknown, NextPhaseUnavailable, Held }
#[derive(Clone, Serialize, Deserialize)] #[serde(deny_unknown_fields)]
pub struct WorkflowWait { pub kind: WorkflowWaitKind, pub detail: String /* one of the fixed §9.3 constants, ≤128 B */ }
// TaskFacts gains `#[serde(default, skip_serializing_if = "Option::is_none")] workflow_wait: Option<WorkflowWait>`.
pub(super) fn workflow_wait(c: &Connection, task: &Task) -> Result<Option<WorkflowWait>>; // §9.3
impl PhaseHandoffs { pub(super) fn retire_closed(&self, task: TaskId, ack: &ClosedPhaseAck) -> Result<()>; }
// RN's retire_closed_marked / retire_closed accept ClosedPhaseAck (generalized; RN behavior unchanged).
pub(crate) enum InvocationObservation { /* existing */ ClosedSuccess }
```

**Existing APIs consumed unchanged (V):**
- `NativePhaseSession::binding_snapshot`, `is_live` and `validate_known_registration`;
- the accessors of `NativePhaseBinding` and `OwnedPhaseSettlement`, and `belongs_to` (C1 adds only `validate_terminal_images_tx`; no existing accessor changes);
- `ConsumedPhaseInput::{belongs_to, effect, frame_sha256, acknowledgement}`;
- `plan_current_phase` and `validate_current_tx`;
- `OriginalMarker::validate_driver_live_tx`;
- `ExactRowMutation` and `PrivatePermitManager::with_exact_permit`;
- `charged_scope_bytes`;
- `ResultStore::verify` (historical `RetainedGit`) and `ready_result`;
- `DriverAssociation::{same_association, publish_exact}`;
- `prepare_pack`, `validate_transition`, `validate_context` and `validate_evidence`.

No existing private constructor is widened. `OwnedPhaseSettlement::completed` stays `pub(super)` (it gains the `images` argument, still sealed), `NativeTerminalCommit` is still not re-exported, and `WorkflowPublication`'s fields stay private. The `TaskFacts` wire change is additive but `deny_unknown_fields` makes older clients reject the new field; the CLI and Runtime ship from one binary, so this is recorded in the protocol notes rather than as a version bump (#81 owns the versioned client API).

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
  - `s.unit()` is WorkKnown/Success with closed effects, open finalization and session = allocated (decoded value; the `cleanup` overlay is not compared).
  - `s.session()` is Exited with `native_ref == s.thread()`.
- **Unit lineage (C1, SC-13).** Inside the Immediate, `s.validate_terminal_unit_tx(tx)` compares the stored current Unit row (raw body and indexed columns, `unit_index_matches`) exactly with the terminal's sealed `unit_after` raw. Cleanup observations never change that stored row; the decoded `cleanup` field is an overlay from `cleanup_observations` and is excluded only from decoded-value comparisons (`s.unit()` versus the decoded current Unit, used at planning outside Store). No other Unit field may differ.
- **Immediate conjuncts (C1, SC-05).** These run in addition to the binder's `validate_current_tx`, `validate_driver_live_tx` and negative identities:
  - `s.validate_terminal_images_tx(tx)`: the sealed `SettledTerminalImages` retained from the SAME terminal commit compare, exactly and only, the invocation row (Closed, with the terminal's `native_thread`/`native_turn`), the `native_results` row (the receipt raw the terminal wrote), the Exited Session `records` row, the unchanged registered-owner row, the `managed_phase_readiness` row and the `managed_phase_admissions` row. The late binder never sees, copies or constructs these images, and no image is read from current rows and adopted as expected;
  - additional scalar conjuncts derived from the settlement's existing accessors: the admission's `input_effect_id`/`frame_sha256` equal `s.consumed()`, and the latest Session is Exited with `native_ref == s.thread()`. The late variant of `latest_session` admits only Exited; the normal variant is unchanged;
  - the registered-owner check is the late variant of `validate_registered_owner_tx`: the owner row is exact (the terminal never writes it) and the joined invocation is required to be `closed`; the normal variant keeps `<>'closed'`.
- **Post-exit Session writes.** If a legitimate writer rewrites the Exited Session `records` row after the terminal, the exact image no longer matches and late binding is a definitive `Conflict` followed by Held. No such writer is known at I; SC2 asserts the late bind still happens once with cleanup Unknown present. This is a stated limit, not a relaxation.
- **Projection and payload.** The projection is identical to normal (`session_id` plus derived `execution`). The payload is identical except `proof_source="closed_settlement"` and `private_receipt_ref=<receipt id>`, as managed binding §4 already specifies.
- **Fast terminal.** The terminal commit re-validates `validate_current_tx` itself (`I:state/execution/native_phase.rs:456`), so a terminal and a normal binding serialize on the SQLite writer lock. A terminal that committed before the normal binding Immediate began changes the Unit index, so the normal binding's in-transaction check refuses before any write: a typed definitive `Conflict` (§5.4), and a later turn plans late.

### 5.2 Root reconciliation sweep (`PhaseJobs::reconcile_success`)

The service loop calls the synchronous classification after RN's `reconcile_nonsuccess` (`I:runtime/service.rs:92`); its `pending` ORs into the loop's `pending`. It reuses RN's sweep discipline: it snapshots ≤128 entries under `entries`, classifies under `job.state` only, uses its own `success_cursor`, takes ≤8 turns per sweep, runs ≤1 query-only snapshot plus ≤1 Store-mutex transaction per turn, and backs off 100 ms → 5 s with its own `success_due`/`success_backoff` fields (RN's `closure_due` is untouched). Turns that need `control_admission` (closure confirmation, Driver publication retry) are returned as `SuccessAction`s; after the synchronous part returns, the service loop runs each under its own non-blocking `Runtime::try_admit_root_success` admission through `PhaseDispatcher::{confirm_success, retry_success_publication}` (§4, SC-06; C2, C1-M01). A returned action consumes its turn; a busy admission leaves the action retained and pending for a later turn, with no await.

A job is **due** (C1, SC-01) when its `outcome` is `Ok(Launched)`, its `success_due` has passed, and one of these holds:
- **(D1) Unbound.** Its binding is not yet acknowledged (no plan, an uncertain retained plan, or a definitive `Conflict` awaiting backoff).
- **(D2) Bound, awaiting settlement.** Its binding is acknowledged, no `SuccessContinuation` is installed, and `!binding.owner().is_live()`. `is_live()` is one atomic load with no lock and no SQL (`I:execution/native/phase_protocol.rs:401–403`); a live owner is skipped at that cost and does not consume a turn. This class exists because settlement is projected into the Native owner and never changes `JobState.outcome` (`I:phase_protocol.rs:618–639`; `I:runtime/phase_jobs.rs:774–799`), and no watch or Notify exists for settlement. It does not rely on notifications or passive observers.
- **(D3) Housekeeping.** Its `SuccessContinuation` has a pending uncertain write, an unpublished Driver closure, or releasable acknowledgments.

Per turn, in order:

1. **Uncertain retained binding plan.** Run `confirm_managed_binding(plan)`:
   - Known → install `BindingAcknowledgment`;
   - RolledBack → clear the uncertain mark and keep the plan;
   - Err → Held, re-probed read-only every 5 s.
2. **No acknowledgment.** Take `snap = binding.owner_arc().binding_snapshot()?`:
   - settlement Some → `plan_late_binding`, then `bind_managed_phase`. The new plan replaces the retained one **only** after that retained plan is RolledBack or definitively refused; never both. A late plan's own definitive `Conflict` (§5.1) is Held: it is not replanned, because every late image derives from the SAME settlement and a mismatch cannot be repaired by replanning;
   - settlement None, owner live, last refusal a definitive typed `Conflict` (§5.4) → retry `plan_managed_binding` after backoff;
   - otherwise not due.

   A retained plan is replaced only after it is RolledBack or definitively refused; the start task's existing `ensure!(binding_plan.is_none())` (`I:runtime/phase_jobs.rs:891–896`) is generalized to that rule.
3. **Acknowledgment, owner not live (D2).** Take `snap = binding.owner_arc().binding_snapshot()?` (projection mutex plus Weak upgrades; memory only):
   - settlement Some → `SettledPhase::issue(snap, ack)`, then install `SuccessContinuation` once (pointer-checked) and send a watch hint;
   - settlement None → **not yet settled; no conclusion.** A revoked owner without a settlement is not evidence of non-success: `completed` revokes the owner (`I:phase_protocol.rs:619`) before it sets the settlement Weak under the projection mutex (`I:631–639`), so a sweep can observe that gap; and `Core::drop` revokes first (`I:execution/native.rs:2346`) while a saved terminal can still be persisted and settled later (`I:native.rs:690–697`). The job stays in D2 with `success_backoff` raised to 5 s and the attention "owned success settlement not observed"; it keeps being re-polled at that cadence, with no Store access and no success credit. It leaves D2 only when a settlement appears (continuation installed), when RN or a later bound non-success closure retires the job, or at shutdown. Cost: one atomic load plus one projection-mutex snapshot per such job per 5 s, inside the ≤8-turn cap.
4. **Continuation housekeeping (D3).** At most one of: confirm a retained uncertain claim or observed plan (Store turn, no admission); return a closure-confirmation action (admitted, §10.2); return a Driver-publication retry action (admitted, §10.4); or release acknowledgments (§10.5).

The start task's existing one-shot `bind_returned` remains the fast path. It now installs its plan before Store, as today, and records `Known`, `Conflict` or uncertain instead of only Ok/Err. Its outcome is never retried in the start task.

**Convergence.** Normal and late cannot both commit: the trigger allows at most one `session_bound` per operation, the first-link CAS requires the original Workflow, and only one plan is retained per job. Duplicate delivery (start task and sweep, or repeated sweeps) meets the single-flight stage token and then exact confirmation. Lost, full or closed watch notifications only delay the sweep; the service timer re-classifies from retained state, including D2 for a normally bound job that settles later.

**Sweep cost.** Per sweep: ≤128 jobs classified under short `job.state` locks, plus one atomic load per D2 candidate; ≤8 turns; per turn ≤1 query-only snapshot plus ≤1 Store transaction, or one returned admitted action.

### 5.3 Confirmation

**Write eligibility and factual confirmation are separate predicates (C1, SC-04).** Write eligibility (`bind_managed_phase`, normal or late) keeps the full native currency of its kind unchanged: for normal, `normal_eligibility` (live, no settlement, effects and finalization open, no work, Active, DispatchPending/Running/WaitingQuota; `I:state/managed_binding/binding.rs:158–181`), the registered owner with a non-closed invocation (`I:140–156`), the exact invocation (`I:183–239`), the exact live Session (`I:55–115, 404–408`) and negative identities. A genuine terminal legitimately advances the Unit, invocation, Session, readiness and admission rows (`I:state/execution/native_phase/terminal.rs:472–598, 700–748`) and the settlement revokes the owner (`I:execution/native/phase_protocol.rs:619`); quota and live-quota writers can also advance the Unit before the terminal. After such progress the normal write predicate can never hold again. Confirmation therefore does not re-evaluate it.

`confirm_managed_binding` runs one read-only Immediate (no write) with this **lineage confirmation** predicate, identical for normal and late plans:
1. **Unchanged parents and authority (never relaxed).** `marker.validate_open_tx`; `original.before.validate_projection` against the current Workflow (exact P/G/T, Task = marker `task_after`, locks, latest Context = original); `marker.validate_driver_live_tx` (the SAME frozen post-marker Driver row and live association); the selected database path; the registered owner row equal to the plan's `owner_raw` (owner identity; the invocation-state join is dropped for confirmation only).
2. **SAME Native lineage, any legitimate lifecycle state.**
   - Unit: `validate_unit_identity(current, plan.current.unit(), false)` holds (id, scope, kind, generation, owner epoch, phase, provider, worktree, branch, base, profile digest, cookie, `created_at`, version ≥ the plan's; `I:state/managed_binding/successor.rs:123–149`), and separately `session_id` equals the plan's allocated Session (that function does not compare it). Any lifecycle, quota or terminal state produced by the Native writers listed in §7 item 3 is allowed; confirmation decides only whether the binding committed, which the Workflow and ledger in step 3 determine.
   - Session: the plan's Session immutables (id, scope, agent, provider, role, worktree, model, effort, `started_at`, `native_ref` when already returned) are unchanged. Any state transition the Native owner itself produces is allowed.
   - Invocation: identity only (id, Unit, Session, generation, epoch and input digests), any state.
   - No foreign owner, no second allocation, and no other operation on the marker.
3. **Workflow and ledger.**
   - **Known** iff the Workflow raw equals the plan's exact postimage (`plan.after.raw()`) and exactly one `session_bound` link exists for the operation with the plan's `at` and byte-equal payload. This applies equally to a committed normal or late plan.
   - **RolledBack** iff the Workflow raw equals the original post-marker raw (`original.workflow_after`) and there are zero links of any `KINDS` for the operation. This holds even after legitimate Native progress, because the binding writes the Workflow and its link in one Immediate under the writer lock; a terminal never writes the Workflow or links.
   - **Anything else is Held.** A lineage mismatch in step 2 or any parent change in step 1 is Held, never Known or RolledBack.

After RolledBack, if the owner holds a settlement, the late plan may replace the retained normal plan (§5.2 step 2); if not, the SAME normal plan is retried only while its write predicate still holds.

The fresh-plan `AlreadyBound` path (`binding.rs:331–358`) is reached only when no retained plan exists (impossible within one epoch); a mismatching `proof_source` is Held.

### 5.4 Outcome classification of every new write (C1, SC-11)

At I, the binding Immediate is begun with `transaction_with_behavior(Immediate)?` (`I:state/managed_binding/binding.rs:395–397`), so SQLITE_BUSY (the Store connection's `busy_timeout` is 5 s, `I:state/mod.rs:156`) propagates as a plain `Err`, and the start task records only Ok/Err (`I:runtime/phase_jobs.rs:789–797`). There is no typed refusal today. Every new write port (binding normal/late, claim, observed, closure) uses this classification:
- **`Conflict` (definitive, typed)** only for a deterministic refusal after which no write is possible (C2, C1-L02), exactly two kinds:
  - the selected-database-path refusal before any transaction is constructed (`I:binding.rs:386–393`);
  - a typed refusal of an in-transaction check that runs **before the first `execute`**: `validate_current_tx`, `validate_driver_live_tx`, the write-eligibility predicate, the exact owner/invocation/Session/sealed-image checks, the ledger/budget/effect predicates. The transaction is dropped without any write, so no commit is possible.
- **`Err` (uncertain)** for begin errors (including BUSY), an error after the first `execute`, and commit errors. The SAME retained plan then goes to confirmation (§5.3, §11).
- A planning error outside Store retains no plan and is a refusal of that turn (not a write outcome).

Genuine reachable `Conflict` sources for a normal binding are legitimate writers that advance currency between plan and write: the Native owner advancing the Session record (Starting→Running, PID or `native_ref`; `I:binding.rs:404–408`), a Unit lifecycle or quota transition or the terminal itself (exact Unit index, `I:successor.rs:389–391`), or a Task/Goal/Context parent change through an existing legitimate writer (`validate_projection`). SQLite writer-lock contention is **not** a `Conflict`; it is uncertain.

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

`continue_settled_executor` dispatches on the Workflow attempt state and the retained stage. It performs at most one **Workflow/ledger-advancing** write per call (bind, claim, observed or closure). The capture call additionally writes its helper journal rows (one Pending insert and one reconcile transaction per helper, §8), one `stage_result` insert, one `ready_result` update with dependency rows, a durable manifest file with fsync, and the gate's `Verification` record; these are not Workflow/ledger advances and are counted in §8 and §12 (C1, SC-08). Every await happens before any Store guard:

| Attempt / stage | Action |
| --- | --- |
| Running and bound | (a) capture (§8); (b) `inputs` → captured `SourceSnapshot`; (c) drift policy: class escalation, rules change or `!same_sources` for a non-target-producing phase → Held with no write (invalidation of a bound open phase needs its own typed port); (d) plan and write `gate_claim` (§9.1) |
| Evaluating, claim acknowledged, no evaluation started | Set `evaluation_started` under the stage mutex, then `evaluate_settled` (§9.2). Retain `SettledGateCompletion`, then plan and write `gate_observed` |
| Evaluating, evaluation started, no retained outcome | Held: "gate outcome unknown; explicit recovery". No re-evaluation |
| Evaluating with a Passed observation acknowledged | `settled_publication`, then `prepare_pack(next)`, then plan and materialize; then `admit_success` (§4), one Store turn that closes and publishes the Driver (§10), and release |
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
3. `c.settled.settlement().validate_terminal_unit_tx(tx)`: the stored current Unit row equals the terminal's sealed `unit_after` raw exactly (raw and indexed columns; C1, SC-05/SC-13). Under Store only the exact stored Unit bytes are compared; the decoded `cleanup` overlay is not part of that row. At I the `execution_units` writers are: pre-terminal Native writers (preparation, registration, ACK, quota and live quota: `I:state/runtime/driver/preparation.rs:688`, `state/execution/native_phase/transport.rs:404`, `native_phase.rs:1423`, `native_phase/quota.rs:1242`, `native_phase/live_quota.rs:339`), the terminal itself (`I:native_phase/terminal.rs:73`), the pre-marker first-Executor preparation (`I:state/runtime/driver/executor.rs:224`), RN non-success closure (`I:native_phase/version/closure.rs:167`, which requires an unbound operation) and the generic `write_unit` (`I:state/execution.rs:318`, reached only through generic authority that refuses a marker-bound Driver). None of them can legitimately change a settled, bound, marker-bound Unit before closure W1, so a mismatch is drift: Held. No replan loop is needed; the earlier "≤3 replans" rule is removed.
4. Finalization open, effects closed, `session_id` = allocated.

Exact P/G bodies subsume the generic parent-activity and governing-digest checks. `SettledCurrency` is planned query-only once per stage outside the Store mutex and reused; each protected Store call and each fence tick runs the exact recheck above (no decode, encode or hash). The number of rechecks is therefore the number of protected calls plus fence ticks, counted in §8 and §12 (C1, SC-08; the earlier "≤14 helpers, ≤15 snapshots" bound was false).

**Consumer mapping.** Generic consumers stay unchanged and keep refusing marker-bound rows. Every protected consumer below is new and uses `validate_settled_tx`. The table is the complete inventory of generic currency checks reached on the success path at I (C1, SC-02/SC-03):

| # | Generic consumer (refuses marker) | Protected PR1 consumer |
| --- | --- | --- |
| G1 | `UnitGit::run_command` pre-spawn `validate_execution` (`I:git_io.rs:169`) | `UnitGit::for_settled`: `validate_settled_helper` under the Store mutex |
| G2 | `reserve_execution_helper_pinned` → `validate_authority` (`I:effects.rs:61`) | `reserve_settled_helper` (same row shape and kind; `validate_settled_tx` instead of `validate_authority`) |
| G3 | Running-helper fence in `capture_scoped_pinned` → `validate_execution(native, !native)` every 50 ms, first tick immediate (`I:process.rs:357–380`) | `capture_scoped_with(.., HelperCurrency::Settled(c))`: the SAME interval, immediate first tick, pinned identity comparison and kill-on-Err drop, with `validate_settled_helper` instead of `validate_execution`. The fence is preserved, not removed. A fence refusal signals the group, journals the effect `Unknown` through the unchanged `reconcile_managed_effect_pinned` path, fails the capture, and holds the stage; closure then refuses on the unsettled effect (§10.2) |
| G4 | `ResultStore::capture` → `validate_execution` (`I:results.rs:214`) and `stage_result` → `validate_authority` (`I:artifacts.rs:231`) | `capture_settled` → `stage_settled_result`; `ready_result` (no authority check, `I:artifacts.rs:244`) unchanged |
| G5 | `workflow_publication` → `validate_execution` before (`I:results.rs:399`) and after (`I:421`) `UnitGit::new` + `verify_inner(Current)` | `settled_publication`: protected validator before and after, `UnitGit::for_settled` (G1–G3 per helper) + `verify_inner(Current)` |
| G6 | `ManagedWorkflowGates::terminal` → `validate_execution` (`I:workflow_gates.rs:168`) | `settled_terminal`: the same predicates sourced from the settlement and exact rows |
| G7 | `evaluate`'s final Unit recheck `validate_execution(&unit.authority(), false, true)` compared with the checked Unit (`I:workflow_gates.rs:364–370`) | `evaluate_settled`: `validate_settled_tx` plus the same comparison of the stored Unit row with the checked Unit (§9.2) |
| G8 | `frame()` capture branch; its prepared-only branch `validate_execution(…, true, true)` (`I:workflow_source.rs:914–918`) is not reached once an artifact exists | `capture_settled` (same slot serialization; requires the slot's `handoff` custody allocation = settlement allocation) |
| G9 | `publish_workflow_result_tx` → `validate_authority` (`I:artifacts.rs:401`) | `publish_result_core` inside `close_phase_success` (§10) |
| — | Generic Driver `validate` / Source `validate_row` / `after_write` | Unused on the protected path. Source7 present at the marker → success closure Held (§3.2) |

Retained readers that need no replacement: `ResultStore::verify` / `RetainedGit` use `validate_retained_inspection` (epoch, artifact, Unit kind; no Driver check, `I:artifacts.rs:5–41`) and their own 50 ms fence (`I:retained_io.rs:60–70, 153–160`); `put_record` for the `Verification` receipt has no authority check (`I:workflow_gates.rs:389`).

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

### 8.1 Actual command catalogue and effect rows (C1, SC-08)

Counted at I. "A" is the number of `.gitattributes` files and "B" the number of eligible corpus blobs in the captured tree.

| Step | Commands (effect kind) | Cardinality and existing bound |
| --- | --- | --- |
| `frame()` HEAD | `rev-parse HEAD` (`git_helper`) | 1 (`I:workflow_source.rs:845–848`) |
| Source corpus | `ls-tree` + one `cat-file` per ordinary blob ≤256 KiB (`git_helper`) | 1 + B; B ≤ 4096 (the `entries.len() < 4096` check runs before each insertion, `I:workflow_source.rs:92–94`, so 4096 entries are admitted; C2, C1-L01); summed blob bytes ≤16 MiB (`I:1242`); zero-size blobs do not consume the byte budget |
| Ownership | 8 commands (`git_helper`) | 8, fixed (`I:git_io.rs:249–306`) |
| Capture checks | object format, `merge-base --is-ancestor`, `^{commit}` | 3 (`I:results.rs:235–265`) |
| Content scans | `ls-tree -z`, LFS pattern scan, `ls-tree --name-only`, + one `show` per `.gitattributes` | 3 + A; A ≤ 4096 (same tree as the corpus) and each output ≤4 MiB (`I:process.rs:324`) (`I:results.rs:777–810`) |
| `results.git` | optional `init`, format, fetch, `fsck --full --strict`, two `rev-list --missing=error` | 5 or 6 (`I:results.rs:293–346`) |
| Historical verify after capture | 5 (`retained_git`, recorded on the SAME Unit) | 5 (`I:results.rs:430–500`; `I:artifacts.rs:91–108`) |
| Gate `verify` | 5 (`retained_git`) | 5 (`I:workflow_gates.rs:339`) |
| Gate source recapture (`frame()` re-verify) | 5 (`retained_git`) | 5 (`I:workflow_source.rs:873`) |
| `settled_publication` `verify_inner(Current)` | 5 (`git_helper`) | 5 |

One successful Implement PR1 run therefore writes **N = 41 or 42 + A + B** `managed_effects` rows on the Unit (≤ 42 + 4096 + 4096 = **8234** at the existing corpus and tree bounds), each with one Pending insert and one reconcile transaction. Of these, the 15 `retained_git` rows are written by `RetainedGit`, which runs its own 50 ms retained fence (`I:execution/retained_io.rs:60–70, 153–160`) and is not routed through G1–G3. Every `git_helper` reaches G1–G3 (§7); each fence tick is one exact recheck. A fenced `git_helper` future lasts at most the 60 s collection timeout plus the ≤10 s `stop_and_reap` wait inside the same future (`I:process.rs:268–275, 329–341`), so it runs ≤ 1 + 70 s / 50 ms = **1401** fence rechecks. `reserve_execution_helper_pinned` has no per-Unit row-count gate (`I:effects.rs:41–89`), and the native admission gate `count < 256` over `LIMIT 257` (`I:native_phase.rs:1346–1351`) counts every `managed_effects` row of the Unit but gates only Native effect admission, which is already closed for a settled Unit. PR1 adds **no new cap**: the existing corpus/tree bounds already bound N for one run, and PR1 never re-captures within a stage (a capture failure holds the stage). A repository whose catalogue exceeds an existing bound is refused by that existing bound and the stage is Held, truthfully; PR1 does not guarantee that every maximal repository closes.

## 9. Gate claim, evaluation and observed (P)

### 9.1 `gate_claim`

- **Workflow delta.** From the bound endpoint, only the active attempt changes: `state` Running → Evaluating and `claimed_observations = observations.len()`. Record `version+1` and `updated_at=at`. A typed roundtrip and neutralized byte-equality are checked as in RN's `workflow_delta`.
- **Nothing else.** No Task, blocker, Session, source or Context write.
- **Link payload.** The trigger-required header plus `{claimed_observations, gate_phase, closure_headroom_bytes}`.
- **Immediate.** `validate_settled_tx`, then the budget: `charged + growth + link + SUCCESS_CLOSURE_HEADROOM ≤ WORKFLOW_BYTES` (§12). Then exact permits for the `records` UPDATE and the `audit` INSERT, mirroring the binder (`binding.rs:435–462`).
- **Single pair.** One pair per PR1 run. The 100th claim refuses by trigger, but PR1 never re-claims.

### 9.2 `evaluate_settled`

It is `evaluate` with four substitutions (C1, SC-03 adds the fourth):
1. The claim check is `Self::claim` (reads only).
2. Implement uses `settled_terminal`, which applies the same predicates as `terminal` (`I:workflow_gates.rs:157–201`) from the SAME settlement: `attempt.execution`/`unit`/`agent`/`session_id`; Exited; Success; WorkKnown; finalization open; effects closed. Decoded values are compared with the `cleanup` overlay excluded, and the stored Unit row is compared exactly with `validate_terminal_unit_tx` (SC-13).
3. The source recapture is `capture_settled` and must equal the invocation sources.
4. The final Unit recheck after historical verification and source recapture (`I:workflow_gates.rs:364–370`, generic G7) becomes `validate_settled_tx` under the Store mutex, whose item 3 compares the stored Unit raw and index (`unit_index_matches`) with the sealed terminal raw, the same raw step 2 checked. Today's decoded serde comparison (`I:workflow_gates.rs:364–369`) is not reused, because the decoded value includes the `cleanup` overlay (`I:state/execution.rs:249–264`) and a cleanup observation arriving between steps 2 and 4 would otherwise produce a spurious mismatch. A mismatch is drift: `evaluate_settled` returns a completion with `SettledGateOutcome::Unknown` and the stage records `gate_unknown` Held through `gate_observed` (§9.3), never Passed.

Everything else is unchanged: the Ready artifact = the invocation artifact; `ResultStore::verify` (retained reader, no replacement needed); the exact claim recheck (`I:workflow_gates.rs:360–363`); the `Verification` receipt via `put_record`; `Evidence.session_id = settlement Session`. Requirements and every non-supported phase return the existing `Waiting`. No generic `validate_execution`, `validate_authority` or Driver `validate` call remains on the Implement settled path (§7 G6, G7).

`SettledGateCompletion` is constructed only here. It is retained in the stage before any write. **Error transport (C2, C1-M05).** `evaluate_settled` first runs the claim check (step 1); a failure there returns `Err` and no completion, so the stage stays "evaluation started, no retained outcome" → Held (§6.2). Every failure after the claim check passed (terminal predicates, `verify`, source recapture, the final Unit recheck, the `Verification` receipt write) returns `Ok(SettledGateCompletion { outcome: SettledGateOutcome::Unknown, receipt: None, claim })`; the raw error goes only to bounded in-memory attention. A gate `Passed`/`Waiting`/`Failed` result returns `Known(outcome)`. `plan_settled_gate_observed` maps `Unknown` to the §9.3 `Err (unknown)` row; no caller outside the producer can construct either variant.

### 9.3 `gate_observed` (fused, one link)

- **Workflow delta.** Append exactly one `GateObservation { sources: authority_only(source), outcome, error, at }`, encoded at most 64 KiB (refused otherwise, never truncated). Then apply the disposition:

  | Outcome | Attempt after | Workflow-level | Link `outcome`, `reason_code` |
  | --- | --- | --- | --- |
  | Passed(evidence) | stays Evaluating | — | `passed`, gate receipt id, evidence SHA-256 |
  | Waiting(fixed string) | Waiting, `detail` = that compile-time string (≤128 B) | — | `waiting`, `evidence_integration_unavailable` |
  | Failed(_) | Waiting, `detail` = `"gate failed; bound non-success closure unavailable"` | `held_reason` = same constant | `held`, `gate_failed` |
  | `SettledGateOutcome::Unknown` (evaluation error after the claim check) | Waiting, `detail` = `"gate outcome unknown; explicit recovery required"`; `error` = that constant | `held_reason` = same | `held`, `gate_unknown` |

- **No raw text.** Raw error and gate text never reach durable rows or payloads; they go only to bounded in-memory attention.
- **No Task writes.** No Task `WaitingHuman` or blocker is written. Waiting and Held status are derived from the Workflow for the Driver, Goal and CLI through the consumer below.
- **Trigger.** The trigger requires the immediately preceding `gate_claim`.

#### 9.3.1 Status consumer (C1, SC-10 / SC-14)

At I, Goal status counts and task pages derive Task state through `state/runtime/waiting.rs` (`observe`, `I:20–60`; `effective_state`, `I:132–145`), whose reader requires `native_effects_open=1`, a `driving` Driver and `start_ended=0 AND known_terminal=0`; after an owned success it returns generic Held, and `effective_state` maps only Quota/Capacity, otherwise returning stored Task state. `TaskFacts { scope, version, state, phase }` (`I:runtime/control.rs:158–165`, `deny_unknown_fields`) has no field for the Workflow attempt `detail` or `held_reason`, and the CLI prints only those fields (`I:cli/goal_facts.rs:174–197`). The Driver alone reports Workflow Waiting (`I:workflow/driven_initial.rs:194–207`).

PR1 adds one bounded, read-only, nongrant consumer:
- **Reader.** `waiting::workflow_wait(conn, task)` reads the Task's single Workflow record row (`records … kind='workflow' LIMIT 2`, exactly one required) and extracts, with `json_extract` only (it never decodes the whole body; observations can reach 64 KiB each), the active attempt index, that attempt's `state`/`detail`, the Workflow-level `held_reason` and `finished`. Two exclusive branches:
  - **Open phase** (the Task's latest managed operation has `phase_open=1`): report a wait only if the active attempt's state is `Waiting` or `held_reason` is set; Running, Evaluating or any other non-waiting state reports **no** wait (`None`), never `Held`.
  - **Post-closure** (C2, C1-M02; the durable state §6.3 produces): the latest managed operation has `phase_open=0` and exactly one `phase_closed` link whose payload has `closure="success"`; the Workflow has `active=None` and `finished=false`; the Task's `phase` equals the Workflow's next phase; and the Task's Driver row is `driving` with `marker IS NULL`. Then report `NextPhaseUnavailable` with the fixed detail `"next phase typed Driver continuation unavailable (SC-N)"`. The in-memory `StepResult::Waiting` is not read; the status is derived only from these durable rows.
  - Otherwise (no managed operation, or a state matching neither branch): `None`.
- **Mapping.** In the open branch, exact matches against the fixed §9.3 constants map to `WorkflowWaitKind::{EvidenceIntegrationUnavailable, GateFailed, GateUnknown}` with the constant as `detail`; a `Waiting` state or `held_reason` with any other or oversized value maps to `Held` with the fixed detail `"workflow held"`. Raw text is never echoed.
- **Wiring.** `runtime_goal_task_page` fills `TaskFacts.workflow_wait` (`I:state/runtime/goals.rs:574–581`); the CLI prints it after the state (`I:cli/goal_facts.rs:190`). `GoalFacts` counts keep using stored/effective state; no new count is added.
- **Cost.** ≤128 Tasks per page (`I:goals.rs:529`) × (1 indexed Workflow query with ≤5 `json_extract` + 1 indexed operation/link query + 1 indexed Driver row query); ≤ about 200 additional wire bytes per Task, inside the existing 64 KiB page packing (`GOAL_TASK_PAGE_BYTES`, `I:state/runtime/recorded.rs:13`), which already pages on overflow.
- **Never.** It writes nothing, changes no Task state, grants nothing, and is not called by the Driver or any writer.

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
3. **Unit.** The old image is the terminal's sealed `unit_after` raw (checked by `validate_terminal_unit_tx`, §7 item 3); the new image sets `result_finalization_open=false`, `artifact_id`, `version+1`, `updated_at = at` on that value. The planner builds the new body from the settlement's decoded Unit with `cleanup` taken from the terminal's value, never from the read overlay, so the overlay is not written into the body.
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

**Admission (C1, SC-06).** `close_phase_success` itself is synchronous and never awaits. Its callers hold `control_admission` for the whole turn, through one of two Runtime-owned entry points (§4):
- **Driver worker:** `InstalledDriverComposition::admit_success(lifetime)` returns a `SuccessAdmission` (owned admission guard plus strong `Arc<Runtime>`), exactly like CA's `admit_activation_inner` (`I:runtime/installation.rs:162–196`): Weak upgrade, cancel-biased `lock_owned`, `ensure!(service_running() && is_current())`, association checks. The worker then takes the synchronous SharedStore for one turn (`close_phase_success` or `confirm_phase_success`, then `publish_success_driver`), drops the Store guard, then explicitly drops the admission (CA caller pattern, `I:workflow.rs:1145–1165`).
- **Root sweep:** `Runtime::try_admit_root_success(weak)` upgrades the service task's `Weak<Runtime>` and, holding that strong Arc for the whole segment (as CA holds it across `lock_owned`, `I:runtime/installation.rs:166–175`), takes `control_admission.clone().try_lock_owned()` without awaiting (C2, C1-M01), then checks `service_running` and `!stopping`. If the admission is busy (for example, shutdown holds it while joining the service task, `I:runtime/service.rs:125–143`), nothing is attempted and the action stays pending. `PhaseDispatcher::confirm_success` runs only `confirm_phase_success` (and, if Known, `publish_success_driver`) in one synchronous turn; the admission is dropped before the next action, like `publish_planned_marker`'s single admitted segment (`I:runtime/phase_supervisor.rs:1067–1152`). The Root never commits a new closure (§14).

Then one `Immediate`:
1. The selected database path. `validate_settled_tx` against the planned endpoint (Workflow = the `gate_observed` postimage; ledger head = that link).
2. The Unit, artifact (Ready) and Task (the marker `task_after` raw) rows are exact. The latest Context version = the original, and no row exists at the next version.
3. **Effects (C1, SC-08).** `NOT EXISTS (SELECT 1 FROM managed_effects WHERE unit_id=?1 AND state NOT IN ('confirmed','resolved') LIMIT 1)` over every effect kind of the Unit (the state domain is `pending|confirmed|resolved|unknown`, `I:state/execution/native_phase/version.rs:22`, so this means no Pending and no Unknown row), the same shape as the existing predicates at `I:state/runtime/driver/executor.rs:219` and `I:state/execution.rs:685`. It is one indexed existence probe, not a bounded scan, so it is correct for any N (§8.1) and never truncates.
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

Still holding the Store guard and the admission of the same turn, run `publish_success_driver`: an exact committed-row read, then the SAME association's `publish_exact`. It returns the `SuccessClosureAcknowledgment` with `driver_published` = the outcome.

- **On failure:** the acknowledgment and the plan are retained. A later turn re-runs only `publish_success_driver` (idempotent per `publish_exact`), each attempt under a fresh admission: the SAME live worker through `admit_success`, or the Root sweep through `Runtime::try_admit_root_success` plus `PhaseDispatcher::retry_success_publication` (one non-blocking admission, one Store turn, release; never held across a backoff, like the marker rollback's separate re-acquisition at `I:phase_supervisor.rs:1161`).
- **After an actual worker exit:** Held. Nothing is revived, and no cache is published from rows.

### 10.5 Acknowledgment consumers

Order (all pointer-checked; removals are retried without a DB write):
1. Install `JobState.closed_ack = ClosedPhaseAck::Success` and send `ClosedSuccess`.
2. Driver published (§10.4).
3. The Driver step's Sources-slot `retire_closed_handoff` sets `sources_retired`.
4. Root: `retire_closed_marked` (PhaseSupervisor slot), then `PhaseHandoffs::retire_closed`, then `retire_closed` (PhaseJobs entry).

Until step 4 finishes, `ensure_*_shutdown_complete` keeps failing, as today. The Native registry phase Entry is **not** released (§18).

### 10.6 Confirmation of claim, observed and closure (C2, C1-H01)

`validate_settled_tx` (§7) checks an **open** phase at one expected endpoint. A committed write moves that endpoint (claim and observed advance the Workflow and ledger head; closure also closes the operation, advances Task, Context, Unit, artifact and Driver, and clears the marker). Confirmation therefore never applies `validate_settled_tx` alone. Each `confirm_*` runs one read-only Immediate (no write):

1. **Common immutable authority (never relaxed).** The selected database path; exact Project and Goal rows (no PR1 write touches them); locks; the SAME live Driver association object (`same_association`, not the Driver row bytes); the original input, owner and epoch identities; the sealed terminal images of the settlement (§5.1). Any failure is Held.
2. **Postimage branch (Known).**
   - Claim / observed: the Workflow raw equals the plan's postimage; the ledger head is exactly the plan's link (kind, `at`, byte-equal data); the open-phase conjuncts of `validate_settled_tx` hold **at that post endpoint** (operation open, Task = marker `task_after`, latest Context = original, stored Unit = sealed terminal raw, frozen post-marker Driver row).
   - Closure: every W1–W8 row equals the plan's postimage (Unit, artifact Published, Task, the new Context version, operation `phase_open=0` version 2, Workflow, the `phase_closed` link with the plan's `at`/data as the ledger head, the marker-free Driver row), using the plan's retained images, the same way RN classifies its closed image (`RN:state/execution/native_phase/nonsuccess.rs:194–206`, `PhaseImage::Closed`).
3. **Preimage branch (RolledBack).** Every row equals the plan's preimage and the planned link is absent: for claim/observed, `validate_settled_tx` at the pre endpoint (prior Workflow and prior ledger head); for closure, `validate_settled_tx` at the `gate_observed` endpoint, the Ready artifact, the marker Task, no Context row at the next version, operation `phase_open=1`, and the frozen post-marker Driver row.
4. **Anything else is Held**, including a mix of post- and preimages, a link without its postimages, or postimages without the link.

SC4's after-commit variants (claim, observed, closure) and the Root closure confirmation (§5.2 D3) are verified against this contract.

## 11. Uncertain commits, repeated delivery, cancellation and stop (P)

- **Uniform uncertainty rule.** Every new write (binding, claim, observed, closure) returns `Known`, typed `Conflict` or `Err` (uncertain), classified as in §5.4. After `Err`, only the SAME retained plan is confirmed. Binding uses the lineage predicate of §5.3; claim, observed and closure use the two-branch confirmation of §10.6 (C2, C1-H01), never the open-phase validator alone:
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
- **Stop.** Shutdown and `Runtime::drop` revoke the worker, so `validate_driver_live_tx` refuses later writes. Every closure, closure-confirmation and publication turn holds a `SuccessAdmission` (worker or Root), which retains both the admission guard and a strong `Arc<Runtime>`, across commit and Driver publication; shutdown acquires `control_admission` with its existing 5 s timeout (`I:runtime/service.rs:126–128`), so stop and commit linearize as in CA §5.7. Because `Runtime::drop` takes no admission (`I:runtime/mod.rs:130–135`), it is the strong `Arc<Runtime>` inside every `SuccessAdmission`, held only for the admitted synchronous segment, that prevents drop from running inside it; no strong Runtime is held across a Git or gate await (`I:runtime/installation.rs:229` discipline).
- **Restart.** A new epoch never reconstructs `SettledPhase` or plans. Open operations stay Held; closed ones stay closed. RestoreFactualBinding is absent, and nothing is inferred from timeouts.

## 12. Finite costs (P)

All figures are encoded lengths taken from existing bounds. They are not heap or RSS bounds, and control M1 measures them. There is no truncation: an over-bound value refuses (Held).

| Transaction | Mandatory surfaces (each existing bound) |
| --- | --- |
| Late bind | The binder's surfaces (P/G/T ≤3×8 MiB, Workflow ≤8 MiB old/new, locks ≤256×16 KiB, Context ≤8 MiB stored-decoder bound — an admitted original Context's data is a ≤1 MiB payload in a ≤2 MiB frame, `I:execution/phase.rs:118, 206–232`, `I:execution/native.rs:305–306`, own Session ≤4 MiB, negative identities ≤4096×16 KiB, owner ≤32 KiB, invocation ≤`INVOCATION_BYTES` = 64 KiB) plus the sealed terminal images (invocation, receipt ≤`RECEIPT_BYTES`, Session ≤4 MiB, owner ≤32 KiB, admission ≤8192 B, readiness ≤4096 B); one link ≤4096 B |
| Settled helper | Per helper: one exact recheck under the Store mutex before spawn (G1), one `managed_effects` insert ≤8 KiB with its recheck (G2), one recheck per 50 ms fence tick including the immediate first tick (G3, bounded by the 60 s collection timeout plus the ≤10 s reap: ≤1401 ticks), and one reconcile transaction. Per PR1 run: N = 41 or 42 + A + B effect rows, ≤8234 (§8.1); the 15 `retained_git` rows use the retained fence instead of G1–G3 |
| `gate_claim` | Projection set + Workflow old/new ≤2×8 MiB + link + `charged_scope_bytes` aggregates |
| `gate_observed` | as claim + observation ≤64 KiB |
| Closure | Projection set; Workflow old/new ≤2×8 MiB; Task old/new ≤2×1 MiB; Context new ≤8 MiB; artifact ≤2×128 KiB; Unit ≤2×44 KiB; operation ≤2×4 MiB; Driver ≤2×128 KiB; one `NOT EXISTS` effect probe (no row images, §10.2); link ≤4096 B; four permit rows |
| Driver publication | one Driver row ≤128 KiB |

- **Mandatory closure reservation.** `SUCCESS_CLOSURE_HEADROOM` = 8 MiB (Context) + 8 MiB (maximum Workflow postimage) + 1 MiB (Task) + 512 KiB (artifact, Unit, Driver, receipt record) = 17.5 MiB. `gate_claim` refuses unless the headroom fits. This is in addition to the existing link reserve (1 MiB − spent while `phase_open=1`), which closure releases by derivation.
- **Ledger.** PR1 spends 4 links (bind, claim, observed, closure) of 256. Diagnostics and hold allowances are untouched, and the `phase_closed` slot is never consumed early.
- **Retained memory (C1, SC-09).** Encoded bytes of owned copies, at the existing bounds (`BODY_BYTES` 8 MiB, `I:state/managed_binding/canonical.rs:12`; `SESSION_BYTES` 4 MiB, `I:state/managed_binding/schema.rs:29`; `OWNER_BYTES` 32 KiB; `INVOCATION_BYTES` 64 KiB, `I:execution/native_result.rs:10`; `UNIT_BYTES` 16 KiB, `I:successor.rs:21`). `Body<T>` owns both its raw and its canonical encoding plus the parsed value (`I:canonical.rs:18–37`); parsed values have no byte bound and are heap only.
  - **Shared custody (not added by PR1):** `proof: Arc<NativePhaseBinding>` (already held by the job) and `Arc<MarkerPublicationPlan>` (marker custody: P/G/T, before-Workflow, Context, locks, `task_after`, `workflow_after`, frame and rows). This existing custody is retained per job before PR1 and is listed here only for completeness.
  - **Binding plan, additional owned copies** (`I:state/managed_binding/binding.rs:34–45, 316–376`): successor Workflow `Arc<Body<Record>>` decoded freshly (raw + canonical ≤16 MiB), current Unit (≤32 KiB), ledger head (≤4096 B + parsed), own Session (raw + canonical ≤8 MiB), `owner_raw` (≤32 KiB), invocation (raw + canonical ≤128 KiB), `after` Workflow (raw + canonical ≤16 MiB), `audit_data` (≤4096 B), and `record_mutation` with full old/new raw images (≤16 MiB plus small columns; `I:permits.rs:185–190`, `I:marker_rows.rs:372–381`). Total ≈ **56.2 MiB** per plan, plus parsed values. `copy_for_transaction` (`I:permits.rs:195–201`) adds ≈16 MiB transiently during each write attempt.
  - **Success stage:** compact plans plus at most ONE closure Context raw (≤8 MiB) and the observation (≤64 KiB): ≈ 8.06 MiB. `SettledTerminalImages` (§4): ≤ about 6.3 MiB (receipt raw ≤`RECEIPT_BYTES` = 2 MiB, `I:execution/native_result.rs:11`; Session ≤4 MiB; invocation ≤64 KiB; owner ≤32 KiB; Unit ≤16 KiB; admission ≤8192 B; readiness ≤4096 B).
  - **Worst case:** ≤128 jobs × (56.2 + 8.06 + 6.3) MiB ≈ **8.8 GiB** encoded, excluding pre-existing marker custody, parsed values and allocator/RSS overhead, plus ≤16 MiB transient per concurrent write attempt (writes are serialized by the Store mutex, so one at a time). The earlier "≈2.5 GiB" figure omitted the canonical encodings, the successor copy, the mutation images, the invocation, owner and Unit. PR1 prescribes no new cap; this is reported as a policy question (for example, sharing the first-bind successor Workflow with `workflow_after` would save ≈16 MiB per plan). With the #81 rule of one active Task per Project, the practical job count is bounded by the number of concurrently active Projects, but the code bound remains `MAX_JOBS` = 128 (`I:runtime/phase_jobs.rs:22`).
- **Sweep.** ≤128 jobs classified (one atomic load per D2 candidate), ≤8 turns, and per turn ≤1 query-only snapshot plus ≤1 Store transaction or one admitted action.

## 13. Locks and lifetime (P)

| Held | May take |
| --- | --- |
| `PhaseJobs.entries` | nothing new (existing `start` only) |
| `job.state`, `SuccessContinuation.stage`, `success_cursor` | nothing (leaves) |
| `PhaseHandoffs.entries` | `slot.handoff` → `assets` (existing order) |
| Sources slot (tokio) | SharedStore briefly, never across an await (existing `frame()` discipline); Git awaits hold no Store |
| `control_admission` (held only as a `SuccessAdmission`: the worker's awaited, cancel-biased `admit_success` or the Root's non-blocking `try_admit_root_success`; never across a sleep, backoff, Git or gate await) | SharedStore (synchronous), then the permit manager, then the association `binding` mutex |
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
| `bind_managed_phase` return type; `ManagedBindingPlan` kind | `I:runtime/phase_jobs.rs:887–905` (only caller) | Mapped to `Known`/typed `Conflict`/uncertain (§5.4); the normal write predicate is unchanged; confirmation is the separate lineage predicate (§5.3) |
| `NativeTerminalCommit`/`into_parts`, `PhaseActor::settled`, `OwnedPhaseSettlement::completed` gain sealed `SettledTerminalImages` | `I:execution/native.rs:1294–1307` (only caller chain) | Additive argument; non-success terminals pass none; existing replay keeps the first settlement |
| `capture_scoped_pinned` becomes a wrapper over `capture_scoped_with(.., HelperCurrency)` | every existing helper caller | `Generic` arm byte-identical to today |
| `TaskFacts.workflow_wait`; `waiting::workflow_wait` | `I:state/runtime/goals.rs:574–581`; `I:cli/goal_facts.rs:190`; `I:runtime/control.rs:158–165` | Additive optional field; older clients with `deny_unknown_fields` reject it (protocol note) |
| `runtime/service.rs` loop awaits returned `SuccessAction`s | `I:runtime/service.rs:91–104` | After the synchronous sweep, through `PhaseDispatcher`; RN unchanged |
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
| `phase_open=0` | `charged_scope_bytes` reserve; `state/runtime/waiting.rs` (`phase_open=1` filter; C1, SC-14 path correction); Driver claim occupancy; quota policy | Reserve released; quota/capacity waiting status stops; `workflow_wait` reports the post-closure `NextPhaseUnavailable` while the Driver is `driving` (§9.3.1); Task occupancy remains through its `driving` Driver |
| New constants | `SUCCESS_CLOSURE_HEADROOM`, 64 KiB observation, success turns 8 (the "3 currency replans" constant is removed by C1) | PR1-local |
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
| SC3 | Definitive normal `Conflict` and retry (C1, SC-11). While the owner is live and pre-terminal, the genuine Native owner advances its own Session record (Starting→Running, or PID/`native_ref`) after `plan_managed_binding` and before `bind_managed_phase` (a cfg(test) timing seam holds the start task between plan and write; the advance itself is the real owner's write). Only a genuine Session record advance counts for this positive (C2, C1-M03): if the fixture peer's protocol produces none after the start returns, SC3 is SETUP. Assertions: the write returns typed `Conflict` from the pre-write Session check, zero links, Workflow unchanged; the sweep's later `plan_managed_binding` retry binds `normal_return`; no `closed_settlement` link exists. A separate variant holds the SQLite writer lock with a second connection across the bind: the outcome is uncertain `Err` (not `Conflict`), followed by confirmation Known or RolledBack. **SC3-P (separate negative):** a Task or Goal change through an existing legitimate writer during the same hold makes the write a typed `Conflict` and every later normal plan refuse on the original parent projection (`validate_projection`, `I:state/managed_binding/snapshot.rs:400–425`): zero links, Held, no frame refresh (MB2). |
| SC4 | Uncertain commits. A failpoint returns `Err` after commit, and separately before commit, in each of: bind, claim, observed, closure. Assertions: the confirmation is Known or RolledBack; the SAME `at`/digests; exactly one link of each kind; L-x variants (a parent drift after commit) are Held. **SC4-N (C1, SC-04):** a normal binding whose commit is reported uncertain, followed by the genuine owned terminal before confirmation (the peer released by `fixture-release`): confirmation is Known (committed variant) or RolledBack (pre-commit variant) despite the advanced Unit/invocation/Session/readiness/admission rows; in the RolledBack variant the late plan then binds once with `closed_settlement`. |
| SC5 | Lost notifications and later settlement (C1, SC-01). A seam suppresses job watch sends. (a) Binding still converges via the service timer within ⌈n/8⌉ sweeps plus backoff. (b) Normal `Bound` first, then the peer is released and settles: the D2 class discovers the settlement within ⌈n/8⌉ sweeps plus backoff and installs the continuation; no notification is used. (c) An `ordinary-failure` peer after `Bound`: no continuation is installed, the job is re-polled at most once per 5 s (observed through the cfg(test) turn counter), it consumes no Store transaction, and it reports the attention text. (d) A cfg(test) seam pauses `completed` between `revoke` and setting the settlement Weak while the sweep runs: the sweep records no conclusion, and after release the continuation is installed. |
| SC6 | STANDARD Requirements. Bind → capture (Ready) → claim → observed Waiting (`evidence_integration_unavailable`). After ≥50 Driver polls: still one claim, no `phase_closed`, Task unchanged, Requirements not completed, artifact not Published. |
| SC7 | Negatives after genuine setup: <br>• Goal or Task edited through an existing legitimate writer during the held start → Conflict/Held and zero links; <br>• `ordinary-failure` scenario → no settlement, Held "bound non-success closure unavailable"; <br>• peer killed before terminal → Unknown/Lost → no late bind; <br>• Runtime restart → Held and no reconstruction. |
| SC8 | Graph integrity. Between capture and closure, a negative-only corruption of a `results.git` object → `settled_publication` refuses and closure is never committed. |
| SC9 | Stop (C1, SC-06). (a) Shutdown while the worker awaits `admit_success` → the biased select returns cancelled, no commit. (b) Shutdown acquires `control_admission` first and then joins the service task while the Root has a pending closure-confirmation or publication action: `try_admit_root_success` returns None without awaiting, the service loop observes `stopping` and exits, and shutdown completes within its join bound without relying on the 5 s timeout (C2, C1-M01); no confirmation write or publication occurs. After the Runtime is dropped the Weak upgrade refuses. (c) Shutdown after commit with publication pending → Held, counted pending, no revival; `retry_success_publication` refuses once stopping. (d) Shutdown's 5 s admission acquisition cannot interleave inside an admitted closure turn (commit and publication observed together or not at all). |
| SC10 | Four Tasks in **four distinct Projects** (two Claude, two Codex; one active Task per Project, per Issue #43 / #81) in commit mode → four independent closures, with no cross-job acknowledgment or Driver publication. |
| SC11 | Protected helper fence (C1, SC-02). A genuine protected Git helper in capture is held by a cfg(test) timing seam for ≥3 fence ticks: it completes and its effect is Confirmed (the fence ran `validate_settled_helper`, not the generic validator). Negative: a parent Goal edit through an existing legitimate writer while the helper is held → the next tick refuses, the group is signalled, the effect is `Unknown`, capture fails, the stage is Held, and closure never commits. |
| SC12 | Status visibility (C1, SC-10). SC6's Requirements Waiting is reported by `GoalTasks` as `workflow_wait = {evidence_integration_unavailable, <constant>}` and printed by the CLI; SC1 after closure reports `next_phase_unavailable`; Task state and blockers are unchanged in both. |
| M1a | Mechanical representation measurement only, **not a positive** (C1, SC-12): encode/decode at the `BODY_BYTES`/`SESSION_BYTES` limits for Workflow, Session and a closure Context, and report owned bytes for every §12 retained-memory term. No Native allocation is involved and no control credit is earned. |
| M1b | Reachable positive (C1, SC-12): an original Context whose data is a ≤1 MiB payload in a ≤2 MiB frame (the admitted input limits, `I:execution/phase.rs:118, 206–232`), a closure Context as large as the genuine `prepare_pack` writer produces, and genuine Workflow/lock sizes; report measured retained and per-turn peaks against §12. An original Context above the input limits stops before marker/binding and is SETUP, not coverage. |
| M1c | Headroom boundary (C1, SC-12), counted only if genuine: before Goal acceptance, legitimate `Store::put_context` history in the Task scope (no size cap before acceptance, `I:state/mod.rs:1443–1462`) raises `charged_scope_bytes` into the band (128 MiB − 17.5 MiB, 128 MiB − marker extras]; then accept, prepare, mark and bind with a final ≤1 MiB-payload Context. Assertion: `gate_claim` refuses on headroom with zero claim links. If acceptance, preparation or another reader refuses large history first, M1c is recorded as SETUP and the headroom check is defense-only. |

| Mutant (compiled) | Killed by (first assertion) |
| --- | --- |
| Restore the Task write in binding, claim or observed | SC1: Task.version unchanged before closure |
| Late planner accepts settlement None or a live owner | **defense only** for the production caller (C1, SC-11): the sweep calls `plan_late_binding` only when the snapshot settlement is Some (§5.2 step 2), so this predicate is unreachable from production. A direct planner unit test asserts the refusal for a genuine pre-terminal proof; it earns predicate-level credit only and never credits a committed `closed_settlement` link |
| Late planner skips `validate_terminal_images_tx` | defense only (the sealed images come from the SAME terminal commit; no genuine producer of a mismatch other than the post-exit Session writer limit in §5.1) |
| D2 class removed from the due predicate (C1, SC-01) | SC5(b): no continuation installed after normal Bound then settlement |
| 5 s backoff for a revoked owner without settlement omitted | SC5(c): the job is snapshotted every sweep (turn counter observation) |
| Revoked-without-settlement concluded as permanent non-success | SC5(d): no continuation after the paused settlement is released |
| Confirmation reuses the normal write predicate (C1, SC-04) | SC4-N: Held instead of Known/RolledBack after genuine terminal progress |
| Confirmation drops the parent projection check | SC4 L-x: Known after a parent drift |
| Lock contention mapped to `Conflict` (C1, SC-11) | SC3 lock variant: plan dropped and replanned instead of confirmed |
| Settled fence uses the generic `validate_execution` (C1, SC-02) | SC11: helper refused on the first tick, effect `Unknown`, no Ready artifact |
| Settled fence removed entirely | SC11 negative: helper completes Confirmed after the parent edit |
| `evaluate_settled` final recheck uses the generic `validate_execution` (C1, SC-03) | SC1: gate refuses ("marker-bound Driver…"), observed `gate_unknown`, no closure |
| Closure without admission (C1, SC-06) | SC9(d): commit observed without publication across a shutdown boundary |
| Closure effect probe accepts Unknown (C1, SC-08) | **defense only** (C2, C1-M04): SC11's negative stops at the fence and capture, before any Passed observation, and no genuine producer leaves an `Unknown` effect on the Unit after a Passed observation while closure is still reachable (a `settled_publication` helper failure fails publication before closure). No consumer kill is claimed |
| `workflow_wait` reader removed (C1, SC-10) | SC12: no `workflow_wait` in `GoalTasks` |
| Confirm accepts a preimage as Known (any port) | SC4 pre-commit: acknowledgment while the link is absent |
| Replan with a new `at` after uncertainty | SC4: link `at` or digest differs from the SAME plan |
| Root `reconcile_success` call removed | SC2: no binding |
| Root admission awaits `lock_owned` instead of `try_lock_owned` (C2, C1-M01) | SC9(b): shutdown join waits until its timeout |
| Closure confirmation uses `validate_settled_tx` alone (C2, C1-H01) | SC4 closure after-commit variant: Held instead of Known |
| `evaluate_settled` returns `Err` instead of an `Unknown` completion after the claim check (C2, C1-M05) | a cfg(test) fault in the gate's `verify` after the claim: no `gate_observed` with `gate_unknown` is written |
| Capture uses the generic `validate_execution` | SC1: capture refusal ("marker-bound Driver…"), no Ready artifact |
| Protected validator omits `validate_driver_live_tx` | SC9 variant: a helper intent row after revocation |
| Gate Waiting rewrites Task WaitingHuman/blocker | SC6: Task changed |
| Automatic re-claim after Waiting | SC6: second `gate_claim` |
| Requirements Waiting mapped to Passed | SC6: `phase_closed` present |
| Closure skips `settled_publication` graph verification | SC8: closure committed |
| Closure omits phase_open 1→0 | SC1: chain trigger refuses (trigger kill) |
| Closure omits the Driver re-anchor or its publication | SC1: generic `validate` / association `is_current` false after closure |
| Post-closure Waiting branch removed | SC1: worker exits with Err |
| Headroom check removed | M1c: claim admitted inside the band, then closure refused on budget. If M1c is SETUP, **defense only** (C1, SC-12); large single bodies alone do not establish causality |
| Exact stored-Unit comparison widened to tolerate any field (C1, SC-13) | defense only (no genuine writer changes a settled, bound, marker-bound Unit before W1; §7 item 3) |
| `same_association` check removed from lookup | defense only (one Driver per Task) |

Review checks (not mutants): no decode, hash or encode under the new Immediate ports; no await under SharedStore.

## 17. Acceptance and MVP mapping (no test was run)

| Condition | PR1 contribution | Status |
| --- | --- | --- |
| MB-AC1 | Real current callback and completion stay eligible after Bound (SC1) | Partial; requires execution |
| MB-AC2 | Existing binder plus new ports refuse drift (SC7) | Partial; full per-predicate matrix unmet |
| MB-AC3 | Late identity negatives on the same checks | Partial |
| MB-AC4 / 4.a | Normal/late race, uncertain commit (including SC4-N lineage confirmation after genuine terminal progress), lost notifications and later settlement discovery, typed Conflict versus uncertain contention, NotDispatched refusal, cleanup-Unknown late bind once | Designed (SC2–SC5, SC7); restart remains Held (unmet) |
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

1. After design approval, the implementer (assigned by the user; at C1 time the Claude Code development session on `feature/issue-43-native-nonsuccess-controls`) implements on a composed commit that contains I and has passed its own separate composition review (§1 item 8), or on a successor that satisfies the same condition (C2, C1-L03). I itself is unreviewed and is not a starting point by ancestry alone. Each item is committed separately:
   - (a) binding write/confirm types (typed Conflict, lineage confirmation), sealed `SettledTerminalImages`, and the late planner;
   - (b) Root sweep (D1–D3 and the 5 s re-poll), `SettledPhase`, lookup and the admitted Root actions;
   - (c) protected currency, `HelperCurrency` fence, helpers and capture;
   - (d) claim, evaluate and observed;
   - (e) closure, Driver advance and publication;
   - (f) releases, Driver branches and the `workflow_wait` status consumer;
   - (g) seams, controls and mutants;
   - (h) master updates (`master/agent-execution.md`, `master/workflow-engine.md`) with implemented facts only, in the same PR.
2. This HOW: ONE independent non-author Sol 6.1 (`gpt-6.1-sol`) high review, orchestrated by Root after a full read of the frozen authored commit. Re-review covers only the finding side's delta. C1 is that delta; its review is requested from the finding side (Sol, through the user's `codex-ready` label flow on Issue #43).
3. Source: three parallel high initial reviews (Sol on the implementation diff, Opus on security/contract, Grok on reconciliation). Fixes go to the non-finding side, and Grok does not return. This HOW grants no source, qualification or merge approval.
