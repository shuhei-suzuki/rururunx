# Issue 6: Native Codex session design

Status: implementation in progress. Installed protocol baseline: Codex CLI 0.160.0.
The installed CLI's generated schemas govern compatibility; current documentation
can describe fields absent from that version.

## Native transport and ownership

One privately launched native `codex app-server` belongs to each rrx Session. The
runtime supplies a custom Unix URI in a freshly created owner-only directory.
Installed Codex may make that path an alias to its short hashed native socket in
an owner-only temporary directory. Verify alias/target inode identities and native
socket permissions, then authenticate the Unix peer UID/PID and its membership in
the still-owned unreaped process group. An alias name or shared native directory
alone never authorizes a connection; never chmod/adopt an external shared target.
Unix transport uses WebSocket HTTP Upgrade; maintained Rust WebSocket framing
handles fragmentation/control frames and bounds incoming/outgoing messages. The
runtime never connects to the shared native daemon/default control socket.

The CLI can be a Node wrapper whose Rust child has a different PID. The owned
wrapper process group is the cleanup unit. Reuse the existing adapter's validated
group guard, kill-before-reap, optimistic Session persistence and cancellation
reservation behavior. Native Git/filesystem preflight runs outside the shared
Store mutex; persisted Project/Goal/Task versions are rechecked before dispatch.
The final admission uses one SQLite Immediate transaction for those parent
versions, the complete scoped lock/version set (including inactive lock ABA),
Session CAS and its scoped `session.saved` audit. A metadata-only consumed-input
intent contains a private submission ID, byte count, digest and input version;
neither the native prompt nor credentials enter the audit. Failed admission
restores the previous in-memory intent and sends no native inference request.
Encode and bound the entire RPC frame before consumption, including JSON escape
overhead. The single private dispatch method performs admission then sends that
prepared frame; caller-wire fixtures exercise a second SQLite writer after final
preflight and check that committed consumption precedes actual wire delivery.

Task roles validate exact worktree/common-dir ownership and prepared HEAD. Review
also verifies a clean immutable locked revision. Source-only consultation does not
canonicalize a namespace that need not exist yet. Its source is the exact owning
registered primary Git root, with a scoped read-only native profile.

## Native lifecycle

Initialize, acknowledge initialized, inspect native readiness/requirements, then
start or resume an exact owned native thread. Check returned UUID/CWD and active
permission profile before sending the prepared payload as a native turn. Explicit
model/effort are passed only when present. Native provider selection and instruction
sources are preserved; no replacement base/developer system instructions.

Correlate every response, notification and server request with its native
RPC/thread/turn/item identity. Unknown, malformed and foreign requests fail closed.
Pending callbacks are bounded and replay-safe. Default routing keeps native
automatic review; explicit runtime-broker routing uses the supported client route
only when permitted by native requirements. Replies cannot introduce persistent
grants or policy amendments. Structured output is a native per-turn capability;
schemas come from the caller, not provider-specific workflow logic.

Native turn completion and OS process completion are distinct. Persist the native
UUID for owned resume, close the private session server and verify whole-group
termination before releasing runtime reservations. A crash or unverified cleanup
remains Lost. After a consumed input, missing authoritative exact native terminal
evidence also remains Lost even when every owned process is known dead; clear the
dead PID but retain the operation reservation and never automatically replay.
An interrupt acknowledgement alone does not establish the model/operation outcome.
A native UUID or audited PID/PGID is never proof of process ownership
after a restart.

## Decision-only sessions

Use a named scoped permissions profile rather than treating the legacy read-only
sandbox label as Project isolation. Local probes show shared temporary directories
remain readable through minimal defaults unless explicitly denied; reopen only the
owning workspace roots. Decision roles use read-only files and disabled network,
and cannot approve their own sandbox escalation.

Restrict native command, browser, computer, image-generation, app, Code Mode and
sub-agent operation capabilities through their installed native controls. Disable
actual configured MCP servers using validated override identifiers and verify the
native thread inventory exposes no external tools before inference. An empty MCP
table does not remove inherited servers. Saved disabled plugin IDs in 0.160.0 do
not filter capabilities. Hooks, authentication, rule discovery and trust remain
active; incompatible managed requirements or unverified capabilities are explicit
failures, never a reason to bypass them.

