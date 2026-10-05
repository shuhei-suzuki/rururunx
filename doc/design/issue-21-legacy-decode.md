# Issue21 legacy Usage decode-error projection component

STRICT limited follow-up under approved Requirements8 e001483 (safe error requirements at lines170–172) and Design4 8a4016a (safe malformed-state refusal). Initial base mainb8b1906, normally composed with environment main2c6ae9d at775dc7d.
Whole21 and private qualified query/native/epoch/raw-retirement producers remain OPEN.

The prior read identity component correctly rejects row/body mismatches, but its
shared decode() propagates serde detail under an anyhow source chain. Malformed
Usage fields can quote arbitrary body strings through alternate Display or Debug.
Both previous Source1 reviewers verified this behavior and marked it deferred.

Only Store::usage maps body decode failure to the fixed message
`invalid persisted usage snapshot`, with no source chain. Existing requested-scope
validation, row identity comparison, SQL filtering/ordering and valid legacy values
remain unchanged. No schema, writes, audit, owner, Session, native or other reader
change. SQL errors and unsupported/tampered schema are outside this narrow body
projection. This does not bound allocation or retire arbitrary valid raw metadata.

Actual public Store::usage controls use valid public legacy writers, then only
JSON-valid body-type corruption in an isolated SQLite fixture with schema checks
enabled. Input tokens, cost and Scope malformed fields carry a nonsecret synthetic
canary. Task/Goal/Project views must refuse with exact static Display/alternate Display, canary-free Debug and no source chain, and unchanged all Usage/audit rows and owners.
Original history is restored/readable. Reverting the projection, adding raw cause
context, or formatting raw cause into the message must fail the actual consumer;
compile/setup failures earn no kill. Full default regression/lint/build, two
independent source reviews and current exact-source CI gate a limited merge.
