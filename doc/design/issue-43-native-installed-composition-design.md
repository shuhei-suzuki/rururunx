# Issue 43: Native installed composition and service Task admission (G5)

## 1. Status, pins and categories

1. This is a proposed authority/contract HOW supplement for gate G5 (Installed composition / static admission) of the [transport HOW](issue-43-native-transport-integration-design.md) §16. It connects the following approved contracts and does not replace any of them: the [prepared HOW](issue-43-native-prepared-producer-design.md) at `a3a41965ee29291523c2c4edb47b3f1fd119a047` (§§8, 9.3, 10.3, 13, 15); the transport HOW (§§1.1, 7, 10, 16); the [Runtime Driver producer](issue-43-runtime-driver-producer-design.md) (§§2, 5 and its first Executor increment); the [Source handoff](issue-43-source-native-handoff-design.md) §5; the [marker dispatch](issue-43-original-source-marker-dispatch-design.md); the [phase supervisor](issue-43-phase-supervisor-integration-design.md) §9; the [managed binding](issue-43-managed-binding-design.md); and the [Runtime scheduler](runtime-scheduler-integration-design.md) §§5–6. It is not a requirements change, does not update the master design and claims nothing implemented.
2. **Pins.** Source facts are read at `f6fd556c` (clean author worktree: G1 `c83960c1` plus the G2 net `c2dec6ac..0c6fe3c7`). G4 facts are read with `git show` at `7838a7348aa29efa5edda714675ce45fbf61cc7e` (G2 base, not Root-integrated). Paths are relative to `crates/rrx/src/`.
3. **Categories.**
   - **V (verified):** read in source at the stated pin. No build, test, lint, Agent or OS run was executed for this HOW.
   - **P (proposed):** this contract; not implemented.
   - **A (acknowledged):** reported by its owner and relied on here without re-verification. G1 is adopting B1/B2, with no final source proof. At `f6fd556c`, the all-target check passes and the 19 nongrant primitives pass, but the full default-sandbox regression is 334 passed / 468 failed / 33 ignored and Clippy is RED, so nothing is qualified as a whole. G4's source controls are primitive, and its independent review and genuine assembly are unverified. PR52 is DRAFT and CONFLICTING at `b191b466`, with an old Ubuntu FAILURE and macOS CANCELLED. Root's mandatory SourceStop/Cancel findings, and the author/permission decisions on them, are unresolved. No Native first-Task, four-Task, both-OS, install or dogfood run is qualified.
4. **Never authority here:** public capability metadata or `probe`, configured aliases, scheduler/attention rows, Driver/Unit/marker SQL, test-only constructors, static fixtures, fake grants, boolean or optional availability switches, copied DB state, Native permission bypass, a Task.version-changing binder, a generic legacy binder fallback, or manufactured #19 proof. rrx is not a sandbox and makes no process-death claim. Official CLIs run on the host with the user's usual HOME, config, hooks and auth. Nothing here inspects, copies or transfers them, and no VM, container, separate user, root or outer sandbox is used.

### 1.1 One-line acceptance condition (P)

The real retained Runtime graph admits a trusted ready Task through its actual Driver/Source/marker/Native/binder producers while forged metadata/current SQL cannot replace installation or original input custody. This is an incremental prerequisite, not an MVP completion criterion.

## 2. Verified missing seams (V)

