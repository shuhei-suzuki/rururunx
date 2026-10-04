# Issue 43 design: record-only native Session binding

Status: proposed Design9 from approved Requirements9 at91d4f34; independent STRICT
design/source and actual composed producer/native/recovery gates pending. No implementation.
Requirements: [issue-43-requirements.md](../requirements/issue-43-requirements.md).
This supplements [the shared Issue43 design](issue-43-design.md). Both describe
the same proposed gate; neither is implemented. The exact transaction mechanics
below resolve audit, lock and private-owner details before source work.

## Cause and scope

At main `4851fcd`, `WorkflowEngine::prepare_agent` persists the dispatch marker
through the ordinary Task/Workflow transaction, then calls the selected registered
adapter. After `start` returns it sets the active attempt's Session ID and calls
that same transaction again. `put_task_tx` increments Task.version even though
the Task body was otherwise unchanged. Native scope snapshots captured after the
marker are thereby invalidated by their own runtime registration.

Use the sole private binding transaction for normal return and authentic settled-success late registration. Dispatch marker/ordinary phase transitions retain their paths; the two bound-live poll diagnostic arms use the separate private record-only diagnostic port below. This does not
make an unchanged Task write generally optional. No new native operation or
permission is admitted by binding, and transport exit never becomes gate evidence.

The dedicated transaction is the sole writer of an existing attempt's Session ID.
Every ordinary `put_workflow_transition` access mode and its `validate_transition`
reject any existing history entry's session_id change. Newly appended attempts
still require session_id=None. Keep closure guards on the ordinary path; move
binding validation to the narrow path. The approved narrow authentic settled-success route uses this SAME sole binder and actual #14 restore/fencing; all other recovery retains its own private ports. Current production has one assignment (workflow.rs:1016)
and one native reservation initializer with None (workflow.rs:879); Evidence's
separate session_id is an outcome reference, not a PhaseAttempt writer.

## Engine call and trusted identity

For every native Executor/Reviewer phase, resolve the selected adapter and probe
outside the Store mutex **before reserving any PhaseAttempt**, publishing that
phase's ContextVersion or committing its dispatch marker. In current step this
belongs after the source/policy-drift branches and before clear_hold/prepare_pack
(workflow.rs:857); the capability check currently in prepare_agent at :957 is too
late. Keep legitimate prior policy invalidation as its own separate return path.
Native Git/gate phases do not select an AgentAdapter through this gate.

Require probe.agent to equal the selected registered name, a nonempty provider,
the required Execute/Review role, and the new implementation-owned
Capability::PreparedInputAdmission in both capabilities() and probe.capabilities.
This capability is absent by default and advertised by Rust implementation only
after its actual #19 Session admission port is wired. Config cannot enable it;
there is no class exemption, caller flag, JSON discriminator or test-only
production bypass. It promises that a fresh Workflow launch creates the real
private owner/preparation pair before returning; it does not itself prove that
pair. Store always checks the actual durable proof independently.

Missing adapter/reviewer, unsupported capability, invalid probe identity or
probe failure returns an explicit error without calling fail, creating a phase,
clearing a durable hold, writing a ContextVersion/Session/audit, claiming dispatch
or launching a native child. Retain the selected immutable adapter/identity for
prepare_agent and recheck current policy/owners before marker as usual; do not
silently select a different adapter after preflight. Existing registry only
validates name uniqueness. Preserve executable validation, authentication,
hooks and native default model/effort.

Retain a crate-private `NativeBindingIdentity` built from the selected registry
name, probed provider, exact launch Scope, role and Task worktree, together with
the returned Session ID and returned optional native_ref. The returned actor,
provider, role, scope and worktree must match these captured values. A returned
Lost Session is rejected even if its durable record subsequently changes. The
returned Session is an identity observation from the selected adapter, not a
process handle, native ACK or prepared-input credential.

INSIDE the dispatch-marker Immediate transaction capture every scoped WorktreeLock ID/version and resulting post-marker P/G/T/Workflow marker frame before calling start. This is the same complete set used by native
scope consumers, including inactive historical lock records. Retain it in the
private binding expectation; no lock is created or adopted by this capture.

After successful start, create the candidate Workflow data by setting only the
original active attempt's absent Session ID inside the binder. Call private
`Store::bind_workflow_session` with the genuine private operation identity and
NormalReturned identity (or the private ClosedSettlement variant). The transaction
reads its sole expected tuple from the immutable Store-derived marker frame; caller
values only cross-check this tuple and cannot replace it. Do not
refresh owners, recapture a newer Task version or retry native/phase admission on CAS failure:
that would silently accept revocation during native start. On success update only
the caller's Workflow record to the committed version and return Started. Keep
the caller's Task byte-for-byte at its pre-start snapshot/version.

A definitive binding refusal propagates a typed binding/refusal result, never a native
failure. NormalReturned Transient or CommitUncertain returns StartedBindingDeferred;
the genuine managed invocation retains the returned identity, immutable original frame
and actual owned launch independently of the caller future. It retries/reconciles only
that factual binding under the policy below. Leave the already committed unbound
dispatch claim and any durable Session intact. Do not call `fail`, retry
start, stop/adopt a Session, release the claim, or publish completion in that error
path. Observer polling remains passive under #41; #13/#14 own reconciliation.

## One Immediate transaction

The method is crate-private and performs no await, filesystem access, child launch
or native wire operation while holding Store. SQLite Immediate excludes other
writers across connections. All checks and the one Workflow/audit write use its
Transaction; a failed check, write or audit rolls back everything. Update the
caller's mutable Record only after successful commit.

1. Read the immutable private marker frame and its Project, Goal and Task by exact
   IDs. Validate their owning edges, frame versions, Registered Project, active Goal
   and nonterminal Task. Caller snapshots can only cross-check this frame; no
   caller-supplied Task change is admitted. No Project/Goal/Task write follows.
2. Read the existing Workflow by ID and require frame Workflow-version equality
   and immutable ID/kind/Scope. Its checked version
   covers factual timestamps; no caller Record supplies an alternative. Require exactly
   one Workflow in the Task
   scope. Decode with existing Workflow validation, check Task workflow/risk,
   revision and context pointer, active index bounds, current generation, a native
   Executor/Reviewer phase, Running state, absent Session ID, committed
   dispatch_started, expected agent, and no completion or finished authority.
3. Clone the complete previous JSON data and replace exactly
   `history[active].session_id` with the NormalReturned identity or the genuine
   ClosedSettlement allocated owner ID derived inside this transaction. Require this Value to equal
   the internally constructed candidate data. Post-epoch managed bodies reject unknown
   stored fields before decoding/hashing; complete equality covers EVERY typed field. The caller supplies no mutable candidate or alternate expected tuple.
   Run the ordinary structural validator on the unchanged previous Workflow and
   explicit private binding validation on the candidate. The ordinary validator
   must reject this ID change, so do not introduce a caller-supplied bypass flag.
   The private validator requires the complete one-field delta and every binding
   predicate. Do not validate only a subset or round-trip away unknown fields.
4. Read every scoped WorktreeLock ID/version, sort and compare the complete set
   to the captured set, rejecting additions, removal and version change. Read
   the latest ContextVersion in the exact Task scope. Require it to be the
   Task/attempt/Workflow context_version and run `validate_context` against the
   candidate. This preserves source hashes, generation, phase and revision. It
   neither inserts ContextVersion nor performs new-operation source admission.
