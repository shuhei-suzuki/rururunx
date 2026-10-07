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