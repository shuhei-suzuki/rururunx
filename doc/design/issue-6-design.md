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
Stop and approval cancellation keep the same sole native reader active for a
bounded five-second terminal drain. The interrupt response is correlated but is
not terminal evidence. Exact callbacks received while draining are declined and
audited without a grant; new caller grants are rejected. An exact interrupted
turn becomes Stopped only after verified cleanup, while an absent/foreign
terminal remains Lost. A completion racing stop retains its actual native outcome.
A native UUID or audited PID/PGID is never proof of process ownership
after a restart.

A fresh continuation pins its input metadata while Starting. Retire the prior
`native_turn` into `previous_native_turn` before that publication, so a pending
new input cannot appear to belong to the old turn. The new acknowledged turn
later supplies the current identity. Pre-dispatch rollback restores the exact
historical Session, including its original turn and consumed-input marker.

## Preparing continuation cancellation (design approved; source review pending)

Fresh start, resume and checkpoint atomically install a private per-attempt control when
claiming the registry transition, before any await or Starting publication.
The design at `26945b0` passed its fifth independent native design review. That
approval does not establish implementation or native containment. Private
cancellation-aware Git/bootstrap waits retain their actual owned children. The
registered per-attempt control and retained task now cover fresh start, resume,
checkpoint and stop, with one stop receiver transferred through acknowledgement
into supervision. Causal consumers exercise the actual SQLite admission CAS,
owned Git/bootstrap and scoped Unix RPC paths. Immutable independent source review
and final combined checks remain pending; preparing-lifetime closure is not claimed.
A terminal shared watch does not exclude preparation. Stop captures the current
control Arc and exact owned scope under the registry lock; its target remains that
attempt for the whole call. It never re-resolves a later registry control, sends
to the retired channel or waits on the shared watch's latest value. Existing
public SessionRef stays unchanged; the first poll defines this invocation's target.

Use a small synchronous admission mutex with Preparing, CancelledBeforeAdmission
and Consumed states, plus terminal Failing(kind) before admission and terminal
CheckpointCommitted for a checkpoint, and a
separate level-triggered watch<AttemptPhase>.
AttemptPhase is Preparing, AwaitingTurnAck, Supervised, or Finished(TypedOutcome).
TypedOutcome is RestoredBeforeAdmission{cause: Cancelled | Failed(kind)},
CheckpointCommitted{input_version}, FailedBeforeAdmission{cause,snapshot},
Terminal{snapshot}, Lost{cause,publication_result}, RestoreUnpublished{cause,error},
or FreshUnpublished{cause,error}. A terminal snapshot includes the exact serialized
Session and native attempt identity, rather than a raw Store record version.
Every Finished variant also retains its published Session snapshot, or None when
publication failed, so an idle stop never returns another attempt's current status.
A captured attempt retains
its own terminal snapshot and outcome even if a newer preparation replaces the
shared watch. A waiter subscribes before checking the level and never relies on
Notify edges. Finished is published only AFTER owned async cleanup and the final
Session publication or its explicit failure; the Reservation owns this last act.
Transferred supervision is Supervised, not Finished. The sole supervisor writes
this same attempt's terminal outcome after cleanup/publication, including errors.
Fresh start registers its Preparing entry/control before its first Starting persist,
under the same registry/control/Store order. It uses the same owned preparation task
and Preparing-only caller-drop guard as resume. A failed initial persist removes only
that exact installed Arc and publishes an unpublished outcome; it never adopts another
entry. A pre-admission cancelled or failed fresh start has no historical Session to
restore: after verified cleanup it persists Failed with the factual cause; unknown
cleanup/publication retains Lost/error. The entry is visible for stop/status once the
Starting record is visible. No Store write precedes registered control ownership.
Every fresh preparation persist, including discovery/native PID and native binding,
and its conservative Drop publication update that same watch while holding Store.
This is the same ordered publication used for resume; no stale provisional watch
may diverge from a successfully persisted preparation Session.

Stop atomically cancels Preparing, or queues an interrupt to the captured attempt's
new channel if Consumed/Supervised. Hold the admission mutex through the exact
consumed-input Store CAS, reject cancellation with zero turn/start writes, and set
Consumed only on successful CAS. A failed CAS atomically moves Preparing to
Failing(the actual typed error) in that same control critical section, with exact
in-memory intent rollback; it is unconsumed but cannot be relabelled by later stop.
Pre-encode the full
bounded frame first. Use registry then control then Store lock order; admission
uses control then Store. Never take control from Store or hold a lock over await.
Checkpoint's final validated-request replacement uses registry/control/Store in
that order with lock-held helpers, never current()/reference() reentry. Cancellation
winning before replacement preserves the prior request and exact prior Session.
The successful checkpoint commit changes its control to CheckpointCommitted in
that SAME critical section as request replacement and exact Session restoration.
Stop cannot change this terminal admission state to Cancelled. Its factual outcome
is CheckpointCommitted{input_version}, even in the interval before Finished is sent.
Both linear orders distinguish a cancelled unchanged request from a committed new
input that a later resume actually consumes; no cancellation classification follows
a committed checkpoint or a Failed(kind) restore.

Linearize the first pre-admission terminal decision under the control mutex:
Preparing becomes CancelledBeforeAdmission only when cancellation wins, or Failing(kind)
when a detected preparation error wins. A later stop joins Failing and cannot relabel
it cancellation. A won cancellation alone supplies cause=Cancelled. Cancellation-aware
Git, bootstrap and RPC waits return a distinct private Cancelled result after cleanup;
they do not map the resulting killed child exit to OwnershipMismatch, LaunchFailure or
StateConflict. A real ownership/security failure linearized first retains its kind and
metadata-only audit. Later observations cannot rewrite the first cause; a separately
observed security failure is recorded without inventing success or allowing dispatch.
Consumed execution still uses its authoritative native terminal/drain outcome.
Route every pre-admission error exit, including propagated errors, through one
private cause-selection helper before any cleanup await; no generic early return
can leave a known failed attempt Preparing.

