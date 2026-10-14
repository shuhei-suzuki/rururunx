# P4 local implementation verification

## Current macOS First implementation state

The platform amendment approved at `aed6f6b4` is implemented. Local checks below
cover `e8359628`; fresh Sol and Opus initial implementation reviews at
`cb2dcd66` both APPROVE with mandatory 0. macOS keeps
its existing backend; Linux every target_env uses an untouched Command/no-op
install with no additional hygiene callback, syscall or probe. Other targets
retain ENOSYS. Six consuming signatures and existing caller migrations remain.
Linux six-API controls compare fully validated original/wrapped descriptor
reports with all four known original writers inherited; Linux EOF pairs use a
test-owned CLOEXEC baseline. Mac-only callback/list/injection controls are cfg
scoped. No test serialization, timeout extension, skip/ignore or warning
suppression is introduced.

| Local check | Fixed source | Result |
| --- | --- | --- |
| `cargo fmt --all -- --check` | `e8359628b057a23162063bec75497b53ac7eaaac` | PASS |
| `cargo clippy --locked --offline --workspace --all-targets -- -D warnings` | same | PASS |
| `cargo test --locked --offline -p rrx-spawn-hygiene -- --nocapture` | same | 20 top-level tests PASS, 0 failed, 0 ignored; nested workers are separate observations |

Commands retained default test parallelism. The direct Cargo children were
waited/reaped and the source stayed clean before/after each command. This is a
local unit result, not proof of broader fixture/group death, macOS V-F/M06,
Linux equivalence, source mutants on the new mapping, CI or live qualification.
The first `ae23b43b` formatting check passed; Clippy stopped with two
`question_mark` findings in the Option report parser and tests were not started.
`e8359628` changes only those two equivalent None-propagation expressions; the
failed attempt remains separate and is not relabeled as success.

The two fresh implementation reviews cover source wiring and security/contracts,
not final qualification. Sol served model/effort are unreported; Opus reports
only `claude-opus-5-5`, with effort unreported. Review body hashes are
`7758552af2724834d798c5d1721e5919bed266506f6fef6a88ab0be3f6baf977`
and `d60e25f3f4a7c250b9cec7f9a6e0f104633e42382368899289db9ca0eea0f1e5`.

New Mac mapping controls at cb2dcd66 have two distinct source operators and
three compiled actual assertion kills with three same-tree restored passes:
Mac install no-op makes API_SPAWN_STD and C1 reject actual inherited writers;
header omission makes the pure invalid-report control reject the unexpected
acceptance. API/C1 preserve worker signal-before-reap and complete ownership
records. The pure case creates no fixture Child. Four actual Mach-O binaries
and source/compiler/log bindings were preserved; the parent checked all 3613
regular archive file bytes and four binary hashes/magic. Only the completed
private-source and targets were removed after live equality and terminal checks;
the original product worktrees and evidence remain. Earlier EOF operators were
not rerun or relabeled to the new source.

Linux local minimal controls did not start: fixed Docker image inspection
returned `No such image`. No Cargo/test/container was launched or certified.
The missing prerequisite and fixed-source plan are retained. This is not a
test success, a waived safety condition or a Linux full-campaign restart.
New Docker resources/build cache were not created. Required final Linux safety
checks remain unqualified; source review is not execution proof.

Final-source M06 numeric conditions are met as recorded below; final macOS V-F,
unresolved failure attribution and safety/CI/owner gates remain unqualified.
Historical checks and failed attempts below retain their original source scope.
GNU C2d/V3/V-F and forced-fork evidence are deferred history under the platform
amendment, not current Linux hygiene or qualification. The historical macOS
UNKNOWN comparison is not reclassified; macOS attribution remains unresolved.

## Final-source M06 and diagnostic correction results

M06 uses product `42440e5c6d9d9c954aee7b5c8257d94a9b4664da`, with only the
existing private project_api trace overlay. Two actual compiler-bound Mach-O
service/test pairs and identical overlay bytes were checked before the campaign.
The pilot is separate and does not contribute to these 100 attempts.

