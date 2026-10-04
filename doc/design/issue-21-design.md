# Issue 21: Qualified telemetry and native comparison

Status: Design4, verified independent Design1–Design3 corrections; design/source gates pending.
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
Actual runtime Git/verification ownership under pending #60 also remains required;
its process/effect records are not automatically provider token observations.
Design/source changes must inventory actual latest implementations before wiring.
Coordinate one migration with their final composed epoch, not a separate schema6.

## Producer possession and complete operation inventory

Add crate-private telemetry interfaces in telemetry/{identity,normalize,store,query,
benchmark}. Public reports are read-only numeric projections. Trusted runtime
composition installs adapter implementations and immutable counter contracts;
model/config JSON cannot register one. Controlled transport implementations have
the same caller contract; installed implementation identity determines synthetic
provenance, rather than a transport-declared label. Collect only native output and
protocol channels the adapter already owns, without exporters or new native setup.

An actual supervisor acquires a nonserializable measurement-only handle before
every runtime-owned provider operation. Identity contains Project, optional Goal/
Task, actual native operation and input/attempt identity, runtime instance, adapter
implementation/profile, source revision, operation purpose and beneficiary links.
Capture effective reduction-feature, Project-policy and native-configuration digests
from the actual runtime composition at that dispatch, separately from plan claims.
The native-configuration digest covers only rrx-owned launch/profile/composition
parameters and allowlisted observed protocol facts, never credential/user-native
configuration files. Missing attestation keeps configuration measurement unverified;
an optional projection failure cannot imply agreement with the plan.
Task dispatches derive generation/index/phase/context/round/slot from the actual
owning Workflow/ReviewSet transaction; callers cannot supply attribution labels.
Task-free and helper operations have explicit purposes rather than fake Tasks.
The handle cannot grant native input, cleanup, release, review or Goal authority.

The inventory source is the owning producer's existing durable operation/admission
record, written before effects under that producer's policy. Join its identity and
immutable descriptor; do not create a separate telemetry prerequisite for dispatch.
Publish optional measurement projections only after the producer's owning admission
commit, in a separate bounded telemetry transaction; its failure cannot roll back
or delay awaiting telemetry retries before dispatch. Completeness joins the existing
producer record, rather than requiring atomic telemetry publication. Existing admission or
mandatory context storage failures retain their existing meaning. Measurement-handle
or telemetry storage/capacity failure cannot change native or context behavior.
The adapter receives the handle when available and publishes bounded observations
from owned responses. Startup failure, cancel, retry, missing collection, publication
conflict and Drop remain in the producer inventory with unknown measurement fields.
A retry is a distinct operation; unchanged input is no replay exemption. Unknown
durable ownership stays under producer recovery; telemetry never releases it.
If producer inventory or its correlation cannot be proved complete, scope coverage
is unknown, including when the optional projection or failure marker cannot persist.
Queries never infer completeness from the surviving telemetry rows alone.

All reduction helpers, selection/condensation/expansion generation, initial setup
and work outside Task phases use this same inventory. Owning helpers record exact
beneficiary identities without assigning shared work wholly to one lucky Task.
Native internal calls need either a verified complete encompassing measurement or
a complete disjoint partition. Unknown unobserved internal/delegated work makes
coverage incomplete; do not invent inner call IDs or assume one CLI result means
one model call. Separate apparatus calls use predeclared apparatus identity and
cannot feed lane execution or acceptance gates.

Actual attach/input-capable intervals and other unowned native turns are inventoried
as unowned/Human-purpose intervals linked to the native session, without assigning
Task phase/round authority. Observation ports cannot grant attach permission.
An attach during an owned operation, between owned turns, or any unprovable interval
gap invalidates cumulative-derived attribution spanning that interval. Only counters
whose native contract proves exact owned-turn scope remain qualified. Codex's replay
of a previous owned total alone does not prove absence of intervening unowned turns.
Unknown native-internal compaction/helper intervals also leave coverage unknown.
Continuity requires POSITIVE durable evidence independent of optional telemetry:
the actual ingress/attach producer's own grant/close records, complete adapter-owned
turn identity boundaries excluding foreign turns, or a native contract proving the
counter is scoped to the exact owned turn. Absence of interval/telemetry rows never
proves continuity. Attach/input-capable windows without those records, and missing/
overflowed interval projections without another complete owned source, are unprovable
gaps for that session. Pending #6/#11/#15/#19/#58 ingress producers must actually
provide these facts; otherwise cumulative-derived deltas remain unknown.

