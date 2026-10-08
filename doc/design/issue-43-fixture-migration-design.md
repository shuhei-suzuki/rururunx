# Issue 43: fixture migration to accepted ingress (FM) HOW — draft

- **Status:** revision R3, answering Sol 6049240790 (FM-M1, FM-M2, FM-M3, FM-L1; §8.5 decided as L). §8.2, §8.3, §8.5 and §8.6 are replaced, and §9 is new. Nothing is implemented.
- **Pin:** `cb5dddd`. Paths are relative to `crates/rrx/`. The inventory is `doc/design/issue-43-fixture-migration-inventory.md` (`415453f`).
- **Why:** Linux CI on PR #80 reports 418 passed / 284 failed / 20 ignored. All 284 failures are the trusted-ingress fixture refusals; no SC/BR control fails.
- **Constraints (STRICT, unchanged):**
  - no relaxed authority, CAS, owner, marker or #19 check;
  - no test-only authority constructor;
  - no SQL-seeded positive;
  - no disabled or ignored test;
  - no Task.version binding.

## 1. Shared fixture (in-crate)

1. Widen the existing accepted-ingress fixture (`src/runtime/tests.rs` `ControlFixture`, precedent `66f170b`) to `pub(crate)` constructors only. A Goal still comes only from `Runtime::handle_control(CreateGoal)` over the accepted Unix peer.
   - `ControlFixture::accepted(configure: impl FnOnce(&mut Config), plan: GoalPlan, git: bool) -> (Self, Vec<Task>)`
   - accessors `owner()`, `runtime()`, `store()`, `state_path()`
2. Workflow class and risk come from the plan plus `config.minimum_workflow` / `risk_mapping` (Quick = R0 + minimum Quick), as the installed controls already do.
3. Tests that execute (reserve, prepare, finish, step) start the Runtime and use the genuine Driver, like the SC1 lane. Worktree and branch are then the Driver-prepared `rrx/{task}/{unit}`. Assertions that named `feature/task` or a forged path are rewritten to the genuine values. No forged `worktree`/`branch` is written.

## 2. Integration crate (`tests/*.rs`)

Only the public route is used: `rrx::cli::service::serve(state, config)` on a temp state, then `rrx::cli::client::request(state, ControlAction::CreateGoal{..})`. The Project is registered first (`ProjectRegistry::add`) and the test's owner is dropped before `serve`, because `serve` opens its own owner.

## 3. Groups

| Group | Sites (failing tests) | Migration |
| --- | --- | --- |
| A — plan-expressible | `src/execution/results/tests.rs` (90), `src/state/execution/tests.rs` (19), `src/state/execution/native_results_tests.rs` (2) | §1; execution through the started Runtime and live Driver |
| B — forged worktree/branch | `src/workflow/tests.rs` (72), `src/adapter.rs` (12), `src/state/native_dispatch_tests.rs` (9), `src/adapter/grok/mod.rs` (1), `tests/support/grok_fixture.rs` (39), `tests/adapter.rs` (19) | §1/§2 with Driver-prepared worktree/branch; assertions use genuine names |
| I — `Task.issue` | `src/codex/ownership.rs` (53), `tests/state.rs` (14), `tests/context.rs` (16), `tests/project.rs` (4) | decision D1 |
| L — legacy-only behavior | `tests/git.rs` (18; legacy `WorktreeManager`, issue-numbered worktrees, Tasks added after creation), `tests/goal_graph.rs` (2; generic Goal DAG mutation, legacy history) | decision D3 |
| C — public route | `tests/managed_tools.rs`, `tests/managed_docker.rs` (1 each) | §2; a sibling Task moves into the plan |

## 4. Decisions (user, 2026-10-07)

