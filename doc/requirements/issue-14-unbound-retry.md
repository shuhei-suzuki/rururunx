# Issue 14: fence explicit retry of an unbound native dispatch

Requirements3 approved at1463d37 by two independent native reviewers, no
Critical/High/Medium or blockers. Low precision dispositions accompany Design1.
Risk STRICT: shared Workflow/Store
reservation and replay boundary. Base main efe9774, normal composition of63 after
both Requirements1 reviewers finished/cleaned. Source candidate verification and
independent source reviews pending; approved WHAT is unchanged.
Related open Issue14's "Verified explicit retry gap" acceptance; limited
prerequisite only, not Scheduler/restart/native recovery completion.

## Problem and scope

WorkflowEngine::retry admits Waiting/Failed nonirreversible attempts and checks
Sessions, but a committed native marker with no bound Session passes that loop.
The Store's crate-private StateOnly closure also accepts it. Missing Session does
not prove that no input was sent or that the preparation/native owner is absent.
Issue41 deliberately left this gap for14; ordinary observation never replays.

## Required behavior and enforcement

1. An active Executor/Reviewer attempt with dispatch_started=true and no bound
   session_id cannot be explicitly retried. Failed/Waiting, user reason, no Session
   rows or unrelated terminal Sessions prove no safe release. Refuse before
   mutation with exact Task/Workflow Record/body/version/active/history/retries,
   all contexts/pointers, Sessions/locks, Project/Goal rows/versions and audit
   unchanged. Fixed bounded explanation: launch outcome unknown, launch may or
   may not have begun, trusted original-attempt recovery required. Do not assert
   alive/dead/interrupted or echo caller reason, native or persisted payload.
2. Use BOTH public Engine retry and existing crate-private Store transaction
   closure fences. Store must refuse any closure/replacement of an active native
   marked-unbound reservation, including via StateOnly or generation change;
   marker/Session/phase identity alone supplies no resolution. Preserve the same
   original active attempt when its owner publishes Running→Failed, binds its
   returned Session or records TerminalDecision; those are not reservation
   replacement/closure and remain allowed. Do not add a new
   proof/API/schema. Existing CAS compares original Task/Record and owner versions;
   existing non-Running marker/Session immutability and native termination checks
   remain. A future genuine14 recovery producer needs its own reviewed authority.
3. Existing supported retries remain available: definitive pre-marker Failed
   native decisions with dispatch_started=false/session_id=None (both actors),
   resolved nonirreversible EvidencePort Waiting/Failed outcomes, and native
   Waiting/Failed outcomes with an exact terminal bound Session satisfying all
   existing fences. Pre-marker registry/capability/missing-worktree failures must
   not become permanent reservations. The component certifies no legacy bound
   Session's native/F1 termination beyond existing policy.
4. Running/Interrupted/Evaluating retry and irreversible outcome fences stay.
   EvidencePort attempts never carry a native marker or Session under the Store
   transition invariant. Proven pre-marker owner-local release, pre-marker source
   invalidation/generation changes and committed terminal-Task recovery remain
   valid under41's exact ownership/dispatch/Session fences. Bound terminal Session
   progression stays unchanged. No generation change may waive an unresolved
   marked-unbound native reservation.
5. No force/bypass flag, inferred no-dispatch certificate, recovery JSON, elapsed
   time/PID proof, fabricated Session/termination, timeout or permission relaxation.
   Adapter ErrorKind/text, including deterministic configuration refusal, is NOT
   trusted no-dispatch evidence. Retain unknown outcome until a genuine reviewed
   producer resolves the exact original attempt.

## Accepted availability consequence

The marker commits BEFORE every adapter.start. Generic executable/config/model/
effort/input/binding/Git/preflight/Store/process-start errors, and Grok pre-persist
input/capture/environment/ownership/registration/Session-save errors returned
from start, can yield the same Failed marked-unbound attempt.
Grok Git/profile/admission/spawn/actor failures AFTER start returns its saved
Session are bound outcomes under existing policy (Failed or Lost) ONLY after the
Workflow Session-binding transition commits. A binding CAS loss leaves Issue41's
retained Running marked-unbound attempt regardless of the actor's later terminal
Session; durable fields determine classification and14 recovery remains required.
Generic can save its own same-scope/agent/role terminal Session
before returning Err; that unbound row supplies no trusted no-dispatch proof.
Enumerate all current start implementations and
transitive error producers in source impact analysis; do not infer effects from
an error label. Conservatively, even user-fixable post-marker configuration errors
retain the claim: Task cannot progress, Goal cannot complete through it, and Project
removal is blocked. Neither retry, resume_gate nor cancel/fail plus TerminalRecovery
provides an operator escape before separate14 trusted recovery exists. This is an
explicit accepted prerequisite consequence of the open Issue14 no-replay contract,
not production availability or parent completion. README/master/Issue14 must state
it. Current status/Scheduler integration remains pending; classify this state from
actor/state/dispatch_started/session_id, never error text, and claim no new endpoint.

