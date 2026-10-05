# Native Review Engine integration design

Risk: STRICT. Status: design proposal against source
`ed3c1078a0f5a9646a8c4991881179887b57b28e`; no production code changed or native
profile qualified. Normative requirements:
[Review integration](../requirements/review-engine-integration-requirements.md).
Keep [Agent execution design](agent-execution-design.md) result-protection,
fencing, snapshot and best-effort cleanup boundaries.

## 1. Scope and existing consumers

Add `review` orchestration plus typed Store/native/Workflow ports. ReviewEngine
owns Set/round/member/finding resolution. Workflow owns phase ordering and the
single active review-round claim; Runtime owns scheduling/cleanup, NativeSessions
owns native Session/turns, ResultStore owns retained input/snapshot provenance.
Goal consumes accepted Workflow gates; `goal.rs` is currently structural DAG
validation and is not a native launch or review certificate producer.

Replace reviewer branches selecting `reviewers.first()` in `WorkflowEngine::launch`
and `prepare_managed` with round delegation before any reviewer native effects.
Executor bootstrap adoption and subsequent fresh Executor paths stay separate.
Existing PhaseGates do not synthesize approval from a native terminal. For managed
review phases, a dedicated typed Review certificate port supplies evidence; generic
`GateOutcome::Passed(Evidence { review_approved: true })` cannot bypass it.
Unmanaged legacy integrations remain explicitly noncertifying under this profile.

There is no existing `WorkflowReservationGrant` type. Implemented
`execution::model::WorkflowReservation` is a CAS tuple used by
`Store::reserve_workflow_execution`; it can bind one unit only. This design adds a
private-field `ReviewMemberReservationGrant` and makes it a required, genuinely
produced extension of that authority, not a rename of caller metadata.

## 2. Policy activation and bundle production

`ReviewPolicy` freezes slots (unique keys, registered alias/provider/model/effort),
completion, quorum, parallelism, independent floor, security flag, blocking set,
author exclusion, predesignated clearance confirmers and time/budget limits.
Activate through the trusted application policy composition before first round,
with exact approved governing digest and origin. Task review config/rule changes
are candidate content until separately activated; target bytes cannot govern their
own approval. Capture approved Project/Runtime review rubric at the accepted
governing revision, distinct from the target if those rules are changed. Unsupported
conflicting native instruction auto-discovery holds before model input; do not
disable user rules/hooks or quietly inject the author's version as governing policy.

`ReviewBundleProducer` consumes private verified retained-artifact provenance,
actual immutable Task Context, activated rules, exact base/target trees and genuine
delta/check/attribution evidence. Returns a private non-deserializable
`FrozenReviewBundle`: exact scope/artifact snapshot, core bytes/digest, per-slot
additive specialization, dependency/coverage/exposure manifests and actual retained
charge. Store validates encoded bounds and each requested slot against this proof.
The committed index is a lexical selector, not bundle or authorship authority.
All required claims/rules survive discretionary selection; missing evidence yields
NeedsContext/Human hold. #20 may later provide this exact contract; a fixture producer
uses the same public composition boundary with explicit noncertifying qualification.
Bundle construction before round activation may inspect only the exact indexed
artifact through existing current-Runtime retained-reader admission: each bounded
Git read has its own durable intent before spawn and epoch/artifact receipt CAS.
It creates no member Unit/Session/input or model call. Member snapshot preparation
begins only after the round/member reservation below; generic unregistered Git
helpers and live executor-source fallback are unavailable to either producer.

Core consists of exact scope/target/base and retained manifest, Task acceptance,
applicable requirements/design/test references, activated rubric/instructions,
source map and exact evidence, prior-round finding manifest/required unresolved text,
attributed verification and fix claims. No raw transcript or peer/current-round result.
Each prepared frame wraps the identical core plus slot identity/focus and result
schema, with recorded digests and per-member prior-claim exposure. Specialization
cannot replace the core. Later rounds use exact before/after SHAs AND trees;
tree-identical commits are identity deltas, not changed-target retry permission.

