# Issue 43: Preserve native authority during Workflow Session binding

Workflow: STRICT. Requirements baseline `d8c5266` approved by native independent
requirements re-review4; joint item6/Design4 approved at `77da799` by two independent
reviewers. Requirements6 corrects receipt-sourced authority and production caller;
independent requirements/design delta and all implementation gates remain pending.

## Problem and MVP relationship

A successful Workflow native launch currently binds the returned Session by changing
only `history[active].session_id`, then persisting both Workflow and the unchanged
Task. The Task version increment invalidates native adapters' captured scope. A
still-running decision can become ineligible solely because the runtime registered
its own Session. Main `4851fcd` and Issue 41 `1418b36` have this behavior. Claude
public `1c098e1` atomically fences final decision currency by the captured Task version;
Codex and Grok also use scope snapshots. Fake launchers omit these native guards.

This is necessary for native adapters #5/#6/#7 to compose with Workflow #8/#41,
prepared-input ownership #19, Review #9 and dogfood #16. Existing #42 GitHub ownership
visibility is independent. Recovery #14 and external evidence reconciliation #13
retain their own acceptance criteria.

## Required behavior

1. Binding changes only the active Running native attempt's absent `session_id` to
   its exact persisted returned Session ID, or the exact allocated ID obtained via
   the private receipt-sourced path below. Persist the Workflow record and scoped
   factual audit in one Immediate transaction. Preserve Task, Project, Goal, every
   Session and every scoped WorktreeLock body and version. The binding writes only
   its Workflow record and audit; #19 private binding must not rewrite a native-fenced
   record or change its expected Session CAS/admission pins.
2. Check exact captured Project, Goal, Task and Workflow versions, active Project/
   Goal/Task state, Workflow scope/generation/active attempt, committed dispatch marker,
   current context pointer and identity. Stale snapshots fail without DB/audit change.
3. Require exact expected native actor, role, Task worktree and owning scope. Require
   the persisted Session's immutable ID/scope/provider/actor/role/worktree to equal
   the trusted registered adapter's returned identity, or the exact sealed receipt-
   sourced identity under the additional private contract below. If a native UUID was already
   returned, require it unchanged. A Starting return can legitimately precede PID,
   UUID, Running or terminal publication; binding reads the latest durable lifecycle
   in its transaction without treating mutable startup fields as an identity change.
   No PID/UUID hint grants process ownership or transport completion. Typed input's
   private allocation remains mandatory even when a public identity matches.
   Lost, foreign, malformed, ambiguous or missing Sessions cannot bind.
   Ambiguity includes a returned ID missing its Session record, conflicting immutable
   actor/scope identity, or a duplicate provider/native UUID on a different Session
   within the owning scope. No cross-project native UUID lookup grants ownership.
4. Preserve the original reservation after binding failure, including a still-running
   Session. No release, retry, ownership transfer, native turn or fabricated completion
   follows a failed binding transaction.
5. The binding primitive is private to the crate and narrowly validates the complete
   allowed Workflow delta. It cannot change other history, Task fields, context, actor,
   marker, completion, terminal decision or native outcome. Ordinary phase/context/
   state transitions retain their existing Task-write transaction.
6. Every fresh Workflow native Executor/Reviewer binding requires prepared input
   #19: bind only the exact allocated private Session owner
   for that scoped native attempt in the same transaction. A visible Workflow Session
   ID is not an allocation credential. Closure still checks that owner even when the
   public attempt's Session ID is absent. Issue 19 provides these typed-slot guards;
   this issue must retain and compose with them, without claiming unmerged coverage.
   Derive the obligation from durable native phase/scope/context, never a caller
   Option. Missing private allocation/pair rejects. Historical untyped Workflow
   remains readable history and cannot authorize a fresh native binding. The narrow
   binding port is the sole existing-attempt Session-ID writer; every ordinary
   WorkflowAccess mode refuses that delta. New attempts still start unbound.
7. Native caller completion/callback eligibility remains unchanged by a valid binding.
   A real Task/Project/Goal/lock/currency change must retain its current refusal. No
   provider scope fence or native auth/hook/trust/permission control is weakened.
   Existing binding uses StateOnly (`WorkflowEngine::persist`); it is factual owner
   registration after native launch, not reserve-time operation admission. Preserve
   that distinction: ReadOnly/Mutating reserve-time scope sweeps are not silently
   repurposed to reject the returned live Executor or legitimate parallel Reviewer.
   Other scoped Sessions/locks are untouched and never gain ownership or completion
   from this binding; native admission/currency and future closure guards still apply.
8. Update Workflow master design and verification. Independent immutable source review,
   meaningful controls/mutations, build/test/lint and exact-head CI precede merge.

## Receipt-sourced factual binding delta

Pending #19 Design23 at82537888819a14805f8126d1ee8f23b58ffaad10 describes a case where
the actual supervisor allocated/consumed/completed a successful native operation
and published its known-current settlement receipt, but delivery of the start result
was dropped before Workflow received the Session ID. A public ID remains absent;
the private current outcome must not be replaced by another dispatch or PID adoption.
This is proposed dependency research, not merged producer or recovery evidence.

