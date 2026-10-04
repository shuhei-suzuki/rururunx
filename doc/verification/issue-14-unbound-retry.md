# Issue14 unbound native retry component

Requirements3 approved at1463d37: TWO independent APPROVE, noCHM/blockers.
Design2 at3c3a42f approved by TWO independent native reviewers, zeroCHM/blockers.
Source1 at49844a6 implements the approved scope; independent source review pending.
Open Issue14 explicitly requires this follow-up from Issue41. Historical base main efe9774
allowed explicit Failed+dispatch_started+session_id=None retry when the
Session loop has nothing to reject. Existing characterization is evidence of the
unsafe release behavior, not native recovery proof. No schema/permission/timeout
change proposed. Whole14/Scheduler/recovery/MVP remain OPEN.

Required gates: independent requirements review, component design review, actual
held-start Executor/Reviewer consumers and valid retry controls, compiled causal
mutants, default regressions/lint/build/affected release tests, independent source
reviews, current both-OS CI/actual source identity, limited PR merge/cleanup.

Requirements1 at535452c: BOTH independent native request_changes, each verified
High pre-marker retry/control omission and Medium equivalent EvidencePort mutant,
enforcement layer and availability disclosure. Both actual selected wrappers
closed/cleaned; no source/design approval. Coordinator inspected actual before-
marker registry/capability fail, marker/Session immutability, Store own_found and
closure/terminal fences. Requirements2 chooses dual Engine+Store enforcement with
actual positives/directStore/combinedmutants and explicit no-escape consequence.
All findings/dispositions retained. Normal no-conflict main63 composition completed
after review closure. Requirements2 re-review pending; no implementation.

Requirements2 at1f3b01e: A request_changes with one verified Medium proofgap;
B approve with the same Medium nonblocking. No Critical/High remains, but gate
notapproved. Both selected wrappers closed. Requirements3 names actual held
Running direct Store closures/generation with validcontext, preserves same-owner
binding/fail/decision and classifies Failed-generation falsecredit. Grok returned
startErr versus bound post-start actor failure split corrected; Generic-shaped
unbound terminal row control added, observation writer inventoried and header
updated. Source unchanged; independent Requirements3 delta re-review pending.

Requirements3 at1463d37: BOTH independent native APPROVE, zeroCHM and blockers,
selected wrappers actuallyclosed/cleaned. Three distinct optional Lows verified: Grok
bound outcome depends on binding commit, row-resolution mutation attribution per
guard, and co-located valid pre-marker raw-builder controls. Design1 records finite
precision/disposition with no WHAT change. Requirements amended only to qualify
the binding sentence; source/test/build untouched. Design1 independent review next;
whole14/MVP/native/F1 and genuine recovery producer remain OPEN.

Design1 c1b9d1b: BOTH independent native APPROVE, noCritical/High/blockers, B one
verified nonblockingMedium: new closure guard would mask41TRmutation evidence.
Design2 chooses explicit non-TR newguard; existing earlierTRidenticalpredicate still
fences everyaccess and preserves41causalmutants. Sharedstaticmessage mandatory;
first-configured-phasecontext, staleCAS priority, directbound/port retrybuilders
and3/4pausecounterfacts specified. Bothwrappers closed; no source edits. Design2
independent delta review required before source; no parent/native/F1 qualification.


Design2 at3c3a42f: BOTH independent native APPROVE, zeroCHM/blockers, actual selected
wrappers closed/cleaned. Low A correctly classifies removing the redundant new
TerminalRecovery exemption as equivalent/no credit. Low B identifies raw
ReadOnly/Mutating paths: both now have direct Executor held-start controls.
[Design2 results](issue-14-unbound-retry-design2-reviews.json) record the exact
reviewed source and finite dispositions. WHAT is unchanged.

