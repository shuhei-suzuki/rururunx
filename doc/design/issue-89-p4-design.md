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

## Verification state

Local macOS compilation, fmt, all-target Clippy, 11 new-crate controls, 91 related
integration tests and tracker regression pass. V1/V2/V4/V5/V6/V7/V8/V9 and OW1
gate/authority/classification source mutants are detected and restored controls
pass. Linux C2d/V3 remain unexecuted locally. Initial full execution regression
has 186 pass, two Docker failures and one existing ignore; focused Docker replay
is 11 pass on both P4 and fixed main. The first failures remain recorded and their
frequency/cause is not certified. See [verification ledger](../verification/issue-89-p4.md).

M06, V-F and independent implementation/security reviews remain parent-owned
merge gates. Measurement uses a private instrumentation overlay so measurement-only
259a467 is not merged. No all-OS, process-death or full issue-completion claim is made.
