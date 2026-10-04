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
native side effects. #12 approval and #13 recovery remain pending; uncertain phase
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
dispatch marker distinguishes undispatched reservations from possible launch interruption.
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
terminal reservation release keeps Task/decision/context/evidence immutable and requires
owned persisted termination plus all executor/Lost fences. Unknown outcomes or unbound
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


## 18. Native Session binding authority

A successful native launch may return a persisted Session before the Workflow attempt has
recorded its `session_id`. That factual registration must not invalidate the native
adapter's already-captured Project/Goal/Task/lock currency.

Workflow therefore uses a narrow crate-private Session-binding transaction rather than
the ordinary Task/Workflow persistence path. The transaction rechecks exact captured
Project, Goal, Task and Workflow versions; active scope/generation/attempt; committed
dispatch marker; context identity; scoped lock currency; and the exact persisted returned
Session identity. On success it changes only the active Running attempt's absent
`session_id`, advances the Workflow record version and appends one factual audit. Task,
Project, Goal, Session, Context and WorktreeLock bodies/versions are preserved.

Binding is StateOnly factual owner registration after native launch, not new-operation
admission. A visible Workflow SessionId is not a private prepared-input allocation
credential. Provider-native authority remains unchanged: Claude reservation currency,
Grok Actor ownership checks and Codex approval scope/Session guards continue to decide
their own callbacks/replies/completion. Binding failure preserves the existing reservation
and any live Session; it never releases, retries, transfers ownership or fabricates a
native result. See [Issue #43 design](../issue-43-design.md).
