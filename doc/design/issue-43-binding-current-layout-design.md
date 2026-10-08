# Binding10 current-layout refusal correction

Status: proposed local bugfix design, no source approval or production change.
Inherits [Binding design](issue-43-managed-binding-design.md) §3/3.1/8/9 and
[schema/permits design](issue-43-binding-schema-permits-design.md).
Fixed inspected source: `4713fc80903e35dd76348a19d3ba7ae928f0f1ac`.

## Verified cause and chosen refusal

B10F-B-M1: Store::initialize_observed accepts initial current10 without SQL layout
validation, and locked-current10 checks only application_id. Actual prior Store
`00df66a966cd636e50dbc0453eea390b603cb898` also writes10 without the new
binding_record_native_ref trigger. Corrected code can therefore reopen that10
while the established Session identity remains mutable. Label/application alone
is insufficient. The correction explicitly refuses incompatible current10 before
effects; no11 allocation, automatic upgrade, whole-DDL reinstall or new privilege.

## Exact consumer checks

Private managed_binding::validate_current_layout compares the COMPLETE SQL-bearing
non-internal SQLite object inventory (name/type/table/SQL) with this binary's
compiled current contract. Include every base/Runtime/execution/Native/Source7/
Verifier/Binding table, index and domain/writer trigger. Missing, changed or extra
objects refuse. Trigger-name/presence-only checks cannot qualify changed actions.
Comparison must not erase string-literal semantics. SQLite-generated internal
sqlite_* objects/automatic indexes are not user contract SQL.

Build the reference with actual compiled installers on a separate empty in-memory
Connection. Never recursively Store::open or install DDL on the selected DB,
classify history, open an Owner/epoch, or derive expected layout from the inspected
user DB. A compiled reference is not Driver/native/input/settlement authority.
Before copying SQL, bound complete original lengths: ≤1024 objects, ≤2 MiB total,
≤64 KiB/object with checked arithmetic. Expected inventory meets the same caps;
overflow/missing/NULL never truncates into absence. Read in one SQLite transaction.

- Initial current10: coherent read transaction rechecks exact version/application
  and complete layout BEFORE journal_mode/WAL or returning Store.
- Initially observed0/9, then locked-current10: validate the same Immediate snapshot,
  then valid current10 exits without installing anything; old10 rejects atomically.
- Fresh/ordered migration: validate resulting complete layout in the existing TX
  before commit, retaining valid0/9→10 semantics as a producer assertion.

Refusal leaves original DB/version/rows/audit/layout unchanged. Function registration
and connection-local pragmas do not publish DB authority. Current/future cached
writer guards remain separate required controls. Same-UID direct DDL tampering is
not a secrecy/sandbox claim. This gate does not qualify native composition.

## Actual historical producer and causal controls

Generate old10 with REAL immutable00df Store execution in a separate owned,
account-free baseline harness committed before running. Only a test/export harness
may be added; all original production Store/DDL bytes remain unchanged. Record
exact source/build binding, DB/version/layout hashes and commands. Deleting a
current trigger cannot substitute for that producer. A fixture/inventory saved
from observed old Store output retains its provenance and is labelled reproduction;
it does not replace direct actual-old-Store controls.

1. Actual00df-created10 → corrected Store::open refuses; original DB bytes/version,
   schema/records/audit remain unchanged, including schema label10.
2. Hold the corrected real first0/9 observation seam. Actual00df Store initializes
   or migrates that path to10 before releasing it. The locked-current consumer
   refuses the old winner unchanged. Actual timeout/join outcomes are recorded.
3. New fresh10/reopen,9→10 and valid concurrent-current10 remain positive/no reinstall.
   Historical UUID and malformed-fact controls retain their meaning; no private
   Driver/owner/proof is seeded or inferred.
4. Compile separate omissions of normal and locked-current validation. Passing
   actual historical producer prerequisites must reach intended erroneous Store
   acceptance; setup/build failure earns no kill. Restore exact source and reprove
   controls. Missing/changed/extra current protection objects are additional
   corruption controls, not historical-baseline substitutes.

Source/quality/default regression and independent corrected-source gates remain.
Previous failures and whole Runtime/native/binder/MVP gaps stay open. This fixed
commit changes only this design file; frozen471 and Driver7db are untouched.

## Allocation trigger provider-column correction

Actual SQLite statement preparation exposed `binding_operation_allocation`
referencing `execution_units.provider`. ExecutionUnit's provider is stored in its
validated complete JSON body, not a relational column. SQLite accepts creating
the trigger but cannot compile a subsequent operation INSERT, independently of
whether a genuine publisher or its permission exists. Compare the same selected
provider with `json_extract(u.body,'$.provider')`; all original composed-contract,
scope, epoch, generation and exact mutation-permission guards remain mandatory.
No new provider column, authority row or permissive fallback is introduced.

On the real empty current Store, prepare EXPLAIN INSERT/UPDATE/DELETE for every
private table without stepping any statement. This compiles the actual protected
trigger programs and checks no table contents or change count advanced. Restoring
the bad column reference in a separately compiled mutation must fail this control
at SQL preparation, not at Rust compilation or a fabricated grant prerequisite.
This is SQL mechanical qualification only, never a managed positive publisher.
Current-layout validation continues to require exact protection objects; a Store
containing the previous defective trigger is refused under the changed current
layout, not silently rewritten or treated as a migration of current authority.