| Item | Fact at `f6fd556c` unless marked |
|---|---|
| Runtime graph | `runtime/mod.rs:18–34` retains the owner, config, DriverRegistry, PhaseSupervisor, PhaseJobs, PhaseDispatcher, PhaseHandoffs, admission flags and the loop handle. It has no AgentRegistry, Sources, Gates, verifier or Engine. `Runtime::new` (`:37–82`) is synchronous. Its production caller is `cli/service.rs:32`; tests call it in `runtime/tests.rs`, `runtime/phase_supervisor/tests.rs`, `runtime/phase_handoffs/tests.rs`, `state/managed_binding/tests.rs` and `execution/native/phase_fence_tests.rs` |
| Constructors | `AgentRegistry::from_managed_config` (`adapter.rs:330–422`) requires Tokio, an explicit provider and one resolvable executable for every agent. It parses the optional compatibility declaration and builds one `NativeSessions::with_limits` (`execution/native.rs:165–178`), which binds `owner.ipc_path()` (`execution/ipc.rs:119–121`) and starts the 30 s cleanup sweep (`execution/cleanup.rs:20–35`). Only a test calls it (`execution/phase.rs:307`). `ManagedWorkflowSources::new` (`execution/workflow_source.rs:395–402`), `ManagedWorkflowGates::new` (`execution/workflow_gates.rs:47–53`), `ManagedVerifier::new` (`execution/verification/mod.rs:222–238`) and `WorkflowEngine::new` (`workflow.rs:587–612`) have no production caller |
| Ingress | Same-UID `HumanIngress` (`runtime/control.rs:179–219`) reaches `accept_runtime_goal` (`state/runtime/goals.rs:24`), which writes the Goal as Running (`:117`) and writes its `scheduler_tasks` rows (`:179`). Successful controls wake the loop (`runtime/control.rs:325,387`) |
| Composition | `InstalledDriverComposition` (`state/managed_binding/composition.rs:16–24`) is non-Clone. It holds a Weak Runtime, the owner, a Weak PhaseSupervisor, the original Task, Sources, the Engine and the selected `NativePhasePort`. `validate_for` (`:26–50`) checks pointer identities, the running service and `WorkflowEngine::matches_composition` (`workflow.rs:569–586`). Its issuer `Runtime::installed_driver_composition` (`:70–75`) always bails |
| Driver admission | `Runtime::admit_task_driver` (`runtime/task_driver.rs:13–67`) awaits control admission, validates the composition, then plans, reserves and claims. It spawns the real worker, which runs `prepare_driven` → `initialize_driven` → `step_driven_initial` (`:69–116`). It has no caller |
| Service loop | `runtime/service.rs:44–102` reconciles only attention, pending phases and Driver exits. `Store::reconcile_runtime_attention` (`state/runtime/service.rs:5–89`) writes the hold `native_binding_unavailable` to every scheduled Task. Nothing selects or admits a ready Task. `shutdown` (`:106–138`) holds control admission while it joins the loop, for up to 5 s |
| Readiness | `plan_initial_driver` (`state/runtime/driver/claim.rs:243–290`) reads three inputs. The pins snapshot (`state/runtime/driver.rs:96–152`) covers the accepted definition digest, Project Registered, Goal Running, a non-terminal Task and DAG membership. `fresh` (`claim.rs:233–241`) requires no Unit, execution, Source recovery, operation, Driver, Workflow or Context history. The third input is the hard prerequisites (`:109–231`). `claim_initial_driver` (`:313–350`) re-validates them, CASes the scheduler ranks, enforces the distinct-Task global and Project capacity (`:337–338`) and advances the Project and Goal rotations |
| Preflight | `require_managed_native_binding_composed` (`workflow.rs:529–534`) always refuses. Its callers are `preflight_native_adapter` (`:1417–1462`), `prepare_agent` (`:1471`) and `prepare_managed` (`:1656`); after the refusal point, `preflight_native_adapter` reads public `capabilities`/`probe` and requires `PreparedInputAdmission`. `preflight_native_adapter` is reached from legacy `step` (`:1180`, `:1941`) and from the driven lane (`workflow/driven_initial.rs:44,57,156`). `NativeAdapter::capabilities` (`adapter/native.rs:264–272`) does not include `PreparedInputAdmission` |
| Driven lane | Issue/Worktree gates and the first Executor reservation and offer exist (`driven_initial.rs:11–131,133–192,194–244`); every later phase bails (`:59`) |
| Source ingress | `Runtime::reserve_source_handoff` (`runtime/phase_handoffs.rs:659–677`) checks the owner and the alias. It does not check that the port belongs to an installed Registry |
| Root job → Native | Marker hand-off calls `PhaseJobs::start` (`runtime/phase_supervisor.rs:915`; `runtime/phase_jobs.rs:232–308`). That job awaits `start_phase` on the allocation's selected port and, on `Launched`, calls the record-only `bind_returned` (`:363`) |
| Native start | `NativeSessions::start_phase_inner` (`execution/native.rs:205–213`) awaits `begin_phase_preparation` and then bails. `begin_phase_preparation` (`execution/native/preparation.rs:760–858`) runs S0–S7 and then bails. Its `issue_prepared` (`native/prepared.rs:217–238`) still bails with "prepared provider effect-budget qualification pending" |
| G4 (at `7838a734`) | `NativeSessions::start_prepared_transport(&self, &Arc<NativePreparationCustody>, Arc<PreparedNativePhase>) -> Result<NativePhaseStart>` (`execution/native/transport.rs:496`) registers, spawns into `NativeChildCell` and transfers into Core. It waits up to 5 s for first-poll acceptance and returns `Launched`. `retain_transport` (`native/preparation.rs:40–44`) requires the SAME retained Prepared. At this pin, `begin_phase_preparation` and `start_phase_inner` still bail, and `NativeTransportCustody::request_stop` (`transport.rs:94`, `pub(super)`) has no Root caller |

