# Managed ReviewRound implementation and Binding10 / schema11 connection

Risk: STRICT. Status: corrected implementation supplement; documentation only.
No ReviewEngine, member owner, cohort admission, certificate, schema11 or native
profile availability is delivered. The prior candidate is immutable at
`63c3964c1354b385d117f18fa2b693c6f4c84e74`; this is its separately reviewed successor.
Normative [Review requirements](../requirements/review-engine-integration-requirements.md)
and [Review design](review-engine-integration-design.md) remain in force. The
approved [managed binding requirements](../requirements/issue-43-managed-binding-requirements.md)
and [Binding10 design](issue-43-managed-binding-design.md) are carried unchanged
from checkpoint `2e364bb792d23de365ad5e727356c874db9bf560` (design source
`cfd1800f8801fecfa47854fbaa0b08ec1f377985`). Their single-phase types are not round types.

Actual reference source `802764f03893d0ee8e08ee19045ccb333387ad9b` composes Native6,
SourceRecovery7 and Verifier8, schema8. Runtime9's bounded controller/empty Driver
milestone does not issue native Driver authority. Actual Binder10 and its retained
native Driver remain unimplemented dependencies. The agreed migration order is
**Verifier8 → actual Runtime9 → Binding10 → ReviewRound11**, with no skipped or
empty migration. This branch starts at63c and does not compose those production
sources; the reference above is a source-reading baseline, not this branch's API.

## 1. Actual consumer gaps and immutable authority

| Actual source consumer | Review11 required extension |
| --- | --- |
| `workflow.rs::prepare_managed`, Launched branch | Delegate reviewer phases before ordinary single-Unit reservation; activate one real round, with singular agent/unit/session/execution absent. Never use the first reviewer as the phase attester. |
| `state/execution.rs::checked_workflow_binding` | Its one empty Unit slot is not a sibling allocator. Separate member reservation consumes actual round/capacity authority. |
| `NativeAdapter::start_managed`, `NativeSessions::start` | Current Unit/ManagedInput is not the #19 owner/prepared input. Add a distinct private member branch through the same registered vtable and real Core, composing10 invariants. |
| `native_results.rs::bound_context/reserve_native_input` | Current Context payload equality is single-invocation behavior. The member branch checks unchanged shared launch Context plus exact registered per-member frame, retaining both proofs. |
| `native_results.rs::finish_native_result` | Actual owned receipt is necessary, but is not a member owner, factual binding or certificate. Extend the same terminal transaction to consume the member's exact allocation/admission. |
| `ResultSnapshot::completion` / readonly Workflow acceptance | One private successful snapshot proof is not collective completion. Produce member-scoped evidence; only the collective certificate consumer completes the review phase. |
| `QuotaScheduler` / quota SQL lease producer | Individual leases cannot create concurrent-only whole-roster authority. Add atomic cohort holds to the existing capacity authority before any member reservation. |
| `SourceRecovery7` / actual future Driver | Both require a separately reviewed round successor consumer anchored in their original saved claims, without generic after_write or current-pins refresh. |

Native6 proves protocol-owned acquisition, not the missing full private phase
protocol. Scalar Unit/Session/invocation/receipt IDs, model echoes, SQL rows, DTOs,
capability labels or a fresh hash cannot construct Review authority. Actor/claim
objects below have private fields, no public constructor/Deserialize and no Clone
ownership handle. Bounded diagnostic DTOs are content only.

| New private type | Actual producer / consumer |
| --- | --- |
| `ManagedReviewComposition` | Registered composition installation after actual9/10/11 writer, member vtable, retained Driver, bundle/profile and cohort consumers qualify; Workflow/Driver preflight only. |
| `FrozenReviewBundle` | Actual retained-artifact/bundle/activated-rule/attribution producer; round planner consumes it, never a Task-authored configuration. |
| `ReviewRoundOriginalMarker` | Private round-marker transaction; immutable complete original owner/Workflow/Context/lock frame and allocation manifest. |
| `ReviewCapacityAdmission` | Existing quota authority's actual all-or-nothing cohort transaction or qualified serialized single-slot transaction; required by member reservation. |
| `ReviewMemberAllocation` | Actual member reservation transaction activates the round’s fixed slot/Session/planned Unit/native origin/input only after capacity admission; not a validated Session or delivered input. |
| `ReviewMemberOriginalFrame` | Same member Unit registration transaction seals original round pins plus actual Unit/generation/profile/path before its first helper; no recaptured parent/source authority. |
| `ReviewMemberReservationGrant` | Private member Unit/profile/resource transaction after capacity admission; actual snapshot preparer only. |
| `PreparedReviewMember` | Actual pre-qualified readonly snapshot and registered member-frame producer; Native member launch consumes it. |
| `ReviewMemberLaunch` | Retained round/member supervisor handoff of allocation, grant, prepared input and original marker; actual registered vtable consumes once. |
| `ValidatedReviewSession` | Actual Native registration transaction checks the same prepared allocation; normal factual member binder. |
| `ConsumedReviewInput` | Actual one-shot before-wire member input transaction; exact recorded intent, not atomic OS delivery. |
| `OwnedReviewSettlement` | Actual owned Native terminal transaction with correlated consumed/ACK input; late member binder, not review approval. |
| `ReviewRoundSuccessor` | Store-owned coherent bounded round/Workflow chain planner; specifically enumerated currency consumers, not native grant by itself. |
| `ReviewMemberCompletion` | Actual owned receipt plus parsed-content status and actual snapshot provenance; sole logical member closure. |
| `ReviewCertificateProof` | Actual collective retained-graph/current member/disposition re-verifier; sole atomic Workflow review acceptance. |

