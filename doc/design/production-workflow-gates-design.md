# Production Workflow gate design

Risk: STRICT. Status: pre-implementation proposal. Implements
[G1–G8](../requirements/production-workflow-gates-requirements.md) as an incremental
Phase2 integration. No additional human boundary or native qualification is
introduced. Source baseline `ed3c1078a0f5a9646a8c4991881179887b57b28e`.

## 1. Components and data flow

Add `execution::workflow_gates::ManagedWorkflowGates`, constructed with the same
RuntimeOwner and concrete Arc<ManagedWorkflowSources> that feed WorkflowEngine.
Implement PhaseGates. It has no public Passed injection, external command or
credential access. Existing PendingGates remains useful for explicitly absent
integrations. The Runtime will wire this concrete port; merely adding a library
does not provide an operational CLI.

```text
registered Task → managed source preparation → Workflow Context/Evaluating claim
  → initial preparation checks → bounded persisted receipt → audited observation
  → private same-unit adoption → managed terminal → retained source capture
  → exact terminal/artifact check → receipt → atomic Workflow/result publication
  → commit gate checks Published artifact → verifier/review integration (pending)
```

## 2. Exact claim inspection

Load the sole scoped Workflow record and typed WorkflowSnapshot under Store
mutex. Require invocation Task and Project to equal current persisted versions;
Goal is active. Active attempt must be Evaluating, exact phase, generation,
Context version, and claimed_observations equal current observation count.
Persisted Context must equal the invocation Context in full. Context includes
the launched revision; source includes the newly observed result for
target-producing phases. Compare current source to the invocation's observed
source using the managed producer before completing the receipt, preserving
mandatory rule and current Project/Goal digest checks.

Keep a private Claim containing record ID/version, active index, Workflow
generation, Task version, Context version and claimed observation count. Never
reconstruct an authority from just Task ID. Reload and compare exact Claim after
I/O under the same Store mutex before put_record. Recording an observation is a
local SQLite effect; stale errors do not permit external replay. WorkflowEngine
will independently recapture and validate sources, append its audited observation
and apply its existing transaction. Generic Verification records cannot mint
native grants or replace that transaction; a crash between receipt and observation
leaves a diagnostic receipt, not a completed phase. No schema migration is needed.

## 3. Initial preparation and specification

ManagedWorkflowSources provides a crate-private `verify_initial` method. While
holding its Task slot, require actual retained PreparedExecutor, matching Scope,
no artifact and an exact rendered SourceSnapshot for phase/budget. Do not take or
clone the capability. Add a crate-private PreparedExecutor namespace check which
uses its own provenance, validates current exact authority, loads current Task
and Project, validates resource profile, obtains the existing common-Git lease
and checks UnitGit ownership, exact base HEAD and clean tracked/untracked status.
Adoption repeats these checks after its own reservation; this initial gate does
not widen bootstrap launch privileges. All Git checks use registered UnitGit
helpers and their existing pending/confirmed receipts.

Issue requires valid local specification and matching instruction digests as
rendered by the managed source producer. Worktree uses the same source/capability
and physical namespace check. Missing specification returns a definitive Waiting
reason so the caller can update inputs and use fresh recovery; do not silently
revive old source capabilities after instructions change. Record checked unit
identity/profile/base without setting work, Session or artifact fields.

## 4. Implement and commit

Implement matches actual SessionStatus to persisted Session and native binding,
and checks successful WorkKnown Executor with native effects closed and result
finalization open. Check exact ManagedSessionRef and the current unit rather than
the volatile launch-time version (terminal legitimately bumps unit version).
The invocation artifact is Ready, belongs to that Executor, and matches observed
SHA and all source dependencies. ResultStore.verify performs registered retained
graph/manifest/ref checks. Recheck claim, unit and artifact after I/O. Return
exactly one `rrx-artifact:<id>` marker along with the gate receipt. Engine's
WorkflowPublication performs its own private verification and atomic acceptance;
do not call ResultStore.publish separately or invent terminal success.

For the three commit phases require the corresponding completed Executor
evidence (Requirements, Design, Implement), not a random older Task artifact.
It must have one unique artifact marker; current source must match this artifact
and the prerequisite revision/source frame. Require Published state, verify
retained data, and record artifact identity/digest. No native Session exists for
the commit gate; evidence.session_id remains None. Requirements/Design/Impact
content validation is deliberately pending rather than approximated by exit zero.

## 5. Receipt and phase coverage

Verification record schema `managed_workflow_gate_v1` uses bounded JSON:
claim, phase/Scope, observed Source authority excluding payload, launched Context
revision and SHA256 of full serialized Context data, checked unit identity,
optional artifact ID/manifest SHA256, and local specification summary digest.
Evidence uses `rrx-gate:<RecordId>` plus an artifact marker where applicable,
Context version and Session from the current claim, and all source versions as
dependencies. Values use existing source bounds and avoid native output secrets.
The persisted record is retrievable through existing Store.record; audit and
Workflow are the authoritative state, not string parsing of this diagnostic.

| Phase | Qualified behavior in this component |
| --- | --- |
| Issue | Registered local Task specification and real preparation; no remote claim |
| Worktree | Live capability, resource profile, ownership, clean exact base |
| Implement | Owned successful terminal, exact retained Ready artifact |
| RequirementsCommit / DesignCommit / Commit | Corresponding completed Executor's Published artifact |
| Requirements / Design / ImpactAnalysis | Waiting for qualified content policy |
| Review phases | Waiting for ReviewEngine; never infer approval |
| Tests / ExpandedRegression / Mutation / Browser / Staging | Waiting for actual verifier evidence |
| PR / MergeGate / Cleanup | Waiting for qualified external/reclamation integration |

## 6. Impact and verification

Consumers: PhaseGates::complete, Workflow evaluation claim/observation/resume,
managed source cache/digests, bootstrap preparation/adoption, native terminal
identity, artifact publication and retained inspection. Do not change evidence
validation to admit weaker source/session identity. Existing fixture gates may
remain in legacy controls, but add tests invoking the shipped gate in the actual
Workflow through first Executor publication and subsequent commit.

Negative controls cover preparation path mutation and lost capability, missing
Task specification, forged/stale invocation/transport/artifact, retained
corruption, prerequisite mismatch and unsupported phase wait. Fixed controls and
compiled mutations test production checks. Verification records are observations
only; no new cross-connection authority claim is made. Native/OS/parallelism
qualification and #12 verifier acceptance remain outstanding after this component.