## Interactive connection and observability

A native TUI may connect to the same private server and exact thread through
`codex --remote unix://PATH -C WORKTREE resume UUID`. Native folder trust is retained.
Metadata-only thread creation does not necessarily materialize a resumable rollout;
a stored turn is a prerequisite observed in the installed CLI. An alive process at
a trust prompt proves transport and native gating, not completed interactive use.
Availability requires a real terminal and verified scoped native session.

Native token notifications contain last and cumulative usage. Deduplicate by turn
and avoid summing repeated cumulative values. `last` describes one model call;
use the cumulative gauge for a fresh thread and subtract the preceding owned
supervisor's cumulative gauge on resume to include every tool/model cycle in the
current turn. Missing baseline fields and counter resets remain unknown. Keep bounded assistant/stderr tails,
nullable provider fields, and explicit unavailable cost. Preserve observed cache
zeros as reported values rather than inventing zeros for missing fields.

## Executor and approval routing

Executor uses its exact Task worktree, a scoped writable permissions profile,
disabled network, denied shared temporary roots and no profile inheritance.
The installed built-in `local` environment must report ready and receive exactly
the owning CWD/workspace roots. Empty native environments disable all execution;
decision roles deliberately use that setting. Stable `code_mode_host`, shell and
unified execution settings remain native-owned for Executor. Other external
operation capabilities and inherited MCP servers are restricted and verified.

Preserve the native approval reviewer. Native callbacks retain exact thread,
turn, item, opaque request ID and original parameters. Default callbacks surface
as WaitingHuman, with denial/cancellation available but runtime grants disabled.
`with_runtime_broker` is an explicit trusted runtime integration; it works only
when the installed policy already selects the native user/client reviewer route.
It cannot substitute for automatic/managed review. That prerequisite is checked
before a model turn, without overriding the native reviewer.

Approval replies include the exact native turn, request ID and a digest of the
reviewed method, arguments and planned operation contents. File-change callbacks
without preceding bounded native patch facts cannot be granted. Scoped target
paths are revalidated against canonical ancestors and hard-link ambiguity before
a grant; native commandActions remain best-effort display data. Queues/maps and
parameters are bounded; replay, unsupported arguments, additional permissions,
persistent roots, remote environments and terminal-input approvals fail closed.
Only one-operation accept/decline/cancel are representable. Validate native
workspace/lifecycle ownership again before a grant, journal intent before the
native wire reply, and fence the Session CAS. An intent is not evidence that the
operation completed. Audit failure prevents a grant. Store/watch status updates
publish under the same Store mutex. Pending callbacks retain native processes and
reservations until a reply, explicit stop or verified cleanup.

Current interactive/attach implementation is still pending. The transport probe
at a native trust prompt is not claimed as completed interactive operation.

## Proposed native TUI gateway (revised; conformance and re-review required)

Run the real installed native Codex CLI in an owned PTY, connecting its exact
stored native UUID to a private runtime Unix gateway. One native RPC actor owns
all upstream requests, callbacks and turn state; the frontend never gets a second
upstream connection. Native UI, instructions, authentication and trust remain
native. Initial scope is Project-only Consultant, text-only and decision-only. Executor,
Reviewer and ApprovalReviewer interactive conversions are unavailable.

Attach requires a live owning interactive seed supervisor. A terminal seed is
not reopened without fresh explicit continuation; initial attach returns
UnsupportedCapability instead. A private typed InteractiveState is persisted
in recovery: Seed, Idle, Admitting, HumanTurn, Stopping. Idle uses existing
SessionState::WaitingHuman with reason `consult_input`, retains both owned groups
and the reservation, and is not an urgent permission escalation. Human submissions
have a separate monotonically increasing `human_sequence`, advanced by Session CAS
with consumed intent; they never change the seed's Context input version or hashes.
Build candidates without changing the prior idle record. Rejected admission leaves
that exact idle record intact; consumed admission can never be rolled back/replayed.
No new public SessionState, record kind or migration is needed for these bounded
typed recovery fields. A future Task/Goal interactive route needs separate Context
source/concurrency conformance rather than implicitly using the Project route.