None is an alias, wrapper constructor from DTO, conversion from Single PhaseOwner,
or representative Session for a group. Binding10's single-phase binder rejects a
review_round attempt. Review11's member binder rejects single-phase operations.
Joint independent design/source review covers the new extension, including every
shared Native/Store/Source/Driver consumer; design approval alone does not enable it.

## 2. Preflight, capacity, marker and member preparation order

1. Actual Driver/Workflow checks ManagedReviewComposition before hold clearance,
   fresh phase Context/reservation, source preparation, marker, member helper or
   spawn. Qualified provider list, vtable origin, native instruction/read-only
   profile, accepted policy, genuine attribution/bundle and retained Driver are
   implementation-owned prerequisites. Missing ports yield named pre-effect holds.
   Public start/standalone Native6 cannot bypass a protected Workflow/member scope.
2. A coherent off-Store planner resolves current accepted9 Driver claim, immutable
   retained artifact and complete source/Context/lock frame. Bundle inspection uses
   separately registered current-Runtime retained-read intents before each bounded
   Git read, as approved R5. It is not member preparation, grants no member Unit,
   Session/input or round worktree lock, and never reads the live executor tree.
3. Validate original qualified channel envelope and all inherited lineage charges.
   For CONCURRENT_ONLY, invoke the actual whole-roster capacity producer **before**
   reserve_review_round/member, allocation handles, Unit/grant, snapshot materialization,
   member helper, round worktree lock or native input. An unavailable/queued cohort
   returns only bounded queue/attention metadata: zero member allocations/grants,
   zero Units/resource or partial scheduling leases, zero member helper/input intents
   and no round WorktreeLock/dispatch marker. No preparation followed by later
   capacity refusal is allowed. Prior independent bundle reads stay separately
   classified, never counted as member preparation or capacity acceptance.
4. For CONCURRENT_ONLY, only after successful complete capacity admission does
   reserve_review_round consume that admission and exact FrozenReviewBundle/Driver
   claim. SERIALIZED_CAPABLE may register its immutable unstarted round under the
   qualified schedule, but issues no live member allocation/grant before that
   slot’s actual capacity admission. Under
   Immediate, recheck full owners, sole Workflow/active phase, Context, full locks,
   artifact, policy/roster/author floor, original schedule, capacity identity/version
   and complete retention charge. Write Set/Round/Slots/Inputs/Workers, the unbound
   round marker and immutable allocations, Workflow review_round reference and
   ONE private marker audit atomically. Durable allocation IDs are the frozen plan,
   not returned live member handles. Reserve the Task execution generation once,
   with an active_review_round pointer rather than a representative active_unit;
   all members use that same generation. Queue capacity is retained before this
   marker. No helper/child/wire runs inside Store. Capacity commit followed by
   marker failure is conservatively released only by actual no-effect disposition;
   uncertain commit is read-only reconciled, never copied into a new round.
5. Under the existing short Owner resource-admission gate, draft one member's
   fresh paths/ports/temp/profile, then reserve_review_member consumes its exact
   current capacity/allocation/round/frame and inserts Reviewer Unit/resource leases.
   This transaction also activates its private allocation and seals
   ReviewMemberOriginalFrame: the unchanged original round P/G/T/W/Context/lock
   pins plus the newly registered exact Unit/generation/profile/path. Profile/path
   planning is off Store; exact draft/lease CAS is checked inside. The original
   round frame is not changed, and no current parent/source values replace it.
   The private non-Clone grant and lifetime guard are produced only at commit.
   Release the resource gate before materialization, filesystem, Git or other await.
   Simultaneous member preparation must retain actual same-Runtime port uniqueness.
6. Actual AttemptManager member snapshot consumes that grant, makes fresh immutable
   input and distinct output namespaces from the exact retained artifact, and
   records every preparation helper under its Unit/slot before effects. Post-await
   checks validate original pins, Unit generation, snapshot/profile and frame bytes.
   Mark prepared only with private actual pre-qualification; no ordinary snapshot
   or readonly boolean can be adopted. Task executor worktree/branch stays unchanged.
7. The independently retained member supervisor receives ReviewMemberLaunch and
   the armed guard before adapter start. It owns start/result reconciliation across
   Engine timeout/abort/drop and factual binder failure. Only this actual vtable
   consumes the same launch, with exact registered alias/provider/model/effort/role.
   Native validates it before readonly Git/version/start helpers and again at actual
   registration/input, using original currency and capacity, never another UUID.

