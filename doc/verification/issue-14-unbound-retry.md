# Issue14 unbound native retry component

Requirements1 NOT APPROVED. No implementation or qualification yet.
Open Issue14 explicitly requires this follow-up from Issue41. Current main2c6ae9d
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