## Actual controls and mutation evidence

- Hold actual WorkflowEngine::step adapter.start AFTER the marker commits, for
  both Executor and Reviewer. Verify durable marker/no bound Session; release
  start with a synthetic error, then verify real Failed unbound outcome. Refusal
  is independent of error kind/text: existing Fake Unsupported label earns no
  no-dispatch proof. Public retry refuses unchanged, subsequent ordinary step
  does not replay, and both adapter launch counters are stable. Fake fixtures
  provide no native/F1 certificate or actual host-process-death evidence.
- Explicitly map Executor fixture to no Session rows and Reviewer fixture to an
  unrelated terminal Executor Session from Implement. Before/after snapshots must
  include Project/Goal versions, Task, ALL Task-scope Record kinds, EVERY context
  version and complete audit rows (not only latest context or a paginated prefix),
  plus both launch counters. Use isolated SQLite fixture/raw reads for snapshots;
  schema checks remain enabled. No raw rewrite to bypass marker constraints.
- Real pre-marker Executor and Reviewer failures through step remain retryable:
  unregistered Executor and Reviewer lacking Review are viable controls. Verify
  marker=false/no Session, exact original-index RetryEvent and active=None; fix
  actor configuration through supported writer and next step reserves a new
  attempt/launches exactly once. Run resolved bound-Session and EvidencePort retry
  positives with actual transition/counter assertions too.
- Direct crate-private Store closure of the actual marked-unbound Failed fixture
  must refuse unchanged; matching pre-marker/bound/port closures must work.
  Also hold the actual Running marked-unbound start for both actors, before any
  Session exists for that invocation. Direct Store attempts to close it via
  Running→Failed+RetryEvent in the same generation, or Running→Interrupted plus
  generation+1/valid invalidation/fresh valid context, must refuse unchanged ONLY
  because of the new fence. Release the actual owner and join its real handle;
  successful single launch/binding or its Failed publication keeps the original
  attempt correctly. Separately verify TerminalDecision preserves active.
  Failed→Interrupted generation change already fails existing state validation:
  NO new-fence/mutant credit for that fixture. Running controls are crate-private
  boundary evidence, not a public retry or native/F1 certificate.
- Include a held-start Executor variant whose synthetic adapter saves a terminal
  Failed Session of its own scope/agent/role before returning Err (Generic-shaped
  persistence only, no native certificate). Both public retry and direct Store
  closure refuse with full snapshot/counters unchanged; a same-agent/role terminal
  row cannot supply resolution. Exact terminal bound-Session positives still work.
  Existing synchronous characterization remains in Git history; rename/update
  post-fix assertions and all41 references truthfully.
- Killable compiled mutants: remove Store fence (direct Store consumer); remove
  BOTH Engine+Store fences (public held-start consumer); narrow BOTH fences to
  either actor (other held-start consumer); drop dispatch_started condition in
  either guard (pre-marker positives); refuse all sessionless attempts (port
  positives). Exact forms must be recorded. Engine-only removal remains masked
  by Store, actor-condition removal is equivalent because port marker impossible,
  and Engine Failed-only narrowing cannot be killed by production Waiting: NO
  credit. Store Failed/Waiting-only or same-generation-only narrowing MUST be
  killed by the actual held Running closure/generation controls. These are direct
  Store consumers; do not mislabel public retry kills. A same-agent/role terminal
  row-as-resolution mutant must fail the matching-row controls.
  Any constructed marked-unbound Waiting coverage is defense in depth only, not
  a public consumer/mutation claim. Restore exact source/tree and rerun controls.

## Impact, gates and boundaries

Enumerate every retry caller, every put_workflow_transition caller and reader of
phase actor/dispatch_started/session_id across the workspace using rg. Current
production transition writers are persist, reserve and persist_decision; tests
also use crate-private Store directly. The other Workflow Record writer is
observe_workflow_gate via observe_gate: it requires the same Evaluating active
index and changes only observations/detail, without closing or replacing it.
Confirm put_record rejects Workflow and inventory all direct writers too.
Inspect step/poll, prepare_agent/owner
release, retry, resume_gate, fail/hold/invalidation, escalate, request_finalization,
cancel/fail_task/terminate, terminal release, Store CAS/generation/closure guards,
Project removal and existing status consumers. Record symbol/file/line and actual
non-impact rationale: no active closure, requires active=None, terminal-decision
only, existing marker/Session fence or new fence. No parsing of refusal text.

Default full regression, fmt/all-target Clippy-Dwarnings, debug/release builds and
affected Workflow release tests; independent requirements/design/source reviews
and current both-OS exact-source CI before limited merge. Retain prior failures
with source/cause limits. No direct main edits or tracking Issue14 closure. No new
schema/native producer, scheduling, attach, restart, full14/native/F1/MVP acceptance.
