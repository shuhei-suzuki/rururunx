# Original marker planning implementation checkpoint

This records partial implementation of the approved
[managed binding design](issue-43-managed-binding-design.md), §2.2. It does not
approve a marker transaction, private Session binder, Native start or Phase2.

## Implemented source

`state/managed_binding/marker_plan.rs` takes the actual selected-port
`NativeAllocation` by immutable borrow. It checks its retained selected owner
state path/root/instance/epoch, then reads complete current Unit, immutable
execution-context admission and P/G/T/Workflow/latest Context/all scoped locks
in one separate readonly transaction. It does not open another RuntimeOwner,
acquire SharedStore, advance epoch, run Git/version helpers or write anything.

Complete original Unit capture is compared with the current typed body; indexed
scope/kind/generation/epoch/version/native/result flags/path/branch and current
Task execution generation must agree. The original execution-context P/G versions
and governing digest must agree with the same captured P/G bodies. Source7
provenance and actual Driver ownership remain separate unimplemented consumers.

The sole active native attempt must have its prepared Unit, selected actor and
role, original Context version, no Session/execution reference, no prior dispatch
and no finished/terminal/held state. Context scope/version/revision/source map,
full encoded native payload, Workflow phase/class/generation/budget/source frame
and prepared namespace are compared. Reviewer requires a retained artifact and
its own prepared path; it does not reuse Task executor worktree as its identity.

The nongrant plan fixes exact post-marker Task/Workflow encoded bytes, versions
and timestamp before any writer transaction. Task state/phase plus version/time
and the sole attempt's dispatch flag plus Record version/time are the intended
initial marker projection. It retains exact old bytes separately from canonical
body hashes. The proposed complete frame includes all original body pins,
Unit/governing/input/profile/selection identities and sorted complete lock pins.
It uses the approved complete encoding recipe and NUL-terminated body domain.

## Remaining authority and verification gates

`ManagedMarkerPlan` is not `OriginalMarker`, is not Clone/Deserialize and has no
publication or Native launch method. A hash or successful plan cannot register a
Session, consume input, grant permission or settle work. Its transaction recheck
alone is nongrant and must be inside the eventual Immediate producer, including
exact current Unit indexes and immutable execution-context admission.

The actual retained supervisor capacity, original Driver claim/pin handoff,
Source7 pre-marker receipt, schema10 operation/owner/input rows, mandatory audit
budget reservation, genuine marker publication and all Native registration/input/
ACK/terminal consumers remain necessary. OriginalMarker can be issued only after
that real transaction commits. No DTO, SQL row or named capacity token replaces
these missing producers. Native availability remains unavailable.

Current existing scope controls use genuine trusted Goal ingress but empty
Workflow/Context/lock inventories. They do not qualify the new positive marker
path. Actual complete populated/corrupted inventories, current Unit/admission
drift, original Context/actor/profile changes, near8MiB Workflow cost, and actual
producer mutations require genuine prepared allocation and Driver prerequisites;
no private owner/Unit/proof may be SQL-seeded to bypass missing prerequisites.

Compile/fmt checks and existing read-plan controls are recorded separately from
positive marker, binder, full regression, authenticated Native and both-OS gates.
No README, license, process-custody or security-sandbox claim is changed.
