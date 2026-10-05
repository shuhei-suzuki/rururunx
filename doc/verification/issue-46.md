# Issue 46 verification

STRICT shared owned-process death evidence. Requirements3 and both Design3 gates
approved. Both Source1 coverage findings were verified fixed; both Source2 reviewers
approved source7611d65 with no Critical/High/Medium findings. Exact source-head
Linux/macOS CI passed; final documentation-head CI remains the merge gate. Earlier
failed runs are preserved and are not superseded as causal explanations.

Issue41 exact `888d86d` macOS CI37169413513 reported `native process inspection
 timed out` from bounded Context Git, followed by uncertainty cascades. Linux was
cancelled by failfast after successful tests/debug. Native Claude separately
observed default-parallel Git5s timeouts. Neither establishes causality.

Read-only installed macOS26.6.2 build25G83 own-group fixture used exactly
`/opt/homebrew/bin/python3.14` with waitid/WNOWAIT. Two fresh session leaders each
owned two children. Explicit `ps -g PGID -o pid=,pgid=,stat=` returned exactly the
three expected members, equalled global-table filtering and excluded the second
owned group. An unreaped zombie leader with two live S children remained visible.
Only those owned groups were signaled; cleanup was verified before leader reap.

Five sequential quiet samples: global1431rows median23.121ms; selected3rows
median3.761ms. These are local observations, not CI-load evidence or a production
performance guarantee. Source/result scripts remain temporary fixture evidence;
public structured evidence will be published with validated formal/implementation
rounds. Published Apple blobs are pinned in the design; installed binary/source
identity is unverified. XNU global traversal/temporary allocation remains explicit.

Historical pre-implementation source safety gap: Apple ps can emit sysctl stderr yet return0, including
initial/exhausted failures. Retry can sleep1s on non-ENOMEM. Current discarded
stderr/empty-as-dead was insufficient death proof. The original read-only observation
changed no shared helper. The implemented contract below keeps250ms, sticky
uncertainty and default concurrency; no historical root-cause claim.

## Requirements review1 and verified refinements

Immutable public `f844d4c` Requirements1 native session
`f5c3cbdd-df13-437f-a863-92ef89f38e2c` completed request_changes; owned cleanup was
verified. R46-1 High is a verified requirement omission: exact-group completeness
must include cross-UID/non-TTY members, which same-UID fixtures cannot establish.
Pinned Apple ps selector under UNIX2003 u03 increments nselectors/xkeep_implied before default-owner
insertion; one-group optimization occurs afterward, then keepit bypasses UID
filtering. XNU PGRP callback has UID/TTY check flags zero. This source-scoped
verification is distinct from installed binary identity or privileged acceptance.

R46-2 Medium is verified: XNU skips SIDL/forking while collecting PIDs under lock,
then fills records after unlock with zombie fallback. The non-atomic fork/exit
window is now an explicit residual limitation. R46-3 Low partial-success KILL
non-claim and R46-4 Low observable error-category precision are also included;
inspection Unknown retains its category and maps to Lost, valid live returns
EPERM, no raw stderr or additional durable permission data. No code or runtime
evidence changed. Requirements2 delta re-review and design gate remain required.

## Requirements2 mode refinement

Public `7c9be4c` Requirements2 completed request_changes with verified cleanup.
R46-5 Medium identifies a missing explicit u03 precondition/source explanation.
Installed compat(5) and verified Apple Libc get_compat.c/h blobs establish
UNIX2003 true when COMMAND_MODE is absent; leading-minus -g does not clear u03.
The earlier fixture script actually used subprocess env={} for both query forms;
the review lacked that source/provenance, so an inherited mode is not attributed
to that observation. Exact argv/environment are now in the public historical
observation artifact, with a script digest. No new privileged or production-helper
acceptance is claimed. Legacy-mode negative acceptance remains required before
implementation acceptance. R46-6 Low removes undefined visible-member wording;
R46-7 Low aligns master/README observational and residual-limit language.
Requirements3 delta re-review precedes design gate/code.

