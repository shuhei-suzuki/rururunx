# Workflow verification integration requirements

Risk: STRICT (Workflow acceptance, execution grants, retained evidence and SQLite
writer authority). Status: proposed Phase2 implementation supplement; no verifier,
browser, staging or native qualification follows from this document. Independent
requirements/design review precedes production changes. This component adds no
human phase boundary beyond the existing Phase2 boundary.

## 1. Purpose and baseline

Implement Issue #12's impact, tests, regression, mutation, headed browser and
staging evidence through the actual Workflow consumer. Source baseline:
`a79a3051d41bc122e1e7e88cb7399c3cdfb83787`. The saved Issue #12 inventory requires
consumer searches, impact-based regression selection, a causal mutation with
restored controls, a deterministic headed backend without an LLM, conditional
staging and retrievable evidence references. It depends on #8/#9/#19/#20; this
supplement does not declare those dependencies complete.

Preserve [result protection](agent-execution-requirements.md), the
[Workflow contract](../design/master/workflow-engine.md) and the
[initial production gates](production-workflow-gates-requirements.md). The current
EvidencePort phases have no Agent Session. Existing readonly native completion
does not accept them; a command-only verifier grant and typed acceptance are new
work. Tests returning public `Passed`, a generic Verification record or exit zero
from an unrelated helper cannot certify a production phase.

## 2. Policy, applicability and impact

V1. Resolve a bounded, immutable verification plan from committed Project rules,
Task acceptance criteria, risk classification and the current source frame. Each
required category has approved commands or a qualified backend, explicit inputs,
success criteria, limits and prerequisites. Runtime/operator command approval is
distinct from Agent-authored configuration or suggestions. Committed code may be
executed only through an explicitly admitted Project tool profile. Do not execute
arbitrary strings as a shell, silently substitute commands or treat an empty plan
as success. Required incompatible hooks/tools hold before effects.

V2. Tests cover configured test, typecheck, lint and build obligations separately.
ExpandedRegression adds selected impacted-consumer tests and Project-required
broader controls. QUICK still has relevant verification. STANDARD/STRICT require
impact evidence before Tests. Required browser/staging obligations cannot vanish
because a preset flag is false or an Agent says they are unnecessary: escalate
monotonically and recompute the phase/context plan before dispatch. Optional gates
may be absent only with a persisted policy-bound applicability decision. Unknown
classification or missing mandatory commands/prerequisites waits, never passes.

V3. Impact evidence binds the exact implementation commit, base, changed paths/
contracts, actual bounded consumer-search operations, searched scope, affected
locations, non-impact rationale and selected regression IDs. Native proposals are
input to validation, not authority. Search failure, truncation, unsearched required
scope or an unresolved omission prevents certification. Analysis does not claim
exhaustive semantic correctness: required independent review checks its reasoning.
ImpactAnalysis must not silently change the implementation target. Source changes
return to implementation and invalidate analysis, tests and reviews.

## 3. Execution and readonly input

V4. Before any preparation/helper/command effect, atomically register an exact
Evaluating Workflow claim, approved plan and command-only Verifier unit. Bind
Scope, Task execution generation, owner epoch, sole Workflow ID/version/index,
phase, Context version and complete Context digest, observation claim, prerequisites,
governing policy and Published retained artifact/manifest/dependency versions.
Use a private grant; public DTOs, ordinary unit reservation and Session labels
cannot create it. Verifier admission does not advance executor generation or
change Task worktree/branch. It consumes execution capacity/resources, not a
provider subscription or synthetic Agent Session.

V5. Give each run a fresh independent exact-SHA readonly ResultSnapshot and
separate output/temp/cache/ports/managed Docker namespace. Never run verification
in a live Executor worktree or reuse a cancelled/retried snapshot. Check graph,
manifest, source permissions, HEAD, tracked digest and cleanliness before/after
execution and before acceptance. Approved source-writing tools are incompatible
with this profile. Mutation uses a separately declared disposable derived input;
the baseline remains readonly. Same-user raw bypass/transient mutation and
arbitrary shared external effects remain uncovered. rururunx is not a security
sandbox; no process-death guarantee is introduced.