| # | Decision |
| --- | --- |
| D1 | (b): drop `issue` from fixtures whose assertions do not depend on it. There is no WHAT change. A test whose subject is issue-based naming keeps `issue` on a legacy row through the legacy `put_task` writer (§7.1 S5). |
| D2 | Yes. Tests that must reach Native execution, the Driver or accepted-only writers use the started Runtime and live Driver. |
| D3 | Per-test classification by the §6.4 rule: L, D2 or R (§7.2). |
| D4 | (a): the 17 D2 tests blocked by S2 (verification 12, managed 5) become R now. Their positive scenarios are carried as SC-N continuation work, not as failing tests. |
| D5 | The S4 Goal-change halves are replaced by a Project or Task change wherever that keeps the test's subject; otherwise that half becomes R. On the accepted harness, a Goal pause or cancel uses the real `SetGoalLifecycle` control. |
| EF | The restart defect (EF HOW R1, `ccaef5f`) is fixed in #43, after Sol approves EF. |

## 5. Order and evidence

1. Groups are migrated in A → C → B → I → L order, in separate commits.
2. Each commit records its HEAD, the selected test names (never a zero-test run) and the unchanged STRICT guards.
3. The target is Linux CI with 0 failed; macOS CI stays on the user's machine.

## 6. Prototype evidence (R1)

The prototype is a scratch patch and is not committed. It swaps only the shared `src/execution/results/tests.rs` `fixture()` (used by 13 sibling modules) for genuinely legacy rows. Those rows are built through the ordered historical migration path from an accepted-ingress Goal, exactly as `66f170b` `ordered_format_migration_*` does: the historical schema, a copy of the non-grant tables only, `user_version = 2`, then `Store::open` migrates. No authority, Driver or execution row is copied, and no guard is changed.

### 6.1 Source facts

- **F1.** The public preparation route refuses accepted Goals. `AttemptManager::prepare`, `ManagedWorkflowSources::prepare` and `WorktreeManager::*` read `Store::legacy_worktree_task`, which refuses when a `goal_authority` row exists (`src/state/runtime/driver.rs:436-459`). Existing controls pin this refusal: `src/runtime/tests.rs` `actual_accepted_worktree_routes_refuse_before_valid_native_git_helpers` and `public_source_preparation_refuses_accepted_goal_before_unit_or_helper_effects`.

  Consequence: for group A, swapping in the accepted-ingress fixture alone cannot work. Each test that prepares a unit needs either:
  - legacy rows (D3(b), applied to group A); or
  - the started Runtime and live Driver (D2).
- **F2.** Native execution on legacy rows refuses with "managed native binding and private admission are not composed" (`src/workflow.rs:43`). Tests that reach that point need D2 whatever is chosen for F1.

### 6.2 Measurements (lib, Linux)

| Run | HEAD | Result |
| --- | --- | --- |
| Base | `5e6392d` | 418 passed / 284 failed / 20 ignored |
| Prototype (one fixture swap) | `5e6392d` + scratch patch | 459 passed / 243 failed / 20 ignored |

By test name: 41 newly pass and 0 newly fail. The 41 are:

| Module | Tests |
| --- | --- |
| `execution::native::tests` | 20 |
| `execution::results::tests` | 7 |
| `workflow::committed_source_tests` | 5 |
| `execution::ipc::tests` | 3 |
| `adapter::native::tests` | 2 |
| `execution::phase::tests` | 2 |
| `execution::cleanup::tests` | 1 |
| `execution::workflow_source::tests` | 1 |

The fixture's remaining users now fail later, past the ingress check:

| Failure | Count | Where | Meaning |
| --- | --- | --- | --- |
| F2 refusal | 24 | `workflow/verification_tests.rs:80` (12), `workflow_source/recovery_tests.rs:122` (7, the Implement `engine.step`), `workflow/committed_source_tests.rs` (4), `workflow/managed_tests.rs:626` (1) | the legacy Engine steps into a native phase; see §7 |
| `managed command failed (exit 128)` | 8 | `retained_tests/routing_tests.rs:74`, `managed_tests.rs:235,418,797,966`, `native/tests.rs:1925,2241` | prototype defect, not a product refusal: these tests hard-code `dir.path().join("repo")`; the prototype rooted the Project at `real-git-source`. The fix is to keep the Project root at `dir/repo`. |
| Docker probe unavailable | 7 | `execution/docker/tests.rs` | environment: this container has no Docker; the GitHub Linux runner does |

