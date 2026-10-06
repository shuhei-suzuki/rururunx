# Managed Native version-helper integration design

## 1. Status, scope and dependencies

This is a proposed, STRICT implementation supplement to
`issue-43-native-preparation-integration-design.md`, including its original
custody and transport amendments. It does not replace the complete Binding
contract or enable Native availability. Production implementation of this
increment waits for independent design review and actual producer source review.
There is no authenticated CLI, four-Task, OS or process-death qualification.

This revision explicitly supersedes only frozen `b130e1ec735925dc1da0c1871229894cfdd6dfa2`
§6's blanket exact-image `managed_effects` guards and §7's consequent new-migration
prerequisite, including references to those obligations in §4/§8. It uses existing10
with genuine owned proof and strict original inventory/CAS. All installed10 guards,
private owner/input/Session/binder rules, complete #19-equivalent preparation,
permissions, Task-version invariants and static preflight remain mandatory.
The original HOW and its closed design reviews remain historical evidence.

The source baseline is `10a4af99ee788dee6cf5e4065be986934559cfd6`:
actual Root jobs create EMPTY custody before marker/start; actual selected Native
start installs its same-Unit actor and independently saves the original readiness
plan before Immediate. The only write is allocated1→preparing2. Errors retain
actor/plan/known commit and revoke preparation; they do not retire a Unit or
release its gate. This source received two independent component-only reviews.
Its fixed formal fmt/check/build passed; Clippy and full library regression are
RED. No genuine first-stage positive was reached. These are separate open gates.

The next increment adds exactly one host-native `<original program> --version`
helper for that original operation, plus bounded original resource-profile
qualification. It adds no Git helper, quota admission, Session, native transport,
input, ACK or prepared-to-Core proof. A successful version probe remains a
private preparation observation. The remaining start path still refuses.
Existing mandatory hooks/settings, readonly artifact/graph verification and
full #19-equivalent prepared-input completion remain later conjuncts.

There is no security sandbox: profile/command checks are cooperative facts,
not protection against an Agent intentionally changing files or escaping a group.
Auth/settings stay inherited. rrx does not read, copy, transfer or serialize
credentials; this probe does not run an authentication or inference command.

## 2. Actual consumers to replace or preserve

Source references below are to the pinned baseline, not moving worktree bytes.

| Current consumer | New protected consumer |
| --- | --- |
| `execution/native.rs:319–329`, generic profile read and command construction | Private actor-qualified finite profile/command plan, outside Store/admission |
| `native.rs:332–365`, generic helper reserve and child spawn while Store is held | Exact protected intent commit, release Store, SAME actual admission retained for synchronous raw-child creation/custody |
| `native.rs:361–390`, local HelperGuard, generic capture/receipt and version parser | Independently retained same helper plan, eager raw-child capture and sealed owned observation before optional persistence |
| `process.rs:26–34`, raw child exists before fallible PID checks | New retained raw-child adoption API; existing legacy spawn is unchanged |
| `process.rs:143–174`, generic ExecutionAuthority fence | Private same actor/current/Driver/source/readiness fence; no generic fallback |
| `owner.rs:124–155`, HelperGuard Drop updates receipt from ID | Protected helper abandonment retains originals and factual unknown; never uses this generic guard |
| `resources.rs:317–329`, read-all then size check | Read at most64 KiB+1 before decode; strict duplicate/budget checks and original digest/namespace qualification |
| `native/preparation.rs`, known readiness commit followed by deliberate unavailability error | Continue inline into this private version-only stage before returning; retain the final no-Session/no-transport refusal |

The old protected start cannot reach its generic helper branch. Its standalone
public ManagedInput and legacy refusal gates remain. Protected route replacement
is an explicit actual selected-vtable continuation of the SAME custody/actor,
not a `native=true` flag, callback, new public authority mode or current-row lookup.
The actual start saves its known readiness commit and releases its readiness
admission guard. It then invokes this private continuation inline, before it
returns an error. It does not call start again,
claim custody again or reopen an actor already abandoned by Root. Reconciliation
after returned error confirms facts only and never starts a new helper.