5. Read the returned ID's durable Session Record and enforce record/payload
   ID, exact Scope, kind and immutable actor/provider/role/worktree equality with
   the trusted binding identity. If the returned native_ref was Some, require
   the latest durable value to be identical. A None return may legitimately
   become Some before binding. Read current lifecycle inside this transaction:
   Starting, Running, WaitingApproval, WaitingHuman and factual known terminal
   Exited/Failed/Stopped may bind; Lost, absent, invalid or foreign records reject.
   PID, actual model/effort, recovery metadata and Session record version may
   legitimately advance between start return and this read; do not compare them
   to a stale complete returned snapshot or rewrite them.
   There is no stale returned-Session version CAS: the latest Session is read and
   identity-checked while Immediate excludes writers. An own-Session CAS race at
   the provider's currency consumer remains separately required; binding may not
   invalidate that consumer by rewriting its Session record.
6. Inspect scoped Session identities using the latest durable native_ref read in
   step5 whenever it is Some, including when start returned None. A returned Some
   must equal that value. Reject another scoped Session with the same provider
   and UUID, including a live or Lost sibling. Do not use a cross-project UUID
   search, PID hint, public recovery JSON
   or UUID equality to acquire ownership. The current Workflow calls fresh
   `start`, not `resume`; if a future caller reuses a native UUID through a
   different predecessor Session, a reviewed private resume-lineage port is
   required before exempting that predecessor. No generic JSON exception.
7. Every fresh Workflow native Executor/Reviewer binding requires #19's private
   allocation/input-pair predicate in this same transaction. Store derives this
   obligation from durable native phase/scope/context; no caller Option or public
   discriminator can skip it. It must match the allocated owner, exact context
   version, payload SHA/size, revision and complete sources and current native
   actor pins. A private validated Starting preparation may bind before ACK;
   it cannot assert delivery. An old terminal snapshot restored after failed new
   preparation cannot bind as the new input actor. Binding writes no Session,
   ContextVersion, WorktreeLock or native-fenced CAS metadata.
8. Save only the Workflow Record, incrementing its version once and updating its
   normal updated_at, and append exactly one scoped factual audit. Commit.

Do not use the ReadOnly/Mutating reserve-time Session/lock sweep here. The returned
live Executor and other legitimate Reviewer Sessions can be present. Read other
Sessions for identity ambiguity only; unchanged sibling live/Lost facts neither
grant ownership nor reject this factual binding merely by being present. New
operation admission and future closure keep their separate existing guards.

The SQL count and Session scans remain bounded by the owning Task's stored scope;
they do not scan other Projects. Native inspection/execution/cleanup deadlines are
unchanged; bounded active-driver wake is scheduling only. The binding/diagnostic
ports join the composed writer epoch and have no independent schema history.

## Persistence and audit implementation boundary

Reuse `put_record_tx`'s record CAS/version validation, immutable scope/kind,
worktree exclusion and serialization path. Introduce a private typed audit choice
only if needed to supply the binding fact: the existing helper delegates to the
same internal record writer with its current default audit, while this one caller
selects `rrx.private.workflow.session_bound`. It replaces the default workflow.saved audit
for this call, rather than adding a second event. Preserve every other caller's
existing event name/payload and write behavior.

Reserve the exact `rrx.private.workflow.session_bound` event name in `reserved_audit_kind`:
public `audit` and `audit_if_current` must refuse it. Primary inspection found
the current reserved list includes .saved and workflow.gate_observed, but would
allow this new name unless extended. Test both APIs and kill the omitted-guard
mutant; public caller facts cannot impersonate the private binding journal.

The binding audit uses ONLY the single exact4KiB field list in shared Design section11,
including proof_source and exact operation/receipt references. The complete Workflow
delta is durably present in its own record. No prompt, environment/config value,
permission/native response or outcome/delivery claim. Private audit choice is not a
public arbitrary-event API and cannot substitute for required Store guards.

## Integration with typed ownership and follow-ups

Issue19's final composed candidate schema/private source is not yet implemented; main is schema3. Its reviewed design allocates
one phase_session_owners row per scoped native context_version; every typed
Starting/first-consumed write allocates it atomically. The binding predicate must
read that row and the private validated-preparation/admitted input pair using
the existing Transaction, never a preceding unlocked lookup or public fields.
There is no permissive fallback if the private row is absent or inconsistent.
Historical untyped Workflow is readable history, never eligible for a fresh native
binding. Every fresh native phase must obtain the actual #19 private owner/pair.
The Engine supplies no optional allocation authority; an expectation can only
cross-check Store-derived proof. An omitted caller expectation never skips it.

Implement the narrow record-only path in isolation with production failing closed
when the private predicate is absent. No permissive predicate or SQL-seeded pair
earns positive binding acceptance on that staged branch. Then compose the
actual #19 predicate before claiming typed-native acceptance or merging this
issue. Use a co-integration branch preserving both source commit ancestries for
the #19 predicate and #43 binding port. Run their approved combined source review,
native integration and exact-head CI on a **single combined PR merge vehicle**
before either issue closes. Preserve both source ancestries in that combined PR;
main never receives a ready #43 binding port without #19. Update superseded draft
PRs to link that vehicle rather than separately merging their unintegrated heads.
This avoids making either completed merge a prerequisite for the other's acceptance. Neither
independent staged branch claims integrated native acceptance. Coordinate the
private helper signature with the #19 owner; do not create
a second competing allocator or copy private authority into Workflow JSON.
Typed closure must still check the allocated owner even while public Session ID
is None, and still require actual current-input consumption for Succeeded.

Combine #41 and approved actual #5/#6/#7 source in an isolated integration branch
for native composition checks. Changes to their private consumer signatures are
reviewed on that immutable combined revision. #46 process inspection and the
Grok environment-isolation follow-up retain their own scopes; binding cannot
weaken cleanup or environment authority to make acceptance pass. ReviewSet9
multi-member slots are a separate typed port, not an exception to one owner.

## Actual admission writers and fixture migration

"Native" here describes the Workflow Executor/Reviewer phase, including generic
and protocol-controlled adapters, rather than an exemption limited to native
provider brands. There are five known launch implementations at the inspected
public baselines. None earns compliance from capabilities alone. Recheck this
inventory on the combined source, including new #41 barriers and all impl
AgentAdapter occurrences, before implementation review.

| Launch implementation / primary baseline | Current Session write and required migration |
| --- | --- |
| GenericCliAdapter, 0665583 adapter.rs:628/:695 | Starting/Running call save_session then generic put_session. Current metadata has input_bytes and lacks payload SHA/current-consumption proof. Move validated Starting and actual input consumption to #19 NativeCAS with exact P/G/T plus complete lock-set expectations, all five prepared-input pins and runtime-created unique intent. Consume atomically before the real stdin write; no invented native ACK. Keep existing Execute-only role and unsupported model/effort behavior. Advertise PreparedInputAdmission only after these writes are wired. |
| Workflow FakeAgent, 0665583 workflow/tests.rs:48/:86-96 | Bare put_session and recovery Null currently provide no pair. Migrate the shared fixture to real #19 NativeCAS preparation/admission under the actual owner/lock snapshots, full payload pins and unique intent before start returns. Its controlled successful wire boundary represents fixture consumption only. No allocator stub or generic Session write may create a positive owner. Fake execution is synthetic evidence. |
| Claude, f9b671f claude/session.rs:186-194 and :140-163 | Fresh persist uses generic put_session; commit_current uses put_session_if_current. Fresh preparation must use #19 NativeCAS, and the existing current admission must supply its full protected preparation/input pair and actual pre-wire consumed intent. Move every operation/terminal journal out of that canonical consumed key as specified below. Terminal observations and validated rollback remain their distinct modes. Preserve ALLOW-only current fences and independent DENY own-Session publication. |
| Codex, 6749505 codex/session.rs persist_unchecked/private rollback and admit_dispatch | Generic fresh/checkpoint preparation and rollback are distinct from current admit_dispatch. Wire fresh/committed checkpoint through #19 NativeCAS, include missing revision/source_versions in the consumed DTO, keep private exact restore provenance, and preserve F4 admission-mutex/first-cause ordering around the consumed CAS and first wire. Stop/caller-drop cannot relabel failed admission. |
| Grok, 65aa940 adapter/grok/mod.rs:225/:322 | save_current already uses put_session_if_current, but initial Starting lacks the five prepared-input pins. Supply #19 protected preparation/actor pins plus unique intent at actual consumption before callback/startup wire. Terminal save_session is factual observation, never allocation or consumed admission. |

