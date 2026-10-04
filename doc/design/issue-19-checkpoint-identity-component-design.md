# Issue19 checkpoint append identity component

Status: Design1 component APPROVABLE (native d52f3139 at4740148); source
gate pending. This is a small correction of the existing artifact contract; native managed
operation/settlement, production migration and whole Issue19 acceptance remain OPEN.
Main has schema3 and no CPP service. This candidate branch has an unmerged schema5
CPP prototype; no deployment or schema6 source is proposed here.

Problem: private Store::append_pack_checkpoint ties outer Record scope/kind/version
and predecessor argument to live rows, but not decoded Checkpoint.scope/session/
role/previous/format. Its sole service caller currently creates consistent bodies.
Review5 L1/Review7 L5 therefore identify internal producer-integrity omissions,
not demonstrated native ownership or arbitrary-user action authorization defects.

Existing requirements: typed Project/Goal/Task-scoped append; consecutive immutable
predecessor; mandatory semantic events persist; head+record+audit atomic and CAS.
This correction implements those existing requirements and changes no native
writer policy, schema, row caps, event truth classification or migration epoch.

## Exact implementation scope

In the existing append transaction, bind the decoded Checkpoint to:

- exact expected format rrx.checkpoint.v1 and supplied scope;
- supplied Session ID and current persisted Session role;
- supplied immutable previous CheckpointRef, including None;
- exact Task worktree already matched to the persisted Session;
- for a predecessor, the new retained mandatory list starts with the complete
  predecessor's retained list in the same order and with exact event/Session facts.

Existing P/G/T version CAS, live head reference/digest, current Session version,
terminal/publication fences, consecutive chain/sequence and atomic audit are kept.
No predicate treats public role or an initial terminal label as native cleanup.
Already materialized bounded JSON stays under current service/Store contracts;
this is no streaming optimization or compact-transaction performance claim.

Use explicit equality and prefix checks on existing typed DTOs, with fixed bounded
error messages. No JSON receipt token, new generic API, private owner index,
SQL schema change or unrelated source reclassification. The sole production caller
ContextPacks::checkpoint constructs all pinned fields and clones the old retained
list, so accepted output bytes, digest and head behavior remain unchanged.

## Actual consumer proof

Own tempfile repo/Store fixture creates a genuine service checkpoint from a factual
initial terminal Consultant Session. That is application history only: no inference,
managed admission, native process/cleanup or settlement evidence. Capture exact
current P/G/T/Session versions and checkpoint reference, construct the consecutive
next body consistently, then call actual crate-private append_pack_checkpoint.

Positive first/consecutive controls persist and load through ContextPacks reader.
For each inner scope, Session, role, predecessor, format, worktree and dropped/
rewritten mandatory-prefix variant, the actual Store consumer refuses and record ID,
head and checkpoint.saved audit remain unchanged. A stale outer CAS and current-head
race retain existing failure behavior. No raw SQL seeding is counted as a positive.
Atomic audit-trigger failure retains no partial record/head, using only the owned DB.

Compile separately committed omission mutants against each important equality and
mandatory-prefix predicate. Count only runtime assertion failures where consumer
would accept the inconsistent DTO, not compiler/parser/unrelated later failures.
Restore committed clean controls. Run focused existing CPP/state regressions, lint
and default-parallel full checks when composition changes justify them.

## Scope boundary

The component review covers only that private append consumer/caller contract and
its direct fixtures. It cannot qualify PR39's native/session paths, schema5 prepared
input index or proposed6 managed authority, real providers, legacy migration or
19/23/43/60 composition. Parent CPP service still needs its full source gate.


Reader-only classification, sequence-range, omitted commitment, recent accounting,
nullable measurements and new suffix provenance predicates remain residual outside
this identity/prefix correction. This component does not claim that append accepts
exactly the set of DTOs accepted by the reader. Native/whole-CPP gates remain OPEN.
