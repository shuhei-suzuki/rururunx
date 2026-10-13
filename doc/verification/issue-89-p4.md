# P4 local implementation verification

Issue: <https://github.com/shuhei-suzuki/rururunx/issues/89>. Baseline:
`b8344ab951fa24f545174917eee7be067284c625`. Final source checks use `9db35e5d`.
This is partial issue work. Independent reviews, M06/V-F, owner approval and merge
are not completed here. F1/F3/F4 remain under
[CI 品質の収束](https://claude.ai/code/session_018R7Q39hySMcnivGdtBcyJC).

## Results

| Check | Result |
|---|---|
| cargo check --offline --workspace --all-targets | PASS |
| cargo fmt --all --check | PASS |
| cargo clippy --offline --workspace --all-targets -- -D warnings | PASS |
| rrx-spawn-hygiene lib | 11 PASS; C1/C2a/b/c/e/f/g, listfd classifier, OW1, OW2 and worker |
| rrx integration adapter/cli/context/git/project/project_api | 91 PASS in normal host permissions |
| rrx-process-tracker | 4 unit + 1 integration PASS; 1 existing integration ignore |
| rrx lib execution:: filter | 186 PASS / 2 Docker FAIL / 1 existing ignore; all attempts retained |
| focused Docker default-parallel replay | P4 11 PASS; fixed main archive 11 PASS |
| Linux C2d/V3 | NOT RUN on this macOS host |
| Browser verification | NOT RUN; no UI/API response change |

The full execution failures are `docker_filter_and_operation_identity_are_independently_checked`
(missing unregistered-own-volume) and `registered_docker_cleanup_preserves_three_sibling_units_and_known_work`
(docker_reconciliation_incomplete). They occurred while other private mutant
compilations were active. Load/budget interaction is a hypothesis, not an established
cause. The focused replays do not certify frequency equivalence; V-F must do that.
No test assertions, deadlines, ownership, concurrency or ignore lists are weakened.

The first integration attempt failed 19 adapter SETUP checks. A standalone Unix
socket probe demonstrated sandbox bind EPERM. The formal require_escalated retry
passed all 91 integration cases; both attempts and the probe are retained.

The first C2f fixture failed SETUP: soft NOFILE 1048575, target fd 1048574, dup EINVAL
(22), recovered through S7 ESRCH. A source-copy replay records the same before/after
limits and errno. The dedicated worker now sets its own soft limit to 1024 or less,
duplicates to exactly soft−1 and records fd 1023. Other tests/harness retain their
limits. C2f and V1-high prove the high descriptor causal path.

Initial compilation found rustix type/API mismatches, Tokio namespace inference,
borrowed OwnedProcess consumers and one temporary variable shadowing. These were
fixed and locally committed before final verification. Initial interactive
diagnostics remain in the task session; persistent later attempts are listed below.

## Source mutant evidence

Each private copy has a recorded original/mutated/restored source hash set and
actual binary SHA256, a dedicated target, exact test command, actual assertion,
classification/custody trace and restored normal pair. Worker selection is exactly
one test without test-threads override. Production original sources stay committed
and clean while tests run.

| Mutant | Actual detection | Restored pair |
|---|---|---|
| V1 callback installation removed | C1 RESULT fail: writer delays EOF | PASS |
| V1-high same removal | C2f RESULT fail: high writer delays EOF | PASS |
| V2 full-return rejection removed | C2e RESULT fail: unexpected Child, killed/reaped | PASS |
| V3 Linux ENOSYS ignored | NOT RUN; CI gate remains | NOT RUN |
| V4 mark stdio CLOEXEC | C2g RESULT fail: Broken pipe | PASS |
| V5 close rather than retain descriptors | C2b RESULT fail: unexpected Child, killed/reaped | PASS |
| V6 production fcntl non-EBADF failure ignored | C2c RESULT fail: unexpected Child, killed/reaped | PASS |
| V7 raw production git_owner spawn | Clippy disallowed-methods rejects actual call | PASS |
| V8 zero listfd return treated as valid | LISTFDS RESULT fail: expected EIO, got Ok(0) | PASS |
| V9 macOS pre_exec pause | TIMEOUT + SIGKILL + complete recovery | PASS |
| OW1 gate bypass | custody table fails on remaining-group Finish | PASS |
| OW1 lost authority signal | custody table rejects StopGroup after loss | PASS |
| OW1 contradictory result accepted | classification table expects NO_RESULT | PASS |

V1/V2/V4/V5/V6/V8 recovery records S1 group, S2 WNOWAIT exit, accepted S4 before
single S6 reap, then S7 ESRCH, complete=true. V9 records S1 group, S3 ok with fresh
WNOWAIT confirmation, S2 exit, accepted S4, single S6 reap with signal=9 and S7
ESRCH, complete=true. V9 alone is registered as an intentional stop mutant;
unregistered TIMEOUT, SETUP/NO_RESULT or unproved collection never count killed.
OW1 pure mutants create no process; their failed assertions do not orphan workers.

The initial V8 direct pure table is supplemented by the LISTFDS group harness.
The initial V7 copy omitted README (used by include_str), so its restored all-target
Clippy failed compilation. It is not used as a completed causal pair. V7-complete
includes README and all source inputs: only disallowed raw spawn fails, restoration
passes. The original incomplete attempt remains recorded.

## Evidence and cleanup

Private evidence is retained under `.rrx/p4-implementation-20261013/` in the main
repository. This is a saved artifact destination, not a worktree. Logs/JSON include:

- controls-final, controls-harness-final, controls-638-final, controls-causal-final;
- clippy-final, clippy-harness-final, clippy-638-final, clippy-causal-final;
- compile-638-final, fmt-final, fmt-638-final, fmt-causal-final;
- regression-integration, socket-probe, regression-integration-host;
- regression-tracker, regression-execution, regression-docker-p4/main;
- V1/V2/V4/V5/V6/V7/V8/V9, SETUP-C2f and all restored controls;
- V1-high, V7-complete, V8-harness, OW1-gate/loss/classification and restored controls;
- each *-source.json (source/binary hashes), main-docker-source.json, cleanup-before/after.json.

Sixteen task-only source/target pairs are restored/hash-checked and removed.
Logs, hashes and audit records are preserved. The actual P4 worktree and branch
remain for parent-owned independent review, CI/measurement, publication and merge;
their cleanup occurs after merge. No root checkout or other agent worktree is removed.

## Remaining gates

Linux C2d/V3, M06 macOS 50×2 with 100+ observable trials, V-F main/P4 Linux/macOS
two jobs × eight repeats (including failure frequency), independent Sol/Opus
implementation/security review, owner approval, PR publication/merge and worktree
cleanup remain with the parent. Known Tokio post-spawn Err custody limitations
remain unresolved by this fd boundary. Do not close issue 89 from this partial PR.
