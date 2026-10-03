# Issue 19 design: typed working context with explicit authority

Use a `context_pack` library service backed by existing ContextVersion envelopes
and scoped Checkpoint records. Pack data is typed and bounded; reads verify envelope
scope/version/digest and exact pointer ownership before trusting it. Context bodies
reference authoritative files by owned scope/path/SHA256, not replacement summaries.

Task authority projects only meaningful Goal/Task/Project fields. Database revision,
timestamps and context pointers are CAS guards, not semantic inputs: publishing
v1 must leave v1 usable. Native repository authority comes from Issue 18's bounded
FD/Git snapshot and content hashes; HEAD alone is insufficient. Rebuild outside
SharedStore, compare authority, then use atomic publication/current-state guards.

Packing places the full typed mandatory pack frame before a budgeted repository
slice. The complete rendered bytes count toward the byte estimate. Rules and
explicit evidence/expansions remain complete or return a budget failure. Stored
packs keep compact metadata/references, not full source payloads. Native input
preparation re-fetches current source through the selector and never trusts a
serialized caller-provided repository map.

Goal packs contain compact Task descriptors and exact pack version/digest refs,
DAG edges, cross-Task facts and aggregate metrics. They do not embed Task packs,
source slices or Task chat. An owned bound Task anchors repository/rule observation;
referenced Tasks remain explicitly scoped. Known stale Task references are rejected.

Deterministic condensation operates on typed event classifications. Semantic events
are retained with sequence/provenance; duplicate semantic facts can be deduplicated
without losing their category. Only explicitly transient notes can leave the recent
window. This strategy cannot infer safety meaning from arbitrary unclassified chat;
callers must normalize native events honestly. Mandatory state overflow is explicit,
not silent truncation. Checkpoints record input digest, consumed sequence range,
retained facts and bounded verbatim tail, with audit metadata excluding text bodies.
Existing checkpoint authority and previously retained semantic facts must survive
incremental checkpointing even after they leave the recent window.

Checkpoint append may observe a running owned Session but never changes its launch
pack or attempt identity. Standalone pack publication rejects live/Lost sessions,
locks and workflow-owned Tasks atomically; Workflow Engine owns phase publication.
Consultant checkpoints promote only within the same Project/Goal after source
reference validation, retaining historical source Session/Task/HEAD provenance.
Target rules/constraints and source hashes are rebuilt; no full chat is copied.

Use no new persistence format. Add narrow Store transactions beside existing
helpers and coordinate their workflow fences with Issue 8. The PhaseContext port
provides a draft payload/source authority while Engine owns ContextVersion/Task
pointer publication and attempt lifecycle. Provider operations remain in adapters.

Validation uses isolated native repositories, restart/version idempotence,
same-length dirty/rule/config/artifact changes, two Projects with identical names,
Goal DAG refs, budget/mandatory preservation, incremental condensation, malformed
references, consultation provenance and atomic cross-connection/session fences.
Important guarantees receive detached mutation proof and immutable native review.
