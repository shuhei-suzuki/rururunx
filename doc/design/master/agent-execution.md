# Native Agent Result Protection Design

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

## 4. Cleanup and outcome reporting

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