A qualified SERIALIZED_CAPABLE profile may reserve an unstarted round, with no
member authority for queued slots. Before **each** member reservation/grant/preparation,
its actual single-slot ReviewCapacityAdmission must win under the same quota SQL
producer and original elapsed envelope. This is the only permitted serial shape;
CONCURRENT_ONLY cannot become serialized after wait/startup failure. Unqualified
schedules refuse before member effects. Current Grok has no genuine managed member
port: Triple and any roster needing Grok stay Unsupported, until a distinct actual
Grok producer/input/result/read-only/profile extension is jointly reviewed and
qualified. Fake/legacy Grok capability or fabricated receipt cannot count as positive.

## 3. Existing quota authority and complete cohort producer

Extend the existing global/Project/provider/account quota admission; do not add a
second semaphore or parallel independent capacity ledger. New indexed
`review_capacity_holds` represent not-yet-materialized member allocations within
that same authority. They are needed because existing quota_leases have a Unit FK,
while a queued cohort must create no member Unit before successful admission.

`admit_review_cohort` rechecks the whole original qualification/roster/policy,
current pools/caps/cooldown/probe versions, actual per-Project distinct-Task capacity
and all active leases **plus untransferred capacity holds** under one Immediate
transaction. Compute complete global/Project/provider requirements using real
providers/accounts, not aliases. Either reserve every slot hold and immutable cohort
identity/member manifest, or commit none. A changed fact/insufficient bucket only
records bounded wait metadata outside grant authority. Two two-member cohorts at
capacity3 admit one complete pair and the other zero.

The private admission identifies actual retained scheduler owner and all exact hold
IDs/versions/slot requirements, schedule origin and expiry. A row alone cannot
reconstruct it after Drop/restart. `reserve_review_member` refuses before its first
Unit insertion unless that actual admission remains current and unused for this
slot. It atomically transfers the slot's hold into the existing Unit-bound quota
lease and binds Unit/slot, marking the hold transferred with exact CAS. Effective
capacity counts the lease OR pending hold, never both and never neither across the
transaction. All legacy/executor/reviewer/approval admission, waiter/due, pool
status, release, restart and effective-cap consumers must include holds; a producer
which ignores them prevents ReviewComposition installation. No double native
reservation and no capacity leak on partial prep/Drop is accepted.

Runtime's Project Task limit counts union of actual driving claims/admitted leases/
pending holds by distinct Task, not one Task per member. Per-native permits count
every actual slot. Actual9's admission must expose these precise union/current SQL
consumers; a read-only Goal controller is insufficient. Resource path/port locks,
retention reservation and capacity admission are distinct resources.

Native startup keeps the original cohort/skew window. Unstarted slot failure or
expiry makes the whole round noncertifying and fences later member inputs; already
started members retain owned evidence and close logically. It does not guarantee
physical startup or release live permits on timeout alone. Pending holds return
only through exact private no-effect/cancellation/revocation disposal; transferred
leases follow each worker's actual logical closure. Unknown storage/start remains
Held and visible. Guard Drop attempts conservative closure, records/enqueues exact
owned disposition, and cannot promise SQLite/OS availability or recycle uncertain
resources. No round may retain partial holds as a queued concurrent cohort.

## 4. Full original frame and distinct round/member factual binding

ReviewRoundOriginalMarker is the immutable **post-marker** complete P/G/T versions
and encoded body digests; exact Workflow Record/version/body and active index/phase/
generation; state-root instance/epoch; accepted Driver original claim/transition;
shared launch/latest Context full bodies/source maps/revision/governing/instruction;
full retained artifact/base/target/tree/dependency snapshot; complete scoped
WorktreeLock ID/version/body-digest set including inactive; activated policy,
cumulative authors, roster/core/focus/member input and channel/schedule identities;
and complete immutable per-slot allocation manifest. Complete lock/identity query
bounds reject overflow rather than truncate an absence proof.

The round manifest fixes unique planned Unit IDs, shared execution generation,
Session IDs, slots, registered vtable/native origins, required namespace/profile
constraints and exact input identities. The manifest is not snapshot provenance.
After capacity wins, actual member Unit registration seals the exact newly allocated
profile/path in its separate immutable ReviewMemberOriginalFrame, referencing all
unchanged round-original pins. This is the authorized member allocation producer,
not a replacement of previously captured Unit/frame fields. Resource-dependent
ports/cookies must not be guessed in a prior marker and refreshed later. Before
Native starts, the frame already includes the exact actual Unit/profile/path and
actual prepared snapshot pair; only real registration makes ValidatedReviewSession.
Members use independent Unit/resource-owned readonly namespaces, not a new legacy
Task-scope WorktreeLock that would invalidate the captured complete lock set. Neither a
later current read, sibling wait, returned NativeStart DTO nor member terminal can
replace original pins. Marker body/hash planning is off SharedStore using a coherent
read-only connection to the retained canonical database; publication compares exact
preplanned old/new encoded bytes and versions under Immediate, with no hashing,
filesystem, process or native call while held.

