# Issue 77: macOS Strong Execution Domain requirements

**Status:** Proposed
**Parent:** #75
**Workflow:** STRICT

## Purpose

Provide a macOS Strong backend whose guarantee does not depend on complete host PID/PGID/session discovery.

A per-Task/per-attempt VM-class boundary is the first implementation candidate. Apple container / Virtualization-based execution is a candidate, not an assumed capability.

## Requirements

1. The Strong boundary MUST be created and ready before any Agent/Git/helper/hook external execution for the Domain.
2. Strong settlement MUST be based on exact boundary lifecycle, not selected ProcessGroup death.
3. Terminating Domain A MUST NOT stop Domain B or unrelated host-native processes.
4. Descendants that change parent, process group or session remain inside the boundary.
5. Worktree/repository exposure MUST be explicit and scoped. VM use does not waive #60 shared Git/effect rules.
6. Required native authentication/settings/hooks/trust/model defaults must remain authoritative for each advertised Strong capability.
7. Secrets/auth values MUST NOT be copied into durable rururunx state as part of VM preparation.
8. Persistent work outside the VM/boundary composes #78.
9. Capabilities that cannot operate compatibly inside Strong MUST be reported Unsupported rather than falling back silently.
10. Host-native execution remains available only as a separately qualified Native/Compatibility profile.
11. A successful ProcessGroup cleanup receipt MUST NOT upgrade Native/Compatibility to Strong.
12. Backend/native identity changes require requalification before Strong support is advertised.

## Acceptance

- AC77-1: requirements/design compare VM-class and host-native containment limits.
- AC77-2: no external execution occurs before boundary readiness.
- AC77-3: nested/reparented/new-session descendants remain contained.
- AC77-4: A termination leaves B running.
- AC77-5: actual required worktree/Git/native configuration paths are proven without disabling required hooks/rules.
- AC77-6: unsupported native capabilities are explicit and pre-effect.
- AC77-7: external delegation remains unsettled until #78.
- AC77-8: installed macOS conformance records exact backend/native identities.
- AC77-9: #16 4+ Task acceptance uses only actually qualified profiles.

## Non-goals

- Global host process kills.
- Same-UID process ownership.
- Claiming the VM contains host/external delegated services.
- Requiring every macOS capability to be Strong.
- Weakening native agent safety/defaults for compatibility.