## Field normalization, model units and replay

An immutable CounterContract identifies implementation/version, operation/turn
scope, native paths for observed consuming model/tokenizer identity, counter units,
incremental/cumulative semantics, per-field reduction rule, category partition and
containing/contained relationships. It defines interpretation, not observed model
facts. Each observation/partition node derives its actual consuming model from that
event's native data. An initial session/contract model is eligible only with native
proof that this operation cannot fall back or use helper models; otherwise a mixed
observation without a complete per-model partition remains unqualified. Qualification
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

Observation identity is (owned operation ID, native response/turn ID), or an ordinal
assigned once at the actual owned native event/stream position when no native ID
exists. Collection channel, status/final/recovery path and producer kind are not part
of the identity of one event. Identical finalized canonical content is a no-op;
conflicting content rejects without replacing evidence. Wall time, equal payload
and Session ID alone do not deduplicate distinct operations. Ordinals and versions
are positive checked i64. Intermediate cumulative notifications update bounded,
non-additive progress for that same turn; they do not consume finalized-observation
slots. Accept one immutable final snapshot. A missing trustworthy final stays unknown.
Each adapter CounterContract names and validates its actual terminal native event
(for example Codex turn completion or Claude result), without accepting a status
label or timeout as final. Grok remains unqualified until its own native contract
and conformance establish these semantics.
Each CounterContract selects exactly one effective value per operation, field and
partition node: a qualified final cumulative snapshot supersedes progress, while
incremental events sum only when native evidence proves disjointness. A snapshot
and a delta derived from it cannot both contribute. Unknown combination is invalid.
Late facts append versioned missing-coverage evidence but cannot rewrite failures,
ownership or frozen benchmark results. Each attempt pins a predeclared measurement
cutoff; post-cutoff facts are diagnostic and cannot repair its scored validity.

Native monetary estimates remain separate from operator-derived estimates. Native
finite values retain their declared precision and scope; no billing claim. Computed
cost uses checked fixed-point/rational arithmetic under an immutable operator-
activated PriceSchedule ID/content digest: model, currency, effective interval,
category rates, thresholds and rounding. Unknown model/categories/rate partition
make computed cost unavailable. A schedule change creates a new ID; queries never
reinterpret old results with today's prices. Keep currency groups separate and
report estimate basis/rounding. Price activation is trusted application ingress,
not agent JSON; no automatic price lookup or current-price correctness claim.
Every threshold/rate declares its billing evaluation unit and counter basis (such
as each native model request, rather than a turn/invocation containing many calls).
Apply it only where CounterContract scope proves that exact unit, or a complete
disjoint partition proves every included call's eligible tier. Aggregate tokens
crossing a request threshold cannot price all calls at that tier; no average-call,
worst-tier or requested-model substitute. An aggregate may use a uniform tier only
when actual contract/partition evidence proves ALL included billing units occupy
that tier, including helper models and category overlap. Otherwise computed cost
is unavailable, even with qualified turn tokens. A two-call turn whose aggregate
crosses a per-request threshold but individual calls do not must refuse an aggregate
tier estimate/comparison until its per-call tier eligibility is actually proven.

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
The actual #19/#20 prompt/dispatch producer owns mandatory-content delivery evidence
in its own admission/dispatch and wire/retention records under existing mandatory-
context policy, outside optional telemetry tables. A prepared manifest or consumed
authorization is not proof of sent bytes. Telemetry projects actual sent-item digests
or proven controlled-retention references, never manufactures them. Its projection
write failure leaves the producer facts intact and metric coverage unknown.
If the runtime commits a next dispatch after its own condensation/replacement/
selection/loader without the required producer re-delivery/retention record and
delivery, score MandatoryDeliveryLoss as absolute safety failure; dropping BOTH the
actual re-delivery and its record cannot become measurement-unverified. Independent
known loss stays loss even if other evidence is missing. A mandatory-context refusal
before dispatch retains its actual execution outcome. Unknown retention depending
on unowned native behavior after recorded delivery stays safety-unverified and
cannot certify acceptance; optional telemetry failures never ratify missing safety.

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
These maxima need not fit simultaneously. They bound telemetry detail, never native
turns, retries or context effects. Reserve one fixed-size scope overflow/unknown
marker outside detail capacity; on exhaustion preserve prior detail and attempt that
marker while dispatch follows existing producer policy. If even the marker write
fails, reconciliation against producer inventory/correlation remains conservative:
coverage cannot become complete. Do not clear an overflow by dropping old attempts.
Benchmark plan/attempt capacity may refuse a NEW benchmark registration before its
first dispatch; it cannot suppress a running attempt or its negative outcome.
No query truncation or exhausted history is evidence of complete coverage.

