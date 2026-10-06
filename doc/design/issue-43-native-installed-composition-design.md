# Issue 43: Native installed composition and service Task admission (G5)

## 1. Status, pins and categories

1. This is a proposed authority/contract HOW supplement for gate G5 (Installed composition / static admission) of the [transport HOW](issue-43-native-transport-integration-design.md) §16. It connects the following approved contracts and does not replace any of them: the [prepared HOW](issue-43-native-prepared-producer-design.md) at `a3a41965ee29291523c2c4edb47b3f1fd119a047` (§§8, 9.3, 10.3, 13, 15); the transport HOW (§§1.1, 7, 10, 16); the [Runtime Driver producer](issue-43-runtime-driver-producer-design.md) (§§2, 5 and its first Executor increment); the [Source handoff](issue-43-source-native-handoff-design.md) §5; the [marker dispatch](issue-43-original-source-marker-dispatch-design.md); the [phase supervisor](issue-43-phase-supervisor-integration-design.md) §9; the [managed binding](issue-43-managed-binding-design.md); and the [Runtime scheduler](runtime-scheduler-integration-design.md) §§5–6. It is not a requirements change, does not update the master design and claims nothing implemented.
2. **Pins.** Current source facts are read with `git show` at `7be25fec1c7c2ce7ed1daee5bdf347818c9e31df`, the clean frozen combined source. It integrates the genuine G1 final net `50deb781..a096f83b` (V: `7be25fec` carries the same crate diffstat) and the G4 final source net `0c6fe3c7..4f1bf463`, the eight-file patch integrated by `81061d32` (V). A comparison of whole divergent-branch endpoints is not the G4 integration patch. Locations marked `f6fd556c` or `7838a734` are historical first-draft reads. Unless a row is marked `7be25fec`, the behaviour they state was spot-checked by symbol at `7be25fec` and is unchanged, but line numbers may have drifted, so implementers locate symbols by name. Paths are relative to `crates/rrx/src/`.
3. **Categories.**
   - **V (verified):** read in source at the stated pin. No build, test, lint, Agent or OS run was executed for this HOW.
   - **P (proposed):** this contract; not implemented.
   - **A (acknowledged):** reported by its owner and relied on here without re-verification. At `7be25fec`, fmt, check and build pass, and 50 scoped controls pass: 49 distinct primitive/head/inventory/process/closure/waiting controls plus one actual same-UID accepted-Goal/page test. None of them is a Native producer qualification. The full offline default-sandbox regression log has 20 outer workspace target summaries, totalling 334 passed / 438 failed / 32 ignored with 12 failed targets. Its 23 raw emitted summaries total 352 / 466 / 33 only because they include three nested test-child summaries (17/27/1, 1/0/0, 0/1/0), which are not counted as independent outer tests. Its failure causality is not fully verified. Clippy `-D warnings` is RED (400 lib / 306 lib-test errors). Nothing is qualified as a whole, and the `f6fd556c` figures (334 / 468 / 33) are historical. G4 is integrated, but `start_phase_inner` still bails before transport, so its genuine assembly is unexercised. PR52 was last reported DRAFT and CONFLICTING at `b191b466`, with an old Ubuntu FAILURE and macOS CANCELLED. Root's mandatory SourceStop/Cancel findings, and the author/permission decisions on them, are unresolved. No current-head CI, real N1/N4/hook/auth/quota recovery, Native first-Task, four-Task, both-OS, install or dogfood run is qualified.
4. **Never authority here:** public capability metadata or `probe`, configured aliases, scheduler/attention rows, Driver/Unit/marker SQL, test-only constructors, static fixtures, fake grants, boolean or optional availability switches, copied DB state, Native permission bypass, a Task.version-changing binder, a generic legacy binder fallback, or manufactured #19 proof. rrx is not a sandbox and makes no process-death claim. Official CLIs run on the host with the user's usual HOME, config, hooks and auth. Nothing here inspects, copies or transfers them, and no VM, container, separate user, root or outer sandbox is used.

### 1.1 One-line acceptance condition (P)

The real retained Runtime graph admits a trusted ready Task through its actual Driver/Source/marker/Native/binder producers while forged metadata/current SQL cannot replace installation or original input custody. This is an incremental prerequisite, not an MVP completion criterion.

## 2. Verified missing seams (V)

