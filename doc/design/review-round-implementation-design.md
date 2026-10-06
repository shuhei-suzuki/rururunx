# Managed ReviewRound implementation and schema10 connection

Status: proposed implementation supplement; no source or native qualification.
Baseline `5b6e8a8152926b643c6926b87e853de12b9319fc` supplies the integrated
Native6 collector/receipt and Recovery7. Corrected Verifier8 is fixed at
`00d43a185bcab7d16a8c3cdf019bbb3b34eb6167` on its separate branch; it has not
yet been composed with this baseline. Operational Runtime/Goal schema9 is a
required pending dependency. Root assigned ReviewRound **schema10** after that
actual ordered composition. This supplement maps the approved
[requirements](../requirements/review-engine-integration-requirements.md),
[design](review-engine-integration-design.md) and
[owned native receipts](native-result-receipts-design.md) to current producers.
It changes neither roster/floor policy nor the independent native acceptance work.

## 1. Existing consumers and required seams

| Current source | Actual constraint | ReviewRound connection |
| --- | --- | --- |
| `workflow.rs::prepare_managed` and `adapter/native.rs::start_managed` | One `PhaseAttempt.agent/unit/session_id`; adapter compares exactly that Unit and `Context.data` payload. | Managed Reviewer phases delegate to ReviewEngine before ordinary preparation; new round reference replaces representative Unit/Session, and the adapter consumes a private member admission branch. |
| `state/execution.rs::checked_workflow_binding` | Phase must be undispatched, with no Unit/Session; reserving the first Unit consumes that reservation. | Separate checked Round/Worker reservation does not call this ordinary single-Unit producer for siblings. |
| `state/execution/native_results.rs::bound_context/reserve_native_input` | Complete actual payload equals serialized current Context data; current Task context is checked before writing input. | Review launch checks current launch Context independently, plus exact registered member payload bytes/hash. Do not substitute model echoes or delete the Context check. |
| `execution/results.rs::ResultSnapshot::completion` | Private verified snapshot, exact retained artifact and known successful Unit terminal; its proof authorizes current single-review Workflow acceptance. | Reuse real snapshot production/verification, then create a member-bound private completion. No member directly advances Workflow. Failure content never manufactures successful readonly provenance. |
| `state/execution/native_results.rs::finish_native_result` | Owned actual terminal inserts receipt and closes work/Session/quota together, or retains a historical draft; `NativeStatus.result` is only its projection. | Same acquisition producer persists native answers. Round binding is supplied by the registered member invocation and rechecked for current ownership; parsing reads the durable receipt by its exact indexed invocation. |
| Corrected Verifier8 `VerificationAbandonment` and `VerificationCompletion` | Lifetime guard remains owned from registered Unit through private completion handoff; known terminal is persisted before fallible stream retention. | Review worker owns an equivalent guard through parsed-result/readonly completion and SQL member closure. A detached/aborted future cannot leave an open worker grant. |
| `execution/quota.rs::QuotaScheduler` | Single actual SQL lease authority; current per-Unit admission is not an atomic cohort producer. | Reuse pool observations/leases and configured caps; add private Round-aware serialized admission and atomic complete-roster cohort reservation. Missing qualified schedule/capacity producer holds before effects. |
| `state/mod.rs` Workflow transition and C8 typed completion port | Generic Record/Evidence cannot attest managed Tests; private proof and exact typed SQL acceptance compose. | Add an independent managed-review marker and collective typed acceptance port. No generic Passed/review-approved value or ordinary readonly transition completes a managed Round. |

The existing Native receipt is necessary but insufficient: it binds Unit, Session,
epoch/generation, actual input, Context, source map, target/artifact and provider
turn. It has no Set/Round/Slot ownership until the new durable member binding is
registered. `ManagedInput` is currently a public DTO; adding a public Round UUID
to that DTO cannot be the new authority.

## 2. Private authority and actual path

Add `review/` for typed policy, bundle, result validation and orchestration, and
`state/review/` for bounded indexed reads and transactional mutations. Durable
DTOs, strict JSON parsing, public status and SQL row names carry no capabilities.

