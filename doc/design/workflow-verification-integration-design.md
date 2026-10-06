# Workflow verification integration design

Risk: STRICT. Status: proposed Phase2 supplement for
[V1–V12](../requirements/workflow-verification-integration-requirements.md); no
production implementation or native qualification. Source baseline
`a79a3051d41bc122e1e7e88cb7399c3cdfb83787`. Independent review precedes changes.
This extends [production gates](production-workflow-gates-design.md), not their
current qualified coverage. The original reviewed migration was unallocated.
Subsequent coordination assigns the first command-only Tests delivery schema8,
after Native results6 and Published-frame recovery7; future ReviewRound uses9.
Compose those actual preceding fixed migrations before shared schema edits; do not
publish an independently numbered or partial migration. Other verification
categories remain explicit pending integrations until implemented and qualified.

## 1. Actual consumer and proposed components

At the baseline, WorkflowEngine enters Tests/ExpandedRegression/Mutation/Browser/
Staging as Actor::EvidencePort and directly evaluates a Running attempt with no
Session or unit. Its evaluation claim is persisted before PhaseGates::complete.
ManagedWorkflowGates returns Waiting for these phases. AttemptManager can reserve
a standalone Verifier but prepare_workflow_snapshot hardcodes Reviewer;
checked_workflow_binding admits Running Executor/Reviewer only. ResultSnapshot's
private completion requires a successful native terminal; Workflow/Store readonly
acceptance requires Actor::Reviewer and a Session. UnitGit's registered collector
captures bounded stdout, discards stderr and uses fixed helper limits. None of
these paths currently certifies command verification.

Add crate-private components (names are proposed, not existing APIs):

- VerificationPlanner: committed policy/applicability and approved tool profiles.
- VerificationGrant: nonserializable exact claim/run/snapshot admission authority.
- ManagedVerifier: fresh snapshot, registered command/backend intents and collector.
- VerificationCompletion: private retained-input/receipt verification proof.
- ManagedVerificationGates: dispatch/collect this producer through PhaseGates.
- Store verification reservation, terminal receipt and accepted-phase transactions.

Public callers may request evaluation, inspect receipts and register proposed
configuration. They cannot construct grants/completions, assert command success,
or pass a verification record into an acceptance API. Keep the workspace unsafe
forbid policy. No native adapter, credential or ps-observation extension is needed.

```text
Published implementation SHA + committed policy + actual Workflow Context
  → applicability/approved plan → exact Evaluating claim → private Verifier grant
  → fresh readonly snapshot → intent before each effect → real collector receipts
  → retained evidence + final readonly checks → private verification completion
  → audited observation + exact atomic Workflow acceptance → immutable refs
```

## 2. Plan, permissions and phase coverage

Freeze VerificationPlan identity/digest, Scope, target artifact/revision/base,
source/dependency versions, full Context digest, policy and applicability evidence.
Each command has an ID/category, canonical admitted executable/profile identity,
argv array, source-relative CWD, input refs, finite wall/drain/output limits and
typed success criterion. Commands are sequential initially; no arbitrary pipeline,
shell interpolation or detached command is certified. A repository script is
allowed only as an explicitly admitted executable tool profile whose trusted
configuration accepts its scope and output contract; committing a script does
not automatically grant permission. Required project skills/rules come from the
managed committed frame and its mandatory rule digests, not the live rule loader.
Skill text may supply proposals; it cannot widen executable/network permissions.

The operator-approved profile catalog is activated separately from Task-authored
files. A Task can narrow an approved command selection, subject to mandatory
coverage; a new executable/action or changed privileged/network profile needs the
existing policy approval path. No per-command human interruption is added for
already authorized routine tests. Missing approval is a typed Waiting reason.
Resolve program/CWD symlinks and routing before intent, revalidate immediately
before spawn. Qualified profiles explicitly declare external dependencies,
output routing and supported executable/version families; unsupported source
writers hold. Use external target/output/cache paths, locked dependency behavior
where supported and private/disabled supported daemons; preserve required hooks
and reject incompatible ones rather than silently disabling them. Do not invent
compatibility for Cargo lock updates, npm installs or other source-writing tools.

