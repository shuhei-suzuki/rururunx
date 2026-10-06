# Issue 43: Native preparation, helper and quota integration

## 1. Status and fixed source

This is a component implementation supplement to the approved
[managed binding design](issue-43-managed-binding-design.md), especially §§2,
5.2, 7–10, and the
[retained supervisor design](issue-43-phase-supervisor-integration-design.md).
It adds no alternative authority or relaxation of the approved requirements.
Source baseline is `52d9f532e78116cabd307d76256fb2965f7462d9`.
This document proposes implementation; it does not certify a working managed
Native launch, authenticated Agent compatibility, cleanup or Phase2 completion.
Native availability and the installed composition issuer remain refused until
the actual producer/consumer chain and its required qualification are complete.

The baseline already retains a genuine selected allocation, committed
OriginalMarker, Runtime-owned launch job, actual Native registration/input/ACK
and saved terminal objects. Its protected preparation path is still incomplete:

| Actual baseline consumer | Blocking generic dependency |
| --- | --- |
| `execution/native.rs`, `start_with_launch` | Generic execution/input validation, PreparationGuard, profile, readonly Git and version helper paths follow the initial private launch check |
| `state/execution/native_phase.rs`, `validate_phase_preparation` and private stage plans | Generic `validate_authority` still requires the marker-incompatible ordinary Driver/source route |
| `execution/git_io.rs`, UnitGit; `execution/process.rs`, `capture_scoped_pinned` | Generic helper intents and periodic permission validation do not consume the original marked phase |
| `state/execution/effects.rs` | Generic helper/Native transport reconciliation does not accept an actual prepared marked actor |
| `state/execution/quotas.rs` | Generic admission, candidate validation, telemetry and wait/recovery writers lack the original private marked actor; treating their refusal as an absent sibling is unsafe |
| `execution/native.rs`, pre-Session Waiting | An ordinary Waiting DTO/guard disarm is not the approved actual NoCurrentDispatch/parking handoff |

The implementation must replace every listed protected consumer, including
already-written registration, input, projection and terminal plan dependencies.
A preliminary check followed by the generic path is insufficient. Public legacy
entry continues to refuse protected scopes. Unprotected historical paths retain
their existing checks. A real process, copied allocation facts, Session/receipt
rows, a version string or a named flag cannot substitute for the private chain.

## 2. Actual actor and complete prepared input

`NativePreparationActor` is a private, non-Deserialize producer owned by the
actual selected Native start. Its constructor accepts the SAME retained
`Arc<PhaseLaunchParts>` only from the selected NativeSessions vtable. It retains
the original allocation, same-Unit start gate and preparation responsibility in
the Runtime-owned start future. It cannot be made from ManagedInput, IDs or SQL.
The selected weak adapter must upgrade and match its original object/origin at
each effect; losing it holds the original operation. No edge creates a strong
cycle back through NativeSessions or the Runtime slot/job.

Before a helper, a separate read-only coherent snapshot produces a sealed
`NativePreparationPlan`. Full parsing, hashing, filesystem/profile qualification
and exact image construction occur outside SharedStore. Planning checks the
actual original allocation input bytes, original prepared source/Context and
original Unit against the retained marker plan. The live mutable worktree is
never used to regenerate policy, prompt or source pins.

The complete prepared-input contract equivalent to the approved #19 protocol
is enforced through actual producers and consumers:

1. The actual pre-marker Driver/Sources lane prepares the fresh executor or
   readonly reviewer namespace and exact committed Context/template bytes. Its
   original preparation ownership remains in the genuine allocation/marker;
   arbitrary equal-looking PreparedInput is not a replacement.
2. Selected allocation captures provider/alias/origin/role/model/effort/program,
   full Unit/path/profile, input kind/revision/version/source inventory/payload
   and complete encoded input. Marker publication validates these exact captured
   facts against the original Context and genuine preparation/Driver advance.
3. Native preparation rechecks that same origin, namespace/profile and source
   contract. Reviewer input additionally consumes its exact indexed retained
   artifact/readonly source qualification and lease; a caller's artifact ID or
   successful Git command does not qualify an artifact.