| Item | Fact at `f6fd556c` unless marked `7be25fec` (§1.2) |
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
| Native start (`7be25fec`) | `NativeSessions::start_phase_inner` (`execution/native.rs:208–215`) awaits `begin_phase_preparation`, binds the returned Prepared as `_prepared` and bails with "actual Prepared Native input retained; transport composition unavailable". `begin_phase_preparation` (`execution/native/preparation.rs:819–910`) returns `self.issue_prepared(&custody)`. `issue_prepared` (`native/prepared.rs:225–247`) checks the provider effect budget (`check_prepared`) and returns the SAME Arc from `custody.retain_prepared(..)`. The earlier "effect-budget qualification pending" bail is gone |
| G4 (`7be25fec`, integrated by `81061d32`) | `NativeSessions::start_prepared_transport(&self, &Arc<NativePreparationCustody>, Arc<PreparedNativePhase>) -> Result<NativePhaseStart>` (`execution/native/transport.rs:498`) has no caller. It registers, spawns into `NativeChildCell`, transfers into Core and returns `Launched` after first-poll acceptance (A: behaviour as reviewed by its owner). `retain_transport` (`native/preparation.rs:51–70`) requires the SAME retained Prepared. `NativePreparationCustody::request_stop` (`preparation.rs:48`) is `pub(crate)` and calls `abandon`. The transport custody's own `request_stop` (`transport.rs:94`) is `pub(super)`. Neither has a Root caller |
| Task reads (`7be25fec`) | `admit_task_driver` rereads the Task with `Store::task` (`runtime/task_driver.rs:23–29` → `state/mod.rs:288,1634–1641`), which transfers the whole `SELECT body` and decodes it with no bound. `plan_initial_driver` reads through `driver.rs::bounded` (`:83–95`): a length statement, then the body, inside one read-only snapshot transaction, so no over-bound body is transferred. `managed_binding/snapshot.rs:60–73` and `runtime/waiting.rs` already use the single-statement idiom `CASE WHEN length(CAST(body AS BLOB))<=bound THEN body END`. `validate_for` (`composition.rs:26–50`) serializes the Task with `serde_json::to_value` |
| Attention and Goal facts (`7be25fec`) | `reconcile_runtime_attention` (`state/runtime/service.rs:5–74`) CAS-writes the exact hold string H (`:36–38`) to every `scheduler_tasks` row whose attention differs. Goal acceptance inserts rows with NULL attention (`state/runtime/goals.rs:179`), and no other path updates `attention`. `runtime_goal_facts` (`goals.rs:329–337`) hardcodes `dispatch_available: false` and `UnavailableReason::NativeBindingUnavailable`. `UnavailableReason` (`runtime/control.rs:154–160`, serde snake_case) is only constructed, with no exhaustive match, and is never persisted. Rotations cannot decrease, `queue_sequence` is immutable, and `scheduler_tasks` and `task_drivers` rows cannot be deleted (`state/runtime/schema.sql:68–75,102,127–137`) |

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
//   admission_cursor: std::sync::Mutex<Option<CandidateKey>>, // §4.1 pass cursor; nongrant hint
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
- The value is built from `Arc::downgrade(self)`, `Arc::downgrade(&self.phases)`, `graph.sources`, `graph.engine`, `port` and a clone of the Task returned by the bounded current read (§4.3) under control admission.

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

The supervisor loop (`runtime/service.rs`) calls `runtime.admit_ready_tasks()` after `observe_task_drivers`, only while `service_running()`, and only in an iteration whose attention reconcile returned `more == false`, so reconcile paging never repeats a sweep. The result is the number of claims. Only a nonzero result counts as pending for `PhaseSupervisor::delay`.

```rust
impl Runtime {
    fn admit_ready_tasks(self: &Arc<Self>) -> Result<usize>;            // sync; loop-only; claims made
    fn admit_task_driver(self: &Arc<Self>, _admission: &tokio::sync::MutexGuard<'_, ()>,
        task: TaskId, composition: InstalledDriverComposition) -> Result<AdmitOutcome>; // was async
}
enum AdmitOutcome { Claimed, Skipped(SkipReason) }  // private; SkipReason names the refusing stage; nongrant
struct CandidateKey {         // exact SQL values as read; compared only by SQL; nongrant
    project_rotation: u64, project_id: String, goal_rotation: u64,
    goal_id: String, queue_sequence: u64, task_id: String,
}
enum CandidatePage { GlobalFull, Rows { keys: Vec<CandidateKey>, more: bool } }
impl Store {
    pub(crate) fn ready_driver_candidates(&self, instance: &str, epoch: u64,
        after: Option<&CandidateKey>, global_limit: usize, project_limit: usize)
        -> Result<CandidatePage>;                                       // read-only; nongrant
}
```