There is no representative PhaseAttempt Session/Unit/execution. Round-member
factual binding has a separate sole private `bind_review_member` port. It writes
only the member's originally absent Session/invocation/execution association in
`review_member_units` and ONE existing reserved member-bound audit/link in one
Immediate transaction. It does **not** update Task, P/G, Workflow, Context, Session,
locks, input/capacity/owner/successor/Driver rows. Execution identity is derived from
actual immutable allocation/Unit/Session/invocation, never caller metadata. Native
registration writes its own ready/Session facts first; binder records correlation
without revising them. No hidden ordinary Workflow persistence accompanies it.

Before first member binding, original Workflow marker version/body must still be
exact; no intervening Workflow diagnostic/gate bookkeeping is accepted. Sibling
member-only native telemetry/bindings/closures do not mutate that Workflow. Under
Immediate check current complete original P/G/T/Context/locks, exact round/slot/
Unit/current epoch and its original actual member frame/allocation/preparation pair, current latest own
Session identity/alias/provider/role/prepared path/model/effort/vtable origin,
returned native_ref when Some, and complete scoped provider/native UUID ambiguity
including Lost/history/non-indexable identities. Legitimate PID/native UUID/lifecycle
advancement is not a stale returned-Session version CAS.

Normal binding accepts genuine prepared/validated registration before input/ACK.
Late binding additionally requires the same actual consumed/ACK-correlated input,
current owned successful terminal and logical terminal/input settlement under the
original frame/epoch. Terminal winning the normal-return race forces this late
predicate; a prepared handle cannot bypass known Failure/Unknown. Cleanup Unknown/
Leftovers alone neither grants nor blocks eligible factual binding. NotDispatched,
HistoricalDraft, ambiguous input/external outcome, stale epoch and unknown work
remain noncertifying holds. NativeTerminal/receipt IDs cannot mint the proof.
Known no-dispatch/failure uses separately proved nongrant closure, with an unbound
association remaining absent. This preserves Binding10 MB3/MB4/MB7 rather than
turning a terminal label into allocation.

AlreadyBound requires this same operation/slot/pair, exact committed reserved link
and tuple, plus complete genuine current round successor. It performs no write,
audit/input. Transient rollback defers the same actual invocation; uncertain commit
reads exact indexed association/link before retry. Schema/proof/bounds errors hold.
The supervisor retains actual start/Core/private terminal proof across lost/full
wake and caller Drop; binding error cannot retire a launched owner, fail the Task,
release its live reservation, refresh pins or start another member. Retained real
Driver provides bounded fair paging/timer fallback and durable level readiness
before/with logical settlement, rechecked on registration/start end/error/drop/
cancel. Passive status cannot bind or reconstruct a supervisor.

## 5. Shared Context, native input and current successor consumers

All members reference the same immutable launch Context/version/full source map
and unchanged shared factual core. Every registered member frame records identical
core bytes plus separately hashed additive focus/slot/schema envelope and its exact
complete outgoing bytes/hash. No Context increment per member. Native Core retains
actual current Context and member payload as separate correlated pins.

`register_review_session` and `reserve_review_native_input` are new distinct private
branches of the actual Native/Store producer, through the registered adapter vtable.
They check the ReviewMemberLaunch/allocation/prepared pair/actual snapshot, full
original frame and original current capacity/schedule. They must not simply delete
`bound_context` payload equality or pass a changed public ManagedInput to the old
single branch. Validate latest Task Context pointer/full shared Context against
OriginalMarker, exact member frame from immutable review_inputs, its bundle proof
and actual owned wire bytes independently. NativeInvocation binds both digests,
round/slot/Unit/Session and selected profile; model echoes confer no authority.

The actual before-wire transaction records one unique intent/effect and immutable
consumed pair plus provider thread/turn/frame metadata. ACK/transport/permission/
terminal journals cannot overwrite input. Cancellation is ordered at this same
private admission; partial/lost wire/ACK remains uncertain and never authorizes a
second prompt. Due/wait resume claims exact current worker/capacity row/version,
rechecks original elapsed queue/skew envelope and sends no duplicate input. ALLOW/
delegation requires original currency and real own Session; DENY/revocation stays
nongrant. Marked member Units refuse generic start, standalone Session/input, Unit
adoption, single Workflow native/readonly publication and public DTO bypasses.

**Live status adaptation:** the older Review design's mutable Task/Workflow wait
projection is implemented here as a coherent *derived* Task projection, preserving
R6 and Binding10's exact original P/G/T pins. A member wait/closure transaction
writes only own worker/Unit/quota/readiness and bounded round status facts; stored
Task stays at its original Reviewing marker/version and Workflow stays original
through live member work. Effective Workflow/Goal/Runtime/CLI Task status is
Reviewing if any unresolved member is active/runnable, WaitingQuota only if every
unresolved member is quota-parked, otherwise the exact named hold. It must be shown
and used by actual scheduling/status consumers, not cosmetic text concealing an
ordinary Task update. This replaces63c's physical open-phase Task/W projection
hook; it does not ignore Task CAS or relax the approved mixed-member policy.
Only separately authorized round terminal/collective closure may publish enumerated
Task/Context/Workflow deltas. Sibling observations do not invalidate another member
or update its original input, result, worker version or native proof.