4. The actual private preparation object is issued only after all required
   finite helper outcomes and profile checks have known settlement. Registration
   consumes it once for the originally allocated Session/invocation. The
   prepared object is retained across uncertain registration; no new pair is
   allocated and no rows recreate it.
5. Native prewire consumes the same private input once; validated actual ACK is
   retained before fallible journals. ACK is distinct from Starting→Running and
   nativeRef projection. Existing saved terminal facts remain ahead of optional
   quota/storage reads on every Core route.

No `prepare_managed` re-entry, new source snapshot, current-row Driver ticket,
new marker or payload reconstruction is an allowed resume. If the original
Driver/Sources preparation seal is not carried by the actual marker, its genuine
consumer must be added with its owning author before this stage can qualify.
This document does not declare an existing allocation alone a complete proof.

The actual Native command is a distinct effect from preparation helpers. Its
registration-to-spawn and pre-Core custody replacement is mandatory §4.1 below;
the helper gate or an earlier successful preparation check cannot authorize it.

## 3. Private Store seam and transaction validation

Proposed crate-private signatures name real argument producers; they are not
public constructors, callbacks or readable DTO grants:

```rust
// Native owns these types and their actual start/child/input issuers.
struct NativePreparationActor { /* same retained launch and actual start */ }
struct NativePreparationPlan { /* sealed snapshot; nongrant by itself */ }
struct PreparedNativePhase { /* same actor; actual settled preparation */ }
struct PhaseHelperPlan { /* exact action and before/after images */ }
struct PhaseHelperObservation { /* actual retained owned child observation */ }
struct PreparedPhaseNoCurrentDispatch { /* actual supervisor-owned state */ }
struct NativeQuotaPlan { /* finite pool/lease/waiter/readiness images */ }

// Planning borrows authentic state; no lock-spanning filesystem/hash/await.
fn plan_phase_preparation(owner: &RuntimeOwner, actor: &NativePreparationActor)
    -> Result<NativePreparationPlan>;
fn plan_phase_helper(actor: &NativePreparationActor, action: PhaseHelperAction)
    -> Result<PhaseHelperPlan>;
// Store methods consume sealed plans; actual Native invokes process effects.
fn begin_phase_preparation(&mut self, plan: NativePreparationPlan) -> Result<()>;
fn reserve_phase_helper(&mut self, plan: PhaseHelperPlan)
    -> Result<CommittedPhaseHelperIntent>;
fn settle_phase_helper(&mut self, plan: PhaseHelperSettlementPlan)
    -> Result<CommittedPhaseHelperSettlement>;
fn park_phase_quota(&mut self, plan: NativeQuotaPlan) -> Result<PhaseQuotaCommit>;
```

The committed intent/settlement objects are private same-plan known-commit
acknowledgements, not permissions obtainable by reading effect rows. Actual
prepared registration and input APIs consume `PreparedNativePhase`/the real
registered actor rather than accepting a generic ExecutionAuthority as proof.
Concrete signatures can be narrowed during source implementation; ownership,
producer privacy and mandatory checks cannot be weakened.

Every effect/registration/input/normal projection Immediate transaction checks:
selected actual database, SAME retained launch/allocation/operation, actual
selected vtable identity, `validate_current_tx`,
`OriginalMarker::validate_driver_live_tx`, exact current full Unit/index/epoch/
generation, original P/G/T/Context/scoped locks, open lifecycle, actual stage and
original owner/input/readiness relation. Root's current successor and Driver-live
checks are conjuncts, not replacements for Native eligibility. Exact old/new rows
and compiled permissions are derived outside the writer mutex. No await, process,
path canonicalization or full-frame hash occurs inside Immediate.

Generic authority validation may be factored into nongrant factual predicates
and its existing ordinary Driver/source predicate. The ordinary caller always
keeps both. The protected private caller must instead consume the actual
original Driver/Source preparation plus Root current-successor predicates AND
all existing relevant Unit, Task, parent activity, input and reservation checks.
There is no boolean protected bypass, public validation mode, optional actor
fallback or `.ok()` demotion of failed currency.

