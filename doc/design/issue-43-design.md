# Issue 43 Design: Preserve native authority during Workflow Session binding

**Status:** Design8 proposed from Requirements9 approved at91d4f34; independent
design/source reviews and actual producer/native/recovery gates pending
**Workflow:** STRICT  
**Scope:** factual native binding, managed settled-success route and two bounded live diagnostics

## 1. Goal

Preserve native adapter authority when Workflow records the Session returned by a successful native launch.

The current failure mode is self-inflicted currency drift:

```
Workflow reserves attempt
  ↓
native adapter captures Project / Goal / Task / lock currency
  ↓
adapter returns Session
  ↓
Workflow binds session_id
  ↓
generic Workflow persistence rewrites unchanged Task
  ↓
Task.version increments
  ↓
still-running native turn sees stale Task currency
  ↓
valid callback / completion / approval becomes ineligible
```

Binding a factual Session identity must not mutate Task authority merely because the runtime registered its own already-launched Session.

## 2. Design principles

1. Session binding is factual registration, not new operation admission.
2. Task, Project, Goal, Session and WorktreeLock authority remain unchanged by a valid binding.
3. Only the Workflow record and one scoped audit entry may be written.
4. Binding must be exact-scope, exact-generation and exact-attempt.
5. A public Session ID is not a native ownership credential.
6. Native provider authority remains provider-specific and unchanged.
7. Binding failure never releases, retries, transfers or fabricates completion.
8. No independent binding-only migration; compose the mandatory #19/#23/#43/#58/#60 writer epoch.

## 3. New private Store primitive

Introduce one crate-private transaction dedicated to Workflow native binding.

Logical shape:

```text
bind_workflow_native_session(
    operation: PrivateManagedOperationIdentity,
    variant: PrivateBindingRequest, // NormalReturned(identity) or ClosedSettlement
) -> BindingResult
```

The exact Rust names may differ, but the primitive must not be exposed as a general state mutation API.

### 3.1 Expected snapshot

The marker transaction privately captures resulting post-marker authority atomically before launch (see Design7 mechanics):

- project_id + expected Project version
- goal_id + expected Goal version
- task_id + expected Task version
- workflow record ID + expected Workflow version
- workflow generation
- active phase / attempt identity
- expected actor/provider/role
- exact Task worktree identity
- dispatch marker identity
- context pointer/version identity where applicable
- complete sorted scoped lock IDs/versions/digest and actual private operation owner

The returned registered Session identity is a post-start input, never marker-time
knowledge. The Store-derived immutable frame is the only expectation; any caller
snapshot is merely a cross-check and cannot substitute newer authority.

Every fresh Workflow Executor/Reviewer binding must obtain #19's actual private
allocation/input-pair proof inside the transaction. There is no untyped fresh
binding path; historical untyped records remain readable history. Missing or
inconsistent private proof rejects regardless of caller expectations. Availability
of the compiled #19 port is an integration gate, never a runtime bypass.

### 3.2 Allowed write set

The transaction may change only:

```
Workflow.history[active_attempt].session_id:
    None → exact NormalReturned identity or private ClosedSettlement allocated SessionId

Workflow.version:
    n → n + 1

Audit:
    append one factual rrx.private.workflow.session_bound event
```

It must not change:

- Task body or Task.version
- Project body or Project.version
- Goal body or Goal.version
- any Session body or Session.version
- any WorktreeLock body or version
- context pointer/body/version
- workflow generation
- attempt actor/role/provider/worktree
- dispatch marker
- evidence
- terminal state
- review/approval state
- completion/outcome fields

Any requested delta outside this allowlist fails closed.

## 4. Transaction contract

Use one SQLite Immediate transaction.

Inside the same transaction:

1. reload Project, Goal, Task and Workflow
2. verify exact expected versions
3. verify Project/Goal/Task are still active and eligible
4. verify exact Workflow scope/generation/active attempt
5. verify active attempt is Running and has no bound Session
6. verify committed dispatch marker is the expected marker
7. verify context pointer/identity remains current
8. load the exact Session from NormalReturned identity or private ClosedSettlement allocation
9. validate immutable Session identity
10. validate native identity uniqueness within the owning scope
11. derive and validate mandatory private prepared-input allocation/input-pair owner
12. apply only the allowed Workflow delta
13. append one factual audit entry
14. commit