## 3. Concrete private types and custody

```rust
// Names describe planned private ports, not existing completed APIs.
struct NativeVersionHelperPlan { /* SAME actor/ready commit/current; exact images */ }
struct NativeVersionHelperCustody { /* original plan/raw child/observation */ }
struct NativeVersionObservation { /* private actual capture + qualified version */ }
struct NativeHelperSettlementPlan { /* SAME owned observation; exact receipt CAS */ }

impl NativeSessions {
    async fn prepare_phase_version(
        &self, original: Arc<NativePreparationCustody>
    ) -> Result<()>; // still no NativePhaseStart::Launched
}
impl Store {
    fn plan_phase_version_intent(runtime: &RuntimeOwner,
        original: &Arc<NativePreparationActor>,
        ready: &Arc<NativePreparationCommit>)
        -> Result<Arc<NativeVersionHelperPlan>>;
    fn reserve_phase_version_intent(&mut self,
        original: &Arc<NativeVersionHelperPlan>,
        admission: &PhaseEffectAdmissionGuard) -> Result<NativeHelperIntentCommit>;
    fn record_phase_version_observation(&mut self,
        original: &NativeHelperSettlementPlan) -> Result<()>;
}
```

Only the actual selected Native continuation creates a helper custody/command
producer. Only the bounded actual capture/parser issues a version observation;
DTOs, receipt rows, effect IDs, parsed version text supplied by callers and SQL
cannot construct it. Store issues intent-known-commit only from this same plan.
`NativePreparationCommit` is retained as a SAME Arc in original custody; making
this actual known observation shareable does not add a constructor or grant.

Before first intent Immediate, original preparation custody installs exactly one
helper custody plus its SAME original plan and eager capture-ownership guard.
The helper custody owns actor/plan/observations as siblings. Its inherited strong
path is actor → PhaseLaunchParts → MarkerPublicationRetention → PhaseSupervisor.
This original launch custody is intentional and is not replaced by Weak merely
to remove that edge. The helper adds no strong return edge to outer preparation
custody, NativeSessions, Runtime, registry or its own task/handle. Queue retention
alone does not establish a Runtime/job return edge; check the complete new graph
in the eventual source.
The actor keeps its existing Weak original-custody link. The Root job retains
preparation custody independently of the caller/start future.

Capture's independently owned task holds the inner helper custody; a separate
sibling task handle does not return through the task to its own outer registry.
The eager ownership guard is constructed before spawn/first poll. No local
future, receiver, generic HelperGuard or JoinHandle Drop is the sole owner of
raw child, original plan or observed result. The same-Unit guard stays held.

## 4. Plan and intent before effects

After releasing the readiness guard, outside ALL Store/queue/source/custody/
admission locks, qualify the selected original absolute
program and current directory, bounded physical resource profile and original
Unit/profile identity. Read profile bytes with a64 KiB+1 sentinel, strict bounded
decoder and complete encoded budget; require the original digest and canonical
per-Unit temp/output/cache/tool paths. Capture no auth/settings values. Preserve
inherited environment and required hooks; prepare only the existing resource
overlay and bounded Git-config-count overlay outside Store. Owner-assigned
namespace/discovery identifiers are not copied native authentication credentials,
but RRX_PROCESS_COOKIE/RRX_RUNTIME_SOCKET have tool-IPC sensitivity. The probe
gets no IPC/tool grant. Cookie, socket path or same UID never independently grants
later protected tools: those require actual current registered invocation, exact
allowed action/input/Unit and private Native permission. Current generic marked
tool authority stays refused; no cookie secrecy or same-user containment claim.

This is no new full prepared-input certificate. Original readonly reviewer
snapshot/artifact and hook qualification are mandatory later. This version-only
increment cannot quietly substitute its physical profile checks for those ports.
Executable-path checks cannot eliminate host package replacement races and do
not certify that arbitrary hook behavior is safe.