Source1 at49844a6 adds nine Workflow lines and eight Store lines, plus synthetic
consumer controls and factual specification/status updates. The ONE static literal
is used at both new fences; the earlier TerminalRecovery guard remains independent.
Eight new tests execute both actor held-start errors, matching terminal persistence,
Running direct retry/generation/read-only/mutating closures, same-owner bind/fail,
terminal decision, both pre-marker offsets3/4 with raw retry/generation positive
controls, configuration repair and bound/port positive retries. Every normal held
owner is released and joined; these provide no native/managed19/death certificate.
Snapshots compare all columns/rows of every fixture owner, Record kind, context,
usage and complete audit, with both launch counters; no prefix/latest shortcut.

Default full workspace debug passes **271 Rust tests plus two doctests**, with24
ignored; affected Workflow release passes75. Fmt, all-target Clippy-Dwarnings and
both all-target builds pass. Full workspace release tests were NOT run on this
head. Historical main555 release failure remains cause/regression UNKNOWN. The
[gates](issue-14-unbound-retry-source1-gates.json) identify exact source and logs.
CI [37241088835](https://github.com/shuhei-suzuki/rururunx/actions/runs/37241088835)
passes EVERY Linux/macOS step. Actual checkout9e039953 has parentsEFE/49844a6,
complete tree and every tracked blob equal49844a6, independently obtained from
GitHub commit/tree APIs: [CI proof](issue-14-unbound-retry-source1-ci.json).

Mutation verification records25 compiled executions: **17 unique assertion-killed
operators** (15 new-fence operators plus two existing41 TerminalRecovery operators),
one additional direct Failed consumer of the SAME Store-removal operator, five
survivors and two setup failures with NO credit. Engine-only removal, Engine-only
matching-row resolution, actor-conjunct removal, EngineFailed-only and removal of
the redundant TR exemption survive as classified. Old41 direct Store TR removal
commits and fails unwrap_err; both oldTR removals commit and fail is_err, so their
causality is preserved without differing error-text credit. StateOnly-only narrowing
commits a real ReadOnly closure; same-generation-only narrowing commits a real
Running generation closure. Refusal mutations affecting pre-marker/bound/port
availability fail at actual retry unwrap, not at string comparison.

The first broad Store all-sessionless and missing-session-conjunct operators fail
in Fixture::through before reaching the retry: their initial driver's optimistic
assertion-kill classification is preserved and CORRECTED to setup_failure_no_credit.
Separate erroneous predicates scoped to actual closing Waiting/Failed retry outcomes
preserve construction and fail at the real public retry unwrap. They are distinct
operators, not retrospective qualification of the broad setup failures. All25
[exact patches](issue-14-unbound-retry-source1-mutant-patches.diff), hashes/consumer
assertions and dispositions are in the [mutant ledger](issue-14-unbound-retry-source1-mutants.json).
Final restored782fc3f has the COMPLETE49844a6 tree, clean, with all75 Workflow tests
passing. Synthetic mutation-test panics earn no native cleanup evidence.

The [rg inventory](issue-14-unbound-retry-source1-impact-rg.txt) covers workspace
actor/marker/Session readers, every retry/transition caller and Workflow writer.
A detailed source consumer rationale follows. Source review and later-composition
checks are still pending. Whole14/Scheduler/restart/native/F1/MVP remain OPEN;
unknown post-marker outcomes have no operator escape or producer authority.


### Source1 consumer impact at49844a6

All locations below are relative to that immutable source; the raw rg inventory
also includes integration controls and every actor/Session field reader.

| Consumer / location | Closure and effect rationale |
| --- | --- |
| Workflow `persist`515 / `reserve`529 / `persist_decision`1376 | The ONLY production callers of put_workflow_transition; StateOnly, ReadOnly/Mutating and terminal access modes all retain existing identity/CAS checks. |
| Workflow `initialize`617 / `escalate`704 / `request_finalization`1427 | Initial workflow or no active attempt required; cannot discharge a marked-unbound active claim. |
| Workflow `step`796 / `poll`1206 | Step delegates an existing active attempt to poll. Unbound Running waits, Failed observes failure without replay; new reservations only when active=None. |
| Workflow `release_preparation`986 | Exact original pre-marker owned attempt and owner/Record versions required. Marker=false closure remains valid. |
| Workflow `prepare_agent`1042 | Pre-marker source invalidation and registry/capability/worktree failure decisions remain valid. Marker publish1159 keeps active; returned Session binding1194 and startErr failure1203 keep active. |
| Workflow `invalidate_attempt`1508 / `invalidate_attempt_preparation`1518 | Valid pre-marker or bound/evidence outcomes close and publish a new generation/context. A raw marked-unbound closure is now refused regardless of generation; Failed→Interrupted without terminal decision already fails validation. |
| Workflow `evaluate`1674 | Agent evidence requires its bound Session. Completion closes bound native or EvidencePort attempts; no unknown unbound native completion can pass existing validation. |
| Workflow `hold`1632 / `fail`1663 | Hold and truthful same-owner failure preserve active. No new-fence refusal is introduced on these factual publications. |
| Workflow `retry`2010 | Public native Waiting/Failed admission then NEW unknown marker fence before Sessions or mutations. Resolved/pre-marker/native-bound and port outcomes retain RetryEvent and blocker behavior. |
| Workflow `resume_gate`1600 | Waiting or irreversible Failed gate reconciliation; cannot close/relaunch a nonirreversible native Failed unknown claim. |
| Workflow `cancel`1344 / `fail_task`1347 / `terminate`1350 | TerminalDecision keeps the original active reservation, so it does not enter the new closure block. |
| Workflow `release_terminal_reservation`1393 / Store493 | Existing Engine and conservative Store TerminalRecovery unbound fences remain authoritative. New guard exempts only this redundant mode; actual original41 single/combined mutants still commit when those original guards are removed. |
| Workflow `observe_gate`1324 / Store `observe_workflow_gate`616 | Other direct Workflow Record writer requires SAME Evaluating active identity and only appends actual observation/detail. No active replacement/closure, no bypass. |
| Store `put_record`347 | Explicitly rejects RecordKind::Workflow. Other public Record writers cannot replace workflow authority. |
| Store `put_workflow_transition`362 / `validate_transition`2232 | New guard inside original-active closure block, after structural transition validation, before Session scan/context/Task/Record/audit commits. ReadOnly/Mutating extra guards also remain. Raw active replacement is covered by the same original-attempt test. |
| Store Task/Record CAS / Workflow `own_task_marker_rollback`2611 | New refusal precedes Task/Record CAS; stale closing callers can get untyped refusal instead of SnapshotChanged. No closing caller parses or depends on it. Marker publication keeps active, so its typed rollback classifier is unchanged. |
| Project `status`243 / `remove`261 / Store `ensure_project_idle`1410 | Existing active Workflow blocks removal even if Task/Goal is terminal. No new status/CLI endpoint; counts and classification remain current interfaces. |
| GenericCliAdapter `start`554 | Mode/model/effort/role, request/input/binding/Store/lock/executable, Session-save/Git/preflight/spawn/binding/registry failures all return before Workflow binding. Some save their own Failed/Lost rows before Err. No error label or terminal matching row resolves Workflow's marked-unbound claim. |
| GrokAdapter `launch`238 / `start`459 | Mode/role/scope/capture/environment/registry-capacity/save_current failures precede saved-Session return. supervise319 executes after Session reservation/registration; subsequent process/actor errors are bound ONLY if Workflow binding actually commits. A binding CAS loss remains original Running marked-unbound despite later terminal Session. |
| AgentAdapter `start`206 / actual implementations | At this source the production implementors are GenericCliAdapter and GrokAdapter; synthetic test implementors are not native authority. Later provider composition needs fresh inventory and regression, not extrapolation from this source. |

No schema/threshold/allowlist/environment/permission/default setting changes. A
single bounded shared error literal is explanatory, not a typed outcome or proof.
All Generic/Grok transitive error routes remain conservative by durable marker,
actor and Session fields; source does not certify their process or reader cleanup.
