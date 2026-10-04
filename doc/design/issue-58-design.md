# Issue 58 design: retained Task-free native consultation ownership

Status: Design5 proposed; Requirements8 approved at8facb8d. No source/profile,
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
of the actual predicate, not an automatic security label. Legacy schema3 runtimes
never acquired a new instance lease. Upgrade does not claim a universal old-helper
death proof from SQLite, historical rows or a shutdown assertion. Under actual reviewed
exclusive connection coordination, drain/recover every DURABLY RECORDED legacy owner
(nonterminal/Lost Sessions, unsettled locks and active/unbound/evaluating Workflow
claims) through the genuine #14 controlled legacy handoff. Atomically install the
composed epoch triggers/current-only writers and user_version rejected by old open
paths before any new-epoch admission. Actual pre-opened old writer and reopen-race
controls are required. A failed preflight/schema migration leaves bytes unchanged;
no preparatory user_version commit is silently reclassified as a failed atomic upgrade.
Current-source migration must align with #19's stricter old Workflow/irreversible-tail
inventory, not omit a known owner simply because a generic state label is terminal.

Documented legacy residual: unrecorded helper effects of historical schema3 runs have
no private provenance/bridge owner and are NOT certified drained, adopted, settled or
contained by this upgrade. Absence of a row does not prove their death. They remain
outside new retained-owner authority like independently started user/IDE activity;
upgrade/readiness makes no complete historical or machine-wide cleanup claim. This
residual never clears a known live/uncertain recorded hold or permits it to overlap
new conflicting work. Supported quiescent legacy data with genuine recorded-owner
drain may upgrade; missing real drain for recorded uncertainty stays unready. Required
control upgrades schema3 historical data with no actual recorded live workload, and
refuses unchanged when a genuine recorded hold lacks settlement. Real native ownership,
restart recovery and supported legacy upgrade are mandatory source/conformance gates;
the residual is explicit, not a positive resource-cleanup certificate. Failed migration preserves schema/data/history
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
Actual executable identity is part of the supported Git profile, not argv name.
Publish a reviewed per-binary identity table binding opened executable/build digest,
compiled system-config prefix and concrete shim resolution inputs. For macOS /usr/bin/git
this includes actual selected developer-directory identity and DEVELOPER_DIR/xcrun
routing; no guessed /etc/gitconfig or PATH name is a complete inventory. Standard
installed Linux/macOS profiles need real producer/conformance before MVP acceptance.
Unknown/custom builds stay Unsupported before Project effects. A separately reviewed
runtime-global owned60 config-introspection path may bootstrap a supported executable
only with its own complete effect inventory/conflict reservation/cleanup; it cannot
execute unowned while merely claiming that config reads are harmless. This producer
is proposed, not available or evidenced by the table's existence.

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
in any exclusion-connected closure, including all former instances,
recovery intents/partial effects and additional128 reserved successor/cleanup members.
Admission computes actual merged exclusion closure and required profile worst-case
recovery headroom BEFORE effects, refusing if ordinary admission would consume that
reserve or exceed4096. Each permitted recovery effect counts its actual bounded new
owners/intents in that same invariant; actual successors adopt and consolidate fully
fenced prior members without dropping obligations. Unknown/unfenced members are never
compacted. Finite exhaustion holds with attention; real at-bound admission-refusal plus
successful actual successor recovery controls are mandatory. Unlimited arbitrary
crash chains are not promised by finite storage, but admitted ordinary workloads and
required crash-at-fence/reconciliation cases must retain adequate recovery capacity.

