# Process descriptor hygiene decision record

Issue: <https://github.com/shuhei-suzuki/rururunx/issues/89>. Source baseline:
`b8344ab951fa24f545174917eee7be067284c625`.

This record synchronizes the approved public design:
[v3](https://github.com/shuhei-suzuki/rururunx/issues/89#issuecomment-6142021167),
[v4](https://github.com/shuhei-suzuki/rururunx/issues/89#issuecomment-6142993780),
[fork supplement](https://github.com/shuhei-suzuki/rururunx/issues/89#issuecomment-6143186286),
[v5](https://github.com/shuhei-suzuki/rururunx/issues/89#issuecomment-6143883600),
[v6](https://github.com/shuhei-suzuki/rururunx/issues/89#issuecomment-6145645072).
Later versions override earlier versions. Independent design approval is
[recorded separately](https://github.com/shuhei-suzuki/rururunx/issues/89#issuecomment-6145821412).
Implementation review/test success remain distinct gates.

## Boundary and source-to-sink analysis

Six safe consuming functions install one pre_exec callback and delegate to the
original std/Tokio spawn/output/status method. Tokio output/status evaluate the
original method synchronously before returning their future; using async fn would
move spawn outside current caller locks. Stdio defaults/overrides, group,
kill_on_drop, cancellation and Child adoption remain original. Unsafe operations
are confined to the new crate's platform module; fixtures use safe rustix APIs.

| Source | Sink | Safety rule |
|---|---|---|
| Caller-owned Command | original method | move once; retain executable/args/env/cwd/stdio/group |
| macOS size probe | aligned Vec/c_int bytes | positive aligned probe; checked arithmetic; parent allocation |
| macOS byte return | record slice/fcntl | reject nonpositive, unaligned, oversized or full returns |
| inherited fd >= 3 | CLOEXEC | F_GETFD/F_SETFD; only F_GETFD EBADF is skipped |
| Linux kernel | close_range CLOEXEC | preserve any failure errno; no fallback |
| harness owned Child | signal/reap | only un-reaped original leader confers authority |

Owner, other user, administrator and unauthenticated caller permissions are
unchanged: the boundary accepts an already-authorized Command and confers no new
command/signal permission. Marker/CAS/Task.version/session authorization are
unchanged. Kernel FFI values are validated independently of caller inputs. New
dependencies reuse existing workspace libc/Tokio/rustix/tempfile versions; the
registry package/version set in Cargo.lock remains unchanged.

macOS uses max(records*2+64,1024), checked count/byte arithmetic and c_int range.
Its aligned initialized proc_fdinfo buffer is allocated before fork. The child
allocates no memory, takes no locks, creates no fd and never resizes. The pure
classify_listfds function rejects negative/zero/unaligned/oversized/full byte
returns, saving errno or EIO; full capacity is ENOBUFS. fcntl sets CLOEXEC rather
than closing fd. Linux GNU uses SYS_close_range(3,u32::MAX,CLOEXEC), failing closed
on ENOSYS/EINVAL and any other errno. Unsupported targets refuse with ENOSYS.

The v3-approved Linux support baseline is GNU/glibc >= 2.34 and kernel >= 5.11:
[close_range history and CLOEXEC availability](https://man7.org/linux/man-pages/man2/close_range.2.html).
The source uses a raw syscall, not the glibc wrapper; no runtime glibc version
rejection is implemented. Musl/other targets and blocked/unavailable syscall
environments fail closed. README and master describe those current facts.

pre_exec forces std's fork path; V-F measures performance/frequency consequences.
Hygiene cannot prove process death. Tokio build_child registration Err after std
spawn can still leave a child. Existing uncertainty/Unknown/Lost logic stays in
force; this limitation is retained rather than claimed fixed.

## Impact analysis and consumer migration

All Command methods across rrx src/tests and process-tracker tests move through
the boundary. Clippy disallowed-methods detects raw std/Tokio spawn/output/status
over all targets. Raw exceptions are confined to the new crate with local reasons:
original method delegation, test injection, regular-file raw worker harness and
OW2 synthetic member. No production allowlist or warning suppression is added.

Spawn stays in original preflight/git-owner/store locks and Err branches. Child
is immediately stored in the existing ProcessGroup, NativeChildCell,
RetainedRawProcess, OwnedProcess or standard Child owner. OwnedProcess::spawn
consumes Tokio Command; its consumers are execution/git_io, retained_io, native,
docker, IPC, verification and routing tests. Test-only capture/capture_observed
also consume Command; results::git and routing tests move the configured value
without cloning/reconstruction or a new unlocked gap.

Inspection and Grok fixture files receive only mechanical launch migration;
inspection/owner/watchdog/timeout/environment semantics belong to
[CI 品質の収束](https://claude.ai/code/session_018R7Q39hySMcnivGdtBcyJC).
NativeSessions status, Claude decoding, metrics and M0 logic are unchanged; shared
files receive only Command migration. RRX_P4_CASE/RRX_P4_DIR are private test worker
inputs and confer no authority. No schema/UI/runtime response/production env change.

## First implementation review: per-entry-point controls

P4-I-M01 adds six separate controls for the six safe public APIs. Each runs under
the existing WorkerGroupOwner and calls that public API directly. Before creating
the Tokio runtime, safe OwnedFd allocations reserve slots 3..9; four owned writer
witnesses have CLOEXEC cleared and their own fstat device/inode recorded. No dup2
overwrites an existing fd. A finite Python probe runs with `-I -S`, validates its
typed argv, and fstats all witnesses before opening its regular-file report.
This makes output/status a completed child-side observation rather than live EOF.

The bounded UTF-8 TSV report must match the complete unique slot set, process
group observation, legal tokens and successful child exit. Only a valid report
of an identical inherited writer yields RESULT fail. Setup, read, launch/wait,
malformed report and observation failures yield SETUP or NO_RESULT; they never
kill a mutant. At the `149cf859` per-entry-point unit, the inheritance reader and
InnerOwner::close were unchanged; the later EOF observation repair is described
below. Stdio/cancel/kill_on_drop controls
remain in place, and optional unpolled Tokio controls are not made mandatory.

Six V1 variants bypass prepare at one public API at a time, each paired with its
own exact outer control and restored normal source. Dedicated targets and source/
binary hashes are required. After the immutable initial M06 campaign finishes,
fixed `149cf859` passes all 18 controls, fmt and workspace all-target Clippy. All
six individual bypass mutants yield an actual failing assertion with complete
custody, and all byte-identical source restoration pairs pass. A separate pure
unknown/error report guard mutant is detected and restored. The initial Darwin
signed st_dev compile failure is retained; a checked conversion refuses negative
identities as SETUP. See the verification ledger for hashes and remaining gates.

## Controls and custody

C1/C2a/f use a live stdin-waiting child, a pipe writer with explicit CLOEXEC flags,
bounded ready/EOF polling and read==0 EOF, with liveness before/after. C2b expects
ENOENT; C2c EIO without marker; Linux C2d ENOSYS; macOS C2e actual one-record-full
ENOBUFS; C2g checks stdin/stdout/stderr and std/Tokio method semantics. Injection is
cfg(test) only. Unexpected Child is killed/reaped before failure. Worker-local
WNOWAIT verifies no child/zombie remains after expected std Err.

The raw worker is its own process group with regular-file stdout/stderr and null
stdin. Exactly one worker test is selected, with no test-threads override. A sole
WorkerGroupOwner retains Child/PID/result directory/deadlines/evidence, confirms
P>1 and getpgid(P)==P, observes with WNOWAIT, sends final group signal on all paths
before a single reap, and uses only signal 0 after reap until ESRCH. Every real
signal has fresh same-thread WNOWAIT confirmation. Result text/PIDs never confer
authority. macOS S4 EPERM permits reap but only S7 ESRCH permits completion; S3
EPERM never proves stop, and non-macOS S4 EPERM never authorizes reap.

The deadline starts before spawn (20 s), with 10 s recovery. Group failure permits
only original-leader stopping/reaping and then permanent Held. Observation/wait
authority loss permanently prohibits real signals or wait. Missing ESRCH/expired
recovery preserves custody. Held may be indefinite: no assert/return/Drop completion
or detached owner before all five gates. Drop invokes the same collection loop.
No third watchdog or all-OS bootstrap hard bound is added. Late completion fails.

SETUP, NO_RESULT, RESULT pass/fail and TIMEOUT remain distinct. Only timely RESULT
pass passes; only timely RESULT fail kills a mutant. Registered V9 stall requires
TIMEOUT plus successful S3, later WNOWAIT exit, accepted S4, SIGKILL reap and S7
ESRCH. SETUP/NO_RESULT/unproved recovery never count killed. OW1 covers pure state
decisions including the inexpensive v6-L03 EPERM cases. OW2 leaves a live synthetic
member and checks common S2/S4/S6/S7 order. V1–V8 alter installation/count checks/
failure/stdio/close/errno/raw migration/zero return; V9 stalls pre_exec. Mutation
copies use dedicated targets and recorded source/binary hashes/restored normal
controls. Mutants deliberately entering permanent Held are not run.

## EOF observation contract

Ready, EOF, read/poll/configuration error, cancellation, actual read-attempted
deadline, transport receive/disconnect/timeout, reader join/panic and the two
child-liveness observations are distinct typed facts. The read FD is made
nonblocking; a poll timeout alone never proves a read attempt or delayed EOF.
The channel observation and the single completed join must agree, including
successful delivery. Missing/duplicate join, reader panic, contradictory delivery,
transport failure, I/O error and cancellation produce NO_RESULT. Failed ready,
group or liveness prerequisites produce SETUP after observation-error checks.
Only ready + live before/after + received/joined EOF passes; delayed EOF requires
the same prerequisites and a genuine EOF-purpose deadline with an attempted read.

Inner child/thread collection precedes classification. Actual child-observation
or collection loss retains the inner owner for outer containment; it does not
permit new signals from logged identifiers. Synthetic observation-fault controls
do not establish actual kernel authority loss. All 17 fault cases have a normal
real-worker pair. They assert complete outer custody, the typed witness and that
SETUP/NO_RESULT are never mutant kills, including the registered-stop parameter.
The existing WNOWAIT/final-group-signal/single-reap/ESRCH contract is unchanged.

## Verification state

Local macOS compilation, fmt, all-target Clippy, 11 new-crate controls, 91 related
integration tests and tracker regression pass. V1/V2/V4/V5/V6/V7/V8/V9 and OW1
gate/authority/classification source mutants are detected and restored controls
pass. The later GNU follow-up at fixed
`861549995b12b3afe6d46a07d06a47a21b6b83a8` records all 18 controls and exact C2d
PASS, V3's actual RESULT fail/assertion with complete custody, and byte-identical
restored-control PASS. Its platform/source/binary hashes, raw logs,
`complete-proof.json` and independent parent audit are bound in the
[GNU follow-up ledger](../verification/issue-89-p4.md#gnu-control-and-v3-follow-up).
These precede the EOF repair; the new EOF source mutants are macOS evidence.
Initial full execution regression
has 186 pass, two Docker failures and one existing ignore; focused Docker replay
is 11 pass on both P4 and fixed main. The first failures remain recorded and their
frequency/cause is not certified. See [verification ledger](../verification/issue-89-p4.md).

M06's initial fixed-c3fb macOS 50 x 2 campaign passes its numerical conditions:
100 observable attempts, no target failures or observable B_END, and all product
runs PASS. Both private overlay copies are archived/hash-checked and removed.
Later cfg(test)/document changes leave measured production and original binaries
unchanged; measurement is not relabeled to a newer commit. The observer's extra
children/scheduling/FD effects remain limitations. The EOF observation repair
has local macOS/GNU normal controls and eight macOS source-mutant/restoration pairs;
its independent repair review is pending. V-F's observed GNU F1 increase and
unresolved causal attribution remain merge STOP conditions. Measurement uses a private overlay so measurement-only
259a467 is not merged. No all-OS, process-death or full issue-completion claim is made.

Both macOS and GNU V-F executions are complete, while acceptance remains unmet.
The macOS source-scoped comparison remains INCONCLUSIVE / UNKNOWN_ATTRIBUTION
despite completed 16-attempt variants; its exact report and parent audit are
referenced in the ledger. Completed finite observations do not establish equal
host load, original FD ownership or the failure's causal mechanism. Historical
NOT RUN entries retain their original unit scope and point to later evidence.

## Platform scope amendment (macOS First)

Status: platform separation implemented. Local macOS formatting, workspace
all-targets Clippy and 20 hygiene tests passed at `e8359628`; initial independent
Sol/Opus implementation source reviews at `cb2dcd66` both APPROVE with
mandatory 0. Final macOS qualification, Linux safety execution and merge are
UNPROVEN.
The design amendment at `aed6f6b4` passed fresh independent Sol design R1 with
zero mandatory findings. This section supersedes these earlier statements in this record (the
v3–v6 text above is kept as reviewed history):
- the Linux `close_range` source-to-sink row and the GNU raw-syscall backend;
- the GNU/glibc >= 2.34 / kernel >= 5.11 support baseline;
- the musl ENOSYS refusal, and "README and master describe those current facts";
- "pre_exec forces std's fork path" (now true on macOS only);
- Linux C2d and GNU V3 as acceptance;
- the GNU F1 increase as a merge STOP for this PR;
- "its independent repair review is pending".

The EOF repair has its reviews: Sol R2 at `fe3f0d3e` with zero required findings,
and Opus R3 at `d8721fba` (document repair) APPROVE with zero required findings.
Whole-P4 final scope has not passed or merged.

### Decision
The `platform` module splits only at cfg function boundaries:
- `target_os = "macos"`: `install`, `install_with_options`, `Options`
  (`errno`/`capacity` under cfg(test)) and `classify_listfds`. The callback order,
  probe, capacity and checked arithmetic are byte-for-byte unchanged in meaning.
- `target_os = "linux"`: `install` returns `Ok(())` without touching the Command.
- everything else: `install` returns ENOSYS.

The GNU close_range body is removed from production source. It stays as
experimental evidence at its fixed SHAs only. `lib.rs` keeps its six signatures and
`prepare_*`. Only the module doc changes, to say hygiene is macOS-only. Every item
compiles warning-free per target: no allow(dead_code), cfg_attr suppression or new
production allowlist. Cargo.toml, Cargo.lock, clippy.toml and ci.yml are unchanged.
Auth, CAS, owner, marker, #19 and process custody are unchanged.

### Test mapping (each worker case declares its per-target expected classification first)
| Control | macOS | Linux |
|---|---|---|
| C1, C2f, LISTFDS, C2c, C2e, V9 stall | hygiene/injection, unchanged | not compiled; there is no callback to inject into |
| C2a (explicit CLOEXEC writer), C2b ENOENT + WNOWAIT no-zombie, C2g stdio/method semantics, OW1/OW2 | unchanged | same controls against the original operation |
| API_* ×6 | all witnesses closed → pass; inherited → RESULT fail | wrapped and direct-original reports both fully validated and equal, with all four cleared witnesses inherited → pass; valid but unequal → RESULT fail; otherwise NO_RESULT |
| 17 EOF fault/normal pairs | writer CLOEXEC cleared (unchanged) | test-owned CLOEXEC writer; classifier/error semantics unchanged |
| Mutants | V1 ×6 bypass, V2, V4–V9, 8 EOF | L1: re-enabling the old GNU close_range/CLOEXEC backend must fail API equivalence; V3/C2d are historical GNU evidence only |

The Linux direct-original helper adds one test-only, Linux-only raw-method exception
with a local reason. It uses the existing finite probe and report validator, with no
new bound, timeout or privilege. Mutant kills count only as an actual compiled
assertion in a source/binary-hash-bound binary, followed by a same-bytes restored
PASS. Input negatives, compile failures, SETUP, NO_RESULT and cleanup Unknown never
count as kills. macOS mutants are re-run where the mutated region or its consumer
changed. Otherwise they cite their fixed-SHA result plus a diff proof that nothing
changed there. Tests stay parallel (no test-threads override).

### Impact
- Public: the six APIs, `prepare_*` and platform `Options`/module cfg.
- Unchanged: OwnedProcess and the existing call-site migrations (earlier caller
  analysis reused), and the Clippy raw-method ban.
- Test consumers: `controls`, `api_wiring`, `eof_observation`, `owner` and the
  injection/callback-order cases, as mapped above.
- Documents: requirements, issue comment, master `agent-execution.md`, README and
  the verification ledger.
- Linux behaviour returns to main semantics. That is not a new production contract.

### Authorization
- Owner-authorized, not re-asked: macOS First; platform separation that keeps the
  Linux baseline; deferral of Linux work; Mac F3 after P4.
- Existing approvals reused: v3–v6 for the macOS backend, the custody contracts and
  unsupported-target ENOSYS.
- Still needs a concrete decision: the final PR merge (owner); any future Linux
  hygiene, or resolving the GNU STOP (this Issue's deferred scope); any change to the V-F
  predicate once classification has started.

## macOS F1 mechanism and focused frequency controls

Status: designed, not run. Revised after design re-review round 2 findings
D03-R2, D08, D09; requires the finder's re-review before any harness exists.
The final macOS V-F (main 16 / P4 16) stays INCONCLUSIVE and unchanged. It
had 0/16 qualified attempts per variant. A read-only source join found 24 F1
positives in 10 main attempts and none in P4. Not observing F1 in P4 does not
show a zero rate.

### Purpose and owner decision
The P4 merge condition blocks merge on a higher macOS F1 frequency, and an
unknown never counts as no increase. On 2026-10-15 the owner decided to settle
this gate with two synthetic controls on final source:
(1) a deterministic mechanism control (MC), and (2) a focused, pre-registered
frequency control (FC). Both run outside the scope of the original 32
attempts.

F1 is `RuntimeOwner::open` refusing because
`flock(owner.lock, LOCK_EX|LOCK_NB)` returns WouldBlock
(`execution/owner.rs:225`, macOS errno 35). Every observed site is a test
restart in the same process:
- `runtime/tests.rs:520` (13 positives);
- two sites in `installation/tests/epoch_fence.rs` (8 and 3 positives).

Each restart drops the owner, waits for the Weak to expire and reopens. The
lock and DB descriptors are verified CLOEXEC (owner.rs:199-201, 227-231).

Hypothesis H (unproven): a child forked by a sibling thread keeps a duplicate
of the owner-lock open file description from fork until exec closes CLOEXEC
descriptors. A same-process reopen inside that window is refused with os35.

Std 1.91.1 (`sys/process/unix/unix.rs`) uses `posix_spawn` unless one of these
shapes forces fork+exec:
- uid/gid/groups/chroot is set;
- `env_saw_path() && !program_is_path()`;
- there are non-empty pre_exec closures;
- on Apple, cwd is set and the program path is relative;
- setsid is requested (non-Linux-gnu).

The orchestrator checked these fallback facts against the Rust 1.91.1 source.
`process_group` uses POSIX_SPAWN_SETPGROUP and does not force fork. Main has no
pre_exec (rrx forbids `unsafe`), so on main only those shapes fork. P4 adds a
closure to every spawn, so on macOS every P4 spawn forks and the hygiene
callback runs inside the fork-to-exec window. P4 can therefore widen exposure:
more spawn shapes fork, and the window is longer. P4 does not keep the owner
lock or any other CLOEXEC descriptor at fd >= 3 past exec; fd 0-2 are
inherited by design.

The controls do not prove:
- the natural F1 frequency on main or P4;
- which holder caused the original F1s;
- the root cause of every main F1;
- process-group or descendant death;
- Linux behaviour (full Linux P4 stays deferred).

### Production lock/fencing impact
Refusal happens in `lock_state_root` before `Store::open`. The only non-test
openers are `cli/service.rs:91` (Runtime service) and `cli/project.rs:57`
(offline `OwnerLock::exclusive`). Every other `RuntimeOwner::open` is in a
`cfg(test)` module or an integration test.

| Path | When the extra holder matters | Outcome |
|---|---|---|
| Same-process reopen | Only tests today; no production code reopens in-process | os35 refusal |
| Offline `OwnerLock` | Only after the daemon's owner is gone; while the daemon lives it holds the lock anyway | `owner busy`, `EXIT_ROUTING_REFUSED` |
| Daemon restart (new process) | The old process's last forked child has not reached exec yet | `OwnerBusy` from `open_owner` |

A refusal fails closed:
- the error is typed busy (WOULDBLOCK in the chain);
- `begin_execution_epoch` is not reached, so no epoch is issued;
- no Store is opened and no state is written;
- rrx adds no retry, and this design adds none.

Fencing and CAS see no new owner. P4 does not hold the lock past exec:
- hygiene only sets CLOEXEC on fd >= 3 and closes nothing;
- the lock is already CLOEXEC, so exec closes it;
- exec failure and callback failure both end in the child's `_exit`, which also
  closes it.

The extra exposure is bounded by each child's fork-to-exec time. If either
control shows that a contract must change, work stops and a concrete diff goes
to the owner. Examples are retention past exec, or a busy-retry or lock
contract change.

### Hard constraints
- Nothing changes in production code, permission, ownership, CAS, marker,
  fencing or lock.
- Forbidden: retries that hide a refusal, timeout-only extensions, and skipping,
  ignoring or serializing existing tests.
- Also forbidden: test-only permission constructors, an SQL success seed,
  Task.version as authority, and warning suppression.
- Measurement code lives only in private copies under the repository root's
  ignored `.rrx/`, in `.rrx/p4-f1-controls-20261015/{main,p4}`.
  - Each copy is extracted from a tracked `git archive` of main
    `b8344ab951fa24f545174917eee7be067284c625` or P4 `44405c93`.
  - The exact SHA and the archive sha256 are recorded at creation.
  - Each copy has its own target dir. Copies are never merged.
- Any `unsafe` pre_exec closure (the MC gate, the FC-POS delay) lives in a
  local, unpublished dev-only crate inside the P4 copy, added only as an rrx
  dev-dependency of that copy. rrx keeps `unsafe` forbidden.
- Copies build warning-free. Copy-local manifest and lockfile changes are
  recorded by hash.
- Every MC/FC test invocation runs as a child of the private measurement
  runner, in its own process group, with a fixed deadline. On overrun the
  runner kills the group, reaps the test process and records NO_RESULT.
  Descendant death beyond that direct reap is recorded as unproven.
- Every child is waited and reaped before any assertion or result. No thread
  is detached, and no result is returned while any spawner thread is unjoined
  or any child is unreaped.
- No signal is ever sent to a raw pid. A child is signalled only through its
  unwaited std `Child`.
- These are SETUP/NO_RESULT, never a causal pass or kill: wait errors, FD-slot
  reuse, missing observations, bound overruns and panics.
- The original 32 attempts and their unknowns stay unchanged.
- The old GNU fixture `.rrx/p4-f1-causal-fixture-20261014` (old sources,
  `Some(11)` fixed, gate before hygiene) is not reused or restarted.

### MC: mechanism control (P4 copy, macOS only)
MC is one integration-test function in the P4 copy, invoked with `--exact`.
No other test thread in the binary forks, so no other test's children inherit
the owner lock. Default parallelism is unchanged (no `--test-threads`). Each
case uses a fresh temp state. Every wait has a fixed bound; an overrun is
NO_RESULT.

Steps:
1. Classifier controls. C0+: hold `OwnerLock::exclusive(state)`, then
   `RuntimeOwner::open(state)`. The classifier must return RefusedOs35: a
   `rustix::io::Errno` in the chain with raw value 35. C0−: make `owner.lock`
   a symlink. `open` must refuse, and the classifier must return NotOs35.
2. Open a real `RuntimeOwner` (epoch E). Find the parent's lock descriptor by
   `F_GETPATH`. Exactly one fd must resolve to the canonical `owner.lock`;
   otherwise SETUP.
3. Create two CLOEXEC pipes: `entered` and `gate`. Build a std `Command` for
   `/bin/sh -c 'printf R; read -r line'` with stdin and stdout piped. Register
   the caller gate closure through the dev-only crate. In the child it writes
   exactly one byte on `entered`, then blocks in `read(gate, 1 byte)`. It
   writes no pid. Both calls are async-signal-safe. A short or failed write or
   read returns an error, so spawn fails (SETUP).
4. Call `rrx_spawn_hygiene::spawn_std(command)` on a dedicated thread. Std
   returns only after exec, so this thread blocks. pre_exec closures run in
   registration order: the caller gate runs before P4's hygiene callback.
   Hygiene only sets CLOEXEC, so retention through hygiene is unchanged.
5. The test thread reads the one byte from `entered`. Without it, the result
   is SETUP.
6. Drop the sole owner `Arc` synchronously and confirm the Weak has expired.
   `F_GETFD` on the recorded fd must give EBADF. A valid fd means FD-slot reuse
   (SETUP).
7. Take a logical snapshot through a read-only SQLite connection that does not
   use `Store::open`: tables, their rows in rowid order, and the epoch.
   Call `RuntimeOwner::open(same path)` once and record the outcome. Take the
   snapshot again.
8. Release the gate by writing one byte (dropping the parent's write end is not
   enough, because the child holds a copy). Join the spawner thread within a
   bound (gate-release join rule below) and take the Child. Read the child's
   `R` from stdout within a bound, which shows that exec happened.
9. While the child is still alive (blocked on stdin), call
   `RuntimeOwner::open` once and record the outcome and epoch.
10. Drop the new owner. Write one line `Q\n` to the child's stdin and then
    close it, so `read -r line` succeeds and the child exits 0. Wait/reap it
    and record the exit status.
11. Only then evaluate:
    - step 7 refused with os35, and the before/after snapshots and epoch are
      equal;
    - step 9 acquired, with epoch exactly E+1;
    - the child exited 0.

Gate-release join rule. After writing the gate byte, the test thread joins
the spawner thread with a bound. Nothing is signalled by pid:
- `spawn_std` returned Err (hygiene or exec failed after the gate): std's fork
  path read the reported errno and already waited the child, so its pid may
  be reused. The case is NO_RESULT.
- `spawn_std` returned the Child: the normal path continues, and that Child is
  waited.
- The thread has not returned within the bound: the test process returns no
  result and blocks. The outer runner (own process group, fixed deadline)
  kills the group, reaps the test-process leader and records NO_RESULT.
  Descendant death beyond that direct reap is unproven (see Hard constraints).

On other failures: release the gate and join the spawner thread under the same
rule. If it returned the Child, write `Q\n`, close stdin, then wait. If the
child is still alive after the bound, it is killed through its `Child` and
reaped, and the run is NO_RESULT.

Negative control N: identical, except no gate is registered and there is no
`entered` read. `spawn_std` returns after exec, the child reports `R`, the
owner is dropped and EBADF is confirmed. Reopen while the child is alive must
acquire with E+1. The child then gets `Q\n` and a closed stdin like step 10,
and must exit 0.

Mutants, each run before MC results count:

| Mutant | Required outcome |
|---|---|
| Gate read removed (only the blocking `read(gate)`; the `entered` write stays) | Observation and reaping complete; the step-7 refusal assertion fails (reopen acquires) |
| Classifier accepts any error, not only os35 | C0− fails |
| Same-bytes restoration | Full MC passes |

A kill counts only if a compiled assertion in a binary bound to source and
binary hashes fails, and the same-bytes restored binary then passes.

MC runs 10 gated cases and 10 N cases in one invocation:
- MECHANISM_SHOWN: all 20 are qualified and all agree with the expectations
  above;
- NOT_SHOWN: a qualified gated case acquires at step 7;
- otherwise NO_RESULT.

### FC: focused frequency control (main and P4 copies)
Each copy gets one measurement-only integration test target with identical
source, except for the spawn line and the P4-only FC-POS function:
- main: plain `std::process::Command::spawn`;
- P4: `rrx_spawn_hygiene::spawn_std`.

Each target is built once with `cargo test --no-run` and the same profile in
both copies (`--release` is not required). The binary sha256 is recorded.
Test functions `fc_s1`, `fc_s2` and (P4 only) `fc_pos` are selected with
`--exact`. One invocation runs one block of 1000 counted iterations for one
arm. Each block is a separate test-process invocation with a fresh temp state
and freshly started spawners. Default parallelism is unchanged.

Child shapes, run as separate arms:
- S1: `env_clear()` + `env("PATH", "/usr/bin:/bin")` + bare program `true`.
  It forks on both copies.
- S2: absolute `/usr/bin/true`, with no env change. It uses posix_spawn on
  main and fork on P4. macOS posix_spawn still copies the descriptor table
  in-kernel before closing CLOEXEC descriptors, so S2-main is a comparison arm,
  not a zero control.
- FC-POS (P4 copy, positive sensitivity arm): S1 through `spawn_std`, plus a
  dev-crate pre_exec closure that calls `nanosleep` for 1 ms
  (async-signal-safe) before hygiene. It lengthens only the fork-to-exec
  window. FC can return NOT_OBSERVED_HIGHER only if FC-POS is detected as
  higher than P4-S1 (rule below); otherwise the result is not a pass.

Fixed values: K = 20000 counted iterations per arm, in blocks of 1000, and
M = 8 spawner threads. The arms are main-S1, P4-S1, FC-POS, main-S2 and
P4-S2, giving 100 blocks in 20 rounds. The order is fixed before the start and
balanced ABBA across rounds:
- odd rounds (main first): main-S1, P4-S1, FC-POS, main-S2, P4-S2;
- even rounds (P4 first): P4-S1, FC-POS, main-S1, P4-S2, main-S2.

All runs use the same Mac, as idle as possible.

Block procedure:
1. Open the owner (labelled SETUP open, not counted).
2. Start M spawners. Each loops over:
   build the shape → spawn (stdio null) → `wait()` → require exit 0.
   It counts spawned, reaped, spawn errors and wait errors. It also keeps a
   completed-cycle counter, incremented only after spawn + reap + exit 0.
   Each spawner owns its in-flight Child in a guard whose Drop kills and waits
   it. A panic sets a stop flag.
3. For each of the 1000 counted iterations:
   - while the owner is still held, wait (bounded) until every spawner has
     completed at least one new cycle since the reference point: the end of
     the previous counted iteration, the block's SETUP open, or the last
     resume, whichever is latest;
   - then take the owner's Weak, drop the sole Arc and poll Weak expiry
     (bounded, 1 ms poll, no other wait);
   - call `RuntimeOwner::open` once, immediately and without blocking.

   No step waits for children to drain between the drop and the open, so the
   open sees whatever children are in flight.

   Outcomes:

   | Outcome | When | Effect |
   |---|---|---|
   | Acquired | Epoch is exactly previous+1 (otherwise SETUP) | Becomes the owner for the next iteration |
   | RefusedOs35 | The MC classifier returns RefusedOs35 | Counted, and the iteration ends |
   | SETUP/NO_RESULT | Any other error, a load-wait or expiry overrun, or a spawner error in the iteration | Iteration ends; a non-os35 open error is also a veto |

4. Recovery after a refusal or SETUP:
   - raise the pause flag;
   - wait (bounded) until every spawner has reaped its in-flight child and
     parked;
   - run one RECOVERY open, which is never counted (the counted attempt is not
     retried);
   - resume the spawners.

   If the RECOVERY open is refused, that refusal is not a counted F1. The
   block's remaining iterations are NO_RESULT, the anomaly is recorded, and
   it is a veto (see the rule). In P4 arms, those iterations also count
   against P4 in the decision rule.
5. At block end, stop and join every spawner thread. Any panic or join error,
   or spawned != reaped, makes the whole block NO_RESULT.

Records per block:
- copy SHA, archive, source and binary hashes;
- argv, round, position in the round, start/end time, macOS and toolchain
  version;
- `sysctl vm.loadavg` before and after;
- counts of Acquired, RefusedOs35 and SETUP/NO_RESULT;
- RECOVERY outcomes;
- spawned, reaped, spawn-error and wait-error counts;
- per counted iteration, each spawner's completed-cycle delta from the
  reference point to the drop.

### Pre-registered decision rule
This rule is fixed before any FC run and cannot be edited afterwards. The
section hash is recorded before block 1.

Inference unit: the block. Blocks are separate test-process invocations with
a fresh temp state and freshly started spawners, and they are the independent
units. Dependence among iterations inside a block is allowed and not
modelled.

Block rates, per shape and round r (r = 1..20, 1000 counted iterations per
arm block):
- P4 rate p_r = (RefusedOs35 + SETUP/NO_RESULT) / 1000 (conservative);
- main rate m_r = RefusedOs35 / 1000;
- d_r = p_r - m_r;
- FC-POS rate q_r = observed RefusedOs35 / 1000;
- e_r = q_r - (P4-S1 conservative rate p_r of the same round).

Missing iterations never help NOT_OBSERVED_HIGHER anywhere in the rule: they
raise P4's rate, lower main's and FC-POS's observed rates, and so make both
non-inferiority and sensitivity harder to show.

Bounds use Student t with 19 df over the 20 block values, one-sided 95%:
- UB(d) = mean(d_r) + t(0.95, 19) * sd(d_r) / sqrt(20);
- L_main = max(0, mean(m_r) - t(0.95, 19) * sd(m_r) / sqrt(20));
- delta = max(0.001, 0.25 * L_main): an absolute floor of 0.1 percentage
  point, otherwise 25% of main's lower block-mean rate;
- LB(e) = mean(e_r) - t(0.95, 19) * sd(e_r) / sqrt(20).

Assumption: blocks are independent invocations, and the block-mean statistic
is treated as approximately normal at 20 blocks. If this assumption is not met
(e.g., any block NO_RESULT, which is already a veto), the result is
INCONCLUSIVE, never NOT_OBSERVED_HIGHER.

Veto, in any arm: a RECOVERY refusal, a non-os35 open error, a block-level
NO_RESULT anomaly, or spawned != reaped. The event's position and the number
of remaining iterations do not matter.

Outcomes:
- STOP (P4 higher): for any shape, either
  - a pooled one-sided Fisher exact test (hypergeometric P(X >= P4
    refusals), fixed margins) on the 2×2 table copy (P4, main) × outcome
    (RefusedOs35, Acquired), over observed qualified iterations, gives
    P4 > main with p < 0.05; or
  - a per-block paired sign test gives one-sided p < 0.05 for P4 higher.
    Each pair is the main and P4 block of the same shape and round, compared
    by observed RefusedOs35; ties are dropped.

  There is no multiplicity correction; this deliberately leans toward STOP.
- NOT_OBSERVED_HIGHER: only if all of these hold:
  - for every shape, UB(d) <= delta;
  - sensitivity: LB(e) > 0;
  - SETUP/NO_RESULT <= 2% of iterations in every arm;
  - no veto.
- INCONCLUSIVE: anything else, including a failed sensitivity check.

Pooled counts treat iterations as independent Bernoulli trials, which this
design does not assume. Pooled Clopper-Pearson bounds and pooled Fisher are
recorded as descriptive only; neither gives the pass. Pooled Fisher stays a
STOP trigger: a misspecified pooled test can only make STOP more likely, which
is the safe direction. The ABBA order and the sign test guard against drift.
Per-block counts are reported.

delta is a pre-registered engineering margin for a synthetic load, not a
property of natural use. STOP and INCONCLUSIVE keep P4 unmergeable and are
reported to the owner. NOT_OBSERVED_HIGHER is a synthetic-load finding, not
proof of natural non-increase. The owner decides the gate with these results
together with the MC verdict. FC runs whatever the MC verdict is, and neither
control alone authorizes merge.

### Review and records
- This design is reviewed by the non-writing family (Sol, high). Zero confirmed
  mandatory findings are required before harness work.
- The harness is implemented by Sol (high). Its source is reviewed before runs.
- Results go into `doc/verification/issue-89-p4.md`, in a new section separate
  from the 32-attempt V-F.
- Private evidence (raw logs, hashes, archives) is kept under `.rrx/`.
- Measurement copies are removed only after their hashes and archives are
  verified, and are never merged.
