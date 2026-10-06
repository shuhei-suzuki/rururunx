# Issue 75: Execution Domain requirements

**Status:** Proposed normative requirements  
**Workflow:** STRICT  
**Issue:** #75  
**Children:** #76 Linux, #77 macOS, #78 Delegation Registry  
**Consumers:** #6, #14, #16, #58, #60

## 1. Purpose

rururunx must own the lifetime of work it starts or delegates strongly enough to:

- register the work before external effects begin;
- stop or recover one Task/attempt without killing unrelated Tasks;
- survive caller cancellation and runtime interruption without abandoning work;
- distinguish execution success/failure from workload cleanup;
- refuse capabilities whose reachable execution cannot be contained or explicitly delegated;
- preserve native agent authentication, policy, hooks, trust and required behavior.

The common abstraction is an **Execution Domain**. A Domain is a durable, exact Project/Goal/Task/attempt-scoped ownership record plus a platform backend capable of enforcing the advertised execution profile.

## 2. Definitions

### Execution Domain

A provider-neutral ownership boundary for one managed operation/attempt. It is not a PID, process group, native session identifier or public Session row.

### Local workload

Processes/resources whose execution lifetime is enforced by the selected local backend.

### Delegated operation

Persistent work transferred to another execution authority such that death of the local workload does not prove that work ended. Examples include container-engine jobs, service managers, tmux-like servers, remote execution endpoints and deferred job services.

### Strong profile

A capability profile whose entire reachable local workload is contained by an enforced Domain and whose persistent external execution routes are either governed by #78 or blocked before effects.

### Native/Compatibility profile

A capability profile that preserves host-native behavior and may rely on narrower adapter-specific ownership. It may only advertise guarantees actually established by exact conformance and must never be promoted to Strong by label.

### Cleanup settlement

A verified conclusion that every enabled local resource and registered delegated operation for the Domain reached its reviewed terminal cleanup condition.

## 3. Normative invariants

### R75-1 — owner before effects

The runtime MUST create and durably bind an exact Execution Domain before the first external executable, helper, hook, Agent, verification runner, Git command, launcher probe, server or delegated job for that operation.

Pre-domain work is limited to in-process/non-executing metadata validation that cannot invoke external activation.

### R75-2 — ownership is not reconstructed from process hints

PID, PGID, SID, native session IDs, command lines, telemetry and persisted status text MAY be retained as bounded evidence, but MUST NOT create or widen ownership after the fact.

Recovery may use only a backend-specific durable identity/lease whose authority was established before effects.

### R75-3 — profile completeness is default-deny

For each supported profile, every reachable immediate or deferred execution route MUST be:

1. contained by the local backend;
2. registered under a reviewed DelegatedOperation contract; or
3. blocked before its first side effect.

An unclassified reachable persistent execution route makes that profile Unsupported.

### R75-4 — caller lifetime is weaker than owner lifetime

Caller Drop, cancellation, timeout, adapter return, public Session mutation, Task relabel or Runtime interruption MUST NOT discard the actual owner.

Revocation MUST be level-triggered. After revocation becomes effective, no new local or delegated work may be admitted for that Domain.

### R75-5 — execution outcome and cleanup outcome are independent

The model MUST preserve independent values:

```text
ExecutionOutcome = Succeeded | Failed | Cancelled | Lost
CleanupOutcome   = Pending | Verified | Held | Unknown
```

A valid state includes `Lost + Verified`. Cleanup verification MUST NOT fabricate successful execution. Execution success MUST NOT imply cleanup verification.

### R75-6 — fail closed

Unknown or incomplete cleanup, lost owner authority, unclassified delegation, failed cleanup publication or uncertain provider state MUST retain the affected reservation/hold.

No retry, replay, Task/Goal completion, Project removal or conflicting execution may rely on a terminal label alone.

### R75-7 — Task noninterference

Stopping, revoking or recovering Domain A MUST target only A's exact local/delegated ownership.

The implementation MUST NOT use global process kills, shared daemon termination, broad same-UID adoption, global native-session cleanup or shared-resource deletion as a substitute for exact ownership.

Shared repository/service effects remain governed by #60 effect/conflict authority.

### R75-8 — native policy compatibility

A backend MUST preserve required native authentication, rules, hooks, trust, provider selection and model/effort defaults.

If the Strong backend cannot preserve a required capability, that capability is Unsupported for Strong. The runtime MUST NOT silently fall back to a weaker profile.

### R75-9 — platform-specific enforcement

The common interface MUST NOT require the same kernel mechanism on every OS.

- Linux Strong uses #76. cgroup v2 is the primary cohort/termination boundary; namespaces may strengthen isolation.
- macOS Strong uses #77. VM-class containment is the first candidate when host-native process mechanisms cannot prove whole-workload containment.
- Native/Compatibility backends remain separate and require their own exact evidence.

### R75-10 — external delegation settlement

Local backend death alone MUST NOT settle a DelegatedOperation.

