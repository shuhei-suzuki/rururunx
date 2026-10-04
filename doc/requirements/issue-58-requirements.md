# Issue 58: Retained ownership of Task-free native Consultant work

Workflow: STRICT (native ownership, durable authority and Project exclusion).
Status: Requirements1; requirements/design/source gates pending.
Baseline: main80452f4/schema3. MVP follow-up from #6 F1 and #19; see
[Issue58](https://github.com/shuhei-suzuki/rururunx/issues/58).

## Purpose and current gap

Required native consultation/attach must not let actual owned startup, native work
or cleanup disappear behind an ordinary terminal Session label. Project removal,
root/common-Git-directory mutations and competing admission must observe retained
ownership even before a first Session exists, after a turn ends while its server
remains owned, or when cleanup/outcome is unknown.

Actual main Store::ensure_project_idle1300–1331 uses terminal Session labels;
generic put_record_tx freezes Session actor/worktree identity but does not attest
cleanup. git::executor_reserved359–368 only counts Executor roles. These are
existing foundations, not a private Consultant hold/settlement producer.
Public #19 Design19
[02e9ea2](https://github.com/shuhei-suzuki/rururunx/blob/02e9ea2d7276973eeae7ac7efb9b3025eed9aad8/doc/design/issue-19-design.md)
proposes managed Task phase operations and excludes Task-free Consultant from
those operations; it is pending review/source, not this feature's implementation.
Public #6 design's proposed native TUI gateway is Project-only Consultant,
sole-upstream actor, text/decision-only and per-submission CAS. Interactive human
results do not prove Workflow transport success. Preserve that scope.

## Ownership and supported scope

- Scope is exact Project and optional Goal, with Task absent. Bind to canonical
  primary Project.root, repository/common-dir identity, registered implementation,
  native profile, actor/role, mode and native session/turn identities. Overlapping
  Issue names, paths or native IDs in another Project grant no authority.
- Acquisition requires actual registered Project/current versions and applicable
  Goal authority/lifecycle under #23. A Goal-less consultation does not fabricate
  a Goal or Task. No Task-worktree path/namespace, Executor/Reviewer/ApprovalReviewer
  conversion or Task dispatch/completion permit is introduced.
- Actual local runtime composition selects supported profiles; native/model JSON,
  public role names, configuration self-labels or first-use success cannot enroll
  a producer. Every fresh Task-free native entry requires this actual retained-owner
  port before effects. The shared Store denies generic nonterminal native Session
  insertion/update, including terminal-to-live and identical-body version bumps,
  regardless of public role/path; a direct adapter or generic record wrapper cannot
  evade this fence. Initial terminal factual history grants no launch/release proof.
  Each implemented profile must have reviewed native conformance for
  its complete required startup/turn/service/cleanup workload under #6 F1.
  Unsupported profiles reject before reserve/process/native-visible effect;
  required Consult/attach acceptance stays open rather than waived.
- Before ANY native startup/probe/discovery side effect, the actual owned supervisor
  must receive an opaque, nonserializable retained-owner capability captured from
  successful exact scoped acquisition. A durable row alone, supplied PID, copied
  UUID or claimed runtime identity cannot stand in for the live owner.
- Durable identity/lifetime and input/source pins are immutable. Transfer ownership
  once to the actual supervisor; do not clone a terminal-release capability or let
  an untrusted collector/observer manufacture it. Generic Session/Record writers
  cannot hide or mutate its owner-bound factual Session, including identical-body
  version bumps; actual supervisor observations have a dedicated private path.
  Metrics and bounded audit remain separate and cannot grant release/admission.
- Retain Session-less setup, pending input, live service, native background work,
  dropped futures, publication conflicts and Lost/unknown outcomes. A completed
  native turn does not release a still-owned server or the consultation lifetime.
  Missing Session/PID, terminal public labels, process-group hints, interrupt/close
  acknowledgements and configuration claims are not cleanup proof.

## Exclusion, settlement and recovery

- Define the actual root/common-dir conflict scope in design from complete effect
  inventory, including preserved mandatory native hooks/filesystem and delegation.
  A Consultant label is not proof of read-only behavior. Reject conflicting native/
  Git/write admission and Project removal while retained work may affect that
  scope. Independent Projects remain runnable; no runtime-global blanket hold.
  The exclusion query also includes actual Task-managed ownership from #19 with
  reachable shared root/common-Git effects, including Lost/uncertain Task work;
  Task identity or separate worktree never proves disjoint shared effects.
- Every actual consumer reads retained ownership under its current authority/CAS:
  Project removal/mutation, root/common-dir Git/worktree operations, native and
  input admission, applicable Goal completion/pack publication, capacity and restart
  reconciliation. Enumerate exact call sites before source. Do not rely solely on
  Session state, executor_reserved or a cached ready/status result.
- An exact owner may perform its separately allowed factual observations, denial
  and cleanup without treating its own hold as an unrelated competing owner.
  Native/Source/Git calls remain outside SQLite/SharedStore locks. Inactive-owner
  closure never grants new native work or relaxes native current-input/Broker guards.
- Release is private actual supervisor evidence of no dispatch/settled setup, or
  known exact native outcome AND all profile-required resource cleanup. It pins the
  same immutable operation/scope, input/Session versions and actual owner; publication
  and permitted release commit atomically with bounded audit. Unknown/Lost/native
  escape or failed publication keeps the hold and exposes attention.
- Keep input admission, native completion, resource settlement and accepted Goal/
  Task success distinct. A Consultant receipt cannot close a Workflow phase,
  certify a review, resolve a criterion or release another owner.
- Runtime restart never reconstructs a live owner from persisted hints. Retain
  unknown ownership until separately reviewed #14 recovery proves actual cleanup
  or safely fenced no-effect and excludes the former owner. Human labels/boot hints
  cannot fabricate death or unlock replay. Lost remains absorbing in ordinary APIs.
- Migration coordinates one schema/writer-epoch history with #19, fences ALL generic
  and private writers including old open connections, and refuses unresolved legacy
  workload. Preserve actual old bytes/history; no legacy terminal-label ratification.
  Final production profile readiness includes actual #14 recovery, not fake receipts.

## Native TUI and continuation

- Reuse #6's Project-only native TUI gateway: one actor/upstream connection, exact
  stored native session, bounded text/decision inputs and one unresolved submission.
  Every human/native-originated inference submission and operation grant uses the
  existing actual per-submission source/Session CAS before wire; no second ledger.
- Native-originated steer/compact/review/inference either consumes that same genuine
  current admission or is explicitly unsupported before effect. Historical DENY
  keeps its separately applicable exact request/turn authority; it cannot release
  lifetime ownership or acquire fresh input/operation authority.
- Attach binds only the live exact owning gateway, with its retained hold. A dead/
  terminal seed does not reopen by ID. Continuation uses a separately validated
  exact prior owned outcome/settlement and fresh admission, with existing native
  UUID/input semantics; unknown cleanup blocks it. Preserve native config/auth/
  model/effort/hook/UI defaults and existing unsupported Task interactive conversions.
- General detached-native-command containment stays #6 F1. This issue consumes a
  real supported cleanup contract; it does not introduce privileged infrastructure,
  change platform scope or assert arbitrary descendant death from group cleanup.

## Bounds, verification and delivery

Design finite ownership/history/turn/reference/serialized/nesting/query bounds and
checked versions, including coherent complete Project-level exclusion scans and
capacity exhaustion. No truncated scan proves idle or permits a release. Persist
only allowlisted identities/digests/versions and bounded reason codes, never secrets,
environment values, native transcripts or prompt text. Status is bounded, scoped,
read-only and honest about uncertain owned work. Trust is the application API;
malicious linked code/direct same-UID machine/DB actions are outside it, not secured
by a biological Human or OS sandbox claim.

Real controlled/private and required native controls cover startup before Session,
retained live server after turn, TUI input/decision ordering, same-scope continuation,
generic terminal/role/owner forgery, conflicting writer/publication failure, dropped
owner, Lost, restart, inactive closure, Project remove and common-dir operations.
Use two isolated Projects; never modify unrelated user repositories.

Compiled causal mutants must reach the actual removal/admission/settlement/TUI
consumers: omit hold lookup, release on terminal label/PID absence, forge owner from
row identity, treat a turn as lifetime settlement, omit root/common-dir exclusion,
or bypass a submission CAS. Controls prove exact settled positives and independent
Project progress. Setup refusal, timeout or helper-only checks are not consumer
kills. Restore exact source and passing controls after each mutation.

Run shared-state/native/Git/registry/Workflow regressions, fmt/clippy/build, exact
Linux/macOS CI and independent immutable source reviews. Update README/master
designs to actual behavior. Dependencies: #19 coordinated private authority/epoch;
#6 profile/TUI conformance; #11 consult driver, #14 recovery, #23 Goal consumers and
#26 Project management. Requirements research is runnable independently; no missing
producer is replaced by a fixture. Keep #58/#6 acceptance open until actual
composition passes; no full #16 dogfood/MVP completion is claimed here.
