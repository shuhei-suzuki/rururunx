# Issue 43: Native transport registration, spawn and pre-Core custody

## 1. Status, scope and exact source

This is the authorization/contract HOW supplement required by approved
[Native preparation HOW](issue-43-native-preparation-integration-design.md)
§4.1 ("exact durable representation and guard layout must be coordinated/reviewed
with Root before source installation"). It refines §§2, 3.1, 3.2, 4.1, 5 and 6 of
that document and the [managed binding design](issue-43-managed-binding-design.md)
§§2.3, 3, 5.1 and 8. It adds no alternative authority, does not relax any approved
requirement and does not redesign the genuine #19 admission, result protection,
immutable commits, disposable worktrees or best-effort hygiene model. Existing
requirements remain authoritative; this is not a requirements spec and contains
no master-design update claiming implementation.

Exact source: `76a58b6e3dfa9cedb7e528296fb59d2fa4725666` (clean, fixed). All line
references below are to that commit, not to moving worktree bytes.

Three categories are kept separate throughout:

- **Actual (A)** — present in source at the pinned commit.
- **Proposed (P)** — this contract; not implemented.
- **Verified (V)** — only what source reading at the pinned commit shows. No
  build, test, lint or runtime was executed for this HOW. Full regression and
  Clippy are RED at this source; rrx native one-/four-Task, macOS/Linux, auth and
  hooks remain unqualified; installed composition and static admission remain
  closed. Narrow pure tests/compiled mutants are not real actor lifecycle proof.
  Inherited Stop repeated-failure and request-identity findings remain open; this
  HOW neither fixes them nor declares their authorization satisfied.

Out of scope (unchanged): credentials, HOME, root, VM/container, settings, hooks,
license, ps46/60, public actor constructors, callback capabilities, optional
protected bypasses, sandbox/death proof, FakeAgent admission, row-created
ownership and any Task.version-changing Session binding.

### 1.1 One-line acceptance condition (P)

A protected Native command is created only by the SAME actual `PreparedNativePhase`
after one known-committed Immediate registration + distinct `native_phase_transport`
intent under the SAME Runtime stop admission with SharedStore released, and every
returned `Child` is moved into preallocated custody before any fallible step and
stays there until the SAME registered actor's Core accepts it; every other path is
no-spawn, Held, or nongrant closure.

## 2. Existing consumers and why they cannot be used (V)

| Actual consumer at 76a58b6e | Problem for the protected transport |
| --- | --- |
| `execution/native.rs:215–648` `start_with_launch(…, launch: Option<…>)` | Only caller passes `None` (`native.rs:194`), so the protected branch is dead today, but it still exists: generic `validate_execution` (`:264–269`), generic helper/version under Store (`:335–361`), generic quota (`:403–420`), registration and `OwnedProcess::spawn` **while holding SharedStore** (`:513–560`), then post-spawn PID/pipe/registry work before Core (`:561–641`). It must be deleted, not kept as fallback |
| `execution/native.rs:207–214` `start_phase_inner` | Calls `begin_phase_preparation` then bails; correct refusal today |
| `execution/native/preparation.rs:286–372` | Ends with an explicit refusal after the version helper; no `PreparedNativePhase` producer exists |
| `state/execution/native_phase.rs:1087–1209` `plan_phase_registration`, `:1417–1479` `register_phase_session`, `:1394–1413` `validate_phase_preparation` | Consume `initial_readiness` = allocated **v1** (`:982–1000`). Actual preparation already moved readiness allocated v1 → preparing v2 (`native_phase/preparation.rs:108–117,137–161`), so these can never succeed on the prepared lineage. They also call generic `validate_authority(…, true, false)` (`:1407,1462`), which runs the ordinary Driver/source route (`state/execution.rs:458–497`) |
| `state/execution/native_phase.rs:311–330` (`plan_owner_currency`) and `native_phase/terminal.rs:305–316` | Hard-code registered readiness `(v2, !ended) ∨ (v3, ended)`; a prepared registration is `registered vP+1` with `P ≥ 2` |
| `state/execution/native_phase.rs:353–378` `NativeOwnerPlan::validate_tx/validate_terminal_tx` | Still call generic `validate_authority`; every registered dispatch/ACK/projection/terminal consumer therefore refuses a marked scope (gate G2, §16) |
| `execution/process.rs:109–119` `OwnedProcess::spawn` | Creates the child before fallible ID checks (`:111–113`); an `Err` cannot prove no child |
| `execution/process.rs:17–99` `RetainedRawProcess` | Correct adopt-first primitive, but exposes only stdout/stderr (`pipes`, `:44–51`) and no in-place upgrade to `OwnedProcess` |
| `execution/native/version.rs:144–180` `NativeVersionHelperCustody::reconcile` | Re-reads the complete effect inventory and requires `== pending ∨ == after` (`version/closure.rs:374–377,401–403`). Any later effect row makes a re-run conflict; `preparation.rs:150–174` re-runs it on every reconcile wake |
| `state/execution/effects.rs:129–189` `reconcile_managed_effect_pinned`, `:247–283` `reserve_effect_tx` | Generic journal writers with no kind restriction |
| `state/execution/effects.rs:3–28` `fence_epoch_effects` | Every `pending` effect becomes `unknown` at a new Runtime epoch |

`runtime/phase_effect_admission.rs:12–87` (A) is the SAME stop-admission object:
it shares `Runtime.control_admission` and the `running/stopping` atomics
(`runtime/mod.rs:43–61`). Cooperative shutdown (`runtime/service.rs:106–111`) and
Driver/control paths (`runtime/control.rs:303,329,364`, `task_driver.rs:18,55`)
acquire it. `Runtime::drop` (`runtime/mod.rs:115–120`) sets `stopping`
**without** acquiring it; that revocation is not linearized by admission (§11).

## 3. Producer and type graph (P)

All types are crate-private, non-`Clone`, non-`Serialize`/`Deserialize`, with
private fields and no constructor reachable from IDs, SQL rows or DTOs.

```text
Root PhaseJobs Entry -> Job -> JobState.preparation (A, phase_jobs.rs:33-42)
  -> NativePreparationCustody (A)            [one per original operation, <=128]
       actor: NativePreparationActor (A)     [holds same-Unit gate in one-time cell]
       plan / known readiness commit (A)
       helper: NativeVersionHelperCustody (A) + version_closed (P)
       prepared: Option<Arc<PreparedNativePhase>> (P, issued by absent producer G1)
       transport: Option<Arc<NativeTransportCustody>> (P)
NativeTransportCustody (P)
  plan: Arc<NativeTransportStartPlan>        [Store-owned sealed images]
  candidate: Arc<PhaseActor>                 [built before admission; unissued]
  registration: Option<Arc<NativeTransportRegistration>>  [known-commit ack]
  creation: CreationState                    [one-shot: NotAttempted|Attempted|ReturnedChild]
  raw: Mutex<RetainedRawProcess>             [Empty|RawChild|Qualified; poison-recovering]
  prebuilt: Option<PrebuiltEntry>            [watch/mpsc/terminal cell/Entry value]
  handoff: Handoff                           [None|Offered|Accepted]
  observation / settlement / closure: Option<Arc<..>> (one slot each)
  stop_requested: AtomicBool, reason: &'static str
NativeTransportStartPlan -> PreparedNativePhase -> NativePreparationActor -> PhaseLaunchParts
NativeTransportRegistration -> SAME NativeTransportStartPlan
PhaseActor (A) -> NativePhaseSession (A + P fields) -> PhaseLaunchParts
NativePhaseSession -> NativeTransportRegistration (P; immutable origin, no custody edge)
PhaseLaunchParts -Weak-> NativePreparationCustody (A)
NativePreparationActor -Weak-> custody, -Weak-> NativeSessions (A)
NativeSessions.preparations -Weak-> NativePreparationCustody (A)
Core (A) -> child: OwnedProcess, phase: Arc<PhaseActor>
```

No strong edge returns to Runtime, PhaseJobs, JobState, NativeSessions or the
NativeAdapter: the transport custody is a sibling slot of the Root-retained
preparation custody, reached from Native only through the existing Weak index.
No new registry or index is added. This removes local return edges; it is not a
proof that every existing ownership graph is cycle-free.

### 3.1 Private signatures (P)

Names may be narrowed during source implementation; producer privacy and
mandatory checks may not be weakened.

```rust
// Native (execution/native/…). Issued only by the actual preparation producer
// after every required helper/profile/quota outcome has known settlement (G1).
pub(crate) struct PreparedNativePhase { /* see §5.1 */ }
fn issue_prepared(custody: &Arc<NativePreparationCustody>)
    -> Result<Arc<PreparedNativePhase>>;            // absent today (G1)

// Built outside admission/Store/custody locks.
fn plan_transport_command(owner: &Arc<RuntimeOwner>, prepared: &Arc<PreparedNativePhase>)
    -> Result<NativeTransportCommand>;              // fixed argv/env/cwd; §6
// Store (state/execution/native_phase/transport.rs). Snapshot outside SharedStore.
fn plan_prepared_transport(runtime: &RuntimeOwner, prepared: Arc<PreparedNativePhase>,
    command_digest: String, native_uuid: Option<String>)
    -> Result<Arc<NativeTransportStartPlan>>;
fn register_prepared_transport(&mut self, plan: &Arc<NativeTransportStartPlan>,
    admission: &PhaseEffectAdmissionGuard) -> Result<NativeTransportRegistration>;
fn confirm_prepared_transport(&mut self, plan: &Arc<NativeTransportStartPlan>,
    admission: &PhaseEffectAdmissionGuard) -> Result<RegistrationProbe>; // §8
fn plan_transport_settlement(reg: &Arc<NativeTransportRegistration>,
    obs: Arc<NativeTransportObservation>) -> Result<Arc<NativeTransportSettlementPlan>>;
fn record_transport_settlement(&mut self, s: &Arc<NativeTransportSettlementPlan>) -> Result<()>;
fn plan_transport_closure(runtime: &RuntimeOwner, s: Arc<NativeTransportSettlementPlan>)
    -> Result<Arc<NativeTransportClosurePlan>>;
fn close_transport_observation(&mut self, c: &Arc<NativeTransportClosurePlan>) -> Result<()>;

// Process (execution/process.rs), additions only; legacy OwnedProcess::spawn unchanged.
impl RetainedRawProcess {
    fn native_pipes(&mut self) -> Result<(ChildStdin, ChildStdout, ChildStderr)>;
    fn into_owned(&mut self) -> Result<OwnedProcess>; // Err leaves the SAME Child in place
}
```

`RegistrationProbe` is `{ Committed(NativeTransportRegistration), Absent, Held }`.
`into_owned` succeeds only when `pid` is already qualified and the cell holds the
child; the move itself is infallible after that check.

## 4. Durable representation (P)

### 4.1 Choice

The transport-start intent is ONE new row in the existing `managed_effects` table
with body kind `native_phase_transport`, inserted in the SAME Immediate transaction
as the registration rows. Alternatives rejected:

- Native6 `native_invocations.state` — would need a CHECK/DDL change and is the
  input/ACK journal; overloading it conflates transport with input.
- `managed_phase_readiness` — its complete body JSON is matched exactly by four
  consumers (§2); adding keys breaks them, and readiness is not an effect journal.
- A new table — needs Binding11 layout, catalogue and cached-writer migration for
  no additional guarantee over §4.4.

It is distinct from `native_phase_version` (helper), `native_setup`,
`native_input` and `native_permission` (protocol messages; `native.rs:986`,
`native_phase.rs:468`). It implies no input, ACK, Running or nativeRef.

### 4.2 Complete registration transaction images (allowed write tuple)

Let `f = launch.allocation().facts()`, `P` = prepared readiness version
(`P = 2` without parking; the quota increment may advance it, §5.3), `U` = prepared
Unit version, `T` = transport effect OperationId generated once in the plan.

| Row (table, key) | Old image (complete indexed + body + version) | New image |
| --- | --- | --- |
| `managed_phase_owners`, `f.pair_id` | existing `original_owner` image (`native_phase.rs:905–919`): body `{owner_id,operation_id,scope,unit_id,owner_epoch,execution_generation,allocated_session_id,provider,alias,role,worktree,origin,"native_invocation_id":null,"validated":false,"version":1}`, columns `native_invocation_id=NULL, validated=0, version=1` | same columns with `native_invocation_id=f.invocation_id, validated=1, version=2`; body same keys with those three values (unchanged from `:1132–1146`) |
| `managed_phase_readiness`, `f.operation_id` | `{operation_id,origin,owner_epoch,"state":"preparing","start_ended":false,"known_terminal":false,"parking_version":null,"version":P}`; columns identical; ≤4096 B | `state="registered"`, `version=P+1`, all other keys/columns unchanged |
| `records`, `f.session_id` | absent (`no_registration`) | INSERT `(id,'session',p,g,t,1,body)`; `Record{kind:Session,version:1,data:Session}` with `Session{id:f.session_id, scope:f.scope, agent:f.alias, provider:f.provider, role:f.role, native_ref:None, pid:None, worktree:f.path, state:Starting, model:f.model, effort:f.effort, recovery:{unit,generation,epoch}, started_at}`; ≤32 KiB |
| `session_units` | absent | INSERT `(f.session_id, f.unit_id, p,g,t,'pending')` |
| `execution_units`, `f.unit_id` | full prepared image: `version=U`, `state=Preparing`, `session_id=None`, `native_effects_open`, `result_finalization_open`, `work=None`, `disposition=Active` | `version=U+1`, `session_id=Some(f.session_id)`, `state=DispatchPending`, `updated_at`; CAS on `id,version,body` (as `:1468`); ≤16 KiB |
| `native_invocations`, `f.invocation_id` | absent | INSERT `state='not_dispatched', version=1`, body from `NativeSeed::invocation` with `native_version` = text of the SAME owned version observation, `profile="text_v1/{profile_digest}"`; ≤`INVOCATION_BYTES` |
| `managed_effects`, `T` | absent; idempotency key absent | INSERT, §4.3 |
| `audit` | — | existing `execution.session_intent` and `execution.native_invocation` events only; no new audit kind |

Exact binding permission tuple (unchanged structure, `native_phase.rs:1426–1456`):
`managed_phase_owners UPDATE(v1→v2)`, `managed_phase_readiness UPDATE(preparing P
→ registered P+1)`, `records INSERT`. `ensure_consumed` precedes commit. The
`binding_managed_phase_readiness_core` trigger (`schema.sql:178`) already accepts
`version=OLD+1`, origin/epoch unchanged and monotonic flags. `managed_effects` is
not a binding-permit table; its INSERT is governed by the existing
writer-contract10 guard only.

### 4.3 Transport effect images

Intent (v1), generated once in the plan and charged before commit:

```text
ManagedEffect {
  id: T, unit_id: f.unit_id, scope: f.scope,
  kind: "native_phase_transport",
  idempotency_key: "native-transport-{f.operation_id}",          // 53 bytes
  expected_target: "transport:{sha256(operation_id:pair_id:epoch:session_id:
                    invocation_id:command_digest)}",
  state: Pending, receipt: {}, version: 1 }
```

`idempotency_key` is UNIQUE in DDL (`execution.sql:103`), so at most one transport
intent can ever exist per original operation, independently of in-memory state.
`command_digest` is SHA-256 over the canonical fixed command plan (§6) with
environment **keys only**; the IPC cookie, socket path values and inherited
environment values are never persisted, hashed into rows or logged.

Settlement (v2) is written once from the SAME actual `NativeTransportObservation`:

| Actual creation observation | `state` | Required receipt keys (≤16 entries, key ≤64 B, value ≤256 B, no control chars) |
| --- | --- | --- |
| One-shot latch never set (spawn not invoked) | `resolved` | `creation=not_attempted`, `handoff=none` |
| `Command::spawn` returned `Err` | `unknown` | `creation=attempt_without_returned_handle`, `spawn_error=<not_found\|permission_denied\|other>` |
| `Child` returned | `confirmed` | `creation=returned_child`, `identity=<qualified\|unqualified>`, `pipes=<complete\|incomplete\|n/a>`, `handoff=<core_accepted\|core_offered\|precore_retained>`, `hygiene=<group_signal_attempted\|direct_kill_attempted\|unknown\|n/a>`, `reap=<exit:N\|signal\|unknown\|n/a>` |

`confirmed` means "a process was created and retained", never work success,
input, ACK or owned completion. No PID, argv, path, stderr or provider output is
stored.

### 4.4 Compatibility plan (no DDL)

- `SCHEMA_VERSION` stays 10; writer-contract registration (`state/mod.rs:86–101`),
  `schema.sql`, `permits.rs` columns, the `validate_current_layout` catalogue
  (`schema.rs:249–268`) and pre-open checks are unchanged. No Binding11 is
  allocated.
- The new kind uses existing columns with producer-generated bounded values and
  must decode as an exact `ManagedEffect` through the existing `effect_tx`,
  `managed_effects`, `fence_epoch_effects` and version `EffectImage::decode`
  readers (required control, §15).
- New binary narrowing (code-level, nongrant): `reserve_effect_tx`,
  `reserve_execution_helper_pinned` and `reconcile_managed_effect_pinned` refuse
  any `native_phase_*` kind before writing. This does not extend the
  `native_phase_version` exception; it closes the generic writers to both
  private kinds.
- Actual cached or new-open old10 binaries (including 76a58b6e) are not fenced.
  Their behavior on the new rows is conservative and nongrant: generic gates treat
  non-`git_helper` or pending/unknown rows as unsettled (`execution.rs:685`,
  `driver/executor.rs:212`, `driver/preparation.rs:349,546`); epoch fence marks a
  pending row unknown; an old generic reconcile may rewrite the row, which the new
  private consumer detects as an exact-image CAS conflict → Held. Old10 owner/
  terminal readers expect registered v2/v3 and refuse vP+1 → Held. Old10 cannot
  register a protected transport (its `start_phase_inner` refuses). Do not claim
  old10 writes are rejected by a schema fence.
- A future DB-level guard on `managed_effects` kinds would need its own ordered
  Binding11 migration and actual cached-writer matrix; it is not delivered here.

## 5. Prepared → registered → input identity continuity (P)

### 5.1 `PreparedNativePhase` contents (absent producer, G1)

Issued once by Native only, from the SAME custody, after: readiness known commit
(A); version helper settlement **closed** (A + P flag); every required readonly Git
qualification/artifact lease outcome settled (in progress separately, G1); profile/
program/path qualification; quota admission settled as admitted (not parked).
It retains, without copying: the SAME `NativePreparationActor`; the SAME
`NativePreparationCommit`; the SAME version `NativeHelperSettlementPlan` (its
`after` inventory is the expected inventory, borrowed, not cloned); the owned
version text; the exact prepared readiness `PairRow` (`state=preparing`, version P);
the exact prepared Unit image (full encoded body and version U); the governing
digest already fixed from original marker parents. It grants nothing by itself and
cannot be issued from rows, IDs, a version string or a successful helper exit.

`reconcile_known_commit` (`preparation.rs:150–205`) must return immediately once
`version_closed` is set by the SAME successful `close_phase_version_observation`
that observed `current == settlement.after`; no later wake re-reads the version
inventory. Without this, the transport row would make version reconciliation
conflict forever.

### 5.2 Registered actor continuity

- The registration candidate `PhaseActor` is constructed before admission from the
  plan's `Session` and `record_version = 1`, via a replacement for
  `PhaseActor::registered` (`phase_protocol.rs:62–76`) that additionally takes the
  `Arc<NativeTransportRegistration>` it will carry. It is unpublished: held only by
  the transport custody, not in `NativeSessions.entries`, not returned, and its
  `live` flag is false until the SAME known commit (or exact confirmation, §8)
  flips it with one infallible atomic store.
- `NativePhaseSession` (P) gains immutable `registered_readiness: u64 = P+1` and the
  registration origin Arc. `plan_owner_currency` (`native_phase.rs:311–330`) and the
  terminal reader (`terminal.rs:305–316`) accept only `(P+1, !ended)` or
  `(P+2, ended)` for this actor, not the constants 2/3. Rows of any other lineage
  refuse (Held). The allocated-v1→registered-v2 producer is deleted.
- Input continuity: Core receives a clone of the SAME `f.input` (`PreparedInput`
  captured in the allocation) and `f.model/f.effort`; registration keeps the
  existing `encode_input(seed.input()) == f.input_bytes` check (`:1111`); dispatch
  keeps comparing payload with `f.input.payload` (`:529–556`). No `ManagedInput`,
  re-read Context or regenerated pin participates.
- The Session/invocation/pair are exactly the originally allocated IDs; no UUID is
  generated after preparation except `T` and, for Claude, the native session UUID,
  both generated once in the sealed plan and reused on every retry of that plan.

### 5.3 Pre-Session parking stays distinct

Registration requires prepared readiness `state=preparing` exactly; a `parked`
readiness or an active waiter refuses before any effect. Once the registration
known commit exists a Session exists: `PreparedPhaseNoCurrentDispatch` can no
longer be issued for this operation, and any spawn attempt (even `not_attempted`
after a known registration) never becomes a pre-Session no-dispatch certificate.
If quota parking can leave the Unit in a state other than `Preparing`, the quota
increment must state the exact prepared Unit predicate; `registration_unit`
(`:1008–1038`) is not silently relaxed here.

## 6. Command plan (P)

Built outside admission/Store from the prepared actor, reusing the version
helper's bounded physical profile qualification (`version.rs:218–285`, factored
into a shared private function; not `ResourceManager::profile`):