A distinct bounded `ReviewRoundSuccessor` recognizer validates immutable member
binding/closure facts and narrowly enumerated round control/terminal events,
anchored to the original round marker. Member events do not use or counterfeit
Binding10's single-phase session_bound link. Single-phase256-link class allowances
remain unchanged. Review11 member ledger has separately reserved bounded classes
(§6), and any Workflow changes use only jointly reviewed round-specific terminal/
collective closure ports with full old/new projection and ONE reserved Workflow
link. No native_diagnostic/gate bookkeeping may precede first member binding;
prebind telemetry lives only in actual readiness/Session/worker facts.

Source7 validate_task/validate_binding/recovered_source_task and actual inputs/
prepare_pack/registered Git consumers and actual9 Driver must consume the new
sealed round successor from the same coherent off-lock planner. Anchor at their
saved original claim, using the actual separately authorized pre/post round-marker
receipt if claim predates it; never substitute a current snapshot or invoke generic
after_write. Every other saved source/owner/Context/artifact pin stays exact during
live work. Collective closure consumes genuine chain and exact current CAS in its
own source/Driver publication transaction; binding cannot refresh those rows.
If these actual new consumers cannot be exposed, ManagedReviewComposition stays
absent. A generic round row or same-version hash cannot establish currency.

## 6. Schema11, sole writers, ledgers and finite costs

Use the approved indexed Set/Round/Slot/Worker/Input/Result/Finding/Disposition/
Certificate/Lineage tables, plus the following explicitly distinct ports/tables.
Final DDL/body/index/trigger definitions are part of the composed source review,
not optional glue delegated to a caller. Every row has exact Scope/FKs and checked
positive version≤i64::MAX; bounded body/index equality is checked before authority.

| Table / surface | Required binding and writer |
| --- | --- |
| `workflow_review_contracts` | Immutable activation from actual accepted policy/composition at Workflow initialization, before any Unit/Round; all success writers protected even with no Unit. Legacy rows migrate Held, not to new owners. |
| `review_round_operations` | One immutable original marker/operation per Workflow generation/index/round; complete allocated roster, source/frame/origin. Private marker only. |
| `review_member_allocations` | Unique(round,slot), planned Unit/Session IDs unique, original selected native origin/input constraints; actual member producer activates its nonserializable handle after capacity. Immutable actual member frame binds Unit/profile/path; cannot bind until real preparation/registration. |
| `review_member_units` | Exact round/slot/actual Unit FK once, same scope/generation and member-frame reference; registered Session/invocation associations absent until sole private member binder, UNIQUE Unit/Session/invocation. No sibling/representative reuse. |
| `task_execution.active_review_round` | Actual round marker allocates generation once; mutually exclusive active_unit/active_review_round. Per-member register/close/drop never increments or clears the whole Task generation/pointer. Only genuine round closure/cancel owns that delta. |
| `review_member_inputs/admissions` | Immutable member frame/pair; unique one-shot original intent/effect/invocation and exact shared Context/native wire digest, ACK facts separated. Native actual private producer only. |
| `review_capacity_holds` | Existing quota authority's pending all-or-none or serialized capacity, exact cohort/slot/provider/account/scope/rank/expiry/version, transfers once to Unit lease. Not a second capacity authority. |
| `review_cohort_admissions` | Immutable original qualified complete member allocation/capacity manifest and schedule; actual producer provenance, no public mint. Pending queue metadata cannot claim it. |
| `review_member_readiness/status` | Bounded durable factual level/derived effective state, no owner construction or appended per-wake grant journal. |
| Existing audit / review ledger links | One immutable audit row per actual member/round fact, unique operation/slot/class/ordinal/predecessor and hashes. No mirrored audit+ledger duplicate. Sole typed ports only. |
| `review_member_completions` | Immutable actual receipt/input/native work/content/snapshot/closure binding; genuine private closure only. |

All supported generic Record/Session/Workflow/Recovery and raw protected writers
reject member association changes and protected round/input/allocation/certificate
writes, including existing-key INSERT/REPLACE/UPSERT under recursive_triggers OFF.
Public audit/audit_if_current refuse every reserved private review kind. Exact
connection-local scoped write permits exist only inside the producing transaction,
reset on every error/rollback; no callback, nested DB/IO or reusable RecordOnly flag.
Existing-attempt singular Session/execution remain exclusively Binding10's writer;
Review11 never writes them. Existing generic Unit retire/cancel/adopt/finalization
paths must detect marked members: actual member guard/closure fences only its own
Unit/worker, without fence_task_tx or changing shared Task generation. Trusted whole
Task/round revocation fences all members through its separately authorized port.
A generic retirement helper cannot accidentally cancel sibling reviewers. Generic Passed, no-Unit evidence, old single Readonly,
terminal receipt and caller counts cannot complete a protected review phase.