| Phase | Required plan/proof |
| --- | --- |
| Tests | Relevant tests plus independently required typecheck, lint and build |
| ExpandedRegression | Impact-selected consumer tests plus configured broader suite |
| Mutation | Baseline, intentional patch, causal failure and restored control |
| Browser | Qualified deterministic headed backend and required assertions |
| Staging | Approved exact non-production target/actions and assertions |

No successful category substitutes for another. Non-applicable optional commands
have a typed rationale bound to the classifier/plan; required commands cannot be
skipped. Empty Tests or ExpandedRegression is Waiting unless the qualified Project
policy explicitly declares the category non-applicable with evidence. Required
browser/staging risk escalates the Workflow preset/config monotonically before
starting verification; update the Context and invalidate affected observations.
Existing config booleans are not a bypass. A policy conflict/unknown risk holds.

## 3. ImpactAnalysis and selection boundary

Issue #12 requires an impact artifact, while ImpactAnalysis currently remains an
Executor phase before Commit. Keep that actor and existing owned native terminal/
publication checks; do not reinterpret a generic native answer as verified impact.
For this phase, freeze the completed Implement artifact SHA as the analysis target.
Pin the Task's approved initial input commit as the comparison base and retain
its complete graph; do not use the newly prepared ImpactAnalysis unit's base
(already the implementation SHA) or a moving branch. Include earlier milestone
changes and changed policy/rule files in classification. An absent or ambiguous
original baseline holds rather than certifying an empty diff.
Analysis proposals and search diagnostics go to the unit's separate output, not
new committed source. Capture/publication may retain the same exact SHA through
the ImpactAnalysis unit; Commit still requires the corresponding completed
Implement SHA. If the analysis Agent changes source or proposes a fix, reject
analysis certification and request a fresh Implement generation before reanalysis.
An impact artifact does not become its own source commit, avoiding a self-hash loop.

Add a typed ImpactReport producer under this Executor's current phase authority.
Its proposals name changed paths/contracts, affected consumers and non-impact
rationale. Bind proposals to the actual owned native result receipt and strict
bounded schema; this depends on the Native result acquisition component, not
arbitrary Session text or a caller-provided report. Runtime derives changed paths
from exact retained base/target Git
objects and executes approved bounded search plans against that exact input,
recording search argv, locations, complete result refs and coverage. The immutable
report binds target SHA, search receipts, classification, required domains and
selected stable test IDs in the approved catalog. Native proposals are validated
against those receipts/catalog and independently reviewed; they cannot declare
searches complete or add shell commands. A search with zero matches records that
actual outcome; exit semantics are tool-profile-specific, not generic failure.

Required scope must be covered or explicitly unresolved; output overflow, failed
search and missing consumers prevent pass. Soundness of semantic non-impact
reasoning remains an independent review obligation, not a Runtime proof. QUICK
uses mandatory relevant-test policy and bounded changed-path classification even
without an ImpactAnalysis phase; unknown impact escalates/holds. Tests and broader
regression consume the report bound to their exact Published target. If the report
or its typed producer is absent, STANDARD/STRICT wait; this supplement's test
commands alone cannot complete those workflows. Requirements/Design milestone
policies, PR/merge and cleanup remain separate pending integrations.

## 4. Exact claim and command-only reservation

Reuse the production gate's full invocation checks: current Task/Project DTOs,
active Goal, sole Workflow ID/version, exact active Evaluating phase/index,
Workflow generation, claimed observation count, full stored Context equality,
exact prerequisites/prior observations, and freshly rendered managed source.
Add exact plan, Published artifact/manifest, Task execution generation, epoch,
instruction/policy digest and dependency frame. Reload this Claim after every
preparation/command I/O; do not reconstruct it from Task ID or revision alone.