- program: `f.program`, absolute, canonical, regular file (rechecked here);
  cwd: `f.path`, absolute, canonical. Both server-decided by the original
  allocation; never caller input.
- argv: Codex `["app-server","--listen","stdio://"]`; Claude the existing fixed
  vector (`native.rs:455–482`) with plan-generated session UUID, `f.model`,
  `f.effort` and `--permission-mode plan` for non-Executor. No shell, no caller
  argv, no new flag. Each element ≤4096 B, ≤16 elements.
- environment: the qualified profile overlay `profile.environment(cookie, socket)`
  only, as today; inherited host/auth environment unchanged and out of scope.
- `stdin/stdout/stderr` piped, `process_group(0)`, `kill_on_drop(true)`.
- Complete encoded plan ≤64 KiB. Verifier command-only Units refuse
  (`verification::is_command_unit`).

The command is not a `PhaseHelperAction`, not a callback and cannot be built from
a row. It is cooperative role policy, not host containment.

## 7. Protected start sequence (P)

Continuation of `begin_phase_preparation` after `issue_prepared` succeeds:

1. **Outside all locks:** build `NativeTransportCommand`; `plan_prepared_transport`
   on a separate bounded readonly snapshot (current successor, Driver-live,
   `registration_attempt`, owner v1, prepared readiness, prepared Unit, budgeted
   inventory == prepared expected, idempotency absence). Build the candidate
   `PhaseActor`, `watch`/`mpsc(16)` channels, frozen-terminal cell and the
   unpublished `Entry` value (`PrebuiltEntry`). Allocate `NativeTransportCustody`
   with `raw = Empty`, `creation = NotAttempted`.