Preparation readiness moves allocated→preparing once under exact same-pair CAS;
parking/resume changes only its typed current state and monotonic observations.
Registration validation accepts only the real prepared actor's eligible state,
not an unconditional allocated-v1 predicate or a free current-row state lookup.
Existing initial owner/input identities remain immutable. The same original
actor, stage history and settled effects accompany the transition.

## 4. Helpers, capture and mandatory abandonment

The finite helper action vocabulary is NativeVersion and the already required
readonly Git qualification commands: exact HEAD, status, committed object/profile
and retained artifact checks. Each variant generates argv internally from the
original allocation/qualified object input; no arbitrary shell, mutable refs,
caller argv, delegated Docker action or unrelated command is admitted here.
Git uses the existing sanitized environment, disabled lazy fetch/replacements,
separate output/profile and genuine readonly lease. Configuration/hooks unsupported
by the qualified profile refuse; this stage introduces no authentication copying,
HOME substitution, filesystem sandbox or hooks override.

The actual effect intent is committed BEFORE spawning. The real admission gate
serializes the last currency check/known intent and synchronous process spawn
against cancellation; SQL commit and process spawn remain separate effects.
No Store/actor/queue lock spans an await. The actual Runtime admission lock may
be acquired asynchronously before its synchronous check/intent/spawn section;
there is no await inside that section. Cancellation before spawn prevents it;
cancellation after spawn retains the actual child and exact intent for nongrant stop/settlement.
If that serialization cannot be provided by the actual owner/Driver admission
port, helpers stay refused. A successful earlier validation is not a spawn grant.
All non-effectful planning precedes that gate, and joins/hashing/capture follow
its release. Different Tasks keep independent owned children/cookies/namespaces.

Capture uses a typed phase observer carrying the same actor and committed intent;
its periodic fence checks the same original/current private chain rather than
generic `capture_scoped`'s ordinary Driver validation. OwnedProcess remains
unreaped while group hygiene uses leader identity. Bounded output/timeout,
cancel/fence and actual exit observations follow the existing capture contract.
Native helper stdout is not result approval or owner evidence. A nonzero expected
readonly qualification outcome can be settled and then interpreted by its
allowlisted consumer; settled does not imply successful preparation.

Intent, spawn uncertainty and actual observation stay in the Runtime-owned job
before a fallible journal. HelperGuard/actor abandonment captures authentic
child/effect identity before optional Store calls. A mandatory nongrant closure
port may reconcile the same original effect and latest full Unit CAS after normal
currency is revoked, without reopening effects, granting input or manufacturing
success. Known actual exit/work is preserved; unresolved spawn/drain/storage is
held and reported. No helper is replayed merely because its journal is absent.
Unknown irreversible effects prevent NoCurrentDispatch and logical closure.
Physical process cleanup remains best effort, independent of the work result.

Each known-commit effect update checks exact original effect identity and
version/body preimage. On commit uncertainty it retains the same sealed plan
and confirms exact postimage or proven absence/rollback before any retry. Actual
observation/settlement survives caller Drop, wake loss and journal error. A fresh
read of an effect row cannot construct an observation or prepared owner.

Preparation admits at most 32 helper intents per original operation, including
failed attempts; IDs and settled outcomes are a bounded retained inventory.
Each version helper captures at most 64 KiB combined stdout/stderr and each Git
qualification at most 1 MiB combined output, with at most 8 MiB aggregate captured
bytes for preparation. Overflow is an actual incomplete observation, never
successful version/source qualification. Existing shorter child deadlines and
stricter verb/output limits still apply; each helper also has a 30-second outer
deadline, followed by the existing bounded drain/stop attempt. Neither deadline
certifies process death. There is no unbounded helper retry; at most one exact
postimage/absence reconciliation attempt occurs per wake/timer, with held state
and capped 100-ms–5-second backoff between unchanged failures.

### 4.1 Actual Native transport registration, spawn and pre-Core custody

Baseline `execution/native.rs:539–586` holds SharedStore through actual child
spawn; `587–672` then extracts pipes and inserts the registry entry before Core
ownership is transferred. Releasing only Store would open a stop-versus-spawn
interval and still leave post-spawn fallible work before retained child custody.
This path must be replaced together with helper preparation, not treated as an
ordinary legacy start or granted by its generic registration guard.

