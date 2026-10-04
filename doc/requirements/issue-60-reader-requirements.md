# Issue60 selected Git reader lifetime — Requirements3 candidate

Risk: STRICT. Status: UNAPPROVED requirements candidate. Separate from the merged
inspection diagnostics slice. Combined Requirements2 was request_changes twice;
its source findings motivate this contract, not approval. Reader Requirements1
was also request_changes twice; verified corrections below remain pending
re-review. Reader Requirements2 also returned request_changes twice at6faf26f;
current corrections are not approved. Requirements, design and source gates for this component are still required before production edits.
Initial base: public main2c6ae9d6f54e68a3bdbead25817c46981d231db0.
Current candidate normally composes main efe977486693a065122d8fc177b0e83d29620bdc;
its independent usage-read error projection changes no native/common source.

## Problem and verified scope

The current `bounded_git_raw_inner` spawns stdout/stderr Tokio reader tasks. A
cleanup, reap or observation error returns without observing their JoinHandles;
dropping a JoinHandle detaches the task. The reader collection's first read/join
error also leaves the peer unobserved. Observation and output timeouts call abort
but return without observing its completion. These are verified source lifetime
gaps, not measured causes of any historical inspection/EOF/CI failure.

`ProcessGroup::reap` clears its flag after direct child wait, before reader
settlement. An early caller Drop can reach the group's synchronous cleanup through
Drop; the caller runtime does not itself provide a shutdown-independent owner.
`ProcessGroup::new` consumes the actual Child before its fallible identity checks;
its failure API cannot return that anchor. Constructor failure is a defensive
ownership requirement, not a demonstrated real invalid-PID occurrence.

Existing selected-group cleanup proves neither a complete enabled workload nor
escaped/helper/delegated-service cleanup. This component owns actual selected Git
Child/group/reader/job lifetime only; whole Issue60 durable runtime owner,
reservation/effect/default-deny helper inventory, recovery and F1 remain OPEN.
Changing this component must not certify those broader profiles as supported.

Actual current consumers: Generic validate_git preflight, Grok initial/refresh
ownership (initial/refresh AND checkpoint) and index_digest, Context
git_value_owned/git_value and scalar bounded_git.
Synchronous git.rs/runtime registry, Generic native constructor/reader abort,
Grok native constructor/stderr abort and future provider copies remain explicitly
unmigrated inventory; no consumer obtains full lifetime proof merely by its label.
Non-Git Generic/Grok Tokio spawn-wrapper Err can precede native owner registration;
reservation release while an actual process exists remains OPEN. The Git own-flag
predicate does not cover executor spawn; no actual occurrence is asserted.

## Proposed normative contract

The numbered clauses7–15 of [the reader/driver candidate](issue-60-git-reader-contract-draft.md)
are the full proposed contract reviewed together with this document. They remain
UNAPPROVED. In particular:

- Before Git spawn, reserve actual bounded capacity for supervisor, both readers
  and cleanup; exactly64 total job permits, four per operation, maximum16 admitted
  operations/32 readers. Dispatcher/monitor/setup work cannot hide additional
  unbounded jobs or per-waiter tasks. Admission waits only within the existing caller
  deadline, with no effects or new uncertainty on proven pre-effect refusal.
- Acquire the actual native Child before ALL fallible Tokio/runtime wrapping, stdio/
  signal registration and group binding. Tokio spawn Err can follow native spawn.
  Retain Child/group, both readers and cleanup work independently of the caller future/runtime.
  Cancellation is nonblocking. A bounded preavailable independent driver must
  deliver existing first cleanup even when the caller runtime shuts down before
  dispatch. Total driver loss/poison/panic stays retained Unknown; no fallback job,
  numeric PID rescue, automatic replay or restart recovery.
- Keep internal native uncertainty separate from the caller's publication flag.
  The caller flag becomes uncertain before effects. Mandatory clear points are a
  live in-budget complete-settlement return, including a fully settled error, and
  a genuine native OS-spawn failure with proven no surviving child in the actual
  verified CI toolchain/command profile. Err/no handle alone does not prove this. A wrapper/registration
  Err is not no-effect. Dropped/returned Unknown is irreversible for that call;
  late settlement never clears Context's existing sticky latch.
- Preserve primary error priority and original kinds: cleanup/reap, observe,
  stdout then stderr read/join collection, output_open, unsuccessful Git exit,
  success. A peer reader failure must not reorder the first established error.
  For post-spawn initialization error, cleanup/reap failure wins; otherwise keep
  the original initialization kind. Binding failure without established signal
  identity retains LaunchFailure/Unknown without an invented group action.
  Abort is requested cancellation; only observed returned/panicked/cancelled joins
  prove that reader future/endpoint ended, never successful read or native death.
