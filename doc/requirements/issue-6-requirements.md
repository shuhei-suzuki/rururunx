# Issue 6: Native Codex adapter requirements

Workflow: STRICT (native authorization, process ownership and shared adapter APIs).
Dependency: Issue 4 is merged at `52e7bbb`; persistence, Git ownership and registry
foundations are available. Issue 19's actual private native operation/settlement
port is a managed-operation acceptance dependency; its composition is pending.
Issue 14 owns subsequent audited recovery from held uncertainty. Issue 16 consumes
the completed adapter in aggregate native dogfooding, rather than being a
prerequisite for publishing an adapter implementation checkpoint.
Source: [Issue 6](https://github.com/shuhei-suzuki/rururunx/issues/6),
product requirements and master Agent Adapter design.

## Behavior

1. Execute prepared Task context through the installed native Codex CLI in the
   exact owning worktree. Consultation can use the exact registered source without
   requiring an Issue or writable worker on the protected base branch.
2. Support noninteractive factual Review Bundles and approval review. These roles
   cannot perform the operation they review. Native filesystem/network limits and
   external tool restrictions are enforced before sending a model turn.
3. Preserve native authentication, account/provider selection, hooks, mandatory
   rules, project trust and permission policy. Do not extract OAuth tokens, replace
   native inference with direct API requests, ignore user configuration, or supply
   bypass flags. Model and effort are explicit only when configured; otherwise
   native defaults remain authoritative.
4. Use native session UUIDs with durable rrx SessionIds. Exact owned session resume
   and interactive/native TUI connection are supported only when their native
   prerequisites are established. No last-session inference, foreign UUID adoption,
   history injection or duplicate independent execution conversation.
5. Surface native permission and human-input requests where the native route
   exposes them. Preserve native automatic approval review by default. An explicit
   runtime-broker route may use the native client callback route when permitted;
   it cannot override a native denial or weaken managed requirements.
6. Fail explicitly for missing executable/auth, incompatible native policy or
   protocol, malformed/oversized messages, timeout, uncertain process termination,
   stale prepared input, immutable locks or ownership conflicts.
7. Keep subprocess I/O, output retention, IPC queues and native request maps
   bounded. Supervision is event-driven. Confirm the actual owned native workload
   cleanup before marking a runtime Session stopped/exited; one selected parent
   process group is insufficient when native commands can create other groups or
   sessions. Uncertain termination stays reserved.
8. Record observed native token/cache metrics with exact attribution. Missing
   cost/telemetry stays null with a reason; deduplicate cumulative turn updates.
9. Atomically fence owning parent versions, scoped lock versions and Session CAS
   with durable metadata-only input consumption before inference. After possible
   dispatch, absent exact authoritative native terminal evidence stays Lost even
   after verified process death or an interrupt acknowledgement. Never implicitly
   replay the consumed input; fresh continuation metadata belongs to a new
   higher-version Starting attempt and is immutable during that attempt.
10. Register one private attempt control before any fresh start, resume or
    checkpoint await/publication. The owned task outlives caller cancellation and
    completes bounded cleanup plus exact terminal restoration/publication. Stop
    targets its captured attempt, joins its level-triggered factual outcome and
    never cancels a later attempt. Consumed-input CAS and checkpoint replacement
    share their admission mutex with cancellation; failed CAS preserves the actual
    first cause. Queued consumed-turn stop survives acknowledgement and transfer
    to the sole supervisor. Unpublished or uncertain completion remains explicit.
11. Acquire a genuine private attempt-bound native workload owner before the
    first execution of any native process, including launcher/version/auth/policy
    probes, configuration discovery and server startup. Pre-owner checks are
    limited to non-executing filesystem/registry metadata validation.
    Its supported profile must cover all processes that profile can start: setup,
    native helpers, preserved hooks/MCP, tool commands and frontend children where
    enabled. Unsupported ownership fails explicitly before that first native
    execution and before model-input admission/consumption; no app-server probe
    may bootstrap the proof it already needs. Native process IDs, parent/group/session hints,
    termination acknowledgements and background-terminal inventories do not
    create ownership or whole-workload cleanup authority. After possible dispatch,
    absent cleanup authority remains Lost with private completion false even if
    the native turn and every known parent group have ended. Native startup without
    input consumption also remains Lost/reserved if its cleanup is unknown;
    absence of model dispatch never authorizes rollback over live/unknown setup.
    Clearing a proven dead parent PID does not release the uncertain workload.

## Boundaries

The Rust runtime implements protocol/session supervision rather than coding-agent
reasoning. The native CLI owns inference, authentication, tool behavior and native
conversation history. Prepared context selection is owned by Issues 18–20; review
aggregation and approval policy remain Issues 9–10. CLI factory/TUI orchestration
is integrated later. Native Goal support is optional and does not block this issue.

Each session has one Project/Goal/Task scope and private owned process/IPC state.
No shared app-server daemon, global process kills, foreign browser/account changes,
or native trust/configuration writes are used to make a test pass. Tests operate on
isolated temporary Git repositories and supplied synthetic review text.

## F1 workload ownership and platform acceptance (requirements correction)

This STRICT correction closes an identified guarantee gap; its design and source
gates remain pending. It does not waive the native Codex MVP requirement or claim
that the current adapter has enforced the new workload boundary.

[README](../../README.md) names macOS and Linux as MVP hosts. The product's
native adapter MUST set includes Codex, and Issue 16 still requires real native
dogfooding. Issue 6 capability completion requires installed real-Codex ownership
and execution proof on BOTH macOS and Linux, not just whichever host already has
a backend. For this Goal the existing macOS26.6.2 host is an actual required
conformance target. The Linux proof must name its actual OS/kernel/native version
and backend prerequisites; moving CI runner labels are build/test evidence, not
native acceptance. Minimum deployment versions/prerequisites must be explicitly
documented before a supported-host claim, with no inferred macOS27-only waiver.
A Linux-only candidate or an explicit unsupported result is not macOS acceptance.
Local-first operation and unchanged native
auth/defaults/hooks/trust remain required; no privileged system installation,
account/config mutation or substitute model API is implied by this correction.

Workload readiness is profile-specific and must be established at the real
launch/operation consumer. ANY role with a reachable command/helper/hook/MCP or
frontend path that can create arbitrary processes requires ownership of that
entire enabled path, including arbitrary child group/session creation. A native
read-only filesystem profile does not imply no-command execution.
Decision Review, ApprovalReviewer and no-command Consult can use a smaller
profile only when its enforceable no-command inventory and its actual setup,
hook, MCP and helper lifecycle are independently covered. Profiles reduce enabled
capability classes, never cleanup strength for an enabled class. A zero model-tool
count cannot prove startup cleanup. Mandatory hooks and rules stay preserved;
their absence or supported lifecycle cannot be assumed or achieved by disabling
them. Every supported profile must bind its actual executed binary/launcher to an
installed-conformance-verified identity set, not merely a minimum version. Every
control relied upon for coverage or delegation blocking, including full Executor
profiles, has the same exact identity/input and lifetime binding requirements.
Unverified or changed identities refuse before reservation when detectable by
non-executing metadata; later detection requires actual private post-marker
cleanup/settlement and never a retroactive readiness claim.
Coverage/control evidence must come from enforceable native/kernel controls
or a side-effect-free authoritative source bound to the exact effective inputs
the native process consumes (user/project/managed/trust/profile/plugin/environment
layers), plus the resolved executable and complete launcher/shim path chain,
trusted package version metadata and content digests. Version metadata cannot
require an unowned executable probe. Bind the actual executed entrypoint to this
identity, not a later PATH resolution. rrx's own reconstruction of native
configuration cannot mint a smaller owner. Changed, unresolved or unbound inputs
are unknown. If reduced startup coverage cannot be established beforehand,
require full workload authority or
fail unsupported before the first native execution. A later native inventory
mismatch is a failure under the cleanup rules below, never a retroactive owner.
Before the first model turn or tool grant, authoritative running-native evidence
or already effective kernel enforcement must confirm each relied-upon capability
restriction. Startup/hooks/MCP/frontend controls must be effective before the
first path they constrain executes; later native self-report cannot cover that
earlier window. If confirmation/binding is unavailable, treat the path as reachable
and require full applicable ownership or remain unsupported.
Keep the exact input/executable binding throughout the attempt. Detected changes
are unknown and invoke the cleanup rules; they never widen the profile. If this
binding cannot be maintained, the relying profile is unsupported. Kernel controls
that refuse a required native hook/helper do not establish native compatibility;
metadata-only success does not prove Review or Executor compatibility.

Fix the attempt's profile at owner acquisition to the widest capability reachable
through its entire lifecycle. Attach, frontend launch, native permission changes
and broker replies cannot widen that profile. Refuse any expansion before its
first side effect or grant; a wider profile needs a separate fresh higher-version
attempt after the prior workload's cleanup is proven. A no-command owner cannot
authorize file-only Executor/ApplyPatch solely by its role name: actual native
operation and setup compatibility require their own bounded inventory and proof.

The workload boundary also includes persistent execution delegated to an existing
process outside descendant enrollment, such as a container engine, service manager,
tmux server or remote command endpoint. A descendant-only cgroup, singleton or
event stream cannot certify those jobs cleaned. Each reachable delegation path
must either be covered by genuine current ownership/cleanup authority, or blocked
by enforceable profile controls before the first native execution; otherwise the
profile is unsupported. This applies to preserved startup/hooks/MCP and frontend
paths as well as model tools. Ordinary bounded native inference/auth requests are
not claimed as remotely owned process cohorts, and remain native responsibilities;
they cannot become an untracked persistent Task-execution channel. No narrower
delegated-job exclusion or whole-workload claim is authorized by this correction.
No global service kill, broad process adoption or host-policy mutation is implied.
Publish each profile's delegation inventory and its completeness basis, bound to
the conformance-verified native identity/effective inputs. Reachability is default
deny under the actual enforced controls; any unclassified route is unsupported.
List justified bounded request endpoints separately from persistent execution
routes, including deferred file-mediated schedulers and autostart entries. A named
daemon denylist alone is not a completeness proof.

Before any native process may have executed, registered Preparing cleanup and
exact historical rollback/factual failure remain authoritative. Once any native
process may have executed, rollback, terminal restoration and reservation release
require current whole-workload cleanup authority, even before input/operation
admission. Unsupported/incompatible discovery, timeout, stop or caller drop do
not exempt this window. Unknown cleanup remains Lost/reserved with private
completion false and a startup-cleanup reason; it does not fabricate a consumed
input or native turn. After possible input/operation admission, unknown cleanup
likewise remains Lost/reserved and cannot certify transport success, settle an
operation, permit replay or release its reservation. The actual
private Issue 19 operation/settlement port must compose current native outcome
with current workload cleanup; neither a source-version record nor JSON fields
mint that authority. No model input is consumed merely to discover ownership.
Until that actual Issue 19 port is composed, managed native operations cannot be
admitted or settled by an adapter-local JSON/state substitute. Unsupported/synthetic
controls are allowed development evidence, not a managed-operation grant.

Managed ordering distinguishes profile readiness, operation reservation and model
input consumption. Implementation-owned unsupported profiles refuse before
clear_hold/reserve/context/dispatch_started and never fall back to legacy start.
For a supported profile, the successful exact reservation precedes Issue19's
atomic dispatch_started plus operation-lease transaction. That marker is not model
input consumption or cleanup proof. The managed entry point synchronously moves
the private operation handle into its registered owned attempt before any await
or caller-cancellable point. The supervisor acquires the attempt's genuine workload
owner before any native execution,
and retains both through startup, admission and settlement. Runtime owner-acquisition
failure after the marker sends no native process or input, but cannot erase the
operation by adapter rollback. A post-marker failure before possible model-input
admission/consumption closes only through the
actual private port's NoCurrentDispatch receipt: tracked pristine no-execution
authority, or tracked setup cleanup with no current admission/consumption/uncertainty.
Pristine no-effect authority is consumed before the first Session publication,
external Git/process/native connection or owned resource effect; subsequent failure
requires that effect's actual tracked cleanup. A durable effects_started flag alone
is not proof of never executing.
After possible admission/consumption, only the applicable KnownCurrentTerminal
non-success receipt can close a known failure/interruption, with exact authoritative
current native outcome and current whole-workload cleanup. Missing either remains
Lost/held. Success additionally requires the port's success predicates. Neither
receipt class substitutes for the other or turns an already Lost attempt releasable.
Preparing rollback is an adapter-internal historical restoration, never a managed
lease release. Unavailable proof retains the operation; Error/drop/terminal labels
are not substitutes. Protected standalone native launch remains unsupported before
reservation/process until its separately reviewed private authority exists; it
cannot borrow a Workflow operation lease or silently use a generic legacy path.
Here protected standalone means Task input with Issue19 typed pack ancestry/private
prepared-frame authority launched outside the exact managed entry point, including
start/resume/checkpoint and ReadOnly aliases. It does not mean every unprotected
Project consultation is such a protected Task. Issue19 owns that private frame/lease
extension; Issue6 owns its native adapter consumer and actual cleanup. Issue19's
minimum unsupported boundary is an interim safety gate, not acceptance of required
native consultation or interactive attachment.

No resume, checkpoint continuation, retry or fresh replacement may execute in the
same Session/worktree/lock scope until the earlier workload's cleanup is proven
and, for Lost, Issue14's separately approved audited recovery has released it.
Proven cleanup is necessary and never sufficient to release Lost.
Known-alive, pending, incomplete and unknown cleanup all block same-scope execution.
Lost is absorbing under ordinary adapter calls. Escalate to Human and retain the
reservation; a recorded human judgement, PID clearing or manual state change
does not certify cleanup, settle the operation or authorize implicit replay.
Later authority from the retained genuine owner may record factual cleanup; any
new recovery/release policy belongs to Issue 14's separately reviewed audited
contract and cannot be invented here from boot/PID hints. Other independent Tasks
can progress without touching the held scope.

A backend acceptance claim requires first-party platform contracts, actual
installed conformance and a private owned handle that cannot be deserialized or
adopted from recorded process numbers. Startup/fork/exec enrollment must be
covered from the first possible child, cross-project/workload migration must be
excluded, and bounded event/inspection loss stays Unknown. Selected-group proof
continues to be useful for that group; it is never extended to unobserved jobs.

Current installed Codex0.160.0 evidence establishes a separate native PTY SID and
a synthetic second-SID child surviving both native command termination and
owned stdio-server exit. The finite child later self-completes; only the wrapper's
selected owned server group is independently death-verified and reaped. This is
standalone command conformance, not a successful unified_exec/adapter cleanup
test. No currently verified macOS backend provides the required entire workload
authority. Linux delegated-cgroup and macOS scoped event mechanisms are design
candidates only; platform prerequisites and actual ownership/cancellation proof
remain acceptance blockers until their own gates succeed.

## Acceptance ordering

Clean published implementation checkpoints, and separately reviewed safe component
integration, may precede complete native platform acceptance. They must remain
explicit partial checkpoints with the unaccepted capabilities held/unsupported;
they do not close Issue 6 or advertise complete Codex Core. Issue 6 capability
completion requires both-host real native Executor/decision/Consult/resume/stop
and owned native interactive/TUI attachment proof, in the scopes required by the
product (native Goal support remains optional), and
actual private Issue 19 composition. Issue 16 then verifies the whole multi-Project
workflow using these real accepted routes. Neither Issue 6 nor Issue 16 can close
on unsupported-only, singleton file-only or synthetic no-subprocess results.
Recorded native limitations cannot waive a baseline host's required native
execution. If no compatible unprivileged backend is verified, keep that capability
blocked and escalate a concrete host/backend prerequisite decision; never infer
permission for privileged installation or silently change the required hosts.

## Acceptance evidence

- Real installed Codex executes in an isolated owning Task worktree.
- Native review produces a valid structured result without target operations.
- Explicit model/effort and unchanged native defaults are observable.
- Exact owned resume preserves rrx/native identities and refuses foreign scope.
- Interactive/native TUI route retains native trust/permission prompts; any
  unavailable prerequisite has an explicit typed outcome and holds capability
  completion; Unsupported is never native attachment acceptance.
- Callback correlation rejects foreign/replayed IDs and persistent grants.
- Two-Project context, environment, Git and native-reference isolation regressions.
- Bounded framing, failure cleanup, cancellation, stale revision and immutable
  review regressions fail under meaningful mutations.
- Existing workspace checks, immutable independent native review, exact-head
  Linux/macOS CI and factual limitations before component merge. These limits
  cannot replace the required native capability completion gates above.
- Genuine profile-specific workload authority before the actual first native
  execution consumer, including launcher, version, auth, policy, protocol and
  configuration probes; unsupported readiness executes no native process and
  sends no model frame or model-input admission/consumption. A real-consumer
  fixture and compiled mutation moving a native probe before owner acquisition
  must expose this boundary.
- Managed unsupported-profile refusal precedes reservation/marker. A separate
  actual post-marker owner-acquisition failure fixture proves that only tracked
  NoCurrentDispatch settlement can close the operation; rollback alone cannot.
  Protected standalone refusal occurs before reservation or any native process.
- A causal real-consumer fixture in which known parent-group death and a valid
  native terminal cannot certify an escaped/unverified workload, plus compiled
  mutations removing the caller readiness/cleanup guard.
- Actual pre-consumption startup helper/hook/MCP uncertainty under cancel, timeout
  or caller drop cannot publish historical rollback or release its scoped
  reservation; a caller-guard mutation must make this real-consumer fixture fail.
- Reduced-profile attach/permission/broker expansion is rejected before native
  side effects; a guard-removal mutation must expose that real caller regression.
- Actual executable/launcher and effective-input binding mismatch before dispatch
  or during any supported attempt cannot grant tools or certify cleanup. Exercise
  the running-native/kernel capability confirmation at the real grant consumer.
- Real-consumer immediate-daemon and deferred file-mediated delegation fixtures
  prove persistent Task work cannot start outside the profile's owned boundary.
  A guard-removal mutant exposes the rejection; enrolled descendants ending
  cannot certify a delegated job's cleanup.
- Installed real native proof on BOTH required host families for nested child
  sessions, retained ownership during stop/caller drop, bounded uncertainty and
  exact private Issue 19 settlement composition. Issue 16 separately carries the
  aggregate real native workflow gate. Unsupported/file-only fallback and
  synthetic no-subprocess controls do not satisfy platform/native MVP acceptance.