Use the SAME actual actor, known readiness plan/commit, original launch/source
seal and selected port. A separate coherent bounded snapshot captures full
preparing2 readiness, original unregistered owner/input, current Unit and current
Workflow successor, with epoch/generation/P/G/T/Context/full locks. The Root
current/source/Driver-live conjunction is mandatory in the final SAME Immediate.
The actor must be unrevoked for a new intent. No hash/encoding/path work or
Source/Queue lock occurs while SharedStore is held.

One helper ID is allocated once by this actual Native producer and saved in its
plan before the transaction. Command is exactly the original program, one
`--version` argument, original worktree and original resource cookie. There is
no caller-provided argv/purpose/target. Effect kind is `native_phase_version`.
Persist exactly the existing nine top-level ManagedEffect fields: id, unit_id,
scope, kind, idempotency_key, expected_target, state, receipt, version. Unknown
fields remain refused by strict readers and startup epoch fencing. Pending
state/version are Pending1 and receipt is the existing empty string map. Canonical
safe version-purpose identity/digest summaries fit the existing idempotency_key
≤256 bytes and expected_target≤4096 bytes. Complete original operation/pair/origin/
epoch/generation/profile/command facts remain in the retained private plan, never
new top-level JSON fields. Settlement uses only capped safe string-map facts.
Persisted fields are bookkeeping, never Native authority. Complete encoded body
ceiling is8 KiB, including receipt/framing; no raw environment/output/errors.

After saving SAME helper custody/plan, acquire a NEW guard from the SAME original
launch's actual admission object. Stop during outside-lock planning refuses at
acquisition/final checks. Sameness means this actual object/launch, not one guard
held across both stages. Reserve one pending `managed_effects` image with a NEW
private INSERT from the sealed plan, under same actual admission and existing10
SQL/writer guards. Preserve ordinary scope/index identity/bounds and bootstrap/
command-only verifier exclusions plus actual current/source/Driver/fullUnit/pair/
readiness/unrevoked actor checks. This does not call reserve_execution_helper or
generic reserve_effect_tx; their kind allowlist and marked-scope refusal stay.
No effect-table permit, DDL or generic-authority exemption is added. Recheck
complete original images and absence
of the SAME helper ID/idempotency identity; commit before process creation.
Task, Workflow, Context, Driver, Source, Session, input, owner and readiness stay
unchanged. Count this helper once, never rotate IDs on an error. The full HOW's
≤32 helper-intents-per-original-operation ceiling remains; this increment permits
one version intent only. Bounded reads use a sentinel, not unrestricted COUNT.

The original plan saves a complete indexed Unit effect inventory, not just pending
rows or selected receipt fields. A two-pass coherent read accepts at most256 rows,
body≤8 KiB and WHOLE encoded inventory≤2 MiB inclusive of ALL copied columns,
framing and header. A257th matching row/oversize refuses before string extraction
or effects. First pass uses typeof and length(CAST(column AS BLOB)) with checked
arithmetic before copying text/decoding: five identity TEXT columns are canonical
36-byte IDs; idempotency_key is nonempty≤256 bytes; state is an allowed≤9-byte
token; version is positive/in range; body is UTF8≤8 KiB. The retained inventory
format charges eight TEXT raw byte lengths plus eight8-byte length prefixes,
version as an8-byte integer, an8-byte row tag and16-byte inventory header. SQL
rowid may order the scan but is not an additional effect identity or planned
postimage field; match the complete indexed nine columns by original effect ID.
This complete length-framed representation is immutable. Other serialization or
copies need their own explicit bound; escaping/metadata is never uncharged.
Only a complete successful first pass allows second-pass extraction in the SAME
read transaction. Second pass repeats shape/length checks before extraction,
compares all indexed columns/exact bodies/ID/idempotency/scope, and uses strict
bounded decoding to reject duplicates/index-body disagreement. Body8 KiB and
inclusive2 MiB are independent:256 maximum-size bodies plus metadata do not fit.
Existing original Source preparation may have recorded Git helpers. Their rows
are nongrant bookkeeping beside the genuine accepted Source seal, not a substitute
for that provenance. Accept only its finite pre-Session Git-helper baseline with
confirmed/resolved state and the one actual new version intent; unexpected kinds,
new IDs or unresolved baseline effects are Held even if another row says confirmed.
This increment admits no delegated action. The baseline cannot supply the new
version observation or prove that a missing child/dispatch was never created.

