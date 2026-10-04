# Workflow Engine Design

**Status:** Rust phase runner implemented (Issue #8); dependent integrations pending
**Scope:** MVP workflow orchestration

## 1. Goal

Represent the current development process as an executable stateful workflow while allowing lighter paths for small changes.

Task Workflow Engine operates below Goal Runtime and always within a Project boundary. Goal Runtime selects dependency-ready Tasks; Workflow Engine executes the selected Task through QUICK/STANDARD/STRICT phases.

## 2. Built-in workflow presets

### QUICK

```text
Prepare Worktree
→ Implement
→ Commit
→ Relevant Verification
→ Review
→ PR
```

### STANDARD

```text
Issue
→ Worktree
→ Requirements
→ Requirements Commit
→ Requirements Review
→ Design
→ Design Commit
→ Design Review
→ Implement
→ Impact Analysis
→ Commit
→ Tests
→ Implementation Review
→ PR
→ Merge Gate
→ Cleanup
```

### STRICT

STANDARD plus configured:

- security review
- expanded regression
- mutation verification
- headed browser verification
- staging verification
- stricter merge gate

## 3. Workflow selection

Selection takes the monotonic maximum of stored Task workflow, runtime/Project policy
minimum, explicit stricter user choice and risk classifier recommendation. The configured
default is reserved for Task creation integration (#11), not a phase-engine floor;
Task::new currently uses STANDARD until its caller supplies a classified stored workflow.

A user may make a workflow stricter.

A workflow must not be automatically downgraded below project minimum.

## 4. Dynamic escalation

Any phase may raise risk.

Examples:

- shared module discovered → QUICK to STANDARD
- DB schema/auth/security boundary discovered → STRICT

Escalation records:

- old workflow
- new workflow
- reason
- triggering evidence
- timestamp

## 5. Phase contract

Each phase declares:

- prerequisites
- inputs
- actor type
- outputs/artifacts
- success criteria
- required reviews
- allowed retry behavior
- failure/escalation behavior
- context inputs / Context Pack version
- context budget class

## 6. Review policy

Review policy is data, not hard-coded Triple Review logic.

Example:

```yaml
implementation_review:
  reviewers: [claude, codex]
  completion: all
  parallel: true
```

Example triple preset:

```yaml
implementation_review:
  preset: triple-adversarial
  reviewers: [claude, codex, grok]
  completion: all
  parallel: true
```

Example quorum:

```yaml
implementation_review:
  reviewers: [claude, codex, grok]
  completion: quorum
  quorum: 2
```

Supported completion modes:

- `all`
- `quorum`
- `any`

## 7. Review rounds

A failed/blocking review transitions to a remediation loop:

```text
Review
→ Verify findings
→ Fix
→ Commit
→ Re-run required verification
→ Re-review
```

The workflow keeps round history rather than overwriting previous review results.

## 8. Impact analysis gate

STANDARD/STRICT may require an impact-analysis artifact before test completion.

The artifact should include:

- changed contract/symbol/value
- consumers searched
- impacted locations
- non-impact rationale
- tests selected from analysis

## 9. Mutation gate

When required by policy, mutation verification is a separate gate with evidence of:

- mutation introduced
- expected test failed
- original implementation restored
- test passed
- worktree clean

## 10. Browser and staging gates

Project policy determines when these gates are mandatory.

They should generate durable evidence references for PR/report generation.

## 11. Merge gate

Merge gate checks configured conditions such as:

- required reviews satisfied
- no unresolved Critical/High
- security review satisfied
- required tests passed
- PR not draft
- base branch allowed
- production-impact policy satisfied

If a configured condition requires a human, state becomes WAITING_HUMAN.

## 12. Project rule precedence

Project-specific configuration/rules override global defaults.

The engine should load project rules before selecting a workflow and again when entering phases whose scoped rules may differ.

## 13. Context lifecycle

Workflow Engine coordinates Context Efficiency artifacts at phase boundaries.

Expected behavior:

- Task creation establishes Context Pack v1
- requirements/design/implementation milestones update the Context Pack
- commit/revision changes invalidate stale diff/review slices
- review launch freezes a deterministic Review Bundle for that Review Set
- re-review prefers a delta bundle from the previous reviewed revision
- dynamic workflow escalation may add required rules/artifacts and therefore creates a new Context Pack version
- context expansion requested by an agent is recorded but does not silently rewrite authoritative artifacts

A phase must not start with a Context Pack known to reference an obsolete target revision.

## 14. Token-budget policy

Workflow presets may define different default context budgets.

Example direction:

- QUICK: small repository-map/review budget
- STANDARD: normal budget
- STRICT: broader evidence/context budget

Safety-critical context is mandatory and outside discretionary trimming.

Budget exhaustion results in prioritization or explicit expansion, never silent removal of mandatory evidence.

## 15. Human override

Human actions should be explicit and audited.

Supported examples:

- choose stricter workflow
- approve an escalated decision
- cancel task
- retry failed phase
- choose alternate reviewer/executor
- require manual merge

Human override should not silently disable mandatory project safety rules.

## 16. Goal Runtime integration

Goal Runtime does not replace Task workflows.

```text
Goal Runtime
  ↓ selects dependency-ready Task
Workflow Engine
  ↓ executes QUICK / STANDARD / STRICT
Task result / events
  ↓
Goal Runtime reevaluates DAG + completion criteria
```

Workflow Engine emits normalized events for:

- Task started/completed/failed/blocked
- workflow escalation
- Human escalation
- review/security gate status
- generated follow-up Task proposal
- final Task acceptance evidence

Goal Runtime consumes these events to continue execution without requiring a new user prompt after every Task.

All emitted Task events include project_id and goal_id so global scheduling/status cannot confuse Tasks from different repositories.

A Goal may define minimum workflow constraints for all child Tasks, but project-level minimums remain authoritative.

## 17. Implemented Rust contract

`workflow::WorkflowEngine` initializes one exact Task workflow, then `step` reserves
and dispatches a phase or polls its stable native Session. QUICK, STANDARD and
STRICT execute against `AgentAdapter`; formal requirements/design/impact phases
use executor capability and review phases require explicit Review capability.
Generic CLI's honest Execute-only contract therefore cannot act as a reviewer.
The baseline delegates one configured reviewer Session; independent multi-reviewer
sets, completion policy and remediation reconciliation remain #9 integration.

`WorkflowSources` supplies factual scoped revision/source versions and selected
payload (#18). Authority versions remain comparable across phases; phase and budget
change discretionary selection, not which authority changes are detected. The
runner reloads canonical owning Project config/rule files outside SharedStore and
hashes the same bytes it parses. Mandatory rule text is always included. Runtime
minimum, Project overlay, stored class, explicit stricter choice and risk mapping
combine by maximum. Policy changes retain previously mandatory optional gates.

Every phase carries `ContextBudget` (Small/Normal/Broad plus configured discretionary
limit), ContextVersion and exact source versions. Each transition creates a new
phase-tagged pack, retaining older immutable versions. Revision/source/rule changes
between phases or during review/verification invalidate completion evidence and
restart prerequisites conservatively; mutating requirements/design/implementation
milestones may publish their newly observed sources. Commit/PR cannot carry stale
verification/review evidence across changed targets. Dynamic escalation keeps
attempt history but starts a new generation and context version.

`PhaseGates` is an explicit trusted integration port for external actions and actual
evidence. It receives valid ordered prerequisites, source snapshot, prepared context
and optional native transport result. Passed evidence must match phase, Scope,
revision, source versions and Session, with durable artifact references. Reviews
require an explicit approved verdict. Missing integration (`PendingGates`) waits;
no successful exit invents verification, approval, PR, merge or cleanup evidence.
Ports must use their own Git/Session ownership and transactional guards at actual
native side effects. #10 approval and #14 owner/restart recovery remain pending; uncertain phase
reservations never trigger automatic retry. Explicit retry preserves prior attempt
and reason history and rejects live/Lost executor reservations.

Store format v3 persists Workflow record authority. Task state, workflow history
and ContextVersion pointer commit atomically with Project/Goal/Task/record CAS;
gate evaluation is separately reserved to prevent concurrent duplicate port calls,
and completed attempt/decision history is immutable. Workflow authority uses only
the atomic transition API. Initial authority cannot contain fabricated completion;
new completion requires the active Evaluating→Succeeded phase with exact scoped
evidence. General Store Task/context writes cannot override workflow-owned fields.
A rollback leaves no orphan context or overwritten concurrent metadata. Blocked,
Removed, paused/terminal owners cannot progress. Mutation dispatch rejects active
review locks and all agent dispatch rejects reserved executors. This is a phase
runner library; scheduling, workflow CLI/TUI and actual GitHub publication/merge
ports remain dependent work. See [Issue #8 design](../issue-8-workflow-engine.md).

User Goal §35 sets the executable order: each requirements/design/implementation
milestone commits before its verification or formal review. Target-producing commit
ports attest the captured final HEAD. Target-preserving PR/merge gates never receive
known stale prerequisites. Review evidence binds explicit source dependency digests
(including mandatory rules), so changed approved artifacts restart prerequisites.
Definitive rejected/invalid evidence persists Failed; uncertain integration errors
retain Evaluating for recovery. `resume_gate` reevaluates a definitive Waiting result
with the same Session and launch context; `retry` explicitly relaunches only resolved
attempts. Native restart diagnostics never imply verified process death. Invalidation
history records old/new revision/source digests and cause independently of escalation.
Default class is creation fallback; class-specific breadth is interpreted by #18 within
configured phase token caps, with mandatory rules retained outside those caps.

The Store Workflow mutation API is crate-private. Actual external gate observations
are recorded before postgate activity/freshness checks, including artifact refs when
a Goal becomes paused or a Project Blocked; observation does not authorize completion.
A same-generation source rebind requires target-producing completion, and unknown
Evaluating reservations cannot be released by changing generation. An immutable native
dispatch marker records whether launch may have begun; it does not prove owner
absence. Issue #41 implements passive Waiting observation for agent Running
attempts without a Session, before or after that marker. Such observation changes
no Task/Workflow/context/audit state. Only the invocation whose reserve committed
may release its own proven pre-dispatch preparation error, using its exact
committed Workflow Record identity/version and original attempt. Reservation CAS
losers never release. Fresh-read CAS preserves concurrent Task metadata and all
executor/Lost fences. Preparation pins the reserved actor and Task binding; an
actor override requires release and a new reservation before any dispatch.
For an eligible error with active owners and a winning release CAS, an originally
unbound Task assigned during pre-refresh preparation releases the claim and
returns the preparation error; the next ordinary step reserves with the fresh
binding. Assigned bindings remain exact. An
assignment after refresh can reject a definitive missing-worktree publication
through Store binding immutability, leaving the claim for #14.

Eligibility ends immediately before fail/invalidate/hold/marker publication,
including within invalidation helpers. Only a typed pre-commit owning Task-row
SnapshotChanged matching table `tasks` and the owning Task ID from the marker
transaction can restore it. Project/Goal/Record
version errors and all definitive-publication errors remain reserved.
Inactive owners, definitive-decision publication
conflicts, untyped/unknown marker errors, release CAS/termination fence failures
and unknown/dispatched outcomes stay reserved for #14; no native replay occurs.
EvidencePort claim evaluation remains separate and does not use agent release.
Explicit cancel/fail decisions retain native/phase reservations until verified recovery.
QUICK PR-created is nonterminal; request_finalization adds actual MergeGate/Cleanup ports
before Completed. Native termination state is a trusted provider/recovery attestation,
never an inference from arbitrary JSON or caller cancellation. A live unrelated consultant
may coexist with read-only review; owned and Lost/executor termination fences remain.
Cleanup freezes its pre-disposal source pack because the owning worktree is removed.

Known definitive gate observations can resume postgate validation/publication without
calling the external port again. Irreversible Pr/MergeGate/Cleanup outcomes and accepted
PR/merge evidence prevent generation invalidation into duplicate operations; drift holds
for explicit reconciliation (#13). Their ports receive prior attempt observations and
resume idempotently; Cleanup freezes pre-disposal sources even after Waiting/Failed.
Cancel/fail decisions are conservative under inactive Goals or Blocked Projects; a narrow
terminal reservation release keeps Task/decision/context/evidence immutable. A
bound Session requires owned persisted termination plus all executor/Lost fences.
For an undispatched attempt without a Session, the committed terminal decision,
transactional terminal-Task fence and absence of a dispatch marker exclude the
suspended owner from launching; all executor/Lost fences still apply. No inferred
owner absence is needed for this existing explicit TerminalRecovery path.
Unknown outcomes or unbound
dispatch stay reserved. General workflow progression cannot resurrect terminal Tasks.

An evaluation claim binds the exact prior observation count. The private observer
appends exactly one scope/phase/generation/Session/ContextVersion-bound outcome at that
index; poll/restart/release/invalidation use only the current claim's result, never a
prior resumed round. Completion requires that actual Passed evidence. Unknown in-flight
resumed claims stay reserved, including after cancellation. Terminal ordinary Failed
attempts can close as Interrupted only for the immutable decision with native fences.
Durable irreversible holds publish held_reason/WaitingHuman/blocker and await actual
[Issue #13](https://github.com/shuhei-suzuki/rururunx/issues/13) outcome reconciliation.
Observation/audit metadata retain authority digests without copied Context Pack text.
Cleanup freezes the reserved class/phases, independent of later runtime policy capture.
Project risk-mapping recommendations can strengthen but cannot weaken runtime mappings.

Issue #14 must reconcile orphaned undispatched attempts, dropped futures/crashes,
inactive-owner preparation errors, definitive-decision publication conflicts,
marker Project/Goal-version conflicts (including Project/Goal metadata edits and
lifecycle ABA), release-token Record-version mismatches, untyped/unknown marker
errors, release CAS/executor-Lost fence failures, post-dispatch Session-binding
conflicts and unknown reversible Evaluating claims.

#41 adds no durable owner-absence proof. Project/Goal row versions participate in
marker, definitive-publication and post-dispatch Session-binding CAS. Future per-Task progress writers must use Task-scoped records
instead of bumping the shared Goal row on every step, or account for resulting
#14 recovery frequency among concurrent sibling Tasks. Ordinary observation must allow evaluation still being
in flight. A marker without a Session may precede any native launch and does not
prove launch occurred. #13 remains responsible for irreversible
Pr/MergeGate/Cleanup outcomes.
See [Issue #41 design](../issue-41-design.md).

Existing integration gaps are explicit. `retry` admits Failed unbound
dispatch markers without native-outcome evidence when no Session is persisted;
Issue #14 must fence this explicit API, while ordinary observation never replays.
The current successful native Session-binding publication writes an unchanged
Task via the coordinated transition and increments its raw version. Providers
admitted under that Task version can then reject their own completion/approval.
A separate atomic Record-only binding contract (#43) must preserve exact owner/context/
Session identity guards without incrementing unchanged Task fields; Issue #41
does not introduce that shared integration API.

## 18. Current native Session registration

Current main uses ordinary Task/Workflow persistence after adapter start returns.
It records the attempt's Session ID and rewrites Task, advancing its version.
That can invalidate the native adapter's captured scope during registration.
The record-only correction and its typed-input/admission integration are proposed
in [Issue43 design](../issue-43-design.md); neither the private binding port nor
its admission capability is implemented in main. Update this section to the
actual corrected contract only after the combined source is accepted.

Requirements9 also inventories two bound-live poll diagnostic writers: status error
and persisted/status mismatch currently refresh owners and use ordinary Task-writing
persist, invalidating native captured Task currency. Proposed #43 uses a separate
private record-only observe_native_diagnostic port for only these diagnostic deltas,
not the binder. It preserves all parent/Session/full-lock bytes/versions and native
pins, coalesces repeated diagnostics, and grants no binding/outcome/retry/release or
fresh authority. Actual managed held-turn controls and branch-specific Task-rewrite
mutants are required. This port/source is unimplemented, not part of merged #41;
terminal/authorized lifecycle/unknown recovery transitions retain their own gates.


Issue43 proposed Design7 additionally carries one immutable resulting marker frame
through actual native preparation/consumption/AllowCurrent, derives binding readiness
from marked/unbound/phase-open private operations, reserves per-Project operation
capacity atomically inside41reserve and parks only definitive Held observations without repeated binder
transactions. Driver and binder independently require actual runtime-instance match
or genuine14restore proof. These19/23/43/14 contracts remain unimplemented proposals;
merged41observer behavior and normal lifecycle authority are unchanged.

Capacity derives from active marked/unmarked native claims UNION actual open operations,
so41owned pre-marker release/invalidation naturally retires capacity and orphan claims
remain counted. Transient rollback-confirmed storage failures retry the same factual
binder fairly; definitive predicate mismatch parks. Uncertain commit reconciles only
durable facts. No added native dispatch, owner refresh or new binder write is allowed.
