# Issue #1 verification

Implementation revision: `64ffec7`. PR: https://github.com/shuhei-suzuki/rururunx/pull/28.
Host: macOS 26.6.2 arm64; Rust 1.91.1. Workflow: STRICT after configuration-isolation findings.

## Acceptance evidence

- Rust/MSRV/distribution/process boundary: issue design and master architecture §12.
- Debug/release builds pass with `--locked --workspace`.
- Release `rrx --help` passes; 3 unit + 5 executable integration tests pass.
- fmt and all-target clippy with `-D warnings` pass.
- CI runs all gates on macOS/Linux; exact revision outcomes are preserved in PR #28.
- README Development documents setup/build/test/install and startup measurement.
- Release help startup, 30 warm samples: median 4.195 ms, minimum 3.732 ms, maximum 6.130 ms. Benchmark JSON includes scope/binary/version/platform/warmup count. Live scheduler idle resource measurement belongs to #14/#27; none is claimed here.

## Independent review and finding verification

Claude Code reviewed supplied immutable factual diffs with operation tools disabled.
The executor's chat was not forwarded. Reviewed source revisions: `22c2344`,
`0813d26`, `64ffec7`. Final source delta has no blockers.

Verified and fixed: runtime-wide settings were project-overridable; workflow minimum
could decrease; loader/invalid-input tests were incomplete; missing config was
ignored without a subcommand; benchmark JSON lacked scope/binary; Project could
add runtime agent names. Typed project schemas, stricter minimum combination,
source diagnostics and executable tests now protect those paths. Master design
was updated to match implemented scope. The reviewer's claimed external rule
forbidding issue-doc references is not present in this repository and was not
adopted as a project rule.

## Mutation and regression

In detached temporary worktrees, protected tests failed when:
1. recursive config overlay was replaced (initial implementation);
2. workflow minimum was allowed to decrease;
3. project schema accepted runtime-wide session limits;
4. unknown runtime config keys were accepted.

Original bytes were restored and full tests passed with clean mutation worktrees.
A restored parallel test exposed timestamp-only fixture name collisions; an atomic
counter was added. No mutation was made to the independently reviewed worktree.

## Impact and limits

Configuration/workflow-class are downstream scheduler/adapter/context contracts.
First executable foundation: no earlier runtime features to regress. Config-check
and help do not start sessions or write local state. No browser/STG target applies.
No-subcommand help is temporary until #15 supplies the TUI. Task/Goal execution,
persistence and native adapters are not claimed implemented by this Issue.
