# Runtime first operational delivery: integration supplement

Status: proposed STRICT component design; no Runtime production implementation or
schema9 migration exists at this commit. Requirements and scope remain the approved
[Runtime requirements](../requirements/runtime-scheduler-integration-requirements.md),
[Runtime design](runtime-scheduler-integration-design.md),
[Goal authority design](issue-23-design.md), and latest
[native execution R1–R9](../requirements/agent-execution-requirements.md).
This selects their first actual operational implementation, without changing WHAT.

Source baseline: `5c070d4aa4502e9ef737fe5960e74fbd03cdb92b`, unmerged Phase2
schema7. Verifier8 must be composed before implementing the ordered 8→9 Runtime
migration; future ReviewRound allocation is10. No empty intermediate migration.
Native receipt follow-up is separately reviewed; a missing qualified provider,
Sources route or PhaseGate remains unavailable. Account-free component controls
and documentation cannot certify authenticated native/MVP acceptance.

## 1. Actual connections and delivery boundary

| Existing producer/consumer at baseline | First operational connection |
| --- | --- |
| `domain.rs::Goal/CompletionCriterion/Task`, `goal.rs::hard_order` | Canonical accepted definition and complete graph, typed lifecycle/graph ports and readiness extraction; structural ordering alone grants nothing. |
| `execution/owner.rs::RuntimeOwner::open` | One retained service lock/epoch, acquired once; attach/read commands do not instantiate another owner. |
| `execution/workflow_source.rs::prepare`, `InitialWorkflowExecutor` | New Workflow preparation under genuine Driver claim, before initialize; retain the actual single-use initial capability until first native adoption. |
| `ManagedWorkflowSources::recover_retained`, schema7 `source_recoveries` | Bind the source-only recovery to the actual Driver in the claim/accept/helper transactions; never reconstruct an old prepared/native capability. |
| `workflow.rs::WorkflowEngine::{initialize,step,retry,resume_gate,cancel,request_finalization}` | One retained Task driver serializes deliberate calls; Workflow phase/claim/Context/native CAS remains authoritative. |
| `adapter.rs::AgentRegistry`, managed adapter status/watch | Real registered adapters and their watches, not invented result or capability providers. |
| `execution/quota.rs`, `state/execution/quotas.rs` | Single existing native lease authority plus persisted Scheduler rank and distinct-Task Project count in the actual admission transaction. |
| `ManagedWorkflowGates`, `ResultStore`, `WorkflowPublication` | Real initial phases and accepted publication; later unsupported gates wait. Verifier8 supplies only its qualified supported plans. |
| `main.rs::Command`, global state selection | Root-coordinated trusted CLI ingress/control service wiring; no TaskTool endpoint for Goal control. |

First delivery must be runnable through real Runtime composition and trusted
control ingress, not a standalone pure queue helper. It creates and runs an
explicit accepted Goal/DAG, drives qualified initial phases, parks named missing
gates, supports status/pause/resume/cancel and retained Published-source restart.
It does not invent a planner, infer evaluators from prose, accept Ready drafts,
resume uncertain native/external effects, or implement section5.2 pre-artifact
bootstrap recovery. Those routes return an explicit Unsupported/attention reason.
Full automatic Goal completion stays unavailable wherever required evidence/gates
are missing; admission and waiting are operational outcomes, not Goal success.

## 2. Canonical accepted Goal and trusted ingress

Implement ONE accepted-definition/trust module under `runtime::goal_authority`,
reused by future Review/Human consumers rather than defining parallel authorities.
Private non-serializable `HumanIngress` and `ControllerAuthority` are constructed
only by Runtime's dedicated trusted control-service handler. The executable sends
validated typed actions and the handler derives its own local invocation/caller
identity under the approved trust boundary; the DTO never supplies an authority.
The CLI has no authority constructor or serialized capability;
no model/TaskTool/Broker JSON principal, role name, serialized enum or audit row
constructs it. Controller creation requires the exact durably Human-approved
policy identity/version/content digest. This is the approved application boundary,
not biological identity, same-UID OS secrecy or security against malicious linked
Rust code/direct host DB or executable access.

