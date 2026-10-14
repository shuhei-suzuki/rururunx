# Process descriptor hygiene requirements

Issue: <https://github.com/shuhei-suzuki/rururunx/issues/89> (P4 only).
Platform scope: macOS First per the
[owner policy record](https://github.com/shuhei-suzuki/rururunx/issues/89#issuecomment-6159598215).

macOS exec children must not keep unrelated pipe writers they inherit during
concurrent launches. EOF delivery must still work. Launches are not serialized.
Process custody, deadlines, cancellation, marker/CAS/#19 authority and
classification do not change. Task.version is not authority.

The approved scope is a separate `rrx-spawn-hygiene` crate in a single P4 PR.
Every std/Tokio Command spawn/output/status path consumes its Command through six
safe functions: production, tests, examples and process-tracker. The existing
call-site migrations and OwnedProcess consumers stay as they are. rrx keeps unsafe
forbidden. Each function keeps the original std/Tokio operation unchanged:

- stdio defaults and overrides;
- for Tokio output/status, the synchronous spawn before the future is returned;
- cancellation, kill_on_drop and the process group;
- Child custody, caller locks and errors.

Platform handling:

- macOS installs the existing preallocated libproc/fcntl pre_exec hygiene. It marks
  fd >= 3 CLOEXEC without closing anything, and keeps stdio and std's internal
  error pipe. If complete hygiene cannot be proven, exec is refused. There is no
  fallback scan.
- Linux (every target_env) calls the original operation directly, as on main
  `b8344ab9`: no additional hygiene pre_exec callback, close_range or probe, and no change
  to which launches fork. No Linux descriptor-hygiene guarantee or full P4 qualification
  is claimed.
- All other targets refuse launch with ENOSYS. This is unchanged.

Required acceptance on the final source:

- macOS: C1, C2a–c, C2e–g, LISTFDS, OW1/OW2, the six public-API controls, the
  macOS-relevant V1–V9 mutants, and the 17 typed EOF fault/normal pairs with actual
  child collection. Also fmt, all-target compile/Clippy and affected regressions.
  M06 50 trials × 2 on the final source: 100+ observable attempts, zero c_s3d4/m2_a
  failures and no observable B_END. V-F main/P4 macOS: two jobs × eight full
  rrx-lib runs, judged by a predicate fixed before classification.
- Linux: six-API delegation-equivalence controls, C2a/C2b/C2g, OW1/OW2, the EOF
  pairs with a test-owned CLOEXEC baseline, all-target Clippy and the required CI.
  These are regression controls, not hygiene evidence.
- Both: independent implementation and security reviews. Every attempt is kept,
  including unobservable failed trials. Unknown is never counted as non-increase.
  Merge is blocked by any of: a higher macOS F1 frequency, a shared severe defect,
  or a Linux safety regression.

Platform separation is implemented from the approved `aed6f6b4` amendment;
its local macOS formatting, workspace/all-targets warnings-denied Clippy and
20 hygiene tests passed at `e8359628`. Linux controls, final macOS campaigns and
independent implementation reviews remain unqualified. Existing historical
proofs are not relabeled to this source.

The parent owns measurement, CI, publication, owner approval and merge. Each
specific PR merge needs owner approval. None of these gates is certified yet.

Deferred, not erased: Linux hygiene application, full Linux qualification,
stabilization, high-load work, GNU V-F reassessment and the private forced-fork
fixture repair. Earlier GNU support-baseline, C2d/V3 and Linux V-F requirements are
superseded by the design's platform scope amendment. Their evidence stays recorded.

This PR is partial issue work and must not close issue 89. P4 merges before F3.
F1/F3/F4 belong to [CI 品質の収束](https://claude.ai/code/session_018R7Q39hySMcnivGdtBcyJC).
A known Tokio post-spawn registration Err path may leave a child. P4 keeps the
existing classification and does not claim that limitation is resolved.
