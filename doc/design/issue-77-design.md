# Issue 77: macOS Strong Execution Domain design

**Status:** Proposed
**Requirements:** doc/requirements/issue-77-requirements.md
**Parent contract:** #75

## Decision

Do not define macOS Strong as "ProcessGroup plus better scanning".

Use a VM-class per-domain boundary when exact whole-local-workload containment is required. Keep host-native ProcessGroup/inspection as a separate Native/Compatibility backend for capabilities that have exact evidence.

## Candidate architecture

~~~text
rrx supervisor
   |
   +-- ExecutionDomain A -> VM boundary A -> worktree A -> Agent/helpers
   +-- ExecutionDomain B -> VM boundary B -> worktree B -> Agent/helpers
~~~

The VM/backend adapter exposes prepare, activate, revoke, terminate and inspect_settlement under the common #75 interface.

Apple container / Virtualization-based execution is the first candidate. Qualification decides whether it can preserve required native CLI behavior; the design does not pre-approve it.

## Filesystem and Git

Expose only the worktree/project paths required by the capability.

The repository common directory, hooks/config/fsmonitor/helpers and shared effects remain governed by #60. A VM boundary is not proof that two Tasks can safely mutate the same shared Git surface.

## Native identity/configuration

The backend records nonsecret identity facts needed to reproduce the supported profile. Authentication secrets remain owned by the native tool/platform and are not stored in ExecutionDomain rows.

A Strong capability is unavailable if the required auth/config/hook/trust behavior cannot be preserved safely in the boundary.

## Native/Compatibility path

Existing selected ProcessGroup cleanup and targeted macOS inspection remain bounded components for host-native execution.

They can certify only the exact narrower behavior they prove. They do not establish arbitrary descendant/session containment or external delegated-job cleanup.

## Recovery

#14 recovers exact VM/backend identities created before effects. Missing or ambiguous authority becomes Held/Unknown.

It must not scan/adopt arbitrary host processes to recreate Strong.

## Tests

- boundary exists before first executable;
- descendant/new-session containment;
- exact A termination while B runs;
- native config/auth compatibility;
- required hooks/rules preserved;
- worktree isolation;
- delegated external work remains pending;
- Strong unavailable -> explicit refusal;
- Native/Compatibility cannot be mislabeled Strong;
- 4+ concurrent qualified domains on the actual supported host.