Add a distinct Store reservation transaction for an Evaluating EvidencePort
claim. It registers a fresh UnitKind::Verifier with immutable command-only
purpose/run ID and approved plan digest, current generation/epoch, source artifact
and ResourceProfile; writes its ManagedUnitRef into that exact attempt and returns
a private VerificationGrant. Task worktree/branch, executor generation and native
Session remain unchanged. This authorized internal write deliberately bumps
Workflow version: return the exact successor claim to the trusted engine, which
reloads it before subsequent observation/acceptance. It must not mistake its own
authorized reservation for a foreign stale invocation or broadly ignore version
changes. The public PhaseInvocation alone cannot perform this adoption.

Do not hide this mutation inside the existing `PhaseGates::complete` return value:
it carries only a public GateOutcome. Add a trusted crate-private engine/producer
entry point returning the exact reservation successor and optional private
completion. Engine reloads and compares the original claim, authorized successor
and all untouched fields before its ordinary source recapture/observation step;
its next expected version includes only those enumerated internal writes. Error/
Waiting results retain that successor too, so they can record diagnostics without
replaying preparation. Existing public PhaseGates stays a legacy/diagnostic port,
whose Passed cannot certify this managed path. Record the managed verification
contract immutably at Workflow activation, before any verifier unit exists, and
check it in every Store success writer; omitting a unit must not bypass the guard.
The managed source producer supplies the original baseline/graph pin at activation
or a private exact bootstrap-adoption transition, before implementation starts;
caller revision strings and a later impact report cannot replace it. Recovery must
revalidate that pin through its explicit retained-frame capability. These fields/
guarded transitions require the coordinated migration.

Command-only purpose is enforced at reservation, helper/tool grant, Session/native
registration and completion boundaries. Existing public prepare_snapshot and
unit labels cannot mint VerificationGrant. Deny Agent Session/native adapter
registration for these units even if a legacy effects-open flag is true. A new
private command terminal producer observes the command set and closes effect
authority; existing native ReadonlyCompletion is not reused to fabricate native
success. Store validation recognizes purpose and rejects old writer contracts.
Do not grant arbitrary UnitGit commands or historical artifact readers launch
authority. Capacity counts this unit through preparation and active commands;
subscription permits remain reserved for actual provider invocations.

Register all worktree/resource/preparation intents before materialization. Use
ResultStore.snapshot for a separate retained exact-SHA clone without alternates
or shared object routing. Preserve its private provenance. Verify prior and final
source mode/hash/HEAD/cleanliness and retained graph/manifest, outside ledger locks.
Fresh retries get new unit/run/path/resources. Cancellation/replacement fences
this run; retained artifacts stay usable and sibling Tasks' authority is unchanged.

## 5. Real command lifecycle and finite diagnostics

Suggested durable states: Planned → Admitted → Running → Terminal → Inspected →
Accepted; Held/Cancelled/Unknown are explicit outcomes, not success aliases.
Each command receipt starts NotDispatched. Under a short Store transaction/lock,
recheck grant, lease and current claim, write the pending scoped operation intent,
and spawn the actual owned process without an intervening await. If spawn fails,
record that failure separately from an unknown dispatched effect. All app/server/
browser/dependency probes follow the same admission; no unregistered helper gap.

An independent verifier collector keeps actual owned child identity until reaped,
captures both streams, applies monotonic wall and drain deadlines, fences authority
at bounded intervals and requests owned group termination on cancellation/timeout.
It rechecks immediately before each effect and receipt. It never kills arbitrary
numeric PIDs or broad shared daemon names. Group/pipe cleanup is best effort and
records requested/leftovers/unknown separately. A direct child's exit status and
known assertions are persisted even when output drain/cleanup fails. Such a run
has known work with incomplete verification evidence, not fabricated full success.
Killed/timed-out/cancelled commands cannot be counted as successful later.

