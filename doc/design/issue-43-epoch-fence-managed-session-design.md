# Issue 43: epoch fencing of a managed Native Session (EF) HOW

- **Status:** proposed STRICT HOW, revision R1. Nothing below is implemented on the branch.
- **Pin:** `5ad3ae3`. Paths are relative to `crates/rrx/src/`.
- **Scope decision pending:** the user decides whether EF lands in #43 or moves to #14 / #81.
- **Unchanged WHAT:** MB1–MB9, the SC HOW and the BR HOW. No new recovery is added: an open managed operation stays open and Held after restart; original-attempt recovery remains #14.

## 1. Verified defect

| # | Fact | Source |
| --- | --- | --- |
| F1 | Opening a new owner epoch runs `begin_execution_epoch`, which fences every non-terminal managed Session to `Lost` through `fence_epoch_sessions` → `write_record_tx` (generic records writer). | `state/execution.rs:532–590`; `state/execution/sessions.rs:139–175` |
| F2 | A Session allocated to a `managed_phase_owners` row is a protected record: the `binding_record_UPDATE` trigger refuses any write without an exact private permit ("private managed Record writer required"). | `state/managed_binding/schema.rs:317–330` |
| F3 | Therefore, with a phase Session still `Starting`/`Running` (for example bound and not settled), `RuntimeOwner::open` fails: the whole epoch transaction rolls back and no Runtime can start on that state. A closed success has an `Exited` Session (skipped), which is why no existing control saw it. | reproduced at `f70f22b` by SC7-R (uncommitted) |
| F4 | The same epoch transaction already uses the private permit manager for another protected table (`source_recovery::invalidate_epoch` → `write` with an `ExactRowMutation`). | `state/execution/source_recovery.rs:462–575` |

## 2. Design

1. `fence_epoch_sessions` receives the store's `PrivatePermitManager` (as `invalidate_epoch` already does).
2. For a Session allocated to a managed phase owner (`EXISTS managed_phase_owners WHERE allocated_session_id = id`), the existing `Lost` record is written under exactly one `ExactRowMutation` permit:
   - `records UPDATE`, old image = the current raw row, new image = the bumped `Lost` record from `checked_session_record`;
   - `with_exact_permit` + `ensure_consumed`.
3. A non-managed Session keeps the generic writer, unchanged.
4. **Not changed:**
   - The identity checks of `fence_epoch_sessions`, the Unit fencing (`native_effects_open = 0`, `Lost`/`WorkUnknown`, cleanup job) and the effects fence.
   - The managed operation, marker and Workflow rows: no link and no closure are written, so the phase stays open and Held (MB7 / #14).
   - No Session projection trigger is relaxed. The permit authorizes only the exact old → new row image.
5. **Nothing reconstructs:** the new Runtime builds no job, binding plan or continuation from rows (the existing behavior). EF adds none.

## 3. Prototype evidence (uncommitted, Linux)

- **Base:** `5ad3ae3` plus §2 only.
- **SC7-R (claude, codex): 2 passed.** The steps are:
  1. normal Bound;
  2. shutdown;
  3. drop the old owner custody;
  4. `RuntimeOwner::open` on the same state succeeds;
  5. a new Runtime starts and is woken for 6 s.

  Result: no new link, no reconstructed job, and the Task is unchanged.

## 4. Controls and mutants (P)

- **SC7-R:** as in §3 (both providers).
- **EF-N (negative):** a non-managed (legacy) Session still uses the generic writer and is fenced `Lost` (existing control set).
- **Mutants:**
  - Write the managed Session without the permit. `RuntimeOwner::open` fails and SC7-R fails at setup.
  - Fence the managed Session with a wrong old image. The permit refuses and the open fails.

## 5. Impact

| Changed | Consumers | Handling |
| --- | --- | --- |
| `sessions::fence_epoch_sessions` signature and managed branch | `begin_execution_epoch` | passes `self.binding_permits` |
| Tests | SC7-R | new |
