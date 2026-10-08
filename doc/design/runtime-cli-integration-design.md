# Operational Runtime CLI and local control integration

Status: proposed STRICT design supplement; no CLI Runtime/control implementation
is claimed. Implements approved [Runtime requirements RS-R4/RS-AC9](../requirements/runtime-scheduler-integration-requirements.md),
[Runtime design §9](runtime-scheduler-integration-design.md#9-cli-and-control-plane-boundary)
and [Goal authority](issue-23-design.md). Source baseline
`5c070d4aa4502e9ef737fe5960e74fbd03cdb92b`. This supplements the first operational
Runtime delivery; accepted Goal/Driver producers remain a separate implementation.

## 1. First product consumer

Current `main.rs` exposes config-check and Project registration only. Add actual
service/controller consumers, using the existing global state selection and frozen
Config. The Runtime retains the sole `RuntimeOwner`; clients never call
`RuntimeOwner::open`. Existing help/version/config-check stay without state effects.
No authentication file is read/copied/forwarded, no CLI settings/hooks are replaced,
and no VM/container/root/service installation is introduced.

| Command | Actual service action |
| --- | --- |
| `rrx serve` | Foreground service: acquire one owner lock/epoch, reconcile, compose actual registry/Sources/Gates/Runtime, and accept dedicated local controls. |
| `rrx goal <objective>` / `goal --file <text>` | Create an inert Analyzing proposal; prose never manufactures runnable Tasks/evaluators. |
| `rrx goal --plan <toml>` | Validate and explicitly accept one typed definition/graph through the genuine Human ingress and approved policy, then enter Running. |
| `rrx run <instruction> --agent <name>` | Explicit single-Task typed Goal/graph shorthand, using supported fixed criterion and selected committed Project policy. Missing producers remain named waits. |
| `rrx goal status <id>` | Read actual scoped Goal facts; retain dispatch availability and named attention from the service. |
| `rrx goal tasks <id> [--after <task-id>] [--maximum <1..128>]` | Read one bounded Task page for the selected Project/Goal, with Goal version, each Task version and explicit next cursor. |
| Project routing (`--project <UUID/name>` or registered CWD) | Read-only `ResolveProject` selects exactly one registered Project for the scoped command; no client-side Store lookup or implicit registration. |
| `rrx goal pause/resume/cancel [id]` | Typed current lifecycle/control; acknowledge only a committed decision. |
| `rrx status --all`, `rrx agents` | Current Project/Goal/Task/phase/native facts and configured/effective capability/limit reductions. |
| `rrx stop/resume --task <id>` | Genuine exact Task cancellation or authorized fresh retry; no old Unit/Session resurrection. Unsupported resume is explicit. |
| `rrx stop --runtime` | Close new admissions, durably fence according to policy and perform bounded owned shutdown. |
| `rrx logs`, `rrx attach` / `goal attach` | Bounded scoped audit/result references and Runtime event/status view; provider-native interactive attach only if genuinely supported. |

The completed default interactive `rrx` opens the Runtime-wide terminal view. The
first source unit retains the existing no-argument help behavior until that view
exists, without claiming default-command acceptance. A service may be spawned
using the current installed executable only for an explicitly requested mutating/
interactive operation that authorizes startup. Explicit `serve`/restart acquires
the owner lock and new epoch once; `rrx run` is single-Task execution, and Goal/Task
resume is a typed current-service decision, never an old grant/Session revival.
A second service loses the owner lock before epoch rollover or scheduling.
Read-only status/tasks/routing/agents/logs never auto-start, create state or open
an owner/epoch. Missing state, execution directory, descriptor or listening
service gives explicit `rrx serve` guidance; clients never delete stale descriptors
or fall back to another state/service. If startup is absent in a source milestone,
give the same explicit instruction. `consult/review/approve` and native attach
require their genuine later producer and report Unsupported while absent; this
does not satisfy their full MVP acceptance.

## 2. Dedicated endpoint and lifecycle

Use a separate local Unix control socket; TaskTool IPC is never Goal/control
ingress. After acquiring the owner lock, bind a short socket in an owner-only
temporary directory and atomically write a bounded descriptor in the canonical
state execution root. Descriptor pins canonical state identity, Runtime instance,
epoch, protocol version and socket path. It contains no reusable Human authority,
credential or native input. Require private directory/file permissions and refuse
symlinks/foreign state metadata. A stale descriptor is replaced only by the new
owner after the lock; PID existence or elapsed time never grants ownership.

Clients resolve the same canonical state and perform an exact instance/epoch/
protocol handshake. Descriptor/socket/handshake mismatch refuses without falling
back to a foreign service. Restart publishes a new descriptor; live old connections
are fenced by the owning Runtime. Only the owner removes its exact descriptor on
shutdown. A failed response is an uncertain command outcome, never permission to
replay a new request ID blindly. Persisted scoped request ID + action fingerprint
and expected version make exact retries idempotent; changed replay refuses.

The dedicated handler alone constructs the Runtime module's private Human ingress
from its genuine local invocation/caller boundary. The accepted Unix connection
uses the safe Tokio peer-credential API to check caller UID against the service
UID; failure refuses, never trusts a request field or environment label. Requests have no principal,
role, authority enum or serialized capability factory. Controller policy activation
requires exact Human-approved identity/version/content digest; config/repository
text is not approval. Native/TaskTool/Broker handlers cannot obtain this authority.
A same-user process can invoke local binaries or access host files outside these
APIs; rururunx is not a security sandbox. This is an application composition
boundary, not proof of biological Human identity.

## 3. Finite protocol and selectors

One request/response frame is original UTF-8 JSON, checked by the shared strict
decoder before DTO conversion; deny unknown fields/duplicate keys, trailing values
and unsupported methods. Protocol limits: request1MiB/response4MiB/depth32; strings
and arrays also obey the shared finite profile and stricter method DTO caps. No
repair, unbounded line buffer or raw rejected-frame diagnostic. Queue maximum256
and bounded active connections; reject overload before admitting a control.
Cancellation of a socket wait does not undo an already committed command.

Selectors use exact UUIDs or unambiguous registered Project names/CWD identity.
Goal/Task selectors resolve only within the selected Project/Goal; duplicate names
fail, never choose the first row. Lifecycle/graph actions bind the actual current
expected Project/Goal/Task version and exact accepted definition/policy. A CAS
conflict is surfaced; neither client nor handler refreshes into implicit consent.
Each status response is its own coherent bounded extraction, with explicit limits
or incompleteness. Large inventories expose pages, never silent truncation or an
unbounded hidden collection; a partial projection grants no readiness/no-owner
proof. No SharedStore lock is held across socket/Git/process waits.

For `GoalTasks`, reuse the existing `ControlAction::GoalTasks` and
`state/runtime/goals.rs::runtime_goal_task_page` contract: the bare `after` Task UUID
is valid only when present in the exact selected Project/Goal inventory. Foreign
or no-longer-present cursors refuse. The CLI requests one page with maximum 1..128
and returns its Goal version, each Task version and next cursor explicitly; users
request any next page separately. `next=None` describes the end of that extraction,
not a combined snapshot with prior pages. Goal versions alone do not freeze Task
versions. There is no automatic page traversal/aggregation or combined-complete
snapshot claim, even when Goal versions happen to match. The same independent
observation rule governs later Project/Goal inventory pages; version changes must
remain visible and cannot be hidden as one coherent inventory. This decision needs
no invented version-bound cursor or new DTO authority.

All new Runtime read controls, including status/tasks/routing/agents/logs, perform
no durable writes to owner/epoch, Project/Goal/Task versions or bodies, control ack,
audit or attention rows on success or refusal. Service attention reconciliation and
scheduling are separate producers. Existing Project-registry CLI reconciliation
semantics remain unchanged; its concurrent writes surface through the actual CAS
and are attributed separately, not silently adopted by a read or refreshed consent.

## 4. Typed input and terminal view

`--plan` uses a bounded TOML DTO with explicit Project selection, Goal objective/
criteria/constraints/non-goals/fixed source refs, Task definitions and dependency
edges. Reject unknown/duplicate fields, cyclic/foreign graph, unsupported evaluator
and over-budget inputs before Goal/Task publication or Git effects. The service's
actual typed graph transaction assigns scoped stable identities and computes the
canonical accepted-definition digest; a submitted digest/accepted flag is content.
Verification commands come only from the actual approved operator catalog, not
Agent prose or Task JSON. An accepted graph is immutable except its genuine typed
additive followup transaction; material expansion remains Human-only.

The terminal view groups Project → Goal → Task → phase/unit/provider. Show current
work and cleanup independently, immutable artifact/review SHA references, wait
reason/next due, requested versus effective limits and Human attention. Event
subscriptions plus bounded level refresh update this view; rendering does not
drive Workflow or rewrite Goal/Task versions. Keep Runtime status authority in
the service, not a fabricated local TUI state. Noninteractive invocation provides
plain/JSON status rather than terminal control escapes. Native interactive attach
is distinct from this Runtime view and is never advertised from a status stream.

Logs expose bounded audit categories, scoped references and qualified result
metadata; no raw rejected frames, stdin transcript, authentication errors or
credential/config bodies. Unknown token/cost fields stay null. User-requested
answer inspection uses the typed result receipt and does not create Review votes.

## 5. Actual qualification and limits

Controls execute the compiled CLI against the real service and typed ingress.
The following observations map one-to-one to the approved RS-AC9 sub-IDs; they are
required controls to implement and run, not current PASS claims.

| Acceptance ID | Actual consumer observation |
| --- | --- |
| RS-AC9.a | Start/stop/restart the compiled foreground service; race a second owner and verify one epoch per successful startup. Absent state/execution directory/descriptor/listener reads create nothing, preserve stale discovery and give `rrx serve` guidance. Exercise single-Task run and current Goal/Task resume separately from service restart. |
| RS-AC9.b | Compiled inline/file creates inert proposed work, and bounded typed plan goes through actual trusted acceptance. Cyclic/foreign/duplicate/unsupported input has no Goal/Task publication or Git effects. Accepted positive setup uses genuine Unix ingress, never seeded private rows. |
| RS-AC9.c | Compiled Goal status/tasks and status --all expose actual hierarchy, requested/effective caps, work/cleanup/evidence and attention/wait/next-due. Observe bounded independent pages with exact Goal/Task versions and next; foreign/disappeared cursor refuses, and Task or Goal changes between pages never become a combined-complete claim. Missing first-unit fields remain explicitly incomplete, never invented. |
| RS-AC9.d | Real pause/resume/cancel and Task stop/fresh retry commit current exact scoped decisions; stale CAS, old input/result/grant refuse while siblings/history persist. Named unavailable producers remain unaccepted work. |
| RS-AC9.e | Actual scoped bounded logs redact secrets/raw rejected frames/config/credentials and keep unavailable metrics null. Read results do not fabricate Review/evidence authority. |
| RS-AC9.f | Actual supported native Goal/Task session attach reaches exact adapter/session scope. An Unsupported response is an honest interim observation, never a PASS for this row or Issue #24/MVP attach. Runtime status streaming alone is not native attach. |
| RS-AC9.g | Two isolated registered repositories exercise UUID/name/CWD routing, ambiguity and foreign Goal/Task refusal. Actual foreign peer UID/selected state, stale instance/epoch and mismatched/missing Hello refuse before effects; supplied labels/request/env/descriptor/DTO/text increase no authority. Record actual peer-UID evidence and its OS/source as specified below. |
| RS-AC9.h | Under a controlled idle service, successful and refused status/tasks/routing/agents/status --all/logs preserve complete relevant before/after row images and counts, owner/epoch, Project/Goal/Task versions, ack/audit/attention. Help/version/config-check and native credential/config/hook files remain unchanged. |

For RS-AC9.h, finish startup reconciliation and establish stable idle baseline row
images/counts, then bracket each successful/refused query with complete bounded
fixture snapshots. Prevent scheduling/lifecycle/legacy Project CLI writes during
that measurement. Run a separate idle control window and attribute any independent
service reconciliation write by its actual producer/sequence; if it prevents a
stable baseline, the no-write observation is unproven, not an ignored difference.
No attention/audit row is exempted simply because a background loop exists.

For RS-AC9.g, foreign-UID evidence requires an authorized OS/CI fixture with an
already provisioned distinct unprivileged UID and actual peer-credential/refusal
observation at the dedicated control boundary. Record peer/service UIDs and that
no control effects occurred. Do not create accounts, escalate privileges or require
root/VM/container installation for this task. If this environment is unavailable,
record foreign-UID acceptance as an unproven remaining gate. Static permission
inspection, a forged UID field, socketpair of the same UID or a mutation alone
cannot certify the actual foreign-UID observation or complete RS-AC9.g.

Further controls cover strict decoding/changed request replay, terminal-before-
subscribe and service restart, two isolated Projects/four overlapping account-free
Tasks and genuine published status/work-cleanup axes. Do not seed private accepted
rows or pass test-only origin flags as positive authority. Compile causal omissions
of endpoint identity/strict decoding/expected version/cursor scope and observe
actual consumer assertion failures, then restore exact source. Independent STRICT
source review and appropriate build/lint/Debug/Release/install checks are required.

The first source unit wires serve, real Goal status/tasks/routing and Runtime
metadata status. It must retain `dispatch_available:false`, named native holds and
`operational:false`, and disclose unavailable hierarchy/cap/unit/provider/evidence
fields. Later units add inert proposal/typed graph/lifecycle, logs/native attach and
their genuine producers on the same ingress. Endpoint-qualified components do not
implement the missing request server: per-connection failures must not end the
accept loop, overload must refuse before admission, owned connection handles and
shutdown waits must be finite, RuntimeStop must not self-join, and a shutdown error
must not become a stopped response. These are source implementation/qualification
tasks, not findings already tested or source acceptance granted by this design.

Partial component/sub-ID qualification never completes RS-AC9, Issue #24 or full
MVP; missing producers and unproven actual observations remain explicit gates.

Account-free CLI tests establish wiring. Authenticated Claude/Codex, settings/hooks,
both-OS four-Task operation, real review/approval/consult, complete recovery and
Context Efficiency dogfood remain separate required acceptance. No guarantee that
all descendants die, no security sandbox, and no complete Phase2/MVP claim follows.

## 6. Impact analysis

This contract affects future `main.rs` command consumers, `cli/endpoint.rs` bind/
accept/connect and `cli/transport.rs` strict framing, with real routing and Goal
extraction through `runtime/control.rs` and `state/runtime/{routing,goals}.rs`.
The independent-page decision preserves current request/response DTOs and schema;
it adds no Goal/Task version mutation or version-bound cursor capability. Changes
to endpoint error handling or connection shutdown must retain its UID/identity/
private-path checks and cover all endpoint consumers. The acceptance matrix adds
compiled consumer regression observations alongside existing CLI, endpoint,
transport, routing and lifecycle controls. Native credential/config/hook behavior
and legacy Project-registry reconciliation remain outside this change; concurrent
legacy writers retain their existing semantics and current CAS checks.