Review policies ship all-of-two and 2-of-three configurations, plus Triple
Claude/Codex/Grok all-of-three and its explicit author-visible variant. Provider
readiness is separate from configuration parsing: current managed NativeSessions
supports Claude/Codex only; the legacy Grok adapter is not automatically a managed
member producer. No Triple execution availability is claimed until a reviewed Grok
member/result/read-only contract exists. Runtime permits may serialize members
only if genuine registered SERIALIZED_CAPABLE native-channel qualification covers
that exact roster/view and every reachable schedule under the frozen finite queue
policy. A CONCURRENT_ONLY profile never falls back to per-member serialized
admission. Equivalent core bytes or a profile label is not this qualification.
Freeze I9-AC-8.k's actual qualification identity, source/key scope, load/write timing,
permitted queue/start-skew/parallel/permit envelope and its producer evidence with
the policy. Absent genuine qualification refuses before member effects.

## 3. Persisted schema and invariants

Add schema/connection contract 6 after the current contract 5. This is a logical
schema plan; implementation must provide actual ordered migration and SQL guards.
The integrating owner assigns the next actual number from the combined source:
if another Phase2 change already consumes schema 6, rebase this migration as the
next ordered step, never reuse/skip its marker or overwrite another guard contract.
Migration allocation and the old-writer matrix must be reviewed against that final
combined immutable commit. No competing worktree may install a different schema
body under the same version. Schema5-to-next migration reserves no native effects.
Use indexed ownership columns with checked typed bodies, composite Scope foreign
keys, version CAS and transactional audit; do not store the whole round in one
mutable free-form Record. Existing immutable Context rows are referenced, not
incremented per slot. Table names below are proposed, not existing APIs.

| Table | Required identity/columns and constraints |
| --- | --- |
| `review_lineages` | id, exact Project/origin Task/review phase, version, charged/reserved bytes, admitted rounds, confirmation/tree and transient-cause counters; descendants jointly reference all inherited roots. |
| `review_sets` | id, Scope, Workflow Record/index/generation, version, state, activated policy/roster/governing digests, cumulative author manifest, security/floor, active round; partial UNIQUE one nonterminal Set per current Task/phase. |
| `review_set_roots` | (set, lineage) unique; immutable inherited budget/obligation membership. |
| `review_rounds` | id, set, sequence, version, epoch, state, artifact id/version/full snapshot digest, exact SHA/base/tree, launch Context version/digest, Workflow claim/version and accepted Project/Goal/task semantic digests, core/policy/roster hashes, qualified native-channel identity/envelope, original schedule origin, optional exact cohort lease, noncertifying schedule disposition, reserved charge, deadlines; UNIQUE(set, sequence), at most one active round. |
| `review_slots` | (round, slot key), immutable registered alias/provider/model/effort, eligible-author classification and focus/input/exposure hashes; UNIQUE slot key, no Session reused by slots. |
| `review_workers` | (round, slot), version, state, unique Unit id, unique Session id when present, generation/epoch, dispatch operation/receipt, wait reason/due, deadline clock fields, original qualified-schedule admission receipt; exact FK to slot and execution Unit Scope. |
| `review_inputs` | immutable round/slot complete bytes or exact durable owned content reference, byte count/hash, core/specialization/Context digests, coverage/exposure; no peer-output references. |
| `native_results` | immutable operation/Unit/Session/native turn receipt, provider profile/version, terminal work/disposition, answer acquisition hash/count/prefix/overflow/ambiguity fields and owned bounded answer bytes; exact member input digest when review-owned. |
| `review_results` | immutable round/slot/receipt, validated verdict or named invalid status, original typed envelope/content hash, measured model/effort or unavailable reason; one accepted receipt per invocation. |
| `review_findings`, `review_dispositions` | immutable original finding identity/text/hash/severity/locations/evidence and separate actor/target/inspection/disposition entries; never UPDATE original claims or verdict. |
| `review_certificates` | immutable round identity plus full artifact/core/policy/roster/current source digests, exact counted member receipts/eligible floor, all outcomes/exposures and resolution evidence; UNIQUE round. |

Structured findings/dispositions/results/inputs are append-only, enforced with
SQL triggers; mutable Set/round/worker updates require contract-6 connection guards.
Existing Workflow/current Task pointer adds optional `review_round` identity with
strict serde migration defaults; singular unit/agent/Session fields are absent for
new rounds. Review certificate refs extend typed Evidence rather than filling a
single Session ID with an arbitrary representative. Migrated legacy records retain
history and are not new certificates. Tests must cover open contract-5 writer failure
after migration and old binary reopen refusal before effects.

