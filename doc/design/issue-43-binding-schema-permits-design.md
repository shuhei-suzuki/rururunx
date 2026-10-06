# Binding10 schema and exact-write integration choices

This supplement implements §§3/3.1/8/9 of the approved
[managed binding design](issue-43-managed-binding-design.md). The isolated base is
`7558c59f592ef028b183d43ae2053a76e6b83491` (actual Runtime9 candidate, not main),
with Root's real canonical/snapshot/classification scaffold composed separately.
These choices add no native owner, prepared-input producer, live Driver or readiness.
The complete marker/binder/Native source composition requires independent review.

Eight new private tables accompany the existing append-only audit ledger. The
compiled table/column image in permits.rs is exhaustive. A private synchronous
`with_exact_permit` installs bounded exact OLD/NEW SQLite scalar images; INSERT
and DELETE have NULL absent-side images. The callback compares actual type/value
and complete raw body bytes, consumes each permission once and does no SQL,
filesystem access or whole-body hash. The RAII guard revokes on success, error and
unwind; nested permissions and poisoned state cannot grant. The producer checks
all planned permissions consumed before committing its own transaction. No permit
API is public, serializable or a substitute for PhaseSessionOwner/PreparedPhaseInput.
The Store retains its connection-local manager; current raw fixture writers have
an empty manager. Metadata, immutable-core and no-REPLACE constraints still apply
when an exact permission is present.

All Session Record writers atomically maintain the factual negative index through
AFTER triggers. A pure original-byte projection validates duplicate keys, typed
Record/Session members, scope/ID/version agreement and finite identity text. Invalid,
unknown, mismatched or over-4-MiB bodies map to a scoped malformed entry. Such an
entry cannot prove absence or allocate ownership. Direct index INSERT/UPDATE must
match the current indexed source and its exact pure projection; DELETE and existing
key INSERT/REPLACE refuse, including with recursive_triggers off. Session history
kind/ID/scope changes, nonconsecutive versions, replacement and deletion refuse.
No UNIQUE(provider,native_ref) erases duplicate historical/Lost identities.

Migration is one Immediate transaction. Actual9 objects and every old mutable
writer contract are checked before installation, together with all Binding10
reserved names. Older ordered migrations preserve their existing shape checks.
All old/new mutable table guards require exact compiled contract10. Existing active
or unfinished Workflow history receives LegacyHeld only; no marker/owner/input or
receipt is fabricated. Malformed Workflow envelopes or bodies refuse migration.
All Session/Workflow rows are scanned in 128-row metadata pages, including negative
rowids. At most 65,536 rows and 512 MiB of complete original encoded body lengths
are admitted. Full original lengths are counted before each individual body read;
over-4-MiB Session bodies are not copied but remain charged and project malformed.
Workflow bodies have their separate 8-MiB bound. Overflow, namespace/layout collision
or any failure rolls back the whole upgrade and leaves the old schema unchanged.
The installer returns measured rows, source bytes, pages, malformed Sessions and
held Workflows for controls; it does not write a new authority receipt.

Private audit kinds are the seven existing compiled factual classes. Their exact
header and canonical complete-body recipe are unchanged. SQL checks scope, immutable
marker, local ordinal, adjacent versions/body hashes, uniqueness of predecessor and
successor and finite class allowances: 99 claim/observation pairs, one binding, one
terminal decision, one closure, eight hold/clear links and 47 diagnostic links.
First predecessor is marker_digest. Complete digest continuity and typed allowed
Workflow deltas remain the genuine sealed Core planner's responsibility: the callback
does not hash an audit under the writer lock. Link encoding excludes its own digest.
The exact audit permission includes SQLite sequence; implicit autoincrement uses
NEW.sequence=-1 in BEFORE INSERT, while an explicit planned sequence matches literally.
No separate ledger head/mirror is written. Closure is the sole allowed reserved
link after phase_open becomes false; other classes require open phase. Quota
reservation/retirement and native/failure closure are separate genuine producers,
not enabled by DDL or private permission mechanics.

Primitive schema controls can authorize an exact retained historical contract row
and exercise permission/error/unwind/Replace mechanics. They do not construct an
owner/input/admission or stand in for actual producer qualification. The isolated
future11 contract control tests cached/current rejection without installing a fake
production ReviewRound11 migration. Actual8→9→10 composition, future real11, Native
and Driver integration, and all original Runtime9 regression failures remain gates.