Issue19 distinguishes NativeCAS from ObservationOnly: generic put_session cannot
allocate, prepare or consume. Each fresh preparation/admission/consumption write must use the real private
NativeCAS port after exact P/G/T and full lock-set CAS. Historical operation
observations follow their separately constrained modes below. Neither the binder
nor Workflow manufactures that proof. At these baselines no other Workflow test
adapter implements AgentAdapter: #41 extends the same FakeAgent with barriers.

The default from_config path currently registers GenericCliAdapter. Migrating it
is part of the combined implementation; it is not an accepted MVP limitation.
Before migration, the explicit pre-reservation capability rejection prevents a
permanently held launch. Unsupported new adapters also reject there. Separately,
an adapter that falsely advertises the capability and returns an ObservationOnly-
acceptable initial known-terminal Session without private rows must fail the
actual binder after start, retain its committed unbound claim and durable Session,
and produce no binding audit or release/retry. A fresh protected Running write
can instead be rejected by #19 during start; that setup refusal earns no binder
mutant kill. This is a contract
violation distinct from honest unsupported preflight. Positive generic and fake
controls must both obtain real allocation/pair through their actual start paths.

Primary test inventory at 0665583 finds one AgentAdapter impl, two PhaseAttempt
literal constructors starting with None, sixteen ordinary
put_workflow_transition call sites and no direct assignment of
PhaseAttempt.session_id in tests. The assignment to evidence.session_id at :331
is a different outcome field. The sixteen calls initialize unbound history or
preserve bound IDs across evidence/authority/terminal/cancel transitions; there
is no verified direct-bound-seeding defect. Engine start/binding builds their
bound attempts today, so migrate that common path and shared FakeAgent rather
than granting ordinary transitions a fixture exemption.

The existing explicit_stricter_selection_and_unsupported_review_capability_are_honest
control currently observes post-reservation failure; migrate its unsupported branch
to the new pre-reservation explicit error and assert no attempt/claim/context/audit
write. Registry/probe failures get the same causal consumer controls.

Required bound consumers include all_presets_drive_real_adapter_calls_and_persist_phase_context_history,
restarted_native_attempt_reports_durable_recovery_without_rebinding_or_launching,
lost_reviewer_also_requires_verified_recovery_before_retry,
quick_requires_actual_merge_cleanup_before_terminal_and_cancel_never_implies_native_death,
inactive_owners_allow_cancel_but_terminal_release_requires_verified_session, and
rejected_review_cancel_releases_only_verified_terminal_attempt_and_project.
Recheck #41's preparation_dispatched_observation_and_direct_store_terminal_fence,
preparation_pause_cancel_and_terminal_recovery_keep_owner_fenced, and
preparation_release_cas_and_executor_lost_fences_do_not_retry in co-integration.
Their bound fixtures must come from Engine + actual #19 admission + narrow binder.
Capture FakeAgent's scope/lock snapshot at start entry before its pause, as real
adapters do; persist preparation after the pause so the existing no-Session-during-
held-start control stays meaningful. Recapture must not erase a paused interval
revocation. Required current-source revalidation before first wire remains separate.
Historical direct state seeding, if newly introduced, is history-only and never
binding coverage. Existing unchanged-ID ordinary transitions keep closure proof
and their normal Task writes; every forbidden ID delta fails before any write.

Add real Engine controls for missing admission capability (no phase/context/claim,
Session, audit or child), misadvertised admission (held post-marker contract
failure), and generic/fake/native successful admission. Kill an omitted-capability
check at the pre-reservation consumer, and omitted-private-proof check at the
post-start binder. A setup refusal is not a private-proof mutant kill. Inventory
all protected writer and fixture consumers again at the immutable combined head.

## Post-consumption operation publications

The protected recovery.dispatch_intent is solely the canonical prepared-input
consumption DTO. It retains its runtime UUID, complete input pins and authority
versions through the corresponding native input outcome. No permission, denial,
terminal transport hint or callback replaces it. Private allocation/preparation/
admitted rows remain unchanged by those observations. A second consumed input
still requires its own reviewed fresh-admission path; an operation journal never
becomes one. Align this contract with #19 before joint source acceptance.

| Actual writer / baseline | Journal and private publication mode |
| --- | --- |
| Claude f9b671f explicit permission reply :1639/:1647/:1653 | Write a separate bounded recovery.operation_intent journal, retaining pending native request/operation digest and actual ALLOW/DENY fact. ALLOW keeps commit_current/NativeCAS with full current scope and lock fences before wire. DENY keeps own-Session CAS and existing activity/worktree constraints through the explicitly allowed Acked historical observation outcome, without a raw parent-version CAS or new consumption. |
| Claude automatic DENY :1496/:1513, pending/cancel :1504/:1531 | Move all decision facts away from dispatch_intent. Exact pending request publication/removal and WaitingApproval-to-Running observation keep actor/input/consumed pair unchanged and use the historical observation mode. Broker-disabled/capacity denial remains durable before wire; failed publication stops without a phantom response. |
| Claude terminal transport :411/:1696 | Protected Tasks reject Interactive/PTY startup and terminal_input before reservation, Starting publication, spawn or terminal bytes. Do not allocate a typed phase merely to reject later; pending Waiting states grant no bypass. Outside protected Tasks, transport facts may use a separate bounded recovery.transport_intent journal, without fabricating prepared input consumption or typed Workflow authority. Workflow is NonInteractive. |
| Codex 6749505 answer_approval :2020–2062 | The actual code does not overwrite dispatch_intent. Its separate codex.approval.reply_intent audit precedes reply: Approve uses audit_if_current then put_session_if_current/NativeCAS; Decline uses audit then unchanged own-Session put_session/historical observation. Post-reply removal preserves the consumed pair; any Running-target publication still requires the remembered full version fence under #19. Exact unchanged Waiting fixed Decline is a separate historic nongrant case. Preserve existing audit/wire ordering; do not invent a new consumed intent. |
| Grok 65aa940 callback :804–927 and prompt intent :1041 | Callback ownership/FS checks and grok.fs_observed audit remain separate from input consumption. Reject-only native permission callbacks confer no grant. The input intent at prompt dispatch becomes the canonical DTO during #19 migration; callbacks cannot replace it. Preserve existing Actor::owner before every side effect/result. |

