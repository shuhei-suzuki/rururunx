# Issue 6: Native Codex adapter requirements

Workflow: STRICT (native authorization, process ownership and shared adapter APIs).
Dependency: Issue 4 is merged at `52e7bbb`; persistence, Git ownership and registry
foundations are available. Issue 19's actual private native operation/settlement
port is a managed-operation acceptance dependency; its composition is pending.
Issue 58's actual non-Task Consultant hold/settlement and Issue 14's audited
recovery/fencing are acceptance dependencies, not authority inferred from Session
labels. Issues 9–10 own the pending cross-agent decision-member/slot contract.
Native Task attach requires the composed Issues 6/11/15 frontend/driver/caller,
Issue 19 managed ownership/input and Issue 14 recovery contracts. Issues12/13 and
the open Issue60 Git follow-up must enforce later-artifact execution/publication boundaries.
Issues 5/7 supply their supported native profiles' mixed-workload enforcement.
[Issue60](https://github.com/shuhei-suzuki/rururunx/issues/60) owns the unfinished
runtime Git/helper and Generic lifecycle/enforcement follow-up to merged Issues3/4,
and the actual Issues12/13 job/evidence reservation/settlement composition. Its
runtime Git/helper scope covers every caller: Issues18–20 source/context capture,
Issue19 pre-marker and gate-claim physical/admission capture, Issue15 status/TUI,
Issue26 registry validation and Issue14 recovery inspection, as well as native
adapter attempts and job/evidence consumers. Those foundations stay closed; no new
workload proof is inferred from their merged status.
Issue 16 consumes the completed adapter in aggregate native dogfooding, rather
than being a
prerequisite for publishing an adapter implementation checkpoint.
Source: [Issue 6](https://github.com/shuhei-suzuki/rururunx/issues/6),
product requirements and master Agent Adapter design.

## Execution Domain normalization (#75)

Provider-neutral workload ownership, containment profiles, revocation and cleanup settlement are now governed by [Issue 75 requirements](issue-75-requirements.md) and the [Execution Domain master design](../design/master/execution-domain.md).

The detailed F1 material in this document remains binding for **Codex-specific** reachable-command inventory, native identity/effective-input binding, authentication/policy compatibility, protocol/session behavior and installed Codex conformance. Where it restates generic workload-lifetime rules, interpret it through #75 rather than as an independent weaker ownership system.

In particular:

- Codex receives an exact private Execution Domain permit before external execution; it does not create runtime ownership from PID/PGID/SID/native UUID evidence.
- Linux Strong containment is implemented/qualified by #76; macOS Strong by #77.
- Host-native selected ProcessGroup/inspection paths are Native/Compatibility evidence unless the applicable Strong backend is independently qualified.
- Persistent external execution uses #78 and cannot be settled from native parent/group death.
- #14 owns recovery of exact persisted Domain/delegation identities and #60 owns shared-effect compatibility.
- Existing Codex evidence does not retroactively upgrade any capability to Strong.

Current partial Stage A component availability is **EMPTY**. Ordinary launch,
resume, checkpoint, attach and approval paths refuse before new effects; there is
no ready workload backend or dispatch producer. The scoped source checkpoint may
merge independently, but none of the required native or Stage B acceptance gates
below is completed or waived by this refusal boundary.

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
    first execution of any attempt runtime or native process, including adapter
    Git/helper preflight, grant preflight, status/inventory observation and cleanup
    checks, launcher/version/auth/policy probes, configuration discovery and server
    startup. Pre-owner checks are
    limited to non-executing filesystem/registry metadata validation.
    Its supported profile must cover all processes the attempt can start: setup,
    native helpers, preserved hooks/MCP, tool commands and frontend children where
    enabled. Unsupported ownership fails explicitly before that first runtime/native
    execution and before model-input admission/consumption; no Git/helper or app-server probe
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
conversation history. Prepared context selection is owned by Issues18–20, with
the pre-marker execution/observation boundary below; review aggregation and
approval policy remain Issues9–10. CLI factory/TUI orchestration
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
Formal Reviewer phases (RequirementsReview, DesignReview, ImplementationReview,
SecurityReview), ApprovalReviewer under the pending Issues9/10 authority, and
no-command Consult can use a smaller profile only when its enforceable no-command inventory and its actual setup,
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
Identity-set admission requires a published per-host owned conformance artifact
covering the exact chain/digests, profile controls and delegation inventory. First
production use, successful metadata startup and caller configuration cannot add
an identity. A changed build remains typed held/unsupported until re-verified.
Re-verification of a changed native/runtime/Git/helper identity also requires its
applicable mixed-combination and B-already-live→A-Lost conformance against every
already-supported profile. Until then the affected combinations stay unsupported;
single-profile metadata/native execution proof cannot preserve mixed readiness.
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
Inert work products awaiting a separately governed execution/publication consumer
are not jobs already delegated by this attempt. Issue12 verification, Issue60's
runtime Git/helper follow-up to Issue3 and Issue13 PR/CI orchestration must establish their own current workload
ownership and command-safety gate before executing/publishing those artifacts;
this attempt's cleanup does not certify that later execution. Executable Git
hooks and shared/common-directory configuration that can execute commands must
be blocked by bound, enforceable profile controls before the first native execution.
These surfaces have standing ungoverned consumers, including user/IDE Git: hooks,
core.hooksPath/fsmonitor/sshCommand, credential/filter/diff/merge helpers, shell
aliases and executable include/includeIf configuration. Protect concrete configured
shared Git/hooks and ambient direct entry targets, including hooksPath indirection,
even when the directly named entry script is Task code. Classify further edges by
their actual trigger, not arbitrary tracing depth or mere file readability.
Automatic activation without a new governed runtime action (for example background
IDE/status fsmonitor, schedulers or autostart) requires a bounded complete closure
of its resolved executable/config/indirection edges, bound to actual inputs and
source digests. Unknown, cyclic, unbounded or truncated automatic closure is
Unsupported for the affected surface/profile; a traversal cutoff cannot certify
completeness. No current resolution/enforcement producer is assumed here.
Here automatic activation means a concrete configured Git/hooks/fsmonitor,
scheduler, service or autostart edge. IDE/language-server-intrinsic background
compilation, proc macros, build.rs or tests-on-save with no such configured edge
are the external same-UID application-boundary limit; they are not owned future
processes or a reason to classify every Rust source as an automatic closure.
For an explicit independent outside-runtime user Git/build action, its configured
hook/direct entry target remains protected, while transitive ordinary source,
tests or build.rs are not recursively reclassified merely because that action could
execute them. Their future execution is an explicit application-boundary limit.
Ordinary worktree code/scripts/build.rs stored for a later genuinely governed
runtime operation are inert work products; that operation freshly validates exact
sources/current authority and owns its actual workload. Runtime/native-triggered
Git/build execution stays governed by actual admission/cohort and the continuous
effect contract below. F1 covers runtime-owned workload/resource lifetime, not
external user/IDE-created future processes. Task worktrees are not claimed OS-private
from the same user. An independent action cannot disguise an already automatic
activation route, and no broad native-default/hook disable establishes acceptance.
Files consumed automatically by an external scheduler/service/autostart mechanism
remain current delegation routes. Preserve mandatory native hooks/settings; if
their required shared writes cannot coexist with this boundary, the profile is
unsupported and remains an MVP blocker, rather than disabling those hooks.
An explicitly requested edit to an ungoverned ambient activation chain has no
supported route unless a separately reviewed governed mechanism exists. Refuse
that required capability before reservation/effects when known; enforce the bound
write restriction before any later attempted activation change. A native tool
denial is not proof of cleanup or a fabricated terminal outcome. Record the
specific capability limit and settle/hold using actual outcome and cleanup rules.
A real attempted shared hook/config write plus ungoverned-reader fixture must
prove the profile refuses direct/automatic activation changes, including hooksPath
indirection to a non-hook direct entry script and a bounded automatic transitive
target. A separate explicit-user-trigger hook→ordinary-code fixture proves the
documented classification boundary without claiming that external execution owned.
Ordinary-code persistence followed by actual governed runtime/native execution
exercises fresh/current admission and owned cohort at that consumer. Required
target/dogfood surfaces whose closure or compatible enforcement is unresolved
remain MVP blockers, not a silent general Executor support claim.

All runtime Git/helper execution is subject to Issue60's pending owner, current
conflict reservation and default-deny contract, even outside a native attempt.
This includes Issues18–20 SourcePort/context/source capture, Issue19 pre-marker
physical hashing/admission validation, Workflow gate-claim/AllowCurrent source
capture, Issue15 status/TUI observation, Issue26 registration/reconciliation and
validation, and Issue14 recovery inspection. A read-only purpose or running before
the native marker does not exempt an external Git process or configured helper.
Every such capture must either use strictly non-executing in-process scoped reads
that cannot invoke fsmonitor, hooks, filters, textconv or external helpers, or
acquire its own actual Issue60 runtime workload owner and durable current conflict
reservation before its first execution. Bind the exact command/control and
effective inputs to the same completeness/default-deny inventory; explicit
no-ext-diff/no-textconv arguments cover only those bound paths, not all Git routes.
A pre-marker runtime capture is an independent runtime workload, not an early
native Issue19 phase lease or permission to move the native marker before context
exists. Retain its actual owner and reservation until its own known current outcome
AND complete enabled-profile cleanup are settled; an error, valid source digest or
successful ContextVersion/gate publication cannot release it. Its held/unknown
cleanup must prevent conflicting subsequent capture, native admission and release.
Unsupported capture readiness executes no Git/helper and cannot be bypassed by
retrying under a read-only label. No source producer or consumer is claimed here.

Under an unresolved hold, bounded scoped in-process Store/filesystem observation
may report factual state without execution, mutation, cleanup claims or hold
release. Existing data must remain identified as historical/held; an observation
alone cannot attest current input/admission eligibility. Any observation invoking
Git/helpers needs its own genuine current owner/reservation and actual continuously
enforced compatibility with the held workload, even in the primary root. A
read-only argv/role is not proof of nonconflicting execution. Missing compatibility
refuses that observation with an explicit held/unsupported reason, while safe
non-executing state reporting remains available. Fresh source publication still
requires its actual source currency and consumer fences; observation cannot mint
native operation authority or certify cleanup. These callers are runtime matrix
parties with the same already-live peer→Lost obligations, not passive agent-proof
consumers.

Every adapter-executed attempt process belongs to that attempt's actual workload,
including runtime Git/helper preflight, admission/grant rechecks, status/inventory
observation and cleanup checks. Enroll these under the same genuinely owned bound
profile/inventory, or compose an actual Issue60 runtime Git owner and settlement;
direct-child/selected-group completion is not their whole-workload cleanup.
Their configured automatic activation/delegation routes are covered or blocked
before execution under the same default-deny rule. No current consumer is claimed.
Before any owned runtime or native process/resource effect may have executed,
registered Preparing cleanup and exact historical rollback/factual failure remain
authoritative. Once any such process/effect may have executed, rollback,
terminal restoration and reservation release
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
input consumption. Any pre-marker source/context execution obeys the independent
runtime capture contract above; acquiring the later native attempt owner cannot
retroactively cover it. Implementation-owned unsupported profiles refuse before
clear_hold/reserve/context/dispatch_started and never fall back to legacy start.
For a supported profile, the successful exact reservation precedes Issue19's
atomic dispatch_started plus operation-lease transaction. That marker is not model
input consumption or cleanup proof. The managed entry point synchronously moves
the private operation handle into its registered owned attempt before any await
or caller-cancellable point. The supervisor acquires the attempt's genuine workload
owner before any runtime/native process execution,
and retains both through startup, admission and settlement. Runtime owner-acquisition
failure after the marker sends no runtime/native process or input, but cannot erase the
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
Admission/consumption followed by a verified zero-wire local failure is not a
native terminal outcome. Until a separately approved Issue19 receipt/recovery
contract covers that exact case, retain the operation held, do not mint either
receipt, and never replay the consumed input. Unknown delivery remains Lost;
the current conservative consumed-boundary Lost treatment is not weakened by a
claim of prewire failure. Proven cleanup alone cannot release an absorbing Lost
attempt. This is an explicit pending integration boundary, not an invented receipt.
Preparing rollback is an adapter-internal historical restoration, never a managed
lease release. Unavailable proof retains the operation; Error/drop/terminal labels
are not substitutes. Protected standalone native launch remains unsupported before
reservation/process until its separately reviewed private authority exists; it
cannot borrow a Workflow operation lease or silently use a generic legacy path.
Here protected standalone means Task input with Issue19 typed pack ancestry/private
prepared-frame authority launched outside the exact managed entry point, including
start/resume/checkpoint and ReadOnly aliases. It does not mean every unprotected
Project consultation is such a protected Task. Required protected Task execution,
resume and checkpoint use the exact managed path; no future standalone lease
extension is presumed or required to close this component. Issue19 owns the private
managed frame/operation port; Issue6 owns its native consumer and actual cleanup. Issue19's
minimum unsupported boundary is an interim safety gate, not acceptance of required
native consultation or interactive attachment.

Decision roles also require genuine retained native lifetime ownership. Existing
formal Workflow Reviewer phases (RequirementsReview, DesignReview,
ImplementationReview and SecurityReview) use the actual Issue19 typed managed
phase and private operation/settlement authority. DecisionReview is only conceptual
future decision-only terminology, not an existing Phase enum or callable port.
Cross-agent ApprovalReviewer requires Issues9/10's
separately reviewed retained decision-member/slot authority bound to the original
pending operation, request, input and sources, with its own native settlement.
A leased DecisionTask is only a pending design candidate, not an approved callable
producer. An operation-free decision means it cannot execute the reviewed action;
it does not mean its native workload has no lifetime lease. Issue58 is Consultant-
only and supplies neither decision role's authority. The requesting operation and
affected scope remain held while a reviewer workload is live/unknown or Lost,
until actual native cleanup and any separately approved recovery are established;
a review result or parent terminal label cannot release them. Each reviewer also
needs its own immutable decision CWD and F2 safety gate. Unsupported-before-effect
remains mandatory until these actual consumers compose; Broker acceptance is open.

The initial partial native interactive route follows the scoped gateway contract:
Project-only, text-only, decision-only Consult, one actual upstream actor and grant
ledger. It cannot attach as an interactive conversion to a managed Executor,
Reviewer or ApprovalReviewer attempt and cannot satisfy Workflow transport success
or its receipt. Every frontend-originated inference input and approval passes the
same exact current scope/Session admission/consumption and broker CAS before the
native wire; native permission/settings expansion is refused. Human input has its
own monotonic submission identity and current outcome, never the seed's completion.
This Project-only route does not satisfy the product's required native Task attach:
[product §34](product-requirements.md#34-cli-and-tui) requires `rrx attach <task>`
to connect to the native session. Task attach remains a mandatory open integration
blocker. Issue6 owns provider/frontend conformance; Issue11 the consult/attach driver
as applicable; Issue15 the CLI/TUI caller; Issue19 exact managed Task ownership and
phase-input authority; Issue14 restart/recovery/fencing. Their reviewed composed
Task-attach contract must preserve the sole actor, current admission/consumption
and grant ledger; neither input nor observed frontend output may bypass those
guards or certify a phase receipt. Whether a route is observation-only or permits
guarded submission requires that actual reviewed contract; neither is presumed
implemented here. Task attach cannot be converted into Issue58 consultation.
Native Goal support remains optional. A real unadmitted-frontend-turn fixture must
prove no inference or Workflow receipt can be certified from that turn.

Operation-less Project Consult/attach requires its own genuine private owned hold
and durable held-scope guard that generic Session relabel/deletion or operator JSON
cannot clear. [Issue58](https://github.com/shuhei-suzuki/rururunx/issues/58) owns
that non-Task retained ownership/settlement and primary/common-dir exclusion
contract; Issues6/11 consume its actual private authority and Issue14 owns the
separately reviewed audited recovery. Issue19 managed Task receipts do not supply
this authority. Until those
actual hold/release/recovery consumers compose, this profile is unsupported before
effects and remains a native Core acceptance blocker, not a ready capability.
The conflict extent includes the primary checkout and Git common directory,
including shared refs/config/hooks and worktree lifecycle metadata. An unresolved
hold (Lost, unknown ownership/enforcement or incomplete post-attempt cleanup)
blocks new conflicting primary-root execution, overlapping native launch,
base update/merge, worktree create/remove and Project removal/rebinding until
actual cleanup and required Lost recovery. Common-directory identity alone does
not blanket-exclude all live, currently owned governed work. Issue58's live
Consult conflict contract must compose the continuously enforced compatibility
matrix below before same-repository concurrency can be accepted. No current
producer or noninterference proof is assumed here. Independent work in another
Project/disjoint repository may
continue; a Task sharing this Git common directory is not automatically disjoint.
A generic-relabel fixture and actual primary/common-Git mutation/removal consumers
must prove the retained hold cannot be released by terminal labels or dead-parent
hints. Restart preserves uncertainty; recorded IDs never recreate the private owner.

No resume, checkpoint continuation, retry or fresh replacement may execute in the
same exact Session/worktree or an incompatible locked/effect scope until the earlier workload's cleanup is proven
and, for Lost, Issue14's separately approved audited recovery has released it.
Proven cleanup is necessary and never sufficient to release Lost.
Known-alive, pending, incomplete and unknown cleanup all block that incompatible
replacement; the wider same-repository concurrency boundary is defined below.
For held Task workloads (Lost, unknown ownership/enforcement or incomplete cleanup
after the attempt), the conflict extent includes every reachable Git common-directory
ref/config/hook and worktree-lifecycle surface. Apply the same new-conflicting-effect
exclusion above; a separate Task worktree is not automatically independent. This
uncertainty rule does not serialize all live owned work by common-directory identity.

Issue6 F1 coordinates the required provider-neutral reviewed live compatibility
matrix and continuously enforced effect contract, with Issue60 runtime Git/helper,
Issue19 managed-operation consumers and Issue58's Consultant conflict contract.
Issues5/6/7 and Issue60's Generic follow-up must supply each supported profile's own continuous enforcement
and both sides' protection/settlement; Codex-only proof does not cover another
provider. It must cover own-branch/worktree effects,
atomic append-only shared objects or a proven equivalent, shared executable surface
protection and current currency, from the first effect throughout native lifetime.
Different Task/branch IDs or Git locking alone do not prove those restrictions.
Unknown enforcement/profile remains unsupported; no passing live producer is
claimed here. The required four-plus parallel Task acceptance remains open until
actual matrix/enforcement and aggregate conformance succeed, not replaced with
serialization as a product change. A supported matrix must genuinely protect an
already-admitted Task B when Task A becomes Lost: prevent conflicting B effects
under its own enforcement/settlement path, or prove their continued noninterference.
Rejecting new rrx admission cannot stop already-running native B writes, and a Lost
label cannot retroactively do so. Require both positive same-repository live
Executors plus open Consult concurrency, and mixed-provider B-already-live→A-Lost
conformance across every supported required provider/profile combination.
Runtime-owned workloads are also matrix parties, not merely consumers of agent
proof: Issue60 owns the actual runtime Git/helper/Generic resource and current
conflict reservation/settlement contract; Issues12/13 supply their actual test,
regression/mutation/browser/staging and PR/CI/merge/cleanup job/evidence consumers.
Issues18–20/19 pre-marker/gate-claim capture, Issue15 status/TUI, Issue26 registry
and Issue14 recovery execution are also actual runtime matrix parties and Issue60
composition consumers; their non-executing reads are distinguished above.
These producers must supply continuous enforcement, durable held/Lost uncertainty,
known current outcome AND complete enabled-profile resource cleanup, and both-side
protection/settlement through caller Drop, cancellation and failed publication.
No native Issue19 phase lease is invented for EvidencePort. Their separately
reviewed delegated-service contracts bound actual applicable resources; F1 does
not claim all future cloud/deferred jobs or external user/IDE processes owned.
Require positive runtime Evidence/Git/verification↔native and runtime↔runtime
concurrency plus already-live peer B→A-Lost enforcement/settlement conformance.
Until these actual consumers compose, affected combinations remain unsupported
and required four-plus concurrency open. Whichever issue enables a new provider
or runtime profile must prove its mixed combinations against every already-supported
applicable profile before advertising readiness; closing this component cannot
pre-authorize a later profile or skip those gates.
Every runtime matrix party applies the same default-deny delegation inventory
and completeness basis, bound to its actual executor/helper identity and effective
inputs. Each reachable immediate-daemon, service or deferred file-mediated execution
route must have genuine current ownership/cleanup under an actual reviewed service
contract, or be blocked by bound enforceable controls before the first runtime
execution. Otherwise that runtime profile is Unsupported. The limit on future
cloud/deferred-job claims does not exempt any route already reachable from this
attempt. Actual Issues12/13/60 consumers must exercise immediate and deferred
delegation fixtures and causal caller/readiness mutants.
Lost is absorbing under ordinary adapter calls. Escalate to Human and retain the
reservation; a recorded human judgement, PID clearing or manual state change
does not certify cleanup, settle the operation or authorize implicit replay.
Later authority from the retained genuine owner may record factual cleanup; any
new recovery/release policy belongs to Issue 14's separately reviewed audited
contract and cannot be invented here from boot/PID hints. Independent work in
another Project/disjoint repository can progress without touching the held scope.

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
product: Project-native Consult and native Task attachment are required; native
Goal support remains optional. Completion also requires actual private Issue19
managed composition, Issue58 non-Task hold/settlement, Issue14 recovery/fencing,
Issues9/10 retained approval-decision ownership, the Issues6/11/15/19/14 composed
Task-attach contract, and Issues60/12/13 actual runtime workload and later-consumer
execution/publication guards. Issue60 is a pending producer, not an existing port.
The provider-neutral live compatibility/enforcement matrix supplied by each actual
Issues5/6/7/Issue60 Generic/runtime and Issues12/13 supported workload, composed
with Issues19/58
must also meet product §31's four-plus simultaneous Task requirement; an unresolved
concurrency boundary is a Core blocker, not permission to serialize the product.
These are acceptance dependencies, not implied existing ports. Issue 16 then
verifies the whole multi-Project
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
- Genuine profile-specific workload authority before the actual first runtime/native
  execution consumer, including launcher, version, auth, policy, protocol and
  configuration and adapter Git/helper probes; unsupported readiness executes no
  runtime/native process and
  sends no model frame or model-input admission/consumption. A real-consumer
  fixture and compiled mutation moving a native probe before owner acquisition
  must expose this boundary.
- An actual adapter runtime preflight/helper starting a detached configured
  fsmonitor/hook/service before native startup cannot permit Preparing rollback,
  NoCurrentDispatch or native completion to release the attempt from direct-child
  death; same/composed owner cleanup is required. Mutate the real caller guard.
- Actual pre-marker context/source/admission and gate-claim capture, status/TUI,
  registry and recovery callers exercise configured fsmonitor plus filter/textconv
  fixtures. Pure in-process reads cannot trigger them; executing consumers must
  refuse before execution or prove genuine ownership, complete enabled cleanup and
  retained holds before release. A native attempt owner acquired later cannot
  certify earlier capture cleanup. Mutating each real execution/readiness or
  settlement guard must expose the bypass. Separately prove bounded non-executing
  held-state reporting remains available and cannot claim freshness/cleanup, while
  unproved conflicting Git observation is refused; no blanket primary-root status
  denial substitutes for the compatibility classification.
- Runtime Issues12/13/60 immediate-daemon and deferred file-mediated delegation
  consumers prove routes genuinely owned or denied before execution; absent actual
  contract or bound denial stays Unsupported, not an omitted enabled resource class.
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
- A later execution consumer refuses attempt-written executable hook/config
  private artifacts without its own current ownership and command-safety admission.
  A separate attempted shared hook/config write with an ungoverned reader proves
  pre-execution profile enforcement; a guard-removal mutant exposes the refusal.
  Include direct hooksPath indirection, bounded automatic activation closure,
  explicit independent user-triggered hook→ordinary-code classification and actual
  governed runtime/native execution of ordinary persisted code. Unknown automatic
  closure refuses; same-user readability alone cannot classify every file as delegation.
- A Lost Task workload with reachable common-Git writes prevents another Task's
  actual shared-Git mutation/base/worktree-lifecycle consumer; removing that caller
  hold guard must fail the fixture. Different Task IDs do not establish independence.
- Positive live same-repository Executors and an open Consult progress under the
  actual compatibility matrix. With B already admitted/running when A becomes Lost,
  actual enforcement prevents conflicting B effects or proves noninterference;
  a caller/enforcement mutation fails this fixture. A new-admission-only test cannot
  certify the existing native workload. Exercise mixed required provider/profile
  pairs in both roles, actual runtime Evidence/Git/verification↔native and runtime↔runtime
  pairs, including failed publication/Drop and already-live peers. New profile
  producers rerun combinations with all applicable already-supported profiles.
  Issue16 retains aggregate four-plus proof.
- A Lost approval reviewer keeps its requesting operation and affected scope held
  at the real broker/settlement consumer until actual native cleanup and separately
  gated recovery; removing the retained member/slot guard must fail the fixture.
- Real native Task attach on both required hosts satisfies product §34 through
  the reviewed composed frontend/driver/caller/managed-input/recovery contract;
  Project-only Consult attach and Unsupported cannot satisfy this requirement.
- Installed real native proof on BOTH required host families for nested child
  sessions, retained ownership during stop/caller drop, bounded uncertainty and
  exact private Issue 19 settlement composition. Issue 16 separately carries the
  aggregate real native workflow gate. Unsupported/file-only fallback and
  synthetic no-subprocess controls do not satisfy platform/native MVP acceptance.
