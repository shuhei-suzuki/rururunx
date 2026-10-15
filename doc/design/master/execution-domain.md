# Execution Domain

**Status:** Draft master design  
**Normative issue:** #75  
**Platform backends:** #76 Linux, #77 macOS  
**Persistent delegation:** #78  
**Recovery consumer:** #14  
**Shared-effect consumer:** #60

## Purpose

Execution Domain is the runtime-wide ownership boundary for externally executing work.

It answers four questions:

1. Which exact Project/Task/attempt owns this workload?
2. Which local backend is responsible for containing and terminating it?
3. Which persistent external operations were delegated outside that local boundary?
4. Has execution outcome been determined separately from cleanup settlement?

It does not replace Agent adapters, Workflow, Scheduler or Git effect arbitration.

## Architecture

~~~text
Workflow / Scheduler
        |
        v
Execution Domain Manager
        |
        +-- durable ExecutionDomain record
        +-- revocation latch / owner epoch
        +-- local backend
        |      +-- Linux Strong: cgroup v2 (#76)
        |      +-- macOS Strong: VM-class boundary (#77)
        |      +-- Native/Compatibility: adapter-qualified backend
        |
        +-- Delegation Registry (#78)
        |      +-- Docker
        |      +-- future service/remote providers
        |
        +-- Shared Effect Authority (#60)
        |
        +-- Recovery / fencing (#14)
~~~

## Core rule

No external execution is allowed before an exact Domain owner and selected backend are ready.

"External execution" includes native Agents, Git/helper commands, hooks, verification runners, setup probes, local servers and persistent delegated jobs.

Pure in-process metadata checks may occur before Domain creation only when they cannot trigger external activation.

## Profiles

### Strong

Every reachable local execution route is contained by the backend. Persistent execution outside that boundary is registered through #78 or refused before effects.

### Native/Compatibility

Host-native operation may use narrower provider-specific custody where exact evidence exists. It never inherits Strong by role, platform or successful direct-child/process-group cleanup.

A Task cannot silently change profile after execution begins.

## State

Execution lifetime and cleanup lifetime are separate.

~~~text
ExecutionOutcome
  Succeeded
  Failed
  Cancelled
  Lost

CleanupOutcome
  Pending
  Verified
  Held
  Unknown
~~~

A normal and important state is Lost + Verified: the runtime knows all owned work is gone but does not know whether the final operation succeeded.

Succeeded + Pending/Held is also valid: the Agent returned success but owned/delegated resources have not settled.

## Lifecycle

~~~text
Preparing -> Ready -> Running
                       |
                       v
                    Revoking
                       |
                       v
                   Terminating
                       |
                       v
                    Settling
                       |
               +-------+--------+
               |                |
            Settled          Held/Unknown
~~~

Revocation is durable and level-triggered. After it is committed, new work cannot be admitted to the Domain.

## Ownership identity

Backend identity is established before effects and persisted with the Domain.

Numeric PID/PGID/SID, native Session IDs, command lines and telemetry are diagnostics only. They cannot be used later to adopt a new process or reconstruct ownership authority.

## Platform strategy

### Linux

The Strong backend is based on a dedicated cgroup v2 domain. Process-group/session changes do not alter ownership. Namespaces may provide stronger isolation but are not the ownership identity.

### macOS

The Strong backend uses a VM-class per-domain boundary when host-native process primitives cannot prove whole-workload containment. Apple container / Virtualization-based execution is the first candidate and requires exact installed conformance.

Host-native ProcessGroup inspection remains useful for Native/Compatibility paths and diagnostics.

## Delegation

A VM or cgroup does not prove cleanup of work transferred to an external execution authority.

Persistent delegation uses exact DelegatedOperation records with provider-specific create/start/cancel/verify semantics.

Shared daemons are not killed to clean one Task. For example, Task A cleanup targets A's registered container, not Docker itself.

## Shared resources

Execution Domain provides lifetime ownership, not universal resource isolation.

Shared Git/common-directory/config/helper/service effects continue to use #60 compatibility and conflict reservations.

Stopping Task A must not invalidate Task B's independent Domain or effect ownership.

## Recovery

#14 owns restart recovery and scheduler fencing.

Recovery enumerates unresolved exact Domain/delegation records and reconciles them using their pre-effect backend/provider identity.

If authoritative settlement cannot be established, the record remains Held/Unknown. A PID hint, terminal Session row or user-edited status is not sufficient.

Owner epochs prevent a stale pre-restart owner from dispatching or publishing after recovery transfers authority.

## Agent adapter integration

Adapters receive a DomainPermit or equivalent private handle before any launch-capable path.

Adapters own provider-specific protocol/session behavior. They do not decide whether a weaker process primitive can satisfy the runtime-wide Strong contract.

A provider may further restrict a profile; it cannot weaken the common invariants.

## Support claims

Support is expressed as a profile + backend + capability combination, for example:

~~~text
linux / strong / codex-execute
macos / strong / codex-review
macos / native-compatibility / codex-consult
~~~

The runtime must report Unsupported when the exact combination is not qualified.

## Final MVP proof

#16 aggregate dogfood must show at least four concurrent independent Tasks/Domains, make one Task cancel or become Lost, settle/recover that Task exactly, and demonstrate that the remaining Tasks continue without collateral termination or shared-state corruption.

See doc/requirements/issue-75-requirements.md and doc/design/issue-75-design.md for the normative contract and detailed design.
