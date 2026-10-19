# Native Agent Result Protection Design

## Process launch boundary

All std/Tokio Command spawn/output/status paths consume their configured Command
through `rrx-spawn-hygiene`. On macOS the boundary marks inherited fd >= 3
CLOEXEC before exec, preserving stdio and std's error pipe; it validates a
complete aligned libproc list and refuses malformed/full lists or fcntl failures
without fallback. On Linux it delegates to the original operation without an
additional hygiene pre_exec callback and provides no descriptor hygiene: children
inherit every descriptor not marked CLOEXEC, as with the original operation.
Other targets refuse launch with `ENOSYS`. The rrx crate continues to forbid
unsafe code.

`RuntimeOwner::open_mode` selects the execution mode; `open` retains the native
default. Under the exclusive owner lock, startup reads the typed opposite-mode
inventory before `begin_execution_epoch`. Busy inventories refuse startup.
Cookie discovery requires every supplied cookie to have complete tracker coverage
and zero matches; errors and excessive inventories refuse. Epoch advancement is
never death evidence. Successful pipeline startup records the pre-epoch
inventory/discovery summary as `mode_quiescence` under its new epoch.
Its writer consumes a sealed receipt constructed only by `check_mode_inventory`;
an arbitrary JSON body cannot authorize a qualified quiescence observation.

Pipeline Q reuses `execution::process::OwnedProcess` and `bounded_read`: spawn
hygiene, unreaped identity, group signaling before reap and direct-child fallback.
Each probe starts its five-second observation deadline immediately after spawn
on a blocking worker, capped by a 35-second overall
CLI qualification observation deadline covering image/config reads and all probes,
with four KiB combined output per probe. Nonblocking open and descriptor
checks refuse special files. Probe-only macOS EPERM resolution accepts only a
confirmed all-dead group inspection; live/unknown/failed inspection refuses Q.
The blocking worker's JoinHandle and unreaped custody remain retained when the
deadline fires. Cleanup result and direct-child reap precede qualification
publication or another probe. Shared `signal_group`, `stop_and_reap` and native
capture bounds are unchanged. No Store mutex/transaction or control guard spans
a probe; the owner flock remains held for the Runtime lifetime.

Pipeline claims and launch intents require a `CliPipeline` owner and all four
qualified current-epoch subjects. Idle sweeps check DB rows and config digest;
`CliQualifier::before_launch` re-hashes both images immediately before launch,
outside Store transactions, between tx1 and spawn. PATH prefixes and every parent
must be root/current-user owned with no group/world write permission and are
rechecked there. PATH entries and executable paths (including canonical targets)
must not overlap work root, CODEX_HOME, non-Removed Projects or the state directory.
Work root and CODEX_HOME must be canonical, user-owned 0700
directories separate from each other, state and Projects. Project registration
and re-registration also reject overlaps before publication. Other Codex layers
(managed/system config, AGENTS.md and rules) are not inspected and remain an
operator-controlled residual risk. Probe cookies are not recorded; escaped
probe descendants are outside the durable mode cookie inventory.

The pipeline sweep has no step executor and never enters native reconciliation
or admission. A stale Q ends the sweep and appears as
`pipeline_qualification_error` in RuntimeStatus. The pipeline CreateGoal control
returns typed `PipelineGoalAcceptanceUnavailable` (pipeline Goal acceptance is
not available yet) without publishing pipeline Goals.
Native S is unchanged.

The six functions delegate to their corresponding original methods. Tokio
output/status spawn synchronously before returning the future. Stdio defaults,
cancellation, kill_on_drop, process group, original locks and immediate Child
adoption retain their meaning. The macOS boundary proves descriptor hygiene rather
than process death. Tokio errors during post-spawn registration can still leave
a child; existing uncertainty and custody rules remain applicable.

The private registration source consumes the same retained Prepared object and
its known quota readiness/Unit lineage. Registration and its distinct transport
intent share an exact Immediate transaction. A preallocated Candidate retains
the plan and activates once from that plan's known commit acknowledgement;
revocation cannot reopen it. Registered dispatch, ACK, projection and terminal
currency use the original Source/frame and current Driver conjunction. Public
Native composition and capability advertisement remain refused on public standalone
paths. The unmerged Runtime source installs one managed graph and connects the
original first Executor to its retained Prepared and physical transport. Genuine
registration lifecycle controls, transport custody and independent source review remain
unverified; these source primitives do not establish Native availability.
Registered Codex retry/probe and Claude wait/recovery callbacks use the SAME Live
actor and exact active own lease through bounded private snapshot/Immediate plans.
Their window and pool images are compared before writes; cooldown and recovery
conditions match the existing quota routes. Genuine registered quota lifecycle
qualification remains unverified.

