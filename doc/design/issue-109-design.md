# Issue 109 design: Rust CI work selection and dependency cache

## Workflow shape

The owner's 2026-10-21 decision moves checks that can run on the PC to the PC.
`Rust CI` keeps its push-to-main and pull-request triggers, with one `check` job
on `ubuntu-latest`. Checkout and change detection always run.
For code changes, the sequence is Rust 1.91.1 setup, cache restore, fmt,
locked all-targets workspace clippy with `-D warnings`, then optional cache save.
Actions no longer runs tests, macOS or the release build. Those are verified on
the PC through `scripts/local-ci.sh`, with commands, SHA, test counts and outcomes
committed in `doc/verification/issue-<number>.md` and referenced from the PR.
Because the repository is public, PR and issue comments carry only IDs, titles,
severities and statuses (owner decision, 2026-10-21). The record is committed after
the verified SHA; when the diff from that SHA is only the record, the run stays
valid without a re-run. The trade-off is explicit: the independent GitHub check covers
fmt and clippy only; test and release evidence relies on the local run.
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
the automatic merge gate requires at least one successful check run. The single
`check` job can finish successfully after checkout/detection on a doc-only PR.
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
`refs/heads/main`, after successful `clippy` and without an exact cache hit, saves
the restored primary key. PRs never invoke save. GitHub cache branch scope lets
PRs read main caches but prevents a PR-scoped entry from being restored on main;
the workflow does not use `pull_request_target` or execute PR code with secrets.
A PR can change its own workflow, but cannot thereby write a main-scoped cache.
Cached files must be treated as public, with no credentials or secrets stored.

Actions never builds release targets, so it adds no `target/release` artifacts
to the cache. There is no post-job combined cache save.
`CARGO_INCREMENTAL: 0` applies to all Actions Cargo steps, limiting
incremental-cache growth. Immutable exact-key entries are reused, not refreshed
on each source change. Cargo fingerprinting rebuilds changed source/inputs;
fallback target contents may be stale and cost extra rebuilds. fmt and clippy
always execute regardless of cache hit. Caches are an optimization, not
verification receipts. Eviction, misses and restore failure can cause a cold
build without changing which checks run. An invalid cache can be evicted or the
namespace bumped; no remote cache operation is part of local verification.

## Local verification runner

Run `bash scripts/local-ci.sh` from the repository root of a clean macOS worktree
at the commit to be pushed/reviewed. Tracked changes and untracked files are
refused before any Cargo or Docker work. Repository paths containing a comma,
double quote or newline are also refused with a Docker mount CSV error, since
the archive and results paths become `--mount` values. A sentinel preserves
trailing newlines in both `git rev-parse --show-toplevel` and `pwd -P` output;
only the output delimiter and sentinel are removed. Both paths are checked
before any file creation, Cargo or Docker work. The script records the full HEAD,
UTC date, `uname -sm`, rustc/Cargo/Docker versions, then runs host fmt, locked
workspace/all-target clippy with warnings denied, locked workspace tests, and a
locked release workspace build. Every host Cargo invocation (including version
reporting) uses `env -u` for `GITHUB_TOKEN`, `GH_TOKEN`, `GH_ENTERPRISE_TOKEN`,
and `GITHUB_ENTERPRISE_TOKEN`. This removes those four tokens from Cargo and its
children; it does not claim to remove other host configuration or credentials.
Build scripts, proc macros and tests run with the user's privileges, credentials
and network on the host. The four-variable removal is not a sandbox. Run the
script only on commits produced by this workflow or whose diff has been reviewed;
external PR code is not executed locally without review or an isolated environment.

The `main-contained` step resolves `refs/remotes/origin/main` once with
`git rev-parse --verify --end-of-options 'refs/remotes/origin/main^{commit}'`,
records `origin/main <sha>` in the summary, and passes that SHA and the captured
HEAD SHA to `git merge-base --is-ancestor`. Thus a branch or tag named
`origin/main` cannot shadow the remote-tracking ref. Ancestry is PASS,
non-ancestry or an error is FAIL, and a missing/unresolvable ref is NOT RUN. It affects the
overall result like any other step. It does not fetch. Before the final local run
before merge, fetch first, merge or rebase current `origin/main` into the branch
if necessary, and run the full script on that clean HEAD. If main advances before
merge, repeat this integration and final local verification on the new HEAD.
`doc/`-only PRs need no local script: they change no code, and the Actions
documentation-only path still supplies a successful check.