1. If `installed` is `Err`, return 0 without any read.
2. Call `control_admission.try_lock()`. If it is busy, return 0; the next wake retries. The loop never awaits control admission, because `shutdown` holds it while joining the loop.
3. **Read.** Load `cursor` from `admission_cursor` and call `ready_driver_candidates(after = cursor)`. This is one read-only statement in one read transaction, and it requires the current owner epoch. It first counts global distinct-Task occupancy with the SAME union that `claim_initial_driver` uses, and returns `GlobalFull` when that count reaches `global_limit`. Otherwise it returns at most 64 keys (with a 65-row sentinel) from `scheduler_tasks`. A row qualifies when its Task has no `task_drivers` or `execution_units` row, its Project's current occupancy is below `project_limit`, and its CURRENT key is greater than `after` by SQL row-value comparison. Rows are ordered by that key, ascending. The key is the scheduler design's order `(project rotation, project_id, goal rotation, goal_id, queue_sequence, task_id)`. Ranks and occupancy are read in this statement and never cached across a claim. The page grants nothing.
4. **Evaluate** keys in page order, with at most 32 evaluations per sweep in total. Each evaluation runs the bounded current Task read (§4.3), then `installed_driver_composition`, then `admit_task_driver`.
   - **Skip.** A refusal before `reserve_pending` returns `Skipped` and writes nothing. It can come from the bounded read, a missing port or declaration, `validate_for`, or a plan refusal (Goal not Running, an unmet hard prerequisite, existing history). Set `cursor` to this key and evaluate the next key.
   - **Claim.** On `Claimed`, set `cursor` to this key as read (its pre-claim value). DISCARD the rest of the page and go back to step 3. The claim advanced its Project's and Goal's rotations and occupancy, so the remaining keys may be out of rank order or may name a Project that is now saturated.
   - **Error.** An `Err` from `reserve_pending`, `claim_initial_driver` (including its rank CAS and capacity refusals), `bind_claim` or spawn ends the sweep. `cursor` moves past this key, so a failing candidate cannot stall the pass; it is evaluated again in the next pass. The existing claim and close paths own that result.
5. **End.**
   - `GlobalFull` ends the sweep and leaves `cursor` unchanged.
   - A page consumed with `more` reads the next page after `cursor` (step 3).
   - A read that returns no key ends the pass: `cursor` becomes `None` and the sweep ends. The next pass starts at the next sweep, never within this one.
   - Reaching 32 evaluations ends the sweep and keeps `cursor`.
   - In every case `cursor` is stored back into `admission_cursor`.

**Cursor and key contract.** A key is a value read at one instant. It is never authority, and only SQL compares keys. No Task's key ever decreases: rotations cannot decrease, `queue_sequence` is immutable and scheduler rows are retained (V, §2). A claim inserts a permanent `task_drivers` row. That removes the claimed Task from the candidate set and raises the current key of every other Task in its Project and Goal. Consequently:

- After each claim, the next selection uses the post-claim rotation and occupancy. The claimed Project's and Goal's other Tasks move later, so a Project or Goal of equal or lower rank is selected next. A Project that reaches `max_tasks_per_project` is filtered out by the read, while disjoint Projects that fit are still selected in the same sweep.
- Keys above `cursor` fall into three groups:
  - this pass's unevaluated candidates;
  - Tasks whose key a claim in their own Project or Goal raised past `cursor` (they are re-evaluated later in this pass);
  - new Tasks appended above `cursor`.

  Tasks created below `cursor`, refused Tasks and Tasks filtered out by Project saturation wait for the next pass.
- Each evaluation either removes its Task permanently (a claim) or moves `cursor` past the Task's current key. A refused Task re-enters above `cursor` only through a claim in its own Project or Goal. A pass over a finite candidate set therefore ends after finitely many sweeps.
- A Task that stays a candidate, and whose Project fits when the cursor reaches it, is evaluated at least once in that pass.
- The number of sweeps per pass is not a fixed function of the candidate count, because claims raise keys back above the cursor and new Tasks may be appended. This HOW asserts no ⌈N/32⌉ bound, no fixed selection-turn bound and no completion time.

**Cost and polling.**

- A sweep does at most 32 evaluations and at most 33 candidate reads: the first read plus one after each claim. 32 evaluations can never exhaust a 64-key page. Each read returns at most 65 rows, and the sweep has no await.
- Only claims make the result nonzero. A sweep that ends with zero claims neither wakes the loop nor counts as pending, whether it stopped at its evaluation budget, at `GlobalFull`, at busy admission or at the end of the pass. The next sweep waits for a control or Driver wake, or for `PhaseSupervisor::delay`.
- The existing `wake.notify_one()` after a spawn reruns the loop only after a claim, and each claim permanently removes one fresh Task from the candidate set.

`admit_task_driver` keeps every existing step and check: Task read, `validate_for`, plan, `reserve_pending`, `claim_initial_driver`, `bind_claim`, spawn and wake. Three things change:

- It borrows the caller's guard instead of awaiting a new one.
- Its Task read is the bounded reader of §4.3, which replaces `Store::task`.
- Its refusals before `reserve_pending` return `Skipped` instead of `Err`.

Its worker still acquires control admission itself before activation (A).

### 4.2 Readiness, quotas and refusals

- **Readiness.** The authority stays where it is today. The accepted definition digest, Project Registered, Goal Running, non-terminal Task, DAG membership, hard prerequisites and freshness are all checked in the plan and again in the claim Immediate (§2). The candidate read is only a prefilter.
- **Restart and preparation recovery.** `fresh` refuses a Task with any prior Unit, operation, Driver, Workflow or Context, and that Task keeps the attention hold. Pre-artifact recovery and the fresh-attempt restart producer stay unsupported (Driver producer §5).
- **Quotas.** G5 adds no counter or semaphore and pre-reserves nothing. Driver capacity is the claim's distinct-Task union against `global_max_sessions` and `max_tasks_per_project`. The candidate read repeats that union only as a nongrant prefilter (§4.1); the claim's Immediate count stays the authority. PhaseSupervisor slots (A), `NativeLimits` session caps (A) and G1's S6 provider admit-or-park quota are unchanged.
- **Attention.** See §4.4. No audit kind is added.