No worker callback needs a frozen aggregate Task.version to equal every sibling's
original version. Freeze Task semantic digest/Context and round currency; maintain
current Task/Workflow projection versions in the active round through exact Store
projection transactions. Member updates check those current indexed pins and only
their own worker CAS; they do not change shared Context or another worker's version.
Externally changing Task semantics/Project/Goal/generation invalidates the round.
This distinction must be encoded, not implemented as ignoring Task CAS wholesale.

## 4. Reserve and prepare transactions

`ReviewEngine::activate_round` calls `Store::reserve_review_round` in one Immediate
transaction, with actual Workflow phase reservation and FrozenReviewBundle proof.
Check current owner epoch, sole exact Workflow Record/version/active index, review
phase Running but undispatched, Task lifecycle/scope/projection/generation and
Context identity, Project/Goal versions/activated governing digest, complete
Published artifact snapshot/dependencies, roster/profile eligibility, author/floor,
full encoded frames and all inherited budget reservations. Persist Set/round/input
and roster workers, link active Workflow attempt to round, record audit and reserve
worst-case retained charges. No Git/version/native effect occurs before this commit.
Competing activations lose unique/CAS checks and create no detached effects.

For a queued worker, `Store::reserve_review_member` rechecks the active round and
frozen slot, current projection currency and quota/resource scheduling grant. It
allocates one Reviewer Unit plus durable path/port/temp/profile leases, binds it
to that worker and captures current execution generation. A returned private
`ReviewMemberReservationGrant` contains exact round/worker/slot/input/artifact/
Workflow pins and Unit identity. The new AttemptManager member-snapshot method
consumes this grant; it cannot call the old single-unit
`checked_workflow_binding` path or mutate Task executor worktree/branch. Paths are
fresh independent repositories populated by ResultStore, never linked to an
executor's current mutable Git state. A preparation abandonment guard fences only
this owned worker/Unit and preserves historical receipts/resources.
The grant is non-Clone, non-deserializable producer authority, obtained only after
the actual round/member reservation and verified bundle/snapshot composition.
Unit/Session rows or Evidence JSON alone cannot recreate it. Pending before-spawn
waits retain the live owner capability; after restart an explicit recovery producer
must requalify input/snapshot and create new authority or require a fresh round.

ResultSnapshot verifies retained identity, actual detached SHA, readonly tracked
source/admin data, distinct output and source manifest. Prepare helpers are recorded
under the member grant before each effect. After awaits, revalidate round/member/
artifact/currency; a stale guard cannot mark preparation ready or dispatch. Prepared
snapshot handle remains private provenance for pre-input and final validation.
Prior published artifact bytes are retained even if preparation fails.

## 5. Public adapter, native input and quota consumption

Extend `ManagedInput` with an explicit private managed authority variant:
ExecutorWorkflow versus ReviewMember. Public DTO fields alone cannot construct the
member grant. `AgentAdapter::start_managed` remains the actual vtable route; do not
introduce an engine-only shortcut while leaving a looser public reviewer route.
`NativeAdapter::start_managed` matches the variant and delegates member admission
to the exact Store check rather than comparing the singular PhaseAttempt.unit.
Role/kind/provider/registered alias/model/effort, request Project/path, snapshot
proof, frame bytes/hash and Context/artifact/policy pins must match. Generic start,
Executor/ApprovalReviewer roles or an ordinary Reviewer Unit cannot impersonate it.

Store Session registration and `admit_native_frame` independently validate the
durable member binding/currency and single dispatch winner before bootstrap/input.
Actual input admission also consumes current original-schedule qualification and
elapsed-envelope checks below; a pre-await capability check cannot substitute.
Same provider alias does not mean same slot. All public NativeSessions paths check
this authority variant; no Workflow-present caller can bypass it by directly
passing an arbitrary prepared Reviewer Unit or self-labelled ReviewBundle.
Claim `dispatch_pending` durably before start; all native version/setup helpers
carry its Unit/member scope. Register Session before native input. Persist input
intent before write and owned acknowledgement afterwards. Cancellation can win
before an intent; after intent, unknown acknowledgement is not permission to replay.
There is no SQL lock across version, quota, snapshot or protocol awaits.

