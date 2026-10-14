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
all-targets Clippy and 20 hygiene tests passed at `e8359628`; independent
implementation review, final macOS qualification and merge are UNPROVEN.
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
