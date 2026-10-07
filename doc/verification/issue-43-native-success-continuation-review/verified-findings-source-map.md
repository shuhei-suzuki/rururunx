# Verified SC-01–12 source map

This report verifies literal findings against immutable objects. It neither repeats the independent review nor authorizes changes. Eleven findings contain confirmed contract, cost or control-plan corrections. SC-07 has a real API/placement omission; its mandatory status is not independently established, because legal producer-local implementation remains possible. Root owns final classification.

| Pin | Exact commit |
|---|---|
| H | `0c35653c7a8aa05c78c5285c2f4b5370db520e70` |
| R | `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` |
| CA | `95ef0e783c0b18c90dfef23ba1b77b9dc67735f1` |
| RN | `1d783440b7f4d705bf8cf4c667e022f5cec87777` |
| CA-current | `2c24f3f6e66f45692e416c77dc3e8d64fcbc9276` |
| RN-current | `e70faa9dc900c5df0dd26d9983425cf6d63876a3` |

Original review: /Users/shuheisuzuki/Documents/dev/rururunx/.rrx/development-evidence/issue43-native-success-continuation-design/sol-independent-review-result.md; 19757 bytes; SHA-256 `dc5749ceff265c831b599387ef47781bf1623f851a88cfe88401ec9a808890f7`. Frozen H doc SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6`; HEAD/status/diff-check and saved Git blob hashes verified by this writer.

## Classification and actual consumers

| Finding | Factual classification |
|---|---|
| SC-01 | confirmed_mandatory_contract_correction |
| SC-02 | confirmed_mandatory_contract_correction |
| SC-03 | confirmed_mandatory_contract_correction |
| SC-04 | confirmed_mandatory_contract_correction |
| SC-05 | confirmed_mandatory_contract_correction |
| SC-06 | confirmed_mandatory_contract_correction |
| SC-07 | verified_api_omission_mandatory_status_unverified |
| SC-08 | confirmed_mandatory_cost_and_control_correction |
| SC-09 | confirmed_mandatory_cost_correction |
| SC-10 | confirmed_mandatory_consumer_correction |
| SC-11 | confirmed_mandatory_control_plan_correction |
| SC-12 | confirmed_mandatory_control_plan_correction |

### SC-01 — Acknowledged launched jobs awaiting settlement are excluded from the due predicate.

H requires no acknowledgment or an already installed continuation for due classification. The normal start stores Launched once and binds once; subsequent terminal delivery projects into the same Native owner. It does not change JobState.outcome. Thus the timer never enters the discovery step for normal Bound followed by settlement.

Static liveness contradiction, also independently confirmed by Root. No successful continuation control was executed. Preserve bounded fair discovery and avoid observer ownership.

Evidence: `H-due`, `H-next`, `job-shape`, `job-return`, `job-bind`, `binding-snapshot`, `terminal-delivery`, `WHAT-MB7-9`, `WHAT-AC`.


### SC-02 — Protected Git helper admission does not replace the running helper currency monitor.

UnitGit awaits capture_scoped_pinned. The 50 ms timer checks Unit identity and then generic validate_execution even with driver=None. Generic Driver validation refuses a marked row. A monitor refusal is journaled Unknown and the proposed capture requires settled helpers. This is a live protected consumer absent from H’s replacement map.

Do not delete the timer/fence. No executed timing or mutant credit; actual protected currency/lifetime must be connected at this consumer.

Evidence: `H-capture`, `git-run`, `git-monitor-call`, `process-monitor`, `marker-refusal`, `WHAT-MB7-9`.


### SC-03 — The final Implement Unit check still reaches generic marker refusal.

H lists only three evaluate substitutions and preserves everything else. The existing evaluate path separately runs validate_execution(false,true) after historical verify and source recapture, then checks exact artifact and writes Verification. Replacing terminal alone leaves this check.

Preserve claim/artifact/currentness checks and connect this separate protected check; not a request for new evidence semantics.

Evidence: `H-gate`, `gate-final`, `marker-refusal`, `WHAT-AC`.


### SC-04 — Full normal write currency prevents SAME-plan confirmation after genuine Native progress.

Normal eligibility requires proof.is_live, no settlement, open effects/finalization and absent work; owner/invocation and Session images remain exact. Terminal persistence advances Unit, invocation, Session, readiness and admission. completed also revokes the owner, so NativePhaseBinding::is_live checks the changing owner state as well as its captured live boolean. H confirms under full original currency and forbids replacing an uncertain plan before rollback/refusal. A genuine terminal can therefore strand committed and rolled-back binding classification.

Confirming factual commit/rollback is distinct from issuing a new normal write. No relaxation of original parent/input/owner/lock/epoch checks is authorized. No race control executed.

Evidence: `H-due`, `H-uncertain`, `normal-session`, `normal-eligibility`, `normal-write`, `binding-access`, `binding-snapshot`, `settlement-issuer`, `terminal-progress`, `WHAT-MB7-9`, `WHAT-AC`.


### SC-05 — Existing settlement accessors cannot deliver the promised complete terminal postimages.

NativeTerminalPlan retains full invocation/readiness/admission before/after PairRow images. NativeTerminalCommit::into_parts and phase.settled deliver Unit, receipt, Session/version and settlement only. OwnedPhaseSettlement retains consumed input and thread/turn, not those complete terminal SQL postimages. Admission/readiness versions and bodies are derived from actual preimages. H says expected images come from the SAME settlement with existing accessors unchanged; that delivery is absent.

Rows read later can be compared as facts but cannot be adopted as the producer’s expected images or create a grant. No new serializable image credential or constructor widening is authorized.

Evidence: `H-late`, `H-sealed`, `terminal-shape`, `terminal-progress`, `terminal-delivery`, `settlement-shape`, `settlement-access`, `WHAT-MB7-9`.


### SC-06 — The closure/Root publication callers have no declared async admission and Runtime lifetime bridge.

H synchronously declares close_phase_success(&mut Store, material) and reconcile_success(&PhaseSupervisor,&AtomicBool), yet assigns awaited control_admission acquisition before Store and retention through publication/retries. Store contains only Connection and permits. Existing service sweep holds no admission. CA’s real async admit_activation retains OwnedMutexGuard plus strong Arc<Runtime>, with explicit drop order and association/composition checks. None of those objects are arguments/owned fields in the proposed closure call path.

This confirms an owner/caller/lifetime contract gap, not an observed stop exploit. Both Driver closure and Root publication retries need a concrete caller bridge. Preserve actual CA ordering; do not add a launch barrier or settle SourceStop/Cancel/R2 policy.

Evidence: `H-api`, `H-closure`, `H-uncertain`, `H-lock`, `store-shape`, `admission-shape`, `admission-acquire`, `service-loop`, `driver-publish`.


### SC-07 — Private ownership is real; mandatory design blockage is not independently established.

DriverMarkerAdvance ticket/next/body are private to marker.rs. H places a new impl in its sibling closure.rs, but already declares the needed method on the existing type. That method can legally live in the original producer module without exposing fields. H’s SettledGateCompletion private fields need a sealed consumption/accessor operation for a state-side planner; none is listed. Such an operation can also be supplied in the owning module without changing the authorized semantic contract.

Do not award compile-refusal credit: proposed code was never compiled. Narrow producer-owned placement/consumption clarification is useful. Source implementation must obey Rust privacy, but this inventory alone does not prove a new mandatory WHAT/HOW gate; Root decides classification before any author batch.

Evidence: `H-api`, `driver-private`, `driver-ports`, `driver-publish`, `WHAT-MB7-9`.


### SC-08 — The preserved capture catalogue exceeds the stated helper and write count.

Existing-repository capture contains 8 ownership + 3 format/ancestry/commit + 3 content scan + 5 destination/fetch/fsck/two traversals = 19 helper commands, plus one per .gitattributes and optional init; H also requires HEAD, historical verification, source corpus and later recapture/publication. Corpus admits at most 4096 entries and reads each eligible blob, not 14 helpers. Every UnitGit helper reserves then reconciles an effect; capture separately stages and marks Ready. Generic helper reservation has no aggregate count gate. The 256 admission gate and bounded Native inventory reader belong to distinct Native ports and do not bound this helper writer by construction. Existing workflow_publication validates generic finalization both before and after its five-command graph verification; H already replaces that publication entry point. Those two checks are inventoried here without creating another finding.

Confirmed false accounting and omitted capacity-consumer inventory. Valid large workloads may truthfully become Held at an existing profile bound; this map prescribes neither a new cap nor a guarantee that every maximum repository closes. No runtime overflow/control executed.

Evidence: `H-capture`, `H-next`, `H-closure`, `H-cost`, `capture-catalogue`, `capture-ownership`, `capture-content`, `capture-verify`, `corpus-bound`, `corpus-catalogue`, `helper-writer`, `input-effect-cap`, `inventory-cap`, `HOW-cost`, `WHAT-MB7-9`.


### SC-09 — 12 MiB per binding plan and the derived 2.5 GiB total omit retained encodings/images.

ManagedBindingPlan holds current successor, own Session/invocation/after Body and exact mutation. CurrentWorkflowSuccessor holds Arc<Body<Record>> and original plan. Body owns raw, canonical and parsed representations. ExactRowMutation holds independently owned old/new SqlValue vectors, and plan construction fills their body strings. Copy-for-transaction clones them again transiently. Shared Arcs do not imply repeated additional allocation, but the total retained custody and additional copies must be distinguished; the two named raw strings alone are not the whole encoded total.

No RSS/heap number or new cap is asserted; H explicitly disclaims those. This is an encoded/owned-copy accounting correction under existing finite-cost requirements. No allocation measurement performed.

Evidence: `H-cost`, `plan-memory`, `plan-construction`, `successor-memory`, `body-memory`, `mutation-memory`, `HOW-cost`, `WHAT-MB7-9`.


### SC-10 — The factual Workflow Waiting/Held status lacks a Goal/task-page/CLI consumer path.

H’s gate_observed retains Waiting/detail or Held reason without a Task rewrite and promises derived Goal/CLI visibility. Goal status counts and task pages consume waiting::observe/effective_state. That bounded reader requires native effects open and readiness start_ended=0/known_terminal=0; settled success instead returns generic Held. Its only effective state mappings are quota/capacity. Runtime control delegates to those readers; CLI prints their response. No Workflow evidence_integration_unavailable/held_reason observation is connected.

Actual non-Implement integration Waiting is truthful and must remain unverified; visibility is the missing consumer. Do not write Task WaitingHuman/blocker or infer runnable/policy/completion from factual rows.

Evidence: `H-gate`, `gate-final`, `waiting-reader`, `goal-status`, `goal-page`, `control-read`, `cli-read`, `cli-format`, `HOW-status`.


### SC-11 — SC3 does not establish the named Conflict branch or the claimed late-link mutant assertion.

Current Immediate acquisition propagates Err; H calls Err uncertain and SC3 nevertheless calls genuine lock contention Conflict. A future port could explicitly classify a known pre-transaction refusal, but the preserved current port does not. H’s Root calls late planning only with settlement Some; removing a None/live check inside that planner does not by itself make this caller reach it without settlement, nor furnish its other required images/receipt. A direct planner check is not a committed closed_settlement link.

Correct causal-control/outcome attribution is mandatory for claimed evidence. No demonstrated production bypass and no executed mutant kill. A direct defense-only check or an explicitly genuine reachable consumer setup must be labeled honestly; this map does not choose new failure semantics.

Evidence: `H-due`, `H-uncertain`, `H-controls`, `normal-write`, `H-late`, `WHAT-AC`.


### SC-12 — M1’s original Context maximum and headroom mutant are not genuine accepted positive setups as stated.

Native Source serializes Context.data into payload and carries its source_hashes/revision/scope/version into the original input. encode_input admits payload <=1 MiB and whole escaped frame <=2 MiB. ContextVersion has precisely those fields. An ordinary compactly written original Context near the stored decoder’s 8 MiB maximum cannot be counted as accepted merely from that decoder ceiling. A newly created closure Context and mechanical representation measurement are distinct. charged_scope_bytes aggregates whole scoped history plus open reservations; individual maximum bodies do not establish charged+growth+link near 128 MiB or causality for omitting 17.5 MiB headroom.

M1 can measure decoder/representation maxima separately, but oversized ingress is SETUP, not PR1 coverage. No admitted boundary or mutant kill was executed. Preserve actual input/scope bounds and genuine writer setup; no synthetic positive rows.

Evidence: `H-cost`, `H-controls`, `context-shape`, `context-input`, `input-limit`, `frame-limit`, `scope-charge`, `scope-bound`, `HOW-cost`, `WHAT-AC`.


## Current immutable pin differences

Direct source deltas inspected: test-only observations/accessors and publication probes, service observe_task_drivers visibility, and nongrant Native contract decision extraction preserving original planner predicates. The actual Native version helper monitor also now releases its raw custody mutex guard before hygiene reacquires it (CA-current-version-monitor); this fixes setup liveness and must not be omitted from pin provenance. It is a different monitor from UnitGit capture_scoped_pinned and changes neither SC-02’s generic finalization fence nor successful settlement delivery/binding/gate/status consumers. CA95 binding/setup controls are inventory, not execution evidence here.

Borrowed PhaseClosedLink retains SAME closure and audit_data for nonsuccess write; one corresponding consumer now passes link(). No successful owner/input/terminal/gate or normal binding consumer changed.

`/Users/shuheisuzuki/Documents/dev/rururunx/.rrx/development-evidence/issue43-native-success-continuation-design/sc-source-objects/CA-current-delta.patch`: 9202 bytes, SHA-256 `5674626df99af643f8abc3e5a2642563bf5865812a84cdb43e87d3e6b7a392db`.

`/Users/shuheisuzuki/Documents/dev/rururunx/.rrx/development-evidence/issue43-native-success-continuation-design/sc-source-objects/CA-current-monitor-delta.patch`: 3466 bytes, SHA-256 `2bd8c54583e4b6e75e8fa6c03786b71f34bf29d345e8f6d3ec57adffca3127dc`.

`/Users/shuheisuzuki/Documents/dev/rururunx/.rrx/development-evidence/issue43-native-success-continuation-design/sc-source-objects/RN-current-delta.patch`: 3480 bytes, SHA-256 `2ff28c4e268d77e96d702f314fce742d80ce9d61bfb49855cf7f49d10ebc193f`.

`/Users/shuheisuzuki/Documents/dev/rururunx/.rrx/development-evidence/issue43-native-success-continuation-design/sc-source-objects/RN-current-nonsuccess-delta.patch`: 615 bytes, SHA-256 `27ef3aaca9bfd85b10d11a216999985ef35b2c74f20ab9cb2937477a3d95a8f2`.


## Genuine control setup inventory, no execution credit

CA `fixture-setup` configures a real NativeAdapter and accepted Goal through ControlFixture, with an external protocol peer. `fixture-bound` checks the registered invocation and exact Task/Workflow binding facts, then releases and tears down; it proves no result/gate/closure in this report. R `fixture-commit` can wait for release and make a real worktree commit through ordinary Git. These are setup sources for actual composed controls, not copied grants, synthetic accepted rows, official auth/hook qualification or executed SC1. Normal/late success, whole-image confirmations, lock contention, stop timing, corruption timing and admitted headroom remain unexecuted.


## Optional factual notes and unchanged limits

SC-13: cleanup writes observation/event and overlays cleanup on read (`cleanup-write`, `cleanup-overlay`); it does not CAS the indexed Unit body/version. Optional correction only. SC-14: actual path is `state/runtime/waiting.rs`; the path typo is optional and distinct from confirmed SC-10.

- This is independent factual verification of the one completed review, not another review, contract authoring or approval.
- No source, WHAT/HOW/master, tests, permissions, hooks or configuration changed; no new model, provider, native workload, build, test or mutant ran.
- The original final review body is preserved byte-for-byte. Its mandatory labels are claims being checked, not approval evidence.
- H is proposed only. CA and RN are independent immutable descendants of R; this report does not treat them as composed or qualified source.
- Original-phase successful closure and legal marker-free next Driver/Source currentness are the common necessary dependency. Broader prepared HOW9.3 continuation, later offers/Reviewer/retries/retention/restart/evidence remain open; SC-N is not silently completed.
- Requirements/non-Implement evidence integration is absent; existing evaluate genuinely returns Waiting. Ready capture is not Published/Passed acceptance. Implement-only success cannot qualify the full contract or full MVP.
- Weak lookup alone is not a lifetime defect: actual handoff/marker ticket custody is retained. RN nonsuccess borrowed link cannot act as registered successful settlement proof.
- Cleanup Unknown/Leftovers remain independent from work outcome and do not create authority or impose a physical-death launch/closure barrier.
- SourceStop, Cancel and R2 human decisions remain unresolved; no reader, timeout, row-derived grant, synthetic owner or guard relaxation settles them.
- source_implementation_authorized=false; native_qualified=false; mvp_qualified=false; merge_ready=false. Root must independently verify classification before any nonfinding Opus author batch.

## Root go-ahead body, unchanged

This records the completed independent review admission; it does not grant authoring or source implementation. SHA-256 `67ebc931eccd014f1c3d482b7bcb1c498f3f16eeb33872907e1e4cbd81d6c14b`.

```json
{
  "recorded_at": "2026-10-07T02:12:19.551333+00:00",
  "head": "0c35653c7a8aa05c78c5285c2f4b5370db520e70",
  "base": "66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4",
  "worktree": "/Users/shuheisuzuki/Documents/dev/rururunx/worktree/issue-43-native-success-continuation-design",
  "clean": true,
  "handoff_sha256": "320360bc6b625923df743a378cd82d31f57f16b9f33c74b70095061688ceb2e4",
  "document_sha256": "2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6",
  "root_full_read_lines": [
    1,
    695
  ],
  "independent_review_authorized": true,
  "reviewer": "gpt-6.1-sol",
  "effort": "high",
  "reviewer_count": 1,
  "review_scope": "Entire proposed PR1 HOW against immutable governing WHAT and pinned production consumers; no production authorization",
  "explicit_contract_questions": [
    "Is original-phase closure plus legal next Driver/Source currentness sufficient for PR1, given prepared HOW9.3 Driver continuation after initial Evidence and the proposed SC-N wait?",
    "Do Waiting cases preserve actual missing Requirements evidence rather than asserting Implement-only/full Workflow success?",
    "Are all proposed APIs, currentness, uncertainty, terminal snapshots, permits and bounds concretely implementable against the immutable source pins?"
  ],
  "source_implementation_authorized": false,
  "native_qualified": false,
  "mvp_qualified": false,
  "merge_ready": false
}
```


## Contiguous literal evidence

Each excerpt is an exact contiguous line range from its specified Git blob, without editorial ellipses. Excerpt SHA-256 covers original bytes, excluding displayed line prefixes. Full cached object hashes and paths are in sc-source-object-manifest.json.


### WHAT-MB7-9

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `doc/requirements/issue-43-managed-binding-requirements.md` · lines 110–161 · blob `0731deb88550750ece8a3f81e7aa08b7501f891d` · object SHA-256 `11ec2b9281fa38558ce10b8edcfaa58e2a7c3c419f8e5e9b3cffa655ad297387` · excerpt SHA-256 `795934b007696d6c926d1ba854edd7181e8ee368fe71242dd828b4ece17d6413`.

```text
110: MB7. A post-marker binding refusal preserves the exact reservation and actual
111: owned invocation. It cannot fail the Task, release/retry/transfer, recapture
112: authority, send another input, stop another Task, or fabricate completion.
113: Storage contention/uncertain commit produces a typed deferred factual binding
114: handled by the actual retained invocation and authoritative Driver, independently
115: of a dropped Engine future. Unknown start delivery stays held. No-current-dispatch
116: and known failure use their separately proved non-success closure, not invented
117: binding. The successful late path uses the same binder and only a sealed genuine
118: current allocated/input/success-settlement proof, never a caller receipt ID.
119: 
120: For this managed profile, success-settlement means the actual private allocated
121: phase owner and prepared/admitted/actually consumed input, the exact correlated
122: current owned Native terminal with known successful work under the original
123: marker/instance, and recorded logical native-terminal/input-settlement evidence.
124: It is neither a physical-death/full-cleanup proof nor an accepted Workflow success.
125: Runtime-only result finalization may remain open for genuine later capture; it
126: grants no further native input/effects. This explicitly supersedes the legacy
127: late-binding physical full-settlement and unknown-cleanup refusal for this profile
128: only. Cleanup Unknown/Leftovers alone neither creates authority nor blocks an
129: otherwise eligible factual late binding or independent progress. Unknown work,
130: HistoricalDraft or ambiguous/missing input/current proof, stale/restored authority
131: and irreversible external-operation uncertainty keep their holds. Genuine result
132: capture/publication and success closure remain separately required; the terminal
133: receipt does not certify them.
134: 
135: MB8. Correctness does not depend on a notification edge: reconciliation readiness
136: is durable before/with settlement, rechecked on Driver registration/wake/return
137: and after owned start ends (normal, error, timeout, abort/drop or cancel). Finite
138: fair timer fallback covers lost/full/closed notifications without busy polling.
139: Passive status/poll observers do not bind or acquire ownership. Restart uses the
140: actual reviewed fencing/restore protocol; a durable row cannot reconstruct a
141: live owner. Unsupported/unknown recovery stays held and visible. Duplicate and
142: delayed normal/late delivery produce one exact binding/audit.
143: 
144: MB9. Private current-successor ledger and separately authorized bound-live
145: diagnostic/gate/closure ports retain original source/actor/lock/native currency.
146: Binding never authorizes their wider deltas. Preserve Design10's reserved-key
147: protection, immutable chain, finite per-class allowances and mandatory closure
148: capacity. Optional diagnostics cannot consume mandatory closure reserve or make
149: first binding tolerate Workflow drift. Full body planning/hashing occurs outside
150: the Store mutex; publication has no await/filesystem/native operation and enforces
151: explicit finite owner/body/lock/identity/ledger cost bounds. Bounds are admission
152: conditions, not truncation or a weakened empty-set proof.
153: 
154: SourceRecovery7's exact Workflow pins and Runtime9's future Driver pins must
155: recognize only a genuine private permitted successor anchored at their existing
156: immutable authority. Binding cannot update those authority rows, call the generic
157: after_write refresh, or silently recapture current pins to avoid a conflict.
158: The design must inventory every native live wait/status arm, including quota
159: Task projections and pre-Session NativeStart::Waiting: they are not automatically
160: the legacy detail-only diagnostic delta. Any legitimate state-changing port needs
161: its own exact typed authority/write contract; factual binding grants none.
```


### WHAT-AC

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `doc/requirements/issue-43-managed-binding-requirements.md` · lines 163–191 · blob `0731deb88550750ece8a3f81e7aa08b7501f891d` · object SHA-256 `11ec2b9281fa38558ce10b8edcfaa58e2a7c3c419f8e5e9b3cffa655ad297387` · excerpt SHA-256 `13f8345a826a1eec3987597d94b65998d22a50630bf66805d287cfc5747bf451`.

```text
163: ## Acceptance and evidence
164: 
165: | ID | Required actual consumer evidence |
166: | --- | --- |
167: | MB-AC1 | Engine + actual private #19 producer + registered native invocation holds a turn across marker/start/bind. Exact Task/P/G/Session/full locks stay unchanged; one Workflow/audit advance; real current callback/completion remains eligible. |
168: | MB-AC2 | Separate SQLite writer changes each P/G/T/Workflow/context/lock predicate during held start, including same-version raw body/REPLACE and added/deleted locks. Binding refuses with no binding/audit writes or new native input. |
169: | MB-AC3 | Foreign/missing/Lost/ambiguous Session, wrong actor/provider/role/worktree/native UUID/private owner/input pair, extra projection and generic writer attempts all refuse at the actual consumer. Passing producer prerequisites precede each negative. |
170: | MB-AC4 | Actual owned successful settlement before lost start delivery; normal/late race, deferred commit, dropped Engine future, lost/full notifications and actual Driver wake converge once. Authentic restart preserves required fencing; rows alone never grant admission. |
171: | MB-AC4.a | Normal returned Starting before input/ACK binds its genuine prepared owner without delivery/success claims. The same NotDispatched owner cannot late-bind. Actual current consumed successful terminal with cleanup Unknown/Leftovers can late-bind once; Unknown work, HistoricalDraft, wrong input/marker/instance and ambiguous external outcome cannot. |
172: | MB-AC5 | Genuine bind through diagnostic/gate/success and failure closure; exhausted optional allowance still permits mandatory closure; malformed/disconnected/replayed ledger rejects without effects. |
173: | MB-AC6 | Migrate positive Workflow fixtures through actual producer ports. Capability-only advertisers remain negative. No disabled tests, fake owner/pair, copied authority, fixture exemption or weakened native mechanism. |
174: | MB-AC7 | Compile clean committed mutations restoring Task bump, refreshing captured frame, dropping each CAS/identity/private-proof/sole-writer/delta/ledger check or inferring success. Each fails at its intended actual consumer assertion; exact source restoration passes. |
175: | MB-AC8 | Independent immutable requirements/design/source reviews; composed current-main fmt/Clippy/build/full Workflow regressions and macOS/Ubuntu CI. Controlled protocol peers and authenticated CLI/hook/four-Task qualification are reported separately. |
176: 
177: ## Delivery and open gates
178: 
179: Requirements approval precedes managed binding design and production changes.
180: The design must specify actual allocation/frame producers, every writer/consumer,
181: schema/migration composition, finite cost classes, typed failure/reconciliation
182: states and current Driver integration. Native readiness remains unavailable until
183: those real ports compose; an isolated binder test is not a completed delivery.
184: 
185: The legacy approved private #19 ports and this managed profile are not declared
186: equivalent. An implementation must either compose the actual #19 producer or
187: deliver its complete prepared-owner/input protocol through the real managed
188: adapter and all protected writers, with explicit joint design/source review and
189: the above causal controls. Renaming Unit/Invocation DTOs is insufficient. The
190: normal/late/diagnostic/recovery requirements remain open until their actual
191: consumers are verified. No README, license or whole-MVP claim is changed here.
```


### HOW-status

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `doc/design/issue-43-managed-binding-design.md` · lines 565–590 · blob `cf19dcc12cf2d0410727a457e5e4d2eb5625f399` · object SHA-256 `f03efbb1b2ab7986936cac72be5580c939f31c223a4178c74d0a5d5e79dd4d00` · excerpt SHA-256 `eb760b4a1888aeae4026ce1ef9417cb34b5e0938f9897417cfe13c9b55060d25`.

```text
565: | session_bound | §4 only; actual normal/late private producer, first link |
566: | native_diagnostic | Bound live status_unavailable or persisted_status_mismatch only: active.detail≤128 UTF-8 bytes, Record version/time and one link; no Task/owner/native wait write |
567: | gate_claim | Actual private gate invocation: Evaluating + exact claimed_observations; no Session binding/source refresh |
568: | gate_observed | Actual compact claim outcome + exact Waiting/held disposition fused into ONE link/transaction, including bounded detail/held_reason; repeated identity may change only checked occurrence count/last_time |
569: | gate_hold | Actual typed hold/clear policy, exact held_reason/detail; no Task WaitingHuman rewrite |
570: | terminal_decision | Trusted cancel/fail_task action and its separately checked Task lifecycle delta; no success/binding/input |
571: | phase_closed | Actual eligible typed result/gate/non-success/TerminalRecovery proof and enumerated final phase/body/Task/Context changes; no transport-derived approval |
572: 
573: Native quota live wait/recovery does **not** use native_diagnostic or these Workflow
574: links. It updates its genuine Unit/pool/waiter/lease facts and derived status only;
575: no Task/W rewrite occurs while open. Terminal quota/capacity interruption uses
576: typed non-success phase closure with preserved Unknown work and explicit wait/fresh
577: retry policy, not live quota as failure. This choice preserves fixed ledger budgets.
578: Typed derived waiting feeds actual Workflow/Goal/Runtime/CLI status and scheduling;
579: it is not cosmetic text masking a hidden Task update.
580: 
581: SourceRecovery7 currently compares serialized full snapshot pins exactly
582: (`source_recovery.rs:286–293`) and generic after_write refreshes its row
583: (`342–367`). A binder Workflow+1 therefore needs a **new bounded private successor
584: recognizer** in validate_task/validate_binding/recovered_source_task and the actual
585: inputs/prepare_pack/registered Git consumers. Its anchor is the existing saved
586: SourceRecovery Workflow pin at the operation marker, advanced only by the genuine
587: pre-marker typed reserve/marker transaction. It may resolve that exact anchored
588: Workflow pin to the chain-proved current Workflow while every other saved source
589: pin remains exact. It neither mutates row.pins/version nor invokes generic
590: after_write. Subsequent original operation source authority stays the original
```


### HOW-cost

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `doc/design/issue-43-managed-binding-design.md` · lines 670–697 · blob `cf19dcc12cf2d0410727a457e5e4d2eb5625f399` · object SHA-256 `f03efbb1b2ab7986936cac72be5580c939f31c223a4178c74d0a5d5e79dd4d00` · excerpt SHA-256 `d9deef4717300fd45968a4a6e0da614fa78c398c8624d3e380176d88a0f6ffb5`.

```text
670: releases only unspent reserve; spent retained bytes remain charged. Held retains
671: reserve. All new durable bodies count toward the actual128-MiB Workflow budget.
672: 
673: Complete chain extraction outside Store is≤256 links/≤1 MiB. Sealed compact
674: transaction proof is≤128 metadata rows×4096 bytes≤512 KiB, including all actual
675: certificates/endpoints/scalars. This is **not** the total SQL/JSON/write cost.
676: Every port declares only the surfaces it reads and instruments complete encoded
677: rows, repeated reads, JSON/CHECK and writes; unlisted/oversized surfaces refuse.
678: 
679: | Additional mandatory surface | Maximum complete encoded cost class |
680: | --- | --- |
681: | P/G/T | One each≤8 MiB; existing Goal DAG limits remain |
682: | Workflow | One≤8 MiB; OLD/NEW full projection and body-write/CHECK cost measured |
683: | launch/latest Context | At most two rows≤8 MiB each; input payload remains≤1 MiB |
684: | own Session | One≤4 MiB; recovery≤3 MiB/depth32/nodes32768; current read inside Immediate |
685: | original operation / frozen settlement | One≤4 MiB / one≤64 KiB; Native6 answer text≤1 MiB separately, receipt's existing complete encoding bounds retained |
686: | preparation/admission/owner | One each≤2 MiB; stricter8192-byte consumed admission remains |
687: | scoped WorktreeLock set | Complete≤256×16384 bytes≤4 MiB; sorted original ID/version/digest inventory≤64 KiB |
688: | scoped identity / retained physical owner negative lookup | Complete indexed≤4096×16384 bytes≤64 MiB, paths≤4096 bytes; overflow refuses; no physical acquisition by binder |
689: | applicable logical host-effect closure | ≤256 admissions×8 KiB +256 outcomes×2 KiB; all worker joins outside Store; stricter existing Unit/effect limits still apply |
690: 
691: Mandatory indices/scope constraints prevent unlimited irrelevant global scans;
692: complete own-scope index validation and bounded malformed flags are required.
693: Full body planning/hash/chain proof and filesystem/path preparation occur outside
694: Store. Publication has no await, filesystem, process or native call. Current row
695: CAS and full scoped lock/own Session/negative checks stay inside Immediate.
696: Finite cost is neither a measured latency bound nor a guarantee of mutex fairness;
697: near-bound controls must report other-Project contention honestly.
```


### prepared-PR1

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `doc/design/issue-43-native-prepared-producer-design.md` · lines 548–555 · blob `0a541476d18081cb6bf7e03b0c2477ecd6848eba` · object SHA-256 `02d30312099882f7744f0f79a6aa3db46845399a8e80fca7949fe64d4b39a326` · excerpt SHA-256 `f4dd1285bd96ba94f6faf221563e231c7187b9ad04fbaa99ed8d06fd1e273f8f`.

```text
548: ### 9.3 Prerequisites (missing; owned by A/Root; not satisfied by this HOW)
549: 
550: 1. **PR-1:** Driver continuation after initial Evidence. Today `driven_initial.rs:56–59` refuses it.
551: 2. **PR-2:** Source offers for a later Executor, a retry and a Reviewer. These replace the first-Executor-only predicates at `native_handoff.rs:448–519`. A retry uses a NEW Unit, worktree, branch and namespace (`attempts.rs:352–470`, `resources.rs:294–297`), and a new operation, pair, Session and marker. The prior operation must be closed or Held. It is never reused.
552: 3. **PR-3:** a Reviewer Frame at artifact revision `A`, built from retained objects. Also `ResultSnapshot` creation moved into the Source lane (`attempts.rs:604–711`) with the `ReviewerArtifactLease` handed into Source custody.
553: 4. **PR-4:** artifact retention release must treat an open operation whose custody references the artifact as a live dependency.
554: 
555: Until PR-1 to PR-4 exist, a Reviewer or retry Unit refuses before any Git intent, exactly as today.
```


### H-due

H `0c35653c7a8aa05c78c5285c2f4b5370db520e70` · `doc/design/issue-43-native-success-continuation-design.md` · lines 302–330 · blob `fd9010eb78c3b7f91dad165447e754166a7daaa7` · object SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6` · excerpt SHA-256 `84df85b0e5c27c834f7f30d42390829956c15e6dd069b50b80d822f6f1bab44d`.