V6. Every command/probe/backend action gets a durable scoped intent before spawn
or external dispatch. A qualified collector observes its actual child/backend
completion, exact argv/profile/CWD/inputs, exit or signal, timing and bounded
stdout/stderr/artifact references. The verifier uses finite configured wall,
drain and output limits, current authority fences and owned-process cancellation;
it does not inherit the Git helper's fixed 60-second/ stdout-only contract.
Missing executable, incompatible tool/output routing, display, dependency or
staging authorization waits before the affected effect. No held SQLite lock
spans an await or long command.

## 4. Specialized proofs

V7. Mutation requires a successful baseline control, an explicit intentional
patch bound to the baseline commit, actual successful build/setup as applicable,
the expected behavioral test assertion failure, and an unmodified restored control
that passes. Compile/setup failure, timeout, signal, output loss or an unrelated
test failure earns no mutation kill credit. Preserve mutant/restore identities,
commands and diagnostics. A fresh independent restored input avoids reuse of a
mutant namespace that might still be written by survivors.

V8. Browser verification is provider-neutral. At least one qualified deterministic
headed backend operates without an LLM and records actual owned browser launch,
headed mode/display, target, assertions, backend/version/profile and evidence
digests. A caller's `headed=true`, exit zero or screenshot path is insufficient.
Adaptive/agentic backends (#31) are optional and cannot replace a mandatory
deterministic proof. Browser binaries/display and Project app dependencies are
explicit external prerequisites, not claimed to ship through `cargo install rrx`.

V9. Staging is required for configured API/auth/DB/infra risk classes. Resolve an
explicit admitted non-production target, resources, allowed actions, assertions
and existing authorization before dispatch. Deny implicit production targets,
global teardown and sibling resource cleanup. Record external operation identity
and acknowledgement/confirmation separately. An unknown acknowledgement cannot
be retried automatically unless a qualified idempotent reconciliation establishes
the exact prior effect. Do not inspect, copy or log credentials; tool-owned auth
and configured reference availability remain prerequisites.

## 5. Completion, evidence and recovery

V10. Work and cleanup remain two axes. Known child exit is retained independently
of group/pipe cleanup. All required successful commands/assertions, complete
diagnostics under the qualified limits, exact artifact and readonly validation
are needed for verification success. Known failed assertions fail verification;
unobservable completion or invalid/incomplete evidence is unknown/non-certifying.
Timeout/cancellation cannot certify success even if a child later exits zero.
Cleanup leftovers/unknown never overwrite known work or alone block certification
of protected immutable inputs/evidence. Report the coverage limits explicitly.

V11. Persist bounded immutable verification receipts and content-addressed raw
evidence before notifying observers. Context/ReviewBundle refer to the exact
receipt, commit, plan and safe finite summary; full logs are retrievable through
validated artifact references, not duplicated in prompts. Raw evidence is private,
potentially sensitive output; no environment dumps, credential discovery or raw
automatic publication. Bound counts, sizes, parsing and retrieval; truncated
required evidence cannot be certified. Generic record/ref text cannot mint proof.

V12. Only the private qualified producer can mint command completion and readonly
verification proof. Workflow accepts that proof in an exact CAS transaction that
rechecks current claim/Context/owners, required plan coverage, unit generation,
terminal receipts, artifact and dependencies, records the accepted receipt IDs,
and closes result-finalization. Ordinary `put_workflow_transition`, forged public
`GateOutcome::Passed`, old connection writers and receipts from another run cannot
complete a managed verification phase. A receipt observed before a crash is not
accepted phase authority. Reopening is read-only unless current recovery authority
revalidates the exact retained inputs/receipts and wins the current claim CAS.
Unknown in-flight commands are not reconstructed as successful or blindly resent.

## 6. Acceptance scope

Demonstrate actual Workflow EvidencePort consumption in isolated account-free
repositories: real Published inputs, real commands/assertions, private grant and
acceptance, diagnostic retrieval after reopen and executor-path removal, cancel/
restart/stale races, required coverage and sibling isolation. Use causal compiled
mutations of claim, terminal, readonly, applicability and proof-source guards with
runtime assertion failures and restored controls; compilation failures do not
count. Run relevant regression/typecheck/lint/build on a fixed clean commit.
Label OS/tool/backend evidence precisely. Deterministic browser/staging and native
four-Task/auth/hooks/subscription checks remain unverified until actually run;
fixtures never establish those claims. No license or README Status change occurs.
