# Issue 43 design: record-only native Session binding

Status: proposed Design4; joint private-publication alignment and independent
STRICT delta review pending. No implementation.
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

Introduce a private binding transaction for that single successful-start path.
Keep the dispatch-marker reservation and every other phase, context, risk,
observation and terminal transition on the ordinary transaction. This does not
make an unchanged Task write generally optional. No new native operation or
permission is admitted by binding, and transport exit never becomes gate evidence.

The dedicated transaction is the sole writer of an existing attempt's Session ID.
Every ordinary `put_workflow_transition` access mode and its `validate_transition`
reject any existing history entry's session_id change. Newly appended attempts
still require session_id=None. Keep closure guards on the ordinary path; move
binding validation to the narrow path. A future recovery binder needs its own
reviewed private port. Current production has one assignment (workflow.rs:1016)
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

After the dispatch-marker commit, capture every scoped WorktreeLock ID/version
under Store before calling start. This is the same complete set used by native
scope consumers, including inactive historical lock records. Retain it in the
private binding expectation; no lock is created or adopted by this capture.

After successful start, create the candidate Workflow data by setting only the
captured active attempt's absent Session ID. Call private
`Store::bind_workflow_session` with the captured Project/Goal/Task snapshots,
the expected Workflow record and candidate, the captured lock set, and this
binding identity. Do not
refresh owners, recapture a newer Task version or retry this call on CAS failure:
that would silently accept revocation during native start. On success update only
the caller's Workflow record to the committed version and return Started. Keep
the caller's Task byte-for-byte at its pre-start snapshot/version.

On any binding failure propagate an explicit error. Leave the already committed
unbound dispatch claim and any durable Session intact. Do not call `fail`, retry
start, stop/adopt a Session, release the claim, or publish completion in that error
path. Observer polling remains passive under #41; #13/#14 own reconciliation.

## One Immediate transaction

The method is crate-private and performs no await, filesystem access, child launch
or native wire operation while holding Store. SQLite Immediate excludes other
writers across connections. All checks and the one Workflow/audit write use its
Transaction; a failed check, write or audit rolls back everything. Update the
caller's mutable Record only after successful commit.

1. Read captured Project, Goal and Task by their exact IDs. Validate their owning
   edges, exact versions, Registered Project, active Goal and nonterminal Task.
   Compare the full Task serialization with the captured Task; candidate binding
   cannot carry an in-memory Task change. No Project/Goal/Task write follows.
2. Read the existing Workflow by ID and require exact kind, Scope, version and
   unchanged Record identity/timestamps. Require exactly one Workflow in the Task
   scope. Decode with existing Workflow validation, check Task workflow/risk,
   revision and context pointer, active index bounds, current generation, a native
   Executor/Reviewer phase, Running state, absent Session ID, committed
   dispatch_started, expected agent, and no completion or finished authority.
3. Clone the complete previous JSON data and replace exactly
   `history[active].session_id` with the returned ID. Require this Value to equal
   the supplied candidate data, including every other known or unknown field.
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
they do not scan other Projects. This design adds no process inspection deadline
or native execution timeout and no schema solely for record-only binding.

## Persistence and audit implementation boundary

Reuse `put_record_tx`'s record CAS/version validation, immutable scope/kind,
worktree exclusion and serialization path. Introduce a private typed audit choice
only if needed to supply the binding fact: the existing helper delegates to the
same internal record writer with its current default audit, while this one caller
selects `workflow.session_bound`. It replaces the default workflow.saved audit
for this call, rather than adding a second event. Preserve every other caller's
existing event name/payload and write behavior.

Reserve the exact `workflow.session_bound` event name in `reserved_audit_kind`:
public `audit` and `audit_if_current` must refuse it. Primary inspection found
the current reserved list includes .saved and workflow.gate_observed, but would
allow this new name unless extended. Test both APIs and kill the omitted-guard
mutant; public caller facts cannot impersonate the private binding journal.

The factual binding event records owning Scope, Workflow ID/new version,
generation, active attempt index/phase, context version, returned Session ID,
agent/provider/role, captured/new Workflow versions, preserved Task version,
dispatch_started=true and the versioned marker tuple. Timestamp is AuditEvent.at.
It contains no prompt, environment value, config, permission,
native response, native outcome or assertion of delivery. The complete Workflow
delta is durably present in its own record. The private audit choice is not a
public arbitrary-event API and cannot substitute for required Store guards.

## Integration with typed ownership and follow-ups

Issue19's schema6 source is not yet implemented. Its reviewed design allocates
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
| Codex 6749505 answer_approval :2020–2062 | The actual code does not overwrite dispatch_intent. Its separate codex.approval.reply_intent audit precedes reply: Approve uses audit_if_current then put_session_if_current/NativeCAS; Decline uses audit then unchanged own-Session put_session/historical observation. Post-reply removal of the native in-memory pending request and Running publication use the Acked historical observation outcome with the consumed pair intact. Preserve existing audit/wire ordering; do not invent a new consumed intent. |
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
activity/worktree constraints; historical does not override ProjectBlocked or an
own-Session/write failure. Existing denial then stops without a reply wire.
ALLOW's separate full current NativeCAS and actual owned pending-request proof
remain mandatory. Parent Task metadata/version revocation alone cannot turn
DENY into an operation grant or require ALLOW's parent-version fence.

Inventory every consumer of the renamed operation/transport journal: Claude
permission assertions, reserved intent audit payloads, SQL ALLOW-failure trigger
(current :3005) and control/mutation readers. Preserve event names/order and
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