Extend CompletionCriterion with the approved typed evaluator and legacy
Unverified default. Canonical definition includes title/objective, ordered
criterion IDs/descriptions/evaluator tags, constraints, non-goals and fixed source
refs; use the #23 fixed field order/UTF-8 length-framed encoding. `goal_authority`
contains its Store-computed exact digest and provenance, not duplicate content.
Accepted fields are immutable; no operator/controller edit or migration ratifies
legacy `satisfied/evidence`. Unsupported evaluators refuse accepted creation or
produce the already approved explicit Unknown outcome, never default satisfied.

Private typed ports (names below are proposed, no existing API claim):

- `create_goal(ingress, expected_project, definition, accepted_policy)` inserts
  canonical Goal Created + exact authority + audit in one Immediate transaction.
- `accept_initial_graph(authority, expected_goal, tasks, edges)` validates full
  membership, hard DAG, Task policy/class/risk and references, atomically creates
  actual Tasks and canonical nodes/edges. No GitHub writes occur here. An explicit
  empty planning Goal may exist but is not dispatchable/completable; prose-only
  creation remains Analyzing/proposed until an actual graph is accepted.
- `set_goal_lifecycle(authority, expected_goal, target, reason)` implements #23's
  approved transitions. Running requires accepted definition/graph; Completed is
  exclusively the private complete-evidence evaluator. Resume rechecks current
  applicability and leaves unsupported recovery or unknown external effect held.
- `accept_followup` uses the same additive graph/Task transaction and trusted
  bounded policy; material/unprovable expansion requires genuine Human ingress.
  No ordinary observation may activate a graph change. First delivery accepts
  explicit Human-authorized graph additions; autonomous followup acceptance remains
  Unsupported unless its genuine approved applicability/policy producer exists.

Generic `put_goal` rejects inserts and every changed field, permitting only exact
persisted replay as a no-op. Generic managed Task creation/change also refuses;
owned Workflow/Unit projection transactions are the specific permitted writers.
Legacy Goals remain definition-unverified, never Runtime-ready. Typed Cancel/Fail
can preserve their exact invalid legacy DAG/history without repair or dispatch.
Old fixture setup must migrate to genuine trusted test ingress and these actual
ports; no production bypass for generic fixture callers. Future CLI inline/file
syntax feeds this same validated definition/graph, not a second representation.

## 3. Schema9 and bounded persistence

After real8 is fixed, install ordered8→9 and fresh9 atomically. Keep the existing
per-connection exact writer-version function/triggers on ALL mutable tables and
refuse old8 reopen and pre-open cached read→write/INSERT/UPDATE/DELETE, including
private audit/metadata. Migration rejects malformed/conflicting new namespaces
without changing original bytes/version; old rows are not retagged approved.
The Runtime root lock and new owner epoch are separate from migration acceptance.

New private tables: `goal_authority`, approved controller policy references,
`task_drivers`, scoped scheduler cursors/queue sequence, bounded Goal evaluations,
proposals and control acknowledgements. Canonical Goal remains the sole definition
and DAG. Private identity/body/index equality, scope foreign keys, checked positive
versions ≤i64::MAX, NO public Record write/Replace/identity mutation, reserved
append-only audits and explicit finite retention apply. Driver current row is
unique per Task; historical claim decisions retain immutable scoped references.

Use #23's approved bounds: definition1MiB, Goal JSON4MiB,4096 total scoped Tasks
including unlisted,16384 edges,128 criteria/refs,256 followups/64 definition-change
proposals,16KiB individual text/64KiB proposal or evidence/depth32. Goal extraction
is complete, at most4096 Workflow,8192 Session and8192 lock/recovery records and
32MiB aggregate decoded bodies; evidence has its separate32MiB/32768-record cap.
Driver metadata128KiB/row and one current row per scoped Task; cursor/control
metadata64KiB/row, control queue256 requests per Runtime; reject overflow before
mutation/effect and expose attention. Scans check counts/encoded lengths before
decode; truncation cannot prove readiness or absence of owner. Across Projects,
page complete scoped Goal extractions; an oversized Goal is held without stalling
truly disjoint eligible Projects. These are finite component policies, not a
512KiB total SQL/heap/latency promise. No external I/O under SharedStore/SQL locks. Align each actual reader with these
approved row caps: schema7 SourceRecovery currently bounds Goal at1MiB, so the
Runtime seam must deliberately update its Goal reader to the approved4MiB full-row
cap and control the boundary; a larger accepted Goal must not fail secretly in
source reconstruction. This does not raise the1MiB accepted-definition cap.