## 3. Installed graph and ownership (P)

### 3.1 Production construction

`Runtime::new(owner, config)` stays the only production constructor, and tests use the same one. After the existing objects are built, it runs these steps once, in order:

1. `AgentRegistry::from_managed_config(&config, owner.clone())` → `Arc<AgentRegistry>`, holding the SAME NativeSessions, adapters and `NativePhasePort`s.
2. `ManagedWorkflowSources::new(owner.clone(), config.clone())` → `Arc<ManagedWorkflowSources>`.
3. `ManagedWorkflowGates::new(owner.clone(), sources.clone())` and `ManagedVerifier::new(owner.clone(), sources.clone())`.
4. `WorkflowEngine::new(owner.store(), registry.clone(), config.clone(), sources.clone(), Arc::new(gates))?.with_verifier(Arc::new(verifier))?` → `Arc<WorkflowEngine>`.

Any failure in steps 1–4 is recorded as an `InstallationRefusal`, and the Runtime is still constructed. Goal controls stay usable, no Driver is ever admitted and the existing attention hold remains. An invalid `Config` still fails `Runtime::new` (A). There is no retry, no lazy installation and no second construction path.

```rust
// runtime/installation.rs (new private module)
struct InstalledNativeGraph {           // non-Clone; never exposed outside `runtime`
    registry: Arc<AgentRegistry>,       // the ONLY managed registry of this owner
    sources: Arc<ManagedWorkflowSources>,
    engine: Arc<WorkflowEngine>,        // owns the SAME-Sources gates and verifier
}
struct InstallationRefusal(Box<str>);   // <=256 B diagnostic; nongrant
// Runtime gains two fields, set once in new() and never replaced:
//   installed: std::result::Result<InstalledNativeGraph, InstallationRefusal>,
//   admission_cursor: std::sync::Mutex<Option<CandidateKey>>, // §4.1 order key; nongrant hint
```

The `Result` is not a switch. No boolean, configuration flag or row can turn a refusal into a graph, and only the real objects of the `Ok` variant can be borrowed. Neither variant is serialized, cloned or reachable from outside `runtime`.

### 3.2 Composition issuer

`InstalledDriverComposition` moves to `runtime/installation.rs`. It keeps the same fields, `validate_for` and accessors, and gains crate-private `selected()` and `original_task()` accessors. `state::managed_binding` re-exports it, so `claim.rs` and `task_driver.rs` keep their paths. Its fields become private to the issuing module.