A failure at any check rolls back with no Workflow or audit change.

## 5. Session identity validation

The persisted Session is authoritative for registration facts.

Require exact equality for immutable identity:

- SessionId
- Project / Goal / Task scope
- provider
- actor
- role
- worktree / repository ownership
- Store-derived private phase allocation and current prepared/admitted input-pair identity

If the adapter already returned a native UUID/reference, require it unchanged when present.

Mutable startup fields are not immutable identity and may advance between launch return and binding:

- Starting → Running
- PID publication
- native UUID publication when the launch return legitimately precedes it
- terminal publication

Binding reads the latest durable lifecycle in the transaction.

A Starting Session is valid if its immutable identity and ownership are exact.

The following cannot bind:

- missing Session record
- Lost Session
- malformed Session
- foreign scope
- actor mismatch
- role mismatch
- worktree mismatch
- provider mismatch
- duplicate provider/native UUID owned by a different Session in the same scope
- ambiguous or conflicting immutable identity

No cross-project native UUID lookup grants ownership.

## 6. Provider-native authority preservation

This primitive deliberately does not invent a common provider completion fence.

It preserves existing provider-specific guards.

### 6.1 Claude Reviewer

A held Reviewer turn remains eligible for final decision publication through its existing scoped reservation / `commit_current` currency check after Workflow binding.

Expected property:

```
valid bind → Task.version unchanged → reviewer decision fence remains current
```

### 6.2 Claude Executor

A held native permission request remains eligible for an exact Broker ALLOW publication through the existing scoped reservation before the exact permission reply is written.

DENY remains a separate required control because denial grants no operation authority and must not be blocked merely by parent-currency rules that apply to ALLOW.

Binding must not relax Claude native auth, hooks, trust or explicit Broker opt-in policy.

### 6.3 Grok

Existing `Actor::owner` / scope recheck remains authoritative at:

- startup
- native callback handling
- final native result

Valid Workflow binding must not invalidate those checks because it does not rewrite Task/Project/Goal/lock currency.

### 6.4 Codex

Existing `answer_approval` current-scope and Session guards remain authoritative before the exact native operation reply.

Do not infer final decision-currency semantics from transport completion alone.

## 7. Prepared input / Context Pack integration

Issue #19 owns typed prepared-input allocation and closure.

This design composes with it as follows:

- public Workflow `session_id` is only a factual reference
- it is never sufficient to prove private prepared-input ownership
- every fresh native binding validates the exact private allocated Session owner and input pair in the same transaction
- binding does not rewrite Context Pack authority or its native-fenced record
- binding does not change the expected Session CAS/admission pins used by #19
- closure still validates its private owner independently

Until #19 integration is merged, tests must not claim typed-slot coverage that is not present.

## 8. Interaction with Issue #41

Issue #41 owns pre-dispatch preparation reservation behavior.

The two contracts are distinct:

```
#41
reservation exists
→ observer cannot prematurely clear live preparation

#43
native launch returned Session
→ Workflow can register Session without invalidating native currency
```

A held launch must demonstrate:

1. reservation remains owned under passive observation
2. adapter returns exact Session
3. #43 binding succeeds
4. Task.version does not change
5. native caller remains current
6. exactly one actual native dispatch occurred

Binding failure preserves the original reservation and any still-running Session.

It never calls native/phase retry, release or recovery automatically. Rollback-confirmed
transient storage binding may retry the SAME private identity/frame; NormalReturned
returns StartedBindingDeferred while its genuine managed invocation retains ownership.

## 9. StateOnly semantics

Session binding is factual registration through the dedicated private record-only port. It must not call ordinary `WorkflowAccess::StateOnly` persistence, which rewrites Task.

Do not repurpose reserve-time ReadOnly/Mutating admission sweeps for this operation.

Reason:

- the operation occurs after native launch
- it must not reject a legitimate returned Executor merely because unrelated Reviewer/Session state exists
- it must not grant any new native operation authority

Other scoped Sessions and locks are read only as currency/identity facts where required. They are neither adopted nor completed.

## 10. Concurrency behavior

### 10.1 Competing binder

Two binders race for the same attempt:

- only the transaction matching the exact Workflow version and absent `session_id` wins
- loser observes version/session mismatch
- loser writes no audit

### 10.2 Task change

A second writer changes Task/version before binding:

- binding fails
- Session remains reserved/live as previously owned
- no retry/release occurs

### 10.3 Project or Goal change

Version/activity mismatch fails binding without mutation.

### 10.4 Own Session CAS race

If the persisted returned Session identity/lifecycle relevant to binding changes inconsistently during the race, binding fails closed.

Legitimate startup progression that preserves immutable identity is allowed.

### 10.5 WorktreeLock-only race

A scoped lock version/identity change must fail independently even when Task is unchanged.

This prevents Task-version tests from masking lock currency.

### 10.6 Sibling Sessions

Binding does not write sibling Sessions.

Tests compare sibling Session body and version before/after directly; own-Session CAS is not sufficient evidence.

## 11. Audit event

Append one bounded factual event on success.

Exact binding audit fields, maximum4KiB encoded (Scope uses its three owning IDs):

```text
kind = rrx.private.workflow.session_bound
project_id
goal_id
task_id
workflow_id
generation
attempt_index
phase
session_id
provider
actor
role
workflow_version_before
workflow_version_after
task_version_preserved
dispatch_started = true
marker_identity = scope / workflow ID / captured Workflow version / generation / active index / context version
context_version
proof_source = normal_return | closed_settlement
private_operation_ref
private_receipt_ref = absent for normal_return, exact genuine receipt for closed_settlement
original_marker_frame_sha256
workflow_body_sha256_before
workflow_body_sha256_after
prior_ledger_digest = marker anchor for first binding, preceding private link digest otherwise
canonical_body_recipe = rrx.workflow-body-sha256/v1
timestamp = existing AuditEvent.at
```

Do not copy prompts, transcripts, credentials or native protocol payloads.

A failed binding emits no success audit. Diagnostic failure logging may use existing bounded non-authoritative channels.
Public audit/audit_if_current refuse this reserved kind. The mechanics use this same bounded payload, including
workflow_version_before and task_version_preserved;
there is no separate marker UUID beyond the exact versioned tuple above.

## 12. Workflow Engine integration

Replace the current generic persistence path used solely for post-launch Session binding.

Current conceptual behavior:

```
attempt.session_id = returned_session
persist(Task + Workflow)
```

New behavior:

```
returned_session = adapter.start(...)
bind_workflow_native_session(private_operation, NormalReturned(returned_identity))
```

After successful bind:

- use the committed Workflow Record returned by the binding transaction, without a newer owner refresh
- continue polling using the bound Session
- preserve captured Task authority for the already-running native turn

Ordinary phase/context transitions retain their atomic paths; the two bound-live poll diagnostic branches use the separate record-only port specified below.

This is not a broad optimization to avoid Task writes elsewhere.

## 13. Failure behavior

Binding errors are typed and conservative.

Suggested categories:

- stale Project
- stale Goal
- stale Task
- stale Workflow
- inactive owner
- wrong generation
- wrong active attempt
- dispatch marker mismatch
- context mismatch
- Session missing
- Session Lost
- Session identity mismatch
- native UUID ambiguity
- private allocation mismatch
- lock mismatch
- forbidden Workflow delta
- audit/storage failure

On any failure:

- preserve reservation
- preserve existing Session state
- do not send native input
- do not send approval reply
- do not claim completion
- do not retry native/phase admission or refresh the original frame
- do not transfer ownership

Rollback-confirmed transient storage retries and bounded uncertain-commit factual
reconciliation use the SAME private frame/identity, with the typed non-failure normal
return and retention rules in the mechanics. Deterministic refusals park with attention.

## 14. Verification plan

Use temporary repositories, scoped worktrees and isolated SQLite databases.