For this operation publication, #19 Acked requires the new protected tuple to
match the exact existing private admitted digest and the same consumed UUID.
Historical operation observation is not a new admission mode: it checks exact
own Session version and immutable actor/native identity, unchanged prepared/input/
consumed pins and matching private pair. It may publish the actual existing input's
pending/decision lifecycle, including WaitingApproval-to-Running, under #19's
explicit Acked historical observation outcome (within its Acked/ConsumedHistorical
classification). It allocates/consumes/binds nothing,
changes no private digest/owner, and grants no operation authority. Lost stays
absorbing and malformed/foreign/absent pairs reject. Preserve existing generic
activity/worktree constraints and own-Session/write failures. Preserve the actual
provider-specific ordering: Claude DENY changes WaitingApproval to Running before
wire and ProjectBlocked stops that write; Codex Decline prewire persists unchanged
WaitingApproval, a safe update that can reach wire under ProjectBlocked, then its
postreply Running publication can fail. Do not add a universal denial parent fence.
ALLOW's separate full current NativeCAS and actual owned pending-request proof
remain mandatory. Parent Task metadata/version revocation alone cannot turn
DENY into an operation grant or require ALLOW's parent-version fence.

Pre-reply Running-target/ACK observations preserve exact tuple/consumed UUID and remembered full P/G/T version fences under #19. Only the specifically exact unchanged-Waiting fixed Decline and conservative Lost diagnostics use historic provenance without renewed parent grant.

Inventory every consumer of the renamed operation/transport journal: Claude
permission assertions, reserved intent audit payloads, SQL ALLOW-failure trigger
(current :3005) and control/mutation readers. The shared Session.saved projection
currently selects only dispatch_intent: extend it with bounded operation/transport
facts so Claude decision audits remain observable. Retarget :2486/:2493/:3005/:3022
to operation_intent and pair refusals with successful populated-journal controls. Preserve event names/order and
update the causal field selection; no assertion should begin reading the
prepared-input UUID as a permission decision. Add post-binding consumed-input
controls: ALLOW survives and later exact Succeeded closure still resolves that
same pair; DENY survives Task metadata/version revocation, while ProjectBlocked,
own-Session CAS/storage failure and caller cancellation retain their current
no-wire behavior. Codex Approve/Decline and post-reply Running retain the exact
consumed UUID/pins; Grok callback/result retains its consumed proof. Kill an
operation-overwrites-consumed-key mutant at the actual native reply plus #19
closure consumer, and a DENY-through-parent-CAS mutant at the actual denial wire.
Exercise actual Claude Interactive startup and terminal_input entry points on a
protected Task: refusal leaves no new reservation/Starting row/spawn or wire,
including a pending Waiting state. Kill the omitted-protected-mode guard at these
actual consumers; a Workflow-only NonInteractive setting is not coverage. These
requirements align with #19 Design15; neither source gate is complete.
No current production caller invokes step outside library tests at this baseline.
Scheduler/Goal/CLI integrations must surface bounded configuration errors from
pre-reservation Err and avoid hot retries; they cannot synthesize Failed, release
holds or acquire a permit by treating Err as successful idle progress.

## Verification and meaningful mutations

All positive binding tests run on the actual #19/#43 co-integration revision.
First test Workflow's actual successful-start consumer with a protocol-controlled
registered adapter that persists a real Session through #19 NativeCAS admission.
Assert P/G/T and every scoped
Session/WorktreeLock body/version unchanged, Workflow.version advances once,
only the active absent ID changes, and exactly one binding audit is appended.
Exercise a Starting return followed by durable Running, effective binding and
terminal publication before registration; none grants completion. Assert passive
observers during held start and binding failure, retaining #41 behavior.

Use second SQLite writers for P/G/T/Workflow revocation and own Session deletion,
Lost or identity drift. Exercise context-pointer/source mismatch, duplicate UUID,
missing/wrong actor/provider/role/worktree, invalid marker, stale generation,
extra candidate JSON/Record metadata delta, two Workflows and audit/write failure.
Each failure preserves the durable claim,
all unaffected rows and the audit count. Test typed allocated-owner mismatches,
absent/foreign pair, pre-ACK preparation, restored old terminal snapshot and
unbound closure after actual #19 integration.

Sibling live Reviewer, Lost and executor-reserved records are success controls
when their UUID does not conflict: binding succeeds and changes none of their
bodies/versions. Use a held adapter/read barrier immediately before returning to
the Engine so
independent native writes cannot contaminate the binding-only preservation interval.
For separate admission controls rebuild the reserve candidate from each current
Workflow version and isolate the refusal cause: Lost or executor-reserved sibling,
or live Session for Mutating. ReadOnly with only a live Reviewer can instead fail
later on existing active history and earns no admission-sweep kill credit. Its
existing refusal/StateGuardError must stay exact.
validate_worktree_exclusion has Session/WorktreeLock branches and no Workflow
branch, so the shared record writer does not itself reject these sibling positives.
No sibling gains currency/completion, and binding returns only the committed
Workflow record, no value accepted as an admission credential. Actual validators
still require their private owner/pair. Kill a reserve-sweep-in-binding mutant and
a bound-ID-as-admission-proof mutant at those causal consumers.

Add ordinary-transition controls for every WorkflowAccess rejecting existing
None-to-Some, Some-to-None and Some-to-different ID changes, in active and nonactive
history entries, and new-history injected IDs. Unchanged existing IDs remain valid
for ordinary phase/context/terminal transitions with their separate guards.
A caller omitting allocation
expectation must still fail on absent/restored old private ownership. For UUID
uniqueness, hold a None return, publish a duplicate durable Some before binding,
and reject without write/audit; kill the returned-value-only lookup mutant.
Preservation assertions surround only the binding transaction: independent native
startup can legitimately advance its own records outside that interval.

At the exact combined revision exercise every requirements matrix row: Claude
Reviewer final commit_current; Claude Executor Broker ALLOW before wire with its
DENY control; Grok held startup, callback and final owner checks; Codex Executor
operation approval before wire with a DENY control. Keep explicit native broker
opt-in and current native policy. Claude's ALLOW-only condition is confirmed at
f9b671f: the repeated scope checks and commit_current are inside `if allow`, while
DENY uses commit_session_only. Codex's preflight is likewise Approve-only; denial
still needs its own Session identity. Pin exact observed error/publication per
caller instead of assuming all Session/P/G/T losses yield the same error kind.

Provide passing controls, then compiled mutants restoring Task rewrite and
changing each preserved P/G/T/own-Session/lock class. Run the named actual currency
consumer so the change causes its real refusal. Use own-Session-only and lock-only
mutations without Task change, and direct sibling before/after preservation.
Mutate meaningful binding CAS, complete-delta, trusted provider/actor, lifecycle,
context and private-owner checks; assert the actual Engine/Store path fails at
the causal consumer. Mere compilation failure or setup failure earns no kill.
Restore source bytes and passing controls; record native failures distinctly from
fake-protocol tests and never claim real-model acceptance from a fake child.

Finish with immutable independent implementation/security reviews, scoped and
workspace regression appropriate to the integrated source, fmt/clippy/build,
and exact-head Linux/macOS CI. Preserve failed CI and native observations.
Update Workflow master design in the implementation PR. Issue43 remains open
until native matrix, typed integration and merge criteria are actually satisfied.

## Approved-design implementation inventory

Both Design4 reviewers approved77da799. The following actual-consumer details
remain mandatory source work, with independent immutable source review: Fake
receives the original marker frame before its existing pre-preparation pause and adds a second post-
preparation/pre-return barrier for binder P/G/T/Workflow/full-lock CAS mutations.
Injection before preparation earns only start-admission refusal, no binder kill.
A protected terminal_input guard mutant needs a reachable native entry: an
unprotected-era Interactive Consultant on a Task later protected by a ReadOnly
Reviewer frame, or the actual native-entry guard seam with retained owned terminal.
Startup rejection cannot itself kill an omitted terminal_input guard. The source
review must resolve the reachable construction rather than count setup failures.