A completed prepared seed permits native rollout resume, but does not prove a
human conversation. Journal seed completion before accepting a human submission;
never steer or interrupt a runtime seed through frontend input. Runtime checkpoint,
resume and context refresh reject overlap with an attached or unresolved human
turn under the same exclusive transition claim. Stop remains independently
available. Scope/environment/profile snapshots are frozen for the supervisor;
changes require explicit stop and fresh admission rather than hidden rebinding.

### Native trust and frontend conformance

Preserve the exact existing trust entries relevant to the owning CWD, primary Git
root and applicable parent resolution; neither drop them nor fabricate trust in a
UI config projection. Identify the installed native trust lookup and write path
before advertising Interactive. Initial admission requires that the actual native
lookup already resolves the exact own folder as trusted. Otherwise return typed
UnsupportedCapability (unmet native trust prerequisite) before frontend launch;
the existing noninteractive route and the retained trust-screen probe remain
distinct evidence. Reject every frontend trust/config write and reload. Native
trust/settings changes require stop and fresh supervisor admission. Never select
trust, fabricate a trusted projection, or write trust/config/auth to make a probe pass.

Before the gateway binds the expected native client, report that the client is
not yet bound and its cause is unobserved. Production does not parse PTY screen
text to infer trust, approval or readiness. Fixed-label screen inspection belongs
only to installed-CLI conformance tests. Local frontend file/editor/shell/slash
commands, trust handling and descendant job control may bypass RPC; inspect and
conform those paths with native controls before claiming a runtime boundary.
Gateway allowlisting alone is insufficient. Unconformed frontend action paths
keep Interactive/Attach unavailable rather than being replaced with a chat UI.

### Gateway identity and native policy

Persist the second owned frontend PID/PGID and known SID, if available, in Session
recovery before accepting a gateway client. These are diagnostics, never restart
ownership credentials. Authenticate the connecting UID/PID and membership in the
still-owned unreaped frontend group; check owner-only socket directory/inodes.
Permit one bounded connection and one in-process host-terminal attachment; no
external attach IPC or recorded-PID adoption is part of this initial route.

Use fixed validated native argv and a frontend-only private native home with only
conformed safe UI/project/trust projections. Never copy auth, credential values,
global history, rollout files or native state databases. The inference server
keeps the actual existing native home/auth/hooks/rules/managed requirements and
defaults; the frontend uses that server's scoped account/model interfaces.
Conform client bootstrap against this home rather than inventing authentication.
Use the installed first-party `codex sandbox` with its managed requirements and
a stricter named filesystem/network profile around the frontend: own source read,
private frontend home write, required immutable executable/system reads and exact
private gateway Unix socket only. Additional controls must never weaken native
requirements. Verify actual local access, child behavior and wrapper interoperability
on each host; any unsupported path is a typed unmet prerequisite. No frontend
VISUAL/EDITOR or provider-secret forwarding exists in this initial route.
Reject foreign Git routing, history paths and native UUIDs. Initialize
the upstream once. The frontend receives a bounded cached validated initialize
response, may request only a subset of negotiated capabilities and cannot add
unknown opt-outs. Keep frontend and upstream RPC ID namespaces separate.

Every read/resume/turn route binds the exact UUID/CWD and, when applicable, owned
turn ID. Serve frontend resume from the owning already verified resume result
with fresh own-thread observations; never blindly forward frontend policy fields
or perform an independent raw resume. Reject settings, model/effort/profile,
provider, environment, instructions, history and permission changes before wire
dispatch rather than silently rewriting a UI choice. Preserve native defaults
when the caller left model/effort unspecified; an echoed UI default must not
create a sticky explicit override. Initial native settings changes are unsupported.
Conform each echoed field against values previously served by the owner. Accept
only byte-equal no-op echoes, journal their omission, and build upstream turn/start
from exact UUID plus text (and only caller-configured explicit effort). Never
forward frontend permission/sandbox/approval fields. Changed/unknown fields fail
before consumption; omission needs installed-native no-op proof for each field.

