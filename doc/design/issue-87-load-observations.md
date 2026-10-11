# Issue 87: CA4i, SC10 and SC7 under parallel load — design delta (for review before code)

Paths are relative to `crates/rrx/src`. Base: `main` `c75b3f0`. Measurements ran on draft PR #90 (trace and workflow only, not for merge) and on the owner's Mac. This delta changes no production code. It changes only what three tests observe, when they start observing, and (SC10) when the test releases its fixture peers.

## 0. Measurement summary

Command: `cargo test --locked -p rrx --lib -- [--test-threads=N]` after `cargo build --locked --workspace`, on GitHub-hosted ubuntu-latest (4 vCPU, non-root) and macos-latest (arm64, 3 vCPU). `--test-threads=12` is the load amplifier: on a 3–4 vCPU runner it is close to the thread-per-core ratio of a 10–12 core Mac running the default.

| Run | Ref | OS | Mode | CA4i | SC7 codex | SC10 |
| --- | --- | --- | --- | --- | --- | --- |
| 38098194595 | `1c08445` | Linux | default ×3 / t12 ×3 / serial ×3 | pass / **fail 3** / pass | pass / **fail 3** / pass | pass |
| 38098194595 | head | Linux | same | pass / **fail 3** / pass | pass / **fail 3** / pass | pass |
| 38098194595 | `1c08445` | macOS | same | pass / **fail 3** / pass | pass | pass |
| 38098194595 | head | macOS | same | pass / **fail 3** / pass | pass | pass |
| 38102140447 | head | Linux | t12 ×2 | **fail 2** | pass | pass |
| 38103458272 | head | Linux | t12 ×4 | **fail 4** | **fail 4** | pass |
| 38105816181 | head | Linux | t24 ×2 | **fail 2** | **fail 2** (and SC7 claude) | **fail 2** (setup, §3) |

Baseline and head behave the same, so none of the three is an S4 regression.

**Owner's Mac** (MacBook Pro M5 Max, 18 CPUs, so 18 test threads by default), measurement head `aa4034a`, default `cargo test --locked -p rrx --lib`:

| Runs | Open-file limit | Result |
| --- | --- | --- |
| 5 | `ulimit -n` 256 (macOS default) | every run: 530 of 834 tests fail with `Too many open files` (EMFILE), SC10 among them at Runtime build. Not the #87 failure; see §8 |
| 5 | `ulimit -n 10240` | 11, 7, 9, 5 and 6 failures (598–841 s). **SC10 fails in 4 of 5** with the Issue's message. Run 1 also fails CA4i (`S1=35`) and SC7 claude and codex (`0 settlement polls, bound 3`) |
| 3 | 10240, SC10 alone, `--test-threads=1` | 3 of 3 pass |

## 1. CA4i — cause

**What it waits on.** `activation/pages.rs` claims 128 Tasks and waits for 128 S1 activation-hook observations within a fixed 120 s from `start()`.

**Where the time goes** (trace inside the CA4i section, Linux t12, 38102140447). Every claim is made within 4 service sweeps. Each preparation then takes the owner-wide Git gate (`RuntimeOwner::git_gate`, `execution/owner.rs`, taken by `git_lease` in `AttemptManager::prepare_inner`). The 67 and 65 gate holds never overlap. They follow each other 0.1–0.3 s apart, with a median hold of 1.55 s (0.53–2.44 s), and the wait grows linearly in FIFO order to 68–69 s. `driver … prepared` events arrive one at a time with a median gap of 1.8 s (1.7 s on macOS). Under that load, 128 preparations need ≈ 230 s, and S1 stops at 49–65 when the 120 s deadline expires. Every preparation that reached the gate completed. No context-slot (`bounded_fs`) timeout was seen, and the service loop never exited.

**Causal control (measured).** CA4i was run alone in serial (`--test-threads=1`) with a test-only extra hold inside the gate (38103458272):

| Extra hold per gate lease | Result |
| --- | --- |
| 0 ms | pass, 90.8 s |
| 500 ms | pass, 205.9 s |
| 1000 ms | **fail**: `claims=128 S1=96 unfinished=128`, preparations 1.19 s apart |

So the failure follows exactly from serialized gate throughput × 128 against a fixed total. Lock contention and a stalled queue are ruled out by the non-overlapping, steadily advancing holds.

**Production semantics.** The single Git gate is a deliberate owner-wide serialization. It is FIFO, has no deadline, and every waiter eventually runs. This delta does not change it. A Runtime with many simultaneously admitted Tasks prepares them one at a time; that is a throughput property, not a liveness defect.

**Also observed, not changed.** A service sweep that claims 32 Tasks holds `control_admission` for 5–11 s (32 evaluations of 0.11–0.50 s each). Newly spawned Drivers wait for it at startup for 2–10 s. This delays starts but loses nothing. It is outside #87 and noted here for the owner.

