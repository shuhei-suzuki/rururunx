# Issue 76: Linux cgroup v2 Execution Domain requirements

**Status:** Proposed
**Parent:** #75
**Workflow:** STRICT

## Purpose

Implement the Linux Strong backend for the #75 Execution Domain contract.

The backend must own the complete reachable local workload for an admitted profile without relying on parent PID or process-group ancestry, and must stop one Task/attempt without affecting unrelated Domains.

## Requirements

1. A dedicated cgroup v2 domain MUST exist and be bound to the exact ExecutionDomain before the first external executable for that Domain starts.
2. cgroup identity is the primary local workload ownership boundary. PID/PGID/SID are diagnostics only.
3. The launcher and every ordinary descendant MUST execute inside the exact domain. Reparenting, creating a new process group or creating a new session must not escape ownership.
4. cgroup-wide termination MUST target only the exact domain/subtree owned by the attempt.
5. CleanupVerified requires backend-authoritative evidence that the owned local workload is empty/settled. Direct-child or PGID death is insufficient.
6. Cancellation/runtime loss must persist revocation and retain the Domain until termination/settlement completes or becomes Held/Unknown.
7. Unsupported cgroup delegation/permission/configuration prerequisites MUST refuse Strong before external execution. The implementation must not assume root.
8. PID/mount/network namespaces MAY strengthen isolation but are optional unless a capability explicitly requires them.
9. Persistent external execution through Docker/service managers/remote endpoints composes #78 and cannot be settled from cgroup death.
10. Required native auth/settings/hooks/trust/defaults must remain usable. If not, the capability is Unsupported for Linux Strong.
11. Task A termination MUST NOT signal/kill Task B or a shared provider daemon.
12. #14 recovery may use only the persisted exact cgroup/backend identity established before effects.

## Acceptance

- AC76-1: register-before-effect is causally tested.
- AC76-2: fork + reparent + new PGID/SID remain owned or the capability is refused.
- AC76-3: A termination leaves B running under concurrent execution.
- AC76-4: caller/runtime cancellation does not admit new work after revocation.
- AC76-5: local cleanup cannot settle outstanding #78 delegation.
- AC76-6: stale numeric process hints cannot be adopted.
- AC76-7: at least four concurrent domains run on a real supported Linux host.
- AC76-8: actual Agent/Git/helper compatibility is qualified for each advertised Strong capability.
- AC76-9: unsupported host prerequisites fail before external execution.

## Non-goals

- Treating PID namespace as the ownership identity.
- Root-only installation as an implicit MVP prerequisite.
- Owning unrelated same-user processes.
- Replacing #60 shared-effect arbitration.