## 4. Genuine Driver ownership and readiness claim

`TaskDriver` is private/non-Clone and belongs to one retained supervisor task in
`Runtime`'s owned JoinSet/registry, with the live RuntimeOwner and exact private
claim identity. A durable driver DTO, timeout, PID or row existence cannot create
it. Private borrowed scope bindings may be carried by the actual Sources/Engine/
native consumer while this owner remains registered; they cannot be serialized
or constructed by caller JSON. Cancellation/Drop revokes new invocations and
persists the exact claim's conservative disposition; retained native supervisors
keep their existing handles/result bookkeeping. Failure to record revocation
leaves held authority, never a second owner or automatic successful release.

`extract_readiness` is a bounded Deferred coherent read: current Registered
Project, accepted Running Goal+definition/policy digest, complete actual Task
membership/DAG, Workflow/Context/evidence/Unit/Session/effects/locks/resources and
applicable physical namespace conflicts. A hard predecessor needs the actual
current Workflow success extractor; Task Completed, generic Passed/Verification
Record, URL or native terminal alone is insufficient. Unknown/missing proof holds
only its actual dependency/effect scope. Soft edges are advisory. Readiness is a
classification and expected snapshot, never a dispatch capability.

`claim_task_driver(owner, expected_readiness)` Immediate transaction rechecks
epoch/instance, all authority and prerequisite pins, complete conflict inventory,
no competing claim, Driver limits and distinct-Task Project capacity. Store chooses
the persisted queue rank; only a successful claim consumes its fair cursor. It
writes claim/audit before spawning the retained driver. Claim commit failure starts
no driver. Spawn failure closes only the genuine still-no-effect claim under exact
CAS; uncertain commit is read-only reconciled, never repeated blindly. Driver may
record initial Unit/recovery references only through the corresponding producing
transaction. Expected P/G/T/W/Context pins are captured, not fetched later to
ratify arbitrary drift. No lease expiry/ordinary poll may steal a live claim.

Owned Workflow transitions validate the live claim binding and original expected
pins BEFORE their writes, then advance only checked resulting Task/W/Context
bookkeeping of the exact same Task/generation. Initialization records resulting
Workflow pins atomically with the initial Driver binding. Driver state/version
advancement shares that transaction and returns its exact resulting private binding
to the owning driver. Claim ID/epoch remains fixed; a helper captures the exact
version for its await. Other same-Task authority advancement waits for that helper
or deliberately invalidates it. Never reread a changed public row and manufacture
a new binding; public Task/Record mutations never ratify it.
Governing P/G definition/context/graph drift invalidates; no benign refresh of native
input. This adds ownership checks without removing existing Workflow claims,
result publication, source/input digests or provider Session CAS.

## 5. Actual Sources/Engine invocation and source-recovery seam

For a Task with no Workflow, the genuine Driver calls ManagedSources.prepare under
its private binding. AttemptManager's actual registered source Unit transaction
checks/binds exact claim before the first Git helper. Async reads keep the actual
initial capability/abandonment guard. Successful initialize atomically binds its
real Unit/frame digest and resulting Task/W/Context to the same Driver. Source
preparation success is not native work success. Unit adoption is still single-use
and Engine/Store first-native reservation retains all existing source/namespace
and Session-free checks. No unregistered pre-bootstrap Git or live-context fallback.

For existing Workflow with Published retained source, add a Driver-bound overload
of schema7's actual source-only begin/accept path. `begin` checks the genuine live
Driver/current version AND complete original P/G/T/W/Context/artifact pins and
records exact Driver ID/version in source-recovery metadata in the same transaction.
Registered RetainedGit reservation, periodic fence and helper receipt transaction
validate BOTH SourceReadBinding and Driver binding; a driver cancel/pause/epoch
change during an await refuses. The reconstructed complete graph/frame remains
from actual registered retained objects/config/rules, without cache seeding.