Correction to R1 as first pushed (`2e53a64`): `recovery_tests.rs:122` is F2, not F1, because `ManagedWorkflowSources::prepare` at `:113` passes on legacy rows. No test in `execution/native/tests.rs` hits F2.

### 6.3 Open question this raises (for D3 and review)

A test that passes on legacy rows covers the legacy row, not an accepted Goal. This is real coverage only where production still executes that path for migrated legacy Goals. Where the path is accepted-only in production, the test must use D2 instead; otherwise it gives false assurance.

The per-test classification is therefore part of the HOW review: legacy-reachable mechanics go on legacy rows, accepted-only paths go on the started Runtime and Driver.

### 6.4 Classification facts (for the D3 table)

- **F3.** The public `AttemptManager::prepare` / `prepare_snapshot` route has no production caller; only tests use it. Production preparation goes through two routes:
  - `ManagedWorkflowSources::prepare` → `prepare_workflow_source` (`execution/workflow_source.rs:619`), which refuses accepted Goals per F1;
  - the Driver routes `prepare_driver_source` and the workflow reservation.
- **F4.** Outside preparation, production code reads `goal_authority` only in these places:
  - Goal and Context writers: `state/mod.rs:452,1448`;
  - Runtime ingress, Driver and attention: `state/runtime/{goals,driver,service}.rs`;
  - the protected snapshot: `state/managed_binding/protection.rs:28`;
  - the schema table lists: `state/execution.rs:37`, `state/runtime/mod.rs:11`.

  `ResultStore` capture, publish, verify and snapshot, and the retained-inspection mechanics, do not consult it. So for these mechanics, the unit's origin (legacy row or Driver-prepared) changes only how the test reaches the unit, not the code under test.
- **F5.** The F2 refusal is intended and pinned: the installed-composition HOW C4 keeps legacy `step` returning `ManagedBindingUnavailable` (`doc/design/issue-43-native-installed-composition-design.md:317`). Tests that reach it on legacy rows must not relax it. Each such test either:
  - moves to the started Runtime and installed driven lane (D2); or
  - becomes a refusal assertion where its subject is the legacy step itself.

Proposed rule for the D3 table: mechanics that are origin-agnostic per F4 go on legacy rows; anything that reaches Native execution, the Driver, Workflow steps or accepted-only writers goes on D2.

## 7. Classification (R1, read-only analysis at `7362bf7`)

### 7.1 Further source facts

- **S1.** The public legacy `WorkflowEngine::initialize` / `step` has no production caller; only tests call it. Production uses `initialize_driven` and `step_driven_initial` (`runtime/task_driver.rs:117,126`).
- **S2.** The D2 driven lane reaches only:
  - the initial Evidence phases;
  - the first Executor phase;
  - its settled closure.

  The next phase returns "typed Driver continuation unavailable (SC-N)" (`workflow/driven_settled.rs:29-49`, `driven_initial.rs:154`).
  - D2 runs only through the installed `NativePhasePort`, and `allocate` accepts only `claude|codex` (`adapter/native.rs:143`).
  - Commit, Tests, Review, PR, MergeGate, Cleanup, escalation and retry have no D2 target today.
- **S3.** The standalone Native entry refuses protected Tasks: `NativeSessions::start` returns `AuthorityUnavailable` when `managed_phase_required` (`execution/native.rs:278-285`; `managed_binding/protection.rs:77`). `execution::native` and `adapter::native` tests can therefore pass only on legacy rows.
- **S4.** `put_goal` refuses any change on every row (`state/mod.rs:438-441`). `SetGoalLifecycle` requires `goal_authority` (`state/runtime/goals.rs:227-231`). Consequences:
  - a Goal pause or cancel is impossible on legacy rows;
  - a Goal objective, constraints or blockers change is impossible on any rows.
- **S5.** On legacy rows the generic `put_task` still writes (`state/mod.rs:453-509`), so `issue`, a binding and sibling Tasks are expressible there through the legacy writer.

### 7.2 Table (approximate counts; split tests by name in §7.3)

