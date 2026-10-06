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
intent under the SAME Runtime stop admission with SharedStore released, which
activates only the SAME pre-built candidate actor of that plan; every returned
`Child` is adopted into the preallocated custody cell before any fallible step,
is upgraded there in place, and leaves that cell only by one infallible move into
the SAME registered actor's Core; every other path is no-spawn, Held, or nongrant
closure.

### 1.2 Revision delta (P)

This revision corrects only the two defects confirmed by the independent Sol high
delta review of the previous revision. Changed: §§1.2, 3 (the `Core.handoff` and
`CoreShell` graph lines), 3.1 (transfer signature and text, ack accessor), 5.2
(the `Live` lane split and the readiness bullet), 7 (steps 5b and 5e), 8.2
(closure actor predicate), 9 (actor-state row and note), 10 (affected rows), 10.1,
11 (lock-order and drop bullets), 13 (per-custody slot row), 14.1 (affected rows),
15 (affected controls and mutants) and the G4 row of §16. Everything else is
unchanged.

- **R1 refused transfer can neither drop nor relock under the child mutex.**
  Previously `transfer_with` took a `FnOnce` that captured the `CoreShell`, and
  the shell carried a `TransportHandoff` whose `Drop` acquires the child mutex
  (§11). On a refused precondition (e.g. `stop_requested`) the unused closure,
  and with it the shell and its handoff, was dropped inside that SAME mutex. That
  deadlocked before Entry removal, shell drop and bounded reap. Now:
  - the shell carries only an unarmed plain `Weak<NativeTransportCustody>`, with
    no `Drop` effect;
  - the `Drop`-bearing `TransportHandoff` is constructed only by
    `CoreShell::into_core`, i.e. only after a successful transfer;
  - `transfer_with` takes the shell by value plus a capture-free `fn` pointer. A
    refusal returns the SAME shell by value in `Refused<S>`, with the cell
    untouched;
  - the caller ends the child-mutex scope before using either result. A refused
    shell is therefore destroyed, after Entry removal, with no child mutex held,
    and a built Core leaves the section by value.

  The child never leaves the cell on refusal and stays independently
  Root-retained. On success it moves cell → Core with no local-only process
  window. A refusal records no `Offered` and no acceptance.
- **R2 terminal planning is not gated on `Live`.** The previous text made
  `Live` a precondition of terminal planning. But both of these revoke BEFORE the
  genuine Lost/nongrant terminal is planned:
  - `RevokedKnown`;
  - an unpolled `Core::drop`. Actual `Core::drop` (`native.rs:2248–2256`) calls
    `revoke()` and then `drop_phase` → `persist_saved_terminal` →
    `terminal_plan` (`native.rs:1876–1910,1276–1292`).

  Actual `plan_phase_terminal` (`terminal.rs:173–181`) deliberately takes a
  non-live `binding_snapshot`, and its normal mode uses
  `plan_owner_currency(…, true)`, which skips `is_live`
  (`native_phase.rs:230–243`). Now `Live` gates only new effects, input and
  normal currency. Terminal planning and nongrant transport closure require the
  SAME activation's retained ack (state `Live` or `Revoked`). They refuse an
  unacked actor (`Candidate`, or `Revoked` without an ack). They never reopen the
  actor, store `Live`, admit input or grant permission.
- **Footprint correction.** The inline `OnceLock<RegistrationAck>` is 4 words +
  tag, not "3 words + tag": `RegistrationAck` holds two `u64` values and one
  enum. The §13 row is otherwise unchanged.

Previous revision delta (closed; retained for traceability, not reopened):

- **D1 pre-Core child custody.** The earlier `into_owned()`/`native_pipes()` on
  `RetainedRawProcess` returned an `OwnedProcess`/pipes to the caller, so Entry,
  registry, shell and task preparation ran with the child in a local value. They
  are replaced by a custody-resident `NativeChildCell` that is upgraded in place.
  A single checked, infallible final transfer moves the process into the SAME
  Core and records `Offered` in the same section. First-poll `Accepted` and
  unpolled `DroppedUnpolled` are distinct (§§3.1, 7, 10, 12).
- **D2 candidate without an unissued registration.** The pre-admission candidate
  no longer takes an `Arc<NativeTransportRegistration>`. It is a concrete
  nongrant `Candidate` built from the SAME plan. The SAME-plan known commit yields
  a by-value `KnownTransportRegistration` that activates it once and in place: an
  inline `OnceLock<RegistrationAck>` plus a `Candidate → Live` CAS that cannot
  reopen `Revoked`. There is no heap allocation, hash, FS or await in that step,
  and equal SQL rows never create an actor or commit (§§3, 3.1, 5.2, 7, 8.1).
- **D3 Claude argv.** The preserved existing Claude vector has 12–18 elements.
  The inclusive bound is now ≤18 elements × ≤128 B. Role, model and effort
  controls are specified (§6, §13, §15).
- **C1 inventory composability.** The version observation (text + closed ack) and
  the last complete settled helper manifest are different objects. The separate
  readonly draft's nongrant completion is only a possible manifest link; it is not
  Prepared, registration, input or static-admission proof (§5.1).

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
  command: NativeTransportCommand            [sealed program/argv/env/cwd, §6]
  candidate: Arc<PhaseActor>                 [built before admission from SAME plan;
                                              state Candidate; carries no registration]
  phase: Mutex<TransportPhase>               [Planned|RegistrationUncertain|Registered|Held;
                                              short; never held with Store]
  child: Mutex<ChildCustody>                 [dedicated, poison-recovering, §7]
       cell: NativeChildCell                 [Empty|Raw(RetainedRawProcess)|Owned(OwnedProcess)|Transferred]
       creation: Creation                    [NotAttempted|Attempted|ReturnedChild|SpawnErr(class)]
       handoff: Handoff                      [None|Offered|Accepted|DroppedUnpolled]
       stop_requested: bool                  [set only under this mutex]
  control: OnceLock<mpsc::Sender<Control>>   [clone of the prebuilt sender, set at transfer]
  handoff_notify: tokio::sync::Notify
  observation / settlement / closure: Option<Arc<..>> (one slot each)
  reason: &'static str
NativeTransportStartPlan -> PreparedNativePhase -> NativePreparationActor -> PhaseLaunchParts
PhaseActor (A) -> NativePhaseSession (A + P fields) -> PhaseLaunchParts
NativePhaseSession.origin: Arc<NativeTransportStartPlan>   (P, immutable, set at candidate construction)
NativePhaseSession.ack: OnceLock<RegistrationAck>         (P, inline Copy value, one-time)
NativePhaseSession.state: AtomicU8 Candidate|Live|Revoked (P, replaces live: AtomicBool)
PhaseLaunchParts -Weak-> NativePreparationCustody (A)
NativePreparationActor -Weak-> custody, -Weak-> NativeSessions (A)
NativeSessions.preparations -Weak-> NativePreparationCustody (A)
Core (A) -> child: OwnedProcess, phase: Arc<PhaseActor>
Core.handoff: Option<TransportHandoff> (P) -Weak-> NativeTransportCustody
                                             [armed; constructed only by
                                              CoreShell::into_core after a
                                              successful transfer]
CoreShell (P)                                [start-future local; every Core field
                                              except child and handoff; holds the
                                              unarmed handoff target as a plain
                                              Weak<NativeTransportCustody> with no
                                              Drop effect; never holds a child]
