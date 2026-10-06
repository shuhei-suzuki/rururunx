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
| `rrx goal status/pause/resume/cancel [id]` | Scoped current status or typed lifecycle/control; acknowledge only a committed decision. |
| `rrx status --all`, `rrx agents` | Current Project/Goal/Task/phase/native facts and configured/effective capability/limit reductions. |
| `rrx stop/resume --task <id>` | Genuine exact Task cancellation or authorized fresh retry; no old Unit/Session resurrection. Unsupported resume is explicit. |
| `rrx stop --runtime` | Close new admissions, durably fence according to policy and perform bounded owned shutdown. |
| `rrx logs`, `rrx attach` / `goal attach` | Bounded scoped audit/result references and Runtime event/status view; provider-native interactive attach only if genuinely supported. |

The default interactive `rrx` opens the Runtime-wide terminal view. A service may
be spawned using the current installed executable only for an explicitly requested
mutating/interactive command. Read-only status/agents/logs never auto-start or
change an epoch. `serve` is also available without background launch. A second
service loses the existing owner lock before scheduling. If service startup is
not implemented in a source milestone, give the explicit `rrx serve` instruction;
do not imply full default-command acceptance. `consult/review/approve` require
their genuine later producer and report Unsupported while absent. These are
remaining MVP gates, not completed by this first consumer.

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
Status is a coherent bounded extraction with explicit pagination/completeness.
Large inventories are paged; truncated data never establishes no owner/readiness.
No SharedStore lock is held across socket/Git/process waits.

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

Controls must execute the compiled CLI against the real service and typed ingress:
help/config purity; registered Project resolution; typed graph positive and cyclic/
foreign/duplicate invalid input; read commands with unchanged live epoch; service
startup race; strict decoder/changed request replay; committed lifecycle and stale
CAS; published status/work-cleanup axes; two isolated Projects/four overlapping
account-free Tasks; terminal-before-subscribe and service restart. Do not seed
private accepted rows or pass test-only origin flags as positive authority. Compile
causal omissions of endpoint identity/strict decoding/expected version and observe
actual consumer assertion failures, then restore exact source. Independent STRICT
source review and appropriate build/lint/Debug/Release/install checks are required.

Account-free CLI tests establish wiring. Authenticated Claude/Codex, settings/hooks,
both-OS four-Task operation, real review/approval/consult, complete recovery and
Context Efficiency dogfood remain separate required acceptance. No guarantee that
all descendants die, no security sandbox, and no complete Phase2/MVP claim follows.
