# Issue60 diagnostic component design

Risk: STRICT. Design2 approved independently twice at201bd89; six Low source-phase
clarifications below. Source implemented; verified Source1 corrections/independent
re-review and final CI remain pending.
Requirements4 has two independent diagnostic-only approvals at26de8da. The later
source-aside correction does not change its required real-inspector consumer criterion.
Full Issue60 workload ownership/effect/delegation/durable settlement, native16 and the
separate reader/driver contract remain unapproved/open. Baseline main054aefd; no
new process backend, containment or availability claim.

## Exact unchanged runtime boundary

Only macOS `adapter::inspection` gains diagnostic facts. Production still runs trusted
`/bin/ps -g <owned unreaped leader> -o pid=,pgid=,stat=` with env_clear, null stdin,
piped stdout/stderr and no COMMAND_MODE override. Keep std Child lifetime, signal/wait
choices,250ms post-spawn observation,1MiB per-stream bounds,8192-byte nonblocking
chunks,5ms idle polling, original guard order and error kind/priority. Successful
KILL/ESRCH and Linux behavior stay unchanged. Every baseline reader, blocking Drop,
reap/orphan, retry and uncertain-worker gap remains open.

`ProcessGroup::new/reap/Drop`, `bounded_git_raw_inner`, cleanup scheduling, reader
abort/join behavior, caller flags, admission/runtime driver and Store are unchanged
in production. Error text is the only new output from the inspector. No facts become
permission, death, native completion, freshness, replay or reservation authority.

## Private typed fact frame

Add private finite enums and a fixed-size fact snapshot, containing no command or
process values. A small internal collector records actual facts at source. All collection and final
collection lives only in shared inspect_command/observe_command/complete/Inspector/
Stream/validate code, including private common helpers. FINAL error attachment is
only observe_command's Err arm and inspect_command's error returns. Complete, Stream
and validate keep their original inner error values, Display and IO kinds. Production-only inspect,
macos_group_is_dead, process_group_inspection, inspect_process_group and resolver
closure arms remain byte-identical pass-throughs. No production-only fact decoration
or stripping is allowed: the shared attachment site is the causal mutation target. An error
owns a snapshot; no caller may supply it as authority. Public `io::Result<bool>` and
AdapterError shape remain compatible. Source error categories are typed, never
reconstructed from strings. Facts are not a new persisted record or audit schema.

| Field family | Meaning and bounds |
| --- | --- |
| site | exact static input/spawn/endpoint/setup/loop/drain/read/allocation/post-read/budget/status/post-status/exit/stderr/validation/post-validation site |
| stream | stdout, stderr, none or unavailable |
| refusal | static guard/framing category, deadline, allocation or finite IO kind |
| each stream | available counters for invoked reads, successful bytes read, WouldBlock and Interrupted; EOF observation state |
| status | actual try_wait calls, pending/Interrupted/error result counts and first observed exit category |
| validation | not_reached, valid_live, valid_dead or exact typed framing refusal; separate from masking post-validation deadline |
| elapsed | observation duration at first failure, with spawn and mandatory cleanup durations separately unavailable/observed |
| cleanup | not_reached, reaped_by_status_observation, kill_requested_then_wait_reaped, wait_failed_uncertain, relinquished_without_cleanup or unavailable |
| kill | not_requested, returned_ok, returned_error with finite IO kind or unavailable |

The general vocabulary allows unavailable; currently every production attachment has
an explicit selected site/stream (including known none), a known no-child cleanup/
kill state at input/spawn, or a known measured/no-op cleanup state at later returns.
Thus stream/cleanup/kill unavailable are unreachable at every current attached error;
no enum variant is invented for them. Optional stream/status/timing measurements still
render unavailable. The unit-only unattached default formatter case is not a runtime
error sample or a future authority promise.

