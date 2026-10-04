# Issue 43 Design: Preserve native authority during Workflow Session binding

**Status:** Design draft; implementation pending  
**Workflow:** STRICT  
**Scope:** Workflow native Session binding only

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
8. No schema migration is required for this change.

## 3. New private Store primitive

Introduce one crate-private transaction dedicated to Workflow native binding.

Logical shape:

```text
bind_workflow_native_session(
    expected: WorkflowNativeBindingExpectation,
    returned: RegisteredNativeSessionIdentity,
) -> BindingResult
```

The exact Rust names may differ, but the primitive must not be exposed as a general state mutation API.

### 3.1 Expected snapshot

The caller supplies the authority captured before launch:

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
- expected native Session ID returned by the registered adapter
- expected private prepared-input owner/allocation identity when #19 integration is available

### 3.2 Allowed write set

The transaction may change only:

```
Workflow.history[active_attempt].session_id:
    None → exact returned SessionId

Workflow.version:
    n → n + 1

Audit:
    append one factual workflow_native_session_bound event
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
8. load the exact persisted returned Session
9. validate immutable Session identity
10. validate native identity uniqueness within the owning scope
11. validate private prepared-input allocation owner when available
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
- rururunx Session ownership identity

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
- when typed allocation is available, binding validates the exact allocated private Session owner in the same transaction
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

It never calls retry/release/recovery automatically.

## 9. StateOnly semantics

Session binding remains `StateOnly` factual registration.

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

Suggested fields:

```text
kind = workflow_native_session_bound
project_id
goal_id
task_id
workflow_id
generation
attempt_id
session_id
provider
actor
role
workflow_version_before
workflow_version_after
task_version_preserved
dispatch_marker
context_version
timestamp
```

Do not copy prompts, transcripts, credentials or native protocol payloads.

A failed binding emits no success audit. Diagnostic failure logging may use existing bounded non-authoritative channels.

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
bind_workflow_native_session(expected, returned_session)
```

After successful bind:

- reload canonical Workflow state
- continue polling using the bound Session
- preserve captured Task authority for the already-running native turn

All ordinary phase transitions continue using existing Task/Workflow/context atomic transitions.

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
- do not retry
- do not transfer ownership

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
- omitting own Session identity/CAS
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

- restart recovery
- unknown dispatch retry
- external side-effect reconciliation
- Review Set slot allocation
- scheduler behavior
- provider permission-policy changes
- native PID/UUID adoption
- broad Task-write optimization
- new schema
- cross-project native UUID ownership
- native completion inference

## 18. Transaction implementation details and gate status

See [binding mechanics](issue-43-binding-mechanics.md) for the same proposed
design's concrete Engine/Store boundary, complete-JSON delta validation, trusted
registry probe, exact full scoped lock-set capture/comparison, and audit writer.
The lock expectation is captured after dispatch-marker commit and before start;
binding never refreshes revoked owner snapshots to make a CAS pass.

The allowed Record write includes its ordinary updated_at as well as version;
all other Record metadata and Workflow JSON remain exact. Use one factual binding
audit instead of also appending workflow.saved. Every other record writer retains
its existing audit behavior.

The latest durable Session is identity/lifecycle checked inside Immediate. Its
legitimate startup progress is not rejected by a stale returned-record version.
The own-Session CAS mutation requirement also exercises the provider's actual
currency consumer; it does not introduce a contradictory stale-start CAS.

For typed input, #19 private allocation/input-pair checks are mandatory inside
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

Both documents remain draft pending independent STRICT design review. The
requirements gate approved d8c5266; no production implementation is present.
