# Issue 21: Qualified telemetry and native comparison

Status: Design1, independent design/source review pending.
Requirements8 at e001483 received two independent approvals, no findings, and
verified owned cleanup. This document proposes APIs/storage; none is implemented.
Baseline main80452f4/schema3; required native/context producers remain pending.

## Existing interfaces and integration boundary

AgentAdapter::usage takes collector-supplied phase/round. Store::put_usage looks up
Session before its write transaction and appends arbitrary Usage/cache_metadata;
Store::usage reads all raw rows. Generic supplies bytes without native tokens.
Grok exposes selected native-entry counters without a qualified counter contract.
Claude f9b671f distinguishes current invocation tokens from resumed cost/model
gauges. Codex d752d77 preserves turn identity, resume baseline, reset and late-turn
validity. Reuse those distinctions; do not difference already incremental fields.

#19's proposed managed operation is the Task dispatch identity; #58 owns Task-free
Consult lifetime/input identity; #9/#10 own decision-member identity. No table row,
public Session or configuration label can replace an actual registered producer.
Their private ports/profile conformance and #14 recovery remain production gates.
Design/source changes must inventory actual latest implementations before wiring.
Coordinate one migration with their final composed epoch, not a separate schema6.

## Producer possession and complete operation inventory

Add crate-private telemetry interfaces in telemetry/{identity,normalize,store,query,
benchmark}. Public reports are read-only numeric projections. Trusted runtime
composition installs adapter implementations and immutable counter contracts;
model/config JSON cannot register one. Controlled transport implementations have
the same production caller contract but their evidence is labelled synthetic.

An actual supervisor acquires a nonserializable measurement-only handle before
every runtime-owned provider operation. Identity contains Project, optional Goal/
Task, actual native operation and input/attempt identity, runtime instance, adapter
implementation/profile, source revision, operation purpose and beneficiary links.
Task dispatches derive generation/index/phase/context/round/slot from the actual
owning Workflow/ReviewSet transaction; callers cannot supply attribution labels.
Task-free and helper operations have explicit purposes rather than fake Tasks.
The handle cannot grant native input, cleanup, release, review or Goal authority.

Register the inventory row and immutable descriptor before native effects, linked
atomically to the owning operation/admission marker. The real adapter receives the
handle; it publishes bounded observations from owned responses. Startup failure,
cancel, retry, missing collection, publication conflict and Drop remain inventoried
with unknown fields. A retry is a distinct operation; unchanged input is no replay
exemption. Unknown durable ownership is retained under the producer's recovery
contract; telemetry never releases it. A failed inventory transaction prevents that
new dispatch, not historical denial/cleanup or factual late observations.

All reduction helpers, selection/condensation/expansion generation, initial setup
and work outside Task phases use this same inventory. Owning helpers record exact
beneficiary identities without assigning shared work wholly to one lucky Task.
Native internal calls need either a verified complete encompassing measurement or
a complete disjoint partition. Unknown unobserved internal/delegated work makes
coverage incomplete; do not invent inner call IDs or assume one CLI result means
one model call. Separate apparatus calls use predeclared apparatus identity and
cannot feed lane execution or acceptance gates.

## Field normalization, model units and replay

An immutable CounterContract identifies implementation/version, operation/turn
scope, observed consuming model identity, tokenizer unit, incremental/cumulative
semantics, category partition and containing/contained relationships. Qualification
requires primary contract evidence plus real native conformance. Requested model
and effort are separate; unknown observed model/tokenizer prevents comparable
token qualification, while unknown effort alone supports an honest qualified-unit
descriptive report with its causal confound disclosed.

Persist recognized registered model/contract IDs only. Unknown native strings become
bounded unavailable reasons, not raw metadata. Observations carry field-level
Measured/Unavailable/Invalid states and source identity. Integer token/byte counts
must fit nonnegative SQLite i64; aggregation uses checked u128. No missing zero,
unchecked cast or floating token count. Overflow invalidates the affected metric.
Reset/late/baseline mismatches preserve the field's invalidity, never clamp negative
differences. Normalize cumulative fields only against the contract's exact owned
baseline; Claude resumed raw cost/model gauges remain unqualified, and Codex's
already normalized turn counters are not differenced again.

Logical input is inclusive of the contract-defined cache-read/write categories;
exclusive uncached input is separately named. Sum only proven disjoint categories.
Cache reads are a subset for the ratio, not extra logical input or reduction credit.
Partition graphs are finite acyclic identity graphs; an encompassing total and its
contained children cannot both contribute to an aggregate. Mixed models/tokenizers
remain separate groups; no arithmetic across incompatible units.

Replay key is (operation ID, producer kind, ordinal). Identical canonical content is
a no-op; conflicting content rejects without replacing prior evidence. Wall time,
equal payload and Session ID alone do not deduplicate distinct operations. Ordinals
and versions are positive checked i64, immutable once accepted. Late observations
may improve explicitly missing measurement coverage through a new versioned event,
but cannot rewrite prior failure, benchmark outcome or native ownership facts.