`accept` rechecks full original snapshot/private frame proof and Driver identity/
version, writes Installed recovery and Driver's resulting recovery/frame pins
atomically. The Task slot stays serialized through accept+map install; current
capture/committed_input/unit grant check both bindings. An accepted but not yet
installed map after crash is reconstructed through a NEW actual claim/complete
verify; a persisted row never recreates a capability. Installed frame payload is
never borrowed from old launch JSON. Existing schema7 standalone source-only
component route stays distinguished from Runtime authority; it cannot dispatch an
accepted managed Goal without genuine Driver binding.

Typed same-Task Workflow bookkeeping must use one prevalidated shared transaction
plan to advance both Driver and source-recovery pins against checked resulting
rows. Do not advance Driver first and then authorize Source from a new row, or
vice versa. Cancellation/terminal/generation changes invalidate source and deny new
invocation atomically. Source7 generic-drift rejection remains; the source-only
row/claim does not become native authority. Runtime boundary rejects section5.2
pre-artifact recovery, Ready draft, old unknown/native/effect history or missing
retained proof before any attempted ordinary prepare/initialize replay.

## 6. Project Task accounting and actual native fairness

Implement one indexed distinct-Task union used inside BOTH driver-claim and native
quota transactions: Driving claims, active quota leases (all native roles), and
current owned in-flight preparation/helper effects whose logical authority has
not been fenced. Count one Task once. Registered pre-Session/pre-helper preparation
is covered by its claim; handoff to native does not double count. Parked waits
release Driver capacity only after exact no-current-helper/no-active-native-lease
checks; retained unit/worktree/resource identity remains. Live native quota waits
stay counted. Closed/fenced historical cleanup backlog consumes independent
resource pressure, not a fabricated live native permit or proof of death.

Configured Runtime and Project Driver Task ceilings bound actual supervisor jobs;
there is no fixed four-Task queue ceiling. Native actual effective limits remain
component global6/provider executor2/total3/global executor4 until their existing
bounded-resource generalization is implemented/qualified. Report requested versus
effective reductions, including requested12; do not claim configured12 concurrency.
Driver count is not phase permit count: all concurrently admitted reviewer members
need their actual native leases; missing reviewed cohort producer waits, never
partial-member semaphore deadlock. No second global native semaphore in Runtime.

Persist Project→Goal rotation and FIFO eligibility sequence/UUID tie break in
scheduler tables. Continuously eligible Project gets an opportunity within P
successful fair selection turns if it can fit and native turns finish; no bound
on model completion is inferred. Saturated/ineligible pools are skipped, waiting
age retained, and new-ready entries append. Actual reserve_execution_quota ordering
must consume trusted persisted Project/Goal rank inside its existing Immediate
transaction, preserving role headroom/probe exclusivity and all provider aliases.
An outer queue's rank alone cannot override inner waiter fairness. Epoch/cancel/
foreign/malformed/stale rank refuses; cursor/rank changes update scheduler tables
only, never Goal/Task/native authority. Same-attempt live telemetry uses existing
pool correlation rules; terminal interruptions require deliberate current-policy
fresh retry, never duplicate pending input or auth-as-quota classification.

## 7. Event driver, controls and independent result/cleanup axes

Coalescing per-Task/Goal dirty generation plus bounded Notify/event queue256, a
retained JoinSet and monotonic earliest-deadline heap drive actual consumer calls.
Store commits publish wake hints after commit; driver completion is independently
observed by JoinSet. Queue full/closed/drop cannot clear level readiness. Clear dirty
only after comparing consumed generation and reloading durable state. Duplicate
wake invokes no second start. Startup and proposed30s bounded paginated level scan
recover lost notifications; unchanged holds cause no Immediate write/spin.

