# Issue14 explicit unbound native retry fence

Design2 candidate NOT APPROVED. STRICT shared Engine/Store reservation boundary.
Requirements3 approved1463d37 with two independent native APPROVE/noCHM/blockers;
precise Low dispositions recorded in requirements3-reviews.json. Base main efe9774.
No production changes yet. Whole14, Scheduler/restart/native/F1/MVP remain OPEN.

## Two guards and immutable reservation

No schema or public API change. Use the existing PhaseAttempt's actor, marker and
session_id and current transition CAS. Predicate: phase.actor()!=EvidencePort AND
dispatch_started AND session_id.is_none(). All original legacy fields are unchanged.

Engine retry: after read/active/Waiting-or-Failed admission, before its Session loop
and every RetryEvent/completed_at/active/blocker mutation, refuse this predicate.
Existing Running/Interrupted/Evaluating, irreversible and live/Lost/executor fences
remain. No ErrorKind, timeout, user reason or matching terminal row resolves it.
Static message: `native launch outcome unknown; launch may or may not have begun;
original-attempt recovery required (#14)` (one bounded literal, no raw cause/data).
No consumer parses it. Classification uses the durable fields only.

Store put_workflow_transition: in its existing previous.active Some and after.active
!=previous.active closure block, inspect the original attempt and refuse the same
predicate for access!=WorkflowAccess::TerminalRecovery BEFORE Session iteration
or context/Task/Record/audit publication. TerminalRecovery keeps its EARLIER
conservative identical unbound marker fence; exempt it only from the redundant new
guard, so every access mode still refuses this unresolved closure. No recovery
proof is created or removed. This preserves41's single Store TerminalRecovery and
combined Engine+Store TerminalRecovery causal mutants; a differing error-text
assertion never earns kill credit. Keep
validate_transition and all owner/Task/Record CAS, context identity, generation,
Session/lock and terminal fences. SQLite immediate transaction rolls back refusal;
input Task/Record structs update only after successful commit. Refuse a raw current
index replacement and generation closure too. No same-generation/state-only escape.

Keep active unchanged when the actual owner publishes Running→Failed, binds the
returned Session (Running→Running) or records TerminalDecision. These bypass the
closure block correctly; its purpose is preventing removal of the original unknown
reservation, not preventing truthful facts. TerminalRecovery's independent marker
fence stays and is the sole enforcement for that access mode. Pre-marker
release/invalidation/generation and bound valid progress
remain available. A future14 genuine producer needs separate reviewed authority;
this component adds no bypass/proof/certificate/force flag.

The Engine and Store guard expressions remain locally explicit, using existing
Actor and actor() with no new public helper/type. Two guards enable independent
consumer checks; require ONE shared crate-private static explanation literal for
Engine and the new Store guard, with no capability meaning. Engine-only removal
must remain indistinguishable; actual accepted closure/state/counter changes, not
message differences, determine causal kill credit. Removing one Engine guard is
masked by Store and cannot
earn mutation credit. No production decoder or error-chain behavior outside these
valid-snapshot guard refusals changes.

## Real fixtures and complete snapshots

Reuse owned temporary SQLite/Git Fixture, FakeAgent's actual start_pause and
start_error, ready_agent(false/true), spawn_step and bounded Pause. Fake is an
unqualified synthetic adapter: no native/F1, death or actual19 certificate claim.
No process-global state or timeout extension. Always release/join real owners.

Add a component-only raw snapshot helper reading every Project/Goal/Task row/body/
version, ALL Task-scoped Records including Session/locks, ALL context versions and
complete audit rows; include both adapter launch counters. Use stable SQL ordering,
no paginated prefix. Schema/constraints remain enabled. Read valid typed ownership
before comparison; no raw write to bypass Store dispatch constraints.

Held-start unknown controls for both actors: owner commits marker and pauses in
actual start, with no new Session for that invocation. Assert executor has no
Session rows; reviewer has only the unrelated terminal Implement Session. Release
with synthetic start_error, assert actual Failed/marked/unbound and stable launch
counts, then public retry must refuse exact static text and unchanged snapshot.
Ordinary observation must not replay. Refusal depends on fields, not error label.
A separate same-scope/agent/role terminal Failed row variant uses test-only Fake
persistence before returning Err, matching Generic shape but no real native proof.
Public and direct Store retry closures still refuse. Bound terminal positives
continue to clear active and append the exact-index RetryEvent without extra launch.

