# Issue60 diagnostic component design

Risk: STRICT. Design1 candidate, not implemented or source-approved.
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
process values. A small internal collector records actual facts at source. An error
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

All counters saturate u64. A stream becomes measurable only after its endpoint is
available; no endpoint is unavailable rather than zero reads/false EOF. After setup,
zero reads means actually zero calls. EOF vocabulary distinguishes unavailable,
not_observed, pending after a non-EOF read and observed EOF. Count successful bytes
immediately from the read result even if reserve/store later fails; it is bytes read,
not buffer capacity. Count every read invocation including Interrupted/WouldBlock.
A successful0-byte read observes EOF. Count only invoked try_wait, not a cached exit.
Exit categories are unavailable, success, nonzero_exit or signaled_or_other; no numeric
PID/exit code/status text is necessary. Finite IO kinds map unmatched variants to other.

Do not add argv/path/env/auth/config, PIDs/PGIDs, raw rows/stderr/child bytes, foreign
identities or arbitrary native bodies to facts. Counter/elapsed fields cannot explain
scheduler delay, an inherited FD or group ancestry. Missing data is unavailable.

## Collection and first-error attachment

Pass an invocation-local collector through the existing Inspector and Stream runner.
No shared/global mutable test clock or fact sink is used. Each check records its
static site before performing the unchanged check, and sets the refusal only when
that check fails. Stream setup errors retain endpoint/other-stream observations
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
retain IO kind and a static site message. Do not expose child bytes as error bodies.
A fixed maximum-size rendering test covers maximum counters and every enum; formatter
uses finite fields without child-derived allocation. If optional fact formatting is
unavailable, keep the original error and cleanup outcome, never synthesize success.

A completed Frame carries its observations into the existing exit/stderr/frame guards.
The validator produces private typed framing refusals at their source while preserving
its externally observed IO kind/boolean and fixed message. No substring classifier is
introduced. Preserve full-frame validation before valid_live/valid_dead. The result of
validate is recorded independently, then the unchanged post-validation deadline is
checked: a timeout still wins even if validation had an error; retain that masked typed
result separately. Success does not render/persist facts or change boolean outcomes.

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
cleanup failure; message facts cannot classify/alter its clean operand. Native hooks,
auth/defaults, permissions, environmental policy, Store/schema and safety gates are
untouched. Future copied5/6/19 consumers need composition review; no universal acceptance.

## Private seams and causal verification

Use per-invocation cfg(test) hooks for deterministic failure/clock sites. They can fail
one exact endpoint/fcntl/read/allocation/status/deadline site and pass real execution
elsewhere. Label these injections; do not claim the OS naturally reached them. Injection
of a status error follows the actual relinquishment branch, including no cached-PID
kill/reap. Each adjacent deadline check is reached deterministically rather than waiting
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
Assert original kind, exact reached facts, retained uncertainty and derived refusal
without a new sample. A compiled mutation removing construction/attachment at the
inspector-to-resolver boundary must fail this actual consumer. General formatting unit
mutants do not earn this reach credit. Reach Generic/Grok existing sinks where actual
fixture support permits; disclose remaining sink coverage precisely.

Real controlled subprocess/owned-pipe tests cover pending stdout/stderr, actual EOF and
observed child exit, exit/stderr/framing failures, valid_live/dead and cleanup. Keep
owned inspector/fixture cleanup and joins even on assertion failure; no unrelated PID
signals. Parser/site/counter injections and maximum formatter tests are unit evidence.
Use both omission and authority-refusal mutants: removing relinquishment distinction
must fail an intended consumer; facts may not grant cleanup to Unknown. Positive existing
native/fixture results must remain identical. Keep all old timeout/cap/ownership logs;
new diagnostics are not an explanation for them.

## Implementation gates and explicit limits

Committed requirements precede narrowed approvals; committed Design1 precedes two
independent public immutable design reviews. No source before both design approvals.
Source/impact commit precedes affected tests, compiled causal mutants and independent
source reviews. Run default debug/release, fmt/all-targetClippy, both builds and final
public Linux/macOS CI; preserve trigger vs actual checkout parents/tree/source provenance.
No blind old-head rerun, serialized default tests, larger deadlines or Unknown relaxation.
Final red blocks merge. Fresh changed diagnostic source may be measured once by CI;
passing it does not repair an unexplained earlier inspector timeout.

This component supplies finite failure-stage evidence only. It does not improve the
native backend's known availability, fix detached Git readers/unreaped anchors/async
blocking Drop, or establish complete enabled-workload settlement. The unapproved reader/
driver draft, durable workload reservations, recovery14, full60 and native16 stay open.