| Fixed macOS campaign | Actual result |
| --- | --- |
| Two jobs, 50 original attempts each | 100 completed; 97 full project_api PASS, 3 FAIL |
| c_s3d4 / m2_a failures | 0 |
| Observable attempts / exact target traces | 100 / 200 |
| B_END / SETUP / NO_RESULT | 0 / 0 / 0 |
| Other failures | job1 reps1, 2, 30: c_s3d2_status_reconciles_only_its_own_project, existing elapsed < 5 seconds assertion |
| Direct child custody | All 1700 recorded Cargo/Git children, including 100 Cargo children, waited/reaped; runner completion does not imply all tests passed |

The c_s3d2 failures and original exit101/logs remain. Source binding identifies
the actual assertion, not its cause; hidden upstream F1 remains unexcluded.
No deadline extension, retry replacement, ignore, skip or serial flag was used.
Trace/lsof observer cost is not natural-load or causal equivalence. Fixture Drop
stop success and complete descendant/group death remain UNPROVEN.

Evidence: `.rrx/p4-macos-final-m06-20261014/results/final-m06-report.json`,
SHA256 `3e988b2663b0d0dd4d523f25b512ac8360c0e81af93afb4a911976c7570f2883`;
all100 parent raw/source/artifact audit SHA256
`cd90c84f63e04100e247a4d9c7b12efca67de8f3374d863eed7c21ed701c9a4e`.
Five archives retain private Git/source history, all original attempts and four
actual Mach-O payloads. Parent verified 11726 regular payloads / 3374377495
bytes against archive and live file bytes/modes; archive audit SHA256
`e4172c689e6f30be2a350de22e9e875265bdc2fe72714d62d970f201ec98fd96`.
Only completed measurement job1/job2 source/target copies were removed after
terminal and preservation checks, with four actual absences confirmed. Logical
removed bytes are 3284341883; observed host free-space increase is 14508032,
not a task-only physical reclaim claim. No Docker resources were created/deleted.
Original product worktrees and all measurement results/archives remain.

The private diagnostic consumer is independently corrected at `cff37a99`, with
64 pure properties, two actual interpreted Python assertion operators and two
identical-source restorations PASS, five direct Python children waited/reaped.
Earlier actual normal selector observations and all unfavorable historical
records remain unchanged. These controls are not native-signal qualification
or Rust native source-mutant evidence.

Initial Sol/Opus findings and subsequent confirmed data errors were checked
against source and controls. After the basic three rounds, finder-side Opus
confirmed only the remaining parent-storage/exception and original-reason
corrections: APPROVE, mandatory 0. Body SHA256
`42d9242c3d0651b078a1e5625c279fdd609973174c1913acfb7dae674c24cc75`.
Runner reports only claude-opus-5-5 matching the requested model; effort is
unreported. Eight originals were preserved by bytes/modes/hash and the completed
review-only temporary directory was removed.

Optional Low observations are retained here rather than opening duplicate Issues:

| ID | Optional improvement / retained limit |
| --- | --- |
| N1 | Add a successful-observation plus parent-write-failure control for status downgrade/original status; current code downgrade is source-verified and summary qualification fails closed. |
| N2 | An owner-start log OSError followed by wait interrupt can preserve OSError as the raised exception; interrupted receipt/NO_RESULT/stop remain, but interrupt exit identity is not guaranteed in that combination. |
| N3 | A known failed parent output path is not retried; an earlier results file may be stale. The surviving summary records that path as unavailable; no retry was added. |
| N4 | Side-result storage failure leaves annotation but can lack original_observation_status and retain an overly broad original_failed_not_discarded marker. |

Arbitrary repeated native signals/cancellation and whole-disk persistence remain
unqualified. This local tooling review proves neither F1 exclusion, root cause,
group death, all nine producer stages nor full P4/Goal acceptance.

Four fresh source-only main/P4 copies for the final full-library comparison are
prepared and parent-audited. Source trees/bytes/sealed modes match fixed 060dadcb
and 8cc759b2 origins. Actual service/lib build, fresh inventory and the planned
32 default-parallel full-library attempts have not started. Empty public cache
is not cache readiness. The initial import path/mode setup failure and separate
source-only continuation are retained. Final V-F, unresolved macOS failures,
Linux required shared-safety/CI, specific owner PR approval and merge remain.

