# Issue 43: fixture migration to accepted ingress (FM) HOW — draft

- **Status:** draft, revision R0. Not submitted for review: three user decisions are pending (§4). Nothing is implemented.
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

## 4. Decisions pending (user)

- **D1 `Task.issue`:**
  - (a) add `issue` to `TaskDefinition` (a WHAT change); or
  - (b) drop `issue` from fixtures whose assertions do not depend on it. Recommended: (b), which needs no WHAT change.

  `issue` is consumed only by the legacy `WorktreeManager` naming (`src/git.rs:109`) and Driver namespace identity checks. Accepted Tasks always have `issue = None`.
- **D2:** execution tests use a started Runtime and the live Driver (§1.3). Recommended: yes.
- **D3 legacy-only behavior:**
  - (a) convert into refusal controls on accepted Goals (the legacy writer refuses), keeping the old positive behavior out of scope; or
  - (b) keep it on genuinely legacy rows built through the ordered historical migration path (precedent `66f170b` `ordered_format_migration_*`).

  Disabling or ignoring is excluded.

## 5. Order and evidence

1. Groups are migrated in A → C → B → I → L order, in separate commits.
2. Each commit records its HEAD, the selected test names (never a zero-test run) and the unchanged STRICT guards.
3. The target is Linux CI with 0 failed; macOS CI stays on the user's machine.