Use existing NativeSessions/QuotaScheduler global/Project/provider caps and leases
only for schedules covered by the original frozen channel qualification. Each
queued worker's due claim and actual native input producer rechecks elapsed queue
time/start skew, current permits, scope and the original qualified channel identity.
`NativeAdapter` admission and `admit_native_frame` cannot refresh the schedule origin
or silently drop that check. Freeze a finite queue expiry compatible with the
profile; an unbounded queue cannot qualify a finite native-input window. A failed
check atomically fences further round input and marks the whole round noncertifying
with original qualification/time/permit evidence. Already admitted workers retain
their owned findings/results and complete logical closure; no prior approval is
relabelled or discarded. A new round requires the existing genuine retry authority.

For CONCURRENT_ONLY, add a private actual `ReviewCohortAdmission` producer and Store
transaction before allocating member preparation/native effects. Revalidate all
frozen slot/provider/profile/quota currency and compute the complete cohort's
global/Project/provider requirements. In one Immediate transaction acquire all
required scheduling permits/leases, or none; persist the exact cohort id, member
allocation and versions. A queued cohort has zero partial member scheduling leases
and a wholly unadmitted Set has no review worktree lock. Existing per-member quota
lease calls cannot mint this authority; until the genuine atomic producer and its
actual NativeAdapter/Session/input consumers exist, this profile refuses before
effects. Resource-path/retention bookkeeping does not pretend to reserve capacity.
Only successful whole-cohort reservation issues linked private member grants;
startup uses the frozen qualified skew window and admission before every model
input. Startup failure/expired skew makes the whole round noncertifying, preserving
already started workers and forbidding serialized fallback. It does not promise
all native startups succeed. Release individual scheduling allocations only on
their exact logical closure, keeping the queued cohort atomic on its next claim;
uncertain physical resources remain quarantined independently.

For a qualified serialized-capable per-member wait, update only the owning worker;
for a concurrent-only wait, update the whole zero-permit cohort atomically.
No Session/input has been created
on a before-spawn wait. Due recheck claims that exact current worker/wait record
with CAS, rechecks input/currency and uses the existing bounded probe policy.
WaitingQuota is not a failed/reject opinion. Accepted exhaustion/recovery live
telemetry preserves same Unit/Session/turn, native input bytes and worker invocation.
Task state projection is Reviewing if any review worker is active/runnable,
WaitingQuota only if all unresolved work is quota-waiting, and named capacity or
Human wait otherwise. A projection update atomically updates round's Task/Workflow
CAS pins; queued siblings stay valid. Notifications never edit a peer's result.

If native terminal interrupts for quota, persist work Unknown/QuotaInterrupted and
any acquired partial content, logically close the worker and round without a
certificate. After bounded quota recovery, a new full-roster round is authorized
under the recorded retry/lineage policy, gets new snapshots/Sessions and no carried
approvals. Quota interruption is a separate recoverable scheduling cause, not a
diagnostic failure resampling exemption; repeated identical no-recovery probes park
without burning rounds. Explicit Human/activated controller retry authority still
applies. A native willRetry=true continuation is not a new round and receives no
second prompt. Paid API or roster downgrade is never an automatic recovery.

## 6. Receive actual answers and validate results

Add provider-owned answer acquisition in NativeSessions; `NativeStatus.result`
becomes a projection of a durable typed receipt, not the only answer storage.
Codex collects exact owned `item/completed` agentMessage content for the current
thread/turn and joins only the declared supported final-answer shape, then binds
it to the owned `turn/completed` terminal. The baseline saves that terminal object
alone, which cannot be parsed as review text. Foreign thread/turn/item events,
tool output and previous answers cannot populate the slot. Multiple final answers
or uncertain association is invalid/ambiguous, not "take the last JSON".
Claude extracts supported final result text/structured_output according to its
versioned native schema from the owned result envelope; contradictory or multiple
candidate representations fail qualification. Do not parse arbitrary stderr/auth
logs. Add exact extraction producer controls for both real vtable paths.

At acquisition, enforce complete response/prefix limits before JSON allocation,
record retained prefix hash, observed length/at-least bound and unseen-suffix flag.
Persist result receipt atomically with owning Unit/Session known native terminal,
or durable draft diagnostic when its authority was fenced. Watch notifies only
after that transaction. Native work Success and cleanup Unknown remain independent.
Restart can consume this actual receipt; no opaque turn status means approval.

