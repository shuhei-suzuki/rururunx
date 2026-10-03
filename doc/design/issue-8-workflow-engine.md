# Issue #8 workflow engine

## Runtime contract

The stepwise engine owns one workflow record per exact Task scope. A phase is
reserved durably before dispatch. Agent phases launch via AgentAdapter and retain
Session identity; subsequent steps inspect the same Session. External phases run
an explicit evidence port. Every published pack calls the source selector with its actual target phase and
budget, then checks unchanged authority; stored metadata cannot relabel a payload
selected for another phase.
Completion requires typed evidence with exact phase,
scope, revision, source versions, launch ContextVersion and saved native actor. Review phases additionally require a review
verdict supplied by the review integration; native exit zero is insufficient.

Missing evidence becomes a durable waiting attempt. Failed/Lost sessions do not
advance. Gate evaluation receives its own durable CAS reservation before an external port
is called, preventing duplicate side effects from simultaneous native polls.
An interrupted Running/Evaluating attempt stays reserved for explicit recovery (#13)
rather than silently launching another process. Phase retries are explicit and
must not bypass native executor reservations.

## Presets and escalation

QUICK: Worktree, Implement, Commit, Tests, ImplementationReview, PR.
STANDARD: Issue, Worktree, Requirements, RequirementsCommit, RequirementsReview,
Design, DesignCommit, DesignReview, Implement, ImpactAnalysis, Commit, Tests,
ImplementationReview, PR, MergeGate,
Cleanup. STRICT inserts SecurityReview, ExpandedRegression and Mutation; browser
and staging phases are configurable and require separate evidence. Security and
mutation evidence are never synthesized.

Risk R0 recommends QUICK, R1/R2 STANDARD, R3 STRICT. Effective selection takes the
maximum of configured minimum, stored Task workflow, requested stricter
choice and risk. Escalation preserves historical attempts but invalidates active
phase completion evidence and creates a new context generation. Conservative
restart runs newly mandatory prerequisites before implementation/review again.

## Atomic persistence and context

Store format v3 adds Workflow record authority; ordered marker migrations preserve
v1/v2 JSON and reject future formats. Transaction helpers preserve all existing
Task/Session/lock ownership guards. A workflow transition checks Project and Goal
versions and activity, Task CAS, Workflow CAS, and consecutive ContextVersion before
committing Task, workflow history and optional new ContextVersion together.

Project config/rules and context source capture run outside SharedStore. Rules are
resolved only within the owning source and preserve mandatory content. Snapshot
versions are rechecked in the transition transaction. Workflow authority is writable
only through this atomic API; old attempts and decision history cannot be truncated
or rewritten, and launch sources are checked again before native dispatch. A context source port returns
exact scope, revision, source versions and factual payload. The engine adds phase,
workflow generation, rule versions and context budget class. A new phase or source
change creates a new pack; dependent completion evidence is invalidated. Freshness
is checked again after agent/external execution before accepting evidence. A phase
cannot reuse an obsolete review bundle. Budget class guides #18 selection, without
trimming mandatory rules.

## Integrations pending

#9 supplies independent review sets and remediation-round verdicts; #12 supplies
approval decisions without weakening native controls; #13 reconciles interrupted
phase/session reservations; #18 supplies repository context selection. Evidence
ports are authoritative trusted caller integrations, not sandbox attestations.
This issue implements orchestration, not these integrations or arbitrary command
verification. Environment values stay in memory and do not enter workflow history.

Native transport completion uses additive `AgentAdapter::transport_succeeded`.
The generic default requires Exited, no failure, and actual exit zero. A persistent
server provider may override using its private owned completion journal after
verified group cleanup and terminal persistence, preserving actual OS exit code.
Caller-supplied recovery JSON is insufficient. Successful transport still requires
separate scoped review/test/acceptance evidence.

## Reviewed gate lifecycle and revision semantics

Explicit user Goal section 35 requires commit before tests/review. RequirementsCommit,
DesignCommit and implementation Commit are trusted Git evidence ports; their successful
process exit cannot invent a commit artifact. The captured revision is the actual
owning HEAD. Target-producing Issue/Worktree/requirements/design/implementation/impact
and commit phases attest their final captured authority, which the engine publishes
atomically. PR and merge gates are target-preserving: known drift prevents invocation,
and drift after an actual external operation retains its factual result in a durable
human/recovery hold; it never restarts the generation into a duplicate PR or merge.

Review Evidence includes exact source dependency digests selected by the trusted review
integration, including all mandatory rule digests and at least one reviewed artifact.
Later artifact changes restart prerequisites when these digests change. HEAD alone may
change during another committed milestone without changing the approved document;
its explicit dependency digests still bind that earlier formal review. Dependencies
are not a claim that arbitrary generic commands enforce a read-only review.

Definitive invalid evidence or a rejected verdict records Failed with its reason and
allows explicit remediation/retry once native Sessions are resolved. Unknown gate
errors retain Evaluating with a recovery diagnostic; no native death is inferred.
A waiting gate can use `resume_gate` to reevaluate the same ContextVersion/Session
without launching again. Until accepted, a produced target is not published as fresh:
the launch pack remains immutable with `context_fresh=false`. Explicit phase `retry`
relaunches separately, removes only that attempt's blocker and preserves its original
completion timestamp. Restart status errors keep the Running reservation and record
recovery diagnostics; #13 must verify native termination before any release.

After a definitive gate result, owners are reread, activity and source authority are
checked again, and only workflow fields merge into the latest Task. Issue/worktree
bindings, Goal metadata and unrelated Task blockers/actions survive under refreshed
Project/Goal/Task/record CAS. Policy/source changes invalidate authority before external
effects; existing PR/merge/cleanup outcomes instead remain held for reconciliation;
a later CAS conflict still preserves the evaluation reservation for recovery.

The Store rejects general Task/context writes that change workflow-owned fields.
Conservative WaitingHuman metadata updates remain available within existing Blocked
Project rules. Initial authority cannot contain invented evidence/history; each new
completion requires the active Evaluating→Succeeded transition and matching scoped
phase/generation/Session/ContextVersion evidence. Unresolved reservations, finished
flags and terminal Task states cannot be forged by skipping those transitions.
Native live/Lost records remain authoritative even for state-only reservation release.

Default workflow is reserved for Task creation integration (#11); Task::new currently
uses STANDARD, and the phase engine consumes the stored class. It is not a minimum for an explicitly classified
Task. BudgetClass guides #18 breadth/selection; configured phase token caps apply to
all classes until measured per-class settings exist. Mandatory rules are never trimmed.
Finished workflows require a new Task for further escalation. Every generation
invalidation records cause and old/new revision/source digests separately from actual
class escalation.

## Conservative operation observations and terminal decisions

The raw Workflow mutation API is crate-private. Its private unit regressions include
positive valid controls; a compile-fail consumer check prevents external raw authority
writes. Within one generation, source authority changes only with a target-producing
completion. Equal-class escalation on changed authority records invalidation. A new
generation interrupts an active attempt; an Evaluating unknown observation cannot be
released by a generation edit. Native dispatch has an immutable durable marker before
start, distinguishing known undispatched reservations from uncertain launch interruption.
Reserved external phases with no evaluation claim can safely reenter evaluation. Owner
refresh and source recheck precede both the gate claim and native dispatch.

Each gate return (including integration error) is journaled immediately in its exact
active attempt and a scoped workflow.gate_observed event, before postgate ownership,
activity, filesystem or CAS checks. The observation preserves its actual artifact
references and error even if a Goal is paused or a Project becomes Blocked. It does
not approve the evidence, change Task/native state or release a reservation. Known definitive Evaluating observations retain their pre-operation source authority and
resume only the postgate validation/publication path after a transient owner/source/CAS
error, without repeating the port. Unknown observations await #13 reconciliation.
The closing fence checks the attempt's own native Session plus executor/Lost reservations;
an unrelated live consultant may coexist with read-only review, while mutation admission
still fences all live agents. Native completion requires the owned persisted Exited state
and the provider's transport contract. Raw Session terminal writes remain a trusted
provider/recovery boundary: only verified native supervision may attest termination;
caller recovery JSON, cancellation or failure decisions cannot prove process death.

`cancel`/`fail_task` append explicit exact-scope terminal decisions while retaining any
active phase and native reservation. They do not signal or imply process termination.
QUICK ends at PR-created, which remains nonterminal. `request_finalization` privately
extends that completed preset with real MergeGate and Cleanup ports, preserving the
reviewed HEAD; only their accepted scoped evidence permits Completed. Goal completion
and Project removal still require their own independent evidence/terminal conditions.
Cleanup disposes the worktree, so its final pack freezes the pre-disposal reviewed
source and actual scoped cleanup evidence; the engine does not recapture a deleted
worktree or claim post-disposal freshness for subsequent execution.

If identical foreign blocker text makes ownership ambiguous, retry retains it rather
than removing another component's blocker. Typed blocker routing remains #12 integration.

## Reconciliation after external effects

Pr, MergeGate and Cleanup are irreversible phases. Their observations and prior artifact
references are passed to the same attempt on explicit resume; ports must reconcile them
idempotently. Source/dependency drift after an actual operation holds the known outcome
for explicit #13/human reconciliation rather than invalidating and creating a duplicate
PR or restarting implementation after a merge. Accepted PR/merge evidence also prevents
subsequent drift or escalation from discarding the generation. Ordinary retry cannot
release these phases. Failed/Waiting Cleanup resumes its frozen pre-disposal source,
including the next final pack, without reopening a removed worktree.

Cancellation/failure decisions are conservative and remain available under Paused,
Cancelled or Blocked Goals and Blocked Projects. Removed Projects reject them.
`release_terminal_reservation` only closes the immutable decision's recorded attempt,
keeps Task/decision/context/evidence unchanged, and requires its owned persisted native
termination plus all executor/Lost fences. Unknown gate outcomes and unbound native
dispatch remain reserved for #13's explicit reconciliation authority. This method cannot
prove process death; the existing trusted provider/recovery Session boundary must do so.
Workflow progression remains forbidden for terminal Tasks. External WaitingHuman is a
conservative interruption, while the next accepted workflow transition owns its phase
state; finished QUICK finalization therefore uses workflow PR evidence rather than the
interruption state. Blocker removal uses refreshed owner data and retains ambiguous
identical foreign text until typed #12 ownership is available.

## Exact evaluation claims

Each Evaluating transition records `claimed_observations`, the exact observation count
for that attempt's current round. The private observer requires exact scope, phase,
generation, Session, ContextVersion and claim identity and appends exactly one outcome.
Only the observation at that claim index can resolve it; prior rounds stay immutable.
Concurrent polling or restart during a resumed round cannot replay an old Waiting or
Failed outcome. Cancellation retains that in-flight reservation, and terminal release
refuses it until its exact current outcome is known. A completed phase must carry that
claim's actual Passed evidence. A cancelled ordinary Failed phase can close as Interrupted
only for its immutable terminal decision, after the same native termination fences.

Observations retain authority identity/digests with an empty payload; the immutable
ContextVersion owns source text. Human/recovery holds persist `held_reason`, WaitingHuman
and a scoped blocker, and repeated identical polls do not rewrite it. The external
outcome reconciliation integration is an explicit prerequisite in
[Issue #13](https://github.com/shuhei-suzuki/rururunx/issues/13); #8 never fabricates its
authority. Cleanup reuses its reserved workflow class/configured phases as well as its
pre-disposal source. Project risk mappings combine elementwise maxima with runtime
recommendations; they may strengthen safety classification, never weaken it.
Native status divergence persists a bounded recovery diagnostic and returns Waiting
without another gate invocation, rebinding or inferred native death.

Raw transition writes cannot append or alter observations: only the private audited
observer may do so. Holds remove the attempt's known prior Waiting/Failed blocker from
refreshed owners before replacing its diagnostic, preserving unrelated/ambiguous text.
Earlier unpublished Issue 8 v3 snapshots without evaluation claims intentionally fail
to deserialize; disposable development fixtures must use fresh state. No implicit zero
claim default is safe. Production v1/v2 migration remains supported and lossless.
