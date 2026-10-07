# Issue 43: Composed Workflow native-contract activation producer (CA)

## 1. Status, pins and categories

1. This is a proposed requirements addendum and authority/contract HOW for the one missing prerequisite lane between Workflow activation and the original marker. It connects, and does not replace, the [managed binding](issue-43-managed-binding-design.md) (§3 `workflow_native_contracts`, §8), the [Runtime Driver producer](issue-43-runtime-driver-producer-design.md) (§§2, 5 and the finite preparation correction), the [prepared producer](issue-43-native-prepared-producer-design.md), the [marker dispatch](issue-43-original-source-marker-dispatch-design.md) and the [installed composition (G5)](issue-43-native-installed-composition-design.md). It does not update the master design and claims nothing implemented.
2. **Pins.** The first draft read source facts at the frozen clean base `a21cdabfeb63f826a9421741e76dc80a826cc101`. G5 component facts are read with `git show` at `c51af44733f0f6e42ec09bf0914a6fd92055192c`, whose merge base with `a21` is `a21`. The CA-H1..CA-H5 correction reads the CURRENT integrated source `c9791175d153b4801c2fd42cc0e557b84a8e477e` (G5 integrated), using `git show` only. This design worktree's own tree stays at `a21`. Paths are relative to `crates/rrx/src/`. Line numbers are at `a21` unless marked `c51` or `c979`, and implementers locate symbols by name. The independent review reports that the cited activation consumers are unchanged between `c51` and `c979` (A).
3. **Categories.**
   - **V (verified):** read in source at the stated pin. No build, test, lint, Agent or OS run was executed for this document.
   - **P (proposed):** this contract; not implemented.
   - **A (acknowledged):** reported by Root or by the G5 owner and relied on here without re-verification.
4. **Never authority here:** a successful SELECT or decode of any contract row, configured aliases or capability metadata, a public `AgentRegistry`/`WorkflowEngine` constructor, test-only constructors, SQL inserted to make a positive fixture pass, a boolean or optional switch, copied DB state, Native permission bypass, a Task.version-changing binder, or a later UPDATE of immutable contract history. Activation is necessary for a marker and is never sufficient for Native dispatch. rrx is not a sandbox. Official CLIs run on the host with the user's usual HOME, config, hooks and auth. Nothing here reads, copies or transfers them, and no VM, container, separate user, root, new HOME or outer sandbox is used. Licensing and the #46/#60 process observation are untouched.
5. **Current integrated status (A; not qualification).** At `c979`, G5 is integrated locally and all of its owned source handles are closed. Its genuine marker planning still stops because composed activation is missing (§2). The reported Runtime19 regression and local install smoke PASS qualify neither Native dispatch nor a full Workflow. Strict Clippy is reported RED, and no CI has run on the current source.

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
| G5 stop (A) | At `c51`, `drive` calls `initialize_driven(task,&sources,&lifetime)` (`runtime/task_driver.rs`, c51), and the genuine C5/C6 setup reaches Source preparation and marker planning. Planning refuses with "actual composed Workflow activation unavailable", so no marker or Native invocation runs and no positive or mutation credit exists. At `c979` the call is unchanged (V), and the same stop is reported (A) |
| Installed declaration (c979) | `NativeAdapter.compatibility: Option<Arc<NativeCompatDeclaration>>` is `pub(crate)` (`adapter/native.rs:12–19`). The declaration's `digest` is a private field that `NativeCompatDeclaration::installed` computes once (`execution/native/compat.rs:5–17`). `NativeCompatQualification::declaration_digest` (`:150–153`, `pub(super)`) belongs to a qualification that retains a captured actor and version-helper custody (`:18–27,125–147`) |
| Module boundary (c979) | `runtime` and `state` are sibling modules (`lib.rs:12–13`). `InstalledDriverComposition` lives in `runtime::installation` and is re-exported to `state` (`state/managed_binding/mod.rs:31`) |
| Driver-lane Record images (c979) | `plan_initial_gate` and `plan_first_executor` first change the Workflow data and only then capture `record_before` (`state/runtime/driver/gates.rs:252–265,300–307`; `executor.rs:125–135,176–183`). That value is the prescribed writer input at the old version and time, not the stored row. The original persisted Workflow is the SAME ticket's `ScopePlan.workflow: Body<Record>`, which holds the exact raw bytes (`managed_binding/snapshot.rs:18–27,343–348`); `validate_current_tx` compares those bytes exactly. The write is `write_record_tx_at`, which runs `UPDATE records SET version,body WHERE id AND version` (`state/mod.rs:1858–1878,2185–2208`). `record_image` (`marker_rows.rs`) builds the complete seven-column image |
| Stop boundary (c979) | `shutdown` sets `stopping` and calls `stop_all` under `control_admission` (`runtime/service.rs:113–120`). `Drop for Runtime` does both without admission (`runtime/mod.rs:124–128`), and `stop_all` takes no Store lock (`runtime/driver.rs:455–461`). `finish_input_tx` checks the association (`preparation.rs:393–403`) before `tx.commit()` (`state/mod.rs:1101–1103`), and initialization's Store section holds no admission (`workflow.rs:1035–1057`). Driver startup (the spawn closure in `runtime/task_driver.rs`) and marker publication (`runtime/phase_supervisor.rs:1007–1083`) already take `control_admission` before Store. The other revocations are the worker's own `WorkerLifetime` drop, a `PendingRegistration` drop and an observed finished job (`runtime/driver.rs:86–113,485–496`) |
| Off-Store contract decode (c979) | `plan_marker_publication` decodes the contract inside the `snapshot` connection. That connection is its own query-only SQLite connection and takes no SharedStore (`publication.rs:253–270`; `snapshot.rs:29–56`). Under Store, `validate_contract` compares only exact columns and the retained body (`publication.rs:289–297`, used at `:426,477`) |
| Test hooks (c979) | `EngineHooks` (`workflow.rs:545–552`) holds `before_publication`, `before_release`, `before_wait_claim`, `before_reserve` and `attempt_started_at`. Initialization invokes none of them. `before_reserve` runs only during ordinary reservation (`:1327–1337`), and `before_publication` only during later result publication (`:3078–3087`) |
| Composition sharing (c979) | Every `Runtime::installed_driver_composition` call builds a new `InstalledDriverComposition`, but its `runtime`, `owner`, `phases`, `sources` and `engine` are the SAME shared objects for every Task of that Runtime. Only `original_task` and `selected` differ (`runtime/installation.rs:15–23,148–155`). `plan.is_retained()` proves only that plan's own slot custody (`runtime/driver.rs:247–256`), not its relation to a composition |
| Postcommit publication (c979) | After `tx.commit()`, `put_workflow_transition_inner` propagates `publish_driver_preparation` with `?` (`state/mod.rs:1101–1105`), and `drive` propagates the initialization error (`runtime/task_driver.rs:114–118`). Worker exit revokes the slot (`WorkerLifetime::drop`, `runtime/driver.rs:86–100`; `observe_finished`, `:474–500`), and `publish_exact` requires an active, entered, unrevoked worker (`:271–284`). `reconcile_driver_preparation` returns `false` while a plan is neither publishable nor provably rolled back (`preparation.rs:601–660`). `observe_task_drivers` discards that value and returns only unfinished jobs (`runtime/service.rs:20–41`), which is all that `shutdown` waits on (`:133–140`). `acknowledge_exit` keeps an activated slot together with its retained preparation (`runtime/driver.rs:592–602`) |
| SharedStore poison (c979) | SharedStore is `Arc<std::sync::Mutex<Store>>` (`adapter.rs:17–19,44`). A panic that unwinds while a caller holds its guard poisons it, and every later service acquisition refuses with "state poisoned" (`runtime/service.rs:25–29,69–74`). Nothing in `runtime/` or `state/runtime/` catches unwinding. The crate's existing containment pattern is `catch_unwind(AssertUnwindSafe(..))` (`adapter/git_owner.rs:1280`, `codex/custody.rs:479`). No workspace manifest sets `panic = "abort"` (grep) |