2. Short custody lock: install the transport custody into the preparation custody
   slot exactly once (pointer-checked); refuse if abandoned. No SQL/await under it.
3. `launch.admission().enter(launch)` — may await before acquisition.
4. **Admission section (no await, FS, hash or allocation-heavy work):**
   a. `admission.validate_for`, `actor.validate_open`, custody not revoked/stop-
      requested.
   b. Lock SharedStore; `register_prepared_transport` (§9 checks, §4.2 writes);
      release SharedStore immediately after commit/rollback.
   c. On `Ok`: retain the `NativeTransportRegistration` in custody and flip the
      candidate actor live — both infallible after pre-reserved slots. On `Err`:
      set `RegistrationUncertain`, release admission, do NOT spawn (§8).
   d. Recheck `admission.validate_for` and `actor.validate_open`; set the one-shot
      `creation = Attempted` latch; call `Command::spawn()` synchronously.
   e. On `Ok(child)`: FIRST action is `raw.adopt(child)` under the dedicated
      poison-recovering raw mutex (no allocation, no fallible lock); set
      `ReturnedChild`. On `Err(e)`: record the bounded error class only.
   f. Drop the admission guard.
5. **After admission:** under the raw mutex, `qualify()` PID in place; take
   `native_pipes()`; `into_owned()`. Any failure leaves the SAME child in custody
   (`precore_retained`) and proceeds to §10 pre-Core closure.