## 2. SC7 — cause

**What it waits on.** `success.rs` `sc7_ordinary_failure` releases the completion. It then waits for `observed_jobs().any(|j| j.finished)` and from that moment observes 8 s for 1..=bound settlement polls.

**The anchor is not the terminal.** `ObservedJob::finished` is `JoinHandle::is_finished` of the phase job's start task (`runtime/phase_jobs.rs`, spawned in the launch path). That task ends after launch and the normal bind, before the completion is even released. In every failure, `job finished` and `completion released` carry the same timestamp. The non-success terminal comes later: the fixture answers, `persist_saved_terminal` (`execution/native.rs`) commits it and revokes the owner. Until then the owner is live, and `settlement_turn`/`binding_turn` return `Skipped` by design ("a live owner costs no turn").

| Run | release → owner revoked | window length | revoked after window end |
| --- | --- | --- | --- |
| t12-1 | 8.80 s | 8.03 s | 0.76 s |
| t12-2 | 12.08 s | 8.01 s | 4.07 s |
| t12-3 | 9.35 s | 9.36 s | ≈0 |
| t12-4 | 9.96 s | 8.03 s | 1.97 s |
| t24 codex | 38.2 s, 29.2 s | 8 s | 30.2 s, 21 s |
| t24 claude | 13.2 s, 12.3 s | 8 s | ≈5 s |

Every failure has `turns=[]`, `owner_live: Some(true)` and `settled: Some(false)` at window end, and the owner clears just after. In serial the terminal follows the release within milliseconds.

**Causal control (measured, 38105816181).** SC7 codex was run alone in serial with a test-only delay before the terminal commit. With 0 ms it passes (10.1 s). With 10 000 ms it **fails**: `SC5(c) codex: 0 settlement polls, bound 3`, the owner is revoked 10.7 s after release, and it clears 12.7 s after the window end.

**Production semantics.** The production Root behaves as designed: no conclusion while the owner is live, then a settlement poll at most every 5 s. The test's window simply starts before the event it is about.

## 3. SC10 — cause

**What it waits on.** `success.rs` `sc10_four_projects_close_independently` starts four Tasks (two Claude, two Codex) in four Projects. It waits for each Task's normal Bound **in turn**, and only then writes `fixture-bootstrap-release` and `fixture-release` for all four (`release_completion`). It then gives all four Tasks 120 s to close.

**The bootstrap hold is shared across Tasks.** In commit mode the fixture peer holds its bootstrap reply (Claude `control_response` to `initialize`, Codex `initialize` result) until `fixture-bootstrap-release` exists (`runtime/installation/tests.rs` `fixture_mode`). The Runtime waits at most 30 s for that reply (`execution/native.rs`: the Claude initialize handshake and `Lines::response`). Normal Bound happens before the reply, so a Task can be Bound while its bootstrap is still held. Because the test releases every bootstrap only after the **last** Task is Bound, each Task's bootstrap wait is as long as the spread between its own `initialize` and the last Task's Bound.