```rust
impl Runtime {
    pub(crate) fn installed_driver_composition(self: &Arc<Self>, task: &Task)
        -> Result<InstalledDriverComposition>;
}
```

Every conjunct is checked before any Store write, helper, Source slot or Driver reservation:

- `installed` is `Ok`, and `service_running()` holds.
- `port = graph.registry.native_phase_port(&task.executor)?`, and `port.selected_adapter()?` upgrades the SAME adapter and NativeSessions.
- That adapter's installed `compatibility` declaration is `Some`. This is the static half of prepared HOW §10.3(c); the role-specific S1 still runs at start.
- `graph.engine.matches_composition(&self.owner, &graph.sources, &port)` holds.
- The value is built from `Arc::downgrade(self)`, `Arc::downgrade(&self.phases)`, `graph.sources`, `graph.engine`, `port` and a clone of the Task read under control admission.

The Reviewer alias is not required, so a Task is not refused before its implemented first Executor just because a later phase is still unsupported (§5).

### 3.3 Objects, owners and Weak edges

| Object | Created by | Owned by | Edges |
|---|---|---|---|
| RuntimeOwner | `RuntimeOwner::open` in `serve` (A) | serve, Runtime and every component | none to Runtime |
| DriverRegistry, PhaseSupervisor, PhaseJobs, PhaseDispatcher, PhaseHandoffs | `Runtime::new` (A) | Runtime | the composition holds the supervisor only as Weak (phase supervisor §9) |
| Registry, NativeSessions, ToolServer, CleanupWorker, NativeAdapter, NativePhasePort | step 1 (P) | graph and Engine | the port holds Weak adapter and sessions (A, `adapter/native.rs:22–33`) |
| Sources, Gates, Verifier, Engine | steps 2–4 (P) | graph; the Engine owns Gates and Verifier | none to Runtime (V: `workflow.rs:553–565`, `workflow_source.rs:395–402`) |
| Composition | §3.2 (P), once per admitted Task | moved into `PendingDriverClaim` (A) | Weak Runtime and supervisor; strong Sources, Engine and port |
| Claim, worker, ticket, Source slot, allocation, handoff, marker, job, custody, actor, Prepared, transport, binding | existing producers (A, G1, G4) | Driver slot, Sources, PhaseSupervisor and PhaseJobs, as today | unchanged |

No new strong edge reaches Runtime. Dropping Runtime sets `stopping` and revokes Drivers (A, `runtime/mod.rs:115–121`), so workers fail their Weak upgrade. The graph is freed when the last worker's composition drops; NativeSessions' Drop then aborts the ToolServer and the cleanup sweep. A second `ToolServer` on the same owner fails at bind (V: `ipc.rs:120`), so each owner has only one managed registry. That is an observation, and nothing relies on it as authority.

## 4. Service Task admission (P)

### 4.1 Sweep

The supervisor loop (`runtime/service.rs:80–81`) calls `runtime.admit_ready_tasks()` after `observe_task_drivers`, and only while `service_running()`. A nonzero result counts as pending for `PhaseSupervisor::delay`.

```rust
impl Runtime {
    fn admit_ready_tasks(self: &Arc<Self>) -> Result<usize>;            // sync; loop-only
    fn admit_task_driver(self: &Arc<Self>, _admission: &tokio::sync::MutexGuard<'_, ()>,
        task: TaskId, composition: InstalledDriverComposition) -> Result<()>; // was async
}
impl Store {
    pub(crate) fn ready_driver_candidates(&self, instance: &str, epoch: u64,
        after: Option<&CandidateKey>, limit: usize)
        -> Result<(Vec<TaskId>, Option<CandidateKey>)>;                 // read-only; nongrant
}
```

