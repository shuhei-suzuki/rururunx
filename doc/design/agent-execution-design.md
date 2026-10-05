# Native Agent execution design

Risk: STRICT. Status: Phase1 design proposal, not implemented or native-qualified.
Implements [requirements R1–R9](../requirements/agent-execution-requirements.md).
Requirements were independently approved at
`1eccba7b4ea24566c1299d66b9ef5e8ee379dc74` before this design was written.
The Phase1 verification record records immutable review commits and limitations.
Human approval at the phase boundary is required before production changes.

## 1. Selected contract and boundaries

This profile protects accepted results by retained commit objects, isolated
review/test inputs, fresh attempts and cooperative resource policies. It does
not prove workload death. Native Claude Code/Codex run as the logged-in host user
without an outer sandbox. rururunx is not a security sandbox. Cleanup is best
effort on both OSes; no Tier/custody or complete descendant collection is claimed.

Retained Git objects and evidence live outside executor namespaces. They are
protected from ordinary writes to an old worktree, not from arbitrary same-user
absolute-path access, host failure, storage exhaustion or destructive shared
commands. Hooks/plugins must follow the declared resource profile. Unknown
undeclared behavior is reported as uncovered if discovered. Declared incompatible
behavior is rejected before effects or requires a recorded user exception; an
exception lowers coverage, never silently makes the behavior isolated.

Linux optionally collects a user scope; macOS uses the owned process group and
bounded cookie discovery. No launchd, privileged helper, VM, namespace, entitlement
or provider-authentication replacement is required. `cargo install` delivers one
rrx binary including its internal helper entry points. Git and the supported,
logged-in official native CLIs remain external prerequisites. Docker/build tools
are optional, required only by Tasks that declare them.

## 2. Components and execution identity

The existing WorkflowEngine remains the authority for phase/evidence advancement;
Session is native transport state, not Task truth. Add these internal components:

| Component | Responsibility |
| --- | --- |
| AttemptManager / Store | Scope, generation CAS, fresh binding, launch intent, logical retirement and historical work outcome. |
| ResultStore | Independent per-Project bare Git repository, captured object graphs, hashed evidence files and retained refs. |
| SnapshotManager | Fresh detached review/verifier repository/workspace for one pinned artifact; no live executor reads. |
| ResourceManager / TaskTool | Durable unit leases, Runtime IPC admission and managed tool invocation policies. |
| QuotaScheduler | Shared account/window state, executor/reviewer leases, fair waiting and bounded recovery. |
| Reclaimer | Independently queued, bounded tracked-resource cleanup and coverage observations. |
| Native adapters | Official CLI dispatch, native terminal/quota decoding, permissions and cancel signals; no source retrieval or Task-success decisions. |

`ExecutionUnitId` is UUID; unit kinds are executor attempt, reviewer and verifier.
`AttemptId` identifies one executor unit. Each unit stores Scope, Task generation,
parent artifact (review/verification), phase, provider, SessionId, owner Runtime
epoch and monotonically increasing record version. A Task has `active_attempt_id`
and `generation`; its `worktree`/`branch` fields remain a current-attempt projection.
Historical Session `worktree` and its unit binding are immutable. Multiple reviewer
units may share artifact identity but never a writable snapshot/resource namespace.

Every operation carries `ExecutionAuthority {scope, unit_id, task_generation,
runtime_epoch, session_id, version}`. Native thread/turn/request identity is also
checked where available. Public SessionRef and PreparedInput gain explicit unit
and artifact identity. No wildcard generation or Task-only fallback may authorize
publication, tool launch, grant or callback. Separate accounting updates can refer
to a retired unit without restoring its authority.

Authority has two separately persisted permissions: `native_effects_open` and
`result_finalization_open`. A reliable natural terminal closes native tool/input/
grant permissions but leaves Runtime-only finalization open for the known work.
This permits exact commit capture/publication, not another Agent turn or remote
mutation. Cancellation, replacement and epoch fencing close both by generation
CAS. Waiting/status updates preserve the same native turn's semantic authority;
Store acquires a fresh Task version for CAS and validates unit/generation/phase
rather than requiring an obsolete launch-time Task version to remain unchanged.

The Runtime exclusively locks its state-root lockfile using a kernel file lock.
Runtime instance UUID is the durable state-root namespace, unchanged on restart;
owner epoch changes on each new scheduler. Paths never move merely due to restart.
The descriptor is close-on-exec and never inherited by native children; test this
on both OSes. Only the owner writes scheduling state. Restart acquires the lock,
increments the persisted epoch and fences old units. Lock release proves that
the former scheduler is gone, **not** that its Agents are gone. No TTL-based
concurrent scheduler takeover. Read-only status clients use SQLite and IPC;
mutating commands go through the owner. Lost IPC cannot authorize helper effects.

## 3. Persistence and schema 4

Current State is schema 3 (`crates/rrx/src/state/schema.sql`). Keep existing scoped
projects/goals/tasks/records/context/usage/audit tables. Add typed tables instead
of placing cross-unit uniqueness solely in unindexed JSON. All scoped rows carry
`project_id, goal_id, task_id` with composite FK to existing Task scope; body JSON
must be valid and deserialized with strict validation. IDs/versions and indexed
fields must agree with body values. Enum values and numeric ranges are CHECKed.

