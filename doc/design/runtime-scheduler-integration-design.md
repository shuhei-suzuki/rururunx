# Runtime and global Scheduler integration design

Status: proposed STRICT design only; no production code added. Requirements:
[Runtime Scheduler integration](../requirements/runtime-scheduler-integration-requirements.md).
Actual base `ed3c1078a0f5a9646a8c4991881179887b57b28e`. This candidate connects
the existing approved native result-protection contract; it does not certify its
producer/profile/native acceptance or amend the earlier unmerged designs silently.

## 1. Source inventory and connection plan

| Actual symbol/file | Proposed consumer |
| --- | --- |
| `execution/owner.rs::RuntimeOwner::open` | Single retained Runtime service; owner lock/epoch entered once before reconciliation and scheduling. |
| `main.rs::Command`, `project::default_state_path` | Same global state selection; commands connect to owner rather than open competing epochs. Existing help/config paths remain pure. |
| `goal.rs::TaskDag::hard_order`, `state::Store::put_goal` | Structural validation reused before accepted graph commit; separate scoped readiness/evidence extraction needed. |
| `domain.rs::Goal/CompletionCriterion/Task` | Existing authoritative entities; accepted-definition/evaluator extensions from #23, not a duplicate scheduler DAG. |
| `execution/workflow_source.rs::ManagedWorkflowSources` | `prepare` before `WorkflowEngine::initialize`, then actual committed frame and same-unit first adoption. |
| `workflow.rs::initialize/step/cancel/retry/resume_gate/request_finalization` | One per-Task driver; current exact phase claims/bind/publication CAS preserved. Explicit retry/finalization, never generic periodic replay. |
| `execution/native.rs::NativeSessions::subscribe/status`, `NativeLimits` | Native status watch wakes driver; unchanged current caps are effective admission policy. |
| `execution/quota.rs::QuotaScheduler`, `state/execution/quotas.rs` | Single native permit authority and provider/account recovery; fair Scheduler rank composes inside final admission. |
| `execution/results.rs::ResultStore`, managed `WorkflowPublication` | Verified retained artifact plus actual Workflow completion drives dependencies; raw Verification Record/exit zero does not. |
| `state/execution.rs::begin_execution_epoch`, resource/effect/cleanup stores | Startup fences old admission, classifies held operations and preserves historical results/cleanup. |

Current `TaskDag::hard_order` only checks structure; current CLI has no Goal/run
commands; `CompletionCriterion` lacks typed evaluator origin. Runtime readiness,
control service and automatic completion below are new implementation work.
Initial production PhaseGates presently being developed can establish only their
actual scoped Issue/Worktree/Executor/commit evidence. Later test/review/external
gates must wait until their real producers exist. No broad fabricated gate is
part of this design.

## 2. Components and authority

Proposed private `runtime::Runtime` retains one `Arc<RuntimeOwner>`, validated
frozen runtime Config, the real managed AgentRegistry/Sources/PhaseGates, per-Task
drivers, an event receiver, bounded timer heap and control endpoint. `Scheduler`
selects work; it cannot itself manufacture native input, result publication,
accepted Goal definition or ApprovalBroker decisions. NativeSessions remains the
actual native dispatch owner. No planner/supervisor LLM is added.

`GoalController` uses #23's sole canonical accepted-definition/typed lifecycle
ports. Extend the existing Goal definition with accepted typed evaluators;
`goal_authority` identifies exact immutable definition/provenance, as planned in
`issue-23-design.md`. Do not maintain another authoritative Task graph. Definitions
and lifecycle are not constructed from native replies, JSON role names or generic
Store writes. Runtime-controller authority follows the trusted approved policy;
Human authority comes only from actual product ingress. Same-UID host takeover is
outside this application boundary.

`ReadinessSnapshot` is a bounded non-authoritative extraction: exact Project/Goal
versions/definition digest, scoped Task/Workflow versions, dependency evidence
references and classified reason. `Ready` is not a public dispatch capability.
The Store validates the same inputs when creating the driver claim; corrupted or
changed rows refuse, not refresh into implicit consent. Project validation/Git,
retained artifact verification and source assembly run outside SharedStore locks;
their exact resulting identities/hashes/versions are rechecked at mutation.

## 3. Persistence and ordered migration

Candidate schema starts from the actual Phase2 **5**, not main's historical3.
The scheduler/Goal authority write contract requires an ordered next migration
(candidate6 if no preceding merged contract consumes that number). Final number
is chosen once in the combined delivery. This is a proposed migration, not a
source change or an available API. Preserve 1→2→3→4→5 order and existing per-connection
writer-version trigger fencing of already-open old writers. Refusal on malformed
scope/body/history leaves original data/version unchanged; test old5 cached
read→write/INSERT/UPDATE/DELETE and old binary reopen, then current writer success.
Do not migrate legacy `satisfied` claims into evaluator authority.

