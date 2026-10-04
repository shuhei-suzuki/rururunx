# Issue 21: Attributable telemetry and Context Efficiency comparison

Workflow: STRICT (shared durable metrics, provider normalization and Project isolation).
Status: Requirements6; independent Req1–Req5 findings verified and refined below.
Requirements6/design/source reviews pending.
Baseline: main80452f4, schema3. Depends on merged #2 and #4; part of #17.

## Purpose and existing gap

Measure token/cost amplification and context efficiency without changing native
execution, permission, ownership or correctness decisions. Source of truth is
Issue21, Product Requirements40.7–40.10, Context Efficiency master design and the
active MVP Goal's final dogfood requirements.

The current AgentAdapter::usage returns Usage with scoped identity and nullable
metrics. GenericAdapter reports unavailable tokens/cost and prepared-input bytes.
Store::put_usage appends a Usage and audit after a Session scope/agent lookup;
Store::usage returns scoped raw rows. There is no qualified adapter measurement
provenance, replay identity, cumulative-counter normalization, bounded aggregate
API, benchmark runner or quality comparison. Existing fields are a foundation,
not acceptance of this Issue.

## Scope and exclusions

#21 owns qualified telemetry collection/normalization, bounded metrics queries,
cache-awareness hooks, actual context-event integration, KPI derivations and the
representative native enabled/baseline runner/report. Billing/account management,
new native telemetry export infrastructure, OS isolation, model/default changes,
TUI rendering and the full16 dogfood are separate. This does not waive the actual
producer or benchmark conditions needed for #21 closure below.

## Measurement and attribution

- Attribute every observation to exact Project/Goal/Task (where applicable),
  rrx Session, native-session/operation identity, agent/provider, requested/configured
  model/effort, phase, review round, reviewer slot/roster and target revision.
  Derive these ownership/request attributes from the immutable actual dispatch
  record, captured before execution. Separately derive the ACTUAL consuming model,
  effort and tokenizer from the adapter-owned native observation per measurement
  or disjoint containment partition. Requested settings never substitute for
  unreported effective settings; unknown native fallback/helper models stay unknown.
  Grouping, pricing, cached ratios and comparison use actual observed consuming units.
  Collection-time usage(phase, round) arguments and agent-authored/public JSON cannot set or
  override them; intervals spanning operations/rounds remain unattributed there.
  Overlapping Issue numbers and native IDs do not merge different Project scopes.
- Distinguish measured provider tokens, labeled packing estimates, runtime byte
  counts, runtime elapsed time and estimated/reported cost. Context/Repo Map sizes
  carry units, exact artifact/version and algorithm where estimated. Bytes are
  never relabeled as measured tokens. Runtime monotonic segment duration is not
  native API duration or Goal/Task wall time; parallel durations are not summed
  into elapsed completion time. Restart segments retain their provenance.
- Supported native metrics require an adapter-owned observation of the actual
  response and documented counter semantics. Record provider/runtime version and
  whether a measurement covers one request, one turn, a native session cumulative
  counter or an unknown interval. Public/generic Usage, model JSON and inferred
  recovery values cannot acquire measured-provider provenance merely by carrying
  numeric fields. Trusted linked code remains outside this application boundary.
- Preserve bounded raw numeric measurements as explicitly unqualified observations
  when semantics or interval attribution are unknown. Exclude them from additive
  Task/Goal/benchmark token/cost totals. Do not treat resumed native CLI aggregate
  modelUsage/total_cost/duration fields as this turn's incremental measurements.
- Normalize a cumulative counter only across compatible, attributable boundaries:
  same native counter identity/epoch, counter definition, pricing/unit and owned
  operation interval. Require a valid preceding baseline, monotonic values and
  exact association of the increment. Counter reset, missing baseline, overlapping
  intervals or an unowned/ambiguous increment stays unknown; no subtract-and-guess,
  zero clamp or automatic session replacement that changes native behavior.
- An observation/replayed response uses the owned operation plus native response/
  turn ID or a durable owned event ordinal when native lacks one; never collection
  time, query sequence or payload equality. Same identity and identical payload is
  a no-op, conflicting payload rejects, and different identities with equal counts
  remain distinct genuine observations, without replay changes to totals or audit.
  Assign an ordinal once at the actual owned native event/stream position, never
  per collection channel: status and final-result views of one event share identity.
  A cumulative snapshot and its qualified delta cannot both be counted.
  Collection before/after restart or repeated status queries cannot double count.