Class key:
- **L:** legacy rows.
- **D2:** accepted Goal plus the started Runtime and Driver.
- **R:** refusal assertion.
- **I:** depends on D1.

| Module | n | Class | Reason |
| --- | --- | --- | --- |
| `codex::session::tests` (+ `custody_mechanics`), `codex::ownership::tests` | 53 | L + I (drop `issue`) | ScopeSnapshot, approval and custody mechanics. No assertion reads `issue`. `WorktreeManager::create` works on legacy rows. |
| `adapter::grok::{environment,reader,fixture_support,receipt_support}` | 40 | L | Fixture `tests/support/grok_fixture.rs:51-88` forges a worktree. Use `WorktreeManager::create` on the legacy Task instead. Counts include isolated child re-runs. |
| `adapter::git_owner::tests` | 9 | L | GenericCliAdapter owner pool. 4 direct tests plus meta-test child re-runs. |
| `state::execution::tests`, `native_results_tests`, `native_dispatch_tests` | 30 | L (3 split by S4) | Ledger and CAS mechanics. `driver::validate` returns Ok for unmanaged Tasks (`driver.rs:321-324`). |
| `execution::{native,results,ipc,phase,cleanup,docker}`, `adapter::native` | ≈52 | L (only option, S3) | 41 already pass in the prototype. The rest are the exit-128 prototype defect and Docker. |
| `workflow::tests` | 64 | 4 L, 4 split, **56 R** | Step into Implement or later on the legacy Engine, which hits F2 (`workflow.rs:1598`). S1: no production caller. S2: no D2 target. FakeAgent cannot run on D2. `native_preflight_tests.rs:198` already pins the refusal. |
| `workflow::tests::unbound_retry` | 8 | R | `ready_agent` steps into a native phase, which hits F2. |
| `adapter::grok::environment_workflow_tests` | 2 | R | The legacy Engine with Grok steps into Implement, which hits F2. Grok is not an installed provider. |
| `workflow::committed_source_tests` | 9 | 5 L, 4 D2 | The 5 cover initial Evidence and gates. The 4 step into Implement; 301 then asserts Commit and Tests, which is past S2. |
| `workflow::committed_source_tests::verification_tests` | 12 | D2, **blocked by S2** | `publish()` needs native Implement, then Commit and Tests. |
| `workflow::managed_tests` | 5 | D2, partly blocked by S2 | Steps into a native phase. 511 reaches Review. |
| `execution::workflow_source::{recovery_tests,tests}` | 9 | D2 (8) / L (1) | The Implement step hits F2. First-Executor adoption is inside the lane. |
| `adapter::tests` | 4 | 2 L, 2 split by S4 | |
| `runtime::phase_supervisor::tests` | 7 | uncertain | Already accepted. F1 at `:225-228` (public prepare). The subject is the test-only `reserve_pending_phase`, and Native start is forbidden. A live Driver would go past preparation. |

Approximate tally:
- **L:** ≈175.
- **R:** ≈66.
- **D2:** ≈36, of which 17 are blocked by S2 today.
- **Uncertain:** 7.

### 7.3 Tests blocked by S4 (Goal change)

These tests need a split: replace the Goal change with a Project or Task change, or turn the Goal half into a refusal.

- `workflow::tests`: 1097 (pause), 3818 (cancel), 2083 (constraints; also `issue`, a sibling Task and the forged `feature/fresh-task`), and the `set_goal` helper.
- `adapter::tests`: 2090 (objective), 2177 (pause).
- `state::execution::tests`: 1343 (pause and resume), 1604 (constraints and blockers).
- `state::native_dispatch_tests`: 46 (objective).
- `codex::session::tests`: `session.rs:4459` (pause).
- `committed_source_tests`: 631.
- `verification_tests`: 576.

### 7.4 Decisions this adds (user)