### 4.3 Bounded current Task reads

```rust
// state/runtime/driver/candidates.rs (new; private to state::runtime)
impl Store {
    pub(crate) fn current_task_bounded(&self, key: &CandidateKey) -> Result<Task>; // nongrant
}
```

The sweep (§4.1 step 4) and the reread in `admit_task_driver` both use this reader, and it is the only Task reader on the admission path. The `Store::task` call in `admit_task_driver` is removed. The reader runs ONE statement that returns at most one row:

```sql
SELECT t.id, t.version, t.project_id, t.goal_id,
       CASE WHEN typeof(t.body)='text' AND length(CAST(t.body AS BLOB))<=1048576 THEN t.body END
FROM tasks t JOIN scheduler_tasks s
  ON s.task_id=t.id AND s.goal_id=t.goal_id AND s.project_id=t.project_id
WHERE t.id=?1 AND s.goal_id=?2 AND s.project_id=?3 AND s.queue_sequence=?4
```

- **No row** is a refusal.
- **NULL body column.** A body over 1 MiB, or one that is not text, comes back as NULL. That is a refusal before the body is transferred, decoded, cloned or serialized.
- **Decode.** The body is decoded with the existing `decode`; a decode failure is a refusal.
- **Identity.** The decoded `id`, `version`, `project_id` and `goal_id` must equal the SAME row's index columns and the candidate key; otherwise it is a refusal. Index and body come from one row of one statement, so no second read can pair them inconsistently.
- **Refusal effect.** Every refusal returns `Skipped` with a ≤256 B nongrant diagnostic that holds no body bytes. It writes nothing, changes no attention and no authority, moves `cursor` past this key, and does not stop evaluation of the next candidate.
- **After the bound.** Only once this bound has passed does the Task reach the composition clone, the `serde_json::to_value` comparison in `validate_for` and the Driver row encoding (still ≤128 KiB).
- `plan_initial_driver` keeps its existing bounded snapshot read (§2), unchanged.

### 4.4 Attention and Goal facts

Attention is display only. The issuer, the plan, the claim, the Engine and Native start read none of the attention value, the driving predicate below or the Goal facts. At `7be25fec`, only reconcile, Goal status and tests read them (V).

1. **One hold value.** H is the exact string that `reconcile_runtime_attention` writes today. The implementation moves it into one private function that both writers use, so there is no second spelling.
2. **Clear on genuine admission only.** Inside `claim_initial_driver`'s Immediate transaction, after the `task_drivers` INSERT and both rotation CASes and before commit, run `UPDATE scheduler_tasks SET attention=NULL WHERE task_id=?1 AND goal_id=?2 AND project_id=?3 AND queue_sequence=?4 AND attention IS ?5` with `?5` = H.
   - It changes zero or one row, and zero is not an error.
   - Any value other than H is preserved.
   - This transaction exists only for a composition-validated claim. No SQL row, reconcile or status read ever clears H.
3. **Reconcile preserves driving Tasks and restores holds.** The reconcile read adds this predicate per row: `EXISTS(SELECT 1 FROM task_drivers d WHERE d.task_id=s.task_id AND d.goal_id=s.goal_id AND d.project_id=s.project_id AND d.owner_epoch=?epoch AND d.state='driving')`.
   - A driving row's attention is left unchanged: NULL after a genuine clear, H if the row was forged, or any other value.
   - Other rows keep today's H write, and its CAS gains the same predicate as `NOT EXISTS`.
   - When the Driver closes (`invalid`) or the owner epoch changes, the next reconcile restores H.
4. **Goal facts.** `runtime_goal_facts` stops hardcoding its attention and computes it in its existing read transaction.
   - If any Task of the Goal has a current-epoch `driving` row whose `scheduler_tasks.attention IS NULL`, it reports `attention: UnavailableReason::NativeContinuationUnavailable`. This is a new variant, with wire value `"native_continuation_unavailable"`. It means a genuine Driver claim exists for a first Executor, while continuation, later Executors, retries and Reviewer phases (PR-1 to PR-4) stay unavailable.
   - Otherwise it reports `NativeBindingUnavailable`, as today.
   - `dispatch_available` stays `false` in every case. It means full Workflow dispatch, and a first-Executor-only admission is not that.
   - No field is added, and `GoalTaskPage`/`TaskFacts` are unchanged.
   - A forged pair of rows (a driving row plus a NULL attention) changes only this display and never admission (C4).
