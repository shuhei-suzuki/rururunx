# Issue 43 design: record-only native Session binding

Status: proposed Design2; independent STRICT delta review pending. No implementation.
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

Before committing the dispatch marker, resolve the adapter from the existing
immutable AgentRegistry and call its existing `probe` outside the Store mutex.
Require probe.agent to equal the selected registered name, a nonempty provider,
and the required role capability. Keep existing capability checks, executable
validation and native defaults. Probe failure uses the pre-launch failure path;
it does not launch a model, override authentication or infer authentication from
an unobservable value. Registry registration currently validates name uniqueness,
not probe identity; the new check must therefore be explicit.

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

Implement and review the narrow record-only path in isolation, then compose the
actual #19 predicate before claiming typed-native acceptance or merging this
issue. Use a co-integration branch preserving both source commit ancestries for
the #19 predicate and #43 binding port. Run their approved combined source review,
native integration and exact-head CI before the ready PR(s) merge. This avoids
making either completed merge a prerequisite for the other's acceptance. Neither
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

## Verification and meaningful mutations

First test Workflow's actual successful-start consumer with a protocol-controlled
registered adapter that persists a real Session. Assert P/G/T and every scoped
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
bodies/versions. Immediately before/after binding compare the same ReadOnly or
Mutating reserve attempt: its existing refusal/StateGuardError must stay exact.
No sibling gains currency/completion, and binding returns only the committed
Workflow record, no value accepted as an admission credential. Actual validators
still require their private owner/pair. Kill a reserve-sweep-in-binding mutant and
a bound-ID-as-admission-proof mutant at those causal consumers.

Add ordinary-transition controls for every WorkflowAccess rejecting an existing
None-to-Some Session ID and new-history injected IDs. A caller omitting allocation
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
