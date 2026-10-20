# Issue 109 design: Rust CI work selection and dependency cache

## Workflow shape

`Rust CI` keeps its push-to-main and pull-request triggers and the `check` matrix
on `ubuntu-latest` and `macos-latest`. Checkout and change detection always run.
For code changes, the sequence is Rust 1.91.1 setup, cache restore, fmt,
all-targets workspace clippy with `-D warnings`, locked workspace tests, optional
cache save, then a locked release workspace build on both PRs and pushes. Clippy and
tests already compile the targets, so a separate debug build adds no coverage.
Every heavy step has a step-level code-change guard; the job itself is never
skipped. `contents: read` and checkout's disabled credential persistence remain.
Every action uses a full 40-character SHA pin; no third-party action is added.
The existing toolchain action uses
`8632757d5a758c452f724e9d837848c694a7bb48`, which installs Rust 1.91.1.

The workflow concurrency group combines `github.workflow` with the PR number,
falling back to the unique `github.run_id` for pushes. Cancellation is enabled
only for `pull_request`: superseded PR runs share a group and are cancelled.
Main push runs never share a group, so neither pending nor in-progress main
push runs are cancelled by concurrency.

## Documentation-only detection

Checkout fetches depth 2. On `pull_request`, the merge commit must have a second
parent, and `git diff --name-only --no-renames HEAD^1 HEAD --` must succeed and
produce a nonempty list. Only when every listed path begins with `doc/` does the
step emit `code_changed=false`. Disabling rename detection lists both endpoints
of moves across the documentation boundary. Missing parents, failed diffs and
empty lists retain the initial `true`; every push retains `true` as well.

The Bash script reads `GITHUB_EVENT_NAME`, writes its diff to
`RUNNER_TEMP/ci-changed-paths`, and appends the boolean to `GITHUB_OUTPUT`.
No event payload expression is interpolated into shell code. Paths are read
literally with `IFS= read -r` and are never evaluated as commands. Git-quoted
unusual names conservatively run the checks instead of widening the exemption.

Trigger `paths`/`paths-ignore` filters and job-level conditions are not used:
the automatic merge gate requires at least one successful check run. Both
matrix jobs can finish successfully after checkout/detection on a doc-only PR.
This success means documentation classification, not that Rust checks ran.
`README.md` is deliberately outside the exemption:
`crates/rrx/tests/cli.rs:148` uses `include_str!("../../../README.md")`.
`rg -n 'doc/' crates` found no readers; the README reference was confirmed with
`rg -n 'README\.md' crates`. Adding a future code dependency on `doc/` requires
revisiting this optimization.

## Cache scope and trust

Both cache operations use first-party `actions/cache/restore` and
`actions/cache/save` at `55cc8345863c7cc4c66a329aec7e433d2d1c52a9` (v6.1.0).
Paths are `~/.cargo/registry/index/`, `~/.cargo/registry/cache/`,
`~/.cargo/git/db/` and `target/`. The primary key is
`cargo-v1-${{ runner.os }}-${{ runner.arch }}-${{ hashFiles('rust-toolchain.toml', 'Cargo.lock') }}`;
the fallback prefix omits only the hash. OS and architecture isolate native
artifacts, and toolchain/lock changes select a new primary key. `cargo-v1` is the
cache namespace/version and must be bumped for any cache-contract change,
including changes to action pins.

Restore runs for every code-check job, including fork PRs. Only a push to
`refs/heads/main`, after successful tests and without an exact cache hit, saves
the restored primary key. PRs never invoke save. GitHub cache branch scope lets
PRs read main caches but prevents a PR-scoped entry from being restored on main;
the workflow does not use `pull_request_target` or execute PR code with secrets.
A PR can change its own workflow, but cannot thereby write a main-scoped cache.
Cached files must be treated as public, with no credentials or secrets stored.

