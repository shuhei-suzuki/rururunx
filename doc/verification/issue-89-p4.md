# P4 local implementation verification

Issue: <https://github.com/shuhei-suzuki/rururunx/issues/89>. Baseline:
`b8344ab951fa24f545174917eee7be067284c625`. Original checks below preceded
`c3fb1bea`; the per-entry-point repair checks use fixed `149cf859`.
This is partial issue work. M06's initial campaign is complete; V-F, review repair
approval, owner approval and merge are not completed here. F1/F3/F4 remain under
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
| Per-entry-point repair at 149cf859 | 18/18 crate tests PASS; default parallelism |
| Repair fmt / workspace all-target Clippy -D warnings | PASS / PASS |
| Six individual public prepare-bypass mutants | 6 detected with complete custody; 6 byte-identical source restoration pairs PASS |
| Report unknown/error guard mutant | pure NoResult assertion detects; restored source/control PASS |
| M06 initial macOS 50 x 2 at c3fb1bea | 100/100 PASS and observable; target failures/B_END/other failures/unobservable all 0 |

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

## Per-entry-point review repair verification

Six new outer controls call spawn/output/status_std/tokio separately. Safe OwnedFd
allocations occupy actual slots 3..9 before Tokio runtime construction. Four writer
witnesses (4/6/8/9) have observed CLOEXEC=0 and writer-end device/inode identities;
the three pipes are distinct and the safe duplicate matches its source writer.
The child uses `/usr/bin/python3 -I -S` and observes all witnesses before opening
its regular report. Bounded typed report errors are SETUP/NO_RESULT, never RESULT
fail. Only a valid identical inherited writer report fails the control.

The first compile at `fa291dc7` failed because Darwin's st_dev is signed i32.
`149cf859` introduces a checked portable conversion, with negative identities
classified SETUP and negative/zero/u64-max pure counterparts. The initial compile
failure is retained. The fixed source is committed/clean before all tests.

All 18 crate tests, fmt and workspace all-target Clippy pass on macOS. Six separate
source copies each bypass prepare in just one public entry, preserving the original
method delegation. Each exact control records a timely RESULT fail for inherited
writer, an actual left-Fail/right-Pass assertion, and complete S1/S2/S4/S6/S7 custody.
Each restored normal pair passes, with byte-identical source and complete recovery.
No SETUP, NO_RESULT, unregistered TIMEOUT or incomplete recovery is counted killed.
The seventh copy changes the common unknown/error report-token guard to Inherited;
the pure parser's expected-NoResult assertion kills it without creating a process,
and its restored normal pair passes. Existing inheritance/InnerOwner::close code is
unchanged; the separate EOF-reader classification review finding remains unresolved.

Every copy has its own target, compiled-source archives, source/actual binary SHA256,
explicit cached scope, local commits and normal/mutant/restored raw logs. Normal and
restored source inventories match exactly; their relinked binary SHA256 values differ
and binary reproducibility is not claimed. The three compile/test phases and their
actual observed behavior are retained. M0 verification was reported running in a
different worktree/target; that concurrent load and nonsecret build settings are
recorded. No test-threads override, watchdog or changed deadline is used.

## M06 initial campaign

Two independent git archive copies of fixed `c3fb1bea` (origin tree
`4d385dbfd92464d24a6ff9c40fe3f9411881ae07`) receive the validated private overlay:
143 project_api additions, no original deletions. Each overlay is committed clean
before compiling in a fresh separate target. Both jobs run all project_api tests
50 times with default parallelism under normal host permissions.

All initial 100 attempts are retained; there is no rerun or supplemental attempt.
All are PASS and observable, with exactly two trace records each (200 total).
c_s3d4/m2_a failure attempts, observable B_END, unobservable attempts, other product
failures and panic classifications are all zero. The initial inventory and source
are consistent and M06's numerical conditions are met. Each job's 51 direct cargo
children (compile plus 50 attempts) has matching retained-owner and wait/reap events.
This proves direct cargo collection, not descendant process-group death. lsof's extra
observer children, scheduling and the safe baseline output capture's FD effects
remain measurement limitations.

The M06 source stays c3fb throughout both jobs. Later commits change only cfg(test)
fixtures and documents; the measured production files and the original worktree's
existing binary inventory are unchanged. New tests build in a separate target, so
this does not claim identical binaries between the instrumented measurement copies
and the later repair build. The M06 evidence is bound to c3fb and its saved source/
compiled-binary hashes, not relabeled as a fresh campaign at a newer commit.

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

M06 evidence is saved under
`.rrx/handoff-goal-20261013/p4-m06-initial-20261013/` (summary, complete-proof,
required-artifacts and cleanup JSON; all-attempt archive; origin and transformed
source archives; all raw logs/attempts/owner events and binary inventories).
Both dedicated measurement copies/targets are removed after saved-source/hash/clean
verification. Review-repair evidence is saved separately under
`.rrx/handoff-goal-20261013/p4-api-verification-20261014/`, including the initial
Darwin compile failure and seven restored source/control pairs. All seven private
mutation copies and the dedicated normal verification target are removed after
source/evidence archive and clean checks. Artifact destinations are retained.