5. **Consumers checked at `7be25fec`.**
   - **Enum.** It is only constructed (`runtime/control.rs:409–410`, `state/runtime/proposals.rs:200–201`, `state/runtime/goals.rs:336,426`), with no exhaustive match. It is never persisted and crosses only the same-binary control socket.
   - **CLI.** `main.rs` `print_control` (`:306–340`) and `print_decision` (`:158–171`) print facts verbatim. They keep `native_dispatch` listed as unavailable, `native_dispatch_available: false` and `complete: false`, all of which stay true.
   - **Tests.** The assertions in `runtime/tests.rs:635–637,740–748`, `tests/runtime_cli.rs:295–296` and `tests/goal_cli.rs:328,564` cover undeclared fixtures, which never claim, so they stay valid.
   - **Design docs.** The [runtime CLI design](runtime-cli-integration-design.md)'s requirement for `dispatch_available:false` and named holds is preserved. The implementing PR updates the master attention description; this HOW changes no master fact.

## 5. Driven Workflow consumer and phase support (P)

`step_driven_initial(&self, task, composition: &InstalledDriverComposition, lifetime)` takes the composition in place of its separate `sources` and `runtime` parameters, and `drive` passes `claim.composition()`. The step adds one private check:

```rust
fn preflight_installed_native(&self, c: &InstalledDriverComposition, task: &Task, phase: Phase)
    -> Result<Arc<NativePhasePort>>;
```

Before any namespace helper, Source capture or Store write, the check refuses unless all of these hold: `std::ptr::eq(self, c.engine().as_ref())`; the Weak Runtime upgrades and `composition_is_current` holds; the Task id equals the original Task's id and `task.executor == c.selected().alias()`; `Arc::ptr_eq(&self.registry.native_phase_port(alias)?, c.selected())`; and the phase is the first configured Executor with no prior Executor attempt. The current genuine Running reservation of this same attempt is not a prior attempt. Later Executors and retries belong to PR-2.

The check never calls `capabilities`, `probe` or `require_managed_native_binding_composed`. `PreparedInputAdmission` stays unadvertised: prepared HOW §10.3(c) only limits which adapters may advertise it and does not require advertising it. Driven lines 44 and 166 (at `7be25fec`) use the check. Line 57 (a Reviewer, or another non-Executor Native phase) is unchanged: it still refuses through `preflight_native_adapter` and the continuation bail before its own effects, and that refusal does not block the Task's earlier first Executor. `Runtime::reserve_source_handoff` is unchanged, because its only caller (`driven_initial.rs:182` at `7be25fec`) passes the port that this check has already proved pointer-equal to the composition's port.

**Old public standalone paths stay unsupported.** Legacy `step`, `preflight_native_adapter`, `prepare_agent`, `prepare_managed`, `NativeSessions::start` on a protected scope and `AgentAdapter::start_managed` keep their unconditional refusals. An Engine, Sources or Registry built through the public constructors cannot obtain a composition, even over the same owner, because the issuer exists only on `Runtime` and only for its own graph.

## 6. G1 → G4 connection (P)

```rust
// execution/native/preparation.rs — unchanged (V at 7be25fec, :819–910)
pub(super) async fn begin_phase_preparation(&self, launch: Arc<PhaseLaunchParts>,
    custody: Arc<NativePreparationCustody>) -> Result<Arc<PreparedNativePhase>>;
//   S0-S6, then `self.issue_prepared(&custody)` returns the retained Prepared Arc
// execution/native.rs — G5 replaces the one remaining bail (V at 7be25fec, :208–215)
async fn start_phase_inner(&self, launch: Arc<PhaseLaunchParts>,
    custody: Arc<NativePreparationCustody>) -> Result<NativePhaseStart> {
    let prepared = self.begin_phase_preparation(launch, custody.clone()).await?;
    self.start_prepared_transport(&custody, prepared).await
}
```

`issue_prepared` already checks the provider effect budget and returns the SAME Arc from `custody.retain_prepared(..)`. The earlier budget bail is gone (V at `7be25fec`). Today `start_phase_inner` binds that Arc as `_prepared` and refuses before transport. G5's only change here is to replace that trailing bail. `start_prepared_transport` (integrated by `81061d32`, no caller today) accepts the Arc only when `retain_transport` finds it pointer-equal to the custody's Prepared (V at `7be25fec`). There is no re-entry, no second `claim_start`, no rebuilt command and no other issuer. The `Launched` proof returns through the existing PhaseJobs record-only binder. Every error takes the existing `NativePhaseStartError` → `Failed`/`abandon` path (A, `phase_jobs.rs:275–290`), followed by G1 closure or Held. The bail is removed only as part of this connection.

## 7. Merge order and the approved-contract conflict (P / A)

