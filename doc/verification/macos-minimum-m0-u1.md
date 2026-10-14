# M0 first implementation unit: limited macOS evidence

Issue: <https://github.com/shuhei-suzuki/rururunx/issues/16>.
This record covers the initial Claude wire/metrics unit and its confirmed
repairs. It does not qualify whole M0, installed S/P, Task Waiting/Held,
live CLI, Goal Runtime, the final workflow, CI or merge.

The whole M0 design at `c6413082` remains owner-approved. The static catalog
is observation material, not runtime qualification. O03 was independently closed
at `2105e6b2`; it was not reopened without a new source defect.

## Fixed source and checks

The current code source is `a61d038d92e28363ca18b5d9133a255e0db43f37`.
The source stayed clean before and after each local command. Tests retained
default parallelism; no skip, ignore, timeout extension or warning suppression
was added as a repair.

| Check | Source | Result |
| --- | --- | --- |
| `cargo fmt --all -- --check` | a61d038d | PASS |
| `cargo clippy --locked --offline --workspace --all-targets -- -D warnings` | a61d038d | PASS |
| `cargo test --locked --offline -p rrx --lib execution::claude_wire -- --nocapture` | a61d038d | 24 PASS, 0 ignored |
| I04 Lines/queue actual consumer exact test | a61d038d | 1 PASS, 0 ignored |
| native / quota / installed M0 selected regression | 2752e1fc | 35 / 6 / 6 PASS, 0 ignored; retain this original SHA scope |

Earlier fixture-format compiler failures and the two Clippy findings remain
separate failed attempts. They were corrected in local commits and are not
rewritten as successes. The current result is not a full workspace test result.

## Source operators and restored controls

Seven distinct compiled source operators failed the intended actual assertions.
Every case was restored to the same source bytes and passed the same control.

| Operator | Actual consumer protected |
| --- | --- |
| I01 stale status snapshot | latest published native status without a frozen snapshot |
| I02 old bootstrap path | request/session/input-operation correlation |
| I03 raw API-status consumption | checked integer Presence and authentication/capacity classification |
| I04 Value decoder replacing original bytes | original integer precision through Lines and real result/metrics consumer |
| encoded queue guard omission | encoded boundary with short original exponent tokens |
| original queue guard omission | raw boundary and recovery of both counters on dequeue |
| bootstrap remember omission | modified-content reuse of an initialization UUID |

These are seven actual assertion kills and seven restored passes. A mutation
that failed to compile is retained separately and is not an eighth kill.
The initial exact-source import whitespace check is also retained separately;
new operator diffs passed their checks. Input negatives alone are not counted
as mutation evidence.

The I02 bootstrap-path kill stops at its first foreign-operation assertion;
its later UUID cases are not credited there. The separate remember operator
reaches UUID content refusal. The I04 kill stops at its first maximum-counter
assertion; only the restored control covers the subsequent queued scenarios.

Actual Native Child stop/wait outcomes supplied 49 receipts across mutation
and restored runs. The two queue cases supplied four real Child waits before
their budget assertions. Direct Cargo/helper children were waited/reaped.
These observations do not prove all groups or escaped descendants dead.

## Independent repair confirmation

Fresh Sol `gpt-6.1-sol/high`, official runner `rereview/round 3`, confirmed
I01–I04 and the shared queue consumers at fixed a61d038d:
**APPROVE, mandatory 0, new findings 0**. This is the first repair confirmation
for the I01–I04 lineage, after the initial implementation reviews and O03's
separate round-two confirmation. Served model/effort were not reported.
The reviewer did not execute tests or write code.

Body SHA256:
`9fa7de18dea1ec0b2e9ff9dc76d42b0fc7779a2402fc4cfce159eaa6cd328436`.
The packet contained source/diff/design excerpts and test summaries; raw logs,
previous review originals and real runtime resource IDs were excluded.

## Saved evidence and cleanup

Parent verification checked the seven compiler/test/log bindings and all actual
archive bytes: evidence 976 files, private source/history 737 files, and final
target payload 5020 files. Parent audit SHA256:
`60d6adad270fe949294f313a4b2bc176fd0832213d2dd5ec9d72e3b4f6a402cf`.
The final Mac test binary is Mach-O 64-bit. Historical binary hashes were
recorded immediately before each execution; the target archive contains only
the final payload, not all historical binaries.

After terminal, clean/tree and live byte/mode checks, only the completed
mutation private-source and dedicated-target copies were removed and absence
confirmed. Original M0 worktree, failure records, raw evidence and archives
were retained. The completed review's nine original files were hash-verified
and saved before removing its dedicated temporary output directory.
No Docker resource or builder cache was created by this local unit.

## Remaining whole-M0 acceptance

All-alias S/fresh P, pre-registration gate, Codex finite decoder/handshake/rate
metadata, journal/settlement/Unknown, Task Waiting/Held, saved successful
original controls, legacy safety migration, installed/owner-run/live Mac
acceptance and final PR/CI/owner approval/merge remain incomplete. Existing
version gates are retained until S/P wiring prevents a fail-open intermediate
state. CLI version observations are not new permission pins.

The owner's macOS First order is macOS P4, macOS F3, then Goal Runtime
#22/#23/#24/#27. F3 product ownership stays with
[CI 品質の収束](https://claude.ai/code/session_018R7Q39hySMcnivGdtBcyJC).
This finite unit and its backup do not complete any parent Issue or Goal.