| Table | Keys and essential fields / constraints |
| --- | --- |
| runtime_epoch | Single row for state-root identity, current epoch and Runtime instance UUID. Update only under exclusive lock. |
| execution_units | PK unit_id; Scope FK; kind; generation; phase; owner_epoch; state; native_effects_open and result_finalization_open; work_outcome nullable until observed; disposition; cleanup_outcome default unknown; version; fresh absolute worktree/branch; base_sha; profile_digest; Session reference. UNIQUE worktree, UNIQUE non-null branch per Project. Partial UNIQUE executor with either permission open per Task. |
| task_execution | PK Task scope; current generation and active_attempt_id FK. Store validates referenced unit belongs to same scope/kind/generation. CAS atomically updates Task JSON projection and Workflow. |
| session_units | Session record ID + Project FK; unit_id FK; exact scope; native dispatch/ack state and bounded protocol identity metadata. One Session belongs to one unit; a new native process gets a new Session. |
| result_artifacts | PK artifact_id; Scope/unit FK; kind; exact full SHA/base; object format; manifest digest; storage location; state staging/ready/published/invalid; version. Published rows and identity fields immutable. |
| artifact_dependencies | artifact FK; named dependency kind/id/digest/source version; snapshot IDs and required-content manifests. No credentials or unrestricted environment dump. |
| resource_leases | PK lease_id; unit FK; resource kind; host/state-root namespace; value; intent ID; state reserved/creating/active/quarantined/released; version. Partial UNIQUE kind+namespace+value for nonreleased leases; allocator transaction checks port-range overlap across all Projects and abandoned units. |
| managed_effects | PK operation_id; unit FK; kind; idempotency key/digest; expected target; pending/confirmed/unknown/resolved; bounded nonsecret receipt. UNIQUE operation/idempotency key. External ambiguity gates only its dependent phase. |
| cleanup_jobs / observations | Job unit FK, version, next_due/backoff, cursor and deadline; append-only observations with timestamp, resource coverage, outcome, remaining safe IDs and bounded categories. Unit aggregate points at observation ID. |
| quota_pools / windows | PK provider+opaque account key; bucket/model key, unknown/available/exhausted/stale; optional percentage/reset; observed_at, source/version, freshness and window identity. NULL means unknown, never zero remaining. |
| quota_leases / waiters | Unit FK; pool/buckets; epoch; executor/reviewer role; active/released; wait reason, next_probe_at and fairness sequence. Unique active unit lease; cancelled/retired units cannot acquire. Leases estimate concurrency, not promised token consumption. |

Constraints crossing tables (scope, active generation, resource overlap, source
freshness) are enforced by dedicated Store operations under `BEGIN IMMEDIATE`,
not an unlocked check followed by generic `put_record`. Keep append-only audit and
immutable ContextVersion triggers. Dedicated operations include reserve_unit,
ack_dispatch, publish_result, retire_unit, reserve_quota, record_quota,
reserve_effect, reconcile_effect and record_cleanup. They atomically validate
Project/Goal activity and version, Task CAS, Workflow reservation and unit authority.
Native calls, Git, file copies and OS scans run outside the transaction; their
receipts are revalidated on entry. No blocking I/O while holding Store mutex.

Migration v3→v4 runs in one transaction, validates old records and creates tables
before updating `user_version`. New Task JSON fields use validated defaults;
older fields and historical records are retained. Existing bound worktrees become
`LegacyUnreconciled` units with unknown work/cleanup and reserved historical paths,
not new reusable attempts. Do not derive signal authority from stored PID/PGID.
Old native dispatch/pending external markers remain unresolved until reconciliation.
Malformed or contradictory legacy data aborts migration unchanged. A new local
attempt can be allocated after explicit legacy logical fencing; its external
dependent phase still cannot replay an unknown remote effect. Schema 3 readers
must reject version 4 on new open. Already-open schema 3 connections are a
separate case: the old version check runs only on initialize and the new owner
lock cannot fence them. Migration installs BEFORE INSERT/UPDATE/DELETE guards on
every existing and new mutable application table, calling a connection-local,
zero-argument `rrx_writer_contract_version()` function and rejecting any value
other than 4. New writers register the side-effect-free function through safe
rusqlite functions before migration/use; it is trigger-usable and innocuous, not
DIRECTONLY. Old connections lack it, so post-migration writes fail, including
cached statements recompiled after schema change. Do not use a global SQL flag
which old connections could inherit accidentally. Existing write transactions
serialize before migration; old read snapshots cannot silently upgrade through a
new committed schema. This fences DB writes, not legacy native processes/effects.
Tests must retain an old Store connection across migration and exercise cached
and fresh INSERT/UPDATE/DELETE/REPLACE, old read→write transactions and successful
new-writer operations, plus missing-function failure. Treat guard registration or
coverage failure as migration refusal. Round-trip, failed-migration and new-open
older-reader tests are also required; opening a DB does not launch or kill anything.

## 4. State machines and reservation replacement

Work outcome is nullable before reliable observation, then success/failure/unknown.
Task cancellation is a disposition (`cancelled`), not an invented work failure.
Cleanup is independently unknown/reclaimed/leftovers with coverage/time. Native
success only means a reliable successful native terminal; Workflow success still
requires its exact evidence/review/test policy.

| Event | Unit/Task transition and action |
| --- | --- |
| Admission | Create Reserved unit, increment generation on new executor attempt, bind fresh names, reserve resources/quota; persist intent before preparation. Ineligible capacity queues/refuses without launch. |
| Start | Reserved→Preparing→DispatchPending; record Session/dispatch intent before spawn. Confirm acknowledged native start as Running. Unacknowledged dispatch stays unknown on crash. |
| Reliable native terminal | Running→WorkKnown; close native_effects_open, retain result_finalization_open and capture dependencies. Preserve success/failure; process cleanup may start concurrently, destructive input disposal waits for finalization. Successful candidate enters result capture/evidence. |
| Transport loss without terminal | WorkUnknown; fence local write/result/grant authority; checkpoint only supported data; queue cleanup. New local attempt permitted with fresh paths, external ambiguity still reconciled. |
| Confirmed quota, native still retrying | Task WaitingQuota, unit retains same Session/turn and existing lease. Do not send duplicate input or create another live executor. |
| Confirmed quota, native turn ended | Task WaitingQuota; unit retired with quota-interrupted disposition, work unknown unless already known. Preserve durable drafts/checkpoint references. Fresh attempt after recovery. |
| Cancel | CAS retires current unit authority, increments generation, marks Task cancellation disposition and closes quota waiter; queue cleanup. Other Task units untouched. Late native events become historical diagnostics only. |
| Retry/fix | Choose explicit pinned base/draft policy; create new unit/worktree/branch and resources, never old path. Failed cleanup is a backlog item, not whole-Task reservation. |
| Review/test | Reserve independent units from published artifact. Accept evidence only for exact artifact/source versions; no executor-path lock or process-death prerequisite. |
| Cleanup observation | Update historical cleanup and lease quarantine/release; never change work, Task phase, accepted artifact or newer unit authority. |
| Runtime restart | Fence prior epoch, reconcile durable result/effect intents; old unacknowledged/active work stays unknown. Independently schedule recovery/cleanup, never send stale permissions. |