1. If `installed` is `Err`, return 0 without any read.
2. Call `control_admission.try_lock()`. If it is busy, return 0; the next wake retries. The loop never awaits control admission, because `shutdown` holds it while joining the loop.
3. Read one page of at most 64 candidates (65-row sentinel) after the nongrant cursor. Order the page by the scheduler design's key `(project rotation, project_id, goal rotation, goal_id, queue_sequence, task_id)`, require the current owner epoch, and exclude Tasks that have a `task_drivers` or `execution_units` row. The page grants nothing.
4. Handle at most 32 candidates per sweep. For each one, read the Task, then call `installed_driver_composition`, then `plan_initial_driver`. Any refusal skips the candidate with no write; refusals include no port or declaration, a Goal that is not Running, an unmet hard prerequisite, existing history (`fresh`) and a rank change. Otherwise call `admit_task_driver`. If `reserve_pending`, `claim_initial_driver` or spawn refuses, end the sweep, because capacity or concurrency changed; the existing claim and close paths own that result.
5. Advance the cursor to the last examined key, and reset it at the end of the list. Every eligible Task is therefore examined within ⌈N/32⌉ sweeps. This bounds the opportunity to be admitted, not the completion time.

`admit_task_driver` keeps every existing step and check: Task read, `validate_for`, plan, `reserve_pending`, `claim_initial_driver`, `bind_claim`, spawn and wake. The only change is that it borrows the caller's guard instead of awaiting a new one. Its worker still acquires control admission itself before activation (A).

### 4.2 Readiness, quotas and refusals

- **Readiness.** The authority stays where it is today. The accepted definition digest, Project Registered, Goal Running, non-terminal Task, DAG membership, hard prerequisites and freshness are all checked in the plan and again in the claim Immediate (§2). The candidate read is only a prefilter.
- **Restart and preparation recovery.** `fresh` refuses a Task with any prior Unit, operation, Driver, Workflow or Context, and that Task keeps the attention hold. Pre-artifact recovery and the fresh-attempt restart producer stay unsupported (Driver producer §5).
- **Quotas.** G5 adds no counter or semaphore and pre-reserves nothing. Driver capacity is the claim's distinct-Task union against `global_max_sessions` and `max_tasks_per_project`. PhaseSupervisor slots (A), `NativeLimits` session caps (A) and G1's S6 provider admit-or-park quota are unchanged.
- **Attention.** `reconcile_runtime_attention` gains one predicate: it does not overwrite the attention of a Task that has a current-epoch `driving` row in `task_drivers`. All other Tasks keep the existing hold. No audit kind is added.

## 5. Driven Workflow consumer and phase support (P)

`step_driven_initial(&self, task, composition: &InstalledDriverComposition, lifetime)` takes the composition in place of its separate `sources` and `runtime` parameters, and `drive` passes `claim.composition()`. The step adds one private check:

```rust
fn preflight_installed_native(&self, c: &InstalledDriverComposition, task: &Task, phase: Phase)
    -> Result<Arc<NativePhasePort>>;
```

Before any namespace helper, Source capture or Store write, the check refuses unless all of these hold: `std::ptr::eq(self, c.engine().as_ref())`; the Weak Runtime upgrades and `composition_is_current` holds; the Task id equals the original Task's id and `task.executor == c.selected().alias()`; `Arc::ptr_eq(&self.registry.native_phase_port(alias)?, c.selected())`; and the phase is the first configured Executor with no prior Executor attempt (later Executors and retries belong to PR-2).

The check never calls `capabilities`, `probe` or `require_managed_native_binding_composed`. `PreparedInputAdmission` stays unadvertised: prepared HOW §10.3(c) only limits which adapters may advertise it and does not require advertising it. Driven lines 44 and 156 use the check. Line 57 (a Reviewer, or another non-Executor Native phase) is unchanged: it still refuses through `preflight_native_adapter` and the continuation bail before its own effects, and that refusal does not block the Task's earlier first Executor. `Runtime::reserve_source_handoff` is unchanged, because its only caller (`driven_initial.rs:172`) passes the port that this check has already proved pointer-equal to the composition's port.