**Conflict.** An immutable per-Workflow row cannot equal a per-port origin and a per-Unit profile for more than one phase. If it is pinned to the first Executor, every Reviewer (another port UUID) and every later Unit (another profile) is refused, and the contract cannot be updated. A first-executor pin is therefore not a prerequisite for the requested full Workflow, and this design does not use one.

## 3. WHAT addendum (P)

- **R1 Producer.** The only composed-contract writer is the genuine Driver activation transaction. Its inputs are the SAME `InstalledDriverComposition` held by the claim (identified by its issuer-owned original composition identity, §5.2), the retained `DriverPreparationAdvance` initial-input plan, the actual Workflow Record created in that transaction and the selected owner's database and epoch. It commits only while the SAME Runtime's control admission is held (§5.7). If a stop wins admission, no activation occurs. A committed activation whose cache publication faults recoverably is published only by exact reconciliation of its SAME retained plan while the SAME worker is still live (§5.7). After an actual worker exit or revocation, it stays Held under that plan, and shutdown reports it pending.
- **R2 Full-Workflow coverage.** The contract covers the executor alias in the Executor role and every distinct `task.reviewers` alias in the Reviewer role, each of which must be an installed, declared Native port of that composition. Per-phase identity (Unit, profile, Session, port origin, allocation) is checked at each marker, never pinned in the contract.
- **R3 Unchanged transitions.** The activation keeps its existing single Task version bump and Workflow version 0→1. Record-only Session binding still never changes Task.version. No other row is added to the activation footprint. Every later initial gate and first-Executor Workflow write keeps its prescribed consecutive version and planned timestamp. It also consumes one exact `records` permit, whose old image is the original persisted row (§5.5).
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

- `member = digest(b"rrx.native-roster-member/v1\0", canonical{role, alias, provider, program, declaration_digest, port_origin})`. Every value is read from the SAME `Arc<NativePhasePort>`, its upgraded adapter and that adapter's installed declaration (§5.1).
- `profile_digest = digest(b"rrx.native-roster/v1\0", canonical(members))`.
- The canonical body and both digests are computed outside SharedStore. The issuer computes the members and the roster digest (§5.2), and `plan_marker_publication` computes the marker's member (§5.6).

## 5. HOW (P)

### 5.1 Installation identity and the installed declaration digest

`AgentRegistry::from_managed_config` mints one `installation: Uuid` and passes it to `NativePhasePort::installed(adapter, installation)` (still `pub(super)`). The port exposes `pub(crate) fn installation_id()`. The SAME installed `NativeCompatDeclaration` (`execution/native/compat.rs:5–17`, c979) gains one read-only `pub(crate) fn digest(&self) -> &str` over its existing private digest, which `installed` computed once at registry installation. The member digest reads it only as `port.selected_adapter()?.compatibility.as_ref()` → `digest()`. `NativeAdapter.compatibility` is already `pub(crate)` (`adapter/native.rs:12–19`, c979). Activation never calls `qualify_compat_static`, `NativeCompatStatic::observe` or a version helper, and never constructs `NativeCompatQualification`. It leaves `NativeCompatQualification::declaration_digest` (`compat.rs:150–153`) `pub(super)` and unchanged. These are identities, not grants. Public `from_config` creates no ports.

### 5.2 Roster issuer and prefilter

```rust
// runtime/installation.rs (c979): the issuer's own module, a sibling of `state`
struct CompositionIdentity { task: TaskId } // module-private, non-Clone, no Deserialize/Default; its Arc pointer is the identity
// InstalledDriverComposition gains one private field: `identity: Arc<CompositionIdentity>`.
pub(crate) struct ActivationRoster { composition: Arc<CompositionIdentity>, task: TaskId, executor: String,
    reviewers: Vec<String>, installation: Uuid, members: Vec<String>, digest: String,
    runtime: Weak<Runtime>, phases: Weak<PhaseSupervisor>, owner: Arc<RuntimeOwner> } // non-Clone, no Deserialize/Default
impl ActivationRoster {
    fn new(composition: &InstalledDriverComposition, installation: Uuid, members: Vec<String>) -> Result<Self>; // module-private
    pub(crate) fn task(&self) -> TaskId;
    pub(crate) fn executor(&self) -> &str;
    pub(crate) fn reviewers(&self) -> &[String];
    pub(crate) fn installation(&self) -> Uuid;
    pub(crate) fn members(&self) -> &[String];
    pub(crate) fn digest(&self) -> &str;
    pub(crate) fn owner(&self) -> &Arc<RuntimeOwner>;
    pub(crate) fn is_same_composition(&self, composition: &InstalledDriverComposition) -> bool; // identity pointer first; nongrant
}
impl InstalledDriverComposition {
    pub(crate) fn activation_roster(&self, task: &Task) -> Result<ActivationRoster>; // the ONLY caller of ActivationRoster::new
}
// state/managed_binding/activation.rs (new private module): pure, nongrant, never under SharedStore
pub(crate) fn member_digest(role: SessionRole, port: &NativePhasePort) -> Result<String>; // upgrades the SAME adapter; reads its installed declaration
pub(crate) fn roster_digest(members: &[String]) -> Result<String>;
// workflow.rs: nongrant accessor over the Engine's own registry
impl WorkflowEngine { pub(crate) fn installed_native_port(&self, alias: &str) -> Result<Arc<NativePhasePort>>; }
```

Rust restricted visibility can name only an ancestor module, and `runtime` and `state` are siblings (`lib.rs:12–13`, c979). Construction is therefore sealed by module ownership:

- `ActivationRoster` lives in `runtime::installation` beside its sole issuer.
- Its fields are private and `new` is module-private, so only `activation_roster` builds one.
- `state` names the type through a `pub(crate) use` re-export, as it already does for `InstalledDriverComposition` (`state/managed_binding/mod.rs:31`, c979), and reads it only through the accessors.
- No public, `#[cfg(test)]` or `Default` constructor exists. Installation tests obtain a roster only from `activation_roster` on a genuine composition. A descendant test module that builds a roster directly earns no credit.

**Original composition identity.** One Runtime issues many Task compositions that share the SAME Runtime, owner, phases, Sources and Engine (§2). Shared-pointer equality is therefore necessary but never sufficient.