```text
302: ### 5.2 Root reconciliation sweep (`PhaseJobs::reconcile_success`)
303: 
304: The service loop calls it after RN's `reconcile_nonsuccess`; its `bool` ORs into `pending`. It reuses RN's sweep discipline: it snapshots ≤128 entries under `entries`, classifies under `job.state` only, uses its own `success_cursor`, takes ≤8 turns per sweep, runs ≤1 query-only snapshot plus ≤1 Store-mutex transaction per turn, and backs off 100 ms → 5 s.
305: 
306: A job is **due** when its `outcome` is `Launched` and its binding is not yet acknowledged, or its `SuccessContinuation` has a pending uncertain write, an unpublished Driver closure, or releasable acknowledgments. Per turn, in order:
307: 
308: 1. **Uncertain retained binding plan.** Run `confirm_managed_binding(plan)`:
309:    - Known → install `BindingAcknowledgment`;
310:    - RolledBack → clear the uncertain mark and keep the plan;
311:    - Err → Held, re-probed read-only every 5 s.
312: 2. **No acknowledgment.** Take `snap = binding.owner_arc().binding_snapshot()?`:
313:    - settlement Some → `plan_late_binding`, then `bind_managed_phase`. The new plan replaces the retained one **only** after that retained plan is RolledBack or definitively refused; never both;
314:    - settlement None, owner live, last refusal a definitive `Conflict` → retry `plan_managed_binding` after backoff;
315:    - otherwise not due.
316: 3. **Acknowledgment plus settlement.** `SettledPhase::issue(snap, ack)`, then install `SuccessContinuation` once (pointer-checked) and send a watch hint. This is memory only.
317: 4. **Continuation housekeeping.** At most one of: confirm a retained uncertain claim, observed or closure plan (§11); publish the Driver closure (§10.4); or release acknowledgments (§10.5).
318: 
319: The start task's existing one-shot `bind_returned` remains the fast path. It now installs its plan before Store, as today, and records `Known`, `Conflict` or uncertain instead of only Ok/Err. Its outcome is never retried in the start task.
320: 
321: **Convergence.** Normal and late cannot both commit: the trigger allows at most one `session_bound` per operation, the first-link CAS requires the original Workflow, and only one plan is retained per job. Duplicate delivery (start task and sweep, or repeated sweeps) meets the single-flight stage token and then exact confirmation. Lost, full or closed watch notifications only delay the sweep; the service timer re-classifies from retained state.
322: 
323: ### 5.3 Confirmation
324: 
325: `confirm_managed_binding` runs one read Immediate with no write, under the full original currency of the plan's kind:
326: - **Known** iff the Workflow equals the plan's exact postimage and exactly one `session_bound` link exists with the plan's `at` and byte-equal payload. This applies equally to a committed normal or late plan.
327: - **RolledBack** iff the Workflow equals the original post-marker raw and there are zero links.
328: - **Anything else is Held.**
329: 
330: The fresh-plan `AlreadyBound` path (`binding.rs:331–358`) is reached only when no retained plan exists (impossible within one epoch); a mismatching `proof_source` is Held.
```


### H-next

H `0c35653c7a8aa05c78c5285c2f4b5370db520e70` · `doc/design/issue-43-native-success-continuation-design.md` · lines 332–367 · blob `fd9010eb78c3b7f91dad165447e754166a7daaa7` · object SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6` · excerpt SHA-256 `48697cb247857fde87b2517690ee7a6e517d4242b17f1888c3f8e471208cffc3`.

```text
332: ## 6. Driver continuation (P)
333: 
334: ### 6.1 Lookup
335: 
336: `Runtime::settled_phase(task, association)` reads Root custody, pointer-checked, holding one lock at a time:
337: 1. `PhaseHandoffs.entries[task]` → `slot.handoff` → `assets.origin`.
338: 2. `origin.ticket.upgrade()`; its association must satisfy `same_association(caller)`. Proposed: a `pub(crate)` read-only accessor on `DriverReadTicket` that returns its retained association by reference.
339: 3. `origin.allocation` → the PhaseJobs entry by operation, with `Arc::ptr_eq` on the allocation.
340: 4. The job's `success` cell, or its RN or success closed acknowledgment.
341: 
342: It returns `NoHandoff`, `Pending` (not yet bound or settled), `Held(reason)`, `Settled` or `Closed`. Nothing is read from SQL. Passive status and Goal views do not call it.
343: 
344: ### 6.2 Step state machine
345: 
346: In `step_driven_initial`, the order is:
347: 1. RN's Failed branch (unchanged).
348: 2. For an active Executor attempt: `Settled` → `continue_settled_executor`; `Pending`/`NoHandoff` → the existing handoff observation (unchanged); `Held` → `StepResult::Waiting{reason}` with no write.
349: 
350: `continue_settled_executor` dispatches on the Workflow attempt state and the retained stage. It performs at most one durable write per call, and every await happens before any Store guard:
351: 
352: | Attempt / stage | Action |
353: | --- | --- |
354: | Running and bound | (a) capture (§8); (b) `inputs` → captured `SourceSnapshot`; (c) drift policy: class escalation, rules change or `!same_sources` for a non-target-producing phase → Held with no write (invalidation of a bound open phase needs its own typed port); (d) plan and write `gate_claim` (§9.1) |
355: | Evaluating, claim acknowledged, no evaluation started | Set `evaluation_started` under the stage mutex, then `evaluate_settled` (§9.2). Retain `SettledGateCompletion`, then plan and write `gate_observed` |
356: | Evaluating, evaluation started, no retained outcome | Held: "gate outcome unknown; explicit recovery". No re-evaluation |
357: | Evaluating with a Passed observation acknowledged | `settled_publication`, then `prepare_pack(next)`, then plan, materialize and close (§10) |
358: | Waiting (from `gate_observed`) | Read-only `StepResult::Waiting{detail}`. **No automatic re-claim** |
359: 
360: The SAME retained plan is reused after an uncertain write (§11). The Driver writes or confirms only through the single-flight stage token, and the Root sweep never evaluates gates or captures.
361: 
362: ### 6.3 After closure
363: 
364: Once the job holds a known success acknowledgment and the Driver is published, these run once each:
365: - `retire_closed_handoff` on the Sources slot (pointer-checked custody allocation);
366: - the read-only branch: active None, with the last attempt the first-Executor Succeeded phase → `StepResult::Waiting { phase: next, reason: "<next> typed Driver continuation unavailable (SC-N)" }`. It writes nothing, grants nothing, and replaces only the `bail!` for this state. `Finished` (no next phase) is returned unchanged.
367: 
```


### H-late

H `0c35653c7a8aa05c78c5285c2f4b5370db520e70` · `doc/design/issue-43-native-success-continuation-design.md` · lines 280–300 · blob `fd9010eb78c3b7f91dad165447e754166a7daaa7` · object SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6` · excerpt SHA-256 `f18eaf25e9ba4173376e924e6f099b5eaa710f9c846655da4203cc5e8b6fc3f2`.

```text
280: 
281: The late planner shares `projection` and the Workflow, Session and identity planning with the normal binder.
282: 
283: - **Memory conjuncts.**
284:   - `!proof.is_live()`, and `proof.owner().validate_known_registration()` is Ok.
285:   - `s = proof.settlement()` is Some, with `s.belongs_to(owner)` and `s.consumed().belongs_to(owner)`.
286:   - `proof.consumed()`, when Some, is pointer-equal to `s.consumed()`.
287:   - `s.allocation()` is pointer-equal to `marker.allocation()`.
288:   - The receipt is `OwnedTerminal` with Success.
289:   - `s.unit()` is WorkKnown/Success with closed effects, open finalization and session = allocated.
290:   - `s.session()` is Exited with `native_ref == s.thread()`.
291: - **Unit lineage.** The current successor Unit must equal `s.unit()` except for the **cleanup axis**: `version`, `updated_at` and `cleanup` only. This tolerates Core's `record_execution_cleanup` and nothing else.
292: - **Immediate conjuncts.** These run in addition to the binder's `validate_current_tx`, `validate_driver_live_tx`, the exact Session row and negative identities. Every expected image is planned outside Store from the SAME settlement:
293:   - the registered owner row is exact, with the invocation required to be `closed`. This is a variant of `validate_registered_owner_tx`; the normal variant keeps `<>'closed'`;
294:   - the invocation row is exact (Closed, with `native_thread`/`native_turn` equal to `s.thread()`/`s.turn()`);
295:   - the `native_results` row for the invocation is exact, body = the planned `serde_json::to_string(s.receipt())`;
296:   - the `managed_phase_admissions` row is exact with `confirmed=1, settled=1, uncertain=0`, and `input_effect_id`/`frame_sha256` equal to `s.consumed()`;
297:   - the `managed_phase_readiness` row is exact (`closed`, `start_ended=1`, `known_terminal=1`);
298:   - the latest Session is Exited. The late variant of `latest_session` admits only Exited, and the normal variant is unchanged.
299: - **Projection and payload.** The projection is identical to normal (`session_id` plus derived `execution`). The payload is identical except `proof_source="closed_settlement"` and `private_receipt_ref=<receipt id>`, as managed binding §4 already specifies.
300: - **Fast terminal.** A terminal that commits during a normal binding Immediate changes the Unit index, so normal refuses as a definitive `Conflict`, and a later turn plans late.
```


### H-sealed

H `0c35653c7a8aa05c78c5285c2f4b5370db520e70` · `doc/design/issue-43-native-success-continuation-design.md` · lines 245–257 · blob `fd9010eb78c3b7f91dad165447e754166a7daaa7` · object SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6` · excerpt SHA-256 `bafb5e02a9b651ab6e6fc8ed3e82e7ac5f51c04df9dfa8fc1412ce48e7374f5e`.

```text
245: **Existing APIs consumed unchanged (V):**
246: - `NativePhaseSession::binding_snapshot`, `is_live` and `validate_known_registration`;
247: - the accessors of `NativePhaseBinding` and `OwnedPhaseSettlement`, and `belongs_to`;
248: - `ConsumedPhaseInput::{belongs_to, effect, frame_sha256, acknowledgement}`;
249: - `plan_current_phase` and `validate_current_tx`;
250: - `OriginalMarker::validate_driver_live_tx`;
251: - `ExactRowMutation` and `PrivatePermitManager::with_exact_permit`;
252: - `charged_scope_bytes`;
253: - `ResultStore::verify` (historical `RetainedGit`) and `ready_result`;
254: - `DriverAssociation::{same_association, publish_exact}`;
255: - `prepare_pack`, `validate_transition`, `validate_context` and `validate_evidence`.
256: 
257: No existing private constructor is widened. `OwnedPhaseSettlement::completed` stays `pub(super)`, and `WorkflowPublication`'s fields stay private.
```


### H-api

H `0c35653c7a8aa05c78c5285c2f4b5370db520e70` · `doc/design/issue-43-native-success-continuation-design.md` · lines 167–240 · blob `fd9010eb78c3b7f91dad165447e754166a7daaa7` · object SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6` · excerpt SHA-256 `e4dbf39c6d7e11a3f6eed5d2c95f941bb4b86a2ca2a2f535ab4903a6d1ce5509`.

```text
167: pub(crate) struct GateClaimPlan { /* compact: digests, at, attempt, headroom; Arc<SettledPhase> */ }
168: pub(crate) struct GateClaimAcknowledgment { plan: Arc<GateClaimPlan> }
169: pub(crate) struct GateObservedPlan { /* claim ack, retained outcome, observation ≤64 KiB, digests */ }
170: pub(crate) struct GateObservedAcknowledgment { plan: Arc<GateObservedPlan> }
171: pub(crate) struct SuccessClosurePlan { /* §10.1 */ }
172: pub(crate) struct SuccessClosureMaterial { /* one turn; borrowed, freed after the Store guard */ }
173: pub(crate) struct SuccessClosureAcknowledgment { plan: Arc<SuccessClosurePlan> }
174: pub(crate) enum SuccessWrite<A> { Known(Arc<A>), Conflict(anyhow::Error) }   // Err = uncertain
175: pub(crate) enum SuccessConfirmation<A> { Known(Arc<A>), RolledBack }         // Err = Held
176: impl Store {
177:     pub(crate) fn plan_settled_gate_claim(owner: &RuntimeOwner, c: SettledCurrency, at: i64)
178:         -> Result<Arc<GateClaimPlan>>;
179:     pub(crate) fn claim_settled_gate(&mut self, p: &Arc<GateClaimPlan>) -> Result<SuccessWrite<GateClaimAcknowledgment>>;
180:     pub(crate) fn confirm_settled_gate_claim(&mut self, p: &Arc<GateClaimPlan>)
181:         -> Result<SuccessConfirmation<GateClaimAcknowledgment>>;
182:     pub(crate) fn plan_settled_gate_observed(owner: &RuntimeOwner, claim: &Arc<GateClaimAcknowledgment>,
183:         outcome: SettledGateCompletion, at: i64) -> Result<Arc<GateObservedPlan>>;
184:     pub(crate) fn observe_settled_gate(&mut self, p: &Arc<GateObservedPlan>)
185:         -> Result<SuccessWrite<GateObservedAcknowledgment>>;
186:     pub(crate) fn confirm_settled_gate_observed(&mut self, p: &Arc<GateObservedPlan>)
187:         -> Result<SuccessConfirmation<GateObservedAcknowledgment>>;
188:     pub(crate) fn plan_phase_success(owner: &RuntimeOwner, observed: &Arc<GateObservedAcknowledgment>,
189:         publication: WorkflowPublication, context: ContextVersion, at: i64) -> Result<Arc<SuccessClosurePlan>>;
190:     pub(crate) fn materialize_phase_success(p: &Arc<SuccessClosurePlan>) -> Result<SuccessClosureMaterial>;
191:     pub(crate) fn close_phase_success(&mut self, m: &SuccessClosureMaterial)
192:         -> Result<SuccessWrite<SuccessClosureAcknowledgment>>;
193:     pub(crate) fn confirm_phase_success(&mut self, m: &SuccessClosureMaterial)
194:         -> Result<SuccessConfirmation<SuccessClosureAcknowledgment>>;
195:     /// Exact committed-row check plus the SAME association's publish_exact (mirrors publish_driver_marker).
196:     pub(crate) fn publish_success_driver(&mut self, ack: &Arc<SuccessClosureAcknowledgment>) -> Result<()>;
197:     pub(crate) fn reserve_settled_helper(&mut self, c: &SettledCurrency, id: OperationId, path: &Path)
198:         -> Result<()>;                            // kind "git_helper", native=false only
199:     pub(crate) fn stage_settled_result(&mut self, c: &SettledCurrency, a: &ResultArtifact) -> Result<()>;
200: }
201: 
202: // ---- state/runtime/driver/closure.rs (new; sibling of marker.rs) ----
203: pub(crate) struct DriverClosureAdvance { /* old = SAME DriverMarkerAdvance post row; new = marker-free row */ }
204: impl DriverMarkerAdvance {               // existing type
205:     pub(crate) fn plan_success_closure(&self, task_after: &Task, workflow_after: &Record,
206:         context_after: &ContextVersion, unit_after: &ExecutionUnit) -> Result<DriverClosureAdvance>;
207: }
208: 
209: // ---- execution/results.rs, git_io.rs, workflow_source.rs, workflow_gates.rs (extended) ----
210: impl ResultStore {
211:     pub(crate) async fn capture_settled(&self, s: &Arc<SettledPhase>, revision: &str,
212:         sources: BTreeMap<String, String>) -> Result<ResultArtifact>;
213:     /// The sole sibling issuer of the existing private WorkflowPublication.
214:     pub(crate) async fn settled_publication(&self, s: &Arc<SettledPhase>, artifact: ArtifactId)
215:         -> Result<WorkflowPublication>;
216: }
217: impl UnitGit { pub(crate) fn for_settled(owner: Arc<RuntimeOwner>, s: Arc<SettledPhase>) -> Result<Self>; }
218: impl ManagedWorkflowSources {
219:     pub(crate) async fn capture_settled(&self, s: &Arc<SettledPhase>, project: &Project, task: &Task)
220:         -> Result<Arc<Frame>>;
221:     pub(crate) async fn retire_closed_handoff(&self, task: TaskId, ack: &ClosedPhaseAck) -> Result<()>;
222: }
223: pub(crate) struct SettledGateCompletion { outcome: GateOutcome, receipt: Option<Record>, claim: Arc<GateClaimAcknowledgment> }
224: impl ManagedWorkflowGates {
225:     pub(crate) async fn evaluate_settled(&self, s: &Arc<SettledPhase>, claim: &Arc<GateClaimAcknowledgment>,
226:         invocation: PhaseInvocation) -> Result<SettledGateCompletion>;
227: }
228: 
229: // ---- runtime (extended) ----
230: pub(crate) struct SuccessContinuation {                     // runtime::phase_jobs; held in JobState
231:     settled: Arc<SettledPhase>,
232:     stage: Mutex<SuccessStage>,                               // leaf mutex, single-flight token
233: }
234: pub(crate) enum ClosedPhaseAck { NonSuccess(Arc<PhaseClosedAcknowledgment>), Success(Arc<SuccessClosureAcknowledgment>) }
235: pub(crate) enum SettledLookup { NoHandoff, Pending, Held(&'static str), Settled(Arc<SuccessContinuation>), Closed(ClosedPhaseAck) }
236: impl Runtime {
237:     pub(crate) fn settled_phase(&self, task: TaskId, association: &DriverAssociation) -> Result<SettledLookup>;
238: }
239: impl PhaseJobs { pub(super) fn reconcile_success(&self, phases: &PhaseSupervisor, stopping: &AtomicBool) -> Result<bool>; }
240: impl PhaseHandoffs { pub(super) fn retire_closed(&self, task: TaskId, ack: &ClosedPhaseAck) -> Result<()>; }
```