**What happens under load** (owner's Mac, run 1, trace inside the SC10 section). The four Tasks reach Bound at t=361.6, 362.1, 386.7 and 387.1 s, so the spread is 25.5 s. One Task's peer had sent `initialize` earlier. At t=384.4, 2.7 s before the shared release at t=387.1, its unit records `native terminal committed and owner revoked (no settlement)`. From then on every success turn for it is a `settlement poll` (25 of them, every 5 s), and the Task never closes. The other three close normally. The panic shows `links ["session_bound"]`, `owner_live: Some(false)`, `settled: Some(false)`, `success_attention: "owned success settlement not observed"`. In serial, all four Bound within ≈4 s, far below 30 s.

**Causal control (measured, 38129592262, Linux and macOS identical).** SC10 alone, `--test-threads=1`, with a test-only delay between "all four Bound" and the completion release:

| Delay before release | Bootstrap release | Result |
| --- | --- | --- |
| 0 | shared (unchanged test) | pass (14 s Linux, 19 s macOS) |
| 25 s | shared | pass (39 s) |
| 35 s | shared | **fail**: all four Tasks log `claude bootstrap initialize timed out (30 s)` or `native rpc response 1 timed out (30 s)`, then `native terminal committed and owner revoked (no settlement)`, then only `settlement poll`; `SC10: Task … did not close` |
| 35 s | **per Task, right after its own Bound** | pass (49 s) |

So the failure follows exactly from holding a Task's bootstrap past the production 30 s bootstrap deadline. The shared release turns another Task's bind latency into this Task's bootstrap wait. Releasing each bootstrap at its own Bound removes the failure even with the same 35 s delay before the completion release.

**Production semantics.** The 30 s bootstrap deadline is a production startup bound: a native peer that does not answer `initialize` within 30 s is failed. That is correct behavior, and the Root correctly reaches no success for a non-success terminal. This delta does not change it.

## 4. Delta

| Item | Change |
| --- | --- |
| **D1 (CA4i)** | `activation/pages.rs` setup wait. Replace the fixed 120 s total (`setup_end`) with a progress bound. The wait fails when neither the claim count nor the S1 count has advanced for `SETUP_STALL` = 30 s. The snapshot, controlled shutdown and panic text are kept, and the panic names the stall. **Why this is the right bound:** the production gate has no total deadline by design. What the test can soundly assert is that each claimed preparation keeps reaching S1, which is a per-step liveness property. The longest gap measured between progress steps is ≈10.7 s (from `start()` to the first S1 under t12, Linux). Between S1 steps it is ≤3.4 s (t12) and 1.19 s (the +1000 ms control). 30 s is about 3× the longest gap and still catches a stalled queue, worker or gate. The later waits in CA4i are unchanged |
| **D2 (SC7)** | `success.rs` `sc7_ordinary_failure`: the window starts at the non-success terminal. Replace the wait for `j.finished` with a wait (same 60 s SETUP bound, label `SETUP: ordinary-failure terminal not observed`) for `owner_live == Some(false)` on the job's binding owner. This is the moment `persist_saved_terminal` has committed the terminal and revoked the owner. The 8 s window, the bound formula and every assertion (1..=bound polls, the attention text, no gate link, never success-closed, Task version unchanged) are unchanged. Both `sc7_claude` and `sc7_codex` use this function |
| **D3 (SC10)** | `success.rs` `sc10_four_projects_close_independently`: right after each Task's own `wait_normal_bound`, write that Task's `fixture-bootstrap-release` only. The shared completion release after all four are Bound (`release_completion`, which also writes `fixture-release`), the `marked` snapshot, the 120 s close bound and every assertion are unchanged. **Why this is sound:** the bootstrap hold exists only to keep a peer from progressing before its own normal Bound (`fixture_mode`: "Hold only the external protocol peer's bootstrap, after a real spawn"). Once that Task is Bound, the hold has done its job. Holding it longer only ties this Task to the 30 s bootstrap deadline through other Tasks' bind latency. The peer still waits for `fixture-release` before it commits or completes, so the four completions are still released together. The control's per-Task run passed with the same `marked`-version assertion, so bootstrap progress does not move the Task version. `sc10_cross_project_quota_drift_replans_from_fresh_images` already releases each bootstrap at its own Bound with the existing `release_bootstrap` helper; D3 reuses it |

No auth/CAS/owner/marker/#19/process-custody check changes. No test is skipped, ignored or deleted. No production timeout or ownership behavior changes.

## 5. Controls (to run after approval)

| Control | Expectation |
| --- | --- |
| C87-1 | CA4i alone in serial with +1000 ms per gate lease: unchanged test FAIL (measured); with D1, PASS |
| C87-2 | CA4i with one gate lease that never ends (the 10th): with D1 it FAILs with the stall message within ≈30 s of the stall. It neither hangs nor passes |
| C87-3 | SC7 codex alone in serial with the terminal commit delayed 10 s: unchanged FAIL (measured); with D1/D2, PASS |
| C87-4 | Same for SC7 claude |
| C87-5 | Linux t12 ×4 and macOS t12 ×3 at the fixed head: CA4i and SC7 0 failures |
| C87-6 | SC10 alone in serial with the completion release delayed 35 s: unchanged FAIL (measured, both OSes); with D3, PASS |

The three knobs (an extra gate hold, a terminal delay, a completion-release delay) are test-only. **Q1 for the owner:** (a) keep them on the measurement branch only, with the controls recorded here as runs (the default proposed), or (b) add them in-tree as `cfg(test)` seams with the controls as ignored-by-default tests.

## 6. Mutants (each restored)

| Mutant | Expected detection |
| --- | --- |
| m1 D1 reverted (fixed 120 s total) | C87-1 FAIL |
| m2 D1 without the stall bound (wait unbounded) | C87-2 does not finish (job timeout) |
| m3 D2 reverted (window from `finished`) | C87-3 FAIL |
| m4 production: `settlement_turn` returns `Skipped` for a revoked owner without a settlement | D2 SC7 FAIL with 0 polls. This shows D2 still detects a real missing poll |
| m5 production: `binding_turn` skips the unbound settlement poll | D2 SC7 FAIL (whichever path the provider takes). Measured: survives, see results below |
| m6 D3 reverted (shared bootstrap release) | C87-6 FAIL |
| m7 production: `settlement_turn` never installs the continuation (always `settlement poll`) | D3 SC10 FAIL (`did not close`). This shows D3 still detects a Task that really cannot close |

### Results at the D1–D3 head (`3428dc9`, Issue 87 measure run 38134685896, plan `verify`, both OSes identical)

| Run | Linux | macOS |
| --- | --- | --- |
| CA4i, SC7 codex/claude, SC10 unchanged inputs, serial | pass (127 s) | pass |
| C87-1 gate +1000 ms | pass (327 s) | pass (364 s) |
| C87-2 10th lease never ends | FAIL in 46 s (stall bound) | FAIL in 48 s |
| C87-3 / C87-4 terminal +10 s | pass / pass | pass / pass |
| C87-6 release +35 s | pass (49 s) | pass (54 s) |
| m1 fixed 120 s, gate +1000 ms | FAIL (124 s) | FAIL (128 s) |
| m2 unbounded, 10th lease never ends | does not finish (`timeout 600`, exit 124) | not run (no `timeout`/`gtimeout` on the runner) |
| m3 window from `finished`, terminal +10 s | FAIL: 0 polls, `owner_live: Some(true)` at window end | same |
| m4 `settlement_turn` skips without a settlement | FAIL both providers: 0 polls, `owner_live: Some(false)` | same |
| m5 `binding_turn` skips the unbound poll | **survives** (both providers pass) | survives |
| m6 shared bootstrap release, +35 s | FAIL `did not close`, `settled: Some(false)` | same |
| m7 continuation never installed | FAIL `did not close`, `settled: Some(true)`, settlement polls only | same |

m5 survives because SC7's Task is Bound (`observation: Bound` in every snapshot), so its turns go through `settlement_turn`, never the unbound branch of `binding_turn`. SC7 therefore does not cover that branch; the expectation in §6 was wrong. m4 is the mutant that shows D2 still detects a missing poll. Rust CI (fmt, clippy `-D warnings`, test) on `3428dc9`: green on both OSes (run 38134685906).

### Parallel runs at the D1–D3 head (`--test-threads=12`)

| Job | CA4i | SC7 codex | SC10 | other failures |
| --- | --- | --- | --- | --- |
| Linux ×4 | 4/4 | 4/4 | 4/4 | run 3: `cli::endpoint` ×2 (EAGAIN, §8) |
| macOS runner ×3 | 3/3 | 3/3 | **2/3** | §8 |

The macOS SC10 failure (run 1) is a different mode from the one D3 fixes: `SETUP: normal Bound absent` after the 60 s `wait_normal_bound`. No production deadline fired in SC10. The four units kept taking the gate (holds of 1.4–9.9 s, waits up to 13 s), but no Task was Bound within 60 s. In the same run, single-Task tests that also hold the peer's bootstrap until Bound failed when the 30 s bootstrap deadline fired before Bound (`sc4_codex`, `sc4n_codex`, `scr_codex`; `sc4_claude` in run 3). And `ca6` (run 2) recorded one service sweep body blocked for 40.5 s inside `observe_task_drivers`. Since Bound is written by a sweep, a sweep blocked for that long explains both the missing Bound and the expired bootstrap deadline. This is a hypothesis under measurement: commit `2b47305` adds step timings inside `observe_task_drivers` and dumps the sweep's current stage when the Bound wait fails. Nothing is changed for it until the cause is shown.

## 7. Verification plan

`cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings` and `cargo test --locked --workspace` on Linux (non-root) and macOS through CI. In addition, the macOS parallel rrx lib at the default thread count 10 times and with `--test-threads=12` 3 times at the fixed head, recording CA4i/SC7/SC10 results and refs here. On the owner's Mac (18 threads) this needs `ulimit -n 10240` in that shell (§8).

## 8. Seen in the same runs, outside #87

Recorded for triage, not addressed here:
- `sc3p_*` (#89 item 1)
- Grok `watchdog_expired` ≈60 s (#89 item 3 family)
- `fm_d2_committed_source_631_codex_quick_front` (`EOF while parsing a value` reading `fixture-native-input.json`)
- `ca1`/`ca2` (`original marker and Bound absent`)
- `fm_rb … another Runtime owns this state root`
- `c_s1e2_bind_replaces_known_leftovers_and_refuses_foreign` (`control endpoint already published`, EAGAIN)
- Docker cleanup tests, `sc9d`/`sc4n` setup
- `retained_inspection_inflight_…` (`ParseLength { len: 0 }`)
- Owner's Mac, default `ulimit -n` 256 with 18 test threads: 530 tests fail with `Too many open files`. A developer-environment limit, not #87; with `ulimit -n 10240` there were no EMFILE errors in 5 runs
- Owner's Mac with 10240, run 1: `grok::environment_tests` ×5 (`watchdog_expired` ≈60 s; #89 item 3 family), `docker_filter_and_operation_identity_are_independently_checked`, `ca3b_original_composition_cross_pairs_refuse_before_store` (`original marker and Bound absent`)