```

No strong edge returns to Runtime, PhaseJobs, JobState, NativeSessions or the
NativeAdapter. The transport custody is a sibling slot of the Root-retained
preparation custody, and Native reaches it only through the existing Weak index.
The candidate's `origin` points at the plan, and the plan points at the Prepared
phase and the preparation actor; none of these points back to the transport
custody. Core reaches the custody only by Weak. No new registry or index is
added. This removes local return edges; it is not a proof that every existing
ownership graph is cycle-free.

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
    admission: &PhaseEffectAdmissionGuard) -> Result<KnownTransportRegistration>;
fn confirm_prepared_transport(&mut self, plan: &Arc<NativeTransportStartPlan>,
    admission: &PhaseEffectAdmissionGuard) -> Result<RegistrationProbe>; // §8
fn plan_transport_settlement(actor: &Arc<PhaseActor>,
    obs: Arc<NativeTransportObservation>) -> Result<Arc<NativeTransportSettlementPlan>>;
fn record_transport_settlement(&mut self, s: &Arc<NativeTransportSettlementPlan>) -> Result<()>;
fn plan_transport_closure(runtime: &RuntimeOwner, s: Arc<NativeTransportSettlementPlan>)
    -> Result<Arc<NativeTransportClosurePlan>>;
fn close_transport_observation(&mut self, c: &Arc<NativeTransportClosurePlan>) -> Result<()>;

// Known-commit token: produced only inside the two Store functions above for the
// SAME plan Arc passed in. By value, non-Clone, private fields, no row/ID/DTO
// constructor. Holds a refcount clone of that plan Arc and Copy facts only.
pub(crate) struct KnownTransportRegistration {
    plan: Arc<NativeTransportStartPlan>,
    ack: RegistrationAck,
}
#[derive(Clone, Copy)]
pub(crate) struct RegistrationAck {
    readiness: u64,      // P+1
    unit_version: u64,   // U+1
    source: AckSource,   // Committed | Confirmed
}

// Native phase protocol (execution/native/phase_protocol.rs).
impl PhaseActor {
    // Step 1, outside every lock. State Candidate, ack empty, origin = plan.
    fn prepared_candidate(plan: &Arc<NativeTransportStartPlan>, session: Session)
        -> Result<Arc<Self>>;
    // Step 4c, inside admission after SharedStore release. Pointer check, inline
    // OnceLock set, one CAS. No heap allocation, SQL, FS, hash or await.
    fn activate(&self, known: KnownTransportRegistration) -> Activation;
}
enum Activation { Live, RevokedKnown, Mismatch }
impl NativePhaseSession {
    // Copy of the ack set by this actor's own activation; None while unacked.
    // Read-only: the terminal and nongrant-closure lane (§5.2) uses this, never
    // `Live`. It cannot set, replace or clear the ack, or change `state`.
    fn registration_ack(&self) -> Option<RegistrationAck>;
}

// Process (execution/process.rs), additions only. Legacy OwnedProcess::spawn and
// every existing RetainedRawProcess method are unchanged.
pub(crate) enum NativeChildCell {
    Empty,
    Raw(RetainedRawProcess),
    Owned(OwnedProcess),
    Transferred,
}
// A refused transfer hands the caller's value back unchanged; nothing is dropped
// inside the method.
pub(crate) struct Refused<S> { pub(crate) shell: S, pub(crate) reason: &'static str }
impl NativeChildCell {
    fn adopt(&mut self, child: Child);                        // Empty -> Raw; infallible, no allocation
    fn qualify(&mut self) -> Result<()>;                      // Raw in place; Err leaves the SAME Raw
    fn upgrade_in_place(&mut self) -> Result<()>;             // Raw(qualified) -> Owned; returns no process
    fn take_native_pipes(&mut self) -> Result<NativePipes>;   // Owned; all-three presence check before any take
    fn transfer_with<S, T>(&mut self, open: bool, shell: S,
        build: fn(S, OwnedProcess) -> T) -> Result<T, Refused<S>>;
                                                              // checks first; refusal returns the SAME
                                                              // shell, cell untouched; success moves
                                                              // Owned into `build`; cell = Transferred
    fn hygiene(&mut self) -> Hygiene;                         // group signal if qualified, else direct start_kill
    fn try_reap(&mut self) -> Result<Option<ExitStatus>>;     // sync try_wait; marks reaped/unreaped=false
}
impl OwnedProcess {
    fn from_qualified(child: Child, pid: Pid) -> Self;        // private to process.rs; unreaped = true
}

// Native (execution/native.rs).
struct CoreShell {
    /* every Core field except `child` and `handoff` */
    handoff_target: Weak<NativeTransportCustody>,             // unarmed; plain Weak, no Drop effect
}
impl CoreShell { fn into_core(self, child: OwnedProcess) -> Core; } // pure field moves; the only
                                                                    // constructor of TransportHandoff:
                                                                    // handoff = Some(TransportHandoff
                                                                    // { custody: handoff_target })
struct TransportHandoff { custody: Weak<NativeTransportCustody> }  // exists only inside a Core
impl TransportHandoff { fn accept(&self) -> Result<()>; }           // first statement of Core::run
impl Drop for TransportHandoff { /* Offered -> DroppedUnpolled, notify; takes the child mutex */ }
```

`RegistrationProbe` is `{ Committed(KnownTransportRegistration), Absent, Held }`.

`upgrade_in_place` checks four things before moving anything: the cell is `Raw`,
it holds a child, it is not reaped, and `pid` is qualified. Only then does it
take the child out of the `RetainedRawProcess` and write
`Owned(OwnedProcess::from_qualified(child, pid))` into the SAME cell. The emptied
raw value drops with `child = None`, so its `Drop` hygiene sends no signal
(`process.rs:66–72`). No method of `NativeChildCell` returns an `OwnedProcess`
or `Child` to its caller.

`transfer_with` checks these five things before calling `build`:
- the cell is `Owned`;
- all three pipes are already taken;
- the process is not reaped;
- the handoff is `None`;
- `!stop_requested`.

The last two arrive as `open`, which the caller computes from the SAME
`ChildCustody` under the SAME child-mutex guard. If any check fails, `build` is
never called. The process stays in the cell, and the SAME `shell` is returned by
value in `Refused`. Nothing is dropped inside the method.

`build` is a capture-free `fn` pointer (`CoreShell::into_core`), so the method
holds no other value that a refusal could drop. `CoreShell` has no `Drop` impl.
None of its fields acquires the child, custody-state, `entries` or Store mutex
when dropped: its handoff target is a plain `Weak`, and the custody keeps its own
strong reference to the candidate. A field that would violate this is refused.

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
| `Child` returned | `confirmed` | `creation=returned_child`, `identity=<qualified\|unqualified>`, `pipes=<complete\|incomplete\|n/a>`, `handoff=<core_accepted\|core_dropped_unpolled\|precore_retained>`, `hygiene=<group_signal_attempted\|direct_kill_attempted\|unknown\|n/a>`, `reap=<exit:N\|signal\|unknown\|n/a>` |

`confirmed` means "a process was created and retained", never work success,
input, ACK or owned completion. No PID, argv, path, stderr or provider output is
stored. `Offered` is never a settlement value: the row is written only after the
handoff resolves (§7 step 8).

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

Native alone issues it, once, from the SAME custody, after all of the following:
- readiness known commit (A);
- version helper settlement **closed** (A + P flag);
- every required readonly Git qualification and artifact lease outcome settled
  (in progress separately, G1);
- profile/program/path qualification;
- quota admission settled as admitted (not parked).

It retains, without copying:
- the SAME `NativePreparationActor` and the SAME `NativePreparationCommit`;
- the SAME version observation, i.e. the owned version text plus the closed
  `native_phase_version` settlement acknowledgement;
- exactly one borrowed **last complete settled helper manifest** (defined below);
- the exact prepared readiness `PairRow` (`state=preparing`, version P);
- the exact prepared Unit image (full encoded body and version U);
- the governing digest already fixed from original marker parents.

It grants nothing by itself and cannot be issued from rows, IDs, a version string,
a successful helper exit or a helper completion.

The version observation and the expected inventory are different objects:

- **Version observation (text + ack).** It comes from the SAME version
  `NativeHelperSettlementPlan` and its closed settlement. It is used only for
  `native_invocations.native_version` (§4.2) and the provider version gate. It
  is never an inventory baseline.
- **Last complete settled helper manifest.** This is the complete `after` effect
  inventory of the LAST helper that settled in the SAME preparation custody's
  ordered helper history. Prepared borrows it; it is never merged, recomputed or
  derived from rows. If no readonly helper follows the version helper, it is the
  version settlement's `after`. If the separately authored readonly source draft
  supplies its nongrant completion, it is that completion's settled `after`. In
  that draft the completion is `NativeReadonlyHelperCompletion`, carrying a
  `NativeHelperHistoryCommit`. Those names come from that draft, are absent at
  76a58b6e, and are neither reviewed nor tested by this HOW. Each history link's
  `before` must equal the previous link's `after` exactly; otherwise the history
  is Held and Prepared stays absent. This manifest is the "prepared expected"
  inventory used in §§7 and 9.

A helper completion is a factual nongrant record. It is not registration, input,
ACK, static admission or Prepared proof. Full `PreparedNativePhase` stays absent
(G1) until every producer below is conjoined in the SAME custody:
- #19 prepared input;
- hooks/settings qualification;
- quota admission;
- Reviewer genuine artifact/readonly lease.

This HOW does not define, extend or widen the distinct marked readonly Git seam.

`reconcile_known_commit` (`preparation.rs:150–205`) must return immediately once
`version_closed` is set by the SAME successful `close_phase_version_observation`
that observed `current == settlement.after`; no later wake re-reads the version
inventory. Without this, any later readonly helper row or the transport row would
make version reconciliation conflict forever.

### 5.2 Registered actor continuity

- **Candidate (step 1, outside every lock).**
  - `PhaseActor::prepared_candidate(&plan, session)` runs after
    `plan_prepared_transport` returns.
  - It performs today's identity checks of `NativePhaseSession::registered`
    (`phase_protocol.rs:250–273`) against the plan's Session and
    `record_version = 1`.
  - It records `origin = plan.clone()` and `registered_readiness = P+1`
    immutably.
  - It allocates the complete actor: Arc, projection Mutex, retained-proofs
    Mutex, an empty inline `OnceLock<RegistrationAck>`, and
    `state = Candidate`.
  - It references no registration object, because none exists before the
    commit.
  - Only the transport custody holds it: it is not in `NativeSessions.entries`,
    not returned and not in any binding.
- **Candidate and Revoked are nongrant; `Live` gates only new effects, input
  and normal currency.** Only `Live` passes these checks:
  - `NativePhaseBinding::is_live`, `binding_snapshot`'s `live` field and
    `ConsumedPhaseInput::admitted` (`phase_protocol.rs:321–383`);
  - normal currency planning: `plan_owner_currency(…, false)` and
    `NativeOwnerPlan::validate_tx` (`native_phase.rs:224–243,353–364`). These
    are used for dispatch, ACK, projection and the normal transport settlement
    (§8.2).

  `revoke()` stores `Revoked` unconditionally. No transition leaves `Revoked`.
- **Known-registration lane (terminal planning and nongrant closure).** The
  following are NOT gated on `Live`:
  - terminal planning:
    - actual `plan_phase_terminal` (`terminal.rs:173–181`), which deliberately
      takes a non-live `binding_snapshot`;
    - its terminal currency `plan_owner_currency(…, true)`;
    - the terminal readiness reader;
  - the §8.2 nongrant transport closure.

  As an added conjunct to their existing checks (never a replacement), each of
  them requires both of these:
  - `registration_ack()` is `Some`: the ack set by this actor's own activation
    (Activation step 2 below) for its immutable `origin`;
  - the state is `Live` or `Revoked`.

  An unacked `Candidate` is refused (Held), and so is a `Revoked` actor whose
  ack was never set (stop before any known commit). The latter stays Held until
  the SAME plan's probe returns `Committed` and activation records the ack
  (`RevokedKnown`, §8.1).

  This lane is what lets the genuine terminal be planned in these cases:
  - `RevokedKnown`;
  - a stop after activation;
  - an unpolled `Core::drop`, which revokes before `drop_phase`
    (`native.rs:2248–2256,1876–1910,1276–1292`).

  Each then reaches the genuine Lost terminal of §10.1 or Core's existing
  terminal.

  The lane only reads the ack. It never does any of these:
  - store `Live`, or set, replace or clear the ack;
  - issue `ConsumedPhaseInput`, a permission or a new effect.

  With no consumed input, `owned_success` stays impossible (`PhaseActor::settled`
  requires `consumed()`, `phase_protocol.rs:181–206`). Normal terminal currency
  for a known `Completed` outcome keeps its existing current/Driver checks.
- **Known-commit token.** Only two outcomes produce a
  `KnownTransportRegistration`: `register_prepared_transport` returning `Ok`, and
  `confirm_prepared_transport` returning `Committed`.
  - The token is by value, for the SAME `&Arc<NativeTransportStartPlan>` passed
    in, inside the Store call, before SharedStore is released.
  - It carries a refcount clone of that plan Arc (no heap allocation) and Copy
    facts only.
  - Rows, equal-looking postimages, IDs and DTOs never produce a token.
  - A token never constructs an actor.
- **Activation (step 4c, admission held, SharedStore released).**
  `candidate.activate(known)` does the following, in order:
  1. Checks `Arc::ptr_eq(&known.plan, &origin)`; on failure it returns
     `Mismatch` (Held, no spawn). The private producer cannot cause this.
  2. Calls `ack.set(known.ack)`. The set is inline and one-time. The ack is
     retained as the factual known commit even when the actor is already
     revoked. A second set cannot happen through the private producer; if it
     did, the result is Held.
  3. Runs `state.compare_exchange(Candidate, Live)`. If it fails because the
     state is `Revoked`, the result is `RevokedKnown`: the registration is known,
     there is no spawn, and the path continues in §10 (`not_attempted`) and
     §10.1.

  Activation does no SQL, FS, hash, await or heap allocation. A stop that wins
  before activation therefore can never be reopened by a later activation.
- **Readiness.** `NativePhaseSession` gains `registered_readiness = P+1` and the
  `origin` Arc.
  - `plan_owner_currency` (`native_phase.rs:311–330`) and the terminal reader
    (`terminal.rs:305–316`) accept only `(P+1, !ended)` or `(P+2, ended)`, and
    only for an actor whose ack is set. Normal mode additionally requires
    `Live`; terminal mode accepts `Live` or `Revoked`. They no longer use the
    constants 2/3.
  - Rows of any other lineage refuse (Held).
  - The allocated-v1→registered-v2 producer is deleted.
- **Input continuity.**
  - Core receives a clone of the SAME `f.input` (`PreparedInput` captured in the
    allocation) and `f.model`/`f.effort`.
  - Registration keeps the existing check `encode_input(seed.input()) ==
    f.input_bytes` (`:1111`).
  - Dispatch keeps comparing the payload with `f.input.payload` (`:529–556`).
  - No `ManagedInput`, re-read Context or regenerated pin participates.
- **IDs.** The Session, invocation and pair are exactly the originally allocated
  IDs. After preparation, only two IDs are generated: `T` and, for Claude, the
  native session UUID. Both are generated once in the sealed plan and reused on
  every retry of that plan.

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

- **Program and cwd.**
  - program: `f.program`, absolute, canonical, regular file (rechecked here).
  - cwd: `f.path`, absolute, canonical.
  - Both are server-decided by the original allocation; never caller input.
- **Codex argv** is exactly `["app-server","--listen","stdio://"]` (3 elements).
  Codex model/effort are not argv; they travel only as JSON protocol values
  (`native.rs:2009–2028`).
