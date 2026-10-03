# Issue 18 verification

Date: 2026-10-03. Workflow: STRICT. PR: [#35](https://github.com/shuhei-suzuki/rururunx/pull/35).

Production-code revision: `2b3fa458a0d03975d075be1c69b82db2cd50a174`.
Final code/test revision: `5691dcd4e10fd1be0be8f9adfb28074648bb2d0a`.
Subsequent code/test commits refine native configuration diagnostics, same-length
content freshness, successful fresh-validation controls and documented limits.
They do not change production behavior. This report is a documentation-only delta.
No merge, Issue closure, native inference or workflow dogfood is claimed here.

## API and checks

`RepositoryContext::{index,validate,select,expand}` accepts an exact bound Task
scope. `Budget` bounds rendered bytes and estimated tokens. `SelectionOutcome`
returns `Ready(ContextSlice)` or `NeedsBudget` without launchable payload;
file/symbol/caller/callee expansions preserve all requested content or fail.
`ContextSlice::prepared_input` rechecks freshness for the existing adapter input.
Project/Goal/Task versions are transient freshness guards; source versions contain
content digests, not Task state versions changed by workflow transitions.
Durable ContextVersion publishing and conditional Context Pack assembly remain
Issues 8/19. `Store::audit_if_current` provides an additive atomic version/activity
check and scoped audit append without a schema migration.

Final code/test-head checks with pinned Rust 1.91.1:

- `cargo fmt --all -- --check`: pass.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: pass.
- `cargo test --locked --all-targets`: 97 pass (11 library, 18 adapter,
  5 CLI, 16 context, 18 Git, 16 registry, 13 state).
- Debug workspace/example and release workspace builds: pass.
- [Exact code/test-head Linux/macOS CI](https://github.com/shuhei-suzuki/rururunx/actions/runs/37103843498): success.

The native fixture suite covers two isolated Projects with the same Issue 42,
concurrent context observations, exact worktree/branch/scope, blocked Projects,
state changes and separate-connection audit CAS, dirty same-length changes,
HEAD/add/delete/staged rename changes, whitespace paths, admitted ignored evidence,
primary rules/config, moved-root file identity, symlinks/hardlinks/nested repos,
canonical Git metadata/namespace/special files, source and graph bounds,
mandatory preservation, byte accounting, escaped typed frames, selection reasons,
all four expansion modes and nullable measurements. Fixtures use real Git.
Production preserves native configuration and hooks; fixture isolation does not
change native runtime behavior.

The rebuilt `repository-context` inspection example exercised map, selection,
caller expansion and an insufficient budget against an isolated native repository
and existing durable bound Task. Results are in
`/private/tmp/rururunx-issue18-native-cli-smoke.json`, built at `da3afbf5b097550b36327c42c3992bd09a8d5e3b`
with the same final production code. It writes scoped observations but starts no
agent. These are local inspection measurements, not provider token/cost metrics.

## Mutation evidence

Clean detached temporary worktrees kept review inputs immutable. All 21 unique
mutations failed their intended runtime assertion, with no compile-error result.
Seventeen core mutants ran at `ad985cb`; three final Git guards ran at `da3afbf`;
the same-length content mutant ran at final code/test head. Changes after the core
round preserve those guard implementations.

| Mutation | Verified boundary |
| --- | --- |
| Ignore source digest comparison | ignored evidence and primary rule/config freshness |
| Omit mandatory rules | complete registered rule content |
| Omit mandatory evidence | required evidence remains complete |
| Bypass mandatory budget | insufficient budget cannot yield launchable input |
| Bypass optional budget | rendered optional content and wrapper obey limits |
| Remove no-follow open | symlink source isolation |
| Ignore normalized relative components | source path containment |
| Ignore bound Task branch | exact owned Git worktree |
| Raise file maximum | whole-index rejection of excessive sources |
| Trim raw NUL inventory | leading/trailing whitespace path preservation |
| Ignore root device/inode | replaced source/worktree roots cannot reuse context |
| Permit hardlinks | foreign shared inode rejection |
| Ignore transactional version CAS | cross-connection scoped audit freshness |
| Render unescaped body text | mandatory frame provenance cannot be forged |
| Disable graph visit limit | dense graph global work bound |
| Restrict callers to definitions | reference caller expansion |
| Share live process flag as global latch | simultaneous two-Project observations |
| Omit no-renames inventory flag | staged rename preimage remains missing/changed |
| Use bare synchronous Git executable | worktree-local relative PATH cannot replace Git |
| Discard GIT_CONFIG_GLOBAL | configured malformed include error stays authoritative |
| Hash byte length instead of contents | same-length already-dirty edit invalidates map |

Results/logs: `/private/tmp/rururunx-issue18-final-mutations.json`,
`rururunx-issue18-delta-mutations.json`, `rururunx-issue18-config-mutation.json`,
`rururunx-issue18-hash-mutation.json`, and their per-mutant logs.
The configured-include mutation failed the actual native configuration assertion;
repeat registration is explicitly idempotent. The hash-by-length mutation failed
`unwrap_err` after the second same-length edit, after successful fresh validation.

## Independent native review

Configured native Claude (`claude-opus-5-5`, medium effort), tools and MCP disabled,
factual source/docs/diff only, default model/auth/hooks/settings retained. Reviews
used committed immutable clean public heads. Findings were verified against source
and targeted mutation/native fixtures before corrections. An obsolete review was
cancelled when a self-detected live-flag defect changed its code head; its empty
report is not counted as approval.

| Round/revision | Verified outcome | Native session / local result |
| --- | --- | --- |
| 1 / 2dc00007 | Fix raw NUL trimming, atomic audit CAS, body framing and graph work bounds | `1e1d2016-9bf0-4be1-b5f4-b3878098fbb7` / `rururunx-issue18-native-review.json` |
| 2 / ad985cb | Core fixes verified; fix staged rename preimage and bound resolver/canonicalization work | `b8d3cc07-96bf-4542-b658-567ca7e662fe` / `rururunx-issue18-native-final-review.json` |
| 3 / da3afbf | No Critical/High implementation defects; verify configuration test causality and document per-call deadlines/native PATH scope | `306c5080-5428-433f-9a04-73ba34b9a794` / `rururunx-issue18-native-final-delta-review.json` |
| 4 / f1abc09 | Configuration/idempotence closed; strengthen direct freshness controls and content-hash proof | `3dbcf254-b384-4b55-b257-ab63a1263e24` / `rururunx-issue18-native-test-review.json` |
| 5 / 5691dcd | No Critical/High/Medium; successful fresh control and hash-by-length mutant prove same-length content freshness | `489c1d2e-7b7e-4588-ad61-37d3ca5056dd` / `rururunx-issue18-native-hash-review.json` |

All native result paths above are under `/private/tmp/`. They preserve actual
review usage/cost fields. Development review measurements are not implementation
measurements of Context Efficiency or runtime two/triple-review workflows.

## Limits and integration

The map is a bounded lexical approximation, not a compiler call graph. Identifier
mentions can include comments/strings; per-file lexical clipping is observable in
map metadata. Unsafe, oversized, non-UTF8-path or unsupported directory/submodule
inputs fail closed rather than produce a falsely complete index. Binary sources
are hashed and skipped. Explicitly admitted ignored files are bounded and scoped.
Filesystem observations are non-atomic: consumers validate immediately before use.
Device/inode checks narrow replacement races but do not guarantee inode non-reuse.

`utf8_bytes_v1` is a deliberately conservative packing estimate, not a model
specific tokenizer or provider context-window guarantee. Measured tokens remain
`None`; no fabricated provider usage/cost is recorded. Scoped JSON-line frames
separate headers, rules, evidence, source and map sections. A downstream Context
Pack must keep that provenance and mandatory-content policy.

Git/filesystem work runs outside SharedStore. Git groups have time/output cleanup
bounds, filesystem slot waits/jobs are separately bounded, and graph/scanner work
has deadlines. This is not a five-second total indexing SLA. Two global worker
slots can be occupied by uncancellable stalled filesystem syscalls; other Projects
then time out. Uncertain native Git cleanup blocks all context launches in that
process; automatic recovery is not implemented. Ordinary live Git calls use
independent flags and permit concurrent Project observations.

Native environment/configuration/hooks remain authoritative. Only the top-level
Git executable excludes relative PATH candidates; child tools/hooks retain PATH.
An isolated macOS probe with `core.fsmonitor=true` verified that native inventory
queries can start a detached Git-managed fsmonitor daemon even with
`--no-optional-locks`; that infrastructure service outlives the observation group
and is not an AgentSession. The fixture-owned daemon was stopped after the probe.
Evidence: `/private/tmp/rururunx-issue18-fsmonitor-object-probe.json`. Runtime does
not disable native configuration or stop unrelated Git-managed services.

The library and inspection example are complete for this Issue. Main CLI context
commands, automatic agent expansion transport, progressive rule selection,
checkpoint condensation, provider accounting and final real-Goal benchmark remain
dependent work; README and master requirements describe that partial integration.