## Requirements3 gate

Independent native Requirements3 at public `d3aa523` completed approve with no
blockers, verified owned cleanup. It verified R46-1..7 refinements from pinned
Libc/ps/XNU source; approval is requirements-only, not design/runtime evidence.
Optional R46-8 exact-env/legacy fixtures and R46-9 actual exit/stderr accounting
are now explicit acceptance conditions. Historical env={} metadata is an executor
attestation, not a committed reproducible fixture; unrecorded exit/stderr stay
null. R46-10 verified pinned sysctl_prochandle needed>copied oldlen returns ENOMEM,
then ps resize/retry/exhaustion surfaces stderr even if exit0. These refinements
are included for scoped recheck in the independent design gate before code.
Raw resumed native token/cost/API counters have unverified per-round attribution.


## Design1 findings verified before code

At immutable public d56f2bf, independent native ownership reviewer24c2b45d-8de7-
43c0-986e-a8bad432b1ea approved with optional refinements; bounded/consumer reviewer
 a92952cc-9d95-4f4f-922d-f106a0ea3a02 requested changes with three Medium blockers.
Both operations completed and owned cleanup was verified before these edits.

BOUND-D46-1 is verified: existing helper joins the reader after timeout and does not
bound EOF after inspector exit. Design2 chooses a single-thread nonblocking drain,
retained-writer causal fixture and explicit mandatory blocking reap outside the
unchanged250ms observation budget. No detached reader/new reader-spawn gate remains.
Pinned Rust1.91.1 pipe.rs confirms macOS pipe+separate CLOEXEC race; this is a residual
inheritance possibility, not a measured cause of existing CI failures.

BOUND-D46-2 is verified mutant misattribution: a removed/global selector commonly
causes rejection and does not prove false death. Design2 separates those liveness
controls from safety-critical -g→-p leader-only zombie/live-child false-death operator.
BOUND-D46-3 is verified: current fail_cleanup skips signal/resolver/inspection and
cannot prove Unknown consumer propagation. Explicit per-invocation macOS test plans
now target real resolver, bounded Git/Context isolated latch and native Grok terminal
consumer; injected PERM scope remains distinct from real native signal proof.

Optional OWN/BOUND refinements specify legacy argument ordering, direct Grok helper
consumption, constructor process_group(0) obligation, two-window Drop retry/blocking
residual, production-shaped fixtures, actual exit/stderr/state spelling and macOS-only
conditional compilation. Apple print.c/keyword.c formatter blobs and Rust pipe blob
were fetched from primary public repositories and pinned; installed/source identity
remains unverified. Requirements4 wording aligns the approved fallible-reader intent with fallible
nonblocking pipe setup and explicitly scopes the observation/reap deadline; no
timeout increase is introduced. The combined requirements/design alignment delta
is included in Design2 before code. No runtime code/tests changed.

Issue41 final63bbd1d exact CI37177404219 preserved Linux success/macOS failure: both
Grok failures reported native process inspection timed out after library, adapter,
CLI, Context and Git tests passed. Earlier ba70c3f/bab6f54 both-platform successes do
not supersede this exact failure. No rerun/dummy evidence commit or root-cause claim.


## Design2 verified disposition and Design3 delta

At7ce6f01 both native Design2 operations completed, owned cleanup verified. Ownership
reviewer24c2b45d approved with optional refinements; bounded reviewer a92952cc requested
changes solely for Medium D2-1. Both identify the same verified fixture ownership gap:
a consumer reaps/drops its only leader while a PERM-without-KILL live child survives.
Design3 moves selection/legacy safety mutants to fixture-owned ProcessGroup/resolver,
retaining the unreaped leader through real cleanup before reap; actual Lost consumer
plans always perform owned real KILL before injecting observational Unknown. Credit
is explicitly separated; no unsafe numeric PGID cleanup after consumption/reap.

