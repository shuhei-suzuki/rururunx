# Issue 43: original Workflow ledger currency

The current-plan reader requires the actual OriginalMarker and retained owner.
It uses one separate query-only snapshot, never a new RuntimeOwner or current
SQL operation as a substitute original. Complete original P/G/post-Task/Context/
lock bytes remain pinned. The original open operation and prepared template are
exact; Native owner/readiness facts can advance only through their separate
private producers.

All compiled factual links for this operation are read in sequence, including
foreign/malformed scope conflicts rather than filtering them into absence. The
reader refuses a257th link or any payload above4096 bytes. It verifies complete
unambiguous encoding, original scope/Workflow/generation/attempt/phase/Context,
original marker identity, consecutive Workflow versions, preceding body digest,
preceding ledger digest, and first-link-only session binding. Each link hashes
the complete immutable payload under the specified domain with final NUL; the
current complete Workflow body must equal the final chain endpoint digest.

The current transaction validator consumes this private nongrant read product
and the same actual original plan. It rechecks complete current Workflow and
original owner/context/lock bytes, exact open operation/preparation, bounded link
count and immutable endpoint sequence/metadata/original encoded bytes. It relies
on the already protected append-only audit history, so it does not rehash256
links or reserialize payloads under SharedStore. A new head requires a new
coherent read before any still-unconsumed effect, never new original pins.

This validates ledger currency only. It is not Native eligibility, a consumed
input or terminal proof, an AlreadyBound decision, or independent validation of
every typed factual projection. Each protected Native consumer must still check
its actual retained PhaseLaunch/owner/input/terminal and exact current pair/Unit/
Session facts in the same transaction. Every factual writer must validate its
own complete permitted old/new Workflow delta before its exact permission is
installed. Those producer/consumer connections, active-attempt eligibility,
normal/late record-only binder, other factual ports and genuine positive chain
controls remain incomplete. No availability or success claim is enabled here.