- **D3 (sharpened):** about 66 legacy-Engine tests become refusal assertions (R). This drops their positive coverage of legacy-only phases that production never runs (S1). Managed equivalents beyond S2 do not exist until the typed continuation (SC-N) lands.
- **D4:** the 17 D2 tests blocked by S2:
  - (a) convert to R now; or
  - (b) keep them and carry the remaining positive scenarios into the SC-N continuation work.

  Ignoring or disabling is excluded, so (b) needs another home for these tests before Linux CI can be green.
- **D5:** the S4 Goal-change halves (§7.3): replace the Goal change with a Project or Task change, or turn that half into a refusal.
- **Note:** an existing SQL-seeded historical builder exists at `execution/native/tests/terminal_lock_tests.rs:11-128` ("old9", raw `INSERT`s, then `Store::open` migrates). FM does not use it: the "no SQL-seeded positive" rule.

## 8. HOW (R3)

### 8.1 L: legacy-row harness

Applies to about 175 tests.

1. **The fixture.** `crate::runtime::legacy_goal_fixture(seed, plan) -> LegacyFixture` is `cfg(test)`, `pub(crate)`, in `src/runtime/tests.rs`. It works in three steps:
   1. It creates the Goal through `ControlFixture` accepted Unix-peer ingress, with no authority constructor.
   2. It runs the ordered historical migration, factored out of `workflow/tests.rs` `ordered_format_migration_*` (which then calls the helper):
      - the historical `state/schema.sql`;
      - `INSERT … SELECT` of the non-grant tables only (`projects`, `goals`, `tasks`, `records`, `context_versions`, `usage`, `audit`) from the accepted-ingress database;
      - `user_version = 2`;
      - `Store::open` migrates.
   3. It opens a fresh `RuntimeOwner` on the migrated file.

   The rows are produced by genuine ingress and only copied, so this is not an SQL-seeded positive. The authority, scheduler, Driver and execution tables are never copied; a post-condition asserts that they are empty.
2. **`LegacyFixture` fields:** `{ holder, owner, task(s), root }`, with `path()`. The Project root is `<dir>/repo`, because tests hard-code it (§6.2 exit-128 defect). `seed(&root)` makes the initial commits before Goal creation. The five call sites typed `tempfile::TempDir` change to `LegacyFixture`.
3. **Shape.** Sibling Tasks are expressed in the plan. Legacy `put_task` is used only where post-creation Task addition on a legacy row is itself the subject (S5).
4. **Worktree and branch.** These come from `WorktreeManager::create` on the legacy Task. Forged `git worktree add -b feature/task` (grok fixture) and forged bindings are removed. Assertions on names are rewritten to the genuine values.
5. **`issue` (D1).** It is removed from the codex fixture (`codex/ownership.rs:699`). Issue-naming subjects (`tests/git.rs`) set it on the legacy row through legacy `put_task`.
6. **`Store::memory` sites** (`adapter.rs`, `state::execution::tests`, `native_dispatch_tests`) move to the file-backed migrated store.
7. **Integration crate.** `tests/support/legacy.rs` does the same through the public route only:
   1. `ProjectRegistry::add`;
   2. `rrx::cli::service::serve`;
   3. `rrx::cli::client::request(CreateGoal)`;
   4. shutdown;
   5. the same ordered-migration copy, with `rusqlite` and `include_str!` of the historical schema.

   This applies to `tests/{adapter,git,context,state,project}.rs`.

### 8.2 D2: per named case (FM-M1)

D2 is used only where the D2 lane really reaches the test's consumer. §9 lists every named case and subcase with its producer→consumer path, its legitimate drift and its R and deferred parts.

What is never used as a solution:
- marker removal;
- opening the generic Task writer;
- SQL grants;
- Task.version binding changes;
- new test-only authority constructors;
- a mid-flight `begin_execution_epoch` (a startup-only seam).

Legitimate drift on D2 (verified at `38e4f56`):
- **Project registry changes** (name, `rule_refs`, `config_ref`, `environment_refs`): `put_project` has no accepted-Goal check. Root and identity are refused (`state/mod.rs:351-366`). These fields feed `governing_digest` (`state/execution.rs:498-511`).
- **Goal lifecycle** through `ControlAction::SetGoalLifecycle` on live ingress (`state/runtime/goals.rs:358`).
- **Filesystem changes** in the worktree, in the retained manifest and in live rules.
- **A real restart:** shutdown, then `RuntimeOwner::open`.
- **Future drop or abort.**