Default maximums for the initial finite profile: 32 commands/run, 128 argv entries/
command, 32 KiB encoded argv, 1 MiB total plan, 64 KiB summary/receipt metadata,
32 MiB each stdout/stderr per command, 256 MiB total raw evidence/run, 128 evidence
files/run and 32 MiB/file. Parsing depth ≤32 and strings/counts receive explicit
bounds before allocation. Configured wall timeout is 1 second–2 hours; drain is
1–10 seconds, cancellation fence ≤100 ms, reap wait ≤10 seconds. Profiles may
lower these maxima; raising them needs a separately reviewed profile. Limits are
proposals to test, not measured response-time claims. Capture bounded bytes to
private exclusive files; hash exact bytes, fsync/atomically finalize protected
evidence before committing references. Collector-owned capture file descriptors
are not inherited by children. Copy admitted output/backend files into independent
Runtime retained evidence storage outside every unit's writable output/temp/cache
before finalizing; a path into an old output directory is not an immutable ref.
Accepted metadata contains content digests and cannot be overwritten through
generic record writers. Never follow evidence symlinks or allow
path escapes. Overflow stops the operation and preserves a marked finite prefix;
it is non-certifying, not silent truncation. Missing disk/stream/file or malformed
required backend evidence similarly prevents certification.

Persist command identity, intent, actual dispatch/exit or signal, start/end and
deadline, assertion result, stream completeness, file refs/digests and cleanup
coverage. WorkOutcome is actual known success/failure/unknown; VerificationStatus
additionally distinguishes certifying pass, failed assertion, unsupported and
incomplete/unknown proof. For example exit zero plus lost stderr is known child
success with non-certifying evidence. Negative exit does not alone prove a causal
mutation. Raw logs can contain secrets despite no credential inspection: store
privately, summarize using safe finite fields, publish only explicitly selected
redacted evidence. Do not read inherited auth/config files or log environment.

## 6. Mutation input and causal credit

The readonly baseline is never chmod-writable or patched. First run the exact
required original control on a qualified baseline snapshot. Materialize a fresh
private derived mutation namespace from that same retained graph, with an explicit
bounded MutationManifest (base SHA/tree, changed path/blob identities, patch digest,
selected control, expected assertion signature). Only this declared namespace is
writable for the mutation producer; no reviewer/verifier baseline privilege is
widened. Record every preparation/mutation effect before execution and check that
the resulting diff is exactly the declared change. Undeclared source writes fail.

Run required build/setup successfully, then the selected test must fail at the
declared behavioral assertion. Compile errors, missing executable, unrelated
failures, timeout/signal or incomplete diagnostics give zero kill credit. For
tools without a qualified assertion classifier, retain failure diagnostics but
hold for causal verification; an Agent's prose cannot mint credit. Record real
control IDs, invocation and diagnostic references, not just expected exit codes.

Restore by preparing another fresh independent readonly namespace at the original
SHA/tree; do not trust patch reversal in the old mutant path. Verify exact clean
HEAD/manifest/tree, rerun the original control successfully and record these refs.
This provides restored-control evidence while the dirty mutant is quarantined
for cleanup. It does not claim the old mutant path was repaired or emptied. A
surviving mutant process has no authority over the restored namespace or retained
proof; same-user deliberate cross-path writes remain outside this profile.

## 7. Deterministic headed browser and staging

Proposed first browser backend: a Runtime-owned deterministic Chromium/CDP driver,
not an LLM. Its qualified profile declares the canonical supported browser binary/
version family, real display/session requirement, fresh private browser profile,
private debugging port and bounded scripted navigation/assertions. Qualify this
backend on macOS and Linux before advertising support. Missing binary/display or
required Project app dependencies is Waiting before browser/server effects; a
headless fallback cannot pass a headed obligation. `cargo install rrx` supplies
the runner, not browser binaries, a graphical session or Project dependencies.

The private driver registers actual app/browser process intents and protocol
actions, requests a real headed launch, binds the owned launched instance to its
endpoint/context, performs the admitted assertions, and retains screenshots plus
safe finite navigation/assertion/backend diagnostics. Reject foreign/reused
endpoints, incomplete launch handshake and caller-provided headed reports. A
Project test may contribute assertions through its approved profile, but its JSON
is not headed-launch authority. Remote browsing outside admitted target origins,
downloads or broad host permissions require a distinct profile. Other deterministic
backends can implement the same typed contract; adaptive #31 stays optional.

