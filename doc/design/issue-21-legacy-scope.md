# Issue21 legacy usage read scope component

STRICT component under approved Requirements8 e001483 and Design4 8a4016a.
Baseline main054aefd/schema3. Whole21 remains OPEN. This implements the existing
row/body identity validation condition for the legacy read API, without qualifying
legacy counters or granting execution authority.

Current Store::usage filters SQLite scope columns but decodes and returns body
without checking its identity. A drifted/corrupt body can therefore expose a foreign
Project/Goal/Task or Session through an otherwise scoped query. It also accepts a
Task filter without Goal identity, bypassing the shared Scope shape rule.

The query first uses existing validate_scope. One SELECT projects the existing
Project/Goal/Task/Session identity columns plus body, under unchanged SQL filters and
sequence order. Before returning each decoded Usage, require exact equality of all
four body identities with that SAME row. collect returns an error for the complete
query if any row is inconsistent; it never returns an earlier partial vector. Optional
Goal/Task identity must preserve None exactly. The existing usage-table CHECK already
requires Task columns to have Goal columns; matching body identities preserves that
shape for supported schema3 history. No new redundant body-shape guard is added. The identity-mismatch and requested-scope refusal messages contain only static
codes/text, not the foreign body. The separate legacy decode-error component maps malformed Usage bodies to a
static refusal without a serde source chain. Other readers and full bounded
safe-code projection remain pending. No repair, audit, schema, owner or historical write.
Valid Task, Goal, Project and NoTask history retain existing representation/order.

Actual consumers: state integration history/reopen tests, missing-usage validation
and integer-boundary regression; Generic adapter persistence via those public APIs.
No production runtime aggregate, CLI/TUI metrics or qualified collector exists.
Other public read methods and arbitrary raw metadata are not covered by this slice.
Approved full raw-writer retirement, bounded safe projections/pagination/query
capabilities, writer epoch and native telemetry/benchmark gates remain mandatory.

Real query controls: valid Task/Goal/Project reads, equal Issue42 in distinct Projects,
six Task-row body/column corruptions (Project, Goal, Task, Session, missing Goal/Task),
valid Project/Goal-scoped rows and NULL-column/body-Some corruptions, invalid
Task-without-Goal scope, exact persisted usage/owner/Session/audit snapshots before and
after refusal, and original history restored/readable. SQLite corruption is an
isolated fixture, not an application attack or native ownership producer. Compiled
mutants remove each equality and the initial scope check at Store::usage, treat NULL
columns as wildcards, or decode optional columns non-optionally; real
integration controls must fail. Full default regression, fmt/Clippy/debug/release
builds, two independent source reviews and exact-source CI required before merge.
No Closes21; this is a limited bug fix to an unqualified legacy surface.