## Design9: one immutable marker frame and receipt-sourced route

Approved Requirements9 is authoritative. The marker transaction itself constructs
and privately persists `DispatchBindingFrame` in immutable #19 native-operation
marker_P/G/T/W_version columns plus full sorted scoped lock IDs/versions/digest,
Workflow generation/active index/context/source/marker tuple and registered actor/
provider/role/worktree. The composed marker still advances Task and Workflow versions
once, preserving current marker behavior. Build an exact checked deterministic write
plan under Immediate: unchanged parent versions, Task+1 and W+1. INSERT the operation
with these resulting versions BEFORE Workflow marker UPDATE, then execute the planned
Task/Workflow writes and assert every resulting version/full frame before commit.
The marker trigger verifies the resulting Workflow row against that inserted operation.
Any overflow, unplanned write or mismatch rolls back all rows. Never UPDATE an
immutable operation frame after insertion or save pre-marker versions as expectations.
Capture the full lock set INSIDE this SAME transaction; proposed #19 Design31 supplies
the matching producer contract, not an already implemented schema or source port.
The normal returned-Session and late receipt-sourced binder read exactly this frame;
no later lock read/current-version refresh or #19 bookkeeping refresh replaces it.
Marker/owner provenance is actual private operation evidence; DTO labels/JSON cannot
construct it.
Carry this sealed nonserializable frame in genuine `ManagedPhaseLaunch` to EVERY
provider entry and to first preparation, first/new consumption and AllowCurrent.
Start-entry ScopeSnapshot must equal the marker frame before any native effect or
PreparedInput bytes. The original frame supplies expected P/G/T/full locks throughout
the operation, never a fresh start capture. Pre-effect/pre-consumption drift can use
only genuine pristine/NoCurrentDispatch refusal with full owned cleanup and #19's
non-success closure: current rows supply closure-write CAS only, preserve current
lifecycle/decision and immutable original provenance, never renew a grant. That may
retire the exact Interrupted/Failed phase under #23 policy; any fresh retry is a new
independent admission. Consumed or uncertain operations remain held for actual #14. Scope, context and
source remain immutable. A strict mismatch in either binder, including sibling Goal
metadata movement, holds without automatic fail/retry/release; the separately proven
pre-input NoCurrentDispatch closure above is not a binder authority refresh. This availability limit does not waive
native multi-Task acceptance. No unrelated-parent tolerance is introduced.

The sole binder takes a private binding request variant, NormalReturned or
ClosedSettlement; the caller cannot select weaker predicates. NormalReturned uses
captured registered identity and actual #19 current allocated/preparation/admitted
input evidence (Starting need not have ACK). ClosedSettlement constructs sealed
`ClosedSettlementBinding` INSIDE its Immediate transaction, from the exact private
current operation, actual allocated/admitted/consumed actor/input, genuine successful
KnownCurrentTerminal receipt, exact current Session body/version checksum and full
profile cleanup. Require current Exited success; Lost, unknown cleanup/outcome,
restored prior terminal, NoSession/NoCurrentDispatch never binds. Both variants apply
all original frame/current activity, full locks, context, immutable Session identity
and latest native UUID uniqueness predicates; no public receipt IDs manufacture
native evidence. Both write only absent active Session ID, ordinary Record version/
updated_at and ONE reserved factual audit. The single exact4KiB binding audit field list is Design section11; mechanics use
those fields, including normal_return/closed_settlement and exact operation/receipt
references (receipt absent for normal return). No native output/error/prompt or outcome claim.
Binding proves identity only; #19 current-success closure is independently necessary.

If a delayed normal return races late binding, exactly one succeeds/audits. A loser
writes nothing and keeps original actual owner/claim/receipt; AlreadyBound may only
report a SAME private bound operation fact, never infer it from a matching public ID.
AlreadyBound for that SAME genuine private operation returns Started to a losing
normal invocation. No retry, fail, release or native dispatch follows a lost race.

## Design9: managed launch endings and durable readiness

Introduce private Engine `reconcile_settled_native_binding` called ONLY by the
actual #23 authoritative active Task driver and that managed invocation's post-marker
start-Err path. Passive #41 step/poll/status observers never call it. Managed invocation
ownership lives in actual runtime launch supervision independently of droppable Engine
futures; caller abort/drop is an observed ending, not the sole destructor callback.
Readiness is a DERIVED level predicate over actual durable #19 facts: a dispatch-
marked operation, phase_open and absent active Session ID is pending; a genuine
current receipt makes it a candidate for proof. There is no separate persistent
ready/retirement row or notification-authority flag. Normal/late binding removes the
unbound predicate; every real NoCurrentDispatch/failure/TerminalRecovery phase closure
removes phase_open. A retry uses a new operation. Receipt publication commits in the
real settlement transaction; duplicate/full/lost notification is only a wake hint.
Binder re-derives sealed proof under Immediate and preserves its exact existing write
set. Derived readiness does not add a binder, diagnostic or closure write.

Capacity is DERIVED from durable native claims, not a separate pre-claim token or
row. Inside the SAME Immediate #41 reserve transaction, count the Project's complete
active native Executor/Reviewer attempts (marked OR unmarked) plus genuine phase_open
#19 operations whose claim is no longer active. Deduplicate by exact Project/Task/
Workflow/generation/attempt/context identity. Include ALL retained states/instances;
ordinary terminal/cancel labels cannot omit a still-open operation. Proposed4096
ordinary claims per Project is finite policy, separate from actual #23 resource
permits and reserved recovery headroom. Admit only if the complete count plus the
new claim fits, and atomically commit that Workflow reservation before effects.
Source supplies a complete indexed read-only SQL view over validated Workflow active
claims UNION private open operations; no truncated history scan or persisted capacity
row. Separate Store connections/runtimes see the same count under Immediate. Missing
view/integrity or unsupported producer refuses before claim commit/native effects.

The marker consumes this already-counted exact durable claim; no new capacity slot or
quota race arises there. Validate the original claim and inclusion, never re-check a
new unreserved quota after it. #41 owned release, preparation invalidation and genuine
pristine pre-marker closure remove their active claim naturally. A marked operation
remains counted until genuine #19 phase closure, even after binding; cancellation with
active=None still counts any phase_open operation. Crash/drop/release-CAS failure keeps
the orphan claim counted until actual14 recovery. This defines EVERY pre-marker exit
without adding writes to41release/binder/diagnostic or a generic marker-error release
exception. Closed historical records consume no slot. Per-Project bounds/fair23permits
retain B progress while A is held; actual physical exclusion rules still apply.
These source/view/41/19/23 integrations are mandatory, not an implemented producer.