### H-capture

H `0c35653c7a8aa05c78c5285c2f4b5370db520e70` · `doc/design/issue-43-native-success-continuation-design.md` · lines 371–404 · blob `fd9010eb78c3b7f91dad165447e754166a7daaa7` · object SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6` · excerpt SHA-256 `7b04b77df0d6316963828a8b372569ae280c3dd33f13ad988b9bc21f8d51de5d`.

```text
371: 
372: 1. `marker.validate_driver_live_tx(tx)`: the SAME frozen post-marker Driver row, the live association, and the exact Source7 result or ticket source.
373: 2. `validate_current_tx(tx, marker, &c.current)`: the open operation; exact P/G/T with the Task = marker `task_after`; the Workflow endpoint and ledger head; locks and latest Context = the original; the exact current Unit index.
374: 3. The planned Unit equals `c.settled.settlement().unit()` modulo the cleanup axis. This was checked at planning. Under Store only the exact current Unit bytes are compared, and a mismatch replans (≤3 consecutive replans per stage, then Held).
375: 4. Finalization open, effects closed, `session_id` = allocated.
376: 
377: Exact P/G bodies subsume the generic parent-activity and governing-digest checks. Each protected Store call plans `SettledCurrency` query-only outside the Store mutex, so a capture with ≤14 helpers runs ≤15 snapshots plus 1 recheck.
378: 
379: **Consumer mapping.** Generic consumers stay unchanged and keep refusing marker-bound rows. Every protected consumer below is new and uses `validate_settled_tx`:
380: 
381: | Generic consumer (refuses marker) | Protected PR1 consumer |
382: | --- | --- |
383: | `UnitGit::run_command` → `validate_execution` + `reserve_execution_helper_pinned` | `UnitGit::for_settled` → `reserve_settled_helper`; `reconcile_managed_effect` (no authority check) unchanged |
384: | `ResultStore::capture` → `validate_execution`, `stage_result` | `capture_settled` → `stage_settled_result`; `ready_result` unchanged |
385: | `workflow_publication` → `validate_execution` + `UnitGit::new` | `settled_publication` → protected validator + `UnitGit::for_settled` + `verify_inner(Current)` |
386: | `ManagedWorkflowGates::terminal` → `validate_execution` | `settled_terminal`: the same predicates sourced from the settlement and exact rows |
387: | `frame()` capture branch | `capture_settled` (same slot serialization; requires the slot's `handoff` custody allocation = settlement allocation) |
388: | `publish_workflow_result_tx` → `validate_authority` | `publish_result_core` inside `close_phase_success` (§10) |
389: | Generic Driver `validate` / Source `validate_row` / `after_write` | Unused on the protected path. Source7 present at the marker → success closure Held (§3.2) |
390: 
391: ## 8. Capture (P)
392: 
393: `capture_settled` keeps `capture`'s exact steps:
394: - HEAD of the Unit worktree via `rev-parse`; the object-format check; ownership;
395: - `merge-base --is-ancestor base HEAD`; exact `^{commit}`; `qualified_content_scoped`;
396: - staging; fetching commit and base into the per-Project `results.git` (no alternates or hardlinks); `fsck --full --strict`; `rev-list --missing=error` for both OIDs;
397: - the durable manifest, fsync, and Ready.
398: 
399: Every helper runs under `UnitGit::for_settled`; the Driver ticket is not used. `capture_settled` then runs `verify` (historical reader). The Sources slot frame becomes the captured revision and artifact, exactly as `frame()` does today.
400: 
401: - **Content stays untrusted.** Native output is untrusted Git content; fsck and the bounded readers are the existing safeguards.
402: - **A text answer is not a result.** An `APPROVE` text answer is not a result. Only the captured HEAD graph is.
403: - **HEAD == base.** PR1 preserves the existing predicate, which accepts HEAD == base (AUTHOR_NOTES). Controls credit only the commit-producing fixture mode.
404: - **Helper outcomes.** A helper `Unknown` outcome makes the capture fail, and the stage is Held: closure requires every Unit helper effect to be settled (§10.2).
```


### H-gate

H `0c35653c7a8aa05c78c5285c2f4b5370db520e70` · `doc/design/issue-43-native-success-continuation-design.md` · lines 416–440 · blob `fd9010eb78c3b7f91dad165447e754166a7daaa7` · object SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6` · excerpt SHA-256 `5654aa42bc0a4b12a23318238e137f28632c77e226204f58b9da1249118f8018`.

```text
416: ### 9.2 `evaluate_settled`
417: 
418: It is `evaluate` with three substitutions:
419: 1. The claim check is `Self::claim` (reads only).
420: 2. Implement uses `settled_terminal`, which applies the same predicates as `terminal` (`workflow_gates.rs:172–198`) from the SAME settlement: `attempt.execution`/`unit`/`agent`/`session_id`; Exited; Success; WorkKnown; finalization open; effects closed. The Unit is compared modulo the cleanup axis.
421: 3. The source recapture is `capture_settled` and must equal the invocation sources.
422: 
423: Everything else is unchanged: the Ready artifact = the invocation artifact; `ResultStore::verify`; the exact claim recheck; the `Verification` receipt via `put_record`; `Evidence.session_id = settlement Session`. Requirements and every non-supported phase return the existing `Waiting`.
424: 
425: `SettledGateCompletion` is constructed only here. It is retained in the stage before any write.
426: 
427: ### 9.3 `gate_observed` (fused, one link)
428: 
429: - **Workflow delta.** Append exactly one `GateObservation { sources: authority_only(source), outcome, error, at }`, encoded at most 64 KiB (refused otherwise, never truncated). Then apply the disposition:
430: 
431:   | Outcome | Attempt after | Workflow-level | Link `outcome`, `reason_code` |
432:   | --- | --- | --- | --- |
433:   | Passed(evidence) | stays Evaluating | — | `passed`, gate receipt id, evidence SHA-256 |
434:   | Waiting(fixed string) | Waiting, `detail` = that compile-time string (≤128 B) | — | `waiting`, `evidence_integration_unavailable` |
435:   | Failed(_) | Waiting, `detail` = `"gate failed; bound non-success closure unavailable"` | `held_reason` = same constant | `held`, `gate_failed` |
436:   | Err (unknown) | Waiting, `detail` = `"gate outcome unknown; explicit recovery required"`; `error` = that constant | `held_reason` = same | `held`, `gate_unknown` |
437: 
438: - **No raw text.** Raw error and gate text never reach durable rows or payloads; they go only to bounded in-memory attention.
439: - **No Task writes.** No Task `WaitingHuman` or blocker is written. Waiting and Held status are derived from the Workflow for the Driver, Goal and CLI.
440: - **Trigger.** The trigger requires the immediately preceding `gate_claim`.
```


### H-closure

H `0c35653c7a8aa05c78c5285c2f4b5370db520e70` · `doc/design/issue-43-native-success-continuation-design.md` · lines 471–505 · blob `fd9010eb78c3b7f91dad165447e754166a7daaa7` · object SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6` · excerpt SHA-256 `63e155e6d438b493945622b9647ba4fe44b8ae4555795ab1e856643563eba830`.

```text
471: The plan retains compact digests and scalars, plus the Context raw (≤8 MiB, not re-derivable: rules are read from the Project) and the Arcs. Every other image is re-materialized per turn from the SAME retained inputs, and the digests must match; a mismatch is Held.
472: 
473: ### 10.2 Immediate conjuncts (all before any write)
474: 
475: `close_phase_success` takes `control_admission` (awaited before Store, as CA §5.7 and marker publication do), then the synchronous SharedStore, then one `Immediate`:
476: 1. The selected database path. `validate_settled_tx` against the planned endpoint (Workflow = the `gate_observed` postimage; ledger head = that link).
477: 2. The Unit, artifact (Ready) and Task (the marker `task_after` raw) rows are exact. The latest Context version = the original, and no row exists at the next version.
478: 3. Every Unit `managed_effects` row is non-Pending and non-Unknown (≤257 rows); there is no `retained_git` Pending row.
479: 4. Ledger: count < 256 and zero `phase_closed`.
480: 5. Budget: `charged + all growth ≤ WORKFLOW_BYTES`.
481: 
482: Any failure rolls back and returns `Conflict(cause)`.
483: 
484: ### 10.3 Writes (rowcount exactly 1 each)
485: 
486: | # | Row | Guard |
487: | --- | --- | --- |
488: | W1 | `execution_units` exact 13-column CAS | writer contract (not a permit table) |
489: | W2 | `result_artifacts` Ready → Published, exact version/body CAS | existing publication semantics |
490: | W3 | `tasks` exact old/new CAS (the `put_task_tx_at` column set plus a `body=old` predicate) | not a permit table |
491: | W4 | `context_versions` INSERT (existing `put_context_tx`) | uniqueness |
492: | W5 | `managed_phase_operations` 32-column exact UPDATE | exact permit; core trigger |
493: | W6 | `records` Workflow exact UPDATE | exact permit; identity/version trigger |
494: | W7 | `audit` `phase_closed` INSERT (`sequence` allocated under TX) | exact permit; private, no-replace and chain triggers (need W5 and W6 first) |
495: | W8 | `task_drivers` exact UPDATE (marker cleared) | exact permit (`binding_driver_UPDATE`) |
496: | — | existing non-reserved `execution.result_published` event | unchanged semantics |
497: 
498: W5–W8 run inside ONE `with_exact_permit` window of four rows (≤128), followed by `ensure_consumed`, then commit. The following are never written: Project, Goal, Session, owners, inputs, admissions, readiness, Source7, locks, `task_execution`, `cleanup_jobs` and quota rows.
499: 
500: ### 10.4 Post-commit Driver publication
501: 
502: Still holding the Store guard and admission, run `publish_success_driver`: an exact committed-row read, then the SAME association's `publish_exact`. It returns the `SuccessClosureAcknowledgment` with `driver_published` = the outcome.
503: 
504: - **On failure:** the acknowledgment and the plan are retained. A later turn by the Root sweep or the SAME live worker re-runs only `publish_success_driver` (idempotent per `publish_exact`).
505: - **After an actual worker exit:** Held. Nothing is revived, and no cache is published from rows.
```


### H-uncertain

H `0c35653c7a8aa05c78c5285c2f4b5370db520e70` · `doc/design/issue-43-native-success-continuation-design.md` · lines 517–532 · blob `fd9010eb78c3b7f91dad165447e754166a7daaa7` · object SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6` · excerpt SHA-256 `c6c8bcf1e6cde0d0bbef30b8aa30e37a40c96fcc4786a2638c695cfdb2b01963`.

```text
517: ## 11. Uncertain commits, repeated delivery, cancellation and stop (P)
518: 
519: - **Uniform uncertainty rule.** Every new write (binding, claim, observed, closure) returns `Known`, `Conflict` or `Err` (uncertain). After `Err`, only the SAME retained plan is confirmed:
520:   - **Known** iff every postimage is exact and exactly one link with the plan's `at` and data exists;
521:   - **RolledBack** iff every preimage is exact and the link is absent;
522:   - **otherwise Held**, with a read-only re-probe every 5 s.
523: 
524:   After RolledBack the SAME plan, `at` and digests are retried. A `Conflict` drops the plan, and the next turn replans from the SAME proofs; drift is never laundered, because plans derive from the original and settlement images. Link presence alone is never acknowledgment.
525: - **Repeated delivery.** The Driver and the Root sweep share the stage's single-flight token. A duplicate step sees Known through confirmation. Trigger uniqueness, the Workflow version CAS and the per-kind allowances make a second link impossible.
526: - **Dropped futures.**
527:   - A dropped start future: the job retains the outcome and binding.
528:   - A dropped Driver future after a retained write: the Root sweep confirms it.
529:   - A Driver dropped during gate evaluation: Held (§6.2).
530:   - A Driver cancelled between claim and observed: the claim is retained, Evaluating; Held.
531: - **Stop.** Shutdown and `Runtime::drop` revoke the worker, so `validate_driver_live_tx` refuses later writes. Closure holds `control_admission` across commit and Driver publication, so stop and commit linearize as in CA §5.7.
532: - **Restart.** A new epoch never reconstructs `SettledPhase` or plans. Open operations stay Held; closed ones stay closed. RestoreFactualBinding is absent, and nothing is inferred from timeouts.
```


### H-cost

H `0c35653c7a8aa05c78c5285c2f4b5370db520e70` · `doc/design/issue-43-native-success-continuation-design.md` · lines 534–552 · blob `fd9010eb78c3b7f91dad165447e754166a7daaa7` · object SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6` · excerpt SHA-256 `bb77a9e0a9829b63c3b5fba98671990e57c3ac41f671343cca68a1e6283a1766`.

```text
534: ## 12. Finite costs (P)
535: 
536: All figures are encoded lengths taken from existing bounds. They are not heap or RSS bounds, and control M1 measures them. There is no truncation: an over-bound value refuses (Held).
537: 
538: | Transaction | Mandatory surfaces (each existing bound) |
539: | --- | --- |
540: | Late bind | The binder's surfaces (P/G/T ≤3×8 MiB, Workflow ≤8 MiB old/new, locks ≤256×16 KiB, Context ≤8 MiB, own Session ≤4 MiB, negative identities ≤4096×16 KiB, owner ≤32 KiB, invocation ≤`INVOCATION_BYTES`) plus receipt ≤`RECEIPT_BYTES`, admission ≤8192 B and readiness ≤4096 B; one link ≤4096 B |
541: | Settled helper | Projection set above (no Session or identity scan); one `managed_effects` insert ≤8 KiB |
542: | `gate_claim` | Projection set + Workflow old/new ≤2×8 MiB + link + `charged_scope_bytes` aggregates |
543: | `gate_observed` | as claim + observation ≤64 KiB |
544: | Closure | Projection set; Workflow old/new ≤2×8 MiB; Task old/new ≤2×1 MiB; Context new ≤8 MiB; artifact ≤2×128 KiB; Unit ≤2×44 KiB; operation ≤2×4 MiB; Driver ≤2×128 KiB; effects ≤257 rows ×8 KiB; link ≤4096 B; four permit rows |
545: | Driver publication | one Driver row ≤128 KiB |
546: 
547: - **Mandatory closure reservation.** `SUCCESS_CLOSURE_HEADROOM` = 8 MiB (Context) + 8 MiB (maximum Workflow postimage) + 1 MiB (Task) + 512 KiB (artifact, Unit, Driver, receipt record) = 17.5 MiB. `gate_claim` refuses unless the headroom fits. This is in addition to the existing link reserve (1 MiB − spent while `phase_open=1`), which closure releases by derivation.
548: - **Ledger.** PR1 spends 4 links (bind, claim, observed, closure) of 256. Diagnostics and hold allowances are untouched, and the `phase_closed` slot is never consumed early.
549: - **Retained memory.**
550:   - The binding plan (existing: Workflow after ≤8 MiB + Session ≤4 MiB) stays at most one per job.
551:   - The success stage retains compact plans plus at most ONE closure Context raw (≤8 MiB) and the observation (≤64 KiB).
552:   - Worst case: ≤128 jobs × (12 + 8.1) MiB ≈ 2.5 GiB encoded. That is finite, but large. M1 reports measured values, and AUTHOR_NOTES records the policy question.
```


### H-lock

H `0c35653c7a8aa05c78c5285c2f4b5370db520e70` · `doc/design/issue-43-native-success-continuation-design.md` · lines 555–569 · blob `fd9010eb78c3b7f91dad165447e754166a7daaa7` · object SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6` · excerpt SHA-256 `a5bb94cb45aeeabaa74cccb993e64403648e434bec6b36213d75b074b1943856`.

```text
555: ## 13. Locks and lifetime (P)
556: 
557: | Held | May take |
558: | --- | --- |
559: | `PhaseJobs.entries` | nothing new (existing `start` only) |
560: | `job.state`, `SuccessContinuation.stage`, `success_cursor` | nothing (leaves) |
561: | `PhaseHandoffs.entries` | `slot.handoff` → `assets` (existing order) |
562: | Sources slot (tokio) | SharedStore briefly, never across an await (existing `frame()` discipline); Git awaits hold no Store |
563: | `control_admission` (await) | SharedStore (synchronous), then the permit manager, then the association `binding` mutex |
564: | SharedStore | the permit manager leaf; custody and actor leaves only through existing pointer checks |
565: 
566: - **Store calls.** No Root mutex is held across a Store call, and no await happens under SharedStore.
567: - **Heavy work.** Material, Context and plans are dropped after the Store guard (RN's `borrowed_store_turn`).
568: - **Debug assertions.** The existing debug `ROOT_LOCK_DEPTH` and per-turn transaction counters are reused for every new port.
569: - **Cycles.** No ownership cycle (§4 graph).
```


### H-controls

H `0c35653c7a8aa05c78c5285c2f4b5370db520e70` · `doc/design/issue-43-native-success-continuation-design.md` · lines 621–650 · blob `fd9010eb78c3b7f91dad165447e754166a7daaa7` · object SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6` · excerpt SHA-256 `8b603004bf46e8e0aeeed8406a969b51c70e92d5d2f3e63de6ed3541c4d24a29`.

```text
621: | SC1 | Claude and Codex, QUICK Implement, commit mode. The stage sequence is held start → normal `Bound` → release `fixture-release` → owned success → capture → claim → Passed → observed → closure. Assertions: <br>• Task.version equal to the marker until closure, then +1 exactly once; <br>• links `[session_bound, gate_claim, gate_observed, phase_closed]`; <br>• artifact Published, with revision = fixture HEAD ≠ base and `results.git` refs and fsck; <br>• Unit finalization closed; <br>• exactly one new Context; <br>• the Driver row marker-free and the generic `validate` passing; <br>• the next step Waiting (`commit`), with the worker still driving. |
622: | SC2 | Late. A cfg(test) seam delays `bind_returned` until the settlement exists. Normal refuses (`Conflict`); the Root sweep late-binds with `proof_source=closed_settlement` and `private_receipt_ref`; then SC1's assertions. Cleanup Unknown is present and binding still happens once. |
623: | SC3 | NotDispatched. Normal binding is forced to `Conflict` by genuine writer-lock contention while the owner is live and pre-ACK. Late planning refuses (no settlement); the later normal retry binds `normal_return`. |
624: | SC4 | Uncertain commits. A failpoint returns `Err` after commit, and separately before commit, in each of: bind, claim, observed, closure. Assertions: the confirmation is Known or RolledBack; the SAME `at`/digests; exactly one link of each kind; L-x variants (a C1 drift after commit) are Held. |
625: | SC5 | Lost notifications. A seam suppresses job watch sends; binding still converges via the service timer within ⌈n/8⌉ sweeps plus backoff. |
626: | SC6 | STANDARD Requirements. Bind → capture (Ready) → claim → observed Waiting (`evidence_integration_unavailable`). After ≥50 Driver polls: still one claim, no `phase_closed`, Task unchanged, Requirements not completed, artifact not Published. |
627: | SC7 | Negatives after genuine setup: <br>• Goal or Task edited through an existing legitimate writer during the held start → Conflict/Held and zero links; <br>• `ordinary-failure` scenario → no settlement, Held "bound non-success closure unavailable"; <br>• peer killed before terminal → Unknown/Lost → no late bind; <br>• Runtime restart → Held and no reconstruction. |
628: | SC8 | Graph integrity. Between capture and closure, a negative-only corruption of a `results.git` object → `settled_publication` refuses and closure is never committed. |
629: | SC9 | Stop. Shutdown while closure waits for admission → no commit. Shutdown after commit with publication pending → Held, counted pending, no revival. |
630: | SC10 | Four Tasks across two Projects (two Claude, two Codex) in commit mode → four independent closures, with no cross-job acknowledgment or Driver publication. |
631: | M1 | Near-bound bodies (Workflow and Context ≈8 MiB, 256 locks): report measured retained and per-turn peaks against §12 terms. Fewer than the genuine bound is SETUP. |
632: 
633: | Mutant (compiled) | Killed by (first assertion) |
634: | --- | --- |
635: | Restore the Task write in binding, claim or observed | SC1: Task.version unchanged before closure |
636: | Late planner accepts settlement None or a live owner | SC3: a `closed_settlement` link before any terminal |
637: | Late planner skips the receipt/admission/readiness exact rows | defense only (the issuer guarantees them; no genuine producer of a mismatch) |
638: | Confirm accepts a preimage as Known (any port) | SC4 pre-commit: acknowledgment while the link is absent |
639: | Replan with a new `at` after uncertainty | SC4: link `at` or digest differs from the SAME plan |
640: | Root `reconcile_success` call removed | SC2: no binding |
641: | Capture uses the generic `validate_execution` | SC1: capture refusal ("marker-bound Driver…"), no Ready artifact |
642: | Protected validator omits `validate_driver_live_tx` | SC9 variant: a helper intent row after revocation |
643: | Gate Waiting rewrites Task WaitingHuman/blocker | SC6: Task changed |
644: | Automatic re-claim after Waiting | SC6: second `gate_claim` |
645: | Requirements Waiting mapped to Passed | SC6: `phase_closed` present |
646: | Closure skips `settled_publication` graph verification | SC8: closure committed |
647: | Closure omits phase_open 1→0 | SC1: chain trigger refuses (trigger kill) |
648: | Closure omits the Driver re-anchor or its publication | SC1: generic `validate` / association `is_current` false after closure |
649: | Post-closure Waiting branch removed | SC1: worker exits with Err |
650: | Headroom check removed | M1 near-bound claim admitted then closure refused; SETUP if near-bound bodies cannot be produced genuinely |
```


### H-open

H `0c35653c7a8aa05c78c5285c2f4b5370db520e70` · `doc/design/issue-43-native-success-continuation-design.md` · lines 672–680 · blob `fd9010eb78c3b7f91dad165447e754166a7daaa7` · object SHA-256 `2022e85329253dfbbb7ae5f84f22e0b8d32f187337395614a8b3d38afeed4af6` · excerpt SHA-256 `9a5ee6e239244ba07c36a4c126f7bdc89788576a361fefbb4fe930e7680a31e8`.

```text
672: 
673: - **Composition.** Composition R+CA+RN-final, reviewed (§1 item 8).
674: - **SC-N.** Post-closure Evidence-phase Driver continuation (Commit, Tests, Pr, MergeGate, Cleanup) under the protected Workflow guard.
675: - **Requirements/Design evidence.** Requirements and Design evidence integrations, plus an explicit typed gate-resume lane.
676: - **Bound non-success closure.** Gate Failed/unknown; Native failure, Unknown or capacity after Bound. Also `native_diagnostic` and `gate_hold`/clear ports.
677: - **PR-2 to PR-4** (later, retry and Reviewer offers; ReviewerArtifactLease; artifact retention). ReviewEngine fan-out.
678: - **Restart and quotas.** RestoreFactualBinding / restart migration; native quotas and cancellation; scheduler progress.
679: - **Native registry.** Release of the phase Entry (an unbounded-over-time Entry growth risk before long dogfood).
680: - **Source7.** Source7-anchored success closure.
```


### job-shape

RN `1d783440b7f4d705bf8cf4c667e022f5cec87777` · `crates/rrx/src/runtime/phase_jobs.rs` · lines 132–181 · blob `73efd9255e56d2a5539dca4823b0bacaeb36e399` · object SHA-256 `ce58e32049e06847295d3ba6933354ea62ed4cfedf46badea8c3bd1c6af494fb` · excerpt SHA-256 `fd08d4dca29064e352065023deebb0684a6dc800b5b6f0b399b0ee1d16f77fea`.

```rust
132: struct JobState {
133:     // EMPTY/nongrant until the SAME actual selected Native start installs its
134:     // own actor and original plan. Retained BEFORE marker/start/future effects.
135:     preparation: Arc<NativePreparationCustody>,
136:     observation: InvocationObservation,
137:     launch: Option<Arc<PhaseLaunchParts>>,
138:     outcome: Option<std::result::Result<RetainedStart, NativePhaseStartError>>,
139:     binding_plan: Option<Arc<ManagedBindingPlan>>,
140:     binding_error: Option<anyhow::Error>,
141:     nonsuccess: Option<Arc<crate::state::NativeNonSuccessClosurePlan>>,
142:     closed_ack: Option<Arc<crate::state::PhaseClosedAcknowledgment>>,
143:     closure_due: Instant,
144:     closure_backoff: u64,
145:     uncertain: bool,
146:     slot_released: bool,
147:     attention: Option<&'static str>,
148: }
149: /// Actual returned objects, never reconstructed from DTOs or registry IDs.
150: enum RetainedStart {
151:     Launched {
152:         _handle: ManagedSessionRef,
153:         binding: Arc<NativePhaseBinding>,
154:     },
155: }
156: struct Job {
157:     allocation: Arc<NativeAllocation>,
158:     state: Mutex<JobState>,
159:     changed: watch::Sender<InvocationObservation>,
160: }
161: struct Entry {
162:     job: Arc<Job>,
163:     // Independent registry ownership, never a strong return edge from Job.
164:     handle: Option<JoinHandle<()>>,
165: }
166: 
167: /// Produced by the actual reservation, retaining SAME entry identity without
168: /// a job -> launch -> reservation -> job ownership cycle. Reuse never grants
169: /// this call permission to remove somebody else's existing reservation.
170: pub(super) struct PhaseJobReservation {
171:     job: Weak<Job>,
172:     allocation: Arc<NativeAllocation>,
173:     fresh: bool,
174: }
175: 
176: /// Runtime-owned sibling of PhaseSupervisor. Neither slots nor jobs own it.
177: #[derive(Default)]
178: pub(super) struct PhaseJobs {
179:     entries: Mutex<BTreeMap<OperationId, Entry>>,
180:     closure_cursor: Mutex<Option<OperationId>>,
181: }
```


### job-return

RN `1d783440b7f4d705bf8cf4c667e022f5cec87777` · `crates/rrx/src/runtime/phase_jobs.rs` · lines 683–723 · blob `73efd9255e56d2a5539dca4823b0bacaeb36e399` · object SHA-256 `ce58e32049e06847295d3ba6933354ea62ed4cfedf46badea8c3bd1c6af494fb` · excerpt SHA-256 `a35d30f1ea68e60a48b83fe5932f4444fc46b21a0765a0d23a7151c4c4709495`.

```rust
683:             let outcome = result.map(|start| match start {
684:                 NativePhaseStart::Launched { handle, binding } => RetainedStart::Launched {
685:                     _handle: handle,
686:                     binding: Arc::from(binding),
687:                 },
688:             });
689:             let refused = outcome.is_err();
690:             let (observation, binding) = match &outcome {
691:                 Ok(RetainedStart::Launched { binding, .. }) => {
692:                     (InvocationObservation::Binding, Some(binding.clone()))
693:                 }
694:                 Err(_) => (InvocationObservation::Failed, None),
695:             };
696:             {
697:                 let mut state = job.state.lock().unwrap_or_else(|e| e.into_inner());
698:                 state.outcome = Some(outcome);
699:                 state.observation = observation;
700:             }
701:             if refused {
702:                 // Actual outcome is already retained. Nongrant abandonment must
703:                 // not hold the Root job mutex or retire Unit/Task from an error.
704:                 preparation.abandon();
705:             }
706:             job.changed.send_replace(observation);
707:             if let Some(binding) = binding {
708:                 // Both the actual proof and handle are already retained. No
709:                 // fallible planning or Store access can consume their sole owner.
710:                 let result = job.bind_returned(binding);
711:                 let observation = if result.is_ok() {
712:                     InvocationObservation::Bound
713:                 } else {
714:                     InvocationObservation::BindingHeld
715:                 };
716:                 {
717:                     let mut state = job.state.lock().unwrap_or_else(|e| e.into_inner());
718:                     state.binding_error = result.err();
719:                     state.observation = observation;
720:                 }
721:                 job.changed.send_replace(observation);
722:             }
723:         }));
```


### job-bind

RN `1d783440b7f4d705bf8cf4c667e022f5cec87777` · `crates/rrx/src/runtime/phase_jobs.rs` · lines 809–855 · blob `73efd9255e56d2a5539dca4823b0bacaeb36e399` · object SHA-256 `ce58e32049e06847295d3ba6933354ea62ed4cfedf46badea8c3bd1c6af494fb` · excerpt SHA-256 `5831dcf1891e5ed541550c07b38df5aae49ab6314a6082f9d287b422608ee959`.

```rust
809:     fn bind_returned(&self, proof: Arc<NativePhaseBinding>) -> Result<()> {
810:         let owner = self.allocation.selected_port().owner();
811:         let plan = Arc::new(plan_managed_binding(owner, proof)?);
812:         {
813:             let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
814:             ensure!(
815:                 state.binding_plan.is_none(),
816:                 "binding plan already retained"
817:             );
818:             state.binding_plan = Some(plan.clone());
819:         }
820:         // Short job locks above never overlap the selected owner's Store lock.
821:         owner
822:             .store
823:             .lock()
824:             .map_err(|_| anyhow::anyhow!("binding Store poisoned"))?
825:             .bind_managed_phase(&plan)?;
826:         Ok(())
827:     }
828: }
829: 
830: enum TurnResult {
831:     Known(crate::state::PhaseClosedAcknowledgment),
832:     Conflict(anyhow::Error),
833:     RolledBack,
834: }
835: 
836: struct RunningJob(Arc<Job>);
837: impl Drop for RunningJob {
838:     fn drop(&mut self) {
839:         let preparation = {
840:             let mut state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
841:             if state.outcome.is_none() || state.observation == InvocationObservation::Binding {
842:                 state.observation = InvocationObservation::Uncertain;
843:                 self.0
844:                     .changed
845:                     .send_replace(InvocationObservation::Uncertain);
846:                 Some(state.preparation.clone())
847:             } else {
848:                 None
849:             }
850:         };
851:         if let Some(preparation) = preparation {
852:             preparation.abandon();
853:         }
854:     }
855: }
```


### binding-snapshot

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/native/phase_protocol.rs` · lines 398–429 · blob `26e8aec7f42316ea3f0e5828ff3d4e287a6af372` · object SHA-256 `96b3fa95c27c776be5f8d8991bbb5f9add3276554738f335493cdc9934d4fee8` · excerpt SHA-256 `5436e3067f16a111452603025ac51ea4dc6217971d2eaaafa0a355ea458889ab`.