Proposed scheduler-owned tables (all scope/epoch/body/index checks and bounded
typed access):

- `task_drivers`: one current row per Task, Project/Goal, epoch, claim UUID/version,
  claimed Task/Workflow generation/versions, state (`Driving`, `Parked`, `Held`),
  reason/deadline and initial preparation Unit reference if any. Historical claim
  transitions remain audited. A nullable initial Unit is not launch permission.
- `scheduler_projects`: persisted round-robin last-served sequence and next Goal
  cursor; per-Goal queue position likewise persists using a scoped scheduler row.
  These records never change authoritative Goal/Task/native versions.
- `goal_evaluations`: typed current evidence outcomes and accepted-definition/source
  pins, separate from Goal's mutable display booleans. Publication uses #23 ports.

Each row has strict identity/body-column equality, monotonic checked CAS and
foreign ownership checks. Generic Record APIs cannot write reserved scheduler /
evaluation authority. Claims are exact epoch tokens; wall-clock expiry is never
authority to steal a live driver. Event observations/attention may be ordinary
bounded facts but never override these tables. Store scans require pagination/
bounds and completeness indicators; truncated inventory cannot prove no owner.

## 4. Ready extraction and Goal completion

On each affected Goal event, extract the accepted definition, scoped declared
Tasks and referenced Workflow/evidence. Validate hard_order and complete endpoint
ownership. Include Tasks associated with that Goal but absent from DAG in ownership
and completion diagnostics; do not silently hide them. Partition PendingReady,
Driving, WaitingQuota/Capacity/Resource/Human, HeldExternal, CompletedVerified,
Failed/Cancelled and LegacyUnverified. Only Running accepted Goals produce new
ready Tasks. All hard prerequisites require current verified completed Workflow
policy and accepted retained evidence; soft edges do not suppress readiness.
ReadyPR is not CompletedVerified. A recovered native terminal alone does not pass
a gate. Unavailable production gate/evaluator produces a named hold.

At least the #23 `RequiredTasksVerified` evaluator consumes actual current complete
Workflow evidence, immutable retained artifact/source versions and required policy
results; it rejects strings/URLs/booleans/diagnostic records as substitutes. Other
accepted evaluator types either have their genuine producer or return Unknown.
Human attestation binds current criterion/definition/artifact and actual trusted
ingress. Factual criterion outcomes publish separate evaluation rows, not a Goal
version increment per poll. Completion transaction rechecks accepted definition,
all required evidence and applicable outstanding external operations before writing
Goal Completed and audit; cancellation/publication CAS has one winner.

Goal Context Pack and derived progress summarize facts without rewriting shared
Goal/Task native authority on every sibling completion. Current governing digest
includes Goal context pointer/DAG; accepted graph/context authority changes are
queued while applicable turns are live, or explicitly revoke/fence them under
reviewed policy. No benign-bookkeeping exemption or new semantic Goal.version is
invented here. A follow-up proposal is inert until #23's trusted graph acceptance
transaction checks scope/dependencies/policy/current authority. Existing ready
Tasks continue without prompting. Material scope expansion remains Human-only.

## 5. Task driver and initialization order

Proposed `Store::claim_task_driver(snapshot, owner_epoch)` transaction validates
accepted Running Goal, current scoped Task/Workflow and predecessor evidence,
absence of competing Driving claim, effective Project Task capacity, and actual
phase prerequisites. It commits a claim and advances fair Project/Goal cursor
before dispatching the per-Task driver. Exact claim/version is carried privately;
claim publication failure starts no worker. This is driver capacity, not a second
native session lease. Duplicate candidates/CAS losers re-read or park.

First Task drive:

1. Claim driver and validate actual selected provider/config/resource readiness.
2. `ManagedWorkflowSources::prepare(task, provider)` registers its preparation Unit
   before any Git helper; retain the genuine prepared capability in Sources.
3. Re-read actual Task after registration changed its projection/version, persist
   the claimed Unit reference under exact claim CAS, and initialize Workflow using
   the committed policy/rules/tree. A cancellation/epoch change in between causes
   existing guard retirement; driver cannot substitute a persisted row as capability.
4. Drive initial evidence phases only through real qualified production gates.
5. First native Executor consumes the live capability via existing atomic adoption;
   later phases use new attempts or independent retained snapshots. No initial
   fake Success/artifact or legacy live fallback.

