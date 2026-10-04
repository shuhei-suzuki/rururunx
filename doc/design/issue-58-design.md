# Issue 58 design: retained Task-free native consultation ownership

Status: Design3 proposed; Requirements8 approved at8facb8d. No source/profile,
recovery, native TUI or whole-issue acceptance is claimed. STRICT.
Source baseline: integrated main d87faec/schema3, branch f84c94e; approved
[requirements](../requirements/issue-58-requirements.md) remain authoritative.

## 1. Authority and composition

Introduce one private `ConsultLifetimeOwner` at runtime composition, minted ONLY by
successful private Store acquisition and transferred once to the genuine native
supervisor before startup. It is nonserializable, non-Clone and unavailable through
public Session/Record/JSON constructors. Runtime composition registers the reviewed
implementation/profile and actual #6 F1 cleanup contract; provider/model/role/PID
labels do not enroll it. Persisted holds survive a crash but do not mint a live owner.
Trusted application API, not arbitrary linked code, same-UID OS/DB secrecy or a
biological Human test, is the boundary.

Coordinate one linear schema/writer epoch with #19 managed Task authority, #23
Goal writers, #43 binding and #60 runtime effect ownership. Final version is selected
by that composed migration, not an independent #58 schema number. Every table write
including generic/private APIs and already-open connections is database-fenced. The
composed migration installs BEFORE INSERT/UPDATE/DELETE triggers on EVERY mutable
table (including Projects, Goals, Sessions, audit and new private tables). Triggers
require the actual per-connection current-epoch application function registered ONLY
by current Store composition, not supplied JSON. Old schema3 connections have no such
function and fail after SQLite schema-cookie reload; a Rust-only check is insufficient.
Private authority functions additionally require the actual private write context;
generic current writers cannot fake private ports by knowing the epoch. Migration
uses its own reviewed private transaction context. Inventory all table names and
trigger coverage in the composed source. Use trusted_schema=OFF and audited
read-only integer/boolean predicate functions tagged INNOCUOUS (no I/O, SQL, arbitrary
state mutation or secrets), NOT DIRECTONLY (must run from triggers), and not a
cached DETERMINISTIC epoch value. Missing/misflagged functions fail closed. Startup
self-tests require current-context writes pass and actually pre-opened old/absent-
context writes fail on every protected table; failure blocks readiness. Actual
rusqlite functions feature/API wiring is part of the composed source, absent today.
See SQLite [function flags](https://www.sqlite.org/c3ref/c_deterministic.html) and
[application functions](https://www.sqlite.org/appfunc.html); INNOCUOUS requires review
of the actual predicate, not an automatic security label. Migration preflight proves
NO live pre-epoch runtime instance (including executing readers/helpers), drains every
actual legacy owner and requires
reviewed #14 recovery for uncertainty. Failed migration preserves schema/data/history
bytes. Terminal labels, inactive rows or an epoch retag do not prove physical fencing.
#58 cannot precede #19 Task-scope fences or #60 reverse producers in production.
Candidate component tests can run in isolated stores; missing ports keep affected
production profiles unready, and do not waive mandatory Consult/attach/recovery.

## 2. Physical scope and bounded complete inventory

Represent `PhysicalScope` as bounded typed canonical domains: Project root, worktree
namespace, gitdir/common directory and EACH actual native/runtime profile filesystem,
hook/helper/delegate domain. ANY cross-kind domain pair equal or ancestor/nested is
an overlap: common-dir inside another root/namespace and root inside another common-
dir are conflicts too. Use opened dev/ino identity chains plus canonical paths;
case-insensitive/firmlink aliases cannot be accepted from string-prefix comparison.
Profile inventory classifies external native config/session-store/HOME/temp and shared
Git objects/alternates/hooksPath/includes: genuinely isolated, continuously enforced
compatible shared use, or Unsupported before effects. Never ignore them or make HOME
an implicit global exclusive lock. Mandatory defaults/hooks cannot be disabled to
produce the proof. Complete bounded current effective profile inventory is a readiness
gate; unknown config/delegation domains are Unsupported, not assumed inert.
Compare all retained holds/reservations across ALL Project states, including Blocked/
Removed and uncertain historical instances. IDs/worktree labels never imply physical
independence. Registration/reactivation/root/identity changes use this same cross-kind
predicate and actual current domain inventory before introducing any overlap.

Bootstrap ONLY bounded non-executing in-process path/canonical metadata and Git
layout reads: plain primary .git directory or a validated bounded gitdir/commondir
layout. Bound each metadata file to16KiB, paths to4096UTF-8 bytes, link/file hops to32
and aggregate bootstrap data to256KiB. Enumerate bounded alternates/hooksPath/includes using EXACT later executing Git
environment and all applicable system/global/XDG/local config, conditional includes
and environment-injected config (GIT_CONFIG_PARAMETERS/COUNT/KEY_n/VALUE_n,
GLOBAL/SYSTEM/NOSYSTEM/EXEC_PATH/TEMPLATE_DIR); unknown/unclassified injection or
conditional include refuses before execution. No secret values are persisted. Resolve
without execution or refuse the layout/profile; verify opened physical identity chains
and changing aliases before effects. Unprovable/changing/unsupported layout refuses
before external Git; no guessed candidate reservation around a Git-first probe.
Source equivalence is a separate condition: raw files cannot impersonate filtered/
attribute-transformed Git content. Pure capture returns typedUnknown/refuses where
configured transforms prevent equivalent current source. No implicit config bypass.

One Immediate transaction reads the COMPLETE indexed intersection inventory and
current P/G/T/Session/full scoped lock versions applicable to the action. Maintain a
private physical-domain/ancestor identity index atomically with every registry/hold
mutation, across every retained state. Query every domain-pair overlap, never only a
Project-ID partition. Settled Removed history stays immutable/readable in historical
tables but leaves this active index ONLY when a private atomic retirement proves no
retained owner/reservation/recovery/reference requiring exclusion; no FK deletion or
terminal-label retirement. Reactivation re-enters through full overlap admission.
Missing/inconsistent index or unknown physical identity refuses, not truncated idle.

Proposed finite limits:4096 active indexed Project identities (new registration may
refuse at capacity, existing disjoint Projects remain runnable);64 effect domains per
owner;32 ordinary owner/reservation members per registered Project;4096 TOTAL members
in any intersecting physical connected closure, including all former instances,
recovery intents/partial effects and additional128 reserved successor/cleanup members.
Admission computes actual merged overlap closure and required profile worst-case
recovery headroom BEFORE effects, refusing if ordinary admission would consume that
reserve or exceed4096. Each permitted recovery effect counts its actual bounded new
owners/intents in that same invariant; actual successors adopt and consolidate fully
fenced prior members without dropping obligations. Unknown/unfenced members are never
compacted. Finite exhaustion holds with attention; real at-bound admission-refusal plus
successful actual successor recovery controls are mandatory. Unlimited arbitrary
crash chains are not promised by finite storage, but admitted ordinary workloads and
required crash-at-fence/reconciliation cases must retain adequate recovery capacity.

There is NO runtime-global retained-owner pool which Project A's uncertainty can
exhaust to deny an already-registered physically disjoint B. A quota is charged to
its exact Project plus affected physical closure; cross-scope transfers/registration
merges recompute the full invariant. Indexed intersection query at most4096 members;
no LIMIT/truncate and idle inference. The global DB may retain historical records;
it is not all decoded into a fixed global owner arena. Source must prove index/query
completeness and scoped costs, including retired history and concurrent reactivation.
64KiB encoded owner/receipt,64KiB bounded audit,128 submissions per lifetime,
256 operation references and nesting16 remain limits. Checked counters/versions fit
signed-i64; total encoded size validated before insertion. Any affected cap refuses
before effects without clamping. Numbers are policy bounds, not OS/RAM performance
proof; a same-scope exhausted case never earns native parallelism or recovery credit.

### Physical overlap, exclusion and compatible shared domains

Physical overlap is inventory, NOT the exclusion graph. An EXCLUSION EDGE exists
only when an overlapping pair lacks an ACTUAL reviewed continuously enforced profile
compatibility relation. Compute capacity closure/headroom and successor adoption over
exclusion edges, not all physical overlaps. Unknown compatibility is exclusion. Each
compatible shared native HOME/config/session-store domain carries exact per-instance
exclusive subdomains/resources plus its continuous enforcement obligation; no string
role/readOnly or same native UUID claims compatibility. Native default shared stores
cannot silently join every Project into one quota or adoption set.

The relation must survive Lost and apply to the successor's OWN actual fencing,
inspection and reconciliation effects. Compatible live peers outside the exclusion
closure stay live, retain their own quota, and are not adopted/fenced simply due to
shared configuration. Any dynamic transition which could create a new exclusion edge
must have pre-admitted reserved worst-case capacity and actually protect already-live
peers; a profile whose compatibility cannot remain enforced through Lost/recovery is
Unsupported before original effects. This is not an ordinary Lost-compatible bypass:
actual #6/#60 resource enforcement and exact #14 successor remain necessary. Private
physical index still queries ALL overlaps, then derives exclusion from actual pinned
compatible profiles, never omits a shared domain. An active executing mutator or
capture conflicts unless its concrete effect/governing-domain compatibility is proved.

Required control uses real native per-user shared store: A becomes Lost and its owned
successor fences/adopts/settles A's exclusive workload while B remains live and can
admit within its separate quota. Exhausted A closure cannot refuse B. Kill a mutant
that builds closures over all overlaps and one that treats Lost/shared labels as proof.
If genuine profile conformance is absent these controls remain mandatory/unready;
physical strings cannot stand in for an actual positive.

## 3. Persisted owner and transaction API

A private ownership table (not generic Record) stores immutable owner ID, physical
scope, Project/optional Goal, Task absent, runtime instance/epoch, registered profile,
actor/mode, input/source pins, actual native identities when known and checked version.
Store minimal allowlisted IDs/digests/version numbers/reason codes, never environment
values, prompts/native transcripts, credentials or raw configuration. States distinguish
ReservedSetup, OwnedLive, HeldUncertain and Settled; native turn state is separate.
A Session-less ReservedSetup hold already excludes competing effects. Actual Task-free
entry is typed start_consult(owner: ConsultLifetimeOwner, ...), consuming the owner
by value. Change generic AgentAdapter start/resume request to private-constructed
TaskScopedLaunchRequest with a REQUIRED Task ID/validated Task scope; a Task-free
request cannot be constructed through that signature. Migrate direct inherent and
registry Arc<dyn AgentAdapter> callers, ALL providers/fixtures. A compile-fail doctest
and real no-effect direct/registry rejection controls verify the boundary; public
registration/labels cannot replace the actual typed Consult entry. Implemented providers must migrate every direct entry, including
Session-less launch/probe paths; no Session write is needed to trigger that guard.

Proposed private operations, all epoch-fenced and with bounded audit in the same Tx:

| Operation | Required genuine authority and transaction effect |
| --- | --- |
| acquire_lifetime | registered profile + current Registered Project/accepted applicable nonterminal Goal + complete compatible physical inventory; inserts retained setup and mints owner |
| attach_owned_session | exact live owner, immutable scope/actor/input/native identity, Session CAS; creates/binds its factual Session without fresh inference |
| admit_submission/grant | exact live owner and current source/input/Session/P/G/full locks + lifecycle + reverse exclusion; consumes one actual current intent before wire |
| observe_owned | exact owner/operation/input/Session CAS; factual state only, no fresh authority or release |
| deny_owned | exact pending native request/turn/operation and owner; fixed DENY only under provider-specific historic guards |
| settle_lifetime | exact owner + known current outcome and full required physical profile cleanup (or proved settled/no-dispatch setup); atomic factual final publication + release |

All generic nonterminal Task-free Session inserts/updates are denied for EVERY
provider/profile/role, including identical-body bumps and terminal-to-live. Generic
Session/Record/scoped CAS cannot change any owner-bound Session or ownership row,
including terminal factual updates; they cannot hide a hold through role/path/body
changes. Generic terminal historical insertion is nonauthoritative and cannot bind,
release, resume or mint an owner. #19 supplies corresponding managed Task fences.
Metrics/audit observers never grant ownership or input. Private ports verify actual
capability identity, not merely matching public IDs/versions.

No native/Source/Git call occurs under SQLite/SharedStore locks. Capture uses an
actual private `CaptureReadOwner` reservation over complete physical/governing domains
BEFORE even pure in-process source reads. Executing capture ALSO requires its own
#60 effect owner/profile. The reservation prevents intersecting mutator admission
and remains retained THROUGH acquisition/admission/continuation handoff. Actual #60
capture outcome/full cleanup must already be proved, but its exclusion reservation
is not released before handoff. In the SAME Immediate transaction, recheck original
input/source/P/G/Session/lock/physical frame and exact CaptureReadOwner, atomically
consume its known-settled capture witness into lifetime/current input admission and
retire only that capture read protection. No gap permits a mutator to reserve/mutate/
settle between capture and admission; a prior idle/version pin is not enough. Drop
or failed handoff retains protection until its exact owned no-effect/settled closure.
A pure-only read reservation is separately revocable by private exact owner CAS or
genuine instance-epoch exclusion after crash: it never launched an effect, and revoked
handoff then fails. An executing capture retains its genuine #60 effects/reservation
and full physical recovery obligation; public pure/readOnly labels cannot classify it.

Pure capture has local nonserializable read authority, not fake native/helper cleanup.
Already-live native/runtime work must have a reviewed continuously enforced compatible
capture relationship which stabilizes governing domains; otherwise capture/admission
returns held/Unsupported. Post-acquisition executing revalidation uses a named reviewed
Consultant-self #60 capture compatibility profile plus the same retained handoff; own
Consultant recognition is not a bypass. Native-mutated target bytes are not silently
recaptured as governing input. External independent user/IDE writes remain outside
application authority; this DB/handoff check DOES NOT detect arbitrary such writes
between capture and wire. Do not claim machine/filesystem secrecy or atomicity against
those writers. Observed stale source at a real validation point still refuses.
Supervisor ownership
is independent of droppable caller futures. Drop only posts bounded in-memory hint;
no blocking Store IO/destructor release. Failed publication retains actual ownership
and attention; in-memory/watch terminal success cannot override retained durable hold.

## 4. Two-sided exclusion and actual effect owners

Use one private exclusion service shared by Consultant lifetime, #19 Task native
operations and #60 runtime effects. Before an executing Git/helper/test/browser/STG/
mutation/hook/delegate action, #60 atomically acquires a nonserializable effect owner
against all holds/full locks. Consultant acquisition, admission and continuation do
reverse checks in the SAME admission transaction. Out-of-lock side effects begin only
with that actual reservation. A mutation lock/active=false row or cached idle result
is insufficient. Actual #60 known outcome plus full profile cleanup/reconciliation
settles its reservation; native cleanup cannot settle an unrelated runtime helper.
Uncertain/partial outcome, crash or failed publication retains it for #14.

A later native owner does not cover Consultant pre-acquisition identity/context Git.
Every executing observation/capture party requires its OWN actual #60 contract before
execution, even if read-only in intent. Composed post-acquisition observations still
possess actual #60 owner/cleanup and continuous compatibility; recognizing their own
Consultant is not a bypass. Strictly non-executing bounded held-state reporting stays
available without Git freshness, Source equivalence or cleanup claims.

A reviewed compatibility relation is profile/effect-specific, symmetric, continuously
enforced during admitted work and through Lost; it is not a boolean readOnly field.
Distinct disjoint Projects remain runnable. Unknown overlapping compatibility refuses.
If either already-running peer becomes Lost, actual resource profile must protect the
other peer or settle it conservatively; blocking only new admissions is insufficient.
Blanket serialization earns no required4+Task/native parallelism acceptance.

## 5. Lifecycle, facts and per-submission gateway

Fresh input, native-originated steer/compact/review/inference, ALLOW/operation grants,
input-capable attach, continuation and authority-granting publications ALWAYS require
current Registered Project and accepted nonterminal applicable Goal under #23. Goal
cancel/fail may close logical lifecycle while retaining holds and denying new input;
Goal completion requires real settlement. Own factual observation, historic DENY and
cleanup have distinct inactive-owner predicates and never grant progression.
Lost is absorbing in ordinary ports; diagnostic writes can only add uncertainty,
with exact owner/operation/input/Session CAS, no release or refreshed authority.

Reuse #6 Project-only native TUI gateway: one upstream actor, exact live native
session, bounded text/decision payload and one unresolved submission. One composed
private owner/per-submission transaction admits before wire; no parallel ledger.
No binding from dead/terminal row seeds. Goal-scoped interactive TUI remains Unsupported
until separately reviewed #6 conformance extends it. Task `rrx attach <task>` is still
required/open under #6/#11/#15/#19/#14 and is not satisfied by this gateway.

Current-input ALLOW uses current source/governing/identity and lifecycle predicates;
DENY pins exact historic pending request/turn and owns no new work. Preserve provider
specific behavior: Claude revoked/Blocked denial can produce no wire; actual Codex
fixed decline for unchanged Waiting may wire with exact own Session/operation guards,
then any attempted Running grant is stopped by current lifecycle checks. Do not turn
this into a generic stale-owner ALLOW or label-based denial permit. Actual producer
migration tests must cover both behaviors. Attach observation-only handles cannot
submit input. Resume needs actual validated prior outcome/full settlement and a new
admission, preserving native UUID/input semantics; unknown cleanup blocks it.

A native turn can end while its server remains live: retain the lifetime. Native
completion, profile resource settlement, lifetime release and Workflow/Goal success
are separate facts. Stop/close drives the real supervisor/profile cleanup. Missing
PID, group leader exit, native interrupt acknowledgement or public terminal label
never proves that required resources/hook/delegate work ended. Preserve native config,
auth/model/effort/hooks/UI defaults; no disabled native features count as acceptance.

## 6. Exclusive successor and recovery of recovery

#14 is the sole private recovery producer. Before ANY fencing/inspection/reconciliation
effect it atomically reserves one exclusive recovery intent and #6/#60 effect owner
for the complete bounded exclusion closure and actual required compatibility
obligations on shared domains, retaining all original holds. Capture exact
former instances/epochs, prior intents/versions and associated partial effects.
Fencing requires actual profile resource evidence and private writer-epoch exclusion;
row/boot/heartbeat/PID labels alone cannot fence actual native/helper resources.
Live legitimate instances conflict unless reviewed #14 explicitly authorizes their
controlled shutdown; no arbitrary PID kill or row-based live-owner reconstruction.

Pending replacement of a prior recovery intent reserves only a private competing
intent and retains all original obligations. It MUST NOT bump/revoke the still-live
prior owner's settlement CAS before genuine authorization/instance exclusion. A live prior may finish normally;
new claimant withdraws only its own proved no-effect pending claim, atomically, without
releasing prior holds. Pending claims are a PRIVATE explicitly pre-effect class: no
native/helper/inspection/fence authority; excluded ONLY from the EXACT prior intent’s
adoption, owned #60 recovery reservation and settlement conflict predicates. They
remain irrelevant to ordinary admission because original holds still block it.
Prior adoption/settlement atomically invalidates claims pinned to superseded prior
versions. A crashed claim provably still in private pre-effect state may be discarded
by that exact prior/successor without physical fencing; public inactive labels cannot
prove this state. Use one checked private pending slot per exact prior intent/version, charged to
reserved recovery headroom; concurrent proposals cannot consume the prior's reserve.
Define Pending -> Authorized -> Fencing -> Adopted explicitly. Pending has NO effect
capability. Authorized transition requires the reviewed #14 sealed NON-LABEL witness
that the exact prior runtime instance is no longer a legitimate live owner, or an
explicitly reviewed authorized controlled-shutdown protocol; atomically exclude its
actual private instance/writer epoch and retain every hold. Candidate witness producer
is an actual kernel-released runtime-instance lease held continuously by the runtime
through its worker lifetime, with exact private lease/instance ownership and genuine
non-executing observation; a missing PID/heartbeat/file/row/boot label is not one.
Its real Linux/macOS owner/lifetime controls and producer review are mandatory before
profile readiness. Slow live prior still holding its genuine lease cannot be authorized
and completes normally. Unknown lease/failure remains pending/attention.

Only this genuine authorization/epoch exclusion allows the claimant to acquire its
OWN retained #6/#60 recovery effect capability under the original exclusive intent
and enter Fencing, inspecting/physically fencing surviving helpers/native resources.
PHYSICAL complete fencing proof is required BEFORE adoption, not before granting the
authorized owned fence operation. Instance-lease death/epoch exclusion is NOT resource
cleanup proof. No third-party unowned fencing is permitted. Every original/prior
recovery effect remains retained through this stage and any crash. Its effect
reservation remains retained thereafter. Settlement/withdrawal CAS pins own exact
rows/identity/current versions and required outcome/cleanup; never compares competing
pre-effect pending rows as a reason to invalidate a live prior. Active adoption still
checks every actual effectful/live/uncertain outside owner and its actual overlap/
exclusion relation; compatible shared peers retain continuously enforced obligations. Competing replacement
claimants CAS on their exact private pending authority; no public successor label.

A crashed recovery's intent, partially fenced resources and partial reconciliation
are themselves retained owners. Successor genuinely fences EACH former instance and
prior recovery owner/effects, then atomically adopts ALL exclusion-connected members covered
by the union of genuine proofs into ONE successor WITHOUT release. Adoption includes
complete current outside holds/full locks and original immutable identities; truncated,
missing proof/live/unfenced NONCOMPATIBLE intersections refuse. Compatible peers
remain outside adoption only through actual continuous shared-domain enforcement
which also governs successor fencing/inspection/reconciliation, never a row exemption. Intersecting adopted members do not
block their own sole successor's inspection, and another proved former instance is
not a conflict solely due to its epoch. Partial adoption cannot start reconciliation.
No ordinary Lost-compatible exemption and no global fence substitute.

Actual #60 adopted-scope inspection/reconciliation runs outside Store lock. Release
only after known current recovery outcome and full required profile cleanup plus
atomic factual settlement/audit. Crash at intent/partial fence or reconciliation leaves
all obligations for another successor. Missing actual #14/#6/#60 port leaves scoped
attention/read-only status, and production profile readiness remains unfulfilled.

## 7. Concrete consumer migration inventory

| Current source / planned producer | Required change before acceptance |
| --- | --- |
| state::put_session/put_session_if_current/put_record_tx and all generic writers | epoch fence, generic Task-free live and owner-bound factual write fences; no identical bump bypass |
| Store::put_project/ensure_project_idle; project::Registry::{add,reconcile,status,remove}, validate/repository_identity | complete all-state physical inventory; mutation owner; pure bootstrap or actual #60 observation before Git |
| git::WorktreeManager::{create,status,ensure_mutation_allowed,lock_review,verify_review,cleanup}; repository_identity/owned_status | reserve BEFORE first check-ref/show-ref/status/hook/helper/filesystem effect; exact owned release/partial-failure recovery |
| context::capture/validate/select and #18–20 packs/gate claims, workflow::Source capture | pure equivalent reads or actual #60 observation; include Consultant pre-acquisition and per-input calls |
| Generic/Grok startup/preflight; pending Claude/Codex consult/TUI and #11 driver | genuine retained owner before any probe/spawn/input; exact private publication/cleanup ports; no caller future ownership |
| #12/#13/#60 verification/Evidence/test/expanded/mutation/browser/STG | own reverse effect reservation and real settlement, continuously protect admitted peers |
| Store::put_goal and #23 Goal completion/cancel/fail/pack publication | same Immediate Tx refuses Completed with ANY unsettled Goal-scoped hold; Cancel/Fail retain holds and deny fresh authority; no public terminal bypass |
| #15 status/TUI/attach | pure non-writing held status; fresh-grant versus factual ports |
| workflow::inputs/load_rules; project::resolve_file/git_metadata/validate_namespace/validate_inputs/validate/effective_config/scoped_file/environment_names | pure equivalent reads or OWN actual #60 reservation BEFORE each executing Git route, including rule-ref-directory rev-parse |
| ProjectRegistry::resolve/list/reconcile and cwd WorktreeManager::status | separate pure held report from executing validation/reconcile; process each Project independently; held/Unsupported A cannot fail B status/removal or become Blocked from an unexecuted check |
| Grok checkpoint verify_git; supervise verify_binding/index_digest at PreSpawn/InSessionBinding/Reconciliation | actual own #60 effect/capture owner and cleanup, stable handoff/compatibility with native hold; native-only owner cannot settle helper |
| #14 recovery; #26 Project registry | actual exclusive intent/fence/full-union adoption; no public-row producer |

Current main has generic Task reservation and #41 owned attempt diagnostics, not these
Consultant/runtime/recovery ports. Unmerged provider/operation plans do not prove
implementation. The pre-source inventory above includes actual current callers; source acceptance re-inventories every call site, including
wrappers/delegates, and remove every unreserved effect route. A safe pure status path
cannot silently substitute for required effectful freshness. Unsupported before effects
is temporary truthful readiness, not final MVP acceptance.

## 8. Actual controls, mutants and gates

Carry every approved requirements verification obligation into source inventory.
Controls use real private acquisition/supervisor/consumer APIs, not seeded SQL/receipts.

| Consumer family | Required positive/negative controls and compiled causal mutations |
| --- | --- |
| lifetime | Session-less owned startup; live server after completed turn; genuine stop/settle positive; omit hold / release on turn/PID/terminal/drop / forge owner from row |
| publication/current grants | exact legitimate observations; conflicting Session writer and publication failure; every submission/ALLOW/input attach/continuation/publication revoked Project/Goal; omit lifecycle/owner/CAS, DENY-to-ALLOW bypass |
| generic stores | every label/role/profile Task-free live insert, identical bump, terminal-to-live and owner factual overwrite refused; remove generic fences |
| physical/reverse consumers | two disjoint Projects progress; nested roots/shared common-dir refuse; live hook/test job versus Consult BOTH directions; Lost job; partial worktree removal; omit root/common-dir or reverse reservation/release ownership |
| capture/status | configured fsmonitor/hooks/filter/textconv on actual registry/status/capture pre-acquisition/per-input; genuine runtime owner and cleanup; pure held status remains available; move executing capture before owner / substitute native cleanup / omit reverse lookup |
| compatibility | actual same-Project compatible profiles and conflict negatives; already-admitted peer protected when either becomes Lost; omit continuous enforcement / trust path/role |
| successor | actual partial removal + Consult + revalidation holds; fence/full-union adopt without release; live/unfenced outside peer negatives; omit physical fence / partial adoption / premature release / Lost-compatible bypass |
| repeated recovery | crash after intent/partial fence AND during reconciliation; next owner fences originals + prior effects; two former instances; slow live prior completes while claimant withdraws no-effect; omit prior fence / revoke prior CAS / lose prior members / row-only replacement |
| bounds/migration | Project A quota/Lost while disjoint B still acquires; retired settled Removed history does not exhaust current scans; actual closure-at-bound refuses ordinary effects but successor recovers; complete index under reactivation/alias/domain merges; actually pre-opened schema3 put_project(Removed)/put_session/put_goal and all private writes fail DB epoch triggers; mutation omits trigger/index/quota-headroom |
| capture handoff | mutator attempts reserve/mutate/settle INSIDE capture-to-admission window; retained protection refuses it and legitimate handoff passes; missing protection/early-release mutants permit intended stale source violation |
| Goal/registry | actual put_goal Completed with live/Lost Goal hold refuses; Cancel/Fail preserves it; held A pure status plus B actual status/removal succeeds; omit terminal hold check or hard-fail cross-Project sweep mutant |
| replacement | claimant inserted between live prior intent and adoption plus crashed pre-effect claimant; prior adopts/executes genuine #60 recovery/settles; one-slot bound; remove pending-class exception/invalidation or grant pre-effect authority mutant |
| authorization/fencing | actual crashed prior with orphan helper: genuine instance lease exclusion -> owned fencing -> full proof -> adopt/settle; slow live prior refuses authorization; heartbeat/PID label authorization and fence-before-owner mutants |
| shared compatible domains | actual same native per-user store, A Lost recovery while B stays live/admissible within its own quota; overlap-to-global-closure or row-only Lost-compatible mutants |

Each credited mutant compiles, crosses real positive prerequisites and reaches the
intended actual consumer assertion; setup refusal/timeout/helper-only failure earns
no kill. Use isolated temporary repositories, preserve all failures and exact restored
source/tree passing controls. Required actual native #5/#6/#7 profiles/defaults,
#14 recovery and compatible multi-Project/4+Task fixtures are separate mandatory
conformance, not supplied by controlled subprocess tests. Run impacted Store/Git/
registry/context/Workflow/native regressions, fmt/all-target Clippy/debug-release
builds and Linux/macOS CI; record actual checkout/parents/source blobs separately
from trigger SHA. Two independent immutable design and source reviews precede
acceptance. Update README/master design only to actual qualified behavior; #16 final
Goal/review/approval/multi-Project/Context ON/OFF remains open.

Design1 findings were verified against actual Store::put_goal, Registry::{resolve,
list,reconcile}, project::resolve_file/git_metadata and Workflow::inputs/load_rules,
plus Grok checkpoint/reconciliation Git consumers. Design2 corrects capture ABA with
retained private read protection and atomic handoff; cross-kind physical/profile domain
scope and aliases; scoped indexed capacity/recovery headroom/settled history retirement;
actual generic Goal terminal/registry sweep consumers; database-enforced old-connection
writer epoch; pending replacement pre-effect noninterference and typed Consult entry.
All controls/producer/source/native gates remain mandatory and unimplemented.

Design2 residuals were independently verified: overlap-only connected closures would
couple shared HOME/config across Projects, and requiring physical fence proof before
owning the fence operation was circular. Design3 separates actual exclusion edges,
per-instance exclusive resources and continuous shared-domain obligations; genuine
instance authorization/epoch exclusion precedes owned physical fencing, whose full
proof precedes adoption. Typed Task-only requests, pure capture revocation, exact
effective Git config/environment inventory and SQLite predicate flags/self-tests
are explicit. No actual instance lease/resource backend, schema or native proof is
introduced by this design; all mandatory producer/native/recovery gates remain open.