All active mutation reservations are **unit-specific**. Reviewer isolation permits
reviewing a prior accepted SHA while another executor unit uses another worktree;
Workflow may advance only if artifact/dependency versions still match its policy.
Keep exact source freshness, reviewer identity/quorum, permission/risk and unknown
external-operation guards. Replace Lost-derived whole-Task exclusion; Lost still
describes a genuinely missing transport, not evidence of physical ownership.

Timeout/release of an executor quota lease after fencing permits new cooperative
work, not proof the native subscription stopped consuming capacity. Account
exhaustion can still result from a survivor; report this limitation and apply the
next official quota observation pool-wide. Do not create unlimited retries.

## 5. Git capture, snapshots and publication

Retained-result inspection after retirement/restart has a separate Runtime-only
read capability. Validate the current owner epoch and the complete indexed
Ready/Published artifact snapshot before effects; its historical executor unit
must retain the same Scope. No old generation/Session permission is reopened.
Persist a `retained_git` intent containing artifact ID/version, manifest digest,
current reader epoch and finite action before spawning current Runtime Git. The
allowlist contains only exact commit/base `rev-parse`, complete graph `rev-list`
and `fsck`. Disable lazy fetching/replacement objects for this historical reader,
use an operation cookie distinct from the producing unit, and omit native IPC/
profile grants. Do not require old executor/profile/temp paths. Git resolution
uses the current Runtime PATH and canonical executable; the receipt hashes its
path reference, not its contents/version. No implicit qualification claim follows.
Bound manifest reads to 128 KiB plus one sentinel byte, Git output/wait and
in-flight epoch/artifact checks. Confirm a helper receipt only in a transaction
that revalidates the same artifact/epoch. Drop, timeout, invalidation or lost epoch
keeps an unknown helper observation. Public publication CAS rechecks the complete
verified artifact snapshot, rather than trusting only its ID. Historical read
failure never writes Task work, accepted evidence or cleanup disposition.