There is NO runtime-global retained-owner pool which Project A's uncertainty can
exhaust to deny an already-registered exclusion-disjoint or genuinely compatible B. A quota is charged to
its exact Project plus affected exclusion closure; cross-scope transfers/registration
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
EVERY capability holder enters the private registry when minted: pure capture,
pre-transfer Consult setup, actual runtime/native supervisor and all nested workers.
Each private unique holder cell has exact generation/scope and bounded preallocated
state. A never-effect-authorized holder's Drop seals an I/O-free atomic no-effect
abandonment witness; reliable bounded registry scanning consumes it outside all locks
through exact Store owner CAS. Notification is only a hint, not sole progress. This
returns pure/pre-transfer abandoned capacity without restarting disjoint live work.
If effect authority was ever granted, Drop cannot mint no-effect proof; retain its
full structured workload for the authentic ended/revoked/#14 path. A public state or
strong-count guess is not a witness. Actual cancel-at-capture/acquire-to-transfer and
lost-wake controls must prove genuine release versus held effect-bearing state.
A Session-less ReservedSetup hold already excludes competing effects. Actual Task-free
entry is typed start_consult(owner: ConsultLifetimeOwner, ...), consuming the owner
by value. ALL native-effect AgentAdapter methods migrate to private-constructed
Task-scoped types with a REQUIRED validated Task: start/resume, attach, checkpoint,
submit_approval, stop, workload_release, start_native_goal and resume_native_goal. Use TaskScopedLaunchRequest,
TaskScopedSessionRef, TaskScopedPreparedInput and TaskScopedNativeGoalRef as applicable;
public SessionRef/PreparedInput/NativeGoalRef fields, Scope::project/goal and JSON cannot
construct these authority-bearing types. Native goal references alone are execution
references, never an owner, even when they have no Session ID or no Session write.
Direct inherent, trait/default and registry Arc<dyn AgentAdapter> consumers, ALL
providers/fixtures migrate together. Task-free grants/checkpoint/input attach/continue/
native-goal work use only private owner-bound handle methods which consume or borrow
the genuine ConsultLifetimeOwner, or a separately reviewed genuine native-Goal owner;
until that method/producer exists it is Unsupported by signature before effects.
Issue25 remains optional and cannot obstruct required Consult/Task entry support.
Observation-only in-memory status/subscribe/usage/native_goal_status may retain public
references only if their actual implementation performs no external effect. Stop and
release never grant work: require the actual retained owner for physical stop/settle,
and keep public output release distinct from workload release. Probe execution requires
its actual owned #60 port too. No Session write is needed to enforce this typed boundary.
Compile-fail doctests and real zero-effect direct/registry rejection controls cover
EACH method, including Task-free attach/stop/workload release and Goal-scoped native-goal start/resume; public
registration/labels or an unsupported default implementation are not a positive owner.

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

#14 is the sole private recovery producer. ONE Pending -> Authorized -> Fencing ->
Adopted protocol governs EVERY former owner in the complete exclusion closure,
original Lost/uncertain supervisors and prior recovery intents alike. First-level
recovery cannot acquire a #6/#60 fence effect owner merely from a Lost row. Pending
reserves only the private exclusive no-effect proposal with original holds retained;
capture exact supervisor/instance/epoch, intent versions and partial effects. EVERY
covered member needs genuine authorization and exact private epoch exclusion before
any claimant fencing/inspection/reconciliation effect capability is minted.