The pre-intent baseline is at most254 rows; version postimage is at most255,
reserving at least one later native-input slot under its unchanged count<256
guard. Other required helper/transport intents need separate finite headroom
and cannot borrow this reserved slot. This does not certify all later preparation
fits. The intent transaction checks
that complete SAME saved baseline set and inserts
the planned helper. Every subsequent private fence/settlement/continuation checks
the corresponding exact expected set, including its pending or known settled
helper image. No current-row refresh repairs drift. This preserves the original
inventory across preparation while allowing unrelated Tasks' journals to change.

Existing10 has no Unit-leading managed_effects index. LIMIT257 bounds matching
rows, not unrelated history visited. Each complete inventory routine (both passes
and final transaction comparisons) additionally uses a scoped SQLite VM progress
budget. The chosen planned implementation enables rusqlite's `hooks` feature and
uses safe Connection::progress_handler. Local pinned rusqlite0.37.0 has hooks=[]
and that public safe API in hooks/mod.rs; hooks adds no optional dependency.
Cargo.lock/local cache contain this version and its ordinary dependencies.
Baseline enables bundled/functions ONLY: hooks and the bounded actual consumer
are NOT currently provided. Enabling/qualifying them is mandatory later source
work, with no schema/index/DDL change; this HOW has not compiled or tested it.

One inventory routine shares a checked counter across all statements/passes:
callback interval1000 VM operations; interrupt by the1000th callback. Callback
does no SQL, lock acquisition, allocation, I/O or side effects. Arm before
statement preparation/stepping, retain the budget through complete comparison,
and reset progress_handler(0, None::<fn() -> bool>) after dropping statements on
EVERY success/error/unwind path before connection reuse/rollback. A private
lexical reset guard forbids nested progress scopes or replacing an installed
hook. Independent read-only planning connections use the same strategy; final
SharedStore exclusive connection access lasts this synchronous routine only.
Interruption is typed InventoryWorkLimit/Held, discards ALL partial rows/proofs,
rolls back any intent and preserves original custody. It cannot substitute a
count/own-row/head check or authorize spawn. Reset before mandatory nongrant
closure SQL, which retains its parent finite budget instead of an exhausted
helper counter. No await, path checks, hashing or encoding under SharedStore.

This bounds approximate SQLite VM work, not wall time, OS I/O, SQLite internal
allocation or all parent-validation/transaction work; those costs are separate.
Large unrelated history can cause honest refusal even for a tiny selected Unit.
No global history cap, silent pruning or new access index is introduced.50ms is
a requested non-overlapping cadence, not a response-time guarantee: complete
≤2 MiB set comparison can repeat20 times/s (about40 MiB/s per helper before
separate costs), without a backlog of fence tasks. Final eligibility, each tick,
settlement and future consumers retain complete set checks and the same budget.
Actual statement/interruption/reset and multi-Task Store contention remain
mandatory implementation controls, not measured evidence in this design.

## 5. Raw child, capture and settlement

The NEW guard from the SAME actual admission covers the final intent/eligibility
transaction and following
synchronous child section. Release SharedStore after known commit while retaining
that SAME admission. A fallible intent commit is Held and cannot spawn; original
plan reconciliation must prove its exact sealed pre/post images first. A
definitive pre-write refusal/proven rollback before ANY child attempt can retry
inline only with SAME plan/ID, still-current unrevoked actor/admission and parent
finite retries/backoff. Known post confirms SAME intent, never repeats spawn.
Commit/spawn uncertainty stays Held. After returned error/Root abandonment,
reconciliation records facts only and cannot revive the actor or helper.