The host's `linux-archive` step saves only tracked HEAD content using
`git archive --format=tar --output=<run>/source.tar <captured-head-sha>`. The verification
container mounts this tar read-only at `/source.tar`, never the worktree.
Ignored host `worktree/`, `.rrx/` and `target/` content is not visible. `/src` is a
fresh per-container tmpfs at a fixed path. Before extraction, the container's
main shell opens `/target/.local-ci.lock` on descriptor 9 and acquires an exclusive
`flock --exclusive 9`. Both open and lock failures exit the container immediately:
no later Linux step can PASS, and missing result markers fail the host run.
The main shell keeps the descriptor and lock until container exit. Linux
verification from different worktrees is serialized by this shared-target-volume
lock. No in-container Cargo command runs before it is acquired.
Linux setup extracts the tar there with
`tar --extract --touch`, giving every tracked source fresh mtimes instead of
archive timestamps. Extraction happens after the preceding container has released
the lock, so sources cannot predate that container's compilation start while
waiting for Cargo's own build lock. This invalidates source fingerprints left by
another worktree in the shared target cache, while the fixed source path permits
dependency reuse. A Docker VM clock moving backwards between runs remains an
accepted residual risk for mtime-based artifact reuse.

One `docker run --rm` verification container using `rust:1.91.1` runs Linux setup,
components, fmt, locked clippy with `-D warnings`, and locked workspace tests, in
that order and while holding the lock. It uses the host
UID/GID and label `rururunx-local-ci=1`, with its ID in `container.cid`.
Only the per-run Linux log directory is a writable host bind at `/results`.
No host environment is forwarded. Explicit container values are
`HOME=/home/local-ci`, `TMPDIR=/tmp`, `USER=local-ci`, `LOGNAME=local-ci`,
`CARGO_HOME=/cargo`, `CARGO_TARGET_DIR=/target/target` and `CARGO_INCREMENTAL=0`.
HOME, TMPDIR and CARGO_HOME each have a per-run tmpfs. Cargo's `config.toml`
cannot persist between runs. RUSTUP_HOME is not overridden. The image's
`/usr/local/rustup` and toolchain directory are world-writable (mode 777), so the
verification UID can write them. Its installed components are cargo, rust-std
and rustc; rustfmt and clippy are absent. After extraction, `linux-components`
explicitly runs `rustup component add rustfmt clippy` from `/src`, honoring
`rust-toolchain.toml` rather than relying on rustup auto-install defaults.
Each run installs the components into the container's own writable layer,
which `--rm` discards. No toolchain changes or caches persist between runs.
Fixed USER/LOGNAME supply a non-secret identity
without depending on a passwd entry for the host UID. GitHub tokens are not passed.
The `linux-image` step records `docker image inspect --format
'{{index .RepoDigests 0}}' rust:1.91.1`, falling back to the image ID if no repo
digest exists, in both `metadata.log` and the summary.

Three named volumes persist: `rururunx-local-ci-registry` at
`$CARGO_HOME/registry`, `rururunx-local-ci-git` at `$CARGO_HOME/git`, and
`rururunx-local-ci-target` at `/target`. A separate `linux-cache-setup` helper
container uses the same image, `--rm` and label, records `cache-container.cid`,
and mounts only those volumes. It runs as root solely to set the three mount
roots to mode 1777 (no recursive chmod), allowing the host UID to use new volumes.
It mounts no host files or source and uses per-run HOME/TMPDIR tmpfs.
Both containers are auto-removed. The volumes are standing project caches,
not leftovers subject to generic task cleanup, and are not verification evidence.
Cargo's package-cache lock files live at the top of the per-run CARGO_HOME,
so those locks alone do not protect the shared registry/Git volumes. The
`/target/.local-ci.lock` lock covers all in-container Cargo commands, including
fmt, clippy and tests, serializing access to these shared caches as well as target.
They are trusted only for trusted commits: after running anything untrusted,
ensure no run uses them and remove all three with
`docker volume rm rururunx-local-ci-registry rururunx-local-ci-git rururunx-local-ci-target`.
The old `rururunx-local-ci-cargo` volume is no longer mounted or used by the script.
The first run needs network access for the image. Every run needs network access
for rustfmt/clippy installation, and dependencies need it on cache misses;
cache hits do not skip checks. Overlapping runs of the same worktree/summary
are unsupported even though Linux verification across worktrees is serialized.

