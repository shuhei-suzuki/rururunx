# Issue 41: owned preparation errors and passive Workflow observation

Risk: STRICT. Depends on Issue 8. No schema or public API changes.
Requirements: `doc/requirements/issue-41-requirements.md`.

## Ownership and observation

A phase reservation commits Task, Workflow and ContextVersion atomically. Missing
Session/dispatch evidence does not distinguish a suspended owner from a crashed
one. For an active nonterminal Task, ordinary `poll` therefore returns Waiting for an agent Running attempt with
no Session. Before its marker: preparation may still be active; after its marker:
launch may still be active or require reconciliation. Neither branch persists,
refreshes owners, clears an active pointer or invokes an adapter. The EvidencePort
branch continues to call its existing transactional evaluation path.

Reservation ownership is invocation-local proof. Create a private token only
AFTER this invocation's `reserve` returned success and AFTER the EvidencePort
branch. The token captures exact Task scope, committed Workflow Record ID/version,
generation, active history index, context pointer and serialized original attempt.
A reservation CAS loser or unknown commit outcome never creates a token. The
committed Record version distinguishes owners even if their proposed attempt
fields and timestamps coincide. No new persisted nonce or lease is introduced.

## Preparation error boundary

Keep the agent's post-reservation preparation/dispatch in a private method, with
an explicit release-eligibility state. Ordinary capture/config/serialization or
owner-refresh errors before attempting marker publication are eligible. Before
publishing a definitive Failed/Invalidated decision, disable release: a failed
publication must not become a retry. Before marker publication, disable release.
A typed `StateGuardError::SnapshotChanged` from that transaction proves rollback
before commit and may restore eligibility; arbitrary database/commit errors do
not. After a successful marker commit or any call to `adapter.start`, no error is
eligible. Native side effects and Session acknowledgements keep their existing
conservative behavior and are reconciled through Issue 14.

On an eligible returned error, synchronously re-read the owning Workflow and
Task. Require exact scope/Record ID AND Record version equal to the token, exact
generation/active index/context, and byte-equivalent original Running attempt,
with no Session/dispatch marker. Require active Project/Goal/nonterminal Task.
Any mismatch, lifecycle change, Session/executor fence or failed release CAS
retains the reservation. Inactive owner checks occur before any write. Pause then
resume by itself leaves that held reservation for explicit recovery in Issue 14.

For a proven current active claim only, set that attempt Failed, timestamp/detail
it with the returned preparation error, append a factual RetryEvent for the same
index and clear `active`. Keep every freshly read Task metadata field, Task phase,
ContextVersion, source/evidence, decision and unrelated history unchanged. Persist
through existing `put_workflow_transition` StateOnly transaction, which fences
Project/Goal versions, Task/Workflow CAS, context and all executor/Lost Sessions.
A second writer between re-read and release still wins its CAS; no overwrite or
retry of the release. Return the original preparation error, augmented with a
retained-reservation diagnostic if release fails. The next ordinary step may
reserve again only after this successful owner-local no-dispatch release.

Process crash, dropped owner futures, inactive-owner failure, conflicting Workflow
records and post-dispatch Session-binding failures are intentionally conservative
and depend on #14. A committed terminal decision may still use existing explicit
TerminalRecovery: the terminal-Task transaction fence and no Session/dispatch
marker exclude the suspended owner from future dispatch, while executor/Lost
fences remain mandatory. This path needs no inferred owner absence and introduces
no new terminal-recovery behavior. No PID hints, process death inference, timeout changes, trust
changes or automatic native replay are introduced.

## Consumer verification

- Hold both real post-reservation source capture awaits separately and hold the
  actual `adapter.start` await. A same-Engine and fresh-Engine observer each
  returns Waiting; Task/Workflow bodies and versions, context head and audit stay
  identical. Release the owner and observe exactly one adapter launch/Session.
- Drop a held owner future. A fresh Engine keeps the orphan unchanged. Native
  dispatch never happened; passive observation alone still cannot release it.
- Two owners read the same pre-reservation state; hold both pre-reservation
  captures, let one commit and suspend after reserve, then run the losing CAS.
  Its error must not alter the winner's claim; the winner launches exactly once.
- Inject an actual post-reservation capture error and the existing final-marker
  Task-metadata CAS race. Owner-local release preserves metadata and permits the
  next step. The EvidencePort half must preserve active index, history length,
  retries and context pointer, then complete the same attempt; strengthen its
  assertions without changing its existing evaluation path.
- Pause and cancel at each held preparation capture. After owner resumes, there
  is no native launch and no release write. Pause/resume remains held. Record the
  expected lifecycle/decision change independently of observation. For each held
  capture, cancel then explicitly TerminalRecover before resuming the owner; no
  owner input/write occurs and Interrupted history/decision stay unchanged.
- Inject a second-writer conflict on release, a definitive decision publication
  conflict, and a Session-binding conflict after held adapter start. Each retains
  the exact reservation and concurrent metadata; dispatched cases send no replay.
- Compiled mutants restoring premature observer reset, observer writes, omitted
  owner-error release, reserve-loser release and agent release incorrectly applied
  to EvidencePort must fail consumer assertions;
  restore byte-exact source and repeat original tests.

All tests use synthetic domain/adapters, owned temporary SQLite/Git fixtures and
bounded synchronization. Run full Workflow/shared-state regressions, formatting,
strict Clippy, debug/release build and exact-head Linux/macOS CI. Independent
immutable source review follows verification; no current-head green claim is
inferred from earlier checks or constrained-concurrency diagnostics.

## Impact

`WorkflowEngine::step/poll`, shared Workflow reservation consumers, Task metadata
CAS and StateOnly Store transition validation. Review (#9), Goal/Scheduler
(#14/#23/#24/#27) classify these waits using durable actor/state/Session/dispatch
fields, never text parsing. External irreversible gate observation/reconciliation
remains #13. Requirements, this design and master Workflow design change together.

Master Workflow and README references align approval with #10, verification with
#12, external irreversible reconciliation with #13, and preparation/native restart
recovery with #14. Update the private post-start acknowledgement comment to #14
without changing its behavior; pending unrelated Issue19 owned-ack integration
will merge normally. Issue14 now lists the retained classes and trusted-proof
acceptance explicitly; Issue41 stores no new durable owner-absence evidence.

Requirements gate: public42462bce7fba83b9b104afd27e4d65ea8f66824a native
requirements review e56f04e7-8eb1-41e5-84ac-036f3f4bef5b approved with no blockers.
Optional liveness refinement is deliberately deferred: a failed publication of a
definitive decision retains its claim rather than changing that decision into an
automatic retry. README records this recovery dependency. Align master marker and
terminal-recovery wording with the actual proofs above. Unknown reversible
EvidencePort evaluation claims are owner/restart recovery14; irreversible
Pr/MergeGate/Cleanup outcome reconciliation remains13. Their observation reason
must allow an evaluator still being in flight and use durable phase/state.

The old final-claim native regression legitimately changes from error then
observer Invalidated then Started to owner error/release then next step Started.
Its EvidencePort case retains the original attempt/context, with stronger checks.