1. The real managed Workflow activation obtains a private review activation from
   accepted Runtime9 Goal/Task policy and the actual retained Owner. Persist a
   managed review contract/marker in the Workflow initialization transaction.
   Existing managed Workflows migrate to a named hold; legacy review evidence
   is retained without being converted into a certificate. Old success writers
   remain denied even before a Round/Unit exists. A library caller, Agent/TaskTool
   request, generic Record or typed Goal DTO cannot mint this activation.
2. `ReviewEngine::prepare` consumes the exact active reviewer phase reservation,
   current accepted driver claim and retained source/artifact provenance. A
   private bundle producer freezes identical mandatory core, additive focus,
   current activated governing policy, cumulative authors and declared profile
   overhead. Tracked target rule/config proposals do not govern their own review.
   Missing accepted activation, attribution/delta, mandatory evidence, instruction
   compatibility or qualified native schedule is a named pre-effect hold.
3. `Store::reserve_review_round` in one Immediate transaction rechecks the entire
   Scope/Task/Workflow/Context/Project/Goal/artifact/driver/activation claim,
   lineage budget and immutable bundle; registers Set/Round/Slots/Inputs/Workers
   and a `PhaseAttempt.review_round` reference. No first-member native launch is
   hidden inside this transaction, and no Git/native await holds a SQL lock.
4. Resource draft, Unit insertion and leases use the shared Owner resource gate.
   `Store::reserve_review_member` binds exactly one fresh Reviewer Unit to the
   current worker, same Task execution generation and registered input/profile.
   It returns a non-Clone private-field `ReviewMemberGrant`, never a DTO-derived
   reconstruction. Release the resource gate before materialization or Git await.
   A preparation/worker guard is already armed at Unit registration.
5. `AttemptManager`'s new member snapshot entry accepts only that grant and uses
   the real retained artifact and disposable readonly input/output namespaces.
   Native/helper/delegated/resource admission checks the durable member contract
   through the actual Unit authority validator. Ordinary `prepare_snapshot`,
   `start`, public managed launch or standalone Session operations cannot adopt
   a marked member Unit. Input instructions/settings/hook compatibility must be
   qualified by the frozen profile; source permissions are cooperative protection,
   not a same-user security boundary.
6. The actual adapter vtable gains a private member launch port or equivalent
   inaccessible authority variant, consumed by the real NativeAdapter. It checks
   exact alias/provider/model/effort/Project/role/path/member input. NativeSessions
   checks it before version helper, Session registration and actual stdin intent;
   Session plus native invocation plus Worker binding are inserted atomically.
   Native input intent rechecks complete current Round/Worker/input/Context and
   schedule/cohort authority immediately before spawn/write, under the existing
   actual effects boundary. A cloned DTO or old grant cannot dispatch again.
7. Core retains the private member launch identity with its existing owned process
   and actual answer collector. All live Session/quota/input acknowledgments and
   owned terminal classification recheck that same durable binding. Work terminal
   remains known even if review parsing/retention later fails; stale input receives
   historical-only result authority. No earlier reaping of the leader is added.

The new private port must be exercised through registered Claude/Codex adapters,
their ordinary actual stdio/Core and Store consumers. A test-only qualified native
peer can exercise orchestration; it cannot qualify the official CLI's hooks,
authentication, channel independence or model behavior. Formal Grok/Triple remains
unavailable until a real compatible managed provider is qualified.

## 3. Schema10 and complete claim currency

Use the approved Set/Round/Slot/Worker/Input/Result/Finding/Disposition/Certificate/
Lineage tables, plus `workflow_review_contracts` and indexed `review_member_units`
to distinguish marked Units at every ordinary admission/acceptance boundary.
The member association binds Round/Slot/Unit once and Session/invocation once;
UNIQUE Unit/Session/invocation prevents cross-slot reuse. Typed reads validate
every body/index/FK identity with SQL byte/count bounds before decoding. Immutable
inputs, roster/policy/profile, findings and certificates cannot be rewritten by
projection updates. Result source is the exact native receipt and invocation,
not a copy of unverified `NativeStatus.result`.