The real selected Native start uses a private `NativeTransportStartPlan` derived
from its SAME `PreparedNativePhase`, original allocation/launch and fixed command.
Its finite internally generated command is the existing qualified Claude or
Codex provider profile, with original program/cwd/environment/model/effort and
bounded pipes. It is not a PhaseHelperAction or arbitrary command callback.
Registration/transport-start intent use the actual originally allocated Session/
invocation/operation and same pair. The explicit transport-start effect is distinct
from later protocol boot-call/input effect intents; none implies input or ACK.
Its exact durable representation and guard layout must be coordinated/reviewed
with Root before source installation; this supplement grants no existing generic
managed-effect kind extra permissions.

Before admission or any process effect, the actual Native registry/Runtime-owned
start retains a preallocated private `NativePreCoreCustody` for this SAME launch.
It has the original immutable registration/transport plan and prepared actor,
allocated Session/invocation and an initially empty owned-child cell. Runtime's
real original start job retains this actual producer independently of the Engine
future; no public arbitrary future, callback or returned DTO creates that custody.
It has no strong return edge to Runtime/phase queue and cannot be constructed from
SQL or a copied Session. Actual registry entry identity and allocations needed
for the final move are established before spawn; empty custody grants nothing.

The concrete protected `start_with_launch` replacement is:

1. Plan exact full registration and transport-start intent images outside Store;
   retain their SAME Arcs and preparation actor in that actual start custody.
2. Acquire the SAME real Runtime/Driver stop-admission object used for helpers
   and trusted stop. It is the original object retained by the private launch,
   with no strong Runtime/job backedge or generic permission callback. Recheck
   service/Driver revocation and exact selected adapter/prepared stage identity.
3. Under SharedStore, run the exact last current-successor/Driver-live/full Unit/
   pair/prepared-input checks and one Immediate registration plus transport-start
   intent transaction. Retain its actual known-commit acknowledgement and actual
   registered actor before any later fallible projection. Registration uncertainty
   is Held; do not spawn based on an unconfirmed return/error or reallocate IDs.
4. Release SharedStore while still holding that SAME admission guard. Synchronously
   invoke OwnedProcess::spawn, then immediately move the returned real owned child
   into the preallocated retained custody, before fallible pipe extraction,
   Session PID projection, channel/registry work or Core construction. The move
   has no allocation, fallible locking or await; a dedicated short mutex may
   recover its poison for ownership retention, but poison cannot grant effects.
   Actual abort/unwind cannot leave the child solely in an unretained temporary.
5. Release admission before pipe/I/O/handshake awaits. Transfer child/actor/controls
   to an actual retained Core job only after all required handoff objects exist.
   Its abandonment guard is constructed BEFORE task spawn/first poll. The same
   original pre-Core custody remains until the exact Core accepts ownership; no
   interval has only an Engine future, Entry DTO or unpolled inner guard as owner.
   Core input consumption and ACK then follow their separate private protocol.

There is no process call inside Immediate, and no filesystem/hash/await while
holding Store, actor, queue or child-custody mutex. Admission alone spans the
short synchronous registration/known-commit/spawn/ownership-move section. It does
not span child execution, pipe handshake or Core I/O. A stop that wins admission
first prevents both registration and spawn. If registration wins, stop waits
until spawn/no-spawn outcome and authentic custody are installed, then targets
that SAME operation; this is linearization, not a process-death guarantee.

| Actual observation/fault | Required same-original handling |
| --- | --- |
| Registration confirmed rollback | No spawn; retain original prepared actor/plan. Only proved unchanged predicates allow an original operation retry; never new allocation/pins |
| Registration commit uncertain | No spawn. Retain original plan and resolve exact original postimages/preimages under the private known-commit protocol; row equality alone does not construct a registered actor |
| Known registration but stop/revocation before spawn | No input/ACK/success; actual custody/registration acknowledgement enters nongrant same-original closure. No generic retirement Task rewrite |
| OwnedProcess::spawn returns error | Retain real invocation/intent and bounded actual error classification; no invented process/Owned success and no blind process replay. Only authentic settled no-child state can contribute to non-success/no-dispatch closure |
| Child created, pipe/registry/projection failure | Retained pre-Core custody owns child, same original actor and observed outcome BEFORE failure. Actual nongrant stop/drain/terminal reconciliation uses it; missing pipes or poisoned registry cannot discard it or mint NoCurrentDispatch |
| Caller/start future or Runtime Drop before Core handoff | Actual independently retained custody and its eager abandonment guard remain responsible for exact child/outcome. Original marked operation is Held until genuine nongrant settlement; no new start/input or DTO restoration |
| Exact Core accepts ownership | One actual move into retained Core; pre-Core responsibility ends only after acknowledged transfer. Later known terminal storage errors retain actual terminal as in the baseline corrections |