The approved contracts forbid issuing composition before its producers exist. Prepared HOW §10.3(a) requires that "G1–G4 and RN-1 are implemented and independently reviewed", transport HOW §16 keeps G5 "Remain closed", and Source handoff §5 keeps the issuer closed "until the genuine whole composition is present and positively qualified". Merging a first-Executor-only partial issuer ahead of them would therefore be a real conflict. It would also leave two hazards: without RN-1, a refused start keeps `phase_open=1` and consumes distinct-Task capacity; without G3, `shutdown` cannot stop a spawned child in the SAME custody. G5 is therefore one source increment. It may be developed on an integration branch, and it merges only after all of the following:

1. G1-final: B1/B2 adopted and the source independently reviewed. The final net `50deb781..a096f83b` is integrated in `7be25fec` (§1.2). Its independent review is the G1 owner's record (A).
2. Root integration of G4: the final source net `0c6fe3c7..4f1bf463` (eight files) is integrated by `81061d32` (V). The first draft's `git diff --stat f6fd556c 7838a734` (26 files) compared divergent branch endpoints and was not the G4 integration patch.
3. RN-1.
4. G3, the Root caller of SAME-custody `request_stop`, which must follow Root's own decision on the unresolved SourceStop/Cancel findings. This HOW neither decides nor closes those findings.
5. Official qualification on the unmerged integration branch. The [marker dispatch](issue-43-original-source-marker-dispatch-design.md) §§5 and 7 require it before merge: the real official Claude and Codex Agents, the four-Task acceptance, and both macOS and Linux, together with N1/N4/H/Q under prepared HOW §13.4. It runs through the production constructor on that branch, with the official CLIs on the host using the user's usual HOME, config, hooks and auth (§1.4).

Only item 5 meets "positively qualified" in Source handoff §5 and the marker design's merge condition. The account-free protocol-fixture controls of §10 are wiring controls. They run first, on the same branch, and earn no qualification credit. This HOW does not amend the marker contract or lower any merge gate. Install and every other gate stay where their own contract places them. There is no circularity: those contracts forbid merging the issuer before it is qualified. They do not forbid building it on the unmerged integration branch, which is where qualification runs.

**Installation availability versus qualification.** Architectural availability means that the production `Runtime::new` + `start` reach §§3–6. Qualification is separate: it is the observed version, hook, auth and Native behaviour of the official CLIs on macOS and Linux (prepared HOW §13.4). Without a compatibility declaration in the user's config, production admits no Task. That declaration is the existing S1 input, not a switch. README, master, release and MVP claims stay unchanged until qualification.

## 8. Startup, state and stop failure table (P unless marked A)

| Point | Condition | Result |
|---|---|---|
| `Runtime::new` | invalid `Config` | Err (A) |
| `Runtime::new` | no Tokio, provider absent, executable unresolved, invalid declaration, IPC bind failure, Engine/Sources owner mismatch | `InstallationRefusal`; Runtime and controls usable; zero admission |
| sweep | `installed` is `Err`, stopping, or admission busy | return 0; no read or write |
| sweep | global occupancy at `global_max_sessions` | `GlobalFull`; return the claims so far; cursor unchanged; no write |
| sweep | bounded-read, composition, `validate_for` or plan refusal | `Skipped`; no write or attention change; cursor past the key; next candidate evaluated |
| sweep | claim committed | H cleared in the same transaction (§4.4); rest of the page discarded; re-read with post-claim ranks |
| sweep | reserve, claim or spawn refusal | end the sweep; cursor past the key; existing close-unactivated / Held paths (A) |
| worker | Weak Runtime gone, lifetime cancelled or revoked | existing refusal before the next effect (A) |
| driven step | unsupported phase or port mismatch | named refusal before its own effects |
| Native start | any S0–S7 refusal | G1 closure, then RN-1 or Held; job `Failed` |
| transport | G4 refusal or first-poll drop | G4 Held rules |
| shutdown | normal | `stopping` under admission, `stop_all`, `close_unmarked`; marked jobs reach SAME-custody `request_stop` through G3, otherwise stay Held (A until G3) |
| Runtime Drop | any | `stopping` and Driver revocation (A); graph freed after the last worker |

**Bounds.** Each Runtime has one graph. A sweep holds the control guard for at most 32 evaluations and 33 candidate reads of at most 65 rows each, with no await. Per-Task bounds are unchanged (A): Task body ≤1 MiB (now enforced in SQL before transfer on the admission path, §4.3), Driver metadata ≤128 KiB, fewer than 4096 Driver entries, and at most 128 pending phases, 128 jobs and 128 Native preparations. The cursor is a single key.

## 9. Impact analysis