Preallocate independent raw-child custody and capture responsibility before
calling `Command::spawn`. On success move the actual raw tokio Child into it
immediately, before PID conversion, pipe extraction, task/reader registration or
any other fallible step. Qualify the child in place; do not call existing
`OwnedProcess::spawn` and assume an Err means no child. A spawn Err/empty cell is
an attempted creation with no handle, not a NoChild/NoCurrentDispatch certificate.
Do not repeat an uncertain spawn. Error-return paths keep the original attempt,
raw handle if present and capture responsibility; no future-local ownership gap.
The new raw custody destructor attempts permitted group hygiene BEFORE raw Child
field destruction/reaping, only for a qualified original unreaped leader. Before
PID/identity qualification, retain SAME raw Child and use only valid direct-handle
best-effort actions. Never adopt a newly discovered numeric PID for signaling.
Unknown hygiene does not discard known actual work observations.

Release admission before capture awaits. No SharedStore/admission/queue/source
mutex spans child I/O. The actual child has its own process group, original Task
cookie and null stdin; stdout/stderr are piped into bounded readers. Capture
stdout+stderr COMBINED≤64 KiB with checked shared accounting including discarded
stderr; stricter per-stream limits never replenish that shared quota. Use
≤30s command deadline,
≤2s drain and ≤10s best-effort direct stop/reap. Overflow and inherited open pipes
do not create an unbounded suffix drain or indefinite native lease.

A50ms fence uses outside-Store original planning and the same-transaction
current/source/Driver/Unit/readiness conjunction; cancellation/revocation initiates
best-effort stop of this retained child. A fence/storage error retains observation
and originals rather than reclassifying a known exit. Keep the leader unreaped
until group signaling. Do not infer descendant death from leader exit, ESRCH,
empty pipes or wait success; hygiene result is separate from helper outcome.

Retain bounded actual exit/output observation in helper custody before any
optional Store access, version parsing or receipt projection. Receipt contains
only capped outcome/exit/version-profile/hash/byte-count/hygiene fields, not raw
stdout/stderr/errors. Exit0 alone does not qualify a version. Existing pinned
parsers are retained: Codex's exact `codex-cli 0.160.0` and Claude's local
`2.1.283` profile. This is parser/profile support, not current CLI compatibility.
Unsupported version or malformed/overflow output is not a prepared proof.
The inline continuation awaits bounded capture/stop/drain outcome and saves its
actual observation or Unknown custody BEFORE deliberately returning mandatory
Unavailable. Root's following Err abandonment preserves custody/known result.
Cancel/stop still revokes eligibility and best-effort stops the original child;
neither error return nor post-error reconciliation admits another helper.

Only a SAME saved owned observation can plan pending→confirmed/unknown receipt
CAS; strict complete body/version/index/operation and whole inventory checks are
mandatory. The privately retained settlement plan fixes exact pre/post images and
uses checked version advancement. Both normal write and uncertain-commit confirmation
require that SAME plan/observation, original intent identity and exact indexed
inventory; missing/mixed/unexpected images remain Held. A current confirmed receipt
alone cannot issue settlement or repair a conflict. Persistence
error cannot discard known exit or version, and receipt rows cannot recreate it.
Unknown capture/creation remains Held. Complete physical collection never grants
logical settlement or releases the preparation gate. No Task result or Workflow
opinion is written, and no quota/native-input/Session/start continuation is admitted.

## 6. Owned proof, compatible journal interference and nongrant closure

No blanket `managed_effects` exact-image SQL guards are introduced. Existing10
keeps its existing version guards and private Binding protections unchanged.
The new helper's intent/observation/settlement ports are private actual producers;
their complete image/set CAS is mandatory even though the table permits other
compatible journal writers. No public selector, optional authority mode, raw
permission callback or generic marked-scope authorization is added.