For direct Store retry builders, clone fresh actual Task/Workflow Record and scoped
owner versions. Same-generation closure changes active=None, original Running→Failed
if necessary, completed_at only if absent, an exact prior-index RetryEvent and
only the permitted retry fields. The SAME retry builder must also COMMIT on a
bound terminal Failed native review-rejection fixture, nonirreversible port
Waiting/Failed fixture and pre-marker definitive Failed fixture, with exact-index
RetryEvent and active=None. These validate construction, not extra mutation credit.
Reuse it on pre-marker and marked Running fixtures too. For generation closure,
use actual held Running (not Failed), clone the
snapshot, call existing invalidate/new-generation helpers and make_context for the
FIRST configured phase AFTER invalidate (next_phase(&invalidated).unwrap()), using
workflow.sources, new generation/class/budget and latest_context+1. Match
Task.context_version/revision and Workflow.context_version as set_context does;
clear completed/active, append one
invalidation, set old Running→Interrupted with timestamp and publish matching valid
ContextVersion/Task pointer. Do not guess invalid context data. The generation
builder must first succeed on actual held pre-marker Running for BOTH actors.
Run both pre-marker capture_pause offsets3 and4 for each actor. Resume/join that
former owner and compare the full post-closure snapshot and both launch counters:
no stale write/audit or launch. Its error/eligibility differs before/after refresh;
assert no specific error text or release-eligibility path.

On actual held marked-unbound Running for both actors, the same builders must be
refused unchanged only by the new Store guard. Resume/join owner afterwards and
verify normal single original-attempt binding or Failed publication; separate
TerminalDecision control retains active. A Failed→Interrupted generation request
already fails validate_transition; report no new-fence credit. Post-start binding
CAS loss remains41's Running marked-unbound recovery case even if the adapter's
Session row later becomes terminal. No extra no-dispatch proof is inferred.

Pre-marker positive public step controls for both actors: unknown executor registry
entry or reviewer lacking Review publishes definitive Failed with marker=false and
no bound Session. Retry appends one exact-index event and clears active; supported
Task actor writer repairs configuration, then next step reserves fresh and launches
exactly once. Existing bound terminal review rejection and nonirreversible port
Waiting/Failed retries must keep their actual transitions/counters. Marked-unbound
Waiting is not public-reachable; any direct constructed check is defense in depth,
with no public mutation claim. Rename/update the former synchronous gap regression
and41 references; retain historical behavior in original Git/log evidence.

## Causal mutants and restore

Every operator starts from a clean committed fixed source in an isolated mutation
worktree, compiles, then fails at the relevant consumer assertion. Compile/setup,
state-regression or invalid-context failures earn no new-fence credit. Record exact
patch/source/test/tree/log and actual assertion, and restore exact tree/control.

- Remove Store guard: direct Failed/Running retry and Running generation consumers.
- Remove BOTH guards: public held-start unknown refusal consumers.
- BOTH guards Executor-only or Reviewer-only: opposite actor public held-start.
- Drop marker conjunct in Engine or Store: actual pre-marker public positives.
- Refuse sessionless across all actors: actual port retry positives (record guard).
- Store Failed/Waiting-only: direct Running same-generation refusal.
- Store same-generation-only: direct Running generation refusal.
- Terminal same-agent/role row as resolution: Store-only direct matching-row,
  BOTH public matching-row. Engine-only remains masked with no kill credit.

Engine-only removal is masked; actor-conjunct removal is equivalent because port
marker is forbidden; Engine Failed-only has no public marked Waiting control.
Explicitly report executed survivors without inventing kills; optional direct
Waiting checks cannot upgrade them to public-consumer evidence.

## Source impact and verification

Publish actual rg inventory of every retry/transition caller, attempt-field reader
and Workflow Record writer, with current symbol/file/line and closure rationale.
Current source transition writers: persist, reserve, persist_decision; other direct
writer observe_workflow_gate keeps same Evaluating active and only observation/
detail. put_record rejects Workflow. The new closure guard is before Task/Record
CAS, so stale closure callers may receive its untyped refusal instead of typed
SnapshotChanged; no current closure caller relies on that classification. Marker
publication keeps active unchanged, so41 own_task_marker_rollback typed-classifier
semantics are unaffected. Include41 terminal recovery tests/design/mutation evidence
as consumers; their existing TR guards remain independently causal under the new
non-TR guard. Include step/poll, prepare_agent/release,
retry/resume_gate, fail/hold/invalidation, terminate/cancel/fail_task, escalation/
finalization, terminal recovery, Store owner/CAS/generation/closure, Project removal
and current status interfaces. All default settings/hooks/permissions unchanged.

Generic returns startErr before Workflow binding, sometimes after saving its own
terminal row. Grok launch returns startErr only before/at save_current; post-Ok actor
failures are bound ONLY when Workflow binding commits. A binding CAS loss leaves
Running marked-unbound despite the real Session row; classify from durable fields.
These unknown attempts block Task/Goal progress and Project removal until genuine14
recovery; no operator retry/cancel-terminal-release escape or availability claim.
Update README/master/41 disclosure and authoritative14 evidence on implemented
status after source qualification. No new CLI/status endpoint or Scheduler claim.

Run fmt, default normal-host full workspace tests, all-target Clippy-Dwarnings,
debug/release builds and affected Workflow release tests from clean changed source.
Keep historical main555 release failure/cause UNKNOWN separately. Compiled mutants,
independent immutable Source reviews and current both-OS CI with actual checkout/
complete-tree/source identity gate a limited merge; empty closing references and
normal cleanup. No whole14/native/F1/MVP acceptance or producer authority.