Native helper, registered Session and transport intent abandonment share their
authentic latest full Unit/Session/pair CAS and mandatory closure allowance.
Normal current-currency failure does not prevent factual stop/known-work retention,
but that nongrant closure opens no input/effect permission and issues no successful
settlement from an absent child, Unknown or stored answer. A post-spawn child can
never qualify pre-Session NoCurrentDispatch. Physical hygiene remains best effort.
If the real admission/pre-Core retention/closure interfaces are not yet composed,
this actual Native transport effect stays refused; the old under-Store spawn
branch is not a temporary protected fallback.

## 5. Quota and exact original-operation resume

Pre-Session quota runs only after actual preparation helpers are settled.
`PreparedPhaseNoCurrentDispatch` is issued by the real retained start actor:
it has no registered Session/invocation, child/input/ALLOW or consumed admission,
and no unresolved helper/delegated effect. Complete bounded negative SQL checks
conjoin this actual local state; row absence or Waiting DTO alone never emits it.
The actual pending capacity and preparation ownership stay retained while parked.

The typed parking transaction writes only exact Unit, applicable quota pool/
waiter/lease and bounded readiness images. Task, Workflow, Context, Driver and
Source7 rows/versions remain unchanged; it appends no factual Workflow link.
Actual Runtime/Workflow/CLI derives WaitingQuota/capacity status from these facts.
It does not persist ordinary Task WaitingQuota or recapture authority on resume.

Quota planning uses the actual selected profile's fixed global/per-Project/
provider caps, participating accepted windows, probe lease and fair waiter
policy. Every active lease, including a marked sibling, counts toward the active
union. An ordinary validator's refusal for a marked candidate must not remove
its lease from capacity or queue accounting. Candidate eligibility is separately
qualified by genuine owned scheduler registration/retained operation; unresolved
eligibility holds its admitted resources conservatively. Bounds apply before
body materialization, with overflow/refusal rather than truncated success.

The first profile admits at most 4096 active lease identities globally, 4096 due
waiter candidates in the selected provider/account pool, 256 previous same-Task
capacity-interruption identities, 64 participating windows and one selected pool.
Each inventory query uses its corresponding limit+1 overflow sentinel before
decoding, never an unbounded COUNT. Complete scalar index inventory is ≤4 MiB;
required Unit bodies are each ≤16 KiB and all candidate/history bodies together
are ≤72 MiB. Window bodies are each ≤8192 bytes, complete pool/waiter/lease images
each ≤8192 bytes, and the additional complete quota snapshot/plan is ≤8 MiB.
The existing global/provider configured concurrency maximum 1024 remains. A
second pass decodes only after complete count/byte qualification. Existing
stricter limits apply. Final Immediate rechecks exact selected pool/windows/
waiter/Unit and the complete relevant active/candidate inventory, so new or
changed leases invalidate the plan; no row is ignored as irrelevant after
planning. No whole global JSON decode, arbitrary account list or infinite probe
loop is introduced. Overflow refuses admission before effects; existing leases
and mandatory cancellation/closure remain available through their bounded
own-operation closure port, without scanning this whole admission inventory.

Exact due-claim includes original Unit/operation, waiter/version/reason/due and
readiness parkingVersion. It consumes that record once under same Immediate
current/Driver/actor checks. The retained start resumes the original actor/pair
and settled helpers; no new version helper is launched unless its earlier
actual outcome proves a separately allowed new attempt. Uncertain old helpers
remain held. Due changes do not reset original elapsed-capacity bounds. Unknown
capacity expires according to the qualified finite recheck policy, preserves
Unknown work and requires genuine non-success closure/fresh-attempt policy.