- **Claude argv** keeps the existing vector and order exactly
  (`native.rs:454–482`), in these segments:
  1. `-p --input-format stream-json --output-format stream-json --verbose
     --session-id` (7 elements);
  2. the plan-generated session UUID (1 element, 36 B);
  3. `--permission-prompt-tool stdio --settings {"forceLoginMethod":"claudeai"}`
     (4 elements);
  4. `--model <f.model>` only when `f.model` is `Some` (2 elements);
  5. `--effort <f.effort>` only when `f.effort` is `Some` (2 elements);
  6. `--permission-mode plan` only when the role is not Executor (2 elements).

  So n is 12–18. Executor gives 12–16 and Reviewer gives 14–18. The inclusive
  bound is **≤18 elements, each ≤128 B**. That fits: the longest fixed element is
  32 B, the UUID is 36 B, and model/effort are at most 128 B. Total argv is
  ≤2304 B. The built vector is checked against this bound before the digest and
  before registration.
- **Role control.**
  - The role is `f.role`. Protected allocation admits only Executor or Reviewer
    (`adapter/native.rs:149–153`).
  - The plan cross-checks it against the prepared Unit kind (Executor↔Executor,
    Reviewer↔Reviewer) and the plan's Session role; any mismatch refuses.
  - Plan mode is keyed on `role != Executor`, so a Reviewer can never omit it.
  - Verifier/command-only Units refuse (`verification::is_command_unit`).
  - Plan mode is a cooperative native role policy, not host containment.
- **Model/effort control.**
  - Values come only from `f.model`/`f.effort`. Caller argv, rows, Context and
    config re-reads are never used.
  - The allocation predicate is rechecked: non-empty, ≤128 B, no control
    character (`adapter/native.rs:140–145`).
  - Additionally, the first byte must not be `-`, so the CLI can never parse a
    value as an option. This narrowing is protected-only and refuses before
    registration.
  - Each value is its own argv element directly after its flag: no `=` joining,
    no shell.
  - The SAME values feed the Session image (§4.2) and the Core run arguments.
- **No new inputs.** No shell, no caller argv, no new flag.
- **Environment:** the qualified profile overlay `profile.environment(cookie,
  socket)` only, as today. The inherited host/auth environment is unchanged and
  out of scope.
- **Process setup:** `stdin/stdout/stderr` piped, `process_group(0)`,
  `kill_on_drop(true)`.
- **Size:** the complete encoded plan is ≤64 KiB.
- **Failure:** any failure above refuses with no effect, before admission.

The command is not a `PhaseHelperAction`, not a callback and cannot be built from
a row.

## 7. Protected start sequence (P)

Continuation of `begin_phase_preparation` after `issue_prepared` succeeds:

1. **Outside all locks:**
   - Build `NativeTransportCommand` (§6).
   - Run `plan_prepared_transport` on a separate bounded readonly snapshot. It
     checks: current successor, Driver-live, `registration_attempt`, owner v1,
     prepared readiness, prepared Unit, budgeted inventory == Prepared's last
     complete settled helper manifest (§5.1), idempotency absence, and that the
     original SessionId is absent from `NativeSessions.entries`.
   - Build `PhaseActor::prepared_candidate` (§5.2), the `watch`/`mpsc(16)`
     channels and the frozen-terminal cell.
   - Allocate `NativeTransportCustody` with `phase = Planned` and `ChildCustody
     { cell: Empty, creation: NotAttempted, handoff: None, stop_requested:
     false }`.
2. **Short custody lock:** install the transport custody into the preparation
   custody slot exactly once (pointer-checked); refuse if abandoned. No SQL or
   await under it.
3. **`launch.admission().enter(launch)`** — may await before acquisition. A
   start future dropped here leaves no child by construction.