Project query capability comes from trusted application selection of that Project;
Runtime-wide/operator capability comes only from local operator ingress. They are
nonserializable and cannot be minted by native/Workflow/Broker channels. This is
the approved application boundary; same-UID/direct DB or malicious linked code is
outside it. Do not introduce a biological Human/OS security claim.

Use a deferred read transaction for count/byte projections, inventory completeness,
observations and source links. Verify row/body scope before bounded decoding; refuse
oversize/malformed/foreign state with safe codes. Keyset cursor binds scope, filters,
snapshot digest and last immutable identity. Pin append-sequence high-water marks
for producer inventory, finalized observations and coverage versions, so concurrent
appends do not prevent paging through an immutable view. Changed scope/filter/epoch
or unavailable pinned history returns StaleCursor; no mixed-snapshot total.
Pin producer row versions for mutable outcomes as well as append positions; read
the corresponding immutable outcome facts/versioned projection at that high-water
snapshot, or refuse unavailable history rather than mixing later status changes.
Query output contains only safe
identities, numbers, coverage and reason enums, never prompt/transcript/env values,
raw native errors or arbitrary metadata. SQL projects only required bounded fields.

Direct Task/round/phase metrics contain their own disjoint qualified operations.
Shared beneficiary overhead is separately reported. A full lane sum alone can
establish end-to-end reduction, and only with complete inventory/coverage. Group
input/output/cache/cost by compatible model/unit/category/currency; show measured
and unknown operation counts, failure/cancel/retry counts and incomplete totals.
Cached ratio is unavailable for zero/missing/incompatible inclusive input.

## Stable prefix, provider cache awareness and amplification

At the actual #19/#20 authorized prompt-construction consumer, construct stable
prefixes by deterministic ordering of reusable instructions/tool definitions/source
slices. Key the authorized input by exact Project identity, source revision, rules,
artifact/version/content digests and phase/current authority. Equal authorized input
has byte-identical prefix; any key change invalidates prior material. No cross-Project
or cross-lane reuse, current peer findings, stale evidence or cached admission proof.
Native-owned prompt portions remain native; no promise that the runtime controls
their ordering or cache namespace. No raw cache contents enter telemetry.

Discover cache capabilities through the actual registered adapter and supported
owned native protocol/version. Persist only bounded supported/unsupported/unknown
capability enums and numeric observations. Use explicit cache channels only where
already authorized and supported; unsupported remains visible. Do not alter model,
effort, credentials, hooks, permissions, history or native defaults to manufacture
a hit. Warm, cold, expired and unavailable cache paths retain identical mandatory
evidence, freshness, admission, reviewer independence and completion/cleanup gates.
Fallback uses equivalent current authorized input or explicit native unsupported/
error behavior. Cache performance is never evidence of logical input reduction.

Register the single-reviewer reference descriptor as separate predeclared apparatus
before lane execution, on the exact immutable review bundle and consuming model/
unit/category definition, but run the reference ONLY AFTER all lane operations and
repetitions it could warm have frozen outputs and passed their measurement cutoff.
Pin this order in the plan; it cannot feed findings, gates or lane cache state.
Any known earlier/concurrent reference with a potentially shared provider prefix
marks affected attempts cache-confounded, preventing clean cost/cache/time claims;
preserve that exposure in subsequent plans rather than assuming a fresh Session
clears provider caches. An alternative order requires actual native proof of cache
namespace/prefix separation, not distinct runtime Project labels. Record reference
cache state as apparatus-only. Amplification uses inclusive logical input, so the
reference's own cache hits do not shrink its denominator. For each compatible
model group, amplification is actual independent-reviewer input divided by that
measured single-reviewer reference; expose slot/count, shared/specific and cache/
uncached components with coverage. Heterogeneous rosters need per-model references
and ratios; no mixed-model token sum. Replacements retain original attempts and new
slot/roster versions. Missing/zero/incompatible references yield no ratio, never a
packing estimate substitute. Reference calls are apparatus inventory, not lane work.

