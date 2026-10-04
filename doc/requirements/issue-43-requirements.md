# Issue 43: Preserve native authority during Workflow Session binding

Workflow: STRICT. Requirements pending independent review; implementation absent.

## Problem and MVP relationship

A successful Workflow native launch currently binds the returned Session by changing
only `history[active].session_id`, then persisting both Workflow and the unchanged
Task. The Task version increment invalidates native adapters' captured scope. A
still-running decision can become ineligible solely because the runtime registered
its own Session. Main `4851fcd` and Issue 41 `1418b36` have this behavior. Claude
public `1c098e1` atomically fences final decision currency by the captured Task version;
Codex and Grok also use scope snapshots. Fake launchers omit these native guards.

This is necessary for native adapters #5/#6/#7 to compose with Workflow #8/#41,
prepared-input ownership #19, Review #9 and dogfood #16. Existing #42 GitHub ownership
visibility is independent. Recovery #14 and external evidence reconciliation #13
retain their own acceptance criteria.

## Required behavior

1. Binding changes only the active Running native attempt's absent `session_id` to
   its exact persisted returned Session ID. Persist the Workflow record and scoped
   factual audit in one Immediate transaction. Preserve Task bytes and version.
2. Check exact captured Project, Goal, Task and Workflow versions, active Project/
   Goal/Task state, Workflow scope/generation/active attempt, committed dispatch marker,
   current context pointer and identity. Stale snapshots fail without DB/audit change.
3. Require exact expected native actor, role, Task worktree and owning scope. Require
   the persisted Session's immutable ID/scope/provider/actor/role/worktree to equal
   the trusted registered adapter's returned identity. If a native UUID was already
   returned, require it unchanged. A Starting return can legitimately precede PID,
   UUID, Running or terminal publication; binding reads the latest durable lifecycle
   in its transaction without treating mutable startup fields as an identity change.
   No PID/UUID hint grants process ownership or transport completion. Typed input's
   private allocation remains mandatory even when a public identity matches.
   Lost, foreign, malformed, ambiguous or missing Sessions cannot bind.
4. Preserve the original reservation after binding failure, including a still-running
   Session. No release, retry, ownership transfer, native turn or fabricated completion
   follows a failed binding transaction.
5. The binding primitive is private to the crate and narrowly validates the complete
   allowed Workflow delta. It cannot change other history, Task fields, context, actor,
   marker, completion, terminal decision or native outcome. Ordinary phase/context/
   state transitions retain their existing Task-write transaction.
6. For typed prepared input #19, bind only the exact allocated private Session owner
   for that scoped native attempt in the same transaction. A visible Workflow Session
   ID is not an allocation credential. Closure still checks that owner even when the
   public attempt's Session ID is absent. Issue 19 provides these typed-slot guards;
   this issue must retain and compose with them, without claiming unmerged coverage.
7. Native caller completion/callback eligibility remains unchanged by a valid binding.
   A real Task/Project/Goal/lock/currency change must retain its current refusal. No
   provider scope fence or native auth/hook/trust/permission control is weakened.
8. Update Workflow master design and verification. Independent immutable source review,
   meaningful controls/mutations, build/test/lint and exact-head CI precede merge.

## Impact and verification

Consumers: Workflow native launch binding, State persistence/audit, native scope
snapshots and private completion, decision/approval observation, #19 allocation and
closure, #41 pre-dispatch claims and terminal recovery. No schema change is required
solely for record-only binding. No broad Task-write optimization is introduced.

Use isolated temporary Projects/Git worktrees/SQLite databases. Hold an actual
registered adapter's native protocol turn across Workflow binding, then observe its
private completion/callback. Prove unchanged Task/version, one Workflow version advance,
one factual audit and valid exact binding. Retain observed native failures separately
from fake-protocol controls and avoid claiming real model acceptance from a fake.

Second SQLite writers change Task, Project, Goal and Workflow versions; invalid actor,
role, worktree, context, marker, Session identity/state and extra record deltas fail
without DB/audit change. Hold start before binding and preserve passive #41 polling.
Typed private-owner tests compose with #19 after its implementation is available.

Compile mutants restoring the Task rewrite, omitting each meaningful CAS/identity
boundary, or admitting additional Workflow changes. Assert causal real consumers fail,
restore exact bytes and pass controls. A merely mirrored helper test is insufficient.

## Limits

Arbitrary parent bookkeeping during a live turn is not authorized by this primitive.
Actual identity drift still requires explicit recovery; this issue does not guess
ownership or adopt PID/UUID hints. Restart, unknown dispatch,
explicit retry with unbound markers, Review Set slots, scheduler and external side
effects retain their separate issue contracts.
