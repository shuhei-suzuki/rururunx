# Issue 43: Composed Workflow native-contract activation producer (CA)

## 1. Status, pins and categories

1. This is a proposed requirements addendum and authority/contract HOW for the one missing prerequisite lane between Workflow activation and the original marker. It connects, and does not replace, the [managed binding](issue-43-managed-binding-design.md) (§3 `workflow_native_contracts`, §8), the [Runtime Driver producer](issue-43-runtime-driver-producer-design.md) (§§2, 5 and the finite preparation correction), the [prepared producer](issue-43-native-prepared-producer-design.md), the [marker dispatch](issue-43-original-source-marker-dispatch-design.md) and the [installed composition (G5)](issue-43-native-installed-composition-design.md). It does not update the master design and claims nothing implemented.
2. **Pins.** Source facts are read at the frozen clean base `a21cdabfeb63f826a9421741e76dc80a826cc101`. G5 component facts are read with `git show` at `c51af44733f0f6e42ec09bf0914a6fd92055192c`, whose merge base with `a21` is `a21`. Paths are relative to `crates/rrx/src/`. Line numbers are at `a21` unless marked `c51`; implementers locate symbols by name.
3. **Categories.**
   - **V (verified):** read in source at the stated pin. No build, test, lint, Agent or OS run was executed for this document.
   - **P (proposed):** this contract; not implemented.
   - **A (acknowledged):** reported by Root or by the G5 owner and relied on here without re-verification.
4. **Never authority here:** a successful SELECT or decode of any contract row, configured aliases or capability metadata, a public `AgentRegistry`/`WorkflowEngine` constructor, test-only constructors, SQL inserted to make a positive fixture pass, a boolean or optional switch, copied DB state, Native permission bypass, a Task.version-changing binder, or a later UPDATE of immutable contract history. Activation is necessary for a marker and is never sufficient for Native dispatch. rrx is not a sandbox. Official CLIs run on the host with the user's usual HOME, config, hooks and auth. Nothing here reads, copies or transfers them, and no VM, container, separate user, root, new HOME or outer sandbox is used. Licensing and the #46/#60 process observation are untouched.

### 1.1 One-line acceptance condition (P)

Every genuinely Driver-activated Workflow commits, in its own activation transaction, one immutable composed contract derived from the SAME installed composition that covers every Native role and alias its configured phases need, and every later original marker for that Workflow is admitted only by that contract plus all existing checks.

## 2. Verified defect (V)

| Item | Fact |
|---|---|
| Consumer | `plan_marker_publication` (`state/managed_binding/publication.rs:235–280`) requires an existing row with `contract_state='composed'`, `version=1`, the exact Workflow/P/G/T, `owner_epoch = unit.owner_epoch`, `origin = allocation port origin_id` and `profile_digest = unit.profile_digest`, with a canonical body ≤4096 B. `validate_contract` (`:289–298`) repeats it in the marker and confirmation Immediate transactions (`:426`, `:477`) |
| No producer | No production INSERT writes `composed`. The only writers are the migration (`state/managed_binding/schema.rs:510–535`, `legacy_held`, epoch 0, NULL profile) and a schema-mechanics test (`schema_tests.rs:511`) |
| Activation today | `Engine::initialize_driven` (`workflow.rs:922–935`) → `initialize_inner` (`:1027–1057`) → `Store::activate_driven_workflow` (`state/mod.rs:613–632`) → `put_workflow_transition_inner` (`:665–1110`), which inserts the Workflow at version 1 (`:1078–1087`) and only `verification::install_contract` (`:1091–1093`; `state/execution/verification.rs:24–56`) |
| Per-port origin | `NativePhasePort::installed` mints a fresh `origin_id` per alias port (`adapter/native.rs:49–59`), created per alias in `AgentRegistry::from_managed_config` (`adapter.rs:404–421`) |
| Per-Unit profile | `Unit.profile_digest` is the `ResourceProfile` digest (`execution/attempts.rs:417,678`), which hashes the Unit ID, port range and Unit root (`execution/resources.rs:209–227`). It is unique per Unit and therefore per phase |
| One row per Workflow | `workflow_id` is the primary key, and triggers refuse replace, delete and every UPDATE (`managed_binding/schema.sql:2–14,140–142`) |
| Protection side effect | When a contract exists, `binding_record_{INSERT,UPDATE,DELETE}` require an exact `records` permit for the Workflow, and `binding_record_workflow_identity` requires `version = old + 1` (`schema.rs:316–336`). `binding_operation_allocation` requires `c.owner_epoch = operation.owner_epoch` (`schema.sql:191–193`) |
| Driver-lane writers | After activation, the gate edges and the first-Executor reservation write the Workflow through `put_workflow_transition_inner` with no `records` permit (`state/mod.rs:633–662,1085–1086`). `runtime/`, `cli/` and `main.rs` have no generic Workflow writer caller (grep) |
| Roster | Every class's phase list contains `ImplementationReview` (`workflow.rs:153–188`), an `Actor::Reviewer` phase (`:103–114`). Reviewer selection reads `task.reviewers` (`:1317–1321,1440–1447`) |
| G5 stop (A) | At `c51`, `drive` calls `initialize_driven(task,&sources,&lifetime)` (`runtime/task_driver.rs`, c51), and the genuine C5/C6 setup reaches Source preparation and marker planning. Planning refuses with "actual composed Workflow activation unavailable", so no marker or Native invocation runs and no positive or mutation credit exists |

