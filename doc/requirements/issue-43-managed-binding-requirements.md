# Issue 43: managed native Session binding requirements

Risk: STRICT. Status: requirements approved by two independent round2 reviews
of `7e47421a24225583795245b307e574fb3156dd7e`; see the
[review evidence](../verification/issue-43-managed-binding-requirements-round2.json).
No binder, allocation producer, migration or native qualification is delivered
by this file.
Source baseline: `5b6e8a8152926b643c6926b87e853de12b9319fc`. The reviewed legacy
contract is [Requirements9](https://github.com/shuhei-suzuki/rururunx/blob/b191b466d5dea303585cfaf6968c6fb178be79cd/doc/requirements/issue-43-requirements.md)
and [Design10](https://github.com/shuhei-suzuki/rururunx/blob/b191b466d5dea303585cfaf6968c6fb178be79cd/doc/design/issue-43-binding-mechanics.md).
This supplement connects that contract to the
[managed execution profile](agent-execution-requirements.md); it does not claim
that its ExecutionUnit or NativeInvocation already implements private #19 admission.

## Verified problem and dependencies

At the source baseline, the normal managed Workflow return sets session_id and
execution, refreshes Project/Goal/Task snapshots, then uses the ordinary Workflow
transaction (`workflow.rs::prepare_managed`). That transaction unconditionally
calls put_task_tx. The unmanaged return also uses the ordinary transaction.
Neither is the required factual record-only binding. A later snapshot refresh
can also erase the interval in which an owner changed during start.

The actual native registration transaction already writes Session, session_units
and NativeInvocation without writing Task. It checks exact input/context bytes.
It does not capture the original marker P/G/T/Workflow/full-lock frame, allocate
the #19 private phase owner/input pair, or prove the sole binding writer. Public
ExecutionAuthority/ManagedSessionRef, serialized rows, source hashes and a
returned Session ID are not that missing credential. Current managed authority
validation uses Unit/generation/epoch and governing content; no observed old
Task-version native failure is asserted for this new profile.

Runtime9 Driver readiness and Review member execution depend on actual binding
composition. Accepted Goal content, read-only status, cancellation fencing and
other independently qualified consumers may progress without advertising native
Driver readiness. Schema8 verifier and schema9 Runtime migrations must compose;
the binding implementation cannot independently reuse a reserved schema number.

## Required behavior

MB1. A successful first binding is one factual SQLite Immediate transaction.
It changes only the active native attempt's absent session_id and its previously
absent managed execution reference to the exact private allocated Session/Unit,
the Workflow record version/updated_at, and one reserved bounded factual audit.
The managed reference is a derived redundant projection; it grants no authority.
Task, Project, Goal, Session, Context and every scoped WorktreeLock body/version
remain byte-for-byte unchanged. Binding never claims work success, native death,
cleanup success, a Review verdict or phase completion. Ordinary phase/context/
state transitions keep their existing separately authorized Task writes.

MB2. Before the native start interval, the actual dispatch-marker transaction
captures an immutable private frame containing exact P/G/T identities/versions,
Workflow identity/version/complete body hash, scope, active index/generation,
Runtime state-root instance/epoch, Unit identity/execution generation,
phase/actor/provider/role, exact input/context/source/revision pins and the complete
scoped WorktreeLock identity/version set. It captures the resulting post-marker
Task/Workflow versions in that same transaction. Later current reads, a Driver
snapshot, a retry or a native returned identity cannot replace this frame. First
binding requires the original Workflow marker version/body, with no intervening
diagnostic or bookkeeping link. Current activity and every exact frame predicate
must still hold; unrelated parent version changes are not exempted.

MB3. Every fresh native Workflow attempt requires the genuine private allocated
phase Session owner and prepared-input preparation/admission/consumption pair
specified by #19. Its actual producer must be connected to the real registered
adapter's owned invocation and Store transaction. Marker, allocation and input
intent identities correlate exactly; later permission/transport/terminal journals
cannot overwrite or manufacture the consumed input. No public IDs/JSON, optional
caller flag, SQL fixture seed, synthesized hash, Unit DTO, terminal receipt or
capability advertisement can supply this credential. A genuine private producer
is mandatory on both normal return and closed-success late binding. Normal factual
binding permits the genuine allocated/validated/prepared pair in its defined
pre-input/pre-ACK state: it does not require or manufacture delivery, consumption
or acknowledgment. That same NotDispatched preparation cannot authorize late
successful binding, which additionally requires the actual correlated consumed
input and current owned successful terminal. Standalone
non-Workflow native fixtures do not establish this acceptance criterion.

MB4. The normal returned identity is checked against the selected registered
adapter and latest durable Session: exact ID/scope/actor/provider/role/worktree,
plus returned native_ref when present. The worktree is the genuinely prepared
Unit namespace: executor Task projection or a separate immutable-input reviewer
snapshot, never a live executor worktree substituted for review. This worktree
adaptation follows result protection; all other private owner/identity checks
remain mandatory. Starting may legitimately advance PID/native UUID/lifecycle
before binding; those observations neither grant authority nor invalidate an
otherwise identical return. Lost, absent, foreign or ambiguous Sessions reject.
The latest scoped provider/native UUID must have no distinct conflicting Session,
including Lost/history, and malformed/non-indexable scoped identities cannot
prove absence. Current Session is read under the transaction and never rewritten.

MB5. One crate-private binder is the sole existing-attempt Session-ID/managed
execution writer. Every ordinary Workflow access mode, generic record writer,
raw supported writer and recovery writer rejects that delta. New attempts start
unbound; repeated same-operation delivery can only return a proved AlreadyBound
fact with no second write/audit. A complete before/after projection check rejects
extra history, detail, context, marker, completion, actor or authority changes.
No broadly reusable record-only write option or caller-selectable access mode is
introduced. Reserved private audit names cannot be forged by generic audit APIs.

MB6. Missing composition refuses before fresh native phase Context/reservation,
marker, preparation helpers, process spawn or input effects. Genuine composition
availability is implementation-owned and absent by default, not inferred from
CLI presence. Config/FakeAgent cannot enable it. The final profile preserves
official authentication, settings, required hooks, permissions, policy, protected
base and source checks. No ownership sampling extension, outer sandbox, root,
VM, container or credential copying is introduced. rururunx is not a security
sandbox; cleanup remains best effort and separate from work outcome.

MB7. A post-marker binding refusal preserves the exact reservation and actual
owned invocation. It cannot fail the Task, release/retry/transfer, recapture
authority, send another input, stop another Task, or fabricate completion.
Storage contention/uncertain commit produces a typed deferred factual binding
handled by the actual retained invocation and authoritative Driver, independently
of a dropped Engine future. Unknown start delivery stays held. No-current-dispatch
and known failure use their separately proved non-success closure, not invented
binding. The successful late path uses the same binder and only a sealed genuine
current allocated/input/success-settlement proof, never a caller receipt ID.

For this managed profile, success-settlement means the actual private allocated
phase owner and prepared/admitted/actually consumed input, the exact correlated
current owned Native terminal with known successful work under the original
marker/instance, and recorded logical native-terminal/input-settlement evidence.
It is neither a physical-death/full-cleanup proof nor an accepted Workflow success.
Runtime-only result finalization may remain open for genuine later capture; it
grants no further native input/effects. This explicitly supersedes the legacy
late-binding physical full-settlement and unknown-cleanup refusal for this profile
only. Cleanup Unknown/Leftovers alone neither creates authority nor blocks an
otherwise eligible factual late binding or independent progress. Unknown work,
HistoricalDraft or ambiguous/missing input/current proof, stale/restored authority
and irreversible external-operation uncertainty keep their holds. Genuine result
capture/publication and success closure remain separately required; the terminal
receipt does not certify them.

MB8. Correctness does not depend on a notification edge: reconciliation readiness
is durable before/with settlement, rechecked on Driver registration/wake/return
and after owned start ends (normal, error, timeout, abort/drop or cancel). Finite
fair timer fallback covers lost/full/closed notifications without busy polling.
Passive status/poll observers do not bind or acquire ownership. Restart uses the
actual reviewed fencing/restore protocol; a durable row cannot reconstruct a
live owner. Unsupported/unknown recovery stays held and visible. Duplicate and
delayed normal/late delivery produce one exact binding/audit.

MB9. Private current-successor ledger and separately authorized bound-live
diagnostic/gate/closure ports retain original source/actor/lock/native currency.
Binding never authorizes their wider deltas. Preserve Design10's reserved-key
protection, immutable chain, finite per-class allowances and mandatory closure
capacity. Optional diagnostics cannot consume mandatory closure reserve or make
first binding tolerate Workflow drift. Full body planning/hashing occurs outside
the Store mutex; publication has no await/filesystem/native operation and enforces
explicit finite owner/body/lock/identity/ledger cost bounds. Bounds are admission
conditions, not truncation or a weakened empty-set proof.

SourceRecovery7's exact Workflow pins and Runtime9's future Driver pins must
recognize only a genuine private permitted successor anchored at their existing
immutable authority. Binding cannot update those authority rows, call the generic
after_write refresh, or silently recapture current pins to avoid a conflict.
The design must inventory every native live wait/status arm, including quota
Task projections and pre-Session NativeStart::Waiting: they are not automatically
the legacy detail-only diagnostic delta. Any legitimate state-changing port needs
its own exact typed authority/write contract; factual binding grants none.

## Acceptance and evidence

| ID | Required actual consumer evidence |
| --- | --- |
| MB-AC1 | Engine + actual private #19 producer + registered native invocation holds a turn across marker/start/bind. Exact Task/P/G/Session/full locks stay unchanged; one Workflow/audit advance; real current callback/completion remains eligible. |
| MB-AC2 | Separate SQLite writer changes each P/G/T/Workflow/context/lock predicate during held start, including same-version raw body/REPLACE and added/deleted locks. Binding refuses with no binding/audit writes or new native input. |
| MB-AC3 | Foreign/missing/Lost/ambiguous Session, wrong actor/provider/role/worktree/native UUID/private owner/input pair, extra projection and generic writer attempts all refuse at the actual consumer. Passing producer prerequisites precede each negative. |
| MB-AC4 | Actual owned successful settlement before lost start delivery; normal/late race, deferred commit, dropped Engine future, lost/full notifications and actual Driver wake converge once. Authentic restart preserves required fencing; rows alone never grant admission. |
| MB-AC4.a | Normal returned Starting before input/ACK binds its genuine prepared owner without delivery/success claims. The same NotDispatched owner cannot late-bind. Actual current consumed successful terminal with cleanup Unknown/Leftovers can late-bind once; Unknown work, HistoricalDraft, wrong input/marker/instance and ambiguous external outcome cannot. |
| MB-AC5 | Genuine bind through diagnostic/gate/success and failure closure; exhausted optional allowance still permits mandatory closure; malformed/disconnected/replayed ledger rejects without effects. |
| MB-AC6 | Migrate positive Workflow fixtures through actual producer ports. Capability-only advertisers remain negative. No disabled tests, fake owner/pair, copied authority, fixture exemption or weakened native mechanism. |
| MB-AC7 | Compile clean committed mutations restoring Task bump, refreshing captured frame, dropping each CAS/identity/private-proof/sole-writer/delta/ledger check or inferring success. Each fails at its intended actual consumer assertion; exact source restoration passes. |
| MB-AC8 | Independent immutable requirements/design/source reviews; composed current-main fmt/Clippy/build/full Workflow regressions and macOS/Ubuntu CI. Controlled protocol peers and authenticated CLI/hook/four-Task qualification are reported separately. |

## Delivery and open gates

Requirements approval precedes managed binding design and production changes.
The design must specify actual allocation/frame producers, every writer/consumer,
schema/migration composition, finite cost classes, typed failure/reconciliation
states and current Driver integration. Native readiness remains unavailable until
those real ports compose; an isolated binder test is not a completed delivery.

The legacy approved private #19 ports and this managed profile are not declared
equivalent. An implementation must either compose the actual #19 producer or
deliver its complete prepared-owner/input protocol through the real managed
adapter and all protected writers, with explicit joint design/source review and
the above causal controls. Renaming Unit/Invocation DTOs is insufficient. The
normal/late/diagnostic/recovery requirements remain open until their actual
consumers are verified. No README, license or whole-MVP claim is changed here.
