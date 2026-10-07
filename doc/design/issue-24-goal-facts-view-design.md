# Issue 24: Recorded Goal facts view — HOW

Status: WHAT/HOW approved by independent Opus high delta review; source implementation present, source tests and independent source review pending. No native, full-Workflow, MVP or merge qualification is asserted.

## 1. Baseline and unchanged read boundary

Implements the [WHAT](../requirements/issue-24-goal-facts-view-requirements.md) against `c9791175d153b4801c2fd42cc0e557b84a8e477e`. Reuse the [approved Runtime CLI read contract](runtime-cli-integration-design.md), not a new inventory/control API.

The existing chain is compiled CLI → `cli/client.rs` → observed endpoint/Hello → actual Unix peer → `HumanIngress` → `Runtime::handle_control` → transactional Store reader. `owner_current` checks the ingress's instance/epoch. `current_goal` checks selected Project membership, Goal body/index ID/version, a 4 MiB Goal body bound and the accepted immutable definition digest. `scoped_tasks` checks the complete selected inventory, each Task body/index/ownership/version, exact DAG node membership and `TaskDag::hard_order` validation of nodes, all edge endpoints/pairs and hard cycles. Preserve every check, before projection.

The CLI separately resolves Project identity/version/root. `current_goal` does not reread Project body/version in the Goal transaction. Return the selected Project ID only; do not report that separate routing version as a coherent Goal-read version. Same-UID clients may select registered Projects under existing routing; a Goal belonging to a different selected Project refuses. DTO labels introduce no principal or execution right.

Accepted facts are projected from the SAME Goal and full validated Task inventory inside one existing read transaction. Preserve existing effective waiting-state observation; add stored-state facts without changing that observation's semantics. Detach owned bounded output before socket writes; no Store guard crosses an await. No read writes acknowledgement, audit, attention, owner, epoch or domain rows. Refusal remains sanitized through the existing `Rejected`/connection failure behavior.

## 2. Explicit additive wire extension

`ControlAction`, `ControlResponse` and `TaskFacts` currently deny unknown fields. Unconditional additive response fields would break old strict clients. Keep protocol version 1 and all framing/strict-decoding predicates.

Add `view: Option<GoalReadView>` to ONLY existing `GoalStatus` and `GoalTasks`. `GoalReadView` has one snake-case wire value: `recorded_v1`. Use `default` and `skip_serializing_if = Option::is_none`; omitted or null means the existing legacy read. Unknown strings/types/fields and duplicate keys refuse. No new cursor, selector or authority field is introduced.

Add `recorded: Option<RecordedGoalStatus>` to `GoalFacts` and `recorded: Option<RecordedGoalTaskPage>` to `GoalTaskPage`, likewise defaulted/omitted when absent. Add `recorded: Option<RecordedProposalStatus>` to `GoalProposalFacts`. Retain all existing response fields and enum discriminants; `TaskFacts` itself is unchanged. Every new object denies unknown fields. All IDs, versions, states and booleans use existing typed wire encodings; counts are nonnegative integers.

Emission rule: `view=None` emits no new keys and uses the existing behavior/budgets; `recorded_v1` emits the matching non-null object or refuses. A client requesting this view treats a missing/mismatched recorded object as unsupported/inconsistent, never as an empty complete view. A new CLI talking to an old service may receive `Rejected` or a strict-decode connection failure. Keep the sanitized refusal and conditional guidance: if the service predates the requested view, explicitly restart a matching service. A generic rejection does not prove version incompatibility; it may be a scope/currentness refusal. Do not silently retry a weaker read or restart the service.

Old client → new service compatibility is preserved by opt-in emission. New client → old service is explicit incompatibility, not automatic feature negotiation. Keep exact endpoint/Hello/instance/epoch validation. Future view revisions require their own explicit value and review.

## 3. Exact projection objects

The following are complete field allowlists, not domain serialization. All listed fields are required inside an emitted recorded object. Nullable values are encoded as JSON null, not omitted.

`RecordedGoalStatus`:

| Field | Type and meaning |
| --- | --- |
| `view` | Literal `recorded_v1`. |
| `project` | Selected ProjectId, validated against the Goal body/index. |
| `accepted` | Literal true. |
| `criteria` | `RecordedCriteria`, below. |
| `dag` | `{node_count: usize, hard_edge_count: usize, soft_edge_count: usize}` from the validated Goal DAG. Nodes/edges are exposed by Task pages, not duplicated here. |
| `criterion_evaluation` | Literal `unavailable`: no genuine evaluator result producer is consumed. |
| `runnable_admission` | Literal `unknown`: this reader performs no admission. |

