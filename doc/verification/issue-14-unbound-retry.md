# Issue14 unbound native retry component

Requirements3 approved at1463d37: TWO independent APPROVE, noCHM/blockers.
Design1 candidate NOT APPROVED; no implementation/source qualification yet.
Open Issue14 explicitly requires this follow-up from Issue41. Current main efe9774
still allows explicit Failed+dispatch_started+session_id=None retry when the
Session loop has nothing to reject. Existing characterization is evidence of the
unsafe release behavior, not native recovery proof. No schema/permission/timeout
change proposed. Whole14/Scheduler/recovery/MVP remain OPEN.

Required gates: independent requirements review, component design review, actual
held-start Executor/Reviewer consumers and valid retry controls, compiled causal
mutants, default regressions/lint/build/affected release tests, independent source
reviews, current both-OS CI/actual source identity, limited PR merge/cleanup.

Requirements1 at535452c: BOTH independent native request_changes, each verified
High pre-marker retry/control omission and Medium equivalent EvidencePort mutant,
enforcement layer and availability disclosure. Both actual selected wrappers
closed/cleaned; no source/design approval. Coordinator inspected actual before-
marker registry/capability fail, marker/Session immutability, Store own_found and
closure/terminal fences. Requirements2 chooses dual Engine+Store enforcement with
actual positives/directStore/combinedmutants and explicit no-escape consequence.
All findings/dispositions retained. Normal no-conflict main63 composition completed
after review closure. Requirements2 re-review pending; no implementation.

Requirements2 at1f3b01e: A request_changes with one verified Medium proofgap;
B approve with the same Medium nonblocking. No Critical/High remains, but gate
notapproved. Both selected wrappers closed. Requirements3 names actual held
Running direct Store closures/generation with validcontext, preserves same-owner
binding/fail/decision and classifies Failed-generation falsecredit. Grok returned
startErr versus bound post-start actor failure split corrected; Generic-shaped
unbound terminal row control added, observation writer inventoried and header
updated. Source unchanged; independent Requirements3 delta re-review pending.

Requirements3 at1463d37: BOTH independent native APPROVE, zeroCHM and blockers,
selected wrappers actuallyclosed/cleaned. Three distinct optional Lows verified: Grok
bound outcome depends on binding commit, row-resolution mutation attribution per
guard, and co-located valid pre-marker raw-builder controls. Design1 records finite
precision/disposition with no WHAT change. Requirements amended only to qualify
the binding sentence; source/test/build untouched. Design1 independent review next;
whole14/MVP/native/F1 and genuine recovery producer remain OPEN.