Issue: <https://github.com/shuhei-suzuki/rururunx/issues/89>. Baseline:
`b8344ab951fa24f545174917eee7be067284c625`. Original checks below preceded
`c3fb1bea`; the per-entry-point repair checks use fixed `149cf859`.
This is partial issue work. M06's initial campaign and both V-F executions are
complete; their historical V-F acceptance was unmet (GNU observed F1 increase
and macOS INCONCLUSIVE attribution). The macOS First scope above defers the GNU
qualification; final-source acceptance, owner approval and merge
remain separate gates. F1/F3/F4 remain under
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
| Linux C2d/V3 (initial macOS-only checks) | NOT RUN at that stage; later [GNU follow-up](#gnu-control-and-v3-follow-up) completed |
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
| V3 Linux ENOSYS ignored (initial macOS unit) | NOT RUN at that stage; later [GNU actual kill](#gnu-control-and-v3-follow-up) | NOT RUN at that stage; later GNU same-byte restoration PASS |
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

## GNU control and V3 follow-up

The later GNU validation uses fixed source
`861549995b12b3afe6d46a07d06a47a21b6b83a8`, before the EOF repair. The aarch64
GNU/glibc run uses the immutable Rust image, `--init`, network-none, cap-drop ALL,
no-new-privileges/default seccomp, read-only source/registry and dedicated writable
targets. `controls3.log` records all 18 crate tests PASS with default parallelism,
0 fail/ignore/filter; `c2d3.log` records the exact C2d ENOSYS control PASS with
complete S1/S2/S4/S6/S7 custody.

V3 changes only the Linux common production error-classification branch in
`crates/rrx-spawn-hygiene/src/platform.rs`: original/restored SHA256
`710ce4d74ef223737318eaba3dd98229446352b946e4cd2eb1b403f9347e8423`,
mutant SHA256 `d6db9cc42f20c30c1249199f3ebebe4d072a09a8f4dc73e6734cde5e9fbaa78c`.
`v3mutant.log` records timely `RESULT fail:unexpected Child`, the actual
left-Fail/right-Pass assertion, complete=true and S1/S2/S4/S6/S7; outer exit is
101. `v3restored.log` records the same exact control PASS after byte-identical
source restoration. Normal/restored binary SHA256 is
`8df5ce0c28b2af2fb44b1a1d69a79dc64af915e3ddc70dedaed454f32b5ffdde`;
mutant binary SHA256 is
`a53ca7a51ff84deeb880d7543c2b143a61b68f7021cc824fa12f754b2ef704b7`.
SETUP/NO_RESULT/TIMEOUT and incomplete custody do not count as kills.

Evidence is retained in the main repository's
`.rrx/p4-linux-validation-20261014/`, including `complete-proof.json` (SHA256
`226fcdc3c2be0b69d1a4bac236c854d08d03b9490d2288457fc36726c83288f2`),
`v3-mutant-manifest.json`, normal/mutant/restored source archives, the four logs
above and client-owner/absence records. Independent parent audit
`.rrx/handoff-goal-20261013/linux-controls-parent-audit-20261014.json` (SHA256
`b01095307f1553b545e51cddc13a6efcf8391f4897edca3985368b73c7d29f16`)
confirms the source, nine original clients' logs/wait-reap, container absence,
archive hashes and seven task-path cleanups. The earlier controls2 attempt
without `--init` retained OW2 at S7 with an unreaped PID1 orphan; its container
stop/client exit137 remains Held/unproved recovery, never PASS or killed.
The three initial failed attempts are preserved. GNU fmt/clippy were NOT RUN in
this campaign. This GNU V3 evidence does not certify the later macOS-only EOF
source mutants or close V-F/merge gates.

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
and its restored normal pair passes. At the `149cf859` unit, the existing
inheritance/InnerOwner::close code was unchanged and the separate EOF-reader
classification finding remained unresolved. The later EOF repair and its
distinct validation are recorded below.

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

Remaining classification limits are separate from the EOF repair: unexpected
non-EOF `waitid` errors in `controls::expect_spawn_err` and Tokio Runtime creation
errors in `controls::stdio` can still reach the worker's generic RESULT fail
branch. No permanent pure delayed-EOF positive control was added; the temporary
callback-bypass C1/C2f mutations provide actual delayed-EOF evidence. Synthetic
BeforeExited/AfterExited controls validate classification but do not establish
coverage of the real `observe_live` try_wait(Some)/reaped transition. These are
unmodified limits to reassess before using a generic kill predicate beyond the
source-bound controls; they do not relabel completed EOF evidence as missing or
claim all observation-error paths safe.

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

The service-binary-corrected macOS V-F also completed both 16-attempt variants.
Its separate source-scoped report retains 289 actual failed-test occurrences on
main (50 F1_OBSERVED, 214 OTHER_OBSERVED, 25 UNKNOWN) and 119 on P4 (0, 89, 30).
The result remains INCONCLUSIVE / UNKNOWN_ATTRIBUTION; finite observed counts
do not erase UNKNOWNs, establish equivalent load or identify original FD holders
or a causal mechanism. Original records/initial STOP and all prior attempts are
retained without test reruns. Report:
`.rrx/p4-vf-scoped-audit-20261014/secondary/results/full32-secondary-scoped-audit/report.json`,
SHA256 `8be3ef2bf09be3d538b08a2cddaca1a27d208551fb2e092572b2d8ef23a3beac`.
Parent audit: `.rrx/handoff-goal-20261013/secondary-macos-result-parent-audit-20261014.json`,
SHA256 `a910d003bc27669bd12fb8d86eaa0c68b61631925aa81a800211e7f8a1ed7cd6`.

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
confirmation. Existing Linux C2d/V3 acceptance is bound to the fixed source and
evidence in [GNU follow-up](#gnu-control-and-v3-follow-up). Known Tokio post-spawn Err custody limitations
remain unresolved by this fd boundary. Do not close issue 89 from this partial PR.

## Historical macOS First scope before platform implementation

At this historical checkpoint no platform-split source existed. Later current
implementation and qualification facts are recorded at the top of this document:
- EOF repair `e2886a1a`: macOS and GNU each 20/20 PASS. Eight macOS source mutants
  produced nine actual kills, with nine same-byte restorations PASS.
- Reviews: Sol R2 `fe3f0d3e`, zero required findings. Opus R3 `d8721fba`, APPROVE
  with zero required findings.
- M06 at fixed `c3fb` (100 observable, PASS) is historical evidence only. It does
  not qualify the final source.
- The earlier macOS V-F (`86154999`) stays INCONCLUSIVE / UNKNOWN_ATTRIBUTION:
  - F1: main 50, P4 0. Unknown: main 25, P4 30.
  - P4's 30 Unknowns: Docker partial report 13, audit nested exit 7, Grok nested
    cause hidden 7, explicit timeout 1, watchdog 1, SC10 bound not reached 1.
  - Hidden errors can include owner open failure, so the absence of F1 text does not
    exclude F1.
- GNU V-F (main 0/16, P4 5/16, all 32 receipts) is retained and deferred, not
  relabelled. Whether the GNU increase was caused by the backend remains unproven.
  Linux delegation removes the backend; it does not explain the increase.
- The forced-fork fixture has a required classification defect and no actual runs.
  It is not accepted evidence.

Fresh macOS acceptance, all on the final source:
- controls, affected regressions, fmt, Clippy, mutants as mapped, and M06 50 × 2;
- V-F main/P4, 2 × 8 full rrx-lib runs. The predicate is fixed before
  classification: count failed occurrences and attempts with at least one failure.
  Declare the failure-path mapping before the campaign. A non-increase claim
  needs conclusive source-bound classification; Unknown and Other with unknown
  causes cannot simply be assumed unrelated to F1. Expose omitted errors in a
  symmetric diagnostic overlay and retain unresolved attribution explicitly.
  No new numerical acceptance predicate is introduced by this amendment.
- An optional diagnostic test overlay may run symmetrically on main and P4,
  classified apart from the original attempts.
- Linux: Clippy and crate tests in the existing immutable GNU image, plus the
  required CI.

At that checkpoint platform implementation, final macOS qualification and merge
were UNPROVEN; the current platform unit and M06 do not close the remaining gates.