OWN/BOUND-D2-2/4 retained-writer controls now use an owned completion thread/watchdog,
close writer and join before assertion; valid all-Z frame also kills missing-EOF.
OWN-D2-2 and BOUND-D2-5 pin tasks.c d2fcb07.. (table " RUSITH?", no Z), limit suffixes
to documented/pinned emitter, reject E after Z and unsupported states. OWN-D2-3 adds
exclusive read/write endpoint provenance as residual: bounded EOF does not close
foreign-byte injection/theft across the non-atomic macOS CLOEXEC window. OWN-D2-5
non-retriable try_wait error never authorizes numeric signal. BOUND-D2-4 always attempts
wait after kill error; wait failure retains explicit cleanup uncertainty. D2 legacy
control is actual exit1/nonempty stderr; master separates observation from unbounded
mandatory reap. macOS all-target lint checks compiled seams; Linux verifies no references
to compiled-out items. Generic Unknown/Lost consumer is explicitly included.

Optional BOUND-D2-3 poll(2) module change is declined to keep the shared5ms polling
contract/scope; overflow fixtures assert size categories with otherwise-valid frames,
and any timing/guard masking receives no credit. OWN-D1 read-only foreign-UID query
remains optional/unrun; pinned source supplies limited cross-UID selection evidence,
not installed identity/privileged signal proof. Other OWN/BOUND-D1 refinements were
adopted in Design2/3: owned group selection, exact argv/mode, primary formatter evidence,
construction obligations, direct Grok use, Drop windows/residual and macOS cfg scope.
Raw resumed usage/cost/API counters remain unverified per-round attribution. No code
changed; combined requirements alignment was consistent in both Design2 assessments,
and this narrowed Design3 delta must approve before implementation.


## Design3 gate and implementation refinements

Both independent native Design3 reviewers approved exact8aa434f with no blockers;
owned cleanup verified before edits. Optional findings are applied in implementation:
idle-only5ms sleep (data flow unthrottled), no signal or numeric wait after ECHILD/
non-retriable ownership loss, canonical ordered suffixes, independent fixture-local
killpg/member handles (not mutated cleanup), single-process Unknown consumer shims,
legacy exec shim, generous retained-writer release watchdog. Workspace Cargo.toml
already enables rustix fs; source review will include manifests to verify that fact.
Mandatory reap/endpoint/sampling/global-kernel residuals remain explicit. Code/tests
are not accepted by design approval; causal fixtures/mutants/source reviews remain.


## First inspector fixtures and invocation diagnosis

Committed17c7e21 targeted default four-test run: one passed, three failed. Retained
endpoint fixture referenced nonexistent macOS /bin/true; fixed to a trusted shell
producer. Tiny diagnostic and stdout1MiB shebang fixtures returned actual250ms timeout
(~252.7ms for overflow); no deadline/latch/concurrency relaxation. Isolated committed
producer diagnostics fe1f989/61c9a4a used env_clear, owned waitid/WNOWAIT and verified
cleanup-before-reap: direct temporary shebang small/giant/chunk scripts330–359ms
after1.5ms spawn; direct /bin/sh -c8.38ms, /bin/sh file7.43ms, /bin/ps3.43ms.
This identifies local fixture invocation delay, not its OS mechanism, pipe-inheritance
causality or historical CI cause. Peer full process suite overlapped first fixtures;
its presence alone does not establish load causality, later diagnostic window was quiet.

Private command construction seam now permits direct trusted interpreter prefix for
synthetic tests; production remains only absolute/bin/ps with no prefix and exact
shared -g/-o/env_clear/stdio/deadline gates. Tests generate chunks rather than a giant
shell literal, and the owned producer emits the retained endpoint's frame. Retained
endpoint uses an owned UnixStream FD as stdout through the same File drain completion
runner, so credit is descriptor EOF/IO causality, not a privileged ps or signal proof.
Default controls/source review must still verify the correction; original failure stays.