6. Insert the prebuilt `Entry` into `NativeSessions.entries` (key = original
   SessionId, absence checked in step 1); on poison/collision the child stays in
   custody (§10). Spawn the stderr drain task.
7. Construct `Core` (infallible struct construction) owning the `OwnedProcess`,
   the SAME `PhaseActor` and prebuilt channels; set `handoff = Offered`; if
   `stop_requested`, revoke the session actor and pre-queue `Control::Cancel`;
   `tokio::spawn(core.run(f.input.clone(), f.model, f.effort, profile))`. Core's
   existing `Drop` (`native.rs:2248–2256` → `drop_phase`) is the eager abandonment
   guard for an unpolled or aborted task.
8. Core's first poll sets `handoff = Accepted` (P hook at the top of `Core::run`),
   then releases the same-Unit start gate once (§12). The start future records the
   transport settlement (§8.2) and returns `NativePhaseStart::Launched { handle,
   binding: phase.owner.binding_snapshot() }`; Root's existing `bind_returned`
   (`phase_jobs.rs:286–301,356–374`) continues unchanged.

There is no process call inside Immediate; no FS/hash/await while holding Store,
actor, queue, custody-state or raw-child mutex. Admission does not span pipe
handshake, Core I/O or child execution. Core input consumption/ACK remain their
separate private protocol and are not implied by any step above.