`RecordedCriteria` is a strict internally tagged union:

- `{availability: "available", recorded_count: usize, items: Vec<RecordedCriterion>}`. Items preserve the accepted definition's criterion order; every recorded criterion is present. `recorded_count == items.len()`.
- `{availability: "unavailable", recorded_count: usize, reason: "projection_budget"}`. No items key, partial list or invented zero count. This reason covers encoded-byte or unchanged strict output-profile overflow; it does not mean a false criterion.

`RecordedCriterion`: `{id: String, description: String, evaluator: CriterionEvaluator, recorded_satisfied: bool, recorded_evidence: Option<String>}`. Copy exact stored values, with the existing evaluator declaration shape (`required_tasks_verified`, `human` with `goal_pack_input`, or `unverified`). Accepted-definition checks remain responsible for valid accepted declarations. Neither the boolean nor the presence/contents of evidence verifies the evaluator. References are opaque text, never filesystem/URL reads or clickable terminal control sequences. Do not derive Goal completion from Task counts or these fields.

`RecordedProposalStatus`: `{view: "recorded_v1", project: ProjectId, accepted: false, criteria: "not_accepted", dag: "not_accepted", criterion_evaluation: "unavailable", runnable_admission: "unknown"}`. Use ONLY after the existing bounded scoped `proposal_facts` provenance/current-version checks succeed. Preserve its original objective and fields. Prose creates no criteria/DAG; do not present invented empty accepted lists. `GoalTasks` for a proposal/other nonaccepted Goal continues to refuse via the existing accepted reader.

`RecordedGoalTaskPage`: `{view: "recorded_v1", project: ProjectId, dag: DagCounts, nodes: Vec<RecordedDagNode>, criterion_evaluation: "unavailable", runnable_admission: "unknown"}`. `DagCounts` has the same three fields as status. Nodes correspond one-to-one and in the same order to the original `tasks` array; their IDs/versions must match. An empty final page has empty nodes, not a complete-global-graph claim.

`RecordedDagNode`: `{task: TaskId, version: u64, stored_state: TaskState, incoming_count: usize, hard_incoming_count: usize, structural_dependencies: "unconstrained" | "requires_prerequisite_evidence", incoming: RecordedIncoming}`.

- `structural_dependencies=unconstrained` iff hard incoming count is zero; otherwise `requires_prerequisite_evidence`. This is structural information only. A stored Completed predecessor does not prove actual complete Workflow evidence; no met/verified/ready claim is added.
- `RecordedIncoming` is `{availability: "available", items: Vec<RecordedPrerequisite>}` or `{availability: "unavailable", reason: "projection_budget"}`. Available items contain every incoming edge for this node, ordered by prerequisite UUID. No prefix truncation. Counts remain exact when items are unavailable.
- `RecordedPrerequisite`: `{task: TaskId, version: u64, stored_state: TaskState, hard: bool}`. The dependent is the containing node's Task. Lookup ONLY in the same already-validated scoped inventory; no title resolution or other-Goal lookup. Include both hard and soft edges. Every edge is represented once across a hypothetical unchanged complete traversal, at its dependent; independently collected pages still do not constitute such a snapshot.
- Original `tasks[].state` remains the existing effective display state, with original Task version. `nodes[].stored_state` is the actual stored state at that version. Neither display projection rewrites Task state/version or qualifies live capacity, native transport, permission or resume.

## 4. Finite budgets, overflow and paging

Keep these distinct inclusive limits:

| Stage | Bound and consumer |
| --- | --- |
| Existing stored Goal read | 4 MiB Goal body, before decoding. |
| Existing scoped inventory | ≤4096 Tasks, ≤32 MiB aggregate body bytes, each Task ≤1 MiB; DAG ≤4096 nodes/16384 edges. Preserve these source bounds. |
| Existing control transport | Request 1 MiB, response 4 MiB; unchanged strict profile also caps depth 32, nodes 65536, individual string 1 MiB, object/array entries 4096. Byte fit alone does not imply profile fit. |
| New recorded status/proposal response | `GOAL_RECORDED_STATUS_BYTES = 128 * 1024` for the complete serialized ControlResponse JSON, before its LF delimiter. Derived from the existing proposal objective bound: at most `6 * 16 * 1024 = 98304` encoded bytes plus fixed response/recorded-metadata overhead, rounded up to 128 KiB. Legacy no-view status behavior is unchanged. |
| New per-node incoming projection | `GOAL_RECORDED_INCOMING_BYTES = 8 * 1024` for the complete serialized RecordedIncoming object. Bounds relationship detail before owning/copying it. |
| Existing Task page, including enrichment | `GOAL_TASK_PAGE_BYTES = 64 * 1024` for the complete serialized ControlResponse JSON, before LF. Extract the existing literal to this named constant; do not change its value or widen transport. Maximum remains 1..128. |

