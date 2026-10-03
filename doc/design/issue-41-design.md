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
generation, active history index, context pointer, reserved worktree/branch and
original attempt data from the Record/Task values returned by the successful
reserve call, without an intervening re-read or await. The latter already includes
the reserved actor. Before the marker, compare refreshed Task actor and binding
against this token; an executor/reviewer override is an eligible preparation
error, release under the original token and reserve anew with the new actor.
Session binding must compare the returned actor with the reserved attempt actor,
never only with a refreshed Task field. Assigned worktree/branch changes are
already rejected by Store::put_task; retain that immutable binding guard. An
unbound-to-bound update, where allowed, must also start with a fresh reservation.
A reservation CAS loser or unknown commit outcome never creates a token. The
committed Record version distinguishes owners even if their proposed attempt
fields and timestamps coincide. No new persisted nonce or lease is introduced.

## Preparation error boundary

Keep the agent's post-reservation preparation/dispatch in a private method, with
an explicit release-eligibility state. Ordinary capture/config/serialization or
owner-refresh errors before attempting marker publication are eligible. Keep
eligibility enabled through captures/config/pack construction inside both
pre-marker source-invalidation branches; in-memory changes do not publish a
decision. Disable eligibility immediately before each non-release persist call:
fail, invalidate, hold and marker. The private invalidation helper must expose
this call-site boundary to owned agent preparation, while its other callers keep
existing behavior. A failed definitive publication must not become a retry.
Only a typed `StateGuardError::SnapshotChanged` with table `tasks` AND the owning
Task ID from the marker transaction restores eligibility after pre-commit
rollback. Project/Goal/Record version conflicts retain the reservation for #14,
even if the latest owners are active. Definitive fail/invalidate/hold publication
errors never restore eligibility, typed or untyped; neither do arbitrary marker
database/commit errors. After a successful marker commit or any `adapter.start`, no error is
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
it with a fixed factual pre-dispatch error classification (no copied raw source
or agent output), append a factual RetryEvent for the same
index and clear `active`. Keep every freshly read Task metadata field, Task phase,
ContextVersion, source/evidence, decision and unrelated history unchanged. Persist
through existing `put_workflow_transition` StateOnly transaction, which fences
Project/Goal versions, Task/Workflow CAS, context and all executor/Lost Sessions.
A second writer between re-read and release still wins its CAS; no overwrite or
retry of the release. Return the original preparation error, augmented with a
retained-reservation diagnostic if release fails. The next ordinary step may
reserve again only after this successful owner-local no-dispatch release.

Process crash, dropped owner futures, inactive-owner failure, conflicting Workflow
records, unknown/untyped marker publication outcomes (including a marker without a
Session even when start was never called), release CAS conflicts/executor-Lost
fences, marker Project/Goal-version conflicts, and post-dispatch Session-binding
failures are intentionally conservative
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
  is no native launch and no release write. Resume after this observed inactivity
  remains held. Also pause, resume, then continue an owner at each main capture:
  before refresh, active refreshed owners deterministically return Started with
  one adapter.start, one bound Session, the same attempt and no Failed/RetryEvent;
  after
  refresh, marker Goal-version CAS fails and stays held. Neither ordering releases
  or re-reserves a claim merely because of lifecycle ABA. A mutant widening
  eligible marker errors to Project/Goal versions must fail. Record the
  expected lifecycle/decision change independently of observation. For each held
  capture, cancel then explicitly TerminalRecover before resuming the owner; no
  owner input/write occurs and Interrupted history/decision stay unchanged.
- Hold adapter.start, cancel, then assert TerminalRecovery refuses the dispatched
  unbound reservation and leaves Task/Record versions unchanged. Hold before the
  adapter persists any Session and assert its absence. Engine-only dispatch-fence
  removal is masked by the Store fence, without kill credit; removing both must
  fail this consumer. Also call the crate-private Store TerminalRecovery directly:
  a dispatched unbound attempt is rejected with unchanged Task/Record versions,
  independently killing Store-only fence removal. Resume start: exactly one start
  call, Session binding is rejected by overlapping terminal-Task and stale-version
  fences; do not attribute that rejection uniquely to either fence. No owner release
  or replay occurs.
- Hold both source-invalidation branches’ internal capture/pack awaits. Capture
  errors remain eligible until publication; active owners release with no dispatch.
  A second source edit inside invalidation also releases. Pause/cancel there
  prevents owner writes; fail/invalidation/hold publication conflicts remain held.
- Override executor/reviewer during each main held capture. No launch under the
  old actor claim; the next reservation uses the new actor. Public Store rejects
  assigned worktree/branch changes rather than admitting an impossible fixture.
- Inject a second-writer conflict on release, a definitive decision publication
  conflict, and a Session-binding conflict after held adapter start. For definitive
  publications, use Task metadata-only conflicts with the same Workflow Record
  version and active owners, so terminal/Record fences cannot mask an actual
  incorrect release. Removed disable or wrongly restored typed eligibility must
  be killed by detecting committed release, not differing error text. Each retains
  the exact reservation and concurrent metadata; dispatched cases send no replay.
- Use cfg(test) one-shot hooks at release's fresh-read/CAS boundary and a fixed
  attempt timestamp for owner-race tests. A metadata writer AFTER re-read must
  cause release CAS loss, preserve Running/current metadata and return a retained
  diagnostic. A retry-on-release-CAS-loss mutant must fail this actual consumer.