| Changed / consumed | Consumers (V) | Handling |
|---|---|---|
| `Runtime::new` installs the graph | `cli/service.rs:32` and the five test files of §2 | Every Runtime built under Tokio now binds the owner IPC socket and runs the cleanup sweep. The supervisor, handoff and fence tests also build a registry or `NativeSessions` on the same owner, so the source increment enumerates them: runtime-module tests use the installed graph, and the others become explicitly asserted `InstallationRefusal` fixtures. Neither case gets a test-only constructor. Migrated fixtures, including the duplicate IPC-binding fixtures, keep their original assertions. ControlFixture (`/bin/true`, no declaration) is refused at the declaration conjunct, so its zero-Driver assertions still hold. It is the C2s smoke refusal, not the paired C2 control |
| `InstalledDriverComposition` relocation and issuer | `state/runtime/driver/claim.rs:27,91,313–316`; `runtime/task_driver.rs:5,31,46,71–74` | path re-exported; checks identical |
| `admit_task_driver` takes the guard, returns `AdmitOutcome` and rereads through §4.3 | only the new sweep | no other caller (V). Its `Store::task` call (`runtime/task_driver.rs:23–29`) is removed; `Store::task` itself and its other callers are unchanged |
| `step_driven_initial` signature | `runtime/task_driver.rs:104` | composition passed in place of Sources and Weak Runtime |
| `preflight_installed_native` | `workflow/driven_initial.rs:44,166` (at `7be25fec`) | line 57 and the legacy callers keep `preflight_native_adapter` |
| Attention transition (§4.4) | `claim_initial_driver` (clears H); `reconcile_runtime_attention` (driving predicate in its read and CAS) | genuinely claimed Tasks lose only H, other values are kept, and H is restored after closure or an epoch change |
| `UnavailableReason::NativeContinuationUnavailable`; `runtime_goal_facts` | the consumers of §4.4 item 5 | additive variant; `dispatch_available` stays false; the listed tests are unchanged |
| `current_task_bounded`, `ready_driver_candidates`, `CandidateKey`, `AdmitOutcome` | the sweep and `admit_task_driver` only | new private readers and types; no schema change |
| `start_phase_inner` | `execution/native.rs:208–215`; `transport.rs:498` (`start_prepared_transport`, no caller today) | the one remaining bail is replaced; `begin_phase_preparation` already returns the retained Prepared (V at `7be25fec`) |
| New constants | page 64/65, 32 evaluations per sweep, the pass cursor | local; not shared with any existing bound. The 1 MiB Task bound reuses the existing value |

**Not affected:** SQL schema, `SCHEMA_VERSION`, permits and triggers; Task/Workflow/Context write contracts and the record-only binder; quota policy; the public `AgentRegistry`/`WorkflowEngine`/Sources constructors, which keep their signatures and give no route to composition; user settings, credentials and hooks, which are never read.

## 10. Controls (P; none executed)

Genuine controls use only `Runtime::new` + `start`, Goals accepted through the real HumanIngress (ControlFixture same-UID peer with isolated Git), the service loop's own sweep, and Agent programs configured through `AgentConfig.command` + `compatibility`. Account-free positives may configure the existing protocol fixture (`execution/native/native_fixture.py`) as that external program. The fixture stands in only for the CLI process, so it qualifies rrx wiring and never Native behaviour, and it never satisfies §7 item 5. A control that cannot reach its producer is a **SETUP/UNVERIFIED** result and earns no credit. A compile failure is never a kill.

| ID | Control | Needs |
|---|---|---|
| C1 | Installed graph: two compositions share the SAME Engine, Registry and port Arcs, and no Agent process ran at construction (fixture marker absent) | G5 only |
| C2 | Paired declaration control: the protocol fixture configured as `AgentConfig.command` WITHOUT `compatibility`. Expect an issuer refusal, zero `task_drivers`, Unit and helper rows, and attention H. It is paired with C5: the SAME configured fixture executable, arguments, provider and config, differing ONLY in whether the declaration is present | G5 only |
| C2s | Smoke refusal: ControlFixture `/bin/true` without a declaration yields zero rows while Goal controls still answer. It is not the paired control and not evidence for the declaration conjunct | G5 only |
| C3 | Executor alias absent, a generic agent, or an invalid executable: issuer refusal or `InstallationRefusal`, while Goal controls still answer | G5 only |
| C4 | Forgery: an SQL-inserted `driving` row, an attention edit, a public generic adapter advertising `PreparedInputAdmission`, or a second Engine from public constructors yields no admission and no worker; legacy `step` still returns `ManagedBindingUnavailable` | G5 only |
| C5 | One ready Task, same fixture as C2 plus the declaration: exactly one claim and rotation +1, then Source prep, Issue/Worktree, first Executor reservation, handoff, marker, job and S0–S7 on the SAME allocation, issuing the SAME retained Prepared. There is no budget-refusal assertion, because that bail is gone at `7be25fec` | G5 |
| C6 | C5 continued: Prepared → registration → spawn → Core → `Launched` → record-only binder `Bound`, with Task.version unchanged by binding | G5 (G1-final and G4 are integrated at `7be25fec`) |
| C7 | A paused Goal, an unmet hard dependency and a prior-history Task are each skipped with zero writes, while a ready sibling is admitted | G5 only (the sibling reaches its claim) |
| C8 | Caps and rotation after each claim: (a) `global_max_sessions=1` admits one of two ready Tasks, and the next read returns `GlobalFull`; (b) under default caps, 4 Tasks in 2 Projects (2 Claude + 2 Codex fixtures) are claimed in ONE sweep in post-claim rank order, alternating Projects and then Goals; (c) with `max_tasks_per_project=1`, two ready Tasks in Project A (ranked first) and one in Project B, one sweep claims A's first Task and B's Task, filtering saturated A without ending the sweep | G5 |
| C9 | Shutdown during a sweep, and while admission is held, completes within 5 s with no claim after `stopping` | G5 only |
| C10 | Runtime Drop during Source prep: Weak probes show the Runtime freed, no new effect, and the graph freed after the worker ends | G5 only |
| C11 | A Reviewer or later Executor phase refuses before its own effects after the first Executor ran | PR-1 to reach it; SETUP until then |
| C12 | Stop after spawn reaches SAME-custody `request_stop` | G3; SETUP until then |
| C13 | Bounded Task read, paired: a Task whose body is exactly 1 MiB versus one byte over, otherwise identical, written by a raw-SQL negative fixture after acceptance. Further cases: a non-text body, an undecodable body, and a body whose id or version differs from its index row. Each over-bound or malformed Task is `Skipped` with no write and no attention change, while a ready sibling ranked after it is claimed in the same sweep. Reread variant: a runtime-module test holds admission, issues the composition from a bounded row, replaces the body with an over-bound one and calls `admit_task_driver`, which returns `Skipped` at the bounded-reread stage before `validate_for` and plan | G5; a trigger refusal of the raw edit is SETUP |
| C14 | Attention: after C5's claim, the claimed Task's attention is NULL, and its Goal reports `native_continuation_unavailable` with `dispatch_available:false`. A skipped sibling keeps H, and another Goal reports `native_binding_unavailable`. After the Driver closes unactivated, the next reconcile restores H. A forged `driving` row keeps H, and its Goal reports `native_binding_unavailable` | G5 |
| C15 | Pass progress: 40 Tasks of a paused Goal ranked ahead of one ready Task. The first sweep evaluates 32 and claims nothing, the second claims the ready Task, and a later empty read resets the cursor | G5 |

