# Issue 75: Execution Domain design

**Status:** Proposed
**Requirements:** doc/requirements/issue-75-requirements.md
**Master design:** doc/design/master/execution-domain.md

## 1. Design objective

Provide one durable runtime ownership abstraction that can supervise native Agent attempts, runtime Git/helpers/hooks, verification jobs, non-Task Consultant lifetimes, and external persistent delegated execution without pretending that one OS process primitive solves every case.

The design separates:

1. logical ownership — durable rururunx state and exact scope/attempt identity;
2. local containment backend — platform/profile-specific enforcement;
3. delegated execution registry — exact external resources that outlive local processes;
4. effect/conflict authority — shared-resource compatibility owned by #60;
5. recovery — restart reconciliation/fencing owned by #14.

## 2. Core types

Suggested Rust-facing model:

~~~rust
pub struct ExecutionDomainId(Uuid);

pub enum ExecutionProfile {
    Strong,
    NativeCompatibility,
}

pub enum DomainLifecycle {
    Preparing,
    Ready,
    Running,
    Revoking,
    Terminating,
    Settling,
    Settled,
    Held,
}

pub enum ExecutionOutcome {
    Succeeded,
    Failed,
    Cancelled,
    Lost,
}

pub enum CleanupOutcome {
    Pending,
    Verified,
    Held,
    Unknown,
}

pub struct ExecutionDomainRecord {
    pub id: ExecutionDomainId,
    pub scope: Scope,
    pub attempt_id: AttemptId,
    pub profile: ExecutionProfile,
    pub backend: BackendKind,
    pub backend_identity: BackendIdentity,
    pub lifecycle: DomainLifecycle,
    pub revocation_epoch: u64,
    pub execution_outcome: Option<ExecutionOutcome>,
    pub cleanup_outcome: CleanupOutcome,
    pub cleanup_evidence_ref: Option<EvidenceRef>,
    pub owner_epoch: u64,
}
~~~

The stored record describes authority and lifecycle. It is not itself an OS handle. The live runtime owns a non-serializable ExecutionDomainHandle supplied by the backend.

## 3. Backend interface

Logical interface:

~~~rust
#[async_trait]
trait ExecutionDomainBackend {
    async fn prepare(&self, request: DomainRequest)
        -> Result<PreparedDomain, DomainError>;

    async fn activate(&self, prepared: PreparedDomain)
        -> Result<ActiveDomain, DomainError>;

    async fn revoke(&self, domain: &ActiveDomain)
        -> Result<RevocationReceipt, DomainError>;

    async fn terminate(&self, domain: &ActiveDomain)
        -> Result<TerminationReceipt, DomainError>;

    async fn inspect_settlement(&self, identity: &BackendIdentity)
        -> Result<SettlementObservation, DomainError>;

    fn capabilities(&self) -> BackendCapabilities;
}
~~~

Important ordering:

~~~text
validate in-process metadata
        ↓
