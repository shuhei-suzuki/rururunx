# Issue 43: fixture migration to accepted ingress — inventory (step 5, facts only)

- **Status:** inventory only; no HOW decided, no source changed. Pinned at `8f66fa2`. Paths relative to `crates/rrx/`.
- **Failure mass:** 372 workspace tests panic with "Goal creation requires trusted control ingress", concentrated in 19 helper sites.
- **Precedent:** `66f170b` (`crate::runtime::accepted_goal_fixture`, in-crate only).

## 1. What accepted ingress can and cannot produce

| Item | Accepted route | Source |
| --- | --- | --- |
| Goal title/objective/criteria/constraints | `GoalDefinition` | `src/runtime/goal.rs:28` |
| Task title/acceptance/executor/reviewers/workflow/risk, DAG | `TaskDefinition`, `PlanDependency` | `src/runtime/goal.rs:38,50` |
| Quick workflow | only with `risk = R0` and `config.minimum_workflow = Quick` | `goal.rs:166`, `config.rs:166,191` |
| `Task.issue` | **none** (no production writer) | only test write `src/codex/ownership.rs:699` |
| arbitrary `worktree` / `feature/task` branch | **none**; only execution reserve / Driver preparation, branch `rrx/{task}/{unit}`, live Driver required | `state/execution.rs:894-897`, `driver/preparation.rs:215-217` |
| Tasks added / Goal edited after acceptance | **refused** | `state/mod.rs:426-487` |
| `WorktreeManager::create` for accepted Goals | **refused** | `state/runtime/driver.rs:454-457` |
| execution reserve / workflow transition on managed Tasks | needs live `task_drivers` row (`driving`, current epoch) | `driver.rs:320-337`, `execution.rs:783`, `state/mod.rs:713` |
| integration crate (`tests/*.rs`) | only `rrx::cli::service::serve` + `rrx::cli::client::request` (or the binary `serve` + `goal --plan`); `serve` starts the Runtime | `cli/service.rs:29`, `cli/client.rs:8` |

## 2. Sites

| Group | Site (failing tests) | Blocker beyond visibility |
| --- | --- | --- |
| A | `src/execution/results/tests.rs:33` (90), `src/state/execution/tests.rs:25` (19), `src/state/execution/native_results_tests.rs:167` (2) | plan-expressible; `Store::memory` → `RuntimeOwner`; execution then needs a live Driver |
| B | `src/workflow/tests.rs:490` (72) | bound fixture forges `worktree`/`feature/task`; `step` needs a Driver |
| B | `src/codex/ownership.rs:697` (53) | `issue`; `WorktreeManager` refused |
| B | `src/adapter.rs:1984` (12), `src/state/native_dispatch_tests.rs:24` (9), `src/adapter/grok/mod.rs:1672` (1) | forged worktree/branch or `WorktreeManager` |
| C | `tests/support/grok_fixture.rs:88` (39, shared with in-crate), `tests/adapter.rs:99` (19) | forged worktree/branch |
| C | `tests/git.rs:62` (18), `tests/context.rs:86` (16), `tests/state.rs:30` (14), `tests/project.rs:101` (4) | `issue`, `WorktreeManager`, Tasks added later |
| C | `tests/managed_tools.rs:97`, `tests/managed_docker.rs:106` (1 each) | serve + Driver; sibling Task into the plan |
| C | `tests/goal_graph.rs:115,226` (2) | exercise generic Goal DAG mutation / legacy rows, which ingress cannot express |

## 3. Open decisions (user, WHAT-level)

1. `Task.issue`: add to `TaskDefinition` (WHAT change), or remove from fixtures whose assertions do not depend on it.
2. Tests that need a worktree/branch or execution: drive them through a started Runtime and a live Driver (`rrx/{task}/{unit}`), as the SC1 control does — heavier and slower.
3. Tests of legacy-only behavior (generic Goal mutation, `WorktreeManager` on accepted Goals, legacy DAG history): convert into refusal controls on accepted Goals, or keep them on genuinely legacy (non-accepted) rows built through the historical migration path. Disabling/ignoring is excluded.
