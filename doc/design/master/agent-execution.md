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
Session/native turn, artifact and record versions. Cleanup uncertainty retires
logical write/grant authority without turning a known native result into failure.
Unknown external effects remain subject to idempotent reconciliation.

Each retry gets a new executor worktree, branch and resources. Accepted commit
objects and hashed evidence are retained in independent Runtime storage. Each
reviewer/verifier receives a separate exact-SHA snapshot, never authoritative
reads from a changing executor worktree. Publication selects accepted SHA with
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

Phase1 consists of independently reviewed requirements/design only. Phase2
implements a coordinated profile without an unsafe intermediate launch path.
Phase3 validates actual four-Task native runs, sibling cancellation, crash/restart,
detached helpers, quota waiting/recovery and auth/settings/hooks on both OSes.
Hosted CI fixtures and controlled authenticated runs are labelled separately.
README Status changes only after Phase3; unverified behavior stays unverified.