Driver registers its event wait then rechecks derived state with read-only bounded
queries, never an Immediate transaction just to poll. At most64 eligible members per
fair round-robin pass; proposed100ms–5s adaptive fallback, unchanged native cleanup
deadlines. Only a DefinitiveRefusal parks an in-memory non-authoritative Held hint keyed
by observed P/G/T/W/full-lock versions, lifecycle, operation/receipt and #14 proof
identity. Unchanged Held entries issue ZERO repeated Immediate binder transactions
across timer/notification/evaluation wakes. Relevant changes invalidate the hint and
recheck actual proof; restart performs a bounded cold read then parks unsupported
entries. Private outcomes distinguish Bound, AlreadyBound(same operation), DefinitiveRefusal,
Transient and CommitUncertain. Definitive means an actually read frame/version/
lifecycle/identity/private-proof mismatch. Deterministic encoding/size bounds,
SQLITE_CONSTRAINT (including trigger RAISE(ABORT)), schema mismatch and writer-fence
refusals are DefinitiveRefusal/integrity attention, parked without unchanged repeated
Immediate attempts. Only SQLITE_BUSY/LOCKED or classified transient IOERR with confirmed
complete rollback is Transient: do not park it under an unchanged predicate key.
A record/audit INSERT failure is not automatically transient. Retry the SAME private binder/frame on later fair100ms–5s timer wakes,
without owner refresh, native redispatch or phase failure. Persistent storage trouble
reports attention with bounded backoff; it never converts to a terminal outcome.
CommitUncertain performs bounded read-only reconciliation of exact Workflow/operation/
audit facts before deciding Bound/AlreadyBound or confirmed unchanged rollback/retry;
unknown never assumes either commit or rollback and cannot publish/retry native work.
These typed outcomes apply to BOTH NormalReturned and ClosedSettlement. On the normal
route the owning invocation retains the exact returned identity/frame and reports
StartedBindingDeferred, never Failed, while the native turn continues. Fair same-frame
storage retry does not renew admission. If the original return delivery is lost, only
a later genuine current successful receipt/full cleanup may qualify ClosedSettlement;
unknown/current failure retains its separately reviewed closure/hold route. Definitive
refusal does not redispatch. CommitUncertain reconciles actual exact durable facts and
never equates an error with rollback. This remains owned during caller Drop.
Hints grant nothing and persistence is unnecessary. Per-Project fair cursors
prevent parked A entries consuming all B work. The genuine runtime driver and wait-
register/recheck sequence recover lost notifications without a busy loop; numbers are
finite policy bounds, not OS real-time guarantees or implemented-source claims.

BOTH driver candidate predicate AND binder transaction compare invoking genuine
runtime-instance ownership to operation/receipt instance. A different instance requires
sealed authentic #14 fencing/restore proof before registration; otherwise Held with
zero writes and no repeated Immediate retries. Persisted UUID/receipt/readiness alone
cannot mint that proof. Same-instance matching is necessary, not a cleanup substitute.

Post-marker outcomes: genuine current successful receipt -> binder route; pending/
unknown -> held under existing real supervisor; NoCurrentDispatch -> its #19 proven
non-success closure only; genuine current failure with complete required settlement
-> separately reviewed #19 non-success closure WITHOUT absent Session-ID binding.
Missing start return is never a native failure. Explicit abort/cancel/lifecycle revocation
can make current activity fail; then keep held ownership rather than binding stale
success. Restart requires authentic durable frame/current-success evidence AND actual
#14 fencing/restore before this route, never row-based native owner reconstruction.
Actual #19/#23/#43/#14 producer ancestry co-integration is the merge vehicle; no
completed-merge cycle or SQL-seeded/direct-binder-only acceptance.

## Design9: separate record-only bound-live diagnostics

Current d87faec bound `poll` writes Task through refresh_owners + ordinary persist on
adapter.status error and validate_persisted_status mismatch. Replace ONLY those arms
with private `observe_native_diagnostic`, distinct from binding. Transaction checks
exact Workflow Record version/generation/active Running bound attempt and genuine
private operation/context/input/actor/Session identity. Use original expected private
identity, not parent authority recapture. Return latest committed Workflow Record only.

Allowed delta is solely active.detail -> allowlisted reason status_unavailable or
persisted_status_mismatch with bounded fact fields (total128UTF-8 bytes), plus checked
Wversion/updated_at and one bounded reserved rrx.private.workflow.native_diagnostic event. Audit
maximum4KiB; both public audit APIs refuse this kind. Enumerate both exact compiled
private constants in reserved_audit_kind under the composed rrx.private.* prefix. Compare complete previous/candidate
JSON and Record metadata, rejecting every extra field including unknown fields.
Identical diagnostic is true no-write/no-audit; no infinite repeat history list.
No Task/P/G/Session/full-lock body/version write, private input/owner/admission pin
change, marker/Session ID/context/attempt outcome or grant. Status mismatch is simply
unverified observation, not native terminal or failure. No new Source/native/FS call
under Store. Stale Workflow or missing exact private proof holds/refuses unchanged.

An accepted bound-live diagnostic is a Workflow factual revision only. It never
updates the immutable marker frame or native currency. #19/#23 terminal/PhaseGates
consumers use exact current Workflow Record CAS PLUS the sealed factual Workflow
successor proof below, unchanged private native pins and genuine receipts. This bounded
factual ledger is essential to distinguish legitimate W revisions from drift; it never
updates the marker frame or renews a native grant. They still check actual current source/owners and settlement/outcomes, cannot
renew admission authority or discard existing evidence. Diagnostic replay/races do not bind
or close anything. Inventory every actual Workflow persist/refresh_owners call at the
source revision: marker pre-admission; binding sole private ID writer; these diagnostic
arms record-only; unbound poll passive; genuine terminal/evaluation transitions normal
with private settlement; authorized cancel/fail/lifecycle revoke authority and retain
ownership; other Source/retry/recovery transitions retain separate reviewed gates.
Unclassified live bookkeeping cannot claim native-currency preservation.

## Design9: extended actual consumer controls and mutations

Keep all prior concrete provider/ordinary-access/private publication matrices.
Add genuine managed controls: normal and late paths use exact marker frame/full locks;
actual success before dropping pending start; BOTH Err and drop endings still pending,
later genuine settlement and lost/full/closed notification converge once WITHOUT
restart/fail/retry/second dispatch. Genuine two-binder race audits once. Restart positive
uses actual #14 proof, not public receipt seed. Negative actual former terminal,
Unknown/Lost/non-dispatch/current failure, refreshed parent/lock expectation and stale
original frame all hold without binding. Known current failure independently uses
#19 non-success closure and never becomes success or absent-ID binding.

Kill fresh-recaptured-frame, notification-only, missing real driver lookup, fail-retry,
and missing derived-readiness/wake mutants. The latter keeps normal evaluation/
launch-ending rederivation and notification wakes, so late settlement AFTER a pending
Err/drop plus lost notification must expose stranding, not a weaker mutant that drops
all evaluation. Timer/wait registration/recheck controls must exercise real #23 driver.

For EACH bound-live diagnostic arm hold a genuine native turn and force actual status
error or actual persisted/watch mismatch. Poll truthfully waits with preserved P/G/T,
all Session/lock bodies/versions and native pins; once status converges the exact owned
callback/decision remains eligible without redispatch. Repeated reason has no writes;
stale Workflow/op CAS, unauthorized detail/outcome/ID extra delta and audit failure
leave every row unchanged. Mutate ordinary Task-writing persist in EACH branch,
owner/attempt CAS omission, extra-state/ID allowance and mismatch->fail/retry. Preserve
normal terminal and authorized revocation negatives, restore exact passing source.
Actual default native5/6/7 conformance, required restart and co-integrated producer
controls remain mandatory; synthetic callback/SQL/direct binder alone does not count.
No implementation/native/profile acceptance is claimed by these design documents.


## Design9: additional genuine consumer controls

Run more than4096 sequential normal dispatch/bind/closure cycles without history slot
leak; Held Project A does not block B. Two-connection/cross-runtime claim races are atomic inside41reserve: each result
remains marker-admissible or refuses before commit with zero effects/no stranded claim. Across N unchanged Held wakes observe zero
repeated Immediate binder transactions, while a relevant real receipt/recovery change
causes one recheck. Mutate readiness derivation, capacity derivation/claim transaction ordering,
and Held parking at the actual driver, with restored controls.