Status: proposed Phase1 policy. Production behavior is unchanged until the
coordinated STRICT implementation is reviewed and delivered. This document is
not native acceptance or permission to begin the next phase.

## 1. Policy precedence

The latest native Agent request selects result protection plus best-effort cleanup.
For the implemented new execution profile, the
[requirements](../../requirements/agent-execution-requirements.md) and
[detailed design](../agent-execution-design.md) supersede conflicting process-death,
Lost-reservation and live-worktree-freeze prerequisites in issue-4/46/60 and the
other master documents. Their descriptions of current/historical implementation
remain accurate for that baseline. No old source/native acceptance is implied.

rururunx is not a security sandbox. Official logged-in Claude Code/Codex run on
the host with their existing auth/settings/hooks and their own permission policy.
No VM/containerized Agent, separate user, root service or outer sandbox is used.
No credential inspection, copying or transfer is introduced.

## 2. Authority and durable results

The CLI pipeline launches through the Runtime's `control_admission` boundary.
Inside that boundary, launch checks service/stopping state; its intent transaction
checks current owner/epoch, independent run and step versions, a Running Goal,
current qualification, native exhaustion and the daemon-wide process and provider
wait gates. `StepLauncher` releases the Store mutex before directory work,
qualification rechecks and `rrx_spawn_hygiene::spawn_tokio`. It records an intent
before spawning, then records the owned leader's PID, equal PGID and tracker birth
before sending input. Spawn errors become unknown; only pre-spawn failures can
be launch rejections. Failed running receipts retain launching unless death is
proved, in which case the exit records `killed_after_record_failure`.

Process steps count globally across Projects, including unknown and held without
confirmed death. Prepare and release do not count. Native Exhausted observations
block launches until native observation replacement; elapsed time cannot clear
them. One durable provider-wait row gates all Agent providers and Projects, while
test steps are exempt. A due wait permits one probe; only its classified known
success clears the wait. Other probe outcomes extend backoff. The Runtime's
post-exit limit transition also takes `control_admission` and rechecks stopping;
the Store revalidates Running Goal, epoch and both versions in its final tx.

Workflow authority binds Scope, execution unit, Task generation, Runtime epoch,
Session/native turn, artifact and record versions. Natural completion closes Native
effect/grant authority while Runtime result-finalization remains open. Cancellation
or replacement closes both; the janitor cannot revoke finalization or dispose
its inputs. Cleanup uncertainty does not turn known native work into failure.
Unknown external effects remain subject to idempotent reconciliation.

Each retry gets a new executor worktree, branch and resources. Accepted commit
objects and hashed evidence are retained in independent Runtime storage. Each
reviewer/verifier receives a separate exact-SHA snapshot with read-only source and
separate output under a qualified tool profile, never authoritative reads from a
changing executor worktree. Before/after hashes alone do not prove absence of
transient mutation; same-user permission/raw bypasses remain uncovered. Required
incompatible source-writing hooks/tools refuse that profile. Publication selects accepted SHA with
expected-old-target validation. Quorum, rule completeness, permissions and review
findings verification remain required.

## 3. Scheduler and resource policy

Unit leases cover ports, temp/output/cache paths, Docker identities and supported
tool daemon/control endpoints. Managed invocation profiles enforce labeling and
private/disabled daemon behavior for their supported forms. They do not intercept
arbitrary host commands. Native HOME/provider roots stay unchanged; unknown
overrides/raw sockets/shared absolute paths are uncovered or declared exceptions.
Common Git mutations are serialized and automatic maintenance/fsmonitor disabled
for managed commands without disabling required hooks. Unknown shared locks are
not deleted based on age; independent retained results remain usable.

Subscription pools are provider/account/bucket resources separate from Usage and
concurrency. Confirmed exhaustion is waiting, not work failure. Unknown remaining
capacity uses bounded explicit concurrency/backoff, not invented balances or
ownership-style blanket refusal. Mandatory reviewer policy is preserved; known
capacity, fairness and review headroom influence admission.

An owned Claude plan-window exhaustion updates both its live unit's durable wait
reason and watch status after the pool accepts it. Native input/lease are retained;
no prompt is resent and sibling unit state is unchanged. Foreign/unknown events
do not qualify that wait. Telemetry-only waiting does not mint recovery-probe
authority or reopen exhaustion on an unsolicited same-window allowed event.
Accepted recovery of every participating plan window returns the same unit and
watch to Running/no wait under current authority and lease checks; Workflow
restores its phase state without restarting input. Reliable native success
remains success. This staged
producer policy is covered by local protocol controls, not actual account recovery.

## 4. Cleanup and outcome reporting