**Required compiled mutants.** Each must fail its named control. A compile or SETUP failure is never a kill.

- **Composition and graph**
  - the declaration conjunct removed (C2, with the paired fixture)
  - the port looked up in a fresh registry, or the graph rebuilt per sweep (C1)
  - the driven preflight reverted to `preflight_native_adapter`, or made to require a capability (C5)
  - the composition holding a strong `Arc<Runtime>` (C10)
  - `start_phase_inner` calling `issue_prepared` again or passing a non-retained Prepared (C6)
  - the first-Executor conjunct removed (C11, SETUP until PR-1)
- **Sweep**
  - the sweep using `lock().await` (C9)
  - a pre-reserve refusal ending the sweep instead of skipping (C7, C13)
  - the rest of a pre-claim page evaluated after a claim instead of re-reading (C8b)
  - the Project-saturation filter dropped from the read, or ending the sweep (C8c)
  - the cursor not stored between sweeps (C15)
- **Bounded read**
  - the SQL bound removed from `current_task_bounded` (C13: the over-bound Task is claimed)
  - its body/index identity conjunct removed (C13)
  - the `admit_task_driver` reread reverted to `Store::task` (C13 reread variant: the refusal moves from the bounded reread to `validate_for`)
- **Attention**
  - the claim's attention clear removed (C14)
  - the reconcile driving predicate removed (C14)
  - the clear moved into reconcile for any driving row (C14, forged row)
  - `runtime_goal_facts` reverted to its hardcoded attention (C14)

## 11. Later increments and consumer boundaries

A positive C5/C6 result is an intermediate wiring control. It is not qualification (§7 item 5), full composition, the full Workflow or the MVP. The following remain required, each at its named consumer:

- **PR-1:** post-initial Driver continuation (`driven_initial.rs:59`, and the offer observation at `:133–192`), consuming the binding, the terminal and RN-1 `phase_closed`.
- **PR-2:** later Executor, retry and Reviewer offers (`execution/workflow_source/native_handoff.rs:448–536`), which widen `preflight_installed_native` per actor.
- **PR-3:** a Source-owned `ResultSnapshot` and `ReviewerArtifactLease` at A (`execution/attempts.rs:622`). The Engine's `managed_snapshots` (`workflow.rs:560`) stay legacy-only.
- **PR-4:** artifact retention that treats open custody as a live dependency (`state/execution/artifacts.rs`).
- G3, RN-1, cross-epoch closure and the restart fresh-attempt producer.
- Install on both hosts, full regression and Clippy GREEN, CI and #16, each where its own contract places it. The real-Agent four-Task both-OS qualification with N1/N4/H/Q is also a merge gate of this increment (§7 item 5).

The existing approved contracts already cover G1 (prepared HOW), G4 (transport HOW), the claim and worker (Driver producer), the handoff and marker (Source and marker designs) and the binding (managed binding). This HOW adds only §§3–6 and their controls.