Stop awaits only its captured attempt's level-triggered outcome. For restored
pre-admission cancellation return StateConflict with a factual cancellation/restore
classification ONLY when this invocation won or joined Preparing cancellation.
Read phase before admission state: capturing an already Finished attempt is an idle
terminal stop and returns its matching persisted snapshot, regardless of the cause
of an earlier restore. A committed checkpoint returns the actual restored terminal
Session as Ok(status), never cancellation; CheckpointCommitted{input_version} remains
the private factual outcome used to select that snapshot. A failure of
an attempt that this stop joined returns its actual Failed(kind) classification.
Usage of that restored Session remains the prior actual turn, with no new turn
attribution. For a consumed attempt, return its own terminal status only if the
persisted Session exactly matches its snapshot and native attempt identity. Do not
compare raw Store versions: Starting and exact restore legitimately increment them.
An idle stop after a committed, cancelled or failed checkpoint, or a cancelled resume,
still returns the prior terminal Session. The Finished-before-control-restore window
also returns that matching terminal Session without claiming this stop cancelled it.
If Session
has advanced to attempt B, return StateConflict identifying advancement, with no
cancellation or interrupt to B. Unpublished restore or cleanup uncertainty returns
an explicit error; a completion signal alone never establishes successful stop.
A fresh pre-admission cancellation returns its newly persisted Failed Session as
Ok(status), with the actual cancellation cause and no native completion/exit claim.
The advancement error carries the captured attempt's own factual outcome instead
of losing evidence that A actually stopped before B advanced.

The per-attempt task is owned independently of the caller future. Make the private
adapter registry Arc-shared so the task can own immutable adapter handles and the
TransitionClaim. Keep its JoinHandle in the registered attempt, release that handle
on task completion through that exact control Arc, never the registry's current
lookup, and never abort it when a caller disappears. A caller-side
guard requests cancellation on drop; it cannot claim native completion. The owned
task continues bounded cleanup and exact restore before completing the attempt.
The guard only cancels Preparing, never interrupts Consumed/Supervised, and is
disarmed when the caller takes its result. Losing that admission race leaves the
actual consumed turn supervised; it does not silently inject an interrupt.
Panic/runtime shutdown retains the existing conservative Drop/Lost fallback and
finishes with a typed uncertain/unpublished outcome, never a fabricated restore.
No task may be spawned without its registered control/transition ownership.
The private task owner claims Installing before spawn but holds no owner mutex
across submission. A closed Tokio runtime can synchronously drop that submitted
future. TaskGuard releases its own slot to Released; installation observes that
level and drops the completed handle rather than keeping it or deadlocking on a
nested release. Missing async runtime returns a typed launch failure and the
conservative unpublished drop outcome. Handle release also drops JoinHandle outside
the owner mutex. These private owner states do not alter public Session lifecycle.

Check cancellation before Starting publication, every child spawn, thread/start or
thread/resume, each inventory page, final binding and checkpoint publication.
Cancellation-aware bounded Git waits retain their ProcessGroup outside the select:
on cancel explicitly kill/inspect/reap asynchronously before returning. Preserve
the existing five-second Git budget and process inspection/death guards. Expose
this through an additive private helper; existing callers keep unchanged behavior.
Native bootstrap similarly retains its group locally through cancellation and
awaits owned shutdown. Cancellation of RPC setup borrows an existing NativeServer;
drop only that RPC future, then await native.shutdown before restore. Never select
away a whole future that exclusively owns a child and then claim cleanup proved.
Read-only filesystem jobs retain existing bounded handling and carry no model
input or Store write. Intermediate cancellation prevents later spawns/UUID setup,
not merely the final turn/start. Once consumption commits, keep the same stop
channel through the existing bounded turn acknowledgement; on acknowledgement
use the five-second terminal drain. Missing acknowledgement remains Lost. No
existing native RPC/start/cleanup/inspection deadline is relaxed.

Controls live until final cleanup/publication. The transition owner restores the
old terminal control only on matching installed Arc identity; successful resume
keeps that same new control for supervision. Restore leaves the terminal Session
snapshot current despite incremented Store versions. Release the TransitionClaim
in the SAME registry critical section that installs new evidence, reply channel,
publisher and Supervised phase; the supervisor owns the control, never that claim.
This makes pending_approvals, usage and transport_succeeded observable throughout
the resumed native turn, including WaitingApproval and WaitingHuman. Old-control
restore and JoinHandle release never run while holding Store; observation takes
registry then Store. A closed captured stop channel waits on its own level-triggered
phase and cannot report SessionLost from a retired receiver. Separate phase/status
channels prevent a later checkpoint/resume from stealing a stop completion.
No schema, caller-provided ownership token, native auth/hook/trust change is added.

Tests gate actual fresh start/resume/checkpoint before Starting, during Git/native bootstrap,
before admission and after consumption/before ack. Assert no later spawn or
thread/resume after pre-admission cancel, zero input writes/consumption, exact
restored Session/request/evidence, verified owned cleanup and bounded completion.
Exercise both stop/admission orderings, queued interrupts and unobserved ack Lost.
Stop after Finished-before-old-control restore and stop after long supervision
must not miss notification. Gate stop-A's waiter; finish A and start checkpoint B
and resume B, then wake A: no B cancellation or interrupt and A returns bounded.
Inject restore CAS failure and consumed-then-RPC timeout to assert typed outcomes.
Drop callers at every held preparation window: owned task restores asynchronously;
panic remains Lost, with panic cause distinct from publication failure. Gate idle
stop after start/Exited and each checkpoint/restore outcome, plus fresh-start stop
and Finished-before-control-restore; a raw-record-version comparison mutant must fail.
Gate both checkpoint commit/stop linear orders and prove the committed input is
resumed, killing a mutant that leaves Preparing after request replacement. Resume
into WaitingApproval, read pending_approvals and usage, then submit its exact hash
and assert exactly one native reply; keeping the transition claim through supervision
must fail that consumer. Compiled mutants remove cancellation checks, mutex ordering,
channel transfer, level-triggered Finished, attempt-bound lookup, publication-before-
completion or owned task lifetime. Kill each with causal consumer assertions,
restore exact source and run controls. The installed-native descendant/Decision-CWD/
config-provenance findings remain separate merge blockers. Independent design
approval precedes implementation; actual scope additions get shared-helper regression.