All counters saturate u64. A stream becomes measurable only after its endpoint is
available; no endpoint is unavailable rather than zero reads/false EOF. After setup,
zero reads means actually zero calls. EOF vocabulary distinguishes unavailable,
not_observed, pending after a non-EOF read and observed EOF. Count successful bytes
immediately from the read result even if reserve/store later fails; it is bytes read,
not buffer capacity. Count every read invocation including Interrupted/WouldBlock.
A successful0-byte read observes EOF. Count only invoked try_wait, not a cached exit.
Status measurements are unavailable until the first actual try_wait; a loop-entry
timeout does not manufacture zero status polls. Capture a non-Interrupted try_wait
error's underlying finite IO kind before baseline discards it; its returned IO kind
still stays Other. Exit categories are unavailable, success, nonzero_exit or signaled_or_other; no numeric
PID/exit code/status text is necessary. Finite IO kinds map unmatched variants to other.

Do not add argv/path/env/auth/config, PIDs/PGIDs, raw rows/stderr/child bytes, foreign
identities or arbitrary native bodies to facts. Counter/elapsed fields cannot explain
scheduler delay, an inherited FD or group ancestry. Missing data is unavailable.

## Collection and first-error attachment

Pass an invocation-local collector through the existing Inspector and Stream runner.
No shared/global mutable test clock or fact sink is used. Each check records its
candidate static site before performing the unchanged check; freeze returned failure
site/refusal/elapsed at the selected error return, not merely the latest successful
check. Deferred validation below is a deliberate exception to immediate selection. Stream setup errors retain endpoint/other-stream observations
already reached; they do not invent an unopened peer's measurements. Nonblocking
flags and File ownership are unchanged.

The collector captures the first failure site/elapsed/counters/EOF/pre-cleanup status
before owned endpoints drop. After the existing mandatory cleanup it records separate
kill/wait/cleanup observations; it never overwrites the first status sample with an
exit seen only during cleanup. In particular:

- try_wait Some(exit) has already reaped the actual direct child. Later cleanup is a
  no-op and the fact is reaped_by_status_observation.
- a non-Interrupted try_wait error clears unreaped with no exit in baseline. Preserve
  that relinquishment and no additional kill/wait. A subsequent cleanup Ok/no-op is
  relinquished_without_cleanup, never verified reap.
- an unreaped Child receives the existing kill and mandatory wait. Capture their
  separate results, including kill error followed by successful wait. Successful wait
  is kill_requested_then_wait_reaped; failed wait is wait_failed_uncertain. Interrupted
  wait retries are unchanged. Total cleanup time has no250ms promise.

The first error retains its original IO kind when cleanup also fails. A typed wrapper
formats its static original category plus bounded facts AFTER mandatory cleanup;
formatting cannot skip cleanup or turn an error into success. Existing fixed human
prefixes used by tests remain available. Raw syscall error text need not be copied:
retain original returned IO kind, source finite IO kind and a static site message.
This explicitly replaces raw syscall Display at decorated inspect errors; wrapped
io::Error.raw_os_error is None, not a promised preserved errno. Current production
callers stringify/kind-check only. Existing complete-directory-read unit expects
raw ISDIR: keep its unwrapped inner read error. The real live-member resolver control
expects raw PERM: Ok(false) inspection still returns original signal PERM unchanged.
Also keep Stream overflow exact Display at480, complete TimedOut at535 and wrapped
inspect substring assertions420/450/758. Any collector signature/literal updates at
470–473,511–525,548–562,582–588 are mechanical; existing assertions stay unchanged.
No raw-metadata consumer is silently changed. Known framing/
timeout human prefixes remain. Do not expose child bytes as error bodies.
Display writes only fixed strs, enums and integers, propagating only writer errors;
it never originates fmt::Error, unwraps or invents an optional formatting-failure
branch. String formatting therefore cannot turn this Display into a cleanup-worker
panic. The maximum-size rendering test exercises that EXACT Display with every enum
and maximum counters; no child-derived allocation or raw-body formatter is called.

A completed Frame carries its observations into the existing exit/stderr/frame guards.
The validator produces private typed framing refusals at their source while preserving
its externally observed IO kind/boolean and fixed message. No substring classifier is
introduced. Preserve full-frame validation before valid_live/valid_dead. The result of
validate ONLY writes its separate typed validation field, without freezing failure.
Then run the unchanged post-validation deadline. Deadline Err selects returned
site=deadline_after_validation, refusal=deadline and elapsed there, retaining masked
valid_live/valid_dead/framing separately. Deadline Ok plus validation Err selects
site=frame_validation, its typed refusal and elapsed at that selected return. Thus
an unmasked framing failure never carries a successful deadline-check site, and a
masked framing failure never carries framing as its returned timeout site. Success does not render/persist facts or change boolean outcomes.