### Consumer fixture boundary

Per-invocation macOS test-only plans route the existing ProcessGroup signal resolver through the real bounded inspector. Consumer plans first issue actual KILL against their owned unreaped group, then inject PERM and one fixed Unknown observation; production has no plan, environment override, or prefix. Only the inspection module's retained-leader fixtures can create the no-KILL live-group plan. Their cleanup independently signals the still-unreaped group and waits the directly owned member before reaping the leader, so a mutated selection/resolver cannot erase fixture cleanup authority.

Git and Context preserve their existing production entry points and process-wide uncertainty policy; a private core lets Context tests use an isolated latch rather than clearing or poisoning the process-wide production latch. Generic launch and Grok supervision install plans only on the actual native executor child after preflight. The Grok child consumer is entered through an env-cleared re-execution of the test binary and calls the unchanged public constructor with a synthetic HOME/PATH. It exercises failed initialization and cleanup reservation, not installed Grok inference or credential acceptance. Production Grok environment policy is unchanged; Issue 51 remains separate. Committed controls and mutation outcomes are recorded below; independent source review remains pending.

At committed `1112268`, macOS all-target clippy with denied warnings passed; the seven inspection tests passed with default concurrency (0.26s). The default-concurrency `unknown` filter passed nine tests but failed the sanitized Grok parent watchdog at 60s. Source inspection verified a fixture-only self-deadlock: its post-terminal Store mutex guard remained held while `transport_succeeded` called `entry` → `assert_saved` and tried to acquire the same mutex. The fixture now drops that guard first. This is not an inspector deadline or native Grok defect, and the failed run is preserved. The parent completed actual group cleanup and reap before asserting its timeout. The corrected fixture requires a new committed control; no pass is claimed here.

## Committed consumer controls and causal mutations

Corrected `900e79d` default `unknown` filter: 10/10 passed (1.65s). Source `bab243a4add0a636f12f798e498d4eab99a73a97` full default-concurrency macOS workspace debug and release suites both passed: 182 Rust tests plus two doctests, three explicit ignored entries (two installed Grok/auth acceptances and the sanitized child-only entry; the ordinary parent invokes that child). Release compilation completed in 30.60s. These are current synthetic/native-process controls, not new installed-agent inference acceptance. Non-test debug/release build and latest all-target lint gates remain required before readiness.

The isolated detached mutation worktree compiled 12 test runs from 11 distinct source operators. Every run failed its intended actual assertion; none earned compile-error or timeout-without-cleanup credit. Exact base-to-mutant patches, heads, log hashes and causal excerpts are checked in [the mutation ledger](issue-46-mutations.json). The stderr-removal source operator ran separately through Generic and the env-cleared synthetic Grok consumer (same committed mutant `c732c89`): Generic changed Lost→Exited, Grok changed Lost→Failed. Missing expected leader exposed successful bounded Git output instead of SessionLost; ignored live members, leader-only selection, and legacy argument reordering cleared actual retained group ownership. Removing EOF accepted a frame while its writer was held; removing NONBLOCK exceeded the independent five-second release watchdog, which released the FD and joined/reaped before asserting. Disconnecting each adapter's private seam changed its expected terminal result, proving caller wiring. Clearing Context uncertainty failed the real core's sticky-latch assertion; this is scoped to that state assertion and the restored later-launch refusal, not a separate compiled proof of every public index caller. Global enumeration earned liveness-only credit: the all-zombie owned group could no longer be accepted because foreign rows were rejected. It does not prove false-death safety.