Registered live telemetry uses its actual private Session/input actor and exact
current active lease. Foreign, unknown and stale buckets cannot rewrite siblings
or replace success. Accepted Available for all participating buckets can clear
the live derived wait only under the same current authority/lease. This does not
mint a probe except the existing qualified retry producer. Actual owned terminal
is retained BEFORE optional quota reads; quota or cleanup cannot overwrite known
work. Terminal interruption does not invent a failure from quota exhaustion.

## 6. Ownership, boundaries and qualification

B owns Native start/phase actors, the new private
`state/execution/native_phase/preparation.rs` planner/transactions and actual
Native helper/capture/quota consumers. Shared factoring in execution authority,
Git/process/effects/quotas/Session writers is coordinated with Root/A before edit.
Root owns marked Runtime custody, PhaseHandoffs, current-successor/binder and
compiled exact-permit core; A owns genuine Driver/Source preparation and original
advance/liveness. This supplement allocates no schema version. Any new durable
column/table or guard layout needs an explicit coordinated migration and cached
writer controls; readiness/effect rows are not silently repurposed as grants.

Required seams are concrete and conjunctive: original Driver/Source preparation
seal consumed by Native; Root current+Driver admission serialization; narrow
exact-image effect/quota/readiness permissions; Runtime-owned parking/resume and
derived status; nongrant abandonment/terminal projection. Until each real source
exists, the corresponding effect stays refused or Held. A stand-in callback,
always-live Runtime, plain ticket or fake private constructor is not an interim
implementation. Overall availability cannot be enabled by this component alone.

| Required account-free control | Actual producer/consumer and negative |
| --- | --- |
| Prepared source | Genuine Driver/Sources→allocation→marker→Native consumes committed config/rules A after live B changes; Context/template/profile/readonly artifact drift refuses before helper/input |
| Before helpers | Exact selected vtable and original marker; parent/body/lock/Unit/epoch/generation change between plan and Immediate refuses; no helper executes |
| Helper retention | Real version/Git owned child; intent precedes spawn; cancellation/start-future Drop before first poll/while capture, storage fault before receipt preserves child/outcome and siblings |
| Native transport admission | Real private registration/intent and SAME stop gate; pause after commit before spawn, race actual stop, verify its winning order and exact retained child/no-child custody; other Tasks proceed after the short section |
| Pre-Core child handoff | Real child created before injected pipe/registry/projection fault or future Drop; retained actual custody stops/reconciles same child without input/replay/Task write; eager guard survives unpolled Core job; registration uncertainty prevents spawn |
| Preparation state | Same actor/readiness preparing, registration once; replay or another alias/pair/Session refused; rollback and uncertain commit preserve original plan, no new UUID |
| Pre-Session wait | Settled helpers and genuine NoCurrentDispatch; exact parked due record resumes SAME operation/Unit/pair once, Task/W versions unchanged; unknown helper/duplicate/stale waiter refuses |
| Quota independence | Marked plus legacy active lease union, caps/fairness/boundary overflow, foreign/stale bucket; same-lease live recovery, no probe mint or Task/W write, no input resend |
| Mandatory close | Original actual helper/terminal after revoke, current full Unit CAS, known result preserved; no new permission/input or Owned success from unknown/rows |
| Finite compatibility | Complete encoded bounds and catalog/pre-open cached writer controls for changed guards; original old routes still refuse protected scope |

Each positive control must first reach genuine Driver, allocation, marker and
Runtime retention through the actual producer. SQL-seeded owner/Unit/Driver and
setup failure do not qualify positives or kill mutants. Controlled protocol peers
exercise actual Native stdio but do not certify official CLI/auth/hooks. Formal
controls run only after a clean fixed commit; meaningful compiled mutations must
fail the intended consumer assertion and restore the exact recorded source tree.
Independent source review excludes each author's inherited components. Default
full regression and lint failures remain open quality gates. Four-Task and real
macOS/Linux Native compatibility remain the later user-approved qualification.
rururunx is not a security sandbox; work, immutable result and best-effort cleanup
remain separate.