Authorization has two reviewed genuine producers. Across a dead runtime, an actually
released continuously held instance lease excludes its exact formerly registered
supervisors. Inside a still-live runtime, the actual private supervisor registry must
retain and observe the COMPLETE structured in-process workload: top supervisor,
every nested async task, blocking worker, callback mutation and output/drain worker.
Reserve its private registry slot BEFORE scheduling; the worker cannot perform any
effect until synchronous handoff stores the returned JoinHandle and opens its genuine
private start gate. Failed handoff retains the slot and unopened/owned worker; no
spawn-then-enroll effect gap or queue label can substitute for this handshake.
Supervisor panic/caller Drop cannot detach its children by dropping the only handles.
Authorization requires all exact owned in-process jobs joined, THEN synchronous
revocation of every admission capability/queued grant path. Abort request or top-task
join alone is insufficient; a running spawn_blocking worker is non-abortable and held
until its actual join. Unregistered/detached/unjoinable work refuses authorization and
needs actual controlled stop/restart, never a thread/process-number death inference.
Transfer retained resource handles through one nonserializable single-owner handoff
to the exact CO-RESIDENT #14 claimant. Authorized CAS pins that claimant's instance,
identity/generation and genuine received handle extent; a remote instance cannot
present this live-registry witness or reconstruct unreaped leader handles from DB.
Dead-instance recovery uses its separately reviewed kernel/profile proof instead. Its sealed generation-
bound ended/revoked witness is not a persisted label, caller bool or heartbeat. Registry
revocation is I/O-free. Use preallocated per-owner cells and a non-poisoning lock with
checked nonpanicking updates; no user callback/allocation/Store/native call under it.
An interrupted entry is conservatively invalid/held without mutating disjoint cells.
A registry-wide integrity failure reports actual restart-required readiness, never
claims B progress; actual panic-under-entry-lock controls must prove B remains live
or truthfully report that broader failure. The immutable witness
then supplies an outside-lock Immediate CAS excluding that exact supervisor epoch;
no gap can resurrect its revoked capability. Other supervisors/Project B remain live
and their epochs unchanged. Complete in-process join is NOT external helper/native cleanup: after
authorization, the claimant owns the real fence operation, then proves full physical
settlement before adoption. Unjoined/wedged live supervisors cannot be recovered from
labels; use a genuinely reviewed controlled stop/restart protocol. Recovery must not
require whole-runtime shutdown for an ordinary ended supervisor's Lost hold.
Actual registry/lease/#14 producers, supervisor-scoped epoch and resource-handle
transfer remain unimplemented source gates. Physical compatibility includes fencing
effects and protects already-live peers. No arbitrary PID kill/reconstructed owner.