## 8. Same-plan reconciliation (P)

### 8.1 Registration

The custody retains the SAME plan and original pair across every outcome. At most
one exact probe per due wake, capped 100 ms–5 s backoff, under a fresh admission
guard, by `confirm_prepared_transport`:

| Probe observation (all images read in one Immediate) | Result |
| --- | --- |
| Complete postimage of every §4.2 row equals the plan's new images, transport row is v1 pending exactly | `Committed` → issue the SAME `NativeTransportRegistration` and flip the SAME candidate; spawn may then proceed in that same admission section (step 4d) |
| Complete preimage of every row and all inserted keys absent | `Absent` → retry the SAME plan's Immediate (same `T`, Session, invocation, native UUID) only if all §9 normal checks pass |
| Mixed, foreign suffix, transport row not pending v1, stale current/Driver, unknown | `Held`; no spawn, no new plan, no row-to-actor construction |

Equal-looking rows never create an actor; only the SAME retained plan and
candidate can be issued. A generic Store error, empty raw cell or missing Session
is never rollback evidence.

### 8.2 Transport settlement and closure

`record_transport_settlement` (normal, while the registered actor is open):
selected DB; SAME registration and observation; §9 normal predicates for the
registered owner (after G2 replacement); the transport row's exact v1 preimage
(all nine columns); single-row CAS update to the §4.3 v2 image; rowcount must be 1.
It deliberately does **not** compare the complete Unit inventory: after Core
acceptance the SAME actor's own setup/input dispatches legitimately append rows.
The pre-spawn baseline was already full-inventory exact in §9.