Cancel during gated Git and bootstrap must return the private Cancelled cause,
never an invented ownership/launch failure. Gate actual worktree inode replacement
and stop in both orders: failure-first retains audited OwnershipMismatch, cancel-first
retains Cancelled with any later ownership observation separately recorded. Kill
mutants using generic non-success child-exit mapping or final-state cause inference.
Gate a real second-writer admission CAS failure while holding native cleanup, then
call stop/drop the caller: zero wire writes, unconsumed input, retained first actual
typed failure and matching final Session/watch. Kill a mutant leaving Preparing
after failed CAS.
Timeout/drop fresh start during Git/bootstrap must leave verified cleanup and Failed,
not unconsumed Lost; stop/status through its persisted Starting reference must observe
the exact owned preparation. Kill transfer-only registry insertion with that consumer.

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

Prior component implementation adds provider methods without changing generic
claims. The proposed F1 correction below supersedes native Codex executable
readiness claims and requires its named consumer integration before acceptance.
Shared process/preflight helper visibility may expand
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

Lock snapshots use the exact optional Project/Goal/Task Scope that admission
checks. `Store::records` is a descendant listing when Goal/Task IDs are absent;
filter those records before constructing the CAS lock set. A source-only Project
or Goal consultation must not inherit another Task's active or historical review
lock. Task-scoped execution/review continues to fence its full exact lock set,
including inactive versions so unlock/relock ABA remains detectable.

Rejected grant preflight checks cleanup uncertainty only for groups created by
that preflight. The already running native server has an owned live group flag;
it must not make every rejected grant fatal. Retain all new group flags in the
Session owner throughout cancellation. A new unverified Git cleanup still ends
the attempt as Lost; a scoped path/version rejection with confirmed cleanup keeps
the native request pending and deniable without an accept intent or wire reply.

The predecessor control must already be Finished before a terminal shared watch can
admit a new checkpoint/resume. The sole final successful Store/watch publication
has no later duplicate shared send. Starting publication is tracked per attempt,
separately from the historical record version: a cancelled invocation that has
not written Starting restores its exact previous private outcome without any
Session CAS or audit write. Once supervision installs its new evidence journal,
abnormal Reservation drop clears that same journal. First-cause classification
remains independent from cleanup uncertainty; Lost preparation returns SessionLost
and retains a bounded private cleanup diagnostic, while durable audit records
only canonical uncertainty reasons, failure kinds and a detail-retained flag.

## F1 correction: owned workload and real consumer gates (proposed)

Requirements gate: exact `a79850c29dbbc0c43d4ba6fd33143b4f2554212c` passed two
independent native requirements reviews (no Critical/High/Medium findings).
This section is a proposed STRICT design, not source/backend acceptance. It
supersedes earlier wording that treats the owned wrapper process group as the
complete cleanup unit. F4's registered preparation/cancellation mechanisms remain
component authority; they do not establish whole native workload cleanup.

### Evidence and supported-profile floor