Review schema v1 uses strict duplicate-key rejecting decoding at every object,
bounded nesting/counts/strings and deny-unknown-field typed validation. Envelope:

```text
schema_version, scope, set_id, round_id, slot_key, execution_generation,
artifact_id, target_sha, core_hash, policy_hash, input_hash,
verdict(APPROVE|REQUEST_CHANGES|ESCALATE), findings[], prior_dispositions[]
```

Finding fields bind local unique ID, C/H/M/L, exact original claim, exact target
relative locations, cited evidence IDs and optional risk proposal. Disposition
fields bind original round/slot/finding/hash, exact inspected target/evidence and
enumerated resolution. The model cannot invent evidence references: validate each
against the captured input, its actual qualified expansions or separately recorded
inspection. Hash/count complete encoded entries (8192 bytes), not only claim text.
Allow at most 256 findings and complete 1 MiB envelope/frame ceilings concurrently;
maximum counts do not promise all maximum strings fit. Model echoes are compared
against the receipt's actual member authority, never accepted as that authority.

`Store::record_review_result` requires exact terminal receipt/version/worker CAS,
active round or historical diagnostic disposition, artifact/input currency and
validated result proof. A duplicate same receipt is idempotent; changed content
or a second receipt is rejected. Failed/missing/partial/overflow/malformed result
statuses are explicit, retained and non-approving. An APPROVE with a potential
Medium still creates a blocker. Requested vs reported model/effort mismatch fails;
unavailable actual values are named and require-verified policy may refuse.
Parsing well-formed APPROVE content from a failed native terminal records that
original opinion and findings as acquired content; it does not make the worker
successful or countable. Native work/disposition and content validity remain
separate fields through result persistence, member closure and final SQL counting.

## 7. Member logical closure and round aggregate

Replace current single-success `ReadonlyCompletion` coupling for members with
private `ReviewMemberCompletion` proofs produced from genuine snapshot/native
result provenance. Success requires actual readonly snapshot verification before
and after review and exact full retained artifact; other terminal dispositions
retain their work/result/unknown coverage, without fabricating snapshot success.
`Store::close_review_member` atomically records parsed/invalid/no-answer result,
immutable findings, worker logical terminal and finalization disposition, closes
owned native/finalization authority, releases exact scheduling lease/waiter and
schedules cleanup. Cancellation/failure/unknown paths use an explicit draft/unknown
disposition and quarantine uncertain physical resources. No Unit belonging to
another worker is closed. These independent closures do not complete Workflow.

Logical member closure means no further effects can be granted by the Runtime,
its owned input/result classification is durable, and draft/result retention is
decided. It does not mean all descendant processes died. Unexamined partial or
unknown result content holds certification/inspection, while best-effort cleanup
unknown alone does not. This is the explicit override of #9's custody-dependent
settlement language for the new profile. Native/session liveness, logical grant
closure, opinion validity and cleanup are separate fields.

After all roster slots logically close, runtime may reveal peer findings for
verification/aggregation. Default has no early stop. For each counted slot, require
the conjunction: exact owned current terminal WorkOutcome::Success with completed
noncancelled disposition, genuine verified readonly snapshot completion, validated
current owned receipt/input/envelope, successful logical worker closure and APPROVE.
Mode counting and independent-floor counting use that SAME predicate; floor adds
eligible non-author identity. Failure/Unknown/cancelled/timeout/Lost contributes
zero to both even when it emitted valid APPROVE content. Retain those findings and
inspection obligations; a potential Medium from a failed worker still prevents
certification until independently resolved. No successful content parse, cleanup
observation or row labelled Closed replaces genuine native/snapshot provenance.
Aggregate checks mode count
AND eligible independent floor, all required per-finding dispositions and independent
clearances, no potential/verified blocker, ESCALATE, dispute, unseen suffix or
unexamined content, current artifact/bundle/policy/rules. Diagnostic-only failures
may be tolerated by quorum/any; REQUEST_CHANGES and ESCALATE do not count.
All mode needs all approvals. No cleanup result changes these counts.