Each run has a fresh `.rrx/local-ci/<sha>/run.XXXXXX/` log directory, with Linux
step logs and fresh status markers in its `linux/` subdirectory. Even when setup or an earlier
check fails, later checks are attempted; lock open/acquisition failure instead
exits before any Linux step. Stdout and `.rrx/local-ci/<sha>.txt`
contain the same summary: metadata and per-step PASS/FAIL/NOT RUN, seconds and
full HEAD. A failed Docker launch or a missing/invalid Linux status marker cannot
pass. The parser requires the entire file to be one line `PASS <digits>` or
`FAIL <digits>` followed by exactly one newline, with no other bytes. It requires
a successful newline-terminated read, checks the ASCII fields, and compares byte
length to the accepted line plus newline (also rejecting embedded NUL bytes).
Failed reads, missing newlines, extra lines/fields, empty files and non-digit
durations produce NOT RUN and fail the run. Any marker can still be forged by
malicious code under test, so the commit trust rule is required. Fresh run paths
prevent stale markers from passing. A final check verifies
that HEAD and the worktree remain unchanged. Environment variables are never
dumped. The script exits nonzero for failures or unexecuted checks.

`--host-only` reports all Linux steps as NOT RUN (including `linux-components`
between `linux-setup` and `linux-fmt`) and exits nonzero unless paired
with `--allow-partial`. The latter acknowledges only that explicit skip; it does
not mask host failures or allow a non-macOS host. A partial run is diagnostic and
does not satisfy `CLAUDE.md`'s complete pre-push requirement. The script is invoked
with Bash and does not require an executable bit.

## Round-1 findings retained

- M1 remains fixed: main pushes have unique `run_id` concurrency groups, including
  while pending; PR runs share a PR-number group and cancel superseded runs.
- M2 (release build before merge) is now met by the local script's mandatory
  `cargo build --locked --release --workspace` and final-SHA PR evidence instead
  of Actions. A green GitHub check alone does not establish release success.
- L1 remains fixed: the orchestrator's normal actionlint success on `92ae7ad6`
  is recorded below separately from any incomplete sandbox rerun.
- L2 remains fixed: all actions, including the toolchain action, use full SHA
  pins. Cache-contract changes, including action-pin changes, bump the namespace.

## Impact analysis

- Changed workflow symbols: workflow concurrency, `check.env.CARGO_INCREMENTAL`,
  checkout depth/pin, `changes`/`code_changed`, `cache`/`cache-hit`/
  `cache-primary-key`, `clippy.outcome` (replacing `test.outcome`), step guards
  and cache paths/keys. These outputs are consumed only by later steps in the
  same job. The Cargo
  environment variable is consumed by Cargo/rustc; it adds no runtime setting.
  GitHub supplies `GITHUB_EVENT_NAME`, `RUNNER_TEMP` and `GITHUB_OUTPUT`; no
  caller-supplied script or event text is evaluated.
- `.github/workflows/ci.yml` is consumed by GitHub Actions for the two existing
  events. Check-run names change from `check (ubuntu-latest)` and
  `check (macos-latest)` to `check`. Consumers are GitHub branch protection/rulesets
  that require named contexts, the owner's automatic merge gate, and verification
  receipt readers. Any required named context must migrate to `check` before
  merge; the owner's at-least-one-success condition still holds for code and
  doc-only PRs. Remote configuration is not inspected or changed in this offline
  task, so the orchestrator must check it before merge. Repository searches find
  the old names in historical `doc/verification/` receipts and this issue's docs,
  not executable in-repository consumers.
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
- `scripts/local-ci.sh` is a new tracked developer entry point consumed by the
  policy in `CLAUDE.md`. Its label identifies the auto-removed containers; the three
  named volumes are intentional standing caches, not task cleanup leftovers.
  `/results` is the only writable bind mount and is scoped to ignored per-run
  logs. No Docker socket, host home or host credential directory is mounted.
  The host's four removed GitHub token names are consumed by Cargo children and
  native-agent/GitHub helpers they might launch; removal applies uniformly to
  all host Cargo steps, while fixed Linux identity values support tests.
  Local caches/results cannot satisfy the remote successful-check requirement.
- Actions no longer tests main pushes or the synthetic PR merge result. Testing
  branch HEAD alone would miss integration defects with newer main changes.
  `CLAUDE.md` therefore requires fetch and integration of current `origin/main`
  before final local verification/merge; `main-contained` enforces ancestry of
  the locally known ref. The final local run covers the integrated code, while
  the Actions check still proves only fmt/clippy. A missing/stale local ref is
  not proof of current main coverage: missing fails and freshness requires fetch.