Per round reserve before effects a separately bounded member/round ledger: at most
32 member_bound links (one/slot),32 member_closed links, one round_started,
one round_noncertifying, one round_closed and one certificate. Worker wait/due/live
quota/readiness are bounded current facts with exact own CAS, not new ledger links;
repeat delivery is no-write. Round cancellation records at most one nongrant
round_noncertifying and one closure, not32 Workflow session bindings. Each link
complete≤4096 bytes; member/round operations use immutable exact original references.
Finding/result/disposition/expansion events have their approved separately measured
lineage charges and bounded records, not this ledger's spare authority. This
68-link maximum is distinct from inherited single-phase256 links, never spends
another operation's mandatory cancel/closure budget. Atomic publication can coalesce
round_closed+certificate as two distinct reserved facts with one SQL transaction;
no duplicated public generic audit. Maximum coexistence is counted, not assumed.

Preserve shared64-round/128MiB actual lineage bounds and8MiB verification/4MiB
control allowances, including64KiB overflow margin. Charge actual owned copies:
core/member/prepared frame≤1MiB each, complete acquired Review envelope≤1MiB,
Native6 complete receipt≤2MiB, diagnostics≤64KiB, complete findings≤8192 bytes/256
entries, prior-dispositions/expansions and all tables/index/audit/owner/frame copies.
Equal hashes do not make copies free; external exact references declare owner/bounds
and availability. Unsupported calculable provider overhead refuses before input.
All new binding/marker/ledger retention is included before capacity/round activation;
unused reserve releases only on genuine logical round closure, spent stays charged.

Plan/decode/hash/chain outside SharedStore. Explicit complete transaction surfaces:
one P/G/T≤8MiB each (current actual Goal remains4MiB); one W≤8MiB including OLD/NEW
write/CHECK; at most two shared Context≤8MiB each; each actually participating
member own Session≤4MiB/recovery≤3MiB depth32/nodes32768; operation≤4MiB and frozen
settlement≤64KiB; prepared/allocation≤2MiB and consumed admission≤8192 bytes;
complete scoped locks≤256×16384=4MiB; complete scoped identity negative query≤4096
bounded metadata rows with malformed flags and conservative overflow refusal.
Collective reads cap32 member records/receipts/completions plus approved disposition/
findings inventory at its separate complete count/byte ceilings. Compact proof
size is not total SQL/JSON/body/CHECK cost; measure repeated reads/writes too. All
simultaneous maxima need not fit128MiB; admission rejects profiles exceeding charge.
Finite cost is not measured latency/fairness or a physical cleanup deadline.

Fresh11 and ordered10→11 install only after actual8→9→10. Namespace/version/
marker/guard migration is one rollback-safe transaction; add all new mutable tables
and reinstall every existing writer contract to11. Cached10 INSERT/UPDATE/DELETE,
old10 reopen and current11 future-schema reopen must refuse. No rows are upgraded
to live owners/approval by migration; legacy evidence stays historical.

## 7. Actual answers, logical closure and collective acceptance

Core keeps real member launch/Unit/Session/provider turn and actual final answer
collector. Extend actual Native terminal TX to consume exact member input/binding,
retaining original known work before fallible stream/parse/snapshot retention. Current
owned receipt closes native effects, records Session/quota/result atomically; stale
callback retains HistoricalDraft and zero votes. Text/exit0/EOF/native turn object is
not an opinion. Claude supported final extraction and Codex owned agentMessage must
follow their actual versioned protocols; contradictory/multiple answers fail.

Strict duplicate-key/bounded `strict_json::decode` plus deny-unknown typed Review
schema validates full actual acquired envelope and exact scope/Set/round/slot/
generation/artifact/SHA/core/policy/input. Locations/evidence/required dispositions
must join the actual frozen/qualified expansion/inspection manifest. Model echoes
cannot supply ownership. Missing/partial/overflow/unseen suffix/ambiguous content
retains bounded diagnostic and inspection obligations; no tolerated empty success.
A valid APPROVE on Native Failure/Unknown preserves original opinion/findings but
counts zero for both mode and independent floor. Potential Medium still vetoes.

A successful member closure additionally requires its exact factual association
AlreadyBound from the sole member binder. Known native Success may remain durable
with pending/deferred association and open Runtime-only finalization; retain that
actual terminal proof while binding retries, with no new native input. Do not count
or issue successful member completion from a merely registered Session. Failure/
Unknown/no-dispatch nongrant closure may keep association absent; a later callback
cannot upgrade that closed noncertifying member to Success.

The member lifetime guard remains owned through actual successful before/after
readonly snapshot provenance, parsed/invalid result classification and private
`close_review_member` handoff. Native terminal alone does not disarm it. Closure
stores immutable actual completion/result/findings, fences exact worker effects/
finalization, releases exact capacity wait/lease and schedules best-effort cleanup
atomically. Failure/cancel/unknown closure preserves work/content/unknown and uses
noncertifying disposition without fabricated readonly Success. Retention fault and
future abort controls must preserve known observations and close/enqueue unknown
owned grants. Uncertain resources remain quarantined; logical closure is not death.

