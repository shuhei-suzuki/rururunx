# Issue #2 verification

Source implementation: `1dbf1a0` (runtime code at `f3506e2`).
PR: https://github.com/shuhei-suzuki/rururunx/pull/29.
Host: macOS arm64, Rust 1.91.1. Workflow: STRICT.

## Acceptance evidence

- Typed Project/Goal/Task UUID snapshots support create/read/update via Store.
- Real SQLite reopen retains Goal objective/criteria/DAG, Task phase/worktree/branch, native session references/PID hints, pending review/approval, context versions and usage.
- Two Projects can each own Issue #42; foreign Goal/Task/session/usage/context/DAG references fail.
- Snapshot optimistic versions reject stale writers without changing state/history.
- Snapshot and audit are atomic: injected journal failure rolls state back.
- Review/approval payload evidence is preserved across snapshot updates.
- Context versions are consecutive and immutable under Store and ordinary SQL.
- Missing telemetry remains null; unavailable all-missing usage requires a reason.
- App identity and schema guards reject foreign/future DBs without changing bytes; unknown typed snapshot fields fail instead of silently disappearing.
- 20 tests pass (8 config/CLI regression + 12 real SQLite integration); fmt, clippy `-D warnings`, debug/release builds pass.
- macOS/Linux CI runs the same gates; exact final outcomes are retained by PR #29.
- Migration/version and current implementation match the issue/master design.

## Independent findings: verify → fix → re-review

Claude Code receives immutable source diffs/requirements/design and test summaries,
with operation tools disabled. No executor transcript is sent.

At `2febe34`, reviewer H1 identified REPLACE overwriting audit events and M1
identified foreign DAG references. New Rust tests were committed (`4368248`) and
both failed against the actual implementation. `b615ba2` adds duplicate-insert,
monotonic audit, context SQL guards, owned DAG reference validation, reserved event
kinds, database identity and strict snapshot parsing. Re-review confirmed all
initial findings fixed with no code blockers. Its literal SQLite prefix finding
was corrected using `substr`, with a real `sqlite3_data` fixture and mutation proof.
Final delta review and CI gate merge.

## Isolated mutation evidence

Each mutation caused the protected test to fail; original bytes were restored,
full tests passed, and the detached worktree was clean:

| Removed/broken guarantee | Tested revision |
| --- | --- |
| Foreign key ownership enforcement | `2febe34` |
| Audit UPDATE protection | `2febe34` |
| Task journal insertion/atomic rollback | `2febe34` |
| Prior decision evidence | `2febe34` |
| Audit REPLACE protection | `b615ba2` |
| Goal Task reference ownership | `b615ba2` |
| Audit monotonic sequence | `9f515fe` |
| Declared DAG edge endpoints | `9f515fe` |
| Literal SQLite internal-name prefix check | `9f515fe` |
| Future schema rejection | `1dbf1a0` |
| Database application identity | `1dbf1a0` |
| Unknown snapshot-field rejection | `1dbf1a0` |
| Context SQL UPDATE protection | `1dbf1a0` |

## Scope / limits

Persistence does not establish native process liveness. Project Git registration,
DAG cycle/readiness/completion evaluation, workflows and runtime recovery remain
in their dependent Issues. Audit triggers enforce append-only ordinary SQL, not
protection against a database owner who drops triggers/changes the file. Usage is
append-only through Store API. No external staging/browser target applies to this
local persistence component. Unreleased intermediate v1 DBs without the RRX1 app
marker are not adopted; isolated fixtures are recreated.