## Error transport and discarded paths

The resolver passes the inspector io::Error through unchanged. Existing cleanup_group
formats the first error; its later ProcessGroup Drop retry cannot mutate an owned first
snapshot. No new channel/log/stderr/global fact store captures discarded Drop results.
Those retry/Drop facts are unavailable to callers. Nothing modifies common Drop.

Existing sinks may include new bounded text, with unchanged priority/authority:

- Context `bounded context Git: {e:?}` preserves first error; later sticky-latch refusal
  has no fresh inspector sample.
- Generic adapter.launch_failure.reason and runtime SessionStatus.failure, from Git
  preflight/post-spawn cleanup and terminal supervision.
- Grok turn_observed.diagnostic, reconciliation_error and runtime SessionStatus.failure,
  including the existing unknown-dispatch rewrite, native group cleanup and ownership/
  index reconciliation Git paths.

session.saved continues projecting only state, agent, provider, role, native_ref and
recovery.dispatch_intent. Grok cleanup_receipt retains its exact schema and unclassified
cleanup failure; message facts cannot classify/alter its clean operand. Master rationale must explicitly keep unclassified as POLICY until a
separately reviewed typed consumer exists: AdapterError erases structure, safe text
may remain, and parsing it grants no authority. Native hooks,
auth/defaults, permissions, environmental policy, Store/schema and safety gates are
untouched. Synchronous git.rs gets no diagnostic/consumer credit in this component;
any later route requires inventory/composition review. Future copied5/6/19 consumers
also need composition review; no universal acceptance.

## Private seams and causal verification

Use per-invocation cfg(test) hooks for deterministic failure/clock sites. They can fail
one exact endpoint/fcntl/read/allocation/status/deadline site and pass real execution
elsewhere. Label these injections; do not claim the OS naturally reached them. Injection
of a status error is UNIT-only and substitutes an error only AFTER an actual owned
Child try_wait has returned Some(exit), so std has already reaped that direct child.
Retain its actual Child handle in the fixture through synthetic relinquishment; never
rescue by numeric observed-callback PID, waitid/adoption or later PID signals. The
source error branch then clears unreaped/has no reported exit just as baseline.
Hold the unit logical deadline open with the private per-invocation clock seam;
this is deterministic unit injection, not a wider native budget. Open/non-target
Expire checks have a distinct test-only10s real watchdog and keep5ms idle polling
past the real250ms timestamp, avoiding a busy-spin. This bounds the fixture loop,
not existing unbounded spawn/mandatory cleanup or native workloads. Production Real
clock and idle behavior are unchanged. Shared post-error
cleanup/fact recording is one helper used by observe_command and the fixture.
Any cleanup-call omission mutant in this synthetic state uses recorded fixture-only
kill/wait operations, not a new real syscall on a reaped PID. Assert no calls after
relinquishment and independently verify the pre-injection actual reap even on assertion
failure. Relinquishment mutation earns UNIT GUARD credit only, not authority-consumer
closure. No stranded live child/zombie or real status-error acceptance is claimed.
Each adjacent deadline check is reached deterministically rather than waiting
for scheduling between statements. Production uses its unchanged monotonic clock.

The existing TestPlan800–914 already reaches real `inspect_with_prefix→inspect_command`
from ProcessGroup kill_group. Its KillAndUnknown signal first KILLs the actual owned
group and then injects PERM; the controlled `/bin/sh` feeds a shim frame through the
real selected argv/env/stdio/guard/cleanup path. These are labelled injected signal and
shim execution, not actual OS permission failure or installed ps acceptance. No new
executable seam is required just to execute this route. Preserve that complete source
in review packets; the prior packet omission/false fabricated-result inference is
withdrawn. Any future hand-built fact/ioError bypass earns rendering pass-through only.

