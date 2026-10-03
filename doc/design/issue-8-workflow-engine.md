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

QUICK: Worktree, Implement, Tests, ImplementationReview, Commit, PR.
STANDARD: Issue, Worktree, Requirements, RequirementsReview, Design, DesignReview,
Implement, ImpactAnalysis, Tests, ImplementationReview, Commit, PR, MergeGate,
Cleanup. STRICT inserts SecurityReview, ExpandedRegression and Mutation; browser
and staging phases are configurable and require separate evidence. Security and
mutation evidence are never synthesized.

Risk R0 recommends QUICK, R1/R2 STANDARD, R3 STRICT. Effective selection takes the
maximum of configured default/minimum, stored Task workflow, requested stricter
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
