# Issue 109 requirements: reduce redundant Rust CI work

## Purpose and scope

Run checks on the PC wherever possible, keeping one independent GitHub Actions
check for the owner's automatic merge gate. Change `.github/workflows/ci.yml`,
add `scripts/local-ci.sh`, and document their behavior consistently with
`CLAUDE.md`. Rust CI has one Ubuntu job named `check`, including for doc-only PRs.
Actions verifies fmt and clippy only; tests, macOS and the release build are
verified locally before push/merge, with results recorded in the PR.

Old `claude/*` measurement workflow branch cleanup and required-check configuration
are outside this change. No Rust source, dependency, deployment or
runtime behavior changes belong here.

## Acceptance criteria

- One `check` job on `ubuntu-latest` runs on every pull request and push to main, without trigger path filters or a job-level skip. Consumers of the old matrix check names must use `check` if they require a named context.
- PR merge checkouts use depth 2 and compare `HEAD^1` to `HEAD` with rename detection disabled; only nonempty changes entirely under `doc/` skip heavy steps.
- Missing merge parents, failed diffs and empty diffs run all code checks; every push also runs them.
- README-only changes run code checks because the CLI tests compile `README.md` into the test binary.
- Actions code checks retain Rust 1.91.1, fmt and locked workspace/all-targets clippy with warnings denied. Actions does not run tests, macOS or a release build.
- First-party SHA-pinned cache restore/save covers Cargo registry index/cache, Git databases and `target/`, with OS/architecture/toolchain-and-lockfile keys and matching prefix restoration.
- Only main pushes with successful clippy and no exact cache hit may save; PRs including forks never save and no release build runs in Actions.
- Superseded PR runs are cancelled by workflow and PR number, while pushes to main are never cancelled.
- `bash scripts/local-ci.sh` requires a clean worktree and repository-root invocation, records full HEAD/date/OS/tool versions, and runs macOS fmt, locked clippy/tests and a locked release workspace build. All host Cargo calls remove `GITHUB_TOKEN`, `GH_TOKEN`, `GH_ENTERPRISE_TOKEN` and `GITHUB_ENTERPRISE_TOKEN`.
- The same script runs Linux fmt, locked clippy/tests in one `docker run --rm` container using `rust:1.91.1`, the host UID/GID, a read-only `/src`, fixed `HOME`/`USER`/`LOGNAME`, no forwarded host environment, label `rururunx-local-ci=1`, and two documented standing named caches prefixed `rururunx-local-ci-`.
- All steps are attempted despite earlier failures. Per-step status and seconds, HEAD and metadata appear in stdout and `.rrx/local-ci/<sha>.txt`; step logs are retained under `.rrx/local-ci/<sha>/`. A failed or unexecuted step fails the run. `--host-only` explicitly reports Linux NOT RUN and fails unless `--allow-partial` is also given; partial evidence does not meet the pre-push verification requirement.
- Release compilation must pass locally for the reviewed SHA before merge; the PR records local commands, SHA, test counts and outcomes. GitHub's independent successful check proves fmt/clippy only (or doc-only classification).
- Normal local actionlint (with ShellCheck enabled) and real-merge detection tests pass, and an all-doc-only detection mutant fails the tests.
- The local script passes `bash -n` and ShellCheck; actual Cargo/Docker execution is performed by the orchestrator on the host.

## Constraints

This is a public repository: no secrets, tokens, real resource IDs or raw logs
are recorded. All actions are pinned by full 40-character commit SHA. Cache actions
use `55cc8345863c7cc4c66a329aec7e433d2d1c52a9` (v6.1.0); no third-party
action is added. The existing toolchain action is pinned to
`8632757d5a758c452f724e9d837848c694a7bb48`, which installs Rust `1.91.1`.
Keep `permissions: contents: read`, checkout `persist-credentials: false`, both
triggers and the byte-identical documentation detection step. Set job-wide
`CARGO_INCREMENTAL: 0`. Cache misses and
uncertain path classification must never remove a code check.