**Conflict.** An immutable per-Workflow row cannot equal a per-port origin and a per-Unit profile for more than one phase. If it is pinned to the first Executor, every Reviewer (another port UUID) and every later Unit (another profile) is refused, and the contract cannot be updated. A first-executor pin is therefore not a prerequisite for the requested full Workflow, and this design does not use one.

## 3. WHAT addendum (P)

- **R1 Producer.** The only composed-contract writer is the genuine Driver activation transaction. Its inputs are the SAME `InstalledDriverComposition` held by the claim, the retained `DriverPreparationAdvance` initial-input plan, the actual Workflow Record created in that transaction and the selected owner's database and epoch.
- **R2 Full-Workflow coverage.** The contract covers the executor alias in the Executor role and every distinct `task.reviewers` alias in the Reviewer role, each of which must be an installed, declared Native port of that composition. Per-phase identity (Unit, profile, Session, port origin, allocation) is checked at each marker, never pinned in the contract.
- **R3 Unchanged transitions.** The activation keeps its existing single Task version bump and Workflow version 0→1. Record-only Session binding still never changes Task.version. No other row is added to the activation footprint.
- **R4 Nongrant.** A contract admits only an original marker plan that also passes every existing owner, Context, Driver, Unit, Source, budget and private-vtable check. It grants no helper, start, binder or terminal effect.
- **R5 Preservation.** `legacy_held` rows, the schema, `SCHEMA_VERSION`, triggers and the permit catalogue are unchanged. A contract is never updated, replaced or deleted.
- **R6 Gates unchanged.** Reviewer markers (PR-2), later Executors and retries, restart continuation, four Tasks across two Projects on both OSes with the official CLIs, truthful best-effort cleanup and every G5 §7 merge gate stay mandatory. This lane passing satisfies none of them.

## 4. Contract semantics (P)

### 4.1 Premise changes, with preserved conditions

| Column (no schema change) | Old predicate (`publication.rs:246–268`) | New meaning | Why (V) | Preserved equivalent authority |
|---|---|---|---|---|
| `origin` | = the allocation port's `origin_id` | = the installed registry's `installation` UUID (§5.1) | a per-port UUID cannot cover two aliases | The port `origin_id` stays in the member digest (§4.2), so a per-port check is retained and strengthened with alias, provider, program, declaration and role |
| `profile_digest` | = `unit.profile_digest` | = the roster digest over the member digests | per-Unit and unknowable at activation | The Unit profile stays bound per marker by `marker_plan.rs:128` (`u.profile_digest == facts.profile_digest`), the full Unit body CAS (`:147`, `publication.rs:429–434`), `managed_phase_inputs.profile_digest` (`marker_rows.rs:176,275`), `phase_protocol.rs:586` and `ResourceProfile::validate` (`resources.rs:105–121`) |
| `owner_epoch` | = `unit.owner_epoch` | unchanged: the activation epoch | — | unchanged, and so is the trigger (`schema.sql:191–193`) |