- Use the single existing250ms output budget for collection/abort/both joins.
  Do not reset it or add another abort budget. Native inspector250ms and existing
  finite Git caller deadline remain unchanged; mandatory inspector cleanup/reap
  is not a claimed total operation-time bound. Unknown retains actual anchors,
  permits and conservative caller facts; settled read failures preserve errors
  without inventing uncertainty. No success or permit release before all actual
  selected group/Child/readers/cleanup jobs are observed settled.
- Selected-group death and direct-child exit remain weaker than whole workload
  settlement. This process-local component cannot release a durable effect hold,
  authorize native input/replay, mint source freshness, or advertise F1 readiness.
  Context and Grok equations remain unchanged. Explicit Generic live launch-error
  change is impact-gated: Lost on SessionLost OR retained own uncertainty, original
  error kind preserved. Grok checkpoint Err only retains pool resources; its absent
  receipt/durable sink and later-resume conflict enforcement remain OPEN.

## Shared impact and exclusions

The earlier draft proposed leaving common new/reap/Drop behavior unchanged.
Implementation must explicitly resolve actual Child retention across construction
failure and prevent caller-side synchronous cleanup without silently changing
shared native behavior. Design must choose a narrowly scoped Git owner/wrapper or
an independently impact-reviewed additive shared port. Any changed common
constructor/Drop/reap semantics require full source and all actual current native
and Generic consumer impact in the immutable review. No port is presumed supplied
by Issue6's currently approved empty/no-effect StageA; its genuine all-job StageB
custodian/containment/settlement is still pending. This candidate authorizes no
production common helper edit before these gates.

No schema/Store authority, native auth/hooks/default model/effort, environment,
ACP/permission, Git configuration/hook bypass, scheduler, kernel backend, global
signal, PID adoption, deadline/latch/Unknown relaxation or test serialization.
At least four selected operations must progress with healthy driver/free capacity;
global capacity couples Projects and cannot promise isolation/fairness under holds.
This is not actual native3/runtime mixed-workload MVP acceptance.

## Acceptance gates — pending

1. Two independent immutable requirements and design reviews approve this partial
   component before source; all findings verified against actual source. Include
   the full linked candidate and actual common/consumer source, not copied prose.
2. Real isolated Git/child consumers prove success and settled errors, first cleanup
   on caller runtime shutdown, caller Drop before/after spawn, constructor failure
   anchor retention before runtime/stdio/signal wrapping, real native-spawned std
   Child then forced initializer Err with retained-Unknown AND fully-settled-error
   controls, pending peer after primary read error, observed cancellation,
   cleanup/reap failure, genuine native nonexistent-executable spawn refusal with
   profile-specific no-survivor primary proof, clear flag/no Context latch/Generic
   Failed, healthy reserved-supervisor late observation and mandatory
   permit release with unchanged flag/latch, and lost-supervisor no-observer permanent
   retention. Prove owning-runtime resource survival after caller runtime shutdown;
   no raw-PID or outside-Child reap. The std Child anchor cannot use the current
   Tokio-Child ProcessGroup; independently gated shared primitives OR Git-local
   equivalent must preserve existing signal/inspection/TestPlan semantics, with
   real-route differential controls/parity mutants on both OS. Preserve error
   priority and
   demonstrate flags/permits/anchors at the actual consuming boundary.
3. Actual cap admission/wait/no-spawn and four-progress controls, destructive
   private pool isolation, driver panic/poison/shutdown ownership and no native work
   under Store/pool mutex or async poller. Every supervisor/reader/cleanup job capacity
   is counted before spawn; no hidden fallback or hold-join scheduling. Bounded
   driver frame must observe the supervisor future completed/destroyed before its
   terminal slot release, with no extra observer or native/blocking work after it.
4. Compiled actual consumer mutants reach the intended missing guard; masked or
   unit-only effects earn corresponding limited attribution. Exact restored
   controls pass. No caller-only copied helper can substitute for real routing.
   Route current7+1 Unknown fixtures and new destructive Generic/Grok consumers through cfg(test)
   private pools; prove production retained count unchanged and non-poller private
   teardown/retention. Actual Generic returned output_open/peer/Timeout/initialization
   Unknown must persist Lost/reserve, with original non-SessionLost API kind; settled
   errors must remain Failed. Checkpoint Err plus pool retention carries no false
   receipt or durable-state credit.
5. Two independent immutable source reviews and fix/rereview; appropriate default
   debug/release tests, fmt/all-target Clippy, both builds and current Linux/macOS
   CI with actual checkout/parents/tree proof. Preserve every prior failure and
   cause AND regression UNKNOWN; passing new source is not historical causality.
6. Partial disposition keeps whole60/reader availability where unproved, F1,
   durable restart/recovery and actual native/runtime concurrency16 OPEN. No
   diagnostics-only approval or newly joined reader constitutes full acceptance.