Actual Context tests use a private latch to isolate destructive first-Unknown controls
from default concurrent tests. The supplied plan must execute the real inspector; its
error snapshot must survive resolver→cleanup_group→bounded Git→Context rendering.
Inspect Context anyhow chain ({:#} or chain()), not top-level Display which hides
the bounded-context-Git cause. Discriminate original returned inspector IO kind
(TimedOut versus guard InvalidData/Other) independently of the reported site, then
check matching site family; a mislabelled site cannot select its own acceptance path.
Assert original kind, a present safe fact block, retained uncertainty and derived
refusal without a fresh sample. Exact deterministic guard/refusal/stream, EOF/exit/
validation/cleanup/kill categories and fixed shim stderr-byte count apply ONLY when
that guard actually wins. A deadline-masked run accepts only a deadline-family site
and its truthful retained fields, preserving failure without exact-site/throughput
credit. Context Timeout accepts that family; exact adjacent sites are unit clock-seam
evidence only. Never assert exact elapsed/read/WouldBlock/try_wait counts or dynamic
leader-digit-dependent stdout length at Context. Counter invariants require invoked
reads >= WouldBlock+Interrupted+observed-EOF reads and positive try_wait calls only
when status was reached; deterministic counter-omission kills earn unit/seam credit.
Unmasked framing shims must carry frame_validation, masked valid_live/framing unit
cases must carry deadline_after_validation with separate validation result. A compiled mutation removing construction/attachment at the
inspector-to-resolver boundary must fail this actual consumer. General formatting unit
mutants do not earn this reach credit. A separate pre-effect entry-hop fixture uses
nonexistent executable through process_group_inspection→inspect_process_group→inspect
and asserts spawn-site facts survive; injected resolver PERM opens that route without
any child/signal. Label entry-hop-only credit, not Context/OS-permission acceptance.
Production-only wrappers stay unchanged; an untested decoration diff is prohibited. Reach Generic/Grok existing sinks where actual
fixture support permits; disclose remaining sink coverage precisely. Generic terminal
failure and post-spawn launch-failure audit use the existing native plan and held-start
barrier. Grok completed-success synthetic ACP exercises the cleanup diagnostic and
runtime failure with unchanged unclassified/clean=false receipt. Existing Grok
predispatch/unowned-read errors correctly mask later cleanup text by priority, so they
do not prove fact transport. Grok reconciliation_error host-Git transport lacks a
current stage-specific plan seam and has no direct fact-transport assertion in this scope; unchanged forwarding is source-only evidence;
no production priority or stage port is changed to make the fixture pass.

Real controlled subprocess/owned-pipe tests cover pending stdout/stderr, actual EOF and
observed child exit, exit/stderr/framing failures, valid_live/dead and cleanup. Keep
owned inspector/fixture cleanup and joins even on assertion failure; no unrelated PID
signals. Parser/site/counter injections and maximum formatter tests are unit evidence.
Use both omission and authority-refusal mutants: removing relinquishment distinction
must fail an intended consumer; facts may not grant cleanup to Unknown. Positive existing
native/fixture results must remain identical. Keep all old timeout/cap/ownership logs;
new diagnostics are not an explanation for them.

## Implementation gates and explicit limits

Committed requirements precede narrowed approvals; the current committed design revision precedes two
independent public immutable design reviews. No source before both design approvals.
Source/impact commit precedes affected tests, compiled causal mutants and independent
source reviews. Run default debug/release, fmt/all-targetClippy, both builds and final
public Linux/macOS CI; preserve trigger vs actual checkout parents/tree/source provenance.
No blind old-head rerun, serialized default tests, larger deadlines or Unknown relaxation.
Final red blocks merge. Fresh changed diagnostic source may be measured once by CI;
passing it does not repair an unexplained earlier inspector timeout. Implementation
updates README/master pending status only after actual source gates; the master Grok
unclassified rationale above must remain consistent with the unchanged receipt.

This component supplies finite failure-stage evidence only. It does not improve the
native backend's known availability, fix detached Git readers/unreaped anchors/async
blocking Drop, or establish complete enabled-workload settlement. The unapproved reader/
driver draft, durable workload reservations, recovery14, full60 and native16 stay open.