- Re-persist the exact Workflow body through StateOnly while an owner is held;
  its attempt stays byte-identical but Record version increases. The owner fails
  preparation and must retain it. Removing only the version check must fail.
  This coordinated writer updates Task before Record, so the marker sees an
  owning Task-row conflict; the release-token Record version rejects release.
  Record-only factual gate observation requires Evaluating and cannot change this
  Running claim. Do not claim a records-table marker consumer. Other Task IDs are
  unreachable in this marker; direct classifier tests are defense in depth,
  without consumer mutation credit for those alternatives.
- For both owners' identical attempted data, a reserve-loser token minted from
  the pre-commit snapshot is masked by the Record-version guard: record it as a
  defense-in-depth equivalent, with no mutation credit. A combined mutant that
  mints on reserve failure and adopts the winner's fresh committed token must
  fail the controlled loser/winner consumer race. No stale tuple is ownership.
- Use a marker-specific SQLite UPDATE trigger on the owning Store connection to
  abort only a Workflow-body false-to-true active dispatch_started transition.
  A database-written one-shot counter would roll back, so do not rely on it.
  The single marker UPDATE returns its unique trigger error; owner release keeps
  dispatch_started false and is not aborted. Retained same-version claim proves
  conservative classification. Under the any-marker-error-eligible mutant, assert
  release actually committed Failed+RetryEvent and incremented versions. Drop the
  trigger before next-step control. This does not simulate
  an unknown COMMIT outcome, which remains conservative and explicitly untested.
- Compiled mutants: premature observer reset/observer writes, owner release
  omission, exact-version check omission, incorrect EvidencePort release,
  marker-after-start, direct missing Store TerminalRecovery dispatch fence,
  combined missing Engine/Store TerminalRecovery dispatch fences, retry after
  release CAS loss, removed definitive-publication disable, and unknown marker
  error eligibility. Restore exact source and run controls. Document equivalent
  single mutants separately from causal combined mutants; no false killing credit.
  Fixed attempt timestamps and hooks are per Engine/Store, never process-global.

All tests use synthetic domain/adapters, owned temporary SQLite/Git fixtures and
bounded synchronization. Run full Workflow/shared-state regressions, formatting,
strict Clippy, debug/release build and exact-head Linux/macOS CI. Independent
immutable source review follows verification; no current-head green claim is
inferred from earlier checks or constrained-concurrency diagnostics.

## Impact

Inspected nine reservation entrypoints: step/poll (workflow.rs 753/1020), retry
(1819), resume_gate (1409), request_finalization (1254), cancel/fail_task via
terminate (1177–1183), release_terminal_reservation (1226), and escalate (656).
Only step/poll acquire new preparation/release behavior. retry refuses Running;
resume_gate requires Waiting or irreversible Failed; finalization and escalation
require no active claim. cancel/fail commit a terminal decision without closing
the reservation. TerminalRecovery requires that decision and both Engine/Store
dispatch fences. No nonterminal entrypoint releases on a missing Session alone.
Recovery reason references distinguish reversible owner recovery14 from
irreversible outcome reconciliation13 without changing their fences.

Search of README/docs/source found old reset/retry strings only in workflow.rs
1073–1087 and the old Evaluating reason at1054. No production string-parsing
consumer was found. dispatch_started test consumers are workflow/tests.rs590
(preset marker),999/2188 (synthetic histories), and3186 (final-claim regression).
The first three keep their assertions; final-claim changes only its agent path to
immediate owner release then next Started, preserving the EvidencePort path.
Existing Issue8 verification remains historical; README/master describe current
behavior rather than rewriting old evidence. New tests cover entrypoint refusals.

Recovery reference audit in workflow.rs: the post-start acknowledgement comment
(1018) and unbound dispatch guard (1244) are native/owner recovery #14. Evaluating
wait (1054), terminal-recovery unknown-outcome guard (1240), and resume_gate comment
(1408) distinguish reversible owner claims (#14) from PrGate/MergeGate/Cleanup
irreversible outcomes (#13), using durable phase/state. Native no-Session waits
(1073–1087) use #14 without asserting interruption. Actual observed external-effect
drift paths (793/1351/1532/1704/1759) remain #13. No tests parse the unbound-dispatch
or interrupted-evaluation messages; tests 3463/3624 match only the preserved
`unknown external outcome` fragment. The existing final-claim test at3186 has an
agent observer-Invalidated expectation at3217, already explicitly replaced above;
the other Invalidated expectations refer to policy/source/approval drift and retain
their behavior.

Project/Goal writer audit: ProjectRegistry::add writes initial registration,
explicit validated recovery and changed name/config/rules/environment/namespace/
capacity metadata (project.rs144); reconcile writes Registered-to-Blocked for
invalid inputs (160); remove soft-removes when Store permits it (265).
ProjectRegistry::list and resolve/status invoke reconcile, so they can persist
Blocked transitions and are not unconditional read-only paths. Store::project,
projects, goal, goals and WorkflowEngine::read snapshot access are pure reads.
The new agent observer branches are write-free; EvidencePort poll keeps its
existing Workflow/Task mutation path.
All current Goal writes use public Store::put_goal: creation and caller-authorized
metadata/lifecycle updates, including pause/resume; no production Goal CLI/runtime
writer yet calls it. Workflow Context/Task publication does not write Goal rows.
Per-Task progress writers in future #23/#24/#27 must avoid routine Goal-version
bumps or account for the resulting #14 recovery frequency. Any Project/Goal
metadata or lifecycle version change after refresh can retain a live preparation
claim even with active owners; #41 preserves this conservative boundary rather
than introducing a new semantic-version policy.

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
Marker-without-Session reason must also allow launch never having
begun (a marker-commit error does not establish that adapter.start ran).

The old final-claim native regression legitimately changes from error then
observer Invalidated then Started to owner error/release then next step Started.
Its EvidencePort case retains the original attempt/context, with stronger checks.