`close_transport_observation` (nongrant, after revocation or currency loss):
validates only `validate_preparation_original` (immutable Source/marker lineage),
the latest complete Unit image CAS (reuse `LatestUnitImage`,
`version/closure.rs:11–153`, factored to a shared module), the latest exact
Session record, owner v2 and readiness (`registered P+1` or later `closed`) images,
and the transport row's exact v1 preimage. It writes ONLY the transport row. It
issues no permission, input, owned success or Session/Unit change, and preserves
any known terminal independently (Core's frozen terminal is untouched). Drift
keeps the SAME observation Held. Both writers are idempotent on the exact v2
postimage (return Ok without writing).

## 9. Normal versus nongrant checks (P)

| Check | Registration Immediate | Spawn (4d) | Normal settlement | Nongrant closure |
| --- | --- | --- | --- | --- |
| Selected DB / selected vtable / SAME launch, custody, actor pointers | ✓ | ✓ (pointers) | ✓ | ✓ |
| `PhaseEffectAdmissionGuard::validate_for` | ✓ | ✓ | — | — |
| `actor.validate_open` (not revoked) | ✓ | ✓ | ✓ | ✗ (revocation allowed) |
| `validate_preparation_origin_tx` (current successor + Driver-live) | ✓ | — (no Store) | ✓ | ✗ |
| `validate_preparation_original` (immutable lineage) | ✓ | — | ✓ | ✓ |
| Unit authority facts, effect-open, parent activity, governing digest (`execution.rs:379–457`) | ✓ | — | ✓ | ✗ |
| Generic `validate_authority` | ✗ never | ✗ | ✗ | ✗ |
| `registration_unit` + prepared full Unit CAS | ✓ | — | — | latest full Unit CAS |
| `no_registration`, owner v1, prepared readiness P exact | ✓ | — | owner v2, readiness P+1 | latest owner/readiness exact |
| Complete inventory == prepared expected, ≤252 rows, key absent | ✓ | — | own row exact | own row exact |
| Verifier command-only refusal | ✓ | — | — | — |

There is no boolean bypass, public validation mode, optional actor fallback or
`.ok()` demotion. Root current/Driver checks are conjuncts, never replacements.

## 10. Stop, abort and fault state table (P)

| Actual observation/fault | Custody state | Handling |
| --- | --- | --- |
| Stop wins admission before 4a | Registered = none | No registration, no spawn; preparation Held per approved §3.2 |
| Registration rollback proved (`Absent`) | Planned | No spawn; SAME plan retry only if all §9 checks pass; never new IDs/pins |
| Registration commit uncertain | RegistrationUncertain | No spawn; §8.1 probe; Held otherwise |
| Known registration, stop/revocation before 4d | Registered, `NotAttempted` | No spawn; settlement `resolved/not_attempted` (authentic local precreation proof); registered owner revoked; terminal Lost via §10.1; no input/ACK |
| `Command::spawn` `Err` | Attempted, Empty | Settlement `unknown`; never NoChild/NoCurrentDispatch; no replay; Held for attention |
| Child returned, PID qualification fails | RawChild | Child retained; no group signal; direct `start_kill`/bounded reap (best effort); settlement `confirmed/unqualified/precore_retained`; never NoChild |
| Child qualified, pipes/registry/Entry failure | Qualified | Retained custody owns child; group signal + bounded reap; settlement `confirmed/precore_retained`; terminal via §10.1 |
| Start future dropped/aborted, Engine timeout, Runtime Drop before 7 | any pre-Core | Root-retained custody keeps child/outcome; `abandon()` extended to transport sets stop_requested; preparation Held until nongrant settlement; no new start/input/DTO restoration |
| Stop after 7 (Offered/Accepted) | Offered/Accepted | Send `Control::Cancel` on the prebuilt sender and revoke the session actor; Core's existing cancel/terminal path |
| Core task never polled (executor shutdown) | Offered | Core `Drop` → `drop_phase` captures Unknown/Lost terminal (A); custody stays Offered/Held; gate not released |
| Core accepted | Accepted | Pre-Core responsibility ends; gate released once; settlement `confirmed/core_accepted` |
| Settlement/closure Store error | unchanged | SAME observation retained; one probe per wake; no state inferred from error |

### 10.1 Pre-Core terminal without Core

When a registration is known but no Core exists, a private Native-only
`NativeTerminal` (as in `RegistrationGuard`, `native.rs:1171–1201`: acquisition
Missing, `HistoricalDraft`, `observed_work=Unknown`, `disposition=Lost`) is
persisted ONLY through the SAME `PhaseActor::terminal_plan` →
`finish_phase_terminal` (`phase_protocol.rs:110–148`, `native.rs:1286–1329`). With
no consumed input, `owned_success` is impossible. This path depends on gate G2;
until then the terminal stays Held while §8.2 closure still records the factual
creation outcome. Generic `RegistrationGuard`, `retire_execution_as` and
`close_execution_session` are never used for protected scopes.

## 11. Concurrency, locks and ownership (P)

Lock order: admission (async acquire) → SharedStore (sync, released before spawn)
→ raw-child mutex (short). Custody-state, Root job, entries and preparation-index
mutexes are never held together with SharedStore or across await, SQL, spawn or
hashing. The raw mutex recovers poison only to retain/hygiene the child; poison
never grants an effect.

Linearization: a stop holding `control_admission` either precedes 4a (no
registration/spawn) or waits until 4f, when the custody already holds either a
no-spawn registration or the adopted child; it then targets that SAME operation
through the custody (Root caller is gate G3). `Runtime::drop` sets `stopping`
without admission; the 4d recheck narrows but cannot close that window. A child
spawned in it is retained and closed nongrantly. This is linearization of
admission, not a process-death guarantee.

Ownership: Root `JobState.preparation` strongly retains the preparation custody
and therefore the transport custody independently of the Engine future. Neither
custody nor plan owns NativeSessions/NativeAdapter, Runtime, PhaseJobs or the
JoinHandle. Different Tasks proceed concurrently after their own short admission
sections.

## 12. Gate release and memory (P)

The actor's same-Unit gate (`preparation.rs:240,328–331`) is released exactly once
by taking the guard out of its one-time cell and dropping it outside all locks, on:
(a) `handoff = Accepted`; or (b) known `not_attempted` settlement + known terminal
or closure; or (c) a pre-Core created child observed reaped + known transport
closure + known terminal. Every uncertain outcome keeps the gate and is visibly
Held; no forced unlock permits a replacement start. Removing the Root job entry
additionally requires no remaining reconciliation responsibility. Release never
authorizes another preparation or owner. Memory teardown is not logical closure.

## 13. Finite inclusive limits (checked before copies) (P unless marked A)

| Item | Bound |
| --- | --- |
| Effect inventory before registration | ≤252 rows (transport + up to 2 setup + 1 input ≤256); complete ≤256 rows, ≤2 MiB all-column framing, body ≤8192 B, VM budget (A, `version.rs:13–73`) |
| Transport effect | idempotency 53 B, expected_target ≤256 B, body ≤8192 B, receipt ≤16 entries |
| Session record / Unit / invocation / readiness / owner read | ≤32 KiB / ≤16 KiB / `INVOCATION_BYTES` / ≤4096 B / ≤32 KiB (A) |
| Command plan | ≤16 argv × ≤4096 B, env overlay ≤64 entries, encoded ≤64 KiB; profile file ≤64 KiB+1 read (A) |
| Version text reused | ≤64 KiB combined capture (A) |
| Custodies | one transport custody per operation; ≤128 operations (A, `MAX_JOBS`, preparation index) |
| Per-custody slots | plan, candidate, registration, observation, settlement, closure, prebuilt entry: one each; no history lists |
| Plan memory | images ≤~2.2 MiB per plan without copying the borrowed expected inventory |
| Reconciliation | ≤1 exact probe per wake, 100 ms–5 s capped backoff |
| Pre-Core stop | group/direct signal then ≤10 s reap polling at 20 ms (as version helper); not death proof |
| Control channel / line limits | mpsc 16; Claude `LINE_LIMIT`, Codex 4 MiB (A) |

Overflow refuses before INSERT/spawn; settlement overflow refuses before write
with the observation retained.

## 14. Impact analysis

### 14.1 Changed symbols (P)

| Symbol (file) | Change | Callers/consumers checked (V) |
| --- | --- | --- |
| `start_with_launch` (`native.rs:215`) | remove `launch` parameter and every protected branch (`:237–245,429–431,492–497,500–560,642–645`) | only `start_inner` (`:194`, passes `None`) |
| `start_phase_inner` (`native.rs:207`) | continue into §7 only after `issue_prepared` | `start_phase` ← `NativePhasePort` ← `phase_jobs.rs:253–257` |
| `NativePreparationCustody` (`native/preparation.rs:9–231`) | add `prepared`, `transport`, `version_closed`; extend `abandon`/`Drop`/`reconcile_known_commit` | `phase_jobs.rs:140,195–228,283,393`; `begin_phase_preparation` |
| `NativeVersionHelperCustody::reconcile` (`version.rs:144`) | early return when closed; `physical_command` profile part factored | `preparation.rs:173`, `version.rs:357` |
| `PhaseActor::registered`, `NativePhaseSession::registered` (`phase_protocol.rs:62,250`) | replaced by prepared-registered constructor with origin + readiness version | only `native.rs:535` (deleted) |
| `plan_owner_currency` (`native_phase.rs:233`), terminal reader (`terminal.rs:302`) | readiness version from actor | dispatch, ACK, projection, terminal, `actual_native_authority` |
| `plan_phase_registration`, `register_phase_session`, `validate_phase_preparation`, `NativeRegistrationPlan` (`native_phase.rs:887–1209,1384–1479`) | deleted; replaced by `transport.rs` | only `native.rs:244,504,520` |
| `original_owner`, `check_owner_indices`, `no_registration`, `registration_unit`, `registration_attempt` | reused unchanged | preparation and transport plans |
| `Inventory`, `EffectImage`, `InventoryBudget` (`version.rs`), `LatestUnitImage` (`version/closure.rs`) | visibility to `pub(super)` / shared module | version + transport |
| `RetainedRawProcess` (`process.rs:17`) | add `native_pipes`, `into_owned` | version helper (unchanged use) |
| generic effect writers (`effects.rs:41,129,247`) | refuse `native_phase_*` | all generic callers; legitimate kinds unaffected |
| `Core::run` (`native.rs:1696`) | first-poll acceptance hook when phase present | phase Core only |
| `state/mod.rs:28–31` | re-export new types | crate-private |

### 14.2 Unchanged guards and bounds (V)

`schema.sql` triggers including `binding_managed_phase_readiness_core`,
`binding_managed_phase_owners_core`, `binding_owner_registration`,
`binding_admission_allocation` (`native_input` kind) and
`binding_admission_not_parked`; `permits.rs` columns and the ≤128-row permit plan;
writer-contract10; `PhaseEffectAdmission`; `PhaseJobs` `MAX_JOBS=128`;
`PAIR_BODY_BYTES`; `INVOCATION_BYTES`; Native line limits.

### 14.3 Consumers of the new row/value (V reading; behavior P)

`execution.rs:685` and `driver/executor.rs:212` (non-`git_helper` ⇒ unsettled/
forbidden — conservative); `driver/preparation.rs:349,546` (Task-wide pending/
unknown blocks preparation until settled; Unknown remains an attention hold);
`effects.rs:3–28` epoch fence; `plan_phase_dispatch` count `<256`
(`native_phase.rs:1293–1298`) — satisfied by the ≤252 baseline; `verification.rs:567`
(command Units only; transport refuses them); `adapter/native.rs:621` (test filter
on `native_version`, unaffected). Environment values (cookie, socket) reach only
the child; no persisted consumer.

### 14.4 Not affected (reason)

Root marker/publication/binder Task versions (no Task/Workflow row written by
registration beyond existing events); public `ManagedInput` start (unprotected path
unchanged and protected still refused at `native.rs:231–234`); Grok/Codex legacy
adapters (no new port).

## 15. Tests and controls (P; none executed)

Compatibility (Store-level, compiled): new intent/settlement images decode through
actual old10 `effect_tx`/`managed_effects`/epoch fence/`EffectImage::decode`;
UNIQUE idempotency rejects a second intent; layout catalogue unchanged; generic
reserve/reconcile refuse `native_phase_*`; compiled old10 (76a58b6e) generic
reconcile interference → private CAS conflict/Held; 252/253 baseline and 256/257,
inclusive 2 MiB boundaries; readiness `(P+1,!ended)`/`(P+2,ended)` accepted, v2/v3
and other lineages refused.

Causal actual-producer controls (only after G1/G2 allow a genuine positive; until
then record SETUP refusal and credit no mutant kill): real Goal/Driver/Source/
marker/job/actor chain to `PreparedNativePhase`; pause after registration commit
before spawn and race a real `control_admission` stop (both orders); injected
commit error with actual committed postimage → no spawn until exact confirmation;
true rollback → same-plan retry with identical IDs; injected PID/pipe/Entry/registry
fault after real `Command::spawn` → SAME child retained and reaped, no input/
Task write, never NoChild; spawn `Err` → `unknown`, no replay; start-future Drop
and unpolled Core task; Core accepted → gate released exactly once, Weak probes
show no custody/actor/plan cycle; other Task proceeds after the short section;
controlled protocol peer exercising stdio (not official CLI/auth/hooks).

Required compiled mutants (must fail the intended consumer assertion): spawn under
SharedStore; adopt after `qualify`; spawn on registration `Err`; omitted admission
recheck; generic `validate_authority` reintroduced; inventory equality removed;
readiness constant 2/3 restored; settlement full-inventory CAS (breaks after
setup dispatch); `version_closed` early return removed; candidate actor live before
commit; idempotency key per-plan instead of per-operation.

## 16. Unresolved genuine seams and phase gates

| Gate | Owner | Required before transport composition |
| --- | --- | --- |
| G1 `PreparedNativePhase` issuer: full #19 prepared input, readonly Git source/seal (being implemented separately), Reviewer genuine artifact/readonly lease, hooks/settings qualification, quota admission vs parking | Native (B) with A/Root | Absent; transport stays refused |
| G2 Registered-owner factual predicates replacing `validate_authority` in `NativeOwnerPlan::validate_tx/validate_terminal_tx` (dispatch, ACK, projection, terminal) | input/ACK increment | Without it every spawned child fails its first dispatch; composition stays refused |
| G3 Root stop caller targeting the SAME custody (`request_stop`) | Root | Cooperative shutdown keeps Held only |
| G4 Core first-poll acceptance hook and gate release | Native | Part of this increment |
| G5 Installed composition / static admission | Root | Remain closed |
| G6 Inherited Stop repeated-failure and request-identity findings | existing owners | Open; not addressed |

Increment order: (1) this HOW's independent Sol high review; (2) G1 producer;
(3) transport source (§§4–12) with G2 in the same or an earlier reviewed increment;
(4) actual-producer controls; (5) later user-approved four-Task and real macOS/
Linux qualification. Until each real source exists, the corresponding effect stays
refused or Held. rururunx is not a security sandbox; work, immutable result and
best-effort cleanup remain separate.
