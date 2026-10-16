# M0 U3 verification — startup probes, v12 journal and private S custody

Input HEAD: `535ce4745ba032cf1ad4a4a26ff65d08e15b248e`, on top of `407b94db` (host-verification fixes F1–F3) and `c87800eb` (U3 implementation), base `72673811` (U2 approved). Approved role revision: `2a9114bc`, B1–B4/B2a. Local macOS (Darwin 25.6); Cargo `/Users/shuheisuzuki/.cargo/bin/cargo`; all dependency operations locked/offline. Socket and process tests need host permissions, so every suite and mutation ran outside the command sandbox under the standing local-verification approval, at a committed HEAD with a clean working tree before and after. No network, CI, deployment, or live Agent qualification was performed. Independent review has not yet run on this HEAD.

## Result and scope

U3 registers retained startup custody at `Runtime::start` and probes every configured native alias. Claude aliases run Version and the Reviewer initialize with the real plan-mode argv (`--permission-mode plan` is required before spawn). Codex aliases run Version; Schema and Handshake are refused with distinct finite reasons (`codex_schema_unavailable`, `codex_handshake_unavailable`) until U4. **No overall S pass is published, so Executor admission stays 0** (`installed_driver_composition` refuses before any claim). Schema 12 adds the `runtime_startup_probes` journal: intent before spawn, exact CAS on state/version/body for settlement, per-epoch/alias/row/total/uncertain budgets, 4096-byte JSON bodies (Rust bound plus SQL `CHECK`), old-epoch pending intents fenced to unknown, and 64-epoch pruning of confirmed history. Startup directories are created descriptor-relative under the root, bound to the owner.lock identity, mode 0700, and the program identity (path, stat, SHA-256) is rechecked before each spawn. The control protocol is 5. Every new spawn goes through `rrx_spawn_hygiene`.

Recursive cleanup of successful startup directories is deferred to U4 together with its only consumer (the overall S success); U3 adds no unreachable producer. Failed runs retain their paths.

No auth/CAS/owner/marker/fencing/#19/process-custody or quota check was relaxed. No Task.version authority, test authority constructor, SQL success seed, `allow(dead_code)`, suppression, new ignore/skip/serialization, or timeout-only change was introduced.

## Host-verification corrections

The first host run of the U3 implementation found five defects that the implementer's sandbox could not observe. Each was fixed by the implementer and confirmed on the host.

| ID | Defect | Cause | Fix |
|---|---|---|---|
| F1 (High) | Every naturally exited startup child failed group hygiene on macOS, so S could never pass (all five host-only U3 tests failed alone). | `RetainedRawProcess::hygiene` accepted only `Ok`/`ESRCH` from the group signal. On Darwin, `killpg` on a group whose only member is the unreaped leader returns `EPERM` for signal 0 and SIGKILL (`ESRCH` after reaping); reproduced with a standalone child. | Reuse the existing macOS resolver (`adapter::resolve_macos_signal_result` + `macos_group_is_dead`): `EPERM` counts only after positive `/bin/ps` inspection, still before reap; inspection failure remains a refusal. Non-macOS unchanged. Class: 4 strict-result consumers checked, 1 fixed; `OwnedProcess::signal_group` is reachable outside U3 and unchanged (it already falls back to a direct stop and reap). |
| F2 (Medium) | An absent bootstrap rate snapshot (`{}`) became `ProtocolFailure`; the existing `bootstrap_failure_categories…` test failed. | The U3 typed rate decoder classified every decode error, including absence. | `quota::codex_windows` returns `MetadataUnavailable` when `rateLimits` is absent and `ProtocolFailure` for a present malformed value; bootstrap propagates it. Existing test unchanged; a `metadata-malformed` case and `missing_and_malformed_rate_snapshots_have_distinct_failures` were added. |
| F3 (Medium) | `runtime_cli` whole-state comparisons raced the startup journal (only `runtime_startup_probes` differed: pending v1 → confirmed v2). | The background S task settles after the baseline was read. | `tests/support/startup.rs` waits for public `RuntimeStatus` startup settlement before a baseline; the startup table stays in the comparison. Applied to `runtime_cli` and the `goal_cli` `stable()` helper (7 sites checked, 3 fixed). |
| G1 (Medium) | `tests/daemon.rs` C-S1e2 still expected protocol 4. | Protocol bump consumer missed. | The test replaces every older protocol `1..PROTOCOL_VERSION` and uses the public constant; unknown-protocol refusal uses current+1. Production already lists 1–5 as replaceable. 20 consumers checked, 1 fixed. |
| G2 (Medium) | `goal_cli` lifecycle test refused with `ClaudeExecutorUnavailable`. | The fixture used a Claude worker; Goal validation has required a Codex Executor since U2 (`77d3b016`). The intended B1 contract; the fixture was stale. | The fixture worker is Codex; `require_executor` unchanged. 9 Claude-Executor fixture sites checked, 1 fixed; the rest are asserted refusals or admission-0 blocked. |

### Verification scope of U2

The U2 report lists five integration targets as the integration suite. The crate has **sixteen** integration targets plus the two helper crates; `goal_cli` and `daemon` were not run at U2, which is why G2 (introduced at U2) surfaced only here. From U3 on every target is run (below).

## Final verification

Full suites ran at `407b94db` (`.rrx/u3/host/full2/`: input/output HEAD and status, per-command logs and exit codes, `full-results.json`, `failures.tsv`). `535ce474` changes only `tests/daemon.rs` and `tests/goal_cli.rs`; at `535ce474` fmt, clippy (`.rrx/u3/host/lint-535/`) and both changed targets (`.rrx/u3/host/fix2-targets/`) were rerun. The bin is rebuilt before the library suite because the daemon controls exec `target/debug/rrx`.