- `Runtime::installed_driver_composition` is the only constructor of `InstalledDriverComposition` (`runtime/installation.rs:148–155`, c979). Each call mints one fresh `Arc<CompositionIdentity>` that holds the original Task ID and no strong Runtime reference.
- `ActivationRoster::new` copies that `Arc`, the original Task ID, the executor alias and the reviewer aliases from `original_task()`, never from the caller's Task.
- `is_same_composition` holds only if all of these hold: `Arc::ptr_eq(&roster.composition, &composition.identity)`; `roster.task == composition.original_task().id`; and the `Weak<Runtime>`, `Weak<PhaseSupervisor>` and owner pointers are equal.
- `CompositionIdentity` has no public, `#[cfg(test)]`, static, `Default` or deserializing constructor. It is not re-exported to `state`, and it has no serialized or row form. No metadata, alias, SQL row, Task ID value or test fixture reconstructs it or grants positive authority in its place.
- §5.3 binds the identity to the activation input, the retained ticket and the worker association.

`activation_roster` refuses unless all of these hold:

- `composition.is_current()`, `task.id == original_task().id`, and `task.executor` and `task.reviewers` equal the original Task's;
- `original_task().executor == selected().alias()`, with the executor port pointer-equal to `selected()`;
- `original_task().reviewers` has 1–8 distinct aliases;
- every member port comes from `engine().installed_native_port(alias)`, `selected_adapter()` upgrades, the port's owner is the composition owner, the provider is `claude|codex`, the declaration is `Some`, and all ports share one `installation_id`;
- each member is `member_digest(role, port)`, and the roster digest is `roster_digest(members)`.

The issuer performs no IO and takes neither a Store lock nor control admission. It constructs no compatibility qualification and runs no helper. `Runtime::installed_driver_composition` (c51) runs the same reviewer conjuncts as a nongrant prefilter, and their refusal is `SkipReason::Composition` with no write. Activation re-derives the roster authoritatively.

### 5.3 Retained activation plan and activation admission

```rust
// state/managed_binding/activation.rs
pub(crate) struct NativeActivationPlan { roster: ActivationRoster, workflow: RecordId, scope: Scope, epoch: u64,
    instance: String, state_path: PathBuf, body: String, row: Vec<SqlValue>, insert: ExactRowMutation,
    #[cfg(test)] seams: ActivationSeams } // private fields, non-Clone
pub(crate) fn plan_native_activation(roster: ActivationRoster, record: &Record, task: &Task) -> Result<NativeActivationPlan>;
pub(crate) enum ActivationCommit { Published, Deferred } // outcome only, never a grant (§5.7)
// runtime/installation.rs
pub(crate) struct ActivationAdmission { runtime: Arc<Runtime>, admission: tokio::sync::OwnedMutexGuard<()> } // private, non-Clone
impl InstalledDriverComposition {
    pub(crate) async fn admit_activation(&self, plan: &Arc<DriverPreparationAdvance>,
        lifetime: &WorkerLifetime) -> Result<ActivationAdmission>;
    pub(crate) async fn admit_activation_recovery(&self, plan: &Arc<DriverPreparationAdvance>,
        lifetime: &WorkerLifetime) -> Result<ActivationAdmission>; // §5.7 SAME-worker recovery only
}
```

The plan consumes the roster. It therefore retains the original composition identity, Task and owner linkage (the SAME identity `Arc`, `Weak<Runtime>`, `Weak<PhaseSupervisor>` and `Arc<RuntimeOwner>`) without holding a strong Runtime reference.

- **Requirements.** These must hold:
  - `record.kind == Workflow`, `record.version == 0` and `record.scope == task.scope()`;
  - `task.id == roster.task()`, `task.executor == roster.executor()` and `task.reviewers == roster.reviewers()` (same order and length);
  - `epoch = roster.owner().epoch() > 0`.

  The instance and path come from that owner. A roster issued for another composition's Task therefore refuses here, outside SharedStore and before any write.
- **Off-Store construction.** The plan builds the canonical body, the ten retained column values and the INSERT mutation once, outside SharedStore. The body must be ≤4096 B and must round-trip through `Body::decode`.
- **Retention.** `DriverReadTicket::plan_initial_input` (`state/runtime/driver/preparation.rs:41`) gains a parameter `activation: NativeActivationPlan`, which is stored in `InitialInput`. The plan must match the input's scope and record ID, and `roster.task()` must equal the SAME ticket's Task ID. It is therefore retained in the SAME `Arc<DriverPreparationAdvance>` that the Driver slot retains before SQL (finite preparation correction).
- **Access.** For admission and recovery, `DriverPreparationAdvance` exposes only two read-only accessors: `activation_roster()` and `planned_binding()`. The second returns the exact Driver postimage `(id, epoch, version, body)` that the plan already retains. No other constructor exists.

In `initialize_inner` (`workflow.rs:1035–1057`), `initialize_driven(task, composition, lifetime)` replaces `&sources`, and the Sources come from `composition.sources()`. `drive` passes `claim.composition()` (c979). The order is:

1. `verification_activation`;
2. `initial_input_frame(...).await`;
3. `activation_roster`, then `plan_native_activation`;
4. `plan_initial_input`, which retains the plan;
5. `composition.admit_activation(&plan, lifetime).await`;
6. a synchronous `store.lock()` and `activate_driven_workflow`, which returns `ActivationCommit` (§5.7);
7. drop the Store guard, then the admission;
8. only on `Deferred`, the SAME worker's recovery (§5.7), before `initialize_driven` returns.

All Source, helper, file, encoding and hashing work happens before the first admission. The only later awaits are step 8's admission and bounded-wait awaits, and none of them holds a Store guard.

`admit_activation` runs these steps:

1. It upgrades the composition's original `Weak<Runtime>` and refuses if the Runtime has ended.
2. It awaits `control_admission.clone().lock_owned()` inside a biased `select!` with `lifetime.cancelled()`.
3. Without another await, it refuses unless all of these hold:
   - `service_running()` and `composition.is_current()`;
   - `association = lifetime.association()?` (live and unrevoked);
   - `plan.activation_roster()?.is_same_composition(self)`, `self.original_task().id == association.task()` and `plan.belongs_to(&association)`;
   - `plan.is_retained()`.

   The identity, the input Task, the retained ticket and the live worker association are therefore one linkage. A sibling composition's roster or plan refuses before any Store write.

`admit_activation_recovery` runs the same steps 1–3, except that recovery reads `plan.is_retained()` under Store instead (§5.7).

The returned guard grants nothing in SQL. It only holds the actual stop boundary, and its strong Runtime reference delays `Runtime::drop` for no longer than the bounded synchronous segment (§5.7).

### 5.4 Activation transaction: placement, checks and footprint

In `put_workflow_transition_inner`, the plan writes the contract only for `Activation(_, Some(plan))` whose input carries a `NativeActivationPlan`. The write comes after `install_contract` (`:1091–1093`) and before `source_recovery::after_write` (`:1094`). The caller holds `ActivationAdmission` for the whole call.

```rust
impl DriverPreparationAdvance {
    pub(in crate::state) fn write_native_activation_tx(&self, tx: &Transaction<'_>,
        permits: &PrivatePermitManager) -> Result<()>;
}
```