G5 §3.2 says the Reviewer alias "is not required". That premise changes only for configuration. Every class has a Reviewer phase (§2), so a Task without an installed, declared reviewer cannot finish its Workflow, and the issuer now refuses it before any Source or Git effect (§5.2). G5's rule that a Task is not refused merely because later-phase code is unimplemented still holds.

### 4.2 Body

The private `NativeContract` (`publication.rs:24–36`, `deny_unknown_fields`) gains one field, `members: Vec<String>`: sorted, unique, lowercase 64-hex values, at most 9 (one executor and at most 8 reviewers). No composed row exists, and `legacy_held` bodies are never decoded as `NativeContract`, because the query filters on `composed`, so this is not a stored-data migration. The encoding is `canonical::encode`, and the body must pass `Body::<NativeContract>::decode(raw, 4096)` before SQL.

- `member = digest(b"rrx.native-roster-member/v1\0", canonical{role, alias, provider, program, declaration_digest, port_origin})`. Every value is read from the SAME `Arc<NativePhasePort>` and its upgraded adapter.
- `profile_digest = digest(b"rrx.native-roster/v1\0", canonical(members))`.

## 5. HOW (P)

### 5.1 Installation identity

`AgentRegistry::from_managed_config` mints one `installation: Uuid` and passes it to `NativePhasePort::installed(adapter, installation)` (still `pub(super)`). The port exposes `pub(crate) fn installation_id()`, and `NativeAdapter` exposes a `pub(crate)` read of the existing `declaration_digest` (`execution/native/compat.rs:151`, today `pub(super)`). These are identities, not grants. Public `from_config` creates no ports.

### 5.2 Roster issuer and prefilter

```rust
// state/managed_binding/activation.rs (new, private module; re-exported pub(crate) types only)
pub(crate) struct ActivationRoster { installation: Uuid, members: Vec<String>, digest: String, owner: Arc<RuntimeOwner> } // non-Clone, no Deserialize
pub(crate) fn member_digest(role: SessionRole, port: &NativePhasePort) -> Result<String>; // pure; upgrades the SAME adapter
// runtime/installation.rs (c51 module): the ONLY constructor call site
impl InstalledDriverComposition {
    pub(crate) fn activation_roster(&self, task: &Task) -> Result<ActivationRoster>;
}
// workflow.rs: nongrant accessor over the Engine's own registry
impl WorkflowEngine { pub(crate) fn installed_native_port(&self, alias: &str) -> Result<Arc<NativePhasePort>>; }
```

`ActivationRoster::new` is `pub(in crate::runtime)`-reachable only through `activation_roster`. Its fields are private, and the state side reads it only through accessors. `activation_roster` refuses unless all of these hold:

- `composition.is_current()` and `task.id == original_task().id`;
- `task.executor == selected().alias()`, with the executor port pointer-equal to `selected()`;
- `task.reviewers` has 1–8 distinct aliases;
- every member port comes from `engine().installed_native_port(alias)`, `selected_adapter()` upgrades, the port's owner is the composition owner, the provider is `claude|codex`, the declaration is `Some`, and all ports share one `installation_id`.

It performs no IO and takes no Store lock. `Runtime::installed_driver_composition` (c51) runs the same reviewer conjuncts as a nongrant prefilter, and their refusal is `SkipReason::Composition` with no write. Activation re-derives the roster authoritatively.

### 5.3 Retained activation plan

```rust
pub(crate) struct NativeActivationPlan { workflow: RecordId, scope: Scope, epoch: u64, instance: String,
    state_path: PathBuf, installation: Uuid, digest: String, body: String, insert: ExactRowMutation } // private fields
pub(crate) fn plan_native_activation(roster: &ActivationRoster, record: &Record, task: &Task) -> Result<NativeActivationPlan>;
```

The plan requires `record.kind == Workflow`, `record.version == 0`, `record.scope == task.scope()` and `epoch = roster.owner.epoch() > 0`, and it fills the instance and path from that owner. The body must be ≤4096 B and must round-trip through `Body::decode`. `DriverReadTicket::plan_initial_input` (`state/runtime/driver/preparation.rs:41`) gains a parameter `activation: NativeActivationPlan`. The plan is stored in `InitialInput` and must match the input's scope and record ID, so it is retained in the SAME `Arc<DriverPreparationAdvance>` that the Driver slot retains before SQL (finite preparation correction). No other constructor exists.