Before registering input reserve a checked complete lineage charge: each actual
owned core/member/frame copy, the Native6 complete encoded receipt ceiling of
2 MiB per invocation (not only its 1 MiB decoded answer), each retained parsed
envelope/claim/row/index/audit copy and explicitly bounded diagnostics/expansions,
plus the approved 8 MiB verification and 4 MiB control allowances. Each write
consumes its reserved allowance in the same transaction; overflow retains only
the separately reserved diagnostic representation and holds certification.
Equal hashes do not make separate retained copies free. The inherited shared
64-round/128 MiB lineage ceiling applies before effects; support for a roster
count does not imply that all maximum-size profiles fit that ceiling together.
Unsupported/unbounded producer overhead refuses that profile rather than
silently omitting its acquisition/storage cost.

Frozen semantic pins include launch Context/full source map, Task execution and
Workflow generations, Task instructions, accepted Goal definition/driver policy,
governing policy/author lineage, full exact retained artifact, core/focus/input
hashes and original qualified schedule. Mutable current Task/Workflow projection
versions are advanced only in the same transaction that writes a legitimate
member wait/closure projection. Native telemetry changes its own Unit/Session
versions without changing those semantic pins. Sibling updates must never
silently refresh an unrelated Project/Goal/source/Context claim.

Central `validate_review_unit_tx` is called by the existing `validate_authority`
for marked member Units whenever native or finalization effects are requested.
The exact current member row/Round/Workflow/Task pins are therefore checked by
Git, version helpers, resources, tool/delegation and quota consumers as well as
Session/input. Session-specific checks add its bound invocation; actual stdin
adds registered member bytes and original schedule. Inspection/historical reads
are separate and cannot reopen authority. Cancellation/fresh-round/epoch recovery
fences all workers in one transaction before physical stop requests.

Runtime9's driver claim must remain valid across legitimate member-only projection
changes through its typed same-transaction hook. Review code will consume the
actual private driver API after the fixed schema9 delivery; it will not invent
accepted Goal authority from current `runtime::GoalDefinition` DTOs or infer it
from a Running label. Changes to shared schema, Workflow/native glue and driver
hooks wait for that fixed composed baseline and receive independent source review.

Install fresh schema10 and ordered 9→10 only after actual 8→9 exists. Add every
mutable table to writer guards and reinstall existing guards against contract10;
cached/open schema9 writers and pre-schema10 reopen writers reject writes.
Namespace/version/guard/marker/audit migration is one rollback-safe transaction.
No skipped schema number, parser-only install or legacy boolean certification.

## 4. Scheduling, closure and aggregate acceptance

Policy validation supplies N=1–32/all/quorum/any with the independent floor,
explicit author eligibility, real provider aliases and Triple requirements. The
frozen qualification producer supplies SERIALIZED_CAPABLE or CONCURRENT_ONLY,
supported finite startup/queue/opinion bounds and exact channel/instruction proof.
Requested profile strings and version checks alone do not mint this capability.
Serialized members obtain only their own SQL quota permits, rechecking the
original finite schedule before every actual input. Concurrent-only reserves the
entire eligible cohort's global/project/provider capacity atomically or reserves
zero; queued cohorts hold zero permits, create no partial native inputs, and do
not degrade to serial execution after a startup failure. Admission is separate
from resource-leasing mutual exclusion.

Quota live waits preserve the same Unit/Session/invocation/input. Durable worker
wait updates and Round Task/Workflow projection are one transaction: Reviewing
if any unresolved member is active/runnable; WaitingQuota only when every
unresolved member is quota-parked. Accepted same-bucket recovery changes only
the owning worker and its exact authority. Due wait recheck claims the exact
current row/version, not a copied request. Terminal quota interruption closes
the invocation and noncertifying Round with retained content; authorized bounded
recovery reserves a fresh complete roster, no carried approvals or same-round
second input. Quota wait does not extend the qualified pre-input schedule.

The strict result validator consumes original owned answer bytes from the bounded
durable receipt through `strict_json::decode`, then typed deny-unknown-field schema.
It requires exact Round/Slot/generation/artifact/core/policy/input echoes and
validated target-relative locations/evidence references. Complete envelopes and
complete encoded finding/disposition entries obey the approved byte/count bounds.
Duplicate keys, changed receipts, missing/partial/ambiguous/overflow content and
unseen suffix retain finite diagnostics and inspection obligations; they never
become approvals. A valid APPROVE on native Failure/Unknown remains original
opinion content, with zero mode/floor votes and any potential blocker preserved.