StagingPlan freezes exact non-production target identity, Project/Task namespace,
approved action set, required assertions, test data/resource allocation and tool
profile. Runtime owns action admission and records idempotency keys plus target/
resource digest before effect. Operator/project authorization may already cover
routine staging actions; missing/new destructive or production authority waits.
Use ordinary tool auth without inspecting, copying or logging credentials.
Staging cleanup never uses a broad Task label alone for unrelated external effects.

Unknown create/deploy acknowledgement is reconciled against its recorded target/
operation through a qualified idempotent backend; otherwise hold and expose it.
No automatic second deployment or assumed rollback follows from a timeout.
Successful test assertions and exact target provenance must be confirmed; teardown
has its own outcome and cannot overwrite them. Target unavailable/unknown/production
or shared destructive fixture configuration never passes staging. An implementation
may initially return explicit Waiting for unsupported backends, but must keep
staging obligations visible and cannot claim Issue #12 complete until qualified.

## 8. Durable proof and atomic Workflow acceptance

A VerificationReceipt stores run/plan/claim identities; Scope/phase/Context digest;
artifact/revision/manifest/dependency frame; verifier unit/generation/epoch/profile;
required coverage, applicability/impact refs; immutable command/backend receipts;
readonly validation digest; bounded summary, raw refs and work/cleanup axes.
Use existing RecordKind::Verification as a diagnostic projection and Context/
ReviewBundle refs where suitable. Generic Record APIs do not produce qualified
terminal, readonly or acceptance authority. Add coordinated typed ledger storage/
indices/triggers for immutable command-only purpose, dispatch and terminal receipt
ownership, accepted receipt refs and old-writer contract exclusion. Schema number
and concrete tables are assigned with implementation ownership, not in parallel
by this document. This is a required migration, not a claim that generic records
alone provide cross-connection protection.

Only the actual collector/driver, operating under a private grant, can commit a
qualified terminal receipt. Commit known work, closed effect authority and receipt
atomically before watch notification. Private snapshot inspection then mints
VerificationCompletion bound to exact immutable receipt digests and required plan
coverage; proof values cannot be deserialized. Preserve finalization until phase
acceptance or explicit cancellation. A successful gate observation refers to this
completion but is still not accepted authority.

Extend the actual Workflow persist/evaluate path for EvidencePort units: it must
obtain this private completion, not call the current Session-based readonly
completion or fall through to ordinary put_workflow_transition. Store's new
accepted-phase transaction rechecks sole record and exact expected successor
claim, Task/Project/Goal/current epoch/generation, complete Context, plan/coverage,
unit purpose/phase/terminal, Published artifact/full dependencies, receipt version/
hashes and exact expected phase transition. Reverify protected retained evidence
bytes/digests immediately before minting completion; missing/corrupt evidence
cannot be accepted from stored metadata alone. Require session_id None and
review_approved None; a verifier does not approve review. Atomically write accepted
receipt references, audited observation, succeeded phase/next Context, close unit
finalization and queue best-effort cleanup. Concurrent cancellation/version/source
drift loses CAS without completing a phase. Generic managed EvidencePort success
without this proof is rejected by Store, including older open writer connections.
Failure/Waiting diagnostic transitions retain their ordinary non-acceptance path.

Evidence references are typed scoped run/receipt IDs plus content digests and
category, not arbitrary host paths or fetched URLs. Context/ReviewBundle contain
the exact accepted commit, plan/receipt identities and safe finite summary in both
Context Efficiency ON/OFF modes. Mandatory result/coverage refs cannot be trimmed;
expansion retrieves the exact immutable bytes under Scope and size bounds, without
reading live executor logs or substituting a newer run. Historical inspection
needs current read-only retention authority, not old native permission. Record
reclamation pins separately from process cleanup and release them only through
the explicit result-retention policy.

Workflow's own reservation and observation writes have explicit trusted expected
successor versions. Retain immutable origin run/claim and ordered observation
identity instead of relaxing equality to accommodate internal updates. Never
accept a previously closed/replaced claim, another phase's receipt or another
Task's successful commands. A duplicate accepted request returns the existing
accepted identity after exact equality checks; it never reruns commands.