reserve effect/conflict scope (#60 where applicable)
        ↓
prepare backend boundary
        ↓
persist ExecutionDomain + backend identity
        ↓
activate boundary
        ↓
external execution allowed
~~~

There is no spawn-then-register fast path.

## 4. Admission gate

Every execution-capable consumer obtains a DomainPermit containing domain ID, attempt ID, owner epoch, profile and capability class.

The permit is checked at each first-effect boundary:

- Agent launch/resume/checkpoint;
- Git/helper invocation;
- hook-capable Git operation;
- test/mutation/browser/staging runner;
- persistent service/job delegation;
- native TUI/server launch.

A permit is not transferable across attempts. Once revocation is latched, permit checks fail before new effects.

## 5. Strong vs Native/Compatibility

### Strong

Strong means the runtime can explain every reachable execution path:

~~~text
local execution
    -> contained by backend

persistent external execution
    -> exact DelegatedOperation

unclassified route
    -> Unsupported before effects
~~~

Strong is a profile claim, not a Task label.

### Native/Compatibility

Native/Compatibility permits host-native operation where the adapter can prove its narrower ownership contract.

Existing ProcessGroup/inspection code can remain a backend component for such profiles. Its evidence is not promoted to arbitrary-descendant containment.

The scheduler may select a profile explicitly or by reviewed capability policy. It must not silently switch Strong to NativeCompatibility after work begins.

## 6. Linux backend composition (#76)

Primary identity:

~~~text
ExecutionDomainId
   ↕ durable mapping
cgroup identity under rururunx-managed hierarchy
~~~

Design rules:

- the cgroup is created before the launcher is permitted to execute;
- the launcher starts inside the exact domain;
- descendant process-group/session changes do not change Domain ownership;
- Domain termination is cgroup-wide, never host-global;
- settlement observes backend-authoritative empty/inactive state;
- namespaces are optional additional isolation, not the base identity;
- delegated Docker/service work is tracked separately by #78.

The implementation must define supported cgroup delegation/systemd-user prerequisites rather than assuming root privileges.

## 7. macOS backend composition (#77)

Host PID/process-group APIs do not become the Strong definition.

Strong candidate:

~~~text
Task/Attempt
   ↓
ExecutionDomain
   ↓
VM-class boundary
   ↓
Agent + Git + helpers + descendants
~~~

The first candidate is Apple container / Virtualization-based execution or an equivalent per-domain VM backend.

The backend owns boundary creation/readiness, exact Task worktree exposure, lifecycle termination, settlement evidence, backend identity/version and explicit capability limitations.

Host-native Agent operation remains a separate Native/Compatibility backend. If the VM boundary cannot preserve a required native capability, that capability is Unsupported for Strong rather than bypassed.

## 8. Delegation composition (#78)

The local backend exposes a DelegationPermit bound to the Domain.

Provider flow:

~~~text
reserve DelegatedOperation
        ↓
provider create/allocate
        ↓
persist exact provider resource ID
        ↓
provider start/continue
        ↓
Running
~~~

Where a provider cannot separate create/start, its design must specify the smallest atomic/compensating window and how failure becomes Held/Unknown.

Domain cleanup settlement joins:

~~~text
local settlement
AND
all required delegated-operation settlements
AND
required shared-effect settlement
~~~

Only that conjunction permits CleanupOutcome::Verified.

## 9. Revocation and cancellation

Revocation is a durable state transition, not a transient signal.

~~~text
Active
  ├─ caller cancel
  ├─ Task cancel
  ├─ timeout
  ├─ runtime shutdown
  └─ policy revoke
        ↓
persist revocation epoch
        ↓
deny new permits
        ↓
revoke delegated operations
        ↓
terminate local backend
        ↓
verify settlement
~~~

The order may overlap for latency, but no later admission may race ahead of a committed revocation epoch. The caller waiting for stop is not the owner of cleanup progress.

## 10. Runtime/supervisor lifetime

The API is designed so a durable supervisor can outlive CLI/Workflow callers.

The exact daemon/process packaging is owned by #14, but the boundary is:

- Domain records and revocation intent are durable;
- live backend handles are retained by the supervisor while available;
- after supervisor restart, #14 reconstructs only through backend/provider identities created before effects;
- inability to reconstruct produces Held/Unknown, not fabricated death;
- a stale pre-restart actor is fenced by owner epoch before it may publish or dispatch.

## 11. Outcome publication

Execution result and cleanup result have separate writers and evidence.

Example:

~~~text
native response not observed
process/VM/delegation cleanup verified

ExecutionOutcome = Lost
CleanupOutcome   = Verified
~~~

Scheduler policy may allow a later retry after exact Lost+Verified settlement, but it may never rewrite the original attempt as successful.

Conversely:

~~~text
Agent reports success
external delegated container still running

ExecutionOutcome = Succeeded
CleanupOutcome   = Pending/Held
~~~

The Task cannot release conflicting ownership until cleanup settlement is valid.

## 12. Shared effect integration (#60)

Domain membership is necessary but not sufficient for Task isolation.

Before shared effects, consumers also acquire an effect lease compatible with already-running peers.

Examples:

- own worktree writes: normally Task-private;
- Git common-object append: may be compatible under reviewed semantics;
- ref/branch mutation: exact scoped lease;
- shared hooks/config/fsmonitor/helper activation: explicit compatibility enforcement;
- provider daemon: daemon remains shared, delegated child/resource is exact.

If Task A becomes Lost, #60 continuous enforcement prevents A unresolved effect scope from being reused by B when that would be unsafe.

## 13. Persistence/schema direction

Recommended tables:

~~~text
execution_domains
execution_domain_events
delegated_operations        (#78)
delegated_operation_events  (#78)
~~~

ExecutionDomain rows are append/audit-friendly with compare-and-swap owner epoch/version.

Do not store OAuth tokens, raw provider secrets, arbitrary environment values, or untrusted caller PID/PGID as authority.

Migration coordination must preserve the existing linear schema/writer history used by #19/#58/#60.

## 14. Error model

Suggested classes:

- UnsupportedBackend
- UnsupportedProfile
- BackendNotReady
- DomainConflict
- Revoked
- TerminationFailed
- SettlementUnknown
- DelegationUnsupported
- RecoveryAuthorityMissing
- BackendIdentityChanged

Errors after possible effects never roll state back to pristine without authoritative no-effect proof.

## 15. Test strategy

### Contract tests

Every backend runs the same semantic suite:

- register-before-effect;
- revoke-before-first-effect;
- caller-drop retention;
- repeated stop idempotence;
- Lost + Verified;
- success + pending cleanup;
- stale owner epoch refusal;
- A cancellation while B continues.

### Linux #76

Test fork/reparent, new process group/session, descendant trees, cgroup-wide termination, backend settlement, 4+ concurrent domains and unsupported prerequisite refusal.

### macOS #77

Test boundary readiness before execution, descendant/session containment, A termination while B runs, worktree/config/auth compatibility, exact unsupported capability reporting and Native/Compatibility not masquerading as Strong.

### Delegation #78

Test Docker exact container registration, A container stop while daemon/B continue, runtime crash windows, provider unreachable -> Held, forged resource ID refusal and local cleanup not settling delegated work.

### Aggregate #16

At least four Tasks with mixed execution/verification work, one cancelled or Lost during the run, no collateral effects, exact cleanup evidence and no manual terminal polling.

## 16. Migration plan

1. Land #75 contract/state types with no new Strong capability claims.
2. Route new/updated execution consumers through the common admission API.
3. Implement #76 Linux Strong.
4. Implement/qualify #77 macOS Strong independently.
5. Implement #78 delegated execution, initially Docker.
6. Update #6/#58/#60 consumers to remove duplicated weaker ownership decisions.
7. Complete #14 recovery/supervisor integration.
8. Run #16 aggregate conformance.

Existing process-group code remains in place until each consumer is migrated. There is no blanket replacement commit and no inferred support upgrade.