- Missing/unsupported/invalid values carry per-field reasons. Zero means an actual
  qualified measurement of zero. Partial totals include coverage and exclusions;
  unavailable fields never become zero or a complete savings claim. Checked
  arithmetic, finite nonnegative costs and explicit overflow errors prevent wrap,
  NaN/infinity or precision loss from becoming successful totals.
- Token categories retain native semantics: whether input includes cached reads,
  cache writes or only uncached tokens. Add disjoint categories only when documented.
  Report cached ratio only with a compatible measured denominator; otherwise it
  is unavailable. Cache use, cache-hit performance and logical context reduction
  are different measurements.
  Every token KPI/comparison names its input definition. A logical-context reduction
  needs a native logical-input-inclusive total or a verified disjoint sum of uncached,
  cache-read and cache-write tokens. Exclusive input with unavailable cache categories
  is only an "uncached input change (includes cache effects)"; logical reduction is
  unavailable. Stable-prefix/cache effects never become logical-context savings.

## Complete operation accounting and cost

- Coverage denominator is the complete durable inventory of actual provider-consuming
  operations: native requests/turns, all retries/failed/timeout/cancelled attempts,
  explicit cache-write calls and any summary/condensation/selection/pack/expansion
  model calls, including runtime-owned calls outside ordinary Task Sessions.
  Inventory links exact owner and feature event before dispatch. Missing collection
  or crash-before-collection leaves an uncovered operation, never removes its cost.
  If dispatch inventory itself cannot be proven complete, coverage is unknown.
  Native execution retains its existing policy; optional telemetry failure cannot
  manufacture complete inventory or redispatch. Complete totals require verified
  complete inventory and qualified coverage for the reported metric/category.
- Qualify each inventoried operation per field: measured, unqualified, unsupported,
  missing or lost with bounded reason. Query partial known totals as partial.
  Full-workload savings require complete compatible coverage in both lanes.
  A predeclared matched subset may have its own descriptive comparison only,
  explicitly excluding unknown work and never claiming whole-workload savings.
  Asymmetric coverage yields not-comparable, rather than a lower-cost winner.
- Include every attributable efficiency-layer provider call and cache-write premium
  in the enabled lane, including its failed/retried attempts. Only local CPU/byte
  overhead is separately identified; wall completion time includes all work.
  Record the beneficiary Task/round/Goal links of shared out-of-scope feature work.
  Direct scoped totals are descriptive together with linked, unallocated overhead;
  they cannot establish scoped savings by moving work to a Goal/runtime owner.
  Only complete lane-level totals, including all shared provider overhead, establish
  an end-to-end reduction. Do not sum shared cost once per beneficiary or silently
  allocate it to a scope without a separately reviewed predeclared partition rule.
- Observation containment records whether a turn/session total includes child
  requests, subagents, helper models, per-model breakdowns or tool usage. Sum only
  one verified non-overlapping partition. Do not add an outer total and its parts;
  unknown containment prevents qualified additive aggregation.
- Token units and categories are specific to the consuming provider/model/tokenizer.
  Group incompatible models; never use an unlabeled mixed-model sum for ratios,
  amplification or savings. Reasoning/thinking/tool tokens obey the same documented
  overlap/disjointness rule as cache/input categories, never count them twice.
- Cost is explicitly native-reported estimate or runtime-computed estimate, never
  billed truth. A computed estimate requires the actual consuming model, operator-
  supplied versioned price schedule/content digest, currency/effective period and
  rates matching every eligible disjoint category. Unknown rate/model/category
  means unavailable, never a default price. Project/model data cannot activate prices.
  Runtime operator pricing changes create a new immutable derivation, never rewrite
  prior estimates. Provider-reported and computed estimates are shown separately.
  Sum only compatible currencies/bases; group other values without conversion.
  Both benchmark lanes pin one schedule for computed-cost comparison; unreported
  native pricing/basis prevents a comparable native-cost savings claim.

## Durable storage and query boundary

- Append scoped observations and their own audit transactionally under current
  writer epoch and actual Session/operation correlation. Keep separate immutable
  measurement/normalization provenance rather than rewriting native-fenced Goal,
  Task, Session, Workflow, context or lock rows to publish derived totals.
  Telemetry gives no dispatch, Passed, completion, cleanup or process-death proof.