Inside the existing Immediate transaction, it uses only retained values and exact SQL comparisons. It does not encode, decode or hash contract data. It refuses unless all of these hold:

- `tx.path()` equals the plan's state path, and the `runtime_epoch` singleton's `instance_id` and `epoch` equal the plan's. These are redundant defenses after the ticket's and Driver's epoch currency, and §8 gives them no credit.
- The Workflow row written in this transaction exactly equals the plan's seven retained columns: ID, `workflow`, P/G/T, version 1 and `record_body`.
- No contract row exists for the `workflow_id`.

It then runs `permits.with_exact_permit(vec![plan.insert.copy_for_transaction()?], || { INSERT the ten retained values; ensure_consumed() })`. The permit windows run one after another, so this never nests the Driver or Source permits of `finish_input_tx` or `after_write`. `finish_input_tx` then runs `validate_result`, which for an activation input now requires the exact contract row (all ten retained columns). This happens before the existing association check and Driver CAS. The existing `validate_input_before_tx` and `ticket.validate_current_tx`, frame, Context, prerequisite and revocation checks are unchanged.

**Footprint.** One added row, the composed contract. The Task, Workflow, Context, verification-contract, Driver and audit writes are unchanged. There is no external effect and no await after admission. Locks are taken in this order: `control_admission`, the existing Store mutex, then the permit manager (§5.7).

### 5.5 Driver-lane Workflow writes after activation

At `c979`, every real initial Driver-lane Workflow writer reaches `put_workflow_transition_inner` through `DriverGate` or `DriverFirstExecutor`:

- `apply_driven_initial_gate` for the Claim edge (`workflow/driven_initial.rs:108–112`);
- `apply_driven_initial_gate` for the Reserve and Complete edges (`:153–163`, `:371–382`);
- `reserve_driven_first_executor` (`:279–283`).

For each of them, `write_record_tx_at` (`state/mod.rs:1085–1086`) runs inside one exact `records` UPDATE permit window. `plan_initial_gate` and `plan_first_executor` plan the window's images outside Store and retain them in `InitialInput`.

- **Old image: the original persisted row.** The image is `record_image(original.parsed(), original.raw())`, taken from the SAME ticket's `ScopePlan.workflow` body. These are the exact bytes that `validate_input_before_tx` → `validate_current_tx` already compares in the same transaction. The old image is distinct from `input.record_before`. That value is the prescribed writer input, carrying the changed data at the old version and time, and it is never compared with stored bytes.
- **New image: the prescribed postimage.** The image is `record_image(input.record, input.record_body)`. The plan constructor requires the same ID, kind and P/G/T as the original, `version == original.version + 1` and `updated_at == input_timestamp()`. It also requires `record_body` to be exactly what `write_record_tx_at(record_before, input_timestamp())` writes, which `finish_input_tx` already compares.
- **Window.** `permits.with_exact_permit(vec![records.copy_for_transaction()?], || { guard_record_tx; write_record_tx_at(..); ensure_consumed() })`. The window closes before the Source and Driver windows open, so nothing nests. The initial activation's Workflow INSERT needs no window, because no contract exists until §5.4 inserts it later in the same transaction.
- **Drift.** If the stored Workflow differs from the original bytes, `validate_current_tx` refuses before any write. The trigger independently refuses any UPDATE whose old row differs from the permit. Either way the transaction rolls back with no committed change. Nothing re-reads the row to build a replacement image.
- **Fail closed.** If no contract exists, the permit is not consumed and `ensure_consumed` refuses.

No other writer gets a window, so a contracted Workflow refuses them all, as managed binding §8 requires. This includes the generic `observe_workflow_gate` (`state/mod.rs:1114`, called from `workflow.rs:2217`, c979). The driven Complete edge instead carries its observation inside the plan. The marker's `workflow_mutation` and the binder's permits are unchanged.

### 5.6 Marker predicate: decision outside Store, exact comparison under Store

- **Decision outside Store.** `plan_marker_publication` computes `member = member_digest(facts.role, allocation.selected_port())` and `installation = selected_port.installation_id()` in memory. In its existing read-only `snapshot` connection (`publication.rs:253–270`), it reads only the bounded raw row selected by Workflow/P/G/T, `owner_epoch = facts.epoch`, `origin = installation`, `contract_state = 'composed'` and `version = 1`. After the snapshot returns, still outside SharedStore, it strictly decodes `Body::<NativeContract>`. It then requires the indexed columns to equal the body, `roster_digest(members) == profile_digest` and `member ∈ members`. `MarkerPublicationPlan` retains the exact raw body, `origin` and `profile_digest`.
- **Under Store.** `validate_contract` (marker `:426`, confirmation `:477`) stays one exact EXISTS. It compares Workflow/P/G/T, `owner_epoch = facts.epoch`, the retained `origin` and `profile_digest`, `composed`, version 1 and `body =` the retained raw body. It performs no decode, canonical encoding or hash. Membership authority comes from the decision made outside Store, which is bound to these exact immutable bytes. The marker's existing scope, Driver, budget and Unit currency checks run in the same transaction.

The per-Unit profile and per-port checks are unchanged (§4.1). Every other marker, confirmation, budget, Driver and Unit check is unchanged.

### 5.7 Admission, commit, publication, uncertainty, cancellation and stop

- **Lock order and lifetime.** `control_admission` comes first and is taken by an await. SharedStore follows synchronously, and the permit manager is taken inside the transaction. No code may await `control_admission` while holding SharedStore; §7 lists the consumers to check. After admission there is no await, Source or helper work, file access, or contract encoding or hashing. Guards drop in this order: the Store guard, then the admission, then the strong Runtime reference.
- **Linearization.** While activation holds admission and a strong Runtime reference, a stop cannot intervene:
  - `shutdown` cannot set `stopping` or call `stop_all`, because it must take admission first (`service.rs:113–120`);
  - `Runtime::drop` cannot run;
  - the remaining revocations come from this worker's own lifetime or job, which cannot run during its synchronous segment.

  The association check in `finish_input_tx`, the commit, and either the cache publication or the contained `Deferred` outcome therefore all happen inside one admitted segment, which closes the check-then-commit window. A deferred publication completes only through exact reconciliation of the SAME plan while the SAME worker is still live. A stop that wins admission revokes every slot first. The waiting activation then observes cancellation (the `select!` is biased) or refuses at its post-admission checks, and no activation occurs.