4. **Admission section** (synchronous; no await, FS, hash or heap allocation
   outside the SQL engine's own use inside the Immediate):
   a. Check `admission.validate_for`, `actor.validate_open`, that the custody is
      not stop-requested, and that the candidate is in `Candidate`.
   b. Lock SharedStore; run `register_prepared_transport` (§9 checks, §4.2
      writes); release SharedStore immediately after commit or rollback.
   c. On `Ok(known)`, call `candidate.activate(known)`:
      - `Live` → `phase = Registered`;
      - `RevokedKnown` → `phase = Registered`, no spawn (§10);
      - `Mismatch` → Held, no spawn.

      On `Err`, set `RegistrationUncertain`, do NOT spawn, and go to §8.
   d. Recheck `admission.validate_for`, `actor.validate_open` and candidate
      `Live`. Acquire the child mutex (poison-recovering) and require
      `!stop_requested`. Set `creation = Attempted`, then call
      `Command::spawn()` synchronously while holding that mutex.
      - On `Ok(child)`: the very next statement is the infallible,
        allocation-free `cell.adopt(child)`; set `ReturnedChild`.
      - On `Err(e)`: record only the bounded error class.

      Release the child mutex. Because the mutex is acquired before the spawn,
      adoption never waits on a fallible lock after a child exists.
   e. Drop the admission guard.
5. **Synchronous pre-Core segment.** There is no `.await` from 4e through step
   6's `tokio::spawn`, so the start future cannot be cancelled inside this
   segment. A panic unwinds only child-free locals; the child stays in the
   Root-retained cell.
   a. Child mutex section: `qualify()`, then `upgrade_in_place()`, then
      `take_native_pipes()` (all three present before any take). Release the
      mutex. Any `Err` leaves the SAME child in the cell (`Raw` or `Owned`) →
      `precore_retained`, §10.2.
   b. Outside the child mutex, build the `CoreShell`:
      - `Lines::new(stdin, stdout, limit)` and the channels;
      - the frozen-terminal cell;
      - the registered Unit/Session images taken from the plan's new images;
      - `record_version = 1` and `invocation = f.invocation_id`;
      - the SAME candidate Arc (now `Live`);
      - the unarmed handoff target `Weak<NativeTransportCustody>`. This is a
        plain `Weak`: no `TransportHandoff` exists yet, and dropping the shell
        never touches the custody;
      - a clone of the prebuilt control sender, kept for the 5e `control` set.

      All fallible or allocation-heavy work happens here, and the shell holds
      no child.
   c. Insert the prebuilt `Entry` (key = original SessionId) into
      `NativeSessions.entries`. On poison or collision, drop the shell (closing
      the pipes; no lock is held and its `Weak` has no `Drop` effect). The child
      stays in the cell → §10.2.
   d. Spawn the stderr drain task and store its JoinHandle in the shell.
   e. **Final transfer.** In one block scope, acquire the child mutex, compute
      `open = (handoff == None && !stop_requested)` from that SAME guard, and
      call `cell.transfer_with(open, shell, CoreShell::into_core)`:
      - All preconditions (§3.1) are checked before any move.
      - **On success:**
        - the `OwnedProcess` moves from the cell straight into the Core struct
          literal;
        - `into_core` wraps the shell's `Weak` into the armed
          `TransportHandoff`. This is the only place one is constructed;
        - every other field is a plain move of the already-built shell;
        - there is no `?`, allocation, lock acquisition or await between.
      - **Same-section writes after success.** Set `cell = Transferred`,
        `handoff = Offered` and `control` once, from the sender clone prepared
        in 5b. These are a plain enum store and a one-time `OnceLock` set that
        only this section performs. They cannot fail or panic.
      - **Leaving the section.** The block evaluates to
        `Result<Core, Refused<CoreShell>>` by value. The guard is released at
        the end of the block, before either value is used or dropped. No Core,
        `TransportHandoff` or shell is ever dropped while the child mutex is
        held.
      - **On `Refused { shell, .. }`** (stop requested, or an impossible state),
        `cell`, `creation` and `handoff` are unchanged: no `Offered` and no
        acceptance is recorded. After the guard is released, in this order:
        1. remove the Entry just inserted (synchronously, holding the `entries`
           mutex alone);
        2. drop the shell, which closes its pipes (its `Weak` has no `Drop`
           effect);
        3. keep the SAME child in the cell → §10.2.
6. **Spawn Core.** Call `tokio::spawn(core.run(f.input.clone(), f.model,
   f.effort, profile))` right after releasing the child mutex.
   - Between that release and the spawn, the only owner is the Core value: the
     SAME eager owner with the existing `Drop` (`native.rs:2248–2256` →
     `drop_phase`). Nothing fallible runs there.
   - If tokio drops the future unpolled (runtime shutdown, task cancelled before
     first poll), `Core::drop` revokes the actor and captures Lost through
     `drop_phase` (A). That terminal is planned through the known-registration
     lane (§5.2). The `OwnedProcess` drop group-signals (A). The
     `TransportHandoff` drop then turns `Offered` into `DroppedUnpolled` and
     notifies. It takes the child mutex only after `Core::drop`'s body has
     released the Store, and never while the child mutex is held.
7. **First poll.** The first statement of `Core::run`, before any wire I/O or
   Store access, is `handoff.accept()`:
   - In a short child-mutex section it moves `Offered → Accepted` and notifies,
     then releases the mutex.
   - Then, outside all locks, it takes the preparation actor's start-gate guard
     out of its one-time cell and drops it (§12).
   - If acceptance cannot be recorded (custody gone, poisoned mutex, or state
     not `Offered`), Core revokes its own owner and returns through its existing
     Lost path without dispatch, and the gate is not released by Core.
8. **Settlement and return.** The start future waits on `handoff_notify` for at
   most 5 s, holding no lock, until the handoff is `Accepted` or
   `DroppedUnpolled`.
   - It then records the transport settlement (§8.2) for the observed handoff.
     If the handoff is still `Offered` at the bound, it records nothing; the
     SAME custody records it on a later reconcile wake.
   - Unless the handoff is `DroppedUnpolled` (in which case it returns `Err`
     with the SAME launch, and Root keeps the custody Held), it returns
     `NativePhaseStart::Launched { handle, binding:
     phase.owner.binding_snapshot() }`.
   - Root's existing `bind_returned` (`phase_jobs.rs:286–301,356–374`) continues
     unchanged.

There is no process call inside the Immediate. No FS, hash or await runs while
holding the Store, actor, queue, custody-state or child mutex. Admission does not
span the pipe handshake, Core I/O or child execution. Core input
consumption/ACK remain their separate private protocol and are not implied by any
step above.

## 8. Same-plan reconciliation (P)

### 8.1 Registration

The custody retains the SAME plan, the SAME candidate and the original pair
across every outcome. At most one exact probe runs per due wake, with capped
100 ms–5 s backoff, under a fresh admission guard, by
`confirm_prepared_transport`:

| Probe observation (all images read in one Immediate) | Result |
| --- | --- |
| Complete postimage of every §4.2 row equals the plan's new images, transport row is v1 pending exactly | `Committed(KnownTransportRegistration)` for the SAME plan Arc → `candidate.activate` in that same admission section (step 4c); spawn may then proceed (step 4d) |
| Complete preimage of every row and all inserted keys absent | `Absent` → retry the SAME plan's Immediate (same `T`, Session, invocation, native UUID) only if all §9 normal checks pass |
| Mixed, foreign suffix, transport row not pending v1, stale current/Driver, unknown | `Held`; no spawn, no new plan, no row-to-actor construction |

Equal-looking rows never create an actor or a token; only the SAME retained plan
can produce a token, and only the SAME retained candidate can be activated. A
generic Store error, empty child cell or missing Session is never rollback
evidence.

### 8.2 Transport settlement and closure

`record_transport_settlement` is the normal writer, used while the registered
actor is `Live`. It requires:
- the selected DB;
- the SAME activated candidate (ack set, `origin` pointer equals the custody
  plan) and the SAME observation;
- the §9 normal predicates for the registered owner (after the G2 replacement);
- the transport row's exact v1 preimage (all nine columns).

It performs a single-row CAS update to the §4.3 v2 image; rowcount must be 1. It
deliberately does **not** compare the complete Unit inventory: after Core
acceptance the SAME actor's own setup/input dispatches legitimately append rows.
The pre-spawn baseline was already full-inventory exact in §9.

`close_transport_observation` is the nongrant writer, used after revocation or
currency loss, including `RevokedKnown` and `DroppedUnpolled`. It validates only:
- the SAME candidate in the known-registration lane (§5.2): `origin` pointer
  equals the custody plan, `registration_ack()` is `Some`, and the state is
  `Live` or `Revoked`. An unacked actor is refused;
- `validate_preparation_original` (immutable Source/marker lineage);
- the latest complete Unit image CAS (reusing `LatestUnitImage` from
  `version/closure.rs:11–153`, factored to a shared module);
- the latest exact Session record;
- the owner v2 and readiness (`registered P+1` or later `closed`) images;
- the transport row's exact v1 preimage.

It writes ONLY the transport row. It issues no permission, input, owned success
or Session/Unit change, and preserves any known terminal independently (Core's
frozen terminal is untouched). Drift keeps the SAME observation Held.

Both writers are idempotent on the exact v2 postimage: they return `Ok` without
writing.

## 9. Normal versus nongrant checks (P)

| Check | Registration Immediate | Spawn (4d) | Normal settlement | Nongrant closure |
| --- | --- | --- | --- | --- |
| Selected DB / selected vtable / SAME launch, custody, actor pointers | ✓ | ✓ (pointers) | ✓ | ✓ |
| Actor state (§5.2) | `Candidate` (4a) | `Live` | `Live` | ack set, `Live` or `Revoked`; unacked refused |
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
Terminal planning (§10.1) uses the nongrant-closure actor predicate plus its
existing terminal checks; it never uses the `Live`-only predicate.

## 10. Stop, abort and fault state table (P)

| Actual observation/fault | Custody state | Handling |
| --- | --- | --- |
| Stop wins admission before 4a | Planned | No registration, no spawn; preparation Held per approved §3.2 |
| Registration rollback proved (`Absent`) | Planned | No spawn; SAME plan retry only if all §9 checks pass; never new IDs/pins |
| Registration commit uncertain | RegistrationUncertain | No spawn; §8.1 probe; Held otherwise. A stop meanwhile revokes the unacked candidate; terminal planning and closure stay refused until a `Committed` probe activates it (`RevokedKnown`) |
| Known registration, candidate already `Revoked` at activation (`RevokedKnown`), or stop/revocation before 4d | Registered, `NotAttempted`, cell Empty | No spawn; settlement `resolved/not_attempted` by nongrant closure (authentic local precreation proof); terminal Lost via §10.1, admitted by the retained ack on the `Revoked` actor (§5.2); no input/ACK; never reopened |
| Activation `Mismatch` | Held | No spawn; impossible via the private producer; Held for attention |
| `Command::spawn` `Err` | Attempted, Empty | Settlement `unknown`; never NoChild/NoCurrentDispatch; no replay; Held for attention |
| Child returned, PID qualification fails | Raw (unqualified) | Child retained in the cell; no group signal; direct `start_kill` + bounded reap (§10.2); settlement `confirmed/unqualified/precore_retained`; never NoChild |
| In-place upgrade precondition fails (defensive) | Raw (qualified) | SAME child in the cell; group signal + bounded reap; `precore_retained` |
| Pipe presence check fails (any of stdin/stdout/stderr absent) | Owned, no pipe taken | SAME child in the cell; group signal + bounded reap; `pipes=incomplete`, `precore_retained` |
| Shell build or Entry insert fails (poison/collision) | Owned, pipes taken | Shell dropped with no lock held (pipes closed; unarmed `Weak`); SAME child in the cell; group signal + bounded reap; `pipes=complete`, `precore_retained`; terminal via §10.1 |
| Transfer refused (stop requested under the child mutex, or impossible state) | Owned | `transfer_with` returns the SAME unarmed shell by value; `cell`/`handoff` unchanged (no `Offered`/`DroppedUnpolled`); the child-mutex scope ends; then the Entry is removed synchronously and the shell is dropped with no lock held; SAME child in the cell; §10.2; `precore_retained` |
| Panic anywhere in step 5 | Raw/Owned (mutex possibly poisoned) | Only child-free locals unwind; a shell holds no armed handoff, so unwinding never re-enters the child mutex; the cell keeps the child; poison is recovered only for retention/hygiene; reconcile wake runs §10.2 |
| Start future dropped at the step-3 await | Planned | No registration, no child |
| Start future dropped during the step-8 wait | Offered/Accepted/DroppedUnpolled | Core (or its Drop) owns the child; the SAME custody records settlement on a reconcile wake |
| Engine timeout / Runtime Drop / `abandon()` with a child still in the cell (after a step-5 failure) | Raw/Owned | Root-retained custody keeps the child; `abandon()` extended to transport sets `stop_requested` under the child mutex and revokes the candidate; reconcile wake runs §10.2 and nongrant closure; custody memory Drop (teardown) runs the existing `RetainedRawProcess`/`OwnedProcess` Drop hygiene only and is not logical closure |
| Stop after transfer (Offered/Accepted) | Offered/Accepted | Under the child mutex observe the handoff; revoke the session actor (Core's fence observes it) and best-effort `try_send(Control::Cancel)` on the retained sender; Core's existing cancel/terminal path, whose terminal planning uses the known-registration lane |
| Core future dropped before first poll | DroppedUnpolled | `Core::drop` → `revoke` + `drop_phase` Lost terminal (A), admitted by the retained ack on the now-`Revoked` actor (§5.2); `OwnedProcess` Drop group signal (A); `TransportHandoff` Drop marks `DroppedUnpolled` (outside the child mutex, after the Store is released); settlement by nongrant closure `confirmed/core_dropped_unpolled`, `hygiene=group_signal_attempted`, `reap=unknown`; gate stays Held; start returns `Err` |
| First poll cannot record acceptance | Offered | Core revokes its own owner before any wire I/O; existing Lost path; no dispatch; gate not released by Core; Held |
| Core accepted | Accepted | Pre-Core responsibility ends; gate released once; settlement `confirmed/core_accepted` |
| Core aborted or dropped after acceptance | Accepted | Core's existing Drop/terminal path; `TransportHandoff` Drop is a no-op |
| Settlement/closure Store error | unchanged | SAME observation retained; one probe per wake; no state inferred from error |

### 10.1 Pre-Core terminal without Core

When a registration is known but no Core exists, a private Native-only
`NativeTerminal` is built as in `RegistrationGuard` (`native.rs:1137–1201`):
acquisition Missing, `HistoricalDraft`, `observed_work=Unknown`,
`disposition=Lost`. It is persisted ONLY through the SAME
`PhaseActor::terminal_plan` → `finish_phase_terminal` (`phase_protocol.rs:110–148`,
`native.rs:1286–1329`). With no consumed input, `owned_success` is impossible.

Here the actor is `Revoked`, or `Live` until the stop revokes it, and its ack
is set. Terminal planning admits it only through the known-registration lane of
§5.2 (`registration_ack()` is `Some`), never through `Live`. The lane neither
reopens the actor nor admits input. An unacked actor keeps this terminal Held.

This path depends on gate G2. Until G2 exists, the terminal stays Held while the
§8.2 closure still records the factual creation outcome. Generic
`RegistrationGuard`, `retire_execution_as` and `close_execution_session` are
never used for protected scopes.

### 10.2 Pre-Core child closure

This applies only while the cell is `Raw` or `Owned`:
1. In one short child-mutex section, run `cell.hygiene()`:
   - `Owned` or qualified `Raw` → process-group signal;
   - unqualified `Raw` → direct `start_kill` only; it never adopts another PID.
2. Release the mutex.
3. Poll `cell.try_reap()` every 20 ms, re-acquiring the mutex briefly for each
   poll, for at most 10 s. A reaped status sets `reaped`/`unreaped = false` in
   place, so no later Drop signals a recycled group.
4. Record `hygiene` and `reap=<exit:N|signal|unknown>` in the observation.

This is best-effort hygiene, not a process-death or isolation guarantee.

## 11. Concurrency, locks and ownership (P)

**Lock order.**
- The sequence is admission (async acquire) → SharedStore (sync, released before
  spawn) → child mutex (short).
- Never nested: the child mutex with SharedStore, with the `entries` mutex, or
  with the actor start-gate cell. Step 5c locks `entries` with the child mutex
  released. Step 5e locks the child mutex with `entries` released, and a refused
  5e removes the Entry only after its child-mutex scope has ended.
- Custody-state, Root job, `entries` and preparation-index mutexes are never
  held with SharedStore, or across an await, SQL, spawn or hashing.
- The child mutex is held across `Command::spawn()` in 4d. That is a
  synchronous syscall under admission, not under Store.
- **`TransportHandoff` Drop and the child mutex.**
  - The `TransportHandoff` Drop takes only the child mutex. It runs after
    `Core::drop`'s body has released the Store.
  - A `TransportHandoff` exists only inside a Core built by a successful
    `transfer_with`; a `CoreShell` carries only a plain `Weak`.
  - Nothing whose `Drop` acquires the child mutex (Core, `TransportHandoff`) is
    dropped while the child mutex is held. A built Core leaves the 5e section by
    value, and a refused shell is returned by value and dropped after the guard
    is released.
- Poison on the child mutex is recovered only to retain the child or run its
  hygiene; poison never grants an effect or an acceptance.

**Linearization.**
- A stop holding `control_admission` either precedes 4a (no registration or
  spawn), or waits until 4e. By then the custody holds either a no-spawn
  registration or the adopted child, and the stop targets that SAME operation
  through the custody (the Root caller is gate G3).
- `request_stop` sets `stop_requested` while holding the child mutex, so it is
  totally ordered with the 4d spawn check and the 5e transfer. Either no child
  is created or transferred, or the handoff is `Offered`/`Accepted` and the
  stop goes to Core.
- Candidate `revoke` is ordered against activation by the
  `Candidate|Live|Revoked` CAS (§5.2).
- `Runtime::drop` sets `stopping` without admission. The 4d recheck narrows that
  window but cannot close it; a child spawned in it is retained and closed
  nongrantly.
- This is linearization of admission, not a process-death guarantee.

**Ownership.**
- Root `JobState.preparation` strongly retains the preparation custody, and
  therefore the transport custody, independently of the Engine future.
- Until the step-5e transfer, the child is owned by that custody's cell and
  nowhere else. From then on it is owned by the SAME Core.
- Neither the custody nor the plan owns NativeSessions/NativeAdapter, Runtime,
  PhaseJobs or the JoinHandle. Core reaches the custody only by Weak.
- Different Tasks proceed concurrently after their own short admission sections.

## 12. Gate release and memory (P)

The actor's same-Unit gate (`preparation.rs:240,328–331`) is released exactly
once, by taking the guard out of its one-time cell and dropping it outside all
locks. That happens on exactly one of:
- (a) `handoff = Accepted`, released by the `accept()` hook after the child
  mutex is released;
- (b) a known `not_attempted` settlement plus a known terminal or closure;
- (c) a pre-Core created child that the custody observed reaped (`try_reap`
  returned a status), plus a known transport closure and a known terminal.

`Offered` that never resolves, `DroppedUnpolled` (the child was signalled by
Drop, but the custody never observed a reap), and every other uncertain outcome
keep the gate and are visibly Held. No forced unlock permits a replacement start.
Removing the Root job entry additionally requires no remaining reconciliation
responsibility. Release never authorizes another preparation or owner. Memory
teardown is not logical closure.

## 13. Finite inclusive limits (checked before copies) (P unless marked A)

| Item | Bound |
| --- | --- |
| Effect inventory before registration | ≤252 rows (transport + up to 2 setup + 1 input ≤256); complete ≤256 rows, ≤2 MiB all-column framing, body ≤8192 B, VM budget (A, `version.rs:13–73`) |
| Transport effect | idempotency 53 B, expected_target ≤256 B, body ≤8192 B, receipt ≤16 entries |
| Session record / Unit / invocation / readiness / owner read | ≤32 KiB / ≤16 KiB / `INVOCATION_BYTES` / ≤4096 B / ≤32 KiB (A) |
| Command plan | Codex exactly 3 argv; Claude 12–18 argv, inclusive ≤18 elements × ≤128 B (≤2304 B total); model/effort ≤128 B each (A allocation bound); env overlay ≤64 entries; encoded ≤64 KiB; profile file ≤64 KiB+1 read (A) |
| Version text reused | ≤64 KiB combined capture (A) |
| Custodies | one transport custody per operation; ≤128 operations (A, `MAX_JOBS`, preparation index) |
| Per-custody slots | plan, command, candidate, child cell, retained control sender, observation, settlement, closure: one each; no history lists. `CoreShell` is a start-future local, holds no child and no armed `TransportHandoff` (only one plain `Weak`); a refused shell is one returned value, dropped outside the child mutex |
| Candidate activation | one inline `OnceLock<RegistrationAck>` (4 words + tag) per candidate; one CAS; no heap allocation |
| Plan memory | images ≤~2.2 MiB per plan without copying the borrowed expected manifest |
| Reconciliation | ≤1 exact probe per wake, 100 ms–5 s capped backoff |
| Start-future handoff wait | ≤5 s, no lock held; unresolved → later reconcile wake |
| Pre-Core stop | group/direct signal then ≤10 s reap polling at 20 ms (as version helper); not death proof |
| Control channel / line limits | mpsc 16; Claude `LINE_LIMIT` 2 MiB, Codex 4 MiB (A) |

An overflow refuses before the INSERT and before spawn. A settlement overflow
refuses before the write, with the observation retained.

## 14. Impact analysis

### 14.1 Changed symbols (P)

| Symbol (file) | Change | Callers/consumers checked (V) |
| --- | --- | --- |
| `start_with_launch` (`native.rs:215`) | remove `launch` parameter and every protected branch (`:237–245,429–431,492–497,500–560,642–645`) | only `start_inner` (`:194`, passes `None`); public legacy behavior unchanged |
| `start_phase_inner` (`native.rs:207`) | continue into §7 only after `issue_prepared` | `start_phase` ← `NativePhasePort` ← `phase_jobs.rs:253–257` |
| `NativePreparationCustody` (`native/preparation.rs:9–231`) | add `prepared`, `transport`, `version_closed`; extend `abandon`/`Drop` (set `stop_requested` under the child mutex, revoke candidate) and `reconcile_known_commit` | `phase_jobs.rs:140,195–228,283,393`; `begin_phase_preparation` |
| `NativeVersionHelperCustody::reconcile` (`version.rs:144`) | early return when closed; `physical_command` profile part factored | `preparation.rs:173`, `version.rs:357` |
| `PhaseActor::registered`, `NativePhaseSession::registered` (`phase_protocol.rs:62,250`) | replaced by `prepared_candidate` + `activate` (§5.2); identity checks retained | only `native.rs:535` (deleted) |
| `NativePhaseSession.live: AtomicBool` (`phase_protocol.rs:16`) | replaced by `state: AtomicU8` `Candidate\|Live\|Revoked` plus `origin`, `registered_readiness`, `ack` and the read-only `registration_ack()` accessor | `revoke` (`:318`), `binding_snapshot` (`:321–334`), `NativePhaseBinding::is_live` (`:358`), `ConsumedPhaseInput::admitted` (`:375–383`): each reads `Live` only; terminal and nongrant-closure consumers read `registration_ack()` instead (next row, §8.2) |
| `plan_owner_currency` (`native_phase.rs:233`), `plan_phase_terminal` and terminal reader (`terminal.rs:173–181,302`) | readiness version from the activated actor; normal mode (`terminal_ending = false`) requires `Live`; terminal mode replaces `terminal_ending \|\| is_live()` with `terminal_ending ∧ registration_ack().is_some()` (state `Live` or `Revoked`), refusing unacked actors; `plan_phase_terminal` keeps its non-live `binding_snapshot` and adds the same ack conjunct | dispatch, ACK, projection (normal); terminal via `PhaseActor::terminal_plan` ← `persist_saved_terminal` ← `drop_phase`/Core terminal and §10.1; `actual_native_authority` |
| `plan_phase_registration`, `register_phase_session`, `validate_phase_preparation`, `NativeRegistrationPlan` (`native_phase.rs:887–1209,1384–1479`) | deleted; replaced by `transport.rs` | only `native.rs:244,504,520` |
| `original_owner`, `check_owner_indices`, `no_registration`, `registration_unit`, `registration_attempt` | reused unchanged | preparation and transport plans |
| `Inventory`, `EffectImage`, `InventoryBudget` (`version.rs`), `LatestUnitImage` (`version/closure.rs`) | visibility to `pub(super)` / shared module | version + transport |
| `RetainedRawProcess` (`process.rs:17–99`) | unchanged (no `into_owned`/`native_pipes`); wrapped by the new `NativeChildCell` | version helper use unchanged |
| `NativeChildCell`, `Refused<S>`, `OwnedProcess::from_qualified` (`process.rs`, new) | in-place upgrade, pipe take, checked transfer taking the shell by value with a capture-free `fn` builder and returning it on refusal, sync hygiene/reap | transport custody only; `OwnedProcess::spawn` and its callers unchanged |
| `Core` (`native.rs:1341`) | add `handoff: Option<TransportHandoff>`; protected construction only via `CoreShell::into_core` inside a successful `transfer_with`, which is the only constructor of `TransportHandoff` | legacy construction (`native.rs:608`) passes `None`; legacy behavior unchanged |
| `Core::run` (`native.rs:1696`) | first statement `handoff.accept()` when present | phase Core only |
| generic effect writers (`effects.rs:41,129,247`) | refuse `native_phase_*` | all generic callers; legitimate kinds unaffected |
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

**Compatibility (Store-level, compiled).** These require:
- the new intent/settlement images decode through the actual old10
  `effect_tx`/`managed_effects`/epoch fence/`EffectImage::decode`;
- UNIQUE idempotency rejects a second intent;
- the layout catalogue is unchanged;
- generic reserve/reconcile refuse `native_phase_*`;
- compiled old10 (76a58b6e) generic reconcile interference → private CAS
  conflict/Held;
- the 252/253 baseline, 256/257 and inclusive 2 MiB boundaries;
- readiness `(P+1,!ended)`/`(P+2,ended)` accepted for an activated actor
  (`Live` in normal mode; `Live` or `Revoked` with ack in terminal mode), and
  v2/v3, other lineages and an unacked actor refused.

**Pure protocol controls (compiled; not lifecycle proof).**
- **Candidate.** A built candidate:
  - is not live;
  - yields a non-live `binding_snapshot`;
  - fails `ConsumedPhaseInput::admitted`.
- **Activation.** These cases are required:
  - a token for the SAME plan Arc → `Live` and ack set;
  - a token for a different plan Arc with byte-equal images → `Mismatch`, no
    ack, not live;
  - `revoke` before activation → `RevokedKnown`, ack set, still not live,
    nothing reopens it;
  - a second activation is refused.
- **Known-registration lane.** These cases are required:
  - **`RevokedKnown` actor.**
    - Admitted: terminal-mode `plan_owner_currency` and `plan_phase_terminal`
      (Lost);
    - `owned_success` is false;
    - still not live, and `registration_ack()` is unchanged;
    - refused: `ConsumedPhaseInput::admitted`, normal-mode currency and normal
      settlement.
  - **Unacked actors.** An unacked `Candidate`, and a `Revoked` actor with no
    ack, are refused by terminal planning and by nongrant closure.
  - **Unpolled Core drop.** A `Live` actor revoked by `Core::drop` before its
    first poll has its Lost terminal admitted and is not reopened.
- **Argv.**
  - Claude Executor with no model/effort → exactly 12 elements in the existing
    order; Reviewer with both → exactly 18, ending in `--permission-mode plan`;
    each element ≤128 B.
  - A synthetic 19th element is refused before the digest.
  - Codex → exactly 3 elements; model/effort appear only in the protocol JSON.
  - A model/effort with a leading `-`, a control character, empty, or 129 B is
    refused before registration with no effect.
  - A role/Unit-kind mismatch is refused.

**Child-cell controls** (controlled child such as `/bin/sleep`; compiled).
- Inject a fault after each of `qualify`, `upgrade_in_place`, the pipe-presence
  check, the shell build, the Entry insert and the transfer precondition. Each
  must leave the SAME child in the cell, with the PID unchanged and the variant
  as specified in §10.
- No `NativeChildCell` API returns an `OwnedProcess`/`Child`.
- A transfer refused by `stop_requested`, and by each other precondition, must
  satisfy all of the following:
  - it returns the SAME shell (pointer-identified);
  - it completes within a bounded test timeout, with no deadlock;
  - it leaves `handoff = None` (no `Offered`/`DroppedUnpolled`);
  - the Entry is removed, then the shell is dropped with the child mutex
    observably free;
  - §10.2 then runs on the SAME child.
- A `CoreShell` dropped while the child mutex is held by the test does not
  block (no custody lock on drop).
- §10.2 group-signals or direct-kills, reaps within the bound, and leaves no
  later Drop signal after the reap.
- A successful transfer leaves the cell `Transferred` and the handoff `Offered`
  in one section.
- Dropping a Core value before its first poll yields `DroppedUnpolled`, keeps the
  gate held and captures a Lost terminal.
- A first poll yields `Accepted` and releases the gate exactly once.
- An unrecordable acceptance does no wire I/O.

**Inventory composability** runs only after the separate readonly draft source
exists; it is not claimed here. Prepared's expected manifest must be the last
link's `after`. A broken link (`before ≠` previous `after`) must keep Prepared
absent. A helper completion alone must never issue Prepared, a token or an
activation.

**Causal actual-producer controls.** These run only after G1/G2 allow a genuine
positive; until then, record the SETUP refusal and credit no mutant kill.
- The real Goal/Driver/Source/marker/job/actor chain reaches
  `PreparedNativePhase`.
- Pause after the registration commit, before spawn, and race a real
  `control_admission` stop (both orders). In the stop-first order, the genuine
  Lost terminal is planned through the ack lane on the `Revoked` actor.
- Pause after the Entry insert, before 5e, and set `stop_requested` through the
  real stop path. Required outcome:
  - the transfer is refused, with no deadlock;
  - the Entry is removed and the shell dropped outside the child mutex;
  - the SAME child is reaped;
  - settlement is `precore_retained`;
  - no `Offered` is recorded and no input/Task write happens.
- An injected commit error with an actual committed postimage → no spawn until
  exact confirmation activates the SAME candidate.
- A true rollback → same-plan retry with identical IDs.
- An injected PID/pipe/shell/Entry/registry fault after a real `Command::spawn`
  → the SAME child is retained in the cell and reaped, with no input/Task write
  and never NoChild.
- Spawn `Err` → `unknown`, no replay.
- Start-future Drop during the step-8 wait, and an unpolled Core task (runtime
  shutdown) → `DroppedUnpolled`/Held, with the Lost terminal admitted through
  the ack lane.
- Core accepted → gate released exactly once, and Weak probes show no
  custody/actor/plan cycle.
- Another Task proceeds after the short section.
- A controlled protocol peer exercises stdio (not the official CLI, auth or
  hooks).

**Required compiled mutants** (each must fail the intended consumer assertion):
- spawn under SharedStore;
- adopt after `qualify`;
- child mutex acquired after spawn;
- spawn on registration `Err`;
- omitted admission recheck;
- generic `validate_authority` reintroduced;
- inventory equality removed;
- readiness constant 2/3 restored;
- settlement full-inventory CAS (breaks after setup dispatch);
- `version_closed` early return removed;
- candidate live before activation;
- activation as `store(Live)` instead of CAS (reopens `Revoked`);
- activation without the plan pointer check (accepts the equal-row foreign
  token);
- idempotency key per-plan instead of per-operation;
- upgrade returning `OwnedProcess` to the caller with Entry insert after it
  (fault leaves no child in the cell);
- pipes taken before the all-three presence check;
- transfer without the `stop_requested` check;
- refused transfer drops the shell inside the child-mutex section, e.g. via a
  closure-captured shell (the stop-refusal control times out);
- `TransportHandoff` constructed in `CoreShell` before transfer (a refused-shell
  drop relocks the child mutex or fakes `DroppedUnpolled`);
- Entry removed or shell dropped before the child-mutex scope ends on refusal;
- handoff set to `Offered` outside the transfer section;
- gate released on `Offered`;
- the `TransportHandoff` Drop not marking `DroppedUnpolled`;
- terminal planning gated on `Live` (the `RevokedKnown` and unpolled-drop Lost
  terminals are refused);
- terminal planning or nongrant closure without the ack conjunct (an unacked
  `Candidate` is admitted);
- the terminal lane storing `Live` or admitting input on a `Revoked` actor;
- Claude argv bound 16 restored (refuses the 18-element Reviewer vector);
- plan mode keyed on `Verifier` instead of `!= Executor`;
- the leading-`-` model/effort refusal removed.

## 16. Unresolved genuine seams and phase gates

| Gate | Owner | Required before transport composition |
| --- | --- | --- |
| G1 `PreparedNativePhase` issuer: full #19 prepared input, readonly Git source/seal (being implemented separately), Reviewer genuine artifact/readonly lease, hooks/settings qualification, quota admission vs parking | Native (B) with A/Root | Absent; transport stays refused |
| G2 Registered-owner factual predicates replacing `validate_authority` in `NativeOwnerPlan::validate_tx/validate_terminal_tx` (dispatch, ACK, projection, terminal) | input/ACK increment | Without it every spawned child fails its first dispatch; composition stays refused |
| G3 Root stop caller targeting the SAME custody (`request_stop`) | Root | Cooperative shutdown keeps Held only |
| G4 `NativeChildCell` in-place upgrade and checked transfer (refusal returns the unarmed shell by value; `TransportHandoff` constructed only by a successful transfer), `TransportHandoff` first-poll acceptance / unpolled-drop marking, and gate release | Native | Part of this increment |
| G5 Installed composition / static admission | Root | Remain closed |
| G6 Inherited Stop repeated-failure and request-identity findings | existing owners | Open; not addressed |

Increment order:
1. this HOW's independent Sol high review, now its delta re-review;
2. the G1 producer;
3. transport source (§§4–12), with G2 in the same or an earlier reviewed
   increment;
4. actual-producer controls;
5. later user-approved four-Task and real macOS/Linux qualification.

Until each real source exists, the corresponding effect stays refused or Held.
rururunx is not a security sandbox; work, immutable result and best-effort cleanup
remain separate.