| Actual writer or consumer | Required behavior in this increment |
| --- | --- |
| Generic reserve/helper/native-input paths | Existing marked Driver/Native authority refusals remain; no route through generic validation is enabled |
| Generic reconcile or `owner::HelperGuard` Drop | Their existing journal writes do not mint an owned observation; any changed expected image/set causes private CAS conflict/Held; this helper never installs the generic guard |
| Actual epoch fencing | Existing pending→unknown/version advance remains nongrant; original epoch/current/Driver/readiness failure stops eligibility, retains actual observations and never reconstructs/replays an actor |
| Cleanup intents/receipts | Existing closed-Unit/current cleanup claim protects hygiene; receipt state cannot close Native permission or manufacture a helper/absence/Session/result proof |
| Verifier and retained Git writers | Keep their own private grant/provenance and existing guards; a foreign or unexpected same-Unit effect is not this helper's owned observation and makes its inventory conflict/Held |
| Later Native prepared/registration/input/ACK/terminal consumers | SAME actual owned helper evidence and exact original expected inventory are mandatory added conjuncts; existing genuine owner/input/ACK/terminal and protected-table permissions remain mandatory; rows alone never supply them |
| Driver initial-input/preparation (`driver/preparation.rs:326–351,529–565`) | Task-wide pending/unknown history keeps applicable preparation/retry blocked, including after restart; changing a row state never issues original attempt or absence proof |
| Earlier Executor/bootstrap adoption (`driver/executor.rs:193–219`, `execution.rs:658–663`) | These confirmed/resolved Git-helper gates precede this version stage; they cannot qualify its later effect or accept an unknown owned attempt |
| Ordinary marked Driver advance (`driver.rs:318–350`) | Existing refusal remains; no generic after-write refresh or row-state-based retry/advance is enabled |

For example, generic `reconcile_managed_effect` can change this Pending1 row to
Confirmed2 with an apparent exit/version receipt. The actual helper must not skip
capture, replay or qualify from that row. Its actual capture remains independently
owned; settlement of its saved Pending1 preimage conflicts and stays Held. After
genuine settlement, a generic version3 update likewise conflicts with the saved
postimage at the next private consumer. Delete/replace/index/body drift also
refuses; version or state alone is insufficient. Supported API version advances
cannot restore an original image. No current-row replanning hides interference.

An exact known-postimage confirmation is factual only when the independently
retained SAME original plan and actual owned observation already exist. It is
not a constructor from SQL. Future NoCurrentDispatch/parking must additionally
use the complete actual helper-attempt manifest and actual outcomes, not just a
query showing no pending/unknown rows. Unexpected confirmed delegated effects
cannot prove absence or permit a new dispatch. This version-only increment
produces neither NoCurrentDispatch nor full prepared input.
In particular an epoch-written Unknown→Resolved/Confirmed journal change is NOT
logical closure or a retry/dispatch grant. Applicable Task-wide holds remain a
bounded attention condition until separately reviewed genuine original-attempt/
observation and permission/ownership settlement exists. This increment supplies
no recovery capability and never clears the hold merely because a query finds
no pending/unknown rows. Unrelated Task journals are not an authority exemption.

The existing journal remains weaker than a sole-writer immutable log. A compatible
writer can alter diagnostic receipts, produce conflicts and delay progress; this
design prevents such receipts from supplying Native authority, not every journal
update. SQLite and child creation are not atomic: a journal writer may race after
the last check and a genuinely already-admitted single helper may run. The actual
original intent/one-shot creation state remains its cause; the raced row never
authorizes another spawn, registration or input. Subsequent exact consumers
detect drift/Held. Intentional same-user SQL mutate-and-restore is outside the
cooperative accident model; rururunx is not a security sandbox.

All full plans/encoding remain outside SharedStore. Terminal/cancel/epoch cleanup
does not need open preparation authority to record nongrant facts, but may not
convert Unknown into an owned success, discard known exit or reopen the actor.
Both Unit permission flags/currentness and actual retained ownership remain
separate from journal state. Best-effort cleanup does not certify child/descendant
death or logically settled work. Existing restart/closure defects, if found, keep
their own gates; this increment does not claim to repair or qualify all of them.

## 7. Existing10 compatibility and evidence boundary

