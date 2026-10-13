# Process descriptor hygiene requirements

Issue: <https://github.com/shuhei-suzuki/rururunx/issues/89> (P4 only).

Exec children must not retain unrelated pipe writers inherited during concurrent
launches. Preserve EOF delivery without serializing launches or changing process
custody, deadlines, cancellation, marker/CAS/Task.version authority or classification.

The approved scope is a separate `rrx-spawn-hygiene` crate and single P4 PR.
All std/Tokio Command spawn/output/status paths (production, tests, examples and
process-tracker) consume Command through six safe functions. rrx keeps unsafe
forbidden. Mark fd >= 3 CLOEXEC without closing descriptors; preserve stdio and
std's internal error pipe. Refuse exec if complete hygiene is unprovable. No
fallback scan is permitted. Unsupported targets refuse launch.

Required acceptance: C1/C2a–g, V1–V9, OW1/OW2; fmt/all-target compile/clippy and
affected regressions; independent implementation/security reviews; M06 macOS
50 trials twice (100+ observable, zero c_s3d4/m2_a failures and observable B_END);
V-F main/P4 Linux/macOS two jobs × eight repeats. Retain all attempts, including
unobservable failed trials. Increased F1 frequency blocks merge. The parent owns
measurement/CI, publication, owner approval and merge. These gates are not yet
certified by initial source implementation.

This PR is partial issue work and must not close issue 89. P4 merges before F3.
F1/F3/F4 belong to [CI 品質の収束](https://claude.ai/code/session_018R7Q39hySMcnivGdtBcyJC).
The known Tokio post-spawn registration Err path may leave a child; P4 preserves
the existing classification and does not claim that limitation resolved.