- **Service liveness.** `running` can become false without admission only when the service loop ends (`Running`, `service.rs:12–17`). Admission checks it once. A later change neither retracts nor revalidates a commit. The commit is published in the same segment, deferred to the SAME live worker's recovery, or Held, as described below. No liveness flag is a grant.
- **Known commit.** Inside the admitted segment, after `tx.commit()` returns `Ok`, the activation path runs its cache publication inside the contained postcommit boundary below. On success, `publish_driver_preparation` runs `validate_result`, which for an activation input also requires the exact contract row (the ten retained columns). It then publishes the Driver cache from the retained postimage and retires the plan, and the call returns `Published`.
- **Contained postcommit boundary (activation only).** It applies only to `Activation(_, Some(plan))` in `put_workflow_transition_inner`, and only after `tx.commit()` returns `Ok`.
  - **Scope.** The S3 seam (test builds only) and `self.publish_driver_preparation(plan)` run inside one `std::panic::catch_unwind(AssertUnwindSafe(..))`, the crate's existing pattern (§2). The boundary lies inside the `&mut Store` method. An unwind that it catches therefore never reaches the caller's SharedStore guard, which is later dropped normally rather than while panicking, so the Mutex is not poisoned.
  - **Legal release.** After an `Err` or a caught panic, the boundary requires `self.connection.is_autocommit()`, which means that the publication's own read transaction was rolled back by its drop. If this holds, it discards the panic payload without inspecting it, sets the out-parameters to the committed Task and Workflow, and returns `ActivationCommit::Deferred`. The caller then drops the Store guard and the admission normally. If the check fails, a caught panic is resumed with `resume_unwind` and an `Err` is returned unchanged, and the uncontained path below applies.
  - **No poison clearing.** The boundary never calls `into_inner` or `clear_poison` on any lock, never recovers a poisoned guard and never weakens a check. A poisoned slot `binding` or `preparation` Mutex keeps refusing through its existing error mapping, so such a plan stays Held.
  - **Out of scope.** Precommit work, gate and first-Executor publication, and every other caller keep their existing propagation. No recovery is claimed for them.
- **Recovery by the SAME live worker.** `Deferred` is not an error, so `initialize_inner` does not propagate it to `drive`. The worker keeps its `WorkerLifetime`, and its slot stays active, entered and unrevoked.
  - **Segments.** The worker runs at most three recovery segments, separated by a 100 ms sleep inside a biased `select!` with `lifetime.cancelled()`. Each segment awaits `admit_activation_recovery`, then synchronously locks Store and calls `reconcile_driver_preparation(&plan)` on the SAME `Arc`. Its publication path is the unchanged `publish_driver_preparation` → `publish_exact`, whose liveness conjuncts hold because the worker is still live.
  - **Concurrent sweep.** The service sweep may reconcile the same plan concurrently. Store exclusion serializes the two, and `publish_exact` is idempotent only for the exact planned postimage.
  - **Success.** Recovery succeeds only when `!plan.is_retained()?` and `association.binding()? == plan.planned_binding()`. Then `initialize_driven` returns, and `drive` continues to the gate loop.
  - **No fabrication.** Nothing re-reads rows to build a publication, rebuilds a roster or plan, retries the INSERT, or derives a grant or scope from rows.
- **Actual exit or revocation stays Held.** The worker may end before recovery succeeds: by cancellation, `shutdown`, `Runtime::drop`, an `Err` or a panic, or after three failed segments (for example because the Task changed). It then returns or unwinds without retiring the plan, and its `WorkerLifetime::drop` revokes the slot and marks the SAME plan reconciliation-ready. From then on two refusals hold. `publish_exact` refuses because the worker is no longer live. `validate_rollback` also refuses: it now requires that no contract exists for the planned Workflow, and here the committed contract and Driver row both exist. `reconcile_driver_preparation` therefore returns `false`, and the plan stays Held in slot custody. Nothing revives the worker, reactivates the slot or publishes the cache from rows. `acknowledge_exit` additionally refuses to remove a slot that still retains a preparation.
- **Uncontained panic.** A panic outside the contained boundary while the SharedStore guard is held (for example precommit, S2 `Panic`) unwinds through that guard and poisons it, as today. The Immediate transaction rolls back when dropped, but no code proves that rollback through the poisoned Store. Service sweeps and `shutdown` return "state poisoned", and the plan stays retained. No code clears or bypasses that poison. A panic outside every Store guard (S4 `Panic`) poisons nothing and is the actual-exit case above.
- **Shutdown accounting.** `observe_task_drivers` keeps its bounded sweep. It reconciles only the rotating page of at most 64 plans that `pending_preparations` returns (`runtime/driver.rs:503–541`, c979), and it acknowledges exits as it does today. Then, after every Store guard of the sweep has been released, it returns the number of unfinished jobs plus `DriverRegistry::retained_preparations()?`.
  - **The new method.** It is read-only and counts, across the whole registry (≤4096 slots), the slots that still retain a preparation. It locks `entries` and then each slot's `preparation` Mutex, in the same order as `pending_preparations`. It clones, constructs, retires and grants nothing, and takes no Store lock. A poisoned registry or preparation custody Mutex returns `Err`, which propagates.
  - **Meaning of zero.** Zero means no retained preparation exists anywhere in the registry, not merely that the visited page reconciled. A Held plan outside the page therefore keeps the count positive.
  - **Consumers.** While any Held plan exists, even after every job has finished, `shutdown`'s existing loop stays pending and then fails with "Task Driver shutdown remains pending with owned handles". Store errors still propagate. A live worker's in-flight preparation may also be counted, alongside its unfinished job. The service loop's delay treats the count only as a poll hint.
- **Cancellation.** The `select!` in `drive` can cancel only at awaits. Nothing is written up to and including the first admission await, and the Store section has no await. A cancellation during step-8 recovery comes after the commit. It is therefore an actual exit, and the plan stays Held.
- **Revocation, shutdown or Runtime Drop before admission.** Each of these revokes the slot, so the activation is cancelled or refused before the Store section. No Workflow and no contract remain, and the SAME retained plan is reconciled as a rollback.
- **Process death after commit.** A new epoch never reconstructs the plan. `fresh` refuses the Task (G5 §4.2). The contract stays immutable and keeps protecting the Workflow. This design makes no process-death guarantee.

### 5.8 Same-Workflow multi-alias and restart

- **Within one epoch.** Every Executor and Reviewer allocation is admitted by membership, whatever its Unit, profile or port. PR-2 may choose any reviewer listed in `task.reviewers` without a new contract. An alias outside the roster refuses, and a later change to a Task's reviewers leaves its Workflow unable to admit the new alias, which is Held rather than updated.
- **Restart (unmet; explicit later requirement).** The contract `owner_epoch` and `origin` are activation facts, and the trigger pins the operation epoch. A Task owns exactly one Workflow (`state/mod.rs:772–774`). A cross-epoch continuation therefore needs the genuine restart producer to add an append-only, permit-guarded successor relation keyed by (workflow_id, epoch), carrying the new installation and a re-derived roster from that restart's SAME composition. It must extend `binding_operation_allocation`, the §5.6 decision outside Store and `validate_contract` together, through a schema migration with a permit-catalogue entry. It must never UPDATE this row, and `legacy_held` stays nongrant. Until then the restart gate is not met and is not redefined.

## 6. Implementation sequence (Sol high)

Work in a new worktree branched from the integrated `c9791175d153b4801c2fd42cc0e557b84a8e477e`, which contains `a21` and G5. Never use or modify any other active worktree. Commit each item, and keep the worktree clean before every control run.