Before each human inference, perform the owning native policy/config/tool
inventory checks and bounded filesystem/Git preflight, then atomic Store admission.
`thread/read` has no active permission-profile field in the pinned schema, so it
cannot alone prove that binding; use a conformed authoritative owner-controlled
check or reject dispatch. Bind a frozen policy identity from nonsecret native layer
version identifiers, exact trust facts, permission profile and tool-inventory
digest, and compare it immediately before wire with no intervening frontend RPC.
Native changes require stop/fresh admission. Inside the same SQLite transaction,
compare the original frozen parent versions and nonsecret scope/config/reference
snapshot, rather than accepting a newly captured Project with old process state.
Forwarded environment values remain private; never persist credential-value hashes.
No
surviving external tools or unknown inventory is allowed. Project config responses
are a conformed minimal projection including exact applicable trust facts, never
raw global config, credential references or another Project's metadata. Unknown
methods, fork/list/search, global writes, persistent grants, remote execution and
feature expansion receive explicit Unsupported errors.

### Submission, callbacks and telemetry

Accept bounded text only; images, skill attachments and opaque context require
separate scoped conformance. Each submission has one private identity/origin and
only one unresolved input may exist. In one SQLite Immediate transaction, fence
Project/Goal/Task versions, scoped lock versions and Session CAS, publishing
metadata-only consumed intent in recovery and the same `session.saved` audit.
Record byte count/digest/version, never full prompt/auth values. Once wire dispatch
may begin, an unobserved outcome stays Lost even with dead owned processes; never
resend or replay it automatically. Steer/compact/review or any other inference
method uses the identical admission boundary or is explicitly unsupported.
Run bounded SQLite admission on a blocking worker under the per-Session dispatch
permit while the sole reader drains. A stop before commit prevents consumption;
if commit may already have occurred, join/resolve that owned worker and retain
Lost without sending/restoring/replaying. No other inference send interleaves.

Bind the acknowledged native turn to that submission before exposing completion,
usage or callbacks. Native terminal success requires exact IDs, private completion
evidence and verified cleanup where terminal release is requested. A prior seed
or cumulative gauge is only a baseline, never completion for the new input.
Freeze private seed submission/turn/result-digest evidence separately from human
turns and retain result tails per submission. Interactive sessions cannot satisfy
Workflow transport success; human completion never replaces a runtime result.
Human-turn Usage has existing nonnull phase `consult`, null Context Pack/round
attribution, and submission/native-turn/origin metadata. Missing/reset counters
remain nullable with a reason; do not reuse the seed's Context Pack telemetry.
Each turn uses the exact preceding owned terminal high-water gauge as baseline.
Only its bound turn-ID notifications count. A late preceding-turn update after
current-turn observations, a decrease, or an unobserved preceding outcome makes
derived values null with a reason instead of inflating or double-counting them.

Callback disposition is explicit: operation/grant requests are declined before
frontend relay and are never shown as replyable requests; surface a scoped rrx
event/audit entry instead. Initial routes have no frontend callback-reply category.
No accept-for-session, permission amendment or persistent grant
is accepted. Unsupported questions/elicitations receive a bounded native error
until their reply path is conformed. Native resolved/completed notices retire
ledger entries and are relayed only for exact own identities. Correlate each
frontend callback ID for one reply; unknown/replayed replies receive an error.
A future executor route must choose a single Human or broker grant authority and
journal verified operation intent before wire approval; it is not advertised here.

### PTY lifecycle and attachment

Safe Rust PTY allocation and the existing owned unreaped ProcessGroup guards
supervise server and frontend separately. Use readiness-based bounded I/O. Never
await a frontend socket/PTY write in the sole upstream reader: use a bounded
outgoing queue and nonblocking admission, while stop/callback draining remain
responsive. Oversize or queue overflow closes the route, records failure and
performs verified frontend cleanup; never truncate/drop protocol messages and
continue. Detached output has bounded retention and continues bounded draining.

A host TTY requires an exclusive attachment permit, raw-mode RAII, exact attribute
restoration on detach/error and supported graceful SIGTERM/HUP handling. No claim
of SIGKILL restoration is made. Resize signals must execute while the actor still
owns an unreaped frontend group; a cached PID is insufficient. Actual native
redraw/controlling-terminal behavior requires an installed-CLI conformance test.
A byte tail is not reconstructed screen state. Initial reattachment is idle-only
and needs observed native redraw or a fresh frontend after verified old-group
death, using the same owned UUID and supervisor. A version-pinned fixed redraw
trigger is proved during conformance; production issues that trigger and never
parses screen text as readiness. Without such a pin, use a fresh frontend.