## Immutable benchmark plans and outcome axes

Trusted benchmark composition registers a plan under the shared epoch before the
first dispatch. Descriptor freezes workload/initial source, actual shipped default
and Project policy digests, baseline changes, native/provider configurations,
features, counter contracts, oracle/adjudication, repetition count/time stopping,
counterbalanced order, tolerances and apparatus classification. Every started
attempt has a permanent inventory/outcome; cleanup never deletes plan history.
Derive the shipped-default digest from the tested source/build's actual compiled
default configuration, and reject a claimed default that differs. At each actual
dispatch compare its runtime-derived effective feature/policy/native configuration
digests against that lane plan; a mid-run change or non-default enabled composition
is a retained execution/configuration failure, not a valid reduction experiment.
Record native observed configuration separately when available; a runtime digest
does not prove unreported native model/effort, whose unknown confound remains.
Own old native history/config/defaults remain unchanged. Give EACH lane AND
repetition distinct owned fixture paths/repositories, Project/Goal/Task identities
in the shared registry and native sessions. Pin only permitted canonical identity
substitutions in the plan; all Pack/expansion/query channels enforce that boundary.
Record known exposure.
Exposure/confounding refuses causal/equivalence claims rather than claiming secrecy.
Report known lane-to-lane/repetition provider cache warmth, actual eligible cache-read
observations and pinned counterbalanced order along with apparatus exposure; distinct
runtime sessions/Projects do not prove cold provider caches.

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
Plans store preregistered answer commitments/digests, not answer text; actual
independent adjudication reveals answers outside lane inputs only after outputs
freeze, while preserving item/creator/exposure identities and the frozen oracle.

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
Exercise repeated Codex cumulative notifications and status/final/recovery views
through actual aggregation/query/report; prove one finalized effective value.
Attach between owned turns AND during a turn leaves interval-derived usage unknown.
At both operation/ledger capacity and actual telemetry write failure, native and
mandatory-context behavior stays unchanged while coverage becomes unknown. Test
cross-lane Pack/expansion/metrics refusal, mid-run policy drift, measured amplification
references and current stable-prefix invalidation. Warm/cold/unavailable cache paths
have the same actual freshness/admission/evidence/gate results.
Lost/overflowed attach-interval projection still yields unknown cumulative attribution
at the actual query/report, never inferred continuity. A deliberately early reference
is refused or retained cache-confounded for every affected later lane/repetition.
Dropping both actual mandatory re-delivery and its producer record after own
condensation is scored safety loss at the real dispatch/report consumer; dropping
only optional projection preserves producer safety evidence with unknown metrics.
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
Additional actual-consumer mutants sum cumulative notifications, double-count one
event through two collection paths, refuse native/context effects on telemetry
capacity or write error, attribute an unowned attach interval, substitute a contract
model for a native fallback, cross lane context/query scopes, certify actual config
drift, and use cache presence to bypass freshness or mandatory evidence. Kill these
at their real query/report/dispatch/context/certification consumers, not helper-only
checks; restored positives must prove the relevant native/context prerequisites.
Also mutate absence-of-interval-row into continuity, early apparatus warming into
an unconfounded report, and simultaneous own re-delivery/record loss into mere
measurement-invalid. Actual producer prerequisites distinguish safety loss from
projection failure and prevent helper/timeout-only mutation credit.
Cost/report controls and compiled mutants apply request-level tiers to a multi-call
aggregate without proven partition, compare that fabricated cost, or mix mutable
producer outcome versions across pages. Restore exact actual cost/query positives.

Run relevant native/context/Store/Workflow/registry regressions, fmt/clippy/tests,
debug/release builds, exact Linux/macOS CI and two independent immutable source/
security reviews. Current approved-Req8 CI's old inspection cap-test failure is
preserved in verification; #55's related real source gate is pending. No rerun-only
fix, native defaults change, deployment, source acceptance or MVP completion is
implied by this design. Update master/README only to actual implemented behavior.