CLI pipeline `StepCustody` concurrently handles stdin, bounded stdout/stderr,
leader exit, deadlines and stop notifications after releasing the control boundary.
Captured evidence is retained as private read-only files and read back with its
length and SHA-256 before provider classification. Stdout overflow rejects the
result after death confirmation; stderr retains its bounded tail and records
truncation. Observation writes never take `control_admission`, so a controller can
wait for custody while holding that boundary.

`adapter::ProcessGroup` keeps the leader unreaped through group cleanup for native
adapters and Codex preparation/transport. Pipeline `StepCustody` wraps its existing
observe/cleanup/reap API. Pipeline settlement additionally requires observed leader
exit, delivered group cleanup, leader reap, observed empty group and complete cookie discovery
with zero matches. A deadline cannot supply any of those facts. Incomplete proof
holds the running step; monitor failure records unknown. No discovered PID is
signalled on macOS.

Work success/failure/unknown and cleanup reclaimed/leftovers/unknown are independent
axes. Reclaimed describes observed tracked resources at a time, not whole workload
absence. Owned child/group stop, safe cookie discovery, optional Linux user scope
and exact Docker label cleanup are best effort. Unsafe discovered-PID signaling
is skipped when atomic identity cannot be maintained; it is reported honestly.
No ps-observation extension, containment Tier or cleanup continuity during Runtime
SIGKILL is claimed. Restart fences old authority and reconciles durable intents.

## 5. Integration and evidence

Session remains transport state; Task/attempt is execution authority; reservation
is unit-specific; Workflow still owns phase truth; Context/ReviewBundle bind actual
snapshot content; Review Engine/Approval Broker bind artifact and generation;
Goal completion depends on accepted workflow evidence, not native exit zero.
SQLite migration and all native/public API changes receive STRICT treatment.

Historical retained-result inspection uses the current Runtime epoch and an exact
ledger artifact snapshot, independently of the producing Task's retired generation
or closed permissions. Each finite read-only Git command has a durable scoped
intent before spawn, an independent operation cookie, bounded output/wait and
epoch/artifact revalidation before its receipt. It never restores native or
finalization authority or requires historical executor/profile/temp paths. It
resolves the current Runtime's canonical Git executable; a receipt hashes that
path reference, not binary contents or a qualified version. Missing Git refuses
inspection. Inspection failures do not alter accepted work or cleanup results.
Internal Git commands clear inherited common-directory/object/namespace/ancestry
routing and disable lazy fetch/replacement objects. Retained repositories refuse
canonical-path aliases, external common/alternate stores and shallow/graft metadata;
otherwise matching refs plus successful traversal could hide missing ancestry.

Phase1 consists of independently reviewed requirements/design only. Phase2
implements a coordinated profile without an unsafe intermediate launch path.
Phase3 validates actual four-Task native runs, sibling cancellation, crash/restart,
detached helpers, quota waiting/recovery and auth/settings/hooks on both OSes.
Hosted CI fixtures and controlled authenticated runs are labelled separately.
README Status changes only after Phase3; unverified behavior stays unverified.

Initial production context uses a preparation-only Executor registered before
base-resolution Git, with a private live ownership capability retained until the
first native Workflow phase. Its committed-object input does not invent Agent
success. Exact atomic adoption binds the same unit/generation/namespace to that
first phase; public native start and Session registration refuse the preparation
phase before effects. Source ports for subsequent phases consume genuine retained
artifacts. The prepared frame also feeds Workflow's actual mandatory rule/config consumer
from the same commit; declared external rule dependencies remain scoped and
hashed separately, with no legacy live tracked-rule fallback.
Schema 5 updates connection-local writer guards so already-open
contract-4 writers cannot reinterpret the preparation-only mode. See detailed
design section 10.1; this is a staged implementation contract, not shipped CLI or
actual native acceptance evidence.

## 6. Staged Docker command profile

The Phase2 component candidate supports managed `run`/`create` and filtered `ps`
through the installed rrx tool entry. It is not qualified with an actual Docker
daemon yet. It selects client/server 28 with API 1.48–1.51 and uses API 1.48 for
managed commands. Docker's [API matrix](https://docs.docker.com/reference/api/engine/)
documents those 28.x API versions and the effect of `DOCKER_API_VERSION`; this is
a protocol selection, not evidence that every patch or template has run. Other
versions, remote transports, relative configuration roots, `DOCKER_HOST` and raw
Compose profiles refuse this managed path before creation. They are explicit
coverage limits, not native Agent availability or a requirement to install Docker.