Use capped serialization that stops at the relevant budget while writing; test the unchanged strict outgoing profile as well. Prefer borrowed narrow candidate projections into the capped writer before copying strings/relationship lists. Do not serialize/clone an entire raw Goal/Task or an over-budget projection merely to measure it. Existing bounded decoded inventory still exists; these output limits are not numerical heap or concurrency-memory claims.

The existing proposal writer and `proposal_facts` allow any nonempty-after-trim objective up to 16 KiB of UTF-8, including U+0001. serde_json may encode each input byte as six ASCII bytes; a 16 KiB U+0001 objective alone needs 98304 bytes. The other proposal fields are fixed keys, UUIDs, bounded numeric/enum/boolean values and the fixed recorded proposal object, with overhead well below the remaining 32 KiB. No proposal valid under the unchanged writer and scoped/provenance/currentness checks may be refused solely by this new default-view budget. Preserve its whole objective without truncation, new input restrictions or altered writer bounds. The 128 KiB status bound remains below the unchanged 4 MiB transport limit.

For accepted status, first attempt the whole recorded criteria set within the complete 128 KiB response and outgoing profile. If it cannot fit, substitute the explicit unavailable criteria object with exact recorded count; retain DAG counts and original facts. Serialize/check the final complete response. A final response that fails a preserved predicate refuses without any partial socket frame; the fixed minimal accepted projection and every valid proposal fit the new byte bound. Criteria paging is not added in this increment; over-budget detail remains visibly unavailable.

Keep the strict outgoing-profile check as a defensive predicate. Profile overflow is unreachable by construction through these real reader projections: status has at most 128 criteria, a Task page has at most 128 Task/node pairs, each incoming list is bounded by 8 KiB, and the typed shapes and encoded budgets bound depth, node/entry counts and strings below the unchanged profile ceilings. Cover profile rejection separately at unit level; report this reader's end-to-end overflow case as unreachable, not PASS or a missing-producer case. Do not fabricate rows, grants or relaxed readers to reach it.

For each Task node, attempt the whole incoming list within 8 KiB/profile. On overflow use the small unavailable object with exact counts. Do not silently drop relationships to fit a page. Then pack complete original TaskFacts + matching recorded nodes under the unchanged 64 KiB page bound; reduce the returned node count when adding another pair would overflow. Never downgrade relationship detail merely to squeeze more nodes into a page.

Preserve ascending UUID Task order and `after` membership check in the full selected inventory. `next` is the last actually returned Task ID iff eligible Tasks remain; if a pair cannot fit and no Task was returned, refuse. No skipped Task, empty nonfinal page or cursor pointing at an omitted row. `next=None` means this extraction reached the end, not that all criteria/relationships were available or earlier pages match now. Overflow does not change the cursor's authority or introduce continuation tokens.

Every new page reruns existing current-owner, Goal and full scoped inventory checks. Return current Goal and Task versions each time. Matching Goal versions do not freeze Task versions; separately fetched status, routing and pages are independent observations. The CLI does not auto-follow `next`, merge pages, infer a complete graph or issue hidden reads.

## 5. CLI rendering and security self-check

Existing `goal status` and `goal tasks` request `recorded_v1` by default; syntax, selectors, `--after`, `--maximum` and `--json` stay unchanged. Library callers may omit view for legacy wire behavior.

JSON retains the existing `{observation: "independent_scoped_observation", complete: false, unavailable_fields: [...], facts: ControlResponse}` wrapper. Keep existing unavailable fields; add `verified_criterion_completion` and `runnable_admission`. If detail overflows, also identify `recorded_criteria` or the affected `task_incoming_relationships` as unavailable. The typed facts carry exact counts/reasons; no aggregate `complete=true` or progress percentage is produced.

Plain status prints selected Project ID, Goal ID/version/state, criterion IDs/descriptions/declarations and explicitly labeled recorded satisfied/evidence, followed by `verified criterion completion: unavailable`, DAG counts and admission unknown. Plain tasks prints Task ID/version, stored/effective state/phase, incoming hard/soft endpoints with versions/states (or unavailable detail with counts), structural classification and explicit next cursor. Print the independent-observation/incomplete notice for both. Proposal output says `not accepted` with no criteria/DAG and preserves the objective.