`ReviewEngine::completion` returns private `ReviewCertificateProof` only after
reverification of retained input and all current result/resolution snapshots.
`Store::publish_workflow_review` uses one Immediate transaction with full proof
comparisons: current epoch, Task lifecycle/current projection/semantic digest,
execution and Workflow generation, Project/Goal/governing versions, sole Workflow
Record/active phase/round/version, immutable launch Context and next consecutive
Context, full exact artifact snapshot, policy/roster/core/input hashes, each worker
closure/result and immutable resolution evidence. Recheck original qualified
schedule/cohort, certifying round disposition and the owned-success/readonly/valid
receipt/closed-success predicate for every counted member inside this transaction;
caller-provided counts, terminal row labels or result JSON cannot mint it.
Commit certificate + review
Evidence reference + Workflow phase success/next active state + Task projection
+ next Context + audit together. No representative Session stands for the set.
Cancellation/source drift racing publication has one winner. Failed CAS changes
nothing; retained results remain diagnostic/draft and cannot authorize a newer
round. Subsequent Goal/PR/merge consumers validate certificate/artifact/dependencies
and expected target under their own gates; a certificate is not merge permission.

## 8. Finding verification, fix and subsequent rounds

Record original finding and independent inspection as separate immutable objects.
Verification is a typed actor/target/evidence producer; no automatic supervisor LLM
attests a fact. Native verification gets its own read-only Unit/snapshot/Session;
Human actions enter an explicit trusted application port unavailable to Workflow,
Agent/IPC/Broker JSON. Library principal/origin cannot be relabelled CLI/Human;
same-UID direct machine invocation remains outside this application guarantee.

Clearance requires eligible non-author original finder in a later fresh Session,
predesignated agent/family-distinct pair excluding all recorded authors, or actual
trusted Human judgment; attach exact inspected locations/check evidence/disposition.
Disputed eligible evidence holds. A general APPROVE or new SHA is not a clearance.
Unchanged-target Human adjudication may clear a finding without changing original
verdict only under #9's narrow same-round branch; ESCALATE/dispute still needs a
new full round. Post-opinion policy relaxation/roster change requires trusted
activation retaining original obligations and mandatory floors.

Fix authorization references exact concern/inspection, bounded affected hunks and
author, then reserves a new Executor generation only after the review round is
logically closed/fenced. It uses existing fresh AttemptManager and ResultStore
commit capture/publication, never writes the readonly reviewer snapshot. The next
round reads that new Published artifact and genuine delta, reruns the full roster
with fresh Sessions and retains old outputs, author lineage and exposure. Review
findings and repair claims are separate until independent re-review resolves them.
No within-round retry or success-count carryover. Same-tree automatic confirmation
is at most once per lineage/tree on unused concern-linked evidence; diagnostic
failure/controller-contention retries share the existing two-retry lineage ceiling
and require qualifying changed facts. Quota recovery rounds additionally preserve
the same 64-round/byte budget and named scheduling-cause provenance, without treating
quota as review disagreement or clearing any old finding. Unsupported attribution/
delta/expansion producer holds rather than fabricating an exit.

## 9. Recovery, bounds and delivery order

On Runtime epoch recovery, fence old scheduling/native grants, mark ambiguous
dispatch/result acquisition explicitly unknown, preserve durable receipts/artifacts
and run best-effort cleanup. Do not reattach from PID/hints or consume old capability
objects. A completed durable receipt may be inspected as historical evidence;
resumed certification uses explicit current recovery authority revalidating every
pin, not an epoch number rewritten into the old round. Otherwise close without
certificate and use a fresh authorized full round. Known certificates remain
immutable; missing/corrupt retained objects invalidate use without overwriting work.

Before any input reserve full lineage charge: actual core/member/frame bytes,
per-slot 1 MiB result plus 64 KiB diagnostics and bounded expansion representation,
actual producer-declared row/receipt/usage overhead, fixed 8 MiB verification and
4 MiB control/attention/journal allowances, including 64 KiB overflow margin inside
the control allowance. Charge actual owned copies even with equal hashes; exact
external immutable references declare owner/bounds and availability, not free
retention. Preserve #9 expansion request 1–64/default8 and default1MiB bytes/slot,
shared 64 rounds/128MiB actual retained lineage quotas and applicable external
source-owner limits. Missing calculable native/profile overhead refuses that formal
shape. Evidence/row writes cannot exceed reserved space silently.

