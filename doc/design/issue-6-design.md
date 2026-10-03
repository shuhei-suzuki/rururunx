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
remains Lost. A native UUID or audited PID/PGID is never proof of process ownership
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

## Proposed native TUI gateway (requires independent design review)

Use the real installed native Codex TUI in an owned PTY. The TUI connects to a
runtime-owned private Unix gateway and the exact stored native UUID. Its requests
are handled by the **single** owning native RPC supervisor; do not give it an
independent upstream app-server client that can bypass the approval ledger.
The native screen, folder trust, authentication and instructions stay native.
A runtime conversation renderer is not an alternative implementation.

The initial interactive route is Consultant-only and decision-only. Executor
interactive input/grants and attachment to an active executor need separate
conformance before capability claims. Review/ApprovalReviewer cannot be converted
to an interactive mutating session. A stored prepared turn is required for native
rollout resume; its completion is not evidence of a human conversation. Between
turns, the owning server and native PTY remain supervised and their reservations
held. Native trust/onboarding remain WaitingHuman and are never selected by rrx.

Authenticate the gateway client's UID, PID and membership in the still-owned
unreaped native TUI group before WebSocket upgrade. Its directory/socket are
owner-only and checked by inode identity, with one bounded connection and one
host-terminal attachment. Never accept an audited PID or arbitrary external
client as an owning process. Every gateway request/response/notification is
bounded and correlated independently from the app-server RPC ID namespace.

Forward only conformed native methods. Read/resume/turn requests require the exact
owning native UUID; active-turn requests also require the current owned turn ID.
Resume cannot supply history, a rollout path, config, instructions, provider,
permission expansion or a new workspace. Enforce the original scoped CWD,
decision profile, disabled execution environments and exact own roots before
forwarding; verify the native response still reports them. Unknown mutations,
thread fork/list/history search, global config writes, persistent grants, remote
execution and feature re-enablement return explicit Unsupported errors.

Native UI metadata is also a scope boundary. Project-related metadata is requested
only for the exact owning CWD/UUID; do not forward another Project's configuration,
credential references, file contents or thread roster. Project/global config
responses need a minimal conformed UI projection, never a raw config dump.
Preserve installed model/effort defaults. Explicit native UI changes can be
supported only when conformed and compatible with the caller's explicit settings;
no default model/effort is invented to make the UI work.

For an initial conversation, allow bounded text input only. File/remote images,
skill attachments, tool output and opaque additional context require their own
scoped facts and are unavailable until conformed. A user message gets a private
one-time submission identity and a durable metadata-only write-ahead consumption
record before native inference. Record its byte count/digest and owning scope,
without storing the full native transcript or credentials in runtime audit.
Recheck Store Project/Goal/Task versions and locks atomically with the Session CAS
after bounded filesystem/Git checks outside the Store mutex. Once dispatch may
have begun, never replay an interrupted/uncertain user message automatically.

The single supervisor binds new native turns, usage baselines and pending callback
identities before exposing their observations. Seed and later turns cannot be
counted as duplicate completion or attributed to an older Context Pack. Decision
sessions reject target-operation grants; unsupported native callbacks are denied
explicitly with scoped audit. A future executor attachment must choose one Human
or broker grant authority, revalidate exact operation contents and journal intent
before wire approval. A native TUI choice alone is not permission to persist a
grant or broaden the runtime boundary.

PTY allocation uses safe Rust APIs and its own process-group guard. A real host
TTY attachment obtains an exclusive permit, forwards native bytes/input and
resize events, and restores terminal attributes through RAII on detach/error.
Bounded nonblocking I/O must remain stoppable when either side stalls; no blocked
stdin worker survives cancellation. Detached output remains bounded. Confirm
death of both app-server and native TUI groups before reaping and publishing
terminal/releasing reservations. Any unverified group leaves Lost with scoped
diagnostics. PTY isatty alone does not prove controlling-terminal/native-screen
conformance; actual installed CLI behavior must be observed.

Acceptance includes native text conversation on the same UUID; native trust gate
retention; detach/reattach/resize; cancellation with descendants; identity/replay,
config projection and permission-expansion fixtures; text submission CAS races;
new-turn usage attribution; compiled semantic mutations and independent source
review. Until those pass, Interactive/Attach remain explicitly unavailable.

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
Checkpoint may refresh mutable own Project metadata, preserving its canonical
repository/base/worktree identity. Observation of usage/pending grants/completion
binds one registry owner to the exact persisted turn, preventing resume races.
Provider transport completion uses the private completed journal plus Exited,
verified cleanup and matching persisted Session; OS exit zero and caller recovery
JSON cannot fabricate native success. Retryable native errors retain the turn;
resolved/completed callbacks retire pending grants with no runtime wire approval.
