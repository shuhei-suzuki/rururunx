# Issue 43: epoch fencing of a managed Native Session (EF) HOW

- **Status:** proposed STRICT HOW, revision R2. It answers Sol EF-M01 and EF-L01 (comment 6048588326): §2.6 and §4 are rewritten. Nothing below is implemented on the branch.
- **Pin:** `5ad3ae3`. Paths are relative to `crates/rrx/src/`.
- **Scope:** EF lands in #43 (user decision, 2026-10-07).
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
6. **One image, one write (EF-L01).** The managed branch never wraps the generic `write_record_tx`. A shared helper splits that writer into prepare and apply, and the generic branch keeps calling both halves unchanged.
   1. **One instant.** `at` is a single `now_ms()`, taken once per epoch transaction and shared by every managed Session in it.
   2. **Next record.** `next = checked_session_record(..)` is computed once: version bumped exactly once, `updated_at = at`, `body = serde_json::to_string(&next)` serialized once.
   3. **Old image.** The actual stored row, all seven columns (`id, kind, project_id, goal_id, task_id, version, body`), read with one `SELECT` inside the same transaction. It is never re-serialized.
   4. **New image.** `record_image(&next, &body)`, the same seven columns.
   5. **Write.** The `UPDATE` binds exactly that `body` and `next.version`, guarded by the old version and old body. It runs inside `with_exact_permit([records UPDATE old→new])`, followed by `ensure_consumed`.
   6. **Audit parity.** The `session.saved` event is appended with the same evidence fields as the generic writer, and `execution.session_epoch_lost` follows unchanged.

## 3. Prototype evidence (uncommitted, Linux)

- **Base:** `5ad3ae3` plus §2 only.
- **SC7-R (claude, codex): 2 passed.** The steps are:
  1. normal Bound;
  2. shutdown;
  3. drop the old owner custody;
  4. `RuntimeOwner::open` on the same state succeeds;
  5. a new Runtime starts and is woken for 6 s.

  Result: no new link, no reconstructed job, and the Task is unchanged.

## 4. Controls and mutants (R2, EF-M01)

Each control is a positive/negative pair on genuine state: an accepted Goal, the installed lane, a genuinely bound managed Session (SC lane). There is no SQL-seeded Session and no test-only authority. Assertions read the DB through existing readers.

1. **EF1 — managed Lost postimage (claude, codex).**
   - **Setup:** normal Bound (Session `Starting` or `Running`), then shutdown, then a new `RuntimeOwner::open`.
   - **The Session:**
     - the Session record is `Lost`;
     - its version is exactly +1;
     - `updated_at` equals the epoch's `at`;
     - its immutable identity (`id`, scope, provider, agent, role, worktree, model, effort) and `native_ref` are unchanged.
   - **Projections:**
     - `session_units.dispatch_state = 'unknown'`;
     - `scoped_session_identities` still matches the record's identity source.
   - **Events:** one `session.saved` and one `execution.session_epoch_lost`.
2. **EF2 — same-transaction Unit fence and no new authority.** In the same epoch:
   - **The Unit:**
     - `native_effects_open = 0`;
     - `WorkUnknown`/`Lost` as the existing predicate decides;
     - generation advanced;
     - the cleanup job present;
     - pending effects become `Unknown`;
     - quota released per the existing rule.
   - **Unchanged rows:** the managed operation, marker, Workflow, private links and the Task stay byte-identical.
   - **Nothing new:** the new Runtime, woken 6 s, holds no job, plan, Native authority or link (SC7-R).
3. **EF3 — negatives with a preceding write (all-or-nothing).**
   - **noPermit / wrongOld:** a mutant writes with no permit, or with an old image differing by one byte. A preceding legacy Session in the same epoch is ordered first, so a partial write would be visible.
   - **Expected:** the open fails.
   - **Whole-state equality before/after, for all of:**
     - Session rows (managed and legacy);
     - `session_units` and identities;
     - Units, cleanup and effects;
     - audit and events;
     - `runtime_epoch`.
   - **Permit hygiene:**
     - after the failure, no permit remains;
     - a direct generic `UPDATE` of the protected record is still refused.
4. **EF4 — contrasts.**
   - **Legacy positive:** a non-managed Session is fenced `Lost` through the generic writer; the existing control is kept.
   - **Managed terminal skip:** an `Exited`/`Stopped`/`Failed` managed Session is untouched. This is the existing `session_terminal` predicate (`state/mod.rs:2013`), asserted as is.
   - **Lost is not skipped:** a second restart re-fences an already `Lost` managed Session. The write is `Lost`→`Lost`, version +1, under the same exact permit. This matches the legacy generic writer, so no new skip rule is introduced.
   - **First open on empty state** succeeds.
5. **Compiled mutants mapped to named assertions.**
   - **M-skip:** `continue` instead of fencing a managed Session. Expected kill: EF1's `Lost` state and version assertions, and the projection assertion.
   - **M-nopermit / M-wrongold:** expected kill by EF3.
   - **M-doublebump:** wrap the generic writer around the prepared record. Expected kill: EF1's version-exactly+1 and `updated_at` assertions, and the permit's exact new image.
   - **Procedure:** each mutant is applied, the control is shown to fail at the named line, and the source is restored and shown passing.

## 5. Impact

| Changed | Consumers | Handling |
| --- | --- | --- |
| `sessions::fence_epoch_sessions` signature and managed branch | `begin_execution_epoch` | passes `self.binding_permits` |
| Tests | SC7-R | new |