Immediately after the real marker, normal binding passes using checked resulting
Task/W versions. A pre-marker-frame mutant fails this positive consumer. Change EACH
P/G/T and a lock-only row between marker and adapter entry: zero PreparedInput/effect
and genuine closable NoCurrentDispatch. A fresh-capture mutant must reach the actual
consumer and fail that zero-effect/closure expectation; post-consumption negatives
stay held without authority refresh. Persist genuine ready/receipt state, restart a
different actual runtime instance: zero binding until genuine14proof. Omit the driver
and binder runtime-instance checks independently and kill each at its real consumer.

Both public audit APIs refuse EACH reserved private name; Workflow saved/bound count
consumers use the exact private binding constant. Same-operation late bind then normal
return yields Started, exactly one audit and no retry/fail/release. Diagnostics retain
the full original frame and current-Record CAS plus the bounded genuine factual
successor ledger, never current-row self-adoption or refreshed native authority.
All controls require actual co-integrated19/23/43/14 producers; isolated SQL/direct
binder tests and this proposed design do not qualify production/native acceptance.


## Design9: remaining liveness and perturbation controls

More than4096 sequential genuine pre-marker preparation errors with41release AND
source invalidations must leave a later native dispatch admitted. Real crashes and
release-CAS refusals stay counted; bound phase-open and canceled-but-open operations
remain counted once. Independent connections/runtimes cannot over-admit. Kill a mutant
which counts only marked operations or omits pre-marker active claims; exact restored
consumer must pass. Capacity is derived, so there is no synthetic retirement-row test.

After genuine late success, inject one actual SQLITE_BUSY/LOCKED or classified
transient IOERR with confirmed full rollback: same versions/frame remain, then a real timer wake binds exactly once without
restart, authority refresh or redispatch. A Transient-parking mutant must strand this
actual-driver control. Definitive held controls still observe ZERO unchanged repeated
Immediate binder transactions. Unknown-commit controls cannot infer success/retry.

Normal production lock writers refuse while a genuine operation is open. For an
independent lock-only CAS negative, the isolated fixture's second epoch-enabled SQLite
connection uses its controlled test-only external-writer injection to mutate exactly
one lock row with P/G/T unchanged. No production generic lock bypass is added. This
is corruption/race injection, never private producer or positive cleanup evidence;
the authentic operation/binder/native consumer supplies all positive prerequisites.
Kill lock-check omission at that consumer; public SQL seeding is still not an owner.


## Design9: bounded factual Workflow successor proof

Co-integrate the proposed #19 Design33 contract at public cac5618. That dependency
is itself under review, not implemented or source approved. The immutable marker
anchors complete canonical Workflow body SHA256, planned post-marker W version,
full P/G/T/context/source/actor/lock frame and operation identity. An unbound first
binder compares that exact original W/body. Binding legitimately increments W;
bound diagnostics and subsequent #19 phase closure must distinguish those genuine
factual revisions from arbitrary current-row writes. Original P/G/T/full locks,
context/source/input/actor/native-grant pins remain mandatory and NEVER refresh.

The existing ONE reserved audit in the SAME binding/diagnostic Immediate transaction
is also an append-only factual ledger link: exact scope/operation/attempt, original
marker-frame digest, predecessor/successor W versions, COMPLETE predecessor/successor
Workflow body SHA256 and prior ledger-link digest. First binding starts at the exact
immutable marker W/hash. Checked consecutive W versions and the complete allowed
projection are verified before writing; then re-read stored W/body/hash before commit.
Binding changes only absent active.session_id to the variant-specific exact private
allocated identity; diagnostic changes only the allowlisted bounded detail. No extra
operation/table/count-row update, Task/Session write or default workflow.saved audit.

Canonical body encoding is a versioned rrx.workflow-body-sha256/v1 recipe: recursively
lexicographic UTF-8 object keys, original array order, compact serde_json string/number
encoding of the COMPLETE stored Workflow data, including every field. SHA256 domain
bytes are b"rrx.workflow-body-sha256/v1\0" followed by that encoded body. A private
link digest uses b"rrx.workflow-ledger-sha256/v1\0" followed by the same canonical
encoding of the complete immutable private link payload (it does not contain its
own digest). First prior_ledger_digest is the exact immutable marker anchor hash.
Unknown or
unsupported fields reject; no projection/ignored-field hashing. The encoder, exact
hash domain bytes and typed stored-body round trip need golden fixtures shared by
marker, binding, diagnostics, gate links and closure. Record version/updated_at are
checked separately as the prescribed metadata delta. Never hash caller-claimed values.

Generated/indexed audit operation/frame/predecessor/successor keys plus uniqueness
constraints identify exact consecutive genuine private links. Store alone derives the
sealed CurrentWorkflowSuccessor from marker through latest link and exact current
W/body. A reserved string/public audit/Session receipt/current-row self-match cannot
mint it. Both public audit APIs refuse every compiled reserved private kind. Derivation
is bounded to256 compact links per operation,4096 UTF-8 encoded bytes each,1 MiB total,
charged to #19's existing128-MiB Workflow quota. Reserve the explicit finite allowances below before effects; optional diagnostic
budget exhaustion coalesces/no-writes
with attention and cannot consume mandatory closure capacity or strand closure.

#19/#8 managed gate-claim and outcome observation are separate record-only consumers,
with rrx.private.workflow.gate_claim and rrx.private.workflow.gate_observed links.
They may not bump Task before final atomic native phase closure. Those ports require
their own genuine current gate/result authority; effectful gates also require actual
#12/#60 owned jobs before effects. Pure evaluators require genuine typed input/results,
not an invented external job. They preserve original grant/source/lock currency and
the exact W factual chain. A native receipt alone cannot certify ReviewPassed or
TestPassed. These are dependency integrations, not extra #43 binder/diagnostic writes.

Required actual producer controls: genuine bind→diagnostic→successful gate/phase
closure and genuine bind→diagnostic→current failure/explicit-retry hold; exhausted
optional diagnostics still permit mandatory closure; rollback leaves zero link/write.
Foreign W version/body drift, missing/different link, ordinary forged audit, altered
marker anchor, illegal bind projection and cross-operation replay hold without new
input. Kill omitted chain/hash/current-self-match mutants at real closure, then exact
restored controls. No private SQL seeding or direct binder-only fixture qualifies.

NormalReturned BUSY control must keep one actual native turn current, return typed
StartedBindingDeferred and later bind once with zero Failed/redispatch/owner refresh.
NormalReturned CommitUncertain must reconcile only exact durable bind/link facts;
inconsistent/unknown results retain ownership. Deterministic trigger ABORT, over-bound
encoding and writer-fence controls park with zero repeated unchanged Immediate calls;
classifying a constraint as Transient must fail that consumer. Existing late-success
storage retry controls remain mandatory. These contracts require independent review,
actual #19/#23/#41/#43/#14 integration and native/source gates; none exists in main.


## Design9: complete open-phase Workflow writer classification

Verified current workflow.rs terminate(1348–1372) sets terminal_decision and rewrites
Task/W through TerminalDecision, retaining active. release_terminal_reservation
(1392–1421) later performs TerminalRecovery. These are authorized existing consumers,
not foreign drift and not #43 binding. Gate hold(1630–1654), fail(1658–1669) and
evaluate's post-observation Waiting/irreversible-failure persistence are additional
open-phase writers. Actual source acceptance inventories EVERY persist/refresh_owners/
put_workflow_transition/put_record_tx call; unknown open-phase writers cannot ship.