**Old public standalone paths stay unsupported.** Legacy `step`, `preflight_native_adapter`, `prepare_agent`, `prepare_managed`, `NativeSessions::start` on a protected scope and `AgentAdapter::start_managed` keep their unconditional refusals. An Engine, Sources or Registry built through the public constructors cannot obtain a composition, even over the same owner, because the issuer exists only on `Runtime` and only for its own graph.

## 6. G1 → G4 connection (P)

```rust
// execution/native/preparation.rs
pub(super) async fn begin_phase_preparation(&self, launch: Arc<PhaseLaunchParts>,
    custody: Arc<NativePreparationCustody>) -> Result<Arc<PreparedNativePhase>>;
//   S0-S6 unchanged; the trailing bail becomes `let prepared = self.issue_prepared(&custody)?; Ok(prepared)`
// execution/native.rs
async fn start_phase_inner(&self, launch: Arc<PhaseLaunchParts>,
    custody: Arc<NativePreparationCustody>) -> Result<NativePhaseStart> {
    let prepared = self.begin_phase_preparation(launch, custody.clone()).await?;
    self.start_prepared_transport(&custody, prepared).await
}
```

`issue_prepared` returns the SAME Arc from `custody.retain_prepared(..)`, but only after G1-final removes its budget bail. `start_prepared_transport` accepts that Arc only when `retain_transport` finds it pointer-equal to the custody's Prepared (V at `7838a734`). There is no re-entry, no second `claim_start`, no rebuilt command and no other issuer. The `Launched` proof returns through the existing PhaseJobs record-only binder. Every error takes the existing `NativePhaseStartError` → `Failed`/`abandon` path (A, `phase_jobs.rs:275–290`), followed by G1 closure or Held. Both trailing bails are removed together, and only as part of this connection.

## 7. Merge order and the approved-contract conflict (P / A)

The approved contracts forbid issuing composition before its producers exist. Prepared HOW §10.3(a) requires that "G1–G4 and RN-1 are implemented and independently reviewed", transport HOW §16 keeps G5 "Remain closed", and Source handoff §5 keeps the issuer closed "until the genuine whole composition is present and positively qualified". Merging a first-Executor-only partial issuer ahead of them would therefore be a real conflict. It would also leave two hazards: without RN-1, a refused start keeps `phase_open=1` and consumes distinct-Task capacity; without G3, `shutdown` cannot stop a spawned child in the SAME custody. G5 is therefore one source increment. It may be developed on an integration branch, and it merges only after all of the following:

1. G1-final: B1/B2 adopted and the source independently reviewed.
2. Root integration of G4 `7838a734` onto the G1 line. V: `git diff --stat f6fd556c 7838a734` touches 26 files, including `state/execution/native_phase/quota.rs` and `quota_policy.rs`.
3. RN-1.
4. G3, the Root caller of SAME-custody `request_stop`, which must follow Root's own decision on the unresolved SourceStop/Cancel findings. This HOW neither decides nor closes those findings.

"Positively qualified" is met by the genuine account-free controls of §10, run through the production constructor on that branch before merge. Official-CLI N1/N4/H/Q/Install remain release gates, not merge gates. If Root or the reviewer reads Source handoff §5 as requiring official-CLI qualification before the issuer exists, that reading is a true circularity. The only way out is to qualify on the unmerged branch, and that must be stated openly rather than bypassed.

**Installation availability versus qualification.** Architectural availability means that the production `Runtime::new` + `start` reach §§3–6. Qualification is separate: it is the observed version, hook, auth and Native behaviour of the official CLIs on macOS and Linux (prepared HOW §13.4). Without a compatibility declaration in the user's config, production admits no Task. That declaration is the existing S1 input, not a switch. README, master, release and MVP claims stay unchanged until qualification.

## 8. Startup, state and stop failure table (P unless marked A)