Workflow's own reservation and exact unit/Context/Task/P/G CAS remain the phase
authority. Driver claim complements, not replaces, those checks. Call `step` once
per current event/deadline and retain the actual future until completion; one driver
per Task. After Started subscribe to real native status. Post-initialization
source/driver failures leave explicit held state and exact original reservations;
owner-local verified rollback remains as existing #41 permits. Dropping a driver
does not prove no dispatch. Durable stale driver/Unit references are classified
at restart, never used to reconstruct PreparedExecutor or native handles.

## 6. Capacity and fairness composition

Accounting has two separate units:

| Counter | Complete source and lifetime |
| --- | --- |
| Global/provider native sessions | Active existing `quota_leases`, including pre-Session admission and all native roles; released only by actual terminal/fencing contract. |
| Project Tasks | Union of distinct Tasks with Driving claims or active native quota leases, plus owned in-flight preparation effects; one Task counted once. Parked pre-input no-helper/no-lease waits yield driver slot atomically. |
| Preparation/evidence/cleanup workers, retained bytes/ports | Their actual bounded jobs/resource ledgers; separate pressure, never fabricated provider quota. |

Current quota `project_capacity_blocked` only counts Tasks with active native
leases; it does not count pre-admission preparation. The composed grant must use
the same distinct-Task union above in driver claims and native admission, so
preparation cannot bypass max_tasks_per_project and the native handoff cannot
double count its own Task. The logical claim/park change and count test are in
Immediate transactions. A live native quota-waiting turn stays counted. A terminal
or fenced historical unit may retain cleanup/resources while no longer consuming
a native session permit; pressure/backlog quotas remain independent.

No extra `Semaphore<global_sessions>` is acquired across the Task workflow. Use
NativeLimits/QuotaScheduler and their actual Store admission; they already count
lease-before-Session and aliases share the provider's strictest cap. Initially
publish effective global6/default provider executor2/total3/global executor4 and
all reductions. Do not advertise config12 as actual12. Generalization must update
bounded process/reader/worker capacity and configurable headroom together through
a reviewed source delta and controls; default four-executor mixed-provider acceptance
does not qualify arbitrary higher limits. This remains a full configured-policy
acceptance gap until implemented/qualified.

Scheduler rotates eligible Projects, then eligible Goals, then FIFO eligible Task
enqueue sequence with UUID tie break. Persist cursor on each successful claim;
newly ready work is appended, not sorted eternally ahead by UUID. Skip held/saturated
entries without spending the entire turn retrying them. If P continuously eligible
Projects can each fit an available slot and native operations eventually finish,
each gets a claim opportunity within P successful selection turns; this is an
opportunity bound, not native completion time. Within-Goal age has the analogous
finite eligible queue bound. Quota-blocked entries retain age without owning idle
permits.

Provider admission currently sorts its own waiters using last-role and sequence.
The outer rotation alone cannot override that order. The composed Store admission
must incorporate trusted persisted Project/Goal selection rank within compatible
role/account buckets, preserving bounded role headroom/probe exclusivity and
skipping lifecycle/Project/capacity-ineligible candidates. Cursor consumption and
grant have exact CAS; caller-supplied rank alone is no authority. Test both the
outer selection and actual inner grant. Round cohort permit reservation for #9/#10
must be their actual reviewed atomic full-cohort contract; absent producer means
WaitingEvidence, not partial-member permit deadlock or weakened reviewer policy.

## 7. Event loop, retry and shutdown

Use a bounded coalescing wake queue keyed by scoped Task/Goal plus `Notify` and a
monotonic timer heap for the earliest durable next_due. Native watch receivers
publish status changes; Store/controller commits publish wake hints only after
commit. Queue-full retains a level dirty flag; no dropped hint clears dirty state.
Driver completion is independently observed, not solely a sender's one message.
The loop clears a dirty flag only after comparing the consumed generation and
reloading durable readiness; concurrent wake leaves it dirty. Exact due waiter
claim remains Workflow's existing Record CAS before native re-entry. Watching
status does not invoke a new native start/PhaseGate or reread source on every tick.

At startup and bounded housekeeping (proposed 30s, not sub-second polling), perform
a paginated level-state check so missing events converge. Timers are only hints:
re-read durable deadlines and recheck lifecycle/epoch before grant. Forward clock
jump may wake early for checked admission; backward jump uses bounded monotonic
recheck/backoff and records attention rather than spin. Idle path blocks on event,
earliest due or housekeeping, with no repeated Immediate writes for unchanged holds.

Live quota continuation uses the same native turn/input/unit; native Available
telemetry must be accepted by the existing pool rules before live wait is cleared.
Terminal quota/capacity interruption is classified, fenced and explicitly retried
in a fresh attempt. Shared provider/account probes remain bounded and single-owner.
Auth/Unsupported/unknown external effects cannot be retried as quota. Reversible
known no-effect preparation errors follow existing private owner release; marker
ambiguity requires exact recovery. Repeated failures have visible bounded backoff /
attention and do not reset review lineage budgets.