Exact production source restoration at detached `ed69214c75c1b3708416fa5c4cfba0eb7d924e46` was byte-identical under crates to `bab243a`; restored default controls passed seven inspection tests (0.26s) and ten Unknown-filter tests (1.64s; overlaps the inspection group). The clean mutation worktree was normally removed after completion. Mutation patch metadata was normalized against the fixed original base, including the repeated stderr operator, rather than against the preceding mutant. The original unsuccessful fixture/test runs remain above; no historical CI cause is inferred. At `749039f`, the added safe directory-FD fixture passed with all eight inspection tests at default concurrency (0.26s), covering complete() read-error handling and Inspector::cleanup kill/reap directly. It does not execute observe_command's kind-preserving cleanup-error arm; that arm and fallible fcntl configuration ownership remain source-reviewed only. No synthetic syscall failure is advertised as real OS proof.

Independent immutable source reviews and exact-head Linux/macOS CI are still pending. Future native5/6 callers require normal integration and their own real consumer gates; no acceptance is inferred from source compatibility.

Latest `749039fa48a3c03b90dd8ab8e272f177d21a8edf` gates: fmt check, macOS all-target clippy with warnings denied, non-test debug build and non-test release build passed (clippy1.32s/debug1.56s/release5.02s). Compared with the full debug/release suite at `bab243a`, only an extra private test and evidence were added; current production source is unchanged. Latest complete workspace execution will be supplied by exact-head CI, not inferred from the earlier suite's 182-test count. Source-review input includes the immutable complete changed helpers/callers, manifests, formal requirements/design and honest failing/restored ledger; no model/auth acceptance or historical cleanup-cause claim is added.

## Source1 verified findings and exact CI failure

Both fresh independent Source1 reviewers completed at immutable public `9a3761e96323a954b46a0ed8ededd3012c8f68be` and requested changes; owned cleanup verified (ownership UUID `0711f83d-3d32-4847-9204-fb37d599932d`, bounded UUID `fa90a4cb-9c58-4f79-b39f-d3c2be3ff837`). They found no production safety defect but verified Medium coverage gaps: the live-member signal seam skipped the production observer; real ps samples lacked actual exit/stderr/raw-state/exact-membership/global-baseline records; exit/timeout negative plans had empty stdout masking the intended guards; compiled framing/foreign/duplicate/exit/size/deadline operators were missing. Existing 12-run ledger retains its narrower claims. Low diagnostics originally replaced the observation category on reap error; stale documentation described code as only proposed.

Fixes now in progress: signal-only live plan falls through to the real absolute production ps construction; only legacy uses a shell, with exec `/usr/bin/env -i COMMAND_MODE=legacy /bin/ps` to remove shell-added environment. The common frame observer preserves identical argv/env/stdio/budget/kill/reap policy and exposes private metadata to tests; production exit/stderr/frame gates remain authoritative. Actual owned rows and independent fixed global query membership are recorded without unrelated global process data. Valid expected-leader frames precede exit1/timeout and a new partial frame, isolating the intended guards. Read-error/mandatory-reap failure preserves original bounded kind plus static cleanup diagnostic. All changes require committed controls, new causal operators and delta re-review; none is accepted by this paragraph.

