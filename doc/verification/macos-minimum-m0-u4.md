# M0 U4 verification — Codex finite contract, Schema gate and Codex Handshake in S

Input HEAD: `85daecf0`, on top of `b0f50b0f` (host fix 1), `55142749` (U4 implementation), `430efc2e` (owner approval of the Codex contract revision C1–C4) and `fc331446` (U3 round 2 carry-over). Contract basis: `doc/verification/macos-minimum-m0-catalog-0162.json` (codex rust-v0.162.1, 12 sources, 303 shared definitions, subtree `bbdede32…`), generator `scripts/build-codex-catalog.py`. Local macOS; Cargo `/Users/shuheisuzuki/.cargo/bin/cargo`, locked/offline. Every suite and mutation ran outside the command sandbox under the standing local-verification approval, at a committed HEAD with a clean tree before and after. No network, CI, deployment or live Agent qualification. Independent review has not yet run on this HEAD.

## Result and scope

- **Codex finite wire contract** (`execution/codex_wire.rs`, `codex_wire/types.rs`): 8 sent methods + `initialized` (no params), 8 responses, 84 notifications by class (21 typed, 4 method + ownership, 59 ownership only), 11 server requests (new server requests refuse), ThreadItem identities for all 19 kinds with typed content only for agentMessage / commandExecution / fileChange, Thread core, Turn, CodexErrorInfo incl. `Unclassified` (failed turn → Unknown/Held, never quota/auth/capacity), MessagePhase `partial_answer` (never an answer), ConfigRead judgement paths, McpServerStatusList core. Strict validation precedes ownership filtering; collector and approval entries take validated types. `terminal_sha256` stays the whole-Turn integrity digest inside the four existing whole-receipt checks.
- **Schema gate** (`execution/native/schema.rs`): `generate-json-schema --experimental` inside private S custody; 12 fixed manifest files; bounded sinks (process 64 KiB, files 16 MiB, file 1 MiB, 1024 entries / depth 4, schema JSON depth 64 / nodes 262144); strict exact match incl. default/enum/const values; tolerated growth within 32 item kinds / 64 notifications; refusal only for new structured reaches absent from the 20 basis reach pairs; finite result, kind, one bounded position and counts in the journal, no raw schema.
- **Codex Handshake in S**: initialize → initialized → account/read (refreshToken=false) → environment/status → rate read, each correlated and typed; requires ChatGPT, ready, valid rate; never thread/start or turn/start.
- **All-alias S** can now publish Passed. **Admission stays 0**: `require_qualified` returns the typed `FreshPreparationUnavailable` until fresh P (U5); no Driver, Unit, Session, transport or NativeInput.
- **MP-2 successful-run cleanup** of the run's own alias/epoch directories (descriptor-relative, no-follow, identity/UID/device checks, 1024 entries / depth 4); failed runs keep their paths.
- Definition → rrx type → decoder → consumer table: `doc/verification/macos-minimum-m0-u4-contract.md`; reduced fixture `crates/rrx/tests/fixtures/codex-contract/{basis-pair,manifest}.json`.

No auth/CAS/owner/marker/fencing/#19/process-custody or quota check was relaxed; no Task.version authority, test authority constructor, SQL success seed, `allow(dead_code)`, suppression, ignore/skip/serialization or timeout-only change was introduced. Master design sections are recorded for the PR group B update (not edited here).

## Host-verification corrections

| ID | Defect | Fix |
|---|---|---|
| H1 | The bootstrap `environment` case became ProtocolFailure. | The fixture sent `unsupported`, outside the approved environment domain (ready / pending / disconnected / unknown); it now sends a legal non-ready kind (UnsupportedCapability, expectation unchanged). A separate malformed case stays ProtocolFailure (`b0f50b0f`). |
| H2 | The settlement headroom boundary test no longer held after the U4 observation grew. | The test derives its boundary from the production `Observation::reservation()`; the reservation now covers the longest Schema reason (`server_request_added`). Bounds stay 4096 in Rust and SQL (`b0f50b0f`). |
| H3 | The bootstrap `metadata` case (`{}` rate response) became ProtocolFailure. | The approved basis makes `rateLimits` required, so a missing snapshot is a malformed response: the case now expects ProtocolFailure, and the U3 F2 absent-snapshot branch in `codex_windows` is removed. This supersedes the U3 F2 expectation and narrows acceptance. `NativeFailure::MetadataUnavailable` is kept only so persisted records (present since `74f48484`) still decode; a test reads one back (`85daecf0`). A class check of 219 pre-existing enum/state definitions found no other removed or renamed persisted value. |

## Final verification

At `85daecf0` (`.rrx/u3/host/full4/`): fmt, clippy and the bin build pass; lib 788 / 117 / 27 (nested child 43 / 1 / 1); every integration target and both helper crates pass except `daemon_native` 1 / 2 / 0. All 120 failures were rerun alone: two pass alone (`adapter::git_owner` parallel-only kinds); the **118 that fail alone are exactly the U3 set** — 115 admission-0 lib controls, the two admission-0 `daemon_native` controls and the known bootstrap file-reader. All 19 new U4 tests pass, including the real-child S controls (all-alias pass keeps admission 0; Schema alone or a bad Handshake refuses). Superseded partial runs at `55142749` and `b0f50b0f` are kept under `.rrx/u3/host/full4-*-partial/`.

## Mutations

All 138 operators ran at `85daecf0` (`.rrx/u3/host/mut-u4/index.tsv`, logs under `.rrx/u3/host/mutations-u4/`, failing assertions in `mut-u4/kill-reasons.tsv`). Each run must execute exactly one test; originals are restored by hash with a clean-tree check.

| Result | Count | Notes |
|---|---:|---|
| Killed at the intended check, restored run passes | 135 | Operators 115 (`u4-schema-alone-as-handshake`) and 48 are detected by a deadline (`Elapsed`); 114 (`u4-s-pass-substitutes-for-p`) by a missing typed refusal (`Option::unwrap`). |
| Equivalent (layered bound) | 2 | 36 (Rust intent bound; SQL CHECK still refuses) with supplemental 36s killed earlier; 110 (schema file pre-check; `decode_schema`'s 1 MiB frame bound still refuses) with supplemental **110s** (both bounds widened) killed at `file-plus-one`. |
| Not selected by the harness, rerun by hand | 1 | 126 has an id without its integration target; rerun on `runtime_cli` and `goal_cli`, killed at "startup did not settle normally" on both, restored runs pass. |

## Limits

- Live confirmation on the Mac (owner-run) is still required: the installed CLI's `generate-json-schema --experimental` output against the 0.162.1 basis, the ThreadStart sandbox shape, config/read features and mcp_servers shapes, and `thread/settings/updated`.
- Fresh P (U5), the fixed-sandbox Executor flow (U6), pin removal (U7) and the owner procedure (U8) are not done. SC10 stays BLOCKED pending issue #100.
- No task-owned worktree, Docker resource or build cache was created; Docker resources deleted: none, reclaimed: 0. Public tag checkouts used for the catalog stay under `.rrx/u4/` (Git-ignored). Test daemons were stopped through their endpoints.