- Coordinate durable formats and all already-open writer fencing with #19. Old
  Usage remains legacy/unqualified; migration never infers trusted producer or
  interval attribution from numbers alone. A scope mismatch, conflicting replay
  or storage error cannot partially append observations, totals or audit.
- Missing metrics do not stop supported native execution. Collection/storage
  failures are observable and keep incomplete coverage, while transport results,
  owned cleanup and mandatory workflow errors retain their actual meanings.
  Metrics retries never redispatch a native request or reacquire admission.
- Provide bounded library/API queries by Task, Goal, Project, agent, phase and
  review round, and explicit runtime-wide grouping that retains Project identity.
  Scope-bound query capabilities restrict Project consumers to their actual owner.
  Explicit runtime-wide numeric grouping requires the trusted runtime-operator
  ingress, unreachable from agent/Project IPC; it exposes no foreign artifact or
  session references. This is the approved23 application boundary: trusted linked
  code/direct same-UID machine actions are outside it, not an OS authentication claim.
  Current/historical queries are read-only, use one coherent durable snapshot,
  and do not silently truncate or mix scopes. Removed/blocked owners may expose
  their own history without granting new execution. Design names finite row,
  payload, nesting, aggregate and pagination limits before source implementation.
- Persist only allowlisted numeric/timing/cache semantics and bounded references.
  No credential values, environment contents, headers, prompts, native transcripts
  or cache contents in telemetry/audit. Existing put_usage/raw usage APIs must be
  retired or routed through the same epoch-fenced bounded allowlisted surface,
  with generic writes permanently unqualified. No API/report/audit may expose raw
  legacy cache_metadata or free-text reasons; decode bounded safe projections.
  Arbitrary raw cache_metadata never reaches UI/reporting. Invalid or unknown fields retain a bounded reason rather
  than unbounded provider payloads or sensitive error strings.