### 14.1 Core transaction controls

Verify successful binding:

- Task body unchanged
- Task.version unchanged
- Project unchanged
- Goal unchanged
- own Session unchanged
- sibling Sessions unchanged
- WorktreeLocks unchanged
- one Workflow version increment
- only active attempt `session_id` changes
- exactly one factual audit appended

### 14.2 Stale/foreign controls

Each independently fails with no DB/audit mutation:

- Project version change
- Goal version change
- Task version change
- Workflow version change
- inactive Project/Goal/Task
- generation mismatch
- active attempt mismatch
- context pointer mismatch
- dispatch marker mismatch
- actor mismatch
- role mismatch
- worktree mismatch
- provider mismatch
- missing Session
- Lost Session
- duplicate/ambiguous native UUID
- lock-only version change
- forbidden extra Workflow delta

### 14.3 Real native held-turn composition

At the immutable integrated revision, exercise the required actual consumers from the Issue requirements:

| Provider / role | Held-turn evidence after binding |
| --- | --- |
| Claude Reviewer | decision completion via current scoped reservation |
| Claude Executor | exact Broker ALLOW publication before permission reply; DENY control |
| Grok | startup, callback and final result ownership stages |
| Codex Executor | approval scope/Session guard before exact operation reply |

Initial input before `start` returns does not count as post-binding evidence.

If a real Workflow caller cannot expose one path, document that concrete limitation rather than substituting a fake claim.

## 15. Mutation plan

Compile causal mutants for:

- restoring generic Task rewrite during binding
- omitting Project CAS
- omitting Goal CAS
- omitting Task CAS
- omitting Workflow CAS
- omitting latest own Session identity/lifecycle checks, plus a distinct native-consumer own-Session CAS mutant
- omitting scoped WorktreeLock fence
- accepting wrong actor/role/worktree/provider
- accepting duplicate native UUID ambiguity
- admitting extra Workflow changes
- allowing binding without exact dispatch marker
- allowing stale context pointer

A meaningful mutant must be killed by an actual consumer or transaction invariant, not merely a mirrored helper assertion.

Restore exact production bytes after mutation runs and re-run controls.

## 16. Documentation updates

Before merge:

- keep Issue #43 requirements authoritative
- add this issue design
- update Workflow master design with the factual native binding transaction
- record exact verification evidence in an Issue #43 verification document
- note provider-specific unavailable guarantees rather than inventing uniform behavior

## 17. Non-goals

This issue does not implement:

- general restart recovery (actual #14 is required for the narrow authentic settled-success route)
- unknown dispatch retry
- external side-effect reconciliation
- Review Set slot allocation
- general scheduler behavior (actual #23 durable readiness is required for this factual route)
- provider permission-policy changes
- native PID/UUID adoption
- broad Task-write optimization
- an independent schema history outside the composed writer epoch
- cross-project native UUID ownership
- native completion inference

## 18. Transaction implementation details and gate status

See [binding mechanics](issue-43-binding-mechanics.md) for the same proposed
design's concrete Engine/Store boundary, complete-JSON delta validation, trusted
registry probe, exact full scoped lock-set capture/comparison, and audit writer.
The complete lock expectation is captured INSIDE the dispatch-marker transaction with its resulting P/G/T/Workflow versions before start;
binding never refreshes revoked owner snapshots to make a CAS pass.

The allowed Record write includes its ordinary updated_at as well as version;
all other Record metadata and Workflow JSON remain exact. Use one factual binding
audit instead of also appending workflow.saved. Every other record writer retains
its existing audit behavior.

The latest durable Session is identity/lifecycle checked inside Immediate. Its
legitimate startup progress is not rejected by a stale returned-record version.
The own-Session CAS mutation requirement also exercises the provider's actual
currency consumer; it does not introduce a contradictory stale-start CAS.

For every fresh native Executor/Reviewer binding, #19 private allocation/input-pair
checks are mandatory inside
this transaction. A missing private predicate is an integration/merge blocker,
never a permissive fallback. Starting preparation is allocation proof, not ACK;
restored prior terminal snapshots cannot bind as new input owners. No typed
acceptance is claimed until source integration and the native matrix pass.

Failure prohibitions apply to actions by the binder: it sends no additional
input, turn or approval and performs no retry, release or transfer. Existing
owned native supervision and explicit denial keep their provider-native controls;
this method neither grants their authority nor pretends to suspend already
dispatched work. Current fresh-start Workflow has no resumed-UUID predecessor
exemption; any future exception requires verified private resume lineage.

Design4/item6 was approved at77da799. Requirements9 is approved at91d4f34; both Design7 documents remain proposed pending independent STRICT design/source review. No production implementation is present.

The narrow binding primitive is the only writer of existing PhaseAttempt Session
IDs. All ordinary WorkflowAccess modes refuse an existing ID delta; new native
attempts still begin unbound. The latest durable UUID, including a Some published
after a None return, is the key for same-scope ambiguity checks. Sibling live
Reviewer/Lost/executor-reserved facts are positive unchanged-binding controls;
separate operation admission still refuses the same unsafe condition before and
after binding. See the mechanics for the causal control/mutant matrix.

## 19. Impact analysis at the design baseline

Primary locations at public12f545f unless explicitly pinned otherwise:

- workflow.rs:879 reserves a native attempt with None; :1016 is the sole production
  assignment. validate_transition:2139 and :2148–2155 currently permit active
  None-to-Some; :2250 requires new attempts to begin unbound. Both ordinary
  permissions become reject-only for ID deltas; retain the latter initializer
  guard and check every WorkflowAccess caller.
- state/mod.rs:355 put_workflow_transition, :516 ordinary validator and :522–570
  closure guard compose separately with #19. The new binder owns registration;
  ordinary transitions retain closure proof and Task writes.
- state/mod.rs:1166–1236 shared record writer affects Session/Review/Approval/
  WorktreeLock/Workflow audit formatting. Its default callers stay
  exact. Session.saved adds only bounded operation_intent/transport_intent facts
  beside the canonical consumed DTO; other default payloads stay exact. reserved_audit_kind:1575 must include the selected new binding kind.
- workflow/tests.rs:574 is the only current reader that counts workflow.saved;
  change its per-Workflow-write invariant to count saved plus session_bound, and
  preserve separate gate_observed evidence counts. :3267 public reserved-audit
  tests gain the new kind. Other event queries preserve names and provenance.
- adapter.rs:259–301 registry/from_config only validates names, so the new probe
  gate is observable. GenericCliAdapter probe:576 uses generic-cli, matching its
  Session provider; FakeAgent probe/start in workflow/tests.rs use fake. Native
  Grok public65aa940 probe:353 uses grok; Claude publicf9b671f probe:936 and Codex
  public08979bd probe:1106 use their own provider strings. Check the exact provider
  writers and positive generic/fake/native controls in the implementation revision.
- Native5/6/7 currency and permission consumers retain the requirements matrix.
  Their current locks are captured/validated rather than written by native start;
  any changed private signature or prebinding lock writer in the combined source
  requires a new impact check, not an assumed compatible stale snapshot.
- Schema6 allocation/pairs are #19's source dependency. Use co-integration with
  both commit ancestries, combined source/native acceptance and exact CI before
  ready PR(s) merge; neither issue depends on the other's completed merge.

## 20. Design3 admission and fixture impact

[Binding mechanics](issue-43-binding-mechanics.md#actual-admission-writers-and-fixture-migration)
inventories all five known launch implementations: GenericCliAdapter, shared
Workflow FakeAgent, Claude, Codex and Grok. Each must write the real #19 private
allocation/preparation/admitted pair through NativeCAS, including the complete
P/G/T and scoped-lock CAS. ObservationOnly Session persistence cannot create
that authority. Generic/Fake are consumers of the same Workflow native phase;
there is no provider-class exemption or test-only production bypass.

Add implementation-owned Capability::PreparedInputAdmission, absent by default
and not configurable. Resolve/probe/role/capability check before the native phase
reservation, context publication and dispatch marker (current step:860, rather
than current prepare_agent:957). Unsupported adapters return an explicit error
without phase/claim/Session/audit or native child; they do not call fail. Existing
policy/source invalidation remains a separate earlier path. A false capability
advertiser that starts without a real private pair still fails binding after the
marker and stays held under the existing error rule. Actual proof remains solely
Store-derived. Migration of the default generic path is part of the implementation,
not an accepted MVP limitation.

The main baseline has sixteen ordinary transition test calls, two unbound
PhaseAttempt constructors and one shared FakeAgent. Bound fixtures come through
Engine; direct PhaseAttempt.session_id test seeding was not found. Migrate the
common start/bind fixture through actual #19 and the narrow port. Re-inventory
#41's extended FakeAgent/barrier/terminal tests at the combined immutable head.
Unchanged bound IDs keep their ordinary closure/Task-write path. Test every
WorkflowAccess against None-to-Some, Some-to-None and Some-to-different changes
in active and nonactive entries; no fixture exemption.

Staged #43 production fails closed without #19 and earns no positive binding
coverage. Positive generic/fake/native controls, sibling success, startup
progression and the native matrix all run on the actual co-integration revision.
Use one combined PR preserving both source ancestries, reviewed/tested with
exact-head CI, before either issue closes. Main never receives the ready binding
port without the real #19 predicate; no completed-merge dependency cycle.
The separate ordinary closure checks stay in their ordinary transaction, while
binding proof lives only in the #43 port. Update master design to implementation
fact only in the combined source PR.


## 21. Preserve the consumed input through native operation decisions

Design3 launch-admission inventory is verified, but actual Claude permission and
terminal paths also overwrite recovery.dispatch_intent. That key must contain
only #19's canonical consumed PreparedInput DTO. Move permission/transport facts
to distinct bounded journals and migrate their actual audit/assertion/SQL-trigger
consumers. The mechanics inventory explicit and automatic Claude ALLOW/DENY,
Codex pre/postreply writes and Grok callback/input publications, with separate
NativeCAS versus Acked/ConsumedHistorical modes. Operation observation cannot
allocate, consume or overwrite private input authority. Existing ALLOW fences,
DENY own-Session/state constraints and Lost absorption remain mandatory.
Protected Tasks reject Interactive/PTY startup and terminal input before any
reservation, Starting row, spawn or terminal bytes; NonInteractive Workflow
configuration alone does not protect the direct native adapter entry points.

Design4 and Design7 proposal reviews were approved; Design8 actual #19/#23/#14 composition and independent review remain pending;
source remains absent. Positive post-binding decision and later typed closure
must retain the exact consumed UUID/pair. A new Journal name is not itself proof
of correctness; actual reply/closure controls and causal mutants are required.

## 22. Design8 composition and diagnostic contract

[Binding mechanics](issue-43-binding-mechanics.md) contains the detailed Design8
transaction, managed driver, diagnostic and extended control/mutation contracts.
Requirements9 approved at91d4f34 governs both documents. Normal return and authentic
settled-success registration use the SAME immutable full frame captured atomically
INSIDE marker commit; no later parent/lock recapture. Genuine #19 current successful
receipt/profile cleanup is required for the late path. Readiness derives from durable
marked/unbound/phase-open operations, with actual #23 owned launch supervision independent
of dropped Engine futures, per-Project durable claim capacity reserved atomically inside #41 reserve and parked
Held observations. Lost
notification is recovered by bounded fair active-driver wake, never passive poll.

The separate record-only bound-live diagnostic port changes only bounded active detail,
Workflow version/updated_at and a reserved audit; no P/G/T/Session/lock or native pin
write. Consumers use current-Record CAS PLUS the bounded sealed factual W-successor
ledger anchored at the immutable marker, unchanged private pins and genuine receipts.
Original P/G/T/source/context/full-lock/native grant pins never refresh. Actual held-native status-error and persisted/status mismatch controls
and EACH branch's Task-rewrite mutant are required. No diagnostic binds, fails/retries,
releases or grants. Known actual current failure uses #19 non-success closure without
absent Session-ID binding; actual #14 restore/fencing gates restart. All private source,
required native conformance and co-integration gates remain pending.