Save precedes the release build, so this new namespace contains no
`target/release` artifacts. There is no post-job combined cache save that could
capture them later. `CARGO_INCREMENTAL: 0` applies to all Cargo steps, limiting
incremental-cache growth. Immutable exact-key entries are reused, not refreshed
on each source change. Cargo fingerprinting rebuilds changed source/inputs;
fallback target contents may be stale and cost extra rebuilds. fmt, clippy and
tests always execute regardless of cache hit. Caches are an optimization, not
verification receipts. Eviction, misses and restore failure can cause a cold
build without changing which checks run. An invalid cache can be evicted or the
namespace bumped; no remote cache operation is part of local verification.

## Impact analysis

- Changed workflow symbols: workflow concurrency, `check.env.CARGO_INCREMENTAL`,
  checkout depth/pin, `changes`/`code_changed`, `cache`/`cache-hit`/
  `cache-primary-key`, `test.outcome`, step guards and cache paths/keys. These
  outputs are consumed only by later steps in the same matrix job. The Cargo
  environment variable is consumed by Cargo/rustc; it adds no runtime setting.
  GitHub supplies `GITHUB_EVENT_NAME`, `RUNNER_TEMP` and `GITHUB_OUTPUT`; no
  caller-supplied script or event text is evaluated.
- `.github/workflows/ci.yml` is consumed by GitHub Actions for the two existing
  events. Workflow/job/matrix names remain unchanged: `check (ubuntu-latest)`
  and `check (macos-latest)`. External required-check configuration is unchanged;
  the orchestrator's automatic merge gate still sees successful matrix checks
  on doc-only PRs. Remote configuration is not inspected or changed here.
- Repository references to the workflow in
  `doc/verification/*ci-provenance*.json` bind past runs by workflow blob and
  retain historical step names. Other `doc/verification/` gate/check receipts
  and design references describe past verification; none schedules the current
  workflow, so no historical evidence is rewritten.
- `doc/design/master/architecture.md` already identifies the CI toolchain in
  its Rust foundation section. It is synchronized with the current CI shape;
  other master mentions describe historical evidence or product verification,
  not this Actions configuration. The requirements and this issue design
  describe the changed contract without changing product APIs.
- Rust source, Cargo manifests/lockfile/toolchain file, CLI/runtime behavior,
  schema, authorization, dependencies and deployment remain unchanged. The
  README include remains covered by full checks. The absence of `doc/` readers
  is the basis for the narrow documentation exemption.
- Local helpers under ignored `.rrx/ci/` are not workflow dependencies. They
  compare the tested snippet byte-for-byte with the inline workflow script,
  exercise real merge repositories and delete only their owned temporary tree.
  No new tracked executable or test infrastructure is introduced.

## Local verification

Run `/opt/homebrew/bin/actionlint .github/workflows/ci.yml` and
`python3 .rrx/ci/detect_test.py`. The latter compares the shared reference snippet
with the workflow and tests doc-only, code, mixed, cross-boundary rename, empty
diff, non-merge HEAD and README-only cases. Additional controls cover pushes
and a failed diff. Its mutation control changes the snippet to classify every
change as doc-only and requires the same assertions to reject it. Throwaway
repositories exist only under `.rrx/ci/tmp/` and are removed on exit. No network,
hosted Actions execution or writes to the working branch's Git state are needed.

The orchestrator ran `/opt/homebrew/bin/actionlint .github/workflows/ci.yml`
with ShellCheck enabled to completion on `92ae7ad6`; it passed. After the
round-1 fixes based on that HEAD, `python3 .rrx/ci/detect_test.py` passed all nine
detection controls, rejected the all-doc-only mutant in six cases, and removed
its temporary tree. The workflow detection snippet remains byte-for-byte equal
to the tested reference.
The normal actionlint invocation was also re-run after the fixes in the Sol
sandbox, but stalled without output and was interrupted (exit 130). That rerun
did not complete; the orchestrator's passing result above applies to `92ae7ad6`.