Driver serializes one actual Engine invocation. It records Waiting reason/current
attempt/native due, subscribes the actual adapter watch after Started/Running and
rechecks durable status so terminal-before-subscribe is not lost. A wake chooses
polling the bound active attempt, explicit current due-wait claim/resume_gate,
next phase, or allowed fresh retry; it does not call arbitrary gate/start again.
Use existing exact due-record CAS. Timers recheck persisted deadline/current epoch;
clock shifts get finite monotonic recheck/backoff, not loops or automatic replay.
Unsupported/gate MissingProducer parks until a relevant capability/control change,
not every housekeeping tick. Session-less external Evaluating/unknown effects stay
held; nonempty observations are not invocation/death permission.

Pause/cancel commands are genuine trusted control requests with scoped idempotent
request ID/expected Goal version; validate typed lifecycle and commit its decision
plus driver grant revocation first. Conservative Engine TerminalDecision and Unit
fencing consume that private control authority even after Driver revocation/Goal
pause; they may only cancel/invalidate, not advance or regrant. They must not demand
a Running Goal/live Driving claim merely to record the cancel. Same-transaction
applicable Unit/Task fences
prevent a prepared/input grant racing the decision. Then request actual retained
native stop outside locks; keep bounded observations/result retention. Paused Goal
cannot new-start/resume gate/input; factual historical terminal/status/cleanup can
still be retained through existing ports. Resume does not reopen old units or old
Sessions, and unsupported pre-artifact recovery remains visible. Cancel is terminal,
uses existing Engine cancel/Unit fences, and preserves frozen accepted definition,
history and retained results. Stop Runtime closes new claims first and supervises
existing operations' bounded shutdown; no drop-to-prove-death shortcut.

Status and progress derive current facts or publish bounded separate observation/
evaluation/control rows. Task completion/poll/status never increments Goal.version
or sibling Task authority. Accepted graph/Goal Context writes are genuine authority
changes: queue until applicable admitted turns are logically closed, or explicitly
revoke/fence under approved policy. Driver metadata refresh cannot exempt them.
Native WorkOutcome and cleanup stay independent: known Success/Failure is retained
even when best-effort cleanup is Unknown/Leftovers. Cleanup alone neither completes
Goal nor invalidates accepted immutable output. Current unsettled input/operation
and uncertain external effect still block inappropriate grants; physical namespace/
port pressure follows its actual resource scope. No all-descendants death claim.

## 8. Completion, CLI seam and qualification

Implement the approved RequiredTasksVerified extractor from actual finished
Workflow/configured gate evidence, immutable Published artifact verification and
current source/dependency/policy pins, with no outstanding logical invocation or
uncertain external effect. It cannot infer full success from the initial limited
gates/native receipt. Publish evaluation as separate bounded evidence; sole private
completion transaction checks current accepted definition/DAG/all Tasks/evaluations/
proposals/operations and writes Goal Completed once. Unsupported criterion or
missing later Review/Merge/cleanup evidence is Unknown/Waiting. Human-attested
criteria require the actual ingress/current target, never accepted JSON claims.

Root coordinates main CLI commands against one Runtime control service: explicit
validated Goal definition/graph create, status/pause/resume/cancel; Runtime run,
status --all/stop/resume/log references. Control endpoint is separate from TaskTool
socket and does not accept a public Human label. Read/status/help/config commands
cause no owner-epoch rollover or native effects. No fabricated attach capability;
unsupported adapter attach is reported. Runtime module without this actual product
consumer is a source milestone, not operational acceptance.

Required component controls use the real ingress/driver/Sources/Engine/admission
ports, isolated Git repositories and owned account-free stdio peers, no SQL-seeded
private approvals. Prove rejected legacy/foreign/cyclic graph and generic writers;
accepted immutable definition/automatic missing-evaluator wait; two Projects/four
overlapping Tasks/provider aliases; same Issue numbers; capacity during preparation
and native handoff; actual inner fairness/restart cursor; duplicate/lost/full wakes,
terminal-before-subscribe and exact competing due claim; pause/cancel at claim,
Git await, pre-input and live-quota wait; siblings progress without Goal rewrites;
Published-source restart+map install/driver/Context/cancel CAS. Preserve section5.2
unsupported after initialize, Ready draft and unknown effects. Actual configured
native/hook/auth/OS4+ Task runs and fullRS-AC1–9/native12-cap generalization remain
separate required acceptance gates, not satisfied by fixture results.