| Point | Condition | Result |
|---|---|---|
| `Runtime::new` | invalid `Config` | Err (A) |
| `Runtime::new` | no Tokio, provider absent, executable unresolved, invalid declaration, IPC bind failure, Engine/Sources owner mismatch | `InstallationRefusal`; Runtime and controls usable; zero admission |
| sweep | `installed` is `Err`, stopping, or admission busy | return 0; no read or write |
| sweep | composition or plan refusal | skip the Task; no write |
| sweep | reserve, claim or spawn refusal | end the sweep; existing close-unactivated / Held paths (A) |
| worker | Weak Runtime gone, lifetime cancelled or revoked | existing refusal before the next effect (A) |
| driven step | unsupported phase or port mismatch | named refusal before its own effects |
| Native start | any S0–S7 refusal | G1 closure, then RN-1 or Held; job `Failed` |
| transport | G4 refusal or first-poll drop | G4 Held rules |
| shutdown | normal | `stopping` under admission, `stop_all`, `close_unmarked`; marked jobs reach SAME-custody `request_stop` through G3, otherwise stay Held (A until G3) |
| Runtime Drop | any | `stopping` and Driver revocation (A); graph freed after the last worker |

**Bounds.** Each Runtime has one graph. A sweep holds the control guard for at most one page of 65 rows and 32 synchronous plan/claim transactions, with no await. Per-Task bounds are unchanged (A): Task body ≤1 MiB, Driver metadata ≤128 KiB, fewer than 4096 Driver entries, and at most 128 pending phases, 128 jobs and 128 Native preparations. The cursor is a single key.

## 9. Impact analysis

| Changed / consumed | Consumers (V) | Handling |
|---|---|---|
| `Runtime::new` installs the graph | `cli/service.rs:32` and the five test files of §2 | Every Runtime built under Tokio now binds the owner IPC socket and runs the cleanup sweep. The supervisor, handoff and fence tests also build a registry or `NativeSessions` on the same owner, so the source increment enumerates them: runtime-module tests use the installed graph, and the others become explicitly asserted `InstallationRefusal` fixtures. Neither case gets a test-only constructor. ControlFixture (`/bin/true`, no declaration) is refused at the declaration conjunct, so its zero-Driver assertions still hold |
| `InstalledDriverComposition` relocation and issuer | `state/runtime/driver/claim.rs:27,91,313–316`; `runtime/task_driver.rs:5,31,46,71–74` | path re-exported; checks identical |
| `admit_task_driver` takes the guard | only the new sweep | no other caller (V) |
| `step_driven_initial` signature | `runtime/task_driver.rs:104` | composition passed in place of Sources and Weak Runtime |
| `preflight_installed_native` | `workflow/driven_initial.rs:44,156` | line 57 and the legacy callers keep `preflight_native_adapter` |
| Attention predicate | `reconcile_runtime_attention`; Goal status and Task page readers | driving Tasks no longer show the hold |
| `begin_phase_preparation` return type and `start_phase_inner` | `execution/native.rs:205–213`; G4 `transport.rs:496` | both bails removed together |
| New constants | sweep 64/65/32 and the cursor | local; not shared with any existing bound |

**Not affected:** SQL schema, `SCHEMA_VERSION`, permits and triggers; Task/Workflow/Context write contracts and the record-only binder; quota policy; the public `AgentRegistry`/`WorkflowEngine`/Sources constructors, which keep their signatures and give no route to composition; user settings, credentials and hooks, which are never read.

## 10. Controls (P; none executed)

Genuine controls use only `Runtime::new` + `start`, Goals accepted through the real HumanIngress (ControlFixture same-UID peer with isolated Git), the service loop's own sweep, and Agent programs configured through `AgentConfig.command` + `compatibility`. Account-free positives may configure the existing protocol fixture (`execution/native/native_fixture.py`) as that external program. The fixture stands in only for the CLI process, so it qualifies rrx wiring and never Native behaviour. A control that cannot reach its producer is a **SETUP/UNVERIFIED** result and earns no credit. A compile failure is never a kill.