#78 owns the provider-neutral registry and provider-specific create/start/cancel/verify contract. A Domain cannot become CleanupVerified while any required delegated operation remains Pending/Held/Unknown.

### R75-11 — durable recovery input

At minimum, the runtime MUST durably record:

```text
ExecutionDomain {
  id
  project_id
  goal_id?
  task_id?
  attempt_id
  profile
  backend
  backend_identity
  lifecycle
  revocation_state
  execution_outcome?
  cleanup_outcome
  cleanup_evidence_ref?
  owner_epoch
}
```

Backend secrets, auth values and arbitrary caller-provided process IDs MUST NOT be persisted as authority.

### R75-12 — recovery responsibility

#14 owns restart recovery and fencing.

Recovery MUST:

- enumerate unresolved exact Domain/DelegatedOperation records;
- consult only the exact backend/provider identity established before effects;
- preserve Unknown/Held when proof is unavailable;
- never adopt a numeric process/resource hint supplied by status JSON;
- prevent a stale prior owner from publishing or dispatching after a later epoch takes authority.

### R75-13 — consumers use the common contract

#6, #58 and #60 MUST consume or reference the Execution Domain contract for workload lifetime instead of creating weaker provider-local definitions.

Provider-specific controls MAY add constraints but MUST NOT weaken R75 invariants.

### R75-14 — installed conformance before support claims

A backend/profile support claim requires exact installed-host conformance for the actual backend/native/tool identity and effective configuration relied upon.

A changed identity or changed enforced input invalidates the prior support claim until applicable requalification.

### R75-15 — aggregate concurrency proof

#16 final acceptance MUST include at least four independent Tasks/Domains with:

- A running or delegated workload in each;
- one Task cancelled or made Lost;
- exact cleanup/recovery of that Task;
- the other Tasks continuing without collateral termination or shared-state corruption;
- profile-specific support evidence on each advertised MVP OS.

## 4. Lifecycle requirements

The logical lifecycle is:

```text
Preparing
  -> Ready
  -> Running
  -> Revoking
  -> Terminating
  -> Settling
  -> Settled
```

After any possible external effect, uncertainty may transition to Held/Unknown but MUST NOT transition to a clean pre-execution state without authoritative evidence.

Backend readiness MUST be complete before Running can cause the first external execution.

Settlement requires both:

1. an explicit ExecutionOutcome (or explicit Lost); and
2. CleanupOutcome::Verified for all required local/delegated resources.

## 5. Shared-effect boundary

Execution Domain answers **who owns workload lifetime**. It does not make shared filesystem/Git/service operations conflict-free.

#60 remains responsible for:

- effect/conflict reservations;
- shared Git/common-directory compatibility;
- runtime verification/Git/helper consumers;
- continuous protection of already-running peers.

A VM/cgroup does not waive #60.

## 6. Native/Compatibility boundary

Existing ProcessGroup and targeted inspection logic remains valid bounded evidence for the profile/call sites it actually covers.

It MUST NOT be used to infer:

- arbitrary detached descendant capture;
- external delegated-job cleanup;
- Strong profile readiness;
- recovery authority over a new process with a reused numeric identifier.

## 7. Non-goals

- OS-level protection from all independent same-user activity.
- Owning future work merely because Task source files could later be executed by an IDE/user.
- Disabling mandatory native hooks or configuration.
- Killing shared Docker/service daemons.
- Replacing #14 scheduling/recovery or #60 effect arbitration.
- Requiring VM execution on Linux when cgroup-based Strong conformance is sufficient.
- Requiring every macOS capability to use Strong when a separately qualified Native/Compatibility capability is explicitly selected.

## 8. Acceptance matrix

| ID | Acceptance |
| --- | --- |
| AC75-1 | Domain is durably registered and backend-ready before first external execution. |
| AC75-2 | Caller cancellation/runtime interruption cannot abandon an owned workload or admit new effects after revocation. |
| AC75-3 | ExecutionOutcome and CleanupOutcome remain independent through persistence/recovery. |
| AC75-4 | Task A termination cannot affect concurrently running Task B. |
| AC75-5 | Unclassified persistent delegation refuses Strong before effects. |
| AC75-6 | Local cleanup cannot settle an outstanding delegated operation. |
| AC75-7 | PID/PGID/SID/native IDs never become post-hoc ownership authority. |
| AC75-8 | #14 exact recovery preserves Unknown/Held when authoritative proof is absent. |
| AC75-9 | #6/#58/#60 consumers compose the contract without weaker fallback. |
| AC75-10 | Linux Strong is qualified only by #76 installed conformance. |
| AC75-11 | macOS Strong is qualified only by #77 installed conformance. |
| AC75-12 | #16 demonstrates 4+ Task noninterference with cancellation/Lost recovery. |

## 9. Review and rollout gates

Requirements and design are reviewed before source implementation.

The common state/interface may land before either Strong backend is advertised. Until a platform child Issue is qualified, the runtime reports that Strong backend/profile as unavailable rather than borrowing existing process-group evidence.

No acceptance in this Issue closes #6, #14, #16, #58 or #60 by itself.