Not legitimate:
- `put_task` and `put_goal` on accepted rows (`state/mod.rs:418-491`);
- forged transports;
- `DROP TRIGGER`.

### 8.3 R: refusal assertions, one contract per distinct boundary (FM-M2)

| Boundary | Refused at | Typed refusal / writer contract | Preimage asserted unchanged |
| --- | --- | --- | --- |
| **F2:** legacy native step (`preflight_native_adapter`, `prepare_agent`, `prepare_managed`) | `workflow.rs:1598/1630/1815`, before source capture (`:1334-1340`) | `NativePreflightRefusal::ManagedBindingUnavailable` (`:539-544`). A scenario where `MissingReviewer` or `AdapterUnavailable` is selected first (`:1581`) asserts that refusal instead. | Task body and version, events, Units/records, `observed_git_outputs()`, adapter callback counter |
| **F1:** public preparation on accepted Goals | `state/runtime/driver.rs:454-457` | the existing message | as in `public_source_preparation_refuses_accepted_goal_before_unit_or_helper_effects` |
| **S4-W:** generic Goal writer | `state/mod.rs:418-445` | "Goal creation requires trusted control ingress" / "Goal changes require trusted typed control ingress" | Goal body and version, the input DTO, related Tasks, audit |
| **S5-W:** generic Task writer on accepted rows | `state/mod.rs:451-491` | "managed Task creation requires trusted graph control" / "managed Task changes require typed owned control/Workflow transaction" | Task body and version, the input DTO, audit |
| **RB:** recovery after restart | `state/runtime/goals.rs:437-445` (Pause then Resume on live ingress); the Driver re-plan is refused by `fresh()` (`state/runtime/driver/claim.rs:233-238`) | `UnavailableReason::FreshBootstrapRecoveryUnavailable` | Goal, Task and Workflow rows; no job; no link |