1. §5.1 installation identity and `NativeCompatDeclaration::digest`.
2. §5.2 `member_digest`, `roster_digest`, `ActivationRoster`, `activation_roster`, `installed_native_port` and the issuer prefilter.
3. §5.2 `CompositionIdentity`; §5.3 `NativeActivationPlan`, the `plan_initial_input` parameter, `admit_activation`, `admit_activation_recovery`, and the `initialize_driven` signature, ordering and call site.
4. §5.4 the contract write; §5.7 the `validate_result`/`validate_rollback` extensions, the contained postcommit boundary and `ActivationCommit`, SAME-worker recovery, the registry-wide `observe_task_drivers` count (`DriverRegistry::retained_preparations`) and the `acknowledge_exit` conjunct.
5. §5.5 the original-row Record images and the permit windows for every Driver-lane writer.
6. §5.6 the decision outside Store, the retained exact columns and the body `members` field.
7. The §8 seams (`#[cfg(test)]` only), the fixture migration (§7), and the §8 controls and mutants.
8. Master updates in the same implementing PR, after the source exists: `master/workflow-engine.md` (the Issue43 status block and §17: activation contract semantics and protected Workflow writers) and `master/agent-adapter.md` §15 (installation identity and roster member).

## 7. Impact analysis

| Changed / consumed | Consumers (V) | Handling |
|---|---|---|
| `NativePhasePort::installed` signature and the `installation` field | `adapter.rs:420` only | additive identity; tests that call `from_managed_config` are unchanged |
| `NativeCompatDeclaration::digest` (new read-only accessor) | the new `member_digest` only. The existing `compatibility` readers (`compat.rs:70,92`, `runtime/installation.rs:139`, c979) are unchanged | read only. `NativeCompatQualification::declaration_digest` and its consumer `execution/native/registration.rs:50` stay `pub(super)` and unchanged |
| `ActivationRoster`, `ActivationAdmission` (`runtime::installation`) and their `state` re-export | `activation_roster`, `plan_native_activation`, `admit_activation`, `admit_activation_recovery`, `initialize_inner` | module-private construction; re-exported in the same way as `InstalledDriverComposition` (`state/managed_binding/mod.rs:31`, c979) |
| `CompositionIdentity` and the new private `InstalledDriverComposition.identity` field | minted only in `Runtime::installed_driver_composition` (`runtime/installation.rs:148–155`, c979); read only by `ActivationRoster::new` and `is_same_composition` | not re-exported; `validate_for` and the other composition accessors are unchanged |
| `initialize_driven` signature and ordering | `runtime/task_driver.rs` `drive` (c979) only | passes the claim's composition; admission comes after every await |
| One more `control_admission` acquirer | `runtime/service.rs:45,115`; the `task_driver.rs` startup closure; `control.rs:304,330,365`; `phase_supervisor.rs:933,1007,1096,1146,1179,1232`; `phase_handoffs.rs:655`; `admission.rs:37` (`try_lock`) (c979) | the implementer reads each one to confirm that none awaits admission while holding SharedStore. Activation holds admission only for its synchronous segment, well within `shutdown`'s 5 s admission timeout |
| `plan_initial_input` signature; `InitialInput` gains the activation plan and the `records` mutation | `workflow.rs:1039–1041`; `plan_initial_gate`, `plan_first_executor`; `put_workflow_transition_inner` | retained in the same Arc; images and values built outside Store |
| `record_image` visibility (`marker_rows.rs`) | `workflow_mutation` and its existing marker users | widened within `crate::state` only; behaviour unchanged |
| `put_workflow_transition_inner` | activation, gate, first-Executor, verification, readonly, result and legacy callers (`state/mod.rs:592–662`, `workflow.rs:674–806`) | the contract write runs only for an activation that carries a plan; the windows run only for `DriverGate`/`DriverFirstExecutor`; other callers are unchanged and, as in §8, refused on contracted Workflows |
| `put_workflow_transition_inner` result and `activate_driven_workflow` → `ActivationCommit` | `initialize_inner` is the only consumer of `Deferred`; `activate_managed_workflow`, `apply_driven_initial_gate`, `reserve_driven_first_executor` and the other wrappers (`state/mod.rs:592–662`) | only the activation-with-plan path can return `Deferred`; every other wrapper requires `Published`, so its behaviour is unchanged |
| `observe_task_drivers` count, the new read-only `DriverRegistry::retained_preparations`, and the `acknowledge_exit` removal conjunct | the service loop's delay and `shutdown`'s pending loop consume the count (`runtime/service.rs:20–41,89–92,133–140`); `retained_preparations` and `acknowledge_exit` are called only by `observe_task_drivers` (`runtime/driver.rs:503–541,572–602`, c979) | Held plans now count as pending across the whole registry, not only within the 64-plan page. The count reads custody after the sweep's Store guards are released, uses the same lock order as `pending_preparations` and returns poison as `Err`. Store errors still propagate. An activated slot was already kept, so removal is unchanged for a never-activated slot |
| Contract existence enables the record guards (`schema.rs:316–336`) | every Workflow writer of a driven Task, including `observe_workflow_gate` (`state/mod.rs:1114`, `workflow.rs:2217`, c979) | the Runtime lane has only Driver-planned writers (§5.5 inventory), the marker and the binder; generic writers are refused; PR-1–PR-4 consumers must use exact permits |
| `validate_result` / `validate_rollback` | `publish_driver_preparation`, `reconcile_driver_preparation` (`preparation.rs:601–660`) | exact contract presence or absence over the retained columns, with no decode; other plans are unchanged |
| Marker predicate | `plan_marker_publication`, `validate_contract` (`publication.rs:235–298,426,477`) | §5.6: the decision is made outside Store, and `MarkerPublicationPlan` retains `origin`/`profile_digest`; no test asserts the old messages (grep) |
| Contract body and budget | `charged_scope_bytes` (`publication.rs:304–332`) | ≤4096 B, already charged |
| `managed_phase_required` (`protection.rs:66–79`) | legacy Native6 | already true once the Workflow exists; unchanged |
| Issuer prefilter | G5 sweep, C2–C15 (c51 `runtime/installation/tests.rs`) | claiming fixtures declare a reviewer alias (the same fixture program under its own declaration); the undeclared and smoke fixtures still refuse earlier |
| `#[cfg(test)]` seams S1–S5 (§8) | `EngineHooks` (`workflow.rs:545–552`, c979), the retained activation plan and a Runtime test field read by `observe_task_drivers` | test builds only; timing, fault or borrowed probe only; no constructor, row or grant |
| New constants | ≤8 reviewers, ≤9 members, two digest domains | local; not shared |

**Not affected:** the schema, `SCHEMA_VERSION`, triggers, the permit catalogue, `legacy_held` migration and rows, the verification contract, the record-only binder, the Source handoff, PhaseJobs, transport, quota, attention, user settings, credentials and hooks.

## 8. Controls (P; none executed)

Every control uses the compiled `Runtime::new` + `start` installed graph, Goals accepted through the real HumanIngress, the service sweep and `native_fixture.py` configured as `AgentConfig.command` with a declaration. A fixture is external protocol wiring and never Native qualification. A raw-SQL or forged row is allowed only as a negative stimulus and earns no positive credit. A compile failure or a SETUP stop is never a positive result or a kill. A mutant is killed only by its named assertion, after the control's genuine normal setup has succeeded.

A refusal that happens before the named stage earns no credit for that stage.

**Seams (P).** The seams exist only under `#[cfg(test)]` and are absent from non-test builds. They inject timing or faults only and never grant authority. The existing `before_reserve` and `before_publication` hooks run in other paths and never pause activation.