After all roster members logically close, reveal findings for independent inspection.
No early stop. Closed member evidence is immutable; collective re-verification uses
actual current Runtime retained-read intents, never UnitGit/finalization on a closed
Unit or a newly manufactured ResultSnapshot::completion. Crash before private
completion is unknown/noncertifying, not a replay by epoch rewrite.

`publish_workflow_review` consumes actual ReviewCertificateProof and rechecks the
same conjunction inside Immediate for every counted member: current owned native
Success with noncancelled completed disposition, genuine readonly provenance,
exact valid receipt/input/envelope, genuine committed allocation/association chain,
successful logical closure and APPROVE. Floor
adds eligible non-author identity to this same predicate. Check all/quorum/any AND
QUICK/STANDARD floor1 or STRICT floor2, all original policy/roster/author/lineage,
qualified original schedule/cohort, every finding/disposition/blocker/exposure,
current full retained artifact and original owners/round/Workflow/Context/source/
Driver chain. No caller-provided aggregate, boolean or terminal row replaces it.
Write certificate, typed multi-member Evidence ref, review phase success, enumerated
Task projection, next consecutive Context, source/Driver closure and reserved audit
atomically; failed CAS changes nothing. Neither representative Session nor one
member completion is collective authority. Cleanup alone changes no vote/work.

Preserve1–32/all/quorum(N)/any, unique roster/default author exclusion, cumulative
Runtime-recorded delta authors, explicit allow-self without independent-floor or
clearance eligibility, STRICT any's two-floor and Triple all-of-three. Unknown
attribution/rule activation/native profile remains held. Unsupported Triple never
silently becomes Claude/Codex. Potential/verified C/H/M, ESCALATE/dispute/unexamined
content blocks every mode; Low may additionally block. Diagnostic-only failure may
be tolerated only after genuine inspection; REQUEST_CHANGES never becomes APPROVE
because findings were dismissed. Original finding and independent clearance remain
separate immutable evidence under existing #9 Human/finder/predesignated pair rules.

Fresh fix uses actual Executor/artifact/delta/author after logical round fencing,
then fresh full-roster snapshots/Sessions with inherited roots/obligations/bytes.
No carried approvals, within-round relaunch or repeated unchanged-tree resampling.
Live quota willRetry keeps same invocation/input; terminal quota interruption retains
Unknown/content and closes noncertifying, then requires actual changed recovery and
fresh authorized full round. Preserve one same-tree confirmation and two transient
retry ceilings as applicable, shared64-round budgets and original schedule expiry.
Restart fences oldepoch; rows/receipts/UUID/PID cannot recreate member/Driver owners.
Only distinct actual reviewed recovery can inspect current known evidence for
certification; otherwise historical diagnosis/fresh authorized round, never rewritten
old epoch. Binding10's across-epoch factual-only restore remains its separate gate.

## 8. Writer/consumer replacement and required controls

| Actual route | Required private boundary |
| --- | --- |
| Workflow initialization/native preparation | ManagedReviewComposition + activation before effects; no representative reviewer/single-Unit reservation. |
| Actual quota pools/waiters/releases/status/Driver limits | Include pending holds in existing capacity authority; whole cohort before any member Unit/grant/helper; exact transfer to leases. |
| Unit resource/snapshot/helper admission | Actual capacity+allocation+grant before registration/materialization; same short resource gate and off-lock preparation. |
| Public adapter/Native start/Session/input | Distinct private member vtable; full original frame/pair, prepared namespace and one-shot actual wire admission. |
| ACK/ALLOW/quota/terminal collector | Own member identity and original current input; no sibling overwrites; actual Native owned settlement. |
| Normal/late factual member binding | Sole member-only association+one audit, same pair/normal versus consumed-success predicates; retained readiness/supervisor. |
| Gate observation/worker wait/diagnostic | Bound current own factual rows; derived coherent status, no hidden Task/W/Context/pin refresh. |
| Source7/Driver/prepare_pack/retained Git | Actual anchored sealed round successor; no generic after_write/current authority recapture. |
| Single readonly/generic Passed/Record/recovery/raw writers | Protected managed-round success/association delta refuses even with no Unit; legacy remains noncertifying. |
| Result parse/member closure/aggregate/fix/restart | Actual owned proof, lifetime handoff and exact independent certificate CAS; no text/rows/Fake authority. |

Account-free controls must use actual registered adapter stdio/Core/Store/Workflow
and genuine producers before any mutation assertion. Required additions to approved
R1–R7 matrix:

- CONCURRENT_ONLY saturated valid cohort: member preparation call yields zero
  allocation handles/Units/leases/locks/helper intents/input, not only zero prompts.
  Restore capacity and reach the real whole-cohort positive; two pairs/capacity3
  demonstrate one pair/other zero and later progress. Mutation omitting actual
  capacity check in reserve_review_member must fail at intended preparation consumer.
- Real allocated/shared-Context/member payload/Unit namespace: normal preinput/preACK
  bind once and actual callback remains current; foreign/changed pair/vtable/frame/
  original owners/locks/returned UUID refuse both binder and prewire. Independently
  mutate Native member producer, Session/input consumer, sole writer and final SQL.