In `initialize_inner` (`workflow.rs:1035–1057`), the driven tuple carries the composition: `initialize_driven(task, composition, lifetime)` replaces `&sources`, and `drive` passes `claim.composition()` (c51). The roster and plan are built after `verification_activation` and before `store.lock()`.

### 5.4 Activation transaction: placement, checks and footprint

In `put_workflow_transition_inner`, only for `Activation(_, Some(plan))`, the plan writes the contract after `install_contract` (`:1091–1093`) and before `source_recovery::after_write` (`:1094`):

```rust
impl DriverPreparationAdvance {
    pub(in crate::state) fn write_native_activation_tx(&self, tx: &Transaction<'_>,
        permits: &PrivatePermitManager, workflow: &Record) -> Result<()>;
}
```

Inside the existing Immediate transaction, it refuses unless all of these hold:

- `tx.path()` equals the plan's state path;
- the `runtime_epoch` singleton's `instance_id` and `epoch` equal the plan's;
- the just-written Workflow row has the planned ID, P/G/T, version 1 and the exact `record_body`;
- no contract row exists for the `workflow_id`.

It then runs `permits.with_exact_permit(vec![plan.insert.copy_for_transaction()?], || { INSERT …; ensure_consumed() })`. The permit windows are sequential, so this never nests the Driver or Source permits of `finish_input_tx` or `after_write`. The existing `ticket.validate_current_tx`, frame, Context, prerequisite and revocation checks are unchanged.

**Footprint.** One added row, the composed contract. The Task, Workflow, Context, verification-contract, Driver and audit writes are unchanged. There is no external effect, no await and no lock beyond the existing Store mutex.

### 5.5 Driver-lane Workflow writes after activation

For `DriverGate` and `DriverFirstExecutor`, `write_record_tx_at` (`state/mod.rs:1085–1086`) runs inside one exact `records` UPDATE permit window:

- **Old image:** the current row read in the same transaction after `validate_input_before_tx`. Its bytes must equal `serde_json::to_string(&input.record_before)`; otherwise the write is refused with a named error.
- **New image:** the plan's prescribed post body, which `finish_input_tx` already compares.

The window then calls `ensure_consumed()`. If no contract exists, the permit is not consumed and the write refuses, which fails closed. Other generic writers are not given a window, so a contracted Workflow refuses them, as managed binding §8 requires. The marker's `workflow_mutation` and the binder's permits are unchanged.

### 5.6 Marker predicate

`plan_marker_publication` and `validate_contract` move to one private `activated_contract(tx, workflow, allocation) -> Result<String>`, which reads the bounded row by Workflow/P/G/T, `owner_epoch = facts.epoch`, `contract_state = 'composed'` and `version = 1`. It then requires all of the following:

- the row's `origin` equals `allocation.selected_port().installation_id()`;
- the body decodes, its indexed columns equal the body, and `profile_digest` equals the roster digest of `members`;
- `member_digest(facts.role, selected_port)` is in `members`.

`validate_contract` keeps its exact-body EXISTS. Every other marker, confirmation, budget, Driver and Unit check is unchanged.

### 5.7 Commit, uncertainty, cancellation and stop

- **Known commit.** `publish_driver_preparation` runs `validate_result`, which for an initial input now also requires the exact contract row (all columns and the body).
- **Error or unwind.** The `Applying` guard marks the SAME plan reconciliation-ready. `reconcile_driver_preparation` (`runtime/service.rs:29`) either publishes the post image (with the contract) or proves rollback, and `validate_rollback` now also requires that no contract exists for the planned Workflow. If neither holds, the plan stays held. Nothing builds a new plan, retries the INSERT or reads a row as proof.
- **Cancellation.** The `tokio::select!` in `drive` can cancel only at awaits. Before the Store section nothing is written, and the Store section has no await.
- **Revocation, shutdown or Runtime Drop before commit.** The existing association check in `finish_input_tx` aborts the transaction, leaving no Workflow and no contract.
- **Process death after commit.** A new epoch never reconstructs the plan. `fresh` refuses the Task (G5 §4.2). The contract stays immutable and keeps protecting the Workflow.

