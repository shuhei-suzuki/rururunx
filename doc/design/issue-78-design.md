# Issue 78: Delegation Registry design

**Status:** Proposed
**Requirements:** doc/requirements/issue-78-requirements.md
**Parent contract:** #75

## Architecture

~~~text
ExecutionDomain
    |
    +-- local backend
    |
    +-- DelegationRegistry
            |
            +-- DockerProvider
            +-- future providers
~~~

A provider implements reserve/create/start/revoke/inspect settlement behavior under an exact DomainPermit/DelegationPermit.

## Lifecycle

~~~text
Reserved
   -> Allocating
   -> Starting
   -> Running
   -> Revoking
   -> Verifying
   -> Settled

Any ambiguous post-effect state -> Held/Unknown
~~~

## Docker first provider

Preferred flow where supported:

~~~text
persist Reserved
  -> create exact container
  -> persist authoritative container ID
  -> start
  -> Running
~~~

Cancellation targets that exact container. The Docker daemon remains shared and must never be killed to clean one Task.

Settlement uses authoritative provider inspection sufficient to show the registered resource is stopped/removed according to the reviewed contract.

## Crash windows

If allocation may have occurred but exact identity publication is uncertain, the operation cannot be reset to pristine automatically.

The provider design must either establish an atomic/idempotent identity key before allocation or retain Held/Unknown until exact reconciliation proves the original effect.

Automatic duplicate replay is forbidden.

## Security

Provider credentials remain external/native. Persist only nonsecret provider identity and resource references needed for exact recovery.

Never accept an arbitrary external resource ID from a public caller as cancellation authority.

## Composition

ExecutionDomain cleanup is a join:

~~~text
local backend settled
AND
all required DelegatedOperations settled
AND
required #60 effect settlement
~~~

Only then may the Domain publish CleanupOutcome::Verified.

## Recovery

#14 enumerates unresolved operations and asks the exact provider implementation to reconcile the exact pre-effect provider/resource identity.

If the provider cannot establish the original resource state, the operation remains Held/Unknown.

## Tests

- exact Docker container identity;
- A stop leaves shared daemon/B container alive;
- local parent/domain death does not settle container;
- crash after reserve, after allocate, after ID persist, after start;
- provider unavailable;
- stale owner epoch;
- forged external ID;
- revocation before new allocation;
- repeated stop/reconcile idempotence.