Allow the SAME sole #43 binder to obtain that ID only from a sealed Store-derived
ClosedSettlementBinding proof, constructed inside its Immediate transaction from
the exact private phase Session owner/operation/KnownCurrentTerminal successful
receipt, current Session body/version checksum and complete current allocated/
admitted/consumed input actor/frame pins. Caller JSON, receipt/session IDs, terminal
labels and public version matches cannot manufacture this proof or a live owner.
Require actual current Exited success and verified profile-required settlement;
NoSession/NoCurrentDispatch, restored prior Exited, unknown outcome/cleanup and Lost
are nonbinding. A row/hint is not genuine supervisor evidence.

Preserve every normal #43 current captured P/G/T activity/version, Workflow scope/
generation/active Running attempt/index, context/source, dispatch marker, complete
scoped locks, actor/role/provider/worktree/native identity and uniqueness predicate.
No caller can select a weaker predicate. Writes remain exact active session_id
None→allocated ID, ordinary Workflow Record version/updated_at and one bounded
reserved factual audit only; no Task/Project/Goal/Session/lock body/version changes.
The proof grants no input/ALLOW/resume/retry/release/transfer or success. Ordinary
#19 current-success closure still independently validates actual consumed current
input, native outcome and complete settlement; factual binding alone cannot pass
a phase or complete a Goal.

For this late path, expected authority is the IMMUTABLE durable dispatch binding
frame captured at marker commit: original P/G/T versions, post-marker Workflow
Record version/marker tuple, context/source and complete scoped lock-set versions/
identities, plus pinned registered actor/provider/role/worktree. Derive it inside
the transaction from #19's private operation/marker provenance, not a later caller's
read of current rows. Receipt, preparation/consumption and current Session must
correlate to that frame. Do not replace it with refreshed current versions or
#19's separately permitted bookkeeping refresh. Any mismatch, including a sibling
Goal bookkeeping version change, rejects with no writes and keeps claim/receipt for
reviewed recovery. No implicit tolerance for unrelated metadata or weakened CAS.

The sole late-binding production route is proposed private Engine
reconcile_settled_native_binding, invoked by the authoritative #23 active Task
driver on the exact #19 settled-operation notification, and by that same managed
launch invocation's post-marker start-Err handling. The driver/notification/private
proof must actually compose in the source PR; a test-only direct binder call cannot
close this acceptance. Passive #41 step/poll observers, status and ordinary record
reads never invoke it or bind. It is a narrowly permitted factual Workflow write,
not release/retry/adoption or a new-input progression grant. Generic caller JSON
cannot produce its private operation/frame/receipt evidence.

A managed post-marker start error/lost delivery branches on actual private state:
KnownCurrentTerminal successful receipt invokes that route and never fail/retry;
an existing/pending/unknown operation without a conclusive receipt stays held under
its real supervisor; NoCurrentDispatch permits only its existing non-success closure;
known current failure uses the separately reviewed #19 non-success closure without
binding an absent Session ID. Do not map a missing start result to native failure
or release. Restart may re-enter this active-driver factual route ONLY with authentic
durable current-operation/frame/known-success proof and the actual reviewed #14
restart fencing/restore protocol. It reconstructs no native owner and keeps all
normal active/source predicates. Absent those producer/recovery ports the path is
Unsupported/held; restart integration remains mandatory before its acceptance.

Race with a delayed normal return still permits one exact binding/audit only. A
losing binder preserves the original claim/receipt and grants no fail/retry/input;
any AlreadyBound diagnostic must prove the same exact operation/private bound fact.
Audit carries bounded proof-source enum and exact operation/receipt references,
never receipt payloads or an agent-selected authority class.

An inactive owner or stale source/authority rejects while preserving the original
claim/receipt/reservation. There is no inactive/restored recovery exemption; future
#14 recovery mode requires a separately reviewed #14/#19/#43 contract. Keep the
combined real #19 producer/all-writer epoch and native-profile source gates before
deployment, with no generic/public receipt path or temporary permissive port.

Exercise an actual owned supervisor finishing/recording settlement before delivering
its start result, drop that delivery, then invoke the real binder and ordinary
current-success closure. Compare exact unchanged Task/P/G/Session/full locks and
one factual audit/Workflow advance; demonstrate no second dispatch/native bytes.
Independently reject stale/inactive/foreign/restored/missing private proof and wrong
operation/input/native identity with no DB/audit mutation. Compiled actual-consumer
mutants substitute caller receipt/ID, omit current input/receipt/owner checks, accept
restored-prior or inactive evidence, broaden writes, or infer success from binding;
each must reach its intended assertion with actual passing producer prerequisites.
Controlled fixtures must create proof through real private ports, never seed SQL.
Drive the actual managed Engine start-error and #23 driver-notification routes
after real supervisor success, plus actual restart protocol where claimed; assert
no Failed closure, retry or second dispatch and passive observers remain unchanged.
Independently change each P/G/T version and a lock-only frame after marker commit,
then require rejection with no row/audit mutation and preserved receipt. Kill
freshly-recaptured expectation, missing actual-driver receipt lookup and fail→retry
mutants at those actual consumers with exact restored passing controls.