## 9. Restart, cancellation and surviving writers

Cancelled/replaced units close effect and finalization authority before asynchronous
cleanup. Drop guards record unknown operations and schedule cleanup; they cannot
publish a pass. After Runtime SIGKILL, durable Running/pending operations remain
unknown until reconciled. No original child wait status is invented from PID
absence, a log suffix or a marker file. New epoch grants cannot revive native
permissions or blindly replay staging effects.

Accepted receipts are historical facts: inspection under the current epoch verifies
exact retained artifact/evidence without changing accepted work. For complete
terminal receipts whose phase was not accepted, recovery may revalidate immutable
proof under an explicit current claim/recovery capability and exact CAS; it cannot
reuse old executable grants. If required provenance, current Context/policy or
namespace protection cannot be re-established, retain historical diagnostics and
start a fresh run on explicit safe retry policy. Raw paths/record IDs never restore
proof. Retained inputs/evidence stay pinned through acceptance/recovery; the janitor
cannot delete them based solely on a closed process or stale unit age.

Executor/mutant leftovers can still write their old namespaces. Fresh independent
snapshots, exact retained commits, protected finalized evidence and private typed
CAS prevent those ordinary writes from changing accepted results. Cleanup is
reported separately; no all-process absence, security confinement or external
shared-service isolation is asserted. Unsupported shared state/absolute paths/
raw sockets are explicit coverage exceptions or incompatible required profiles.

## 10. Implementation sequence and acceptance evidence

1. Coordinate writer contract; implement typed plan/applicability and claim-only
   Verifier reservation/adoption, preserving actual Workflow successor versions.
2. Add registered command collector, finite immutable evidence and command-only
   terminal; actual Tests and ExpandedRegression proof/Store acceptance.
3. Add qualified ImpactReport binding/selection and mutation-derived input/credit.
4. Add deterministic headed driver and explicit staging backend contracts; hold
   unsupported profiles without dropping obligations.
5. Wire recovery, Context/ReviewBundle/inspection refs and Runtime readiness ports.

Expose durable run state/next action to the scheduler's typed readiness consumer;
long commands do not hold the single Runtime driver or require polling Session
watch state. A driver claim admits one exact run, and completion observation causes
a new eligibility decision under current capacity/epoch. An absent verifier port
has a precise pending-integration reason; it never routes these phases to a native
Agent or turns the remaining Workflow into complete. Runtime/CLI integration is
still pending at this source baseline.

Use actual Workflow/ManagedSources/Published ResultStore consumers, not fixture
Passed injection. Controls include separate test/lint/build obligations, missing
command/output/dependency, failed assertions, output overflow, timeout, cancellation,
dirty/moved readonly input, claim/Context/full-DTO/source drift, generic/foreign/
old receipt forgery, cross-connection stale writer and replay-after-acceptance.
Check causal mutation versus compile/setup failure and restored independent control;
headed launch versus headless/foreign endpoint; staging unknown acknowledgement
versus confirmed idempotent reconciliation; policy flags versus mandatory risk.
Crash cuts cover intent-before-spawn, terminal-before-watch and proof-before-CAS.
Reopen evidence after deleting the original Executor path. At least four scoped
command Tasks include an ordinary detached survivor and cancelling one must leave
sibling commits/accepted receipts unchanged. Compile mutations of actual guards,
obtain runtime assertion failures and restore clean controls before broad checks.

Source controls on one OS/backend do not qualify another or official Agent auth/
hooks/subscription behavior. All native four-Task validation belongs to Phase3;
README Status changes afterward. The deterministic browser and real staging matrix
must name OS/backend/version/display/target and unsupported combinations. Keep
license files unchanged and do not claim process collection or full MVP completion.

## 11. First Schema8 command-only delivery

The first delivery implements the actual managed Tests consumer. It does not
complete Issue12. Impact selection, ExpandedRegression, mutation generation and
credit, headed browser, staging and general restart/resume producers remain
unavailable. Their named managed phases wait and Store refuses new completed
markers from generic callers. Standard/Strict prerequisites remain mandatory;
the absence of an Impact producer is not an implicit Quick downgrade.

