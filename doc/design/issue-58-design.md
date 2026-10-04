# Issue 58 design: retained Task-free native consultation ownership

Status: Design1 proposed; Requirements8 approved at8facb8d. No source/profile,
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
including generic/private APIs and already-open connections checks this epoch under
the same transaction; migration preflight drains actual legacy owners and requires
reviewed #14 recovery for uncertainty. Failed migration preserves schema/data/history
bytes. Terminal labels, inactive rows or an epoch retag do not prove physical fencing.
#58 cannot precede #19 Task-scope fences or #60 reverse producers in production.
Candidate component tests can run in isolated stores; missing ports keep affected
production profiles unready, and do not waive mandatory Consult/attach/recovery.

## 2. Physical scope and bounded complete inventory

Represent `PhysicalScope` with canonical Project root, canonical common Git directory,
registered repository identity and the complete effect domains of the actual profile.
Two roots conflict if equal or ancestor/nested; any shared common directory conflicts
unless actual reviewed continuously enforced compatibility proves those effects safe.
Compare all retained holds and reservations across ALL Project states, including
Blocked/Removed and uncertain historical instances. IDs/worktree names/role labels
never imply physical independence. Project root/identity/registration mutations use
this same inventory and cannot introduce overlap around an existing owner.

Bootstrap ONLY bounded non-executing in-process path/canonical metadata and Git
layout reads: plain primary .git directory or a validated bounded gitdir/commondir
layout. Bound each metadata file to16KiB, paths to4096UTF-8 bytes, link/file hops to32
and aggregate bootstrap data to256KiB. Unprovable/changing/unsupported layout refuses
before external Git; no guessed candidate reservation around a Git-first probe.
Source equivalence is a separate condition: raw files cannot impersonate filtered/
attribute-transformed Git content. Pure capture returns typedUnknown/refuses where
configured transforms prevent equivalent current source. No implicit config bypass.

One Immediate transaction reads the complete coherent exclusion inventory plus
current P/G/T/Session/full scoped lock versions applicable to the action. Proposed
bounds:4096 registered/history Project identities;16384 active retained owners and
reservations;4096 members per recovery closure;64 physical domains per owner;
64KiB encoded owner/receipt;64KiB bounded audit body;128 submissions per lifetime,
256 operation references;nesting16. Every counter/version uses checked arithmetic
within signed-i64 storage range. Validate total serialized size before insertion.
Hit any bound: explicit Capacity/NeedsAttention BEFORE acquisition/effects; never
LIMIT/page/truncate a scan and infer idle. Recovery over these bounds remains held,
with bounded non-executing attention. No overflow clamp or unbounded provenance list.
These are initial finite policy limits, not OS/RAM/performance proofs.

## 3. Persisted owner and transaction API

A private ownership table (not generic Record) stores immutable owner ID, physical
scope, Project/optional Goal, Task absent, runtime instance/epoch, registered profile,
actor/mode, input/source pins, actual native identities when known and checked version.
Store minimal allowlisted IDs/digests/version numbers/reason codes, never environment
values, prompts/native transcripts, credentials or raw configuration. States distinguish
ReservedSetup, OwnedLive, HeldUncertain and Settled; native turn state is separate.
A Session-less ReservedSetup hold already excludes competing effects.

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

No native/Source/Git call occurs under SQLite/SharedStore locks. Capture immutable
expected action/frame under lock, perform only genuinely reserved or pure work
outside, then atomically recheck the ORIGINAL expected frame. Stale results hold/refuse;
never refresh a frame to make already-executed effects current. Supervisor ownership
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
for the complete bounded physical closure, retaining all original holds. Capture exact
former instances/epochs, prior intents/versions and associated partial effects.
Fencing requires actual profile resource evidence and private writer-epoch exclusion;
row/boot/heartbeat/PID labels alone cannot fence actual native/helper resources.
Live legitimate instances conflict unless reviewed #14 explicitly authorizes their
controlled shutdown; no arbitrary PID kill or row-based live-owner reconstruction.

Pending replacement of a prior recovery intent reserves only a private competing
intent and retains all original obligations. It MUST NOT bump/revoke the still-live
prior owner's settlement CAS before actual fencing. A live prior may finish normally;
new claimant withdraws only its own proved no-effect pending claim, atomically, without
releasing prior holds. Failed withdrawal remains owned/held. Competing replacement
claimants CAS on their exact private pending authority; no public successor label.

A crashed recovery's intent, partially fenced resources and partial reconciliation
are themselves retained owners. Successor genuinely fences EACH former instance and
prior recovery owner/effects, then atomically adopts ALL intersecting members covered
by the union of genuine proofs into ONE successor WITHOUT release. Adoption includes
complete current outside holds/full locks and original immutable identities; truncated,
missing proof/live/unfenced intersections refuse. Intersecting adopted members do not
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
| #15 status/TUI/attach; #23 Goal completion/cancel/fail/pack publication | pure held status; fresh-grant versus factual ports; no logical close release |
| #14 recovery; #26 Project registry | actual exclusive intent/fence/full-union adoption; no public-row producer |

Current main has generic Task reservation and #41 owned attempt diagnostics, not these
Consultant/runtime/recovery ports. Unmerged provider/operation plans do not prove
implementation. Source acceptance must enumerate every actual call site, including
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
| bounds/migration | every proposed cap/checked overflow refuses before effects; complete scan under concurrent registration; old open generic/private connection writes fenced; legacy uncertainty migration fails with byte-identical history; remove scan completeness/epoch fence |

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