The initial safe PTY path does not create/adopt a new SID or controlling terminal;
its installed native frontend must work with isatty, owned-group resize and no
local helper/job-control paths. Remote `!` commands use native thread/shellCommand
and are rejected; missing VISUAL/EDITOR prevents the known local editor spawn.
Conform any remaining local descendants before claiming group-only cleanup. A
future controlling-terminal path must own an unreaped session leader and inspect
the complete SID/TTY cohort with bounded trusted enumeration before cleanup;
it must never signal a shared parent SID or infer ownership from persisted hints.
If either path cannot prove the live ownership unit, retain Lost/unavailable.

Frontend exit during an unresolved human input requests interruption and stops
both groups, retaining Lost if no authoritative native terminal was observed.
Frontend exit, overflow and authentication failure use one bounded termination
procedure: interrupt exact own turn, keep the reader draining to a fixed terminal
deadline, then stop and verify both groups. Interrupt acknowledgement is not the
terminal event, and timeout stays Lost.
Idle exit performs bounded stop without claiming transport success. Confirm all
owned groups dead before reaping/releasing reservations; escaping descendants or
unverified cleanup retain Lost. Recovery never adopts PID/SID hints or kills global
native sessions. Production capability claims require actual same-UUID human
conversation, retained trust, local-action boundaries, detach/reattach/resize,
descendant cancellation, CAS/replay/policy/queue fault fixtures, compiled semantic
mutations and independent implementation review. Current transport/trust-screen
probes do not satisfy these gates, so Interactive/Attach remain unavailable.

## Impact and verification

Additive provider implementation and optional adapter methods; generic capability
claims remain unchanged. Shared process/preflight helper visibility may expand
within the crate while existing guards/tests retain their behavior. No new state
kind or database migration is required by this issue. Provider-neutral workflow,
review, context and registry factory integrations remain independent consumers.

Use protocol fixtures for correlation/framing/failures and actual installed native
Codex for execute/review/resume/interactive evidence. Mutate scope, native CWD,
revision, decision restrictions, callback identity and usage deduplication guards.
Re-run existing generic process/Git/registry regression tests after shared changes.

Primary references: [App Server](https://learn.chatgpt.com/docs/app-server),
[Configuration](https://learn.chatgpt.com/docs/config-file/config-reference),
[Permissions](https://learn.chatgpt.com/docs/permissions), installed generated
schemas and actual isolated native probes. Production proof is recorded separately
after implementation; reconnaissance alone does not satisfy the acceptance gates.


## Compatibility and refreshed continuation

The current experimental protocol is explicitly conformed against installed
`codex-cli 0.160.0`. Unknown native versions fail before model inference and need
fresh scope/protocol conformance. Validate initialize native-home/userAgent/local
platform metadata without using userAgent as a version or ownership credential.
Selected custom provider credential/header **references** can be forwarded from
the owning environment; unselected or foreign Project references cannot. Native
account readiness is queried with refresh disabled; credentials are never read
from account payloads or persisted, and missing required login is a typed outcome.

A terminal resume requires explicitly checkpointed continuation input with a
higher version than the last attempt. Do not repeat a cached mutating prompt.
Persist the new input revision/version/bytes/SHA-256/source versions in the new Starting
attempt before admission. Starting-to-Running and subsequent observations retain
that same source binding; failure before dispatch restores the original terminal
record and its watch state rather than consuming the new attempt.
The new Starting recovery contains `pre_dispatch_restore_sha256`, lowercase hex
SHA-256 of the exact prior terminal Session serialization. Shared context guards
can verify that proof against the prior persisted record and permit exact
historical restoration only while no current dispatch intent was consumed and
`native_dispatch_unobserved` is not true. It never permits post-dispatch rollback.
Checkpoint may refresh mutable own Project metadata, preserving its canonical
repository/base/worktree identity. Observation of usage/pending grants/completion
binds one registry owner to the exact persisted turn, preventing resume races.
Provider transport completion uses the private completed journal plus Exited,
verified cleanup and matching persisted Session; OS exit zero and caller recovery
JSON cannot fabricate native success. Retryable native errors retain the turn;
resolved/completed callbacks retire pending grants with no runtime wire approval.