### 5.8 Same-Workflow multi-alias and restart

- **Within one epoch.** Every Executor and Reviewer allocation is admitted by membership, whatever its Unit, profile or port. PR-2 may choose any reviewer listed in `task.reviewers` without a new contract. An alias outside the roster refuses, and a later change to a Task's reviewers leaves its Workflow unable to admit the new alias, which is Held rather than updated.
- **Restart (unmet; explicit later requirement).** The contract `owner_epoch` and `origin` are activation facts, and the trigger pins the operation epoch. A Task owns exactly one Workflow (`state/mod.rs:772–774`). A cross-epoch continuation therefore needs the genuine restart producer to add an append-only, permit-guarded successor relation keyed by (workflow_id, epoch), carrying the new installation and a re-derived roster from that restart's SAME composition. It must extend `binding_operation_allocation` and `activated_contract` together through a schema migration with a permit-catalogue entry. It must never UPDATE this row, and `legacy_held` stays nongrant. Until then the restart gate is not met and is not redefined.

## 6. Implementation sequence (Sol high)

Work in a new worktree branched from `c51` (whose parent is `a21`). Never use or modify c51's active worktree. Commit each item, and keep the worktree clean before every control run.

1. §5.1 installation identity and the declaration-digest accessor.
2. §5.2 `member_digest`, `ActivationRoster`, `activation_roster`, `installed_native_port` and the issuer prefilter.
3. §5.3 `NativeActivationPlan`, the `plan_initial_input` parameter and the `initialize_driven` signature and call site.
4. §5.4 the contract write and §5.7 the `validate_result`/`validate_rollback` extensions.
5. §5.5 the Driver-lane permit windows.
6. §5.6 the marker predicate and the body `members` field.
7. Fixture migration (§7) and the §8 controls and mutants.
8. Master updates in the same implementing PR, after the source exists: `master/workflow-engine.md` (the Issue43 status block and §17: activation contract semantics and protected Workflow writers) and `master/agent-adapter.md` §15 (installation identity and roster member).

## 7. Impact analysis

| Changed / consumed | Consumers (V) | Handling |
|---|---|---|
| `NativePhasePort::installed` signature and the `installation` field | `adapter.rs:420` only | additive identity; tests that call `from_managed_config` are unchanged |
| `declaration_digest` visibility | `execution/native/registration.rs:50`, the new `member_digest` | read only |
| `initialize_driven` signature | `runtime/task_driver.rs` `drive` (c51) only | passes the claim's composition |
| `plan_initial_input` signature | `workflow.rs:1039–1041` only | the plan is retained in the same Arc |
| `put_workflow_transition_inner` | activation, gate, first-Executor, verification, readonly, result and legacy callers (`state/mod.rs:592–662`, `workflow.rs:674–806`) | the contract write runs only for `Activation(_, Some)`; the windows only for `DriverGate`/`DriverFirstExecutor`; other callers are unchanged and, as in §8, refused on contracted Workflows |
| Contract existence enables the record guards (`schema.rs:316–336`) | every Workflow writer of a driven Task | the Runtime lane has only Driver-planned writers, the marker and the binder (§2 grep); PR-1–PR-4 consumers must use exact permits |
| `validate_result` / `validate_rollback` | `publish_driver_preparation`, `reconcile_driver_preparation` (`preparation.rs:601–660`) | adds the exact contract presence or absence check; other plans are unchanged |
| Marker predicate | `plan_marker_publication`, `validate_contract` (`publication.rs:235–298,426,477`) | §5.6; no test asserts the old messages (grep) |
| Contract body and budget | `charged_scope_bytes` (`publication.rs:304–332`) | ≤4096 B, already charged |
| `managed_phase_required` (`protection.rs:66–79`) | legacy Native6 | already true once the Workflow exists; unchanged |
| Issuer prefilter | G5 sweep, C2–C15 (c51 `runtime/installation/tests.rs`) | claiming fixtures declare a reviewer alias (the same fixture program under its own declaration); the undeclared and smoke fixtures still refuse earlier |
| New constants | ≤8 reviewers, ≤9 members, two digest domains | local; not shared |