Opinion/startup/queue/deferred-capacity/logical-close clocks are recorded separately;
startup begins at actual permit acquisition, default60s/range1–3600s per #9.
Scheduling deadlines and opinion deadlines use frozen supported profile bounds,
not cleanup/death guarantees. Quota parked time is named, excluded from active
model timeout and still exposed as Task wall time; bounded rechecks/backoff and
cancel remain available. Quota parking never pauses or refreshes the original
qualified pre-input queue/start-skew window; expire that round if the original
native-channel envelope no longer permits a queued member's input.
Shutdown/cookie/Docker observations are independent
cleanup jobs; no physical settlement deadline is claimed for host-native hooks.

Implementation sequence after requirements/design independent approval:

1. Next assigned schema (currently9 after Native6, Published-frame recovery7 and Verifier8),
   immutable round/member ports, grant/CAS/fencing and migration controls.
2. Owned provider answer acquisition/receipt and strict parser, exact public
   NativeAdapter/NativeSessions/member Session/input consumers.
3. Production bundle/activation/attribution readiness and independent fresh snapshot
   preparation, parallel cap/quota scheduling and mixed wait projection.
4. Member logical closure, finding resolution and atomic collective Workflow
   certificate publication; fresh fix/delta/re-review and recovery.
5. Account-free actual consumer controls, independent committed source review,
   then separately authorized official CLI/native/two-OS conformance.

Each implementation milestone stays noncertifying until the whole actual path is
connected and qualified. Required mutation controls remove member binding checks
at adapter, Session/input admission and final SQL publication independently;
the corresponding bypass/foreign/stale consumer must then fail its assertion.
Remove owned answer extraction and exit-zero/missing-answer controls must fail;
remove potential Medium veto and 2-of-3 control must fail; remove identical core
or governing-policy authority and actual independent input controls must fail.
Current writer success/old writer refusal, concurrent duplicate starts, cancellation
versus certificate rollback, retained graph loss, four members with sibling cancel,
live quota/recovery and fresh round terminal quota tests are required. Synthetic
native peers are labelled orchestration evidence only.
Add I9-AC-8.k actual consumer controls: a genuinely qualified finite-window
serialized-capable profile admits its first worker, healthy contention delays the
second beyond the original window, and the actual second input producer emits
zero input while the whole round has zero certificate and retains first-worker
closure/evidence. A concurrent-only missing cohort capacity emits zero input and
holds zero partial member permits. Two two-member cohorts competing for capacity
three admit exactly one complete pair, queue the other with zero, and admit the
second pair after genuine returned capacity. A cohort startup failure cannot
convert the round to serialized/certifying. Remove queued elapsed-schedule checking
or replace atomic cohort acquisition with incremental partial holding; the
corresponding real producer control must fail. A synthetic clock/channel label is
not genuine native-profile qualification evidence.
Add failure-terminal result controls through actual native receipt/parse/aggregate
and publication consumers: syntactically valid exact APPROVE with native Failure
or Unknown contributes zero mode/floor votes, so all-of-two or STRICT's two-floor
cannot be satisfied by one real success plus that invocation. Failure containing
a potential Medium retains inspection/veto and cannot be hidden by quorum/any.
The restored Success plus verified readonly completion/current receipt/closed
worker is a separate positive. Mutations removing native-success checking from
aggregation AND final SQL counting must be caught at their actual consumers.
Actual credentials, models,
hooks/read-only enforcement and official profile independence are not tested by
this documentation delivery.

## 10. Human decisions and known limits

Approval is needed for these requirements/design before production changes.
The deliberate new-profile settlement change follows the user's latest result
protection/best-effort cleanup request, but #9 public closure wording remains
historical until separately reconciled; no external Issue was edited here.
Decide the initial formally supported native roster/profile list and explicit
governing activation path from actual qualification evidence; unavailable Triple,
4+/repeated-family and conflicting rule-edit native views remain named open gates.
No author-floor waiver, automatic paid billing, hidden credential reader, security
sandbox or complete process collection is proposed. External hooks/settings can
still change behavior or read same-user files; strict Runtime input independence
is narrower than OS isolation and is stated in every certificate/status.
