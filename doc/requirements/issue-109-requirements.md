# Issue 109 requirements: reduce redundant Rust CI work

## Purpose and scope

Reduce GitHub Actions work without weakening checks that protect code changes.
Change `.github/workflows/ci.yml` and document its behavior. Preserve the Rust CI
workflow, `check` job and Ubuntu/macOS matrix check names so existing consumers
continue to receive successful check runs, including for documentation-only PRs.

Old `claude/*` measurement workflow branch cleanup and required-check configuration
are outside this change. No Rust source, dependency, test policy, deployment or
runtime behavior changes belong here.

## Acceptance criteria

- Both OS jobs run on every pull request and push to main, without trigger path filters or a job-level skip.
- PR merge checkouts use depth 2 and compare `HEAD^1` to `HEAD` with rename detection disabled; only nonempty changes entirely under `doc/` skip heavy steps.
- Missing merge parents, failed diffs and empty diffs run all code checks; every push also runs them.
- README-only changes run code checks because the CLI tests compile `README.md` into the test binary.
- Code changes retain Rust 1.91.1, fmt, workspace/all-targets clippy with warnings denied and locked workspace tests on both OSes.
- First-party SHA-pinned cache restore/save covers Cargo registry index/cache, Git databases and `target/`, with OS/architecture/toolchain-and-lockfile keys and matching prefix restoration.
- Only successful main push tests may save a missing cache, before release compilation; PRs including forks never save and release artifacts never enter the cache.
- Superseded PR runs are cancelled by workflow and PR number, while pushes to main are never cancelled.
- Release workspace builds remain locked and run only on pushes; the redundant debug workspace build is removed.
- Local actionlint and real-merge detection tests pass, and an all-doc-only detection mutant fails the tests.

## Constraints

This is a public repository: no secrets, tokens, real resource IDs or raw logs
are recorded. First-party actions are pinned by full commit SHA. Cache actions
use `55cc8345863c7cc4c66a329aec7e433d2d1c52a9` (v6.1.0); no third-party
action is added. The existing toolchain action remains at `1.91.1`.
Keep `permissions: contents: read`, checkout `persist-credentials: false`, both
triggers and both OSes. Set job-wide `CARGO_INCREMENTAL: 0`. Cache misses and
uncertain path classification must never remove a code check.