Runtime internal Git commands remove inherited repository/common-directory,
object-store, namespace and ancestry-routing variables and disable replacement
objects and lazy fetch; native Agent auth/settings/hooks remain inherited. Physical
retained storage must be canonical and independent: refuse `commondir`, alternates,
`shallow` and `info/grafts`, including symlink entries. Check before/after capture
fetch and before graph inspection; no file-read failure implies absence. The
[Git repository environment](https://git-scm.com/docs/git) documents common-dir
and object routing. [Git's shallow semantics](https://git-scm.com/docs/shallow)
and [graft reader](https://github.com/git/git/blob/v2.49.0/commit.c) interpret
ancestry metadata: successful rev-list/fsck alone does not establish full original
ancestry under those overrides. Removal of `GIT_SHALLOW_FILE`/`GIT_GRAFT_FILE` is
defensive; their direct environment behavior has not been established for the
tested Git version, and is not credited as a reproduced exploit.

State layout (names are logical; path validation forbids nesting state/result
storage under a Task workspace or aliasing sibling namespaces):

```text
state/<runtime-id>/projects/<project-id>/results.git
state/<runtime-id>/artifacts/<artifact-id>/manifest.json + evidence files
project-worktree-root/<task-id>/<attempt-id>/executor
unit-root/<unit-id>/snapshot.git + workspace + tmp + output + tool-bin
```

Executor worktrees start at a selected full base SHA with a unique
`rrx/<task-id>/<attempt-id>` branch. Runtime native Git calls validate Project
identity, canonical common directory, own path and protected base without reading
provider credentials. Existing common-repository checks remain for executor
worktrees. Snapshot validation uses a distinct ResultSnapshot provenance type;
do not bypass executor checks with arbitrary different Git directories.

Result capture protocol:

1. Persist `result_artifacts(staging)` for current authority with requested full
   commit/base and safe manifest destination. The native terminal/commit proposal
   is a candidate, not accepted evidence. Require the commit belongs to the
   selected task lineage and does not mutate a protected ref.
2. Fetch the exact commit/base object graphs over local Git transport into the
   independent result repository. No `--shared`, `--reference`, shallow/partial
   object dependency or hardlinks to executor objects. Explicit full object IDs
   are used, not branch HEAD re-read. Reject missing/promisor objects or incomplete
   required parents/trees/blobs; preserve object format. Source modification may
   make capture fail, but cannot select different accepted content.
3. Verify object connectivity/content IDs and create private reachable refs per
   artifact. Write manifest/evidence to fresh temp files, flush/sync and atomically
   rename; ensure durable parent entries before marking ready. Git object/ref
   durability is explicitly checked/synced for the required Runtime-SIGKILL case;
   arbitrary power-loss/device-failure durability is not promised.
4. `publish_result` CAS checks result-finalization permission and unit/generation/epoch, Workflow/source
   versions and content digest; records published artifact and ContextVersion in
   the same DB transaction. Cancellation winning this race rejects publication;
   abandoned captured content remains a draft/retention item. Successful publication
   closes finalization permission. Failure to capture closes it only after a durable
   unresolved/draft disposition, never by silently deleting the source.
5. Crash after file/ref write but before DB publication leaves staging data.
   Restart verifies it and reconciles as ready/draft, never auto-publishes for a
   fenced generation. DB published with missing/corrupt content becomes invalid
   evidence and stops affected advancement; never invent a replacement SHA.

For each review/test unit, make a **separate independent Git repository** from
retained objects and detach at exact artifact SHA. It has its own Git admin data,
index and writable build output; it does not share mutable refs/config with the
executor or another reviewer. A clone implementation must use ordinary transport
(`--no-local`) without alternates; the Runtime result repository is serialized
only for short ref/import/retention operations. Verify actual materialized HEAD
and tracked-source manifest before and after review/test. These rechecks are
diagnostics, **not proof** against a source change consumed and then restored.
Before dispatch, SnapshotManager removes write permission from tracked source
files/directories and pinned Git admin content, retaining executable/read bits;
declared writable output/temp paths are separate and cannot contain tracked input.
Native reviewers additionally require their supported read-only review capability.
Verifiers use qualified tool profiles that keep source read-only and place all
generated content in output paths (for example Cargo target/OUT_DIR outside source).
Never fall back to writable source after an error. Source-writing generators or
hooks must be assigned an explicit derived-output profile whose inputs/tool/digest
are recorded and validated, or the required verification is refused as unsupported.
An ordinary mutate-consume-restore control must fail on its first source write,
not pass because final hashes match. Mixed source/output directories, source
symlinks/aliases and required incompatible hook behavior refuse that profile.

This is cooperative input protection, not a same-user security barrier: permission
changes, alternate absolute inputs and tools/plugins bypassing the declared profile
are uncovered. A chmod-and-restore/raw bypass control demonstrates this limitation
and must never be reported as proven stable-input acceptance. Profiles permitting
such behavior cannot qualify R2; detection invalidates evidence or records unknown
coverage. Do not claim rechecks detect every undeclared transient bypass. Native
user/auth/config roots are never write-protected; only the disposable source copy
is. No namespace/Seatbelt or extra Agent permission mode is applied to executors.
Required source checks never permit executor-path reads as authoritative data.

Tracked rules/config come from the same commit; external project rule files are
explicit noncredential references with content hashes captured into the artifact.
Provider user settings/config roots remain inherited by official CLI, unparsed by
rrx; record a nonsecret user-declared compatibility profile/version, not credential
content or a hash of credential storage. Preserve required Git/native hooks by
explicit path references; never copy an entire user configuration tree. Hooks
that require a mutable common repository or sibling writes need a declared
compatible integration/exception or refusal. Phase3 verifies this, including
checkout-specific hook behavior. Changing non-versioned user settings during a
review is an uncovered input; do not call the whole execution deterministic.

Initial supported required-content profile: ordinary complete Git repositories
without required submodule/LFS materialization or external symlinks into another
Task. Detect gitlinks, LFS pointers/filters and escaping symlinks before Task
effects; refuse with the named missing capability when required. Do not globally
enable network hydration or copy credential-bearing configuration to make this
pass. Later profiles may implement pinned recursive object retention explicitly.

Publish/merge uses retained artifact SHA, expected-old target and pending
operation receipt. A merge creating a new commit requires its own validation
against the reviewed parent/diff policy; never call a newly synthesized commit
the already reviewed SHA. Ref drift invalidates that gate and requires rebase/new
attempt/review. Remote PR/merge requests use idempotency/reconciliation keys and
preserve native permissions; an unknown response is not a retry authorization.
Retention only releases refs after no workflow/review/publish dependency needs
them and the explicit retention policy expires. Never use Task cancellation as
permission to remove another unit's retained objects/evidence.

## 6. Native launch and environment compatibility

Add provider-owned `NativeInheritedEnvironment` launch mode for Claude/Codex.
Inherit the invoking user's normal native environment through the process API;
rrx does not enumerate/copy auth files or serialize credential values. Apply only
validated Runtime resource overlays and configured nonsecret project references.
Do not change HOME, CODEX_HOME, Claude user settings roots, SSH_AUTH_SOCK, native
permission/sandbox policy or mandatory hooks. Resolve official CLI paths before
TaskTool PATH overlays. Avoid fallback to API-key/billed API execution.

The existing Generic/Grok explicit environment contract and foreign-override
guards are retained; their behavior is not silently converted to broad inheritance.
Runtime routing/resource overlays cannot be set by caller-provided arbitrary
environment. Reject conflicting reserved keys. No full environment, auth headers,
argv secrets or native raw output is written to audit. Existing bounded raw tails
remain memory-only; persist normalized nonsecret events and safe resource IDs.

Every managed unit gets `RRX_UNIT_ID`, `RRX_TASK_ID`, `RRX_GENERATION`,
`RRX_PROCESS_COOKIE`, `RRX_TMPDIR`, `TMPDIR`, `RRX_OUTPUT_DIR`,
`RRX_PORT_START`, `RRX_PORT_END`, `RRX_DOCKER_PROJECT`, and resource profile digest.
Cookie is random provenance, not a secret/authentication boundary. TaskTool IPC
contains these identifiers and is validated against current DB authority.
Preparation helpers receive the same unit profile. Authentication remains in the
official native CLI, including Docker's own CLI when needed.

Codex keeps app-server event/RPC/permission framing and exact native thread/turn
validation. Replace ownership-only Availability with a conformance manifest and
R2–R4 admission. Main's `codex-cli 0.160.0` pin is a source baseline, not a claim of
Phase3 compatibility. Integrate/review the unmerged Claude candidate before
advertising it; preserve its structured headless terminal and required hooks.
Supported version manifests require committed schema fixtures and negative
controls in Phase2; actual account/hook acceptance is separately recorded in
Phase3. Unsupported features return capability errors, not hidden fallback.
Do not change user statusline, permission mode or hooks to collect quota signals.

## 7. Managed resources and tool policy

Defaults: four executor slots globally with configurable Project/provider limits,
plus two reserved global reviewer/verifier slots when host capacity permits.
Resource allocator includes retired units' quarantined leases. Port allocations
are 32-port intervals in a configurable nonprivileged pool; exhaustion queues with
`WaitingResource` or refuses before launch, not `WaitingQuota`. Bind failures
reconcile the interval rather than stealing a sibling port. Paths are created
with no-follow/canonical-parent checks and checked for aliases. No state-root
cleanup command may traverse a symlink into user or sibling storage.

TaskTool is implemented in the installed rrx binary, exposed by Runtime-generated
unit-local tool-name shims. Only declared/version-qualified command forms are
mediated; explicit `rrx task-tool` invocation is also supported. Shims send bounded
normalized intent to Runtime over local IPC; it records intent and checks unit
authority before permitting the tool. The helper then execs the pre-resolved
official binary with its own normal authentication. Real binary resolution avoids
shim recursion. No general socket interception, credential proxying, shell string
evaluation or global user config rewriting is introduced.

| Tool | Selected managed policy / limit |
| --- | --- |
| Git | Runtime-only validated `-c`/config-env overlays: gc.auto=0, maintenance.auto=false, core.fsmonitor=false. Serialize common ref/maintenance operations; never disable hooks or set global config. Reject managed global gc/prune/maintenance/config mutations. Raw bypass remains uncovered. |
| Docker run/create | Allocate operation UUID, deterministic unique container name and mandatory Runtime/Project/Task/unit labels; journal before call, enforce exact label keys and own ports/mounts/network. Persist returned ID then inspect matching labels. Lost acknowledgement reconciles by name+labels, never blindly reruns. No shared writable mounts/raw socket/global prune. |
| Docker Compose | Unit project name; validated normalized config with required service/container/network/volume labels and names, private mounts/ports; reject external/shared volumes/networks, fixed conflicting names, unsupported includes/plugins. Invoke official CLI using temporary config without logging credential fields. Cannot silently rely on COMPOSE_PROJECT_NAME alone. |
| Gradle / gradlew | Managed entry supplies --no-daemon and private Gradle user home/daemon registry and caches. It may still create a disposable single-use JVM. Do not call shared --stop. User init/properties with credential content are not copied; projects needing them require an explicit native-compatible reference/profile or are unsupported. Wrapper scripts must use the managed entry; direct ./gradlew bypass is uncovered. |
| Bazel / bazelisk | Unit output_user_root/output_base; select tested batch/private-server profile. No shared bazel shutdown. Preserve project/user rc by native handling unless incompatible overrides demand refusal. |
| sccache | Unit cache and server endpoint for supported versions, with tested local-only configuration; disallow shared stop-server. Alternatively qualified compile-wrapper bypass disables it; merely clearing an env var does not disable project hard-coded sccache. Unsupported endpoint/config combinations refuse the declared tool. |
| tmux | Force unit private socket path (-S) for supported commands; reject conflicting socket override. kill-server is allowed only on that exact private endpoint. User tmux config may spawn undeclared effects and needs compatible profile. |
| SSH | Managed calls set ControlMaster=no, ControlPath=none and ControlPersist=no; prohibit shared control -O commands. ControlMaster=no alone still permits existing-master reuse. Native keys/agent/config are handled by ssh, not rrx. |

Runtime-owned overlays are finite supported profiles, not interception of every
Agent shell or hook. Absolute binaries, PATH resets, nested containers, tool
plugins, raw API calls and new undeclared tools can bypass them. Inputs must state
supported tool entry forms; detected bypass is recorded as an unmanaged effect
and coverage exception. Do not promise arbitrary future command isolation.
Managed tool requests from a cancelled unit refuse further effects; already
dispatched unknown operations are reconciled under their historical identity.

Shared Git lock handling uses known operation receipts and narrow live ownership,
not age-based deletion. Record holder and affected operation; wait/refuse only
the dependent common-Git operation. Never force-remove an unknown lock. Previously
published results and independent snapshots remain usable. Cleanup worktree removal
does not force dirty deletion or remove shared objects/refs belonging to siblings.

## 8. Quota observations and scheduling

Provider account key comes from an opaque official native account identifier
when exposed, otherwise one conservative provider-wide pool. User aliases label
pools but cannot prove separate subscriptions. Scope-local Usage telemetry remains
consumed usage, never remaining subscription capacity. Account switch notification
invalidates stale windows/leases; do not infer identity by reading login stores.

Normalize native data to `QuotaObservation {pool, bucket, window_id, status,
used_percent?, resets_at?, observed_at, source_version, signal_confidence}`.
Persist only necessary nonsecret metadata. Percentage is a scheduling heuristic,
not a computable token balance. All relevant known buckets must permit admission;
out-of-order prior-window events cannot clear newer exhaustion. Same-connection
sequence and window/reset identity are validated; ambiguous newer connection data
marks stale and requests fresh official metadata before reopening an exhausted pool.

Codex: use supported account/rateLimits/read and account/rateLimits/updated schema,
where the installed CLI exposes them; parse multi-bucket optional windows. A
documented UsageLimitExceeded category is an investigation input, not an assumed
wire spelling. Pin observed schema/error types for supported versions in Phase2.
Do not route account-wide events through a fabricated Task/native-turn identity.

Claude: retain structured system/api_retry events (attempt, retry delay/status)
as retry telemetry. An HTTP 429 alone is transient throttling, not subscription
proof. Decode structured rate-limit/terminal subscription signals only in a
version-qualified manifest with fixtures; unknown shape stays unclassified.
Remaining windows may remain unknown. No statusline hook installation/config edit
or credential scraping. Phase3 must establish actual exhaustion/wait/resume, or
that row remains unsupported/unverified in published behavior.

Confirmed exhaustion closes pool admission, marks affected pending/active units
WaitingQuota and persists reset/recovery policy. Preserve a still-retrying native
turn and its current quota lease. It may complete work; known completion is not
discarded by quota telemetry. If the turn is terminal, retire its authority and
lease, preserve drafts, schedule a fresh attempt after capacity returns. Never
resume/reuse its worktree. Copy a draft only by capturing its explicit commit and
selecting that retained SHA as a new base; native resume/checkpoint support must
also pass authority/CWD compatibility, otherwise launch a new native session.

For the staged Claude producer, a recognized rejected plan-window event must
match the owned native Session and remain exhausted in the accepted pool state
before moving that exact live unit to WaitingQuota. Publish the same authority
and wait reason to its watch as its durable status. Retain the current native
input and quota lease; this notification neither sends another prompt nor changes
a sibling's execution state. An allowed event for another bucket cannot erase
the wait. Unknown windows and foreign Session notifications do not mint confirmed
subscription waiting. Telemetry-only waiting does not grant a recovery-probe
lease: an unsolicited allowed event cannot reopen the same exhausted window.
Codex's owned `willRetry=true` subscription error separately establishes a native
retry; a scheduler-admitted recovery probe retains its existing rights. When
every participating plan window is Available in the accepted ledger, revalidate
the same live unit/generation/epoch/Session and active lease transactionally,
return it to Running and clear the wait in the watch/status. Workflow polling
then restores the active phase's Task state without creating another attempt.
Unrelated windows, ignored stale recovery and unknown data cannot clear the wait.
Neither recovery nor waiting resends input or releases the live lease. A reliable
successful result still closes the live wait
with successful work, even if pool telemetry remains exhausted. Native retry
behavior and real quota recovery remain Phase3 qualification, not fixture proof.

With Unknown/Stale balance, default per-provider concurrency ceiling is three
including reviewers, executor sub-ceiling two. Global policy reserves reviewer
slots and uses fair queues; two Claude plus two Codex executors can overlap if
configured capacity permits, with later reviewer progression. No promise of exact
quota reservation. Known near-exhaustion delays executors first using configurable
headroom; mandatory reviewer identity/quorum is never changed to save quota.
Reviewers get age-based priority within the finite headroom. Policy changes are
audited and cannot revive a cancelled waiter.

Reported reset schedules one pool-wide refresh with jitter. Unknown reset uses
bounded exponential cooldown (initial 60 seconds, cap 30 minutes) and one probe
lease per pool, never each Task polling independently. Prefer metadata-only official
queries where supported; otherwise admit one explicit bounded native recovery
attempt after cooldown, preserving its work/resources. Still-running retrying
turns do not receive duplicate input. Manual resume requests a recheck subject to
cooldown; it does not erase exhaustion or authorize paid API billing. Auth errors,
unsupported metadata and failed probes remain named, not fabricated capacity.
Unclassified terminal limit-like errors record work unknown and `WaitingCapacity`
with bounded diagnostic/recheck; they are not labelled confirmed subscription
exhaustion or successful work. Definitive unrelated native failures remain failures.

## 9. Best-effort cleanup and OS boundary

Cancellation/replacement retires both logical permissions before cleanup. Natural
completion closes only native-effect permission; Runtime result-finalization
remains until capture/publication or durable draft disposition (§2/5). Cleanup
jobs have explicit dependencies: process stop/observations may run immediately,
but worktree/branch/input deletion and dependent path release wait for finalization
closure plus the draft retention policy. The janitor cannot close finalization
or delete an input merely to make reclamation succeed. First use owned
native cancel and safe unreaped process-group/direct-child stop/reap, then bounded
cookie discovery, optional exact user-scope stop and managed Docker deletion.
Natural native completion may precede reclamation. Keep owned child reaping and
bounded output drain; never convert known work to Lost solely for cleanup failure.
No signal-all, daemon-global kill, Docker prune or user-wide process scan logging.

Add independent `rrx-process-tracker` crate only if needed for safe OS wrappers;
it must not depend on rrx or SQLite. Expose typed process identity, limited cookie
match results, coverage and fallible signaling. The main crate keeps unsafe forbid;
any FFI unsafe is narrowly documented in that crate with its own audited lint
configuration, not a workspace-wide relaxation. Public discovery never returns
foreign environment/argv contents, credentials, or an authority minted from a PID.

Linux reads accessible same-user /proc identity/environment for cookie matching,
uses pidfd where supported for exact discovered process signaling, and records
denied/inaccessible/exited scans as coverage limits. macOS uses public libproc birth
identity and accessible KERN_PROCARGS2 cookie data; environment may be omitted for
restricted binaries. Do not request private entitlement/SIP changes. Filter only
the cookie field in temporary bounded buffers; discard all other bytes without
persisting/interpreting credentials. A scan is not proof of complete ownership.
Birth identity is rechecked; where atomic process-handle signaling is unavailable,
automatic discovered-PID kill is disabled rather than risking a reused unrelated
PID. Report those cookie matches as leftovers/unknown and still use safely owned
unreaped children/groups. This conservative fallback is explicitly tested on macOS;
the user request's cookie termination is attempted only when safe identity allows.

Linux `systemd-run --user --scope` is optional and version-qualified. Register a
unique unit name/identity before invoking; preserve caller env/CWD/stdio and disable
command argument variable expansion where the supported version permits. A scope
creation attempt cannot fall back by blindly spawning again after an ambiguous
acknowledgement: reconcile first. Failed-before-spawn capability probe permits
normal direct-child fallback. Stop only exact recorded scope/Invocation identity;
record cgroup observation as optional coverage, not general custody. Do not enable
lingering, install services or assume a user manager on CI/containers.

Docker cleanup enumerates exact namespace labels and verifies each ID/operation;
only that unit's nonshared containers/networks/volumes are targets. Creation
pending at crash reconciles by deterministic identity. Unavailable daemon/context
is unknown, not empty. Do not read Docker auth files or cache auth headers.
Leases for unconfirmed live ports/resources stay quarantined. Worktrees are never
reused; dirty drafts are retained. Cleanup retries have bounded work budgets and
backoff independent of Workflow progress. Explicit operator reconciliation can
release a resource with an audited coverage exception, not declare workload death.

Aggregate cleanup is leftovers if a registered survivor is observed, unknown if
any required recorded-resource observation is unavailable/unresolved, otherwise
reclaimed **for observed tracked resources only**. Store per-resource coverage so
an observation of leftover plus unknown is not hidden by the aggregate. Old jobs
cannot free a new unit's lease or amend work/evidence. Runtime SIGKILL stops its
janitor; restart continues ledger reconciliation. Cleanup continuity is not claimed.

## 10. Workflow and source replacement map

The exact Phase0 [refusal inventory](../verification/agent-execution-phase0-refusals.md)
and [invariants](../verification/agent-execution-phase0-invariants.md) remain the
source-level change checklist. Baseline file paths below are under `crates/rrx/src`;
Claude paths refer to the unmerged candidate, not current main.

| Existing path / consumer | Phase2 change / guards retained |
| --- | --- |
| codex/availability.rs; preparation/transport/RPC/input/permission/checkpoint gates | Replace ownership prerequisite with native conformance and ExecutionAuthority/resource/artifact admission; retain operation capabilities and exact permission framing. |
| codex/session.rs native terminal and stop; Claude session.rs terminal/stop | Preserve reliable native work independently of cleanup, normalize quotas and publish scoped historical cleanup; never invent completion from transport loss. Integrate Claude candidate through its reviewed source boundary. |
| adapter.rs ProcessGroup/supervisor/Reservation Drop; process observation helpers | Retain owned child lifecycle/output bounds; cleanup-only uncertainty produces cleanup unknown, actual unknown work stays unknown. Freeze ps observer rather than extend it. |
| state/mod.rs Session admission/put_workflow_transition/terminal recovery | Add unit-aware atomic APIs; replace Task-wide Lost exclusion and single historical binding with generation/current projection. Retain scope, Project/Goal versions, CAS, unknown external dispatch and concurrent same-unit writer guards. |
| domain.rs Session/Task/Record and adapter.rs LaunchRequest/PreparedInput/SessionRef/SessionStatus | Add typed unit/artifact identity and independent outcomes/wait reasons; migration defaults, strict serialization and no wildcard identity. Consumption Usage remains unchanged in meaning. |
| git.rs create/status/review lock/cleanup/validate_worktree_ownership | Fresh attempt paths, provenance-specific snapshot validation, actual SHA capture. Replace live executor freeze/death requirement; retain protected-base, canonical identity, safe ref CAS and unknown common lock refusal. |
| workflow.rs retry/native dispatch/recovery/source capture/evidence/Cleanup | Allocate fresh attempts; separate quota/resource wait and cleanup backlog; source/evidence ports consume ArtifactRef snapshot. Preserve pending-operation idempotency, Workflow phase policy and exact dependency versions. |
| context.rs readers/cache/ContextPack/ReviewBundle | Read committed snapshot inputs; cache keys include artifact/rule/dependency versions. Preserve bounds/mandatory rules; reader cleanup diagnostics never invent source completeness or accepted work. Frozen #60 readers remain bounded utility code, not global ownership authority. |
| main.rs + Runtime integration | Add coherent run/status/stop/resume/logs/doctor and hidden TaskTool entry. Scheduler/account state is shared across Projects; do not just advertise execute on an unavailable adapter. |
| Goal evaluation / Review Engine / Approval Broker | Goal success still requires workflow evidence; quota waiting is not failure. Requests bind unit/generation/artifact; retired approvals cannot grant new effects. No self-review/quorum weakening. |

Schema/API changes are one coordinated STRICT delivery; no intermediate public
profile enables native execution before R2–R4 are operative. Linux implementation
then macOS does not imply Linux-only release meets the requested two-OS acceptance.
Config/public profile must expose the supported forms and reasons; hidden test
capabilities cannot stand in for production support.

### 10.1. Initial source preparation and first native adoption

The production source port must break the initial dependency cycle explicitly:
Workflow needs a full input revision and repository context before native phase
reservation, while every Git helper needs an already registered execution unit.
Runtime prepares one fresh Executor before `WorkflowEngine::initialize`, using a
reserved preparation-only phase `workflow_source_bootstrap`. The existing attempt
allocator journals the unit, generation, namespace and resources before resolving
the configured base branch through UnitGit. The result remains Preparing, with
no Session, native input, work result or result artifact. Namespace preparation
is not successful Agent work and must not publish a fabricated result artifact.

Only the actual preparation producer returns a private, non-serializable,
non-Clone prepared-executor capability. It retains its abandonment guard through
initial source capture, Workflow initialization and the initial non-native Issue
or Worktree evidence phase. Runtime prepares before initialization reads its Task
snapshot, so registration's Task projection change is not an accidental Context
CAS conflict. Production sources read the exact base commit's tree/blob objects
through registered, bounded UnitGit reads; a SHA attached to a live filesystem
inventory is insufficient. Repository-map selection retains mandatory rules,
scope/version identity, complete inventory bounds and explicit skipped content.
Legacy context readers and #46/#60 ps observation are not extended as a fallback.
This also replaces the separate tracked rule/config ingress in `Workflow.inputs`
and `prepare_pack`: the prepared committed-source frame supplies their exact
bytes, hashes and parsed policy from the same base commit, instead of the legacy
live `load_rules` producer. Sources.capture alone cannot replace that consumer,
which currently forbids `rules:` keys and independently prepends live rules.
Explicitly declared supported external noncredential rule references retain
their separate scoped, hashed dependency policy; they are not falsely described
as committed objects. First adoption validates the complete prepared source/rule
frame against its Context; unsupported origins refuse without a live fallback.

At the first real Executor phase (Implement for Quick, Requirements for
Standard/Strict), the capability revalidates the actual prepared namespace and
source revision and invokes an Immediate adoption transaction. It checks exact
unit/version/epoch/generation, active Task projection, provider, profile, path,
branch, base, Preparing/Active, open permissions, absent Session/work/artifact,
and absence of unresolved preparation helpers, native input and quota admission.
The existing full Workflow reservation checks are shared: current Project/Goal
versions and accepted governing digest, Task CAS/scope/lifecycle, sole Workflow
record/version, active index/generation, undispatched Executor phase, exact Context
version and immutable base. Only the preparation phase and the active history's
unit binding change, with unit/record versions and an audit event. Generation,
worktree and resource identity stay the same. Failure rolls back both changes;
abandonment retires only the exact owned preparation. A Task hint or recovered row
cannot recreate this capability. Later Executor phases use genuinely new attempts;
their source port reads a genuine retained artifact without reopening old native
or finalization authority. Reviewer/verifier snapshots remain independent.

Native start rejects the preparation-only phase before version probes, quota or
Session effects; Session registration independently rejects it inside its actual
transaction. Public ordinary preparation cannot request the reserved phase.
The persisted grant restriction changes the writer contract: schema 5 replaces
the schema-4 connection-local write guards with contract 5 guards in one migration,
without changing execution table layout. Fresh databases install contract 5.
Already-open schema-4 writers, including ones with the old function registered,
must fail writes after migration; old binaries must reject reopening schema 5.
Migration does not fabricate preparation provenance or adopt existing units.

Controls must exercise actual consumers: registered Git before bootstrap effects;
no native version/Session/input before adoption; same-ID/generation/path first
adoption; atomic refusal on Context/Workflow/Task/provider/epoch drift, unresolved
helpers and repeat adoption; abandonment/restart retirement; old-writer guard
failure and current-writer success. Removing each decisive grant/CAS guard must
make its consumer control fail. This contract is a Phase2 implementation plan,
not evidence of production source ports or native compatibility.
An actual initialize/prepare_pack control must distinguish tracked rule/config A
at the base commit from live Project bytes B: payload, rule hashes and selected
policy use A or explicitly refuse, never silently use B. Omitting the rule-frame
wiring while retaining the committed repository index must fail this control.

## 11. CLI, observability and verification plan

`rrx run` resolves registered Goal/Task workflow and starts the owner scheduler;
`rrx status` reports unit, exact accepted SHA, waiting reason, work and cleanup
axes; `rrx stop` performs logical cancellation then enqueues reclamation;
`rrx resume` rechecks account/resource/external gates and creates fresh attempts
where needed; `rrx logs` exposes bounded/sanitized event metadata, not credential
environment dumps. `rrx doctor` reports native version/profile support, result
storage/path capability, resource forms and optional scope/cookie visibility with
reasons. It does not log in, read credentials or claim an atomic cleanup Tier.
Explicit authenticated metadata refresh is a separate native action, not doctor.

Crash control points and required verification:

| Control point | Expected persisted/recovered behavior |
| --- | --- |
| Resource intent before directory/tool spawn | Reserved names survive; reconcile actual effect before repeated creation. |
| Native spawn before dispatch acknowledgement | Old work unknown/fenced, cleanup backlog; no borrowed PID/grant or duplicate external replay. |
| Artifact files/refs ready before DB publication | Staging/ready draft, verify content; fenced artifact cannot automatically advance Task. |
| Cancellation races publish/grant/helper admission | One generation-CAS winner; losing authority cannot affect newer/sibling units. Already issued effects reconcile historically. |
| Tool create acknowledged lost / Docker daemon unavailable | Deterministic identity resolves or unknown remains; never issue global cleanup or recreate blindly. |
| Runtime SIGKILL after accepted evidence | Retained SHA/manifest and review content survive restart; active native sessions unknown and fresh attempt policy used. |
| Quota exhaustion/reset/out-of-order notification | Shared pool waiting, no work failure, bounded one-probe recovery, no stale event clearing new exhaustion. |
| Normal completion with concurrent janitor | Process cleanup cannot revoke result-finalization permission or delete capture input; result publish succeeds unless cancellation/replacement wins its CAS. |
| Schema migration with pre-open old Store | Old cached/fresh writes reject at DB guard; new writer succeeds; no Task/Session authority corruption. |
| Snapshot mutate-consume-restore | Supported ordinary source writes fail before consumption; output writes succeed. Explicit permission/raw bypass is uncovered, never passed as input immutability proof. |
| Orphan namespace and pending cleanup | New namespace proceeds subject to finite capacity; old leases not immediately recycled. |

Phase2 verifies state/API migration, effect receipts, causal fault-injection and
cross-unit races at committed source, supported version protocol negative controls,
resource policies and required default-concurrency checks on both OSes. No fixture
is described as actual authenticated acceptance. Phase3 runs the requirements
matrix with actual logged-in Agents and at least four overlapping Tasks per OS,
records the cancel and Runtime-SIGKILL cases separately, and compares exact commit,
review/input and evidence hashes. macOS setsid unavailable forms are named skips;
portable detached fixtures and actual Docker availability are recorded. Quota
fixtures cannot replace an actual quota exhaustion/resume observation.

GitHub-hosted Linux/macOS jobs can cover filesystem/DB/process APIs and injected
protocols; user-systemd manager, Docker daemon, build tools and provider accounts
are conditional capabilities, not assumed services/secrets. Actual user-auth/hook
compatibility must be run in an authorized controlled host environment. Failure,
skip, unsupported and unverified rows remain visible. README Status changes only
after Phase3 to reflect observed capability/limits; licenses stay untouched.

## 12. Primary reference boundaries

References were inspected on 2026-10-05 JST. They support mechanisms, not the
unimplemented profile's native acceptance:

- [Git clone](https://git-scm.com/docs/git-clone): local hardlinks/alternates can
  retain source-storage dependence; use independent object transfer for results.
- [Docker run labels](https://docs.docker.com/reference/cli/docker/container/run/):
  labels require explicit flags; env alone does not label a detached container.
- [Gradle Daemon](https://docs.gradle.org/current/userguide/gradle_daemon.html):
  --no-daemon can still create a single-use JVM; private registry matters.
- [Bazel output layout](https://bazel.build/remote/output-directories): startup
  output roots/base can be selected per execution unit.
- [sccache configuration](https://github.com/mozilla/sccache/blob/main/docs/Configuration.md):
  cache/server settings need a version-qualified private profile.
- [SSH config](https://man.openbsd.org/ssh_config), [tmux](https://man.openbsd.org/tmux):
  explicit control/socket selection; no inference of sibling-safe teardown.
- [systemd-run source manual](https://github.com/systemd/systemd/blob/main/man/systemd-run.xml):
  user scope inherits caller environment, command expansion/version must be handled.
- [Apple process metadata](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/proc_info.h)
  and [procargs implementation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_sysctl.c):
  birth timestamps exist; environment visibility may be restricted. No atomic
  unprivileged discovered-process signal guarantee is inferred.
- [Codex app-server](https://learn.chatgpt.com/docs/app-server),
  [Claude headless](https://code.claude.com/docs/en/headless): native schema/version
  conformance and actual account behavior need separate evidence.
- [SQLite connection-local functions](https://www.sqlite.org/c3ref/create_function.html),
  [schema-change statement recompilation](https://www.sqlite.org/c3ref/prepare.html),
  [triggers](https://sqlite.org/lang_createtrigger.html) and
  [transaction isolation](https://sqlite.org/isolation.html): support the proposed
  old-writer guard; its actual bundled-rusqlite behavior remains a Phase2 test gate.