| ID | Control | Needs |
|---|---|---|
| C1 | Installed graph: two compositions share the SAME Engine, Registry and port Arcs, and no Agent process ran at construction (fixture marker absent) | G5 only |
| C2 | No declaration (`/bin/true`): zero `task_drivers`, Unit and helper rows; paired with C5, differing only in the declaration | G5 only |
| C3 | Executor alias absent, a generic agent, or an invalid executable: issuer refusal or `InstallationRefusal`, while Goal controls still answer | G5 only |
| C4 | Forgery: an SQL-inserted `driving` row, an attention edit, a public generic adapter advertising `PreparedInputAdmission`, or a second Engine from public constructors yields no admission and no worker; legacy `step` still returns `ManagedBindingUnavailable` | G5 only |
| C5 | One ready Task: exactly one claim and rotation +1, then Source prep, Issue/Worktree, first Executor reservation, handoff, marker, job and S0–S7 on the SAME allocation | G5; with today's G1 it ends at the budget refusal, which C5 asserts |
| C6 | C5 continued: Prepared → registration → spawn → Core → `Launched` → record-only binder `Bound`, with Task.version unchanged by binding | G1-final and integrated G4 |
| C7 | A paused Goal, an unmet hard dependency and a prior-history Task are each skipped with zero writes, while a ready sibling is admitted | G5 only (the sibling reaches its claim) |
| C8 | Caps: `global_max_sessions=1` admits one of two ready Tasks; under default caps, 4 Tasks in 2 Projects (2 Claude + 2 Codex fixtures) are admitted with Project/Goal rotation | G5; the Native part needs G1 and G4 |
| C9 | Shutdown during a sweep, and while admission is held, completes within 5 s with no claim after `stopping` | G5 only |
| C10 | Runtime Drop during Source prep: Weak probes show the Runtime freed, no new effect, and the graph freed after the worker ends | G5 only |
| C11 | A Reviewer or later Executor phase refuses before its own effects after the first Executor ran | PR-1 to reach it; SETUP until then |
| C12 | Stop after spawn reaches SAME-custody `request_stop` | G3; SETUP until then |

**Required compiled mutants** (each must fail its named control): the declaration conjunct removed (C2); the port looked up in a fresh registry, or the graph rebuilt per sweep (C1); the driven preflight reverted to `preflight_native_adapter` or made to require a capability (C5); the sweep using `lock().await` (C9); a plan refusal ending the sweep instead of skipping (C7); the composition holding a strong `Arc<Runtime>` (C10); the attention predicate removed (C7 status read); `start_phase_inner` calling `issue_prepared` again or passing a non-retained Prepared (C6); the first-Executor conjunct removed (C11, SETUP until PR-1). A compile or SETUP failure is never a kill.

## 11. Later increments and consumer boundaries

A positive C5/C6 result is an intermediate dependency control. It is not full composition, the full Workflow or the MVP. The following remain required, each at its named consumer:

- **PR-1:** post-initial Driver continuation (`driven_initial.rs:59`, and the offer observation at `:133–192`), consuming the binding, the terminal and RN-1 `phase_closed`.
- **PR-2:** later Executor, retry and Reviewer offers (`execution/workflow_source/native_handoff.rs:448–536`), which widen `preflight_installed_native` per actor.
- **PR-3:** a Source-owned `ResultSnapshot` and `ReviewerArtifactLease` at A (`execution/attempts.rs:622`). The Engine's `managed_snapshots` (`workflow.rs:560`) stay legacy-only.
- **PR-4:** artifact retention that treats open custody as a live dependency (`state/execution/artifacts.rs`).
- G3, RN-1, cross-epoch closure and the restart fresh-attempt producer.
- N1/N4/H/Q/Install on both hosts, full regression and Clippy GREEN, CI, and #16.

The existing approved contracts already cover G1 (prepared HOW), G4 (transport HOW), the claim and worker (Driver producer), the handoff and marker (Source and marker designs) and the binding (managed binding). This HOW adds only §§3–6 and their controls.