- Two or four genuine slots bind/close/wait concurrently with unchanged original
  Task/W/Context/Source/Driver pins; coherent mixed/all-quota status, sibling cancel
  and no representative Session. Generic writer and same-version/REPLACE guards
  must catch actual bypasses without disabling current positive fixtures.
- Deferred first save, Engine abort/drop and lost/full wake retain exact actual
  supervisor/Core and known result; normal/late race gives one association/audit;
  NotDispatched/Failure/Unknown/Draft/stale epoch cannot late-bind Success.
- SERIALIZED_CAPABLE original window expires before queued second input: zero
  second input/certificate and retained first evidence. Concurrent startup failure
  cannot serially fallback; live quota does not reset original window or resend.
- Actual native Failure/Unknown with valid APPROVE gives zero mode/floor votes and
  preserves potential Medium veto; genuine successful readonly completion is
  separate positive. Retention error/future abort keeps known work/noncertifying
  diagnostics and closes/enqueues exact logical authority without claiming death.
- Genuine Source7/Driver round successor to collective closure, retained graph loss,
  Task/parent/source/cancel race and failed certificate CAS rollback; all/quorum/any,
  STRICT floors, author lineage/dispositions/full-round fresh fix remain unchanged.
- Complete actual retention/body/index/ledger caps, migration10→11/current-positive
  writers/cached10/refused reopen, control/closure reserves and cross-Project
  contention instrumented on exact composed clean source.

Compiled mutation control must compile, reach actual qualified prerequisites and
fail its causal assertion; compile/setup refusal is not a kill. Preserve exact
source/mutation/restore commits, logs/commands and every unresolved failure. No
seeded private capability, fixture-only Passed, fake Native opinion, waived positive
matrix or test disabling can qualify member/round production.

## 9. Closed round1 dispositions, mapping and delivery gates

| Closed design finding | Correction proposed here; not source qualification |
| --- | --- |
| C-RRI-M1, verified Medium: missing genuine Binding10 protocol/member dependency | §§1/2/4/5 require distinct actual round/member allocation/prepared/registration/consumption/normal-late binding/supervisor ports, full original frame, real vtable and original anchored Source7/Driver consumers. No Unit/worker/receipt reinterpretation. |
| C-RRI-M2, verified Medium: capacity after member reservation/preparation | §§2/3 require actual complete-cohort admission before every member handle/Unit/grant/helper/lock/input; pending holds integrate existing quota and actual member consumer validates them. No-capacity queue has zero member effects. |

The [original raw reports and proposed dispositions](../verification/review-round-implementation-design-round1-review-outcomes.json)
retain the fixed reviewed source and both closed reports without erasing findings.
Both prior reports were closed before author correction: Root APPROVE_COMPONENT_DESIGN
with zero findings and C REQUEST_CHANGES_COMPONENT_DESIGN with the two above;
Root subsequently verified both causal gaps. Their fixed63c findings remain
historical evidence, not approval of this authored successor. The allocation change
from10 to11 was coordinated dependency metadata, not a defect counted in round1.

| Normative contract | Exact implementation seam |
| --- | --- |
| Review R1 / I9-AC-8.k | §§2/3/7 original roster/mode/floor/profile/window and atomic capacity before preparation |
| Review R2/R5 | §§1/2/4/5 actual bundle/core/Context/allocation/snapshot/private member vtable |
| Review R3/R4 | §7 actual owned answer, strict content versus authority, identical vote/floor predicate and blocker/clearance/fix policy |
| Review R6 | §§3/5/7 exact own quota/derived mixed states/cancel/logical closure/fenced recovery |
| Review R7 | §6 complete retention/schema/old writers, §8 causal actual consumer controls |
| Binding MB1/MB5 | §§4/6 distinct sole member writer; no singular Session/Task/Workflow binding side effects |
| Binding MB2/MB3/MB4 | §§1/2/4/5 immutable full original frame and actual selected owner/prepared/consumed/latest identity |
| Binding MB6/MB7/MB8/MB9 | §§2/4/5/6 preflight, retained actual supervisor/Driver, durable readiness, original successor and bounded nongrant closure |

Before source development, Root and a different author independently review this
fixed corrected design and its actual9/10 consumer extensions. Source order is
actual9/10 composition → quota/cohort plus original round/member immutable ports
and11 guards → real snapshot/native/private binding/successor consumers → logical
member closure and certificate → full migration/causal/mutation/regression/source
review. Shared production files/schema are not edited by this documentation task.
Every intermediate utility/parser/table branch remains noncertifying until the
entire path qualifies; no actual Review availability is claimed from approved docs.

Actual official Claude/Codex/Grok authentication/settings/required hooks, native
instruction/channel/read-only qualification, two-OS four-Task result protection,
#16 representative two/Triple and public MVP remain separate user-approved native
acceptance. No accounts, credentials, main/GitHub, license or README are changed.
rururunx is not a security sandbox; immutable result authority and logical closure
are separate from best-effort process/cookie/Docker cleanup.