**Not affected:** the schema, `SCHEMA_VERSION`, triggers, the permit catalogue, `legacy_held` migration and rows, the verification contract, the record-only binder, the Source handoff, PhaseJobs, transport, quota, attention, user settings, credentials and hooks.

## 8. Controls (P; none executed)

Every control uses the compiled `Runtime::new` + `start` installed graph, Goals accepted through the real HumanIngress, the service sweep and `native_fixture.py` configured as `AgentConfig.command` with a declaration. A fixture is external protocol wiring and never Native qualification. A raw-SQL or forged row is allowed only as a negative stimulus and earns no positive credit. A compile failure or a SETUP stop is never a positive result or a kill. A mutant is killed only by its named assertion, after the control's genuine normal setup has succeeded.

| ID | Control |
|---|---|
| CA1 | For each provider, with executor `worker` and reviewers `rev-a` and `rev-b` (both declared): the contract row is byte-equal to the plan; `origin` equals the registry installation; `members` equals an independent test-side recomputation from each port's accessors using the §4.2 encoding; the epoch equals the runtime epoch. Then the gate edges commit under their windows, followed by the marker, `Launched`, the record-only binder `Bound`, `Task.version == marker_task_version` and a contract that is byte-identical afterwards |
| CA2 | Four Tasks in two Projects (two Claude, two Codex executors): one contract each, with its own scope and roster and no cross-Workflow reuse |
| CA3 | Paired with CA1 (identical configuration except the named difference): no reviewers, a reviewer alias absent, a reviewer without a declaration, or nine reviewers → `Skipped(Composition)`, with zero Driver/Unit/Source/Workflow/contract rows while a sibling is claimed |
| CA4 | Genuine setup paused through the existing test-only Engine hook (timing only), then: Driver revoked or shutdown before commit → no Workflow and no contract; a Context or frame drift → existing refusal, no contract; a pre-commit fault → `validate_rollback` holds; a post-commit unwind → the service publishes the SAME plan, with exactly one contract row and no second INSERT |
| CA5 | With a genuine contract present, the planner is called directly on genuine retained objects: (a) the SAME Unit allocated through an installed, declared alias outside the roster → membership refusal and zero operation rows, paired with the roster port succeeding; (b) Task A's allocation against Task B's Workflow → scope refusal; (c) a migrated `legacy_held` Workflow → state refusal with the legacy row byte-identical; (d) a negative-only forged row whose members do not hash to `profile_digest` → integrity refusal. If no seam can borrow genuine retained objects, the result is SETUP |
| CA6 | A public-constructor generic Workflow write on a contracted Workflow is refused with "private managed Record writer required" and the Workflow is unchanged; UPDATE, DELETE and REPLACE of a genuinely produced contract are refused |
| CA-R | A Reviewer-role marker admitted by membership: **unmet until PR-2** produces a genuine Reviewer Unit; SETUP until then |
| CA-E | A cross-epoch continuation: **unmet until the restart producer** (§5.8) |

**Mutants (compiled):**

| Mutant | Killed by |
|---|---|
| producer call removed | CA1 marker stage |
| reviewers dropped from the roster | CA1 members assertion |
| port origin, program or declaration omitted from the member digest | CA1 recomputation |
| membership check removed | CA5a |
| scope conjunct removed | CA5b |
| `composed` filter removed | CA5c |
| digest check removed | CA5d |
| prefilter reviewer conjunct removed | CA3 (the refusal moves after the Source rows) |
| CA4 epoch or instance check removed | CA4 epoch variant (setup paused by the hook) |
| `validate_rollback` contract-absence check removed | not credited: equivalent under atomicity |
| gate window removed | the CA1 gate-stage assertion after a genuine activation |

## 9. Current status and limits

- Nothing in this document is implemented, built, tested, reviewed or qualified.
- This lane makes a first-Executor marker reachable only on the integration branch containing c51. It does not make a full Workflow, Reviewer or later phase, restart continuation, the four-Task both-OS run, the official real-CLI N1/N4/H/Q qualification, install, CI, regression or Clippy GREEN, or merge readiness. Each remains at its owning contract.
- The next step is one independent Sol high review of this document. Implementation starts only after zero confirmed unmet items.