A private `ReviewMemberCompletion` joins the actual native receipt, parsed/invalid
result, registered member input and successful readonly snapshot provenance where
available. The lifetime guard stays owned through `Store::close_review_member`;
it is not disarmed when only the native terminal or snapshot has been obtained.
That transaction stores findings/results, closes only that worker and finalization
flags, releases exact waits/permits and schedules cleanup. Failure/cancel uses
an explicit noncertifying closure; it does not require or forge a successful
readonly completion. Preserve known native work, failure content and historical
diagnostics even when current acceptance loses CAS. Uncertain physical resources
are quarantined; cleanup status alone never rewrites work/opinion or blocks a
valid independent future namespace. A guard Drop closes unfinished logical
permissions conservatively, without asserting SQLite/OS availability guarantees.

Persist an immutable member-completion record only from that private actual
receipt/snapshot proof in the closure transaction, binding its input manifest,
artifact, native receipt and result digests. An arbitrary readonly boolean or
deserialized completion DTO cannot create it. After closure, finalization flags
remain closed: collective retained-graph re-verification uses current Runtime
registered retained-read intents, not `ResultSnapshot::completion` or UnitGit on
the already closed member. The genuine before/after member snapshot observation
is the closed member's immutable evidence; aggregate verifies its exact stored
binding and the independently re-read retained graph, without granting a second
native input or manufacturing new historical snapshot success. A crash before
that private closure produces a noncertifying unknown member/fresh authorized
Round, not recovery by changing its epoch or inferring success from a Unit row.

After all roster members logically close, reveal results for verification. Typed
finding inspection/clearance keeps original claim and actor evidence distinct;
unsupported inspection, disputed/ESCALATE/unexamined content and potential blockers
hold the certificate. Fix/re-review requires a fresh Executor artifact/delta,
full new roster/Sessions and inherited authors/lineage charge; no result sampling
or quorum policy relaxation clears an unresolved finding.

Private collective completion re-verifies retained inputs and exact current
member/disposition records. The final `Store::publish_workflow_review` Immediate
transaction independently recomputes the countable conjunction: owned current
native Success + successful readonly proof + exact valid receipt/input/envelope
+ successful logical closure + APPROVE. Independent floor adds non-author
eligibility to that same predicate. It checks mode, every blocker/disposition,
original qualified schedule, driver/current complete owner claim and reserved
lineage charge. Certificate, typed review Evidence, reviewer phase success,
Task projection, next consecutive Context and audit publish atomically. A managed
review marker rejects every generic/no-Unit/single-readonly success writer.

## 5. Component delivery and verification limits

First deliver isolated typed policy/result/claim modules and this mapping without
schema/glue installation. Their parsers/data validators are components only.
After fixed Native6+Recovery7+corrected Verifier8+Runtime9 composition, add schema10
and private producer/consumer paths together: Workflow Round reservation, member
snapshot/actual adapter/Session/input, closure, aggregate publication and recovery.
Missing genuine policy/author/delta/instruction/schedule profiles yield explicit
readiness holds, never synthetic production capabilities. Trusted Human/controller
clearance and fix producers consume the real Runtime9 application authority;
unsupported operations hold rather than using a model or generic Record as Human.

Account-free controls must exercise real adapter stdio, durable receipt, snapshot,
Store and Workflow acceptance: N/all/quorum/any/STRICT floors, byte-identical core,
author/alias refusal, member binding and input-byte mutations, duplicate starts,
four workers/sibling cancellation, success versus failure APPROVE, potential Medium
veto, failed CAS rollback, quota live/terminal paths, cohort zero-partial permits,
schedule expiration, lifetime abort/handoff, source/retained graph drift, fresh
fix/full-round retry and current10/old9 writers. Strict bounds charge actual encoded
copies/native receipts before input against inherited retention budgets. Kill
compiled mutations at the actual admission/closure/final SQL consumers; parser
unit tests alone do not establish an execution or approval path.

No official authentication/settings/hooks, native channel qualification, two-OS
four-Task acceptance, ReviewRound completion, public MVP or final Phase2 is claimed
by this supplement. The current Native/Verifier qualification remains scoped to
its independently reviewed components, not transferred to future member glue.