| Command (absolute Cargo path above) | Passed / failed / ignored | Result |
|---|---|---|
| `cargo fmt --all -- --check` (535ce474) | n/a | PASS |
| `cargo clippy --locked --offline --workspace --all-targets -- -D warnings` (535ce474) | n/a | PASS |
| `cargo build --locked --offline -p rrx --bins` | n/a | PASS |
| `cargo test --locked --offline -p rrx --lib` | 755 / 117 / 27 (nested child 44 / 0 / 1) | FAIL (classified below) |
| `--test adapter` / `cli` / `codex_unavailable` / `context` | 19/0/0, 6/0/0, 1/0/1 (child 1/0/0), 16/0/0 | PASS |
| `--test daemon` (535ce474) | 6 / 0 / 0 | PASS |
| `--test daemon_native` | 1 / 2 / 0 | FAIL (admission 0) |
| `--test git` / `goal_graph` / `grok` / `managed_docker` / `managed_tools` | 18/0/0, 4/0/0, 16/0/2, 1/0/1, 1/0/0 | PASS |
| `--test goal_cli` (535ce474) | 7 / 0 / 0 | PASS |
| `--test project` / `project_api` / `runtime_cli` / `state` | 18/0/0, 14/0/0, 12/0/0, 18/0/0 | PASS |
| `-p rrx-spawn-hygiene` / `-p rrx-process-tracker` | 20/0/0, 4/0/0 (+1/0/1) | PASS |

Every failure was rerun alone with `--exact --nocapture` (121 invocations, `.rrx/u3/host/full2/alone/`). One passed alone: `u3_aggregate_capture_boundary_is_shared_by_all_aliases` (its 15 s settle bound expired under the parallel load; bound unchanged). The `daemon` and `goal_cli` failures were G1/G2 and pass at `535ce474`.

| Remaining alone failures | Count | Attribution |
|---|---:|---|
| Expected admission 0 | 115 lib + 2 `daemon_native` | Executor/activation positive controls need an installed Driver claim, which U3 keeps closed (`SETUP: normal Bound absent`, `normal write not reached`, `SN absent`, `activation not reached`, `Task not in flight`, …). Includes `u2::native_handoff_refuses_claude_provider_before_ticket_and_asset_transfer`, whose refusal hook is reached only after a claim, and three SC10 controls. Restored by U4 when S can pass. |
| BLOCKED #100 | (within the above) | `sc10_four_projects_close_independently` is additionally BLOCKED pending issue #100 as recorded at U2. |
| Known environmental | 1 lib | `workflow_source_bootstrap_file_reader_enforces_type_literal_path_and_size`, as at U2. |

The 27 library ignores and the target-level ignores predate this work. No ignore was added. The frozen legacy ledger is unchanged (560 IDs, 187/371/2, pending 189); U3 changes no legacy disposition.

## Mutations

All 72 planned operators ran on the host at `535ce474` (`.rrx/u3/host/mut/index.tsv`, per-operator logs under `.rrx/u3/host/mutations/`). Each applies exact source bytes, runs one test with `--exact --nocapture` (a run that does not execute exactly one test is not counted), restores the original bytes with a hash check, requires a clean tree, and reruns the test. The failing assertion of every mutant is listed in `.rrx/u3/host/mut/kill-reasons.tsv`.

| Result | Count | Notes |
|---|---:|---|
| Killed at the intended assertion, restored run passes | 71 | Covers provider/config divergence, removed alias, the N-4 post-claim zero-effects boundary, selfcheck propagation, real Reviewer argv, initialize correlation, wire round trip, old-writer guards for schemas 4–11, CAS/body/alias/epoch/total/uncertain/prune budgets, path/lock inode and unsafe mode, byte/stderr/aggregate budgets, deadline renewal, panic/Drop/join/reap, forced-death fencing, F1–F3 and G1–G2. Operator 48 (`C1-old-hello-socket-accept`) is detected by the client's refusal deadline (`Elapsed`), not an equality assertion. |
| Equivalent | 1 | Operator 36 widens only the Rust intent bound; the SQL `CHECK(length(CAST(body AS BLOB)) <= 4096)` still refuses a 4097-byte body. A supplemental mutant widening both (`36s`) fails `MP-1: intent body bound widened`; its restored run passes. |

## Harness notes and limits

- A first full run hung for three hours: the library daemon controls executed a stale `target/debug/rrx` (protocol 4). The run script now builds the bin first. Separately, `cli::daemon::tests::c_s1b2_identity_match_refuses_a_replacement_epoch` hangs instead of failing when its setup panics while the paused start task is inside `block_in_place`; this predates U3 and is recorded for a follow-up.
- Real foreign-uid / euid-mismatch producer proofs need the actual privilege conditions and are not claimed; descriptor/value predicates are separate evidence.
- The master design (`doc/design/master/`) is not updated for U3: U3 is an intermediate state (admission 0) that U4 changes. The master sections for startup probes, the schema 12 journal and control protocol 5 are updated once with the U2–U8 group before it goes to main, in the same pull request.
- Linux/macOS CI, Codex finite Schema/Handshake (U4), P/Executor setup/turn, profile/pin removal and the real Mac owner-run are not performed or claimed. SC10 stays BLOCKED pending issue #100.
- No UI/browser work applies. No task-owned worktree, checkout, Docker resource or build cache was created; Docker resources deleted: none, reclaimed: 0. Test daemons started by the suites were stopped through their endpoints. Scratch, scripts and evidence are only under `.rrx/u3/` (Git-ignored).
