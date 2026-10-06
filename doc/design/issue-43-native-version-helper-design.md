# Managed Native version-helper integration design

## 1. Status, scope and dependencies

This is a proposed, STRICT implementation supplement to
`issue-43-native-preparation-integration-design.md`, including its original
custody and transport amendments. It does not replace the complete Binding
contract or enable Native availability. Production implementation of this
increment waits for independent design review and the ordered migration decision
in §7. There is no authenticated CLI, four-Task, OS or process-death qualification.

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
| `native.rs:366–398`, local HelperGuard, generic capture/receipt and version parser | Independently retained same helper plan, eager raw-child capture and sealed owned observation before optional persistence |
| `process.rs:26–34`, raw child exists before fallible PID checks | New retained raw-child adoption API; existing legacy spawn is unchanged |
| `process.rs:143–174`, generic ExecutionAuthority fence | Private same actor/current/Driver/source/readiness fence; no generic fallback |
| `owner.rs:125–153`, HelperGuard Drop updates receipt from ID | Protected helper abandonment retains originals and factual unknown; never uses this generic guard |
| `resources.rs:317–329`, read-all then size check | Read at most64 KiB+1 before decode; strict duplicate/budget checks and original digest/namespace qualification |
| `native/preparation.rs`, known readiness commit followed by deliberate unavailability error | Continue inline into this private version-only stage before returning; retain the final no-Session/no-transport refusal |

The old protected start cannot reach its generic helper branch. Its standalone
public ManagedInput and legacy refusal gates remain. Protected route replacement
is an explicit actual selected-vtable continuation of the SAME custody/actor,
not a `native=true` flag, callback, new public authority mode or current-row lookup.
The actual start invokes this private continuation immediately after saving its
known readiness commit, before it returns an error. It does not call start again,
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
The helper custody owns actor/plan/observations as siblings, with no strong edge
back to outer preparation custody, NativeSessions, Runtime, queue or registry.
The actor keeps its existing Weak original-custody link. The Root job retains
preparation custody independently of the caller/start future.

Capture's independently owned task holds the inner helper custody; a separate
sibling task handle does not return through the task to its own outer registry.
The eager ownership guard is constructed before spawn/first poll. No local
future, receiver, generic HelperGuard or JoinHandle Drop is the sole owner of
raw child, original plan or observed result. The same-Unit guard stays held.

## 4. Plan and intent before effects

Outside Store/queue/source/custody locks, qualify the selected original absolute
program and current directory, bounded physical resource profile and original
Unit/profile identity. Read profile bytes with a64 KiB+1 sentinel, strict bounded
decoder and complete encoded budget; require the original digest and canonical
per-Unit temp/output/cache/tool paths. Capture no auth/settings values. Preserve
inherited environment and required hooks; prepare only the existing non-secret
resource overlay and bounded Git-config-count overlay outside Store.

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
no caller-provided argv/purpose/target. Effect kind is `native_phase_version`;
the complete bounded body records original phase operation/pair, origin,
Unit/epoch/generation, profile/command digest and that exact helper ID. These
fields are indexes/facts, never Native authority. Body ceiling is8 KiB. No raw
environment or arbitrary stdout/stderr is included.

Reserve exactly one pending `managed_effects` image under a private exact-image
permit and same actual admission. Recheck complete original images and absence
of the SAME helper ID/idempotency identity; commit before process creation.
Task, Workflow, Context, Driver, Source, Session, input, owner and readiness stay
unchanged. Count this helper once, never rotate IDs on an error. The full HOW's
≤32 helper-intents-per-original-operation ceiling remains; this increment permits
one version intent only. Bounded reads use a sentinel, not unrestricted COUNT.

## 5. Raw child, capture and settlement

Actual admission covers the final intent/eligibility transaction and following
synchronous child section. Release SharedStore after known commit while retaining
that SAME admission. A fallible intent commit is Held and cannot spawn; original
plan reconciliation must prove its exact sealed pre/post images first.

Preallocate independent raw-child custody and capture responsibility before
calling `Command::spawn`. On success move the actual raw tokio Child into it
immediately, before PID conversion, pipe extraction, task/reader registration or
any other fallible step. Qualify the child in place; do not call existing
`OwnedProcess::spawn` and assume an Err means no child. A spawn Err/empty cell is
an attempted creation with no handle, not a NoChild/NoCurrentDispatch certificate.
Do not repeat an uncertain spawn. Error-return paths keep the original attempt,
raw handle if present and capture responsibility; no future-local ownership gap.