## Impact and verification

Consumers: Workflow native launch binding, State persistence/audit, native scope
snapshots and private completion, decision/approval observation, #19 allocation and
closure, #41 pre-dispatch claims and terminal recovery. No schema change is required
solely for record-only binding. No broad Task-write optimization is introduced.

Use isolated temporary Projects/Git worktrees/SQLite databases. Hold an actual
registered adapter's native protocol turn across Workflow binding, then observe its
private completion/callback. Prove unchanged Task/version, one Workflow version advance,
one factual audit and valid exact binding. Retain observed native failures separately
from fake-protocol controls and avoid claiming real model acceptance from a fake.

Inventory the exact role/consumer at the immutable integrated source revision:
Claude `session::supervise` decision final `Reservation::commit_current` uses raw
Task/P/G/lock and Session CAS. Executor final currency is observational and cannot earn
decision mutation credit; its initial input and Broker ALLOW publication separately
use that full fence before their wire write. Grok `Actor::owner`, called before startup/callbacks/final
native result, checks `ScopeSnapshot::recheck_scope` for every role. Codex
`answer_approval` checks that scope before an operation reply; current Codex transport
completion alone does not establish final decision-currency enforcement. The hold
and Task-rewrite mutant must exercise each row below, with a passing control and
compiled failing mutant at the actual listed native protocol consumer:

| Provider/role | Required held-turn consumer after Workflow binding |
| --- | --- |
| Claude Reviewer | Decision completion through `Reservation::commit_current` |
| Claude Executor | Broker ALLOW publication through `Reservation::commit_current`, before the exact permission reply wire write |
| Grok | Each exercised ownership stage: startup, native callback and final result through `Actor::owner` |
| Codex Executor | `answer_approval` current-scope and Session guard before its exact native operation reply |

Hold the Claude Executor native permission request across binding, then prove its
exact ALLOW response survives binding and fails under the Task-rewrite mutant.
Its DENY response is a required control: it grants no operation authority and
uses own-Session publication without ALLOW's raw parent-version CAS: a Task
metadata/version change must not prevent exact denial. Existing ProjectBlocked,
own-Session CAS/storage and activity/worktree refusals still stop without wire;
this is not a universal denial exception to those constraints. Preserve the adapter's existing explicit Broker opt-in/native policy;
the test must not relax automatic native review. If an integrated Workflow caller
cannot expose this path, record the concrete caller limitation rather than silently
omitting the row. Initial input admission before `start` returns does not earn
post-binding coverage. Grok returns Starting before its owned actor task runs;
its startup, callback and result stages require held-task controls after binding.

Record distinct behavior and unavailable guarantees; do not invent a common final
decision fence or infer a kill from an unfenced Executor completion. These rows are
required composition checks, not interchangeable alternatives. Also compile a binding
mutant for each preserved class: Task, Project, Goal, Session and scoped WorktreeLock.
Each must meet a named real currency consumer; Claude `put_session_if_current` is
one known consumer of all five classes, including its own Session CAS and the exact
full scoped lock set. Sibling Session preservation also needs direct before/after
body/version assertions; an own-Session CAS cannot prove a sibling was unchanged.
Include an own-Session CAS race and a lock-only version change so neither can be
masked by a concurrent Task change. Restore exact source bytes and pass controls.
If a specific role/provider has no consumer for a class, record that unavailable
guarantee explicitly while preserving the class and testing its known consumer.
Codex/Grok `recheck_scope` also recaptures the complete Project against the original
request; include that `capture` comparison when identifying the Project consumer,
rather than inferring a missing Project fence from the later Goal/Task/lock comparison.
Read other live Reviewer and Lost/reserved Session facts without changing or adopting
them, and verify this factual binding cannot serve as new-operation admission.

Second SQLite writers change Task, Project, Goal and Workflow versions; invalid actor,
role, worktree, context, marker, Session identity/state and extra record deltas fail
without DB/audit change. Hold start before binding and preserve passive #41 polling.
Typed private-owner tests compose with #19 after its implementation is available.

Compile mutants restoring the Task rewrite, omitting each meaningful CAS/identity
boundary, or admitting additional Workflow changes. Assert causal real consumers fail,
restore exact bytes and pass controls. A merely mirrored helper test is insufficient.

## Limits

Arbitrary parent bookkeeping during a live turn is not authorized by this primitive.
Actual identity drift still requires explicit recovery; this issue does not guess
ownership or adopt PID/UUID hints. Restart, unknown dispatch,
explicit retry with unbound markers, Review Set slots, scheduler and external side
effects retain their separate issue contracts.
The receipt-sourced active-driver route above composes authentic settled-current
restart evidence only after #14 gating; all other restart/unknown/unbound recovery
and inactive success remain outside this binder and retain their separate contracts.
