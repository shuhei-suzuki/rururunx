# Production Workflow gate requirements

Risk: STRICT (shared Workflow and execution authority). Status: proposal for a
Phase2 component, not native acceptance or completion of Phase2. Requirements
and design precede production changes and receive independent review. This
component does not create a new human approval boundary within Phase2.

## 1. Purpose and sources

Replace fixture evidence for initial admission and implementation-result gates
with evidence of real managed preparation and retained commits. Preserve
[native result protection](agent-execution-requirements.md) R1–R9 and the
[Workflow contract](../design/master/workflow-engine.md). Source baseline:
`ed3c1078a0f5a9646a8c4991881179887b57b28e`. Issue #12 requires actual verification
evidence, not a successful native exit. ReviewEngine (#9), verifier execution
(#12), Runtime/CLI and remote publication remain separate required integrations.

## 2. Initial gate authority

G1. Issue checks the registered local Task specification: owning Project/Goal,
nonempty title/objective/acceptance criteria and exact committed instruction
digests. A local specification can satisfy this phase; it does not claim a
GitHub Issue was created, synchronized or independently accepted.

G2. Worktree requires the live, non-reconstructible preparation capability owned
by ManagedWorkflowSources. Check the actual registered Executor unit, bootstrap
phase, generation/epoch, profile, Task projection and exact clean base HEAD,
including common-Git ownership under the existing lease. A path string, ledger
row, fabricated marker or preparation-only work outcome cannot satisfy it.
Neither gate starts an Agent, consumes the first-unit adoption capability,
publishes an artifact or claims process recovery. Known changed/unsupported
inputs wait or refuse before native launch; operational ambiguity stays unknown.

G3. At evaluation, use the exact sole Workflow record, current Evaluating attempt,
generation/index/observation claim, Scope, Task version and immutable Context
version. Recheck these after all I/O before recording a successful receipt.
Context launch revision may precede an Executor-produced revision; every receipt
records both instead of requiring their equality for target-producing phases.

## 3. Implementation and commit gates

G4. Implement requires a known successful, owned managed terminal. Its exact
Session/unit/generation/epoch must agree with persisted Session and Workflow
binding. Require the source artifact to belong to that unit, with exactly the
observed revision and dependency frame. Verify its retained manifest, references
and complete object graphs. Native exit success alone never passes the gate.
No claim is made that acceptance criteria, tests or review have passed here.

G5. RequirementsCommit, DesignCommit and Commit check a Published retained
artifact from the immediately preceding corresponding completed Executor phase.
Its scope, revision, dependencies and unique artifact marker must agree. They
perform no live-worktree commit or branch-HEAD selection. Executor instructions
must produce commits; uncommitted drafts cannot pass. Existing atomic Workflow
publication remains the only acceptance transition; gates cannot bypass it.

G6. Requirements, Design and ImpactAnalysis additionally need a qualified
milestone content policy. This first component does not infer their completeness
from a native terminal or arbitrary prose. It returns Waiting for these phases
until their actual artifact policy is implemented.

## 4. Durable evidence and unsupported phases

G7. Successful gates store a bounded Verification record, with schema identifier,
Scope, phase, exact Workflow claim identity, Task/Context versions, launch and
observed revision, dependencies, checked unit/artifact and safe manifest digest.
Evidence references that record and the retained artifact where applicable.
Records survive restart and executor-path deletion. They are observations, not
launch permissions or reusable proofs: existing Workflow CAS, audited gate
observation and private publication proof still decide acceptance. Aborted/stale
observations may remain diagnostic; they must never complete a newer attempt.
Do not store credentials, environment dumps or unrestricted native output.

G8. All remaining phases return an explicit Waiting reason until their real
integration is installed: native review verdicts, tests/regression, mutation,
headed browser, staging, PR, merge and cleanup. In particular no successful
fixture, result artifact, exit zero or cleanup request stands in for these
different results. This component alone cannot finish a Task or satisfy #12.
Cleanup remains best effort and does not overwrite work outcome. rururunx is not
a security sandbox. No new ownership/ps observation, credential handling,
privilege, license change or authenticated native acceptance is in scope.

## 5. Verification criteria

Use actual managed source preparation, Workflow consumption and retained Git in
account-free isolated repositories. Demonstrate initial admission, same-unit
native handoff, real protocol terminal and retained publication through the
production gate; a protocol fixture is not an official CLI qualification.
Reject dirty/moved preparation, absent live capability, foreign/stale claims,
missing acceptance criteria, non-success/foreign terminal, corrupted manifest or
graph and wrong commit prerequisite. Prove unsupported phases wait rather than
pass, evidence is retrievable after reopening, and old worktree changes do not
alter accepted input. Compile causal mutations of preparation and terminal/
artifact checks and show actual test assertion failures plus restored controls.
Run affected regression, build, lint and format on a fixed clean commit; record
OS, source and limitations. macOS fixture results are not Linux/native evidence.