## EOF observation repair and completed GNU V-F

The EOF repair is test-harness-only. `lib.rs`, both platform implementations,
the consuming production APIs and project_api production/test files are unchanged
from the measured source. The initial local compile failure at `27ae6123`
(three rustix namespace errors and one unused import warning) is retained.
`5d2339b1` changes only the namespace/import; `e2886a1a` formats imports only.

| Local check | Fixed source | Result |
|---|---|---|
| macOS `cargo test --locked --offline -p rrx-spawn-hygiene --lib -- --nocapture` (dedicated target) | 5d2339b1 | 20 pass, 0 fail/ignore/filter; default parallelism |
| macOS `cargo clippy --locked --offline --workspace --all-targets -- -D warnings` (same dedicated target) | 5d2339b1 | PASS, 40.79 s; direct child wait/reap |
| workspace `cargo fmt --all -- --check` | e2886a1a | PASS; initial pre-format check failure retained |
| GNU full crate, locked/offline, default parallelism, same source archive | e2886a1a | 20 pass, 0 fail/ignore/filter; 4.12 s tests; compiler-JSON/actual AArch64 ELF bound |
| seven EOF classification/emission source mutants | e2886a1a archive | Seven actual equality-assertion failures with complete owner traces; each same-byte restoration and same exact control PASS |
| std callback-install bypass source mutant | e2886a1a archive | C1 and high-FD C2f actual delayed-EOF assertion failures with complete custody; both restored controls PASS |

All new local checks use task-only source/target resources. The GNU run uses the
existing immutable GNU image with `--init`, network-none, cap-drop ALL,
no-new-privileges/default seccomp, source/public registry read-only and only its
dedicated target writable. Its Docker run and immutable-CID absence observer
are directly waited/reaped. Source bytes remain identical. Mac source mutants
compile an exact compiler-JSON artifact and invoke that actual libtest binary
with the same exact consumer and default concurrency; compile/setup/observation
failures are not kills. All original logs/failed attempts and restoration records
are retained. These new mutant executions are macOS evidence, not GNU mutants.
Linux-specific fmt/clippy are not proved by the macOS checks.

The planned GNU V-F two jobs x eight repeats for each of main and P4 is complete
at frozen main `b8344ab9` and P4 `86154999`. All 32 original raw/row/receipt pairs,
four source copies and eight actual ELF artifacts are bound; all 32 Docker run
clients and 32 absence observers were waited/reaped. All immutable task-container
IDs were observed absent. Initial original NO_RESULT annotations remain intact.
The separate final GNU join has no guard errors and returns
STOP_OBSERVED_F1_INCREASE: main failed attempts 0/16, P4 5/16, with its unchanged
finite predicate classifying endpoint F1 one and UNKNOWN four. A separate
source-bound supplement ties the other four exact same-name panic/FAILED pairs
to actual Runtime owner reopen nonblocking flock EWOULDBLOCK, before epoch
creation. The supplemental observed F1 attempt rates are therefore 0/16 and
5/16 (31.25%). Original UNKNOWNs are not erased. This is a finite sample, not a
population rate; unequal/uncontrolled host load and unidentified original FD
holders prevent causal attribution. The completed macOS V-F remains inconclusive
for unknown attribution. No retry, production lock/unlock/fencing change or
P4->F3 ordering change has been applied. Dedicated fork/exec causal controls and
production-contract impact assessment remain pending.

Saved evidence: `.rrx/p4-eof-normal-20261014/`,
`.rrx/p4-eof-normal-fix1-20261014/`, `.rrx/p4-eof-clippy-20261014/`,
`.rrx/p4-eof-linux-20261014/`, `.rrx/p4-eof-mutant-preparation-20261014/`,
`.rrx/p4-linux-vf-20261014/`, and the completed secondary GNU report under
`.rrx/p4-linux-full32-offline-preparation-20261014/results/full32-initial/`.
Source supplement and independent parent input proofs are retained separately.
No group/fixture-wide death is inferred from Docker client exit or absence.

## Remaining gates

The GNU F1 increase STOP, dedicated causal controls/production lock-fencing
impact assessment, unresolved macOS V-F attribution, independent Sol/Opus
implementation/security repair confirmation, applicable approval conditions,
final mandatory CI, PR publication/merge and worktree cleanup remain with the
parent. EOF local controls/mutants do not close the finding without independent
confirmation. Existing Linux C2d/V3 acceptance is recorded separately. Known Tokio post-spawn Err custody limitations
remain unresolved by this fd boundary. Do not close issue 89 from this partial PR.