Use one plain-output quoted-string escaper for every user-controlled string, including proposal objective, criterion ID/description, evidence and phase. Iterate Unicode characters and encode every `char::is_control()` character as ASCII `\uXXXX` (C0, DEL and C1 are all in the BMP); quote and backslash retain JSON-string escaping, and other characters retain their UTF-8 values. These quoted strings decode to the byte-exact input. Plain `serde_json::to_string` alone is insufficient: it escapes C0 but emits DEL/C1 raw. Emit no raw control characters originating in these strings, automatic links or evidence dereference. Formatter-owned line separators remain ordinary layout. JSON mode keeps the existing valid serde_json serialization and exact decoded stored strings; do not apply the plain escaper to DTO values or double-escape JSON. Refusals preserve existing sanitized output and nonzero exit behavior.

| Surface / source → sink | Preserved rule |
| --- | --- |
| Request IDs, selected Project/Goal/Task cursor → service reader | Actual peer UID, Hello and ingress identity precede current-owner/scoped checks; parameterized existing queries, no SQL identifier from user input. |
| Stored Goal/Task/edge/evidence text → DTO → JSON/plain terminal | Narrow allowlists, bounded capped encoding and strict profile, terminal string escaping; no command/path/URL execution. |
| Client/display → durable Store / native operations | No write or issuer path. DTOs, declaration strings, counts and cursors grant nothing. |

Read decisions under the existing contract: actual same-UID operator with current service observation and consistent selected scope may read; the same operator selecting a foreign Goal/cursor refuses. Another actual UID or absent trusted ingress refuses. Administrator labels, request metadata or claimed ownership confer no special bypass; only actual same-UID ingress follows the same scoped rule. No new role/tenant boundary or authorization policy is created.

## 6. Impact map and future controls

Source references below are baseline evidence; test names are inspected consumers, not executed results.

| Consumer | Required implementation impact |
| --- | --- |
| `runtime/control.rs:35–44,98–128,342–362` | Optional view and recorded DTO fields; strict enum/object decoding, handler forwarding. Preserve ingress at 179–221 and original action/response semantics. |
| `state/runtime/goals.rs:10–22,205–344,506–575` | Reuse owner/currentGoal/scopedTasks; allowlisted projection, same-transaction endpoint versions, explicit budget fallback and paired page packing. New bound constants cover all measuring/final checks; inspect every former 64 KiB literal in this reader. |
| `state/runtime/proposals.rs:36–39,138–204` | Preserve the writer's 16 KiB UTF-8 objective contract and existing bounded provenance/body/index/version refusal. Add opt-in not-accepted metadata under the shared 128 KiB status bound; no new proposal/acceptance writer or input restriction. |
| `domain.rs:198–237`; `goal.rs:8–82`; `runtime/goal.rs:12–16,128–159` | Existing criterion/evaluator shapes, structural bounds and immutable definition checks are read dependencies. Do not alter domain schema, validation bounds or completion rules. |
| `state/runtime/waiting.rs` | Existing display-only effective Task state consumer; use its existing result, with stored state separately projected. No quota/Unit/readiness writer changes. |
| `cli/client.rs:8–23`, `cli/service.rs:29–89`, `cli/endpoint.rs`, `cli/transport.rs:8–10,70–155`, `execution/strict_json.rs:18–32` | Existing typed transport carries optional data. Preserve shared pins, finite profiles, socket deadlines and outgoing validation. New smaller view budgets belong to the reader; do not pass unsupported sizes into transport `limits`. |
| `main.rs:179–221,306–340` | Request explicit view, verify appropriate enriched response; same routed client path; implement one control-character escaper for all user-controlled plain strings and additive JSON incomplete metadata, preserving decoded JSON strings without opening Store or fetching extra pages. |
| `runtime/tests.rs:546–638`, `runtime/installation/tests.rs:277–286` | Rust action/response constructors/patterns must handle optional fields; preserve original admission/foreign-scope/page assertions. |
| `tests/runtime_cli.rs:160–221,234–345,377–421`; `tests/goal_cli.rs` | Genuine accepted compiled ingress and idle all-table row-image controls; preserve existing six proposal/plan/lifecycle assertions and raw-frame controls. Add response/formatter compatibility expectations, not refusal-only replacement tests. |

Future controls must reach actual response consumers:

- **GF1 accepted projection:** actual registered account-free Git Projects and compiled service accept a typed plan with two criterion declarations, hard/soft dependencies and a disconnected node. Inspect all IDs/versions and exact field sets against its genuine accepted Goal/Task rows. Include U+007F and U+009B in a genuine accepted criterion description (and ID), assert their ASCII escapes and absence of raw DEL/C1 code points in the plain string output, and assert byte-exact stored values after JSON decoding. Check every reachable user-controlled plain string through its real consumer; evidence/phase lacking a genuine producer retain honest unverified end-to-end status, with the shared escaper covered at unit level. Assert plain and JSON labels, effective/stored state distinction and `complete=false` without launching an Agent.
- **GF2 proposal and unknown evidence:** compiled prose/file proposal remains inert; view marks not accepted and tasks refuses. Submit a genuine 16 KiB U+0001 objective through the existing compiled writer, then read it through the existing omitted-view legacy consumer and the recorded_v1 consumer in both plain and JSON. All four outputs must succeed, retain the whole objective byte-exact after decoding their quoted string/JSON, and preserve inert/unknown facts; no new CLI view selector is needed. Genuine accepted baseline asserts the actual stored flags/evidence and unavailable evaluation. Nondefault `satisfied=true`/evidence controls depend on an existing genuine supported publisher; if absent, mark those cases unverified with that producer named. Do not fabricate rows, synthesize a writer or change protected permits for coverage. RequiredTasksVerified declaration alone does not count as a produced result.
- **GF3 scope/currentness:** accepted positive paired with foreign selected Project/Goal/cursor, disappeared cursor where a genuine supported transition can establish it, stale instance/epoch and malformed/duplicate/unknown view frames. Preserve body/index/definition/Task-set/DAG checks in source and existing regressions; do not fabricate protected rows to manufacture coverage. Inspect unchanged complete row images, not just exit codes. If an intended currentness predicate cannot be reached through real ingress/producers, record that control as unverified rather than crediting an earlier refusal.
- **GF4 budget/paging:** accepted large plan exercises criteria fallback and more-than-one page; boundary byte lengths include UTF-8 and JSON escapes. Strict-profile rejection is a defensive unit test; overflow through the bounded real reader is unreachable by construction and has no end-to-end PASS claim. Whole incoming list exceeds 8 KiB, yet exact counts and no prefix survive. Assert full recorded status/proposal frame ≤128 KiB and full Task page frame ≤64 KiB, original cursor membership/order, no omitted row and explicit unsupported old-service behavior. Between independent reads, a legitimate supported lifecycle change proves versions are freshly observed and `complete=false` persists; do not synthesize a writer just to change Task evidence.
- **GF5 no writes/regression:** establish and recheck the compiled idle service baseline, bracket successful/refused reads with the existing all-durable-table sorted row-image helper. No scheduler/lifecycle activity may be hidden by excluding owner, ack, audit or attention tables. Run existing Goal CLI controls, Runtime reader/installation consumers, strict-frame tests, build/fmt and strict clippy; report baseline or unavailable checks honestly.
- **Causal compiled omissions:** independently omit enrichment emission (GF1 exact fields), exchange an endpoint's version (GF1 identity/version), replace the plain escaper with plain serde_json string serialization (GF1 raw U+007F/U+009B assertion), restore the 64 KiB complete recorded status/proposal cap (GF2 genuine 16 KiB U+0001 readability assertion), publish a verified-completion claim from the recorded flag (GF2 unknown assertion), omit an actual reachable scope/currentness predicate (GF3 refusal), omit criteria/incoming/page cap (GF4 bounded/fallback assertions), or write an acknowledgement from a read (GF5 complete row-image equality). Each mutant must compile and reach the intended assertion; the restored 64 KiB cap must be the cause of the GF2 recorded refusal, not an unrelated setup/transport failure. Independently preserved guards may make a particular omission ineffective, which must be reported rather than counted as a kill. Record exact patch/log hashes; setup refusal, transport failure before the tested predicate or compilation failure is not a kill. Do not weaken guards to create a positive.

No controls are executed or new-view qualification claimed in this author-only task. Root separately reports the unchanged baseline's five compiled `runtime_cli` controls PASS at c979; evidence is `c979-runtime-cli-baseline.log`, SHA256 `465530cb5cd9b958343bc8ed606d71f75ab2427dbbd63f51af617a2ee9ee4a42`, under repository-root `.rrx/development-evidence/issue43-native-producer-transport-integration/`. This confirms the existing consumer baseline only, not any proposed enrichment or native execution.

Before source work, freeze these two documents for one independent nonauthor Opus high review. Later source work updates relevant master present facts only after implementation; the full native/Goal/Issue 24 gates remain open, including continuation, parallelism, recovery, native attach, event updates and TUI.
