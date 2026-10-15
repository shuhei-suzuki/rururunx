# Issue 78: Delegation Registry requirements

**Status:** Proposed
**Parent:** #75
**Workflow:** STRICT
**Initial provider:** Docker/container engine

## Purpose

Track and settle persistent Task work transferred outside the local Execution Domain.

Local process, cgroup or VM death must never certify cleanup of an externally delegated job.

## Requirements

1. Every persistent delegated operation MUST be reserved and bound to an exact ExecutionDomain/attempt before it can start.
2. The provider-specific external resource identity MUST be captured from the authoritative provider and persisted as soon as safely possible.
3. Caller-supplied resource IDs MUST NOT create adoption/cancellation authority.
4. Revocation is level-triggered; after owning Domain revocation, no new delegated job may be admitted.
5. Cancellation targets only the exact registered resource and MUST NOT terminate a shared provider daemon.
6. Provider acknowledgement alone is not cleanup proof. Provider-specific settlement verification is required.
7. Local Domain cleanup cannot become Verified while a required DelegatedOperation is Pending/Held/Unknown.
8. Runtime/caller loss preserves the operation, revocation intent and owner epoch for #14 recovery.
9. Ambiguous create/start/publication windows MUST resolve conservatively and MUST NOT duplicate replay.
10. Provider unreachable/unknown state becomes Held/Unknown.
11. Credentials/auth values are never persisted in DelegatedOperation state.
12. Unclassified reachable persistent delegation makes the owning Strong profile Unsupported before effects.
13. Bounded ordinary inference/API requests that cannot become persistent Task execution are documented separately and are not automatically DelegatedOperations.

## Minimum model

~~~text
DelegatedOperation
  id
  execution_domain_id
  scope/attempt
  provider
  provider_identity
  external_resource_id
  lifecycle
  revocation_epoch
  execution_outcome?
  cleanup_outcome
  cleanup_evidence_ref?
  owner_epoch
~~~

## Acceptance

- AC78-1: Docker is the first concrete provider.
- AC78-2: Task A exact container cleanup leaves Docker daemon and Task B container running.
- AC78-3: local Domain death with live A container does not verify cleanup.
- AC78-4: crash windows do not cause implicit duplicate start/replay.
- AC78-5: forged resource IDs cannot adopt/cancel unrelated resources.
- AC78-6: provider unreachable remains Held/Unknown.
- AC78-7: no new delegated work after revocation.
- AC78-8: #14 can reconcile only exact persisted provider identities.
- AC78-9: #60 uses the exact operation for conflict/shared-effect decisions where applicable.

## Non-goals

- Killing shared daemons.
- Claiming every cloud service is supported.
- Persisting provider secrets.
- Using local containment as delegated settlement proof.