Pending replacement of a prior recovery intent reserves only a private competing
intent and retains all original obligations. It MUST NOT bump/revoke the still-live
prior owner's settlement CAS before genuine authorization/instance exclusion. A live prior may finish normally;
new claimant withdraws only its own proved no-effect pending claim, atomically, without
releasing prior holds. Pending claims are a PRIVATE explicitly pre-effect class: no
native/helper/inspection/fence authority; excluded ONLY from EACH covered exact original owner or prior intent’s
owned cleanup/recovery reservation, adoption and settlement conflict predicates. They
remain irrelevant to ordinary admission because original holds still block it.
Original owner cleanup/settlement or prior adoption/settlement atomically invalidates
claims pinned to its superseded versions. A first-level Pending proposal cannot block
a live original HeldUncertain owner from completing its own legitimate cleanup. A crashed claim provably still in private pre-effect state may be discarded
by that exact prior/successor without physical fencing; public inactive labels cannot
prove this state. Use one checked private pending slot per exact prior intent/version, charged to
reserved recovery headroom; concurrent proposals cannot consume the prior's reserve.
Define Pending -> Authorized -> Fencing -> Adopted explicitly. Pending has NO effect
capability. Authorized transition requires the reviewed #14 sealed NON-LABEL witness
that EACH exact former supervisor is no longer a legitimate live owner through
the unified instance-lease or live-registry producer above, or an
explicitly reviewed authorized controlled-shutdown protocol; atomically exclude its
actual private supervisor/instance writer epoch and retain every hold. Candidate
cross-instance witness producer
is an actual kernel-released runtime-instance lease held continuously by the runtime
through its worker lifetime, with exact private lease/instance ownership and genuine
non-executing observation; a missing PID/heartbeat/file/row/boot label is not one.
Its real Linux/macOS owner/lifetime controls and producer review are mandatory before
profile readiness.
Candidate lease uses a dedicated private open-file-description flock on a verified
local supported filesystem, nonblocking exact conflict checks and atomically opened
CLOEXEC descriptor; no process-associated POSIX F_SETLK record locks. Never expose,
duplicate/unlock the lease FD or upgrade/downgrade its lock during the lifetime.
Same-process contenders use a separately opened description plus registry identity
checks; a duplicate FD is not an independent conflict proof. Rust/private ownership
retains the descriptor until exact runtime shutdown; native/helper spawn must not
inherit it across exec, and any fork-before-exec path must close inherited duplicates
without unlocking the parent's description. Actual both-host controls cover unrelated
same-file FD close, separate-open same/cross-process conflict, intentional dup/fork
inheritance, exec orphan and crash. Mutants removing CLOEXEC or substituting POSIX
record locks must reach the false witness/extended-lease consumer. A wedged in-instance
worker still holding live capability is not authorized by timeout; ended/revoked exact
workers use the registry route while disjoint B continues. Advisory leases prove only
cooperating runtime-instance authorization, never native resource containment.
These constraints follow primary [Linux flock](https://man7.org/linux/man-pages/man2/flock.2.html),
[Apple flock](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/flock.2.html)
and [Apple fcntl](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fcntl.2.html)
semantics; archival API docs are not installed-host conformance evidence. Slow live prior still holding its genuine lease cannot be authorized
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
| AgentAdapter start/resume/attach/checkpoint/submit_approval/stop/workload_release/start_native_goal/resume_native_goal; public SessionRef/PreparedInput/NativeGoalRef constructors | required private Task-scoped types for every generic effect entry; Task-free effects only actual owner-bound methods, observation/stop/output-release separated and never grant authority |
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
| authorization/fencing | EVERY first-level/replacement member gated: crashed prior orphan lease -> owned fence -> proof -> adopt; live runtime ended/revoked exact supervisor recovery while B stays live; unjoined/slow live refuses; omit original-owner authorization/registry revocation, heartbeat/PID label or fence-before-owner mutants |
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


Design3 reviewers independently verified one shared Medium (untyped public native-goal,
attach/checkpoint/approval trait routes) and a second Medium (first-level recovery missed
authorization, instance-only witness stopped disjoint live peers). Design4 extends the
type boundary to every actual effect entry and unifies original/replacement recovery
with supervisor-granular genuine registry and dead-instance lease witnesses. Stale
overlap quota language is corrected; lease descriptor/fork/record-lock controls and
truthful legacy-drain readiness are explicit. Actual source/backend/legacy bridge and
native conformance remain mandatory and absent. Requirements8 is unchanged.


## Complete in-process and legacy acceptance controls

Actual Grok callback/source60 scoped mutating spawn_blocking work starts, then the
supervisor panics/aborts: original registry still owns the worker, authorization is
refused until actual join; no claimed cleanup/adoption while it can mutate. A
top-supervisor-only join mutant must expose that continuing effect. Same-instance
claimant with transferred exact handles progresses after full join; another runtime
presenting that witness refuses before fence/adoption. Kill claimant-binding omission.
Pure capture cancellation, pre-transfer Consult drop and lost abandonment notification
converge through genuine no-effect cell/owner CAS without runtime restart. An effect-
bearing holder remains held until full settlement. Original uncertain owner cleans
up/settles despite inserted first-level Pending; stale proposals invalidate atomically.
Per-entry panic isolation must preserve disjoint B or report true broader readiness.
Typed stop/workload-release rejection is tested alongside every native grant entry;
public output-only release is neither process stop nor reservation release.

The in-process constraints follow Tokio [JoinHandle](https://docs.rs/tokio/1.53.1/tokio/task/struct.JoinHandle.html)
and [spawn_blocking](https://docs.rs/tokio/1.53.1/tokio/task/fn.spawn_blocking.html)
semantics; locked Cargo dependency1.53.1 source confirms detachment and non-abortable blocking
workers; installed-host controls remain required.
Top-level completion or an abort request cannot certify a detached blocking writer.
Design4 actual reviewers identified these verified design gaps, with owned cleanup
complete. Proposed Design5 changes no source/requirements/schema/native readiness;
all actual #14/#19/#23/#60/registry/lease/native conformance gates remain mandatory.