Release admission before capture awaits. No SharedStore/admission/queue/source
mutex spans child I/O. The actual child has its own process group, original Task
cookie and null stdin; stdout/stderr are piped into bounded readers. Capture
stdout≤64 KiB, stderr≤64 KiB (diagnostics discarded), ≤30s command deadline,
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

Only a SAME saved owned observation can plan pending→confirmed/unknown receipt
CAS; strict complete body/version/index/operation checks are mandatory. Persistence
error cannot discard known exit or version, and receipt rows cannot recreate it.
Unknown capture/creation remains Held. Complete physical collection never grants
logical settlement or releases the preparation gate. No Task result or Workflow
opinion is written, and no quota/native-input/Session/start continuation is admitted.

## 6. All effect writers and nongrant closure

Protect ALL `managed_effects` images for a Unit associated with a managed phase,
not only the new kind/profile. Exact guards cover OLD and NEW Unit associations,
prevent reclassification/move/delete/REPLACE bypass, and accept only compiled
private image permits. Add the closed compiled column slice for this table; no
public table selector, raw permission callback or generic effect exception.

| Actual writer | Required protected behavior |
| --- | --- |
| `effects.rs` generic reserve/reserve_effect_tx/native-input callers | Refuse; separate genuine private helper/input producer needed |
| generic reconcile and `owner::HelperGuard` Drop | Refuse protected writes; original helper observation/abandonment port only |
| `effects::fence_epoch_effects` | Bounded exact nongrant pending→unknown plans during actual epoch fencing, no row-to-owner restore or grant |
| `cleanup.rs` delegated/cleanup insertion and reconciliation | Qualified exact nongrant cleanup intent/receipt ports; no effect-profile exemption |
| `verification.rs` command/abandonment writers | Existing actual private verifier proof plus exact protected receipt permits; otherwise refuse |
| `artifacts.rs` retained Git intents/receipts | Existing private retained-read or Source provenance plus exact permits; otherwise refuse |
| future Native input/ACK/terminal writers | Genuine consumed pair and owned actor/terminal plus exact image permissions; current generic paths stay refused |

All complete plans/encoding happen outside SharedStore. Epoch/cleanup factual
closure may operate after normal actor/Driver revocation, but validates exact
original indexed effects and latest Unit/epoch/cleanup relation, writes only
nongrant facts and cannot certify a version, input, owner, success or absence.
Such allowance is distinct from new open preparation permission. Until every
actual affected writer has an approved concrete port, the changed contract
cannot be installed/enabled merely because version-probe tests pass.

## 7. Ordered migration dependency

Baseline `state::SCHEMA_VERSION` is10. `managed_effects` currently has ordinary
writer-version guards but no Binding exact-image guard/slice. Its generic
`reconcile_managed_effect` can update by ID/version without a protected-scope
check. Consequently this stage requires a fresh ordered migration and cached
old-writer defense; changing established10 DDL is forbidden.

The existing reservation is **Verifier8 → Runtime9 → Binding10 → ReviewRound11**,
in the approved Binding design§1 (`2e364bb792d23de365ad5e727356c874db9bf560`,
design-approval checkpoint) and `master/workflow-engine.md:22`. Root coordination
also assigns active Review implementation authorship to C; that coordination is
not proof that schema11 is installed or implemented. The Root schema owner must
allocate the actual next migration and adjust any uninstalled reservation before
production work. This document assigns no number and silently installs nothing.

Migration must boundedly qualify existing effects, roll back malformed/oversize
inventories without repair, compose every existing guard and qualify current
layout without recreating it. Formal matrix includes preopened/cache-compiled
actual10 writer, new-open actual10, new writer on previous/current/future layout,
and protected INSERT/UPDATE/DELETE/REPLACE attempts. Cached old10 writes must fail
before effects under the new version; a new binary's simulated number is not that
control. Actual migration allocation/all-writer closure are engineering dependencies,
not a request to weaken authority or a claim of human approval.

## 8. Verification gates and remaining work

Fixed clean source and two independent reviews precede acceptance. Positive
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
other Task's helper continues; all-writer and actual-old10 matrix above. Compiled
mutations must fail intended runtime assertions, not compile/setup failures.

This component leaves generic authority refused and global composition issuer
unavailable until genuine full prepared-input, remaining Git/hooks/quota/transport,
normal/late binder, nongrant closure and Runtime handoff consumers qualify. It
does not assert `cargo install` Native usability, authenticated compatibility,
both-OS conformance, four-Task result independence or process reclamation.