| Seam | Location | Modes |
|---|---|---|
| S1 `before_activation_admission` | an `EngineHooks` field, awaited in `initialize_inner` after `plan_initial_input` retains the plan and before `admit_activation`, inside `drive`'s existing `select!`. It receives a borrowed `ActivationProbe<'_>` that references the genuine composition, retained plan, `WorkerLifetime`, Task and Workflow Record. The probe constructs, stores and returns nothing | async pause |
| S2 `activation_precommit` | carried by the retained plan from an `EngineHooks` field set before the Driver starts; called in `put_workflow_transition_inner` after `finish_input_tx` and before `tx.commit()` | `Block` (a std barrier on a multi-thread runtime with at least four workers), `Fail` or `Panic` (uncontained) |
| S3 `activation_postcommit` | the same carrier; called inside the contained postcommit boundary after `tx.commit()` returns `Ok` and before `publish_driver_preparation` | `Fail` or `Panic` |
| S4 `activation_deferred` | the same carrier; called in `initialize_inner` on `Deferred`, after the Store guard and the admission have dropped and before the first recovery admission | `Pause` (async, cancellable), `Exit` (return `Err`) or `Panic` (outside every Store and admission guard) |
| S5 `before_service_reconcile` | a `#[cfg(test)]` Runtime field set before `start`; called in `observe_task_drivers` before `pending_preparations` | `Block` (a std barrier). After the test closes it, only the first arrival parks, which is the service loop's sweep. The test observes that park, and its own later direct `observe_task_drivers` calls pass. The test opens it before `shutdown` |

| ID | Control |
|---|---|
| CA1 | For each provider, with executor `worker` and reviewers `rev-a` and `rev-b` (both declared). **Activation stage:** the Workflow, first Context, Driver, verification contract and exactly one composed contract appear in one commit. The contract row is byte-equal to the plan, `origin` equals the registry installation, and the epoch equals the runtime epoch. `members` equals an independent test-side recomputation from each port's accessors and the SAME installed declaration, using the §4.2 encoding. **Gate stage:** the Reserve, Claim and Complete edges and the first-Executor reservation each commit one Workflow UPDATE. Its version is the original version plus one, its `updated_at` is the planned timestamp, and it consumes its one `records` permit. Then come the marker, `Launched`, the record-only binder `Bound`, `Task.version == marker_task_version` and a contract that is still byte-identical afterwards |
| CA2 | Four Tasks in two Projects (two Claude, two Codex executors): one contract each, with its own scope and roster and no cross-Workflow reuse |
| CA3 | Paired with CA1 (identical configuration except the named difference): no reviewers, a reviewer alias absent, a reviewer without a declaration, or nine reviewers → `Skipped(Composition)`, with zero Driver/Unit/Source/Workflow/contract rows while a sibling is claimed |
| CA3b | **Sibling compositions.** One Runtime claims sibling Tasks A (reviewer `rev-a`) and B (reviewer `rev-b`). Both workers pause at S1 after genuine setup, each with its own retained plan, and no Workflow or contract row exists yet. In B's S1 hook, the test also obtains genuine compositions from the SAME issuer, `Runtime::installed_driver_composition`: A′ for Task A and B′ for Task B. Each has a fresh identity. **Cross-pairs, before any write:** (i) `plan_native_activation(A′.activation_roster(task_A), B's Record, B's Task)` refuses outside Store. The assertion is made on that call's own result, before any ticket, retention or admission. (ii) `A′.admit_activation(B's plan, B's lifetime)` refuses after admission and before Store. (iii) `B′.admit_activation(B's plan, B's lifetime)` has the same Task, the same association and the same shared pointers, but its identity differs, so it refuses. After each of these, zero Workflow and contract rows exist. **Valid pairs:** both workers are then released. Each commits exactly one contract whose `members` equal its own recomputation, and the two roster digests differ |
| CA4a | Paused at S1, then either `shutdown` or dropping the last external `Arc<Runtime>`. The first failure is Driver cancellation at the S1 or admission await. Zero Workflow and contract rows remain, the SAME plan is reconciled as a rollback, and no second plan exists. This credits the property "no activation after a stop". It does not credit the post-admission conjuncts, because revocation makes them redundant here |
| CA4b | Linearization with S2 `Block`. While the segment is blocked, start `shutdown()` on another task. For a bounded 1 s, `is_stopping()` stays false and `shutdown` stays pending on admission. After release: one commit, one contract and a published Driver cache, then `shutdown` completes within its 5 s bound. A separate variant drops the test's own `Arc<Runtime>` while holding a `Weak` and asserts that the `Weak` still upgrades during the block |
| CA4c | S2 `Fail`: rollback, with zero Workflow and contract rows. The service proves rollback of the SAME plan, and nothing retries the INSERT |
| CA4d | **Recoverable fault.** S5 is closed. S3 `Fail` runs, and separately S3 `Panic`, each after the genuine setup reaches commit. The activation call returns `Deferred`, and `owner.store.lock()` is `Ok` (not poisoned). The slot's JoinHandle stays pending, no exit is observed, and the SAME plan `Arc` stays retained until the worker's own admitted recovery publishes it. Then: exactly one byte-identical contract row, the planned Driver cache, no second INSERT and no new plan. The SAME worker then commits the Reserve edge. S5 opens afterwards, and the service finds nothing to reconcile |
| CA4e | S3 `Fail` with S4 `Pause` and S5 closed. A Task change is made through a legitimate existing control route, then S4 is released. Each recovery segment's publication fails `validate_result`, and rollback also fails. After three segments the worker exits. Once S5 opens, `reconcile` returns `false` for the SAME plan, the plan stays Held, and `observe_task_drivers` counts it. No marker or row-derived grant appears. If no legitimate route changes the Task at `c979`, the result is SETUP with no credit |
| CA4f | Paused at S1, a Context or frame drift through a legitimate route → the existing refusal in `validate_input_before_tx`, and no contract |
| CA4g | **Actual exit stays Held.** S5 is closed. S3 `Fail` runs, then separately S4 `Exit` and S4 `Panic`. The worker ends, and `observe_finished` records `Failed` or `Panicked`. S5 then opens. `reconcile_driver_preparation` returns `false` for the SAME plan `Arc`. The slot binding stays the original pre-image, and exactly one contract exists. SharedStore is not poisoned. `observe_task_drivers` returns at least 1 while no job remains unfinished. Credit comes from these reconcile and count assertions. `shutdown`'s pending error is recorded, but its timeout is not credited |
| CA4h | **Uncontained panic.** S2 `Panic` unwinds through the SharedStore guard. A fresh read-only connection shows zero Workflow and contract rows. `owner.store.lock()` is `Err` (poisoned), `observe_finished` records `Panicked`, and the SAME plan stays retained. The service sweep and `shutdown` both return "state poisoned". No code path recovers the guard |
| CA4i | **Registry-wide count across pages.** The configured global and per-Project Driver limits are ≥128 (genuine bound ≤4096, `state/runtime/driver/claim.rs:249–250`, c979). **Setup:** 128 sibling Tasks are genuinely claimed, and every worker pauses at S1 with its plan retained. The test closes S5 and observes the service loop parked there. The S2, S3 and S4 hooks select their mode by the plan's Task. 127 workers run S2 `Fail`: each rolls back, leaves zero rows and exits. One worker runs S3 `Fail` and then S4 `Exit`, leaving one contract and an exited Held plan, as in CA4g. **Sweeps:** after all 128 jobs have finished, the test makes two consecutive direct `observe_task_drivers` calls. Two 64-plan pages cover all 128 plans whatever the cursor. Whichever page excludes the Held plan therefore visits only rollback plans. **Credit:** each call returns ≥1. After the second call, `retained_preparations()` is exactly 1, the slot's retained preparation is the SAME Held `Arc` (`Arc::ptr_eq`), and exactly one contract exists. **Paired all-rollback case:** all 128 workers run S2 `Fail`. The first call returns ≥1, then the second returns 0 and `retained_preparations()` is 0. Credit comes only from these return values and custody assertions; `shutdown` and its timeout earn none. Fewer than 128 genuine claims is SETUP, with no credit |
| CA5 | How far the contract predicate is reachable in this increment: (a) **Membership.** The first-Executor alias must equal `task.executor` before the contract is read (`marker_plan.rs:245–251`, c979). No genuine first-Executor object therefore reaches membership with a non-member, so this moves to CA-R. (b) **Foreign Workflow.** It is refused at `marker_plan.rs:194–200` first. (c) **Migrated `legacy_held` Workflow.** It has no Driver-lane route, and it would also fail on epoch 0, the NULL profile and the missing `members` (`schema.rs:518–534`). (b) and (c) are negative regressions only. (d) **Integrity.** The decision function outside Store is given the genuine allocation's member and installation plus a negative-only forged raw row whose members do not hash to `profile_digest` → integrity refusal. This counts as unit integrity only, never as a genuine-stage kill |
| CA6 | A public-constructor generic Workflow write, or `observe_workflow_gate`, on a contracted Workflow is refused with "private managed Record writer required", and the Workflow is unchanged. UPDATE, DELETE and REPLACE of a genuinely produced contract are refused |
| CA-R | A Reviewer-role marker admitted by membership. It also covers a Reviewer alias added to `task.reviewers` after activation, which passes `marker_plan.rs:245–251` but fails membership. **Unmet until PR-2** produces a genuine Reviewer Unit; SETUP until then |
| CA-E | A cross-epoch continuation: **unmet until the restart producer** (§5.8) |