Runtime probes, the actual shim and historical cleanup bind the same canonical
Docker configuration-directory digest, explicit context name and observed engine
digest. The shim transmits a directory-reference digest and host-override flag;
the Runtime checks these before probes, and the shim rechecks the granted reference
before spawn. Credentials/configuration files are not read, copied or substituted.
Recorded context names alone cannot establish the actual engine. Inaccessible or
changed references refuse; the ordinary inherited authentication context remains
the CLI's responsibility.

Cleanup verifies four namespace labels plus recorded creation operation and name
before a full-ID kill, reinspects identity/running state and removes non-forcibly.
Each mutation intent retains its exact container and engine digest before spawn;
the typed attempted action exists before waiting for acknowledgement. Timeouts
retain unknown acknowledgement and confirmation for that exact action. Inventories
can change: the Docker report reserves 128 remaining identities across the first
and final container inventories, networks and volumes; cookie observations reserve
the other 896 identities of the persisted 1024 bound. Partial reports enforce the
same cap and record limited coverage rather than complete absence.

Unexpected matching networks/volumes are reported, not deleted without registered
creation/attachment evidence. Image-declared anonymous volumes, raw Docker API
delegation and unsupported Compose are not fully tracked by this finite profile.
No global prune, forced removal, complete workload collection or resource-release
claim follows from an empty list. Worktrees/ports remain retained or quarantined;
operational Runtime integration and actual OS/Docker/native qualification are pending.

The managed Workflow input consumer receives committed policy, rules and repository
selection in one private frame; it bypasses the legacy live rule loader. Complete
inventory is retained in the pure index and represented by a digest in the bounded
Workflow/artifact authority map. First-unit adoption checks both this frame and the
actual complete Context envelope. Cached frames refuse accepted Project/Goal
instruction drift even after preparation ownership is consumed. Finite retained
inspection can read its exact commit tree and the bounded ordinary blobs observed
in that tree. Automatic changed-input recovery and full Runtime gate/CLI wiring
remain separate work; component controls do not qualify native compatibility.


The selected first Executor preparation has a private 13-action readonly Git
continuation after its genuine version observation. It preserves inherited native
settings and credentials, queries only seven fixed noncredential config keys, and
qualifies all effective check-in conversion attributes before Status can run.
Clean/process filters, text/eol/crlf, ident and working-tree-encoding refuse when
the fixed pathspec set differs from the retained Source inventory. Index flags,
mode/type, sparse/bare/object format and raw-content conversion mismatches refuse.
Successful correspondence remains a retained nongrant fact. The continuation
retains installed compatibility, one checked nongrant command, a no-dispatch
value and private pre-Session quota plans. Quota parking retains the SAME input,
helpers and operation; own known postimages supply due-claim lineage. Revoked
closure never refreshes that lineage from rows. Revoked preparation bookkeeping
has a synchronous one-transaction close/confirm step: confirmed rollback clears
the saved preparation plan before a later fresh close attempt. Transport custody
consumption clears the pre-dispatch witness. The sealed ended-start closure
interface retains the same actor, no-dispatch and known preparation closure;
the Root sweep consumes only its same-job ended-start witness. It runs at most
eight due closure turns per bounded registry pass, saves the first unserved due
job in a rotation cursor, and keeps uncertain plans unchanged across confirmation.
Typed known acknowledgments alone retire the original publishing slot and
finished job. These private interfaces do not qualify Native execution or
composition; genuine activation and normal ended-start controls remain pending.
Private Prepared issuance checks
the SAME allocation's fixed provider/role budget at version, Git, Prepared and
registration. Private quota uses two prebuilt branches and at most eight exact
writer-connected Legacy validations before the own fair position, with malformed
owner data or an unresolved ninth call held without a write. Public standalone Session/transport composition and Native capability
advertisement remain unavailable. The Runtime-only first Executor connection is
unmerged and unqualified; continuation and Root stop integration remain absent. Component source
and primitive real-Git controls do not qualify the actual installed actor chain,
Git version, authenticated Agent, both-host operation or four-Task acceptance.

The configured account-free Runtime control reaches a genuine Driver, real Git
Source preparation, initial Worktree gate and original first Executor reservation.
It then remains Held at marker planning: the exact composed
`workflow_native_contracts` predicate has no production activation producer.
Prepared issuance, registration, spawn and record-only binding are therefore
SETUP/UNVERIFIED on this installed lane. No SQL activation or predicate bypass is
provided by the source increment.

On the #43 branch, an owned Native success terminal seals its committed terminal
images for the success continuation, which captures results only through a
protected settled currency. A Native owner writer refused for an unlinked plan
re-plans exactly once when a fresh successor holds only the SAME operation's
first `session_bound` link and the identical Unit; terminal planning is never
retried. Post-bind effects and the owned terminal are reached on the
commit-mode fixture lane, which does not qualify official Native execution.
