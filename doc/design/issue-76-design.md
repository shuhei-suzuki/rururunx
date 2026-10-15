# Issue 76: Linux cgroup v2 Execution Domain design

**Status:** Proposed
**Requirements:** doc/requirements/issue-76-requirements.md
**Parent contract:** #75

## Backend model

Each Strong ExecutionDomain receives one cgroup subtree under a runtime-owned hierarchy.

~~~text
rururunx
  runtime-<id>
    domain-<execution-domain-id>
~~~

The durable BackendIdentity stores the exact managed hierarchy/domain identity and backend version/feature facts needed for #14 reconciliation. It does not store arbitrary PID authority.

## Launch sequence

~~~text
in-process validation
  -> prepare cgroup/domain
  -> reserve #60 effect scope as needed
  -> persist ExecutionDomain + BackendIdentity
  -> activate launcher inside domain
  -> allow Agent/Git/helper execution
~~~

No external command may be used to discover whether the backend is usable unless that probe itself is executed under an already valid Domain or is a side-effect-free authoritative metadata check.

## Termination

Revocation is persisted first. The backend then denies new permits and performs cgroup-wide termination for the exact domain.

Settlement requires authoritative domain-empty/inactive evidence after termination. The selected leader PID/PGID is not the cleanup predicate.

## Isolation

Process-group/session changes do not escape the cgroup boundary. Optional namespaces can isolate PID visibility, mounts or network when required by a profile but are not needed to define ownership.

Shared filesystem/Git semantics remain #60 concerns.

## Recovery

#14 reads the exact stored BackendIdentity and asks the backend to inspect/reconcile that domain. If the identity is gone but absence cannot be distinguished from lost authority safely, the record remains Held/Unknown.

No scanning of all same-user processes and no command-line matching is allowed for adoption.

## Tests

Required fixtures:

- direct child;
- forked/reparented child;
- child with new process group;
- child with new session;
- nested descendant tree;
- A/B concurrent Domains;
- cancellation before first effect;
- cancellation after descendants exist;
- backend prerequisite unavailable;
- local cleanup with outstanding delegated operation.

Installed conformance records host/kernel/backend prerequisites and actual native Agent behavior.
