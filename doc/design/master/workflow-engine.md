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
→ Relevant Verification
→ Review
→ Commit
→ PR
```

### STANDARD

```text
Issue
→ Worktree
→ Requirements
→ Requirements Review
→ Design
→ Design Review
→ Implement
→ Impact Analysis
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

Selection sources, highest precedence first:

1. project policy minimum
2. explicit stricter user choice
3. risk classifier recommendation
4. default workflow

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
the atomic transition API.
rollback leaves no orphan context or overwritten concurrent metadata. Blocked,
Removed, paused/terminal owners cannot progress. Mutation dispatch rejects active
review locks and all agent dispatch rejects reserved executors. This is a phase
runner library; scheduling, workflow CLI/TUI and actual GitHub publication/merge
ports remain dependent work. See [Issue #8 design](../issue-8-workflow-engine.md).
