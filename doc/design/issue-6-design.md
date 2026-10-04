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
independent native requirements reviews with no Critical/High/Medium findings.
Design10 is a proposed STRICT design; it is not source/backend/MVP approval.
Production remains the unchanged `cab665d56f7032f086946c1acfce448f5b9d076d`
selected-group/F4 component. This section supersedes earlier full-cleanup or
capability eligibility wording, without erasing that component's evidence.

### Verified gap and required acceptance

Pinned Codex `rust-v0.160.0` at
`a956835d020762cb2b570053af06f643a11c0ecc` creates separate sessions for PTY and
shell commands. Logical termination, background inventory and server shutdown do
not export an actual retained whole-workload death authority.
Primary sources: [PTY setup](https://github.com/openai/codex/blob/a956835d020762cb2b570053af06f643a11c0ecc/codex-rs/utils/pty/src/pty.rs#L645),
[shell spawn](https://github.com/openai/codex/blob/a956835d020762cb2b570053af06f643a11c0ecc/codex-rs/core/src/spawn.rs#L101),
[termination](https://github.com/openai/codex/blob/a956835d020762cb2b570053af06f643a11c0ecc/codex-rs/utils/pty/src/process.rs),
[unified_exec](https://github.com/openai/codex/blob/a956835d020762cb2b570053af06f643a11c0ecc/codex-rs/core/src/unified_exec/process.rs#L259).
Finite owned standalone probes observed a second-SID command continue after native
termination and, separately, after stdio-server exit. It self-expired; independently
verified cleanup covered only the actual unreaped server's selected group. These
are not model/unified_exec/adapter or all-descendant acceptance tests.

Both macOS and Linux remain required MVP hosts, including the actual macOS26.6.2
target. Executor, formal Reviewer, Consult, fresh-input resume, stop, native Task
attach and four-plus compatible workload concurrency require installed native
conformance on both hosts. Unsupported-only safety is never a release waiver.
No native hooks/auth/trust/default tools/model/effort are disabled to fit a profile.
No privileged installation, host/version change or replacement model API is implied.

The macOS deny-fork singleton proved direct-Rust metadata initialize/config-read
only. The ordinary Node shim failed spawn; hooks, Review and ApplyPatch compatibility
remain unproven. Linux delegated cgroup-v2 and macOS mechanisms are research
candidates, not enabled backend permits. File-effect fencing alone is not resource
death/accounting. No current backend proves the entire required native workload.

### Stage A: actual EMPTY/no-effect capability correction

This is the only implementation scope proposed for the next source gate. The
implementation-owned production availability set starts EMPTY and has no production
ready constructor. It cannot be populated by config, environment, Store rows,
provider aliases, caller flags, telemetry, saved PIDs or a public builder.
No future ownership or settlement API is invented by this correction.

The low-level surface is part of this same boundary, not an upstream-guard premise:
seal codex::transport and the effectful protocol surface to the Codex module. Remove
public NativeServer::launch, rpc field and socket accessor; NativeRpc construction,
connect handshake, send/call/initialize/dispatch must not be externally callable.
Every remaining in-module creating/dispatch route requires the single EMPTY gate
before effects; receive/cleanup of an already-genuine retained owner is narrowly
separate. Source inventory includes lib.rs/codex/mod.rs exports and every low-level
symbol, with external ordinary-library rustc-JSON rejection controls. No #11/#15 or
conformance caller receives an ungoverned socket/client; isolated existing test
harnesses migrate to the private cfg(test) component seam.

Every own entry to native launch/dispatch, including public/inherent/trait
start/start_structured, resume, checkpoint, attach and permission submission,
checks this unavailable boundary before registry insertion/terminal eviction/
control replacement, a first await, Session/Starting/history write, consumed input,
Git/helper/native process, connection, discovery, model frame or broker grant.
Only bounded strictly in-process input validation may precede it. Absent the
backend/actual callable producer, return typed Unsupported with a fixed reason.
Repeated calls do not change a previous Session, input, private control or hold,
and cannot retry/fallback into an executing route.

Synchronous capabilities() is exactly an empty BTreeSet, including no telemetry,
structured, permission, resume or checkpoint flags. Probe returns
Err(UnsupportedCapability) with the fixed reason "native workload ownership and
dispatch producer unavailable"; it returns no AgentInfo/model/effort support flags,
and performs no version/auth/config/Git execution or cached-metadata admission.
Native discovery,
policy=None and executing native --version cannot bootstrap eligibility. Pure
construction and descriptor reads do not start jobs or inspect a repository.
Capabilities require both an actual ready backend AND the actual callable owning
driver/producer, including source/currency/hold consumers. A fixture-ready backend
alone cannot enable the legacy callable route. A future managed-only producer
cannot enable flags on public legacy start that still refuses.

Own status/subscribe/usage/pending_approvals retain current private owner checks:
unowned IDs return SessionLost, never persisted Starting/Running as live status.
Historical reporting remains on the unchanged in-process Store path; no new history
API is supplied. Actual retained owners may expose factual private observations,
without helpers or fresh-completion claims. transport_succeeded is always false
under EMPTY, including test-simulated component completion; a historical selected-
group journal cannot become whole-workload success. Missing full owner/settlement
returns unavailable,
never an observed server exit, native terminal, stop acknowledgement or public
cleanup flag as a completion certificate.

The correction must not abandon already-actual retained owners. Existing internal
component stop/cleanup may assist only its genuine unreaped selected groups and
retain first cause, uncertainty and actual handles. It cannot publish complete
workload settlement, evict/release/reuse an unsettled control or erase a hold from
selected cleanup. Public release refuses unsettled entries BEFORE claim()/transition
mutation; eviction never removes them. Public submit_approval refuses even Deny/Cancel
under EMPTY; internal
already-owned stop/decline/interrupt cleanup may only reduce the prior workload,
never ALLOW/new input or publish full settlement. Public stop is explicitly this
reducing path for an already-genuine captured attempt; it may cancel/interrupt/
drain, and Ok(status) is not settlement/transport success. It never looks up a
later attempt or creates a new owner. For unowned IDs it returns SessionLost with
zero effects. The zero-frame oracle excludes ONLY this genuine reducing path,
not public approval submission or any new input/ALLOW. No saved PID/Session row
reconstructs that owner. In a restarted runtime absent genuine ownership, only
historical Store reporting is available; adapter status is SessionLost and
mutations refuse.
The later actual #14 recovery producer, not a manual label, releases uncertainty.

Stage A can progress independently of missing backend/19/58/60 source because it
does not call those producers or acquire an operation/resource. This is an adapter
component boundary, not a claim that current Workflow's earlier capture/reserve is
already protected. Workflow must eventually select actual readiness BEFORE
clear_hold/capture/reserve/marker. Each executing capture needs its OWN actual60
owner and complete settlement; later native refusal never retroactively covers it.
Current factory/config still selects Generic; no config-native registration or
whole-runtime safety is supplied by the Codex boundary alone. At the frozen current
source, Workflow inputs/load_rules/resolve_file/git_text can execute before the
later descriptor check; clear_hold, reserve and failure/WaitingHuman publication
can also occur despite a late Codex refusal. These remain actual shared-driver
changes, not fixed or retroactively owned by Stage A. A pending #43 preflight is
not composed #19/#60 readiness or evidence of safe earlier capture.

### Source A controls and impact

| Actual current consumer | Stage A correction / retained open dependency |
| --- | --- |
| adapter.rs:204–217 trait capabilities/probe/start_structured; AgentRegistry get300/from_config277 | Check own callable Codex route without changing Generic's API or claiming registry integration. EMPTY cannot advertise operational flags. |
| session.rs:1649 capabilities/probe; start/resume/checkpoint/attach and aliases | Earliest no-effect gate; no registry/history/input/control mutation, native/Git/helper process or frame. |
| mod.rs exports; transport.rs:135 launch /152 launch_preparing, rpc80/socket272; protocol.rs:182 connect,202 send,286 call,354 initialize | Seal exported effectful routes and gate each remaining own creation/dispatch. Ordinary-library rejection and actual zero-effect oracle cover all symbols, not only adapter aliases. |
| session.rs:1396 native --version; ownership.rs:369 Git/resolve, filesystem helper | No Git/metadata/version/discovery before gate; per-fixture absolute sentinel and private site oracle expose each actual call. |
| answer_approval1999–2087 / frontend input | No grant/dispatch expansion without actual ready backend plus original owning input/operation producer. |
| terminal546 /2346 and transport_succeeded1679; release1819/evict110 | Selected component evidence cannot mint full-cleanup or release authority. Already-consumed unknown remains Lost/no replay. |
| Reservation::Drop626; TaskGuard attempt.rs:244–274; release_task190; RegisteredTransition::Drop170 | Preserve actual retained owner, first cause and unpublished/Unknown state. No new complete-cleanup or producer proof from Drop. Full later nonexecuting Drop producer remains open below. |
| workflow.rs capture564/reserve949/capability1132/start1178/poll transport1296; validate_persisted_status1308; resume_gate1598/transport1621; evaluate1672 | Earlier executing capture and reservation remain actual19/60 driver integration blockers, not fixed by late Unsupported. |
| git.rs cleanup283/executor_reserved359/ensure_no_executor370; state ensure_project_idle1300/worktree exclusion1375 | Current terminal-label/Executor-only exclusions are not genuine retained ownership. Shared19/58/60/14 hold consumers remain pending. |

The production representation needs no inhabited ready token, variant or constructor:
EMPTY is an in-process fixed refusal. No broad dead_code allowance or non-test
factory exists. The ONE private cfg(test) seam supplies backend/owning-producer
inputs to the SAME gate called by unmodified public entries; neither alone passes.
The refusal path and EVERY public/inherent/trait/dynamic/low-level entry/alias body
are cfg-unconditional. The sole cfg(test) acceptance branch is inside that gate;
other cfg(test) instrumentation may observe, never replace routing/refusal bodies.
Inventory cfg(not(test)), cfg_attr and feature/target conditional aliases too;
no ordinary-only success/fallback or cfg-selected entry implementation exists.
Both test inputs permit component regressions only, never production/native/full-
cleanup readiness. Ordinary construction defaults to EMPTY in test builds too.
All test fields/factories/site oracles are absent from ordinary library builds;
Pinned rustc JSON controls classify EACH symbol kind, not any-error acceptance:
associated methods missing E0599 versus present-private E0624; fields missing
E0609 versus private E0616; unresolved free items/types/module/import paths use
their actual E0425/E0412/E0433/E0432 versus private-path E0603. Struct-literal fields
use absent E0560 versus private E0451; tests must
separately prove field absence, not merely a privacy/build error. Run exact
positive/private comparison snippets on Rust1.91.1 and pin emitted codes at source
gate; unrelated codes fail. Production sealed exports are privacy controls (E0603),
not test-item absence proof. Keep cfg(test)/export inventory; compile_fail labels
alone never suffice.

Actual callable controls use isolated Store/repositories and per-fixture oracles:
an absolute sentinel Codex executable records any invocation; private cfg(test)
per-adapter site counters record entry to Git resolution/preflight, filesystem/
executable metadata, version, transport creation/connection and frame/grant sites.
A scoped absolute Git sentinel at the resolve seam detects actual invocation.
No in-process PATH/HOME/TMPDIR mutation or global process enumeration is an oracle.
Snapshots cover Session/history/consumption/marker/input, registry/control identity,
retained entries, watch/channel state and sentinel/site counters. Constructor,
exact descriptors/probe, observers and all inherent/dynamic/low-level aliases
must preserve zero children/connections/frames and identical snapshots under EMPTY.

Route-guard seeding uses a protocol-speaking fake app-server through the REAL
public start route with only that single test seam enabled. It produces the actual
registry/control/Starting/admission/current native terminal (or pending request)
under synthetic fixture ownership. No direct registry insert or prefinished
manufactured Entry is allowed in these route controls. Existing terminal_adapter,
supervisor_fixture and direct Reservation fixtures remain private-function
component tests ONLY, not real-route guard evidence. Resume/checkpoint controls
stage higher-version input after this genuine terminal, then disable seam inputs
before the public call; approval controls disable them after a real synthetic
owned live pending request. No owner is manufactured from a row/JSON/PID. A removed
guard must reach actual
Starting/Git/native/channel/frame effects observed by these oracles, rather than
merely change Unsupported to SessionLost. Fixture-owned resources remain tracked
and explicitly cleaned even when a mutant reaches effects.

Operators remove each actual route/conjunction guard, expose a low-level constructor,
insert Git preflight or executable metadata/--version before the gate, restore any
single descriptor flag, promote selected completion to transport success, or allow
release/eviction of unsettled entries. Each compiled mutant must be assertion-killed
by the effect/snapshot/descriptor/ordinary-API oracle it actually targets; no error-
label-only effect credit. Already inert attach and trait-default native-goal paths
have no downstream executing route: guard-removal mutants are equivalent/redundant
survivors, honestly recorded without effect-kill credit. Their typed refusal and
zero-effect/descriptor controls remain required. Restored controls and owned
fixture cleanup are required.
Existing component positive cancellation/CAS/framing/checkpoint tests migrate to
this single seam and remain component evidence. Their public transport success is
false and unsettled release/eviction refuses; private component journal checks may
still verify old limited observations without a new receipt. Test-scoped retention
has a fixed bound: saturation refuses BEFORE insertion and never evicts unsettled
owners. Fixture teardown is cleanup of its actual synthetic handles, not settlement.

At the frozen source, actual external consumers include
examples/native_codex_protocol.rs (public NativeServer::launch/rpc) and
examples/native_codex_session.rs (CodexAdapter). Source A makes both ordinary-build
examples refuse Unsupported BEFORE fixture/files/Store/Git/executable metadata or
native work, with no opt-in bypass. Historical native proof artifacts remain
preserved; private component harnesses are not installed native conformance.
Actual direct constructor/connect consumers otherwise occur in own session/
transport/protocol test modules. Workflow/state have no direct Codex import at
this head, but their dynamic AgentAdapter expectations remain impact inventory.
Source impact still inventories every reference and external integration caller;
migrate component assumptions explicitly rather than deleting regressions.
The SAME Source A PR updates master agent-adapter §§15/16/17 (including caller-schema
Codex structured/transport claims), every grep hit naming Codex support, README and
this design's status to EMPTY/Unsupported facts and retained test-only component
evidence. No current positive executable capability or whole-cleanup support is
advertised. Dynamic Workflow poll/validate_persisted_status/resume_gate controls
cover false transport completion and unowned SessionLost without changing shared
production consumers or claiming current #19/#60 composition.

A REQUIRED executed integration target links the ORDINARY non-cfg(test) library,
in debug AND release, with no special feature or test authority. It calls every
CodexAdapter public/inherent/trait and Arc<dyn AgentAdapter> alias, constructor/
broker descriptor, exact empty capabilities/probe, start/structured/resume/
checkpoint/attach/approval/release and unknown-ID observers/stop. Assertions match
each class: dispatch/refused release has the fixed Unsupported reason; unowned
observers/stop SessionLost; transport_succeeded false; native-goal defaults keep
exact existing typed Unsupported and are inert. No merely compiled example/test
or cfg(test) result substitutes for execution of this shipped path.

Its subprocess harness receives explicit scoped PATH/HOME/Git sentinel environment
at spawn, never mutating parent process environment. Absolute Codex/Git sentinels
and ordinary public Store snapshots detect actual invocation/Starting/history/
input effects; observable adapter outcomes check no owner/completion promotion.
No cfg(test) counter/factory is an oracle there. The harness retains and cleans
its actual synthetic child/group ownership even for effect-reaching mutants; it
uses no real native authentication/model/foreign repository. Kill a compiled
ORDINARY-library gate success/fallthrough mutant with these runtime effect/
snapshot assertions, not only a changed error label. Also inspect the ordinary
export/symbol/cfg inventory. Both ordinary execution and private component unit
controls are required; unit tests, cargo-build or compile-time absence alone
cannot establish the shipped no-effect claim.

Exact reviewed source, default debug/release suites, fmt/clippy -D warnings, builds
and both-OS CI are required. CI trigger head and actual checkout merge SHA/parents
plus reviewed blob equality are separate facts. Historical cab macOS inspection
timeout and later main21 Grok reconciliation red stay visible, with no deadline,
default-concurrency or same-head-rerun waiver. Stage A cannot close Issue6 or MVP.

### Stage B: required ready-backend source/conformance gates — OPEN

Design10 does not approve an implementation of ready enrollment/containment or
settlement. A backend needs its own concrete requirements/design/source review and
installed conformance before an availability entry can exist. Proposed private roles
below describe prerequisites, not callable ports. Missing producers stay Unsupported
before effects; already-admitted/consumed unknown stays held/Lost, without outcome,
receipt, continuation or replay. Native support remains mandatory/open, not optional.

**Readiness and effective inputs.** A ready manifest must be implementation-owned,
immutable in the reviewed binary, initially EMPTY. A new entry requires a reviewed
build and actual host proof, binding OS/kernel/native chain, role/class, exact
executable/shim/interpreter/platform child, complete default effective controls and
bounded input/delegation closure. Per-attempt bounded nonexecuting metadata rechecks
identity/current source before reservation and at actual creating effect. Metadata
is not exec enforcement; continuous identity/effect enforcement must cover changes
after check. Missing/unbounded closure is unavailable, never executing discovery.

Preserve native defaults. rrx config reconstruction can identify candidates/refusal,
never narrow the startup extent. A smaller extent needs independently effective
native/kernel default-deny or a verified first-party nonexecuting authority; no such
general resolver is available here. Custom provider env_key/headers stay unavailable
without complete bound nonexecuting authority. Complete actual final environment
must be scoped/provenance-bound (#51), not expanded by executing discovery. Distinguish
code/config/route selectors, opaque non-selecting credentials, and ephemeral endpoint
predicates; credential values/digests never enter public artifacts/audit/telemetry.

Native-owned mutable credential/state locations are a separate bound class: exact
nonsecret location/type/owner/mode/allowed consumers and real enforced write/compatibility
extent, without reading credential contents. Normal native refresh writes are not
immutable-size/mtime drift. Pinned native [refresh persistence](https://github.com/openai/codex/blob/a956835d020762cb2b570053af06f643a11c0ecc/codex-rs/login/src/auth/manager.rs#L3092)
and [file storage](https://github.com/openai/codex/blob/a956835d020762cb2b570053af06f643a11c0ecc/codex-rs/login/src/auth/storage.rs#L206)
rewrite auth state; no live credential files were read to establish this source fact.
Nonsecret authority or widest genuine control must cover auth-route selection.
No token extraction, forced login/refresh or immutable credential-file digest.
Actual native refresh/default-auth conformance remains open, not a metadata-only proof.

**Actual ownership and phase inputs.** One move-only actual workload anchor is a
composite owning every enabled setup/native/tool/frontend/helper/resource and tracked
delegated service, including escaped sessions. No telemetry PID/group/row adoption.
Unknown acquisition retains exact genuine partial resources; Err/Drop is not absence.
Managed Task/formal Reviewer needs actual19 Fresh/Continue marker+operation and NEW
Starting/input admission. Prior closed receipt is provenance only; consumed/admitted
zero-wire remains held14. Protected standalone checkpoint/resume stays Unsupported;
pure staging cannot mutate prior pins or mint checkpoint success. Non-Task Consult
needs actual58. Approval action decisions need actual9/10 retained member/slot bound
to original request/input/source and its own native lifetime. Existing formal phases
are RequirementsReview/DesignReview/ImplementationReview/SecurityReview; conceptual
DecisionReview/DecisionTask is not an enum, lease or producer.

Native Task attach remains actual6 frontend /11 driver /15 caller /19 exact input and
phase ownership /14 recovery; no conversion through58 Consult. A frontend is a
pre-enrolled subset of its seed's composite workload, not a new owner/input ledger or
permission source. Actual widest profile cannot expand through attach or ALLOW.
Executor/Reviewer/Consult/structured/resume/permission capabilities require the exact
callable producer conjunction, not another role's readiness.

**Handoff and complete closed-set work.** Before marker, pure current readiness and
independent60 capture settlement precede native claim; reservation then actual19
marker+operation is atomic. MarkerPending remains retained through ambiguous commit.
The private entry synchronously makes actual operation/control resident before any
worker can run. It is I/O-free; drop registry/coordinator/control/Store guards before
submission. Enroll each job and reserve its complete creating/draining/cleanup bundle
before submission; open its start gate only after genuine JoinHandle transfer. Closed
submission/unopened gate failure uses CANCEL plus actual join, not a worker waiting
forever for GO. Live refusal routes the resident still-pristine handle to a pre-enrolled
owned refusal worker (or returns it synchronously to the driver's still-owned handoff
task) after guards release; only actual19 refusal publication frees the claim. Missing/
closed runtime retains it. No 'some worker may later exist' settlement premise.

ALL nested async/blocking tasks/readers/drains/cancellation observers must belong to
an actual complete structured registry. Close ALL registrations/admissions/queued
grants for the generation first, join the complete captured work-set, then check closed
membership under the critical section. No top-level/supervisor exception or self-join
is accepted. An independently retained passive runtime custodian observes genuine
JoinHandles including the top supervisor; the joined job does not issue its own
witness. Custodian/observation/publication/reclaim authority needs actual reviewed
19/58/runtime composition and closure over every capability holder. It cannot execute
native/Git work or mint a second model-input ledger. Before that producer exists,
there is no complete joined witness or settled receipt. Cancellation/abort/shutdown
timeout/controller exit is not join. Started/unjoinable work retains actual handles.

The same retained custodian must observe completed owners, retry exact CAS-lost
retirement via level-triggered flags outside locks, and bound/reconcile completed
job cells. A missing reclaimer remains a held capacity fact, not a Drop release.
No I/O, SQLite/audit publication, worker waits or external callbacks under coordinator;
registry→control→Store remains allowed only without it. Continue generation recheck
inside Store uses a sealed lock-free private cell, bumped before registry eviction,
not a reverse registry lock. Actual effects_started publication precedes acquisition/
Session/startup. Its false→true transition is not an in-memory bit.

**Measured bounds and independent progress.** Earlier unmeasured 45/2284/73152
arithmetic is withdrawn; it is not available capacity. A concrete backend's reviewed
bundle inventory must count EACH process anchor, reader, async job, blocking worker,
observer, custodian, inspector and reclaimed/residual owner. Every creating process
has its OWN complete inspector/cleanup execution bundle, including creator/discovery/
server when they are processes; no silent two-inspector/four-process assumption.
Its artifact binds checked finite heap/thread/process/job ceilings and measured low
startup/idle overhead. Real execution capacity must be reserved before effects,
separate from logical cells. Stalled metadata cannot occupy promised cleanup/inspection
workers or unrelated Projects' shared blocking capacity. Dedicated bounded executor/
reserved-worker design and all affected Generic/Grok/context/Workflow consumers require
their own source gate; no thread creation after child creation is assumed infallible.

Project activation reads exact registered max_tasks in-process, never executing
effective_config/validate/resolve_file/load_rules/git_text. A concrete profile defines
its supported finite bounds; out-of-range is typed unavailable before claim, never
clamped. It accounts configured quorum including three-plus reviewer sets, four-plus
Tasks and open Consult under actual scheduler defaults; default12 does not prove
four Executors+eight waiting review members+Consult13 fit. Actual16 aggregate remains
open. All current plus residual epochs charge the SAME Project's retained budget;
reactivation cannot reset it. Actual residual anchors remain reachable in their same
slots, not adopted or converted into active-partition-wide unused reservations.

Premarker requests need explicit finite bundle sizes, single-flight per affected
Project/root/surface and bounded retained stalled-worker charge. Healthy disjoint B
has reserved minimum admission/cleanup execution capacity when A stalls/saturates;
no new A retries can fill a runtime-global pool or inherit B's share. Same physical
filesystem/shared surface bounds compose with genuine continuously enforced compatibility;
unknown surface affects its actual conflicting closure, not all physical overlap.
Define bounded active/residual/global exhaustion and activation/reactivation CAS before
enabling a profile. Tests need A residual-saturated/stalled, B disjoint still admitted,
each actual class C-1/C/C+1 and compiled per-Project-charge/shared-pool/reclaim mutants.
No current quota/low-idle/native profile acceptance is supplied by these obligations.

**Spawn, Drop and transitive inventory.** No reachable helper bypasses actual enrolled
ownership. Standing guards cover std/tokio process creation/exec/fork/PTY; tokio/Runtime/
Handle/task Builder/JoinSet/LocalSet spawn and blocking/spawn_local; std thread Builder/
scope; EVERY shared/transitive crate reference and implicit destructor. Clippy config
uses a crate-root allow with inner Codex deny and reviewed narrow sealed-path allowances,
or equivalent source guard, so unrelated modules do not acquire accidental allowances.
Shared source impact is explicit. External-crate closure is reviewed classification
pinned to Cargo.lock versions/checksums/features and exact source; relevant dependency
changes invalidate it until re-reviewed, not stable call-graph introspection.

Raw ProcessGroup::Drop presently reaches kill_group→macOS EPERM→executing inspector.
A by-value wrapper cannot suppress that implicit raw Drop. Future ready source needs
type-level nonexecuting destructor/enrollment authority and independently owned explicit
inspection, preserving250ms/1MiB/full-frame/Unknown/unreaped guards. Proposed signal-only
Drop requires the still-actual unreaped anchor, sticky uncertainty and retained observers;
no inspection/thread/SQLite/allocating cleanup or death certificate in Drop. Shared
Generic/Grok/Codex source+consumer gates remain open; Stage A does not change adapter.rs
or claim those fixes. Actual macOS EPERM Drop and transitive spawn/dependency-version
mutants must expose untracked-helper bypasses before any ready profile.

**Exclusion, legacy and resource settlement.** Actual19/58/60 shared hold/common-Git
consumers precede readiness. Physical overlap is not an exclusion edge or a global HOME
quota pool. Continuously enforced provider-neutral compatibility must protect already-live
B when A becomes Lost across5/6/7/Generic/native↔runtime/Evidence pairs. Shared config,
hooksPath/direct automatic activation targets need bounded resolved closure/digests;
unknown automatic closure refuses affected profiles. Ordinary code consumed only by a
later fresh governed owned operation remains inert. Explicit independent user/IDE future
actions remain application-boundary limits, not F1-owned future processes.

EVERY pre-F1 Session with startup/dispatch possibility and no genuine whole-workload
proof is refusal-only for overlapping effects, INCLUDING Exited/Stopped/Failed labels
and ambiguous ownerless nonterminal rows. A mechanical epoch/readiness floor cannot
import a legacy label/private component bit as new cleanup. Missing legacy extent
refuses affected scope; no synthesized owner, PID killing or terminal adoption. Operator
statements are unsupported-deployment notes ONLY, never drain/release certificates.
Actual14 owned recovery/fencing is required; current generic terminal/Executor-only
consumers do not already enforce this. Refusal-only durable holds consume no active
unused owner partition; genuine residual resources remain charged to their own domain.

Complete settlement requires exact current native outcome AND complete enabled setup/
tool/frontend/delegated/job cleanup under actual current scope/input/operation authority,
then durable private publication. Native ACK/list/exit/selected group death, complete
joined work alone, Err/Drop, failed publication and old receipt never mint it. Lost remains
absorbing; factual later cleanup can retire actual anchors only after genuine cleanup
publication/reclaim, never release the durable conflict/replay hold. Authorized14 fencing
must acquire genuine former-instance exclusion/resource transfer before its effect and
prove physical fencing before adoption; no record-only epoch or helper bypass.

### Acceptance ledger

Stage A's review/tests prove only the own no-effect refusal boundary. Ready-backend
identity/enrollment/resource bounds, complete all-job custodian/settlement, actual19/58/60/
14/9/10 producers, F2 decision cwd, IPC02/#51 provenance, native broker, native Task attach,
both-host execution and Issue16 mixed four-plus acceptance remain OPEN. The
required producer/control inventory also includes #5/#7/Generic mixed profiles,
#11/#15 attach driver/caller, #12/#13 runtime jobs/evidence and later-consumer
publication guards, #23 managed-completion consumer, delegated-service and shared
hook/config activation/write-refusal fixtures. No such producer is supplied here.
Adding any ready
profile requires new immutable requirements/design/source review, actual consumer controls
and mutants, exact both-OS gates and installed positive native conformance. No dependency
is made optional and no production availability can be minted from this Design10 document.