Pinned Codex `rust-v0.160.0` source at
`a956835d020762cb2b570053af06f643a11c0ecc` has native PTY `setsid` and shell
TTY detachment. Its command termination/shutdown acknowledgements and logical
background inventory do not export a retained whole-workload death authority.
Primary pinned authorities: [PTY child terminal setup](https://github.com/openai/codex/blob/a956835d020762cb2b570053af06f643a11c0ecc/codex-rs/utils/pty/src/pty.rs#L645),
[shell spawning](https://github.com/openai/codex/blob/a956835d020762cb2b570053af06f643a11c0ecc/codex-rs/core/src/spawn.rs#L101),
[local termination](https://github.com/openai/codex/blob/a956835d020762cb2b570053af06f643a11c0ecc/codex-rs/utils/pty/src/process.rs),
and [unified_exec outcome](https://github.com/openai/codex/blob/a956835d020762cb2b570053af06f643a11c0ecc/codex-rs/core/src/unified_exec/process.rs#L259).
Owned finite standalone command probes observed a second-SID child continue after
native command termination and separately after stdio-server exit. The child
self-expired; only the actual unreaped server's selected group was independently
inspected and reaped. This is no model/unified_exec/adapter acceptance proof.

Current production profile readiness is unproven for both required hosts. The
initial correction must have no production backend permit, and refuse affected
execution before effects. It cannot replace macOS/Linux native Core with an
unsupported-only release. Real both-host Executor/Reviewer/Consult, resume/stop,
Task-native attach and four-plus concurrency remain required acceptance gates.

The macOS26.6.2 sandbox-exec deny-fork singleton only proved direct-Rust-binary
metadata initialize/config-read. The ordinary Node shim failed spawn; required
hooks, Review and sandboxed ApplyPatch helper compatibility remain unproven. It is
not an installed native general-profile permit. No required native hook/default
is disabled, no Node launcher omission is silently generalized, and no platform
version/privileged prerequisite is changed. Linux delegated cgroup-v2 is a design
candidate, not an available macOS backend or verified Linux deployment.

### Proposed private roles and enabled routes

Names are private non-Serde proposals, not an existing API. ProfileReadiness is
attempt-free; ProfilePermit is attempt-bound under the precise ordering below.
ProfileBinding fixes the widest enabled startup/native/frontend/service lifetime,
exact launcher chain and actual effective controls/inputs, completeness basis and
compatible effect epoch. It never contains credential values or kill hints.
OwnedWorkload contains the single actual backend/resource anchor; controls reference
the same retained owner, never clone an independent grant. WorkloadCleanup is a
private factual current observation created by that actual owner; selected-group
death is a strictly smaller observation and cannot construct complete cleanup.

No public no_commands/cleanup_verified flag, native telemetry PID, fixture JSON or
arbitrary adopt/kill(number) interface exists. Every enabled immediate/deferred
route must enroll into the actual owned unit, have an explicit reviewed retained
delegated-service owner, or be blocked before execution by identity/input-bound
effective control. Otherwise the profile is unavailable. Parent/cohort emptiness
cannot certify delegated job cleanup. Reduced model tools are not a file-only
Executor permit. Required native config/hooks/auth/trust/defaults are preserved.
The widest lifetime profile cannot expand through permission replies, broker grants
or frontend attach. Expansion needs an actual fresh higher-version attempt after
prior cleanup/current authority and the corresponding ownership contract.

Managed Task/formal Reviewer composition requires the actual Issue 19 contract; non-Task Consult the actual Issue 58 contract;
approval action decisions actual Issues 9/10 retained member/slot bound to original request,
input/source and own native lifetime; Task attach actual Issues 6/11/15/19/14. Actual formal
Reviewer phases are RequirementsReview/DesignReview/ImplementationReview/SecurityReview.
DecisionReview/DecisionTask is only pending conceptual shorthand, not an enum/port.
No missing producer is supplied by this proposed interface; each is an explicit
pre-effect refusal/deployment blocker, never narrowed required product acceptance.

### Readiness, effect ordering and retention authority (Design3 proposal)

This correction refines Design2 before source. Proposed names are private roles,
not callable ports. `ProfileReadiness` is an attempt-free, strictly non-executing
role/scope/repository/host descriptor. Its implementation-owned admission set
binds the widest startup lifetime extent, the exact installed executable and
launcher/interpreter/platform-child resolution chain, immutable conformance basis,
and effective controls/inputs whose authority can genuinely be established without
execution. No Project TOML, environment, Store row or public builder can add an
identity to that set. Unknown or unbounded identity/input closure is unavailable;
it is never a reason to run an unrestricted discovery probe.

A composed driver freshly revalidates bounded non-executing metadata BEFORE
clear_hold, capture, reserve or marker. Its bounded blocking worker runs without a
native process, owner resource or operation marker; caller drop cannot leave a
workload. For every actual executable/shim/interpreter/platform child and effective
config/trust/profile/attribute input, check no-follow device/inode/size/mode/owner/
mtime/ctime and the resolved indirection chain. Missing, replaced, changed or unknown
inputs return a typed unavailable result before reservation, including binary updates
and edited native config. Cache content digests only under the complete metadata
identity; never use cached readiness alone for current admission. Bounded-input and
filesystem limitations are part of the supported profile. Permit revalidation and
actual exec enforcement still cover changes AFTER that check; metadata equality is
not immunity to a concurrent replacement.

`capabilities()` and `probe()` conservatively describe their actual callable route,
not a backend alone. Executable capabilities require BOTH implementation-owned
profile readiness AND the real composed route producer and durable hold consumers.
Execute/Review on a managed route require actual #19, Consult requires actual #58;
Resume, ContextCheckpoint, PermissionInterception and derived NonInteractive/
StructuredOutput/usage features obey the same route conjunction. A managed-only
producer cannot enable flags on the legacy start API that still refuses. Current
Workflow uses the legacy capabilities/start pair: those flags stay absent, even
under a fixture-ready backend, until that exact consumer selects the composed entry.
The future managed driver consumes its own reviewed route readiness before reserve;
no new trait/API is claimed here. Synchronous descriptor calls cannot run discovery
or claim caller-specific current admission from a cached bit.

`ProfilePermit` is different: after pure current validation and the actual owning
reservation contract, it binds a fresh opaque attempt identity to revalidated
readiness/effect inputs and the actual workload owner. Identity at actual exec must
match the verified entrypoint, interpreter, PATH/shim child resolution and bound
control inputs under the backend's continuously enforced contract. Path digest
then unrestricted path exec is insufficient. Mismatch before effects is refused;
after effects it is Unknown/held. No presently available production execution
mechanism proves this whole chain; this is a backend acceptance obligation.

Public start/start_structured/resume/checkpoint/attach and aliases run their
readiness/ownership-contract gate BEFORE registry insertion, terminal eviction,
claiming a transition or replacing a control, as well as before the first await,
Starting write, native/Git/helper process, connection, consumed-input publication
or model frame. Initial empty production backend leaves registry, previous Session/
history/input and Store unchanged. Repeat calls do not evict a held entry or cause
a retry/fallback launch. Independently of backend readiness, all unmanaged
Task-scoped native launches remain Unsupported until the actual #19 managed
ownership producer composes, including legacy/untyped/ReadOnly paths. Non-Task
routes independently need #58. A permit never converts legacy start into a producer.

For a genuinely composed route, ordering is:

1. Pure bounded validation and fresh non-executing route/identity/overlap readiness;
   mint an opaque attempt key and claim that conflict extent's capacity, without
   creating an owner/resource or model grant.
2. Actual reservation, then atomic marker plus private operation. The actual driver
   retains the move-only claim and operation in its independently owned handoff
   task from marker commit; no await/caller-drop gap exposes an orphan handle.
3. The private managed entry is synchronous, not `Box::pin(async launch(...))` whose
   registration starts on first poll. In one no-await section revalidate readiness/
   overlap, bind the keyed claim, move the exact operation into registered Preparing
   control, and install/spawn its retained owned task. Only then return an observation
   future. Same logical handoff applies to managed resume/checkpoint. A closed runtime
   or registration failure leaves the actual operation/claim retained, or consumes
   the actual pristine-refusal port if genuinely no effect and publication succeeds;
   returning Err or dropping an unpolled future cannot release it.
4. Private effects_started BEFORE any resource-creating owner acquisition or Starting
   publication; tracked setup acquisition; native work and model-input admission.

The pre-marker claim is move-only RAII: refusal before a marker returns capacity.
Once marker-bound, Drop transfers to the same independent retained owner table;
it does not free capacity or release the durable operation. Registration and
handoff cannot replace a previous current control until the new exact control/
claim is retained. Driver and adapter must share a reviewed synchronous handoff,
not independent future Drop assumptions. This producer is still pending #19 source.

An acquisition that creates a cohort/subscription/service resource is itself an
effect. Truly no-effect failure may consume the actual private pristine refusal
port. Partial acquisition or post-Session failure needs actual tracked setup
outcome and complete setup cleanup, with no private admission/consumption/uncertainty.
The creator retains partially acquired resources until verified cleanup or held
uncertainty; an Err cannot discard them. NoCurrentDispatch is never inferred from
rollback, a row or selected-group death. Consumed/admitted zero-wire remains held
pending separately reviewed #14 recovery; it has no inferred native outcome or
receipt. Public #19 Design24 at `7bee9031b6b1c2690e007c0557267493d404a9e2`
is a candidate, not an available implementation-owned port.

The discovery server currently launches with policy=None before DecisionPolicy is
resolved. Its startup needs ownership of the widest reachable helpers/hooks/MCP/
tools, or authoritative strictly non-executing resolution and already-effective
bound restrictions before launch. Output from unrestricted executing discovery
cannot retrospectively mint a smaller permit. Mandatory native config/hooks/defaults
remain preserved; inability to cover them makes that profile unavailable.

The bounded private retained-owner table is separate from status registry and
replaceable F4 controls. It stores exact opaque attempt/operation keys AND current
conflict extent: Project, canonical primary root, Git common directory and worktree,
plus bound effect/profile epoch. No record/PID import or independent-owner Clone
exists. Actual adapter admission consults live AND held entries before marker and
again during synchronous handoff/permit acquisition. Any held/unknown overlapping
extent, or live overlapping extent without continuously enforced reviewed matrix
compatibility, refuses admission. Different Task IDs do not make common-Git effects
disjoint. Own-table checks are necessary, not protection from other providers;
actual #19/#58/#60 common conflict consumers must compose before readiness.

Capacity is partitioned by canonical conflict extent, coalescing Projects that
share a common directory. Each registered disjoint extent receives a guaranteed
quota: at least configured max_tasks_per_project live Task slots, explicitly bounded enabled
Consult/frontend/runtime setup slots, and a positive bounded held-entry reserve.
The implementation fixes numeric bounds for each enabled route before readiness;
no caller can raise them. Global memory bound is the sum of quotas for the bounded
registered native-enabled extent roster, not a shared exhaustible admission counter.
New extent activation reserves its guaranteed quota before advertising native readiness
or refuses that activation; it does not forbid registering a Project for read-only
status. Checked sums and allocation limits refuse activation before marker/effects,
never reclaim another unsettled extent's quota. Existing disjoint extent guarantees
survive this refusal. No unbounded count/allocation is accepted as bounded memory. An extent at quota gets a typed
scope-local capacity refusal; A's Lost entries never consume B's guaranteed quota.
The current config's global_max_sessions bound is not silently reused as this hold pool.

The actual creator/preparation/supervisor synchronously transfers the same sole
anchor into its keyed slot; controls only reference it. Ordinary release or eviction
cannot remove an unsettled owner. The genuine owner may retire its in-memory resource
anchor after recording factual COMPLETE enabled cleanup and closing all its own
observers; a durable absorbing Lost/operation/conflict hold still survives unchanged.
That retirement frees resource capacity, never admission into the held extent,
settlement, continuation or replay. Cleanup Unknown retains the anchor/slot. A
failed cleanup-record publication retains both. Actual #14 recovery alone may
release a Lost hold; no such port is claimed. Marker-bound pristine refusal frees
the slot only after the real no-effect settlement publication, never from Drop.

Session-less startup owners remain reachable through a private operation/attempt-
keyed retained-control route: exact existing owner may request stop, observe bounded
facts and record complete factual cleanup without a SessionRef. Non-executing status
can expose a redacted held identifier/reason, not reconstruct/adopt the control or
release the hold. Removing an unpublished Session registry entry cannot remove this
route. Transition restoration and failed initial persistence likewise cannot destroy
an unsettled anchor or replace a Lost current attempt with historical settled control.
OwnedWorkload Drop is best-effort, never a cleanup certificate. Runtime exit cannot
preserve handles magically: durable holds survive, but genuine restart ownership/
release needs separately reviewed #14 recovery, never audit-number reconstruction.

### Complete cleanup consumers and inspection resources

These actual current callers must consume the corrected private predicates:

| Current authority in cab665d56f7032f086946c1acfce448f5b9d076d (source unchanged in Design3) | Required correction |
| --- | --- |
| session.rs:294 Reservation::finish_preparation_error; context-closure early errors | Before effects require genuine pristine; after any setup/Git/native/resource effect require tracked complete cleanup before Failed/restored outcome or scoped reservation release. |
| session.rs:1010 prepare_checkpoint commit; 1221 prepare_launch; discovery result_after_cleanup at 1445 | Checkpoint restoration, policy discovery success and preparation errors cannot certify cleanup from selected groups only. |
| transport.rs:320 failure_after_cleanup /332 result_after_cleanup and launch failure branches | Preserve original cause, represent workload uncertainty separately, and never return a successful discovery value as complete cleanup authority. |
| session.rs:170 RegisteredTransition::Drop / restoration; register_fresh at 715 /register_existing at 757 | Preserve unsettled owner's independent anchor through initial-persist failure or old-control restoration; readiness runs before registry mutation. |
| session.rs:546 Reservation::terminal; supervised terminal at2346; transport_succeeded1679 | Exact current native outcome AND complete enabled workload cleanup plus persisted exact current scope/input/source authority, not ACK/terminal labels. |
| stop at 1704; release at 1819; evict_one_terminal at 110; resume at 1813; checkpoint at 1931 | Captured attempt identity remains exact; unknown setup/workload prevents ordinary release, eviction, restoration, continuation or reuse. |
| preparation.rs:257 bounded_git; session.rs:1396 native --version; ownership.rs:345 verify_binding /360 verify_git /369 verify_git_preparing | Require actual sealed #60 job enrollment before any execution, exact bound Git/helper identity/input chain and its retained settlement; fresh Preparation is not enrollment. |
| transport.rs:135 NativeServer::launch /153 launch_preparing; adapter.rs:338 ProcessGroup::new | Native spawning requires the borrowed sealed attempt enrollment token; fresh-Preparation launch cannot bypass it. ProcessGroup is only a selected-group component, never token/cohort authority. |
| session.rs:1999–2087 answer_approval preflight, uncertain_since classification; proposed frontend input | Grant-time Git needs actual #60 enrolled job; revalidate widest profile/identity/effective inputs and deny expansion. Non-fatal uncertain_since classification is not cleanup. |
| private retained table admission / handoff / permit | Gate exact extent overlap with live/held entries before marker and again at handoff; scope-local quota and genuine complete-cleanup anchor retirement never release durable Lost. |

Every Codex spawning helper must require a borrowed sealed enrollment token from
its actual owner: native helpers use OwnedWorkload; the chosen adapter Git/helper
route uses actual #60 runtime-job authority. Token construction is private to the
verified backend/producer, never a Preparation, ProcessOwnership flag or record.
Remove or gate fresh-Preparation execution variants such as NativeServer::launch
and verify_git; all ProcessGroup construction in Codex must follow enrolled spawn.
A compile-time/private API seam plus full spawn-site inventory closes structural
bypasses; targeted mutants complement that seam, not replace it.

If an effect occurred before model admission and its complete setup cleanup is
unknown, keep held/Lost with a factual startup-cleanup reason. Persist Lost only
where an actual Session exists; otherwise retain the private owner and owning
durable operation/reservation without inventing a Session or consumed/model flag.
Do not fabricate native_dispatch_unobserved for mere startup, and do not mistake
!inference_started or selected ProcessOwnership::uncertain()==false for pristine.
Failed publication remains held with private completion false. F4 first-cause,
queued stop, atomic admission and exact snapshot guards remain unchanged.

Move the same private owner reference, operation and queued stop receiver through
Supervised installation in the same critical section, with no gap where a caller
can cancel/drop the retained preparation/supervisor and discard its anchor.
Cancellation/timeout latches its first factual cause before cleanup awaits.
Panic/runtime shutdown never mint completion or erase durable uncertainty.
Native exact-turn interrupt/drain and scoped background cancellation assist only;
ACK/list/server shutdown cannot settle. The actual retained backend/service owners
stop and verify every enabled unit, and independently reap covered startup/helper/
frontend groups. Binding drift, loss/truncation or unexpected delegation stays
Unknown/Lost; preserve existing deadlines and never adopt numeric membership hints.
Known dead server PID may be cleared factually while workload/operation uncertainty
remains held. Lost remains absorbing under ordinary calls even after later factual
cleanup, pending actual separately reviewed #14 audited recovery. A human judgement,
manual label or old completion flag is no cleanup certificate or replay grant.

Every inspector is an enabled execution route. macOS selected-group inspection
currently runs trusted /bin/ps. Any future complete backend must either use a
verified bounded safe non-executing kernel query, or bind this inspector to a
separate actual owned bounded helper unit outside the cohort whose emptiness it
observes. Inspector exact executable/input authority, startup and complete own
cleanup precede acceptance of that observation; it cannot count itself in target
emptiness or be an unowned cleanup exception. Preserve existing 250 ms/1 MiB,
complete-frame, owned unreaped anchor and Unknown semantics. The existing selected
inspector control is not already a full helper/cohort backend. Unknown inspector
coverage makes the affected production cleanup profile unavailable. Other runtime
Git/helper capture routes compose pending #60, never a later native phase lease.

### Public impact and legacy limits

This correction supersedes the earlier Impact sentence about unchanged capability
claims and master section 16's implemented capability description for F1 readiness.
Those paragraphs record a tested selected-group/component baseline, not current
whole-workload eligibility. Actual current consumers are:

| Consumer | Current fact / initial correction and composed obligation |
| --- | --- |
| adapter.rs:204–217 AgentAdapter capabilities/probe/start_structured; AgentRegistry::get at 300 | Flags require backend AND the actual callable composed producer/hold consumers. Legacy route stays absent even if fixture backend is ready; dyn Unsupported is not a post-marker strategy. |
| adapter.rs:277 AgentRegistry::from_config | Current factory creates Generic only; explicit Codex registration remains separate. Do not claim config-native provider selection is already integrated. |
| workflow.rs:813 inputs; capture564; reserve949; capability1132; start1178 | Capture and reservation currently precede capability checking. Adapter gate alone cannot close that gap. Actual19/60 driver must select/profile-gate before clear_hold/capture/reserve, or independently own that executing capture and retain a typed hold; no retry/fallback. Pending integration is a deployment blocker, not silently fixed here. |
| workflow.rs:1296 transport_succeeded /evaluate | Current terminal adapter proof cannot become whole-workload release/evidence. Pending19 settlement and actual evidence consumers must close their own held authority. |
| main.rs:80 config-check; pending Issue 15 agent/attach output | Existing command validates TOML only and has no native agents readiness path. Future output must distinguish unavailable profile, metadata, selected component and genuinely supported backend. |
| session.rs:1649 capabilities/probe, start/resume/checkpoint/stop/release | Empty production backend gates before mutation/effects; read-only factual current/historical status remains available without freshness/settlement claims. |
| git.rs:98 create/283 cleanup/359 executor_reserved/370 ensure_no_executor; state/mod.rs:1300 ensure_project_idle /1375 validate_worktree_exclusion; ownership.rs:266–278 capture | Generic terminal-label gap includes pre-F1 records AND the Executor-only filter: F1-era Reviewer/Consultant Lost is ignored by cleanup/lock/Executor insertion/capture. Decision/Consult readiness requires actual durable hold consumers here plus #9/#10/#19/#58. Own-table refusal alone cannot close generic or other-provider bypasses; no legacy owner is synthesized. |

Previously running component Sessions have no newly acquired F1 owner. The corrected
adapter must not treat old private Evidence.completed/selected terminal as new
whole-workload completion, continuation or release authority. Stop can assist only
through still-actual retained handles and covered helpers. Persisted pre-F1 terminal
labels are still accepted by generic Store/Git consumers until their actual shared
hold/recovery integration; operator-attested legacy drain is an explicit deployment
limit, not kernel cleanup proof. This proposal does not claim those consumers are
already protected. F2 decision cwd and IPC02/#51 environment provenance remain open.

### Runtime capture before the native marker

Issue60 is the pending independent runtime Git/helper owner/reservation producer.
For this proposal, choose the independent #60 route for adapter Git/version/helper
execution, including in-attempt preparation, checkpoint, grant and status. Native
server/frontend/tool execution uses the attempt owner; no ambiguous optional owner
substitution remains. Each executing Git/helper route requires its own actual #60
reservation/enrollment and private known-outcome plus complete-cleanup settlement.
The native attempt tracks these exact job handles from before spawn and retains them
through cleanup. Its NoCurrentDispatch or KnownCurrentTerminal predicate explicitly
includes every associated #60 job's actual outcome and complete cleanup; a job's
Lost/unknown or failed publication prevents native settlement. Completing the native
cohort alone does not cover Git's detached fsmonitor/hooks. Independent pre-marker
capture settles under its own #60 authority before native reservation, never a later
attempt receipt. The #60 producer binds once each actual Git executable/interpreter/
PATH child and effective system/global/include/includeIf/config/hooksPath/fsmonitor/
attribute/filter inputs, then enforces that exact execution identity throughout.
No repeated unrestricted PATH resolution or ambient environment can substitute.
These are composition obligations; current pending #60 has no callable token/receipt.

Successful protected checkpoint is a separately classified input-update path:
checkpoint Git uses its own #60 owner/reservation/settlement, while atomic Session/
request replacement needs actual #19 managed current-input/operation authority.
The current CheckpointCommitted label is not native terminal outcome, refusal, or
success settlement. Until the reviewed #19 checkpoint-success publication route
and #60 cleanup conjunct compose, checkpoint is Unsupported before effects. It
cannot borrow a lease from a future resumed native turn, mint NoCurrentDispatch
for success, or invent a new #19 receipt class. This remains mandatory/pending.

All external runtime execution is covered, including adapter preflight/grant/status/
cleanup, Issues18–20 source/context capture, #19 pre-marker physical/admission hashes,
Workflow gate-claim/AllowCurrent capture, #15 status/TUI, #26 registry and #14
recovery. Context capture that precedes a native marker cannot use a later native
owner or invent an early native phase lease. The capture's own durable current
conflict reservation and actual owner precede first Git/helper execution; known
current outcome AND complete enabled runtime cleanup precede its settlement.
Source digest/ContextVersion or gate publication success is never cleanup proof.
Failure/Drop/timeout or uncertainty retains that runtime owner/hold and prevents
conflicting capture/native admission/removal. Its own reviewed source producer is
pending; no callable Issue60 port is created by this design.

The alternative is strictly non-executing bounded scoped in-process reads that
cannot invoke fsmonitor/hooks/filters/textconv/external helpers. A library label
alone does not prove that property. Capture must define the exact authority it
hashes and maintain the same content/source semantics at the native consumer.
Configured attributes/clean-smudge/LFS/eol/ident/worktree-encoding that cannot be
faithfully applied without prohibited execution make affected current facts
Unknown. Refuse/hold current source publication with a typed content-identity
reason, or use the actual owned executing route. Never publish filter-divergent
facts as current or silently strip configuration to obtain a passing hash. Raw
scoped bytes can be displayed with their explicit raw/historical identity but
cannot substitute for native-equivalent admission authority. A real capture
fixture with LFS/clean-filter paths must prove this distinction and kill a mutant
that publishes unknown/divergent facts as current.

During unresolved holds, bounded non-executing Store/filesystem observations may
report factual historical/held state without cleanup/admission/freshness claims.
Git-invoking observation needs its own actual owner/reservation plus continuously
enforced compatibility with the held workload, including the primary root.
Read-only purpose or argv is not nonconflict proof. Otherwise refuse that action
with a held/unsupported reason while safe non-executing state reporting stays
available. Existing context diff no-ext-diff/no-textconv arguments constrain those
bound routes only, not all repository-configured activation routes.

### Live effect contract and backend acceptance

Providers5/6/7/Generic and all runtime Issue60/12/13/capture/status/registry/recovery
profiles are actual both-side matrix parties, composed with #19/#58 epochs. The
actual reviewed compatibility contract covers own-worktree/branch writes, atomic
append-only shared objects or proved equivalent, shared executable/config/hook
activation targets and bound current authority for the entire lifetime. Common-dir
identity alone neither permits effects nor blanket-serializes live Tasks.
Already-running B when A becomes Lost must be protected by B's real enforcement/
settlement or proved noninterference. A new-admission-only refusal cannot stop B.
New or changed native/Git/helper/profile identity re-verifies every applicable
mixed combination against supported peers, including B-already-live→A-Lost, before
advertising readiness. Positive same-repository Executors plus open Consult and
required four-plus Task aggregate proof remain acceptance conditions, not inferred
from a matrix table. No such producer is implemented by this proposal.

Concrete configured automatic shared activation closure is bounded and pinned by
executable/config/indirection inputs/digests; unknown/cyclic/truncated/unbounded
closure makes the affected profile Unsupported. Direct hooksPath/entry targets
stay protected. Ordinary code consumed only by a later fresh genuinely governed
runtime action remains inert work product. Explicit independent user/IDE future
execution is a documented application boundary, never an F1-owned future-process
theorem. Reachable current service/deferred routes still require actual owners or
bound pre-execution denial; the application boundary cannot omit them.

Linux cgroup-v2 conformance would need private immutable handles, pre-exec exact
owned enrollment, no foreign injection/same-UID migration/control-FD escape,
bounded populated/death observation and complete current service coverage. Current
Apple launchd same-group cleanup cannot cover the demonstrated second SID; ES
candidates need independently validated platform/entitlement/safe API/death proof.
No backend is installed/configured here. If no compatible backend is verified,
retain the platform blocker and escalate only a concrete validated prerequisite/
distribution decision; no generic permission or native-default waiver is implied.

### Fixture separation and acceptance traceability

A private cfg(test)-only backend, never Cargo feature/TOML/environment/public
builder, binds exact finite synthetic executable identities and genuinely retained
fixture owners. Existing internal F4 owned-Git, supervisor sleep/bootstrap,
Unix-WebSocket CAS/ack/stop/cancel/drop/restore and checkpoint controls migrate to
it so their actual consumers stay reachable. Finite subprocess fixtures may retain
a self-expiring second-SID child's actual owner and produce Unknown; they can claim
complete cleanup only for their explicitly enrolled finite closure. "No-subprocess"
describes particular synthetic peers, not a prohibition on these required tests.
Existing F4 controls cannot certify ambient host Git. They migrate to enrolled
finite synthetic Git identities with isolated owned executable/config/input closure,
or remain Unknown and assert held outcomes. Any actual host-Git fixture needs its
own verified finite fixture closure and controlled config inputs; uncontrolled
system/global config, PATH, fsmonitor or hooks cannot inherit a certificate from
selected parent death. Fixture-specific isolation is never a production/native
mandatory-config override.

No runtime/native JSON creates fixture authority. A non-cfg(test) integration test
against the ordinary library must verify empty production route readiness/capability
and actual zero-effect launch refusal; a compile_fail doctest additionally proves
fixture constructors cannot be accessed. A unit test built with cfg(test) alone
cannot establish production emptiness. Real
installed native positives remain acceptance-blocked until genuine host/backend
conformance; they are not converted to passing synthetic substitutes.

Each acceptance-evidence item is mapped below. C means actual Issue6 caller guard
and compiled causal mutant; R means initial honest pre-effect refusal plus unchanged
registry/Store/process/frame evidence; P means pending composed producer/real
installed backend gate. C/R alone cannot satisfy P or MVP acceptance.

| Requirement evidence | Gate and actual owner/consumer |
| --- | --- |
| Installed Executor, decision structured native result, requested/default settings | C protocol/output/nullable response observations; R no backend; P both-host backend plus actual Issue 21 consuming model telemetry. |
| Exact own refreshed resume and foreign refusal | C existing F4/input/currency controls migrated to finite backend; R before transition; P native backend/Issue 19 continuation. |
| Interactive trust/permission and required Task attach | C expansion/unadmitted turn/owned frontend lifecycle; R unavailable; P Issues 6/11/15/19/14 composed both-host native attachment. |
| Callback correlation, replay and persistent-grant refusal | C actual submit/answer consumer, identity/effective-input rechecks, bounded ledger; R unsupported profile; P native backend and Issues 9/10 broker. |
| Two-Project context/env/Git/reference isolation | C actual ownership/native request consumers; R before effects; P F2, Issue 51 and Issues 19/58/60 current authority. |
| Bounded failure/cancel/stale/immutable review and source/CI checks | C held-owner release/evict/restore/Drop/publication; C-1/C/C+1 scope quota, A-full/B-admitted, premarker-claim release, complete-cleanup slot retirement with Lost hold unchanged, and global-counter mutant; exact debug/release/bothOS; P installed whole-workload and F2 proof. |
| First runtime/native version/auth/config/Git/preflight probe | C gate-before-process/registry, cache-only readiness, replaced binary/changed config with zero marker/operation/history, sealed enrollment and unenrolled-helper mutants; R zero child/frame/consumption; P identity-bound backend. |
| Detached fsmonitor/hook before native startup; pre-consumption startup uncertainty | C finish_preparation_error/context/checkpoint/discovery/transition restoration each plus removal mutant; P finite escaped-owner control and actual whole native backend. |
| Premarker/context/gate/status/registry/recovery capture, filters and held observation | C Issue 6 own preflight refusal and faithful nonexec content Unknown; P Issues 60/18–20/19/15/26/14 executing capture owner/settlement, LFS/filter/textconv and actual caller mutants. |
| Runtime immediate/deferred delegation | P Issues 12/13/60 actual enabled route ownership or effective denial before execution; real readiness/settlement consumer mutants; no future cloud ownership theorem. |
| Unsupported before marker; post-marker acquisition failure | C actual Workflow capability1132 with fixture-ready backend/no producer: no dispatch_started or native marker; backend-only capability mutant. Existing earlier context/reserve is still the documented gap. P actual19 driver gate before clear_hold/context/reserve: zero marker/operation/context/history. C synchronous handoff drop-before-first-poll/closed-runtime/marker-to-registration plus delayed-registration/claim-release mutants; P actual19 pristine/partial setup settlement. |
| Checkpoint success and Session-less setup owner | C actual checkpoint refuses unavailable managed-success/#60 route; owner-substitution mutant. C operation-keyed no-Session stop/observation/factual cleanup/publication failure, never hold release; P actual19 current-input update and60 settlement. |
| Adapter Git grant/preparation/status binding | C configured finite fsmonitor, PATH swap and exact helper/grant enrollment removal mutants; P actual60 binding/known outcome/cleanup conjunct at native settlement. |
| Parent group death plus terminal cannot settle escaped workload | C terminal/transport/restore consumer mutant with actual finite second-SID owner; P installed nested sessions/backend. |
| Reduced-profile attach/permission/broker expansion | C actual action/grant refuses before effects and removal mutant; R unavailable; P Issues 9/10/11/15 owned frontend/broker composition. |
| Executable/launcher/effective binding drift before/during work | C real launch/dispatch/grant rejects drift; R unavailable unknown identity; P backend actual executed chain/kernel enforcement, not descriptor equality alone. |
| Immediate daemon and deferred file-mediated native delegation | C default-deny/enroll consumer + removal mutant; P complete enabled native/service ownership and actual installed conformance. |
| Shared hook/config/indirection write; bounded automatic closure; later governed ordinary artifact | C launch/grant effect enforcement and actual later consumer refusal mutants; P Issue 6 backend/60/19/58 continuously enforced matrix. Explicit outside user-triggered actions remain application-boundary limits. |
| Lost Task versus common-Git mutation/base/worktree lifecycle | C actual native admission checks table extents: A Lost/B common-dir refused/disjoint Project admitted, extent-check omission killed. P Issues60/19/58/14 shared consumers/retained epochs, Reviewer/Consult role-filter bypass mutant; no Task-ID isolation. |
| Live same-repository peers, runtime↔native/runtime, B-live→A-Lost | P all enabled Issues 5/6/7/Generic/60/12/13 profile producers continuously enforce both sides and failed publication/Drop; new identity reruns applicable pairs; Issue 16 four-plus aggregate. |
| Lost approval reviewer holds original requesting operation | P Issues 9/10 actual decision-member/slot/settlement bound to original request/input/source with removal mutant; not conceptual DecisionTask or operation-free lifetime. |
| Both-host nested native/stop/drop/private settlement | P actual backend,19 owned settlement and Issue 16 aggregate; no selected singleton/metadata/Unsupported or fixture substitute. |

This table also requires inspector binding/cleanup mutants, held-table capacity
refusal, release/evict/transition anchor retention and empty production-build checks.
Backend/producer availability is a separately gated source and installed conformance
fact. No newly added profile inherits earlier other-profile matrix proof.

Requirements approval is not design approval. Design3 must pass independent immutable
native design gates before source. Exact-head fmt/clippy/default debug+release tests,
builds, source review and both-host CI are separate from genuine enabled-profile
installed conformance and Issue 16 aggregate acceptance. An initial honest no-backend safety
checkpoint cannot close Issue6 or waive either host's mandatory native capabilities.