Exact CI [37186662264](https://github.com/shuhei-suzuki/rururunx/actions/runs/37186662264) on9a: macOS job111389910712 fmt/clippy passed; lib86passed/1failed/1ignored, stopped at shell-loop stdout overflow returning actualTimedOut253.872ms. Ubuntu job111389910849 fmt/clippy/test/debug build passed; release cancelled by fail-fast, so no final Linux success is claimed. Full failed log preserved locally. No rerun/deadline/concurrency change. Isolated synthetic producer diagnostic committed temp `e32d44e`: same env-clear single-process programs observed shell130printf loop27–30ms/~1040read chunks, fixed-width awk stdout8–17ms/~129chunks and stderr14–15ms/~129chunks; all exit0 and cleanup-before-reap verified. This measures local producer shape, not the CI mechanism or any historical ps cause. Test-only overflow generation now execs the single trusted awk producer with bounded constant output, leaving the shared production observation contract unchanged.

## Source1 fix controls and second compiled mutation round

At `dbfa35519b7a64fa7d1a1ded6a216abb737c8225`, corrected default inspection8/8 passed0.26s, and actual Unknown-filter10/10 passed1.66s. Full default debug and release workspace suites passed183Rust+2doctests, with three explicit ignored entries unchanged. Native metadata emitted by the full debug run established selected/global filtered exact sets: leader47663Z/member47673S; all-zombie47667Z/47672Z; excluded other live group47713Z/47719S. Production queries exited0 with stderr0. Legacy frame exited1 with stderr190, using env-i with onlyCOMMAND_MODE=legacy. Actual stdout/stderr size categories arrived21.5/22.5ms in the targeted run; retained-writer timeout251.38ms. These are synthetic owned-process observations, not cross-credential or installed-agent proof. The test-only metadata now additionally records Command getter program/argv/explicit-env-entry count before actual spawn, while the environment reset is source-enforced; no kernel environment attestation is claimed.

The [second compiled ledger](issue-46-mutations-round2.json) adds eight operators: four actual resolver/Git consumer kills (production observer forced true, exit status, missing trailing newline and central observation deadline); two validator-only foreign/duplicate operators; two size-detector runs masked by independent completeness/diagnostic rejection and explicitly credited as none. All compiled and reached actual assertions; size masks are not claimed as false-death or bounded-memory breaches. Central deadline removal accepted the otherwise-valid frame after its owned sleep2 exited; it is one reused deadline-policy operator, not a claim of independently mutating every call site. Restored exact source `0c14c94297bbecb16e5d927618fddd1de96b8061` passed default inspection8 and Unknown10 controls; the clean detached mutation worktree was normally removed. Exact normalized base-to-mutant patches and failed excerpts remain in the ledger. No deadline/latch/default-parallelism change.

Postfix all-target clippy atdbfa rejected one needless borrow in the new literal-string producer fixture (`builtin(&body)`); correction is `builtin(body)` only, not a runtime failure or process-inspection cause. Latest lint/build/metadata controls and source delta reviews remain pending before readiness.

At `30e5e48`, latest metadata controls8/8 passed0.26s; all-target macOS clippy-Dwarnings, fmt check, non-test debug and release builds passed (clippy1.41s/debug1.71s; release5.36s). [Measured owned observations](issue-46-owned-observation.json) parse actual fixture emissions, separating source/binary/argv-environment provenance; historic recon nulls remain untouched. Program/args/explicit-environment count come from actual Command getters before spawn; environment reset is source-enforced, not kernel attested. No unrelated global process rows or native credentials appear. Production safety policy and earlier full default debug/release183Rust+2docs controls remain unchanged; final exact published-head CI and independent scoped Source2 approval still required.


## Source2 approval, exact source CI and Low dispositions

Both resumed independent native Source2 operations completed at immutable public
`7611d656ab15e63ba7a5552c38804c9f214877bb`, approved with no Critical/High/Medium
finding, and verified owned cleanup. Ownership reviewer
`0711f83d-3d32-4847-9204-fb37d599932d` and bounded reviewer
`fa90a4cb-9c58-4f79-b39f-d3c2be3ff837` independently verified the Source1 fixes and
preservation of Unknown/Lost, kill/reap, exact selector and shared250ms/1MiB policy.
Their resumed usage/cost/API duration counters have unverified per-round attribution;
no measured wall time or final reviewer-run test result is inferred. The prompts
still listed CI pending; separate exact source CI below closes that merge gate.

Exact [source CI37189195160](https://github.com/shuhei-suzuki/rururunx/actions/runs/37189195160)
passed both required contexts: check (macos-latest), job111397635471, and
check (ubuntu-latest), job111397635538. Both ran fmt, all-target clippy-Dwarnings,
full workspace tests with default concurrency, debug build and release build. Local
full debug/release183Rust+2doctests counts remain scoped to dbfa355, with the three
explicit ignored entries described above; the exact CI independently verifies the
latest source without attributing that old count to a newer binary. Final outcome
text changes no Rust source; its exact published-head CI is still required before merge.

Optional Low dispositions are verified and bounded:

- Diagnostic-only global -A samples still use the shared250ms/strict parsing path.
  This can fail a fixture on slow or unusual unrelated rows. Current exact source CI
  passed; no separately relaxed diagnostic budget, parser or test serialization is
  introduced. Production selected -g observation is unchanged, and global rows never
  grant signal authority. Preserve any future such failure without attributing it to
  production unsafety or historical CI causes.
- Command getter explicit-environment count alone is not a causal env_clear proof.
  The reset remains source-enforced; no kernel attestation or compiled env_clear-
  removal operator is claimed. A legacy inheritance failure remains Unknown.
- The directory-FD fixture covers complete()/Inspector::cleanup directly; mandatory
  wait-failure kind preservation is source-reviewed only, with no injected syscall
  execution claim. The broader earlier sentence has been narrowed.
- Round1 leader-only, legacy-order and ignored-live-member operator credit belongs
  to bab243a's former shell-shim fixture path. Those three operators were not rerun
  against the current production-observer fixture. Round2's actual production-death-
  observer bypass operator supplies current consumer wiring evidence; it does not
  retroactively move the earlier operators' credit to this head.
- Both size-detector operators remain masked and earn no causal false-death/bounded-
  memory credit. The optional valid-row-at-byte-boundary fixture is not implemented.
  Trusted ps whitespace parsing and the lock-refusal assertion's missing positive
  control remain optional refinements; Lost assertions supply the documented credit.

The complete two-round ledger is20compiled runs/19distinct operators:15actual-path
assertion kills (including the scoped Context core state assertion),2validator-only,
1liveness-only and2masked/no-credit runs. It does not claim every operator is current-
head false-death safety proof. Both restored detached worktrees were normally removed.
Native5/6/41 integration and installed native/auth/inference acceptance remain separate.
Cross-credential visibility/source identity, global kernel allocation/traversal,
non-atomic sampling, partial KILL/group escapes, FD provenance and mandatory reap/
async Drop residuals remain explicitly unclosed.

### Separate Issue23 baseline failure

Docs-only Issue23 headD1aad526b [CI37188880032](https://github.com/shuhei-suzuki/rururunx/actions/runs/37188880032)
passed Ubuntu but failed macOS Context target9passed/7failed. Its first local_map
failure atline185 reported SessionLost: native process group cleanup failed: native
process inspection timed out; the following six failures reported the earlier
Context Git cleanup uncertainty sticky latch. Library75, Codex19 and CLI5 passed
before that target. These are one observed timeout and six subsequent refusals,
not seven independent process inspection failures. The original full failed log is
retained at `/private/tmp/rururunx-issue23-design1-ci-failed.log`. No rerun, attribution
to the local producer diagnostic or claim that #46 reproduces/closes this exact
historical failure is made. Actual consumer coverage and later integration must be
assessed independently on their own meaningful published heads.


### Later Issue41 test-evidence precision

Issue41 test-only3d30399 renamed the historical `each_output_stream_has_a_causal_size_failure` wrapper to `oversized_child_streams_remain_unknown_when_size_or_deadline_wins`: oversized OS producers may reach the unchanged250ms deadline before the size category, so only size-or-actual-timeout Unknown/no accepted frame is required. This wrapper supplies neither causal size-cap credit nor incidental throughput/idle-only-sleep liveness coverage; cleanup is unchanged source behavior, not a separate wrapper assertion. The earlier round2 test name and timing records remain historical. Deterministic prepared reader-boundary cap evidence is separately unit-only in [Issue41](issue-41.md) and its [cap mutation ledger](issue-41-cap-mutation.json); neither former masked size operator gains retrospective credit. Production Issue46 bounds and safety contract are unchanged.