- API results make partial/unknown values and units visible. TUI/CLI consumers
  (#10/#24) use this contract; a new command or dashboard is not implicitly added
  by this issue. Goal completion time and progress originate in actual #23/#24
  lifecycle observations, never from token counts. Until those producers exist,
  Goal time is explicitly unavailable with a reason and its acceptance stays open.

## Cache awareness and context events

- Keep reusable instructions/tool definitions/source slices deterministically
  ordered where the runtime controls prompt construction. Preserve exact Project,
  revision, rules, artifact and phase currency. Stable prefixes never reuse stale
  authority, copy peer findings or share another Project's context/cache content.
- Discover cache capability through the actual registered adapter. Use only native
  supported mechanisms allowed by existing policy; unsupported explicit caching
  remains visibly unsupported. This work never changes native default model,
  effort, credentials, hooks, permissions or safety rules to manufacture a hit.
- Log actual Context Pack/Repo Map publication, condensation and expansion with
  scoped artifact/version, size/unit and producer event identity. Count an actual
  completed event once; requests, failed publication and replay are distinguished.
  Context Pack size/version requires an actual published #19 pack; prepared-input
  bytes come from owned prepared-input provenance and have a different label.
  Generic recovery JSON sizes/versions remain unqualified. Integrate actual
  #18/#19/#20 producers before claiming their event acceptance.
- Cache hit/miss/unavailability cannot change mandatory evidence, admission,
  review independence, completion, cleanup or required context behavior. An expired
  or absent cache falls back to an equivalent authorized input or explicit native
  unsupported/error result; never substitute foreign/stale material.

## Enabled/baseline comparison

- Provide an explicit test/benchmark mode that compares the same representative
  workload with Context Efficiency enabled and optional reduction disabled.
  Preserve mandatory rules/evidence, Project isolation, fresh-input/lock checks,
  reviewer independence, review floors, permission policy and workflow obligations
  in both modes. Freeze workload/revisions/feature sets, metrics, comparison rule,
  repetitions and quality oracle before the first run. Baseline is the enabled
  configuration minus declared optional reduction; no inflated straw-man context.
  Credit only exercised features and integrate their real producers. At minimum the
  native comparison exercises2 independent reviewers and a round>1 delta re-review.
  Baseline never forwards executor chat or peer findings to reviewers.
  Resource/budget exhaustion is an explicit failure, not silent mandatory truncation.
  Register EVERY immutable plan and started run durably before its first dispatch;
  acceptance/reporting includes aborted and superseded plans with all results.
  A successor cites its predecessor and an actual runtime/reduction behavior-fixing
  source revision or a feature predeclared disabled with narrower claims. Verify
  the prior regressing inputs/oracle items against the fix; an apparatus/fixture edit
  or revision hash alone is not proof. It cannot shop away a
  regression by reducing oracle sensitivity, representativeness, repetitions or
  roster without separately reviewed justification. Previous observed regressions
  remain disclosed beside later fixes/results. An unchanged-source/configuration
  successor is allowed only for a registered measurement-invalid or execution-
  environment reason, never unfavorable quality/savings. Pool ALL valid repetitions
  of the same source/frozen configuration across registered plans into one claim
  denominator and dispersion under a stopping rule declared before the first run.
  The latest/most favorable plan never drives the claim alone. Changed source or
  configuration has its own claim scope and cannot pool with or replace old results.
  Registry/history is append-only under the shared writer epoch and survives fixture
  cleanup; ad-hoc/unregistered runs are inadmissible acceptance evidence. No OS
  integrity claim extends beyond the stated application trust boundary.
  For EVERY dispatch in both lanes, retain actual producer-backed delivery evidence
  mapping the complete mandatory rules/evidence set computed from that dispatch's
  current authorized inputs to delivered content identities/digests and scope.
  A configuration list, prepared-but-unsent payload or agent's assertion is not
  delivery proof. Missing, stale or truncated mandatory delivery invalidates the
  pair and prevents savings/quality equivalence, irrespective of token coverage.
  Derive this set from the reduction-independent Project/workflow mandatory rule
  and evidence policy for the exact phase/scope, computed under the same policy in
  both lanes. The selection/progressive loader under evaluation cannot reclassify
  its own missing item as optional. Delivery is in that dispatch's actual sent
  input, or separately proven runtime-controlled retained content with no intervening
  condensation, session replacement or unknown compaction. Otherwise re-deliver or
  mark unverified. Native self-loaded files are a separate native-loaded/unverified
  category under preserved defaults; they cannot substitute for runtime-mandatory
  delivered content or acquire reduction credit. No proof of native internal
  attention/retention after delivery is claimed. Lost context fails required checks.
  If a runtime-owned condensation, replacement, selection or loader intervenes,
  skipping mandatory re-delivery is actual delivery loss and an absolute safety
  failure, never the measurement-invalid "unverified" escape. Only retention tied
  to unowned native behavior after a recorded earlier delivery to that same session
  can remain unverified measurement evidence; it cannot certify current delivery.
- Record immutable workload/revision/acceptance and verification plan, selected
  reduction features, provider/native versions, actual effective model/effort,
  policy/capability configuration, telemetry qualification and cache conditions.
  Do not change these settings while attributing a difference to context reduction.
- Run lanes in authorized isolated fixtures/worktrees/registries with distinct
  native sessions and exact scoped identity. Never modify unrelated user repos,
  reuse one lane's generated outputs as the other's starting state or leak findings
  between independent reviewers. Inventory known persistent native memory/history
  channels without reading/copying secret contents. Record possible prior exposure,
  cache warmth and actual model/effort per observation where available; unknown or
  mismatched model/effort mix, including native automatic fallback, prevents causal
  savings. Give each lane AND repetition unique owned fixture paths, Project/Task
  identities and native sessions, avoiding controllable path-keyed memory collision.
  Recorded known-channel cross-lane/prior-repetition exposure, or exposure that
  cannot be excluded on an inventoried reachable channel, makes the pair confounded:
  no causal savings or quality-equivalence certification. Retain all such attempts
  in the report with exposed/unverified and qualified cohorts clearly distinguished;
  never drop them to improve an overall result. Uncontrolled cross-lane/reviewer
  state is a confound, never blind independence. Preserve native auth/hooks/defaults,
  never redirect/copy credentials or delete user native history. Clean only owned
  runtime fixtures and disclose native-created history left to its provider; no
  history-free OS isolation claim. Run predeclared repeated paired/counterbalanced
  lanes, retain EVERY attempted repetition including unfavorable/failing results,
  and report dispersion/cache/environment. Differences within observed variation
  remain inconclusive/indicative, not established causal savings.
  Freeze invalidation/exclusion rules with distinct outcome axes for both lanes:
  measurement-invalid (coverage gap, unknown model/effort, native fallback/compaction
  or unowned attach interval) makes comparison unavailable, never a task-success,
  quality or safety score; execution failure (crash, timeout or budget exhaustion)
  is that lane's failure under the frozen acceptance tests. Absolute safety invariants
  (mandatory delivery loss, gate/freshness bypass or actual safety failure) in EITHER
  lane prevent acceptance. Quality oracle detection is scored per item and paired
  repetition under the frozen comparison rule: enabled-only sensitive-item misses
  or enabled detection worse than baseline beyond predeclared tolerance are quality
  regressions and prevent acceptance. Equal/baseline-only misses are disclosed
  detection limits/rates, not Context Efficiency regression or enabled safety credit.
  Neither oracle counts alone nor equal misses certify equivalent quality. Measurement
  failure is not proof of execution failure or safety loss. Known native provider-
  load fallback can be an environmental confound, not attributed to reduction
  without evidence. Report all axes and invalidations by lane/run order. Baseline
  failure never becomes an enabled safety advantage. None are neutral omissions.
  ANY required repetition failing coverage, delivery, quality or
  comparison conditions prevents a whole-run causal savings/equivalence claim.
  Qualified survivor cohorts are descriptive only; retaining failures in an appendix
  does not permit savings on a favorable subset or lost counterbalance. Preserve
  original aggregate failure/unknown denominators and the frozen repetition plan.
- Compare actual Goal/Task completion time, throughput, waiting/blocked time,
  measured input/cached/output tokens, qualified estimated/reported cost, review
  time and actual Human interruptions/attention where observable. Missing Human
  or native timing/usage metrics remain missing, never invented zeroes. Keep
  local CPU/byte overhead separate from provider measurements; ALL provider calls
  incurred by reduction stay charged to the enabled lane as required above.
- Evaluate task success and review quality using the same independent acceptance
  tests and verified predeclared seeded findings/oracle; disclose false positives,
  missed findings and safety failures. Hidden oracle answers/expected findings are
  excluded from runtime-provided lane/reviewer inputs and caches; disclosed ordinary
  acceptance requirements remain available. Match with deterministic predeclared
  criteria or independent lane-blind adjudication. Uncontrolled native access/exposure
  is a confound, not an OS secrecy guarantee. Without independent quality oracle,
  quality is unverified and benchmark acceptance remains unmet. Equal findings counts, model
  assertions, exit zero or cheaper tokens alone never certify equivalent quality.
  Failed/timeout/unsupported lanes stay in the report rather than being discarded.
  Every credited reduction feature needs predeclared verified oracle items sensitive
  to the content it can omit: an out-of-slice caller/dependency or triggered rule,
  a decision/failure predating condensation, or an unresolved prior-round finding
  and impact outside a round>1 delta. Record per-feature oracle coverage/detection;
  exercised features without sensitive oracle items have unverified quality and
  receive no savings credit. Keep hidden answers outside lane-readable fixture
  trees/commit history and runtime inputs/caches, without claiming an OS sandbox.
  Report fix/round counts and early termination beside costs. Per-round/per-feature
  comparisons need identical immutable revision/bundle inputs; divergent trajectories
  support only a qualified end-to-end comparison, never same-input round savings.
  Oracle adjudication and separate amplification-reference calls are benchmark
  apparatus, reported separately for both lanes rather than reduction work.
  End-to-end savings AND quality-equivalence certification require sensitive oracle
  coverage for EVERY actually exercised reduction feature. Any uncovered feature
  makes the full result quality-unverified/descriptive, even if the overall oracle
  for other features passes. An uncovered feature must be predeclared disabled to
  avoid that result. Per-feature credit needs predeclared same-input ablation runs.
  Both lanes begin from the same frozen starting artifacts, without pre-generated
  reduction outputs; any enabled setup generation is charged in provider cost and
  completion time. Equality of cross-lane inputs means equality after canonical
  substitution ONLY of predeclared lane identity fields; report their actual byte/
  token lengths. No source/content/rule/finding substitution hides lane differences.
  Apparatus classification is fixed before dispatch, and its outputs never feed
  execution/gates. Disclose providers excluded from the actual default MVP roster
  and restrict comparisons to measured providers. Human attach turns and unowned
  native-internal calls/compaction leave interval coverage unknown unless the owned
  observation documents a complete disjoint encompassing partition.
- The runner/reporting contract may be tested with controlled fixtures; final
  native representative comparison is required before benchmark acceptance.
  Issue16 additionally owns full multi-Project/Goal/four-Task dogfood evidence.
  #21 supplies the computable KPI contract below. Do not close21 or17 by treating
  synthetic numbers as native evidence,
  or declare the MVP complete because a benchmark harness builds.

## KPI and closure contract

Input Tokens per Task/Round and Estimated Cost per Task sum qualified non-overlapping
inventoried operations in that exact scope, grouped by model/unit/pricing basis and
with coverage. Report linked shared overhead separately as described above. These direct scoped
metrics remain descriptive; complete lane totals establish end-to-end savings.
Cached Ratio is eligible cached reads divided by compatible total
input that includes those reads; absent/zero/incompatible denominator is unavailable.
Reviewer amplification per round compares the sum of actual independent reviewer
input to a predeclared measured single-reviewer reference on the same immutable
bundle/model; report reviewer slots/count, shared/specific input and cached/uncached
separately. A heterogeneous roster requires compatible per-model references, never
a cross-model token sum. A replaced reviewer gets a new immutable operation and
slot/roster version; attempts stay accounted for rather than rewriting the old slot.
An absent reference means no amplification ratio. Reduction is paired
(baseline-enabled)/baseline for a compatible complete metric; zero/unknown baseline
or any failed coverage/quality/comparison condition prevents a savings claim.
Human interruptions/attention use actual scoped interaction events and interval
semantics, deduplicated; missing producer is unavailable, never zero. #16 exercises
these contracts with its full native dogfood, not the first definition of metrics.

| Issue21 acceptance/scope | Required evidence and closure owner |
| --- | --- |
| Usage exposed by native | #21 integrates every merged5/6/7 adapter's actual owned response or explicit per-field unsupported reason; real native evidence required |
| Missing usage preserves execution | Actual missing/invalid/failed collection consumer, explicit uncovered inventory and unchanged execution authority |
| Cache counters/use | Documented native category semantics plus actual observation or unsupported reason; no guessed ratio |
| Per-Task API query | Bounded scoped public Rust/library reporting API, complete inventory/coverage and two-Project negative access controls; TUI/CLI10/24 consumes it later |
| Baseline comparison mode | #21 native runner executes predeclared equivalent enabled/baseline feature configurations; no safety/independence weakening |
| Token/cost/time/success/review-quality report | Qualified metrics or explicit unavailable, all attempts, dispersion/confounds, per-dispatch mandatory delivery, feature-sensitive independent frozen quality oracle and comparability decisions; the minimum native token comparison below is required |
| Cache-independent correctness | Actual warm/cold/unavailable-cache freshness/admission/gate controls and compiled bypass mutant |
| Goal aggregate token/cost/time query | #21 scoped API with actual23/24 lifecycle timing producer; unavailable is honest interim output but Goal-time acceptance stays open until producer integrates |
| Pack/Repo Map/condensation/expansion telemetry | Actual18/19/20 event/artifact provenance, idempotent event counts and full provider overhead accounting; #21 stays open while required producers absent |
| Stable prefix / provider cache awareness | Byte-identical equal authorized input, current Project/revision/rule invalidation, actual registered capability and supported native channels |
| Product40.10 KPIs | Computable contract above and representative native comparisons; #16 additionally supplies full Goal/multi-Project/four-Task evidence |

No scope checkbox is silently transferred away. #21 stays open until its complete
contract, required integrated producers and representative native benchmark pass.
The native token-comparison acceptance requires qualified comparable input/output
token measurements with complete operation/partition coverage for EVERY provider
used in the representative lanes. An all-unavailable report satisfies honest
missing-telemetry behavior only; it cannot close the benchmark/token KPI acceptance.
Missing native exposure keeps that acceptance open with its actual named dependency
or separately reviewed owned-channel policy blocker. Cached counters and monetary
cost retain the source Issue's where-available condition: show qualified native or
operator-configured estimates when available, otherwise explicit unavailable and
no cache/cost savings claim. Do not invent measurements or change native defaults.
The minimum token comparison states the native input definition it measures; it
does not certify logical reduction without inclusive coverage above. Acceptance
requires an executed representative comparison with actual quality/safety evidence
and honest comparability decisions, not a positive saving or causal conclusion.
Unknown effective effort/model can produce an explicitly non-comparable native
result rather than fabricated certainty. Missing required token measurements,
unverified feature-sensitive oracle coverage or observed safety/quality regression
still leave their acceptance conditions unmet.
An unknown consuming model/tokenizer leaves token units unqualified and their
minimum token-comparison acceptance OPEN; it cannot use this non-comparable-outcome
exception. Unknown effective effort alone prevents a causal claim while retaining
otherwise qualified model/unit measurements. Show paired input, output/reasoning
and cost-where-available changes together; input savings alone is never net savings.
Delivery metadata stores bounded identities/digests, not forbidden prompt contents.
For non-omitting ordering/cache features, coverage checks ordering/content equivalence
and the whole-workload sensitive oracle rather than inventing a content omission.
TUI rendering and final16 whole-MVP dogfood are separate; neither is falsely claimed
by API implementation. Use one linear schema history coordinated with19, reject
unknown/future formats without downgrade, and test every real predecessor.
Collect only from output/protocol channels the adapter already owns. Do not enable
new native exporters/OTEL/network export or change flags/env/config solely to obtain
metrics without explicit separately reviewed policy; otherwise mark unsupported.
Native benchmark invocation requires existing user/operator authorization (the
current MVP Goal explicitly authorizes native dogfood), safe nonsecret fixture data
and finite execution/time limits. Honor requested spend limits only through a
qualified enforceable contract; unknown cumulative billing cannot prove a hard cap.
No new per-run confirmation or invented budget is required when already authorized.

## Verification and delivery

Review requirements, then design, then immutable source/security scope. Inventory
actual provider5/6/7 usage producers, Store usage/audit/query, Workflow phase/round
correlation, context publishers and lifecycle timing consumers. Preserve native
defaults, exact input/ownership fences and independence.

Use two isolated Projects with overlapping Issue/phase/native counter labels.
Cover qualified turn measurements, unknown/cumulative snapshots, compatible delta,
reset/overlap/missing baseline, mixed input/cache definitions, partial/null/zero,
checked overflow, price changes, replay/conflicting duplicate, restart, observation
transaction rollback/second-writer conflicts, pagination/bounds and no Goal/Task
version changes. Check actual available native fields and explicit missing values,
not adapter fixtures alone. Verify secret-bearing arbitrary metadata is rejected.

Compiled causal mutants must reach real collection/aggregation/report consumers:
count cumulative snapshots as turn deltas, double count replay, zero-fill missing,
merge cross-Project identity, sum incompatible cache categories, lose failed lanes,
make cache hit bypass freshness/required evidence, and publish metric observations
by changing native-fenced parent currency. Passing controls and restored source
are mandatory; compilation/setup failure or unrelated safety refusal is no kill.
Also mutate dropped-operation coverage, omitted condensation/provider overhead,
asymmetric exclusions, collection-time phase/round, outer-plus-child totals,
time-derived replay identity, collapsed payload-equal genuine events, legacy
secret/raw append exposure and dropped unfavorable repetitions at real report
consumers. Stable-prefix determinism/currency and warm/cold/unavailable cache
controls retain equivalent authority. Enabled/baseline native runs require
quality/safety evidence beside savings.
Also kill compiled mutants that replace an observed fallback model with the requested
dispatch model at pricing/comparability consumers; move a Task's provider condensation/
pack work to a Goal/runtime owner then allow scoped savings; omit a mandatory delivered
item but still report savings; or credit a feature without its sensitive quality
oracle/with known-channel exposure. All-unavailable native counters must leave the
token benchmark acceptance open. These are real reporting/closure consumer controls,
not model assertions that a deliberately weakened fixture remains safe.
Further reporting mutants must reject survivor-only savings after an enabled-lane
invalid pair; end-to-end credit when any exercised feature lacks a sensitive oracle;
exclusive uncached change relabeled logical reduction; and a loader reclassifying
a policy-mandatory item as optional or counting stale prior delivery after loss.
Also reject replacing/dropping an unfavorable predecessor plan at the actual closure
report consumer, and scoring a baseline coverage gap as a safety/task failure or
enabled quality advantage. Restored controls must distinguish measured invalidity,
actual execution failure and independently verified safety/quality regression.
Reject real consumer mutants that condense, skip mandatory re-delivery and hide the
loss as unverified measurement; mark equal/baseline-only oracle misses as a reduction
regression; or select a favorable unchanged-source plan instead of its pooled history.

Run relevant state/provider/context/workflow regressions, fmt/clippy/build, exact
Linux/macOS CI and independent source reviews. Update master design/README only to
actual behavior and retain downstream producer/TUI/dogfood obligations explicitly.
