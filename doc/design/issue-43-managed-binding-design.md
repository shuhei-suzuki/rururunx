# Issue 43: managed native Session binding design

Risk: STRICT. Status: design candidate; requirements approved at
`7e47421a24225583795245b307e574fb3156dd7e`, with the approval checkpoint at
`079f5fd37184240c08fbca3859ec6a0ac78ae139`. No production implementation,
native Driver readiness, migration qualification or authenticated CLI acceptance
is delivered by this document.

Normative requirements: [managed binding](../requirements/issue-43-managed-binding-requirements.md).
Inherited contracts: [Requirements9](https://github.com/shuhei-suzuki/rururunx/blob/b191b466d5dea303585cfaf6968c6fb178be79cd/doc/requirements/issue-43-requirements.md)
and [Design10 mechanics](https://github.com/shuhei-suzuki/rururunx/blob/b191b466d5dea303585cfaf6968c6fb178be79cd/doc/design/issue-43-binding-mechanics.md).
The factual binding, original frame, sole writer, complete ledger, negative identity
checks and finite cost contracts are preserved. The managed requirements explicitly
adapt reviewer worktrees and late logical settlement; they do not make current
Unit, NativeInvocation or NativeResult DTOs equivalent to the private #19 protocol.

## 1. Actual baseline and delivery boundary

The source inspected here is immutable
`802764f03893d0ee8e08ee19045ccb333387ad9b` (schema8). Native6 supplies genuine owned
stdio acquisition and result receipts; SourceRecovery7 supplies current retained
source recovery; Verifier8 supplies command-only Tests receipts and private acceptance.
None supplies the missing complete phase owner/prepared-input protocol or binder.

At that source, Workflow's managed Launched arm sets Session/execution, refreshes
owners, performs ordinary persistence and only then disarms its preparation guard
(`workflow.rs:1653–1657`). Ordinary persistence writes Task before Workflow
(`state/mod.rs:905–910`). A save refusal can drop the preparation guard for an
already launched invocation. Pre-Session Waiting and due polling also persist
Task/Workflow and re-enter preparation (`workflow.rs:1679–1695,1751–1818`).
Those are actual replacement sites, not valid record-only binding producers.

Native's Session registration is after readonly Git/version helpers and quota
admission (`execution/native.rs:185–342,353–427`). Its private NativeSeed binds
Context/payload to Unit/Session, but has no original Workflow/full-lock frame.
`reserve_native_input` has real before-wire intent and current Context checks
(`state/execution/native_results.rs:257–328`); that is retained and extended,
not treated as an already complete phase admission credential.

This component must deliver the complete new managed private protocol through
the real selected adapter and all protected writers. The current legacy #19
allocated owner and the new owner remain different types. There is no public
conversion, receipt-to-owner constructor, SQL fixture capability or shared fake
authority. Account-free peers exercise the real production producer paths.

Migration order is **Verifier8 → actual Runtime9 → Binding10 → ReviewRound11**.
Runtime9 is under development, not inferred from a schema number or the current
read-only controller. Binding10 DDL/guards are installed only after joint protocol
design/source review and composition with actual9. The previously fixed Review
draft remains immutable; its later allocation update uses11. Native-ready Driver
is a separate actual private consumer gate, described below.

## 2. Types, allocation and original marker

All authority types below are crate-private, non-Deserialize, non-publicly
constructible and have private fields. Allocation/launch/consumption ownership
handles are non-Clone. Scalar IDs and diagnostic read DTOs are freely readable
but cannot instantiate a proof. Store code validates current rows even when a
genuine proof is supplied. Type names in this document specify new ports, not
APIs present at the baseline.

| Type | Sole actual producer and permitted consumer |
| --- | --- |
| ManagedNativeComposition | Registry installation after real Claude/Codex vtable, private protocol, sole writers, Driver retention and migration guards compose; Workflow preflight only |
| OriginalMarker | Dispatch-marker Immediate transaction; immutable original frame and operation identity used by every subsequent phase port |
| PhaseSessionOwner | That marker transaction allocates exact Session ID to the actual selected NativeSessions/Unit/phase; readiness is completed only by the genuine Session registration transaction |
| PreparedPhaseInput | Marker transaction accepts the Store-validated actual committed Context/input plan; actual Native preparation and wire builder validate this same pair |
| PhaseLaunch | Marker producer hands the exact owner/input pair to an independently retained managed phase supervisor and actual adapter start; cannot be reminted from ManagedInput |
| ValidatedPhaseSession | Actual Native register transaction after qualification, before owned child spawn; binds the allocated Session and NativeInvocation to the pair |
| ConsumedPhaseInput | Actual `reserve_native_input` transaction consumes the one-shot private input admission; represents recorded consumption/intent, not OS delivery |
| OwnedPhaseSettlement | Actual owned terminal transaction after exact consumed input and acknowledged invocation/result checks; late factual binding only |
| CurrentWorkflowSuccessor | Store-owned coherent complete chain/body planner; current currency for specifically enumerated ports, never native grant authority by itself |
| DeferredBinding | Actual retained supervisor with original owner/operation; exact retry/reconciliation, no new dispatch |

### 2.1 Preflight and prepared namespace

Workflow checks implementation-owned composition before hold clearance, a fresh
phase Context, reservation, Unit preparation, marker, helper or spawn. Missing
composition returns typed UnsupportedComposition with no effects. Capability
metadata, CLI existence, configuration, FakeAgent and an owned-looking result
cannot enable it. Standalone Native6 remains a separate profile; a Workflow scope
protected by Binding10 cannot bypass preflight through that entry.

Actual registered adapter selection fixes alias, provider, native role, model/effort
and supported protocol profile. A later registry lookup cannot substitute a vtable.
Initial Source preparation and fresh Unit reservation use their existing exact
private source/Worktree contracts. Executor uses its actual fresh Unit namespace;
reviewer uses the genuinely prepared immutable-source readonly snapshot with
separate output namespace, artifact/profile and pre/post qualification. Task's
executor worktree is not used as a reviewer identity. No new sandbox, user, VM,
container, root requirement, HOME change or credential handling is introduced.
Required hooks/settings remain qualified prerequisites; unsupported profiles refuse.
The actual Driver checks this preflight before calling initial
ManagedWorkflowSources::prepare as well as before Workflow::step. Source preparation
or a public registry lookup alone cannot enable native dispatch. Protected Task/
Workflow activation routes require the same private composition; standalone source
fixtures are not evidence of that route. No supported native entry may hide an
effectful probe before this static implementation check.

### 2.2 Marker transaction and pre-Session allocation

Outside the Store mutex, the actual supervisor plans bounded canonical bodies,
input bytes and complete scoped lock inventory from a coherent read snapshot.
The marker Immediate transaction checks the exact current snapshots, Unit/profile,
Context/input source and complete lock set; applies only the authorized initial
dispatch-marker Task/Workflow transition; allocates the phase owner/input pair;
reserves the operation's mandatory audit/quota budget; and commits OriginalMarker.
The owner exists **before all Native start helpers**, not only when Session is
registered later. It reserves a new exact Session ID, selected adapter origin,
actual prepared Unit/path and input identity. It cannot bind before real Session
registration, send input or represent successful work while merely allocated.
The plan fixes exact post-marker encoded Task/Workflow bytes, version increments
and timestamps before Immediate. The transaction publishes those planned bytes
under exact old-body/version CAS, then records the already planned resulting
frame; it does not hash a newly reread full body inside the lock. Source7's
separately authorized pre-marker advance similarly consumes an exact sealed
pre/post plan and current row CAS, rather than generic recapture after a write.

OriginalMarker contains the resulting **post-marker** P/G/T versions and exact
body digests; Workflow Record ID/version/complete canonical body digest; scope;
active attempt index/phase/actor and Workflow generation; state-root instance and
current epoch; Unit ID/execution generation/profile/path; selected provider/alias/
role; allocated Session ID; exact launch/latest Context versions/digests, revision,
source map, artifact/governing/instruction identities and prepared payload digest;
and the complete sorted scoped WorktreeLock ID/version/body-digest set. Array
order, inactive history, budgets, retries, detail, completion and all other Workflow
fields are included in the complete body hash. Empty lock sets require a complete
query, not a truncated scan. Full scoped means the existing owning Task scope's
entire WorktreeLock set, without selecting only one visible lock.

No later refresh_owners, returned Session, Driver snapshot, pause, helper outcome
or current read can replace these pins. The immutable input pair points to exact
Context/payload bytes and the selected native wire profile; dynamic owned thread/
turn identifiers join the admitted wire receipt later, without replacing input.
Actual start receives the private PhaseLaunch, not a public tuple of IDs.

### 2.3 Preparation, registration and one-shot input

The actual NativeSessions start path validates PhaseLaunch before readonly Git,
version and other scoped helper intents. Every helper remains registered to the
same Unit before effects and rechecks original currency, epoch and cancellation.
Resource admission gate coverage remains draft → Unit registration → lease reserve;
it is released before materialization, Git, child wait or command await. Neither
that gate nor a SQLite mutex spans wire IO. Registration uses the originally
allocated Session ID, not a fresh UUID generated after helpers.

`register_native_session` gains a private managed branch that, in one Immediate
transaction, checks OriginalMarker, current full owners/locks, actual prepared
namespace/profile, selected adapter and exact input pair; writes the Starting
Session/session_units and NativeInvocation; and records the pair as validated.
Its Unit/Session/Context/receipt protections and Verifier8 command-only refusal
remain. Merely copying a NativeSeed or allocating a public ExecutionAuthority
cannot enter this branch. It must refuse before child spawn on stale currency.

The actual Core owns the validated pair. Codex initialize/thread/turn and Claude
stream initiation keep their real bounded raw protocol checks. Before input wire
bytes, `reserve_native_input` atomically validates the original frame plus permitted
factual Workflow successors, actual latest Context pointer/full payload, Session,
Unit generation/epoch and one-shot pair; records exact frame hash, Native6 input
effect ID and owned thread/turn metadata; and consumes admission once. Permissions,
hooks and later transport journals cannot change the consumed input row. ALLOW
and delegated effects also require original currency and real current own Session;
DENY/revocation retains its separate nongrant authority. Admission is a before-wire
authorization, not an atomic transaction with the OS or a delivery guarantee.
Cancel races are ordered at actual private admission; already admitted partial
wire delivery remains uncertain and is never replayed automatically.

Confirmed delivery/ACK update only their own typed evidence, retaining immutable
pair/frame/effect identity. A partial write, lost ACK, conflicting thread/turn,
duplicate changed input or stale epoch cannot be converted to NotDispatched.
Actual Native6 collector and terminal projection remain the producers of owned
work and retained answer content; text/exit0 alone does not authorize binding or
Review approval.

### 2.4 Actual adapter and closure seams

Registry's actual Claude/Codex installation keeps a crate-private ManagedNativePort
backed by the same NativeSessions instance as the selected public adapter. Its
start_phase consumes PhaseLaunch and returns an owned invocation channel to the
supervisor; its control/settlement ports address only that allocation. Private
descriptor origin includes the selected registry instance and actual provider
vtable. A caller cannot ask any public Adapter or another alias to adopt it.
Public start_managed/SessionRef paths detect the protected contract and refuse
without the matching private route; their readable DTOs are not alternate grants.
Legacy Codex, GenericCLI, Grok and Fake implementations do not expose this new
port. Interactive/standalone entry cannot attach to a protected phase owner.

The actual marker producer calls NativeSessions' private allocation factory before
its commit and checks selected origin in the transaction; the factory's provisional
handle is unusable until that exact marker commits. The real registration branch
validates the allocated handle and consumes the preparation handle once. On commit
uncertainty neither producer retries a new UUID: the supervisor reads the unique
operation/allocation/Session fact before proceeding. Actual terminal processing
hands its existing private NativeTerminal correlation to the logical settlement
producer in the same native-result/Session transaction. The binder never constructs
that terminal proof. A typed non-success producer can close an actual allocation
with no current dispatch or known owned failure; it cannot mint late success.

## 3. Durable schema and writer guards

Binding10 adds the following private tables and reserved indices on existing audit.
Scope/identity columns are indexed
and checked against strict bounded body decoding. Maximums are complete encoded
bytes, not selected fields. Foreign keys and uniqueness bind to existing actual
Unit, Context, Session/native invocation and Workflow identities. Diagnostic read
APIs expose records, never a handle with dispatch/binding authority.

| Table | Immutable core / mutable typed facts / limits |
| --- | --- |
| workflow_native_contracts | Actual activation Workflow/scope/profile/epoch origin; legacy-held or composed contract; never configuration-synthesized; one per Workflow, ≤4096 bytes |
| managed_phase_operations | OriginalMarker, pair/owner IDs, allocated Session, original Workflow body hash/reference and reserved allowance; immutable; phase-open/closure and supervisor readiness facts typed separately; one per marker, operation metadata≤4 MiB |
| managed_marker_bodies | One immutable original complete encoded Workflow body≤8 MiB per operation; referenced digest/index; full extraction outside Store, no extra full body read in binder Immediate |
| managed_phase_owners | Operation/Unit/epoch/generation/provider/alias/role/actual path/Session origin; immutable; actual registration readiness/NativeInvocation correlation typed; one per operation and unique Session, body≤2 MiB |
| managed_phase_inputs | Exact preparation Context/template/profile/source pins; immutable; one per owner, body≤2 MiB; payload itself≤1 MiB and original existing frame budgets still apply |
| managed_phase_admissions | One actual consumed input intent per pair, exact Native6 effect/frame/owned thread-turn binding; immutable core, separate confirmed/uncertain/settled status; body≤8192 bytes |
| managed_phase_readiness | One actual supervisor-owned bounded row per operation with immutable origin, current parking/reconcile state and monotonic start-ended/known-terminal observations; ≤4096 complete bytes; no unbounded notification history or reconstructed authority |
| existing audit: reserved factual links | ONE immutable audit row is the link; expression indices enforce operation/local ordinal/predecessor identity; ≤4096 bytes/link, ≤256 links; no second mirror-table write |
| scoped_session_identities | All scoped Session/provider/native UUID identity records, including Lost/history; actual Session writer transaction maintains indexed identity/malformed flag; metadata≤4096 bytes/entry |

Existing Native6 NativeInvocation, results and effects remain canonical native
transport/answer evidence. These tables do not create an independent owned result
or mutable copied authority. A terminal correlation references the exact existing
receipt/version/hash; HistoricalDraft cannot be upgraded. Persisted owner rows
alone cannot reconstruct the in-memory producer/private allocator. Actual activation
protects fresh Workflow native attempts before a Unit or Session exists, following
Verifier8's independent managed-contract principle.

All mutable tables participate in connection-local schema contract10 guards;
cached older connections cannot write by using raw SQL. All generic Record,
Session, audit, WorkflowAccess and recovery writes reject an existing protected
attempt's session_id/execution None→Some or change. New attempts start both absent.
The binder alone uses a narrowly private fixed SQL/typed projection path, not a
caller-selectable RecordOnly access mode. Paired Session/execution must be absent
or exact together; malformed partial tuples reject. Existing Unit reference is
already fixed before marker and is not changed by binding.

Guards reject same-version body mutation and existing-key INSERT/REPLACE on
protected Workflow Records and all reserved audit/operation/link identities;
reserved append-only UPDATE/DELETE and conflicting predecessor/sequence also
reject with recursive_triggers OFF. Generic audit APIs cannot use compiled private
names. Trigger CHECKs bound actual OLD/NEW complete bodies. Connection-local
private port tokens authorize only a derived exact delta and are scoped/reset for
that transaction, never reusable public writer permissions. Raw SQL by arbitrary
applications or the same OS user is outside a security guarantee; supported rrx
writers and cached rrx binaries must fail closed.

The fresh/migrated10 transaction verifies actual9 namespace/layout, installs
all10 tables/guards atomically and updates schema only after success. Collision,
partial or wrong9 layout leaves9 untouched. Existing active operations become
LegacyHeld; migration cannot manufacture owners, consumed input or OriginalMarker.
Newopen9 binaries reject10; already-open9 APIs and raw cached writers reject every
new/old mutable table. Newopen10 and cached10 similarly reject actual11 after
Review migration composition. The actual8→9→10→11 matrix, concurrent initializers,
fresh namespace and pre-open writers require compiled controls. Installing10 is
not evidence that operational Runtime9/Driver or Review11 is implemented.

### 3.1 Keys, state facts and transaction tokens

Operations have primary key operation_id, unique
(workflow_id, workflow_generation, attempt_index), explicit scoped P/G/T columns,
owner_epoch, Unit/execution_generation and allocated Session ID; the full immutable
marker metadata and digest agree with those columns. Owner and preparation IDs
are unique foreign keys to that operation; allocation has unique Session ID and
Unit+phase-operation identity. Registration adds only its exact existing invocation
ID and validated fact once. Admissions have unique pair_id, Native6 input effect
ID and invocation ID, with foreign keys to all three actual producers. Confirmed
and settled facts are monotonic and cannot replace frame/thread/turn/pair identity.
Current NoCurrentDispatch parking and consumed admission are mutually exclusive;
successful exact due-claim/registration invalidates the old parking claim. A retained
historical parked fact cannot authorize a later resume or consumption. Session index
keys include Session ID, scope, provider, nullable native_ref and an explicit
malformed flag; no unique(provider,native_ref) constraint is used to hide historical
conflicts. Complete scoped negative queries read all distinct matching identities.

The negative lookup reads the protected identity projection/index, not 4096 full
Session recovery bodies. Actual Session writers validate bounded body→projection
agreement atomically; raw supported Session writes without that exact projection
reject or mark the scope non-indexable. Legacy indexing validates complete bodies
in bounded pages and produces no grant while incomplete/oversized/malformed;
there is no truncated-success certificate. Fresh negative queries check indexed
record IDs/versions and invalidity flags; own latest Session's full body is still
read and checked inside Immediate. The final migration must declare/instrument
its separate total byte/row bound, refusing qualification rather than loading an
unbounded historical corpus.

The existing audit.sequence is the globally unique audit identity. A reserved
link's local ordinal (1..256) derives from consecutive Workflow versions relative
to OriginalMarker, without adding a new field to Design10's payload. Expression
indices on compiled factual kinds enforce unique operation+Workflow-after-version
and operation+prior_ledger_digest. Links carry original marker digest, Workflow
versions/body digests before/after and their exact typed payload. They enforce a
single append-only chain; ordinal1 predecessor is OriginalMarker, not
an arbitrary current audit. SQL before-insert checks reject replacement conflicts
before deletion regardless of recursive_triggers. No separately mutable ledger
head/count is written by binding: sequence/head/allowance usage derives from the
bounded immutable indexed chain. Operation closure/budget-release facts belong
to the separate phase_closed transaction, not binder's write set.

Store plans a PrivateWritePermit containing exact old/new encoded body, row IDs,
versions and audit payload for one port. The connection-local guard checks exact
planned bytes and metadata rather than accepting a broad access-mode name. The
permit is installed only by crate-private Store code for that transaction and
cleared on commit/rollback/error; callbacks perform no nested DB or filesystem
reads. Supported public writer APIs cannot construct a permit. Owner P/G/T,
Context and lock checks compare exact bounded encoded bodies/version inventories
against the sealed coherent read plan; same-version mutation and index/body
inconsistency refuse. No in-lock rehash is needed to grant currency.

The native_diagnostic payload is exactly the Design10 header: kind, scoped
P/G/T/Workflow IDs, Workflow generation/attempt/phase, own Session, private
operation, OriginalMarker digest, Workflow versions/body digests before/after,
prior ledger digest, canonical body recipe, allowlisted reason, detail≤128 UTF-8
bytes, Context version and existing audit timestamp. Complete encoding≤4096 bytes.
It carries no raw CLI error, credentials, argv/environment, grants or resource list.
Other port schemas use that same predecessor header plus only their enumerated
typed projection. The final DDL and writer CHECKs must be reviewed together with
the actual producer; table presence or a token function name alone is insufficient.

## 4. Record-only binder

The sole private `bind_managed_phase` consumer accepts an actual normal-return
ValidatedPhaseSession or actual OwnedPhaseSettlement, OriginalMarker and a Store
sealed plan. No caller constructs this argument from SessionRef, receipt ID,
execution status, NativeStart DTO or arbitrary JSON. Normal and late use the same
transaction and exact projection; only the private eligibility predicate differs.

Full canonical planning/ledger extraction occurs in a coherent snapshot outside
held SharedStore. Recursive object keys sort by UTF-8 byte order, arrays retain
order and compact JSON retains exact validated value semantics. Domains are
`rrx.workflow-body-sha256/v1\0` and `rrx.workflow-ledger-sha256/v1\0` as Design10;
the literal final NUL is part of hashing. Typed envelopes include canonicalization
version. Original captured Record body bytes also remain retained, so an encoded
body mutation cannot evade the current version guards.

The actual Store-owned planner uses a separate read-only/query-only connection
to the retained owner's canonical selected state database, with a coherent read
transaction; it does not call RuntimeOwner::open or increment epoch. Read snapshot
creation/decoding and body/chain planning happen before acquiring the writer Store
mutex. Private origin/current-index checks qualify its result; publication compares
exact current metadata/body bytes under Immediate. Reading unrelated rows in
separate unlocked calls is not a coherent snapshot. Native original-currency and
Source7 private validation receive this same sealed plan and check its exact head
inside their own effect transaction; they do not recompute full body hashes inside
the mutex. A changed head requires a fresh genuine successor proof before the
still-unconsumed effect, not fresh original pins or a replayed wire effect.

The Immediate transaction checks:

1. Exact current state-root instance/epoch, operation/phase-open identity, Unit/
   generation/profile and original allocated/prepared/actual registered pair.
2. Exact original current P/G/T identities, versions and full bounded bodies,
   active owner states, Context/current pointer/source/revision/governing/instruction,
   complete scoped locks and active attempt identity. Unrelated parent version
   advancement is not exempted. No current-pins refresh is performed.
3. For **first binding**, current Workflow version/body equals the original
   post-marker frame, both fields are absent and there are zero factual links.
   Diagnostic/gate/bookkeeping before first binding cannot become a predecessor.
4. Latest actual own Session matches allocated ID/scope/actor/alias/provider/role/
   prepared Unit worktree/model/effort and immutable native origin. Returned
   native_ref, when Some, equals latest. Starting PID/native UUID/lifecycle may
   legitimately advance, so no stale returned Session-version CAS is imposed.
   Missing/Lost/foreign/ambiguous Session refuses.
5. A complete scoped identity query proves no distinct Session with the same
   provider/native UUID, including Lost/history. All body/index identities agree;
   malformed/non-indexable entries or scan overflow refuse, never count as absence.
6. Normal eligibility requires genuine prepared+validated registration, even
   before input/ACK; it does not manufacture consumption or delivery. Late
   eligibility additionally requires actual consumed pair, current owned successful
   terminal and logical input/terminal settlement at the same original epoch.
   If terminal publication won the race with normal delivery, the successful
   terminal must pass this genuine settlement predicate too. Known Failure/Unknown
   follows its non-success/held path; a prepared normal handle cannot bypass it.
7. The planned before/after complete Workflow projection differs **only** at this
   active attempt's absent session_id and execution reference. Execution is derived
   from the exact private Unit/Session relation, not caller authority. Record
   version/updated_at are the only changed Record metadata.

It writes the Workflow Record directly with CAS and ONE reserved session_bound
audit/link in the same transaction. No Task, P/G, Session, Context, WorktreeLock,
owner, count, readiness, SourceRecovery or Driver authority row is updated. Generic
workflow.saved/gate events are suppressed for this port. Audit retains Design10's
exact field list below; Unit/execution is bound through the private operation and
hashed complete Workflow, not an extra public grant. No provider payload,
credentials or process list is copied.
Readiness acknowledgement and supervisor progress are later typed facts, independent
of factual binding's atomic write set.

The binding payload fields are exactly: kind=session_bound, project_id, goal_id,
task_id, workflow_id, generation, attempt_index, phase, session_id, provider,
actor, role, workflow_version_before/after, task_version_preserved,
dispatch_started=true, marker_identity (scope/Workflow/captured version/generation/
index/Context version), context_version, proof_source (normal_return or
closed_settlement), private_operation_ref, private_receipt_ref (absent normal;
exact genuine settlement late), original_marker_frame_sha256,
workflow_body_sha256_before/after, prior_ledger_digest,
canonical_body_recipe=rrx.workflow-body-sha256/v1, and existing audit timestamp.
The compiled kind is rrx.private.workflow.session_bound. Complete encoding≤4096
bytes includes keys/envelopes. The link digest hashes the complete immutable
payload and excludes its own digest. There is no duplicate ledger-row insert.

AlreadyBound requires the same immutable operation/pair and exact committed
session_bound link/body/derived tuple, plus a complete proved current permitted
successor. It performs no write, audit or new input. A different operation, partial
tuple, absent audit, matching current IDs without proof or foreign suffix refuses.
Normal/late races converge at this check. A binding refusal leaves the exact live
reservation/invocation held; it never calls ordinary fail/retry/release/refresh.

## 5. Retained supervision, parking and readiness

### 5.1 Ownership handoff before any launch result

A real `ManagedPhaseSupervisor`, retained by RuntimeOwner/actual Driver, receives
PhaseLaunch and the preparation guard **before** adapter start. Engine awaits a
result from this owned invocation; it does not own the only start future or armed
guard after marker. The supervisor retains exact selected adapter, owner/pair,
original frame and invocation outcome across Engine timeout/drop/abort. Actual
Launched handoff transfers/disarms preparation guard before any binding error can
unwind; Core owns the process/terminal proof and supervisor owns factual reconciliation.
It never detaches an unobserved helper or turns an unknown start into a new start.

Actual Core terminal processing continues to preserve observed work and durable
receipt on storage retry, projects current Session coherently, and uses unreaped
OwnedProcess identity for group hygiene before reaping. Binder retry does not
kill Core, discard known successful terminal or send input. Core's existing saved
private terminal proof can retry result persistence; that is distinct from passive
status binding. Process/cookie/Docker cleanup stays best effort and cannot certify
logical result, native death or Review approval.

### 5.2 Pre-Session NoCurrentDispatch and quota parking

Version/readonly helpers can run before quota admits a Session. Only the actual
supervisor with the original PhaseLaunch and actual helper settlement can produce
`NoCurrentDispatch`: no Session registration, no native child start/input/ALLOW,
no unresolved helper/delegated effect and no consumed pair. Merely Session None,
an error string or a public Waiting DTO does not prove this state.

The typed parking transaction changes only existing Unit/quota waiter/lease facts
and bounded supervisor readiness. It does **not** write Task or Workflow, append
a factual Workflow link, clear dispatch_started, recapture marker/pins or relinquish
the original operation. Actual Driver and Workflow status derive effective quota/
capacity waiting from these facts; stored Task remains at its original marker
state/version. Usage classification and exact bucket/probe/fair waiter controls
remain the actual quota producer's responsibility. Due-claim consumes the exact
Unit/waiter/version/due record once, revalidates OriginalMarker and resumes only
the retained not-yet-dispatched launch. Helpers are not replayed when unsettled.
No new Unit, Context, Session owner or PhaseAttempt is created by this resume.

This replaces the current pre-Session persist/re-entry branch. A missing retained
producer, stale original owners/locks or unknown helper/start outcome parks Held
with attention; it cannot use generic pause/retry to mint a new marker. Proven
non-success closure can later close this original operation and allow an explicit
fresh attempt, with fresh worktree/resource semantics retained.

### 5.3 Binding deferred states and active Driver

Durable readiness is derived from phase-open marked/unbound operations plus
actual allocation/registration/input/terminal facts and typed start-ended observations.
It is present before/with successful logical settlement, not created solely by
the caller's notification. Supervisor registration precedes launch. Completion
(normal/error/drop/timeout/cancel), Driver registration/wake and explicit active
reconcile recheck this level predicate. Wake messages carry only bounded IDs and
are hints. Lost/full/closed channels cannot remove readiness.

Transient BUSY/LOCKED with confirmed rollback yields DeferredBinding retaining
the exact invocation. Uncertain commit yields CommitUncertain: read exact Workflow
and unique reserved link to prove AlreadyBound or confirmed rollback before retry.
Constraint/schema/proof/bounds errors are not contention. They park Held with a
bounded reason and no new grant. Durable terminal persistence retries retain the
actual private known-terminal proof, never infer success from stored text.

The **actual Runtime9 native-ready Driver integration port** must retain these
supervisors independently of Engine futures, provide bounded fair per-Project
capacity and enumerate ≤64 pending operations/page with a stable fair cursor.
It has finite fallback timers (100 ms–5 s capped backoff) and retries only predicate
change/actual due time; unchanged Held predicates are not a busy write loop.
Passive status/poll cannot bind, claim a launch or reconstruct ownership. Native
availability remains refused until this genuine Driver path exists and is tested;
read-only ResolveProject, accepted Goal metadata and a named Driver DTO do not
satisfy it. Pending queue capacity is reserved before marker, not after losing a
launch result. Same-Project fairness is tested independently of global quota.

| Actual operation observation | Allowed next action, without authority recapture |
| --- | --- |
| Allocated / start retained | Genuine same supervisor prepares or qualifies helpers; otherwise Held |
| Proved NoCurrentDispatch / due wait | Park original pair; exact due-claim can resume once after original currency revalidation |
| Registered / normal identity available | Binder normal path, or defer/hold preserving actual invocation |
| Input intent consumed / outcome uncertain | No redispatch or late success; real collector/own proof may resolve, otherwise Held |
| Owned known successful logical settlement | Same binder late predicate, then separately genuine result/gate closure |
| Owned known failure / proved no dispatch failure | Typed non-success closure; absent binding remains absent |
| CommitUncertain | Exact original Workflow/audit read proves prior commit or rollback before retry |
| Trusted terminal decision / revoked | No further grant/binder/gate; actual nongrant stop/eligible TerminalRecovery only |
| New epoch / missing actual producer | Held visible; authentic factual-only restore gate or explicit qualified fresh closure/retry |

Readiness row state updates are bounded current observations, not a copied grant or
unbounded per-notification journal. They cannot replace original marker, immutable
owner/pair or native evidence. Terminal proof retention belongs to the actual actor;
status cannot read a row and recreate it. Driver never steals a different supervisor's
operation, and cancellation targets exact owned operation/Unit, preserving siblings.

## 6. Late settlement, closure and restart

Successful late binding requires the same real allocated/validated pair and
consumed admission; exact current owned Native6 successful terminal with own
Session/Unit/input/ACK/frame; and recorded logical terminal/input settlement.
Native effects are closed. Result-finalization may remain open for the genuine
Runtime capture; binding neither closes it nor publishes an artifact. Cleanup
Unknown/Leftovers alone does not block or authorize this path. Result capture and
immutable artifact acceptance remain separate gated consumers. HistoricalDraft,
Unknown work, NotDispatched pair, stale instance/epoch, ambiguous external effect
or partial/uncertain input remains held. Known Failure uses genuine non-success
closure without inventing absent Session binding or an approved result.

Task-terminal cancel/fail_task first performs its actual trusted lifecycle
revocation even if factual bookkeeping is unprovable. The typed terminal_decision
port validates the original chain and authorized Task-before/after lifecycle delta,
writes that decision plus one reserved factual link, and prohibits subsequent
binding/diagnostic/gate admission. Revocation is not native permission and does
not forgive source/Workflow drift. If its chain cannot be proved, real stop/fence
remains available while bookkeeping is held; cancellation cannot launder drift.

`phase_closed` consumes a genuine successful captured-result/readonly/gate proof,
known non-success/no-current-dispatch proof, or eligible TerminalRecovery decision,
with complete prefix and all logical applicable effects settled. The corresponding
typed port enumerates exact Task/Workflow/Context/artifact changes and releases
unspent ledger reservation atomically. It does not require proving process death;
uncertain irreversible external effects still hold. Destructive namespace cleanup
waits for genuine capture/finalization/draft retention; physical cleanup jobs may
continue afterwards without changing work. Closed unbound failures retain Session
None. Engine::fail is non-success phase closure/awaiting_explicit_retry, not an
invented Task-terminal decision. Unknown work stays visible for explicit recovery.

Restart currently calls `begin_execution_epoch` (`state/execution.rs:485–557`): it
retains state-root instance, advances epoch, closes Native/finalization permissions,
fences Sessions/effects/leases and invalidates SourceRecovery7. Binding cannot
rewrite OriginalMarker epoch or manufacture a current owner from these rows.
Active/unknown old operations remain Held; an actual fresh retry follows genuine
non-success closure and new worktree/Unit/generation, not a copied input replay.

An across-epoch factual-only restore requires the actual reviewed #14 fencing and
restore producer, with evidence of original allocated/consumed **owned known terminal
committed before fencing**, authentic operation origin, exact unchanged original
P/G/T/W/Context/locks and complete old ledger. A private RestoreFactualBinding must
record original epoch versus the actual fenced recovery epoch, bind no live owner,
open no permissions and send no input. It is not produced by receipt reads or a
Current Runtime ID. This baseline has no such complete port: implementation keeps
that restore Unsupported/Held until its distinct joint source review and causal
controls qualify it. Logical known-work preservation and ordinary historical result
inspection remain available without falsely restoring native authority.

## 7. Factual successor consumers and SourceRecovery7

First binding has zero predecessor links. Thereafter a complete ≤256-link chain
anchors every permitted Workflow change to the immutable OriginalMarker. Store
planning validates all original pins and exact canonical allowed deltas in a
coherent snapshot, producing a sealed CurrentWorkflowSuccessor outside held Store.
Inside Immediate, exact current Workflow version/body identity and latest immutable
link ID/sequence/digest must match the plan. Reserved guards make version/head CAS
meaningful; an arbitrary current body hash or audit tail cannot mint this proof.

| Port / reserved kind | Exact permitted projection and producer |
| --- | --- |
| session_bound | §4 only; actual normal/late private producer, first link |
| native_diagnostic | Bound live status_unavailable or persisted_status_mismatch only: active.detail≤128 UTF-8 bytes, Record version/time and one link; no Task/owner/native wait write |
| gate_claim | Actual private gate invocation: Evaluating + exact claimed_observations; no Session binding/source refresh |
| gate_observed | Actual compact claim outcome + exact Waiting/held disposition fused into ONE link/transaction, including bounded detail/held_reason; repeated identity may change only checked occurrence count/last_time |
| gate_hold | Actual typed hold/clear policy, exact held_reason/detail; no Task WaitingHuman rewrite |
| terminal_decision | Trusted cancel/fail_task action and its separately checked Task lifecycle delta; no success/binding/input |
| phase_closed | Actual eligible typed result/gate/non-success/TerminalRecovery proof and enumerated final phase/body/Task/Context changes; no transport-derived approval |

Native quota live wait/recovery does **not** use native_diagnostic or these Workflow
links. It updates its genuine Unit/pool/waiter/lease facts and derived status only;
no Task/W rewrite occurs while open. Terminal quota/capacity interruption uses
typed non-success phase closure with preserved Unknown work and explicit wait/fresh
retry policy, not live quota as failure. This choice preserves fixed ledger budgets.
Typed derived waiting feeds actual Workflow/Goal/Runtime/CLI status and scheduling;
it is not cosmetic text masking a hidden Task update.

SourceRecovery7 currently compares serialized full snapshot pins exactly
(`source_recovery.rs:286–293`) and generic after_write refreshes its row
(`342–367`). A binder Workflow+1 therefore needs a **new bounded private successor
recognizer** in validate_task/validate_binding/recovered_source_task and the actual
inputs/prepare_pack/registered Git consumers. Its anchor is the existing saved
SourceRecovery Workflow pin at the operation marker, advanced only by the genuine
pre-marker typed reserve/marker transaction. It may resolve that exact anchored
Workflow pin to the chain-proved current Workflow while every other saved source
pin remains exact. It neither mutates row.pins/version nor invokes generic
after_write. Subsequent original operation source authority stays the original
one; the factual resolver is not a new native admission credential.

Runtime9 Driver's actual saved Workflow pin must use the same chain-recognition
contract anchored at its existing private claim, with exact scope/operation/epoch.
If that claim predates marker, the actual marker receipt must prove the separately
authorized exact pre/post marker transition from that saved claim; a current
post-marker snapshot is not substituted for the claim. Source7 may perform its
existing separately authorized pre-marker advance, but binding itself never does.
Binding cannot refresh Driver/Goal authority rows. If Driver9 cannot expose this
genuine consumer, native-ready composition refuses. Current read-only routing does
not supply it. Source7's later ordinary typed publication/closure can advance
Task/Context/artifact pins only through its separately authorized transaction,
after validating the genuine original chain; this is not binder bookkeeping.
Source/body drift or a disconnected chain refuses effects. Trusted revocation
keeps its independent nongrant path and does not adopt changed parent authority.

## 8. Writer and actual consumer replacement inventory

All locations refer to802764f. Ordinary writes remain available for unprotected
legacy scopes and separately authorized closed-operation transitions. Every protected
open native phase writer must enter a named private port or refuse; no catch-all
refresh_owners/persist route remains. Source patches must re-inventory all call sites
against the final composed commit, including new Runtime9/Review11 callers.

| Actual baseline caller | Required protected-profile replacement |
| --- | --- |
| Workflow initialize/reserve/context (`workflow.rs:566–712,832–910,1101–1125`) | Composed activation before native phase effects; ordinary authorized pre-marker Context/Task reserve remains exact; no retroactive owner creation |
| prepare_agent legacy return (`1273–1453`) | No protected-scope use without genuine private protocol; generic/Fake/capability-only legacy entries refuse preflight |
| prepare_managed refresh/marker (`1599–1615`) | Actual private marker/allocation TX; no later refresh of OriginalMarker |
| Launched (`1653–1657`) | Supervisor already owns launch/guard; exact private binder; errors defer/hold instead of retiring launched Unit |
| pre-Session Waiting/error (`1679–1695`) / due claim (`1751–1818`) | §5.2 proved NoCurrentDispatch parking and exact original-operation resume; no ordinary Task/W save or marker re-entry |
| status error (`1854–1855`) / persisted mismatch (`1869–1870`) | Separate bound-only native_diagnostic port; capped reasons, no raw provider error; before binding keep supervisor readiness/attention without W links |
| live quota (`1880–1881`) / recovery (`1949–1950`) | Genuine quota facts + derived WaitingQuota/running status; no Task version/state rewrite |
| terminal quota/capacity (`1911–1912`) / Unknown (`1923–1924`) | Actual typed non-success close/hold policy; no invented work Failure or native retry/input replay |
| actual Native known failure/result finalization (`1954` onwards) | Genuine terminal, retained draft/result and typed closure ports; no absent ID fabricated binding |
| gate observe (`2004–2013`), evaluate/apply (`2378–2885`) | Complete private gate claim + fused observed/disposition + atomic completion chain, retaining Verifier8/private result acceptance |
| cancel/fail_task/terminal recovery (`2024–2074`) | Trusted nongrant terminal_decision/recovery, complete chain; no native currency refresh |
| finalization/escalation/invalidate/hold/fail/retry (`931–1018,2108–2375,2892–3001`) | Actual closed-operation source transition, typed hold/decision/closure, or refusal; no open operation ordinary persist |
| ordinary transition validator (`3184–3340`) / WorkflowAccess and Store (`state/mod.rs:427–910`) | Reject every existing Session/execution bind delta in all modes; only private binder changes them; native factual ports bypass Task writer with exact projections |
| observe_workflow_gate (`state/mod.rs:919–985`) | Protected private fused gate observation replaces workflow.gate_observed and generic workflow.saved; no generic Source after_write |
| execution reserve/adopt (`state/execution.rs:574–872`) | Pre-marker Unit relation, exact private reservation; protected original marker prohibits later adoption/reallocation |
| retirement Task projection (`state/execution.rs:999–1018`) / artifact Task publication (`artifacts.rs:323`) | Genuine nongrant/closed-result typed transaction only; no inadvertent Task write on binding/refusal/live quota |
| Verifier8 reserve/accept (`verification.rs:698–795,982–1110`) | Preserve command-only grant and no-native marker; exact factual gate successor/closure consumption when composing adjacent native phases |

### 8.1 Every Session writer

| Actual writer | Required rule |
| --- | --- |
| register_session_tx (`state/execution/sessions.rs:200–251`) / register_native_session (`native_results.rs:230–255`) | Exact original allocation/pair branch; atomic Session identity index; cannot register a second Session for one owner; keep command-only refusal |
| Core ACK update (`sessions.rs:21–51`, `native.rs:1177`) | Latest Session CAS/immutable identity plus original private current phase validation; positive PID/native UUID advancement preserved |
| close_session_tx / close_execution_session (`sessions.rs:55–70,253–279`) | Actual own nongrant terminal evidence, no new admission, preserve known work; index includes terminal/Lost history |
| fence_epoch_sessions (`sessions.rs:139–179`) / Owner preparation Drop (`execution/owner.rs:65–81`) | Factual Lost/fencing only, no owner reconstruction; guard ownership distinguishes never-launched from already retained actual invocation |
| private Native result/registration guard (`native_results.rs:365–503`, Native Core/drop) | Actual observed proof/correlation retained across storage errors; own Session updates and logical terminal facts atomically coherent |
| public put_session (`state/mod.rs:990`), conditional/environment APIs (`1007–1156`), generic put_record/write_record_tx (`412,1631`) | No protected private owner creation or native Session mutation/binding; reserved scopes use actual typed writers only |
| Generic CLI (`adapter.rs:1123`), Grok (`adapter/grok/mod.rs:425,844`) | Protected scope refusal until actual complete private protocol; no capability/config bypass |
| Legacy Codex (`codex/session.rs:521,555,577,1321,2268–2270`) | Existing legacy authority remains distinct; public legacy entry cannot mutate new managed owner Session or mint phase pair |
| any recovery/Session importer/raw supported writer | Complete identity index transaction and typed fencing/restore contract; malformed history refuses negative proof; never create live proof from body |

Inline fixture-only Session writers and cfg(test) register_execution_session do
not qualify the real phase producer. Public Record and Session query DTOs remain
diagnostic. Every actual Session write maintains identity indices including
replacement attempts and lifecycle changes, with positive body/index consistency
validation; migrations scan complete legacy scope or mark it malformed/held.
The negative check is scoped and finite, LIMIT4097 overflow refuses. Current own
Session is checked inside Immediate; identity alias/provider native UUID equality
uses validated exact strings, never untrusted normalization dropping a conflict.

## 9. Explicit finite allowances and transactional cost

The inherited per-operation256 links are reserved before effects:99 gate cycles
×two links=198, one binding, one terminal decision, one closure, eight hold/clear
links (four pairs),47 optional diagnostics. No quota status link is added. Each
gate reserves its pair before effect; each hold reserves its clear. The100th gate
refuses before claim/effect. After99 Waiting, only trusted cancellation and eligible
TerminalRecovery remain; no native redispatch/budget reset. An admitted99th Passed
can close. Hold/clear exhaustion refuses further gates and successful closure;
mandatory cancellation/closure slots remain available. Diagnostics exhaustion
cannot consume those slots. Repeated no-write facts spend no second link; repeated
actual gate calls do spend their pair even when compact outcomes coalesce. Closure
releases only unspent reserve; spent retained bytes remain charged. Held retains
reserve. All new durable bodies count toward the actual128-MiB Workflow budget.

Complete chain extraction outside Store is≤256 links/≤1 MiB. Sealed compact
transaction proof is≤128 metadata rows×4096 bytes≤512 KiB, including all actual
certificates/endpoints/scalars. This is **not** the total SQL/JSON/write cost.
Every port declares only the surfaces it reads and instruments complete encoded
rows, repeated reads, JSON/CHECK and writes; unlisted/oversized surfaces refuse.

| Additional mandatory surface | Maximum complete encoded cost class |
| --- | --- |
| P/G/T | One each≤8 MiB; existing Goal DAG limits remain |
| Workflow | One≤8 MiB; OLD/NEW full projection and body-write/CHECK cost measured |
| launch/latest Context | At most two rows≤8 MiB each; input payload remains≤1 MiB |
| own Session | One≤4 MiB; recovery≤3 MiB/depth32/nodes32768; current read inside Immediate |
| original operation / frozen settlement | One≤4 MiB / one≤64 KiB; Native6 answer text≤1 MiB separately, receipt's existing complete encoding bounds retained |
| preparation/admission/owner | One each≤2 MiB; stricter8192-byte consumed admission remains |
| scoped WorktreeLock set | Complete≤256×16384 bytes≤4 MiB; sorted original ID/version/digest inventory≤64 KiB |
| scoped identity / retained physical owner negative lookup | Complete indexed≤4096×16384 bytes≤64 MiB, paths≤4096 bytes; overflow refuses; no physical acquisition by binder |
| applicable logical host-effect closure | ≤256 admissions×8 KiB +256 outcomes×2 KiB; all worker joins outside Store; stricter existing Unit/effect limits still apply |

Mandatory indices/scope constraints prevent unlimited irrelevant global scans;
complete own-scope index validation and bounded malformed flags are required.
Full body planning/hash/chain proof and filesystem/path preparation occur outside
Store. Publication has no await, filesystem, process or native call. Current row
CAS and full scoped lock/own Session/negative checks stay inside Immediate.
Finite cost is neither a measured latency bound nor a guarantee of mutex fairness;
near-bound controls must report other-Project contention honestly.

## 10. Implementation and qualification sequence

1. Compose fixed actual Runtime9 and Verifier8 baseline; re-inventory every writer.
   Implement private phase allocation/preparation/registration/input/terminal ports
   and Binding10 immutable schema/guards, without claiming readiness.
2. Implement retained supervisor, exact NoCurrentDispatch parking, derived quota
   status and private current-source/Driver successor recognition; no generic
   after_write authority refresh. Keep unsupported actual Driver/restart gates closed.
3. Connect Workflow preflight/marker/normal and late binder; replace all open-phase
   writers with exact private diagnostic/gate/decision/closure ports. Requalify
   Native6, Source7 and Verifier8 production consumers with the composed source.
4. Run causal account-free actual-adapter/stdio controls, compiled mutation controls,
   old cached writer/migration matrix and clean fixed full regression. Independent
   source reviewers must review actual producer/consumer composition, not only SQL.
5. Native official CLI authentication/settings/hooks/four-Task tests remain Phase3
   user-approved qualification on macOS/Linux, reported separately. No cleanup,
   universal compatibility, process-death or OSS-readiness claim follows from fixtures.

| Required control group | Actual causal consumer |
| --- | --- |
| Normal held start | Actual Engine/registered Claude and Codex private producer: preinput/preACK Starting binds once; Task/P/G/Session/Context/locks unchanged; real callback remains eligible |
| Exact source/CAS | Second DB changes each original owner/context/W/lock body/version, added/deleted locks, same-version mutation/REPLACE; binder and prewire admission refuse without new input/audit |
| Identity and writers | Actual prepared reviewer path; foreign/missing/Lost/duplicate historical UUID/malformed index; every generic protected writer; paired delta and extra history/detail reject |
| Guard handoff | Fault first binder save after real Launched and abort Engine awaiting start: supervisor keeps exact own Core, known result and input count; no preparation Drop retires live invocation |
| Pre-Session wait | Actual settled version/helper+quota refusal, exact due waiter/resume original marker/unit/pair, unchanged Task/W; stale waiter and unsettled helper refuse without recapture |
| Deferred/late | Actual current consumed successful terminal before lost return with cleanup Unknown/Leftovers binds once; normal/late race; uncertain commit and lost/full wake; NOT NotDispatched/HistoricalDraft/Unknown/foreign epoch |
| Quota/live consumers | Genuine wait/recovery produces derived Workflow/Driver status, unchanged open Task/W, no resend; terminal interruption closes non-success with fresh retry; siblings unaffected |
| Successor/closure | Genuine binding→diagnostic→gate→result/Verifier→closure chain; Source7 and actual Driver original pins remain untouched/current; cancel with drift does not launder it |
| Finite/DDL | 99/100 gates, four hold pairs, diagnostics exhaustion, mandatory closure, closed reserve release; near8-MiB body/4-MiB Session/256 locks; raw guards with recursive_triggers OFF; cached9/10/newopen refusal matrix |
| Restart | Actual fencing preserves known work and holds unknown; row-only owner/receipt cannot restore; authentic #14 factual-only path tested only after its real producer exists |

Each positive fixture first reaches genuine actual allocation/registration/admission
prerequisites. Mutation controls remove exact checks/restore Task bump/refresh pins/
remove sole writer or predecessor/negative index/private pair/guard handoff/readiness,
and fail at the intended consumer assertion. Compile/setup failure is not a kill.
Record clean exact source commit, mutation/restore trees, commands and limits. Full
fixtures cannot be disabled, exempted or replaced with seeded private authority.

## 11. Requirements mapping and human gates

| Requirement | Concrete design sections |
| --- | --- |
| MB1 | §4 fixed atomic write set, §7 exact factual links |
| MB2 | §2.2 immutable full post-marker frame, §4 original CAS |
| MB3 | §2 complete actual owner/input protocol, §3 protected origin, §6 settlement |
| MB4 | §2 prepared namespace, §4 latest own and complete negative identity checks |
| MB5 | §3 sole writer guards, §8 complete call-site inventory |
| MB6 | §2.1 before-effects preflight, §1 actual composition boundary |
| MB7 | §5 retained supervision/deferred parking, §6 late/non-success split |
| MB8 | §5.3 actual Driver/durable level, §6 fenced authentic restore boundary |
| MB9 | §7 genuine successor/typed ports, §8 wait/writer inventory, §9 budget/cost |

Independent joint design approval is required before production source development;
independent source approval is required before installation/native availability.
Actual Runtime9 Driver and #14 recovery capabilities cannot be
invented to bypass dependencies. Unsupported restore remains held and visible.
The schema order is agreed allocation, not evidence of available implementations.
This proposal changes no license, README, authenticated account state or main.
rururunx is not a security sandbox. Work outcome, immutable result certification
and best-effort cleanup remain separate throughout.