`execution::verification::ManagedVerifier` shares the exact RuntimeOwner and
ManagedWorkflowSources. Trusted operator integration explicitly calls `admit_tests`
before Workflow activation, then installs that verifier on the actual Engine.
No repository command string or Task-authored DTO activates a catalog. A catalog
has explicit Tests/typecheck/lint/build applicability, relevant required Tests and
commands for every other required category. NotApplicable requires a reason and
cannot silently drop mandatory Workflow obligations. This delivery freezes one
catalog per Project and one exact catalog digest, or unavailable NULL catalog, per
Workflow. Late catalog replacement/version selection is unsupported; existing
NULL-contract Workflows require a genuine fresh Workflow rather than retagging.

The operator admits canonical executable files outside managed writable roots,
pins their actual bytes, and supplies finite argv/cwd/limits. Bare shell evaluation
profiles are refused. Program identity is rechecked before reservation and every
command. This is explicit operator qualification, not proof that arbitrary program
code has no external effects. Tests which require daemon, network, absolute shared
paths, Docker, services, mutable external dependencies or hooks beyond the admitted
command contract have not been qualified by this delivery. Task namespace overlays
and ordinary PATH tool mediation remain cooperative; an intentional absolute-path
or same-user bypass is outside the application threat model. This is not a security
sandbox or a claimed general build-tool isolation solution.

Actual Engine initialize installs the immutable managed verification contract in
the same transaction as Task/Workflow state, before any Verifier Unit exists.
Store's managed success guard therefore applies even when an invocation has no
Unit or verifier port. Legacy nonformal library Workflows retain their distinct
behavior. Real7→8 migration installs a historical unavailable contract for proper
existing managed execution Workflows, never executable grants or accepted success.
Fresh8 and ordered migration use exact writer8 fencing on every mutable table;
namespace conflicts roll back. No empty intermediate schema is installed.

A private claim pins full P/G/T/W/Context/source/artifact authority. Reservation
creates a command-only Verifier Unit and returns the actual authorized Workflow
successor containing that Unit. It creates no Agent Session or native quota lease.
Generic Session, native terminal, non-Git helper and delegated effect ports refuse
this Unit before their intents. Preparation materializes a fresh resource namespace
and an independently retained read-only source snapshot of the Published commit.
The private producer observes canonical command cwd ancestry off Store/SQL locks,
then the real command-intent transaction checks the current whole claim, immutable
catalog, exact command and sealed observation before owned child spawn. Commands
are ordered; a subsequent command cannot follow non-certifying/incomplete work.

The collector observes the owned unreaped child, retains stdout/stderr separately
with finite limits, and records actual exit/signal, timeout/cancel/overflow/drain
conditions. Group stop is best effort and cleanup remains Unknown independently of
known work. It does not prove descendant absence. Actual terminal command receipts
can survive cancellation as historical diagnostics, but cannot mint a current
completion. Retained evidence files are outside every Unit writable namespace;
same-user intentional tampering is detected on inspection, not contained by OS
security. Protected stream retrieval verifies byte count/digest and caller budget.

Only complete admitted coverage, actual successful command receipts and reverified
retained input/artifact/streams mint the private completion. Store accepts it with
exact current authority and the one audited Workflow observation, atomically
advancing accepted run/Unit/Workflow/source pins. Known failure remains Failure;
overflow, incomplete evidence, changed input or uncertain operation wait without
acceptance. DTO `certifying`, caller Passed, evidence URL or raw run rows grant no
permission. Required readonly proof cannot be replaced by a command exit0.

Runtime/TaskDriver/operational CLI wiring, capacity-aware parallel command scheduling
and general crash recovery are separate integration work. Account-free controls
use actual Engine, managed Sources, Published artifacts, operator admission and
owned command collection. Fixture Claude/Codex stdio peers are not authenticated
official CLI qualification. Full Phase3/native four-Task/hook/subscription and
Linux/macOS qualification remain required independently. Source review and fixed
control/mutation handles are supplied separately; this description is not their
acceptance result.