```rust
398:     pub(super) fn revoke(&self) {
399:         self.state.store(REVOKED, Ordering::SeqCst);
400:     }
401:     pub(crate) fn is_live(&self) -> bool {
402:         self.state.load(Ordering::SeqCst) == LIVE && self.ack.get().is_some()
403:     }
404:     pub(crate) fn registration_ack(&self) -> Option<RegistrationAck> {
405:         self.ack.get().copied()
406:     }
407:     pub(crate) fn origin(&self) -> &Arc<NativeTransportStartPlan> {
408:         &self.origin
409:     }
410:     pub(crate) fn validate_known_registration(&self) -> Result<RegistrationAck> {
411:         known_ack(&self.state, &self.ack)
412:     }
413:     pub(crate) fn registered_readiness(&self) -> Result<u64> {
414:         self.origin.registered_readiness()
415:     }
416:     pub(crate) fn binding_snapshot(self: &Arc<Self>) -> Result<NativePhaseBinding> {
417:         let projection = self
418:             .projection
419:             .lock()
420:             .map_err(|_| anyhow::anyhow!("native phase projection unavailable"))?;
421:         Ok(NativePhaseBinding {
422:             owner: self.clone(),
423:             session: projection.session.clone(),
424:             record_version: projection.record_version,
425:             consumed: projection.consumed.as_ref().and_then(Weak::upgrade),
426:             settlement: projection.settlement.as_ref().and_then(Weak::upgrade),
427:             live: self.is_live(),
428:         })
429:     }
```