Shutdown closes new claims/grants, issues scoped logical cancellation per policy,
awaits retained owned supervisors boundedly and persists unknown unfinished facts.
Never drop bookkeeping to claim cleanup. Cleanup worker remains independent with
bounded service and no global prune. Result-finalization authority remains separate
from native-effects authority as approved; janitor must not delete capture inputs
before publication/refusal is durably resolved.

## 8. Restart and recovery

The owner acquires the same state-root lock and runs begin_execution_epoch once.
Old native/finalization authority is retired by existing epoch handling. Then:

1. Validate Project registry identities/references through bounded off-lock I/O;
   persist missing/moved/replaced sources as Blocked, no silent rebinding.
2. Load accepted Goals/graphs and all scoped driver/Workflow/Unit/Session/effect /
   resource/result/quota records. Preserve historical evidence and metrics.
3. Classify claims: never started and proven no-effect; preparation-only abandoned;
   live old input unknown; reliable old terminal; pending external outcome; published
   artifact; cleanup backlog. Missing PID/Session alone proves none of these.
4. Close old driver epochs under exact CAS. Genuine accepted retained artifacts
   are verified with current read authority; old preparation rows cannot be adopted.
   Unknown local work may receive an explicit fresh retry using actual new namespace;
   uncertain PR/merge/deploy remains held for #13 idempotent reconciliation.
5. Recompute distinct Task/native capacity and fair queues; startup's release of
   old logical quota leases is not resource death. Quarantined physical namespace /
   ports remain unavailable. Derived ready lists/cursors grant nothing by themselves.
6. Resume only accepted Running Goals whose actual current producers/policy allow
   it. Others expose attention/Unsupported/WaitingResource, rather than fabricate
   reconnect or permanent global hold unrelated to actual effect scope.

Atomic generation/cancellation/late-result controls are reused, not bypassed by
the scheduler. Legacy definition-unverified/external-unknown history is retained
without retroactive success. Source migration and scheduler startup remain separate
operations: unsafe/invalid migration refusal cannot be relabeled successful recovery.

## 9. CLI and control-plane boundary

Proposed foreground Runtime command owns the service lifecycle; other supported
CLI/TUI commands attach over a separate local control endpoint (do not reuse native
TaskTool IPC as trusted Human ingress). Read commands do not enter a new owner epoch.
One-shot start can spawn/connect the same service when explicitly requested;
help/version/config-check do not. A racing second service fails the existing owner
lock. The endpoint uses bounded strict method/DTO decoding, exact runtime state
identity and selectors. TaskTool/native/Broker handlers cannot construct Human
Goal/lifecycle authority. Same-user direct CLI execution remains the explicit
application trust limit, not a universal host access-control promise.

Commands map to accepted Goal creation/lifecycle, Task-driver enqueue, actual managed
cancel/fresh retry, bounded scoped logs/status and genuine adapter attach where
supported. `goal --file` supplies objective plus explicit accepted criterion/graph
input or enters Analyzing/proposal state; it does not infer executable evaluator
commands from text. Follow-up acceptance is a distinct explicit controller action.
Pause/cancel acknowledge after durable authority change; they do not certify all
children dead. Status exposes independent work/cleanup axes and current readiness /
effective capacity. Missing attach or later gate support is explicit Unsupported.

## 10. Verification and remaining delivery gates

Implement only after independent requirements then design approvals. Schema/claim /
trusted ingress source needs STRICT independent review. Two isolated repositories
and account-free peers exercise RS-AC1–9 mechanics with actual Store/Workflow/
Sources/native wire/resource/artifact consumers. Include contested grants, queue
loss, suspended driver, helper waits, due claims, clock changes, cancel/publication
and abrupt owner restart. Kill compiled actual-consumer mutants for each authority /
limit/fairness/evidence check; test-only fabricated Success/private SQL seeding is
not a positive producer. Default parallel Debug/Release, fmt/Clippy/build and
exact-head Linux/macOS CI failures remain factual until resolved.

Actual logged-in Claude/Codex four-Task/two-Project operation, settings/hooks/quota
compatibility, current immutable review/test SHA controls and #16 enabled/baseline
measurements are separate required native acceptance. Synthetic tests cannot tick
those rows. Real #9 independent review, #10 approvals, #12 impact, #13 external
operations, #19 full Context/condensation, #20 bundles and #21 telemetry producers
must compose before their gates can pass. Goal evaluator/accepted definition and
CLI Runtime are not supplied by the current source baseline. No optional browser,
PWA/native Goal or privileged service is introduced to fill these gaps. This design
does not claim final Phase2, whole #14/#23/#24/#27, or MVP completion.