**Mutants (compiled).** Each kill names the first observable failure.

| Mutant | Killed by |
|---|---|
| producer call removed | CA1 activation stage. `validate_result` in `finish_input_tx` refuses the absent contract before commit, leaving zero Workflow and contract rows. The mutant never reaches the marker stage |
| producer and the `validate_result` presence check both removed | CA1 gate stage. The first Reserve window's `ensure_consumed` refuses, because without a contract the permit goes unused |
| reviewers dropped from the roster | CA1 members assertion |
| port origin, program or declaration omitted from the member digest | CA1 recomputation |
| `records` old image taken from `record_before` | CA1 gate stage, at the first Reserve: "private managed Record writer required" |
| gate window removed | CA1 gate stage, at the first Reserve: same message |
| first-Executor window removed | CA1 first-Executor stage: same message |
| admission not acquired | CA4b: `is_stopping()` becomes true during the blocked segment |
| composition identity pointer removed from `is_same_composition` | CA3b (iii): the fresh genuine identity B′ is admitted |
| the whole three-part roster binding (Task, executor and reviewer equality) removed together from `plan_native_activation` | CA3b (i): the cross-pair plan is constructed, so the assertion on that call's own result fails before any ticket, retention or admission |
| any one roster binding comparison removed alone | not credited: CA3b (i) differs in both Task and reviewer, so a remaining comparison still refuses. Each individual comparison is defense in depth. Individual credit would need a genuine, legal pair that isolates that one comparison. None is specified, and an unavailable setup is SETUP with no credit |
| Task/association linkage conjunct removed alone from `admit_activation` | not credited: the identity conjunct refuses first in every reachable cross-pair |
| contained boundary removed (S3 failure propagates out of `drive`) | CA4d `Fail`: the worker exits and never commits the Reserve edge |
| `catch_unwind` removed, or moved outside the Store guard | CA4d `Panic`: `owner.store.lock()` is poisoned |
| boundary widened over precommit work | CA4h: SharedStore is not poisoned |
| poison recovery (`into_inner`/`clear_poison`) added to a service Store acquisition | CA4h: the sweep proceeds instead of returning "state poisoned" |
| `observe_task_drivers` returns unfinished jobs only (registry-wide count dropped) | CA4g count assertion; CA4i |
| count limited to the `false` results of the reconciled page | CA4i: the call whose page excludes the Held plan returns 0 |
| registry count read before the page's reconciliation | CA4i all-rollback: the second call returns 64 instead of 0 |
| `publish_exact` liveness conjunct removed | CA4g: the cache is published for a revoked worker, and `reconcile` returns `true` |
| `is_autocommit` check removed from the boundary | not credited: the publication's transaction always rolls back by drop, so no causal route exists |
| `acknowledge_exit` preparation conjunct removed | not credited: activated slots are already kept |
| prefilter reviewer conjunct removed | CA3 (the refusal moves after the Source rows) |
| digest check removed | CA5d, as unit integrity only |
| strong Runtime not retained during the segment | not credited. The service loop may hold its own strong reference while it waits on Store, so the CA4b `Weak` assertion cannot isolate this mutant |
| membership check removed | not credited in this increment; credited at CA-R after PR-2 |
| scope conjunct removed; `composed` filter removed | not credited: earlier guards (`marker_plan.rs:194–200`), or the epoch, profile and `members` checks, refuse first |
| path, epoch or instance check in `write_native_activation_tx` removed | not credited: no in-process causal route exists, and the ticket's and Driver's epoch currency checks run first |
| `validate_result` presence check removed alone | not credited: equivalent while the producer runs |
| `validate_rollback` contract-absence check removed | not credited: equivalent under atomicity |
| `ensure_consumed` removed from a `records` window | not credited: the trigger already requires the exact permit |

Review check (not a mutant): `validate_contract`, `validate_result` and `write_native_activation_tx` contain no decode, canonical-encoding or digest call.

## 9. Current status and limits

- Nothing in this document is implemented, built, tested, reviewed or qualified.
- This lane makes a first-Executor marker reachable only on the integration branch containing G5 (`c979`). It does not make a full Workflow, Reviewer or later phase, restart continuation, the four-Task both-OS run, the official real-CLI N1/N4/H/Q qualification, install, CI, regression or Clippy GREEN, or merge readiness. Each remains at its owning contract. The reported Runtime19 and local install smoke PASS do not qualify this lane (A). Strict Clippy is RED (A), and no CI has run on the current source.
- The next step is one finding-side Sol high delta review of this correction: the CA-H3 original composition identity, the CA-H3/H5 recoverable publication and shutdown accounting, and the CA-H5 panic boundary. Implementation starts only after zero confirmed unmet items. This correction grants no source, qualification or merge approval.