Native monetary estimates remain separate from operator-derived estimates. Native
finite values retain their declared precision and scope; no billing claim. Computed
cost uses checked fixed-point/rational arithmetic under an immutable operator-
activated PriceSchedule ID/content digest: model, currency, effective interval,
category rates, thresholds and rounding. Unknown model/categories/rate partition
make computed cost unavailable. A schedule change creates a new ID; queries never
reinterpret old results with today's prices. Keep currency groups separate and
report estimate basis/rounding. Price activation is trusted application ingress,
not agent JSON; no automatic price lookup or current-price correctness claim.

## Durable rows, epoch and source artifacts

Conceptual private tables: telemetry_operations, telemetry_observations,
telemetry_artifact_events, telemetry_price_schedules, benchmark_plans,
benchmark_attempts and benchmark_outcomes. Each has exact scope columns, version,
canonical descriptor digest and owning producer/foreign-key identity. Private
observations append; explicitly versioned coverage is read coherently. Source
Session/Goal/Task/Workflow bodies and versions are never bumped for telemetry.

Artifact events come from the actual #18/#19/#20 publication, retrieval,
condensation or expansion consumer. Pin Project/Goal/Task/revision, artifact
identity/version/source digest, actual bytes, selected budget and operation links.
Prepared recovery.input_bytes is not published-pack size; a stored reference is
not actual delivery. Delivery records link each dispatch's exact sent content or
proven runtime-controlled retained content to the reduction-independent mandatory
set/policy digest. Native self-loaded rules remain separate exposure, with no
internal attention proof. Own condensation/replacement requires mandatory delivery
again; its omission is absolute safety loss, not merely unavailable measurement.

All writes share the final composed Store writer epoch. Generic Record/Usage/audit
writers cannot create private rows or reserved telemetry/benchmark events. Retire
Store::put_usage/usage and normal AgentAdapter::usage raw-metadata collection;
legacy rows are immutable unqualified history, never backfilled into owned calls.
Migrate every actual runtime/query/test caller to its proper typed port. Legacy
compatibility may expose bounded counts/status only, not raw cache_metadata.
Explicit migration follows #19 drain/recovery/preflight and old-connection fencing;
failure leaves original bytes/schema/history unchanged. No implicit open upgrade.

## Bounds and scoped queries

Initial independent limits: descriptor/event encoded64KiB, recognized identity128
UTF8 bytes, digests32 bytes, depth16, 64 beneficiary/partition links per operation,
4096 operations per Task and 65536 per plan, 64 observations per operation,
4096 artifact events per Task, 32 currencies/model-unit groups per page, page128
rows and encoded output256KiB. Plan128 attempts, 64 feature/oracle registrations,
4096 oracle items and total encoded plan1MiB; per-attempt retained metadata1MiB.
Aggregate ledger bytes are capped independently (64MiB per Task, 256MiB per plan).
These maxima need not fit simultaneously. Capacity refuses the new effect before
dispatch, exposes bounded attention and preserves existing ownership/history.
No query truncation or exhausted history is evidence of complete coverage.

Project query capability comes from trusted application selection of that Project;
Runtime-wide/operator capability comes only from local operator ingress. They are
nonserializable and cannot be minted by native/Workflow/Broker channels. This is
the approved application boundary; same-UID/direct DB or malicious linked code is
outside it. Do not introduce a biological Human/OS security claim.

Use a deferred read transaction for count/byte projections, inventory completeness,
observations and source links. Verify row/body scope before bounded decoding; refuse
oversize/malformed/foreign state with safe codes. Keyset cursor binds scope, filters,
snapshot digest and last immutable identity. A changed snapshot returns StaleCursor,
requiring a fresh query; no mixed-snapshot total. Query output contains only safe
identities, numbers, coverage and reason enums, never prompt/transcript/env values,
raw native errors or arbitrary metadata. SQL projects only required bounded fields.

Direct Task/round/phase metrics contain their own disjoint qualified operations.
Shared beneficiary overhead is separately reported. A full lane sum alone can
establish end-to-end reduction, and only with complete inventory/coverage. Group
input/output/cache/cost by compatible model/unit/category/currency; show measured
and unknown operation counts, failure/cancel/retry counts and incomplete totals.
Cached ratio is unavailable for zero/missing/incompatible inclusive input.

## Immutable benchmark plans and outcome axes

Trusted benchmark composition registers a plan under the shared epoch before the
first dispatch. Descriptor freezes workload/initial source, actual shipped default
and Project policy digests, baseline changes, native/provider configurations,
features, counter contracts, oracle/adjudication, repetition count/time stopping,
counterbalanced order, tolerances and apparatus classification. Every started
attempt has a permanent inventory/outcome; cleanup never deletes plan history.
Own old native history/config/defaults remain unchanged; use distinct owned lane/
repetition paths, session IDs and fixture repositories, recording known exposure.
Exposure/confounding refuses causal/equivalence claims rather than claiming secrecy.

