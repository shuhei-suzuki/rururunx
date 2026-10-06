# Runtime and global Scheduler integration requirements

Status: proposed STRICT integration contract; documentation only. Source baseline:
`ed3c1078a0f5a9646a8c4991881179887b57b28e` (unmerged Phase2). This document does
not qualify native providers, automatic Goal completion, or a shipped Runtime.

## 1. Authority, scope and existing requirements

This connects existing MVP requirements, rather than introducing an orchestration
LLM: Product Requirements, [architecture](../design/master/architecture.md),
[Goal Runtime](../design/master/goal-runtime.md),
[multi-project Runtime](../design/master/multi-project-runtime.md), and approved
[native execution R1–R9](agent-execution-requirements.md). The authorized MVP Goal
requires continuous dependency-ready execution, 4+ parallel Tasks, fair global /
Project / provider limits, recovery and CLI observability (§§7–14,32–33,39–40).
The latest approved result-protection policy governs native execution: logical
fencing, immutable results and best-effort cleanup, without whole-workload death
or same-user sandbox claims. Historical ownership-only constraints are not silently
reintroduced as readiness predicates.

Related public scopes: [#23](https://github.com/shuhei-suzuki/rururunx/issues/23)
Goal definitions/DAG/evaluation; [#24](https://github.com/shuhei-suzuki/rururunx/issues/24)
Goal loop/CLI; [#27](https://github.com/shuhei-suzuki/rururunx/issues/27) global
fairness/accounting; [#14](https://github.com/shuhei-suzuki/rururunx/issues/14)
recovery; [#19](https://github.com/shuhei-suzuki/rururunx/issues/19) Context;
[#9](https://github.com/shuhei-suzuki/rururunx/issues/9) review;
[#10](https://github.com/shuhei-suzuki/rururunx/issues/10) approvals. Issue bodies
were inspected from the saved open-Issue inventory available during this task;
they are scope references, not a claim that live GitHub dependencies are closed.
Parent tracking #22/#17 and final #16 remain open until real acceptance.

## 2. Actual foundation and gaps

| At the pinned source | Integration still required |
| --- | --- |
| `goal.rs::TaskDag::hard_order`, Store Goal reference checks | Readiness from accepted scoped definition and current verified predecessor evidence; ordering grants no launch. |
| `CompletionCriterion {id,description,evidence,satisfied}` | Typed accepted evaluators and trusted evidence; public booleans/URLs are insufficient. |
| `RuntimeOwner::open`, durable execution epoch, units/resources/effects | One retained operational Runtime, driver ownership, durable scheduling claims and restart classification. |
| `ManagedWorkflowSources::prepare`, initialize, first-unit adoption, committed frame | Actual Runtime wiring before initialization; no unregistered initial Git or fabricated Agent result. |
| `NativeLimits`, `QuotaScheduler`, Store quota leases/waiters/windows | Runtime-wide ready selection, fair cross-Project admission and persisted deadlines; component admission is not a Goal scheduler. |
| Workflow step, exact due-wait CAS, typed managed binding/publication/cancel | Serialized per-Task drivers and genuine production evidence ports; completed fixtures do not implement review/test/PR gates. |
| `main.rs` config-check/project commands | Goal/run/status/stop/resume/logs/attach control routes; a second CLI invocation must not retire the running epoch. |

Current component limits are effective global `min(config,6)`, provider executor
default 2 / total 3 (strictest alias cap can reduce them), and executor headroom
`max(global-2,1)`. Thus global6 permits at most four executors; mixed providers
can supply four independent Tasks when other conditions permit. These are current
implementation limits, not a permanent fixed-four/full-MVP contract. Runtime
must display requested and effective caps. Generalizing configured limits above
these caps requires bounded resource qualification and source/CI/native controls;
the global12 example is not currently an effective twelve-session promise.
Silently clamping and declaring complete configured-limit acceptance is prohibited.

## 3. Goal readiness and completion (RS-R1)

Each managed Goal belongs to one Registered Project. Accepted objective, criteria,
evaluator, constraints/non-goals and source references follow the existing
[#23 requirements](issue-23-requirements.md): trusted creation, immutable accepted
definition, explicit typed lifecycle and additive graph authority. Agent/native
JSON, generic persistence and planner proposals cannot mint accepted authority.
Human ingress is an application trust boundary, not biometric or same-UID security.

Managed dispatch requires Running Goal, current scoped DAG/Task ownership, every
hard prerequisite's verified Workflow completion, current required artifact/gate
evidence, supported provider/phase/input profile, and no applicable lifecycle or
external-outcome hold. Failed/cancelled/ReadyPR/transport-success-only prerequisites
do not satisfy hard edges. Soft edges are advisory. Legacy definition-unverified
Goals/Tasks are visible but cannot become managed-ready by inference. An empty
Task set does not complete a Goal. Display blockers and stored ready lists are
observations, not permissions.

Completion evaluates every accepted criterion against exact current scoped
evidence and records Passed/Failed/Unknown with producer, artifact/SHA/source and
definition versions. Missing, stale or unsupported evaluators remain Unknown with
actionable attention. At least RequiredTasksVerified must have an actual non-Human
producer/consumer test; arbitrary textual criteria are not automatically true.
Human-attested criteria require the actual trusted ingress and exact target;
native/model replies cannot masquerade as that ingress. Goal Completed is a
transactional evaluator outcome, never merely `satisfied=true`, all Tasks terminal,
native exit zero, or a diagnostic Verification Record. Unresolved external effects
and incomplete required review/verification remain explicit holds. Cleanup-only
leftovers do not invalidate otherwise verified work; uncertain resources remain
unavailable for reuse and may create honest resource-pressure waiting.

## 4. Scheduling, claims and limits (RS-R2)

One Runtime owns the configured global state; Project/Goal UUIDs and scoped Task
IDs survive restart. A durable exact epoch/Task/Workflow-version claim prevents two
drivers launching the same phase. An in-memory queue/future or expiring timestamp
alone is insufficient. Readiness is a versioned observation; admission rechecks its
complete authority and prerequisite evidence in the claiming transaction. Enabling
required typed producers is an integration gate, never an optional JSON bypass.

Native session limits count admitted unit leases before Session creation, including
executors, reviewers and approval workers. They are phase/unit counts, not a count
of Task history, spawned child processes or Task rows. Multiple members of one
review round each consume their actual native permits. Per-Project Task capacity
counts distinct Tasks currently driving preparation/effects or admitted native
units; multiple phases/members of the same Task do not multiply its Task count.
Parked pre-input waits without an executing helper or admitted native lease may
yield the driver Task slot; their durable unit/resources/claim identity remain.
An already admitted live quota-waiting turn retains its actual permit. Yielding a
slot is not a death/resource-release assertion. Preparations and evidence workers
also have finite worker/disk/output limits independently of native session permits.

All global, Project and provider limits apply atomically at the actual grant.
Alias names do not create new provider/account budgets. Project overlays can only
reduce applicable limits. Reviewer/approval headroom and fair admission must not
deadlock a round by holding partial native permits while waiting for its remaining
mandatory cohort; actual #9/#10 allocation contracts must compose before those
phases are enabled. Existing component quota leases remain the native capacity
authority; the Runtime must not double-reserve a second session permit.

Eligible Projects receive bounded round-robin opportunities, with round-robin Goals
and oldest eligible Tasks within a Goal. A saturated quota/account, held Task or
large Project cannot consume every selection attempt or starve a genuinely eligible
disjoint Project. Fairness is defined for repeated available admission opportunities,
not as a deadline for provider output or a guarantee through unresolved physical
conflicts. Global selection and provider waiter ordering must compose; outer queue
fairness alone cannot certify fairness if the inner admission rejects every turn.

## 5. Events, waits and recovery (RS-R3)

Progress uses native watch events, committed Store mutations, controller commands
and the next durable deadline. Wakeups are hints; authoritative state is reloaded.
Lost/duplicate/coalesced events, driver-future drop and restart cannot strand durable
ready work or duplicate effects. No high-frequency full-project/Agent polling or
Supervisor LLM is required. Idle Runtime blocks on events/deadlines. Queue capacity,
shutdown and storage errors have bounded explicit outcomes, never dropped terminal
facts. Cancel is serviceable while scheduling/admission/Git preparation waits.

Restart acquires the sole Runtime owner and changes epoch once, fences old grants,
recomputes readiness/limits from durable claims, units and quota data, and classifies
each old operation. Recovered rows cannot recreate live prepared capabilities or
native handles. Unknown local work remains unknown and may only continue as an
explicit fresh fenced attempt; durable accepted artifacts remain usable. Pending
PR/merge/deploy outcomes require idempotent reconciliation, never blind replay.
Unknown cleanup stays historical on the original unit. Safe disjoint work continues
unless an applicable external/common-Git/resource constraint forbids it.

Restart must reconstruct a **new committed source frame**, not just verify a
retained result. A new `ManagedWorkflowSources` has an empty Task map and its
ordinary `prepare` rejects an existing Workflow. For a Published artifact, a
private current-epoch retained-frame recovery producer reads exact registered
retained Git objects, validates complete artifact/dependency/governing digests and
installs the frame under exact Scope/Task/Workflow/generation/Context/Project/Goal
and driver-claim CAS. This grants source reading only, never old native effects,
result-finalization permission, first-unit adoption or an old Session handle.

An initialized Workflow with no Published artifact needs a separate fresh bootstrap
recovery producer. It atomically fences and records the old preparation/current
phase, preserves history and accepted policy, and reserves a newly named preparation
Unit before Git. Rebuild its exact approved committed frame under current recovery
authority; install only after current source/frame and Task/Workflow/Context CAS.
First native adoption requires this genuinely new single-use prepared capability,
never a revived row. Prior consumed or ambiguous native/external effects require
their explicit fresh retry/reconciliation policy, not this pre-input bootstrap exit.
Ready-but-unpublished artifacts remain draft/unknown; only an actual qualified
private finalization-reconciliation producer may accept them, otherwise retain
their history and use an explicitly authorized fresh attempt. Missing recovery
producer, retained graph, exact initial revision or governing approval gives a
named hold, not `initialize` replay or a fallback to live source bytes.

Confirmed quota exhaustion is WaitingQuota. Capacity, resource pressure, auth,
unknown metadata and Human attention are distinct reasons. Official structured
observations and bounded shared-pool probes follow R5; text/HTTP429 alone is not
quota evidence. A live native continuation does not get duplicate input or a new
attempt. A terminal quota-interrupted turn resumes only through a fresh namespace
and current admission. Cancelled waiters never wake into grants. Wall-clock changes
cannot create an unbounded retry loop; persisted due times remain checked.

## 6. CLI, authority changes and visibility (RS-R4)

Expose Goal inline/file creation/status/tasks/pause/resume/cancel/attach, Task
stop/fresh retry, and Runtime start/status --all/stop/restart/logs. Each scoped
operation selects exactly one registered Project by UUID, unambiguous registered
name or registered CWD identity, then resolves Goal/Task only inside that scope;
unknown, ambiguous or foreign selections refuse. Help/version/config-check retain
no state side effects. CLI/TUI controls attach to the same owner control service;
they do not call `RuntimeOwner::open` for each command. Native credential/config
roots and hooks remain untouched. Production native Goal/Task attach may return
explicit Unsupported while its actual adapter capability is absent; this is an
honest interim limitation, not attach acceptance for Issue #24 or MVP.

Allow local controls only from an actual accepted connection whose peer UID
matches the Runtime owner's UID, with the selected canonical state and exact
protocol/Hello instance and epoch bound to the current owner. Clients verify this
service identity before sending an action; the service checks current owner
authority before admitting its scoped operation. Endpoint discovery is separate
from native TaskTool/Broker ingress, which cannot construct local control authority.

Request fields, environment or `--state` selection, discovery descriptors, public
DTOs, Agent/native text and administrator/Human labels are content, not grants.
The genuine local connection and current service checks must remain necessary;
none of those supplied values can substitute for them or increase authority.

Refuse foreign peer UID, a different selected canonical state, stale instance or
epoch, and missing/mismatched protocol handshake before control effects. Do not
fall back to a foreign service or silently refresh stale consent. As in RS-R1 and
the [Goal authority requirements](issue-23-requirements.md), a same-UID host process,
including a native Agent allowed to invoke the CLI, is not distinguishable as a
biological Human; this application boundary is not a host security sandbox.

Explicit Runtime service start/restart alone acquires the sole owner lock and a
new epoch exactly once per successful startup; a second service must lose the
lock without changing the live epoch. Runtime restart after stop is this explicit
new-owner operation. Product `rrx run` means single-Task execution, not service
startup. Goal/Task resume or retry is a typed current-service command under current
authority, never Runtime restart or resurrection of an old grant/Unit/Session.
Read-only clients never auto-start a service, create state or open an owner/epoch;
when unavailable they give an explicit instruction to start `rrx serve`.
Mutating/interactive commands may optionally start the service only when that
explicit user operation authorizes service startup; startup remains a distinct
owner operation, never a hidden consequence of a read query.

Creation accepts a validated explicit graph and supported criteria, or retains
analyzing/proposed work pending trusted acceptance. It does not silently invent
Tasks/evaluators from prose. Continue through ready existing work without a new
prompt; proposed follow-ups record rationale/scope/dependencies/risk and pass #23
acceptance policy. Material scope expansion requires Human authority. Progress,
metrics and factual observations must not rewrite shared Goal/version or sibling
Task/native inputs. Applicable authority/DAG/context changes are deliberate gated
changes, not a bookkeeping exemption from existing native currency checks.

Status groups Project → Goal → Task → phase/unit/provider, showing requested/
effective limits, ready/held/wait reasons/next due, evidence qualification, work and
cleanup as separate axes, historical unknown attempts, unresolved external effects
and Human attention. No terminal巡回 is required to discover those waits. Logs are
bounded scoped references and redact secrets; unavailable token/price fields stay
null. Acceptance requires real control-plane queries, not a fabricated TUI model.

Runtime read controls for Goal status/tasks, Project routing, Runtime status --all
and logs perform no durable writes: they do not change owner/epoch, Project/Goal/
Task versions or bodies, control acknowledgements, audit or attention rows.
Background scheduling decisions remain separate producers, never effects of a
query. This condition applies to the new Runtime read controls; it does not change
the existing legacy Project-registry CLI's explicit reconciliation semantics.
Responses use bounded projections with explicit scope/version and pagination or
a stated limit/incompleteness reason. Silent truncation, unbounded hidden collection
and interpreting a partial inventory as complete readiness/no-owner proof are
prohibited; a changed version between pages cannot be presented as one snapshot.

## 7. Acceptance and independent gates

Before source: independently approve this REQUIREMENTS then DESIGN at immutable
commits; this combined draft is not either approval. Source uses STRICT schema /
authorization / concurrency review, causal mutations and Linux/macOS CI.

| ID | Required actual observation |
| --- | --- |
| RS-AC1 | Trusted Goal creation, invalid/cyclic/foreign graph refusal, legacy-unverified hold, failed dependency and forged satisfied/evidence negatives. |
| RS-AC2 | Four overlapping Tasks across two separate fixture repositories and at least two providers; same Issue numbers; distinct unit/resource/context/artifact identities. Then actual supported logged-in native runs on both OSs. |
| RS-AC3 | Exact global/Project/provider limits under competing grants, aliases, pre-Session preparation and reviewer accounting; requested/effective cap reporting and deferred-cap limitation. |
| RS-AC4 | Large continually ready Project versus small ready Project, saturated pool skipped, eligible older waiters fairly admitted; finite opportunity bound; restart preserves fairness state. |
| RS-AC5 | Last predecessor publication makes next Task advance without prompt; duplicate/lost events, due-wait races and full/closed channel converge exactly once; no idle busy loop. |
| RS-AC6 | Pause/cancel while queued, preparing, quota-waiting and running; siblings progress; late old input/result/grant refused; known work and cleanup independently retained. |
| RS-AC7 | Crash before/after claim, prepare, Session bind, native input, result publication and external intent; fresh epoch refuses old grants and does not reconstruct provenance or replay uncertain external effects. |
| RS-AC7.a | Restart after genuine Published Implement: new Sources supplies exact retained config/rules/corpus to a supported next phase while the executor workspace is changed or absent; frame-install/current-authority drift refuses. |
| RS-AC7.b | Restart after initialize before native input: new preparation Unit/namespace and actual initial gates, preserved old history, no old adoption/Session; competing recovery, cancel and source-install CAS have one winner. Ready-unpublished stays draft/unknown without qualified reconciliation. |
| RS-AC8 | RequiredTasksVerified automatic criterion with genuine published Workflow/artifact evidence; all Tasks complete but missing criterion remains incomplete; stale evidence/Human/native JSON cannot complete Goal. |
| RS-AC9 | All operational CLI observations RS-AC9.a–h below, through the compiled CLI and real control service. |

| ID | Required actual CLI/control observation |
| --- | --- |
| RS-AC9.a | Explicit service start/stop/restart, second-owner refusal and one new epoch per successful startup; `rrx run` performs single-Task execution. Read clients on absent service/state create nothing and instruct explicit `rrx serve`; Goal/Task resume never rolls the epoch or revives old grants. |
| RS-AC9.b | Goal inline/file creation retains an inert Analyzing/proposed objective until trusted acceptance; an explicit validated graph and supported criteria enter the actual typed acceptance path. Invalid/cyclic/foreign graph and unsupported evaluator refuse before publication/effects; prose invents no executable Tasks. |
| RS-AC9.c | Compiled Goal status/tasks and Runtime status --all query actual Project → Goal → Task → phase/unit/provider facts, requested/effective caps, work/cleanup/evidence and Human attention/wait/next-due reasons. Large inventories expose scoped version/cursor or explicit incompleteness; foreign cursor and mixed-version completeness refuse. |
| RS-AC9.d | Goal pause/resume/cancel and Task stop/fresh retry commit the exact current scoped lifecycle/authority decision, preserve siblings/history and refuse stale CAS/late grants. Unsupported producer paths remain named holds and do not count as successful lifecycle/execution acceptance. |
| RS-AC9.e | Logs provide bounded scoped audit/result references, redact secrets and leave unavailable metrics null. They expose no raw rejected frames, credential/config bodies or fabricated evidence. |
| RS-AC9.f | Actual supported Goal/Task native-session attach reaches the genuine adapter/session in exact scope. Explicit Unsupported correctly describes an intermediate unavailable capability but does not satisfy this row, Issue #24 or MVP attach acceptance. |
| RS-AC9.g | Registered Project UUID/name/CWD selects exactly one Project, with Goal/Task scoped inside it. Actual foreign UID/state, stale instance/epoch and missing/mismatched handshake refuse before effects; supplied request/env/descriptor/DTO/Agent text or Human/administrator labels cannot increase authority. |
| RS-AC9.h | Help/version/config-check and new Runtime read controls have no state effects; successful and refused Goal status/tasks/routing/status --all/logs leave the relevant complete before/after row images and counts, owner/epoch, Project/Goal/Task versions, ack/audit/attention unchanged under an idle controlled service. Background writes are separately attributed, and no native credential/config/hook file changes occur. |

Partial sub-ID/component acceptance never establishes complete RS-AC9, Issue #24
or full MVP acceptance. Account-free control-plane observations do not substitute
for actual supported native execution/attach, four-Task/two-Project operation,
both-OS compatibility or the other required acceptance gates. Missing producers
remain visible delivery work; successful status wiring cannot complete a Goal.

Removing readiness/claim CAS, any admission dimension, provider alias sharing,
inner fairness composition, deadline wake, epoch fence, evidence qualification,
current-result binding or non-authoritative progress separation must fail its actual
consumer control. Controlled protocol fixtures establish mechanics only; they do
not tick native/hook/quota/dogfood acceptance. Missing #9/#10/#12/#13/#19/#20/#21
producer paths leave affected phases explicitly held. No generic Passed shim may
make the full Goal complete. #16 representative two/Triple-review and efficiency
dogfood, whole #23/#24/#27 and final MVP remain required.