| Open managed phase writer | Private producer and exact bounded Workflow projection |
| --- | --- |
| Marker, before native effects | #19 marker commit establishes original full frame/body; checked planned Task/W+1 |
| First factual binding | #43 session_bound: ONLY exact absent SessionID; no parent/operation write |
| Bound live diagnostic | #43 native_diagnostic: ONLY allowlisted active.detail; no parent/operation write |
| Gate claim | #19/#8 gate_claim: ONLY Evaluating and exact claimed_observations; genuine gate invocation authority |
| Gate result / Waiting | #19/#8 gate_observed: fuse actual compact claimed outcome + its genuine typed disposition in ONE result transaction. Waiting/known held failure may set exact active Waiting/heldFailed, bounded detail/held_reason; no later ordinary Task/W persist. Passed leaves final success/Task/context changes to genuine atomic phase closure |
| Gate hold / clear | #19/#8 gate_hold: ONLY exact active bounded detail/held_reason under actual typed gate policy; same-marker/body successor, no native grant/source/parent refresh |
| Authorized cancel/fail decision | #19/#8 terminal_decision: ONLY previously absent exact terminal_decision in W, plus separately authorized checked Task lifecycle delta in SAME transaction; no SessionID/outcome/claim removal |
| Native/phase closure, including TerminalRecovery | #19 phase_closed: actual eligible settled receipt/current typed gate result or separately authorized terminal decision, complete genuine factual prefix, exact final allowed phase/body/Task projection and actual required jobs settled |
| Escalation/source invalidation/retry/new context | Under #19 own closed-operation rules after actual owned stop→receipt→closure, not ordinary writes into an open phase; #41 pristine pre-marker policy remains separate |

The compiled reserved kinds additionally include rrx.private.workflow.gate_hold,
rrx.private.workflow.terminal_decision and rrx.private.workflow.phase_closed. All are
dependency-owned ports, not extra #43 binder/diagnostic scope or writes. The managed
Workflow Waiting/held disposition informs actual #15/#23 status/readiness/scheduling;
it MUST NOT be implemented as a hidden Task WaitingHuman/version rewrite before phase
closure. Actual consumers must use this derived factual status while preserving Task
authority. Transport success, public status or ordinary JSON cannot mint a typed gate
result, lifecycle action or ledger link. Each producer validates its complete exact
allowed projection before emitting ONE private audit; it replaces generic workflow.saved
in that transaction rather than adding a second default audit through put_record_tx.

TerminalDecision checks the genuine prefix/full body BEFORE appending, for BOTH bound
and unbound attempts. Its actual authorized controller/lifecycle action, current Task
before/after body/version and immutable original operation identity are validated
separately. Cancellation is not a fresh native grant and does not refresh original
P/G/T/source/locks. Its Task lifecycle revocation is preserved; once decision commits,
no new binder/diagnostic/gate admission is permitted. After actual complete eligible
settlement, the separate nongrant TerminalRecovery consumes this genuine decision and
the exact full successor chain, leaves SessionID None when unbound and closes once.
It retains the separately reviewed inactive-owner closure scope, not the active-only
binding predicate. A current Task terminal label cannot create this decision proof.

Foreign W drift followed by cancel may NOT launder a predecessor into a valid chain.
Actual controller revocation/stop can retain its separate legitimate Task/Goal/Project
authority while refusing unprovable Workflow bookkeeping and holding the operation for
#14. TerminalDecision/TerminalRecovery cannot accept a current-row-only match, forgive
missing links or grant native inputs under changed owners. Explicit lifecycle revoke
must remain available through its trusted existing port; a broken W ledger is never a
reason to permit fresh input. This is not an exception to native admission currency.

## Design9: finite ledger allowances and compact transactional proof

The per-operation256-link policy is explicit: up to99 genuine gate cycles×at most TWO
links (claim + fused outcome/disposition)=198, binding1, terminal_decision1,
phase_closed1, gate_hold/clear8, optional native_diagnostic47. Total256; reserve those
exact per-class allowances before marker/effects under the existing Workflow quota.
Each actual cycle reserves its pair BEFORE a gate effect. The100th refuses before
claim/effect with bounded attention; no Failed, native/phase redispatch or budget reset.
An admitted99th Passed result can always commit and close using reserved headroom.
Cancel and full TerminalRecovery stay available at exhaustion. Gate hold exhaustion
coalesces/no-writes with attention, never consumes cancellation/closure slots. Duplicate
facts no-write. More than64 identical Waiting outcomes followed by Passed fits this
allowance and retains exact coalesced repeat count; coalescing does not erase ledger
spend or native call count. Unknown effect stays retained; finite policy is no timing
guarantee and does not authorize a Human-only policy change.

Derive/verify the COMPLETE ≤256-link,≤1-MiB append-only chain in a coherent read
snapshot OUTSIDE held SharedStore/write transaction into a Store-only nonserializable
sealed factual proof. In the Immediate transaction compare exact immutable marker,
operation/attempt/frame, current W version/COMPLETE body hash and exact latest genuine
link ID/sequence/digest. Generated indexed audit keys, uniqueness and reserved append-
only UPDATE/DELETE refusal preserve the verified immutable prefix; no arbitrary audit
tail/current row self-match suffices. Concurrent W/link/head mutation makes publication
CAS refuse. The proof cannot be minted by a caller hash or receipt JSON. No new binder
operation/count-row write. Actual compact transaction checks remain bounded to the
shared #19 ≤128 compact rows/≤512 KiB plus declared scalar guards; do not replay full
Workflow/native-result/ledger histories while holding SQLite write lock. Full current
Workflow body hashing/validation uses the genuine version-pinned Store extraction and
exact publication CAS, not an untrusted claimed hash. Co-integration must implement
and test the real private proof producer and DB immutability guards, not SQL seeding.

## Design9: exact diagnostic link fields and new consumer controls

The native_diagnostic payload is EXACTLY: kind, project_id/goal_id/task_id,
workflow_id, generation, attempt_index, phase, session_id, private_operation_ref,
original_marker_frame_sha256, workflow_version_before/after,
workflow_body_sha256_before/after, prior_ledger_digest, canonical_body_recipe,
reason(status_unavailable|persisted_status_mismatch), detail(≤128 UTF-8 bytes),
context_version and timestamp(existing AuditEvent.at). Whole encoded event ≤4096
bytes. No provider payload, process list, argv/environment, grants or other fields.
Terminal/gate/closure links use the same genuine canonical predecessor/successor
header and their EXACT typed allowed projection; each dependency port must enumerate
its final schema and compiled reserved constant in the joint source review.

Required actual controls: bound AND unbound cancel/fail→genuine eligible cleanup→
TerminalRecovery closes once, preserving Task decision and absent ID; foreign W drift
then cancel remains held while actual revocation/stop still works. Omit decision link,
accept forged ordinary terminal audit, bypass closure chain and gate-hold Task-rewrite
mutants independently fail at these genuine consumers. At99-cycle boundary an admitted
Passed closes;100th refuses before call/claim. More than64 Waiting then Passed succeeds;
diagnostic/hold exhaustion still permits cancel/full settled closure. Concurrent link/
body drift between out-of-lock proof extraction and publication refuses unchanged;
UPDATE/DELETE of reserved links and forged proof constructor fail. Restore exact
source controls. All #19/#23/#41/#43/#14/#8/#9/#12/#15/#60 producers, native/recovery
conformance, exact tested source CI and independent source review remain unimplemented
mandatory gates. This Design9 proposal cannot qualify them.