Baseline disables evaluated reduction only; preserves mandatory safety/context,
native defaults, independent reviews and equivalent workflow gates. Exercise actual
required context producers, at least two reviewers and a round>1 delta re-review.
Never route current peer findings or executor full chat to another reviewer.
Native/provider setup and reduction-generated preparation charge the enabled lane;
apparatus work is separately reported and never feeds the execution/gates.

Each attempt separately records measurement validity, execution outcome, absolute
safety and per-item paired oracle detection. Invalid telemetry does not hide quality
or safety outcomes where outputs exist. Runtime-owned missed mandatory delivery is
an absolute safety failure; safety failure in either lane blocks acceptance. Oracle
enabled-only misses beyond frozen tolerance are relative reduction regression;
equal/baseline-only misses are disclosed detection limits, not enabled safety credit.
Environmental labels require independently evidenced cause outside runtime-owned
code. Shared runtime defects remain execution failures. Unknown outputs remain
quality-unverified; timeout/failed lanes stay in the denominator.

All certifying sensitive items, including first-plan items, require independent
creator/feature-author exposure records and pre-run registration. The creator may
know public acceptance contracts but must not tune items to feature heuristics;
feature authors cannot tune against certifying items. Record provider/family and
exposure as confounds without inventing an OS secrecy guarantee. Keep answers out
of lane inputs/caches/fixture trees/history. Author-overlapping or tuning-exposed
items are regression-only. Frozen baseline discrimination must establish sensitivity
for every exercised feature; both-missed items do not nominally cover a feature.

Pool ALL attempts across same-source/configuration plans with unknown/failing axes;
never stop after N valid, replace invalid attempts or choose latest favorable plan.
Any invalid required attempt prevents whole-run causal/equivalence certification
for that source/configuration. An unchanged successor may disclose diagnostics but
cannot repair that certification. Changed behavior/configuration requires registered
successor identity and actual fixing evidence; a hash/apparatus edit alone cannot
launder prior regressions. Keep prior cases and independent fresh held-out affected
feature items. A disabled feature must actually be default-off or fixed, and remains
subject to required MVP producer acceptance. Whole-lane claims require all exercised
features covered; narrower ablation claims have their own predeclared same inputs.

Completion/cost/tokens/wait/throughput/review timing uses actual producer events.
Durations require an owned monotonic clock interval; across restart/missing boundaries
remain unavailable. Human interruptions/attention require actual trusted ingress/
UI events; no model guess or missing zero. CPU/byte overhead stays separate.
Freeze output report digest and append later status: changing actual shipped default,
policy/source/contracts stales old equivalence/savings claims without rewriting them.
Closure requires real qualified native input/output comparison, complete coverage
and independent safety/quality evidence, not positive savings; where-available cost/
cache remain nullable. #16 still owns complete Goal/multi-Project/four-Task dogfood.

## Source inventory, controls and gates

Before source, enumerate all Usage/store/query/audit callers, owning native parser/
counter paths, helper dispatches, context artifact consumers, timing/Human producers,
benchmark/README/TUI consumers and every epoch writer. #5/#6/#7 supply actual native
measurements, #18/#19/#20 artifact/mandatory delivery, #9/#10 slots/decisions,
#23/#24 lifecycle, #14 restart ownership and #16 final native evidence. Missing
ports leave their required producer acceptance open; a registry table is no proof.

Actual controls cover replay/conflict/no audit bump, stale scope/epoch/old connection,
wrong model/unit, reset/late/cumulative semantics, nested totals, helper omission,
shared overhead, monetary schedule change, query scope/overflow/page drift, genuine
sent mandatory context, every attempted plan, all outcome axes, initial/successor
oracle independence, true default pins, known exposure and retained negative history.
Use required native parser/transport conformance plus controlled deterministic cases;
synthetic measurements cannot satisfy the native benchmark.

Compile isolated mutants at real aggregation/query/report/certification consumers:
skip failed/helper inventory; accept collector phase/model or foreign artifact;
double-count containment/cache categories; sum invalid/reset counters; replay
distinct calls by equal payload; omit shared overhead; use current prices for old
events; leak legacy metadata/project rows; mix snapshot pages; omit epoch guard;
mark own mandatory loss unverified; valid-only/until-valid pool; drop unfavorable
history; certify tuned/or author-overlapping items; disable a feature only in the
benchmark while default remains on. Each must reach its intended assertion with
passing positive controls, exact restoration and no unrelated timeout/setup credit.

Run relevant native/context/Store/Workflow/registry regressions, fmt/clippy/tests,
debug/release builds, exact Linux/macOS CI and two independent immutable source/
security reviews. Current approved-Req8 CI's old inspection cap-test failure is
preserved in verification; #55's related real source gate is pending. No rerun-only
fix, native defaults change, deployment, source acceptance or MVP completion is
implied by this design. Update master/README only to actual implemented behavior.