- Cargo and tests consume the changed Linux HOME/TMPDIR/CARGO_HOME and fixed
  `/src` path. HOME/TMPDIR remain writable per run; source mtimes refresh each
  run after taking the shared-target lock, which also protects shared registry/Git
  caches despite per-run Cargo package-cache locks. Only registry/Git/target
  persist. No Cargo config, toolchain changes or rustup cache persists;
  rustfmt/clippy installation needs network every run. The new components
  step is consumed by both the result-marker loop and host-only NOT RUN list.
  Ref ancestry and archive creation consume captured SHAs, not ambiguous or
  moving short refs; the main SHA also appears in the summary. Repository and
  physical-directory paths are checked without trimming path newlines before
  those paths reach log creation or Docker mount arguments.
  Host build scripts/proc macros/tests retain user privileges,
  credentials and network except for the four removed token variables, so
  reviewed-commit trust is a necessary security boundary. Linux markers are
  reporting aids, not tamper-proof evidence.
- Rust source, Cargo manifests/lockfile/toolchain file, CLI/runtime behavior,
  schema, authorization, dependencies and deployment remain unchanged. The
  README include remains covered by local tests and Actions code detection.
  The absence of `doc/` readers
  is the basis for the narrow documentation exemption.
- Local helpers under ignored `.rrx/ci/` are not workflow dependencies. They
  compare the tested snippet byte-for-byte with the inline workflow script,
  exercise real merge repositories and delete only their owned temporary tree.
  The new tracked Bash script is separate from these detection-test helpers.

## Local verification

Run `/opt/homebrew/bin/actionlint .github/workflows/ci.yml` and
the fallback `/opt/homebrew/bin/actionlint -shellcheck= .github/workflows/ci.yml`,
`bash -n scripts/local-ci.sh`, `/opt/homebrew/bin/shellcheck scripts/local-ci.sh`,
and `python3 .rrx/ci/detect_test.py`. The latter compares the shared reference snippet
with the workflow and tests doc-only, code, mixed, cross-boundary rename, empty
diff, non-merge HEAD and README-only cases. Additional controls cover pushes
and a failed diff. Its mutation control changes the snippet to classify every
change as doc-only and requires the same assertions to reject it. Throwaway
repositories exist only under `.rrx/ci/tmp/` and are removed on exit. No network,
hosted Actions execution or writes to the working branch's Git state are needed
for these static/detection checks. Actual Cargo and Docker execution is reserved
for the orchestrator on a clean committed SHA; syntax/lint success does not
establish that the host/container checks have passed.
An ignored `.rrx/ci/marker_test.py` extracts the exact marker-parser function
byte-for-byte from the script and tests valid PASS/FAIL and malformed files,
including missing newline, extra line/field, empty and non-digit cases. Its
negative control removes whole-file byte-length validation and confirms that
the extra-line case detects the resulting false acceptance.
Run `python3 .rrx/ci/marker_test.py` and `python3 .rrx/ci/fix4/path_test.py`.
The latter extracts the path-handling code byte-for-byte and checks a normal path
plus comma, double quote, embedded newline, one trailing newline and two trailing
newlines. It also checks physical-directory rejection independently of the Git
path. Its negative control restores the old command substitutions and confirms
that trailing-newline paths are accepted.
All temporary test directories are created beneath `.rrx/ci/fix4/` and removed
on exit. These checks do not execute Cargo or Docker.

The orchestrator ran `/opt/homebrew/bin/actionlint .github/workflows/ci.yml`
with ShellCheck enabled to completion on `92ae7ad6`; it passed. After the
round-1 fixes based on that HEAD, `python3 .rrx/ci/detect_test.py` passed all nine
detection controls, rejected the all-doc-only mutant in six cases, and removed
its temporary tree. The workflow detection snippet remains byte-for-byte equal
to the tested reference.
The normal actionlint invocation was also re-run after the fixes in the Sol
sandbox, but stalled without output and was interrupted (exit 130). That rerun
did not complete; the orchestrator's passing result above applies to `92ae7ad6`.

For the local-first revision based on `85ea0c64`, normal actionlint again stalled
without output in the Sol sandbox and was interrupted (exit 130); it is not a
passing result for this revision. `actionlint -shellcheck=` passed, as did
`bash -n scripts/local-ci.sh` and ShellCheck on the script. Detection tests passed
all nine controls, rejected the all-doc-only mutant in six cases, confirmed the
inline snippet byte-for-byte, and removed their temporary repositories. The
orchestrator must complete normal actionlint and the actual local Cargo/Docker
run on the final committed SHA before recording full verification in
`doc/verification/issue-109.md`.