Each R test keeps its setup up to the refused call, takes the preimage immediately before that call, and asserts the typed refusal plus the unchanged preimage. A doc comment names the former positive subject and what replaces it:
- a deferred owner (SC-N or #14); or
- "none: retired path", where the consumer is the legacy adoption that is reachable only behind F2.

**`tests/goal_graph.rs` (2):** class S4-W. These tests do not use F1/F2. The old proofs — DAG validation, and hold/terminal on a historically invalid graph injected by SQL — are not reproduced by a writer-refusal proof. That loss is recorded in the doc comment. No new enum or SQL path is added.

### 8.4 S4 splits (D5)

Applies to the 12 tests listed in §7.3. In each:
- an owner-change or pause subject becomes a Project registration change or a Task-level change on the same row; or
- on the accepted harness, the subject uses `SetGoalLifecycle`; or
- where neither keeps the subject, that half becomes R.

### 8.5 `runtime::phase_supervisor::tests` (decided: L, all 7)

The reviewer decided all 7 are L, with no fallback to R.
- **Fixture:** several Projects and Tasks are created on one accepted-ingress database, migrated (§8.1), and then the Runtime is opened over the migrated file.
- **Allowed:** the real public `prepare` consumer and the real allocation/`PreparationGuard`.
- **Not added:** no test-only authority constructor.

What this L coverage does not prove:
- accepted Source, marker or Native eligibility;
- the one-active-Task-per-Project MVP scheduler, because a same-Project multi-allocation queue test is not counted as that proof.

### 8.6 Order and evidence (FM-M3, FM-L1)

1. **Commits, in this order:**
   1. L in-crate: the `results` fixture, then `codex`, `grok`, `git_owner`, `state`, `adapter`, `phase_supervisor`;
   2. L integration;
   3. D2 (§9);
   4. R (§8.3);
   5. S4 splits.
2. **Unchanged production authority (diff, not grep).**
   - For each commit: `git diff <fixed base>..HEAD --stat` and the full diff of every non-test path.
   - The allowed non-test changes are only the `cfg(test)` fixture visibility (`pub(crate)` on test modules).
   - Any change under `state/`, `runtime/`, `execution/`, `workflow*` outside `#[cfg(test)]` items stops the commit and goes to review.
   - The consumer checks are re-stated in the commit body: the Driver checks, protected Native refusal, owner identity, retained membership, effect admission and #19.
3. **Refusal boundaries.**
   - Every R test is mapped to one §8.3 row: its refusal point, typed refusal, preimage and observation.
   - `observed_git_outputs()` counts only completed outputs of the thread's Git wrapper. Each boundary therefore also asserts:
     - an adapter callback counter (`cfg(test)`, read-only) where an adapter is in reach;
     - a Unit/record count where a reservation is in reach.
4. **Compiled mutants**, one per distinct boundary and one per shared fixture, not repeated across all ~83 tests. Each must fail its named assertion and pass again when restored:
   - **Mutation:** a Task write moved before the F2 refusal. **Expected kill:** the preimage assertion.
   - **Mutation:** the F2 check replaced by an `AdapterUnavailable` return. **Expected kill:** the typed-refusal assertion.
   - **Mutation:** the S4-W refusal message bypassed by a Goal write. **Expected kill:** Goal body/version unchanged.
   - **Mutation:** RB Resume made to return `Running`. **Expected kill:** the typed refusal.
   - **Mutation:** the legacy fixture's migration copying `goal_authority`. **Expected kill:** the L post-condition, and the F1 path refuses.
   - **Mutation:** the D2 pending identity, capacity or publication retention broken (SC9-style). **Expected kill:** the D2 assertions of §9.

   After restoring, the commit records `git diff <fixed base> -- <mutated path>` (empty) and `git status --short` (clean).
5. **Per commit record:** the HEAD, the named tests and their counts (never a zero-test run).
6. **Target:** Linux `cargo test --workspace` with 0 failed. Docker is verified on the CI runner; macOS stays on the user's machine.
7. **Migration recipe (FM-L1).**
   - The factored helper keeps `application_id` and `user_version = 2`, as in `66f170b`.
   - **Required empty after migration:** `goal_authority`, `scheduler_projects`, `scheduler_goals`, `scheduler_tasks`, `task_drivers`, all `managed_*` and `native_*` tables, `execution_units`, `session_units`, `task_execution`, `source_recoveries`.
   - **Exempt from the emptiness check:** `runtime_epoch`, whose singleton the migration itself creates (`state/execution.rs:55`).
   - **Also verified:**
     - the copy source has not executed;
     - no Task has a worktree, so no `LegacyUnreconciled` Unit is created (`state/execution.rs:66`).

## 9. Per-case D2 / R / deferred table (FM-M1; read-only analysis at `38e4f56`)

Each row gives what the test proves today, its D2 part, its R part (with its §8.3 boundary) and what is deferred.

**`workflow_source/recovery_tests`.** All 11 subcases share these facts:
- **Producer:** the SC1 lane, with `config_ref`/`rule_refs` committed before `accept`.
- **Consumer:** `begin_retained_source_recovery` and the recovery port. They are test-only until #14 adds their consumer (`source_recovery.rs:61-62`, `workflow_source.rs:671-760`).
- **No Source7 row on D2:** `fresh()` refuses any `source_recoveries` row (`claim.rs:233-238`).
- **Classification:** the consumer is not production, so these are not counted as D2.

| Subcase | Proves today | D2 part | R part | Deferred |
| --- | --- | --- | --- | --- |
| `published_recovery_reopens_exact_frame…` | exact Published frame rebuilt from committed rules; Commit advances pins | the producer only (SC1 proves it) | RB; S5-W (its Task title edit) | #14 recovery; SC-N Commit |
| `source_claim_checks_helper_admission…` | exclusive claim; stale DTO helper refusal | — | RB; S5-W | #14 |
| `actual_recovery_future_drop…` | a future drop abandons only its own claim | — | RB | #14 |
| `final_recovery_acceptance…` [no epoch] | a Task edit after reads refuses install | — | RB; S5-W | #14 |
| `final_recovery_acceptance…` [epoch] | an epoch race refuses install | — | RB | #14 (flag: a mid-flight epoch has no legitimate producer even there) |
| `recovery_rejects_bad_manifest…` | corrupt manifest or DTO drift refuses before helpers | — | RB | #14 |
| `source_bound_inflight_helper…` [mode 0] | a Task edit during a helper leaves an Unknown receipt | — | RB; S5-W | #14 |
| `source_bound_inflight_helper…` [mode 1] | an epoch change during a helper is refused | — | RB | #14 (flag: as above) |
| `source_bound_inflight_helper…` [mode 2] | an aborted helper leaves an Unknown receipt | — | RB | #14 |
| `unpublished_ready_and_foreign…` [Ready] | a Ready artifact is never a source | — (SC8 covers Ready, not Published) | RB | #14 |
| `unpublished_ready_and_foreign…` [foreign ×3] | a foreign DTO is never source authority | — | RB | #14 |

**First adoption.** Six subcases, all with R part F2 and nothing deferred:
- `workflow_source::tests::first_adoption_refuses_changed_envelope_task_cas_and_unsettled_helper_atomically`: Envelope, TaskCas, Helper (×3);
- `committed_source_tests::actual_first_adoption_refuses_payload_drift_and_stale_task_cas_before_native` (`:970`): Payload, TaskCas, StoredContext (×3).

Their consumer `adopt_prepared_workflow_execution` (`state/execution.rs:636-728`) is reachable only through legacy `prepare_agent` behind F2 (`workflow.rs:1815`). That makes it a retired path, so the subject is lost and recorded as such. The D2 lane uses `first_executor_frame` instead (`driven_initial.rs:228-345`). An optional new D2 analog is noted, not counted: a Project change before reservation is refused by `same_sources` (`driven_initial.rs:319-322`).

**`committed_source_tests`:**

| Case | Proves today | D2 part | R part | Deferred |
| --- | --- | --- | --- | --- |
| `:301` [claude, codex] | gates, native result, Commit, Waiting on Tests, 3 gate receipts, reopen, survivor edit | Worktree gate and Implement closure, 2 `managed_workflow_gate_v1` receipts, reopen, survivor worktree edit plus `verify` | — | SC-N: Commit, the 3rd receipt, Waiting on Tests |
| `:470` [manifest] | a corrupted retained manifest leaves Implement Waiting, nothing Published | hold at `SETTLED_EVALUATION` or `SETTLED_CLOSURE`, corrupt the manifest, expect not Published and the Task unchanged | — | — |
| `:470` [terminal] | the gate rejects a forged transport `work` | — (no legitimate producer; SC7 covers the genuine non-success) | F2 | none: retired path (flag) |
| `:631` Quick [claude, codex] | live rules B ignored, committed A applies, same Unit adopted, wire payload equals Context, Published | the whole front half | claude tail: S4-W (Goal `objective`) | codex tail (`rule_refs` cleared → "instructions changed"): SC-N |
| `:631` Standard, Strict [×2 providers] | the same front half with Requirements | the Requirements bind, wire payload, committed policy | — | SC-N-adjacent evidence integration (SC6 shows the hold) |

**Integration crate:**
- `tests/managed_tools.rs` and `tests/managed_docker.rs` are D2 through `serve` plus the Driver. The configured compat fixture runs the Git or Docker calls inside the Native phase, because only it holds `RRX_PROCESS_COOKIE`.
- The `current_exe` shim target needs the same repoint as today.
- If this proves infeasible, implementation stops and the case is returned to review. There is no silent fallback.

**Totals (by subcase):**

| Class | Count | Members |
| --- | --- | --- |
| D2 | 11 | `:301` ×2, `:470`-manifest, `:631` fronts ×6, `managed_tools`, `managed_docker` |
| R | 25 | recovery ×11, adoption ×6, `:470`-terminal, `:631`-claude tail, plus the S5-W parts carried inside recovery rows |
| Deferred | 18 tails | SC-N ×3, evidence integration ×4, #14 ×11 |