Baseline `state::SCHEMA_VERSION` is10. This increment changes neither schema,
layout fingerprint, trigger/permit slices nor writer-contract registration. It
uses existing effect columns with bounded producer-generated values. No11
allocation or all-writer conversion is required for this narrower owned-proof
contract. A future journal-integrity/new-layout feature would need its own ordered
migration and actual cached-writer matrix; it is not silently delivered here.

The uninstalled reservation **Verifier8 → Runtime9 → Binding10 → ReviewRound11**
remains as recorded in approved Binding design§1
(`2e364bb792d23de365ad5e727356c874db9bf560`) and the migration-order paragraph
of `master/workflow-engine.md` at this pinned source.
No installed or implemented Review11 is inferred, and this helper does not consume
that reservation or depend on future Review authority.

An actual cached or newly opened old10 writer remains compatible with existing10;
do not claim its permitted effect journaling is rejected by a new schema fence.
Its changed row/set must instead conflict at the new private actual helper
consumer, with original observation retained and no input/prepared/Session proof.
Existing old9/newer-version rejection, Binding exact owner/input/Workflow/Session/
Driver/Source/audit guards and Task-version immutability remain unchanged and
must continue to hold. New private helper images must decode as existing exact
ManagedEffect through actual old10 managed_effects/effect_tx/startup epoch fence;
existing-reader compatibility is mandatory, not merely SQL json_valid success.
A new binary with a simulated old version is not evidence
about actual old10 behavior. Actual compiled old10 interference controls qualify
this narrower boundary, not journal immutability or new migration readiness.

## 8. Verification gates and remaining work

Fixed clean source and actual Claude/Codex/Grok Triple reviews precede acceptance,
including the matching master delta. This revision is HOW-only; local safe-hook
source inspection is not a build or VM-budget qualification. Positive
controls require the real accepted Goal/Driver/Source/capacity/marker/launch/job/
actor chain; no SQL-seeded owner/Unit, fake capability or row-derived proof.
If issuer composition still refuses, record SETUP refusal and do not credit a
mutant kill or first-stage/version success. Pure parsers and legacy controls are
separate evidence.

Required actual controls: original profile A versus changed B; stop between
known intent and spawn; error after raw-child creation before PID/pipes/reader;
unpolled capture task/future Drop; bounded overflow/no-terminal/held-pipe timeout;
known exit before optional Store fault; SAME original intent/observation retry;
stale epoch/Unit/source/ticket/readiness/owner refusal; Task/version unchanged;
other Task's helper continues. Actual generic reconciliation must change the
pending image before capture and the settled image before the next private
consumer: each conflicts/Held without a second child or fabricated qualification.
Exercise missing/replaced/foreign/index-mismatched/oversized rows, extra confirmed
same-Unit effects, baseline254→version255 with one reserved input slot, baseline255
refusal before intent/spawn, separate full inventory256/257 and inclusive2 MiB
all-column/body boundaries, combined64 KiB/64 KiB+1 across both streams and actual
compatible
cached/new-open old10 journal interference. Protected owner/input/Session/Workflow/
Driver/Source and old9/newer-version negative controls remain separate gates.
Compiled omissions of actual owned-observation dependency, original full set/CAS,
saved postimage, epoch/current/Driver/source/pair or one-shot spawn checks must
fail intended consumer assertions. Compile/setup failures do not count as
mutation kills. A small valid body plus oversized identity/idempotency/index TEXT
must refuse before extraction. Keep selected Unit fixed while growing unrelated
history: the complete routine must finish within VM budget or interrupt/Held
without partial proof/effect; reset the hook for an unrelated next operation and
mandatory closure. Exercise error/unwind, cumulative multi-statement budget and
multiple near-boundary Tasks without a universal stop-latency claim. Old10 decode/
epoch compatibility and an extra-top-level-field mutation are separate controls.

This component leaves generic authority refused and global composition issuer
unavailable until genuine full prepared-input, remaining Git/hooks/quota/transport,
normal/late binder, nongrant closure and Runtime handoff consumers qualify. It
does not assert `cargo install` Native usability, authenticated compatibility,
both-OS conformance, four-Task result independence or process reclamation.