Compile causal omissions of acceptance digest/claim CAS, distinct-Task count,
inner rank, level wake/deadline, source+driver joint binding and generic-drift
refusal into actual controls; setup/compile/refusal-before-target earns no credit.
Migration old8 open/cached writer and rollback controls are mandatory. Independent
STRICT source review at fixed clean commits, meaningful Debug/Release checks and
Linux/macOS CI precede delivery. Production work follows approval of this finite
supplement; no positive readiness inferred from this design.

## 9. Source ownership and integration sequence

A owns `runtime/*`, new `state/runtime/*`, Goal-specific trusted producers and
narrow domain/Goal/state/SourceRecovery/Workflow glue. Shared schema9/mod/version
edits begin only after C's fixed Verifier8 milestone, with B's Native6 corrections
normally composed. Root coordinates main CLI trust/service consumer. Typed ordinary
Workflow hooks coordinate with C8; do not copy gates or own concurrent shared files.
Sequence: compose real8 → schema9+Goal trust/Driver/store seams → actual Runtime
Sources/Engine event consumer → Root CLI composition → complete component controls,
causal mutations and independent source reviews. Each coherent source milestone
is committed before tests. Pending producers remain named unavailable throughout.

## 10. Source9 first callable controller milestone

This initial Source9 component implements the actual Unix-peer CreateGoal,
GoalStatus, GoalTasks, Goal lifecycle and control-service start/shutdown consumers.
It accepts an explicit bounded definition and graph in one transaction, records
request acknowledgements and installs ordered actual8→9 writer fences. Public
Goal DTO insertion or changed-field replay is refused; exact persisted Goal and
accepted Task replay is a no-op. Legacy criterion JSON defaults to Unverified,
without becoming an accepted definition. The Source7/C8 Goal readers use the
existing approved4MiB Goal bound, separate from definition1MiB.

GoalTasks is routing/status only: exact accepted Goal and complete bounded scoped
Task inventory, deterministic TaskId cursor,1..128 entries per page and complete
response≤64KiB. A foreign cursor, malformed identity or changed scope yields no
partial authority. The compact GoalAccepted acknowledgement contains no4096-Task
payload. Actual graph facts and current Task states do not certify readiness.

The service reloads durable queue rows on wake/timer with bounded keyset sweeps,
publishing NativeBindingUnavailable attention without rewriting P/G/T/Workflow
versions. No fair admission or native dispatch is implemented by this sweep. The
retained live Driver registry has no registration constructor and remains empty;
accepted Goal preparation, Context/Workflow publication and native/finalization
grants requiring a Driver therefore refuse. Metadata always reports
operational=false, independently of whether the control-service loop is running.
The actual same-operation binder and retained Driver issuer/readiness producer
must compose with Binding10 before these ports can admit actual work; ReviewRound
is allocated11. No Binding10 DDL or legacy allocation proof is part of9.

Lifecycle controls are explicit logical fences, not proof of actual cleanup or
successful Workflow termination. Cancel/Fail/Pause close applicable Unit grants
using the existing scoped producer and retain native cleanup uncertainty. Task
and Workflow terminal projections, actual owned stop callbacks and recovery remain
pending. Resume is restricted to accepted, Registered, truly never-prepared
graphs with unchanged accepted Runtime policy; any Unit/Workflow/Session/lock,
Context or SourceRecovery history requires the still-unavailable recovery/Driver
port. A remembered row never reconstructs authority. Already-completed legacy
or managed history is not silently imported through generic writers.

This milestone does not complete§4–9 or whole930. Prior positive fixture setup
must migrate through real trusted control ingress; actual native positive paths
then additionally require the missing binder/Driver. Setup incompatibility and
producer unavailability are recorded separately, never disabled or converted to
passing refusal tests to obtain a green regression. Account-free controller
controls qualify only the above component. CLI service wiring and whole Runtime,
4-Task/2-Project scheduling, accepted evidence evaluation and native/profile
qualification remain open.
