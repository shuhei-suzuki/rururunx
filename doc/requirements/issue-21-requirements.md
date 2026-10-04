# Issue 21: Attributable telemetry and Context Efficiency comparison

Workflow: STRICT (shared durable metrics, provider normalization and Project isolation).
Status: requirements draft; requirements/design/source reviews pending.
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

## Measurement and attribution

- Attribute every observation to exact Project/Goal/Task (where applicable),
  rrx Session, native-session/operation identity, agent/provider, phase, review
  round and bounded observation identity. Task/phase/round membership comes from
  actual owned runtime/adapter inputs, not labels supplied by model output.
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
- An observation/replayed provider response is idempotent by its exact producer
  identity and payload. Conflicting duplicates reject, without changed totals or
  audit. A cumulative snapshot and its qualified delta cannot both be counted.
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
  Current/historical queries are read-only, use one coherent durable snapshot,
  and do not silently truncate or mix scopes. Removed/blocked owners may expose
  their own history without granting new execution. Design names finite row,
  payload, nesting, aggregate and pagination limits before source implementation.
- Persist only allowlisted numeric/timing/cache semantics and bounded references.
  No credential values, environment contents, headers, prompts, native transcripts
  or cache contents in telemetry/audit. Do not forward arbitrary raw cache_metadata
  JSON to UI/reporting. Invalid or unknown fields retain a bounded reason rather
  than unbounded provider payloads or sensitive error strings.
- API results make partial/unknown values and units visible. TUI/CLI consumers
  (#10/#24) use this contract; a new command or dashboard is not implicitly added
  by this issue. Goal completion time and progress originate in actual #23/#24
  lifecycle observations, never from token counts.

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
  Integrate actual #18/#19/#20 producers before claiming their event acceptance.
- Cache hit/miss/unavailability cannot change mandatory evidence, admission,
  review independence, completion, cleanup or required context behavior. An expired
  or absent cache falls back to an equivalent authorized input or explicit native
  unsupported/error result; never substitute foreign/stale material.

## Enabled/baseline comparison

- Provide an explicit test/benchmark mode that compares the same representative
  workload with Context Efficiency enabled and optional reduction disabled.
  Preserve mandatory rules/evidence, Project isolation, fresh-input/lock checks,
  reviewer independence, review floors, permission policy and workflow obligations
  in both modes. Baseline never forwards executor chat or peer findings to reviewers.
  Resource/budget exhaustion is an explicit failure, not silent mandatory truncation.
- Record immutable workload/revision/acceptance and verification plan, selected
  reduction features, provider/native versions, actual effective model/effort,
  policy/capability configuration, telemetry qualification and cache conditions.
  Do not change these settings while attributing a difference to context reduction.
- Run lanes in authorized isolated fixtures/worktrees/registries with distinct
  native sessions and exact scoped identity. Never modify unrelated user repos,
  reuse one lane's generated outputs as the other's starting state or leak findings
  between independent reviewers. Repetitions/order/cache warmth and environmental
  differences are reported; unbalanced runs cannot imply causal savings.
- Compare actual Goal/Task completion time, throughput, waiting/blocked time,
  measured input/cached/output tokens, qualified estimated/reported cost, review
  time and actual Human interruptions/attention where observable. Missing Human
  or native timing/usage metrics remain missing, never invented zeroes. Keep
  runtime overhead and context-selection bytes separate from provider measurements.
- Evaluate task success and review quality using the same independent acceptance
  tests and verified seeded findings/oracle where appropriate; disclose false
  positives/missed findings and safety failures. Equal findings counts, model
  assertions, exit zero or cheaper tokens alone never certify equivalent quality.
  Failed/timeout/unsupported lanes stay in the report rather than being discarded.
- The runner/reporting contract may be tested with controlled fixtures; final
  native representative comparison is required before benchmark acceptance.
  Issue16 additionally owns full multi-Project/Goal/four-Task dogfood and all listed
  KPIs. Do not close21 or17 by treating synthetic numbers as that native evidence,
  or declare the MVP complete because a benchmark harness builds.

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
Enabled/baseline native runs require quality/safety evidence beside savings.

Run relevant state/provider/context/workflow regressions, fmt/clippy/build, exact
Linux/macOS CI and independent source reviews. Update master design/README only to
actual behavior and retain downstream producer/TUI/dogfood obligations explicitly.