### binding-access

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/native/phase_protocol.rs` · lines 430–458 · blob `26e8aec7f42316ea3f0e5828ff3d4e287a6af372` · object SHA-256 `96b3fa95c27c776be5f8d8991bbb5f9add3276554738f335493cdc9934d4fee8` · excerpt SHA-256 `6962faacd291b855efb9970fe986a7b455749b462f22614d79c3c2b78c7702c0`.

```rust
430: }
431: impl NativePhaseBinding {
432:     pub(crate) fn owner_arc(&self) -> &Arc<NativePhaseSession> {
433:         &self.owner
434:     }
435:     pub(crate) fn owner(&self) -> &NativePhaseSession {
436:         &self.owner
437:     }
438:     pub(crate) fn marker(&self) -> &OriginalMarker {
439:         self.owner.marker()
440:     }
441:     pub(crate) fn allocation(&self) -> &NativeAllocation {
442:         self.owner.allocation()
443:     }
444:     pub(crate) fn session(&self) -> &Session {
445:         &self.session
446:     }
447:     pub(crate) fn record_version(&self) -> u64 {
448:         self.record_version
449:     }
450:     pub(crate) fn consumed(&self) -> Option<&ConsumedPhaseInput> {
451:         self.consumed.as_deref()
452:     }
453:     pub(crate) fn settlement(&self) -> Option<&OwnedPhaseSettlement> {
454:         self.settlement.as_deref()
455:     }
456:     pub(crate) fn is_live(&self) -> bool {
457:         self.live && self.owner.is_live()
458:     }
```


### settlement-shape

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/native/phase_protocol.rs` · lines 301–315 · blob `26e8aec7f42316ea3f0e5828ff3d4e287a6af372` · object SHA-256 `96b3fa95c27c776be5f8d8991bbb5f9add3276554738f335493cdc9934d4fee8` · excerpt SHA-256 `c8b19a3bc4a7cf3152bbdd629adac8f845c0719fac6e27a164e01af1a029bcc7`.

```rust
301:     }
302: }
303: /// Only an actual saved NativeTerminal followed by successful own logical
304: /// closure can issue this. It is not a review opinion or a cleanup guarantee.
305: pub(crate) struct OwnedPhaseSettlement {
306:     owner: Arc<NativePhaseSession>,
307:     consumed: Arc<ConsumedPhaseInput>,
308:     terminal: Arc<NativeTerminal>,
309:     unit: ExecutionUnit,
310:     receipt: native_result::NativeResultReceipt,
311:     session: Session,
312:     record_version: u64,
313:     thread: String,
314:     turn: Option<String>,
315: }
```


### settlement-issuer

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/native/phase_protocol.rs` · lines 553–641 · blob `26e8aec7f42316ea3f0e5828ff3d4e287a6af372` · object SHA-256 `96b3fa95c27c776be5f8d8991bbb5f9add3276554738f335493cdc9934d4fee8` · excerpt SHA-256 `ec32b19509b1cc89f326efbbba4bb076d2fa7cdb49f47dde013068c9bf2e1a68`.

```rust
553:     pub(super) fn completed(
554:         consumed: Arc<ConsumedPhaseInput>,
555:         terminal: Arc<NativeTerminal>,
556:         unit: ExecutionUnit,
557:         receipt: native_result::NativeResultReceipt,
558:         session: Session,
559:         record_version: u64,
560:     ) -> Result<Arc<Self>> {
561:         let owner = consumed.owner.clone();
562:         let facts = owner.allocation().facts();
563:         // The actual terminal transaction changes only receipt authority. Keep
564:         // every bounded answer/protocol/diagnostic byte tied to that SAME saved
565:         // terminal instead of accepting matching selected hashes or public IDs.
566:         let mut expected_receipt = terminal.receipt().clone();
567:         expected_receipt.authority = native_result::ReceiptAuthority::OwnedTerminal;
568:         ensure!(
569:             terminal.receipt().observed_work == WorkOutcome::Success
570:                 && receipt == expected_receipt
571:                 && receipt.observed_work == WorkOutcome::Success
572:                 && receipt.authority == native_result::ReceiptAuthority::OwnedTerminal
573:                 && receipt.invocation_id == facts.invocation_id
574:                 && receipt.unit_id == facts.unit_id
575:                 && receipt.session_id == facts.session_id
576:                 && receipt.scope == *facts.scope
577:                 && receipt.generation == facts.generation
578:                 && receipt.owner_epoch == facts.epoch
579:                 && receipt.provider == facts.provider
580:                 && unit.id == facts.unit_id
581:                 && unit.scope == *facts.scope
582:                 && unit.generation == facts.generation
583:                 && unit.owner_epoch == facts.epoch
584:                 && unit.provider == facts.provider
585:                 && unit.worktree == facts.path
586:                 && unit.profile_digest == facts.profile_digest
587:                 && unit.work == Some(WorkOutcome::Success)
588:                 && !unit.native_effects_open
589:                 && unit.state == UnitState::WorkKnown
590:                 && unit.session_id == Some(facts.session_id)
591:                 && session.id == facts.session_id
592:                 && session.scope == *facts.scope
593:                 && session.agent == facts.alias
594:                 && session.provider == facts.provider
595:                 && session.role == facts.role
596:                 && session.worktree == facts.path
597:                 && session.model.as_deref() == facts.model
598:                 && session.effort.as_deref() == facts.effort
599:                 && session.state == SessionState::Exited,
600:             "native phase lacks actual owned logical success"
601:         );
602:         let (thread, turn) = {
603:             let acknowledgement = consumed
604:                 .acknowledgement
605:                 .lock()
606:                 .map_err(|_| anyhow::anyhow!("native phase acknowledgement unavailable"))?;
607:             let ack = acknowledgement
608:                 .as_ref()
609:                 .context("native phase input lacks actual acknowledgement")?;
610:             ensure!(
611:                 receipt.native_thread.as_deref() == Some(ack.thread.as_str())
612:                     && receipt.native_turn == ack.turn
613:                     && session.native_ref.as_deref() == Some(ack.thread.as_str()),
614:                 "native phase terminal acknowledgement changed"
615:             );
616:             (ack.thread.clone(), ack.turn.clone())
617:         };
618:         owner.project(&session, record_version)?;
619:         owner.revoke();
620:         let settlement = Arc::new(Self {
621:             owner: owner.clone(),
622:             consumed,
623:             terminal,
624:             unit,
625:             receipt,
626:             session,
627:             record_version,
628:             thread,
629:             turn,
630:         });
631:         let mut projection = owner
632:             .projection
633:             .lock()
634:             .map_err(|_| anyhow::anyhow!("native phase projection unavailable"))?;
635:         ensure!(
636:             projection.settlement.is_none(),
637:             "native phase terminal already projected"
638:         );
639:         projection.settlement = Some(Arc::downgrade(&settlement));
640:         Ok(settlement)
641:     }
```


### settlement-access

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/native/phase_protocol.rs` · lines 642–672 · blob `26e8aec7f42316ea3f0e5828ff3d4e287a6af372` · object SHA-256 `96b3fa95c27c776be5f8d8991bbb5f9add3276554738f335493cdc9934d4fee8` · excerpt SHA-256 `9be56dd2fc727e9792d6aebf25957129ccce7782fbd42377327ac9d421c8a0f2`.

```rust
642:     pub(crate) fn marker(&self) -> &OriginalMarker {
643:         self.owner.marker()
644:     }
645:     pub(crate) fn allocation(&self) -> &NativeAllocation {
646:         self.owner.allocation()
647:     }
648:     pub(crate) fn consumed(&self) -> &ConsumedPhaseInput {
649:         &self.consumed
650:     }
651:     pub(crate) fn terminal(&self) -> &NativeTerminal {
652:         &self.terminal
653:     }
654:     pub(crate) fn unit(&self) -> &ExecutionUnit {
655:         &self.unit
656:     }
657:     pub(crate) fn receipt(&self) -> &native_result::NativeResultReceipt {
658:         &self.receipt
659:     }
660:     pub(crate) fn session(&self) -> &Session {
661:         &self.session
662:     }
663:     pub(crate) fn record_version(&self) -> u64 {
664:         self.record_version
665:     }
666:     pub(crate) fn thread(&self) -> &str {
667:         &self.thread
668:     }
669:     pub(crate) fn turn(&self) -> Option<&str> {
670:         self.turn.as_deref()
671:     }
672: }
```


### terminal-shape

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/execution/native_phase/terminal.rs` · lines 116–169 · blob `703082c30db807852839d3108341a18ca97dc90c` · object SHA-256 `37838036a6c0c45dd911c0e4e0a64ff53eeb9fc8503c995013f1f37060bbbf1d` · excerpt SHA-256 `5175e2282100f5ee85edad6e3d14da697121f42d50bb0b4ee2cd64a206cdb571`.

```rust
116: pub(crate) struct NativeTerminalPlan {
117:     phase: Arc<NativePhaseSession>,
118:     terminal: Arc<NativeTerminal>,
119:     normal: Option<NativeOwnerPlan>,
120:     unit: UnitImage,
121:     unit_after: UnitImage,
122:     invocation: InvocationImage,
123:     invocation_after: InvocationImage,
124:     session: PairRow,
125:     session_after: PairRow,
126:     closed_session: Session,
127:     owner: PairRow,
128:     readiness: PairRow,
129:     readiness_after: PairRow,
130:     admission: Option<PairRow>,
131:     admission_after: Option<PairRow>,
132:     receipt: NativeResultReceipt,
133:     receipt_raw: String,
134:     session_version: u64,
135:     input_effect: Option<(OperationId, String, u64, EffectState)>,
136: }
137: 
138: /// Only a known transaction/confirmation of the SAME retained plan creates this.
139: /// It is consumed by the actual actor, not serialized or used as review approval.
140: pub(crate) struct NativeTerminalCommit {
141:     terminal: Arc<NativeTerminal>,
142:     phase: Arc<NativePhaseSession>,
143:     unit: ExecutionUnit,
144:     receipt: NativeResultReceipt,
145:     session: Session,
146:     version: u64,
147:     owned_success: bool,
148: }
149: impl NativeTerminalCommit {
150:     pub(crate) fn into_parts(
151:         self,
152:     ) -> (
153:         Arc<NativeTerminal>,
154:         Arc<NativePhaseSession>,
155:         ExecutionUnit,
156:         NativeResultReceipt,
157:         Session,
158:         u64,
159:         bool,
160:     ) {
161:         (
162:             self.terminal,
163:             self.phase,
164:             self.unit,
165:             self.receipt,
166:             self.session,
167:             self.version,
168:             self.owned_success,
169:         )
```


### terminal-progress

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/execution/native_phase/terminal.rs` · lines 472–595 · blob `703082c30db807852839d3108341a18ca97dc90c` · object SHA-256 `37838036a6c0c45dd911c0e4e0a64ff53eeb9fc8503c995013f1f37060bbbf1d` · excerpt SHA-256 `7718159610f663c1ec08358ba834baf7e01f8ef8cf95289312c9727c0f924743`.

```rust
472:     let mut after = unit.value.clone();
473:     if after.native_effects_open || after.work.is_none() {
474:         if after.work.is_none() {
475:             after.work = Some(if owned {
476:                 receipt.observed_work
477:             } else {
478:                 WorkOutcome::Unknown
479:             });
480:             after.disposition = if owned
481:                 || (receipt.observed_work == WorkOutcome::Unknown
482:                     && receipt.disposition != Disposition::Completed)
483:             {
484:                 receipt.disposition
485:             } else {
486:                 Disposition::Lost
487:             };
488:             after.state = if after.work == Some(WorkOutcome::Unknown) {
489:                 UnitState::WorkUnknown
490:             } else {
491:                 UnitState::WorkKnown
492:             };
493:         }
494:         after.native_effects_open = false;
495:         if !owned || after.disposition != Disposition::Completed {
496:             after.result_finalization_open = false;
497:         }
498:         after.wait_reason = match after.disposition {
499:             Disposition::QuotaInterrupted => Some(WaitReason::Quota),
500:             Disposition::CapacityInterrupted => Some(WaitReason::Capacity),
501:             _ => None,
502:         };
503:         after.capacity_retry_at = (after.disposition == Disposition::CapacityInterrupted)
504:             .then(|| now_ms().saturating_add(60_000));
505:         after.version = after
506:             .version
507:             .checked_add(1)
508:             .context("terminal Unit version exhausted")?;
509:         after.updated_at = now_ms();
510:     }
511:     let unit_after = UnitImage::encoded(after)?;
512:     let mut n = invocation.value.clone();
513:     // This is only factual projection of the retained actual ACK. An unconfirmed
514:     // input remains historical and cannot become an OwnedPhaseSettlement.
515:     n.native_thread = receipt.native_thread.clone();
516:     n.native_turn = receipt.native_turn.clone();
517:     n.state = InvocationState::Closed;
518:     n.version = n
519:         .version
520:         .checked_add(1)
521:         .context("terminal invocation version exhausted")?;
522:     let invocation_after = InvocationImage::encoded(n)?;
523:     let mut record: Record = serde_json::from_value(session.body()?)?;
524:     let mut closed_session: Session = serde_json::from_value(record.data.clone())?;
525:     if closed_session.native_ref.is_none() {
526:         closed_session.native_ref = receipt.native_thread.clone();
527:     }
528:     let mut session_after = PairRow {
529:         table: session.table,
530:         values: session.values.clone(),
531:     };
532:     if !session_terminal(closed_session.state) && closed_session.state != SessionState::Lost {
533:         closed_session.state = if unit_after.value.disposition == Disposition::Cancelled {
534:             SessionState::Stopped
535:         } else if unit_after.value.work == Some(WorkOutcome::Failure) {
536:             SessionState::Failed
537:         } else if unit_after.value.work == Some(WorkOutcome::Unknown) {
538:             SessionState::Lost
539:         } else {
540:             SessionState::Exited
541:         };
542:     }
543:     if serde_json::to_value(&closed_session)? != record.data {
544:         record.version = record
545:             .version
546:             .checked_add(1)
547:             .context("terminal Session version exhausted")?;
548:         record.updated_at = now_ms();
549:         record.data = serde_json::to_value(&closed_session)?;
550:         session_after.replace("version", SqlValue::Integer(i64::try_from(record.version)?))?;
551:         session_after.set_body(&serde_json::to_value(&record)?)?;
552:     }
553:     let mut readiness_after = PairRow {
554:         table: readiness.table,
555:         values: readiness.values.clone(),
556:     };
557:     let mut body = readiness_after.body()?;
558:     let version = body["version"]
559:         .as_u64()
560:         .context("readiness version absent")?
561:         .checked_add(1)
562:         .context("readiness version exhausted")?;
563:     body["state"] = json!("closed");
564:     body["start_ended"] = json!(true);
565:     body["known_terminal"] = json!(true);
566:     body["version"] = json!(version);
567:     readiness_after.replace("state", SqlValue::Text("closed".into()))?;
568:     readiness_after.replace("start_ended", SqlValue::Integer(1))?;
569:     readiness_after.replace("known_terminal", SqlValue::Integer(1))?;
570:     readiness_after.replace("version", SqlValue::Integer(i64::try_from(version)?))?;
571:     readiness_after.set_body(&body)?;
572:     let admission_after = admission
573:         .as_ref()
574:         .map(|a| -> Result<PairRow> {
575:             let mut next = PairRow {
576:                 table: a.table,
577:                 values: a.values.clone(),
578:             };
579:             let mut body = next.body()?;
580:             let version = body["version"]
581:                 .as_u64()
582:                 .context("admission version absent")?
583:                 .checked_add(1)
584:                 .context("admission version exhausted")?;
585:             let settled = body["confirmed"] == true
586:                 && input_effect
587:                     .as_ref()
588:                     .is_some_and(|e| e.3 == EffectState::Confirmed);
589:             body["settled"] = json!(settled);
590:             body["uncertain"] = json!(!settled);
591:             body["version"] = json!(version);
592:             next.replace("settled", SqlValue::Integer(i64::from(settled)))?;
593:             next.replace("uncertain", SqlValue::Integer(i64::from(!settled)))?;
594:             next.replace("version", SqlValue::Integer(i64::try_from(version)?))?;
595:             next.set_body(&body)?;
```


### terminal-delivery

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/native.rs` · lines 1214–1270 · blob `5648a9559a29cef3d68d134ed54d004b38087bb1` · object SHA-256 `4095b9d0a45a40412c25ea1bdc8a59c4615cf27fad94858bf04b5dcda8388f30` · excerpt SHA-256 `d5ade4c0d687600f5eee5aaee69a4334853a48d3e38e94fd940b5d84842cd4e4`.

```rust
1214:     if let Some(original) = frozen.as_ref() {
1215:         ensure!(
1216:             Arc::ptr_eq(original, proof),
1217:             "native terminal compare-clear origin changed"
1218:         );
1219:         *frozen = None;
1220:     }
1221:     Ok(())
1222: }
1223: fn persist_saved_terminal(
1224:     owner: &RuntimeOwner,
1225:     phase: Option<&Arc<phase_protocol::PhaseActor>>,
1226:     terminal: &Arc<NativeTerminal>,
1227: ) -> Result<(
1228:     ExecutionUnit,
1229:     native_result::NativeResultReceipt,
1230:     Session,
1231:     u64,
1232: )> {
1233:     if let Some(phase) = phase {
1234:         let original = phase.terminal_plan(owner, terminal)?;
1235:         let first = owner
1236:             .store
1237:             .lock()
1238:             .map_err(|_| anyhow::anyhow!("state poisoned"))?
1239:             .finish_phase_terminal(&original);
1240:         let commit = match first {
1241:             Ok(commit) => commit,
1242:             Err(error) => {
1243:                 // A failed commit might actually have committed. Only actual
1244:                 // receipt absence permits replacing its original plan. At most
1245:                 // one confirmed-rollback replan is attempted per observation.
1246:                 if !original.confirmed_absent(owner)? {
1247:                     return Err(error);
1248:                 }
1249:                 let next = phase.replan_terminal_after_absence(owner, &original, terminal)?;
1250:                 owner
1251:                     .store
1252:                     .lock()
1253:                     .map_err(|_| anyhow::anyhow!("state poisoned"))?
1254:                     .finish_phase_terminal(&next)?
1255:             }
1256:         };
1257:         let (saved, actual_owner, unit, receipt, session, version, owned_success) =
1258:             commit.into_parts();
1259:         ensure!(
1260:             Arc::ptr_eq(&saved, terminal) && Arc::ptr_eq(&actual_owner, &phase.owner),
1261:             "native terminal transaction changed private origin"
1262:         );
1263:         if owned_success {
1264:             phase.settled(
1265:                 saved,
1266:                 unit.clone(),
1267:                 receipt.clone(),
1268:                 session.clone(),
1269:                 version,
1270:             )?;
```


### normal-session

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/managed_binding/binding.rs` · lines 75–103 · blob `310a55a2a30ed1dc09ecc7b807561af10428790a` · object SHA-256 `3ec2e6c63efdbea440a7f9f2473ff8af8ed7787aa81cd836ada30d83de915978` · excerpt SHA-256 `fb913ecd41fe74c38bcabdfdb0f4b3560149d186ae32dd16715c2a541ad272cd`.

```rust
75:             && record.version > 0
76:             && record.version <= i64::MAX as u64
77:             && session.id == f.session_id
78:             && session.scope == *f.scope
79:             && session.agent == f.alias
80:             && session.provider == f.provider
81:             && session.role == f.role
82:             && session.worktree == f.path
83:             && session.model.as_deref() == f.model
84:             && session.effort.as_deref() == f.effort
85:             && session.started_at == proof.session().started_at
86:             && proof
87:                 .session()
88:                 .native_ref
89:                 .as_ref()
90:                 .is_none_or(|id| session.native_ref.as_ref() == Some(id))
91:             && matches!(
92:                 session.state,
93:                 SessionState::Starting
94:                     | SessionState::Running
95:                     | SessionState::WaitingApproval
96:                     | SessionState::WaitingHuman
97:             ),
98:         "own latest Session immutable identity or normal eligibility changed"
99:     );
100:     // This current snapshot is not a stale returned Session-version CAS. PID,
101:     // initial native UUID and lifecycle can have advanced before this read.
102:     let exact: bool = c.query_row(
103:         "SELECT EXISTS(SELECT 1 FROM records WHERE id=?1 AND kind='session' AND project_id=?2 AND goal_id=?3 AND task_id=?4 AND version=?5 AND body=?6)",
```


### normal-eligibility

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/managed_binding/binding.rs` · lines 158–223 · blob `310a55a2a30ed1dc09ecc7b807561af10428790a` · object SHA-256 `3ec2e6c63efdbea440a7f9f2473ff8af8ed7787aa81cd836ada30d83de915978` · excerpt SHA-256 `d5950e8f10abbd90f87a684b2ea89ebda29e9197a171ced7194bdbbba171903e`.

```rust
158: fn normal_eligibility(
159:     proof: &NativePhaseBinding,
160:     current: &CurrentWorkflowSuccessor,
161: ) -> Result<()> {
162:     let f = proof.allocation().facts();
163:     let unit = current.unit();
164:     ensure!(
165:         proof.is_live()
166:             && proof.owner().launch_parts().is_retained()
167:             && std::ptr::eq(proof.marker().allocation().as_ref(), proof.allocation())
168:             && proof.settlement().is_none()
169:             && unit.session_id == Some(f.session_id)
170:             && unit.native_effects_open
171:             && unit.result_finalization_open
172:             && unit.work.is_none()
173:             && unit.disposition == Disposition::Active
174:             && matches!(
175:                 unit.state,
176:                 UnitState::DispatchPending | UnitState::Running | UnitState::WaitingQuota
177:             ),
178:         "normal binding lacks genuine live registration; terminal settlement requires its own predicate"
179:     );
180:     Ok(())
181: }
182: 
183: fn binding_invocation(
184:     c: &Connection,
185:     proof: &NativePhaseBinding,
186:     current: &CurrentWorkflowSuccessor,
187: ) -> Result<Body<NativeInvocation>> {
188:     let f = proof.allocation().facts();
189:     let raw: Option<String> = c.query_row(
190:         "SELECT CASE WHEN length(CAST(body AS BLOB))<=?2 THEN body END FROM native_invocations WHERE id=?1",
191:         params![f.invocation_id.to_string(),native_result::INVOCATION_BYTES],|r|r.get(0),
192:     )?;
193:     let body = Body::<NativeInvocation>::decode(
194:         raw.context("registered invocation body over bound")?,
195:         native_result::INVOCATION_BYTES,
196:     )?;
197:     let invocation = body.parsed();
198:     invocation.validate()?;
199:     ensure!(
200:         invocation.id == f.invocation_id
201:             && invocation.unit_id == f.unit_id
202:             && invocation.session_id == f.session_id
203:             && invocation.scope == *f.scope
204:             && invocation.generation == f.generation
205:             && invocation.owner_epoch == f.epoch
206:             && invocation.provider == f.provider
207:             && invocation.state != InvocationState::Closed
208:             && invocation.unit_version >= f.unit_version
209:             && invocation.unit_version <= current.unit().version
210:             && invocation.context_version == Some(f.input.version)
211:             && invocation.context_sha256.as_deref()
212:                 == Some(
213:                     native_result::digest(&serde_json::to_vec(
214:                         proof.marker().original_plan().context().0
215:                     )?)
216:                     .as_str()
217:                 )
218:             && invocation.source_versions == f.input.source_versions
219:             && invocation.source_sha256
220:                 == native_result::digest(&serde_json::to_vec(&f.input.source_versions)?)
221:             && invocation.revision == f.input.revision
222:             && invocation.artifact_id == f.artifact
223:             && invocation.payload_sha256 == native_result::digest(f.input.payload.as_bytes()),
```


### normal-write

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/managed_binding/binding.rs` · lines 389–408 · blob `310a55a2a30ed1dc09ecc7b807561af10428790a` · object SHA-256 `3ec2e6c63efdbea440a7f9f2473ff8af8ed7787aa81cd836ada30d83de915978` · excerpt SHA-256 `6e54723ac15c8bde0710c6c8b8308dd7ec2eade427fc9d1d8e87fdd578873df6`.

```rust
389:                 .facts()
390:                 .state_path
391:                 .to_str()
392:                 .is_some_and(|path| self.connection.path() == Some(path)),
393:             "Session binder is not the selected owner's database"
394:         );
395:         let tx = self
396:             .connection
397:             .transaction_with_behavior(TransactionBehavior::Immediate)?;
398:         validate_current_tx(&tx, plan.proof.marker(), &plan.current)?;
399:         plan.proof.marker().validate_driver_live_tx(&tx)?;
400:         normal_eligibility(&plan.proof, &plan.current)?;
401:         validate_registered_owner_tx(&tx, &plan.proof, &plan.owner_raw)?;
402:         validate_invocation_tx(&tx, &plan.invocation)?;
403:         let record = plan.session.parsed();
404:         let exact: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE id=?1 AND kind='session' AND project_id=?2 AND goal_id IS ?3 AND task_id IS ?4 AND version=?5 AND body=?6)",params![record.id.to_string(),record.scope.project_id.to_string(),record.scope.goal_id.map(|v|v.to_string()),record.scope.task_id.map(|v|v.to_string()),record.version,plan.session.raw()],|r|r.get(0))?;
405:         ensure!(
406:             exact,
407:             "current latest Session advanced; plan again without refreshing original pins"
408:         );
```


### git-run

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/git_io.rs` · lines 135–173 · blob `a8c0d50371c3a344833bcebb4f64cab1c1ba79e6` · object SHA-256 `07942394dcef7dc45b05086532ed9d195bfe69dd1183841a6a96f4308f0dc2e7` · excerpt SHA-256 `4d03995fdd99544646ca02dc3b9cca7bef3cef5a7c666d9d57b5ec58df493860`.

```rust
135:     async fn run_command(
136:         &self,
137:         root: &Path,
138:         mut command: tokio::process::Command,
139:         kind: &str,
140:     ) -> Result<process::CommandCapture> {
141:         command
142:             .envs(
143:                 self.profile
144:                     .environment(&self.unit.cookie, self.owner.ipc_path())?,
145:             )
146:             .stdin(Stdio::null())
147:             .stdout(Stdio::piped())
148:             .stderr(Stdio::piped());
149:         let operation = OperationId::new();
150:         if let Some(lease) = &self.git_lease {
151:             command.env("RRX_GIT_GATE_TOKEN", lease.id.to_string());
152:         } else {
153:             command.env_remove("RRX_GIT_GATE_TOKEN");
154:         }
155:         let child = {
156:             let mut store = self
157:                 .owner
158:                 .store
159:                 .lock()
160:                 .map_err(|_| anyhow::anyhow!("state poisoned"))?;
161:             let current = store.execution_unit(self.unit.id)?;
162:             ensure!(
163:                 current.scope == self.unit.scope
164:                     && current.generation == self.unit.generation
165:                     && current.owner_epoch == self.unit.owner_epoch
166:                     && current.session_id == self.unit.session_id,
167:                 "Git preparation authority retired"
168:             );
169:             store.validate_execution(&current.authority(), self.native, !self.native)?;
170:             // Capture/inspection has Runtime-only finalization authority; never grant native tools.
171:             store.reserve_execution_helper_pinned(
172:                 &current.authority(),
173:                 operation,
```


### git-monitor-call

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/git_io.rs` · lines 192–239 · blob `a8c0d50371c3a344833bcebb4f64cab1c1ba79e6` · object SHA-256 `07942394dcef7dc45b05086532ed9d195bfe69dd1183841a6a96f4308f0dc2e7` · excerpt SHA-256 `2cda2b8d5787009ff7ed7bbf05c6730859c80835d83de6d3c8e3d8b02aa448c2`.

```rust
192:         let mut helper_guard = owner::HelperGuard::new(self.owner.clone(), operation);
193:         let observed = process::capture_scoped_pinned(
194:             child,
195:             &self.owner,
196:             &self.unit,
197:             self.native,
198:             self.driver.as_ref(),
199:         )
200:         .await;
201:         let mut receipt = BTreeMap::new();
202:         if let Ok(o) = &observed {
203:             receipt.insert(
204:                 "exit".into(),
205:                 o.receipt
206:                     .status
207:                     .code()
208:                     .map_or_else(|| "signal".into(), |c| c.to_string()),
209:             );
210:             receipt.insert(
211:                 "group_cleanup".into(),
212:                 if o.receipt.group_error.is_some() {
213:                     "unknown"
214:                 } else {
215:                     "requested"
216:                 }
217:                 .into(),
218:             );
219:         }
220:         self.owner
221:             .store
222:             .lock()
223:             .map_err(|_| anyhow::anyhow!("state poisoned"))?
224:             .reconcile_managed_effect_pinned(
225:                 operation,
226:                 1,
227:                 if observed.is_ok() {
228:                     EffectState::Confirmed
229:                 } else {
230:                     EffectState::Unknown
231:                 },
232:                 receipt,
233:                 if observed.is_ok() {
234:                     self.driver.as_ref()
235:                 } else {
236:                     None
237:                 },
238:             )?;
239:         helper_guard.disarm();
```


### process-monitor

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/process.rs` · lines 357–379 · blob `f5a63f7dcfb890142bba6b849f0962cec39f71b5` · object SHA-256 `249f046912cad6b757f063b3547f262043f2119c304e86c710b4fb0add2bf91f` · excerpt SHA-256 `ae774bf579377ae2da6ebe9de1b95f1cff7c567d5e546d47834ed4cf33e40eb8`.

```rust
357: pub(crate) async fn capture_scoped_pinned(
358:     child: OwnedProcess,
359:     owner: &super::RuntimeOwner,
360:     pinned: &super::ExecutionUnit,
361:     native: bool,
362:     driver: Option<&crate::state::DriverReadTicket>,
363: ) -> Result<CommandCapture> {
364:     let capture = capture_child(child);
365:     tokio::pin!(capture);
366:     let mut fence = tokio::time::interval(Duration::from_millis(50));
367:     loop {
368:         tokio::select! {
369:             observed = &mut capture => return observed,
370:             _ = fence.tick() => {
371:                 let mut store = owner.store.lock().map_err(|_|anyhow::anyhow!("state poisoned"))?;
372:                 if let Some(ticket) = driver { store.validate_driver_read(ticket)?; }
373:                 let current = store.execution_unit(pinned.id)?;
374:                 ensure!(current.scope == pinned.scope && current.generation == pinned.generation && current.owner_epoch == pinned.owner_epoch && current.session_id == pinned.session_id, "helper execution identity changed");
375:                 store.validate_execution(&current.authority(), native, !native)?;
376:             }
377:         }
378:     }
379: }
```


### marker-refusal

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/runtime/driver.rs` · lines 331–343 · blob `18db97d7437cd3612355926002003fba5cbc3cba` · object SHA-256 `629e1c2dadd192e9597d5b87982f0961e99e2de6a827065c06613d7d5f66e153` · excerpt SHA-256 `31714e68e8f350e8784d2cbbd2bcb147f3652b440af468291ccf84f3f223d007`.

```rust
331:     ensure!(epoch > 0 && epoch == current_epoch, "Driver epoch retired");
332:     let live: bool = c.query_row(
333:         "SELECT rrx_live_task_driver(?1,?2,?3,?4,?5)",
334:         params![task_id.to_string(), id, epoch, version, body],
335:         |r| r.get(0),
336:     )?;
337:     ensure!(live, "actual retained Task Driver unavailable");
338:     let row: Row = decode(body)?;
339:     ensure!(
340:         row.marker.is_none(),
341:         "marker-bound Driver requires the genuine managed successor/lifecycle reader"
342:     );
343:     let current = snapshot(c, task_id)?;
```


### gate-final

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/workflow_gates.rs` · lines 323–388 · blob `8bc39714fad0d82fb7df6527a8e0d2d2109a194b` · object SHA-256 `83244e016d798395aece728b3d61d60e210514b7e65e09ceb719586413b34785` · excerpt SHA-256 `3fb694d83e9d002f9ed20eb69d8403e24c16db98a83a00b1a1e4271bf77d5a5f`.

```rust
323:             _ => {
324:                 return Ok(GateOutcome::Waiting(format!(
325:                     "{} requires its qualified production evidence integration",
326:                     phase.key()
327:                 )));
328:             }
329:         }
330:         if let Some(artifact) = &checked_artifact {
331:             ensure!(
332:                 invocation.sources.artifact == Some(artifact.id)
333:                     && artifact.scope == invocation.task.scope()
334:                     && artifact.revision == invocation.sources.revision
335:                     && artifact.dependencies == invocation.sources.source_versions,
336:                 "gate retained artifact differs from observed input"
337:             );
338:             ResultStore::new(self.owner.clone())
339:                 .verify(artifact)
340:                 .await?;
341:         }
342:         let current = self
343:             .sources
344:             .capture(
345:                 invocation.project.clone(),
346:                 invocation.task.clone(),
347:                 phase,
348:                 invocation.budget.clone(),
349:             )
350:             .await?;
351:         ensure!(
352:             current == invocation.sources,
353:             "gate source changed during checks"
354:         );
355:         let mut store = self
356:             .owner
357:             .store
358:             .lock()
359:             .map_err(|_| anyhow::anyhow!("state poisoned"))?;
360:         ensure!(
361:             Self::claim(&store, &invocation)?.0 == claim,
362:             "gate claim changed during checks"
363:         );
364:         if let Some(unit) = &checked_unit {
365:             ensure!(
366:                 serde_json::to_value(store.validate_execution(&unit.authority(), false, true)?)?
367:                     == serde_json::to_value(unit)?,
368:                 "gate unit changed during checks"
369:             );
370:         }
371:         if let Some(artifact) = &checked_artifact {
372:             ensure!(
373:                 serde_json::to_value(store.result_artifact(artifact.id)?)?
374:                     == serde_json::to_value(artifact)?,
375:                 "gate artifact changed during checks"
376:             );
377:         }
378:         let mut receipt = Record::new(
379:             invocation.task.scope(),
380:             RecordKind::Verification,
381:             json!({"schema":"managed_workflow_gate_v1","claim":claim,"phase":phase,
382:                 "revision":invocation.sources.revision,"sources":invocation.sources.source_versions,
383:                 "launch_revision":invocation.context.revision,
384:                 "context_data_sha256":digest(&serde_json::to_vec(&invocation.context.data)?),
385:                 "unit":checked_unit.as_ref().map(ManagedUnitRef::from),
386:                 "profile_digest":checked_unit.as_ref().map(|u| &u.profile_digest),
387:                 "artifact":checked_artifact.as_ref().map(|a| a.id),
388:                 "manifest_sha256":checked_artifact.as_ref().map(|a| &a.manifest_sha256)}),
```


### store-shape

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/mod.rs` · lines 88–92 · blob `f8bec61c9ab5258adaabad10525d4ce6fbe2cef0` · object SHA-256 `fbbd7f790809bf9fbaa4b9250af2aa589511e930d1883b1de3778f76cd98bb3f` · excerpt SHA-256 `76ac62d064cd62baff0d8dbd60f8d620d2a6fb9e6ed8df7e1425328730bf1197`.

```rust
88: pub struct Store {
89:     connection: Connection,
90:     #[allow(dead_code)] // Actual managed marker/binder is composed separately.
91:     binding_permits: std::sync::Arc<managed_binding::PrivatePermitManager>,
92: }
```


### admission-shape

CA `95ef0e783c0b18c90dfef23ba1b77b9dc67735f1` · `crates/rrx/src/runtime/installation.rs` · lines 82–87 · blob `e0dab8d453a6bfa10700228f930d64b4ba291462` · object SHA-256 `8e39e03597afeb20a34e4e69101c071ad82432c09748f6daa82b603feb30ae3e` · excerpt SHA-256 `2bb86fcdb723f19565e7178ad14766bc1cec54d7c5d6e7ef7352cb1358ae31d7`.

```rust
82: /// Stop exclusion only: no SQL permission or Native grant. Field order drops
83: /// admission before the last strong Runtime reference.
84: pub(crate) struct ActivationAdmission {
85:     _admission: tokio::sync::OwnedMutexGuard<()>,
86:     _runtime: Arc<Runtime>,
87: }
```


### admission-acquire

CA `95ef0e783c0b18c90dfef23ba1b77b9dc67735f1` · `crates/rrx/src/runtime/installation.rs` · lines 148–194 · blob `e0dab8d453a6bfa10700228f930d64b4ba291462` · object SHA-256 `8e39e03597afeb20a34e4e69101c071ad82432c09748f6daa82b603feb30ae3e` · excerpt SHA-256 `7ac782cf815bbb224feac13c653065364724c236ab07dbfaaedd7468130b39af`.

```rust
148:     pub(crate) async fn admit_activation(
149:         &self,
150:         plan: &Arc<DriverPreparationAdvance>,
151:         lifetime: &WorkerLifetime,
152:     ) -> Result<ActivationAdmission> {
153:         self.admit_activation_inner(plan, lifetime, true).await
154:     }
155:     pub(crate) async fn admit_activation_recovery(
156:         &self,
157:         plan: &Arc<DriverPreparationAdvance>,
158:         lifetime: &WorkerLifetime,
159:     ) -> Result<ActivationAdmission> {
160:         self.admit_activation_inner(plan, lifetime, false).await
161:     }
162:     async fn admit_activation_inner(
163:         &self,
164:         plan: &Arc<DriverPreparationAdvance>,
165:         lifetime: &WorkerLifetime,
166:         require_retained: bool,
167:     ) -> Result<ActivationAdmission> {
168:         let runtime = self
169:             .runtime
170:             .upgrade()
171:             .ok_or_else(|| anyhow::anyhow!("activation Runtime ended"))?;
172:         let admission = tokio::select! { biased;
173:             ()=lifetime.cancelled()=>return Err(anyhow::anyhow!("activation Driver cancelled")),
174:             guard=runtime.control_admission.clone().lock_owned()=>guard,
175:         };
176:         ensure!(
177:             runtime.service_running() && self.is_current(),
178:             "Runtime stopped before activation"
179:         );
180:         let association = lifetime.association()?;
181:         ensure!(
182:             plan.activation_roster()?.is_same_composition(self)
183:                 && self.original_task.id == association.task()
184:                 && plan.belongs_to(&association),
185:             "activation original composition/worker linkage differs"
186:         );
187:         if require_retained {
188:             ensure!(plan.is_retained()?, "activation preparation lost custody");
189:         }
190:         Ok(ActivationAdmission {
191:             _admission: admission,
192:             _runtime: runtime,
193:         })
194:     }
```


### service-loop

CA `95ef0e783c0b18c90dfef23ba1b77b9dc67735f1` · `crates/rrx/src/runtime/service.rs` · lines 73–105 · blob `6d1353ffe5731ff12a6ec75d6527f2bc10d52cde` · object SHA-256 `e6820a65d8d54781ee0e30f73027e98c2d6c1b847eac69b63546d0355063c758` · excerpt SHA-256 `3726adbdcdf75f24e09d356289a90b724deddcac296f71c6b724f1eed9af25ae`.

```rust
73:             loop {
74:                 let Some(runtime) = retained.upgrade() else {
75:                     return Ok(());
76:                 };
77:                 if runtime.stopping.load(Ordering::SeqCst) {
78:                     return Ok(());
79:                 }
80:                 let (next, more) = runtime
81:                     .owner
82:                     .store
83:                     .lock()
84:                     .map_err(|_| anyhow::anyhow!("state poisoned"))?
85:                     .reconcile_runtime_attention(
86:                         runtime.owner.instance_id(),
87:                         runtime.owner.epoch(),
88:                         sequence,
89:                     )?;
90:                 sequence = next;
91:                 let pending = runtime.phases.reconcile_pending()?;
92:                 let driver_pending = runtime.observe_task_drivers()?;
93:                 // A refusal after reservation ends only this saved-cursor
94:                 // sweep. Its retained claim/closure owns the outcome.
95:                 let claims = if more {
96:                     0
97:                 } else {
98:                     runtime.admit_ready_tasks().unwrap_or_default()
99:                 };
100:                 let delay = super::phase_supervisor::PhaseSupervisor::delay(
101:                     pending || driver_pending > 0 || claims > 0,
102:                     &mut backoff,
103:                 );
104:                 let wake = runtime.wake.clone();
105:                 drop(runtime);
```


### driver-private

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/runtime/driver/marker.rs` · lines 10–30 · blob `66771f508f0dd146f56b4465c75bbf4575851d10` · object SHA-256 `96ebf29fcf391d8b560173a7a5e3667924e5ad2e4d066a9e07aef6befc4e6862` · excerpt SHA-256 `b5dcfc01396b770e2e775b4717787bad6dd22b94204ec69b8f04bd3101e098ba`.

```rust
10: pub(crate) struct DriverMarkerAdvance {
11:     ticket: Arc<DriverReadTicket>,
12:     next: Row,
13:     body: String,
14:     task: Task,
15:     task_body: String,
16:     workflow: Record,
17:     workflow_body: String,
18:     source: Option<SourceMarkerAdvance>,
19: }
20: /// Created only by checking a completed planned mutation on the same Store
21: /// after commit. Not Clone/Deserialize and never an input/Native credential.
22: pub(crate) struct DriverPublication {
23:     pub(super) task: TaskId,
24:     pub(super) id: Uuid,
25:     pub(super) epoch: u64,
26:     pub(super) before_version: u64,
27:     pub(super) before_body: String,
28:     pub(super) after_version: u64,
29:     pub(super) after_body: String,
30: }
```


### driver-ports

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/runtime/driver/marker.rs` · lines 155–186 · blob `66771f508f0dd146f56b4465c75bbf4575851d10` · object SHA-256 `96ebf29fcf391d8b560173a7a5e3667924e5ad2e4d066a9e07aef6befc4e6862` · excerpt SHA-256 `879536a4957ca30b8ffd3c1a0e2a4984c0681ff8eeb7595bb86e16ac850c8aea`.

```rust
155:     pub(in crate::state) fn exact_mutations(&self) -> Result<Vec<ExactRowMutation>> {
156:         let mut mutations = vec![ExactRowMutation::new(
157:             "task_drivers",
158:             "UPDATE",
159:             Some(image(&self.ticket.row, &self.ticket.body)?),
160:             Some(image(&self.next, &self.body)?),
161:         )?];
162:         if let Some(source) = &self.source {
163:             mutations.push(source.mutation()?);
164:         }
165:         Ok(mutations)
166:     }
167:     /// Required BEFORE Root's Task/Workflow marker writes, in their same TX.
168:     pub(crate) fn validate_current_tx(&self, tx: &Transaction<'_>) -> Result<()> {
169:         self.ticket.validate_current_tx(tx)
170:     }
171:     /// Required AFTER the fixed Task/W writes. Root supplies ONE exact permit
172:     /// batch covering every planned row; no nested manager or generic refresh.
173:     pub(crate) fn freeze_marker_tx(&self, tx: &Transaction<'_>) -> Result<()> {
174:         self.ticket.validate_ancillary_tx(tx)?;
175:         self.ticket.scope.validate_projection(
176:             tx,
177:             &self.task,
178:             &self.task_body,
179:             Some((&self.workflow, &self.workflow_body)),
180:         )?;
181:         if let Some(source) = &self.source {
182:             source.write_tx(tx)?;
183:         }
184:         ensure!(tx.execute("UPDATE task_drivers SET version=?1,body=?2 WHERE task_id=?3 AND id=?4 AND owner_epoch=?5 AND version=?6 AND state='driving' AND body=?7",params![self.next.version,self.body,self.task.id.to_string(),self.next.id.to_string(),self.next.epoch,self.ticket.row.version,self.ticket.body])?==1,"Driver marker freeze CAS changed");
185:         Ok(())
186:     }
```


### driver-publish

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/runtime/driver/marker.rs` · lines 188–223 · blob `66771f508f0dd146f56b4465c75bbf4575851d10` · object SHA-256 `96ebf29fcf391d8b560173a7a5e3667924e5ad2e4d066a9e07aef6befc4e6862` · excerpt SHA-256 `84260e0f3d6a1e3696d4d4a6fc7e003adbdb21b381b036c60c6150a75a754ecf`.

```rust
188: impl Store {
189:     /// Known-commit publication while SharedStore remains excluded. A failed
190:     /// check leaves this exact owned plan with its caller, never adopts newer rows.
191:     pub(crate) fn publish_driver_marker(&mut self, plan: &DriverMarkerAdvance) -> Result<()> {
192:         ensure!(
193:             self.connection.is_autocommit(),
194:             "Driver publication precedes commit"
195:         );
196:         let tx = self.connection.transaction()?;
197:         plan.ticket.scope.validate_projection(
198:             &tx,
199:             &plan.task,
200:             &plan.task_body,
201:             Some((&plan.workflow, &plan.workflow_body)),
202:         )?;
203:         plan.ticket.validate_selection_tx(&tx)?;
204:         if let Some(source) = &plan.source {
205:             source.validate_result_tx(&tx)?;
206:         } else {
207:             plan.ticket.validate_source_tx(&tx)?;
208:         }
209:         let exact:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM task_drivers WHERE task_id=?1 AND id=?2 AND owner_epoch=?3 AND version=?4 AND state='driving' AND body=?5)",params![plan.task.id.to_string(),plan.next.id.to_string(),plan.next.epoch,plan.next.version,plan.body],|r|r.get(0))?;
210:         ensure!(exact, "planned committed Driver row differs");
211:         tx.commit()?;
212:         let publication = DriverPublication {
213:             task: plan.task.id,
214:             id: plan.next.id,
215:             epoch: plan.next.epoch,
216:             before_version: plan.ticket.row.version,
217:             before_body: plan.ticket.body.clone(),
218:             after_version: plan.next.version,
219:             after_body: plan.body.clone(),
220:         };
221:         plan.ticket.association.publish_exact(&publication)
222:     }
223: }
```


### capture-catalogue

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/results.rs` · lines 224–369 · blob `c4bdfc9c018b45479d327b764737f093c55074f2` · object SHA-256 `290baf091d847c9fd2ba3e900d67e5d9326cd50c7c741e45e3742534f8cec847` · excerpt SHA-256 `09aece4e065afd151dcb5553362c88c0b4ebd07fe0d0ad5f0d713b8608314011`.

```rust
224:             .project(unit.scope.project_id)?
225:             .context("Project missing")?;
226:         let source = self
227:             .owner
228:             .store
229:             .lock()
230:             .map_err(|_| anyhow::anyhow!("state poisoned"))?
231:             .task(unit.scope.task_id.context("Task missing")?)?
232:             .context("Task missing")?;
233:         let io = UnitGit::new(self.owner.clone(), &unit, false)?;
234:         io.ownership(&project, &source).await?;
235:         let format = io
236:             .text(&unit.worktree, ["rev-parse", "--show-object-format"])
237:             .await?;
238:         ensure!(
239:             matches!(format.as_str(), "sha1" | "sha256"),
240:             "unsupported Git object format"
241:         );
242:         ensure!(
243:             revision.len() == if format == "sha1" { 40 } else { 64 },
244:             "OID format mismatch"
245:         );
246:         io.run(
247:             &unit.worktree,
248:             [
249:                 "merge-base",
250:                 "--is-ancestor",
251:                 unit.base_sha.as_str(),
252:                 revision,
253:             ],
254:         )
255:         .await?;
256:         ensure!(
257:             io.text(
258:                 &unit.worktree,
259:                 ["rev-parse", "--verify", &format!("{revision}^{{commit}}")]
260:             )
261:             .await?
262:                 == revision,
263:             "result is not exact commit"
264:         );
265:         qualified_content_scoped(&unit.worktree, revision, &io).await?;
266:         let _guard = self.gate.lock().await;
267:         let repository = self
268:             .owner
269:             .root
270:             .join("projects")
271:             .join(project.id.to_string())
272:             .join("results.git");
273:         let id = ArtifactId::new();
274:         let directory = self.owner.root.join("artifacts").join(id.to_string());
275:         let manifest = directory.join("manifest.json");
276:         let mut artifact = ResultArtifact {
277:             id,
278:             scope: unit.scope.clone(),
279:             unit_id: unit.id,
280:             state: ArtifactState::Staging,
281:             revision: revision.into(),
282:             base_sha: unit.base_sha.clone(),
283:             object_format: format.clone(),
284:             repository: repository.clone(),
285:             manifest,
286:             manifest_sha256: String::new(),
287:             dependencies: sources.clone(),
288:             version: 1,
289:             created_at: crate::domain::now_ms(),
290:         };
291:         self.owner
292:             .store
293:             .lock()
294:             .map_err(|_| anyhow::anyhow!("state poisoned"))?
295:             .stage_result(authority, &artifact)?;
296:         std::fs::create_dir_all(repository.parent().context("result parent missing")?)?;
297:         ensure!(
298:             !repository.starts_with(&unit.worktree) && !unit.worktree.starts_with(&repository),
299:             "results overlap executor"
300:         );
301:         if !repository.exists() {
302:             io.run(
303:                 repository.parent().context("repository parent missing")?,
304:                 [
305:                     "init",
306:                     "--bare",
307:                     &format!("--object-format={format}"),
308:                     repository.to_str().context("repository UTF-8")?,
309:                 ],
310:             )
311:             .await?;
312:         }
313:         retained_storage(&repository)?;
314:         ensure!(
315:             io.text(&repository, ["rev-parse", "--show-object-format"])
316:                 .await?
317:                 == format,
318:             "repository format changed"
319:         );
320:         // fetch copies object graphs over upload-pack. No clone-local hardlinks or alternates.
321:         io.run(
322:             &repository,
323:             [
324:                 "-c",
325:                 "fetch.fsckObjects=true",
326:                 "fetch",
327:                 "--no-tags",
328:                 "--no-write-fetch-head",
329:                 "--no-recurse-submodules",
330:                 "--",
331:                 unit.worktree.to_str().context("worktree UTF-8")?,
332:                 &format!("{revision}:refs/rrx/{id}/commit"),
333:                 &format!("{}:refs/rrx/{id}/base", unit.base_sha),
334:             ],
335:         )
336:         .await?;
337:         retained_storage(&repository)?;
338:         io.run(&repository, ["fsck", "--full", "--strict", "--no-dangling"])
339:             .await?;
340:         for oid in [revision, unit.base_sha.as_str()] {
341:             io.run(
342:                 &repository,
343:                 ["rev-list", "--objects", "--missing=error", oid],
344:             )
345:             .await?;
346:         }
347:         std::fs::create_dir_all(directory.parent().context("artifact root missing")?)?;
348:         std::fs::create_dir(&directory)?;
349:         let bytes = serde_json::to_vec(&ResultManifest {
350:             artifact: id,
351:             unit: unit.id,
352:             revision: revision.into(),
353:             base: unit.base_sha.clone(),
354:             object_format: format,
355:             sources,
356:         })?;
357:         durable_file(&artifact.manifest, &bytes)?;
358:         sync_tree(&repository)?;
359:         File::open(repository.parent().context("repository parent missing")?)?.sync_all()?;
360:         File::open(directory.parent().context("artifact parent missing")?)?.sync_all()?;
361:         artifact.manifest_sha256 = hex(&bytes);
362:         artifact.state = ArtifactState::Ready;
363:         artifact.version = 2;
364:         self.owner
365:             .store
366:             .lock()
367:             .map_err(|_| anyhow::anyhow!("state poisoned"))?
368:             .ready_result(&artifact, 1)?;
369:         Ok(artifact)
```


### capture-ownership

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/git_io.rs` · lines 249–294 · blob `a8c0d50371c3a344833bcebb4f64cab1c1ba79e6` · object SHA-256 `07942394dcef7dc45b05086532ed9d195bfe69dd1183841a6a96f4308f0dc2e7` · excerpt SHA-256 `5c5ba89e10eb4816469d4c87c70dcef879a638549e2c96048b325cc40c77cfa8`.

```rust
249:     pub(crate) async fn ownership(
250:         &self,
251:         project: &crate::domain::Project,
252:         task: &crate::domain::Task,
253:     ) -> Result<()> {
254:         let root = &project.root;
255:         let path = task.worktree.as_deref().context("Task worktree missing")?;
256:         let source_top = PathBuf::from(self.text(root, ["rev-parse", "--show-toplevel"]).await?);
257:         let source_git_dir = PathBuf::from(
258:             self.text(root, ["rev-parse", "--path-format=absolute", "--git-dir"])
259:                 .await?,
260:         );
261:         let source_common = PathBuf::from(
262:             self.text(
263:                 root,
264:                 ["rev-parse", "--path-format=absolute", "--git-common-dir"],
265:             )
266:             .await?,
267:         );
268:         let source_roots = self
269:             .text(
270:                 root,
271:                 [
272:                     "rev-list",
273:                     "--max-parents=0",
274:                     &format!("refs/heads/{}", project.base_branch),
275:                 ],
276:             )
277:             .await?
278:             .lines()
279:             .map(str::to_owned)
280:             .collect();
281:         let task_top = PathBuf::from(self.text(path, ["rev-parse", "--show-toplevel"]).await?);
282:         let task_common = PathBuf::from(
283:             self.text(
284:                 path,
285:                 ["rev-parse", "--path-format=absolute", "--git-common-dir"],
286:             )
287:             .await?,
288:         );
289:         let branch = self
290:             .text(path, ["symbolic-ref", "--quiet", "--short", "HEAD"])
291:             .await?;
292:         let revision = self
293:             .text(path, ["rev-parse", "--verify", "HEAD^{commit}"])
294:             .await?;
```


### capture-content

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/results.rs` · lines 777–818 · blob `c4bdfc9c018b45479d327b764737f093c55074f2` · object SHA-256 `290baf091d847c9fd2ba3e900d67e5d9326cd50c7c741e45e3742534f8cec847` · excerpt SHA-256 `2fb2417b8eaada5db2f3ceea2a226b5f11213366d64424ea73ef5fbab76f4250`.

```rust
777: async fn qualified_content_inner(root: &Path, revision: &str, io: &UnitGit) -> Result<()> {
778:     let tree = read_git(root, ["ls-tree", "-r", "-z", revision], io).await?;
779:     for line in tree.split(|b| *b == 0).filter(|s| !s.is_empty()) {
780:         ensure!(
781:             !line.starts_with(b"160000 "),
782:             "submodules require a qualified profile"
783:         );
784:         ensure!(
785:             !line.starts_with(b"120000 "),
786:             "source symlinks require a qualified profile"
787:         );
788:     }
789:     // LFS pointers are ordinary Git blobs, not their required external content.
790:     let args = [
791:         "grep",
792:         "-l",
793:         "-I",
794:         "-e",
795:         "version https://git-lfs.github.com/spec/v1",
796:         revision,
797:         "--",
798:     ];
799:     let observed = io.run_observed(root, args).await?;
800:     ensure!(
801:         observed.receipt.status.code() == Some(1)
802:             || (observed.receipt.status.success() && observed.stdout.is_empty()),
803:         "LFS pointer scan found unsupported content or failed"
804:     );
805:     let attrs = read_git(root, ["ls-tree", "-r", "--name-only", revision], io).await?;
806:     for name in std::str::from_utf8(&attrs)?
807:         .lines()
808:         .filter(|n| n.ends_with(".gitattributes"))
809:     {
810:         let bytes = read_git(root, ["show", &format!("{revision}:{name}")], io).await?;
811:         ensure!(
812:             !bytes
813:                 .windows(b"filter=lfs".len())
814:                 .any(|w| w == b"filter=lfs"),
815:             "LFS attributes require a qualified profile"
816:         );
817:     }
818:     Ok(())
```


### capture-verify

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/results.rs` · lines 371–509 · blob `c4bdfc9c018b45479d327b764737f093c55074f2` · object SHA-256 `290baf091d847c9fd2ba3e900d67e5d9326cd50c7c741e45e3742534f8cec847` · excerpt SHA-256 `988c46f2cc48fb4eb1ec723a389f5d28b784cab1054b1b40d12b86fef8e41871`.

```rust
371:     pub async fn verify(&self, artifact: &ResultArtifact) -> Result<()> {
372:         let io = RetainedGit::new(self.owner.clone(), artifact)?;
373:         self.verify_inner(artifact, RetainedReader::Historical(&io))
374:             .await?;
375:         io.validate()
376:     }
377:     pub(crate) async fn verify_recovery(
378:         &self,
379:         artifact: &ResultArtifact,
380:         binding: &crate::state::SourceReadBinding,
381:     ) -> Result<()> {
382:         let io = RetainedGit::for_recovery(self.owner.clone(), artifact, binding.clone())?;
383:         self.verify_inner(artifact, RetainedReader::Historical(&io))
384:             .await?;
385:         io.validate()
386:     }
387:     pub(crate) async fn workflow_publication(
388:         &self,
389:         authority: &ExecutionAuthority,
390:         artifact: ArtifactId,
391:     ) -> Result<WorkflowPublication> {
392:         let (unit, artifact) = {
393:             let store = self
394:                 .owner
395:                 .store
396:                 .lock()
397:                 .map_err(|_| anyhow::anyhow!("state poisoned"))?;
398:             (
399:                 store.validate_execution(authority, false, true)?,
400:                 store.result_artifact(artifact)?,
401:             )
402:         };
403:         ensure!(
404:             unit.kind == UnitKind::Executor
405:                 && unit.work == Some(WorkOutcome::Success)
406:                 && !unit.native_effects_open
407:                 && artifact.unit_id == unit.id
408:                 && artifact.scope == unit.scope
409:                 && artifact.state == ArtifactState::Ready,
410:             "publication verification requires exact successful executor artifact"
411:         );
412:         let io = UnitGit::new(self.owner.clone(), &unit, false)?;
413:         self.verify_inner(&artifact, RetainedReader::Current(&io))
414:             .await?;
415:         {
416:             let store = self
417:                 .owner
418:                 .store
419:                 .lock()
420:                 .map_err(|_| anyhow::anyhow!("state poisoned"))?;
421:             store.validate_execution(authority, false, true)?;
422:             ensure!(
423:                 serde_json::to_value(store.result_artifact(artifact.id)?)?
424:                     == serde_json::to_value(&artifact)?,
425:                 "artifact changed during publication verification"
426:             );
427:         }
428:         Ok(WorkflowPublication {
429:             authority: authority.clone(),
430:             artifact,
431:         })
432:     }
433:     async fn verify_inner(&self, artifact: &ResultArtifact, io: RetainedReader<'_>) -> Result<()> {
434:         ensure!(
435:             matches!(
436:                 artifact.state,
437:                 ArtifactState::Ready | ArtifactState::Published
438:             ),
439:             "artifact is not usable"
440:         );
441:         ensure!(
442:             artifact.repository
443:                 == self
444:                     .owner
445:                     .root
446:                     .join("projects")
447:                     .join(artifact.scope.project_id.to_string())
448:                     .join("results.git")
449:                 && artifact.manifest
450:                     == self
451:                         .owner
452:                         .root
453:                         .join("artifacts")
454:                         .join(artifact.id.to_string())
455:                         .join("manifest.json"),
456:             "artifact storage mismatch"
457:         );
458:         let mut bytes = Vec::new();
459:         File::open(&artifact.manifest)?
460:             .take(128 * 1024 + 1)
461:             .read_to_end(&mut bytes)?;
462:         ensure!(
463:             bytes.len() <= 128 * 1024 && hex(&bytes) == artifact.manifest_sha256,
464:             "artifact manifest corruption"
465:         );
466:         let m: ResultManifest = serde_json::from_slice(&bytes)?;
467:         ensure!(
468:             m.artifact == artifact.id
469:                 && m.unit == artifact.unit_id
470:                 && m.revision == artifact.revision
471:                 && m.base == artifact.base_sha
472:                 && m.object_format == artifact.object_format
473:                 && m.sources == artifact.dependencies,
474:             "artifact manifest identity mismatch"
475:         );
476:         retained_storage(&artifact.repository)?;
477:         for (name, oid) in [("commit", &artifact.revision), ("base", &artifact.base_sha)] {
478:             ensure!(
479:                 text(
480:                     &retained_git(
481:                         &artifact.repository,
482:                         &io,
483:                         [
484:                             "rev-parse",
485:                             "--verify",
486:                             &format!("refs/rrx/{}/{name}", artifact.id)
487:                         ]
488:                     )
489:                     .await?
490:                 )? == *oid,
491:                 "retained reference changed"
492:             );
493:             retained_git(
494:                 &artifact.repository,
495:                 &io,
496:                 ["rev-list", "--objects", "--missing=error", oid],
497:             )
498:             .await?;
499:         }
500:         retained_git(
501:             &artifact.repository,
502:             &io,
503:             ["fsck", "--full", "--strict", "--no-dangling"],
504:         )
505:         .await?;
506:         Ok(())
507:     }
508:     pub async fn publish(
509:         &self,
```


### corpus-bound

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/workflow_source.rs` · lines 86–134 · blob `15e3513b8abe19d94a1817ea34252bcdc31efb68` · object SHA-256 `4b57e5afa4e66a51c7e2a828e9bb07ce1dae763a6ac58dbbaebb04c76aa9a00f` · excerpt SHA-256 `e4361d599c030290aa0f610fc0a502bb2029fa1488602c707cca2e2c0b05e694`.

```rust
86: pub(crate) fn parse_committed_tree(bytes: &[u8]) -> Result<BTreeMap<String, CommittedTreeEntry>> {
87:     ensure!(
88:         bytes.len() <= 16 * 1024 * 1024 && (bytes.is_empty() || bytes.last() == Some(&0)),
89:         "committed tree bound/frame differs"
90:     );
91:     let mut entries = BTreeMap::new();
92:     for entry in bytes.split(|b| *b == 0).filter(|e| !e.is_empty()) {
93:         ensure!(
94:             entries.len() < 4096,
95:             "committed inventory exceeds 4096 files"
96:         );
97:         let text = std::str::from_utf8(entry)?;
98:         let (header, path) = text
99:             .split_once('\t')
100:             .context("committed tree path absent")?;
101:         relative(path)?;
102:         let fields: Vec<_> = header.split_whitespace().collect();
103:         ensure!(
104:             fields.len() == 4
105:                 && valid_oid(fields[2])
106:                 && matches!(
107:                     (fields[0], fields[1]),
108:                     ("100644" | "100755" | "120000", "blob") | ("160000", "commit")
109:                 ),
110:             "committed tree physical type/OID differs"
111:         );
112:         let size = if fields[1] == "blob" {
113:             Some(fields[3].parse::<usize>()?)
114:         } else {
115:             ensure!(fields[3] == "-", "committed gitlink size differs");
116:             None
117:         };
118:         ensure!(
119:             entries
120:                 .insert(
121:                     path.to_owned(),
122:                     CommittedTreeEntry {
123:                         mode: fields[0].into(),
124:                         kind: fields[1].into(),
125:                         oid: fields[2].into(),
126:                         size,
127:                     }
128:                 )
129:                 .is_none(),
130:             "duplicate committed tree path"
131:         );
132:     }
133:     Ok(entries)
134: }
```


### corpus-catalogue

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/workflow_source.rs` · lines 1220–1258 · blob `15e3513b8abe19d94a1817ea34252bcdc31efb68` · object SHA-256 `4b57e5afa4e66a51c7e2a828e9bb07ce1dae763a6ac58dbbaebb04c76aa9a00f` · excerpt SHA-256 `804a38e25bc39d27f5561f5284345382356b71203ec052a4f0ac0b45b54c9340`.

```rust
1220: async fn read_corpus(io: CorpusReader<'_>, path: &Path, revision: &str) -> Result<CommittedCorpus> {
1221:     ensure!(
1222:         valid_oid(revision),
1223:         "committed inventory requires exact OID"
1224:     );
1225:     let tree = io
1226:         .run(path, ["ls-tree", "-r", "-z", "-l", "--full-tree", revision])
1227:         .await?;
1228:     let mut files = Vec::new();
1229:     let mut total = 0usize;
1230:     let native_tree = parse_committed_tree(&tree)?;
1231:     for (name, entry) in &native_tree {
1232:         let ordinary = matches!(entry.mode.as_str(), "100644" | "100755") && entry.kind == "blob";
1233:         let size = entry.size.unwrap_or(0);
1234:         let (bytes, skipped) = if !ordinary {
1235:             (None, Some(UNSUPPORTED_ENTRY.into()))
1236:         } else if size > 256 * 1024 {
1237:             (None, Some("file exceeds 256 KiB".into()))
1238:         } else {
1239:             total = total
1240:                 .checked_add(size)
1241:                 .context("committed corpus size overflow")?;
1242:             ensure!(total <= 16 * 1024 * 1024, "committed corpus exceeds 16 MiB");
1243:             let bytes = io.run(path, ["cat-file", "blob", &entry.oid]).await?;
1244:             ensure!(bytes.len() == size, "committed blob size mismatch");
1245:             (Some(bytes), None)
1246:         };
1247:         files.push(CommittedFile {
1248:             path: name.clone(),
1249:             oid: entry.oid.clone(),
1250:             bytes,
1251:             skipped,
1252:         });
1253:     }
1254:     Ok(CommittedCorpus {
1255:         files,
1256:         tree: native_tree,
1257:     })
1258: }
```


### helper-writer

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/execution/effects.rs` · lines 41–100 · blob `6fcfea86d5b0cca54142a4d564f20a11495c1993` · object SHA-256 `76275722e63f1b7158272d0d88c2524e5d5576d0bda33c645e2827590547febb` · excerpt SHA-256 `8d99350865249ce03e97177ef66d9191ede99b90bc299f234bf826481664e159`.

```rust
41:     pub(crate) fn reserve_execution_helper_pinned(
42:         &mut self,
43:         authority: &ExecutionAuthority,
44:         id: OperationId,
45:         native: bool,
46:         path: &std::path::Path,
47:         kind: &str,
48:         driver: Option<&DriverReadTicket>,
49:     ) -> Result<()> {
50:         ensure!(path.is_absolute(), "helper path must be absolute");
51:         ensure!(
52:             matches!(kind, "git_helper" | "native_version" | "docker_probe"),
53:             "unsupported unit helper kind"
54:         );
55:         let tx = self
56:             .connection
57:             .transaction_with_behavior(TransactionBehavior::Immediate)?;
58:         if let Some(ticket) = driver {
59:             ticket.validate_current_tx(&tx)?;
60:         }
61:         let unit = validate_authority(&tx, authority, native, !native)?;
62:         ensure!(
63:             !verification::is_command_unit(&tx, unit.id)? || kind == "git_helper",
64:             "command-only verifier only permits scoped snapshot Git helpers"
65:         );
66:         ensure!(
67:             unit.phase != WORKFLOW_SOURCE_BOOTSTRAP || kind == "git_helper",
68:             "source preparation only permits registered Git helpers"
69:         );
70:         if let Some(ticket) = driver {
71:             ticket.preparation_matches(&unit)?;
72:         }
73:         let effect = ManagedEffect {
74:             id,
75:             unit_id: authority.unit_id,
76:             scope: authority.scope.clone(),
77:             kind: kind.into(),
78:             idempotency_key: format!("helper-{id}"),
79:             expected_target: path.to_string_lossy().into(),
80:             state: EffectState::Pending,
81:             receipt: BTreeMap::new(),
82:             version: 1,
83:         };
84:         let (p, g, t) = scope_keys(&effect.scope)?;
85:         tx.execute("INSERT INTO managed_effects(id,unit_id,project_id,goal_id,task_id,idempotency_key,state,version,body) VALUES(?1,?2,?3,?4,?5,?6,'pending',1,?7)",
86:             params![id.to_string(),effect.unit_id.to_string(),p,g,t,effect.idempotency_key,serde_json::to_string(&effect)?])?;
87:         tx.commit()?;
88:         Ok(())
89:     }
90:     pub(crate) fn reserve_managed_effect(
91:         &mut self,
92:         authority: &ExecutionAuthority,
93:         effect: &ManagedEffect,
94:     ) -> Result<()> {
95:         let tx = self
96:             .connection
97:             .transaction_with_behavior(TransactionBehavior::Immediate)?;
98:         reserve_effect_tx(&tx, authority, effect)?;
99:         tx.commit()?;
100:         Ok(())
```


### input-effect-cap

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/execution/native_phase.rs` · lines 1330–1346 · blob `b60f5f0fa002f113e7dcf82734652a09e533995d` · object SHA-256 `6186f883b541edf15d99764914f79c0fafa2692147a924ddc21fe08bf9349473` · excerpt SHA-256 `f24e6cc248638ea1d570aecf5b0abecd98cd04b9f957be5ed453be52b13a2974`.

```rust
1330:         let mutations = plan
1331:             .admission
1332:             .as_ref()
1333:             .map(PairRow::insert_permission)
1334:             .transpose()?;
1335:         let tx = self
1336:             .connection
1337:             .transaction_with_behavior(TransactionBehavior::Immediate)?;
1338:         let write = || -> Result<()> {
1339:             plan.owner.validate_tx(&tx)?;
1340:             plan.before.validate_tx(&tx)?;
1341:             let count: usize = tx.query_row(
1342:                 "SELECT count(*) FROM (SELECT 1 FROM managed_effects WHERE unit_id=?1 LIMIT 257)",
1343:                 [plan.effect.unit_id.to_string()],
1344:                 |r| r.get(0),
1345:             )?;
1346:             ensure!(count < 256, "Native effect admission profile exhausted");
```


### inventory-cap

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/execution/native_phase/version.rs` · lines 18–22 · blob `769c7f68b01511f12dc91b4f7b38cbd2a049c39f` · object SHA-256 `33114dcadedce4dacaf9f85789d66e4eb05b7bed7aa521477947e408bb3f3845` · excerpt SHA-256 `060216d5c483c016af123b0ae5ec1361c3fa3f28537548dadede482bb4b02a98`.

```rust
18: const ROWS: usize = 256;
19: const BODY_BYTES: usize = 8192;
20: const INVENTORY_BYTES: usize = 2 * 1024 * 1024;
21: const SELECT: &str = "SELECT id,unit_id,project_id,goal_id,task_id,idempotency_key,state,body,version FROM managed_effects WHERE unit_id=?1 ORDER BY id LIMIT 257";
22: const SHAPES: &str = "SELECT typeof(id),length(CAST(id AS BLOB)),typeof(unit_id),length(CAST(unit_id AS BLOB)),typeof(project_id),length(CAST(project_id AS BLOB)),typeof(goal_id),length(CAST(goal_id AS BLOB)),typeof(task_id),length(CAST(task_id AS BLOB)),typeof(idempotency_key),length(CAST(idempotency_key AS BLOB)),typeof(state),length(CAST(state AS BLOB)),typeof(body),length(CAST(body AS BLOB)),typeof(version),version,state IN ('pending','confirmed','resolved','unknown') FROM managed_effects WHERE unit_id=?1 ORDER BY id LIMIT 257";
```


### plan-memory

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/managed_binding/binding.rs` · lines 34–44 · blob `310a55a2a30ed1dc09ecc7b807561af10428790a` · object SHA-256 `3ec2e6c63efdbea440a7f9f2473ff8af8ed7787aa81cd836ada30d83de915978` · excerpt SHA-256 `9bcd7a8f8023cacf2b239f0568466e1566c7c70702bd44a5f2b631b7069a8876`.

```rust
34: pub(crate) struct ManagedBindingPlan {
35:     proof: Arc<NativePhaseBinding>,
36:     current: CurrentWorkflowSuccessor,
37:     session: Body<Record>,
38:     owner_raw: String,
39:     invocation: Body<NativeInvocation>,
40:     after: Body<Record>,
41:     audit_data: String,
42:     at: i64,
43:     record_mutation: ExactRowMutation,
44:     already_bound: bool,
```


### plan-construction

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/managed_binding/binding.rs` · lines 359–377 · blob `310a55a2a30ed1dc09ecc7b807561af10428790a` · object SHA-256 `3ec2e6c63efdbea440a7f9f2473ff8af8ed7787aa81cd836ada30d83de915978` · excerpt SHA-256 `2ca11577c1b490dd9f8073c3030d15a5187d8e2ca9d7aaff8677431cb45e0d7e`.

```rust
359:     let record_mutation = ExactRowMutation::new(
360:         "records",
361:         "UPDATE",
362:         Some(record_image(current.workflow(), current.workflow_raw())?),
363:         Some(record_image(after.parsed(), after.raw())?),
364:     )?;
365:     Ok(ManagedBindingPlan {
366:         proof,
367:         current,
368:         session,
369:         owner_raw,
370:         invocation,
371:         after,
372:         audit_data,
373:         at,
374:         record_mutation,
375:         already_bound,
376:     })
377: }
```


### successor-memory

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/managed_binding/successor.rs` · lines 23–44 · blob `b6779dd5f67acae63a4574d0f94f31d66a7dcf4d` · object SHA-256 `8ea5e1c24686a5c5afc56036c18c81063084d2afae3fdc5e234c38efe76efcc4` · excerpt SHA-256 `0f7813eb2de2c235000c16c0c116c5ad28d6791b2b98fddee8dd37f191c72de1`.

```rust
23: /// Private coherent read product, not Clone/Deserialize or Native authority.
24: /// Keeping the original actual plan prevents SQL/current-row origin replacement.
25: pub(crate) struct CurrentWorkflowSuccessor {
26:     original: Arc<MarkerPublicationPlan>,
27:     workflow: Arc<Body<Record>>,
28:     unit: Arc<Body<ExecutionUnit>>,
29:     count: usize,
30:     head: Option<Arc<Link>>,
31: }
32: struct Link {
33:     event: AuditEvent,
34:     raw: String,
35: }
36: impl CurrentWorkflowSuccessor {
37:     pub(in crate::state) fn copy_original(&self) -> Self {
38:         Self {
39:             original: self.original.clone(),
40:             workflow: self.workflow.clone(),
41:             unit: self.unit.clone(),
42:             count: self.count,
43:             head: self.head.clone(),
44:         }
```


### body-memory

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/managed_binding/canonical.rs` · lines 18–48 · blob `31e2aaa302ec65208b0b3679281a4a9fbfb9d944` · object SHA-256 `6f77f8b0a0218d47bd5ee6df77c147c518d6c84573ecd42a19bac9a6886e32a2` · excerpt SHA-256 `dad70beb7bbf33b981d335b15152bfe092801ae51a3ad34aeeda142288bc61a6`.

```rust
18: pub(in crate::state) struct Body<T> {
19:     raw: String,
20:     parsed: T,
21:     canonical: Vec<u8>,
22: }
23: impl<T: DeserializeOwned + Serialize> Body<T> {
24:     pub(in crate::state) fn decode(raw: String, limit: usize) -> Result<Self> {
25:         let value = decode_value(&raw, limit)?;
26:         let parsed: T = serde_json::from_value(value.clone())
27:             .map_err(|_| anyhow::anyhow!("managed body does not match its typed schema"))?;
28:         let canonical = encode(&value, limit)?;
29:         ensure!(
30:             encode(&serde_json::to_value(&parsed)?, limit)? == canonical,
31:             "managed body typed roundtrip drops or changes fields"
32:         );
33:         Ok(Self {
34:             raw,
35:             parsed,
36:             canonical,
37:         })
38:     }
39:     pub(in crate::state) fn raw(&self) -> &str {
40:         &self.raw
41:     }
42:     pub(in crate::state) fn parsed(&self) -> &T {
43:         &self.parsed
44:     }
45:     pub(super) fn digest(&self, domain: &[u8]) -> String {
46:         digest(domain, &self.canonical)
47:     }
48: }
```


### mutation-memory

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/managed_binding/permits.rs` · lines 182–230 · blob `635bef05a90bf7b509351a7bfd8a65ae901885fc` · object SHA-256 `24b5c2fffc50c40a102f4cb37bedb88113b40c6c690cfb6eed8ff83c0453aace` · excerpt SHA-256 `b4ce05e1c6b302ab06dff8758cc4ebfac6b8684ef2d480721154598910e3fc4a`.

```rust
182: /// Not serializable; only managed-binding Store code can build a full row plan.
183: /// Value::Real is rejected, rather than conflating SQLite numeric encodings.
184: #[allow(dead_code)] // Actual marker/binder producer is composed separately.
185: pub(in crate::state) struct ExactRowMutation {
186:     table: &'static str,
187:     action: &'static str,
188:     old: Vec<Value>,
189:     new: Vec<Value>,
190: }
191: #[allow(dead_code)]
192: impl ExactRowMutation {
193:     /// Copy only this exact already validated SQL image for retained-plan retry.
194:     /// This never issues a Native proof and is private to managed-binding code.
195:     pub(in crate::state) fn copy_for_transaction(&self) -> Result<Self> {
196:         Self::new(
197:             self.table,
198:             self.action,
199:             (self.action != "INSERT").then(|| self.old.clone()),
200:             (self.action != "DELETE").then(|| self.new.clone()),
201:         )
202:     }
203:     pub(in crate::state) fn new(
204:         table: &'static str,
205:         action: &'static str,
206:         old: Option<Vec<Value>>,
207:         new: Option<Vec<Value>>,
208:     ) -> Result<Self> {
209:         let n = columns(table)
210:             .ok_or_else(|| anyhow::anyhow!("unknown private permit table"))?
211:             .len();
212:         ensure!(
213:             matches!(
214:                 (action, old.is_some(), new.is_some()),
215:                 ("INSERT", false, true) | ("UPDATE", true, true) | ("DELETE", true, false)
216:             ),
217:             "invalid exact mutation action"
218:         );
219:         for image in [&old, &new].into_iter().flatten() {
220:             ensure!(
221:                 image.len() == n && image.iter().all(|v| !matches!(v, Value::Real(_))),
222:                 "incomplete exact mutation image"
223:             );
224:         }
225:         Ok(Self {
226:             table,
227:             action,
228:             old: old.unwrap_or_else(|| vec![Value::Null; n]),
229:             new: new.unwrap_or_else(|| vec![Value::Null; n]),
230:         })
```


### waiting-reader

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/runtime/waiting.rs` · lines 31–159 · blob `6bfce9f521cd909b40fd42e7208e411fb5ff6a91` · object SHA-256 `68b495c983534332b8de38f1ad0570161c861dc03aee91508f2da919e3470818` · excerpt SHA-256 `3b6209725296b6a6b956084f389c24c0378c949d0e6f5dc37019f0071f2ebc0d`.

```rust
31: fn observe_bounded(
32:     connection: &Connection,
33:     task: &Task,
34: ) -> Result<Option<PhaseWaitingObservation>> {
35:     let mut query = connection.prepare_cached(
36:         "SELECT CASE WHEN typeof(u.body)='text' AND length(CAST(u.body AS BLOB))<=16384 THEN u.body END,
37:          CASE WHEN typeof(r.body)='text' AND length(CAST(r.body AS BLOB))<=4096 THEN r.body END,
38:          r.state,r.version,r.parking_version,
39:          CASE WHEN length(w.reason)<=16 THEN w.reason END,w.next_due,
40:          COALESCE(u.project_id=o.project_id AND u.goal_id=o.goal_id AND u.task_id=o.task_id
41:           AND u.generation=o.execution_generation AND u.owner_epoch=o.owner_epoch
42:           AND u.owner_epoch=e.epoch AND u.native_effects_open=1
43:           AND o.marker_task_version=?4
44:           AND ow.operation_id=o.operation_id AND ow.unit_id=o.unit_id
45:           AND ow.project_id=o.project_id AND ow.goal_id=o.goal_id AND ow.task_id=o.task_id
46:           AND ow.owner_epoch=o.owner_epoch AND ow.execution_generation=o.execution_generation
47:           AND ow.origin=o.origin AND ow.provider=o.provider
48:           AND d.owner_epoch=o.owner_epoch AND d.state='driving'
49:           AND r.owner_epoch=o.owner_epoch AND r.origin=o.origin
50:           AND r.start_ended=0 AND r.known_terminal=0,0),
51:          COALESCE(w.provider=o.provider AND w.account_key='unknown' AND w.resume_state='preparing',0),
52:          o.unit_id,o.provider,o.phase,o.execution_generation,o.owner_epoch,u.version,o.operation_id,o.origin
53:          FROM managed_phase_operations o
54:          LEFT JOIN execution_units u ON u.id=o.unit_id
55:          LEFT JOIN managed_phase_owners ow ON ow.owner_id=o.owner_id
56:          LEFT JOIN managed_phase_readiness r ON r.operation_id=o.operation_id
57:          LEFT JOIN quota_waiters w ON w.unit_id=o.unit_id
58:          LEFT JOIN task_drivers d ON d.task_id=o.task_id AND d.goal_id=o.goal_id AND d.project_id=o.project_id
59:          LEFT JOIN runtime_epoch e ON e.singleton=1
60:          WHERE o.project_id=?1 AND o.goal_id=?2 AND o.task_id=?3 AND o.phase_open=1 LIMIT 2"
61:     )?;
62:     let mut rows = query.query(params![
63:         task.project_id.to_string(),
64:         task.goal_id.to_string(),
65:         task.id.to_string(),
66:         task.version
67:     ])?;
68:     let Some(row) = rows.next()? else {
69:         return Ok(None);
70:     };
71:     let observation = (|| -> Result<Option<PhaseWaitingObservation>> {
72:         if !row.get::<_, bool>(7)? {
73:             return Ok(Some(HELD));
74:         }
75:         let state: String = row.get(2)?;
76:         if matches!(state.as_str(), "held" | "deferred") {
77:             return Ok(Some(HELD));
78:         }
79:         if state != "parked" {
80:             return Ok(None);
81:         }
82:         let raw: String = row.get(0)?;
83:         let unit: ExecutionUnit = serde_json::from_str(&raw)?;
84:         let ready: Value = serde_json::from_str(&row.get::<_, String>(1)?)?;
85:         let version: u64 = row.get(3)?;
86:         let parking: Option<u64> = row.get(4)?;
87:         let reason: String = row.get(5)?;
88:         let due: i64 = row.get(6)?;
89:         let expected = match reason.as_str() {
90:             "quota" => WaitReason::Quota,
91:             "capacity" => WaitReason::Capacity,
92:             _ => return Ok(Some(HELD)),
93:         };
94:         let matches = row.get::<_, bool>(8)?
95:             && parking == Some(version)
96:             && ready["state"] == "parked"
97:             && ready["version"] == version
98:             && ready["parking_version"] == version
99:             && ready["start_ended"] == false
100:             && ready["known_terminal"] == false
101:             && unit.id.to_string() == row.get::<_, String>(9)?
102:             && unit.scope == task.scope()
103:             && unit.state == UnitState::Preparing
104:             && unit.native_effects_open
105:             && unit.wait_reason == Some(expected)
106:             && unit.session_id.is_none()
107:             && unit.provider == row.get::<_, String>(10)?
108:             && unit.phase == row.get::<_, String>(11)?
109:             && unit.generation == row.get::<_, u64>(12)?
110:             && unit.owner_epoch == row.get::<_, u64>(13)?
111:             && unit.version == row.get::<_, u64>(14)?
112:             && ready["operation_id"] == row.get::<_, String>(15)?
113:             && ready["origin"] == row.get::<_, String>(16)?
114:             && ready["owner_epoch"] == unit.owner_epoch;
115:         Ok(Some(if matches {
116:             PhaseWaitingObservation::Waiting {
117:                 reason: expected,
118:                 next_due: due,
119:             }
120:         } else {
121:             HELD
122:         }))
123:     })()
124:     .unwrap_or(Some(HELD));
125:     Ok(if rows.next()?.is_some() {
126:         Some(HELD)
127:     } else {
128:         observation
129:     })
130: }
131: 
132: pub(super) fn effective_state(
133:     task: &Task,
134:     observation: Option<&PhaseWaitingObservation>,
135: ) -> TaskState {
136:     match observation {
137:         Some(PhaseWaitingObservation::Waiting {
138:             reason: WaitReason::Quota,
139:             ..
140:         }) => TaskState::WaitingQuota,
141:         Some(PhaseWaitingObservation::Waiting {
142:             reason: WaitReason::Capacity,
143:             ..
144:         }) => TaskState::WaitingCapacity,
145:         _ => task.state,
146:     }
147: }
148: 
149: impl Store {
150:     pub(crate) fn phase_waiting_observation(
151:         &self,
152:         task: &Task,
153:         goal: &Goal,
154:     ) -> Result<Option<PhaseWaitingObservation>> {
155:         let tx = self.connection.unchecked_transaction()?;
156:         let observation = observe(&tx, task, goal);
157:         tx.commit()?;
158:         Ok(observation)
159:     }
```


### goal-status

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/runtime/goals.rs` · lines 312–335 · blob `a1a84ad03f6735d720384a08c5ffc082cf72e634` · object SHA-256 `d5622623d1689545c67365ae41dd78055240161bc5306c96468d160843142529` · excerpt SHA-256 `43d5c628fdddbcbcf090174d125230575a96f973862bb474d943921c60d80e91`.

```rust
312:             return Ok(response);
313:         }
314:         let goal = current_goal(&tx, project, id)?;
315:         let tasks = scoped_tasks(&tx, &goal)?;
316:         let mut states = BTreeMap::new();
317:         for task in &tasks {
318:             let observation = super::waiting::observe(&tx, task, &goal);
319:             let state = super::waiting::effective_state(task, observation.as_ref());
320:             *states
321:                 .entry(
322:                     serde_json::to_value(state)?
323:                         .as_str()
324:                         .context("Task state encoding")?
325:                         .to_owned(),
326:                 )
327:                 .or_insert(0) += 1;
328:         }
329:         let initial_driver:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM scheduler_tasks s JOIN task_drivers d ON d.task_id=s.task_id AND d.goal_id=s.goal_id AND d.project_id=s.project_id WHERE s.goal_id=?1 AND s.project_id=?2 AND s.attention IS NULL AND d.owner_epoch=?3 AND d.state='driving')",params![id.to_string(),project.to_string(),ingress.identity().1],|r|r.get(0))?;
330:         let response = ControlResponse::GoalFacts {
331:             goal: id,
332:             version: goal.version,
333:             state: goal.state,
334:             task_count: tasks.len(),
335:             states,
```


### goal-page

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/runtime/goals.rs` · lines 529–555 · blob `a1a84ad03f6735d720384a08c5ffc082cf72e634` · object SHA-256 `d5622623d1689545c67365ae41dd78055240161bc5306c96468d160843142529` · excerpt SHA-256 `8d6099449be6244998d9e09d932f596d2bdcde47ae246516baa91a47d6dc3149`.

```rust
529:         let eligible = all
530:             .iter()
531:             .filter(|t| after.is_none_or(|a| t.id > a))
532:             .collect::<Vec<_>>();
533:         let mut facts = Vec::new();
534:         let mut next = None;
535:         for task in eligible.iter().take(maximum) {
536:             ensure!(
537:                 task.phase.as_ref().is_none_or(|p| p.len() <= 128),
538:                 "Task phase exceeds status bound"
539:             );
540:             facts.push(TaskFacts {
541:                 scope: task.scope(),
542:                 version: task.version,
543:                 state: super::waiting::effective_state(
544:                     task,
545:                     super::waiting::observe(&tx, task, &goal).as_ref(),
546:                 ),
547:                 phase: task.phase.clone(),
548:             });
549:             let more = eligible.len() > facts.len();
550:             let candidate = ControlResponse::GoalTaskPage {
551:                 goal: id,
552:                 version: goal.version,
553:                 tasks: facts.clone(),
554:                 next: more.then_some(task.id),
555:             };
```


### control-read

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/runtime/control.rs` · lines 342–363 · blob `e7ddbd3ea59d45a8da2dfb33905ae1c9448b000f` · object SHA-256 `03ffcab929390bb5d603bcc668215a386580f629662c9344ffc2f8907b749d2f` · excerpt SHA-256 `8608b5e1c407420cc613ca19795056fb0ca149c93836f8dea3e2ab49248e5e74`.

```rust
342:         if let ControlAction::GoalStatus { project, goal } = &request.action {
343:             return self
344:                 .owner
345:                 .store
346:                 .lock()
347:                 .map_err(|_| anyhow::anyhow!("state poisoned"))?
348:                 .runtime_goal_facts(&ingress, *project, *goal);
349:         }
350:         if let ControlAction::GoalTasks {
351:             project,
352:             goal,
353:             after,
354:             maximum,
355:         } = &request.action
356:         {
357:             return self
358:                 .owner
359:                 .store
360:                 .lock()
361:                 .map_err(|_| anyhow::anyhow!("state poisoned"))?
362:                 .runtime_goal_task_page(&ingress, *project, *goal, *after, *maximum);
363:         }
```


### cli-read

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/main.rs` · lines 185–224 · blob `e40bb9c3d690b09c98c42d501af9bb778ed1530b` · object SHA-256 `f88469d7e5dd97c2ebeaa54813d601e892f1ebfd25ee64550b71a1dcd7b606f9` · excerpt SHA-256 `976bbaa0d47aed6ddea8a1a7282c46f89c5dd3891556fcf436cadb60449811a4`.

```rust
185:             read => {
186:                 let (goal, selector, json, page) = match read {
187:                     GoalCommand::Status {
188:                         goal,
189:                         project,
190:                         json,
191:                     } => (goal, project, json, None),
192:                     GoalCommand::Tasks {
193:                         goal,
194:                         project,
195:                         after,
196:                         maximum,
197:                         json,
198:                     } => (goal, project, json, Some((after, maximum))),
199:                     _ => unreachable!(),
200:                 };
201:                 let (project, _) = resolve_project(state, selector).await?;
202:                 let action = match page {
203:                     Some((after, maximum)) => ControlAction::GoalTasks {
204:                         project,
205:                         goal,
206:                         after,
207:                         maximum: usize::from(maximum),
208:                     },
209:                     None => ControlAction::GoalStatus { project, goal },
210:                 };
211:                 let response = client::request(state, action).await?;
212:                 ensure!(
213:                     matches!(
214:                         (&response, page),
215:                         (
216:                             ControlResponse::GoalFacts { .. }
217:                                 | ControlResponse::GoalProposalFacts { .. },
218:                             None
219:                         ) | (ControlResponse::GoalTaskPage { .. }, Some(_))
220:                     ),
221:                     "unexpected Goal response"
222:                 );
223:                 return print_control(response, json, false);
224:             }
```


### cli-format

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/main.rs` · lines 306–339 · blob `e40bb9c3d690b09c98c42d501af9bb778ed1530b` · object SHA-256 `f88469d7e5dd97c2ebeaa54813d601e892f1ebfd25ee64550b71a1dcd7b606f9` · excerpt SHA-256 `28f77c0a02c5dcbf843322df5b13e5ccd9763d145899e4407cea462da9c39e9d`.

```rust
306: fn print_control(response: ControlResponse, json: bool, metadata: bool) -> Result<()> {
307:     let observation = if metadata {
308:         "runtime_metadata"
309:     } else {
310:         "independent_scoped_observation"
311:     };
312:     let unavailable = if metadata {
313:         vec![
314:             "project_goal_task_hierarchy",
315:             "effective_native_limits",
316:             "unit_provider_evidence",
317:             "wait_cleanup_details",
318:         ]
319:     } else {
320:         vec![
321:             "native_dispatch",
322:             "unit_provider_evidence",
323:             "wait_cleanup_details",
324:         ]
325:     };
326:     let output = serde_json::json!({
327:         "observation": observation, "complete": false,
328:         "unavailable_fields": unavailable, "facts": response,
329:     });
330:     if json {
331:         println!("{}", serde_json::to_string_pretty(&output)?);
332:     } else {
333:         println!(
334:             "{}\nIncomplete observation: {}",
335:             serde_json::to_string_pretty(&output["facts"])?,
336:             unavailable.join(", ")
337:         );
338:     }
339:     Ok(())
```


### context-input

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/workflow_source/native_handoff.rs` · lines 538–563 · blob `8acfa9a0aa5eb1415e86ec0e27ca577d1bb920cd` · object SHA-256 `6fb8610c82d5844109ce61ddabaf961da8efb54b570c34c15cc0829f12bc155a` · excerpt SHA-256 `1707241753f4503e7138f18bc830c9b2a22374e977a8e9908430ca1fce9ac75f`.

```rust
538:         // This temporary frame checker is not stored in the Source custody.
539:         let proof = InitialInputFrame {
540:             producer: self.clone(),
541:             slot: slot.clone(),
542:             frame: state.frame.clone(),
543:             unit: unit.clone(),
544:         };
545:         proof.validate_context(task, record, context)?;
546:         let config = state.frame.config.agents.get(&task.executor);
547:         let allocation = selected.allocate(
548:             ManagedInput {
549:                 agent: task.executor.clone(),
550:                 authority: unit.authority(),
551:                 artifact: None,
552:                 input: PreparedInput {
553:                     scope: context.scope.clone(),
554:                     kind: InputKind::ContextPack,
555:                     revision: context.revision.clone(),
556:                     version: context.version,
557:                     source_versions: context.source_hashes.clone(),
558:                     payload: serde_json::to_string(&context.data)?,
559:                 },
560:             },
561:             config.and_then(|c| c.model.clone()),
562:             config.and_then(|c| c.effort.clone()),
563:         )?;
```


### input-limit

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/phase.rs` · lines 199–260 · blob `dac28d202bf05b4b257086844fbd944746b3480b` · object SHA-256 `ac02ccb3dc583f4e0b99ac23aad0cdc1adbc45314a805100bcf51d0a114991fe` · excerpt SHA-256 `e0862d9964e0fee32c63c6c4412dbb147316535a8116348648d25bfe0b6472df`.

```rust
199: /// Encode all input fields into a finite original template. A bounded writer
200: /// enforces the complete escaped encoding cost, including keys and overhead.
201: pub(crate) fn encode_input(input: &PreparedInput) -> Result<Vec<u8>> {
202:     ensure!(
203:         input.scope.goal_id.is_some()
204:             && input.scope.task_id.is_some()
205:             && input.version > 0
206:             && super::valid_oid(&input.revision)
207:             && !input.payload.is_empty()
208:             && input.payload.len() <= 1024 * 1024
209:             && input.source_versions.len() <= 4096
210:             && input.source_versions.iter().all(|(key, value)| {
211:                 !key.is_empty()
212:                     && key.len() <= 4096
213:                     && !key.chars().any(char::is_control)
214:                     && !value.is_empty()
215:                     && value.len() <= 128
216:                     && !value.chars().any(char::is_control)
217:             }),
218:         "native allocation input bounds unavailable"
219:     );
220:     #[derive(Serialize)]
221:     struct Frame<'a> {
222:         scope: &'a Scope,
223:         kind: &'static str,
224:         revision: &'a str,
225:         version: u64,
226:         source_versions: &'a std::collections::BTreeMap<String, String>,
227:         payload: &'a str,
228:     }
229:     struct Limited(Vec<u8>);
230:     impl Write for Limited {
231:         fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
232:             if bytes.len() > INPUT_BYTES.saturating_sub(self.0.len()) {
233:                 return Err(std::io::Error::new(
234:                     std::io::ErrorKind::InvalidData,
235:                     "native input encoding exceeds bound",
236:                 ));
237:             }
238:             self.0.extend_from_slice(bytes);
239:             Ok(bytes.len())
240:         }
241:         fn flush(&mut self) -> std::io::Result<()> {
242:             Ok(())
243:         }
244:     }
245:     let mut encoded = Limited(Vec::new());
246:     serde_json::to_writer(
247:         &mut encoded,
248:         &Frame {
249:             scope: &input.scope,
250:             kind: match input.kind {
251:                 crate::adapter::InputKind::ContextPack => "context_pack",
252:                 crate::adapter::InputKind::ReviewBundle => "review_bundle",
253:             },
254:             revision: &input.revision,
255:             version: input.version,
256:             source_versions: &input.source_versions,
257:             payload: &input.payload,
258:         },
259:     )?;
260:     Ok(encoded.0)
```


### frame-limit

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/phase.rs` · lines 114–120 · blob `dac28d202bf05b4b257086844fbd944746b3480b` · object SHA-256 `ac02ccb3dc583f4e0b99ac23aad0cdc1adbc45314a805100bcf51d0a114991fe` · excerpt SHA-256 `2341b9135c4de339591dcabb8ea395294ed2cb3c4767b385b308dfc4a2477d05`.

```rust
114:     }
115: }
116: 
117: const UNIT_BYTES: usize = 16 * 1024;
118: const INPUT_BYTES: usize = 2 * 1024 * 1024;
119: 
120: /// Provisional current-unit snapshot, on a separate read-only connection. Full
```


### context-shape

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/domain.rs` · lines 428–436 · blob `a320327fff917543dca49101f6c86616884540db` · object SHA-256 `7903da7d3c61c607cf14d01f849983584e41b1f22e8b2cce69a3aa2218615df3` · excerpt SHA-256 `e357bf680bdbf29e81a8bfbdda71d5f44b122989085df6a8f4258d5a623370d2`.

```rust
428: #[derive(Debug, Clone, Deserialize, Serialize)]
429: #[serde(deny_unknown_fields)]
430: pub struct ContextVersion {
431:     pub scope: Scope,
432:     pub version: u64,
433:     pub revision: String,
434:     pub source_hashes: std::collections::BTreeMap<String, String>,
435:     pub data: Value,
436: }
```


### scope-charge

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/managed_binding/publication.rs` · lines 301–375 · blob `4f71978344a6aabed5b1372c2c5a59f3f9a3e68e` · object SHA-256 `2a9317663dc0ed7f986d21275b0dfb3e4501f8de6e9f3b5eb157eff8d7999bb6` · excerpt SHA-256 `78a8609c699f5c9c4ad78118fe562974d79330aa796d621867c9ed317d5836d8`.

```rust
301: /// Complete scoped encoded body costs and open audit reservations. Aggregate
302: /// queries copy no bodies; every surface has an explicit row/byte refusal bound.
303: /// This is accounting, not a Native/Driver permission or a latency guarantee.
304: pub(super) fn charged_scope_bytes(tx: &Transaction<'_>, scope: &Scope) -> Result<u64> {
305:     let p = scope.project_id.to_string();
306:     let g = scope.goal_id.context("budget Goal missing")?.to_string();
307:     let t = scope.task_id.context("budget Task missing")?.to_string();
308:     let mut used = 0u64;
309:     // Compiled table names only, all values parameterized. This includes the
310:     // entire Task history rather than hiding older operation/Session costs.
311:     for table in [
312:         "records",
313:         "context_versions",
314:         "execution_units",
315:         "managed_phase_operations",
316:         "managed_phase_owners",
317:         "managed_phase_inputs",
318:         "workflow_native_contracts",
319:         "task_drivers",
320:         "source_recoveries",
321:         "native_invocations",
322:         "native_results",
323:         "scoped_session_identities",
324:         "result_artifacts",
325:         "resource_leases",
326:         "managed_effects",
327:         "verification_runs",
328:     ] {
329:         let (count, bytes): (u64, u64) = tx.query_row(
330:             &format!("SELECT count(*),COALESCE(sum(bytes),0) FROM (SELECT length(CAST(body AS BLOB)) bytes FROM {table} WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 LIMIT 4097)"),
331:             params![p, g, t], |r| Ok((r.get(0)?, r.get(1)?)),
332:         )?;
333:         ensure!(
334:             count <= 4096 && bytes <= WORKFLOW_BYTES,
335:             "complete marker budget surface exceeds bound"
336:         );
337:         used = used.checked_add(bytes).context("marker budget overflow")?;
338:     }
339:     for table in [
340:         "managed_marker_bodies",
341:         "managed_phase_admissions",
342:         "managed_phase_readiness",
343:     ] {
344:         let (count, bytes): (u64, u64) = tx.query_row(
345:             &format!("SELECT count(*),COALESCE(sum(bytes),0) FROM (SELECT length(CAST(s.body AS BLOB)) bytes FROM {table} s JOIN managed_phase_operations o ON o.operation_id=s.operation_id WHERE o.project_id=?1 AND o.goal_id=?2 AND o.task_id=?3 LIMIT 4097)"),
346:             params![p, g, t], |r| Ok((r.get(0)?, r.get(1)?)),
347:         )?;
348:         ensure!(
349:             count <= 4096 && bytes <= WORKFLOW_BYTES,
350:             "complete marker budget relation exceeds bound"
351:         );
352:         used = used.checked_add(bytes).context("marker budget overflow")?;
353:     }
354:     let (count, audit_bytes): (u64, u64) = tx.query_row(
355:         "SELECT count(*),COALESCE(sum(bytes),0) FROM (SELECT length(CAST(data AS BLOB)) bytes FROM audit WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind GLOB 'rrx.private.workflow.*' LIMIT 65537)",
356:         params![p, g, t], |r| Ok((r.get(0)?, r.get(1)?)),
357:     )?;
358:     ensure!(
359:         count <= 65536 && audit_bytes <= WORKFLOW_BYTES,
360:         "complete marker audit budget exceeds bound"
361:     );
362:     used = used
363:         .checked_add(audit_bytes)
364:         .context("marker budget overflow")?;
365:     let (open, reserved): (u64, u64) = tx.query_row(
366:         "WITH spent AS (SELECT json_extract(data,'$.private_operation_ref') operation_id,sum(length(CAST(data AS BLOB))) bytes FROM audit WHERE project_id=?1 AND goal_id=?2 AND task_id=?3 AND kind GLOB 'rrx.private.workflow.*' GROUP BY json_extract(data,'$.private_operation_ref')) SELECT count(*),COALESCE(sum(max(0,1048576-COALESCE(s.bytes,0))),0) FROM managed_phase_operations o LEFT JOIN spent s ON s.operation_id=o.operation_id WHERE o.project_id=?1 AND o.goal_id=?2 AND o.task_id=?3 AND o.phase_open=1",
367:         params![p, g, t], |r| Ok((r.get(0)?, r.get(1)?)),
368:     )?;
369:     ensure!(
370:         open <= 128 && reserved <= WORKFLOW_BYTES,
371:         "open marker audit reservation exceeds bound"
372:     );
373:     used.checked_add(reserved)
374:         .context("complete scope budget overflow")
375: }
```


### scope-bound

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/managed_binding/marker_rows.rs` · lines 10–16 · blob `22aa02fa8c6413f88dd7464a242bffbaeed06799` · object SHA-256 `caedca52a25d245fc9b10473bfe9d79a536a616cb4fd47fa4e5a4ea4a8f5a5b2` · excerpt SHA-256 `978ea472e1016ee4ba63f2ad538e8e06944af90c6bb806b4dd358297353ad172`.

```rust
10: use rusqlite::{Transaction, params_from_iter, types::Value as SqlValue};
11: use serde_json::json;
12: 
13: pub(super) const LINK_RESERVE_BYTES: u64 = 256 * 4096;
14: pub(super) const WORKFLOW_BYTES: u64 = 128 * 1024 * 1024;
15: 
16: /// Only this module can construct a row. Table/column names are compiled;
```


### cleanup-write

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/execution.rs` · lines 1129–1160 · blob `ba58dbbe685924efaf87bea0885eb6b52a0866fe` · object SHA-256 `f30614eacadc146368e5c0545060c0691a77b685271a4dc87f8476f502719d0a` · excerpt SHA-256 `ee79481cbbdac6ad1e5c14fcbe751d18c43a90dcea13fd366c9fc78e24dd0ed9`.

```rust
1129:     pub(crate) fn record_execution_cleanup(
1130:         &mut self,
1131:         observation: &CleanupObservation,
1132:     ) -> Result<()> {
1133:         ensure!(
1134:             observation.coverage.len() <= 32
1135:                 && observation.remaining.len() <= 1024
1136:                 && observation.errors.len() <= 32
1137:                 && cleanup::valid_cleanup_actions(&observation.actions),
1138:             "cleanup observation bound exceeded"
1139:         );
1140:         let tx = self
1141:             .connection
1142:             .transaction_with_behavior(TransactionBehavior::Immediate)?;
1143:         let mut unit = unit_tx(&tx, observation.unit_id)?;
1144:         unit.cleanup = observation.outcome;
1145:         tx.execute(
1146:             "INSERT INTO cleanup_observations(unit_id,at,body) VALUES(?1,?2,?3)",
1147:             params![
1148:                 unit.id.to_string(),
1149:                 observation.at,
1150:                 serde_json::to_string(observation)?
1151:             ],
1152:         )?;
1153:         append_event(
1154:             &tx,
1155:             &unit.scope,
1156:             "execution.cleanup_observed",
1157:             json!({"unit":unit.id,"work":unit.work,"cleanup":unit.cleanup,"coverage":observation.coverage}),
1158:         )?;
1159:         tx.commit()?;
1160:         Ok(())
```


### cleanup-overlay

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/state/execution.rs` · lines 249–267 · blob `ba58dbbe685924efaf87bea0885eb6b52a0866fe` · object SHA-256 `f30614eacadc146368e5c0545060c0691a77b685271a4dc87f8476f502719d0a` · excerpt SHA-256 `9dc1550eb90067363b268c4f64c042d883844db3f9cf453917b0e52c2e92e0ad`.

```rust
249:     // Cleanup is an independent factual projection. Appending a janitor receipt
250:     // cannot supersede native/finalization authority or invalidate a capture CAS.
251:     if let Some((at, body)) = tx
252:         .query_row(
253:             "SELECT at,body FROM cleanup_observations WHERE unit_id=?1 ORDER BY rowid DESC LIMIT 1",
254:             [id.to_string()],
255:             |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
256:         )
257:         .optional()?
258:     {
259:         let observation: CleanupObservation = decode(body)?;
260:         ensure!(
261:             observation.unit_id == id && observation.at == at,
262:             "cleanup observation indexed/body mismatch"
263:         );
264:         unit.cleanup = observation.outcome;
265:     }
266:     Ok(unit)
267: }
```


### fixture-setup

CA `95ef0e783c0b18c90dfef23ba1b77b9dc67735f1` · `crates/rrx/src/runtime/installation/tests.rs` · lines 17–91 · blob `0e3b5973f81720e8b073feb252093e493d3b3f8d` · object SHA-256 `0ee62dffb2e55aaf3f562029be40aea5e0982ab58287a0c97fe990304adca369` · excerpt SHA-256 `c6de09829908164d5e8e4e30649bdcf6fcd2deaed8c6c7c9b6662a8412203fa5`.

```rust
17: fn fixture(provider: &str, declared: bool) -> ControlFixture {
18:     fixture_with(provider, declared, |_| {})
19: }
20: fn fixture_with(
21:     provider: &str,
22:     declared: bool,
23:     configure: impl FnOnce(&mut crate::config::Config),
24: ) -> ControlFixture {
25:     ControlFixture::configured(|dir| {
26:         let path = dir.join("configured-protocol-fixture");
27:         let source = include_str!("../../execution/native/native_fixture.py");
28:         // Hold only the external protocol peer's bootstrap, after a real spawn.
29:         // This is not an rrx admission/authority or permission switch.
30:         let source=source.replacen("while True: time.sleep(0.02)","while not os.path.exists(os.path.join(os.environ[\"RRX_OUTPUT_DIR\"], \"fixture-bootstrap-release\")): time.sleep(0.02)",1);
31:         std::fs::write(&path,format!("#!/usr/bin/python3\nPROVIDER={provider:?}\nBOOTSTRAP_HOLD=True\nWORKFLOW_SCENARIO='answer-normal'\n{source}")).unwrap();
32:         std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
33:         let mut config = crate::config::Config {
34:             minimum_workflow: crate::config::WorkflowClass::Quick,
35:             ..Default::default()
36:         };
37:         config.workflow.risk_mapping = [crate::config::WorkflowClass::Quick; 4];
38:         config.agents.insert(
39:             "worker".into(),
40:             AgentConfig {
41:                 provider: Some(provider.into()),
42:                 command: vec![path.to_string_lossy().into()],
43:                 compatibility: declared.then(|| NativeCompatConfig {
44:                     profile: "rrx-native-inherited-v1".into(),
45:                     cli_version: if provider == "claude" {
46:                         "2.1.283"
47:                     } else {
48:                         "codex-cli 0.160.0"
49:                     }
50:                     .into(),
51:                     settings: "inherited".into(),
52:                     user_hooks: vec![],
53:                 }),
54:                 ..Default::default()
55:             },
56:         );
57:         for alias in ["rev-a", "rev-b"] {
58:             config
59:                 .agents
60:                 .insert(alias.into(), config.agents["worker"].clone());
61:         }
62:         configure(&mut config);
63:         config
64:     })
65: }
66: async fn accept(f: &ControlFixture, count: usize) -> (GoalId, Vec<Task>) {
67:     let mut p = plan();
68:     p.tasks[0].risk = RiskClass::R1;
69:     p.tasks[0].reviewers = vec!["rev-a".into(), "rev-b".into()];
70:     let original = p.tasks[0].clone();
71:     p.tasks = (0..count)
72:         .map(|i| {
73:             let mut task = original.clone();
74:             task.key = format!("task-{i}");
75:             task.title = format!("work-{i}");
76:             task
77:         })
78:         .collect();
79:     let goal = f.create(p).await;
80:     let store = f.owner.store.lock().unwrap();
81:     let tasks = store
82:         .goal(goal)
83:         .unwrap()
84:         .unwrap()
85:         .dag
86:         .nodes
87:         .iter()
88:         .map(|id| store.task(*id).unwrap().unwrap())
89:         .collect();
90:     (goal, tasks)
91: }
```


### fixture-bound

CA `95ef0e783c0b18c90dfef23ba1b77b9dc67735f1` · `crates/rrx/src/runtime/installation/tests/activation.rs` · lines 119–215 · blob `d1c8d2c81842d857db789ea7cbfe4d9d13f89721` · object SHA-256 `2cbc0ac2d83b68e2df5681001adf64b512620c36a754b89d4eac46deac757695` · excerpt SHA-256 `5063773132fd837fb2800179cb384703b026f603b842a1aa455a9df63a48a312`.

```rust
119: #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
120: async fn ca1_actual_activation_gates_marker_and_record_only_bound() {
121:     for provider in ["claude", "codex"] {
122:         let mut f = fixture(provider, true);
123:         f.register_real_git_project();
124:         let (_, tasks) = accept(&f, 1).await;
125:         let expected = Arc::new(Mutex::new(None));
126:         let arrived = Arc::new(AtomicBool::new(false));
127:         let capture = expected.clone();
128:         let signal = arrived.clone();
129:         f.runtime
130:             .installed
131:             .as_ref()
132:             .ok()
133:             .unwrap()
134:             .engine
135:             .set_activation_hooks(
136:                 Some(Arc::new(move |probe| {
137:                     *capture.lock().unwrap() = Some(expected_contract(&probe));
138:                     signal.store(true, Ordering::SeqCst);
139:                     Box::pin(async {})
140:                 })),
141:                 None,
142:             );
143:         f.runtime.start().await.unwrap();
144:         wait_for(
145:             || arrived.load(Ordering::SeqCst),
146:             "SETUP: genuine retained activation input not reached",
147:         )
148:         .await;
149:         wait_for(
150:             || count(&f, "workflow_native_contracts") == 1,
151:             "CA1 activation stage: composed contract absent",
152:         )
153:         .await;
154:         let expected = expected.lock().unwrap().clone().unwrap();
155:         let contract: String = raw(&f)
156:             .query_row("SELECT body FROM workflow_native_contracts", [], |r| {
157:                 r.get(0)
158:             })
159:             .unwrap();
160:         assert_eq!(
161:             contract,
162:             String::from_utf8(canonical(&expected)).unwrap(),
163:             "CA1 members and exact activation image"
164:         );
165:         let (record, workflow) = wait_bound(&f, &tasks[0]).await;
166:         let stored = f
167:             .owner
168:             .store
169:             .lock()
170:             .unwrap()
171:             .task(tasks[0].id)
172:             .unwrap()
173:             .unwrap();
174:         let (task_version, workflow_version): (u64, u64) = raw(&f)
175:             .query_row(
176:                 "SELECT marker_task_version,marker_workflow_version FROM managed_phase_operations",
177:                 [],
178:                 |r| Ok((r.get(0)?, r.get(1)?)),
179:             )
180:             .unwrap();
181:         assert_eq!(
182:             stored.version, task_version,
183:             "CA1 record-only binder changed Task"
184:         );
185:         assert_eq!(
186:             record.version,
187:             workflow_version + 1,
188:             "CA1 binder record version"
189:         );
190:         assert_eq!(
191:             workflow_version, 9,
192:             "CA1 initial + six gate edges + Executor reservation + marker"
193:         );
194:         let advances:u64=raw(&f).query_row("SELECT count(*) FROM audit WHERE kind='rrx.private.runtime.driver_initial_input_advanced'",[],|r|r.get(0)).unwrap();
195:         assert_eq!(
196:             advances, 8,
197:             "CA1 activation, Reserve/Claim/Complete and Executor record windows"
198:         );
199:         assert_eq!(
200:             count(&f, "native_invocations"),
201:             1,
202:             "CA1 actual registered launch"
203:         );
204:         let after: String = raw(&f)
205:             .query_row("SELECT body FROM workflow_native_contracts", [], |r| {
206:                 r.get(0)
207:             })
208:             .unwrap();
209:         assert_eq!(
210:             after, contract,
211:             "CA1 immutable contract changed at marker/bind"
212:         );
213:         release_peer(&f, &workflow);
214:         finish(f).await;
215:     }
```


### fixture-commit

R `66f170bd33dc487fc58a963c8ef8f7d2d9ca08c4` · `crates/rrx/src/execution/native/native_fixture.py` · lines 21–44 · blob `5802c5f50b7a196008836aa6debef092eb61fb26` · object SHA-256 `0e46f8e3ec2cd6b3ab0248174eafbf134de0b621e493bb5ac478bcbdfd3fc5d2` · excerpt SHA-256 `91cbfd8f141d9c9a8e657e51f64436c9eba41006d5aad475c548df3b2b7a145c`.

```python
21: def complete(payload):
22:     if payload == "readonly-review":
23:         if PROVIDER == "claude":
24:             assert sys.argv[sys.argv.index("--permission-mode") + 1] == "plan"
25:         else:
26:             assert NATIVE_SANDBOX == "read-only"
27:         subprocess.run(["/usr/bin/git", "rev-parse", "HEAD"], check=True, stdout=subprocess.DEVNULL)
28:         try:
29:             with open("fixture-review-write", "w") as out: out.write("must be refused")
30:         except PermissionError:
31:             pass
32:         else:
33:             raise RuntimeError("readonly source accepted a normal write")
34:         return
35:     with open(os.path.join(os.environ["RRX_OUTPUT_DIR"], "fixture-ready"), "w") as ready:
36:         ready.write("ready\n")
37:     release = os.path.join(os.environ["RRX_OUTPUT_DIR"], "fixture-release")
38:     while not os.path.exists(release):
39:         time.sleep(0.02)
40:     with open("fixture-result.txt", "w") as result:
41:         result.write(os.environ["RRX_UNIT_ID"] + "\n")
42:     # The fixture qualifies protocol correlation, not the installed shim/native-agent matrix.
43:     subprocess.run(["/usr/bin/git", "add", "fixture-result.txt"], check=True, stdout=subprocess.DEVNULL)
44:     subprocess.run(["/usr/bin/git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "-c", "commit.gpgsign=false", "commit", "-m", "protocol fixture"], check=True, stdout=subprocess.DEVNULL)
```


### RN-current-link

RN-current `e70faa9dc900c5df0dd26d9983425cf6d63876a3` · `crates/rrx/src/state/managed_binding/closure.rs` · lines 54–64 · blob `32490867b2e2989591a7add30d9e89a0a1e8d1df` · object SHA-256 `e7b91f77e5609c95b39763f4d50fdba9869e12313c369d9dee35174b12b42519` · excerpt SHA-256 `5b2c0f5974f6cdadd4cf6e68aa33dc5f3c1879f8dfd48210d9f5e625ec577b86`.

```rust
54: /// Borrowed W4 inputs of the SAME retained plan for one writer call.
55: /// Fields and construction stay in this module; owns no images or payload.
56: pub(in crate::state) struct PhaseClosedLink<'a> {
57:     closure: &'a UnlinkedPhaseClosure,
58:     audit_data: &'a str,
59: }
60: 
61: fn workflow_delta(
62:     before: &Record,
63:     task: &crate::domain::Task,
64:     original_unit: &crate::execution::ExecutionUnit,
```


### RN-current-link-issuer

RN-current `e70faa9dc900c5df0dd26d9983425cf6d63876a3` · `crates/rrx/src/state/managed_binding/closure.rs` · lines 224–235 · blob `32490867b2e2989591a7add30d9e89a0a1e8d1df` · object SHA-256 `e7b91f77e5609c95b39763f4d50fdba9869e12313c369d9dee35174b12b42519` · excerpt SHA-256 `31fc41e69771fcf1ab3b863059e7b93216c7ee9dfdf9323b2b65ff49169bd163`.

```rust
224:     })
225: }
226: impl UnlinkedPhaseClosure {
227:     pub(in crate::state) fn link<'a>(&'a self, audit_data: &'a str) -> PhaseClosedLink<'a> {
228:         PhaseClosedLink {
229:             closure: self,
230:             audit_data,
231:         }
232:     }
233:     pub(in crate::state) fn attempt(&self) -> usize {
234:         self.attempt
235:     }
```


### RN-current-consumer

RN-current `e70faa9dc900c5df0dd26d9983425cf6d63876a3` · `crates/rrx/src/state/execution/native_phase/nonsuccess.rs` · lines 155–176 · blob `ea9d14630dad972851702fbad715878342fe6c64` · object SHA-256 `9cdf2556b4b54f4d6ce3f114d6118cc858c4073a37ebbca1c6d30dfb85952b71` · excerpt SHA-256 `f8c33dfc9758aa6f49782de50fe2add6c2821655b326c913074411ef021d35af`.

```rust
155:         {
156:             let budget = InventoryBudget::new(&tx)?;
157:             if let Err(cause) = budget.finish(validate_open(&tx, plan)) {
158:                 return Ok(NativeNonSuccessWrite::Conflict(cause));
159:             }
160:         }
161:         let reader = NonSuccessReader(());
162:         material.images.write_tx(
163:             &reader,
164:             &tx,
165:             &self.binding_permits,
166:             plan.proof.launch().marker(),
167:             plan.images.link(&plan.audit_data),
168:             |tx| {
169:                 plan.proof
170:                     .closure(&reader)
171:                     .unit()
172:                     .write_retired_tx(tx, &plan.unit_after)
173:             },
174:         )?;
175:         tx.commit()?;
176:         Ok(NativeNonSuccessWrite::Known(PhaseClosedAcknowledgment {
```


### CA-current-version-monitor

CA-current `2c24f3f6e66f45692e416c77dc3e8d64fcbc9276` · `crates/rrx/src/execution/native/version.rs` · lines 705–745 · blob `bf1ee4ffe2ccd151e85e48a7b501a5f89d077ca2` · object SHA-256 `45c8884f33ed2f3599af0dbf21ac6363650e2c9ac143fba97bcbfeb3702a7141` · excerpt SHA-256 `3d4bfea5270f8989539621f9ff537428a951f838032e0d4ce0703a584d5fcccd`.

```rust
705:                         Ok(0) => out_open = false,
706:                         Ok(n) => {
707:                             if !output.observe(&out_buffer[..n],true) { break; }
708:                         }, Err(_) => break,
709:                     }
710:                 },
711:                 read = err.read(&mut err_buffer[..read_len]), if err_open => {
712:                     match read { Ok(0) => err_open = false, Ok(n) => {
713:                         if !output.observe(&err_buffer[..n],false) { break; }
714:                     }, Err(_) => break }
715:                 },
716:                 _ = tokio::time::sleep_until(end) => break,
717:                 _ = fence.tick() => {
718:                     let valid = owner.upgrade().is_some_and(|owner| {
719:                         owner.store.lock().ok().is_some_and(|mut store|
720:                             store.validate_phase_version_fence(&helper.plan,&intent).is_ok())
721:                     });
722:                     if !valid { break; }
723:                     if !exited {
724:                         // End the raw guard before the arm may run hygiene,
725:                         // which takes the same custody mutex again.
726:                         let observed_exit = { helper.raw().exited_unreaped() };
727:                         match observed_exit {
728:                             Ok(true) => {
729:                                 exited = true;
730:                                 let _ = helper.raw().hygiene();
731:                                 drain_deadline = Some(tokio::time::Instant::now()+Duration::from_secs(2));
732:                             }, Ok(false) => {}, Err(_) => break,
733:                         }
734:                     }
735:                 },
736:             }
737:         }
738:     }
739:     let group_hygiene = helper.raw().hygiene();
740:     let stop_deadline = tokio::time::Instant::now() + Duration::from_secs(10);
741:     loop {
742:         if !helper.raw().has_child() {
743:             break;
744:         }
745:         match helper.raw().reap() {
```
